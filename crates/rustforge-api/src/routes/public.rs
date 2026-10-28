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

pub(super) async fn build_site_context(
    state: &AppState,
    locale: &str,
) -> ApiResult<serde_json::Value> {
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
    let menus = site_settings.get("site.menus").cloned().unwrap_or_else(|| {
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
        "page_for_posts": site_settings.get("reading.page_for_posts"),
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
    Query(params): Query<super::public_lists::ListParams>,
) -> ApiResult<axum::response::Response> {
    super::public_lists::validate_locale(&locale)?;
    state.ssr.synchronize(&state.db).await?;
    let (_, number) = params.normalized()?;
    let path = format!("/{locale}?page={number}");
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
            {
                match entries::get_published_page(&db, front_id).await {
                    Ok(mut entry) => {
                        if number != 1 {
                            return Err(crate::error::ApiError::NotFound);
                        }
                        entry.body_html =
                            plugins.filter(names::CONTENT_RENDER, entry.body_html).await;
                        base_ctx["template"] = serde_json::json!("single.html");
                        base_ctx["entry"] = serde_json::to_value(&entry).unwrap_or_default();
                        return Ok(MinijinjaValue::from_serialize(&base_ctx));
                    }
                    Err(rustforge_core::CoreError::NotFound) => {}
                    Err(err) => return Err(err.into()),
                }
            }

            let query = EntryQuery {
                public_only: true,
                content_type: Some("post".into()),
                per_page: Some(per_page),
                page: Some(number),
                ..Default::default()
            };
            let page = entries::list(&db, &query).await?;
            base_ctx["template"] = serde_json::json!("home.html");
            super::public_lists::page_context(&mut base_ctx, &page, &format!("/{loc}"), None)?;

            Ok(MinijinjaValue::from_serialize(&base_ctx))
        })
        .await;

    match html {
        Ok(html) => Ok(Html(html).into_response()),
        Err(crate::error::ApiError::NotFound) => {
            super::public_lists::not_found(&state, &locale).await
        }
        Err(e) => Err(e),
    }
}

/// Page d'un article ou contenu unique multilingue
pub async fn ssr_single(
    State(state): State<AppState>,
    Path((locale, content_type, slug)): Path<(String, String, String)>,
    Query(params): Query<super::public_lists::ListParams>,
) -> ApiResult<axum::response::Response> {
    super::public_lists::validate_locale(&locale)?;
    state.ssr.synchronize(&state.db).await?;
    let (_, number) = params.normalized()?;
    let path = format!("/{locale}/{content_type}/{slug}?page={number}");
    let db = state.db.clone();
    let plugins = state.plugins.clone();
    let state_ref = state.clone();
    let loc = locale.clone();
    let ct = content_type.clone();
    let sl = slug.clone();

    let rendered = state.ssr.render_page(&locale, &path, move || async move {
        let mut base_ctx = build_site_context(&state_ref, &loc).await?;
        let mut entry = entries::get_published(&db, &ct, &sl).await?;

        if ct == "page" && base_ctx["show_on_front"] == "page" && base_ctx["page_for_posts"].as_str() == Some(&entry.id.to_string()) {
            let page = entries::list(&db,&EntryQuery { public_only:true, content_type:Some("post".into()), page:Some(number), per_page:base_ctx["reading_per_page"].as_i64(), ..Default::default() }).await?;
            base_ctx["template"] = serde_json::json!("listing.html");
            base_ctx["title"] = serde_json::json!(entry.title);
            super::public_lists::page_context(&mut base_ctx,&page,&format!("/{loc}/{ct}/{sl}"),None)?;
            return Ok(MinijinjaValue::from_serialize(base_ctx));
        }
        if number != 1 { return Err(crate::error::ApiError::NotFound); }

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
            super::public_lists::not_found(&state, &locale).await
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
    super::public_lists::validate_locale(&locale)?;
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
                return Ok(Redirect::permanent(&format!(
                    "/{locale}/{content_type}/{target_slug}/zen"
                ))
                .into_response());
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
