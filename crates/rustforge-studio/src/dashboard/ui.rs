use super::{Activity, DashboardData, QuickDraft, client, editor_href, state_key};
use crate::{app::session_token, messages::text};
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
pub fn Dashboard(token: ReadSignal<Option<String>>, on_expired: Callback<()>) -> impl IntoView {
    let data = RwSignal::new(None::<DashboardData>);
    let loaded_for = RwSignal::new(None::<String>);
    let loading = RwSignal::new(false);
    let error = RwSignal::new(None::<client::ApiError>);
    let refresh = RwSignal::new(0_u64);
    let generation = Rc::new(Cell::new(0_u64));
    let alive = Arc::new(AtomicBool::new(true));
    let cleanup = alive.clone();
    on_cleanup(move || cleanup.store(false, Ordering::Relaxed));
    let visible = Memo::new(move |_| {
        (token.get().is_some() && token.get() == loaded_for.get())
            .then(|| data.get())
            .flatten()
    });
    Effect::new(move |_| {
        refresh.get();
        let current = token.get();
        let ticket = generation.get().wrapping_add(1);
        generation.set(ticket);
        error.set(None);
        if current != loaded_for.get_untracked() {
            data.set(None);
        }
        let Some(current) = current else {
            loading.set(false);
            return;
        };
        loading.set(true);
        let generation = generation.clone();
        let alive = alive.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = client::load(&current).await;
            if !alive.load(Ordering::Relaxed)
                || generation.get() != ticket
                || session_token().ok().flatten().as_deref() != Some(current.as_str())
            {
                return;
            }
            loading.set(false);
            match result {
                Ok(stats) => {
                    loaded_for.set(Some(current));
                    data.set(Some(stats));
                }
                Err(failure) => {
                    if matches!(
                        failure,
                        client::ApiError::Unauthorized | client::ApiError::Forbidden
                    ) {
                        data.set(None);
                    }
                    error.set(Some(failure));
                    if failure == client::ApiError::Unauthorized {
                        on_expired.run(());
                    }
                }
            }
        });
    });
    let preferences = expect_context::<RwSignal<super::layout::Preferences>>();
    let stale = Memo::new(move |_| error.get().is_some());
    let saved = Callback::new(move |_: ()| refresh.update(|v| *v = v.wrapping_add(1)));
    view! {
        <section class="dashboard" aria-label=text("dashboard.title")>
            <div class="dashboard-heading"><div><h1>{text("dashboard.title")}</h1><p>{text("dashboard.intro")}</p></div>
                <button class="rf-button" disabled=move || loading.get() || token.get().is_none()
                    on:click=move |_| refresh.update(|v| *v = v.wrapping_add(1))>{text("dashboard.refresh")}</button>
            </div>
            <Show when=move || token.get().is_none()><p class="dashboard-notice">{text("dashboard.resume")}</p></Show>
            <Show when=move || loading.get()><p role="status">{text("dashboard.loading")}</p></Show>
            {move || error.get().map(|failure| view!{<p class="dashboard-notice error" role="alert">{text(failure.key())}</p>})}
            <super::layout::LayoutControls token on_expired/>
            <Show when=move ||visible.get().is_some()><div class="dashboard-grid"><For each=move ||preferences.get().dashboard.order key=|id|id.clone() children=move |id|{
                let hidden_id=id.clone();let is_draft=id=="draft";
                view!{<div class="rf-widget" style:display=move ||if preferences.get().dashboard.hidden.contains(&hidden_id){"none"}else{""}>
                    {if is_draft{view!{<QuickDraftForm token on_expired on_saved=saved/>}.into_any()}else{view!{<DataWidget id data=visible stale/>}.into_any()}}
                </div>}
            }/></div></Show>
        </section>
    }
}

