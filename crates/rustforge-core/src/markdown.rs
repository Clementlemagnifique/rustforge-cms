//! Rendu Markdown → HTML assaini.

use pulldown_cmark::{Options, Parser, html};

/// Convertit du Markdown (CommonMark + GFM) en HTML, puis l'assainit
/// pour neutraliser tout script injecté par un auteur.
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

    ammonia::Builder::default()
        .add_generic_attributes(["id", "class"])
        .add_tags(["input", "details", "summary", "div", "span", "figure", "figcaption"])
        .add_tag_attributes("input", ["type", "checked", "disabled"])
        .add_tag_attributes("details", ["open"])
        .clean(&raw)
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
}
