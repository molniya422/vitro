//! Start Menu scanning, app identity, launch, and window reconciliation.
//!
//! Pure functions take explicit inputs so they test without Win32. Only the
//! `resolve_shortcut_target` and `launch` functions touch the platform.

use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppEntry {
    /// Stable identity: normalised absolute shortcut path.
    pub id: String,
    pub display_name: String,
    /// Resolved executable path when known; used for reconciliation.
    pub target: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindowRef {
    pub process_path: String,
    pub minimized: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunningInfo {
    pub running: bool,
    pub minimized: u32,
}

fn normalize(path: &str) -> String {
    path.replace('\\', "/").to_ascii_lowercase()
}

/// Match each app to the windows that belong to it.
pub fn reconcile(apps: &[AppEntry], windows: &[WindowRef]) -> Vec<RunningInfo> {
    apps.iter()
        .map(|app| {
            let mut running = false;
            let mut minimized = 0;
            if let Some(target) = app.target.as_deref() {
                let target = normalize(target);
                for window in windows {
                    if normalize(&window.process_path) == target {
                        running = true;
                        if window.minimized {
                            minimized += 1;
                        }
                    }
                }
            }
            RunningInfo { running, minimized }
        })
        .collect()
}

/// Scan directories for `.lnk` shortcuts. Missing directories are skipped.
pub fn catalog_from_dirs(dirs: &[PathBuf]) -> Vec<AppEntry> {
    let mut entries: Vec<AppEntry> = Vec::new();
    // Start Menu duplicates the same shortcut across the per-user and all-users
    // roots, so the display name is the identity that collapses them.
    let mut seen: Vec<String> = Vec::new();

    for dir in dirs {
        let Ok(read) = std::fs::read_dir(dir) else {
            continue;
        };
        for item in read.flatten() {
            let path = item.path();
            if path.extension().and_then(|e| e.to_str()) != Some("lnk") {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let key = stem.to_ascii_lowercase();
            if seen.contains(&key) {
                continue;
            }
            seen.push(key);
            entries.push(AppEntry {
                id: normalize(&path.to_string_lossy()),
                display_name: stem.to_string(),
                target: resolve_shortcut_target(&path),
            });
        }
    }

    entries.sort_by_key(|a| a.display_name.to_ascii_lowercase());
    entries
}

/// Standard Start Menu roots for this machine.
pub fn start_menu_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(program_data) = std::env::var("ProgramData") {
        dirs.push(Path::new(&program_data).join("Microsoft/Windows/Start Menu/Programs"));
    }
    if let Ok(appdata) = std::env::var("APPDATA") {
        dirs.push(Path::new(&appdata).join("Microsoft/Windows/Start Menu/Programs"));
    }
    dirs
}

#[cfg(windows)]
fn resolve_shortcut_target(lnk: &Path) -> Option<String> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::{Interface, PCWSTR};
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, IPersistFile, CLSCTX_INPROC_SERVER,
        COINIT_APARTMENTTHREADED, STGM_READ,
    };
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink, SLGP_RAWPATH};

    fn wide(value: &std::ffi::OsStr) -> Vec<u16> {
        value.encode_wide().chain(std::iter::once(0)).collect()
    }

    // SAFETY: COM is initialised for the duration of this call and uninitialised
    // only when this call is what initialised it. The interface pointer is
    // created and released locally.
    unsafe {
        // Tauri may already have initialised this thread as STA, in which case
        // CoInitializeEx succeeds (S_FALSE) and the matching CoUninitialize is
        // correct. If the thread uses another apartment model it returns
        // RPC_E_CHANGED_MODE; uninitialising then would tear down someone
        // else's COM apartment, so only balance a success.
        let we_initialised = CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_ok();
        let result = (|| -> Option<String> {
            let link: IShellLinkW =
                CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).ok()?;
            let persist: IPersistFile = link.cast().ok()?;
            let path = wide(lnk.as_os_str());
            persist.Load(PCWSTR(path.as_ptr()), STGM_READ).ok()?;
            let mut buffer = [0u16; 1024];
            link.GetPath(&mut buffer, std::ptr::null_mut(), SLGP_RAWPATH.0 as u32)
                .ok()?;
            let end = buffer.iter().position(|c| *c == 0).unwrap_or(buffer.len());
            if end == 0 {
                return None;
            }
            Some(String::from_utf16_lossy(&buffer[..end]))
        })();
        if we_initialised {
            CoUninitialize();
        }
        result
    }
}

#[cfg(not(windows))]
fn resolve_shortcut_target(_lnk: &Path) -> Option<String> {
    None
}

/// Launch a dock item. Returns a typed error message on failure.
pub fn launch(id: &str) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows::core::PCWSTR;
        use windows::Win32::UI::Shell::ShellExecuteW;
        use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

        let path = id.replace('/', "\\");
        let wide: Vec<u16> = std::ffi::OsStr::new(&path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let verb: Vec<u16> = "open".encode_utf16().chain(std::iter::once(0)).collect();
        // SAFETY: both strings are NUL-terminated and outlive the call.
        let result = unsafe {
            ShellExecuteW(
                None,
                PCWSTR(verb.as_ptr()),
                PCWSTR(wide.as_ptr()),
                PCWSTR::null(),
                PCWSTR::null(),
                SW_SHOWNORMAL,
            )
        };
        // ShellExecuteW returns a value greater than 32 on success.
        if result.0 as usize > 32 {
            Ok(())
        } else {
            Err(format!(
                "ShellExecuteW failed with code {}",
                result.0 as usize
            ))
        }
    }
    #[cfg(not(windows))]
    {
        let _ = id;
        Err("launch is only supported on Windows".into())
    }
}
