//! Static compositions on the shared document contract, usable by every editor consumer.
use crate::{CoreError, CoreResult, document};
use serde_json::{Value, json};

pub const CATALOG: &[(&str, &str)] = &[
    ("hero_cta", "composition.hero"),
    ("columns_2_features", "composition.features"),
    ("faq_accordion", "composition.faq"),
    ("landing", "composition.landing"),
];

pub fn blocks(id: &str, mut new_id: impl FnMut() -> String) -> CoreResult<Vec<Value>> {
    let hero = || {
        vec![
            json!({"rf_type":"heading","rf_text":"Présentez votre offre en une phrase","rf_level":2}),
            json!({"rf_type":"paragraph","rf_text":"Expliquez à qui vous vous adressez et le bénéfice concret de votre proposition."}),
            json!({"rf_type":"button","rf_text":"Nous contacter","rf_url":"mailto:contact@example.invalid","rf_style":"primary"}),
        ]
    };
    let features = || {
        vec![
            json!({"rf_type":"columns","rf_items":["### Premier avantage\nDécrivez un bénéfice concret.","### Deuxième avantage\nAjoutez une preuve ou un exemple."]}),
        ]
    };
    let faq = || {
        vec![
            json!({"rf_type":"heading","rf_text":"Questions fréquentes","rf_level":2}),
            json!({"rf_type":"details","rf_caption":"À qui s’adresse cette offre ?","rf_text":"Précisez le public concerné."}),
            json!({"rf_type":"details","rf_caption":"Comment commencer ?","rf_text":"Décrivez la prochaine étape et les conditions utiles."}),
        ]
    };
    let mut blocks = match id {
        "hero_cta" => hero(),
        "columns_2_features" => features(),
        "faq_accordion" => faq(),
        "landing" => {
            let mut all = hero();
            all.extend(features());
            all.extend(faq());
            all
        }
        _ => return Err(CoreError::validation("Composition inconnue")),
    };
    for block in &mut blocks {
        block["rf_id"] = json!(new_id());
    }
    // Every composition must obey the same contract as saved and rendered content.
    document::project(
        "",
        &json!({"rf_schema":document::SCHEMA,"rf_blocks":blocks}),
    )?;
    Ok(blocks)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compositions_use_valid_shared_blocks_and_fresh_ids() {
        let mut ids = std::collections::HashSet::new();
        let mut next = 0;
        for (id, _) in CATALOG {
            for _ in 0..2 {
                let blocks = blocks(id, || {
                    next += 1;
                    format!("composition-{next}")
                })
                .unwrap();
                assert!(!blocks.is_empty());
                for block in blocks {
                    assert!(ids.insert(block["rf_id"].as_str().unwrap().to_owned()));
                }
            }
        }
        assert!(blocks("unknown", || "id".into()).is_err());
    }
}
