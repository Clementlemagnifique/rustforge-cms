use std::io::{self, Read};
fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let fixtures: Vec<serde_json::Value> = serde_json::from_str(&input).unwrap();
    let results: Vec<serde_json::Value> = fixtures
        .iter()
        .map(|f| {
            serde_json::from_str(&rustforge_studio::inspect_document(
                f["body"].as_str().unwrap_or(""),
                &f["blocks"].to_string(),
            ))
            .unwrap()
        })
        .collect();
    println!("{}", serde_json::to_string(&results).unwrap());
}
