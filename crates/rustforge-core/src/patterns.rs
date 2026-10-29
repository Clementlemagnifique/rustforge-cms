//! Gestion des motifs synchronisés (synced patterns / reusable blocks) - WP-34, STAB-07
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{CoreError, CoreResult, document, slugify};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SyncedPattern {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub document: Value,
    pub is_synced: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewPattern {
    pub slug: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub document: Value,
    pub is_synced: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdatePattern {
    pub title: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub document: Option<Value>,
    pub is_synced: Option<bool>,
}

fn valid_slug(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 96
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}

fn validate_title(t: &str) -> CoreResult<&str> {
    let trimmed = t.trim();
    if trimmed.is_empty() || trimmed.len() > 255 || trimmed.chars().any(char::is_control) {
        return Err(CoreError::validation(
            "Titre du motif invalide (1 à 255 caractères sans contrôle)",
        ));
    }
    Ok(trimmed)
}

fn validate_category(c: &str) -> CoreResult<&str> {
    let trimmed = c.trim();
    if trimmed.is_empty()
        || trimmed.len() > 64
        || !trimmed
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return Err(CoreError::validation("Catégorie de motif invalide"));
    }
    Ok(trimmed)
}

pub async fn list(db: &PgPool) -> CoreResult<Vec<SyncedPattern>> {
    let rows = sqlx::query_as::<_, SyncedPattern>(
        "SELECT id, slug, title, description, category, document, is_synced, created_at, updated_at
         FROM synced_patterns
         ORDER BY category ASC, title ASC",
    )
    .fetch_all(db)
    .await?;
    Ok(rows)
}

pub async fn get_by_id(db: &PgPool, id: Uuid) -> CoreResult<SyncedPattern> {
    sqlx::query_as::<_, SyncedPattern>(
        "SELECT id, slug, title, description, category, document, is_synced, created_at, updated_at
         FROM synced_patterns WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(db)
    .await?
    .ok_or(CoreError::NotFound)
}

pub async fn get_by_slug(db: &PgPool, slug: &str) -> CoreResult<SyncedPattern> {
    sqlx::query_as::<_, SyncedPattern>(
        "SELECT id, slug, title, description, category, document, is_synced, created_at, updated_at
         FROM synced_patterns WHERE slug = $1",
    )
    .bind(slug)
    .fetch_optional(db)
    .await?
    .ok_or(CoreError::NotFound)
}

pub async fn create(db: &PgPool, input: NewPattern) -> CoreResult<SyncedPattern> {
    let title = validate_title(&input.title)?;
    let category = match input.category.as_deref() {
        Some(c) => validate_category(c)?,
        None => "content",
    };
    let slug = match input.slug.as_deref().filter(|s| !s.trim().is_empty()) {
        Some(s) => {
            let s = s.trim();
            if !valid_slug(s) {
                return Err(CoreError::validation("Slug du motif invalide"));
            }
            s.to_string()
        }
        None => {
            let s = slugify(title);
            if s.is_empty() {
                format!("pattern-{}", Uuid::new_v4().simple())
            } else {
                s
            }
        }
    };
    let description = input.description.as_deref().unwrap_or("").trim();
    if description.len() > 1024 {
        return Err(CoreError::validation("Description du motif trop longue"));
    }

    // Validation du schéma rf.document/1
    document::project("", &input.document)?;

    let is_synced = input.is_synced.unwrap_or(true);
    let id = Uuid::new_v4();

    let pattern = sqlx::query_as::<_, SyncedPattern>(
        "INSERT INTO synced_patterns(id, slug, title, description, category, document, is_synced, created_at, updated_at)
         VALUES($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
         RETURNING id, slug, title, description, category, document, is_synced, created_at, updated_at"
    )
    .bind(id)
    .bind(slug)
    .bind(title)
    .bind(description)
    .bind(category)
    .bind(&input.document)
    .bind(is_synced)
    .fetch_one(db)
    .await?;

    Ok(pattern)
}

pub async fn update(db: &PgPool, id: Uuid, input: UpdatePattern) -> CoreResult<SyncedPattern> {
    let existing = get_by_id(db, id).await?;

    let title = match input.title.as_deref() {
        Some(t) => validate_title(t)?,
        None => &existing.title,
    };
    let description = match input.description.as_deref() {
        Some(d) => {
            let trimmed = d.trim();
            if trimmed.len() > 1024 {
                return Err(CoreError::validation("Description du motif trop longue"));
            }
            trimmed
        }
        None => &existing.description,
    };
    let category = match input.category.as_deref() {
        Some(c) => validate_category(c)?,
        None => &existing.category,
    };
    let is_synced = input.is_synced.unwrap_or(existing.is_synced);

    let document = match input.document {
        Some(ref doc) => {
            document::project("", doc)?;
            doc
        }
        None => &existing.document,
    };

    let updated = sqlx::query_as::<_, SyncedPattern>(
        "UPDATE synced_patterns
         SET title = $2, description = $3, category = $4, document = $5, is_synced = $6, updated_at = NOW()
         WHERE id = $1
         RETURNING id, slug, title, description, category, document, is_synced, created_at, updated_at"
    )
    .bind(id)
    .bind(title)
    .bind(description)
    .bind(category)
    .bind(document)
    .bind(is_synced)
    .fetch_one(db)
    .await?;

    Ok(updated)
}

pub async fn delete(db: &PgPool, id: Uuid) -> CoreResult<bool> {
    let res = sqlx::query("DELETE FROM synced_patterns WHERE id = $1")
        .bind(id)
        .execute(db)
        .await?;
    Ok(res.rows_affected() > 0)
}

pub async fn count(db: &PgPool) -> CoreResult<i64> {
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM synced_patterns")
        .fetch_one(db)
        .await?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn validates_slugs_and_titles() {
        assert!(valid_slug("hero-cta"));
        assert!(valid_slug("features_grid_2"));
        assert!(!valid_slug("Invalid Slug!"));
        assert!(!valid_slug("slug/with/slash"));
        assert!(!valid_slug(""));

        assert!(validate_title("Titre Valide").is_ok());
        assert!(validate_title("").is_err());
        assert!(validate_title("Titre\nInvalide").is_err());
        assert!(validate_title(&"a".repeat(256)).is_err());
    }

    #[test]
    fn validates_pattern_document_schema() {
        let valid_doc = json!({
            "rf_schema": "rf.document/1",
            "rf_blocks": [
                {
                    "rf_id": "b1",
                    "rf_type": "heading",
                    "rf_text": "Titre du motif",
                    "rf_level": 2
                },
                {
                    "rf_id": "b2",
                    "rf_type": "paragraph",
                    "rf_text": "Description réutilisable"
                }
            ]
        });
        assert!(document::project("", &valid_doc).is_ok());

        let invalid_doc = json!({
            "rf_schema": "rf.document/1",
            "rf_blocks": [
                {
                    "rf_id": "err",
                    "rf_type": "unknown_type"
                }
            ]
        });
        assert!(document::project("", &invalid_doc).is_err());
    }
}
