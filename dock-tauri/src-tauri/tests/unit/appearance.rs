use dock_tauri_lib::appearance::{device_icon_px, DockAppearance, DockMetrics};
use dock_tauri_lib::settings::Settings;

fn defaults() -> Settings {
    Settings::default()
}

fn with(mutate: impl FnOnce(&mut Settings)) -> Settings {
    let mut settings = defaults();
    mutate(&mut settings);
    settings
}

#[test]
fn defaults_derive_the_expected_metrics() {
    let metrics = DockMetrics::from_settings(&defaults());
    assert_eq!(metrics.icon_size, 48.0);
    assert_eq!(metrics.icon_gap, 8.0);
    assert_eq!(metrics.padding_h, 16.0);
    assert_eq!(metrics.padding_v, 10.0);
    assert_eq!(metrics.pill_height(), 68.0);
    assert_eq!(metrics.headroom, 84.0);
    assert_eq!(metrics.reflection_space, 40.0);
    assert_eq!(metrics.leading_tiles, 4);

    let appearance = DockAppearance::from_settings(&defaults());
    assert!((appearance.magnify_radius - 123.2).abs() < 1e-9);
}

#[test]
fn headroom_grows_with_magnify_peak_and_shrinks_without_tooltips() {
    let peak_two = with(|s| s.magnify_peak = 2.0);
    assert_eq!(DockMetrics::from_settings(&peak_two).headroom, 116.0);

    let no_tooltips = with(|s| {
        s.magnify_peak = 2.0;
        s.tooltips = false;
    });
    assert_eq!(DockMetrics::from_settings(&no_tooltips).headroom, 64.0);

    let no_magnify = with(|s| s.magnify = false);
    assert_eq!(DockMetrics::from_settings(&no_magnify).headroom, 34.0);
}

#[test]
fn reflection_off_removes_the_reserved_space() {
    let settings = with(|s| s.reflection = false);
    assert_eq!(DockMetrics::from_settings(&settings).reflection_space, 0.0);
}

#[test]
fn leading_tiles_follow_route_entry_toggles() {
    let all = DockMetrics::from_settings(&defaults());
    assert_eq!(all.leading_tiles, 4);

    let two = with(|s| s.show_launchpad_entry = false);
    assert_eq!(DockMetrics::from_settings(&two).leading_tiles, 3);

    let one = with(|s| {
        s.show_launchpad_entry = false;
        s.show_monitor_entry = false;
    });
    assert_eq!(DockMetrics::from_settings(&one).leading_tiles, 2);

    let none = with(|s| {
        s.show_launchpad_entry = false;
        s.show_monitor_entry = false;
        s.show_shelf_entry = false;
    });
    assert_eq!(DockMetrics::from_settings(&none).leading_tiles, 1);

    let appearance = DockAppearance::from_settings(&none);
    assert_eq!(appearance.leading_tiles, 1);
    assert!(!appearance.show_shelf_entry);
}

#[test]
fn appearance_carries_every_css_value() {
    let appearance = DockAppearance::from_settings(&defaults());
    assert_eq!(appearance.icon_size, 48.0);
    assert_eq!(appearance.icon_gap, 8.0);
    assert_eq!(appearance.padding_h, 16.0);
    assert_eq!(appearance.padding_v, 10.0);
    assert_eq!(appearance.pill_radius, 34.0);
    assert_eq!(appearance.tint, "#161a26");
    assert_eq!(appearance.fill_opacity, 0.62);
    assert_eq!(appearance.icon_radius, 22.0);
    assert_eq!(appearance.reflection_space, 40.0);
    assert_eq!(appearance.magnify_peak, 1.6);

    let json = serde_json::to_value(&appearance).unwrap();
    assert!(json.get("iconSize").is_some());
    assert!(json.get("icon_size").is_none());
    assert!(json.get("magnifyRadius").is_some());
    assert_eq!(json["leadingTiles"], 4);
    assert_eq!(json["showShelfEntry"], true);
}

#[test]
fn device_icon_px_is_rounded_and_clamped() {
    assert_eq!(device_icon_px(48.0, 1.5), 72);
    assert_eq!(device_icon_px(80.0, 3.0), 240);
    assert_eq!(device_icon_px(8.0, 1.0), 16);
    assert_eq!(device_icon_px(999.0, 1.0), 256);
    assert_eq!(device_icon_px(48.0, 0.0), 48);
}
