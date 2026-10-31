use crate::{
    http::{self, Guard},
    messages::text,
};
use leptos::prelude::*;
use serde_json::{Value, json};
#[component]
pub fn Learning(token: ReadSignal<Option<String>>, on_expired: Callback<()>) -> impl IntoView {
    let enrollments = RwSignal::new(Vec::<Value>::new());
    let courses = RwSignal::new(Vec::<Value>::new());
    let lessons = RwSignal::new(Vec::<Value>::new());
    let course_id = RwSignal::new(None::<String>);
    let busy = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let progress = RwSignal::new(0_u8);
    let guard = Guard::new();
    let loader = guard.clone();
    Effect::new(move |_| {
        let token = token.get();
        let ticket = loader.begin();
        courses.set(vec![]);
        lessons.set(vec![]);
        let Some(token) = token else { return };
        let guard = loader.clone();
        busy.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request("GET", "/api/v1/lms/my-courses", &token, None).await;
            let enrollment_result =
                http::request("GET", "/api/v1/lms/my-enrollments", &token, None).await;
            if !guard.accepts(ticket, &token) {
                return;
            }
            busy.set(false);
            if let Ok(value) = enrollment_result {
                enrollments.set(serde_json::from_value(value).unwrap_or_default());
            }
            match result {
                Ok(value) => courses.set(serde_json::from_value(value).unwrap_or_default()),
                Err(error) => {
                    message.set(Some(error.message));
                    if error.unauthorized {
                        on_expired.run(());
                    }
                }
            }
        });
    });
    let lesson_guard = guard.clone();
    let open = Callback::new(move |id: String| {
        if busy.get_untracked() {
            return;
        }
        let Some(token) = token.get_untracked() else {
            return;
        };
        let ticket = lesson_guard.begin();
        let guard = lesson_guard.clone();
        progress.set(
            enrollments
                .get_untracked()
                .iter()
                .find(|row| row["course_id"].as_str() == Some(&id))
                .and_then(|row| row["progress_pct"].as_u64())
                .unwrap_or(0)
                .min(100) as u8,
        );
        course_id.set(Some(id.clone()));
        lessons.set(vec![]);
        busy.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request(
                "GET",
                &format!("/api/v1/lms/courses/{id}/lessons"),
                &token,
                None,
            )
            .await;
            if !guard.accepts(ticket, &token) {
                return;
            }
            busy.set(false);
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
    let save = move |_| {
        if busy.get_untracked() {
            return;
        }
        let (Some(id), Some(token)) = (course_id.get_untracked(), token.get_untracked()) else {
            return;
        };
        let value = progress.get_untracked();
        let ticket = guard.begin();
        let guard = guard.clone();
        busy.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request(
                "POST",
                &format!("/api/v1/lms/courses/{id}/progress"),
                &token,
                Some(&json!({"progress_pct":value})),
            )
            .await;
            if !guard.accepts(ticket, &token) {
                return;
            }
            busy.set(false);
            match result {
                Ok(_) => {
                    enrollments.update(|rows| {
                        if let Some(row) = rows
                            .iter_mut()
                            .find(|row| row["course_id"].as_str() == Some(&id))
                        {
                            row["progress_pct"] = json!(value);
                        }
                    });
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
    view! {<h1>{text("learning.title")}</h1>{move ||message.get().map(|message|view!{<p class="rf-notice" role="status">{message}</p>})}<div class="rf-screen-grid">{move ||courses.get().into_iter().map(|course|{let id=course["id"].as_str().unwrap_or_default().to_owned();view!{<section class="rf-panel rf-screen-panel"><h2>{course["title"].as_str().unwrap_or_default().to_owned()}</h2><p>{course["description"].as_str().unwrap_or_default().to_owned()}</p><button class="rf-button" disabled=move ||busy.get() on:click=move |_|open.run(id.clone())>{text("learning.open")}</button></section>}}).collect_view()}</div><Show when=move ||courses.get().is_empty()&&!busy.get()><p class="rf-notice">{text("learning.empty")}</p></Show><section>{move ||lessons.get().into_iter().map(|lesson|view!{<article class="rf-panel rf-screen-panel rf-layout-options"><h2>{lesson["title"].as_str().unwrap_or_default().to_owned()}</h2><div inner_html=rustforge_document::markdown::render(lesson["content"].as_str().unwrap_or_default())/></article>}).collect_view()}</section><Show when=move ||course_id.get().is_some()><section class="rf-panel rf-screen-panel"><h2>{text("learning.progress")}</h2><p>{text("learning.progress_hint")}</p><label for="rf-learning-progress">{text("learning.percent")}</label><input id="rf-learning-progress" type="number" min="0" max="100" prop:value=move ||progress.get() on:input=move |event|{if let Ok(value)=event_target_value(&event).parse::<u8>()&& value<=100{progress.set(value);}}/><button class="rf-button primary" disabled=move ||busy.get() on:click=save.clone()>{text("form.save")}</button></section></Show>}
}
