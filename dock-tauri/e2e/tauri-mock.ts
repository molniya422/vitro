/**
 * A browser-side stand-in for the Tauri v2 IPC bridge.
 *
 * Installed with `page.addInitScript(TAURI_MOCK)` before the app loads. It
 * mirrors the shape the installed `@tauri-apps/api` 2.12.4 actually uses:
 *
 * - `invoke(cmd, args, options)` -> `window.__TAURI_INTERNALS__.invoke(...)`,
 *   which resolves with the raw Rust command result (no `{ type: "Ok" }`).
 * - `listen(event, handler)` calls `transformCallback(handler)` to obtain a
 *   numeric id, then invokes `plugin:event|listen` with that id.
 * - `unlisten` calls `window.__TAURI_EVENT_PLUGIN_INTERNALS__.unregisterListener`.
 */
export const TAURI_MOCK = `
(() => {
  "use strict";

  const DOCK_ITEMS = [
    { id: "idle-app", kind: "app", name: "Idle App", icon: null, running: false, minimized: 0, disabledReason: null },
    { id: "running-app", kind: "app", name: "Running App", icon: null, running: true, minimized: 0, disabledReason: null },
    { id: "minimized-app", kind: "app", name: "Minimized App", icon: null, running: true, minimized: 2, disabledReason: null },
    { id: "broken-app", kind: "app", name: "Broken App", icon: null, running: false, minimized: 0, disabledReason: "Missing target" },
    { id: "mail-app", kind: "app", name: "Mail", icon: null, running: false, minimized: 0, disabledReason: null },
    { id: "browser-app", kind: "app", name: "Browser", icon: null, running: false, minimized: 0, disabledReason: null },
    { id: "notes-app", kind: "app", name: "Notes", icon: null, running: false, minimized: 0, disabledReason: null },
    { id: "media-app", kind: "app", name: "Media Player", icon: null, running: false, minimized: 0, disabledReason: null },
    { id: "terminal-app", kind: "app", name: "Terminal", icon: null, running: false, minimized: 0, disabledReason: null },
    { id: "files-app", kind: "app", name: "Files", icon: null, running: false, minimized: 0, disabledReason: null }
  ];

  const APPEARANCE = {
    iconSize: 48,
    iconGap: 8,
    paddingH: 16,
    paddingV: 10,
    pillRadius: 34,
    tint: "#161a26",
    fillOpacity: 0.62,
    iconRadius: 22,
    reflection: true,
    reflectionStrength: 0.25,
    reflectionSpace: 40,
    magnify: true,
    magnifyPeak: 1.6,
    magnifyRadius: 123.2,
    reducedMotion: false,
    profile: "vitro",
    tooltips: true,
    tooltipDelayMs: 500,
    showRunningDots: true,
    showBadges: true,
    showLaunchpadEntry: true,
    showMonitorEntry: true,
    showShelfEntry: true,
    leadingTiles: 4
  };

  const DOCK_STATE = {
    items: DOCK_ITEMS,
    edge: "bottom",
    appearance: APPEARANCE
  };

  const CATALOG = [
    { id: "cat-alpha", kind: "app", name: "Alpha Player", icon: null, running: false, minimized: 0, disabledReason: null },
    { id: "cat-bravo", kind: "app", name: "Bravo Editor", icon: null, running: false, minimized: 0, disabledReason: null },
    { id: "cat-charlie", kind: "app", name: "Charlie Notes", icon: null, running: false, minimized: 0, disabledReason: null },
    { id: "cat-delta", kind: "app", name: "Delta Mail", icon: null, running: false, minimized: 0, disabledReason: null },
    { id: "cat-echo", kind: "app", name: "Echo Browser", icon: null, running: false, minimized: 0, disabledReason: null },
    { id: "cat-foxtrot", kind: "app", name: "Foxtrot Files", icon: null, running: false, minimized: 0, disabledReason: null }
  ];

  const SETTINGS = {
    iconSize: 48,
    iconGap: 8,
    paddingH: 16,
    paddingV: 10,
    pillRadius: 34,
    tint: "#161a26",
    fillOpacity: 0.62,
    iconRadius: 22,
    reflection: true,
    reflectionStrength: 0.25,
    magnify: true,
    magnifyPeak: 1.6,
    magnifyRadiusIcons: 2.2,
    profile: "vitro",
    reducedMotion: false,
    tooltips: true,
    tooltipDelayMs: 500,
    showRunningDots: true,
    showBadges: true,
    autohide: false,
    showDelayMs: 300,
    hideDelayMs: 500,
    yieldFullscreen: true,
    showLaunchpadEntry: true,
    showMonitorEntry: true,
    showShelfEntry: true
  };

  function syncAppearance() {
    const a = DOCK_STATE.appearance;
    a.iconSize = SETTINGS.iconSize;
    a.iconGap = SETTINGS.iconGap;
    a.paddingH = SETTINGS.paddingH;
    a.paddingV = SETTINGS.paddingV;
    a.pillRadius = SETTINGS.pillRadius;
    a.tint = SETTINGS.tint;
    a.fillOpacity = SETTINGS.fillOpacity;
    a.iconRadius = SETTINGS.iconRadius;
    a.reflection = SETTINGS.reflection;
    a.reflectionStrength = SETTINGS.reflectionStrength;
    a.reflectionSpace = SETTINGS.reflection
      ? Math.round((SETTINGS.iconSize * 5) / 6)
      : 0;
    a.magnify = SETTINGS.magnify;
    a.magnifyPeak = SETTINGS.magnifyPeak;
    a.magnifyRadius =
      Math.round(
        SETTINGS.magnifyRadiusIcons * (SETTINGS.iconSize + SETTINGS.iconGap) * 100
      ) / 100;
    a.reducedMotion = SETTINGS.reducedMotion;
    a.profile = SETTINGS.profile;
    a.tooltips = SETTINGS.tooltips;
    a.tooltipDelayMs = SETTINGS.tooltipDelayMs;
    a.showRunningDots = SETTINGS.showRunningDots;
    a.showBadges = SETTINGS.showBadges;
    a.showLaunchpadEntry = SETTINGS.showLaunchpadEntry;
    a.showMonitorEntry = SETTINGS.showMonitorEntry;
    a.showShelfEntry = SETTINGS.showShelfEntry;
    a.leadingTiles =
      1 +
      (SETTINGS.showLaunchpadEntry ? 1 : 0) +
      (SETTINGS.showMonitorEntry ? 1 : 0) +
      (SETTINGS.showShelfEntry ? 1 : 0);
  }

  let SHELF = [];

  function shelfItem(path) {
    // Split on a backslash built from its code point: this source lives inside
    // a template literal, so a literal escape would collapse before it runs.
    const parts = String(path)
      .split(String.fromCharCode(92))
      .join("/")
      .split("/")
      .filter(Boolean);
    const name = parts.pop() || String(path);
    return { id: path, name: name, path: path, isDir: false, missing: false, icon: null };
  }

  const MONITOR_SAMPLE = {
    cpu: 42.5,
    memUsed: 6 * 1024 * 1024 * 1024,
    memTotal: 16 * 1024 * 1024 * 1024,
    diskUsed: 100 * 1024 * 1024 * 1024,
    diskTotal: 500 * 1024 * 1024 * 1024,
    netRx: 1234567,
    netTx: 7654321
  };

  const calls = [];
  const callbacks = new Map();
  const listeners = new Map();
  const failures = new Set();
  let nextCallbackId = 1;
  let nextEventId = 1;

  function clone(value) {
    return JSON.parse(JSON.stringify(value));
  }

  window.__MOCK_CALLS__ = calls;
  window.__MOCK_DOCK_STATE__ = DOCK_STATE;

  window.__MOCK_FAIL__ = function (cmd) {
    failures.add(cmd);
  };

  window.__MOCK_EMIT__ = function (event, payload) {
    const ids = listeners.get(event);
    if (!ids) return;
    for (const id of Array.from(ids)) {
      const callback = callbacks.get(id);
      if (callback) callback({ event: event, id: nextEventId++, payload: payload });
    }
  };

  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = {
    unregisterListener: function (event, id) {
      const ids = listeners.get(event);
      if (ids) ids.delete(id);
      callbacks.delete(id);
    }
  };

  window.__TAURI_INTERNALS__ = {
    callbacks: callbacks,
    metadata: {
      currentWindow: { label: "main" },
      currentWebview: { windowLabel: "main", label: "main" }
    },

    transformCallback: function (callback, once) {
      const id = nextCallbackId++;
      callbacks.set(id, function (data) {
        if (once) callbacks.delete(id);
        if (typeof callback === "function") callback(data);
      });
      return id;
    },

    unregisterCallback: function (id) {
      callbacks.delete(id);
    },

    runCallback: function (id, data) {
      const callback = callbacks.get(id);
      if (callback) callback(data);
    },

    invoke: async function (cmd, args) {
      const payload = clone(args === undefined ? {} : args);
      calls.push({ cmd: cmd, args: payload });
      if (failures.has(cmd)) {
        throw new Error("mock failure: " + cmd);
      }

      switch (cmd) {
        case "get_dock_state":
          return clone(window.__MOCK_DOCK_STATE__);
        case "get_catalog":
          return clone(CATALOG);
        case "get_settings":
          return clone(SETTINGS);
        case "save_settings": {
          if (payload.settings) {
            Object.assign(SETTINGS, payload.settings);
            syncAppearance();
            window.__MOCK_EMIT__("dock://state", clone(DOCK_STATE));
          }
          return null;
        }
        case "pin_app": {
          const id = payload.id;
          if (!DOCK_STATE.items.some((item) => item.id === id)) {
            const found = CATALOG.find((item) => item.id === id);
            if (found && DOCK_STATE.items.length < 10) {
              DOCK_STATE.items.push(clone(found));
            }
          }
          window.__MOCK_EMIT__("dock://state", clone(DOCK_STATE));
          return null;
        }
        case "unpin_app": {
          const index = DOCK_STATE.items.findIndex((item) => item.id === payload.id);
          if (index >= 0) DOCK_STATE.items.splice(index, 1);
          window.__MOCK_EMIT__("dock://state", clone(DOCK_STATE));
          return null;
        }
        case "reorder_pins": {
          const byId = new Map(DOCK_STATE.items.map((item) => [item.id, item]));
          const next = (payload.ids || [])
            .map((id) => byId.get(id))
            .filter(Boolean);
          if (next.length === DOCK_STATE.items.length) {
            DOCK_STATE.items = next;
          }
          window.__MOCK_EMIT__("dock://state", clone(DOCK_STATE));
          return null;
        }
        case "sample_monitor":
          return clone(MONITOR_SAMPLE);
        case "plugin:event|listen": {
          const ids = listeners.get(payload.event) || new Set();
          ids.add(payload.handler);
          listeners.set(payload.event, ids);
          return payload.handler;
        }
        case "plugin:event|unlisten": {
          const ids = listeners.get(payload.event);
          if (ids) ids.delete(payload.eventId);
          callbacks.delete(payload.eventId);
          return null;
        }
        case "set_hit_rects":
          return null;
        case "get_shelf":
          return clone(SHELF);
        case "shelf_add": {
          for (const path of payload.paths || []) {
            const known = SHELF.some(
              (item) => item.path.toLowerCase() === String(path).toLowerCase(),
            );
            if (!known && SHELF.length < 12) SHELF.push(shelfItem(path));
          }
          return clone(SHELF);
        }
        case "shelf_remove":
          SHELF = SHELF.filter((item) => item.path !== payload.path);
          return clone(SHELF);
        case "shelf_clear":
          SHELF = [];
          return clone(SHELF);
        case "open_shelf_item":
          return null;
        default:
          return null;
      }
    }
  };
})();
`;
