//! Pure rectangles for mouse transparency and the auto-hide hot zone.

/// A physical-pixel rectangle in screen coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    pub fn from_edges(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    /// Build a rectangle from an origin and size, rounding each edge so a
    /// fractional DPI scale cannot leave a one-pixel seam.
    pub fn from_origin_size(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            left: x.round() as i32,
            top: y.round() as i32,
            right: (x + width).round() as i32,
            bottom: (y + height).round() as i32,
        }
    }

    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.left && x < self.right && y >= self.top && y < self.bottom
    }

    pub fn expanded(&self, by: i32) -> Self {
        Self {
            left: self.left - by,
            top: self.top - by,
            right: self.right + by,
            bottom: self.bottom + by,
        }
    }

    /// Extend downward to `bottom`, keeping the horizontal extent.
    ///
    /// The reflection band lives below the pill. Including it in the pointer
    /// presence area stops a reveal from immediately hiding when the pointer is
    /// still at the screen edge; the band is still click-through.
    pub fn extended_down(&self, bottom: i32) -> Self {
        Self {
            left: self.left,
            top: self.top,
            right: self.right,
            bottom: self.bottom.max(bottom),
        }
    }
}

/// Convert a CSS-pixel rect reported by the webview into screen pixels.
pub fn physical_rect(
    window_pos: (i32, i32),
    css_x: f64,
    css_y: f64,
    css_width: f64,
    css_height: f64,
    scale: f64,
) -> Rect {
    let scale = if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    };
    Rect {
        left: window_pos.0 + (css_x * scale).round() as i32,
        top: window_pos.1 + (css_y * scale).round() as i32,
        right: window_pos.0 + ((css_x + css_width) * scale).round() as i32,
        bottom: window_pos.1 + ((css_y + css_height) * scale).round() as i32,
    }
}

/// The visible pill inside a dock window, in physical screen pixels.
///
/// Height and reflection come from the same `DockMetrics` that sized the
/// window, so a report only needs the measured pill width. That width is stable
/// while auto-hide swaps the window between the full dock and the edge strip,
/// unlike a CSS x/y measured in a different viewport layout.
pub fn pill_rect(
    window: Rect,
    pill_width_css: f64,
    scale: f64,
    pill_height_css: f64,
    reflection_css: f64,
) -> Rect {
    let scale = if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    };
    let width = (pill_width_css * scale)
        .round()
        .clamp(0.0, window.width().max(0) as f64) as i32;
    let height = (pill_height_css * scale).round().max(0.0) as i32;
    let reflection = (reflection_css * scale).round().max(0.0) as i32;
    let left = window.left + ((window.width() - width) / 2);
    let bottom = window.bottom - reflection;
    let top = bottom - height;
    Rect {
        left,
        top,
        right: left + width,
        bottom,
    }
}

pub fn any_contains(rects: &[Rect], x: i32, y: i32) -> bool {
    rects.iter().any(|rect| rect.contains(x, y))
}

/// Poll quickly while the pointer is near the dock, slowly when it is away.
///
/// 16 ms keeps the click-through toggle imperceptible without a 60 Hz wakeup
/// while the user is working elsewhere.
pub fn poll_interval_ms(pointer: (i32, i32), dock: Rect) -> u64 {
    const NEAR_PX: i32 = 300;
    if dock.expanded(NEAR_PX).contains(pointer.0, pointer.1) {
        16
    } else {
        100
    }
}
