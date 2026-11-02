use dock_tauri_lib::appearance::DockMetrics;
use dock_tauri_lib::geometry::{DockGeometry, MonitorRect};
use dock_tauri_lib::settings::Settings;
use dock_tauri_lib::window_geom::route_size;

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
fn dock_route_uses_the_pill_geometry() {
    let dock = DockGeometry::compute(monitor(), 6, metrics());
    let (w, h) = route_size("dock", monitor(), 6, &metrics());
    assert_eq!(w, dock.width);
    assert_eq!(h, dock.height);
}

#[test]
fn launchpad_route_fills_the_work_area() {
    let (w, h) = route_size("launchpad", monitor(), 6, &metrics());
    assert_eq!(w, 1920.0);
    assert_eq!(h, 1040.0);
}

#[test]
fn settings_route_is_a_fixed_logical_panel() {
    let (w, h) = route_size("settings", monitor(), 6, &metrics());
    assert_eq!(w, 820.0);
    assert_eq!(h, 680.0);
}

#[test]
fn settings_route_keeps_its_css_size_at_high_dpi() {
    let mut high_dpi = monitor();
    high_dpi.scale = 1.5;
    let (w, h) = route_size("settings", high_dpi, 6, &metrics());
    assert_eq!(w, 820.0 * 1.5);
    assert_eq!(h, 680.0 * 1.5);
}

#[test]
fn monitor_route_is_taller_than_the_dock() {
    let (_, dock_h) = route_size("dock", monitor(), 6, &metrics());
    let (_, monitor_h) = route_size("monitor", monitor(), 6, &metrics());
    assert!(monitor_h > dock_h);
}

#[test]
fn shelf_route_matches_the_monitor_panel() {
    assert_eq!(
        route_size("shelf", monitor(), 6, &metrics()),
        route_size("monitor", monitor(), 6, &metrics())
    );
}

#[test]
fn monitor_route_scales_its_extra_height_with_dpi() {
    let mut high_dpi = monitor();
    high_dpi.scale = 1.5;
    let (_, dock_h) = route_size("dock", high_dpi, 6, &metrics());
    let (_, monitor_h) = route_size("monitor", high_dpi, 6, &metrics());
    assert_eq!(monitor_h - dock_h, 320.0 * 1.5);
}

#[test]
fn monitor_route_scales_its_minimum_width_with_dpi() {
    let mut high_dpi = monitor();
    high_dpi.scale = 1.5;
    let (width, _) = route_size("monitor", high_dpi, 0, &metrics());
    assert_eq!(width, 360.0 * 1.5);
}
