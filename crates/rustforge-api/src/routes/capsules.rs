//! Routes de Gestion des Capsules de Sauvegarde & Migration (.rfcapsule).
//! Alter ego d'All-in-One WP Migration :
//! Permet d'exporter l'intégralité du site en 1 clic et de restaurer ou cloner sur n'importe quel domaine.

use axum::{
    Json,
    body::Body,
    extract::{Multipart, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use chrono::Utc;
use serde_json::json;
use std::io::Cursor;

use crate::{
    auth::AuthUser,
    error::{ApiError, ApiResult},
    state::AppState,
};
use rustforge_core::{migration::CapsuleManager, models::Role, settings};

/// Exporte l'intégralité du site (Base de données + Médias) au format souverain .rfcapsule
pub async fn export_capsule(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Response, ApiError> {
    if user.user.role != Role::Admin {
        return Err(ApiError::Forbidden);
    }

    // Récupération de l'URL du site et de son titre pour nommer l'archive
    let site_url: Option<String> = settings::get(&state.db, "site.url")
        .await
        .ok()
        .flatten()
        .and_then(|v| v.as_str().map(|s| s.to_string()));

    let mut buffer = Cursor::new(Vec::new());
    let _ =
        CapsuleManager::pack_to_writer(&state.db, &state.storage_paths(), &mut buffer, site_url)
            .await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Erreur génération capsule: {}", e)))?;

    let bytes = buffer.into_inner();
    let filename = format!(
        "rustforge-capsule-{}.rfcapsule",
        Utc::now().format("%Y%m%d-%H%M%S")
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename)).unwrap_or_else(
            |_| HeaderValue::from_static("attachment; filename=\"site.rfcapsule\""),
        ),
    );
    headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&bytes.len().to_string())
            .unwrap_or_else(|_| HeaderValue::from_static("0")),
    );

    Ok((StatusCode::OK, headers, Body::from(bytes)).into_response())
}

/// Inspecte le contenu d'un fichier .rfcapsule avant restauration
pub async fn inspect_capsule(
    user: AuthUser,
    mut multipart: Multipart,
) -> ApiResult<Json<serde_json::Value>> {
    if user.user.role != Role::Admin {
        return Err(ApiError::Forbidden);
    }

    let mut capsule_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" || name == "capsule" {
            let data = field
                .bytes()
                .await
                .map_err(|e| ApiError::BadRequest(e.to_string()))?;
            capsule_bytes = Some(data.to_vec());
        }
    }

    let bytes = capsule_bytes
        .ok_or_else(|| ApiError::BadRequest("Aucun fichier capsule fourni".to_string()))?;
    let manifest = CapsuleManager::inspect(Cursor::new(bytes))
        .map_err(|e| ApiError::BadRequest(format!("Archive capsule invalide: {}", e)))?;

    Ok(Json(json!({
        "valid": true,
        "manifest": manifest,
    })))
}

/// Restaure ou clone un site complet depuis une archive .rfcapsule avec réécriture d'URL
pub async fn restore_capsule(
    State(state): State<AppState>,
    user: AuthUser,
    mut multipart: Multipart,
) -> ApiResult<Json<serde_json::Value>> {
    if user.user.role != Role::Admin {
        return Err(ApiError::Forbidden);
    }

    let mut capsule_bytes: Option<Vec<u8>> = None;
    let mut new_site_url: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" || name == "capsule" {
            let data = field
                .bytes()
                .await
                .map_err(|e| ApiError::BadRequest(e.to_string()))?;
            capsule_bytes = Some(data.to_vec());
        } else if name == "new_site_url" {
            let val = field
                .text()
                .await
                .map_err(|e| ApiError::BadRequest(e.to_string()))?;
            if !val.trim().is_empty() {
                new_site_url = Some(val.trim().to_string());
            }
        }
    }

    let bytes = capsule_bytes
        .ok_or_else(|| ApiError::BadRequest("Aucun fichier capsule fourni".to_string()))?;
    let cursor = Cursor::new(bytes);
    let report = CapsuleManager::unpack(
        &state.db,
        &state.storage_paths(),
        cursor,
        new_site_url,
        user.user.id,
    )
    .await?;

    Ok(Json(json!({
        "success": true,
        "message": "Site et médias restaurés avec succès !",
        "manifest": report.manifest,
        "entries_restored": report.import_report.entries_imported,
        "terms_restored": report.import_report.terms_imported,
        "media_extracted": report.media_files_extracted,
        "vault_fields_stored": report.import_report.legacy_fields_stored_in_vault,
        "execution_time_ms": report.execution_time_ms,
    })))
}
