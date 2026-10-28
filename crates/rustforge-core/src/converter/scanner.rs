//! Scanner Dynamique d'Installations, Extensions et Thèmes WordPress.
//!
//! Analyse statique sécurisée sans aucune exécution de runtime PHP :
//! - Détection de la version réelle du cœur WordPress via `wp-includes/version.php`
//! - Extraction des métadonnées des plugins installés via les en-têtes standard
//! - Détection des thèmes FSE / blocs (`theme.json`)
//! - Mappage en temps réel avec le `CapabilityRegistry` natif de RustForge
//! - Évaluation de la couverture native et de l'exposition critique à la crise PHP2Shell.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use super::manifest::ConversionStrategyKind;
use super::registry::CapabilityRegistry;
use crate::error::CoreError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginScanItem {
    pub slug: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub native_module: Option<String>,
    pub strategy: ConversionStrategyKind,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeScanItem {
    pub slug: String,
    pub name: String,
    pub version: String,
    pub is_fse_block_theme: bool,
    pub has_theme_json: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WpScanResult {
    pub target: String,
    pub target_type: String,
    pub wp_version: Option<String>,
    pub db_prefix: Option<String>,
    pub active_theme: Option<String>,
    pub plugins: Vec<PluginScanItem>,
    pub themes: Vec<ThemeScanItem>,
    pub media_count: usize,
    pub native_coverage_pct: f64,
    pub php2shell_threat_level: String,
    pub estimated_speedup: &'static str,
}

pub struct WordPressScanner;

impl WordPressScanner {
    /// Analyse de manière dynamique une cible (répertoire WordPress, dossier de plugins/thèmes, archive ZIP ou URL distante).
    pub async fn scan(target_str: &str) -> Result<WpScanResult, CoreError> {
        let trimmed = target_str.trim();

        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            Self::scan_remote_url(trimmed).await
        } else {
            let path = Path::new(trimmed);
            if !path.exists() {
                return Err(CoreError::Validation(format!(
                    "Cible introuvable : le chemin « {} » n'existe pas.",
                    trimmed
                )));
            }

            if path.is_file() && trimmed.ends_with(".zip") {
                Self::scan_zip_file(path)
            } else if path.is_dir() {
                Self::scan_local_directory(path)
            } else {
                Err(CoreError::Validation(format!(
                    "Type de cible non pris en charge pour « {} ».",
                    trimmed
                )))
            }
        }
    }

    /// Analyse d'une installation locale ou d'un sous-dossier WordPress.
    fn scan_local_directory(dir: &Path) -> Result<WpScanResult, CoreError> {
        let mut wp_version = None;
        let mut db_prefix = None;
        let mut plugins = Vec::new();
        let mut themes = Vec::new();
        let mut media_count = 0;
        let mut target_type = "generic_directory".to_string();

        // 1. Détection racine WordPress
        let version_php = dir.join("wp-includes").join("version.php");
        if version_php.exists() {
            target_type = "wordpress_root".to_string();
            if let Ok(content) = fs::read_to_string(&version_php) {
                wp_version = Self::extract_php_variable(&content, "$wp_version");
            }

            let wp_config = dir.join("wp-config.php");
            let wp_config_sample = dir.join("wp-config-sample.php");
            let config_to_read = if wp_config.exists() {
                Some(wp_config)
            } else if wp_config_sample.exists() {
                Some(wp_config_sample)
            } else {
                None
            };

            if let Some(cfg) = config_to_read
                && let Ok(content) = fs::read_to_string(cfg)
            {
                db_prefix = Self::extract_php_variable(&content, "$table_prefix");
            }

            // Plugins WordPress
            let plugins_dir = dir.join("wp-content").join("plugins");
            if plugins_dir.is_dir() {
                plugins = Self::scan_plugins_directory(&plugins_dir);
            }

            // Thèmes WordPress
            let themes_dir = dir.join("wp-content").join("themes");
            if themes_dir.is_dir() {
                themes = Self::scan_themes_directory(&themes_dir);
            }

            // Médias
            let uploads_dir = dir.join("wp-content").join("uploads");
            if uploads_dir.is_dir() {
                media_count = Self::count_files_recursive(&uploads_dir);
            }
        } else if dir
            .file_name()
            .map(|n| n == "plugins" || n == "plugins-ref")
            .unwrap_or(false)
        {
            target_type = "plugins_directory".to_string();
            plugins = Self::scan_plugins_directory(dir);
        } else if dir
            .file_name()
            .map(|n| n == "themes" || n == "theme-ref")
            .unwrap_or(false)
        {
            target_type = "themes_directory".to_string();
            themes = Self::scan_themes_directory(dir);
        } else {
            // Tente de scanner comme un plugin individuel ou inspecte récursivement
            if let Some(plugin) = Self::scan_single_plugin_dir(dir) {
                target_type = "single_plugin".to_string();
                plugins.push(plugin);
            } else if let Some(theme) = Self::scan_single_theme_dir(dir) {
                target_type = "single_theme".to_string();
                themes.push(theme);
            } else {
                // Tente d'analyser les sous-dossiers comme des plugins
                plugins = Self::scan_plugins_directory(dir);
            }
        }

        let native_count = plugins
            .iter()
            .filter(|p| p.strategy == ConversionStrategyKind::NativeReuse)
            .count();
        let native_coverage_pct = if plugins.is_empty() {
            0.0
        } else {
            (native_count as f64 / plugins.len() as f64) * 100.0
        };

        let active_theme = None; // A directory listing cannot identify the active theme.

        Ok(WpScanResult {
            target: dir.to_string_lossy().to_string(),
            target_type,
            wp_version,
            db_prefix,
            active_theme,
            plugins,
            themes,
            media_count,
            native_coverage_pct,
            php2shell_threat_level:
                "Non évalué : cet inventaire ne prouve ni vulnérabilité ni absence de vulnérabilité"
                    .to_string(),
            estimated_speedup: "Non mesuré : benchmark avant/après requis",
        })
    }

    /// Scanne un dossier contenant plusieurs plugins.
    pub fn scan_plugins_directory(plugins_dir: &Path) -> Vec<PluginScanItem> {
        let mut list = Vec::new();
        let Ok(entries) = fs::read_dir(plugins_dir) else {
            return list;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(item) = Self::scan_single_plugin_dir(&path) {
                    list.push(item);
                }
            } else if path.extension().map(|e| e == "php").unwrap_or(false) {
                let file_name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                if file_name != "index.php"
                    && let Some(item) = Self::scan_single_plugin_file(&path)
                {
                    list.push(item);
                }
            }
        }

        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
    }

    /// Scanne un plugin individuel dans son propre dossier.
    fn scan_single_plugin_dir(dir: &Path) -> Option<PluginScanItem> {
        let slug = dir.file_name()?.to_string_lossy().to_string();
        let Ok(files) = fs::read_dir(dir) else {
            return None;
        };

        let mut php_files = Vec::new();
        for file in files.flatten() {
            let p = file.path();
            if p.extension().map(|e| e == "php").unwrap_or(false) {
                php_files.push(p);
            }
        }

        // Tenter d'abord slug.php
        let exact_candidate = dir.join(format!("{}.php", slug));
        if exact_candidate.exists()
            && let Some(item) = Self::parse_plugin_header_file(&exact_candidate, &slug)
        {
            return Some(item);
        }

        // Tenter les autres fichiers PHP du répertoire
        for p in php_files {
            if let Some(item) = Self::parse_plugin_header_file(&p, &slug) {
                return Some(item);
            }
        }

        // Si aucun en-tête n'est trouvé, créer une entrée basée sur le nom de dossier
        let (native_mod, strategy, caps) = Self::match_capabilities(&slug, &slug);
        Some(PluginScanItem {
            slug: slug.clone(),
            name: slug.clone(),
            version: "indéterminée".to_string(),
            description: "Extension WordPress détectée".to_string(),
            native_module: native_mod,
            strategy,
            capabilities: caps,
        })
    }

    /// Scanne un plugin sous forme de fichier PHP unique (ex: hello.php).
    fn scan_single_plugin_file(file: &Path) -> Option<PluginScanItem> {
        let stem = file.file_stem()?.to_string_lossy().to_string();
        Self::parse_plugin_header_file(file, &stem)
    }

    /// Analyse l'en-tête standard WordPress d'un fichier PHP.
    fn parse_plugin_header_file(file: &Path, slug: &str) -> Option<PluginScanItem> {
        let Ok(bytes) = fs::read(file) else {
            return None;
        };
        let slice = if bytes.len() > 8192 {
            &bytes[..8192]
        } else {
            &bytes
        };
        let content = String::from_utf8_lossy(slice);

        let name = Self::extract_header(&content, "Plugin Name")?;
        let version =
            Self::extract_header(&content, "Version").unwrap_or_else(|| "1.0.0".to_string());
        let description = Self::extract_header(&content, "Description").unwrap_or_default();

        let (native_module, strategy, capabilities) = Self::match_capabilities(slug, &name);

        Some(PluginScanItem {
            slug: slug.to_string(),
            name,
            version,
            description,
            native_module,
            strategy,
            capabilities,
        })
    }

    /// Scanne un dossier de thèmes WordPress.
    pub fn scan_themes_directory(themes_dir: &Path) -> Vec<ThemeScanItem> {
        let mut list = Vec::new();
        let Ok(entries) = fs::read_dir(themes_dir) else {
            return list;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir()
                && let Some(item) = Self::scan_single_theme_dir(&path)
            {
                list.push(item);
            }
        }

        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
    }

    /// Scanne un thème WordPress individuel.
    fn scan_single_theme_dir(dir: &Path) -> Option<ThemeScanItem> {
        let slug = dir.file_name()?.to_string_lossy().to_string();
        let style_css = dir.join("style.css");
        let theme_json = dir.join("theme.json");
        let has_theme_json = theme_json.exists();

        let (name, version) = if style_css.exists() {
            let content = fs::read_to_string(&style_css).unwrap_or_default();
            let n = Self::extract_header(&content, "Theme Name").unwrap_or_else(|| slug.clone());
            let v =
                Self::extract_header(&content, "Version").unwrap_or_else(|| "1.0.0".to_string());
            (n, v)
        } else {
            (slug.clone(), "1.0.0".to_string())
        };

        Some(ThemeScanItem {
            slug,
            name,
            version,
            is_fse_block_theme: has_theme_json,
            has_theme_json,
        })
    }

    /// Mappe un plugin contre le `CapabilityRegistry` pour déterminer le remplacement natif RustForge.
    fn match_capabilities(
        slug: &str,
        name: &str,
    ) -> (Option<String>, ConversionStrategyKind, Vec<String>) {
        let slug_lower = slug.to_lowercase();
        let name_lower = name.to_lowercase();

        let all_caps = CapabilityRegistry::all_capabilities();
        let mut matched_caps = Vec::new();
        let mut native_module = None;

        for cap in all_caps {
            let matches_plugin = cap
                .matching_plugins
                .iter()
                .any(|&p| slug_lower.contains(p) || name_lower.contains(p));

            if matches_plugin {
                matched_caps.push(cap.id.to_string());
                if native_module.is_none() && cap.target_module.is_some() {
                    native_module = cap.target_module.map(|m| m.to_string());
                }
            }
        }

        // Détection heuristique supplémentaire pour les modules standards RustForge
        if native_module.is_none() {
            if slug_lower.contains("form") || name_lower.contains("form") {
                native_module = Some("forms".to_string());
                matched_caps.push("forms.builder_and_submission".to_string());
            } else if slug_lower.contains("shop")
                || slug_lower.contains("commerce")
                || slug_lower.contains("download")
            {
                native_module = Some("ecommerce".to_string());
                matched_caps.push("commerce.digital_downloads".to_string());
            } else if slug_lower.contains("lms")
                || slug_lower.contains("course")
                || slug_lower.contains("learn")
            {
                native_module = Some("lms".to_string());
                matched_caps.push("lms.courses_and_lessons".to_string());
            } else if slug_lower.contains("media")
                || slug_lower.contains("image")
                || slug_lower.contains("smush")
            {
                native_module = Some("media".to_string());
                matched_caps.push("media.folders_and_webp".to_string());
            } else if slug_lower.contains("seo") || slug_lower.contains("sitemap") {
                native_module = Some("seo".to_string());
                matched_caps.push("seo.metadata_and_geo".to_string());
            } else if slug_lower.contains("security") || slug_lower.contains("firewall") {
                native_module = Some("security".to_string());
                matched_caps.push("security.zero_trust".to_string());
            } else if slug_lower.contains("cache") {
                native_module = Some("caching".to_string());
                matched_caps.push("performance.moka_cache".to_string());
            } else if slug_lower.contains("translate") || slug_lower.contains("lang") {
                native_module = Some("i18n".to_string());
                matched_caps.push("i18n.multilingual".to_string());
            } else if slug_lower.contains("publishpress")
                || slug_lower.contains("editorial")
                || slug_lower.contains("schedule")
            {
                native_module = Some("scheduler".to_string());
                matched_caps.push("editorial.scheduler".to_string());
            } else if slug_lower.contains("mail") || slug_lower.contains("newsletter") {
                native_module = Some("automations".to_string());
                matched_caps.push("marketing.email_campaigns".to_string());
            } else if slug_lower.contains("stat") || slug_lower.contains("analytic") {
                native_module = Some("analytics".to_string());
                matched_caps.push("analytics.privacy_friendly".to_string());
            }
        }

        let strategy = if native_module.is_some() {
            ConversionStrategyKind::NativeReuse
        } else {
            ConversionStrategyKind::ControlledWasm
        };

        (native_module, strategy, matched_caps)
    }

    /// Scanne un package ZIP (analyseur d'archive).
    fn scan_zip_file(zip_path: &Path) -> Result<WpScanResult, CoreError> {
        let bytes = fs::read(zip_path).map_err(|e| {
            CoreError::Validation(format!("Impossible de lire le fichier ZIP : {}", e))
        })?;

        let manifest = super::analyzer::PackageAnalyzer::analyze_zip(&bytes)?;

        let mut plugins = Vec::new();
        let mut themes = Vec::new();

        match manifest.package.kind {
            super::manifest::PackageKind::ThemeFse | super::manifest::PackageKind::ThemeClassic => {
                themes.push(ThemeScanItem {
                    slug: manifest.package.slug.clone(),
                    name: manifest.package.name.clone(),
                    version: manifest.package.version.clone(),
                    is_fse_block_theme: manifest.package.has_theme_json,
                    has_theme_json: manifest.package.has_theme_json,
                });
            }
            super::manifest::PackageKind::Plugin => {
                let (native_mod, strategy, caps) =
                    Self::match_capabilities(&manifest.package.slug, &manifest.package.name);
                plugins.push(PluginScanItem {
                    slug: manifest.package.slug.clone(),
                    name: manifest.package.name.clone(),
                    version: manifest.package.version.clone(),
                    description: manifest.package.description.clone(),
                    native_module: native_mod,
                    strategy,
                    capabilities: caps,
                });
            }
            super::manifest::PackageKind::Unknown => {}
        }

        let native_count = plugins
            .iter()
            .filter(|p| p.strategy == ConversionStrategyKind::NativeReuse)
            .count();
        let native_coverage_pct = if plugins.is_empty() {
            0.0
        } else {
            (native_count as f64 / plugins.len() as f64) * 100.0
        };

        Ok(WpScanResult {
            target: zip_path.to_string_lossy().to_string(),
            target_type: "zip_archive".to_string(),
            wp_version: None,
            db_prefix: None,
            active_theme: None,
            plugins,
            themes,
            media_count: 0,
            native_coverage_pct,
            php2shell_threat_level:
                "Non évalué : cet inventaire ne prouve ni vulnérabilité ni absence de vulnérabilité"
                    .to_string(),
            estimated_speedup: "Non mesuré : benchmark avant/après requis",
        })
    }

    /// Scanne une URL distante WordPress via HTTP.
    async fn scan_remote_url(url: &str) -> Result<WpScanResult, CoreError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| CoreError::Validation(format!("Erreur client HTTP : {}", e)))?;

        let res = client
            .get(url)
            .header(
                "User-Agent",
                "RustForge-Scanner/2027 (+https://rustforge.dev)",
            )
            .send()
            .await
            .map_err(|e| {
                CoreError::Validation(format!("Échec de connexion vers {} : {}", url, e))
            })?;

        let text = res.text().await.unwrap_or_default();

        // 1. Détection de version WP via <meta name="generator" content="WordPress X.Y.Z" />
        let wp_version = if let Some(pos) = text.find("name=\"generator\"") {
            let slice = &text[pos..pos + 120.min(text.len() - pos)];
            if let Some(content_pos) = slice.find("content=\"WordPress ") {
                let ver_str = &slice[content_pos + "content=\"WordPress ".len()..];
                ver_str.split('"').next().map(|s| s.to_string())
            } else {
                None
            }
        } else {
            None
        };

        // 2. Détection de thèmes et plugins via les chemins /wp-content/
        let mut plugins_found = std::collections::HashSet::new();
        let mut theme_found = None;

        for line in text.split(&['"', '\''][..]) {
            if let Some(pos) = line.find("/wp-content/plugins/") {
                let rest = &line[pos + "/wp-content/plugins/".len()..];
                if let Some(slug) = rest.split('/').next()
                    && !slug.is_empty()
                    && !slug.contains('?')
                {
                    plugins_found.insert(slug.to_string());
                }
            } else if let Some(pos) = line.find("/wp-content/themes/") {
                let rest = &line[pos + "/wp-content/themes/".len()..];
                if let Some(slug) = rest.split('/').next()
                    && !slug.is_empty()
                    && !slug.contains('?')
                    && theme_found.is_none()
                {
                    theme_found = Some(slug.to_string());
                }
            }
        }

        let mut plugins = Vec::new();
        for slug in plugins_found {
            let (native_mod, strategy, caps) = Self::match_capabilities(&slug, &slug);
            plugins.push(PluginScanItem {
                slug: slug.clone(),
                name: slug.clone(),
                version: "détectée en ligne".to_string(),
                description: "Extension identifiée via l'empreinte web HTML".to_string(),
                native_module: native_mod,
                strategy,
                capabilities: caps,
            });
        }

        let mut themes = Vec::new();
        if let Some(slug) = theme_found {
            themes.push(ThemeScanItem {
                slug: slug.clone(),
                name: slug,
                version: "active".to_string(),
                is_fse_block_theme: true,
                has_theme_json: true,
            });
        }

        let native_count = plugins
            .iter()
            .filter(|p| p.strategy == ConversionStrategyKind::NativeReuse)
            .count();
        let native_coverage_pct = if plugins.is_empty() {
            0.0
        } else {
            (native_count as f64 / plugins.len() as f64) * 100.0
        };

        Ok(WpScanResult {
            target: url.to_string(),
            target_type: "remote_url".to_string(),
            wp_version,
            db_prefix: None,
            active_theme: None,
            plugins,
            themes,
            media_count: 0,
            native_coverage_pct,
            php2shell_threat_level:
                "Non évalué : cet inventaire ne prouve ni vulnérabilité ni absence de vulnérabilité"
                    .to_string(),
            estimated_speedup: "Non mesuré : benchmark avant/après requis",
        })
    }

    /// Extrait une variable PHP simple (ex: `$wp_version = '7.1.3';`).
    fn extract_php_variable(content: &str, var_name: &str) -> Option<String> {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with(var_name)
                && let Some(eq_pos) = trimmed.find('=')
            {
                let val_part = trimmed[eq_pos + 1..].trim().trim_end_matches(';');
                let clean = val_part.trim_matches(|c| c == '\'' || c == '"' || c == ' ');
                if !clean.is_empty() {
                    return Some(clean.to_string());
                }
            }
        }
        None
    }

    /// Extrait une valeur d'en-tête PHP (ex: `Plugin Name: ...`).
    fn extract_header(content: &str, header_name: &str) -> Option<String> {
        for line in content.lines() {
            let trimmed = line.trim_start_matches(['*', '/', '#', ' ']);
            if let Some(colon_pos) = trimmed.find(':') {
                let key = trimmed[..colon_pos].trim();
                if key.eq_ignore_ascii_case(header_name) {
                    let val = trimmed[colon_pos + 1..].trim();
                    if !val.is_empty() {
                        return Some(val.to_string());
                    }
                }
            }
        }
        None
    }

    /// Compte les fichiers de manière récursive.
    fn count_files_recursive(dir: &Path) -> usize {
        let mut count = 0;
        let Ok(entries) = fs::read_dir(dir) else {
            return 0;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                count += Self::count_files_recursive(&path);
            } else if path.is_file() {
                count += 1;
            }
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_php_variable() {
        let php = "<?php\n$wp_version = '7.1.3';\n$table_prefix = 'wp_';\n";
        assert_eq!(
            WordPressScanner::extract_php_variable(php, "$wp_version"),
            Some("7.1.3".to_string())
        );
        assert_eq!(
            WordPressScanner::extract_php_variable(php, "$table_prefix"),
            Some("wp_".to_string())
        );
    }

    #[test]
    fn test_extract_header() {
        let header = "/*\nPlugin Name: WPForms Lite\nVersion: 2.0.3\nDescription: Form builder\n*/";
        assert_eq!(
            WordPressScanner::extract_header(header, "Plugin Name"),
            Some("WPForms Lite".to_string())
        );
        assert_eq!(
            WordPressScanner::extract_header(header, "Version"),
            Some("2.0.3".to_string())
        );
        assert_eq!(
            WordPressScanner::extract_header(header, "Description"),
            Some("Form builder".to_string())
        );
    }

    #[tokio::test]
    async fn test_scan_wordpress_directory_if_present() {
        let temp_dir = std::env::temp_dir().join(format!("rf_test_scan_{}", uuid::Uuid::new_v4()));
        let wp_inc = temp_dir.join("wp-includes");
        let _ = tokio::fs::create_dir_all(&wp_inc).await;
        let _ = tokio::fs::write(
            wp_inc.join("version.php"),
            "<?php\n$wp_version = '7.1.3';\n$wp_db_version = 53478;\n",
        )
        .await;

        let res = WordPressScanner::scan(temp_dir.to_str().unwrap())
            .await
            .unwrap();
        assert_eq!(res.wp_version, Some("7.1.3".to_string()));
        assert_eq!(res.target_type, "wordpress_root");

        let _ = tokio::fs::remove_dir_all(&temp_dir).await;
    }
}
