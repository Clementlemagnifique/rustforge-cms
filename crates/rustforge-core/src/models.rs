//! Types du domaine RustForge.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

use crate::fields::FieldDef;

// ---------------------------------------------------------------------------
// Utilisateurs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Editor,
    Author,
    Subscriber,
}

impl Role {
    /// Peut gérer la configuration du site (utilisateurs, réglages, plugins, types).
    pub fn is_admin(self) -> bool {
        matches!(self, Role::Admin)
    }

    /// Peut modifier les contenus de tous les auteurs et les taxonomies.
    pub fn can_edit_others(self) -> bool {
        matches!(self, Role::Admin | Role::Editor)
    }

    /// Peut créer des contenus et téléverser des médias.
    pub fn can_write(self) -> bool {
        matches!(self, Role::Admin | Role::Editor | Role::Author)
    }
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub display_name: String,
    pub role: Role,
    pub bio: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub entry_count: i64,
    #[serde(skip)]
    pub password_hash: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Types de contenu
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ContentType {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub name_singular: String,
    pub description: String,
    pub icon: String,
    pub hierarchical: bool,
    pub has_revisions: bool,
    pub is_system: bool,
    #[sqlx(json)]
    pub fields: Vec<FieldDef>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Nombre d'entrées (hors corbeille).
    pub entry_count: i64,
}

// ---------------------------------------------------------------------------
// Entrées
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "entry_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum EntryStatus {
    Draft,
    Published,
    Archived,
    Trash,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TermRef {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub taxonomy: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Entry {
    pub id: Uuid,
    pub content_type_id: Uuid,
    pub content_type: String,
    pub parent_id: Option<Uuid>,
    pub author_id: Option<Uuid>,
    pub author_name: Option<String>,
    pub title: String,
    pub slug: String,
    pub excerpt: String,
    pub body: String,
    pub body_html: String,
    pub blocks: Value,
    pub intent: Value,
    pub design_tokens: Value,
    pub status: EntryStatus,
    pub fields: Value,
    pub featured_media_id: Option<Uuid>,
    pub featured_media_url: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub scheduled_unpublish_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[sqlx(skip)]
    pub terms: Vec<TermRef>,
}

impl Entry {
    /// The public API never serializes raw bodies, archived metadata or private fields.
    pub fn public_view(self) -> Value {
        serde_json::json!({
            "id": self.id, "content_type": self.content_type, "slug": self.slug,
            "title": self.title, "excerpt": self.excerpt, "body_html": self.body_html,
            "author_name": self.author_name, "published_at": self.published_at,
            "featured_media_url": self.featured_media_url, "terms": self.terms,
            "updated_at": self.updated_at
        })
    }
}

/// Données d'écriture d'une entrée (création et mise à jour).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryInput {
    #[serde(default)]
    pub expected_updated_at: Option<DateTime<Utc>>,
    pub title: String,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub excerpt: String,
    #[serde(default)]
    pub body: String,
    #[serde(default = "empty_array")]
    pub blocks: Value,
    #[serde(default = "empty_object")]
    pub intent: Value,
    #[serde(default = "empty_object")]
    pub design_tokens: Value,
    #[serde(default = "default_status")]
    pub status: EntryStatus,
    #[serde(default = "empty_object")]
    pub fields: Value,
    #[serde(default)]
    pub featured_media_id: Option<Uuid>,
    #[serde(default)]
    pub parent_id: Option<Uuid>,
    #[serde(default)]
    pub published_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub scheduled_unpublish_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub term_ids: Option<Vec<Uuid>>,
}

impl EntryInput {
    /// Only keys actually provided are replaced. JSON null explicitly clears nullable relations.
    pub fn from_patch(entry: &Entry, patch: serde_json::Value) -> crate::CoreResult<Self> {
        crate::document::validate_source_patch(&entry.blocks, &entry.body, &patch)?;
        let mut base = serde_json::json!({
            "expected_updated_at": entry.updated_at,
            "title": entry.title, "slug": entry.slug, "excerpt": entry.excerpt,
            "body": entry.body, "blocks": entry.blocks, "intent": entry.intent,
            "design_tokens": entry.design_tokens, "status": entry.status, "fields": entry.fields,
            "parent_id": entry.parent_id, "featured_media_id": entry.featured_media_id,
            "published_at": entry.published_at, "scheduled_unpublish_at": entry.scheduled_unpublish_at,
            "term_ids": entry.terms.iter().map(|t| t.id).collect::<Vec<_>>()
        });
        let patch = patch
            .as_object()
            .ok_or_else(|| crate::CoreError::validation("un objet de modification est requis"))?;
        if let Some(expected) = patch.get("expected_updated_at").and_then(|v| v.as_str()) {
            let expected = chrono::DateTime::parse_from_rfc3339(expected)
                .map_err(|_| crate::CoreError::validation("version de contenu invalide"))?;
            if expected.with_timezone(&chrono::Utc) != entry.updated_at {
                return Err(crate::CoreError::Conflict(
                    "Ce contenu a changé. Rechargez-le avant de réessayer.".into(),
                ));
            }
        }
        for (key, value) in patch {
            if key == "expected_updated_at" {
                continue;
            }
            if !base.as_object().expect("object").contains_key(key) {
                return Err(crate::CoreError::validation(format!(
                    "champ de modification inconnu : {key}"
                )));
            }
            base[key] = value.clone();
        }
        serde_json::from_value(base).map_err(|e| crate::CoreError::validation(e.to_string()))
    }
}

fn empty_array() -> Value {
    Value::Array(Vec::new())
}

fn default_status() -> EntryStatus {
    EntryStatus::Draft
}
fn empty_object() -> Value {
    Value::Object(Default::default())
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Revision {
    pub id: Uuid,
    pub entry_id: Uuid,
    pub author_id: Option<Uuid>,
    pub author_name: Option<String>,
    pub title: String,
    pub excerpt: String,
    pub body: String,
    pub fields: Value,
    pub snapshot: Option<Value>,
    pub created_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Taxonomies
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Taxonomy {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub hierarchical: bool,
    pub is_system: bool,
    pub term_count: i64,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Term {
    pub id: Uuid,
    pub taxonomy_id: Uuid,
    pub taxonomy: String,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub entry_count: i64,
    pub created_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Médias
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Media {
    pub id: Uuid,
    pub path: String,
    pub url: String,
    pub original_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub alt: String,
    pub caption: String,
    pub uploaded_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub folder_id: Option<Uuid>,
    pub webp_path: Option<String>,
    pub webp_url: Option<String>,
    pub external_provider: Option<String>,
    pub metadata: Value,
}

// ---------------------------------------------------------------------------
// Commentaires (WP-42 à WP-45, WP-82 à WP-87)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum CommentStatus {
    Pending,
    Approved,
    Spam,
    Trash,
}

impl CommentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            CommentStatus::Pending => "pending",
            CommentStatus::Approved => "approved",
            CommentStatus::Spam => "spam",
            CommentStatus::Trash => "trash",
        }
    }
}

impl std::fmt::Display for CommentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for CommentStatus {
    type Err = crate::error::CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "pending" => Ok(CommentStatus::Pending),
            "approved" => Ok(CommentStatus::Approved),
            "spam" => Ok(CommentStatus::Spam),
            "trash" => Ok(CommentStatus::Trash),
            _ => Err(crate::error::CoreError::validation("Statut de commentaire invalide")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Comment {
    pub id: Uuid,
    pub entry_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub author_id: Option<Uuid>,
    pub author_name: String,
    pub author_email: String,
    pub author_url: Option<String>,
    pub author_ip: Option<String>,
    pub author_user_agent: Option<String>,
    pub content: String,
    pub content_html: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[sqlx(default)]
    pub entry_title: Option<String>,
    #[sqlx(default)]
    pub entry_slug: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicCommentView {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub author_name: String,
    pub author_url: Option<String>,
    pub content_html: String,
    pub created_at: DateTime<Utc>,
    pub children: Vec<PublicCommentView>,
}
