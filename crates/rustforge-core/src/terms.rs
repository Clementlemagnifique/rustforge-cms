//! Taxonomies et termes (catégories hiérarchiques, étiquettes, taxonomies personnalisées).
//!
//! Garantit :
//! - l'absence de cycles (`A -> B -> A`) et l'homogénéité de taxonomie sur `parent_id`,
//! - la réaffectation sans perte des contenus (`entry_terms`) et des sous-termes lors d'une
//!   fusion (`merge_terms`) ou d'une suppression avec cible (`delete_term_with_reassignment`).

use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    CoreError, CoreResult,
    models::{Taxonomy, Term},
};

const TERM_SELECT: &str = "SELECT t.id, t.taxonomy_id, tx.slug AS taxonomy, t.parent_id, t.name, t.slug,
        t.description, t.created_at,
        (SELECT count(*) FROM entry_terms et JOIN entries e ON e.id = et.entry_id
          WHERE et.term_id = t.id AND e.status <> 'trash') AS entry_count
    FROM terms t JOIN taxonomies tx ON tx.id = t.taxonomy_id";

#[derive(Debug, Deserialize)]
pub struct TermInput {
    /// Slug de la taxonomie (création uniquement).
    pub taxonomy: Option<String>,
    pub name: String,
    pub slug: Option<String>,
    #[serde(default)]
    pub description: String,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct TaxonomyInput {
    pub name: String,
    pub slug: Option<String>,
    #[serde(default)]
    pub hierarchical: bool,
}

pub async fn list_taxonomies(db: &PgPool) -> CoreResult<Vec<Taxonomy>> {
    Ok(sqlx::query_as::<_, Taxonomy>(
        "SELECT tx.id, tx.slug, tx.name, tx.hierarchical, tx.is_system,
                (SELECT count(*) FROM terms t WHERE t.taxonomy_id = tx.id) AS term_count
         FROM taxonomies tx ORDER BY tx.is_system DESC, tx.name",
    )
    .fetch_all(db)
    .await?)
}

pub async fn create_taxonomy(db: &PgPool, input: TaxonomyInput) -> CoreResult<Taxonomy> {
    if input.name.trim().is_empty() {
        return Err(CoreError::validation("le nom est obligatoire"));
    }
    let slug = crate::slugify(input.slug.as_deref().unwrap_or(&input.name));
    sqlx::query("INSERT INTO taxonomies (slug, name, hierarchical) VALUES ($1, $2, $3)")
        .bind(&slug)
        .bind(input.name.trim())
        .bind(input.hierarchical)
        .execute(db)
        .await?;
    list_taxonomies(db)
        .await?
        .into_iter()
        .find(|t| t.slug == slug)
        .ok_or(CoreError::NotFound)
}

pub async fn delete_taxonomy(db: &PgPool, id: Uuid) -> CoreResult<()> {
    let is_system: bool = sqlx::query_scalar("SELECT is_system FROM taxonomies WHERE id = $1")
        .bind(id)
        .fetch_one(db)
        .await?;
    if is_system {
        return Err(CoreError::Conflict(
            "les taxonomies système ne peuvent pas être supprimées".into(),
        ));
    }
    sqlx::query("DELETE FROM taxonomies WHERE id = $1")
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn list_terms(db: &PgPool, taxonomy: Option<&str>) -> CoreResult<Vec<Term>> {
    Ok(sqlx::query_as::<_, Term>(&format!(
        "{TERM_SELECT} WHERE ($1::text IS NULL OR tx.slug = $1) ORDER BY tx.slug, t.name"
    ))
    .bind(taxonomy)
    .fetch_all(db)
    .await?)
}

pub async fn get_term(db: &PgPool, id: Uuid) -> CoreResult<Term> {
    sqlx::query_as::<_, Term>(&format!("{TERM_SELECT} WHERE t.id = $1"))
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or(CoreError::NotFound)
}

/// Vérifie que `parent_id` appartient à la même taxonomie hiérarchique et ne crée aucun cycle.
async fn validate_term_parent_hierarchy(
    db: &PgPool,
    term_id: Option<Uuid>,
    taxonomy_id: Uuid,
    hierarchical: bool,
    parent_id: Option<Uuid>,
) -> CoreResult<Option<Uuid>> {
    if !hierarchical {
        return Ok(None);
    }
    let Some(mut cursor) = parent_id else {
        return Ok(None);
    };
    if Some(cursor) == term_id {
        return Err(CoreError::validation(
            "un terme ne peut pas être son propre parent",
        ));
    }
    for _ in 0..32 {
        let row: Option<(Uuid, Option<Uuid>)> =
            sqlx::query_as("SELECT taxonomy_id, parent_id FROM terms WHERE id = $1")
                .bind(cursor)
                .fetch_optional(db)
                .await?;
        let Some((parent_taxonomy_id, next_parent)) = row else {
            return Err(CoreError::validation("le terme parent sélectionné est introuvable"));
        };
        if parent_taxonomy_id != taxonomy_id {
            return Err(CoreError::validation(
                "le terme parent doit appartenir à la même taxonomie",
            ));
        }
        let Some(next) = next_parent else {
            return Ok(parent_id);
        };
        if Some(next) == term_id || next == cursor {
            return Err(CoreError::validation(
                "hiérarchie de termes invalide : cycle détecté",
            ));
        }
        cursor = next;
    }
    Err(CoreError::validation(
        "hiérarchie de termes trop profonde (maximum 32 niveaux)",
    ))
}

pub async fn create_term(db: &PgPool, input: TermInput) -> CoreResult<Term> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(CoreError::validation("le nom est obligatoire"));
    }
    let taxonomy = input
        .taxonomy
        .as_deref()
        .ok_or_else(|| CoreError::validation("taxonomie manquante"))?;
    let (taxonomy_id, hierarchical): (Uuid, bool) =
        sqlx::query_as("SELECT id, hierarchical FROM taxonomies WHERE slug = $1")
            .bind(taxonomy)
            .fetch_optional(db)
            .await?
            .ok_or_else(|| CoreError::validation(format!("taxonomie inconnue « {taxonomy} »")))?;

