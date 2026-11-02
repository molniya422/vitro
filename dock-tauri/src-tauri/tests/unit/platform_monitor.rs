use dock_tauri_lib::platform::primary_work_area;

#[test]
fn primary_work_area_is_on_screen_and_sane() {
    let m = primary_work_area().expect("primary monitor");
    assert!(m.right > m.left, "work area must have width");
    assert!(m.bottom > m.top, "work area must have height");
    assert!(
        (1.0..=4.0).contains(&m.scale),
        "scale must be plausible, got {}",
        m.scale
    );
}

#[test]
fn work_area_never_exceeds_the_full_screen() {
    let m = primary_work_area().expect("primary monitor");
    let full = dock_tauri_lib::platform::primary_screen_rect().expect("screen rect");
    assert!(m.bottom - m.top <= full.bottom - full.top);
}
