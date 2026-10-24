// ─────────────────────────────────────────────────────────────────────────────
// Vitro v3 — 全量重写
//   行为底:NeXuS 逆向规格 R1-R12(D:\rea\nexus-dock-behavior-spec.md,Frida 证据)
//   代码底:Glassboard windows/ 模块化结构(setup/z_order/tray/preferences 思路)
//   视觉底:vitro-nexus-full.html(SQ 图标库 + Tahoe 玻璃 + 倒影 + docklet 徽章)
// 硬预算:RAM ≤30MB · 空闲 CPU ≈0%(零轮询构造保证)· 单文件原生
// 架构决策:窗口 = 固定宽透明画布(交换链一次成型,永不 Resize);
//          玻璃药丸按磁贴数自适应宽度居中绘制,药丸外像素由 DComp 透明穿透。
// ─────────────────────────────────────────────────────────────────────────────
#![allow(non_snake_case)]
#![allow(static_mut_refs)]

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::DirectWrite::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWINDOWATTRIBUTE};
use windows::Win32::System::Threading::{GetCurrentProcess, SetProcessWorkingSetSize};

mod config;
mod icons;
mod gpu;
mod state;
mod input;
mod tray;
mod dockwin;
mod lpad { include!("lpad.rs"); }
mod plugin { include!("plugin.rs"); }

use config::DockConfig;
use state::*;

