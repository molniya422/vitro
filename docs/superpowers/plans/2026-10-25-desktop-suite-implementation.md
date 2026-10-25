# Nexus + BitDock Desktop Suite Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `subagent-driven-development` (recommended) or `executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build an independent Windows desktop enhancement suite under `D:\dockyh\dock-app` with a real Dock, Launchpad, Shelf, window/task management, controlled system surface, permissioned Docklets, settings, and native macOS-style material and motion.

**Architecture:** The product is a new Rust workspace with pure `dock-core` domain logic, a typed `windows-platform` adapter, a native Win32/Direct2D/DirectComposition UI, a sandboxed `docklet-host`, and a small `shell-runtime` composition root. Glassboard remains a separate product, and Vitro is used only as a legacy reference for isolated algorithms and artwork; no Vitro or GlassDesk runtime code is compiled into this suite.

**Tech Stack:** Rust stable installed on `D:\rust` (`CARGO_HOME=D:\rust\cargo`, `RUSTUP_HOME=D:\rust\rustup`), Win32 APIs, Direct2D, DirectWrite, DirectComposition, DWM backdrop APIs, Windows UI Automation for acceptance checks, TOML/JSON schema validation, QuickJS or an isolated helper process for Docklets, and Rust unit/integration tests.

---

## Source and boundary map

The implementation must use these documents as normative inputs, reading their full text before changing code:

- `D:\dockyh\docs\research\macos-dock-spec.md` — geometry, material, hit testing, auto-hide, and macOS parity behavior.
- `D:\dockyh\docs\research\nexus-dock-behavior-spec.md` — Nexus R1–R12 interaction acceptance.
- `D:\dockyh\docs\research\mydockfinder-bitdock-deepdive.md` — launcher, shelf, task, tray, and Docklet behavior.
- `D:\dockyh\docs\research\plugin-design-language.md` — Docklet visual forms and extension contract.
- `D:\dockyh\docs\research\design-language-research.md`, `frontend-design-research.md`, `skillry-premium-design-research.md`, and `tech-stack-research.md` — visual and implementation constraints.
- `D:\dockyh\docs\specs\effects-catalog.toml`, `motion-tokens.toml`, and `macos-parity.toml` — machine-readable effect, motion, and material tokens.
- `D:\dockyh\docs\superpowers\specs\2026-10-25-desktop-suite-design.md` — product boundary and architecture contract.
- `D:\dockyh\docs\research\glassdesk-vitro-audit.md` — permitted reference extraction and licensing boundary.

The approved asset source is the inspected artwork under `D:\dockyh\vitro\assets\icons`; each asset must be inventoried, re-keyed, and copied into the new product with provenance recorded. The HTML prototypes are behavior and visual references only. No DOM runtime, old Vitro window/state code, GlassDesk code, or BitDock/Nexus code is imported.

The new product tree is fixed before implementation:

```text
D:\dockyh\dock-app\
  Cargo.toml
  Cargo.lock
  src\lib.rs
  src\main.rs
  src\core\{mod.rs,model.rs,layout.rs,reducer.rs,persistence.rs,menus.rs,errors.rs}
  src\platform\windows\{mod.rs,monitor.rs,dpi.rs,app_catalog.rs,window_tracker.rs,
    window_activation.rs,icons.rs,dragdrop.rs,recycle_bin.rs,tray.rs,hotzones.rs,taskbar.rs}
  src\ui\{mod.rs,window.rs,renderer.rs,material.rs,geometry.rs,hit_test.rs,
    animation.rs,menus.rs,flyouts.rs,preview.rs,accessibility.rs}
  src\docklets\{mod.rs,manifest.rs,permissions.rs,installer.rs,host.rs,sandbox.rs,
    scheduler.rs,builtins.rs}
  src\runtime\{mod.rs,shell.rs,settings.rs,recovery.rs,logging.rs}
  assets\icons\, assets\noise\
  tests\unit\, tests\platform\, tests\desktop\, tests\visual\, tests\perf\
  schemas\docklet-manifest.schema.json
```

