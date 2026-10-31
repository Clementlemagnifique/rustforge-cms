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
use rustforge_core::forms::{self, Form, FormInput, FormSubmission, SubmitFormPayload};

async fn manage(state: &AppState, auth: &AuthUser, id: Uuid) -> ApiResult<()> {
    let form = forms::get_form(&state.db, id).await?;
    if !auth.user.role.can_write()
        || (!auth.user.role.can_edit_others() && form.created_by != Some(auth.user.id))
    {
        return Err(ApiError::Forbidden);
    }
    Ok(())
}

pub async fn list(State(state): State<AppState>, _auth: AuthUser) -> ApiResult<Json<Vec<Form>>> {
    rustforge_core::security::require_writer(_auth.user.role)?;
    let mut forms = forms::list_forms(&state.db).await?;
    if !_auth.user.role.can_edit_others() {
        forms.retain(|form| form.created_by == Some(_auth.user.id));
    }
    Ok(Json(forms))
}

pub async fn get(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Form>> {
    manage(&state, &_auth, id).await?;
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
    manage(&state, &auth, id).await?;
    let form = forms::update_form(&state.db, id, input).await?;
    Ok(Json(form))
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    manage(&state, &auth, id).await?;
    forms::delete_form(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

#[derive(serde::Serialize)]
pub struct InboxResponse {
    #[serde(flatten)]
    page: rustforge_core::Page<FormSubmission>,
    counts: forms::SubmissionCounts,
}
pub async fn list_submissions(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Query(filter): Query<forms::SubmissionFilter>,
) -> ApiResult<Json<InboxResponse>> {
    manage(&state, &auth, id).await?;
    let inbox = forms::submission_inbox(&state.db, id, &filter).await?;
    Ok(Json(InboxResponse {
        page: inbox.page,
        counts: inbox.counts,
    }))
}

pub async fn change_submission(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(sub_id): Path<Uuid>,
    Json(change): Json<forms::SubmissionChange>,
) -> ApiResult<Json<FormSubmission>> {
    let form_id: Option<Uuid> =
        sqlx::query_scalar("SELECT form_id FROM form_submissions WHERE id=$1")
            .bind(sub_id)
            .fetch_optional(&state.db)
            .await
            .map_err(rustforge_core::CoreError::from)?;
    let form_id = form_id.ok_or(ApiError::NotFound)?;
    manage(&state, &auth, form_id).await?;
    Ok(Json(
        forms::change_submission(&state.db, form_id, sub_id, &change).await?,
    ))
}

pub async fn mark_read(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(sub_id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    let form_id: Option<Uuid> =
        sqlx::query_scalar("SELECT form_id FROM form_submissions WHERE id = $1")
            .bind(sub_id)
            .fetch_optional(&state.db)
            .await
            .map_err(rustforge_core::CoreError::from)?;
    manage(&state, &auth, form_id.ok_or(ApiError::NotFound)?).await?;
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

pub async fn batch_submissions(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(batch): Json<forms::SubmissionBatch>,
) -> ApiResult<Json<serde_json::Value>> {
    manage(&state, &auth, id).await?;
    let affected = forms::batch_submissions(&state.db, id, &batch).await?;
    Ok(Json(serde_json::json!({"affected":affected})))
}
