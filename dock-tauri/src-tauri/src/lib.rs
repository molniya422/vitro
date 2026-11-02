//! Dock Tauri host: window geometry, app catalog, metrics sidecar, settings.

pub mod appearance;
pub mod apps;
pub mod autohide;
pub mod commands;
pub mod foreground;
pub mod geometry;
pub mod hit_test;
pub mod icon_cache;
pub mod metrics;
pub mod platform;
pub mod pointer_watch;
pub mod settings;
pub mod shelf;
pub mod state;
pub mod window_geom;
pub mod windows_tracker;

use std::sync::Mutex;
use tauri::{Emitter, Manager};

pub fn product_name() -> &'static str {
    "Dock Tauri"
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let settings_path = settings_dir().join("settings.json");
            let mut startup = settings::load_or_recover(&settings_path).unwrap_or_default();
            let catalog = apps::catalog_from_dirs(&apps::start_menu_dirs());

            // First run (or a settings file predating pinning): seed a compact
            // dock from the catalog rather than showing every installed app.
            if startup.pinned.is_empty() {
                startup.pinned = state::seed_pins(&catalog);
                let _ = settings::save_atomic(&settings_path, &startup);
            }
            let pinned = startup.pinned.clone();
            let initial_autohide = startup.autohide;
            let initial_show_delay = startup.show_delay_ms as u64;
            let initial_hide_delay = startup.hide_delay_ms as u64;

            app.manage(commands::Runtime {
                catalog: Mutex::new(catalog),
                pinned: Mutex::new(pinned),
                settings_path,
                settings: Mutex::new(startup),
                icons: Mutex::new(None),
                sidecar: Mutex::new(metrics::Supervisor::new(commands::sidecar_spec())),
                autohide: Mutex::new(autohide::AutoHide::new_with_delays(
                    initial_autohide,
                    initial_show_delay,
                    initial_hide_delay,
                )),
                route: Mutex::new("dock".to_string()),
                hit: Mutex::new(None),
                drag: Mutex::new(false),
                shelf: Mutex::new(Vec::new()),
            });

            // Position the window at the bottom edge before it becomes visible.
            // The watcher owns every subsequent geometry change.
            let initial_handle = app.handle().clone();
            let initial_runtime = app.state::<commands::Runtime>();
            let initial_visible = initial_runtime
                .autohide
                .lock()
                .map(|machine| machine.visibility() == autohide::Visibility::Visible)
                .unwrap_or(true);
            let _ =
                commands::apply_autohide_geometry(&initial_handle, &initial_runtime, initial_visible);

            // Pay the Python/psutil import cost before the monitor is first
            // opened. The supervisor keeps the child alive afterwards.
            let warm = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(500));
                let runtime = warm.state::<commands::Runtime>();
                if let Ok(mut supervisor) = runtime.sidecar.lock() {
                    let _ = supervisor.sample();
                };
            });

            // 1 Hz window reconciliation. Bounded, never per-frame. Only the
            // dock route needs pushes, and only when the running set, pins, or
            // settings actually changed: icon data URLs are expensive to copy.
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let mut last: Option<(Vec<String>, Vec<apps::RunningInfo>, settings::Settings)> =
                    None;
                loop {
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                    let runtime = handle.state::<commands::Runtime>();
                    // Drop an idle Python child so the closed monitor costs
                    // nothing; `set_route("monitor")` warms it again.
                    if let Ok(mut supervisor) = runtime.sidecar.lock() {
                        supervisor.reap_if_idle();
                    }
                    let route = runtime
                        .route
                        .lock()
                        .map(|route| route.clone())
                        .unwrap_or_default();
                    if route != "dock" {
                        last = None;
                        continue;
                    }
                    let Ok(settings) = runtime.settings.lock().map(|s| s.clone()) else {
                        continue;
                    };
                    let Ok(catalog) = runtime.catalog.lock() else {
                        continue;
                    };
                    let Ok(pinned) = runtime.pinned.lock() else {
                        continue;
                    };
                    let running = windows_tracker::snapshot_and_reconcile(&catalog);
                    let fingerprint = (pinned.clone(), running.clone(), settings.clone());
                    if last.as_ref() == Some(&fingerprint) {
                        continue;
                    }
                    let Ok(state) = commands::build_state_with_running(
                        &runtime,
                        &catalog,
                        &pinned,
                        &running,
                        &settings,
                    ) else {
                        continue;
                    };
                    last = Some(fingerprint);
                    let _ = handle.emit("dock://state", &state);
                }
            });

            if let Some(window) = app.get_webview_window("dock") {
                let _ = window.show();
            }
            pointer_watch::spawn(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_dock_state,
            commands::get_catalog,
            commands::pin_app,
            commands::unpin_app,
            commands::reorder_pins,
            commands::launch_item,
            commands::set_route,
            commands::refresh_state,
            commands::get_settings,
            commands::save_settings,
            commands::sample_monitor,
            commands::set_hit_rects,
            commands::set_drag_active,
            commands::get_shelf,
            commands::shelf_add,
            commands::shelf_remove,
            commands::shelf_clear,
            commands::open_shelf_item,
        ])
        .run(tauri::generate_context!())
        .expect("failed to start Dock Tauri");
}

fn settings_dir() -> std::path::PathBuf {
    std::env::var("APPDATA")
        .map(|base| std::path::PathBuf::from(base).join("DockApp"))
        .unwrap_or_else(|_| std::env::temp_dir())
}
