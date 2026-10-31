//! Endpoints de mesure d'audience et de rapports souverains (MOD-STAT-01 à MOD-STAT-04).

use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

use crate::{
    auth::{AuthUser, verify_token},
    error::{ApiError, ApiResult},
    state::AppState,
};
use rustforge_core::analytics::{self, AnalyticsSettings, AnalyticsSummary, HitInput};

/// Payload public pour la transmission d'un affichage de page (hit)
#[derive(Debug, Deserialize)]
pub struct PublicHitPayload {
    pub path: String,
    pub referrer: Option<String>,
    #[serde(default)]
    pub screen_width: Option<u32>,
}

/// Paramètres de requête pour la synthèse
#[derive(Debug, Deserialize)]
pub struct SummaryQuery {
    pub period: Option<String>,
}

/// Payload pour la demande de purge manuelle
#[derive(Debug, Deserialize)]
pub struct PurgePayload {
    pub retention_days: Option<i32>,
}

/// Réponse générique pour les actions administratives
#[derive(Debug, Serialize)]
pub struct ActionResponse {
    pub deleted: u64,
    pub message: String,
}

/// Collecte publique non bloquante d'un affichage de page.
///
/// Respecte strictement :
/// - DNT / Sec-GPC : acquitté sans écriture.
/// - Robots et indexeurs : ignorés.
/// - Chemins sensibles / admin : exclus.
/// - Utilisateurs connectés avec droits de rédaction : exclus.
pub async fn public_hit(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<PublicHitPayload>,
) -> Response {
    if !rustforge_core::settings::is_module_active(&state.db, "analytics").await {
        return StatusCode::NO_CONTENT.into_response();
    }

    // Contrôle DNT (Do Not Track) et Sec-GPC (Global Privacy Control)
    let dnt = headers
        .get("dnt")
        .and_then(|h| h.to_str().ok())
        .map(|v| v == "1")
        .unwrap_or(false)
        || headers
            .get("sec-gpc")
            .and_then(|h| h.to_str().ok())
            .map(|v| v == "1")
            .unwrap_or(false);

    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    let referrer = payload
        .referrer
        .filter(|r| !r.trim().is_empty())
        .or_else(|| {
            headers
                .get(header::REFERER)
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string())
        });

    let site_host = headers.get(header::HOST).and_then(|h| h.to_str().ok());

    // Détermination de l'IP cliente selon les en-têtes de reverse proxy de confiance
    let client_ip = headers
        .get("cf-connecting-ip")
        .or_else(|| headers.get("x-real-ip"))
        .or_else(|| headers.get("x-forwarded-for"))
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "127.0.0.1".to_string());

    // Détecte si le visiteur dispose d'un jeton de session connecté avec rôle rédacteur
    let mut is_authenticated_staff = false;
    if let Some(auth_header) = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        && let Some(token) = auth_header.strip_prefix("Bearer ")
        && let Ok(claims) = verify_token(token, &state.jwt_secret)
    {
        is_authenticated_staff = claims.role.can_write();
    }

    let hit = HitInput {
        path: payload.path,
        referrer,
        user_agent,
        ip: client_ip,
        dnt,
        is_authenticated_staff,
        country: None,
    };

    let _ = analytics::record_hit(&state.db, hit, site_host).await;

    StatusCode::NO_CONTENT.into_response()
}

/// Retourne la synthèse des métriques pour la période demandée (MOD-STAT-03)
pub async fn get_summary(
    State(state): State<AppState>,
    Query(query): Query<SummaryQuery>,
    auth: AuthUser,
) -> ApiResult<Json<AnalyticsSummary>> {
    rustforge_core::security::require_writer(auth.user.role)?;
    let period = query.period.unwrap_or_else(|| "7d".to_string());
    let summary = analytics::get_summary(&state.db, &period).await?;
    Ok(Json(summary))
}

