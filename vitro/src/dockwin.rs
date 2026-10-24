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
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Graphics::Gdi::{SetBkMode, SetTextColor, TRANSPARENT};

use crate::config::{self, DockConfig, DockItem, PluginSpec};
use crate::icons::{extract_icon_96, sq_guess};
use crate::input;
use crate::lpad;
use crate::state::*;

// D2D 裸指针参数统一封装(0.58 全是 Option<*const T>)
pub(crate) fn some_ptr<T>(t: &T) -> *const T { t }

// ── 状态保存(NeXuS R11)──
pub(crate) fn save_state() {
    unsafe {
        let st = s();
        let items: Vec<DockItem> = st.tiles.iter()
            .filter(|t| !t.is_plugin && !t.path.is_empty())
            .map(|t| DockItem { name: t.name.clone(), path: t.path.clone() }).collect();
        let cfg = DockConfig { items, plugins: st.plugins.clone(), autohide: st.autohide };
        config::save_config(&cfg);
    }
}

// ── 窗口过程 ──
pub(crate) unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_APP_MOUSE => { on_mouse(); LRESULT(0) }
        WM_TIMER => {
            match wp.0 as u32 {
                TIMER_DWELL => on_dwell(),
                TIMER_ANIM => render_frame(),
                TIMER_HIDE => input::on_hide_timer(),
                _ => {}
            }
            LRESULT(0)
        }
        WM_LBUTTONDOWN => { on_click(lp.0 as i32 & 0xFFFF, (lp.0 as i32 >> 16) & 0xFFFF); LRESULT(0) }
        WM_RBUTTONDOWN => { let _ = on_menu(lp.0 as i32 & 0xFFFF, (lp.0 as i32 >> 16) & 0xFFFF); LRESULT(0) }
        WM_KEYDOWN | WM_CHAR => { if wp.0 as u16 == 27 { shutdown(hwnd); } LRESULT(0) }
        WM_POWERBROADCAST => { if wp.0 as u32 == 10 { input::update_power(); } LRESULT(0) }
        WM_HOTKEY => {
            if wp.0 as i32 == 2 { let _ = reload_theme(); } else { lpad::toggle(); }
            LRESULT(0)
        }
        WM_DROPFILES => { on_drop(HDROP(wp.0 as *mut core::ffi::c_void)); LRESULT(0) }
        WM_APP_TRAY => {
            let ev = (lp.0 as u32) & 0xFFFF;
            if ev == WM_RBUTTONUP || ev == WM_LBUTTONUP { let _ = crate::tray::on_tray_menu(); }
            LRESULT(0)
        }
        WM_DESTROY => { shutdown(hwnd); LRESULT(0) }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

pub(crate) unsafe extern "system" fn tip_wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if msg == WM_PAINT {
        let mut ps = PAINTSTRUCT::default();
        let hdc = BeginPaint(hwnd, &mut ps);
        let st = s();
        if let Some(idx) = st.tip_shown_for {
            if idx < st.tiles.len() {
                let mut name: Vec<u16> = st.tiles[idx].name.encode_utf16().chain(std::iter::once(0)).collect();
                let mut rc = RECT { left: 0, top: 0, right: 160, bottom: 34 };
                let _ = SetBkMode(hdc, TRANSPARENT);
                let _ = SetTextColor(hdc, COLORREF(0x00F4F8FF));
                let _ = DrawTextW(hdc, &mut name, &mut rc, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
            }
        }
        let _ = EndPaint(hwnd, &ps);
        return LRESULT(0);
    }
    DefWindowProcW(hwnd, msg, wp, lp)
}

// ── 悬停 / tooltip(NeXuS R3:200ms dwell)──
unsafe fn on_mouse() {
    let st = s();
    let h = st.hit_test(st.cursor.0 as i32, st.cursor.1 as i32);
    if h != st.hover_idx {
        st.hover_idx = h;
        st.hover_since = GetTickCount();
        if h.is_some() { SetTimer(st.hwnd, TIMER_DWELL as usize, 120, None); }
        else { KillTimer(st.hwnd, TIMER_DWELL as usize); hide_tip(); }
    }
    render_frame();
}

unsafe fn on_dwell() {
    let st = s();
    if st.hover_idx.is_some() && st.tip_shown_for.is_none()
        && GetTickCount().wrapping_sub(st.hover_since) >= 200 {
        show_tip(st.hover_idx.unwrap());
    }
}

