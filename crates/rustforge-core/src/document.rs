//! Premier format de document natif. Les identifiants sont indépendants de la langue.
//! Le HTML passe par le renderer Markdown assaini existant, également utilisé en aperçu.
use crate::{CoreError, CoreResult, markdown};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub const SCHEMA: &str = "rf.document/1";
const MAX_BYTES: usize = 512 * 1024;
pub const MAX_BLOCKS: usize = 256;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Document {
    rf_schema: String,
    rf_blocks: Vec<Block>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Block {
    rf_id: String,
    rf_type: Kind,
    #[serde(default)]
    rf_text: String,
    #[serde(default = "heading_level")]
    rf_level: u8,
    #[serde(default)]
    rf_ordered: bool,
    #[serde(default)]
    rf_items: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    rf_url: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    rf_alt: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    rf_caption: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    rf_media_id: Option<Uuid>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    rf_style: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    rf_ref: String,
}

impl Block {
    fn new(id: String, kind: Kind) -> Self {
        Self {
            rf_id: id,
            rf_type: kind,
            rf_text: String::new(),
            rf_level: 2,
            rf_ordered: false,
            rf_items: Vec::new(),
            rf_url: String::new(),
            rf_alt: String::new(),
            rf_caption: String::new(),
            rf_media_id: None,
            rf_style: String::new(),
            rf_ref: String::new(),
        }
    }
}

fn heading_level() -> u8 {
    2
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum Kind {
    Paragraph,
    Heading,
    Quote,
    Code,
    List,
    Divider,
    Image,
    Button,
    Callout,
    Details,
    Columns,
    MediaText,
    Pattern,
    Table,
    Spacer,
    Cover,
    Group,
    Product,
    Course,
}

fn validate_safe_url(raw: &str, image_only: bool) -> CoreResult<&str> {
    if raw.is_empty() || raw.len() > 2048 || raw.trim() != raw {
        return Err(CoreError::validation("URL du bloc absente ou trop longue"));
    }
    if raw.chars().any(|c| {
        c.is_whitespace()
            || c.is_control()
            || matches!(c, '<' | '>' | '"' | '\'' | '(' | ')' | '\\' | '`')
    }) {
        return Err(CoreError::validation(
            "Caractères interdits dans l’URL du bloc",
        ));
    }
    let lower = raw.to_ascii_lowercase();
    let allowed = if image_only {
        (raw.starts_with('/') && !raw.starts_with("//") && !raw.split('/').any(|s| s == ".."))
            || lower.starts_with("https://")
            || lower.starts_with("http://")
    } else {
        (raw.starts_with('/') && !raw.starts_with("//") && !raw.split('/').any(|s| s == ".."))
            || (raw.starts_with('#') && raw.len() > 1)
            || lower.starts_with("https://")
            || lower.starts_with("http://")
            || lower.starts_with("mailto:")
    };
    if !allowed {
        return Err(CoreError::validation(
            "Schéma ou chemin d’URL non autorisé dans le bloc",
        ));
    }
    Ok(raw)
}

fn literal(text: &str) -> String {
    // Texte simple: ni HTML arbitraire, ni lien actif, ni syntaxe injectée.
    let mut output = String::with_capacity(text.len());
    for ch in text.chars() {
        if ch == '&' {
            output.push_str("&amp;");
            continue;
        }
        if "\\`*_{}[]<>()#+-.!|>~".contains(ch) {
            output.push('\\');
        }
        if ch != '\r' {
            output.push(ch);
        }
    }
    output
}

fn safe_rich_text(text: &str) -> String {
    // Échapper strictement HTML brut pour éviter les injections XSS,
    // tout en préservant le formatage Markdown en ligne (gras, italique, code, liens).
    let mut output = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '\r' => {}
            _ => output.push(ch),
        }
    }
    output
}