## Task 1: Create the isolated Rust workspace

**Files:**
- Create: `D:\dockyh\dock-app\Cargo.toml`
- Create: `D:\dockyh\dock-app\src\main.rs`
- Create: `D:\dockyh\dock-app\src\core\mod.rs`
- Create: `D:\dockyh\dock-app\src\platform\windows\mod.rs`
- Create: `D:\dockyh\dock-app\src\ui\mod.rs`
- Create: `D:\dockyh\dock-app\src\docklets\mod.rs`
- Create: `D:\dockyh\dock-app\src\runtime\mod.rs`
- Test: `D:\dockyh\dock-app\tests\unit\workspace_smoke.rs`

- [ ] **Step 1: Write the failing workspace test**

```rust
#[test]
fn workspace_exposes_product_modules() {
    assert_eq!(dock_app::product_name(), "Dock App");
}
```

- [ ] **Step 2: Run it to verify it fails**

Run from PowerShell:

```powershell
$env:CARGO_HOME='D:\rust\cargo'; $env:RUSTUP_HOME='D:\rust\rustup'
& D:\rust\cargo\bin\cargo.exe test --manifest-path D:\dockyh\dock-app\Cargo.toml workspace_exposes_product_modules
```

Expected: FAIL because the new crate and `product_name` do not exist.

- [ ] **Step 3: Write the minimal workspace**

```toml
[package]
name = "dock-app"
version = "0.1.0"
edition = "2021"

[lib]
name = "dock_app"
path = "src/lib.rs"

[[bin]]
name = "dock-app"
path = "src/main.rs"

[dependencies]
serde = { version = "1", features = ["derive"] }
toml = "0.8"
thiserror = "2"
uuid = { version = "1", features = ["serde", "v4"] }
windows = { version = "0.61", features = ["Win32_Foundation", "Win32_Graphics_Direct2D", "Win32_Graphics_DirectWrite", "Win32_Graphics_Dxgi", "Win32_System_Com", "Win32_UI_WindowsAndMessaging"] }
```

```rust
// src/lib.rs
pub fn product_name() -> &'static str { "Dock App" }
pub mod core;
pub mod docklets;
pub mod platform;
pub mod runtime;
pub mod ui;
```

- [ ] **Step 4: Run the test and commit**

Run `cargo test --manifest-path D:\dockyh\dock-app\Cargo.toml`. Expected: PASS with the empty modules compiling. Commit with `git add dock-app && git commit -m "feat: create independent dock app workspace"`.

## Task 2: Implement `dock-core` domain types and persistence

**Files:**
- Create: `src/core/model.rs`, `src/core/layout.rs`, `src/core/reducer.rs`, `src/core/persistence.rs`, `src/core/menus.rs`, `src/core/errors.rs`
- Modify: `src/core/mod.rs`
- Test: `tests/unit/core_model.rs`, `tests/unit/core_reducer.rs`, `tests/unit/persistence.rs`

- [ ] **Step 1: Write failing tests for identity, layout, commands, and recovery**

```rust
#[test]
fn reorder_is_stable_and_rejects_unknown_item() {
    let mut state = fixture_state();
    assert!(state.apply(Command::Move { id: item("mail"), before: Some(item("finder")) }).is_ok());
    assert_eq!(state.layout.ids(), [item("mail"), item("finder")]);
    assert!(state.apply(Command::Remove { id: item("missing") }).is_err());
}

#[test]
fn interrupted_write_restores_last_valid_snapshot() {
    let store = MemoryStore::with_bytes(br"broken");
    let state = load_or_recover(&store).unwrap();
    assert_eq!(state.schema_version, CURRENT_SCHEMA);
}
```

- [ ] **Step 2: Run the focused tests and verify failure**

