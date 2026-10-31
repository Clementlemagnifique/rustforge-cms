//! Services de gestion des commentaires, des discussions et de la modération souveraine.
//!
//! Couvre les exigences :
//! - `WP-42` : Création, attente, approbation, spam, corbeille, restauration et réponses.
//! - `WP-43` : Filtres par statut, recherche, actions groupées en masse et pagination.
//! - `WP-44` : Confidentialité des données auteurs (exclusion des e-mails/IPs des vues publiques).
//! - `WP-45` : Antispam, contrôle du nombre de liens, liste de modération et liste noire.
//! - `WP-82` à `WP-87` : Réglages de discussion (héritage, conditions, emboîtement, avatars).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::error::{CoreError, CoreResult};
use crate::models::{Comment, CommentStatus, PublicCommentView};

/// Paramètres pour la création d'un commentaire.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateCommentParams {
    pub entry_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub author_id: Option<Uuid>,
    pub author_name: String,
    pub author_email: String,
    pub author_url: Option<String>,
    pub author_ip: Option<String>,
    pub author_user_agent: Option<String>,
    pub content: String,
}

/// Paramètres de filtrage pour le listage des commentaires (Studio).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CommentFilter {
    pub status: Option<CommentStatus>,
    pub entry_id: Option<Uuid>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Statistiques de modération (compteurs par onglet).
#[derive(Debug, Clone, Serialize, Default, FromRow)]
pub struct CommentCounts {
    pub all: i64,
    pub pending: i64,
    pub approved: i64,
    pub spam: i64,
    pub trash: i64,
}

/// Contrat public partagé entre API et rendu : aucune donnée privée de réglage.
#[derive(Debug, Clone, Serialize)]
pub struct DiscussionPolicy {
    pub open: bool,
    pub require_identity: bool,
    pub require_registration: bool,
    pub threaded: bool,
    pub depth: usize,
    pub newest_first: bool,
    pub show_avatars: bool,
    pub avatar_default: String,
    pub remember_identity: bool,
    pub close_old: bool,
    pub close_days: i64,
    pub paginated: bool,
    pub per_page: i64,
    pub default_last: bool,
}

impl DiscussionPolicy {
    pub fn from_settings(values: &std::collections::BTreeMap<String, serde_json::Value>) -> Self {
        let flag = |key: &str, default| {
            values
                .get(key)
                .and_then(|v| v.as_bool().or_else(|| v.as_str().map(|s| s == "open")))
                .unwrap_or(default)
        };
        let number =
            |key: &str, default| values.get(key).and_then(|v| v.as_i64()).unwrap_or(default);
        Self {
            open: flag("discussion.default_comment_status", true),
            require_identity: flag("discussion.require_name_email", true),
            require_registration: flag("discussion.comment_registration", false),
            threaded: flag("discussion.thread_comments", true),
            depth: number("discussion.thread_comments_depth", 3).clamp(1, 5) as usize,
            newest_first: values
                .get("discussion.comment_order")
                .and_then(|v| v.as_str())
                == Some("desc"),
            show_avatars: flag("discussion.show_avatars", true),
            avatar_default: values
                .get("discussion.avatar_default")
                .and_then(|v| v.as_str())
                .unwrap_or("mystery")
                .into(),
            remember_identity: flag("discussion.show_comments_cookies_opt_in", false),
            close_old: flag("discussion.close_comments_for_old_posts", false),
            close_days: number("discussion.close_comments_days", 14).max(0),
            paginated: flag("discussion.page_comments", false),
            per_page: number("discussion.comments_per_page", 50).clamp(1, 100),
            default_last: values
                .get("discussion.default_comments_page")
                .and_then(|v| v.as_str())
                == Some("newest"),
        }
    }
    /// Le choix local remplace seulement l’ouverture par défaut ; les autres protections restent actives.
    pub fn with_entry_fields(mut self, fields: &serde_json::Value) -> Self {
        match fields.get("_rf_comment_status").and_then(|v| v.as_str()) {
            Some("open") => self.open = true,
            Some("closed") => self.open = false,
            _ => {}
        }
        self
    }

    pub fn can_submit(&self, published_at: Option<DateTime<Utc>>) -> bool {
        self.open
            && !(self.close_old
                && published_at.is_some_and(|date| {
                    Utc::now() >= date + chrono::Duration::days(self.close_days.min(365000))
                }))
    }
}

