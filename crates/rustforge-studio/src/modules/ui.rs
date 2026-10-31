use super::Resource;
use crate::{
    forms::ui::{FormField, Values, patch, selected},
    http::{self, Guard},
    messages::text,
};
use leptos::prelude::*;
use serde_json::{Value, json};
#[component]
pub fn ResourceScreen(
    kind: Resource,
    token: ReadSignal<Option<String>>,
    on_expired: Callback<()>,
    #[prop(default = false)] creating: bool,
) -> impl IntoView {
    let items = RwSignal::new(Vec::<Value>::new());
    let selected_item = RwSignal::new(creating.then(|| kind.empty()));
    let values = RwSignal::new(selected(&kind.fields(), &kind.empty()));
    let original = RwSignal::new(Values::new());
    let complex = RwSignal::new(kind.empty());
    let busy = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let dirty = crate::http::dirty_scope();
    let refresh = RwSignal::new(0_u64);
    let roster = RwSignal::new(Vec::<Value>::new());
    let recipients = RwSignal::new(Vec::<String>::new());
    let confirm = RwSignal::new(false);
    let invocation = RwSignal::new(None::<String>);
    let run_result = RwSignal::new(None::<Value>);
    let guard = Guard::new();
    let loader = guard.clone();
    // FormField's page selector context is unused for these blueprints, but remains well-defined.
    provide_context(RwSignal::new(Vec::<(String, String)>::new()));
    Effect::new(move |_| {
        let current = token.get();
        refresh.get();
        let ticket = loader.begin();
        let Some(current) = current else {
            items.set(vec![]);
            return;
        };
        busy.set(true);
        let guard = loader.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request("GET", kind.endpoint(), &current, None).await;
            let mut users = Vec::new();
            if matches!(kind, Resource::Campaigns | Resource::Recipes) {
                for page in 1..=20 {
                    let response = http::request(
                        "GET",
                        &format!("/api/v1/users?per_page=100&page={page}"),
                        &current,
                        None,
                    )
                    .await;
                    let Ok(value) = response else { break };
                    users.extend(value["items"].as_array().cloned().unwrap_or_default());
                    if page >= value["total_pages"].as_u64().unwrap_or(0) {
                        break;
                    }
                }
            }
            if !guard.accepts(ticket, &current) {
                return;
            }
            busy.set(false);
            roster.set(users);
            match result {
                Ok(value) => {
                    if let Ok(value) = serde_json::from_value(value) {
                        items.set(value);
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
    let all_dirty = expect_context::<RwSignal<bool>>();
    let edit = Callback::new(move |item: Value| {
        if all_dirty.get_untracked()
            && !web_sys::window().is_some_and(|window| {
                window
                    .confirm_with_message(text("form.leave"))
                    .unwrap_or(false)
            })
        {
            return;
        }
        let form = selected(&kind.fields(), &item);
        values.set(form.clone());
        original.set(form);
        complex.set(item.clone());
        recipients.set(if kind == Resource::Campaigns {
            item["recipient_ids"]
                .as_array()
                .map(|items| {
                    items
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default()
        } else {
            item["recipient_id"]
                .as_str()
                .map(|id| vec![id.into()])
                .unwrap_or_default()
        });
        selected_item.set(Some(item));
        dirty.set(false);
        message.set(None);
        confirm.set(false);
        run_result.set(None);
        invocation.set(None);
    });
    let save_guard = guard.clone();
    let save = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        if busy.get_untracked() {
            return;
        }
        let Some(current) = token.get_untracked() else {
            return;
        };
        let Some(item) = selected_item.get_untracked() else {
            return;
        };
        let creating = item["id"].is_null();
        let mut body = match patch(&kind.fields(), &values.get_untracked(), &Values::new()) {
            Ok(body) => body,
            Err(key) => {
                message.set(Some(text(key).into()));
                return;
            }
        };
        if kind == Resource::Forms {
            body["fields"] = complex.get_untracked()["fields"].clone();
            body["settings"] = complex
                .get_untracked()
                .get("settings")
                .cloned()
                .unwrap_or_else(|| json!({}));
        }
        if kind == Resource::Galleries {
            body["settings"] = complex
                .get_untracked()
                .get("settings")
                .cloned()
                .unwrap_or_else(|| json!({}));
        }
        if body["slug"] == "" {
            body.as_object_mut().unwrap().remove("slug");
        }
        if matches!(kind, Resource::Campaigns | Resource::Recipes) {
            body["expected_version"] = item["version"].clone();
            if kind == Resource::Campaigns {
                body["recipient_ids"] = json!(recipients.get_untracked());
            } else {
                body["recipient_id"] = recipients
                    .get_untracked()
                    .first()
                    .map(|id| json!(id))
                    .unwrap_or(Value::Null);
            }
        }
        let path = if creating {
            kind.endpoint().into()
        } else {
            let id = item["id"].as_str().unwrap_or_default();
            if !crate::editor::local_id(id) {
                return;
            }
            format!("{}/{id}", kind.endpoint())
        };
        let ticket = save_guard.begin();
        let guard = save_guard.clone();
        busy.set(true);
        message.set(None);
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
                    let form = selected(&kind.fields(), &value);
                    values.set(form.clone());
                    original.set(form);
                    complex.set(value.clone());
                    selected_item.set(Some(value));
                    dirty.set(false);
                    message.set(Some(text("form.saved").into()));
                    refresh.update(|n| *n = n.wrapping_add(1));
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
    let action_guard = guard.clone();
    let execute = Callback::new(move |_: ()| {
        if busy.get_untracked() || dirty.get_untracked() {
            return;
        }
        let Some(current) = token.get_untracked() else {
            return;
        };
        let Some(item) = selected_item.get_untracked() else {
            return;
        };
        let Some(id) = item["id"]
            .as_str()
            .filter(|id| crate::editor::local_id(id))
            .map(str::to_owned)
        else {
            return;
        };
        let operation = if kind == Resource::Campaigns {
            "queue"
        } else {
            "run"
        };
        let mut body = json!({"expected_version":item["version"]});
        if kind == Resource::Recipes {
            let id = invocation
                .get_untracked()
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
            invocation.set(Some(id.clone()));
            body["invocation_id"] = json!(id);
        }
        let ticket = action_guard.begin();
        let guard = action_guard.clone();
        busy.set(true);
        confirm.set(false);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request(
                "POST",
                &format!("{}/{id}/{operation}", kind.endpoint()),
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
                    if kind == Resource::Campaigns {
                        selected_item.set(Some(value.clone()));
                    }
                    run_result.set(Some(value));
                    invocation.set(None);
                    message.set(Some(text("module.queued").into()));
                    refresh.update(|n| *n = n.wrapping_add(1));
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
    let capture_guard = guard;
    let capture = move |_| {
        if busy.get_untracked() {
            return;
        }
        let Some(current) = token.get_untracked() else {
            return;
        };
        let path = if kind == Resource::Campaigns {
            "/api/v1/mailing/capture"
        } else {
            "/api/v1/automations/capture"
        };
        let guard = capture_guard.clone();
        let ticket = guard.begin();
        busy.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request("POST", path, &current, None).await;
            if !guard.accepts(ticket, &current) {
                return;
            }
            busy.set(false);
            match result {
                Ok(value) => message.set(Some(format!(
                    "{} {}",
                    value["handled"].as_u64().unwrap_or(0),
                    text("module.captured")
                ))),
                Err(error) => {
                    message.set(Some(error.message));
                    if error.unauthorized {
                        on_expired.run(());
                    }
                }
            }
        });
    };
    view! {<section class="rf-screen"><div class="rf-page-heading"><h1>{text(kind.title())}</h1><button class="rf-button" disabled=move ||busy.get() on:click=move |_|edit.run(kind.empty())>{text("module.new")}</button><button class="rf-button" disabled=move ||busy.get() on:click=move |_|refresh.update(|n|*n=n.wrapping_add(1))>{text("dashboard.refresh")}</button></div>
       <Show when=move ||matches!(kind,Resource::Campaigns|Resource::Recipes)><p class="rf-notice">{text("module.capture_only")}</p><a class="rf-button" href="#jobs">{text("jobs.title")}</a><button class="rf-button" disabled=move ||busy.get() on:click=capture.clone()>{text("module.capture")}</button></Show>
       {move ||message.get().map(|message|view!{<p class="rf-notice" role="status">{message}</p>})}
       <div class="rf-module-workspace"><section class="rf-panel rf-screen-panel"><h2>{text("module.library")}</h2><ul class="rf-section-links">{move ||items.get().into_iter().map(|item|{let label=item["name"].as_str().or_else(||item["title"].as_str()).unwrap_or_default().to_owned();view!{<li><button class="rf-text-button" disabled=move ||busy.get() on:click=move |_|edit.run(item.clone())>{label}</button></li>}}).collect_view()}</ul><Show when=move ||items.get().is_empty()&&!busy.get()><p>{text("screens.empty")}</p></Show></section>
       <Show when=move ||selected_item.get().is_some()><section><form on:submit=save.clone()><fieldset class="rf-form" disabled=move ||busy.get()||(kind==Resource::Campaigns&&selected_item.get().is_some_and(|item|item["state"]=="queued"))><legend>{text("module.details")}</legend>{kind.fields().into_iter().map(|field|view!{<FormField field values dirty message/>}).collect_view()}
       <Show when=move ||kind==Resource::Forms><SchemaBuilder document=complex dirty/><FormNotifications document=complex dirty/></Show>
       <Show when=move ||matches!(kind,Resource::Campaigns|Resource::Recipes)><fieldset class="rf-recipient-list"><legend>{text("module.recipients")}</legend>{move ||roster.get().into_iter().filter_map(|user|{let id=user["id"].as_str().filter(|id|crate::editor::local_id(id))?.to_owned();let checked_id=id.clone();Some(view!{<label><input type=if kind==Resource::Recipes{"radio"}else{"checkbox"} name="rf-recipient" prop:checked=move ||recipients.get().contains(&checked_id) on:change=move |event|{let checked=event_target_checked(&event);recipients.update(|items|{if kind==Resource::Recipes{items.clear();}items.retain(|value|value!=&id);if checked{items.push(id.clone());}});dirty.set(true);}/>{user["display_name"].as_str().unwrap_or_default().to_owned()}" — "{user["email"].as_str().unwrap_or_default().to_owned()}</label>})}).collect_view()}</fieldset></Show>
       <button class="rf-button primary" disabled=move ||!dirty.get()>{text("form.save")}</button><button class="rf-button" type="button" on:click=move |_|{selected_item.set(None);dirty.set(false);}>{text("form.cancel")}</button></fieldset></form>
       <Show when=move ||matches!(kind,Resource::Campaigns|Resource::Recipes)&&selected_item.get().is_some_and(|item|!item["id"].is_null())><button class="rf-button" disabled=move ||busy.get()||dirty.get() on:click=move |_|confirm.set(true)>{text("module.prepare_run")}</button><Show when=move ||confirm.get()><div class="rf-notice"><p>{text("module.run_confirm")}</p><p>{recipients.get_untracked().len()}" "{text("module.recipients")}</p><button class="rf-button primary" disabled=move ||busy.get() on:click=move |_|execute.run(())>{text("module.run")}</button><button class="rf-button" on:click=move |_|confirm.set(false)>{text("form.cancel")}</button></div></Show></Show>
       {move ||run_result.get().map(|_|view!{<p class="rf-notice">{text("module.receipt")}<a href="#jobs">{text("jobs.title")}</a></p>})}
    <Show when=move ||kind==Resource::Courses><CourseContent course=selected_item token on_expired/></Show>
       <Show when=move ||kind==Resource::Galleries><GalleryContent gallery=selected_item token on_expired/></Show>
       <Show when=move ||kind==Resource::Forms><crate::submissions::ui::Submissions form=selected_item token on_expired/></Show>
       </section></Show></div></section>}
}
#[component]
fn SchemaBuilder(document: RwSignal<Value>, dirty: RwSignal<bool>) -> impl IntoView {
    let new_key = RwSignal::new(String::new());
    view! {<section class="rf-panel rf-screen-panel"><h2>{text("module.fields")}</h2><For each={move ||document.get()["fields"]["properties"].as_object().map(|properties|properties.keys().cloned().collect::<Vec<_>>()).unwrap_or_default()} key=|key|key.clone() children=move |key|{let key=StoredValue::new(key);view!{<div class="rf-schema-row"><strong>{key.get_value()}</strong><label>{text("module.label")}<input prop:value=move ||document.get()["fields"]["properties"][key.get_value()]["title"].as_str().unwrap_or_default().to_owned() on:input=move |event|{document.update(|value|value["fields"]["properties"][key.get_value()]["title"]=json!(event_target_value(&event)));dirty.set(true);}/></label><label>{text("module.field_type")}<select prop:value=move ||document.get()["fields"]["properties"][key.get_value()]["type"].as_str().unwrap_or("string").to_owned() on:change=move |event|{document.update(|value|value["fields"]["properties"][key.get_value()]["type"]=json!(event_target_value(&event)));dirty.set(true);}><option value="string">{text("module.text")}</option><option value="integer">{text("module.integer")}</option><option value="boolean">{text("module.boolean")}</option></select></label><label><input type="checkbox" prop:checked=move ||document.get()["fields"]["required"].as_array().is_some_and(|items|items.contains(&json!(key.get_value()))) on:change=move |event|{let checked=event_target_checked(&event);document.update(|value|{if !value["fields"]["required"].is_array(){value["fields"]["required"]=json!([]);}let required=value["fields"]["required"].as_array_mut().unwrap();required.retain(|value|value!=&json!(key.get_value()));if checked{required.push(json!(key.get_value()));}});dirty.set(true);}/>{text("module.required")}</label><button class="rf-button" type="button" on:click=move |_|{document.update(|value|{if let Some(properties)=value["fields"]["properties"].as_object_mut(){properties.remove(&key.get_value());}
    if let Some(required)=value["fields"]["required"].as_array_mut(){required.retain(|value|value!=&json!(key.get_value()));}});dirty.set(true);}>{text("editor.remove")}</button></div>}}/>
        <label>{text("module.field_key")}<input prop:value=move ||new_key.get() maxlength="64" on:input=move |event|new_key.set(event_target_value(&event))/></label><button class="rf-button" type="button" on:click=move |_|{let key=new_key.get_untracked();if key.is_empty()||!key.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_'){return;}document.update(|value|{if !value["fields"]["properties"].is_object(){value["fields"]["properties"]=json!({});}
    if value["fields"]["properties"].get(&key).is_none(){value["fields"]["properties"][&key]=json!({"type":"string","title":key});}});new_key.set(String::new());dirty.set(true);}>{text("module.add_field")}</button></section>}
}

#[component]
fn CourseContent(
    course: RwSignal<Option<Value>>,
    token: ReadSignal<Option<String>>,
    on_expired: Callback<()>,
) -> impl IntoView {
    let lesson_dirty = crate::http::dirty_scope();
    let lessons = RwSignal::new(Vec::<Value>::new());
    let users = RwSignal::new(Vec::<Value>::new());
    let enrollments = RwSignal::new(Vec::<Value>::new());
    let title = RwSignal::new(String::new());
    let content = RwSignal::new(String::new());
    let lesson_id = RwSignal::new(None::<String>);
    let selected_user = RwSignal::new(String::new());
    let busy = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let refresh = RwSignal::new(0_u64);
    let guard = Guard::new();
    let loaded_for = RwSignal::new(None::<String>);
    let loader = guard.clone();
    let course_id = Memo::new(move |_| {
        course
            .get()
            .and_then(|value| value["id"].as_str().map(str::to_owned))
    });
    Effect::new(move |_| {
        let id = course_id.get();
        if id != loaded_for.get_untracked() {
            title.set(String::new());
            content.set(String::new());
            lesson_id.set(None);
            lesson_dirty.set(false);
            loaded_for.set(id.clone());
        }
        let token = token.get();
        refresh.get();
        let ticket = loader.begin();
        lessons.set(vec![]);
        let (Some(id), Some(token)) = (id, token) else {
            return;
        };
        let guard = loader.clone();
        busy.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request(
                "GET",
                &format!("/api/v1/lms/courses/{id}/lessons"),
                &token,
                None,
            )
            .await;
            let enrollment_result = http::request(
                "GET",
                &format!("/api/v1/lms/courses/{id}/enrollments"),
                &token,
                None,
            )
            .await;
            let user_result =
                http::request("GET", "/api/v1/users?per_page=100&page=1", &token, None).await;
            if !guard.accepts(ticket, &token) {
                return;
            }
            busy.set(false);
            if let Ok(value) = enrollment_result {
                enrollments.set(serde_json::from_value(value).unwrap_or_default());
            }
            if let Ok(value) = user_result {
                users.set(value["items"].as_array().cloned().unwrap_or_default());
            }
            match result {
                Ok(value) => lessons.set(serde_json::from_value(value).unwrap_or_default()),
                Err(error) => {
                    message.set(Some(error.message));
                    if error.unauthorized {
                        on_expired.run(());
                    }
                }
            }
        });
    });
    let save_guard = guard.clone();
    let save = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        if busy.get_untracked() {
            return;
        }
        let (Some(id), Some(token)) = (course_id.get_untracked(), token.get_untracked()) else {
            return;
        };
        let existing = lesson_id.get_untracked();
        let path = if let Some(lesson) = &existing {
            format!("/api/v1/lms/courses/{id}/lessons/{lesson}")
        } else {
            format!("/api/v1/lms/courses/{id}/lessons")
        };
        let body = json!({"title":title.get_untracked(),"content":content.get_untracked()});
        let ticket = save_guard.begin();
        let guard = save_guard.clone();
        busy.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request(
                if existing.is_some() { "PUT" } else { "POST" },
                &path,
                &token,
                Some(&body),
            )
            .await;
            if !guard.accepts(ticket, &token) {
                return;
            }
            busy.set(false);
            match result {
                Ok(_) => {
                    title.set(String::new());
                    content.set(String::new());
                    lesson_dirty.set(false);
                    lesson_id.set(None);
                    message.set(Some(text("form.saved").into()));
                    refresh.update(|n| *n = n.wrapping_add(1));
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
    let enroll = move |_| {
        if busy.get_untracked() {
            return;
        }
        let (Some(id), Some(token)) = (course_id.get_untracked(), token.get_untracked()) else {
            return;
        };
        let user = selected_user.get_untracked();
        if !crate::editor::local_id(&user) {
            return;
        }
        let ticket = guard.begin();
        let guard = guard.clone();
        busy.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request(
                "POST",
                &format!("/api/v1/lms/courses/{id}/enrollments"),
                &token,
                Some(&json!({"user_id":user})),
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
    view! {<Show when=move ||course_id.get().is_some()><details class="rf-panel rf-screen-panel rf-layout-options"><summary>{text("module.lessons")}</summary>{move ||message.get().map(|message|view!{<p role="status">{message}</p>})}<ul>{move ||lessons.get().into_iter().map(|lesson|{let label=lesson["title"].as_str().unwrap_or_default().to_owned();view!{<li><button class="rf-text-button" on:click=move |_|{if lesson_dirty.get_untracked()&&!web_sys::window().is_some_and(|window|window.confirm_with_message(text("form.leave")).unwrap_or(false)){return;}lesson_dirty.set(false);title.set(lesson["title"].as_str().unwrap_or_default().into());content.set(lesson["content"].as_str().unwrap_or_default().into());lesson_id.set(lesson["id"].as_str().map(str::to_owned));}>{label}</button></li>}}).collect_view()}</ul><form class="rf-compact-form" on:submit=save.clone()><label>{text("screens.title")}<input required maxlength="300" prop:value=move ||title.get() on:input=move |event|{title.set(event_target_value(&event));lesson_dirty.set(true);}/></label><label>{text("editor.content")}<textarea prop:value=move ||content.get() on:input=move |event|{content.set(event_target_value(&event));lesson_dirty.set(true);}/></label><button class="rf-button primary" disabled=move ||busy.get()>{text("module.save_lesson")}</button><button type="button" class="rf-button" on:click=move |_|{if lesson_dirty.get_untracked()&&!web_sys::window().is_some_and(|window|window.confirm_with_message(text("form.leave")).unwrap_or(false)){return;}lesson_id.set(None);title.set(String::new());content.set(String::new());lesson_dirty.set(false);}>{text("module.new_lesson")}</button></form><h2>{text("module.enrollments")}</h2><ul>{move ||enrollments.get().into_iter().map(|enrollment|view!{<li>{enrollment["display_name"].as_str().unwrap_or_default().to_owned()}" — "{enrollment["progress_pct"].as_u64().unwrap_or(0)}" %"</li>}).collect_view()}</ul><label for="rf-enroll-user">{text("module.enroll_user")}</label><select id="rf-enroll-user" prop:value=move ||selected_user.get() on:change=move |event|selected_user.set(event_target_value(&event))><option value="">{text("module.choose_user")}</option>{move ||users.get().into_iter().map(|user|view!{<option value=user["id"].as_str().unwrap_or_default().to_owned()>{user["display_name"].as_str().unwrap_or_default().to_owned()}</option>}).collect_view()}</select><button class="rf-button" disabled=move ||busy.get() on:click=enroll.clone()>{text("module.enroll")}</button></details></Show>}
}
#[component]
fn GalleryContent(
    gallery: RwSignal<Option<Value>>,
    token: ReadSignal<Option<String>>,
    on_expired: Callback<()>,
) -> impl IntoView {
    let items = RwSignal::new(Vec::<Value>::new());
    let message = RwSignal::new(None::<String>);
    let busy = RwSignal::new(false);
    let refresh = RwSignal::new(0_u64);
    let guard = Guard::new();
    let loader = guard.clone();
    let gallery_id = Memo::new(move |_| {
        gallery
            .get()
            .and_then(|value| value["id"].as_str().map(str::to_owned))
    });
    Effect::new(move |_| {
        let id = gallery_id.get();
        let token = token.get();
        refresh.get();
        let ticket = loader.begin();
        let (Some(id), Some(token)) = (id, token) else {
            busy.set(false);
            return;
        };
        let guard = loader.clone();
        busy.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let result =
                http::request("GET", &format!("/api/v1/galleries/{id}"), &token, None).await;
            if !guard.accepts(ticket, &token) {
                return;
            }
            busy.set(false);
            match result {
                Ok(value) => items.set(value["items"].as_array().cloned().unwrap_or_default()),
                Err(error) => {
                    message.set(Some(error.message));
                    if error.unauthorized {
                        on_expired.run(());
                    }
                }
            }
        });
    });
    let choose = Callback::new(move |media: Value| {
        if busy.get_untracked() {
            return;
        }
        let (Some(id), Some(token)) = (gallery_id.get_untracked(), token.get_untracked()) else {
            return;
        };
        let kind = if media["mime_type"]
            .as_str()
            .is_some_and(|mime| mime.starts_with("video/"))
        {
            "VideoLocal"
        } else {
            "Image"
        };
        let body = json!({"item_type":kind,"media_id":media["id"],"title":media["original_name"],"caption":media["caption"],"sort_order":items.get_untracked().len()});
        let ticket = guard.begin();
        let guard = guard.clone();
        busy.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request(
                "POST",
                &format!("/api/v1/galleries/{id}/items"),
                &token,
                Some(&body),
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
    });
    view! {<Show when=move ||gallery_id.get().is_some()><section class="rf-panel rf-screen-panel"><h2>{text("module.gallery_items")}</h2><crate::media_ui::Picker token on_choose=choose/>{move ||message.get().map(|message|view!{<p role="status">{message}</p>})}<ul>{move ||items.get().into_iter().map(|item|view!{<li>{item["title"].as_str().unwrap_or_default().to_owned()}" — "{item["item_type"].as_str().unwrap_or_default().to_owned()}</li>}).collect_view()}</ul></section></Show>}
}

#[component]
fn FormNotifications(document: RwSignal<Value>, dirty: RwSignal<bool>) -> impl IntoView {
    view! {<section class="rf-panel rf-screen-panel"><h2>{text("notice.title")}</h2><p>{text("notice.hint")}</p><label><input type="checkbox" disabled=move ||document.get().get("settings").is_some_and(|v|!v.is_null()&&!v.is_object()) prop:checked=move ||document.get().pointer("/settings/notifications/capture_owner").and_then(Value::as_bool)==Some(true) on:change=move |event|{let enabled=event_target_checked(&event);document.update(|value|{if !value["settings"].is_object(){value["settings"]=json!({});}value["settings"]["notifications"]=json!({"capture_owner":enabled});});dirty.set(true);}/>{text("notice.enable")}</label><p class="rf-description">{text("notice.future")}</p></section>}
}
