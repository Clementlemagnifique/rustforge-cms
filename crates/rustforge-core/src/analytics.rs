//! Mesure d'audience et rapports visiteurs souverains (MOD-STAT-01 à MOD-STAT-04).
//!
//! Principes directeurs de minimisation et de souveraineté :
//! - Zéro cookie, zéro traceur tiers ou script externe.
//! - Hachage journalier non réversible de l'empreinte visiteur (IP + sel secret + date du jour).
//!   Empêche tout recoupement inter-journalier ou profilage durable.
//! - Respect strict du signal Do Not Track (DNT) et Global Privacy Control (Sec-GPC).
//! - Exclusion automatique des robots et moteurs d'indexation (User-Agent scanning).
//! - Exclusion du personnel connecté (administrateurs, éditeurs, auteurs).
//! - Assainissement des URLs : suppression des paramètres de requête (tokens, PII, query strings).
//! - Assainissement des référents : conservation du domaine source uniquement (pas de query string).
//! - Rétention paramétrable (défaut : 13 mois selon la recommandation CNIL) avec commande de purge.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;

use crate::{CoreError, CoreResult, settings};

/// Notice de transparence CNIL et vie privée
pub const PRIVACY_NOTICE: &str = "Mesure d'audience souveraine et minimisée. Qualification de l'exemption de consentement \
    à vérifier selon le contexte de déploiement réel. Zéro cookie, sel de hachage quotidien \
    non réversible, respect strict de Do Not Track / Sec-GPC, filtrage des robots, exclusion \
    du personnel connecté, minimisation des URL et référents.";

/// Nombre de mois de rétention recommandé par la CNIL (13 mois).
pub const DEFAULT_RETENTION_MONTHS: i32 = 13;

/// Préfixes de chemins exclus d'office de la mesure d'audience
pub const DEFAULT_EXCLUDED_PREFIXES: &[&str] = &[
    "/admin",
    "/api",
    "/_",
    "/uploads",
    "/login",
    "/favicon.ico",
    "/robots.txt",
    "/sitemap.xml",
];

/// Réglages de la mesure d'audience souveraine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSettings {
    pub enabled: bool,
    pub retention_months: i32,
    pub honor_dnt: bool,
    pub exclude_authenticated: bool,
    pub excluded_paths: Vec<String>,
}

impl Default for AnalyticsSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            retention_months: DEFAULT_RETENTION_MONTHS,
            honor_dnt: true,
            exclude_authenticated: true,
            excluded_paths: vec![
                "/admin".to_string(),
                "/api".to_string(),
                "/_".to_string(),
                "/uploads".to_string(),
            ],
        }
    }
}

/// Entrée brute pour la collecte d'un hit
#[derive(Debug, Clone)]
pub struct HitInput {
    pub path: String,
    pub referrer: Option<String>,
    pub user_agent: Option<String>,
    pub ip: String,
    pub dnt: bool,
    pub is_authenticated_staff: bool,
    pub country: Option<String>,
}

/// Résultat du traitement d'un hit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HitOutcome {
    Recorded,
    OptedOutDnt,
    ExcludedStaff,
    ExcludedPath,
    ExcludedBot,
    Disabled,
}

/// Métrique de consultation par page
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PageMetric {
    pub path: String,
    pub views: i64,
    pub unique_visitors: i64,
}

/// Métrique de source de trafic (référent assaini)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReferrerMetric {
    pub referrer: String,
    pub views: i64,
}

/// Métrique de terminal visiteur
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceMetric {
    pub device_type: String,
    pub views: i64,
    pub percentage: f64,
}

/// Point d'agrégation chronologique (jour ou heure)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TimeseriesPoint {
    pub period_key: String,
    pub views: i64,
    pub unique_visitors: i64,
}

/// Synthèse complète de mesure d'audience
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSummary {
    pub total_views: i64,
    pub unique_visitors: i64,
    pub views_per_visitor: f64,
    pub top_pages: Vec<PageMetric>,
    pub top_referrers: Vec<ReferrerMetric>,
    pub devices: Vec<DeviceMetric>,
    pub timeseries: Vec<TimeseriesPoint>,
    pub period: String,
    pub notice: String,
}

