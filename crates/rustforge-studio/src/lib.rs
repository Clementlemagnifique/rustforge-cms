//! Phase 0 : pont documentaire. La coquille Leptos et le remplacement du Studio suivent.
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
/// Réponse JSON stable utilisable par le client et les tests natifs.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn inspect_document(body: &str, blocks_json: &str) -> String {
    let response = (|| {
        let blocks: serde_json::Value =
            serde_json::from_str(blocks_json).map_err(|e| e.to_string())?;
        let (projected, html) =
            rustforge_document::document::project(body, &blocks).map_err(|e| e.to_string())?;
        Ok::<_, String>(
            serde_json::json!({"body":projected,"html":html,"metrics":rustforge_document::visible_metrics(&projected)}),
        )
    })();
    match response {
        Ok(value) => serde_json::json!({"ok":true,"data":value}).to_string(),
        Err(error) => serde_json::json!({"ok":false,"error":error}).to_string(),
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn shares_native_validation() {
        let document = r#"{"rf_schema":"rf.document/1","rf_blocks":[{"rf_id":"one","rf_type":"paragraph","rf_text":"Bonjour monde"}]}"#;
        let result: serde_json::Value =
            serde_json::from_str(&super::inspect_document("", document)).unwrap();
        assert_eq!(result["ok"], true);
        assert_eq!(result["data"]["metrics"]["words"], 2);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&super::inspect_document(
                "",
                r#"{"rf_schema":"unknown"}"#
            ))
            .unwrap()["ok"],
            false
        );
    }
}

#[cfg(all(target_arch = "wasm32", feature = "ui"))]
pub mod app;

pub mod navigation;

pub mod dashboard;
pub mod messages;
pub mod session;

#[cfg(all(target_arch = "wasm32", feature = "ui"))]
pub mod shell;

pub mod screens;

pub mod forms;
#[cfg(all(target_arch = "wasm32", feature = "ui"))]
pub mod http;

pub mod editor;

#[cfg(all(target_arch = "wasm32", feature = "ui"))]
pub mod media_ui;

#[cfg(all(target_arch = "wasm32", feature = "ui"))]
pub mod appearance_ui;

pub mod modules;

pub mod block_catalog;

#[cfg(all(target_arch = "wasm32", feature = "ui"))]
pub mod diagnostics_ui;

#[cfg(all(target_arch = "wasm32", feature = "ui"))]
pub mod assistant_ui;

#[cfg(all(target_arch = "wasm32", feature = "ui"))]
pub mod learning_ui;

#[cfg(all(target_arch = "wasm32", feature = "ui"))]
pub mod orders_ui;

#[cfg(all(target_arch = "wasm32", feature = "ui"))]
pub mod comments_ui;

pub mod relations;

#[cfg(all(target_arch = "wasm32", feature = "ui"))]
pub mod taxonomies_ui;

pub mod submission_csv;

pub mod submissions;
