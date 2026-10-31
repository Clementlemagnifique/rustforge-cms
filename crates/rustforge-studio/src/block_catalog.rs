//! Business block catalog. Only explicitly implemented types enter the document contract.
#[derive(Clone, Copy)]
pub struct PlannedBlock {
    pub id: &'static str,
    pub title: &'static str,
    pub module: &'static str,
    pub reference: &'static str,
    pub options: &'static [&'static str],
}
pub const PLANNED: &[PlannedBlock] = &[
    PlannedBlock {
        id: "form_embed",
        title: "blocks.form",
        module: "forms",
        reference: "wordpress-plugins-ref/plugins-ref/wpforms-lite",
        options: &[
            "blocks.form_select",
            "blocks.form_style",
            "blocks.form_confirmation",
            "blocks.form_conditions",
        ],
    },
    PlannedBlock {
        id: "course_list",
        title: "blocks.course_list",
        module: "lms",
        reference: "wordpress-plugins-ref/plugins-ref/sensei-lms",
        options: &[
            "blocks.course_filters",
            "blocks.course_access",
            "blocks.course_progress",
        ],
    },
    PlannedBlock {
        id: "lesson_navigation",
        title: "blocks.lesson_navigation",
        module: "lms",
        reference: "wordpress-plugins-ref/plugins-ref/sensei-lms",
        options: &[
            "blocks.lesson_order",
            "blocks.lesson_completion",
            "blocks.course_access",
        ],
    },
    PlannedBlock {
        id: "quiz",
        title: "blocks.quiz",
        module: "lms",
        reference: "wordpress-plugins-ref/plugins-ref/sensei-lms",
        options: &[
            "blocks.quiz_questions",
            "blocks.quiz_scoring",
            "blocks.quiz_attempts",
        ],
    },
    PlannedBlock {
        id: "product_collection",
        title: "blocks.products",
        module: "ecommerce",
        reference: "wordpress-plugins-ref/plugins-ref/easy-digital-downloads",
        options: &[
            "blocks.product_filters",
            "blocks.product_prices",
            "blocks.product_downloads",
        ],
    },
    PlannedBlock {
        id: "checkout",
        title: "blocks.checkout",
        module: "ecommerce",
        reference: "wordpress-plugins-ref/plugins-ref/easy-digital-downloads",
        options: &[
            "blocks.checkout_summary",
            "blocks.checkout_gateways",
            "blocks.checkout_consent",
        ],
    },
    PlannedBlock {
        id: "gallery_embed",
        title: "blocks.gallery",
        module: "galleries",
        reference: "wordpress-plugins-ref/plugins-ref/gt3-photo-video-gallery",
        options: &[
            "blocks.gallery_sources",
            "blocks.gallery_layout",
            "blocks.gallery_lightbox",
        ],
    },
    PlannedBlock {
        id: "mail_signup",
        title: "blocks.signup",
        module: "mailing",
        reference: "wordpress-plugins-ref/plugins-ref/mailpoet",
        options: &[
            "blocks.signup_list",
            "blocks.signup_consent",
            "blocks.signup_confirmation",
        ],
    },
    PlannedBlock {
        id: "events",
        title: "blocks.events",
        module: "events",
        reference: "wordpress-plugins-ref/plugins-ref/the-events-calendar",
        options: &[
            "blocks.event_views",
            "blocks.event_filters",
            "blocks.event_timezone",
        ],
    },
    PlannedBlock {
        id: "discussion",
        title: "blocks.discussion",
        module: "comments",
        reference: "wordpress-plugins-ref/plugins-ref/wpdiscuz",
        options: &[
            "blocks.discussion_threads",
            "blocks.discussion_moderation",
            "blocks.discussion_subscriptions",
        ],
    },
];
#[cfg(all(target_arch = "wasm32", feature = "ui"))]
pub mod ui;

/// Functional inspiration, distinct from insertable page blocks.
pub const FUTURE_ADJUSTMENTS: &[(&str, &str)] = &[
    (
        "adjustment.media_cleanup",
        "adjustment.media_cleanup_options",
    ),
    ("adjustment.image_sizes", "adjustment.image_sizes_options"),
    ("adjustment.gallery", "adjustment.gallery_options"),
    ("adjustment.ai", "adjustment.ai_options"),
    ("adjustment.mail", "adjustment.mail_options"),
];
