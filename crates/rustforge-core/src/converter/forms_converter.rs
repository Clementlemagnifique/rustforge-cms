//! Convertisseur déclaratif pour les extensions de formulaires WordPress (Cas B).
//! Distingue le code applicatif PHP (remplacé par rustforge-core::forms)
//! des données de formulaires stockées en base de données.

use serde::{Deserialize, Serialize};
use super::manifest::{CompatibilityReport, CompatibilityStatus, ConversionStrategyKind};

/// Proposition de schéma de formulaire RustForge prêt à l'emploi.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormProposal {
    pub name: String,
    pub title: String,
    pub fields_schema: serde_json::Value,
    pub honeypot_enabled: bool,
    pub notify_email: Option<String>,
}

/// Résultat d'analyse et proposition pour un plugin de formulaires WordPress.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormsConversionResult {
    pub source_plugin_slug: String,
    pub strategy: ConversionStrategyKind,
    pub compatibility: CompatibilityReport,
    pub code_disposition: String,
    pub data_migration_guidance: String,
    pub proposed_form: FormProposal,
}

pub struct FormsConverter;

impl FormsConverter {
    /// Analyse un plugin de formulaire et produit le plan de remplacement par le module natif Forms.
    pub fn plan_conversion(slug: &str, author_or_title: &str) -> FormsConversionResult {
        let is_wpforms = slug.contains("wpforms");
        let is_cf7 = slug.contains("contact-form-7") || slug.contains("cf7");

        let title = if is_wpforms {
            "Formulaire de Contact Standard (Import WPForms)"
        } else if is_cf7 {
            "Formulaire de Contact Standard (Import CF7)"
        } else {
            "Formulaire de Contact Sécurisé RustForge"
        };

        // Schéma JSON natif RustForge (JSON Schema strict)
        let fields_schema = serde_json::json!({
            "type": "object",
            "required": ["nom", "email", "message"],
            "properties": {
                "nom": {
                    "type": "string",
                    "title": "Votre nom",
                    "minLength": 2
                },
                "email": {
                    "type": "string",
                    "format": "email",
                    "title": "Adresse e-mail"
                },
                "sujet": {
                    "type": "string",
                    "title": "Objet du message"
                },
                "message": {
                    "type": "string",
                    "title": "Votre message",
                    "minLength": 10
                }
            }
        });

        let proposed_form = FormProposal {
            name: format!("contact-{}", slug::slugify(slug)),
            title: title.to_string(),
            fields_schema,
            honeypot_enabled: true,
            notify_email: None,
        };

        let report = CompatibilityReport {
            score: 95,
            status: CompatibilityStatus::FullySupported,
            migrated_features: vec![
                "Soumissions sécurisées avec validation JSON Schema stricte en Rust".to_string(),
                "Protection anti-spam native par pot de miel (Honeypot) sans captcha".to_string(),
                "Hachage d'adresses IP anonymisé RGPD avec sel journalier tournant".to_string(),
                "Stockage persistant des soumissions dans la table forms_submissions".to_string(),
            ],
            replaced_features: vec![
                "Le code PHP et les scripts jQuery du plugin sont totalement neutralisés".to_string(),
                "Les shortcodes ([wpforms id=X]) sont remplacés par le bloc universel FormEmbed".to_string(),
                "Les requêtes AJAX WordPress (admin-ajax.php) sont remplacées par l'endpoint REST /api/v1/forms/{id}/submit".to_string(),
            ],
            partially_supported: vec![
                "Logiques conditionnelles multi-pages complexes (nécessite schéma conditionnel étendu)".to_string(),
            ],
            incompatible_features: vec![
                "Extensions PHP tierces propriétaires non documentées".to_string(),
            ],
        };

        FormsConversionResult {
            source_plugin_slug: slug.to_string(),
            strategy: ConversionStrategyKind::NativeReuse,
            compatibility: report,
            code_disposition: format!(
                "Le code PHP de '{}' ({}) est entièrement remplacé par le module natif RustForge Forms. Aucun fichier PHP n'est conservé ni exécuté.",
                slug, author_or_title
            ),
            data_migration_guidance: "IMPORTANT : Les formulaires existants et leurs soumissions historiques stockés en base de données WordPress (tables wp_posts / wp_wpforms_* / wp_postmeta) seront importés via le connecteur de base de données RustForge DataMorph, indépendamment de ce ZIP de plugin.".to_string(),
            proposed_form,
        }
    }
}