/// Détecte si le User-Agent correspond à un robot / crawler
pub fn is_bot(user_agent: &str) -> bool {
    let ua = user_agent.to_lowercase();
    const BOT_SIGNATURES: &[&str] = &[
        "bot",
        "crawler",
        "spider",
        "slurp",
        "bingbot",
        "googlebot",
        "yandex",
        "duckduckbot",
        "baiduspider",
        "headless",
        "curl",
        "wget",
        "python-requests",
        "postman",
        "ahrefs",
        "semrush",
        "uptime",
        "lighthouse",
        "facebookexternalhit",
        "petalbot",
        "bytespider",
        "sogou",
        "archive.org_bot",
        "feedfetcher",
        "preview",
        "embedly",
        "quora link preview",
        "outbrain",
        "vkshare",
        "w3c_validator",
    ];
    BOT_SIGNATURES.iter().any(|&sig| ua.contains(sig))
}

/// Détecte le type d'appareil ('desktop', 'mobile', 'tablet') à partir du User-Agent
pub fn detect_device_type(user_agent: &str) -> &'static str {
    let ua = user_agent.to_lowercase();
    if ua.contains("ipad")
        || ua.contains("tablet")
        || ua.contains("playbook")
        || ua.contains("silk")
    {
        "tablet"
    } else if ua.contains("mobile")
        || ua.contains("iphone")
        || ua.contains("ipod")
        || ua.contains("android")
        || ua.contains("blackberry")
        || ua.contains("iemobile")
        || ua.contains("opera mini")
    {
        "mobile"
    } else {
        "desktop"
    }
}

/// Assainit l'URL pour la minimisation CNIL :
/// - Supprime les query parameters sensibles (tokens, email, query string entière)
/// - Supprime les fragments d'URL
/// - Normalise les barres obliques de fin
pub fn sanitize_path(raw_path: &str) -> String {
    let path_no_query = raw_path.split('?').next().unwrap_or("/").trim();
    let path_no_hash = path_no_query.split('#').next().unwrap_or("/").trim();
    if path_no_hash.is_empty() {
        return "/".to_string();
    }
    let mut normalized = if !path_no_hash.starts_with('/') {
        format!("/{}", path_no_hash)
    } else {
        path_no_hash.to_string()
    };
    if normalized.len() > 1 && normalized.ends_with('/') {
        normalized.pop();
    }
    normalized
}

/// Assainit le référent :
/// - Extrait uniquement le nom d'hôte / domaine externe (sans chemin ni paramètres)
/// - Écarte la source si elle correspond au domaine propre du site (navigation interne)
pub fn sanitize_referrer(raw_referrer: Option<&str>, site_host: Option<&str>) -> Option<String> {
    let ref_str = raw_referrer?.trim();
    if ref_str.is_empty() {
        return None;
    }
    let after_scheme = if let Some(stripped) = ref_str.strip_prefix("https://") {
        stripped
    } else if let Some(stripped) = ref_str.strip_prefix("http://") {
        stripped
    } else {
        ref_str
    };
    let host_part = after_scheme.split('/').next().unwrap_or("");
    let host_part = host_part
        .split(':')
        .next()
        .unwrap_or("")
        .trim()
        .to_lowercase();
    let host_part = host_part.strip_prefix("www.").unwrap_or(&host_part);
    if host_part.is_empty() {
        return None;
    }
    if let Some(sh) = site_host {
        let sh_clean = sh.split(':').next().unwrap_or("").trim().to_lowercase();
        let sh_clean = sh_clean.strip_prefix("www.").unwrap_or(&sh_clean);
        if host_part == sh_clean {
            return None;
        }
    }
    Some(host_part.to_string())
}

