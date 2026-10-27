use std::{net::SocketAddr, path::PathBuf};

use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use rustforge_api::{AppState, build_router};
use rustforge_core::{models::Role, users::NewUser};
use rustforge_plugins::PluginManager;

#[derive(Parser)]
#[command(name = "rustforge")]
#[command(about = "RustForge CMS — CMS souverain IA-First de nouvelle génération en Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Lance le serveur HTTP RustForge CMS (API + Studio + Uploads).
    Serve {
        #[arg(short, long, env = "PORT", default_value = "8080")]
        port: u16,
        #[arg(long, env = "HOST", default_value = "0.0.0.0")]
        host: String,
    },
    /// Applique les migrations PostgreSQL.
    Migrate,
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

fn load_config_file() {
    let path = std::path::Path::new("config.toml");
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('#') || trimmed.is_empty() || trimmed.starts_with('[') {
                    continue;
                }
                if let Some((k, v)) = trimmed.split_once('=') {
                    let key = k.trim();
                    let val = v.trim().trim_matches('"').trim_matches('\'');
                    match key {
                        "url" if std::env::var("DATABASE_URL").is_err() => {
                            unsafe { std::env::set_var("DATABASE_URL", val); }
                        }
                        "port" if std::env::var("PORT").is_err() => {
                            unsafe { std::env::set_var("PORT", val); }
                        }
                        "host" if std::env::var("HOST").is_err() => {
                            unsafe { std::env::set_var("HOST", val); }
                        }
                        "jwt_secret" if std::env::var("JWT_SECRET").is_err() => {
                            unsafe { std::env::set_var("JWT_SECRET", val); }
                        }
                        "uploads_dir" if std::env::var("UPLOADS_DIR").is_err() => {
                            unsafe { std::env::set_var("UPLOADS_DIR", val); }
                        }
                        "plugins_dir" if std::env::var("PLUGINS_DIR").is_err() => {
                            unsafe { std::env::set_var("PLUGINS_DIR", val); }
                        }
                        "admin_dist" if std::env::var("ADMIN_DIST").is_err() => {
                            unsafe { std::env::set_var("ADMIN_DIST", val); }
                        }
                        "themes_dir" if std::env::var("THEMES_DIR").is_err() => {
                            unsafe { std::env::set_var("THEMES_DIR", val); }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    load_config_file();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rustforge=debug,rustforge_api=debug,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/trustpress".to_string());

    match cli.command {
        Commands::Migrate => {
            println!("🔄 Connexion à PostgreSQL : {database_url}");
            let pool = rustforge_core::connect(&database_url).await?;
            println!("🚀 Application des migrations...");
            rustforge_core::migrate(&pool).await?;
            println!("✅ Base de données à jour !");
        }
        Commands::CreateAdmin { email, username, password, name } => {
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
                        tracing::warn!("Migrations non exécutées ou déjà en cours: {e}");
                    }
                    p
                }
                Err(e) => {
                    tracing::error!("Impossible de joindre la base PostgreSQL ({database_url}): {e}");
                    tracing::warn!("Serveur démarré en attente de la base de données.");
                    return Err(e.into());
                }
            };

            let plugins_dir = std::env::var("PLUGINS_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("plugins"));

            let uploads_dir = std::env::var("UPLOADS_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("uploads"));
            tokio::fs::create_dir_all(&uploads_dir).await?;

            let admin_dist = std::env::var("ADMIN_DIST")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("admin/dist"));

            let themes_dir = std::env::var("THEMES_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("themes"));

            let active_theme = std::env::var("ACTIVE_THEME")
                .unwrap_or_else(|_| "default".to_string());

            let jwt_secret = std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "rustforge-super-secret-development-key-2026".to_string());

            let plugin_manager = PluginManager::new(pool.clone(), plugins_dir).await?;
            let app_state = AppState::new(
                pool.clone(),
                plugin_manager,
                jwt_secret,
                uploads_dir,
                Some(admin_dist),
                themes_dir,
                active_theme,
            );

            let router = build_router(app_state);
            let addr: SocketAddr = format!("{host}:{port}").parse()?;

            // Démarrage du Planificateur Éditorial Souverain en tâche asynchrone Tokio
            rustforge_core::scheduler::start_scheduler(pool.clone(), 30);

            println!("✨ RustForge écoute sur http://{addr}");
            println!("👉 API REST : http://{addr}/api/v1");
            println!("👉 Outils IA / MCP : http://{addr}/api/v1/ai/tools");
            println!("🕒 Planificateur Éditorial : actif (vérification auto toutes les 30s)");

            let listener = tokio::net::TcpListener::bind(&addr).await?;
            axum::serve(listener, router).await?;
        }
        Commands::ImportWxr { file } => {
            println!("🚀 Lancement du moteur DataMorph sur l'export : {:?}", file);
            if !file.exists() {
                anyhow::bail!("Fichier introuvable : {:?}", file);
            }
            let f = std::fs::File::open(&file)?;
            let reader = std::io::BufReader::new(f);
            let doc = rustforge_core::migration::WxrImporter::parse(reader)?;

            println!("📦 Document TMIR analysé avec succès :");
            println!("   • Source : {}", doc.source_site_url.as_deref().unwrap_or("Non spécifiée"));
            println!("   • Entrées trouvées : {}", doc.entries.len());
            println!("   • Termes/Taxonomies : {}", doc.terms.len());
            println!("   • Médias référencés : {}", doc.media.len());

            println!("💾 Application en base PostgreSQL...");
            let pool = rustforge_core::connect(&database_url).await?;

            let author_id: Option<uuid::Uuid> = sqlx::query_scalar("SELECT id FROM users ORDER BY created_at ASC LIMIT 1")
                .fetch_optional(&pool)
                .await?;

            let admin_id = match author_id {
                Some(id) => id,
                None => {
                    let user = rustforge_core::users::create(&pool, rustforge_core::users::NewUser {
                        email: "admin@rustforge.local".into(),
                        username: "admin".into(),
                        password: "DefaultSecurePassword2026!".into(),
                        display_name: Some("Importateur DataMorph".into()),
                        role: Role::Admin,
                    }).await?;
                    user.id
                }
            };

            let report = rustforge_core::migration::DataMorphImporter::apply(&pool, &doc, admin_id).await?;
            println!("\n✨ Rapport d'importation DataMorph :");
            println!("   ✅ Entrées importées : {}", report.entries_imported);
            println!("   ⏭️  Entrées ignorées / doublons : {}", report.entries_skipped);
            println!("   🏷️  Termes créés : {}", report.terms_imported);
            println!("   🖼️  Médias ingérés dans la base : {} (sur {} détectés)", report.media_imported, report.media_detected);
            println!("   🔒 Champs spécifiques préservés dans le coffre-fort _rf_vault : {}", report.legacy_fields_stored_in_vault);
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

            println!("\n🧩 EXTENSIONS DÉTECTÉES & ÉQUIVALENCE NATIVE ({} trouvées) :", result.plugins.len());
            for p in &result.plugins {
                let badge = match p.strategy {
                    rustforge_core::converter::ConversionStrategyKind::NativeReuse => {
                        format!("✅ Remplacé nativement par RustForge [{}]", p.native_module.as_deref().unwrap_or("natif"))
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
            println!("   • Taux de remplacement immédiat : {:.1} %", result.native_coverage_pct);
            println!("   • Accélération mesurée / projetée : {}", result.estimated_speedup);

            println!("\n⚠️  DIAGNOSTIC CYBERSÉCURITÉ RUNTIME :");
            println!("   • Vulnérabilité PHP2Shell (CVE-2026-109618 / 19) : {}", result.php2shell_threat_level);
            println!("   • Immunité RustForge CMS : Garantie mémoire Rust, Zéro PHP en production, Zéro RCE.");

            println!("\n💡 Pour exécuter l'ingestion de données : lancez `rustforge import-content <export.xml>` ou glissez l'archive dans le Studio /admin.");
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
                    println!("\n--- DÉBUT DE L'EXPORT SOUVERAIN (Format: {}) ---", format.to_uppercase());
                    println!("{}", content);
                    println!("--- FIN DE L'EXPORT SOUVERAIN ---");
                }
            }
        }
        Commands::PackCapsule { output, site_url } => {
            println!("📦 Création de la Capsule RustForge complète...");
            let pool = rustforge_core::connect(&database_url).await?;
            let uploads_dir = std::env::var("UPLOADS_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("uploads"));

            let report = rustforge_core::migration::CapsuleManager::pack(&pool, &uploads_dir, &output, site_url).await?;
            println!("\n✨ Capsule créée avec succès !");
            println!("   📁 Fichier : {:?}", report.capsule_path);
            println!("   📦 Entrées sauvegardées : {}", report.entries_packed);
            println!("   🖼️  Fichiers médias empaquetés : {}", report.media_files_packed);
            println!("   ⚖️  Taille de l'archive : {:.2} Ko", (report.file_size_bytes as f64) / 1024.0);
            println!("   ⏱️  Temps d'exécution : {} ms", report.execution_time_ms);
        }
        Commands::UnpackCapsule { file, new_site_url } => {
            println!("🚀 Décapsulation et restauration du site depuis : {:?}", file);
            if !file.exists() {
                anyhow::bail!("Fichier capsule introuvable : {:?}", file);
            }
            let pool = rustforge_core::connect(&database_url).await?;
            rustforge_core::migrate(&pool).await?;

            let uploads_dir = std::env::var("UPLOADS_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("uploads"));

            let author_id: Option<uuid::Uuid> = sqlx::query_scalar("SELECT id FROM users ORDER BY created_at ASC LIMIT 1")
                .fetch_optional(&pool)
                .await?;

            let admin_id = match author_id {
                Some(id) => id,
                None => {
                    let user = rustforge_core::users::create(&pool, rustforge_core::users::NewUser {
                        email: "admin@rustforge.local".into(),
                        username: "admin".into(),
                        password: "DefaultSecurePassword2026!".into(),
                        display_name: Some("Restaurateur de Capsule".into()),
                        role: Role::Admin,
                    }).await?;
                    user.id
                }
            };

            let f = std::fs::File::open(&file)?;
            let report = rustforge_core::migration::CapsuleManager::unpack(&pool, &uploads_dir, f, new_site_url, admin_id).await?;

            println!("\n✨ Restauration de la Capsule achevée avec succès !");
            println!("   🌐 Site source : {}", report.manifest.source_site_url.as_deref().unwrap_or("Non spécifié"));
            println!("   ✅ Entrées restaurées : {}", report.import_report.entries_imported);
            println!("   🏷️  Termes/Taxonomies créés : {}", report.import_report.terms_imported);
            println!("   🖼️  Fichiers médias extraits : {}", report.media_files_extracted);
            println!("   ⏱️  Temps d'exécution : {} ms", report.execution_time_ms);
        }
    }

    Ok(())
}
