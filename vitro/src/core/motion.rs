//! Pure motion calculations. Timers and message dispatch stay in the platform layer.

pub(crate) fn magnification_target(
    cursor: (f32, f32),
    center: (f32, f32),
    radius: f32,
    peak: f32,
) -> f32 {
    let distance = ((cursor.0 - center.0).powi(2) + (cursor.1 - center.1).powi(2))
        .sqrt()
        .min(radius);
    let falloff = 1.0 - distance / radius;
    1.0 + (peak - 1.0) * falloff.powf(1.6)
}

pub(crate) fn spring_step(current: f32, target: f32, dt_ms: u32, rate: f32) -> f32 {
    let amount = (dt_ms as f32 / 1000.0 * rate).min(1.0);
    current + (target - current) * amount
}

pub(crate) fn settling_toward(scales: &[f32], targets: &[f32], epsilon: f32) -> bool {
    scales.iter().zip(targets).any(|(scale, target)| (*scale - *target).abs() > epsilon)
}

pub(crate) fn needs_animation_timer(
    scales: &[f32],
    targets: &[f32],
    epsilon: f32,
    sliding: bool,
    has_running_items: bool,
    low_power: bool,
) -> bool {
    settling_toward(scales, targets, epsilon) || sliding || (has_running_items && !low_power)
}

/// Advance a subdock's open/close transition without overshooting either end.
pub(crate) fn subdock_progress_step(current: f32, closing: bool, dt_ms: u32, duration_ms: f32) -> f32 {
    let amount = (dt_ms as f32 / duration_ms.max(1.0)).clamp(0.0, 1.0);
    if closing {
        (current - amount).max(0.0)
    } else {
        (current + amount).min(1.0)
    }
}

pub(crate) fn subdock_hover_effect(hovered: bool) -> (f32, f32) {
    if hovered { (1.08, 2.0) } else { (1.0, 0.0) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_scale_is_peak_at_cursor() {
        assert!((magnification_target((0.0, 0.0), (0.0, 0.0), 100.0, 1.6) - 1.6).abs() < 0.001);
    }

    #[test]
    fn spring_step_moves_toward_target_without_overshoot() {
        let value = spring_step(1.0, 1.6, 16, 14.0);
        assert!(value > 1.0 && value < 1.6);
    }

    #[test]
    fn settling_ignores_already_resting_scales() {
        assert!(!settling_toward(&[1.0, 1.202], &[1.0, 1.2], 0.004));
        assert!(settling_toward(&[1.0, 1.02], &[1.0, 1.2], 0.004));
    }

    #[test]
    fn static_hover_does_not_keep_animation_timer_alive() {
        assert!(!needs_animation_timer(&[1.4], &[1.4], 0.004, false, false, false));
    }

    #[test]
    fn moving_spring_and_running_breath_keep_timer_alive() {
        assert!(needs_animation_timer(&[1.1], &[1.4], 0.004, false, false, false));
        assert!(needs_animation_timer(&[1.0], &[1.0], 0.004, false, true, false));
        assert!(!needs_animation_timer(&[1.0], &[1.0], 0.004, false, true, true));
    }

    #[test]
    fn subdock_progress_closes_without_overshoot() {
        assert!((subdock_progress_step(0.5, true, 45, 180.0) - 0.25).abs() < 0.001);
        assert_eq!(subdock_progress_step(0.1, true, 180, 180.0), 0.0);
    }

    #[test]
    fn subdock_progress_opens_without_overshoot() {
        assert!((subdock_progress_step(0.5, false, 45, 180.0) - 0.75).abs() < 0.001);
        assert_eq!(subdock_progress_step(0.9, false, 180, 180.0), 1.0);
    }

    #[test]
    fn subdock_hover_lifts_and_scales_only_the_active_item() {
        assert_eq!(subdock_hover_effect(true), (1.08, 2.0));
        assert_eq!(subdock_hover_effect(false), (1.0, 0.0));
    }
}
