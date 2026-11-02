# Tauri + React + Python Dock — Design

**Date:** 2026-10-26
**Status:** Approved design, ready for implementation planning
**Product location:** `D:\dockyh\dock-tauri\`
**Relationship to existing work:** A separate product. The native Rust `dock-app` workspace (commits `eab0664`, `b23c845`, `f4fb8da`) stays committed and paused. Neither product imports the other, and the provenance/release boundary rules in `dock-app/assets/manifest.json` continue to apply to native `dock-app` only.

---

## 1. Decision record: why this stack, and what it costs

The existing approved specs ban WebView for the Dock runtime:

- `docs/research/00-SUMMARY.md:58` — "自绘层 D2D + DComp 合成;禁 WebView(红线)"
- `docs/superpowers/specs/2026-10-25-desktop-suite-resource-design.md:30` — "不引入 WebView 作为 Dock runtime"
- `docs/research/tech-stack-research.md:95` — Tauri is listed under "不引入"

Those specs were written to beat Seelen UI's 70–120 MB WebView footprint. This product deliberately reverses that decision, and the cost is paid in the performance gate rather than hidden: a Tauri build **cannot** reach the native target's single-process memory profile, so the gate is split into two numbers instead of one (section 8).

What the stack buys: React's declarative model for the magnification/reflection dock, a mature app-grid surface for Launchpad, and Python's `psutil` for system metrics without hand-rolling Win32 performance counters.

**Non-goals, restated so they do not creep back in:** no Explorer/DWM injection, no shell replacement, no taskbar takeover, no WebView-based fake windows, no arbitrary script execution.

---

## 2. Product scope (v1)

Six surfaces ship in v1:

1. **Dock** — bottom edge only; magnification with neighbor falloff, reflection, running indicators, minimized counts, click to launch/activate, right-click item menu (including add/remove).
2. **Auto-hide** — edge-triggered reveal, configurable delay, manual lock; defaults **off**.
3. **Window minimize-into-dock** — polling-based window snapshot; minimized windows show as a count on the owning app's dock tile; clicking the tile restores and activates.
4. **Launchpad** — app grid from Start Menu scan, search, categories, explicit empty/loading/error states.
5. **Floating system monitor** — CPU, memory, disk, network from the Python sidecar; sparklines; unavailable state when the sidecar is down.
6. **Settings** — icon size, magnification toggle, auto-hide, motion profile, reduced motion, monitor panel visibility, import/export/reset.

**Deferred, explicitly:** left/right/top edges, drag-to-reorder within the dock, drag-from-Launchpad-to-dock, Shelf, stacks, system tray mirroring, Docklet plugin host, genie/scale minimize animation, DWM live window thumbnails, sub-docks, density/variance dials.

Adding an app to the dock in v1 is done through the item/blank context menu, not by dragging. This keeps the v1 drag surface at zero and is the main scope difference from the "edges + drag reorder" option.

---

## 3. Process model

Three processes, one renderer.

| Process | Language | Owns | Measured by |
|---|---|---|---|
| `dock-tauri.exe` | Rust | Window geometry, auto-hide state machine, app catalog, icon extraction, settings, IPC, sidecar supervision | Host memory gate (< 30 MB) |
| WebView2 tree | Chromium | All pixels. React owns rendering entirely. | Total memory gate (< 200 MB) |
| `monitor.exe` | Python | `psutil` sampling only, spoken over stdio | Reported separately, not gated |

**Exactly one WebView2 renderer exists at any time.** This is the single most important constraint in the design: it is why the total-memory gate is reachable at all. It is enforced by the single-window architecture (section 4).

The Python sidecar is spawned by Rust, owned by Rust, and terminated when the Rust host exits. It never listens on a socket.

---

## 4. Window architecture: single window, internal routes

One Tauri window, resized by Rust as the user moves between surfaces.

| Route | Window geometry | Trigger |
|---|---|---|
| Dock (default) | Pill bounds + 84 px top margin (magnified icons, tooltip) + 40 px bottom margin (reflection). Anchored to the bottom edge, horizontally centered on the work area. | Always |
| Launchpad | Full work area of the dock's monitor | Dock button / gesture |
| Monitor | Dock width + upward panel; bottom-anchored | Dock button |
| Settings | Centered panel, fixed size | Dock button / item menu |

**Why not a full-screen transparent overlay:** an overlay needs the host to know where the cursor is so it can toggle `ignore_cursor_events`. That requires either a polling timer (violates the idle-CPU gate) or `SetWindowsHookEx` (real input-correctness risk). The tight-window approach makes the pill itself the hit region, so there is no click-through logic and no cursor polling anywhere in the system.

**Why the window margins are part of the geometry, not optional:** magnification at 1.6× grows icons upward and reflections extend downward. Sizing the window to include both means magnified content is never clipped, which is the precondition that makes a non-fullscreen dock possible. Geometry is computed per monitor from work area and DPI, never hardcoded.

**Consequence to accept:** the window resizes when routes change. This happens only on explicit user action (opening Launchpad, monitor, or settings), never during hover or idle.

---

## 5. Component boundaries

| Unit | Owns | Depends on | Tested by |
|---|---|---|---|
| `src-tauri/src/shell` | Per-monitor window geometry, auto-hide state machine, reveal/collapse, always-on-top policy | Win32 monitor/window APIs, `tauri` | Rust unit tests on pure geometry and state transitions |
| `src-tauri/src/apps` | Start Menu `.lnk` scan, `Get-StartApps` for UWP identity, launch, running-window reconciliation, minimized counts | Win32 Shell/COM via `windows` crate | Rust tests on fixture directory; live `--diagnose-apps` |
| `src-tauri/src/icons` | Icon extraction at requested size, PNG disk cache | Win32 Shell, `image` crate | Rust tests over fixture exes/lnks; cache-hit assertions |
| `src-tauri/src/metrics` | Sidecar spawn, JSON-RPC framing, restart policy, sample lifecycle | child process | Rust tests with a scripted fake sidecar binary |
| `src-tauri/src/settings` | Versioned JSON, atomic write, `.bak` recovery, import/export/reset | filesystem | Rust tests with temp dirs and corruption cases |
| `backend/monitor` | `psutil` sampling and nothing else | `psutil` | pytest with injected fake `psutil` |
| `src/` (React) | Routes, dock rendering, magnify math, tooltip, Launchpad, monitor charts, settings form | IPC only | Vitest for pure logic; Playwright with mocked IPC |

**Read-only shell commands:** `apps` and `icons` need the `windows` crate for Shell/COM calls. The host crate therefore enables only the features it uses (`Win32_Foundation`, `Win32_UI_Shell`, `Win32_UI_WindowsAndMessaging`, `Win32_System_Com`, `Win32_Graphics_Gdi` for icon bitmaps) rather than the full crate, to keep host binary size and startup work down.

**Boundary rule:** `src/` never calls Win32 and never spawns processes. `src-tauri/src/shell` and `apps` never render. `backend/monitor` never touches the UI, the filesystem, or the network beyond what `psutil` exposes.

---

## 6. Data flow and IPC contract

Three channels, each with one job.

| Channel | Direction | Transport | Payload |
|---|---|---|---|
| React ↔ Rust | both | Tauri `invoke` (request/response) | Commands and their results |
| Rust → React | one-way push | Tauri events: `dock://state`, `monitor://sample`, `monitor://status` | Render snapshots; React never polls |
| Rust ↔ Python | both | **newline-delimited JSON-RPC over stdio** | `{"cmd":"sample"}` → `{"t":"sample",...}` |

