// dockwin.rs — dock 窗口:过程 + 渲染 + 交互
// 视觉规格 = vitro-nexus-full.html / macos-dock-spec.md:
//   玻璃药丸(DWM 亚克力 + 1px 描边 + 顶缘内高光 + 底缘内影)
//   falloff 弹簧放大(peak/radius 来自主题)· 运行呼吸点(频率=进程 CPU)
//   倒影(PushLayer 渐变遮罩镜像)· 插件磁贴 = SQ 图标 + 底部数值药丸
// 交互 = NeXuS R2-R7:悬停 tooltip · 点击启动 · 右键菜单 · 拖放加图标
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Foundation::Numerics::Matrix3x2;
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::DirectWrite::*;
use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::System::SystemInformation::GetTickCount;
use windows::Win32::System::Threading::{GetProcessTimes, WaitForSingleObject};
use windows::Win32::Graphics::Gdi::{DT_CENTER, DT_VCENTER, DT_SINGLELINE, PAINTSTRUCT, BeginPaint, EndPaint, DrawTextW, InvalidateRect};
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Graphics::Gdi::{SetBkMode, SetTextColor, TRANSPARENT};

use crate::config::{self, DockConfig, DockItem};
use crate::core::event::{BehaviorEvent, Effect, Timer};
use crate::core::model::TileId;
use crate::icons::{extract_icon_96, sq_guess};
use crate::input;
use crate::lpad;
use crate::runtime::{self, AppRuntime};
use crate::state::*;

// D2D 裸指针参数统一封装(0.58 全是 Option<*const T>)
pub(crate) fn some_ptr<T>(t: &T) -> *const T { t }

// ── 状态保存(NeXuS R11)──
pub(crate) fn save_state(state: &State, autohide: bool) {
    let items: Vec<DockItem> = state.tiles.iter()
        .filter(|t| !t.is_plugin && !t.path.is_empty())
        .map(|t| DockItem { name: t.name.clone(), path: t.path.clone() }).collect();
    let cfg = DockConfig { items, plugins: state.plugins.clone(), autohide };
    config::save_config(&cfg);
}

