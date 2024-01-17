use app::App;
use leptos::mount_to_body;
use wasm_bindgen::prelude::wasm_bindgen;
mod app;
mod components;
mod core;
mod env;

#[wasm_bindgen(start)]
pub fn start() {
    console_log::init_with_level(log::Level::Debug).unwrap_or_default();
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