/// Récupère la configuration de mesure d'audience
pub async fn get_settings(
    State(state): State<AppState>,
    auth: AuthUser,
) -> ApiResult<Json<AnalyticsSettings>> {
    if !auth.user.role.is_admin() {
        return Err(ApiError::Forbidden);
    }
    let settings = analytics::get_settings(&state.db).await?;
    Ok(Json(settings))
}

/// Met à jour la configuration de mesure d'audience
pub async fn update_settings(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(settings): Json<AnalyticsSettings>,
) -> ApiResult<Json<AnalyticsSettings>> {
    if !auth.user.role.is_admin() {
        return Err(ApiError::Forbidden);
    }
    analytics::save_settings(&state.db, &settings).await?;
    Ok(Json(settings))
}

/// Purge les enregistrements anciens (MOD-STAT-04)
pub async fn purge(
    State(state): State<AppState>,
    auth: AuthUser,
    payload: Option<Json<PurgePayload>>,
) -> ApiResult<Json<ActionResponse>> {
    if !auth.user.role.is_admin() {
        return Err(ApiError::Forbidden);
    }
    let days = if let Some(Json(p)) = payload
        && let Some(d) = p.retention_days
    {
        d
    } else {
        let cfg = analytics::get_settings(&state.db).await?;
        cfg.retention_months * 30
    };

    let count = analytics::purge_records_older_than(&state.db, days).await?;
    Ok(Json(ActionResponse {
        deleted: count,
        message: format!("Purge réussie : {count} enregistrements supprimés."),
    }))
}

/// Efface toutes les données de mesure d'audience (MOD-STAT-04)
pub async fn clear_data(
    State(state): State<AppState>,
    auth: AuthUser,
) -> ApiResult<Json<ActionResponse>> {
    if !auth.user.role.is_admin() {
        return Err(ApiError::Forbidden);
    }
    let count = analytics::clear_all_records(&state.db).await?;
    Ok(Json(ActionResponse {
        deleted: count,
        message: format!("Données d'audience réinitialisées : {count} enregistrements supprimés."),
    }))
}

/// Exporte un rapport CSV d'audience sécurisé contre l'injection de formules tableur
pub async fn export_csv(
    State(state): State<AppState>,
    Query(query): Query<SummaryQuery>,
    auth: AuthUser,
) -> ApiResult<Response> {
    rustforge_core::security::require_writer(auth.user.role)?;
    let period = query.period.unwrap_or_else(|| "30d".to_string());
    let summary = analytics::get_summary(&state.db, &period).await?;
    let csv = analytics::export_summary_csv(&summary);

    let headers = [
        (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"statistiques-audience.csv\"",
        ),
    ];
    Ok((StatusCode::OK, headers, csv).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::create_token;
    use rustforge_core::models::Role;
    use uuid::Uuid;

    #[test]
    fn test_public_hit_payload_deserialization() {
        let json = r#"{"path": "/mon-article", "referrer": "https://example.com/source", "screen_width": 1280}"#;
        let payload: PublicHitPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.path, "/mon-article");
        assert_eq!(
            payload.referrer.as_deref(),
            Some("https://example.com/source")
        );
        assert_eq!(payload.screen_width, Some(1280));

        let minimal_json = r#"{"path": "/"}"#;
        let min_payload: PublicHitPayload = serde_json::from_str(minimal_json).unwrap();
        assert_eq!(min_payload.path, "/");
        assert_eq!(min_payload.referrer, None);
        assert_eq!(min_payload.screen_width, None);
    }

    #[test]
    fn test_staff_token_detection() {
        let secret = "secret-test-long-enough-32-chars-ok";
        let admin_token = create_token(Uuid::new_v4(), Role::Admin, secret).unwrap();
        let sub_token = create_token(Uuid::new_v4(), Role::Subscriber, secret).unwrap();

        let admin_claims = verify_token(&admin_token, secret).unwrap();
        assert!(admin_claims.role.can_write());

        let sub_claims = verify_token(&sub_token, secret).unwrap();
        assert!(!sub_claims.role.can_write());
    }
}