    let validated_parent =
        validate_term_parent_hierarchy(db, None, taxonomy_id, hierarchical, input.parent_id)
            .await?;

    let slug = crate::slugify(input.slug.as_deref().filter(|s| !s.is_empty()).unwrap_or(name));
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO terms (taxonomy_id, parent_id, name, slug, description) VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(taxonomy_id)
    .bind(validated_parent)
    .bind(name)
    .bind(slug)
    .bind(input.description.trim())
    .fetch_one(db)
    .await?;
    get_term(db, id).await
}

pub async fn update_term(db: &PgPool, id: Uuid, input: TermInput) -> CoreResult<Term> {
    let current = get_term(db, id).await?;
    let name = input.name.trim();
    if name.is_empty() {
        return Err(CoreError::validation("le nom est obligatoire"));
    }
    let hierarchical: bool =
        sqlx::query_scalar("SELECT hierarchical FROM taxonomies WHERE id = $1")
            .bind(current.taxonomy_id)
            .fetch_one(db)
            .await?;

    let validated_parent = validate_term_parent_hierarchy(
        db,
        Some(id),
        current.taxonomy_id,
        hierarchical,
        input.parent_id,
    )
    .await?;

    let slug = crate::slugify(
        input
            .slug
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(&current.slug),
    );
    sqlx::query(
        "UPDATE terms SET name = $2, slug = $3, description = $4, parent_id = $5 WHERE id = $1",
    )
    .bind(id)
    .bind(name)
    .bind(slug)
    .bind(input.description.trim())
    .bind(validated_parent)
    .execute(db)
    .await?;
    get_term(db, id).await
}

/// Supprime un terme en réaffectant optionnellement ses contenus et ses sous-termes vers `reassign_to`.
pub async fn delete_term_with_reassignment(
    db: &PgPool,
    id: Uuid,
    reassign_to: Option<Uuid>,
) -> CoreResult<()> {
    let current = get_term(db, id).await?;

    if let Some(target_id) = reassign_to {
        if target_id == id {
            return Err(CoreError::validation(
                "impossible de réaffecter un terme vers lui-même",
            ));
        }
        let target = get_term(db, target_id).await?;
        if target.taxonomy_id != current.taxonomy_id {
            return Err(CoreError::validation(
                "le terme de réaffectation doit appartenir à la même taxonomie",
            ));
        }
        // Vérifier que le rattachement des sous-termes à target_id ne crée pas un cycle
        // (c'est-à-dire que target_id n'est pas un descendant de `id`).
        let mut cursor = target.parent_id;
        while let Some(pid) = cursor {
            if pid == id {
                return Err(CoreError::validation(
                    "impossible de fusionner ou réaffecter un terme parent vers l'un de ses propres sous-termes",
                ));
            }
            cursor = sqlx::query_scalar("SELECT parent_id FROM terms WHERE id = $1")
                .bind(pid)
                .fetch_optional(db)
                .await?
                .flatten();
        }
    }

    let mut tx = db.begin().await?;
    if let Some(target_id) = reassign_to {
        // Transférer toutes les liaisons d'articles/pages sans doublon
        sqlx::query(
            "INSERT INTO entry_terms (entry_id, term_id)
             SELECT entry_id, $2 FROM entry_terms WHERE term_id = $1
             ON CONFLICT DO NOTHING",
        )
        .bind(id)
        .bind(target_id)
        .execute(&mut *tx)
        .await?;

        // Rattacher les sous-termes directs au terme cible
        sqlx::query("UPDATE terms SET parent_id = $2 WHERE parent_id = $1")
            .bind(id)
            .bind(target_id)
            .execute(&mut *tx)
            .await?;
    } else {
        // Promouvoir les sous-termes directs au parent du terme supprimé
        sqlx::query("UPDATE terms SET parent_id = $2 WHERE parent_id = $1")
            .bind(id)
            .bind(current.parent_id)
            .execute(&mut *tx)
            .await?;
    }

    let res = sqlx::query("DELETE FROM terms WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    if res.rows_affected() == 0 {
        return Err(CoreError::NotFound);
    }
    tx.commit().await?;
    Ok(())
}

pub async fn delete_term(db: &PgPool, id: Uuid) -> CoreResult<()> {
    delete_term_with_reassignment(db, id, None).await
}

/// Fusionne `source_id` dans `target_id` (transfère les articles et sous-termes, puis supprime `source_id`).
pub async fn merge_terms(db: &PgPool, source_id: Uuid, target_id: Uuid) -> CoreResult<Term> {
    delete_term_with_reassignment(db, source_id, Some(target_id)).await?;
    get_term(db, target_id).await
}
