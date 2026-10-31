use crate::{
    http::{self, Guard},
    messages::text,
};
use leptos::prelude::*;
use serde_json::Value;
#[component]
pub fn FormBlock(block: Memo<Value>, on_choose: Callback<String>) -> impl IntoView {
    let token = expect_context::<ReadSignal<Option<String>>>();
    let rows = RwSignal::new(Vec::<Value>::new());
    let busy = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let guard = Guard::new();
    Effect::new(move |_| {
        let current = token.get();
        let guard = guard.clone();
        let ticket = guard.begin();
        let Some(current) = current else { return };
        busy.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request("GET", "/api/v1/forms", &current, None).await;
            if !guard.accepts(ticket, &current) {
                return;
            }
            busy.set(false);
            match result {
                Ok(value) => match serde_json::from_value::<Vec<Value>>(value) {
                    Ok(value) => {
                        rows.set(
                            value
                                .into_iter()
                                .filter(|row| row["is_active"].as_bool() == Some(true))
                                .collect(),
                        );
                        message.set(None);
                    }
                    Err(_) => message.set(Some(text("screens.response").into())),
                },
                Err(error) => message.set(Some(error.message)),
            }
        });
    });
    view! {<p>{text("form_block.hint")}</p>{move ||message.get().map(|message|view!{<p role="status">{message}</p>})}<label>{text("form_block.select")}<select disabled=move ||busy.get()||message.get().is_some() prop:value=move ||block.get()["rf_ref"].as_str().unwrap_or_default().to_owned() on:change=move |event|on_choose.run(event_target_value(&event))><option value="">{text("form_block.choose")}</option>{move ||{let current=block.get()["rf_ref"].as_str().unwrap_or_default().to_owned();let missing=!current.is_empty()&&!rows.get().iter().any(|row|row["id"].as_str()==Some(&current));view!{{missing.then(||view!{<option value=current selected=true>{text("form_block.missing")}</option>})}{rows.get().into_iter().map(|row|{let id=StoredValue::new(row["id"].as_str().unwrap_or_default().to_owned());view!{<option value=id.get_value() selected=move ||block.get()["rf_ref"].as_str()==Some(&id.get_value())>{row["name"].as_str().unwrap_or_default().to_owned()}</option>}}).collect_view()}}}}</select></label>
    {move ||block.get()["rf_ref"].as_str().filter(|id|super::local_id(id)).map(|id|view!{<a class="rf-button" href=format!("/forms/{id}") target="_blank" rel="noopener noreferrer">{text("form_block.preview")}</a>})}
    <p class="rf-description">{text("form_block.reference")}</p>}
}
