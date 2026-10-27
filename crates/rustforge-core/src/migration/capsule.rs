//! Système d'Encapsulation Universelle RustForge (Capsule Engine).
//! Inspiré par la simplicité de l'outil All-in-One WP Migration :
//! Permet d'encapsuler et de décapsuler un site RustForge complet dans un fichier unique `.rfcapsule` (archive ZIP compressée).
//! 
//! Contenu d'une capsule :
//! - `capsule.json` : Métadonnées du site, version, date, URL d'origine
//! - `database.rfir.json` : Exportation intégrale de la base de données au format pivot RFIR
//! - `uploads/` : Fichiers physiques de la médiathèque (images WebP, SVG, documents)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::Path;
use uuid::Uuid;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

use crate::CoreError;
use super::exporter::DataMorphExporter;
use super::importer::DataMorphImporter;
use super::tmir::{RFIRDocument, RFIRImportReport};

/// Métadonnées d'une capsule RustForge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsuleManifest {
    pub version: String,
    pub generator: String,
    pub created_at: DateTime<Utc>,
    pub source_site_url: Option<String>,
    pub entry_count: usize,
    pub term_count: usize,
    pub media_count: usize,
}

/// Rapport de création d'une capsule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsulePackReport {
    pub capsule_path: String,
    pub file_size_bytes: u64,
    pub entries_packed: usize,
    pub media_files_packed: usize,
    pub execution_time_ms: u128,
}

/// Rapport de restauration d'une capsule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsuleUnpackReport {
    pub manifest: CapsuleManifest,
    pub import_report: RFIRImportReport,
    pub media_files_extracted: usize,
    pub execution_time_ms: u128,
}

pub struct CapsuleManager;

impl CapsuleManager {
    /// Encapsule l'ensemble du site (Base de données + Fichiers médias) directement dans un flux binaire (Write + Seek).
    pub async fn pack_to_writer<W: Write + Seek>(
        pool: &PgPool,
        uploads_dir: &Path,
        writer: W,
        site_url: Option<String>,
    ) -> Result<(CapsuleManifest, usize), CoreError> {
        // 1. Extraction de la base de données sous forme de document pivot RFIR
        let mut doc = DataMorphExporter::dump_database(pool).await?;
        if site_url.is_some() {
            doc.source_site_url = site_url.clone();
        }

        let entry_count = doc.entries.len();
        let term_count = doc.terms.len();
        let media_count = doc.media.len();

        let manifest = CapsuleManifest {
            version: "1.0".to_string(),
            generator: "RustForge Capsule Engine 1.0".to_string(),
            created_at: Utc::now(),
            source_site_url: site_url,
            entry_count,
            term_count,
            media_count,
        };

        // 2. Création de l'archive ZIP
        let mut zip = ZipWriter::new(writer);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);

        // A. Écriture du manifest
        zip.start_file("capsule.json", options)
            .map_err(|e| CoreError::Validation(format!("Erreur ZIP manifest: {}", e)))?;
        let manifest_bytes = serde_json::to_vec_pretty(&manifest)
            .map_err(|e| CoreError::Validation(format!("Erreur sérialisation manifest: {}", e)))?;
        zip.write_all(&manifest_bytes)
            .map_err(|e| CoreError::Validation(format!("Erreur écriture manifest: {}", e)))?;

        // B. Écriture de la base de données
        zip.start_file("database.rfir.json", options)
            .map_err(|e| CoreError::Validation(format!("Erreur ZIP database: {}", e)))?;
        let db_bytes = serde_json::to_vec_pretty(&doc)
            .map_err(|e| CoreError::Validation(format!("Erreur sérialisation BDD: {}", e)))?;
        zip.write_all(&db_bytes)
            .map_err(|e| CoreError::Validation(format!("Erreur écriture BDD: {}", e)))?;

        // C. Écriture des fichiers de la médiathèque (uploads/)
        let mut media_files_packed = 0;
        if uploads_dir.exists() && uploads_dir.is_dir() {
            media_files_packed = Self::zip_directory_recursive(&mut zip, uploads_dir, uploads_dir, options)?;
        }

