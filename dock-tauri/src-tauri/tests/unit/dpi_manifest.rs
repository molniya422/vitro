//! Guards the DPI-awareness manifest. Without it, Win32 virtualises coordinates
//! on scaled displays and the dock renders at the wrong physical size.

#[test]
fn the_manifest_file_declares_per_monitor_v2() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dock-tauri.exe.manifest");
    let text = std::fs::read_to_string(&path).expect("manifest must exist");
    assert!(text.contains("PerMonitorV2"), "missing PerMonitorV2");
    assert!(text.contains("dpiAware"), "missing dpiAware");
}

#[test]
fn build_script_embeds_the_manifest() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("build.rs");
    let text = std::fs::read_to_string(&path).expect("build.rs must exist");
    assert!(
        text.contains("/MANIFESTINPUT:dock-tauri.exe.manifest"),
        "build.rs must pass the manifest to the linker"
    );
}
