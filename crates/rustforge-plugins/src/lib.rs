//! # RustForge Plugins
//!
//! Runtime de plugins WebAssembly basé sur Wasmtime et le *component model*.
//!
//! - Chaque plugin est un composant `.wasm` respectant `wit/rustforge-plugin.wit`.
//! - Chaque appel s'exécute dans un **store neuf** : isolation totale entre appels,
//!   mémoire plafonnée et CPU limité par *fuel*.
//! - Aucun accès FS/réseau : le contexte WASI est vide. Les capacités
//!   supplémentaires (stockage…) sont accordées explicitement par l'administrateur.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
};

use anyhow::{Context, Result};
use async_trait::async_trait;
use rustforge_core::{HookRunner, hooks::names};
use serde::Serialize;
use sqlx::PgPool;
use wasmtime::{
    Config, Engine, Store, StoreLimits, StoreLimitsBuilder,
    component::{Component, HasSelf, Linker, ResourceTable},
};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

mod bindings {
    wasmtime::component::bindgen!({
        world: "plugin",
        path: "../../wit",
    });
}

use bindings::{
    Plugin, PluginPre,
    rustforge::plugin::{host, types},
};

/// Mémoire linéaire maximale par instance.
const MAX_MEMORY: usize = 32 * 1024 * 1024;
/// Budget CPU par appel (instructions approximatives).
const FUEL_PER_CALL: u64 = 200_000_000;
/// Limites du stockage clé/valeur.
const KV_MAX_KEYS: usize = 1_000;
const KV_MAX_KEY_LEN: usize = 256;
const KV_MAX_VALUE_LEN: usize = 64 * 1024;

type Kv = Arc<Mutex<HashMap<String, String>>>;

// ---------------------------------------------------------------------------
// État hôte d'une instance
// ---------------------------------------------------------------------------

pub struct HostState {
    wasi: WasiCtx,
    table: ResourceTable,
    limits: StoreLimits,
    plugin_id: String,
    /// `None` si la capacité `storage` n'est pas accordée.
    kv: Option<Kv>,
    kv_dirty: bool,
}

impl WasiView for HostState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

impl types::Host for HostState {}

impl host::Host for HostState {
    fn log(&mut self, level: host::Level, message: String) {
        let id = self.plugin_id.as_str();
        match level {
            host::Level::Trace => tracing::trace!(plugin = id, "{message}"),
            host::Level::Debug => tracing::debug!(plugin = id, "{message}"),
            host::Level::Info => tracing::info!(plugin = id, "{message}"),
            host::Level::Warn => tracing::warn!(plugin = id, "{message}"),
            host::Level::Error => tracing::error!(plugin = id, "{message}"),
        }
    }

    fn kv_get(&mut self, key: String) -> Option<String> {
        self.kv.as_ref()?.lock().ok()?.get(&key).cloned()
    }

    fn kv_set(&mut self, key: String, value: String) -> Result<(), String> {
        let Some(kv) = &self.kv else {
            return Err("capacité « storage » non accordée".into());
        };
        if key.is_empty() || key.len() > KV_MAX_KEY_LEN {
            return Err(format!("clé invalide (1 à {KV_MAX_KEY_LEN} octets)"));
        }
        if value.len() > KV_MAX_VALUE_LEN {
            return Err(format!(
                "valeur trop grande (max {KV_MAX_VALUE_LEN} octets)"
            ));
        }
        let mut map = kv.lock().map_err(|_| "stockage indisponible".to_string())?;
        if !map.contains_key(&key) && map.len() >= KV_MAX_KEYS {
            return Err(format!("quota atteint ({KV_MAX_KEYS} clés)"));
        }
        map.insert(key, value);
        self.kv_dirty = true;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Description publique d'un plugin
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub hooks: Vec<String>,
    pub capabilities: Vec<String>,
    pub granted: Vec<String>,
    pub enabled: bool,
    pub file: String,
    /// Erreur de chargement éventuelle (plugin inutilisable).
    pub error: Option<String>,
}

fn capability_name(c: types::Capability) -> &'static str {
    match c {
        types::Capability::Storage => "storage",
        types::Capability::HttpOutbound => "http-outbound",
        types::Capability::PublicRoutes => "public-routes",
        types::Capability::EditorFields => "editor-fields",
    }
}

fn valid_plugin_id(id: &str) -> bool {
    (2..=64).contains(&id.len())
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !id.starts_with('-')
        && !id.ends_with('-')
}

