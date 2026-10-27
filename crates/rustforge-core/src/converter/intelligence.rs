//! Capability Intelligence Engine, Gap Analysis, Capability Composer
//! et Micro-Plugin Builder pour RustForge.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ---------------------------------------------------------------------------
// 1. Capability Registry — Contrats Versionnés & États Réels
// ---------------------------------------------------------------------------

/// Disponibilité réelle d'une capacité dans RustForge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityOperationalStatus {
    /// Implémenté, typé et validé par des tests unitaires/intégration
    ImplementedAndTested,
    /// Présent dans le code mais sans validation automatisée complète
    ImplementedUnverified,
    /// Cœur fonctionnel disponible, certaines options manquantes
    PartiallyAvailable,
    /// Prévu dans la feuille de route
    Planned,
    /// Rejeté pour des motifs de sécurité ou hors périmètre
    Unavailable,
}

/// Contrat formel et versionné d'une capacité RustForge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityContract {
    pub id: String,
    pub version: String,
    pub description: String,
    pub provider_module: String,
    pub operational_status: CapabilityOperationalStatus,
    pub inputs_schema: Option<Value>,
    pub outputs_schema: Option<Value>,
    pub required_permissions: Vec<String>,
    pub produced_events: Vec<String>,
    pub side_effects: Vec<String>,
}

// ---------------------------------------------------------------------------
// 2. Feature Discovery Engine & Graphe Fonctionnel
// ---------------------------------------------------------------------------

/// Niveau de certitude d'une preuve identifiée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceLevel {
    Certain,
    High,
    Medium,
    Heuristic,
    Hypothetical,
}

/// Nœud élémentaire du graphe de fonctionnalités.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureNode {
    pub id: String,
    pub name: String,
    pub description: String,
    pub source_reference: String,
    pub confidence: ConfidenceLevel,
    pub dependencies: Vec<String>,
    pub business_effects: Vec<String>,
}

/// Graphe de fonctionnalités extrait du package source.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FunctionalFeatureGraph {
    pub package_slug: String,
    pub nodes: Vec<FeatureNode>,
}

// ---------------------------------------------------------------------------
// 3. Gap Analysis Engine — L'Arbitrage de Conversion
// ---------------------------------------------------------------------------

/// Type d'arbitrage calculé pour une fonctionnalité.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GapResolutionKind {
    /// Fonctionnalité déjà disponible nativement
    Native,
    /// Fonctionnalité existante nécessitant une configuration/option
    Configurable,
    /// Réalisable par combinaison déclarative de plusieurs capacités
    Composable,
    /// Absente du cœur : génération ciblée d'une micro-extension WASM
    Generatable,
    /// Non réalisable automatiquement avec le SDK actuel
    Unsupported,
    /// Données d'analyse insuffisantes
    Unknown,
}

/// Décision d'arbitrage pour un nœud du graphe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureGapDecision {
    pub feature_id: String,
    pub resolution: GapResolutionKind,
    pub matched_capabilities: Vec<String>,
    pub rationale: String,
}

pub struct GapAnalysisEngine;

