//! Stable message keys; French only for now. Server-provided menu labels remain authoritative.
use std::{collections::BTreeMap, sync::LazyLock};
static FRENCH: LazyLock<BTreeMap<String, String>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../locales/fr.json")).expect("valid bundled French catalog")
});
pub fn text(key: &'static str) -> &'static str {
    FRENCH.get(key).map(String::as_str).unwrap_or(key)
}
#[cfg(test)]
mod tests {
    #[test]
    fn shell_message_keys_are_complete() {
        let app = [
            include_str!("app.rs"),
            include_str!("shell.rs"),
            include_str!("screens.rs"),
            include_str!("screens/ui.rs"),
            include_str!("dashboard/ui.rs"),
            include_str!("dashboard/client.rs"),
            include_str!("dashboard.rs"),
            include_str!("dashboard/layout.rs"),
            include_str!("forms/ui.rs"),
            include_str!("editor/ui.rs"),
            include_str!("editor/revisions.rs"),
            include_str!("editor/form_block.rs"),
            include_str!("editor/featured.rs"),
            include_str!("editor/builder.rs"),
            include_str!("relations/ui.rs"),
            include_str!("taxonomies_ui.rs"),
            include_str!("http.rs"),
            include_str!("media_ui.rs"),
            include_str!("appearance_ui.rs"),
            include_str!("modules/ui.rs"),
            include_str!("submissions/ui.rs"),
            include_str!("modules/center.rs"),
            include_str!("block_catalog/ui.rs"),
            include_str!("diagnostics_ui.rs"),
            include_str!("assistant_ui.rs"),
            include_str!("learning_ui.rs"),
            include_str!("orders_ui.rs"),
            include_str!("comments_ui.rs"),
        ]
        .join("\n");
        for rest in app.split("text(\"").skip(1) {
            let key = rest.split('"').next().unwrap();
            assert!(
                super::FRENCH.get(key).is_some_and(|s| !s.trim().is_empty()),
                "Missing {key}"
            );
        }
        for key in app.split('"').filter(|key| {
            key.starts_with("dashboard.")
                || key.starts_with("shell.")
                || key.starts_with("screens.")
        }) {
            assert!(super::FRENCH.contains_key(key), "Missing {key}");
        }
    }
}

#[cfg(test)]
mod blueprint_tests {
    use super::*;
    #[test]
    fn descriptors_have_translated_labels() {
        use crate::forms::{Control, FormKind};
        let mut keys = Vec::new();
        for kind in [
            FormKind::General,
            FormKind::FormRetention,
            FormKind::Reading,
            FormKind::Writing,
            FormKind::Discussion,
            FormKind::Media,
            FormKind::Permalinks,
            FormKind::Profile,
            FormKind::User,
            FormKind::NewUser,
            FormKind::Styles,
            FormKind::Analytics,
        ] {
            keys.push(kind.title());
            for field in kind.fields() {
                keys.push(field.label);
                if let Control::Select(options) = field.control {
                    keys.extend(options.iter().map(|(_, label)| *label));
                }
            }
        }
        for resource in [
            crate::modules::Resource::Forms,
            crate::modules::Resource::Courses,
            crate::modules::Resource::Products,
            crate::modules::Resource::Galleries,
            crate::modules::Resource::Campaigns,
            crate::modules::Resource::Recipes,
        ] {
            keys.push(resource.title());
            for field in resource.fields() {
                keys.push(field.label);
                if let Control::Select(options) = field.control {
                    keys.extend(options.iter().map(|(_, label)| *label));
                }
            }
        }
        keys.extend(crate::editor::BLOCKS.iter().map(|(_, label)| *label));
        for block in crate::block_catalog::PLANNED {
            keys.push(block.title);
            keys.extend(block.options.iter().copied());
        }
        for (title, options) in crate::block_catalog::FUTURE_ADJUSTMENTS {
            keys.push(title);
            keys.push(options);
        }
        for key in keys {
            assert!(
                FRENCH
                    .get(key)
                    .is_some_and(|value| !value.trim().is_empty()),
                "Missing {key}"
            );
        }
    }
}
