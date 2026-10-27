//! Moteur de conversion universel RustForge (Universal Converter).
//! Ingestion sécurisée de ZIP WordPress (thèmes et plugins), détection des capacités,
//! réutilisation native et transmutation déclarative sans exécution PHP.

pub mod analyzer;
pub mod forms_converter;
pub mod intelligence;
pub mod manifest;
pub mod providers;
pub mod registry;
pub mod scanner;
pub mod theme_converter;

pub use analyzer::PackageAnalyzer;
pub use forms_converter::{FormsConversionResult, FormsConverter};
pub use intelligence::{
    CapabilityOperationalStatus, CapabilityComposer, CapabilityComposition, ConfidenceLevel,
    FeatureGapDecision, FeatureNode, FunctionalFeatureGraph, GapAnalysisEngine, GapResolutionKind,
    MicroPluginBuilder, MicroPluginPackage, ValidationEngine, ValidationResult,
};
pub use manifest::*;
pub use providers::{
    AnalysisEvidence, AnalysisProvider, AnalysisTarget, JavaScriptStaticProvider,
    ReaAnalysisProvider, WordPressMetadataProvider,
};
pub use registry::CapabilityRegistry;
pub use scanner::{PluginScanItem, ThemeScanItem, WordPressScanner, WpScanResult};
pub use theme_converter::{ConvertedTheme, FseThemeConverter};

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    /// Crée un ZIP en mémoire pour simuler un thème FSE WordPress (Cas A : Twenty Twenty-Five)
    fn build_mock_fse_theme_zip() -> Vec<u8> {
        let mut buffer = Vec::new();
        {
            let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buffer));
            let options = SimpleFileOptions::default();

            // style.css
            zip.start_file("twentytwentyfive/style.css", options).unwrap();
            zip.write_all("/*\nTheme Name: Twenty Twenty-Five\nVersion: 1.0.0\nAuthor: WordPress.org\nDescription: Theme FSE par defaut 2026.\n*/".as_bytes()).unwrap();

            // theme.json
            zip.start_file("twentytwentyfive/theme.json", options).unwrap();
            let theme_json = r##"{
                "version": 3,
                "settings": {
                    "color": {
                        "palette": [
                            {"slug": "primary", "color": "#113355", "name": "Primary"},
                            {"slug": "secondary", "color": "#ffaa00", "name": "Accent"},
                            {"slug": "background", "color": "#f8fafc", "name": "Background"}
                        ]
                    },
                    "typography": {
                        "fontFamilies": [
                            {"slug": "heading", "fontFamily": "Inter, sans-serif", "name": "Heading Font"},
                            {"slug": "body", "fontFamily": "Roboto, sans-serif", "name": "Body Font"}
                        ]
                    }
                }
            }"##;
            zip.write_all(theme_json.as_bytes()).unwrap();

            // templates/index.html
            zip.start_file("twentytwentyfive/templates/index.html", options).unwrap();
            let template_html = r#"<!-- wp:group {"layout":{"type":"constrained"}} -->
<div class="wp-block-group">
  <!-- wp:site-title /-->
  <!-- wp:post-title {"level":1} /-->
  <!-- wp:post-date /-->
  <!-- wp:post-content /-->
</div>
<!-- /wp:group -->"#;
            zip.write_all(template_html.as_bytes()).unwrap();

            zip.finish().unwrap();
        }
        buffer
    }

    /// Crée un ZIP en mémoire pour simuler un plugin WordPress de formulaires (Cas B : WPForms Lite)
    fn build_mock_forms_plugin_zip() -> Vec<u8> {
        let mut buffer = Vec::new();
        {
            let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buffer));
            let options = SimpleFileOptions::default();

            // wpforms.php
            zip.start_file("wpforms-lite/wpforms.php", options).unwrap();
            let php_content = r#"<?php
/**
 * Plugin Name: WPForms Lite
 * Version: 1.9.1
 * Author: WPForms
 * Description: Drag & Drop WordPress Form Builder.
 */