unsafe fn show_tip(idx: usize) {
    let st = s();
    st.tip_shown_for = Some(idx);
    let (cx, _, _) = st.tile_center_local(idx);
    let px = st.pill_x();
    let _ = SetWindowPos(st.tip_hwnd, HWND_TOPMOST,
        st.win_x + (px + cx - 80.0) as i32, st.win_y - 38, 0, 0, SWP_NOSIZE | SWP_SHOWWINDOW);
    let _ = InvalidateRect(st.tip_hwnd, None, TRUE);
}

pub(crate) unsafe fn hide_tip() {
    let st = s();
    if st.tip_shown_for.is_some() {
        st.tip_shown_for = None;
        let _ = ShowWindow(st.tip_hwnd, SW_HIDE);
    }
}

// ── 启动(NeXuS R4/R5:ShellExecute + 句柄跟踪)──
unsafe fn on_click(x: i32, y: i32) {
    let st = s();
    // 屏幕坐标命中(lp 是客户区坐标 → 平移窗口原点)
    if let Some(i) = st.hit_test(st.win_x + x, st.win_y + y) { launch(i); }
}

unsafe fn launch(i: usize) {
    let st = s();
    if i >= st.tiles.len() { return; }
    let t = &st.tiles[i];
    if t.is_plugin || t.path.is_empty() { return; }
    let wpath: Vec<u16> = t.path.encode_utf16().chain(std::iter::once(0)).collect();
    let mut sei = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS,
        lpFile: PCWSTR(wpath.as_ptr()),
        lpVerb: w!("open"),
        nShow: SW_SHOWNORMAL.0,
        ..Default::default()
    };
    if ShellExecuteExW(&mut sei).is_ok() && !sei.hProcess.is_invalid() {
        if let Some(h) = st.running.get_mut(i) { *h = Some(sei.hProcess); }
    }
    render_frame();
}

// ── 右键菜单(NeXuS R7)──
unsafe fn on_menu(x: i32, y: i32) -> Result<()> {
    let st = s();
    let sx = st.win_x + x;
    let sy = st.win_y + y;
    let on_icon = st.hit_test(sx, sy);
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
    let _ = SetForegroundWindow(st.hwnd);
    let cmd = TrackPopupMenu(menu, TPM_RIGHTBUTTON | TPM_RETURNCMD, sx, sy, 0, st.hwnd, None);
    let _ = DestroyMenu(menu);
    match cmd.0 {
        1 => { if let Some(i) = s().hit_test(sx, sy) { launch(i); } }
        3 => {
            let st = s();
            if let Some(i) = st.hit_test(sx, sy) {
                st.tiles.remove(i);
                if i < st.running.len() { st.running.remove(i); }
                st.relayout();
                save_state();
                render_frame();
            }
        }
        4 => lpad::toggle(),
        5 => reload_theme()?,
        2 => shutdown(s().hwnd),
        _ => {}
    }
    Ok(())
}

// ── 拖放加图标(NeXuS R6)──
unsafe fn on_drop(h: HDROP) {
    let st = s();
    let cnt = DragQueryFileW(h, u32::MAX, None);
    for i in 0..cnt {
        let mut buf = [0u16; 1024];
        let len = DragQueryFileW(h, i, Some(&mut buf)) as usize;
        if len == 0 { continue; }
        let path = String::from_utf16_lossy(&buf[..len]);
        let name = std::path::Path::new(&path).file_stem()
            .map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "项目".into());
        if st.tiles.iter().any(|t| t.path.eq_ignore_ascii_case(&path)) { continue; }
        let bitmap = extract_icon_96(&st.gpu.dc, &path);
        let sq = if bitmap.is_none() { sq_guess(&name) } else { None };
        let fallback = if bitmap.is_none() && sq.is_none() {
            make_text_bitmap(&st.gpu.dc, &st.white,
                name.chars().next().map(|c| c.to_string()).unwrap_or_default().as_str(), 24.0)
        } else { None };
        let color_idx = name.bytes().map(|b| b as usize).sum::<usize>() % 8;
        st.tiles.push(Tile { name, path, bitmap, fallback, is_plugin: false, color_idx, sq, label: None });
    }
    DragFinish(h);
    st.relayout();
    save_state();
    render_frame();
}

