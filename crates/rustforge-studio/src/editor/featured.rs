//! Featured image is a media reference, independent from document blocks.
use crate::{
    http::{self, Guard},
    messages::text,
};
use leptos::prelude::*;
use serde_json::{Value, json};
#[component]
pub fn FeaturedImage(
    data: RwSignal<Value>,
    history: RwSignal<super::History>,
    dirty: RwSignal<bool>,
    busy: RwSignal<bool>,
    token: ReadSignal<Option<String>>,
    on_expired: Callback<()>,
) -> impl IntoView {
    let metadata = RwSignal::new(None::<Value>);
    let message = RwSignal::new(None::<String>);
    let loading = RwSignal::new(false);
    let guard = Guard::new();
    let image_id = Memo::new(move |_| data.get()["featured_media_id"].as_str().map(str::to_owned));
    Effect::new(move |_| {
        let id = image_id.get();
        let current = token.get();
        let ticket = guard.begin();
        let guard = guard.clone();
        metadata.set(None);
        message.set(None);
        loading.set(false);
        let (Some(id), Some(current)) = (id, current) else {
            return;
        };
        if !super::local_id(&id) {
            return;
        }
        loading.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request("GET", &format!("/api/v1/media/{id}"), &current, None).await;
            if !guard.accepts(ticket, &current) {
                return;
            }
            loading.set(false);
            match result {
                Ok(media) => metadata.set(Some(media)),
                Err(f) => {
                    message.set(Some(f.message));
                    if f.unauthorized {
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
        let Some(id) = media["id"]
            .as_str()
            .filter(|id| super::local_id(id))
            .map(str::to_owned)
        else {
            return;
        };
        super::ui::change(data, history, dirty, |entry| {
            entry["featured_media_id"] = json!(id);
        });
    });
    view! {<section class="rf-featured-inspector"><h3>{text("featured.title")}</h3><p class="rf-description">{text("featured.hint")}</p><fieldset disabled=move ||busy.get()><legend class="rf-sr-only">{text("featured.title")}</legend><crate::media_ui::Picker token on_choose=choose images_only=true/>
        <Show when=move ||loading.get()><p role="status">{text("screens.loading")}</p></Show>
        {move ||message.get().map(|m|view!{<p class="rf-notice" role="status">{m}</p>})}
        {move ||metadata.get().map(|media|{let image=media["mime_type"].as_str().is_some_and(|mime|mime.starts_with("image/"));let url=media["url"].as_str().filter(|url|url.starts_with("/uploads/")).map(str::to_owned);view!{<p>{media["original_name"].as_str().unwrap_or_default().to_owned()}</p>{url.filter(|_|image).map(|url|view!{<img class="rf-featured-preview" src=url alt=media["alt"].as_str().unwrap_or_default().to_owned()/>})}<a href=format!("#media:{}",media["id"].as_str().unwrap_or_default())>{text("featured.metadata")}</a>}})}
        <Show when=move ||image_id.get().is_some()><button type="button" class="rf-button" on:click=move |_|super::ui::change(data,history,dirty,|entry|entry["featured_media_id"]=Value::Null)>{text("featured.clear")}</button></Show>
    </fieldset></section>}
}
