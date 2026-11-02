# Dock Tauri — Phase 5: session shelf, schema 3, and art icons

> **For agentic workers:** implemented; keep this as the design record and rerun the
> verification commands before changing the shelf or the settings schema.

**Implementation status (2026-10-29).** T29–T34 are implemented. Rust 130, Vitest 78,
Playwright 20, and pytest 9 pass. The release/MSI was rebuilt and includes this phase.

**Context.** Phases 3 and 4 finished configurability, native input, drag reorder, and
Launchpad drag-in. The next deferred feature on the owner's list is the shelf: a scratch
space for files dragged from Explorer. The same turn received
`dock-tauri/design-art/`, which supplies the final application icon and a matching tray
glyph.

**Constraints carried in:**

- One transparent window; React swaps routes. Only interactive regions receive OS drops.
- The settings schema was at 2; adding a dock entry needs a schema bump.
- No new npm or Rust dependencies.
- The shelf must not become a general launcher for webview-supplied paths.

## T29 — Pure shelf path list

Files: `src-tauri/src/shelf.rs` (new), `src-tauri/tests/unit/shelf.rs` (new),
`src-tauri/Cargo.toml`.

- `push_unique`: trims, rejects empty, dedupes case-insensitively, enforces
  `SHELF_CAP = 12`.
- `remove`: case-insensitive removal.
- `display_name`: last path component, or the path for a root such as `C:\`.

Verify: `cargo test` covers append, duplicate, empty, cap, removal, and names.

## T30 — Schema 3 and dock geometry

Files: `src-tauri/src/settings.rs`, `appearance.rs`, `window_geom.rs`, tests under
`src-tauri/tests/unit/`, `src/types.ts`, `src/settings/validate.ts`, `preview.tsx`,
`src/settings/SettingsPanel.tsx`, `e2e/tauri-mock.ts`.

- `CURRENT_SCHEMA = 3`; `show_shelf_entry` defaults true. `read_and_validate` accepts
  schemas 1..=3; older files load with the default and are rewritten as 3 on save.
- `leading_tiles` = control + Launchpad + monitor + shelf toggles; dock width follows.
- `route_size` gives the shelf the same panel geometry as the monitor.
- The settings page exposes "Show shelf tile"; the preview counts the fourth leading tile.

Verify: settings migration and defaults, appearance/geometry widths, preview and panel
tests, `tsc`.

## T31 — Shelf commands and route

Files: `src-tauri/src/commands.rs`, `lib.rs`, `src/api.ts`.

- `Runtime.shelf: Mutex<Vec<String>>`, session-only.
- `get_shelf`, `shelf_add`, `shelf_remove`, `shelf_clear` return the updated
  `ShelfItemView[]` (name, path, is-dir, missing, Shell icon at 64 CSS px).
- `open_shelf_item` rejects paths that are not on the shelf.
- `set_route` accepts `shelf`; the watcher treats it like the other panels (fully
  interactive).

Verify: `cargo test`, then the live panel check in T34.

## T32 — React shelf panel and drop handling

Files: `src/App.tsx`, `src/shelf/Shelf.tsx` (new), `src/shelf/shelf.css` (new),
`src/dock/Dock.tsx`, `src/dock/Glyph.tsx`, tests in `src/shelf/Shelf.test.tsx`,
`e2e/panels.spec.ts`.

- `getCurrentWebview().onDragDropEvent` forwards `drop.paths` to `shelf_add`, stores the
  returned view, and switches to the shelf route.
- The dock gets a shelf entry tile; the panel lists items with icon, name, full path,
  open/remove/clear, and a session-only hint. Missing files are disabled.
- The panel is opaque (settings material) and accepts drops itself.

Verify: Vitest panel tests; Playwright opens the empty shelf, emits
`tauri://drag-drop`, then asserts `shelf_add`, `open_shelf_item`, removal, and that no
`launch_item` fired.

## T33 — Art assets

Files: `src-tauri/icons/*`, `src/dock/Glyph.tsx`.

- Application icon replaced with `design-art/icons/dock.ico` plus the matching PNG sizes
  (32/64/128/256/512) under Tauri's names.
- Shelf glyph replaced with `design-art/art/dock-glyphs.svg#g-tray` paths (24 grid,
  1.7 stroke), keeping the existing glyph contract.
- The existing gear, chart, and sliders glyphs were aligned with the same export (the
  folder and grid already matched); the grid, stroke, and accessible names are unchanged.

## T34 — Verification, docs, release

- Live: the release window measured 1212×288 with four leading tiles; the shelf entry
  opened a 1212×768 empty panel. The real OS file drop was not synthesised (Windows drop
  automation is unreliable); use §8.6 of the handover with a real mouse.
  `Monitor` was opened after the sidecar idle-reap and rendered live samples.
- Perf was not re-measured after this phase; the added surface is idle unless open, and
  the sidecar idle-stop from Phase 4 is in the same build.
- README, design spec §16, handover, and this plan updated.

## Verification results

| Suite | Result |
|---|---|
| `cargo test` | 130 passed (122 → 130) |
| `npm test` | 78 passed (72 → 78) |
| `npm run test:e2e` | 20 passed (18 → 20) |
| `python -m pytest backend/tests` | 9 passed |
| Release perf (reveal once, 60 s settle ×3) | **5.9 MB host / 186.6 MB total / 0.03 % CPU** — all gates pass |

## Post-plan addition (2026-10-29)

- **Launch pulse** (T23): `DockItem` gained `launching`; `Dock.activate` sets a 700 ms
  pulse, and `dock.css` animates the icon image so the tile's drag transform is untouched.
  A drag release never pulses, and both the in-app reduced-motion setting and the OS
  preference disable the animation. Vitest: 78 → 80.
- **Glyph alignment**: the gear, chart, and sliders glyphs now use the same
  `design-art/art/dock-glyphs.svg` exports as the folder and grid (24 grid, 1.7 stroke).
- **Click-to-activate**: `launch_item` resolves the app, snapshots windows, and focuses a
  matching window (exact process path, then file name; a visible window is preferred and a
  minimized one is restored) instead of running ShellExecute. With no window it falls back
  to launching. `WindowRef` now carries the HWND; `apps::focus_handle` is pure and tested.
  Rust: 130 → 132.
- **Desktop/taskbar exclusion**: the first live run focused the desktop instead of the
  minimized Explorer window, because `Progman` is a visible explorer.exe window. The
  snapshot now skips `Progman`, `WorkerW`, and `Shell_TrayWnd`; the retest restored the
  minimized Explorer window and made it the foreground window (HWND match).

## Follow-ups

- Real file-drop and open/remove verification with a mouse (handover §8.6).
- Shelf drag-out, persistence, and folder expansion are not implemented.
- The art pack's P0 items remain: squircle clip (needs an `iconShape` setting), token
  colour scales, light theme; then material/iconShape settings with Acrylic disabled until
  T18 is decided.
