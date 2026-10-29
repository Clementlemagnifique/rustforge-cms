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

        // Exécution du pipeline de traitement Rust (WebP + Variantes responsives + Sanitization SVG)
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

        // Écriture des variantes responsives (WP-39, WP-88)
        let mut variants_json = serde_json::Map::new();
        for variant in &processed.variants {
            let var_filename = format!("{base_uuid}-{}.{}", variant.name, variant.ext);
            let var_rel = format!("{sub_dir}/{var_filename}");
            let var_abs = state.uploads_dir.join(&var_rel);
            tokio::fs::write(&var_abs, &variant.bytes)
                .await
                .map_err(|e| ApiError::Internal(e.into()))?;

            variants_json.insert(
                variant.name.clone(),
                serde_json::json!({
                    "path": var_rel,
                    "url": format!("/uploads/{var_rel}"),
                    "width": variant.width,
                    "height": variant.height,
                    "size_bytes": variant.bytes.len(),
                    "mime_type": variant.mime,
                }),
            );
        }

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
                    "is_webp_optimized": processed.is_webp_converted,
                    "variants": variants_json,
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
    let (rel_path, webp_path, extra_paths) =
        media::delete_checked(&state.db, id, query.force).await?;
    let _ = tokio::fs::remove_file(state.uploads_dir.join(rel_path)).await;
    if let Some(wp) = webp_path {
        let _ = tokio::fs::remove_file(state.uploads_dir.join(wp)).await;
    }
    for extra in extra_paths {
        let _ = tokio::fs::remove_file(state.uploads_dir.join(extra)).await;
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

#[derive(Debug, Deserialize)]
pub struct RotateRequest {
    pub degrees: i32,
}

/// Applique une rotation à une image et régénère les variantes avec archivage de l'original (WP-39).
pub async fn rotate(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<RotateRequest>,
) -> ApiResult<Json<Media>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    let current = media::get(&state.db, id).await?;
    if !matches!(
        current.mime_type.as_str(),
        "image/jpeg" | "image/png" | "image/webp"
    ) {
        return Err(ApiError::BadRequest(
            "Seules les images matricielles (JPEG, PNG, WebP) peuvent être pivotées.".into(),
        ));
    }

    let main_abs = state.uploads_dir.join(&current.path);
    let bytes = tokio::fs::read(&main_abs)
        .await
        .map_err(|e| ApiError::Internal(e.into()))?;

    // Sauvegarde de l'original souverain si pas encore archivé (WP-39)
    let mut metadata = current.metadata.clone();
    if metadata.get("original_path").is_none() {
        let ext = current
            .path
            .rsplit('.')
            .next()
            .unwrap_or("bin");
        let orig_filename = format!("{}-original.{}", current.id, ext);
        let sub_dir = current
            .path
            .rsplit_once('/')
            .map(|(d, _)| d)
            .unwrap_or("");
        let orig_rel = if sub_dir.is_empty() {
            orig_filename
        } else {
            format!("{sub_dir}/{orig_filename}")
        };
        let orig_abs = state.uploads_dir.join(&orig_rel);
        tokio::fs::write(&orig_abs, &bytes)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;
        metadata["original_path"] = serde_json::Value::String(orig_rel);
    }

    // Applique la rotation
    let (rotated_bytes, w, h) = media_processor::rotate_image_bytes(&bytes, req.degrees, 85.0)?;
    tokio::fs::write(&main_abs, &rotated_bytes)
        .await
        .map_err(|e| ApiError::Internal(e.into()))?;

    // Régénère les variantes depuis l'image pivotée
    let (new_variants, _, _) = media_processor::generate_variants_from_bytes(
        &rotated_bytes,
        &media_processor::default_variant_specs(),
        80.0,
    )?;

    let sub_dir = current
        .path
        .rsplit_once('/')
        .map(|(d, _)| d)
        .unwrap_or("");
    let base_id = current.id.to_string();
    let mut variants_json = serde_json::Map::new();
    for var in &new_variants {
        let var_filename = format!("{base_id}-{}.{}", var.name, var.ext);
        let var_rel = if sub_dir.is_empty() {
            var_filename
        } else {
            format!("{sub_dir}/{var_filename}")
        };
        let var_abs = state.uploads_dir.join(&var_rel);
        tokio::fs::write(&var_abs, &var.bytes)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;
        variants_json.insert(
            var.name.clone(),
            serde_json::json!({
                "path": var_rel,
                "url": format!("/uploads/{var_rel}"),
                "width": var.width,
                "height": var.height,
                "size_bytes": var.bytes.len(),
                "mime_type": var.mime,
            }),
        );
    }
    metadata["variants"] = serde_json::Value::Object(variants_json);

    let updated = media::update_transformed(
        &state.db,
        id,
        w,
        h,
        rotated_bytes.len() as i64,
        metadata,
    )
    .await?;

    Ok(Json(updated))
}

/// Restaure un média transformé à son état original souverain (WP-39).
pub async fn restore(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Media>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    let current = media::get(&state.db, id).await?;
    let Some(orig_rel) = current
        .metadata
        .get("original_path")
        .and_then(|v| v.as_str())
    else {
        return Err(ApiError::BadRequest(
            "Ce média n'a pas été transformé ou son original est introuvable.".into(),
        ));
    };

    let orig_abs = state.uploads_dir.join(orig_rel);
    let orig_bytes = tokio::fs::read(&orig_abs)
        .await
        .map_err(|e| ApiError::Internal(e.into()))?;

    let main_abs = state.uploads_dir.join(&current.path);
    tokio::fs::write(&main_abs, &orig_bytes)
        .await
        .map_err(|e| ApiError::Internal(e.into()))?;

    // Supprime le fichier d'original archivé
    let _ = tokio::fs::remove_file(orig_abs).await;

    // Régénère les variantes depuis l'original restauré
    let (new_variants, orig_w, orig_h) = media_processor::generate_variants_from_bytes(
        &orig_bytes,
        &media_processor::default_variant_specs(),
        80.0,
    )?;

    let sub_dir = current
        .path
        .rsplit_once('/')
        .map(|(d, _)| d)
        .unwrap_or("");
    let base_id = current.id.to_string();
    let mut variants_json = serde_json::Map::new();
    for var in &new_variants {
        let var_filename = format!("{base_id}-{}.{}", var.name, var.ext);
        let var_rel = if sub_dir.is_empty() {
            var_filename
        } else {
            format!("{sub_dir}/{var_filename}")
        };
        let var_abs = state.uploads_dir.join(&var_rel);
        tokio::fs::write(&var_abs, &var.bytes)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;
        variants_json.insert(
            var.name.clone(),
            serde_json::json!({
                "path": var_rel,
                "url": format!("/uploads/{var_rel}"),
                "width": var.width,
                "height": var.height,
                "size_bytes": var.bytes.len(),
                "mime_type": var.mime,
            }),
        );
    }

    let mut metadata = current.metadata.clone();
    if let Some(obj) = metadata.as_object_mut() {
        obj.remove("original_path");
        obj.insert(
            "variants".to_string(),
            serde_json::Value::Object(variants_json),
        );
    }

    let updated = media::update_transformed(
        &state.db,
        id,
        orig_w,
        orig_h,
        orig_bytes.len() as i64,
        metadata,
    )
    .await?;

    Ok(Json(updated))
}
