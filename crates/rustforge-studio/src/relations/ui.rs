use crate::{
    editor::{History, ui::change},
    http::{self, Guard},
    messages::text,
};
use leptos::prelude::*;
use serde_json::{Value, json};
/// Read only the pages visible to the current account, with an explicit bounded limit.
pub async fn pages(token: &str) -> Result<Vec<Value>, http::Failure> {
    let mut rows = vec![];
    for page in 1..=100 {
        let value = http::request(
            "GET",
            &format!("/api/v1/entries?content_type=page&per_page=100&order=title&page={page}"),
            token,
            None,
        )
        .await?;
        let Some(items) = value["items"].as_array() else {
            return Err(http::Failure::message("screens.response"));
        };
        rows.extend(items.iter().cloned());
        if value["pages"].as_u64().unwrap_or(1) <= page {
            return Ok(rows);
        }
    }
    Err(http::Failure::message("relations.limit"))
}
#[component]
pub fn Inspector(
    data: RwSignal<Value>,
    history: RwSignal<History>,
    dirty: RwSignal<bool>,
    busy: RwSignal<bool>,
    token: ReadSignal<Option<String>>,
    on_expired: Callback<()>,
) -> impl IntoView {
    let terms = RwSignal::new(Vec::<Value>::new());
    let taxonomies = RwSignal::new(Vec::<Value>::new());
    let page_rows = RwSignal::new(Vec::<Value>::new());
    let ready = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let guard = Guard::new();
    let kind = Memo::new(move |_| {
        data.get()["content_type"]
            .as_str()
            .unwrap_or_default()
            .to_owned()
    });
    Effect::new(move |_| {
        let current = token.get();
        let kind = kind.get();
        let guard = guard.clone();
        let ticket = guard.begin();
        ready.set(false);
        let Some(current) = current else { return };
        wasm_bindgen_futures::spawn_local(async move {
            let result = async {
                let all_terms = http::request("GET", "/api/v1/terms", &current, None).await?;
                let all_taxonomies =
                    http::request("GET", "/api/v1/taxonomies", &current, None).await?;
                let pages = if kind == "page" {
                    pages(&current).await?
                } else {
                    vec![]
                };
                Ok::<_, http::Failure>((all_terms, all_taxonomies, pages))
            }
            .await;
            if !guard.accepts(ticket, &current) {
                return;
            }
            match result {
                Ok((all_terms, all_taxonomies, pages)) => {
                    if let (Ok(t), Ok(tx)) = (
                        serde_json::from_value(all_terms),
                        serde_json::from_value(all_taxonomies),
                    ) {
                        terms.set(t);
                        taxonomies.set(tx);
                        page_rows.set(pages);
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
    view! {<section class="rf-entry-relations"><h3>{text("relations.title")}</h3>{move ||message.get().map(|message|view!{<p role="status">{message}</p>})}<fieldset disabled=move ||busy.get()||!ready.get()><legend class="rf-sr-only">{text("relations.title")}</legend>
    <Show when=move ||kind.get()=="page"><label for="rf-parent-page">{text("relations.parent")}</label><select id="rf-parent-page" prop:value=move ||data.get()["parent_id"].as_str().unwrap_or_default().to_owned() on:change=move |event|{let id=event_target_value(&event);change(data,history,dirty,|data|data["parent_id"]=if id.is_empty(){Value::Null}else{json!(id)});}>
    <option value="">{text("relations.no_parent")}</option>
    {move ||{let current=data.get();let options=super::options(&page_rows.get(),current["id"].as_str());let existing=current["parent_id"].as_str().filter(|id|!options.iter().any(|(key,_)|key==id)).map(str::to_owned);view!{{existing.map(|id|view!{<option value=id selected=true>{text("relations.existing_parent")}</option>})}{options.into_iter().map(|(id,label)|{let id=StoredValue::new(id);view!{<option value=id.get_value() selected=move ||data.get()["parent_id"].as_str()==Some(&id.get_value())>{label}</option>}}).collect_view()}}}}
    </select><p class="rf-description">{text("relations.parent_hint")}</p></Show>
    {move ||taxonomies.get().into_iter().map(|taxonomy|{let slug=taxonomy["slug"].as_str().unwrap_or_default().to_owned();let label=taxonomy["name"].as_str().unwrap_or_default().to_owned();let slug=StoredValue::new(slug);view!{<fieldset class="rf-term-choices"><legend>{label}</legend>{move ||{let items=terms.get().into_iter().filter(|term|term["taxonomy"].as_str()==Some(&slug.get_value())).collect::<Vec<_>>();super::options(&items,None).into_iter().map(|(id,label)|{let id=StoredValue::new(id);view!{<label><input type="checkbox" prop:checked=move ||selected(&data.get(),&id.get_value()) on:change=move |event|{let checked=event_target_checked(&event);change(data,history,dirty,|entry|{let mut ids=ids(entry);ids.retain(|item|item!=&id.get_value());if checked{ids.push(id.get_value());}entry["term_ids"]=json!(ids);});}/>{label}</label>}}).collect_view()}}<QuickTerm taxonomy=taxonomy.clone() terms data history dirty busy token on_expired/></fieldset>}}).collect_view()}
    </fieldset></section>}
}
fn ids(value: &Value) -> Vec<String> {
    value["term_ids"]
        .as_array()
        .map(|ids| {
            ids.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_else(|| {
            value["terms"]
                .as_array()
                .map(|terms| {
                    terms
                        .iter()
                        .filter_map(|term| term["id"].as_str())
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default()
        })
}
fn selected(value: &Value, id: &str) -> bool {
    ids(value).iter().any(|item| item == id)
}
#[component]
pub fn PageTree(token: ReadSignal<Option<String>>) -> impl IntoView {
    let rows = RwSignal::new(Vec::<Value>::new());
    let message = RwSignal::new(None::<String>);
    let busy = RwSignal::new(false);
    let guard = Guard::new();
    let load = move |_| {
        let Some(current) = token.get_untracked() else {
            return;
        };
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        let guard = guard.clone();
        let ticket = guard.begin();
        wasm_bindgen_futures::spawn_local(async move {
            let result = pages(&current).await;
            if !guard.accepts(ticket, &current) {
                return;
            }
            busy.set(false);
            match result {
                Ok(value) => {
                    rows.set(value);
                    message.set(None);
                }
                Err(error) => message.set(Some(error.message)),
            }
        });
    };
    view! {<details class="rf-panel rf-screen-panel rf-layout-options"><summary>{text("relations.tree")}</summary><button class="rf-button" disabled=move ||busy.get() on:click=load>{text("dashboard.refresh")}</button>{move ||message.get().map(|message|view!{<p role="status">{message}</p>})}<ul>{move ||super::options(&rows.get(),None).into_iter().map(|(id,label)|view!{<li><a href=format!("#editor:{id}:page")>{label}</a></li>}).collect_view()}</ul></details>}
}

#[component]
fn QuickTerm(
    taxonomy: Value,
    terms: RwSignal<Vec<Value>>,
    data: RwSignal<Value>,
    history: RwSignal<History>,
    dirty: RwSignal<bool>,
    busy: RwSignal<bool>,
    token: ReadSignal<Option<String>>,
    on_expired: Callback<()>,
) -> impl IntoView {
    let taxonomy = StoredValue::new(taxonomy);
    let name = RwSignal::new(String::new());
    let parent = RwSignal::new(String::new());
    let message = RwSignal::new(None::<String>);
    let pending = crate::http::dirty_scope();
    let guard = Guard::new();
    let create = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        if busy.get_untracked() || name.get_untracked().trim().is_empty() {
            return;
        }
        let Some(current) = token.get_untracked() else {
            return;
        };
        let body = json!({"taxonomy":taxonomy.get_value()["slug"],"name":name.get_untracked(),"parent_id":if parent.get_untracked().is_empty(){Value::Null}else{json!(parent.get_untracked())}});
        let guard = guard.clone();
        let ticket = guard.begin();
        busy.set(true);
        message.set(None);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request("POST", "/api/v1/terms", &current, Some(&body)).await;
            if !guard.accepts(ticket, &current) {
                return;
            }
            busy.set(false);
            match result {
                Ok(term) => {
                    if let Some(id) = term["id"].as_str().map(str::to_owned) {
                        terms.update(|rows| rows.push(term));
                        change(data, history, dirty, |entry| {
                            let mut selected = ids(entry);
                            selected.push(id);
                            selected.sort();
                            selected.dedup();
                            entry["term_ids"] = json!(selected);
                        });
                        name.set(String::new());
                        parent.set(String::new());
                        pending.set(false);
                        message.set(Some(text("relations.term_added").into()));
                    }
                }
                Err(f) => {
                    message.set(Some(f.message));
                    if f.unauthorized {
                        on_expired.run(());
                    }
                }
            }
        });
    };
    view! {<details class="rf-quick-term"><summary>{text("relations.quick_term")}</summary><form on:submit=create><label>{text("relations.term_name")}<input maxlength="200" required=true prop:value=move ||name.get() on:input=move |event|{name.set(event_target_value(&event));pending.set(!name.get_untracked().is_empty()||!parent.get_untracked().is_empty());}/></label><Show when=move ||taxonomy.get_value()["hierarchical"]==true><label>{text("taxonomy.parent")}<select prop:value=move ||parent.get() on:change=move |event|{parent.set(event_target_value(&event));pending.set(!name.get_untracked().is_empty()||!parent.get_untracked().is_empty());}><option value="">{text("relations.no_parent")}</option>{move ||crate::relations::options(&terms.get().into_iter().filter(|term|term["taxonomy"]==taxonomy.get_value()["slug"]).collect::<Vec<_>>(),None).into_iter().map(|(id,label)|{let id=StoredValue::new(id);view!{<option value=id.get_value() selected=move ||parent.get()==id.get_value()>{label}</option>}}).collect_view()}</select></label></Show><button class="rf-button" disabled=move ||busy.get()||name.get().trim().is_empty()>{text("relations.create_select")}</button><button type="button" class="rf-button" disabled=move ||busy.get() on:click=move |_|{name.set(String::new());parent.set(String::new());pending.set(false);message.set(None);}>{text("form.cancel")}</button></form>{move ||message.get().map(|m|view!{<p role="status">{m}</p>})}</details>}
}
