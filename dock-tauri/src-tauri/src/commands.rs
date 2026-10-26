//! Tauri command surface. Thin: parse, delegate, return typed errors.

use crate::apps::{self, AppEntry};
use crate::autohide::{Action, AutoHide, Event, Visibility};
use crate::metrics::Sample;
use crate::settings::{self, Settings};
use crate::state::{build_catalog, build_dock_state, DockConfig, DockItemView, DockStateView};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};

pub struct Runtime {
    /// Every discovered app. Launchpad renders this list; the dock does not.
    pub catalog: Mutex<Vec<AppEntry>>,
    /// Ids shown in the dock, in display order. Capped by `DOCK_ITEM_CAP`.
    pub pinned: Mutex<Vec<String>>,
    pub settings_path: PathBuf,
    /// Owns at most one sidecar and respawns it after a crash.
    pub sidecar: Mutex<crate::metrics::Supervisor>,
    /// Rust owns the auto-hide decision; React only reports pointer presence.
    pub autohide: Mutex<AutoHide>,
}

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

pub(crate) fn config_from(settings: &Settings) -> DockConfig {
    DockConfig {
        autohide: settings.autohide,
        magnify: settings.magnify,
        reduced_motion: settings.reduced_motion,
        profile: settings.profile.clone(),
    }
}

#[tauri::command]
pub fn get_dock_state(runtime: State<'_, Runtime>) -> Result<DockStateView, CommandError> {
    let settings = settings::load_or_recover(&runtime.settings_path)
        .map_err(|e| CommandError::new("settings_unreadable", e.to_string()))?;
    let catalog = runtime
        .catalog
        .lock()
        .map_err(|_| CommandError::new("lock_poisoned", "app catalog lock poisoned"))?;
    let pinned = runtime
        .pinned
        .lock()
        .map_err(|_| CommandError::new("lock_poisoned", "pinned lock poisoned"))?;
    let running = crate::windows_tracker::snapshot_and_reconcile(&catalog);
    let icons = crate::icon_cache::icons_for(&catalog, 48);
    Ok(build_dock_state(
        &catalog,
        &pinned,
        &running,
        &icons,
        &config_from(&settings),
    ))
}

/// Every discovered app, for Launchpad. Never capped by the dock limit.
#[tauri::command]
pub fn get_catalog(runtime: State<'_, Runtime>) -> Result<Vec<DockItemView>, CommandError> {
    let catalog = runtime
        .catalog
        .lock()
        .map_err(|_| CommandError::new("lock_poisoned", "app catalog lock poisoned"))?;
    let running = crate::windows_tracker::snapshot_and_reconcile(&catalog);
    let icons = crate::icon_cache::icons_for(&catalog, 48);
    Ok(build_catalog(&catalog, &running, &icons))
}

fn persist_pins(runtime: &Runtime, pinned: &[String]) -> Result<(), CommandError> {
    let mut settings = settings::load_or_recover(&runtime.settings_path)
        .map_err(|e| CommandError::new("settings_unreadable", e.to_string()))?;
    settings.pinned = pinned.to_vec();
    settings::save_atomic(&runtime.settings_path, &settings)
        .map_err(|e| CommandError::new("settings_unwritable", e.to_string()))
}

/// Add an app to the dock. Idempotent; rejects a full dock or an unknown id.
#[tauri::command]
pub fn pin_app(runtime: State<'_, Runtime>, id: String) -> Result<(), CommandError> {
    let catalog = runtime
        .catalog
        .lock()
        .map_err(|_| CommandError::new("lock_poisoned", "app catalog lock poisoned"))?;
    let known = apps::system_entry(&id).is_some() || catalog.iter().any(|app| app.id == id);
    if !known {
        return Err(CommandError::new("unknown_app", id));
    }

    let mut pinned = runtime
        .pinned
        .lock()
        .map_err(|_| CommandError::new("lock_poisoned", "pinned lock poisoned"))?;
    if pinned.iter().any(|pinned_id| pinned_id == &id) {
        return Ok(());
    }
    if pinned.len() >= crate::state::DOCK_ITEM_CAP {
        return Err(CommandError::new(
            "dock_full",
            format!("the dock is limited to {}", crate::state::DOCK_ITEM_CAP),
        ));
    }

    let mut next = pinned.clone();
    next.push(id);
    persist_pins(&runtime, &next)?;
    *pinned = next;
    Ok(())
}

