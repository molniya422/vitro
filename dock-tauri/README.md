# Dock Tauri

A Windows dock built with Tauri + React, with a Python `psutil` sidecar for system
metrics.

The dock is a bottom-edge glass pill that launches and activates applications, shows
running and minimized state, magnifies under the pointer, hides at the screen edge, and
lets clicks pass through every transparent pixel around it. One window hosts all five
surfaces: the dock, Launchpad, the shelf, the system monitor, and a tabbed settings page.

## Features

- **Dock** — real Start Menu icons extracted at device-pixel resolution (sharp at
  150 % DPI), magnification with a configurable peak and reach, an optional reflection,
  tooltips with a configurable delay, running indicators, minimized counts, a right-click
  menu for opening or removing an app, drag-to-reorder with a gap that opens under the
  pointer, a short launch bounce, and click-to-activate: clicking a running app brings its
  window forward (restoring it first if minimized) instead of opening another instance.
- **Click-through** — the transparent headroom above the pill and the reflection band
  below it never intercept clicks. A native pointer watcher toggles window transparency
  using the measured pill width, so the app underneath keeps receiving input. A tile drag
  temporarily owns the whole window so the gesture keeps its pointer events.
- **Auto-hide** — on by default. It is scoped to the pill, not the whole window:
  leaving the pill starts the hide delay even if the pointer is still inside the old
  window band. Both reveal and hide delays are configurable.
- **Fullscreen yield** — a real fullscreen foreground window suppresses the dock
  completely (the window is hidden); leaving fullscreen restores the edge strip.
- **Settings** — live-applied and persisted atomically with a `.bak` recovery file.
  Appearance, behaviour, auto-hide, dock entries, and app management are editable, with
  a live preview strip. Changes apply while the panel is open and survive restart.
- **Launchpad** — every installed app, searchable; the catalog and its icons load only
  when Launchpad first opens. Dragging a tile to the bottom edge reveals a dock strip and
  dropping it adds the app at the hovered gap.
- **Shelf** — drop files or folders on the dock (or on an open shelf) to park up to 12
  for the session, with their real Shell icons. Click to open, remove individually, or
  clear all. The list is deliberately session-only, like the macOS shelf.
- **System monitor** — CPU (with sparkline), memory, disk, and network. The Python
  sidecar is pre-warmed after startup and re-warmed when the monitor opens; it is stopped
  after 45 s without a sample, so a closed monitor costs no resident Python process.
- **Performance** — one WebView2 renderer, a lazily loaded icon snapshot, and no icon
  work in the 1 Hz window sync.

Not in v1: left/right/top dock edges, stacks, tray mirroring, Docklet plugins, DWM
thumbnails, MSIX/AUMID identity, custom icon files, and real desktop blur.

## Settings

Every value is stored under `%APPDATA%\DockApp\settings.json` (schema 3) and can be
changed from the settings page. Defaults are shown in parentheses.

| Group | Values |
|---|---|
| Appearance | Icon size 32–80 px (48), icon spacing 0–24 px (8), horizontal padding 4–32 px (16), vertical padding 2–24 px (10), bar radius 0–64 px (34), glass tint (`#161a26`), glass opacity 15–100 % (62 %), icon radius 0–50 % (22 %), reflection on/off and strength 0–100 % (25 %). |
| Behaviour | Magnification on/off, peak 1.00–2.00× (1.6), reach 1–5 icons (2.2), elastic/restrained profile, reduced motion, tooltips + delay 0–2000 ms (500), running dots, minimized badges. |
| Auto-hide | On/off (on), reveal delay 0–1000 ms (300), hide delay 100–3000 ms (500), hide over fullscreen apps (on). |
| Dock entries | Show Launchpad tile (on), show monitor tile (on), show shelf tile (on). |
| Apps | Add, remove, and reorder up to 10 pinned apps; the dock right-click menu removes a non-system app. |

Height follows `icon size + 2 × vertical padding`; width follows the item count,
spacing, padding, and the enabled route entries. That keeps the window exactly pill-sized
while allowing the layout to grow without clipping.

Changes are debounced by 300 ms and saved atomically (temp file → fsync → `.bak` →
rename). Invalid values are rejected by both the TypeScript and Rust validators.

## Requirements

