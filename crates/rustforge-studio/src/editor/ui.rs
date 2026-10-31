use super::{BLOCKS, History, Location, block, empty, payload, projected};
use crate::{
    http::{self, Guard},
    messages::text,
};
use leptos::prelude::*;
use serde_json::{Value, json};
pub(crate) fn change(
    data: RwSignal<Value>,
    history: RwSignal<History>,
    dirty: RwSignal<bool>,
    edit: impl FnOnce(&mut Value),
) {
    let before = data.get_untracked();
    let mut next = before.clone();
    edit(&mut next);
    if next != before {
        history.update(|history| history.push(&before));
        data.set(next);
        dirty.set(true);
    }
}
#[component]
pub fn Editor(
    location: Location,
    token: ReadSignal<Option<String>>,
    on_expired: Callback<()>,
) -> impl IntoView {
    let owner = expect_context::<RwSignal<Option<String>>>();
    let menus = expect_context::<ReadSignal<Vec<crate::navigation::MenuNode>>>();
    let forms_available = Memo::new(move |_| {
        crate::shell::find_route(&menus.get(), "forms").is_some_and(|node| !node.planning)
    });
    let draft_suffix = location
        .id
        .clone()
        .unwrap_or_else(|| format!("new-{}", location.content_type));
    let draft_key = Memo::new(move |_| {
        owner
            .get()
            .map(|owner| format!("rustforge.draft.{owner}.{draft_suffix}"))
    });
    let recovered = RwSignal::new(None::<Value>);
    let backup_message = RwSignal::new(None::<String>);
    let kind = location.content_type.clone();
    let existing = location.id.clone();
    let data = RwSignal::new(empty(&kind));
    let baseline = RwSignal::new(Value::Null);
    let history = RwSignal::new(History::default());
    let loaded = RwSignal::new(false);
    let busy = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let preview = RwSignal::new(false);
    let preview_width = RwSignal::new("100%".to_owned());
    let show_revisions = RwSignal::new(false);
    let confirm_publish = RwSignal::new(false);
    let dirty = crate::http::dirty_scope();
    let guard = Guard::new();
    let loader = guard.clone();
    Effect::new(move |_| {
        let current = token.get();
        let ticket = loader.begin();
        loaded.set(false);
        data.set(empty(&kind));
        dirty.set(false);
        let Some(current) = current else { return };
        let Some(id) = existing.clone() else {
            loaded.set(true);
            baseline.set(data.get_untracked());
            return;
        };
        let guard = loader.clone();
        let kind = kind.clone();
        busy.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let result =
                http::request("GET", &format!("/api/v1/entries/{id}"), &current, None).await;
            if !guard.accepts(ticket, &current) {
                return;
            }
            busy.set(false);
            match result {
                Ok(value) => {
                    if value["content_type"].as_str() != Some(&kind) {
                        message.set(Some(text("screens.unavailable").into()));
                        return;
                    }
                    baseline.set(value.clone());
                    data.set(value);
                    loaded.set(true);
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
    Effect::new(move |_| {
        if !loaded.get() {
            return;
        }
        if let Some(key) = draft_key.get() {
            let stored = web_sys::window()
                .and_then(|window| window.local_storage().ok().flatten())
                .and_then(|storage| storage.get_item(&key).ok().flatten())
                .filter(|raw| raw.len() <= 2 * 1024 * 1024)
                .and_then(|raw| serde_json::from_str::<Value>(&raw).ok());
            recovered.set(stored.filter(|value| {
                value.is_object()
                    && value["content_type"] == data.get_untracked()["content_type"]
                    && value["id"] == data.get_untracked()["id"]
            }));
        }
    });
    let backup = Callback::new(move |_: ()| {
        if !dirty.get_untracked() || !loaded.get_untracked() {
            return;
        }
        let result = (|| {
            let key = draft_key.get_untracked()?;
            let storage = web_sys::window()?.local_storage().ok()??;
            let encoded = serde_json::to_string(&super::backup(&data.get_untracked())).ok()?;
            if encoded.len() > 2 * 1024 * 1024 {
                return None;
            }
            storage.set_item(&key, &encoded).ok()?;
            Some(())
        })();
        backup_message.set(Some(
            text(if result.is_some() {
                "editor.local_saved"
            } else {
                "editor.local_failed"
            })
            .into(),
        ));
    });
    let timer =
        set_interval_with_handle(move || backup.run(()), std::time::Duration::from_secs(2)).ok();
    on_cleanup(move || {
        if let Some(timer) = timer {
            timer.clear();
        }
    });
    let save_guard = guard.clone();
    let save_kind = location.content_type.clone();
    let save = Callback::new(move |publish: bool| {
        if busy.get_untracked() || !loaded.get_untracked() {
            return;
        }
        let Some(current) = token.get_untracked() else {
            return;
        };
        let mut value = data.get_untracked();
        if publish {
            value["status"] = json!("published");
            if value["published_at"].is_null() {
                value["published_at"] = json!(
                    js_sys::Date::new_0()
                        .to_iso_string()
                        .as_string()
                        .unwrap_or_default()
                );
            }
        }
        if !publish
            && value["status"] == "published"
            && baseline.get_untracked()["status"] != "published"
        {
            confirm_publish.set(true);
            return;
        }
        let creating = value["id"].as_str().is_none();
        let body = match payload(&value, creating) {
            Ok(body) => body,
            Err(error) => {
                message.set(Some(error));
                return;
            }
        };
        let path = if creating {
            format!("/api/v1/entries?content_type={save_kind}")
        } else {
            format!(
                "/api/v1/entries/{}",
                value["id"].as_str().unwrap_or_default()
            )
        };
        let guard = save_guard.clone();
        let ticket = guard.begin();
        busy.set(true);
        message.set(None);
        confirm_publish.set(false);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request(
                if creating { "POST" } else { "PUT" },
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
                Ok(value) => {
                    if let Some(key) = draft_key.get_untracked()
                        && let Some(storage) = web_sys::window()
                            .and_then(|window| window.local_storage().ok().flatten())
                    {
                        let _ = storage.remove_item(&key);
                    }
                    recovered.set(None);
                    backup_message.set(None);
                    baseline.set(value.clone());
                    data.set(value);
                    dirty.set(false);
                    history.set(History::default());
                    message.set(Some(text("form.saved").into()));
                    if creating
                        && let Some(id) = data.get_untracked()["id"]
                            .as_str()
                            .filter(|id| super::local_id(id))
                        && let Some(window) = web_sys::window()
                    {
                        let _ = window.location().set_hash(&format!(
                            "editor:{id}:{}",
                            data.get_untracked()["content_type"]
                                .as_str()
                                .unwrap_or("post")
                        ));
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
    let on_restored = Callback::new(move |value: Value| {
        if let Some(key) = draft_key.get_untracked()
            && let Some(storage) =
                web_sys::window().and_then(|window| window.local_storage().ok().flatten())
        {
            let _ = storage.remove_item(&key);
        }
        recovered.set(None);
        backup_message.set(None);
        message.set(None);
        baseline.set(value.clone());
        data.set(value);
        dirty.set(false);
        history.set(History::default());
    });
    view! {<section class="rf-editor" on:focusout=move |_|backup.run(())><div class="rf-editor-toolbar"><a class="rf-button" href=format!("#entries:{}",location.content_type)>{text("editor.back")}</a><h1>{text("editor.title")}</h1>{move ||data.get()["id"].as_str().filter(|id|super::local_id(id)).map(|id|view!{<a class="rf-button" href=format!("/admin/#editor:{}:{}",id,data.get()["content_type"].as_str().unwrap_or("post")) target="_blank" rel="noopener noreferrer">{text("builder.transition")}</a>})}<button class="rf-button" disabled=move ||busy.get() on:click=move |_|{history.update(|history|if let Some(value)=history.undo(&data.get_untracked()){data.set(value);dirty.set(true);});}>{text("editor.undo")}</button><button class="rf-button" disabled=move ||busy.get() on:click=move |_|history.update(|history|if let Some(value)=history.redo(&data.get_untracked()){data.set(value);dirty.set(true);})>{text("editor.redo")}</button><button class="rf-button" on:click=move |_|preview.update(|value|*value = !*value)>{text("editor.preview")}</button><button class="rf-button primary" disabled=move ||busy.get()||!loaded.get() on:click=move |_|save.run(false)>{move ||if busy.get(){text("form.saving")}else{text("form.save")}}</button><button class="rf-button" disabled=move ||busy.get()||!loaded.get() on:click=move |_|confirm_publish.set(true)>{text("editor.publish")}</button>{move ||if !dirty.get()&&!busy.get(){super::public_link(&data.get(),js_sys::Date::now() as i64).map(|url|view!{<a class="rf-button" href=url target="_blank" rel="noopener noreferrer">{text("editor.view_public")}</a>})}else{None}}</div>
    {move ||message.get().map(|message|view!{<p class="rf-notice" role="status">{message}</p>})}
    {move ||backup_message.get().map(|message|view!{<p role="status" class="rf-description">{message}</p>})}
    <Show when=move ||recovered.get().is_some()><div class="rf-notice"><p>{text("editor.recovery")}</p><button class="rf-button" on:click=move |_|{if let Some(value)=recovered.get_untracked(){change(data,history,dirty,|entry|*entry=value);}recovered.set(None);}>{text("editor.recover")}</button><button class="rf-button" on:click=move |_|{if let Some(key)=draft_key.get_untracked()&& let Some(storage)=web_sys::window().and_then(|window|window.local_storage().ok().flatten()){let _=storage.remove_item(&key);}recovered.set(None);}>{text("editor.discard_backup")}</button></div></Show>
    <Show when=move ||confirm_publish.get()><div class="rf-notice"><p>{text("editor.publish_confirm")}</p><button class="rf-button primary" disabled=move ||busy.get() on:click=move |_|save.run(true)>{text("editor.publish_yes")}</button><button class="rf-button" on:click=move |_|confirm_publish.set(false)>{text("form.cancel")}</button></div></Show>
    <Show when=move ||loaded.get()><div class="rf-editor-layout"><fieldset class="rf-editor-canvas" disabled=move ||busy.get()><legend class="rf-sr-only">{text("editor.content")}</legend><label for="rf-entry-title">{text("screens.title")}</label><input id="rf-entry-title" class="rf-editor-title" maxlength="300" prop:value=move ||data.get()["title"].as_str().unwrap_or_default().to_owned() on:input=move |event|change(data,history,dirty,|value|value["title"]=json!(event_target_value(&event)))/>
    <Show when=move ||data.get()["blocks"]["rf_schema"]=="rf.document/1" fallback=move ||view!{<p class="rf-notice">{text("builder.legacy_hint")}</p><label for="rf-legacy-body">{text("editor.legacy")}</label><textarea id="rf-legacy-body" prop:value=move ||data.get()["body"].as_str().unwrap_or_default().to_owned() on:input=move |event|change(data,history,dirty,|value|value["body"]=json!(event_target_value(&event)))/><button class="rf-button" on:click=move |_|{let body=data.get_untracked()["body"].as_str().unwrap_or_default().to_owned();match rustforge_document::document::from_markdown(&body){Ok(blocks)=>change(data,history,dirty,|value|value["blocks"]=blocks),Err(error)=>message.set(Some(error.to_string()))}}>{text("editor.convert")}</button>}>
    <super::builder::Inserter data history dirty busy forms_available/><super::builder::Outline data/>
    <For each={move ||data.get()["blocks"]["rf_blocks"].as_array().cloned().unwrap_or_default().into_iter().filter_map(|block|block["rf_id"].as_str().map(str::to_owned)).collect::<Vec<_>>()} key=|id|id.clone() children=move |id|view!{<BlockEditor id data history dirty/>}/>
    <label for="rf-insert-kind">{text("editor.add")}</label><select id="rf-insert-kind" on:change=move |event|{let kind=event_target_value(&event);if BLOCKS.iter().any(|(key,_)|key==&kind)&&(kind!="form_embed"||forms_available.get_untracked()){change(data,history,dirty,|value|{let blocks=value["blocks"]["rf_blocks"].as_array_mut().unwrap();if blocks.len()<256{blocks.push(block(&kind,uuid::Uuid::new_v4().to_string()));}});event_target::<web_sys::HtmlSelectElement>(&event).set_value("");}}><option value="">{text("editor.choose")}</option>{BLOCKS.iter().map(|(kind,key)|view!{<option value=*kind disabled=move ||*kind=="form_embed"&&!forms_available.get()>{text(key)}</option>}).collect_view()}</select>
    </Show>
    </fieldset><aside class="rf-panel rf-screen-panel rf-editor-inspector"><h2>{text("editor.document")}</h2><EntryField name="slug" label="editor.slug" data history dirty busy/><EntryField name="excerpt" label="editor.excerpt" data history dirty busy/><label for="rf-entry-state">{text("screens.status")}</label><select id="rf-entry-state" disabled=move ||busy.get() prop:value=move ||data.get()["status"].as_str().unwrap_or("draft").to_owned() on:change=move |event|change(data,history,dirty,|value|value["status"]=json!(event_target_value(&event)))><option value="draft" selected=move ||data.get()["status"]=="draft">{text("dashboard.draft")}</option><option value="published" selected=move ||data.get()["status"]=="published">{text("list.published_scheduled")}</option><option value="archived" selected=move ||data.get()["status"]=="archived">{text("dashboard.archived")}</option></select><p class="rf-description">{text("editor.schedule_hint")}</p><EntryField name="published_at" label="editor.publish_date" data history dirty busy/><EntryField name="scheduled_unpublish_at" label="editor.unpublish_date" data history dirty busy/>
    <section class="rf-discussion-inspector"><h3>{text("editor.discussion")}</h3><label for="rf-entry-comments">{text("editor.comments_setting")}</label><select id="rf-entry-comments" disabled=move ||busy.get() prop:value=move ||data.get()["fields"]["_rf_comment_status"].as_str().unwrap_or("").to_owned() on:change=move |event|{let status=event_target_value(&event);change(data,history,dirty,|value|{if !value["fields"].is_object(){value["fields"]=json!({});}value["fields"]["_rf_comment_status"]=if status.is_empty(){Value::Null}else{json!(status)};});}>
    <option value="" selected=move ||data.get()["fields"]["_rf_comment_status"].is_null()>{text("editor.comments_inherit")}</option><option value="open" selected=move ||data.get()["fields"]["_rf_comment_status"]=="open">{text("editor.comments_open")}</option><option value="closed" selected=move ||data.get()["fields"]["_rf_comment_status"]=="closed">{text("editor.comments_closed")}</option></select><p class="rf-description">{text("editor.comments_hint")}</p></section>
    <super::builder::Template data history dirty busy/>
    <super::featured::FeaturedImage data history dirty busy token on_expired/>
    <crate::relations::ui::Inspector data history dirty busy token on_expired/>
    <button class="rf-button" aria-expanded=move ||show_revisions.get().to_string() disabled=move ||busy.get()||data.get()["id"].is_null() on:click=move |_|show_revisions.update(|value|*value = !*value)>{text("editor.revisions")}</button><p>{move ||if dirty.get(){text("form.unsaved")}else{""}}</p></aside></div></Show>
    <Show when=move ||show_revisions.get()><super::revisions::RevisionHistory data dirty busy token on_expired on_restored/></Show>
    <crate::block_catalog::ui::Catalog/>
    <Show when=move ||preview.get()><section class="rf-panel rf-screen-panel"><h2>{text("editor.preview")}</h2><label for="rf-preview-width">{text("builder.preview_width")}</label><select id="rf-preview-width" on:change=move |e|preview_width.set(event_target_value(&e))><option value="100%">{text("builder.desktop")}</option><option value="768px">{text("builder.tablet")}</option><option value="390px">{text("builder.mobile")}</option></select><p class="rf-description">{text("builder.preview_hint")}</p>{move ||match projected(&data.get()){Ok((_,html))=>view!{<div class="rf-document-preview" style=move ||format!("max-width:{};margin:auto",preview_width.get()) inner_html=html/>}.into_any(),Err(error)=>view!{<p class="rf-notice rf-notice-error" role="alert">{error}</p>}.into_any()}}</section></Show>
    </section>}
}
#[component]
fn EntryField(
    name: &'static str,
    label: &'static str,
    data: RwSignal<Value>,
    history: RwSignal<History>,
    dirty: RwSignal<bool>,
    busy: RwSignal<bool>,
) -> impl IntoView {
    let id = format!("rf-entry-{name}");
    view! {<label for=id.clone()>{text(label)}</label><input id=id disabled=move ||busy.get() prop:value=move ||data.get()[name].as_str().unwrap_or_default().to_owned() on:input=move |event|change(data,history,dirty,|value|{let input=event_target_value(&event);value[name]=if matches!(name,"published_at"|"scheduled_unpublish_at")&&input.is_empty(){Value::Null}else{json!(input)};})/>}
}
#[component]
fn BlockEditor(
    id: String,
    data: RwSignal<Value>,
    history: RwSignal<History>,
    dirty: RwSignal<bool>,
) -> impl IntoView {
    let token = expect_context::<ReadSignal<Option<String>>>();
    let block_id = StoredValue::new(id.clone());
    let block = Memo::new(move |_| {
        data.get()["blocks"]["rf_blocks"]
            .as_array()
            .and_then(|blocks| {
                blocks
                    .iter()
                    .find(|block| block["rf_id"].as_str() == Some(&block_id.get_value()))
            })
            .cloned()
            .unwrap_or(Value::Null)
    });
    let set = Callback::new(move |(key, value): (&'static str, Value)| {
        let id = block_id.get_value();
        change(data, history, dirty, |entry| {
            if let Some(block) = entry["blocks"]["rf_blocks"]
                .as_array_mut()
                .and_then(|blocks| {
                    blocks
                        .iter_mut()
                        .find(|block| block["rf_id"].as_str() == Some(&id))
                })
            {
                block[key] = value;
            }
        });
    });
    let kind = block.get_untracked()["rf_type"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    let heading = BLOCKS
        .iter()
        .find(|(key, _)| key == &kind)
        .map(|(_, label)| text(label))
        .unwrap_or(text("editor.block"));
    let move_block = move |direction: i32| {
        let id = block_id.get_value();
        change(data, history, dirty, |entry| {
            if let Some(blocks) = entry["blocks"]["rf_blocks"].as_array_mut()
                && let Some(index) = blocks
                    .iter()
                    .position(|block| block["rf_id"].as_str() == Some(&id))
            {
                let next = index as i32 + direction;
                if next >= 0 && (next as usize) < blocks.len() {
                    blocks.swap(index, next as usize);
                }
            }
        });
    };
    view! {<section class="rf-native-block" id=format!("rf-block-{}",block_id.get_value()) tabindex="-1"><div class="rf-block-toolbar"><h2>{heading}</h2><button type="button" class="rf-button" on:click=move |_|{if let Some(next)=super::duplicate_block(&data.get_untracked(),&block_id.get_value()){change(data,history,dirty,|entry|*entry=next);}}>{text("builder.duplicate")}</button><button class="rf-button" on:click=move |_|move_block(-1)>{text("layout.up")}</button><button class="rf-button" on:click=move |_|move_block(1)>{text("layout.down")}</button><button class="rf-button" on:click=move |_|{let id=block_id.get_value();change(data,history,dirty,|entry|if let Some(blocks)=entry["blocks"]["rf_blocks"].as_array_mut(){blocks.retain(|block|block["rf_id"].as_str()!=Some(&id));});}>{text("editor.remove")}</button></div>
    {if !matches!(kind.as_str(),"divider"|"image"|"list"|"columns"|"spacer"|"gallery"|"video"|"audio"|"form_embed"){view!{<label>{text("editor.content")}<textarea prop:value=move ||block.get()["rf_text"].as_str().unwrap_or_default().to_owned() on:input=move |event|set.run(("rf_text",json!(event_target_value(&event))))/></label>}.into_any()}else{().into_any()}}
    {if kind=="form_embed"{view!{<super::form_block::FormBlock block on_choose=Callback::new(move |id:String|set.run(("rf_ref",json!(id))))/>}.into_any()}else{().into_any()}}
    {if kind=="heading"{view!{<label>{text("editor.level")}<input type="number" min="1" max="6" prop:value=move ||block.get()["rf_level"].as_u64().unwrap_or(2) on:input=move |event|{if let Ok(value)=event_target_value(&event).parse::<u8>(){set.run(("rf_level",json!(value)));}}/></label>}.into_any()}else{().into_any()}}
    {if matches!(kind.as_str(),"list"|"table"|"gallery"){view!{<label>{text("editor.items")}<textarea prop:value=move ||block.get()["rf_items"].as_array().map(|items|items.iter().filter_map(Value::as_str).collect::<Vec<_>>().join("\n")).unwrap_or_default() on:input=move |event|set.run(("rf_items",json!(event_target_value(&event).lines().collect::<Vec<_>>()))) /></label>}.into_any()}else{().into_any()}}
    {if kind=="columns"{view!{<For each={move ||(0..block.get()["rf_items"].as_array().map(Vec::len).unwrap_or(0)).collect::<Vec<_>>()} key=|i|*i children=move |index|view!{<label>{format!("{} {}",text("builder.column"),index+1)}<textarea prop:value=move ||block.get()["rf_items"][index].as_str().unwrap_or_default().to_owned() on:input=move |event|{let mut items=block.get_untracked()["rf_items"].clone();if let Some(item)=items.as_array_mut().and_then(|items|items.get_mut(index)){*item=json!(event_target_value(&event));set.run(("rf_items",items));}}/></label>}/>}.into_any()}else{().into_any()}}
    {if kind=="list"{view!{<label><input type="checkbox" prop:checked=move ||block.get()["rf_ordered"].as_bool().unwrap_or(false) on:change=move |event|set.run(("rf_ordered",json!(event_target_checked(&event))))/>{text("editor.ordered")}</label>}.into_any()}else{().into_any()}}
    {if matches!(kind.as_str(),"image"|"media_text"|"cover"|"video"|"audio"|"file"){let with_id=kind!="audio";view!{<crate::media_ui::Picker token on_choose=Callback::new(move |media:Value|{let id=block_id.get_value();change(data,history,dirty,|entry|if let Some(block)=entry["blocks"]["rf_blocks"].as_array_mut().and_then(|blocks|blocks.iter_mut().find(|block|block["rf_id"].as_str()==Some(&id))){block["rf_url"]=media["url"].clone();if with_id{block["rf_media_id"]=media["id"].clone();}});})/>}.into_any()}else{().into_any()}}
    {[("rf_url","editor.url",matches!(kind.as_str(),"image"|"button"|"media_text"|"cover"|"product"|"course"|"video"|"audio"|"file")),("rf_alt","editor.alt",matches!(kind.as_str(),"image"|"media_text"|"cover"|"product"|"course"|"video"|"file")),("rf_caption","editor.caption",matches!(kind.as_str(),"image"|"quote"|"details"|"media_text"|"cover"|"table"|"product"|"course"|"gallery"|"video"|"audio"|"file")),("rf_style","editor.style",matches!(kind.as_str(),"callout"|"media_text"|"columns"|"pattern"|"button"|"spacer"|"cover"|"group"|"product"|"course"|"gallery"|"video")),("rf_ref","editor.reference",matches!(kind.as_str(),"pattern"|"product"|"course"))].into_iter().filter(|(_,_,show)|*show).map(|(key,label,_)|view!{<label>{text(label)}<input maxlength="2048" prop:value=move ||block.get()[key].as_str().unwrap_or_default().to_owned() on:input=move |event|set.run((key,json!(event_target_value(&event))))/></label>}).collect_view()}
    </section>}
}
