//! Same-origin authenticated requests. Error text is never interpreted as markup.
use crate::{app::session_token, messages::text};
use serde_json::Value;
#[derive(Clone, Debug)]
pub struct Failure {
    pub unauthorized: bool,
    pub message: String,
}
impl Failure {
    pub(crate) fn message(key: &'static str) -> Self {
        Self {
            unauthorized: false,
            message: text(key).into(),
        }
    }
}
pub async fn request(
    method: &str,
    path: &str,
    token: &str,
    body: Option<&Value>,
) -> Result<Value, Failure> {
    if session_token().ok().flatten().as_deref() != Some(token) {
        return Err(Failure::message("session.changed"));
    }
    let request = match method {
        "GET" => gloo_net::http::Request::get(path),
        "POST" => gloo_net::http::Request::post(path),
        "PUT" => gloo_net::http::Request::put(path),
        "DELETE" => gloo_net::http::Request::delete(path),
        _ => return Err(Failure::message("form.invalid")),
    }
    .header("Authorization", &format!("Bearer {token}"));
    let response = if let Some(body) = body {
        request
            .json(body)
            .map_err(|_| Failure::message("form.invalid"))?
            .send()
            .await
    } else {
        request.send().await
    }
    .map_err(|_| {
        Failure::message(if method == "GET" {
            "screens.network"
        } else {
            "form.unconfirmed"
        })
    })?;
    if response.status() == 401 {
        return Err(Failure {
            unauthorized: true,
            message: text("dashboard.unauthorized").into(),
        });
    }
    if response.status() == 403 {
        return Err(Failure::message("screens.forbidden"));
    }
    if !response.ok() {
        if matches!(response.status(), 400 | 409 | 422) {
            let error: Value = response.json().await.unwrap_or(Value::Null);
            return Err(Failure {
                unauthorized: false,
                message: error
                    .pointer("/error/message")
                    .and_then(Value::as_str)
                    .filter(|s| s.len() <= 1024)
                    .unwrap_or(text("form.invalid"))
                    .into(),
            });
        }
        return Err(Failure::message(if method == "GET" {
            "screens.response"
        } else {
            "form.unconfirmed"
        }));
    }
    response.json().await.map_err(|_| {
        Failure::message(if method == "GET" {
            "screens.response"
        } else {
            "form.unconfirmed"
        })
    })
}
/// The same owner/token must still be active when a response arrives.
#[derive(Clone)]
pub struct Guard {
    alive: std::sync::Arc<std::sync::atomic::AtomicBool>,
    sequence: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}
impl Default for Guard {
    fn default() -> Self {
        Self::new()
    }
}

impl Guard {
    pub fn new() -> Self {
        use leptos::prelude::*;
        let alive = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
        let cleanup = alive.clone();
        on_cleanup(move || cleanup.store(false, std::sync::atomic::Ordering::Relaxed));
        Self {
            alive,
            sequence: Default::default(),
        }
    }
    pub fn begin(&self) -> usize {
        self.sequence
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            .wrapping_add(1)
    }
    pub fn alive(&self, ticket: usize) -> bool {
        self.alive.load(std::sync::atomic::Ordering::Relaxed)
            && self.sequence.load(std::sync::atomic::Ordering::Relaxed) == ticket
    }
    pub fn accepts(&self, ticket: usize, token: &str) -> bool {
        self.alive.load(std::sync::atomic::Ordering::Relaxed)
            && self.sequence.load(std::sync::atomic::Ordering::Relaxed) == ticket
            && session_token().ok().flatten().as_deref() == Some(token)
    }
}

#[derive(Clone, Copy)]
pub struct DirtyScopes(pub leptos::prelude::RwSignal<std::collections::BTreeSet<String>>);
pub fn dirty_scope() -> leptos::prelude::RwSignal<bool> {
    use leptos::prelude::*;
    let registry = expect_context::<DirtyScopes>();
    let dirty = RwSignal::new(false);
    let id = uuid::Uuid::new_v4().to_string();
    let cleanup = id.clone();
    Effect::new(move |_| {
        let changed = dirty.get();
        registry.0.update(|all| {
            if changed {
                all.insert(id.clone());
            } else {
                all.remove(&id);
            }
        });
    });
    on_cleanup(move || {
        let _ = registry.0.try_update(|all| {
            all.remove(&cleanup);
        });
    });
    dirty
}