struct LoadedPlugin {
    info: Mutex<PluginInfo>,
    pre: Option<PluginPre<HostState>>,
    kv: Kv,
}

impl LoadedPlugin {
    fn snapshot(&self) -> PluginInfo {
        self.info.lock().expect("plugin info lock").clone()
    }
}

// ---------------------------------------------------------------------------
// Runtime
// ---------------------------------------------------------------------------

struct Runtime {
    engine: Engine,
    linker: Linker<HostState>,
}

impl Runtime {
    fn new() -> Result<Self> {
        let mut config = Config::new();
        config.consume_fuel(true);
        config.wasm_component_model(true);
        let engine = Engine::new(&config)?;

        let mut linker = Linker::<HostState>::new(&engine);
        wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;
        Plugin::add_to_linker::<HostState, HasSelf<HostState>>(&mut linker, |state| state)?;
        Ok(Self { engine, linker })
    }

    fn new_store(&self, plugin_id: &str, kv: Option<Kv>) -> Result<Store<HostState>> {
        let state = HostState {
            wasi: WasiCtxBuilder::new().build(),
            table: ResourceTable::new(),
            limits: StoreLimitsBuilder::new()
                .memory_size(MAX_MEMORY)
                .instances(16)
                .build(),
            plugin_id: plugin_id.to_string(),
            kv,
            kv_dirty: false,
        };
        let mut store = Store::new(&self.engine, state);
        store.limiter(|s| &mut s.limits);
        store.set_fuel(FUEL_PER_CALL)?;
        Ok(store)
    }

    /// Compile un fichier et lit son manifeste (bloquant).
    fn load(&self, path: &Path) -> Result<(PluginPre<HostState>, types::Manifest)> {
        let component = Component::from_file(&self.engine, path)
            .map_err(|e| anyhow::anyhow!("compilation de {}: {e}", path.display()))?;
        let instance_pre = self
            .linker
            .instantiate_pre(&component)
            .map_err(|e| anyhow::anyhow!("liaison des imports: {e}"))?;
        let pre = PluginPre::new(instance_pre).map_err(|e| anyhow::anyhow!("PluginPre: {e}"))?;
        let mut store = self.new_store("<chargement>", None)?;
        let instance = pre
            .instantiate(&mut store)
            .map_err(|e| anyhow::anyhow!("instanciation: {e}"))?;
        let manifest = instance
            .call_manifest(&mut store)
            .map_err(|e| anyhow::anyhow!("appel de manifest(): {e}"))?;
        Ok((pre, manifest))
    }
}

/// Résultat d'un appel sur une instance fraîche.
fn invoke<R>(
    rt: &Runtime,
    plugin: &LoadedPlugin,
    f: impl FnOnce(&Plugin, &mut Store<HostState>) -> wasmtime::Result<R>,
) -> Result<(R, bool)> {
    let info = plugin.snapshot();
    let pre = plugin.pre.as_ref().context("plugin non chargé")?;
    let kv = info
        .granted
        .iter()
        .any(|c| c == "storage")
        .then(|| plugin.kv.clone());
    let mut store = rt.new_store(&info.id, kv)?;
    let instance = pre
        .instantiate(&mut store)
        .map_err(|e| anyhow::anyhow!("instanciation: {e}"))?;
    let out = f(&instance, &mut store).map_err(|e| anyhow::anyhow!("appel: {e}"))?;
    Ok((out, store.data().kv_dirty))
}

/// Gestionnaire des plugins : découverte, état, exécution des hooks.
pub struct PluginManager {
    rt: Arc<Runtime>,
    db: PgPool,
    dir: PathBuf,
    plugins: RwLock<Vec<Arc<LoadedPlugin>>>,
}

impl PluginManager {
    /// Crée le gestionnaire et charge les plugins présents dans `dir`.
    pub async fn new(db: PgPool, dir: impl Into<PathBuf>) -> Result<Arc<Self>> {
        let manager = Arc::new(Self {
            rt: Arc::new(Runtime::new()?),
            db,
            dir: dir.into(),
            plugins: RwLock::new(Vec::new()),
        });
        manager.rescan().await?;
        Ok(manager)
    }

    pub fn directory(&self) -> &Path {
        &self.dir
    }

