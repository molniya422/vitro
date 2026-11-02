//! Tauri command surface. Thin: parse, delegate, return typed errors.

use crate::appearance::{DockAppearance, DockMetrics};
use crate::apps::{self, AppEntry, RunningInfo};
use crate::autohide::{AutoHide, Event, Visibility};
use crate::hit_test::Rect;
use crate::icon_cache::IconSnapshot;
use crate::metrics::Sample;
use crate::settings::{self, Settings, SettingsView};
use crate::state::{build_catalog, build_dock_state, DockItemView, DockStateView, DOCK_ITEM_CAP};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};

pub struct Runtime {
    /// Every discovered app. Launchpad renders this list; the dock does not.
    pub catalog: Mutex<Vec<AppEntry>>,
    /// Ids shown in the dock, in display order. Capped by `DOCK_ITEM_CAP`.
    pub pinned: Mutex<Vec<String>>,
    pub settings_path: PathBuf,
    /// In-memory source of truth. Updated by `save_settings`; the 1 Hz sync
    /// reads this instead of touching the disk.
    pub settings: Mutex<Settings>,
    /// Lazily loaded icon data URLs, reused across state pushes.
    pub icons: Mutex<Option<IconSnapshot>>,
    /// Owns at most one sidecar and respawns it after a crash.
    pub sidecar: Mutex<crate::metrics::Supervisor>,
    /// Rust owns the auto-hide and hit-testing decisions.
    pub autohide: Mutex<AutoHide>,
    /// The surface currently shown in the single window.
    pub route: Mutex<String>,
    /// Interactive rects reported by React, in CSS pixels relative to the
    /// window. `None` means "the whole window is interactive" (safe startup
    /// fallback and one frame after a geometry change).
    pub hit: Mutex<Option<HitRegion>>,
    /// A tile is being dragged. The watcher keeps the window interactive and
    /// visible for the whole gesture, wherever the pointer goes.
    pub drag: Mutex<bool>,
    /// Session-only shelf paths, in drop order.
    pub shelf: Mutex<Vec<String>>,
}

#[derive(Clone, Debug)]
pub struct HitRegion {
    /// Measured pill width in CSS pixels. Position and height are derived from
    /// the window and settings so auto-hide geometry changes cannot stale them.
    pub pill_width_css: f64,
    /// Extra interactive overlays (the context menu) in CSS pixel coordinates.
    pub rects: Vec<Rect>,
    pub scale: f64,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CssRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// A context menu and the pill are the only overlays today; cap the list so a
/// compromised webview cannot make the watcher do unbounded work.
const MAX_HIT_RECTS: usize = 16;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub code: String,
    pub detail: String,
}

impl CommandError {
    fn new(code: &str, detail: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            detail: detail.into(),
        }
    }
}

fn lock_error(what: &str) -> CommandError {
    CommandError::new("lock_poisoned", format!("{what} lock poisoned"))
}

fn current_settings(runtime: &Runtime) -> Result<Settings, CommandError> {
    runtime
        .settings
        .lock()
        .map(|settings| settings.clone())
        .map_err(|_| lock_error("settings"))
}

fn current_route(runtime: &Runtime) -> Result<String, CommandError> {
    runtime
        .route
        .lock()
        .map(|route| route.clone())
        .map_err(|_| lock_error("route"))
}

fn monitor_scale() -> f64 {
    crate::platform::primary_work_area()
        .map(|monitor| {
            if monitor.scale > 0.0 && monitor.scale.is_finite() {
                monitor.scale
            } else {
                1.0
            }
        })
        .unwrap_or(1.0)
}

fn pinned_indices(catalog: &[AppEntry], pinned: &[String]) -> Vec<usize> {
    pinned
        .iter()
        .take(DOCK_ITEM_CAP)
        .filter_map(|id| catalog.iter().position(|app| &app.id == id))
        .collect()
}

