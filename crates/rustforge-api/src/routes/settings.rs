use std::collections::BTreeMap;

use axum::{
    Json,
    extract::State,
};
use serde_json::Value;

use crate::{
    auth::AuthUser,
    error::{ApiError, ApiResult},
    state::AppState,
};
use rustforge_core::settings;

pub async fn get_all(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> ApiResult<Json<BTreeMap<String, Value>>> {
    let all = settings::all(&state.db).await?;
    Ok(Json(all))
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(values): Json<BTreeMap<String, Value>>,
) -> ApiResult<Json<BTreeMap<String, Value>>> {
    if !auth.user.role.is_admin() {
        return Err(ApiError::Forbidden);
    }
    let updated = settings::update(&state.db, values).await?;
    Ok(Json(updated))
}
