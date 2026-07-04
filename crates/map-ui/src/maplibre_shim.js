const DEFAULT_CENTER = [121.47, 31.23];
const MAP_STORES = (globalThis.__instantSpaceMaps ||= new globalThis.Map());
const PENDING_POINTS = (globalThis.__instantSpacePendingPoints ||= new globalThis.Map());

const MAP_STYLES = {
  roadmap: {
    label: "roadmap",
    styleUrl: "https://tiles.openfreemap.org/styles/liberty",
  },
  dark: {
    label: "dark",
    styleUrl: "https://tiles.openfreemap.org/styles/dark",
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
  return (MAP_STYLES[styleKey] || MAP_STYLES.roadmap).styleUrl;
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

function setElementStyleState(element, styleKey) {
  element.dataset.mapStyle = styleKey;
}

function setElementProjectionState(element, projectionKey) {
  element.dataset.mapProjection = projectionKey;
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
  const store = getStore(elementId);
  if (!store?.map || !globalThis.maplibregl) {
    PENDING_POINTS.set(elementId, points);
    return;
  }

  for (const marker of store.markers) {
    marker.remove();
  }

  store.markers = points
    .filter((point) => Number.isFinite(point.lng) && Number.isFinite(point.lat))
    .map((point) => {
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

      markerElement.addEventListener("click", () => {
        focusMapPoint(elementId, point.lng, point.lat);
        const selector = `[data-space-select="${escapeSelectorValue(point.id)}"]`;
        document.querySelector(selector)?.click();
      });

      return new globalThis.maplibregl.Marker({
        element: markerElement,
        anchor: "bottom",
      })
        .setLngLat([point.lng, point.lat])
        .addTo(store.map);
    });

  fitMapToPoints(store.map, points);
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
    return;
  }

  const nextStyle = getStyleKey(styleKey);
  const nextProjection = getProjectionKey(projectionKey);
  if (element.dataset.mapMounted === "true") {
    setMapStyle(elementId, nextStyle);
    setMapProjection(elementId, nextProjection);
    return;
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
    projection: projectionSpec(nextProjection),
  });

  map.addControl(
    new globalThis.maplibregl.AttributionControl({ compact: true }),
    "bottom-right",
  );

  MAP_STORES.set(elementId, {
    map,
    markers: [],
    styleKey: nextStyle,
    projectionKey: nextProjection,
  });

  const pending = PENDING_POINTS.get(elementId);
  if (pending) {
    renderMarkers(elementId, pending);
    PENDING_POINTS.delete(elementId);
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
  store.map.once("styledata", () => applyProjection(store.map, store.projectionKey));
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

export function zoomMapIn(elementId) {
  getStore(elementId)?.map?.zoomIn({ duration: 220 });
}

export function zoomMapOut(elementId) {
  getStore(elementId)?.map?.zoomOut({ duration: 220 });
}
