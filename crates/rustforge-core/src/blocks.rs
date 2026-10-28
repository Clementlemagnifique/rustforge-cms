//! # Registre Universel de Blocs Sémantiques (Universal Block Registry)
//!
//! Définit la structure centrale des blocs de contenu de RustForge.
//! Chaque bloc possède un type strict, un schéma JSON pour ses données et
//! des fonctions de rendu HTML sémantiques assainies réutilisables en SSR et Headless.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockType {
    // Blocs Fondamentaux (Toujours actifs)
    Paragraph,
    Heading,
    Image,
    Quote,
    Code,
    Callout,
    Columns,
    Button,
    Faq,
    Divider,

    // Blocs Métiers Spécialisés (Conditionnés aux modules actifs)
    FormEmbed,
    GalleryEmbed,
    CourseCard,
    ProductCard,

    // Bloc de repli pour les éléments non reconnus / migrés
    CustomEmbed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub id: String,
    #[serde(rename = "type")]
    pub block_type: BlockType,
    pub data: Value,
    #[serde(default)]
    pub settings: Value,
}

fn escape_html(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _ => out.push(ch),
        }
    }
    out
}

fn safe_identifier<'a>(raw: &'a str, fallback: &'a str) -> &'a str {
    if !raw.is_empty()
        && raw.len() <= 64
        && raw
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        raw
    } else {
        fallback
    }
}

fn safe_url(raw: &str, image_only: bool) -> &str {
    let trimmed = raw.trim();
    if trimmed.is_empty()
        || trimmed.len() > 2048
        || trimmed.chars().any(|c| {
            c.is_whitespace()
                || c.is_control()
                || matches!(c, '<' | '>' | '"' | '\'' | '\\' | '`')
        })
    {
        return if image_only { "" } else { "#" };
    }
    let lower = trimmed.to_ascii_lowercase();
    let ok = if image_only {
        (trimmed.starts_with('/')
            && !trimmed.starts_with("//")
            && !trimmed.split('/').any(|s| s == ".."))
            || lower.starts_with("https://")
            || lower.starts_with("http://")
    } else {
        (trimmed.starts_with('/')
            && !trimmed.starts_with("//")
            && !trimmed.split('/').any(|s| s == ".."))
            || (trimmed.starts_with('#') && trimmed.len() > 1)
            || lower.starts_with("https://")
            || lower.starts_with("http://")
            || lower.starts_with("mailto:")
    };
    if ok {
        trimmed
    } else if image_only {
        ""
    } else {
        "#"
    }
}

fn sanitize_block_html(raw: &str) -> String {
    ammonia::Builder::default()
        .add_tags(["figure", "figcaption", "details", "summary", "cite"])
        .add_generic_attributes(["id", "class"])
        .add_tag_attributes("img", ["loading"])
        .add_tag_attributes(
            "div",
            [
                "data-form-id",
                "data-gallery-id",
                "data-course-id",
                "data-product-id",
            ],
        )
        .clean(raw)
        .to_string()
}

impl Block {
    pub fn new(id: impl Into<String>, block_type: BlockType, data: Value) -> Self {
        Self {
            id: id.into(),
            block_type,
            data,
            settings: json!({}),
        }
    }

