use dock_tauri_lib::hit_test::{
    any_contains, physical_rect, pill_rect, poll_interval_ms, Rect,
};

#[test]
fn contains_uses_half_open_edges() {
    let rect = Rect::from_edges(10, 20, 110, 120);
    assert!(rect.contains(10, 20));
    assert!(rect.contains(109, 119));
    assert!(!rect.contains(110, 60));
    assert!(!rect.contains(60, 120));
    assert!(!rect.contains(9, 60));
}

#[test]
fn headroom_and_reflection_are_outside_the_pill_rect() {
    // Pill at y=84 for the default geometry, 68 px tall.
    let pill = Rect::from_edges(500, 84, 800, 152);
    assert!(!pill.contains(600, 40)); // magnification headroom
    assert!(!pill.contains(600, 160)); // reflection band
    assert!(pill.contains(600, 100));
}

#[test]
fn physical_rect_scales_from_css_and_window_origin() {
    let rect = physical_rect((100, 200), 16.0, 40.0, 300.0, 68.0, 1.5);
    assert_eq!(rect, Rect::from_edges(124, 260, 574, 362));
}

#[test]
fn physical_rect_tolerates_a_bad_scale() {
    let rect = physical_rect((0, 0), 10.0, 10.0, 100.0, 20.0, 0.0);
    assert_eq!(rect, Rect::from_edges(10, 10, 110, 30));
}

#[test]
fn any_contains_checks_every_rect() {
    let rects = [
        Rect::from_edges(0, 0, 10, 10),
        Rect::from_edges(100, 100, 150, 150),
    ];
    assert!(any_contains(&rects, 5, 5));
    assert!(any_contains(&rects, 120, 120));
    assert!(!any_contains(&rects, 50, 50));
    assert!(!any_contains(&[], 0, 0));
}

#[test]
fn extended_down_reaches_the_screen_edge_but_never_shrinks() {
    let pill = Rect::from_edges(10, 10, 110, 50);
    assert_eq!(
        pill.extended_down(100),
        Rect::from_edges(10, 10, 110, 100)
    );
    assert_eq!(pill.extended_down(40), pill);
}

#[test]
fn pill_rect_lands_on_the_same_pixels_at_any_viewport_layout() {
    // Default dock window on the 150% display: 1128x288 at (716, 1152).
    let window = Rect::from_edges(716, 1152, 1844, 1440);
    let expanded = pill_rect(window, 752.0, 1.5, 68.0, 40.0);
    assert_eq!(expanded, Rect::from_edges(716, 1278, 1844, 1380));

    // Collapsed: the control tile only, centred in the same window.
    let collapsed = pill_rect(window, 80.0, 1.5, 68.0, 40.0);
    assert_eq!(collapsed, Rect::from_edges(1220, 1278, 1340, 1380));
    assert_eq!(collapsed.width(), 120);
}

#[test]
fn pill_rect_never_exceeds_its_window() {
    let window = Rect::from_edges(0, 0, 100, 100);
    let pill = pill_rect(window, 1000.0, 1.0, 68.0, 10.0);
    assert!(pill.left >= window.left);
    assert!(pill.right <= window.right);
}

#[test]
fn polling_is_fast_near_the_dock_and_slow_away_from_it() {
    let dock = Rect::from_edges(100, 100, 200, 200);
    assert_eq!(poll_interval_ms((150, 150), dock), 16);
    assert_eq!(poll_interval_ms((450, 150), dock), 16); // 300 px reach
    assert_eq!(poll_interval_ms((600, 150), dock), 100);
    assert_eq!(poll_interval_ms((0, 0), dock), 16);
}
