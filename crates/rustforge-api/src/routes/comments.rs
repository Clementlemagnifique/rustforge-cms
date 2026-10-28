//! Routes HTTP pour la gestion et la modération des commentaires (WP-42 à WP-45, WP-82 à WP-87).

use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};
use rustforge_core::{
    comments::{
        self, CommentCounts, CommentFilter, CreateCommentParams,
    },
    models::{Comment, CommentStatus, PublicCommentView},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    error::{ApiError, ApiResult},
    AppState,
};

/// Payload public de soumission d'un commentaire.
#[derive(Debug, Deserialize)]
pub struct PublicCommentSubmission {
    pub author_name: String,
    pub author_email: String,
    pub author_url: Option<String>,
    pub content: String,
    pub parent_id: Option<Uuid>,
    /// Champ pot de miel (honeypot) antispam : doit être laissé vide par les utilisateurs humains.
    pub website_url_hp: Option<String>,
}

/// Payload de mise à jour d'un statut unitaire.
#[derive(Debug, Deserialize)]
pub struct UpdateCommentStatusPayload {
    pub status: CommentStatus,
}

/// Payload d'action groupée (modération de masse WP-43).
#[derive(Debug, Deserialize)]
pub struct BatchCommentStatusPayload {
    pub ids: Vec<Uuid>,
    pub status: CommentStatus,
}

/// Payload de modification de contenu de commentaire.
#[derive(Debug, Deserialize)]
pub struct UpdateCommentContentPayload {
    pub content: String,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub author_url: Option<String>,
}

/// Payload de réponse d'un modérateur.
#[derive(Debug, Deserialize)]
pub struct ReplyCommentPayload {
    pub content: String,
}

/// Réponse paginée de liste de commentaires dans le Studio.
#[derive(Debug, Serialize)]
pub struct CommentListResponse {
    pub items: Vec<Comment>,
    pub total: i64,
    pub counts: CommentCounts,
}

// ---------------------------------------------------------------------------
// Routes Publiques
// ---------------------------------------------------------------------------

/// Soumet un nouveau commentaire sur une publication (`POST /api/v1/public/entries/{id}/comments`).
pub async fn public_submit(
    State(state): State<AppState>,
    Path(entry_id): Path<Uuid>,
    headers: HeaderMap,
    Json(payload): Json<PublicCommentSubmission>,
) -> ApiResult<Json<PublicCommentView>> {
    // 1. Détection pot de miel (Honeypot) antispam
    if let Some(ref hp) = payload.website_url_hp
        && !hp.trim().is_empty()
    {
        return Err(ApiError::BadRequest(
            "Soumission rejetée par le filtre antispam.".into(),
        ));
    }

    // 2. Récupération des réglages de discussion
    let all_settings = rustforge_core::settings::all(&state.db).await?;
    let comments_open = all_settings
        .get("discussion.default_comment_status")
        .map(|v| v.as_bool().unwrap_or(true) || v.as_str() == Some("open"))
        .unwrap_or(true);

    if !comments_open {
        return Err(ApiError::BadRequest(
            "Les commentaires sont actuellement fermés sur ce site.".into(),
        ));
    }

    let require_reg = all_settings
        .get("discussion.comment_registration")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if require_reg {
        return Err(ApiError::Unauthorized);
    }

    let ip = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string());

    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.trim().to_string());

    let params = CreateCommentParams {
        entry_id,
        parent_id: payload.parent_id,
        author_id: None,
        author_name: payload.author_name,
        author_email: payload.author_email,
        author_url: payload.author_url,
        author_ip: ip,
        author_user_agent: user_agent,
        content: payload.content,
    };

    let comment = comments::create_comment(&state.db, params).await?;

    // Invalider le cache de rendu public SSR
    state.ssr.invalidate_all().await;

    // Renvoyer la vue publique anonymisée (sans e-mail ni IP - WP-44)
    Ok(Json(PublicCommentView {
        id: comment.id,
        parent_id: comment.parent_id,
        author_name: comment.author_name,
        author_url: comment.author_url,
        content_html: comment.content_html,
        created_at: comment.created_at,
        children: Vec::new(),
    }))
}

/// Récupère la liste hiérarchique des commentaires approuvés pour une publication (`GET /api/v1/public/entries/{id}/comments`).
pub async fn public_list(
    State(state): State<AppState>,
    Path(entry_id): Path<Uuid>,
) -> ApiResult<Json<Vec<PublicCommentView>>> {
    let all_settings = rustforge_core::settings::all(&state.db).await?;
    let max_depth = all_settings
        .get("discussion.thread_comments_depth")
        .and_then(|v| v.as_i64())
        .unwrap_or(3) as usize;

    let tree = comments::get_entry_approved_comments_tree(&state.db, entry_id, max_depth).await?;
    Ok(Json(tree))
}

