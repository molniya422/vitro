// lpad.rs — M3 抽屉启动器:全屏网格 + 搜索 + >命令 + =计算器(boa)
// 触发:RegisterHotKey(Ctrl+Alt+D)→ WM_HOTKEY(消息驱动,零轮询)/ dock 右键"启动器…"
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_HARDWARE;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::DirectComposition::*;
use windows::Win32::Graphics::DirectWrite::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::launcher_model::{append_utf16_unit, parse_query_mode, QueryMode};
use crate::runtime::{self, AppRuntime};

const WM_APP_LPAD: u32 = 0x8001;

pub(crate) struct LpadState {
    pub(crate) hwnd: HWND,
    sc: IDXGISwapChain1,
    rt: ID2D1RenderTarget,
    fmt: Option<IDWriteTextFormat>,
    brush_bg: ID2D1SolidColorBrush,
    brush_tile: ID2D1SolidColorBrush,
    brush_tile_sel: ID2D1SolidColorBrush,
    brush_txt: ID2D1SolidColorBrush,
    brush_muted: ID2D1SolidColorBrush,
    apps: Vec<(String, String)>,
    query: String,
    visible: Vec<usize>,
    sel: usize,
    open: bool,
    calc_result: Option<String>,
    pending_high_surrogate: Option<u16>,
    sw: i32,
    sh: i32,
}

// ── 应用枚举:开始菜单 .lnk 递归 ──
fn collect_apps() -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut roots: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(pd) = std::env::var("ProgramData") {
        roots.push(std::path::Path::new(&pd).join(r"Microsoft\Windows\Start Menu\Programs"));
    }
    if let Ok(ad) = std::env::var("APPDATA") {
        roots.push(std::path::Path::new(&ad).join(r"Microsoft\Windows\Start Menu\Programs"));
    }
    for root in &roots { walk_dir(root, &mut out, 0); }
    out.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    out.truncate(300);
    out
}

fn walk_dir(dir: &std::path::Path, out: &mut Vec<(String, String)>, depth: u32) {
    if depth > 4 || out.len() > 400 { return; }
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() { walk_dir(&p, out, depth + 1); }
            else if let Some(ext) = p.extension() {
                if ext.eq_ignore_ascii_case("lnk") {
                    let name = p.file_stem().unwrap().to_string_lossy().to_string();
                    out.push((name, p.to_string_lossy().to_string()));
                }
            }
        }
    }
}

// ── =计算器:boa 引擎求值(仅白名单字符)──
fn calc_eval(expr: &str) -> Option<String> {
    use boa_engine::{Context, Source};
    if expr.is_empty() { return None; }
    if !expr.chars().all(|c| "0123456789+-*/(). %".contains(c)) { return None; }
    let mut ctx = Context::default();
    match ctx.eval(Source::from_bytes(expr.as_bytes())) {
        Ok(v) => {
            let s = v.to_string(&mut ctx).ok()?.to_std_string_escaped();
            Some(s)
        }
        Err(_) => None,
    }
}

// ── 懒创建 DWrite(与 plugin.rs 同模式)──
unsafe fn lpad_fmt(lp: &mut LpadState) -> &IDWriteTextFormat {
    if lp.fmt.is_none() {
        let dw: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED).unwrap();
        let fmt = dw.CreateTextFormat(
            w!("Microsoft YaHei UI"), None, DWRITE_FONT_WEIGHT_SEMI_BOLD, DWRITE_FONT_STYLE_NORMAL,
            DWRITE_FONT_STRETCH_NORMAL, 16.0, w!("zh-CN"),
        ).unwrap();
        let _ = fmt.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);
        let _ = fmt.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);
        lp.fmt = Some(fmt);
    }
    lp.fmt.as_ref().unwrap()
}

