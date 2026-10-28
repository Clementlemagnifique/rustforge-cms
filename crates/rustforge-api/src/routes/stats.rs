use axum::{Json, extract::State};

use crate::{auth::AuthUser, error::ApiResult, state::AppState};
use rustforge_core::stats::{DashboardStats, dashboard_for};

pub async fn get_dashboard(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> ApiResult<Json<DashboardStats>> {
    rustforge_core::security::require_writer(_auth.user.role)?;
    let author = if _auth.user.role.can_edit_others() {
        None
    } else {
        Some(_auth.user.id)
    };
    let stats = dashboard_for(&state.db, author).await?;
    Ok(Json(stats))
}
