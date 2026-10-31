use super::{ListKind, ListState, Snapshot, decode};
use crate::{app::session_token, messages::text, navigation::MenuNode, shell};
use leptos::prelude::*;
use std::{
    cell::Cell,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

#[component]
pub fn ListScreen(
    kind: ListKind,
    node: MenuNode,
    token: ReadSignal<Option<String>>,
    on_expired: Callback<()>,
) -> impl IntoView {
    let has_draft = expect_context::<RwSignal<bool>>();
    let states = expect_context::<RwSignal<std::collections::BTreeMap<String, ListState>>>();
    let route_key = node.route.clone().unwrap_or_default();
    let initial = states.with_untracked(|all| all.get(&route_key).cloned().unwrap_or_default());
    let search = RwSignal::new(initial.search.clone());
    let query = RwSignal::new(initial.search);
    let filter = RwSignal::new(initial.filter);
    let order = RwSignal::new(initial.order);
    let page = RwSignal::new(initial.page);
    let columns = RwSignal::new(initial.columns);
    let options = RwSignal::new(false);
    Effect::new(move |_| {
        let state = ListState {
            search: query.get(),
            filter: filter.get(),
            order: order.get(),
            page: page.get(),
            columns: columns.get(),
        };
        states.update(|all| {
            all.insert(route_key.clone(), state);
        });
    });
    let refresh = RwSignal::new(0_u64);
    let loading = RwSignal::new(false);
    let data = RwSignal::new(None::<Snapshot>);
    let error = RwSignal::new(None::<&'static str>);
    let generation = Rc::new(Cell::new(0_u64));
    let alive = Arc::new(AtomicBool::new(true));
    let cleanup = alive.clone();
    on_cleanup(move || cleanup.store(false, Ordering::Relaxed));
    Effect::new(move |_| {
        let current = token.get();
        let url = kind.filtered_endpoint(&ListState {
            search: query.get(),
            filter: filter.get(),
            order: order.get(),
            page: page.get(),
            columns: [true; 4],
        });
        refresh.get();
        let ticket = generation.get().wrapping_add(1);
        generation.set(ticket);
        data.set(None);
        error.set(None);
        loading.set(false);
        let Some(current) = current else { return };
        loading.set(true);
        let generation = generation.clone();
        let alive = alive.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = async {
                let response = gloo_net::http::Request::get(&url)
                    .header("Authorization", &format!("Bearer {current}"))
                    .send()
                    .await
                    .map_err(|_| "screens.network")?;
                match response.status() {
                    401 => return Err("dashboard.unauthorized"),
                    403 => return Err("screens.forbidden"),
                    200 => {}
                    _ => return Err("screens.response"),
                }
                let body = response.text().await.map_err(|_| "screens.response")?;
                decode(kind, &body, (js_sys::Date::now() / 1000.) as i64)
                    .map_err(|_| "screens.response")
            }
            .await;
            if !alive.load(Ordering::Relaxed)
                || generation.get() != ticket
                || session_token().ok().flatten().as_deref() != Some(current.as_str())
            {
                return;
            }
            loading.set(false);
            match result {
                Ok(snapshot) => data.set(Some(snapshot)),
                Err(key) => {
                    error.set(Some(key));
                    if key == "dashboard.unauthorized" {
                        on_expired.run(());
                    }
                }
            }
        });
    });
    let title = node.label;
    let route = node.route.unwrap_or_default();
    view! {
        <section class="rf-screen" aria-label=title.clone()>
            <div class="rf-page-heading"><h1>{title.clone()}</h1><a class="rf-button" href=format!("/admin/#{route}")>{text("screens.manage")}</a></div>
            <Show when=move ||kind==ListKind::Pages><crate::relations::ui::PageTree token/></Show>
            <p class="rf-description">{text("screens.readonly")}</p><button class="rf-button" aria-expanded=move ||options.get().to_string() on:click=move |_|options.update(|open|*open = !*open)>{text("list.columns")}</button>
            <Show when=move ||options.get()><fieldset class="rf-column-options"><legend>{text("list.columns")}</legend>{kind.headings().into_iter().enumerate().skip(1).map(|(index,key)|view!{<label><input type="checkbox" prop:checked=move ||columns.get()[index] on:change=move |event|columns.update(|all|all[index]=event_target_checked(&event))/>{text(key)}</label>}).collect_view()}</fieldset></Show>
            <Show when=move ||kind==ListKind::Media><crate::media_ui::Upload token on_expired on_uploaded=Callback::new(move |_|refresh.update(|n|*n=n.wrapping_add(1)))/></Show>
            <form class="rf-list-toolbar" on:submit=move |event|{event.prevent_default();if has_draft.get_untracked(){return;}query.set(search.get_untracked().trim().to_owned());page.set(1);refresh.update(|n|*n=n.wrapping_add(1));}>
                <div class="rf-list-filters"><label>{text("list.filter")}<select disabled=move ||has_draft.get() prop:value=move ||filter.get() on:change=move |event|{filter.set(event_target_value(&event));page.set(1);}>
                <option value="">{text("list.all")}</option>{match kind{ListKind::Posts|ListKind::Pages=>vec![("draft","dashboard.draft"),("published","list.published_scheduled"),("archived","dashboard.archived"),("trash","dashboard.trash")],ListKind::Users=>vec![("admin","screens.admin"),("editor","screens.editor"),("author","screens.author"),("subscriber","screens.subscriber")],ListKind::Media=>vec![("image","form.image"),("video","form.video"),("audio","form.audio"),("application","list.documents")]}.into_iter().map(|(value,key)|view!{<option value=value>{text(key)}</option>}).collect_view()}</select></label>
                <Show when=move ||matches!(kind,ListKind::Posts|ListKind::Pages)><label>{text("list.sort")}<select disabled=move ||has_draft.get() prop:value=move ||order.get() on:change=move |event|{order.set(event_target_value(&event));page.set(1);}><option value="updated">{text("screens.date")}</option><option value="title">{text("screens.title")}</option><option value="created">{text("list.created")}</option><option value="published">{text("list.publication")}</option></select></label></Show></div>
                <div class="rf-search-field"><label for="rf-list-search">{text("screens.search")}</label><input id="rf-list-search" type="search" maxlength="200" prop:value=move ||search.get() on:input=move |event|search.set(event_target_value(&event))/><button class="rf-button" disabled=move ||token.get().is_none()||loading.get()||has_draft.get()>{text("screens.search")}</button></div>
                <button type="button" class="rf-button" disabled=move ||loading.get()||token.get().is_none()||has_draft.get() on:click=move |_|refresh.update(|n|*n=n.wrapping_add(1))>{text("dashboard.refresh")}</button>
            </form>
            <Show when=move ||token.get().is_none()><p class="rf-notice">{text("screens.resume")}</p></Show>
            <Show when=move ||loading.get()><p class="rf-notice" role="status">{text("screens.loading")}</p></Show>
            {move ||error.get().map(|key|view!{<p class="rf-notice rf-notice-error" role="alert">{text(key)}</p>})}
            {move ||data.get().map(|snapshot|{
                let has_previous=snapshot.page>1;let has_next=snapshot.page<snapshot.pages;
                view!{
                    <div class="rf-table-scroll" tabindex="0" role="region" aria-label=text("screens.results")><table class="rf-list-table"><caption class="rf-sr-only">{text("screens.results")}</caption>
                        <thead><tr>{kind.headings().into_iter().enumerate().map(|(index,key)|view!{<th scope="col" style:display=move ||if columns.get()[index]{""}else{"none"}>{text(key)}</th>}).collect_view()}</tr></thead>
                        <tbody>{if snapshot.rows.is_empty(){view!{<tr><td colspan="4" class="rf-empty">{text("screens.empty")}</td></tr>}.into_any()}else{snapshot.rows.into_iter().map(|row|{let [first,second,third,fourth]=row.cells;view!{<tr><th scope="row">{if let Some(href)=row.href {view!{<a href=href>{first}</a>}.into_any()}else{view!{<span>{first}</span>}.into_any()}}{row.entry.map(|entry|view!{<EntryActions entry token on_expired on_changed=Callback::new(move |_|refresh.update(|n|*n=n.wrapping_add(1)))/>})}</th><td style:display=move ||if columns.get()[1]{""}else{"none"}>{second}</td><td style:display=move ||if columns.get()[2]{""}else{"none"}>{third}</td><td style:display=move ||if columns.get()[3]{""}else{"none"}>{fourth}</td></tr>}}).collect_view().into_any()}}</tbody>
                    </table></div><p class="rf-table-hint">{text("screens.scroll")}</p>
                    <div class="rf-table-footer"><p role="status">{snapshot.total}" "{text("screens.items")}" · "{text("screens.page")}" "{snapshot.page}" / "{snapshot.pages.max(1)}</p><nav aria-label=text("screens.pagination")><button class="rf-button" disabled=move ||!has_previous||has_draft.get() on:click=move |_|page.update(|n|*n=n.saturating_sub(1).max(1))>{text("screens.previous")}</button><button class="rf-button" disabled=move ||!has_next||has_draft.get() on:click=move |_|page.update(|n|*n=n.saturating_add(1))>{text("screens.next")}</button></nav></div>
                }
            })}
        </section>
    }
}

