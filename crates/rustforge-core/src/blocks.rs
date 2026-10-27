//! # Registre Universel de Blocs Sémantiques (Universal Block Registry)
//!
//! Définit la structure centrale des blocs de contenu de RustForge.
//! Chaque bloc possède un type strict, un schéma JSON pour ses données et
//! des fonctions de rendu HTML sémantiques réutilisables en SSR et Headless.

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

impl Block {
    pub fn new(id: impl Into<String>, block_type: BlockType, data: Value) -> Self {
        Self {
            id: id.into(),
            block_type,
            data,
            settings: json!({}),
        }
    }

    /// Rendu HTML par défaut d'un bloc individuel
    pub fn to_html(&self) -> String {
        match self.block_type {
            BlockType::Heading => {
                let level = self.data.get("level").and_then(|v| v.as_u64()).unwrap_or(2).clamp(1, 6);
                let text = self.data.get("text").and_then(|v| v.as_str()).unwrap_or("");
                format!("<h{level} class=\"tp-heading\">{text}</h{level}>")
            }
            BlockType::Paragraph => {
                let text = self.data.get("text").and_then(|v| v.as_str()).unwrap_or("");
                format!("<p class=\"tp-paragraph\">{text}</p>")
            }
            BlockType::Quote => {
                let text = self.data.get("text").and_then(|v| v.as_str()).unwrap_or("");
                let author = self.data.get("author").and_then(|v| v.as_str()).unwrap_or("");
                let cite = if !author.is_empty() {
                    format!("<cite class=\"tp-cite\">— {author}</cite>")
                } else {
                    String::new()
                };
                format!("<blockquote class=\"tp-quote\"><p>{text}</p>{cite}</blockquote>")
            }
            BlockType::Callout => {
                let text = self.data.get("text").and_then(|v| v.as_str()).unwrap_or("");
                let variant = self.data.get("variant").and_then(|v| v.as_str()).unwrap_or("info");
                let icon = match variant {
                    "warning" => "⚠️",
                    "success" => "✅",
                    "danger" => "⛔",
                    _ => "💡",
                };
                format!(
                    r#"<div class="tp-callout tp-callout-{variant}" style="padding: 1rem; border-left: 4px solid var(--tp-primary); background: rgba(255,255,255,0.03); border-radius: var(--tp-radius); margin: 1.5rem 0;">
                        <span style="margin-right: 0.5rem;">{icon}</span> {text}
                    </div>"#
                )
            }
            BlockType::Code => {
                let code = self.data.get("code").and_then(|v| v.as_str()).unwrap_or("");
                let lang = self.data.get("language").and_then(|v| v.as_str()).unwrap_or("text");
                format!("<pre class=\"tp-code-block\"><code class=\"language-{lang}\">{code}</code></pre>")
            }
            BlockType::Image => {
                let url = self.data.get("url").and_then(|v| v.as_str()).unwrap_or("");
                let alt = self.data.get("alt").and_then(|v| v.as_str()).unwrap_or("");
                let caption = self.data.get("caption").and_then(|v| v.as_str()).unwrap_or("");
                let cap_html = if !caption.is_empty() {
                    format!("<figcaption style=\"text-align:center;font-size:0.85rem;color:#94a3b8;margin-top:0.5rem;\">{caption}</figcaption>")
                } else {
                    String::new()
                };
                format!(
                    r#"<figure style="margin: 2rem 0;">
                        <img src="{url}" alt="{alt}" loading="lazy" style="max-width: 100%; height: auto; border-radius: var(--tp-radius);" />
                        {cap_html}
                    </figure>"#
                )
            }
            BlockType::Button => {
                let text = self.data.get("text").and_then(|v| v.as_str()).unwrap_or("En savoir plus");
                let url = self.data.get("url").and_then(|v| v.as_str()).unwrap_or("#");
                format!(
                    r#"<div style="margin: 1.5rem 0;">
                        <a href="{url}" class="btn-primary" style="display:inline-block;padding:0.75rem 1.5rem;background:var(--tp-primary);color:white;text-decoration:none;border-radius:var(--tp-radius);font-weight:600;">{text}</a>
                    </div>"#
                )
            }
            BlockType::Divider => "<hr style=\"border: 0; border-top: 1px solid rgba(255,255,255,0.1); margin: 2.5rem 0;\" />".to_string(),
            BlockType::FormEmbed => {
                let form_id = self.data.get("form_id").and_then(|v| v.as_str()).unwrap_or("");
                format!("<div class=\"tp-form-container\" data-form-id=\"{form_id}\"></div>")
            }
            BlockType::GalleryEmbed => {
                let gallery_id = self.data.get("gallery_id").and_then(|v| v.as_str()).unwrap_or("");
                format!("<div class=\"tp-gallery-container\" data-gallery-id=\"{gallery_id}\"></div>")
            }
            BlockType::CourseCard => {
                let course_id = self.data.get("course_id").and_then(|v| v.as_str()).unwrap_or("");
                format!("<div class=\"tp-course-card\" data-course-id=\"{course_id}\"></div>")
            }
            BlockType::ProductCard => {
                let product_id = self.data.get("product_id").and_then(|v| v.as_str()).unwrap_or("");
                format!("<div class=\"tp-product-card\" data-product-id=\"{product_id}\"></div>")
            }
            BlockType::Faq => {
                let q = self.data.get("question").and_then(|v| v.as_str()).unwrap_or("");
                let a = self.data.get("answer").and_then(|v| v.as_str()).unwrap_or("");
                format!(
                    r#"<details style="border: 1px solid rgba(255,255,255,0.08); padding: 1rem; border-radius: var(--tp-radius); margin-bottom: 0.75rem;">
                        <summary style="font-weight: 600; cursor: pointer; color: white;">{q}</summary>
                        <p style="margin-top: 0.75rem; color: #94a3b8;">{a}</p>
                    </details>"#
                )
            }
            BlockType::CustomEmbed => {
                let raw_html = self.data.get("raw_html").and_then(|v| v.as_str()).unwrap_or("");
                format!("<div class=\"tp-custom-embed\">{raw_html}</div>")
            }
            BlockType::Columns => {
                let inner = self.data.get("columns").and_then(|v| v.as_array());
                let mut cols_html = String::new();
                if let Some(cols) = inner {
                    for col in cols {
                        let text = col.get("text").and_then(|v| v.as_str()).unwrap_or("");
                        cols_html.push_str(&format!("<div style=\"flex: 1;\"><p>{text}</p></div>"));
                    }
                }
                format!("<div class=\"tp-columns\" style=\"display:flex;gap:1.5rem;\">{cols_html}</div>")
            }
        }
    }
}

/// Convertit un tableau JSON de blocs en flux HTML continu
pub fn render_blocks_to_html(blocks: &[Block]) -> String {
    blocks.iter().map(|b| b.to_html()).collect::<Vec<_>>().join("\n")
}
