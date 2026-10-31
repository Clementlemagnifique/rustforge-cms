//! Entrées : création, édition, révisions, corbeille, requêtes filtrées.

use std::collections::HashMap;

use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::{
    CoreError, CoreResult, HookRunner, Page, PageParams, content_types, fields,
    hooks::names,
    models::{ContentType, Entry, EntryInput, EntryStatus, Revision, TermRef},
};

const SELECT: &str =
    "SELECT e.id, e.content_type_id, ct.slug AS content_type, e.parent_id, e.author_id,
        u.display_name AS author_name, e.title, e.slug, e.excerpt, e.body, e.body_html,
        e.blocks, e.intent, e.design_tokens,
        e.status, e.fields,
        e.featured_media_id, CASE WHEN m.mime_type IN ('image/jpeg','image/png','image/gif','image/webp','image/avif','image/svg+xml') THEN ('/uploads/' || m.path) END AS featured_media_url,
        m.alt AS featured_media_alt, m.caption AS featured_media_caption,
        e.published_at, e.scheduled_unpublish_at, e.locale, e.translation_group_id, e.created_at, e.updated_at
    FROM entries e
    JOIN content_types ct ON ct.id = e.content_type_id
    LEFT JOIN users u ON u.id = e.author_id
    LEFT JOIN media m ON m.id = e.featured_media_id";

const MAX_REVISIONS: i64 = 30;

/// Règle unique de visibilité, indépendante du passage du planificateur.
const PUBLIC_VISIBILITY: &str = "e.status = 'published' AND e.published_at <= now() AND (e.scheduled_unpublish_at IS NULL OR e.scheduled_unpublish_at > now())";

/// Filtres de liste (`?content_type=post&status=draft&q=rust&term=actu&page=2`).
#[derive(Debug, Default, Clone, Deserialize)]
pub struct EntryQuery {
    pub content_type: Option<String>,
    pub status: Option<EntryStatus>,
    pub author_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    /// Slug d'un terme.
    pub term: Option<String>,
    /// Restreint `term` à une taxonomie.
    pub taxonomy: Option<String>,
    /// Recherche plein texte.
    pub q: Option<String>,
    /// `updated` (défaut admin), `published` (défaut public), `title`, `created`.
    pub order: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub locale: Option<String>,
    pub translation_group_id: Option<Uuid>,
    /// Restreint aux entrées publiées et visibles (API publique).
    #[serde(skip)]
    pub public_only: bool,
}

fn push_filters(qb: &mut QueryBuilder<'_, Postgres>, q: &EntryQuery) {
    qb.push(" WHERE TRUE");
    if let Some(ct) = q.content_type.as_ref().filter(|s| !s.is_empty()) {
        qb.push(" AND ct.slug = ").push_bind(ct.clone());
    }
    if q.public_only {
        qb.push(" AND ").push(PUBLIC_VISIBILITY);
    } else {
        match q.status {
            Some(status) => {
                qb.push(" AND e.status = ").push_bind(status);
            }
            None => {
                qb.push(" AND e.status <> 'trash'");
            }
        }
    }
    if let Some(locale) = q.locale.as_ref().filter(|s| !s.is_empty()) {
        qb.push(" AND e.locale = ").push_bind(locale.to_lowercase());
    }
    if let Some(tg) = q.translation_group_id {
        qb.push(" AND e.translation_group_id = ").push_bind(tg);
    }
    if let Some(author) = q.author_id {
        qb.push(" AND e.author_id = ").push_bind(author);
    }
    if let Some(parent) = q.parent_id {
        qb.push(" AND e.parent_id = ").push_bind(parent);
    }
    if let Some(term) = q.term.as_ref().filter(|s| !s.is_empty()) {
        qb.push(
            " AND EXISTS (SELECT 1 FROM entry_terms et JOIN terms t ON t.id = et.term_id
              JOIN taxonomies tx ON tx.id = t.taxonomy_id WHERE et.entry_id = e.id AND t.slug = ",
        )
        .push_bind(term.clone());
        if let Some(tax) = q.taxonomy.as_ref().filter(|s| !s.is_empty()) {
            qb.push(" AND tx.slug = ").push_bind(tax.clone());
        }
        qb.push(")");
    }
    if let Some(search) = q.q.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        qb.push(" AND (e.search @@ websearch_to_tsquery('simple', ")
            .push_bind(search.to_string())
            .push(") OR e.title ILIKE ")
            .push_bind(format!("%{search}%"))
            .push(")");
    }
}

pub async fn list(db: &PgPool, q: &EntryQuery) -> CoreResult<Page<Entry>> {
    let params = PageParams::new(q.page, q.per_page);

    let mut count_qb = QueryBuilder::new(
        "SELECT count(*) FROM entries e JOIN content_types ct ON ct.id = e.content_type_id",
    );
    push_filters(&mut count_qb, q);
    let total: i64 = count_qb.build_query_scalar().fetch_one(db).await?;

    let mut qb = QueryBuilder::new(SELECT);
    push_filters(&mut qb, q);
    let default_order = if q.public_only {
        "published"
    } else {
        "updated"
    };
    let order = match q.order.as_deref().unwrap_or(default_order) {
        "title" => " ORDER BY e.title ASC, e.id ASC",
        "created" => " ORDER BY e.created_at DESC, e.id DESC",
        "published" => " ORDER BY e.published_at DESC NULLS LAST, e.created_at DESC, e.id DESC",
        _ => " ORDER BY e.updated_at DESC, e.id DESC",
    };
    qb.push(order);
    qb.push(" LIMIT ").push_bind(params.limit());
    qb.push(" OFFSET ").push_bind(params.offset());

    let mut items: Vec<Entry> = qb.build_query_as().fetch_all(db).await?;
    attach_terms(db, &mut items).await?;
    Ok(Page::new(items, total, params))
}

