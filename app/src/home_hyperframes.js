// Homepage examples carousel. It communicates breadth of real-world uses,
// while keeping the browser's native document scrolling intact.
(function () {
  "use strict";
  if (typeof window === "undefined") return;

  var reduceMotion = window.matchMedia &&
    window.matchMedia("(prefers-reduced-motion: reduce)");

  function bindReveals() {
    var sections = document.querySelectorAll(
      ".inspace-home .home-examples, .inspace-home .home-featured-spaces, .inspace-home .home-featured-stories, .inspace-home .home-journey, .inspace-home .home-host-call"
    );
    if (!sections.length) return;

    for (var i = 0; i < sections.length; i++) {
      if (sections[i].getAttribute("data-home-reveal-bound") === "true") continue;
      sections[i].setAttribute("data-home-reveal-bound", "true");
      sections[i].classList.add("home-reveal");
    }

    if ((reduceMotion && reduceMotion.matches) || !("IntersectionObserver" in window)) {
      for (var j = 0; j < sections.length; j++) sections[j].classList.add("is-revealed");
      return;
    }

    var observer = new IntersectionObserver(function (entries, currentObserver) {
      for (var k = 0; k < entries.length; k++) {
        if (!entries[k].isIntersecting) continue;
        entries[k].target.classList.add("is-revealed");
        currentObserver.unobserve(entries[k].target);
      }
    }, { rootMargin: "0px 0px -8%", threshold: 0.08 });

    for (var n = 0; n < sections.length; n++) {
      if (!sections[n].classList.contains("is-revealed")) observer.observe(sections[n]);
    }
  }

  function bindFeaturedSpaces() {
    var roots = document.querySelectorAll("[data-home-space-carousel]");

    for (var r = 0; r < roots.length; r++) {
      (function (root) {
        if (root.getAttribute("data-home-space-bound") === "true") return;

        var stage = root.querySelector("[data-home-space-stage]");
        var slides = root.querySelectorAll("[data-home-space-slide]");
        var dots = root.querySelectorAll("[data-home-space-dot]");
        var previous = root.querySelector("[data-home-space-prev]");
        var next = root.querySelector("[data-home-space-next]");
        var current = root.querySelector("[data-home-space-current]");
        if (!stage || !slides.length) return;

        var active = 0;
        var timer = 0;
        var paused = false;
        var inView = true;
        var pointerId = null;
        var pointerStartX = 0;
        var pointerLastX = 0;
        var pointerMoved = false;
        var suppressClickUntil = 0;

        function signedDistance(index) {
          var distance = index - active;
          var half = slides.length / 2;
          if (distance > half) distance -= slides.length;
          if (distance < -half) distance += slides.length;
          return distance;
        }

        function layout() {
          var width = stage.clientWidth || root.clientWidth || 960;
          var compact = width < 620;
          var step = compact
            ? Math.max(205, Math.min(width * 0.66, 265))
            : Math.max(260, Math.min(width * 0.31, 355));

          for (var i = 0; i < slides.length; i++) {
            var distance = signedDistance(i);
            var magnitude = Math.abs(distance);
            var direction = distance < 0 ? -1 : 1;
            var x = 0;
            var z = -180;
            var scale = 0.62;
            var opacity = 0;
            var turn = 0;

            if (magnitude === 0) {
              z = 90;
              scale = 1;
              opacity = 1;
            } else if (magnitude === 1) {
              x = direction * step;
              z = -20;
              scale = compact ? 0.78 : 0.84;
              opacity = compact ? 0.58 : 0.78;
              turn = direction * -7;
            } else if (magnitude === 2 && !compact) {
              x = direction * step * 1.7;
              z = -130;
              scale = 0.68;
              opacity = 0.2;
              turn = direction * -10;
            }

            var selected = magnitude === 0;
            var visible = magnitude <= 1 || (magnitude === 2 && !compact);
            slides[i].style.setProperty("--space-x", x + "px");
            slides[i].style.setProperty("--space-z", z + "px");
            slides[i].style.setProperty("--space-scale", scale);
            slides[i].style.setProperty("--space-opacity", opacity);
            slides[i].style.setProperty("--space-turn", turn + "deg");
            slides[i].style.zIndex = selected ? "12" : String(Math.max(1, 8 - magnitude));
            slides[i].style.pointerEvents = visible ? "auto" : "none";
            slides[i].classList.toggle("is-active", selected);
            slides[i].setAttribute("aria-hidden", visible ? "false" : "true");

            var links = slides[i].querySelectorAll("a, button, input, select, textarea");
            for (var l = 0; l < links.length; l++) links[l].tabIndex = selected ? 0 : -1;
            if (dots[i]) {
              dots[i].setAttribute("aria-current", selected ? "true" : "false");
              dots[i].tabIndex = selected ? 0 : -1;
            }
          }
          if (current) current.textContent = String(active + 1).padStart(2, "0");
        }

        function show(index, focusActive) {
          active = (index + slides.length) % slides.length;
          layout();
          if (focusActive) {
            var link = slides[active].querySelector("a, button");
            if (link) link.focus({ preventScroll: true });
          }
        }

        function stop() {
          if (!timer) return;
          window.clearInterval(timer);
          timer = 0;
        }

        function start() {
          stop();
          if ((reduceMotion && reduceMotion.matches) || paused || !inView || document.hidden) return;
          timer = window.setInterval(function () {
            if (!root.isConnected) {
              stop();
              return;
            }
            show(active + 1, false);
          }, 5200);
        }

        function finishPointer(event, cancelled) {
          if (pointerId === null || (event.pointerId != null && event.pointerId !== pointerId)) return;
          var delta = pointerLastX - pointerStartX;
          if (!cancelled && Math.abs(delta) > 44) show(active + (delta < 0 ? 1 : -1), false);
          if (pointerMoved) suppressClickUntil = Date.now() + 260;
          pointerId = null;
          pointerMoved = false;
          root.classList.remove("is-dragging");
          root.style.removeProperty("--space-drag-shift");
          start();
        }

        if (previous) previous.addEventListener("click", function () {
          show(active - 1, false);
          start();
        });
        if (next) next.addEventListener("click", function () {
          show(active + 1, false);
          start();
        });
        for (var d = 0; d < dots.length; d++) {
          (function (index) {
            dots[index].addEventListener("click", function () {
              show(index, false);
              start();
            });
          })(d);
        }

        stage.addEventListener("keydown", function (event) {
          if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return;
          event.preventDefault();
          show(active + (event.key === "ArrowRight" ? 1 : -1), true);
          start();
        });
        stage.addEventListener("click", function (event) {
          var slide = event.target.closest("[data-home-space-slide]");
          if (!slide || !stage.contains(slide)) return;
          var index = Number(slide.getAttribute("data-home-space-index"));
          if (Date.now() < suppressClickUntil) {
            event.preventDefault();
            event.stopImmediatePropagation();
            return;
          }
          if (index !== active) {
            event.preventDefault();
            event.stopImmediatePropagation();
            show(index, false);
            start();
          }
        }, true);
        stage.addEventListener("pointerdown", function (event) {
          if (event.button != null && event.button !== 0) return;
          pointerId = event.pointerId;
          pointerStartX = event.clientX;
          pointerLastX = event.clientX;
          pointerMoved = false;
          paused = true;
          stop();
          // Do not steal a normal link/button click. Pointer capture is only
          // needed when the gesture starts on non-interactive carousel space.
          if (!event.target.closest("a, button") && stage.setPointerCapture) {
            stage.setPointerCapture(event.pointerId);
          }
        });
        stage.addEventListener("pointermove", function (event) {
          if (pointerId === null || event.pointerId !== pointerId) return;
          pointerLastX = event.clientX;
          var delta = pointerLastX - pointerStartX;
          if (Math.abs(delta) > 7) {
            pointerMoved = true;
            root.classList.add("is-dragging");
            root.style.setProperty("--space-drag-shift", Math.max(-96, Math.min(96, delta * 0.32)) + "px");
          }
        });
        stage.addEventListener("pointerup", function (event) {
          paused = false;
          finishPointer(event, false);
        });
        stage.addEventListener("pointercancel", function (event) {
          paused = false;
          finishPointer(event, true);
        });
        stage.addEventListener("focusin", function () { paused = true; stop(); });
        stage.addEventListener("focusout", function () {
          window.setTimeout(function () {
            if (!root.contains(document.activeElement)) {
              paused = false;
              start();
            }
          }, 0);
        });
        window.addEventListener("resize", layout, { passive: true });
        document.addEventListener("visibilitychange", function () {
          if (document.hidden) stop();
          else start();
        });
        if (reduceMotion) {
          var handleMotionPreference = function () {
            if (reduceMotion.matches) stop();
            else start();
          };
          if (reduceMotion.addEventListener) reduceMotion.addEventListener("change", handleMotionPreference);
          else if (reduceMotion.addListener) reduceMotion.addListener(handleMotionPreference);
        }

        if ("IntersectionObserver" in window) {
          var visibilityObserver = new IntersectionObserver(function (entries) {
            inView = !!entries[0] && entries[0].isIntersecting;
            if (inView) start();
            else stop();
          }, { threshold: 0.18 });
          visibilityObserver.observe(root);
        }

        root.setAttribute("data-home-space-bound", "true");
        root.classList.add("is-enhanced");
        show(0, false);
        start();
      })(roots[r]);
    }
  }

  function boot() {
    bindReveals();
    bindFeaturedSpaces();
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
  if ("MutationObserver" in window) {
    var queuedBoot = 0;
    var observer = new MutationObserver(function () {
      if (queuedBoot) return;
      queuedBoot = window.setTimeout(function () {
        queuedBoot = 0;
        boot();
      }, 120);
    });
    observer.observe(document.documentElement, { childList: true, subtree: true });
  }
})();
