//! Présentation du Studio : navigation et barre communes, sans droits métier locaux.
use crate::{messages::text, navigation::MenuNode};
use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub fn Icon(kind: String) -> impl IntoView {
    let path = match kind.as_str() {
        "rf_dashboard" => "M3 15a9 9 0 1 1 18 0 M12 12l4-4 M7 19h10",
        "rf_media" | "rf_galleries" => "M3 3h18v18H3z M3 16l6-6 4 4 3-3 5 5 M7 7h.01",
        "rf_users" | "profile" | "rf_work" => {
            "M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2 M9 3a4 4 0 1 0 0 8 4 4 0 0 0 0-8 M17 4a4 4 0 0 1 0 7 M22 21v-2a4 4 0 0 0-3-4"
        }
        "rf_settings" | "rf_tools" | "rf_modules" => "M4 7h16 M4 17h16 M8 3v8 M16 13v8",
        "rf_comments" => "M3 3h18v14H8l-5 4z",
        "rf_mail" => "M3 5h18v14H3z M3 5l9 7 9-7",
        "rf_events" | "rf_editorial" => "M3 5h18v16H3z M3 10h18 M7 2v6 M17 2v6",
        "rf_ai" | "rf_automations" => "M12 2l3 7 7 3-7 3-3 7-3-7-7-3 7-3z",
        "rf_commerce" => "M3 7h18l-2 14H5z M8 7V5a4 4 0 0 1 8 0v2",
        "rf_lms" => "M1 8l11-5 11 5-11 5z M5 10v7l7 4 7-4v-7",
        "rf_appearance" => {
            "M12 3a9 9 0 1 0 0 18h2a2 2 0 0 0 0-4h-1a2 2 0 0 1 0-4h3a5 5 0 0 0 0-10z"
        }
        "rf_analytics" => "M4 21V11 M12 21V3 M20 21v-7",
        "site" => "M2 11l10-9 10 9 M5 9v12h14V9 M9 21v-8h6v8",
        "create" => "M12 4v16 M4 12h16",
        "collapse" => "M15 5l-7 7 7 7",
        _ => "M5 2h9l5 5v15H5z M14 2v6h5 M8 12h8 M8 16h8",
    };
    view! {<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d=path/></svg>}
}

#[component]
pub fn MenuBranch(
    node: MenuNode,
    route: ReadSignal<String>,
    collapsed: ReadSignal<bool>,
    close_epoch: ReadSignal<u64>,
) -> impl IntoView {
    let root = NodeRef::<leptos::html::Div>::new();
    let hover = RwSignal::new(false);
    let top = RwSignal::new(32.0_f64);
    let active_node = node.clone();
    let active = Memo::new(move |_| active_node.contains_route(&route.get()));
    Effect::new(move |_| {
        route.get();
        close_epoch.get();
        hover.set(false);
    });
    let rows = node.children.iter().map(count_rows).sum::<usize>();
    let open = move || {
        if let Some(element) = root.get() {
            let height = web_sys::window()
                .and_then(|w| w.inner_height().ok())
                .and_then(|v| v.as_f64())
                .unwrap_or(800.0);
            let popup = (rows as f64 * 30.0 + 44.0).min((height - 64.0).max(40.0));
            top.set(
                element
                    .get_bounding_client_rect()
                    .top()
                    .min((height - popup - 12.0).max(32.0))
                    .max(32.0),
            );
        }
        hover.set(true);
    };
    let primary = node
        .route
        .clone()
        .or_else(|| node.children.iter().find_map(|child| child.route.clone()));
    let href = primary
        .map(|target| format!("#{target}"))
        .unwrap_or_else(|| "#dashboard".into());
    let hint = if node.planning {
        format!("{} · {}", node.label, text("planning"))
    } else {
        node.label.clone()
    };
    let inline = node.children.clone();
    let flyout = node.children.clone();
    let has_children = !inline.is_empty();
    view! {
        <div node_ref=root class="rf-menu-root" class:rf-current-parent=move || active.get() data-separator-before=node.separator_before.then_some("true")
            on:mouseenter=move |_| open() on:mouseleave=move |_| hover.set(false) on:focusin=move |_| open()
            on:focusout=move |event| {
                let inside = event.related_target().and_then(|target| target.dyn_into::<web_sys::Node>().ok())
                    .is_some_and(|target| root.get().is_some_and(|element| element.contains(Some(&target))));
                if !inside { hover.set(false); }
            }>
            <a href=href title=hint aria-current=move || active.get().then_some("page")>
                <span class="rf-menu-icon"><Icon kind=node.id/></span>
                <span class="rf-menu-title">{node.label.clone()}{node.planning.then(||view!{<span class="rf-menu-note">{text("planning")}</span>})}</span>
            </a>
            <Show when=move || has_children && active.get()>
                <div class="rf-menu-inline"><Submenu nodes=inline.clone() route depth=0/></div>
            </Show>
            <Show when=move || has_children && hover.get() && (collapsed.get() || !active.get())>
                <div class="rf-menu-flyout" style:top=move || format!("{}px",top.get())>
                    <div class="rf-flyout-title">{node.label.clone()}</div><Submenu nodes=flyout.clone() route depth=0/>
                </div>
            </Show>
        </div>
    }
}

fn count_rows(node: &MenuNode) -> usize {
    1 + node.children.iter().map(count_rows).sum::<usize>()
}