/// Ensure the requested indices are loaded, creating/resetting the snapshot if
/// the catalog length or device pixel size changed.
fn icon_urls(
    runtime: &Runtime,
    catalog: &[AppEntry],
    indices: &[usize],
    px: u32,
) -> Result<Vec<Option<String>>, CommandError> {
    let mut guard = runtime
        .icons
        .lock()
        .map_err(|_| lock_error("icons"))?;
    let reset = guard
        .as_ref()
        .map_or(true, |snapshot| !snapshot.matches(catalog.len(), px));
    if reset {
        *guard = Some(IconSnapshot::new(catalog.len(), px));
    }
    let snapshot = guard.as_mut().expect("snapshot was just initialized");
    snapshot.ensure(catalog, indices.iter().copied());
    Ok(snapshot.urls().to_vec())
}

/// Build a state from an already-computed running snapshot.
///
/// The 1 Hz thread computes `running` once for its change fingerprint and then
/// reuses it here, rather than enumerating windows twice.
pub(crate) fn build_state_with_running(
    runtime: &Runtime,
    catalog: &[AppEntry],
    pinned: &[String],
    running: &[RunningInfo],
    settings: &Settings,
) -> Result<DockStateView, CommandError> {
    let metrics = DockMetrics::from_settings(settings);
    let px = metrics.device_icon_px(monitor_scale());
    let indices = pinned_indices(catalog, pinned);
    let icons = icon_urls(runtime, catalog, &indices, px)?;
    let appearance = DockAppearance::from_settings(settings);
    Ok(build_dock_state(catalog, pinned, running, &icons, &appearance))
}

#[tauri::command]
pub fn get_dock_state(runtime: State<'_, Runtime>) -> Result<DockStateView, CommandError> {
    let settings = current_settings(&runtime)?;
    let catalog = runtime
        .catalog
        .lock()
        .map_err(|_| lock_error("app catalog"))?;
    let pinned = runtime
        .pinned
        .lock()
        .map_err(|_| lock_error("pinned"))?;
    let running = crate::windows_tracker::snapshot_and_reconcile(&catalog);
    build_state_with_running(&runtime, &catalog, &pinned, &running, &settings)
}

/// Every discovered app, for Launchpad. Never capped by the dock limit.
#[tauri::command]
pub fn get_catalog(runtime: State<'_, Runtime>) -> Result<Vec<DockItemView>, CommandError> {
    let settings = current_settings(&runtime)?;
    let catalog = runtime
        .catalog
        .lock()
        .map_err(|_| lock_error("app catalog"))?;
    let px = DockMetrics::from_settings(&settings).device_icon_px(monitor_scale());
    let indices: Vec<usize> = (0..catalog.len()).collect();
    let icons = icon_urls(&runtime, &catalog, &indices, px)?;
    let running = crate::windows_tracker::snapshot_and_reconcile(&catalog);
    Ok(build_catalog(&catalog, &running, &icons))
}

fn persist_pins(runtime: &Runtime, pinned: &[String]) -> Result<(), CommandError> {
    let mut settings = current_settings(runtime)?;
    settings.pinned = pinned.to_vec();
    settings::save_atomic(&runtime.settings_path, &settings)
        .map_err(|e| CommandError::new("settings_unwritable", e.to_string()))?;
    *runtime
        .settings
        .lock()
        .map_err(|_| lock_error("settings"))? = settings;
    Ok(())
}

/// Add an app to the dock. Idempotent; rejects a full dock or an unknown id.
#[tauri::command]
pub fn pin_app(
    app: AppHandle,
    runtime: State<'_, Runtime>,
    id: String,
) -> Result<(), CommandError> {
    {
        let catalog = runtime
            .catalog
            .lock()
            .map_err(|_| lock_error("app catalog"))?;
        let known = apps::system_entry(&id).is_some() || catalog.iter().any(|app| app.id == id);
        if !known {
            return Err(CommandError::new("unknown_app", id));
        }

        let mut pinned = runtime
            .pinned
            .lock()
            .map_err(|_| lock_error("pinned"))?;
        if pinned.iter().any(|pinned_id| pinned_id == &id) {
            return Ok(());
        }
        if pinned.len() >= DOCK_ITEM_CAP {
            return Err(CommandError::new(
                "dock_full",
                format!("the dock is limited to {DOCK_ITEM_CAP}"),
            ));
        }

        let mut next = pinned.clone();
        next.push(id);
        persist_pins(&runtime, &next)?;
        *pinned = next;
    }
    emit_state(&app, &runtime)
}

