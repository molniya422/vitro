# Dock Tauri — Phase 3 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task.

**Context.** Phase 1 (T1–T15) shipped the dock. Phase 2 added Playwright E2E (T16) and a frozen
sidecar + MSI (T17, done on this machine but not yet reflected in the README). T18 (real DWM
acrylic) remains blocked on a product decision and is **not** part of this phase.

**Implementation status (2026-10-26).** T19–T23 are implemented and T24's verification,
release build, memory measurement, README, and spec update are complete. Deviations from
the steps below, all verified by tests or live use:

- Hit testing reports the pill **width** and lets Rust derive its rect from the window
  and metrics, instead of trusting a CSS x/y measured in a different viewport layout. This
  removed a stale-rect race across auto-hide resizes.
- Settings auto-save after a 300 ms debounce; `Save` and `Close` flush the pending edit.
  The initial load and StrictMode remount never write.
- T23's icon shadow, pressed state, reflection/dot/badge gating, glass gradient, and
  tooltip material are done. A distinct launch pulse and a true squircle mask remain
  cosmetic follow-ups; the configurable icon radius is the current mask.
- The release measured **5.1 MB host / 179.2 MB total / 0.04 % idle CPU**; both gates pass.
- Auto-hide was left enabled in the local settings after verification because it is the
  requested behaviour. The MSI was built but not installed.

Phase 3 exists because of the owner's live feedback and a new product requirement:

> 目前dock不会自动隐藏，干扰前台运行的软件，鼠标还有一段距离就会触发，图标也很模糊，其他项的
> 渲染也不贴近mac的dock，启动软件和插件也很慢，也不能自己调整高度宽度，没法增加或删除项目
>
> 所有的功能都要可以改，比如高度宽度透明度等等，要有一个设置页面专门改这些内容。

The complaints are already traced to concrete code, not guesses:

| Complaint | Actual cause |
|---|---|
| Dock does not auto-hide | `autohide` defaults to `false`; when enabled, `document.mouseleave` only fires when leaving the **whole 1128×288 window**, so the band above the icons counts as "inside". |
| Interferes with foreground apps / blocks clicks | The transparent headroom (84 px) and reflection band (40 px) are part of the topmost window and still hit-test as the dock. Clicks in that band never reach the app underneath. |
| Triggers while the pointer is still far away | Same window-sized hot zone: auto-hide and hover use the window rect, not the pill rect. |
| Icons are blurry | Icons are extracted at a hard-coded 48 px but the screen is 150 % DPI, so a 48 CSS px icon needs a ~72 physical px source and is upscaled. |
| Launching feels slow | The 1 Hz loop re-reads **every** catalog icon from disk and re-encodes each as base64 on every tick (`lib.rs:77`, `commands.rs:63/81/208`). `get_catalog` runs on first paint, and the Python sidecar is cold on first monitor open. |
| Cannot adjust height/width | `icon_size` is persisted but dead: `route_size(..., 48.0)` is hard-coded at `commands.rs:173` and `:311`, and `dock.css` pins `--icon-size: 48px`. |
| Cannot add/remove items | `pin_app`/`unpin_app` exist and are wrapped in `api.ts`, but no UI calls them. |

**References studied (read-only — do not modify):**
- `D:\dockyh\cedro-modern-dock-main\...` — the settings taxonomy this phase adopts:
  `IconSize`, `IconSpacing`, `Transparency`, `BorderRounding`, `TintIcons`, margins, plus a
  live-apply pattern (`OnIconSizeChanged` → appearance service → refresh). No code is copied.
- `D:\dockyh\vitro\src\config.rs` — precedent for clamped theme values (`alpha`, `tint`,
  `accent`, `peak`, `radius`).
- `D:\dockyh\macos-dock-for-windows-master` — a Photoshop theme/asset pack, not a program.
  Used only as a visual target.

**Non-goals for this phase (still wanted later):** drag-to-reorder on the dock, drag from
Launchpad, Shelf, tray mirroring, Docklet host, MSIX/AUMID identity, custom icon files, real
desktop blur (T18). T17 packaging is done but its README claims are corrected in Task 24.

---

## Acceptance criteria

1. Every dock appearance/behaviour parameter in the table below is editable in a dedicated,
   keyboard-accessible settings page and persists across restart.
2. Transparent pixels of the dock window never intercept mouse input; clicking in the headroom
   or reflection band reaches the application underneath. The pill remains fully interactive.
3. Auto-hide uses the **pill rect**, not the window rect. With auto-hide on, the dock hides
   after the configured delay once the pointer leaves the pill, and reveals after the configured
   delay when the pointer enters the bottom hot strip.
4. A fullscreen foreground app suppresses the dock completely and restores it on exit.
5. At 150 % DPI, icons are extracted at device-pixel resolution and render sharp; changing icon
   size re-extracts at the new size.
6. Icon data URLs are computed once per size/catalog change and reused by the 1 Hz sync. The
   catalog is fetched lazily, and the Python sidecar is pre-warmed after startup.
7. Apps can be added, removed, and reordered from the settings page, and removed from a dock
   right-click menu. Launchpad add/remove errors are surfaced.
8. A visual pass makes icons, reflection, running dots, tooltips, press feedback, and the glass
   material closer to the macOS dock, with a preview strip in settings.
9. All suites pass (Rust, Vitest, Playwright, pytest), both memory gates still pass, and the
   README describes the product as it actually is.

---

## The settings model (schema v2)

One `Settings` struct is the single source of truth. Rust derives a serialisable `DockAppearance`
from it for React; React never invents a default that Rust does not know.

| File key | UI label | Type | Range | Default |
|---|---|---|---:|---:|
| `icon_size` | Icon size | u32 | 32–80 | 48 |
| `icon_gap` | Icon spacing | u32 | 0–24 | 8 |
| `padding_h` | Horizontal padding | u32 | 4–32 | 16 |
| `padding_v` | Vertical padding (bar height) | u32 | 2–24 | 10 |
| `pill_radius` | Bar corner radius | u32 | 0–64 | 34 |
| `tint` | Glass tint | String `#rrggbb` | — | `#161a26` |
| `fill_opacity` | Glass opacity | f64 | 0.15–1.0 | 0.62 |
| `icon_radius` | Icon corner radius | u32 (%) | 0–50 | 22 |
| `reflection` | Show reflection | bool | — | true |
| `reflection_strength` | Reflection strength | f64 | 0.0–1.0 | 0.25 |
| `magnify` | Magnification | bool | — | true |
| `magnify_peak` | Magnify peak | f64 | 1.0–2.0 | 1.6 |
| `magnify_radius_icons` | Magnify reach (icons) | f64 | 1.0–5.0 | 2.2 |
| `profile` | Motion profile | `macos` \| `vitro` | — | `vitro` |
| `reduced_motion` | Reduced motion | bool | — | false |
| `tooltips` | Show tooltips | bool | — | true |
| `tooltip_delay_ms` | Tooltip delay | u32 | 0–2000 | 500 |
| `show_running_dots` | Running indicators | bool | — | true |
| `show_badges` | Minimized counts | bool | — | true |
| `autohide` | Auto-hide | bool | — | true (new installs) |
| `show_delay_ms` | Reveal delay | u32 | 0–1000 | 300 |
| `hide_delay_ms` | Hide delay | u32 | 100–3000 | 500 |
| `yield_fullscreen` | Hide over fullscreen apps | bool | — | true |
| `show_launchpad_entry` | Show Launchpad tile | bool | — | true |
| `show_monitor_entry` | Show monitor tile | bool | — | true |
| `pinned` | Dock items | Vec<String> | managed by commands | — |

