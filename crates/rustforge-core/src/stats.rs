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
    pub content_types: Vec<TypeStats>,
    pub media: i64,
    pub media_bytes: i64,
    pub users: i64,
    pub terms: i64,
    pub recent: Vec<Entry>,
}

pub async fn dashboard(db: &PgPool) -> CoreResult<DashboardStats> {
    let content_types = sqlx::query_as::<_, TypeStats>(
        "SELECT ct.slug, ct.name, ct.icon,
            count(e.id) FILTER (WHERE e.status = 'published' AND e.published_at <= now()) AS published,
            count(e.id) FILTER (WHERE e.status = 'draft') AS drafts,
            count(e.id) FILTER (WHERE e.status = 'published' AND e.published_at > now()) AS scheduled
         FROM content_types ct LEFT JOIN entries e ON e.content_type_id = ct.id
         GROUP BY ct.id ORDER BY ct.is_system DESC, ct.name",
    )
    .fetch_all(db)
    .await?;

    let (media, media_bytes): (i64, i64) =
        sqlx::query_as("SELECT count(*), coalesce(sum(size_bytes), 0)::bigint FROM media")
            .fetch_one(db)
            .await?;
    let users: i64 = sqlx::query_scalar("SELECT count(*) FROM users").fetch_one(db).await?;
    let terms: i64 = sqlx::query_scalar("SELECT count(*) FROM terms").fetch_one(db).await?;
    let recent = entries::list(db, &EntryQuery { per_page: Some(6), ..Default::default() }).await?.items;

    Ok(DashboardStats { content_types, media, media_bytes, users, terms, recent })
}
