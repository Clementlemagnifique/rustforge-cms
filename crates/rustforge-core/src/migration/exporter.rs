//! Moteur d'exportation souverain RustForge (DataMorph Exporter).
//! Permet l'exportation intégrale de la base de données vers :
//! 1. Le document pivot RFIR (JSON formatté).
//! 2. Le standard XML souverain RustForge (RF-XML) avec balises <rf:*>, <!-- rf:* --> et coffre-fort _rf_vault.

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use std::collections::BTreeMap;
use uuid::Uuid;

use crate::CoreError;
use super::tmir::{RFIRDocument, RFIREntry, RFIREntryStatus, RFIRMedia, RFIRTerm};

pub struct DataMorphExporter;

impl DataMorphExporter {
    /// Extrait l'ensemble des données d'un site RustForge sous forme de document pivot RFIRDocument.
    pub async fn dump_database(pool: &PgPool) -> Result<RFIRDocument, CoreError> {
        let mut doc = RFIRDocument::default();
        doc.source_generator = "RustForge Sovereign Exporter 1.0".to_string();

        // 1. Termes et taxonomies
        let term_rows: Vec<(Uuid, String, String, String, Option<String>)> = sqlx::query_as(
            "SELECT t.id, tx.slug as taxonomy, t.name, t.slug, t.description
             FROM terms t
             JOIN taxonomies tx ON tx.id = t.taxonomy_id
             ORDER BY tx.slug, t.name",
        )
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        for (id, tax, name, slug, desc) in term_rows {
            doc.terms.push(RFIRTerm {
                source_id: id.to_string(),
                taxonomy: tax,
                slug,
                name,
                description: desc,
                parent_slug: None,
            });
        }

        // 2. Médias
        let media_rows: Vec<(Uuid, String, String, String, Option<String>)> = sqlx::query_as(
            "SELECT id, filename, path, mime_type, alt
             FROM media
             ORDER BY created_at ASC",
        )
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        for (id, filename, path, mime_type, alt) in media_rows {
            let url = format!("/uploads/{}", path);
            doc.media.push(RFIRMedia {
                source_id: id.to_string(),
                source_url: url,
                filename,
                mime_type,
                title: None,
                alt_text: alt,
                parent_entry_id: None,
            });
        }

        // 3. Entrées
        let entry_rows: Vec<(
            Uuid,
            String,
            String,
            String,
            String,
            Option<String>,
            String,
            Option<String>,
            Option<String>,
            Option<DateTime<Utc>>,
            Option<String>,
            Value,
        )> = sqlx::query_as(
            "SELECT e.id, ct.slug as content_type, e.title, e.slug, e.body, e.excerpt,
                    e.status::text, u.display_name as author_name, u.email as author_email,
                    e.published_at, ('/uploads/' || m.path) as featured_media_url, e.fields
             FROM entries e
             JOIN content_types ct ON ct.id = e.content_type_id
             LEFT JOIN users u ON u.id = e.author_id
             LEFT JOIN media m ON m.id = e.featured_media_id
             WHERE e.status <> 'trash'
             ORDER BY e.created_at ASC",
        )
        .fetch_all(pool)
        .await
        .map_err(CoreError::Database)?;

        for (
            id,
            ct,
            title,
            slug,
            body,
            excerpt,
            status_str,
            author_name,
            author_email,
            published_at,
            featured_media_url,
            fields,
        ) in entry_rows
        {
            let status = match status_str.as_str() {
                "published" => RFIREntryStatus::Published,
                "pending" => RFIREntryStatus::Pending,
                "private" => RFIREntryStatus::Private,
                "trash" => RFIREntryStatus::Trash,
                _ => RFIREntryStatus::Draft,
            };

            // Extraction des catégories
            let mut categories = Vec::new();
            if let Some(arr) = fields
                .get("rf_categories")
                .or_else(|| fields.get("categories"))
                .and_then(|v| v.as_array())
            {
                for c in arr {
                    if let Some(s) = c.as_str() {
                        categories.push(s.to_string());
                    }
                }
            }

            // Extraction des tags
            let mut tags = Vec::new();
            if let Some(arr) = fields
                .get("rf_tags")
                .or_else(|| fields.get("tags"))
                .and_then(|v| v.as_array())
            {
                for t in arr {
                    if let Some(s) = t.as_str() {
                        tags.push(s.to_string());
                    }
                }
            }

            // Extraction du coffre-fort _rf_vault
            let mut legacy_vault = BTreeMap::new();
            if let Some(obj) = fields
                .get("_rf_vault")
                .or_else(|| fields.get("_legacy_vault"))
                .and_then(|v| v.as_object())
            {
                for (k, v) in obj {
                    legacy_vault.insert(k.clone(), v.clone());
                }
            }

            doc.entries.push(RFIREntry {
                source_id: id.to_string(),
                slug,
                title,
                content_type: ct,
                status,
                raw_content: body,
                excerpt,
                author_name,
                author_email,
                published_at,
                categories,
                tags,
                featured_image_url: featured_media_url,
                legacy_vault,
            });
        }

        Ok(doc)
    }

    /// Exporte le document pivot en JSON formatté.
    pub fn to_json(doc: &RFIRDocument) -> Result<String, CoreError> {
        serde_json::to_string_pretty(doc).map_err(|e| CoreError::Validation(e.to_string()))
    }