/// Remove an app from the dock. Idempotent.
#[tauri::command]
pub fn unpin_app(
    app: AppHandle,
    runtime: State<'_, Runtime>,
    id: String,
) -> Result<(), CommandError> {
    {
        let mut pinned = runtime
            .pinned
            .lock()
            .map_err(|_| lock_error("pinned"))?;
        let Some(index) = pinned.iter().position(|pinned_id| pinned_id == &id) else {
            return Ok(());
        };

        let mut next = pinned.clone();
        next.remove(index);
        persist_pins(&runtime, &next)?;
        *pinned = next;
    }
    emit_state(&app, &runtime)
}

/// Reorder the dock. `ids` must be a permutation of the current pins.
#[tauri::command]
pub fn reorder_pins(
    app: AppHandle,
    runtime: State<'_, Runtime>,
    ids: Vec<String>,
) -> Result<(), CommandError> {
    {
        let mut pinned = runtime
            .pinned
            .lock()
            .map_err(|_| lock_error("pinned"))?;
        let next = crate::state::apply_order(&pinned, &ids)
            .map_err(|detail| CommandError::new("invalid_order", detail))?;
        persist_pins(&runtime, &next)?;
        *pinned = next;
    }
    emit_state(&app, &runtime)
}

#[tauri::command]
pub fn launch_item(runtime: State<'_, Runtime>, id: String) -> Result<(), CommandError> {
    // A running app comes forward instead of spawning another instance.
    let app = apps::system_entry(&id).or_else(|| {
        runtime
            .catalog
            .lock()
            .ok()
            .and_then(|catalog| catalog.iter().find(|app| app.id == id).cloned())
    });
    if let Some(app) = app {
        let windows = crate::windows_tracker::snapshot_windows();
        if let Some(hwnd) = apps::focus_handle(&app, &windows) {
            crate::platform::activate_window(hwnd);
            return Ok(());
        }
    }
    apps::launch(&id).map_err(|e| CommandError::new("launch_failed", e))
}

fn dock_item_count(runtime: &Runtime) -> Result<usize, CommandError> {
    runtime
        .pinned
        .lock()
        .map(|pinned| pinned.len().min(DOCK_ITEM_CAP))
        .map_err(|_| lock_error("pinned"))
}

#[tauri::command]
pub fn set_hit_rects(
    runtime: State<'_, Runtime>,
    pill_width: f64,
    rects: Vec<CssRect>,
    scale: f64,
) -> Result<(), CommandError> {
    let scale = if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    };
    let pill_width_css = if pill_width.is_finite() && pill_width > 0.0 {
        pill_width
    } else {
        0.0
    };
    let rects = rects
        .into_iter()
        .take(MAX_HIT_RECTS)
        .map(|rect| Rect::from_origin_size(rect.x, rect.y, rect.width, rect.height))
        .collect();
    *runtime.hit.lock().map_err(|_| lock_error("hit rects"))? = Some(HitRegion {
        pill_width_css,
        rects,
        scale,
    });
    Ok(())
}

