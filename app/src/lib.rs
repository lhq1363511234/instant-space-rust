#![recursion_limit = "512"]

pub mod app;
pub mod app_state;
pub mod components;
pub mod error;
pub mod i18n;
pub mod pages;
pub mod server;

#[cfg(feature = "hydrate")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "hydrate")]
#[wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(app::App);
}
