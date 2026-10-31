use crate::{
    http::{self, Guard},
    messages::text,
};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Deserialize, Serialize)]
pub struct Preferences {
    pub version: u8,
    pub dashboard: Layout,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct Layout {
    pub order: Vec<String>,
    pub hidden: Vec<String>,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            version: 1,
            dashboard: Layout {
                order: ["overview", "activity", "health", "draft"]
                    .map(str::to_owned)
                    .to_vec(),
                hidden: vec![],
            },
        }
    }
}
pub fn label(id: &str) -> &'static str {
    match id {
        "overview" => text("dashboard.overview"),
        "activity" => text("dashboard.activity"),
        "health" => text("dashboard.health"),
        _ => text("dashboard.quick_draft"),
    }
}
#[component]
pub fn LayoutControls(
    token: ReadSignal<Option<String>>,
    on_expired: Callback<()>,
) -> impl IntoView {
    let prefs = expect_context::<RwSignal<Preferences>>();
    let ready = RwSignal::new(false);
    let busy = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let changed = crate::http::dirty_scope();
    let guard = Guard::new();
    let loader = guard.clone();
    Effect::new(move |_| {
        let token = token.get();
        let ticket = loader.begin();
        ready.set(false);
        prefs.set(Preferences::default());
        let Some(token) = token else { return };
        let guard = loader.clone();
        busy.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request("GET", "/api/v1/admin/preferences", &token, None).await;
            if !guard.accepts(ticket, &token) {
                return;
            }
            busy.set(false);
            match result {
                Ok(data) => {
                    if let Ok(value) = serde_json::from_value::<Preferences>(data) {
                        prefs.set(value);
                        ready.set(true);
                    } else {
                        message.set(Some(text("screens.response").into()));
                    }
                }
                Err(error) => {
                    message.set(Some(error.message));
                    if error.unauthorized {
                        on_expired.run(());
                    }
                }
            }
        });
    });
    let save = move |_| {
        let Some(token) = token.get_untracked() else {
            return;
        };
        if busy.get_untracked() || !ready.get_untracked() {
            return;
        }
        let body = serde_json::to_value(prefs.get_untracked()).unwrap_or_default();
        let ticket = guard.begin();
        let guard = guard.clone();
        busy.set(true);
        message.set(None);
        wasm_bindgen_futures::spawn_local(async move {
            let result =
                http::request("PUT", "/api/v1/admin/preferences", &token, Some(&body)).await;
            if !guard.accepts(ticket, &token) {
                return;
            }
            busy.set(false);
            match result {
                Ok(_) => {
                    changed.set(false);
                    message.set(Some(text("form.saved").into()));
                }
                Err(error) => {
                    message.set(Some(error.message));
                    if error.unauthorized {
                        on_expired.run(());
                    }
                }
            }
        });
    };
    view! {<details class="rf-panel rf-screen-panel rf-layout-options"><summary>{text("layout.title")}</summary><p>{text("layout.hint")}</p><fieldset disabled=move ||busy.get()||!ready.get()><legend class="rf-sr-only">{text("layout.title")}</legend><ol>{move ||prefs.get().dashboard.order.into_iter().enumerate().map(|(index,id)|{let visible_id=id.clone();let up_id=id.clone();let down_id=id.clone();view!{<li><label><input type="checkbox" prop:checked=move ||!prefs.get().dashboard.hidden.contains(&visible_id) on:change=move |event|{let show=event_target_checked(&event);prefs.update(|prefs|{prefs.dashboard.hidden.retain(|value|value!=&id);if !show{prefs.dashboard.hidden.push(id.clone());}});changed.set(true);}/>{label(&up_id)}</label><button class="rf-button" disabled=index==0 aria-label=format!("{} {}",text("layout.up"),label(&up_id)) on:click=move |_|{prefs.update(|prefs|{if let Some(index)=prefs.dashboard.order.iter().position(|id|id==&up_id)&& index>0{prefs.dashboard.order.swap(index,index-1);}});changed.set(true);}>{text("layout.up")}</button><button class="rf-button" disabled=index==3 aria-label=format!("{} {}",text("layout.down"),label(&down_id)) on:click=move |_|{prefs.update(|prefs|{if let Some(index)=prefs.dashboard.order.iter().position(|id|id==&down_id)&& index+1<prefs.dashboard.order.len(){prefs.dashboard.order.swap(index,index+1);}});changed.set(true);}>{text("layout.down")}</button></li>}}).collect_view()}</ol><button class="rf-button primary" disabled=move ||!changed.get() on:click=save>{text("form.save")}</button><button class="rf-button" on:click=move |_|{prefs.set(Preferences::default());changed.set(true);}>{text("layout.reset")}</button></fieldset>{move ||message.get().map(|message|view!{<p role="status">{message}</p>})}</details>}
}
