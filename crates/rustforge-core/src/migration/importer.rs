//! Moteur de persistance DataMorph vers PostgreSQL.
//! Applique un document TMIR vers les tables `entries`, `terms`, `taxonomies`, `content_types`
//! et stocke l'intégralité des champs méta spécifiques dans `_legacy_vault`.

use std::collections::HashMap;
use std::time::Instant;
use sqlx::PgPool;
use uuid::Uuid;

use super::tmir::{TMIRDocument, TMIRImportReport, TMIREntryStatus};
use crate::error::CoreResult;

pub struct DataMorphImporter;

impl DataMorphImporter {
    /// Applique un document TMIR dans la base de données PostgreSQL de RustForge.
    pub async fn apply(db: &PgPool, doc: &TMIRDocument, default_author_id: Uuid) -> CoreResult<TMIRImportReport> {
        let start = Instant::now();
        let mut report = TMIRImportReport::default();

        // 1. Découverte et création dynamique des Types de Contenus (Content Types) manquants
        let mut content_type_ids: HashMap<String, Uuid> = HashMap::new();
        let mut encountered_types = std::collections::HashSet::new();
        for entry in &doc.entries {
            encountered_types.insert(entry.content_type.clone());
        }

        for ctype in encountered_types {
            let existing_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM content_types WHERE slug = $1")
                .bind(&ctype)
                .fetch_optional(db)
                .await?;

            let ct_id = match existing_id {
                Some(id) => id,
                None => {
                    let (type_name, type_singular) = match ctype.as_str() {
                        "post" => ("Articles", "Article"),
                        "page" => ("Pages", "Page"),
                        "product" => ("Produits", "Produit"),
                        "course" => ("Formations", "Formation"),
                        other => (other, other),
                    };

                    let new_id: Uuid = sqlx::query_scalar(
                        r#"INSERT INTO content_types (
                            slug, name, name_singular, description, icon,
                            hierarchical, has_revisions, is_system, fields, created_at, updated_at
                        )
                        VALUES ($1, $2, $3, $4, 'file-text', false, true, false, '[]'::jsonb, now(), now())
                        ON CONFLICT (slug) DO UPDATE SET name = EXCLUDED.name
                        RETURNING id"#
                    )
                    .bind(&ctype)
                    .bind(type_name)
                    .bind(type_singular)
                    .bind(format!("Type de contenu importé automatiquement par DataMorph ({})", ctype))
                    .fetch_one(db)
                    .await?;

                    report.created_content_types.push(ctype.clone());
                    new_id
                }
            };

            content_type_ids.insert(ctype, ct_id);
        }

        // S'assurer que le type "post" par défaut est résolu si présent
        if !content_type_ids.contains_key("post") {
            if let Ok(Some(id)) = sqlx::query_scalar::<_, Uuid>("SELECT id FROM content_types WHERE slug = 'post'").fetch_optional(db).await {
                content_type_ids.insert("post".to_string(), id);
            }
        }

        // 2. Ingestion des termes de taxonomies (catégories, tags, custom taxonomies)
        let mut taxonomy_ids: HashMap<String, Uuid> = HashMap::new();
        for term in &doc.terms {
            let tax_slug = if term.taxonomy == "post_tag" {
                "tag".to_string()
            } else {
                term.taxonomy.clone()
            };

            let tax_id = match taxonomy_ids.get(&tax_slug) {
                Some(&id) => id,
                None => {
                    let tid: Uuid = match sqlx::query_scalar::<_, Uuid>("SELECT id FROM taxonomies WHERE slug = $1")
                        .bind(&tax_slug)
                        .fetch_optional(db)
                        .await?
                    {
                        Some(id) => id,
                        None => {
                            let tax_name = match tax_slug.as_str() {
                                "category" => "Catégories",
                                "tag" => "Étiquettes",
                                other => other,
                            };

                            sqlx::query_scalar::<_, Uuid>(
                                r#"INSERT INTO taxonomies (slug, name, hierarchical, is_system, created_at)
                                   VALUES ($1, $2, false, false, now())
                                   ON CONFLICT (slug) DO UPDATE SET name = EXCLUDED.name
                                   RETURNING id"#
                            )
                            .bind(&tax_slug)
                            .bind(tax_name)
                            .fetch_one(db)
                            .await?
                        }
                    };
                    taxonomy_ids.insert(tax_slug.clone(), tid);
                    tid
                }
            };

            let term_desc = term.description.as_deref().unwrap_or("");
            let res = sqlx::query(
                r#"INSERT INTO terms (taxonomy_id, name, slug, description, created_at)
                   VALUES ($1, $2, $3, $4, now())
                   ON CONFLICT (taxonomy_id, slug) DO UPDATE SET name = EXCLUDED.name"#
            )
            .bind(tax_id)
            .bind(&term.name)
            .bind(&term.slug)
            .bind(term_desc)
            .execute(db)
            .await;

            if let Ok(r) = res {
                if r.rows_affected() > 0 {
                    report.terms_imported += 1;
                }
            }
        }