impl GapAnalysisEngine {
    /// Évalue chaque nœud du graphe de fonctionnalités par rapport au Capability Registry.
    pub fn evaluate(graph: &FunctionalFeatureGraph) -> Vec<FeatureGapDecision> {
        let mut decisions = Vec::new();

        for node in &graph.nodes {
            let id_lower = node.id.to_lowercase();
            let desc_lower = node.description.to_lowercase();

            if id_lower.contains("link_order_to_course") || desc_lower.contains("inscription post-achat") {
                decisions.push(FeatureGapDecision {
                    feature_id: node.id.clone(),
                    resolution: GapResolutionKind::Composable,
                    matched_capabilities: vec!["commerce.order_flow".to_string(), "lms.course_enrollment".to_string()],
                    rationale: "Réalisable par composition déclarative de commerce.order.completed ➔ lms.enroll_user".to_string(),
                });
            } else if id_lower.contains("form") || desc_lower.contains("form") {
                decisions.push(FeatureGapDecision {
                    feature_id: node.id.clone(),
                    resolution: GapResolutionKind::Native,
                    matched_capabilities: vec!["forms.submit".to_string(), "forms.builder".to_string()],
                    rationale: "Remplacé nativement par le module RustForge Forms (JSON Schema + Honeypot)".to_string(),
                });
            } else if id_lower.contains("order") || id_lower.contains("checkout") || desc_lower.contains("téléchargement") {
                decisions.push(FeatureGapDecision {
                    feature_id: node.id.clone(),
                    resolution: GapResolutionKind::Native,
                    matched_capabilities: vec!["commerce.order_flow".to_string(), "commerce.signed_downloads".to_string()],
                    rationale: "Pris en charge nativement par le module E-Commerce dématérialisé (HMAC-SHA256)".to_string(),
                });
            } else if id_lower.contains("course") || id_lower.contains("lesson") || desc_lower.contains("formation") {
                decisions.push(FeatureGapDecision {
                    feature_id: node.id.clone(),
                    resolution: GapResolutionKind::Native,
                    matched_capabilities: vec!["lms.course_enrollment".to_string()],
                    rationale: "Pris en charge nativement par le module LMS avec inscription atomique".to_string(),
                });
            } else if id_lower.contains("reading_time") || desc_lower.contains("temps de lecture") {
                decisions.push(FeatureGapDecision {
                    feature_id: node.id.clone(),
                    resolution: GapResolutionKind::Generatable,
                    matched_capabilities: vec!["content.filter.render".to_string()],
                    rationale: "Fonctionnalité absente du cœur : génération d'une micro-extension WASM isolée".to_string(),
                });
            } else {
                decisions.push(FeatureGapDecision {
                    feature_id: node.id.clone(),
                    resolution: GapResolutionKind::Configurable,
                    matched_capabilities: vec!["content.crud".to_string()],
                    rationale: "Adaptable via types de contenus personnalisés et stockage extensible".to_string(),
                });
            }
        }

        decisions
    }
}

// ---------------------------------------------------------------------------
// 4. Capability Composer — L'Assemblage Déclaratif
// ---------------------------------------------------------------------------

/// Étape élémentaire d'un pipeline d'assemblage déclaratif.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionStep {
    pub step: usize,
    pub capability: String,
    pub input_mapping: Value,
}

/// Définition d'une composition déclarative sans code Rust compilé.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityComposition {
    pub id: String,
    pub name: String,
    pub trigger_event: String,
    pub pipeline: Vec<CompositionStep>,
    pub created_at: String,
}

pub struct CapabilityComposer;

impl CapabilityComposer {
    /// Crée une composition standard reliant deux capacités.
    pub fn compose_ecommerce_lms_link() -> CapabilityComposition {
        CapabilityComposition {
            id: "ecommerce_to_lms_auto_enroll".to_string(),
            name: "Liaison automatique Commande ➔ Inscription Formation".to_string(),
            trigger_event: "commerce.order.completed".to_string(),
            pipeline: vec![
                CompositionStep {
                    step: 1,
                    capability: "lms.enroll_user".to_string(),
                    input_mapping: serde_json::json!({
                        "user_id": "$event.customer_id",
                        "course_id": "$event.course_id"
                    }),
                },
            ],
            created_at: Utc::now().to_rfc3339(),
        }
    }
}

// ---------------------------------------------------------------------------
// 5. Micro-Plugin Builder — Le SDK WASM Isolé
// ---------------------------------------------------------------------------

/// Fichier généré d'une micro-extension WebAssembly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFile {
    pub path: String,
    pub content: String,
}

/// Package complet d'une micro-extension WASM prêt à compiler.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroPluginPackage {
    pub extension_slug: String,
    pub name: String,
    pub files: Vec<GeneratedFile>,
}

pub struct MicroPluginBuilder;

