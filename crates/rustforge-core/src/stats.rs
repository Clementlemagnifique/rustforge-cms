//! Statistiques du tableau de bord.

use serde::Serialize;
use sqlx::PgPool;

use crate::{
    CoreResult,
    entries::{self, EntryQuery},
    models::Entry,
};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TypeStats {
    pub slug: String,
    pub name: String,
    pub icon: String,
    pub published: i64,
    pub drafts: i64,
    pub scheduled: i64,
}

#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub server_version: &'static str,
    pub content_types: Vec<TypeStats>,
    pub media: i64,
    pub media_bytes: i64,
    pub users: i64,
    pub terms: i64,
    pub recent: Vec<Entry>,
    /// Projection légère et explicite pour le Studio Rust ; `recent` reste compatible.
    pub activity: Vec<DashboardActivity>,
}

#[derive(Debug, Serialize)]
pub struct DashboardActivity {
    pub id: uuid::Uuid,
    pub title: String,
    pub content_type: String,
    pub state: &'static str,
}

pub async fn dashboard(db: &PgPool) -> CoreResult<DashboardStats> {
    dashboard_for(db, None).await
}

pub async fn dashboard_for(db: &PgPool, author: Option<uuid::Uuid>) -> CoreResult<DashboardStats> {
    let now = chrono::Utc::now();
    let content_types = sqlx::query_as::<_, TypeStats>(
        "SELECT ct.slug, ct.name, ct.icon,
            count(e.id) FILTER (WHERE e.status = 'published' AND e.published_at <= $2) AS published,
            count(e.id) FILTER (WHERE e.status = 'draft') AS drafts,
            count(e.id) FILTER (WHERE e.status = 'published' AND e.published_at > $2) AS scheduled
         FROM content_types ct LEFT JOIN entries e ON e.content_type_id = ct.id AND ($1::uuid IS NULL OR e.author_id=$1)
         GROUP BY ct.id ORDER BY ct.is_system DESC, ct.name",
    )
    .bind(author).bind(now).fetch_all(db)
    .await?;

    let (media, media_bytes): (i64, i64) =
        sqlx::query_as("SELECT count(*), coalesce(sum(size_bytes), 0)::bigint FROM media WHERE ($1::uuid IS NULL OR uploaded_by=$1)")
            .bind(author).fetch_one(db)
            .await?;
    let users: i64 = if author.is_none() {
        sqlx::query_scalar("SELECT count(*) FROM users")
            .fetch_one(db)
            .await?
    } else {
        0
    };
    let terms: i64 = if author.is_none() {
        sqlx::query_scalar("SELECT count(*) FROM terms")
            .fetch_one(db)
            .await?
    } else {
        0
    };
    let recent = entries::list(
        db,
        &EntryQuery {
            per_page: Some(6),
            author_id: author,
            ..Default::default()
        },
    )
    .await?
    .items;

    let activity = recent
        .iter()
        .map(|entry| DashboardActivity {
            id: entry.id,
            title: entry.title.clone(),
            content_type: entry.content_type.clone(),
            state: match entry.status {
                crate::models::EntryStatus::Draft => "draft",
                crate::models::EntryStatus::Archived => "archived",
                crate::models::EntryStatus::Trash => "trash",
                crate::models::EntryStatus::Published => match entry.published_at {
                    Some(at) if at > now => "scheduled",
                    Some(_) => "published",
                    None => "undated",
                },
            },
        })
        .collect();
    Ok(DashboardStats {
        server_version: env!("CARGO_PKG_VERSION"),
        content_types,
        media,
        media_bytes,
        users,
        terms,
        recent,
        activity,
    })
}
