//! Native taxonomy terms, using the existing taxonomy and term services.
use crate::{
    http::{self, Guard},
    messages::text,
};
use leptos::prelude::*;
use serde_json::{Value, json};
#[component]
pub fn Taxonomies(
    initial: String,
    #[prop(default = false)] definitions: bool,
    token: ReadSignal<Option<String>>,
    on_expired: Callback<()>,
) -> impl IntoView {
    let taxonomy = RwSignal::new(initial);
    let taxonomies = RwSignal::new(Vec::<Value>::new());
    let rows = RwSignal::new(Vec::<Value>::new());
    let form = RwSignal::new(json!({"name":"","slug":"","description":"","parent_id":null}));
    let editing = RwSignal::new(None::<String>);
    let dirty = crate::http::dirty_scope();
    let busy = RwSignal::new(false);
    let loaded = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let deleting = RwSignal::new(false);
    let merging = RwSignal::new(false);
    let target = RwSignal::new(String::new());
    let refresh = RwSignal::new(0_u32);
    let guard = Guard::new();
    let loader = guard.clone();
    Effect::new(move |_| {
        let current = token.get();
        let selected = taxonomy.get();
        refresh.get();
        let guard = loader.clone();
        let ticket = guard.begin();
        loaded.set(false);
        let Some(current) = current else { return };
        busy.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let result = async {
                let tx = http::request("GET", "/api/v1/taxonomies", &current, None).await?;
                let terms = http::request("GET", "/api/v1/terms", &current, None).await?;
                Ok::<_, http::Failure>((tx, terms))
            }
            .await;
            if !guard.accepts(ticket, &current) {
                return;
            }
            busy.set(false);
            match result {
                Ok((tx, terms)) => {
                    let result = serde_json::from_value::<Vec<Value>>(tx).and_then(|tx| {
                        serde_json::from_value::<Vec<Value>>(terms).map(|terms| (tx, terms))
                    });
                    match result {
                        Ok((tx, terms)) => {
                            if !tx.iter().any(|row| row["slug"].as_str() == Some(&selected)) {
                                message.set(Some(text("screens.unavailable").into()));
                                return;
                            }
                            taxonomies.set(tx);
                            rows.set(terms);
                            loaded.set(true);
                        }
                        Err(_) => message.set(Some(text("screens.response").into())),
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
    let search = RwSignal::new(String::new());
    let items = Memo::new(move |_| {
        rows.get()
            .into_iter()
            .filter(|row| row["taxonomy"].as_str() == Some(&taxonomy.get()))
            .collect::<Vec<_>>()
    });
    let hierarchical = Memo::new(move |_| {
        taxonomies
            .get()
            .iter()
            .find(|row| row["slug"].as_str() == Some(&taxonomy.get()))
            .is_some_and(|row| row["hierarchical"].as_bool() == Some(true))
    });
    let reset = Callback::new(move |_: ()| {
        form.set(json!({"name":"","slug":"","description":"","parent_id":null}));
        editing.set(None);
        deleting.set(false);
        merging.set(false);
        dirty.set(false);
        target.set(String::new());
    });
    let mutation = guard.clone();
    let save = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        if busy.get_untracked() || !loaded.get_untracked() {
            return;
        }
        let Some(current) = token.get_untracked() else {
            return;
        };
        let id = editing.get_untracked();
        let mut body = form.get_untracked();
        body["taxonomy"] = json!(taxonomy.get_untracked());
        let path = id
            .as_ref()
            .map(|id| format!("/api/v1/terms/{id}"))
            .unwrap_or("/api/v1/terms".into());
        let guard = mutation.clone();
        let ticket = guard.begin();
        busy.set(true);
        message.set(None);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request(
                if id.is_some() { "PUT" } else { "POST" },
                &path,
                &current,
                Some(&body),
            )
            .await;
            if !guard.accepts(ticket, &current) {
                return;
            }
            busy.set(false);
            match result {
                Ok(_) => {
                    reset.run(());
                    message.set(Some(text("form.saved").into()));
                    refresh.update(|value| *value = value.wrapping_add(1));
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
    let removal = guard.clone();
    let remove = move |_| {
        if busy.get_untracked() || !deleting.get_untracked() || dirty.get_untracked() {
            return;
        }
        let Some(id) = editing.get_untracked() else {
            return;
        };
        let Some(current) = token.get_untracked() else {
            return;
        };
        let target = target.get_untracked();
        let merge = merging.get_untracked();
        if merge
            && (target.is_empty()
                || !crate::relations::options(&items.get_untracked(), Some(&id))
                    .iter()
                    .any(|(candidate, _)| candidate == &target))
        {
            return;
        }
        let path = if merge {
            format!("/api/v1/terms/{id}/merge")
        } else if target.is_empty() {
            format!("/api/v1/terms/{id}")
        } else {
            format!("/api/v1/terms/{id}?reassign_to={target}")
        };
        let guard = removal.clone();
        let ticket = guard.begin();
        busy.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let body = json!({"target_id":target});
            let result = http::request(
                if merge { "POST" } else { "DELETE" },
                &path,
                &current,
                if merge { Some(&body) } else { None },
            )
            .await;
            if !guard.accepts(ticket, &current) {
                return;
            }
            busy.set(false);
            match result {
                Ok(_) => {
                    reset.run(());
                    message.set(Some(
                        text(if merge {
                            "taxonomy.merged"
                        } else {
                            "taxonomy.deleted"
                        })
                        .into(),
                    ));
                    refresh.update(|value| *value = value.wrapping_add(1));
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
    view! {<h1>{text(if definitions{"taxonomy.definitions"}else{"taxonomy.title"})}</h1><Show when=move ||definitions><Definitions rows=taxonomies token on_expired locked=Memo::new(move |_|busy.get()||dirty.get()) on_changed=Callback::new(move |slug:Option<String>|{reset.run(());if let Some(slug)=slug{taxonomy.set(slug);}refresh.update(|n|*n=n.wrapping_add(1));})/></Show><p>{text("taxonomy.hint")}</p>{move ||message.get().map(|message|view!{<p class="rf-notice" role="status">{message}</p>})}
    <label for="rf-taxonomy-select">{text("taxonomy.choose")}</label><select id="rf-taxonomy-select" disabled=move ||busy.get()||dirty.get() prop:value=move ||taxonomy.get() on:change=move |event|{reset.run(());taxonomy.set(event_target_value(&event));}>{move ||taxonomies.get().into_iter().map(|row|{let slug=StoredValue::new(row["slug"].as_str().unwrap_or_default().to_owned());view!{<option value=slug.get_value() selected=move ||taxonomy.get()==slug.get_value()>{row["name"].as_str().unwrap_or_default().to_owned()}</option>}}).collect_view()}</select>
    <div class="rf-taxonomy-layout"><section class="rf-panel rf-screen-panel"><h2>{text("taxonomy.terms")}</h2><button class="rf-button" disabled=move ||busy.get()||dirty.get() on:click=move |_|reset.run(())>{text("taxonomy.new")}</button><label>{text("screens.search")}<input type="search" maxlength="200" prop:value=move ||search.get() on:input=move |event|search.set(event_target_value(&event))/></label><ul>{move ||crate::relations::options(&items.get().into_iter().filter(|row|row["name"].as_str().unwrap_or_default().to_lowercase().contains(&search.get().to_lowercase())).collect::<Vec<_>>(),None).into_iter().map(|(id,label)|{let id=StoredValue::new(id);view!{<li><button class="rf-text-button" disabled=move ||busy.get()||dirty.get() on:click=move |_|{if let Some(row)=items.get_untracked().into_iter().find(|row|row["id"].as_str()==Some(&id.get_value())){form.set(json!({"name":row["name"],"slug":row["slug"],"description":row["description"],"parent_id":row["parent_id"]}));editing.set(Some(id.get_value()));deleting.set(false);merging.set(false);target.set(String::new());}}>{label}</button></li>}}).collect_view()}</ul></section>
    <section class="rf-panel rf-screen-panel"><h2>{move ||if editing.get().is_some(){text("taxonomy.edit")}else{text("taxonomy.new")}}</h2><form on:submit=save><fieldset class="rf-form" disabled=move ||busy.get()||!loaded.get()><legend class="rf-sr-only">{text("taxonomy.term")}</legend>
    {[("name","module.name"),("slug","editor.slug")].into_iter().map(|(key,label)|view!{<label>{text(label)}<input required=key=="name" maxlength="200" prop:value=move ||form.get()[key].as_str().unwrap_or_default().to_owned() on:input=move |event|{form.update(|value|value[key]=json!(event_target_value(&event)));dirty.set(true);}/></label>}).collect_view()}
    <label>{text("module.description")}<textarea maxlength="8192" prop:value=move ||form.get()["description"].as_str().unwrap_or_default().to_owned() on:input=move |event|{form.update(|value|value["description"]=json!(event_target_value(&event)));dirty.set(true);}/></label>
    <Show when=move ||hierarchical.get()><label>{text("taxonomy.parent")}<select prop:value=move ||form.get()["parent_id"].as_str().unwrap_or_default().to_owned() on:change=move |event|{let id=event_target_value(&event);form.update(|value|value["parent_id"]=if id.is_empty(){Value::Null}else{json!(id)});dirty.set(true);}><option value="">{text("relations.no_parent")}</option>{move ||crate::relations::options(&items.get(),editing.get().as_deref()).into_iter().map(|(id,label)|{let id=StoredValue::new(id);view!{<option value=id.get_value() selected=move ||form.get()["parent_id"].as_str()==Some(&id.get_value())>{label}</option>}}).collect_view()}</select></label></Show>
    <div class="rf-form-actions"><button class="rf-button primary" disabled=move ||!dirty.get()||form.get()["name"].as_str().unwrap_or_default().trim().is_empty()>{text("form.save")}</button><button type="button" class="rf-button" on:click=move |_|reset.run(())>{text("form.cancel")}</button></div></fieldset></form>
    <Show when=move ||editing.get().is_some()><button class="rf-button" disabled=move ||busy.get()||dirty.get() on:click=move |_|{merging.set(false);target.set(String::new());deleting.set(true);}>{text("taxonomy.delete")}</button><button class="rf-button" disabled=move ||busy.get()||dirty.get() on:click=move |_|{merging.set(true);target.set(String::new());deleting.set(true);}>{text("taxonomy.merge")}</button></Show>
    <Show when=move ||deleting.get()><fieldset disabled=move ||busy.get()><legend>{move ||text(if merging.get(){"taxonomy.merge_confirm"}else{"taxonomy.delete_confirm"})}</legend><p>{move ||text(if merging.get(){"taxonomy.merge_scope"}else{"taxonomy.delete_scope"})}</p><label>{text("taxonomy.reassign")}<select prop:value=move ||target.get() on:change=move |event|target.set(event_target_value(&event))><option value="">{move ||text(if merging.get(){"taxonomy.choose_target"}else{"taxonomy.no_reassign"})}</option>{move ||crate::relations::options(&items.get(),editing.get().as_deref()).into_iter().map(|(id,label)|{let id=StoredValue::new(id);view!{<option value=id.get_value() selected=move ||target.get()==id.get_value()>{label}</option>}}).collect_view()}</select></label><button class="rf-button" disabled=move ||dirty.get()||(merging.get()&&target.get().is_empty()) on:click=remove.clone()>{move ||text(if merging.get(){"taxonomy.merge_final"}else{"taxonomy.delete_final"})}</button><button class="rf-button" on:click=move |_|deleting.set(false)>{text("form.cancel")}</button></fieldset></Show>
    </section></div><p class="rf-description">{text("taxonomy.future")}</p>}
}

#[component]
fn Definitions(
    rows: RwSignal<Vec<Value>>,
    token: ReadSignal<Option<String>>,
    on_expired: Callback<()>,
    locked: Memo<bool>,
    on_changed: Callback<Option<String>>,
) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let slug = RwSignal::new(String::new());
    let hierarchy = RwSignal::new(false);
    let editing = RwSignal::new(None::<Value>);
    let pending = RwSignal::new(None::<Value>);
    let busy = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let dirty = crate::http::dirty_scope();
    let guard = Guard::new();
    let saving = guard.clone();
    let reset = Callback::new(move |_: ()| {
        name.set(String::new());
        slug.set(String::new());
        hierarchy.set(false);
        editing.set(None);
        pending.set(None);
        dirty.set(false);
    });
    let invalidation = guard.clone();
    Effect::new(move |_| {
        token.get();
        invalidation.begin();
        reset.run(());
    });
    let save = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        if busy.get_untracked() || locked.get_untracked() {
            return;
        }
        let Some(current) = token.get_untracked() else {
            return;
        };
        let old = editing.get_untracked();
        let path = old
            .as_ref()
            .and_then(|v| v["id"].as_str())
            .map(|id| format!("/api/v1/taxonomies/{id}"))
            .unwrap_or("/api/v1/taxonomies".into());
        let body = if let Some(old) = &old {
            json!({"name":name.get_untracked(),"expected_name":old["name"]})
        } else {
            let mut body =
                json!({"name":name.get_untracked(),"hierarchical":hierarchy.get_untracked()});
            if !slug.get_untracked().is_empty() {
                body["slug"] = json!(slug.get_untracked());
            }
            body
        };
        let guard = saving.clone();
        let ticket = guard.begin();
        busy.set(true);
        message.set(None);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request(
                if old.is_some() { "PUT" } else { "POST" },
                &path,
                &current,
                Some(&body),
            )
            .await;
            if !guard.accepts(ticket, &current) {
                return;
            }
            busy.set(false);
            match result {
                Ok(row) => {
                    reset.run(());
                    message.set(Some(text("form.saved").into()));
                    on_changed.run(row["slug"].as_str().map(str::to_owned));
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
    let remove = move |_| {
        if busy.get_untracked() || locked.get_untracked() || dirty.get_untracked() {
            return;
        }
        let (Some(row), Some(current)) = (pending.get_untracked(), token.get_untracked()) else {
            return;
        };
        let Some(id) = row["id"]
            .as_str()
            .filter(|id| crate::editor::local_id(id))
            .map(str::to_owned)
        else {
            return;
        };
        let guard = guard.clone();
        let ticket = guard.begin();
        busy.set(true);
        message.set(None);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request(
                "DELETE",
                &format!("/api/v1/taxonomies/{id}"),
                &current,
                None,
            )
            .await;
            if !guard.accepts(ticket, &current) {
                return;
            }
            busy.set(false);
            match result {
                Ok(_) => {
                    reset.run(());
                    message.set(Some(text("taxonomy.definition_deleted").into()));
                    on_changed.run(Some("category".into()));
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
    view! {<section class="rf-panel rf-screen-panel rf-taxonomy-definitions"><h2>{text("taxonomy.definitions")}</h2><p>{text("taxonomy.definition_scope")}</p>
        {move ||message.get().map(|m|view!{<p class="rf-notice" role="status">{m}</p>})}
        <form on:submit=save><fieldset class="rf-form" disabled=move ||busy.get()||locked.get()><legend>{move ||text(if editing.get().is_some(){"taxonomy.rename"}else{"taxonomy.create_definition"})}</legend>
            <label>{text("module.name")}<input required=true maxlength="200" prop:value=move ||name.get() on:input=move |event|{name.set(event_target_value(&event));dirty.set(true);}/></label>
            <label>{text("editor.slug")}<input maxlength="80" disabled=move ||editing.get().is_some() prop:value=move ||slug.get() on:input=move |event|{slug.set(event_target_value(&event));dirty.set(true);}/></label>
            <label><input type="checkbox" disabled=move ||editing.get().is_some() prop:checked=move ||hierarchy.get() on:change=move |event|{hierarchy.set(event_target_checked(&event));dirty.set(true);}/>{text("taxonomy.hierarchical")}</label>
            <button class="rf-button primary" disabled=move ||!dirty.get()||name.get().trim().is_empty()>{text("form.save")}</button><button type="button" class="rf-button" on:click=move |_|reset.run(())>{text("form.cancel")}</button>
        </fieldset></form>
        <div class="rf-table-scroll"><table class="rf-list-table"><thead><tr><th>{text("module.name")}</th><th>{text("editor.slug")}</th><th>{text("taxonomy.terms")}</th><th>{text("module.action")}</th></tr></thead><tbody>{move ||rows.get().into_iter().map(|row|{let row=StoredValue::new(row);let system=row.get_value()["is_system"]==true;view!{<tr><th scope="row">{row.get_value()["name"].as_str().unwrap_or_default().to_owned()}</th><td>{row.get_value()["slug"].as_str().unwrap_or_default().to_owned()}</td><td>{row.get_value()["term_count"].to_string()}</td><td class="rf-definition-actions">
            <button class="rf-button" disabled=move ||busy.get()||locked.get()||dirty.get() on:click=move |_|on_changed.run(row.get_value()["slug"].as_str().map(str::to_owned))>{text("taxonomy.manage_terms")}</button>
            <button class="rf-button" disabled=move ||busy.get()||locked.get()||dirty.get()||system on:click=move |_|{let row=row.get_value();name.set(row["name"].as_str().unwrap_or_default().into());slug.set(row["slug"].as_str().unwrap_or_default().into());hierarchy.set(row["hierarchical"]==true);editing.set(Some(row));pending.set(None);}>{text("taxonomy.rename")}</button>
            <button class="rf-button" disabled=move ||busy.get()||locked.get()||dirty.get()||system||row.get_value()["term_count"].as_i64()!=Some(0) on:click=move |_|pending.set(Some(row.get_value()))>{text("taxonomy.delete_definition")}</button>
        </td></tr>}}).collect_view()}</tbody></table></div>
        {move ||pending.get().map(|row|view!{<div class="rf-notice"><p>{text("taxonomy.definition_confirm")}</p><p>{row["name"].as_str().unwrap_or_default().to_owned()}</p><button class="rf-button" disabled=move ||busy.get()||locked.get() on:click=remove.clone()>{text("module.confirm")}</button><button class="rf-button" on:click=move |_|pending.set(None)>{text("form.cancel")}</button></div>})}
    </section>}
}
