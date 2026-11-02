//! Native pointer watcher: mouse transparency, pill-based auto-hide, and
//! fullscreen yield.
//!
//! CSS cannot make a transparent window pixel pass a click to another process;
//! the window itself still receives it. This thread reads the cursor and
//! toggles `set_ignore_cursor_events`, using the pill rect React measures.
//! Auto-hide moves here too, so it no longer depends on the webview receiving
//! mouse events for a window-sized area.

use crate::appearance::DockMetrics;
use crate::autohide::{Action, Event as AutoEvent, Visibility};
use crate::commands::{self, Runtime};
use crate::hit_test::{any_contains, physical_rect, pill_rect, poll_interval_ms, Rect};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

const FULLSCREEN_CHECK: Duration = Duration::from_millis(500);
const PANEL_POLL: Duration = Duration::from_millis(100);

pub fn spawn(app: AppHandle) {
    let Some(window) = app.get_webview_window("dock") else {
        return;
    };
    let Ok(hwnd) = window.hwnd() else {
        return;
    };
    // Store the raw handle: the `windows` crate version used by Tauri differs
    // from ours, but the underlying pointer is the same.
    let hwnd = hwnd.0 as isize;
    std::thread::spawn(move || run(app, hwnd));
}

fn elapsed_ms(start: Instant) -> u64 {
    start.elapsed().as_millis().min(u64::MAX as u128) as u64
}

fn set_ignore(window: &tauri::WebviewWindow, last: &mut Option<bool>, ignore: bool) {
    if *last == Some(ignore) {
        return;
    }
    if window.set_ignore_cursor_events(ignore).is_ok() {
        *last = Some(ignore);
    }
}

fn screen_rect() -> Option<Rect> {
    crate::platform::primary_screen_rect()
        .map(|monitor| Rect::from_edges(monitor.left, monitor.top, monitor.right, monitor.bottom))
}

/// Is a real fullscreen window in front? The desktop shell covers the screen
/// but must not hide the dock.
fn fullscreen_foreground(own_hwnd: isize) -> bool {
    let Some(foreground) = crate::platform::foreground_window(own_hwnd) else {
        return false;
    };
    if foreground.is_desktop {
        return false;
    }
    let Some(screen) = screen_rect() else {
        return false;
    };
    crate::foreground::covers_monitor(foreground.rect, screen, 2)
}