/// Remove an app from the dock. Idempotent.
#[tauri::command]
pub fn unpin_app(runtime: State<'_, Runtime>, id: String) -> Result<(), CommandError> {
    let mut pinned = runtime
        .pinned
        .lock()
        .map_err(|_| CommandError::new("lock_poisoned", "pinned lock poisoned"))?;
    let Some(index) = pinned.iter().position(|pinned_id| pinned_id == &id) else {
        return Ok(());
    };

    let mut next = pinned.clone();
    next.remove(index);
    persist_pins(&runtime, &next)?;
    *pinned = next;
    Ok(())
}

#[tauri::command]
pub fn launch_item(id: String) -> Result<(), CommandError> {
    apps::launch(&id).map_err(|e| CommandError::new("launch_failed", e))
}

fn dock_item_count(runtime: &Runtime) -> usize {
    runtime
        .pinned
        .lock()
        .map(|pinned| pinned.len().min(crate::state::DOCK_ITEM_CAP))
        .unwrap_or(0)
}

#[tauri::command]
pub fn set_route(
    app: AppHandle,
    runtime: State<'_, Runtime>,
    route: String,
) -> Result<(), CommandError> {
    let allowed = ["dock", "launchpad", "monitor", "settings"];
    if !allowed.contains(&route.as_str()) {
        return Err(CommandError::new("unknown_route", route));
    }

    let monitor = crate::platform::primary_work_area()
        .ok_or_else(|| CommandError::new("no_monitor", "no primary monitor"))?;
    // The window tracks the visible pill, so the pinned count (not the whole
    // catalog) drives its width.
    let item_count = dock_item_count(&runtime);
    let (width, height) = crate::window_geom::route_size(&route, monitor, item_count, 48.0);

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

    app.emit("dock://route", &route)
        .map_err(|e| CommandError::new("emit_failed", e.to_string()))
}

/// Push a fresh snapshot on demand, used by the 1 Hz sync.
#[tauri::command]
pub fn refresh_state(app: AppHandle, runtime: State<'_, Runtime>) -> Result<(), CommandError> {
    let settings = settings::load_or_recover(&runtime.settings_path)
        .map_err(|e| CommandError::new("settings_unreadable", e.to_string()))?;
    let catalog = runtime
        .catalog
        .lock()
        .map_err(|_| CommandError::new("lock_poisoned", "app catalog lock poisoned"))?;
    let pinned = runtime
        .pinned
        .lock()
        .map_err(|_| CommandError::new("lock_poisoned", "pinned lock poisoned"))?;
    let running = crate::windows_tracker::snapshot_and_reconcile(&catalog);
    let icons = crate::icon_cache::icons_for(&catalog, 48);
    let state = build_dock_state(&catalog, &pinned, &running, &icons, &config_from(&settings));
    app.emit("dock://state", &state)
        .map_err(|e| CommandError::new("emit_failed", e.to_string()))
}

#[tauri::command]
pub fn get_settings(runtime: State<'_, Runtime>) -> Result<Settings, CommandError> {
    settings::load_or_recover(&runtime.settings_path)
        .map_err(|e| CommandError::new("settings_unreadable", e.to_string()))
}

#[tauri::command]
pub fn save_settings(
    runtime: State<'_, Runtime>,
    mut settings: Settings,
) -> Result<(), CommandError> {
    // `pinned` is owned by pin_app/unpin_app; the settings form must not clobber
    // it with a stale copy.
    settings.pinned = runtime
        .pinned
        .lock()
        .map_err(|_| CommandError::new("lock_poisoned", "pinned lock poisoned"))?
        .clone();
    settings::save_atomic(&runtime.settings_path, &settings)
        .map_err(|e| CommandError::new("settings_unwritable", e.to_string()))
}

