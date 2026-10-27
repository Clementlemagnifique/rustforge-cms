//! Modèle de manifeste et structures de données pour le RustForge Universal Converter.
//! Format intermédiaire TPM-1.0 (RustForge Manifest version 1.0).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Type de package WordPress détecté dans l'archive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageKind {
    /// Thème moderne Full Site Editing (theme.json + templates de blocs)
    ThemeFse,
    /// Thème classique WordPress (style.css + index.php)
    ThemeClassic,
    /// Plugin ou extension WordPress
    Plugin,
    /// Archive non reconnue ou composite
    Unknown,
}

/// Métadonnées extraites de l'archive WordPress.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub kind: PackageKind,
    pub slug: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub entry_point: Option<String>,
    pub has_php: bool,
    pub has_theme_json: bool,
    pub has_block_templates: bool,
    pub total_files: usize,
    pub uncompressed_size_bytes: u64,
}

/// Hook ou point d'accroche PHP détecté par analyse statique lexicale.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedHook {
    pub hook_type: String, // "action", "filter", "shortcode", "post_type", "block"
    pub name: String,
    pub handler: Option<String>,
    pub file_path: String,
    pub line_number: usize,
}

/// Stratégie de conversion retenue pour le package.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversionStrategyKind {
    /// 1. Réutilisation directe d'un module natif RustForge existant
    NativeReuse,
    /// 2. Transformation déclarative (theme.json, templates FSE, blocs)
    Declarative,
    /// 3. Génération contrôlée d'adaptateur Rust/WASM dans sandbox
    ControlledWasm,
    /// 4. Conversion assistée (audit, données dans _legacy_vault, doc)
    Assisted,
}

/// Évaluation de compatibilité globale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompatibilityStatus {
    FullySupported,
    PartiallySupported,
    RequiresAssistance,
    Incompatible,
}

/// Rapport détaillé des fonctionnalités migrées, remplacées et incompatibles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityReport {
    pub score: u8, // 0 à 100
    pub status: CompatibilityStatus,
    pub migrated_features: Vec<String>,
    pub replaced_features: Vec<String>,
    pub partially_supported: Vec<String>,
    pub incompatible_features: Vec<String>,
}

/// Plan d'action non exécuté préparé pour validation par l'utilisateur.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActivationPlan {
    pub enable_modules: Vec<String>,
    pub register_blocks: Vec<String>,
    pub deploy_theme_slug: Option<String>,
    pub requires_code_generation: bool,
    pub requires_database_migration: bool,
    pub preview_available: bool,
}

/// Manifeste complet de conversion intermédiaire TPM-1.0.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionManifest {
    pub tpm_version: String,
    pub analyzed_at: DateTime<Utc>,
    pub package: PackageMetadata,
    pub detected_hooks: Vec<DetectedHook>,
    pub detected_capabilities: Vec<String>,
    pub strategy: ConversionStrategyKind,
    pub target_native_module: Option<String>,
    pub compatibility: CompatibilityReport,
    pub activation_plan: ActivationPlan,
}
