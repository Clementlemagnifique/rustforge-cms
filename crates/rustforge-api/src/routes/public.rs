use std::collections::BTreeMap;

use axum::{
    Json,
    extract::{Path, Query, State},
    response::{Html, IntoResponse, Redirect},
};
use minijinja::Value as MinijinjaValue;
use serde_json::Value;

use crate::{
    error::ApiResult,
    state::AppState,
};
use rustforge_core::{
    HookRunner, Page,
    entries::{self, EntryQuery},
    hooks::names,
    models::{Entry, Term},
    settings, terms,
};

// ---------------------------------------------------------------------------
// Helpers Rendu SSR & Multilingue
// ---------------------------------------------------------------------------

async fn build_site_context(state: &AppState, locale: &str) -> ApiResult<serde_json::Value> {
    let site_settings = settings::all(&state.db).await?;
    let site_title = site_settings.get("site.title")
        .and_then(|v| v.as_str())
        .unwrap_or("RustForge");
    let site_tagline = site_settings.get("site.tagline")
        .and_then(|v| v.as_str())
        .unwrap_or("Le CMS Rust IA-First");
    let design_tokens = site_settings.get("site.design_tokens")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({
            "primary_color": "#4f46e5",
            "accent_color": "#06b6d4",
            "radius": "12px"
        }));

    Ok(serde_json::json!({
        "locale": locale,
        "site_title": site_title,
        "site_tagline": site_tagline,
        "design_tokens": design_tokens,
        "available_locales": [
            { "code": "fr", "url": "/fr" },
            { "code": "en", "url": "/en" }
        ]
    }))
}

// ---------------------------------------------------------------------------
// Rendu SSR Pages Publiques (< 1ms via cache Moka L1)
// ---------------------------------------------------------------------------

/// Redirection de la racine `/` vers la locale par défaut `/fr`
pub async fn root_redirect() -> impl IntoResponse {
    Redirect::temporary("/fr")
}

/// Page d'accueil publique multilingue
pub async fn ssr_home(
    State(state): State<AppState>,
    Path(locale): Path<String>,
) -> ApiResult<Html<String>> {
    let path = format!("/{locale}");
    let db = state.db.clone();
    let state_ref = state.clone();
    let loc = locale.clone();

    let html = state.ssr.render_page(&locale, &path, move || async move {
        let mut base_ctx = build_site_context(&state_ref, &loc).await?;
        let query = EntryQuery {
            public_only: true,
            per_page: Some(12),
            ..Default::default()
        };
        let page = entries::list(&db, &query).await?;
        base_ctx["template"] = serde_json::json!("home.html");
        base_ctx["entries"] = serde_json::to_value(&page.items).unwrap_or_default();
        
        Ok(MinijinjaValue::from_serialize(&base_ctx))
    }).await?;

    Ok(Html(html))
}

/// Page d'un article ou contenu unique multilingue
pub async fn ssr_single(
    State(state): State<AppState>,
    Path((locale, content_type, slug)): Path<(String, String, String)>,
) -> ApiResult<Html<String>> {
    let path = format!("/{locale}/{content_type}/{slug}");
    let db = state.db.clone();
    let plugins = state.plugins.clone();
    let state_ref = state.clone();
    let loc = locale.clone();
    let ct = content_type.clone();
    let sl = slug.clone();

    let html = state.ssr.render_page(&locale, &path, move || async move {
        let mut base_ctx = build_site_context(&state_ref, &loc).await?;
        let mut entry = entries::get_published(&db, &ct, &sl).await?;
        
        // Exécution des filtres de plugins WASM sur le contenu
        entry.body_html = plugins.filter(names::CONTENT_RENDER, entry.body_html).await;

        base_ctx["template"] = serde_json::json!("single.html");
        base_ctx["entry"] = serde_json::to_value(&entry).unwrap_or_default();
        base_ctx["schema_org"] = serde_json::json!({
            "@context": "https://schema.org",
            "@type": if entry.content_type == "post" { "BlogPosting" } else { "WebPage" },
            "headline": entry.title,
            "description": entry.excerpt,
            "datePublished": entry.published_at,
            "author": { "@type": "Person", "name": entry.author_name.unwrap_or_else(|| "RustForge".into()) }
        });

        Ok(MinijinjaValue::from_serialize(&base_ctx))
    }).await?;

    Ok(Html(html))
}

/// Page en Mode Zen sans distraction
pub async fn ssr_zen(
    State(state): State<AppState>,
    Path((locale, content_type, slug)): Path<(String, String, String)>,
) -> ApiResult<Html<String>> {
    let path = format!("/{locale}/{content_type}/{slug}/zen");
    let db = state.db.clone();
    let plugins = state.plugins.clone();
    let state_ref = state.clone();
    let loc = locale.clone();
    let ct = content_type.clone();
    let sl = slug.clone();

    let html = state.ssr.render_page(&locale, &path, move || async move {
        let mut base_ctx = build_site_context(&state_ref, &loc).await?;
        let mut entry = entries::get_published(&db, &ct, &sl).await?;
        entry.body_html = plugins.filter(names::CONTENT_RENDER, entry.body_html).await;

        base_ctx["template"] = serde_json::json!("zen.html");
        base_ctx["entry"] = serde_json::to_value(&entry).unwrap_or_default();

        Ok(MinijinjaValue::from_serialize(&base_ctx))
    }).await?;

    Ok(Html(html))
}

// ---------------------------------------------------------------------------
// API Headless JSON (pour clients externes : React, Astro, Next.js)
// ---------------------------------------------------------------------------

pub async fn list_entries(
    State(state): State<AppState>,
    Query(mut query): Query<EntryQuery>,
) -> ApiResult<Json<Page<Entry>>> {
    query.public_only = true;
    let page = entries::list(&state.db, &query).await?;
    Ok(Json(page))
}

pub async fn get_entry(
    State(state): State<AppState>,
    Path((content_type, slug)): Path<(String, String)>,
) -> ApiResult<Json<Entry>> {
    let mut entry = entries::get_published(&state.db, &content_type, &slug).await?;
    entry.body_html = state.plugins.filter(names::CONTENT_RENDER, entry.body_html).await;
    Ok(Json(entry))
}

pub async fn get_settings(
    State(state): State<AppState>,
) -> ApiResult<Json<BTreeMap<String, Value>>> {
    let map = settings::public(&state.db).await?;
    Ok(Json(map))
}

pub async fn list_terms(
    State(state): State<AppState>,
    Query(query): Query<crate::routes::taxonomies::TermListQuery>,
) -> ApiResult<Json<Vec<Term>>> {
    let list = terms::list_terms(&state.db, query.taxonomy.as_deref()).await?;
    Ok(Json(list))
}