    /// Exporte le document pivot au standard XML souverain RustForge (RF-XML).
    pub fn to_rf_xml(doc: &RFIRDocument) -> Result<String, CoreError> {
        let mut out = String::with_capacity(16 * 1024);
        out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        out.push_str("<rustforge version=\"1.0\" xmlns:rf=\"https://rustforge.dev/export/1.0/\">\n");
        out.push_str("  <rf:channel>\n");
        out.push_str(&format!("    <rf:generator>{}</rf:generator>\n", xml_escape(&doc.source_generator)));
        out.push_str(&format!("    <rf:generated_at>{}</rf:generated_at>\n", doc.generated_at.to_rfc3339()));
        if let Some(ref url) = doc.source_site_url {
            out.push_str(&format!("    <rf:site_url>{}</rf:site_url>\n", xml_escape(url)));
        }

        // 1. Termes
        if !doc.terms.is_empty() {
            out.push_str("    <rf:terms>\n");
            for term in &doc.terms {
                out.push_str("      <rf:term>\n");
                out.push_str(&format!("        <rf:id>{}</rf:id>\n", term.source_id));
                out.push_str(&format!("        <rf:taxonomy>{}</rf:taxonomy>\n", xml_escape(&term.taxonomy)));
                out.push_str(&format!("        <rf:slug>{}</rf:slug>\n", xml_escape(&term.slug)));
                out.push_str(&format!("        <rf:name><![CDATA[{}]]></rf:name>\n", term.name));
                if let Some(ref desc) = term.description {
                    out.push_str(&format!("        <rf:description><![CDATA[{}]]></rf:description>\n", desc));
                }
                out.push_str("      </rf:term>\n");
            }
            out.push_str("    </rf:terms>\n");
        }

        // 2. Médias
        if !doc.media.is_empty() {
            out.push_str("    <rf:media_list>\n");
            for m in &doc.media {
                out.push_str("      <rf:media>\n");
                out.push_str(&format!("        <rf:id>{}</rf:id>\n", m.source_id));
                out.push_str(&format!("        <rf:url>{}</rf:url>\n", xml_escape(&m.source_url)));
                out.push_str(&format!("        <rf:filename>{}</rf:filename>\n", xml_escape(&m.filename)));
                out.push_str(&format!("        <rf:mime_type>{}</rf:mime_type>\n", xml_escape(&m.mime_type)));
                if let Some(ref alt) = m.alt_text {
                    out.push_str(&format!("        <rf:alt><![CDATA[{}]]></rf:alt>\n", alt));
                }
                out.push_str("      </rf:media>\n");
            }
            out.push_str("    </rf:media_list>\n");
        }

        // 3. Entrées
        out.push_str("    <rf:items>\n");
        for entry in &doc.entries {
            out.push_str("      <rf:item>\n");
            out.push_str(&format!("        <rf:id>{}</rf:id>\n", entry.source_id));
            out.push_str(&format!("        <rf:title><![CDATA[{}]]></rf:title>\n", entry.title));
            out.push_str(&format!("        <rf:slug>{}</rf:slug>\n", xml_escape(&entry.slug)));
            out.push_str(&format!("        <rf:content_type>{}</rf:content_type>\n", xml_escape(&entry.content_type)));
            let status_str = match entry.status {
                RFIREntryStatus::Published => "published",
                RFIREntryStatus::Draft => "draft",
                RFIREntryStatus::Pending => "pending",
                RFIREntryStatus::Private => "private",
                RFIREntryStatus::Trash => "trash",
            };
            out.push_str(&format!("        <rf:status>{}</rf:status>\n", status_str));

            if let Some(ref author) = entry.author_name {
                out.push_str(&format!("        <rf:author><![CDATA[{}]]></rf:author>\n", author));
            }
            if let Some(dt) = entry.published_at {
                out.push_str(&format!("        <rf:published_at>{}</rf:published_at>\n", dt.to_rfc3339()));
            }
            if let Some(ref excerpt) = entry.excerpt {
                out.push_str(&format!("        <rf:excerpt><![CDATA[{}]]></rf:excerpt>\n", excerpt));
            }

            // Contenu (les blocs utilisent <!-- rf:* -->)
            out.push_str(&format!("        <rf:content><![CDATA[{}]]></rf:content>\n", entry.raw_content));

            // Catégories
            if !entry.categories.is_empty() {
                out.push_str("        <rf:categories>\n");
                for cat in &entry.categories {
                    out.push_str(&format!("          <rf:category><![CDATA[{}]]></rf:category>\n", cat));
                }
                out.push_str("        </rf:categories>\n");
            }

            // Tags
            if !entry.tags.is_empty() {
                out.push_str("        <rf:tags>\n");
                for tag in &entry.tags {
                    out.push_str(&format!("          <rf:tag><![CDATA[{}]]></rf:tag>\n", tag));
                }
                out.push_str("        </rf:tags>\n");
            }

            // Image mise en avant
            if let Some(ref media_url) = entry.featured_image_url {
                out.push_str(&format!("        <rf:featured_media_url>{}</rf:featured_media_url>\n", xml_escape(media_url)));
            }

            // Coffre-fort numérique _rf_vault
            if !entry.legacy_vault.is_empty() {
                let vault_json = serde_json::to_string(&entry.legacy_vault).unwrap_or_else(|_| "{}".to_string());
                out.push_str(&format!("        <rf:vault><![CDATA[{}]]></rf:vault>\n", vault_json));
            }

            out.push_str("      </rf:item>\n");
        }
        out.push_str("    </rf:items>\n");

        out.push_str("  </rf:channel>\n");
        out.push_str("</rustforge>\n");

        Ok(out)
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