Run `cargo test --manifest-path D:\dockyh\dock-app\Cargo.toml core_model core_reducer persistence`. Expected: FAIL with missing domain types.

- [ ] **Step 3: Implement pure types and atomic schema**

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ItemId(pub uuid::Uuid);

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum DockItem { App(AppItem), Window(WindowItem), Folder(FolderItem), Stack(StackItem), Docklet(DockletItem), Separator, System(SystemItem) }

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PersistedState { pub schema_version: u32, pub layouts: Vec<DockLayout>, pub settings: SettingsSnapshot, pub docklets: Vec<DockletRecord> }

pub const CURRENT_SCHEMA: u32 = 1;
```

`save_atomic` must write a sibling `.tmp`, flush and close it, rename the old file to `.bak`, rename the temp file into place, and fsync the directory where Windows permits it. `load_or_recover` must validate the schema and fall back to `.bak` before returning a typed error. The reducer must model launch, activate, minimize, close, drag, menu, auto-hide, and Docklet lifecycle commands without importing Windows types.

- [ ] **Step 4: Run all core tests and commit**

Run `cargo test --manifest-path D:\dockyh\dock-app\Cargo.toml core_`. Expected: PASS, including migration and rollback cases. Commit `feat: add dock core model reducer and persistence`.

## Task 3: Build the typed Windows platform layer

**Files:**
- Create: `src/platform/windows/monitor.rs`, `dpi.rs`, `app_catalog.rs`, `window_tracker.rs`, `window_activation.rs`, `icons.rs`, `dragdrop.rs`, `recycle_bin.rs`, `tray.rs`, `hotzones.rs`, `taskbar.rs`
- Modify: `src/platform/windows/mod.rs`
- Test: `tests/platform/app_identity.rs`, `tests/platform/monitor_layout.rs`, `tests/platform/window_events.rs`, `tests/platform/drop_target.rs`

- [ ] **Step 1: Write failing adapter-contract tests**

```rust
#[test]
fn monitor_snapshot_uses_work_area_and_dpi() {
    let snapshot = fake_platform().monitors()[0];
    assert_eq!(snapshot.work_area, Rect::new(0, 0, 1920, 1040));
    assert_eq!(snapshot.dpi, 144);
}

