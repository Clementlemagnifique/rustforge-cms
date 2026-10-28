use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    error::{ApiError, ApiResult},
    state::AppState,
};
use rustforge_core::{
    Page,
    entries::{self, EntryQuery},
    models::{Entry, EntryInput, Revision},
};

#[derive(Debug, Deserialize)]
pub struct CreateEntryQuery {
    #[serde(default = "default_post")]
    pub content_type: String,
}

fn default_post() -> String {
    "post".to_string()
}

pub async fn preview(
    auth: AuthUser,
    Json(input): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    rustforge_core::security::require_writer(auth.user.role)?;
    let body = input
        .get("body")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| ApiError::BadRequest("Texte requis".into()))?;
    if body.len() > 512 * 1024 {
        return Err(ApiError::BadRequest(
            "Texte limité à 512 Kio pour l’aperçu".into(),
        ));
    }
    let blocks = input.get("blocks").cloned().unwrap_or(serde_json::Value::Null);
    let (body, body_html) = rustforge_core::document::project(body, &blocks)?;
    Ok(Json(serde_json::json!({"body":body,"body_html":body_html})))
}

pub async fn convert_blocks(
    auth: AuthUser,
    Json(input): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    rustforge_core::security::require_writer(auth.user.role)?;
    let body = input
        .get("body")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| ApiError::BadRequest("Texte source requis".into()))?;
    if body.len() > 512 * 1024 {
        return Err(ApiError::BadRequest(
            "Texte limité à 512 Kio pour la conversion".into(),
        ));
    }
    let blocks = rustforge_core::document::from_markdown(body)?;
    let (projected_body, body_html) = rustforge_core::document::project("", &blocks)?;
    Ok(Json(serde_json::json!({
        "blocks": blocks,
        "body": projected_body,
        "body_html": body_html,
    })))
}

#[derive(Debug, Deserialize)]
pub struct DeleteQuery {
    #[serde(default)]
    pub force: bool,
}

pub async fn list(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(mut query): Query<EntryQuery>,
) -> ApiResult<Json<Page<Entry>>> {
    rustforge_core::security::require_writer(_auth.user.role)?;
    if !_auth.user.role.can_edit_others() {
        query.author_id = Some(_auth.user.id);
    }
    let page = entries::list(&state.db, &query).await?;
    Ok(Json(page))
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<CreateEntryQuery>,
    Json(input): Json<EntryInput>,
) -> ApiResult<Json<Entry>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    let entry = entries::create(
        &state.db,
        state.plugins.as_ref(),
        &query.content_type,
        auth.user.id,
        input,
    )
    .await?;
    Ok(Json(entry))
}

pub async fn get(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Entry>> {
    let entry = entries::get(&state.db, id).await?;
    rustforge_core::security::require_entry_access(_auth.user.role, _auth.user.id, &entry)?;
    Ok(Json(entry))
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(patch): Json<serde_json::Value>,
) -> ApiResult<Json<Entry>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    let current = entries::get(&state.db, id).await?;
    rustforge_core::security::require_entry_access(auth.user.role, auth.user.id, &current)?;
    let input = EntryInput::from_patch(&current, patch)?;
    if !auth.user.role.can_edit_others() && current.author_id != Some(auth.user.id) {
        return Err(ApiError::Forbidden);
    }
    let entry = entries::update(&state.db, state.plugins.as_ref(), id, auth.user.id, input).await?;
    Ok(Json(entry))
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Query(query): Query<DeleteQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    let current = entries::get(&state.db, id).await?;
    if !auth.user.role.can_edit_others() && current.author_id != Some(auth.user.id) {
        return Err(ApiError::Forbidden);
    }
    entries::delete(&state.db, state.plugins.as_ref(), id, query.force).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn list_revisions(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Vec<Revision>>> {
    let current = entries::get(&state.db, id).await?;
    rustforge_core::security::require_entry_access(_auth.user.role, _auth.user.id, &current)?;
    let revs = entries::revisions(&state.db, id).await?;
    Ok(Json(revs))
}

#[derive(Debug, Default, Deserialize)]
pub struct RestoreRevisionPayload {
    pub expected_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn restore_revision(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, rev_id)): Path<(Uuid, Uuid)>,
    payload: Option<Json<RestoreRevisionPayload>>,
) -> ApiResult<Json<Entry>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    let current = entries::get(&state.db, id).await?;
    rustforge_core::security::require_entry_access(auth.user.role, auth.user.id, &current)?;
    let expected = payload.and_then(|Json(p)| p.expected_updated_at);
    let entry = entries::restore_revision_checked(
        &state.db,
        state.plugins.as_ref(),
        id,
        rev_id,
        auth.user.id,
        expected,
    )
    .await?;
    Ok(Json(entry))
}