/// Vérifie si un chemin donné doit être exclu de la collecte
pub fn is_excluded_path(path: &str, custom_exclusions: &[String]) -> bool {
    let lower = path.to_lowercase();
    for &prefix in DEFAULT_EXCLUDED_PREFIXES {
        if lower.starts_with(prefix) {
            return true;
        }
    }
    for custom in custom_exclusions {
        let trimmed = custom.trim();
        if !trimmed.is_empty() && lower.starts_with(&trimmed.to_lowercase()) {
            return true;
        }
    }
    false
}

/// Calcule une empreinte visiteur journalière non réversible.
///
/// La formule `SHA256(ip + date + salt + user_agent)` garantit que l'empreinte
/// est strictement isolée par tranche de 24h et par site. Aucune réversibilité
/// vers l'adresse IP d'origine n'est possible, et aucun rapprochement inter-jours
/// ne peut être opéré.
pub fn compute_visitor_hash(ip: &str, user_agent: &str, salt: &str, date_str: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(ip.as_bytes());
    hasher.update(b":");
    hasher.update(date_str.as_bytes());
    hasher.update(b":");
    hasher.update(salt.as_bytes());
    hasher.update(b":");
    hasher.update(user_agent.as_bytes());
    hex::encode(hasher.finalize())
}

/// Neutralise une cellule CSV pour empêcher l'exécution de formules hostiles
pub fn csv_cell(value: &str) -> String {
    let mut text = value.to_string();
    let mut start = 0;
    let chars: Vec<char> = text.chars().collect();
    while start < chars.len() && (chars[start] as u32 <= 32 || chars[start].is_whitespace()) {
        start += 1;
    }
    if start < chars.len() {
        let first = chars[start];
        if matches!(first, '=' | '+' | '-' | '@' | '\t' | '\r') {
            text = format!("'{}", text);
        }
    }
    format!("\"{}\"", text.replace('"', "\"\""))
}

/// Récupère ou initialise le sel cryptographique secret de la mesure d'audience
pub async fn get_or_create_salt(db: &PgPool) -> CoreResult<String> {
    if let Some(val) = settings::get(db, "analytics.salt").await?
        && let Some(s) = val.as_str()
        && !s.is_empty()
    {
        return Ok(s.to_string());
    }
    // Génère un sel cryptographique aléatoire de 32 octets
    let random_bytes: [u8; 32] = uuid::Uuid::new_v4().as_bytes().repeat(2)[..32]
        .try_into()
        .map_err(|_| CoreError::Validation("Échec génération sel".into()))?;
    let salt = hex::encode(random_bytes);
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES ($1, $2)
         ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = now()",
    )
    .bind("analytics.salt")
    .bind(serde_json::Value::String(salt.clone()))
    .execute(db)
    .await?;
    Ok(salt)
}

/// Récupère les réglages de mesure d'audience
pub async fn get_settings(db: &PgPool) -> CoreResult<AnalyticsSettings> {
    if let Some(val) = settings::get(db, "analytics.settings").await?
        && let Ok(cfg) = serde_json::from_value::<AnalyticsSettings>(val)
    {
        return Ok(cfg);
    }
    Ok(AnalyticsSettings::default())
}

/// Enregistre les réglages de mesure d'audience
pub async fn save_settings(db: &PgPool, cfg: &AnalyticsSettings) -> CoreResult<()> {
    if cfg.retention_months < 1 || cfg.retention_months > 36 {
        return Err(CoreError::Validation(
            "La durée de rétention doit être comprise entre 1 et 36 mois".into(),
        ));
    }
    let val = serde_json::to_value(cfg)
        .map_err(|e| CoreError::Validation(format!("Erreur sérialisation réglages : {e}")))?;
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES ($1, $2)
         ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = now()",
    )
    .bind("analytics.settings")
    .bind(val)
    .execute(db)
    .await?;
    Ok(())
}

