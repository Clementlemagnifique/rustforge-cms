//! # Flux de syndication RSS 2.0 et Atom 1.0 (WP-79, WP-80, WP-101)
//!
//! Expose les flux éditoriaux conformes aux normes RSS 2.0 et Atom 1.0 :
//! - Filtrage strict sur les articles publiés et visibles (exclusion brouillons, futurs, expirés).
//! - Respect du réglage `reading.posts_per_rss` (limite distincte de la pagination d'affichage).
//! - Respect du réglage `reading.rss_use_excerpt` (texte complet ou extrait).
//! - Protection intégrale contre les injections XML et cassage de balises CDATA.

use crate::{
    error::{ApiError, ApiResult},
    routes::public_lists,
    state::AppState,
};
use axum::{
    body::Body,
    extract::{OriginalUri, Path, Query, State},
    http::{header, StatusCode},
    response::Response,
};
use chrono::Utc;
use rustforge_core::{
    entries::{self, EntryQuery},
    models::Entry,
    settings,
};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedFormat {
    Rss,
    Atom,
}

#[derive(Default, Deserialize)]
pub struct FeedParams {
    pub format: Option<String>,
}

pub fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub fn xml_cdata(s: &str) -> String {
    format!("<![CDATA[{}]]>", s.replace("]]>", "]]]]><![CDATA[>"))
}

#[allow(clippy::too_many_arguments)]
pub fn generate_rss(
    site_title: &str,
    site_tagline: &str,
    site_url: &str,
    locale: &str,
    feed_url: &str,
    entries: &[Entry],
    rss_use_excerpt: bool,
    permalink_structure: &str,
) -> String {
    let mut xml = String::with_capacity(4096);
    let base_url = site_url.trim_end_matches('/');

    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<rss version=\"2.0\"\n");
    xml.push_str("     xmlns:content=\"http://purl.org/rss/1.0/modules/content/\"\n");
    xml.push_str("     xmlns:dc=\"http://purl.org/dc/elements/1.1/\"\n");
    xml.push_str("     xmlns:atom=\"http://www.w3.org/2005/Atom\">\n");
    xml.push_str("  <channel>\n");
    xml.push_str(&format!("    <title>{}</title>\n", xml_escape(site_title)));
    xml.push_str(&format!(
        "    <atom:link href=\"{}\" rel=\"self\" type=\"application/rss+xml\" />\n",
        xml_escape(feed_url)
    ));
    let site_link = if base_url.is_empty() {
        format!("/{locale}")
    } else {
        format!("{base_url}/{locale}")
    };
    xml.push_str(&format!("    <link>{}</link>\n", xml_escape(&site_link)));
    xml.push_str(&format!(
        "    <description>{}</description>\n",
        xml_escape(site_tagline)
    ));

    let now_rfc2822 = Utc::now().to_rfc2822();
    let last_build = entries
        .first()
        .and_then(|e| e.published_at.map(|d| d.to_rfc2822()))
        .unwrap_or_else(|| now_rfc2822.clone());
    xml.push_str(&format!("    <lastBuildDate>{last_build}</lastBuildDate>\n"));
    xml.push_str(&format!(
        "    <language>{}</language>\n",
        xml_escape(locale)
    ));
    xml.push_str("    <generator>RustForge CMS</generator>\n");

    for entry in entries {
        let permalink =
            super::permalinks::compute_entry_permalink(entry, permalink_structure, locale);
        let entry_url = if base_url.is_empty() {
            permalink
        } else {
            format!("{base_url}{permalink}")
        };
        let pub_date = entry
            .published_at
            .map(|d| d.to_rfc2822())
            .unwrap_or_else(|| now_rfc2822.clone());
        let desc = if !entry.excerpt.trim().is_empty() {
            entry.excerpt.clone()
        } else {
            rustforge_core::markdown::excerpt(&entry.body, 35)
        };

        xml.push_str("    <item>\n");
        xml.push_str(&format!(
            "      <title>{}</title>\n",
            xml_escape(&entry.title)
        ));
        xml.push_str(&format!("      <link>{}</link>\n", xml_escape(&entry_url)));
        if let Some(author) = &entry.author_name {
            xml.push_str(&format!(
                "      <dc:creator>{}</dc:creator>\n",
                xml_escape(author)
            ));
        }
        xml.push_str(&format!("      <pubDate>{pub_date}</pubDate>\n"));
        xml.push_str(&format!(
            "      <guid isPermaLink=\"true\">{}</guid>\n",
            xml_escape(&entry_url)
        ));
        xml.push_str(&format!(
            "      <description>{}</description>\n",
            xml_cdata(&desc)
        ));
        if !rss_use_excerpt {
            let content = if !entry.body_html.trim().is_empty() {
                &entry.body_html
            } else {
                &entry.body
            };
            xml.push_str(&format!(
                "      <content:encoded>{}</content:encoded>\n",
                xml_cdata(content)
            ));
        }
        xml.push_str("    </item>\n");
    }

    xml.push_str("  </channel>\n</rss>\n");
    xml
}

