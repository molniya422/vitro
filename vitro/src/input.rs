// input.rs — 全局输入(NeXuS R6b 的反面:零轮询)
// WH_MOUSE_LL → 邻域门控(±200px 或底边召唤区)→ 8ms 节流 PostMessage;光标远离 = 零消息零渲染
// R9 自动隐藏:离开邻域 700ms(TIMER_HIDE)→ 滑出;底边 3px → 召唤。全部事件驱动,无一处轮询。
use windows::Win32::Foundation::*;
use windows::Win32::System::SystemInformation::GetTickCount;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::dockwin::{render_frame, save_state};
use crate::state::*;

pub(crate) unsafe extern "system" fn ll_hook(_code: i32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if wp.0 as u32 == WM_MOUSEMOVE {
        let m = &*(lp.0 as *const MSLLHOOKSTRUCT);
        let (mx, my) = (m.pt.x as f32, m.pt.y as f32);
        let st = s();
        let r = st.pill_rect_screen();
        let bottom_edge = st.autohide && my >= st.screen_h as f32 - 3.0;
        let near_dock = mx >= (r.left - 200) as f32 && mx <= (r.right + 200) as f32
                && my >= (r.top - 200) as f32 && my <= (r.bottom + 200) as f32;
        let near = near_dock || bottom_edge;
        let now = GetTickCount();
        if near && st.slide_target != 0.0 {
            st.slide_target = 0.0; // 召唤
            SetTimer(st.hwnd, TIMER_ANIM as usize, 33, None);
        }
        if near && st.hide_armed {
            st.hide_armed = false;
            let _ = KillTimer(st.hwnd, TIMER_HIDE as usize);
        } else if !near && st.was_near_flag && st.autohide && st.slide_target == 0.0 {
            st.hide_armed = true;
            SetTimer(st.hwnd, TIMER_HIDE as usize, 700, None);
        }
        if (near || st.was_near_flag) && now.wrapping_sub(st.last_post_tick) >= 8 {
            st.last_post_tick = now;
            st.cursor = (mx, my);
            let _ = PostMessageW(st.hwnd, WM_APP_MOUSE, WPARAM(mx as usize), LPARAM(my as isize));
        } else if near {
            st.cursor = (mx, my);
        } else if st.was_near_flag {
            st.cursor = (mx, my); // 松手补发,让弹簧回落
            let _ = PostMessageW(st.hwnd, WM_APP_MOUSE, WPARAM(mx as usize), LPARAM(my as isize));
        }
        st.was_near_flag = near;
    }
    CallNextHookEx(None, 0, wp, lp)
}

// 离开 700ms → 滑出屏外
pub(crate) unsafe fn on_hide_timer() {
    let st = s();
    st.hide_armed = false;
    let _ = KillTimer(st.hwnd, TIMER_HIDE as usize);
    if st.autohide && st.slide_target == 0.0 && st.slide < 0.5 {
        st.slide_target = 1.0;
        st.hover_idx = None;
        crate::dockwin::hide_tip();
        SetTimer(st.hwnd, TIMER_ANIM as usize, 33, None);
        render_frame();
    }
}

// WM_POWERBROADCAST(PBT_APMPOWERSTATUSCHANGE)→ 消息驱动,零轮询
pub(crate) unsafe fn update_power() {
    use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
    let mut stt = SYSTEM_POWER_STATUS::default();
    if GetSystemPowerStatus(&mut stt).is_ok() {
        let st = s();
        let was = st.low_power;
        st.low_power = stt.ACLineStatus == 0;
        if was != st.low_power {
            if st.low_power { let _ = KillTimer(st.hwnd, TIMER_ANIM as usize); st.anim_on = false; }
            println!("[vitro] power: {}", if st.low_power { "电池 → 动效降级" } else { "交流 → 恢复" });
            render_frame();
        }
    }
}

// 托盘/菜单切换自动隐藏时持久化
pub(crate) unsafe fn persist_autohide(autohide: bool) {
    s().autohide = autohide;
    crate::dockwin::save_state();
}
