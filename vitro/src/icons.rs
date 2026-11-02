// icons.rs — 图标资源管线
// 应用:Shell 三档提取(IShellItemImageFactory@96 → SHGetFileInfoW@32 → SQ 兜底)
// 插件/内置:SQ 图标库(从 vitro-nexus-full.html 的 SQ SVG 库以 Chromium 无头栅格化,
//           96px RGBA PNG ×25 共 94KB,include_bytes 内嵌,运行时零文件依赖)
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;

unsafe fn hbitmap_to_bgra(hb: HBITMAP, w: i32, h: i32) -> Option<Vec<u8>> {
    let hdc = GetDC(None);
    let mut bmi = BITMAPINFO::default();
    bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
    bmi.bmiHeader.biWidth = w;
    bmi.bmiHeader.biHeight = -h;
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = BI_RGB.0;
    let mut pixels = vec![0u8; (w * h * 4) as usize];
    let got = GetDIBits(hdc, hb, 0, h as u32,
        Some(pixels.as_mut_ptr() as *mut std::ffi::c_void), &mut bmi, DIB_RGB_COLORS);
    let _ = ReleaseDC(None, hdc);
    if got == 0 { None } else { Some(pixels) }
}

unsafe fn bgra_to_bitmap(dc: &ID2D1DeviceContext, pixels: &[u8], w: u32, h: u32) -> Option<ID2D1Bitmap1> {
    let props = D2D1_BITMAP_PROPERTIES1 {
        pixelFormat: D2D1_PIXEL_FORMAT { format: DXGI_FORMAT_B8G8R8A8_UNORM, alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED },
        dpiX: 96.0, dpiY: 96.0,
        bitmapOptions: D2D1_BITMAP_OPTIONS_NONE,
        colorContext: std::mem::ManuallyDrop::new(None),
    };
    dc.CreateBitmap(D2D_SIZE_U { width: w, height: h },
        Some(pixels.as_ptr() as *const std::ffi::c_void),
        w * 4, std::ptr::from_ref(&props)).ok()
}

pub(crate) unsafe fn extract_icon_96(dc: &ID2D1DeviceContext, path: &str) -> Option<ID2D1Bitmap1> {
    let wpath: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
    // 一档:IShellItemImageFactory @96(lnk/UWP/任意 shell 项)
    if let Ok(item) = SHCreateItemFromParsingName::<_, _, IShellItem>(PCWSTR(wpath.as_ptr()), None) {
        if let Ok(f) = item.cast::<IShellItemImageFactory>() {
            if let Ok(hb) = f.GetImage(SIZE { cx: 96, cy: 96 }, SIIGBF_ICONONLY) {
                let px = hbitmap_to_bgra(hb, 96, 96);
                let _ = DeleteObject(hb);
                if let Some(px) = px {
                    if let Some(b) = bgra_to_bitmap(dc, &px, 96, 96) { return Some(b); }
                }
            }
        }
    }
    // 二档:SHGetFileInfoW @32
    let mut sfi = SHFILEINFOW::default();
    let hr = SHGetFileInfoW(PCWSTR(wpath.as_ptr()), FILE_FLAGS_AND_ATTRIBUTES(0),
        Some(&mut sfi), std::mem::size_of::<SHFILEINFOW>() as u32, SHGFI_ICON | SHGFI_LARGEICON);
    if hr == 0 || sfi.hIcon.is_invalid() { return None; }
    let mut info = ICONINFO::default();
    if GetIconInfo(sfi.hIcon, &mut info).is_err() { let _ = DestroyIcon(sfi.hIcon); return None; }
    let px = hbitmap_to_bgra(info.hbmColor, 32, 32);
    let _ = DeleteObject(info.hbmColor);
    let _ = DeleteObject(info.hbmMask);
    let _ = DestroyIcon(sfi.hIcon);
    px.and_then(|px| bgra_to_bitmap(dc, &px, 32, 32))
}

