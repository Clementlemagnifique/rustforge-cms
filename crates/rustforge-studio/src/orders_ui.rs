//! Manual orders only. This screen never initiates a payment-provider transaction.
use crate::{
    http::{self, Guard},
    messages::text,
};
use leptos::prelude::*;
use serde_json::{Value, json};
#[component]
pub fn Orders(token: ReadSignal<Option<String>>, on_expired: Callback<()>) -> impl IntoView {
    let rows = RwSignal::new(Vec::<Value>::new());
    let products = RwSignal::new(Vec::<Value>::new());
    let selected_product = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let customer = RwSignal::new(String::new());
    let pending = RwSignal::new(None::<(String, &'static str)>);
    let create_confirm = RwSignal::new(false);
    let busy = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let page = RwSignal::new(0_u64);
    let refresh = RwSignal::new(0_u64);
    let guard = Guard::new();
    let loader = guard.clone();
    Effect::new(move |_| {
        let token = token.get();
        let page = page.get();
        refresh.get();
        let ticket = loader.begin();
        let Some(token) = token else {
            rows.set(vec![]);
            return;
        };
        let guard = loader.clone();
        busy.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request(
                "GET",
                &format!("/api/v1/ecommerce/orders?limit=50&offset={}", page * 50),
                &token,
                None,
            )
            .await;
            let catalog = http::request("GET", "/api/v1/ecommerce/products", &token, None).await;
            if !guard.accepts(ticket, &token) {
                return;
            }
            busy.set(false);
            if let Ok(value) = catalog {
                products.set(
                    serde_json::from_value::<Vec<Value>>(value)
                        .unwrap_or_default()
                        .into_iter()
                        .filter(|value| value["status"] == "published")
                        .collect(),
                );
            }
            match result {
                Ok(value) => rows.set(serde_json::from_value(value).unwrap_or_default()),
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
        let Some((id, action)) = pending.get_untracked() else {
            return;
        };
        let ticket = action_guard.begin();
        let guard = action_guard.clone();
        busy.set(true);
        pending.set(None);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request(
                "POST",
                &format!("/api/v1/ecommerce/orders/{id}/{action}"),
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
                    message.set(Some(text("form.saved").into()));
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
    let create = move |_| {
        if busy.get_untracked() {
            return;
        }
        let Some(token) = token.get_untracked() else {
            return;
        };
        let product = selected_product.get_untracked();
        if !crate::editor::local_id(&product) {
            return;
        }
        let body = json!({"customer_email":email.get_untracked(),"customer_name":customer.get_untracked(),"items":[{"product_id":product}],"gateway":"manual"});
        let ticket = guard.begin();
        let guard = guard.clone();
        busy.set(true);
        create_confirm.set(false);
        wasm_bindgen_futures::spawn_local(async move {
            let result = http::request(
                "POST",
                "/api/v1/public/ecommerce/checkout",
                &token,
                Some(&body),
            )
            .await;
            if !guard.accepts(ticket, &token) {
                return;
            }
            busy.set(false);
            match result {
                Ok(_) => {
                    message.set(Some(text("orders.created").into()));
                    email.set(String::new());
                    customer.set(String::new());
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
    view! {<h1>{text("orders.title")}</h1><p class="rf-notice">{text("orders.manual_hint")}</p>{move ||message.get().map(|message|view!{<p class="rf-notice" role="status">{message}</p>})}<details class="rf-panel rf-screen-panel rf-layout-options"><summary>{text("orders.new")}</summary><form class="rf-compact-form" on:submit=move |event|{event.prevent_default();create_confirm.set(true);}>
    <label for="rf-order-email">{text("screens.email")}</label><input id="rf-order-email" type="email" required maxlength="254" prop:value=move ||email.get() on:input=move |event|email.set(event_target_value(&event))/><label for="rf-order-name">{text("screens.name")}</label><input id="rf-order-name" maxlength="200" prop:value=move ||customer.get() on:input=move |event|customer.set(event_target_value(&event))/><label for="rf-order-product">{text("editor.product")}</label><select id="rf-order-product" required prop:value=move ||selected_product.get() on:change=move |event|selected_product.set(event_target_value(&event))><option value="">{text("orders.choose")}</option>{move ||products.get().into_iter().map(|product|view!{<option value=product["id"].as_str().unwrap_or_default().to_owned()>{product["title"].as_str().unwrap_or_default().to_owned()}" — "{product["price_cents"].as_i64().map(|value|format!("{:.2}",value as f64/100.)).unwrap_or_default()}" EUR"</option>}).collect_view()}</select><button class="rf-button" disabled=move ||busy.get()>{text("orders.prepare")}</button></form><Show when=move ||create_confirm.get()><p>{text("orders.create_confirm")}</p><button class="rf-button primary" disabled=move ||busy.get() on:click=create.clone()>{text("orders.create")}</button><button class="rf-button" on:click=move |_|create_confirm.set(false)>{text("form.cancel")}</button></Show></details>
    <Show when=move ||pending.get().is_some()><div class="rf-notice"><p>{text("orders.action_confirm")}</p><button class="rf-button primary" disabled=move ||busy.get() on:click=apply.clone()>{text("module.confirm")}</button><button class="rf-button" on:click=move |_|pending.set(None)>{text("form.cancel")}</button></div></Show>
    <div class="rf-table-scroll"><table class="rf-list-table"><thead><tr><th>{text("orders.number")}</th><th>{text("screens.email")}</th><th>{text("screens.status")}</th><th>{text("orders.total")}</th><th>{text("module.action")}</th></tr></thead><tbody>{move ||rows.get().into_iter().map(|row|{let id=row["id"].as_str().unwrap_or_default().to_owned();let cancel=id.clone();let complete=id.clone();let state=row["status"].as_str().unwrap_or_default().to_owned();view!{<tr><th scope="row">{row["order_number"].as_str().unwrap_or_default().to_owned()}</th><td>{row["customer_email"].as_str().unwrap_or_default().to_owned()}</td><td>{state.clone()}</td><td>{row["total_cents"].as_i64().map(|value|format!("{:.2}",value as f64/100.)).unwrap_or_default()}" "{row["currency"].as_str().unwrap_or_default().to_owned()}</td><td>{if state=="pending"{view!{<button class="rf-button" disabled=move ||busy.get() on:click=move |_|pending.set(Some((complete.clone(),"complete")))>{text("orders.complete")}</button><button class="rf-button" disabled=move ||busy.get() on:click=move |_|pending.set(Some((cancel.clone(),"cancel")))>{text("orders.cancel")}</button>}.into_any()}else{().into_any()}}</td></tr>}}).collect_view()}</tbody></table></div><div class="rf-form-actions"><button class="rf-button" disabled=move ||busy.get()||page.get()==0 on:click=move |_|page.update(|value|*value=value.saturating_sub(1))>{text("screens.previous")}</button><button class="rf-button" disabled=move ||busy.get()||rows.get().len()<50 on:click=move |_|page.update(|value|*value+=1)>{text("screens.next")}</button></div>}
}
