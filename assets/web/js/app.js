/* WAS admin glue — htmx v4 compatibility + CSRF + theme persistence.
 * uikit behaviors stay pristine; everything app-specific lives here. */
(function () {
  "use strict";

  var CSRF_RE = /<meta name="csrf-token" content="([^"]+)"/;

  function currentCsrf() {
    // Prefer live meta (fresh after swaps), fall back to the initial page value
    var meta = document.querySelector('meta[name="csrf-token"]');
    return meta ? meta.getAttribute("content") : "";
  }

  function onReady(fn) {
    if (document.readyState !== "loading") fn();
    else document.addEventListener("DOMContentLoaded", fn);
  }

  onReady(function () {
    // 1) CSRF: stamp every htmx request with the session-derived token.
    document.addEventListener("htmx:config:request", function (evt) {
      var d = evt.detail || {};
      var headers =
        (d.ctx && d.ctx.request && d.ctx.request.headers) || d.headers;
      if (headers) headers["X-CSRF-Token"] = currentCsrf();
    });

    // 2) htmx v4 event compat for uikit behaviors: it listens on
    //    'htmx:afterSettle'; re-broadcast the v4 name under the legacy one.
    document.addEventListener("htmx:after:settle", function (e) {
      document.dispatchEvent(
        new CustomEvent("htmx:afterSettle", { detail: e.detail })
      );
    });
  });

  // 3) Theme persistence — uikit's switch flips data-theme; remember it and
  //    restore before first paint on later loads.
  try {
    var saved = localStorage.getItem("was-theme");
    if (saved === "dark" || saved === "light") {
      document.documentElement.dataset.theme = saved;
    }
    document.addEventListener("change", function (e) {
      var t = e.target;
      if (
        t instanceof HTMLInputElement &&
        t.hasAttribute("data-dt-theme-switch")
      ) {
        localStorage.setItem(
          "was-theme",
          t.checked ? "dark" : "light"
        );
      }
    });
  } catch (_) {
    /* private mode etc. — theme just won't persist */
  }
})();
