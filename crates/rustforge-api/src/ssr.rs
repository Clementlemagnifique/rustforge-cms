//! # Moteur SSR Hybride (MiniJinja 3 + Cache Moka L1)
//!
//! Assure un rendu HTML des pages publiques en < 1ms via cache mémoire,
//! avec gestion native du multilinguisme (/fr/..., /en/...), pré-rendu et invalidation sélective.

use minijinja::{Environment, Value};
use moka::future::Cache;
use std::{
    path::PathBuf,
    sync::{
        Arc, RwLock,
        atomic::{AtomicI64, Ordering},
    },
    time::Duration,
};

use crate::error::{ApiError, ApiResult};

/// Clé de cache pour une URL publique (ex: "fr:/", "en:/about", "fr:/post/puissance-rust").
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    pub revision: i64,
    pub visibility_epoch: i64,
    pub locale: String,
    pub path: String,
}

/// Moteur de rendu SSR dynamique et cache L1 en RAM.
#[derive(Clone)]
pub struct SsrEngine {
    env: Arc<RwLock<Environment<'static>>>,
    revision: Arc<AtomicI64>,
    visibility_epoch: Arc<AtomicI64>,
    reload: Arc<tokio::sync::Mutex<()>>,
    cache: Cache<CacheKey, String>,
    pub themes_dir: PathBuf,
    pub active_theme: String,
}

fn theme_loader(
    path: PathBuf,
) -> impl Fn(&str) -> Result<Option<String>, minijinja::Error> + Send + Sync + 'static {
    let disk = minijinja::path_loader(path);
    move |name| {
        if let Some(source) = disk(name)? {
            return Ok(Some(source));
        }
        Ok(match name {
            "listing.html" => Some(include_str!("../templates/listing.html").into()),
            "pagination.html" => Some(include_str!("../templates/pagination.html").into()),
            "404.html" => Some(include_str!("../templates/404.html").into()),
            _ => None,
        })
    }
}

impl SsrEngine {
    pub fn new(themes_dir: PathBuf, active_theme: String) -> Self {
        let mut env = Environment::new();
        let theme_templates = themes_dir.join(&active_theme).join("templates");

        if theme_templates.exists() {
            tracing::info!(path = ?theme_templates, "Moteur SSR : Chargement dynamique des templates depuis le thème");
            env.set_loader(theme_loader(theme_templates));
        } else {
            tracing::info!("Moteur SSR : Chargement des templates intégrés par défaut");
            env.add_template("base.html", include_str!("../templates/base.html"))
                .expect("template base.html invalide");
            env.add_template("home.html", include_str!("../templates/home.html"))
                .expect("template home.html invalide");
            env.add_template("single.html", include_str!("../templates/single.html"))
                .expect("template single.html invalide");
            for (name, source) in [
                ("listing.html", include_str!("../templates/listing.html")),
                (
                    "pagination.html",
                    include_str!("../templates/pagination.html"),
                ),
                ("404.html", include_str!("../templates/404.html")),
            ] {
                env.add_template(name, source)
                    .expect("template public invalide");
            }
            env.add_template("zen.html", include_str!("../templates/zen.html"))
                .expect("template zen.html invalide");
        }

        // Cache Moka : jusqu'à 5 000 pages HTML en mémoire, TTL 1 heure, invalidation automatique
        let cache = Cache::builder()
            .max_capacity(5_000)
            .time_to_live(Duration::from_secs(3600))
            .build();

        Self {
            env: Arc::new(RwLock::new(env)),
            revision: Arc::new(AtomicI64::new(-1)),
            visibility_epoch: Arc::new(AtomicI64::new(-1)),
            reload: Arc::new(tokio::sync::Mutex::new(())),
            cache,
            themes_dir,
            active_theme,
        }
    }

    /// Database triggers version all public dependencies, including scheduler changes.
    pub async fn synchronize(&self, db: &sqlx::PgPool) -> ApiResult<()> {
        let _guard = self.reload.lock().await;
        let (revision, theme): (i64, Option<String>) = sqlx::query_as("SELECT revision, (SELECT value #>> '{}' FROM settings WHERE key='site.active_theme') FROM runtime_revision WHERE singleton")
            .fetch_one(db).await.map_err(rustforge_core::CoreError::from)?;
        // Une échéance peut modifier la visibilité sans écriture SQL ni tick du scheduler.
        let epoch: i64 = sqlx::query_scalar("SELECT COALESCE((extract(epoch FROM max(deadline)) * 1000000)::bigint, 0) FROM entries e CROSS JOIN LATERAL (VALUES (e.published_at), (e.scheduled_unpublish_at)) AS dates(deadline) WHERE e.status = 'published' AND deadline <= now()")
            .fetch_one(db).await.map_err(rustforge_core::CoreError::from)?;
        if self.visibility_epoch.swap(epoch, Ordering::SeqCst) != epoch {
            self.cache.invalidate_all();
        }
        if revision != self.revision.load(Ordering::SeqCst) {
            let theme = theme.as_deref().unwrap_or(&self.active_theme);
            rustforge_core::security::safe_slug(theme)?;
            if self.themes_dir.exists() {
                let path = rustforge_core::security::confined_path(&self.themes_dir, theme)?
                    .join("templates");
                if !path.is_dir() {
                    return Err(crate::error::ApiError::BadRequest(
                        "Thème actif indisponible".into(),
                    ));
                }
                let mut env = self
                    .env
                    .write()
                    .map_err(|_| ApiError::Internal(anyhow::anyhow!("verrou de rendu")))?;
                env.clear_templates();
                env.set_loader(theme_loader(path));
            }
            self.cache.invalidate_all();
            self.revision.store(revision, Ordering::SeqCst);
        }
        Ok(())
    }

    /// Récupère le HTML depuis le cache L1 ou effectue le rendu via MiniJinja.
    pub async fn render_page<F, Fut>(
        &self,
        locale: &str,
        path: &str,
        fetch_data: F,
    ) -> ApiResult<String>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = ApiResult<Value>>,
    {
        let key = CacheKey {
            revision: self.revision.load(Ordering::SeqCst),
            visibility_epoch: self.visibility_epoch.load(Ordering::SeqCst),
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

        let template_name = ctx
            .get_attr("template")
            .ok()
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_else(|| "single.html".into());

        let html = {
            let env = self
                .env
                .read()
                .map_err(|_| ApiError::Internal(anyhow::anyhow!("verrou de rendu")))?;
            let tmpl = env
                .get_template(&template_name)
                .map_err(|e| ApiError::Internal(anyhow::anyhow!("erreur template : {e}")))?;

            tmpl.render(ctx)
                .map_err(|e| ApiError::Internal(anyhow::anyhow!("erreur rendu SSR : {e}")))?
        };

        // 3. Stockage en cache mémoire
        self.cache.insert(key, html.clone()).await;

        Ok(html)
    }

    /// Invalide une page précise (ex: après mise à jour).
    pub async fn invalidate(&self, locale: &str, path: &str) {
        let key = CacheKey {
            revision: self.revision.load(Ordering::SeqCst),
            visibility_epoch: self.visibility_epoch.load(Ordering::SeqCst),
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
