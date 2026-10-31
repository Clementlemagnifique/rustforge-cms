//! Inbox query contract shared by the native view and focused tests.
#[cfg(all(target_arch = "wasm32", feature = "ui"))]
mod download;
#[cfg(all(target_arch = "wasm32", feature = "ui"))]
pub mod ui;
/// Encode UTF-8 bytes, keeping a search term inside its one query parameter.
pub fn encode_query(value: &str) -> String {
    let mut out = String::new();
    for b in value.bytes() {
        if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
            out.push(b as char)
        } else {
            out.push_str(&format!("%{b:02X}"))
        }
    }
    out
}
#[cfg(test)]
mod tests {
    #[test]
    fn search_cannot_add_filters_or_routes() {
        assert_eq!(
            super::encode_query("é&trash=true# x"),
            "%C3%A9%26trash%3Dtrue%23%20x"
        );
    }
}