**Why stdio, not HTTP:** a local HTTP server needs a port (collision risk), an auth token (a secret to manage), and a firewall story. A stdio pipe has no listener, cannot be reached from another process, and dies with its parent. Python writes one JSON line to stdout; Rust reads lines. The stderr stream is reserved for Python tracebacks and is logged under a distinct error code.

**Core types** (Rust definitions; React consumes the serialized form):

```rust
pub struct DockItemView {
    pub id: Uuid,
    pub kind: ItemKind,          // App | Separator | System | Docklet(deferred)
    pub name: String,
    pub icon: Option<String>,    // data URL from cache; None => fallback glyph
    pub running: bool,
    pub minimized: u32,
    pub disabled_reason: Option<String>,
}

pub struct DockState {
    pub items: Vec<DockItemView>,
    pub edge: Edge,
    pub autohide: bool,
    pub magnify: bool,
    pub reduced_motion: bool,
    pub profile: MotionProfile,  // MacOsRestrained | VitroElastic
}

pub struct MonitorSample {
    pub cpu: f32,                // 0..=100
    pub mem_used: u64,           // bytes
    pub mem_total: u64,
    pub disk_used: u64,
    pub disk_total: u64,
    pub net_rx: u64,             // cumulative bytes
    pub net_tx: u64,
}
```