/// SVG local dérivé uniquement de l'UUID public du commentaire, jamais de l'e-mail.
pub fn avatar_svg(style: &str, id: Uuid) -> String {
    if style == "blank" {
        return String::new();
    }
    let bytes = id.as_bytes();
    let hue = u32::from(bytes[0]) * 360 / 256;
    let color = format!("hsl({hue},55%,45%)");
    let mut svg = String::from(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 16 16\" width=\"32\" height=\"32\" aria-hidden=\"true\">",
    );
    svg.push_str("<rect width=\"16\" height=\"16\" rx=\"2\" fill=\"#e2e8f0\"/>");
    if matches!(style, "identicon" | "retro") {
        let size = if style == "retro" { 8 } else { 5 };
        let cell = 14.0 / f64::from(size);
        for y in 0..size {
            for x in 0..size {
                let mirror_x = x.min(size - 1 - x);
                let seed = (y * 4 + mirror_x) as usize;
                if bytes[seed % 16] & (1 << (seed % 8)) != 0 {
                    svg.push_str(&format!("<rect x=\"{}\" y=\"{}\" width=\"{cell}\" height=\"{cell}\" fill=\"{color}\"/>",1.0+f64::from(x)*cell,1.0+f64::from(y)*cell));
                }
            }
        }
    } else {
        svg.push_str(&format!("<circle cx=\"8\" cy=\"5\" r=\"3\" fill=\"{color}\"/><path d=\"M2 15a6 6 0 0 1 12 0\" fill=\"{color}\"/>"));
    }
    svg.push_str("</svg>");
    svg
}

/// Seuls les liens web absolus sont acceptés pour un auteur public.
fn validated_author_url(raw: Option<&str>) -> CoreResult<Option<String>> {
    let Some(raw) = raw.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(None);
    };
    if raw.len() > 2048 || raw.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err(CoreError::validation("URL auteur invalide"));
    }
    let url = reqwest::Url::parse(raw).map_err(|_| CoreError::validation("URL auteur invalide"))?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(CoreError::validation("URL auteur non autorisée"));
    }
    Ok(Some(url.to_string()))
}

/// Nettoie et échappe le contenu d'un commentaire en HTML sécurisé.
pub fn format_comment_html(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let mut escaped = String::with_capacity(trimmed.len() + 32);
    for c in trimmed.chars() {
        match c {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#x27;"),
            _ => escaped.push(c),
        }
    }

    let normalized = escaped.replace("\r\n", "\n").replace('\r', "\n");
    let paragraphs: Vec<&str> = normalized.split("\n\n").collect();

    let mut out = String::new();
    for p in paragraphs {
        let trimmed_p = p.trim();
        if !trimmed_p.is_empty() {
            out.push_str("<p>");
            out.push_str(&trimmed_p.replace('\n', "<br>"));
            out.push_str("</p>\n");
        }
    }

    if out.is_empty() { escaped } else { out }
}

/// Compte le nombre de liens web dans le contenu (détection d'URLs pour antispam).
pub fn count_links(content: &str) -> usize {
    let lower = content.to_ascii_lowercase();
    let http_count = lower.matches("http://").count();
    let https_count = lower.matches("https://").count();
    let www_count = lower
        .matches("www.")
        .count()
        .saturating_sub(lower.matches("http://www").count() + lower.matches("https://www").count());
    http_count + https_count + www_count
}

/// Vérifie si une chaîne contient l'un des termes d'une liste noire (séparés par retours à la ligne).
pub fn contains_term(haystack: &str, terms_list: &str) -> bool {
    let h_lower = haystack.to_ascii_lowercase();
    for line in terms_list.lines() {
        let term = line.trim().to_ascii_lowercase();
        if !term.is_empty() && h_lower.contains(&term) {
            return true;
        }
    }
    false
}

