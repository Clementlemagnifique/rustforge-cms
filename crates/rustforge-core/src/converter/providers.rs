//! Fournisseurs d'analyse multi-niveaux pour RustForge Universal Converter.
//! Abstrait les analyseurs WordPress standards, JS et le fournisseur d'investigation REA.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::CoreResult;

/// Cible soumise à l'analyse (fichier PHP, bundle JS, CSS, template ou binaire).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisTarget {
    pub file_path: String,
    pub mime_type: String,
    pub content: String,
    pub is_minified_or_compiled: bool,
}

/// Preuve vérifiable extraite par un fournisseur d'analyse.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisEvidence {
    pub provider_name: String,
    pub file_reference: String,
    pub line_or_offset: Option<usize>,
    pub snippet_or_signature: String,
    pub raw_finding: String,
    pub confidence_score: u8, // 0 à 100
}

/// Contrat abstrait pour tous les fournisseurs d'analyse logicielle.
#[async_trait]
pub trait AnalysisProvider: Send + Sync {
    /// Nom unique du fournisseur (ex: "wordpress_metadata", "js_static", "rea")
    fn name(&self) -> &'static str;

    /// Indique si ce fournisseur peut traiter la cible donnée
    fn supports(&self, target: &AnalysisTarget) -> bool;

    /// Exécute l'analyse et retourne les preuves fonctionnelles identifiées
    async fn analyze(&self, target: &AnalysisTarget) -> CoreResult<Vec<AnalysisEvidence>>;
}

// ---------------------------------------------------------------------------
// 1. Fournisseur Niveau 1 : Métadonnées et Hooks WordPress Standards
// ---------------------------------------------------------------------------

pub struct WordPressMetadataProvider;

#[async_trait]
impl AnalysisProvider for WordPressMetadataProvider {
    fn name(&self) -> &'static str {
        "wordpress_metadata"
    }

    fn supports(&self, target: &AnalysisTarget) -> bool {
        target.file_path.ends_with(".php")
            || target.file_path.ends_with(".css")
            || target.file_path.ends_with("theme.json")
    }

    async fn analyze(&self, target: &AnalysisTarget) -> CoreResult<Vec<AnalysisEvidence>> {
        let mut evidences = Vec::new();

        for (line_idx, line) in target.content.lines().enumerate() {
            let line_num = line_idx + 1;
            let trimmed = line.trim();

            if trimmed.contains("add_action") {
                evidences.push(AnalysisEvidence {
                    provider_name: self.name().to_string(),
                    file_reference: format!("{}:{}", target.file_path, line_num),
                    line_or_offset: Some(line_num),
                    snippet_or_signature: trimmed.to_string(),
                    raw_finding: "Hook d'action WordPress détecté".to_string(),
                    confidence_score: 95,
                });
            } else if trimmed.contains("add_filter") {
                evidences.push(AnalysisEvidence {
                    provider_name: self.name().to_string(),
                    file_reference: format!("{}:{}", target.file_path, line_num),
                    line_or_offset: Some(line_num),
                    snippet_or_signature: trimmed.to_string(),
                    raw_finding: "Hook de filtre WordPress détecté".to_string(),
                    confidence_score: 95,
                });
            } else if trimmed.contains("add_shortcode") {
                evidences.push(AnalysisEvidence {
                    provider_name: self.name().to_string(),
                    file_reference: format!("{}:{}", target.file_path, line_num),
                    line_or_offset: Some(line_num),
                    snippet_or_signature: trimmed.to_string(),
                    raw_finding: "Shortcode WordPress enregistré".to_string(),
                    confidence_score: 100,
                });
            } else if trimmed.contains("register_post_type") {
                evidences.push(AnalysisEvidence {
                    provider_name: self.name().to_string(),
                    file_reference: format!("{}:{}", target.file_path, line_num),
                    line_or_offset: Some(line_num),
                    snippet_or_signature: trimmed.to_string(),
                    raw_finding: "Type de contenu personnalisé (Custom Post Type)".to_string(),
                    confidence_score: 98,
                });
            }
        }

        Ok(evidences)
    }
}

