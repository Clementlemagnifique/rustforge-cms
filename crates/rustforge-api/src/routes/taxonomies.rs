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
    models::{Taxonomy, Term},
    terms::{self, TaxonomyInput, TermInput},
};

#[derive(Debug, Deserialize)]
pub struct TermListQuery {
    pub taxonomy: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteTermQuery {
    pub reassign_to: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct MergeTermPayload {
    pub target_id: Uuid,
}

pub async fn list_taxonomies(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> ApiResult<Json<Vec<Taxonomy>>> {
    let list = terms::list_taxonomies(&state.db).await?;
    Ok(Json(list))
}

pub async fn create_taxonomy(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(input): Json<TaxonomyInput>,
) -> ApiResult<Json<Taxonomy>> {
    if !auth.user.role.is_admin() {
        return Err(ApiError::Forbidden);
    }
    let tax = terms::create_taxonomy(&state.db, input).await?;
    Ok(Json(tax))
}

pub async fn delete_taxonomy(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    if !auth.user.role.is_admin() {
        return Err(ApiError::Forbidden);
    }
    terms::delete_taxonomy(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn list_terms(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<TermListQuery>,
) -> ApiResult<Json<Vec<Term>>> {
    let list = terms::list_terms(&state.db, query.taxonomy.as_deref()).await?;
    Ok(Json(list))
}

pub async fn create_term(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(input): Json<TermInput>,
) -> ApiResult<Json<Term>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    let term = terms::create_term(&state.db, input).await?;
    Ok(Json(term))
}

pub async fn update_term(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<TermInput>,
) -> ApiResult<Json<Term>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    let term = terms::update_term(&state.db, id, input).await?;
    Ok(Json(term))
}

pub async fn delete_term(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Query(query): Query<DeleteTermQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    terms::delete_term_with_reassignment(&state.db, id, query.reassign_to).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn merge_term(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<MergeTermPayload>,
) -> ApiResult<Json<Term>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    let merged = terms::merge_terms(&state.db, id, payload.target_id).await?;
    Ok(Json(merged))
}
