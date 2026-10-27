wit_bindgen::generate!({
    world: "plugin",
    path: "../../wit",
});

use crate::rustforge::plugin::types;

struct ReadingTimePlugin;

impl Guest for ReadingTimePlugin {
    fn manifest() -> types::Manifest {
        types::Manifest {
            id: "reading-time".to_string(),
            name: "Temps de lecture".to_string(),
            version: "0.1.0".to_string(),
            description: "Calcule et affiche le temps estimé de lecture en tête des articles.".to_string(),
            hooks: vec!["content.render".to_string()],
            capabilities: vec![],
        }
    }

    fn filter(hook: String, payload: String) -> Result<String, String> {
        if hook == "content.render" {
            // Calcul simple du nombre de mots (environ 200 mots par minute)
            let word_count = payload.split_whitespace().count();
            let minutes = (word_count / 200).max(1);
            let badge = format!(
                r#"<div class="tp-reading-time" style="display:inline-flex;align-items:center;gap:6px;padding:4px 10px;border-radius:9999px;background:rgba(99,102,241,0.1);color:#818cf8;font-size:0.875rem;margin-bottom:1.5rem;font-weight:500;">⏱️ {minutes} min de lecture</div>"#
            );
            Ok(format!("{badge}\n{payload}"))
        } else {
            Ok(payload)
        }
    }

    fn on_action(_hook: String, _payload: String) {
        // Aucune action requise pour ce plugin
    }
}

export!(ReadingTimePlugin);
