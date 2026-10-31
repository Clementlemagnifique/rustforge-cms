//! Native diagnostics and audience views; all values come from the existing services.
use crate::{
    http::{self, Guard},
    messages::text,
};
use leptos::prelude::*;
use serde_json::{Value, json};
#[component]
pub fn Analytics(token: ReadSignal<Option<String>>, on_expired: Callback<()>) -> impl IntoView {
    let period = RwSignal::new("7d".to_owned());
    let data = RwSignal::new(None::<Value>);
    let error = RwSignal::new(None::<String>);
    let busy = RwSignal::new(false);
    let guard = Guard::new();
    Effect::new(move |_| {
        let token = token.get();
        let period = period.get();
        let ticket = guard.begin();
        data.set(None);
        error.set(None);
        let Some(token) = token else { return };
        let guard = guard.clone();
        busy.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request(
                "GET",
                &format!("/api/v1/analytics/summary?period={period}"),
                &token,
                None,
            )
            .await;
            if !guard.accepts(ticket, &token) {
                return;
            }
            busy.set(false);
            match result {
                Ok(value) => data.set(Some(value)),
                Err(failure) => {
                    error.set(Some(failure.message));
                    if failure.unauthorized {
                        on_expired.run(());
                    }
                }
            }
        });
    });
    view! {<h1>{text("module.analytics")}</h1><label for="rf-analytics-period">{text("insight.period")}</label><select id="rf-analytics-period" prop:value=move ||period.get() on:change=move |event|period.set(event_target_value(&event))><option value="24h">{text("insight.day")}</option><option value="7d">{text("insight.week")}</option><option value="30d">{text("insight.month")}</option></select>{move ||error.get().map(|message|view!{<p class="rf-notice" role="alert">{message}</p>})}<Show when=move ||busy.get()><p role="status">{text("screens.loading")}</p></Show>{move ||data.get().map(|data|view!{<p class="rf-notice">{data["notice"].as_str().unwrap_or_default().to_owned()}</p><div class="rf-screen-grid">{[("total_views","insight.views"),("unique_visitors","insight.visitors")].map(|(key,label)|view!{<section class="rf-panel rf-screen-panel"><h2>{text(label)}</h2><p>{data[key].as_i64().map(|n|n.to_string()).unwrap_or_else(||"—".into())}</p></section>})}</div><h2>{text("insight.pages")}</h2><div class="rf-table-scroll"><table class="rf-list-table"><thead><tr><th>{text("insight.path")}</th><th>{text("insight.views")}</th></tr></thead><tbody>{data["top_pages"].as_array().cloned().unwrap_or_default().into_iter().map(|row|view!{<tr><th scope="row">{row["path"].as_str().unwrap_or_default().to_owned()}</th><td>{row["views"].as_i64().map(|n|n.to_string()).unwrap_or_else(||"—".into())}</td></tr>}).collect_view()}</tbody></table></div>})}}
}
#[component]
pub fn Jobs(token: ReadSignal<Option<String>>, on_expired: Callback<()>) -> impl IntoView {
    let page = RwSignal::new(1_u64);
    let pages = RwSignal::new(1_u64);
    let filter = RwSignal::new(String::new());
    let counts = RwSignal::new(Value::Null);
    let mail_mode = RwSignal::new(String::new());
    let total = RwSignal::new(0_u64);
    let handled = RwSignal::new(None::<u64>);
    let data = RwSignal::new(Vec::<Value>::new());
    let busy = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let capture = RwSignal::new(None::<Value>);
    let refresh = RwSignal::new(0_u64);
    let pending = RwSignal::new(None::<(Value, &'static str)>);
    let guard = Guard::new();
    let loader = guard.clone();
    Effect::new(move |_| {
        let token = token.get();
        refresh.get();
        let page = page.get();
        let filter = filter.get();
        pending.set(None);
        capture.set(None);
        message.set(None);
        data.set(vec![]);
        counts.set(Value::Null);
        mail_mode.set(String::new());
        let ticket = loader.begin();
        let Some(token) = token else {
            data.set(vec![]);
            return;
        };
        let guard = loader.clone();
        busy.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request(
                "GET",
                &format!(
                    "/api/v1/jobs?page={page}&per_page=50{}",
                    if filter.is_empty() {
                        String::new()
                    } else {
                        format!("&state={filter}")
                    }
                ),
                &token,
                None,
            )
            .await;
            if !guard.accepts(ticket, &token) {
                return;
            }
            busy.set(false);
            match result {
                Ok(value) => {
                    data.set(value["items"].as_array().cloned().unwrap_or_default());
                    counts.set(value["counts"].clone());
                    mail_mode.set(value["mail_mode"].as_str().unwrap_or_default().to_owned());
                    total.set(value["total"].as_u64().unwrap_or(0));
                    pages.set(value["pages"].as_u64().unwrap_or(1));
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
    let action_guard = guard.clone();
    let apply = move |_| {
        if busy.get_untracked() {
            return;
        }
        let Some(token) = token.get_untracked() else {
            return;
        };
        let Some((row, action)) = pending.get_untracked() else {
            return;
        };
        let (path, body) = if action == "capture-batch" {
            ("/api/v1/jobs/capture".to_owned(), json!({"limit":10}))
        } else {
            let Some(id) = row["id"].as_str().filter(|id| crate::editor::local_id(id)) else {
                return;
            };
            (
                format!("/api/v1/jobs/{id}/{action}"),
                json!({"expected_updated_at":row["updated_at"]}),
            )
        };
        handled.set(None);
        let ticket = action_guard.begin();
        let guard = action_guard.clone();
        busy.set(true);
        message.set(None);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request("POST", &path, &token, Some(&body)).await;
            if !guard.accepts(ticket, &token) {
                return;
            }
            busy.set(false);
            match result {
                Ok(result) => {
                    if action == "capture-batch" {
                        handled.set(result["handled"].as_u64());
                    }
                    page.set(1);
                    pending.set(None);
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
    let preview = Callback::new(move |id: String| {
        if busy.get_untracked() {
            return;
        }
        let Some(token) = token.get_untracked() else {
            return;
        };
        let ticket = guard.begin();
        let guard = guard.clone();
        busy.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let result =
                http::request("GET", &format!("/api/v1/jobs/{id}/capture"), &token, None).await;
            if !guard.accepts(ticket, &token) {
                return;
            }
            busy.set(false);
            match result {
                Ok(value) => capture.set(Some(value)),
                Err(error) => {
                    message.set(Some(error.message));
                    if error.unauthorized {
                        on_expired.run(());
                    }
                }
            }
        });
    });
    view! {<h1>{text("jobs.title")}</h1><p>{text("jobs.transport")}" : "{move ||mail_mode.get()}</p><nav class="rf-job-counts" aria-label=text("jobs.counts")>{["queued","retry","running","failed","succeeded","cancelled","paused_restore"].into_iter().map(|state|view!{<button class="rf-button" aria-pressed=move ||filter.get()==state disabled=move ||busy.get() on:click=move |_|{page.set(1);filter.set(state.into());}>{job_state(state)}" ("{move ||counts.get()[state].as_u64().map(|n|n.to_string()).unwrap_or_else(||if counts.get().is_object(){"0".into()}else{"—".into()} )}")"</button>}).collect_view()}</nav><label>{text("jobs.filter")}<select disabled=move ||busy.get() prop:value=move ||filter.get() on:change=move |event|{page.set(1);filter.set(event_target_value(&event));}><option value="">{text("jobs.all")}</option>{["queued","retry","running","failed","succeeded","cancelled","paused_restore"].into_iter().map(|state|view!{<option value=state>{job_state(state)}</option>}).collect_view()}</select></label><button class="rf-button" disabled=move ||busy.get() on:click=move |_|refresh.update(|n|*n=n.wrapping_add(1))>{text("dashboard.refresh")}</button><button class="rf-button" disabled=move ||busy.get() on:click=move |_|pending.set(Some((Value::Null,"capture-batch")))>{text("jobs.batch")}</button><p class="rf-description">{text("jobs.batch_hint")}</p>{move ||handled.get().map(|n|view!{<p class="rf-notice rf-notice-success" role="status">{text("jobs.handled")}{n}</p>})}<p>{text("jobs.total")}" : "{move ||total.get()}</p><Show when=move ||!busy.get()&&data.get().is_empty()><p>{text("jobs.empty")}</p></Show>{move ||message.get().map(|message|view!{<p class="rf-notice" role="status">{message}</p>})}<Show when=move ||pending.get().is_some()><div class="rf-notice"><p>{move ||text(match pending.get().map(|(_,action)|action){Some("resume-restored")=>"jobs.resume_confirm",Some("capture-batch")=>"jobs.batch_confirm",_=>"jobs.confirm"})}</p><p>{move ||pending.get().filter(|(_,action)|*action!="capture-batch").map(|(row,action)|format!("{} · {} · {}",row["id"].as_str().unwrap_or_default(),text(match action {"cancel"=>"jobs.cancel","retry"=>"jobs.retry",_=>"jobs.resume"}),row["updated_at"].as_str().unwrap_or_default()))}</p><button class="rf-button" disabled=move ||busy.get() on:click=apply.clone()>{text("module.confirm")}</button><button class="rf-button" disabled=move ||busy.get() on:click=move |_|pending.set(None)>{text("form.cancel")}</button></div></Show><div class="rf-table-scroll"><table class="rf-list-table"><thead><tr><th>{text("jobs.kind")}</th><th>{text("screens.status")}</th><th>{text("jobs.attempts")}</th><th>{text("jobs.available")}</th><th>{text("jobs.error")}</th><th>{text("module.action")}</th></tr></thead><tbody>{move ||data.get().into_iter().map(|row|{let cancel=row.clone();let retry=row.clone();let resume=row.clone();let state=row["state"].as_str().unwrap_or_default();let can_cancel=["queued","retry","running","paused_restore"].contains(&state);let can_retry=state=="failed";let can_resume=state=="paused_restore";let can_capture=row["capture_available"].as_bool()==Some(true);let id=row["id"].as_str().unwrap_or_default().to_owned();view!{<tr><th scope="row">{row["kind"].as_str().unwrap_or_default().to_owned()}<small class="rf-job-id">{row["id"].as_str().unwrap_or_default().to_owned()}</small></th><td class="rf-job-state">{job_state(row["state"].as_str().unwrap_or_default())}</td><td>{row["attempts"].to_string()}" / "{row["max_attempts"].to_string()}</td><td>{row["available_at"].as_str().unwrap_or_default().to_owned()}</td><td>{row["error_code"].as_str().unwrap_or("—").to_owned()}</td><td><button class="rf-button" disabled=move ||busy.get()||!can_cancel on:click=move |_|pending.set(Some((cancel.clone(),"cancel")))>{text("jobs.cancel")}</button><button class="rf-button" disabled=move ||busy.get()||!can_retry on:click=move |_|pending.set(Some((retry.clone(),"retry")))>{text("jobs.retry")}</button><button class="rf-button" disabled=move ||busy.get()||!can_resume on:click=move |_|pending.set(Some((resume.clone(),"resume-restored")))>{text("jobs.resume")}</button><button class="rf-button" disabled=move ||busy.get()||!can_capture on:click=move |_|preview.run(id.clone())>{text("jobs.capture")}</button></td></tr>}}).collect_view()}</tbody></table></div><nav><button class="rf-button" disabled=move ||busy.get()||page.get()<=1 on:click=move |_|page.update(|p|*p-=1)>{text("screens.previous")}</button><span>{move ||format!(" {} / {} ", page.get(),pages.get())}</span><button class="rf-button" disabled={move ||busy.get()||page.get()>=pages.get()} on:click=move |_|page.update(|p|*p+=1)>{text("screens.next")}</button></nav>{move ||capture.get().map(|value|view!{<section class="rf-panel rf-screen-panel"><h2>{value["subject"].as_str().unwrap_or_default().to_owned()}</h2><p>{value["recipient"].as_str().unwrap_or_default().to_owned()}</p><p class="rf-capture-body">{value["text"].as_str().unwrap_or_default().to_owned()}</p><button class="rf-button" on:click=move |_|capture.set(None)>{text("form.cancel")}</button></section>})}}
}

fn job_state(state: &str) -> &'static str {
    match state {
        "queued" => text("jobs.queued"),
        "retry" => text("jobs.retry_state"),
        "running" => text("jobs.running"),
        "succeeded" => text("jobs.succeeded"),
        "failed" => text("jobs.failed"),
        "cancelled" => text("jobs.cancelled"),
        "paused_restore" => text("jobs.paused_restore"),
        _ => "—",
    }
}
