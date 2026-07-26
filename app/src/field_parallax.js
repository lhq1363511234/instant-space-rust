// Homepage field strip: photographs drift at their own depth as the page moves.
//
// The rest of the homepage motion is pure CSS scroll timelines. This one needs
// script because each plate carries a different depth and because the drift has
// to keep going for a beat after the finger leaves the screen — a scroll
// timeline snaps to the scroll position exactly, which reads mechanical.
//
// Contract with the markup (app/src/pages/home.rs):
//   [data-parallax-strip]  the <ul>
//   [data-depth]           per-plate multiplier, larger drifts further
//
// Everything is opt-out safe: no script, no reduced motion, or no
// IntersectionObserver leaves the strip exactly as CSS laid it out.
(function () {
  "use strict";
  if (typeof window === "undefined") return;

  var strips = [];
  var running = false;
  var motionOff = false;

  function collect() {
    strips = [];
    var nodes = document.querySelectorAll("[data-parallax-strip]");
    for (var i = 0; i < nodes.length; i++) {
      var plates = nodes[i].querySelectorAll("[data-depth]");
      if (!plates.length) continue;
      var entry = { root: nodes[i], plates: [], visible: false, y: 0, target: 0 };
      for (var j = 0; j < plates.length; j++) {
        entry.plates.push({ el: plates[j], depth: parseFloat(plates[j].getAttribute("data-depth")) || 0 });
      }
      strips.push(entry);
    }
    return strips.length > 0;
  }

  function measure() {
    var vh = window.innerHeight || 1;
    for (var i = 0; i < strips.length; i++) {
      var s = strips[i];
      if (!s.visible) continue;
      var box = s.root.getBoundingClientRect();
      // -1 when the strip is entering from below, +1 once it has left above.
      var progress = (vh - box.top) / (vh + box.height);
      s.target = (progress * 2 - 1) * 78;
    }
  }

  function frame() {
    var moved = false;
    for (var i = 0; i < strips.length; i++) {
      var s = strips[i];
      if (!s.visible) continue;
      // Trailing ease: the plates keep settling after the scroll stops.
      s.y += (s.target - s.y) * 0.085;
      if (Math.abs(s.target - s.y) > 0.05) moved = true;
      for (var j = 0; j < s.plates.length; j++) {
        var p = s.plates[j];
        p.el.style.transform = "translate3d(0," + (s.y * p.depth).toFixed(2) + "px,0)";
      }
    }
    if (moved) {
      window.requestAnimationFrame(frame);
    } else {
      running = false;
    }
  }

  function kick() {
    if (motionOff) return;
    measure();
    if (running) return;
    running = true;
    window.requestAnimationFrame(frame);
  }

  function clear() {
    for (var i = 0; i < strips.length; i++) {
      var s = strips[i];
      s.y = 0;
      s.target = 0;
      for (var j = 0; j < s.plates.length; j++) s.plates[j].el.style.transform = "";
    }
  }

  function start() {
    if (!collect()) return;

    var query = window.matchMedia ? window.matchMedia("(prefers-reduced-motion: reduce)") : null;
    function syncMotion() {
      motionOff = !!(query && query.matches);
      if (motionOff) clear();
      else kick();
    }
    if (query) {
      if (query.addEventListener) query.addEventListener("change", syncMotion);
      else if (query.addListener) query.addListener(syncMotion);
    }
    motionOff = !!(query && query.matches);
    if (motionOff) return;

    if ("IntersectionObserver" in window) {
      var io = new IntersectionObserver(
        function (entries) {
          for (var i = 0; i < entries.length; i++) {
            for (var j = 0; j < strips.length; j++) {
              if (strips[j].root === entries[i].target) strips[j].visible = entries[i].isIntersecting;
            }
          }
          kick();
        },
        { rootMargin: "120px 0px" }
      );
      for (var i = 0; i < strips.length; i++) io.observe(strips[i].root);
    } else {
      for (var k = 0; k < strips.length; k++) strips[k].visible = true;
    }

    window.addEventListener("scroll", kick, { passive: true });
    window.addEventListener("resize", kick, { passive: true });
    kick();
  }

  // The homepage is hydrated by WASM, so the strip does not exist at parse
  // time on a client-side navigation. Re-attach whenever the route changes.
  function boot() {
    start();
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot);
  } else {
    boot();
  }
  window.addEventListener("popstate", function () {
    window.setTimeout(boot, 60);
  });
  document.addEventListener("click", function () {
    window.setTimeout(boot, 220);
  }, true);
})();
