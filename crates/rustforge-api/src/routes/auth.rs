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

#[derive(Debug, Deserialize)]
pub struct RegisterPayload {
    pub login: String,
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
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

/// Inscription publique bornée aux rôles non-privilégiés selon la politique du site (WP-72).
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterPayload>,
) -> ApiResult<Json<User>> {
    // 1. Vérifie si les inscriptions sont ouvertes
    let can_register = rustforge_core::settings::get(&state.db, "users.can_register")
        .await?
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !can_register {
        return Err(crate::error::ApiError::Forbidden);
    }

    // 2. Détermine le rôle par défaut (strictement borné, jamais admin)
    let default_role_str = rustforge_core::settings::get(&state.db, "users.default_role")
        .await?
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_else(|| "subscriber".to_string());

    let role = match default_role_str.as_str() {
        "author" => rustforge_core::models::Role::Author,
        "editor" => rustforge_core::models::Role::Editor,
        _ => rustforge_core::models::Role::Subscriber,
    };

    // 3. Validation des champs
    let login = payload.login.trim().to_string();
    let email = payload.email.trim().to_string();
    if login.is_empty() || login.len() > 64 {
        return Err(crate::error::ApiError::BadRequest(
            "L'identifiant doit contenir entre 1 et 64 caractères.".into(),
        ));
    }
    if email.is_empty() || email.len() > 254 || !email.contains('@') {
        return Err(crate::error::ApiError::BadRequest(
            "Adresse e-mail invalide.".into(),
        ));
    }
    if payload.password.len() < 6 || payload.password.len() > 128 {
        return Err(crate::error::ApiError::BadRequest(
            "Le mot de passe doit comporter au moins 6 caractères.".into(),
        ));
    }

    let display_name = payload
        .display_name
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| login.clone());

    let user = rustforge_core::users::create(
        &state.db,
        rustforge_core::users::NewUser {
            username: login,
            email,
            password: payload.password,
            display_name: Some(display_name),
            role,
        },
    )
    .await?;

    Ok(Json(user))
}

/// Connexion publique bornée aux commentaires, sans profil administratif ni refresh token.
pub async fn comment_login(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> ApiResult<(axum::http::HeaderMap, Json<serde_json::Value>)> {
    if payload.login.len() > 320 || payload.password.len() > 1024 {
        return Err(crate::error::ApiError::BadRequest(
            "Identifiants trop longs".into(),
        ));
    }
    let user =
        rustforge_core::users::authenticate(&state.db, &payload.login, &payload.password).await?;
    let token = crate::auth::create_comment_token(user.id, user.role, &state.jwt_secret)
        .map_err(|error| anyhow::anyhow!("{error}"))?;
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        axum::http::header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-store"),
    );
    Ok((
        headers,
        Json(serde_json::json!({"token":token,"display_name":user.display_name,"expires_in":1800})),
    ))
}