pub async fn get(db: &PgPool, id: Uuid) -> CoreResult<Entry> {
    let mut entry = sqlx::query_as::<_, Entry>(&format!("{SELECT} WHERE e.id = $1"))
        .bind(id)
        .fetch_one(db)
        .await?;
    attach_terms(db, std::slice::from_mut(&mut entry)).await?;
    Ok(entry)
}

/// Page publique sélectionnée comme accueil ; aucune erreur SQL n'est masquée.
pub async fn get_published_page(db: &PgPool, id: Uuid) -> CoreResult<Entry> {
    let mut entry = sqlx::query_as::<_, Entry>(&format!(
        "{SELECT} WHERE e.id = $1 AND ct.slug = 'page' AND {PUBLIC_VISIBILITY}"
    ))
    .bind(id)
    .fetch_one(db)
    .await?;
    attach_terms(db, std::slice::from_mut(&mut entry)).await?;
    Ok(entry)
}

/// Entrée publiée et visible, par type et slug (API publique).
pub async fn get_published(db: &PgPool, content_type: &str, slug: &str) -> CoreResult<Entry> {
    let mut entry = sqlx::query_as::<_, Entry>(&format!(
        "{SELECT} WHERE ct.slug = $1 AND e.slug = $2 AND {PUBLIC_VISIBILITY}"
    ))
    .bind(content_type)
    .bind(slug)
    .fetch_one(db)
    .await?;
    attach_terms(db, std::slice::from_mut(&mut entry)).await?;
    Ok(entry)
}

/// Entrée publiée et visible, par identifiant UUID direct (permaliens numériques `archives/{id}`).
pub async fn get_published_by_id(db: &PgPool, id: Uuid) -> CoreResult<Entry> {
    let mut entry =
        sqlx::query_as::<_, Entry>(&format!("{SELECT} WHERE e.id = $1 AND {PUBLIC_VISIBILITY}"))
            .bind(id)
            .fetch_one(db)
            .await?;
    attach_terms(db, std::slice::from_mut(&mut entry)).await?;
    Ok(entry)
}

/// Si un slug historique (`_rf_vault.rf_old_slugs`) correspond à une entrée publiée dont le slug
/// canonique actuel est différent, retourne le slug canonique cible (résolution directe sans chaîne `A -> B -> C`).
pub async fn resolve_slug_redirect(
    db: &PgPool,
    content_type: &str,
    requested_slug: &str,
) -> CoreResult<Option<String>> {
    let target: Option<String> = sqlx::query_scalar(
        "SELECT e.slug
         FROM entries e
         JOIN content_types ct ON ct.id = e.content_type_id
         WHERE ct.slug = $1
           AND e.status = 'published'
           AND e.published_at <= now()
           AND (e.scheduled_unpublish_at IS NULL OR e.scheduled_unpublish_at > now())
           AND e.slug <> $2
           AND COALESCE(e.fields->'_rf_vault'->'rf_old_slugs', '[]'::jsonb) ? $2
         ORDER BY e.updated_at DESC
         LIMIT 1",
    )
    .bind(content_type)
    .bind(requested_slug)
    .fetch_optional(db)
    .await?;
    Ok(target)
}

/// Enregistre l'ancien slug dans `fields._rf_vault.rf_old_slugs` lors d'un renommage et retire
/// le nouveau slug de l'historique pour empêcher toute boucle de redirection (`A -> B -> A`).
fn record_slug_history(fields: &mut serde_json::Value, old_slug: &str, new_slug: &str) {
    if old_slug == new_slug || old_slug.is_empty() {
        return;
    }
    if !fields.is_object() {
        *fields = json!({});
    }
    let Some(root) = fields.as_object_mut() else {
        return;
    };
    let vault = root
        .entry("_rf_vault".to_string())
        .or_insert_with(|| json!({}));
    if !vault.is_object() {
        *vault = json!({});
    }
    let Some(vault_obj) = vault.as_object_mut() else {
        return;
    };
    let mut slugs: Vec<String> = vault_obj
        .get("rf_old_slugs")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    slugs.retain(|s| s != new_slug && s != old_slug);
    slugs.push(old_slug.to_string());
    if slugs.len() > 32 {
        let excess = slugs.len() - 32;
        slugs.drain(0..excess);
    }
    vault_obj.insert("rf_old_slugs".to_string(), json!(slugs));
}

async fn attach_terms(db: &PgPool, entries: &mut [Entry]) -> CoreResult<()> {
    if entries.is_empty() {
        return Ok(());
    }
    let ids: Vec<Uuid> = entries.iter().map(|e| e.id).collect();

    #[derive(sqlx::FromRow)]
    struct Row {
        entry_id: Uuid,
        #[sqlx(flatten)]
        term: TermRef,
    }

    let rows = sqlx::query_as::<_, Row>(
        "SELECT et.entry_id, t.id, t.name, t.slug, tx.slug AS taxonomy
         FROM entry_terms et
         JOIN terms t ON t.id = et.term_id
         JOIN taxonomies tx ON tx.id = t.taxonomy_id
         WHERE et.entry_id = ANY($1)
         ORDER BY t.name",
    )
    .bind(&ids)
    .fetch_all(db)
    .await?;

    let mut by_entry: HashMap<Uuid, Vec<TermRef>> = HashMap::new();
    for row in rows {
        by_entry.entry(row.entry_id).or_default().push(row.term);
    }
    for entry in entries.iter_mut() {
        // Also neutralize unsafe HTML imported before the consolidation. Raw source stays in body/vault.
        entry.body_html = rustforge_document::markdown::sanitize_html(&entry.body_html);
        entry.terms = by_entry.remove(&entry.id).unwrap_or_default();
    }
    Ok(())
}

