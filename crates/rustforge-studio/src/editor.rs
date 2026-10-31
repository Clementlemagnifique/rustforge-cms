//! Native editor state. Rendering and conversion delegate to rustforge-document.
use serde_json::{Value, json};
#[derive(Clone)]
pub struct Location {
    pub id: Option<String>,
    pub content_type: String,
}
pub fn local_id(id: &str) -> bool {
    id.len() == 36
        && id.bytes().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_hexdigit()
            }
        })
}
impl Location {
    pub fn parse(route: &str) -> Option<Self> {
        let parts: Vec<_> = route.split(':').collect();
        match parts.as_slice() {
            ["new", kind] if matches!(*kind, "post" | "page") => Some(Self {
                id: None,
                content_type: (*kind).into(),
            }),
            ["editor", id, kind] if local_id(id) && matches!(*kind, "post" | "page") => {
                Some(Self {
                    id: Some((*id).into()),
                    content_type: (*kind).into(),
                })
            }
            _ => None,
        }
    }
}
pub const BLOCKS: &[(&str, &str)] = &[
    ("paragraph", "editor.paragraph"),
    ("heading", "editor.heading"),
    ("quote", "editor.quote"),
    ("code", "editor.code"),
    ("list", "editor.list"),
    ("divider", "editor.divider"),
    ("image", "form.image"),
    ("button", "editor.button"),
    ("callout", "editor.callout"),
    ("details", "editor.details"),
    ("columns", "editor.columns"),
    ("media_text", "editor.media_text"),
    ("pattern", "editor.pattern"),
    ("table", "editor.table"),
    ("spacer", "editor.spacer"),
    ("cover", "editor.cover"),
    ("group", "editor.group"),
    ("product", "editor.product"),
    ("course", "editor.course"),
    ("gallery", "form.gallery"),
    ("video", "form.video"),
    ("audio", "form.audio"),
    ("file", "editor.file"),
    ("form_embed", "blocks.form"),
];
pub fn block(kind: &str, id: String) -> Value {
    let mut block = json!({"rf_id":id,"rf_type":kind,"rf_text":"","rf_items":[],"rf_level":2,"rf_ordered":false});
    match kind {
        "columns" => block["rf_items"] = json!(["", ""]),
        "table" => block["rf_items"] = json!([" | "]),
        "button" => block["rf_style"] = json!("primary"),
        "callout" => block["rf_style"] = json!("info"),
        "spacer" => block["rf_style"] = json!("md"),
        "group" => block["rf_style"] = json!("card"),
        _ => {}
    }
    block
}
pub fn empty(kind: &str) -> Value {
    json!({"title":"","slug":"","excerpt":"","status":"draft","content_type":kind,"body":"","blocks":{"rf_schema":"rf.document/1","rf_blocks":[]}})
}
pub fn projected(value: &Value) -> Result<(String, String), String> {
    rustforge_document::document::project(
        value["body"].as_str().unwrap_or_default(),
        &value["blocks"],
    )
    .map_err(|error| error.to_string())
}
pub fn payload(value: &Value, creating: bool) -> Result<Value, String> {
    let (body, _) = projected(value)?;
    let title = value["title"].as_str().unwrap_or_default().trim();
    if title.is_empty() || title.chars().count() > 300 {
        return Err(crate::messages::text("dashboard.title_invalid").into());
    }
    if let Some(id) = value.get("featured_media_id").filter(|id| !id.is_null())
        && id.as_str().is_none_or(|id| !local_id(id))
    {
        return Err(crate::messages::text("featured.invalid").into());
    }
    let mut result = json!({"title":title,"slug":value["slug"],"excerpt":value["excerpt"],"body":body,"blocks":value["blocks"],"status":value["status"]});
    if !creating {
        result["expected_updated_at"] = value["updated_at"].clone();
    }
    for key in [
        "published_at",
        "scheduled_unpublish_at",
        "featured_media_id",
        "parent_id",
        "term_ids",
    ] {
        if value.get(key).is_some() {
            result[key] = value[key].clone();
        }
    }
    if let Some(fields) = document_settings(value)? {
        result["fields"] = fields;
    }
    Ok(result)
}
/// Only editable reserved settings enter payloads and recovery, never arbitrary private fields.
fn document_settings(value: &Value) -> Result<Option<Value>, String> {
    let mut result = serde_json::Map::new();
    for (key, allowed, message) in [
        (
            "_rf_comment_status",
            &["open", "closed"][..],
            "editor.discussion_invalid",
        ),
        (
            "_rf_template",
            &["default", "wide", "landing"][..],
            "editor.template_invalid",
        ),
    ] {
        if let Some(setting) = value["fields"].get(key) {
            if !setting.is_null() && setting.as_str().is_none_or(|s| !allowed.contains(&s)) {
                return Err(crate::messages::text(message).into());
            }
            result.insert(key.into(), setting.clone());
        }
    }
    Ok((!result.is_empty()).then_some(Value::Object(result)))
}