pub(crate) unsafe fn init(hinstance: HINSTANCE) -> Result<LpadState> {
    let (sw, sh) = (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN));
    let wc = WNDCLASSW {
        lpfnWndProc: Some(lpad_wndproc),
        hInstance: hinstance.into(),
        lpszClassName: w!("VitroLpad"),
        hCursor: LoadCursorW(None, IDC_ARROW)?,
        ..Default::default()
    };
    RegisterClassW(&wc);
    let hwnd = CreateWindowExW(
        WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOREDIRECTIONBITMAP,
        w!("VitroLpad"), w!("Vitro 启动器"), WS_POPUP,
        0, 0, sw, sh, None, None, HINSTANCE::from(hinstance), None,
    )?;

    let lp = crate::runtime::cleanup_after_startup_error(
        (|| -> Result<LpadState> {
            let mut d3d = None; let mut ctx3 = None;
            D3D11CreateDevice(None, D3D_DRIVER_TYPE_HARDWARE, HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT, None, D3D11_SDK_VERSION,
                Some(&mut d3d), None, Some(&mut ctx3))?;
            let d3d = d3d.unwrap();
            let dxgi_dev: IDXGIDevice = d3d.cast()?;
            let factory: IDXGIFactory2 = CreateDXGIFactory1::<IDXGIFactory1>()?.cast()?;
            let d2d_factory: ID2D1Factory1 = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;
            let d2d_device = d2d_factory.CreateDevice(&dxgi_dev)?;
            let dc: ID2D1DeviceContext = d2d_device.CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?;
            let sc = factory.CreateSwapChainForComposition(&d3d, &DXGI_SWAP_CHAIN_DESC1 {
                Width: sw as u32, Height: sh as u32,
                Format: DXGI_FORMAT_B8G8R8A8_UNORM, Stereo: false.into(),
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT, BufferCount: 2,
                Scaling: DXGI_SCALING_STRETCH, SwapEffect: DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
                AlphaMode: DXGI_ALPHA_MODE_PREMULTIPLIED, Flags: 0,
            }, None)?;
            let buffer: IDXGISurface = sc.GetBuffer(0)?;
            let props = D2D1_BITMAP_PROPERTIES1 {
                pixelFormat: D2D1_PIXEL_FORMAT { format: DXGI_FORMAT_B8G8R8A8_UNORM, alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED },
                dpiX: 96.0, dpiY: 96.0,
                bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
                colorContext: std::mem::ManuallyDrop::new(None),
            };
            let bmp = dc.CreateBitmapFromDxgiSurface(&buffer, Some(std::ptr::from_ref(&props)))?;
            dc.SetTarget(&bmp.cast::<ID2D1Image>()?);

            let comp: IDCompositionDevice = DCompositionCreateDevice(&dxgi_dev)?;
            let target = comp.CreateTargetForHwnd(hwnd, true)?;
            let visual = comp.CreateVisual()?;
            visual.SetContent(&sc)?;
            target.SetRoot(&visual)?;
            comp.Commit()?;

            let rt: ID2D1RenderTarget = dc.cast()?;
            let bg = rt.CreateSolidColorBrush(std::ptr::from_ref(&D2D1_COLOR_F { r: 0.02, g: 0.03, b: 0.05, a: 0.86 }), None)?;
            let tile = rt.CreateSolidColorBrush(std::ptr::from_ref(&D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 0.06 }), None)?;
            let tile_sel = rt.CreateSolidColorBrush(std::ptr::from_ref(&D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 0.17 }), None)?;
            let txt = rt.CreateSolidColorBrush(std::ptr::from_ref(&D2D1_COLOR_F { r: 0.94, g: 0.96, b: 1.0, a: 1.0 }), None)?;
            let muted = rt.CreateSolidColorBrush(std::ptr::from_ref(&D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 0.55 }), None)?;

            let apps = collect_apps();
            let mut lp = LpadState {
                hwnd, sc, rt,
                fmt: None,
                brush_bg: bg, brush_tile: tile, brush_tile_sel: tile_sel, brush_txt: txt, brush_muted: muted,
                apps, query: String::new(), visible: Vec::new(), sel: 0,
                open: false, calc_result: None, pending_high_surrogate: None, sw, sh,
            };
            refilter(&mut lp);
            Ok(lp)
        })(),
        || { let _ = DestroyWindow(hwnd); },
    )?;
    Ok(lp)
}

