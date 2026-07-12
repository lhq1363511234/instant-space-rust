const DEFAULT_CENTER = [121.47, 31.23];
const MAP_STORES = (globalThis.__instantSpaceMaps ||= new globalThis.Map());
const PENDING_POINTS = (globalThis.__instantSpacePendingPoints ||= new globalThis.Map());
const LAST_POINTS = (globalThis.__instantSpaceLastPoints ||= new globalThis.Map());
const PENDING_VIEWS = (globalThis.__instantSpacePendingViews ||= new globalThis.Map());

const MAP_STYLES = {
  roadmap: {
    label: "roadmap",
    stylePath: "/styles/liberty",
  },
  dark: {
    label: "dark",
    stylePath: "/styles/dark",
  },
};

const MAP_PROJECTIONS = {
  "2d": {
    label: "2d",
    maplibreProjection: "mercator",
  },
  "3d": {
    label: "3d",
    maplibreProjection: "globe",
  },
};

function mapStyle(styleKey) {
  const style = MAP_STYLES[styleKey] || MAP_STYLES.roadmap;
  return `${openFreeMapBase()}${style.stylePath}`;
}

function openFreeMapBase() {
  if (globalThis.location?.pathname?.startsWith("/inspace")) {
    return new URL("/inspace/ofm", globalThis.location.origin).href;
  }

  return "https://tiles.openfreemap.org";
}

function getStyleKey(styleKey) {
  if (styleKey === "roadmap" || styleKey === "dark") {
    return styleKey;
  }

  return "roadmap";
}

function getProjectionKey(projectionKey) {
  if (projectionKey === "2d" || projectionKey === "3d") {
    return projectionKey;
  }

  return "2d";
}

function projectionSpec(projectionKey) {
  const config = MAP_PROJECTIONS[projectionKey] || MAP_PROJECTIONS["2d"];
  return { type: config.maplibreProjection };
}

function getStore(elementId) {
  return MAP_STORES.get(elementId);
}

function cleanupStore(elementId) {
  const store = getStore(elementId);
  if (!store) {
    return;
  }

  removePicker(store);

  for (const marker of store.markers || []) {
    marker.remove();
  }

  try {
    store.map?.remove?.();
  } catch {
    // MapLibre may already have been detached by a client-side route change.
  }

  MAP_STORES.delete(elementId);
}

function setElementStyleState(element, styleKey) {
  element.dataset.mapStyle = styleKey;
}

function setElementProjectionState(element, projectionKey) {
  element.dataset.mapProjection = projectionKey;
}

function formatCoordinate(value) {
  return Number(value).toFixed(6);
}

function dispatchInput(input) {
  input.dispatchEvent(new Event("input", { bubbles: true }));
  input.dispatchEvent(new Event("change", { bubbles: true }));
}

function setInputValue(inputId, value) {
  const input = document.getElementById(inputId);
  if (!input || value === undefined || value === null) {
    return;
  }

  input.value = String(value);
  dispatchInput(input);
}

function setPickedCoordinates(store, lng, lat) {
  if (!Number.isFinite(lng) || !Number.isFinite(lat)) {
    return;
  }

  const picker = store.picker;
  const latInput = document.getElementById(picker?.latInputId || "");
  const lngInput = document.getElementById(picker?.lngInputId || "");
  if (latInput) {
    latInput.value = formatCoordinate(lat);
    dispatchInput(latInput);
  }
  if (lngInput) {
    lngInput.value = formatCoordinate(lng);
    dispatchInput(lngInput);
  }

  if (picker?.marker) {
    picker.marker.setLngLat([lng, lat]);
  }

  store.map?.getContainer?.()?.dispatchEvent(
    new CustomEvent("instant-map-picked", {
      bubbles: true,
      detail: { lng, lat },
    }),
  );

  reverseGeocodePickedPoint(store, lng, lat);
}

