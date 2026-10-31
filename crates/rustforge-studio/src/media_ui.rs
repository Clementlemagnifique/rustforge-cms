use crate::{
    http::{self, Guard},
    messages::text,
};
use leptos::prelude::*;
use serde_json::{Value, json};
#[component]
pub fn Upload(
    token: ReadSignal<Option<String>>,
    on_expired: Callback<()>,
    on_uploaded: Callback<()>,
) -> impl IntoView {
    let input = NodeRef::<leptos::html::Input>::new();
    let busy = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let guard = Guard::new();
    let upload = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        if busy.get_untracked() {
            return;
        }
        let Some(current) = token.get_untracked() else {
            return;
        };
        let Some(file) = input
            .get()
            .and_then(|input| input.files())
            .and_then(|files| files.get(0))
        else {
            return;
        };
        let form = match web_sys::FormData::new() {
            Ok(form) => form,
            Err(_) => return,
        };
        if form
            .append_with_blob_and_filename("file", &file, &file.name())
            .is_err()
        {
            return;
        }
        let ticket = guard.begin();
        let guard = guard.clone();
        busy.set(true);
        message.set(None);
        wasm_bindgen_futures::spawn_local(async move {
            let result = async {
                let response = gloo_net::http::Request::post("/api/v1/media")
                    .header("Authorization", &format!("Bearer {current}"))
                    .body(form)
                    .map_err(|_| text("form.invalid"))?
                    .send()
                    .await
                    .map_err(|_| text("form.unconfirmed"))?;
                if response.status() == 401 {
                    return Err(text("dashboard.unauthorized"));
                }
                if !response.ok() {
                    return Err(text("media.upload_failed"));
                }
                response
                    .json::<Value>()
                    .await
                    .map_err(|_| text("form.unconfirmed"))
            }
            .await;
            if !guard.accepts(ticket, &current) {
                return;
            }
            busy.set(false);
            match result {
                Ok(_) => {
                    if let Some(input) = input.get() {
                        input.set_value("");
                    }
                    message.set(Some(text("media.uploaded").into()));
                    on_uploaded.run(());
                }
                Err(error) => {
                    message.set(Some(error.into()));
                    if error == text("dashboard.unauthorized") {
                        on_expired.run(());
                    }
                }
            }
        });
    };
    view! {<details class="rf-panel rf-screen-panel rf-layout-options"><summary>{text("media.add")}</summary><form class="rf-form-actions" on:submit=upload><label for="rf-upload-file">{text("media.choose")}</label><input id="rf-upload-file" type="file" node_ref=input required disabled=move ||busy.get()/><button class="rf-button primary" disabled=move ||busy.get()||token.get().is_none()>{move ||if busy.get(){text("media.uploading")}else{text("media.add")}}</button></form>{move ||message.get().map(|message|view!{<p role="status">{message}</p>})}</details>}
}
#[component]
pub fn Picker(
    token: ReadSignal<Option<String>>,
    on_choose: Callback<Value>,
    #[prop(default = false)] images_only: bool,
) -> impl IntoView {
    let open = RwSignal::new(false);
    let items = RwSignal::new(Vec::<Value>::new());
    let busy = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let page = RwSignal::new(1_u64);
    let pages = RwSignal::new(1_u64);
    let guard = Guard::new();
    Effect::new(move |_| {
        let current = token.get();
        let page = page.get();
        if !open.get() {
            return;
        }
        let Some(current) = current else { return };
        let ticket = guard.begin();
        let guard = guard.clone();
        busy.set(true);
        message.set(None);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request(
                "GET",
                &format!("/api/v1/media?page={page}&per_page=20"),
                &current,
                None,
            )
            .await;
            if !guard.accepts(ticket, &current) {
                return;
            }
            busy.set(false);
            match result {
                Ok(value) => {
                    items.set(value["items"].as_array().cloned().unwrap_or_default());
                    pages.set(value["total_pages"].as_u64().unwrap_or(1).max(1));
                }
                Err(error) => message.set(Some(error.message)),
            }
        });
    });
    view! {<button class="rf-button" on:click=move |_|open.update(|open|*open = !*open) aria-expanded=move ||open.get().to_string()>{text("media.library")}</button><Show when=move ||open.get()><section class="rf-media-picker" aria-label=text("media.library")>{move ||message.get().map(|message|view!{<p role="alert">{message}</p>})}<ul>{move ||items.get().into_iter().filter(|item|item["url"].as_str().is_some_and(|url|url.starts_with("/uploads/"))&&(!images_only||item["mime_type"].as_str().is_some_and(|mime|matches!(mime,"image/jpeg"|"image/png"|"image/gif"|"image/webp"|"image/avif"|"image/svg+xml")))).map(|item|{let title=item["original_name"].as_str().unwrap_or_default().to_owned();view!{<li><button class="rf-button" disabled=move ||busy.get() on:click=move |_|{on_choose.run(item.clone());open.set(false);}>{title}</button></li>}}).collect_view()}</ul><button class="rf-button" disabled=move ||busy.get()||page.get()<=1 on:click=move |_|page.update(|page|*page-=1)>{text("screens.previous")}</button><button class="rf-button" disabled={move ||busy.get()||page.get()>=pages.get()} on:click=move |_|page.update(|page|*page+=1)>{text("screens.next")}</button></section></Show>}
}
#[component]
pub fn Detail(
    id: String,
    token: ReadSignal<Option<String>>,
    on_expired: Callback<()>,
) -> impl IntoView {
    let data = RwSignal::new(Value::Null);
    let alt = RwSignal::new(String::new());
    let caption = RwSignal::new(String::new());
    let dirty = crate::http::dirty_scope();
    let message = RwSignal::new(None::<String>);
    let busy = RwSignal::new(false);
    let guard = Guard::new();
    let save_id = id.clone();
    let loader = guard.clone();
    Effect::new(move |_| {
        let Some(current) = token.get() else {
            data.set(Value::Null);
            return;
        };
        let id = id.clone();
        let ticket = loader.begin();
        let guard = loader.clone();
        busy.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request("GET", &format!("/api/v1/media/{id}"), &current, None).await;
            if !guard.accepts(ticket, &current) {
                return;
            }
            busy.set(false);
            match result {
                Ok(value) => {
                    alt.set(value["alt"].as_str().unwrap_or_default().into());
                    caption.set(value["caption"].as_str().unwrap_or_default().into());
                    data.set(value);
                    dirty.set(false);
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
    let save = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        if busy.get_untracked() {
            return;
        }
        let Some(current) = token.get_untracked() else {
            return;
        };
        let id = save_id.clone();
        let ticket = guard.begin();
        let guard = guard.clone();
        let body = json!({"alt":alt.get_untracked(),"caption":caption.get_untracked()});
        busy.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let result =
                http::request("PUT", &format!("/api/v1/media/{id}"), &current, Some(&body)).await;
            if !guard.accepts(ticket, &current) {
                return;
            }
            busy.set(false);
            match result {
                Ok(value) => {
                    data.set(value);
                    dirty.set(false);
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
    view! {<h1>{text("media.details")}</h1><a class="rf-button" href="#media">{text("editor.back")}</a>{move ||message.get().map(|message|view!{<p class="rf-notice" role="status">{message}</p>})}<Show when=move ||!data.get().is_null()><div class="rf-screen-grid"><section class="rf-panel rf-screen-panel"><h2>{move ||data.get()["original_name"].as_str().unwrap_or_default().to_owned()}</h2>{move ||data.get()["url"].as_str().filter(|url|url.starts_with("/uploads/")).map(|url|view!{<a href=url.to_owned() target="_blank" rel="noopener noreferrer">{text("media.open")}</a>})}<p>{move ||data.get()["mime_type"].as_str().unwrap_or_default().to_owned()}</p></section><form on:submit=save.clone()><fieldset class="rf-form" disabled=move ||busy.get()><label for="rf-media-alt">{text("editor.alt")}</label><input id="rf-media-alt" maxlength="512" prop:value=move ||alt.get() on:input=move |event|{alt.set(event_target_value(&event));dirty.set(true);}/><label for="rf-media-caption">{text("editor.caption")}</label><textarea id="rf-media-caption" maxlength="1024" prop:value=move ||caption.get() on:input=move |event|{caption.set(event_target_value(&event));dirty.set(true);}/><button class="rf-button primary" disabled=move ||!dirty.get()>{text("form.save")}</button></fieldset></form></div></Show>}
}
