//! Moteur de persistance DataMorph vers PostgreSQL.
//! Applique un document TMIR vers les tables `entries`, `terms`, `taxonomies`, `content_types`
//! et stocke l'intégralité des champs méta spécifiques dans `_legacy_vault`.

use sqlx::PgPool;
use std::collections::HashMap;
use std::time::Instant;
use uuid::Uuid;

use super::tmir::{TMIRDocument, TMIREntryStatus, TMIRImportReport};
use crate::error::CoreResult;

pub struct DataMorphImporter;

impl DataMorphImporter {
    /// Applique un document TMIR dans la base de données PostgreSQL de RustForge.
    pub async fn apply(
        db: &PgPool,
        doc: &TMIRDocument,
        default_author_id: Uuid,
    ) -> CoreResult<TMIRImportReport> {
        if doc.tmir_version != super::tmir::RFIR_CURRENT_VERSION {
            return Err(crate::CoreError::validation("Version RFIR incompatible"));
        }
        let source = doc
            .source_site_url
            .as_deref()
            .filter(|v| !v.trim().is_empty())
            .ok_or_else(|| {
                crate::CoreError::validation("URL source requise pour identifier cet import")
            })?;
        let mut ids = std::collections::HashSet::new();
        for e in &doc.entries {
            if e.source_id.is_empty() || !ids.insert((&e.content_type, &e.source_id)) {
                return Err(crate::CoreError::validation(
                    "Identifiant source vide ou dupliqué",
                ));
            }
        }
        let mut tx = db.begin().await?;
        sqlx::query("SELECT pg_advisory_xact_lock(79643201)")
            .execute(&mut *tx)
            .await?;
        let start = Instant::now();
        let mut report = TMIRImportReport::default();

        // 1. Découverte et création dynamique des Types de Contenus (Content Types) manquants
        let mut content_type_ids: HashMap<String, Uuid> = HashMap::new();
        let mut encountered_types = std::collections::HashSet::new();
        for entry in &doc.entries {
            encountered_types.insert(entry.content_type.clone());
        }

        for ctype in encountered_types {
            let existing_id: Option<Uuid> =
                sqlx::query_scalar("SELECT id FROM content_types WHERE slug = $1")
                    .bind(&ctype)
                    .fetch_optional(&mut *tx)
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
                    .fetch_one(&mut *tx)
                    .await?;

                    report.created_content_types.push(ctype.clone());
                    new_id
                }
            };

