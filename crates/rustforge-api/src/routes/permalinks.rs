//! Gestionnaire et calcul souverain des permaliens (WP-90, WP-91).
//!
//! Assure la conformité avec les structures de permaliens WordPress :
//! - Simple (`plain`) : `/{locale}/{content_type}/{slug}`
//! - Date et titre (`day_and_name`) : `/{locale}/%year%/%monthnum%/%day%/%postname%/`
//! - Mois et titre (`month_and_name`) : `/{locale}/%year%/%monthnum%/%postname%/`
//! - Numérique (`numeric`) : `/{locale}/archives/%post_id%`
//! - Nom de l'article (`post_name`) : `/{locale}/post/%postname%/`
//! - Personnalisée (`custom`) : combinaisons autorisées avec %postname% ou %post_id%
//!
//! Gère également les préfixes personnalisés des taxonomies (`category_base`, `tag_base`)
//! et les redirections permanentes HTTP 308 sans chaîne ni boucle.

use chrono::Datelike;
use rustforge_core::{models::Entry, settings};
use sqlx::PgPool;

/// Calcule le permalien relatif d'une entrée selon la structure configurée.
pub fn compute_entry_permalink(entry: &Entry, structure: &str, locale: &str) -> String {
    let date = entry.published_at.unwrap_or(entry.created_at);
    let author = entry.author_name.as_deref();
    let category = entry
        .terms
        .iter()
        .find(|t| t.taxonomy == "category")
        .map(|t| t.slug.as_str());

    compute_permalink_parts(
        &entry.content_type,
        &entry.slug,
        date,
        entry.id,
        author,
        category,
        structure,
        locale,
    )
}

/// Calcule le permalien relatif à partir des composants individuels d'une publication.
#[allow(clippy::too_many_arguments)]
pub fn compute_permalink_parts(
    content_type: &str,
    slug: &str,
    date: chrono::DateTime<chrono::Utc>,
    id: uuid::Uuid,
    author: Option<&str>,
    category: Option<&str>,
    structure: &str,
    locale: &str,
) -> String {
    if content_type == "page" {
        return format!("/{locale}/page/{slug}");
    }

    let year = date.year();
    let month = date.month();
    let day = date.day();

    match structure.trim() {
        "day_and_name" => {
            format!("/{locale}/{year:04}/{month:02}/{day:02}/{slug}")
        }
        "month_and_name" => {
            format!("/{locale}/{year:04}/{month:02}/{slug}")
        }
        "numeric" => {
            format!("/{locale}/archives/{id}")
        }
        "post_name" => {
            format!("/{locale}/post/{slug}")
        }
        s if s.starts_with('/') => {
            let mut path = s.to_string();
            path = path.replace("%year%", &format!("{year:04}"));
            path = path.replace("%monthnum%", &format!("{month:02}"));
            path = path.replace("%day%", &format!("{day:02}"));
            path = path.replace("%post_id%", &id.to_string());
            path = path.replace("%postname%", slug);

            if let Some(author) = author {
                let safe_author = rustforge_core::slugify(author);
                path = path.replace("%author%", &safe_author);
            } else {
                path = path.replace("%author%", "admin");
            }

            let cat = category.unwrap_or("general");
            path = path.replace("%category%", cat);

            let normalized = path.trim_end_matches('/');
            format!("/{locale}{normalized}")
        }
        // "plain" ou valeur par défaut
        _ => format!("/{locale}/post/{slug}"),
    }
}

/// Calcule le permalien relatif d'une archive de taxonomie (catégorie ou étiquette).
pub fn compute_term_permalink(
    taxonomy: &str,
    term_slug: &str,
    category_base: Option<&str>,
    tag_base: Option<&str>,
    locale: &str,
) -> String {
    let base = match taxonomy {
        "category" => category_base
            .filter(|s| !s.is_empty())
            .unwrap_or("category"),
        "tag" => tag_base.filter(|s| !s.is_empty()).unwrap_or("tag"),
        other => other,
    };
    format!("/{locale}/archive/{base}/{term_slug}")
}

