//! Pure auto-hide state machine. Time is passed in, never read here.

pub const SHOW_DELAY_MS: u64 = 300;
pub const HIDE_DELAY_MS: u64 = 500;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visibility {
    Visible,
    Hidden,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    PointerEntered,
    PointerLeft,
    Tick,
    SetEnabled(bool),
    SetLocked(bool),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    None,
    Reveal,
    Collapse,
}

#[derive(Clone, Copy, Debug)]
pub struct AutoHide {
    enabled: bool,
    locked: bool,
    visibility: Visibility,
    /// Deadline after which the pending transition should fire.
    deadline: Option<u64>,
    /// Which transition the deadline refers to.
    pending: Option<Action>,
}

impl AutoHide {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            locked: false,
            // With auto-hide on, the dock starts collapsed until the pointer
            // approaches; with it off, `visibility()` reports Visible anyway.
            visibility: if enabled {
                Visibility::Hidden
            } else {
                Visibility::Visible
            },
            deadline: None,
            pending: None,
        }
    }

    pub fn visibility(&self) -> Visibility {
        if !self.enabled || self.locked {
            Visibility::Visible
        } else {
            self.visibility
        }
    }

    /// Whether auto-hide is currently active (enabled and not locked).
    pub fn enabled(&self) -> bool {
        self.enabled && !self.locked
    }

    /// True while a delayed transition is armed and a tick is still needed.
    pub fn pending(&self) -> bool {
        self.deadline.is_some()
    }

    pub fn set_locked(&mut self, locked: bool) {
        self.locked = locked;
        if locked {
            self.deadline = None;
            self.pending = None;
        }
    }

    pub fn handle(&mut self, event: Event, now_ms: u64) -> Action {
        match event {
            Event::SetEnabled(enabled) => {
                self.enabled = enabled;
                if !enabled {
                    self.visibility = Visibility::Visible;
                    self.deadline = None;
                    self.pending = None;
                }
                Action::None
            }
            Event::SetLocked(locked) => {
                self.set_locked(locked);
                Action::None
            }
            Event::PointerEntered => {
                self.deadline = None;
                self.pending = None;
                if self.enabled && !self.locked && self.visibility == Visibility::Hidden {
                    self.deadline = Some(now_ms.saturating_add(SHOW_DELAY_MS));
                    self.pending = Some(Action::Reveal);
                }
                Action::None
            }
            Event::PointerLeft => {
                self.deadline = None;
                self.pending = None;
                if self.enabled && !self.locked && self.visibility == Visibility::Visible {
                    self.deadline = Some(now_ms.saturating_add(HIDE_DELAY_MS));
                    self.pending = Some(Action::Collapse);
                }
                Action::None
            }
            Event::Tick => {
                if !self.enabled || self.locked {
                    return Action::None;
                }
                let (Some(deadline), Some(pending)) = (self.deadline, self.pending) else {
                    return Action::None;
                };
                if now_ms < deadline {
                    return Action::None;
                }
                self.deadline = None;
                self.pending = None;
                match pending {
                    Action::Reveal => {
                        self.visibility = Visibility::Visible;
                        Action::Reveal
                    }
                    Action::Collapse => {
                        self.visibility = Visibility::Hidden;
                        Action::Collapse
                    }
                    Action::None => Action::None,
                }
            }
        }
    }
}
