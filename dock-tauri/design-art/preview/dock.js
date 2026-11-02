/* Dock 预览：放大联动、气泡、右键菜单、键盘导航。
   放大公式与 src/dock/magnify.ts 完全一致，便于逐帧比对。 */

(function () {
  "use strict";

  var root = document.documentElement;
  var dock = document.getElementById("dock");
  var tiles = Array.prototype.slice.call(dock.querySelectorAll(".tile"));
  var tooltip = document.getElementById("tooltip");
  var ctx = document.getElementById("ctx");

  var PEAK = 1.6;
  var FALLOFF = 0.45;
  var RADIUS_ICONS = 2.2;
  var TOOLTIP_DELAY = 500;

  var iconSize = 56;
  var iconGap = 9;
  var radiusPx = RADIUS_ICONS * (iconSize + iconGap);

  // 风格会改变间距与内边距（B 更松），所以换风格后必须重算放大半径与中心。
  function refreshMetrics() {
    var computed = getComputedStyle(dock);
    iconSize = parseFloat(computed.getPropertyValue("--icon-size")) || 56;
    iconGap = parseFloat(computed.getPropertyValue("--icon-gap")) || 9;
    radiusPx = RADIUS_ICONS * (iconSize + iconGap);
    measure();
  }

  var reducedMotion = false;
  var centers = [];
  var tooltipTimer = null;
  var tooltipTile = null;
  var menuTile = null;

  tooltip.style.bottom = "calc(100% + 28px)";

  /* ---------------- 放大 ---------------- */

  function scaleAt(distance) {
    var d = Math.max(0, distance);
    if (d >= radiusPx) return 1;
    var normalized = d / radiusPx;
    var weight = Math.exp(-Math.pow(normalized / FALLOFF, 4));
    return 1 + (PEAK - 1) * weight;
  }

  function measure() {
    centers = tiles.map(function (tile) {
      return tile.offsetLeft + tile.offsetWidth / 2;
    });
  }

  function resetScales() {
    tiles.forEach(function (tile) {
      tile.style.transform = "";
      tile.classList.remove("is-magnifying");
    });
  }

  function applyScales(pointerX) {
    tiles.forEach(function (tile, index) {
      var scale = scaleAt(Math.abs(centers[index] - pointerX));
      tile.style.transform = scale > 1.0005 ? "scale(" + scale.toFixed(4) + ")" : "";
      tile.classList.toggle("is-magnifying", scale > 1.02);
    });
  }

  function onMove(event) {
    if (reducedMotion) return;
    dock.classList.remove("is-idle");
    var rect = dock.getBoundingClientRect();
    applyScales(event.clientX - rect.left);
  }

  function onLeave() {
    dock.classList.add("is-idle");
    resetScales();
    hideTooltip();
  }

  dock.addEventListener("pointermove", onMove, { passive: true });
  dock.addEventListener("pointerleave", onLeave);

  /* ---------------- 气泡 ---------------- */

  function hideTooltip() {
    if (tooltipTimer !== null) {
      clearTimeout(tooltipTimer);
      tooltipTimer = null;
    }
    tooltip.hidden = true;
    tooltipTile = null;
  }

  function showTooltip(tile) {
    tooltip.textContent = tile.dataset.name || "";
    tooltip.style.left = tile.offsetLeft + tile.offsetWidth / 2 + "px";
    tooltip.hidden = false;
    tooltipTile = tile;
  }

  tiles.forEach(function (tile) {
    tile.addEventListener("pointerenter", function () {
      if (tooltipTimer !== null) clearTimeout(tooltipTimer);
      hideTooltip();
      tooltipTimer = window.setTimeout(function () {
        tooltipTimer = null;
        showTooltip(tile);
      }, TOOLTIP_DELAY);
    });

    tile.addEventListener("pointerleave", function () {
      if (tooltipTile === tile) hideTooltip();
    });

    tile.addEventListener("contextmenu", function (event) {
      event.preventDefault();
      hideTooltip();
      openMenu(tile, event.clientX, event.clientY);
    });

    tile.addEventListener("click", function () {
      hideTooltip();
    });
  });

  /* ---------------- 右键菜单 ---------------- */

  function openMenu(tile, clientX, clientY) {
    menuTile = tile;
    ctx.hidden = false;
    var rect = ctx.getBoundingClientRect();
    var left = Math.min(Math.max(8, clientX - rect.width / 2), window.innerWidth - rect.width - 8);
    var top = Math.max(8, clientY - rect.height - 14);
    ctx.style.left = left + "px";
    ctx.style.top = top + "px";
    var first = ctx.querySelector("button");
    if (first) first.focus({ preventScroll: true });
  }

  function closeMenu() {
    if (ctx.hidden) return;
    ctx.hidden = true;
    menuTile = null;
  }

  ctx.addEventListener("click", function (event) {
    var button = event.target.closest("button");
    if (button) closeMenu();
  });

  document.addEventListener("pointerdown", function (event) {
    if (!ctx.hidden && !ctx.contains(event.target)) closeMenu();
  });

  document.addEventListener("keydown", function (event) {
    if (event.key === "Escape") {
      closeMenu();
      hideTooltip();
    }
  });

  window.addEventListener("blur", closeMenu);
  window.addEventListener("resize", function () {
    closeMenu();
    measure();
  });

  /* ---------------- 键盘导航 ---------------- */

  var focusables = tiles;

  function setFocus(index) {
    focusables.forEach(function (tile, i) {
      tile.tabIndex = i === index ? 0 : -1;
    });
    focusables[index].focus({ preventScroll: true });
  }

  dock.addEventListener("keydown", function (event) {
    var current = focusables.indexOf(document.activeElement);
    if (current === -1) return;

    if (event.key === "ArrowRight") {
      event.preventDefault();
      setFocus((current + 1) % focusables.length);
    } else if (event.key === "ArrowLeft") {
      event.preventDefault();
      setFocus((current - 1 + focusables.length) % focusables.length);
    } else if (event.key === "Home") {
      event.preventDefault();
      setFocus(0);
    } else if (event.key === "End") {
      event.preventDefault();
      setFocus(focusables.length - 1);
    } else if (event.key === "ContextMenu" || (event.shiftKey && event.key === "F10")) {
      event.preventDefault();
      var rect = focusables[current].getBoundingClientRect();
      openMenu(focusables[current], rect.left + rect.width / 2, rect.top - 8);
    }
  });

  /* ---------------- 顶栏开关 ---------------- */

  document.querySelectorAll("[data-set]").forEach(function (button) {
    button.addEventListener("click", function () {
      var group = button.dataset.set;
      var value = button.dataset.value;

      document.querySelectorAll('[data-set="' + group + '"]').forEach(function (peer) {
        peer.classList.toggle("is-on", peer === button);
      });

      if (group === "theme") root.dataset.theme = value;
      if (group === "motion") {
        root.dataset.motion = value;
        reducedMotion = value === "reduced";
        if (reducedMotion) {
          resetScales();
          hideTooltip();
        }
      }
      if (group === "style") {
        root.dataset.style = value;
        resetScales();
        window.requestAnimationFrame(refreshMetrics);
      }
    });
  });

  var zones = document.querySelector('[data-toggle="zones"]');
  zones.addEventListener("click", function () {
    var on = root.dataset.zones !== "on";
    root.dataset.zones = on ? "on" : "off";
    zones.setAttribute("aria-pressed", String(on));
  });

  /* ---------------- init ---------------- */

  // 便于设计评审直接链接某个状态：
  // dock.html?theme=light&style=frost&motion=reduced&zones=on
  (function applyParams() {
    var params = new URLSearchParams(window.location.search);

    ["theme", "style", "motion"].forEach(function (group) {
      var value = params.get(group);
      if (!value) return;
      root.dataset[group] = value;
      document.querySelectorAll('[data-set="' + group + '"]').forEach(function (peer) {
        peer.classList.toggle("is-on", peer.dataset.value === value);
      });
      if (group === "motion") reducedMotion = value === "reduced";
    });

    if (params.get("zones") === "on") {
      root.dataset.zones = "on";
      zones.setAttribute("aria-pressed", "true");
    }

    // 主题包出图：dock.html?chrome=off&style=frost&theme=light
    if (params.get("chrome") === "off") root.dataset.chrome = "off";
  })();

  if (window.matchMedia && window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
    reducedMotion = true;
    root.dataset.motion = "reduced";
    document.querySelectorAll('[data-set="motion"]').forEach(function (peer) {
      peer.classList.toggle("is-on", peer.dataset.value === "reduced");
    });
  }

  refreshMetrics();
  window.addEventListener("load", refreshMetrics);
})();
