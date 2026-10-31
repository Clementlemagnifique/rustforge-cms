use crate::{
    http::{self, Guard},
    messages::text,
};
use leptos::prelude::*;
use serde_json::{Value, json};
#[component]
pub fn Themes(token: ReadSignal<Option<String>>, on_expired: Callback<()>) -> impl IntoView {
    let themes = RwSignal::new(Vec::<Value>::new());
    let busy = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let pending = RwSignal::new(None::<String>);
    let refresh = RwSignal::new(0_u64);
    let guard = Guard::new();
    let loader = guard.clone();
    Effect::new(move |_| {
        let token = token.get();
        refresh.get();
        themes.set(vec![]);
        let ticket = loader.begin();
        let Some(token) = token else { return };
        let guard = loader.clone();
        busy.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request("GET", "/api/v1/themes", &token, None).await;
            if !guard.accepts(ticket, &token) {
                return;
            }
            busy.set(false);
            match result {
                Ok(value) => themes.set(serde_json::from_value(value).unwrap_or_default()),
                Err(error) => {
                    message.set(Some(error.message));
                    if error.unauthorized {
                        on_expired.run(());
                    }
                }
            }
        });
    });
    let activate = move |_| {
        let Some(token) = token.get_untracked() else {
            return;
        };
        let Some(slug) = pending.get_untracked() else {
            return;
        };
        if busy.get_untracked()
            || !slug
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        {
            return;
        }
        let ticket = guard.begin();
        let guard = guard.clone();
        busy.set(true);
        pending.set(None);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request(
                "POST",
                &format!("/api/v1/themes/{slug}/activate"),
                &token,
                None,
            )
            .await;
            if !guard.accepts(ticket, &token) {
                return;
            }
            busy.set(false);
            match result {
                Ok(_) => {
                    message.set(Some(text("appearance.activated").into()));
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
    view! {<h1>{text("appearance.themes")}</h1><nav class="rf-form-actions"><a class="rf-button" href="#appearance:styles">{text("appearance.styles")}</a><a class="rf-button" href="#appearance:menus">{text("appearance.menus")}</a><a class="rf-button" href="/" target="_blank" rel="noopener noreferrer">{text("shell.site")}</a></nav>{move ||message.get().map(|message|view!{<p class="rf-notice" role="status">{message}</p>})}<Show when=move ||pending.get().is_some()><div class="rf-notice"><p>{text("appearance.activate_confirm")}</p><button class="rf-button primary" disabled=move ||busy.get() on:click=activate.clone()>{text("appearance.activate")}</button><button class="rf-button" on:click=move |_|pending.set(None)>{text("form.cancel")}</button></div></Show><div class="rf-screen-grid">{move ||themes.get().into_iter().map(|theme|{let slug=theme["slug"].as_str().unwrap_or_default().to_owned();let active=theme["is_active"].as_bool().unwrap_or(false);view!{<article class="rf-panel rf-screen-panel"><h2>{theme["name"].as_str().unwrap_or_default().to_owned()}</h2><p>{theme["description"].as_str().unwrap_or_default().to_owned()}</p><p class="rf-description">{theme["author"].as_str().unwrap_or_default().to_owned()}" · "{theme["version"].as_str().unwrap_or_default().to_owned()}</p><button class="rf-button" disabled=move ||active||busy.get() on:click=move |_|pending.set(Some(slug.clone()))>{if active{text("appearance.active")}else{text("appearance.activate")}}</button></article>}}).collect_view()}</div>}
}

#[component]
pub fn PublicMenus(token: ReadSignal<Option<String>>, on_expired: Callback<()>) -> impl IntoView {
    let menus = RwSignal::new(Value::Null);
    let baseline = RwSignal::new(Value::Null);
    let location = RwSignal::new("header".to_owned());
    let busy = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let loaded = RwSignal::new(false);
    let dirty = crate::http::dirty_scope();
    let guard = Guard::new();
    let loader = guard.clone();
    Effect::new(move |_| {
        let token = token.get();
        loaded.set(false);
        let ticket = loader.begin();
        let Some(token) = token else { return };
        let guard = loader.clone();
        busy.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request("GET", "/api/v1/settings", &token, None).await;
            if !guard.accepts(ticket, &token) {
                return;
            }
            busy.set(false);
            match result {
                Ok(value) => {
                    let value = value
                        .get("site.menus")
                        .filter(|value| value.is_object())
                        .cloned()
                        .unwrap_or_else(|| json!({"header":[],"footer":[]}));
                    baseline.set(value.clone());
                    menus.set(value);
                    loaded.set(true);
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
    let save = move |_| {
        if busy.get_untracked() {
            return;
        }
        let Some(token) = token.get_untracked() else {
            return;
        };
        let changed = menus.get_untracked();
        let original = baseline.get_untracked();
        let guard = guard.clone();
        let ticket = guard.begin();
        busy.set(true);
        message.set(None);
        wasm_bindgen_futures::spawn_local(async move {
            let result = async {
                let fresh = http::request("GET", "/api/v1/settings", &token, None).await?;
                let mut combined = fresh
                    .get("site.menus")
                    .filter(|value| value.is_object())
                    .cloned()
                    .unwrap_or_else(|| json!({}));
                for key in ["header", "footer"] {
                    if changed[key] != original[key] {
                        combined[key] = changed[key].clone();
                    }
                }
                http::request(
                    "PUT",
                    "/api/v1/settings",
                    &token,
                    Some(&json!({"site.menus":combined})),
                )
                .await
            }
            .await;
            if !guard.accepts(ticket, &token) {
                return;
            }
            busy.set(false);
            match result {
                Ok(value) => {
                    let value = value["site.menus"].clone();
                    baseline.set(value.clone());
                    menus.set(value);
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
    view! {<h1>{text("appearance.menus")}</h1><p>{text("appearance.menus_hint")}</p>{move ||message.get().map(|message|view!{<p class="rf-notice" role="status">{message}</p>})}<Show when=move ||loaded.get()><fieldset class="rf-form" disabled=move ||busy.get()><legend class="rf-sr-only">{text("appearance.menus")}</legend><label for="rf-menu-location">{text("appearance.location")}</label><select id="rf-menu-location" prop:value=move ||location.get() on:change=move |event|location.set(event_target_value(&event))><option value="header">{text("appearance.header")}</option><option value="footer">{text("appearance.footer")}</option></select>
        <For each={move ||menus.get()[&location.get()].as_array().cloned().unwrap_or_default().into_iter().enumerate().map(|(index,item)|(index,item["id"].as_str().unwrap_or_default().to_owned())).collect::<Vec<_>>()} key=|(index,id)|(*index,id.clone()) children=move |(index,_)|view!{<MenuItem index menus location dirty/>}/>
        <div class="rf-form-actions"><button class="rf-button" on:click=move |_|{let key=location.get_untracked();menus.update(|menus|{if !menus[&key].is_array(){menus[&key]=json!([]);}
    if let Some(items)=menus[&key].as_array_mut()&& items.len()<64{items.push(json!({"id":format!("rf-{}-{}",js_sys::Date::now() as u64,items.len()),"label":"","url":"/","type":"custom","target":"_self"}));}});dirty.set(true);}>{text("appearance.add_link")}</button><button class="rf-button primary" disabled=move ||!dirty.get() on:click=save.clone()>{text("form.save")}</button><button class="rf-button" disabled=move ||!dirty.get() on:click=move |_|{menus.set(baseline.get_untracked());dirty.set(false);}>{text("form.cancel")}</button></div></fieldset></Show>}
}
#[component]
fn MenuItem(
    index: usize,
    menus: RwSignal<Value>,
    location: RwSignal<String>,
    dirty: RwSignal<bool>,
) -> impl IntoView {
    let edit = Callback::new(move |(key, value): (&'static str, Value)| {
        let location = location.get_untracked();
        menus.update(|menus| {
            if let Some(item) = menus[&location]
                .as_array_mut()
                .and_then(|items| items.get_mut(index))
            {
                item[key] = value;
            }
        });
        dirty.set(true);
    });
    let reorder = move |direction: i32| {
        let key = location.get_untracked();
        menus.update(|menus| {
            if let Some(items) = menus[&key].as_array_mut() {
                let next = index as i32 + direction;
                if next >= 0 && (next as usize) < items.len() {
                    items.swap(index, next as usize);
                }
            }
        });
        dirty.set(true);
    };
    view! {<section class="rf-panel rf-screen-panel rf-public-menu-item"><h2>{text("appearance.link")} " "{index+1}</h2>{[("label","appearance.label"),("url","editor.url"),("subtitle","appearance.subtitle"),("badge","appearance.badge"),("classes","appearance.classes")].into_iter().map(|(key,label)|view!{<label>{text(label)}<input maxlength="2048" prop:value=move ||menus.get()[&location.get()][index][key].as_str().unwrap_or_default().to_owned() on:input=move |event|edit.run((key,json!(event_target_value(&event))))/></label>}).collect_view()}<label>{text("appearance.parent")}<select prop:value=move ||menus.get()[&location.get()][index]["parent_id"].as_str().unwrap_or_default().to_owned() on:change=move |event|{let id=event_target_value(&event);edit.run(("parent_id",if id.is_empty(){Value::Null}else{json!(id)}));}><option value="">{text("appearance.no_parent")}</option>{move ||menus.get()[&location.get()].as_array().cloned().unwrap_or_default().into_iter().enumerate().filter(|(i,_)|*i!=index).filter_map(|(_,item)|item["id"].as_str().map(|id|(id.to_owned(),item["label"].as_str().unwrap_or_default().to_owned()))).map(|(id,label)|view!{<option value=id>{label}</option>}).collect_view()}</select></label><label><input type="checkbox" prop:checked=move ||menus.get()[&location.get()][index]["target"]=="_blank" on:change=move |event|edit.run(("target",json!(if event_target_checked(&event){"_blank"}else{"_self"}))) />{text("appearance.new_tab")}</label><div class="rf-form-actions"><button class="rf-button" disabled=index==0 on:click=move |_|reorder(-1)>{text("layout.up")}</button><button class="rf-button" on:click=move |_|reorder(1)>{text("layout.down")}</button><button class="rf-button" on:click=move |_|{let key=location.get_untracked();menus.update(|menus|if let Some(items)=menus[&key].as_array_mut()&& index<items.len(){let removed=items.remove(index);let removed_id=removed["id"].as_str().unwrap_or_default();for item in items{if item["parent_id"].as_str()==Some(removed_id){item["parent_id"]=Value::Null;}}});dirty.set(true);}>{text("editor.remove")}</button></div></section>}
}
