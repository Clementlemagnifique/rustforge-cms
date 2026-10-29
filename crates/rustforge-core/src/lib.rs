//! # RustForge Core
//!
//! Cœur métier du CMS, indépendant de la couche HTTP :
//! - [`models`] : types du domaine (utilisateurs, entrées, termes, médias…)
//! - services par domaine ([`entries`], [`users`], [`terms`]…) opérant sur un [`sqlx::PgPool`]
//! - [`hooks`] : système de filtres / actions branché sur les plugins
//! - [`markdown`], [`fields`] : rendu et validation du contenu

pub mod ai;
pub mod archive;
pub mod blocks;
pub mod comments;
pub mod comment_notifications;
pub mod content_types;
pub mod converter;
pub mod date_utils;
pub mod document;
pub mod ecommerce;
pub mod entries;
pub mod error;
pub mod fields;
pub mod form_validation;
pub mod forms;
pub mod galleries;
pub mod hooks;
pub mod jobs;
pub mod llm;
pub mod lms;
pub mod markdown;
pub mod mail;
pub mod media;
pub mod media_processor;
pub mod migration;
pub mod models;
pub mod pagination;
pub mod patterns;
pub mod scheduler;
pub mod security;
pub mod settings;
pub mod stats;
pub mod terms;
pub mod users;

pub use error::{CoreError, CoreResult};
pub use hooks::{HookRunner, NoopHooks};
pub use pagination::{Page, PageParams};

use sqlx::{PgPool, postgres::PgPoolOptions};

/// Ouvre un pool de connexions PostgreSQL.
pub async fn connect(database_url: &str) -> CoreResult<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(database_url)
        .await?;
    Ok(pool)
}

/// Applique les migrations SQL embarquées dans le binaire.
pub async fn migrate(pool: &PgPool) -> CoreResult<()> {
    sqlx::migrate!("../../migrations")
        .run(pool)
        .await
        .map_err(|e| CoreError::Internal(e.into()))?;
    Ok(())
}

/// Génère un slug URL à partir d'un texte libre (accents translittérés).
pub fn slugify(input: &str) -> String {
    let s = slug::slugify(input.trim());
    if s.is_empty() {
        "sans-titre".to_string()
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_handles_accents() {
        assert_eq!(slugify("  Été à Paris !  "), "ete-a-paris");
        assert_eq!(slugify("???"), "sans-titre");
    }
}
