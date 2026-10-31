//! # RustForge AI & Tool Registry
//!
//! Moteur d'intentions et registre d'outils déclaratifs (Function Calling / MCP)
//! permettant à n'importe quel LLM ou agent vocal de piloter intégralement le CMS.
//!
//! Sépare strictement :
//! - **Le Fond** (sémantique, intention, structure de blocs)
//! - **La Forme** (design tokens, thèmes, styles)
//! - **Les Actions** (commandes atomiques et auditables)

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// Fournisseurs d'IA supportés.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AiProvider {
    Ollama,
    OpenAi,
    Anthropic,
    Gemini,
    Mistral,
    DeepSeek,
    OpenRouter,
}

/// Configuration du fournisseur IA actif.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AiSettings {
    pub provider: AiProvider,
    pub model: String,
    pub api_key: Option<String>,
    pub endpoint: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            provider: AiProvider::Ollama,
            model: "gemma4:12b".to_string(),
            api_key: None,
            endpoint: Some("http://localhost:11434".to_string()),
            temperature: Some(0.4),
            max_tokens: Some(4096),
        }
    }
}

/// Clé de la table `settings` contenant la configuration IA.
pub const SETTINGS_KEY: &str = "ai.settings";

/// Charge la configuration IA (valeurs par défaut pour les champs absents).
pub async fn load_settings(db: &sqlx::PgPool) -> crate::CoreResult<AiSettings> {
    Ok(crate::settings::get(db, SETTINGS_KEY)
        .await?
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default())
}

/// Définition d'un outil actionnable par l'IA (compatible format JSON Schema / OpenAI Tools / MCP).
#[derive(Debug, Clone, Serialize)]
pub struct ToolDefinition {
    pub name: &'static str,
    pub description: &'static str,
    pub parameters: Value,
}

