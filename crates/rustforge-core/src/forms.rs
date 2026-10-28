//! # Moteur de Formulaires Professionnel (Type WPForms Pro / Gravity Forms)
//!
//! Permet la déclaration de formulaires, la validation stricte des champs,
//! la capture sécurisée des soumissions et la protection anti-spam sans captcha intrusif.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{CoreError, CoreResult, Page, PageParams};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Form {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub fields: Value,
    pub settings: Value,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormInput {
    pub name: String,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub fields: Value,
    pub settings: Option<Value>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FormSubmission {
    pub id: Uuid,
    pub form_id: Uuid,
    pub data: Value,
    pub ip_hash: String,
    pub user_agent: Option<String>,
    pub is_spam: bool,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitFormPayload {
    pub data: Value,
    /// Honeypot invisible : si rempli par un bot, la soumission est marquée spam
    pub _website: Option<String>,
    pub user_agent: Option<String>,
    pub client_ip: Option<String>,
}

pub async fn list_forms(db: &PgPool) -> CoreResult<Vec<Form>> {
    Ok(
        sqlx::query_as("SELECT * FROM forms ORDER BY created_at DESC")
            .fetch_all(db)
            .await?,
    )
}

pub async fn get_form(db: &PgPool, id: Uuid) -> CoreResult<Form> {
    Ok(sqlx::query_as("SELECT * FROM forms WHERE id = $1")
        .bind(id)
        .fetch_one(db)
        .await?)
}

pub async fn get_form_by_slug(db: &PgPool, slug: &str) -> CoreResult<Form> {
    Ok(sqlx::query_as("SELECT * FROM forms WHERE slug = $1")
        .bind(slug)
        .fetch_one(db)
        .await?)
}

pub async fn create_form(db: &PgPool, author_id: Uuid, input: FormInput) -> CoreResult<Form> {
    crate::form_validation::schema(&input.fields, 0)?;
    let slug = input.slug.unwrap_or_else(|| crate::slugify(&input.name));
    Ok(sqlx::query_as(
        "INSERT INTO forms (name, slug, description, fields, settings, is_active, created_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *",
    )
    .bind(input.name.trim())
    .bind(slug)
    .bind(input.description.unwrap_or_default())
    .bind(input.fields)
    .bind(input.settings.unwrap_or_else(|| serde_json::json!({})))
    .bind(input.is_active.unwrap_or(true))
    .bind(author_id)
    .fetch_one(db)
    .await?)
}

pub async fn update_form(db: &PgPool, id: Uuid, input: FormInput) -> CoreResult<Form> {
    crate::form_validation::schema(&input.fields, 0)?;
    let current = get_form(db, id).await?;
    let slug = input.slug.unwrap_or(current.slug);
    Ok(sqlx::query_as(
        "UPDATE forms SET name = $2, slug = $3, description = $4, fields = $5, settings = $6, is_active = $7, updated_at = now()
         WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(input.name.trim())
    .bind(slug)
    .bind(input.description.unwrap_or(current.description))
    .bind(input.fields)
    .bind(input.settings.unwrap_or(current.settings))
    .bind(input.is_active.unwrap_or(current.is_active))
    .fetch_one(db)
    .await?)
}

pub async fn delete_form(db: &PgPool, id: Uuid) -> CoreResult<()> {
    sqlx::query("DELETE FROM forms WHERE id = $1")
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Soumissions & Interaction Dashboard
// ---------------------------------------------------------------------------

pub async fn submit(
    db: &PgPool,
    form_id: Uuid,
    payload: SubmitFormPayload,
) -> CoreResult<FormSubmission> {
    let form = get_form(db, form_id).await?;
    if !form.is_active {
        return Err(CoreError::validation(
            "ce formulaire est actuellement désactivé",
        ));
    }
    crate::form_validation::validate(&form.fields, &payload.data)?;

    // Détection Honeypot anti-spam sans captcha intrusif
    let is_spam = payload._website.is_some_and(|w| !w.trim().is_empty());

    // IP collection is disabled. Do not trust a client-supplied IP or claim anonymization.
    let ip_hash = String::new();

    let sub: FormSubmission = sqlx::query_as(
        "INSERT INTO form_submissions (form_id, data, ip_hash, user_agent, is_spam)
         VALUES ($1, $2, $3, $4, $5) RETURNING *",
    )
    .bind(form_id)
    .bind(payload.data)
    .bind(ip_hash)
    .bind(payload.user_agent)
    .bind(is_spam)
    .fetch_one(db)
    .await?;

    Ok(sub)
}

pub async fn list_submissions(
    db: &PgPool,
    form_id: Uuid,
    page: Option<i64>,
    per_page: Option<i64>,
) -> CoreResult<Page<FormSubmission>> {
    let params = PageParams::new(page, per_page);
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM form_submissions WHERE form_id = $1")
        .bind(form_id)
        .fetch_one(db)
        .await?;

    let items = sqlx::query_as(
        "SELECT * FROM form_submissions WHERE form_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(form_id)
    .bind(params.limit())
    .bind(params.offset())
    .fetch_all(db)
    .await?;

    Ok(Page::new(items, total, params))
}

pub async fn mark_as_read(db: &PgPool, submission_id: Uuid) -> CoreResult<()> {
    sqlx::query("UPDATE form_submissions SET is_read = true WHERE id = $1")
        .bind(submission_id)
        .execute(db)
        .await?;
    Ok(())
}