// ── 主题热重载(Ctrl+Alt+R)──
pub(crate) unsafe fn reload_theme() -> Result<()> {
    let t = config::load_theme();
    let st = s();
    st.theme = t.clone();
    rebuild_theme_brushes(st, &t)?;
    render_frame();
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

pub(crate) unsafe fn shutdown(hwnd: HWND) {
    let st = s();
    save_state();
    crate::tray::remove(st.hwnd);
    if !st.hook.is_invalid() { let _ = UnhookWindowsHookEx(st.hook); }
    for h in st.running.iter().flatten() { let _ = CloseHandle(*h); }
    let _ = ShowWindow(st.tip_hwnd, SW_HIDE);
    let _ = DestroyWindow(st.tip_hwnd);
    let _ = DestroyWindow(hwnd);
    PostQuitMessage(0);
}

// ── 渲染 ──
pub(crate) unsafe fn render_frame() {
    let st = s();
    let rt = &st.gpu.rt;
    
    let (cx, cy) = st.cursor; // 屏幕坐标
    let theme_peak = st.theme.peak;
    let theme_radius = st.theme.radius;
    let now = GetTickCount();
    let n = st.tiles.len();
    let px = st.pill_x();
    let pw = st.pill_w as f32;
    let backdrop_ok = st.backdrop_ok;

    // ── R9 滑出/召唤(窗口位移动画,33ms)──
    if st.slide != st.slide_target {
        let dt = now.wrapping_sub(st.last_anim_tick).max(1) as f32;
        let k = (dt / 1000.0 * 10.0).min(1.0);
        st.slide += (st.slide_target - st.slide) * k;
        if (st.slide - st.slide_target).abs() < 0.01 { st.slide = st.slide_target; }
        let y = st.base_y + (st.slide * (WIN_H as f32 + 14.0)) as i32;
        let _ = SetWindowPos(st.hwnd, HWND_TOPMOST, st.win_x, y, 0, 0, SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE);
        st.win_y = y;
    }

    let dt_ms = now.wrapping_sub(st.last_anim_tick).max(1);
    let anim = (st.cursor_inside_pill() && !st.low_power) || st.anim_on;
    if anim || st.slide != st.slide_target { st.last_anim_tick = now; }

    rt.BeginDraw();
    rt.Clear(Some(some_ptr(&D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 0.0 })));

    // 玻璃药丸:系统亚克力生效 → 淡 tint(模糊由 DWM 供给);否则浓 tint 回退
    let ga = if backdrop_ok { st.theme.alpha * 0.42 } else { st.theme.alpha };
    st.glass.SetColor(&D2D1_COLOR_F { r: st.theme.tint[0], g: st.theme.tint[1], b: st.theme.tint[2], a: ga });
    let _ = rt.FillRoundedRectangle(some_ptr(&D2D1_ROUNDED_RECT {
        rect: D2D_RECT_F { left: px, top: 0.0, right: px + pw, bottom: PANEL_H },
        radiusX: 24.0, radiusY: 24.0,
    }), &st.glass);
    let _ = rt.DrawRoundedRectangle(some_ptr(&D2D1_ROUNDED_RECT {
        rect: D2D_RECT_F { left: px + 0.5, top: 0.5, right: px + pw - 0.5, bottom: PANEL_H - 0.5 },
        radiusX: 24.0, radiusY: 24.0,
    }), &st.border, 1.0, None);
    let _ = rt.FillRoundedRectangle(some_ptr(&D2D1_ROUNDED_RECT {
        rect: D2D_RECT_F { left: px + 24.0, top: 1.0, right: px + pw - 24.0, bottom: 4.0 },
        radiusX: 2.0, radiusY: 2.0,
    }), &st.hi);
    let _ = rt.FillRoundedRectangle(some_ptr(&D2D1_ROUNDED_RECT {
        rect: D2D_RECT_F { left: px + 24.0, top: PANEL_H - 4.0, right: px + pw - 24.0, bottom: PANEL_H - 1.0 },
        radiusX: 2.0, radiusY: 2.0,
    }), &st.lo);

    // ── 磁贴:falloff 弹簧 + 呼吸 + 图标/插件 ──
    let tile_sz = tile_size(st.pill_w, n);
    for i in 0..n {
        let (icx_l, icy_l, _) = st.tile_center_local(i);
        let icx_s = st.win_x as f32 + icx_l;
        let icy_s = st.win_y as f32 + icy_l;
        let d = ((cx - icx_s).powi(2) + (cy - icy_s).powi(2)).sqrt().min(theme_radius);
        let f = 1.0 - d / theme_radius;
        let target = 1.0 + (theme_peak - 1.0) * f.powf(1.6);
        let k = (dt_ms as f32 / 1000.0 * 14.0).min(1.0);
        st.scale_anim[i] += (target - st.scale_anim[i]) * k;
        let sc = st.scale_anim[i];
        let size = tile_sz * sc;
        let left = icx_l - size / 2.0;
        let top = icy_l - size / 2.0;

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
                point: D2D_POINT_2F { x: icx_l, y: icy_l },
                radiusX: hr, radiusY: hr,
            }), &st.halo);
        }

        draw_tile_body(i, left, top, size, sc);

        // 运行指示点
        if is_run {
            let dot_r = 2.2 + wave * 1.8;
            let _ = rt.FillEllipse(some_ptr(&D2D1_ELLIPSE {
                point: D2D_POINT_2F { x: icx_l, y: PANEL_H - BASE_UP + tile_sz / 2.0 + 9.0 },
                radiusX: dot_r, radiusY: dot_r,
            }), &st.accent);
        }
    }

    // ── 倒影:PushLayer 渐变遮罩 + 镜像 ──
    st.grad_lin.SetStartPoint(D2D_POINT_2F { x: 0.0, y: PANEL_H - BASE_UP });
    st.grad_lin.SetEndPoint(D2D_POINT_2F { x: 0.0, y: WIN_H as f32 });
    let params = D2D1_LAYER_PARAMETERS1 {
        contentBounds: D2D_RECT_F { left: px, top: PANEL_H - BASE_UP - 1.0, right: px + pw, bottom: WIN_H as f32 + 40.0 },
        geometricMask: std::mem::ManuallyDrop::new(None),
        maskAntialiasMode: D2D1_ANTIALIAS_MODE_PER_PRIMITIVE,
        maskTransform: Matrix3x2 { M31: 0.0, M32: 0.0, M11: 1.0, M12: 0.0, M21: 0.0, M22: 1.0 },
        opacity: 1.0,
        opacityBrush: std::mem::ManuallyDrop::new(Some(st.grad_refl.clone())),
        layerOptions: D2D1_LAYER_OPTIONS1(0),
    };
    st.gpu.dc.PushLayer(&params, &st.layer);
    rt.SetTransform(&Matrix3x2 { M31: 0.0, M32: 2.0 * (PANEL_H - BASE_UP), M11: 1.0, M12: 0.0, M21: 0.0, M22: -1.0 });
    for i in 0..n {
        let (icx_l, icy_l, _) = st.tile_center_local(i);
        let size = tile_sz * st.scale_anim[i];
        draw_tile_body(i, icx_l - size / 2.0, icy_l - size / 2.0, size, st.scale_anim[i]);
    }
    rt.SetTransform(&Matrix3x2 { M31: 0.0, M32: 0.0, M11: 1.0, M12: 0.0, M21: 0.0, M22: 1.0 });
    rt.PopLayer();

    // ── 定时器管理:静止即撤(空闲 0%)──
    let settling = st.scale_anim.iter().take(n).any(|v| (*v - 1.0).abs() > 0.004);
    let should = (st.cursor_inside_pill() && !st.low_power) || settling || st.slide != st.slide_target;
    if should && !st.anim_on { st.anim_on = true; SetTimer(st.hwnd, TIMER_ANIM as usize, 33, None); }
    else if !should && st.anim_on { st.anim_on = false; KillTimer(st.hwnd, TIMER_ANIM as usize); }

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

