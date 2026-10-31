#[cfg(target_arch = "wasm32")]
fn main() {
    #[cfg(debug_assertions)]
    std::panic::set_hook(Box::new(|info| {
        web_sys::console::error_1(&wasm_bindgen::JsValue::from_str(&info.to_string()))
    }));
    leptos::mount::mount_to_body(rustforge_studio::app::App);
}
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("Construire cette interface avec Trunk pour wasm32-unknown-unknown");
}
