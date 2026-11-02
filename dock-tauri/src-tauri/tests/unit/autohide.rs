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

#[test]
fn configurable_delays_drive_both_transitions() {
    let mut ah = AutoHide::new_with_delays(true, 50, 20);
    ah.handle(Event::PointerEntered, 0);
    assert_eq!(ah.handle(Event::Tick, 49), Action::None);
    assert_eq!(ah.handle(Event::Tick, 50), Action::Reveal);

    ah.handle(Event::PointerLeft, 100);
    assert_eq!(ah.handle(Event::Tick, 119), Action::None);
    assert_eq!(ah.handle(Event::Tick, 120), Action::Collapse);
}

#[test]
fn suppression_hides_a_dock_with_auto_hide_disabled() {
    let mut ah = AutoHide::new(false);
    assert_eq!(ah.handle(Event::SetSuppressed(true), 0), Action::None);
    assert!(ah.suppressed());
    assert_eq!(ah.visibility(), Visibility::Hidden);

    ah.handle(Event::PointerEntered, 10);
    assert_eq!(ah.handle(Event::Tick, 10_000), Action::None);
    assert_eq!(ah.visibility(), Visibility::Hidden);

    ah.handle(Event::SetSuppressed(false), 11_000);
    assert!(!ah.suppressed());
    assert_eq!(ah.visibility(), Visibility::Visible);
}

#[test]
fn suppression_ends_hidden_when_auto_hide_is_enabled() {
    let mut ah = AutoHide::new(true);
    ah.handle(Event::SetSuppressed(true), 0);
    ah.handle(Event::PointerEntered, 1);
    assert_eq!(ah.handle(Event::Tick, 10_000), Action::None);

    ah.handle(Event::SetSuppressed(false), 10_000);
    assert_eq!(ah.visibility(), Visibility::Hidden);
    ah.handle(Event::PointerEntered, 20_000);
    assert_eq!(ah.handle(Event::Tick, 20_300), Action::Reveal);
}

#[test]
fn suppression_cancels_an_armed_hide() {
    let mut ah = AutoHide::new(true);
    ah.handle(Event::PointerEntered, 0);
    ah.handle(Event::Tick, 300);
    ah.handle(Event::PointerLeft, 1000);
    assert!(ah.pending());

    ah.handle(Event::SetSuppressed(true), 1100);
    assert!(!ah.pending());
    ah.handle(Event::SetSuppressed(false), 1200);
    assert_eq!(ah.visibility(), Visibility::Hidden);
    assert_eq!(ah.handle(Event::Tick, 60_000), Action::None);
}

