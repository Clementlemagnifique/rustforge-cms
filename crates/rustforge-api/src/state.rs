use crate::ssr::SsrEngine;
use rustforge_plugins::PluginManager;
use sqlx::PgPool;
use std::{path::PathBuf, sync::Arc};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub plugins: Arc<PluginManager>,
    pub jwt_secret: String,
    pub setup_token: Option<String>,
    pub uploads_dir: PathBuf,
    pub protected_downloads_dir: PathBuf,
    pub admin_dist_dir: Option<PathBuf>,
    pub themes_dir: PathBuf,
    pub active_theme: String,
    pub ssr: Arc<SsrEngine>,
}

impl AppState {
    pub fn storage_paths(&self) -> rustforge_core::migration::capsule::StoragePaths {
        rustforge_core::migration::capsule::StoragePaths {
            uploads: self.uploads_dir.clone(),
            themes: self.themes_dir.clone(),
            plugins: self.plugins.directory().into(),
            protected: self.protected_downloads_dir.clone(),
        }
    }
    pub fn new(
        db: PgPool,
        plugins: Arc<PluginManager>,
        jwt_secret: String,
        uploads_dir: PathBuf,
        admin_dist_dir: Option<PathBuf>,
        themes_dir: PathBuf,
        active_theme: String,
    ) -> Self {
        let ssr = Arc::new(SsrEngine::new(themes_dir.clone(), active_theme.clone()));
        Self {
            db,
            plugins,
            jwt_secret,
            setup_token: None,
            uploads_dir,
            protected_downloads_dir: std::env::var("PROTECTED_DOWNLOADS_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("protected_downloads")),
            admin_dist_dir,
            themes_dir,
            active_theme,
            ssr,
        }
    }
}
