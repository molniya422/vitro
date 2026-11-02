// tray.rs — tray integration. Popup selection is applied after the nested
// TrackPopupMenu loop has returned, without holding a runtime borrow.
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::runtime::AppRuntime;
use crate::state::WM_APP_TRAY;

unsafe fn nid(hwnd: HWND) -> NOTIFYICONDATAW {
    let mut tip = [0u16; 128];
    for (d, s) in tip.iter_mut().zip("Vitro Dock".encode_utf16()) { *d = s; }
    NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: 1,
        uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
        uCallbackMessage: WM_APP_TRAY,
        hIcon: LoadIconW(None, IDI_APPLICATION).unwrap_or_default(),
        szTip: tip,
        ..Default::default()
    }
}

pub(crate) unsafe fn init(hwnd: HWND) {
    let _ = Shell_NotifyIconW(NIM_ADD, &nid(hwnd));
}

pub(crate) unsafe fn remove(hwnd: HWND) {
    let _ = Shell_NotifyIconW(NIM_DELETE, &nid(hwnd));
}

pub(crate) unsafe fn on_tray_menu(runtime: *mut AppRuntime) -> Result<()> {
    // Copy everything needed to construct and display the menu. TrackPopupMenu
    // runs a nested message loop, so no runtime/state reference may span it.
    let (hwnd, autohide) = {
        let state = &(*runtime).state;
        (state.hwnd, (*runtime).behavior.autohide)
    };
    let menu = CreatePopupMenu()?;
    let _ = AppendMenuW(menu, MF_STRING, 10, w!("显示 Dock(自动隐藏时移到屏幕底边)"));
    let chk = if autohide { MF_STRING | MF_CHECKED } else { MF_STRING };
    let _ = AppendMenuW(menu, chk, 11, w!("自动隐藏(底边召唤)"));
    let _ = AppendMenuW(menu, MF_SEPARATOR, 0, None);
    let _ = AppendMenuW(menu, MF_STRING, 12, w!("启动器…  Ctrl+Alt+D"));
    let _ = AppendMenuW(menu, MF_STRING, 13, w!("重载主题  Ctrl+Alt+R"));
    let _ = AppendMenuW(menu, MF_SEPARATOR, 0, None);
    let _ = AppendMenuW(menu, MF_STRING, 14, w!("退出 Vitro"));
    let _ = SetForegroundWindow(hwnd);
    let mut pt = POINT::default();
    let _ = GetCursorPos(&mut pt);
    let cmd = TrackPopupMenu(menu, TPM_RIGHTBUTTON | TPM_RETURNCMD, pt.x, pt.y, 0, hwnd, None);
    let _ = DestroyMenu(menu);
    if (*runtime).is_shutting_down() { return Ok(()); }

    match cmd.0 {
        10 => crate::dockwin::show_dock(runtime),
        11 => crate::dockwin::toggle_autohide(runtime),
        12 => crate::lpad::toggle(runtime),
        13 => crate::dockwin::reload_theme(runtime)?,
        14 => crate::dockwin::shutdown(runtime, hwnd),
        _ => {}
    }
    Ok(())
}