#[test]
fn app_identity_survives_window_reconciliation() {
    let app = AppIdentity::from_aumid("Microsoft.WindowsCalculator_8wekyb3d8bbwe!App");
    assert_eq!(reconcile_window(app.clone(), hwnd(7)).app, app);
}
```

- [ ] **Step 2: Run tests and verify failure**

Run `cargo test --manifest-path D:\dockyh\dock-app\Cargo.toml --test app_identity --test monitor_layout --test window_events --test drop_target`. Expected: FAIL because platform adapters are absent.

- [ ] **Step 3: Implement adapters with typed failures**

Implement `MonitorProvider`, `AppCatalog`, `WindowTracker`, `WindowActions`, `DropTarget`, `TrayProvider`, and `TaskbarProvider` traits. Use `EnumWindows` plus WinEvent hooks for event-driven updates, and a bounded 150 ms reconciliation debounce after bursts. Use AUMID/path identity rather than click-created state. Icon extraction must try requested DPI, Shell image list, then the rebuilt asset map. OLE drop targets must accept only declared targets, and recycle-bin deletion must return a failure event when `SHFileOperation`/`IFileOperation` fails.

- [ ] **Step 4: Verify on Windows and commit**

Run `cargo test --manifest-path D:\dockyh\dock-app\Cargo.toml --all-targets`, then run the platform smoke command `cargo run --manifest-path D:\dockyh\dock-app\Cargo.toml -- --diagnose-platform` and verify monitor count, work areas, DPI, taskbar bounds, and at least one top-level window. Commit `feat: add typed windows platform adapters`.

## Task 4: Create native windows, shared geometry, and accessibility surfaces

**Files:**
- Create: `src/ui/window.rs`, `renderer.rs`, `geometry.rs`, `hit_test.rs`, `accessibility.rs`
- Modify: `src/ui/mod.rs`, `src/main.rs`
- Test: `tests/unit/geometry.rs`, `tests/unit/hit_test.rs`, `tests/desktop/window_surface.rs`

- [ ] **Step 1: Write geometry and hit-test tests before Win32 code**

```rust
#[test]
fn dock_geometry_is_shared_by_paint_and_hit_test() {
    let g = DockGeometry::new(Rect::new(0, 0, 1440, 900), Edge::Bottom, 56.0, 1.0);
    let icon = g.icon_rect(2);
    assert!(g.hit_test(icon.center()).is_item(2));
    assert_eq!(g.corner_radius(), 28.0);
}
```

- [ ] **Step 2: Run the focused tests and verify failure**

Run `cargo test --manifest-path D:\dockyh\dock-app\Cargo.toml geometry hit_test`. Expected: FAIL because layout and hit-test functions are missing.

- [ ] **Step 3: Implement the native surface**

Create a per-monitor borderless, transparent, always-on-top window with `WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE`, a DComp visual and D2D target. `DockGeometry` is the single source for paint bounds, hit bounds, tooltip anchor, drag insertion marker, and accessibility rectangles. The window must remain below the taskbar work-area reservation and never replace Explorer.

- [ ] **Step 4: Verify UIA and commit**

Run unit tests, then launch `cargo run --manifest-path D:\dockyh\dock-app\Cargo.toml -- --diagnose-ui`. Use Windows UI Automation to enumerate the Dock surface and verify item names, invoke patterns, keyboard focus, and screen rectangles. Commit `feat: add native dock surface and shared geometry`.

## Task 5: Implement material, motion, and the 11-slot effects engine

**Files:**
- Create: `src/ui/material.rs`, `src/ui/animation.rs`, `assets/noise/noise-256.png`
- Modify: `src/ui/renderer.rs`, `src/runtime/settings.rs`
- Test: `tests/unit/material_tokens.rs`, `tests/unit/effects.rs`, `tests/visual/effects_golden.rs`

- [ ] **Step 1: Write failing token and guard tests**

```rust
#[test]
fn reduced_motion_disables_elastic_overshoot() {
    let profile = MotionProfile::from(Settings { reduced_motion: true, ..Default::default() });
    assert_eq!(profile.peak("magnify"), 1.0);
    assert_eq!(profile.duration_ms("launch"), 200);
}

