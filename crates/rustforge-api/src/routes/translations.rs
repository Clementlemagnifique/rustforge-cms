//! Routes d'API pour le multilinguisme souverain, les catalogues i18n et les grappes de traduction.
//!
//! Exigences :
//! - MOD-I18N-01 : Contrat de localisation, gestion des langues actives et par défaut.
//! - MOD-I18N-02 & MOD-I18N-03 : Éditeur de catalogues, recherche, contextes et synchronisation native.
//! - MOD-I18N-04 : Import / export GNU Gettext PO & JSON.
//! - MOD-I18N-05 : Overrides, persistance des dictionnaires et sauvegarde.
//! - MOD-I18N-07 : Grappes de traduction (`translation_group_id`), liaison / déliaison de contenus et matrices.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::header,
    response::Response,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    error::{ApiError, ApiResult},
    state::AppState,
};
use rustforge_core::{
    entries,
    i18n::{self, CatalogItem, SupportedLanguage, TranslationCatalog},
    models::{Entry, EntryStatus, TranslationSibling},
};

// ---------------------------------------------------------------------------
// Configuration des Langues (MOD-I18N-01)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct LanguagesResponse {
    pub available: Vec<SupportedLanguage>,
    pub active: Vec<SupportedLanguage>,
    pub default: String,
}

#[derive(Debug, Deserialize)]
pub struct SetLanguagesPayload {
    pub active: Vec<String>,
    pub default: String,
}