Notes:
- **Height is derived**, not a separate knob: `pill_height = icon_size + 2 * padding_v`.
  Width is derived from icon size, spacing, padding, the visible leading tiles, and item count.
  This is how macOS-style docks behave; a fixed pixel width would clip content.
- `com.autohide` for new installs becomes **true**. A v1 file migrates with every value
  preserved, including `autohide: false`; the owner enables it in the new settings page during
  Task 24 verification (the old default-file value here was changed while testing, not chosen).
- The unused v1 `show_monitor` key is replaced by `show_monitor_entry` (default true, matching
  what the dock actually rendered). Unknown v1 keys are ignored.
- `fill_opacity` is the glass fill alpha, not window alpha. Fading the whole window would fade
  the icons too. Real desktop blur remains T18.

### Rust shapes

`src-tauri/src/appearance.rs` (new, pure):

```rust
pub struct DockMetrics {           // logical px, before DPI scaling
    pub icon_size: f64,
    pub icon_gap: f64,
    pub padding_h: f64,
    pub padding_v: f64,
    pub headroom: f64,             // magnified icon + tooltip allowance
    pub reflection_space: f64,
    pub leading_tiles: usize,      // control tile + visible route entries
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DockAppearance {        // what React applies as CSS variables
    pub icon_size: f64,
    pub icon_gap: f64,
    pub padding_h: f64,
    pub padding_v: f64,
    pub pill_radius: f64,
    pub tint: String,
    pub fill_opacity: f64,
    pub icon_radius: f64,
    pub reflection: bool,
    pub reflection_strength: f64,
    pub reflection_space: f64,
    pub magnify: bool,
    pub magnify_peak: f64,
    pub magnify_radius: f64,       // logical CSS px = icons * (icon_size + icon_gap)
    pub reduced_motion: bool,
    pub profile: String,
    pub tooltips: bool,
    pub tooltip_delay_ms: u32,
    pub show_running_dots: bool,
    pub show_badges: bool,
    pub show_launchpad_entry: bool,
    pub show_monitor_entry: bool,
    pub leading_tiles: usize,
}
```

Headroom formula (keeps the current 84 px at defaults, but follows the sliders):

```
magnify_extra = if magnify { ceil(icon_size * (peak - 1.0)) } else { 0 }
tooltip_box   = if tooltips { 34 } else { 8 }
headroom      = ceil(magnify_extra + tooltip_box * (if magnify { peak } else { 1.0 }))
reflection_space = if reflection { round(icon_size * 5.0 / 6.0) } else { 0 }
```

The tooltip lives inside the transformed button, so it scales with magnification; the formula
reserves that scaled space. Geometry uses logical values and multiplies by `monitor.scale` only
once, at the end.

### TypeScript shapes

```ts
export interface DockAppearance {
  iconSize: number; iconGap: number; paddingH: number; paddingV: number;
  pillRadius: number; tint: string; fillOpacity: number; iconRadius: number;
  reflection: boolean; reflectionStrength: number; reflectionSpace: number;
  magnify: boolean; magnifyPeak: number; magnifyRadius: number;
  reducedMotion: boolean; profile: MotionProfile;
  tooltips: boolean; tooltipDelayMs: number;
  showRunningDots: boolean; showBadges: boolean;
  showLaunchpadEntry: boolean; showMonitorEntry: boolean;
  leadingTiles: number;
}
```

`DockStateView` loses the duplicated top-level `autohide`/`magnify`/`reducedMotion`/`profile`
fields and gains `appearance: DockAppearance`. `SettingsView` (camelCase IPC DTO) mirrors the
file table; the file on disk stays snake_case.

---

## Architecture decisions

1. **One config path.** `Settings` → `DockMetrics`/`DockAppearance` → geometry + CSS variables.
   No hard-coded `48.0` remains in the host and no hard-coded `--icon-size` drives the dock.
2. **Live apply.** Each settings change is debounced 300 ms, persisted atomically, applied to the
   running window, and broadcast as `dock://state`. Closing the panel flushes the pending save.
3. **Hit testing is native.** CSS `pointer-events: none` cannot make clicks reach another
   application; the window itself still receives them. Instead, a Rust watcher reads
   `GetCursorPos` and toggles `set_ignore_cursor_events` when the pointer crosses the measured
   interactive rects (pill + open context menu). React reports rects in CSS px via
   `set_hit_rects`; the watcher converts them with the window position and scale.
4. **Auto-hide moves to the watcher.** The React `document.mouseenter/mouseleave` + tick loop is
   removed. Rust already owned the state machine; it now also owns pointer presence and applies
   geometry directly, so hiding does not depend on the webview receiving mouse events.
5. **Fullscreen yield.** A pure predicate compares the foreground window rect to the monitor
   **screen** rect (maximised windows stop at the work area and therefore do not trigger it).
   Suppression collapses the dock and disables all hit testing until the fullscreen app exits.
6. **Icons are a snapshot.** Extract once per `(device_px, catalog length)` and reuse the data
   URLs. Device pixel size = `round(icon_size * monitor.scale)`, clamped to 16–256 (the jumbo
   Shell image list is 256 px; never upscale past it).
7. **Settings page = four tabs.** Appearance / Behaviour / Apps / Advanced, with a live preview
   strip in Appearance. The reference products all converge on this structure.

---

## Task 19: Settings schema v2, derivation, and configurable geometry

**Files:**
- Modify: `dock-tauri/src-tauri/src/settings.rs`
- Create: `dock-tauri/src-tauri/src/appearance.rs`
- Modify: `dock-tauri/src-tauri/src/geometry.rs`
- Modify: `dock-tauri/src-tauri/src/window_geom.rs`
- Modify: `dock-tauri/src-tauri/src/state.rs`
- Modify: `dock-tauri/src-tauri/src/lib.rs` (module declaration)
- Modify: `dock-tauri/src-tauri/Cargo.toml` (new `[[test]]` stanzas)
- Modify: `dock-tauri/src-tauri/tests/unit/settings.rs`
- Create: `dock-tauri/src-tauri/tests/unit/appearance.rs`
- Modify: `dock-tauri/src-tauri/tests/unit/geometry.rs`
- Modify: `dock-tauri/src-tauri/tests/unit/window_geom.rs`
- Modify: `dock-tauri/src-tauri/tests/unit/dock_state_view.rs`

