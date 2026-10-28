//! Réglages clé/valeur typés en JSON (≈ `wp_options`).

use std::collections::BTreeMap;

use serde_json::Value;
use sqlx::PgPool;

use crate::{CoreError, CoreResult};

/// Préfixe des réglages exposés publiquement.
pub const PUBLIC_PREFIX: &str = "site.";

fn valid_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= 100
        && key
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '.')
}

pub async fn all(db: &PgPool) -> CoreResult<BTreeMap<String, Value>> {
    let rows: Vec<(String, Value)> = sqlx::query_as("SELECT key, value FROM settings ORDER BY key")
        .fetch_all(db)
        .await?;
    Ok(rows.into_iter().collect())
}

pub async fn public(db: &PgPool) -> CoreResult<BTreeMap<String, Value>> {
    const ALLOWED: &[&str] = &[
        "site.title",
        "site.tagline",
        "site.url",
        "site.language",
        "site.design_tokens",
        "site.menus",
        "reading.show_on_front",
        "reading.page_on_front",
        "reading.page_for_posts",
        "reading.per_page",
        "reading.posts_per_rss",
        "reading.rss_use_excerpt",
        "reading.blog_public",
        "permalink.structure",
        "permalink.category_base",
        "permalink.tag_base",
        "discussion.default_comment_status",
        "discussion.require_name_email",
        "discussion.comment_registration",
        "discussion.close_comments_for_old_posts",
        "discussion.close_comments_days",
        "discussion.thread_comments",
        "discussion.thread_comments_depth",
        "discussion.page_comments",
        "discussion.comments_per_page",
        "discussion.comment_order",
        "discussion.show_avatars",
        "discussion.avatar_default",
    ];
    let rows: Vec<(String, Value)> = sqlx::query_as(
        "SELECT key, value FROM settings WHERE starts_with(key, $1) OR starts_with(key, 'reading.') OR starts_with(key, 'permalink.') OR starts_with(key, 'discussion.') ORDER BY key",
    )
    .bind(PUBLIC_PREFIX)
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .filter(|(k, _)| ALLOWED.contains(&k.as_str()))
        .map(|(k, v)| (k.trim_start_matches(PUBLIC_PREFIX).to_string(), v))
        .collect())
}

pub async fn get(db: &PgPool, key: &str) -> CoreResult<Option<Value>> {
    Ok(
        sqlx::query_scalar("SELECT value FROM settings WHERE key = $1")
            .bind(key)
            .fetch_optional(db)
            .await?,
    )
}

/// Vérifie la sûreté d'une URL de menu public (`WP-46`).
fn validate_menu_url(url: &str) -> CoreResult<()> {
    let trimmed = url.trim();
    if trimmed.is_empty() || trimmed.len() > 2048 {
        return Err(CoreError::validation(
            "L'URL d'un élément de menu doit contenir entre 1 et 2048 caractères.",
        ));
    }
    if trimmed
        .chars()
        .any(|c| c.is_control() || matches!(c, '"' | '\'' | '<' | '>' | '\\'))
    {
        return Err(CoreError::validation(
            "Caractères interdits dans l'URL du menu.",
        ));
    }
    let lower = trimmed.to_ascii_lowercase();
    let allowed = (trimmed.starts_with('/') && !trimmed.starts_with("//"))
        || trimmed.starts_with('#')
        || lower.starts_with("https://")
        || lower.starts_with("http://")
        || lower.starts_with("mailto:")
        || lower.starts_with("tel:");
    if !allowed {
        return Err(CoreError::validation(
            "Protocole d'URL non autorisé dans le menu public.",
        ));
    }
    Ok(())
}

