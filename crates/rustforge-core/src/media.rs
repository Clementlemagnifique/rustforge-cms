//! Médiathèque : métadonnées en base, arborescence de dossiers et variantes WebP.

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{CoreResult, Page, PageParams, models::Media};

const SELECT: &str = "SELECT m.id, m.path, ('/uploads/' || m.path) AS url, m.original_name, m.mime_type, m.size_bytes,
        m.width, m.height, m.alt, m.caption, m.uploaded_by, m.created_at,
        m.folder_id, m.webp_path, ('/uploads/' || m.webp_path) AS webp_url, m.external_provider, m.metadata
    FROM media m";

/// Dossier hiérarchique de médias.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MediaFolder {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
pub struct NewMedia {
    pub path: String,
    pub original_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub uploaded_by: Uuid,
    pub folder_id: Option<Uuid>,
    pub webp_path: Option<String>,
    pub external_provider: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct MediaUpdate {
    pub alt: Option<String>,
    pub caption: Option<String>,
    pub folder_id: Option<Uuid>,
    pub clear_folder: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
pub struct MediaQuery {
    pub q: Option<String>,
    pub kind: Option<String>,
    pub folder_id: Option<Uuid>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

pub async fn create(db: &PgPool, m: NewMedia) -> CoreResult<Media> {
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO media (path, original_name, mime_type, size_bytes, width, height, uploaded_by, folder_id, webp_path, external_provider, metadata)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING id",
    )
    .bind(&m.path)
    .bind(&m.original_name)
    .bind(&m.mime_type)
    .bind(m.size_bytes)
    .bind(m.width)
    .bind(m.height)
    .bind(m.uploaded_by)
    .bind(m.folder_id)
    .bind(m.webp_path)
    .bind(m.external_provider)
    .bind(m.metadata)
    .fetch_one(db)
    .await?;
    get(db, id).await
}

pub async fn get(db: &PgPool, id: Uuid) -> CoreResult<Media> {
    Ok(sqlx::query_as::<_, Media>(&format!("{SELECT} WHERE m.id = $1")).bind(id).fetch_one(db).await?)
}

pub async fn list(db: &PgPool, q: &MediaQuery) -> CoreResult<Page<Media>> {
    let params = PageParams::new(q.page, q.per_page);
    let search = q.q.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()).map(|s| format!("%{s}%"));
    let kind = q.kind.as_ref().filter(|s| !s.is_empty()).map(|s| format!("{s}%"));
    
    let mut qb = sqlx::QueryBuilder::new("SELECT count(*) FROM media m WHERE TRUE");
    if let Some(ref s) = search {
        qb.push(" AND (m.original_name ILIKE ").push_bind(s).push(" OR m.alt ILIKE ").push_bind(s).push(")");
    }
    if let Some(ref k) = kind {
        qb.push(" AND m.mime_type LIKE ").push_bind(k);
    }
    if let Some(f_id) = q.folder_id {
        qb.push(" AND m.folder_id = ").push_bind(f_id);
    }
    let total: i64 = qb.build_query_scalar().fetch_one(db).await?;

    let mut qb = sqlx::QueryBuilder::new(SELECT);
    qb.push(" WHERE TRUE");
    if let Some(ref s) = search {
        qb.push(" AND (m.original_name ILIKE ").push_bind(s).push(" OR m.alt ILIKE ").push_bind(s).push(")");
    }
    if let Some(ref k) = kind {
        qb.push(" AND m.mime_type LIKE ").push_bind(k);
    }
    if let Some(f_id) = q.folder_id {
        qb.push(" AND m.folder_id = ").push_bind(f_id);
    }
    qb.push(" ORDER BY m.created_at DESC LIMIT ").push_bind(params.limit()).push(" OFFSET ").push_bind(params.offset());

    let items = qb.build_query_as::<Media>().fetch_all(db).await?;
    Ok(Page::new(items, total, params))
}

pub async fn update(db: &PgPool, id: Uuid, input: MediaUpdate) -> CoreResult<Media> {
    let current = get(db, id).await?;
    let folder_id = if input.clear_folder == Some(true) {
        None
    } else {
        input.folder_id.or(current.folder_id)
    };
    sqlx::query("UPDATE media SET alt = $2, caption = $3, folder_id = $4 WHERE id = $1")
        .bind(id)
        .bind(input.alt.unwrap_or(current.alt).trim())
        .bind(input.caption.unwrap_or(current.caption).trim())
        .bind(folder_id)
        .execute(db)
        .await?;
    get(db, id).await
}

/// Compte le nombre d'entrées (articles/pages) qui utilisent ce média en image mise en avant
/// ou dans leurs blocs (`rf_media_id` ou chemin d'upload) (`WP-40`).
pub async fn usage_count(db: &PgPool, id: Uuid) -> CoreResult<i64> {
    let current = get(db, id).await?;
    let id_str = id.to_string();
    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM entries
         WHERE status <> 'trash'
           AND (featured_media_id = $1
                OR blocks::text LIKE '%' || $2 || '%'
                OR body LIKE '%' || $3 || '%')",
    )
    .bind(id)
    .bind(&id_str)
    .bind(&current.path)
    .fetch_one(db)
    .await?;
    Ok(count)
}

pub async fn delete(db: &PgPool, id: Uuid) -> CoreResult<(String, Option<String>)> {
    delete_checked(db, id, false).await
}

/// Supprime un média après avoir vérifié qu'il n'est pas référencé par un contenu actif (sauf si `force = true`) (`WP-40`).
pub async fn delete_checked(
    db: &PgPool,
    id: Uuid,
    force: bool,
) -> CoreResult<(String, Option<String>)> {
    if !force {
        let usages = usage_count(db, id).await?;
        if usages > 0 {
            return Err(crate::CoreError::Conflict(format!(
                "Ce média est utilisé dans {usages} contenu(s) actif(s). Détachez-le d'abord ou confirmez la suppression forcée."
            )));
        }
    }
    let row: (String, Option<String>) =
        sqlx::query_as("DELETE FROM media WHERE id = $1 RETURNING path, webp_path")
            .bind(id)
            .fetch_one(db)
            .await?;
    Ok(row)
}

// ---------------------------------------------------------------------------
// Gestion des Dossiers Hiérarchiques
// ---------------------------------------------------------------------------

pub async fn list_folders(db: &PgPool) -> CoreResult<Vec<MediaFolder>> {
    Ok(sqlx::query_as("SELECT id, parent_id, name, slug, created_at FROM media_folders ORDER BY name ASC")
        .fetch_all(db)
        .await?)
}

pub async fn create_folder(db: &PgPool, name: &str, parent_id: Option<Uuid>) -> CoreResult<MediaFolder> {
    let slug = crate::slugify(name);
    Ok(sqlx::query_as(
        "INSERT INTO media_folders (name, slug, parent_id) VALUES ($1, $2, $3) RETURNING id, parent_id, name, slug, created_at",
    )
    .bind(name.trim())
    .bind(slug)
    .bind(parent_id)
    .fetch_one(db)
    .await?)
}

pub async fn delete_folder(db: &PgPool, id: Uuid) -> CoreResult<()> {
    sqlx::query("DELETE FROM media_folders WHERE id = $1").bind(id).execute(db).await?;
    Ok(())
}
