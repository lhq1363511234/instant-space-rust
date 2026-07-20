import assert from "node:assert/strict";
import fs from "node:fs";

const appShell = fs.readFileSync("app/src/app.rs", "utf8");
const staticShell = fs.readFileSync("app/index.html", "utf8");
const wasmApi = fs.readFileSync("crates/map-ui/src/lib.rs", "utf8");
const shim = fs.readFileSync("crates/map-ui/src/maplibre_shim.js", "utf8");
const home = fs.readFileSync("app/src/components/map_home.rs", "utf8");
const header = fs.readFileSync("app/src/components/header.rs", "utf8");
const styles = fs.readFileSync("app/style/main.css", "utf8");

const pkg = JSON.parse(fs.readFileSync("package.json", "utf8"));
assert.match(pkg.dependencies?.["maplibre-gl"] || "", /^\^?5\./, "MapLibre GL JS must be pinned to the v5 line");
for (const source of [appShell, staticShell]) {
  assert.match(source, /\/vendor\/maplibre-gl\/maplibre-gl\.js/, "MapLibre GL JS must be served from local vendor assets");
  assert.match(source, /\/vendor\/maplibre-gl\/maplibre-gl\.css/, "MapLibre GL CSS must be served from local vendor assets");
}

assert.match(shim, /stylePath:\s*"\/styles\/liberty"/, "Road style must use OpenFreeMap liberty");
assert.match(shim, /stylePath:\s*"\/styles\/dark"/, "Dark style must use OpenFreeMap dark");
assert.match(shim, /\/inspace\/ofm/, "Production map runtime must proxy OpenFreeMap through the app origin under /inspace");
assert.doesNotMatch(shim, /demotiles\.maplibre\.org/, "Production map runtime must not use MapLibre demo assets");
assert.doesNotMatch(shim, /satellite|World_Imagery|ArcGIS/i, "Free OpenFreeMap pass must not ship a satellite provider");

assert.match(wasmApi, /enum MapProjection/, "WASM crate must model map projection state");
assert.match(wasmApi, /set_projection/, "WASM crate must expose projection switching");
assert.match(shim, /export function setMapProjection/, "JS adapter must expose projection switching only as an adapter call");
assert.match(shim, /setProjection/, "MapLibre adapter must call setProjection");
assert.match(shim, /dataset\.mapProjection/, "DOM state must expose the active projection for tests and debugging");
assert.match(shim, /mercator/, "2D projection must map to Mercator");
assert.match(shim, /globe/, "3D projection must map to Globe");

const navigationSources = `${home}\n${header}`;
assert.match(navigationSources, /map-projection-switcher/, "Navigation must render a projection switcher");
assert.match(navigationSources, /Switch to 2D map/, "Navigation must expose a 2D map control");
assert.match(navigationSources, /Switch to 3D globe/, "Navigation must expose a 3D globe control");
assert.match(styles, /--map-road-bg:\s*#f8f4f0/, "Road map container must match the OpenFreeMap road background");
assert.match(styles, /#map\[data-map-style="roadmap"\]/, "Road map style state must control the map container background");
assert.match(styles, /#map\[data-map-style="dark"\]/, "Dark map style state must control the map container background");
