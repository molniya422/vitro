// WH_MOUSE_LL performs only atomic snapshot gating and posts coordinates. UI
// state transitions and all timer operations stay in the Dock WndProc.
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicIsize, AtomicU32, Ordering};

use windows::Win32::Foundation::*;
use windows::Win32::System::SystemInformation::GetTickCount;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::core::input::{route_pointer, PointerRouteState, PointerSnapshot, ScreenRect};
use crate::dockwin::render_frame;
use crate::runtime::AppRuntime;
use crate::state::{State, WM_APP_MOUSE};

struct HookSnapshot {
    hwnd: AtomicIsize,
    left: AtomicI32,
    top: AtomicI32,
    right: AtomicI32,
    bottom: AtomicI32,
    screen_height: AtomicI32,
    autohide: AtomicBool,
}

impl HookSnapshot {
    const fn new() -> Self {
        Self {
            hwnd: AtomicIsize::new(0),
            left: AtomicI32::new(0),
            top: AtomicI32::new(0),
            right: AtomicI32::new(0),
            bottom: AtomicI32::new(0),
            screen_height: AtomicI32::new(0),
            autohide: AtomicBool::new(false),
        }
    }

    fn publish(&self, state: &State, autohide: bool) {
        let rect = state.pill_rect_screen();
        self.left.store(rect.left, Ordering::Relaxed);
        self.top.store(rect.top, Ordering::Relaxed);
        self.right.store(rect.right, Ordering::Relaxed);
        self.bottom.store(rect.bottom, Ordering::Relaxed);
        self.screen_height.store(state.screen_h, Ordering::Relaxed);
        self.autohide.store(autohide, Ordering::Relaxed);
        self.hwnd.store(state.hwnd.0 as isize, Ordering::Release);
    }

    fn read(&self) -> Option<(HWND, PointerSnapshot)> {
        let hwnd = self.hwnd.load(Ordering::Acquire);
        if hwnd == 0 {
            return None;
        }
        Some((
            HWND(hwnd as *mut std::ffi::c_void),
            PointerSnapshot {
                pill: ScreenRect {
                    left: self.left.load(Ordering::Relaxed),
                    top: self.top.load(Ordering::Relaxed),
                    right: self.right.load(Ordering::Relaxed),
                    bottom: self.bottom.load(Ordering::Relaxed),
                },
                screen_height: self.screen_height.load(Ordering::Relaxed),
                autohide: self.autohide.load(Ordering::Relaxed),
            },
        ))
    }

    fn clear(&self) {
        self.hwnd.store(0, Ordering::Release);
    }
}

static HOOK_SNAPSHOT: HookSnapshot = HookSnapshot::new();
static ROUTE_WAS_NEAR: AtomicBool = AtomicBool::new(false);
static ROUTE_HAS_LAST_POST: AtomicBool = AtomicBool::new(false);
static ROUTE_LAST_POST: AtomicU32 = AtomicU32::new(0);

pub(crate) fn publish_hook_snapshot(state: &State, autohide: bool) {
    HOOK_SNAPSHOT.publish(state, autohide);
}

pub(crate) fn clear_hook_snapshot() {
    HOOK_SNAPSHOT.clear();
}

pub(crate) fn reset_pointer_route() {
    ROUTE_WAS_NEAR.store(false, Ordering::Relaxed);
    ROUTE_HAS_LAST_POST.store(false, Ordering::Relaxed);
    ROUTE_LAST_POST.store(0, Ordering::Relaxed);
}

pub(crate) unsafe extern "system" fn ll_hook(_code: i32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if wp.0 as u32 == WM_MOUSEMOVE {
        if let Some((hwnd, snapshot)) = HOOK_SNAPSHOT.read() {
            let mouse = &*(lp.0 as *const MSLLHOOKSTRUCT);
            let (x, y) = (mouse.pt.x, mouse.pt.y);
            let previous = PointerRouteState {
                was_near: ROUTE_WAS_NEAR.load(Ordering::Relaxed),
                last_post_tick: ROUTE_HAS_LAST_POST
                    .load(Ordering::Relaxed)
                    .then(|| ROUTE_LAST_POST.load(Ordering::Relaxed)),
            };
            let now = GetTickCount();
            let (next, decision) = route_pointer(snapshot, previous, x, y, now);
            ROUTE_WAS_NEAR.store(next.was_near, Ordering::Relaxed);
            if let Some(last) = next.last_post_tick {
                ROUTE_LAST_POST.store(last, Ordering::Relaxed);
                ROUTE_HAS_LAST_POST.store(true, Ordering::Relaxed);
            }
            if decision.post {
                let _ = PostMessageW(hwnd, WM_APP_MOUSE, WPARAM(x as usize), LPARAM(y as isize));
            }
        }
    }
    CallNextHookEx(None, 0, wp, lp)
}

// WM_POWERBROADCAST(PBT_APMPOWERSTATUSCHANGE) remains message driven.
pub(crate) unsafe fn update_power(runtime: *mut AppRuntime) {
    use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
    let mut status = SYSTEM_POWER_STATUS::default();
    if GetSystemPowerStatus(&mut status).is_ok() {
        let (hwnd, changed, low_power) = {
            let state = &mut (*runtime).state;
            let was = state.low_power;
            state.low_power = status.ACLineStatus == 0;
            (state.hwnd, was != state.low_power, state.low_power)
        };
        if changed {
            if low_power {
                let _ = KillTimer(hwnd, crate::state::TIMER_ANIM as usize);
                (*runtime).state.anim_on = false;
            }
            println!("[vitro] power: {}", if low_power { "电池 → 动效降级" } else { "交流 → 恢复" });
            render_frame(runtime);
        }
    }
}
