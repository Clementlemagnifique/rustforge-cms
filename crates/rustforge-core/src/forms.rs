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
    #[serde(default)]
    pub deleted_at: Option<DateTime<Utc>>,
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
    if input.name.trim().is_empty() || input.name.chars().count() > 200 {
        return Err(CoreError::validation("Nom de formulaire invalide"));
    }
    if let Some(slug) = &input.slug {
        crate::security::safe_slug(slug)?;
    }
    crate::form_validation::schema(&input.fields, 0)?;
    crate::form_notifications::validate_settings(input.settings.as_ref())?;
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
    if input.name.trim().is_empty() || input.name.chars().count() > 200 {
        return Err(CoreError::validation("Nom de formulaire invalide"));
    }
    if let Some(slug) = &input.slug {
        crate::security::safe_slug(slug)?;
    }
    crate::form_validation::schema(&input.fields, 0)?;
    crate::form_notifications::validate_settings(input.settings.as_ref())?;
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

    let mut tx = db.begin().await?;
    let sub: FormSubmission = sqlx::query_as(
        "INSERT INTO form_submissions (form_id, data, ip_hash, user_agent, is_spam)
         VALUES ($1, $2, $3, $4, $5) RETURNING *",
    )
    .bind(form_id)
    .bind(payload.data)
    .bind(ip_hash)
    .bind(payload.user_agent)
    .bind(is_spam)
    .fetch_one(&mut *tx)
    .await?;
    crate::form_notifications::enqueue_in(&mut tx, &form, &sub).await?;
    tx.commit().await?;
    Ok(sub)
}

/// Backwards-compatible inbox listing; trashed records are excluded by default.
pub async fn list_submissions(
    db: &PgPool,
    form_id: Uuid,
    page: Option<i64>,
    per_page: Option<i64>,
) -> CoreResult<Page<FormSubmission>> {
    Ok(submission_inbox(
        db,
        form_id,
        &SubmissionFilter {
            page,
            per_page,
            ..Default::default()
        },
    )
    .await?
    .page)
}

