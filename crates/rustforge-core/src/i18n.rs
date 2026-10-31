//! Multilinguisme souverain, négociation de langue HTTP et catalogues de traduction GNU Gettext.
//!
//! Couvre les exigences MOD-I18N-01 à MOD-I18N-07 :
//! - Distinction stricte entre localisation de l'interface (UI), grappes de contenus (`translation_group_id`)
//!   et routage public / négociation (`Accept-Language`, `hreflang`).
//! - Gestion des langues supportées et actives du CMS (ISO 639-1).
//! - Négociation de langue HTTP selon RFC 9110 avec pondération qualité (`q`).
//! - Dictionnaires PO/JSON conformes GNU Gettext (msgctxt, msgid, msgstr, commentaires, références).
//! - Extraction, fusion non destructive et surcharges de catalogues stockées en base.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{CoreError, CoreResult};

/// Représentation d'une langue supportée par le CMS.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SupportedLanguage {
    pub code: String,
    pub label: String,
    pub is_rtl: bool,
}

/// Catalogue des langues disponibles nativement par défaut dans RustForge CMS.
pub const BUILTIN_LANGUAGES: &[(&str, &str, bool)] = &[
    ("fr", "Français", false),
    ("en", "English", false),
    ("es", "Español", false),
    ("de", "Deutsch", false),
    ("it", "Italiano", false),
    ("pt", "Português", false),
    ("nl", "Nederlands", false),
    ("ar", "العربية", true),
    ("zh", "中文", false),
    ("ja", "日本語", false),
];

/// Retourne la liste complète des langues prédéfinies.
pub fn get_available_languages() -> Vec<SupportedLanguage> {
    BUILTIN_LANGUAGES
        .iter()
        .map(|(code, label, rtl)| SupportedLanguage {
            code: (*code).to_string(),
            label: (*label).to_string(),
            is_rtl: *rtl,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Négociation de langue HTTP (RFC 9110)
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct LangPreference {
    tag: String,
    q: f32,
}

/// Négocie la meilleure langue pour un visiteur selon son en-tête `Accept-Language`,
/// la liste des langues actives configurées sur le CMS et la langue par défaut.
pub fn negotiate_language(
    accept_language_header: Option<&str>,
    active_languages: &[String],
    default_language: &str,
) -> String {
    let Some(header) = accept_language_header else {
        return default_language.to_string();
    };

    let header = header.trim();
    if header.is_empty() {
        return default_language.to_string();
    }

    let mut preferences: Vec<LangPreference> = Vec::new();

    for part in header.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        let mut subparts = part.split(';');
        let lang_tag = subparts.next().unwrap_or("").trim().to_lowercase();
        if lang_tag.is_empty() {
            continue;
        }

        let mut q = 1.0f32;
        for param in subparts {
            let param = param.trim();
            if let Some(val) = param.strip_prefix("q=")
                && let Ok(parsed_q) = val.parse::<f32>()
            {
                q = parsed_q.clamp(0.0, 1.0);
            }
        }

        // Extrait le code primaire (ex: "fr" depuis "fr-FR" ou "fr-CH")
        let primary = lang_tag.split('-').next().unwrap_or(&lang_tag).to_string();

        preferences.push(LangPreference { tag: primary, q });
    }

    // Trie par qualité décroissante (stable)
    preferences.sort_by(|a, b| b.q.partial_cmp(&a.q).unwrap_or(std::cmp::Ordering::Equal));

    for pref in preferences {
        if pref.q <= 0.0 {
            continue;
        }
        if pref.tag == "*" {
            return default_language.to_string();
        }
        if let Some(matched) = active_languages
            .iter()
            .find(|l| l.to_lowercase() == pref.tag)
        {
            return matched.clone();
        }
    }

    default_language.to_string()
}

// ---------------------------------------------------------------------------
// Configuration des langues (Settings)
// ---------------------------------------------------------------------------

const SETTING_LANGUAGES_ACTIVE: &str = "languages.active";
const SETTING_LANGUAGES_DEFAULT: &str = "languages.default";

/// Récupère la langue par défaut du CMS (défaut : "fr").
pub async fn get_default_language(db: &PgPool) -> CoreResult<String> {
    let val = crate::settings::get(db, SETTING_LANGUAGES_DEFAULT).await?;
    let lang = val
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_else(|| "fr".to_string());
    Ok(lang)
}

/// Récupère les langues actuellement activées sur le site.
pub async fn get_active_languages(db: &PgPool) -> CoreResult<Vec<SupportedLanguage>> {
    let val = crate::settings::get(db, SETTING_LANGUAGES_ACTIVE).await?;
    let codes: Vec<String> = val
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_else(|| vec!["fr".to_string()]);

    let available = get_available_languages();
    let mut result = Vec::new();

    for code in codes {
        if let Some(found) = available.iter().find(|l| l.code == code) {
            result.push(found.clone());
        } else {
            result.push(SupportedLanguage {
                code: code.clone(),
                label: code.to_uppercase(),
                is_rtl: false,
            });
        }
    }

    if result.is_empty() {
        result.push(SupportedLanguage {
            code: "fr".to_string(),
            label: "Français".to_string(),
            is_rtl: false,
        });
    }

    Ok(result)
}

/// Met à jour la configuration des langues actives et de la langue par défaut.
pub async fn set_languages_config(
    db: &PgPool,
    active_codes: &[String],
    default_code: &str,
) -> CoreResult<()> {
    if active_codes.is_empty() {
        return Err(CoreError::validation(
            "Au moins une langue active doit être sélectionnée",
        ));
    }

    let default_clean = default_code.trim().to_lowercase();
    if !active_codes
        .iter()
        .any(|c| c.to_lowercase() == default_clean)
    {
        return Err(CoreError::validation(format!(
            "La langue par défaut '{default_clean}' doit figurer parmi les langues actives"
        )));
    }

    let active_clean: Vec<String> = active_codes
        .iter()
        .map(|c| c.trim().to_lowercase())
        .collect();

    let mut tx = db.begin().await?;

    sqlx::query(
        "INSERT INTO settings (key, value, updated_at)
         VALUES ($1, $2, now())
         ON CONFLICT (key) DO UPDATE SET value = $2, updated_at = now()",
    )
    .bind(SETTING_LANGUAGES_ACTIVE)
    .bind(serde_json::to_value(&active_clean).map_err(|e| CoreError::Internal(e.into()))?)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO settings (key, value, updated_at)
         VALUES ($1, $2, now())
         ON CONFLICT (key) DO UPDATE SET value = $2, updated_at = now()",
    )
    .bind(SETTING_LANGUAGES_DEFAULT)
    .bind(serde_json::to_value(&default_clean).map_err(|e| CoreError::Internal(e.into()))?)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Catalogue de traduction GNU Gettext & Modèle de données (MOD-I18N-02 à 05)
// ---------------------------------------------------------------------------

/// Entrée individuelle d'un catalogue de traduction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogItem {
    pub msgid: String,
    pub msgstr: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub comments: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<String>,
}

/// Catalogue complet de chaînes pour une langue donnée.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationCatalog {
    pub locale: String,
    pub items: Vec<CatalogItem>,
    pub updated_at: chrono::DateTime<Utc>,
}

