# Dock Tauri

A Windows dock built with Tauri + React, with a Python `psutil` sidecar for system metrics.

The dock is a bottom-edge glass pill that launches and activates applications, shows which
apps are running, magnifies under the pointer, hides at the screen edge, and hosts three
panels: Launchpad, a system monitor, and settings.

## Features (v1)

- **Dock** — real app icons extracted from the Start Menu shortcuts, parabolic magnification
  with neighbour falloff, static reflection, tooltips after 500 ms, running indicators driven
  by live window state, minimized-window counts.
- **Launch and activate** — click starts an app; clicking a running app brings its window
  forward.
- **Auto-hide** — optional; collapses to a 2 px edge strip and reveals on pointer contact.
  Off by default.
- **Launchpad** — every installed app, searchable, with loading / error / no-results / empty
  states.
- **System monitor** — CPU (with sparkline), memory, disk, network, sampled at 1 Hz while the
  panel is open and not at all when it is closed.
- **Settings** — icon size, magnification, auto-hide, motion profile, reduced motion, monitor
  visibility. Atomic writes with `.bak` recovery.

Not in v1: left/right/top dock edges, drag-to-reorder, drag-from-Launchpad, Shelf, stacks,
tray mirroring, Docklet plugins, DWM window thumbnails, genie minimize animation.

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

## Tests

```bash
# Rust host (71 tests)
D:/rust/cargo/bin/cargo.exe test --manifest-path D:/dockyh/dock-tauri/src-tauri/Cargo.toml

# Frontend (34 tests)
npm test

# Python sidecar (9 tests)
cd backend && D:/jiqixuexi/anaconda/python.exe -m pytest tests -q
```

## Performance

Measured on the development machine (2560×1440 at 150% scaling, Windows 11) with
`scripts/measure_memory.ps1`, median of 3 runs after a settle period.

| Gate | Target | Release build | Dev build |
|---|---:|---:|---:|
| Tauri host, private bytes | < 30 MB | **7.3 MB** | 8.8 MB |
| Host + all WebView2 children | < 200 MB | **182.4 MB** | 197.8 MB |
| Idle CPU (whole tree) | < 1% | **0.16%** | 0.22% |

The two memory numbers measure different things and are reported separately on purpose: the
host alone is far under budget, while the WebView2 renderer tree dominates the total. On this
machine the total has roughly 18 MB of headroom, so treat it as the real gate.

Reproduce with the release app running:

```bash
powershell -ExecutionPolicy Bypass -File scripts/measure_memory.ps1 -SettleSeconds 60 -Runs 3
```

How the numbers are kept low: exactly one WebView2 renderer (one window, internal routes);
no timers at rest; metrics sampling only while the monitor panel is open; window state
reconciled at 1 Hz rather than per frame; icons cached on disk and decoded once.

## Release build

```bash
npm run tauri build -- --no-bundle     # binary only, fast
npm run tauri build                    # MSI installer
```

**Do not use `cargo build --release` on its own.** Without the `custom-protocol` feature that
the Tauri CLI enables, the binary does not embed the frontend and loads an error page instead
of the dock. This was verified the hard way.

## Architecture

One borderless transparent Tauri window whose geometry Rust controls. React owns every pixel
and swaps routes inside that single window, which is what keeps the total memory gate
reachable. Python is a stdio JSON-RPC sidecar (`{"cmd":"sample"}` → one JSON line) spawned and
supervised by Rust — never an HTTP server, no listening port.

```
src-tauri/src/
  geometry.rs        pure per-monitor window rect (no Win32)
  autohide.rs        pure reveal/collapse state machine (time is passed in)
  settings.rs        versioned JSON, atomic write, .bak recovery
  platform.rs        Win32 work-area / DPI adapter
  apps.rs            Start Menu scan, .lnk resolution, launch, reconciliation
  icon_cache.rs      SHGetImageList extraction -> PNG disk cache -> data URLs
  windows_tracker.rs EnumWindows snapshot (visible, titled, non-tool windows)
  metrics.rs         sidecar supervisor with respawn + cooldown
  commands.rs        the invoke surface React calls
src/                 React: dock, launchpad, monitor, settings
backend/             Python sidecar + pytest suite
```

The spec lives at `docs/superpowers/specs/2026-10-26-tauri-python-dock-design.md` and the plan
at `docs/superpowers/plans/2026-10-26-tauri-python-dock-implementation.md`.

## Known limitations

- **Running-state identity.** An app is matched to its windows by executable path. Store/MSIX
  apps have no classic Start Menu target, so they never show a running dot, and apps whose
  launcher is a different binary than the shortcut target (for example Doubao runs
  `D:\doubao\app\Doubao.exe` while its shortcut points at `D:\doubao\Doubao.exe`) are also
  missed. Verified working for ordinary `.exe` shortcuts.
- **Window filter.** Windows whose only window is `WS_EX_TOOLWINDOW` are excluded from the
  snapshot by design, so a few apps can never reconcile.
- **Auto-hide** is implemented and unit-tested as a state machine, and wired through the
  `set_pointer_inside` / `autohide_tick` commands, but the live collapse-to-strip behaviour has
  not been visually signed off. It defaults to off.
- **Glass material** is painted (fill + noise + inner highlight/shadow). The DWM Acrylic
  upgrade described in the spec is not implemented; painted glass is the spec's fallback and is
  what ships.
- **No end-to-end browser tests.** Persistence and state transitions are covered by Rust unit
  tests, not Playwright.
- **Packaging the sidecar** is a script (`backend/build_sidecar.ps1`) but is not wired into the
  Tauri bundle: that needs `bundle.externalBin` plus target-triple naming. Dev builds run the
  sidecar from the Anaconda interpreter via `backend/run_sidecar.py`.
- The app icon is a placeholder pending final artwork.
