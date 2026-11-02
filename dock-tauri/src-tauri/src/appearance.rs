//! Derived dock appearance and geometry inputs. Pure: no Win32, no IO.

use crate::settings::Settings;
use serde::Serialize;

/// Logical-pixel measurements geometry needs. DPI scaling is applied once, by
/// [`crate::geometry::DockGeometry::compute`], after these values are derived.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DockMetrics {
    pub icon_size: f64,
    pub icon_gap: f64,
    pub padding_h: f64,
    pub padding_v: f64,
    /// Space above the glass for magnified icons and tooltips.
    pub headroom: f64,
    /// Space below the glass for the reflection.
    pub reflection_space: f64,
    /// The control tile plus whichever route entry points are enabled.
    pub leading_tiles: usize,
}

impl DockMetrics {
    pub fn from_settings(settings: &Settings) -> Self {
        let icon_size = settings.icon_size as f64;
        let icon_gap = settings.icon_gap as f64;
        let padding_h = settings.padding_h as f64;
        let padding_v = settings.padding_v as f64;

        let leading_tiles = 1
            + usize::from(settings.show_launchpad_entry)
            + usize::from(settings.show_monitor_entry)
            + usize::from(settings.show_shelf_entry);

        // Magnified icons grow upward from the pill. The tooltip is a child of
        // the transformed tile, so its reserved space scales too: at the default
        // 1.6 peak this keeps the historical 84 px headroom.
        let magnify_extra = if settings.magnify {
            (icon_size * (settings.magnify_peak - 1.0)).ceil()
        } else {
            0.0
        };
        let tooltip_box = if settings.tooltips { 34.0 } else { 8.0 };
        let tooltip_scale = if settings.magnify {
            settings.magnify_peak
        } else {
            1.0
        };
        let headroom = (magnify_extra + tooltip_box * tooltip_scale).ceil();

        let reflection_space = if settings.reflection {
            (icon_size * 5.0 / 6.0).round()
        } else {
            0.0
        };

        Self {
            icon_size,
            icon_gap,
            padding_h,
            padding_v,
            headroom,
            reflection_space,
            leading_tiles,
        }
    }

    /// Height of the glass pill in logical pixels.
    pub fn pill_height(&self) -> f64 {
        self.icon_size + self.padding_v * 2.0
    }

    /// Physical source pixels for an icon rendered at `icon_size` logical px.
    pub fn device_icon_px(&self, scale: f64) -> u32 {
        device_icon_px(self.icon_size, scale)
    }
}

/// Physical icon source size for a logical size and DPI scale.
///
/// The Shell jumbo image list tops out at 256 px, so larger requests are
/// clamped rather than upscaled; tiny requests get a readable floor.
pub fn device_icon_px(icon_size: f64, scale: f64) -> u32 {
    let scale = if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    };
    if !(icon_size.is_finite() && icon_size > 0.0) {
        return 16;
    }
    (icon_size * scale).round().clamp(16.0, 256.0) as u32
}

/// Everything React needs to render the dock, carrying no environment data.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DockAppearance {
    pub icon_size: f64,
    pub icon_gap: f64,
    pub padding_h: f64,
    pub padding_v: f64,
    pub pill_radius: f64,
    pub tint: String,
    pub fill_opacity: f64,
    pub icon_radius: f64,
    pub reflection: bool,
    pub reflection_strength: f64,
    pub reflection_space: f64,
    pub magnify: bool,
    pub magnify_peak: f64,
    /// Magnify reach in CSS pixels, derived from the icons-by-reach setting.
    pub magnify_radius: f64,
    pub reduced_motion: bool,
    pub profile: String,
    pub tooltips: bool,
    pub tooltip_delay_ms: u32,
    pub show_running_dots: bool,
    pub show_badges: bool,
    pub show_launchpad_entry: bool,
    pub show_monitor_entry: bool,
    pub show_shelf_entry: bool,
    pub leading_tiles: usize,
}

impl DockAppearance {
    pub fn from_settings(settings: &Settings) -> Self {
        let metrics = DockMetrics::from_settings(settings);
        let magnify_radius =
            round2(settings.magnify_radius_icons * (metrics.icon_size + metrics.icon_gap));

        Self {
            icon_size: metrics.icon_size,
            icon_gap: metrics.icon_gap,
            padding_h: metrics.padding_h,
            padding_v: metrics.padding_v,
            pill_radius: settings.pill_radius as f64,
            tint: settings.tint.clone(),
            fill_opacity: settings.fill_opacity,
            icon_radius: settings.icon_radius as f64,
            reflection: settings.reflection,
            reflection_strength: settings.reflection_strength,
            reflection_space: metrics.reflection_space,
            magnify: settings.magnify,
            magnify_peak: settings.magnify_peak,
            magnify_radius,
            reduced_motion: settings.reduced_motion,
            profile: settings.profile.clone(),
            tooltips: settings.tooltips,
            tooltip_delay_ms: settings.tooltip_delay_ms,
            show_running_dots: settings.show_running_dots,
            show_badges: settings.show_badges,
            show_launchpad_entry: settings.show_launchpad_entry,
            show_monitor_entry: settings.show_monitor_entry,
            show_shelf_entry: settings.show_shelf_entry,
            leading_tiles: metrics.leading_tiles,
        }
    }
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}
