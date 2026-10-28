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

    if out.is_empty() {
        escaped
    } else {
        out
    }
}

/// Compte le nombre de liens web dans le contenu (détection d'URLs pour antispam).
pub fn count_links(content: &str) -> usize {
    let lower = content.to_ascii_lowercase();
    let http_count = lower.matches("http://").count();
    let https_count = lower.matches("https://").count();
    let www_count = lower.matches("www.").count().saturating_sub(
        lower.matches("http://www").count() + lower.matches("https://www").count(),
    );
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
    if count_links(content) >= max_links as usize {
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
    let name = params.author_name.trim();
    let email = params.author_email.trim();
    let content = params.content.trim();

    if name.is_empty() || name.chars().count() > 100 {
        return Err(CoreError::validation(
            "Le nom de l'auteur est requis (max 100 caractères).",
        ));
    }
    if email.is_empty() || email.chars().count() > 254 || !email.contains('@') {
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

    // Vérifier l'existence et l'état de l'entrée associée
    let entry_opt: Option<(Uuid, String, Option<DateTime<Utc>>)> = sqlx::query_as(
        "SELECT id, status::text, published_at FROM entries WHERE id = $1",
    )
    .bind(params.entry_id)
    .fetch_optional(db)
    .await?;

    let Some((_, status, published_at)) = entry_opt else {
        return Err(CoreError::NotFound);
    };

    if status != "published" {
        return Err(CoreError::validation(
            "Les commentaires ne sont autorisés que sur les publications publiées.",
        ));
    }

    // Vérifier si les commentaires sont fermés pour les anciens articles
    let all_settings = crate::settings::all(db).await?;
    let close_old = all_settings
        .get("discussion.close_comments_for_old_posts")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let close_days = all_settings
        .get("discussion.close_comments_days")
        .and_then(|v| v.as_i64())
        .unwrap_or(14);

    if close_old && let Some(pub_at) = published_at {
        let age_days = (Utc::now() - pub_at).num_days();
        if age_days > close_days {
            return Err(CoreError::validation(
                "Les commentaires sont fermés pour cet article ancien.",
            ));
        }
    }

    // Vérifier le parent_id s'il est spécifié
    if let Some(pid) = params.parent_id {
        let parent_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM comments WHERE id = $1 AND entry_id = $2)",
        )
        .bind(pid)
        .bind(params.entry_id)
        .fetch_one(db)
        .await?;

        if !parent_exists {
            return Err(CoreError::validation("Le commentaire parent est introuvable."));
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
    .bind(params.author_url.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(params.author_ip.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(params.author_user_agent.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(content)
    .bind(content_html)
    .bind(status.as_str())
    .fetch_one(db)
    .await?;

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
pub async fn list_comments(
    db: &PgPool,
    filter: CommentFilter,
) -> CoreResult<(Vec<Comment>, i64)> {
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
    sqlx::query_as(
        r#"
        UPDATE comments
        SET status = $2, updated_at = NOW()
        WHERE id = $1
        RETURNING
            id, entry_id, parent_id, author_id, author_name, author_email,
            author_url, author_ip, author_user_agent, content, content_html,
            status, created_at, updated_at,
            NULL::text as entry_title, NULL::text as entry_slug
        "#,
    )
    .bind(id)
    .bind(new_status.as_str())
    .fetch_optional(db)
    .await?
    .ok_or(CoreError::NotFound)
}

/// Met à jour le statut d'un ensemble de commentaires en masse (`WP-43`).
pub async fn batch_update_status(
    db: &PgPool,
    ids: &[Uuid],
    new_status: CommentStatus,
) -> CoreResult<u64> {
    if ids.is_empty() {
        return Ok(0);
    }
    let res = sqlx::query(
        "UPDATE comments SET status = $2, updated_at = NOW() WHERE id = ANY($1)",
    )
    .bind(ids)
    .bind(new_status.as_str())
    .execute(db)
    .await?;

    Ok(res.rows_affected())
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
    let trimmed_content = content.trim();
    if trimmed_content.is_empty() {
        return Err(CoreError::validation("Le contenu ne peut pas être vide."));
    }
    let content_html = format_comment_html(trimmed_content);

    let comment: Comment = sqlx::query_as(
        r#"
        UPDATE comments
        SET content = $2,
            content_html = $3,
            author_name = COALESCE($4, author_name),
            author_email = COALESCE($5, author_email),
            author_url = $6,
            updated_at = NOW()
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
    .bind(author_url.map(str::trim).filter(|s| !s.is_empty()))
    .fetch_optional(db)
    .await?
    .ok_or(CoreError::NotFound)?;

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
    .fetch_one(db)
    .await?;

    Ok(reply)
}

/// Supprime définitivement un commentaire de la base de données.
pub async fn delete_comment(db: &PgPool, id: Uuid) -> CoreResult<bool> {
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

/// Récupère la liste des commentaires approuvés pour une entrée sous forme d'arbre hiérarchique.
pub async fn get_entry_approved_comments_tree(
    db: &PgPool,
    entry_id: Uuid,
    max_depth: usize,
) -> CoreResult<Vec<PublicCommentView>> {
    #[derive(sqlx::FromRow)]
    struct FlatCommentNode {
        id: Uuid,
        parent_id: Option<Uuid>,
        author_name: String,
        author_url: Option<String>,
        content_html: String,
        created_at: DateTime<Utc>,
    }

    let nodes: Vec<FlatCommentNode> = sqlx::query_as(
        r#"
        SELECT id, parent_id, author_name, author_url, content_html, created_at
        FROM comments
        WHERE entry_id = $1 AND status = 'approved'
        ORDER BY created_at ASC
        "#,
    )
    .bind(entry_id)
    .fetch_all(db)
    .await?;

    if nodes.is_empty() {
        return Ok(Vec::new());
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
                    author_url: node.author_url.clone(),
                    content_html: node.content_html.clone(),
                    created_at: node.created_at,
                    children,
                });
            }
        }
        result
    }

    Ok(build_subtree(None, &nodes, 1, max_depth.clamp(1, 5)))
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
    fn test_count_links() {
        assert_eq!(count_links("Texte sans lien"), 0);
        assert_eq!(
            count_links("Visitez https://example.com et http://rust-lang.org"),
            2
        );
        assert_eq!(
            count_links("Lien www.rustforge.org sans protocole"),
            1
        );
    }

    #[test]
    fn test_contains_term() {
        let list = "casino\nviagra\nargent facile\n";
        assert!(contains_term("Achetez du VIAGRA pas cher", list));
        assert!(contains_term("Gagnez de l'argent facile", list));
        assert!(!contains_term("Un article sur Rust et Axum", list));
    }
}