/// Retourne la liste des langues disponibles, des langues actives et la langue par défaut.
pub async fn get_languages(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> ApiResult<Json<LanguagesResponse>> {
    let available = i18n::get_available_languages();
    let active = i18n::get_active_languages(&state.db).await?;
    let default = i18n::get_default_language(&state.db).await?;

    Ok(Json(LanguagesResponse {
        available,
        active,
        default,
    }))
}

/// Met à jour les langues actives et la langue par défaut (Administrateur uniquement).
pub async fn update_languages(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<SetLanguagesPayload>,
) -> ApiResult<Json<LanguagesResponse>> {
    if !auth.user.role.is_admin() {
        return Err(ApiError::Forbidden);
    }

    i18n::set_languages_config(&state.db, &payload.active, &payload.default).await?;

    let available = i18n::get_available_languages();
    let active = i18n::get_active_languages(&state.db).await?;
    let default = i18n::get_default_language(&state.db).await?;

    Ok(Json(LanguagesResponse {
        available,
        active,
        default,
    }))
}

// ---------------------------------------------------------------------------
// Grappes de Traduction des Contenus (MOD-I18N-07)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct MatrixQuery {
    pub content_type: Option<String>,
    pub q: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct MatrixEntryRow {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub content_type: String,
    pub locale: String,
    pub status: EntryStatus,
    pub translation_group_id: Option<Uuid>,
    pub translations: Vec<TranslationSibling>,
}

#[derive(Debug, Serialize)]
pub struct TranslationMatrixResponse {
    pub active_languages: Vec<SupportedLanguage>,
    pub items: Vec<MatrixEntryRow>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

/// Matrice de traduction : liste les entrées et l'état de leurs traductions
/// dans chacune des langues actives du CMS.
pub async fn get_translation_matrix(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(q): Query<MatrixQuery>,
) -> ApiResult<Json<TranslationMatrixResponse>> {
    let active_languages = i18n::get_active_languages(&state.db).await?;

    let mut list_query = entries::EntryQuery {
        content_type: q.content_type,
        q: q.q,
        page: q.page,
        per_page: q.per_page,
        order: Some("updated".to_string()),
        ..Default::default()
    };
    list_query.public_only = false;

    let page_result = entries::list(&state.db, &list_query).await?;
    let mut rows = Vec::with_capacity(page_result.items.len());

    for entry in page_result.items {
        let siblings = entries::get_translations(&state.db, entry.id).await?;
        rows.push(MatrixEntryRow {
            id: entry.id,
            title: entry.title,
            slug: entry.slug,
            content_type: entry.content_type,
            locale: entry.locale,
            status: entry.status,
            translation_group_id: entry.translation_group_id,
            translations: siblings,
        });
    }

    Ok(Json(TranslationMatrixResponse {
        active_languages,
        items: rows,
        total: page_result.total,
        page: page_result.page,
        per_page: page_result.per_page,
    }))
}

/// Récupère la liste des traductions liées à une entrée.
pub async fn get_entry_siblings(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Vec<TranslationSibling>>> {
    let siblings = entries::get_translations(&state.db, id).await?;
    Ok(Json(siblings))
}

#[derive(Debug, Deserialize)]
pub struct CreateTranslationPayload {
    pub target_locale: String,
}

/// Crée une nouvelle traduction pour une entrée dans la langue cible
/// en clonant l'arbre de blocs natif.
pub async fn create_translation_entry(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateTranslationPayload>,
) -> ApiResult<Json<Entry>> {
    let new_entry = entries::create_translation(
        &state.db,
        state.plugins.as_ref(),
        id,
        &payload.target_locale,
        auth.user.id,
    )
    .await?;

    Ok(Json(new_entry))
}

#[derive(Debug, Deserialize)]
pub struct LinkTranslationPayload {
    pub target_id: Uuid,
}

/// Associe deux entrées dans la même grappe de traduction.
pub async fn link_translations_action(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<LinkTranslationPayload>,
) -> ApiResult<Json<serde_json::Value>> {
    entries::link_translations(&state.db, id, payload.target_id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

/// Retire une entrée de sa grappe de traduction.
pub async fn unlink_translation_action(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    entries::unlink_translation(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

// ---------------------------------------------------------------------------
// Éditeur de Catalogues de Traduction UI (MOD-I18N-02, 03, 04, 05)
// ---------------------------------------------------------------------------

/// Retourne le dictionnaire de traduction pour une locale donnée.
pub async fn get_catalog(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(locale): Path<String>,
) -> ApiResult<Json<TranslationCatalog>> {
    let catalog = i18n::get_catalog(&state.db, &locale).await?;
    Ok(Json(catalog))
}

#[derive(Debug, Deserialize)]
pub struct UpdateCatalogPayload {
    pub items: Vec<CatalogItem>,
}

/// Met à jour les traductions d'un catalogue (sauvegarde des surcharges en base).
pub async fn update_catalog(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(locale): Path<String>,
    Json(payload): Json<UpdateCatalogPayload>,
) -> ApiResult<Json<TranslationCatalog>> {
    if !auth.user.role.can_edit_others() {
        return Err(ApiError::Forbidden);
    }

    let catalog = TranslationCatalog {
        locale: locale.to_lowercase(),
        items: payload.items,
        updated_at: chrono::Utc::now(),
    };

    i18n::save_catalog(&state.db, &catalog).await?;
    Ok(Json(catalog))
}

/// Exporte un catalogue au format standard GNU Gettext (.po).
pub async fn export_catalog_po(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(locale): Path<String>,
) -> ApiResult<Response> {
    let catalog = i18n::get_catalog(&state.db, &locale).await?;
    let po_text = i18n::export_po(&catalog);

    let filename = format!("rustforge-{}.po", locale.to_lowercase());
    let disposition = format!("attachment; filename=\"{filename}\"");

    let response = Response::builder()
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .header(header::CONTENT_DISPOSITION, disposition)
        .body(axum::body::Body::from(po_text))
        .map_err(|e| ApiError::Core(rustforge_core::CoreError::Internal(e.into())))?;

    Ok(response)
}

#[derive(Debug, Deserialize)]
pub struct ImportCatalogPayload {
    pub content: String,
    #[serde(default = "default_format_po")]
    pub format: String,
}

fn default_format_po() -> String {
    "po".to_string()
}

/// Importe un fichier GNU Gettext (.po) ou JSON et fusionne sans écraser les traductions existantes.
pub async fn import_catalog(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(locale): Path<String>,
    Json(payload): Json<ImportCatalogPayload>,
) -> ApiResult<Json<TranslationCatalog>> {
    if !auth.user.role.can_edit_others() {
        return Err(ApiError::Forbidden);
    }

    let imported_cat = if payload.format.to_lowercase() == "json" {
        serde_json::from_str::<TranslationCatalog>(&payload.content)
            .map_err(|e| ApiError::BadRequest(format!("Format JSON invalide : {e}")))?
    } else {
        i18n::parse_po(&payload.content, &locale)
            .map_err(|e| ApiError::BadRequest(format!("Format PO invalide : {e}")))?
    };

    let mut current = i18n::get_catalog(&state.db, &locale).await?;
    i18n::sync_catalog(&mut current, &imported_cat);

    i18n::save_catalog(&state.db, &current).await?;
    Ok(Json(current))
}

/// Synchronise le catalogue de la langue avec le socle natif de RustForge CMS
/// (ajoute les nouveaux textes d'interface sans écraser les traductions renseignées).
pub async fn sync_catalog_action(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(locale): Path<String>,
) -> ApiResult<Json<TranslationCatalog>> {
    if !auth.user.role.can_edit_others() {
        return Err(ApiError::Forbidden);
    }

    let base = i18n::extract_default_catalog(&locale);
    let mut current = i18n::get_catalog(&state.db, &locale).await?;
    i18n::sync_catalog(&mut current, &base);

    i18n::save_catalog(&state.db, &current).await?;
    Ok(Json(current))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_languages_payload_deserialization() {
        let json = r#"{"active":["fr","en","es"],"default":"fr"}"#;
        let payload: SetLanguagesPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.active, vec!["fr", "en", "es"]);
        assert_eq!(payload.default, "fr");
    }

    #[test]
    fn test_import_catalog_payload_default_format() {
        let json = r#"{"content":"msgid \"Hello\"\nmsgstr \"Bonjour\"\n"}"#;
        let payload: ImportCatalogPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.format, "po");
        assert!(payload.content.contains("msgid"));
    }
}