/// Évalue automatiquement le statut initial d'un commentaire selon les réglages de discussion.
pub async fn evaluate_comment_status(
    db: &PgPool,
    author_name: &str,
    author_email: &str,
    author_url: Option<&str>,
    author_ip: Option<&str>,
    content: &str,
) -> CoreResult<CommentStatus> {
    let all_settings = crate::settings::all(db).await?;

    let get_bool = |key: &str, default: bool| -> bool {
        all_settings
            .get(key)
            .and_then(|v| v.as_bool())
            .unwrap_or(default)
    };
    let get_str = |key: &str| -> String {
        all_settings
            .get(key)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };
    let get_i64 = |key: &str, default: i64| -> i64 {
        all_settings
            .get(key)
            .and_then(|v| v.as_i64())
            .unwrap_or(default)
    };

    let disallowed_keys = get_str("discussion.disallowed_keys");
    let moderation_keys = get_str("discussion.moderation_keys");
    let max_links = get_i64("discussion.comment_max_links", 2);
    let require_manual = get_bool("discussion.comment_moderation", false);
    let require_previously_approved = get_bool("discussion.comment_previously_approved", true);

    let inspect_target = format!(
        "{author_name} {author_email} {} {} {content}",
        author_url.unwrap_or(""),
        author_ip.unwrap_or("")
    );

    // 1. Liste noire : si un terme interdit est trouvé, marqué comme spam direct
    if !disallowed_keys.trim().is_empty() && contains_term(&inspect_target, &disallowed_keys) {
        return Ok(CommentStatus::Spam);
    }

    // 2. Trop de liens -> modération
    if max_links > 0 && count_links(content) >= max_links as usize {
        return Ok(CommentStatus::Pending);
    }

    // 3. Terme de modération trouvé -> modération
    if !moderation_keys.trim().is_empty() && contains_term(&inspect_target, &moderation_keys) {
        return Ok(CommentStatus::Pending);
    }

    // 4. Modération manuelle obligatoire
    if require_manual {
        return Ok(CommentStatus::Pending);
    }

    // 5. Approbation si commentaire précédent approuvé
    if require_previously_approved {
        if author_email.is_empty() {
            return Ok(CommentStatus::Pending);
        }
        let has_approved: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM comments WHERE author_email = $1 AND status = 'approved')",
        )
        .bind(author_email)
        .fetch_one(db)
        .await?;

        if has_approved {
            return Ok(CommentStatus::Approved);
        }
        return Ok(CommentStatus::Pending);
    }

    Ok(CommentStatus::Approved)
}

/// Crée un nouveau commentaire avec validation et formatage.
pub async fn create_comment(db: &PgPool, params: CreateCommentParams) -> CoreResult<Comment> {
    let author_url = validated_author_url(params.author_url.as_deref())?;
    let entry = crate::entries::get_published_by_id(db, params.entry_id).await?;
    let policy = DiscussionPolicy::from_settings(&crate::settings::all(db).await?)
        .with_entry_fields(&entry.fields);
    let name = params.author_name.trim();
    let email = params.author_email.trim();
    let content = params.content.trim();

    if (policy.require_identity && name.is_empty()) || name.chars().count() > 100 {
        return Err(CoreError::validation(
            "Le nom de l'auteur est requis (max 100 caractères).",
        ));
    }
    if (policy.require_identity && email.is_empty())
        || email.chars().count() > 254
        || (!email.is_empty() && !email.contains('@'))
    {
        return Err(CoreError::validation("L'adresse e-mail est invalide."));
    }
    if content.is_empty() {
        return Err(CoreError::validation(
            "Le contenu du commentaire ne peut pas être vide.",
        ));
    }
    if content.len() > 65_536 {
        return Err(CoreError::validation(
            "Le contenu du commentaire est limité à 64 Kio.",
        ));
    }

    // Même visibilité que les pages et API publiques (dates comprises).
    let published_at = entry.published_at;

    if !policy.can_submit(published_at) {
        return Err(CoreError::validation(
            "Les commentaires sont fermés pour cette publication.",
        ));
    }
    if policy.require_registration && params.author_id.is_none() {
        return Err(CoreError::Forbidden);
    }
    let name = if name.is_empty() { "Visiteur" } else { name };
    if params.parent_id.is_some() && !policy.threaded {
        return Err(CoreError::validation(
            "Les réponses imbriquées sont désactivées.",
        ));
    }

    // Vérifier le parent_id s'il est spécifié
    if let Some(pid) = params.parent_id {
        let depth: Option<i32> = sqlx::query_scalar(
            "WITH RECURSIVE ancestors AS (
                SELECT id, parent_id, 1 AS depth FROM comments
                WHERE id=$1 AND entry_id=$2 AND status='approved'
                UNION ALL
                SELECT c.id, c.parent_id, a.depth+1 FROM comments c
                JOIN ancestors a ON c.id=a.parent_id WHERE a.depth < 6
            ) SELECT max(depth) FROM ancestors",
        )
        .bind(pid)
        .bind(params.entry_id)
        .fetch_one(db)
        .await?;
        if depth.is_none_or(|depth| depth as usize >= policy.depth) {
            return Err(CoreError::validation(
                "Parent inaccessible ou profondeur maximale atteinte.",
            ));
        }
    }

    let status = evaluate_comment_status(
        db,
        name,
        email,
        params.author_url.as_deref(),
        params.author_ip.as_deref(),
        content,
    )
    .await?;

    let content_html = format_comment_html(content);

    let mut tx = db.begin().await?;
    let comment: Comment = sqlx::query_as(
        r#"
        INSERT INTO comments (
            entry_id, parent_id, author_id, author_name, author_email,
            author_url, author_ip, author_user_agent, content, content_html, status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING
            id, entry_id, parent_id, author_id, author_name, author_email,
            author_url, author_ip, author_user_agent, content, content_html,
            status, created_at, updated_at,
            NULL::text as entry_title, NULL::text as entry_slug
        "#,
    )
    .bind(params.entry_id)
    .bind(params.parent_id)
    .bind(params.author_id)
    .bind(name)
    .bind(email)
    .bind(author_url.as_deref())
    .bind(
        params
            .author_ip
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty()),
    )
    .bind(
        params
            .author_user_agent
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty()),
    )
    .bind(content)
    .bind(content_html)
    .bind(status.as_str())
    .fetch_one(&mut *tx)
    .await?;

    crate::comment_notifications::enqueue_in(&mut tx, &comment, params.author_id).await?;
    tx.commit().await?;

    Ok(comment)
}

