//! # Moteur SSR Hybride (MiniJinja 3 + Cache Moka L1)
//!
//! Assure un rendu HTML des pages publiques en < 1ms via cache mémoire,
//! avec gestion native du multilinguisme (/fr/..., /en/...), pré-rendu et invalidation sélective.

use std::{path::PathBuf, sync::Arc, time::Duration};
use minijinja::{Environment, Value};
use moka::future::Cache;

use crate::error::{ApiError, ApiResult};

/// Clé de cache pour une URL publique (ex: "fr:/", "en:/about", "fr:/post/puissance-rust").
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    pub locale: String,
    pub path: String,
}

/// Moteur de rendu SSR dynamique et cache L1 en RAM.
#[derive(Clone)]
pub struct SsrEngine {
    env: Arc<Environment<'static>>,
    cache: Cache<CacheKey, String>,
    pub themes_dir: PathBuf,
    pub active_theme: String,
}

impl SsrEngine {
    pub fn new(themes_dir: PathBuf, active_theme: String) -> Self {
        let mut env = Environment::new();
        let theme_templates = themes_dir.join(&active_theme).join("templates");

        if theme_templates.exists() {
            tracing::info!(path = ?theme_templates, "Moteur SSR : Chargement dynamique des templates depuis le thème");
            env.set_loader(minijinja::path_loader(&theme_templates));
        } else {
            tracing::info!("Moteur SSR : Chargement des templates intégrés par défaut");
            env.add_template("base.html", include_str!("../templates/base.html"))
                .expect("template base.html invalide");
            env.add_template("home.html", include_str!("../templates/home.html"))
                .expect("template home.html invalide");
            env.add_template("single.html", include_str!("../templates/single.html"))
                .expect("template single.html invalide");
            env.add_template("zen.html", include_str!("../templates/zen.html"))
                .expect("template zen.html invalide");
        }

        // Cache Moka : jusqu'à 5 000 pages HTML en mémoire, TTL 1 heure, invalidation automatique
        let cache = Cache::builder()
            .max_capacity(5_000)
            .time_to_live(Duration::from_secs(3600))
            .build();

        Self {
            env: Arc::new(env),
            cache,
            themes_dir,
            active_theme,
        }
    }

    /// Récupère le HTML depuis le cache L1 ou effectue le rendu via MiniJinja.
    pub async fn render_page<F, Fut>(&self, locale: &str, path: &str, fetch_data: F) -> ApiResult<String>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = ApiResult<Value>>,
    {
        let key = CacheKey {
            locale: locale.to_string(),
            path: path.to_string(),
        };

        // 1. TENTATIVE CACHE HIT L1 (RAM < 1ms)
        if let Some(html) = self.cache.get(&key).await {
            tracing::trace!(locale, path, "SSR Cache L1 HIT (< 1ms)");
            return Ok(html);
        }

        // 2. CACHE MISS : Rendu dynamique MiniJinja 3
        tracing::debug!(locale, path, "SSR Cache L1 MISS - Génération MiniJinja");
        let ctx = fetch_data().await?;

        let template_name = ctx.get_attr("template")
            .ok()
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_else(|| "single.html".into());

        let tmpl = self.env.get_template(&template_name)
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("erreur template : {e}")))?;

        let html = tmpl.render(ctx)
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("erreur rendu SSR : {e}")))?;

        // 3. Stockage en cache mémoire
        self.cache.insert(key, html.clone()).await;

        Ok(html)
    }

    /// Invalide une page précise (ex: après mise à jour).
    pub async fn invalidate(&self, locale: &str, path: &str) {
        let key = CacheKey {
            locale: locale.to_string(),
            path: path.to_string(),
        };
        self.cache.invalidate(&key).await;
    }

    /// Invalide tout le cache (ex: changement de thème ou tokens globaux).
    pub async fn invalidate_all(&self) {
        self.cache.invalidate_all();
    }
}
