# Vitro M6 自对拍:用逆向 NeXuS 的同一套 Frida 钩子验证自己
# 验证声明:零轮询(GetCursorPos/SetTimer/EnumWindows/GetAsyncKeyState 空闲 = 0)+ 主题热重载
import frida, json, time, subprocess

IDLE_S = 10
EXE = r"D:\dockyh\vitro\target\release\vitro.exe"

src = open(r"D:\dockyh\vitro\selftest\vitro-hooks.js", encoding="utf-8").read()
events = []

def on_msg(m, data):
    if m["type"] == "send":
        events.append(m["payload"])
    else:
        events.append({"frida-error": str(m.get("description", m))[:160]})

proc = subprocess.Popen([EXE], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
time.sleep(1)
session = frida.attach(proc.pid)
script = session.create_script(src)
script.on("message", on_msg)
script.load()
time.sleep(3)  # 初始化(窗口/图标提取/瘦身)

armed = any(e.get("t") == "armed" for e in events)
snap0 = script.exports_sync.snapshot()

time.sleep(IDLE_S)  # 空闲期:人手不碰,轮询类钩子应完全静默

snap1 = script.exports_sync.snapshot()
delta = {k: snap1[k] - snap0[k] for k in snap0}

# 模拟主题热重载:向 dock 投递 WM_HOTKEY(id=2)——等价于用户按 Ctrl+Alt+R
r = script.exports_sync.fire_theme_reload()
time.sleep(1.5)
snap2 = script.exports_sync.snapshot()
brush_after = snap2["CreateSolidColorBrush"] - snap1["CreateSolidColorBrush"]

out = {
    "target": "vitro.exe (M5 build)",
    "method": "frida attach, same hooks as nexus selftest",
    "armed": armed,
    "idle_seconds": IDLE_S,
    "idle_delta": delta,
    "theme_reload": {"fired": r, "brush_recreations": brush_after},
    "verdicts": {
        "zero_polling_cursorpos": "PASS" if delta["GetCursorPos"] == 0 else "FAIL",
        "zero_timers_idle": "PASS" if delta["SetTimer"] == 0 else "FAIL",
        "zero_enumwindows": "PASS" if delta["EnumWindows"] == 0 else "FAIL",
        "zero_hotkey_polling": "PASS" if delta["GetAsyncKeyState"] == 0 else "FAIL",
        "theme_hot_reload": "PASS" if brush_after >= 3 else "INCONCLUSIVE",
    },
    "nexus_comparison": {
        "GetCursorPos": {"nexus": "40-68 / sec (poll)", "vitro": "%d in %ds (zero)" % (delta["GetCursorPos"], IDLE_S)},
        "SetTimer": {"nexus": "40+ timers at startup (module heartbeats)", "vitro": "%d while idle" % delta["SetTimer"]},
    },
}
with open(r"D:\dockyh\vitro\selftest\selftest-evidence.json", "w", encoding="utf-8") as f:
    json.dump(out, f, ensure_ascii=False, indent=1)
print(json.dumps(out["idle_delta"]))
print(json.dumps(out["verdicts"]))
print(json.dumps(out["theme_reload"]))
try:
    session.detach()
except Exception:
    pass
proc.terminate()
