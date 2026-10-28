//! # Module Galeries Multi-sources (Photos, Vidéos Locales, YouTube, Vimeo)
//!
//! Permet de composer des galeries enrichies et de les appeler facilement dans
//! n'importe quel bloc ou template via leur slug ou UUID.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::CoreResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "gallery_item_type", rename_all = "snake_case")]
pub enum GalleryItemType {
    Image,
    VideoLocal,
    Youtube,
    Vimeo,
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Gallery {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub layout: String,
    pub settings: Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[sqlx(skip)]
    pub items: Vec<GalleryItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GalleryItem {
    pub id: Uuid,
    pub gallery_id: Uuid,
    pub item_type: GalleryItemType,
    pub media_id: Option<Uuid>,
    pub external_url: Option<String>,
    pub title: String,
    pub caption: String,
    pub thumbnail_url: Option<String>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryInput {
    pub name: String,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub layout: Option<String>,
    pub settings: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryItemInput {
    pub item_type: GalleryItemType,
    pub media_id: Option<Uuid>,
    pub external_url: Option<String>,
    pub title: Option<String>,
    pub caption: Option<String>,
    pub thumbnail_url: Option<String>,
    pub sort_order: Option<i32>,
}

pub async fn list_galleries(db: &PgPool) -> CoreResult<Vec<Gallery>> {
    let mut galleries: Vec<Gallery> = sqlx::query_as("SELECT * FROM galleries ORDER BY name ASC")
        .fetch_all(db)
        .await?;
    for g in &mut galleries {
        g.items = get_gallery_items(db, g.id).await?;
    }
    Ok(galleries)
}

pub async fn get_gallery(db: &PgPool, id: Uuid) -> CoreResult<Gallery> {
    let mut gallery: Gallery = sqlx::query_as("SELECT * FROM galleries WHERE id = $1")
        .bind(id)
        .fetch_one(db)
        .await?;
    gallery.items = get_gallery_items(db, gallery.id).await?;
    Ok(gallery)
}

pub async fn get_gallery_by_slug(db: &PgPool, slug: &str) -> CoreResult<Gallery> {
    let mut gallery: Gallery = sqlx::query_as("SELECT * FROM galleries WHERE slug = $1")
        .bind(slug)
        .fetch_one(db)
        .await?;
    gallery.items = get_gallery_items(db, gallery.id).await?;
    Ok(gallery)
}

pub async fn get_gallery_items(db: &PgPool, gallery_id: Uuid) -> CoreResult<Vec<GalleryItem>> {
    Ok(sqlx::query_as(
        "SELECT * FROM gallery_items WHERE gallery_id = $1 ORDER BY sort_order ASC, created_at ASC",
    )
    .bind(gallery_id)
    .fetch_all(db)
    .await?)
}

pub async fn create_gallery(
    db: &PgPool,
    author_id: Uuid,
    input: GalleryInput,
) -> CoreResult<Gallery> {
    let slug = input.slug.unwrap_or_else(|| crate::slugify(&input.name));
    let mut gallery: Gallery = sqlx::query_as(
        "INSERT INTO galleries (name, slug, description, layout, settings, created_by)
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
    )
    .bind(input.name.trim())
    .bind(slug)
    .bind(input.description.unwrap_or_default())
    .bind(input.layout.unwrap_or_else(|| "grid".into()))
    .bind(input.settings.unwrap_or_else(|| serde_json::json!({})))
    .bind(author_id)
    .fetch_one(db)
    .await?;
    gallery.items = vec![];
    Ok(gallery)
}

pub async fn add_item(
    db: &PgPool,
    gallery_id: Uuid,
    input: GalleryItemInput,
) -> CoreResult<GalleryItem> {
    let item: GalleryItem = sqlx::query_as(
        "INSERT INTO gallery_items (gallery_id, item_type, media_id, external_url, title, caption, thumbnail_url, sort_order)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *",
    )
    .bind(gallery_id)
    .bind(input.item_type)
    .bind(input.media_id)
    .bind(input.external_url)
    .bind(input.title.unwrap_or_default())
    .bind(input.caption.unwrap_or_default())
    .bind(input.thumbnail_url)
    .bind(input.sort_order.unwrap_or(0))
    .fetch_one(db)
    .await?;
    Ok(item)
}

pub async fn delete_gallery(db: &PgPool, id: Uuid) -> CoreResult<()> {
    sqlx::query("DELETE FROM galleries WHERE id = $1")
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn delete_item(db: &PgPool, item_id: Uuid) -> CoreResult<()> {
    sqlx::query("DELETE FROM gallery_items WHERE id = $1")
        .bind(item_id)
        .execute(db)
        .await?;
    Ok(())
}
