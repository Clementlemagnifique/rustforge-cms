//! Analyseur sécurisé d'archives WordPress (ZIP).
//! Traite les packages sans aucune exécution PHP et génère le manifeste TPM-1.0.

use chrono::Utc;
use std::io::{Cursor, Read};
use zip::ZipArchive;

use super::manifest::{
    ActivationPlan, ConversionManifest, DetectedHook, PackageKind, PackageMetadata,
};
use super::registry::CapabilityRegistry;
use crate::error::CoreError;

/// Limites de sécurité strictes contre les attaques Zip-Bomb et déni de service.
const MAX_TOTAL_UNCOMPRESSED_BYTES: u64 = 250 * 1024 * 1024; // 250 Mo max décompressé
const MAX_FILES_COUNT: usize = 2_000; // 2000 fichiers max
const MAX_SINGLE_FILE_BYTES: u64 = 50 * 1024 * 1024; // 50 Mo max par fichier

pub struct PackageAnalyzer;

impl PackageAnalyzer {
    /// Analyse une archive ZIP à partir de son buffer d'octets en mémoire vive.
    pub fn analyze_zip(bytes: &[u8]) -> Result<ConversionManifest, CoreError> {
        let cursor = Cursor::new(bytes);
        let mut archive = ZipArchive::new(cursor).map_err(|e| {
            CoreError::Validation(format!("Archive ZIP invalide ou corrompue: {}", e))
        })?;

        crate::archive::validate(&mut archive)?;
        let file_count = archive.len();
        if file_count > MAX_FILES_COUNT {
            return Err(CoreError::Validation(format!(
                "Sécurité : L'archive contient {} fichiers, dépassant le plafond autorisé de {}",
                file_count, MAX_FILES_COUNT
            )));
        }

        let mut total_uncompressed: u64 = 0;
        let mut file_names: Vec<String> = Vec::with_capacity(file_count);
        let mut has_theme_json = false;
        let mut has_block_templates = false;
        let mut has_php = false;
        let mut style_css_content: Option<String> = None;
        let mut main_php_candidates: Vec<(String, String)> = Vec::new(); // (path, content)

        // Passe 1 : Contrôle de sécurité et inventaire rapide
        for i in 0..file_count {
            let file = archive.by_index(i).map_err(|e| {
                CoreError::Validation(format!("Erreur de lecture du fichier ZIP: {}", e))
            })?;

            let name = file.name().to_string();

            // Rejet des chemins malveillants (Directory Traversal)
            if name.contains("..") || name.starts_with('/') || name.starts_with('\\') {
                return Err(CoreError::Validation(
                    "Sécurité : Chemin suspect détecté dans l'archive (Directory Traversal)"
                        .to_string(),
                ));
            }

            let uncompressed_size = file.size();
            if uncompressed_size > MAX_SINGLE_FILE_BYTES {
                return Err(CoreError::Validation(format!(
                    "Sécurité : Fichier {} trop volumineux ({} octets)",
                    name, uncompressed_size
                )));
            }

            total_uncompressed += uncompressed_size;
            if total_uncompressed > MAX_TOTAL_UNCOMPRESSED_BYTES {
                return Err(CoreError::Validation(
                    "Sécurité : Dépassement du quota total de décompression (risque Zip-Bomb)"
                        .to_string(),
                ));
            }

            let name_lower = name.to_lowercase();
            if name_lower.ends_with("theme.json") {
                has_theme_json = true;
            }
            if name_lower.contains("templates/") && name_lower.ends_with(".html") {
                has_block_templates = true;
            }
            if name_lower.ends_with(".php") {
                has_php = true;
            }

            file_names.push(name);
        }

        // Passe 2 : Extraction sélective pour métadonnées et analyse
        for name in &file_names {
            let name_lower = name.to_lowercase();

            // Lecture de style.css pour thèmes
            if (name_lower == "style.css" || name_lower.ends_with("/style.css"))
                && style_css_content.is_none()
                && let Ok(mut file) = archive.by_name(name)
            {
                let content = crate::archive::read_text(&mut file)?;
                {
                    style_css_content = Some(content);
                }
            }

            // Candidats PHP principaux (racine ou 1er niveau)
            let depth = name.split('/').filter(|s| !s.is_empty()).count();
            if name_lower.ends_with(".php")
                && (depth == 1 || depth == 2)
                && let Ok(mut file) = archive.by_name(name)
            {
                // Limiter la lecture aux premiers 32 Ko pour les en-têtes
                let mut buffer = vec![0u8; 32768];
                if let Ok(n) = file.read(&mut buffer) {
                    let content = String::from_utf8_lossy(&buffer[..n]).to_string();
                    main_php_candidates.push((name.clone(), content));
                }
            }
        }

        // Classification Thème vs Plugin
        let mut kind = PackageKind::Unknown;
        let mut name = String::new();
        let mut version = "1.0.0".to_string();
        let mut author = "Inconnu".to_string();
        let mut description = String::new();
        let mut entry_point = None;

        // Vérification Thème
        if let Some(ref css) = style_css_content
            && let Some(theme_name) = extract_header(css, "Theme Name:")
        {
            name = theme_name;
            version = extract_header(css, "Version:").unwrap_or_else(|| "1.0.0".to_string());
            author = extract_header(css, "Author:").unwrap_or_else(|| "Auteur inconnu".to_string());
            description = extract_header(css, "Description:").unwrap_or_default();

            if has_theme_json || has_block_templates {
                kind = PackageKind::ThemeFse;
            } else {
                kind = PackageKind::ThemeClassic;
            }
        }

        // Vérification Plugin (si ce n'est pas un thème)
        if kind == PackageKind::Unknown {
            for (path, content) in &main_php_candidates {
                if let Some(plugin_name) = extract_header(content, "Plugin Name:") {
                    name = plugin_name;
                    version =
                        extract_header(content, "Version:").unwrap_or_else(|| "1.0.0".to_string());
                    author = extract_header(content, "Author:")
                        .unwrap_or_else(|| "Auteur inconnu".to_string());
                    description = extract_header(content, "Description:").unwrap_or_default();
                    entry_point = Some(path.clone());
                    kind = PackageKind::Plugin;
                    break;
                }
            }
        }

        // Fallback nom si non renseigné
        if name.is_empty() {
            if let Some(first_file) = file_names.first() {
                let first_part = first_file.split('/').next().unwrap_or("package");
                name = first_part.to_string();
            } else {
                name = "package-inconnu".to_string();
            }
        }

        let slug = slug::slugify(&name);

        // Détection statique des hooks et shortcodes dans les fichiers PHP analysés
        let mut detected_hooks: Vec<DetectedHook> = Vec::new();
        let mut detected_shortcodes: Vec<String> = Vec::new();
        let mut detected_hook_names: Vec<String> = Vec::new();

        for (path, content) in &main_php_candidates {
            for (line_idx, line) in content.lines().enumerate() {
                let line_num = line_idx + 1;

                // Actions : add_action( '...', ... )
                if line.contains("add_action")
                    && let Some(hook) = extract_function_arg(line, "add_action")
                {
                    detected_hook_names.push(hook.clone());
                    detected_hooks.push(DetectedHook {
                        hook_type: "action".to_string(),
                        name: hook,
                        handler: extract_second_arg(line),
                        file_path: path.clone(),
                        line_number: line_num,
                    });
                }

                // Filtres : add_filter( '...', ... )
                if line.contains("add_filter")
                    && let Some(hook) = extract_function_arg(line, "add_filter")
                {
                    detected_hook_names.push(hook.clone());
                    detected_hooks.push(DetectedHook {
                        hook_type: "filter".to_string(),
                        name: hook,
                        handler: extract_second_arg(line),
                        file_path: path.clone(),
                        line_number: line_num,
                    });
                }

                // Shortcodes : add_shortcode( '...', ... )
                if line.contains("add_shortcode")
                    && let Some(sc) = extract_function_arg(line, "add_shortcode")
                {
                    detected_shortcodes.push(sc.clone());
                    detected_hooks.push(DetectedHook {
                        hook_type: "shortcode".to_string(),
                        name: sc,
                        handler: extract_second_arg(line),
                        file_path: path.clone(),
                        line_number: line_num,
                    });
                }

                // Post Types : register_post_type( '...', ... )
                if line.contains("register_post_type")
                    && let Some(cpt) = extract_function_arg(line, "register_post_type")
                {
                    detected_hooks.push(DetectedHook {
                        hook_type: "post_type".to_string(),
                        name: cpt,
                        handler: None,
                        file_path: path.clone(),
                        line_number: line_num,
                    });
                }
            }
        }

        // Résolution de compatibilité via CapabilityRegistry
        let (strategy, target_module, capabilities, compatibility) = CapabilityRegistry::resolve(
            &slug,
            &detected_hook_names,
            &detected_shortcodes,
            has_theme_json,
            has_block_templates,
        );

        // Préparation du plan d'activation non exécuté
        let mut activation_plan = ActivationPlan::default();
        if let Some(ref m) = target_module {
            activation_plan.enable_modules.push(m.clone());
            if m == "forms" {
                activation_plan
                    .register_blocks
                    .push("form_embed".to_string());
            } else if m == "ecommerce" {
                activation_plan
                    .register_blocks
                    .push("product_card".to_string());
            } else if m == "lms" {
                activation_plan
                    .register_blocks
                    .push("course_card".to_string());
            }
        }
        if kind == PackageKind::ThemeFse {
            activation_plan.deploy_theme_slug = Some(slug.clone());
            activation_plan.preview_available = true;
        }

        let package = PackageMetadata {
            kind,
            slug,
            name,
            version,
            author,
            description,
            entry_point,
            has_php,
            has_theme_json,
            has_block_templates,
            total_files: file_count,
            uncompressed_size_bytes: total_uncompressed,
        };

        Ok(ConversionManifest {
            tpm_version: "1.0".to_string(),
            analyzed_at: Utc::now(),
            package,
            detected_hooks,
            detected_capabilities: capabilities,
            strategy,
            target_native_module: target_module,
            compatibility,
            activation_plan,
        })
    }
}

