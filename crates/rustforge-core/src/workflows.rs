//! Shared durable jobs for local campaign capture and manual automation recipes.
use crate::{CoreError, CoreResult, jobs, mail, users};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
pub const AUTOMATION_CAPTURE_KIND: &str = "rf.automation.mail.capture";
pub const CAPTURE_KIND: &str = "rf.mail.campaign.capture";
#[derive(Clone, Debug, Serialize, sqlx::FromRow)]
pub struct Campaign {
    pub id: Uuid,
    pub name: String,
    pub subject: String,
    pub body: String,
    pub recipient_ids: Vec<Uuid>,
    pub state: String,
    pub version: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CampaignInput {
    pub name: String,
    pub subject: String,
    pub body: String,
    pub recipient_ids: Vec<Uuid>,
    pub expected_version: Option<i64>,
}
fn validate(name: &str, subject: &str, body: &str) -> CoreResult<()> {
    if name.trim().is_empty()
        || name.chars().count() > 200
        || subject.trim().is_empty()
        || subject.len() > 512
        || subject.chars().any(char::is_control)
        || body.trim().is_empty()
        || body.len() > 32768
    {
        return Err(CoreError::validation("Nom, objet ou contenu invalide"));
    }
    Ok(())
}
pub async fn campaigns(db: &PgPool) -> CoreResult<Vec<Campaign>> {
    Ok(sqlx::query_as("SELECT id,name,subject,body_text AS body,recipient_ids,status AS state,version,created_at,updated_at FROM email_campaigns WHERE delivery_mode='capture' ORDER BY updated_at DESC LIMIT 100").fetch_all(db).await?)
}
pub async fn save_campaign(
    db: &PgPool,
    id: Option<Uuid>,
    actor: Uuid,
    mut input: CampaignInput,
) -> CoreResult<Campaign> {
    validate(&input.name, &input.subject, &input.body)?;
    input.recipient_ids.sort();
    input.recipient_ids.dedup();
    if input.recipient_ids.is_empty() || input.recipient_ids.len() > 100 {
        return Err(CoreError::validation(
            "Choisir entre 1 et 100 comptes destinataires",
        ));
    }
    for id in &input.recipient_ids {
        users::get(db, *id).await?;
    }
    let result = if let Some(id) = id {
        sqlx::query_as("UPDATE email_campaigns SET name=$2,subject=$3,body_text=$4,recipient_ids=$5,version=version+1,updated_at=now() WHERE id=$1 AND version=$6 AND status='draft' AND delivery_mode='capture' RETURNING id,name,subject,body_text AS body,recipient_ids,status AS state,version,created_at,updated_at").bind(id).bind(input.name.trim()).bind(input.subject.trim()).bind(input.body).bind(input.recipient_ids).bind(input.expected_version).fetch_optional(db).await?
    } else {
        sqlx::query_as("INSERT INTO email_campaigns(id,name,subject,body_text,recipient_ids,created_by,delivery_mode) VALUES($1,$2,$3,$4,$5,$6,'capture') RETURNING id,name,subject,body_text AS body,recipient_ids,status AS state,version,created_at,updated_at").bind(Uuid::new_v4()).bind(input.name.trim()).bind(input.subject.trim()).bind(input.body).bind(input.recipient_ids).bind(actor).fetch_optional(db).await?
    };
    result.ok_or_else(|| {
        CoreError::Conflict(
            "Campagne modifiée ou déjà mise en file ; rechargez avant de continuer".into(),
        )
    })
}
pub async fn queue_campaign(db: &PgPool, id: Uuid, expected: i64) -> CoreResult<Campaign> {
    let mut tx = db.begin().await?;
    let current:Campaign=sqlx::query_as("SELECT id,name,subject,body_text AS body,recipient_ids,status AS state,version,created_at,updated_at FROM email_campaigns WHERE id=$1 AND delivery_mode='capture' FOR UPDATE").bind(id).fetch_optional(&mut *tx).await?.ok_or(CoreError::NotFound)?;
    if current.state == "queued" {
        tx.commit().await?;
        return Ok(current);
    }
    if current.version != expected {
        return Err(CoreError::Conflict(
            "Campagne modifiée : relisez les destinataires".into(),
        ));
    }
    for recipient in &current.recipient_ids {
        let request = mail::MailRequest {
            recipient_id: *recipient,
            subject: current.subject.clone(),
            text: current.body.clone(),
        };
        request.validate()?;
        jobs::enqueue_in(
            &mut tx,
            &jobs::NewJob {
                kind: CAPTURE_KIND.into(),
                payload_version: 1,
                payload: serde_json::to_value(request)
                    .map_err(|e| CoreError::Internal(e.into()))?,
                dedup_key: format!("rf:campaign:{id}:{recipient}"),
                max_attempts: 5,
                priority: 0,
                available_at: Utc::now(),
            },
        )
        .await?;
    }
    let current=sqlx::query_as("UPDATE email_campaigns SET status='queued',version=version+1,updated_at=now() WHERE id=$1 RETURNING id,name,subject,body_text AS body,recipient_ids,status AS state,version,created_at,updated_at").bind(id).fetch_one(&mut *tx).await?;
    tx.commit().await?;
    Ok(current)
}
#[derive(Clone, Debug, Serialize, sqlx::FromRow)]
pub struct Recipe {
    pub id: Uuid,
    pub name: String,
    pub subject: String,
    pub body: String,
    pub recipient_id: Option<Uuid>,
    pub delay_seconds: i32,
    pub active: bool,
    pub version: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecipeInput {
    pub name: String,
    pub subject: String,
    pub body: String,
    pub recipient_id: Uuid,
    pub delay_seconds: i32,
    pub active: bool,
    pub expected_version: Option<i64>,
}
#[derive(Serialize, sqlx::FromRow)]
pub struct Run {
    pub id: Uuid,
    pub recipe_id: Uuid,
    pub invocation_id: Uuid,
    pub job_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub state: String,
}
pub async fn recipes(db: &PgPool) -> CoreResult<Vec<Recipe>> {
    Ok(sqlx::query_as("SELECT id,title AS name,subject,body_text AS body,recipient_id,delay_seconds,is_active AS active,version,created_at,updated_at FROM automation_recipes WHERE trigger_type='manual_capture' ORDER BY updated_at DESC LIMIT 100").fetch_all(db).await?)
}
pub async fn save_recipe(
    db: &PgPool,
    id: Option<Uuid>,
    actor: Uuid,
    input: RecipeInput,
) -> CoreResult<Recipe> {
    validate(&input.name, &input.subject, &input.body)?;
    if !(0..=2592000).contains(&input.delay_seconds) {
        return Err(CoreError::validation("Délai compris entre 0 et 30 jours"));
    }
    users::get(db, input.recipient_id).await?;
    let row = if let Some(id) = id {
        sqlx::query_as("UPDATE automation_recipes SET title=$2,subject=$3,body_text=$4,recipient_id=$5,delay_seconds=$6,is_active=$7,version=version+1,updated_at=now() WHERE id=$1 AND version=$8 AND trigger_type='manual_capture' RETURNING id,title AS name,subject,body_text AS body,recipient_id,delay_seconds,is_active AS active,version,created_at,updated_at").bind(id).bind(input.name.trim()).bind(input.subject.trim()).bind(input.body).bind(input.recipient_id).bind(input.delay_seconds).bind(input.active).bind(input.expected_version).fetch_optional(db).await?
    } else {
        sqlx::query_as("INSERT INTO automation_recipes(id,title,subject,body_text,recipient_id,delay_seconds,is_active,created_by,trigger_type,actions) VALUES($1,$2,$3,$4,$5,$6,$7,$8,'manual_capture',jsonb_build_array(jsonb_build_object('type','mail.capture'))) RETURNING id,title AS name,subject,body_text AS body,recipient_id,delay_seconds,is_active AS active,version,created_at,updated_at").bind(Uuid::new_v4()).bind(input.name.trim()).bind(input.subject.trim()).bind(input.body).bind(input.recipient_id).bind(input.delay_seconds).bind(input.active).bind(actor).fetch_optional(db).await?
    };
    row.ok_or_else(|| CoreError::Conflict("Recette modifiée ; rechargez avant de continuer".into()))
}
pub async fn run_recipe(db: &PgPool, id: Uuid, invocation: Uuid, expected: i64) -> CoreResult<Run> {
    let mut tx = db.begin().await?;
    let recipe:Recipe=sqlx::query_as("SELECT id,title AS name,subject,body_text AS body,recipient_id,delay_seconds,is_active AS active,version,created_at,updated_at FROM automation_recipes WHERE id=$1 AND trigger_type='manual_capture' FOR UPDATE").bind(id).fetch_optional(&mut *tx).await?.ok_or(CoreError::NotFound)?;
    if let Some(run)=sqlx::query_as::<_,Run>("SELECT l.id,l.recipe_id,l.invocation_id,l.job_id,l.executed_at AS created_at,j.state FROM automation_logs l JOIN rf_jobs j ON j.id=l.job_id WHERE l.recipe_id=$1 AND l.invocation_id=$2").bind(id).bind(invocation).fetch_optional(&mut *tx).await?{tx.commit().await?;return Ok(run);}
    if !recipe.active || recipe.version != expected {
        return Err(CoreError::Conflict(
            "Recette désactivée ou modifiée ; rechargez".into(),
        ));
    }
    let request = mail::MailRequest {
        recipient_id: recipe.recipient_id.ok_or_else(|| {
            CoreError::validation("Le compte destinataire a été supprimé ; modifiez la recette")
        })?,
        subject: recipe.subject,
        text: recipe.body,
    };
    request.validate()?;
    let job = jobs::enqueue_in(
        &mut tx,
        &jobs::NewJob {
            kind: AUTOMATION_CAPTURE_KIND.into(),
            payload_version: 1,
            payload: serde_json::to_value(request).map_err(|e| CoreError::Internal(e.into()))?,
            dedup_key: format!("rf:auto:{id}:{invocation}"),
            max_attempts: 5,
            priority: 0,
            available_at: Utc::now() + Duration::seconds(i64::from(recipe.delay_seconds)),
        },
    )
    .await?;
    let run=sqlx::query_as("INSERT INTO automation_logs(id,recipe_id,invocation_id,job_id,success,details) VALUES($1,$2,$3,$4,false,jsonb_build_object('state','queued')) RETURNING id,recipe_id,invocation_id,job_id,executed_at AS created_at,'queued'::text AS state").bind(Uuid::new_v4()).bind(id).bind(invocation).bind(job.id).fetch_one(&mut *tx).await?;
    tx.commit().await?;
    Ok(run)
}
pub async fn runs(db: &PgPool, id: Uuid) -> CoreResult<Vec<Run>> {
    Ok(sqlx::query_as("SELECT l.id,l.recipe_id,l.invocation_id,l.job_id,l.executed_at AS created_at,j.state FROM automation_logs l JOIN rf_jobs j ON j.id=l.job_id WHERE l.recipe_id=$1 AND l.invocation_id IS NOT NULL ORDER BY l.executed_at DESC LIMIT 100").bind(id).fetch_all(db).await?)
}
