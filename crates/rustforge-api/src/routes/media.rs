use axum::{
    Json,
    extract::{Multipart, Path, Query, State},
};
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    error::{ApiError, ApiResult},
    state::AppState,
};
use rustforge_core::{
    Page,
    media::{self, MediaFolder, MediaQuery, MediaUpdate, NewMedia},
    media_processor,
    models::Media,
};

#[derive(Debug, Deserialize)]
pub struct CreateFolderRequest {
    pub name: String,
    pub parent_id: Option<Uuid>,
}

pub async fn list(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<MediaQuery>,
) -> ApiResult<Json<Page<Media>>> {
    let page = media::list(&state.db, &query).await?;
    Ok(Json(page))
}

pub async fn list_folders(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> ApiResult<Json<Vec<MediaFolder>>> {
    let folders = media::list_folders(&state.db).await?;
    Ok(Json(folders))
}

pub async fn create_folder(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateFolderRequest>,
) -> ApiResult<Json<MediaFolder>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    let folder = media::create_folder(&state.db, &req.name, req.parent_id).await?;
    Ok(Json(folder))
}

pub async fn delete_folder(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    media::delete_folder(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn upload(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<std::collections::HashMap<String, String>>,
    mut multipart: Multipart,
) -> ApiResult<Json<Media>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }

    let folder_id = params
        .get("folder_id")
        .and_then(|id| Uuid::parse_str(id).ok());

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
    {
        let name = field.name().unwrap_or_default().to_string();
        if name != "file" {
            continue;
        }

        let original_name = field.file_name().unwrap_or("fichier").to_string();

        let raw_mime = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        let raw_bytes = field
            .bytes()
            .await
            .map_err(|e| ApiError::BadRequest(e.to_string()))?;

        // Exécution du pipeline de traitement Rust (WebP + Sanitization SVG)
        let processed =
            media_processor::process_upload(&raw_bytes, &original_name, &raw_mime).await?;

        let now = Utc::now();
        let sub_dir = format!("{}/{:02}", now.format("%Y"), now.format("%m"));
        let target_dir = state.uploads_dir.join(&sub_dir);
        tokio::fs::create_dir_all(&target_dir)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;

        let base_uuid = Uuid::new_v4();
        let main_filename = format!("{base_uuid}.{}", processed.final_ext);
        let main_rel_path = format!("{sub_dir}/{main_filename}");
        let main_abs_path = state.uploads_dir.join(&main_rel_path);

        tokio::fs::write(&main_abs_path, &processed.final_bytes)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;

        // Si une conversion WebP optimisée a été produite en parallèle
        let webp_rel_path = if let Some(ref webp_data) = processed.webp_bytes {
            let webp_filename = format!("{base_uuid}.webp");
            let webp_rel = format!("{sub_dir}/{webp_filename}");
            let webp_abs = state.uploads_dir.join(&webp_rel);
            tokio::fs::write(&webp_abs, webp_data)
                .await
                .map_err(|e| ApiError::Internal(e.into()))?;
            Some(webp_rel)
        } else if processed.final_mime == "image/webp" {
            Some(main_rel_path.clone())
        } else {
            None
        };

        let record = media::create(
            &state.db,
            NewMedia {
                path: main_rel_path,
                original_name,
                mime_type: processed.final_mime,
                size_bytes: processed.final_bytes.len() as i64,
                width: processed.width,
                height: processed.height,
                uploaded_by: auth.user.id,
                folder_id,
                webp_path: webp_rel_path,
                external_provider: Some("local".into()),
                metadata: serde_json::json!({
                    "is_webp_optimized": processed.is_webp_converted
                }),
            },
        )
        .await?;

        return Ok(Json(record));
    }

    Err(ApiError::BadRequest("aucun fichier envoyé".to_string()))
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<MediaUpdate>,
) -> ApiResult<Json<Media>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    let m = media::update(&state.db, id, input).await?;
    Ok(Json(m))
}

#[derive(Debug, Default, Deserialize)]
pub struct DeleteMediaQuery {
    #[serde(default)]
    pub force: bool,
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Query(query): Query<DeleteMediaQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    let (rel_path, webp_path) = media::delete_checked(&state.db, id, query.force).await?;
    let _ = tokio::fs::remove_file(state.uploads_dir.join(rel_path)).await;
    if let Some(wp) = webp_path {
        let _ = tokio::fs::remove_file(state.uploads_dir.join(wp)).await;
    }

    Ok(Json(serde_json::json!({ "success": true })))
}