/// A tile drag lasts until the pointer is released. Auto-hide is locked for
/// the gesture so the dock cannot collapse under the pointer, and the watcher
/// stops making the window click-through so the drag keeps its events.
#[tauri::command]
pub fn set_drag_active(runtime: State<'_, Runtime>, active: bool) -> Result<(), CommandError> {
    *runtime.drag.lock().map_err(|_| lock_error("drag"))? = active;
    runtime
        .autohide
        .lock()
        .map_err(|_| lock_error("autohide"))?
        .handle(Event::SetLocked(active), 0);
    Ok(())
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShelfItemView {
    pub id: String,
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub missing: bool,
    pub icon: Option<String>,
}

fn shelf_views(runtime: &Runtime) -> Result<Vec<ShelfItemView>, CommandError> {
    let paths = runtime
        .shelf
        .lock()
        .map_err(|_| lock_error("shelf"))?
        .clone();
    let px = crate::appearance::device_icon_px(64.0, monitor_scale());
    let apps: Vec<AppEntry> = paths
        .iter()
        .map(|path| AppEntry {
            id: path.clone(),
            display_name: crate::shelf::display_name(path),
            target: Some(path.clone()),
        })
        .collect();
    let icons = crate::icon_cache::icons_for(&apps, px);
    Ok(paths
        .iter()
        .enumerate()
        .map(|(index, path)| {
            let metadata = std::fs::metadata(path);
            ShelfItemView {
                id: path.clone(),
                name: apps[index].display_name.clone(),
                path: path.clone(),
                is_dir: metadata.as_ref().map(|meta| meta.is_dir()).unwrap_or(false),
                missing: metadata.is_err(),
                icon: icons.get(index).cloned().flatten(),
            }
        })
        .collect())
}

#[tauri::command]
pub fn get_shelf(runtime: State<'_, Runtime>) -> Result<Vec<ShelfItemView>, CommandError> {
    shelf_views(&runtime)
}

/// Add dropped files to the shelf. Returns the updated list.
#[tauri::command]
pub fn shelf_add(
    runtime: State<'_, Runtime>,
    paths: Vec<String>,
) -> Result<Vec<ShelfItemView>, CommandError> {
    {
        let mut shelf = runtime.shelf.lock().map_err(|_| lock_error("shelf"))?;
        for path in paths {
            crate::shelf::push_unique(&mut shelf, &path);
        }
    }
    shelf_views(&runtime)
}

#[tauri::command]
pub fn shelf_remove(
    runtime: State<'_, Runtime>,
    path: String,
) -> Result<Vec<ShelfItemView>, CommandError> {
    {
        let mut shelf = runtime.shelf.lock().map_err(|_| lock_error("shelf"))?;
        crate::shelf::remove(&mut shelf, &path);
    }
    shelf_views(&runtime)
}

#[tauri::command]
pub fn shelf_clear(runtime: State<'_, Runtime>) -> Result<Vec<ShelfItemView>, CommandError> {
    runtime
        .shelf
        .lock()
        .map_err(|_| lock_error("shelf"))?
        .clear();
    shelf_views(&runtime)
}

/// Open a shelf entry with the shell. Only paths already on the shelf are
/// accepted, so the webview cannot use this as a general launcher.
#[tauri::command]
pub fn open_shelf_item(runtime: State<'_, Runtime>, path: String) -> Result<(), CommandError> {
    let on_shelf = runtime
        .shelf
        .lock()
        .map_err(|_| lock_error("shelf"))?
        .iter()
        .any(|item| item.eq_ignore_ascii_case(&path));
    if !on_shelf {
        return Err(CommandError::new("unknown_shelf_item", path));
    }
    apps::launch(&path).map_err(|error| CommandError::new("open_failed", error))
}

fn apply_route_geometry(app: &AppHandle, runtime: &Runtime) -> Result<(), CommandError> {
    let route = current_route(runtime)?;
    let monitor = crate::platform::primary_work_area()
        .ok_or_else(|| CommandError::new("no_monitor", "no primary monitor"))?;
    let settings = current_settings(runtime)?;
    let metrics = DockMetrics::from_settings(&settings);
    let item_count = dock_item_count(runtime)?;
    let (width, height) = crate::window_geom::route_size(&route, monitor, item_count, &metrics);

    if let Some(window) = app.get_webview_window("dock") {
        let _ = window.set_size(tauri::PhysicalSize::new(width as u32, height as u32));
        let x = if route == "launchpad" {
            monitor.left as f64
        } else {
            monitor.left as f64 + ((monitor.right - monitor.left) as f64 - width) / 2.0
        };
        let y = if route == "launchpad" {
            monitor.top as f64
        } else {
            (monitor.bottom as f64) - height
        };
        let _ = window.set_position(tauri::PhysicalPosition::new(x as i32, y as i32));
    }
    Ok(())
}

fn emit_state(app: &AppHandle, runtime: &Runtime) -> Result<(), CommandError> {
    let settings = current_settings(runtime)?;
    let catalog = runtime
        .catalog
        .lock()
        .map_err(|_| lock_error("app catalog"))?;
    let pinned = runtime
        .pinned
        .lock()
        .map_err(|_| lock_error("pinned"))?;
    let running = crate::windows_tracker::snapshot_and_reconcile(&catalog);
    let state = build_state_with_running(runtime, &catalog, &pinned, &running, &settings)?;
    app.emit("dock://state", &state)
        .map_err(|e| CommandError::new("emit_failed", e.to_string()))
}

#[tauri::command]
pub fn set_route(
    app: AppHandle,
    runtime: State<'_, Runtime>,
    route: String,
) -> Result<(), CommandError> {
    let allowed = ["dock", "launchpad", "monitor", "settings", "shelf"];
    if !allowed.contains(&route.as_str()) {
        return Err(CommandError::new("unknown_route", route));
    }

    *runtime
        .route
        .lock()
        .map_err(|_| lock_error("route"))? = route.clone();
    if route == "dock" {
        // Coming back to the dock respects the auto-hide resting state instead
        // of flashing a full-size dock the pointer is not over.
        let visible = runtime
            .autohide
            .lock()
            .map_err(|_| lock_error("autohide"))?
            .visibility()
            == Visibility::Visible;
        apply_autohide_geometry(&app, &runtime, visible)?;
    } else {
        apply_route_geometry(&app, &runtime)?;
    }

    if route == "monitor" {
        // The supervisor stops the sidecar after 45 s idle; warm it on the
        // route change so the first sample still finds a booted child.
        if let Ok(mut supervisor) = runtime.sidecar.lock() {
            supervisor.warm();
        }
    }

    app.emit("dock://route", &route)
        .map_err(|e| CommandError::new("emit_failed", e.to_string()))?;

    // The dock may have been hidden while this route was open; make sure React
    // has the current items and appearance the moment it remounts.
    if route == "dock" {
        emit_state(&app, &runtime)?;
    }
    Ok(())
}

/// Push a fresh snapshot on demand.
#[tauri::command]
pub fn refresh_state(app: AppHandle, runtime: State<'_, Runtime>) -> Result<(), CommandError> {
    emit_state(&app, &runtime)
}

#[tauri::command]
pub fn get_settings(runtime: State<'_, Runtime>) -> Result<SettingsView, CommandError> {
    Ok(SettingsView::from_settings(&current_settings(&runtime)?))
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    runtime: State<'_, Runtime>,
    settings: SettingsView,
) -> Result<(), CommandError> {
    // `pinned` is owned by pin_app/unpin_app; the settings form must not clobber
    // it with a stale copy.
    let pinned = runtime
        .pinned
        .lock()
        .map_err(|_| lock_error("pinned"))?
        .clone();
    let next = settings.into_settings(pinned);
    settings::save_atomic(&runtime.settings_path, &next)
        .map_err(|e| CommandError::new("settings_unwritable", e.to_string()))?;
    *runtime
        .settings
        .lock()
        .map_err(|_| lock_error("settings"))? = next.clone();

    let visible = {
        let mut machine = runtime
            .autohide
            .lock()
            .map_err(|_| lock_error("autohide"))?;
        machine.handle(Event::SetEnabled(next.autohide), 0);
        machine.set_delays(next.show_delay_ms as u64, next.hide_delay_ms as u64);
        machine.visibility() == Visibility::Visible
    };

    if current_route(&runtime)? == "dock" {
        apply_autohide_geometry(&app, &runtime, visible)?;
    } else {
        apply_route_geometry(&app, &runtime)?;
    }
    emit_state(&app, &runtime)
}

#[tauri::command]
pub fn sample_monitor(runtime: State<'_, Runtime>) -> Result<Sample, CommandError> {
    let mut supervisor = runtime
        .sidecar
        .lock()
        .map_err(|_| lock_error("sidecar"))?;
    // The supervisor drops a broken child and respawns on the next call, so a
    // crashed sidecar recovers without restarting the dock.
    supervisor
        .sample()
        .map_err(|error| CommandError::new("sidecar_unavailable", error.to_string()))
}

/// Resolve how to launch the Python sidecar for this build.
///
/// `exe_dir` is the directory containing dock-tauri.exe. Tauri copies bundled
/// `externalBin` files beside it, so a `monitor.exe` there is the frozen
/// sidecar. `dev_python` and `dev_launcher` are the development fallbacks.
///
/// A bundled binary wins. Otherwise the interpreter + launcher is returned even
/// if either path is missing: the supervisor surfaces a typed spawn error,
/// which is better than a panic.
pub fn resolve_sidecar(
    exe_dir: &Path,
    dev_python: &Path,
    dev_launcher: &Path,
) -> crate::metrics::SidecarSpec {
    let bundled = exe_dir.join("monitor.exe");
    // `is_file` (not `exists`) so a directory named monitor.exe is not mistaken
    // for the frozen binary.
    if bundled.is_file() {
        return crate::metrics::SidecarSpec::new(bundled, Vec::new());
    }
    crate::metrics::SidecarSpec::new(
        dev_python.to_path_buf(),
        vec![dev_launcher.to_string_lossy().to_string()],
    )
}

/// Point at the frozen sidecar when packaged, or the Python dev launcher when
/// running from `target/debug` during development.
pub fn sidecar_spec() -> crate::metrics::SidecarSpec {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let launcher = root.join("../backend/run_sidecar.py");

    let python = std::env::var("DOCK_PYTHON")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("D:/jiqixuexi/anaconda/python.exe"));

    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf));
    match exe_dir {
        Some(dir) => resolve_sidecar(&dir, &python, &launcher),
        None => {
            crate::metrics::SidecarSpec::new(python, vec![launcher.to_string_lossy().to_string()])
        }
    }
}

