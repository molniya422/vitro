//! Win32 display adapter. The only place display geometry leaves Win32.

use crate::geometry::MonitorRect;

#[cfg(windows)]
mod imp {
    use super::MonitorRect;
    use windows::Win32::Foundation::RECT;
    use windows::Win32::UI::HiDpi::GetDpiForSystem;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetSystemMetrics, SystemParametersInfoW, SM_CXSCREEN, SM_CYSCREEN, SPI_GETWORKAREA,
        SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
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
}

#[cfg(windows)]
pub use imp::{primary_screen_rect, primary_work_area};

#[cfg(not(windows))]
pub fn primary_work_area() -> Option<MonitorRect> {
    None
}

#[cfg(not(windows))]
pub fn primary_screen_rect() -> Option<MonitorRect> {
    None
}