        zip.finish().map_err(|e| {
            CoreError::Validation(format!("Erreur finalisation capsule ZIP: {}", e))
        })?;

        Ok((manifest, media_files_packed))
    }

    /// Encapsule l'ensemble du site (Base de données + Fichiers médias) dans une archive .rfcapsule sur disque.
    pub async fn pack(
        pool: &PgPool,
        uploads_dir: &Path,
        output_path: &Path,
        site_url: Option<String>,
    ) -> Result<CapsulePackReport, CoreError> {
        let start = std::time::Instant::now();

        if let Some(parent) = output_path.parent() {
            if !parent.as_os_str().is_empty() {
                let _ = std::fs::create_dir_all(parent);
            }
        }

        let file = File::create(output_path).map_err(|e| {
            CoreError::Validation(format!("Impossible de créer le fichier capsule {:?}: {}", output_path, e))
        })?;

        let (manifest, media_files_packed) = Self::pack_to_writer(pool, uploads_dir, file, site_url).await?;

        let file_size_bytes = std::fs::metadata(output_path).map(|m| m.len()).unwrap_or(0);

        Ok(CapsulePackReport {
            capsule_path: output_path.to_string_lossy().to_string(),
            file_size_bytes,
            entries_packed: manifest.entry_count,
            media_files_packed,
            execution_time_ms: start.elapsed().as_millis(),
        })
    }

    /// Inspecte les métadonnées d'une capsule sans la décompresser ni modifier la base de données.
    pub fn inspect<R: Read + Seek>(reader: R) -> Result<CapsuleManifest, CoreError> {
        let mut archive = ZipArchive::new(reader).map_err(|e| {
            CoreError::Validation(format!("Fichier capsule invalide ou corrompu: {}", e))
        })?;

        let mut file = archive.by_name("capsule.json").map_err(|_| {
            CoreError::Validation("Fichier 'capsule.json' manquant dans l'archive".to_string())
        })?;
        let mut content = String::new();
        file.read_to_string(&mut content).map_err(|e| {
            CoreError::Validation(format!("Erreur lecture manifest: {}", e))
        })?;
        serde_json::from_str(&content).map_err(|e| {
            CoreError::Validation(format!("Manifest de capsule invalide: {}", e))
        })
    }

    /// Décapsule et restaure un site complet depuis une archive .rfcapsule.
    pub async fn unpack<R: Read + Seek>(
        pool: &PgPool,
        uploads_dir: &Path,
        reader: R,
        new_site_url: Option<String>,
        author_id: Uuid,
    ) -> Result<CapsuleUnpackReport, CoreError> {
        let start = std::time::Instant::now();

        let mut archive = ZipArchive::new(reader).map_err(|e| {
            CoreError::Validation(format!("Fichier capsule invalide ou corrompu: {}", e))
        })?;

        // 1. Lecture du manifest
        let manifest: CapsuleManifest = {
            let mut file = archive.by_name("capsule.json").map_err(|_| {
                CoreError::Validation("Fichier 'capsule.json' manquant dans l'archive".to_string())
            })?;
            let mut content = String::new();
            file.read_to_string(&mut content).map_err(|e| {
                CoreError::Validation(format!("Erreur lecture manifest: {}", e))
            })?;
            serde_json::from_str(&content).map_err(|e| {
                CoreError::Validation(format!("Manifest de capsule invalide: {}", e))
            })?
        };

        // 2. Lecture et ré-écriture d'URL de la base de données
        let doc: RFIRDocument = {
            let mut file = archive.by_name("database.rfir.json").map_err(|_| {
                CoreError::Validation("Fichier 'database.rfir.json' manquant dans l'archive".to_string())
            })?;
            let mut content = String::new();
            file.read_to_string(&mut content).map_err(|e| {
                CoreError::Validation(format!("Erreur lecture database: {}", e))
            })?;

            // Remplacement d'URL intelligent si un nouveau domaine cible est spécifié (comme All-in-One WP Migration)
            if let (Some(old_url), Some(new_url)) = (&manifest.source_site_url, &new_site_url) {
                if old_url != new_url {
                    content = content.replace(old_url, new_url);
                }
            }

            serde_json::from_str(&content).map_err(|e| {
                CoreError::Validation(format!("Erreur parsing document RFIR: {}", e))
            })?
        };

        // 3. Extraction physique des fichiers médias dans uploads_dir
        let mut media_files_extracted = 0;
        tokio::fs::create_dir_all(uploads_dir).await.map_err(|e| {
            CoreError::Validation(format!("Impossible de créer le dossier uploads {:?}: {}", uploads_dir, e))
        })?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| {
                CoreError::Validation(format!("Erreur lecture fichier index {}: {}", i, e))
            })?;

            let name = file.name().to_string();
            if name.starts_with("uploads/") && !name.ends_with('/') {
                let rel_path = name.trim_start_matches("uploads/");
                let rel_path_obj = Path::new(rel_path);

                // Sécurité anti-Zip Slip : rejet de tout composant parent (..), racine absolue ou préfixe
                let is_safe = !rel_path_obj.components().any(|c| {
                    matches!(
                        c,
                        std::path::Component::ParentDir
                            | std::path::Component::RootDir
                            | std::path::Component::Prefix(_)
                    )
                });

                if !is_safe {
                    tracing::warn!("⚠️ Tentative d'extraction de chemin suspect ignorée (anti-Zip Slip) : {}", rel_path);
                    continue;
                }

                let dest_path = uploads_dir.join(rel_path);

                if let Some(parent) = dest_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }

                let mut out = File::create(&dest_path).map_err(|e| {
                    CoreError::Validation(format!("Impossible d'extraire {:?}: {}", dest_path, e))
                })?;
                std::io::copy(&mut file, &mut out).map_err(|e| {
                    CoreError::Validation(format!("Erreur écriture {:?}: {}", dest_path, e))
                })?;
                media_files_extracted += 1;
            }
        }

        // 4. Ingestion des données dans PostgreSQL via le moteur DataMorph
        let import_report = DataMorphImporter::apply(pool, &doc, author_id).await?;

        Ok(CapsuleUnpackReport {
            manifest,
            import_report,
            media_files_extracted,
            execution_time_ms: start.elapsed().as_millis(),
        })
    }

    fn zip_directory_recursive<W: Write + Seek>(
        zip: &mut ZipWriter<W>,
        base_dir: &Path,
        current_dir: &Path,
        options: SimpleFileOptions,
    ) -> Result<usize, CoreError> {
        let mut count = 0;
        let entries = std::fs::read_dir(current_dir).map_err(|e| {
            CoreError::Validation(format!("Erreur lecture dossier {:?}: {}", current_dir, e))
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                count += Self::zip_directory_recursive(zip, base_dir, &path, options)?;
            } else if path.is_file() {
                let rel = path.strip_prefix(base_dir).unwrap_or(&path);
                let zip_path = format!("uploads/{}", rel.to_string_lossy().replace('\\', "/"));

                zip.start_file(&zip_path, options).map_err(|e| {
                    CoreError::Validation(format!("Erreur ajout fichier ZIP {}: {}", zip_path, e))
                })?;

                let mut f = File::open(&path).map_err(|e| {
                    CoreError::Validation(format!("Erreur ouverture {:?}: {}", path, e))
                })?;
                let mut buf = Vec::new();
                f.read_to_end(&mut buf).map_err(|e| {
                    CoreError::Validation(format!("Erreur lecture {:?}: {}", path, e))
                })?;
                zip.write_all(&buf).map_err(|e| {
                    CoreError::Validation(format!("Erreur écriture fichier ZIP: {}", e))
                })?;
                count += 1;
            }
        }

        Ok(count)
    }
}
