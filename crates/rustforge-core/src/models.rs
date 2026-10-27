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
    #[serde(skip)]
    pub password_hash: String,
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

/// Données d'écriture d'une entrée (création et mise à jour).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryInput {
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
