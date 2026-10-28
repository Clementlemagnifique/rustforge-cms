use std::collections::BTreeMap;

use axum::{
    Json,
    extract::{Path, Query, State},
    response::{Html, IntoResponse, Redirect},
};
use minijinja::Value as MinijinjaValue;
use serde_json::Value;

use crate::{error::ApiResult, state::AppState};
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
    let site_title = site_settings
        .get("site.title")
        .and_then(|v| v.as_str())
        .unwrap_or("RustForge");
    let site_tagline = site_settings
        .get("site.tagline")
        .and_then(|v| v.as_str())
        .unwrap_or("Le CMS Rust IA-First");
    let design_tokens = site_settings
        .get("site.design_tokens")
        .cloned()
        .unwrap_or_else(|| {
            serde_json::json!({
                "primary_color": "#4f46e5",
                "accent_color": "#06b6d4",
                "radius": "12px",
                "container_width": "1100px",
                "custom_css": ""
            })
        });
    let menus = site_settings
        .get("site.menus")
        .cloned()
        .unwrap_or_else(|| {
            serde_json::json!({
                "header": [],
                "footer": []
            })
        });
    let blog_public = site_settings
        .get("reading.blog_public")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let per_page = site_settings
        .get("reading.per_page")
        .and_then(|v| v.as_i64())
        .filter(|&n| (1..=100).contains(&n))
        .unwrap_or(10);
    let show_on_front = site_settings
        .get("reading.show_on_front")
        .and_then(|v| v.as_str())
        .unwrap_or("posts")
        .to_string();
    let page_on_front = site_settings
        .get("reading.page_on_front")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    Ok(serde_json::json!({
        "locale": locale,
        "site_title": site_title,
        "site_tagline": site_tagline,
        "design_tokens": design_tokens,
        "menus": menus,
        "blog_public": blog_public,
        "reading_per_page": per_page,
        "show_on_front": show_on_front,
        "page_on_front": page_on_front,
        "available_locales": [
            { "code": "fr", "url": "/fr" }
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
    state.ssr.synchronize(&state.db).await?;
    let path = format!("/{locale}");
    let db = state.db.clone();
    let plugins = state.plugins.clone();
    let state_ref = state.clone();
    let loc = locale.clone();

    let html = state
        .ssr
        .render_page(&locale, &path, move || async move {
            let mut base_ctx = build_site_context(&state_ref, &loc).await?;
            let per_page = base_ctx
                .get("reading_per_page")
                .and_then(|v| v.as_i64())
                .unwrap_or(10);
            let show_on_front = base_ctx
                .get("show_on_front")
                .and_then(|v| v.as_str())
                .unwrap_or("posts")
                .to_string();
            let page_on_front = base_ctx
                .get("page_on_front")
                .and_then(|v| v.as_str())
                .and_then(|s| uuid::Uuid::parse_str(s).ok());

            if show_on_front == "page"
                && let Some(front_id) = page_on_front
                && let Ok(mut entry) = entries::get(&db, front_id).await
                && entry.status == rustforge_core::models::EntryStatus::Published
            {
                entry.body_html = plugins.filter(names::CONTENT_RENDER, entry.body_html).await;
                base_ctx["template"] = serde_json::json!("single.html");
                base_ctx["entry"] = serde_json::to_value(&entry).unwrap_or_default();
                return Ok(MinijinjaValue::from_serialize(&base_ctx));
            }

            let query = EntryQuery {
                public_only: true,
                per_page: Some(per_page),
                ..Default::default()
            };
            let page = entries::list(&db, &query).await?;
            base_ctx["template"] = serde_json::json!("home.html");
            base_ctx["entries"] = serde_json::to_value(&page.items).unwrap_or_default();

            Ok(MinijinjaValue::from_serialize(&base_ctx))
        })
        .await?;

    Ok(Html(html))
}

/// Page d'un article ou contenu unique multilingue
pub async fn ssr_single(
    State(state): State<AppState>,
    Path((locale, content_type, slug)): Path<(String, String, String)>,
) -> ApiResult<axum::response::Response> {
    state.ssr.synchronize(&state.db).await?;
    let path = format!("/{locale}/{content_type}/{slug}");
    let db = state.db.clone();
    let plugins = state.plugins.clone();
    let state_ref = state.clone();
    let loc = locale.clone();
    let ct = content_type.clone();
    let sl = slug.clone();

    let rendered = state.ssr.render_page(&locale, &path, move || async move {
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
    }).await;

    match rendered {
        Ok(html) => Ok(Html(html).into_response()),
        Err(
            crate::error::ApiError::NotFound
            | crate::error::ApiError::Core(rustforge_core::CoreError::NotFound),
        ) => {
            if let Some(target_slug) =
                entries::resolve_slug_redirect(&state.db, &content_type, &slug).await?
            {
                return Ok(
                    Redirect::permanent(&format!("/{locale}/{content_type}/{target_slug}"))
                        .into_response(),
                );
            }
            Err(crate::error::ApiError::NotFound)
        }
        Err(err) => Err(err),
    }
}

/// Page en Mode Zen sans distraction
pub async fn ssr_zen(
    State(state): State<AppState>,
    Path((locale, content_type, slug)): Path<(String, String, String)>,
) -> ApiResult<axum::response::Response> {
    if !settings::is_module_active(&state.db, "zen_mode").await {
        return Err(rustforge_core::CoreError::NotFound.into());
    }
    state.ssr.synchronize(&state.db).await?;
    let path = format!("/{locale}/{content_type}/{slug}/zen");
    let db = state.db.clone();
    let plugins = state.plugins.clone();
    let state_ref = state.clone();
    let loc = locale.clone();
    let ct = content_type.clone();
    let sl = slug.clone();

    let rendered = state
        .ssr
        .render_page(&locale, &path, move || async move {
            let mut base_ctx = build_site_context(&state_ref, &loc).await?;
            let mut entry = entries::get_published(&db, &ct, &sl).await?;
            entry.body_html = plugins.filter(names::CONTENT_RENDER, entry.body_html).await;

            base_ctx["template"] = serde_json::json!("zen.html");
            base_ctx["entry"] = serde_json::to_value(&entry).unwrap_or_default();

            Ok(MinijinjaValue::from_serialize(&base_ctx))
        })
        .await;

    match rendered {
        Ok(html) => Ok(Html(html).into_response()),
        Err(
            crate::error::ApiError::NotFound
            | crate::error::ApiError::Core(rustforge_core::CoreError::NotFound),
        ) => {
            if let Some(target_slug) =
                entries::resolve_slug_redirect(&state.db, &content_type, &slug).await?
            {
                return Ok(
                    Redirect::permanent(&format!("/{locale}/{content_type}/{target_slug}/zen"))
                        .into_response(),
                );
            }
            Err(crate::error::ApiError::NotFound)
        }
        Err(err) => Err(err),
    }
}

// ---------------------------------------------------------------------------
// API Headless JSON (pour clients externes : React, Astro, Next.js)
// ---------------------------------------------------------------------------

pub async fn list_entries(
    State(state): State<AppState>,
    Query(mut query): Query<EntryQuery>,
) -> ApiResult<Json<Page<Value>>> {
    query.public_only = true;
    let page = entries::list(&state.db, &query).await?;
    Ok(Json(page.map(Entry::public_view)))
}

pub async fn get_entry(
    State(state): State<AppState>,
    Path((content_type, slug)): Path<(String, String)>,
) -> ApiResult<Json<Value>> {
    let mut entry = entries::get_published(&state.db, &content_type, &slug).await?;
    entry.body_html = state
        .plugins
        .filter(names::CONTENT_RENDER, entry.body_html)
        .await;
    Ok(Json(entry.public_view()))
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

pub async fn preview_theme(
    State(state): State<AppState>,
    auth: crate::auth::AuthUser,
    Path(slug): Path<String>,
) -> ApiResult<Html<String>> {
    if !auth.user.role.is_admin() {
        return Err(crate::error::ApiError::Forbidden);
    }
    rustforge_core::security::safe_slug(&slug)?;
    rustforge_core::security::confined_path(&state.themes_dir, &slug)?;
    let engine = crate::ssr::SsrEngine::new(state.themes_dir.clone(), slug);
    let mut context = build_site_context(&state, "fr").await?;
    context["template"] = serde_json::json!("single.html");
    context["entry"] = serde_json::json!({"title":"Aperçu du thème", "body_html":"<p>Contenu de démonstration. Ce thème n'est pas encore activé.</p>","excerpt":"Aperçu privé", "content_type":"page", "slug":"apercu"});
    let html = engine
        .render_page("fr", "preview", || async {
            Ok(MinijinjaValue::from_serialize(&context))
        })
        .await?;
    Ok(Html(html))
}
