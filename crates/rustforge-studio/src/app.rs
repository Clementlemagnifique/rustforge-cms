//! Première coquille native : menus autorisés depuis l'API, laboratoire de document partagé.
use crate::{
    dashboard::ui::Dashboard,
    messages::text,
    navigation::{self, MenuItem, MenuNode},
    screens::ui::RoutedScreen,
    session::SessionEpoch,
    shell::{Icon, MenuBranch, TopBar},
};
use leptos::prelude::*;
use serde_json::Value;
use wasm_bindgen::JsCast;

fn restore_menu_focus() {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    let Some(active) = document.active_element() else {
        return;
    };
    for (panel, trigger) in [
        (
            "#rf-screen-options-wrap",
            "[aria-controls=rf-screen-options-wrap]",
        ),
        (
            "#rf-contextual-help-wrap",
            "[aria-controls=rf-contextual-help-wrap]",
        ),
    ] {
        if active.closest(panel).ok().flatten().is_some() {
            if let Some(button) = document
                .query_selector(trigger)
                .ok()
                .flatten()
                .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
            {
                let _ = button.focus();
            }
            return;
        }
    }
    for selector in [".rf-bar-dropdown", ".rf-menu-root"] {
        if let Some(container) = active.closest(selector).ok().flatten() {
            if let Some(trigger) = container
                .query_selector(":scope > button, :scope > a")
                .ok()
                .flatten()
                .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
            {
                let _ = trigger.focus();
            }
            break;
        }
    }
}

pub(crate) fn session_token() -> Result<Option<String>, &'static str> {
    web_sys::window()
        .ok_or(text("error.browser"))?
        .local_storage()
        .map_err(|_| text("error.storage"))?
        .ok_or(text("error.storage"))?
        .get_item("rustforge_token")
        .map_err(|_| text("error.session_read"))
}

