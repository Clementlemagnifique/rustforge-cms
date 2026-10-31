//! Convertisseur déclaratif de thèmes WordPress Full Site Editing (FSE) vers RustForge.
//! Cas A : Transmutation des design tokens de theme.json et des templates HTML Gutenberg vers MiniJinja 3.

use serde_json::Value;
use std::collections::HashMap;
use std::io::Cursor;
use zip::ZipArchive;

use crate::error::CoreError;

/// Résultat d'une conversion de thème FSE.
#[derive(Debug, Clone)]
pub struct ConvertedTheme {
    pub slug: String,
    pub name: String,
    pub theme_json_content: String,
    pub templates: HashMap<String, String>, // ex: "base.html", "home.html", "single.html"
}

pub struct FseThemeConverter;

impl FseThemeConverter {
    /// Convertit une archive ZIP de thème FSE en structures de thème RustForge.
    pub fn convert_zip(
        bytes: &[u8],
        target_slug: Option<&str>,
    ) -> Result<ConvertedTheme, CoreError> {
        let cursor = Cursor::new(bytes);
        let mut archive = ZipArchive::new(cursor).map_err(|e| {
            CoreError::Validation(format!("Erreur d'ouverture de l'archive ZIP: {}", e))
        })?;

        crate::archive::validate(&mut archive)?;
        let mut theme_json_raw: Option<String> = None;
        let mut raw_templates: HashMap<String, String> = HashMap::new();
        let mut theme_name = "Thème Converti".to_string();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| {
                CoreError::Validation(format!("Erreur de lecture du fichier ZIP: {}", e))
            })?;

            let name = file.name().to_string();
            let name_lower = name.to_lowercase();

            // 1. Extraction theme.json
            if name_lower.ends_with("theme.json") && theme_json_raw.is_none() {
                let content = crate::archive::read_text(&mut file)?;
                {
                    theme_json_raw = Some(content);
                }
            }

            // 2. Extraction templates FSE
            if name_lower.contains("templates/") && name_lower.ends_with(".html") {
                let template_key = name
                    .split('/')
                    .next_back()
                    .unwrap_or("index.html")
                    .to_string();
                let content = crate::archive::read_text(&mut file)?;
                {
                    raw_templates.insert(template_key, content);
                }
            }