// ── 窗口过程 ──
pub(crate) unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    let Some(runtime) = runtime::from_window(hwnd) else {
        return DefWindowProcW(hwnd, msg, wp, lp);
    };
    let runtime = runtime.as_ptr();
    match msg {
        WM_APP_MOUSE => { on_mouse(runtime, wp.0 as i32, lp.0 as i32); LRESULT(0) }
        WM_TIMER => {
            match wp.0 as u32 {
                TIMER_DWELL => on_dwell(runtime),
                TIMER_ANIM => render_frame(runtime),
                TIMER_HIDE => on_hide_timer(runtime),
                _ => {}
            }
            LRESULT(0)
        }
        WM_LBUTTONDOWN => { begin_tile_pointer(runtime, hwnd, mouse_xy(lp)); LRESULT(0) }
        WM_MOUSEMOVE => { update_tile_drag(runtime, mouse_xy(lp)); LRESULT(0) }
        WM_LBUTTONUP => { finish_tile_pointer(runtime, hwnd, mouse_xy(lp)); LRESULT(0) }
        WM_CAPTURECHANGED => { (*runtime).dock_drag = None; LRESULT(0) }
        WM_RBUTTONDOWN => { let _ = on_menu(runtime, lp.0 as i32 & 0xFFFF, (lp.0 as i32 >> 16) & 0xFFFF); LRESULT(0) }
        WM_KEYDOWN | WM_CHAR => { if wp.0 as u16 == 27 { shutdown(runtime, hwnd); } LRESULT(0) }
        WM_CLOSE => { shutdown(runtime, hwnd); LRESULT(0) }
        WM_POWERBROADCAST => { if wp.0 as u32 == 10 { input::update_power(runtime); } LRESULT(0) }
        WM_HOTKEY => {
            if wp.0 as i32 == 2 { let _ = reload_theme(runtime); } else { lpad::toggle(runtime); }
            LRESULT(0)
        }
        WM_DROPFILES => { on_drop(runtime, HDROP(wp.0 as *mut std::ffi::c_void)); LRESULT(0) }
        WM_APP_TRAY => {
            let ev = (lp.0 as u32) & 0xFFFF;
            if ev == WM_RBUTTONUP || ev == WM_LBUTTONUP { let _ = crate::tray::on_tray_menu(runtime); }
            LRESULT(0)
        }
        WM_NCHITTEST => {
            let sx = (lp.0 as i32) as i16 as i32;
            let sy = ((lp.0 as i32) >> 16) as i16 as i32;
            let state = &(*runtime).state;
            let pill = state.pill_rect_screen();
            let in_pill = sx >= pill.left && sx <= pill.right && sy >= pill.top && sy <= pill.bottom;
            let active = state.hit_test(sx, sy).is_some()
                || state.subdock_hit_test(sx, sy).is_some()
                || in_pill;
            if active { LRESULT(HTCLIENT as isize) } else { LRESULT(HTTRANSPARENT as isize) }
        }
        WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
        // shutdown() owns cleanup and destroys the HWND. WM_DESTROY is only the
        // terminal message; calling shutdown here would recursively destroy it.
        WM_DESTROY => { PostQuitMessage(0); LRESULT(0) }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

pub(crate) unsafe extern "system" fn tip_wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if msg == WM_PAINT {
        let mut ps = PAINTSTRUCT::default();
        let hdc = BeginPaint(hwnd, &mut ps);
        if let Some(runtime) = runtime::from_window(hwnd) {
            let state = &runtime.as_ref().state;
            if let Some(id) = runtime.as_ref().behavior.tooltip {
                let label = runtime.as_ref().subdock_hover
                    .filter(|(parent, _)| *parent == id)
                    .and_then(|(_, index)| state.subdock.as_ref()
                        .and_then(|sub| sub.items.get(index)).map(|item| item.name.as_str()))
                    .or_else(|| state.tiles.iter().find(|tile| tile.id == id).map(|tile| tile.name.as_str()));
                if let Some(label) = label {
                    let mut name: Vec<u16> = label.encode_utf16().chain(std::iter::once(0)).collect();
                    let mut rc = RECT { left: 0, top: 0, right: 160, bottom: 34 };
                    let _ = SetBkMode(hdc, TRANSPARENT);
                    let _ = SetTextColor(hdc, COLORREF(0x00F4F8FF));
                    let _ = DrawTextW(hdc, &mut name, &mut rc, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
                }
            }
        }
        let _ = EndPaint(hwnd, &ps);
        return LRESULT(0);
    }
    DefWindowProcW(hwnd, msg, wp, lp)
}

// ── 悬停 / tooltip(NeXuS R3:200ms dwell)──
unsafe fn on_mouse(runtime: *mut AppRuntime, x: i32, y: i32) {
    let now = GetTickCount();
    let (hit, folder_index, inside_subdock, sub_hit, sub_target) = {
        let state = &(*runtime).state;
        let main_hit = state.hit_test(x, y);
        let folder_index = main_hit.filter(|index| state.tiles[*index].is_folder);
        let sub_hit = state.subdock_hit_test(x, y);
        let inside_subdock = state.subdock_contains(x, y);
        let sub_target = if inside_subdock {
            state.subdock.as_ref().and_then(|sub| sub_hit.map(|index| (sub.parent, index)))
        } else {
            None
        };
        let hit = if inside_subdock {
            sub_target.map(|(parent, _)| parent)
        } else {
            main_hit.map(|index| state.tiles[index].id)
        };
        (hit, folder_index, inside_subdock, sub_hit, sub_target)
    };
    if let Some(sub) = (*runtime).state.subdock.as_mut() { sub.hovered = sub_hit; }
    let sub_hover_changed = (*runtime).subdock_hover != sub_target;
    (*runtime).subdock_hover = sub_target;
    let drag_active = (*runtime).dock_drag.as_ref().is_some_and(|drag| drag.moved);
    if !drag_active {
        if let Some(index) = folder_index {
            open_subdock(runtime, index);
        } else if !inside_subdock {
            close_subdock(runtime);
        } else if sub_target.is_some() && (*runtime).state.subdock.as_ref().is_some_and(|sub| sub.closing) {
            if let Some(sub) = (*runtime).state.subdock.as_mut() {
                sub.closing = false;
            }
            render_frame(runtime);
        }
    }
    if sub_hover_changed {
        let had_tooltip = (*runtime).behavior.tooltip.take().is_some();
        if had_tooltip { hide_tip_window(runtime); }
        (*runtime).behavior.hovered = hit;
        (*runtime).behavior.hover_since = now;
        if hit.is_some() {
            let _ = SetTimer((*runtime).state.hwnd, TIMER_DWELL as usize, 120, None);
        } else {
            let _ = KillTimer((*runtime).state.hwnd, TIMER_DWELL as usize);
        }
    }
    let state = &(*runtime).state;
    let snapshot = crate::core::input::PointerSnapshot {
        pill: {
            let rect = state.pill_rect_screen();
            crate::core::input::ScreenRect {
                left: rect.left,
                top: rect.top,
                right: rect.right,
                bottom: rect.bottom,
            }
        },
        screen_height: state.screen_h,
        autohide: (*runtime).behavior.autohide,
    };
    let near = crate::core::input::route_pointer(
        snapshot, crate::core::input::PointerRouteState::default(), x, y, now,
    ).1.near;
    apply_behavior_event(runtime, BehaviorEvent::PointerMoved { x, y, near, hit, now_ms: now });
}

unsafe fn open_subdock(runtime: *mut AppRuntime, index: usize) {
    let (parent, path, parent_x) = {
        let state = &(*runtime).state;
        let Some(tile) = state.tiles.get(index) else { return; };
        if !tile.is_folder { return; }
        let (x, _, _) = state.tile_center_local(index);
        (tile.id, tile.path.clone(), x)
    };
    if (*runtime).state.subdock.as_ref().is_some_and(|sub| sub.parent == parent) {
        if let Some(sub) = (*runtime).state.subdock.as_mut() {
            if sub.closing {
                sub.closing = false;
                render_frame(runtime);
            }
        }
        return;
    }

    let mut entries: Vec<(String, String)> = std::fs::read_dir(&path)
        .ok()
        .into_iter()
        .flat_map(|rd| rd.flatten())
        .filter_map(|entry| {
            let p = entry.path();
            let name = p.file_name()?.to_string_lossy().into_owned();
            Some((name, p.to_string_lossy().into_owned()))
        })
        .collect();
    entries.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    entries.truncate(10);

    let state = &mut (*runtime).state;
    let mut items = Vec::with_capacity(entries.len());
    for (name, path) in entries {
        let bitmap = crate::icons::extract_icon_96(&state.gpu.dc, &path);
        let sq = if bitmap.is_none() { crate::icons::sq_guess(&name) } else { None };
        let fallback = if bitmap.is_none() && sq.is_none() {
            crate::dockwin::make_text_bitmap(
                &state.gpu.dc,
                &state.white,
                &state.plugin_fmt,
                name.chars().next().map(|c| c.to_string()).unwrap_or_default().as_str(),
                20.0,
            )
        } else { None };
        let color_idx = name.bytes().map(|b| b as usize).sum::<usize>() % 8;
        items.push(SubdockItem { name, path, bitmap, fallback, sq, color_idx });
    }
    if items.is_empty() {
        close_subdock(runtime);
        return;
    }
    let width = crate::state::pill_w_for(items.len(), state.win_w);
    let left = (parent_x - width as f32 / 2.0).clamp(8.0, (state.win_w - width - 8).max(8) as f32);
    state.subdock = Some(Subdock {
        parent,
        items,
        width,
        left,
        top: crate::core::geometry::subdock_top(PANEL_TOP, PANEL_H, 16.0),
        progress: 0.0,
        hovered: None,
        closing: false,
    });
    (*runtime).subdock_hover = None;
    render_frame(runtime);
}

unsafe fn close_subdock(runtime: *mut AppRuntime) {
    let has_subdock = (*runtime).state.subdock.is_some() || (*runtime).subdock_hover.is_some();
    if !has_subdock {
        return;
    }
    (*runtime).subdock_hover = None;
    (*runtime).behavior.hovered = None;
    (*runtime).behavior.hover_since = GetTickCount();
    let _ = KillTimer((*runtime).state.hwnd, TIMER_DWELL as usize);
    if (*runtime).behavior.tooltip.take().is_some() {
        hide_tip_window(runtime);
    }
    let should_render = if let Some(sub) = (*runtime).state.subdock.as_mut() {
        sub.hovered = None;
        if sub.progress <= 0.0 {
            false
        } else {
            sub.closing = true;
            true
        }
    } else {
        false
    };
    if should_render {
        render_frame(runtime);
    } else {
        (*runtime).state.subdock = None;
    }
}

unsafe fn on_dwell(runtime: *mut AppRuntime) {
    apply_behavior_event(runtime, BehaviorEvent::DwellElapsed { now_ms: GetTickCount() });
}

unsafe fn show_tip(runtime: *mut AppRuntime, id: TileId) {
    let (tip_hwnd, x, y) = {
        let state = &(*runtime).state;
        if let Some((parent, index)) = (*runtime).subdock_hover.filter(|(parent, _)| *parent == id) {
            let Some(sub) = state.subdock.as_ref().filter(|sub| sub.parent == parent) else { return; };
            let size = tile_size(sub.width, sub.items.len());
            let cx = sub.left + PAD + size / 2.0 + index as f32 * (size + GAP);
            (state.tip_hwnd, state.win_x + (cx - 80.0) as i32, state.win_y + sub.top as i32 - 38)
        } else {
            let Some(idx) = state.tiles.iter().position(|tile| tile.id == id) else { return; };
            let (cx, _, _) = state.tile_center_local(idx);
            (state.tip_hwnd, state.win_x + (state.pill_x() + cx - 80.0) as i32, state.win_y + PANEL_TOP as i32 - 38)
        }
    };
    let _ = SetWindowPos(tip_hwnd, HWND_TOPMOST, x, y, 0, 0, SWP_NOSIZE | SWP_SHOWWINDOW);
    let _ = InvalidateRect(tip_hwnd, None, TRUE);
}

unsafe fn hide_tip_window(runtime: *mut AppRuntime) {
    let tip_hwnd = (*runtime).state.tip_hwnd;
    let _ = ShowWindow(tip_hwnd, SW_HIDE);
}

unsafe fn apply_behavior_event(runtime: *mut AppRuntime, event: BehaviorEvent) {
    let effects = {
        let runtime = &mut *runtime;
        crate::core::reducer::reduce(&mut runtime.behavior, event)
    };
    execute_behavior_effects(runtime, effects);
}

unsafe fn execute_behavior_effects(runtime: *mut AppRuntime, effects: Vec<Effect>) {
    for effect in effects {
        match effect {
            Effect::SetTimer { timer, interval_ms } => {
                let id = match timer {
                    Timer::TooltipDwell => TIMER_DWELL,
                    Timer::AutoHide => TIMER_HIDE,
                    Timer::Animation => TIMER_ANIM,
                };
                let _ = SetTimer((*runtime).state.hwnd, id as usize, interval_ms, None);
            }
            Effect::KillTimer(timer) => {
                let id = match timer {
                    Timer::TooltipDwell => TIMER_DWELL,
                    Timer::AutoHide => TIMER_HIDE,
                    Timer::Animation => TIMER_ANIM,
                };
                let _ = KillTimer((*runtime).state.hwnd, id as usize);
            }
            Effect::ShowTooltip(id) => show_tip(runtime, id),
            Effect::HideTooltip => hide_tip_window(runtime),
            Effect::SaveAutohide(autohide) => save_state(&(*runtime).state, autohide),
            Effect::PublishHookSnapshot => {
                input::publish_hook_snapshot(&(*runtime).state, (*runtime).behavior.autohide);
            }
            Effect::Render => render_frame(runtime),
        }
    }
}

// ── 启动(NeXuS R4/R5:ShellExecute + 句柄跟踪)──
fn mouse_xy(lp: LPARAM) -> (i32, i32) {
    let packed = lp.0 as u32;
    (packed as u16 as i16 as i32, (packed >> 16) as u16 as i16 as i32)
}

unsafe fn begin_tile_pointer(runtime: *mut AppRuntime, hwnd: HWND, (x, y): (i32, i32)) {
    let drag = {
        let state = &(*runtime).state;
        state.hit_test(state.win_x + x, state.win_y + y)
            .and_then(|index| state.tiles.get(index).map(|tile| tile.id))
            .map(|tile_id| crate::runtime::DockDrag { tile_id, start: (x, y), moved: false, target: None })
    };
    if drag.is_some() {
        (*runtime).dock_drag = drag;
        let _ = SetCapture(hwnd);
    }
}

unsafe fn update_tile_drag(runtime: *mut AppRuntime, (x, y): (i32, i32)) {
    let tile_id = {
        let Some(drag) = (*runtime).dock_drag.as_ref() else { return; };
        if (x - drag.start.0).abs() < 5 && (y - drag.start.1).abs() < 5 { return; }
        drag.tile_id
    };
    let target = {
        let state = &(*runtime).state;
        state.hit_test(state.win_x + x, state.win_y + y)
            .and_then(|index| state.tiles.get(index).map(|tile| tile.id))
            .filter(|id| *id != tile_id)
    };
    let moved = if let Some(drag) = (*runtime).dock_drag.as_mut() {
        drag.moved = true;
        drag.target = target;
        true
    } else { false };
    if moved { render_frame(runtime); }
}

unsafe fn finish_tile_pointer(runtime: *mut AppRuntime, hwnd: HWND, (x, y): (i32, i32)) {
    let drag = (*runtime).dock_drag.take();
    if drag.is_some() && GetCapture() == hwnd { let _ = ReleaseCapture(); }
    let sub_path = {
        let state = &(*runtime).state;
        if drag.as_ref().map_or(true, |drag| !drag.moved) {
            state.subdock_hit_test(state.win_x + x, state.win_y + y)
                .and_then(|index| state.subdock.as_ref()?.items.get(index).map(|item| item.path.clone()))
        } else {
            None
        }
    };
    if let Some(path) = sub_path {
        launch_subdock_path(runtime, &path);
        return;
    }
    let Some(drag) = drag else { return; };
    let (from, to, moved) = {
        let state = &(*runtime).state;
        let from = crate::core::model::tile_index_by_id(state.tiles.iter().map(|tile| tile.id), drag.tile_id);
        let to = state.hit_test(state.win_x + x, state.win_y + y);
        (from, to, drag.moved)
    };
    if moved {
        if let (Some(from), Some(to)) = (from, to) {
            if (*runtime).state.move_tile(from, to) {
                save_state(&(*runtime).state, (*runtime).behavior.autohide);
                input::publish_hook_snapshot(&(*runtime).state, (*runtime).behavior.autohide);
                render_frame(runtime);
            }
        }
    } else if let Some(index) = from {
        if (&(*runtime).state.tiles).get(index).is_some_and(|tile| tile.is_folder) {
            open_subdock(runtime, index);
        } else {
            launch(runtime, index);
        }
    }
    if GetCapture() == hwnd { let _ = ReleaseCapture(); }
}

unsafe fn launch(runtime: *mut AppRuntime, index: usize) {
    let path = {
        let state = &(*runtime).state;
        let Some(tile) = state.tiles.get(index) else { return; };
        if tile.is_plugin || tile.is_folder || tile.path.is_empty() { return; }
        tile.path.clone()
    };
    launch_path(runtime, index, &path);
}

unsafe fn launch_path(runtime: *mut AppRuntime, index: usize, path: &str) {
    let wpath: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
    let mut sei = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS,
        lpFile: PCWSTR(wpath.as_ptr()),
        lpVerb: w!("open"),
        nShow: SW_SHOWNORMAL.0,
        ..Default::default()
    };
    if ShellExecuteExW(&mut sei).is_ok() && !sei.hProcess.is_invalid() {
        let state = &mut (*runtime).state;
        if let Some(handle) = state.running.get_mut(index) { *handle = Some(sei.hProcess); }
    }
    render_frame(runtime);
}

