/* IVEC SR 2 — PWA (portal build) */
(function () {
  "use strict";

  // ---- search ------------------------------------------------------------
  // Anything that looks like a host is opened directly; anything else becomes a
  // Google search. Mirrors src-tauri/src/lib.rs `parse_url` closely enough for
  // the browser build (LAN hosts -> http, everything else -> https).
  var LAN = /^(localhost|127\.\d+\.\d+\.\d+|10\.\d+\.\d+\.\d+|192\.168\.\d+\.\d+|172\.(1[6-9]|2\d|3[01])\.\d+\.\d+|169\.254\.\d+\.\d+|[^/]+\.(local|lan))(:|\/|$)/i;
  var IPV4 = /^\d{1,3}(\.\d{1,3}){3}(:|\/|$)/;
  var HOSTNAME = /^[a-z0-9-]+(\.[a-z0-9-]+)+(:|\/|$)/i;

  function parse(raw) {
    var value = (raw || "").trim();
    if (!value) return "https://www.google.com/";
    if (/^[a-z][a-z0-9+.-]*:\/\//i.test(value)) return value;

    var head = value.split("/")[0];
    var looksHost = IPV4.test(value) || HOSTNAME.test(value);
    if (looksHost && LAN.test(value)) {
      var scheme = /:\d+\//.test(value) || /:\d+$/.test(value) ? "http://" : "http://";
      return scheme + value;
    }
    if (looksHost) return "https://" + value;
    return "https://www.google.com/search?q=" + encodeURIComponent(value);
  }

  var form = document.getElementById("search");
  var input = document.getElementById("q");
  if (form && input) {
    form.addEventListener("submit", function (event) {
      event.preventDefault();
      window.location.href = parse(input.value);
    });
  }

  // App-shortcut entries in the manifest land here as ?go=<url>.
  var jump = new URLSearchParams(window.location.search).get("go");
  if (jump) window.location.replace(jump);

  // ---- installed IVEC apps (optional) ------------------------------------
  // The native app keeps its store in the app data directory, which a PWA
  // cannot read. This section only appears when an app manifest was published
  // alongside the site (see docs/PWA.md).
  fetch("./apps.json", { cache: "no-store" })
    .then(function (response) {
      if (!response.ok) throw new Error("no apps");
      return response.json();
    })
    .then(function (apps) {
      if (!Array.isArray(apps) || !apps.length) return;
      var section = document.getElementById("apps");
      var grid = document.getElementById("apps-grid");
      apps.forEach(function (app) {
        var link = document.createElement("a");
        link.className = "app-card";
        link.href = app.path || "#";

        var badge = document.createElement("span");
        badge.className = "app-badge";
        badge.style.background = app.accent || "#2f7dff";
        badge.textContent = (app.name || "?").slice(0, 1).toUpperCase();

        var label = document.createElement("span");
        label.textContent = app.name || "App";

        link.appendChild(badge);
        link.appendChild(label);
        grid.appendChild(link);
      });
      if (section) section.hidden = false;
    })
    .catch(function () {
      /* no apps.json — normal */
    });

  // ---- install hint ------------------------------------------------------
  var hint = document.getElementById("installhint");
  var ua = navigator.userAgent;
  var isIOS = /iPhone|iPad|iPod/i.test(ua);
  var standalone =
    window.matchMedia("(display-mode: standalone)").matches || navigator.standalone === true;

  if (hint) {
    if (standalone) {
      hint.textContent = "กำลังใช้งานแบบแอปติดตั้งแล้ว ✓";
    } else if (isIOS) {
      hint.textContent = "ติดตั้งบนหน้าจอโฮม: แตะปุ่ม Share ⤴ แล้วเลือก “เพิ่มไปยังหน้าจอโฮม”";
    } else {
      hint.textContent = "ติดตั้งบนหน้าจอโฮม: เมนู ⋮ → “ติดตั้งแอป” / “Add to Home screen”";
    }
  }

  // ---- service worker ----------------------------------------------------
  if ("serviceWorker" in navigator) {
    window.addEventListener("load", function () {
      navigator.serviceWorker.register("./sw.js").catch(function () {
        /* offline support is best-effort */
      });
    });
  }
})();