/// Append without replacing existing content or metadata. Legacy conversion stays explicit.
pub fn append_composition(value: &Value, id: &str) -> Result<Value, String> {
    if value["blocks"]["rf_schema"] != rustforge_document::document::SCHEMA {
        return Err(crate::messages::text("composition.convert_first").into());
    }
    let mut next = value.clone();
    let blocks = rustforge_document::compositions::blocks(id, || uuid::Uuid::new_v4().to_string())
        .map_err(|e| e.to_string())?;
    let current = next["blocks"]["rf_blocks"]
        .as_array_mut()
        .ok_or_else(|| crate::messages::text("composition.invalid").to_owned())?;
    if current.len() + blocks.len() > rustforge_document::document::MAX_BLOCKS {
        return Err(crate::messages::text("composition.limit").into());
    }
    current.extend(blocks);
    projected(&next)?;
    Ok(next)
}

pub fn duplicate_block(value: &Value, id: &str) -> Option<Value> {
    let mut next = value.clone();
    let blocks = next["blocks"]["rf_blocks"].as_array_mut()?;
    if blocks.len() >= rustforge_document::document::MAX_BLOCKS {
        return None;
    }
    let index = blocks
        .iter()
        .position(|b| b["rf_id"].as_str() == Some(id))?;
    let mut duplicate = blocks[index].clone();
    duplicate["rf_id"] = json!(uuid::Uuid::new_v4().to_string());
    blocks.insert(index + 1, duplicate);
    Some(next)
}

/// A restore references one revision and the exact loaded server version only.
pub fn revision_restore(current: &Value, revision: &Value) -> Option<(String, Value)> {
    let entry_id = current["id"].as_str().filter(|id| local_id(id))?;
    let revision_id = revision["id"].as_str().filter(|id| local_id(id))?;
    if revision["entry_id"].as_str() != Some(entry_id) {
        return None;
    }
    let version = current["updated_at"].as_str()?;
    chrono::DateTime::parse_from_rfc3339(version).ok()?;
    Some((
        format!("/api/v1/entries/{entry_id}/revisions/{revision_id}/restore"),
        json!({"expected_updated_at":version}),
    ))
}

#[derive(Default)]
pub struct History {
    past: Vec<Value>,
    future: Vec<Value>,
}
impl History {
    pub fn push(&mut self, current: &Value) {
        self.future.clear();
        self.past.push(current.clone());
        if self.past.len() > 30
            || self
                .past
                .iter()
                .map(|value| value.to_string().len())
                .sum::<usize>()
                > 8 * 1024 * 1024
        {
            self.past.remove(0);
        }
    }
    pub fn undo(&mut self, current: &Value) -> Option<Value> {
        let last = self.past.pop()?;
        self.future.push(current.clone());
        Some(last)
    }
    pub fn redo(&mut self, current: &Value) -> Option<Value> {
        let next = self.future.pop()?;
        self.past.push(current.clone());
        Some(next)
    }
}
#[cfg(all(target_arch = "wasm32", feature = "ui"))]
pub mod ui;