unsafe fn launch_subdock_path(runtime: *mut AppRuntime, path: &str) {
    let wpath: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
    let mut sei = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS,
        lpFile: PCWSTR(wpath.as_ptr()),
        lpVerb: w!("open"),
        nShow: SW_SHOWNORMAL.0,
        ..Default::default()
    };
    if ShellExecuteExW(&mut sei).is_ok() && !sei.hProcess.is_invalid() {
        let _ = CloseHandle(sei.hProcess);
    }
    close_subdock(runtime);
}

// ── 右键菜单(NeXuS R7)──
unsafe fn on_menu(runtime: *mut AppRuntime, x: i32, y: i32) -> Result<()> {
    // TrackPopupMenu dispatches messages recursively. Keep only copied values
    // alive across that loop, then reacquire runtime state for the command.
    let (hwnd, sx, sy, on_icon) = {
        let state = &(*runtime).state;
        let sx = state.win_x + x;
        let sy = state.win_y + y;
        let tile_id = state
            .hit_test(sx, sy)
            .and_then(|index| state.tiles.get(index).map(|tile| tile.id));
        (state.hwnd, sx, sy, tile_id)
    };
    let menu = CreatePopupMenu()?;
    if on_icon.is_some() {
        let _ = AppendMenuW(menu, MF_STRING, 1, w!("打开"));
        let _ = AppendMenuW(menu, MF_STRING, 3, w!("从 Dock 移除"));
        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, None);
    }
    let _ = AppendMenuW(menu, MF_STRING, 4, w!("启动器…  Ctrl+Alt+D"));
    let _ = AppendMenuW(menu, MF_STRING, 5, w!("重载主题  Ctrl+Alt+R"));
    let _ = AppendMenuW(menu, MF_SEPARATOR, 0, None);
    let _ = AppendMenuW(menu, MF_STRING, 2, w!("退出 Vitro"));
    let _ = SetForegroundWindow(hwnd);
    let cmd = TrackPopupMenu(menu, TPM_RIGHTBUTTON | TPM_RETURNCMD, sx, sy, 0, hwnd, None);
    let _ = DestroyMenu(menu);
    if (*runtime).is_shutting_down() { return Ok(()); }

    match cmd.0 {
        1 => if let Some(tile_id) = on_icon {
            let index = crate::core::model::tile_index_by_id(
                (*runtime).state.tiles.iter().map(|tile| tile.id),
                tile_id,
            );
            if let Some(index) = index { launch(runtime, index); }
        },
        3 => if let Some(tile_id) = on_icon {
            let removed_id = {
                let state = &mut (*runtime).state;
                let index = crate::core::model::tile_index_by_id(
                    state.tiles.iter().map(|tile| tile.id),
                    tile_id,
                );
                if let Some(index) = index {
                    let removed = state.tiles.remove(index);
                    if index < state.running.len() {
                        if let Some(handle) = state.running.remove(index) { let _ = CloseHandle(handle); }
                    }
                    state.relayout();
                    Some(removed.id)
                } else {
                    None
                }
            };
            if let Some(id) = removed_id {
                if (*runtime).state.subdock.as_ref().is_some_and(|sub| sub.parent == id) {
                    close_subdock(runtime);
                }
                let effects = crate::core::reducer::reduce(
                    &mut (*runtime).behavior,
                    BehaviorEvent::TileRemoved(id),
                );
                execute_behavior_effects(runtime, effects);
                save_state(&(*runtime).state, (*runtime).behavior.autohide);
                input::publish_hook_snapshot(&(*runtime).state, (*runtime).behavior.autohide);
                render_frame(runtime);
            }
        },
        4 => lpad::toggle(runtime),
        5 => reload_theme(runtime)?,
        2 => shutdown(runtime, hwnd),
        _ => {}
    }
    Ok(())
}