- Windows 11 with WebView2 (preinstalled on current builds)
- Rust 1.89+ at `D:\rust` (adjust the paths below if yours differs)
- Node 20+ with npm
- Python 3.11 with `psutil` (and `pytest` for the backend tests)

## Development

```bash
export CARGO_HOME='D:\rust\cargo' RUSTUP_HOME='D:\rust\rustup'
export PATH="/d/rust/cargo/bin:$PATH"      # the Tauri CLI shells out to cargo
cd D:/dockyh/dock-tauri
npm install
npm run tauri dev
```

The dock starts collapsed when auto-hide is enabled. Move the pointer to the bottom edge
to reveal it.

## Tests

```bash
# Rust host (132 tests)
D:/rust/cargo/bin/cargo.exe test --manifest-path D:/dockyh/dock-tauri/src-tauri/Cargo.toml

# Frontend (80 tests)
npm test

# Playwright against a mocked Tauri IPC (20 tests)
npm run test:e2e

# Python sidecar (9 tests)
cd backend && D:/jiqixuexi/anaconda/python.exe -m pytest tests -q
```

The Playwright suite runs the real React app in Chromium with the native bridge stubbed;
it covers rendering, magnification, tooltips, state pushes, settings tabs, app
add/remove, the right-click menu, dock drag-to-reorder, Launchpad drag-to-dock, Launchpad,
the shelf (including a simulated file drop), and the monitor. Native click-through,
auto-hide distances, and fullscreen yield were
verified live on the development desktop.

## Performance

Measured on the development machine (2560×1440 at 150 % scaling, Windows 11) with
`scripts/measure_memory.ps1`, median of 3 runs, 60 s settle, auto-hide on, and the pointer
away from the dock after revealing it once so the WebView2 raster cache for the full pill
is warm.

| Gate | Target | Release build |
|---|---:|---:|
| Tauri host, private bytes | < 30 MB | **5.9 MB** |
| Host + WebView2 + Python sidecar | < 200 MB | **186.6 MB** |
| Idle CPU (whole tree) | < 1 % | **0.03 %** |

The total includes every `msedgewebview2` process, the frozen Python sidecar and its
child, and `conhost`. The sidecar is stopped after 45 s without a sample and warmed again
on the Monitor route, so the idle total carries no Python process; that change alone took
an earlier post-reveal measurement from 203.4 MB to about 187 MB. The remaining budget is
the WebView2 renderer tree; the host alone is far under its gate.

Numbers from the never-revealed state are lower but not representative: WebView2 only
rasterizes the full pill after it has been shown once. Compare against a measurement taken
with the same sequence, not against the Phase 3 figure.

Reproduce with the release app running:

```bash
powershell -ExecutionPolicy Bypass -File scripts/measure_memory.ps1 -SettleSeconds 60 -Runs 3
```

How the numbers are kept low: one WebView2 renderer and one window; icons are extracted
lazily once per size and served from an in-memory snapshot; the 1 Hz loop emits only
when the running set, pins, or settings change; the pointer watcher polls at 16 ms near
the dock and 100 ms away; metrics sampling runs only while the monitor is open, and the
sidecar process exits after 45 s without a sample.

## Release build

The Python sidecar must be frozen first (PyInstaller is not installed by default):

```bash
D:/jiqixuexi/anaconda/python.exe -m pip install pyinstaller
powershell -ExecutionPolicy Bypass -File backend/build_sidecar.ps1
```

This writes `src-tauri/binaries/monitor-x86_64-pc-windows-msvc.exe`, which Tauri bundles
beside the executable as `monitor.exe`. Then:

```bash
npm run tauri build
```

Outputs:

- `src-tauri/target/release/dock-tauri.exe`
- `src-tauri/target/release/bundle/msi/Dock Tauri_0.1.0_x64_en-US.msi` (9.48 MiB)

**Do not use `cargo build --release` on its own.** Without the `custom-protocol` feature
that the Tauri CLI enables, the binary does not embed the frontend and loads an error
page instead of the dock.

The MSI is not installed automatically. Packaging is verified by building it; installing
is a separate decision.

## Architecture