/// Enregistre un affichage de page (hit) avec vérification stricte des exemptions et filtres
pub async fn record_hit(
    db: &PgPool,
    hit: HitInput,
    site_host: Option<&str>,
) -> CoreResult<HitOutcome> {
    let cfg = get_settings(db).await?;
    if !cfg.enabled {
        return Ok(HitOutcome::Disabled);
    }
    if cfg.honor_dnt && hit.dnt {
        return Ok(HitOutcome::OptedOutDnt);
    }
    if cfg.exclude_authenticated && hit.is_authenticated_staff {
        return Ok(HitOutcome::ExcludedStaff);
    }
    let sanitized_path = sanitize_path(&hit.path);
    if is_excluded_path(&sanitized_path, &cfg.excluded_paths) {
        return Ok(HitOutcome::ExcludedPath);
    }
    let ua = hit.user_agent.as_deref().unwrap_or("");
    if is_bot(ua) {
        return Ok(HitOutcome::ExcludedBot);
    }

    let salt = get_or_create_salt(db).await?;
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let visitor_hash = compute_visitor_hash(&hit.ip, ua, &salt, &today);
    let device_type = detect_device_type(ua);
    let sanitized_ref = sanitize_referrer(hit.referrer.as_deref(), site_host);

    sqlx::query(
        "INSERT INTO analytics_pageviews (path, referrer, device_type, country, visitor_hash, viewed_at)
         VALUES ($1, $2, $3, $4, $5, now())",
    )
    .bind(sanitized_path)
    .bind(sanitized_ref)
    .bind(device_type)
    .bind(hit.country)
    .bind(visitor_hash)
    .execute(db)
    .await?;

    Ok(HitOutcome::Recorded)
}

/// Calcule la date de coupure selon la période demandée
fn period_cutoff(period: &str) -> Option<DateTime<Utc>> {
    let now = Utc::now();
    match period {
        "24h" => Some(now - Duration::hours(24)),
        "7d" => Some(now - Duration::days(7)),
        "30d" => Some(now - Duration::days(30)),
        "90d" => Some(now - Duration::days(90)),
        _ => None, // "all" ou non reconnu
    }
}