    /// Rendu HTML assaini d'un bloc individuel
    pub fn to_html(&self) -> String {
        let raw = match self.block_type {
            BlockType::Heading => {
                let level = self
                    .data
                    .get("level")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(2)
                    .clamp(1, 6);
                let text = escape_html(
                    self.data
                        .get("text")
                        .and_then(|v| v.as_str())
                        .unwrap_or(""),
                );
                format!("<h{level} class=\"rf-heading tp-heading\">{text}</h{level}>")
            }
            BlockType::Paragraph => {
                let text = escape_html(
                    self.data
                        .get("text")
                        .and_then(|v| v.as_str())
                        .unwrap_or(""),
                );
                format!("<p class=\"rf-paragraph tp-paragraph\">{text}</p>")
            }
            BlockType::Quote => {
                let text = escape_html(
                    self.data
                        .get("text")
                        .and_then(|v| v.as_str())
                        .unwrap_or(""),
                );
                let author = escape_html(
                    self.data
                        .get("author")
                        .and_then(|v| v.as_str())
                        .unwrap_or(""),
                );
                let cite = if !author.is_empty() {
                    format!("<cite class=\"rf-cite tp-cite\">— {author}</cite>")
                } else {
                    String::new()
                };
                format!("<blockquote class=\"rf-quote tp-quote\"><p>{text}</p>{cite}</blockquote>")
            }
            BlockType::Callout => {
                let text = escape_html(
                    self.data
                        .get("text")
                        .and_then(|v| v.as_str())
                        .unwrap_or(""),
                );
                let raw_variant = self
                    .data
                    .get("variant")
                    .and_then(|v| v.as_str())
                    .unwrap_or("info");
                let variant = match raw_variant {
                    "warning" | "success" | "danger" | "info" => raw_variant,
                    _ => "info",
                };
                let icon = match variant {
                    "warning" => "⚠️",
                    "success" => "✅",
                    "danger" => "⛔",
                    _ => "💡",
                };
                format!(
                    "<div class=\"rf-callout rf-callout-{variant} tp-callout tp-callout-{variant}\"><span>{icon}</span> {text}</div>"
                )
            }
            BlockType::Code => {
                let code = escape_html(
                    self.data
                        .get("code")
                        .and_then(|v| v.as_str())
                        .unwrap_or(""),
                );
                let lang = safe_identifier(
                    self.data
                        .get("language")
                        .and_then(|v| v.as_str())
                        .unwrap_or("text"),
                    "text",
                );
                format!(
                    "<pre class=\"rf-code-block tp-code-block\"><code class=\"language-{lang}\">{code}</code></pre>"
                )
            }
            BlockType::Image => {
                let url = escape_html(safe_url(
                    self.data.get("url").and_then(|v| v.as_str()).unwrap_or(""),
                    true,
                ));
                let alt = escape_html(
                    self.data.get("alt").and_then(|v| v.as_str()).unwrap_or(""),
                );
                let caption = escape_html(
                    self.data
                        .get("caption")
                        .and_then(|v| v.as_str())
                        .unwrap_or(""),
                );
                if url.is_empty() {
                    String::new()
                } else {
                    let cap_html = if !caption.is_empty() {
                        format!("<figcaption>{caption}</figcaption>")
                    } else {
                        String::new()
                    };
                    format!(
                        "<figure class=\"rf-figure\"><img src=\"{url}\" alt=\"{alt}\" loading=\"lazy\" />{cap_html}</figure>"
                    )
                }
            }
            BlockType::Button => {
                let text = escape_html(
                    self.data
                        .get("text")
                        .and_then(|v| v.as_str())
                        .unwrap_or("En savoir plus"),
                );
                let url = escape_html(safe_url(
                    self.data.get("url").and_then(|v| v.as_str()).unwrap_or("#"),
                    false,
                ));
                format!(
                    "<div class=\"rf-button-wrap\"><a href=\"{url}\" class=\"rf-button btn-primary\">{text}</a></div>"
                )
            }
            BlockType::Divider => "<hr class=\"rf-divider\" />".to_string(),
            BlockType::FormEmbed => {
                let form_id = safe_identifier(
                    self.data
                        .get("form_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or(""),
                    "",
                );
                format!(
                    "<div class=\"rf-form-container tp-form-container\" data-form-id=\"{form_id}\"></div>"
                )
            }
            BlockType::GalleryEmbed => {
                let gallery_id = safe_identifier(
                    self.data
                        .get("gallery_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or(""),
                    "",
                );
                format!(
                    "<div class=\"rf-gallery-container tp-gallery-container\" data-gallery-id=\"{gallery_id}\"></div>"
                )
            }
            BlockType::CourseCard => {
                let course_id = safe_identifier(
                    self.data
                        .get("course_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or(""),
                    "",
                );
                format!(
                    "<div class=\"rf-course-card tp-course-card\" data-course-id=\"{course_id}\"></div>"
                )
            }
            BlockType::ProductCard => {
                let product_id = safe_identifier(
                    self.data
                        .get("product_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or(""),
                    "",
                );
                format!(
                    "<div class=\"rf-product-card tp-product-card\" data-product-id=\"{product_id}\"></div>"
                )
            }
            BlockType::Faq => {
                let q = escape_html(
                    self.data
                        .get("question")
                        .and_then(|v| v.as_str())
                        .unwrap_or(""),
                );
                let a = escape_html(
                    self.data
                        .get("answer")
                        .and_then(|v| v.as_str())
                        .unwrap_or(""),
                );
                format!("<details class=\"rf-faq\"><summary>{q}</summary><p>{a}</p></details>")
            }
            BlockType::CustomEmbed => {
                let raw_html = self
                    .data
                    .get("raw_html")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                format!("<div class=\"rf-custom-embed tp-custom-embed\">{raw_html}</div>")
            }
            BlockType::Columns => {
                let inner = self.data.get("columns").and_then(|v| v.as_array());
                let mut cols_html = String::new();
                if let Some(cols) = inner {
                    for col in cols {
                        let text = escape_html(
                            col.get("text").and_then(|v| v.as_str()).unwrap_or(""),
                        );
                        cols_html.push_str(&format!("<div class=\"rf-column\"><p>{text}</p></div>"));
                    }
                }
                format!("<div class=\"rf-columns tp-columns\">{cols_html}</div>")
            }
        };
        sanitize_block_html(&raw)
    }
}

/// Convertit un tableau JSON de blocs en flux HTML continu et assaini
pub fn render_blocks_to_html(blocks: &[Block]) -> String {
    let joined = blocks
        .iter()
        .map(|b| b.to_html())
        .collect::<Vec<_>>()
        .join("\n");
    sanitize_block_html(&joined)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_html_escapes_injections_and_rejects_unsafe_urls() {
        let blocks = vec![
            Block::new(
                "b1",
                BlockType::Heading,
                json!({"level": 2, "text": "Titre <script>alert(1)</script>"}),
            ),
            Block::new(
                "b2",
                BlockType::Image,
                json!({"url": "javascript:alert(1)", "alt": "\" onerror=\"alert(1)"}),
            ),
            Block::new(
                "b3",
                BlockType::Image,
                json!({"url": "/uploads/2026/10/ok.webp", "alt": "Visuel <sûr>", "caption": "Légende"}),
            ),
            Block::new(
                "b4",
                BlockType::Button,
                json!({"text": "Cliquer", "url": "javascript:evil()"}),
            ),
            Block::new(
                "b5",
                BlockType::CustomEmbed,
                json!({"raw_html": "<p>Texte autorisé</p><script>steal()</script><img src=x onerror=alert(1)>"}),
            ),
            Block::new(
                "b6",
                BlockType::FormEmbed,
                json!({"form_id": "form_1\" onclick=\"alert(1)"}),
            ),
        ];

        let html = render_blocks_to_html(&blocks);
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onerror"));
        assert!(!html.contains("onclick"));
        assert!(!html.contains("javascript:"));
        assert!(html.contains("src=\"/uploads/2026/10/ok.webp\""));
        assert!(html.contains("<figcaption>Légende</figcaption>"));
        assert!(html.contains("data-form-id=\"\""));
    }
}
