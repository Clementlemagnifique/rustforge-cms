//! Common archive limits, applied to every ingestion endpoint before reading entries.
use crate::{CoreError, CoreResult};
use std::{
    collections::HashSet,
    io::{Read, Seek},
};
use zip::ZipArchive;

pub const MAX_FILES: usize = 2_000;
pub const MAX_TOTAL_BYTES: u64 = 512 * 1024 * 1024;
pub const MAX_FILE_BYTES: u64 = 128 * 1024 * 1024;

pub fn read_text(reader: impl Read) -> CoreResult<String> {
    let mut text = String::new();
    reader
        .take(MAX_FILE_BYTES + 1)
        .read_to_string(&mut text)
        .map_err(|e| CoreError::validation(e.to_string()))?;
    if text.len() as u64 > MAX_FILE_BYTES {
        return Err(CoreError::validation("texte décompressé trop volumineux"));
    }
    Ok(text)
}

pub fn validate<R: Read + Seek>(zip: &mut ZipArchive<R>) -> CoreResult<()> {
    if zip.len() > MAX_FILES {
        return Err(CoreError::validation("archive : trop de fichiers"));
    }
    let mut names = HashSet::new();
    let mut total = 0u64;
    for i in 0..zip.len() {
        let file = zip
            .by_index(i)
            .map_err(|e| CoreError::validation(e.to_string()))?;
        crate::security::relative_path(file.name().trim_end_matches('/'))?;
        if !names.insert(file.name().to_lowercase()) {
            return Err(CoreError::validation("archive : chemins dupliqués"));
        }
        if file.is_symlink() {
            return Err(CoreError::validation("archive : lien symbolique interdit"));
        }
        total = total
            .checked_add(file.size())
            .ok_or_else(|| CoreError::validation("archive trop grande"))?;
        if file.size() > MAX_FILE_BYTES || total > MAX_TOTAL_BYTES {
            return Err(CoreError::validation("archive : quota décompressé dépassé"));
        }
    }
    Ok(())
}

pub fn read_entry<R: Read + Seek>(zip: &mut ZipArchive<R>, name: &str) -> CoreResult<Vec<u8>> {
    let file = zip
        .by_name(name)
        .map_err(|e| CoreError::validation(e.to_string()))?;
    let mut data = Vec::new();
    file.take(MAX_FILE_BYTES + 1)
        .read_to_end(&mut data)
        .map_err(|e| CoreError::validation(e.to_string()))?;
    if data.len() as u64 > MAX_FILE_BYTES {
        return Err(CoreError::validation("entrée trop volumineuse"));
    }
    Ok(data)
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Write};
    #[test]
    fn conversion_cannot_bypass_zip_path_checks() {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        writer
            .start_file("../theme.json", zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(b"{}").unwrap();
        let data = writer.finish().unwrap().into_inner();
        assert!(crate::converter::FseThemeConverter::convert_zip(&data, Some("test")).is_err());
    }
}