// SQ 图标库(vitro-nexus-full.html 视觉资产 → 96px PNG)
static SQ_FILES: &[(&str, &[u8])] = &[
    ("battery", include_bytes!("../assets/icons/battery.png")),
    ("browser", include_bytes!("../assets/icons/browser.png")),
    ("calendar", include_bytes!("../assets/icons/calendar.png")),
    ("clip", include_bytes!("../assets/icons/clip.png")),
    ("clock", include_bytes!("../assets/icons/clock.png")),
    ("code", include_bytes!("../assets/icons/code.png")),
    ("control", include_bytes!("../assets/icons/control.png")),
    ("cpu", include_bytes!("../assets/icons/cpu.png")),
    ("doc", include_bytes!("../assets/icons/doc.png")),
    ("drawer", include_bytes!("../assets/icons/drawer.png")),
    ("folder", include_bytes!("../assets/icons/folder.png")),
    ("gamepad", include_bytes!("../assets/icons/gamepad.png")),
    ("mail", include_bytes!("../assets/icons/mail.png")),
    ("menulist", include_bytes!("../assets/icons/menulist.png")),
    ("music", include_bytes!("../assets/icons/music.png")),
    ("net", include_bytes!("../assets/icons/net.png")),
    ("notes", include_bytes!("../assets/icons/notes.png")),
    ("ram", include_bytes!("../assets/icons/ram.png")),
    ("settings", include_bytes!("../assets/icons/settings.png")),
    ("shelfico", include_bytes!("../assets/icons/shelfico.png")),
    ("tasks", include_bytes!("../assets/icons/tasks.png")),
    ("terminal", include_bytes!("../assets/icons/terminal.png")),
    ("trash", include_bytes!("../assets/icons/trash.png")),
    ("tray", include_bytes!("../assets/icons/tray.png")),
    ("weather", include_bytes!("../assets/icons/weather.png")),
];

// 名称 → SQ 键(应用无提取图标 / 插件未指定 icon 时)
pub(crate) fn sq_guess(name: &str) -> Option<&'static str> {
    let n = name.to_lowercase();
    for (k, pats) in [
        ("clock", &["时钟", "clock"][..]), ("weather", &["天气", "weather"]),
        ("cpu", &["cpu"][..]), ("ram", &["内存", "ram"]), ("battery", &["电池", "电源", "battery"]),
        ("net", &["网速", "网络", "net"]), ("clip", &["剪贴", "clip"]), ("calendar", &["日历", "cal"]),
        ("notes", &["笔记", "note"]), ("tasks", &["任务", "task"]), ("mail", &["邮件", "mail"]),
        ("music", &["音乐", "music"]), ("gamepad", &["游戏", "game"]), ("settings", &["设置", "setting"]),
        ("control", &["控制", "control"]), ("terminal", &["终端", "term"]), ("browser", &["浏览器", "brow"]),
        ("code", &["代码", "code"]), ("trash", &["回收", "垃圾", "trash"]), ("drawer", &["抽屉", "drawer"]),
        ("folder", &["文件夹", "folder"]), ("doc", &["文档", "doc"]),
    ] {
        if pats.iter().any(|p| n.contains(p)) { return Some(k); }
    }
    None
}

pub(crate) unsafe fn build_sq_map(dc: &ID2D1DeviceContext) -> std::collections::HashMap<&'static str, ID2D1Bitmap1> {
    let mut m = std::collections::HashMap::new();
    for (k, bytes) in SQ_FILES {
        if let Some(b) = png_to_bitmap(dc, bytes) { m.insert(*k, b); }
        else { println!("[vitro] SQ decode failed: {k}"); }
    }
    m
}

unsafe fn png_to_bitmap(dc: &ID2D1DeviceContext, bytes: &[u8]) -> Option<ID2D1Bitmap1> {
    let dec = png::Decoder::new(std::io::Cursor::new(bytes));
    let mut reader = dec.read_info().ok()?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight { return None; }
    // RGBA 直通 → BGRA 预乘(D2D premultiplied 要求)
    let mut px = Vec::with_capacity(buf.len());
    for p in buf.chunks_exact(4) {
        let a = p[3] as u32;
        px.push((p[2] as u32 * a / 255) as u8);
        px.push((p[1] as u32 * a / 255) as u8);
        px.push((p[0] as u32 * a / 255) as u8);
        px.push(p[3]);
    }
    bgra_to_bitmap(dc, &px, info.width, info.height)
}