**Platform truth rule:** every visible state comes from a platform snapshot. Clicking an app does not optimistically paint a running dot. `apps` reconciles window ownership by `EnumWindows` at **1 Hz** (bounded, not per-frame), and only a changed snapshot emits `dock://state`. A process that exits therefore loses its indicator without any explicit "stopped" message.

---

## 7. Rendering and motion

### 7.1 Glass material

CSS `backdrop-filter` **cannot** blur the desktop from a transparent window — it only blurs content inside the same web page. So the glass is built from two layers:

1. **Painted glass (baseline, always available):** `rgba(22,26,38,.62)` fill, tiled 128 px monochrome noise, 1 px top inner highlight (`white@15%`), 1 px bottom inner shadow (`black@15%`). Parameters are taken verbatim from `docs/specs/macos-parity.toml` (`[material]`: blur 22 dark / 20 light, fill `black@25%` / `white@18%`, saturate 1.08 / 1.10, grain alpha 0.0025 / 0.003, inner hairline values as listed).
2. **DWM Acrylic (probe-gated enhancement):** at startup, try attaching acrylic via `window-vibrancy` and inspect whether the rounded pill corners clip cleanly. If the rectangular blur region bleeds past the corner radius, fall back to painted glass for the session.

Painted glass is the contract; acrylic is an upgrade that can never break the shape. This avoids betting visual correctness on an API edge behaviour.

### 7.2 Magnification discipline

- Scale is a **pure function** `scale(distance, peak, falloff, radius)` implemented once in React and locked by Vitest: `peak = 1.6`, `falloff = 0.45`, `radius = 120` for the Vitro profile; `peak = 1.2` for the macOS profile.
- A `pointermove` handler writes CSS custom properties only (`--scale-3: 1.42`). It never calls `setState`. The compositor interpolates; React does not re-render during hover.
- Icons animate with `transform: scale()` and `transform-origin: bottom center`, so bottoms stay aligned. `width`/`height` are never animated — that would trigger layout.
- Reflection is a **pre-rendered static layer** that only follows with `scaleY`; it is never rebuilt during magnification (following the Nexus R2 performance finding).
- `will-change: transform` is applied only to icons inside the active magnification radius and removed on leave, so the dock does not hold a permanent compositor layer for every tile.
- Reducing motion replaces all scale/bounce with a ≤ 200 ms opacity/position fade and leaves layout and hit targets identical.

### 7.3 Auto-hide without a hook

When auto-hide is enabled, the dock window collapses to a **2 px full-width strip** on the screen edge instead of vanishing. That strip is the hot zone, so reveal is a native mouse-enter event — no cursor polling, no `SetWindowsHookEx`.

Timing follows `macos-parity.toml`: show delay 300 ms, hide delay 500 ms, slide 200 ms with fade. Timers are cancellable and state-driven, so a failed timer can never leave the dock permanently visible.

