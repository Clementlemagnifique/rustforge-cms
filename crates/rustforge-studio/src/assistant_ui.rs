use crate::{
    http::{self, Guard},
    messages::text,
};
use leptos::prelude::*;
use serde_json::{Value, json};
#[component]
pub fn Assistant(token: ReadSignal<Option<String>>, on_expired: Callback<()>) -> impl IntoView {
    let prompt = RwSignal::new(String::new());
    let turns = RwSignal::new(Vec::<Value>::new());
    let busy = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let status = RwSignal::new(None::<Value>);
    let guard = Guard::new();
    let status_guard = guard.clone();
    let dirty = crate::http::dirty_scope();
    Effect::new(move |_| dirty.set(!prompt.get().is_empty()));
    let check = move |_| {
        let Some(token) = token.get_untracked() else {
            return;
        };
        if busy.get_untracked() {
            return;
        }
        let guard = status_guard.clone();
        let ticket = guard.begin();
        busy.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request("GET", "/api/v1/ai/status", &token, None).await;
            if !guard.accepts(ticket, &token) {
                return;
            }
            busy.set(false);
            match result {
                Ok(value) => status.set(Some(value)),
                Err(error) => {
                    message.set(Some(error.message));
                    if error.unauthorized {
                        on_expired.run(());
                    }
                }
            }
        });
    };
    let submit = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        if busy.get_untracked() {
            return;
        }
        let Some(token) = token.get_untracked() else {
            return;
        };
        let input = prompt.get_untracked();
        if input.trim().is_empty() {
            return;
        }
        let mut conversation = turns.get_untracked();
        conversation.push(json!({"role":"user","content":input}));
        if conversation.len() > 20 {
            conversation.drain(..conversation.len() - 20);
        }
        let body = json!({"messages":conversation});
        let guard = guard.clone();
        let ticket = guard.begin();
        busy.set(true);
        message.set(None);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request("POST", "/api/v1/ai/chat", &token, Some(&body)).await;
            if !guard.accepts(ticket, &token) {
                return;
            }
            busy.set(false);
            match result {
                Ok(value) => {
                    conversation.push(json!({"role":"assistant","content":value["reply"].as_str().unwrap_or_default()}));
                    turns.set(conversation);
                    prompt.set(String::new());
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
    view! {<h1>{text("assistant.title")}</h1><p class="rf-notice">{text("assistant.hint")}</p><button class="rf-button" disabled=move ||busy.get() on:click=check>{text("assistant.check")}</button>{move ||status.get().map(|status|view!{<dl><dt>{text("assistant.provider")}</dt><dd>{status["provider"].as_str().unwrap_or_default().to_owned()}</dd><dt>{text("assistant.model")}</dt><dd>{status["model"].as_str().unwrap_or_default().to_owned()}</dd><dt>{text("assistant.available")}</dt><dd>{if status["reachable"].as_bool()==Some(true){text("assistant.yes")}else{text("assistant.no")}}</dd></dl>})}{move ||message.get().map(|message|view!{<p class="rf-notice" role="alert">{message}</p>})}<div aria-live="polite">{move ||turns.get().into_iter().map(|turn|view!{<article class="rf-panel rf-screen-panel"><h2>{if turn["role"]=="user"{text("assistant.you")}else{text("assistant.title")}}</h2><p class="rf-capture-body">{turn["content"].as_str().unwrap_or_default().to_owned()}</p></article>}).collect_view()}</div><form class="rf-compact-form" on:submit=submit><label for="rf-ai-prompt">{text("assistant.request")}</label><textarea id="rf-ai-prompt" required maxlength="5000" prop:value=move ||prompt.get() on:input=move |event|prompt.set(event_target_value(&event)) disabled=move ||busy.get()/><button class="rf-button primary" disabled=move ||busy.get()||token.get().is_none()>{text("assistant.send")}</button></form>}
}
#[component]
pub fn Login(on_ready: Callback<()>) -> impl IntoView {
    let login = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let busy = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let guard = Guard::new();
    let submit = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        if busy.get_untracked() {
            return;
        }
        let previous = crate::app::session_token().ok().flatten();
        let login = login.get_untracked();
        let body = json!({"login":login,"password":password.get_untracked()});
        let ticket = guard.begin();
        let guard = guard.clone();
        busy.set(true);
        message.set(None);
        wasm_bindgen_futures::spawn_local(async move {
            let result = async {
                let response = gloo_net::http::Request::post("/api/v1/auth/login")
                    .json(&body)
                    .map_err(|_| text("form.invalid"))?
                    .send()
                    .await
                    .map_err(|_| text("screens.network"))?;
                if !response.ok() {
                    return Err(if response.status() == 429 {
                        text("login.rate")
                    } else {
                        text("login.failed")
                    });
                }
                let value = response
                    .json::<Value>()
                    .await
                    .map_err(|_| text("screens.response"))?;
                let token = value["token"]
                    .as_str()
                    .filter(|value| !value.is_empty())
                    .ok_or(text("screens.response"))?;
                let storage = web_sys::window()
                    .ok_or(text("error.browser"))?
                    .local_storage()
                    .ok()
                    .flatten()
                    .ok_or(text("error.storage"))?;
                if !guard.alive(ticket) || crate::app::session_token().ok().flatten() != previous {
                    return Err(text("session.changed"));
                }
                storage
                    .set_item("rustforge_token", token)
                    .map_err(|_| text("error.storage"))?;
                Ok(())
            }
            .await;
            // A login has no active token yet; only owner liveness and request generation apply.
            if !guard.alive(ticket) {
                return;
            }
            busy.set(false);
            password.set(String::new());
            match result {
                Ok(_) => on_ready.run(()),
                Err(error) => message.set(Some(error.into())),
            }
        });
    };
    view! {<section class="rf-panel rf-screen-panel rf-login-panel"><h2>{text("login.title")}</h2>{move ||message.get().map(|message|view!{<p role="alert">{message}</p>})}<form class="rf-compact-form" on:submit=submit><label for="rf-login">{text("login.identity")}</label><input id="rf-login" autocomplete="username" required prop:value=move ||login.get() on:input=move |event|login.set(event_target_value(&event)) disabled=move ||busy.get()/><label for="rf-password">{text("form.password")}</label><input id="rf-password" type="password" autocomplete="current-password" required prop:value=move ||password.get() on:input=move |event|password.set(event_target_value(&event)) disabled=move ||busy.get()/><button class="rf-button primary" disabled=move ||busy.get()>{text("login.submit")}</button></form></section>}
}