// ── 拖放加图标(NeXuS R6)──
unsafe fn on_drop(runtime: *mut AppRuntime, h: HDROP) {
    {
        let runtime = &mut *runtime;
        let state = &mut runtime.state;
        let count = DragQueryFileW(h, u32::MAX, None);
        for i in 0..count {
            let mut buf = [0u16; 1024];
            let len = DragQueryFileW(h, i, Some(&mut buf)) as usize;
            if len == 0 { continue; }
            let path = String::from_utf16_lossy(&buf[..len]);
            let name = std::path::Path::new(&path).file_stem()
                .map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "项目".into());
            if state.tiles.iter().any(|t| t.path.eq_ignore_ascii_case(&path)) { continue; }
            let bitmap = extract_icon_96(&state.gpu.dc, &path);
            let sq = if bitmap.is_none() { sq_guess(&name) } else { None };
            let fallback = if bitmap.is_none() && sq.is_none() {
                make_text_bitmap(&state.gpu.dc, &state.white, &state.plugin_fmt,
                    name.chars().next().map(|c| c.to_string()).unwrap_or_default().as_str(), 24.0)
            } else { None };
            let color_idx = name.bytes().map(|b| b as usize).sum::<usize>() % 8;
            let id = runtime.behavior.allocate_tile_id();
            let is_folder = std::path::Path::new(&path).is_dir();
            state.tiles.push(Tile { id, name, path, bitmap, fallback, is_plugin: false, is_folder, color_idx, sq, label: None });
        }
        state.relayout();
    }
    DragFinish(h);
    save_state(&(*runtime).state, (*runtime).behavior.autohide);
    input::publish_hook_snapshot(&(*runtime).state, (*runtime).behavior.autohide);
    render_frame(runtime);
}

// ── 主题热重载(Ctrl+Alt+R)──
pub(crate) unsafe fn reload_theme(runtime: *mut AppRuntime) -> Result<()> {
    let theme = config::load_theme();
    {
        let state = &mut (*runtime).state;
        state.theme = theme.clone();
        rebuild_theme_brushes(state, &theme)?;
    }
    render_frame(runtime);
    println!("[vitro] theme reloaded");
    Ok(())
}

pub(crate) unsafe fn rebuild_theme_brushes(st: &mut State, t: &config::Theme) -> Result<()> {
    let rt = &st.gpu.rt;

    st.glass = rt.CreateSolidColorBrush(some_ptr(&D2D1_COLOR_F { r: t.tint[0], g: t.tint[1], b: t.tint[2], a: t.alpha }), None)?;
    st.accent = rt.CreateSolidColorBrush(some_ptr(&D2D1_COLOR_F { r: t.accent[0], g: t.accent[1], b: t.accent[2], a: 1.0 }), None)?;
    st.halo = rt.CreateSolidColorBrush(some_ptr(&D2D1_COLOR_F { r: t.accent[0], g: t.accent[1], b: t.accent[2], a: 0.30 }), None)?;
    Ok(())
}