#[component]
pub fn App() -> impl IntoView {
    let user_id = RwSignal::new(None::<String>);
    provide_context(user_id);
    let dirty = RwSignal::new(false);
    provide_context(dirty);
    let dirty_scopes = RwSignal::new(std::collections::BTreeSet::<String>::new());
    provide_context(crate::http::DirtyScopes(dirty_scopes));
    Effect::new(move |_| dirty.set(!dirty_scopes.get().is_empty()));
    let list_states = RwSignal::new(std::collections::BTreeMap::<
        String,
        crate::screens::ListState,
    >::new());
    provide_context(list_states);
    let dashboard_prefs = RwSignal::new(crate::dashboard::layout::Preferences::default());
    provide_context(dashboard_prefs);
    let (profile_name, set_profile_name) = signal(None::<String>);
    let (menu_close, set_menu_close) = signal(0_u64);
    let (active_token, set_active_token) = signal(None::<String>);
    provide_context(active_token);
    let (menus, set_menus) = signal(Vec::<MenuNode>::new());
    provide_context(menus);
    let read_route = || {
        web_sys::window()
            .and_then(|w| w.location().hash().ok())
            .and_then(|h| navigation::route_from_hash(&h))
            .unwrap_or_else(|| "dashboard".into())
    };
    let (route, set_route) = signal(read_route());

    Effect::new(move |_| {
        let title = if active_token.get().is_none() {
            text("login.title").to_owned()
        } else if crate::editor::Location::parse(&route.get()).is_some() {
            text("editor.title").to_owned()
        } else {
            crate::shell::find_route(&menus.get(), &route.get())
                .map(|node| node.label)
                .unwrap_or_else(|| text("title").into())
        };
        if let Some(document) = web_sys::window().and_then(|window| window.document()) {
            document.set_title(&format!("{title} ‹ RustForge CMS"));
        }
    });
    let (status, set_status) = signal(text("session.idle"));
    let (busy, set_busy) = signal(false);
    let (collapsed, set_collapsed) = signal(false);
    let (options, set_options) = signal(false);
    let (help, set_help) = signal(false);
    let (compact, set_compact) = signal(false);
    let (mobile_open, set_mobile_open) = signal(false);
    let main_ref = NodeRef::<leptos::html::Main>::new();
    let epoch = RwSignal::new(SessionEpoch::default());
    let observed_token = RwSignal::new(session_token().ok().flatten());
    let invalidate = move || {
        dirty_scopes.set(Default::default());
        user_id.set(None);
        dashboard_prefs.set(Default::default());
        list_states.set(Default::default());
        set_compact.set(false);
        set_profile_name.set(None);
        set_menu_close.update(|value| *value = value.wrapping_add(1));
        set_active_token.set(None);
        epoch.update(SessionEpoch::invalidate);
        observed_token.set(session_token().ok().flatten());
        set_menus.set(Vec::new());
        set_busy.set(false);
        set_status.set(text("session.changed"));
    };
    let leave_listener = window_event_listener(leptos::ev::beforeunload, move |event| {
        if dirty.get_untracked() {
            event.prevent_default();
            event.set_return_value("");
        }
    });
    let click_listener = window_event_listener(leptos::ev::click, move |event| {
        if !dirty.get_untracked() {
            return;
        }
        let Some(target) = event
            .target()
            .and_then(|node| node.dyn_into::<web_sys::Element>().ok())
        else {
            return;
        };
        let Some(link) = target.closest("a[href]").ok().flatten() else {
            return;
        };
        let href = link.get_attribute("href").unwrap_or_default();
        if href == "#rf-main"
            || href.is_empty()
            || link.get_attribute("target").as_deref() == Some("_blank")
            || event.ctrl_key()
            || event.meta_key()
            || event.shift_key()
        {
            return;
        }
        if web_sys::window().is_some_and(|window| {
            !window
                .confirm_with_message(text("form.leave"))
                .unwrap_or(false)
        }) {
            event.prevent_default();
            event.stop_propagation();
        }
    });
    let hash_listener = window_event_listener(leptos::ev::hashchange, move |_| {
        set_route.set(read_route());
        set_menu_close.update(|value| *value = value.wrapping_add(1));
        set_mobile_open.set(false);
    });
    let storage_listener = window_event_listener(leptos::ev::storage, move |event| {
        if event.key().is_none_or(|key| key == "rustforge_token") {
            invalidate();
        }
    });
    let focus_listener = window_event_listener(leptos::ev::focus, move |_| {
        if session_token().is_err()
            || session_token().ok().flatten() != observed_token.get_untracked()
        {
            invalidate();
        }
    });
    let keyboard_listener = window_event_listener(leptos::ev::keydown, move |event| {
        if event.is_instance_of::<web_sys::KeyboardEvent>() && event.key() == "Escape" {
            restore_menu_focus();
            set_menu_close.update(|value| *value = value.wrapping_add(1));
            set_options.set(false);
            set_help.set(false);
            set_mobile_open.set(false);
        }
    });
    on_cleanup(move || {
        leave_listener.remove();
        click_listener.remove();
        hash_listener.remove();
        storage_listener.remove();
        focus_listener.remove();
        keyboard_listener.remove();
    });
    let load = Callback::new(move |_: ()| {
        if busy.get_untracked() {
            return;
        }
        let token = match session_token() {
            Ok(Some(token)) if !token.is_empty() => token,
            result => {
                invalidate();
                set_status.set(result.err().unwrap_or(text("error.login")));
                return;
            }
        };
        observed_token.set(Some(token.clone()));
        let mut guard = epoch.get_untracked();
        let ticket = guard.begin();
        epoch.set(guard);
        set_active_token.set(None);
        set_profile_name.set(None);
        // Clear former permissions even when this refresh fails.
        set_menus.set(Vec::new());
        set_busy.set(true);
        set_status.set(text("session.loading"));
        wasm_bindgen_futures::spawn_local(async move {
            let result = async {
                let response = gloo_net::http::Request::get("/api/v1/admin/navigation")
                    .header("Authorization", &format!("Bearer {token}"))
                    .send()
                    .await
                    .map_err(|_| text("error.api"))?;
                if !response.ok() {
                    return Err(text("error.expired"));
                }
                let data: Value = response.json().await.map_err(|_| text("error.response"))?;
                let items: Vec<MenuItem> = serde_json::from_value(
                    data.get("menu_items").unwrap_or(&data["items"]).clone(),
                )
                .map_err(|_| text("error.navigation"))?;
                let tree = navigation::project(items).map_err(|_| text("error.navigation"))?;
                let response = gloo_net::http::Request::get("/api/v1/auth/me")
                    .header("Authorization", &format!("Bearer {token}"))
                    .send()
                    .await
                    .map_err(|_| text("error.api"))?;
                if !response.ok() {
                    return Err(text("error.expired"));
                }
                let profile: Value = response.json().await.map_err(|_| text("error.response"))?;
                let name = profile
                    .get("display_name")
                    .and_then(Value::as_str)
                    .filter(|name| !name.trim().is_empty())
                    .or_else(|| profile.get("username").and_then(Value::as_str))
                    .unwrap_or(text("shell.account"))
                    .to_owned();
                let id = profile["id"]
                    .as_str()
                    .filter(|id| crate::editor::local_id(id))
                    .ok_or(text("error.response"))?
                    .to_owned();
                Ok((tree, name, id))
            }
            .await;
            if !epoch.get_untracked().accepts(ticket) {
                return;
            }
            if session_token().ok().flatten().as_deref() != Some(token.as_str()) {
                invalidate();
                return;
            }
            match result {
                Ok((items, name, id)) => {
                    user_id.set(Some(id));
                    set_profile_name.set(Some(name));
                    set_menus.set(items);
                    set_active_token.set(Some(token));
                    set_status.set(text("session.ready"));
                }
                Err(message) => {
                    set_menus.set(Vec::new());
                    set_status.set(message);
                }
            }
            set_busy.set(false);
        });
    });
    provide_context(load);
    let logout = Callback::new(move |_| {
        if dirty.get_untracked()
            && web_sys::window().is_some_and(|window| {
                !window
                    .confirm_with_message(text("form.leave"))
                    .unwrap_or(false)
            })
        {
            return;
        }
        let result = (|| {
            let storage = web_sys::window()
                .ok_or(text("error.browser"))?
                .local_storage()
                .map_err(|_| text("error.storage"))?
                .ok_or(text("error.storage"))?;
            storage
                .remove_item("rustforge_token")
                .map_err(|_| text("error.storage"))?;
            storage
                .remove_item("trustpress_token")
                .map_err(|_| text("error.storage"))?;
            Ok::<(), &'static str>(())
        })();
        invalidate();
        set_status.set(result.err().unwrap_or(text("error.login")));
    });
    view! {
        <div class="rf-admin rf-admin-rust">
        <a class="skip-link" href="#rf-main" on:click=move |event| {event.prevent_default(); if let Some(main)=main_ref.get() {let _=main.focus();}}>{text("skip")}</a>
        <TopBar menus user_name=profile_name mobile_open on_mobile=Callback::new(move |_|set_mobile_open.update(|value|*value = !*value)) on_logout=logout close_epoch=menu_close/>
        <div class="layout" class:collapsed=move || collapsed.get() class:mobile-open=move || mobile_open.get() class:rf-compact=move ||compact.get()>
            <aside id="rfadminmenu" aria-label=text("navigation")>
                <nav id="rf-admin-navigation">{move || menus.get().into_iter().map(|node| view! {<MenuBranch node route collapsed close_epoch=menu_close/>}).collect_view()}</nav>
                <div class="rf-menu-footer"><button on:click=move |_| set_collapsed.update(|v|*v = !*v) aria-label=text("toggle_menu") aria-expanded=move || (!collapsed.get()).to_string() aria-controls="rf-admin-navigation"><span style:transform=move || if collapsed.get(){"rotate(180deg)"}else{"none"}><Icon kind=String::from("collapse")/></span><span>{text("shell.collapse")}</span></button></div>
            </aside>
            <main id="rf-main" tabindex="-1" node_ref=main_ref>
                <div class="rf-screen-meta"><button aria-expanded=move || options.get().to_string() aria-controls="rf-screen-options-wrap" on:click=move |_| set_options.update(|v|*v = !*v)>{text("options")}" ▾"</button><button aria-expanded=move || help.get().to_string() aria-controls="rf-contextual-help-wrap" on:click=move |_| set_help.update(|v|*v = !*v)>{text("help")}" ▾"</button></div>
                <Show when=move || options.get()><section id="rf-screen-options-wrap"><h2>{text("screens.display")}</h2>
                    <Show when=move || crate::screens::ListKind::from_route(&route.get()).is_some() fallback=move ||view!{<p>{text("options_pending")}</p>}>
                        <label><input type="checkbox" prop:checked=move ||compact.get() on:change=move |event|set_compact.set(event_target_checked(&event))/>{text("screens.compact")}</label><p class="rf-description">{text("screens.temporary")}</p>
                    </Show></section></Show>
                <Show when=move || help.get()><section id="rf-contextual-help-wrap"><h2>{text("help")}</h2><p>{move || if crate::screens::ListKind::from_route(&route.get()).is_some(){text("screens.help")}else{text("help_body")}}</p></section></Show>
                <p role="status" class:rf-sr-only=move || active_token.get().is_some()>{move || status.get()}</p>
                <Show when=move || active_token.get().is_none()><div class="rf-session-notice"><button class="rf-button" disabled=move || busy.get() on:click=move |_|load.run(())>{text("resume")}</button></div></Show>
                <Show when=move ||active_token.get().is_none()><crate::assistant_ui::Login on_ready=load/></Show>
                <Show when=move || route.get()=="dashboard">
                    <Dashboard token=active_token on_expired=Callback::new(move |_| invalidate())/>
                </Show>
                <Show when=move || route.get()!="dashboard">
                    <RoutedScreen route menus token=active_token on_expired=Callback::new(move |_| invalidate())/>
                </Show>
            </main>
        </div>
        </div>
    }
}
