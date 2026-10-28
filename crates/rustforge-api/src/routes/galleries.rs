use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    error::{ApiError, ApiResult},
    state::AppState,
};
use rustforge_core::galleries::{self, Gallery, GalleryInput, GalleryItem, GalleryItemInput};

pub async fn list(State(state): State<AppState>, _auth: AuthUser) -> ApiResult<Json<Vec<Gallery>>> {
    let galleries = galleries::list_galleries(&state.db).await?;
    Ok(Json(galleries))
}

pub async fn get(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Gallery>> {
    let g = galleries::get_gallery(&state.db, id).await?;
    Ok(Json(g))
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(input): Json<GalleryInput>,
) -> ApiResult<Json<Gallery>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    let g = galleries::create_gallery(&state.db, auth.user.id, input).await?;
    Ok(Json(g))
}

pub async fn add_item(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<GalleryItemInput>,
) -> ApiResult<Json<GalleryItem>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    let item = galleries::add_item(&state.db, id, input).await?;
    Ok(Json(item))
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    galleries::delete_gallery(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn delete_item(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((_gallery_id, item_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<serde_json::Value>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    galleries::delete_item(&state.db, item_id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

// ---------------------------------------------------------------------------
// Route publique de récupération par slug
// ---------------------------------------------------------------------------

pub async fn public_get_by_slug(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> ApiResult<Json<Gallery>> {
    let g = galleries::get_gallery_by_slug(&state.db, &slug).await?;
    Ok(Json(g))
}