// Sonoma 四层材质·第三层:颗粒纹理(单色噪点,alpha 0-6/255 ≈ 0.3% 均匀噪点)
// 以 bitmap-brush WRAP 平铺,FillRoundedRectangle 天然被圆角矩形路径裁剪,零图层成本
pub(crate) unsafe fn build_noise(rt: &ID2D1DeviceContext) -> Result<ID2D1Brush> {
    const N: u32 = 128;
    let mut px = vec![0u8; (N * N * 4) as usize];
    let mut seed: u32 = 0x20261024; // 固定种子:每次启动纹理一致
    for i in 0..(N * N) as usize {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let a = ((seed >> 24) % 7) as u8; // 0..6
        // D2D_BITMAP_ALPHA_MODE_PREMULTIPLIED requires each color channel <= alpha.
        px[i * 4] = a; px[i * 4 + 1] = a; px[i * 4 + 2] = a; px[i * 4 + 3] = a;
    }
    let bmp = rt.CreateBitmap(D2D_SIZE_U { width: N, height: N },
        Some(px.as_ptr() as *const std::ffi::c_void), N * 4,
        &D2D1_BITMAP_PROPERTIES1 {
            pixelFormat: D2D1_PIXEL_FORMAT { format: DXGI_FORMAT_B8G8R8A8_UNORM, alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED },
            dpiX: 96.0, dpiY: 96.0,
            bitmapOptions: D2D1_BITMAP_OPTIONS(0),
            colorContext: core::mem::ManuallyDrop::new(None),
        })?;
    let brush = rt.CreateBitmapBrush(&bmp, Some(&D2D1_BITMAP_BRUSH_PROPERTIES1 {
        extendModeX: D2D1_EXTEND_MODE_WRAP, extendModeY: D2D1_EXTEND_MODE_WRAP,
        interpolationMode: D2D1_INTERPOLATION_MODE_NEAREST_NEIGHBOR,
    }), None)?;
    let brush: ID2D1Brush = brush.into(); // 升型到基类,渲染侧直接当画刷用
    Ok(brush)
}

pub(crate) unsafe fn show_dock(runtime: *mut AppRuntime) {
    apply_behavior_event(runtime, BehaviorEvent::ShowDock);
}

pub(crate) unsafe fn toggle_autohide(runtime: *mut AppRuntime) {
    apply_behavior_event(runtime, BehaviorEvent::ToggleAutohide);
}

unsafe fn on_hide_timer(runtime: *mut AppRuntime) {
    let slide_progress = (*runtime).state.slide;
    apply_behavior_event(runtime, BehaviorEvent::HideElapsed { slide_progress });
}

pub(crate) unsafe fn shutdown(runtime: *mut AppRuntime, hwnd: HWND) {
    // Set the idempotence gate and copy resources first. DestroyWindow sends
    // WM_DESTROY synchronously, so no runtime borrow may survive the call.
    let (dock, tip, hook, handles, config) = {
        let runtime = &mut *runtime;
        if !runtime.begin_shutdown() { return; }
        let state = &mut runtime.state;
        let items = state.tiles.iter()
            .filter(|tile| !tile.is_plugin && !tile.path.is_empty())
            .map(|tile| DockItem { name: tile.name.clone(), path: tile.path.clone() })
            .collect();
        let config = DockConfig {
            items,
            plugins: state.plugins.clone(),
            autohide: runtime.behavior.autohide,
        };
        let handles = state.running.iter_mut().filter_map(Option::take).collect::<Vec<_>>();
        let hook = std::mem::replace(&mut state.hook, HHOOK::default());
        (state.hwnd, state.tip_hwnd, hook, handles, config)
    };

    config::save_config(&config);
    lpad::shutdown(runtime);
    input::clear_hook_snapshot();
    crate::tray::remove(dock);
    let _ = UnregisterHotKey(dock, 1);
    let _ = UnregisterHotKey(dock, 2);
    let _ = DragAcceptFiles(dock, FALSE);
    let _ = KillTimer(dock, TIMER_DWELL as usize);
    let _ = KillTimer(dock, TIMER_ANIM as usize);
    let _ = KillTimer(dock, TIMER_HIDE as usize);
    if !hook.is_invalid() { let _ = UnhookWindowsHookEx(hook); }
    for handle in handles { let _ = CloseHandle(handle); }
    let _ = ShowWindow(tip, SW_HIDE);
    let _ = DestroyWindow(tip);
    let _ = DestroyWindow(hwnd);
}