/// Vérifie la validité d'une liste d'éléments de menu (WP-46, QM-01, QM-02).
fn validate_menu_items(arr: &[Value]) -> CoreResult<()> {
    if arr.len() > 64 {
        return Err(CoreError::validation(
            "Un emplacement de menu est limité à 64 liens.",
        ));
    }

    let mut parent_map = std::collections::HashMap::new();

    for item in arr {
        let Some(item_obj) = item.as_object() else {
            return Err(CoreError::validation(
                "Chaque élément de menu doit être un objet JSON.",
            ));
        };
        let label = item_obj
            .get("label")
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or("");
        if label.is_empty() || label.chars().count() > 80 {
            return Err(CoreError::validation(
                "Le libellé d'un lien de menu est obligatoire (max 80 caractères).",
            ));
        }
        let url = item_obj.get("url").and_then(Value::as_str).unwrap_or("");
        validate_menu_url(url)?;

        if let Some(badge) = item_obj.get("badge").and_then(Value::as_str)
            && badge.chars().count() > 20
        {
            return Err(CoreError::validation(
                "Le badge d'un élément de menu est limité à 20 caractères.",
            ));
        }

        if let Some(subtitle) = item_obj.get("subtitle").and_then(Value::as_str)
            && subtitle.chars().count() > 80
        {
            return Err(CoreError::validation(
                "Le sous-titre d'un élément de menu est limité à 80 caractères.",
            ));
        }

        if let Some(target) = item_obj.get("target").and_then(Value::as_str)
            && !matches!(target, "_self" | "_blank")
        {
            return Err(CoreError::validation(
                "La cible de lien doit être '_self' ou '_blank'.",
            ));
        }

        if let Some(item_type) = item_obj.get("type").and_then(Value::as_str)
            && !matches!(item_type, "custom" | "page" | "category" | "tag")
        {
            return Err(CoreError::validation(
                "Le type d'élément de menu doit être 'custom', 'page', 'category' ou 'tag'.",
            ));
        }

        if let Some(id) = item_obj.get("id").and_then(Value::as_str)
            && let Some(parent_id) = item_obj.get("parent_id").and_then(Value::as_str)
        {
            let trimmed_parent = parent_id.trim();
            if !trimmed_parent.is_empty() {
                if trimmed_parent == id {
                    return Err(CoreError::validation(
                        "Un élément de menu ne peut pas être son propre parent.",
                    ));
                }
                parent_map.insert(id.to_string(), trimmed_parent.to_string());
            }
        }
    }

    // Détection de cycles et contrôle de la profondeur maximale (max 3 niveaux)
    for child_id in parent_map.keys() {
        let mut visited = std::collections::HashSet::new();
        visited.insert(child_id.clone());
        let mut curr = child_id;
        let mut depth = 1;
        while let Some(parent) = parent_map.get(curr) {
            depth += 1;
            if depth > 3 {
                return Err(CoreError::validation(
                    "La profondeur maximale des sous-menus est limitée à 3 niveaux.",
                ));
            }
            if !visited.insert(parent.clone()) {
                return Err(CoreError::validation(
                    "Boucle cyclique détectée dans la hiérarchie des menus.",
                ));
            }
            curr = parent;
        }
    }

    Ok(())
}

