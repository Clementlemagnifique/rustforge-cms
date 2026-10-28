//! Version 2 native snapshots with bounded files, checksums and empty-target restoration.
use super::{
    RFIRImportReport,
    native_backup::{self, NativeSnapshot},
};
use crate::{CoreError, CoreResult, archive, security};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::{Read, Seek, Write},
    path::{Path, PathBuf},
};
use uuid::Uuid;
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsulePackReport {
    pub capsule_path: String,
    pub file_size_bytes: u64,
    pub entries_packed: usize,
    pub media_files_packed: usize,
    pub execution_time_ms: u128,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsuleUnpackReport {
    pub manifest: CapsuleManifest,
    pub import_report: RFIRImportReport,
    pub media_files_extracted: usize,
    pub execution_time_ms: u128,
}

#[derive(Debug, Clone)]
pub struct StoragePaths {
    pub uploads: PathBuf,
    pub themes: PathBuf,
    pub plugins: PathBuf,
    pub protected: PathBuf,
}
impl StoragePaths {
    fn roots(&self) -> [(&'static str, &Path); 4] {
        [
            ("uploads", &self.uploads),
            ("themes", &self.themes),
            ("plugins", &self.plugins),
            ("protected_downloads", &self.protected),
        ]
    }
}

pub struct CapsuleManager;
fn bad(e: impl std::fmt::Display) -> CoreError {
    CoreError::validation(e.to_string())
}
fn checksum(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn collect(
    root: &Path,
    folder: &Path,
    prefix: &str,
    files: &mut BTreeMap<String, Vec<u8>>,
    total: &mut u64,
) -> CoreResult<()> {
    if !folder.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(folder).map_err(bad)? {
        let entry = entry.map_err(bad)?;
        let path = entry.path();
        let kind = entry.file_type().map_err(bad)?;
        if kind.is_symlink() {
            return Err(bad("Sauvegarde : liens symboliques non pris en charge"));
        }
        if kind.is_dir() {
            if prefix == "plugins" {
                continue;
            }
            collect(root, &path, prefix, files, total)?;
        } else if kind.is_file() {
            if prefix == "plugins" && path.extension().is_none_or(|e| e != "wasm") {
                continue;
            }
            let rel = path
                .strip_prefix(root)
                .map_err(bad)?
                .to_string_lossy()
                .replace('\\', "/");
            security::confined_path(root, &rel)?;
            if entry.metadata().map_err(bad)?.len() > archive::MAX_FILE_BYTES {
                return Err(bad("Fichier trop volumineux"));
            }
            let data = fs::read(path).map_err(bad)?;
            *total += data.len() as u64;
            if *total > archive::MAX_TOTAL_BYTES || files.len() + 4 >= archive::MAX_FILES {
                return Err(bad("Quota de sauvegarde dépassé"));
            }
            files.insert(format!("{prefix}/{rel}"), data);
        }
    }
    Ok(())
}

fn validate<R: Read + Seek>(
    zip: &mut ZipArchive<R>,
) -> CoreResult<(CapsuleManifest, NativeSnapshot)> {
    archive::validate(zip)?;
    let manifest: CapsuleManifest =
        serde_json::from_slice(&archive::read_entry(zip, "capsule.json")?).map_err(bad)?;
    if manifest.version != "2.0" {
        return Err(bad(
            "Cette capsule est un ancien export partiel. Une sauvegarde native v2 est requise.",
        ));
    }
    let hashes: BTreeMap<String, String> =
        serde_json::from_slice(&archive::read_entry(zip, "files.json")?).map_err(bad)?;
    if !hashes.contains_key("database.native.json")
        || !hashes.contains_key("capsule.json")
        || hashes.len() + 1 != zip.len()
    {
        return Err(bad("Inventaire de capsule incomplet"));
    }
    for i in 0..zip.len() {
        let name = zip.by_index(i).map_err(bad)?.name().to_string();
        if name == "files.json" {
            continue;
        }
        let digest = hashes
            .get(&name)
            .ok_or_else(|| bad("Fichier non déclaré"))?;
        if name != "database.native.json"
            && name != "capsule.json"
            && !["uploads/", "themes/", "plugins/", "protected_downloads/"]
                .iter()
                .any(|p| name.starts_with(p))
        {
            return Err(bad("Racine de fichier inconnue"));
        }
        if checksum(&archive::read_entry(zip, &name)?) != *digest {
            return Err(bad("Somme de contrôle invalide"));
        }
    }
    let snapshot: NativeSnapshot =
        serde_json::from_slice(&archive::read_entry(zip, "database.native.json")?).map_err(bad)?;
    snapshot.validate()?;
    if snapshot.tables["entries"].len() != manifest.entry_count
        || snapshot.tables["terms"].len() != manifest.term_count
        || snapshot.tables["media"].len() != manifest.media_count
    {
        return Err(bad("Compteurs du manifeste incohérents"));
    }
    Ok((manifest, snapshot))
}

impl CapsuleManager {
    pub async fn pack_to_writer<W: Write + Seek>(
        pool: &PgPool,
        storage: &StoragePaths,
        writer: W,
        site_url: Option<String>,
    ) -> CoreResult<(CapsuleManifest, usize)> {
        let snapshot = native_backup::dump(pool).await?;
        snapshot.validate()?;
        let manifest = CapsuleManifest {
            version: "2.0".into(),
            generator: "RustForge native backup".into(),
            created_at: Utc::now(),
            source_site_url: site_url,
            entry_count: snapshot.tables["entries"].len(),
            term_count: snapshot.tables["terms"].len(),
            media_count: snapshot.tables["media"].len(),
        };
        let mut files = BTreeMap::new();
        let database = serde_json::to_vec(&snapshot).map_err(bad)?;
        if database.len() as u64 > archive::MAX_FILE_BYTES {
            return Err(bad(
                "Sauvegarde native trop volumineuse pour ce format borné",
            ));
        }
        let mut total = database.len() as u64;
        files.insert("database.native.json".into(), database);
        for (prefix, root) in storage.roots() {
            collect(root, root, prefix, &mut files, &mut total)?;
        }
        // A physical file referenced by the database must be present; no silent missing media.
        for media in &snapshot.tables["media"] {
            for column in ["path", "webp_path"] {
                if let Some(path) = media[column].as_str()
                    && !files.contains_key(&format!("uploads/{path}"))
                {
                    return Err(bad(format!("Média physique absent : {path}")));
                }
            }
        }
        let media_files = files.keys().filter(|p| p.starts_with("uploads/")).count();
        files.insert(
            "capsule.json".into(),
            serde_json::to_vec(&manifest).map_err(bad)?,
        );
        let hashes: BTreeMap<_, _> = files
            .iter()
            .map(|(p, b)| (p.clone(), checksum(b)))
            .collect();
        files.insert(
            "files.json".into(),
            serde_json::to_vec(&hashes).map_err(bad)?,
        );
        if files.len() > archive::MAX_FILES
            || files
                .values()
                .any(|b| b.len() as u64 > archive::MAX_FILE_BYTES)
            || files.values().map(|b| b.len() as u64).sum::<u64>() > archive::MAX_TOTAL_BYTES
        {
            return Err(bad("Inventaire compris : quota de sauvegarde dépassé"));
        }
        let mut zip = ZipWriter::new(writer);
        for (path, data) in files {
            zip.start_file(
                path,
                SimpleFileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated)
                    .unix_permissions(0o600),
            )
            .map_err(bad)?;
            zip.write_all(&data).map_err(bad)?;
        }
        zip.finish().map_err(bad)?;
        Ok((manifest, media_files))
    }

    pub async fn pack(
        pool: &PgPool,
        storage: &StoragePaths,
        output: &Path,
        url: Option<String>,
    ) -> CoreResult<CapsulePackReport> {
        let started = std::time::Instant::now();
        if let Some(parent) = output.parent().filter(|p| !p.as_os_str().is_empty()) {
            fs::create_dir_all(parent).map_err(bad)?;
        }
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output)
            .map_err(bad)?;
        let result = Self::pack_to_writer(pool, storage, file, url).await;
        let (manifest, count) = match result {
            Ok(value) => value,
            Err(e) => {
                let _ = fs::remove_file(output);
                return Err(e);
            }
        };
        Ok(CapsulePackReport {
            capsule_path: output.to_string_lossy().into(),
            file_size_bytes: fs::metadata(output).map_err(bad)?.len(),
            entries_packed: manifest.entry_count,
            media_files_packed: count,
            execution_time_ms: started.elapsed().as_millis(),
        })
    }

    pub fn inspect<R: Read + Seek>(reader: R) -> CoreResult<CapsuleManifest> {
        let mut zip = ZipArchive::new(reader).map_err(bad)?;
        Ok(validate(&mut zip)?.0)
    }

    pub async fn unpack<R: Read + Seek>(
        pool: &PgPool,
        storage: &StoragePaths,
        reader: R,
        new_url: Option<String>,
        _author: Uuid,
    ) -> CoreResult<CapsuleUnpackReport> {
        let started = std::time::Instant::now();
        let mut zip = ZipArchive::new(reader).map_err(bad)?;
        let (manifest, mut snapshot) = validate(&mut zip)?;
        if let Some(new) = new_url {
            let parsed = reqwest::Url::parse(&new).map_err(bad)?;
            if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
                return Err(bad("URL cible invalide"));
            }
            if let Some(old) = &manifest.source_site_url {
                fn rewrite(value: &mut serde_json::Value, old: &str, new: &str) {
                    match value {
                        serde_json::Value::String(s) => *s = s.replace(old, new),
                        serde_json::Value::Array(a) => {
                            for v in a {
                                rewrite(v, old, new)
                            }
                        }
                        serde_json::Value::Object(o) => {
                            for v in o.values_mut() {
                                rewrite(v, old, new)
                            }
                        }
                        _ => {}
                    }
                }
                if old.is_empty() {
                    return Err(bad("URL source vide"));
                }
                for rows in snapshot.tables.values_mut() {
                    for row in rows {
                        rewrite(row, old.trim_end_matches('/'), new.trim_end_matches('/'));
                    }
                }
            }
            for row in snapshot.tables.get_mut("settings").expect("validated") {
                if row["key"] == "site.url" {
                    row["value"] = serde_json::json!(new);
                }
            }
        }
        // Validate every destination and hold bounded bytes before modifying either database or files.
        let mut pending = Vec::new();
        for i in 0..zip.len() {
            let name = zip.by_index(i).map_err(bad)?.name().to_string();
            if name == "capsule.json" || name == "files.json" || name == "database.native.json" {
                continue;
            }
            let (prefix, relative) = name
                .split_once('/')
                .ok_or_else(|| bad("Chemin capsule invalide"))?;
            let roots = storage.roots();
            let root = roots
                .iter()
                .find(|(p, _)| *p == prefix)
                .ok_or_else(|| bad("Racine inconnue"))?
                .1;
            fs::create_dir_all(root).map_err(bad)?;
            let destination = security::confined_path(root, relative)?;
            let data = archive::read_entry(&mut zip, &name)?;
            if destination.exists() {
                if fs::read(&destination).map_err(bad)? != data {
                    return Err(CoreError::Conflict(format!(
                        "Fichier cible existant différent : {name}"
                    )));
                }
            } else {
                pending.push((destination, data));
            }
        }
        let mut tx = pool.begin().await?;
        native_backup::restore(&mut tx, &snapshot).await?;
        let mut created = Vec::new();
        let files_result = (|| -> CoreResult<()> {
            for (path, data) in &pending {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).map_err(bad)?;
                }
                let mut out = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(path)
                    .map_err(bad)?;
                created.push(path.clone());
                out.write_all(data).map_err(bad)?;
            }
            Ok(())
        })();
        if let Err(e) = files_result {
            for path in created {
                let _ = fs::remove_file(path);
            }
            return Err(e);
        }
        if let Err(e) = tx.commit().await {
            return Err(CoreError::Internal(anyhow::anyhow!(
                "La confirmation de restauration a échoué ; les fichiers sont conservés. Vérifiez l'état de la base avant reprise : {e}"
            )));
        }
        let uploads = storage.uploads.canonicalize().map_err(bad)?;
        let count = pending
            .iter()
            .filter(|(p, _)| p.starts_with(&uploads))
            .count();
        Ok(CapsuleUnpackReport {
            manifest: manifest.clone(),
            import_report: RFIRImportReport {
                entries_imported: manifest.entry_count,
                terms_imported: manifest.term_count,
                media_detected: manifest.media_count,
                media_imported: manifest.media_count,
                ..Default::default()
            },
            media_files_extracted: count,
            execution_time_ms: started.elapsed().as_millis(),
        })
    }
}
