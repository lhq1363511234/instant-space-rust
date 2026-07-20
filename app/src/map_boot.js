(function () {
  function qs(name) {
    try { return new URLSearchParams(location.search).get(name); } catch (e) { return null; }
  }
  function ofmBase() {
    if ((location.pathname || '').indexOf('/inspace') === 0) {
      return location.origin + '/inspace/ofm';
    }
    return 'https://tiles.openfreemap.org';
  }
  function countryCenter(country) {
    var key = String(country || '').trim().toLowerCase();
    // Capitals: [lng, lat, zoom]
    var table = {
      china: [116.39723, 39.9075, 5.8],
      japan: [139.69171, 35.6895, 5.8],
      'united states': [-77.0369, 38.9072, 5.5],
      usa: [-77.0369, 38.9072, 5.5],
      us: [-77.0369, 38.9072, 5.5],
      'south korea': [126.978, 37.5665, 6],
      korea: [126.978, 37.5665, 6],
      thailand: [100.5018, 13.7563, 6],
      singapore: [103.8198, 1.3521, 10.5],
      'united kingdom': [-0.12574, 51.50853, 6],
      'great britain': [-0.12574, 51.50853, 6],
      britain: [-0.12574, 51.50853, 6],
      uk: [-0.12574, 51.50853, 6],
      france: [2.3522, 48.8566, 6],
      germany: [13.405, 52.52, 6],
      australia: [149.12807, -35.28346, 5.5],
      canada: [-75.6972, 45.4215, 5.5],
      india: [77.209, 28.6139, 5.8],
      indonesia: [106.8456, -6.2088, 5.8],
      malaysia: [101.6869, 3.139, 6.2],
      vietnam: [105.8342, 21.0278, 6],
      russia: [37.6173, 55.7558, 5.2],
      brazil: [-47.92972, -15.77972, 5.5],
      mexico: [-99.1332, 19.4326, 5.8],
      italy: [12.4964, 41.9028, 6],
      spain: [-3.7038, 40.4168, 6]
    };
    if (key === 'taiwan' || key === '台湾' || key === '台灣') return [116.39723, 39.9075, 5.8];
    if (key === '中国') return [116.39723, 39.9075, 5.8];
    if (key === '日本') return [139.69171, 35.6895, 5.8];
    if (key === '美国') return [-77.0369, 38.9072, 5.5];
    if (key === '韩国' || key === '韓國') return [126.978, 37.5665, 6];
    // Prefer full capitals table when available
    if (window.__INSTANT_CAPITALS) {
      var hitFull = window.__INSTANT_CAPITALS[country] || window.__INSTANT_CAPITALS[key];
      // case-insensitive scan
      if (!hitFull) {
        for (var k in window.__INSTANT_CAPITALS) {
          if (String(k).toLowerCase() === key) { hitFull = window.__INSTANT_CAPITALS[k]; break; }
        }
      }
      if (hitFull) return hitFull;
    }
    return table[key] || null;
  }
  function shouldOpenMap() {
    var c = qs('country');
    var map = qs('map');
    var explore = qs('explore');
    return !!(c && String(c).trim()) || map === '1' || map === 'true' || explore === '1' || explore === 'true';
  }
  function showHero() {
    document.documentElement.removeAttribute('data-map-open');
    if (document.body) document.body.removeAttribute('data-map-open');
    var hero = document.querySelector('.home-hero-card');
    if (hero) hero.style.display = '';
  }
  function hideHero() {
    document.documentElement.setAttribute('data-map-open', '1');
    if (document.body) document.body.setAttribute('data-map-open', '1');
    var hero = document.querySelector('.home-hero-card');
    if (hero) hero.style.display = 'none';
  }
  function ensureMap() {
    var el = document.getElementById('map');
    if (!el || !window.maplibregl) return false;
    // Never mount before Leptos hydration is allowed: inserting MapLibre DOM
    // into #map before hydration corrupts the cursor and breaks all on:click.
    if (!window.__instantMapMountAllowed) return false;

    // Prefer existing map (WASM or previous boot)
    try {
      if (window.__instantSpaceMaps && window.__instantSpaceMaps.get) {
        var existing = window.__instantSpaceMaps.get('map');
        if (existing && existing.map) {
          try { existing.map.resize(); } catch (e) {}
          el.dataset.mapMounted = 'true';
          return true;
        }
      }
    } catch (e) {}

    if (el.__instantBootMap) {
      try { el.__instantBootMap.resize(); } catch (e) {}
      return true;
    }

    try {
      var style = ofmBase() + '/styles/liberty';
      var center = [20, 20];
      var zoom = 1.6;
      var c = qs('country');
      if (c) {
        var hit = countryCenter(c);
        if (hit) { center = [hit[0], hit[1]]; zoom = hit[2]; }
      } else if (!shouldOpenMap()) {
        // first paint under hero: still mount so closing hero shows real map
        center = [104, 35];
        zoom = 2.4;
      }
      var map = new maplibregl.Map({
        container: 'map',
        style: style,
        center: center,
        zoom: zoom,
        minZoom: 1.2,
        maxZoom: 18,
        attributionControl: false,
        dragPan: true,
        scrollZoom: true,
        touchZoomRotate: true
      });
      try {
        map.addControl(new maplibregl.AttributionControl({ compact: true }), 'bottom-right');
      } catch (e) {}
      el.dataset.mapMounted = 'true';
      el.__instantBootMap = map;
      window.__instantBootMap = map;
      window.__instantSpaceMaps = window.__instantSpaceMaps || new Map();
      window.__instantSpaceMaps.set('map', {
        map: map,
        markers: [],
        picker: null,
        styleKey: 'roadmap',
        projectionKey: '2d'
      });
      var resize = function () { try { map.resize(); map.triggerRepaint && map.triggerRepaint(); } catch (e) {} };
      [0, 40, 120, 280, 600, 1200, 2000, 3200].forEach(function (ms) { setTimeout(resize, ms); });
      map.once('load', resize);
      map.once('idle', resize);
      window.addEventListener('resize', resize);
      console.info('[instant-space] map boot mounted');
      return true;
    } catch (err) {
      console.error('[instant-space] boot map failed', err);
      return false;
    }
  }
  function boot() {
    if (shouldOpenMap()) hideHero();
    document.addEventListener('click', function (ev) {
      var t = ev.target;
      if (!t || !t.closest) return;
      if (t.closest('.home-hero-close') || t.closest('[data-open-map]') || (t.closest('a') && /[?&]map=1/.test(t.closest('a').getAttribute('href') || ''))) {
        hideHero();
        ensureMap();
      }
    }, true);
    // Auto-close header menus: <details> doesn't close on outside-click or after
    // activating an item natively, so we do it. Keep the menu's own summary
    // (native toggle) and in-menu adjusters (zoom/style/projection/language) open.
    document.addEventListener('click', function (ev) {
      var t = ev.target;
      if (!t || !t.closest) return;
      var menus = document.querySelectorAll('details.nav-menu[open], details.user-menu[open]');
      if (!menus.length) return;
      var sum = t.closest('summary');
      var keep = t.closest('.language-switcher, .map-style-switcher, .map-projection-switcher, .map-zoom-controls');
      menus.forEach(function (det) {
        if (sum && det.contains(sum)) return;
        if (keep && det.contains(t)) return;
        det.open = false;
      });
    }, true);
    startMapMount();
  }
  function startMapMount() {
    // Wait for hydration (WASM) to finish before mounting MapLibre into #map.
    // Fallback: if WASM never signals within 4s, allow mount anyway so the map
    // still works when hydration is unavailable.
    function allow() {
      if (window.__instantMapMountAllowed) return;
      window.__instantMapMountAllowed = true;
      var n = 0;
      var timer = setInterval(function () {
        n += 1;
        if (ensureMap() || n > 100) clearInterval(timer);
      }, 80);
    }
    if (window.__instantSpaceHydrated) { allow(); return; }
    window.addEventListener('instant-space-hydrated', allow, { once: true });
    setTimeout(allow, 4000);
  }
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', boot);
  else boot();
})();