**Known trade-off:** at the bottom edge the strip overlaps the taskbar's lowest 2 px. It handles mouse-enter only and forwards clicks, and auto-hide therefore defaults **off**.

---

## 8. Performance gates

Two memory gates, because one honest number does not exist for this stack.

| Gate | Target | Measurement |
|---|---:|---|
| Tauri host private bytes | **< 30 MB** | 60 s after showing the dock once, pointer away, median of 3 runs |
| Host + all WebView2 children | **< 200 MB** | Same instant, full process-tree sum |
| Idle CPU | **< 1%** | 60 s idle, no animation, no active sampling |
| Hover frame time | p95 ≤ 16.7 ms | Automated hover loop, requestAnimationFrame deltas |
| Cold start to interactive | < 1500 ms | Process spawn to first interactive-ready event |

The measurement script reports host / browser / GPU / utility processes separately so optimization has a named target rather than a single opaque total.

**Optimization commitments that make the gates plausible:**
- One renderer, always (section 3).
- No timers at rest: metrics sampling exists only while the monitor route is open. The
  Python sidecar is stopped after 45 s without a sample and re-warmed on the monitor
  route, so a closed monitor carries no resident interpreter.
- 1 Hz window reconciliation, never per-frame enumeration.
- Icon PNGs cached on disk and served as data URLs, decoded once per session.
- Reflections and noise pre-rendered, not per-frame.
- The gate is measured after one reveal: WebView2 only rasterizes the full pill once it
  has been shown, so a never-revealed measurement under-reports the idle footprint.

---

## 9. Error handling and persistence

Nothing decorative. Every failure has a visible, recoverable state.

| Failure | Visible behaviour |
|---|---|
| App launch fails | Dock tile shake + toast with the real reason + retry action |
| Icon extraction fails | Uniform fallback glyph; never an empty box |
| Python sidecar crashes | Monitor shows "unavailable" with retry; dock is unaffected |
| Settings file corrupt | Load `.bak`; if both corrupt, restore defaults and write a diagnostic log |
| App disappears while docked | Tile shows disabled state with reason; removable via menu |

**Persistence:** `%APPDATA%\DockApp\settings.json`, `schema_version: 1`. Writes follow the verified temp → flush → rename → `.bak` sequence. An interrupted write restores the last valid snapshot. First run seeds the dock from the Start Menu scan (Explorer, Settings, plus up to 8 discovered apps); after that the user's explicit list wins.

---

## 10. Testing

Four layers.

| Layer | Covers |
|---|---|
| Rust unit | Geometry per monitor/DPI, magnification math parity with React, auto-hide state machine, atomic write and `.bak` recovery, Start Menu parsing from a fixture directory, JSON-RPC line framing |
| Rust integration | A scripted **fake sidecar** binary: verifies spawn, normal samples, crash, and restart-with-backoff |
| React (Vitest) | Magnify falloff, search ranking/filtering, running-state mapping, settings validation, four tile states (idle / running / minimized / disabled) |
| Playwright + mocked IPC | Hover magnify → click launch; open Launchpad → search → add to dock via context menu; open monitor → render sample; change setting → persists across restart |

No test asserts on the real Start Menu or real CPU values; both are injected.

**Acceptance checks that need a real desktop** are scripted but require an interactive session, and are recorded as manual sign-off rather than CI: app launch/activate against a fixed app matrix, auto-hide reveal/collapse on the real screen edge, always-on-top behaviour against the taskbar, and the two memory gates plus idle CPU measured with the measurement script.

---

## 11. Known risks

1. **Acrylic corner clipping** — mitigated by the probe-gated fallback to painted glass (7.1).
2. **Z-order competition with a topmost taskbar** — always-on-top policy is documented and covered by an acceptance check; the dock never replaces the taskbar.
3. **UWP app discovery** depends on one `Get-StartApps` call; if it fails, only classic `.lnk` apps appear and the UI says so.
4. **PyInstaller is not installed** in the current environment; packaging Task requires adding it (or an equivalent freezer) before the sidecar can ship.
5. **`pnpm` is not installed**; the frontend uses npm unless that changes.
6. **npm dependency surface** is the least controllable cost in this stack. Every frontend dependency must be justified and pinned, and the build must not pull in a component library, an animation library, or a charting library — sparklines are hand-drawn SVG.

