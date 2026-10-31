//! Allowlisted settings/profile forms: send changed fields, never replay unrelated settings.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Control {
    Text,
    Email,
    Password,
    Page,
    Url,
    Area,
    Lines,
    Bool,
    Number(i64, i64),
    Select(&'static [(&'static str, &'static str)]),
}
#[derive(Clone, Copy)]
pub struct Field {
    pub key: &'static str,
    pub label: &'static str,
    pub control: Control,
    pub default: &'static str,
}
pub(crate) fn field(
    key: &'static str,
    label: &'static str,
    control: Control,
    default: &'static str,
) -> Field {
    Field {
        key,
        label,
        control,
        default,
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FormKind {
    General,
    Reading,
    Writing,
    Discussion,
    Media,
    Permalinks,
    Profile,
    User,
    NewUser,
    Styles,
    Analytics,
    FormRetention,
}
impl FormKind {
    pub fn from_route(route: &str) -> Option<Self> {
        match route {
            "settings" => Some(Self::General),
            "forms:settings" | "settings:forms" => Some(Self::FormRetention),
            "appearance:styles" => Some(Self::Styles),
            "analytics:settings" => Some(Self::Analytics),
            "settings:reading" => Some(Self::Reading),
            "settings:writing" => Some(Self::Writing),
            "settings:discussion" => Some(Self::Discussion),
            "settings:media" => Some(Self::Media),
            "settings:permalinks" => Some(Self::Permalinks),
            "profile" => Some(Self::Profile),
            "users:new" => Some(Self::NewUser),
            _ => None,
        }
    }
    pub fn title(self) -> &'static str {
        match self {
            Self::General => "form.general",
            Self::FormRetention => "retention.title",
            Self::Reading => "form.reading",
            Self::Writing => "form.writing",
            Self::Discussion => "form.discussion",
            Self::Media => "form.media",
            Self::Permalinks => "form.permalinks",
            Self::Profile => "form.profile",
            Self::User => "form.user",
            Self::NewUser => "form.new_user",
            Self::Styles => "appearance.styles",
            Self::Analytics => "insight.settings",
        }
    }
    pub fn fields(self) -> Vec<Field> {
        use Control::*;
        match self {
            Self::FormRetention => vec![field(
                "forms.retention",
                "retention.policy",
                Select(&[
                    ("none", "retention.none"),
                    ("90_days", "retention.90"),
                    ("30_days", "retention.30"),
                ]),
                "none",
            )],
            Self::Analytics => vec![
                field("enabled", "insight.collect", Bool, "true"),
                field("retention_months", "insight.retention", Number(1, 36), "13"),
                field("honor_dnt", "insight.honor_dnt", Bool, "true"),
                field(
                    "exclude_authenticated",
                    "insight.exclude_staff",
                    Bool,
                    "true",
                ),
                field(
                    "excluded_paths",
                    "insight.excluded_paths",
                    Lines,
                    "/admin\n/api\n/_\n/uploads",
                ),
            ],
            Self::Styles => vec![
                field("primary_color", "appearance.primary", Text, "#4f46e5"),
                field("accent_color", "appearance.accent", Text, "#06b6d4"),
                field("bg_color", "appearance.background", Text, "#ffffff"),
                field("text_color", "appearance.text_color", Text, "#1d2327"),
                field("radius", "appearance.radius", Text, "4px"),
                field("container_width", "appearance.width", Text, "1200px"),
                field("font_heading", "appearance.font_heading", Text, "system-ui"),
                field("font_body", "appearance.font_body", Text, "system-ui"),
                field("custom_css", "appearance.css", Area, ""),
            ],
            Self::General => vec![
                field("site.title", "form.site_title", Text, ""),
                field("site.tagline", "form.tagline", Text, ""),
                field("site.url", "form.site_url", Url, ""),
                field("site.home_url", "form.home_url", Url, ""),
                field("site.admin_email", "form.admin_email", Email, ""),
                field("site.language", "form.language", Text, "fr"),
                field("site.timezone", "form.timezone", Text, "Europe/Paris"),
                field("site.date_format", "form.date_format", Text, "j F Y"),
                field("site.time_format", "form.time_format", Text, "H:i"),
                field("site.start_of_week", "form.week_start", Number(0, 6), "1"),
                field("users.can_register", "form.register", Bool, "false"),
                field(
                    "users.default_role",
                    "form.default_role",
                    Select(&[
                        ("subscriber", "screens.subscriber"),
                        ("author", "screens.author"),
                        ("editor", "screens.editor"),
                    ]),
                    "subscriber",
                ),
            ],
            Self::Reading => vec![
                field(
                    "reading.show_on_front",
                    "form.front_mode",
                    Select(&[("posts", "form.latest_posts"), ("page", "form.static_page")]),
                    "posts",
                ),
                field("reading.page_on_front", "form.front_page", Page, ""),
                field("reading.page_for_posts", "form.posts_page", Page, ""),
                field("reading.per_page", "form.per_page", Number(1, 100), "10"),
                field(
                    "reading.posts_per_rss",
                    "form.rss_count",
                    Number(1, 100),
                    "10",
                ),
                field("reading.rss_use_excerpt", "form.rss_excerpt", Bool, "false"),
                field("reading.blog_public", "form.index", Bool, "true"),
            ],
            Self::Writing => vec![
                field(
                    "writing.default_category",
                    "form.default_category",
                    Text,
                    "",
                ),
                field(
                    "writing.default_post_format",
                    "form.post_format",
                    Select(&[
                        ("standard", "form.standard"),
                        ("aside", "form.aside"),
                        ("gallery", "form.gallery"),
                        ("image", "form.image"),
                        ("quote", "form.quote"),
                        ("video", "form.video"),
                        ("audio", "form.audio"),
                        ("link", "form.link"),
                    ]),
                    "standard",
                ),
                field(
                    "writing.default_editor_mode",
                    "form.editor_mode",
                    Select(&[("blocks", "form.blocks"), ("markdown", "form.markdown")]),
                    "blocks",
                ),
            ],
            Self::Discussion => vec![
                field(
                    "discussion.default_comment_status",
                    "form.comments_open",
                    Select(&[("open", "form.open"), ("closed", "form.closed")]),
                    "open",
                ),
                field(
                    "discussion.require_name_email",
                    "form.require_identity",
                    Bool,
                    "true",
                ),
                field(
                    "discussion.comment_registration",
                    "form.require_login",
                    Bool,
                    "false",
                ),
                field(
                    "discussion.close_comments_for_old_posts",
                    "form.close_old",
                    Bool,
                    "false",
                ),
                field(
                    "discussion.close_comments_days",
                    "form.close_days",
                    Number(1, 3650),
                    "14",
                ),
                field("discussion.thread_comments", "form.threaded", Bool, "true"),
                field(
                    "discussion.thread_comments_depth",
                    "form.depth",
                    Number(1, 5),
                    "3",
                ),
                field(
                    "discussion.page_comments",
                    "form.page_comments",
                    Bool,
                    "false",
                ),
                field(
                    "discussion.comments_per_page",
                    "form.comments_per_page",
                    Number(1, 100),
                    "50",
                ),
                field(
                    "discussion.comment_order",
                    "form.comment_order",
                    Select(&[("asc", "form.old_first"), ("desc", "form.new_first")]),
                    "asc",
                ),
                field(
                    "discussion.comment_moderation",
                    "form.moderation",
                    Bool,
                    "true",
                ),
                field(
                    "discussion.comment_previously_approved",
                    "form.approved",
                    Bool,
                    "false",
                ),
                field(
                    "discussion.comment_max_links",
                    "form.max_links",
                    Number(0, 50),
                    "2",
                ),
                field(
                    "discussion.moderation_keys",
                    "form.moderation_keys",
                    Area,
                    "",
                ),
                field(
                    "discussion.disallowed_keys",
                    "form.disallowed_keys",
                    Area,
                    "",
                ),
                field("discussion.show_avatars", "form.avatars", Bool, "true"),
                field(
                    "discussion.show_comments_cookies_opt_in",
                    "form.identity_optin",
                    Bool,
                    "false",
                ),
            ],
            Self::Media => vec![
                field(
                    "media.thumbnail_size_w",
                    "form.thumbnail_w",
                    Number(32, 4096),
                    "150",
                ),
                field(
                    "media.thumbnail_size_h",
                    "form.thumbnail_h",
                    Number(32, 4096),
                    "150",
                ),
                field(
                    "media.medium_size_w",
                    "form.medium_w",
                    Number(32, 4096),
                    "300",
                ),
                field(
                    "media.medium_size_h",
                    "form.medium_h",
                    Number(32, 4096),
                    "300",
                ),
                field(
                    "media.large_size_w",
                    "form.large_w",
                    Number(32, 4096),
                    "1024",
                ),
                field(
                    "media.large_size_h",
                    "form.large_h",
                    Number(32, 4096),
                    "1024",
                ),
            ],
            Self::Permalinks => vec![
                field(
                    "permalink.structure",
                    "form.permalink_structure",
                    Text,
                    "post_name",
                ),
                field(
                    "permalink.category_base",
                    "form.category_base",
                    Text,
                    "category",
                ),
                field("permalink.tag_base", "form.tag_base", Text, "tag"),
            ],
            Self::User => vec![
                field("display_name", "form.display_name", Text, ""),
                field("email", "screens.email", Email, ""),
                field("bio", "form.bio", Area, ""),
                field(
                    "role",
                    "screens.role",
                    Select(&[
                        ("admin", "screens.admin"),
                        ("editor", "screens.editor"),
                        ("author", "screens.author"),
                        ("subscriber", "screens.subscriber"),
                    ]),
                    "subscriber",
                ),
            ],
            Self::NewUser => vec![
                field("username", "form.username", Text, ""),
                field("email", "screens.email", Email, ""),
                field("display_name", "form.display_name", Text, ""),
                field("password", "form.password", Password, ""),
                field(
                    "role",
                    "screens.role",
                    Select(&[
                        ("admin", "screens.admin"),
                        ("editor", "screens.editor"),
                        ("author", "screens.author"),
                        ("subscriber", "screens.subscriber"),
                    ]),
                    "subscriber",
                ),
            ],
            Self::Profile => vec![
                field("display_name", "form.display_name", Text, ""),
                field("email", "screens.email", Email, ""),
                field("bio", "form.bio", Area, ""),
            ],
        }
    }
}
#[cfg(all(target_arch = "wasm32", feature = "ui"))]
pub mod ui;
