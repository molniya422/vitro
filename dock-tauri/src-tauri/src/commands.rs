//! Tauri command surface. Thin: parse, delegate, return typed errors.

use crate::apps::{self, AppEntry};
use crate::metrics::Sample;
use crate::settings::{self, Settings};
use crate::state::{build_dock_state, DockConfig, DockStateView};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};

pub struct Runtime {
    pub apps: Mutex<Vec<AppEntry>>,
    pub settings_path: PathBuf,
    /// Owns at most one sidecar and respawns it after a crash.
    pub sidecar: Mutex<crate::metrics::Supervisor>,
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

fn config_from(settings: &Settings) -> DockConfig {
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
    let apps = runtime
        .apps
        .lock()
        .map_err(|_| CommandError::new("lock_poisoned", "app catalog lock poisoned"))?;
    let running = crate::windows_tracker::snapshot_and_reconcile(&apps);
    let icons = crate::icon_cache::icons_for(&apps, 48);
    Ok(build_dock_state(
        &apps,
        &running,
        &icons,
        &config_from(&settings),
    ))
}

#[tauri::command]
pub fn launch_item(id: String) -> Result<(), CommandError> {
    apps::launch(&id).map_err(|e| CommandError::new("launch_failed", e))
}

#[tauri::command]
pub fn set_route(app: AppHandle, route: String) -> Result<(), CommandError> {
    let allowed = ["dock", "launchpad", "monitor", "settings"];
    if !allowed.contains(&route.as_str()) {
        return Err(CommandError::new("unknown_route", route));
    }
    app.emit("dock://route", &route)
        .map_err(|e| CommandError::new("emit_failed", e.to_string()))
}

#[tauri::command]
pub fn get_settings(runtime: State<'_, Runtime>) -> Result<Settings, CommandError> {
    settings::load_or_recover(&runtime.settings_path)
        .map_err(|e| CommandError::new("settings_unreadable", e.to_string()))
}

#[tauri::command]
pub fn save_settings(runtime: State<'_, Runtime>, settings: Settings) -> Result<(), CommandError> {
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

/// Bring the window up once the first frame is ready, avoiding a white flash.
pub fn show_dock_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("dock") {
        let _ = window.show();
    }
}