/// Valide les réglages structurés critiques (`site.menus` `WP-46` et `site.design_tokens` `WP-48`/`WP-50`).
fn validate_setting_value(key: &str, value: &Value) -> CoreResult<()> {
    if value.is_null() {
        return Ok(());
    }
    if key == "site.menus" {
        let Some(obj) = value.as_object() else {
            return Err(CoreError::validation("site.menus doit être un objet JSON."));
        };
        for (location, items) in obj {
            if !matches!(location.as_str(), "header" | "footer") {
                return Err(CoreError::validation(format!(
                    "Emplacement de menu inconnu « {location} » (attendu : header ou footer)."
                )));
            }
            let Some(arr) = items.as_array() else {
                return Err(CoreError::validation(
                    "Chaque emplacement de menu doit être une liste d'éléments.",
                ));
            };
            validate_menu_items(arr)?;
        }
    } else if key == "site.design_tokens"
        && let Some(obj) = value.as_object()
        && let Some(css) = obj.get("custom_css").and_then(Value::as_str)
    {
        if css.len() > 16 * 1024 {
            return Err(CoreError::validation(
                "Le CSS personnalisé est limité à 16 Kio.",
            ));
        }
        let lower = css.to_ascii_lowercase();
        if css.contains('<')
            || css.contains('>')
            || lower.contains("@import")
            || lower.contains("javascript:")
            || lower.contains("expression(")
        {
            return Err(CoreError::validation(
                "Le CSS personnalisé contient une directive ou balise interdite (<, >, @import, javascript:, expression).",
            ));
        }
    } else if key == "reading.show_on_front" {
        let Some(mode) = value.as_str() else {
            return Err(CoreError::validation(
                "reading.show_on_front doit être 'posts' ou 'page'.",
            ));
        };
        if !matches!(mode, "posts" | "page") {
            return Err(CoreError::validation(
                "reading.show_on_front doit être 'posts' ou 'page'.",
            ));
        }
    } else if key == "reading.per_page" {
        let Some(n) = value.as_i64() else {
            return Err(CoreError::validation(
                "reading.per_page doit être un entier entre 1 et 100.",
            ));
        };
        if !(1..=100).contains(&n) {
            return Err(CoreError::validation(
                "reading.per_page doit être compris entre 1 et 100.",
            ));
        }
    } else if key == "reading.posts_per_rss" {
        let Some(n) = value.as_i64() else {
            return Err(CoreError::validation(
                "reading.posts_per_rss doit être un entier entre 1 et 100.",
            ));
        };
        if !(1..=100).contains(&n) {
            return Err(CoreError::validation(
                "reading.posts_per_rss doit être compris entre 1 et 100.",
            ));
        }
    } else if key == "reading.rss_use_excerpt" && !value.is_boolean() {
        return Err(CoreError::validation(
            "reading.rss_use_excerpt doit être un booléen.",
        ));
    } else if key == "reading.blog_public" && !value.is_boolean() {
        return Err(CoreError::validation(
            "reading.blog_public doit être un booléen.",
        ));
    } else if key == "users.default_role" {
        let Some(role) = value.as_str() else {
            return Err(CoreError::validation(
                "users.default_role doit être 'editor', 'author' ou 'subscriber'.",
            ));
        };
        if !matches!(role, "editor" | "author" | "subscriber") {
            return Err(CoreError::validation(
                "Le rôle par défaut ne peut pas être 'admin' (attendu : editor, author ou subscriber).",
            ));
        }
    } else if key == "users.can_register" && !value.is_boolean() {
        return Err(CoreError::validation(
            "users.can_register doit être un booléen.",
        ));
    } else if key == "site.admin_email" {
        let Some(email) = value.as_str() else {
            return Err(CoreError::validation(
                "site.admin_email doit être une chaîne.",
            ));
        };
        let trimmed = email.trim();
        let valid = !trimmed.is_empty()
            && trimmed.len() <= 254
            && trimmed.split_once('@').is_some_and(|(local, domain)| {
                !local.is_empty()
                    && domain.contains('.')
                    && !domain.starts_with('.')
                    && !domain.ends_with('.')
            });
        if !valid {
            return Err(CoreError::validation(
                "Adresse e-mail d'administration invalide.",
            ));
        }
    } else if key == "permalink.structure" {
        let Some(s) = value.as_str() else {
            return Err(CoreError::validation(
                "permalink.structure doit être une chaîne.",
            ));
        };
        let trimmed = s.trim();
        if trimmed.is_empty()
            || matches!(
                trimmed,
                "plain" | "day_and_name" | "month_and_name" | "numeric" | "post_name"
            )
        {
            return Ok(());
        }
        if trimmed.len() > 256 {
            return Err(CoreError::validation(
                "La structure de permalien est limitée à 256 caractères.",
            ));
        }
        if !trimmed.starts_with('/') {
            return Err(CoreError::validation(
                "Une structure de permalien personnalisée doit commencer par un slash « / ».",
            ));
        }
        if !trimmed.contains("%postname%") && !trimmed.contains("%post_id%") {
            return Err(CoreError::validation(
                "La structure de permalien personnalisée doit contenir au moins %postname% ou %post_id%.",
            ));
        }
        const ALLOWED_TOKENS: &[&str] = &[
            "%year%", "%monthnum%", "%day%", "%hour%", "%minute%", "%second%",
            "%post_id%", "%postname%", "%category%", "%author%",
        ];
        let mut rest = trimmed;
        while let Some(start) = rest.find('%') {
            let after_start = &rest[start + 1..];
            if let Some(end) = after_start.find('%') {
                let token = &rest[start..=start + 1 + end];
                if !ALLOWED_TOKENS.contains(&token) {
                    return Err(CoreError::validation(format!(
                        "Jeton de permalien inconnu « {token} »."
                    )));
                }
                rest = &after_start[end + 1..];
            } else {
                return Err(CoreError::validation(
                    "Jeton de permalien mal formé (délimiteur '%' non fermé).",
                ));
            }
        }
        if trimmed.chars().any(|c| c.is_control() || matches!(c, '<' | '>' | '"' | '\'' | '\\' | '?' | '#')) {
            return Err(CoreError::validation(
                "La structure de permalien contient des caractères interdits.",
            ));
        }
    } else if key == "permalink.category_base" {
        let Some(s) = value.as_str() else {
            return Err(CoreError::validation(
                "permalink.category_base doit être une chaîne.",
            ));
        };
        let trimmed = s.trim();
        if !trimmed.is_empty() {
            if trimmed.len() > 64 {
                return Err(CoreError::validation(
                    "Le préfixe de catégorie ne peut pas dépasser 64 caractères.",
                ));
            }
            crate::security::safe_slug(trimmed).map_err(|_| {
                CoreError::validation(
                    "Le préfixe de catégorie doit être un slug valide (minuscules, chiffres, tirets).",
                )
            })?;
            const RESERVED: &[&str] = &[
                "api", "admin", "uploads", "public", "feed", "search", "archive", "archives", "zen", "post", "page",
            ];
            if RESERVED.contains(&trimmed) && trimmed != "category" {
                return Err(CoreError::validation(format!(
                    "Le préfixe de catégorie « {trimmed} » est un mot réservé par le système."
                )));
            }
        }
    } else if key == "permalink.tag_base" {
        let Some(s) = value.as_str() else {
            return Err(CoreError::validation(
                "permalink.tag_base doit être une chaîne.",
            ));
        };
        let trimmed = s.trim();
        if !trimmed.is_empty() {
            if trimmed.len() > 64 {
                return Err(CoreError::validation(
                    "Le préfixe d'étiquette ne peut pas dépasser 64 caractères.",
                ));
            }
            crate::security::safe_slug(trimmed).map_err(|_| {
                CoreError::validation(
                    "Le préfixe d'étiquette doit être un slug valide (minuscules, chiffres, tirets).",
                )
            })?;
            const RESERVED: &[&str] = &[
                "api", "admin", "uploads", "public", "feed", "search", "archive", "archives", "zen", "post", "page",
            ];
            if RESERVED.contains(&trimmed) && trimmed != "tag" {
                return Err(CoreError::validation(format!(
                    "Le préfixe d'étiquette « {trimmed} » est un mot réservé par le système."
                )));
            }
        }
    } else if key == "discussion.default_comment_status"
        && !value.is_boolean()
        && !value.as_str().is_some_and(|s| matches!(s, "open" | "closed"))
    {
        return Err(CoreError::validation(
            "discussion.default_comment_status doit être un booléen ou 'open'/'closed'.",
        ));
    } else if key == "discussion.require_name_email" && !value.is_boolean() {
        return Err(CoreError::validation("discussion.require_name_email doit être un booléen."));
    } else if key == "discussion.comment_registration" && !value.is_boolean() {
        return Err(CoreError::validation("discussion.comment_registration doit être un booléen."));
    } else if key == "discussion.close_comments_for_old_posts" && !value.is_boolean() {
        return Err(CoreError::validation("discussion.close_comments_for_old_posts doit être un booléen."));
    } else if key == "discussion.close_comments_days" {
        let Some(n) = value.as_i64() else {
            return Err(CoreError::validation("discussion.close_comments_days doit être un entier positif."));
        };
        if !(1..=3650).contains(&n) {
            return Err(CoreError::validation("discussion.close_comments_days doit être compris entre 1 et 3650 jours."));
        }
    } else if key == "discussion.thread_comments" && !value.is_boolean() {
        return Err(CoreError::validation("discussion.thread_comments doit être un booléen."));
    } else if key == "discussion.thread_comments_depth" {
        let Some(n) = value.as_i64() else {
            return Err(CoreError::validation("discussion.thread_comments_depth doit être un entier entre 1 et 5."));
        };
        if !(1..=5).contains(&n) {
            return Err(CoreError::validation("discussion.thread_comments_depth doit être compris entre 1 et 5 niveaux."));
        }
    } else if key == "discussion.page_comments" && !value.is_boolean() {
        return Err(CoreError::validation("discussion.page_comments doit être un booléen."));
    } else if key == "discussion.comments_per_page" {
        let Some(n) = value.as_i64() else {
            return Err(CoreError::validation("discussion.comments_per_page doit être un entier entre 1 et 100."));
        };
        if !(1..=100).contains(&n) {
            return Err(CoreError::validation("discussion.comments_per_page doit être compris entre 1 et 100."));
        }
    } else if key == "discussion.comment_order" {
        let Some(order) = value.as_str() else {
            return Err(CoreError::validation("discussion.comment_order doit être 'asc' ou 'desc'."));
        };
        if !matches!(order, "asc" | "desc") {
            return Err(CoreError::validation("discussion.comment_order doit être 'asc' ou 'desc'."));
        }
    } else if key == "discussion.comment_moderation" && !value.is_boolean() {
        return Err(CoreError::validation("discussion.comment_moderation doit être un booléen."));
    } else if key == "discussion.comment_previously_approved" && !value.is_boolean() {
        return Err(CoreError::validation("discussion.comment_previously_approved doit être un booléen."));
    } else if key == "discussion.comment_max_links" {
        let Some(n) = value.as_i64() else {
            return Err(CoreError::validation("discussion.comment_max_links doit être un entier positif ou nul."));
        };
        if !(0..=50).contains(&n) {
            return Err(CoreError::validation("discussion.comment_max_links doit être compris entre 0 et 50."));
        }
    } else if (key == "discussion.moderation_keys" || key == "discussion.disallowed_keys")
        && let Some(text) = value.as_str()
        && text.len() > 16 * 1024
    {
        return Err(CoreError::validation("La liste de modération ou de mots interdits est limitée à 16 Kio."));
    } else if key == "discussion.show_avatars" && !value.is_boolean() {
        return Err(CoreError::validation("discussion.show_avatars doit être un booléen."));
    } else if key == "discussion.avatar_default" && !value.is_string() {
        return Err(CoreError::validation("discussion.avatar_default doit être une chaîne."));
    }
    Ok(())
}

