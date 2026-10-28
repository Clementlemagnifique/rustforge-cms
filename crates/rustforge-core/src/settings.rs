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
        "reading.per_page",
        "reading.blog_public",
    ];
    let rows: Vec<(String, Value)> = sqlx::query_as(
        "SELECT key, value FROM settings WHERE starts_with(key, $1) OR starts_with(key, 'reading.') ORDER BY key",
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
            if arr.len() > 32 {
                return Err(CoreError::validation(
                    "Un emplacement de menu est limité à 32 liens.",
                ));
            }
            for item in arr {
                let label = item
                    .get("label")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .unwrap_or("");
                if label.is_empty() || label.chars().count() > 80 {
                    return Err(CoreError::validation(
                        "Le libellé d'un lien de menu est obligatoire (max 80 caractères).",
                    ));
                }
                let url = item.get("url").and_then(Value::as_str).unwrap_or("");
                validate_menu_url(url)?;
            }
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
                !local.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
            });
        if !valid {
            return Err(CoreError::validation(
                "Adresse e-mail d'administration invalide.",
            ));
        }
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
    let mut tx = db.begin().await?;
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