impl Default for TranslationCatalog {
    fn default() -> Self {
        Self {
            locale: "fr".to_string(),
            items: Vec::new(),
            updated_at: Utc::now(),
        }
    }
}

/// Échappe une chaîne pour le format PO.
fn escape_po_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

/// Déséchappe une chaîne extraite d'une ligne PO.
fn unescape_po_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Exporte un catalogue vers le format textuel standard GNU Gettext (.po).
pub fn export_po(catalog: &TranslationCatalog) -> String {
    let mut out = String::new();
    let now = Utc::now().format("%Y-%m-%d %H:%M%z").to_string();

    // En-tête PO standard
    out.push_str("msgid \"\"\n");
    out.push_str("msgstr \"\"\n");
    out.push_str("\"Project-Id-Version: RustForge CMS\\n\"\n");
    out.push_str("\"Report-Msgid-Bugs-To: \\n\"\n");
    out.push_str(&format!("\"POT-Creation-Date: {now}\\n\"\n"));
    out.push_str(&format!("\"PO-Revision-Date: {now}\\n\"\n"));
    out.push_str(&format!("\"Language: {}\\n\"\n", catalog.locale));
    out.push_str("\"MIME-Version: 1.0\\n\"\n");
    out.push_str("\"Content-Type: text/plain; charset=UTF-8\\n\"\n");
    out.push_str("\"Content-Transfer-Encoding: 8bit\\n\"\n");
    out.push_str("\"X-Generator: RustForge Sovereign Native i18n\\n\"\n\n");

    for item in &catalog.items {
        for comment in &item.comments {
            out.push_str(&format!("#. {comment}\n"));
        }
        for r in &item.references {
            out.push_str(&format!("#: {r}\n"));
        }
        if let Some(ctx) = &item.context {
            out.push_str(&format!("msgctxt \"{}\"\n", escape_po_string(ctx)));
        }
        out.push_str(&format!("msgid \"{}\"\n", escape_po_string(&item.msgid)));
        out.push_str(&format!(
            "msgstr \"{}\"\n\n",
            escape_po_string(&item.msgstr)
        ));
    }

    out
}