fn refilter(lp: &mut LpadState) {
    let (mode_cmd, mode_calc, q0) = match parse_query_mode(&lp.query) {
        QueryMode::Command(q) => (true, false, q),
        QueryMode::Calculation(q) => (false, true, q),
        QueryMode::Apps(q) => (false, false, q),
    };
    lp.visible.clear();
    if !mode_cmd && !mode_calc {
        for (i, (name, _)) in lp.apps.iter().enumerate() {
            if q0.is_empty() || name.to_lowercase().contains(&q0) { lp.visible.push(i); }
        }
    }
    if mode_calc { lp.calc_result = calc_eval(&q0); } else { lp.calc_result = None; }
    lp.sel = 0;
}

pub(crate) unsafe fn toggle(runtime: *mut AppRuntime) {
    let Some(lp) = (*runtime).launcher.as_mut() else { return; };
    if lp.open {
        lp.open = false;
        let _ = ShowWindow(lp.hwnd, SW_HIDE);
    } else {
        if lp.apps.is_empty() { lp.apps = collect_apps(); refilter(lp); }
        lp.open = true;
        lp.query.clear(); lp.calc_result = None; lp.pending_high_surrogate = None; refilter(lp);
        let _ = ShowWindow(lp.hwnd, SW_SHOW);
        let _ = SetForegroundWindow(lp.hwnd);
        render_lpad(runtime);
    }
}

unsafe fn lpad_shutdown(runtime: *mut AppRuntime) {
    let Some(lp) = (*runtime).launcher.as_mut() else { return; };
    lp.open = false;
    let _ = ShowWindow(lp.hwnd, SW_HIDE);
}

pub(crate) unsafe fn shutdown(runtime: *mut AppRuntime) {
    let Some(lp) = (*runtime).launcher.take() else { return; };
    let _ = ShowWindow(lp.hwnd, SW_HIDE);
    let _ = DestroyWindow(lp.hwnd);
}

unsafe extern "system" fn lpad_wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lpm: LPARAM) -> LRESULT {
    let Some(runtime) = runtime::from_window(hwnd) else {
        return DefWindowProcW(hwnd, msg, wp, lpm);
    };
    let runtime = runtime.as_ptr();
    match msg {
        WM_CHAR => {
            let unit = wp.0 as u16;
            let Some(lpv) = (*runtime).launcher.as_mut() else { return LRESULT(0); };
            if unit == 8 {
                lpv.pending_high_surrogate = None;
                lpv.query.pop();
            } else if unit >= 0x20 {
                append_utf16_unit(&mut lpv.query, &mut lpv.pending_high_surrogate, unit);
            }
            lpv.calc_result = None;
            refilter(lpv);
            render_lpad(runtime);
            LRESULT(0)
        }
        WM_KEYDOWN => {
            let vk = wp.0 as u32;
            let Some(lpv) = (*runtime).launcher.as_mut() else { return LRESULT(0); };
            match vk {
                0x1B => { lpad_shutdown(runtime); LRESULT(0) }          // Esc
                0x0D => { on_enter(runtime); LRESULT(0) }                    // Enter
                0x26 => {
                    let changed = lpv.sel > 0;
                    if changed { lpv.sel -= 1; }
                    if changed { render_lpad(runtime); }
                    LRESULT(0)
                }  // Up
                0x28 => {
                    let changed = lpv.sel + 1 < lpv.visible.len();
                    if changed { lpv.sel += 1; }
                    if changed { render_lpad(runtime); }
                    LRESULT(0)
                }  // Down
                _ => LRESULT(0)
            }
        }
        WM_LBUTTONDOWN => {
            let (x, y) = (lpm.0 as i32 & 0xFFFF, (lpm.0 as i32 >> 16) & 0xFFFF);
            on_lpad_click(runtime, x, y);
            LRESULT(0)
        }
        WM_DESTROY => LRESULT(0),
        _ => DefWindowProcW(hwnd, msg, wp, lpm),
    }
}