async function reverseGeocodePickedPoint(store, lng, lat) {
  const picker = store?.picker;
  if (!picker || !Number.isFinite(lng) || !Number.isFinite(lat)) {
    return;
  }

  const sequence = (picker.geocodeSequence || 0) + 1;
  picker.geocodeSequence = sequence;
  const element = store.map?.getContainer?.();
  if (element) {
    element.dataset.geocoding = "true";
  }

  try {
    const local = await reverseGeocodeFromServer(lng, lat);
    if (local && picker.geocodeSequence === sequence) {
      applyReverseLocation(local);
      return;
    }

    const language = globalThis.navigator?.language?.toLowerCase().startsWith("zh") ? "zh" : "en";
    const url = new URL("https://api.bigdatacloud.net/data/reverse-geocode-client");
    url.searchParams.set("latitude", String(lat));
    url.searchParams.set("longitude", String(lng));
    url.searchParams.set("localityLanguage", language);
    const response = await fetch(url);
    if (!response.ok || picker.geocodeSequence !== sequence) {
      return;
    }
    const data = await response.json();
    const location = normalizeReverseGeocode(data);
    applyReverseLocation(location);
  } catch {
    // Reverse geocoding is a best-effort enhancement; manual fields remain editable.
  } finally {
    if (element && picker.geocodeSequence === sequence) {
      delete element.dataset.geocoding;
    }
  }
}

async function reverseGeocodeFromServer(lng, lat) {
  const base = globalThis.location?.pathname?.startsWith("/inspace")
    ? "/inspace/geo/reverse"
    : "/geo/reverse";
  const url = new URL(base, globalThis.location?.origin || "http://localhost");
  url.searchParams.set("lat", String(lat));
  url.searchParams.set("lng", String(lng));
  const response = await fetch(url, { headers: { accept: "application/json" } });
  if (!response.ok) {
    return null;
  }
  return response.json();
}

function applyReverseLocation(location) {
  setInputValue("space-country", location.country);
  setInputValue("space-province", location.province);
  setInputValue("space-city", location.city);
  setInputValue("space-district", location.district);
  setInputValue("space-spot", location.spot_name || location.spotName || location.locality);
}

function normalizeReverseGeocode(data) {
  const administrative = Array.isArray(data?.localityInfo?.administrative)
    ? data.localityInfo.administrative
    : [];
  const province =
    data?.principalSubdivision ||
    administrative.find((item) => item.adminLevel === 4)?.name ||
    "";
  const city =
    data?.city ||
    administrative.find((item) => item.adminLevel === 6)?.name ||
    data?.locality ||
    "";
  const district =
    data?.locality && data.locality !== city
      ? data.locality
      : administrative.find((item) => item.adminLevel === 8 || item.adminLevel === 10)?.name || "";

  return {
    country: data?.countryName || "",
    province,
    city,
    district,
    spot_name: data?.locality || data?.city || "",
  };
}

function makePickerMarker() {
  const markerElement = document.createElement("div");
  markerElement.className = "map-picker-marker";
  markerElement.innerHTML = `
    <span class="map-picker-dot" aria-hidden="true"></span>
    <span class="map-picker-label">Selected</span>
  `;
  return markerElement;
}

function removePicker(store) {
  if (!store?.picker) {
    return;
  }

  if (store.picker.onClick) {
    store.map?.off?.("click", store.picker.onClick);
  }
  if (store.picker.onDomClick) {
    store.map?.getContainer?.()?.removeEventListener("click", store.picker.onDomClick);
  }

  store.picker.marker?.remove?.();
  const canvas = store.map?.getCanvas?.();
  if (canvas) {
    canvas.style.cursor = "";
  }
  delete store.picker;
}

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

function escapeSelectorValue(value) {
  if (globalThis.CSS?.escape) {
    return globalThis.CSS.escape(value);
  }

  return String(value).replaceAll('"', '\\"');
}