        // 2b. Ingestion réelle des Médias et mappage des pièces jointes
        let mut media_id_by_source_id: HashMap<String, Uuid> = HashMap::new();
        let mut media_id_by_source_url: HashMap<String, Uuid> = HashMap::new();

        report.media_detected = doc.media.len();

        for m in &doc.media {
            let raw_path = if !m.source_url.trim().is_empty() {
                let clean = m.source_url.split('?').next().unwrap_or(&m.source_url);
                if let Some(pos) = clean.find("wp-content/uploads/") {
                    clean[pos + "wp-content/uploads/".len()..].to_string()
                } else if let Some(pos) = clean.find("rf-uploads/") {
                    clean[pos + "rf-uploads/".len()..].to_string()
                } else if let Some(pos) = clean.find("uploads/") {
                    clean[pos + "uploads/".len()..].to_string()
                } else if let Some(last_slash) = clean.rfind('/') {
                    clean[last_slash + 1..].to_string()
                } else {
                    clean.to_string()
                }
            } else if !m.filename.trim().is_empty() {
                m.filename.clone()
            } else {
                format!("imported-{}.bin", m.source_id)
            };

            let path = if raw_path.trim().is_empty() {
                format!("imported-{}.bin", m.source_id)
            } else {
                raw_path
            };

            let original_name = if !m.filename.trim().is_empty() {
                m.filename.clone()
            } else {
                path.clone()
            };

            let mime_type = if !m.mime_type.trim().is_empty() {
                m.mime_type.clone()
            } else if path.ends_with(".png") {
                "image/png".to_string()
            } else if path.ends_with(".webp") {
                "image/webp".to_string()
            } else if path.ends_with(".svg") {
                "image/svg+xml".to_string()
            } else {
                "image/jpeg".to_string()
            };

            let alt_text = m.alt_text.as_deref().unwrap_or("");
            let caption = m.title.as_deref().unwrap_or("");

            let media_res: Result<Uuid, sqlx::Error> = sqlx::query_scalar(
                r#"INSERT INTO media (
                    path, original_name, mime_type, size_bytes,
                    alt, caption, uploaded_by, external_provider, metadata
                )
                VALUES ($1, $2, $3, 0, $4, $5, $6, 'rf_legacy_import', json_build_object('source_id', $7, 'source_url', $8))
                ON CONFLICT (path) DO UPDATE SET
                    alt = CASE WHEN EXCLUDED.alt <> '' THEN EXCLUDED.alt ELSE media.alt END,
                    caption = CASE WHEN EXCLUDED.caption <> '' THEN EXCLUDED.caption ELSE media.caption END
                RETURNING id"#
            )
            .bind(&path)
            .bind(&original_name)
            .bind(&mime_type)
            .bind(alt_text)
            .bind(caption)
            .bind(default_author_id)
            .bind(&m.source_id)
            .bind(&m.source_url)
            .fetch_one(db)
            .await;