/// Parse un fichier standard GNU Gettext (.po) et extrait le catalogue.
pub fn parse_po(po_content: &str, locale: &str) -> CoreResult<TranslationCatalog> {
    let mut items = Vec::new();
    let mut current_comments = Vec::new();
    let mut current_references = Vec::new();
    let mut current_context: Option<String> = None;
    let mut current_msgid: Option<String> = None;
    let mut current_msgstr: Option<String> = None;
    let mut parsing_mode = 0; // 0: none, 1: msgctxt, 2: msgid, 3: msgstr

    for line in po_content.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            if let (Some(id), Some(str_val)) = (current_msgid.take(), current_msgstr.take())
                && !id.is_empty()
            {
                items.push(CatalogItem {
                    msgid: id,
                    msgstr: str_val,
                    context: current_context.take(),
                    comments: std::mem::take(&mut current_comments),
                    references: std::mem::take(&mut current_references),
                });
            }
            parsing_mode = 0;
            continue;
        }

        if let Some(comment) = trimmed.strip_prefix("#.") {
            current_comments.push(comment.trim().to_string());
            continue;
        }

        if let Some(reference) = trimmed.strip_prefix("#:") {
            current_references.push(reference.trim().to_string());
            continue;
        }

        if trimmed.starts_with('#') {
            continue; // Autres commentaires (#~ obsolète, #, etc.)
        }

        if let Some(rest) = trimmed.strip_prefix("msgctxt") {
            let val = extract_quoted_string(rest.trim());
            current_context = Some(val);
            parsing_mode = 1;
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("msgid") {
            let val = extract_quoted_string(rest.trim());
            current_msgid = Some(val);
            parsing_mode = 2;
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("msgstr") {
            let val = extract_quoted_string(rest.trim());
            current_msgstr = Some(val);
            parsing_mode = 3;
            continue;
        }

        if trimmed.starts_with('"') {
            let val = extract_quoted_string(trimmed);
            match parsing_mode {
                1 => {
                    if let Some(ctx) = current_context.as_mut() {
                        ctx.push_str(&val);
                    }
                }
                2 => {
                    if let Some(id) = current_msgid.as_mut() {
                        id.push_str(&val);
                    }
                }
                3 => {
                    if let Some(st) = current_msgstr.as_mut() {
                        st.push_str(&val);
                    }
                }
                _ => {}
            }
        }
    }

    // Dernier élément
    if let (Some(id), Some(str_val)) = (current_msgid, current_msgstr)
        && !id.is_empty()
    {
        items.push(CatalogItem {
            msgid: id,
            msgstr: str_val,
            context: current_context,
            comments: current_comments,
            references: current_references,
        });
    }

    Ok(TranslationCatalog {
        locale: locale.to_string(),
        items,
        updated_at: Utc::now(),
    })
}

fn extract_quoted_string(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        unescape_po_string(&s[1..s.len() - 1])
    } else {
        s.to_string()
    }
}

// ---------------------------------------------------------------------------
// Persistance et synchronisation des catalogues
// ---------------------------------------------------------------------------

fn catalog_setting_key(locale: &str) -> String {
    format!("i18n.catalog.{}", locale.to_lowercase())
}

/// Charge le catalogue d'une langue depuis la base ou extrait le catalogue de base.
pub async fn get_catalog(db: &PgPool, locale: &str) -> CoreResult<TranslationCatalog> {
    let key = catalog_setting_key(locale);
    let val = crate::settings::get(db, &key).await?;

    if let Some(json_val) = val
        && let Ok(cat) = serde_json::from_value::<TranslationCatalog>(json_val)
    {
        return Ok(cat);
    }

    Ok(extract_default_catalog(locale))
}

/// Enregistre un catalogue de traduction en base.
pub async fn save_catalog(db: &PgPool, catalog: &TranslationCatalog) -> CoreResult<()> {
    let key = catalog_setting_key(&catalog.locale);
    let val = serde_json::to_value(catalog).map_err(|e| CoreError::Internal(e.into()))?;

    sqlx::query(
        "INSERT INTO settings (key, value, updated_at)
         VALUES ($1, $2, now())
         ON CONFLICT (key) DO UPDATE SET value = $2, updated_at = now()",
    )
    .bind(&key)
    .bind(val)
    .execute(db)
    .await?;

    Ok(())
}

