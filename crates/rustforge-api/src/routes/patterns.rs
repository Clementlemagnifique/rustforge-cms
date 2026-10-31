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
use rustforge_core::patterns::{self, NewPattern, SyncedPattern, UpdatePattern};

#[derive(Debug, Deserialize)]
pub struct PatternQuery {
    pub category: Option<String>,
    pub slug: Option<String>,
}

pub async fn list(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<PatternQuery>,
) -> ApiResult<Json<Vec<SyncedPattern>>> {
    let mut patterns = patterns::list(&state.db).await?;
    if let Some(category) = query
        .category
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        patterns.retain(|p| p.category.eq_ignore_ascii_case(category));
    }
    if let Some(slug) = query
        .slug
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        patterns.retain(|p| p.slug.eq_ignore_ascii_case(slug));
    }
    Ok(Json(patterns))
}

pub async fn get(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<SyncedPattern>> {
    let pattern = patterns::get_by_id(&state.db, id).await?;
    Ok(Json(pattern))
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(input): Json<NewPattern>,
) -> ApiResult<Json<SyncedPattern>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    let pattern = patterns::create(&state.db, input).await?;
    state.ssr.invalidate_all().await;
    Ok(Json(pattern))
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdatePattern>,
) -> ApiResult<Json<SyncedPattern>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    let pattern = patterns::update(&state.db, id, input).await?;
    state.ssr.invalidate_all().await;
    Ok(Json(pattern))
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    patterns::delete(&state.db, id).await?;
    state.ssr.invalidate_all().await;
    Ok(Json(serde_json::json!({ "success": true })))
}