/// Registre exhaustif des outils exposés aux agents vocaux et textuels.
pub fn get_available_tools() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "create_or_update_content",
            description: "Crée ou met à jour une entrée (article, page, produit...) avec son intention, son contenu (Markdown ou Blocs JSON) et ses tokens de design.",
            parameters: json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "UUID optionnel pour mise à jour" },
                    "content_type": { "type": "string", "description": "Slug du type (post, page, event, product...)" },
                    "title": { "type": "string", "description": "Titre du contenu" },
                    "slug": { "type": "string", "description": "Slug d'URL (généré automatiquement si omis)" },
                    "excerpt": { "type": "string", "description": "Résumé ou chapeau éditorial" },
                    "body": { "type": "string", "description": "Contenu en Markdown" },
                    "blocks": {
                        "type": "array",
                        "description": "Arbre de blocs typés (titre, paragraphe, image, formulaire, faq...)",
                        "items": { "type": "object" }
                    },
                    "intent": {
                        "type": "object",
                        "description": "Fond sémantique : audience cible, ton, mots-clés, objectif de conversion"
                    },
                    "design_tokens": {
                        "type": "object",
                        "description": "Forme visuelle : overrides de couleurs, typographie ou disposition"
                    },
                    "status": { "type": "string", "enum": ["draft", "published", "archived"] },
                    "terms": { "type": "array", "items": { "type": "string" }, "description": "Slugs des catégories/étiquettes" }
                },
                "required": ["content_type", "title"]
            }),
        },
        ToolDefinition {
            name: "update_design_system",
            description: "Modifie les tokens globaux du design system (palette de couleurs, typographies, espacements, rayon des bordures).",
            parameters: json!({
                "type": "object",
                "properties": {
                    "primary_color": { "type": "string", "description": "Code HEX/HSL de la couleur primaire" },
                    "accent_color": { "type": "string", "description": "Couleur d'accentuation" },
                    "font_heading": { "type": "string", "description": "Police pour les titres (ex: 'Plus Jakarta Sans', 'Inter')" },
                    "font_body": { "type": "string", "description": "Police pour le texte" },
                    "dark_mode": { "type": "boolean", "description": "Activation du mode sombre" },
                    "radius": { "type": "string", "description": "Rayon des bordures (ex: '8px', '12px', 'none')" }
                }
            }),
        },
        ToolDefinition {
            name: "create_content_type",
            description: "Définit un nouveau type de contenu personnalisé avec ses champs (ex: Portfolio, Témoignages, FAQ, Événements).",
            parameters: json!({
                "type": "object",
                "properties": {
                    "slug": { "type": "string", "description": "Slug unique en minuscules" },
                    "name": { "type": "string", "description": "Nom au pluriel (ex: Témoignages)" },
                    "name_singular": { "type": "string", "description": "Nom au singulier (ex: Témoignage)" },
                    "icon": { "type": "string", "description": "Identifiant d'icône" },
                    "fields": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "key": { "type": "string" },
                                "label": { "type": "string" },
                                "type": { "type": "string", "enum": ["text", "textarea", "number", "boolean", "date", "url", "select", "media"] },
                                "required": { "type": "boolean" }
                            },
                            "required": ["key", "label", "type"]
                        }
                    }
                },
                "required": ["name", "name_singular"]
            }),
        },
        ToolDefinition {
            name: "audit_geo_and_seo",
            description: "Génère l'analyse SEO sémantique et GEO (Generative Engine Optimization) pour positionner le contenu auprès des LLMs (Perplexity, ChatGPT, Claude).",
            parameters: json!({
                "type": "object",
                "properties": {
                    "entry_id": { "type": "string", "description": "Identifiant UUID de l'entrée à auditer" }
                },
                "required": ["entry_id"]
            }),
        },
        ToolDefinition {
            name: "search_content",
            description: "Recherche des contenus existants (titre, extrait, texte) et renvoie leurs identifiants. À utiliser AVANT de modifier ou d'auditer un contenu dont on ne connaît pas l'UUID.",
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Mots-clés recherchés (vide = derniers contenus modifiés)" },
                    "content_type": { "type": "string", "description": "Filtre optionnel sur le slug du type (post, page...)" },
                    "limit": { "type": "integer", "description": "Nombre maximum de résultats (défaut 5)" }
                }
            }),
        },
        ToolDefinition {
            name: "generate_block",
            description: "Génère un bloc de contenu structuré conforme au schéma rf.document/1 (paragraphe, titre, tableau, citation, callout, bouton, etc.) à insérer dans une entrée.",
            parameters: json!({
                "type": "object",
                "properties": {
                    "kind": {
                        "type": "string",
                        "enum": [
                            "heading", "paragraph", "quote", "code", "list",
                            "divider", "callout", "table", "details", "button",
                            "spacer", "cover", "group", "product", "course"
                        ],
                        "description": "Type de bloc sémantique natif"
                    },
                    "text": { "type": "string", "description": "Texte principal ou Markdown du bloc" },
                    "level": { "type": "integer", "description": "Niveau pour heading (1 à 6, défaut 2)" },
                    "items": { "type": "array", "items": { "type": "string" }, "description": "Éléments pour liste ou lignes de tableau" },
                    "style": { "type": "string", "description": "Variante de style autorisée (ex: 'info', 'warning', 'card', 'primary')" },
                    "url": { "type": "string", "description": "URL sûre pour bouton ou lien" }
                },
                "required": ["kind", "text"]
            }),
        },
        ToolDefinition {
            name: "diff_and_patch_blocks",
            description: "Propose une modification chirurgicale (insertion, remplacement, ajout) sous forme de diff de blocs avec prévisualisation et plan d'approbation.",
            parameters: json!({
                "type": "object",
                "properties": {
                    "entry_id": { "type": "string", "description": "UUID de l'entrée ciblée" },
                    "operation": {
                        "type": "string",
                        "enum": ["append", "prepend", "replace_selection", "insert_at"],
                        "description": "Opération d'édition de blocs demandée"
                    },
                    "target_index": { "type": "integer", "minimum": 0, "description": "Index exact pour insertion ou remplacement ; bornes strictes" },
                    "target_id": { "type": "string", "description": "Identifiant stable du bloc ciblé, préféré à target_index" },
                    "blocks": {
                        "type": "array",
                        "items": { "type": "object" },
                        "description": "Liste de blocs à appliquer"
                    },
                    "explanation": { "type": "string", "description": "Explication de la modification pour approbation humaine" }
                },
                "required": ["entry_id", "operation", "blocks"]
            }),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_settings_default() {
        let settings = AiSettings::default();
        assert_eq!(settings.provider, AiProvider::Ollama);
        assert_eq!(settings.model, "gemma4:12b");
        assert_eq!(settings.endpoint.as_deref(), Some("http://localhost:11434"));
    }

    #[test]
    fn test_get_available_tools_includes_editor_tools() {
        let tools = get_available_tools();
        let names: Vec<&str> = tools.iter().map(|t| t.name).collect();

        assert!(names.contains(&"create_or_update_content"));
        assert!(names.contains(&"update_design_system"));
        assert!(names.contains(&"create_content_type"));
        assert!(names.contains(&"audit_geo_and_seo"));
        assert!(names.contains(&"search_content"));
        assert!(names.contains(&"generate_block"));
        assert!(names.contains(&"diff_and_patch_blocks"));

        let diff_tool = tools
            .iter()
            .find(|t| t.name == "diff_and_patch_blocks")
            .unwrap();
        assert!(diff_tool.parameters["properties"]["blocks"].is_object());
        assert!(diff_tool.parameters["properties"]["operation"].is_object());
    }
}

