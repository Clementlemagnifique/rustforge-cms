//! Read-only native lists over the existing APIs. Mutations stay in the current editor.
use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListKind {
    Posts,
    Pages,
    Media,
    Users,
}
impl ListKind {
    pub fn from_route(route: &str) -> Option<Self> {
        match route {
            "entries:post" => Some(Self::Posts),
            "entries:page" => Some(Self::Pages),
            "media" => Some(Self::Media),
            "users" => Some(Self::Users),
            _ => None,
        }
    }
    pub fn endpoint(self, query: &str, page: u64) -> String {
        let (base, search) = match self {
            Self::Posts => ("entries?content_type=post&", "q"),
            Self::Pages => ("entries?content_type=page&", "q"),
            Self::Media => ("media?", "q"),
            Self::Users => ("users?", "search"),
        };
        let encoded: String = query
            .bytes()
            .map(|b| {
                if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
                    (b as char).to_string()
                } else {
                    format!("%{b:02X}")
                }
            })
            .collect();
        format!(
            "/api/v1/{base}{search}={encoded}&page={}&per_page=20",
            page.max(1)
        )
    }
    pub fn headings(self) -> [&'static str; 4] {
        match self {
            Self::Posts | Self::Pages => [
                "screens.title",
                "screens.author",
                "screens.status",
                "screens.date",
            ],
            Self::Media => [
                "screens.file",
                "screens.format",
                "screens.size",
                "screens.uploaded",
            ],
            Self::Users => [
                "screens.name",
                "screens.email",
                "screens.role",
                "screens.posts",
            ],
        }
    }
}
#[derive(Clone, Debug)]
pub struct ListState {
    pub search: String,
    pub filter: String,
    pub order: String,
    pub page: u64,
    pub columns: [bool; 4],
}
impl Default for ListState {
    fn default() -> Self {
        Self {
            search: String::new(),
            filter: String::new(),
            order: "updated".into(),
            page: 1,
            columns: [true; 4],
        }
    }
}
impl ListKind {
    pub fn filtered_endpoint(self, state: &ListState) -> String {
        let mut endpoint = self.endpoint(&state.search, state.page);
        let filter = state.filter.as_str();
        match self {
            Self::Posts | Self::Pages => {
                if matches!(filter, "draft" | "published" | "archived" | "trash") {
                    endpoint.push_str(&format!("&status={filter}"));
                }
                if matches!(
                    state.order.as_str(),
                    "title" | "created" | "published" | "updated"
                ) {
                    endpoint.push_str(&format!("&order={}", state.order));
                }
            }
            Self::Users => {
                if matches!(filter, "admin" | "editor" | "author" | "subscriber") {
                    endpoint.push_str(&format!("&role={filter}"));
                }
            }
            Self::Media => {
                if matches!(filter, "image" | "video" | "audio" | "application") {
                    endpoint.push_str(&format!("&kind={filter}"));
                }
            }
        }
        endpoint
    }
}
#[derive(Clone, Debug, Deserialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub total_pages: u64,
}
#[derive(Clone, Debug, Deserialize)]
pub struct Entry {
    pub id: String,
    pub title: String,
    pub status: String,
    pub content_type: String,
    pub author_name: Option<String>,
    pub updated_at: String,
    pub published_at: Option<String>,
}
#[derive(Clone, Debug, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub role: String,
    pub entry_count: Option<u64>,
}
#[derive(Clone, Debug, Deserialize)]
pub struct Media {
    pub id: String,
    pub original_name: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub created_at: String,
}
#[derive(Clone, Debug)]
pub struct Row {
    pub entry: Option<EntryAction>,
    pub cells: [String; 4],
    pub href: Option<String>,
}
#[derive(Clone, Debug)]
pub struct EntryAction {
    pub title: String,
    pub id: String,
    pub updated_at: String,
    pub status: String,
}
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub rows: Vec<Row>,
    pub total: u64,
    pub page: u64,
    pub pages: u64,
}
fn snapshot<T>(page: Page<T>, convert: impl Fn(T) -> Row) -> Result<Snapshot, serde_json::Error> {
    if page.items.len() > 20
        || page.page == 0
        || (page.total > 0 && (page.total_pages == 0 || page.page > page.total_pages))
    {
        return Err(<serde_json::Error as serde::de::Error>::custom(
            "Invalid pagination",
        ));
    }
    Ok(Snapshot {
        rows: page.items.into_iter().map(convert).collect(),
        total: page.total,
        page: page.page,
        pages: page.total_pages,
    })
}
fn date(value: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|date| date.format("%d/%m/%Y %H:%M %:z").to_string())
        .unwrap_or_else(|_| "—".into())
}
pub fn decode(kind: ListKind, body: &str, now: i64) -> Result<Snapshot, serde_json::Error> {
    use crate::messages::text;
    match kind {
        ListKind::Posts | ListKind::Pages => {
            snapshot(serde_json::from_str::<Page<Entry>>(body)?, |item| {
                let href = ((kind == ListKind::Posts && item.content_type == "post"
                    || kind == ListKind::Pages && item.content_type == "page")
                    && item.id.len() == 36
                    && item.id.bytes().all(|b| b.is_ascii_hexdigit() || b == b'-'))
                .then(|| format!("#editor:{}:{}", item.id, item.content_type));
                Row {
                    entry: Some(EntryAction {
                        title: item.title.clone(),
                        id: item.id.clone(),
                        updated_at: item.updated_at.clone(),
                        status: item.status.clone(),
                    }),
                    cells: [
                        if item.title.is_empty() {
                            text("screens.untitled").into()
                        } else {
                            item.title
                        },
                        item.author_name.unwrap_or_else(|| "—".into()),
                        text(crate::dashboard::state_key(if item.status == "published" {
                            match item
                                .published_at
                                .as_deref()
                                .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
                            {
                                Some(date) if date.timestamp() > now => "scheduled",
                                Some(_) => "published",
                                None => "undated",
                            }
                        } else {
                            &item.status
                        }))
                        .into(),
                        date(&item.updated_at),
                    ],
                    href,
                }
            })
        }
        ListKind::Users => snapshot(serde_json::from_str::<Page<User>>(body)?, |item| Row {
            entry: None,
            cells: [
                if item.display_name.trim().is_empty() {
                    item.username
                } else {
                    item.display_name
                },
                item.email,
                match item.role.as_str() {
                    "admin" => text("screens.admin"),
                    "editor" => text("screens.editor"),
                    "author" => text("screens.author"),
                    "subscriber" => text("screens.subscriber"),
                    _ => text("screens.unknown"),
                }
                .into(),
                item.entry_count
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "—".into()),
            ],
            href: crate::editor::local_id(&item.id).then(|| format!("#user:{}", item.id)),
        }),
        ListKind::Media => snapshot(serde_json::from_str::<Page<Media>>(body)?, |item| Row {
            entry: None,
            cells: [
                item.original_name,
                item.mime_type,
                format!(
                    "{:.1} {}",
                    item.size_bytes as f64 / 1024.,
                    text("screens.kib")
                ),
                date(&item.created_at),
            ],
            href: crate::editor::local_id(&item.id).then(|| format!("#media:{}", item.id)),
        }),
    }
}
#[cfg(all(target_arch = "wasm32", feature = "ui"))]
pub mod ui;
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn query_cannot_override_scope() {
        let url = ListKind::Posts.endpoint("été&content_type=page#", 0);
        assert!(url.contains("content_type=post&q=%C3%A9t%C3%A9%26content_type%3Dpage%23&page=1"));
        assert_eq!(ListKind::from_route("module:ecommerce"), None);
    }
    #[test]
    fn scheduled_and_undated_are_not_reported_as_published() {
        let data = r#"{"items":[{"id":"00000000-0000-0000-0000-000000000000","title":"Future","status":"published","content_type":"post","updated_at":"2026-10-30T12:00:00Z","published_at":"2027-01-01T00:00:00+02:00"}],"total":1,"page":1,"total_pages":1}"#;
        let snapshot = decode(ListKind::Posts, data, 1_000).unwrap();
        assert_eq!(
            snapshot.rows[0].cells[2],
            crate::messages::text("dashboard.scheduled_item")
        );
        let invalid = data.replace("2027-01-01T00:00:00+02:00", "bad");
        assert_eq!(
            decode(ListKind::Posts, &invalid, 1_000).unwrap().rows[0].cells[2],
            crate::messages::text("dashboard.undated")
        );
    }
    #[test]
    fn unexpected_entry_type_never_produces_editor_link() {
        let data = r#"{"items":[{"id":"00000000-0000-0000-0000-000000000000","title":"<script>","status":"draft","content_type":"product","updated_at":"2026-10-30T12:00:00Z"}],"total":1,"page":1,"total_pages":1}"#;
        let result = decode(ListKind::Posts, data, 0).unwrap();
        assert!(result.rows[0].href.is_none());
        assert_eq!(result.rows[0].cells[0], "<script>");
        assert!(decode(ListKind::Users, data, 0).is_err());
        assert!(
            decode(
                ListKind::Media,
                r#"{"items":[],"total":1,"page":0,"total_pages":1}"#,
                0
            )
            .is_err()
        );
    }
}