#[component]
fn Submenu(nodes: Vec<MenuNode>, route: ReadSignal<String>, depth: usize) -> impl IntoView {
    view! {<ul>{nodes.into_iter().map(|node| {
        let label = if node.planning {format!("{} · {}",node.label,text("planning"))} else {node.label};
        let link = node.route.map(|target| {
            let current = target.clone();
            view!{<a href=format!("#{target}") aria-current=move || (route.get()==current).then_some("page") style:padding-left=format!("{}px",12+depth*12)>{label.clone()}</a>}
        });
        view!{<li>{link}{(!node.children.is_empty()).then(|| view!{<Submenu nodes=node.children route depth=depth+1/>})}</li>}
    }).collect_view()}</ul>}.into_any()
}

#[component]
pub fn TopBar(
    menus: ReadSignal<Vec<MenuNode>>,
    user_name: ReadSignal<Option<String>>,
    mobile_open: ReadSignal<bool>,
    on_mobile: Callback<()>,
    on_logout: Callback<()>,
    close_epoch: ReadSignal<u64>,
) -> impl IntoView {
    let create = RwSignal::new(false);
    let account = RwSignal::new(false);
    let header = NodeRef::<leptos::html::Header>::new();
    Effect::new(move |_| {
        close_epoch.get();
        create.set(false);
        account.set(false);
    });
    let outside = window_event_listener(leptos::ev::mousedown, move |event| {
        let inside = event
            .target()
            .and_then(|t| t.dyn_into::<web_sys::Node>().ok())
            .is_some_and(|target| {
                header
                    .get()
                    .is_some_and(|element| element.contains(Some(&target)))
            });
        if !inside {
            create.set(false);
            account.set(false);
        }
    });
    on_cleanup(move || outside.remove());
    let choices = Memo::new(move |_| {
        [
            ("new:post", "shell.new_post"),
            ("new:page", "shell.new_page"),
            ("media", "shell.new_media"),
            ("users:new", "shell.new_user"),
        ]
        .into_iter()
        .filter(|(route, _)| menus.get().iter().any(|node| available(node, route)))
        .collect::<Vec<_>>()
    });
    view! {
        <header id="rfadminbar" node_ref=header aria-label=text("shell.topbar")>
            <div class="rf-bar-side">
                <button class="rf-mobile-toggle" aria-label=text("mobile_menu") aria-controls="rfadminmenu" aria-expanded=move || mobile_open.get().to_string() on:click=move |_| on_mobile.run(())>"☰"</button>
                <a href="#dashboard" aria-label=text("brand")><span class="rf-brand-mark rf-avatar">"RF"</span><span class="rf-brand-label">{text("brand")}</span></a>
                <a href="/" target="_blank" rel="noopener noreferrer"><Icon kind="site".into()/><span>{text("shell.site")}</span></a>
                <Show when=move || !choices.get().is_empty()>
                    <div class="rf-bar-dropdown" on:mouseleave=move |_| create.set(false)>
                        <button aria-expanded=move || create.get().to_string() aria-controls="rf-new-menu" on:click=move |_| {create.set(true);account.set(false);} on:mouseenter=move |_| create.set(true)><Icon kind="create".into()/>{text("shell.create")}</button>
                        <Show when=move || create.get()><div id="rf-new-menu" class="rf-bar-popover">{move || choices.get().into_iter().map(|(route,key)|view!{<a href=format!("#{route}")>{text(key)}</a>}).collect_view()}</div></Show>
                    </div>
                </Show>
            </div>
            <div class="rf-bar-side"><a href="/admin/">{text("current_studio")}</a>
                <Show when=move || user_name.get().is_some()>
                    <div class="rf-bar-dropdown" on:mouseleave=move |_| account.set(false)>
                        <button aria-expanded=move || account.get().to_string() aria-controls="rf-account-menu" on:click=move |_| {account.set(true);create.set(false);} on:mouseenter=move |_| account.set(true)>
                            <span class="rf-account-name">{text("shell.hello")}{move || user_name.get().unwrap_or_default()}</span>
                            <span class="rf-avatar">{move || user_name.get().unwrap_or_default().chars().take(2).collect::<String>().to_uppercase()}</span>
                        </button>
                        <Show when=move || account.get()><div id="rf-account-menu" class="rf-bar-popover"><a href="#profile">{text("shell.profile")}</a><a href="#learning">{text("learning.title")}</a><button on:click=move |_| on_logout.run(())>{text("shell.logout")}</button></div></Show>
                    </div>
                </Show>
            </div>
        </header>
    }
}

fn available(node: &MenuNode, route: &str) -> bool {
    (!node.planning && node.route.as_deref() == Some(route))
        || node.children.iter().any(|child| available(child, route))
}

#[component]
pub fn PlanningCard(node: MenuNode) -> impl IntoView {
    view! {
        <div class="rf-page-heading"><h1>{node.label}</h1></div>
        <p class="rf-session-notice">{text("shell.plan_notice")}</p>
        <section class="rf-panel rf-screen-panel"><h2>{text("shell.plan_features")}</h2>
            {if node.details.features.is_empty() {view!{<p>{text("shell.plan_children")}</p>}.into_any()}
            else {view!{<ul>{node.details.features.into_iter().map(|feature|view!{<li>{feature}</li>}).collect_view()}</ul>}.into_any()}}
            <p>{text("shell.plan_tasks")}{node.details.tasks.join(", ")}</p>
        </section>
    }
}

pub fn find_route(nodes: &[MenuNode], route: &str) -> Option<MenuNode> {
    nodes.iter().find_map(|node| {
        if node.route.as_deref() == Some(route) {
            Some(node.clone())
        } else {
            find_route(&node.children, route)
        }
    })
}
