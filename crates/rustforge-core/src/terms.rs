//! Taxonomies et termes (catégories, étiquettes, taxonomies personnalisées).

use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    CoreError, CoreResult,
    models::{Taxonomy, Term},
};

const TERM_SELECT: &str = "SELECT t.id, t.taxonomy_id, tx.slug AS taxonomy, t.parent_id, t.name, t.slug,
        t.description, t.created_at,
        (SELECT count(*) FROM entry_terms et JOIN entries e ON e.id = et.entry_id
          WHERE et.term_id = t.id AND e.status <> 'trash') AS entry_count
    FROM terms t JOIN taxonomies tx ON tx.id = t.taxonomy_id";

#[derive(Debug, Deserialize)]
pub struct TermInput {
    /// Slug de la taxonomie (création uniquement).
    pub taxonomy: Option<String>,
    pub name: String,
    pub slug: Option<String>,
    #[serde(default)]
    pub description: String,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct TaxonomyInput {
    pub name: String,
    pub slug: Option<String>,
    #[serde(default)]
    pub hierarchical: bool,
}

pub async fn list_taxonomies(db: &PgPool) -> CoreResult<Vec<Taxonomy>> {
    Ok(sqlx::query_as::<_, Taxonomy>(
        "SELECT tx.id, tx.slug, tx.name, tx.hierarchical, tx.is_system,
                (SELECT count(*) FROM terms t WHERE t.taxonomy_id = tx.id) AS term_count
         FROM taxonomies tx ORDER BY tx.is_system DESC, tx.name",
    )
    .fetch_all(db)
    .await?)
}

pub async fn create_taxonomy(db: &PgPool, input: TaxonomyInput) -> CoreResult<Taxonomy> {
    if input.name.trim().is_empty() {
        return Err(CoreError::validation("le nom est obligatoire"));
    }
    let slug = crate::slugify(input.slug.as_deref().unwrap_or(&input.name));
    sqlx::query("INSERT INTO taxonomies (slug, name, hierarchical) VALUES ($1, $2, $3)")
        .bind(&slug)
        .bind(input.name.trim())
        .bind(input.hierarchical)
        .execute(db)
        .await?;
    list_taxonomies(db).await?.into_iter().find(|t| t.slug == slug).ok_or(CoreError::NotFound)
}

pub async fn delete_taxonomy(db: &PgPool, id: Uuid) -> CoreResult<()> {
    let is_system: bool = sqlx::query_scalar("SELECT is_system FROM taxonomies WHERE id = $1")
        .bind(id)
        .fetch_one(db)
        .await?;
    if is_system {
        return Err(CoreError::Conflict("les taxonomies système ne peuvent pas être supprimées".into()));
    }
    sqlx::query("DELETE FROM taxonomies WHERE id = $1").bind(id).execute(db).await?;
    Ok(())
}

pub async fn list_terms(db: &PgPool, taxonomy: Option<&str>) -> CoreResult<Vec<Term>> {
    Ok(sqlx::query_as::<_, Term>(&format!(
        "{TERM_SELECT} WHERE ($1::text IS NULL OR tx.slug = $1) ORDER BY tx.slug, t.name"
    ))
    .bind(taxonomy)
    .fetch_all(db)
    .await?)
}

pub async fn get_term(db: &PgPool, id: Uuid) -> CoreResult<Term> {
    Ok(sqlx::query_as::<_, Term>(&format!("{TERM_SELECT} WHERE t.id = $1"))
        .bind(id)
        .fetch_one(db)
        .await?)
}

pub async fn create_term(db: &PgPool, input: TermInput) -> CoreResult<Term> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(CoreError::validation("le nom est obligatoire"));
    }
    let taxonomy = input.taxonomy.as_deref().ok_or_else(|| CoreError::validation("taxonomie manquante"))?;
    let (taxonomy_id, hierarchical): (Uuid, bool) =
        sqlx::query_as("SELECT id, hierarchical FROM taxonomies WHERE slug = $1")
            .bind(taxonomy)
            .fetch_optional(db)
            .await?
            .ok_or_else(|| CoreError::validation(format!("taxonomie inconnue « {taxonomy} »")))?;
    let slug = crate::slugify(input.slug.as_deref().filter(|s| !s.is_empty()).unwrap_or(name));
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO terms (taxonomy_id, parent_id, name, slug, description) VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(taxonomy_id)
    .bind(if hierarchical { input.parent_id } else { None })
    .bind(name)
    .bind(slug)
    .bind(input.description.trim())
    .fetch_one(db)
    .await?;
    get_term(db, id).await
}

pub async fn update_term(db: &PgPool, id: Uuid, input: TermInput) -> CoreResult<Term> {
    let current = get_term(db, id).await?;
    let name = input.name.trim();
    if name.is_empty() {
        return Err(CoreError::validation("le nom est obligatoire"));
    }
    if input.parent_id == Some(id) {
        return Err(CoreError::validation("un terme ne peut pas être son propre parent"));
    }
    let slug = crate::slugify(input.slug.as_deref().filter(|s| !s.is_empty()).unwrap_or(&current.slug));
    sqlx::query("UPDATE terms SET name = $2, slug = $3, description = $4, parent_id = $5 WHERE id = $1")
        .bind(id)
        .bind(name)
        .bind(slug)
        .bind(input.description.trim())
        .bind(input.parent_id)
        .execute(db)
        .await?;
    get_term(db, id).await
}

pub async fn delete_term(db: &PgPool, id: Uuid) -> CoreResult<()> {
    let res = sqlx::query("DELETE FROM terms WHERE id = $1").bind(id).execute(db).await?;
    if res.rows_affected() == 0 {
        return Err(CoreError::NotFound);
    }
    Ok(())
}