/// Les blocs hérités restent opaques. Seule une enveloppe native explicite est interprétée.
pub fn project(body: &str, blocks: &Value) -> CoreResult<(String, String)> {
    let Some(schema) = blocks.get("rf_schema") else {
        return Ok((body.to_owned(), markdown::render(body)));
    };
    if schema.as_str() != Some(SCHEMA) {
        return Err(CoreError::validation(
            "Version du document natif non prise en charge",
        ));
    }
    if serde_json::to_vec(blocks)
        .map_err(|e| CoreError::validation(e.to_string()))?
        .len()
        > MAX_BYTES
    {
        return Err(CoreError::validation("Document natif trop volumineux"));
    }
    let document: Document = serde_json::from_value(blocks.clone())
        .map_err(|_| CoreError::validation("Structure du document natif invalide"))?;
    if document.rf_blocks.len() > MAX_BLOCKS {
        return Err(CoreError::validation("Trop de blocs dans le document"));
    }
    let mut seen = std::collections::HashSet::new();
    let mut parts = Vec::new();
    for block in document.rf_blocks {
        if block.rf_id.is_empty()
            || block.rf_id.len() > 64
            || !block
                .rf_id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            || !seen.insert(block.rf_id.clone())
        {
            return Err(CoreError::validation(
                "Identifiant de bloc invalide ou dupliqué",
            ));
        }
        if block.rf_text.len() > 64 * 1024
            || block.rf_items.len() > 256
            || block.rf_items.iter().any(|item| item.len() > 8192)
            || block.rf_url.len() > 2048
            || block.rf_alt.len() > 512
            || block.rf_caption.len() > 1024
            || block.rf_style.len() > 128
            || block.rf_ref.len() > 96
        {
            return Err(CoreError::validation("Contenu de bloc trop volumineux"));
        }
        let text = literal(&block.rf_text);
        if (!matches!(block.rf_type, Kind::List | Kind::Columns | Kind::Pattern | Kind::Table)
            && (!block.rf_items.is_empty() || block.rf_ordered))
            || (matches!(block.rf_type, Kind::List | Kind::Divider | Kind::Image | Kind::Columns | Kind::Spacer)
                && !block.rf_text.is_empty())
            || (!matches!(block.rf_type, Kind::Heading) && block.rf_level != 2)
            || (!matches!(block.rf_type, Kind::Image | Kind::Button | Kind::MediaText | Kind::Cover | Kind::Product | Kind::Course)
                && !block.rf_url.is_empty())
            || (!matches!(block.rf_type, Kind::Image | Kind::Details | Kind::MediaText | Kind::Pattern | Kind::Quote | Kind::Cover | Kind::Table | Kind::Product | Kind::Course)
                && (!block.rf_alt.is_empty()
                    || !block.rf_caption.is_empty()
                    || block.rf_media_id.is_some()))
            || (matches!(block.rf_type, Kind::Quote | Kind::Table)
                && (!block.rf_alt.is_empty() || block.rf_media_id.is_some()))
            || (!matches!(block.rf_type, Kind::Callout | Kind::MediaText | Kind::Columns | Kind::Pattern | Kind::Button | Kind::Spacer | Kind::Cover | Kind::Group | Kind::Product | Kind::Course)
                && !block.rf_style.is_empty())
            || (!matches!(block.rf_type, Kind::Pattern | Kind::Product | Kind::Course) && !block.rf_ref.is_empty())
        {
            return Err(CoreError::validation(
                "Données incompatibles avec le type du bloc",
            ));
        }

        if matches!(block.rf_type, Kind::Product) && block.rf_text.trim().is_empty() {
            return Err(CoreError::validation(
                "Le bloc produit requiert un titre de produit",
            ));
        }

        if matches!(block.rf_type, Kind::Course) && block.rf_text.trim().is_empty() {
            return Err(CoreError::validation(
                "Le bloc formation requiert un titre de formation",
            ));
        }

        if matches!(block.rf_type, Kind::Columns)
            && (block.rf_items.is_empty() || block.rf_items.len() > 4)
        {
            return Err(CoreError::validation(
                "Un bloc colonnes doit comporter entre 1 et 4 colonnes",
            ));
        }
        if matches!(block.rf_type, Kind::MediaText) && block.rf_url.is_empty() {
            return Err(CoreError::validation(
                "L'URL du média est requise pour le bloc média & texte",
            ));
        }
        if matches!(block.rf_type, Kind::Cover) && block.rf_url.is_empty() {
            return Err(CoreError::validation(
                "L'URL de l'image est requise pour le bloc couverture",
            ));
        }
        if matches!(block.rf_type, Kind::Table)
            && block.rf_items.is_empty()
            && block.rf_text.is_empty()
        {
            return Err(CoreError::validation(
                "Le bloc tableau doit comporter des lignes ou du contenu",
            ));
        }
        if matches!(block.rf_type, Kind::Pattern) {
            let r = block.rf_ref.trim();
            if r.is_empty()
                || r.len() > 96
                || !r
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            {
                return Err(CoreError::validation(
                    "Référence du motif synchronisé invalide",
                ));
            }
        }

        parts.push(match block.rf_type {
            Kind::Paragraph => safe_rich_text(&block.rf_text),
            Kind::Heading => {
                if !(1..=6).contains(&block.rf_level) || text.contains('\n') {
                    return Err(CoreError::validation("Niveau ou texte du titre invalide"));
                }
                format!("{} {text}", "#".repeat(usize::from(block.rf_level)))
            }
            Kind::Quote => {
                let quote_content = safe_rich_text(&block.rf_text)
                    .lines()
                    .map(|line| format!("> {line}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                let cite = block.rf_caption.trim();
                if cite.is_empty() {
                    quote_content
                } else {
                    format!("{quote_content}\n>\n> *— {}*", literal(cite).replace('\n', " "))
                }
            }
            Kind::List => block
                .rf_items
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    let marker = if block.rf_ordered {
                        format!("{}.", i + 1)
                    } else {
                        "-".into()
                    };
                    let indent = format!("\n{}", " ".repeat(marker.len() + 1));
                    format!("{marker} {}", safe_rich_text(item).replace('\n', &indent))
                })
                .collect::<Vec<_>>()
                .join("\n"),
            Kind::Code => {
                let longest = block
                    .rf_text
                    .split(|c| c != '`')
                    .map(str::len)
                    .max()
                    .unwrap_or(0);
                if longest > 128 {
                    return Err(CoreError::validation("Délimiteur de code trop long"));
                }
                let fence = "`".repeat((longest + 1).max(3));
                format!("{fence}\n{}\n{fence}", block.rf_text)
            }
            Kind::Divider => "---".into(),
            Kind::Image => {
                let url = validate_safe_url(&block.rf_url, true)?;
                let alt = literal(block.rf_alt.trim()).replace('\n', " ");
                let cap = block.rf_caption.trim();
                if cap.is_empty() {
                    format!("![{alt}]({url})")
                } else {
                    format!("![{alt}]({url})\n*{}*", literal(cap).replace('\n', " "))
                }
            }
            Kind::Button => {
                let url = validate_safe_url(&block.rf_url, false)?;
                let trimmed = block.rf_text.trim();
                if trimmed.is_empty() || block.rf_text.contains('\n') || trimmed.len() > 256 {
                    return Err(CoreError::validation(
                        "Libellé du bouton ou lien d’action invalide",
                    ));
                }
                let style = match block.rf_style.as_str() {
                    "secondary" => "secondary",
                    "outline" => "outline",
                    _ => "primary",
                };
                let escaped_text = literal(trimmed);
                format!("<div class=\"rf-button rf-button-{style}\"><a href=\"{url}\" class=\"rf-btn rf-btn-{style}\">{escaped_text}</a></div>")
            }
            Kind::Callout => {
                let style = match block.rf_style.as_str() {
                    "warning" => "warning",
                    "success" => "success",
                    "tip" => "tip",
                    _ => "info",
                };
                let badge = match style {
                    "warning" => "Attention",
                    "success" => "Succès",
                    "tip" => "Astuce",
                    _ => "Information",
                };
                let content = safe_rich_text(&block.rf_text);
                format!("<div class=\"rf-callout rf-callout-{style}\"><div class=\"rf-callout-header\"><strong>{badge}</strong></div>\n\n{content}\n\n</div>")
            }
            Kind::Details => {
                let summary = if block.rf_caption.trim().is_empty() {
                    "Détails"
                } else {
                    block.rf_caption.trim()
                };
                let safe_summary = literal(summary);
                let content = safe_rich_text(&block.rf_text);
                format!("<details class=\"rf-details\"><summary><strong>{safe_summary}</strong></summary>\n\n<div class=\"rf-details-body\">\n\n{content}\n\n</div>\n\n</details>")
            }
            Kind::Columns => {
                let cols: Vec<String> = block
                    .rf_items
                    .iter()
                    .map(|item| {
                        let safe = safe_rich_text(item);
                        format!("<div class=\"rf-column\">\n\n{safe}\n\n</div>")
                    })
                    .collect();
                let class = match block.rf_style.as_str() {
                    "50-50" => "rf-columns rf-columns-50-50",
                    "33-33-33" => "rf-columns rf-columns-33-33-33",
                    "25-75" => "rf-columns rf-columns-25-75",
                    "75-25" => "rf-columns rf-columns-75-25",
                    "wide" => "rf-columns rf-columns-wide",
                    _ => "rf-columns",
                };
                format!("<div class=\"{class}\">\n{}\n</div>", cols.join("\n"))
            }
            Kind::MediaText => {
                let url = validate_safe_url(&block.rf_url, true)?;
                let alt = literal(block.rf_alt.trim()).replace('\n', " ");
                let pos = if block.rf_style == "media-right" {
                    "rf-media-right"
                } else {
                    "rf-media-left"
                };
                let content = safe_rich_text(&block.rf_text);
                format!("<div class=\"rf-media-text {pos}\"><div class=\"rf-media-figure\"><img src=\"{url}\" alt=\"{alt}\" /></div><div class=\"rf-media-content\">\n\n{content}\n\n</div></div>")
            }
            Kind::Pattern => {
                let r = block.rf_ref.trim();
                let title = if block.rf_caption.trim().is_empty() {
                    "Motif synchronisé"
                } else {
                    block.rf_caption.trim()
                };
                let safe_title = title
                    .replace('&', "&amp;")
                    .replace('"', "&quot;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;");
                let inner = if block.rf_items.is_empty() {
                    safe_rich_text(&block.rf_text)
                } else {
                    block
                        .rf_items
                        .iter()
                        .map(|item| safe_rich_text(item))
                        .collect::<Vec<_>>()
                        .join("\n\n")
                };
                format!("<div class=\"rf-synced-pattern\" data-pattern=\"{r}\" data-pattern-title=\"{safe_title}\">\n\n{inner}\n\n</div>")
            }
            Kind::Table => {
                let mut table_lines = Vec::new();
                if !block.rf_items.is_empty() {
                    let header = &block.rf_items[0];
                    let cols: Vec<&str> = header.split('|').map(|col| col.trim()).collect();
                    let col_count = cols.len().max(1);
                    table_lines.push(format!("| {} |", cols.join(" | ")));
                    table_lines.push(format!("| {} |", vec!["---"; col_count].join(" | ")));
                    for row in &block.rf_items[1..] {
                        let row_cols: Vec<&str> = row.split('|').map(|col| col.trim()).collect();
                        table_lines.push(format!("| {} |", row_cols.join(" | ")));
                    }
                } else if !block.rf_text.is_empty() {
                    table_lines.push(block.rf_text.clone());
                }
                let table_md = table_lines.join("\n");
                let cite = block.rf_caption.trim();
                if cite.is_empty() {
                    table_md
                } else {
                    format!("{table_md}\n\n*Tableau : {}*", literal(cite).replace('\n', " "))
                }
            }
            Kind::Spacer => {
                let style = match block.rf_style.as_str() {
                    "sm" => "sm",
                    "lg" => "lg",
                    "xl" => "xl",
                    _ => "md",
                };
                format!("<div class=\"rf-spacer rf-spacer-{style}\" aria-hidden=\"true\"></div>")
            }
            Kind::Cover => {
                let url = validate_safe_url(&block.rf_url, true)?;
                let alt = literal(block.rf_alt.trim()).replace('\n', " ");
                let style = match block.rf_style.as_str() {
                    "light" => "light",
                    "primary" => "primary",
                    "gradient" => "gradient",
                    _ => "dark",
                };
                let content = safe_rich_text(&block.rf_text);
                format!("<div class=\"rf-cover rf-cover-{style}\"><div class=\"rf-cover-bg\"><img src=\"{url}\" alt=\"{alt}\" /></div><div class=\"rf-cover-content\">\n\n{content}\n\n</div></div>")
            }
            Kind::Group => {
                let style = match block.rf_style.as_str() {
                    "bordered" => "bordered",
                    "muted" => "muted",
                    "highlight" => "highlight",
                    _ => "card",
                };
                let content = safe_rich_text(&block.rf_text);
                format!("<div class=\"rf-group rf-group-{style}\">\n\n{content}\n\n</div>")
            }
            Kind::Product => {
                let title = literal(block.rf_text.trim());
                let price = literal(block.rf_caption.trim());
                let btn_label = if block.rf_alt.trim().is_empty() {
                    "Acheter"
                } else {
                    block.rf_alt.trim()
                };
                let url = if block.rf_url.trim().is_empty() {
                    "#"
                } else {
                    validate_safe_url(&block.rf_url, false)?
                };
                let style = match block.rf_style.as_str() {
                    "compact" => "compact",
                    "cta" => "cta",
                    _ => "card",
                };
                let product_id = block.rf_ref.trim().replace('"', "&quot;");
                format!(
                    "<div class=\"rf-product-card rf-product-{style}\" data-product-id=\"{product_id}\"><div class=\"rf-product-details\"><h3 class=\"rf-product-title\">{title}</h3><div class=\"rf-product-price\">{price}</div></div><a href=\"{url}\" class=\"rf-btn rf-btn-primary\">{btn_label}</a></div>"
                )
            }
            Kind::Course => {
                let title = literal(block.rf_text.trim());
                let meta = literal(block.rf_caption.trim());
                let btn_label = if block.rf_alt.trim().is_empty() {
                    "Accéder à la formation"
                } else {
                    block.rf_alt.trim()
                };
                let url = if block.rf_url.trim().is_empty() {
                    "#"
                } else {
                    validate_safe_url(&block.rf_url, false)?
                };
                let style = match block.rf_style.as_str() {
                    "compact" => "compact",
                    "hero" => "hero",
                    _ => "card",
                };
                let course_id = block.rf_ref.trim().replace('"', "&quot;");
                format!(
                    "<div class=\"rf-course-card rf-course-{style}\" data-course-id=\"{course_id}\"><div class=\"rf-course-details\"><h3 class=\"rf-course-title\">{title}</h3><div class=\"rf-course-meta\">{meta}</div></div><a href=\"{url}\" class=\"rf-btn rf-btn-primary\">{btn_label}</a></div>"
                )
            }
        });
    }
    let body = parts.join("\n\n");
    if body.len() > MAX_BYTES {
        return Err(CoreError::validation(
            "Projection du document trop volumineuse",
        ));
    }
    let html = markdown::render(&body);
    Ok((body, html))
}