---

## 12. Release boundary

`docs/superpowers/plans/2026-10-25-desktop-suite-implementation.md` Task 12 already forbids `Glassboard-main`, `vitro` runtime sources, and copied GlassDesk/BitDock/Nexus code from entering the staging directory. That rule applies here unchanged.

This product adds two constraints of its own:

- The release package contains no HTML/JS **prototype** files from `docs/prototypes/`. They are visual reference only. Since this product *does* ship HTML/JS (the built React bundle), the existing `dock-app` forbidden-extension rule cannot be reused verbatim; the staging scan for `dock-tauri` must instead reject files whose path is under `docs/prototypes/` and reject any `.html` outside `dist/`.
- No legacy `vitro` artwork ships. `dock-app/assets/manifest.json` records all 25 icons as `allowed_use: "reference"` precisely because authorship is unverified. Those icons may be used for local visual comparison; shipping requires the redrawn vector masters described in `docs/superpowers/specs/2026-10-25-desktop-suite-resource-design.md` §3.2 first.

---

## 13. Acceptance definition

A feature is complete only when the platform action, state update, persistence, error path, and accessible interaction all work — not when the icon renders. The v1 release additionally requires both memory gates, the idle-CPU gate, and the cold-start gate to be measured and recorded in the README.

---

## 14. Phase 3 — configurable appearance and native input

This section supersedes the schema-1 persistence detail in §9 and records the product
requirement that **every dock parameter is user-configurable from a dedicated settings
page** (icon size, spacing, padding, radius, tint, opacity, reflection, magnification,
auto-hide, entry visibility, and app membership), plus the input fixes found in live use.

### 14.1 One configuration path

`Settings` (schema 3 since §16, snake_case on disk) → `DockMetrics` / `DockAppearance`
(pure, logical pixels) → window geometry and React CSS variables. React receives the
derived appearance in every `dock://state`; it never invents geometry defaults. The
settings IPC is a camelCase `SettingsView`; pins and the schema version remain owned by
Rust.

Ranges and defaults live in Phase 3 plan §“The settings model”, mirrored by
`src/settings/validate.ts`. Rust sanitises values on load and before save so a hand-edited
file cannot send the window invalid sizes. Schema 1 files load with defaults for new
fields; a stored `autohide` value is preserved, while new installs default to on.

Height is derived as `icon_size + 2 * padding_v`; width is derived from item count,
spacing, padding, and the enabled leading tiles. There is no separate fixed width to
drift from the rendered pill.

### 14.2 Input model

The window is larger than the visible pill (magnify headroom + reflection), so pixels,
not DOM regions, decide input:

- React reports the measured pill **width** in CSS pixels. The watcher derives the pill
  rect's position and height from the window rect and the same metrics that sized it, so
  auto-hide geometry changes cannot stale the hit region. A context menu is reported as
  an extra overlay rect.
- The native watcher polls `GetCursorPos` at 16 ms near the dock and 100 ms away, and
  toggles `set_ignore_cursor_events`. Clicks in the headroom or reflection band reach the
  application underneath; clicks on the pill and an open menu do not.
- Auto-hide is scoped to the pill. The reflection band counts as pointer presence so a
  reveal from the edge does not immediately hide, but it remains click-through. Leaving
  the pill upward starts the hide delay even while the pointer is still inside the
  window — the original defect where the window-sized hot zone kept the dock visible.
- Fullscreen yield compares the foreground window rect to the monitor **screen** rect
  (maximised windows stop at the work area and do not count). Suppression hides the native
  window and disables hit testing; exiting fullscreen restores the edge strip.

### 14.3 Icons and startup cost