    /// (Re)découvre les fichiers `.wasm` du répertoire des plugins.
    pub async fn rescan(&self) -> Result<Vec<PluginInfo>> {
        tokio::fs::create_dir_all(&self.dir).await?;
        let mut files = Vec::new();
        let mut dir = tokio::fs::read_dir(&self.dir).await?;
        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "wasm") {
                files.push(path);
            }
        }
        files.sort();

        let mut loaded = Vec::new();
        for path in files {
            let file = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let rt = self.rt.clone();
            let p = path.clone();
            let result = tokio::task::spawn_blocking(move || rt.load(&p)).await?;

            let plugin = match result {
                Ok((pre, manifest)) => match self.register(&file, manifest).await {
                    Ok((info, kv)) => LoadedPlugin {
                        info: Mutex::new(info),
                        pre: Some(pre),
                        kv: Arc::new(Mutex::new(kv)),
                    },
                    Err(err) => broken(&file, err),
                },
                Err(err) => broken(&file, err),
            };
            let info = plugin.snapshot();
            match &info.error {
                Some(err) => tracing::error!(file = %info.file, "plugin invalide : {err}"),
                None => tracing::info!(
                    plugin = %info.id, version = %info.version, enabled = info.enabled,
                    "plugin chargé"
                ),
            }
            loaded.push(Arc::new(plugin));
        }

        *self.plugins.write().expect("plugins lock") = loaded;
        Ok(self.list())
    }

    /// Enregistre le manifeste en base et renvoie l'état persistant.
    async fn register(
        &self,
        file: &str,
        m: types::Manifest,
    ) -> Result<(PluginInfo, HashMap<String, String>)> {
        anyhow::ensure!(
            valid_plugin_id(&m.id),
            "identifiant de plugin invalide « {} »",
            m.id
        );
        if self.list().iter().any(|p| p.id == m.id) {
            anyhow::bail!("identifiant « {} » déjà utilisé par un autre fichier", m.id);
        }
        for hook in &m.hooks {
            if !names::ALL.contains(&hook.as_str()) {
                tracing::warn!(plugin = %m.id, hook, "hook inconnu");
            }
        }
        let capabilities: Vec<String> = m
            .capabilities
            .iter()
            .map(|c| capability_name(*c).to_string())
            .collect();

        let (enabled, granted, kv): (bool, serde_json::Value, serde_json::Value) = sqlx::query_as(
            "INSERT INTO plugins (id, name, version, description, hooks, capabilities)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name, version = EXCLUDED.version,
                description = EXCLUDED.description, hooks = EXCLUDED.hooks,
                capabilities = EXCLUDED.capabilities, updated_at = now()
             RETURNING enabled, granted, kv",
        )
        .bind(&m.id)
        .bind(&m.name)
        .bind(&m.version)
        .bind(&m.description)
        .bind(serde_json::json!(m.hooks))
        .bind(serde_json::json!(capabilities))
        .fetch_one(&self.db)
        .await?;

        // Une capacité n'est accordée que si elle est toujours demandée.
        let granted: Vec<String> = serde_json::from_value::<Vec<String>>(granted)
            .unwrap_or_default()
            .into_iter()
            .filter(|c| capabilities.contains(c))
            .collect();
        let kv: HashMap<String, String> = serde_json::from_value(kv).unwrap_or_default();

        Ok((
            PluginInfo {
                id: m.id,
                name: m.name,
                version: m.version,
                description: m.description,
                hooks: m.hooks,
                capabilities,
                granted,
                enabled,
                file: file.to_string(),
                error: None,
            },
            kv,
        ))
    }

    pub fn list(&self) -> Vec<PluginInfo> {
        self.plugins
            .read()
            .expect("plugins lock")
            .iter()
            .map(|p| p.snapshot())
            .collect()
    }

    fn find(&self, id: &str) -> Option<Arc<LoadedPlugin>> {
        self.plugins
            .read()
            .expect("plugins lock")
            .iter()
            .find(|p| p.snapshot().id == id)
            .cloned()
    }

    /// Active un plugin en lui accordant les capacités demandées.
    pub async fn enable(&self, id: &str) -> Result<Option<PluginInfo>> {
        let Some(plugin) = self.find(id) else {
            return Ok(None);
        };
        let info = plugin.snapshot();
        anyhow::ensure!(info.error.is_none(), "ce plugin n'a pas pu être chargé");
        sqlx::query(
            "UPDATE plugins SET enabled = true, granted = $2, updated_at = now() WHERE id = $1",
        )
        .bind(id)
        .bind(serde_json::json!(info.capabilities))
        .execute(&self.db)
        .await?;
        let mut guard = plugin.info.lock().expect("plugin info lock");
        guard.enabled = true;
        guard.granted = guard.capabilities.clone();
        Ok(Some(guard.clone()))
    }

    pub async fn disable(&self, id: &str) -> Result<Option<PluginInfo>> {
        let Some(plugin) = self.find(id) else {
            return Ok(None);
        };
        sqlx::query("UPDATE plugins SET enabled = false, updated_at = now() WHERE id = $1")
            .bind(id)
            .execute(&self.db)
            .await?;
        let mut guard = plugin.info.lock().expect("plugin info lock");
        guard.enabled = false;
        Ok(Some(guard.clone()))
    }

    /// Plugins actifs abonnés à `hook`, dans l'ordre alphabétique des fichiers.
    fn subscribers(&self, hook: &str) -> Vec<Arc<LoadedPlugin>> {
        self.plugins
            .read()
            .expect("plugins lock")
            .iter()
            .filter(|p| {
                let info = p.info.lock().expect("plugin info lock");
                info.enabled && info.error.is_none() && info.hooks.iter().any(|h| h == hook)
            })
            .cloned()
            .collect()
    }

    async fn persist_kv(&self, plugin: &LoadedPlugin) {
        let id = plugin.snapshot().id;
        let value = match plugin.kv.lock() {
            Ok(map) => serde_json::json!(*map),
            Err(_) => return,
        };
        if let Err(err) = sqlx::query("UPDATE plugins SET kv = $2 WHERE id = $1")
            .bind(&id)
            .bind(value)
            .execute(&self.db)
            .await
        {
            tracing::error!(plugin = %id, %err, "échec de la sauvegarde du stockage");
        }
    }
}

