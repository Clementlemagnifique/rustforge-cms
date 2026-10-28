//! Format Pivot RFIR (RustForge Intermediate Representation) / TMIR.
//! Schéma intermédiaire universel, strict et versionné pour l'absorption souveraine de données CMS.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Version courante du format intermédiaire souverain RFIR / TMIR.
pub const RFIR_CURRENT_VERSION: &str = "1.0";
pub const TMIR_CURRENT_VERSION: &str = RFIR_CURRENT_VERSION;

/// Document pivot complet représentant une exportation ou extraction de données vers RustForge.
pub type RFIRDocument = TMIRDocument;
pub type RFIREntry = TMIREntry;
pub type RFIRTerm = TMIRTerm;
pub type RFIRMedia = TMIRMedia;
pub type RFIREntryStatus = TMIREntryStatus;
pub type RFIRImportReport = TMIRImportReport;

/// Document pivot complet représentant une exportation ou extraction de données.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TMIRDocument {
    pub tmir_version: String,
    pub generated_at: DateTime<Utc>,
    pub source_generator: String, // ex: "Legacy CMS / WXR 1.2"
    pub source_site_url: Option<String>,
    pub entries: Vec<TMIREntry>,
    pub terms: Vec<TMIRTerm>,
    pub media: Vec<TMIRMedia>,
    pub metadata: BTreeMap<String, Value>,
}

impl Default for TMIRDocument {
    fn default() -> Self {
        Self {
            tmir_version: TMIR_CURRENT_VERSION.to_string(),
            generated_at: Utc::now(),
            source_generator: "RustForge DataMorph".to_string(),
            source_site_url: None,
            entries: Vec::new(),
            terms: Vec::new(),
            media: Vec::new(),
            metadata: BTreeMap::new(),
        }
    }
}

/// Statut de publication d'une entrée TMIR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TMIREntryStatus {
    Published,
    Draft,
    Pending,
    Private,
    Trash,
    Archived,
}

/// Entrée pivot de contenu (Article, Page, Produit, Cours, CPT).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TMIREntry {
    pub source_id: String,
    pub slug: String,
    pub title: String,
    pub content_type: String, // "post", "page", "product", "course", etc.
    pub status: TMIREntryStatus,
    pub raw_content: String,
    pub excerpt: Option<String>,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub featured_image_url: Option<String>,
    /// Le Coffre-Fort Numérique : conserve toutes les métadonnées spécifiques sans perte
    pub legacy_vault: BTreeMap<String, Value>,
}

/// Terme d'une taxonomie (Catégorie, Étiquette, Taxonomie personnalisée).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TMIRTerm {
    pub source_id: String,
    pub taxonomy: String, // "category", "post_tag", "product_cat", etc.
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_slug: Option<String>,
}

/// Média référencé ou attaché.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TMIRMedia {
    pub source_id: String,
    pub source_url: String,
    pub filename: String,
    pub mime_type: String,
    pub title: Option<String>,
    pub alt_text: Option<String>,
    pub parent_entry_id: Option<String>,
}

/// Rapport et statistiques d'une opération DataMorph.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TMIRImportReport {
    pub entries_imported: usize,
    pub entries_skipped: usize,
    pub terms_imported: usize,
    pub media_detected: usize,
    pub media_imported: usize,
    pub legacy_fields_stored_in_vault: usize,
    pub execution_time_ms: u64,
    pub created_content_types: Vec<String>,
    pub warnings: Vec<String>,
}
