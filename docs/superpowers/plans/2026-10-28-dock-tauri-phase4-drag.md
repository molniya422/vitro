# Dock Tauri — Phase 4: drag to reorder and Launchpad drag-in

> **For agentic workers:** this plan is implemented; treat it as the design record for the
> gesture model and rerun the verification commands below before changing it.

**Implementation status (2026-10-28).** T25–T28 are implemented. Rust 122, Vitest 72,
Playwright 18, and pytest 9 pass. The release/MSI was **not** rebuilt, so the existing MSI
predates this phase.

**Context.** Phase 3 finished configurable appearance, native click-through, auto-hide,
sharp icons, and the settings page. The next two items on the owner's list were
drag-to-reorder and dragging an app from Launchpad into the dock. Both are interaction
features; neither changes the settings schema.

**Constraints carried in:**

- One transparent, borderless window; React switches routes inside it. The dock is not on
  screen while Launchpad is open.
- The native pointer watcher owns click-through and auto-hide using the pill rect, and
  toggles `set_ignore_cursor_events` twice a frame at close range.
- No new dependencies.

## T25 — Pure reorder maths

Files: `src/dock/drag.ts` (new), `src/dock/drag.test.ts` (new).

- `reorderIds(ids, from, to)` — move one id, return a copy, clamp the target.
- `targetIndex(pointerX, centers)` — nearest resting slot, clamped to the tile count.
- `shiftFor(index, from, to, pitch)` — the offset that opens the gap for a non-dragged
  tile.
- `placeAt(ids, id, index)` — insert a new id or move an existing one; used by the
  Launchpad drop because the dragged app may not be pinned yet.

Verify: `npm test` covers forward/backward moves, clamping, empty and single-tile docks,
and both gap directions.

## T26 — Dock gesture and native cooperation

Files: `src/dock/Dock.tsx`, `src/dock/DockItem.tsx`, `src/dock/dock.css`,
`src/api.ts`, `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`,
`src-tauri/src/pointer_watch.rs`, tests in `src/dock/Dock.test.tsx` and
`e2e/dock.spec.ts`.

- Press on a tile records `{id, index, startX}` in CSS pill coordinates; the gesture
  becomes a drag after 4 px and only then calls `set_drag_active(true)`.
- The gesture listens on the window, so it survives leaving the tile. `pointercancel` and
  window `blur` cancel it and release the native lock.
- During the drag the tile follows the pointer with `translateX`; neighbours shift by one
  pitch; magnification is suspended; the DOM order is unchanged until the drop.
- On release the new order is committed with `reorder_pins` and shown optimistically
  (`pendingOrder`) until the matching state push arrives. A dropped tile never launches.
- Rust: `Runtime.drag`, `set_drag_active` (sets the flag and `Event::SetLocked`), and a
  watcher branch that keeps the window interactive and visible while dragging.

Verify: `npm test`, `npm run test:e2e`, `cargo test`.

## T27 — Launchpad drag-in

Files: `src/launchpad/Launchpad.tsx`, `src/launchpad/AppTile.tsx`,
`src/launchpad/launchpad.css`, `src/App.tsx`, tests in
`src/launchpad/Launchpad.test.tsx` and `e2e/panels.spec.ts`.

- Pressing a draggable tile and moving 5 px starts a drag with a fixed-position ghost.
- The bottom 140 px is the drop band. While dragging, a dock-shaped strip renders inside
  it and previews the current dock with the app inserted at the nearest gap.
- Gap indices are measured from the strip's rendered icons, so the preview and the drop
  agree at any window size or dock length.
- Drop inside the band: an unpinned app is pinned with `pin_app` and then moved into place
  with `reorder_pins`; an already-pinned app is only moved. Drop elsewhere cancels.
  Clicking a tile still launches it; the `+` button remains the non-pointer path.
- Pinned and disabled tiles are not dragged.

Verify: `npm test`, `npm run test:e2e`.

## T28 — Docs and verification

Files: `README.md`, `docs/superpowers/specs/2026-10-26-tauri-python-dock-design.md`
(§15), `HANDOVER.md`, `playwright.config.ts`.

- README features, architecture map, test counts, and known limitations.
- Design spec §15 records the gesture/native/drop model.
- Playwright `timeout` raised to 90 s because the first navigation after a cold Vite start
  can exceed 30 s on this machine.

## Verification results

| Suite | Result |
|---|---|
| `cargo test` | 122 passed |
| `npm test` | 72 passed (58 → 72) |
| `npm run test:e2e` | 18 passed (16 → 18) |
| `python -m pytest backend/tests` | 9 passed |

Live check: the dev app loaded and a synthetic drag persisted a `reorder_pins` order to
`settings.json`. The full manual drag check was **not** completed with a real mouse: the
synthetic input used for it turned out to be unreliable (it opened the settings panel and
changed values on its own). The settings file was restored to defaults with
`autohide: true` and the original pins. Treat `e2e` as the authority for index maths and
redo §8.5 of `HANDOVER.md` with a real pointer before shipping.

## Follow-ups

Done after the drag work:

- The release/MSI was rebuilt. Re-measuring after a reveal exposed a regression against
  the 200 MB gate (203–232 MB): the frozen Python sidecar stayed resident while the
  monitor was closed. `Supervisor::reap_if_idle` now stops it after 45 s without a
  sample, and `set_route("monitor")` warms it again. Re-measured median: **5.4 MB host /
  186.9 MB total / 0.04 % idle CPU** — all gates pass. The Monitor was opened once after
  a reap and rendered live samples, so the respawn path is verified.
- The README now documents that the measurement reveals the dock once before settling;
  never-revealed numbers under-report WebView2's raster cache.

Still open:

- Mouse-wheel scrolling of the dock, launch pulse, and true squircle mask remain cosmetic
  follow-ups.
- Shelf, tray mirroring, and Docklet hosting are still deferred.