- [ ] **Step 1: Write the failing settings tests**

Extend `tests/unit/settings.rs` and update its `sample()` constructor for every new field:

1. `v1_file_migrates_with_defaults_for_new_fields` — write a literal v1 JSON (the old eight
   fields), load it, assert `schema_version == 2`, `icon_size` preserved, `fill_opacity == 0.62`,
   `yield_fullscreen == true`, and the old `autohide` value preserved.
2. `new_defaults_match_the_settings_table` — assert every default in the table above.
3. `sanitize_clamps_every_numeric_field` — out-of-range values for icon size/gap/padding/radius,
   opacity/strength/peak/radius-icons/delays are clamped to the documented bounds; a bad tint
   falls back to `#161a26`; an unknown profile falls back to `vitro`; a lowercase tint is
   normalised.
4. `save_atomic_writes_schema_two` — read the raw JSON back and assert `"schema_version": 2`.
5. Keep all existing atomic-write, backup, and unsupported-schema tests. `UnsupportedSchema`
   now rejects `999` and accepts `1` and `2`.

- [ ] **Step 2: Write the failing appearance tests**

New `tests/unit/appearance.rs`:

1. `defaults_derive_the_expected_metrics` — icon 48, gap 8, padding 16/10, pill height 68,
   headroom 84, reflection space 40, magnify radius 123.2, leading tiles 3.
2. `headroom_grows_with_magnify_peak_and_shrinks_without_tooltips` — peak 2.0 → larger; tooltips
   off → smaller; magnify off → only the tooltip allowance.
3. `reflection_off_removes_the_reserved_space`.
4. `leading_tiles_follow_route_entry_toggles` — both on = 3, one off = 2, both off = 1.
5. `appearance_carries_every_css_value` — serialise, assert camelCase keys and that
   `magnifyRadius == radius_icons * (icon_size + icon_gap)`.
6. `device_icon_px_is_rounded_and_clamped` — 48 @1.5 = 72; 80 @3 = 240; 8 → 16; 999 → 256.

- [ ] **Step 3: Write the failing geometry tests**

Rewrite `tests/unit/geometry.rs` against the new signature
`DockGeometry::compute(monitor, item_count, metrics)` where `metrics: DockMetrics`:

- default width = `(3*48 + 6*48 + 8*8 + 32)` with 6 apps and 3 leading tiles;
- zero apps still includes the control tile and `leading - 1` gaps (regression: the old
  `leading_width` counted one gap too many when the dock was empty);
- icon 80 + gap 16 + padding 24 scales the width and pill height; headroom/reflection follow;
- DPI 1.5 multiplies the final physical rect;
- narrow work area clamps width before centering, `x >= monitor.left`;
- `pill_top()` = `y + headroom`.

Update `tests/unit/window_geom.rs`: `route_size` takes `&DockMetrics`; settings route is
`820×680` **logical**, scaled by monitor DPI (new test at 1.5); launchpad fills the work area;
monitor route still adds `320 * scale`; dock route equals `DockGeometry::compute`.

- [ ] **Step 4: Implement `settings.rs`**

- `CURRENT_SCHEMA = 2`; add every field with a struct-level `#[serde(default)]`.
- `read_and_validate` accepts 1 and 2, sets `schema_version = CURRENT_SCHEMA`, then calls
  `sanitize()`. Unknown keys are ignored (serde default).
- `Settings::sanitize(&mut self)` clamps per the table, normalises hex to lowercase, validates
  the profile; no error type needed for clamped values.
- `Settings::metrics(&self) -> DockMetrics` and `DockAppearance` construction live in
  `appearance.rs` to keep settings about persistence.
- Keep `save_atomic` and `load_or_recover` unchanged otherwise.

- [ ] **Step 5: Implement `appearance.rs`**

- `DockMetrics::from_settings(&Settings)` with the formulas above.
- `DockMetrics::device_icon_px(scale: f64) -> u32` (`round(icon_size * scale).clamp(16, 256)`).
- `DockAppearance::from_settings(&Settings)`.
- Keep `appearance.rs` free of IO and Win32 so it tests without a desktop.

- [ ] **Step 6: Refactor geometry and state**

- `geometry.rs`: delete the fixed `MAGNIFY_HEADROOM`, `REFLECTION_MARGIN`, `PADDING_H/V`,
  `ICON_GAP` constants; keep only `MonitorRect` and `DockGeometry`. `compute` uses
  `metrics.headroom/reflection_space/pill_height`, leading tiles, and item count. Clamp the
  width to the work area before centering. Add `pill_width` if it removes ambiguity.
- `window_geom.rs`: `route_size(route, monitor, item_count, metrics)`. Scale the settings panel
  logical size by `monitor.scale`.
- `state.rs`: replace `DockConfig` with `&DockAppearance`; `DockStateView { items, edge,
  appearance }`; delete duplicated top-level fields.
- Update `dock_state_view.rs` expectations. Keep `seed_pins`, `DOCK_ITEM_CAP = 10`, and
  `build_catalog` unchanged for now.

- [ ] **Step 7: Register tests and run**

Add to `Cargo.toml`:

```toml
[[test]]
name = "appearance"
path = "tests/unit/appearance.rs"
```

```bash
export CARGO_HOME='D:\rust\cargo' RUSTUP_HOME='D:\rust\rustup'
export PATH="/d/rust/cargo/bin:$PATH"
cd /d/dockyh/dock-tauri
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: all Rust tests pass. Do not proceed while anything is red.

- [ ] **Step 8: Commit**

```bash
git -C /d/dockyh add docs/superpowers/plans/2026-10-26-dock-tauri-phase3.md \
  dock-tauri/src-tauri/src/settings.rs dock-tauri/src-tauri/src/appearance.rs \
  dock-tauri/src-tauri/src/geometry.rs dock-tauri/src-tauri/src/window_geom.rs \
  dock-tauri/src-tauri/src/state.rs dock-tauri/src-tauri/src/lib.rs \
  dock-tauri/src-tauri/Cargo.toml dock-tauri/src-tauri/tests/unit/settings.rs \
  dock-tauri/src-tauri/tests/unit/appearance.rs \
  dock-tauri/src-tauri/tests/unit/geometry.rs \
  dock-tauri/src-tauri/tests/unit/window_geom.rs \
  dock-tauri/src-tauri/tests/unit/dock_state_view.rs