            match media_res {
                Ok(mid) => {
                    report.media_imported += 1;
                    media_id_by_source_id.insert(m.source_id.clone(), mid);
                    if !m.source_url.is_empty() {
                        media_id_by_source_url.insert(m.source_url.clone(), mid);
                    }
                }
                Err(_) => {
                    if let Ok(existing_id) = sqlx::query_scalar::<_, Uuid>("SELECT id FROM media WHERE path = $1")
                        .bind(&path)
                        .fetch_one(db)
                        .await
                    {
                        report.media_imported += 1;
                        media_id_by_source_id.insert(m.source_id.clone(), existing_id);
                        if !m.source_url.is_empty() {
                            media_id_by_source_url.insert(m.source_url.clone(), existing_id);
                        }
                    } else {
                        report.warnings.push(format!("Média ignoré : {}", path));
                    }
                }
            }
        }

        // 3. Ingestion des entrées avec préservation absolue dans _legacy_vault
        for entry in &doc.entries {
            let ct_id = match content_type_ids.get(&entry.content_type) {
                Some(&id) => id,
                None => {
                    report.warnings.push(format!("Type de contenu introuvable pour l'entrée '{}'", entry.title));
                    report.entries_skipped += 1;
                    continue;
                }
            };

            let status_str = match entry.status {
                TMIREntryStatus::Published => "published",
                TMIREntryStatus::Draft => "draft",
                TMIREntryStatus::Pending => "draft",
                TMIREntryStatus::Private => "draft",
                TMIREntryStatus::Trash => "trash",
            };

            // Résolution de l'image mise en avant
            let featured_media_id: Option<Uuid> = {
                if let Some(ref url) = entry.featured_image_url {
                    media_id_by_source_url.get(url).copied()
                } else {
                    None
                }
            }
            .or_else(|| {
                entry.legacy_vault
                    .get("_thumbnail_id")
                    .and_then(|v| {
                        if let Some(s) = v.as_str() {
                            Some(s.to_string())
                        } else if let Some(n) = v.as_i64() {
                            Some(n.to_string())
                        } else {
                            None
                        }
                    })
                    .and_then(|tid| media_id_by_source_id.get(&tid).copied())
            });

            // Construction du payload fields enrichi du coffre-fort _rf_vault / _legacy_vault
            let mut fields_map = serde_json::Map::new();
            if !entry.legacy_vault.is_empty() {
                let vault_val = serde_json::to_value(&entry.legacy_vault).unwrap_or_default();
                fields_map.insert("_rf_vault".to_string(), vault_val.clone());
                fields_map.insert("_legacy_vault".to_string(), vault_val);
                report.legacy_fields_stored_in_vault += entry.legacy_vault.len();
            }

            if !entry.categories.is_empty() {
                let cat_val = serde_json::to_value(&entry.categories).unwrap_or_default();
                fields_map.insert("rf_categories".to_string(), cat_val.clone());
                fields_map.insert("wp_categories".to_string(), cat_val);
            }

            if !entry.tags.is_empty() {
                let tag_val = serde_json::to_value(&entry.tags).unwrap_or_default();
                fields_map.insert("rf_tags".to_string(), tag_val.clone());
                fields_map.insert("wp_tags".to_string(), tag_val);
            }

            let fields_json = serde_json::Value::Object(fields_map);
            let excerpt_str = entry.excerpt.as_deref().unwrap_or("");
            
            // Transmutation souveraine des blocs : <!-- wp:* --> -> <!-- rf:* -->
            let clean_raw_content = entry.raw_content
                .replace("<!-- wp:", "<!-- rf:")
                .replace("<!-- /wp:", "<!-- /rf:");

            let body_html = if clean_raw_content.contains('<') {
                clean_raw_content.clone()
            } else {
                crate::markdown::render(&clean_raw_content)
            };

            let res = sqlx::query_scalar::<_, Uuid>(
                r#"INSERT INTO entries (
                    content_type_id, author_id, title, slug, excerpt, body, body_html,
                    blocks, intent, design_tokens, status, fields, featured_media_id, published_at, created_at, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, '[]'::jsonb, '{}'::jsonb, '{}'::jsonb, $8::entry_status, $9, $10, $11, now(), now())
                ON CONFLICT (content_type_id, slug) DO UPDATE SET
                    title = EXCLUDED.title,
                    excerpt = EXCLUDED.excerpt,
                    body = EXCLUDED.body,
                    body_html = EXCLUDED.body_html,
                    fields = EXCLUDED.fields,
                    featured_media_id = COALESCE(EXCLUDED.featured_media_id, entries.featured_media_id),
                    updated_at = now()
                RETURNING id"#
            )
            .bind(ct_id)
            .bind(default_author_id)
            .bind(&entry.title)
            .bind(&entry.slug)
            .bind(excerpt_str)
            .bind(&clean_raw_content)
            .bind(&body_html)
            .bind(status_str)
            .bind(fields_json)
            .bind(featured_media_id)
            .bind(entry.published_at)
            .fetch_one(db)
            .await;

            match res {
                Ok(entry_id) => {
                    report.entries_imported += 1;

                    // Association dynamique des termes (Catégories et Tags) dans entry_terms
                    for cat in &entry.categories {
                        let term_slug = crate::slugify(cat);
                        let _ = sqlx::query(
                            r#"INSERT INTO entry_terms (entry_id, term_id)
                               SELECT $1, t.id FROM terms t
                               JOIN taxonomies tx ON tx.id = t.taxonomy_id
                               WHERE tx.slug = 'category' AND (t.slug = $2 OR t.name = $3)
                               ON CONFLICT DO NOTHING"#
                        )
                        .bind(entry_id)
                        .bind(&term_slug)
                        .bind(cat)
                        .execute(db)
                        .await;
                    }

                    for tag in &entry.tags {
                        let term_slug = crate::slugify(tag);
                        let _ = sqlx::query(
                            r#"INSERT INTO entry_terms (entry_id, term_id)
                               SELECT $1, t.id FROM terms t
                               JOIN taxonomies tx ON tx.id = t.taxonomy_id
                               WHERE tx.slug = 'tag' AND (t.slug = $2 OR t.name = $3)
                               ON CONFLICT DO NOTHING"#
                        )
                        .bind(entry_id)
                        .bind(&term_slug)
                        .bind(tag)
                        .execute(db)
                        .await;
                    }
                }
                Err(e) => {
                    report.warnings.push(format!("Entrée ignorée '{}' ({}) : {}", entry.title, entry.slug, e));
                    report.entries_skipped += 1;
                }
            }
        }

        report.execution_time_ms = start.elapsed().as_millis() as u64;

        Ok(report)
    }
}
