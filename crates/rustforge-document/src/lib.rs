//! Contrat documentaire commun au serveur natif et au Studio WASM. Aucune DB ni HTTP.
pub mod compositions;
pub mod document;
pub mod markdown;
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Conflict(String),
}
pub type CoreResult<T> = Result<T, CoreError>;
impl CoreError {
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }
}
#[derive(Debug, serde::Serialize, PartialEq)]
pub struct Metrics {
    pub words: usize,
    pub characters: usize,
    pub reading_minutes: usize,
}
pub fn metrics(text: &str) -> Metrics {
    let words = text.split_whitespace().count();
    Metrics {
        words,
        characters: text.chars().count(),
        reading_minutes: words.div_ceil(200),
    }
}
/// Texte visible : ignore syntaxe Markdown, destinations de liens et HTML brut.
pub fn visible_metrics(markdown: &str) -> Metrics {
    use pulldown_cmark::{Event, Parser};
    let mut text = String::new();
    for event in Parser::new(markdown) {
        match event {
            Event::Text(value) | Event::Code(value) => {
                text.push_str(&value);
            }
            Event::SoftBreak | Event::HardBreak | Event::End(_) => text.push(' '),
            _ => {}
        }
    }
    metrics(text.trim())
}
#[cfg(test)]
mod tests {
    #[test]
    fn unicode_metrics() {
        assert_eq!(
            super::metrics("été 🌍"),
            super::Metrics {
                words: 2,
                characters: 5,
                reading_minutes: 1
            }
        );
        assert_eq!(super::metrics("").reading_minutes, 0);
    }
}

#[cfg(test)]
mod visible_tests {
    #[test]
    fn markup_is_not_words() {
        let result =
            super::visible_metrics("# Bonjour **monde**\n\n[Lire](https://example.test/secret)");
        assert_eq!(result.words, 3);
        assert_eq!(super::visible_metrics("").reading_minutes, 0);
    }
}
