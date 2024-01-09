use app::App;
use leptos::*;
mod app;
mod components;
mod core;
mod env;

fn main() {
    console_log::init_with_level(log::Level::Debug).unwrap_or_default();
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