#[test]
fn one_element_cannot_run_two_effects_in_one_slot() {
    let mut guard = EffectGuard::default();
    assert!(guard.start("item-1", EffectSlot::Magnify).is_ok());
    assert!(guard.start("item-1", EffectSlot::Magnify).is_err());
}
```

- [ ] **Step 2: Run tests and verify failure**

Run `cargo test --manifest-path D:\dockyh\dock-app\Cargo.toml material_tokens effects`. Expected: FAIL because the token loader and effect guard do not exist.

- [ ] **Step 3: Implement token loading and rendering**

Load `effects-catalog.toml`, `motion-tokens.toml`, and `macos-parity.toml` at build time and validate all 11 effect slots and 44 presets: magnify, launch, attention, drawer, menu, tooltip, reflection, icon tilt, confirm, badge counter, and dock enter. Implement the macOS restrained profile and Vitro elastic profile behind settings, then apply guards in this order: reduced motion, battery/performance mode, user profile, one active effect per element/slot.

Use Sonoma/Tahoe material tokens: blur 20–28, theme fill 9–25%, saturation lift 8–10%, noise alpha 0.25–0.3%, inner highlight/shadow, no external outline or heavy external shadow, 4 px grid, 4 px running point, and tooltip/badge offsets from the research documents. Geometry must stay identical when effects are disabled.

- [ ] **Step 4: Run golden and performance checks and commit**

Run `cargo test --manifest-path D:\dockyh\dock-app\Cargo.toml --test effects_golden` and compare generated PNGs against checked-in goldens at 100%, 125%, and 150% DPI. Commit `feat: add native material and guarded effects engine`.

## Task 6: Complete Dock Shell behavior

**Files:**
- Modify: `src/core/reducer.rs`, `src/platform/windows/hotzones.rs`, `src/ui/window.rs`, `src/ui/renderer.rs`
- Create: `src/ui/menus.rs`, `src/ui/flyouts.rs`
- Test: `tests/desktop/dock_shell.rs`, `tests/unit/autohide.rs`, `tests/unit/reorder.rs`

- [ ] **Step 1: Write failing R1–R12 interaction tests**

```rust
#[test]
fn autohide_reveals_from_edge_and_hides_after_pointer_leaves() {
    let mut state = fixture_shell(AutoHideMode::Edge);
    state.pointer_at(Edge::Bottom, 400);
    assert_eq!(state.visibility(), Visibility::Revealing);
    state.advance_ms(300);
    assert_eq!(state.visibility(), Visibility::Visible);
    state.pointer_at(Rect::new(900, 100, 901, 101));
    state.advance_ms(500);
    assert_eq!(state.visibility(), Visibility::Hidden);
}
```

- [ ] **Step 2: Run the focused tests and verify failure**

Run `cargo test --manifest-path D:\dockyh\dock-app\Cargo.toml autohide reorder dock_shell`. Expected: FAIL until the shell state machine exists.

- [ ] **Step 3: Implement shell actions**

Cover bottom/left/right/top placement, per-monitor work area and DPI, control tile single/double click, parabolic magnification, tooltip delay, launch bounce, running indicators, badges, data-driven item/blank menus, drag reorder, Explorer file drops, recycle-bin delete with rollback, sub-Dock entry/exit, edge/corner/swipe/hotspot reveal, manual lock, and taskbar coexistence. Auto-hide timers must be cancellable and state-driven so a failed timer cannot leave the Dock permanently visible.

- [ ] **Step 4: Run desktop acceptance tests and commit**

Run `cargo test --manifest-path D:\dockyh\dock-app\Cargo.toml --test dock_shell --test autohide --test reorder`, then execute `cargo run --manifest-path D:\dockyh\dock-app\Cargo.toml -- --acceptance dock-shell`. Commit `feat: implement dock shell interactions and autohide`.

## Task 7: Add Launchpad and Shelf

**Files:**
- Create: `src/core/launcher.rs`, `src/core/shelf.rs`, `src/ui/launchpad.rs`, `src/ui/shelf.rs`
- Modify: `src/core/mod.rs`, `src/ui/mod.rs`, `src/platform/windows/app_catalog.rs`, `src/platform/windows/dragdrop.rs`
- Test: `tests/unit/launcher.rs`, `tests/unit/shelf.rs`, `tests/desktop/launchpad_shelf.rs`

- [ ] **Step 1: Write failing search, stack, and drop tests**

```rust
#[test]
fn launcher_search_handles_missing_icons_and_paginates() {
    let results = catalog().search("calc");
    assert_eq!(results[0].identity.display_name, "Calculator");
    assert!(results[0].icon.is_fallback_or_loaded());
}