#[allow(clippy::too_many_arguments)]
pub fn generate_atom(
    site_title: &str,
    site_tagline: &str,
    site_url: &str,
    locale: &str,
    feed_url: &str,
    entries: &[Entry],
    rss_use_excerpt: bool,
    permalink_structure: &str,
) -> String {
    let mut xml = String::with_capacity(4096);
    let base_url = site_url.trim_end_matches('/');

    xml.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    xml.push_str("<feed xmlns=\"http://www.w3.org/2005/Atom\">\n");
    xml.push_str(&format!("  <title>{}</title>\n", xml_escape(site_title)));
    if !site_tagline.is_empty() {
        xml.push_str(&format!(
            "  <subtitle>{}</subtitle>\n",
            xml_escape(site_tagline)
        ));
    }
    xml.push_str(&format!(
        "  <link href=\"{}\" rel=\"self\" type=\"application/atom+xml\" />\n",
        xml_escape(feed_url)
    ));
    let site_link = if base_url.is_empty() {
        format!("/{locale}")
    } else {
        format!("{base_url}/{locale}")
    };
    xml.push_str(&format!(
        "  <link href=\"{}\" rel=\"alternate\" type=\"text/html\" />\n",
        xml_escape(&site_link)
    ));
    xml.push_str(&format!("  <id>{}</id>\n", xml_escape(feed_url)));

    let now_rfc3339 = Utc::now().to_rfc3339();
    let updated = entries
        .first()
        .and_then(|e| e.published_at.map(|d| d.to_rfc3339()))
        .unwrap_or_else(|| now_rfc3339.clone());
    xml.push_str(&format!("  <updated>{updated}</updated>\n"));
    xml.push_str("  <generator uri=\"https://rustforge.org\" version=\"1.0.0-alpha.1\">RustForge CMS</generator>\n");

    for entry in entries {
        let permalink =
            super::permalinks::compute_entry_permalink(entry, permalink_structure, locale);
        let entry_url = if base_url.is_empty() {
            permalink
        } else {
            format!("{base_url}{permalink}")
        };
        let pub_date = entry
            .published_at
            .map(|d| d.to_rfc3339())
            .unwrap_or_else(|| now_rfc3339.clone());
        let updated_date = entry.created_at.to_rfc3339();
        let desc = if !entry.excerpt.trim().is_empty() {
            entry.excerpt.clone()
        } else {
            rustforge_core::markdown::excerpt(&entry.body, 35)
        };

        xml.push_str("  <entry>\n");
        xml.push_str(&format!(
            "    <title>{}</title>\n",
            xml_escape(&entry.title)
        ));
        xml.push_str(&format!(
            "    <link href=\"{}\" rel=\"alternate\" type=\"text/html\" />\n",
            xml_escape(&entry_url)
        ));
        xml.push_str(&format!("    <id>{}</id>\n", xml_escape(&entry_url)));
        xml.push_str(&format!("    <published>{pub_date}</published>\n"));
        xml.push_str(&format!("    <updated>{updated_date}</updated>\n"));
        if let Some(author) = &entry.author_name {
            xml.push_str("    <author>\n");
            xml.push_str(&format!("      <name>{}</name>\n", xml_escape(author)));
            xml.push_str("    </author>\n");
        }
        xml.push_str(&format!(
            "    <summary type=\"html\">{}</summary>\n",
            xml_cdata(&desc)
        ));
        if !rss_use_excerpt {
            let content = if !entry.body_html.trim().is_empty() {
                &entry.body_html
            } else {
                &entry.body
            };
            xml.push_str(&format!(
                "    <content type=\"html\">{}</content>\n",
                xml_cdata(content)
            ));
        }
        xml.push_str("  </entry>\n");
    }

    xml.push_str("</feed>\n");
    xml
}