/// Données validées et prêtes à l'écriture.
#[derive(serde::Serialize)]
struct Prepared {
    title: String,
    #[serde(rename = "slug")]
    slug_base: String,
    excerpt: String,
    body: String,
    body_html: String,
    blocks: serde_json::Value,
    intent: serde_json::Value,
    design_tokens: serde_json::Value,
    status: EntryStatus,
    fields: serde_json::Value,
    featured_media_id: Option<Uuid>,
    parent_id: Option<Uuid>,
    published_at: Option<chrono::DateTime<Utc>>,
    scheduled_unpublish_at: Option<chrono::DateTime<Utc>>,
    locale: String,
    translation_group_id: Option<Uuid>,
    term_ids: Option<Vec<Uuid>>,
}

/// Fusionne récursivement deux objets JSON sans écraser les clés non mentionnées.
/// Essentiel pour préserver `_rf_vault` et `_legacy_vault` (métadonnées, taxonomies, shortcodes)
/// lors des modifications de contenu depuis le Studio ou l'API.
pub fn merge_fields(base: &serde_json::Value, update: &serde_json::Value) -> serde_json::Value {
    match (base, update) {
        (serde_json::Value::Object(base_map), serde_json::Value::Object(update_map)) => {
            let mut merged = base_map.clone();
            for (k, v) in update_map {
                if let Some(base_val) = merged.get_mut(k)
                    && base_val.is_object()
                    && v.is_object()
                {
                    *base_val = merge_fields(base_val, v);
                    continue;
                }
                merged.insert(k.clone(), v.clone());
            }
            serde_json::Value::Object(merged)
        }
        (_, update) if !update.is_null() => update.clone(),
        (base, _) => base.clone(),
    }
}

async fn apply_before_save(hooks: &dyn HookRunner, input: EntryInput) -> EntryInput {
    let Ok(payload) = serde_json::to_string(&input) else {
        return input;
    };
    let filtered = hooks.filter(names::ENTRY_BEFORE_SAVE, payload).await;
    match serde_json::from_str::<EntryInput>(&filtered) {
        Ok(out) => out,
        Err(err) => {
            tracing::warn!(%err, "le filtre entry.before_save a renvoyé un JSON invalide : ignoré");
            input
        }
    }
}

/// Reserved document settings remain small and validated even through older clients/imports.
fn validate_document_settings(fields: &serde_json::Value) -> CoreResult<()> {
    if let Some(value) = fields.get("_rf_comment_status")
        && !value.is_null()
        && !matches!(value.as_str(), Some("open" | "closed"))
    {
        return Err(CoreError::validation(
            "Discussion : choisissez héritage, ouverte ou fermée",
        ));
    }
    if let Some(value) = fields.get("_rf_template")
        && !value.is_null()
        && !matches!(value.as_str(), Some("default" | "wide" | "landing"))
    {
        return Err(CoreError::validation("Modèle de contenu invalide"));
    }
    Ok(())
}

fn prepare(
    ct: &ContentType,
    input: EntryInput,
    existing_published_at: Option<chrono::DateTime<Utc>>,
    existing_fields: Option<&serde_json::Value>,
) -> CoreResult<Prepared> {
    let title = input.title.trim().to_string();
    if title.is_empty() {
        return Err(CoreError::validation("le titre est obligatoire"));
    }
    if title.chars().count() > 300 {
        return Err(CoreError::validation(
            "le titre ne doit pas dépasser 300 caractères",
        ));
    }
    let slug_base = crate::slugify(
        input
            .slug
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(&title),
    );

    // Si des champs existants existent (mise à jour), on fusionne pour ne jamais écraser _legacy_vault
    let combined_fields = match existing_fields {
        Some(base) => merge_fields(base, &input.fields),
        None => input.fields,
    };
    validate_document_settings(&combined_fields)?;
    let fields = fields::validate_values(&ct.fields, &combined_fields)?;

    // Gestion rigoureuse de published_at :
    // - Si statut Published : on conserve ou initialise avec la date actuelle.
    // - Si statut Draft : on ne conserve published_at QUE si c'est explicitement planifié dans le FUTUR.
    //   Si published_at était dans le passé (article remis en brouillon), le mettre à None
    //   empêche le scheduler de fond de le republier automatiquement toutes les 30 secondes !
    // - Si statut Archived ou Trash : None.
    let now = Utc::now();
    let published_at = match input.status {
        EntryStatus::Published => Some(input.published_at.or(existing_published_at).unwrap_or(now)),
        EntryStatus::Draft => match input.published_at {
            Some(dt) if dt > now => Some(dt),
            _ => None,
        },
        EntryStatus::Archived | EntryStatus::Trash => None,
    };

    let term_ids = input.term_ids.map(|mut ids| {
        ids.sort();
        ids.dedup();
        ids
    });

    let locale = if input.locale.trim().is_empty() {
        "fr".to_string()
    } else {
        input.locale.trim().to_lowercase()
    };
    let translation_group_id = input.translation_group_id;

    let (body, body_html) = crate::document::project(&input.body, &input.blocks)?;
    Ok(Prepared {
        title,
        slug_base,
        excerpt: input.excerpt.trim().to_string(),
        body_html,
        body,
        blocks: input.blocks,
        intent: input.intent,
        design_tokens: input.design_tokens,
        status: input.status,
        fields,
        featured_media_id: input.featured_media_id,
        parent_id: if ct.hierarchical {
            input.parent_id
        } else {
            None
        },
        published_at,
        scheduled_unpublish_at: input.scheduled_unpublish_at,
        locale,
        translation_group_id,
        term_ids,
    })
}

