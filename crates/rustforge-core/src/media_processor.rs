//! # Traitement & Optimisation Native des Médias en Rust
//!
//! Assure un assainissement et une compression optimale sans dépendances C fragiles :
//! - Conversion automatique et transparente vers **WebP** pour réduire le poids de 40 à 75%.
//! - Génération souveraine des variantes d'images (Thumbnail, Medium, Large) (WP-39, WP-88).
//! - Transformations réversibles en mémoire (Rotation, Recadrage) (WP-39).
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

/// Spécification d'une taille de variante d'image (WP-88).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantSpec {
    pub name: String,
    pub max_width: u32,
    pub max_height: u32,
    pub crop: bool,
}

/// Variante d'image générée (WP-39).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedVariant {
    pub name: String,
    pub bytes: Vec<u8>,
    pub width: i32,
    pub height: i32,
    pub ext: String,
    pub mime: String,
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
    pub variants: Vec<ProcessedVariant>,
}

/// Spécifications par défaut des variantes conformes à WP-88 / WordPress.
pub fn default_variant_specs() -> Vec<VariantSpec> {
    vec![
        VariantSpec {
            name: "thumbnail".to_string(),
            max_width: 150,
            max_height: 150,
            crop: true,
        },
        VariantSpec {
            name: "medium".to_string(),
            max_width: 600,
            max_height: 600,
            crop: false,
        },
        VariantSpec {
            name: "large".to_string(),
            max_width: 1200,
            max_height: 1200,
            crop: false,
        },
    ]
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

/// Génère les variantes dimensionnées d'une image matricielle sans modifier l'original (WP-39, WP-88).
pub fn generate_variants_for_image(
    img: &image::DynamicImage,
    specs: &[VariantSpec],
    quality: f32,
) -> Vec<ProcessedVariant> {
    let (orig_w, orig_h) = (img.width(), img.height());
    let mut out = Vec::new();

    for spec in specs {
        // Ne génère pas de variantes plus grandes que l'original sauf si crop est forcé pour vignette
        if !spec.crop && orig_w <= spec.max_width && orig_h <= spec.max_height {
            continue;
        }

        let resized = if spec.crop {
            img.resize_to_fill(
                spec.max_width,
                spec.max_height,
                image::imageops::FilterType::Lanczos3,
            )
        } else {
            img.resize(
                spec.max_width,
                spec.max_height,
                image::imageops::FilterType::Lanczos3,
            )
        };

        let w = resized.width() as i32;
        let h = resized.height() as i32;

        if let Ok(encoder) = webp::Encoder::from_image(&resized) {
            let data = encoder.encode(quality.clamp(1.0, 100.0));
            out.push(ProcessedVariant {
                name: spec.name.clone(),
                bytes: data.to_vec(),
                width: w,
                height: h,
                ext: "webp".to_string(),
                mime: "image/webp".to_string(),
            });
        }
    }

    out
}

/// Génère les variantes dimensionnées directement depuis les octets d'une image matricielle (WP-39).
pub fn generate_variants_from_bytes(
    bytes: &[u8],
    specs: &[VariantSpec],
    quality: f32,
) -> CoreResult<(Vec<ProcessedVariant>, i32, i32)> {
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

    let w = img.width() as i32;
    let h = img.height() as i32;
    let variants = generate_variants_for_image(&img, specs, quality);
    Ok((variants, w, h))
}

/// Applique une rotation à une image en mémoire (WP-39).
pub fn rotate_image_bytes(
    bytes: &[u8],
    degrees: i32,
    quality: f32,
) -> CoreResult<(Vec<u8>, i32, i32)> {
    if !matches!(degrees, 90 | 180 | 270) {
        return Err(CoreError::validation(
            "La rotation doit être de 90, 180 ou 270 degrés.",
        ));
    }
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

    let rotated = match degrees {
        90 => img.rotate90(),
        180 => img.rotate180(),
        270 => img.rotate270(),
        _ => unreachable!(),
    };

    let w = rotated.width() as i32;
    let h = rotated.height() as i32;

    let encoder = webp::Encoder::from_image(&rotated)
        .map_err(|e| CoreError::validation(format!("erreur encodeur WebP : {e}")))?;
    let webp_data = encoder.encode(quality.clamp(1.0, 100.0));

    Ok((webp_data.to_vec(), w, h))
}

/// Applique un recadrage à une image en mémoire (WP-39).
pub fn crop_image_bytes(
    bytes: &[u8],
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    quality: f32,
) -> CoreResult<(Vec<u8>, i32, i32)> {
    if width == 0 || height == 0 || width > 16384 || height > 16384 {
        return Err(CoreError::validation("Dimensions de recadrage invalides."));
    }
    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| CoreError::validation(e.to_string()))?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(16384);
    limits.max_image_height = Some(16384);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    let mut img = reader
        .decode()
        .map_err(|e| CoreError::validation(format!("impossible de décoder l'image : {e}")))?;

    if x + width > img.width() || y + height > img.height() {
        return Err(CoreError::validation(
            "La zone de recadrage dépasse les limites de l'image originale.",
        ));
    }

    let cropped = img.crop(x, y, width, height);
    let w = cropped.width() as i32;
    let h = cropped.height() as i32;

    let encoder = webp::Encoder::from_image(&cropped)
        .map_err(|e| CoreError::validation(format!("erreur encodeur WebP : {e}")))?;
    let webp_data = encoder.encode(quality.clamp(1.0, 100.0));

    Ok((webp_data.to_vec(), w, h))
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
            variants: Vec::new(),
        });
    }

    // Cas 2 : Images JPEG, PNG ou WebP -> Conversion WebP + Variantes responsives (WP-39, WP-88)
    if matches!(mime_type, "image/jpeg" | "image/png" | "image/webp") {
        let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes))
            .with_guessed_format()
            .map_err(|e| CoreError::validation(e.to_string()))?;
        let mut limits = image::Limits::default();
        limits.max_image_width = Some(16384);
        limits.max_image_height = Some(16384);
        limits.max_alloc = Some(128 * 1024 * 1024);
        reader.limits(limits);
        if let Ok(img) = reader.decode() {
            let w = img.width() as i32;
            let h = img.height() as i32;

            let webp_bytes = match webp::Encoder::from_image(&img) {
                Ok(enc) => Some(enc.encode(82.0).to_vec()),
                Err(_) => None,
            };

            let variants = generate_variants_for_image(&img, &default_variant_specs(), 80.0);

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
                is_webp_converted: webp_bytes.is_some(),
                webp_bytes,
                variants,
            });
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
        variants: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_svg_removes_scripts() {
        let dirty = b"<svg xmlns=\"http://www.w3.org/2000/svg\"><script>alert(1)</script><circle cx=\"10\" cy=\"10\" r=\"5\"/></svg>";
        let cleaned = sanitize_svg(dirty).unwrap();
        let cleaned_str = String::from_utf8(cleaned).unwrap();
        assert!(!cleaned_str.contains("<script"));
        assert!(cleaned_str.contains("<circle"));
    }

    #[test]
    fn test_generate_image_variants() {
        // Crée une image test 1600x1200 en mémoire
        let img = image::DynamicImage::ImageRgba8(image::RgbaImage::new(1600, 1200));
        let specs = default_variant_specs();
        let variants = generate_variants_for_image(&img, &specs, 80.0);

        assert_eq!(variants.len(), 3);
        let thumb = variants.iter().find(|v| v.name == "thumbnail").unwrap();
        assert_eq!(thumb.width, 150);
        assert_eq!(thumb.height, 150);

        let med = variants.iter().find(|v| v.name == "medium").unwrap();
        assert_eq!(med.width, 600);
        assert_eq!(med.height, 450); // Ratio 4:3 respecté

        let lrg = variants.iter().find(|v| v.name == "large").unwrap();
        assert_eq!(lrg.width, 1200);
        assert_eq!(lrg.height, 900); // Ratio 4:3 respecté
    }

    #[test]
    fn test_rotate_and_crop_image() {
        // Création d'une image test 400x200 encodée en PNG
        let mut img = image::RgbaImage::new(400, 200);
        for pixel in img.pixels_mut() {
            *pixel = image::Rgba([120, 150, 200, 255]);
        }
        let dynamic = image::DynamicImage::ImageRgba8(img);
        let mut png_bytes = Vec::new();
        dynamic
            .write_to(
                &mut std::io::Cursor::new(&mut png_bytes),
                image::ImageFormat::Png,
            )
            .unwrap();

        // Rotation 90° -> 200x400
        let (rot90, w, h) = rotate_image_bytes(&png_bytes, 90, 85.0).unwrap();
        assert_eq!(w, 200);
        assert_eq!(h, 400);
        assert!(!rot90.is_empty());

        // Rotation 180° -> 400x200
        let (rot180, w, h) = rotate_image_bytes(&png_bytes, 180, 85.0).unwrap();
        assert_eq!(w, 400);
        assert_eq!(h, 200);
        assert!(!rot180.is_empty());

        // Rotation invalide -> Err
        assert!(rotate_image_bytes(&png_bytes, 45, 85.0).is_err());

        // Recadrage 100x100
        let (cropped, cw, ch) = crop_image_bytes(&png_bytes, 50, 50, 100, 100, 85.0).unwrap();
        assert_eq!(cw, 100);
        assert_eq!(ch, 100);
        assert!(!cropped.is_empty());

        // Recadrage hors limites -> Err
        assert!(crop_image_bytes(&png_bytes, 350, 150, 100, 100, 85.0).is_err());
    }
}
