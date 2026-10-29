//! Native database snapshots. Editorial RFIR is deliberately not a database backup.
use crate::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use std::collections::BTreeMap;

pub const TABLES: &[&str] = &[
    "users",
    "refresh_tokens",
    "media_folders",
    "media",
    "content_types",
    "taxonomies",
    "terms",
    "entries",
    "entry_terms",
    "revisions",
    "comments",
    "settings",
    "plugins",
    "forms",
    "form_submissions",
    "galleries",
    "gallery_items",
    "lms_courses",
    "lms_modules",
    "lms_lessons",
    "lms_enrollments",
    "ecommerce_products",
    "ecommerce_discounts",
    "ecommerce_orders",
    "ecommerce_order_items",
    "automation_recipes",
    "automation_logs",
    "email_subscribers",
    "email_campaigns",
    "security_audit_logs",
    "analytics_pageviews",
    "import_origins",
    "ai_action_plans",
    "rf_jobs",
];

#[derive(Debug, Serialize, Deserialize)]
pub struct NativeSnapshot {
    pub format: String,
    pub migrations: Vec<i64>,
    pub tables: BTreeMap<String, Vec<Value>>,
}

pub async fn dump(pool: &PgPool) -> CoreResult<NativeSnapshot> {
    let mut tx = pool.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
        .execute(&mut *tx)
        .await?;
    let migrations =
        sqlx::query_scalar("SELECT version FROM _sqlx_migrations WHERE success ORDER BY version")
            .fetch_all(&mut *tx)
            .await?;
    let mut tables = BTreeMap::new();
    for table in TABLES {
        let query = if *table == "settings" {
            "SELECT to_jsonb(t) FROM settings t WHERE t.key != 'mail.smtp_password'".to_string()
        } else {
            format!("SELECT to_jsonb(t) FROM {table} t")
        };
        let rows = sqlx::query_scalar::<_, Value>(&query)
            .fetch_all(&mut *tx)
            .await?;
        tables.insert((*table).into(), rows);
    }
    tx.commit().await?;
    Ok(NativeSnapshot {
        format: "rustforge-native-3".into(),
        migrations,
        tables,
    })
}

impl NativeSnapshot {
    pub fn validate(&self) -> CoreResult<()> {
        let legacy = self.format == "rustforge-native-2";
        let expected: Vec<_> = TABLES.iter().copied().filter(|t| !legacy || *t != "rf_jobs").collect();
        if (!legacy && self.format != "rustforge-native-3")
            || self.tables.len() != expected.len()
            || expected.iter().any(|t| !self.tables.contains_key(*t))
            || (legacy && self.migrations.contains(&20261029000007))
        {
            return Err(CoreError::validation("Sauvegarde native incomplète ou version incompatible"));
        }
        if !self.tables["users"].iter().any(|r| {
            r["role"] == "admin" && r["password_hash"].as_str().is_some_and(|s| !s.is_empty())
        }) {
            return Err(CoreError::validation(
                "La sauvegarde ne contient aucun administrateur récupérable",
            ));
        }
        if self
            .tables
            .values()
            .any(|rows| rows.iter().any(|r| !r.is_object()))
        {
            return Err(CoreError::validation("Ligne de sauvegarde invalide"));
        }
        Ok(())
    }
}