pub(crate) fn apply_autohide_geometry(
    app: &AppHandle,
    runtime: &Runtime,
    visible: bool,
) -> Result<(), CommandError> {
    let monitor = crate::platform::primary_work_area()
        .ok_or_else(|| CommandError::new("no_monitor", "no primary monitor"))?;
    let settings = current_settings(runtime)?;
    let metrics = DockMetrics::from_settings(&settings);
    let item_count = dock_item_count(runtime)?;
    let (width, height) = crate::window_geom::route_size("dock", monitor, item_count, &metrics);

    // Collapsed: a 2 px full-width strip at the screen edge is the hot zone.
    let (w, h) = if visible {
        (width, height)
    } else {
        ((monitor.right - monitor.left) as f64, 2.0)
    };
    // The visible dock is centered; the strip spans the edge so it is easy to
    // hit from anywhere along the bottom of the screen.
    let x = if visible {
        monitor.left as f64 + ((monitor.right - monitor.left) as f64 - w) / 2.0
    } else {
        monitor.left as f64
    };
    let y = if visible {
        (monitor.bottom as f64) - h
    } else {
        (monitor.bottom as f64) - 2.0
    };

    if let Some(window) = app.get_webview_window("dock") {
        let _ = window.set_size(tauri::PhysicalSize::new(
            w.max(1.0) as u32,
            h.max(1.0) as u32,
        ));
        let _ = window.set_position(tauri::PhysicalPosition::new(x as i32, y as i32));
    }
    Ok(())
}

/// Bring the window up once the first frame is ready, avoiding a white flash.
pub fn show_dock_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("dock") {
        let _ = window.show();
    }
}