fn broken(file: &str, err: anyhow::Error) -> LoadedPlugin {
    let id = file.trim_end_matches(".wasm").to_string();
    LoadedPlugin {
        info: Mutex::new(PluginInfo {
            name: id.clone(),
            id,
            version: String::new(),
            description: String::new(),
            hooks: vec![],
            capabilities: vec![],
            granted: vec![],
            enabled: false,
            file: file.to_string(),
            error: Some(format!("{err:#}")),
        }),
        pre: None,
        kv: Default::default(),
    }
}

#[async_trait]
impl HookRunner for PluginManager {
    async fn filter(&self, hook: &str, mut payload: String) -> String {
        for plugin in self.subscribers(hook) {
            let rt = self.rt.clone();
            let p = plugin.clone();
            let h = hook.to_string();
            let input = payload.clone();
            let result = tokio::task::spawn_blocking(move || {
                invoke(&rt, &p, |instance, store| {
                    instance.call_filter(store, &h, &input)
                })
            })
            .await;

            let id = plugin.snapshot().id;
            match result {
                Ok(Ok((Ok(output), dirty))) => {
                    payload = output;
                    if dirty {
                        self.persist_kv(&plugin).await;
                    }
                }
                Ok(Ok((Err(msg), _))) => {
                    tracing::warn!(plugin = %id, hook, "filtre refusé : {msg}")
                }
                Ok(Err(err)) => tracing::error!(plugin = %id, hook, "échec du plugin : {err:#}"),
                Err(err) => tracing::error!(plugin = %id, hook, "tâche interrompue : {err}"),
            }
        }
        payload
    }

    async fn action(&self, hook: &str, payload: String) {
        for plugin in self.subscribers(hook) {
            let rt = self.rt.clone();
            let p = plugin.clone();
            let h = hook.to_string();
            let input = payload.clone();
            let result = tokio::task::spawn_blocking(move || {
                invoke(&rt, &p, |instance, store| {
                    instance.call_on_action(store, &h, &input)
                })
            })
            .await;

            let id = plugin.snapshot().id;
            match result {
                Ok(Ok(((), dirty))) => {
                    if dirty {
                        self.persist_kv(&plugin).await;
                    }
                }
                Ok(Err(err)) => tracing::error!(plugin = %id, hook, "échec du plugin : {err:#}"),
                Err(err) => tracing::error!(plugin = %id, hook, "tâche interrompue : {err}"),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_ids() {
        assert!(valid_plugin_id("reading-time"));
        assert!(!valid_plugin_id("Reading_Time"));
        assert!(!valid_plugin_id("-x"));
    }

    #[test]
    fn runtime_builds() {
        Runtime::new().expect("le runtime doit se construire");
    }
}
