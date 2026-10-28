//! Types de contenu déclaratifs (articles, pages, et types personnalisés).

use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    CoreError, CoreResult,
    fields::{FieldDef, validate_schema},
    models::ContentType,
};

const SELECT: &str = "SELECT ct.id, ct.slug, ct.name, ct.name_singular, ct.description, ct.icon,
        ct.hierarchical, ct.has_revisions, ct.is_system, ct.fields, ct.created_at, ct.updated_at,
        (SELECT count(*) FROM entries e WHERE e.content_type_id = ct.id AND e.status <> 'trash') AS entry_count
    FROM content_types ct";

/// Slugs réservés par les routes de l'API.
const RESERVED: &[&str] = &[
    "site",
    "content",
    "taxonomies",
    "media",
    "admin",
    "api",
    "uploads",
];

#[derive(Debug, Deserialize)]
pub struct ContentTypeInput {
    pub slug: Option<String>,
    pub name: String,
    pub name_singular: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub hierarchical: bool,
    #[serde(default = "yes")]
    pub has_revisions: bool,
    #[serde(default)]
    pub fields: Vec<FieldDef>,
}

fn yes() -> bool {
    true
}

pub async fn list(db: &PgPool) -> CoreResult<Vec<ContentType>> {
    Ok(sqlx::query_as::<_, ContentType>(&format!(
        "{SELECT} ORDER BY ct.is_system DESC, ct.name ASC"
    ))
    .fetch_all(db)
    .await?)
}

pub async fn get_by_slug(db: &PgPool, slug: &str) -> CoreResult<ContentType> {
    Ok(
        sqlx::query_as::<_, ContentType>(&format!("{SELECT} WHERE ct.slug = $1"))
            .bind(slug)
            .fetch_one(db)
            .await?,
    )
}

pub async fn get(db: &PgPool, id: Uuid) -> CoreResult<ContentType> {
    Ok(
        sqlx::query_as::<_, ContentType>(&format!("{SELECT} WHERE ct.id = $1"))
            .bind(id)
            .fetch_one(db)
            .await?,
    )
}

fn validate(input: &ContentTypeInput) -> CoreResult<()> {
    if input.name.trim().is_empty() || input.name_singular.trim().is_empty() {
        return Err(CoreError::validation(
            "le nom (pluriel et singulier) est obligatoire",
        ));
    }
    validate_schema(&input.fields)
}

pub async fn create(db: &PgPool, input: ContentTypeInput) -> CoreResult<ContentType> {
    validate(&input)?;
    let slug = crate::slugify(input.slug.as_deref().unwrap_or(&input.name_singular));
    if RESERVED.contains(&slug.as_str()) {
        return Err(CoreError::validation(format!(
            "le slug « {slug} » est réservé"
        )));
    }
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO content_types (slug, name, name_singular, description, icon, hierarchical, has_revisions, fields)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
    )
    .bind(&slug)
    .bind(input.name.trim())
    .bind(input.name_singular.trim())
    .bind(input.description.trim())
    .bind(input.icon.as_deref().unwrap_or("box"))
    .bind(input.hierarchical)
    .bind(input.has_revisions)
    .bind(sqlx::types::Json(&input.fields))
    .fetch_one(db)
    .await?;
    get(db, id).await
}

/// Met à jour un type. Le slug d'un type système n'est jamais modifié.
pub async fn update(db: &PgPool, id: Uuid, input: ContentTypeInput) -> CoreResult<ContentType> {
    validate(&input)?;
    let current = get(db, id).await?;
    let slug = if current.is_system {
        current.slug
    } else {
        let s = crate::slugify(input.slug.as_deref().unwrap_or(&current.slug));
        if RESERVED.contains(&s.as_str()) {
            return Err(CoreError::validation(format!(
                "le slug « {s} » est réservé"
            )));
        }
        s
    };
    sqlx::query(
        "UPDATE content_types SET slug = $2, name = $3, name_singular = $4, description = $5, icon = $6,
            hierarchical = $7, has_revisions = $8, fields = $9, updated_at = now()
         WHERE id = $1",
    )
    .bind(id)
    .bind(slug)
    .bind(input.name.trim())
    .bind(input.name_singular.trim())
    .bind(input.description.trim())
    .bind(input.icon.as_deref().unwrap_or(&current.icon))
    .bind(input.hierarchical)
    .bind(input.has_revisions)
    .bind(sqlx::types::Json(&input.fields))
    .execute(db)
    .await?;
    get(db, id).await
}

pub async fn delete(db: &PgPool, id: Uuid) -> CoreResult<()> {
    let current = get(db, id).await?;
    if current.is_system {
        return Err(CoreError::Conflict(
            "les types système ne peuvent pas être supprimés".into(),
        ));
    }
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM entries WHERE content_type_id = $1")
        .bind(id)
        .fetch_one(db)
        .await?;
    if count > 0 {
        return Err(CoreError::Conflict(format!(
            "ce type contient encore {count} entrée(s) : supprimez-les d'abord"
        )));
    }
    sqlx::query("DELETE FROM content_types WHERE id = $1")
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}