git -C /d/dockyh commit -m "feat: add configurable appearance model and geometry"
```

---

## Task 20: Runtime apply, typed settings IPC, icon snapshot, faster startup

**Files:**
- Modify: `dock-tauri/src-tauri/src/settings.rs` (IPC DTOs)
- Modify: `dock-tauri/src-tauri/src/commands.rs`
- Modify: `dock-tauri/src-tauri/src/lib.rs`
- Modify: `dock-tauri/src-tauri/src/icon_cache.rs` (snapshot helper, if useful)
- Modify: `dock-tauri/src-tauri/src/state.rs` (build from appearance)
- Create: `dock-tauri/src-tauri/tests/unit/settings_view.rs`
- Modify: `dock-tauri/src-tauri/Cargo.toml`
- Modify: `dock-tauri/src/api.ts`
- Modify: `dock-tauri/src/types.ts`
- Modify: `dock-tauri/src/App.tsx`
- Modify: `dock-tauri/src/dock/Dock.tsx`
- Create: `dock-tauri/src/dock/layout.ts`
- Create: `dock-tauri/src/dock/layout.test.ts`
- Modify: `dock-tauri/src/dock/DockItem.tsx` (props only in this task)
- Modify: `dock-tauri/src/dock/dock.css` (variables, not the visual pass)
- Modify: `dock-tauri/src/dock/Dock.test.tsx`
- Modify: `dock-tauri/e2e/tauri-mock.ts`

- [ ] **Step 1: Add typed IPC DTOs with tests**

In `settings.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView { /* every file field except schema_version and pinned */ }

impl SettingsView {
    pub fn from_settings(s: &Settings) -> Self;
    pub fn into_settings(self, pinned: Vec<String>) -> Settings; // forces CURRENT_SCHEMA + sanitize
}
```

New `tests/unit/settings_view.rs`:

- `settings_view_round_trips_every_field`;
- `into_settings_preserves_backend_owned_pins`;
- `into_settings_forces_the_current_schema`;
- `get_settings_shape_is_camel_case` (no `icon_size`, no `fill_opacity` in the serialised JSON).

- [ ] **Step 2: Cache settings and icons in `Runtime`**

```rust
pub struct Runtime {
    pub catalog: Mutex<Vec<AppEntry>>,
    pub pinned: Mutex<Vec<String>>,
    pub settings_path: PathBuf,
    pub settings: Mutex<Settings>,          // in-memory source of truth
    pub icons: Mutex<Option<IconSnapshot>>, // px + parallel data URLs
    pub sidecar: Mutex<Supervisor>,
    pub autohide: Mutex<AutoHide>,
    pub route: Mutex<String>,               // "dock" | "launchpad" | "monitor" | "settings"
}

struct IconSnapshot { px: u32, catalog_len: usize, urls: Vec<Option<String>> }
```

- Put `IconSnapshot` and its `ensure(...)` method in `icon_cache.rs`. The snapshot is **lazy
  per index**, not all-or-nothing:

```rust
pub struct IconSnapshot {
    pub px: u32,
    pub urls: Vec<Option<String>>, // parallel to the catalog
    loaded: Vec<bool>,             // a failed extraction still counts as loaded
}
impl IconSnapshot {
    /// Fetch only the requested catalog indices; existing entries are reused.
    pub fn ensure(&mut self, apps: &[AppEntry], indices: impl Iterator<Item = usize>, px: u32);
}
```

  This keeps first paint cheap: the dock requests only its pinned indices, and Launchpad fills
  the rest when it opens. Changing `px` resets the snapshot; a changed catalog length resets it
  too. A second helper `device_icon_px` lives in `appearance.rs`.
- `Runtime::state()` builds `DockStateView` from the cached settings/pins/catalog/snapshot.
- The 1 Hz thread in `lib.rs` uses `runtime.settings` (no disk read), uses the snapshot, and
  **only emits while `route == "dock"`**; it also skips the window snapshot entirely for other
  routes.
- `get_dock_state`/`refresh_state` ensure icons for the pinned ids only; `get_catalog` ensures
  all indices. None re-extracts icons per call after the first.
- `lib.rs` seeds `settings` from the loaded file once; on startup, pass `settings.autohide`
  and the configured delays to `AutoHide`.

- [ ] **Step 3: Make `save_settings` apply live**

```rust
#[tauri::command]
pub fn save_settings(app: AppHandle, runtime: State<Runtime>, settings: SettingsView)
    -> Result<(), CommandError>;
```

Order of operations:
1. `let next = settings.into_settings(runtime.pinned.clone())`.
2. `save_atomic`, then replace `runtime.settings`.
3. Update the auto-hide machine: `handle(SetEnabled(next.autohide))`, `set_delays(...)`;
   forcing a collapse is not required when enabling (the next pointer-leave hides it).
4. If `device_icon_px` changed, drop the icon snapshot; do not block the command by rebuilding.
5. Reapply geometry for the current route (dock: visible/hidden; panels: `route_size`), using
   the new metrics.
6. Emit `dock://state` so the dock re-renders.

- `get_settings` returns `SettingsView`.
- Remove the old `config_from(&Settings)` and the hard-coded `48.0` at the old lines 173/311.
- Add a pure `fn route_size_for(runtime, monitor) -> (f64, f64)` used by `set_route`,
  auto-hide geometry, and `apply_autohide_geometry` so the metrics can never diverge.

- [ ] **Step 4: Pre-warm the sidecar and lazy-load the catalog**

- In `setup`, after `show()`, spawn a thread that waits 500 ms and calls `supervisor.sample()`
  once, discarding the result. This pays the Python/psutil import cost before the monitor opens.
- In `App.tsx`, stop calling `api.getCatalog()` on mount. Fetch it the first time the route is
  `launchpad` (keep loading/error states). `pinApp` still refreshes `getDockState`.
- Surface `pinApp` failures in Launchpad instead of swallowing them (capacity or unknown app).

- [ ] **Step 5: Apply appearance in React**

- Add `layout.ts`:

```ts
export function cssVars(a: DockAppearance): CSSProperties & Record<`--${string}`, string>;
export function iconCenters(count: number, a: DockAppearance): number[];
export function hexToRgba(hex: string, alpha: number): string;
```

  `iconCenters[i] = paddingH + (leadingTiles + i) * (iconSize + iconGap) + iconSize / 2`.
- `Dock.tsx`: delete `ICON_SIZE`/`ICON_GAP`/`PILL_PADDING_H`/`LEADING_TILES`; derive from
  `state.appearance`. Set the CSS variables on `.dock-stage`. Tooltip delay comes from
  appearance. Render the route-entry buttons only when the matching flag is true.
- `dock.css`: every fixed value becomes a variable with the documented default (so a bare
  browser render still looks right); add `--reflection-space` to the stage padding.
- `DockItem.tsx`: gate the reflection, running dot, and badge on appearance flags (props).
- `App.tsx` state type changes (`DockState` no longer has top-level motion fields).

- [ ] **Step 6: Update tests and mocks**

- `layout.test.ts`: CSS variable mapping, rgba conversion, centers at 48/8 with 3 leading tiles,
  and centers shift correctly at 64/12 with 2 leading tiles.
- `Dock.test.tsx`: fixture gains a complete `appearance`; add a test that a 64 px appearance
  renders `--icon-size: 64px` and that hidden route entries are absent.
