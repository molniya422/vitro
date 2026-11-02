//! Win32 display adapter. The only place display geometry leaves Win32.

use crate::geometry::MonitorRect;
use crate::hit_test::Rect;

/// The foreground window, when it is visible and not minimised.
#[derive(Clone, Copy, Debug)]
pub struct ForegroundWindow {
    pub rect: Rect,
    /// Explorer's desktop windows cover the screen but must not hide the dock.
    pub is_desktop: bool,
}

#[cfg(windows)]
mod imp {
    use super::{ForegroundWindow, MonitorRect};
    use crate::hit_test::Rect;
    use windows::Win32::Foundation::{HWND, POINT, RECT};
    use windows::Win32::UI::HiDpi::GetDpiForSystem;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetClassNameW, GetCursorPos, GetForegroundWindow, GetSystemMetrics, GetWindowRect, IsIconic,
        IsWindowVisible, SetForegroundWindow, ShowWindow, SystemParametersInfoW, SM_CXSCREEN,
        SM_CYSCREEN, SPI_GETWORKAREA, SW_RESTORE, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
    };

    fn system_scale() -> f64 {
        // SAFETY: GetDpiForSystem takes no arguments and returns the system DPI.
        // A zero result is treated as 100%.
        let dpi = unsafe { GetDpiForSystem() };
        if dpi == 0 {
            1.0
        } else {
            dpi as f64 / 96.0
        }
    }

    fn rect_to_monitor(rect: RECT, scale: f64) -> MonitorRect {
        MonitorRect {
            left: rect.left,
            top: rect.top,
            right: rect.right,
            bottom: rect.bottom,
            scale,
        }
    }

    pub fn primary_work_area() -> Option<MonitorRect> {
        let mut rect = RECT::default();
        // SAFETY: SPI_GETWORKAREA writes exactly one RECT into the pointer given,
        // which stays live for the duration of the call.
        let result = unsafe {
            SystemParametersInfoW(
                SPI_GETWORKAREA,
                0,
                Some(&mut rect as *mut RECT as *mut core::ffi::c_void),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
            )
        };
        result.ok().map(|_| rect_to_monitor(rect, system_scale()))
    }

    pub fn primary_screen_rect() -> Option<MonitorRect> {
        // SAFETY: both calls take no arguments and return plain integers.
        let (w, h) = unsafe { (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN)) };
        if w <= 0 || h <= 0 {
            return None;
        }
        Some(MonitorRect {
            left: 0,
            top: 0,
            right: w,
            bottom: h,
            scale: system_scale(),
        })
    }

    pub fn cursor_position() -> Option<(i32, i32)> {
        let mut point = POINT::default();
        // SAFETY: GetCursorPos writes one POINT into a live out-parameter.
        unsafe { GetCursorPos(&mut point) }.ok().map(|_| (point.x, point.y))
    }

    pub fn window_rect(hwnd: isize) -> Option<Rect> {
        if hwnd == 0 {
            return None;
        }
        let hwnd = HWND(hwnd as *mut core::ffi::c_void);
        if hwnd.is_invalid() {
            return None;
        }
        let mut rect = RECT::default();
        // SAFETY: the handle is checked and the RECT is a local out-parameter.
        unsafe { GetWindowRect(hwnd, &mut rect) }.ok()?;
        Some(Rect::from_edges(
            rect.left, rect.top, rect.right, rect.bottom,
        ))
    }

    /// Bring a running app's window forward. A minimized window is restored
    /// first; the foreground change is granted because the click that triggered
    /// it landed on our window.
    pub fn activate_window(hwnd: isize) {
        let hwnd = HWND(hwnd as *mut core::ffi::c_void);
        if hwnd.is_invalid() {
            return;
        }
        // SAFETY: the handle is checked above and both calls only affect it.
        unsafe {
            if IsIconic(hwnd).as_bool() {
                let _ = ShowWindow(hwnd, SW_RESTORE);
            }
            let _ = SetForegroundWindow(hwnd);
        }
    }

    pub fn foreground_window(exclude_hwnd: isize) -> Option<ForegroundWindow> {
        // SAFETY: every handle comes from the system, is checked, and is only
        // read through; no window is mutated.
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.is_invalid() || hwnd.0 as isize == exclude_hwnd {
                return None;
            }
            if !IsWindowVisible(hwnd).as_bool() || IsIconic(hwnd).as_bool() {
                return None;
            }
            let mut rect = RECT::default();
            if GetWindowRect(hwnd, &mut rect).is_err() {
                return None;
            }
            let mut class = [0u16; 64];
            let len = GetClassNameW(hwnd, &mut class).max(0) as usize;
            let class = String::from_utf16_lossy(&class[..len.min(class.len())]);
            Some(ForegroundWindow {
                rect: Rect::from_edges(rect.left, rect.top, rect.right, rect.bottom),
                is_desktop: class == "Progman" || class == "WorkerW",
            })
        }
    }
}

#[cfg(windows)]
pub use imp::{
    activate_window, cursor_position, foreground_window, primary_screen_rect, primary_work_area,
    window_rect,
};

#[cfg(not(windows))]
pub fn primary_work_area() -> Option<MonitorRect> {
    None
}

#[cfg(not(windows))]
pub fn primary_screen_rect() -> Option<MonitorRect> {
    None
}

#[cfg(not(windows))]
pub fn cursor_position() -> Option<(i32, i32)> {
    None
}

#[cfg(not(windows))]
pub fn window_rect(_hwnd: isize) -> Option<Rect> {
    None
}

#[cfg(not(windows))]
pub fn activate_window(_hwnd: isize) {}

#[cfg(not(windows))]
pub fn foreground_window(_exclude_hwnd: isize) -> Option<ForegroundWindow> {
    None
}
