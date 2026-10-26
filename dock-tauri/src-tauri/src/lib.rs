//! Dock Tauri host: window geometry, app catalog, metrics sidecar, settings.

pub mod apps;
pub mod autohide;
pub mod commands;
pub mod geometry;
pub mod icon_cache;
pub mod metrics;
pub mod platform;
pub mod settings;
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
            // Auto-hide defaults off; a persisted setting is applied below.
            let mut startup = settings::load_or_recover(&settings_path).unwrap_or_default();
            let apps_root = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::env::temp_dir())
                .join("icons");
            let _icons = icon_cache::IconCache::new(apps_root);
            let catalog = apps::catalog_from_dirs(&apps::start_menu_dirs());

            // First run (or a settings file predating pinning): seed a compact
            // dock from the catalog rather than showing every installed app.
            if startup.pinned.is_empty() {
                startup.pinned = state::seed_pins(&catalog);
                let _ = settings::save_atomic(&settings_path, &startup);
            }
            let pinned = startup.pinned.clone();

            app.manage(commands::Runtime {
                catalog: Mutex::new(catalog),
                pinned: Mutex::new(pinned),
                settings_path,
                sidecar: Mutex::new(metrics::Supervisor::new(commands::sidecar_spec())),
                autohide: Mutex::new(autohide::AutoHide::new(false)),
            });

            // Apply the persisted mode once. This also positions the window at
            // the bottom edge before it becomes visible.
            let _ = commands::set_autohide_enabled(
                app.handle().clone(),
                app.state::<commands::Runtime>(),
                startup.autohide,
            );

            // 1 Hz window reconciliation. Bounded, never per-frame. The catalog
            // lock is released before emitting so a slow webview cannot stall launch.
            let handle = app.handle().clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_millis(1000));
                let runtime = handle.state::<commands::Runtime>();
                let Ok(settings) = settings::load_or_recover(&runtime.settings_path) else {
                    continue;
                };
                let state = {
                    let Ok(catalog) = runtime.catalog.lock() else {
                        continue;
                    };
                    let Ok(pinned) = runtime.pinned.lock() else {
                        continue;
                    };
                    let running = windows_tracker::snapshot_and_reconcile(&catalog);
                    let icons = icon_cache::icons_for(&catalog, 48);
                    state::build_dock_state(
                        &catalog,
                        &pinned,
                        &running,
                        &icons,
                        &commands::config_from(&settings),
                    )
                };
                let _ = handle.emit("dock://state", &state);
            });

            if let Some(window) = app.get_webview_window("dock") {
                let _ = window.show();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_dock_state,
            commands::get_catalog,
            commands::pin_app,
            commands::unpin_app,
            commands::launch_item,
            commands::set_route,
            commands::refresh_state,
            commands::get_settings,
            commands::save_settings,
            commands::sample_monitor,
            commands::set_pointer_inside,
            commands::autohide_tick,
            commands::set_autohide_enabled,
        ])
        .run(tauri::generate_context!())
        .expect("failed to start Dock Tauri");
}

fn settings_dir() -> std::path::PathBuf {
    std::env::var("APPDATA")
        .map(|base| std::path::PathBuf::from(base).join("DockApp"))
        .unwrap_or_else(|_| std::env::temp_dir())
}
