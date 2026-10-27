//! Système de hooks : l'équivalent typé de `add_filter` / `add_action` de WordPress.
//!
//! Le cœur déclenche des hooks nommés ; l'implémentation (plugins WASM) est
//! injectée via le trait [`HookRunner`], ce qui garde `rustforge-core`
//! indépendant du runtime de plugins.

use async_trait::async_trait;

/// Hooks déclenchés par le cœur.
pub mod names {
    /// Filtre — HTML rendu d'une entrée avant diffusion publique. Payload : HTML.
    pub const CONTENT_RENDER: &str = "content.render";
    /// Filtre — données d'une entrée avant enregistrement. Payload : JSON `EntryInput`.
    pub const ENTRY_BEFORE_SAVE: &str = "entry.before_save";
    /// Action — une entrée vient d'être publiée. Payload : JSON `{id, content_type, title, slug}`.
    pub const ENTRY_PUBLISHED: &str = "entry.published";
    /// Action — une entrée a été supprimée définitivement. Payload : JSON `{id}`.
    pub const ENTRY_DELETED: &str = "entry.deleted";

    pub const ALL: &[&str] = &[CONTENT_RENDER, ENTRY_BEFORE_SAVE, ENTRY_PUBLISHED, ENTRY_DELETED];
}

#[async_trait]
pub trait HookRunner: Send + Sync {
    /// Applique successivement les filtres enregistrés pour `hook`.
    /// Un filtre en erreur est ignoré (journalisé) : il ne doit jamais casser le site.
    async fn filter(&self, hook: &str, payload: String) -> String;

    /// Notifie les abonnés de `hook`.
    async fn action(&self, hook: &str, payload: String);
}

/// Implémentation vide (tests, ou plugins désactivés).
pub struct NoopHooks;

#[async_trait]
impl HookRunner for NoopHooks {
    async fn filter(&self, _hook: &str, payload: String) -> String {
        payload
    }

    async fn action(&self, _hook: &str, _payload: String) {}
}
