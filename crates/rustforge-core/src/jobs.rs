//! File durable commune. Pas de transport ni d'exécution de code arbitraire.
use crate::{CoreError, CoreResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Job {
    pub id: Uuid,
    pub kind: String,
    pub payload_version: i32,
    pub payload: Value,
    pub output: Option<Value>,
    pub dedup_key: String,
    pub state: String,
    pub priority: i32,
    pub attempts: i32,
    pub max_attempts: i32,
    pub available_at: DateTime<Utc>,
    pub lease_token: Option<Uuid>,
    pub lease_until: Option<DateTime<Utc>>,
    pub error_code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct NewJob {
    pub kind: String,
    pub payload_version: i32,
    pub payload: Value,
    pub dedup_key: String,
    pub max_attempts: i32,
    pub priority: i32,
    pub available_at: DateTime<Utc>,
}

fn valid_identifier(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.len() <= max
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._:-".contains(&b))
}

impl NewJob {
    pub fn validate(&self) -> CoreResult<()> {
        if !valid_identifier(&self.kind, 96)
            || !valid_identifier(&self.dedup_key, 200)
            || !(1..=65535).contains(&self.payload_version)
            || !(1..=20).contains(&self.max_attempts)
            || !(-100..=100).contains(&self.priority)
            || !self.payload.is_object()
            || self.payload.to_string().len() > 48 * 1024
        {
            return Err(CoreError::validation(
                "Job invalide : identifiant, version, quotas ou payload",
            ));
        }
        Ok(())
    }
}

pub async fn enqueue(db: &PgPool, spec: &NewJob) -> CoreResult<Job> {
    let mut tx = db.begin().await?;
    let job = enqueue_in(&mut tx, spec).await?;
    tx.commit().await?;
    Ok(job)
}

/// À utiliser dans la transaction de l'écriture métier (outbox).
pub async fn enqueue_in(connection: &mut PgConnection, spec: &NewJob) -> CoreResult<Job> {
    spec.validate()?;
    let inserted: Option<Job> = sqlx::query_as(
        "INSERT INTO rf_jobs(id,kind,payload_version,payload,dedup_key,max_attempts,priority,available_at)
         VALUES($1,$2,$3,$4,$5,$6,$7,$8) ON CONFLICT(dedup_key) DO NOTHING RETURNING *")
        .bind(Uuid::new_v4()).bind(&spec.kind).bind(spec.payload_version).bind(&spec.payload)
        .bind(&spec.dedup_key).bind(spec.max_attempts).bind(spec.priority).bind(spec.available_at)
        .fetch_optional(&mut *connection).await?;
    if let Some(job) = inserted {
        return Ok(job);
    }
    let existing: Job = sqlx::query_as("SELECT * FROM rf_jobs WHERE dedup_key=$1")
        .bind(&spec.dedup_key)
        .fetch_one(&mut *connection)
        .await?;
    if existing.kind != spec.kind
        || existing.payload_version != spec.payload_version
        || existing.payload != spec.payload
        || existing.max_attempts != spec.max_attempts
        || existing.priority != spec.priority
    {
        return Err(CoreError::Conflict(
            "Clé d'idempotence déjà utilisée pour un autre job".into(),
        ));
    }
    // Une répétition n'avance pas l'échéance d'un job existant et ne le réactive pas.
    Ok(existing)
}

fn validate_lease(seconds: i64) -> CoreResult<()> {
    if !(1..=600).contains(&seconds) {
        return Err(CoreError::validation(
            "Un bail doit durer entre 1 et 600 secondes",
        ));
    }
    Ok(())
}

/// Prend au plus un job d'un type explicitement pris en charge par le worker.
pub async fn claim(db: &PgPool, kinds: &[String], seconds: i64) -> CoreResult<Option<Job>> {
    validate_lease(seconds)?;
    if kinds.is_empty() || kinds.len() > 32 || kinds.iter().any(|s| !valid_identifier(s, 96)) {
        return Err(CoreError::validation("Types de jobs invalides"));
    }
    let mut tx = db.begin().await?;
    // Borner la maintenance et ne jamais attendre un autre worker.
    sqlx::query("WITH expired AS (
        SELECT id FROM rf_jobs WHERE state='running' AND lease_until<=now() AND attempts>=max_attempts
        ORDER BY lease_until,id LIMIT 100 FOR UPDATE SKIP LOCKED
    ) UPDATE rf_jobs j SET state='failed',lease_token=NULL,lease_until=NULL,
        error_code='lease_exhausted',finished_at=now(),updated_at=now()
        FROM expired e WHERE j.id=e.id").execute(&mut *tx).await?;
    let job = sqlx::query_as("WITH candidate AS (
        SELECT id FROM rf_jobs WHERE kind=ANY($1) AND attempts<max_attempts AND
        ((state IN ('queued','retry') AND available_at<=now()) OR (state='running' AND lease_until<=now()))
        ORDER BY priority DESC,available_at,id LIMIT 1 FOR UPDATE SKIP LOCKED
    ) UPDATE rf_jobs j SET state='running',attempts=attempts+1,lease_token=$2,
        lease_until=now()+($3::bigint * interval '1 second'),updated_at=now(),finished_at=NULL
        FROM candidate c WHERE j.id=c.id RETURNING j.*")
        .bind(kinds).bind(Uuid::new_v4()).bind(seconds).fetch_optional(&mut *tx).await?;
    tx.commit().await?;
    Ok(job)
}

fn owned(rows: u64) -> CoreResult<()> {
    if rows == 1 {
        Ok(())
    } else {
        Err(CoreError::Conflict(
            "Job absent, bail expiré ou remplacé, ou état incompatible".into(),
        ))
    }
}

pub async fn heartbeat(db: &PgPool, id: Uuid, token: Uuid, seconds: i64) -> CoreResult<()> {
    validate_lease(seconds)?;
    let rows = sqlx::query(
        "UPDATE rf_jobs SET lease_until=now()+($3::bigint * interval '1 second'),updated_at=now()
        WHERE id=$1 AND lease_token=$2 AND state='running' AND lease_until>now()",
    )
    .bind(id)
    .bind(token)
    .bind(seconds)
    .execute(db)
    .await?
    .rows_affected();
    owned(rows)
}

pub async fn complete(db: &PgPool, id: Uuid, token: Uuid) -> CoreResult<()> {
    finish(db, id, token, None).await
}

pub async fn complete_with_output(
    db: &PgPool,
    id: Uuid,
    token: Uuid,
    output: &Value,
) -> CoreResult<()> {
    if !output.is_object() || output.to_string().len() > 48 * 1024 {
        return Err(CoreError::validation(
            "Résultat de job invalide ou trop volumineux",
        ));
    }
    finish(db, id, token, Some(output)).await
}

async fn finish(db: &PgPool, id: Uuid, token: Uuid, output: Option<&Value>) -> CoreResult<()> {
    let rows = sqlx::query(
        "UPDATE rf_jobs SET state='succeeded',lease_token=NULL,lease_until=NULL,
        finished_at=now(),updated_at=now(),error_code=NULL,output=$3
        WHERE id=$1 AND lease_token=$2 AND state='running' AND lease_until>now()",
    )
    .bind(id)
    .bind(token)
    .bind(output)
    .execute(db)
    .await?
    .rows_affected();
    owned(rows)
}

/// Code stable non sensible uniquement : ne pas y placer une réponse de fournisseur.
pub async fn fail(
    db: &PgPool,
    id: Uuid,
    token: Uuid,
    code: &str,
    retryable: bool,
) -> CoreResult<()> {
    if !valid_identifier(code, 96) {
        return Err(CoreError::validation("Code d'erreur invalide"));
    }
    let rows = sqlx::query("UPDATE rf_jobs SET
        state=CASE WHEN $4 AND attempts<max_attempts THEN 'retry' ELSE 'failed' END,
        available_at=now()+((LEAST(3600,5*power(2,LEAST(attempts-1,10)))::int + get_byte(uuid_send(id),0)%5)*interval '1 second'),
        finished_at=CASE WHEN $4 AND attempts<max_attempts THEN NULL ELSE now() END,
        lease_token=NULL,lease_until=NULL,updated_at=now(),error_code=$3
        WHERE id=$1 AND lease_token=$2 AND state='running' AND lease_until>now()")
        .bind(id).bind(token).bind(code).bind(retryable).execute(db).await?.rows_affected();
    owned(rows)
}

pub async fn cancel(db: &PgPool, id: Uuid) -> CoreResult<()> {
    let rows = sqlx::query(
        "UPDATE rf_jobs SET state='cancelled',lease_token=NULL,lease_until=NULL,
        finished_at=now(),updated_at=now(),error_code='operator_cancelled'
        WHERE id=$1 AND state IN ('queued','retry','running','paused_restore')",
    )
    .bind(id)
    .execute(db)
    .await?
    .rows_affected();
    owned(rows)
}

/// Seul un opérateur autorisé doit appeler cette fonction, jamais un retry automatique.
pub async fn retry(db: &PgPool, id: Uuid) -> CoreResult<()> {
    let rows = sqlx::query(
        "UPDATE rf_jobs SET state='queued',attempts=0,available_at=now(),
        updated_at=now(),finished_at=NULL,error_code=NULL
        WHERE id=$1 AND state='failed'",
    )
    .bind(id)
    .execute(db)
    .await?
    .rows_affected();
    owned(rows)
}

pub async fn resume_restored(db: &PgPool, id: Uuid) -> CoreResult<()> {
    let rows = sqlx::query(
        "UPDATE rf_jobs SET
        state=CASE WHEN attempts<max_attempts THEN 'queued' ELSE 'failed' END,
        available_at=now(),updated_at=now(),
        finished_at=CASE WHEN attempts<max_attempts THEN NULL ELSE now() END,
        error_code='operator_resumed'
        WHERE id=$1 AND state='paused_restore'",
    )
    .bind(id)
    .execute(db)
    .await?
    .rows_affected();
    owned(rows)
}

/// Vue administrative bornée ; contient les payloads, ne pas exposer au public.
pub async fn list(db: &PgPool, limit: i64) -> CoreResult<Vec<Job>> {
    if !(1..=100).contains(&limit) {
        return Err(CoreError::validation("Limite de liste invalide"));
    }
    Ok(
        sqlx::query_as("SELECT * FROM rf_jobs ORDER BY created_at DESC,id DESC LIMIT $1")
            .bind(limit)
            .fetch_all(db)
            .await?,
    )
}

/// Rétention manuelle, uniquement les états terminaux, au plus 100 lignes par appel.
/// L'idempotence par clé est garantie tant que la ligne est conservée.
pub async fn prune(db: &PgPool, days: i32) -> CoreResult<u64> {
    if !(1..=3650).contains(&days) {
        return Err(CoreError::validation("Rétention invalide"));
    }
    Ok(sqlx::query(
        "WITH obsolete AS (
        SELECT id FROM rf_jobs WHERE state IN ('succeeded','failed','cancelled')
        AND finished_at<now()-($1::int*interval '1 day')
        ORDER BY finished_at,id LIMIT 100 FOR UPDATE SKIP LOCKED
    ) DELETE FROM rf_jobs j USING obsolete o WHERE j.id=o.id",
    )
    .bind(days)
    .execute(db)
    .await?
    .rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_unbounded_or_unversioned_payloads() {
        let mut spec = NewJob {
            kind: "rf.test".into(),
            payload_version: 1,
            payload: serde_json::json!({}),
            dedup_key: "test:1".into(),
            max_attempts: 5,
            priority: 0,
            available_at: Utc::now(),
        };
        assert!(spec.validate().is_ok());
        spec.payload = serde_json::json!({"body":"x".repeat(50*1024)});
        assert!(spec.validate().is_err());
        spec.payload = serde_json::json!({});
        spec.payload_version = 0;
        assert!(spec.validate().is_err());
        spec.payload_version = 1;
        spec.dedup_key = "secret\nvalue".into();
        assert!(spec.validate().is_err());
        assert!(validate_lease(601).is_err());
    }
}
