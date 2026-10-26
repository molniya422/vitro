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

  const DOCK_STATE = {
    items: DOCK_ITEMS,
    edge: "bottom",
    autohide: false,
    magnify: true,
    reducedMotion: false,
    profile: "vitro"
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
    schema_version: 1,
    icon_size: 48,
    magnify: true,
    autohide: false,
    reduced_motion: false,
    profile: "vitro",
    show_monitor: false,
    pinned: []
  };

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
        case "set_pointer_inside":
        case "autohide_tick":
        case "set_autohide_enabled":
          return { enabled: false, visible: true, pending: false };
        default:
          return null;
      }
    }
  };
})();
`;