/// Consistent page layout for screens not ported yet; no business controls are simulated.
#[component]
pub fn ScreenOverview(node: MenuNode) -> impl IntoView {
    let children = node.children.clone();
    let route = node.route.clone().unwrap_or_default();
    view! {
        <section class="rf-screen"><div class="rf-page-heading"><h1>{node.label}</h1><a class="rf-button" href=format!("/admin/#{route}")>{text("open_current")}</a></div>
        <p class="rf-notice">{text("shell.transition")}</p>
        <div class="rf-screen-grid"><section class="rf-panel rf-screen-panel"><h2>{text("screens.sections")}</h2>
        {if children.is_empty(){view!{<p>{text("screens.no_sections")}</p>}.into_any()}else{view!{<ul class="rf-section-links">{children.into_iter().map(|child|{let route=child.route.clone();let label=child.label.clone();view!{<li>{if let Some(route)=route{view!{<a href=format!("#{route}")>{label}</a>}.into_any()}else{view!{<span>{label}</span>}.into_any()}}<span class="rf-description">{if child.planning{text("planning")}else{text("screens.transition_short")}}</span></li>}}).collect_view()}</ul>}.into_any()}}
        </section><section class="rf-panel rf-screen-panel"><h2>{text("shell.plan_features")}</h2>{if node.details.features.is_empty(){view!{<p>{text("shell.plan_children")}</p>}.into_any()}else{view!{<ul>{node.details.features.into_iter().map(|feature|view!{<li>{feature}</li>}).collect_view()}</ul>}.into_any()}}<p class="rf-description">{text("screens.scope_hint")}</p></section></div>
        </section>
    }
}

