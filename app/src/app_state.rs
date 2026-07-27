use instant_map_ui::{MapProjection, MapStyle};
use leptos::prelude::*;

#[derive(Clone, Copy)]
pub struct AppRefreshState {
    pub session: RwSignal<u32>,
    pub spaces: RwSignal<u32>,
    /// Explore travel spaces panel (map list / filters).
    pub explorer_open: RwSignal<bool>,
    /// Product intro / "首页" hero card. Independent from explorer.
    pub hero_open: RwSignal<bool>,
    /// Map presentation controls live in the shared left navigation.
    pub map_style: RwSignal<MapStyle>,
    pub map_projection: RwSignal<MapProjection>,
    /// Destination-guided map/data scope.
    pub dest_country: RwSignal<String>,
    pub dest_province: RwSignal<String>,
    pub dest_city: RwSignal<String>,
    /// True after user confirms a destination (filters active).
    pub dest_confirmed: RwSignal<bool>,
}

pub fn provide_app_refresh_state() -> AppRefreshState {
    let state = AppRefreshState {
        session: RwSignal::new(0),
        spaces: RwSignal::new(0),
        // First visit: show product first screen (hero). Explorer stays closed.
        explorer_open: RwSignal::new(false),
        hero_open: RwSignal::new(true),
        map_style: RwSignal::new(MapStyle::Road),
        map_projection: RwSignal::new(MapProjection::Flat2d),
        dest_country: RwSignal::new(String::new()),
        dest_province: RwSignal::new(String::new()),
        dest_city: RwSignal::new(String::new()),
        dest_confirmed: RwSignal::new(false),
    };
    provide_context(state);
    state
}

pub fn use_app_refresh_state() -> AppRefreshState {
    use_context::<AppRefreshState>().unwrap_or_else(provide_app_refresh_state)
}

pub fn refresh_session() {
    let state = use_app_refresh_state();
    state.session.update(|value| *value += 1);
}

pub fn refresh_spaces() {
    let state = use_app_refresh_state();
    state.spaces.update(|value| *value += 1);
}

pub fn open_explorer() {
    let state = use_app_refresh_state();
    state.hero_open.set(false);
    state.explorer_open.set(true);
}

pub fn close_explorer() {
    let state = use_app_refresh_state();
    state.explorer_open.set(false);
}

pub fn open_hero() {
    let state = use_app_refresh_state();
    state.explorer_open.set(false);
    state.hero_open.set(true);
}

pub fn close_hero() {
    let state = use_app_refresh_state();
    state.hero_open.set(false);
}

pub fn clear_destination() {
    let state = use_app_refresh_state();
    state.dest_country.set(String::new());
    state.dest_province.set(String::new());
    state.dest_city.set(String::new());
    state.dest_confirmed.set(false);
    state.spaces.update(|value| *value += 1);
}

pub fn destination_label(country: &str, province: &str, city: &str) -> String {
    let parts: Vec<&str> = [country, province, city]
        .into_iter()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if parts.is_empty() {
        String::new()
    } else {
        parts.join(" · ")
    }
}
