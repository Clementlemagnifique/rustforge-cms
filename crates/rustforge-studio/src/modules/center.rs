use crate::{
    http::{self, Guard},
    messages::text,
};
use leptos::prelude::*;
use serde_json::{Value, json};
fn label(id: &str) -> &'static str {
    match id {
        "forms" => text("module.forms"),
        "galleries" => text("module.galleries"),
        "lms" => text("module.courses"),
        "ecommerce" => text("module.products"),
        "mailing" => text("module.campaigns"),
        "automations" => text("module.recipes"),
        "analytics" => text("module.analytics"),
        _ => text("module.zen"),
    }
}
#[component]
pub fn Center(token: ReadSignal<Option<String>>, on_expired: Callback<()>) -> impl IntoView {
    let catalog = RwSignal::new(Vec::<Value>::new());
    let modules = RwSignal::new(Value::Null);
    let supported = RwSignal::new(Vec::<String>::new());
    let plugins = RwSignal::new(Vec::<Value>::new());
    let pending = RwSignal::new(None::<(String, bool, bool)>);
    let busy = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let refresh = RwSignal::new(0_u64);
    let reload = expect_context::<Callback<()>>();
    let guard = Guard::new();
    let loader = guard.clone();
    Effect::new(move |_| {
        let current = token.get();
        refresh.get();
        let ticket = loader.begin();
        let Some(current) = current else { return };
        let guard = loader.clone();
        busy.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let nav = http::request("GET", "/api/v1/admin/navigation", &current, None).await;
            let manifests =
                http::request("GET", "/api/v1/admin/module-catalog", &current, None).await;
            let wasm = http::request("GET", "/api/v1/plugins", &current, None).await;
            if !guard.accepts(ticket, &current) {
                return;
            }
            busy.set(false);
            if let Ok(value) = manifests {
                catalog.set(serde_json::from_value(value).unwrap_or_default());
            }
            match nav {
                Ok(value) => {
                    modules.set(value["modules"].clone());
                    supported.set(
                        serde_json::from_value(value["supported_modules"].clone())
                            .unwrap_or_default(),
                    );
                }
                Err(error) => {
                    message.set(Some(error.message));
                    if error.unauthorized {
                        on_expired.run(());
                    }
                }
            }
            if let Ok(value) = wasm {
                plugins.set(serde_json::from_value(value).unwrap_or_default());
            }
        });
    });
    let apply = move |_| {
        let Some(current) = token.get_untracked() else {
            return;
        };
        let Some((id, enabled, wasm)) = pending.get_untracked() else {
            return;
        };
        if busy.get_untracked() {
            return;
        }
        if !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        {
            return;
        }
        let ticket = guard.begin();
        let guard = guard.clone();
        busy.set(true);
        pending.set(None);
        wasm_bindgen_futures::spawn_local(async move {
            let result = if wasm {
                http::request(
                    "POST",
                    &format!(
                        "/api/v1/plugins/{id}/{}",
                        if enabled { "enable" } else { "disable" }
                    ),
                    &current,
                    None,
                )
                .await
            } else {
                http::request(
                    "PUT",
                    "/api/v1/admin/modules",
                    &current,
                    Some(&json!({id:enabled})),
                )
                .await
            };
            if !guard.accepts(ticket, &current) {
                return;
            }
            busy.set(false);
            match result {
                Ok(_) => {
                    reload.run(());
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
    view! {<h1>{text("module.center")}</h1><p>{text("module.center_hint")}</p>{move ||message.get().map(|message|view!{<p class="rf-notice" role="status">{message}</p>})}<Show when=move ||pending.get().is_some()><div class="rf-notice"><p>{text("module.toggle_confirm")}</p><button class="rf-button primary" disabled=move ||busy.get() on:click=apply.clone()>{text("module.confirm")}</button><button class="rf-button" on:click=move |_|pending.set(None)>{text("form.cancel")}</button></div></Show><div class="rf-table-scroll"><table class="rf-list-table"><thead><tr><th>{text("module.name")}</th><th>{text("screens.status")}</th><th>{text("module.action")}</th></tr></thead><tbody>{move ||supported.get().into_iter().map(|id|{let enabled=modules.get()[&id].as_bool().unwrap_or(false);view!{<tr><th scope="row">{label(&id)}{catalog.get().into_iter().find(|manifest|manifest["id"].as_str()==Some(&id)).map(|manifest|view!{<p class="rf-description">{text("blocks.reference")}" "{manifest["references"].as_array().map(|values|values.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(", ")).unwrap_or_default()}</p>})}</th><td>{if enabled{text("module.enabled")}else{text("module.disabled")}}</td><td><button class="rf-button" disabled=move ||busy.get() on:click=move |_|pending.set(Some((id.clone(),!enabled,false)))>{if enabled{text("module.disable")}else{text("module.enable")}}</button></td></tr>}}).collect_view()}</tbody></table></div><h2>{text("module.wasm")}</h2><div class="rf-screen-grid">{move ||plugins.get().into_iter().map(|plugin|{let id=plugin["id"].as_str().unwrap_or_default().to_owned();let enabled=plugin["enabled"].as_bool().unwrap_or(false);view!{<article class="rf-panel rf-screen-panel"><h3>{plugin["name"].as_str().unwrap_or_default().to_owned()}</h3><p>{plugin["description"].as_str().unwrap_or_default().to_owned()}</p><p>{text("module.capabilities")} " "{plugin["capabilities"].as_array().map(|items|items.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(", ")).unwrap_or_default()}</p><button class="rf-button" disabled=move ||busy.get() on:click=move |_|pending.set(Some((id.clone(),!enabled,true)))>{if enabled{text("module.disable")}else{text("module.enable")}}</button></article>}}).collect_view()}</div><crate::block_catalog::ui::Catalog/><crate::block_catalog::ui::Adjustments/>}
}
