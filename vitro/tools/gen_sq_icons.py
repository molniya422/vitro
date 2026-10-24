#!/usr/bin/env python3
"""SQ 图标库再生成管线(vitro/assets/icons/*.png 的来源)

把 docs/prototypes/vitro-nexus-full.html 里的 SQ SVG 图标库提取出来,
用无头 Chromium 栅格化成 96px RGBA PNG,供 include_bytes! 内嵌。

用法:
  python tools/gen_sq_icons.py <vitro-nexus-full.html 或已提取的 _nx.js> <输出目录>
依赖:Chromium/Edge 可执行(自动探测)、Pillow
"""
import json
import os
import re
import subprocess
import sys

CHROME_CANDIDATES = [
    r"D:\playwright-browsers\chromium-1234\chrome-win64\chrome.exe",
    r"C:\Program Files\Google\Chrome\Application\chrome.exe",
    r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
]

# 与 HTML dock 中 sq(g, body) 包装完全一致(渐变 squircle + 顶缘高光 + 1px 描边)
SQ_WRAP = (
    '<defs><linearGradient id="{id}" x1="0" y1="0" x2="0" y2="1">'
    '<stop offset="0" stop-color="{a}"/><stop offset="1" stop-color="{b}"/></linearGradient></defs>'
    '<rect x="2" y="2" width="60" height="60" rx="15" fill="url(#{id})"/>'
    '<path d="M2 21 C2 9 9 2 21 2 H43 C55 2 62 9 62 21 V31 H2 Z" fill="#fff" opacity=".15"/>'
    '<rect x="2.5" y="2.5" width="59" height="59" rx="14.5" fill="none" stroke="rgba(255,255,255,.30)"/>{body}'
)


def extract_sq_entries(src: str) -> dict:
    m = re.search(r'var SQ=\{(.*?)\n  \};', src, re.S)
    if not m:
        raise SystemExit("SQ 库未找到")
    body = m.group(1)
    icons = {}
    for mm in re.finditer(r"(\w+):\{((?:raw|g):.*?)\}(?=,\n|\n)", body):
        name, inner = mm.group(1), mm.group(2)
        if inner.startswith("raw:"):
            icons[name] = inner[4:].strip().strip("'\"")
            continue
        gm = re.match(r"g:\{id:'(\w+)',a:'([^']+)',b:'([^']+)'\},b:'(.*)'\s*$", inner, re.S)
        if gm:
            gid, a, b, bod = gm.groups()
            icons[name] = SQ_WRAP.format(id=gid, a=a, b=b, body=bod)
    return icons


def main():
    src_path, out_dir = sys.argv[1], sys.argv[2]
    os.makedirs(out_dir, exist_ok=True)
    src = open(src_path, encoding="utf-8").read()
    icons = extract_sq_entries(src)
    print(f"提取 {len(icons)} 个 SQ 图标")

    cell, cols = 96, 7
    rows = (len(icons) + cols - 1) // cols
    grid = ['<!doctype html><html><body style="margin:0;background:transparent">']
    for i, (name, body) in enumerate(sorted(icons.items())):
        x, y = (i % cols) * cell, (i // cols) * cell
        svg = f'<svg viewBox="0 0 64 64" width="{cell}" height="{cell}">{body}</svg>'
        grid.append(f'<div style="position:absolute;left:{x}px;top:{y}px;width:{cell}px;height:{cell}px">{svg}</div>')
    grid.append("</body></html>")

    workdir = os.path.dirname(os.path.abspath(src_path))
    grid_html = os.path.join(workdir, "_sqgrid.html")
    open(grid_html, "w", encoding="utf-8").write("\n".join(grid))

    chrome = next((c for c in CHROME_CANDIDATES if os.path.exists(c)), None)
    if not chrome:
        raise SystemExit("未找到 Chromium/Edge,请把路径加进 CHROME_CANDIDATES")
    shot = os.path.join(workdir, "_sqgrid.png")
    subprocess.run([chrome, "--headless", "--disable-gpu",
                    f"--screenshot={shot}", f"--window-size={cols*cell},{rows*cell}",
                    "--default-background-color=00000000", "--hide-scrollbars",
                    "file:///" + grid_html.replace("\\", "/")], check=True)

    from PIL import Image
    img = Image.open(shot).convert("RGBA")
    names = sorted(icons.keys())
    saved = 0
    for i, name in enumerate(names):
        x, y = (i % cols) * cell, (i // cols) * cell
        cellimg = img.crop((x, y, x + cell, y + cell))
        if cellimg.getextrema()[3] == (0, 0):
            print(f"空格子,跳过: {name}")
            continue
        cellimg.save(os.path.join(out_dir, f"{name}.png"))
        saved += 1
    print(f"已写入 {saved} 个 PNG → {out_dir}")


if __name__ == "__main__":
    main()