fn parse_markdown_image_line(line: &str) -> Option<(String, String)> {
    let rest = line.strip_prefix("![")?;
    let (alt, url_part) = rest.split_once("](")?;
    let url = url_part.strip_suffix(')')?;
    validate_safe_url(url, true).ok()?;
    Some((alt.to_string(), url.to_string()))
}

fn parse_markdown_button_line(line: &str) -> Option<(String, String)> {
    let rest = line.strip_prefix('[')?;
    let (label, url_part) = rest.split_once("](")?;
    let url = url_part.strip_suffix(')')?;
    let trimmed = label.trim();
    if trimmed.is_empty() || trimmed.len() > 256 || trimmed.contains('\n') {
        return None;
    }
    validate_safe_url(url, false).ok()?;
    Some((trimmed.to_string(), url.to_string()))
}

/// Convertit explicitement un corps Markdown en enveloppe `rf.document/1` validée.
pub fn from_markdown(markdown: &str) -> CoreResult<Value> {
    if markdown.len() > MAX_BYTES {
        return Err(CoreError::validation("Document source trop volumineux"));
    }
    let normalized = markdown.replace("\r\n", "\n");
    let lines: Vec<&str> = normalized.lines().collect();
    let mut blocks = Vec::new();
    let mut idx = 0;
    let mut seq = 0usize;

    let mut next_id = || -> CoreResult<String> {
        seq += 1;
        if seq > MAX_BLOCKS {
            return Err(CoreError::validation("Trop de blocs dans le document"));
        }
        Ok(format!("b_{seq}"))
    };

    while idx < lines.len() {
        let line = lines[idx];
        let trimmed = line.trim();
        if trimmed.is_empty() {
            idx += 1;
            continue;
        }

        // 1. Bloc de code délimité par ``` ou ~~~
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            let fence_char = trimmed.chars().next().unwrap_or('`');
            let fence_len = trimmed.chars().take_while(|&c| c == fence_char).count();
            idx += 1;
            let mut code_lines = Vec::new();
            while idx < lines.len() {
                let candidate = lines[idx].trim();
                let closing_len = candidate.chars().take_while(|&c| c == fence_char).count();
                if closing_len >= fence_len && candidate.chars().all(|c| c == fence_char) {
                    idx += 1;
                    break;
                }
                code_lines.push(lines[idx]);
                idx += 1;
            }
            let mut b = Block::new(next_id()?, Kind::Code);
            b.rf_text = code_lines.join("\n");
            blocks.push(b);
            continue;
        }

        // 2. Séparateur horizontal
        if matches!(trimmed, "---" | "***" | "___") {
            blocks.push(Block::new(next_id()?, Kind::Divider));
            idx += 1;
            continue;
        }

        // 3. Titre # .. ######
        let hashes = trimmed.chars().take_while(|&c| c == '#').count();
        if (1..=6).contains(&hashes) && trimmed[hashes..].starts_with(' ') {
            let heading_text = trimmed[hashes..].trim().to_string();
            if !heading_text.is_empty() {
                let mut b = Block::new(next_id()?, Kind::Heading);
                b.rf_text = heading_text;
                b.rf_level = hashes as u8;
                blocks.push(b);
                idx += 1;
                continue;
            }
        }

        // 4. Citation >
        if trimmed.starts_with('>') {
            let mut quote_lines = Vec::new();
            while idx < lines.len() {
                let t = lines[idx].trim();
                if let Some(stripped) = t.strip_prefix('>') {
                    quote_lines.push(stripped.strip_prefix(' ').unwrap_or(stripped));
                    idx += 1;
                } else {
                    break;
                }
            }
            let mut caption = String::new();
            if let Some(last) = quote_lines.last() {
                let lt = last.trim();
                if let Some(author) = lt.strip_prefix("*— ").and_then(|s| s.strip_suffix('*')) {
                    caption = author.trim().to_string();
                    quote_lines.pop();
                    if let Some(prev) = quote_lines.last()
                        && prev.trim().is_empty()
                    {
                        quote_lines.pop();
                    }
                }
            }
            let mut b = Block::new(next_id()?, Kind::Quote);
            b.rf_text = quote_lines.join("\n");
            b.rf_caption = caption;
            blocks.push(b);
            continue;
        }

        // 5. Liste à puces (- , * , + )
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
            let mut items = Vec::new();
            while idx < lines.len() && items.len() < 256 {
                let t = lines[idx].trim();
                if let Some(rest) = t
                    .strip_prefix("- ")
                    .or_else(|| t.strip_prefix("* "))
                    .or_else(|| t.strip_prefix("+ "))
                {
                    items.push(rest.trim().to_string());
                    idx += 1;
                } else {
                    break;
                }
            }
            let mut b = Block::new(next_id()?, Kind::List);
            b.rf_items = items;
            blocks.push(b);
            continue;
        }

        // 6. Liste numérotée (1. , 2. , ...)
        let digits = trimmed.chars().take_while(|c| c.is_ascii_digit()).count();
        if digits > 0 && trimmed[digits..].starts_with(". ") {
            let mut items = Vec::new();
            while idx < lines.len() && items.len() < 256 {
                let t = lines[idx].trim();
                let d = t.chars().take_while(|c| c.is_ascii_digit()).count();
                if d > 0 && t[d..].starts_with(". ") {
                    items.push(t[d + 2..].trim().to_string());
                    idx += 1;
                } else {
                    break;
                }
            }
            let mut b = Block::new(next_id()?, Kind::List);
            b.rf_ordered = true;
            b.rf_items = items;
            blocks.push(b);
            continue;
        }

        // 7. Image autonome ![alt](url) suivie d'une légende optionnelle *légende*
        if let Some((alt, url)) = parse_markdown_image_line(trimmed) {
            let mut caption = String::new();
            if idx + 1 < lines.len() {
                let next_trimmed = lines[idx + 1].trim();
                if next_trimmed.len() >= 3
                    && next_trimmed.starts_with('*')
                    && next_trimmed.ends_with('*')
                    && !next_trimmed.starts_with("**")
                {
                    caption = next_trimmed[1..next_trimmed.len() - 1].trim().to_string();
                    idx += 1;
                }
            }
            let mut b = Block::new(next_id()?, Kind::Image);
            b.rf_url = url;
            b.rf_alt = alt;
            b.rf_caption = caption;
            blocks.push(b);
            idx += 1;
            continue;
        }

        // 8. Bouton / lien autonome ou conteneur rf-button
        if trimmed.starts_with("<div class=\"rf-button") && trimmed.contains("</a></div>") {
            let style = if trimmed.contains("rf-button-secondary") {
                "secondary".to_string()
            } else if trimmed.contains("rf-button-outline") {
                "outline".to_string()
            } else {
                "primary".to_string()
            };
            if let Some(href_start) = trimmed.find("href=\"") {
                let rest = &trimmed[href_start + 6..];
                if let Some(href_end) = rest.find('"') {
                    let href = &rest[..href_end];
                    if let Some(label_start) = rest.find('>') {
                        let label_rest = &rest[label_start + 1..];
                        if let Some(label_end) = label_rest.find("</a>") {
                            let label = &label_rest[..label_end];
                            if let Ok(safe_url) = validate_safe_url(href, false) {
                                let mut b = Block::new(next_id()?, Kind::Button);
                                b.rf_text = label.trim().to_string();
                                b.rf_url = safe_url.to_string();
                                b.rf_style = style;
                                blocks.push(b);
                                idx += 1;
                                continue;
                            }
                        }
                    }
                }
            }
        }

        let next_is_blank = idx + 1 >= lines.len() || lines[idx + 1].trim().is_empty();
        if next_is_blank && let Some((label, url)) = parse_markdown_button_line(trimmed) {
            let mut b = Block::new(next_id()?, Kind::Button);
            b.rf_text = label;
            b.rf_url = url;
            b.rf_style = "primary".to_string();
            blocks.push(b);
            idx += 1;
            continue;
        }

        // 9. Tableau GFM (| col 1 | col 2 |)
        if trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.contains('|') {
            let mut t_rows = Vec::new();
            while idx < lines.len() {
                let cand = lines[idx].trim();
                if cand.starts_with('|') && cand.ends_with('|') {
                    // Ignorer la ligne de séparateur standard | --- | --- |
                    let inner = &cand[1..cand.len() - 1];
                    let is_separator = inner.split('|').all(|c| {
                        let t = c.trim();
                        !t.is_empty() && t.chars().all(|ch| ch == '-' || ch == ':')
                    });
                    if !is_separator {
                        let row_clean = inner
                            .split('|')
                            .map(|c| c.trim())
                            .collect::<Vec<_>>()
                            .join(" | ");
                        t_rows.push(row_clean);
                    }
                    idx += 1;
                } else {
                    break;
                }
            }
            let mut caption = String::new();
            let mut lookahead = idx;
            while lookahead < lines.len() && lines[lookahead].trim().is_empty() {
                lookahead += 1;
            }
            if lookahead < lines.len() {
                let next_t = lines[lookahead].trim();
                if let Some(rest) = next_t
                    .strip_prefix("*Tableau : ")
                    .and_then(|s| s.strip_suffix('*'))
                {
                    caption = rest.trim().to_string();
                    idx = lookahead + 1;
                }
            }
            if !t_rows.is_empty() {
                let mut b = Block::new(next_id()?, Kind::Table);
                b.rf_items = t_rows;
                b.rf_caption = caption;
                blocks.push(b);
                continue;
            }
        }

        // 10. Espaceur (<div class="rf-spacer rf-spacer-...")
        if trimmed.starts_with("<div class=\"rf-spacer") {
            let style = if trimmed.contains("rf-spacer-sm") {
                "sm".to_string()
            } else if trimmed.contains("rf-spacer-lg") {
                "lg".to_string()
            } else if trimmed.contains("rf-spacer-xl") {
                "xl".to_string()
            } else {
                "md".to_string()
            };
            let mut b = Block::new(next_id()?, Kind::Spacer);
            b.rf_style = style;
            blocks.push(b);
            idx += 1;
            continue;
        }

        // 11. Couverture (<div class="rf-cover rf-cover-...")
        if trimmed.starts_with("<div class=\"rf-cover") {
            let style = if trimmed.contains("rf-cover-light") {
                "light".to_string()
            } else if trimmed.contains("rf-cover-primary") {
                "primary".to_string()
            } else if trimmed.contains("rf-cover-gradient") {
                "gradient".to_string()
            } else {
                "dark".to_string()
            };
            let mut url = String::new();
            let mut alt = String::new();
            if let Some(src_pos) = trimmed.find("src=\"") {
                let rest = &trimmed[src_pos + 5..];
                if let Some(src_end) = rest.find('"') {
                    url = rest[..src_end].to_string();
                }
            }
            if let Some(alt_pos) = trimmed.find("alt=\"") {
                let rest = &trimmed[alt_pos + 5..];
                if let Some(alt_end) = rest.find('"') {
                    alt = rest[..alt_end].to_string();
                }
            }
            let mut content = String::new();
            if let Some(content_pos) = trimmed.find("<div class=\"rf-cover-content\">") {
                let rest = &trimmed[content_pos + 30..];
                if let Some(content_end) = rest.rfind("</div></div>") {
                    content = rest[..content_end].trim().to_string();
                }
            }
            let mut b = Block::new(next_id()?, Kind::Cover);
            b.rf_style = style;
            b.rf_url = url;
            b.rf_alt = alt;
            b.rf_text = content;
            blocks.push(b);
            idx += 1;
            continue;
        }

        // 12. Groupe / Carte (<div class="rf-group rf-group-...")
        if trimmed.starts_with("<div class=\"rf-group") {
            let style = if trimmed.contains("rf-group-bordered") {
                "bordered".to_string()
            } else if trimmed.contains("rf-group-muted") {
                "muted".to_string()
            } else if trimmed.contains("rf-group-highlight") {
                "highlight".to_string()
            } else {
                "card".to_string()
            };
            let mut content = String::new();
            if let Some(content_start) = trimmed.find('>') {
                let rest = &trimmed[content_start + 1..];
                if let Some(content_end) = rest.rfind("</div>") {
                    content = rest[..content_end].trim().to_string();
                }
            }
            let mut b = Block::new(next_id()?, Kind::Group);
            b.rf_style = style;
            b.rf_text = content;
            blocks.push(b);
            idx += 1;
            continue;
        }

        // 13. Produit Boutique (<div class="rf-product-card...")
        if trimmed.starts_with("<div class=\"rf-product-card") {
            let style = if trimmed.contains("rf-product-compact") {
                "compact".to_string()
            } else if trimmed.contains("rf-product-cta") {
                "cta".to_string()
            } else {
                "card".to_string()
            };
            let product_id = if let Some(pos) = trimmed.find("data-product-id=\"") {
                let sub = &trimmed[pos + 17..];
                sub.split('"').next().unwrap_or("").to_string()
            } else {
                String::new()
            };
            let title = if let Some(pos) = trimmed.find("rf-product-title\">") {
                let sub = &trimmed[pos + 18..];
                sub.split('<').next().unwrap_or("").replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">")
            } else {
                String::new()
            };
            let price = if let Some(pos) = trimmed.find("rf-product-price\">") {
                let sub = &trimmed[pos + 18..];
                sub.split('<').next().unwrap_or("").replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">")
            } else {
                String::new()
            };
            let (url, btn_label) = if let Some(pos) = trimmed.find("<a href=\"") {
                let sub = &trimmed[pos + 9..];
                let u = sub.split('"').next().unwrap_or("").to_string();
                let label = if let Some(lpos) = sub.find('>') {
                    let lsub = &sub[lpos + 1..];
                    lsub.split('<').next().unwrap_or("").replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">")
                } else {
                    "Acheter".to_string()
                };
                (u, label)
            } else {
                (String::new(), "Acheter".to_string())
            };

            let mut b = Block::new(next_id()?, Kind::Product);
            b.rf_style = style;
            b.rf_ref = product_id;
            b.rf_text = title;
            b.rf_caption = price;
            b.rf_url = if url == "#" { String::new() } else { url };
            b.rf_alt = btn_label;
            blocks.push(b);
            idx += 1;
            continue;
        }

        // 14. Formation LMS (<div class="rf-course-card...")
        if trimmed.starts_with("<div class=\"rf-course-card") {
            let style = if trimmed.contains("rf-course-compact") {
                "compact".to_string()
            } else if trimmed.contains("rf-course-hero") {
                "hero".to_string()
            } else {
                "card".to_string()
            };
            let course_id = if let Some(pos) = trimmed.find("data-course-id=\"") {
                let sub = &trimmed[pos + 16..];
                sub.split('"').next().unwrap_or("").to_string()
            } else {
                String::new()
            };
            let title = if let Some(pos) = trimmed.find("rf-course-title\">") {
                let sub = &trimmed[pos + 17..];
                sub.split('<').next().unwrap_or("").replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">")
            } else {
                String::new()
            };
            let meta = if let Some(pos) = trimmed.find("rf-course-meta\">") {
                let sub = &trimmed[pos + 16..];
                sub.split('<').next().unwrap_or("").replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">")
            } else {
                String::new()
            };
            let (url, btn_label) = if let Some(pos) = trimmed.find("<a href=\"") {
                let sub = &trimmed[pos + 9..];
                let u = sub.split('"').next().unwrap_or("").to_string();
                let label = if let Some(lpos) = sub.find('>') {
                    let lsub = &sub[lpos + 1..];
                    lsub.split('<').next().unwrap_or("").replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">")
                } else {
                    "Accéder à la formation".to_string()
                };
                (u, label)
            } else {
                (String::new(), "Accéder à la formation".to_string())
            };

            let mut b = Block::new(next_id()?, Kind::Course);
            b.rf_style = style;
            b.rf_ref = course_id;
            b.rf_text = title;
            b.rf_caption = meta;
            b.rf_url = if url == "#" { String::new() } else { url };
            b.rf_alt = btn_label;
            blocks.push(b);
            idx += 1;
            continue;
        }

        // 15. Paragraphe
        let mut para_lines = Vec::new();
        while idx < lines.len() {
            let t = lines[idx].trim();
            if t.is_empty()
                || t.starts_with("```")
                || t.starts_with("~~~")
                || matches!(t, "---" | "***" | "___")
                || t.starts_with('>')
                || t.starts_with("- ")
                || t.starts_with("* ")
                || t.starts_with("+ ")
                || (t.starts_with('|') && t.ends_with('|'))
                || t.starts_with("<div class=\"rf-")
                || (t.chars().take_while(|&c| c == '#').count() >= 1
                    && t.chars().take_while(|&c| c == '#').count() <= 6
                    && t[t.chars().take_while(|&c| c == '#').count()..].starts_with(' '))
            {
                break;
            }
            para_lines.push(lines[idx]);
            idx += 1;
        }
        let mut b = Block::new(next_id()?, Kind::Paragraph);
        b.rf_text = para_lines.join("\n");
        blocks.push(b);
    }

    if blocks.is_empty() {
        blocks.push(Block::new("b_1".into(), Kind::Paragraph));
    }

    let doc = serde_json::to_value(Document {
        rf_schema: SCHEMA.into(),
        rf_blocks: blocks,
    })
    .map_err(|e| CoreError::validation(e.to_string()))?;
    project("", &doc)?;
    Ok(doc)
}

