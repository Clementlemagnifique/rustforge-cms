//! # SEO structurel : robots.txt, sitemap.xml et balises d'indexation (WP-81, WP-102)
//!
//! Service commun garantissant la cohérence SEO de RustForge :
//! - `/robots.txt` dynamique respectant strictement `reading.blog_public` (noindex global si désactivé).
//! - `/sitemap.xml` conforme sitemaps.org (404 si `blog_public == false`, exclusion stricte des brouillons et expirés).
//! - `/sitemap.xml` et `/{locale}/sitemap.xml` pour l'indexation multilingue.

use crate::{
    error::{ApiError, ApiResult},
    routes::public_lists,
    state::AppState,
};
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::Response,
};
use chrono::{DateTime, Utc};
use rustforge_core::settings;

#[derive(sqlx::FromRow)]
struct SitemapEntry {
    id: uuid::Uuid,
    content_type: String,
    slug: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    published_at: Option<DateTime<Utc>>,
}

pub fn generate_robots_txt(blog_public: bool, site_url: &str) -> String {
    let base_url = site_url.trim_end_matches('/');
    if !blog_public {
        "User-agent: *\nDisallow: /\n".to_string()
    } else if base_url.is_empty() {
        "User-agent: *\nDisallow: /api/\nDisallow: /admin/\nAllow: /\n\nSitemap: /sitemap.xml\n"
            .to_string()
    } else {
        format!(
            "User-agent: *\nDisallow: /api/\nDisallow: /admin/\nAllow: /\n\nSitemap: {base_url}/sitemap.xml\n"
        )
    }
}

pub fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn build_sitemap_xml(
    site_url: &str,
    locale: &str,
    entries: &[SitemapEntry],
    permalink_structure: &str,
) -> String {
    let mut xml = String::with_capacity(4096);
    let base_url = site_url.trim_end_matches('/');

    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");

    let now_rfc3339 = Utc::now().to_rfc3339();
    let home_loc = if base_url.is_empty() {
        format!("/{locale}")
    } else {
        format!("{base_url}/{locale}")
    };

    xml.push_str("  <url>\n");
    xml.push_str(&format!("    <loc>{}</loc>\n", xml_escape(&home_loc)));
    xml.push_str(&format!("    <lastmod>{now_rfc3339}</lastmod>\n"));
    xml.push_str("    <changefreq>daily</changefreq>\n");
    xml.push_str("    <priority>1.0</priority>\n");
    xml.push_str("  </url>\n");

    for entry in entries {
        let date = entry.published_at.unwrap_or(entry.created_at);
        let permalink = super::permalinks::compute_permalink_parts(
            &entry.content_type,
            &entry.slug,
            date,
            entry.id,
            None,
            None,
            permalink_structure,
            locale,
        );
        let entry_loc = if base_url.is_empty() {
            permalink
        } else {
            format!("{base_url}{permalink}")
        };
        let lastmod = entry
            .published_at
            .unwrap_or(entry.updated_at)
            .max(entry.updated_at)
            .to_rfc3339();
        let priority = if entry.content_type == "page" {
            "0.9"
        } else {
            "0.8"
        };
        let changefreq = if entry.content_type == "page" {
            "monthly"
        } else {
            "weekly"
        };

        xml.push_str("  <url>\n");
        xml.push_str(&format!("    <loc>{}</loc>\n", xml_escape(&entry_loc)));
        xml.push_str(&format!("    <lastmod>{lastmod}</lastmod>\n"));
        xml.push_str(&format!("    <changefreq>{changefreq}</changefreq>\n"));
        xml.push_str(&format!("    <priority>{priority}</priority>\n"));
        xml.push_str("  </url>\n");
    }

    xml.push_str("</urlset>\n");
    xml
}

pub async fn robots_txt(State(state): State<AppState>) -> ApiResult<Response> {
    let site_settings = settings::all(&state.db).await?;
    let blog_public = site_settings
        .get("reading.blog_public")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let site_url = site_settings
        .get("site.url")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let txt = generate_robots_txt(blog_public, site_url);
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
        .body(Body::from(txt))
        .map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))
}

async fn handle_sitemap(state: AppState, locale: &str) -> ApiResult<Response> {
    public_lists::validate_locale(locale)?;
    state.ssr.synchronize(&state.db).await?;

    let site_settings = settings::all(&state.db).await?;
    let blog_public = site_settings
        .get("reading.blog_public")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    if !blog_public {
        return Err(ApiError::NotFound);
    }

    let site_url = site_settings
        .get("site.url")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let (permalink_structure, _, _) =
        super::permalinks::load_permalink_settings(&state.db).await;

    let entries: Vec<SitemapEntry> = sqlx::query_as(
        r#"SELECT e.id, ct.slug AS content_type, e.slug, e.created_at, e.updated_at, e.published_at
           FROM entries e
           JOIN content_types ct ON ct.id = e.content_type_id
           WHERE e.status = 'published'::entry_status
             AND e.published_at IS NOT NULL
             AND e.published_at <= now()
             AND (e.scheduled_unpublish_at IS NULL OR e.scheduled_unpublish_at > now())
           ORDER BY e.updated_at DESC
           LIMIT 50000"#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))?;

    let xml = build_sitemap_xml(site_url, locale, &entries, &permalink_structure);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/xml; charset=utf-8")
        .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
        .body(Body::from(xml))
        .map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))
}

pub async fn sitemap_root(State(state): State<AppState>) -> ApiResult<Response> {
    handle_sitemap(state, "fr").await
}

pub async fn sitemap_locale(
    State(state): State<AppState>,
    Path(locale): Path<String>,
) -> ApiResult<Response> {
    handle_sitemap(state, &locale).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_robots_txt_when_blog_public_false() {
        let txt = generate_robots_txt(false, "https://example.com");
        assert_eq!(txt, "User-agent: *\nDisallow: /\n");
        assert!(!txt.contains("Sitemap"));
        assert!(!txt.contains("Allow"));
    }

    #[test]
    fn test_robots_txt_when_blog_public_true() {
        let txt = generate_robots_txt(true, "https://example.com");
        assert!(txt.contains("User-agent: *"));
        assert!(txt.contains("Disallow: /api/"));
        assert!(txt.contains("Disallow: /admin/"));
        assert!(txt.contains("Allow: /"));
        assert!(txt.contains("Sitemap: https://example.com/sitemap.xml"));
    }

    #[test]
    fn test_sitemap_xml_generation_and_escaping() {
        let entries = vec![
            SitemapEntry {
                id: uuid::Uuid::nil(),
                content_type: "post".into(),
                slug: "article-1".into(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                published_at: Some(Utc::now()),
            },
            SitemapEntry {
                id: uuid::Uuid::nil(),
                content_type: "page".into(),
                slug: "page-propos".into(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                published_at: Some(Utc::now()),
            },
        ];
        let xml = build_sitemap_xml("https://example.com", "fr", &entries, "plain");
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">"));
        assert!(xml.contains("<loc>https://example.com/fr</loc>"));
        assert!(xml.contains("<loc>https://example.com/fr/post/article-1</loc>"));
        assert!(xml.contains("<loc>https://example.com/fr/page/page-propos</loc>"));
        assert!(xml.contains("<priority>1.0</priority>"));
        assert!(xml.contains("<priority>0.9</priority>"));
        assert!(xml.contains("<priority>0.8</priority>"));
    }
}
