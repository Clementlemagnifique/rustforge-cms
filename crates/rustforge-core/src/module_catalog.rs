//! Native extension manifests. Disabling a domain keeps its data; permissions stay in its services.
use serde::Serialize;
pub const MODULE_IDS: [&str; 8] = [
    "forms",
    "galleries",
    "zen_mode",
    "ecommerce",
    "lms",
    "analytics",
    "mailing",
    "automations",
];
#[derive(Clone, Copy, Serialize)]
pub struct Manifest {
    pub id: &'static str,
    pub name: &'static str,
    pub version: &'static str,
    pub capabilities: &'static [&'static str],
    pub references: &'static [&'static str],
    pub future_blocks: &'static [&'static str],
    pub keep_data_on_disable: bool,
}
pub const MANIFESTS: &[Manifest] = &[
    Manifest {
        id: "forms",
        name: "Formulaires",
        version: env!("CARGO_PKG_VERSION"),
        capabilities: &[
            "schema_builder",
            "public_submission",
            "submission_review",
            "form_embed",
        ],
        references: &["wpforms-lite"],
        future_blocks: &[],
        keep_data_on_disable: true,
    },
    Manifest {
        id: "galleries",
        name: "Galeries",
        version: env!("CARGO_PKG_VERSION"),
        capabilities: &["local_collection", "media_selection"],
        references: &["gt3-photo-video-gallery", "thedock-media"],
        future_blocks: &["gallery_embed"],
        keep_data_on_disable: true,
    },
    Manifest {
        id: "zen_mode",
        name: "Lecture immersive",
        version: env!("CARGO_PKG_VERSION"),
        capabilities: &["ssr_reading"],
        references: &["wordpress"],
        future_blocks: &[],
        keep_data_on_disable: true,
    },
    Manifest {
        id: "ecommerce",
        name: "Boutique",
        version: env!("CARGO_PKG_VERSION"),
        capabilities: &["products", "manual_orders", "protected_downloads"],
        references: &["easy-digital-downloads"],
        future_blocks: &["product_collection", "checkout"],
        keep_data_on_disable: true,
    },
    Manifest {
        id: "lms",
        name: "Formation",
        version: env!("CARGO_PKG_VERSION"),
        capabilities: &["courses", "lessons", "enrollments", "declared_progress"],
        references: &["sensei-lms"],
        future_blocks: &["course_list", "lesson_navigation", "quiz"],
        keep_data_on_disable: true,
    },
    Manifest {
        id: "analytics",
        name: "Statistiques",
        version: env!("CARGO_PKG_VERSION"),
        capabilities: &["opt_out_collection", "audience_summary"],
        references: &["wp-statistics"],
        future_blocks: &[],
        keep_data_on_disable: true,
    },
    Manifest {
        id: "mailing",
        name: "Campagnes locales",
        version: env!("CARGO_PKG_VERSION"),
        capabilities: &["account_recipients", "campaign_draft", "capture_queue"],
        references: &["mailpoet"],
        future_blocks: &["mail_signup"],
        keep_data_on_disable: true,
    },
    Manifest {
        id: "automations",
        name: "Recettes manuelles",
        version: env!("CARGO_PKG_VERSION"),
        capabilities: &["manual_trigger", "delayed_capture", "durable_jobs"],
        references: &["publishpress"],
        future_blocks: &[],
        keep_data_on_disable: true,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_modules_have_unique_manifests_and_keep_their_data() {
        let ids: std::collections::BTreeSet<_> =
            MANIFESTS.iter().map(|manifest| manifest.id).collect();
        assert_eq!(ids.len(), MODULE_IDS.len());
        assert!(MODULE_IDS.iter().all(|id| ids.contains(id)));
        assert!(
            MANIFESTS
                .iter()
                .all(|manifest| manifest.keep_data_on_disable
                    && !manifest.references.is_empty()
                    && !manifest.capabilities.is_empty())
        );
    }
}
