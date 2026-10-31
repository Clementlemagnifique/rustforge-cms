//! Ingesteur WXR (WordPress eXtended RSS) streaming ultra-rapide basé sur `quick-xml`.
//! Analyse les exports XML officiels de WordPress et peuple le schéma pivot TMIR.

use chrono::{DateTime, Utc};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::collections::BTreeMap;
use std::io::BufRead;

use super::php_serializer::maybe_deserialize_php;
use super::tmir::{TMIRDocument, TMIREntry, TMIREntryStatus, TMIRMedia, TMIRTerm};
use crate::error::CoreError;

pub struct WxrImporter;

impl WxrImporter {
    /// Ingestion streaming d'un document XML WXR depuis un reader.
    pub fn parse<R: BufRead>(reader: R) -> Result<TMIRDocument, CoreError> {
        let mut xml_reader = Reader::from_reader(reader);
        xml_reader.config_mut().trim_text(true);

        let mut doc = TMIRDocument::default();
        let mut document_json = String::new();
        let mut in_document = false;
        let mut buf = Vec::new();

        let mut in_item = false;
        let mut in_category_term = false;
        let mut in_tag_term = false;
        let mut in_postmeta = false;

        let mut current_tag = String::new();
        let mut current_category_domain = String::new();
        let mut current_category_nicename = String::new();

        // Variables d'état pour l'item en cours
        let mut item_id = String::new();
        let mut item_title = String::new();
        let mut item_slug = String::new();
        let mut item_content = String::new();
        let mut item_excerpt = String::new();
        let mut item_creator = String::new();
        let mut item_date = String::new();
        let mut item_status = String::new();
        let mut item_type = "post".to_string();
        let mut item_categories = Vec::new();
        let mut item_tags = Vec::new();
        let mut item_vault = BTreeMap::new();
        let mut item_attachment_url = String::new();

        // Variables pour meta
        let mut current_meta_key = String::new();
        let mut current_meta_val = String::new();

        // Variables pour termes au niveau global
        let mut term_slug = String::new();
        let mut term_name = String::new();
        let mut term_parent = String::new();

        loop {
            match xml_reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    current_tag = tag_name.clone();
                    if tag_name == "rf:document" {
                        in_document = true;
                    }

                    if tag_name == "item" || tag_name == "rf:item" {
                        in_item = true;
                        item_id.clear();
                        item_title.clear();
                        item_slug.clear();
                        item_content.clear();
                        item_excerpt.clear();
                        item_creator.clear();
                        item_date.clear();
                        item_status.clear();
                        item_type = "post".to_string();
                        item_categories.clear();
                        item_tags.clear();
                        item_vault.clear();
                        item_attachment_url.clear();
                    } else if tag_name == "wp:postmeta"
                        || tag_name == "rf:postmeta"
                        || tag_name == "rf:meta"
                    {
                        in_postmeta = true;
                        current_meta_key.clear();
                        current_meta_val.clear();
                    } else if (tag_name == "wp:category"
                        || tag_name == "rf:category"
                        || tag_name == "rf:term")
                        && !in_item
                    {
                        in_category_term = true;
                        term_slug.clear();
                        term_name.clear();
                        term_parent.clear();
                    } else if (tag_name == "wp:tag" || tag_name == "rf:tag") && !in_item {
                        in_tag_term = true;
                        term_slug.clear();
                        term_name.clear();
                    } else if tag_name == "category" && in_item {
                        current_category_domain.clear();
                        current_category_nicename.clear();
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            let val = String::from_utf8_lossy(&attr.value).to_string();
                            if key == "domain" {
                                current_category_domain = val;
                            } else if key == "nicename" {
                                current_category_nicename = val;
                            }
                        }
                    }
                }
                Ok(Event::Text(ref e)) => {
                    let decoded = String::from_utf8_lossy(e.as_ref());
                    let text = quick_xml::escape::unescape(&decoded)
                        .map_err(|e| CoreError::validation(e.to_string()))?
                        .into_owned();
                    if in_document {
                        document_json.push_str(&text);
                        continue;
                    }
                    Self::process_text_node(
                        &text,
                        &current_tag,
                        in_item,
                        in_postmeta,
                        in_category_term,
                        in_tag_term,
                        &current_category_domain,
                        &mut current_meta_key,
                        &mut current_meta_val,
                        &mut item_title,
                        &mut item_creator,
                        &mut item_content,
                        &mut item_excerpt,
                        &mut item_id,
                        &mut item_attachment_url,
                        &mut item_date,
                        &mut item_slug,
                        &mut item_status,
                        &mut item_type,
                        &mut item_categories,
                        &mut item_tags,
                        &mut term_slug,
                        &mut term_name,
                        &mut term_parent,
                        &mut doc.source_site_url,
                    );
                }
                Ok(Event::CData(ref e)) => {
                    let text = String::from_utf8_lossy(e.as_ref()).to_string();
                    if in_document {
                        document_json.push_str(&text);
                        continue;
                    }
                    Self::process_text_node(
                        &text,
                        &current_tag,
                        in_item,
                        in_postmeta,
                        in_category_term,
                        in_tag_term,
                        &current_category_domain,
                        &mut current_meta_key,
                        &mut current_meta_val,
                        &mut item_title,
                        &mut item_creator,
                        &mut item_content,
                        &mut item_excerpt,
                        &mut item_id,
                        &mut item_attachment_url,
                        &mut item_date,
                        &mut item_slug,
                        &mut item_status,
                        &mut item_type,
                        &mut item_categories,
                        &mut item_tags,
                        &mut term_slug,
                        &mut term_name,
                        &mut term_parent,
                        &mut doc.source_site_url,
                    );
                }
                Ok(Event::End(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                    if tag_name == "rf:document" {
                        let decoded: TMIRDocument = serde_json::from_str(&document_json)
                            .map_err(|e| CoreError::validation(e.to_string()))?;
                        if decoded.tmir_version != super::tmir::RFIR_CURRENT_VERSION {
                            return Err(CoreError::validation("version RFIR non prise en charge"));
                        }
                        return Ok(decoded);
                    }
                    if tag_name == "wp:postmeta"
                        || tag_name == "rf:postmeta"
                        || tag_name == "rf:meta"
                    {
                        in_postmeta = false;
                        if !current_meta_key.is_empty() {
                            // Désérialisation automatique des structures PHP en JSONB
                            let deserialized_val = maybe_deserialize_php(&current_meta_val);
                            item_vault.insert(current_meta_key.clone(), deserialized_val);
                        }
                    } else if tag_name == "item" || tag_name == "rf:item" {
                        in_item = false;

                        // Si c'est un média / attachment
                        if item_type == "attachment" {
                            let source_url = if !item_attachment_url.trim().is_empty() {
                                item_attachment_url.trim().to_string()
                            } else {
                                item_vault
                                    .get("_rf_attached_file")
                                    .or_else(|| item_vault.get("_wp_attached_file"))
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string()
                            };

                            let filename = if let Some(last_slash) = source_url.rfind('/') {
                                source_url[last_slash + 1..].to_string()
                            } else if !item_title.trim().is_empty() {
                                item_title.trim().to_string()
                            } else {
                                format!("attachment-{}.jpg", item_id)
                            };

                            let mime_type = if filename.ends_with(".png") {
                                "image/png".to_string()
                            } else if filename.ends_with(".webp") {
                                "image/webp".to_string()
                            } else if filename.ends_with(".svg") {
                                "image/svg+xml".to_string()
                            } else {
                                "image/jpeg".to_string()
                            };

                            doc.media.push(TMIRMedia {
                                source_id: item_id.clone(),
                                source_url,
                                filename,
                                mime_type,
                                title: Some(item_title.clone()),
                                alt_text: item_vault
                                    .get("_wp_attachment_image_alt")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string()),
                                parent_entry_id: None,
                            });
                        } else {
                            // Conversion du statut
                            let status = match item_status.as_str() {
                                "publish" | "published" => TMIREntryStatus::Published,
                                "draft" => TMIREntryStatus::Draft,
                                "pending" => TMIREntryStatus::Pending,
                                "private" => TMIREntryStatus::Private,
                                "trash" => TMIREntryStatus::Trash,
                                "archived" => TMIREntryStatus::Archived,
                                _ => TMIREntryStatus::Draft,
                            };

                            let slug = if item_slug.trim().is_empty() {
                                slug::slugify(&item_title)
                            } else {
                                item_slug.trim().to_string()
                            };

                            let published_at = DateTime::parse_from_rfc3339(item_date.trim())
                                .map(|dt| dt.with_timezone(&Utc))
                                .or_else(|_| {
                                    DateTime::parse_from_str(
                                        &format!("{} +0000", item_date.trim()),
                                        "%Y-%m-%d %H:%M:%S %z",
                                    )
                                    .map(|dt| dt.with_timezone(&Utc))
                                })
                                .ok();

                            doc.entries.push(TMIREntry {
                                source_id: item_id.clone(),
                                slug,
                                title: item_title.clone(),
                                content_type: item_type.clone(),
                                status,
                                raw_content: item_content.clone(),
                                excerpt: if item_excerpt.is_empty() {
                                    None
                                } else {
                                    Some(item_excerpt.clone())
                                },
                                author_name: if item_creator.is_empty() {
                                    None
                                } else {
                                    Some(item_creator.clone())
                                },
                                author_email: None,
                                published_at,
                                categories: item_categories.clone(),
                                tags: item_tags.clone(),
                                featured_image_url: None,
                                legacy_vault: item_vault.clone(),
                            });
                        }
                    } else if (tag_name == "wp:category"
                        || tag_name == "rf:category"
                        || tag_name == "rf:term")
                        && !in_item
                    {
                        in_category_term = false;
                        if !term_slug.is_empty() {
                            doc.terms.push(TMIRTerm {
                                source_id: term_slug.clone(),
                                taxonomy: "category".to_string(),
                                slug: term_slug.clone(),
                                name: if term_name.is_empty() {
                                    term_slug.clone()
                                } else {
                                    term_name.clone()
                                },
                                description: None,
                                parent_slug: if term_parent.is_empty() {
                                    None
                                } else {
                                    Some(term_parent.clone())
                                },
                            });
                        }
                    } else if (tag_name == "wp:tag" || tag_name == "rf:tag") && !in_item {
                        in_tag_term = false;
                        if !term_slug.is_empty() {
                            doc.terms.push(TMIRTerm {
                                source_id: term_slug.clone(),
                                taxonomy: "post_tag".to_string(),
                                slug: term_slug.clone(),
                                name: if term_name.is_empty() {
                                    term_slug.clone()
                                } else {
                                    term_name.clone()
                                },
                                description: None,
                                parent_slug: None,
                            });
                        }
                    }
                    current_tag.clear();
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(CoreError::Validation(format!("Erreur XML WXR: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        Ok(doc)
    }

    #[allow(clippy::too_many_arguments)]
    fn process_text_node(
        text: &str,
        current_tag: &str,
        in_item: bool,
        in_postmeta: bool,
        in_category_term: bool,
        in_tag_term: bool,
        current_category_domain: &str,
        current_meta_key: &mut String,
        current_meta_val: &mut String,
        item_title: &mut String,
        item_creator: &mut String,
        item_content: &mut String,
        item_excerpt: &mut String,
        item_id: &mut String,
        item_attachment_url: &mut String,
        item_date: &mut String,
        item_slug: &mut String,
        item_status: &mut String,
        item_type: &mut String,
        item_categories: &mut Vec<String>,
        item_tags: &mut Vec<String>,
        term_slug: &mut String,
        term_name: &mut String,
        term_parent: &mut String,
        source_site_url: &mut Option<String>,
    ) {
        if in_item {
            if in_postmeta {
                if current_tag == "wp:meta_key"
                    || current_tag == "rf:meta_key"
                    || current_tag == "rf:key"
                {
                    current_meta_key.push_str(text);
                } else if current_tag == "wp:meta_value"
                    || current_tag == "rf:meta_value"
                    || current_tag == "rf:value"
                {
                    current_meta_val.push_str(text);
                }
            } else {
                match current_tag {
                    "title" | "rf:title" => item_title.push_str(text),
                    "dc:creator" | "rf:author" => item_creator.push_str(text),
                    "content:encoded" | "rf:content" => item_content.push_str(text),
                    "excerpt:encoded" | "rf:excerpt" => item_excerpt.push_str(text),
                    "wp:post_id" | "rf:post_id" | "rf:entry_id" | "rf:id" => item_id.push_str(text),
                    "wp:attachment_url"
                    | "rf:attachment_url"
                    | "rf:media_url"
                    | "rf:featured_media_url"
                    | "rf:url" => {
                        item_attachment_url.push_str(text);
                    }
                    "guid" if item_attachment_url.is_empty() => item_attachment_url.push_str(text),
                    "wp:post_date" | "wp:post_date_gmt" | "rf:post_date" | "rf:published_at"
                        if item_date.is_empty() =>
                    {
                        item_date.push_str(text);
                    }
                    "wp:post_name" | "rf:post_name" | "rf:slug" => item_slug.push_str(text),
                    "wp:status" | "rf:status" => item_status.push_str(text),
                    "wp:post_type" | "rf:post_type" | "rf:content_type" => {
                        *item_type = text.trim().to_string()
                    }
                    "category" => {
                        if current_category_domain == "category"
                            || current_category_domain == "rf_category"
                        {
                            item_categories.push(text.trim().to_string());
                        } else if current_category_domain == "post_tag"
                            || current_category_domain == "tag"
                            || current_category_domain == "rf_tag"
                        {
                            item_tags.push(text.trim().to_string());
                        }
                    }
                    "rf:category" => {
                        if !text.trim().is_empty() {
                            item_categories.push(text.trim().to_string());
                        }
                    }
                    "rf:tag" if !text.trim().is_empty() => {
                        item_tags.push(text.trim().to_string());
                    }
                    _ => {}
                }
            }
        } else if in_category_term {
            match current_tag {
                "wp:category_nicename" | "rf:category_nicename" | "rf:slug" => {
                    term_slug.push_str(text)
                }
                "wp:cat_name" | "rf:cat_name" | "rf:name" => term_name.push_str(text),
                "wp:category_parent" | "rf:category_parent" | "rf:parent" => {
                    term_parent.push_str(text)
                }
                _ => {}
            }
        } else if in_tag_term {
            match current_tag {
                "wp:tag_slug" | "rf:tag_slug" | "rf:slug" => term_slug.push_str(text),
                "wp:tag_name" | "rf:tag_name" | "rf:name" => term_name.push_str(text),
                _ => {}
            }
        } else {
            // En-tête global channel
            if (current_tag == "link" || current_tag == "rf:site_url")
                && source_site_url.is_none()
                && !text.trim().is_empty()
            {
                *source_site_url = Some(text.trim().to_string());
            }
        }
    }
}