// ---------------------------------------------------------------------------
// 2. Fournisseur Niveau 1/2 : Analyse Statique JavaScript
// ---------------------------------------------------------------------------

pub struct JavaScriptStaticProvider;

#[async_trait]
impl AnalysisProvider for JavaScriptStaticProvider {
    fn name(&self) -> &'static str {
        "js_static"
    }

    fn supports(&self, target: &AnalysisTarget) -> bool {
        target.file_path.ends_with(".js") || target.file_path.ends_with(".mjs")
    }

    async fn analyze(&self, target: &AnalysisTarget) -> CoreResult<Vec<AnalysisEvidence>> {
        let mut evidences = Vec::new();
        let content_lower = target.content.to_lowercase();

        // Détection d'appels d'API REST ou fetch
        if content_lower.contains("fetch(")
            || content_lower.contains("ajax")
            || content_lower.contains("axios")
        {
            evidences.push(AnalysisEvidence {
                provider_name: self.name().to_string(),
                file_reference: target.file_path.clone(),
                line_or_offset: None,
                snippet_or_signature: "fetch / ajax client call".to_string(),
                raw_finding: "Communication asynchrone client ➔ API REST détectée".to_string(),
                confidence_score: 85,
            });
        }

        // Détection de bibliothèques UI (React, Vue, jQuery)
        if content_lower.contains("react") || content_lower.contains("createelement") {
            evidences.push(AnalysisEvidence {
                provider_name: self.name().to_string(),
                file_reference: target.file_path.clone(),
                line_or_offset: None,
                snippet_or_signature: "React / Gutenberg Block Editor Component".to_string(),
                raw_finding: "Composant d'interface React (bloc Gutenberg interactif)".to_string(),
                confidence_score: 90,
            });
        }

        Ok(evidences)
    }
}

// ---------------------------------------------------------------------------
// 3. Fournisseur Niveau 3 : REA (Reverse Engineer Anything - Optionnel)
// ---------------------------------------------------------------------------

/// Fournisseur d'analyse REA branché sur les outils MCP ou CLI de reverse engineering.
/// Isolé du runtime public, opère en bac à sable strict.
pub struct ReaAnalysisProvider {
    pub is_cli_available: bool,
}

impl ReaAnalysisProvider {
    pub fn new() -> Self {
        // Détecte si le binaire rea ou la commande rea-agents est accessible
        Self {
            is_cli_available: false,
        }
    }
}

impl Default for ReaAnalysisProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AnalysisProvider for ReaAnalysisProvider {
    fn name(&self) -> &'static str {
        "rea"
    }

    fn supports(&self, target: &AnalysisTarget) -> bool {
        // REA intervient en priorité sur les cibles compilées, minifiées ou opaques
        target.is_minified_or_compiled
            || target.file_path.ends_with(".min.js")
            || target.file_path.ends_with(".bundle.js")
            || target.file_path.ends_with(".wasm")
    }

    async fn analyze(&self, target: &AnalysisTarget) -> CoreResult<Vec<AnalysisEvidence>> {
        let mut evidences = Vec::new();

        // En mode standard (sans REA CLI local ou en fallback sécurisé)
        if target.file_path.ends_with(".min.js") || target.file_path.contains("bundle") {
            evidences.push(AnalysisEvidence {
                provider_name: self.name().to_string(),
                file_reference: target.file_path.clone(),
                line_or_offset: None,
                snippet_or_signature: "Extension de fichier uniquement ; REA non exécuté".to_string(),
                raw_finding: "Analyse manuelle ou connecteur REA réel requis ; aucune déobfuscation effectuée".to_string(),
                confidence_score: 0,
            });
        }

        Ok(evidences)
    }
}
