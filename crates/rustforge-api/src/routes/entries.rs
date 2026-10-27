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

#[derive(Debug, Deserialize)]
pub struct DeleteQuery {
    #[serde(default)]
    pub force: bool,
}

pub async fn list(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<EntryQuery>,
) -> ApiResult<Json<Page<Entry>>> {
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
    let entry = entries::create(&state.db, state.plugins.as_ref(), &query.content_type, auth.user.id, input).await?;
    Ok(Json(entry))
}

pub async fn get(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Entry>> {
    let entry = entries::get(&state.db, id).await?;
    Ok(Json(entry))
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<EntryInput>,
) -> ApiResult<Json<Entry>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    let current = entries::get(&state.db, id).await?;
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
    let revs = entries::revisions(&state.db, id).await?;
    Ok(Json(revs))
}

pub async fn restore_revision(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, rev_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<Entry>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    let entry = entries::restore_revision(&state.db, state.plugins.as_ref(), id, rev_id, auth.user.id).await?;
    Ok(Json(entry))
}
