use std::{net::SocketAddr, path::PathBuf};

use clap::{Parser, Subcommand};
use rustforge_api::{AppState, build_router};
use rustforge_core::{models::Role, users::NewUser};
use rustforge_plugins::PluginManager;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
mod initialize;

#[derive(Parser)]
#[command(name = "rustforge")]
#[command(about = "RustForge CMS — CMS souverain IA-First de nouvelle génération en Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Prépare la configuration et les tables, sans écraser une installation existante.
    Init {
        #[arg(long, env = "DATABASE_URL")]
        database_url: Option<String>,
    },
    /// Lance le serveur HTTP RustForge CMS (API + Studio + Uploads).
    Serve {
        #[arg(short, long, env = "PORT", value_name = "PORT")]
        port: Option<u16>,
        #[arg(long, env = "HOST", value_name = "HOST")]
        host: Option<String>,
    },
    /// Applique les migrations PostgreSQL.
    Migrate,
    /// Génère un secret aléatoire local, sans connexion à la base.
    GenerateSecret,
    /// Crée un compte super-administrateur.
    CreateAdmin {
        #[arg(short, long)]
        email: String,
        #[arg(short, long)]
        username: String,
        #[arg(short, long)]
        password: String,
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Affiche la liste des outils déclaratifs (Tool Calling / MCP) pour les agents IA.
    AiTools,
    /// Importe un fichier XML ou export historique via le moteur DataMorph vers le standard RF.
    #[command(alias = "import-content", alias = "import-rf")]
    ImportWxr {
        /// Chemin vers le fichier XML d'exportation.
        #[arg(short, long)]
        file: PathBuf,
    },
    /// Scanne une cible d'anciens CMS (Score de risque PHP2Shell, compatibilité native RustForge).
    #[command(alias = "scan-legacy", alias = "scan")]
    ScanWp {
        /// URL ou répertoire du site à auditer.
        target: String,
    },
    /// Exporte le contenu complet de RustForge au standard RF souverain (RF-XML ou RFIR-JSON).
    #[command(alias = "export-rf", alias = "export")]
    ExportContent {
        /// Format d'exportation : "xml" (par défaut) ou "json" (RFIR).
        #[arg(short, long, default_value = "xml")]
        format: String,
        /// Fichier de sortie (si omis, écrit sur la sortie standard stdout).
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Encapsule l'ensemble du site (Base de données + Médias + Configuration) dans une archive .rfcapsule unique.
    #[command(alias = "export-capsule", alias = "capsule-pack")]
    PackCapsule {
        /// Chemin du fichier capsule à créer (ex: site-backup.rfcapsule).
        #[arg(short, long)]
        output: PathBuf,
        /// URL d'origine du site pour faciliter le remplacement lors de la migration.
        #[arg(long)]
        site_url: Option<String>,
    },
    /// Décapsule et restaure un site complet depuis une archive .rfcapsule.
    #[command(alias = "import-capsule", alias = "capsule-unpack")]
    UnpackCapsule {
        /// Chemin du fichier .rfcapsule à restaurer.
        #[arg(short, long)]
        file: PathBuf,
        /// Nouvelle URL cible pour adapter automatiquement les liens du site.
        #[arg(long)]
        new_site_url: Option<String>,
    },
}

fn load_config_file() -> anyhow::Result<toml::Value> {
    match std::fs::read_to_string("config.toml") {
        Ok(content) => Ok(toml::from_str(&content)?),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            Ok(toml::Value::Table(Default::default()))
        }
        Err(err) => Err(err.into()),
    }
}