#[component]
pub fn RoutedScreen(
    route: ReadSignal<String>,
    menus: ReadSignal<Vec<MenuNode>>,
    token: ReadSignal<Option<String>>,
    on_expired: Callback<()>,
) -> impl IntoView {
    move || {
        let route = route.get();
        let nodes = menus.get();
        untrack(move || {
            if route == "learning" && token.get().is_some() {
                view! {<crate::learning_ui::Learning token on_expired/>}.into_any()
            } else if let Some(id) = route
                .strip_prefix("user:")
                .filter(|id| crate::editor::local_id(id))
                .filter(|_| shell::find_route(&nodes, "users").is_some_and(|node| !node.planning))
            {
                view!{<crate::forms::ui::SettingsForm kind=crate::forms::FormKind::User target=id.to_owned() token on_expired/>}.into_any()
            } else if let Some(id) = route
                .strip_prefix("media:")
                .filter(|id| crate::editor::local_id(id))
                .filter(|_| shell::find_route(&nodes, "media").is_some_and(|node| !node.planning))
            {
                view! {<crate::media_ui::Detail id=id.to_owned() token on_expired/>}.into_any()
            } else if let Some(location) =
                crate::editor::Location::parse(&route).filter(|location| {
                    shell::find_route(&nodes, &format!("entries:{}", location.content_type))
                        .is_some_and(|node| !node.planning)
                })
            {
                view! {<crate::editor::ui::Editor location token on_expired/>}.into_any()
            } else if let Some(node) = shell::find_route(&nodes, &route) {
                if node.planning {
                    view! {<shell::PlanningCard node/>}.into_any()
                } else if matches!(
                    route.as_str(),
                    "taxonomy:category" | "taxonomy:tag" | "taxonomy:manage"
                ) {
                    view! {<crate::taxonomies_ui::Taxonomies initial=if route=="taxonomy:manage"{"category".to_owned()}else{route.trim_start_matches("taxonomy:").to_owned()} definitions=route=="taxonomy:manage" token on_expired/>}.into_any()
                } else if route == "forms:submissions" {
                    view!{<crate::submissions::ui::Inbox token on_expired/><crate::block_catalog::ui::FutureOptions node/>}.into_any()
                } else if route == "comments" {
                    view! {<crate::comments_ui::Comments token on_expired/>}.into_any()
                } else if route == "ecommerce:orders" {
                    view! {<crate::orders_ui::Orders token on_expired/>}.into_any()
                } else if route == "ai-studio" {
                    view! {<crate::assistant_ui::Assistant token on_expired/>}.into_any()
                } else if route == "analytics" {
                    view! {<crate::diagnostics_ui::Analytics token on_expired/>}.into_any()
                } else if route == "jobs" {
                    view! {<crate::diagnostics_ui::Jobs token on_expired/><crate::block_catalog::ui::FutureOptions node/>}.into_any()
                } else if route == "modules" {
                    view! {<crate::modules::center::Center token on_expired/>}.into_any()
                } else if let Some(kind) = crate::modules::Resource::from_route(&route) {
                    view! {<crate::modules::ui::ResourceScreen kind token on_expired creating=route.ends_with(":new")/><crate::block_catalog::ui::FutureOptions node/><crate::block_catalog::ui::Catalog/>}.into_any()
                } else if matches!(route.as_str(), "appearance" | "appearance:themes") {
                    view! {<crate::appearance_ui::Themes token on_expired/>}.into_any()
                } else if matches!(route.as_str(), "appearance:menus" | "menus") {
                    view! {<crate::appearance_ui::PublicMenus token on_expired/>}.into_any()
                } else if let Some(kind) = crate::forms::FormKind::from_route(&route) {
                    if kind == crate::forms::FormKind::FormRetention {
                        view!{<crate::forms::ui::SettingsForm kind token on_expired/><crate::block_catalog::ui::FutureOptions node/>}.into_any()
                    } else {
                        view! {<crate::forms::ui::SettingsForm kind token on_expired/>}.into_any()
                    }
                } else if let Some(kind) = ListKind::from_route(&route) {
                    view! {<ListScreen kind node token on_expired/>}.into_any()
                } else {
                    view! {<ScreenOverview node/>}.into_any()
                }
            } else {
                view!{<h1>{text("title")}</h1><p class="rf-notice">{if token.get().is_none(){text("screens.resume")}else{text("screens.unavailable")}}</p>}.into_any()
            }
        })
    }
}