#[tauri::command]
pub fn sample_monitor(runtime: State<'_, Runtime>) -> Result<Sample, CommandError> {
    let mut supervisor = runtime
        .sidecar
        .lock()
        .map_err(|_| CommandError::new("lock_poisoned", "sidecar lock poisoned"))?;
    // The supervisor drops a broken child and respawns on the next call, so a
    // crashed sidecar recovers without restarting the dock.
    supervisor
        .sample()
        .map_err(|error| CommandError::new("sidecar_unavailable", error.to_string()))
}

/// Resolve how to launch the Python sidecar for this build.
///
/// Dev builds run the interpreter directly; a packaged build will point at a
/// frozen `monitor.exe` produced by `backend/build_sidecar.ps1` (Task 15).
pub fn sidecar_spec() -> crate::metrics::SidecarSpec {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let launcher = root.join("../backend/run_sidecar.py");

    let python = std::env::var("DOCK_PYTHON")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("D:/jiqixuexi/anaconda/python.exe"));

    crate::metrics::SidecarSpec::new(python, vec![launcher.to_string_lossy().to_string()])
}

/// The auto-hide fields React needs to drive its tick loop.
#[derive(Clone, Copy, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoHideState {
    pub enabled: bool,
    pub visible: bool,
    /// True while a delayed transition is armed and Rust needs another tick.
    pub pending: bool,
}

fn apply_autohide_geometry(app: &AppHandle, runtime: &Runtime, visible: bool) {
    let Some(monitor) = crate::platform::primary_work_area() else {
        return;
    };
    let item_count = dock_item_count(runtime);
    let (width, height) = crate::window_geom::route_size("dock", monitor, item_count, 48.0);

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
}

#[tauri::command]
pub fn set_pointer_inside(
    app: AppHandle,
    runtime: State<'_, Runtime>,
    inside: bool,
    now_ms: u64,
) -> Result<AutoHideState, CommandError> {
    let mut machine = runtime
        .autohide
        .lock()
        .map_err(|_| CommandError::new("lock_poisoned", "autohide lock poisoned"))?;
    let action = machine.handle(
        if inside {
            Event::PointerEntered
        } else {
            Event::PointerLeft
        },
        now_ms,
    );
    drop(machine);

    if action != Action::None {
        let visible = action == Action::Reveal;
        apply_autohide_geometry(&app, &runtime, visible);
    }
    Ok(autohide_state(&runtime))
}

#[tauri::command]
pub fn autohide_tick(
    app: AppHandle,
    runtime: State<'_, Runtime>,
    now_ms: u64,
) -> Result<AutoHideState, CommandError> {
    let mut machine = runtime
        .autohide
        .lock()
        .map_err(|_| CommandError::new("lock_poisoned", "autohide lock poisoned"))?;
    let action = machine.handle(Event::Tick, now_ms);
    drop(machine);

    if action != Action::None {
        let visible = action == Action::Reveal;
        apply_autohide_geometry(&app, &runtime, visible);
    }
    Ok(autohide_state(&runtime))
}

fn autohide_state(runtime: &Runtime) -> AutoHideState {
    let machine = runtime.autohide.lock();
    match machine {
        Ok(machine) => AutoHideState {
            enabled: machine.enabled(),
            visible: machine.visibility() == Visibility::Visible,
            pending: machine.pending(),
        },
        Err(_) => AutoHideState {
            enabled: false,
            visible: true,
            pending: false,
        },
    }
}

/// Apply the persisted auto-hide setting at startup and when it changes.
#[tauri::command]
pub fn set_autohide_enabled(
    app: AppHandle,
    runtime: State<'_, Runtime>,
    enabled: bool,
) -> Result<AutoHideState, CommandError> {
    {
        let mut machine = runtime
            .autohide
            .lock()
            .map_err(|_| CommandError::new("lock_poisoned", "autohide lock poisoned"))?;
        // Re-seed instead of toggling: `AutoHide::new(true)` starts collapsed and
        // `new(false)` starts visible, matching the geometry applied below.
        *machine = AutoHide::new(enabled);
    }
    apply_autohide_geometry(&app, &runtime, !enabled);
    Ok(autohide_state(&runtime))
}

/// Bring the window up once the first frame is ready, avoiding a white flash.
pub fn show_dock_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("dock") {
        let _ = window.show();
    }
}
