use dock_tauri_lib::appearance::DockMetrics;
use dock_tauri_lib::geometry::{DockGeometry, MonitorRect};
use dock_tauri_lib::settings::Settings;

fn monitor() -> MonitorRect {
    MonitorRect {
        left: 0,
        top: 0,
        right: 1920,
        bottom: 1040,
        scale: 1.0,
    }
}

fn metrics() -> DockMetrics {
    DockMetrics::from_settings(&Settings::default())
}

#[test]
fn dock_window_is_centered_on_the_work_area_bottom() {
    let g = DockGeometry::compute(monitor(), 6, metrics());
    // Four leading tiles + six apps, nine gaps, two horizontal paddings.
    assert_eq!(g.width, 4.0 * 48.0 + 6.0 * 48.0 + 9.0 * 8.0 + 32.0);
    assert_eq!(g.x, (1920.0 - g.width) / 2.0);
}

#[test]
fn window_includes_magnify_headroom_above_and_reflection_below() {
    let g = DockGeometry::compute(monitor(), 6, metrics());
    assert_eq!(g.headroom, 84.0);
    assert_eq!(g.pill_height, 68.0);
    assert_eq!(g.reflection, 40.0);
    assert_eq!(g.height, 84.0 + 68.0 + 40.0);
    assert_eq!(g.y + g.height, 1040.0);
}

#[test]
fn pill_rect_sits_below_the_magnify_margin() {
    let g = DockGeometry::compute(monitor(), 6, metrics());
    assert_eq!(g.pill_top(), g.y + 84.0);
}

#[test]
fn geometry_scales_with_monitor_dpi() {
    let mut m = monitor();
    m.scale = 1.5;
    let g = DockGeometry::compute(m, 6, metrics());
    assert_eq!(g.width, (4.0 * 48.0 + 6.0 * 48.0 + 9.0 * 8.0 + 32.0) * 1.5);
    assert_eq!(g.height, (84.0 + 68.0 + 40.0) * 1.5);
    assert_eq!(g.y + g.height, 1040.0);
}

#[test]
fn dock_never_exceeds_a_narrow_work_area() {
    let narrow = MonitorRect {
        left: 0,
        top: 0,
        right: 400,
        bottom: 800,
        scale: 1.0,
    };
    let g = DockGeometry::compute(narrow, 20, metrics());
    assert!(g.width <= 400.0);
    assert!(g.x >= 0.0);
}

#[test]
fn an_empty_dock_still_measures_the_control_tile_and_its_gaps() {
    let g = DockGeometry::compute(monitor(), 0, metrics());
    // Control + Launchpad + monitor + shelf entries: 4 tiles, 3 gaps.
    assert_eq!(g.width, 32.0 + 4.0 * 48.0 + 3.0 * 8.0);
}

#[test]
fn custom_metrics_drive_width_height_and_reflection() {
    let settings = Settings {
        icon_size: 80,
        icon_gap: 16,
        padding_h: 24,
        padding_v: 12,
        magnify_peak: 2.0,
        reflection: false,
        ..Settings::default()
    };
    let g = DockGeometry::compute(monitor(), 0, DockMetrics::from_settings(&settings));

    assert_eq!(g.width, 24.0 * 2.0 + 4.0 * 80.0 + 3.0 * 16.0);
    assert_eq!(g.pill_height, 80.0 + 12.0 * 2.0);
    assert_eq!(g.headroom, 80.0 + 34.0 * 2.0);
    assert_eq!(g.reflection, 0.0);
    assert_eq!(g.height, g.headroom + g.pill_height);
}
