//! Routes de l'Installateur Web & Assistant de Configuration RustForge (Setup Wizard).
//! L'alter ego du fameux « Installateur en 5 minutes » de WordPress :
//! Permet d'initialiser ou de restaurer un site complet directement depuis le navigateur.

use axum::{
    Json,
    extract::{Multipart, State},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    auth::create_token,
    error::{ApiError, ApiResult},
    state::AppState,
};
use rustforge_core::{
    migration::CapsuleManager,
    models::{Role, User},
    settings,
    users::{self, NewUser},
};

#[derive(Debug, Serialize)]
pub struct SetupStatusResponse {
    pub is_installed: bool,
    pub version: String,
    pub database_connected: bool,
    pub site_title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TestDbRequest {
    pub database_url: String,
}

#[derive(Debug, Serialize)]
pub struct TestDbResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct InstallRequest {
    pub site_title: String,
    pub admin_email: String,
    pub admin_username: String,
    pub admin_password: String,
    pub site_tagline: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct InstallResponse {
    pub success: bool,
    pub message: String,
    pub token: String,
    pub user: User,
}

/// Vérifie si le CMS est déjà installé et configuré.
pub async fn status(State(state): State<AppState>) -> ApiResult<Json<SetupStatusResponse>> {
    let has_admin: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE role = 'admin')")
        .fetch_one(&state.db)
        .await
        .unwrap_or(false);

    let site_title: Option<String> = if has_admin {
        settings::get(&state.db, "site.title")
            .await
            .ok()
            .flatten()
            .and_then(|v| v.as_str().map(|s| s.to_string()))
    } else {
        None
    };

    Ok(Json(SetupStatusResponse {
        is_installed: has_admin,
        version: "0.1.0".to_string(),
        database_connected: true,
        site_title,
    }))
}

/// Teste la validité d'une chaîne de connexion PostgreSQL en direct.
pub async fn test_db(Json(payload): Json<TestDbRequest>) -> ApiResult<Json<TestDbResponse>> {
    match rustforge_core::connect(&payload.database_url).await {
        Ok(_) => Ok(Json(TestDbResponse {
            success: true,
            message: "Connexion établie avec succès avec PostgreSQL !".to_string(),
        })),
        Err(e) => Ok(Json(TestDbResponse {
            success: false,
            message: format!("Échec de connexion : {}", e),
        })),
    }
}

/// Exécute l'installation complète du CMS : migrations SQL, compte administrateur et réglages de base.
pub async fn install(
    State(state): State<AppState>,
    Json(payload): Json<InstallRequest>,
) -> ApiResult<Json<InstallResponse>> {
    // 1. Vérifie si le site n'est pas déjà installé pour éviter l'écrasement accidentel
    let has_admin: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE role = 'admin')")
        .fetch_one(&state.db)
        .await
        .unwrap_or(false);

    if has_admin {
        return Err(ApiError::BadRequest(
            "RustForge CMS est déjà installé sur cette base de données.".to_string(),
        ));
    }

    if payload.admin_password.len() < 8 {
        return Err(ApiError::BadRequest(
            "Le mot de passe de l'administrateur doit contenir au moins 8 caractères.".to_string(),
        ));
    }

    // 2. Application des migrations SQL
    rustforge_core::migrate(&state.db)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Erreur migrations SQL: {}", e)))?;

    // 3. Création du compte super-administrateur
    let user = users::create(
        &state.db,
        NewUser {
            email: payload.admin_email.trim().to_string(),
            username: payload.admin_username.trim().to_string(),
            display_name: Some(payload.admin_username.trim().to_string()),
            password: payload.admin_password,
            role: Role::Admin,
        },
    )
    .await?;

    // 4. Initialisation des réglages du site
    let mut initial_settings = std::collections::BTreeMap::new();
    initial_settings.insert("site.title".to_string(), json!(payload.site_title.trim()));
    if let Some(tagline) = payload.site_tagline {
        initial_settings.insert("site.tagline".to_string(), json!(tagline.trim()));
    }
    let _ = settings::update(&state.db, initial_settings).await;

    // 5. Génération automatique du token JWT pour connexion immédiate
    let token = create_token(user.id, user.role, &state.jwt_secret)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Erreur génération token: {}", e)))?;

    Ok(Json(InstallResponse {
        success: true,
        message: "Félicitations ! RustForge CMS a été installé avec succès.".to_string(),
        token,
        user,
    }))
}

/// Restauration directe d'une Capsule de site (.rfcapsule) depuis l'installateur web.
pub async fn restore_capsule(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> ApiResult<Json<serde_json::Value>> {
    // Sécurité : l'installateur public ne permet la restauration que sur une instance vierge
    let has_admin: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE role = 'admin')")
        .fetch_one(&state.db)
        .await
        .unwrap_or(false);

    if has_admin {
        return Err(ApiError::BadRequest(
            "RustForge CMS est déjà configuré. Connectez-vous au Studio en tant qu'administrateur pour restaurer une capsule.".to_string(),
        ));
    }

    let mut capsule_bytes: Option<Vec<u8>> = None;
    let mut new_site_url: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| ApiError::BadRequest(e.to_string()))? {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" || name == "capsule" {
            let data = field.bytes().await.map_err(|e| ApiError::BadRequest(e.to_string()))?;
            capsule_bytes = Some(data.to_vec());
        } else if name == "new_site_url" {
            let val = field.text().await.map_err(|e| ApiError::BadRequest(e.to_string()))?;
            if !val.trim().is_empty() {
                new_site_url = Some(val.trim().to_string());
            }
        }
    }

    let bytes = capsule_bytes.ok_or_else(|| ApiError::BadRequest("Aucun fichier capsule fourni".to_string()))?;

    // 1. Exécution des migrations si base vide
    let _ = rustforge_core::migrate(&state.db).await;

    // 2. Recherche ou création de l'administrateur
    let author_id: Option<uuid::Uuid> = sqlx::query_scalar("SELECT id FROM users ORDER BY created_at ASC LIMIT 1")
        .fetch_optional(&state.db)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))?;

    let admin_id = match author_id {
        Some(id) => id,
        None => {
            let user = users::create(
                &state.db,
                NewUser {
                    email: "admin@rustforge.local".into(),
                    username: "admin".into(),
                    password: "DefaultSecurePassword2026!".into(),
                    display_name: Some("Administrateur Capsule".into()),
                    role: Role::Admin,
                },
            )
            .await?;
            user.id
        }
    };

    // 3. Décompression et ingestion
    let cursor = std::io::Cursor::new(bytes);
    let report = CapsuleManager::unpack(&state.db, &state.uploads_dir, cursor, new_site_url, admin_id).await?;

    Ok(Json(json!({
        "success": true,
        "message": "Capsule restaurée avec succès !",
        "manifest": report.manifest,
        "entries_restored": report.import_report.entries_imported,
        "terms_restored": report.import_report.terms_imported,
        "media_extracted": report.media_files_extracted,
        "execution_time_ms": report.execution_time_ms,
    })))
}