            content_type_ids.insert(ctype, ct_id);
        }

        // S'assurer que le type "post" par défaut est résolu si présent
        if !content_type_ids.contains_key("post")
            && let Ok(Some(id)) =
                sqlx::query_scalar::<_, Uuid>("SELECT id FROM content_types WHERE slug = 'post'")
                    .fetch_optional(&mut *tx)
                    .await
        {
            content_type_ids.insert("post".to_string(), id);
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
                    let tid: Uuid = match sqlx::query_scalar::<_, Uuid>(
                        "SELECT id FROM taxonomies WHERE slug = $1",
                    )
                    .bind(&tax_slug)
                    .fetch_optional(&mut *tx)
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
                            .fetch_one(&mut *tx)
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
                   ON CONFLICT (taxonomy_id, slug) DO UPDATE SET name = EXCLUDED.name"#,
            )
            .bind(tax_id)
            .bind(&term.name)
            .bind(&term.slug)
            .bind(term_desc)
            .execute(&mut *tx)
            .await;

            let r = res?;
            report.terms_imported += r.rows_affected() as usize;
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

            crate::security::relative_path(&path)?;
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
            .fetch_one(&mut *tx)
            .await;

            match media_res {
                Ok(mid) => {
                    report.media_imported += 1;
                    media_id_by_source_id.insert(m.source_id.clone(), mid);
                    if !m.source_url.is_empty() {
                        media_id_by_source_url.insert(m.source_url.clone(), mid);
                    }
                }
                Err(err) => return Err(err.into()),
            }
        }

        // 3. Ingestion des entrées avec préservation absolue dans _legacy_vault
        for entry in &doc.entries {
            let ct_id = match content_type_ids.get(&entry.content_type) {
                Some(&id) => id,
                None => {
                    report.warnings.push(format!(
                        "Type de contenu introuvable pour l'entrée '{}'",
                        entry.title
                    ));
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
                TMIREntryStatus::Archived => "archived",
            };
            // A historical WordPress date is not a TrustPress scheduling instruction.
            // In particular, private/pending content must never be auto-published later.
            let published_at = if matches!(entry.status, TMIREntryStatus::Published) {
                Some(entry.published_at.unwrap_or_else(chrono::Utc::now))
            } else {
                None
            };

            let origin:Option<(Uuid,chrono::DateTime<chrono::Utc>)>=sqlx::query_as("SELECT entry_id,target_updated_at FROM import_origins WHERE source=$1 AND entity_type=$2 AND source_id=$3")
                .bind(source).bind(&entry.content_type).bind(&entry.source_id).fetch_optional(&mut *tx).await?;
            if let Some((id, stamp)) = origin {
                let current: (chrono::DateTime<chrono::Utc>, String) =
                    sqlx::query_as("SELECT updated_at,slug FROM entries WHERE id=$1 FOR UPDATE")
                        .bind(id)
                        .fetch_one(&mut *tx)
                        .await?;
                if current.0 != stamp || current.1 != entry.slug {
                    return Err(crate::CoreError::Conflict(format!(
                        "Le contenu '{}' a été modifié depuis son import. Aucun écrasement automatique.",
                        entry.slug
                    )));
                }
            } else {
                let collision: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM entries WHERE content_type_id=$1 AND slug=$2)",
                )
                .bind(ct_id)
                .bind(&entry.slug)
                .fetch_one(&mut *tx)
                .await?;
                if collision {
                    return Err(crate::CoreError::Conflict(format!(
                        "Slug déjà occupé sans correspondance source : {}",
                        entry.slug
                    )));
                }
            }
            // Résolution de l'image mise en avant
            let featured_media_id: Option<Uuid> = {
                if let Some(ref url) = entry.featured_image_url {
                    media_id_by_source_url.get(url).copied()
                } else {
                    None
                }
            }
            .or_else(|| {
                entry
                    .legacy_vault
                    .get("_thumbnail_id")
                    .and_then(|v| {
                        if let Some(s) = v.as_str() {
                            Some(s.to_string())
                        } else {
                            v.as_i64().map(|n| n.to_string())
                        }
                    })
                    .and_then(|tid| media_id_by_source_id.get(&tid).copied())
            });

            // Construction du payload fields enrichi du coffre-fort _rf_vault / _legacy_vault
            let mut fields_map = doc
                .metadata
                .get(&format!("entry_fields:{}", entry.source_id))
                .and_then(serde_json::Value::as_object)
                .cloned()
                .unwrap_or_default();
            if let Some(author) = &entry.author_name {
                fields_map.insert(
                    "_source_author".into(),
                    serde_json::json!({"name":author,"email":entry.author_email}),
                );
            }
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

            fields_map.entry("_source_publication".to_string()).or_insert_with(||serde_json::json!({"status":entry.status,"published_at":entry.published_at}));
            let fields_json = serde_json::Value::Object(fields_map);
            let excerpt_str = entry.excerpt.as_deref().unwrap_or("");
            let blocks = doc.metadata.get(&format!("entry_blocks:{}", entry.source_id))
                .cloned().unwrap_or_else(|| serde_json::json!([]));

            // Transmutation souveraine des blocs : <!-- wp:* --> -> <!-- rf:* -->
            let clean_raw_content = entry
                .raw_content
                .replace("<!-- wp:", "<!-- rf:")
                .replace("<!-- /wp:", "<!-- /rf:");

            let (clean_raw_content, body_html) = if blocks.get("rf_schema").is_some() {
                crate::document::project(&clean_raw_content, &blocks)?
            } else {
                let html = if clean_raw_content.contains('<') {
                    ammonia::clean(&clean_raw_content)
                } else {
                    crate::markdown::render(&clean_raw_content)
                };
                (clean_raw_content, html)
            };

            let res = sqlx::query_scalar::<_, Uuid>(
                r#"INSERT INTO entries (
                    content_type_id, author_id, title, slug, excerpt, body, body_html,
                    blocks, intent, design_tokens, status, fields, featured_media_id, published_at, created_at, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $12, '{}'::jsonb, '{}'::jsonb, $8::entry_status, $9, $10, $11, now(), now())
                ON CONFLICT (content_type_id, slug) DO UPDATE SET
                    title = EXCLUDED.title,
                    excerpt = EXCLUDED.excerpt,
                    body = EXCLUDED.body,
                    body_html = EXCLUDED.body_html,
                    blocks = EXCLUDED.blocks,
                    fields = EXCLUDED.fields,
                    status = EXCLUDED.status, published_at = EXCLUDED.published_at,
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
            .bind(published_at)
            .bind(blocks)
            .fetch_one(&mut *tx)
            .await;

            match res {
                Ok(entry_id) => {
                    report.entries_imported += 1;
                    sqlx::query("INSERT INTO import_origins(source,entity_type,source_id,entry_id,target_updated_at) SELECT $1,$2,$3,id,updated_at FROM entries WHERE id=$4 ON CONFLICT(source,entity_type,source_id) DO UPDATE SET target_updated_at=EXCLUDED.target_updated_at")
                        .bind(source).bind(&entry.content_type).bind(&entry.source_id).bind(entry_id).execute(&mut *tx).await?;

                    // Association dynamique des termes (Catégories et Tags) dans entry_terms
                    sqlx::query("DELETE FROM entry_terms WHERE entry_id=$1 AND term_id IN (SELECT t.id FROM terms t JOIN taxonomies tx ON tx.id=t.taxonomy_id WHERE tx.slug IN ('category','tag'))")
                        .bind(entry_id).execute(&mut *tx).await?;
                    for cat in &entry.categories {
                        let term_slug = crate::slugify(cat);
                        let linked = sqlx::query(
                            r#"INSERT INTO entry_terms (entry_id, term_id)
                               SELECT $1, t.id FROM terms t
                               JOIN taxonomies tx ON tx.id = t.taxonomy_id
                               WHERE tx.slug = 'category' AND (t.slug = $2 OR t.name = $3)
                               ON CONFLICT DO NOTHING"#,
                        )
                        .bind(entry_id)
                        .bind(&term_slug)
                        .bind(cat)
                        .execute(&mut *tx)
                        .await?;
                        if linked.rows_affected() == 0 {
                            return Err(crate::CoreError::validation(
                                "Terme source introuvable ; import annulé",
                            ));
                        }
                    }

                    for tag in &entry.tags {
                        let term_slug = crate::slugify(tag);
                        let linked = sqlx::query(
                            r#"INSERT INTO entry_terms (entry_id, term_id)
                               SELECT $1, t.id FROM terms t
                               JOIN taxonomies tx ON tx.id = t.taxonomy_id
                               WHERE tx.slug = 'tag' AND (t.slug = $2 OR t.name = $3)
                               ON CONFLICT DO NOTHING"#,
                        )
                        .bind(entry_id)
                        .bind(&term_slug)
                        .bind(tag)
                        .execute(&mut *tx)
                        .await?;
                        if linked.rows_affected() == 0 {
                            return Err(crate::CoreError::validation(
                                "Terme source introuvable ; import annulé",
                            ));
                        }
                    }
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }

        if report.media_imported > 0 {
            report.warnings.push("Références médias enregistrées. Les fichiers WXR doivent être transférés et vérifiés séparément.".into());
        }
        report.warnings.push("Les auteurs source sont conservés en provenance ; les contenus sont attribués à l'administrateur importateur.".into());
        tx.commit().await?;
        report.execution_time_ms = start.elapsed().as_millis() as u64;

        Ok(report)
    }
}
