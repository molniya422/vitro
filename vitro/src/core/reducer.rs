use super::event::{BehaviorEvent, Effect};
use super::model::BehaviorState;

pub(crate) fn reduce(state: &mut BehaviorState, event: BehaviorEvent) -> Vec<Effect> {
    use super::event::{BehaviorEvent::*, Timer};

    let mut effects = Vec::new();
    match event {
        PointerMoved { x, y, near, hit, now_ms } => {
            let was_near = state.pointer_near;
            let summon = near && state.slide_target != 0.0;
            if summon {
                state.slide_target = 0.0;
            }

            let cancel_hide = near && state.hide_armed;
            if cancel_hide {
                state.hide_armed = false;
            }
            let arm_hide = !near && was_near && state.autohide && state.slide_target == 0.0;
            if arm_hide {
                state.hide_armed = true;
            }

            state.pointer_near = near;
            state.pointer = (x as f32, y as f32);

            if hit != state.hovered {
                state.hovered = hit;
                state.hover_since = now_ms;
                if state.tooltip.take().is_some() {
                    effects.push(Effect::HideTooltip);
                }
                if hit.is_some() {
                    effects.push(Effect::SetTimer { timer: Timer::TooltipDwell, interval_ms: 120 });
                } else {
                    effects.push(Effect::KillTimer(Timer::TooltipDwell));
                }
            }

            if cancel_hide {
                effects.push(Effect::KillTimer(Timer::AutoHide));
            }
            if arm_hide {
                effects.push(Effect::SetTimer { timer: Timer::AutoHide, interval_ms: 700 });
            }
            if summon {
                effects.push(Effect::SetTimer { timer: Timer::Animation, interval_ms: 33 });
            }
            effects.push(Effect::PublishHookSnapshot);
            effects.push(Effect::Render);
        }
        DwellElapsed { now_ms } => {
            if let Some(tile) = state.hovered {
                if state.tooltip.is_none()
                    && now_ms.wrapping_sub(state.hover_since) >= 200
                {
                    state.tooltip = Some(tile);
                    effects.push(Effect::KillTimer(Timer::TooltipDwell));
                    effects.push(Effect::ShowTooltip(tile));
                }
            }
        }
        HideElapsed { slide_progress } => {
            state.hide_armed = false;
            effects.push(Effect::KillTimer(Timer::AutoHide));
            if state.autohide && state.slide_target == 0.0 && slide_progress < 0.5 {
                state.slide_target = 1.0;
                state.hovered = None;
                if state.tooltip.take().is_some() {
                    effects.push(Effect::HideTooltip);
                }
                effects.push(Effect::SetTimer { timer: Timer::Animation, interval_ms: 33 });
                effects.push(Effect::PublishHookSnapshot);
                effects.push(Effect::Render);
            }
        }
        TileRemoved(tile) => {
            if state.hovered == Some(tile) {
                state.hovered = None;
                effects.push(Effect::KillTimer(Timer::TooltipDwell));
            }
            if state.tooltip == Some(tile) {
                state.tooltip = None;
                effects.push(Effect::HideTooltip);
            }
        }
        ShowDock => {
            let changed = state.slide_target != 0.0;
            state.slide_target = 0.0;
            state.hide_armed = false;
            effects.push(Effect::KillTimer(Timer::AutoHide));
            if changed {
                effects.push(Effect::SetTimer { timer: Timer::Animation, interval_ms: 33 });
            }
            effects.push(Effect::PublishHookSnapshot);
            effects.push(Effect::Render);
        }
        ToggleAutohide => {
            state.autohide = !state.autohide;
            let reveal = !state.autohide && state.slide_target != 0.0;
            if reveal {
                state.slide_target = 0.0;
                effects.push(Effect::SetTimer { timer: Timer::Animation, interval_ms: 33 });
            }
            effects.push(Effect::SaveAutohide(state.autohide));
            effects.push(Effect::PublishHookSnapshot);
            effects.push(Effect::Render);
        }
    }
    effects
}

#[cfg(test)]
mod tests {
    use super::super::event::{BehaviorEvent, Effect, Timer};
    use super::super::model::{BehaviorState, TileId};
    use super::reduce;

    fn tile(value: u64) -> TileId {
        TileId(value)
    }

    fn has_effect(effects: &[Effect], expected: &Effect) -> bool {
        effects.contains(expected)
    }