/// Met à jour plusieurs réglages ; une valeur `null` supprime la clé.
pub async fn update(
    db: &PgPool,
    values: BTreeMap<String, Value>,
) -> CoreResult<BTreeMap<String, Value>> {
    if let Some(bad) = values.keys().find(|k| !valid_key(k)) {
        return Err(CoreError::validation(format!(
            "clé de réglage invalide « {bad} »"
        )));
    }
    for (k, v) in &values {
        validate_setting_value(k, v)?;
    }
    if let (Some(cat), Some(tag)) = (
        values.get("permalink.category_base").and_then(Value::as_str),
        values.get("permalink.tag_base").and_then(Value::as_str),
    ) {
        let cat = cat.trim();
        let tag = tag.trim();
        if !cat.is_empty() && !tag.is_empty() && cat == tag {
            return Err(CoreError::validation(
                "Le préfixe de catégorie et le préfixe d'étiquette ne peuvent pas être identiques.",
            ));
        }
    }
    let mut tx = db.begin().await?;
    if values.keys().any(|k| {
        matches!(
            k.as_str(),
            "reading.show_on_front" | "reading.page_on_front" | "reading.page_for_posts"
        )
    }) {
        sqlx::query("SELECT pg_advisory_xact_lock(72430651)")
            .execute(&mut *tx)
            .await?;
        let rows: Vec<(String, Value)> =
            sqlx::query_as("SELECT key,value FROM settings WHERE key LIKE 'reading.%'")
                .fetch_all(&mut *tx)
                .await?;
        let mut merged: BTreeMap<String, Value> = rows.into_iter().collect();
        merged.extend(values.clone());
        let mut selected = Vec::new();
        for key in ["reading.page_on_front", "reading.page_for_posts"] {
            let value = merged.get(key).unwrap_or(&Value::Null);
            if value.is_null() || value.as_str() == Some("") {
                selected.push(None);
                continue;
            }
            let id = value
                .as_str()
                .and_then(|s| uuid::Uuid::parse_str(s).ok())
                .ok_or_else(|| {
                    CoreError::validation(
                        "La page choisie doit être un identifiant de page valide.",
                    )
                })?;
            let valid: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM entries e JOIN content_types ct ON ct.id=e.content_type_id WHERE e.id=$1 AND ct.slug='page' AND e.status='published' AND e.published_at<=now() AND (e.scheduled_unpublish_at IS NULL OR e.scheduled_unpublish_at>now()))").bind(id).fetch_one(&mut *tx).await?;
            if !valid {
                return Err(CoreError::validation(
                    "Choisissez une page publiée et actuellement visible.",
                ));
            }
            selected.push(Some(id));
        }
        if merged.get("reading.show_on_front").and_then(Value::as_str) == Some("page") {
            if selected[0].is_none() {
                return Err(CoreError::validation("Choisissez une page d’accueil."));
            }
            if selected[0] == selected[1] {
                return Err(CoreError::validation(
                    "L’accueil et la page des articles doivent être distincts.",
                ));
            }
        }
    }
    for (key, value) in values {
        if value.is_null() {
            sqlx::query("DELETE FROM settings WHERE key = $1")
                .bind(&key)
                .execute(&mut *tx)
                .await?;
        } else {
            sqlx::query(
                "INSERT INTO settings (key, value) VALUES ($1, $2)
                 ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = now()",
            )
            .bind(&key)
            .bind(&value)
            .execute(&mut *tx)
            .await?;
        }
    }
    tx.commit().await?;
    all(db).await
}

