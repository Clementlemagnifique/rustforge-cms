//! Adaptateur domaine : le moteur documentaire est partagé avec le client WASM.
pub use rustforge_document::document::{MAX_BLOCKS, SCHEMA};
fn map_error(error: rustforge_document::CoreError) -> crate::CoreError {
    match error {
        rustforge_document::CoreError::Validation(message) => crate::CoreError::Validation(message),
        rustforge_document::CoreError::Conflict(message) => crate::CoreError::Conflict(message),
    }
}
pub fn project(body: &str, blocks: &serde_json::Value) -> crate::CoreResult<(String, String)> {
    rustforge_document::document::project(body, blocks).map_err(map_error)
}
pub fn from_markdown(body: &str) -> crate::CoreResult<serde_json::Value> {
    rustforge_document::document::from_markdown(body).map_err(map_error)
}
pub fn validate_source_patch(
    blocks: &serde_json::Value,
    body: &str,
    patch: &serde_json::Value,
) -> crate::CoreResult<()> {
    rustforge_document::document::validate_source_patch(blocks, body, patch).map_err(map_error)
}