#[derive(Debug, Default, Deserialize)]
pub struct SubmissionFilter {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub q: Option<String>,
    pub is_read: Option<bool>,
    pub is_spam: Option<bool>,
    #[serde(default)]
    pub trash: bool,
}
impl SubmissionFilter {
    pub fn validate(&self) -> CoreResult<()> {
        if self.page.is_some_and(|p| !(1..=1_000_000).contains(&p))
            || self.per_page.is_some_and(|p| !(1..=100).contains(&p))
            || self
                .q
                .as_ref()
                .is_some_and(|q| q.chars().count() > 200 || q.contains('\0'))
        {
            return Err(CoreError::validation(
                "Recherche ou pagination des soumissions invalide",
            ));
        }
        Ok(())
    }
    fn pattern(&self) -> Option<String> {
        self.q
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| {
                format!(
                    "%{}%",
                    s.replace('\\', "\\\\")
                        .replace('%', "\\%")
                        .replace('_', "\\_")
                )
            })
    }
}
#[derive(Debug, Default, Serialize, sqlx::FromRow)]
pub struct SubmissionCounts {
    pub inbox: i64,
    pub unread: i64,
    pub spam: i64,
    pub trash: i64,
}
pub struct SubmissionInbox {
    pub page: Page<FormSubmission>,
    pub counts: SubmissionCounts,
}
pub async fn submission_inbox(
    db: &PgPool,
    form_id: Uuid,
    filter: &SubmissionFilter,
) -> CoreResult<SubmissionInbox> {
    filter.validate()?;
    let params = PageParams::new(filter.page, filter.per_page);
    let pattern = filter.pattern();
    let mut tx = db.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .execute(&mut *tx)
        .await?;
    let counts = sqlx::query_as::<_, SubmissionCounts>("SELECT count(*) FILTER(WHERE deleted_at IS NULL) AS inbox, count(*) FILTER(WHERE deleted_at IS NULL AND NOT is_read) AS unread, count(*) FILTER(WHERE deleted_at IS NULL AND is_spam) AS spam, count(*) FILTER(WHERE deleted_at IS NOT NULL) AS trash FROM form_submissions WHERE form_id=$1").bind(form_id).fetch_one(&mut *tx).await?;
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM form_submissions WHERE form_id=$1 AND (deleted_at IS NOT NULL)=$2 AND ($3::bool IS NULL OR is_read=$3) AND ($4::bool IS NULL OR is_spam=$4) AND ($5::text IS NULL OR data::text ILIKE $5)")
        .bind(form_id).bind(filter.trash).bind(filter.is_read).bind(filter.is_spam).bind(&pattern).fetch_one(&mut *tx).await?;
    let items = sqlx::query_as("SELECT * FROM form_submissions WHERE form_id=$1 AND (deleted_at IS NOT NULL)=$2 AND ($3::bool IS NULL OR is_read=$3) AND ($4::bool IS NULL OR is_spam=$4) AND ($5::text IS NULL OR data::text ILIKE $5) ORDER BY created_at DESC,id DESC LIMIT $6 OFFSET $7")
        .bind(form_id).bind(filter.trash).bind(filter.is_read).bind(filter.is_spam).bind(&pattern).bind(params.limit()).bind(params.offset()).fetch_all(&mut *tx).await?;
    tx.commit().await?;
    Ok(SubmissionInbox {
        page: Page::new(items, total, params),
        counts,
    })
}
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubmissionAction {
    Read,
    Unread,
    Trash,
    Restore,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubmissionChange {
    pub action: SubmissionAction,
    pub expected_is_read: bool,
    pub expected_deleted_at: Option<DateTime<Utc>>,
}
/// Compare the displayed state before applying a reversible change.
pub async fn change_submission(
    db: &PgPool,
    form_id: Uuid,
    id: Uuid,
    change: &SubmissionChange,
) -> CoreResult<FormSubmission> {
    let trash = change.expected_deleted_at.is_some();
    if matches!(change.action, SubmissionAction::Restore) != trash {
        return Err(CoreError::validation(
            "Action incompatible avec la corbeille",
        ));
    }
    let read = match change.action {
        SubmissionAction::Read => true,
        SubmissionAction::Unread => false,
        _ => change.expected_is_read,
    };
    let deleted = match change.action {
        SubmissionAction::Trash => Some(Utc::now()),
        _ => None,
    };
    sqlx::query_as("UPDATE form_submissions SET is_read=$4,deleted_at=$5 WHERE id=$1 AND form_id=$2 AND is_read=$3 AND deleted_at IS NOT DISTINCT FROM $6 RETURNING *")
        .bind(id).bind(form_id).bind(change.expected_is_read).bind(read).bind(deleted).bind(change.expected_deleted_at)
        .fetch_optional(db).await?.ok_or_else(||CoreError::Conflict("Soumission modifiée depuis son affichage ; rechargez la boîte de réception".into()))
}

#[cfg(test)]
mod inbox_tests {
    use super::*;
    #[test]
    fn search_is_literal_bounded_and_pagination_checked() {
        let filter = SubmissionFilter {
            q: Some("  50%_\\x  ".into()),
            ..Default::default()
        };
        assert_eq!(filter.pattern().as_deref(), Some("%50\\%\\_\\\\x%"));
        assert!(filter.validate().is_ok());
        assert!(
            SubmissionFilter {
                page: Some(0),
                ..Default::default()
            }
            .validate()
            .is_err()
        );
        assert!(
            SubmissionFilter {
                per_page: Some(101),
                ..Default::default()
            }
            .validate()
            .is_err()
        );
        assert!(
            SubmissionFilter {
                q: Some("é".repeat(201)),
                ..Default::default()
            }
            .validate()
            .is_err()
        );
    }
}

pub async fn mark_as_read(db: &PgPool, submission_id: Uuid) -> CoreResult<()> {
    sqlx::query("UPDATE form_submissions SET is_read = true WHERE id = $1 AND deleted_at IS NULL")
        .bind(submission_id)
        .execute(db)
        .await?;
    Ok(())
}

/// Only these explicitly offered policies are accepted. Null/missing means no purge.
pub fn retention_days(value: Option<&Value>) -> CoreResult<Option<i32>> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) if s == "none" => Ok(None),
        Some(Value::String(s)) if s == "30_days" => Ok(Some(30)),
        Some(Value::String(s)) if s == "90_days" => Ok(Some(90)),
        _ => Err(CoreError::validation(
            "Conservation attendue : none, 30_days ou 90_days",
        )),
    }
}
/// Bounded scheduler maintenance. Serializes with policy edits; disabled by default.
pub async fn purge_expired_submissions(db: &PgPool) -> CoreResult<u64> {
    let mut tx = db.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(72430652)")
        .execute(&mut *tx)
        .await?;
    let value: Option<Value> =
        sqlx::query_scalar("SELECT value FROM settings WHERE key='forms.retention'")
            .fetch_optional(&mut *tx)
            .await?;
    let Some(days) = retention_days(value.as_ref())? else {
        return Ok(0);
    };
    let removed=sqlx::query("WITH expired AS (SELECT id FROM form_submissions WHERE created_at < now()-make_interval(days=>$1) ORDER BY created_at,id LIMIT 1000 FOR UPDATE SKIP LOCKED) DELETE FROM form_submissions s USING expired e WHERE s.id=e.id")
        .bind(days).execute(&mut *tx).await?.rows_affected();
    tx.commit().await?;
    Ok(removed)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubmissionExpected {
    pub id: Uuid,
    pub expected_is_read: bool,
    pub expected_deleted_at: Option<DateTime<Utc>>,
}
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubmissionBatchAction {
    Read,
    Unread,
    Trash,
    Restore,
    Delete,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubmissionBatch {
    pub action: SubmissionBatchAction,
    pub items: Vec<SubmissionExpected>,
    #[serde(default)]
    pub confirm_delete: bool,
}
/// Lock and validate the entire explicit selection before changing any row.
pub async fn batch_submissions(
    db: &PgPool,
    form_id: Uuid,
    batch: &SubmissionBatch,
) -> CoreResult<usize> {
    use std::collections::HashSet;
    let ids = batch.items.iter().map(|item| item.id).collect::<Vec<_>>();
    if ids.is_empty() || ids.len() > 100 || ids.iter().collect::<HashSet<_>>().len() != ids.len() {
        return Err(CoreError::validation(
            "Sélection attendue : 1 à 100 soumissions distinctes",
        ));
    }
    if matches!(batch.action, SubmissionBatchAction::Delete) && !batch.confirm_delete {
        return Err(CoreError::validation(
            "Confirmez la suppression définitive de la sélection",
        ));
    }
    let mut tx = db.begin().await?;
    let rows: Vec<FormSubmission> = sqlx::query_as(
        "SELECT * FROM form_submissions WHERE form_id=$1 AND id=ANY($2) ORDER BY id FOR UPDATE",
    )
    .bind(form_id)
    .bind(&ids)
    .fetch_all(&mut *tx)
    .await?;
    if rows.len() != ids.len() {
        return Err(CoreError::Conflict(
            "Sélection modifiée ou hors de ce formulaire ; aucune modification".into(),
        ));
    }
    for row in &rows {
        let expected = batch.items.iter().find(|item| item.id == row.id).unwrap();
        if row.is_read != expected.expected_is_read
            || row.deleted_at != expected.expected_deleted_at
        {
            return Err(CoreError::Conflict(
                "Une soumission a changé ; aucune modification du lot".into(),
            ));
        }
        if matches!(
            batch.action,
            SubmissionBatchAction::Restore | SubmissionBatchAction::Delete
        ) != row.deleted_at.is_some()
        {
            return Err(CoreError::validation(
                "Action incompatible avec le dossier de la sélection",
            ));
        }
    }
    match batch.action {
        SubmissionBatchAction::Delete => {
            sqlx::query("DELETE FROM form_submissions WHERE form_id=$1 AND id=ANY($2)")
                .bind(form_id)
                .bind(&ids)
                .execute(&mut *tx)
                .await?;
        }
        _ => {
            let read = match batch.action {
                SubmissionBatchAction::Read => Some(true),
                SubmissionBatchAction::Unread => Some(false),
                _ => None,
            };
            let deleted = if matches!(batch.action, SubmissionBatchAction::Trash) {
                Some(Utc::now())
            } else {
                None
            };
            sqlx::query("UPDATE form_submissions SET is_read=COALESCE($3,is_read),deleted_at=$4 WHERE form_id=$1 AND id=ANY($2)").bind(form_id).bind(&ids).bind(read).bind(deleted).execute(&mut *tx).await?;
        }
    }
    tx.commit().await?;
    Ok(ids.len())
}