unsafe fn on_enter(runtime: *mut AppRuntime) {
    let Some(lp) = (*runtime).launcher.as_mut() else { return; };
    let q = lp.query.clone();
    if let Some(cmd) = q.strip_prefix('>') {
        let c = cmd.trim();
        if !c.is_empty() {
            let cmd_w: Vec<u16> = format!("/c {}\0", c).encode_utf16().collect();
            let _ = ShellExecuteW(None, w!("open"), w!("cmd.exe"), PCWSTR(cmd_w.as_ptr()), None, SW_HIDE);
        }
        lpad_shutdown(runtime);
        return;
    }
    if q.starts_with('=') { return; } // 计算结果已实时显示
    let path = if lp.sel < lp.visible.len() {
        Some(lp.apps[lp.visible[lp.sel]].1.clone())
    } else {
        None
    };
    if let Some(path) = path {
        let wpath: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
        let _ = ShellExecuteW(None, w!("open"), PCWSTR(wpath.as_ptr()), None, None, SW_SHOWNORMAL);
        lpad_shutdown(runtime);
    }
}

unsafe fn on_lpad_click(runtime: *mut AppRuntime, x: i32, y: i32) {
    if let Some(idx) = grid_hit(runtime, x, y) {
        let path = (*runtime).launcher.as_ref().and_then(|lp| {
            lp.visible.get(idx).and_then(|visible_idx| lp.apps.get(*visible_idx)).map(|app| app.1.clone())
        });
        if let Some(path) = path {
            let wpath: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
            let _ = ShellExecuteW(None, w!("open"), PCWSTR(wpath.as_ptr()), None, None, SW_SHOWNORMAL);
            lpad_shutdown(runtime);
        }
    }
}

// 网格几何:5 列,瓦片 170x110,起点 y=250
unsafe fn grid_cell(runtime: *mut AppRuntime, i: usize) -> (f32, f32, f32, f32) {
    let Some(lp) = (*runtime).launcher.as_ref() else { return (0.0, 0.0, 0.0, 0.0); };
    let cols = 5usize;
    let tw = 170.0f32; let th = 110.0f32; let gx = 18.0f32; let gy = 14.0f32;
    let row = (i / cols) as f32; let col = (i % cols) as f32;
    let total_w = cols as f32 * tw + (cols - 1) as f32 * gx;
    let x0 = (lp.sw as f32 - total_w) / 2.0;
    let y0 = 250.0;
    (x0 + col * (tw + gx), y0 + row * (th + gy), tw, th)
}

unsafe fn grid_hit(runtime: *mut AppRuntime, x: i32, y: i32) -> Option<usize> {
    let Some(lp) = (*runtime).launcher.as_ref() else { return None; };
    for i in 0..lp.visible.len().min(15) {
        let (l, t, w, h) = grid_cell(runtime, i);
        if x as f32 >= l && x as f32 <= l + w && y as f32 >= t && y as f32 <= t + h { return Some(i); }
    }
    None
}

