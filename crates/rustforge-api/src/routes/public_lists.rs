//! Pages publiques de listes : aucune lecture administrative de contenu.
use crate::{
    error::{ApiError, ApiResult},
    state::AppState,
};
use axum::{
    extract::{OriginalUri, Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use minijinja::Value;
use rustforge_core::entries::{self, EntryQuery};
use serde::Deserialize;
use serde_json::json;

pub(super) fn validate_locale(locale: &str) -> ApiResult<()> {
    if matches!(locale, "api" | "admin" | "uploads" | "public") {
        return Err(ApiError::NotFound);
    }
    rustforge_core::security::safe_slug(locale)
        .map_err(|_| ApiError::BadRequest("Langue invalide".into()))?;
    Ok(())
}

#[derive(Default, Deserialize)]
pub struct ListParams {
    pub q: Option<String>,
    pub page: Option<String>,
}
impl ListParams {
    pub fn normalized(&self) -> ApiResult<(String, i64)> {
        if self.q.as_ref().is_some_and(|q| q.len() > 4096) {
            return Err(ApiError::BadRequest("Recherche trop longue".into()));
        }
        let q = self
            .q
            .as_deref()
            .unwrap_or("")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        if q.chars().count() > 200 {
            return Err(ApiError::BadRequest(
                "Recherche limitée à 200 caractères".into(),
            ));
        }
        let page = self
            .page
            .as_deref()
            .unwrap_or("1")
            .parse::<i64>()
            .ok()
            .filter(|n| (1..=1_000_000).contains(n))
            .ok_or_else(|| ApiError::BadRequest("Numéro de page invalide".into()))?;
        Ok((q, page))
    }
}
fn encode(raw: &str) -> String {
    raw.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}
pub fn page_context(
    ctx: &mut serde_json::Value,
    page: &rustforge_core::Page<rustforge_core::models::Entry>,
    base: &str,
    q: Option<&str>,
) -> ApiResult<()> {
    if page.page > page.total_pages.max(1) {
        return Err(ApiError::NotFound);
    }
    let link = |n| match q {
        Some(q) => format!("{base}?q={}&page={n}", encode(q)),
        None => format!("{base}?page={n}"),
    };
    ctx["entries"] = json!(page.items);
    ctx["total"] = json!(page.total);
    ctx["page"] = json!(page.page);
    ctx["total_pages"] = json!(page.total_pages);
    ctx["previous_url"] = json!(if page.page > 1 {
        Some(link(page.page - 1))
    } else {
        None
    });
    ctx["next_url"] = json!(if page.page < page.total_pages {
        Some(link(page.page + 1))
    } else {
        None
    });
    Ok(())
}

pub async fn not_found(state: &AppState, locale: &str) -> ApiResult<Response> {
    state.ssr.synchronize(&state.db).await?;
    let state_ref = state.clone();
    let loc = locale.to_string();
    let html = state
        .ssr
        .render_page(locale, "rf:not-found", move || async move {
            let mut ctx = super::public::build_site_context(&state_ref, &loc).await?;
            ctx["template"] = json!("404.html");
            ctx["force_noindex"] = json!(true);
            Ok(Value::from_serialize(ctx))
        })
        .await?;
    Ok((StatusCode::NOT_FOUND, Html(html)).into_response())
}
pub async fn fallback(
    State(state): State<AppState>,
    OriginalUri(uri): OriginalUri,
) -> ApiResult<Response> {
    let first = uri.path().split('/').nth(1).unwrap_or("");
    if matches!(first, "api" | "admin" | "uploads" | "public") {
        return Err(ApiError::NotFound);
    }
    let locale = if first == "fr" { first } else { "fr" };
    not_found(&state, locale).await
}
pub async fn search(
    State(state): State<AppState>,
    Path(locale): Path<String>,
    Query(params): Query<ListParams>,
) -> ApiResult<Response> {
    validate_locale(&locale)?;
    let (q, page) = params.normalized()?;
    render(state, locale, page, Some(q), None).await
}
pub async fn archive(
    State(state): State<AppState>,
    Path((locale, taxonomy, term)): Path<(String, String, String)>,
    Query(params): Query<ListParams>,
) -> ApiResult<Response> {
    validate_locale(&locale)?;
    let (_, page) = params.normalized()?;
    if !matches!(taxonomy.as_str(), "category" | "tag") {
        return not_found(&state, &locale).await;
    }
    render(state, locale, page, None, Some((taxonomy, term))).await
}
async fn render(
    state: AppState,
    locale: String,
    number: i64,
    search: Option<String>,
    archive: Option<(String, String)>,
) -> ApiResult<Response> {
    if rustforge_core::security::safe_slug(&locale).is_err() {
        return Err(ApiError::BadRequest("Langue invalide".into()));
    }
    state.ssr.synchronize(&state.db).await?;
    let key = serde_json::to_string(&("rf.list", &locale, number, &search, &archive)).unwrap();
    let st = state.clone();
    let loc = locale.clone();
    let rendered = state.ssr.render_page(&locale, &key, move || async move {
        let mut ctx = super::public::build_site_context(&st,&loc).await?;
        ctx["template"] = json!("listing.html");
        let mut query = EntryQuery { public_only:true, page:Some(number), per_page:ctx["reading_per_page"].as_i64(), ..Default::default() };
        let (base,q) = if let Some(q) = search {
            ctx["title"] = json!("Recherche"); ctx["search_query"] = json!(q); ctx["is_search"] = json!(true); ctx["force_noindex"] = json!(true);
            let base = format!("/{loc}/search");
            if q.is_empty() {
                if number != 1 { return Err(ApiError::NotFound); }
                ctx["entries"] = json!([]); ctx["total"] = json!(0);
                return Ok(Value::from_serialize(ctx));
            }
            query.q = Some(q.clone()); (base,Some(q))
        } else {
            let (tax,slug) = archive.unwrap();
            let name: Option<String> = sqlx::query_scalar("SELECT t.name FROM terms t JOIN taxonomies tx ON tx.id=t.taxonomy_id WHERE tx.slug=$1 AND t.slug=$2")
                .bind(&tax).bind(&slug).fetch_optional(&st.db).await.map_err(rustforge_core::CoreError::from)?;
            ctx["title"] = json!(name.ok_or(ApiError::NotFound)?);
            query.taxonomy = Some(tax.clone()); query.term = Some(slug.clone());
            (format!("/{loc}/archive/{}/{}",encode(&tax),encode(&slug)),None)
        };
        let page = entries::list(&st.db,&query).await?;
        page_context(&mut ctx,&page,&base,q.as_deref())?;
        Ok(Value::from_serialize(ctx))
    }).await;
    match rendered {
        Ok(html) => Ok(Html(html).into_response()),
        Err(ApiError::NotFound | ApiError::Core(rustforge_core::CoreError::NotFound)) => {
            not_found(&state, &locale).await
        }
        Err(e) => Err(e),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_search_and_pages() {
        assert_eq!(
            ListParams {
                q: Some("  deux  mots ".into()),
                page: None
            }
            .normalized()
            .unwrap(),
            ("deux mots".into(), 1)
        );
        for page in ["0", "-1", "abc", "9223372036854775807"] {
            assert!(
                ListParams {
                    q: None,
                    page: Some(page.into())
                }
                .normalized()
                .is_err()
            );
        }
        assert!(
            ListParams {
                q: Some("é".repeat(201)),
                page: None
            }
            .normalized()
            .is_err()
        );
        assert_eq!(encode("a&é"), "a%26%C3%A9");
    }
}
