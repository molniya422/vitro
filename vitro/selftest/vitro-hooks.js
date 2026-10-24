// vitro-selftest hooks — 用逆向 NeXuS 的同一套钩子验证 Vitro 自己
// 验证目标:零轮询(GetCursorPos/SetTimer/EnumWindows/GetAsyncKeyState = 0)+ 主题热重载路径
let counts = {
    GetCursorPos: 0, SetTimer: 0, EnumWindows: 0, GetAsyncKeyState: 0,
    GetKeyState: 0, PostMessageW: 0, GetMessageW: 0, GetProcessTimes: 0,
    CreateSolidColorBrush: 0, SHGetFileInfoW: 0, ShellExecuteExW: 0,
};
let hwndDock = null;

function hookCount(n) {
    try {
        const p = Module.getGlobalExportByName(n);
        if (p) Interceptor.attach(p, { onEnter() { counts[n]++; } });
    } catch (e) {}
}

["GetCursorPos", "SetTimer", "EnumWindows", "GetAsyncKeyState", "GetKeyState",
 "PostMessageW", "GetMessageW", "GetProcessTimes", "CreateSolidColorBrush",
 "SHGetFileInfoW", "ShellExecuteExW"].forEach(hookCount);

// 捕获 dock 窗口句柄(CreateWindowExW 返回后)
try {
    const p = Module.getGlobalExportByName("CreateWindowExW");
    Interceptor.attach(p, {
        onLeave(ret) {
            if (!hwndDock) {
                // 第一个创建的顶层窗 = VitroDock(初始化顺序)
                hwndDock = ret.returnValue;
                send({ t: "hwnd", hwnd: hwndDock.toString() });
            }
        }
    });
} catch (e) {}

send({ t: "armed", counts: counts });

// 供 Python 通过 rpc 模拟 Ctrl+Alt+R 热键(WM_HOTKEY, id=2)
rpc.exports = {
    fireThemeReload: function () {
        if (!hwndDock) {
            const fw = new NativeFunction(
                Module.getGlobalExportByName("FindWindowW"),
                "pointer", ["pointer", "pointer"]);
            const cls = Memory.allocUtf16String("VitroDock");
            hwndDock = fw(cls, ptr(0));
        }
        if (!hwndDock || hwndDock.isNull()) return "no hwnd";
        const pm = new NativeFunction(
            Module.getGlobalExportByName("PostMessageW"),
            "int", ["pointer", "uint", "uintptr", "long"]);
        pm(hwndDock, 0x0312, 2, 0); // WM_HOTKEY, id=2
        return "posted";
    },
    snapshot: function () { return counts; },
    hwnd: function () { return hwndDock ? hwndDock.toString() : null; }
};