            // 3. Extraction Nom dans style.css
            if (name_lower == "style.css" || name_lower.ends_with("/style.css"))
                && theme_name == "Thème Converti"
            {
                let content = crate::archive::read_text(&mut file)?;
                {
                    for line in content.lines() {
                        let trimmed = line.trim_start_matches(['*', '/', ' ']);
                        if let Some(n) = trimmed.strip_prefix("Theme Name:") {
                            let val = n.trim();
                            if !val.is_empty() {
                                theme_name = val.to_string();
                                break;
                            }
                        }
                    }
                }
            }
        }

        let slug = target_slug
            .map(|s| s.to_string())
            .unwrap_or_else(|| slug::slugify(&theme_name));
        crate::security::safe_slug(&slug)?;

        // 4. Transmutation des Design Tokens vers RustForge theme.json
        let (tp_tokens, primary_color, accent_color, bg_color, text_color, font_heading, font_body) =
            Self::transmute_theme_json(theme_json_raw.as_deref());

        let rustforge_theme_json = serde_json::json!({
            "name": theme_name,
            "version": "1.0.0",
            "author": "RustForge Universal Converter",
            "description": format!("Thème converti automatiquement depuis l'archive WordPress FSE '{}'.", theme_name),
            "tokens": tp_tokens
        });

        let theme_json_str = serde_json::to_string_pretty(&rustforge_theme_json).map_err(|e| {
            CoreError::Validation(format!("Erreur de sérialisation theme.json: {}", e))
        })?;

        // 5. Transmutation des templates vers MiniJinja 3
        let mut converted_templates = HashMap::new();

        // Création du template base.html parent si non présent
        let base_html = format!(
            r#"<!DOCTYPE html>
<html lang="{{{{ locale | default("fr") }}}}">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{{% block title %}}{{{{ site_title }}}}{{% endblock %}}</title>
  <style>
    :root {{
      --tp-primary: {primary};
      --tp-accent: {accent};
      --tp-bg: {bg};
      --tp-text: {text};
      --tp-font-heading: {f_heading};
      --tp-font-body: {f_body};
      --tp-radius: 8px;
    }}
    body {{
      margin: 0;
      font-family: var(--tp-font-body);
      background-color: var(--tp-bg);
      color: var(--tp-text);
      line-height: 1.6;
    }}
    header.tp-header {{
      padding: 1.5rem 2rem;
      border-bottom: 1px solid rgba(0,0,0,0.08);
      display: flex;
      justify-content: space-between;
      align-items: center;
    }}
    .tp-site-brand {{
      font-size: 1.25rem;
      font-weight: 700;
      color: var(--tp-primary);
      text-decoration: none;
    }}
    .tp-container {{
      max-width: 900px;
      margin: 2.5rem auto;
      padding: 0 1.5rem;
    }}
    h1, h2, h3 {{
      font-family: var(--tp-font-heading);
      color: var(--tp-primary);
    }}
    .tp-content img {{
      max-width: 100%;
      height: auto;
      border-radius: var(--tp-radius);
    }}
    footer.tp-footer {{
      margin-top: 4rem;
      padding: 2rem;
      text-align: center;
      font-size: 0.875rem;
      opacity: 0.7;
      border-top: 1px solid rgba(0,0,0,0.08);
    }}
  </style>
  {{% block head %}}{{% endblock %}}
</head>
<body>
  <header class="tp-header">
    <a href="/" class="tp-site-brand">{{{{ site_title }}}}</a>
    <nav>
      <a href="/" style="margin-right: 1rem; color: var(--tp-text); text-decoration: none;">Accueil</a>
      {{% if zen_url %}}<a href="{{{{ zen_url }}}}" style="color: var(--tp-accent); font-weight: 600; text-decoration: none;">Mode Zen</a>{{% endif %}}
    </nav>
  </header>
  <main class="tp-container">
    {{% block content %}}{{% endblock %}}
  </main>
  <footer class="tp-footer">
    <p>&copy; {{{{ site_title }}}} — Propulsé par <a href="https://rustforge.dev" style="color: var(--tp-primary); font-weight: 600;">RustForge</a> (Rust SSR &lt; 1 ms)</p>
  </footer>
</body>
</html>"#,
            primary = primary_color,
            accent = accent_color,
            bg = bg_color,
            text = text_color,
            f_heading = font_heading,
            f_body = font_body
        );

        converted_templates.insert("base.html".to_string(), base_html);

        // Transmutation des templates individuels
        for (tpl_name, raw_html) in &raw_templates {
            let jinja_body = Self::transmute_gutenberg_to_minijinja(raw_html);
            let final_template = format!(
                r#"{{% extends "base.html" %}}

{{% block title %}}{{{{ entry.title | default(site_title) }}}} — {{{{ site_title }}}}{{% endblock %}}

{{% block content %}}
<article class="tp-converted-article">
{}
</article>
{{% endblock %}}
"#,
                jinja_body
            );

            // Mappage des noms standards FSE
            let target_name = match tpl_name.as_str() {
                "index.html" | "home.html" | "front-page.html" => "home.html".to_string(),
                "single.html" | "page.html" => "single.html".to_string(),
                other => other.to_string(),
            };

            converted_templates.insert(target_name, final_template);
        }

        // S'il n'y avait pas de template single.html spécifique, en créer un par défaut
        if !converted_templates.contains_key("single.html") {
            let default_single = r#"{% extends "base.html" %}

{% block title %}{{ entry.title }} — {{ site_title }}{% endblock %}

{% block content %}
<article class="tp-single-entry">
  <header style="margin-bottom: 2rem;">
    <h1 style="font-size: 2.5rem; margin-bottom: 0.5rem;">{{ entry.title }}</h1>
    <div style="font-size: 0.9rem; opacity: 0.7;">
      Publié le {{ entry.published_at | default("Récemment") }} par {{ entry.author_name | default("Admin") }}
    </div>
  </header>
  {% if entry.featured_media_url %}
    <img src="{{ entry.featured_media_url }}" alt="{{ entry.title }}" style="width: 100%; max-height: 400px; object-fit: cover; border-radius: var(--tp-radius); margin-bottom: 2rem;" />
  {% endif %}
  <div class="tp-content" style="font-size: 1.125rem;">
    {{ entry.body_html | safe }}
  </div>
</article>
{% endblock %}
"#.to_string();
            converted_templates.insert("single.html".to_string(), default_single);
        }

        Ok(ConvertedTheme {
            slug,
            name: theme_name,
            theme_json_content: theme_json_str,
            templates: converted_templates,
        })
    }

    /// Transmute theme.json de WordPress vers les tokens de RustForge.
    fn transmute_theme_json(
        raw_json: Option<&str>,
    ) -> (Value, String, String, String, String, String, String) {
        let mut primary = "#1e3a8a".to_string(); // bleu profond par défaut
        let mut accent = "#3b82f6".to_string(); // bleu vif
        let mut bg = "#ffffff".to_string(); // blanc
        let mut text = "#111827".to_string(); // gris sombre
        let mut font_heading = "system-ui, sans-serif".to_string();
        let mut font_body = "system-ui, sans-serif".to_string();

        if let Some(json_str) = raw_json
            && let Ok(v) = serde_json::from_str::<Value>(json_str)
        {
            // Lecture de la palette de couleurs
            if let Some(palette) = v
                .pointer("/settings/color/palette")
                .and_then(|p| p.as_array())
            {
                for item in palette {
                    let slug = item.get("slug").and_then(|s| s.as_str()).unwrap_or("");
                    let color = item.get("color").and_then(|c| c.as_str()).unwrap_or("");
                    if !color.is_empty() {
                        match slug {
                            "primary" | "base" | "contrast" => primary = color.to_string(),
                            "secondary" | "accent" | "highlight" => accent = color.to_string(),
                            "background" | "surface" => bg = color.to_string(),
                            "foreground" | "text" | "body" => text = color.to_string(),
                            _ => {}
                        }
                    }
                }
            }

            // Lecture des familles de police
            if let Some(fonts) = v
                .pointer("/settings/typography/fontFamilies")
                .and_then(|f| f.as_array())
            {
                for item in fonts {
                    let slug = item.get("slug").and_then(|s| s.as_str()).unwrap_or("");
                    let family = item
                        .get("fontFamily")
                        .and_then(|f| f.as_str())
                        .unwrap_or("");
                    if !family.is_empty() {
                        if slug.contains("heading") || slug.contains("title") {
                            font_heading = family.to_string();
                        } else if slug.contains("body") || slug.contains("text") {
                            font_body = family.to_string();
                        }
                    }
                }
            }
        }

        let tokens = serde_json::json!({
            "color_primary": primary,
            "color_accent": accent,
            "color_background": bg,
            "color_text": text,
            "font_heading": font_heading,
            "font_body": font_body,
            "border_radius": "8px"
        });

        (tokens, primary, accent, bg, text, font_heading, font_body)
    }

    /// Convertit les blocs souverains (rf:*) et commentaires historiques (wp:*) en expressions MiniJinja sémantiques.
    fn transmute_gutenberg_to_minijinja(html: &str) -> String {
        let mut result = String::with_capacity(html.len());

        // Nettoyage ligne par ligne des balises de commentaires de blocs
        for line in html.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("<!-- rf:post-title")
                || trimmed.starts_with("<!-- wp:post-title")
            {
                result.push_str("  <h1 class=\"rf-post-title\">{{ entry.title }}</h1>\n");
            } else if trimmed.starts_with("<!-- rf:post-content")
                || trimmed.starts_with("<!-- wp:post-content")
            {
                result.push_str(
                    "  <div class=\"rf-post-content\">{{ entry.body_html | safe }}</div>\n",
                );
            } else if trimmed.starts_with("<!-- rf:site-title")
                || trimmed.starts_with("<!-- wp:site-title")
            {
                result.push_str("  <a href=\"/\" class=\"rf-site-brand\">{{ site_title }}</a>\n");
            } else if trimmed.starts_with("<!-- rf:post-date")
                || trimmed.starts_with("<!-- wp:post-date")
            {
                result.push_str("  <time class=\"rf-date\">{{ entry.published_at }}</time>\n");
            } else if trimmed.starts_with("<!-- rf:post-author")
                || trimmed.starts_with("<!-- wp:post-author")
            {
                result.push_str("  <span class=\"rf-author\">{{ entry.author_name }}</span>\n");
            } else if trimmed.starts_with("<!-- rf:post-featured-image")
                || trimmed.starts_with("<!-- wp:post-featured-image")
            {
                result.push_str("  {% if entry.featured_media_url %}<img src=\"{{ entry.featured_media_url }}\" alt=\"{{ entry.title }}\" class=\"rf-featured-image\" />{% endif %}\n");
            } else if trimmed.starts_with("<!-- rf:")
                || trimmed.starts_with("<!-- /rf:")
                || trimmed.starts_with("<!-- wp:")
                || trimmed.starts_with("<!-- /wp:")
            {
                // Ignore les commentaires de blocs structurels (rf:group, wp:group, rf:columns, etc.)
                continue;
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }

        result
    }
}
