fn main() {
    // Embed the DPI-awareness manifest for MSVC builds. It declares PerMonitorV2
    // so Win32 reports physical pixels on scaled displays.
    #[cfg(target_env = "msvc")]
    {
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg=/MANIFESTINPUT:dock-tauri.exe.manifest");
        println!("cargo:rerun-if-changed=dock-tauri.exe.manifest");
    }
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest()),
    )
    .expect("failed to run tauri-build");
}