async fn handle_feed(
    state: AppState,
    locale: &str,
    format: FeedFormat,
    path_uri: &str,
) -> ApiResult<Response> {
    public_lists::validate_locale(locale)?;
    state.ssr.synchronize(&state.db).await?;

    let site_settings = settings::all(&state.db).await?;
    let site_title = site_settings
        .get("site.title")
        .and_then(|v| v.as_str())
        .unwrap_or("RustForge");
    let site_tagline = site_settings
        .get("site.tagline")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let site_url = site_settings
        .get("site.url")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let posts_per_rss = site_settings
        .get("reading.posts_per_rss")
        .and_then(|v| v.as_i64())
        .filter(|&n| (1..=100).contains(&n))
        .unwrap_or(10);
    let rss_use_excerpt = site_settings
        .get("reading.rss_use_excerpt")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let page = entries::list(
        &state.db,
        &EntryQuery {
            content_type: Some("post".into()),
            order: Some("published".into()),
            page: Some(1),
            per_page: Some(posts_per_rss),
            public_only: true,
            ..Default::default()
        },
    )
    .await?;

    let base_url = site_url.trim_end_matches('/');
    let feed_url = if base_url.is_empty() {
        path_uri.to_string()
    } else {
        format!("{base_url}{path_uri}")
    };

    let (permalink_structure, _, _) =
        super::permalinks::load_permalink_settings(&state.db).await;

    let (content_type, xml) = match format {
        FeedFormat::Rss => (
            "application/rss+xml; charset=utf-8",
            generate_rss(
                site_title,
                site_tagline,
                site_url,
                locale,
                &feed_url,
                &page.items,
                rss_use_excerpt,
                &permalink_structure,
            ),
        ),
        FeedFormat::Atom => (
            "application/atom+xml; charset=utf-8",
            generate_atom(
                site_title,
                site_tagline,
                site_url,
                locale,
                &feed_url,
                &page.items,
                rss_use_excerpt,
                &permalink_structure,
            ),
        ),
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
        .body(Body::from(xml))
        .map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))
}

pub async fn feed_root_rss(
    State(state): State<AppState>,
    uri: OriginalUri,
    Query(params): Query<FeedParams>,
) -> ApiResult<Response> {
    let format = if params.format.as_deref() == Some("atom") {
        FeedFormat::Atom
    } else {
        FeedFormat::Rss
    };
    handle_feed(state, "fr", format, &uri.to_string()).await
}

pub async fn feed_root_atom(
    State(state): State<AppState>,
    uri: OriginalUri,
) -> ApiResult<Response> {
    handle_feed(state, "fr", FeedFormat::Atom, &uri.to_string()).await
}

pub async fn feed_locale_rss(
    State(state): State<AppState>,
    Path(locale): Path<String>,
    uri: OriginalUri,
    Query(params): Query<FeedParams>,
) -> ApiResult<Response> {
    let format = if params.format.as_deref() == Some("atom") {
        FeedFormat::Atom
    } else {
        FeedFormat::Rss
    };
    handle_feed(state, &locale, format, &uri.to_string()).await
}

