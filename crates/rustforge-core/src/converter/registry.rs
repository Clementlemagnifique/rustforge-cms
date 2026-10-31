//! Registre universel des capacités (Capability Registry).
//! Mappe les capacités et hooks WordPress vers les modules natifs RustForge.

use super::manifest::{CompatibilityReport, CompatibilityStatus, ConversionStrategyKind};

/// Descripteur d'une capacité fonctionnelle dans le registre.
#[derive(Debug, Clone)]
pub struct CapabilityDescriptor {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub target_module: Option<&'static str>,
    pub matching_plugins: &'static [&'static str],
    pub matching_hooks: &'static [&'static str],
    pub matching_shortcodes: &'static [&'static str],
}

/// Registre statique des capacités supportées nativement.
pub struct CapabilityRegistry;

impl CapabilityRegistry {
    /// Liste exhaustive des descripteurs de capacités gérées.
    pub fn all_capabilities() -> &'static [CapabilityDescriptor] {
        &[
            // 1. FORMULAIRES
            CapabilityDescriptor {
                id: "forms.builder_and_submission",
                name: "Constructeur et Soumission de Formulaires",
                description: "Prise en charge native des formulaires sécurisés, validation de schéma JSON, honeypot anti-spam sans captcha et hachage IP RGPD.",
                target_module: Some("forms"),
                matching_plugins: &[
                    "wpforms",
                    "wpforms-lite",
                    "contact-form-7",
                    "gravityforms",
                    "formidable",
                    "ninja-forms",
                    "fluentform",
                ],
                matching_hooks: &[
                    "wpforms_process",
                    "wpcf7_before_send_mail",
                    "gform_after_submission",
                    "wp_ajax_wpforms_submit",
                    "wp_ajax_nopriv_wpforms_submit",
                ],
                matching_shortcodes: &["wpforms", "contact-form-7", "gravityform", "ninja_form"],
            },
            // 2. E-COMMERCE NUMÉRIQUE
            CapabilityDescriptor {
                id: "commerce.digital_downloads",
                name: "E-Commerce de Produits Dématérialisés",
                description: "Vente de fichiers protégés, jetons cryptographiquement signés HMAC-SHA256 hors web root, codes promotionnels et quotas.",
                target_module: Some("ecommerce"),
                matching_plugins: &[
                    "easy-digital-downloads",
                    "woocommerce",
                    "download-monitor",
                    "wp-e-commerce",
                ],
                matching_hooks: &[
                    "edd_process_purchase",
                    "woocommerce_checkout_order_processed",
                    "edd_download_files",
                    "woocommerce_payment_complete",
                ],
                matching_shortcodes: &[
                    "downloads",
                    "purchase_link",
                    "woocommerce_checkout",
                    "woocommerce_cart",
                ],
            },
            // 3. LMS & FORMATIONS
            CapabilityDescriptor {
                id: "lms.courses_and_lessons",
                name: "LMS Formations & Inscription Automatique",
                description: "Gestion des cursus, chapitres, leçons vidéo/texte et déblocage atomique de l'inscription élève dès la commande e-commerce.",
                target_module: Some("lms"),
                matching_plugins: &[
                    "sensei-lms",
                    "learndash",
                    "lifterlms",
                    "tutor-lms",
                    "masterstudy-lms",
                ],
                matching_hooks: &[
                    "sensei_user_course_end",
                    "learndash_course_completed",
                    "sensei_course_enrolment",
                    "lifterlms_course_completed",
                ],
                matching_shortcodes: &[
                    "sensei_courses",
                    "sensei_user_courses",
                    "learndash_course_progress",
                ],
            },
            // 4. MÉDIATHÈQUE & DOSSIERS VIRTUELS
            CapabilityDescriptor {
                id: "media.folders_and_webp",
                name: "Médiathèque Virtuelle & Compression WebP",
                description: "Organisation en dossiers virtuels hiérarchiques, compression WebP 82 % native et désinfection XML/SVG stricte.",
                target_module: Some("media"),
                matching_plugins: &[
                    "filebird",
                    "filebird-pro",
                    "real-media-library",
                    "wp-media-folder",
                    "smush",
                    "imagify",
                    "shortpixel",
                    "safe-svg",
                ],
                matching_hooks: &[
                    "wp_handle_upload",
                    "image_make_intermediate_size",
                    "wp_generate_attachment_metadata",
                ],
                matching_shortcodes: &[],
            },
            // 5. GALERIES & MÉDIAS ENRICHIS
            CapabilityDescriptor {
                id: "galleries.albums_and_video",
                name: "Galeries Multi-sources & Intégrations Vidéo",
                description: "Albums photos locaux, vidéos MP4 hébergées et intégrations fluides YouTube / Vimeo.",
                target_module: Some("galleries"),
                matching_plugins: &[
                    "thedock-media",
                    "modula",
                    "envira-gallery",
                    "nextgen-gallery",
                    "foogallery",
                ],
                matching_hooks: &["envira_gallery_output", "modula_shortcode"],
                matching_shortcodes: &["gallery", "modula", "envira-gallery", "video"],
            },
            // 6. THÈMES FSE & DESIGN TOKENS
            CapabilityDescriptor {
                id: "theme.fse_design_tokens",
                name: "Design Tokens & Templates Déclaratifs FSE",
                description: "Transposition directe des tokens de couleur, polices et marges de theme.json vers le système de thèmes CSS et templates MiniJinja 3.",
                target_module: None, // Géré par le moteur SSR de thèmes
                matching_plugins: &[],
                matching_hooks: &["after_setup_theme", "wp_enqueue_scripts"],
                matching_shortcodes: &[],
            },
            // 7. SÉCURITÉ & FIREWALL
            CapabilityDescriptor {
                id: "security.firewall_and_audit",
                name: "Sécurité Native & Limitation de Débit",
                description: "Protection anti-brute force en mémoire, limitation de débit, en-têtes HTTP de sécurité stricts et traçabilité d'audit.",
                target_module: None, // Cœur Axum
                matching_plugins: &[
                    "all-in-one-wp-security-and-firewall",
                    "wordfence",
                    "better-wp-security",
                    "ithemecurity",
                ],
                matching_hooks: &["login_failed", "wp_login", "init"],
                matching_shortcodes: &[],
            },
        ]
    }

    /// Résout la stratégie et produit le rapport de compatibilité pour un package analysé.
    pub fn resolve(
        slug: &str,
        detected_hooks: &[String],
        detected_shortcodes: &[String],
        has_theme_json: bool,
        has_block_templates: bool,
    ) -> (
        ConversionStrategyKind,
        Option<String>,
        Vec<String>,
        CompatibilityReport,
    ) {
        let slug_lower = slug.to_lowercase();
        let mut matched_capabilities = Vec::new();
        let mut target_module = None;

        // Cas A : Thème FSE
        if has_theme_json || has_block_templates {
            matched_capabilities.push("theme.fse_design_tokens".to_string());
            let report = CompatibilityReport {
                score: 95,
                status: CompatibilityStatus::PartiallySupported,
                migrated_features: vec![
                    "Palette de couleurs déclarative transmutée en variables CSS RustForge".to_string(),
                    "Hiérarchie typographique (titres, corps) convertie en tokens de thème".to_string(),
                    "Templates de pages FSE convertis en templates MiniJinja 3 avec rechargement à chaud".to_string(),
                ],
                replaced_features: vec![
                    "Remplacement du moteur de rendu Gutenberg PHP par le moteur SSR MiniJinja 3 (< 1 ms)".to_string(),
                    "Élimination des scripts d'assets WordPress lourds au profit de CSS pur ultra-léger".to_string(),
                ],
                partially_supported: vec![
                    "Styles personnalisés complexes injectés par PHP non standard".to_string(),
                ],
                incompatible_features: vec![
                    "Fichiers PHP de personnalisation dynamique requérant des hooks personnalisés tiers".to_string(),
                ],
            };
            return (
                ConversionStrategyKind::Declarative,
                None,
                matched_capabilities,
                report,
            );
        }

        // Cas B & autres : Détection par matching de plugins / hooks / shortcodes
        for cap in Self::all_capabilities() {
            let matches_slug = cap.matching_plugins.iter().any(|&p| slug_lower.contains(p));
            let matches_hook = cap.matching_hooks.iter().any(|&h| {
                detected_hooks
                    .iter()
                    .any(|dh| dh.to_lowercase().contains(h))
            });
            let matches_sc = cap.matching_shortcodes.iter().any(|&sc| {
                detected_shortcodes
                    .iter()
                    .any(|dsc| dsc.to_lowercase() == sc)
            });

            if matches_slug || matches_hook || matches_sc {
                matched_capabilities.push(cap.id.to_string());
                if target_module.is_none() && cap.target_module.is_some() {
                    target_module = cap.target_module.map(|s| s.to_string());
                }
            }
        }

        // Évaluation du résultat
        if let Some(ref mod_name) = target_module {
            match mod_name.as_str() {
                "forms" => {
                    let report = CompatibilityReport {
                        score: 95,
                        status: CompatibilityStatus::PartiallySupported,
                        migrated_features: vec![
                            "Soumissions de formulaires sécurisées avec validation JSON Schema".to_string(),
                            "Protection anti-spam par pot de miel (Honeypot) sans captcha".to_string(),
                            "Hachage IP anonymisé RGPD (SHA-256 avec sel journalier)".to_string(),
                            "Stockage structuré des entrées et export".to_string(),
                        ],
                        replaced_features: vec![
                            "Les shortcodes de formulaire ([wpforms id=...]) sont remplacés par le bloc universel FormEmbed".to_string(),
                            "Les scripts jQuery frontend sont éliminés au profit d'appels API fetch légers".to_string(),
                        ],
                        partially_supported: vec![],
                        incompatible_features: vec![
                            "Extensions tierces propriétaires intégrant des passerelles de paiement fermées".to_string(),
                        ],
                    };
                    (
                        ConversionStrategyKind::NativeReuse,
                        target_module,
                        matched_capabilities,
                        report,
                    )
                }
                "ecommerce" => {
                    let report = CompatibilityReport {
                        score: 90,
                        status: CompatibilityStatus::PartiallySupported,
                        migrated_features: vec![
                            "Catalogue de produits dématérialisés et gestion des prix".to_string(),
                            "Liens de téléchargement cryptographiquement signés HMAC-SHA256"
                                .to_string(),
                            "Codes de réduction promotionnels et quotas par commande".to_string(),
                            "Tunnel de commande sécurisé et génération des tokens".to_string(),
                        ],
                        replaced_features: vec![
                            "Remplacement de l'écosystème PHP EDD par le module natif SQLx/Rust"
                                .to_string(),
                            "Protection des fichiers en dehors de la racine web publique"
                                .to_string(),
                        ],
                        partially_supported: vec![
                            "Règles complexes de taxes par sous-régions géographiques spécifiques"
                                .to_string(),
                        ],
                        incompatible_features: vec![
                            "Passerelles de paiement obscures sans API REST documentée".to_string(),
                        ],
                    };
                    (
                        ConversionStrategyKind::NativeReuse,
                        target_module,
                        matched_capabilities,
                        report,
                    )
                }
                "lms" => {
                    let report = CompatibilityReport {
                        score: 90,
                        status: CompatibilityStatus::PartiallySupported,
                        migrated_features: vec![
                            "Hiérarchie complète : Cursus ➔ Modules ➔ Leçons vidéo/texte".to_string(),
                            "Inscription automatique atomique de l'acheteur dès la commande".to_string(),
                            "Suivi de progression des apprenants".to_string(),
                        ],
                        replaced_features: vec![
                            "Remplacement des custom post types WordPress par les tables lms_* optimisées".to_string(),
                        ],
                        partially_supported: vec![
                            "Banques de questions à choix multiples complexes".to_string(),
                        ],
                        incompatible_features: vec![
                            "Génération de certificats PDF basée sur des bibliothèques C obsolètes".to_string(),
                        ],
                    };
                    (
                        ConversionStrategyKind::NativeReuse,
                        target_module,
                        matched_capabilities,
                        report,
                    )
                }
                "media" => {
                    let report = CompatibilityReport {
                        score: 98,
                        status: CompatibilityStatus::PartiallySupported,
                        migrated_features: vec![
                            "Arborescence de dossiers virtuels hiérarchiques sans altération d'URL"
                                .to_string(),
                            "Conversion automatique en WebP (qualité 82 % native Rust)".to_string(),
                            "Désinfection XML/SVG stricte (quick-xml) contre les attaques XSS"
                                .to_string(),
                        ],
                        replaced_features: vec![
                            "Élimination des plugins d'optimisation d'images payants à abonnements"
                                .to_string(),
                        ],
                        partially_supported: vec![],
                        incompatible_features: vec![],
                    };
                    (
                        ConversionStrategyKind::NativeReuse,
                        target_module,
                        matched_capabilities,
                        report,
                    )
                }
                "galleries" => {
                    let report = CompatibilityReport {
                        score: 95,
                        status: CompatibilityStatus::PartiallySupported,
                        migrated_features: vec![
                            "Albums photos locaux et galeries visuelles responsives".to_string(),
                            "Intégrations fluides de vidéos YouTube / Vimeo avec lazy loading"
                                .to_string(),
                        ],
                        replaced_features: vec![
                            "Remplacement des scripts lourds par un visualiseur HTML5/CSS natif"
                                .to_string(),
                        ],
                        partially_supported: vec![],
                        incompatible_features: vec![],
                    };
                    (
                        ConversionStrategyKind::NativeReuse,
                        target_module,
                        matched_capabilities,
                        report,
                    )
                }
                _ => (
                    ConversionStrategyKind::NativeReuse,
                    target_module,
                    matched_capabilities,
                    CompatibilityReport {
                        score: 85,
                        status: CompatibilityStatus::PartiallySupported,
                        migrated_features: vec![
                            "Capacités identifiées prises en charge par RustForge".to_string(),
                        ],
                        replaced_features: vec![],
                        partially_supported: vec![],
                        incompatible_features: vec![],
                    },
                ),
            }
        } else {
            // Aucun module natif évident : Stratégie de conversion assistée ou adaptateur WASM
            let report = CompatibilityReport {
                score: 50,
                status: CompatibilityStatus::RequiresAssistance,
                migrated_features: vec![
                    "Extraction des métadonnées et analyse des structures de données".to_string(),
                    "Préservation des données brutes dans le coffre-fort _legacy_vault".to_string(),
                ],
                replaced_features: vec![],
                partially_supported: vec![
                    "Logique métier nécessitant un adaptateur WebAssembly (Wasmtime WASIp2)".to_string(),
                ],
                incompatible_features: vec![
                    "Exécution de code PHP arbitraire non permise sur RustForge pour des raisons de sécurité".to_string(),
                ],
            };
            (
                ConversionStrategyKind::Assisted,
                None,
                matched_capabilities,
                report,
            )
        }
    }
}
