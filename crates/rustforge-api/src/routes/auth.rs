use axum::{Json, extract::State};
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
    let user =
        rustforge_core::users::authenticate(&state.db, &payload.login, &payload.password).await?;
    let token =
        create_token(user.id, user.role, &state.jwt_secret).map_err(|e| anyhow::anyhow!("{e}"))?;
    let _ =
        rustforge_core::users::issue_refresh_token(&state.db, user.id, chrono::Duration::days(7))
            .await;

    Ok(Json(LoginResponse { token, user }))
}

pub async fn me(auth: AuthUser) -> Json<User> {
    Json(auth.user)
}

/// Connexion publique bornée aux commentaires, sans profil administratif ni refresh token.
pub async fn comment_login(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> ApiResult<(axum::http::HeaderMap, Json<serde_json::Value>)> {
    if payload.login.len() > 320 || payload.password.len() > 1024 {
        return Err(crate::error::ApiError::BadRequest("Identifiants trop longs".into()));
    }
    let user = rustforge_core::users::authenticate(&state.db, &payload.login, &payload.password).await?;
    let token = crate::auth::create_comment_token(user.id, user.role, &state.jwt_secret)
        .map_err(|error| anyhow::anyhow!("{error}"))?;
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(axum::http::header::CACHE_CONTROL, axum::http::HeaderValue::from_static("no-store"));
    Ok((headers, Json(serde_json::json!({"token":token,"display_name":user.display_name,"expires_in":1800}))))
}
