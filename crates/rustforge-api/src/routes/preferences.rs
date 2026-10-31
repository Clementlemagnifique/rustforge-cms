//! Account-scoped dashboard preferences; never accepts another account identifier.
use crate::{
    auth::AuthUser,
    error::{ApiError, ApiResult},
    state::AppState,
};
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

const WIDGETS: [&str; 4] = ["overview", "activity", "health", "draft"];
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Preferences {
    pub version: u8,
    pub dashboard: Layout,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Layout {
    pub order: Vec<String>,
    pub hidden: Vec<String>,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            version: 1,
            dashboard: Layout {
                order: WIDGETS.map(str::to_owned).to_vec(),
                hidden: vec![],
            },
        }
    }
}
fn valid(value: &Preferences) -> bool {
    let order = &value.dashboard.order;
    let hidden = &value.dashboard.hidden;
    value.version == 1
        && order.len() == WIDGETS.len()
        && WIDGETS
            .iter()
            .all(|id| order.iter().filter(|item| item.as_str() == *id).count() == 1)
        && hidden.len() <= WIDGETS.len()
        && hidden.iter().all(|id| {
            WIDGETS.contains(&id.as_str())
                && hidden.iter().filter(|other| *other == id).count() == 1
        })
}
fn key(auth: &AuthUser) -> String {
    format!("admin.user_{}.preferences", auth.user.id.simple())
}
pub async fn get(State(state): State<AppState>, auth: AuthUser) -> ApiResult<Json<Preferences>> {
    let value = rustforge_core::settings::get(&state.db, &key(&auth))
        .await?
        .and_then(|value| serde_json::from_value::<Preferences>(value).ok())
        .filter(valid)
        .unwrap_or_default();
    Ok(Json(value))
}
pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(value): Json<Preferences>,
) -> ApiResult<Json<Preferences>> {
    if !valid(&value) {
        return Err(ApiError::BadRequest(
            "Organisation du tableau de bord invalide.".into(),
        ));
    }
    let values = [(
        key(&auth),
        serde_json::to_value(&value).map_err(anyhow::Error::from)?,
    )]
    .into_iter()
    .collect();
    rustforge_core::settings::update(&state.db, values).await?;
    Ok(Json(value))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preferences_cannot_smuggle_permissions_or_unknown_widgets() {
        let mut value = Preferences::default();
        assert!(valid(&value));
        value.dashboard.order[1] = "overview".into();
        assert!(!valid(&value));
        value = Preferences::default();
        value.dashboard.hidden.push("unknown".into());
        assert!(!valid(&value));
        assert!(
            serde_json::from_value::<Preferences>(
                serde_json::json!({"version":1,"dashboard":{"order":[],"hidden":[]},"role":"admin"})
            )
            .is_err()
        );
    }
}