/// Extrait une en-tête standard du style "Theme Name: Mon Thème"
fn extract_header(content: &str, header: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim_start_matches(['*', '/', ' ', '#']);
        if let Some(rest) = trimmed.strip_prefix(header) {
            let val = rest.trim();
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

/// Extrait le 1er argument entre guillemets d'un appel PHP type `add_action( 'init', ... )`
fn extract_function_arg(line: &str, fn_name: &str) -> Option<String> {
    let pos = line.find(fn_name)?;
    let after_fn = &line[pos + fn_name.len()..];
    let open_paren = after_fn.find('(')?;
    let inside = &after_fn[open_paren + 1..];

    // Cherche le premier guillemet simple ou double
    let quote_char = inside.chars().find(|&c| c == '\'' || c == '"')?;
    let start_pos = inside.find(quote_char)? + 1;
    let end_pos = inside[start_pos..].find(quote_char)? + start_pos;
    let arg = inside[start_pos..end_pos].trim();
    if !arg.is_empty() {
        Some(arg.to_string())
    } else {
        None
    }
}

/// Extrait le 2e argument (souvent le nom du callback) d'un appel PHP
fn extract_second_arg(line: &str) -> Option<String> {
    let comma = line.find(',')?;
    let after_comma = &line[comma + 1..];
    let quote_char = after_comma.chars().find(|&c| c == '\'' || c == '"')?;
    let start_pos = after_comma.find(quote_char)? + 1;
    let end_pos = after_comma[start_pos..].find(quote_char)? + start_pos;
    let arg = after_comma[start_pos..end_pos].trim();
    if !arg.is_empty() {
        Some(arg.to_string())
    } else {
        None
    }
}
