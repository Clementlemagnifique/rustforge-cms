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
use rustforge_core::{
    Page, PageParams,
    models::User,
    users::{self, NewUser, UserUpdate},
};

#[derive(Debug, Deserialize)]
pub struct UserQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
}

pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<UserQuery>,
) -> ApiResult<Json<Page<User>>> {
    if !auth.user.role.is_admin() {
        return Err(ApiError::Forbidden);
    }
    let params = PageParams::new(query.page, query.per_page);
    let page = users::list(&state.db, params, query.search.as_deref()).await?;
    Ok(Json(page))
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(input): Json<NewUser>,
) -> ApiResult<Json<User>> {
    if !auth.user.role.is_admin() {
        return Err(ApiError::Forbidden);
    }
    let user = users::create(&state.db, input).await?;
    Ok(Json(user))
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<UserUpdate>,
) -> ApiResult<Json<User>> {
    if !auth.user.role.is_admin() && auth.user.id != id {
        return Err(ApiError::Forbidden);
    }
    // Si l'utilisateur modifie son propre profil sans être admin, il ne peut pas changer son rôle
    let mut input = input;
    if !auth.user.role.is_admin() {
        input.role = None;
    }
    let user = users::update(&state.db, id, input).await?;
    Ok(Json(user))
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    if !auth.user.role.is_admin() {
        return Err(ApiError::Forbidden);
    }
    users::delete(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}
