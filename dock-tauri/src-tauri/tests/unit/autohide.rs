use dock_tauri_lib::autohide::{Action, AutoHide, Event, Visibility};

#[test]
fn disabled_dock_is_always_visible() {
    let mut ah = AutoHide::new(false);
    ah.handle(Event::PointerLeft, 1000);
    assert_eq!(ah.visibility(), Visibility::Visible);
    assert_eq!(ah.handle(Event::Tick, 5000), Action::None);
}

#[test]
fn enter_reveals_after_show_delay() {
    let mut ah = AutoHide::new(true);
    assert_eq!(ah.handle(Event::PointerEntered, 0), Action::None);
    assert_eq!(ah.visibility(), Visibility::Hidden);
    assert_eq!(ah.handle(Event::Tick, 299), Action::None);
    assert_eq!(ah.handle(Event::Tick, 300), Action::Reveal);
    assert_eq!(ah.visibility(), Visibility::Visible);
}

#[test]
fn leave_arms_hide_only_after_hide_delay() {
    let mut ah = AutoHide::new(true);
    ah.handle(Event::PointerEntered, 0);
    ah.handle(Event::Tick, 300);
    assert_eq!(ah.handle(Event::PointerLeft, 1000), Action::None);
    assert_eq!(ah.handle(Event::Tick, 1499), Action::None);
    assert_eq!(ah.handle(Event::Tick, 1500), Action::Collapse);
    assert_eq!(ah.visibility(), Visibility::Hidden);
}

#[test]
fn reentering_cancels_an_armed_hide() {
    let mut ah = AutoHide::new(true);
    ah.handle(Event::PointerEntered, 0);
    ah.handle(Event::Tick, 300);
    ah.handle(Event::PointerLeft, 1000);
    ah.handle(Event::PointerEntered, 1200);
    assert_eq!(ah.handle(Event::Tick, 5000), Action::None);
    assert_eq!(ah.visibility(), Visibility::Visible);
}

#[test]
fn a_failed_timer_cannot_leave_the_dock_stuck_visible() {
    let mut ah = AutoHide::new(true);
    ah.handle(Event::PointerEntered, 0);
    ah.handle(Event::Tick, 300);
    ah.handle(Event::PointerLeft, 1000);
    // No Tick arrives; a late tick well past the deadline must still collapse.
    assert_eq!(ah.handle(Event::Tick, 60_000), Action::Collapse);
    assert_eq!(ah.visibility(), Visibility::Hidden);
}

#[test]
fn lock_forces_visible_and_ignores_pointer() {
    let mut ah = AutoHide::new(true);
    ah.set_locked(true);
    ah.handle(Event::PointerLeft, 0);
    assert_eq!(ah.handle(Event::Tick, 10_000), Action::None);
    assert_eq!(ah.visibility(), Visibility::Visible);
}