- `tauri-mock.ts`: `DOCK_STATE.appearance` fixture; `get_settings` returns the camelCase view;
  `save_settings` merges into the mock settings, updates `DOCK_STATE.appearance.iconSize`
  (and any other fields the test checks), and emits `dock://state` (production does).
- `dock.spec.ts` / `panels.spec.ts`: update the save assertion from `args.settings.icon_size`
  to `args.settings.iconSize`; all other selectors stay meaningful.

- [ ] **Step 7: Run**

```bash
cd /d/dockyh/dock-tauri
npm test
D:/rust/cargo/bin/cargo.exe test --manifest-path src-tauri/Cargo.toml
```

- [ ] **Step 8: Commit**

```bash
git -C /d/dockyh add dock-tauri/src-tauri/src/settings.rs \
  dock-tauri/src-tauri/src/commands.rs dock-tauri/src-tauri/src/lib.rs \
  dock-tauri/src-tauri/src/icon_cache.rs dock-tauri/src-tauri/src/state.rs \
  dock-tauri/src-tauri/tests/unit/settings_view.rs \
  dock-tauri/src-tauri/tests/unit/dock_state_view.rs \
  dock-tauri/src-tauri/Cargo.toml dock-tauri/src/api.ts dock-tauri/src/types.ts \
  dock-tauri/src/App.tsx dock-tauri/src/dock/Dock.tsx dock-tauri/src/dock/layout.ts \
  dock-tauri/src/dock/layout.test.ts dock-tauri/src/dock/DockItem.tsx \
  dock-tauri/src/dock/dock.css dock-tauri/src/dock/Dock.test.tsx \
  dock-tauri/e2e/tauri-mock.ts dock-tauri/e2e/dock.spec.ts dock-tauri/e2e/panels.spec.ts
git -C /d/dockyh commit -m "feat: apply configurable appearance live and cache icon snapshots"
```

**Performance check after this task:** the 1 Hz loop must not stat/read icon files. Confirm by
watching the host's private bytes over 60 s with the dock idle (should be flat), and by adding a
temporary debug counter if needed — remove it before committing.

---

## Task 21: Click-through, pill-based auto-hide, fullscreen yield

**Files:**
- Create: `dock-tauri/src-tauri/src/hit_test.rs`
- Create: `dock-tauri/src-tauri/src/foreground.rs`
- Create: `dock-tauri/src-tauri/src/pointer_watch.rs`
- Modify: `dock-tauri/src-tauri/src/autohide.rs`
- Modify: `dock-tauri/src-tauri/src/commands.rs`
- Modify: `dock-tauri/src-tauri/src/lib.rs`
- Modify: `dock-tauri/src-tauri/src/platform.rs` (cursor/foreground adapters)
- Modify: `dock-tauri/src-tauri/src/settings.rs` (delays/suppression fields already exist)
- Modify: `dock-tauri/src-tauri/Cargo.toml`
- Create: `dock-tauri/src-tauri/tests/unit/hit_test.rs`
- Create: `dock-tauri/src-tauri/tests/unit/foreground.rs`
- Modify: `dock-tauri/src-tauri/tests/unit/autohide.rs`
- Modify: `dock-tauri/src-tauri/tests/unit/test_registration.rs`
- Modify: `dock-tauri/src/App.tsx` (remove pointer/tick effect)
- Modify: `dock-tauri/src/api.ts`, `dock-tauri/src/types.ts`
- Modify: `dock-tauri/src/dock/Dock.tsx` (report pill rects)
- Modify: `dock-tauri/e2e/tauri-mock.ts`

- [ ] **Step 1: Write the failing pure tests**

`hit_test.rs`:

```rust
pub struct Rect { pub left: i32, pub top: i32, pub right: i32, pub bottom: i32 }
impl Rect {
    pub fn from_origin_size(x: f64, y: f64, w: f64, h: f64) -> Self;
    pub fn contains(&self, x: i32, y: i32) -> bool;
    pub fn union(&self, other: &Rect) -> Rect;
}
pub fn physical_rect(window_pos: (i32, i32), css: Rect, scale: f64) -> Rect;
pub fn any_contains(rects: &[Rect], x: i32, y: i32) -> bool;
pub fn poll_interval_ms(pointer: (i32, i32), dock: Rect) -> u64; // 16 ms near, 100 ms far
```

Tests: pill rect contains its corners; headroom and reflection do not; a union of pill + menu
rects contains both; CSS→physical at 1.5 scale and a non-zero window origin; 16 ms within
300 px of the dock, 100 ms beyond.

`foreground.rs`:

```rust
pub fn covers_monitor(window: Rect, screen: Rect, tolerance_px: i32) -> bool;
```

Tests: a fullscreen rect covers; a maximised window whose bottom stops at the work area does
not; a rect offset by 2 px still covers with tolerance 2; a tiny window does not.

`autohide.rs` additions:
- configurable delays are used instead of the constants (`new_with_delays`);
- `Event::SetSuppressed(true)` forces `Visibility::Hidden` even when auto-hide is disabled;
- while suppressed, pointer events and ticks do nothing;
- unsuppressing with auto-hide off reveals, with auto-hide on starts hidden;
- existing tests keep passing with the default delays (300/500).

- [ ] **Step 2: Implement the hit model and watcher**

- `set_hit_rects` command:

```rust
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CssRect { pub x: f64, pub y: f64, pub width: f64, pub height: f64 }

#[tauri::command]
pub fn set_hit_rects(runtime: State<Runtime>, rects: Vec<CssRect>, scale: f64)
    -> Result<(), CommandError>;
```

  Store `Vec<Rect>` (CSS) plus scale in `Runtime.hit`. Missing rects mean "window is fully
  interactive" so the dock works for the first frames.
- `pointer_watch::spawn(app)` starts a thread after setup:
  1. Read `route`, `settings`, auto-hide machine, hit rects.
  2. `GetCursorPos`.
  3. If route != `dock`: interactive = whole window, never auto-hide.
  4. Every 500 ms, if `yield_fullscreen`, check `GetForegroundWindow` (skip our HWND and
     invisible/minimised windows) and `covers_monitor` against `primary_screen_rect()`; feed
     `SetSuppressed`.
  5. If suppressed: interactive = false always.
     Else if hidden: interactive = the collapsed window rect (the 2 px hot strip).
     Else: interactive = union of reported CSS rects converted to physical; if none, the whole
     window.
  6. Toggle `window.set_ignore_cursor_events(!interactive)` only when the value changes.
  7. When the route is dock and not suppressed, feed `PointerEntered`/`PointerLeft` on
     transitions and `Tick` every iteration; apply collapse/reveal geometry on actions.
  8. Sleep `poll_interval_ms(pointer, dock_rect)`.
