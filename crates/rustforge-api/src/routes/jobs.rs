//! Diagnostic administrateur : aucune projection implicite des payloads ou baux.
use crate::{
    auth::AuthUser,
    error::{ApiError, ApiResult},
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::header,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use rustforge_core::{CoreError, jobs, mail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

const STATES: &[&str] = &[
    "queued",
    "running",
    "retry",
    "succeeded",
    "failed",
    "cancelled",
    "paused_restore",
];
fn private(value: impl Serialize) -> Response {
    ([(header::CACHE_CONTROL, "no-store")], Json(value)).into_response()
}
fn admin(auth: &AuthUser) -> ApiResult<()> {
    if !auth.user.role.is_admin() {
        return Err(ApiError::Forbidden);
    }
    Ok(())
}
#[derive(Deserialize, Default)]
pub struct Filter {
    state: Option<String>,
    page: Option<i64>,
    per_page: Option<i64>,
}
#[derive(Serialize, sqlx::FromRow)]
struct Summary {
    id: Uuid,
    kind: String,
    state: String,
    payload_version: i32,
    attempts: i32,
    max_attempts: i32,
    available_at: DateTime<Utc>,
    lease_until: Option<DateTime<Utc>>,
    error_code: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    finished_at: Option<DateTime<Utc>>,
    capture_available: bool,
}
pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(filter): Query<Filter>,
) -> ApiResult<Response> {
    admin(&auth)?;
    let page = filter.page.unwrap_or(1);
    let per_page = filter.per_page.unwrap_or(20);
    if !(1..=1_000_000).contains(&page)
        || !(1..=100).contains(&per_page)
        || filter
            .state
            .as_ref()
            .is_some_and(|s| !STATES.contains(&s.as_str()))
    {
        return Err(ApiError::BadRequest("Filtre ou pagination invalide".into()));
    }
    let mut tx = state.db.begin().await.map_err(CoreError::from)?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .execute(&mut *tx)
        .await
        .map_err(CoreError::from)?;
    let counts: Vec<(String, i64)> =
        sqlx::query_as("SELECT state,count(*) FROM rf_jobs GROUP BY state")
            .fetch_all(&mut *tx)
            .await
            .map_err(CoreError::from)?;
    let total: i64 = counts
        .iter()
        .filter(|(s, _)| filter.state.as_ref().is_none_or(|f| f == s))
        .map(|(_, n)| n)
        .sum();
    let pages = ((total + per_page - 1) / per_page).max(1);
    if page > pages {
        return Err(ApiError::NotFound);
    }
    let items: Vec<Summary> = sqlx::query_as("SELECT id,kind,state,payload_version,attempts,max_attempts,available_at,lease_until,error_code,created_at,updated_at,finished_at,(output->>'transport'='capture' AND jsonb_typeof(output->'capture')='object') IS TRUE AS capture_available FROM rf_jobs WHERE ($1::text IS NULL OR state=$1) ORDER BY created_at DESC,id DESC LIMIT $2 OFFSET $3")
        .bind(&filter.state).bind(per_page).bind((page-1)*per_page).fetch_all(&mut *tx).await.map_err(CoreError::from)?;
    tx.commit().await.map_err(CoreError::from)?;
    let (_, mail_mode) = mail::load_transport(&state.db).await;
    Ok(private(
        json!({"items":items,"total":total,"page":page,"pages":pages,"counts":counts.into_iter().collect::<std::collections::BTreeMap<_,_>>(),"mail_mode":mail_mode}),
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Expected {
    expected_updated_at: DateTime<Utc>,
}
pub async fn action(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, action)): Path<(Uuid, String)>,
    Json(body): Json<Expected>,
) -> ApiResult<Response> {
    admin(&auth)?;
    let expected = Some(body.expected_updated_at);
    match action.as_str() {
        "cancel" => jobs::cancel_if_current(&state.db, id, expected).await?,
        "retry" => jobs::retry_if_current(&state.db, id, expected).await?,
        "resume-restored" => jobs::resume_restored_if_current(&state.db, id, expected).await?,
        _ => return Err(ApiError::NotFound),
    }
    Ok(private(json!({"ok":true})))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Batch {
    limit: u32,
}
pub async fn capture_batch(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<Batch>,
) -> ApiResult<Response> {
    admin(&auth)?;
    if !(1..=10).contains(&body.limit) {
        return Err(ApiError::BadRequest("Lot limité à 10 tâches".into()));
    }
    // Cette route et son bouton annoncent une capture locale, jamais un envoi réel.
    let mail_mode = "capture";
    let handled = mail::run_batch(&state.db, &mail::CaptureTransport, body.limit as usize).await?;
    Ok(private(json!({"handled":handled,"mail_mode":mail_mode})))
}
pub async fn capture(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Response> {
    admin(&auth)?;
    let output: Option<Value> = sqlx::query_scalar("SELECT output FROM rf_jobs WHERE id=$1 AND state='succeeded' AND output->>'transport'='capture'")
        .bind(id).fetch_optional(&state.db).await.map_err(CoreError::from)?.flatten();
    let output = output.ok_or(ApiError::NotFound)?;
    let message: CapturePreview =
        serde_json::from_value(output["capture"].clone()).map_err(|_| ApiError::NotFound)?;
    Ok(private(
        json!({"recipient":message.recipient,"subject":message.subject,"text":message.text}),
    ))
}

#[derive(Deserialize)]
struct CapturePreview {
    recipient: String,
    subject: String,
    text: String,
}
pub async fn no_store(mut response: Response) -> Response {
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-store"),
    );
    response
}