// ── 渲染 ──
pub(crate) unsafe fn render_frame(runtime: *mut AppRuntime) {
    let behavior = &(*runtime).behavior;
    let dragged_id = (*runtime).dock_drag.as_ref().filter(|drag| drag.moved).map(|drag| drag.tile_id);
    let drop_target = (*runtime).dock_drag.as_ref().filter(|drag| drag.moved).and_then(|drag| drag.target);
    let st = &mut (*runtime).state;
    let rt = &st.gpu.rt;
    
    let (cx, cy) = behavior.pointer; // 屏幕坐标
    let theme_peak = st.theme.peak;
    let theme_radius = st.theme.radius;
    let now = GetTickCount();
    let n = st.tiles.len();
    let px = st.pill_x();
    let pw = st.pill_w as f32;
    let backdrop_ok = st.backdrop_ok;
    let pill_radius = PANEL_H * 0.5;
    let panel_top = PANEL_TOP;
    let panel_bottom = panel_top + PANEL_H;
    let baseline = panel_bottom - BASE_UP;

    // ── R9 滑出/召唤(窗口位移动画,33ms)──
    if st.slide != behavior.slide_target {
        let dt = now.wrapping_sub(st.last_anim_tick).max(1) as f32;
        let k = (dt / 1000.0 * 10.0).min(1.0);
        st.slide += (behavior.slide_target - st.slide) * k;
        if (st.slide - behavior.slide_target).abs() < 0.01 { st.slide = behavior.slide_target; }
        let y = st.base_y + (st.slide * (WIN_H as f32 + 14.0)) as i32;
        let _ = SetWindowPos(st.hwnd, HWND_TOPMOST, st.win_x, y, 0, 0, SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE);
        st.win_y = y;
    }

    let dt_ms = now.wrapping_sub(st.last_anim_tick).max(1);
    let anim = (st.cursor_inside_pill((cx, cy)) && !st.low_power) || st.anim_on;
    if anim || st.slide != behavior.slide_target { st.last_anim_tick = now; }

    rt.BeginDraw();
    rt.Clear(Some(some_ptr(&D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 0.0 })));

    // 玻璃药丸:系统亚克力生效 → 淡 tint(模糊由 DWM 供给);否则浓 tint 回退
    let ga = if backdrop_ok { st.theme.alpha * 0.42 } else { st.theme.alpha };
    st.glass.SetColor(&D2D1_COLOR_F { r: st.theme.tint[0], g: st.theme.tint[1], b: st.theme.tint[2], a: ga });
    let _ = rt.FillRoundedRectangle(some_ptr(&D2D1_ROUNDED_RECT {
        rect: D2D_RECT_F { left: px, top: panel_top, right: px + pw, bottom: panel_bottom },
        radiusX: pill_radius, radiusY: pill_radius,
    }), &st.glass);
    // 颗粒纹理层(Sonoma 第三层;WRAP 平铺,路径即裁剪)
    if let Some(nb) = &st.noise {
        let _ = rt.FillRoundedRectangle(some_ptr(&D2D1_ROUNDED_RECT {
            rect: D2D_RECT_F { left: px, top: panel_top, right: px + pw, bottom: panel_bottom },
            radiusX: pill_radius, radiusY: pill_radius,
        }), nb);
    }
    // Tahoe 0.5px 内折射边(高光内侧再走一圈极淡白,造玻璃厚度感)
    let _ = rt.DrawRoundedRectangle(some_ptr(&D2D1_ROUNDED_RECT {
        rect: D2D_RECT_F { left: px + 1.5, top: panel_top + 1.5, right: px + pw - 1.5, bottom: panel_bottom - 1.5 },
        radiusX: pill_radius - 1.5, radiusY: pill_radius - 1.5,
    }), &st.soft, 0.5, None);
    let _ = rt.DrawRoundedRectangle(some_ptr(&D2D1_ROUNDED_RECT {
        rect: D2D_RECT_F { left: px + 0.5, top: panel_top + 0.5, right: px + pw - 0.5, bottom: panel_bottom - 0.5 },
        radiusX: pill_radius - 0.5, radiusY: pill_radius - 0.5,
    }), &st.border, 1.0, None);
    let _ = rt.FillRoundedRectangle(some_ptr(&D2D1_ROUNDED_RECT {
        rect: D2D_RECT_F { left: px + 16.0, top: panel_top + 1.0, right: px + pw - 16.0, bottom: panel_top + 2.0 },
        radiusX: 0.5, radiusY: 0.5,
    }), &st.hi);
    let _ = rt.FillRoundedRectangle(some_ptr(&D2D1_ROUNDED_RECT {
        rect: D2D_RECT_F { left: px + 16.0, top: panel_bottom - 2.0, right: px + pw - 16.0, bottom: panel_bottom - 1.0 },
        radiusX: 0.5, radiusY: 0.5,
    }), &st.lo);

    // 分区线只出现在真实的语义边界(应用区|插件区)，避免用装饰线切碎空 Dock。
    let first_plugin = st.tiles.iter().position(|tile| tile.is_plugin);
    if let Some(first) = first_plugin {
        let boundaries = [Some(first), st.tiles.iter().rposition(|tile| tile.is_plugin).map(|last| last + 1)];
        for boundary in boundaries.into_iter().flatten().filter(|boundary| *boundary > 0 && *boundary < n) {
            if let Some(x) = crate::core::geometry::divider_x(st.pill_w, n, boundary, ICON, GAP, PAD) {
                let _ = rt.DrawLine(
                    D2D_POINT_2F { x: px + x, y: panel_top + 14.0 },
                    D2D_POINT_2F { x: px + x, y: panel_bottom - 14.0 },
                    &st.border,
                    1.0,
                    None,
                );
            }
        }
    }

    // ── 磁贴:falloff 弹簧 + 呼吸 + 图标/插件 ──
    let tile_sz = tile_size(st.pill_w, n);
    for i in 0..n {
        let (icx_l, icy_l, _) = st.tile_center_local(i);
        let icx_s = st.win_x as f32 + icx_l;
        let icy_s = st.win_y as f32 + icy_l;
        let target = crate::core::motion::magnification_target(
            (cx, cy), (icx_s, icy_s), theme_radius, theme_peak,
        );
        st.scale_anim[i] = crate::core::motion::spring_step(st.scale_anim[i], target, dt_ms, 14.0);
        let sc = st.scale_anim[i];
        let is_dragged = dragged_id == Some(st.tiles[i].id);
        let drag_scale = if is_dragged { 1.06 } else { 1.0 };
        let size = tile_sz * sc * drag_scale;
        let left = icx_l - size / 2.0;
        let top = crate::core::geometry::tile_top_for_scale(baseline, 0.0, size) - if is_dragged { 4.0 } else { 0.0 };
        let icon_cy = top + size / 2.0;

        if drop_target == Some(st.tiles[i].id) {
            let _ = rt.DrawRoundedRectangle(some_ptr(&D2D1_ROUNDED_RECT {
                rect: D2D_RECT_F { left: left - 4.0, top: top - 4.0, right: left + size + 4.0, bottom: top + size + 4.0 },
                radiusX: 15.0, radiusY: 15.0,
            }), &st.accent, 2.0, None);
        }

        // 运行呼吸(NeXuS R5 创新:频率 = 进程 CPU 占用)
        let is_run = st.running.get(i).map_or(false, |r| r.is_some());
        let mut wave = 0.0f32;
        if is_run {
            if let Some(h) = st.running[i] {
                let (mut c, mut e, mut k64, mut u) = (FILETIME::default(), FILETIME::default(), FILETIME::default(), FILETIME::default());
                if GetProcessTimes(h, &mut c, &mut e, &mut k64, &mut u).is_err() {
                    let _ = CloseHandle(h);
                    st.running[i] = None;
                } else if anim {
                    let cur = (((k64.dwHighDateTime as u64) << 32) | k64.dwLowDateTime as u64,
                               ((u.dwHighDateTime as u64) << 32) | u.dwLowDateTime as u64);
                    let (pk, pu) = st.cpu_prev[i];
                    let wall = dt_ms as f32 * 10000.0;
                    st.cpu_pct[i] = ((cur.0.wrapping_sub(pk) + cur.1.wrapping_sub(pu)) as f32 / wall * 100.0).clamp(0.0, 100.0);
                    st.cpu_prev[i] = cur;
                    let period = 3000.0 - (st.cpu_pct[i] / 100.0) * 2300.0;
                    wave = ((now as f32 / period * std::f32::consts::TAU).sin() * 0.5 + 0.5)
                        * (0.35 + 0.65 * st.cpu_pct[i] / 100.0).max(0.25);
                }
            }
        }
        if is_run && !st.low_power {
            let hr = size * (0.55 + 0.10 * wave);
            let _ = rt.FillEllipse(some_ptr(&D2D1_ELLIPSE {
                point: D2D_POINT_2F { x: icx_l, y: icon_cy },
                radiusX: hr, radiusY: hr,
            }), &st.halo);
        }

        draw_tile_body(st, i, left, top, size, sc);

        // 运行指示点
        if is_run {
            let dot_r = 2.2 + wave * 1.8;
            let _ = rt.FillEllipse(some_ptr(&D2D1_ELLIPSE {
                point: D2D_POINT_2F {
                    x: icx_l,
                    y: crate::core::geometry::running_indicator_y(baseline, 0.0, tile_sz, 6.0),
                },
                radiusX: dot_r, radiusY: dot_r,
            }), &st.accent);
        }
    }

    // ── 倒影:PushLayer 渐变遮罩 + 镜像 ──
    st.grad_lin.SetStartPoint(D2D_POINT_2F { x: 0.0, y: baseline });
    st.grad_lin.SetEndPoint(D2D_POINT_2F { x: 0.0, y: WIN_H as f32 });
    let params = D2D1_LAYER_PARAMETERS1 {
        contentBounds: D2D_RECT_F { left: px, top: baseline - 1.0, right: px + pw, bottom: WIN_H as f32 + 40.0 },
        geometricMask: std::mem::ManuallyDrop::new(None),
        maskAntialiasMode: D2D1_ANTIALIAS_MODE_PER_PRIMITIVE,
        maskTransform: Matrix3x2 { M31: 0.0, M32: 0.0, M11: 1.0, M12: 0.0, M21: 0.0, M22: 1.0 },
        opacity: 1.0,
        opacityBrush: std::mem::ManuallyDrop::new(Some(st.grad_refl.clone())),
        layerOptions: D2D1_LAYER_OPTIONS1(0),
    };
    st.gpu.dc.PushLayer(&params, &st.layer);
    rt.SetTransform(&Matrix3x2 { M31: 0.0, M32: 2.0 * baseline, M11: 1.0, M12: 0.0, M21: 0.0, M22: -1.0 });
    for i in 0..n {
        let (icx_l, _, _) = st.tile_center_local(i);
        let is_dragged = dragged_id == Some(st.tiles[i].id);
        let drag_scale = if is_dragged { 1.06 } else { 1.0 };
        let size = tile_sz * st.scale_anim[i] * drag_scale;
        let top = crate::core::geometry::tile_top_for_scale(baseline, 0.0, size) - if is_dragged { 4.0 } else { 0.0 };
        draw_tile_body(st, i, icx_l - size / 2.0, top, size, st.scale_anim[i] * drag_scale);
    }
    rt.SetTransform(&Matrix3x2 { M31: 0.0, M32: 0.0, M11: 1.0, M12: 0.0, M21: 0.0, M22: 1.0 });
    rt.PopLayer();

    let remove_subdock = if let Some(sub) = st.subdock.as_mut() {
        sub.progress = crate::core::motion::subdock_progress_step(
            sub.progress,
            sub.closing,
            dt_ms.min(50),
            180.0,
        );
        sub.closing && sub.progress <= 0.0
    } else {
        false
    };
    if remove_subdock {
        st.subdock = None;
    }
    if let Some(sub) = &st.subdock {
        let (origin_x, origin_y) = st.tiles.iter()
            .position(|tile| tile.id == sub.parent)
            .map(|index| {
                let (x, y, _) = st.tile_center_local(index);
                (x, y)
            })
            .unwrap_or((sub.left + sub.width as f32 / 2.0, PANEL_TOP + PANEL_H / 2.0));
        let p = sub.progress.max(0.001);
        rt.SetTransform(&Matrix3x2 {
            M31: origin_x * (1.0 - p),
            M32: origin_y * (1.0 - p),
            M11: p,
            M12: 0.0,
            M21: 0.0,
            M22: p,
        });
        draw_subdock(st, sub);
        rt.SetTransform(&Matrix3x2 { M31: 0.0, M32: 0.0, M11: 1.0, M12: 0.0, M21: 0.0, M22: 1.0 });
    }

    // ── 定时器管理:静止即撤(空闲 0%)──
    let targets: Vec<f32> = (0..n).map(|i| {
        let (x, y, _) = st.tile_center_local(i);
        crate::core::motion::magnification_target(
            (cx, cy), (st.win_x as f32 + x, st.win_y as f32 + y), theme_radius, theme_peak,
        )
    }).collect();
    let has_running = st.running.iter().take(n).any(Option::is_some);
    let should = crate::core::motion::needs_animation_timer(
        &st.scale_anim[..n], &targets, 0.004,
        st.slide != behavior.slide_target, has_running, st.low_power,
    ) || st.subdock.as_ref().is_some_and(|sub| sub.closing || sub.progress < 1.0);
    if should && !st.anim_on { st.anim_on = true; SetTimer(st.hwnd, TIMER_ANIM as usize, 33, None); }
    else if !should && st.anim_on { st.anim_on = false; let _ = KillTimer(st.hwnd, TIMER_ANIM as usize); }

    input::publish_hook_snapshot(st, behavior.autohide);

    let hr_end = rt.EndDraw(None, None);
    if hr_end.is_err() { println!("[vitro] EndDraw FAILED: {:?}", hr_end); }
    let pr = st.gpu.sc.Present(0, DXGI_PRESENT(0));
    if pr.is_err() { println!("[vitro] Present FAILED: {:?}", pr); }

    // 进程存活检查(挂在渲染时机)
    for i in 0..n {
        if let Some(h) = st.running[i] {
            if WaitForSingleObject(h, 0) == WAIT_OBJECT_0 {
                let _ = CloseHandle(h);
                st.running[i] = None;
            }
        }
    }
}

