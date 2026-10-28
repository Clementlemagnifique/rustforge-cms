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

/// Transforme un ensemble de menus plats avec `parent_id` en arbre hiérarchique avec `children` (WP-46, NAV-A/C).
pub fn transform_menus_to_tree(raw_menus: &serde_json::Value) -> serde_json::Value {
    let Some(obj) = raw_menus.as_object() else {
        return serde_json::json!({ "header": [], "footer": [] });
    };

    let mut out = serde_json::Map::new();

    for (loc, items_val) in obj {
        let Some(items) = items_val.as_array() else {
            out.insert(loc.clone(), serde_json::json!([]));
            continue;
        };

        if items.is_empty() {
            out.insert(loc.clone(), serde_json::json!([]));
            continue;
        }

        let mut by_id: std::collections::BTreeMap<String, serde_json::Map<String, serde_json::Value>> =
            std::collections::BTreeMap::new();
        let mut item_order: Vec<String> = Vec::new();

        for (idx, item) in items.iter().enumerate() {
            if let Some(item_obj) = item.as_object() {
                let mut map = item_obj.clone();
                let id = map
                    .get("id")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("rf-item-{idx}"));
                map.insert("id".into(), serde_json::json!(id));
                by_id.insert(id.clone(), map);
                item_order.push(id);
            }
        }

        let mut root_order: Vec<String> = Vec::new();
        for id in &item_order {
            if let Some(item_map) = by_id.get(id) {
                let parent_id = item_map
                    .get("parent_id")
                    .and_then(serde_json::Value::as_str)
                    .map(str::trim)
                    .filter(|p| !p.is_empty() && by_id.contains_key(*p));

                if parent_id.is_none() {
                    root_order.push(id.clone());
                }
            }
        }

        fn nest_item(
            id: &str,
            by_id: &std::collections::BTreeMap<String, serde_json::Map<String, serde_json::Value>>,
            item_order: &[String],
        ) -> serde_json::Value {
            if let Some(item_map) = by_id.get(id) {
                let mut obj = item_map.clone();
                let mut children = Vec::new();
                for other_id in item_order {
                    if let Some(other_map) = by_id.get(other_id)
                        && other_map
                            .get("parent_id")
                            .and_then(serde_json::Value::as_str)
                            .map(str::trim)
                            == Some(id)
                    {
                        children.push(nest_item(other_id, by_id, item_order));
                    }
                }
                obj.insert("children".into(), serde_json::json!(children));
                serde_json::Value::Object(obj)
            } else {
                serde_json::Value::Null
            }
        }

        let roots: Vec<serde_json::Value> = root_order
            .iter()
            .map(|id| nest_item(id, &by_id, &item_order))
            .filter(|v| !v.is_null())
            .collect();

        out.insert(loc.clone(), serde_json::json!(roots));
    }

    serde_json::Value::Object(out)
}

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
    let raw_menus = site_settings.get("site.menus").cloned().unwrap_or_else(|| {
        serde_json::json!({
            "header": [],
            "footer": []
        })
    });
    let menus = transform_menus_to_tree(&raw_menus);
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

    let site_url = site_settings
        .get("site.url")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim_end_matches('/');

    let permalink_structure = site_settings
        .get("permalink.structure")
        .and_then(|v| v.as_str())
        .unwrap_or("plain")
        .to_string();
    let category_base = site_settings
        .get("permalink.category_base")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let tag_base = site_settings
        .get("permalink.tag_base")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Ok(serde_json::json!({
        "locale": locale,
        "site_title": site_title,
        "site_tagline": site_tagline,
        "site_url": site_url,
        "design_tokens": design_tokens,
        "menus": menus,
        "blog_public": blog_public,
        "reading_per_page": per_page,
        "show_on_front": show_on_front,
        "page_on_front": page_on_front,
        "page_for_posts": site_settings.get("reading.page_for_posts"),
        "permalink_structure": permalink_structure,
        "category_base": category_base,
        "tag_base": tag_base,
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

            let site_url = base_ctx.get("site_url").and_then(|v| v.as_str()).unwrap_or("");
            base_ctx["canonical_url"] = serde_json::json!(if number > 1 {
                format!("{site_url}/{loc}?page={number}")
            } else {
                format!("{site_url}/{loc}")
            });

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
                        let site_url = base_ctx.get("site_url").and_then(|v| v.as_str()).unwrap_or("");
                        base_ctx["canonical_url"] = serde_json::json!(format!("{site_url}/{loc}"));
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

async fn render_single_entry(
    state: AppState,
    locale: String,
    mut entry: Entry,
    canonical_url_path: String,
    number: i64,
) -> ApiResult<axum::response::Response> {
    if number != 1 {
        return super::public_lists::not_found(&state, &locale).await;
    }
    state.ssr.synchronize(&state.db).await?;
    let path = format!("{canonical_url_path}?page={number}");
    let plugins = state.plugins.clone();
    let state_ref = state.clone();
    let loc = locale.clone();

    let rendered = state
        .ssr
        .render_page(&locale, &path, move || async move {
            let mut base_ctx = build_site_context(&state_ref, &loc).await?;
            entry.body_html = plugins.filter(names::CONTENT_RENDER, entry.body_html).await;

            let site_url = base_ctx
                .get("site_url")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            base_ctx["canonical_url"] =
                serde_json::json!(format!("{site_url}{canonical_url_path}"));
            base_ctx["template"] = serde_json::json!("single.html");
            base_ctx["entry"] = serde_json::to_value(&entry).unwrap_or_default();

            let comments_tree = rustforge_core::comments::get_entry_approved_comments_tree(
                &state_ref.db,
                entry.id,
                3,
            )
            .await
            .unwrap_or_default();
            base_ctx["comments_count"] = serde_json::json!(comments_tree.len());
            base_ctx["comments"] = serde_json::to_value(&comments_tree).unwrap_or_default();

            base_ctx["schema_org"] = serde_json::json!({
                "@context": "https://schema.org",
                "@type": if entry.content_type == "post" { "BlogPosting" } else { "WebPage" },
                "headline": entry.title,
                "description": entry.excerpt,
                "datePublished": entry.published_at,
                "author": { "@type": "Person", "name": entry.author_name.unwrap_or_else(|| "RustForge".into()) }
            });

            Ok(MinijinjaValue::from_serialize(&base_ctx))
        })
        .await;

    match rendered {
        Ok(html) => Ok(Html(html).into_response()),
        Err(
            crate::error::ApiError::NotFound
            | crate::error::ApiError::Core(rustforge_core::CoreError::NotFound),
        ) => super::public_lists::not_found(&state, &locale).await,
        Err(err) => Err(err),
    }
}

/// Page d'un article ou contenu unique multilingue (avec support des permaliens numériques et redirections).
pub async fn ssr_single(
    State(state): State<AppState>,
    Path((locale, content_type, slug)): Path<(String, String, String)>,
    Query(params): Query<super::public_lists::ListParams>,
) -> ApiResult<axum::response::Response> {
    super::public_lists::validate_locale(&locale)?;
    state.ssr.synchronize(&state.db).await?;
    let (_, number) = params.normalized()?;

    let (structure, category_base, tag_base) =
        super::permalinks::load_permalink_settings(&state.db).await;

    // Redirections des bases de taxonomie sans /archive/
    if (!category_base.is_empty() && content_type == category_base) || content_type == "category" {
        let base = if !category_base.is_empty() {
            &category_base
        } else {
            "category"
        };
        return Ok(Redirect::permanent(&format!("/{locale}/archive/{base}/{slug}")).into_response());
    }
    if (!tag_base.is_empty() && content_type == tag_base) || content_type == "tag" {
        let base = if !tag_base.is_empty() {
            &tag_base
        } else {
            "tag"
        };
        return Ok(Redirect::permanent(&format!("/{locale}/archive/{base}/{slug}")).into_response());
    }

    // Permalien numérique /archives/{id}
    if content_type == "archives" {
        if let Ok(entry_id) = uuid::Uuid::parse_str(&slug) {
            match entries::get_published_by_id(&state.db, entry_id).await {
                Ok(entry) => {
                    let canonical =
                        super::permalinks::compute_entry_permalink(&entry, &structure, &locale);
                    let current_path = format!("/{locale}/archives/{slug}");
                    if canonical != current_path {
                        return Ok(Redirect::permanent(&canonical).into_response());
                    }
                    return render_single_entry(state, locale, entry, canonical, number).await;
                }
                Err(rustforge_core::CoreError::NotFound) => {}
                Err(e) => return Err(e.into()),
            }
        }
        return super::public_lists::not_found(&state, &locale).await;
    }

    // Page de blog / articles
    let base_ctx = build_site_context(&state, &locale).await?;
    if content_type == "page"
        && let Ok(entry) = entries::get_published(&state.db, &content_type, &slug).await
        && base_ctx["show_on_front"] == "page"
        && base_ctx["page_for_posts"].as_str() == Some(&entry.id.to_string())
    {
        let page = entries::list(
            &state.db,
            &EntryQuery {
                public_only: true,
                content_type: Some("post".into()),
                page: Some(number),
                per_page: base_ctx["reading_per_page"].as_i64(),
                ..Default::default()
            },
        )
        .await?;
        let mut listing_ctx = base_ctx;
        listing_ctx["template"] = serde_json::json!("listing.html");
        listing_ctx["title"] = serde_json::json!(entry.title);
        super::public_lists::page_context(
            &mut listing_ctx,
            &page,
            &format!("/{locale}/{content_type}/{slug}"),
            None,
        )?;
        let html = state
            .ssr
            .render_page(&locale, &format!("/{locale}/{content_type}/{slug}?page={number}"), move || async move {
                Ok(MinijinjaValue::from_serialize(listing_ctx))
            })
            .await?;
        return Ok(Html(html).into_response());
    }

    // Recherche de l'entrée publiée
    let entry_res = entries::get_published(&state.db, &content_type, &slug).await;
    let entry = match entry_res {
        Ok(e) => e,
        Err(rustforge_core::CoreError::NotFound) => {
            if let Some(target_slug) =
                entries::resolve_slug_redirect(&state.db, &content_type, &slug).await?
                && let Ok(target) =
                    entries::get_published(&state.db, &content_type, &target_slug).await
            {
                let canonical =
                    super::permalinks::compute_entry_permalink(&target, &structure, &locale);
                return Ok(Redirect::permanent(&canonical).into_response());
            }
            return super::public_lists::not_found(&state, &locale).await;
        }
        Err(e) => return Err(e.into()),
    };

    let canonical = super::permalinks::compute_entry_permalink(&entry, &structure, &locale);
    let current_path = format!("/{locale}/{content_type}/{slug}");
    if canonical != current_path {
        return Ok(Redirect::permanent(&canonical).into_response());
    }

    render_single_entry(state, locale, entry, canonical, number).await
}

/// Permalien date et titre : `/{locale}/{year}/{month}/{day}/{slug}`
pub async fn ssr_day_and_name(
    State(state): State<AppState>,
    Path((locale, year_str, month_str, day_str, slug)): Path<(
        String,
        String,
        String,
        String,
        String,
    )>,
    Query(params): Query<super::public_lists::ListParams>,
) -> ApiResult<axum::response::Response> {
    super::public_lists::validate_locale(&locale)?;
    let (Ok(year), Ok(month), Ok(day)) = (
        year_str.parse::<i32>(),
        month_str.parse::<u32>(),
        day_str.parse::<u32>(),
    ) else {
        return super::public_lists::not_found(&state, &locale).await;
    };
    if !(1900..=2100).contains(&year) || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return super::public_lists::not_found(&state, &locale).await;
    }
    let (_, number) = params.normalized()?;

    let (structure, _, _) = super::permalinks::load_permalink_settings(&state.db).await;

    let entry_res = entries::get_published(&state.db, "post", &slug).await;
    let entry = match entry_res {
        Ok(e) => e,
        Err(rustforge_core::CoreError::NotFound) => {
            if let Some(target_slug) = entries::resolve_slug_redirect(&state.db, "post", &slug).await?
                && let Ok(target) = entries::get_published(&state.db, "post", &target_slug).await
            {
                let canonical =
                    super::permalinks::compute_entry_permalink(&target, &structure, &locale);
                return Ok(Redirect::permanent(&canonical).into_response());
            }
            return super::public_lists::not_found(&state, &locale).await;
        }
        Err(e) => return Err(e.into()),
    };

    let canonical = super::permalinks::compute_entry_permalink(&entry, &structure, &locale);
    let current_path = format!("/{locale}/{year:04}/{month:02}/{day:02}/{slug}");
    if canonical != current_path {
        return Ok(Redirect::permanent(&canonical).into_response());
    }

    render_single_entry(state, locale, entry, canonical, number).await
}

/// Permalien mois et titre : `/{locale}/{year}/{month}/{slug}`
pub async fn ssr_month_and_name(
    State(state): State<AppState>,
    Path((locale, year_str, month_str, slug)): Path<(String, String, String, String)>,
    Query(params): Query<super::public_lists::ListParams>,
) -> ApiResult<axum::response::Response> {
    super::public_lists::validate_locale(&locale)?;
    let (Ok(year), Ok(month)) = (
        year_str.parse::<i32>(),
        month_str.parse::<u32>(),
    ) else {
        return super::public_lists::not_found(&state, &locale).await;
    };
    if !(1900..=2100).contains(&year) || !(1..=12).contains(&month) {
        return super::public_lists::not_found(&state, &locale).await;
    }
    let (_, number) = params.normalized()?;

    let (structure, _, _) = super::permalinks::load_permalink_settings(&state.db).await;

    let entry_res = entries::get_published(&state.db, "post", &slug).await;
    let entry = match entry_res {
        Ok(e) => e,
        Err(rustforge_core::CoreError::NotFound) => {
            if let Some(target_slug) = entries::resolve_slug_redirect(&state.db, "post", &slug).await?
                && let Ok(target) = entries::get_published(&state.db, "post", &target_slug).await
            {
                let canonical =
                    super::permalinks::compute_entry_permalink(&target, &structure, &locale);
                return Ok(Redirect::permanent(&canonical).into_response());
            }
            return super::public_lists::not_found(&state, &locale).await;
        }
        Err(e) => return Err(e.into()),
    };

    let canonical = super::permalinks::compute_entry_permalink(&entry, &structure, &locale);
    let current_path = format!("/{locale}/{year:04}/{month:02}/{slug}");
    if canonical != current_path {
        return Ok(Redirect::permanent(&canonical).into_response());
    }

    render_single_entry(state, locale, entry, canonical, number).await
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_transform_menus_to_tree() {
        // Flat menu gets children: [] on every item
        let flat = json!({
            "header": [
                {"id": "1", "label": "Accueil", "url": "/fr"},
                {"id": "2", "label": "Contact", "url": "/fr/contact"}
            ],
            "footer": []
        });
        let tree = transform_menus_to_tree(&flat);
        let header = tree.get("header").and_then(Value::as_array).unwrap();
        assert_eq!(header.len(), 2);
        assert_eq!(header[0]["label"], "Accueil");
        assert_eq!(header[0]["children"], json!([]));
        assert_eq!(header[1]["label"], "Contact");
        assert_eq!(header[1]["children"], json!([]));

        // Hierarchical menu with nested children and badges
        let hierarchical = json!({
            "header": [
                {"id": "services", "label": "Services", "url": "/services"},
                {"id": "web", "parent_id": "services", "label": "Développement Web", "url": "/services/web", "badge": "Nouveau", "subtitle": "Sur mesure"},
                {"id": "audit", "parent_id": "services", "label": "Audit & Conseil", "url": "/services/audit"},
                {"id": "secu", "parent_id": "audit", "label": "Sécurité Rust", "url": "/services/audit/secu"},
                {"id": "a-propos", "label": "À propos", "url": "/about"}
            ],
            "footer": [
                {"id": "mentions", "label": "Mentions Légales", "url": "/mentions"}
            ]
        });
        let res = transform_menus_to_tree(&hierarchical);
        let res_header = res.get("header").and_then(Value::as_array).unwrap();
        // Roots should be "services" and "a-propos"
        assert_eq!(res_header.len(), 2);
        assert_eq!(res_header[0]["id"], "services");
        let services_children = res_header[0]["children"].as_array().unwrap();
        assert_eq!(services_children.len(), 2);
        assert_eq!(services_children[0]["id"], "web");
        assert_eq!(services_children[0]["badge"], "Nouveau");
        assert_eq!(services_children[0]["subtitle"], "Sur mesure");
        assert_eq!(services_children[1]["id"], "audit");
        let audit_children = services_children[1]["children"].as_array().unwrap();
        assert_eq!(audit_children.len(), 1);
        assert_eq!(audit_children[0]["id"], "secu");

        assert_eq!(res_header[1]["id"], "a-propos");
        assert_eq!(res_header[1]["children"], json!([]));
    }
}