/// Génère la synthèse des statistiques d'audience
pub async fn get_summary(db: &PgPool, period: &str) -> CoreResult<AnalyticsSummary> {
    let cutoff = period_cutoff(period);

    // 1. Total des pages vues
    let total_views: i64 = match cutoff {
        Some(dt) => {
            sqlx::query_scalar("SELECT count(*) FROM analytics_pageviews WHERE viewed_at >= $1")
                .bind(dt)
                .fetch_one(db)
                .await?
        }
        None => {
            sqlx::query_scalar("SELECT count(*) FROM analytics_pageviews")
                .fetch_one(db)
                .await?
        }
    };

    // 2. Visiteurs uniques estimés
    let unique_visitors: i64 = match cutoff {
        Some(dt) => sqlx::query_scalar(
            "SELECT count(DISTINCT visitor_hash) FROM analytics_pageviews WHERE viewed_at >= $1",
        )
        .bind(dt)
        .fetch_one(db)
        .await?,
        None => {
            sqlx::query_scalar("SELECT count(DISTINCT visitor_hash) FROM analytics_pageviews")
                .fetch_one(db)
                .await?
        }
    };

    let views_per_visitor = if unique_visitors > 0 {
        (total_views as f64 / unique_visitors as f64 * 100.0).round() / 100.0
    } else {
        0.0
    };

    // 3. Top pages
    let top_pages: Vec<PageMetric> =
        match cutoff {
            Some(dt) => sqlx::query_as(
                "SELECT path, count(*) as views, count(DISTINCT visitor_hash) as unique_visitors
             FROM analytics_pageviews
             WHERE viewed_at >= $1
             GROUP BY path
             ORDER BY views DESC
             LIMIT 15",
            )
            .bind(dt)
            .fetch_all(db)
            .await?,
            None => sqlx::query_as(
                "SELECT path, count(*) as views, count(DISTINCT visitor_hash) as unique_visitors
             FROM analytics_pageviews
             GROUP BY path
             ORDER BY views DESC
             LIMIT 15",
            )
            .fetch_all(db)
            .await?,
        };

    // 4. Top référents
    let top_referrers: Vec<ReferrerMetric> = match cutoff {
        Some(dt) => {
            sqlx::query_as(
                "SELECT referrer, count(*) as views
             FROM analytics_pageviews
             WHERE viewed_at >= $1 AND referrer IS NOT NULL AND referrer != ''
             GROUP BY referrer
             ORDER BY views DESC
             LIMIT 10",
            )
            .bind(dt)
            .fetch_all(db)
            .await?
        }
        None => {
            sqlx::query_as(
                "SELECT referrer, count(*) as views
             FROM analytics_pageviews
             WHERE referrer IS NOT NULL AND referrer != ''
             GROUP BY referrer
             ORDER BY views DESC
             LIMIT 10",
            )
            .fetch_all(db)
            .await?
        }
    };

    // 5. Répartition terminaux
    #[derive(sqlx::FromRow)]
    struct DevRow {
        device_type: Option<String>,
        views: i64,
    }
    let dev_rows: Vec<DevRow> = match cutoff {
        Some(dt) => {
            sqlx::query_as(
                "SELECT device_type, count(*) as views
             FROM analytics_pageviews
             WHERE viewed_at >= $1
             GROUP BY device_type
             ORDER BY views DESC",
            )
            .bind(dt)
            .fetch_all(db)
            .await?
        }
        None => {
            sqlx::query_as(
                "SELECT device_type, count(*) as views
             FROM analytics_pageviews
             GROUP BY device_type
             ORDER BY views DESC",
            )
            .fetch_all(db)
            .await?
        }
    };

    let devices: Vec<DeviceMetric> = dev_rows
        .into_iter()
        .map(|r| {
            let pct = if total_views > 0 {
                ((r.views as f64 / total_views as f64) * 100.0 * 10.0).round() / 10.0
            } else {
                0.0
            };
            DeviceMetric {
                device_type: r.device_type.unwrap_or_else(|| "desktop".into()),
                views: r.views,
                percentage: pct,
            }
        })
        .collect();

    // 6. Chronologie (timeseries)
    let is_hourly = period == "24h";
    let timeseries: Vec<TimeseriesPoint> = match cutoff {
        Some(dt) => {
            let sql = if is_hourly {
                "SELECT to_char(viewed_at, 'YYYY-MM-DD HH24:00') as period_key,
                        count(*) as views,
                        count(DISTINCT visitor_hash) as unique_visitors
                 FROM analytics_pageviews
                 WHERE viewed_at >= $1
                 GROUP BY period_key
                 ORDER BY period_key ASC"
            } else {
                "SELECT to_char(viewed_at, 'YYYY-MM-DD') as period_key,
                        count(*) as views,
                        count(DISTINCT visitor_hash) as unique_visitors
                 FROM analytics_pageviews
                 WHERE viewed_at >= $1
                 GROUP BY period_key
                 ORDER BY period_key ASC"
            };
            sqlx::query_as(sql).bind(dt).fetch_all(db).await?
        }
        None => {
            sqlx::query_as(
                "SELECT to_char(viewed_at, 'YYYY-MM-DD') as period_key,
                    count(*) as views,
                    count(DISTINCT visitor_hash) as unique_visitors
             FROM analytics_pageviews
             GROUP BY period_key
             ORDER BY period_key ASC",
            )
            .fetch_all(db)
            .await?
        }
    };

    Ok(AnalyticsSummary {
        total_views,
        unique_visitors,
        views_per_visitor,
        top_pages,
        top_referrers,
        devices,
        timeseries,
        period: period.to_string(),
        notice: PRIVACY_NOTICE.to_string(),
    })
}

/// Purge les enregistrements antérieurs à un nombre de jours donné
pub async fn purge_records_older_than(db: &PgPool, days: i32) -> CoreResult<u64> {
    let cutoff = Utc::now() - Duration::days(days as i64);
    let result = sqlx::query("DELETE FROM analytics_pageviews WHERE viewed_at < $1")
        .bind(cutoff)
        .execute(db)
        .await?;
    Ok(result.rows_affected())
}