// 磁贴本体(主绘 + 倒影共用):真实图标 → SQ 图标+徽章 → 彩色字符兜底
unsafe fn draw_tile_body(i: usize, left: f32, top: f32, size: f32, sc: f32) {
    let st = s();
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
        if !tile_label(i).is_empty() {
            let lbl = tile_label(i);
            let fmt = crate::plugin::plugin_fmt_sm();
            let txt: Vec<u16> = lbl.encode_utf16().collect();
            let bw = (txt.len() as f32 * 7.2 + 14.0).min(size * 0.98);
            let bh = 17.0f32.min(size * 0.42);
            let bx = left + size / 2.0 - bw / 2.0;
            let by = top + size - bh / 2.0;
            let _ = rt.FillRoundedRectangle(some_ptr(&D2D1_ROUNDED_RECT {
                rect: D2D_RECT_F { left: bx, top: by, right: bx + bw, bottom: by + bh },
                radiusX: bh / 2.0, radiusY: bh / 2.0,
            }), &st.pill);
            let _ = rt.DrawText(&txt, fmt,
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

// 徽章文本(绕开 State 双重借用)
unsafe fn tile_label(i: usize) -> String {
    s().tiles[i].label.clone().unwrap_or_default()
}

// 字符兜底位图
pub(crate) unsafe fn make_text_bitmap(dc: &ID2D1DeviceContext, white: &ID2D1SolidColorBrush, text: &str, _font: f32) -> Option<ID2D1Bitmap1> {
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
    let fmt = crate::plugin::plugin_fmt();
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
