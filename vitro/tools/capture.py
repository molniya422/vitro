# capture.py — PrintWindow(PW_RENDERFULLCONTENT) 抓取 Vitro Dock 的 DComp 渲染内容
# 用法: python capture.py [输出.png]  (CopyFromScreen/BitBlt(SRCCOPY) 抓不到 DComp 内容,必须 PrintWindow)
import ctypes, sys
from ctypes import wintypes

u = ctypes.windll.user32
g = ctypes.windll.g32 if False else ctypes.windll.gdi32
PW_RENDERFULLCONTENT = 2

def find_vitro():
    hwnd = u.FindWindowW(None, "Vitro Dock")
    if not hwnd:  # 兜底:枚举窗口找类名/标题前缀
        found = []
        @ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)
        def cb(h, _):
            buf = ctypes.create_unicode_buffer(64)
            u.GetWindowTextW(h, buf, 64)
            if buf.value.startswith("Vitro"): found.append((h, buf.value))
            return True
        u.EnumWindows(cb, 0)
        if not found: raise SystemExit("Vitro Dock 窗口未找到(进程在跑吗?)")
        return found[0][0]
    return hwnd

def grab(hwnd, out):
    rect = wintypes.RECT()
    u.GetClientRect(hwnd, ctypes.byref(rect))
    w, h = rect.right, rect.bottom
    hdc = u.GetWindowDC(hwnd)
    mdc = g.CreateCompatibleDC(hdc)
    bmp = g.CreateCompatibleBitmap(hdc, w, h)
    old = g.SelectObject(mdc, bmp)
    ok = u.PrintWindow(hwnd, mdc, PW_RENDERFULLCONTENT)
    class BMIH(ctypes.Structure):
        _fields_ = [("sz", wintypes.DWORD), ("w", ctypes.c_long), ("h", ctypes.c_long),
                    ("planes", wintypes.WORD), ("bits", wintypes.WORD), ("compress", wintypes.DWORD),
                    ("szimg", wintypes.DWORD), ("xppm", ctypes.c_long), ("yppm", ctypes.c_long),
                    ("used", wintypes.DWORD), ("imp", wintypes.DWORD)]
    bi = BMIH(ctypes.sizeof(BMIH), w, -h, 1, 32, 0, 0, 0, 0, 0, 0)
    buf = ctypes.create_string_buffer(w * h * 4)
    g.GetDIBits(mdc, bmp, 0, h, buf, ctypes.byref(bi), 0)
    g.SelectObject(mdc, old); g.DeleteObject(bmp); g.DeleteDC(mdc); u.ReleaseDC(hwnd, hdc)
    if not ok: print("[warn] PrintWindow 返回 FALSE")
    from PIL import Image
    img = Image.frombuffer("RGBA", (w, h), buf.raw, "raw", "BGRA", 0, 1)
    # 合成到深色底(用户桌面壁纸未知,深色底最接近实际观感)
    bg = Image.new("RGBA", (w, h), (13, 16, 28, 255))
    bg.alpha_composite(img)
    bg.convert("RGB").save(out)
    print(f"saved {out}  ({w}x{h})")

if __name__ == "__main__":
    out = sys.argv[1] if len(sys.argv) > 1 else "vitro-capture.png"
    grab(find_vitro(), out)