/// Trouve un slug libre pour ce type (`mon-article`, `mon-article-2`, …).
async fn unique_slug(
    db: &PgPool,
    content_type_id: Uuid,
    base: &str,
    exclude: Option<Uuid>,
) -> CoreResult<String> {
    let taken: Vec<String> = sqlx::query_scalar(
        "SELECT slug FROM entries
         WHERE content_type_id = $1 AND (slug = $2 OR slug LIKE $2 || '-%') AND ($3::uuid IS NULL OR id <> $3)",
    )
    .bind(content_type_id)
    .bind(base)
    .bind(exclude)
    .fetch_all(db)
    .await?;
    if !taken.iter().any(|s| s == base) {
        return Ok(base.to_string());
    }
    let mut n = 2;
    loop {
        let candidate = format!("{base}-{n}");
        if !taken.contains(&candidate) {
            return Ok(candidate);
        }
        n += 1;
    }
}

async fn write_terms(
    tx: &mut sqlx::PgConnection,
    entry_id: Uuid,
    term_ids: &[Uuid],
) -> CoreResult<()> {
    sqlx::query("DELETE FROM entry_terms WHERE entry_id = $1")
        .bind(entry_id)
        .execute(&mut *tx)
        .await?;
    if !term_ids.is_empty() {
        sqlx::query("INSERT INTO entry_terms (entry_id, term_id) SELECT $1, unnest($2::uuid[]) ON CONFLICT DO NOTHING")
            .bind(entry_id)
            .bind(term_ids)
            .execute(&mut *tx)
            .await?;
    }
    Ok(())
}

async fn write_revision(
    tx: &mut sqlx::PgConnection,
    entry_id: Uuid,
    author_id: Uuid,
    p: &Prepared,
) -> CoreResult<()> {
    sqlx::query("INSERT INTO revisions (entry_id, author_id, title, excerpt, body, fields, snapshot) VALUES ($1, $2, $3, $4, $5, $6, $7)")
        .bind(entry_id)
        .bind(author_id)
        .bind(&p.title)
        .bind(&p.excerpt)
        .bind(&p.body)
        .bind(&p.fields)
        .bind(serde_json::to_value(p).map_err(|e| CoreError::Internal(e.into()))?)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "DELETE FROM revisions WHERE entry_id = $1 AND id NOT IN
           (SELECT id FROM revisions WHERE entry_id = $1 ORDER BY created_at DESC LIMIT $2)",
    )
    .bind(entry_id)
    .bind(MAX_REVISIONS)
    .execute(&mut *tx)
    .await?;
    Ok(())
}

async fn notify_published(hooks: &dyn HookRunner, entry: &Entry) {
    let payload = json!({
        "id": entry.id,
        "content_type": entry.content_type,
        "title": entry.title,
        "slug": entry.slug,
    });
    hooks
        .action(names::ENTRY_PUBLISHED, payload.to_string())
        .await;
}

async fn validate_parent_hierarchy(
    db: &PgPool,
    content_type_id: Uuid,
    entry_id: Option<Uuid>,
    parent_id: Option<Uuid>,
) -> CoreResult<()> {
    let Some(mut cursor) = parent_id else {
        return Ok(());
    };
    if entry_id == Some(cursor) {
        return Err(CoreError::validation(
            "une entrée ne peut pas être son propre parent",
        ));
    }
    let mut visited = std::collections::HashSet::new();
    if let Some(self_id) = entry_id {
        visited.insert(self_id);
    }
    for _ in 0..32 {
        if !visited.insert(cursor) {
            return Err(CoreError::validation(
                "cycle détecté dans la hiérarchie des pages parentes",
            ));
        }
        let row: Option<(Uuid, Option<Uuid>)> =
            sqlx::query_as("SELECT content_type_id, parent_id FROM entries WHERE id = $1")
                .bind(cursor)
                .fetch_optional(db)
                .await?;
        let Some((parent_ct, next_parent)) = row else {
            return Err(CoreError::validation("page parente introuvable"));
        };
        if parent_ct != content_type_id {
            return Err(CoreError::validation(
                "la page parente doit appartenir au même type de contenu",
            ));
        }
        match next_parent {
            Some(next) => cursor = next,
            None => return Ok(()),
        }
    }
    Err(CoreError::validation(
        "profondeur maximale de hiérarchie dépassée",
    ))
}

