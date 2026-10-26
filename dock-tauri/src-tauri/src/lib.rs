//! Dock Tauri host: window geometry, app catalog, metrics sidecar, settings.

pub mod autohide;
pub mod geometry;
pub mod settings;

pub fn product_name() -> &'static str {
    "Dock Tauri"
}

pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("failed to start Dock Tauri");
}