    #[test]
    fn entering_dock_updates_pointer_and_arms_tooltip_dwell() {
        let mut state = BehaviorState::new(false);
        let effects = reduce(
            &mut state,
            BehaviorEvent::PointerMoved {
                x: 500,
                y: 850,
                near: true,
                hit: Some(tile(9)),
                now_ms: 100,
            },
        );

        assert!(state.pointer_near);
        assert_eq!(state.pointer, (500.0, 850.0));
        assert_eq!(state.hovered, Some(tile(9)));
        assert_eq!(state.hover_since, 100);
        assert!(has_effect(
            &effects,
            &Effect::SetTimer { timer: Timer::TooltipDwell, interval_ms: 120 }
        ));
        assert!(has_effect(&effects, &Effect::PublishHookSnapshot));
        assert!(has_effect(&effects, &Effect::Render));
    }

    #[test]
    fn leaving_dock_arms_700ms_hide_and_clears_hover_and_tooltip() {
        let mut state = BehaviorState::new(true);
        state.pointer_near = true;
        state.hovered = Some(tile(9));
        state.tooltip = Some(tile(9));

        let effects = reduce(
            &mut state,
            BehaviorEvent::PointerMoved {
                x: 50,
                y: 50,
                near: false,
                hit: None,
                now_ms: 300,
            },
        );

        assert!(!state.pointer_near);
        assert_eq!(state.hovered, None);
        assert_eq!(state.tooltip, None);
        assert!(state.hide_armed);
        assert!(has_effect(
            &effects,
            &Effect::SetTimer { timer: Timer::AutoHide, interval_ms: 700 }
        ));
        assert!(has_effect(&effects, &Effect::KillTimer(Timer::TooltipDwell)));
        assert!(has_effect(&effects, &Effect::HideTooltip));
    }

    #[test]
    fn reentering_near_dock_cancels_an_armed_hide_timer() {
        let mut state = BehaviorState::new(true);
        state.hide_armed = true;

        let effects = reduce(
            &mut state,
            BehaviorEvent::PointerMoved {
                x: 500,
                y: 850,
                near: true,
                hit: None,
                now_ms: 320,
            },
        );

        assert!(!state.hide_armed);
        assert!(has_effect(&effects, &Effect::KillTimer(Timer::AutoHide)));
    }

    #[test]
    fn moving_between_tiles_resets_the_hover_dwell_timestamp() {
        let mut state = BehaviorState::new(false);
        state.pointer_near = true;
        state.hovered = Some(tile(1));
        state.hover_since = 100;

        let effects = reduce(
            &mut state,
            BehaviorEvent::PointerMoved {
                x: 540,
                y: 850,
                near: true,
                hit: Some(tile(2)),
                now_ms: 150,
            },
        );

        assert_eq!(state.hovered, Some(tile(2)));
        assert_eq!(state.hover_since, 150);
        assert!(has_effect(
            &effects,
            &Effect::SetTimer { timer: Timer::TooltipDwell, interval_ms: 120 }
        ));
    }

    #[test]
    fn moving_from_a_visible_tooltip_to_another_tile_hides_the_old_tooltip() {
        let mut state = BehaviorState::new(false);
        state.pointer_near = true;
        state.hovered = Some(tile(1));
        state.tooltip = Some(tile(1));

        let effects = reduce(
            &mut state,
            BehaviorEvent::PointerMoved {
                x: 540,
                y: 850,
                near: true,
                hit: Some(tile(2)),
                now_ms: 150,
            },
        );

        assert_eq!(state.hovered, Some(tile(2)));
        assert_eq!(state.tooltip, None);
        assert!(has_effect(&effects, &Effect::HideTooltip));
        assert!(has_effect(
            &effects,
            &Effect::SetTimer { timer: Timer::TooltipDwell, interval_ms: 120 }
        ));
    }

    #[test]
    fn dwell_timer_waits_for_200ms_then_shows_tooltip() {
        let id = tile(9);
        let mut state = BehaviorState::new(false);
        state.hovered = Some(id);
        state.hover_since = 100;

        let early = reduce(&mut state, BehaviorEvent::DwellElapsed { now_ms: 299 });
        assert_eq!(state.tooltip, None);
        assert!(!early.iter().any(|effect| matches!(effect, Effect::ShowTooltip(_))));

        let elapsed = reduce(&mut state, BehaviorEvent::DwellElapsed { now_ms: 300 });
        assert_eq!(state.tooltip, Some(id));
        assert!(has_effect(&elapsed, &Effect::KillTimer(Timer::TooltipDwell)));
        assert!(has_effect(&elapsed, &Effect::ShowTooltip(id)));
    }