- Use a monotonic `Instant` for `now_ms`, not the wall clock.
- Put the Win32 calls behind small adapters in `platform.rs`; keep the pure predicates separate.
- Add the Windows feature needed for `GetCursorPos`/`GetForegroundWindow`/`GetWindowRect`
  (`Win32_UI_WindowsAndMessaging` is already enabled) and run `cargo check`.

- [ ] **Step 3: Report rects from React**

- `Dock.tsx`: refs on the pill and on the open context menu (Task 22; for now just the pill).
  A `useLayoutEffect` measures `getBoundingClientRect()` for each and calls
  `api.setHitRects(rects, window.devicePixelRatio)`.
- Re-measure on: mount, `ResizeObserver` on the pill, the window `resize` event (auto-hide
  changes the viewport height), item count/appearance changes, and collapsed toggles.
- Collapsed pill: report the collapsed rect (control tile only), so the invisible former icons
  are click-through.
- The watcher ignores these rects while hidden and uses the hot strip instead.

- [ ] **Step 4: Remove the old frontend pointer loop**

- Delete the `route === "dock"` effect in `App.tsx` (lines ~37–90), the `AutoHideState` type,
  and the `setPointerInside`/`autohideTick`/`setAutohideEnabled` wrappers in `api.ts`.
- Remove those commands from `commands.rs` / `generate_handler!`. Keep the internal
  `apply_autohide_geometry` helper used by the watcher and `save_settings`.
- Remove their cases from `tauri-mock.ts`.
- Ensure no dead `Event::SetEnabled` path remains (settings save still uses it internally).

- [ ] **Step 5: Reapply geometry everywhere from one place**

`apply_autohide_geometry(app, runtime, visible)` and `set_route` both call the same
`route_size_for` and leading-tile count. In suppressed state, collapse to the 2 px strip as
well, but the watcher keeps hit testing disabled. Restore the previous visibility on exit.

- [ ] **Step 6: Tests**

```bash
cd /d/dockyh/dock-tauri
cargo test --manifest-path src-tauri/Cargo.toml
npm test
```

Register `hit_test` and `foreground` in `Cargo.toml`. Update `test_registration.rs` if it
enumerates commands.

- [ ] **Step 7: Manual verification (required — not replaceable by unit tests)**

With the dev app running:

1. Put a Notepad window underneath the dock. Click a point **inside the window above the pill**
   (the transparent headroom) and in the reflection band. The caret must move in Notepad; the
   dock must not react.
2. Click a dock icon. The app must launch; magnification must still work.
3. Enable auto-hide. Move the pointer from the pill to 20 px above it: the dock must hide after
   the configured delay, not stay visible until the pointer leaves the whole window.
4. Move to the bottom edge: the dock must reveal after the configured delay.
5. Open a fullscreen video (F11 / a fullscreen game). The dock must vanish immediately and
   return after exiting fullscreen.
6. Collapse the dock with the control tile: clicking the now-empty former icon area must reach
   the app behind it.

- [ ] **Step 8: Commit**

```bash
git -C /d/dockyh add dock-tauri/src-tauri/src/hit_test.rs \
  dock-tauri/src-tauri/src/foreground.rs dock-tauri/src-tauri/src/pointer_watch.rs \
  dock-tauri/src-tauri/src/autohide.rs dock-tauri/src-tauri/src/commands.rs \
  dock-tauri/src-tauri/src/lib.rs dock-tauri/src-tauri/src/platform.rs \
  dock-tauri/src-tauri/Cargo.toml dock-tauri/src-tauri/tests/unit/hit_test.rs \
  dock-tauri/src-tauri/tests/unit/foreground.rs \
  dock-tauri/src-tauri/tests/unit/autohide.rs \
  dock-tauri/src-tauri/tests/unit/test_registration.rs \
  dock-tauri/src/App.tsx dock-tauri/src/api.ts dock-tauri/src/types.ts \
  dock-tauri/src/dock/Dock.tsx dock-tauri/e2e/tauri-mock.ts
git -C /d/dockyh commit -m "fix: pass clicks through the dock margins and scope auto-hide to the pill"
```

---

## Task 22: The settings page, app management, context menu

**Files:**
- Rewrite: `dock-tauri/src/settings/SettingsPanel.tsx`
- Rewrite: `dock-tauri/src/settings/validate.ts` → keep the filename, export the new schema
- Rewrite: `dock-tauri/src/settings/settings.css`
- Create: `dock-tauri/src/settings/fields.tsx` (label + range + output primitives)
- Create: `dock-tauri/src/settings/preview.tsx`
- Create: `dock-tauri/src/dock/ContextMenu.tsx`
- Modify: `dock-tauri/src/dock/Dock.tsx`, `dock-tauri/src/dock/DockItem.tsx`
- Modify: `dock-tauri/src/App.tsx` (catalog lazy load, pin/unpin plumbing, settings props)
- Modify: `dock-tauri/src/launchpad/Launchpad.tsx` (surface pin errors)
- Modify: `dock-tauri/src/settings/SettingsPanel.test.tsx`, `validate.test.ts`
- Create: `dock-tauri/src/settings/preview.test.tsx`
- Modify: `dock-tauri/e2e/panels.spec.ts`, `tauri-mock.ts`
- Modify: `dock-tauri/src-tauri/src/commands.rs` (+ `reorder_pins`)
- Modify: `dock-tauri/src-tauri/src/state.rs` (pure order validation)
- Create: `dock-tauri/src-tauri/tests/unit/pins.rs`
- Modify: `dock-tauri/src-tauri/Cargo.toml`

- [ ] **Step 1: Add `reorder_pins` with a pure validator**

```rust
/// `requested` must be a permutation of `current`; otherwise reject rather than
/// silently dropping or duplicating a pinned app.
pub fn apply_order(current: &[String], requested: &[String]) -> Result<Vec<String>, String>;
```

Tests in `pins.rs`: valid permutation; missing id; unknown id; duplicate id; empty dock.
The command persists, updates `Runtime.pinned`, emits `dock://state`; `pin_app`/`unpin_app`
also emit after success so every panel updates without bespoke refreshes.

- [ ] **Step 2: Build the settings form schema**

`validate.ts` exports:

```ts
export interface SettingsForm { /* all SettingsView camelCase fields except pins */ }
export const DEFAULT_SETTINGS: SettingsForm;
export function validateSettings(form: SettingsForm): ValidationResult;
export function toSettingsPayload(form: SettingsForm): Record<string, unknown>;
```

Table-driven ranges identical to the Rust table. Error strings mention the field name
(`"iconSize must be between 32 and 80"`), because the existing tests and E2E assert on names.
Also validate `tint` against `/^#[0-9a-f]{6}$/i` and `profile` against the two values.

Rewrite `validate.test.ts` to cover every field: accept defaults; reject below/above for each
numeric; reject bad tint/profile; accept boundary values.

- [ ] **Step 3: Build the tabbed panel with live apply**