/// Génération déterministe : même validation que le document rendu et enregistré.
pub fn generate_native_block(args: &serde_json::Value) -> crate::CoreResult<serde_json::Value> {
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Input {
        kind: String,
        text: String,
        #[serde(default)]
        style: String,
        #[serde(default)]
        url: String,
        #[serde(default)]
        items: Vec<String>,
        level: Option<u8>,
    }
    let input: Input = serde_json::from_value(args.clone())
        .map_err(|_| crate::CoreError::validation("Arguments de génération invalides"))?;
    let block = serde_json::json!({
        "rf_id": format!("blk_{}", uuid::Uuid::new_v4().simple()),
        "rf_type": input.kind, "rf_text": input.text, "rf_level": input.level.unwrap_or(2),
        "rf_style": input.style, "rf_url": input.url, "rf_items": input.items,
    });
    crate::document::project(
        "",
        &serde_json::json!({"rf_schema":crate::document::SCHEMA,"rf_blocks":[block.clone()]}),
    )?;
    Ok(block)
}

/// Prépare un document complet sans écrire. Aucun contenu hérité n'est converti implicitement.
pub fn patch_native_blocks(
    current: &serde_json::Value,
    args: &serde_json::Value,
) -> crate::CoreResult<serde_json::Value> {
    use crate::CoreError;
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Patch {
        operation: String,
        blocks: Vec<serde_json::Value>,
        target_index: Option<usize>,
        target_id: Option<String>,
        // Métadonnées du plan vérifiées par l'API, pas des champs du document.
        #[serde(rename = "entry_id")]
        _entry_id: Option<uuid::Uuid>,
        #[serde(rename = "expected_updated_at")]
        _expected: Option<chrono::DateTime<chrono::Utc>>,
        #[serde(rename = "explanation")]
        _explanation: Option<String>,
    }
    if current.get("rf_schema").and_then(|v| v.as_str()) != Some(crate::document::SCHEMA) {
        return Err(CoreError::validation(
            "Le diff exige un document natif ; convertissez explicitement le contenu hérité",
        ));
    }
    crate::document::project("", current)?;
    let patch: Patch = serde_json::from_value(args.clone())
        .map_err(|_| CoreError::validation("Arguments du diff invalides"))?;
    if patch.blocks.is_empty() || patch.blocks.len() > crate::document::MAX_BLOCKS {
        return Err(CoreError::validation(
            "Le diff doit contenir entre 1 et 256 blocs",
        ));
    }
    let mut result = current.clone();
    let blocks = result["rf_blocks"]
        .as_array_mut()
        .ok_or_else(|| CoreError::validation("Blocs natifs manquants"))?;
    let index = if let Some(id) = patch.target_id {
        let found = blocks
            .iter()
            .position(|b| b["rf_id"].as_str() == Some(&id))
            .ok_or_else(|| CoreError::validation("Bloc cible introuvable"))?;
        if patch.target_index.is_some_and(|index| index != found) {
            return Err(CoreError::validation(
                "Identifiant et indice de cible incohérents",
            ));
        }
        Some(found)
    } else {
        patch.target_index
    };
    match patch.operation.as_str() {
        "append" | "prepend" => {
            if index.is_some() {
                return Err(CoreError::validation(
                    "Cette opération ne prend pas de cible",
                ));
            }
            let at = if patch.operation == "append" {
                blocks.len()
            } else {
                0
            };
            blocks.splice(at..at, patch.blocks);
        }
        "insert_at" => {
            let at = index
                .filter(|i| *i <= blocks.len())
                .ok_or_else(|| CoreError::validation("Indice d'insertion invalide"))?;
            blocks.splice(at..at, patch.blocks);
        }
        "replace_selection" => {
            let at = index
                .filter(|i| *i < blocks.len())
                .ok_or_else(|| CoreError::validation("Cible de remplacement invalide"))?;
            // Remplace un bloc sélectionné par tous les blocs proposés, sans troncature.
            blocks.splice(at..=at, patch.blocks);
        }
        _ => return Err(CoreError::validation("Opération de diff inconnue")),
    }
    crate::document::project("", &result)?;
    Ok(result)
}

