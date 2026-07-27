// MapLibre is intentionally lazy-loaded. The dedicated map workspace loads it
// immediately; create/manage-space pickers request it only when their modal is opened.
(function () {
  var MAP_PATHS = ['/map', '/inspace/map'];
  var PICKER_IDS = ['create-space-map', 'manage-space-map'];
  var loading = null;

  function isMapRoute() {
    return MAP_PATHS.indexOf(window.location.pathname) >= 0;
  }

  function hasMapSurface(root) {
    if (!root) return false;
    if (root.nodeType === 1 && root.id === 'map') return true;
    if (typeof root.querySelector === 'function' && root.querySelector('#map')) return true;
    return false;
  }

  function hasMapPicker(root) {
    if (!root) return false;
    if (root.nodeType === 1 && PICKER_IDS.indexOf(root.id) >= 0) return true;
    if (typeof root.querySelector !== 'function') return false;
    return PICKER_IDS.some(function (id) { return root.querySelector('#' + id); });
  }

  function loadScript(src) {
    return new Promise(function (resolve, reject) {
      var existing = document.querySelector('script[data-instant-maplibre]');
      if (existing) {
        if (window.maplibregl) resolve();
        else {
          existing.addEventListener('load', resolve, { once: true });
          existing.addEventListener('error', reject, { once: true });
        }
        return;
      }
      var node = document.createElement('script');
      node.src = src;
      node.dataset.instantMaplibre = 'true';
      node.onload = resolve;
      node.onerror = reject;
      document.head.appendChild(node);
    });
  }

  function loadStyle(href) {
    if (document.querySelector('link[data-instant-maplibre]')) return;
    var node = document.createElement('link');
    node.rel = 'stylesheet';
    node.href = href;
    node.dataset.instantMaplibre = 'true';
    document.head.appendChild(node);
  }

  function assetBase() {
    return location.pathname.indexOf('/inspace/') === 0 || location.pathname === '/inspace' ? '/inspace' : '';
  }

  function requestMapLibre() {
    if (window.maplibregl) return Promise.resolve();
    if (loading) return loading;
    var base = assetBase();
    loadStyle(base + '/vendor/maplibre-gl/maplibre-gl.css');
    loading = loadScript(base + '/vendor/maplibre-gl/maplibre-gl.js').then(function () {
      window.__instantMapMountAllowed = true;
      window.dispatchEvent(new Event('instant-space-hydrated'));
    }).catch(function (err) {
      loading = null;
      console.error('[instant-space] map library failed to load', err);
      throw err;
    });
    return loading;
  }

  window.__instantLoadMapLibre = requestMapLibre;

  function start() {
    if (isMapRoute() || hasMapSurface(document) || hasMapPicker(document)) requestMapLibre();

    var observer = new MutationObserver(function (mutations) {
      for (var i = 0; i < mutations.length; i += 1) {
        for (var j = 0; j < mutations[i].addedNodes.length; j += 1) {
          if (hasMapSurface(mutations[i].addedNodes[j]) || hasMapPicker(mutations[i].addedNodes[j])) {
            requestMapLibre();
            return;
          }
        }
      }
    });
    observer.observe(document.documentElement, { childList: true, subtree: true });
  }

  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', start);
  else start();
})();
