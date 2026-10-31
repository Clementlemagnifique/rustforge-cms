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

async fn manage(state: &AppState, auth: &AuthUser, id: Uuid) -> ApiResult<Gallery> {
    let gallery = galleries::get_gallery(&state.db, id).await?;
    if !auth.user.role.can_write()
        || (!auth.user.role.can_edit_others() && gallery.created_by != Some(auth.user.id))
    {
        return Err(ApiError::Forbidden);
    }
    Ok(gallery)
}

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

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<GalleryInput>,
) -> ApiResult<Json<Gallery>> {
    manage(&state, &auth, id).await?;
    Ok(Json(galleries::update_gallery(&state.db, id, input).await?))
}

pub async fn add_item(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<GalleryItemInput>,
) -> ApiResult<Json<GalleryItem>> {
    manage(&state, &auth, id).await?;
    let item = galleries::add_item(&state.db, id, input).await?;
    Ok(Json(item))
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    manage(&state, &auth, id).await?;
    galleries::delete_gallery(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn delete_item(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((gallery_id, item_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<serde_json::Value>> {
    let gallery = manage(&state, &auth, gallery_id).await?;
    if !gallery.items.iter().any(|item| item.id == item_id) {
        return Err(ApiError::NotFound);
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
