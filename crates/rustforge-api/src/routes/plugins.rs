use axum::{
    Json,
    extract::{Path, State},
};

use crate::{
    auth::AuthUser,
    error::{ApiError, ApiResult},
    state::AppState,
};
use rustforge_plugins::PluginInfo;

pub async fn list(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> Json<Vec<PluginInfo>> {
    Json(state.plugins.list())
}

pub async fn rescan(
    State(state): State<AppState>,
    auth: AuthUser,
) -> ApiResult<Json<Vec<PluginInfo>>> {
    if !auth.user.role.is_admin() {
        return Err(ApiError::Forbidden);
    }
    let list = state.plugins.rescan().await.map_err(|e| ApiError::Internal(e))?;
    Ok(Json(list))
}

pub async fn enable(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    if !auth.user.role.is_admin() {
        return Err(ApiError::Forbidden);
    }
    let info = state.plugins.enable(&id).await.map_err(|e| ApiError::BadRequest(e.to_string()))?;
    match info {
        Some(plugin) => Ok(Json(serde_json::to_value(plugin).unwrap())),
        None => Err(ApiError::NotFound),
    }
}

pub async fn disable(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    if !auth.user.role.is_admin() {
        return Err(ApiError::Forbidden);
    }
    let info = state.plugins.disable(&id).await.map_err(|e| ApiError::BadRequest(e.to_string()))?;
    match info {
        Some(plugin) => Ok(Json(serde_json::to_value(plugin).unwrap())),
        None => Err(ApiError::NotFound),
    }
}