/// Validate new featured references while keeping unchanged legacy references editable.
async fn validate_featured_image(db: &PgPool, id: Option<Uuid>) -> CoreResult<()> {
    let Some(id) = id else { return Ok(()) };
    let row: Option<(String, String)> =
        sqlx::query_as("SELECT mime_type,path FROM media WHERE id=$1")
            .bind(id)
            .fetch_optional(db)
            .await?;
    let Some((mime, path)) = row else {
        return Err(CoreError::validation("Image mise en avant introuvable"));
    };
    if !matches!(
        mime.as_str(),
        "image/jpeg" | "image/png" | "image/gif" | "image/webp" | "image/avif" | "image/svg+xml"
    ) {
        return Err(CoreError::validation(
            "Sélectionnez une image dans la médiathèque",
        ));
    }
    crate::security::relative_path(&path)?;
    Ok(())
}

pub async fn create(
    db: &PgPool,
    hooks: &dyn HookRunner,
    content_type: &str,
    author_id: Uuid,
    input: EntryInput,
) -> CoreResult<Entry> {
    let ct = content_types::get_by_slug(db, content_type).await?;
    let input = apply_before_save(hooks, input).await;
    let mut p = prepare(&ct, input, None, None)?;
    validate_featured_image(db, p.featured_media_id).await?;
    validate_parent_hierarchy(db, ct.id, None, p.parent_id).await?;
    let slug = unique_slug(db, ct.id, &p.slug_base, None).await?;

    p.slug_base = slug.clone();

    // Application des défauts éditoriaux (WP-75)
    if ct.slug == "post" {
        // Catégorie par défaut si aucun terme n'a été spécifié
        if p.term_ids.as_ref().is_none_or(|terms| terms.is_empty()) {
            let def_cat_slug = crate::settings::get(db, "writing.default_category")
                .await?
                .and_then(|v| v.as_str().map(str::to_string))
                .unwrap_or_else(|| "non-classe".to_string());
            let default_cat_id: Option<Uuid> = sqlx::query_scalar(
                "SELECT t.id FROM terms t JOIN taxonomies tx ON tx.id = t.taxonomy_id WHERE tx.slug = 'category' AND (t.slug = $1 OR t.slug = 'non-classe' OR t.slug = 'uncategorized') LIMIT 1"
            )
            .bind(def_cat_slug)
            .fetch_optional(db)
            .await?;
            if let Some(cat_id) = default_cat_id {
                p.term_ids = Some(vec![cat_id]);
            }
        }

        // Format d'article par défaut si non spécifié
        if let Some(obj) = p.fields.as_object_mut()
            && !obj.contains_key("post_format")
        {
            let def_format = crate::settings::get(db, "writing.default_post_format")
                .await?
                .and_then(|v| v.as_str().map(str::to_string))
                .unwrap_or_else(|| "standard".to_string());
            obj.insert("post_format".to_string(), serde_json::json!(def_format));
        }
    }

    let mut tx = db.begin().await?;
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO entries (content_type_id, parent_id, author_id, title, slug, excerpt, body, body_html,
            blocks, intent, design_tokens,
            status, fields, featured_media_id, published_at, scheduled_unpublish_at,
            locale, translation_group_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18) RETURNING id",
    )
    .bind(ct.id)
    .bind(p.parent_id)
    .bind(author_id)
    .bind(&p.title)
    .bind(&slug)
    .bind(&p.excerpt)
    .bind(&p.body)
    .bind(&p.body_html)
    .bind(&p.blocks)
    .bind(&p.intent)
    .bind(&p.design_tokens)
    .bind(p.status)
    .bind(&p.fields)
    .bind(p.featured_media_id)
    .bind(p.published_at)
    .bind(p.scheduled_unpublish_at)
    .bind(&p.locale)
    .bind(p.translation_group_id)
    .fetch_one(&mut *tx)
    .await?;
    if let Some(terms) = &p.term_ids {
        write_terms(&mut tx, id, terms).await?;
    }
    if ct.has_revisions {
        write_revision(&mut tx, id, author_id, &p).await?;
    }
    tx.commit().await?;

    let entry = get(db, id).await?;
    if entry.status == EntryStatus::Published {
        notify_published(hooks, &entry).await;
    }
    Ok(entry)
}

pub async fn update(
    db: &PgPool,
    hooks: &dyn HookRunner,
    id: Uuid,
    editor_id: Uuid,
    input: EntryInput,
) -> CoreResult<Entry> {
    update_internal(db, hooks, id, editor_id, input, false).await
}

