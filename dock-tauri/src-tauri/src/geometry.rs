//! Pure per-monitor window geometry. No Win32 calls — inputs are plain numbers.

use crate::appearance::DockMetrics;

/// A monitor's work area in physical pixels plus its DPI scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MonitorRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    /// DPI scale factor: 1.0 at 96 DPI, 1.5 at 144 DPI.
    pub scale: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DockGeometry {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    /// Height of the glass pill in physical pixels.
    pub pill_height: f64,
    /// Space reserved above the pill for magnified icons and tooltips.
    pub headroom: f64,
    /// Space reserved below the pill for the reflection.
    pub reflection: f64,
}

impl DockGeometry {
    /// Compute the dock window rect for the bottom edge of `monitor`.
    ///
    /// `item_count` counts app tiles only; the control tile and the enabled
    /// route entries are in [`DockMetrics::leading_tiles`].
    pub fn compute(monitor: MonitorRect, item_count: usize, metrics: DockMetrics) -> Self {
        let scale = if monitor.scale > 0.0 && monitor.scale.is_finite() {
            monitor.scale
        } else {
            1.0
        };
        let work_width = ((monitor.right - monitor.left) as f64).max(0.0);
        let work_height = ((monitor.bottom - monitor.top) as f64).max(0.0);

        // Every visible tile has the same width and one gap sits between each
        // pair. The control tile is always present, so `children >= 1`.
        let children = metrics.leading_tiles.max(1) + item_count;
        let gaps = children.saturating_sub(1) as f64;
        let content = metrics.padding_h * 2.0
            + children as f64 * metrics.icon_size
            + gaps * metrics.icon_gap;
        let mut width = content * scale;

        let headroom = metrics.headroom * scale;
        let reflection = metrics.reflection_space * scale;
        let mut pill_height = metrics.pill_height() * scale;
        let mut height = headroom + pill_height + reflection;

        // Clamp to the work area. Width must be clamped before centering so the
        // pill can never start off-screen.
        if width > work_width {
            width = work_width;
        }
        if height > work_height {
            height = work_height;
            pill_height = (height - headroom - reflection).max(0.0);
        }

        Self {
            x: (monitor.left as f64) + (work_width - width) / 2.0,
            y: (monitor.bottom as f64) - height,
            width,
            height,
            pill_height,
            headroom,
            reflection,
        }
    }

    /// Top edge of the glass pill — the shared origin for paint and hit test.
    pub fn pill_top(&self) -> f64 {
        self.y + self.headroom
    }
}
