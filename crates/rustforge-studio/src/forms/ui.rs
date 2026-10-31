use super::{Control, Field, FormKind};
use crate::{
    http::{self, Guard},
    messages::text,
};
use leptos::prelude::*;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
pub(crate) type Values = BTreeMap<String, String>;
pub(crate) fn selected(fields: &[Field], value: &Value) -> Values {
    fields
        .iter()
        .map(|field| {
            let raw = &value[field.key];
            let string = match raw {
                Value::String(v) => v.clone(),
                Value::Bool(v) if field.key == "discussion.default_comment_status" => {
                    if *v { "open" } else { "closed" }.into()
                }
                Value::Bool(v) => v.to_string(),
                Value::Number(v) => v.to_string(),
                Value::Array(items) => items
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join("\n"),
                _ => field.default.into(),
            };
            (field.key.into(), string)
        })
        .collect()
}
pub(crate) fn patch(
    fields: &[Field],
    values: &Values,
    original: &Values,
) -> Result<Value, &'static str> {
    let mut patch = Map::new();
    for field in fields {
        let Some(value) = values.get(field.key) else {
            continue;
        };
        if original.get(field.key) == Some(value) {
            continue;
        }
        let value = match field.control {
            Control::Bool => Value::Bool(value == "true"),
            Control::Lines => serde_json::json!(
                value
                    .lines()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .collect::<Vec<_>>()
            ),
            Control::Page => {
                if value.is_empty() {
                    Value::Null
                } else {
                    if !crate::editor::local_id(value) {
                        return Err("form.invalid");
                    }
                    Value::String(value.clone())
                }
            }
            Control::Number(min, max) => {
                let n = value.parse::<i64>().map_err(|_| "form.invalid")?;
                if !(min..=max).contains(&n) {
                    return Err("form.invalid");
                }
                Value::from(n)
            }
            Control::Select(options) => {
                if !options.iter().any(|(key, _)| key == value) {
                    return Err("form.invalid");
                }
                Value::String(value.clone())
            }
            _ => Value::String(value.clone()),
        };
        patch.insert(field.key.into(), value);
    }
    Ok(Value::Object(patch))
}

#[component]
pub(crate) fn FormField(
    field: Field,
    values: RwSignal<Values>,
    dirty: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) -> impl IntoView {
    let pages = expect_context::<RwSignal<Vec<(String, String)>>>();
    let id = format!("rf-field-{}", field.key.replace('.', "-"));
    let label = text(field.label);
    let edit = move |value: String| {
        values.update(|all| {
            all.insert(field.key.into(), value);
        });
        dirty.set(true);
        message.set(None);
    };
    let value = move || values.with(|all| all.get(field.key).cloned().unwrap_or_default());
    view! {<div class="rf-form-row"><label for=id.clone()>{label}</label><div class="rf-form-control">{match field.control{
       Control::Bool=>view!{<label class="rf-checkbox"><input id=id type="checkbox" prop:checked=move ||value()=="true" on:change=move |event|edit(event_target_checked(&event).to_string())/>{label}</label>}.into_any(),
       Control::Page=>view!{<select id=id prop:value=value on:change=move |event|edit(event_target_value(&event))><option value="">{text("form.none")}</option>{move ||{let current=value();let pages=pages.get();let mut options=pages.clone();if !current.is_empty()&&!pages.iter().any(|(id,_)|id==&current){options.push((current,text("form.missing_page").into()));}options.into_iter().map(|(id,label)|view!{<option value=id>{label}</option>}).collect_view()}}</select>}.into_any(),
    Control::Area|Control::Lines=>view!{<textarea id=id maxlength="16384" prop:value=value on:input=move |event|edit(event_target_value(&event))/>}.into_any(),
       Control::Select(options)=>view!{<select id=id prop:value=value on:change=move |event|edit(event_target_value(&event))>{options.iter().map(|(key,label)|view!{<option value=*key>{text(label)}</option>}).collect_view()}</select>}.into_any(),
       Control::Number(min,max)=>view!{<input id=id type="number" min=min max=max step="1" required prop:value=value on:input=move |event|edit(event_target_value(&event))/>}.into_any(),
       other=>view!{<input id=id type=match other{Control::Email=>"email",Control::Password=>"password",Control::Url=>"url",_=>"text"} maxlength="2048" prop:value=value on:input=move |event|edit(event_target_value(&event))/>}.into_any()
       }}</div></div>}
}

