use dock_tauri_lib::foreground::covers_monitor;
use dock_tauri_lib::hit_test::Rect;

fn screen() -> Rect {
    Rect::from_edges(0, 0, 2560, 1440)
}

#[test]
fn a_fullscreen_window_covers_the_monitor() {
    assert!(covers_monitor(
        Rect::from_edges(0, 0, 2560, 1440),
        screen(),
        2
    ));
}

#[test]
fn a_maximised_window_stops_at_the_work_area_and_does_not_count() {
    // Taskbar occupies the bottom 40 px.
    assert!(!covers_monitor(
        Rect::from_edges(0, 0, 2560, 1400),
        screen(),
        2
    ));
}

#[test]
fn tolerance_accepts_a_borderless_window_a_pixel_short() {
    assert!(covers_monitor(
        Rect::from_edges(0, 0, 2560, 1438),
        screen(),
        2
    ));
    assert!(!covers_monitor(
        Rect::from_edges(0, 0, 2560, 1437),
        screen(),
        2
    ));
}

#[test]
fn an_offset_fullscreen_window_still_counts() {
    assert!(covers_monitor(
        Rect::from_edges(-2, 0, 2558, 1440),
        screen(),
        2
    ));
}

#[test]
fn a_small_window_never_counts() {
    assert!(!covers_monitor(
        Rect::from_edges(100, 100, 800, 600),
        screen(),
        2
    ));
}
