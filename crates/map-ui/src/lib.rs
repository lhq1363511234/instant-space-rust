#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct MapPoint {
    pub id: String,
    pub label: String,
    pub lat: f64,
    pub lng: f64,
    pub is_public: bool,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(module = "/src/maplibre_shim.js")]
extern "C" {
    #[wasm_bindgen(js_name = mountMap)]
    fn mount_map(element_id: &str, style_url: &str);
}

#[cfg(target_arch = "wasm32")]
pub fn mount(element_id: &str, style_url: &str) {
    mount_map(element_id, style_url);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn mount(_element_id: &str, _style_url: &str) {
    // MapLibre is browser-only; native server builds keep this as a no-op.
}
