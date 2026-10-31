//! DTO du tableau de bord et commande de brouillon ; aucune dépendance serveur.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct TypeStats {
    pub slug: String,
    pub name: String,
    pub published: u64,
    pub drafts: u64,
    pub scheduled: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Activity {
    pub id: String,
    pub title: String,
    pub content_type: String,
    #[serde(alias = "status")]
    pub state: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct DashboardData {
    pub server_version: String,
    pub content_types: Vec<TypeStats>,
    pub media: u64,
    pub media_bytes: u64,
    pub activity: Vec<Activity>,
}

#[derive(Serialize)]
pub struct QuickDraft {
    pub title: String,
    pub body: String,
    pub blocks: serde_json::Value,
    pub status: &'static str,
}

impl QuickDraft {
    pub fn new(title: &str, body: &str) -> Result<Self, &'static str> {
        let title = title.trim();
        if title.is_empty() || title.chars().count() > 300 {
            return Err("dashboard.title_invalid");
        }
        if body.chars().count() > 20_000 {
            return Err("dashboard.body_invalid");
        }
        Ok(Self {
            title: title.into(),
            body: String::new(),
            blocks: serde_json::json!({"rf_schema":"rf.document/1","rf_blocks":[{
                "rf_id":"quick-draft-text", "rf_type":"paragraph", "rf_text":body
            }]}),
            status: "draft",
        })
    }
}

pub fn editor_href(entry: &Activity) -> Option<String> {
    // L'éditeur de transition générique ne doit pas contourner un module inactif.
    if !matches!(entry.content_type.as_str(), "post" | "page") {
        return None;
    }
    let route = format!("editor:{}:{}", entry.id, entry.content_type);
    (crate::navigation::route_from_hash(&route).as_ref() == Some(&route))
        .then(|| format!("/admin/rust/#{route}"))
}

pub fn state_key(state: &str) -> &'static str {
    match state {
        "draft" => "dashboard.draft",
        "published" => "dashboard.published_item",
        "scheduled" => "dashboard.scheduled_item",
        "archived" => "dashboard.archived",
        "trash" => "dashboard.trash",
        "undated" => "dashboard.undated",
        _ => "dashboard.unknown_state",
    }
}

#[cfg(all(target_arch = "wasm32", feature = "ui"))]
pub mod ui;

#[cfg(all(target_arch = "wasm32", feature = "ui"))]
mod client;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quick_draft_preserves_unicode_and_uses_the_shared_document() {
        let input = QuickDraft::new("  Ébauche 🌍 ", "Texte **riche**\n\nSuite").unwrap();
        assert_eq!(input.title, "Ébauche 🌍");
        assert_eq!(input.status, "draft");
        assert_eq!(
            input.blocks["rf_blocks"][0]["rf_text"],
            "Texte **riche**\n\nSuite"
        );
        assert!(rustforge_document::document::project(&input.body, &input.blocks).is_ok());
        assert!(QuickDraft::new(" ", "Texte").is_err());
        assert!(QuickDraft::new(&"é".repeat(301), "").is_err());
        assert!(QuickDraft::new("Titre", &"🌍".repeat(20_001)).is_err());
    }

    #[test]
    fn activity_links_cannot_escape_the_local_editor() {
        let mut item = Activity {
            id: "fictitious-id".into(),
            title: "Titre".into(),
            content_type: "post".into(),
            state: "draft".into(),
        };
        assert_eq!(
            editor_href(&item).as_deref(),
            Some("/admin/rust/#editor:fictitious-id:post")
        );
        item.content_type = "post?next=https://outside.invalid".into();
        assert!(editor_href(&item).is_none());
        item.content_type = "course".into();
        assert!(editor_href(&item).is_none());
    }
}

#[cfg(all(target_arch = "wasm32", feature = "ui"))]
pub mod layout;
