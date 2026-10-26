//! Translate a route name into window dimensions.

use crate::geometry::{DockGeometry, MonitorRect, ICON_GAP};

/// The control tile and the two route entry points that precede the app tiles.
const LEADING_CONTROLS: usize = 3;

/// Width of the leading tiles including one gap after each, so the window is
/// exactly as wide as the rendered pill.
fn leading_width(icon_size: f64) -> f64 {
    LEADING_CONTROLS as f64 * (icon_size + ICON_GAP)
}

/// Physical width and height for `route` on `monitor`.
pub fn route_size(
    route: &str,
    monitor: MonitorRect,
    item_count: usize,
    icon_size: f64,
) -> (f64, f64) {
    match route {
        "launchpad" => (
            (monitor.right - monitor.left) as f64,
            (monitor.bottom - monitor.top) as f64,
        ),
        "settings" => (720.0, 560.0),
        "monitor" => {
            let dock =
                DockGeometry::compute(monitor, item_count, icon_size, leading_width(icon_size));
            (dock.width.max(360.0), dock.height + 320.0)
        }
        _ => {
            let dock =
                DockGeometry::compute(monitor, item_count, icon_size, leading_width(icon_size));
            (dock.width, dock.height)
        }
    }
}
