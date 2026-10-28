//! # Traitement & Optimisation Native des Médias en Rust
//!
//! Assure un assainissement et une compression optimale sans dépendances C fragiles :
//! - Conversion automatique et transparente vers **WebP** pour réduire le poids de 40 à 75%.
//! - Désinfection stricte des fichiers **SVG** (élimination des balises `<script>`, `onload`, `javascript:`).
//! - Validation des types MIME étendus (Excel, Word, PDF, Audio, Vidéo, SVG).

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::{CoreError, CoreResult};

/// Types MIME autorisés dans la médiathèque RustForge.
pub fn is_allowed_mime(mime: &str) -> bool {
    matches!(
        mime,
        // Images matricielles
        "image/jpeg" | "image/png" | "image/gif" | "image/webp" | "image/avif"
        // Images vectorielles (désinfectées)
        | "image/svg+xml"
        // Documents bureautiques & PDF
        | "application/pdf"
        | "application/vnd.ms-excel" // .xls
        | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" // .xlsx
        | "application/msword" // .doc
        | "application/vnd.openxmlformats-officedocument.wordprocessingml.document" // .docx
        | "application/vnd.ms-powerpoint"
        | "application/vnd.openxmlformats-officedocument.presentationml.presentation"
        // Archives & données
        | "application/zip" | "text/plain" | "text/csv"
        // Vidéo & Audio
        | "video/mp4" | "video/webm" | "video/quicktime"
        | "audio/mpeg" | "audio/ogg" | "audio/wav"
    )
}

/// Résultat du traitement d'un fichier média par le pipeline Rust.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedMedia {
    pub final_bytes: Vec<u8>,
    pub final_mime: String,
    pub final_ext: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub is_webp_converted: bool,
    pub webp_bytes: Option<Vec<u8>>,
}

/// Désinfecte un contenu SVG en supprimant tout script ou gestionnaire d'événement suspect.
pub fn sanitize_svg(bytes: &[u8]) -> CoreResult<Vec<u8>> {
    let content = std::str::from_utf8(bytes)
        .map_err(|_| CoreError::validation("fichier SVG encodé de manière invalide"))?;

    let lower = content.to_lowercase();
    if lower.contains("<script")
        || lower.contains("javascript:")
        || lower.contains("onload=")
        || lower.contains("onerror=")
    {
        tracing::warn!(
            "Détection et neutralisation d'éléments non sécurisés dans un SVG téléversé"
        );
    }

    // Filtre Ammonia adapté pour le XML/SVG
    let clean = ammonia::Builder::new()
        .tags(std::collections::HashSet::from([
            "svg",
            "path",
            "g",
            "defs",
            "clippath",
            "circle",
            "rect",
            "line",
            "polyline",
            "polygon",
            "text",
            "tspan",
            "use",
            "symbol",
            "lineargradient",
            "radialgradient",
            "stop",
            "mask",
            "filter",
            "feGaussianBlur",
        ]))
        .clean(content)
        .to_string();

    Ok(clean.into_bytes())
}

/// Compresse et convertit une image matricielle en WebP de haute qualité.
pub fn convert_to_webp(bytes: &[u8], quality: f32) -> CoreResult<(Vec<u8>, i32, i32)> {
    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| CoreError::validation(e.to_string()))?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(16384);
    limits.max_image_height = Some(16384);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    let img = reader
        .decode()
        .map_err(|e| CoreError::validation(format!("impossible de décoder l'image : {e}")))?;

    let width = img.width() as i32;
    let height = img.height() as i32;

    let encoder = webp::Encoder::from_image(&img)
        .map_err(|e| CoreError::validation(format!("erreur encodeur WebP : {e}")))?;

    let webp_data = encoder.encode(quality.clamp(1.0, 100.0));
    Ok((webp_data.to_vec(), width, height))
}