#[component]
pub fn SettingsForm(
    kind: FormKind,
    token: ReadSignal<Option<String>>,
    on_expired: Callback<()>,
    #[prop(optional)] target: Option<String>,
) -> impl IntoView {
    let target = StoredValue::new(target);
    let pages = RwSignal::new(Vec::<(String, String)>::new());
    provide_context(pages);
    let values = RwSignal::new(Values::new());
    let original = RwSignal::new(Values::new());
    let retention_ack = RwSignal::new(false);
    Effect::new(move |_| {
        values.get();
        retention_ack.set(false);
    });
    let retention_confirm = Memo::new(move |_| {
        kind == FormKind::FormRetention
            && values
                .get()
                .get("forms.retention")
                .is_some_and(|v| v != "none")
            && values.get().get("forms.retention") != original.get().get("forms.retention")
    });
    let dirty = crate::http::dirty_scope();
    let busy = RwSignal::new(false);
    let loaded = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let success = RwSignal::new(false);
    let profile_id = RwSignal::new(String::new());
    let refresh = RwSignal::new(0_u64);
    let guard = Guard::new();
    let loader = guard.clone();
    Effect::new(move |_| {
        let current = token.get();
        refresh.get();
        let ticket = loader.begin();
        loaded.set(false);
        values.set(Values::new());
        original.set(Values::new());
        dirty.set(false);
        message.set(None);
        let Some(current) = current else {
            busy.set(false);
            return;
        };
        if kind == FormKind::NewUser {
            let defaults = selected(&kind.fields(), &Value::Null);
            values.set(defaults.clone());
            original.set(defaults);
            loaded.set(true);
            busy.set(false);
            return;
        }
        busy.set(true);
        let path = if kind == FormKind::Profile {
            "/api/v1/auth/me".into()
        } else if kind == FormKind::User {
            format!("/api/v1/users/{}", target.get_value().unwrap_or_default())
        } else {
            if kind == FormKind::Analytics {
                "/api/v1/analytics/settings".into()
            } else {
                "/api/v1/settings".into()
            }
        };
        let guard = loader.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request("GET", &path, &current, None).await;
            let mut page_choices = Vec::new();
            if kind == FormKind::Reading && result.is_ok() {
                for page in 1..=100_u64 {
                    let page_result=http::request("GET",&format!("/api/v1/entries?content_type=page&status=published&page={page}&per_page=100"),&current,None).await;
                    let Ok(value) = page_result else { break };
                    let now = js_sys::Date::now() / 1000.;
                    if let Some(items) = value["items"].as_array() {
                        for item in items {
                            let visible = item["published_at"]
                                .as_str()
                                .and_then(|date| chrono::DateTime::parse_from_rfc3339(date).ok())
                                .is_some_and(|date| date.timestamp() as f64 <= now)
                                && item["scheduled_unpublish_at"]
                                    .as_str()
                                    .and_then(|date| {
                                        chrono::DateTime::parse_from_rfc3339(date).ok()
                                    })
                                    .is_none_or(|date| date.timestamp() as f64 > now);
                            if visible
                                && let (Some(id), Some(title)) = (
                                    item["id"].as_str().filter(|id| crate::editor::local_id(id)),
                                    item["title"].as_str(),
                                )
                            {
                                page_choices.push((id.to_owned(), title.to_owned()));
                            }
                        }
                    }
                    if page >= value["total_pages"].as_u64().unwrap_or(0) {
                        break;
                    }
                }
            }

            if !guard.accepts(ticket, &current) {
                return;
            }
            busy.set(false);
            pages.set(page_choices);
            match result {
                Ok(data) => {
                    profile_id.set(data["id"].as_str().unwrap_or_default().into());
                    let selected = selected(
                        &kind.fields(),
                        if kind == FormKind::Styles {
                            &data["site.design_tokens"]
                        } else {
                            &data
                        },
                    );
                    values.set(selected.clone());
                    original.set(selected);
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
    let save_guard = guard;
    let save = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        if busy.get_untracked() || !loaded.get_untracked() {
            return;
        }
        if retention_confirm.get_untracked() && !retention_ack.get_untracked() {
            message.set(Some(text("retention.confirm_needed").into()));
            return;
        }
        let Some(current) = token.get_untracked() else {
            return;
        };
        let mut body = match patch(
            &kind.fields(),
            &values.get_untracked(),
            &original.get_untracked(),
        ) {
            Ok(body) => body,
            Err(key) => {
                message.set(Some(text(key).into()));
                success.set(false);
                return;
            }
        };
        if matches!(kind, FormKind::NewUser | FormKind::Analytics) {
            body = match patch(&kind.fields(), &values.get_untracked(), &Values::new()) {
                Ok(value) => value,
                Err(key) => {
                    message.set(Some(text(key).into()));
                    return;
                }
            };
        }
        if body.as_object().is_none_or(|v| v.is_empty()) {
            dirty.set(false);
            return;
        }
        let path = if kind == FormKind::NewUser {
            "/api/v1/users".into()
        } else if matches!(kind, FormKind::Profile | FormKind::User) {
            let id = profile_id.get_untracked();
            if id.len() != 36 || !id.bytes().all(|b| b.is_ascii_hexdigit() || b == b'-') {
                return;
            }
            format!("/api/v1/users/{id}")
        } else {
            if kind == FormKind::Analytics {
                "/api/v1/analytics/settings".into()
            } else {
                "/api/v1/settings".into()
            }
        };
        let ticket = save_guard.begin();
        let guard = save_guard.clone();
        busy.set(true);
        message.set(None);
        success.set(false);
        wasm_bindgen_futures::spawn_local(async move {
            let result = async {
                let body = if kind == FormKind::Styles {
                    let latest = http::request("GET", "/api/v1/settings", &current, None).await?;
                    let mut tokens = latest
                        .get("site.design_tokens")
                        .filter(|value| value.is_object())
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!({}));
                    for (key, value) in body.as_object().into_iter().flatten() {
                        tokens[key] = value.clone();
                    }
                    serde_json::json!({"site.design_tokens":tokens})
                } else {
                    body
                };
                http::request(
                    if kind == FormKind::NewUser {
                        "POST"
                    } else {
                        "PUT"
                    },
                    &path,
                    &current,
                    Some(&body),
                )
                .await
            }
            .await;
            if !guard.accepts(ticket, &current) {
                return;
            }
            busy.set(false);
            match result {
                Ok(data) => {
                    let selected = selected(
                        &kind.fields(),
                        if kind == FormKind::Styles {
                            &data["site.design_tokens"]
                        } else {
                            &data
                        },
                    );
                    if kind == FormKind::NewUser {
                        dirty.set(false);
                        if let Some(id) =
                            data["id"].as_str().filter(|id| crate::editor::local_id(id))
                            && let Some(window) = web_sys::window()
                        {
                            let _ = window.location().set_hash(&format!("user:{id}"));
                        }
                        return;
                    }

                    values.set(selected.clone());
                    original.set(selected);
                    dirty.set(false);
                    success.set(true);
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
    view! {<section class="rf-screen"><div class="rf-page-heading"><h1>{text(kind.title())}</h1></div><p class="rf-description">{text("form.hint")}</p><Show when=move ||kind==FormKind::FormRetention><p class="rf-notice">{text("retention.warning")}</p></Show>
    {move ||message.get().map(|message|view!{<p class="rf-notice" class:rf-notice-success=move ||success.get() role=if success.get(){"status"}else{"alert"}>{message}</p>})}
    <Show when=move ||busy.get()&&!loaded.get()><p role="status">{text("screens.loading")}</p></Show>
    <Show when=move ||!loaded.get()&&!busy.get()><button class="rf-button" disabled=move ||token.get().is_none() on:click=move |_|refresh.update(|n|*n=n.wrapping_add(1))>{text("form.reload")}</button></Show>
    <Show when=move ||loaded.get()><form on:submit=save.clone()><fieldset class="rf-form" disabled=move ||busy.get()><legend class="rf-sr-only">{text(kind.title())}</legend>{kind.fields().into_iter().map(|field|view!{<FormField field values dirty message/>}).collect_view()}<Show when=move ||retention_confirm.get()><label><input type="checkbox" prop:checked=move ||retention_ack.get() on:change=move |event|retention_ack.set(event_target_checked(&event))/>{text("retention.confirm")}</label></Show><div class="rf-form-actions"><button class="rf-button primary" type="submit" disabled=move ||!dirty.get()||(retention_confirm.get()&&!retention_ack.get())>{move ||if busy.get(){text("form.saving")}else{text("form.save")}}</button><button class="rf-button" type="button" disabled=move ||!dirty.get() on:click=move |_|{values.set(original.get_untracked());dirty.set(false);message.set(None);}>{text("form.cancel")}</button><Show when=move ||dirty.get()><span class="rf-description">{text("form.unsaved")}</span></Show></div></fieldset></form></Show>
    <Show when=move ||matches!(kind,FormKind::Profile|FormKind::User)&&loaded.get()><ProfileSessions user_id=profile_id token on_expired/></Show>
    </section>}
}

#[component]
fn ProfileSessions(
    user_id: RwSignal<String>,
    token: ReadSignal<Option<String>>,
    on_expired: Callback<()>,
) -> impl IntoView {
    let sessions = RwSignal::new(None::<Vec<Value>>);
    let busy = RwSignal::new(false);
    let confirm = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let refresh = RwSignal::new(0_u64);
    let guard = Guard::new();
    let loader = guard.clone();
    Effect::new(move |_| {
        let id = user_id.get();
        let current = token.get();
        refresh.get();
        let ticket = loader.begin();
        sessions.set(None);
        let Some(current) = current else { return };
        if id.len() != 36 {
            return;
        }
        busy.set(true);
        let guard = loader.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request(
                "GET",
                &format!("/api/v1/users/{id}/sessions"),
                &current,
                None,
            )
            .await;
            if !guard.accepts(ticket, &current) {
                return;
            }
            busy.set(false);
            match result {
                Ok(data) => match serde_json::from_value(data) {
                    Ok(data) => sessions.set(Some(data)),
                    Err(_) => error.set(Some(text("screens.response").into())),
                },
                Err(failure) => {
                    error.set(Some(failure.message));
                    if failure.unauthorized {
                        on_expired.run(());
                    }
                }
            }
        });
    });
    let revoke = move |_| {
        let Some(current) = token.get_untracked() else {
            return;
        };
        let id = user_id.get_untracked();
        if id.len() != 36 {
            return;
        }
        let ticket = guard.begin();
        let guard = guard.clone();
        busy.set(true);
        confirm.set(false);
        error.set(None);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request(
                "DELETE",
                &format!("/api/v1/users/{id}/sessions"),
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
                    refresh.update(|n| *n = n.wrapping_add(1));
                    error.set(Some(text("form.sessions_revoked").into()));
                }
                Err(failure) => {
                    error.set(Some(failure.message));
                    if failure.unauthorized {
                        on_expired.run(());
                    }
                }
            }
        });
    };
    view! {<section class="rf-panel rf-screen-panel rf-session-panel"><h2>{text("form.sessions")}</h2><p>{text("form.sessions_hint")}</p>{move ||error.get().map(|error|view!{<p role="status">{error}</p>})}{move ||sessions.get().map(|sessions|view!{<p>{sessions.len()}" "{text("form.active_sessions")}</p><ul>{sessions.into_iter().map(|session|view!{<li>{session["created_at"].as_str().unwrap_or("—").to_owned()}" → "{session["expires_at"].as_str().unwrap_or("—").to_owned()}</li>}).collect_view()}</ul>})}<Show when=move ||confirm.get() fallback=move ||view!{<button class="rf-button" disabled=move ||busy.get() on:click=move |_|confirm.set(true)>{text("form.revoke")}</button>}><p>{text("form.revoke_confirm")}</p><button class="rf-button" disabled=move ||busy.get() on:click=revoke.clone()>{text("form.revoke_yes")}</button><button class="rf-button" on:click=move |_|confirm.set(false)>{text("form.cancel")}</button></Show></section>}
}