fn run(app: AppHandle, own_hwnd: isize) {
    let start = Instant::now();
    let mut last_ignore: Option<bool> = None;
    let mut last_inside = false;
    let mut last_fullscreen_check: Option<Instant> = None;

    loop {
        let runtime = app.state::<Runtime>();
        let Some(window) = app.get_webview_window("dock") else {
            return;
        };
        let now = elapsed_ms(start);
        let route = runtime
            .route
            .lock()
            .map(|route| route.clone())
            .unwrap_or_else(|_| "dock".to_string());
        let Ok(settings) = runtime.settings.lock().map(|s| s.clone()) else {
            std::thread::sleep(PANEL_POLL);
            continue;
        };

        if route != "dock" {
            // Panels fill the window and are fully interactive. Fullscreen
            // suppression only applies to the dock surface.
            set_ignore(&window, &mut last_ignore, false);
            // Pretend the pointer was inside: when the dock returns with the
            // pointer elsewhere, the first frame below emits PointerLeft and
            // arms the hide instead of leaving a visible dock stranded.
            last_inside = true;
            let suppressed = runtime
                .autohide
                .lock()
                .map(|machine| machine.suppressed())
                .unwrap_or(false);
            if suppressed {
                {
                    let Ok(mut machine) = runtime.autohide.lock() else {
                        continue;
                    };
                    machine.handle(AutoEvent::SetSuppressed(false), now);
                }
                let _ = window.show();
                last_ignore = None;
            }
            std::thread::sleep(PANEL_POLL);
            continue;
        }

        // --- tile drag --------------------------------------------------------
        // A drag owns the pointer until it is released. Keeping the whole
        // window interactive is what lets the webview keep receiving moves when
        // the tile is dragged above the pill, and a locked dock cannot collapse
        // out from under the gesture.
        let dragging = runtime.drag.lock().map(|drag| *drag).unwrap_or(false);
        if dragging {
            set_ignore(&window, &mut last_ignore, false);
            last_inside = true;
            std::thread::sleep(Duration::from_millis(16));
            continue;
        }

        // --- fullscreen yield -------------------------------------------------
        let mut suppressed = runtime
            .autohide
            .lock()
            .map(|machine| machine.suppressed())
            .unwrap_or(false);
        let check_due = last_fullscreen_check
            .map(|checked| checked.elapsed() >= FULLSCREEN_CHECK)
            .unwrap_or(true);
        if check_due {
            last_fullscreen_check = Some(Instant::now());
            let fullscreen = settings.yield_fullscreen && fullscreen_foreground(own_hwnd);
            if fullscreen != suppressed {
                {
                    let Ok(mut machine) = runtime.autohide.lock() else {
                        continue;
                    };
                    machine.handle(AutoEvent::SetSuppressed(fullscreen), now);
                }
                suppressed = fullscreen;
                last_inside = false;
                if suppressed {
                    let _ = window.hide();
                    set_ignore(&window, &mut last_ignore, true);
                } else {
                    let _ = window.show();
                    let visible = runtime
                        .autohide
                        .lock()
                        .map(|machine| machine.visibility() == Visibility::Visible)
                        .unwrap_or(true);
                    let _ = commands::apply_autohide_geometry(&app, &runtime, visible);
                    last_ignore = None;
                }
            }
        } else if !settings.yield_fullscreen && suppressed {
            // The setting was turned off while a fullscreen app is still up.
            {
                let Ok(mut machine) = runtime.autohide.lock() else {
                    continue;
                };
                machine.handle(AutoEvent::SetSuppressed(false), now);
            }
            suppressed = false;
            last_inside = false;
            let _ = window.show();
            let visible = runtime
                .autohide
                .lock()
                .map(|machine| machine.visibility() == Visibility::Visible)
                .unwrap_or(true);
            let _ = commands::apply_autohide_geometry(&app, &runtime, visible);
            last_ignore = None;
        }

        if suppressed {
            set_ignore(&window, &mut last_ignore, true);
            std::thread::sleep(PANEL_POLL);
            continue;
        }

        // --- pointer presence and click-through -------------------------------
        let Some(pointer) = crate::platform::cursor_position() else {
            std::thread::sleep(PANEL_POLL);
            continue;
        };
        let Some(window_rect) = crate::platform::window_rect(own_hwnd) else {
            std::thread::sleep(PANEL_POLL);
            continue;
        };

        let visibility = runtime
            .autohide
            .lock()
            .map(|machine| machine.visibility())
            .unwrap_or(Visibility::Visible);
        let hidden = visibility == Visibility::Hidden;

        // The collapsed window is the full-width hot strip. When visible, the
        // pill's position and height come from the same metrics that sized the
        // window; only its measured width comes from the DOM. This survives
        // auto-hide viewport changes without a stale CSS x/y.
        let hit = runtime.hit.lock().ok().and_then(|hit| hit.clone());
        let (click_rects, pointer_rects): (Vec<Rect>, Vec<Rect>) = if hidden {
            (vec![window_rect], vec![window_rect])
        } else if let Some(report) = hit.filter(|report| report.pill_width_css > 0.0) {
            let metrics = DockMetrics::from_settings(&settings);
            let pill = pill_rect(
                window_rect,
                report.pill_width_css,
                report.scale,
                metrics.pill_height(),
                metrics.reflection_space,
            );
            let mut clicks = Vec::with_capacity(report.rects.len() + 1);
            clicks.push(pill);
            // The reflection band below the pill keeps the dock visible when
            // the pointer is at the screen edge, but it stays click-through.
            let mut presence = Vec::with_capacity(report.rects.len() + 1);
            presence.push(pill.extended_down(window_rect.bottom));

            for rect in &report.rects {
                let overlay = physical_rect(
                    (window_rect.left, window_rect.top),
                    rect.left as f64,
                    rect.top as f64,
                    rect.width() as f64,
                    rect.height() as f64,
                    report.scale,
                );
                clicks.push(overlay);
                presence.push(overlay);
            }
            (clicks, presence)
        } else {
            // No measurement yet: treat the whole window as interactive.
            // Blocking a click for one frame is safe; passing a click through
            // the visible pill is not.
            (vec![window_rect], vec![window_rect])
        };

        let inside = any_contains(&pointer_rects, pointer.0, pointer.1);
        let click_through = !any_contains(&click_rects, pointer.0, pointer.1);

        if inside != last_inside {
            let action = runtime
                .autohide
                .lock()
                .map(|mut machine| {
                    machine.handle(
                        if inside {
                            AutoEvent::PointerEntered
                        } else {
                            AutoEvent::PointerLeft
                        },
                        now,
                    )
                })
                .unwrap_or(Action::None);
            last_inside = inside;
            if action != Action::None {
                let visible = action == Action::Reveal;
                let _ = commands::apply_autohide_geometry(&app, &runtime, visible);
                last_ignore = None;
            }
        }

        let action = runtime
            .autohide
            .lock()
            .map(|mut machine| machine.handle(AutoEvent::Tick, now))
            .unwrap_or(Action::None);
        if action != Action::None {
            let visible = action == Action::Reveal;
            let _ = commands::apply_autohide_geometry(&app, &runtime, visible);
            // The pointer was on the hot strip; keep it "inside" so a reveal
            // does not immediately schedule a hide.
            last_inside = visible;
            last_ignore = None;
        }

        set_ignore(&window, &mut last_ignore, click_through);

        let poll_rect = click_rects.first().copied().unwrap_or(window_rect);
        std::thread::sleep(Duration::from_millis(poll_interval_ms(pointer, poll_rect)));
    }
}
