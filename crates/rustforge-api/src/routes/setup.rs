//! Installation requires a one-time operator key, never an unclaimed public admin.
use crate::{
    auth::create_token,
    error::{ApiError, ApiResult},
    state::AppState,
};
use axum::{
    Json,
    extract::{Multipart, State},
};
use rustforge_core::{
    models::{Role, User},
    users::{self, NewUser},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize)]
pub struct SetupStatusResponse {
    pub is_installed: bool,
    pub version: &'static str,
    pub database_connected: bool,
}

#[derive(Deserialize)]
pub struct InstallRequest {
    pub setup_token: String,
    pub site_title: String,
    pub site_url: Option<String>,
    pub admin_email: String,
    pub admin_username: String,
    pub admin_password: String,
    pub site_tagline: Option<String>,
}

#[derive(Serialize)]
pub struct InstallResponse {
    pub success: bool,
    pub message: String,
    pub token: String,
    pub user: User,
}

pub async fn status(State(state): State<AppState>) -> ApiResult<Json<SetupStatusResponse>> {
    let installed: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE role='admin')")
            .fetch_one(&state.db)
            .await
            .map_err(rustforge_core::CoreError::from)?;
    Ok(Json(SetupStatusResponse {
        is_installed: installed,
        version: env!("CARGO_PKG_VERSION"),
        database_connected: true,
    }))
}

pub async fn install(
    State(state): State<AppState>,
    Json(payload): Json<InstallRequest>,
) -> ApiResult<Json<InstallResponse>> {
    let expected = state.setup_token.as_deref().ok_or(ApiError::Forbidden)?;
    if !rustforge_core::security::secret_matches(expected, &payload.setup_token) {
        return Err(ApiError::Forbidden);
    }
    if payload.site_title.trim().is_empty() || payload.site_title.len() > 300 {
        return Err(ApiError::BadRequest("Titre de site invalide".into()));
    }
    let site_url = if let Some(value) = payload.site_url {
        let url = value
            .trim()
            .parse::<reqwest::Url>()
            .map_err(|_| ApiError::BadRequest("Adresse du site invalide".into()))?;
        if !matches!(url.scheme(), "http" | "https")
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err(ApiError::BadRequest(
                "Adresse du site HTTP(S) sans identifiants ni paramètres requise".into(),
            ));
        }
        Some(url.as_str().trim_end_matches('/').to_string())
    } else {
        None
    };
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(rustforge_core::CoreError::from)?;
    sqlx::query("SELECT pg_advisory_xact_lock(79643201)")
        .execute(&mut *tx)
        .await
        .map_err(rustforge_core::CoreError::from)?;
    let installed: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE role='admin')")
            .fetch_one(&mut *tx)
            .await
            .map_err(rustforge_core::CoreError::from)?;
    if installed {
        return Err(ApiError::Forbidden);
    }
    let user = users::create_in(
        &mut tx,
        NewUser {
            email: payload.admin_email,
            username: payload.admin_username,
            display_name: None,
            password: payload.admin_password,
            role: Role::Admin,
        },
    )
    .await?;
    for (key, value) in [
        ("site.title", payload.site_title),
        ("site.tagline", payload.site_tagline.unwrap_or_default()),
    ] {
        sqlx::query("INSERT INTO settings(key,value) VALUES($1,$2) ON CONFLICT(key) DO UPDATE SET value=EXCLUDED.value, updated_at=now()")
            .bind(key).bind(json!(value.trim())).execute(&mut *tx).await.map_err(rustforge_core::CoreError::from)?;
    }
    if let Some(url) = site_url {
        sqlx::query("INSERT INTO settings(key,value) VALUES('site.url',$1) ON CONFLICT(key) DO UPDATE SET value=EXCLUDED.value,updated_at=now()")
            .bind(json!(url)).execute(&mut *tx).await.map_err(rustforge_core::CoreError::from)?;
    }
    let token = create_token(user.id, user.role, &state.jwt_secret)
        .map_err(|e| ApiError::Internal(e.into()))?;
    tx.commit().await.map_err(rustforge_core::CoreError::from)?;
    Ok(Json(InstallResponse {
        success: true,
        message: "Installation terminée.".into(),
        token,
        user,
    }))
}

pub async fn restore_capsule(
    State(_state): State<AppState>,
    _multipart: Multipart,
) -> ApiResult<Json<serde_json::Value>> {
    Err(ApiError::BadRequest("Créez un administrateur puis restaurez depuis le Studio, ou utilisez la CLI sur une base vierge.".into()))
}