#[cfg(test)]
mod editor_tests {
    use super::*;
    use serde_json::json;
    fn block(id: &str, text: &str) -> serde_json::Value {
        json!({"rf_id":id,"rf_type":"paragraph","rf_text":text})
    }
    #[test]
    fn generated_blocks_use_the_renderer_contract() {
        assert!(generate_native_block(&json!({"kind":"paragraph","text":"Bonjour"})).is_ok());
        assert!(generate_native_block(&json!({"kind":"divider","text":""})).is_ok());
        for bad in [
            json!({"kind":"unknown","text":"x"}),
            json!({"kind":"button","text":"x","url":"javascript:alert(1)"}),
            json!({"kind":"heading","text":"x","level":258}),
            json!({"kind":"list","text":"x","items":[1]}),
            json!({"kind":"paragraph","text":"x","style":"injected"}),
        ] {
            assert!(generate_native_block(&bad).is_err(), "{bad}");
        }
    }
    #[test]
    fn patch_preserves_neighbors_and_all_replacement_blocks() {
        let doc = json!({"rf_schema":crate::document::SCHEMA,"rf_blocks":[block("a","A"),block("b","B"),block("c","C")]});
        let result=patch_native_blocks(&doc,&json!({"operation":"replace_selection","target_id":"b","blocks":[block("d","D"),block("e","E")]})).unwrap();
        assert_eq!(
            result["rf_blocks"],
            json!([
                block("a", "A"),
                block("d", "D"),
                block("e", "E"),
                block("c", "C")
            ])
        );
        assert_eq!(doc["rf_blocks"][1]["rf_text"], "B");
        let (body, html) = crate::document::project("old", &result).unwrap();
        assert!(body.contains('D'));
        assert!(html.contains('E'));
    }
    #[test]
    fn patch_rejects_legacy_invalid_targets_and_duplicate_ids() {
        let doc = json!({"rf_schema":crate::document::SCHEMA,"rf_blocks":[block("a","A")]});
        for args in [
            json!({"operation":"insert_at","target_index":2,"blocks":[block("b","B")]}),
            json!({"operation":"replace_selection","target_id":"missing","blocks":[block("b","B")]}),
            json!({"operation":"append","blocks":[block("a","duplicate")]}),
            json!({"operation":"append","blocks":[]}),
        ] {
            assert!(patch_native_blocks(&doc, &args).is_err());
        }
        assert!(
            patch_native_blocks(
                &json!({"version":"rf.document/1","blocks":[]}),
                &json!({"operation":"append","blocks":[block("a","A")]})
            )
            .is_err()
        );
    }
}