/// Charge la configuration des permaliens depuis les réglages.
pub async fn load_permalink_settings(db: &PgPool) -> (String, String, String) {
    let structure = settings::get(db, "permalink.structure")
        .await
        .ok()
        .flatten()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_else(|| "plain".into());

    let category_base = settings::get(db, "permalink.category_base")
        .await
        .ok()
        .flatten()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default();

    let tag_base = settings::get(db, "permalink.tag_base")
        .await
        .ok()
        .flatten()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default();

    (structure, category_base, tag_base)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use rustforge_core::models::{Entry, EntryStatus};
    use uuid::Uuid;

    fn sample_entry(content_type: &str, slug: &str) -> Entry {
        let dt = chrono::Utc
            .with_ymd_and_hms(2026, 10, 28, 14, 30, 0)
            .unwrap();
        let id = Uuid::parse_str("a0000000-0000-0000-0000-000000000001").unwrap();
        Entry {
            id,
            content_type_id: Uuid::nil(),
            content_type: content_type.to_string(),
            parent_id: None,
            title: "Article de test".into(),
            slug: slug.to_string(),
            body: "Corps de l'article".into(),
            body_html: "<p>Corps de l'article</p>".into(),
            excerpt: "Extrait".into(),
            blocks: serde_json::json!([]),
            intent: serde_json::json!({}),
            design_tokens: serde_json::json!({}),
            status: EntryStatus::Published,
            author_id: None,
            author_name: Some("Jean Dupont".into()),
            featured_media_id: None,
            featured_media_url: None,
            featured_media_alt: None,
            featured_media_caption: None,
            fields: serde_json::json!({}),
            published_at: Some(dt),
            scheduled_unpublish_at: None,
            locale: "fr".into(),
            translation_group_id: None,
            created_at: dt,
            updated_at: dt,
            terms: vec![],
        }
    }

    #[test]
    fn test_compute_entry_permalink_structures() {
        let post = sample_entry("post", "mon-super-article");

        assert_eq!(
            compute_entry_permalink(&post, "plain", "fr"),
            "/fr/post/mon-super-article"
        );
        assert_eq!(
            compute_entry_permalink(&post, "post_name", "fr"),
            "/fr/post/mon-super-article"
        );
        assert_eq!(
            compute_entry_permalink(&post, "day_and_name", "fr"),
            "/fr/2026/10/28/mon-super-article"
        );
        assert_eq!(
            compute_entry_permalink(&post, "month_and_name", "fr"),
            "/fr/2026/10/mon-super-article"
        );
        assert_eq!(
            compute_entry_permalink(&post, "numeric", "fr"),
            "/fr/archives/a0000000-0000-0000-0000-000000000001"
        );

        // Structure personnalisée
        assert_eq!(
            compute_entry_permalink(&post, "/blog/%year%/%postname%/", "fr"),
            "/fr/blog/2026/mon-super-article"
        );
    }

    #[test]
    fn test_page_permalink_always_retains_page_structure() {
        let page = sample_entry("page", "contact");

        assert_eq!(
            compute_entry_permalink(&page, "day_and_name", "fr"),
            "/fr/page/contact"
        );
        assert_eq!(
            compute_entry_permalink(&page, "numeric", "fr"),
            "/fr/page/contact"
        );
    }

    #[test]
    fn test_compute_term_permalink_with_custom_bases() {
        assert_eq!(
            compute_term_permalink("category", "actualites", None, None, "fr"),
            "/fr/archive/category/actualites"
        );
        assert_eq!(
            compute_term_permalink("category", "actualites", Some("rubrique"), None, "fr"),
            "/fr/archive/rubrique/actualites"
        );
        assert_eq!(
            compute_term_permalink("tag", "rust", None, Some("mot-cle"), "fr"),
            "/fr/archive/mot-cle/rust"
        );
    }
}