/// Empêcher un client texte/outil IA de prétendre modifier un document natif en ne changeant que sa projection.
pub fn validate_source_patch(
    current_blocks: &Value,
    current_body: &str,
    patch: &Value,
) -> CoreResult<()> {
    if current_blocks.get("rf_schema").is_some()
        && patch
            .get("body")
            .and_then(Value::as_str)
            .is_some_and(|body| body != current_body)
        && patch.get("blocks").is_none_or(Value::is_null)
    {
        return Err(CoreError::Conflict("Ce contenu utilise des blocs natifs. Modifiez ses blocs ou confirmez une conversion en source Markdown.".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn native_document_uses_the_existing_safe_renderer() {
        let doc = json!({"rf_schema":SCHEMA,"rf_blocks":[
            {"rf_id":"title","rf_type":"heading","rf_text":"Un titre","rf_level":2},
            {"rf_id":"text","rf_type":"paragraph","rf_text":"<img src=x onerror=alert(1)> **texte littéral** et [lien](https://rustforge.dev)"},
            {"rf_id":"list","rf_type":"list","rf_items":["premier","second"]},
            {"rf_id":"code","rf_type":"code","rf_text":"```\n<script>bad()</script>"}
        ]});
        let (body, html) = project("source périmée", &doc).unwrap();
        assert!(html.contains("<h2>Un titre</h2>"));
        assert!(html.contains("<ul>"));
        assert!(!html.contains("<img src=x"));
        assert!(!html.contains("<script>"));
        assert!(html.contains("<strong>texte littéral</strong>"));
        assert!(html.contains("href=\"https://rustforge.dev\"") && html.contains(">lien</a>"));
        assert_eq!(html, markdown::render(&body));
        let saved = serde_json::to_string(&doc).unwrap();
        assert_eq!(
            project("", &serde_json::from_str::<Value>(&saved).unwrap()).unwrap(),
            (body, html)
        );
    }

    #[test]
    fn rich_blocks_render_and_validate_correctly() {
        // Callout
        let callout_doc = json!({"rf_schema":SCHEMA,"rf_blocks":[
            {"rf_id":"c1","rf_type":"callout","rf_text":"Information importante avec `code`","rf_style":"warning"}
        ]});
        let (_, html) = project("", &callout_doc).unwrap();
        assert!(html.contains("rf-callout rf-callout-warning"));
        assert!(html.contains("Attention"));
        assert!(html.contains("<code>code</code>"));

        // Details (accordéon)
        let details_doc = json!({"rf_schema":SCHEMA,"rf_blocks":[
            {"rf_id":"d1","rf_type":"details","rf_text":"Réponse détaillée **ici**","rf_caption":"Foire aux questions"}
        ]});
        let (_, html) = project("", &details_doc).unwrap();
        assert!(html.contains("<details class=\"rf-details\"><summary><strong>Foire aux questions</strong></summary>"));
        assert!(html.contains("<strong>ici</strong>"));

        // Columns (colonnes WP-B016 / WP-B015)
        let cols_doc = json!({"rf_schema":SCHEMA,"rf_blocks":[
            {"rf_id":"col1","rf_type":"columns","rf_items":["Colonne 1 **forte**","Colonne 2 avec [lien](/contact)"]}
        ]});
        let (_, html) = project("", &cols_doc).unwrap();
        assert!(html.contains("<div class=\"rf-columns\">"));
        assert!(html.contains("<div class=\"rf-column\">"));
        assert!(html.contains("<strong>forte</strong>"));
        assert!(html.contains("href=\"/contact\"") && html.contains(">lien</a>"));

        // MediaText (WP-B049)
        let mt_doc = json!({"rf_schema":SCHEMA,"rf_blocks":[
            {"rf_id":"mt1","rf_type":"media_text","rf_url":"/uploads/photo.webp","rf_alt":"Illustration","rf_text":"Présentation du produit","rf_style":"media-right"}
        ]});
        let (_, html) = project("", &mt_doc).unwrap();
        assert!(html.contains("rf-media-text rf-media-right"));
        assert!(html.contains("<img src=\"/uploads/photo.webp\" alt=\"Illustration\""));
        assert!(html.contains("Présentation du produit"));

        // Validations d'erreur sur blocs riches
        let invalid_cols = json!({"rf_schema":SCHEMA,"rf_blocks":[
            {"rf_id":"col_err","rf_type":"columns","rf_items":[]}
        ]});
        assert!(project("", &invalid_cols).is_err());

        let invalid_mt = json!({"rf_schema":SCHEMA,"rf_blocks":[
            {"rf_id":"mt_err","rf_type":"media_text","rf_url":"","rf_text":"Sans image"}
        ]});
        assert!(project("", &invalid_mt).is_err());

        // Columns avec style de disposition (50-50, 33-33-33)
        let cols_styled = json!({"rf_schema":SCHEMA,"rf_blocks":[
            {"rf_id":"col_styled","rf_type":"columns","rf_style":"50-50","rf_items":["A","B"]}
        ]});
        let (_, html) = project("", &cols_styled).unwrap();
        assert!(html.contains("rf-columns rf-columns-50-50"));

        // Pattern synchronisé (synced pattern - WP-34, STAB-07)
        let pattern_doc = json!({"rf_schema":SCHEMA,"rf_blocks":[
            {
                "rf_id":"pat1",
                "rf_type":"pattern",
                "rf_ref":"banniere-contact",
                "rf_caption":"Bannière de contact",
                "rf_items":["Contenu du motif **synchronisé**"]
            }
        ]});
        let (_, html) = project("", &pattern_doc).unwrap();
        assert!(html.contains("class=\"rf-synced-pattern\""));
        assert!(html.contains("data-pattern=\"banniere-contact\""));
        assert!(html.contains("data-pattern-title=\"Bannière de contact\""));
        assert!(html.contains("<strong>synchronisé</strong>"));

        // Validation d'erreur sur Pattern (ref vide ou caractères interdits)
        let invalid_pat = json!({"rf_schema":SCHEMA,"rf_blocks":[
            {"rf_id":"bad_pat","rf_type":"pattern","rf_ref":""}
        ]});
        assert!(project("", &invalid_pat).is_err());

        let invalid_pat_chars = json!({"rf_schema":SCHEMA,"rf_blocks":[
            {"rf_id":"bad_pat2","rf_type":"pattern","rf_ref":"invalid/ref<script>"}
        ]});
        assert!(project("", &invalid_pat_chars).is_err());
    }
    #[test]
    fn image_and_button_blocks_validate_urls_and_render_safely() {
        let media_id = Uuid::new_v4();
        let doc = json!({"rf_schema":SCHEMA,"rf_blocks":[
            {"rf_id":"img1","rf_type":"image","rf_url":"/uploads/2026/10/hero.webp","rf_alt":"Aperçu [test]","rf_caption":"Légende *sûre*","rf_media_id":media_id},
            {"rf_id":"btn1","rf_type":"button","rf_text":"Découvrir l’offre","rf_url":"/contact"}
        ]});
        let (body, html) = project("", &doc).unwrap();
        assert!(body.contains("![Aperçu \\[test\\]](/uploads/2026/10/hero.webp)"));
        assert!(html.contains("<img src=\"/uploads/2026/10/hero.webp\" alt=\"Aperçu [test]\""));
        assert!(html.contains("href=\"/contact\"") && html.contains(">Découvrir l’offre</a>"));

        for unsafe_doc in [
            json!({"rf_schema":SCHEMA,"rf_blocks":[{"rf_id":"bad1","rf_type":"image","rf_url":"javascript:alert(1)"}]}),
            json!({"rf_schema":SCHEMA,"rf_blocks":[{"rf_id":"bad2","rf_type":"image","rf_url":"//evil.example/x.png"}]}),
            json!({"rf_schema":SCHEMA,"rf_blocks":[{"rf_id":"bad3","rf_type":"image","rf_url":"/uploads/../secret.png"}]}),
            json!({"rf_schema":SCHEMA,"rf_blocks":[{"rf_id":"bad4","rf_type":"button","rf_text":"Clic","rf_url":"javascript:alert(1)"}]}),
            json!({"rf_schema":SCHEMA,"rf_blocks":[{"rf_id":"bad5","rf_type":"button","rf_text":"","rf_url":"/ok"}]}),
            json!({"rf_schema":SCHEMA,"rf_blocks":[{"rf_id":"bad6","rf_type":"paragraph","rf_text":"ok","rf_url":"/hidden"}]}),
        ] {
            assert!(project("", &unsafe_doc).is_err());
        }
    }
    #[test]
    fn markdown_converts_into_validated_native_document() {
        let md = "# Titre principal\n\nUn paragraphe simple.\n\n> Citation ligne 1\n> Citation ligne 2\n\n- Point A\n- Point B\n\n![Photo](/uploads/photo.webp)\n*Ma légende*\n\n[En savoir plus](https://rustforge.test/doc)\n\n---\n\n```\nfn main() {}\n```";
        let doc = from_markdown(md).unwrap();
        let blocks = doc["rf_blocks"].as_array().unwrap();
        assert_eq!(blocks.len(), 8);
        assert_eq!(blocks[0]["rf_type"], "heading");
        assert_eq!(blocks[0]["rf_level"], 1);
        assert_eq!(blocks[1]["rf_type"], "paragraph");
        assert_eq!(blocks[2]["rf_type"], "quote");
        assert_eq!(blocks[3]["rf_type"], "list");
        assert_eq!(blocks[4]["rf_type"], "image");
        assert_eq!(blocks[4]["rf_caption"], "Ma légende");
        assert_eq!(blocks[5]["rf_type"], "button");
        assert_eq!(blocks[6]["rf_type"], "divider");
        assert_eq!(blocks[7]["rf_type"], "code");
        let (_, html) = project("", &doc).unwrap();
        assert!(html.contains("<h1>Titre principal</h1>"));
        assert!(html.contains("<img src=\"/uploads/photo.webp\" alt=\"Photo\""));
    }
    #[test]
    fn unknown_source_blocks_remain_opaque_and_native_errors_do_not_silently_drop_data() {
        let legacy = json!([{"type":"unknown","raw":"à conserver"}]);
        assert_eq!(project("# Ancien", &legacy).unwrap().0, "# Ancien");
        for doc in [
            json!({"rf_schema":"rf.document/2","rf_blocks":[]}),
            json!({"rf_schema":SCHEMA,"rf_blocks":[{"rf_id":"a","rf_type":"unknown"}]}),
            json!({"rf_schema":SCHEMA,"rf_blocks":[{"rf_id":"a","rf_type":"paragraph"},{"rf_id":"a","rf_type":"paragraph"}]}),
            json!({"rf_schema":SCHEMA,"rf_blocks":[{"rf_id":"a","rf_type":"heading","rf_level":9}]}),
            json!({"rf_schema":SCHEMA,"rf_blocks":[],"unsupported":"do not discard"}),
        ] {
            assert!(project("", &doc).is_err());
        }
    }
    #[test]
    fn text_only_clients_must_not_silently_overwrite_native_documents() {
        let doc = json!({"rf_schema":SCHEMA,"rf_blocks":[]});
        assert!(validate_source_patch(&doc, "ancien", &json!({"body":"nouveau"})).is_err());
        assert!(
            validate_source_patch(&doc, "ancien", &json!({"body":"nouveau","blocks":null}))
                .is_err()
        );
        assert!(validate_source_patch(&doc, "ancien", &json!({"title":"autre titre"})).is_ok());
        assert!(
            validate_source_patch(&doc, "ancien", &json!({"body":"nouveau","blocks":[]})).is_ok()
        );
    }
    #[test]
    fn literal_entities_are_not_interpreted_and_irrelevant_data_is_not_hidden() {
        let doc = serde_json::json!({"rf_schema":SCHEMA,"rf_blocks":[{"rf_id":"p","rf_type":"paragraph","rf_text":"&copy;"}]});
        assert!(project("", &doc).unwrap().1.contains("&amp;copy;"));
        let invalid = serde_json::json!({"rf_schema":SCHEMA,"rf_blocks":[{"rf_id":"l","rf_type":"list","rf_text":"Texte caché","rf_items":["visible"]}]});
        assert!(project("", &invalid).is_err());
    }
    #[test]
    fn test_quote_citation_and_button_styles() {
        // Quote sans citation (WP-B088)
        let q_simple = json!({"rf_schema":SCHEMA,"rf_blocks":[
            {"rf_id":"q1","rf_type":"quote","rf_text":"Une pensée simple"}
        ]});
        let (_, html) = project("", &q_simple).unwrap();
        assert!(html.contains("<blockquote>"));
        assert!(html.contains("Une pensée simple"));

        // Quote avec auteur / citation (WP-B088)
        let q_author = json!({"rf_schema":SCHEMA,"rf_blocks":[
            {"rf_id":"q2","rf_type":"quote","rf_text":"La vie est belle","rf_caption":"Victor Hugo"}
        ]});
        let (body, html) = project("", &q_author).unwrap();
        assert!(body.contains("Victor Hugo"));
        assert!(html.contains("Victor Hugo"));

        // Button avec style (WP-B010)
        let btn_doc = json!({"rf_schema":SCHEMA,"rf_blocks":[
            {"rf_id":"btn1","rf_type":"button","rf_text":"Télécharger","rf_url":"https://rustforge.dev","rf_style":"primary"}
        ]});
        let (body, html) = project("", &btn_doc).unwrap();
        assert!(html.contains("Télécharger"));
        assert!(html.contains("https://rustforge.dev"));
        assert!(html.contains("rf-btn rf-btn-primary"));
        // Roundtrip from_markdown pour quote avec citation et button avec style
        let roundtrip_doc = from_markdown(&format!("{body}\n\n> Sagesse antique\n>\n> *— Marc Aurèle*")).unwrap();
        let rt_blocks = roundtrip_doc["rf_blocks"].as_array().unwrap();
        assert_eq!(rt_blocks.len(), 2);
        assert_eq!(rt_blocks[0]["rf_type"], "button");
        assert_eq!(rt_blocks[0]["rf_style"], "primary");
        assert_eq!(rt_blocks[1]["rf_type"], "quote");
        assert_eq!(rt_blocks[1]["rf_text"], "Sagesse antique");
        assert_eq!(rt_blocks[1]["rf_caption"], "Marc Aurèle");

        // Séparateur (WP-B092)
        let div_doc = json!({"rf_schema":SCHEMA,"rf_blocks":[
            {"rf_id":"sep1","rf_type":"divider"}
        ]});
        let (body, html) = project("", &div_doc).unwrap();
        assert_eq!(body, "---");
        assert!(html.contains("<hr"));

        // Liste à puces et ordonnée (WP-B045)
        let list_doc = json!({"rf_schema":SCHEMA,"rf_blocks":[
            {"rf_id":"l1","rf_type":"list","rf_ordered":true,"rf_items":["Étape A","Étape B"]}
        ]});
        let (_, html) = project("", &list_doc).unwrap();
        assert!(html.contains("<ol>"));
        assert!(html.contains("<li>Étape A</li>"));
        assert!(html.contains("<li>Étape B</li>"));
    }
    #[test]
    fn test_phase1_structural_blocks() {
        // Table block (WP-B099)
        let table_doc = json!({"rf_schema":SCHEMA,"rf_blocks":[
            {
                "rf_id":"t1",
                "rf_type":"table",
                "rf_items":["Produit | Prix | Stock", "RustForge | Gratuit | Illimité", "Support | Inclus | 24/7"],
                "rf_caption":"Tarifs 2026"
            }
        ]});
        let (body, html) = project("", &table_doc).unwrap();
        assert!(body.contains("| Produit | Prix | Stock |"));
        assert!(body.contains("Tarifs 2026"));
        assert!(html.contains("<table>"));
        assert!(html.contains("<th>Produit</th>"));
        assert!(html.contains("<td>RustForge</td>"));
        assert!(html.contains("Tarifs 2026"));

        // Spacer block (WP-B095)
        let spacer_doc = json!({"rf_schema":SCHEMA,"rf_blocks":[
            {"rf_id":"sp1","rf_type":"spacer","rf_style":"lg"}
        ]});
        let (_, html) = project("", &spacer_doc).unwrap();
        assert!(html.contains("class=\"rf-spacer rf-spacer-lg\""));
        assert!(html.contains("aria-hidden=\"true\""));

        // Cover block (WP-B025)
        let cover_doc = json!({"rf_schema":SCHEMA,"rf_blocks":[
            {
                "rf_id":"cov1",
                "rf_type":"cover",
                "rf_url":"/uploads/banner.webp",
                "rf_alt":"Bannière d'accueil",
                "rf_text":"# Titre du Hero\n\nSous-titre inspirant",
                "rf_style":"gradient"
            }
        ]});
        let (_, html) = project("", &cover_doc).unwrap();
        assert!(html.contains("rf-cover rf-cover-gradient"));
        assert!(html.contains("/uploads/banner.webp"));
        assert!(html.contains("Titre du Hero"));

        // Group block (WP-B039)
        let group_doc = json!({"rf_schema":SCHEMA,"rf_blocks":[
            {
                "rf_id":"grp1",
                "rf_type":"group",
                "rf_text":"Texte dans une carte encadrée avec **gras**.",
                "rf_style":"card"
            }
        ]});
        let (_, html) = project("", &group_doc).unwrap();
        assert!(html.contains("rf-group rf-group-card"));
        assert!(html.contains("<strong>gras</strong>"));

        // Roundtrip from_markdown pour Table et Spacer
        let md_source = "| Nom | Version |\n| --- | --- |\n| RustForge | 1.0 |\n\n*Tableau : Composants*\n\n<div class=\"rf-spacer rf-spacer-xl\" aria-hidden=\"true\"></div>";
        let parsed = from_markdown(md_source).unwrap();
        let p_blocks = parsed["rf_blocks"].as_array().unwrap();
        assert_eq!(p_blocks.len(), 2);
        assert_eq!(p_blocks[0]["rf_type"], "table");
        assert_eq!(p_blocks[0]["rf_caption"], "Composants");
        assert_eq!(p_blocks[1]["rf_type"], "spacer");
        assert_eq!(p_blocks[1]["rf_style"], "xl");
    }

    #[test]
    fn test_product_block() {
        let prod_doc = json!({"rf_schema":SCHEMA,"rf_blocks":[
            {
                "rf_id":"prod1",
                "rf_type":"product",
                "rf_text":"Livre Rust & Systèmes Souverains",
                "rf_caption":"39,00 €",
                "rf_ref":"prod-uuid-1234",
                "rf_url":"https://example.com/checkout?product=prod-uuid-1234",
                "rf_alt":"Acheter le guide",
                "rf_style":"card"
            }
        ]});
        let (body, html) = project("", &prod_doc).unwrap();
        assert!(body.contains("rf-product-card rf-product-card"));
        assert!(body.contains("data-product-id=\"prod-uuid-1234\""));
        assert!(body.contains("Livre Rust &amp; Systèmes Souverains"));
        assert!(body.contains("39,00 €"));
        assert!(body.contains("Acheter le guide"));
        assert!(html.contains("rf-product-card"));
        assert!(html.contains("data-product-id=\"prod-uuid-1234\""));

        // Roundtrip from_markdown
        let parsed = from_markdown(&body).unwrap();
        let blocks = parsed["rf_blocks"].as_array().unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0]["rf_type"], "product");
        assert_eq!(blocks[0]["rf_text"], "Livre Rust & Systèmes Souverains");
        assert_eq!(blocks[0]["rf_caption"], "39,00 €");
        assert_eq!(blocks[0]["rf_ref"], "prod-uuid-1234");
        assert_eq!(blocks[0]["rf_alt"], "Acheter le guide");
        assert_eq!(blocks[0]["rf_style"], "card");
    }

    #[test]
    fn test_course_block() {
        let course_doc = json!({"rf_schema":SCHEMA,"rf_blocks":[
            {
                "rf_id":"crs1",
                "rf_type":"course",
                "rf_text":"Maîtriser Rust & Axum de A à Z",
                "rf_caption":"24 leçons • 8h30",
                "rf_ref":"course-uuid-4567",
                "rf_url":"https://example.com/formations/rust-axum",
                "rf_alt":"Commencer le cours",
                "rf_style":"card"
            }
        ]});
        let (body, html) = project("", &course_doc).unwrap();
        assert!(body.contains("rf-course-card rf-course-card"));
        assert!(body.contains("data-course-id=\"course-uuid-4567\""));
        assert!(body.contains("Maîtriser Rust &amp; Axum de A à Z"));
        assert!(body.contains("24 leçons • 8h30"));
        assert!(body.contains("Commencer le cours"));
        assert!(html.contains("rf-course-card"));
        assert!(html.contains("data-course-id=\"course-uuid-4567\""));

        // Roundtrip from_markdown
        let parsed = from_markdown(&body).unwrap();
        let blocks = parsed["rf_blocks"].as_array().unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0]["rf_type"], "course");
        assert_eq!(blocks[0]["rf_text"], "Maîtriser Rust & Axum de A à Z");
        assert_eq!(blocks[0]["rf_caption"], "24 leçons • 8h30");
        assert_eq!(blocks[0]["rf_ref"], "course-uuid-4567");
        assert_eq!(blocks[0]["rf_alt"], "Commencer le cours");
        assert_eq!(blocks[0]["rf_style"], "card");
    }
}
