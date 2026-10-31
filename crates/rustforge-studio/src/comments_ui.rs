use crate::{
    http::{self, Guard},
    messages::text,
};
use leptos::prelude::*;
use serde_json::{Value, json};
#[component]
pub fn Comments(token: ReadSignal<Option<String>>, on_expired: Callback<()>) -> impl IntoView {
    let rows = RwSignal::new(Vec::<Value>::new());
    let state = RwSignal::new(String::new());
    let page = RwSignal::new(0_u64);
    let busy = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let refresh = RwSignal::new(0_u64);
    let pending = RwSignal::new(None::<(String, &'static str)>);
    let guard = Guard::new();
    let loader = guard.clone();
    Effect::new(move |_| {
        let token = token.get();
        let status = state.get();
        let page = page.get();
        refresh.get();
        let ticket = loader.begin();
        let Some(token) = token else {
            rows.set(vec![]);
            return;
        };
        let filter = if status.is_empty() {
            String::new()
        } else {
            format!("&status={status}")
        };
        let guard = loader.clone();
        busy.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request(
                "GET",
                &format!("/api/v1/comments?limit=20&offset={}{filter}", page * 20),
                &token,
                None,
            )
            .await;
            if !guard.accepts(ticket, &token) {
                return;
            }
            busy.set(false);
            match result {
                Ok(value) => rows.set(value["items"].as_array().cloned().unwrap_or_default()),
                Err(error) => {
                    message.set(Some(error.message));
                    if error.unauthorized {
                        on_expired.run(());
                    }
                }
            }
        });
    });
    let apply = move |_| {
        let Some(token) = token.get_untracked() else {
            return;
        };
        let Some((id, status)) = pending.get_untracked() else {
            return;
        };
        if busy.get_untracked() {
            return;
        }
        let ticket = guard.begin();
        let guard = guard.clone();
        busy.set(true);
        pending.set(None);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request(
                "PUT",
                &format!("/api/v1/comments/{id}/status"),
                &token,
                Some(&json!({"status":status})),
            )
            .await;
            if !guard.accepts(ticket, &token) {
                return;
            }
            busy.set(false);
            match result {
                Ok(_) => refresh.update(|n| *n = n.wrapping_add(1)),
                Err(error) => {
                    message.set(Some(error.message));
                    if error.unauthorized {
                        on_expired.run(());
                    }
                }
            }
        });
    };
    view! {<h1>{text("comments.title")}</h1><label for="rf-comments-filter">{text("list.filter")}</label><select id="rf-comments-filter" prop:value=move ||state.get() on:change=move |event|{state.set(event_target_value(&event));page.set(0);}>
    <option value="">{text("list.all")}</option><option value="pending">{text("comments.pending")}</option><option value="approved">{text("comments.approved")}</option><option value="spam">{text("comments.spam")}</option><option value="trash">{text("dashboard.trash")}</option></select>{move ||message.get().map(|message|view!{<p class="rf-notice" role="status">{message}</p>})}<Show when=move ||pending.get().is_some()><div class="rf-notice"><p>{text("comments.confirm")}</p><button class="rf-button" disabled=move ||busy.get() on:click=apply.clone()>{text("module.confirm")}</button><button class="rf-button" on:click=move |_|pending.set(None)>{text("form.cancel")}</button></div></Show><div class="rf-table-scroll"><table class="rf-list-table"><thead><tr><th>{text("screens.author")}</th><th>{text("editor.content")}</th><th>{text("screens.status")}</th><th>{text("module.action")}</th></tr></thead><tbody>{move ||rows.get().into_iter().map(|row|{let id=StoredValue::new(row["id"].as_str().unwrap_or_default().to_owned());view!{<tr><th scope="row">{row["author_name"].as_str().unwrap_or_default().to_owned()}</th><td class="rf-capture-body">{row["content"].as_str().unwrap_or_default().to_owned()}</td><td>{row["status"].as_str().unwrap_or_default().to_owned()}</td><td>{[("approved","comments.approve"),("pending","comments.hold"),("spam","comments.spam"),("trash","list.trash")].map(|(status,label)|view!{<button class="rf-text-button" disabled=move ||busy.get() on:click=move |_|pending.set(Some((id.get_value(),status)))>{text(label)}</button>})}</td></tr>}}).collect_view()}</tbody></table></div><div class="rf-form-actions"><button class="rf-button" disabled=move ||busy.get()||page.get()==0 on:click=move |_|page.update(|page|*page=page.saturating_sub(1))>{text("screens.previous")}</button><button class="rf-button" disabled=move ||busy.get()||rows.get().len()<20 on:click=move |_|page.update(|page|*page+=1)>{text("screens.next")}</button></div>}
}