/// Fusionne un catalogue source avec un catalogue existant : conserve les traductions
/// renseignées dans `target` et ajoute les nouveaux `msgid` du socle.
pub fn sync_catalog(target: &mut TranslationCatalog, source: &TranslationCatalog) {
    for src_item in &source.items {
        if let Some(existing) = target
            .items
            .iter_mut()
            .find(|item| item.msgid == src_item.msgid && item.context == src_item.context)
        {
            // Met à jour les commentaires et références de développement sans écraser la traduction
            existing.comments = src_item.comments.clone();
            existing.references = src_item.references.clone();
        } else {
            // Nouveau texte non traduit
            target.items.push(src_item.clone());
        }
    }
    target.updated_at = Utc::now();
}

/// Extrait les chaînes d'interface par défaut de RustForge CMS pour une locale donnée.
pub fn extract_default_catalog(locale: &str) -> TranslationCatalog {
    let is_en = locale.to_lowercase() == "en";

    let defaults = vec![
        (
            "Tableau de bord",
            if is_en {
                "Dashboard"
            } else {
                "Tableau de bord"
            },
            "Navigation",
        ),
        (
            "Articles",
            if is_en { "Posts" } else { "Articles" },
            "Navigation",
        ),
        ("Pages", "Pages", "Navigation"),
        (
            "Médias",
            if is_en { "Media" } else { "Médias" },
            "Navigation",
        ),
        (
            "Commentaires",
            if is_en { "Comments" } else { "Commentaires" },
            "Navigation",
        ),
        (
            "Formulaires",
            if is_en { "Forms" } else { "Formulaires" },
            "Navigation",
        ),
        (
            "Apparence",
            if is_en { "Appearance" } else { "Apparence" },
            "Navigation",
        ),
        (
            "Extensions",
            if is_en { "Plugins" } else { "Extensions" },
            "Navigation",
        ),
        (
            "Utilisateurs",
            if is_en { "Users" } else { "Utilisateurs" },
            "Navigation",
        ),
        (
            "Outils",
            if is_en { "Tools" } else { "Outils" },
            "Navigation",
        ),
        (
            "Réglages",
            if is_en { "Settings" } else { "Réglages" },
            "Navigation",
        ),
        (
            "Statistiques",
            if is_en { "Analytics" } else { "Statistiques" },
            "Navigation",
        ),
        (
            "Traductions",
            if is_en { "Translations" } else { "Traductions" },
            "Navigation",
        ),
        (
            "Publier",
            if is_en { "Publish" } else { "Publier" },
            "Actions",
        ),
        (
            "Enregistrer le brouillon",
            if is_en {
                "Save Draft"
            } else {
                "Enregistrer le brouillon"
            },
            "Actions",
        ),
        (
            "Mettre à jour",
            if is_en { "Update" } else { "Mettre à jour" },
            "Actions",
        ),
        (
            "Supprimer",
            if is_en { "Delete" } else { "Supprimer" },
            "Actions",
        ),
        (
            "Restaurer",
            if is_en { "Restore" } else { "Restaurer" },
            "Actions",
        ),
        (
            "Rechercher",
            if is_en { "Search" } else { "Rechercher" },
            "Actions",
        ),
        (
            "Filtrer",
            if is_en { "Filter" } else { "Filtrer" },
            "Actions",
        ),
        (
            "Annuler",
            if is_en { "Cancel" } else { "Annuler" },
            "Actions",
        ),
        (
            "Confirmer",
            if is_en { "Confirm" } else { "Confirmer" },
            "Actions",
        ),
        (
            "Exporter",
            if is_en { "Export" } else { "Exporter" },
            "Actions",
        ),
        (
            "Importer",
            if is_en { "Import" } else { "Importer" },
            "Actions",
        ),
        (
            "Publié",
            if is_en { "Published" } else { "Publié" },
            "Status",
        ),
        (
            "Brouillon",
            if is_en { "Draft" } else { "Brouillon" },
            "Status",
        ),
        (
            "Corbeille",
            if is_en { "Trash" } else { "Corbeille" },
            "Status",
        ),
        (
            "En attente",
            if is_en { "Pending" } else { "En attente" },
            "Status",
        ),
        ("Privé", if is_en { "Private" } else { "Privé" }, "Status"),
        (
            "Lire la suite",
            if is_en { "Read more" } else { "Lire la suite" },
            "Editorial",
        ),
        (
            "Catégories",
            if is_en { "Categories" } else { "Catégories" },
            "Taxonomie",
        ),
        (
            "Mots-clés",
            if is_en { "Tags" } else { "Mots-clés" },
            "Taxonomie",
        ),
    ];

    let items = defaults
        .into_iter()
        .map(|(msgid, msgstr, ctx)| CatalogItem {
            msgid: msgid.to_string(),
            msgstr: msgstr.to_string(),
            context: Some(ctx.to_string()),
            comments: vec!["Chaîne d'interface native RustForge".to_string()],
            references: vec!["admin/src/i18n/messages.fr.ts".to_string()],
        })
        .collect();

    TranslationCatalog {
        locale: locale.to_string(),
        items,
        updated_at: Utc::now(),
    }
}

