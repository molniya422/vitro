use dock_tauri_lib::geometry::{DockGeometry, MonitorRect};

fn monitor() -> MonitorRect {
    MonitorRect {
        left: 0,
        top: 0,
        right: 1920,
        bottom: 1040,
        scale: 1.0,
    }
}

#[test]
fn dock_window_is_centered_on_the_work_area_bottom() {
    let g = DockGeometry::compute(monitor(), 6, 48.0, 180.0);
    assert_eq!(g.width, 180.0 + 6.0 * 48.0 + 5.0 * 8.0 + 32.0);
    assert_eq!(g.x, (1920.0 - g.width) / 2.0);
}

#[test]
fn window_includes_magnify_headroom_above_and_reflection_below() {
    let g = DockGeometry::compute(monitor(), 6, 48.0, 180.0);
    assert!(g.height >= 84.0 + 68.0 + 40.0);
    assert_eq!(g.y + g.height, 1040.0);
}

#[test]
fn pill_rect_sits_below_the_magnify_margin() {
    let g = DockGeometry::compute(monitor(), 6, 48.0, 180.0);
    assert_eq!(g.pill_top(), g.y + 84.0);
    assert_eq!(g.pill_height, 68.0);
}

#[test]
fn geometry_scales_with_monitor_dpi() {
    let mut m = monitor();
    m.scale = 1.5;
    let g = DockGeometry::compute(m, 6, 48.0, 180.0);
    assert_eq!(g.width, (180.0 + 6.0 * 48.0 + 5.0 * 8.0 + 32.0) * 1.5);
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
    let g = DockGeometry::compute(narrow, 20, 48.0, 180.0);
    assert!(g.width <= 400.0);
    assert!(g.x >= 0.0);
}