#[test]
fn stack_round_trips_fan_grid_and_list_modes() {
    for mode in [StackMode::Fan, StackMode::Grid, StackMode::List] {
        assert_eq!(decode(encode(stack_fixture(mode))).mode, mode);
    }
}
```

- [ ] **Step 2: Run tests and verify failure**

Run `cargo test --manifest-path D:\dockyh\dock-app\Cargo.toml launcher shelf`. Expected: FAIL because the launcher and shelf models are absent.

- [ ] **Step 3: Implement real sources and actions**

Scan Start Menu and registered app identities, preserve stable AUMID/path identity, support recent apps, categories, search, empty/loading/error states, and drag-to-Dock. Shelf must manage files, folders, downloads, recent documents, and stacks with fan/grid/list views. Dragging a file over an app invokes open-with; dragging to Recycle Bin reports deletion errors.

- [ ] **Step 4: Verify and commit**

Run focused tests and `cargo run --manifest-path D:\dockyh\dock-app\Cargo.toml -- --acceptance launcher-shelf`. Commit `feat: add launchpad and shelf modules`.

## Task 8: Add windows, tasks, previews, tray, clock, and shortcuts

**Files:**
- Create: `src/ui/preview.rs`, `src/ui/accessibility.rs`, `src/core/tasks.rs`, `src/core/tray.rs`
- Modify: `src/platform/windows/window_tracker.rs`, `src/platform/windows/window_activation.rs`, `src/platform/windows/tray.rs`, `src/ui/menus.rs`
- Test: `tests/platform/window_actions.rs`, `tests/desktop/window_preview.rs`, `tests/desktop/tray_surface.rs`, `tests/unit/shortcuts.rs`

- [ ] **Step 1: Write failing window and tray tests**

```rust
#[test]
fn clicking_running_app_activates_matching_window() {
    let action = choose_app_click(app_with_windows(2), ClickKind::Single);
    assert_eq!(action, AppAction::Activate(WindowId(2)));
}