/// Restores within a caller-owned transaction. A configured, populated site is never truncated.
pub async fn restore(tx: &mut sqlx::PgConnection, snapshot: &NativeSnapshot) -> CoreResult<()> {
    snapshot.validate()?;
    sqlx::query("SELECT pg_advisory_xact_lock(79643201)")
        .execute(&mut *tx)
        .await?;
    // Exclude concurrent writes before checking that the destination is disposable.
    sqlx::query(&format!(
        "LOCK TABLE {} IN ACCESS EXCLUSIVE MODE",
        TABLES.join(",")
    ))
    .execute(&mut *tx)
    .await?;
    let versions: Vec<i64> =
        sqlx::query_scalar("SELECT version FROM _sqlx_migrations WHERE success ORDER BY version")
            .fetch_all(&mut *tx)
            .await?;
    let compatible_legacy = snapshot.format == "rustforge-native-2" && {
        let mut upgraded = snapshot.migrations.clone();
        upgraded.push(20261029000007);
        upgraded == versions
    };
    if versions != snapshot.migrations && !compatible_legacy {
        return Err(CoreError::validation(
            "Les migrations de la sauvegarde et de la cible diffèrent",
        ));
    }
    for table in TABLES {
        if matches!(
            *table,
            "content_types" | "taxonomies" | "terms" | "settings" | "users"
        ) {
            continue;
        }
        let count: i64 = sqlx::query_scalar(&format!("SELECT count(*) FROM {table}"))
            .fetch_one(&mut *tx)
            .await?;
        if count > 0 {
            return Err(CoreError::Conflict(
                "Restauration refusée : utilisez une nouvelle base sans données métier.".into(),
            ));
        }
    }
    let users: i64 = sqlx::query_scalar("SELECT count(*) FROM users")
        .fetch_one(&mut *tx)
        .await?;
    let custom_types: i64 =
        sqlx::query_scalar("SELECT count(*) FROM content_types WHERE NOT is_system")
            .fetch_one(&mut *tx)
            .await?;
    let custom_terms: i64 =
        sqlx::query_scalar("SELECT count(*) FROM terms WHERE slug <> 'non-classe'")
            .fetch_one(&mut *tx)
            .await?;
    let custom_taxonomies: i64 =
        sqlx::query_scalar("SELECT count(*) FROM taxonomies WHERE NOT is_system")
            .fetch_one(&mut *tx)
            .await?;
    let custom_settings:i64=sqlx::query_scalar("SELECT count(*) FROM settings WHERE key NOT IN ('site.title','site.tagline','site.url','site.language','reading.per_page')").fetch_one(&mut *tx).await?;
    if users > 1
        || custom_types > 0
        || custom_terms > 0
        || custom_taxonomies > 0
        || custom_settings > 0
    {
        return Err(CoreError::Conflict(
            "La cible contient déjà des données à conserver.".into(),
        ));
    }
    // Restrict to the tables in this format; no CASCADE into unknown application tables.
    sqlx::query(&format!("TRUNCATE {}", TABLES.join(",")))
        .execute(&mut *tx)
        .await?;
    let deferred = [
        ("media_folders", "parent_id"),
        ("terms", "parent_id"),
        ("entries", "parent_id"),
        ("comments", "parent_id"),
        ("lms_courses", "product_id"),
        ("ecommerce_products", "course_id"),
    ];
    for table in TABLES {
        let columns:Vec<String>=sqlx::query_scalar("SELECT column_name FROM information_schema.columns WHERE table_schema=current_schema() AND table_name=$1 AND is_generated='NEVER' ORDER BY ordinal_position")
            .bind(*table).fetch_all(&mut *tx).await?;
        let cols = columns
            .iter()
            .map(|c| format!("\"{c}\""))
            .collect::<Vec<_>>()
            .join(",");
        for original in snapshot.tables.get(*table).into_iter().flatten() {
            let mut row = original.clone();
            for (t, c) in deferred {
                if t == *table {
                    row[c] = Value::Null;
                }
            }
            sqlx::query(&format!("INSERT INTO {table} ({cols}) SELECT {cols} FROM jsonb_populate_record(NULL::{table},$1)"))
                .bind(row).execute(&mut *tx).await?;
        }
    }
    for (table, column) in deferred {
        for row in &snapshot.tables[table] {
            if !row[column].is_null() {
                sqlx::query(&format!(
                    "UPDATE {table} SET {column}=($1->>'{column}')::uuid WHERE id=($1->>'id')::uuid"
                ))
                .bind(row)
                .execute(&mut *tx)
                .await?;
            }
        }
    }
    // Aucun job actif importé ne doit envoyer un message ou rejouer un effet tout seul.
    sqlx::query("UPDATE rf_jobs SET state='paused_restore',lease_token=NULL,lease_until=NULL,
        error_code='restore_requires_review',updated_at=now(),finished_at=NULL
        WHERE state IN ('queued','retry','running')").execute(&mut *tx).await?;
    // Refresh tokens from another machine must not create valid sessions on the target.
    sqlx::query("UPDATE refresh_tokens SET revoked_at=now() WHERE revoked_at IS NULL")
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "UPDATE ai_action_plans SET status='cancelled' WHERE status IN ('pending','executing')",
    )
    .execute(&mut *tx)
    .await?;
    // Nettoyage des secrets d'infrastructure sensibles lors de la restauration
    sqlx::query("DELETE FROM settings WHERE key = 'mail.smtp_password'")
        .execute(&mut *tx)
        .await?;
    Ok(())
}
