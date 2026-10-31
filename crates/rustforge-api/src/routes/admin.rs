//! Navigation déclarative. Les menus ne remplacent jamais les droits des routes métier.
use crate::{
    auth::AuthUser,
    error::{ApiError, ApiResult},
    state::AppState,
};
use axum::{Json, extract::State};
use rustforge_core::{models::Role, settings};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::OnceLock,
};

pub use rustforge_core::module_catalog::MODULE_IDS as SUPPORTED_MODULES;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MenuItem {
    pub id: String,
    pub parent: Option<String>,
    pub label: String,
    pub label_key: String,
    pub route: Option<String>,
    pub module: Option<String>,
    pub roles: Vec<String>,
    pub state: String,
    pub design: String,
    pub features: Vec<String>,
    pub tasks: Vec<String>,
    pub source: String,
    #[serde(default)]
    pub planning: bool,
    #[serde(default)]
    pub separator_before: bool,
}

#[derive(Serialize)]
pub struct Navigation {
    pub schema_version: u8,
    pub items: Vec<MenuItem>,
    pub catalog: Vec<MenuItem>,
    /// Admin-only complete outline; plan routes never enable business endpoints.
    pub menu_items: Vec<MenuItem>,
    pub modules: BTreeMap<String, bool>,
    pub supported_modules: Vec<&'static str>,
}

fn catalog() -> &'static Vec<MenuItem> {
    static ITEMS: OnceLock<Vec<MenuItem>> = OnceLock::new();
    ITEMS.get_or_init(|| {
        serde_json::from_str(include_str!("../admin_navigation.json"))
            .expect("registre de navigation intégré invalide")
    })
}

fn project(role: Role, modules: BTreeMap<String, bool>) -> Navigation {
    let role_name = match role {
        Role::Admin => "admin",
        Role::Editor => "editor",
        Role::Author => "author",
        Role::Subscriber => "subscriber",
    };
    let mut items: Vec<_> = catalog()
        .iter()
        .filter(|item| {
            item.state != "planned"
                && item.roles.iter().any(|r| r == role_name)
                && item
                    .module
                    .as_ref()
                    .is_none_or(|key| modules.get(key) == Some(&true))
        })
        .cloned()
        .collect();
    // Fermer aussi les descendants lorsque leur parent a été retiré.
    loop {
        let ids: BTreeSet<_> = items.iter().map(|n| n.id.clone()).collect();
        let before = items.len();
        items.retain(|n| n.parent.as_ref().is_none_or(|id| ids.contains(id)));
        if before == items.len() {
            break;
        }
    }
    let menu_items = if role.is_admin() {
        catalog()
            .iter()
            .filter(|item| item.roles.iter().any(|r| r == role_name))
            .cloned()
            .map(|mut item| {
                let inactive = item
                    .module
                    .as_ref()
                    .is_some_and(|key| modules.get(key) != Some(&true));
                if item.state == "planned" || inactive || item.route.is_none() {
                    item.route = Some(format!("plan:{}", item.id));
                    item.planning = true;
                }
                item
            })
            .collect()
    } else {
        items.clone()
    };
    Navigation {
        schema_version: 1,
        items,
        menu_items,
        modules,
        catalog: if role.is_admin() {
            catalog().clone()
        } else {
            vec![]
        },
        supported_modules: if role.is_admin() {
            SUPPORTED_MODULES.to_vec()
        } else {
            vec![]
        },
    }
}

pub async fn navigation(
    State(state): State<AppState>,
    auth: AuthUser,
) -> ApiResult<Json<Navigation>> {
    Ok(Json(project(
        auth.user.role,
        settings::get_active_modules(&state.db).await?,
    )))
}

pub async fn update_modules(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(patch): Json<BTreeMap<String, bool>>,
) -> ApiResult<Json<Navigation>> {
    if !auth.user.role.is_admin() {
        return Err(ApiError::Forbidden);
    }
    if patch
        .keys()
        .any(|key| !SUPPORTED_MODULES.contains(&key.as_str()))
    {
        return Err(ApiError::BadRequest("Module natif non disponible".into()));
    }
    let current = settings::get_active_modules(&state.db).await?;
    if !patch.is_empty() {
        let mut initial = current.clone();
        initial.extend(patch.clone());
        // Fusion atomique: une activation ne réécrit pas les autres choix concurrents.
        sqlx::query("INSERT INTO settings(key,value) VALUES ('modules.active',$1) ON CONFLICT(key) DO UPDATE SET value = (CASE WHEN jsonb_typeof(settings.value)='object' THEN settings.value ELSE $2 END) || $3, updated_at=now()")
            .bind(serde_json::json!(initial)).bind(serde_json::json!(current)).bind(serde_json::json!(patch))
            .execute(&state.db).await.map_err(rustforge_core::CoreError::from)?;
    }
    navigation(State(state), auth).await
}

