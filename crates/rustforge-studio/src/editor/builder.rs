//! Shared article/page builder controls; document mutations use the editor history.
use super::{BLOCKS, History, append_composition};
use crate::messages::text;
use leptos::prelude::*;
use serde_json::Value;

#[component]
pub fn Inserter(
    data: RwSignal<Value>,
    history: RwSignal<History>,
    dirty: RwSignal<bool>,
    busy: RwSignal<bool>,
    forms_available: Memo<bool>,
) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let message = RwSignal::new(None::<String>);
    let insert = Callback::new(move |kind: &'static str| {
        if busy.get_untracked() || (kind == "form_embed" && !forms_available.get_untracked()) {
            return;
        }
        super::ui::change(data, history, dirty, |entry| {
            if let Some(blocks) = entry["blocks"]["rf_blocks"].as_array_mut()
                && blocks.len() < rustforge_document::document::MAX_BLOCKS
            {
                blocks.push(super::block(kind, uuid::Uuid::new_v4().to_string()));
            }
        });
    });
    view! {<details class="rf-builder-inserter"><summary>{text("builder.insert")}</summary>
    <fieldset disabled=move ||busy.get()><legend class="rf-sr-only">{text("builder.insert")}</legend>
    <label for="rf-block-search">{text("builder.search")}</label><input id="rf-block-search" type="search" prop:value=move ||search.get() on:input=move |e|search.set(event_target_value(&e))/>
    <h3>{text("builder.blocks")}</h3><div class="rf-builder-grid">{BLOCKS.iter().map(|&(kind,label)|view!{<button type="button" class="rf-button" hidden=move ||!text(label).to_lowercase().contains(&search.get().to_lowercase()) disabled=move ||kind=="form_embed"&&!forms_available.get() on:click=move |_|insert.run(kind)>{text(label)}</button>}).collect_view()}</div>
    <h3>{text("builder.compositions")}</h3><p class="rf-description">{text("composition.hint")}</p>
    <div class="rf-builder-grid">{rustforge_document::compositions::CATALOG.iter().map(|&(id,label)|view!{<button type="button" class="rf-button" on:click=move |_|{
        match append_composition(&data.get_untracked(),id) {
            Ok(next)=>{super::ui::change(data,history,dirty,|entry|*entry=next);message.set(Some(text("composition.inserted").into()));},
            Err(error)=>message.set(Some(error)),
        }
    }>{text(label)}</button>}).collect_view()}</div>
    {move ||message.get().map(|m|view!{<p role="status">{m}</p>})}
    </fieldset></details>}
}

#[component]
pub fn Template(
    data: RwSignal<Value>,
    history: RwSignal<History>,
    dirty: RwSignal<bool>,
    busy: RwSignal<bool>,
) -> impl IntoView {
    view! {<section><h3>{text("builder.template")}</h3><label for="rf-document-template">{text("builder.template_label")}</label>
    <select id="rf-document-template" disabled=move ||busy.get() prop:value=move ||data.get()["fields"]["_rf_template"].as_str().unwrap_or("default").to_owned() on:change=move |e|{let template=event_target_value(&e);super::ui::change(data,history,dirty,|entry|{if !entry["fields"].is_object(){entry["fields"]=serde_json::json!({});}entry["fields"]["_rf_template"]=serde_json::json!(template);});}>
    <option value="default" selected=move ||data.get()["fields"]["_rf_template"].is_null()||data.get()["fields"]["_rf_template"]=="default">{text("builder.default")}</option>
    <option value="wide" selected=move ||data.get()["fields"]["_rf_template"]=="wide">{text("builder.wide")}</option>
    <option value="landing" selected=move ||data.get()["fields"]["_rf_template"]=="landing">{text("builder.landing")}</option></select>
    <p class="rf-description">{text("builder.template_hint")}</p></section>}
}

#[component]
pub fn Outline(data: RwSignal<Value>) -> impl IntoView {
    use wasm_bindgen::JsCast;
    view! {<details class="rf-builder-inserter"><summary>{text("builder.outline")}</summary><ol class="rf-builder-outline">
        {move ||data.get()["blocks"]["rf_blocks"].as_array().cloned().unwrap_or_default().into_iter().map(|block| {
            let id=format!("rf-block-{}",block["rf_id"].as_str().unwrap_or_default());
            let label=super::BLOCKS.iter().find(|(kind,_)|block["rf_type"].as_str()==Some(kind)).map(|(_,label)|text(label)).unwrap_or(text("editor.block"));
            let snippet=block["rf_text"].as_str().unwrap_or_default().chars().take(60).collect::<String>();
            view!{<li><button type="button" class="rf-button" on:click=move |_| {
                if let Some(element)=web_sys::window().and_then(|w|w.document()).and_then(|d|d.get_element_by_id(&id)).and_then(|e|e.dyn_into::<web_sys::HtmlElement>().ok()) {let _=element.focus();}
            }>{label}{if snippet.is_empty(){String::new()}else{format!(" — {snippet}")}}</button></li>}
        }).collect_view()}
    </ol></details>}
}