async fn update_internal(
    db: &PgPool,
    hooks: &dyn HookRunner,
    id: Uuid,
    editor_id: Uuid,
    input: EntryInput,
    replace_fields: bool,
) -> CoreResult<Entry> {
    let current = get(db, id).await?;
    if input
        .expected_updated_at
        .is_some_and(|expected| expected != current.updated_at)
    {
        return Err(CoreError::Conflict(
            "Ce contenu a changé. Rechargez-le avant de réessayer.".into(),
        ));
    }
    let actor = crate::users::get(db, editor_id).await?;
    crate::security::require_entry_access(actor.role, actor.id, &current)?;
    let ct = content_types::get(db, current.content_type_id).await?;
    let input = apply_before_save(hooks, input).await;
    let mut p = prepare(
        &ct,
        input,
        current.published_at,
        if replace_fields {
            None
        } else {
            Some(&current.fields)
        },
    )?;
    if p.featured_media_id != current.featured_media_id {
        validate_featured_image(db, p.featured_media_id).await?;
    }
    if p.term_ids.is_none() {
        p.term_ids = Some(current.terms.iter().map(|t| t.id).collect());
    }
    validate_parent_hierarchy(db, ct.id, Some(id), p.parent_id).await?;
    let slug = unique_slug(db, ct.id, &p.slug_base, Some(id)).await?;
    p.slug_base = slug.clone();
    if current.slug != slug {
        record_slug_history(&mut p.fields, &current.slug, &slug);
    }

    let mut tx = db.begin().await?;
    let changed = sqlx::query(
        "UPDATE entries SET parent_id = $2, title = $3, slug = $4, excerpt = $5, body = $6, body_html = $7,
            blocks = $8, intent = $9, design_tokens = $10,
            status = $11, fields = $12, featured_media_id = $13, published_at = $14, scheduled_unpublish_at = $15,
            locale = $16, translation_group_id = $17, updated_at = now()
         WHERE id = $1 AND updated_at = $18",
    )
    .bind(id)
    .bind(p.parent_id)
    .bind(&p.title)
    .bind(&slug)
    .bind(&p.excerpt)
    .bind(&p.body)
    .bind(&p.body_html)
    .bind(&p.blocks)
    .bind(&p.intent)
    .bind(&p.design_tokens)
    .bind(p.status)
    .bind(&p.fields)
    .bind(p.featured_media_id)
    .bind(p.published_at)
    .bind(p.scheduled_unpublish_at)
    .bind(&p.locale)
    .bind(p.translation_group_id)
    .bind(current.updated_at)
    .execute(&mut *tx)
    .await?;
    if changed.rows_affected() != 1 {
        return Err(CoreError::Conflict(
            "Le contenu a été modifié pendant la sauvegarde.".into(),
        ));
    }
    if let Some(terms) = &p.term_ids {
        write_terms(&mut tx, id, terms).await?;
    }
    let content_changed = true;
    if ct.has_revisions && content_changed {
        write_revision(&mut tx, id, editor_id, &p).await?;
    }
    tx.commit().await?;

    let entry = get(db, id).await?;
    if entry.status == EntryStatus::Published && current.status != EntryStatus::Published {
        notify_published(hooks, &entry).await;
    }
    Ok(entry)
}

/// Met à la corbeille, ou supprime définitivement si `force` ou déjà à la corbeille.
pub async fn delete(db: &PgPool, hooks: &dyn HookRunner, id: Uuid, force: bool) -> CoreResult<()> {
    let current = get(db, id).await?;
    if force || current.status == EntryStatus::Trash {
        sqlx::query("DELETE FROM entries WHERE id = $1")
            .bind(id)
            .execute(db)
            .await?;
        hooks
            .action(names::ENTRY_DELETED, json!({ "id": id }).to_string())
            .await;
    } else {
        sqlx::query("UPDATE entries SET status = 'trash', updated_at = now() WHERE id = $1")
            .bind(id)
            .execute(db)
            .await?;
    }
    Ok(())
}

pub async fn revisions(db: &PgPool, entry_id: Uuid) -> CoreResult<Vec<Revision>> {
    Ok(sqlx::query_as::<_, Revision>(
        "SELECT r.id, r.entry_id, r.author_id, u.display_name AS author_name, r.title, r.excerpt, r.body,
                r.fields, r.snapshot, r.created_at
         FROM revisions r LEFT JOIN users u ON u.id = r.author_id
         WHERE r.entry_id = $1 ORDER BY r.created_at DESC",
    )
    .bind(entry_id)
    .fetch_all(db)
    .await?)
}

/// Restaure le contenu d'une révision (crée une nouvelle révision).
pub async fn restore_revision(
    db: &PgPool,
    hooks: &dyn HookRunner,
    entry_id: Uuid,
    revision_id: Uuid,
    editor_id: Uuid,
) -> CoreResult<Entry> {
    restore_revision_checked(db, hooks, entry_id, revision_id, editor_id, None).await
}

/// Restaure le contenu d'une révision en vérifiant optionnellement `expected_updated_at` (`WP-28`).
pub async fn restore_revision_checked(
    db: &PgPool,
    hooks: &dyn HookRunner,
    entry_id: Uuid,
    revision_id: Uuid,
    editor_id: Uuid,
    expected_updated_at: Option<chrono::DateTime<Utc>>,
) -> CoreResult<Entry> {
    let current = get(db, entry_id).await?;
    if expected_updated_at.is_some_and(|expected| expected != current.updated_at) {
        return Err(CoreError::Conflict(
            "Ce contenu a été modifié par un autre auteur avant la restauration. Rechargez l'éditeur.".into(),
        ));
    }
    let rev = sqlx::query_as::<_, Revision>(
        "SELECT r.id, r.entry_id, r.author_id, NULL::text AS author_name, r.title, r.excerpt, r.body, r.fields, r.snapshot, r.created_at
         FROM revisions r WHERE r.id = $1 AND r.entry_id = $2",
    )
    .bind(revision_id)
    .bind(entry_id)
    .fetch_one(db)
    .await?;

    if let Some(snapshot) = rev.snapshot {
        let mut input: EntryInput = serde_json::from_value(snapshot)
            .map_err(|e| CoreError::validation(format!("Révision invalide : {e}")))?;
        input.expected_updated_at = expected_updated_at;
        return update_internal(db, hooks, entry_id, editor_id, input, true).await;
    }
    if current.blocks.get("rf_schema").is_some() {
        return Err(CoreError::Conflict("Cette ancienne révision ne contient pas de document natif complet. Sa restauration automatique est refusée pour préserver les blocs actuels.".into()));
    }
    let input = EntryInput {
        expected_updated_at,
        title: rev.title,
        slug: Some(current.slug),
        excerpt: rev.excerpt,
        body: rev.body,
        blocks: current.blocks,
        intent: current.intent,
        design_tokens: current.design_tokens,
        status: current.status,
        fields: rev.fields,
        featured_media_id: current.featured_media_id,
        parent_id: current.parent_id,
        published_at: current.published_at,
        scheduled_unpublish_at: current.scheduled_unpublish_at,
        locale: current.locale,
        translation_group_id: current.translation_group_id,
        term_ids: Some(current.terms.iter().map(|t| t.id).collect()),
    };
    update(db, hooks, entry_id, editor_id, input).await
}