function parsePoints(pointsJson) {
  try {
    const parsed = JSON.parse(pointsJson);
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

function renderMarkers(elementId, points) {
  LAST_POINTS.set(elementId, points);
  const store = getStore(elementId);
  if (!store?.map || !globalThis.maplibregl) {
    PENDING_POINTS.set(elementId, points);
    // Keep retrying briefly while MapLibre / container becomes ready.
    if (!store?.map) {
      globalThis.setTimeout?.(() => {
        const latest = LAST_POINTS.get(elementId);
        if (latest) {
          renderMarkers(elementId, latest);
        }
      }, 180);
    }
    return;
  }

  for (const marker of store.markers || []) {
    marker.remove();
  }

  const validPoints = (points || []).filter(
    (point) => Number.isFinite(Number(point.lng)) && Number.isFinite(Number(point.lat)),
  );

  store.markers = validPoints.map((point) => {
    const lng = Number(point.lng);
    const lat = Number(point.lat);
    const label = point.name_zh || point.name_en || "Space";
    const markerElement = document.createElement("button");
    markerElement.type = "button";
    markerElement.className = `map-marker ${point.is_public ? "is-public" : "is-private"}`;
    markerElement.dataset.spaceMarker = point.id;
    markerElement.setAttribute("aria-label", `Open ${label}`);
    markerElement.innerHTML = `
      <span class="map-marker-pin" aria-hidden="true"></span>
      <span class="map-marker-label">${escapeHtml(label)}</span>
    `;

    markerElement.addEventListener("click", (event) => {
      event.preventDefault();
      event.stopPropagation();
      focusMapPoint(elementId, lng, lat);
      const selector = `[data-space-select="${escapeSelectorValue(point.id)}"]`;
      document.querySelector(selector)?.click();
    });

    return new globalThis.maplibregl.Marker({
      element: markerElement,
      anchor: "bottom",
    })
      .setLngLat([lng, lat])
      .addTo(store.map);
  });

  fitMapToPoints(store.map, validPoints);
}

function fitMapToPoints(map, points) {
  const validPoints = points.filter((point) => Number.isFinite(point.lng) && Number.isFinite(point.lat));
  if (!validPoints.length) {
    map.easeTo({ center: DEFAULT_CENTER, zoom: 10, duration: 500 });
    return;
  }

  if (validPoints.length === 1) {
    easeToPoint(map, validPoints[0]);
    return;
  }

  const lngs = validPoints.map((point) => point.lng);
  const lats = validPoints.map((point) => point.lat);
  const minLng = Math.min(...lngs);
  const maxLng = Math.max(...lngs);
  const minLat = Math.min(...lats);
  const maxLat = Math.max(...lats);
  if (Math.abs(maxLng - minLng) < 0.01 && Math.abs(maxLat - minLat) < 0.01) {
    easeToPoint(map, validPoints[0]);
    return;
  }

  const bounds = validPoints.reduce((nextBounds, point) => {
    nextBounds.extend([point.lng, point.lat]);
    return nextBounds;
  }, new globalThis.maplibregl.LngLatBounds([validPoints[0].lng, validPoints[0].lat], [validPoints[0].lng, validPoints[0].lat]));

  const isCompact = globalThis.matchMedia?.("(max-width: 760px)")?.matches;
  map.fitBounds(bounds, {
    padding: isCompact
      ? { top: 340, right: 24, bottom: 280, left: 24 }
      : { top: 220, right: 320, bottom: 140, left: 470 },
    maxZoom: 11.5,
    duration: 650,
    essential: false,
  });
}

function easeToPoint(map, point) {
  map.easeTo({
    center: [point.lng, point.lat],
    zoom: 11.5,
    duration: 650,
    essential: false,
  });
}

function applyProjection(map, projectionKey) {
  if (!map || typeof map.setProjection !== "function") {
    return;
  }

  try {
    map.setProjection(projectionSpec(projectionKey));
  } catch {
    map.setProjection(MAP_PROJECTIONS[projectionKey]?.maplibreProjection || "mercator");
  }
}

export function mountMap(elementId, styleKey, projectionKey) {
  const element = document.getElementById(elementId);
  if (!element || !globalThis.maplibregl) {
    globalThis.requestAnimationFrame?.(() => mountMap(elementId, styleKey, projectionKey));
    return;
  }

  const nextStyle = getStyleKey(styleKey);
  const nextProjection = getProjectionKey(projectionKey);
  const existing = getStore(elementId);
  const existingContainer = existing?.map?.getContainer?.();
  if (existing && existingContainer && existingContainer !== element) {
    cleanupStore(elementId);
    delete element.dataset.mapMounted;
  }

  // Reuse native map_boot instance if present
  if (!getStore(elementId) && element.__instantBootMap) {
    MAP_STORES.set(elementId, {
      map: element.__instantBootMap,
      markers: [],
      picker: null,
      styleKey: nextStyle,
      projectionKey: nextProjection,
    });
    element.dataset.mapMounted = "true";
    try { element.__instantBootMap.resize(); } catch {}
    setMapStyle(elementId, nextStyle);
    setMapProjection(elementId, nextProjection);
    return;
  }

  if (element.dataset.mapMounted === "true" && getStore(elementId)?.map) {
    setMapStyle(elementId, nextStyle);
    setMapProjection(elementId, nextProjection);
    return;
  }
  // If dataset says mounted but no store (stale), allow remount
  if (element.dataset.mapMounted === "true" && !getStore(elementId)?.map) {
    delete element.dataset.mapMounted;
  }

  element.dataset.mapMounted = "true";
  setElementStyleState(element, nextStyle);
  setElementProjectionState(element, nextProjection);

  const map = new globalThis.maplibregl.Map({
    container: elementId,
    style: mapStyle(nextStyle),
    center: DEFAULT_CENTER,
    zoom: 10,
    minZoom: 3,
    maxZoom: 18,
    attributionControl: false,
    renderWorldCopies: false,
    dragPan: true,
    scrollZoom: true,
    dragRotate: false,
    touchZoomRotate: true,
    projection: projectionSpec(nextProjection),
  });

  map.dragPan?.enable?.();
  map.scrollZoom?.enable?.();
  map.touchZoomRotate?.enable?.();

  map.addControl(
    new globalThis.maplibregl.AttributionControl({ compact: true }),
    "bottom-right",
  );

  if (elementId === "create-space-map" && globalThis.maplibregl.NavigationControl) {
    map.addControl(
      new globalThis.maplibregl.NavigationControl({ showCompass: false }),
      "top-right",
    );
  }

  MAP_STORES.set(elementId, {
    map,
    markers: [],
    picker: null,
    styleKey: nextStyle,
    projectionKey: nextProjection,
  });

  const forceReady = () => {
    try {
      map.resize();
    } catch {}
    const pending = PENDING_POINTS.get(elementId) || LAST_POINTS.get(elementId);
    if (pending) {
      renderMarkers(elementId, pending);
      PENDING_POINTS.delete(elementId);
    }
    const pendingView = PENDING_VIEWS.get(elementId);
    if (pendingView) {
      if (applyMapView(map, pendingView.lng, pendingView.lat, pendingView.zoom)) {
        PENDING_VIEWS.delete(elementId);
      }
    }
  };

  // Hero overlay often mounts map at wrong size; keep forcing resize until visible.
  forceReady();
  [50, 150, 300, 600, 1000, 1600].forEach((ms) => {
    globalThis.setTimeout?.(forceReady, ms);
  });
  map.once("load", forceReady);
  map.once("idle", forceReady);
  map.on("error", (err) => {
    console.error("[instant-space] map error", err?.error || err);
  });
}

export function destroyMap(elementId) {
  cleanupStore(elementId);
  const element = document.getElementById(elementId);
  if (element) {
    delete element.dataset.mapMounted;
  }
}

export function setMapStyle(elementId, styleKey) {
  const element = document.getElementById(elementId);
  const store = getStore(elementId);
  const nextStyle = getStyleKey(styleKey);

  if (element) {
    setElementStyleState(element, nextStyle);
  }

  if (!store?.map || store.styleKey === nextStyle) {
    return;
  }

  store.styleKey = nextStyle;
  store.map.setStyle(mapStyle(nextStyle));
  store.map.once("styledata", () => {
    applyProjection(store.map, store.projectionKey);
    const latest = LAST_POINTS.get(elementId) || PENDING_POINTS.get(elementId);
    if (latest) {
      renderMarkers(elementId, latest);
    }
  });
}

export function setMapProjection(elementId, projectionKey) {
  const element = document.getElementById(elementId);
  const store = getStore(elementId);
  const nextProjection = getProjectionKey(projectionKey);

  if (element) {
    setElementProjectionState(element, nextProjection);
  }

  if (!store?.map) {
    return;
  }

  store.projectionKey = nextProjection;
  applyProjection(store.map, nextProjection);
}

export function syncMapPoints(elementId, pointsJson) {
  renderMarkers(elementId, parsePoints(pointsJson));
}

export function focusMapPoint(elementId, lng, lat) {
  const store = getStore(elementId);
  if (!store?.map || !Number.isFinite(lng) || !Number.isFinite(lat)) {
    return;
  }

  store.map.flyTo({
    center: [lng, lat],
    zoom: Math.max(store.map.getZoom(), 10),
    speed: 0.9,
    curve: 1.35,
    essential: false,
  });
}

export function enableCoordinatePicker(elementId, latInputId, lngInputId, lng, lat) {
  const element = document.getElementById(elementId);
  const store = getStore(elementId);
  if (!element || !store?.map || !globalThis.maplibregl) {
    globalThis.requestAnimationFrame?.(() =>
      enableCoordinatePicker(elementId, latInputId, lngInputId, lng, lat),
    );
    return;
  }

  const nextLng = Number.isFinite(lng) ? lng : DEFAULT_CENTER[0];
  const nextLat = Number.isFinite(lat) ? lat : DEFAULT_CENTER[1];

  if (!store.picker) {
    const marker = new globalThis.maplibregl.Marker({
      element: makePickerMarker(),
      anchor: "bottom",
    })
      .setLngLat([nextLng, nextLat])
      .addTo(store.map);

    const onClick = (event) => {
      const point = event?.lngLat;
      if (!point) {
        return;
      }
      setPickedCoordinates(store, point.lng, point.lat);
    };
    const onDomClick = (event) => {
      if (event.target?.closest?.(".maplibregl-ctrl, .map-picker-marker")) {
        return;
      }
      const rect = element.getBoundingClientRect();
      const point = store.map.unproject([
        event.clientX - rect.left,
        event.clientY - rect.top,
      ]);
      setPickedCoordinates(store, point.lng, point.lat);
    };

    store.map.on("click", onClick);
    element.addEventListener("click", onDomClick);
    store.picker = { marker, onClick, onDomClick, latInputId, lngInputId };
    const canvas = store.map.getCanvas?.();
    if (canvas) {
      canvas.style.cursor = "crosshair";
    }
  } else {
    store.picker.latInputId = latInputId;
    store.picker.lngInputId = lngInputId;
    store.picker.marker?.setLngLat([nextLng, nextLat]);
  }

  element.dataset.mapPicker = "true";
  store.map.easeTo({
    center: [nextLng, nextLat],
    zoom: Math.max(store.map.getZoom(), 10.5),
    duration: 350,
    essential: false,
  });
}

export function disableCoordinatePicker(elementId) {
  const element = document.getElementById(elementId);
  const store = getStore(elementId);
  if (store) {
    removePicker(store);
  }
  if (element) {
    delete element.dataset.mapPicker;
  }
}

export function zoomMapIn(elementId) {
  getStore(elementId)?.map?.zoomIn({ duration: 220 });
}

export function zoomMapOut(elementId) {
  getStore(elementId)?.map?.zoomOut({ duration: 220 });
}


export function getPageOrigin() {
  try {
    return globalThis.location?.origin || "";
  } catch {
    return "";
  }
}

export function copyText(text) {
  const value = String(text || "");
  if (!value) {
    return false;
  }

  try {
    if (globalThis.navigator?.clipboard?.writeText) {
      globalThis.navigator.clipboard.writeText(value).catch(() => {
        fallbackCopyText(value);
      });
      return true;
    }
  } catch {
    // fall through
  }

  return fallbackCopyText(value);
}

function fallbackCopyText(value) {
  try {
    const area = document.createElement("textarea");
    area.value = value;
    area.setAttribute("readonly", "true");
    area.style.position = "fixed";
    area.style.opacity = "0";
    document.body.appendChild(area);
    area.select();
    const ok = document.execCommand("copy");
    area.remove();
    return Boolean(ok);
  } catch {
    return false;
  }
}


function applyMapView(map, lng, lat, zoom) {
  if (!map || !Number.isFinite(lng) || !Number.isFinite(lat)) {
    return false;
  }
  // Map often mounts under hero overlay (wrong size). Always resize before fly.
  try {
    map.resize?.();
  } catch {
    // ignore
  }
  const nextZoom = Number.isFinite(zoom) ? zoom : Math.max(map.getZoom?.() || 4, 4);
  try {
    map.flyTo({
      center: [lng, lat],
      zoom: nextZoom,
      speed: 1.05,
      curve: 1.25,
      essential: true,
    });
    return true;
  } catch {
    try {
      map.jumpTo({ center: [lng, lat], zoom: nextZoom });
      return true;
    } catch {
      return false;
    }
  }
}

export function resizeMap(elementId) {
  const store = getStore(elementId);
  if (!store?.map) {
    return false;
  }
  try {
    store.map.resize();
    return true;
  } catch {
    return false;
  }
}

export function revealMap(elementId) {
  // Called when first screen closes: fix blank canvas + reapply pending view.
  const kick = (map) => {
    try { map.resize(); } catch {}
    const pendingView = PENDING_VIEWS.get(elementId);
    if (pendingView) {
      applyMapView(map, pendingView.lng, pendingView.lat, pendingView.zoom);
    } else {
      try {
        const c = map.getCenter?.();
        const z = map.getZoom?.();
        if (c) {
          map.jumpTo({ center: c, zoom: z });
        }
        map.triggerRepaint?.();
      } catch {}
    }
  };

  const store = getStore(elementId);
  if (!store?.map) {
    // Map not mounted yet — mount may still be pending under hero.
    let n = 0;
    const timer = globalThis.setInterval(() => {
      n += 1;
      const s = getStore(elementId);
      if (s?.map) {
        kick(s.map);
        globalThis.clearInterval(timer);
      } else if (n >= 50) {
        // last resort: try mount if MapLibre is present and container exists
        try {
          if (globalThis.maplibregl && document.getElementById(elementId)) {
            mountMap(elementId, "roadmap", "2d");
          }
        } catch {}
        globalThis.clearInterval(timer);
      }
    }, 100);
    return false;
  }
  kick(store.map);
  [80, 200, 400, 800].forEach((ms) => {
    globalThis.setTimeout?.(() => {
      const s = getStore(elementId);
      if (s?.map) kick(s.map);
    }, ms);
  });
  return true;
}

export function focusMapView(elementId, lng, lat, zoom) {
  if (!Number.isFinite(lng) || !Number.isFinite(lat)) {
    return;
  }
  const payload = { lng, lat, zoom: Number.isFinite(zoom) ? zoom : 4.5 };
  PENDING_VIEWS.set(elementId, payload);

  const tryApply = () => {
    const store = getStore(elementId);
    if (!store?.map) {
      return false;
    }
    const pending = PENDING_VIEWS.get(elementId) || payload;
    const ok = applyMapView(store.map, pending.lng, pending.lat, pending.zoom);
    if (ok) {
      PENDING_VIEWS.delete(elementId);
    }
    return ok;
  };

  if (tryApply()) {
    return;
  }

  // Map not ready yet: retry a few times after mount/style load.
  let attempts = 0;
  const timer = globalThis.setInterval(() => {
    attempts += 1;
    if (tryApply() || attempts >= 40) {
      globalThis.clearInterval(timer);
    }
  }, 150);
}


// Keep map canvas correct when hero closes / viewport changes.
if (typeof globalThis !== "undefined" && globalThis.addEventListener) {
  globalThis.addEventListener("resize", () => {
    for (const [id, store] of MAP_STORES.entries()) {
      try {
        store?.map?.resize?.();
      } catch {}
    }
  });
}