#[component]
fn DataWidget(id: String, data: Memo<Option<DashboardData>>, stale: Memo<bool>) -> impl IntoView {
    move || {
        data.get().map(|data|{let activity=data.activity;match id.as_str(){
 "overview"=>view!{            <section class="card dashboard-card"><h2>{text("dashboard.overview")}</h2>
                <p class="dashboard-muted">{text("dashboard.scope")}</p>
                <div class="dashboard-counts">{data.content_types.into_iter().filter(|kind| matches!(kind.slug.as_str(), "post" | "page")).map(|kind| view!{
                    <div class="dashboard-count-group"><h3>{kind.name}</h3><dl>
                        <div><dt>{text("dashboard.published")}</dt><dd>{kind.published}</dd></div>
                        <div><dt>{text("dashboard.drafts")}</dt><dd>{kind.drafts}</dd></div>
                        <div><dt>{text("dashboard.scheduled")}</dt><dd>{kind.scheduled}</dd></div>
                    </dl></div>
                }).collect_view()}</div>
                <p>{data.media}" "{text("dashboard.media")}<span class="dashboard-muted">" · "{format!("{:.1}", data.media_bytes as f64 / 1_048_576.0)}" "{text("dashboard.mib")}</span></p>
                <div class="dashboard-links"><a href="#entries:post">{text("dashboard.posts")}</a><a href="#entries:page">{text("dashboard.pages")}</a></div>
            </section>}.into_any(),
 "activity"=>view!{            <section class="card dashboard-card"><h2>{text("dashboard.activity")}</h2>
                <p class="dashboard-muted">{text("dashboard.activity_hint")}</p>
                {if activity.is_empty() { view!{<p>{text("dashboard.empty")}</p>}.into_any() }
                else { view!{<ul class="dashboard-activity">{activity.into_iter().map(|entry| {
                    let link = editor_href(&entry);
                    let label = text(state_key(&entry.state));
                    view!{<li><span>{if let Some(href)=link {view!{<a href=href>{entry.title}</a>}.into_any()} else {view!{<span>{entry.title}</span>}.into_any()}}</span><span class="dashboard-badge">{label}</span></li>}
                }).collect_view()}</ul>}.into_any() }}
            </section>}.into_any(),
 _=>view!{            <section class="card dashboard-card dashboard-health"><h2>{text("dashboard.health")}</h2>
                <dl><div><dt>{text("dashboard.connection")}</dt><dd>{if stale.get() {text("dashboard.previous_data")} else {text("dashboard.connected")}}</dd></div>
                    <div><dt>{text("dashboard.version")}</dt><dd>{data.server_version}</dd></div></dl>
                <p class="dashboard-muted">{text("dashboard.health_hint")}</p>
                <a href="/admin/#tools:health">{text("dashboard.health_link")}</a>
            </section>}.into_any(),
 }})
    }
}

#[component]
fn QuickDraftForm(
    token: ReadSignal<Option<String>>,
    on_expired: Callback<()>,
    on_saved: Callback<()>,
) -> impl IntoView {
    let title = RwSignal::new(String::new());
    let body = RwSignal::new(String::new());
    let dirty = crate::http::dirty_scope();
    Effect::new(move |_| dirty.set(!title.get().is_empty() || !body.get().is_empty()));
    let saving = RwSignal::new(false);
    let message = RwSignal::new(None::<&'static str>);
    let failed = RwSignal::new(false);
    let created = RwSignal::new(None::<Activity>);
    let alive = Arc::new(AtomicBool::new(true));
    let cleanup = alive.clone();
    on_cleanup(move || cleanup.store(false, Ordering::Relaxed));
    let submit = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        if saving.get_untracked() {
            return;
        }
        let Some(current) = token.get_untracked() else {
            return;
        };
        let input = match QuickDraft::new(&title.get_untracked(), &body.get_untracked()) {
            Ok(input) => input,
            Err(key) => {
                message.set(Some(key));
                failed.set(true);
                return;
            }
        };
        saving.set(true);
        message.set(None);
        created.set(None);
        let alive = alive.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = client::save(&current, &input).await;
            if !alive.load(Ordering::Relaxed)
                || session_token().ok().flatten().as_deref() != Some(current.as_str())
            {
                return;
            }
            saving.set(false);
            match result {
                Ok(entry) => {
                    title.set(String::new());
                    body.set(String::new());
                    created.set(Some(entry));
                    failed.set(false);
                    message.set(Some("dashboard.saved"));
                    on_saved.run(());
                }
                Err(error) => {
                    failed.set(true);
                    message.set(Some(error.key()));
                    if error == client::ApiError::Unauthorized {
                        on_expired.run(());
                    }
                }
            }
        });
    };
    view! {
        <section class="card dashboard-card dashboard-draft"><h2>{text("dashboard.quick_draft")}</h2>
            <form on:submit=submit><fieldset disabled=move || saving.get()>
                <label for="rf-rust-draft-title">{text("dashboard.title_field")}</label>
                <input id="rf-rust-draft-title" name="title" required maxlength="300" prop:value=move || title.get()
                    on:input=move |event| title.set(event_target_value(&event))/>
                <label for="rf-rust-draft-body">{text("dashboard.body_field")}</label>
                <textarea id="rf-rust-draft-body" name="body" maxlength="20000" prop:value=move || body.get()
                    on:input=move |event| body.set(event_target_value(&event))></textarea>
                <button class="rf-button primary" type="submit" disabled=move || saving.get() || title.get().trim().is_empty()>
                    {move || if saving.get() {text("dashboard.saving")} else {text("dashboard.save")}}
                </button>
            </fieldset></form>
            {move || message.get().map(|key|view!{<p class:error=move || failed.get() role=move || if failed.get(){"alert"}else{"status"}>{text(key)}</p>})}
            {move || created.get().and_then(|entry| editor_href(&entry)).map(|href|view!{<a href=href>{text("dashboard.open_draft")}</a>})}
        </section>
    }
}
