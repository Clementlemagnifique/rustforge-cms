//! Shared hierarchy rules for page and taxonomy pickers; the server remains authoritative.
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
pub fn descendants(rows: &[Value], id: &str) -> BTreeSet<String> {
    let mut hidden = BTreeSet::from([id.to_owned()]);
    loop {
        let before = hidden.len();
        for row in rows {
            if row["parent_id"]
                .as_str()
                .is_some_and(|parent| hidden.contains(parent))
                && let Some(id) = row["id"].as_str()
            {
                hidden.insert(id.into());
            }
        }
        if before == hidden.len() {
            break;
        }
    }
    hidden
}
pub fn options(rows: &[Value], excluded: Option<&str>) -> Vec<(String, String)> {
    let hidden = excluded.map(|id| descendants(rows, id)).unwrap_or_default();
    let by_id: BTreeMap<_, _> = rows
        .iter()
        .filter_map(|row| {
            row["id"]
                .as_str()
                .filter(|id| !hidden.contains(*id))
                .map(|id| (id, row))
        })
        .collect();
    let mut children: BTreeMap<Option<&str>, Vec<&str>> = BTreeMap::new();
    for (&id, row) in &by_id {
        let parent = row["parent_id"]
            .as_str()
            .filter(|parent| by_id.contains_key(parent));
        children.entry(parent).or_default().push(id);
    }
    for siblings in children.values_mut() {
        siblings.sort_by_key(|id| name(by_id[id]).to_lowercase());
    }
    fn visit(
        id: &str,
        depth: usize,
        by_id: &BTreeMap<&str, &Value>,
        children: &BTreeMap<Option<&str>, Vec<&str>>,
        seen: &mut BTreeSet<String>,
        out: &mut Vec<(String, String)>,
    ) {
        if !seen.insert(id.into()) {
            return;
        }
        out.push((
            id.into(),
            format!("{}{}", "— ".repeat(depth), name(by_id[id])),
        ));
        if depth < 32
            && let Some(items) = children.get(&Some(id))
        {
            for child in items {
                visit(child, depth + 1, by_id, children, seen, out);
            }
        }
    }
    let mut out = vec![];
    let mut seen = BTreeSet::new();
    if let Some(roots) = children.get(&None) {
        for id in roots {
            visit(id, 0, &by_id, &children, &mut seen, &mut out);
        }
    }
    // Imported orphan/cyclic records remain visible, without infinite recursion.
    for id in by_id.keys() {
        visit(id, 0, &by_id, &children, &mut seen, &mut out);
    }
    out
}
fn name(row: &Value) -> &str {
    row["title"]
        .as_str()
        .or_else(|| row["name"].as_str())
        .unwrap_or("—")
}
#[cfg(all(target_arch = "wasm32", feature = "ui"))]
pub mod ui;
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn descendants_and_cycles_never_become_parent_choices() {
        let rows = vec![
            json!({"id":"a","title":"A"}),
            json!({"id":"b","parent_id":"a","title":"B"}),
            json!({"id":"c","parent_id":"b","title":"C"}),
            json!({"id":"d","title":"D"}),
        ];
        assert_eq!(options(&rows, Some("a")), vec![("d".into(), "D".into())]);
        assert_eq!(options(&rows, None)[2].1, "— — C");
        let cycle = vec![
            json!({"id":"a","parent_id":"b"}),
            json!({"id":"b","parent_id":"a"}),
        ];
        assert!(options(&cycle, Some("a")).is_empty());
        assert_eq!(options(&cycle, None).len(), 2);
    }
}