// ---------------------------------------------------------------------------
// Grappes de traduction (MOD-I18N-07)
// ---------------------------------------------------------------------------

/// Retourne toutes les entrées sœurs appartenant à la même grappe de traduction.
pub async fn get_translations(
    db: &PgPool,
    entry_id: Uuid,
) -> CoreResult<Vec<crate::models::TranslationSibling>> {
    let entry = get(db, entry_id).await?;
    let Some(group_id) = entry.translation_group_id else {
        return Ok(vec![crate::models::TranslationSibling {
            id: entry.id,
            locale: entry.locale,
            title: entry.title,
            slug: entry.slug,
            status: entry.status,
        }]);
    };

    let siblings = sqlx::query_as::<_, crate::models::TranslationSibling>(
        "SELECT id, locale, title, slug, status
         FROM entries
         WHERE translation_group_id = $1 AND status <> 'trash'
         ORDER BY locale ASC",
    )
    .bind(group_id)
    .fetch_all(db)
    .await?;

    Ok(siblings)
}

/// Crée une nouvelle traduction pour une entrée dans la langue cible, en clonant
/// l'arbre de blocs natifs et en liant les deux entrées dans la même grappe.
pub async fn create_translation(
    db: &PgPool,
    hooks: &dyn HookRunner,
    source_id: Uuid,
    target_locale: &str,
    author_id: Uuid,
) -> CoreResult<Entry> {
    let target_locale = target_locale.trim().to_lowercase();
    if target_locale.is_empty() {
        return Err(CoreError::validation("La langue cible est requise"));
    }

    let source = get(db, source_id).await?;
    if source.locale == target_locale {
        return Err(CoreError::validation(format!(
            "L'entrée source est déjà en langue '{}'",
            source.locale
        )));
    }

    // Détermine ou crée le translation_group_id
    let group_id = match source.translation_group_id {
        Some(gid) => {
            // Vérifie si une traduction existe déjà dans cette langue
            let existing: Option<Uuid> = sqlx::query_scalar(
                "SELECT id FROM entries WHERE translation_group_id = $1 AND locale = $2 AND status <> 'trash' LIMIT 1"
            )
            .bind(gid)
            .bind(&target_locale)
            .fetch_optional(db)
            .await?;

            if existing.is_some() {
                return Err(CoreError::Conflict(format!(
                    "Une traduction en '{target_locale}' existe déjà pour ce groupe"
                )));
            }
            gid
        }
        None => {
            let new_gid = Uuid::new_v4();
            sqlx::query("UPDATE entries SET translation_group_id = $1 WHERE id = $2")
                .bind(new_gid)
                .bind(source_id)
                .execute(db)
                .await?;
            new_gid
        }
    };

    let input = EntryInput {
        expected_updated_at: None,
        title: format!("{} ({})", source.title, target_locale.to_uppercase()),
        slug: Some(format!("{}-{}", source.slug, target_locale)),
        excerpt: source.excerpt.clone(),
        body: source.body.clone(),
        blocks: source.blocks.clone(),
        intent: source.intent.clone(),
        design_tokens: source.design_tokens.clone(),
        status: EntryStatus::Draft,
        fields: source.fields.clone(),
        featured_media_id: source.featured_media_id,
        parent_id: None,
        published_at: None,
        scheduled_unpublish_at: None,
        locale: target_locale,
        translation_group_id: Some(group_id),
        term_ids: None,
    };

    create(db, hooks, &source.content_type, author_id, input).await
}

/// Lie deux entrées existantes au sein de la même grappe de traduction.
pub async fn link_translations(db: &PgPool, source_id: Uuid, target_id: Uuid) -> CoreResult<()> {
    if source_id == target_id {
        return Err(CoreError::validation(
            "Impossible de lier une entrée à elle-même",
        ));
    }

    let source = get(db, source_id).await?;
    let target = get(db, target_id).await?;

    if source.locale == target.locale {
        return Err(CoreError::validation(format!(
            "Impossible de lier deux entrées ayant la même langue ('{}')",
            source.locale
        )));
    }

    let group_id = match (source.translation_group_id, target.translation_group_id) {
        (Some(sg), Some(tg)) => {
            if sg == tg {
                return Ok(()); // Déjà liées
            }
            // Fusionne le groupe cible dans le groupe source
            sqlx::query(
                "UPDATE entries SET translation_group_id = $1 WHERE translation_group_id = $2",
            )
            .bind(sg)
            .bind(tg)
            .execute(db)
            .await?;
            sg
        }
        (Some(sg), None) => {
            sqlx::query("UPDATE entries SET translation_group_id = $1 WHERE id = $2")
                .bind(sg)
                .bind(target_id)
                .execute(db)
                .await?;
            sg
        }
        (None, Some(tg)) => {
            sqlx::query("UPDATE entries SET translation_group_id = $1 WHERE id = $2")
                .bind(tg)
                .bind(source_id)
                .execute(db)
                .await?;
            tg
        }
        (None, None) => {
            let new_gid = Uuid::new_v4();
            sqlx::query("UPDATE entries SET translation_group_id = $1 WHERE id = $2 OR id = $3")
                .bind(new_gid)
                .bind(source_id)
                .bind(target_id)
                .execute(db)
                .await?;
            new_gid
        }
    };

    let _ = group_id;
    Ok(())
}

