// Homepage examples carousel. It communicates breadth of real-world uses,
// while keeping the browser's native document scrolling intact.
(function () {
  "use strict";
  if (typeof window === "undefined") return;

  var reduceMotion = window.matchMedia &&
    window.matchMedia("(prefers-reduced-motion: reduce)");

  function boot() {
    var roots = document.querySelectorAll("[data-home-carousel]");

    for (var r = 0; r < roots.length; r++) {
      (function (root) {
        if (root.getAttribute("data-home-carousel-bound") === "true") return;

        var slides = root.querySelectorAll("[data-home-carousel-slide]");
        var dots = root.querySelectorAll("[data-home-carousel-dot]");
        var previous = root.querySelector("[data-home-carousel-prev]");
        var next = root.querySelector("[data-home-carousel-next]");
        if (!slides.length) return;

        var active = 0;
        var timer = 0;
        var paused = false;

        function show(index) {
          active = (index + slides.length) % slides.length;
          for (var i = 0; i < slides.length; i++) {
            var selected = i === active;
            slides[i].classList.toggle("is-active", selected);
            slides[i].setAttribute("aria-hidden", selected ? "false" : "true");
            slides[i].toggleAttribute("inert", !selected);
            var links = slides[i].querySelectorAll("a, button, input, select, textarea");
            for (var l = 0; l < links.length; l++) {
              links[l].tabIndex = selected ? 0 : -1;
            }
            if (dots[i]) {
              dots[i].setAttribute("aria-selected", selected ? "true" : "false");
              dots[i].tabIndex = selected ? 0 : -1;
            }
          }
        }

        function stop() {
          if (timer) {
            window.clearInterval(timer);
            timer = 0;
          }
        }

        function start() {
          stop();
          if ((reduceMotion && reduceMotion.matches) || paused) return;
          timer = window.setInterval(function () {
            if (!root.isConnected) {
              stop();
              return;
            }
            show(active + 1);
          }, 6000);
        }

        if (previous) previous.addEventListener("click", function () {
          show(active - 1);
          start();
        });
        if (next) next.addEventListener("click", function () {
          show(active + 1);
          start();
        });
        for (var i = 0; i < dots.length; i++) {
          (function (index) {
            dots[index].addEventListener("click", function () {
              show(index);
              start();
            });
          })(i);
        }

        root.addEventListener("mouseenter", function () { paused = true; stop(); });
        root.addEventListener("mouseleave", function () { paused = false; start(); });
        root.addEventListener("focusin", function () { paused = true; stop(); });
        root.addEventListener("focusout", function () {
          window.setTimeout(function () {
            if (!root.contains(document.activeElement)) {
              paused = false;
              start();
            }
          }, 0);
        });

        root.setAttribute("data-home-carousel-bound", "true");
        show(0);
        start();
      })(roots[r]);
    }
  }

  if (reduceMotion) {
    var rebind = function () {
      var roots = document.querySelectorAll("[data-home-carousel]");
      for (var i = 0; i < roots.length; i++) {
        roots[i].removeAttribute("data-home-carousel-bound");
      }
      boot();
    };
    if (reduceMotion.addEventListener) reduceMotion.addEventListener("change", rebind);
    else if (reduceMotion.addListener) reduceMotion.addListener(rebind);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot, { once: true });
  } else {
    boot();
  }
  window.addEventListener("popstate", function () { window.setTimeout(boot, 80); });
  document.addEventListener("click", function () { window.setTimeout(boot, 160); }, true);
})();
