use axum::{
    Json,
    extract::{Path, Query, State},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    error::{ApiError, ApiResult},
    state::AppState,
};
use rustforge_core::{
    Page,
    forms::{self, Form, FormInput, FormSubmission, SubmitFormPayload},
};

pub async fn list(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> ApiResult<Json<Vec<Form>>> {
    let forms = forms::list_forms(&state.db).await?;
    Ok(Json(forms))
}

pub async fn get(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Form>> {
    let form = forms::get_form(&state.db, id).await?;
    Ok(Json(form))
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(input): Json<FormInput>,
) -> ApiResult<Json<Form>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    let form = forms::create_form(&state.db, auth.user.id, input).await?;
    Ok(Json(form))
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<FormInput>,
) -> ApiResult<Json<Form>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    let form = forms::update_form(&state.db, id, input).await?;
    Ok(Json(form))
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    forms::delete_form(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn list_submissions(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> ApiResult<Json<Page<FormSubmission>>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    let page = params.get("page").and_then(|p| p.parse().ok());
    let per_page = params.get("per_page").and_then(|p| p.parse().ok());
    let subs = forms::list_submissions(&state.db, id, page, per_page).await?;
    Ok(Json(subs))
}

pub async fn mark_read(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(sub_id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    forms::mark_as_read(&state.db, sub_id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

// ---------------------------------------------------------------------------
// Soumission publique (sécurisée)
// ---------------------------------------------------------------------------

pub async fn public_submit(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<SubmitFormPayload>,
) -> ApiResult<Json<serde_json::Value>> {
    let sub = forms::submit(&state.db, id, payload).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Formulaire transmis avec succès",
        "submission_id": sub.id
    })))
}