pub async fn feed_locale_atom(
    State(state): State<AppState>,
    Path(locale): Path<String>,
    uri: OriginalUri,
) -> ApiResult<Response> {
    handle_feed(state, &locale, FeedFormat::Atom, &uri.to_string()).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use rustforge_core::models::EntryStatus;
    use serde_json::json;
    use uuid::Uuid;

    fn sample_entry(title: &str, slug: &str, body: &str, excerpt: &str) -> Entry {
        Entry {
            id: Uuid::new_v4(),
            content_type: "post".into(),
            content_type_id: Uuid::new_v4(),
            parent_id: None,
            author_id: Some(Uuid::new_v4()),
            author_name: Some("Jean Auteur & Cie".into()),
            title: title.into(),
            slug: slug.into(),
            excerpt: excerpt.into(),
            body: body.into(),
            body_html: format!("<p>{body}</p>"),
            blocks: json!([]),
            intent: json!({}),
            design_tokens: json!({}),
            status: EntryStatus::Published,
            fields: json!({}),
            featured_media_id: None,
            featured_media_url: None,
            published_at: Some(Utc.with_ymd_and_hms(2026, 10, 28, 12, 0, 0).unwrap()),
            scheduled_unpublish_at: None,
            created_at: Utc.with_ymd_and_hms(2026, 10, 28, 10, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 10, 28, 12, 0, 0).unwrap(),
            terms: Vec::new(),
        }
    }

    #[test]
    fn test_rss_xml_structure_and_escaping() {
        let entries = vec![
            sample_entry("Titre <Test> & \"Guillemets\"", "article-1", "Contenu de l'article", "Court résumé"),
        ];
        let xml = generate_rss("Site & Blog", "Mon Slogan <Super>", "https://example.com", "fr", "https://example.com/fr/feed", &entries, false, "plain");

        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains("<rss version=\"2.0\""));
        assert!(xml.contains("<title>Site &amp; Blog</title>"));
        assert!(xml.contains("<description>Mon Slogan &lt;Super&gt;</description>"));
        assert!(xml.contains("<title>Titre &lt;Test&gt; &amp; &quot;Guillemets&quot;</title>"));
        assert!(xml.contains("<link>https://example.com/fr/post/article-1</link>"));
        assert!(xml.contains("<dc:creator>Jean Auteur &amp; Cie</dc:creator>"));
        assert!(xml.contains("<description><![CDATA[Court résumé]]></description>"));
        assert!(xml.contains("<content:encoded><![CDATA[<p>Contenu de l'article</p>]]></content:encoded>"));
    }

    #[test]
    fn test_rss_excerpt_only_omits_content_encoded() {
        let entries = vec![
            sample_entry("Article", "slug", "Grand contenu secret", "Court résumé"),
        ];
        let xml = generate_rss("Site", "", "https://example.com", "fr", "https://example.com/feed", &entries, true, "plain");

        assert!(xml.contains("<description><![CDATA[Court résumé]]></description>"));
        assert!(!xml.contains("<content:encoded>"));
        assert!(!xml.contains("Grand contenu secret"));
    }

    #[test]
    fn test_atom_xml_structure_and_excerpt() {
        let entries = vec![
            sample_entry("Titre Atom", "atom-slug", "Corps Atom", "Résumé Atom"),
        ];
        let xml = generate_atom("Site Atom", "Sous-titre", "https://example.com", "fr", "https://example.com/feed/atom", &entries, false, "plain");

        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"utf-8\"?>"));
        assert!(xml.contains("<feed xmlns=\"http://www.w3.org/2005/Atom\">"));
        assert!(xml.contains("<title>Site Atom</title>"));
        assert!(xml.contains("<subtitle>Sous-titre</subtitle>"));
        assert!(xml.contains("<summary type=\"html\"><![CDATA[Résumé Atom]]></summary>"));
        assert!(xml.contains("<content type=\"html\"><![CDATA[<p>Corps Atom</p>]]></content>"));
    }

    #[test]
    fn test_cdata_split_injection_prevention() {
        let dangerous = "Ceci contient ]]> un piège XML";
        let cdata = xml_cdata(dangerous);
        assert_eq!(cdata, "<![CDATA[Ceci contient ]]]]><![CDATA[> un piège XML]]>");
    }
}