fn configured(config: &toml::Value, key: &str) -> Option<String> {
    if let Ok(value) = std::env::var(key) {
        return Some(value);
    }
    let path = match key {
        "DATABASE_URL" => ["database", "url"],
        "PORT" => ["server", "port"],
        "HOST" => ["server", "host"],
        "JWT_SECRET" => ["server", "jwt_secret"],
        "UPLOADS_DIR" => ["storage", "uploads_dir"],
        "PLUGINS_DIR" => ["storage", "plugins_dir"],
        "THEMES_DIR" => ["storage", "themes_dir"],
        "ADMIN_DIST" => ["storage", "admin_dist"],
        "ACTIVE_THEME" => ["theme", "active"],
        "PROTECTED_DOWNLOADS_DIR" => ["storage", "protected_downloads_dir"],
        "RUSTFORGE_SETUP_TOKEN" => ["server", "setup_token"],
        _ => return None,
    };
    config.get(path[0]).and_then(|v| v.get(path[1])).map(|v| {
        v.as_str()
            .map(str::to_string)
            .unwrap_or_else(|| v.to_string())
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // An extracted installation must never inherit secrets from a parent directory.
    dotenvy::from_path(".env").ok();
    let config = load_config_file()?;
    let setting = |key: &str| configured(&config, key);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rustforge=debug,rustforge_api=debug,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();
    let database_url = setting("DATABASE_URL")
        .unwrap_or_else(|| "postgres://postgres:postgres@localhost:5432/trustpress".to_string());

    match cli.command {
        Commands::Init { database_url } => {
            let url=database_url.ok_or_else(||anyhow::anyhow!("Fournissez DATABASE_URL ou utilisez le lanceur start pour renseigner votre base PostgreSQL."))?;
            initialize::initialize(&std::env::current_dir()?, &url).await?;
        }
        Commands::GenerateSecret => {
            println!(
                "{}{}",
                uuid::Uuid::new_v4().simple(),
                uuid::Uuid::new_v4().simple()
            );
        }
        Commands::Migrate => {
            println!("🔄 Connexion à PostgreSQL");
            let pool = rustforge_core::connect(&database_url).await?;
            println!("🚀 Application des migrations...");
            rustforge_core::migrate(&pool).await?;
            println!("✅ Base de données à jour !");
        }
        Commands::CreateAdmin {
            email,
            username,
            password,
            name,
        } => {
            let pool = rustforge_core::connect(&database_url).await?;
            rustforge_core::migrate(&pool).await?;
            println!("👤 Création de l'administrateur « {username} » ({email})...");
            let user = rustforge_core::users::create(
                &pool,
                NewUser {
                    email,
                    username,
                    display_name: name,
                    password,
                    role: Role::Admin,
                },
            )
            .await?;
            println!("✅ Administrateur créé avec l'identifiant : {}", user.id);
        }
        Commands::AiTools => {
            let tools = rustforge_core::ai::get_available_tools();
            println!("{}", serde_json::to_string_pretty(&tools)?);
        }
        Commands::Serve { port, host } => {
            println!("🚀 Initialisation de RustForge CMS...");
            let pool = match rustforge_core::connect(&database_url).await {
                Ok(p) => {
                    if let Err(e) = rustforge_core::migrate(&p).await {
                        return Err(e.into());
                    }
                    p
                }
                Err(e) => {
                    tracing::error!("Impossible de joindre PostgreSQL");
                    tracing::warn!("Serveur démarré en attente de la base de données.");
                    return Err(e.into());
                }
            };

            let plugins_dir = setting("PLUGINS_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("plugins"));

            let uploads_dir = setting("UPLOADS_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("uploads"));
            tokio::fs::create_dir_all(&uploads_dir).await?;

            let admin_dist = setting("ADMIN_DIST")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("admin/dist"));

            let themes_dir = setting("THEMES_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("themes"));

            let active_theme = setting("ACTIVE_THEME").unwrap_or_else(|| "default".to_string());

            let jwt_secret = setting("JWT_SECRET").unwrap_or_else(String::new);

            let plugin_manager = PluginManager::new(pool.clone(), plugins_dir).await?;
            let mut app_state = AppState::new(
                pool.clone(),
                plugin_manager,
                jwt_secret,
                uploads_dir,
                Some(admin_dist),
                themes_dir,
                active_theme,
            );

            app_state.protected_downloads_dir = setting("PROTECTED_DOWNLOADS_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("protected_downloads"));
            tokio::fs::create_dir_all(&app_state.protected_downloads_dir).await?;
            rustforge_core::security::safe_slug(&app_state.active_theme)?;
            if app_state.jwt_secret.len() < 32
                || app_state.jwt_secret.contains("changez")
                || app_state.jwt_secret.contains("super-secret")
            {
                anyhow::bail!(
                    "Configurez JWT_SECRET avec un secret aléatoire unique d'au moins 32 caractères avant de démarrer."
                );
            }
            let has_admin: bool =
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE role='admin')")
                    .fetch_one(&pool)
                    .await?;
            if !has_admin {
                let setup = setting("RUSTFORGE_SETUP_TOKEN").unwrap_or_else(|| {
                    format!(
                        "{}{}",
                        uuid::Uuid::new_v4().simple(),
                        uuid::Uuid::new_v4().simple()
                    )
                });
                println!("Clé d'installation à saisir dans le Studio : {setup}");
                app_state.setup_token = Some(setup);
            }
            let setup_link_token = app_state.setup_token.clone();
            let router = build_router(app_state);
            let host = host
                .or_else(|| setting("HOST"))
                .unwrap_or_else(|| "127.0.0.1".into());
            let port = port
                .or_else(|| setting("PORT").and_then(|p| p.parse().ok()))
                .unwrap_or(8080);
            let addr: SocketAddr = format!("{host}:{port}").parse()?;

            // Démarrage du Planificateur Éditorial Souverain en tâche asynchrone Tokio
            rustforge_core::scheduler::start_scheduler(pool.clone(), 30);

            println!("✨ RustForge écoute sur http://{addr}");
            let browser_host = if addr.ip().is_unspecified() {
                "127.0.0.1".to_string()
            } else {
                addr.ip().to_string()
            };
            let admin_url = format!("http://{browser_host}:{port}/admin/");
            if let Some(token) = setup_link_token {
                println!("👉 Installer RustForge CMS : {admin_url}#setup={token}");
            } else {
                println!("👉 Administration : {admin_url}");
            }
            println!("👉 API REST : http://{addr}/api/v1");
            println!("👉 Outils IA / MCP : http://{addr}/api/v1/ai/tools");
            println!("🕒 Planificateur Éditorial : actif (vérification auto toutes les 30s)");

            let listener = tokio::net::TcpListener::bind(&addr).await?;
            axum::serve(
                listener,
                router.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .with_graceful_shutdown(async {
                let _ = tokio::signal::ctrl_c().await;
            })
            .await?;
        }
        Commands::ImportWxr { file } => {
            println!("🚀 Lancement du moteur DataMorph sur l'export : {:?}", file);
            if !file.exists() {
                anyhow::bail!("Fichier introuvable : {:?}", file);
            }
            let f = std::fs::File::open(&file)?;
            let reader = std::io::BufReader::new(f);
            let doc = if file.extension().is_some_and(|e| e == "json") {
                serde_json::from_reader(reader)?
            } else {
                rustforge_core::migration::WxrImporter::parse(reader)?
            };

            println!("📦 Document TMIR analysé avec succès :");
            println!(
                "   • Source : {}",
                doc.source_site_url.as_deref().unwrap_or("Non spécifiée")
            );
            println!("   • Entrées trouvées : {}", doc.entries.len());
            println!("   • Termes/Taxonomies : {}", doc.terms.len());
            println!("   • Médias référencés : {}", doc.media.len());

            println!("💾 Application en base PostgreSQL...");
            let pool = rustforge_core::connect(&database_url).await?;

            let author_id: Option<uuid::Uuid> =
                sqlx::query_scalar("SELECT id FROM users ORDER BY created_at ASC LIMIT 1")
                    .fetch_optional(&pool)
                    .await?;

            let admin_id = match author_id {
                Some(id) => id,
                None => anyhow::bail!(
                    "Créez explicitement un administrateur avant cet import : rustforge create-admin"
                ),
            };

            let report =
                rustforge_core::migration::DataMorphImporter::apply(&pool, &doc, admin_id).await?;
            println!("\n✨ Rapport d'importation DataMorph :");
            println!("   ✅ Entrées importées : {}", report.entries_imported);
            println!(
                "   ⏭️  Entrées ignorées / doublons : {}",
                report.entries_skipped
            );
            println!("   🏷️  Termes créés : {}", report.terms_imported);
            println!(
                "   🖼️  Médias ingérés dans la base : {} (sur {} détectés)",
                report.media_imported, report.media_detected
            );
            println!(
                "   🔒 Champs spécifiques préservés dans le coffre-fort _rf_vault : {}",
                report.legacy_fields_stored_in_vault
            );
            println!("   ⏱️  Temps d'exécution : {} ms", report.execution_time_ms);
        }
        Commands::ScanWp { target } => {
            println!("🛡️  Audit de Sécurité & Scanner Dynamique d'Anciens CMS");
            println!("🎯 Cible : {}", target);
            println!("--------------------------------------------------");

            let result = rustforge_core::converter::WordPressScanner::scan(&target).await?;

            println!("📂 Type de cible : {}", result.target_type);
            if let Some(ref ver) = result.wp_version {
                println!("📦 Version du runtime d'origine : {}", ver);
            }
            if let Some(ref pfx) = result.db_prefix {
                println!("🗄️  Préfixe base de données d'origine : « {} »", pfx);
            }
            if let Some(ref th) = result.active_theme {
                println!("🎨 Thème actif : {}", th);
            }
            if result.media_count > 0 {
                println!("🖼️  Fichiers médias détectés : {}", result.media_count);
            }

            println!(
                "\n🧩 EXTENSIONS DÉTECTÉES & ÉQUIVALENCE NATIVE ({} trouvées) :",
                result.plugins.len()
            );
            for p in &result.plugins {
                let badge = match p.strategy {
                    rustforge_core::converter::ConversionStrategyKind::NativeReuse => {
                        format!(
                            "✅ Remplacé nativement par RustForge [{}]",
                            p.native_module.as_deref().unwrap_or("natif")
                        )
                    }
                    rustforge_core::converter::ConversionStrategyKind::Declarative => {
                        "🔄 Transmutation déclarative (schéma sans code)".to_string()
                    }
                    rustforge_core::converter::ConversionStrategyKind::ControlledWasm => {
                        "⚡ Extension Builder IA (Wasmtime sandbox)".to_string()
                    }
                    rustforge_core::converter::ConversionStrategyKind::Assisted => {
                        "📋 Données conservées dans le coffre-fort _rf_vault".to_string()
                    }
                };
                println!("   • {} (v{}) ➔ {}", p.name, p.version, badge);
            }

            if !result.themes.is_empty() {
                println!("\n🎨 THÈMES DÉTECTÉS ({} trouvés) :", result.themes.len());
                for t in &result.themes {
                    let kind = if t.is_fse_block_theme {
                        "Thème basé sur des blocs déclaratifs ➔ Compatible Rendu MiniJinja 3 natif"
                    } else {
                        "Thème classique hérité ➔ Transmutation déclarative de tokens"
                    };
                    println!("   • {} (v{}) ➔ {}", t.name, t.version, kind);
                }
            }

            println!("\n📊 COUVERTURE DES CAPACITÉS NATIVES :");
            println!(
                "   • Correspondances théoriques à vérifier : {:.1} %",
                result.native_coverage_pct
            );
            println!("   • Performances : {}", result.estimated_speedup);

            println!("\n⚠️  DIAGNOSTIC CYBERSÉCURITÉ RUNTIME :");
            println!("   • État de sécurité : {}", result.php2shell_threat_level);
            println!(
                "   • Rust et l’absence de PHP ne constituent pas une preuve d’absence de vulnérabilité."
            );

            println!(
                "\n💡 Pour exécuter l'ingestion de données : lancez `rustforge import-content --file <export.xml>` ou glissez l'archive dans le Studio /admin."
            );
        }
        Commands::ExportContent { format, output } => {
            println!("🚀 Extraction souveraine des données depuis PostgreSQL...");
            let pool = rustforge_core::connect(&database_url).await?;
            let doc = rustforge_core::migration::DataMorphExporter::dump_database(&pool).await?;

            println!("📦 Données extraites avec succès :");
            println!("   • Entrées trouvées : {}", doc.entries.len());
            println!("   • Termes/Taxonomies : {}", doc.terms.len());
            println!("   • Médias référencés : {}", doc.media.len());

            let content = if format.to_lowercase() == "json" {
                rustforge_core::migration::DataMorphExporter::to_json(&doc)?
            } else {
                rustforge_core::migration::DataMorphExporter::to_rf_xml(&doc)?
            };

            match output {
                Some(path) => {
                    tokio::fs::write(&path, &content).await?;
                    println!("✅ Export souverain écrit avec succès dans : {:?}", path);
                    println!("   • Taille : {} octets", content.len());
                }
                None => {
                    println!(
                        "\n--- DÉBUT DE L'EXPORT SOUVERAIN (Format: {}) ---",
                        format.to_uppercase()
                    );
                    println!("{}", content);
                    println!("--- FIN DE L'EXPORT SOUVERAIN ---");
                }
            }
        }
        Commands::PackCapsule { output, site_url } => {
            println!("📦 Création de la Capsule RustForge complète...");
            let pool = rustforge_core::connect(&database_url).await?;
            let uploads_dir = setting("UPLOADS_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("uploads"));

            let storage = rustforge_core::migration::capsule::StoragePaths {
                uploads: uploads_dir.clone(),
                themes: setting("THEMES_DIR")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("themes")),
                plugins: setting("PLUGINS_DIR")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("plugins")),
                protected: setting("PROTECTED_DOWNLOADS_DIR")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("protected_downloads")),
            };
            let report =
                rustforge_core::migration::CapsuleManager::pack(&pool, &storage, &output, site_url)
                    .await?;
            println!("\n✨ Capsule créée avec succès !");
            println!("   📁 Fichier : {:?}", report.capsule_path);
            println!("   📦 Entrées sauvegardées : {}", report.entries_packed);
            println!(
                "   🖼️  Fichiers médias empaquetés : {}",
                report.media_files_packed
            );
            println!(
                "   ⚖️  Taille de l'archive : {:.2} Ko",
                (report.file_size_bytes as f64) / 1024.0
            );
            println!("   ⏱️  Temps d'exécution : {} ms", report.execution_time_ms);
        }
        Commands::UnpackCapsule { file, new_site_url } => {
            println!(
                "🚀 Décapsulation et restauration du site depuis : {:?}",
                file
            );
            if !file.exists() {
                anyhow::bail!("Fichier capsule introuvable : {:?}", file);
            }
            let pool = rustforge_core::connect(&database_url).await?;
            rustforge_core::migrate(&pool).await?;

            let uploads_dir = setting("UPLOADS_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("uploads"));

            let storage = rustforge_core::migration::capsule::StoragePaths {
                uploads: uploads_dir.clone(),
                themes: setting("THEMES_DIR")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("themes")),
                plugins: setting("PLUGINS_DIR")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("plugins")),
                protected: setting("PROTECTED_DOWNLOADS_DIR")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("protected_downloads")),
            };
            let author_id: Option<uuid::Uuid> =
                sqlx::query_scalar("SELECT id FROM users ORDER BY created_at ASC LIMIT 1")
                    .fetch_optional(&pool)
                    .await?;

            let admin_id = match author_id {
                Some(id) => id,
                None => uuid::Uuid::nil(),
            };

            let f = std::fs::File::open(&file)?;
            let report = rustforge_core::migration::CapsuleManager::unpack(
                &pool,
                &storage,
                f,
                new_site_url,
                admin_id,
            )
            .await?;

            println!("\n✨ Restauration de la Capsule achevée avec succès !");
            println!(
                "   🌐 Site source : {}",
                report
                    .manifest
                    .source_site_url
                    .as_deref()
                    .unwrap_or("Non spécifié")
            );
            println!(
                "   ✅ Entrées restaurées : {}",
                report.import_report.entries_imported
            );
            println!(
                "   🏷️  Termes/Taxonomies créés : {}",
                report.import_report.terms_imported
            );
            println!(
                "   🖼️  Fichiers médias extraits : {}",
                report.media_files_extracted
            );
            println!("   ⏱️  Temps d'exécution : {} ms", report.execution_time_ms);
        }
    }

    Ok(())
}
