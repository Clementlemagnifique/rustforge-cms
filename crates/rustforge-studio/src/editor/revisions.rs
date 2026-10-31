//! Revision comparison and restoration through the existing versioned server contract.
use crate::{
    http::{self, Guard},
    messages::text,
};
use leptos::prelude::*;
use serde_json::Value;

#[component]
pub fn RevisionHistory(
    data: RwSignal<Value>,
    dirty: RwSignal<bool>,
    busy: RwSignal<bool>,
    token: ReadSignal<Option<String>>,
    on_expired: Callback<()>,
    on_restored: Callback<Value>,
) -> impl IntoView {
    let rows = RwSignal::new(Vec::<Value>::new());
    let selected = RwSignal::new(None::<Value>);
    let confirming = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let loaded = RwSignal::new(false);
    let refresh = RwSignal::new(0_u32);
    let identity = Memo::new(move |_| (data.get()["id"].clone(), data.get()["updated_at"].clone()));
    let guard = Guard::new();
    let loader = guard.clone();
    Effect::new(move |_| {
        let (id, _) = identity.get();
        let current = token.get();
        refresh.get();
        rows.set(vec![]);
        selected.set(None);
        confirming.set(false);
        loaded.set(false);
        let Some(id) = id
            .as_str()
            .filter(|id| super::local_id(id))
            .map(str::to_owned)
        else {
            return;
        };
        let Some(current) = current else { return };
        let guard = loader.clone();
        let ticket = guard.begin();
        busy.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request(
                "GET",
                &format!("/api/v1/entries/{id}/revisions"),
                &current,
                None,
            )
            .await;
            if !guard.accepts(ticket, &current) {
                return;
            }
            busy.set(false);
            match result {
                Ok(value) => match serde_json::from_value::<Vec<Value>>(value) {
                    Ok(value) => {
                        rows.set(value);
                        loaded.set(true);
                    }
                    Err(_) => message.set(Some(text("screens.response").into())),
                },
                Err(error) => {
                    message.set(Some(error.message));
                    if error.unauthorized {
                        on_expired.run(());
                    }
                }
            }
        });
    });
    let restore_guard = guard.clone();
    let restore = Callback::new(move |_: ()| {
        if busy.get_untracked() || dirty.get_untracked() || !confirming.get_untracked() {
            return;
        }
        let current_entry = data.get_untracked();
        let Some(revision) = selected.get_untracked() else {
            return;
        };
        let Some((path, body)) = super::revision_restore(&current_entry, &revision) else {
            message.set(Some(text("screens.response").into()));
            return;
        };
        let Some(current) = token.get_untracked() else {
            return;
        };
        let guard = restore_guard.clone();
        let ticket = guard.begin();
        busy.set(true);
        confirming.set(false);
        message.set(None);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request("POST", &path, &current, Some(&body)).await;
            if !guard.accepts(ticket, &current) {
                return;
            }
            busy.set(false);
            match result {
                Ok(value)
                    if value["id"] == current_entry["id"] && value["updated_at"].is_string() =>
                {
                    on_restored.run(value);
                    message.set(Some(text("revision.restored").into()));
                }
                Ok(_) => message.set(Some(text("form.unconfirmed").into())),
                Err(error) => {
                    message.set(Some(error.message));
                    if error.unauthorized {
                        on_expired.run(());
                    }
                }
            }
        });
    });
    view! {
        <section class="rf-panel rf-screen-panel rf-revisions" aria-label=text("editor.revisions")>
            <h2>{text("editor.revisions")}</h2>
            <p>{text("revision.help")}</p>
            <button class="rf-button" disabled=move ||busy.get() on:click=move |_|{message.set(None);refresh.update(|value|*value=value.wrapping_add(1));}>{text("dashboard.refresh")}</button>
            {move ||message.get().map(|message|view!{<p class="rf-notice" role="status">{message}</p>})}
            <Show when=move ||loaded.get()&&rows.get().is_empty()><p>{text("revision.empty")}</p></Show>
            <ul class="rf-revision-list">{move ||rows.get().into_iter().map(|revision|{
                let label=format!("{} · {} · {}",revision["created_at"].as_str().unwrap_or("—"),revision["author_name"].as_str().unwrap_or("—"),revision["title"].as_str().unwrap_or_default());
                let id=revision["id"].clone();
                view!{<li><button class="rf-text-button" disabled=move ||busy.get() aria-pressed=move ||selected.get().is_some_and(|row|row["id"]==id).to_string() on:click=move |_|{selected.set(Some(revision.clone()));confirming.set(false);}>{label}</button></li>}
            }).collect_view()}</ul>
            {move ||selected.get().map(|revision|view!{
                <div class="rf-revision-comparison">
                    <RevisionColumn heading="revision.previous" value=revision/>
                    <RevisionCurrent data/>
                </div>
            })}
            <Show when=move ||selected.get().is_some()>
                <p class="rf-description">{text("revision.scope")}</p>
                <Show when=move ||dirty.get()><p class="rf-notice">{text("revision.unsaved")}</p></Show>
                <Show when=move ||!confirming.get() fallback=move ||view!{
                    <div class="rf-notice"><p>{text("revision.confirm")}</p>
                        <button class="rf-button primary" disabled=move ||busy.get()||dirty.get() on:click=move |_|restore.run(())>{text("revision.confirm_button")}</button>
                        <button class="rf-button" disabled=move ||busy.get() on:click=move |_|confirming.set(false)>{text("form.cancel")}</button>
                    </div>
                }><button class="rf-button" disabled=move ||busy.get()||dirty.get() on:click=move |_|confirming.set(true)>{text("revision.restore")}</button></Show>
            </Show>
        </section>
    }
}
#[component]
fn RevisionCurrent(data: RwSignal<Value>) -> impl IntoView {
    view! {<div>{move ||view!{<RevisionColumn heading="revision.current" value=data.get()/>}}</div>}
}
#[component]
fn RevisionColumn(heading: &'static str, value: Value) -> impl IntoView {
    let snapshot = value
        .get("snapshot")
        .filter(|value| value.is_object())
        .unwrap_or(&value);
    let projected = super::projected(snapshot)
        .ok()
        .map(|(body, _)| body)
        .unwrap_or_else(|| value["body"].as_str().unwrap_or_default().to_owned());
    let blocks = snapshot
        .get("blocks")
        .and_then(|blocks| serde_json::to_string_pretty(blocks).ok());
    view! {<section class="rf-revision-version"><h3>{text(heading)}</h3><h4>{value["title"].as_str().unwrap_or_default().to_owned()}</h4><p>{value["excerpt"].as_str().unwrap_or_default().to_owned()}</p>
        <dl>{[("slug","editor.slug"),("status","screens.status"),("published_at","editor.publish_date"),("scheduled_unpublish_at","editor.unpublish_date")].into_iter().map(|(key,label)|{let value=if key=="status"{text(crate::dashboard::state_key(snapshot[key].as_str().unwrap_or_default())).to_owned()}else{snapshot[key].as_str().unwrap_or("—").to_owned()};view!{<dt>{text(label)}</dt><dd>{value}</dd>}}).collect_view()}</dl>
        <pre>{projected}</pre>
        {blocks.map(|blocks|view!{<details><summary>{text("revision.blocks")}</summary><pre>{blocks}</pre></details>})}
    </section>}
}