unsafe fn render_lpad(runtime: *mut AppRuntime) {
    let Some(lp) = (*runtime).launcher.as_mut() else { return; };
    let _ = lpad_fmt(lp);
    let Some(lp) = (*runtime).launcher.as_ref() else { return; };
    let f = lp.fmt.as_ref().unwrap();
    let (rt, bg, tile, tile_sel, txt, muted) =
        (&lp.rt, &lp.brush_bg, &lp.brush_tile, &lp.brush_tile_sel, &lp.brush_txt, &lp.brush_muted);
    let cptr = |c: &D2D1_COLOR_F| Some(std::ptr::from_ref(c));

    rt.BeginDraw();
    rt.Clear(cptr(&D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }));
    let _ = rt.FillRectangle(std::ptr::from_ref(&D2D_RECT_F {
        left: 0.0, top: 0.0, right: lp.sw as f32, bottom: lp.sh as f32 }), bg);

    // 标题 + 提示
    let title: Vec<u16> = "Vitro 启动器".encode_utf16().collect();
    let _ = rt.DrawText(&title, f,
        std::ptr::from_ref(&D2D_RECT_F { left: 0.0, top: 90.0, right: lp.sw as f32, bottom: 130.0 }),
        txt, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
    let hint: Vec<u16> = "输入搜索 · > 命令 · = 计算 · ↑↓ 选择 · Enter 打开 · Esc 关闭".encode_utf16().collect();
    let _ = rt.DrawText(&hint, f,
        std::ptr::from_ref(&D2D_RECT_F { left: 0.0, top: 136.0, right: lp.sw as f32, bottom: 166.0 }),
        muted, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);

    // 搜索框
    let swf = lp.sw as f32;
    let _ = rt.FillRoundedRectangle(
        std::ptr::from_ref(&D2D1_ROUNDED_RECT {
            rect: D2D_RECT_F { left: swf / 2.0 - 260.0, top: 180.0, right: swf / 2.0 + 260.0, bottom: 226.0 },
            radiusX: 12.0, radiusY: 12.0,
        }), tile);
    let disp: String = {
        let q = &lp.query;
        let mut s = q.clone();
        if lp.open { s.push('|'); } // 简易光标
        s
    };
    let dq: Vec<u16> = disp.encode_utf16().collect();
    let _ = rt.DrawText(&dq, f,
        std::ptr::from_ref(&D2D_RECT_F { left: swf / 2.0 - 250.0, top: 184.0, right: swf / 2.0 + 250.0, bottom: 222.0 }),
        txt, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);

    // = 计算结果(实时)
    if let Some(res) = &lp.calc_result {
        let line: Vec<u16> = format!("= {}", res).encode_utf16().collect();
        let _ = rt.DrawText(&line, f,
            std::ptr::from_ref(&D2D_RECT_F { left: 0.0, top: 232.0, right: lp.sw as f32, bottom: 264.0 }),
            txt, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
    }

    // 命令模式提示
    if lp.query.starts_with('>') {
        let c: Vec<u16> = "命令模式 — Enter 执行(cmd /c)".encode_utf16().collect();
        let _ = rt.DrawText(&c, f,
            std::ptr::from_ref(&D2D_RECT_F { left: 0.0, top: 232.0, right: lp.sw as f32, bottom: 264.0 }),
            muted, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
    } else {
        // 应用网格(≤15)
        let max = lp.visible.len().min(15);
        for i in 0..max {
            let cols = 5usize;
            let tw = 170.0f32; let th = 110.0f32; let gx = 18.0f32; let gy = 14.0f32;
            let total_w = cols as f32 * tw + (cols - 1) as f32 * gx;
            let x0 = (lp.sw as f32 - total_w) / 2.0;
            let l = x0 + (i % cols) as f32 * (tw + gx);
            let t = 250.0 + (i / cols) as f32 * (th + gy);
            let (w, h) = (tw, th);
            let brush = if i == lp.sel { tile_sel } else { tile };
            let _ = rt.FillRoundedRectangle(
                std::ptr::from_ref(&D2D1_ROUNDED_RECT {
                    rect: D2D_RECT_F { left: l, top: t, right: l + w, bottom: t + h },
                    radiusX: 14.0, radiusY: 14.0,
                }), brush);
            let name = &lp.apps[lp.visible[i]].0;
            let mut disp_name: String = name.clone();
            if disp_name.chars().count() > 14 { disp_name = disp_name.chars().take(13).collect::<String>() + "…"; }
            let dn: Vec<u16> = disp_name.encode_utf16().collect();
            let _ = rt.DrawText(&dn, f,
                std::ptr::from_ref(&D2D_RECT_F { left: l + 6.0, top: t + 8.0, right: l + w - 6.0, bottom: t + h - 8.0 }),
                if i == lp.sel { txt } else { muted },
                D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
        }
        if max == 0 {
            let no: Vec<u16> = "无匹配应用".encode_utf16().collect();
            let _ = rt.DrawText(&no, f,
                std::ptr::from_ref(&D2D_RECT_F { left: 0.0, top: 300.0, right: lp.sw as f32, bottom: 340.0 }),
                muted, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
        }
    }

    let _ = rt.EndDraw(None, None);
    let _ = lp.sc.Present(0, DXGI_PRESENT(0));
}