// ---------------------------------------------------------------------------
// Tests unitaires
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_negotiate_language_direct_match() {
        let active = vec!["fr".to_string(), "en".to_string(), "es".to_string()];
        let header = Some("fr-FR,fr;q=0.9,en-US;q=0.8,en;q=0.7");
        let lang = negotiate_language(header, &active, "fr");
        assert_eq!(lang, "fr");
    }

    #[test]
    fn test_negotiate_language_prefers_higher_q() {
        let active = vec!["fr".to_string(), "en".to_string(), "de".to_string()];
        let header = Some("fr;q=0.5, en;q=0.9, de;q=0.7");
        let lang = negotiate_language(header, &active, "fr");
        assert_eq!(lang, "en");
    }

    #[test]
    fn test_negotiate_language_fallback_when_no_match() {
        let active = vec!["fr".to_string(), "es".to_string()];
        let header = Some("ja,zh;q=0.9,ru;q=0.8");
        let lang = negotiate_language(header, &active, "fr");
        assert_eq!(lang, "fr");
    }

    #[test]
    fn test_negotiate_language_empty_header() {
        let active = vec!["fr".to_string(), "en".to_string()];
        let lang = negotiate_language(None, &active, "fr");
        assert_eq!(lang, "fr");
    }

    #[test]
    fn test_po_export_and_parse_roundtrip() {
        let original = TranslationCatalog {
            locale: "es".to_string(),
            items: vec![
                CatalogItem {
                    msgid: "Publier".to_string(),
                    msgstr: "Publicar".to_string(),
                    context: Some("Actions".to_string()),
                    comments: vec!["Bouton de validation".to_string()],
                    references: vec!["src/editor.rs:42".to_string()],
                },
                CatalogItem {
                    msgid: "Tableau de bord".to_string(),
                    msgstr: "Panel de control".to_string(),
                    context: Some("Navigation".to_string()),
                    comments: vec![],
                    references: vec![],
                },
            ],
            updated_at: Utc::now(),
        };

        let po_text = export_po(&original);
        assert!(po_text.contains("msgid \"Publier\""));
        assert!(po_text.contains("msgstr \"Publicar\""));
        assert!(po_text.contains("msgctxt \"Actions\""));

        let parsed = parse_po(&po_text, "es").expect("PO parse should succeed");
        assert_eq!(parsed.items.len(), 2);
        assert_eq!(parsed.items[0].msgid, "Publier");
        assert_eq!(parsed.items[0].msgstr, "Publicar");
        assert_eq!(parsed.items[0].context.as_deref(), Some("Actions"));
        assert_eq!(parsed.items[1].msgid, "Tableau de bord");
        assert_eq!(parsed.items[1].msgstr, "Panel de control");

        // Sync test
        let mut target = TranslationCatalog {
            locale: "es".to_string(),
            items: vec![CatalogItem {
                msgid: "Publier".to_string(),
                msgstr: "Publicar".to_string(),
                context: Some("Actions".to_string()),
                comments: vec![],
                references: vec![],
            }],
            updated_at: Utc::now(),
        };
        sync_catalog(&mut target, &original);
        assert_eq!(target.items.len(), 2);
        assert_eq!(target.items[0].msgstr, "Publicar");
    }

    #[test]
    fn test_po_parser_multiline() {
        let po_content = r#"
msgid ""
msgstr ""
"Language: en\n"

msgid "Un texte très long "
"qui continue sur plusieurs lignes."
msgstr "A very long text "
"that continues across lines."
"#;
        let cat = parse_po(po_content, "en").unwrap();
        assert_eq!(cat.items.len(), 1);
        assert_eq!(
            cat.items[0].msgid,
            "Un texte très long qui continue sur plusieurs lignes."
        );
        assert_eq!(
            cat.items[0].msgstr,
            "A very long text that continues across lines."
        );
    }
}