Icons are extracted at `round(icon_size * monitor.scale)` device pixels (clamped to the
Shell jumbo 256 px), so a 48 CSS px icon at 150 % uses a 72 px source instead of an
upscaled 48 px one. The runtime keeps a lazy `IconSnapshot` parallel to the catalog:
the dock loads pinned icons, Launchpad loads the rest on first open, failures are not
retried, and the snapshot resets only when the device size or catalog length changes.
The 1 Hz sync emits only when the running set, pins, or settings change, and it no longer
touches icon files. The Python sidecar is pre-warmed 500 ms after startup.

### 14.4 Settings and app management

The settings window is an opaque, tabbed surface (Appearance / Behaviour / Apps /
Advanced) with a live preview. Edits are debounced 300 ms, validated, saved atomically,
and applied to the running window; closing the panel flushes the last edit. The Apps tab
adds/removes/reorders pinned apps; the dock's right-click menu removes non-system tiles.
All actions emit a fresh `dock://state`, so Launchpad, Settings, and the dock agree
without bespoke refreshes.

### 14.5 Measured result

Release build, 60 s settle, median of three: **5.1 MB host**, **179.2 MB total** including
WebView2, the Python sidecar, and `conhost`, **0.04 % idle CPU**. Both memory gates and
the CPU gate pass with more headroom than Phase 1. Click-through, pill-scoped hide, and
fullscreen suppression were verified live on the development desktop.

> Phase 4 superseded this measurement. That figure was taken without revealing the dock
> first, so WebView2 had not rasterized the full pill. The current gate, method, and
> numbers (**5.9 / 186.6 / 0.03**) are in §8.

### 14.6 Still open

- Real desktop blur remains blocked by the whole-window acrylic limitation in §7.1/T18.
  The settings page controls painted-glass tint and opacity only.
- Tray mirroring, Docklets, MSIX/AUMID identity, and custom icon files remain later work.
  Drag-to-reorder and Launchpad drag-in were added in §15; the shelf is in §16; the launch
  pulse is part of the tile state machine (a 700 ms bounce, suppressed by reduced motion).
  A true squircle mask still needs an `iconShape` decision because it would replace the
  configurable corner radius.
- The dock cap is 10 items; the 2 px collapsed hot strip still overlaps the taskbar's
  lowest pixels (mitigated by fullscreen yield and configurable delays).

## 15. Phase 4 — drag to reorder and Launchpad drag-in

### 15.1 Gesture model

Dock reorder is pointer-driven rather than HTML5 drag-and-drop: the tiles already own
pointer events for magnification and the context menu, and window-level listeners let the
gesture survive leaving the pressed tile. A press becomes a drag after 4 CSS px; below
that threshold it is still a click. The dragged tile follows the pointer with
`translateX`, the tiles between the source and target slots shift one pitch, and the DOM
order is untouched until the drop. Magnification is suspended for the gesture so the
layout the maths reads is the resting one.

The index maths lives in `src/dock/drag.ts` (`reorderIds`, `targetIndex`, `shiftFor`,
`placeAt`) and is unit-tested without a layout.

### 15.2 Native cooperation

`set_drag_active(true | false)` tells Rust that a drag owns the pointer. While active:

- auto-hide is locked (`Event::SetLocked`), so the dock cannot collapse under the gesture;
- the pointer watcher keeps the whole window interactive instead of passing clicks
  through the headroom, so the webview keeps receiving moves above the pill;
- on release the watcher resumes normal presence and click-through handling; if the
  pointer ended outside the pill, the first frame emits `PointerLeft` and arms the
  configured hide delay.

The command is fire-and-forget from React. A window blur cancels the drag and clears the
flag, so a lost `pointerup` cannot leave the dock locked.

### 15.3 Drop semantics

- Reordering commits `reorder_pins` with the full permutation and shows the new order
  optimistically until the matching `dock://state` arrives.
