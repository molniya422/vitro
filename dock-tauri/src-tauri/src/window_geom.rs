//! Translate a route name into window dimensions.

use crate::appearance::DockMetrics;
use crate::geometry::{DockGeometry, MonitorRect};

/// Physical width and height for `route` on `monitor`.
pub fn route_size(
    route: &str,
    monitor: MonitorRect,
    item_count: usize,
    metrics: &DockMetrics,
) -> (f64, f64) {
    let scale = if monitor.scale > 0.0 && monitor.scale.is_finite() {
        monitor.scale
    } else {
        1.0
    };

    match route {
        "launchpad" => (
            (monitor.right - monitor.left) as f64,
            (monitor.bottom - monitor.top) as f64,
        ),
        // Logical panel size, scaled so it keeps the same CSS dimensions at
        // every DPI. The previous fixed 720x560 was physical pixels and became
        // cramped at 150%.
        "settings" => (820.0 * scale, 680.0 * scale),
        // Both panels sit above the dock and share one geometry.
        "monitor" | "shelf" => {
            let dock = DockGeometry::compute(monitor, item_count, *metrics);
            // The extra height and the minimum width are logical values; scale
            // them so the panel keeps its CSS height at every DPI.
            (dock.width.max(360.0 * scale), dock.height + 320.0 * scale)
        }
        _ => {
            let dock = DockGeometry::compute(monitor, item_count, *metrics);
            (dock.width, dock.height)
        }
    }
}
