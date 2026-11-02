//! Pointer gating for the low-level hook, isolated from Win32 and UI state.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ScreenRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PointerSnapshot {
    pub pill: ScreenRect,
    pub screen_height: i32,
    pub autohide: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PointerRouteState {
    pub was_near: bool,
    pub last_post_tick: Option<u32>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PointerDecision {
    pub near: bool,
    pub post: bool,
}

pub(crate) fn route_pointer(
    snapshot: PointerSnapshot,
    previous: PointerRouteState,
    x: i32,
    y: i32,
    now: u32,
) -> (PointerRouteState, PointerDecision) {
    let r = snapshot.pill;
    let bottom_edge = snapshot.autohide && y >= snapshot.screen_height.saturating_sub(3);
    let near_dock = x >= r.left.saturating_sub(200)
        && x <= r.right.saturating_add(200)
        && y >= r.top.saturating_sub(200)
        && y <= r.bottom.saturating_add(200);
    let near = near_dock || bottom_edge;
    let region_changed = previous.was_near != near;
    let elapsed = previous
        .last_post_tick
        .map(|last| now.wrapping_sub(last) >= 8)
        .unwrap_or(true);
    // Deliver entry/exit edges immediately so the UI observes brief crossings;
    // throttle only the repeated samples while the pointer stays in the region.
    let post = region_changed || ((near || previous.was_near) && elapsed);

    let next = PointerRouteState {
        was_near: near,
        last_post_tick: if post {
            Some(now)
        } else {
            previous.last_post_tick
        },
    };
    (next, PointerDecision { near, post })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot() -> PointerSnapshot {
        PointerSnapshot {
            pill: ScreenRect {
                left: 400,
                top: 800,
                right: 600,
                bottom: 900,
            },
            screen_height: 1000,
            autohide: false,
        }
    }

    #[test]
    fn posts_nearby_samples_at_most_every_eight_ms_and_posts_the_leave_edge() {
        let (state, entered) = route_pointer(snapshot(), PointerRouteState::default(), 400, 800, 100);
        assert!(entered.post);
        assert!(entered.near);

        let (state, throttled) = route_pointer(snapshot(), state, 401, 801, 103);
        assert!(!throttled.post);

        let (state, resumed) = route_pointer(snapshot(), state, 402, 802, 108);
        assert!(resumed.post);

        let (_, left) = route_pointer(snapshot(), state, 100, 100, 109);
        assert!(left.post);
        assert!(!left.near);
    }

    #[test]
    fn bottom_edge_only_activates_when_autohide_is_enabled() {
        let mut config = snapshot();
        let point = (20, 998);
        let (_, disabled) = route_pointer(config, PointerRouteState::default(), point.0, point.1, 100);
        assert!(!disabled.near);

        config.autohide = true;
        let (_, enabled) = route_pointer(config, PointerRouteState::default(), point.0, point.1, 100);
        assert!(enabled.near);
        assert!(enabled.post);
    }

    #[test]
    fn region_transitions_bypass_the_interior_sample_throttle() {
        let previous = PointerRouteState {
            was_near: false,
            last_post_tick: Some(100),
        };
        let (state, entered) = route_pointer(snapshot(), previous, 400, 800, 103);
        assert!(entered.post);
        assert!(entered.near);

        let (_, left) = route_pointer(snapshot(), state, 100, 100, 104);
        assert!(left.post);
        assert!(!left.near);
    }

    #[test]
    fn distant_mouse_samples_do_not_post_messages() {
        let (_, decision) = route_pointer(snapshot(), PointerRouteState::default(), 10, 10, 100);
        assert!(!decision.near);
        assert!(!decision.post);
    }
}