fn main() -> Result<()> {
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let cfg: DockConfig = config::load_config();
        let theme = config::load_theme();
        let hinstance = GetModuleHandleW(None)?;

        // 窗口类(dock + tooltip)
        let wc = WNDCLASSW {
            lpfnWndProc: Some(dockwin::wndproc),
            hInstance: hinstance.into(),
            lpszClassName: w!("VitroDock"),
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            ..Default::default()
        };
        RegisterClassW(&wc);
        let twc = WNDCLASSW {
            lpfnWndProc: Some(dockwin::tip_wndproc),
            hInstance: hinstance.into(),
            lpszClassName: w!("VitroTip"),
            hbrBackground: CreateSolidBrush(COLORREF(0x00151A28)),
            ..Default::default()
        };
        RegisterClassW(&twc);

        // 固定画布:底部居中
        let sw = GetSystemMetrics(SM_CXSCREEN);
        let sh = GetSystemMetrics(SM_CYSCREEN);
        let win_w = WIN_W.min(sw - 160);
        let win_x = (sw - win_w) / 2;
        let base_y = sh - WIN_H - 8;

        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOREDIRECTIONBITMAP,
            w!("VitroDock"), w!("Vitro Dock"), WS_POPUP,
            win_x, base_y, win_w, WIN_H, None, None, HINSTANCE::from(hinstance), None,
        )?;
        let tip_hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_LAYERED,
            w!("VitroTip"), w!(""), WS_POPUP, 0, 0, 160, 34, None, None, HINSTANCE::from(hinstance), None,
        )?;
        let _ = SetLayeredWindowAttributes(tip_hwnd, COLORREF(0), 235, LWA_ALPHA);

        // DWM 系统亚克力(Win11 22621+;旧系统静默回退)
        let backdrop_ok = {
            let v: i32 = 3; // DWMSBT_TRANSIENTWINDOW
            DwmSetWindowAttribute(hwnd, DWMWINDOWATTRIBUTE(38),
                &v as *const i32 as *const core::ffi::c_void, 4).is_ok()
        };

        // GPU 管线一次成型
        let gpu = gpu::init(hwnd)?;

        // 画笔(主题色;Ctrl+Alt+R 热重建)
        let rt = &gpu.rt;
        let cptr = |c: &D2D1_COLOR_F| std::ptr::from_ref(c);
        let glass = rt.CreateSolidColorBrush(cptr(&D2D1_COLOR_F { r: theme.tint[0], g: theme.tint[1], b: theme.tint[2], a: theme.alpha }), None)?;
        let white = rt.CreateSolidColorBrush(cptr(&D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 0.72 }), None)?;
        let accent = rt.CreateSolidColorBrush(cptr(&D2D1_COLOR_F { r: theme.accent[0], g: theme.accent[1], b: theme.accent[2], a: 1.0 }), None)?;
        let halo = rt.CreateSolidColorBrush(cptr(&D2D1_COLOR_F { r: theme.accent[0], g: theme.accent[1], b: theme.accent[2], a: 0.30 }), None)?;
        let hi = rt.CreateSolidColorBrush(cptr(&D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 0.16 }), None)?; // Sonoma 深色:顶内高光 white 15%±
        let lo = rt.CreateSolidColorBrush(cptr(&D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 0.12 }), None)?; // 底内影 black 12%
        let border = rt.CreateSolidColorBrush(cptr(&D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 0.10 }), None)?; // HTML 基准 inset 描边 .09-.10
        let soft = rt.CreateSolidColorBrush(cptr(&D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 0.05 }), None)?;
        let pill = rt.CreateSolidColorBrush(cptr(&D2D1_COLOR_F { r: 0.04, g: 0.06, b: 0.11, a: 0.68 }), None)?;
        let palette = dockwin::palette_brushes(rt);
        let noise = dockwin::build_noise(&gpu.dc)?;

        // 倒影渐变遮罩 + 图层
        let stops = [
            D2D1_GRADIENT_STOP { position: 0.0, color: D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 0.42 } },
            D2D1_GRADIENT_STOP { position: 1.0, color: D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 0.0 } },
        ];
        let stopcol = rt.CreateGradientStopCollection(&stops, D2D1_GAMMA_2_2, D2D1_EXTEND_MODE_CLAMP)?;
        let grad_lin = rt.CreateLinearGradientBrush(
            std::ptr::from_ref(&D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES {
                startPoint: D2D_POINT_2F { x: 0.0, y: 0.0 },
                endPoint: D2D_POINT_2F { x: 0.0, y: 1.0 },
            }), None, &stopcol)?;
        let grad_refl: ID2D1Brush = grad_lin.cast()?;
        let layer = rt.CreateLayer(None)?;

        // SQ 图标库(HTML dock 视觉资产,编译期内嵌)
        let sq = icons::build_sq_map(&gpu.dc);

        // 磁贴:应用(Shell 三档提取,SQ 兜底)
        let mut tiles: Vec<Tile> = Vec::new();
        for item in &cfg.items {
            let bitmap = icons::extract_icon_96(&gpu.dc, &item.path);
            let sqk = if bitmap.is_none() { icons::sq_guess(&item.name) } else { None };
            let fallback = if bitmap.is_none() && sqk.is_none() {
                dockwin::make_text_bitmap(&gpu.dc, &white,
                    item.name.chars().next().map(|c| c.to_string()).unwrap_or_default().as_str(), 24.0)
            } else { None };
            let color_idx = item.name.bytes().map(|b| b as usize).sum::<usize>() % 8;
            tiles.push(Tile { name: item.name.clone(), path: item.path.clone(),
                bitmap, fallback, is_plugin: false, color_idx, sq: sqk, label: None });
        }
        // 插件磁贴(boa 沙箱 → SQ 图标 + 数值徽章)
        for ps in &cfg.plugins {
            match plugin::load_plugin(&ps.name, &ps.code) {
                Ok(text) => {
                    let key: Option<&'static str> = if ps.icon.is_empty() { icons::sq_guess(&ps.name) }
                        else { Some(Box::leak(ps.icon.clone().into_boxed_str())) };
                    let fallback = match key {
                        Some(k) if sq.contains_key(k) => None,
                        _ => dockwin::make_text_bitmap(&gpu.dc, &white,
                            ps.name.chars().next().map(|c| c.to_string()).unwrap_or_default().as_str(), 24.0),
                    };
                    let color_idx = ps.name.bytes().map(|b| b as usize).sum::<usize>() % 8;
                    tiles.push(Tile { name: ps.name.clone(), path: String::new(),
                        bitmap: None, fallback, is_plugin: true, color_idx, sq: key, label: Some(text) });
                    println!("[vitro] plugin loaded: {}", ps.name);
                }
                Err(e) => println!("[vitro] plugin error: {} — {}", ps.name, e),
            }
        }

        let n0 = tiles.len();
        STATE = Some(State {
            hwnd, tip_hwnd, gpu,
            tiles,
            glass, white, accent, halo, hi, lo, border, soft, pill, palette,
            sq, grad_refl, grad_lin, layer, noise: Some(noise),
            cursor: (-9999.0, -9999.0),
            hover_idx: None, hover_since: 0, tip_shown_for: None,
            hook: HHOOK::default(), was_near_flag: false, last_post_tick: 0,
            win_w, win_x, win_y: base_y, base_y, screen_h: sh,
            pill_w: 0,
            slide: 0.0, slide_target: 0.0, hide_armed: false, autohide: cfg.autohide,
            low_power: false, anim_on: false,
            cpu_prev: vec![(0, 0); n0], cpu_pct: vec![0.0; n0], last_anim_tick: 0,
            backdrop_ok, scale_anim: vec![1.0; n0], running: vec![None; n0],
            plugins: cfg.plugins.clone(),
            theme: theme.clone(),
        });
        s().relayout();

        // 输入与系统集成(全部事件驱动)
        let _ = DragAcceptFiles(hwnd, TRUE);
        tray::init(hwnd);
        s().hook = SetWindowsHookExW(WH_MOUSE_LL, Some(input::ll_hook), hinstance, 0)?;
        let _ = RegisterHotKey(hwnd, 1, MOD_CONTROL | MOD_ALT, 0x44); // Ctrl+Alt+D 抽屉
        let _ = RegisterHotKey(hwnd, 2, MOD_CONTROL | MOD_ALT, 0x52); // Ctrl+Alt+R 主题
        lpad::init(HINSTANCE::from(hinstance))?;
        input::update_power();

        let _ = ShowWindow(hwnd, SW_SHOW);
        dockwin::render_frame();

        // 首帧后软回收(DWrite 初始化页进 standby,回 fault 近零)
        let _ = SetProcessWorkingSetSize(GetCurrentProcess(), usize::MAX, usize::MAX);
        println!("[vitro] v3 ready: tiles={} pill_w={} backdrop_ok={}", s().tiles.len(), s().pill_w, backdrop_ok);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        Ok(())
    }
}