/// Pipeline principal : analyse, assainit et optimise le média ingéré.
pub async fn process_upload(
    bytes: &[u8],
    original_name: &str,
    mime_type: &str,
) -> CoreResult<ProcessedMedia> {
    if bytes.len() > 32 * 1024 * 1024 {
        return Err(CoreError::validation("fichier trop volumineux"));
    }
    let extension = Path::new(original_name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let allowed = match mime_type {
        "image/jpeg" => matches!(extension.as_str(), "jpg" | "jpeg"),
        "image/png" => extension == "png",
        "image/webp" => extension == "webp",
        "image/gif" => extension == "gif",
        "image/avif" => extension == "avif",
        "image/svg+xml" => extension == "svg",
        "application/pdf" => extension == "pdf" && bytes.starts_with(b"%PDF-"),
        "text/plain" => extension == "txt" && std::str::from_utf8(bytes).is_ok(),
        "text/csv" => extension == "csv" && std::str::from_utf8(bytes).is_ok(),
        "application/zip" => extension == "zip" && bytes.starts_with(b"PK"),
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => {
            extension == "xlsx" && bytes.starts_with(b"PK")
        }
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
            extension == "docx" && bytes.starts_with(b"PK")
        }
        "application/vnd.openxmlformats-officedocument.presentationml.presentation" => {
            extension == "pptx" && bytes.starts_with(b"PK")
        }
        "video/mp4" => extension == "mp4" && bytes.get(4..8) == Some(b"ftyp"),
        "video/webm" => extension == "webm" && bytes.starts_with(&[0x1a, 0x45, 0xdf, 0xa3]),
        "audio/ogg" => extension == "ogg" && bytes.starts_with(b"OggS"),
        "audio/wav" => {
            extension == "wav" && bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WAVE")
        }
        "audio/mpeg" => {
            extension == "mp3"
                && (bytes.starts_with(b"ID3")
                    || bytes
                        .get(..2)
                        .is_some_and(|b| b[0] == 0xff && b[1] & 0xe0 == 0xe0))
        }
        _ => false,
    };
    if !allowed {
        return Err(CoreError::validation(
            "Le contenu, l'extension ou le type déclaré n'est pas autorisé.",
        ));
    }
    if mime_type.starts_with("image/") && mime_type != "image/svg+xml" {
        let guessed =
            image::guess_format(bytes).map_err(|_| CoreError::validation("image invalide"))?;
        if guessed.to_mime_type() != mime_type {
            return Err(CoreError::validation("signature d'image incohérente"));
        }
        let size = imagesize::blob_size(bytes)
            .map_err(|_| CoreError::validation("dimensions d'image invalides"))?;
        if size.width > 16384
            || size.height > 16384
            || size.width.saturating_mul(size.height) > 32_000_000
        {
            return Err(CoreError::validation("image trop grande"));
        }
    }
    if !is_allowed_mime(mime_type) {
        return Err(CoreError::validation(format!(
            "type de fichier « {mime_type} » non autorisé"
        )));
    }

    // Cas 1 : SVG (désinfection obligatoire)
    if mime_type == "image/svg+xml" || original_name.to_lowercase().ends_with(".svg") {
        let clean_svg = sanitize_svg(bytes)?;
        return Ok(ProcessedMedia {
            final_bytes: clean_svg,
            final_mime: "image/svg+xml".into(),
            final_ext: "svg".into(),
            width: None,
            height: None,
            is_webp_converted: false,
            webp_bytes: None,
        });
    }

    // Cas 2 : Images JPEG ou PNG ➔ Conversion WebP systématique
    if matches!(mime_type, "image/jpeg" | "image/png" | "image/webp") {
        match convert_to_webp(bytes, 82.0) {
            Ok((webp_bytes, w, h)) => {
                let original_size = bytes.len();
                let webp_size = webp_bytes.len();
                tracing::info!(
                    original_size,
                    webp_size,
                    savings_pct =
                        ((original_size as f64 - webp_size as f64) / original_size as f64) * 100.0,
                    "Image convertie et optimisée en WebP"
                );

                return Ok(ProcessedMedia {
                    final_bytes: bytes.to_vec(),
                    final_mime: mime_type.to_string(),
                    final_ext: Path::new(original_name)
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("jpg")
                        .to_string(),
                    width: Some(w),
                    height: Some(h),
                    is_webp_converted: true,
                    webp_bytes: Some(webp_bytes),
                });
            }
            Err(e) => {
                tracing::warn!("Échec de la conversion WebP, conservation de l'original : {e}");
            }
        }
    }

    // Cas 3 : Autres types (PDF, Excel, Vidéo...)
    let (width, height) = imagesize::blob_size(bytes)
        .map(|s| (Some(s.width as i32), Some(s.height as i32)))
        .unwrap_or((None, None));

    let ext = Path::new(original_name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin")
        .to_string();

    Ok(ProcessedMedia {
        final_bytes: bytes.to_vec(),
        final_mime: mime_type.to_string(),
        final_ext: ext,
        width,
        height,
        is_webp_converted: false,
        webp_bytes: None,
    })
}
