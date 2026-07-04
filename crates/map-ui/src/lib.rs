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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapStyle {
    Road,
    Dark,
}

impl MapStyle {
    pub fn as_key(self) -> &'static str {
        match self {
            Self::Road => "roadmap",
            Self::Dark => "dark",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapProjection {
    Flat2d,
    Globe3d,
}

impl MapProjection {
    pub fn as_key(self) -> &'static str {
        match self {
            Self::Flat2d => "2d",
            Self::Globe3d => "3d",
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(module = "/src/maplibre_shim.js")]
extern "C" {
    #[wasm_bindgen(js_name = mountMap)]
    fn mount_map(element_id: &str, style_key: &str, projection_key: &str);

    #[wasm_bindgen(js_name = setMapStyle)]
    fn set_map_style(element_id: &str, style_key: &str);

    #[wasm_bindgen(js_name = setMapProjection)]
    fn set_map_projection(element_id: &str, projection_key: &str);

    #[wasm_bindgen(js_name = syncMapPoints)]
    fn sync_map_points(element_id: &str, points_json: &str);

    #[wasm_bindgen(js_name = focusMapPoint)]
    fn focus_map_point(element_id: &str, lng: f64, lat: f64);

    #[wasm_bindgen(js_name = zoomMapIn)]
    fn zoom_map_in(element_id: &str);

    #[wasm_bindgen(js_name = zoomMapOut)]
    fn zoom_map_out(element_id: &str);
}

#[cfg(target_arch = "wasm32")]
pub fn mount(element_id: &str, style: MapStyle, projection: MapProjection) {
    mount_map(element_id, style.as_key(), projection.as_key());
}

#[cfg(target_arch = "wasm32")]
pub fn set_style(element_id: &str, style: MapStyle) {
    set_map_style(element_id, style.as_key());
}

#[cfg(target_arch = "wasm32")]
pub fn set_projection(element_id: &str, projection: MapProjection) {
    set_map_projection(element_id, projection.as_key());
}

#[cfg(target_arch = "wasm32")]
pub fn sync_points(element_id: &str, points_json: &str) {
    sync_map_points(element_id, points_json);
}

#[cfg(target_arch = "wasm32")]
pub fn focus_point(element_id: &str, lng: f64, lat: f64) {
    focus_map_point(element_id, lng, lat);
}

#[cfg(target_arch = "wasm32")]
pub fn zoom_in(element_id: &str) {
    zoom_map_in(element_id);
}

#[cfg(target_arch = "wasm32")]
pub fn zoom_out(element_id: &str) {
    zoom_map_out(element_id);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn mount(_element_id: &str, _style: MapStyle, _projection: MapProjection) {
    // MapLibre is browser-only; native server builds keep this as a no-op.
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_style(_element_id: &str, _style: MapStyle) {
    // MapLibre is browser-only; native server builds keep this as a no-op.
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_projection(_element_id: &str, _projection: MapProjection) {
    // MapLibre is browser-only; native server builds keep this as a no-op.
}

#[cfg(not(target_arch = "wasm32"))]
pub fn sync_points(_element_id: &str, _points_json: &str) {
    // MapLibre is browser-only; native server builds keep this as a no-op.
}

#[cfg(not(target_arch = "wasm32"))]
pub fn focus_point(_element_id: &str, _lng: f64, _lat: f64) {
    // MapLibre is browser-only; native server builds keep this as a no-op.
}

#[cfg(not(target_arch = "wasm32"))]
pub fn zoom_in(_element_id: &str) {
    // MapLibre is browser-only; native server builds keep this as a no-op.
}

#[cfg(not(target_arch = "wasm32"))]
pub fn zoom_out(_element_id: &str) {
    // MapLibre is browser-only; native server builds keep this as a no-op.
}