/// Récupère un commentaire par son identifiant.
pub async fn get_comment(db: &PgPool, id: Uuid) -> CoreResult<Comment> {
    sqlx::query_as(
        r#"
        SELECT
            c.id, c.entry_id, c.parent_id, c.author_id, c.author_name, c.author_email,
            c.author_url, c.author_ip, c.author_user_agent, c.content, c.content_html,
            c.status, c.created_at, c.updated_at,
            e.title as entry_title, e.slug as entry_slug
        FROM comments c
        LEFT JOIN entries e ON e.id = c.entry_id
        WHERE c.id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await?
    .ok_or(CoreError::NotFound)
}

/// Liste les commentaires avec filtres et pagination.
pub async fn list_comments(db: &PgPool, filter: CommentFilter) -> CoreResult<(Vec<Comment>, i64)> {
    let limit = filter.limit.unwrap_or(20).clamp(1, 100);
    let offset = filter.offset.unwrap_or(0).max(0);

    let status_str = filter.status.map(|s| s.as_str().to_string());
    let search_pattern = filter.search.as_ref().map(|s| format!("%{}%", s.trim()));

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM comments c
        LEFT JOIN entries e ON e.id = c.entry_id
        WHERE ($1::text IS NULL OR c.status = $1)
          AND ($2::uuid IS NULL OR c.entry_id = $2)
          AND ($3::text IS NULL OR (c.content ILIKE $3 OR c.author_name ILIKE $3 OR c.author_email ILIKE $3))
        "#,
    )
    .bind(&status_str)
    .bind(filter.entry_id)
    .bind(&search_pattern)
    .fetch_one(db)
    .await?;

    let items: Vec<Comment> = sqlx::query_as(
        r#"
        SELECT
            c.id, c.entry_id, c.parent_id, c.author_id, c.author_name, c.author_email,
            c.author_url, c.author_ip, c.author_user_agent, c.content, c.content_html,
            c.status, c.created_at, c.updated_at,
            e.title as entry_title, e.slug as entry_slug
        FROM comments c
        LEFT JOIN entries e ON e.id = c.entry_id
        WHERE ($1::text IS NULL OR c.status = $1)
          AND ($2::uuid IS NULL OR c.entry_id = $2)
          AND ($3::text IS NULL OR (c.content ILIKE $3 OR c.author_name ILIKE $3 OR c.author_email ILIKE $3))
        ORDER BY c.created_at DESC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(&status_str)
    .bind(filter.entry_id)
    .bind(&search_pattern)
    .bind(limit)
    .bind(offset)
    .fetch_all(db)
    .await?;

    Ok((items, total))
}

/// Met à jour le statut d'un commentaire (modération unitaire).
pub async fn update_comment_status(
    db: &PgPool,
    id: Uuid,
    new_status: CommentStatus,
) -> CoreResult<Comment> {
    update_comment_status_as(db, id, new_status, None).await
}

pub async fn update_comment_status_as(
    db: &PgPool,
    id: Uuid,
    new_status: CommentStatus,
    actor: Option<Uuid>,
) -> CoreResult<Comment> {
    let mut tx = db.begin().await?;
    let result = change_status_in(&mut tx, id, new_status.as_str(), actor)
        .await?
        .ok_or(CoreError::NotFound)?;
    tx.commit().await?;
    Ok(result.0)
}

async fn change_status_in(
    connection: &mut sqlx::PgConnection,
    id: Uuid,
    status: &str,
    actor: Option<Uuid>,
) -> CoreResult<Option<(Comment, usize, bool)>> {
    let old: Option<Comment> = sqlx::query_as("SELECT * FROM comments WHERE id=$1 FOR UPDATE")
        .bind(id)
        .fetch_optional(&mut *connection)
        .await?;
    let Some(old) = old else {
        return Ok(None);
    };
    if old.status == status {
        return Ok(Some((old, 0, false)));
    }
    let comment: Comment = sqlx::query_as(
        "UPDATE comments SET status=$2,updated_at=clock_timestamp() WHERE id=$1 RETURNING *",
    )
    .bind(id)
    .bind(status)
    .fetch_one(&mut *connection)
    .await?;
    let count = crate::comment_notifications::enqueue_in(connection, &comment, actor).await?;
    Ok(Some((comment, count, true)))
}

/// Modération atomique bornée ; ordre stable des verrous pour éviter les interblocages.
pub async fn batch_update_status(
    db: &PgPool,
    ids: &[Uuid],
    new_status: CommentStatus,
) -> CoreResult<u64> {
    batch_update_status_as(db, ids, new_status, None).await
}

pub async fn batch_update_status_as(
    db: &PgPool,
    ids: &[Uuid],
    new_status: CommentStatus,
    actor: Option<Uuid>,
) -> CoreResult<u64> {
    if ids.len() > 100 {
        return Err(CoreError::validation(
            "Lot de modération limité à 100 commentaires",
        ));
    }
    let mut ids = ids.to_vec();
    ids.sort_unstable();
    ids.dedup();
    let mut tx = db.begin().await?;
    let mut changed = 0;
    let mut notifications = 0;
    for id in ids {
        if let Some((_, count, did_change)) =
            change_status_in(&mut tx, id, new_status.as_str(), actor).await?
        {
            notifications += count;
            if notifications > 1000 {
                return Err(CoreError::validation("Lot limité à 1000 notifications"));
            }
            changed += u64::from(did_change);
        }
    }
    tx.commit().await?;
    Ok(changed)
}

/// Modifie le contenu d'un commentaire depuis le Studio.
pub async fn update_comment_content(
    db: &PgPool,
    id: Uuid,
    content: &str,
    author_name: Option<&str>,
    author_email: Option<&str>,
    author_url: Option<&str>,
) -> CoreResult<Comment> {
    update_comment_content_as(db, id, content, author_name, author_email, author_url, None).await
}

pub async fn update_comment_content_as(
    db: &PgPool,
    id: Uuid,
    content: &str,
    author_name: Option<&str>,
    author_email: Option<&str>,
    author_url: Option<&str>,
    actor: Option<Uuid>,
) -> CoreResult<Comment> {
    let author_url = validated_author_url(author_url)?;
    let trimmed_content = content.trim();
    if trimmed_content.is_empty() {
        return Err(CoreError::validation("Le contenu ne peut pas être vide."));
    }
    let content_html = format_comment_html(trimmed_content);

    let mut tx = db.begin().await?;
    let previous: Option<Comment> = sqlx::query_as("SELECT * FROM comments WHERE id=$1 FOR UPDATE")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;
    let previous = previous.ok_or(CoreError::NotFound)?;
    let next_name = author_name
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(&previous.author_name);
    let next_email = author_email
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(&previous.author_email);
    if previous.content == trimmed_content
        && previous.author_name == next_name
        && previous.author_email == next_email
        && previous.author_url == author_url
    {
        return Ok(previous);
    }

    let comment: Comment = sqlx::query_as(
        r#"
        UPDATE comments
        SET content = $2,
            content_html = $3,
            author_name = COALESCE($4, author_name),
            author_email = COALESCE($5, author_email),
            author_url = $6,
            updated_at = clock_timestamp()
        WHERE id = $1
        RETURNING
            id, entry_id, parent_id, author_id, author_name, author_email,
            author_url, author_ip, author_user_agent, content, content_html,
            status, created_at, updated_at,
            NULL::text as entry_title, NULL::text as entry_slug
        "#,
    )
    .bind(id)
    .bind(trimmed_content)
    .bind(content_html)
    .bind(author_name.map(str::trim).filter(|s| !s.is_empty()))
    .bind(author_email.map(str::trim).filter(|s| !s.is_empty()))
    .bind(author_url.as_deref())
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(CoreError::NotFound)?;

    crate::comment_notifications::enqueue_in(&mut tx, &comment, actor).await?;
    tx.commit().await?;

    Ok(comment)
}

/// Réponse éditoriale depuis le Studio : crée automatiquement une réponse approuvée.
pub async fn reply_to_comment(
    db: &PgPool,
    parent_id: Uuid,
    author_id: Uuid,
    author_name: &str,
    author_email: &str,
    content: &str,
) -> CoreResult<Comment> {
    let parent = get_comment(db, parent_id).await?;
    let content_html = format_comment_html(content);

    let mut tx = db.begin().await?;
    let reply: Comment = sqlx::query_as(
        r#"
        INSERT INTO comments (
            entry_id, parent_id, author_id, author_name, author_email,
            author_url, content, content_html, status
        )
        VALUES ($1, $2, $3, $4, $5, NULL, $6, $7, 'approved')
        RETURNING
            id, entry_id, parent_id, author_id, author_name, author_email,
            author_url, author_ip, author_user_agent, content, content_html,
            status, created_at, updated_at,
            NULL::text as entry_title, NULL::text as entry_slug
        "#,
    )
    .bind(parent.entry_id)
    .bind(parent_id)
    .bind(author_id)
    .bind(author_name)
    .bind(author_email)
    .bind(content)
    .bind(content_html)
    .fetch_one(&mut *tx)
    .await?;

    crate::comment_notifications::enqueue_in(&mut tx, &reply, Some(author_id)).await?;
    tx.commit().await?;

    Ok(reply)
}

/// Supprime définitivement un commentaire de la base de données.
pub async fn delete_comment(db: &PgPool, id: Uuid) -> CoreResult<bool> {
    let _ = crate::mail::purge_captures_for_comment(db, id).await;
    let res = sqlx::query("DELETE FROM comments WHERE id = $1")
        .bind(id)
        .execute(db)
        .await?;
    Ok(res.rows_affected() > 0)
}

/// Supprime définitivement tous les commentaires placés dans la corbeille.
pub async fn empty_trash(db: &PgPool) -> CoreResult<u64> {
    let res = sqlx::query("DELETE FROM comments WHERE status = 'trash'")
        .execute(db)
        .await?;
    Ok(res.rows_affected())
}

/// Compte les commentaires par statut pour alimenter les compteurs d'onglets du Studio.
pub async fn count_by_status(db: &PgPool) -> CoreResult<CommentCounts> {
    let counts: CommentCounts = sqlx::query_as(
        r#"
        SELECT
            COUNT(*)::bigint as all,
            COUNT(*) FILTER (WHERE status = 'pending')::bigint as pending,
            COUNT(*) FILTER (WHERE status = 'approved')::bigint as approved,
            COUNT(*) FILTER (WHERE status = 'spam')::bigint as spam,
            COUNT(*) FILTER (WHERE status = 'trash')::bigint as trash
        FROM comments
        "#,
    )
    .fetch_one(db)
    .await?;

    Ok(counts)
}

/// Une page compte les fils racines ; les réponses restent avec leur fil.
#[derive(Debug, Serialize)]
pub struct PublicCommentPage {
    pub items: Vec<PublicCommentView>,
    pub page: i64,
    pub total_pages: i64,
    pub total: i64,
    pub root_total: i64,
}

pub async fn get_entry_comment_page(
    db: &PgPool,
    entry_id: Uuid,
    requested_page: Option<i64>,
) -> CoreResult<PublicCommentPage> {
    crate::entries::get_published_by_id(db, entry_id).await?;
    let policy = DiscussionPolicy::from_settings(&crate::settings::all(db).await?);
    if requested_page.is_some_and(|p| !(1..=1_000_000).contains(&p)) {
        return Err(CoreError::validation(
            "Numéro de page de commentaires invalide",
        ));
    }
    let mut tx = db.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .execute(&mut *tx)
        .await?;
    // L'agrégat et la page partagent le même snapshot ; seuls les fils visibles sont comptés.
    let (root_total, total): (i64, i64) = sqlx::query_as(
        "WITH RECURSIVE visible AS (
            SELECT id, id AS root_id FROM comments
            WHERE entry_id=$1 AND status='approved' AND (NOT $2 OR parent_id IS NULL)
            UNION ALL
            SELECT c.id, v.root_id FROM comments c JOIN visible v ON c.parent_id=v.id
            WHERE $2 AND c.entry_id=$1 AND c.status='approved'
        ) SELECT count(*) FILTER(WHERE id=root_id), count(*) FROM visible",
    )
    .bind(entry_id)
    .bind(policy.threaded)
    .fetch_one(&mut *tx)
    .await?;
    let size = if policy.paginated {
        policy.per_page
    } else {
        2000
    };
    let total_pages = if policy.paginated {
        ((root_total + size - 1) / size).max(1)
    } else {
        1
    };
    let page = requested_page.unwrap_or(if policy.default_last { total_pages } else { 1 });
    if page > total_pages {
        return Err(CoreError::NotFound);
    }
    if !policy.paginated && root_total > 2000 {
        return Err(CoreError::validation(
            "Activez la pagination pour afficher ce volume de commentaires.",
        ));
    }
    #[derive(sqlx::FromRow)]
    struct FlatCommentNode {
        id: Uuid,
        parent_id: Option<Uuid>,
        author_name: String,
        author_url: Option<String>,
        content_html: String,
        created_at: DateTime<Utc>,
    }

    let mut nodes: Vec<FlatCommentNode> = sqlx::query_as(
        "WITH RECURSIVE roots AS (
            SELECT id FROM comments WHERE entry_id=$1 AND status='approved'
                AND (NOT $2 OR parent_id IS NULL)
            ORDER BY CASE WHEN NOT $5 THEN created_at END ASC, CASE WHEN $5 THEN created_at END DESC,
                CASE WHEN NOT $5 THEN id END ASC, CASE WHEN $5 THEN id END DESC
            LIMIT $3 OFFSET $4
        ), visible AS (
            SELECT c.id,c.parent_id,c.author_name,c.author_url,c.content_html,c.created_at
            FROM comments c JOIN roots r ON c.id=r.id
            UNION ALL
            SELECT c.id,c.parent_id,c.author_name,c.author_url,c.content_html,c.created_at
            FROM comments c JOIN visible v ON c.parent_id=v.id
            WHERE $2 AND c.entry_id=$1 AND c.status='approved'
        ) SELECT * FROM visible ORDER BY created_at ASC, id ASC LIMIT 2001")
        .bind(entry_id).bind(policy.threaded).bind(size).bind((page-1)*size)
        .bind(policy.newest_first).fetch_all(&mut *tx).await?;
    tx.commit().await?;
    if nodes.len() > 2000 {
        return Err(CoreError::validation(
            "Le volume de ce fil dépasse la limite de rendu (2000 commentaires).",
        ));
    }
    if policy.newest_first {
        nodes.reverse();
    }
    let visible_depth = if policy.threaded { policy.depth } else { 1 };
    let parents: std::collections::HashMap<_, _> =
        nodes.iter().map(|n| (n.id, n.parent_id)).collect();
    for node in &mut nodes {
        let mut chain = Vec::new();
        let mut parent = node.parent_id;
        while let Some(id) = parent {
            if chain.contains(&id) || chain.len() >= 64 {
                break;
            }
            chain.push(id);
            parent = parents.get(&id).copied().flatten();
        }
        if visible_depth == 1 {
            node.parent_id = None;
        } else if chain.len() >= visible_depth {
            node.parent_id = Some(chain[chain.len() - visible_depth + 1]);
        }
    }

    if nodes.is_empty() {
        return Ok(PublicCommentPage {
            items: Vec::new(),
            page,
            total_pages,
            total,
            root_total,
        });
    }

    fn build_subtree(
        parent: Option<Uuid>,
        nodes: &[FlatCommentNode],
        current_depth: usize,
        max_depth: usize,
    ) -> Vec<PublicCommentView> {
        if current_depth > max_depth {
            return Vec::new();
        }
        let mut result = Vec::new();
        for node in nodes {
            if node.parent_id == parent {
                let children = build_subtree(Some(node.id), nodes, current_depth + 1, max_depth);
                result.push(PublicCommentView {
                    id: node.id,
                    parent_id: node.parent_id,
                    author_name: node.author_name.clone(),
                    author_url: validated_author_url(node.author_url.as_deref())
                        .ok()
                        .flatten(),
                    content_html: node.content_html.clone(),
                    created_at: node.created_at,
                    children,
                });
            }
        }
        result
    }

    Ok(PublicCommentPage {
        items: build_subtree(None, &nodes, 1, visible_depth),
        page,
        total_pages,
        total,
        root_total,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_comment_html_escapes_xss() {
        let raw = "<script>alert('xss')</script>\n\nDeuxième ligne & test > 42";
        let formatted = format_comment_html(raw);
        assert!(!formatted.contains("<script>"));
        assert!(formatted.contains("&lt;script&gt;"));
        assert!(formatted.contains("&amp;"));
        assert!(formatted.contains("&gt; 42"));
        assert!(formatted.contains("<p>"));
    }

    #[test]
    fn native_avatars_are_local_deterministic_and_non_personal() {
        let id = Uuid::from_bytes([255; 16]);
        assert!(avatar_svg("blank", id).is_empty());
        assert_eq!(avatar_svg("identicon", id), avatar_svg("identicon", id));
        assert_ne!(avatar_svg("mystery", id), avatar_svg("retro", id));
        assert_eq!(avatar_svg("unsupported", id), avatar_svg("mystery", id));
        for style in ["mystery", "identicon", "retro"] {
            let svg = avatar_svg(style, id);
            assert!(!svg.contains("<image"));
            assert!(!svg.contains("href="));
            assert!(!svg.contains("script"));
        }
    }

    #[test]
    fn entry_discussion_override_keeps_other_protections_and_inheritance() {
        let values = std::collections::BTreeMap::from([
            (
                "discussion.default_comment_status".into(),
                serde_json::json!(false),
            ),
            (
                "discussion.comment_registration".into(),
                serde_json::json!(true),
            ),
            (
                "discussion.close_comments_for_old_posts".into(),
                serde_json::json!(true),
            ),
        ]);
        let inherited = DiscussionPolicy::from_settings(&values)
            .with_entry_fields(&serde_json::json!({"_rf_comment_status": null}));
        assert!(!inherited.can_submit(Some(Utc::now())));
        let opened = DiscussionPolicy::from_settings(&values)
            .with_entry_fields(&serde_json::json!({"_rf_comment_status":"open"}));
        assert!(opened.can_submit(Some(Utc::now())));
        assert!(opened.require_registration);
        assert!(!opened.can_submit(Some(Utc::now() - chrono::Duration::days(30))));
        let closed = DiscussionPolicy::from_settings(&Default::default())
            .with_entry_fields(&serde_json::json!({"_rf_comment_status":"closed"}));
        assert!(!closed.can_submit(Some(Utc::now())));
    }

    #[test]
    fn discussion_policy_applies_switches_and_legacy_closed() {
        let values = serde_json::from_value(serde_json::json!({
            "discussion.default_comment_status":"closed",
            "discussion.require_name_email":false,
            "discussion.thread_comments_depth":999,
            "discussion.comment_order":"desc", "discussion.show_avatars":false
        }))
        .unwrap();
        let policy = DiscussionPolicy::from_settings(&values);
        assert!(!policy.can_submit(Some(Utc::now())));
        assert!(!policy.require_identity);
        assert_eq!(policy.depth, 5);
        assert!(policy.newest_first);
        assert!(!policy.show_avatars);
    }

    #[test]
    fn author_links_reject_unsafe_schemes() {
        for raw in [
            "javascript:alert(1)",
            "JaVaScRiPt:alert(1)",
            "data:text/html,test",
            "https://ok.test/\npath",
            "https://user:pass@ok.test/",
        ] {
            assert!(validated_author_url(Some(raw)).is_err(), "{raw}");
        }
        assert_eq!(
            validated_author_url(Some("https://example.com/"))
                .unwrap()
                .as_deref(),
            Some("https://example.com/")
        );
        assert!(validated_author_url(None).unwrap().is_none());
    }

    #[test]
    fn test_count_links() {
        assert_eq!(count_links("Texte sans lien"), 0);
        assert_eq!(
            count_links("Visitez https://example.com et http://rust-lang.org"),
            2
        );
        assert_eq!(count_links("Lien www.rustforge.org sans protocole"), 1);
    }

    #[test]
    fn test_contains_term() {
        let list = "casino\nviagra\nargent facile\n";
        assert!(contains_term("Achetez du VIAGRA pas cher", list));
        assert!(contains_term("Gagnez de l'argent facile", list));
        assert!(!contains_term("Un article sur Rust et Axum", list));
    }
}
