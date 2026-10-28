use axum::{
    Json,
    extract::{Multipart, State},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tokio::fs;

use crate::{
    auth::AuthUser,
    error::{ApiError, ApiResult},
    state::AppState,
};
use rustforge_core::{
    converter::{
        ConversionManifest, FormsConversionResult, FormsConverter, FseThemeConverter,
        PackageAnalyzer,
    },
    settings,
};

#[derive(Debug, Deserialize)]
pub struct FormsPlanRequest {
    pub slug: String,
    pub author_or_title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ApplyPlanRequest {
    pub enable_modules: Vec<String>,
    pub activate_theme_slug: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ConvertThemeResponse {
    pub success: bool,
    pub theme_slug: String,
    pub theme_name: String,
    pub templates_count: usize,
    pub target_directory: String,
    pub preview_url: String,
}

#[derive(Debug, Serialize)]
pub struct ApplyPlanResponse {
    pub success: bool,
    pub activated_modules: Vec<String>,
    pub active_theme: String,
    pub message: String,
}

/// Endpoint d'analyse d'une archive ZIP WordPress (glisser-déposer).
pub async fn analyze(
    State(_state): State<AppState>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> ApiResult<Json<ConversionManifest>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }

    let mut zip_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
    {
        let name = field.name().unwrap_or_default().to_string();
        if name == "file" || name == "archive" {
            let bytes = field
                .bytes()
                .await
                .map_err(|e| ApiError::BadRequest(e.to_string()))?;
            zip_bytes = Some(bytes.to_vec());
            break;
        }
    }

    let bytes = zip_bytes
        .ok_or_else(|| ApiError::BadRequest("Aucun fichier 'file' dans l'upload".to_string()))?;

    let manifest =
        PackageAnalyzer::analyze_zip(&bytes).map_err(|e| ApiError::BadRequest(e.to_string()))?;

    Ok(Json(manifest))
}

/// Endpoint de conversion déclarative d'un thème FSE vers MiniJinja 3.
pub async fn convert_theme(
    State(state): State<AppState>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> ApiResult<Json<ConvertThemeResponse>> {
    if !auth.user.role.is_admin() {
        return Err(ApiError::Forbidden);
    }

    let mut zip_bytes: Option<Vec<u8>> = None;
    let mut custom_slug: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
    {
        let name = field.name().unwrap_or_default().to_string();
        if name == "file" || name == "archive" {
            let bytes = field
                .bytes()
                .await
                .map_err(|e| ApiError::BadRequest(e.to_string()))?;
            zip_bytes = Some(bytes.to_vec());
        } else if name == "slug" {
            let text = field.text().await.unwrap_or_default();
            if !text.trim().is_empty() {
                custom_slug = Some(text.trim().to_string());
            }
        }
    }

    let bytes = zip_bytes.ok_or_else(|| {
        ApiError::BadRequest("Archive ZIP requise pour la conversion".to_string())
    })?;

    let converted = FseThemeConverter::convert_zip(&bytes, custom_slug.as_deref())
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    // Écriture sécurisée dans themes/<slug>/
    let theme_dir = state.themes_dir.join(&converted.slug);
    rustforge_core::security::safe_slug(&converted.slug)?;
    rustforge_core::security::confined_path(&state.themes_dir, &converted.slug)?;
    if theme_dir.exists() {
        return Err(ApiError::BadRequest(
            "Ce thème existe déjà. Choisissez un nouveau nom pour préparer la conversion.".into(),
        ));
    }
    let templates_dir = theme_dir.join("templates");

    fs::create_dir_all(&templates_dir)
        .await
        .map_err(|e| ApiError::Internal(e.into()))?;

    // Écriture de theme.json
    let theme_json_path = theme_dir.join("theme.json");
    fs::write(&theme_json_path, converted.theme_json_content.as_bytes())
        .await
        .map_err(|e| ApiError::Internal(e.into()))?;

    // Écriture de chaque template MiniJinja (.html)
    for (tpl_name, tpl_content) in &converted.templates {
        rustforge_core::security::relative_path(tpl_name)?;
        let tpl_path = templates_dir.join(tpl_name);
        fs::write(&tpl_path, tpl_content.as_bytes())
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;
    }

    let preview_url = format!("/api/v1/themes/{}/preview", converted.slug);

    Ok(Json(ConvertThemeResponse {
        success: true,
        theme_slug: converted.slug.clone(),
        theme_name: converted.name,
        templates_count: converted.templates.len(),
        target_directory: theme_dir.to_string_lossy().to_string(),
        preview_url,
    }))
}

/// Endpoint de consultation du plan de substitution de formulaires (Cas B).
pub async fn forms_plan(
    State(_state): State<AppState>,
    _auth: AuthUser,
    Json(req): Json<FormsPlanRequest>,
) -> ApiResult<Json<FormsConversionResult>> {
    let result = FormsConverter::plan_conversion(
        &req.slug,
        req.author_or_title
            .as_deref()
            .unwrap_or("Extension Formulaire"),
    );
    Ok(Json(result))
}

/// Endpoint d'application du plan d'activation des modules et du thème.
pub async fn apply_plan(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<ApplyPlanRequest>,
) -> ApiResult<Json<ApplyPlanResponse>> {
    if !auth.user.role.is_admin() {
        return Err(ApiError::Forbidden);
    }

    // 1. Récupération des réglages actuels de modules
    let mut current_modules = settings::get_active_modules(&state.db).await?;

    // 2. Activation des modules demandés
    for module_name in &req.enable_modules {
        current_modules.insert(module_name.clone(), true);
    }

    let mut update_map = BTreeMap::new();
    update_map.insert(
        "modules.active".to_string(),
        serde_json::to_value(&current_modules).unwrap_or_default(),
    );

    // 3. Changement de thème si demandé
    let mut final_theme = state.active_theme.clone();
    if let Some(ref slug) = req.activate_theme_slug {
        rustforge_core::security::safe_slug(slug)?;
        let theme = rustforge_core::security::confined_path(&state.themes_dir, slug)?;
        if !theme.join("templates/single.html").is_file() {
            return Err(ApiError::BadRequest("Thème incomplet".into()));
        }
        update_map.insert(
            "site.active_theme".to_string(),
            serde_json::Value::String(slug.clone()),
        );
        final_theme = slug.clone();
    }

    settings::update(&state.db, update_map).await?;

    let activated_list: Vec<String> = current_modules
        .into_iter()
        .filter(|(_, active)| *active)
        .map(|(name, _)| name)
        .collect();

    Ok(Json(ApplyPlanResponse {
        success: true,
        activated_modules: activated_list,
        active_theme: final_theme,
        message: "Plan de conversion appliqué avec succès. Modules activés dans RustForge."
            .to_string(),
    }))
}