// ---------------------------------------------------------------------------
// Routes Administrateur / Studio
// ---------------------------------------------------------------------------

/// Liste les commentaires avec filtrage et pagination (`GET /api/v1/comments`).
pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(filter): Query<CommentFilter>,
) -> ApiResult<Json<CommentListResponse>> {
    if !auth.user.role.can_edit_others() {
        return Err(ApiError::Forbidden);
    }

    let (items, total) = comments::list_comments(&state.db, filter).await?;
    let counts = comments::count_by_status(&state.db).await?;

    Ok(Json(CommentListResponse {
        items,
        total,
        counts,
    }))
}

/// Compteurs de commentaires par statut (`GET /api/v1/comments/counts`).
pub async fn counts(
    State(state): State<AppState>,
    auth: AuthUser,
) -> ApiResult<Json<CommentCounts>> {
    if !auth.user.role.can_edit_others() {
        return Err(ApiError::Forbidden);
    }
    let counts = comments::count_by_status(&state.db).await?;
    Ok(Json(counts))
}

/// Récupère un commentaire unitaire (`GET /api/v1/comments/{id}`).
pub async fn get(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Comment>> {
    if !auth.user.role.can_edit_others() {
        return Err(ApiError::Forbidden);
    }
    let comment = comments::get_comment(&state.db, id).await?;
    Ok(Json(comment))
}

/// Met à jour le statut d'un commentaire (`PUT /api/v1/comments/{id}/status`).
pub async fn update_status(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCommentStatusPayload>,
) -> ApiResult<Json<Comment>> {
    if !auth.user.role.can_edit_others() {
        return Err(ApiError::Forbidden);
    }

    let comment = comments::update_comment_status(&state.db, id, payload.status).await?;
    state.ssr.invalidate_all().await;
    Ok(Json(comment))
}

/// Met à jour le statut en masse (`POST /api/v1/comments/batch`).
pub async fn batch_update(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<BatchCommentStatusPayload>,
) -> ApiResult<Json<serde_json::Value>> {
    if !auth.user.role.can_edit_others() {
        return Err(ApiError::Forbidden);
    }

    let count = comments::batch_update_status(&state.db, &payload.ids, payload.status).await?;
    state.ssr.invalidate_all().await;
    Ok(Json(serde_json::json!({ "updated": count })))
}

/// Modifie le contenu d'un commentaire (`PUT /api/v1/comments/{id}`).
pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCommentContentPayload>,
) -> ApiResult<Json<Comment>> {
    if !auth.user.role.can_edit_others() {
        return Err(ApiError::Forbidden);
    }

    let comment = comments::update_comment_content(
        &state.db,
        id,
        &payload.content,
        payload.author_name.as_deref(),
        payload.author_email.as_deref(),
        payload.author_url.as_deref(),
    )
    .await?;

    state.ssr.invalidate_all().await;
    Ok(Json(comment))
}

/// Répond à un commentaire depuis le Studio (`POST /api/v1/comments/{id}/reply`).
pub async fn reply(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<ReplyCommentPayload>,
) -> ApiResult<Json<Comment>> {
    if !auth.user.role.can_edit_others() {
        return Err(ApiError::Forbidden);
    }

    let reply = comments::reply_to_comment(
        &state.db,
        id,
        auth.user.id,
        &auth.user.display_name,
        &auth.user.email,
        &payload.content,
    )
    .await?;

    state.ssr.invalidate_all().await;
    Ok(Json(reply))
}

/// Supprime définitivement un commentaire (`DELETE /api/v1/comments/{id}`).
pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    if !auth.user.role.can_edit_others() {
        return Err(ApiError::Forbidden);
    }

    let deleted = comments::delete_comment(&state.db, id).await?;
    state.ssr.invalidate_all().await;
    Ok(Json(serde_json::json!({ "deleted": deleted })))
}

/// Vide la corbeille des commentaires (`DELETE /api/v1/comments/empty-trash`).
pub async fn empty_trash(
    State(state): State<AppState>,
    auth: AuthUser,
) -> ApiResult<Json<serde_json::Value>> {
    if !auth.user.role.can_edit_others() {
        return Err(ApiError::Forbidden);
    }

    let deleted_count = comments::empty_trash(&state.db).await?;
    state.ssr.invalidate_all().await;
    Ok(Json(serde_json::json!({ "deleted": deleted_count })))
}
