use std::collections::BTreeMap;

use axum::{
    Json,
    extract::{Multipart, State},
};
use serde_json::Value;

use crate::{
    auth::AuthUser,
    error::{ApiError, ApiResult},
    state::AppState,
};
use rustforge_core::{
    media_processor::generate_site_icon_variants,
    settings,
};

pub async fn get_all(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> ApiResult<Json<BTreeMap<String, Value>>> {
    if !_auth.user.role.is_admin() {
        return Err(ApiError::Forbidden);
    }
    let mut all = settings::all(&state.db).await?;
    all.remove("mail.smtp_password");
    if let Some(ai) = all.get_mut("ai.settings").and_then(Value::as_object_mut) {
        ai.remove("api_key");
    }
    Ok(Json(all))
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(values): Json<BTreeMap<String, Value>>,
) -> ApiResult<Json<BTreeMap<String, Value>>> {
    if !auth.user.role.is_admin() {
        return Err(ApiError::Forbidden);
    }
    let mut updated = settings::update(&state.db, values).await?;
    updated.remove("mail.smtp_password");
    if let Some(ai) = updated.get_mut("ai.settings").and_then(Value::as_object_mut) {
        ai.remove("api_key");
    }
    state.ssr.invalidate_all().await;
    Ok(Json(updated))
}

/// Téléverse une image et génère automatiquement les variantes d'icône du site (WP-74).
pub async fn upload_site_icon(
    State(state): State<AppState>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> ApiResult<Json<Value>> {
    if !auth.user.role.is_admin() {
        return Err(ApiError::Forbidden);
    }

    let mut file_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
    {
        if field.name() == Some("file") {
            let bytes = field
                .bytes()
                .await
                .map_err(|e| ApiError::BadRequest(e.to_string()))?;
            file_bytes = Some(bytes.to_vec());
            break;
        }
    }

    let Some(bytes) = file_bytes else {
        return Err(ApiError::BadRequest("Champ 'file' manquant".into()));
    };

    if bytes.len() > 10 * 1024 * 1024 {
        return Err(ApiError::BadRequest("L'image de l'icône ne doit pas dépasser 10 Mo".into()));
    }

    let (variants, _w, _h) = generate_site_icon_variants(&bytes)?;

    let icons_dir = state.uploads_dir.join("icons");
    tokio::fs::create_dir_all(&icons_dir)
        .await
        .map_err(|e| ApiError::Internal(e.into()))?;

    let mut icon_map = serde_json::Map::new();

    for variant in variants {
        let filename = format!("site-{}.webp", variant.name.replace('_', "-"));
        let disk_path = icons_dir.join(&filename);
        tokio::fs::write(&disk_path, &variant.bytes)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;

        let public_url = format!("/uploads/icons/{filename}");
        if variant.name == "icon_512" {
            icon_map.insert("url".to_string(), serde_json::json!(public_url.clone()));
        }
        icon_map.insert(variant.name, serde_json::json!(public_url));
    }

    let icon_value = Value::Object(icon_map);
    let mut to_update = BTreeMap::new();
    to_update.insert("site.icon".to_string(), icon_value.clone());
    settings::update(&state.db, to_update).await?;

    state.ssr.invalidate_all().await;

    Ok(Json(serde_json::json!({
        "success": true,
        "icon": icon_value
    })))
}

/// Supprime l'icône du site et ses variantes physiques (WP-74).
pub async fn delete_site_icon(
    State(state): State<AppState>,
    auth: AuthUser,
) -> ApiResult<Json<Value>> {
    if !auth.user.role.is_admin() {
        return Err(ApiError::Forbidden);
    }

    let icons_dir = state.uploads_dir.join("icons");
    for name in &["site-icon-32.webp", "site-icon-180.webp", "site-icon-192.webp", "site-icon-512.webp"] {
        let p = icons_dir.join(name);
        let _ = tokio::fs::remove_file(p).await;
    }

    let mut to_update = BTreeMap::new();
    to_update.insert("site.icon".to_string(), Value::Null);
    settings::update(&state.db, to_update).await?;

    state.ssr.invalidate_all().await;

    Ok(Json(serde_json::json!({ "success": true })))
}