#[test]
fn unsupported_tray_action_is_explicitly_unavailable() {
    assert!(tray_item("legacy-host").action(Action::Open).is_unavailable());
}
```

- [ ] **Step 2: Run tests and verify failure**

Run `cargo test --manifest-path D:\dockyh\dock-app\Cargo.toml window_actions window_preview tray_surface shortcuts`. Expected: FAIL until snapshot-to-action mapping exists.

- [ ] **Step 3: Implement snapshot-driven controls**

Implement activation, restore, minimize, close, force-close where permitted, per-application window menus, thumbnail previews, workspace/monitor association, disappearing-window notifications, controlled tray listing, clock, keyboard shortcuts, and taskbar coexistence controls. Unsupported tray operations must render disabled state with an explanation instead of a fake button.

- [ ] **Step 4: Verify UIA and commit**

Run desktop tests and use UI Automation to confirm labels, invoke actions, keyboard navigation, and preview close behavior. Commit `feat: add window task tray and accessibility surfaces`.

## Task 9: Implement Docklet manifest, permissions, sandbox, and built-ins

**Files:**
- Create: `schemas/docklet-manifest.schema.json`, `src/docklets/manifest.rs`, `permissions.rs`, `installer.rs`, `host.rs`, `sandbox.rs`, `scheduler.rs`, `builtins.rs`
- Modify: `src/core/model.rs`, `src/runtime/settings.rs`
- Test: `tests/unit/docklet_manifest.rs`, `tests/unit/docklet_permissions.rs`, `tests/desktop/docklet_isolation.rs`

- [ ] **Step 1: Write failing manifest and isolation tests**

```rust
#[test]
fn manifest_rejects_undeclared_capability() {
    let err = Manifest::parse(br#"{"id":"x","version":"1","permissions":["raw_hwnd"]}"#).unwrap_err();
    assert!(matches!(err, ManifestError::UnknownPermission("raw_hwnd")));
}

#[test]
fn crashed_docklet_is_disabled_without_stopping_host() {
    let result = host().run_once("crasher");
    assert!(result.host_alive);
    assert!(result.docklet_disabled);
}
```

- [ ] **Step 2: Run tests and verify failure**

Run `cargo test --manifest-path D:\dockyh\dock-app\Cargo.toml docklet_manifest docklet_permissions docklet_isolation`. Expected: FAIL because no manifest validator or isolated host exists.

- [ ] **Step 3: Implement the constrained contract**

Validate id, version, signature/source metadata, visual form, refresh period, lifecycle, declared permissions, and resource limits. Support icon+badge, 60-point sparkline, normalized mini-dial, and flyout with 16 px margin, 48 px footer, 4 px gutter, Caption 12/16, Body 14/20, and WCAG AA contrast. Run official and third-party Docklets in constrained QuickJS or a helper process; never expose `new Function`, arbitrary JavaScript, DOM, raw HWND, unrestricted Win32, filesystem paths, process control, network, or clipboard without a declared permission and user decision. Disable and report a crashed or over-budget Docklet.

- [ ] **Step 4: Verify built-ins and commit**

Install the time, CPU, memory, and weather placeholder Docklets from signed local packages, exercise install/enable/disable/uninstall and permission denial, then commit `feat: add permissioned docklet host and builtins`.

## Task 10: Add shell runtime, settings, recovery, and update boundaries

**Files:**
- Create: `src/runtime/shell.rs`, `settings.rs`, `recovery.rs`, `logging.rs`
- Modify: `src/main.rs`, `src/core/persistence.rs`
- Test: `tests/unit/settings_migration.rs`, `tests/unit/recovery.rs`, `tests/desktop/single_instance.rs`

- [ ] **Step 1: Write failing migration and shutdown tests**

```rust
#[test]
fn version_zero_settings_migrate_without_losing_layout() {
    let migrated = migrate(version_zero_fixture()).unwrap();
    assert_eq!(migrated.schema_version, CURRENT_SCHEMA);
    assert!(!migrated.layouts.is_empty());
}

#[test]
fn second_instance_forwards_focus_request_and_exits() {
    assert_eq!(launch_second_instance().unwrap(), InstanceResult::ForwardedAndExited);
}
```

- [ ] **Step 2: Run tests and verify failure**

Run `cargo test --manifest-path D:\dockyh\dock-app\Cargo.toml settings_migration recovery single_instance`. Expected: FAIL until runtime ownership exists.

- [ ] **Step 3: Implement runtime boundaries**

Own single-instance mutex and IPC, startup/shutdown gates, crash recovery, logging with error codes, configuration paths under the Dock App namespace, schema migrations, atomic writes and backups, reset/import/export, startup registration, and update boundaries. Keep Dock layout and Win32 handles out of `shell-runtime`; wire services through explicit constructors.

- [ ] **Step 4: Verify restart and commit**

Run the migration/recovery tests, kill the host during a write, restart it, and verify the last valid snapshot loads. Commit `feat: add shell runtime settings and recovery`.

## Task 11: Visual QA, accessibility, and performance gates

**Files:**
- Create: `tests/visual/native_reference.rs`, `tests/perf/idle_budget.rs`, `tests/desktop/accessibility.rs`
- Modify: `src/ui/renderer.rs`, `src/ui/accessibility.rs`, `src/runtime/logging.rs`
- Create: `docs/qa/native-dock-matrix.md`

- [ ] **Step 1: Write failing budget and accessibility tests**

```rust
#[test]
fn idle_budget_is_below_release_thresholds() {
    let sample = run_idle_sample(Duration::from_secs(60));
    assert!(sample.cpu_percent < 1.0);
    assert!(sample.private_mb < 30.0);
}
```

- [ ] **Step 2: Run tests and verify failure**

Run `cargo test --manifest-path D:\dockyh\dock-app\Cargo.toml --test idle_budget --test accessibility`. Expected: FAIL until instrumentation and accessibility providers are implemented.

- [ ] **Step 3: Implement the QA matrix**

Capture native screenshots at 100%, 125%, and 150% DPI for light/dark Sonoma and Tahoe materials, all four edges, one/two monitors, hidden/revealed states, menus, sub-Docks, Launchpad, Shelf stacks, Docklets, reduced motion, and performance mode. Compare against generated golden geometry and token expectations. Verify WCAG AA text/contrast, keyboard navigation, screen-reader names, focus visibility, and disabled/reduced-motion behavior.

- [ ] **Step 4: Enforce performance and commit**

Profile idle CPU/private memory, hover animation, app enumeration, monitor changes, and Docklet sampling. Reject per-frame process enumeration, unbounded polling, dynamic swap-chain resizing during hover, permanent timers, and leaked DComp resources. Commit `test: add native visual accessibility and performance gates`.

## Task 12: Integrate, package, and enforce release boundaries

**Files:**
- Create: `packaging\dock-app.wxs`, `packaging\release.ps1`, `docs\qa\release-checklist.md`
- Modify: `Cargo.toml`, `.gitignore`
- Test: `tests\desktop\release_boundary.rs`

- [ ] **Step 1: Write the failing release-boundary test**

```rust
#[test]
fn release_manifest_contains_only_independent_suite() {
    let files = release_manifest();
    assert!(files.iter().any(|p| p.ends_with("dock-app.exe")));
    assert!(!files.iter().any(|p| p.contains("Glassboard-main")));
    assert!(!files.iter().any(|p| p.contains("vitro")));
}
```

- [ ] **Step 2: Run it and verify failure**

Run `cargo test --manifest-path D:\dockyh\dock-app\Cargo.toml release_boundary`. Expected: FAIL until the packaging manifest is explicit.

- [ ] **Step 3: Implement packaging and source audit**

Package only the independent Dock App binary, schemas, rebuilt artwork, signed built-in Docklets, settings migration metadata, and licenses. Add a release script that fails when `Glassboard-main`, `vitro` runtime sources, copied GlassDesk/BitDock/Nexus code, or unapproved assets enter the staging directory. Record asset provenance and license decisions in `docs/qa/release-checklist.md`.

- [ ] **Step 4: Run the complete verification sequence and commit**

Run:

```powershell
$env:CARGO_HOME='D:\rust\cargo'; $env:RUSTUP_HOME='D:\rust\rustup'
& D:\rust\cargo\bin\cargo.exe fmt --manifest-path D:\dockyh\dock-app\Cargo.toml -- --check
& D:\rust\cargo\bin\cargo.exe test --manifest-path D:\dockyh\dock-app\Cargo.toml --all-targets
& D:\rust\cargo\bin\cargo.exe build --manifest-path D:\dockyh\dock-app\Cargo.toml --release
& D:\dockyh\dock-app\packaging\release.ps1 -Clean
git -C D:\dockyh diff --check
```

Expected: format, unit/platform/desktop/performance/release tests, release build, packaging boundary, and whitespace checks all pass. Commit `release: package independent dock app suite`.

## Plan self-review checklist

- [ ] Every product module in the approved boundary has a task: Dock, Launchpad, Shelf, windows/tasks, tray/clock, Docklets, settings, and shell runtime.
- [ ] Nexus R1–R12 behaviors are represented in Task 6 and verified by desktop acceptance tests.
- [ ] All 11 effect slots and 44 presets, both motion profiles, material tokens, and all effect guards are represented in Task 5.
- [ ] Multi-monitor, DPI, taskbar coexistence, WinEvent reconciliation, OLE drag/drop, recycle-bin errors, and UIA are represented in Tasks 3–4 and 6–8.
- [ ] Atomic persistence, migration, backup, reset/import/export, crash recovery, and single-instance behavior are represented in Tasks 2 and 10.
- [ ] Docklet manifests, permissions, signatures/source metadata, lifecycle, sandbox, resource limits, crash isolation, and built-in forms are represented in Task 9.
- [ ] Idle CPU below 1%, private memory below 30 MB, no unbounded polling, and no permanent timers are checked in Task 11.
- [ ] Release packaging excludes Glassboard and Vitro runtime code and records asset/license provenance in Task 12.
- [ ] Every plan step names concrete files, commands, assertions, and expected outcomes; no unspecified work remains.

