use axum::{
    Json,
    extract::State,
};

use crate::{
    auth::AuthUser,
    error::ApiResult,
    state::AppState,
};
use rustforge_core::stats::{DashboardStats, dashboard};

pub async fn get_dashboard(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> ApiResult<Json<DashboardStats>> {
    let stats = dashboard(&state.db).await?;
    Ok(Json(stats))
}