- Dragging from Launchpad cannot target the dock window directly: the single window is on
  the Launchpad route, so the dock is not on screen. Dragging a tile therefore reveals a
  dock strip at the bottom edge. The strip previews the current dock with the app already
  inserted at the hovered gap. Releasing inside the bottom 140 px pins at that gap
  (`pin_app` then `reorder_pins`); an app that is already pinned is moved instead.
  Releasing elsewhere cancels. Keyboard reordering remains available in Settings, so the
  gesture is not the only way to manage order.

### 15.4 Tests

Vitest covers the pure maths and both gestures (pointer down/move/up, click suppression,
cancel outside the drop band). Playwright drags with real mouse events and asserts
`reorder_pins` / `pin_app` and the absence of `launch_item`. The Rust command is compiled
into the invoke handler; the watcher branch shares the existing poll loop and is a few
lines, verified manually by the owner.

## 16. Phase 5 — session shelf, schema 3, and art assets

### 16.1 Why a session shelf

The shelf is a scratch space, not a file manager. It holds up to 12 paths for the current
session, deduplicated case-insensitively, and drops them on exit. That keeps the window
small, avoids a second persistence format, and matches the macOS shelf's role.

### 16.2 Input path

Tauri's webview drag-drop event (`tauri://drag-drop`) is the only input. It fires only
where the window is interactive, which is the visible pill or an open panel; the
transparent headroom stays click-through. A drop calls `shelf_add`, stores the paths in
`Runtime`, switches to the `shelf` route, and resizes the window to the panel geometry
(shared with the monitor). Icons are extracted through the existing Shell pipeline at
`round(64 × DPI)` and cached under `%APPDATA%\DockApp\icons` like dock icons.

With auto-hide on, the collapsed 2 px strip is a poor drop target: the user reveals the
dock first, then drops on the pill, or drops onto an already-open shelf panel.

### 16.3 Commands and safety

`get_shelf`, `shelf_add`, `shelf_remove`, and `shelf_clear` return the updated view so the
React state never derives from a stale copy. `open_shelf_item` only accepts a path that is
already on the shelf, so a compromised webview cannot turn it into a general launcher.

### 16.4 Schema 3

`show_shelf_entry` joins the dock-entry toggles. It is a bool defaulting to true; schema 1
and 2 files load with the default and are rewritten as schema 3 on the next save. The dock
width and `leading_tiles` include the shelf whenever the toggle is on.

### 16.5 Art assets

The application icon is `design-art/icons/dock.ico` with the matching PNG sizes; the shelf
entry uses `design-art/art/dock-glyphs.svg#g-tray` (24 grid, 1.7 stroke). The rest of the
art pack (glass/bare/frost themes, squircle clip, token scales, light theme, 3D samples) is
a reference for later P0/P1 work, not part of this phase.

### 16.6 Tests

Rust covers the pure path list (dedupe, cap, removal, names). Vitest covers the panel
states and callbacks. Playwright opens the empty shelf, emits a `tauri://drag-drop` event,
and asserts `shelf_add`, `open_shelf_item`, removal, and the absence of a launch.

## 17. Running-app activation

Clicking a dock tile resolves the app's entry, snapshots the visible top-level windows,
and matches them exactly by process path, then by executable file name (the same rule as
the running indicators). A visible match is focused; if only minimized matches exist, the
first is restored and then focused. ShellExecute is used only when no window matches, so a
click on a running app no longer opens a second instance. The focus is best-effort:
Windows can refuse a foreground change, in which case the window is still restored.

`WindowRef` carries the HWND, and `apps::focus_handle` is pure and unit-tested; the Win32
calls (`ShowWindow(SW_RESTORE)`, `SetForegroundWindow`) are verified live.

The window snapshot excludes the desktop (`Progman`/`WorkerW`) and the taskbar
(`Shell_TrayWnd`). Both are owned by explorer.exe and are visible, so without the
exclusion the desktop window wins the "prefer a visible window" rule and a minimized
Explorer window never comes back.
