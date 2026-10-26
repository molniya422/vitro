//! Live window enumeration, reconciled against the app catalog.

use crate::apps::{self, AppEntry, RunningInfo, WindowRef};

#[cfg(windows)]
pub fn snapshot_windows() -> Vec<WindowRef> {
    use std::os::windows::ffi::OsStringExt;
    use windows::core::BOOL;
    use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM, MAX_PATH};
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowLongW, GetWindowTextLengthW, GetWindowThreadProcessId, IsIconic,
        IsWindowVisible, GWL_EXSTYLE, WS_EX_TOOLWINDOW,
    };

    struct Collector {
        windows: Vec<WindowRef>,
    }

    unsafe extern "system" fn callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let collector = &mut *(lparam.0 as *mut Collector);

        // Skip invisible windows and tool windows: the dock never mirrors them.
        if !IsWindowVisible(hwnd).as_bool() {
            return BOOL(1);
        }
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
        if ex_style & WS_EX_TOOLWINDOW.0 != 0 {
            return BOOL(1);
        }
        // Skip untitled windows (splash screens, shell internals).
        if GetWindowTextLengthW(hwnd) == 0 {
            return BOOL(1);
        }

        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return BOOL(1);
        }

        let Ok(process) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
            return BOOL(1);
        };
        let mut buffer = [0u16; MAX_PATH as usize];
        let mut size = buffer.len() as u32;
        let ok = QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            windows::core::PWSTR(buffer.as_mut_ptr()),
            &mut size,
        );
        let _ = CloseHandle(process);
        if ok.is_err() || size == 0 {
            return BOOL(1);
        }
        let path = std::ffi::OsString::from_wide(&buffer[..size as usize])
            .to_string_lossy()
            .to_string();

        collector.windows.push(WindowRef {
            process_path: path,
            minimized: IsIconic(hwnd).as_bool(),
        });
        BOOL(1)
    }

    let mut collector = Collector {
        windows: Vec::new(),
    };
    // SAFETY: `collector` outlives the enumeration; the callback writes only into it.
    unsafe {
        let _ = EnumWindows(
            Some(callback),
            LPARAM(&mut collector as *mut Collector as isize),
        );
    }
    collector.windows
}

#[cfg(not(windows))]
pub fn snapshot_windows() -> Vec<WindowRef> {
    Vec::new()
}

pub fn snapshot_and_reconcile(apps: &[AppEntry]) -> Vec<RunningInfo> {
    let windows = snapshot_windows();
    apps::reconcile(apps, &windows)
}
