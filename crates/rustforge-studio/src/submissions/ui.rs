use crate::{
    http::{self, Guard},
    messages::text,
};
use leptos::prelude::*;
use serde_json::{Value, json};

/// Dedicated menu entry; keeps the form builder separate from its inbox.
#[component]
pub fn Inbox(token: ReadSignal<Option<String>>, on_expired: Callback<()>) -> impl IntoView {
    let forms = RwSignal::new(Vec::<Value>::new());
    let form = RwSignal::new(None::<Value>);
    let busy = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let refresh = RwSignal::new(0_u64);
    let guard = Guard::new();
    Effect::new(move |_| {
        let current = token.get();
        refresh.get();
        let ticket = guard.begin();
        let guard = guard.clone();
        forms.set(vec![]);
        form.set(None);
        message.set(None);
        let Some(current) = current else { return };
        busy.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request("GET", "/api/v1/forms", &current, None).await;
            if !guard.accepts(ticket, &current) {
                return;
            }
            busy.set(false);
            match result {
                Ok(value) => {
                    let items = value.as_array().cloned().unwrap_or_default();
                    form.set(items.first().cloned());
                    forms.set(items);
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
    view! {<h1>{text("inbox.title")}</h1><p class="rf-description">{text("inbox.hint")}</p>
        {move ||message.get().map(|m|view!{<p class="rf-notice" role="alert">{m}</p>})}
        <label>{text("inbox.form")}<select disabled=move ||busy.get() prop:value=move ||form.get().and_then(|f|f["id"].as_str().map(str::to_owned)).unwrap_or_default() on:change=move |event|{let id=event_target_value(&event);form.set(forms.get_untracked().into_iter().find(|f|f["id"].as_str()==Some(&id)));}>{move ||forms.get().into_iter().map(|f|{let id=StoredValue::new(f["id"].as_str().unwrap_or_default().to_owned());view!{<option value=id.get_value() selected=move ||form.get().is_some_and(|f|f["id"].as_str()==Some(&id.get_value()))>{f["name"].as_str().unwrap_or_default().to_owned()}</option>}}).collect_view()}</select></label>
        <button class="rf-button" disabled=move ||busy.get() on:click=move |_|refresh.update(|n|*n=n.wrapping_add(1))>{text("dashboard.refresh")}</button>
        <Show when=move ||!busy.get()&&forms.get().is_empty()><p>{text("inbox.no_forms")}</p></Show>
        <Submissions form token on_expired/>
    }
}
#[component]
pub fn Submissions(
    form: RwSignal<Option<Value>>,
    token: ReadSignal<Option<String>>,
    on_expired: Callback<()>,
) -> impl IntoView {
    let rows = RwSignal::new(Vec::<Value>::new());
    let counts = RwSignal::new(Value::Null);
    let page = RwSignal::new(1_u64);
    let pages = RwSignal::new(1_u64);
    let total = RwSignal::new(0_u64);
    let input = RwSignal::new(String::new());
    let search = RwSignal::new(String::new());
    let read = RwSignal::new(String::new());
    let spam = RwSignal::new(String::new());
    let trash = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let busy = RwSignal::new(false);
    let exporting = RwSignal::new(false);
    let detail = RwSignal::new(None::<Value>);
    let pending = RwSignal::new(None::<Value>);
    let refresh = RwSignal::new(0_u64);
    let selection = RwSignal::new(std::collections::BTreeMap::<String, Value>::new());
    let batch_action = RwSignal::new("read".to_owned());
    let batch_pending = RwSignal::new(None::<(String, Vec<Value>)>);
    let batch_notice = RwSignal::new(None::<u64>);
    let guard = Guard::new();
    let batch_guard = guard.clone();
    let loader = guard.clone();
    let action_guard = guard.clone();
    let export_guard = guard.clone();
    let form_id = Memo::new(move |_| form.get().and_then(|f| f["id"].as_str().map(str::to_owned)));
    Effect::new(move |_| {
        form_id.get();
        input.set(String::new());
        search.set(String::new());
        read.set(String::new());
        spam.set(String::new());
        trash.set(false);
        page.set(1);
    });
    Effect::new(move |_| {
        form_id.get();
        token.get();
        page.get();
        search.get();
        read.get();
        spam.get();
        trash.get();
        detail.set(None);
        pending.set(None);
    });
    Effect::new(move |_| {
        let current = token.get();
        let id = form_id.get();
        let p = page.get();
        let q = search.get();
        let r = read.get();
        let s = spam.get();
        let t = trash.get();
        refresh.get();
        let guard = loader.clone();
        let ticket = guard.begin();
        rows.set(vec![]);
        error.set(None);
        exporting.set(false);
        counts.set(Value::Null);
        let (Some(id), Some(current)) = (id, current) else {
            busy.set(false);
            return;
        };
        busy.set(true);
        let path = format!(
            "/api/v1/forms/{id}/submissions?page={p}&per_page=20&trash={t}&q={}{}{}",
            super::encode_query(&q),
            if r.is_empty() {
                String::new()
            } else {
                format!("&is_read={r}")
            },
            if s.is_empty() {
                String::new()
            } else {
                format!("&is_spam={s}")
            }
        );
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request("GET", &path, &current, None).await;
            if !guard.accepts(ticket, &current) {
                return;
            }
            busy.set(false);
            match result {
                Ok(value) => {
                    rows.set(value["items"].as_array().cloned().unwrap_or_default());
                    counts.set(value["counts"].clone());
                    pages.set(value["total_pages"].as_u64().unwrap_or(1).max(1));
                    total.set(value["total"].as_u64().unwrap_or(0));
                    if p > pages.get_untracked() {
                        page.set(pages.get_untracked());
                    }
                }
                Err(f) => {
                    error.set(Some(f.message));
                    if f.unauthorized {
                        on_expired.run(());
                    }
                }
            }
        });
    });
    let change = Callback::new(move |(row, action): (Value, &'static str)| {
        if busy.get_untracked() || exporting.get_untracked() {
            return;
        }
        let Some(current) = token.get_untracked() else {
            return;
        };
        let Some(id) = row["id"]
            .as_str()
            .filter(|id| crate::editor::local_id(id))
            .map(str::to_owned)
        else {
            return;
        };
        let guard = action_guard.clone();
        let ticket = guard.begin();
        busy.set(true);
        error.set(None);
        wasm_bindgen_futures::spawn_local(async move {
            let result=http::request("PUT",&format!("/api/v1/forms/submissions/{id}"),&current,Some(&json!({"action":action,"expected_is_read":row["is_read"],"expected_deleted_at":row["deleted_at"]}))).await;
            if !guard.accepts(ticket, &current) {
                return;
            }
            busy.set(false);
            match result {
                Ok(updated) => {
                    pending.set(None);
                    if action == "trash" || action == "restore" {
                        detail.set(None);
                    } else if detail
                        .get_untracked()
                        .is_some_and(|d| d["id"] == updated["id"])
                    {
                        detail.set(Some(updated));
                    }
                    refresh.update(|n| *n = n.wrapping_add(1));
                }
                Err(f) => {
                    error.set(Some(f.message));
                    if f.unauthorized {
                        on_expired.run(());
                    }
                }
            }
        });
    });
    Effect::new(move |_| {
        form_id.get();
        token.get();
        page.get();
        search.get();
        read.get();
        spam.get();
        refresh.get();
        let t = trash.get();
        selection.set(Default::default());
        batch_pending.set(None);
        batch_action.set(if t { "restore" } else { "read" }.into());
    });
    let apply_batch = Callback::new(move |_: ()| {
        if busy.get_untracked() || exporting.get_untracked() {
            return;
        }
        let (Some(id), Some(current), Some((action, rows))) = (
            form_id.get_untracked(),
            token.get_untracked(),
            batch_pending.get_untracked(),
        ) else {
            return;
        };
        let body = json!({"action":action,"confirm_delete":action=="delete","items":rows.iter().map(|row|json!({"id":row["id"],"expected_is_read":row["is_read"],"expected_deleted_at":row["deleted_at"]})).collect::<Vec<_>>()});
        let guard = batch_guard.clone();
        let ticket = guard.begin();
        busy.set(true);
        error.set(None);
        batch_notice.set(None);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request(
                "POST",
                &format!("/api/v1/forms/{id}/submissions/batch"),
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
                    batch_notice.set(value["affected"].as_u64());
                    batch_pending.set(None);
                    detail.set(None);
                    page.set(1);
                    refresh.update(|n| *n = n.wrapping_add(1));
                }
                Err(f) => {
                    error.set(Some(f.message));
                    if f.unauthorized {
                        on_expired.run(());
                    }
                }
            }
        });
    });
    let export = move |_| {
        if busy.get_untracked() || exporting.get_untracked() {
            return;
        }
        let (Some(id), Some(current)) = (form_id.get_untracked(), token.get_untracked()) else {
            return;
        };
        let guard = export_guard.clone();
        let ticket = guard.begin();
        exporting.set(true);
        error.set(None);
        wasm_bindgen_futures::spawn_local(async move {
            let result = async {
                let mut export = crate::submission_csv::Export::default();
                for page in 1..=100 {
                    let value = http::request(
                        "GET",
                        &format!("/api/v1/forms/{id}/submissions?page={page}&per_page=100"),
                        &current,
                        None,
                    )
                    .await?;
                    if !guard.accepts(ticket, &current) {
                        return Ok(None);
                    }
                    if export
                        .append(&value, page)
                        .map_err(http::Failure::message)?
                    {
                        return export.finish().map(Some).map_err(http::Failure::message);
                    }
                }
                Err(http::Failure::message("csv.failed"))
            }
            .await;
            if !guard.accepts(ticket, &current) {
                return;
            }
            exporting.set(false);
            match result {
                Ok(Some(csv)) => error.set(Some(
                    text(if super::download::download_csv(&id, &csv).is_ok() {
                        "csv.exported"
                    } else {
                        "csv.download_failed"
                    })
                    .into(),
                )),
                Ok(None) => {}
                Err(f) => {
                    error.set(Some(f.message));
                    if f.unauthorized {
                        on_expired.run(());
                    }
                }
            }
        });
    };
    view! {<Show when=move ||form_id.get().is_some()><section class="rf-panel rf-screen-panel rf-inbox" aria-busy=move ||busy.get()||exporting.get()>
        <h2>{text("module.submissions")}</h2>{move ||form.get().and_then(|f|f["slug"].as_str().filter(|s|s.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'-'||b==b'_')).map(|slug|view!{<a class="rf-button" href=format!("/forms/{slug}") target="_blank" rel="noopener noreferrer">{text("module.public_form")}</a>}))}
        <nav class="rf-inbox-tabs" aria-label=text("inbox.folders")>
            <button class="rf-text-button" aria-pressed=move ||!trash.get() disabled=move ||busy.get()||exporting.get() on:click=move |_|{trash.set(false);page.set(1);}>{text("inbox.received")}" ("{move ||counts.get()["inbox"].as_i64().map(|n|n.to_string()).unwrap_or_else(||"—".into())} ")"</button>
            <span>{text("inbox.unread")}" : "{move ||counts.get()["unread"].as_i64().map(|n|n.to_string()).unwrap_or_else(||"—".into())}</span>
            <span>{text("inbox.spam")}" : "{move ||counts.get()["spam"].as_i64().map(|n|n.to_string()).unwrap_or_else(||"—".into())}</span>
            <button class="rf-text-button" aria-pressed=move ||trash.get() disabled=move ||busy.get()||exporting.get() on:click=move |_|{trash.set(true);page.set(1);}>{text("inbox.trash")}" ("{move ||counts.get()["trash"].as_i64().map(|n|n.to_string()).unwrap_or_else(||"—".into())}")"</button>
        </nav>
        <form class="rf-inbox-filters" on:submit=move |event|{event.prevent_default();if !busy.get_untracked()&&!exporting.get_untracked(){page.set(1);search.set(input.get_untracked());}}>
            <label>{text("inbox.search")}<input type="search" maxlength="200" prop:value=move ||input.get() disabled=move ||busy.get()||exporting.get() on:input=move |event|input.set(event_target_value(&event))/></label>
            <button class="rf-button" disabled=move ||busy.get()||exporting.get()>{text("screens.search")}</button>
            <label>{text("inbox.read_filter")}<select disabled=move ||busy.get()||exporting.get() prop:value=move ||read.get() on:change=move |event|{page.set(1);read.set(event_target_value(&event));}><option value="">{text("inbox.all")}</option><option value="false">{text("inbox.unread")}</option><option value="true">{text("inbox.read")}</option></select></label>
            <label>{text("inbox.spam_filter")}<select disabled=move ||busy.get()||exporting.get() prop:value=move ||spam.get() on:change=move |event|{page.set(1);spam.set(event_target_value(&event));}><option value="">{text("inbox.all")}</option><option value="false">{text("inbox.not_spam")}</option><option value="true">{text("inbox.spam")}</option></select></label>
            <button type="button" class="rf-button" disabled=move ||busy.get()||exporting.get() on:click=move |_|{input.set(String::new());search.set(String::new());read.set(String::new());spam.set(String::new());page.set(1);refresh.update(|n|*n=n.wrapping_add(1));}>{text("inbox.reset")}</button>
        </form>
        <button class="rf-button" disabled=move ||busy.get()||exporting.get() on:click=export.clone()>{move ||text(if exporting.get(){"csv.exporting"}else{"csv.export"})}</button>
        <p class="rf-description">{text("inbox.export_scope")}</p><p class="rf-description">{text("csv.hint")}</p>
        {move ||error.get().map(|m|view!{<p class="rf-notice" role="status">{m}</p>})}
        <Show when=move ||busy.get()><p role="status">{text("screens.loading")}</p></Show>
        <Show when=move ||!busy.get()&&rows.get().is_empty()><p>{text("inbox.empty")}</p></Show>
        <div class="rf-inbox-filters"><label>{text("bulk.action")}<select disabled=move ||busy.get()||exporting.get() prop:value=move ||batch_action.get() on:change=move |event|batch_action.set(event_target_value(&event))>{move ||{let actions=if trash.get(){vec![("restore","inbox.restore"),("delete","bulk.delete")]}else{vec![("read","inbox.mark_read"),("unread","inbox.mark_unread"),("trash","inbox.move_trash")]};actions.into_iter().map(|(value,label)|view!{<option value=value selected=move ||batch_action.get()==value>{text(label)}</option>}).collect_view()}}</select></label><button class="rf-button" disabled=move ||busy.get()||exporting.get()||selection.get().is_empty() on:click=move |_|batch_pending.set(Some((batch_action.get_untracked(),selection.get_untracked().into_values().collect())))>{text("bulk.apply")}" ("{move ||selection.get().len()}")"</button></div>
        {move ||batch_notice.get().map(|n|view!{<p class="rf-notice" role="status">{text("bulk.affected")}{n}</p>})}
        {move ||batch_pending.get().map(|(action,rows)|view!{<div class="rf-notice" role="group" aria-label=text("bulk.confirm")><p>{text(if action=="delete"{"bulk.delete_confirm"}else{"bulk.confirm"})}</p><p>{rows.len()}" · "{text(match action.as_str(){"read"=>"inbox.mark_read","unread"=>"inbox.mark_unread","trash"=>"inbox.move_trash","restore"=>"inbox.restore",_=>"bulk.delete"})}</p><p class="rf-description">{text("bulk.scope")}</p><button class="rf-button" disabled=move ||busy.get()||exporting.get() on:click=move |_|apply_batch.run(())>{text("module.confirm")}</button><button class="rf-button" disabled=move ||busy.get() on:click=move |_|batch_pending.set(None)>{text("form.cancel")}</button></div>})}
        <div class="rf-table-scroll"><table class="rf-list-table"><thead><tr><th><input type="checkbox" aria-label=text("bulk.select_page") disabled=move ||busy.get()||exporting.get()||rows.get().is_empty() prop:checked=move ||!rows.get().is_empty()&&rows.get().iter().all(|r|r["id"].as_str().is_some_and(|id|selection.get().contains_key(id))) on:change=move |event|{let checked=event_target_checked(&event);selection.set(if checked{rows.get_untracked().into_iter().filter_map(|row|row["id"].as_str().map(str::to_owned).map(|id|(id,row))).collect()}else{Default::default()});}/></th><th>{text("inbox.date")}</th><th>{text("inbox.preview")}</th><th>{text("screens.status")}</th><th>{text("module.action")}</th></tr></thead><tbody>
        {move ||rows.get().into_iter().map(|row|{let row=StoredValue::new(row);let unread=row.get_value()["is_read"]==false;let deleted=!row.get_value()["deleted_at"].is_null();view!{<tr class:rf-unread=unread>
            <td><input type="checkbox" aria-label=format!("{} {}",text("bulk.select"),row.get_value()["id"].as_str().unwrap_or_default()) disabled=move ||busy.get()||exporting.get() prop:checked=move ||selection.get().contains_key(row.get_value()["id"].as_str().unwrap_or_default()) on:change=move |event|{let checked=event_target_checked(&event);let value=row.get_value();if let Some(id)=value["id"].as_str().map(str::to_owned){selection.update(|items|{if checked{items.insert(id,value);}else{items.remove(&id);}});}}/></td><th scope="row">{row.get_value()["created_at"].as_str().unwrap_or_default().to_owned()}<small class="rf-job-id">{row.get_value()["id"].as_str().unwrap_or_default().to_owned()}</small></th>
            <td>{row.get_value()["data"].as_object().map(|data|data.iter().map(|(k,v)|format!("{k} : {}",v.as_str().map(str::to_owned).unwrap_or_else(||v.to_string()))).collect::<Vec<_>>().join(" · ").chars().take(160).collect::<String>()).unwrap_or_default()}</td>
            <td>{text(if unread{"inbox.unread"}else{"inbox.read"})}<br/>{text(if row.get_value()["is_spam"]==true{"inbox.spam"}else{"inbox.not_spam"})}</td>
            <td><button class="rf-button" disabled=move ||busy.get()||exporting.get() on:click=move |_|{detail.set(Some(row.get_value()));if unread&&!deleted {change.run((row.get_value(),"read"));}}>{text("inbox.inspect")}</button>
                <button class="rf-button" disabled=move ||busy.get()||exporting.get()||deleted on:click=move |_|change.run((row.get_value(),if unread{"read"}else{"unread"}))>{text(if unread{"inbox.mark_read"}else{"inbox.mark_unread"})}</button>
                <button class="rf-button" disabled=move ||busy.get()||exporting.get() on:click=move |_|{if deleted {change.run((row.get_value(),"restore"));}else{pending.set(Some(row.get_value()));}}>{text(if deleted{"inbox.restore"}else{"inbox.move_trash"})}</button>
            </td></tr>}}).collect_view()}
        </tbody></table></div>
        <nav class="rf-inbox-pages"><button class="rf-button" disabled=move ||busy.get()||exporting.get()||page.get()<=1 on:click=move |_|page.update(|p|*p-=1)>{text("screens.previous")}</button><span>{move ||format!(" {} / {} · {} ",page.get(),pages.get(),total.get())}</span><button class="rf-button" disabled={move ||busy.get()||exporting.get()||page.get()>=pages.get()} on:click=move |_|page.update(|p|*p+=1)>{text("screens.next")}</button></nav>
        {move ||pending.get().map(|row|{let row=StoredValue::new(row);view!{<div class="rf-notice" role="group" aria-label=text("inbox.trash_confirm")><p>{text("inbox.trash_confirm")}</p><p>{row.get_value()["id"].as_str().unwrap_or_default().to_owned()}</p><button class="rf-button" disabled=move ||busy.get()||exporting.get() on:click=move |_|change.run((row.get_value(),"trash"))>{text("module.confirm")}</button><button class="rf-button" disabled=move ||busy.get() on:click=move |_|pending.set(None)>{text("form.cancel")}</button></div>}})}
        {move ||detail.get().map(|row|view!{<section class="rf-panel rf-screen-panel"><h3>{text("inbox.detail")}</h3><p>{row["id"].as_str().unwrap_or_default().to_owned()}</p><dl class="rf-submission-detail">{row["data"].as_object().cloned().unwrap_or_default().into_iter().map(|(key,value)|view!{<dt>{key}</dt><dd>{value.as_str().map(str::to_owned).unwrap_or_else(||value.to_string())}</dd>}).collect_view()}</dl><button class="rf-button" on:click=move |_|detail.set(None)>{text("inbox.close")}</button></section>})}
        <p class="rf-description">{text("inbox.retention_hint")}</p><a href="#settings:forms">{text("retention.title")}</a>
    </section></Show>}
}