- Header: `Settings`, status (`Saving…` / `Saved.` / error alert), `Done`.
- Tabs (`role="tablist"`, each `role="tab"`, panel `role="tabpanel"`):
  - **Appearance** — icon size, icon gap, padding H/V, pill radius, glass opacity, tint
    (color input), icon radius, reflection + strength, and the live `DockPreview`.
  - **Behaviour** — magnification + peak + reach, motion profile, reduced motion, tooltips +
    delay, running dots, minimized badges, auto-hide + reveal/hide delays, fullscreen yield,
    Launchpad entry, monitor entry.
  - **Apps** — pinned list (move up/down, remove) and a searchable catalog with Add buttons.
    System tiles (`kind === "system"`) cannot be removed; move is allowed. Show
    `10` as the dock cap and disable Add when full.
  - **Advanced** — reset appearance/behaviour to defaults (keep pins), the known settings file
    location (`%APPDATA%\DockApp\settings.json`), and a note that the glass is painted (no
    desktop blur yet).
- On open, `get_settings`; populate the form; do not save while loading. The Apps tab separately
  loads `get_dock_state` (for the pinned order) and `get_catalog` (for the add list) when it is
  first opened, so the dock's initial paint does not pay for the catalog.
- On change: update local form (preview updates immediately), then debounce 300 ms:
  validate → `save_settings(toSettingsPayload(form))` → status. Invalid forms show the alert and
  do not call the backend. Flush on `Done`/unmount.
- Keep labels accessible: `htmlFor`/id pairs; tests use `getByLabelText(/icon size/i)`.
- `DockPreview` renders five fixture tiles using the same `cssVars()` helper from `dock/layout`.
  It must reflect size, gap, padding, radius, tint, opacity, icon radius, and reflection live.

- [ ] **Step 4: Dock right-click menu**

- `ContextMenu.tsx`: fixed-position menu at the pointer, clamped into the viewport, closes on
  Escape, outside pointerdown, or action; `role="menu"` / `role="menuitem"`.
- `DockItem` gets `onContextMenu(id, x, y)`; `Dock` owns the menu state and passes
  `onRemove`/`onOpen`.
- Entries: **Open**, **Remove from Dock** (disabled for system tiles). Remove calls
  `api.unpinApp(id)`; the backend emits the new state, so no manual refresh is required.
- Include the menu rect in the reported hit rects while it is open (union with the pill), so its
  clicks are not passed through.
- Update `Dock.test.tsx` / add a context-menu test that the accessible menu appears and calls
  the handler; E2E can right-click and assert `unpin_app`.

- [ ] **Step 5: Launchpad pin feedback**

- `App.addToDock` catches errors and passes a `pinError` to Launchpad; Launchpad renders it in
  the existing alert region. Clear on the next successful add.

- [ ] **Step 6: Settings tests**

- `SettingsPanel.test.tsx`: loading shows defaults; changing icon size triggers one debounced
  `save_settings` with `iconSize`; an out-of-range icon size (same DOM trick as the old test)
  shows an alert and never saves; tabs switch panels.
- `preview.test.tsx`: changing `fillOpacity` updates `--glass-fill` on the preview.
- E2E (`panels.spec.ts`):
  1. Settings opens; Appearance is the default tab; icon-size slider is visible.
  2. Change icon size → wait for `Saved.` → assert `save_settings` carried `iconSize`.
  3. Invalid size → alert, no save.
  4. Apps tab: search "Alpha", Add → `pin_app` called with `cat-alpha`; Remove → `unpin_app`.
  5. Switching to Behaviour and toggling auto-hide persists (`autohide: true`).
  6. Right-click a dock tile → menu → Remove → `unpin_app` called.
  7. Existing Launchpad/monitor tests still pass.
- Update `tauri-mock.ts`: implement `pin_app`/`unpin_app`/`reorder_pins` against a pinned list,
  emit the updated state, and keep `__MOCK_CALLS__` assertions stable.

- [ ] **Step 7: Run**

```bash
cd /d/dockyh/dock-tauri
npm test
npm run test:e2e
D:/rust/cargo/bin/cargo.exe test --manifest-path src-tauri/Cargo.toml
```

- [ ] **Step 8: Commit**

```bash
git -C /d/dockyh add dock-tauri/src/settings dock-tauri/src/dock/ContextMenu.tsx \
  dock-tauri/src/dock/Dock.tsx dock-tauri/src/dock/DockItem.tsx dock-tauri/src/App.tsx \
  dock-tauri/src/launchpad/Launchpad.tsx dock-tauri/src-tauri/src/commands.rs \
  dock-tauri/src-tauri/src/state.rs dock-tauri/src-tauri/tests/unit/pins.rs \
  dock-tauri/src-tauri/Cargo.toml dock-tauri/e2e/panels.spec.ts \
  dock-tauri/e2e/tauri-mock.ts dock-tauri/e2e/dock.spec.ts
git -C /d/dockyh commit -m "feat: add a live settings page and dock item management"
```

---

## Task 23: macOS-like visual pass

**Files:**
- Modify: `dock-tauri/src/dock/DockItem.tsx`
- Modify: `dock-tauri/src/dock/Dock.tsx`
- Modify: `dock-tauri/src/dock/ControlTile.tsx` (markup only if the wrapper changes)
- Modify: `dock-tauri/src/dock/Tooltip.tsx`
- Modify: `dock-tauri/src/dock/dock.css`
- Modify: `dock-tauri/src/dock/Dock.test.tsx`
- Modify: `dock-tauri/src/dock/magnify.ts` / `magnify.test.ts` (only if the interpolation changes)

- [ ] **Step 1: Restructure the icon markup**

Wrap the icon, reflection, dot, and badge in a dedicated `.icon-art` span inside the button.
The button keeps the magnify transform (React inline style); `.icon-art` owns the shadow, mask,
and press animation so the two transforms do not overwrite each other.

- [ ] **Step 2: Apply the visual rules**

- **Icons**: `border-radius: var(--icon-radius)`, `filter: drop-shadow(0 3px 5px rgba(0,0,0,.35))`,
  no inner background for real icons; letter fallback keeps the tinted plate.
- **Reflection**: height `calc(var(--icon-size) * 0.67)`, `opacity: var(--reflection-opacity)`,
  mask gradient, hidden entirely when `reflection` is false. Do not render the second `<img>`
  when disabled (saves a decode).
- **Running dot**: 4 px, white with a dark ring, centred under the icon; hidden when disabled.
- **Badge**: keep the red pill, hidden when disabled; never scales with the icon.
- **Tooltip**: slightly larger radius, subtle border, shadow, and the configured delay;
  `pointer-events: none`.
- **Glass**: `--glass-fill` from tint+opacity, inset top highlight, inset bottom shade, grain
  layer; no outer stroke. Keep the painted material.
- **Press feedback**: `.icon-art { transition: transform 80ms ease }`;
  `.dock-item:active .icon-art { transform: scale(0.9) }`.
