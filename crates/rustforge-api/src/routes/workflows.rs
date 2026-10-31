use crate::{
    auth::AuthUser,
    error::{ApiError, ApiResult},
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, State},
};
use rustforge_core::{
    mail,
    workflows::{self, Campaign, CampaignInput, Recipe, RecipeInput, Run},
};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;
fn admin(auth: &AuthUser) -> ApiResult<()> {
    if !auth.user.role.is_admin() {
        return Err(ApiError::Forbidden);
    }
    Ok(())
}
pub async fn campaigns(
    State(state): State<AppState>,
    auth: AuthUser,
) -> ApiResult<Json<Vec<Campaign>>> {
    admin(&auth)?;
    Ok(Json(workflows::campaigns(&state.db).await?))
}
pub async fn create_campaign(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(input): Json<CampaignInput>,
) -> ApiResult<Json<Campaign>> {
    admin(&auth)?;
    Ok(Json(
        workflows::save_campaign(&state.db, None, auth.user.id, input).await?,
    ))
}
pub async fn update_campaign(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<CampaignInput>,
) -> ApiResult<Json<Campaign>> {
    admin(&auth)?;
    Ok(Json(
        workflows::save_campaign(&state.db, Some(id), auth.user.id, input).await?,
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Queue {
    pub expected_version: i64,
}
pub async fn queue_campaign(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<Queue>,
) -> ApiResult<Json<Campaign>> {
    admin(&auth)?;
    Ok(Json(
        workflows::queue_campaign(&state.db, id, input.expected_version).await?,
    ))
}
pub async fn recipes(
    State(state): State<AppState>,
    auth: AuthUser,
) -> ApiResult<Json<Vec<Recipe>>> {
    admin(&auth)?;
    Ok(Json(workflows::recipes(&state.db).await?))
}
pub async fn create_recipe(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(input): Json<RecipeInput>,
) -> ApiResult<Json<Recipe>> {
    admin(&auth)?;
    Ok(Json(
        workflows::save_recipe(&state.db, None, auth.user.id, input).await?,
    ))
}
pub async fn update_recipe(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<RecipeInput>,
) -> ApiResult<Json<Recipe>> {
    admin(&auth)?;
    Ok(Json(
        workflows::save_recipe(&state.db, Some(id), auth.user.id, input).await?,
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Invoke {
    pub invocation_id: Uuid,
    pub expected_version: i64,
}
pub async fn run_recipe(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<Invoke>,
) -> ApiResult<Json<Run>> {
    admin(&auth)?;
    Ok(Json(
        workflows::run_recipe(&state.db, id, input.invocation_id, input.expected_version).await?,
    ))
}
pub async fn runs(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Vec<Run>>> {
    admin(&auth)?;
    Ok(Json(workflows::runs(&state.db, id).await?))
}
pub async fn capture(State(state): State<AppState>, auth: AuthUser) -> ApiResult<Json<Value>> {
    admin(&auth)?;
    let count = mail::run_campaign_captures(&state.db, 10).await?;
    Ok(Json(json!({"handled":count,"transport":"capture"})))
}
