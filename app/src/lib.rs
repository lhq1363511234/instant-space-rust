#![recursion_limit = "512"]

#[cfg(feature = "ssr")]
pub mod agent_api;
pub mod app;
pub mod app_state;
pub mod components;
pub mod error;
pub mod i18n;
pub mod pages;
#[cfg(feature = "ssr")]
pub mod realtime;
pub mod server;

#[cfg(feature = "hydrate")]
use wasm_bindgen::prelude::*;

// Native map bootstrap (map_boot.js) must not touch #map until Leptos hydration
// has finished, otherwise MapLibre inserts DOM nodes that corrupt the hydration
// cursor and every on:click after #map fails to bind. This flag/event lets the
// native script wait for a safe moment.
#[cfg(feature = "hydrate")]
#[wasm_bindgen(
    inline_js = "export function __instant_mark_hydrated() { try { window.__instantSpaceHydrated = true; window.dispatchEvent(new Event('instant-space-hydrated')); } catch (e) {} }"
)]
extern "C" {
    fn __instant_mark_hydrated();
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(app::App);
    __instant_mark_hydrated();
}