- **Launch feedback**: `Dock` tracks `launchingId` for 350 ms after a successful click; the tile
  gets a short opacity/brightness pulse. A failed launch shows an inline error toast above the
  pill (role `alert`) for 3 s instead of being silently ignored.
- **Magnification feel**: while magnifying use `transition: transform 45ms linear`; on leave use
  the existing spring. Keep the quartic Gaussian and the profile peaks.
- **Reduced motion**: no transform transitions; press feedback becomes opacity only.

- [ ] **Step 3: Tests**

- `Dock.test.tsx`: reflection `<img>` absent when disabled; running dot absent when disabled;
  badge absent when disabled; tooltip respects `tooltipDelayMs` (fake timers); clicking a tile
  sets the launching class then clears it.
- `magnify.test.ts`: unchanged peak/falloff properties; add a test that a larger radius spreads
  the curve as expected when the reactant uses the computed `magnifyRadius`.

- [ ] **Step 4: Manual visual checklist**

Compare against a macOS Big Sur+ dock screenshot at 150 % DPI:

1. Icons crisp at 48 px (not soft), with a soft drop shadow and rounded mask.
2. Reflection visible but subtle; toggling it off removes the strip and the reserved margin.
3. Running dots sit under icons, not inside them; badges stay inside the pill bounds.
4. Tooltip appears above the magnified icon without clipping at peak 2.0.
5. Pressing an icon visibly depresses it; launch gives immediate feedback.
6. Glass opacity/tint changes are visible immediately in the settings preview and on the dock.
7. Reduced motion removes scaling without breaking layout.

- [ ] **Step 5: Commit**

```bash
git -C /d/dockyh add dock-tauri/src/dock
git -C /d/dockyh commit -m "feat: polish icon, reflection, and dock material rendering"
```

---

## Task 24: Full verification, performance, README, release

**Files:**
- Modify: `dock-tauri/README.md`
- Modify: `docs/superpowers/specs/2026-10-26-tauri-python-dock-design.md` (append the Phase 3
  settings section so documentation and implementation agree)
- Possibly modify: `dock-tauri/scripts/measure_memory.ps1` (only if sampling changes)

- [ ] **Step 1: Run every suite**

```bash
cd /d/dockyh/dock-tauri
D:/rust/cargo/bin/cargo.exe test --manifest-path src-tauri/Cargo.toml
npm test
npm run test:e2e
cd backend && D:/jiqixuexi/anaconda/python.exe -m pytest tests -q
```

Expected: all green. Record the exact counts for the README.

- [ ] **Step 2: Build and measure**

```bash
export CARGO_HOME='D:\rust\cargo' RUSTUP_HOME='D:\rust\rustup'
export PATH="/d/rust/cargo/bin:$PATH"
cd /d/dockyh/dock-tauri
npm run tauri build
powershell -ExecutionPolicy Bypass -File scripts/measure_memory.ps1 -SettleSeconds 60 -Runs 3
```

- Use `npm run tauri build` (never bare `cargo build --release`).
- The frozen sidecar must exist at `src-tauri/binaries/monitor-x86_64-pc-windows-msvc.exe`;
  regenerate with `backend/build_sidecar.ps1` if missing. Do not commit it.
- **Do not install the MSI.**
- Both gates must pass: host < 30 MB, host + WebView2 < 200 MB. If the new 60 Hz watcher pushes
  idle CPU above 1 %, raise the far poll interval to 150 ms before changing anything else.

- [ ] **Step 3: Final live checklist**

1. Restart with the release binary; icons sharp at 150 % scaling.
2. Click-through above/around the pill; dock interaction unaffected.
3. Auto-hide reveal/hide distances and delays behave as configured; fullscreen yield works.
4. Settings: change icon size, gap, padding, radius, tint, opacity, magnify peak, auto-hide
   delays; close and reopen the panel — values persisted and dock updated.
5. Add/remove/reorder apps from settings; remove from the dock context menu; Launchpad add.
6. Monitor opens fast on first use (sidecar pre-warmed) and samples live.
7. Launchpad catalog opens without a startup delay.
8. Memory/CPU within gates after 60 s idle.
9. Restore any setting changed only for testing, except auto-hide: leave it enabled because it
   is the owner's requested default behaviour.

- [ ] **Step 4: Update the README**

Replace stale claims with the real state:
- Settings section: the full table (or a grouped summary) and live-apply behavior.
- Auto-hide: default on for new installs, pill-based, fullscreen yield, configurable delays.
- Tests: Rust count, Vitest count, **Playwright E2E now exists**, pytest count.
- Packaging: sidecar is frozen and bundled; `npm run tauri build` produces the MSI; mention
  the sidecar regeneration prerequisite.
- Known limitations: painted glass (no desktop blur; T18 decision still open), 10-item dock cap,
  drag reorder/Launchpad drag/Shelf/tray/Docklet/MSIX still deferred, store apps identity.
- Architecture: add `appearance.rs`, `hit_test.rs`, `pointer_watch.rs`, `foreground.rs`, and the
  settings page modules.
- Point to this Phase 3 plan.

- [ ] **Step 5: Commit**

```bash
git -C /d/dockyh add dock-tauri/README.md \
  docs/superpowers/specs/2026-10-26-tauri-python-dock-design.md \
  docs/superpowers/plans/2026-10-26-dock-tauri-phase3.md
git -C /d/dockyh commit -m "docs: document configurable appearance and phase 3 behaviour"
```

---

## Risk notes

- **Subclassing is avoided.** `set_ignore_cursor_events` is a documented Tauri window API; the
  watcher only toggles it, so there is no custom WndProc to fight tao's message handling.
- **Fast clicks at the pill boundary.** Polling at 16 ms means a click in the first frame after
  entering the pill could pass through. If manual testing shows this, shrink the toggle threshold
  to a 4 px expanded interactive rect (do not extend into the headroom), rather than polling
  faster.
- **Settings writes.** Debouncing bounds writes to ~3/s worst case; `save_atomic` is fsync'd but
  the file is under 2 KB. If profiling shows stalls, persist on pointer-up/change-end instead of
  every debounce, keeping preview live.
- **DPI changes.** The pill rect is reported with `devicePixelRatio`; moving the dock between
  monitors is out of scope (the dock is anchored to the primary monitor), so a DPI change
  without a resize is not expected.
- **Icon re-extraction on size change.** The first change at a new DPI/size re-extracts the
  whole catalog once; the snapshot then serves every tick. Keep the cap (10) in mind if this is
  ever raised.
- **Acrylic.** Do not attempt `apply_acrylic` in this phase. The entire-window blur problem in
  T18 is unchanged, and the settings page's opacity/tint controls are the supported material
  customization until the user picks one of T18's options.

## Deferred to a later phase (unchanged)

Drag-to-reorder on the dock, drag-from-Launchpad, Shelf, tray mirroring, Docklet plugins,
custom icon files, MSIX/AUMID identity, left/right/top edges, real desktop blur.