unsafe fn draw_subdock(st: &State, sub: &Subdock) {
    let rt = &st.gpu.rt;
    let radius = PANEL_H * 0.5;
    let right = sub.left + sub.width as f32;
    let bottom = sub.top + PANEL_H;
    let _ = rt.FillRoundedRectangle(some_ptr(&D2D1_ROUNDED_RECT {
        rect: D2D_RECT_F { left: sub.left, top: sub.top, right, bottom },
        radiusX: radius, radiusY: radius,
    }), &st.glass);
    if let Some(noise) = &st.noise {
        let _ = rt.FillRoundedRectangle(some_ptr(&D2D1_ROUNDED_RECT {
            rect: D2D_RECT_F { left: sub.left, top: sub.top, right, bottom },
            radiusX: radius, radiusY: radius,
        }), noise);
    }
    let _ = rt.DrawRoundedRectangle(some_ptr(&D2D1_ROUNDED_RECT {
        rect: D2D_RECT_F { left: sub.left + 0.5, top: sub.top + 0.5, right: right - 0.5, bottom: bottom - 0.5 },
        radiusX: radius - 0.5, radiusY: radius - 0.5,
    }), &st.border, 1.0, None);
    let size = tile_size(sub.width, sub.items.len());
    for (i, item) in sub.items.iter().enumerate() {
        let left = sub.left + PAD + i as f32 * (size + GAP);
        let top = sub.top + PANEL_H - BASE_UP - size;
        if sub.hovered == Some(i) {
            let _ = rt.FillRoundedRectangle(some_ptr(&D2D1_ROUNDED_RECT {
                rect: D2D_RECT_F { left: left - 4.0, top: top - 4.0, right: left + size + 4.0, bottom: top + size + 4.0 },
                radiusX: 15.0, radiusY: 15.0,
            }), &st.soft);
        }
        if let Some(bitmap) = &item.bitmap {
            let _ = rt.DrawBitmap(bitmap, Some(some_ptr(&D2D_RECT_F { left, top, right: left + size, bottom: top + size })),
                1.0, D2D1_BITMAP_INTERPOLATION_MODE_LINEAR, None);
        } else if let Some(key) = item.sq.and_then(|key| st.sq.get(key)) {
            let _ = rt.DrawBitmap(key, Some(some_ptr(&D2D_RECT_F { left, top, right: left + size, bottom: top + size })),
                1.0, D2D1_BITMAP_INTERPOLATION_MODE_LINEAR, None);
        } else if let Some(fallback) = &item.fallback {
            let _ = rt.DrawBitmap(fallback, Some(some_ptr(&D2D_RECT_F {
                left: left + size * 0.12, top: top + size * 0.12,
                right: left + size * 0.88, bottom: top + size * 0.88,
            })), 1.0, D2D1_BITMAP_INTERPOLATION_MODE_LINEAR, None);
        } else if let Some(brush) = st.palette.get(item.color_idx % st.palette.len()) {
            let _ = rt.FillRoundedRectangle(some_ptr(&D2D1_ROUNDED_RECT {
                rect: D2D_RECT_F { left, top, right: left + size, bottom: top + size },
                radiusX: 13.0, radiusY: 13.0,
            }), brush);
        }
    }
}