/// Retourne l'état d'activation des modules natifs de RustForge
pub async fn get_active_modules(db: &PgPool) -> CoreResult<BTreeMap<String, bool>> {
    let raw = get(db, "modules.active").await?;
    if let Some(val) = raw
        && let Ok(map) = serde_json::from_value::<BTreeMap<String, bool>>(val)
    {
        return Ok(map);
    }
    let mut default_map = BTreeMap::new();
    default_map.insert("forms".to_string(), true);
    default_map.insert("galleries".to_string(), true);
    default_map.insert("zen_mode".to_string(), true);
    default_map.insert("ecommerce".to_string(), true);
    default_map.insert("lms".to_string(), true);
    Ok(default_map)
}

/// Vérifie si un module spécifique est activé
pub async fn is_module_active(db: &PgPool, module: &str) -> bool {
    let modules = get_active_modules(db).await.unwrap_or_default();
    *modules.get(module).unwrap_or(&false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_reading_rss_settings() {
        assert!(validate_setting_value("reading.posts_per_rss", &json!(10)).is_ok());
        assert!(validate_setting_value("reading.posts_per_rss", &json!(1)).is_ok());
        assert!(validate_setting_value("reading.posts_per_rss", &json!(100)).is_ok());
        assert!(validate_setting_value("reading.posts_per_rss", &json!(0)).is_err());
        assert!(validate_setting_value("reading.posts_per_rss", &json!(101)).is_err());
        assert!(validate_setting_value("reading.posts_per_rss", &json!("10")).is_err());

        assert!(validate_setting_value("reading.rss_use_excerpt", &json!(true)).is_ok());
        assert!(validate_setting_value("reading.rss_use_excerpt", &json!(false)).is_ok());
        assert!(validate_setting_value("reading.rss_use_excerpt", &json!("true")).is_err());
        assert!(validate_setting_value("reading.rss_use_excerpt", &json!(1)).is_err());
    }

    #[test]
    fn test_validate_permalink_settings() {
        // Standard presets
        for preset in ["plain", "day_and_name", "month_and_name", "numeric", "post_name", ""] {
            assert!(validate_setting_value("permalink.structure", &json!(preset)).is_ok());
        }

        // Custom structure with valid tokens
        assert!(validate_setting_value("permalink.structure", &json!("/%year%/%postname%/")).is_ok());
        assert!(validate_setting_value("permalink.structure", &json!("/archives/%post_id%")).is_ok());
        assert!(validate_setting_value("permalink.structure", &json!("/blog/%category%/%postname%")).is_ok());

        // Custom structure missing required post identifier
        assert!(validate_setting_value("permalink.structure", &json!("/%year%/%monthnum%/")).is_err());

        // Custom structure without leading slash
        assert!(validate_setting_value("permalink.structure", &json!("%year%/%postname%")).is_err());

        // Custom structure with unknown token
        assert!(validate_setting_value("permalink.structure", &json!("/%unknown%/%postname%")).is_err());

        // Category base
        assert!(validate_setting_value("permalink.category_base", &json!("rubrique")).is_ok());
        assert!(validate_setting_value("permalink.category_base", &json!("category")).is_ok());
        assert!(validate_setting_value("permalink.category_base", &json!("")).is_ok());
        assert!(validate_setting_value("permalink.category_base", &json!("api")).is_err()); // Reserved
        assert!(validate_setting_value("permalink.category_base", &json!("bad/slash")).is_err()); // Slashes forbidden

        // Tag base
        assert!(validate_setting_value("permalink.tag_base", &json!("mot-cle")).is_ok());
        assert!(validate_setting_value("permalink.tag_base", &json!("tag")).is_ok());
        assert!(validate_setting_value("permalink.tag_base", &json!("")).is_ok());
        assert!(validate_setting_value("permalink.tag_base", &json!("admin")).is_err()); // Reserved
    }

    #[test]
    fn test_validate_hierarchical_menus() {
        // Flat legacy menu is valid
        let flat_menu = json!({
            "header": [
                {"label": "Accueil", "url": "/fr"},
                {"label": "Contact", "url": "/fr/contact"}
            ],
            "footer": []
        });
        assert!(validate_setting_value("site.menus", &flat_menu).is_ok());

        // Valid hierarchical menu with parent_id, badge, subtitle, target
        let tree_menu = json!({
            "header": [
                {"id": "1", "label": "Services", "url": "/fr/services"},
                {"id": "2", "parent_id": "1", "label": "Conseil", "url": "/fr/conseil", "badge": "Pro", "subtitle": "Audit d'architecture"},
                {"id": "3", "parent_id": "2", "label": "Sécurité", "url": "/fr/securite"}
            ],
            "footer": []
        });
        assert!(validate_setting_value("site.menus", &tree_menu).is_ok());

        // Self-referencing parent rejected
        let self_parent = json!({
            "header": [
                {"id": "1", "parent_id": "1", "label": "Boucle", "url": "/fr/boucle"}
            ]
        });
        assert!(validate_setting_value("site.menus", &self_parent).is_err());

        // Cyclic loop A -> B -> A rejected
        let cycle_menu = json!({
            "header": [
                {"id": "A", "parent_id": "B", "label": "A", "url": "/fr/a"},
                {"id": "B", "parent_id": "A", "label": "B", "url": "/fr/b"}
            ]
        });
        assert!(validate_setting_value("site.menus", &cycle_menu).is_err());

        // Depth exceeding 3 levels rejected (level 1 -> 2 -> 3 -> 4)
        let deep_menu = json!({
            "header": [
                {"id": "1", "label": "N1", "url": "/1"},
                {"id": "2", "parent_id": "1", "label": "N2", "url": "/2"},
                {"id": "3", "parent_id": "2", "label": "N3", "url": "/3"},
                {"id": "4", "parent_id": "3", "label": "N4", "url": "/4"}
            ]
        });
        assert!(validate_setting_value("site.menus", &deep_menu).is_err());
    }

    #[test]
    fn test_validate_discussion_settings() {
        assert!(validate_setting_value("discussion.default_comment_status", &json!("open")).is_ok());
        assert!(validate_setting_value("discussion.default_comment_status", &json!(true)).is_ok());
        assert!(validate_setting_value("discussion.default_comment_status", &json!("invalid")).is_err());

        assert!(validate_setting_value("discussion.thread_comments_depth", &json!(3)).is_ok());
        assert!(validate_setting_value("discussion.thread_comments_depth", &json!(0)).is_err());
        assert!(validate_setting_value("discussion.thread_comments_depth", &json!(6)).is_err());

        assert!(validate_setting_value("discussion.comment_order", &json!("asc")).is_ok());
        assert!(validate_setting_value("discussion.comment_order", &json!("desc")).is_ok());
        assert!(validate_setting_value("discussion.comment_order", &json!("random")).is_err());

        assert!(validate_setting_value("discussion.comment_max_links", &json!(2)).is_ok());
        assert!(validate_setting_value("discussion.comment_max_links", &json!(-1)).is_err());
    }
}