impl MicroPluginBuilder {
    /// Génère le squelette complet d'une micro-extension WASM standardisée (WASIp2).
    pub fn build_skeleton(slug: &str, name: &str, hook: &str) -> MicroPluginPackage {
        let mut files = Vec::new();

        // 1. Cargo.toml
        let cargo_toml = format!(
            r#"[package]
name = "{slug}"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
wit-bindgen = "0.36"
"#
        );
        files.push(GeneratedFile {
            path: "Cargo.toml".to_string(),
            content: cargo_toml,
        });

        // 2. manifest.toml
        let manifest_toml = format!(
            r#"id = "{slug}"
name = "{name}"
version = "0.1.0"
description = "Micro-extension générée par RustForge Micro-Plugin Builder."
hooks = ["{hook}"]
capabilities = ["storage"]
"#
        );
        files.push(GeneratedFile {
            path: "manifest.toml".to_string(),
            content: manifest_toml,
        });

        // 3. wit/rustforge-plugin.wit
        let wit = r#"package rustforge:plugin@0.1.0;

interface types {
    enum capability { storage, http-outbound }
    record manifest {
        id: string,
        name: string,
        version: string,
        description: string,
        hooks: list<string>,
        capabilities: list<capability>,
    }
}

interface host {
    enum level { trace, debug, info, warn, error }
    log: func(level: level, message: string);
    kv-get: func(key: string) -> option<string>;
    kv-set: func(key: string, value: string) -> result<_, string>;
}

world plugin {
    use types.{manifest};
    import host;
    export manifest: func() -> manifest;
    export filter: func(hook: string, payload: string) -> result<string, string>;
    export on-action: func(hook: string, payload: string);
}
"#;
        files.push(GeneratedFile {
            path: "wit/rustforge-plugin.wit".to_string(),
            content: wit.to_string(),
        });

        // 4. src/lib.rs
        let src_lib = format!(
            r#"#[allow(warnings)]
mod bindings;
use bindings::Guest;
use bindings::rustforge::plugin::types::Manifest;

struct Component;

impl Guest for Component {{
    fn manifest() -> Manifest {{
        Manifest {{
            id: "{slug}".to_string(),
            name: "{name}".to_string(),
            version: "0.1.0".to_string(),
            description: "Micro-extension ciblée RustForge".to_string(),
            hooks: vec!["{hook}".to_string()],
            capabilities: vec![],
        }}
    }}

    fn filter(hook: String, payload: String) -> Result<String, String> {{
        if hook == "{hook}" {{
            // Logique de transformation isolée
            Ok(format!("{{payload}} <!-- Extension: {slug} -->"))
        }} else {{
            Ok(payload)
        }}
    }}

    fn on_action(_hook: String, _payload: String) {{}}
}}

bindings::export!(Component with_types_in bindings);
"#
        );
        files.push(GeneratedFile {
            path: "src/lib.rs".to_string(),
            content: src_lib,
        });

        MicroPluginPackage {
            extension_slug: slug.to_string(),
            name: name.to_string(),
            files,
        }
    }
}

// ---------------------------------------------------------------------------
// 6. Validation Engine & Community Registry
// ---------------------------------------------------------------------------

/// Statut de distribution d'une extension dans le catalogue partagé.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommunityExtensionStatus {
    Private,
    Experimental,
    Community,
    Verified,
    Official,
}

/// Résultat des vérifications du Validation Engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub passed_checks: Vec<String>,
    pub warnings: Vec<String>,
    pub security_score: u8,
}

pub struct ValidationEngine;

impl ValidationEngine {
    /// Valide une micro-extension avant proposition d'installation.
    pub fn validate(package: &MicroPluginPackage) -> ValidationResult {
        let mut passed = Vec::new();
        let mut warnings = Vec::new();

        // 1. Contrôle des fichiers obligatoires
        let has_cargo = package.files.iter().any(|f| f.path == "Cargo.toml");
        let has_manifest = package.files.iter().any(|f| f.path == "manifest.toml");
        let has_wit = package.files.iter().any(|f| f.path.contains(".wit"));

        if has_cargo && has_manifest && has_wit {
            passed.push("Arborescence du SDK WASM complète (Cargo.toml, manifest.toml, wit)".to_string());
        } else {
            warnings.push("Fichiers de contrat manquants dans l'extension générée".to_string());
        }

        // 2. Contrôle de sécurité du code Rust
        let mut unsafe_found = false;
        for file in &package.files {
            if file.content.contains("unsafe ") || file.content.contains("std::fs") || file.content.contains("std::net") {
                unsafe_found = true;
                break;
            }
        }

        if !unsafe_found {
            passed.push("Code 100% sûr : absence d'unsafe, de réseau et d'accès système direct".to_string());
        } else {
            warnings.push("Avertissement : utilisation de blocs unsafe ou appels système non sandboxés".to_string());
        }

        let is_valid = warnings.is_empty();
        let security_score = if is_valid { 100 } else { 70 };

        ValidationResult {
            is_valid,
            passed_checks: passed,
            warnings,
            security_score,
        }
    }
}