// 磁贴本体(主绘 + 倒影共用):真实图标 → SQ 图标+徽章 → 彩色字符兜底
unsafe fn draw_tile_body(st: &State, i: usize, left: f32, top: f32, size: f32, sc: f32) {
    let rt = &st.gpu.rt;
    
    let tile = &st.tiles[i];
    if let Some(bmp) = &tile.bitmap {
        // squircle 底衬(白 5%):统一非规则 Windows 图标
        if !tile.is_plugin {
            let _ = rt.FillRoundedRectangle(some_ptr(&D2D1_ROUNDED_RECT {
                rect: D2D_RECT_F { left: left - 3.0, top: top - 3.0, right: left + size + 3.0, bottom: top + size + 3.0 },
                radiusX: 0.24 * size, radiusY: 0.24 * size,
            }), &st.soft);
        }
        let _ = rt.DrawBitmap(bmp, Some(some_ptr(&D2D_RECT_F { left, top, right: left + size, bottom: top + size })),
            1.0, D2D1_BITMAP_INTERPOLATION_MODE_LINEAR, None);
    } else if let Some(sb) = tile.sq.and_then(|k| st.sq.get(k)) {
        let _ = rt.DrawBitmap(sb, Some(some_ptr(&D2D_RECT_F { left, top, right: left + size, bottom: top + size })),
            1.0, D2D1_BITMAP_INTERPOLATION_MODE_LINEAR, None);
        // 插件徽章:底部深色药丸 + 数值(时钟 12:34 等)——倒影里不画
        let lbl = tile.label.as_deref().unwrap_or_default();
        if !lbl.is_empty() {
            let txt: Vec<u16> = lbl.encode_utf16().collect();
            let bw = (txt.len() as f32 * 7.2 + 14.0).min(size * 0.98);
            let bh = 17.0f32.min(size * 0.42);
            let bx = left + size / 2.0 - bw / 2.0;
            let by = top + size - bh / 2.0;
            let _ = rt.FillRoundedRectangle(some_ptr(&D2D1_ROUNDED_RECT {
                rect: D2D_RECT_F { left: bx, top: by, right: bx + bw, bottom: by + bh },
                radiusX: bh / 2.0, radiusY: bh / 2.0,
            }), &st.pill);
            let _ = rt.DrawText(&txt, &st.plugin_fmt_sm,
                some_ptr(&D2D_RECT_F { left: bx, top: by - 1.0, right: bx + bw, bottom: by + bh }),
                &st.white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
        }
    } else {
        let pb = st.palette.get(tile.color_idx % st.palette.len()).unwrap();
        let _ = rt.FillRoundedRectangle(some_ptr(&D2D1_ROUNDED_RECT {
            rect: D2D_RECT_F { left, top, right: left + size, bottom: top + size },
            radiusX: 13.0 * sc, radiusY: 13.0 * sc,
        }), pb);
        if let Some(fb) = &tile.fallback {
            let _ = rt.DrawBitmap(fb, Some(some_ptr(&D2D_RECT_F {
                left: left + size * 0.12, top: top + size * 0.12,
                right: left + size * 0.88, bottom: top + size * 0.88 })), 1.0, D2D1_BITMAP_INTERPOLATION_MODE_LINEAR, None);
        }
    }
}

// 字符兜底位图
pub(crate) unsafe fn make_text_bitmap(
    dc: &ID2D1DeviceContext,
    white: &ID2D1SolidColorBrush,
    fmt: &IDWriteTextFormat,
    text: &str,
    _font: f32,
) -> Option<ID2D1Bitmap1> {
    let props = D2D1_BITMAP_PROPERTIES1 {
        pixelFormat: D2D1_PIXEL_FORMAT { format: DXGI_FORMAT_B8G8R8A8_UNORM, alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED },
        dpiX: 96.0, dpiY: 96.0,
        bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET,
        colorContext: std::mem::ManuallyDrop::new(None),
    };
    let ib = dc.CreateBitmap(D2D_SIZE_U { width: 64, height: 64 }, None, 0, std::ptr::from_ref(&props)).ok()?;
    let prev = dc.GetTarget().ok();
    dc.SetTarget(&ib.cast::<ID2D1Image>().ok()?);
    dc.BeginDraw();
    let ch: Vec<u16> = text.encode_utf16().take(4).collect();
    let _ = dc.DrawText(&ch, fmt,
        std::ptr::from_ref(&D2D_RECT_F { left: 0.0, top: 0.0, right: 64.0, bottom: 64.0 }),
        white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
    let _ = dc.EndDraw(None, None);
    if let Some(p) = prev { dc.SetTarget(&p); }
    Some(ib)
}

pub(crate) fn palette_brushes(rt: &ID2D1RenderTarget) -> Vec<ID2D1SolidColorBrush> {
    let cols: [(f32, f32, f32); 8] = [
        (0.255, 0.816, 0.949), (0.329, 0.706, 1.0), (1.0, 0.435, 0.616), (0.302, 0.933, 0.624),
        (0.561, 0.4, 1.0), (1.0, 0.71, 0.24), (0.4, 0.85, 0.94), (0.95, 0.5, 0.35),
    ];
    unsafe {
        cols.iter().map(|c| rt.CreateSolidColorBrush(
            std::ptr::from_ref(&D2D1_COLOR_F { r: c.0, g: c.1, b: c.2, a: 0.92 }), None).unwrap()).collect()
    }
}
