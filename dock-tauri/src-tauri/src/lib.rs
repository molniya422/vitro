//! Dock Tauri host: window geometry, app catalog, metrics sidecar, settings.

pub mod apps;
pub mod autohide;
pub mod geometry;
pub mod icon_cache;
pub mod metrics;
pub mod platform;
pub mod settings;

pub fn product_name() -> &'static str {
    "Dock Tauri"
}

pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("failed to start Dock Tauri");
}
