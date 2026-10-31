//! Spreadsheet-safe export of submission fields only; no IP hashes or user agents.
use serde_json::Value;
use std::collections::{BTreeSet, HashSet};

#[derive(Default)]
pub struct Export {
    rows: Vec<Value>,
    ids: HashSet<String>,
    total: Option<u64>,
    bytes: usize,
}
impl Export {
    /// Fail closed on malformed/changing pagination; never download a partial file.
    pub fn append(&mut self, response: &Value, page: u64) -> Result<bool, &'static str> {
        let total = response["total"].as_u64().ok_or("csv.failed")?;
        let pages = response["total_pages"].as_u64().ok_or("csv.failed")?;
        let items = response["items"].as_array().ok_or("csv.failed")?;
        if total > 10_000
            || response["page"].as_u64() != Some(page)
            || response["per_page"].as_u64() != Some(100)
            || pages != total.div_ceil(100)
            || page > pages.max(1)
            || self.total.is_some_and(|old| old != total)
            || items.len() != (total.saturating_sub((page - 1) * 100).min(100) as usize)
        {
            return Err("csv.failed");
        }
        self.total = Some(total);
        for row in items {
            let id = row["id"].as_str().ok_or("csv.failed")?;
            if !self.ids.insert(id.to_owned()) || !row["data"].is_object() {
                return Err("csv.failed");
            }
            self.bytes += row.to_string().len();
            if self.bytes > 10_000_000 {
                return Err("csv.failed");
            }
            self.rows.push(row.clone());
        }
        Ok(self.rows.len() as u64 == total)
    }
    pub fn finish(self) -> Result<String, &'static str> {
        if self.total != Some(self.rows.len() as u64) {
            return Err("csv.failed");
        }
        let keys: BTreeSet<_> = self
            .rows
            .iter()
            .flat_map(|r| r["data"].as_object().unwrap().keys().cloned())
            .collect();
        let mut csv = String::from("\u{feff}");
        let mut header = vec!["ID".to_owned(), "Date".into(), "Statut".into()];
        header.extend(keys.iter().cloned());
        line(&mut csv, &header);
        for row in self.rows {
            let mut cells = vec![
                row["id"].as_str().unwrap_or_default().to_owned(),
                row["created_at"].as_str().unwrap_or_default().to_owned(),
                if row["is_spam"] == true {
                    "Spam"
                } else {
                    "Valide"
                }
                .into(),
            ];
            cells.extend(keys.iter().map(|key| match &row["data"][key] {
                Value::Null => String::new(),
                Value::String(s) => s.clone(),
                value => value.to_string(),
            }));
            line(&mut csv, &cells);
            if csv.len() > 10_000_000 {
                return Err("csv.failed");
            }
        }
        Ok(csv)
    }
}
fn line(output: &mut String, cells: &[String]) {
    for (i, cell) in cells.iter().enumerate() {
        if i != 0 {
            output.push(';');
        }
        output.push('"');
        // Leading whitespace/control characters can hide a formula in spreadsheet imports.
        if cell
            .trim_start_matches(|c: char| c.is_whitespace() || c.is_control() || c == '\u{feff}')
            .starts_with(['=', '+', '-', '@'])
            || cell.starts_with(['\t', '\r', '\n'])
        {
            output.push('\'');
        }
        output.push_str(&cell.replace('"', "\"\""));
        output.push('"');
    }
    output.push_str("\r\n");
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn row(id: usize) -> Value {
        json!({"id":id.to_string(),"created_at":"2026-10-31","data":{"nom":"Élodie;\"bonjour\"\nSuite","formule":" \t=1+1","nested":{"ok":true}},"ip_hash":"SECRET"})
    }
    fn page(items: Vec<Value>, n: u64, total: u64) -> Value {
        json!({"items":items,"page":n,"per_page":100,"total":total,"total_pages":total.div_ceil(100)})
    }
    #[test]
    fn all_pages_and_spreadsheet_escaping() {
        let mut e = Export::default();
        assert!(
            !e.append(&page((0..100).map(row).collect(), 1, 101), 1)
                .unwrap()
        );
        assert!(e.append(&page(vec![row(100)], 2, 101), 2).unwrap());
        let csv = e.finish().unwrap();
        assert!(csv.starts_with('\u{feff}'));
        assert!(csv.contains("Élodie;\"\"bonjour\"\"\nSuite"));
        assert!(csv.contains("' \t=1+1"));
        assert!(csv.contains("{\"\"ok\"\":true}"));
        assert!(!csv.contains("SECRET"));
        assert!(csv.contains("\"100\""));
    }
    #[test]
    fn no_partial_or_changed_export() {
        let mut e = Export::default();
        e.append(&page((0..100).map(row).collect(), 1, 101), 1)
            .unwrap();
        assert!(e.finish().is_err());
        let mut e = Export::default();
        e.append(&page((0..100).map(row).collect(), 1, 101), 1)
            .unwrap();
        assert!(e.append(&page(vec![row(0)], 2, 101), 2).is_err());
        assert!(
            Export::default()
                .append(&page(vec![], 1, 10_001), 1)
                .is_err()
        );
        assert!(Export::default().append(&json!({}), 1).is_err());
    }
    #[test]
    fn empty_and_hostile_headers() {
        let mut e = Export::default();
        assert!(e.append(&page(vec![], 1, 0), 1).unwrap());
        assert_eq!(
            e.finish().unwrap(),
            "\u{feff}\"ID\";\"Date\";\"Statut\"\r\n"
        );
        let mut out = String::new();
        line(
            &mut out,
            &["@SUM(A1)".into(), "\tplain".into(), "a;b".into()],
        );
        assert_eq!(out, "\"'@SUM(A1)\";\"'\tplain\";\"a;b\"\r\n");
    }
}
