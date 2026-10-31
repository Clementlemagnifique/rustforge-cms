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
use rustforge_core::{media_processor::generate_site_icon_variants, settings};

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
    if let Some(ai) = updated
        .get_mut("ai.settings")
        .and_then(Value::as_object_mut)
    {
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
        return Err(ApiError::BadRequest(
            "L'image de l'icône ne doit pas dépasser 10 Mo".into(),
        ));
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
    for name in &[
        "site-icon-32.webp",
        "site-icon-180.webp",
        "site-icon-192.webp",
        "site-icon-512.webp",
    ] {
        let p = icons_dir.join(name);
        let _ = tokio::fs::remove_file(p).await;
    }

    let mut to_update = BTreeMap::new();
    to_update.insert("site.icon".to_string(), Value::Null);
    settings::update(&state.db, to_update).await?;

    state.ssr.invalidate_all().await;

    Ok(Json(serde_json::json!({ "success": true })))
}

// ---------------------------------------------------------------------------
// Gestion des thèmes souverains (TH-01 à 06)
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Serialize)]
pub struct ThemeSummary {
    pub slug: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub is_active: bool,
    pub tokens: serde_json::Value,
}

/// Liste les thèmes disponibles dans le dossier themes/ du CMS.
pub async fn list_themes(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> ApiResult<Json<Vec<ThemeSummary>>> {
    let active_theme = rustforge_core::settings::get(&state.db, "site.active_theme")
        .await?
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_else(|| state.active_theme.clone());

    let mut themes = Vec::new();
    if let Ok(mut entries) = tokio::fs::read_dir(&state.themes_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_dir() {
                let slug = entry.file_name().to_string_lossy().to_string();
                let theme_json_path = path.join("theme.json");
                let mut name = slug.clone();
                let mut description = String::new();
                let mut version = "1.0.0".to_string();
                let mut author = "RustForge".to_string();
                let mut tokens = serde_json::json!({});

                if let Some(parsed) = tokio::fs::read_to_string(&theme_json_path)
                    .await
                    .ok()
                    .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
                {
                    if let Some(n) = parsed.get("name").and_then(|v| v.as_str()) {
                        name = n.to_string();
                    }
                    if let Some(d) = parsed.get("description").and_then(|v| v.as_str()) {
                        description = d.to_string();
                    }
                    if let Some(v) = parsed.get("version").and_then(|v| v.as_str()) {
                        version = v.to_string();
                    }
                    if let Some(a) = parsed.get("author").and_then(|v| v.as_str()) {
                        author = a.to_string();
                    }
                    if let Some(tok) = parsed.get("tokens") {
                        tokens = tok.clone();
                    }
                }

                let is_active = slug == active_theme;
                themes.push(ThemeSummary {
                    slug,
                    name,
                    description,
                    version,
                    author,
                    is_active,
                    tokens,
                });
            }
        }
    }

    // Sort active theme first, then by name
    themes.sort_by(|a, b| {
        if a.is_active != b.is_active {
            b.is_active.cmp(&a.is_active)
        } else {
            a.name.cmp(&b.name)
        }
    });

    Ok(Json(themes))
}

/// Active un thème par son slug (Administrateur uniquement).
pub async fn activate_theme(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path(slug): axum::extract::Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    if !auth.user.role.is_admin() {
        return Err(ApiError::Forbidden);
    }

    rustforge_core::security::safe_slug(&slug)?;
    let theme_dir = rustforge_core::security::confined_path(&state.themes_dir, &slug)?;
    if !theme_dir.exists() {
        return Err(ApiError::NotFound);
    }

    let mut map = BTreeMap::new();
    map.insert("site.active_theme".to_string(), serde_json::json!(slug));

    let theme_json_path = theme_dir.join("theme.json");
    if let Some(tokens) = tokio::fs::read_to_string(&theme_json_path)
        .await
        .ok()
        .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
        .and_then(|parsed| parsed.get("tokens").cloned())
    {
        map.insert("theme.tokens".to_string(), tokens.clone());
        map.insert("site.design_tokens".to_string(), tokens);
    }

    settings::update(&state.db, map).await?;
    state.ssr.invalidate_all().await;

    Ok(Json(serde_json::json!({
        "success": true,
        "active_theme": slug
    })))
}