    #[test]
    fn dwell_elapsed_uses_wrapping_tick_arithmetic() {
        let id = tile(2);
        let mut state = BehaviorState::new(false);
        state.hovered = Some(id);
        state.hover_since = u32::MAX - 99;

        reduce(&mut state, BehaviorEvent::DwellElapsed { now_ms: 100 });

        assert_eq!(state.tooltip, Some(id));
    }

    #[test]
    fn hide_elapsed_hides_only_below_the_halfway_threshold() {
        let mut state = BehaviorState::new(true);
        state.hovered = Some(tile(3));
        state.tooltip = Some(tile(3));

        let effects = reduce(
            &mut state,
            BehaviorEvent::HideElapsed { slide_progress: 0.49 },
        );

        assert!(!state.hide_armed);
        assert_eq!(state.slide_target, 1.0);
        assert_eq!(state.hovered, None);
        assert_eq!(state.tooltip, None);
        assert!(has_effect(&effects, &Effect::KillTimer(Timer::AutoHide)));
        assert!(has_effect(&effects, &Effect::HideTooltip));
        assert!(has_effect(
            &effects,
            &Effect::SetTimer { timer: Timer::Animation, interval_ms: 33 }
        ));
    }

    #[test]
    fn hide_elapsed_does_not_hide_at_or_above_halfway_or_when_disabled() {
        let mut state = BehaviorState::new(true);
        let at_half = reduce(
            &mut state,
            BehaviorEvent::HideElapsed { slide_progress: 0.5 },
        );
        assert_eq!(state.slide_target, 0.0);
        assert!(!at_half.iter().any(|effect| matches!(effect, Effect::SetTimer { timer: Timer::Animation, .. })));

        state.autohide = false;
        let disabled = reduce(
            &mut state,
            BehaviorEvent::HideElapsed { slide_progress: 0.1 },
        );
        assert_eq!(state.slide_target, 0.0);
        assert!(!disabled.iter().any(|effect| matches!(effect, Effect::SetTimer { timer: Timer::Animation, .. })));
    }

    #[test]
    fn show_dock_cancels_hide_and_animates_only_when_target_changes() {
        let mut state = BehaviorState::new(true);
        state.slide_target = 1.0;
        state.hide_armed = true;

        let effects = reduce(&mut state, BehaviorEvent::ShowDock);

        assert_eq!(state.slide_target, 0.0);
        assert!(!state.hide_armed);
        assert!(has_effect(&effects, &Effect::KillTimer(Timer::AutoHide)));
        assert!(has_effect(
            &effects,
            &Effect::SetTimer { timer: Timer::Animation, interval_ms: 33 }
        ));
        assert!(has_effect(&effects, &Effect::PublishHookSnapshot));
        assert!(has_effect(&effects, &Effect::Render));
    }

    #[test]
    fn toggling_autohide_saves_and_reveals_a_hidden_dock_when_disabled() {
        let mut state = BehaviorState::new(true);
        state.slide_target = 1.0;

        let effects = reduce(&mut state, BehaviorEvent::ToggleAutohide);

        assert!(!state.autohide);
        assert_eq!(state.slide_target, 0.0);
        assert!(has_effect(&effects, &Effect::SaveAutohide(false)));
        assert!(has_effect(
            &effects,
            &Effect::SetTimer { timer: Timer::Animation, interval_ms: 33 }
        ));
        assert!(has_effect(&effects, &Effect::PublishHookSnapshot));
        assert!(has_effect(&effects, &Effect::Render));
    }

    #[test]
    fn toggling_autohide_on_does_not_hide_an_already_visible_dock() {
        let mut state = BehaviorState::new(false);
        let effects = reduce(&mut state, BehaviorEvent::ToggleAutohide);

        assert!(state.autohide);
        assert_eq!(state.slide_target, 0.0);
        assert!(has_effect(&effects, &Effect::SaveAutohide(true)));
        assert!(!effects.iter().any(|effect| matches!(effect, Effect::SetTimer { timer: Timer::Animation, .. })));
    }

    #[test]
    fn removing_a_hovered_or_tooltip_tile_clears_its_stable_identity() {
        let mut state = BehaviorState::new(false);
        state.hovered = Some(tile(7));
        state.tooltip = Some(tile(7));

        let effects = reduce(&mut state, BehaviorEvent::TileRemoved(tile(7)));

        assert_eq!(state.hovered, None);
        assert_eq!(state.tooltip, None);
        assert!(has_effect(&effects, &Effect::KillTimer(Timer::TooltipDwell)));
        assert!(has_effect(&effects, &Effect::HideTooltip));
    }
}
