//! Native module screens reuse the domain APIs; flags remain authoritative on the server.
use crate::forms::{Control, Field, field};
use serde_json::{Value, json};
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Resource {
    Forms,
    Courses,
    Products,
    Galleries,
    Campaigns,
    Recipes,
}
impl Resource {
    pub fn from_route(route: &str) -> Option<Self> {
        match route {
            "forms" | "forms:new" | "forms:submissions" => Some(Self::Forms),
            "lms" | "lms:courses" | "lms:lessons" | "lms:learners" => Some(Self::Courses),
            "ecommerce" | "ecommerce:products" => Some(Self::Products),
            "module:galleries" | "galleries:new" => Some(Self::Galleries),
            "mailing" | "mailing:campaigns" | "mailing:new" => Some(Self::Campaigns),
            "automations" | "automations:recipes" | "automations:new" => Some(Self::Recipes),
            _ => None,
        }
    }
    pub fn endpoint(self) -> &'static str {
        match self {
            Self::Forms => "/api/v1/forms",
            Self::Courses => "/api/v1/lms/courses",
            Self::Products => "/api/v1/ecommerce/products",
            Self::Galleries => "/api/v1/galleries",
            Self::Campaigns => "/api/v1/mailing/campaigns",
            Self::Recipes => "/api/v1/automations/recipes",
        }
    }
    pub fn title(self) -> &'static str {
        match self {
            Self::Forms => "module.forms",
            Self::Courses => "module.courses",
            Self::Products => "module.products",
            Self::Galleries => "module.galleries",
            Self::Campaigns => "module.campaigns",
            Self::Recipes => "module.recipes",
        }
    }
    pub fn fields(self) -> Vec<Field> {
        use Control::*;
        match self {
            Self::Forms => vec![
                field("name", "module.name", Text, ""),
                field("slug", "editor.slug", Text, ""),
                field("description", "module.description", Area, ""),
                field("is_active", "module.active", Bool, "false"),
            ],
            Self::Courses => vec![
                field("title", "screens.title", Text, ""),
                field("slug", "editor.slug", Text, ""),
                field("description", "module.description", Area, ""),
                field("content", "editor.content", Area, ""),
                field(
                    "price_cents",
                    "module.price_cents",
                    Number(0, 100000000),
                    "0",
                ),
                field("is_published", "module.published", Bool, "false"),
            ],
            Self::Products => vec![
                field("title", "screens.title", Text, ""),
                field("slug", "editor.slug", Text, ""),
                field("excerpt", "editor.excerpt", Area, ""),
                field("description", "module.description", Area, ""),
                field(
                    "price_cents",
                    "module.price_cents",
                    Number(0, 100000000),
                    "0",
                ),
                field(
                    "download_limit",
                    "module.download_limit",
                    Number(1, 1000),
                    "5",
                ),
                field(
                    "status",
                    "screens.status",
                    Select(&[
                        ("draft", "dashboard.draft"),
                        ("published", "dashboard.published_item"),
                        ("archived", "dashboard.archived"),
                    ]),
                    "draft",
                ),
            ],
            Self::Galleries => vec![
                field("name", "module.name", Text, ""),
                field("slug", "editor.slug", Text, ""),
                field("description", "module.description", Area, ""),
                field(
                    "layout",
                    "module.gallery_layout",
                    Select(&[
                        ("grid", "module.grid"),
                        ("masonry", "module.masonry"),
                        ("slider", "module.slider"),
                    ]),
                    "grid",
                ),
            ],
            Self::Campaigns => vec![
                field("name", "module.name", Text, ""),
                field("subject", "module.subject", Text, ""),
                field("body", "editor.content", Area, ""),
            ],
            Self::Recipes => vec![
                field("name", "module.name", Text, ""),
                field("subject", "module.subject", Text, ""),
                field("body", "editor.content", Area, ""),
                field("delay_seconds", "module.delay", Number(0, 2592000), "0"),
                field("active", "module.active", Bool, "false"),
            ],
        }
    }
    pub fn empty(self) -> Value {
        match self {
            Self::Forms => {
                json!({"fields":{"type":"object","properties":{},"required":[],"additionalProperties":false},"settings":{}})
            }
            Self::Galleries => json!({"settings":{}}),
            _ => json!({}),
        }
    }
}
#[cfg(all(target_arch = "wasm32", feature = "ui"))]
pub mod ui;

#[cfg(all(target_arch = "wasm32", feature = "ui"))]
pub mod center;
