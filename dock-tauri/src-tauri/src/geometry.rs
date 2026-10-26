//! Pure per-monitor window geometry. No Win32 calls — inputs are plain numbers.

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

pub const MAGNIFY_HEADROOM: f64 = 84.0;
pub const REFLECTION_MARGIN: f64 = 40.0;
pub const PADDING_H: f64 = 16.0;
pub const PADDING_V: f64 = 10.0;
pub const ICON_GAP: f64 = 8.0;

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
    /// `item_count` counts app tiles only; the control tile is added separately.
    /// `icon_size` and `control_width` are logical pixels before DPI scaling.
    pub fn compute(
        monitor: MonitorRect,
        item_count: usize,
        icon_size: f64,
        control_width: f64,
    ) -> Self {
        let scale = if monitor.scale > 0.0 {
            monitor.scale
        } else {
            1.0
        };
        let work_width = (monitor.right - monitor.left) as f64;
        let work_height = (monitor.bottom - monitor.top) as f64;

        let gaps = if item_count == 0 {
            0.0
        } else {
            (item_count - 1) as f64 * ICON_GAP
        };
        let content = control_width + item_count as f64 * icon_size + gaps + PADDING_H * 2.0;
        let mut width = content * scale;

        let headroom = MAGNIFY_HEADROOM * scale;
        let reflection = REFLECTION_MARGIN * scale;
        let mut pill_height = (icon_size + PADDING_V * 2.0) * scale;
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
