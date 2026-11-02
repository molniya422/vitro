//! Pure fullscreen predicate. The Win32 lookup lives in `platform`.

use crate::hit_test::Rect;

/// True when `window` covers `screen` within `tolerance_px` on every edge.
///
/// A maximised window stops at the work area (above the taskbar) and therefore
/// does not count; only true fullscreen surfaces suppress the dock.
pub fn covers_monitor(window: Rect, screen: Rect, tolerance_px: i32) -> bool {
    window.left <= screen.left + tolerance_px
        && window.top <= screen.top + tolerance_px
        && window.right >= screen.right - tolerance_px
        && window.bottom >= screen.bottom - tolerance_px
}
