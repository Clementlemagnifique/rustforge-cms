//! Planificateur Éditorial Souverain & Dépublication Automatique (Tokio Background Task).
//!
//! Exécute périodiquement :
//! 1. La publication programmée des articles / pages dont la date de publication est échue.
//! 2. La dépublication automatique (PublishPress Expiration style) des contenus dont `scheduled_unpublish_at` est dépassé.

use chrono::Utc;
use sqlx::PgPool;
use std::time::Duration;
use tracing::{info, warn};

#[derive(Debug, Default, Clone)]
pub struct SchedulerTickReport {
    pub auto_published_count: usize,
    pub auto_unpublished_count: usize,
}

/// Exécute un cycle de vérification et d'application du planning éditorial.
pub async fn run_scheduler_tick(db: &PgPool) -> Result<SchedulerTickReport, sqlx::Error> {
    let now = Utc::now();
    let mut report = SchedulerTickReport::default();

    // 1. Dépublication automatique des contenus arrivés à expiration
    let unpublished_res = sqlx::query(
        r#"UPDATE entries
           SET status = 'archived'::entry_status, updated_at = now()
           WHERE status = 'published'::entry_status
             AND scheduled_unpublish_at IS NOT NULL
             AND scheduled_unpublish_at <= $1"#,
    )
    .bind(now)
    .execute(db)
    .await?;

    report.auto_unpublished_count = unpublished_res.rows_affected() as usize;
    if report.auto_unpublished_count > 0 {
        info!(
            "🕒 [Scheduler] Dépublication automatique de {} contenu(s) expiré(s)",
            report.auto_unpublished_count
        );
    }

    // 2. Publication automatique des contenus planifiés en attente
    let published_res = sqlx::query(
        r#"UPDATE entries
           SET status = 'published'::entry_status, updated_at = now()
           WHERE status = 'draft'::entry_status
             AND published_at IS NOT NULL
             AND published_at <= $1"#,
    )
    .bind(now)
    .execute(db)
    .await?;

    report.auto_published_count = published_res.rows_affected() as usize;
    if report.auto_published_count > 0 {
        info!(
            "🚀 [Scheduler] Publication automatique de {} contenu(s) planifié(s)",
            report.auto_published_count
        );
    }

    Ok(report)
}

/// Démarre la tâche de fond Tokio assurant la planification à la minute près.
pub fn start_scheduler(db: PgPool, interval_secs: u64) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        info!(
            "🕒 Démarrage du Planificateur Éditorial Souverain (vérification toutes les {}s)",
            interval_secs
        );
        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
        loop {
            interval.tick().await;
            if let Err(e) = run_scheduler_tick(&db).await {
                warn!("⚠️ Erreur lors de l'exécution du planificateur éditorial : {e}");
            }
        }
    })
}