/// Retire une entrée de sa grappe de traduction.
pub async fn unlink_translation(db: &PgPool, entry_id: Uuid) -> CoreResult<()> {
    sqlx::query("UPDATE entries SET translation_group_id = NULL WHERE id = $1")
        .bind(entry_id)
        .execute(db)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn document_settings_reject_invalid_discussion_values() {
        for value in [
            serde_json::json!(null),
            serde_json::json!("open"),
            serde_json::json!("closed"),
        ] {
            assert!(
                super::validate_document_settings(&serde_json::json!({"_rf_comment_status":value}))
                    .is_ok()
            );
        }
        for value in [
            serde_json::json!(true),
            serde_json::json!("inherit"),
            serde_json::json!({"open":true}),
        ] {
            assert!(
                super::validate_document_settings(&serde_json::json!({"_rf_comment_status":value}))
                    .is_err()
            );
        }
    }

    #[test]
    fn test_merge_fields_preserves_legacy_vault() {
        let base = json!({
            "_legacy_vault": {
                "_thumbnail_id": "42",
                "_yoast_wpseo_title": "SEO Title",
                "custom_postmeta": "preserved"
            },
            "author_bio": "Original Bio"
        });

        // Mise à jour partielle provenant d'un formulaire qui ne connaît pas _legacy_vault
        let update = json!({
            "author_bio": "Updated Bio"
        });

        let merged = merge_fields(&base, &update);

        assert_eq!(merged["author_bio"], "Updated Bio");
        assert_eq!(merged["_legacy_vault"]["_thumbnail_id"], "42");
        assert_eq!(merged["_legacy_vault"]["custom_postmeta"], "preserved");
    }

    #[test]
    fn test_merge_fields_preserves_rf_vault() {
        let base = json!({
            "_rf_vault": {
                "_thumbnail_id": "42",
                "custom_meta": "sovereign_data"
            },
            "_legacy_vault": {
                "old_key": "legacy_data"
            },
            "rf_categories": ["Actualités", "Rust"],
            "title": "Sovereign Entry"
        });

        // Mise à jour partielle provenant du Studio sans toucher au coffre-fort
        let update = json!({
            "title": "Updated Sovereign Entry"
        });

        let merged = merge_fields(&base, &update);

        assert_eq!(merged["title"], "Updated Sovereign Entry");
        assert_eq!(merged["_rf_vault"]["_thumbnail_id"], "42");
        assert_eq!(merged["_rf_vault"]["custom_meta"], "sovereign_data");
        assert_eq!(merged["_legacy_vault"]["old_key"], "legacy_data");
        assert_eq!(merged["rf_categories"][1], "Rust");
    }

    #[test]
    fn test_prepare_resets_published_at_when_reverted_to_draft() {
        let ct = ContentType {
            id: Uuid::new_v4(),
            slug: "post".to_string(),
            name: "Article".to_string(),
            name_singular: "Article".to_string(),
            description: "".to_string(),
            icon: "".to_string(),
            hierarchical: false,
            has_revisions: true,
            is_system: true,
            fields: vec![],
            entry_count: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let past_date = Utc::now() - chrono::Duration::days(10);

        let input = EntryInput {
            expected_updated_at: None,
            title: "Test Draft".to_string(),
            slug: None,
            excerpt: "".to_string(),
            body: "Hello".to_string(),
            blocks: json!([]),
            intent: json!({}),
            design_tokens: json!({}),
            status: EntryStatus::Draft,
            fields: json!({}),
            featured_media_id: None,
            parent_id: None,
            published_at: None, // non spécifié ou hérité
            scheduled_unpublish_at: None,
            locale: "fr".to_string(),
            translation_group_id: None,
            term_ids: None,
        };

        // Même si existing_published_at était dans le passé (article préalablement publié),
        // pour un brouillon, published_at doit être remis à None pour éviter la réactivation par le scheduler !
        let prepared = prepare(&ct, input, Some(past_date), None).unwrap();
        assert!(
            prepared.published_at.is_none(),
            "Un article remis en brouillon ne doit pas conserver sa date de publication passée"
        );
    }

    #[test]
    fn test_record_slug_history_prevents_loops_and_tracks_chains() {
        let mut fields = json!({});
        record_slug_history(&mut fields, "slug-initial", "slug-v2");
        assert_eq!(fields["_rf_vault"]["rf_old_slugs"], json!(["slug-initial"]));
        record_slug_history(&mut fields, "slug-v2", "slug-v3");
        assert_eq!(
            fields["_rf_vault"]["rf_old_slugs"],
            json!(["slug-initial", "slug-v2"])
        );
        record_slug_history(&mut fields, "slug-v3", "slug-initial");
        assert_eq!(
            fields["_rf_vault"]["rf_old_slugs"],
            json!(["slug-v2", "slug-v3"])
        );
    }
}
