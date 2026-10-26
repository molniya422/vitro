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
pub mod windows_tracker;

use std::sync::Mutex;
use tauri::Manager;

pub fn product_name() -> &'static str {
    "Dock Tauri"
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let settings_path = settings_dir().join("settings.json");
            let apps_root = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::env::temp_dir())
                .join("icons");
            let _icons = icon_cache::IconCache::new(apps_root);
            let catalog = apps::catalog_from_dirs(&apps::start_menu_dirs());
            app.manage(commands::Runtime {
                apps: Mutex::new(catalog),
                settings_path,
                sidecar: Mutex::new(metrics::Supervisor::new(commands::sidecar_spec())),
            });

            if let Some(window) = app.get_webview_window("dock") {
                let _ = window.show();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_dock_state,
            commands::launch_item,
            commands::set_route,
            commands::get_settings,
            commands::save_settings,
            commands::sample_monitor,
        ])
        .run(tauri::generate_context!())
        .expect("failed to start Dock Tauri");
}

fn settings_dir() -> std::path::PathBuf {
    std::env::var("APPDATA")
        .map(|base| std::path::PathBuf::from(base).join("DockApp"))
        .unwrap_or_else(|_| std::env::temp_dir())
}