/// Only editable content is retained locally; imported vaults and unrelated metadata stay on the server.
pub fn backup(value: &Value) -> Value {
    let keys = [
        "id",
        "content_type",
        "updated_at",
        "title",
        "slug",
        "excerpt",
        "body",
        "blocks",
        "status",
        "published_at",
        "scheduled_unpublish_at",
        "featured_media_id",
        "parent_id",
        "term_ids",
        "terms",
    ];
    let mut result = Value::Object(
        keys.into_iter()
            .filter_map(|key| value.get(key).map(|value| (key.to_owned(), value.clone())))
            .collect(),
    );
    if let Ok(Some(fields)) = document_settings(value) {
        result["fields"] = fields;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn discussion_patch_and_recovery_preserve_only_the_reserved_setting() {
        let mut entry = empty("post");
        entry["title"] = json!("Discussion");
        entry["fields"] =
            json!({"_rf_comment_status":"closed", "_legacy_vault":{"private":"opaque"}});
        assert_eq!(
            payload(&entry, false).unwrap()["fields"],
            json!({"_rf_comment_status":"closed"})
        );
        assert_eq!(
            backup(&entry)["fields"],
            json!({"_rf_comment_status":"closed"})
        );
        entry["fields"]["_rf_comment_status"] = Value::Null;
        assert_eq!(
            payload(&entry, false).unwrap()["fields"],
            json!({"_rf_comment_status":null})
        );
        entry["fields"]["_rf_comment_status"] = json!(true);
        assert!(payload(&entry, false).is_err());
        assert!(backup(&entry).get("fields").is_none());
    }

    #[test]
    fn recovery_and_patch_keep_only_the_editable_contract() {
        let mut entry = empty("post");
        entry["id"] = json!("00000000-0000-0000-0000-000000000001");
        entry["updated_at"] = json!("2026-10-30T20:00:00Z");
        entry["title"] = json!("Titre été");
        entry["fields"] = json!({"_legacy_vault":"opaque"});
        entry["intent"] = json!({"private":"opaque"});
        let backup = backup(&entry);
        assert!(backup.get("fields").is_none());
        assert!(backup.get("intent").is_none());
        assert_eq!(backup["updated_at"], entry["updated_at"]);
        let patch = payload(&entry, false).unwrap();
        assert_eq!(patch["expected_updated_at"], entry["updated_at"]);
        assert!(patch.get("fields").is_none());
        assert_eq!(patch["blocks"], entry["blocks"]);
    }
    #[test]
    fn history_restores_document_and_redo_is_invalidated_by_a_new_edit() {
        let mut history = History::default();
        let first = json!({"title":"A","blocks":[1]});
        let second = json!({"title":"B","blocks":[2]});
        history.push(&first);
        assert_eq!(history.undo(&second), Some(first.clone()));
        assert_eq!(history.redo(&first), Some(second.clone()));
        history.undo(&second);
        history.push(&first);
        assert!(history.redo(&second).is_none());
    }
    #[test]
    fn relation_patches_and_local_recovery_keep_explicit_clears() {
        let mut value = empty("page");
        value["title"] = json!("Page fille");
        value["parent_id"] = json!("00000000-0000-0000-0000-000000000001");
        value["term_ids"] = json!(["00000000-0000-0000-0000-000000000002"]);
        assert_eq!(
            payload(&value, true).unwrap()["parent_id"],
            value["parent_id"]
        );
        assert_eq!(backup(&value)["term_ids"], value["term_ids"]);
        value["parent_id"] = Value::Null;
        value["term_ids"] = json!([]);
        let patch = payload(&value, true).unwrap();
        assert!(patch["parent_id"].is_null());
        assert_eq!(patch["term_ids"], json!([]));
        let untouched = payload(&empty("post"), true);
        assert!(untouched.is_err());
        value.as_object_mut().unwrap().remove("term_ids");
        assert!(payload(&value, true).unwrap().get("term_ids").is_none());
    }

    #[test]
    fn restoration_is_bound_to_entry_and_loaded_version() {
        let current = json!({"id":"00000000-0000-0000-0000-000000000001","updated_at":"2026-10-31T09:00:00+01:00","fields":{"private":"opaque"}});
        let mut revision = json!({"id":"00000000-0000-0000-0000-000000000002","entry_id":current["id"],"snapshot":{"title":"old"}});
        let (path, body) = revision_restore(&current, &revision).unwrap();
        assert!(path.ends_with("/00000000-0000-0000-0000-000000000002/restore"));
        assert_eq!(body, json!({"expected_updated_at":current["updated_at"]}));
        revision["entry_id"] = json!("00000000-0000-0000-0000-000000000003");
        assert!(revision_restore(&current, &revision).is_none());
        revision["entry_id"] = current["id"].clone();
        let mut missing = current.clone();
        missing["updated_at"] = Value::Null;
        assert!(revision_restore(&missing, &revision).is_none());
        missing["updated_at"] = json!("yesterday");
        assert!(revision_restore(&missing, &revision).is_none());
    }

    #[test]
    fn dynamic_routes_stay_in_the_supported_content_types() {
        assert!(Location::parse("editor:00000000-0000-0000-0000-000000000001:post").is_some());
        assert!(Location::parse("editor:00000000-0000-0000-0000-000000000001:product").is_none());
        assert!(Location::parse("new:page").is_some());
        assert!(Location::parse("editor:000000000000000000000000000000000000:post").is_none());
    }
}

#[cfg(all(target_arch = "wasm32", feature = "ui"))]
pub mod revisions;

#[cfg(all(target_arch = "wasm32", feature = "ui"))]
pub mod form_block;

#[cfg(all(target_arch = "wasm32", feature = "ui"))]
pub mod featured;

/// Public links are shown only for already published, currently visible saved entries.
pub fn public_link(value: &Value, now_ms: i64) -> Option<String> {
    if value["status"] != "published" || value["id"].as_str().is_none_or(|id| !local_id(id)) {
        return None;
    }
    let published = chrono::DateTime::parse_from_rfc3339(value["published_at"].as_str()?).ok()?;
    if published.timestamp_millis() > now_ms {
        return None;
    }
    if let Some(until) = value["scheduled_unpublish_at"].as_str() {
        let until = chrono::DateTime::parse_from_rfc3339(until).ok()?;
        if until.timestamp_millis() <= now_ms {
            return None;
        }
    }
    let kind = value["content_type"].as_str()?;
    if !matches!(kind, "post" | "page") {
        return None;
    }
    let slug = value["slug"].as_str()?;
    let locale = value["locale"].as_str().unwrap_or("fr");
    if slug.is_empty()
        || locale.is_empty()
        || ![slug, locale].iter().all(|part| {
            part.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        })
    {
        return None;
    }
    Some(format!("/{locale}/{kind}/{slug}"))
}

#[cfg(test)]
mod presentation_tests {
    use super::*;
    #[test]
    fn featured_reference_survives_local_recovery_and_explicit_clear() {
        let mut value = empty("page");
        value["title"] = json!("Présentation");
        value["featured_media_id"] = json!("00000000-0000-0000-0000-000000000001");
        value["featured_media_url"] = json!("/uploads/private-cache.png");
        assert_eq!(
            payload(&backup(&value), true).unwrap()["featured_media_id"],
            value["featured_media_id"]
        );
        assert!(backup(&value).get("featured_media_url").is_none());
        value["featured_media_id"] = Value::Null;
        assert!(payload(&value, true).unwrap()["featured_media_id"].is_null());
        value["featured_media_id"] = json!("https://external.test/image");
        assert!(payload(&value, true).is_err());
    }
    #[test]
    fn public_links_respect_visibility_and_cannot_escape_the_site() {
        let mut value = json!({"id":"00000000-0000-0000-0000-000000000001","status":"published","content_type":"page","slug":"ete","locale":"fr","published_at":"2026-10-31T12:00:00Z"});
        let now = chrono::DateTime::parse_from_rfc3339("2026-10-31T13:00:00Z")
            .unwrap()
            .timestamp_millis();
        assert_eq!(public_link(&value, now).as_deref(), Some("/fr/page/ete"));
        assert!(public_link(&value, now - 7200000).is_none());
        value["slug"] = json!("//external.test");
        assert!(public_link(&value, now).is_none());
        value["slug"] = json!("ete");
        value["scheduled_unpublish_at"] = json!("2026-10-31T12:30:00Z");
        assert!(public_link(&value, now).is_none());
    }
}

#[cfg(all(target_arch = "wasm32", feature = "ui"))]
pub mod builder;

#[cfg(test)]
mod composition_tests {
    use super::*;
    #[test]
    fn shared_builder_appends_and_duplicates_without_losing_metadata_or_content() {
        for kind in ["post", "page"] {
            let mut entry = empty(kind);
            entry["title"] = json!("Essai");
            entry["fields"] = json!({"_legacy_vault":{"keep":true},"_rf_template":"landing"});
            entry["blocks"]["rf_blocks"] = json!([block("paragraph", "original".into())]);
            entry["blocks"]["rf_blocks"][0]["rf_text"] = json!("Conserver");
            let next = append_composition(&entry, "landing").unwrap();
            assert_eq!(
                next["blocks"]["rf_blocks"][0],
                entry["blocks"]["rf_blocks"][0]
            );
            assert_eq!(next["fields"], entry["fields"]);
            let dup = duplicate_block(&next, "original").unwrap();
            assert_eq!(dup["blocks"]["rf_blocks"][1]["rf_text"], "Conserver");
            assert_ne!(dup["blocks"]["rf_blocks"][1]["rf_id"], "original");
            assert_eq!(backup(&next)["fields"], json!({"_rf_template":"landing"}));
            assert_eq!(
                payload(&next, true).unwrap()["fields"],
                json!({"_rf_template":"landing"})
            );
        }
        assert!(
            append_composition(&json!({"blocks":[],"body":"Texte historique"}), "landing").is_err()
        );
        let mut full = empty("page");
        full["blocks"]["rf_blocks"] = json!(
            (0..256)
                .map(|i| block("paragraph", i.to_string()))
                .collect::<Vec<_>>()
        );
        assert!(append_composition(&full, "landing").is_err());
        assert!(duplicate_block(&full, "0").is_none());
    }
}
