//! Dock layout geometry independent from screen/window APIs.

pub(crate) fn pill_width(n: usize, win_w: i32, icon: f32, gap: f32, pad: f32) -> i32 {
    if n == 0 {
        return (2.0 * pad) as i32;
    }
    let width = n as f32 * icon + (n - 1) as f32 * gap + 2.0 * pad;
    (width as i32).min(win_w - 40).max(2 * pad as i32)
}

pub(crate) fn tile_size(pill_w: i32, n: usize, icon: f32, gap: f32, pad: f32) -> f32 {
    if n == 0 {
        return icon;
    }
    let available = (pill_w as f32 - 2.0 * pad).max(60.0);
    ((available / n as f32) - gap).clamp(34.0, icon)
}

pub(crate) fn tile_center_local(
    pill_w: i32,
    n: usize,
    index: usize,
    icon: f32,
    gap: f32,
    pad: f32,
    panel_h: f32,
    base_up: f32,
) -> (f32, f32, f32) {
    let size = tile_size(pill_w, n, icon, gap, pad);
    let x = pad + size / 2.0
        + index as f32 * (size + gap);
    let y = panel_h - base_up - size / 2.0;
    (x, y, size)
}

pub(crate) fn running_indicator_y(panel_h: f32, base_up: f32, icon_size: f32, offset: f32) -> f32 {
    let _ = icon_size;
    panel_h - base_up + offset
}

pub(crate) fn tile_top_for_scale(panel_h: f32, base_up: f32, size: f32) -> f32 {
    panel_h - base_up - size
}

pub(crate) fn subdock_top(panel_top: f32, panel_h: f32, gap: f32) -> f32 {
    panel_top - panel_h - gap
}

pub(crate) fn divider_x(pill_w: i32, n: usize, boundary: usize, icon: f32, gap: f32, pad: f32) -> Option<f32> {
    if boundary == 0 || boundary >= n { return None; }
    let size = tile_size(pill_w, n, icon, gap, pad);
    Some(pad + boundary as f32 * (size + gap) - gap / 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn width_is_bounded_by_canvas() {
        assert_eq!(pill_width(40, 800, 56.0, 13.0, 24.0), 760);
    }

    #[test]
    fn centers_are_symmetric() {
        let x = tile_center_local(300, 3, 0, 56.0, 13.0, 24.0, 92.0, 12.0).0;
        assert!((x - 24.0 - 56.0 / 2.0).abs() < 0.01);
    }

    #[test]
    fn running_indicator_sits_six_pixels_below_the_icon() {
        let y = running_indicator_y(68.0, 10.0, 48.0, 6.0);
        assert!((y - 64.0).abs() < 0.01);
        assert!(y < 68.0);
    }

    #[test]
    fn magnified_tile_keeps_its_bottom_edge_on_the_baseline() {
        let top = tile_top_for_scale(68.0, 10.0, 48.0 * 1.2);
        assert!((top + 48.0 * 1.2 - 58.0).abs() < 0.01);
    }

    #[test]
    fn divider_is_centered_in_the_gap_between_sections() {
        let x = divider_x(248, 4, 2, 48.0, 8.0, 16.0).unwrap();
        assert!((x - 120.0).abs() < 0.01);
        assert!(divider_x(248, 4, 0, 48.0, 8.0, 16.0).is_none());
    }

    #[test]
    fn subdock_has_a_sixteen_pixel_bridge_to_the_main_panel() {
        assert!((subdock_top(220.0, 68.0, 16.0) - 136.0).abs() < 0.01);
    }
}
