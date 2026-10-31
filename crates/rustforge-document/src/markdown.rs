//! Rendu Markdown → HTML assaini.

use pulldown_cmark::{Options, Parser, html};

/// Convertit du Markdown (CommonMark + GFM) en HTML, puis l'assainit
/// pour neutraliser tout script injecté par un auteur.
/// Enrichit automatiquement les balises `<img>` avec `loading="lazy"` et `decoding="async"` (WP-41).
pub fn render(markdown: &str) -> String {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);
    opts.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(markdown, opts);
    let mut raw = String::with_capacity(markdown.len() * 3 / 2);
    html::push_html(&mut raw, parser);

    // Injection automatique de chargement différé et décodage asynchrone (WP-41)
    let enriched = if raw.contains("<img ") && !raw.contains("loading=") {
        let mut out = String::with_capacity(raw.len() + 64);
        let mut rest = raw.as_str();
        while let Some(start) = rest.find("<img ") {
            out.push_str(&rest[..start]);
            let img_part = &rest[start..];
            if let Some(end) = img_part.find("/>") {
                let img_content = &img_part[..end];
                out.push_str(img_content);
                out.push_str(" loading=\"lazy\" decoding=\"async\" />");
                rest = &img_part[end + 2..];
            } else if let Some(end) = img_part.find('>') {
                let img_content = &img_part[..end];
                out.push_str(img_content);
                out.push_str(" loading=\"lazy\" decoding=\"async\">");
                rest = &img_part[end + 1..];
            } else {
                out.push_str(img_part);
                rest = "";
            }
        }
        out.push_str(rest);
        out
    } else {
        raw
    };

    sanitize_html(&enriched)
}

/// One sanitizer for stored historical HTML and newly projected documents.
/// Retains the document's structural classes while stripping active HTML and inline styles.
pub fn sanitize_html(html: &str) -> String {
    ammonia::Builder::default()
        .add_generic_attributes([
            "id",
            "class",
            "data-pattern",
            "data-rf-form",
            "data-pattern-title",
            "aria-hidden",
            "data-product-id",
            "data-course-id",
            "data-gallery",
            "data-poster",
        ])
        .add_tags([
            "input",
            "details",
            "summary",
            "div",
            "span",
            "figure",
            "figcaption",
            "video",
            "audio",
            "source",
            "track",
        ])
        .add_tag_attributes("input", ["type", "checked", "disabled"])
        .add_tag_attributes("details", ["open"])
        .add_tag_attributes(
            "video",
            [
                "src",
                "controls",
                "preload",
                "poster",
                "width",
                "height",
                "loop",
                "muted",
                "playsinline",
            ],
        )
        .add_tag_attributes("audio", ["src", "controls", "preload", "loop", "muted"])
        .add_tag_attributes("source", ["src", "type"])
        .add_tag_attributes("track", ["src", "kind", "srclang", "label", "default"])
        .add_tag_attributes("a", ["download"])
        .add_tag_attributes(
            "img",
            [
                "src", "alt", "title", "width", "height", "srcset", "sizes", "loading", "decoding",
            ],
        )
        .clean(html)
        .to_string()
}

/// Produit un extrait texte brut à partir du Markdown.
pub fn excerpt(markdown: &str, max_chars: usize) -> String {
    let parser = Parser::new(markdown);
    let mut text = String::new();
    for event in parser {
        match event {
            pulldown_cmark::Event::Text(t) | pulldown_cmark::Event::Code(t) => text.push_str(&t),
            pulldown_cmark::Event::SoftBreak
            | pulldown_cmark::Event::HardBreak
            | pulldown_cmark::Event::End(_) => text.push(' '),
            _ => {}
        }
    }
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() <= max_chars {
        return collapsed;
    }
    let cut: String = collapsed.chars().take(max_chars).collect();
    match cut.rfind(' ') {
        Some(i) => format!("{}…", &cut[..i]),
        None => format!("{cut}…"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_and_sanitizes() {
        let html = render("# Titre\n\n**gras** <script>alert(1)</script>");
        assert!(html.contains("<h1>Titre</h1>"));
        assert!(html.contains("<strong>gras</strong>"));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn strips_untrusted_styles_but_preserves_block_classes() {
        let html = render("<div class=\"rf-columns\" style=\"position:fixed;inset:0\">Texte</div>");
        assert!(html.contains("class=\"rf-columns\""));
        assert!(!html.contains("style="));
        assert!(!html.contains("position:fixed"));
    }

    #[test]
    fn builds_excerpt() {
        assert_eq!(excerpt("# Hello\n\nworld *fine*", 100), "Hello world fine");
        assert_eq!(excerpt("un deux trois quatre", 10), "un deux…");
    }

    #[test]
    fn test_responsive_images_lazy_loading_and_srcset() {
        let md = "![Illustration](/uploads/photo.webp)";
        let html = render(md);
        assert!(html.contains("loading=\"lazy\""));
        assert!(html.contains("decoding=\"async\""));
        assert!(html.contains("<img src=\"/uploads/photo.webp\" alt=\"Illustration\""));

        // Vérification de la préservation de srcset et sizes personnalisés (WP-41)
        let custom_img = "<img src=\"/uploads/img.webp\" srcset=\"/uploads/img-300.webp 300w, /uploads/img-600.webp 600w\" sizes=\"(max-width: 600px) 100vw, 600px\" alt=\"Adaptatif\" />";
        let cleaned = render(custom_img);
        assert!(cleaned.contains("srcset="));
        assert!(cleaned.contains("sizes="));
    }
}

#[cfg(test)]
mod stored_html_tests {
    #[test]
    fn read_sanitization_keeps_native_layout_without_reviving_active_html() {
        let html = super::sanitize_html(
            r#"<div class="rf-columns"><div class="rf-column" style="position:fixed" onclick="alert(1)">Texte</div></div><script>alert(1)</script><a href="javascript:alert(1)">Lien</a><video controls src="/uploads/demo.mp4"></video>"#,
        );
        assert!(html.contains("class=\"rf-columns\""));
        assert!(html.contains("class=\"rf-column\""));
        assert!(html.contains("<video"));
        for forbidden in ["<script", "onclick", "style=", "javascript:"] {
            assert!(!html.contains(forbidden));
        }
    }
}