#[component]
fn EntryActions(
    entry: super::EntryAction,
    token: ReadSignal<Option<String>>,
    on_expired: Callback<()>,
    on_changed: Callback<()>,
) -> impl IntoView {
    let confirm = RwSignal::new(false);
    let editing = RwSignal::new(false);
    let original_title = StoredValue::new(entry.title.clone());
    let title = RwSignal::new(entry.title);
    let all_dirty = expect_context::<RwSignal<bool>>();
    let dirty = crate::http::dirty_scope();
    let message = RwSignal::new(None::<String>);
    let busy = RwSignal::new(false);
    let guard = crate::http::Guard::new();
    let id = StoredValue::new(entry.id);
    let version = StoredValue::new(entry.updated_at);
    let trashed = entry.status == "trash";
    let apply = Callback::new(move |quick: bool| {
        let Some(current) = token.get_untracked() else {
            return;
        };
        if busy.get_untracked() || (all_dirty.get_untracked() && !dirty.get_untracked()) {
            return;
        }
        let id = id.get_value();
        if !crate::editor::local_id(&id) {
            return;
        }
        let guard = guard.clone();
        let ticket = guard.begin();
        let body = if quick {
            serde_json::json!({"title":title.get_untracked(),"expected_updated_at":version.get_value()})
        } else {
            serde_json::json!({"status":if trashed{"draft"}else{"trash"},"expected_updated_at":version.get_value()})
        };
        busy.set(true);
        confirm.set(false);
        wasm_bindgen_futures::spawn_local(async move {
            let result = crate::http::request(
                "PUT",
                &format!("/api/v1/entries/{id}"),
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
                    dirty.set(false);
                    on_changed.run(());
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
    view! {<div class="rf-row-actions"><button class="rf-text-button" disabled=move ||busy.get()||all_dirty.get() on:click=move |_|confirm.set(true)>{if trashed{text("list.restore")}else{text("list.trash")}}</button><button class="rf-text-button" disabled=move ||busy.get()||all_dirty.get() on:click=move |_|{if dirty.get_untracked(){return;}editing.update(|value|*value = !*value);}>{text("list.quick_edit")}</button><Show when=move ||confirm.get()><span>{text("list.action_confirm")}</span><button class="rf-button" disabled=move ||busy.get() on:click=move |_|apply.run(false)>{text("module.confirm")}</button><button class="rf-button" on:click=move |_|confirm.set(false)>{text("form.cancel")}</button></Show><Show when=move ||editing.get()><label>{text("screens.title")}<input disabled=move ||busy.get()||(all_dirty.get()&&!dirty.get()) maxlength="300" prop:value=move ||title.get() on:input=move |event|{title.set(event_target_value(&event));dirty.set(true);}/></label><button class="rf-button" disabled=move ||busy.get()||title.get().trim().is_empty() on:click=move |_|apply.run(true)>{text("form.save")}</button><button class="rf-button" on:click=move |_|{title.set(original_title.get_value());editing.set(false);dirty.set(false);}>{text("form.cancel")}</button></Show>{move ||message.get().map(|message|view!{<p role="alert">{message}</p>})}</div>}
}
