use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    error::{ApiError, ApiResult},
    state::AppState,
};
use rustforge_core::{
    content_types::{self, ContentTypeInput},
    models::ContentType,
};

pub async fn list(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> ApiResult<Json<Vec<ContentType>>> {
    let types = content_types::list(&state.db).await?;
    Ok(Json(types))
}

pub async fn get(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<ContentType>> {
    let ct = content_types::get(&state.db, id).await?;
    Ok(Json(ct))
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(input): Json<ContentTypeInput>,
) -> ApiResult<Json<ContentType>> {
    if !auth.user.role.is_admin() {
        return Err(ApiError::Forbidden);
    }
    let ct = content_types::create(&state.db, input).await?;
    Ok(Json(ct))
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<ContentTypeInput>,
) -> ApiResult<Json<ContentType>> {
    if !auth.user.role.is_admin() {
        return Err(ApiError::Forbidden);
    }
    let ct = content_types::update(&state.db, id, input).await?;
    Ok(Json(ct))
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    if !auth.user.role.is_admin() {
        return Err(ApiError::Forbidden);
    }
    content_types::delete(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}
