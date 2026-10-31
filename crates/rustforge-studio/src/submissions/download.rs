pub(super) fn download_csv(id: &str, csv: &str) -> Result<(), wasm_bindgen::JsValue> {
    use wasm_bindgen::{JsCast, closure::Closure};
    let window = web_sys::window().ok_or(wasm_bindgen::JsValue::NULL)?;
    let document = window.document().ok_or(wasm_bindgen::JsValue::NULL)?;
    let parts = js_sys::Array::new();
    parts.push(&wasm_bindgen::JsValue::from_str(csv));
    let options = web_sys::BlobPropertyBag::new();
    options.set_type("text/csv;charset=utf-8");
    let blob = web_sys::Blob::new_with_str_sequence_and_options(&parts, &options)?;
    let url = web_sys::Url::create_object_url_with_blob(&blob)?;
    let result = (|| {
        let anchor = document
            .create_element("a")?
            .dyn_into::<web_sys::HtmlElement>()?;
        anchor.set_attribute("href", &url)?;
        anchor.set_attribute("download", &format!("soumissions-{id}.csv"))?;
        let body = document.body().ok_or(wasm_bindgen::JsValue::NULL)?;
        body.append_child(&anchor)?;
        anchor.click();
        anchor.remove();
        Ok(())
    })();
    if result.is_err() {
        web_sys::Url::revoke_object_url(&url)?;
    } else {
        let cleanup = Closure::once_into_js(move || {
            let _ = web_sys::Url::revoke_object_url(&url);
        });
        window.set_timeout_with_callback_and_timeout_and_arguments_0(
            cleanup.unchecked_ref(),
            30_000,
        )?;
    }
    result
}