One borderless transparent Tauri window whose geometry Rust controls. React owns every
pixel and swaps routes inside that single window, which keeps the total memory gate
reachable. Python is a stdio JSON-RPC sidecar (`{"cmd":"sample"}` → one JSON line)
spawned and supervised by Rust — never an HTTP server, no listening port.

```
src-tauri/src/
  appearance.rs      settings -> DockMetrics/DockAppearance (pure)
  geometry.rs        per-monitor window rect (pure)
  window_geom.rs     route sizes (pure)
  autohide.rs        reveal/collapse state machine, delays, suppression (pure)
  hit_test.rs        physical rects, pill rect, click-through math (pure)
  foreground.rs      fullscreen predicate (pure)
  settings.rs        schema 3 file, migration, sanitisation, atomic write
  shelf.rs           session shelf path list: dedupe, cap, names (pure)
  platform.rs        Win32 work area, DPI, cursor, foreground window
  apps.rs            Start Menu scan, .lnk resolution, launch, reconciliation
  icon_cache.rs      lazy icon snapshot -> PNG disk cache -> data URLs
  windows_tracker.rs EnumWindows snapshot (visible, titled, non-tool windows)
  metrics.rs         sidecar supervisor with respawn, cooldown, idle reap
  pointer_watch.rs   native cursor watcher: transparency, auto-hide, fullscreen, drags
  commands.rs        runtime caches and the invoke surface React calls
src/                 React: dock, launchpad, shelf, monitor, tabbed settings
src/dock/drag.ts     pure reorder maths: move, nearest slot, gap shift, insert
backend/             Python sidecar + pytest suite
```

`Settings → DockMetrics → DockAppearance → window geometry + CSS variables` is the single
configuration path. No dimension is hard-coded in both places.

## Known limitations

- **Painted glass, not desktop blur.** The fill is tint + opacity + grain + inner
  highlights. DWM acrylic blurs the whole window rectangle, including the transparent
  headroom and reflection bands, so it is not applied. The three options for real blur
  are documented in the Phase 2 plan (T18) and still need a product decision.
- **Dock cap.** Up to 10 pinned apps. Reordering works by dragging a tile or from the
  Settings move buttons; both are pointer-driven, and dragging cannot express a keyboard
  equivalent, so the Settings buttons stay as the accessible path.
- **Launchpad drop strip.** The single window only shows one route at a time, so the dock
  itself is not visible while Launchpad is open. Dragging a tile reveals a dock-shaped
  strip at the bottom edge; the drop band is the bottom 140 px. The strip previews the
  dock order with the dragged app inserted at the hovered gap.
- **Store/MSIX apps.** Running state matches a Start Menu shortcut's resolved executable
  by exact path, then by file name. Store apps without a classic Start Menu target never
  reconcile; AUMID-based identity is follow-up work.
- **Window filter.** Windows whose only window is `WS_EX_TOOLWINDOW` are excluded from the
  snapshot.
- **Auto-hide hot strip.** The collapsed dock is a 2 px full-width strip at the screen
  edge, overlapping the taskbar's lowest pixels. Fullscreen yield and the configurable
  hide delay reduce interference; auto-hide can be turned off.
- **Native coverage.** Playwright mocks the Tauri bridge. Click-through, hide distances,
  and fullscreen yield have unit-tested pure predicates plus live manual verification;
  there is no automated native UI driver.
- **Shelf is session-only.** It holds up to 12 paths and is not persisted. Files can only
  be dropped where the window is interactive: the visible pill, or an open panel. With
  auto-hide on, reveal the dock first (hover the bottom edge), then drop on the pill.
  There is no drag-out; click a shelf item to open it in its default app.
- **No custom icon files** yet; icon appearance is limited to size, spacing, padding,
  corner radius, tint, and opacity. The product icon comes from `design-art/icons`.

## References

- Design: `docs/superpowers/specs/2026-10-26-tauri-python-dock-design.md`
- Phase 1 plan: `docs/superpowers/plans/2026-10-26-tauri-python-dock-implementation.md`
- Phase 2 plan: `docs/superpowers/plans/2026-10-26-dock-tauri-phase2.md`
- Phase 3 plan (configurable appearance, click-through, settings): `docs/superpowers/plans/2026-10-26-dock-tauri-phase3.md`
- Handover guide (Chinese, current state, debugging, constraints): [`HANDOVER.md`](HANDOVER.md)