/// Préférences de colonnes partagées, sans exposer les autres réglages du site.
pub async fn columns(
    State(state): State<AppState>,
    auth: AuthUser,
) -> ApiResult<Json<BTreeMap<String, serde_json::Value>>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    let rows: Vec<(String, serde_json::Value)> =
        sqlx::query_as("SELECT key,value FROM settings WHERE key LIKE 'admin.columns.%'")
            .fetch_all(&state.db)
            .await
            .map_err(rustforge_core::CoreError::from)?;
    Ok(Json(rows.into_iter().collect()))
}

pub async fn module_catalog(
    auth: AuthUser,
) -> ApiResult<Json<&'static [rustforge_core::module_catalog::Manifest]>> {
    if !auth.user.role.is_admin() {
        return Err(ApiError::Forbidden);
    }
    Ok(Json(rustforge_core::module_catalog::MANIFESTS))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn module_lifecycle_and_roles_filter_navigation() {
        let on = BTreeMap::from([("lms".into(), true)]);
        assert!(
            project(Role::Admin, on.clone())
                .items
                .iter()
                .any(|n| n.id == "rf_lms")
        );
        assert!(
            !project(Role::Admin, BTreeMap::new())
                .items
                .iter()
                .any(|n| n.module.as_deref() == Some("lms"))
        );
        assert!(
            !project(Role::Author, on)
                .items
                .iter()
                .any(|n| n.id == "rf_lms" || n.id == "rf_settings")
        );
        let reader = project(Role::Subscriber, BTreeMap::new());
        assert!(reader.catalog.is_empty());
        assert!(!reader.items.iter().any(|n| n.id == "rf_posts"));
    }
    #[test]
    fn outline_is_admin_only_and_does_not_activate_disabled_modules() {
        let admin = project(Role::Admin, BTreeMap::new());
        assert_eq!(admin.menu_items.len(), catalog().len());
        let gallery = admin
            .menu_items
            .iter()
            .find(|n| n.id == "rf_galleries")
            .unwrap();
        assert!(gallery.planning);
        assert_eq!(gallery.route.as_deref(), Some("plan:rf_galleries"));
        assert!(
            !admin
                .items
                .iter()
                .any(|n| n.module.as_deref() == Some("galleries"))
        );
        assert!(
            admin
                .menu_items
                .iter()
                .any(|n| n.id == "rf_media_drive" && n.planning)
        );
        for role in [Role::Editor, Role::Author, Role::Subscriber] {
            let view = project(role, BTreeMap::new());
            assert_eq!(view.menu_items.len(), view.items.len());
            assert!(view.menu_items.iter().all(|n| !n.planning));
            assert!(view.catalog.is_empty());
        }
        let roots: Vec<_> = admin
            .menu_items
            .iter()
            .filter(|n| n.parent.is_none())
            .map(|n| n.id.as_str())
            .collect();
        assert_eq!(
            &roots[..8],
            &[
                "rf_dashboard",
                "rf_posts",
                "rf_pages",
                "rf_forms",
                "rf_events",
                "rf_media",
                "rf_galleries",
                "rf_comments"
            ]
        );
    }
    #[test]
    fn catalog_is_a_three_level_tree_with_real_routes_only_when_available() {
        let nodes = catalog();
        let ids: BTreeSet<_> = nodes.iter().map(|n| &n.id).collect();
        assert_eq!(ids.len(), nodes.len());
        for item in nodes {
            assert!(item.id.starts_with("rf_"));
            let mut depth = 1;
            let mut parent = item.parent.as_deref();
            while let Some(id) = parent {
                depth += 1;
                assert!(depth <= 3, "arbre cyclique ou trop profond");
                parent = nodes
                    .iter()
                    .find(|n| n.id == id)
                    .expect("parent absent")
                    .parent
                    .as_deref();
            }
            if item.state == "planned" {
                assert!(item.route.is_none());
            }
        }
    }
}
