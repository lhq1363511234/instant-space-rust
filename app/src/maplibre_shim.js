export function mountMap(elementId, styleUrl) {
  const element = document.getElementById(elementId);
  if (!element || element.dataset.mapMounted === "true") {
    return;
  }

  element.dataset.mapMounted = "true";
  const map = new maplibregl.Map({
    container: elementId,
    style: styleUrl || "https://demotiles.maplibre.org/style.json",
    center: [104.1954, 35.8617],
    zoom: 3.4
  });

  map.addControl(new maplibregl.NavigationControl({ showCompass: false }), "top-left");
}
