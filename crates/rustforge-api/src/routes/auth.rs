use axum::{
    Json,
    extract::State,
};
use serde::{Deserialize, Serialize};

use crate::{
    auth::{AuthUser, create_token},
    error::ApiResult,
    state::AppState,
};
use rustforge_core::models::User;

#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    pub login: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> ApiResult<Json<LoginResponse>> {
    let user = rustforge_core::users::authenticate(&state.db, &payload.login, &payload.password).await?;
    let token = create_token(user.id, user.role, &state.jwt_secret).map_err(|e| anyhow::anyhow!("{e}"))?;

    Ok(Json(LoginResponse { token, user }))
}

pub async fn me(auth: AuthUser) -> Json<User> {
    Json(auth.user)
}