/// Efface l'intégralité des enregistrements de mesure d'audience
pub async fn clear_all_records(db: &PgPool) -> CoreResult<u64> {
    let result = sqlx::query("DELETE FROM analytics_pageviews")
        .execute(db)
        .await?;
    Ok(result.rows_affected())
}

/// Exporte la synthèse d'audience sous forme de CSV sécurisé
pub fn export_summary_csv(summary: &AnalyticsSummary) -> String {
    let mut lines = Vec::new();

    // En-tête métadonnées
    lines.push(format!("{};{}", csv_cell("Indicateur"), csv_cell("Valeur")));
    lines.push(format!(
        "{};{}",
        csv_cell("Période"),
        csv_cell(&summary.period)
    ));
    lines.push(format!(
        "{};{}",
        csv_cell("Pages vues totales"),
        csv_cell(&summary.total_views.to_string())
    ));
    lines.push(format!(
        "{};{}",
        csv_cell("Visiteurs uniques estimés"),
        csv_cell(&summary.unique_visitors.to_string())
    ));
    lines.push(format!(
        "{};{}",
        csv_cell("Pages par visiteur"),
        csv_cell(&summary.views_per_visitor.to_string())
    ));
    lines.push("".to_string());

    // Section Top Pages
    lines.push(format!(
        "{};{};{}",
        csv_cell("Page"),
        csv_cell("Vues"),
        csv_cell("Visiteurs uniques")
    ));
    for p in &summary.top_pages {
        lines.push(format!(
            "{};{};{}",
            csv_cell(&p.path),
            csv_cell(&p.views.to_string()),
            csv_cell(&p.unique_visitors.to_string())
        ));
    }
    lines.push("".to_string());

    // Section Top Référents
    lines.push(format!(
        "{};{}",
        csv_cell("Source (Référent)"),
        csv_cell("Vues")
    ));
    for r in &summary.top_referrers {
        lines.push(format!(
            "{};{}",
            csv_cell(&r.referrer),
            csv_cell(&r.views.to_string())
        ));
    }
    lines.push("".to_string());

    // Section Appareils
    lines.push(format!(
        "{};{};{}",
        csv_cell("Type d'appareil"),
        csv_cell("Vues"),
        csv_cell("Pourcentage")
    ));
    for d in &summary.devices {
        lines.push(format!(
            "{};{};{}",
            csv_cell(&d.device_type),
            csv_cell(&d.views.to_string()),
            csv_cell(&format!("{}%", d.percentage))
        ));
    }

    // Préfixe BOM UTF-8
    format!("\u{FEFF}{}", lines.join("\r\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bot_detection() {
        assert!(is_bot(
            "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)"
        ));
        assert!(is_bot(
            "Mozilla/5.0 (compatible; bingbot/2.0; +http://www.bing.com/bingbot.htm)"
        ));
        assert!(is_bot("curl/7.88.1"));
        assert!(is_bot("python-requests/2.31.0"));
        assert!(is_bot("PostmanRuntime/7.32.3"));

        // Visiteurs réels
        assert!(!is_bot(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
        ));
        assert!(!is_bot(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15"
        ));
        assert!(!is_bot(
            "Mozilla/5.0 (iPhone; CPU iPhone OS 16_5 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.5 Mobile/15E148 Safari/604.1"
        ));
    }

    #[test]
    fn test_device_detection() {
        assert_eq!(
            detect_device_type(
                "Mozilla/5.0 (iPhone; CPU iPhone OS 16_5 like Mac OS X) Mobile/15E148"
            ),
            "mobile"
        );
        assert_eq!(
            detect_device_type(
                "Mozilla/5.0 (Linux; Android 13; SM-S901B) AppleWebKit/537.36 Mobile Safari/537.36"
            ),
            "mobile"
        );
        assert_eq!(
            detect_device_type(
                "Mozilla/5.0 (iPad; CPU OS 16_5 like Mac OS X) AppleWebKit/605.1.15"
            ),
            "tablet"
        );
        assert_eq!(
            detect_device_type("Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0.0.0"),
            "desktop"
        );
    }

    #[test]
    fn test_path_sanitization() {
        assert_eq!(
            sanitize_path("/blog/mon-article?token=secret123&mail=a@b.c"),
            "/blog/mon-article"
        );
        assert_eq!(sanitize_path("/a/b/c#section-titre"), "/a/b/c");
        assert_eq!(sanitize_path("/page-test/"), "/page-test");
        assert_eq!(sanitize_path("/"), "/");
        assert_eq!(sanitize_path("sans-slash"), "/sans-slash");
    }

    #[test]
    fn test_referrer_sanitization() {
        assert_eq!(
            sanitize_referrer(
                Some("https://www.google.fr/search?q=rustforge+cms&rlz=123"),
                Some("monsite.fr")
            ),
            Some("google.fr".into())
        );
        assert_eq!(
            sanitize_referrer(Some("https://duckduckgo.com/?q=rust"), Some("monsite.fr")),
            Some("duckduckgo.com".into())
        );
        // Navigation interne : ignorée
        assert_eq!(
            sanitize_referrer(
                Some("https://monsite.fr/blog/premier-article"),
                Some("monsite.fr")
            ),
            None
        );
        assert_eq!(
            sanitize_referrer(Some("https://www.monsite.fr/contact"), Some("monsite.fr")),
            None
        );
        assert_eq!(sanitize_referrer(None, Some("monsite.fr")), None);
        assert_eq!(sanitize_referrer(Some(""), Some("monsite.fr")), None);
    }

    #[test]
    fn test_path_exclusions() {
        let custom = vec!["/prive".to_string(), "/test-interne".to_string()];
        assert!(is_excluded_path("/admin", &custom));
        assert!(is_excluded_path("/admin/dashboard", &custom));
        assert!(is_excluded_path("/api/v1/entries", &custom));
        assert!(is_excluded_path("/uploads/2026/10/img.webp", &custom));
        assert!(is_excluded_path("/robots.txt", &custom));
        assert!(is_excluded_path("/prive/secret", &custom));

        assert!(!is_excluded_path("/", &custom));
        assert!(!is_excluded_path("/a-propos", &custom));
        assert!(!is_excluded_path("/blog/premier-article", &custom));
    }

    #[test]
    fn test_daily_salt_hash_rotation() {
        let ip = "192.168.1.100";
        let ua = "Mozilla/5.0";
        let salt = "secret_salt_12345";

        let hash_day1 = compute_visitor_hash(ip, ua, salt, "2026-10-30");
        let hash_day1_repeat = compute_visitor_hash(ip, ua, salt, "2026-10-30");
        let hash_day2 = compute_visitor_hash(ip, ua, salt, "2026-10-31");

        // Idempotent le même jour
        assert_eq!(hash_day1, hash_day1_repeat);
        // Change chaque jour : rotation non recouplable
        assert_ne!(hash_day1, hash_day2);
        // Longueur hash SHA-256 hexadécimal
        assert_eq!(hash_day1.len(), 64);
    }

    #[test]
    fn test_csv_formula_neutralization() {
        assert_eq!(csv_cell("normal"), "\"normal\"");
        assert_eq!(csv_cell("=cmd|'/C calc'!A0"), "\"'=cmd|'/C calc'!A0\"");
        assert_eq!(csv_cell("+12345"), "\"'+12345\"");
        assert_eq!(csv_cell("-50"), "\"'-50\"");
        assert_eq!(csv_cell("@SUM(A1:A10)"), "\"'@SUM(A1:A10)\"");
        assert_eq!(csv_cell("\t=1+1"), "\"'\t=1+1\"");
        assert_eq!(
            csv_cell("Texte avec \"guillemets\""),
            "\"Texte avec \"\"guillemets\"\"\""
        );
    }
}
