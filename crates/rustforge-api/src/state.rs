use std::{path::PathBuf, sync::Arc};
use sqlx::PgPool;
use rustforge_plugins::PluginManager;
use crate::ssr::SsrEngine;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub plugins: Arc<PluginManager>,
    pub jwt_secret: String,
    pub uploads_dir: PathBuf,
    pub admin_dist_dir: Option<PathBuf>,
    pub themes_dir: PathBuf,
    pub active_theme: String,
    pub ssr: Arc<SsrEngine>,
}

impl AppState {
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
            uploads_dir,
            admin_dist_dir,
            themes_dir,
            active_theme,
            ssr,
        }
    }
}