add_action( 'init', 'wpforms_init' );
add_action( 'wp_ajax_wpforms_submit', 'wpforms_ajax_submit' );
add_shortcode( 'wpforms', 'wpforms_shortcode_output' );
"#;
            zip.write_all(php_content.as_bytes()).unwrap();

            zip.finish().unwrap();
        }
        buffer
    }

    #[test]
    fn test_case_a_fse_theme_analysis_and_transmutation() {
        let zip_bytes = build_mock_fse_theme_zip();

        // 1. Analyse du package
        let manifest = PackageAnalyzer::analyze_zip(&zip_bytes).expect("L'analyse du thème FSE doit réussir");
        assert_eq!(manifest.package.kind, PackageKind::ThemeFse);
        assert_eq!(manifest.package.name, "Twenty Twenty-Five");
        assert_eq!(manifest.package.version, "1.0.0");
        assert!(manifest.package.has_theme_json);
        assert!(manifest.package.has_block_templates);
        assert_eq!(manifest.strategy, ConversionStrategyKind::Declarative);
        assert_eq!(manifest.compatibility.status, CompatibilityStatus::FullySupported);
        assert!(manifest.activation_plan.preview_available);

        // 2. Conversion déclarative FSE vers MiniJinja 3
        let converted = FseThemeConverter::convert_zip(&zip_bytes, None)
            .expect("La conversion du thème doit réussir");
        assert_eq!(converted.name, "Twenty Twenty-Five");
        assert!(converted.theme_json_content.contains("#113355")); // couleur primaire
        assert!(converted.theme_json_content.contains("#ffaa00")); // couleur accent
        assert!(converted.templates.contains_key("base.html"));
        assert!(converted.templates.contains_key("home.html"));
        assert!(converted.templates.contains_key("single.html"));

        let home_tpl = converted.templates.get("home.html").unwrap();
        assert!(home_tpl.contains("{{ entry.title }}"));
        assert!(home_tpl.contains("{{ entry.rendered_content | safe }}"));
        assert!(home_tpl.contains("{{ site.name }}"));
        // Vérifie qu'aucun commentaire Gutenberg n'a pollué le rendu
        assert!(!home_tpl.contains("<!-- wp:group"));
    }

    #[test]
    fn test_case_b_forms_plugin_analysis_and_native_reuse() {
        let zip_bytes = build_mock_forms_plugin_zip();

        // 1. Analyse du package
        let manifest = PackageAnalyzer::analyze_zip(&zip_bytes).expect("L'analyse du plugin de formulaire doit réussir");
        assert_eq!(manifest.package.kind, PackageKind::Plugin);
        assert_eq!(manifest.package.name, "WPForms Lite");
        assert_eq!(manifest.package.version, "1.9.1");
        assert_eq!(manifest.strategy, ConversionStrategyKind::NativeReuse);
        assert_eq!(manifest.target_native_module, Some("forms".to_string()));
        assert_eq!(manifest.compatibility.status, CompatibilityStatus::FullySupported);
        assert!(manifest.activation_plan.enable_modules.contains(&"forms".to_string()));
        assert!(manifest.activation_plan.register_blocks.contains(&"form_embed".to_string()));

        // Vérifie que les hooks et shortcodes ont été extraits
        let hook_names: Vec<String> = manifest.detected_hooks.into_iter().map(|h| h.name).collect();
        assert!(hook_names.contains(&"init".to_string()));
        assert!(hook_names.contains(&"wp_ajax_wpforms_submit".to_string()));
        assert!(hook_names.contains(&"wpforms".to_string()));

        // 2. Remplacement déclaratif par RustForge Forms
        let forms_plan = FormsConverter::plan_conversion(&manifest.package.slug, &manifest.package.author);
        assert_eq!(forms_plan.strategy, ConversionStrategyKind::NativeReuse);
        assert!(forms_plan.code_disposition.contains("entièrement remplacé par le module natif RustForge Forms"));
        assert!(forms_plan.data_migration_guidance.contains("DataMorph"));
        assert_eq!(forms_plan.proposed_form.honeypot_enabled, true);
    }

    #[test]
    fn test_security_rejection_directory_traversal() {
        let mut buffer = Vec::new();
        {
            let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buffer));
            let options = SimpleFileOptions::default();
            // Injection d'un chemin malveillant
            zip.start_file("../evil.php", options).unwrap();
            zip.write_all(b"<?php echo 'hacked'; ?>").unwrap();
            zip.finish().unwrap();
        }

        let result = PackageAnalyzer::analyze_zip(&buffer);
        assert!(result.is_err(), "L'archive avec Directory Traversal doit être rejetée");
    }

    #[tokio::test]
    async fn test_multi_provider_analysis_including_rea() {
        let php_target = AnalysisTarget {
            file_path: "plugin.php".to_string(),
            mime_type: "text/x-php".to_string(),
            content: "add_action('init', 'setup');\nadd_shortcode('custom_view', 'render_view');".to_string(),
            is_minified_or_compiled: false,
        };

        let wp_provider = WordPressMetadataProvider;
        assert!(wp_provider.supports(&php_target));
        let wp_evidences = wp_provider.analyze(&php_target).await.unwrap();
        assert_eq!(wp_evidences.len(), 2);
        assert_eq!(wp_evidences[0].provider_name, "wordpress_metadata");

        let js_target = AnalysisTarget {
            file_path: "assets/widget.bundle.min.js".to_string(),
            mime_type: "application/javascript".to_string(),
            content: "const api = fetch('/wp-json/v1/data');".to_string(),
            is_minified_or_compiled: true,
        };

        let js_provider = JavaScriptStaticProvider;
        assert!(js_provider.supports(&js_target));
        let js_evidences = js_provider.analyze(&js_target).await.unwrap();
        assert_eq!(js_evidences.len(), 1);
        assert!(js_evidences[0].snippet_or_signature.contains("fetch"));

        let rea_provider = ReaAnalysisProvider::default();
        assert!(rea_provider.supports(&js_target));
        let rea_evidences = rea_provider.analyze(&js_target).await.unwrap();
        assert_eq!(rea_evidences.len(), 1);
        assert_eq!(rea_evidences[0].provider_name, "rea");
    }

    #[test]
    fn test_gap_analysis_and_capability_composer() {
        let mut graph = FunctionalFeatureGraph::default();
        graph.package_slug = "test-course-plugin".to_string();
        graph.nodes.push(FeatureNode {
            id: "contact_form".to_string(),
            name: "Formulaire".to_string(),
            description: "Formulaire de contact".to_string(),
            source_reference: "form.php".to_string(),
            confidence: ConfidenceLevel::Certain,
            dependencies: vec![],
            business_effects: vec![],
        });
        graph.nodes.push(FeatureNode {
            id: "link_order_to_course".to_string(),
            name: "Liaison Achat ➔ Formation".to_string(),
            description: "Inscription post-achat automatique".to_string(),
            source_reference: "bridge.php".to_string(),
            confidence: ConfidenceLevel::Certain,
            dependencies: vec![],
            business_effects: vec![],
        });
        graph.nodes.push(FeatureNode {
            id: "reading_time".to_string(),
            name: "Calcul temps de lecture".to_string(),
            description: "Temps de lecture estimé".to_string(),
            source_reference: "reading.php".to_string(),
            confidence: ConfidenceLevel::High,
            dependencies: vec![],
            business_effects: vec![],
        });

        let decisions = GapAnalysisEngine::evaluate(&graph);
        assert_eq!(decisions.len(), 3);
        assert_eq!(decisions[0].resolution, GapResolutionKind::Native);
        assert_eq!(decisions[1].resolution, GapResolutionKind::Composable);
        assert_eq!(decisions[2].resolution, GapResolutionKind::Generatable);

        // Test composition déclarative
        let composition = CapabilityComposer::compose_ecommerce_lms_link();
        assert_eq!(composition.trigger_event, "commerce.order.completed");
        assert_eq!(composition.pipeline[0].capability, "lms.enroll_user");
    }

    #[test]
    fn test_micro_plugin_builder_and_validation_engine() {
        let package = MicroPluginBuilder::build_skeleton("reading-time-pro", "Reading Time Pro", "content.filter.render");
        assert_eq!(package.extension_slug, "reading-time-pro");
        assert_eq!(package.files.len(), 4);

        let validation = ValidationEngine::validate(&package);
        assert!(validation.is_valid);
        assert_eq!(validation.security_score, 100);
        assert!(validation.passed_checks.iter().any(|c| c.contains("SDK WASM complète")));
        assert!(validation.passed_checks.iter().any(|c| c.contains("100% sûr")));
    }
}
