"""Export the Dock mark to PNG sizes and a multi-size .ico.

Pipeline:
  1. render art/mark.svg and art/mark-small.svg at 1024 px through headless Edge
  2. crop to the alpha bounding box, add a 4 percent margin on sizes 48 and up
  3. write icons/icon-<size>.png
  4. pack icons/dock.ico with PNG payloads (Vista and later)

Run from the design-art folder:
  python tools/export-icons.py
"""

from __future__ import annotations

import os
import struct
import subprocess
import sys
import tempfile
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
ART = ROOT / "art"
ICONS = ROOT / "icons"
RENDER = ROOT / "tools" / "render-icon.html"

EDGE_CANDIDATES = [
    r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
    r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
    r"C:\Program Files\Google\Chrome\Application\chrome.exe",
]

LARGE_SIZES = [512, 256, 128, 64, 48]
SMALL_SIZES = [32, 24, 16]
ICO_SIZES = [256, 128, 64, 48, 32, 24, 16]


def find_browser() -> str:
    for path in EDGE_CANDIDATES:
        if os.path.exists(path):
            return path
    raise SystemExit("No Edge or Chrome found. Edit EDGE_CANDIDATES.")


def render(browser: str, source: str, out_png: Path) -> None:
    """Render one SVG at roughly 1024 px with a transparent background."""
    url = RENDER.as_uri() + "?src=" + source
    subprocess.run(
        [
            browser,
            "--headless=new",
            "--disable-gpu",
            "--hide-scrollbars",
            "--default-background-color=00000000",
            "--window-size=1054,1124",
            f"--screenshot={out_png}",
            url,
        ],
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )


def cut_out(master: Path) -> Image.Image:
    """Crop to the drawn area so the frame offset from the browser does not matter."""
    image = Image.open(master).convert("RGBA")
    box = image.getbbox()
    if box is None:
        raise SystemExit(f"Nothing drawn in {master}")
    return image.crop(box)


def export(cut: Image.Image, size: int, out_png: Path) -> None:
    margin = round(size * 0.04) if size >= 48 else 0
    inner = size - margin * 2
    scaled = cut.resize((inner, inner), Image.LANCZOS)
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    canvas.paste(scaled, (margin, margin), scaled)
    canvas.save(out_png, "PNG", optimize=True)


def square(image: Image.Image) -> Image.Image:
    side = max(image.width, image.height)
    canvas = Image.new("RGBA", (side, side), (0, 0, 0, 0))
    canvas.paste(image, ((side - image.width) // 2, (side - image.height) // 2), image)
    return canvas


def write_ico(paths: list[tuple[int, Path]], out_ico: Path) -> None:
    payload = [(size, p.read_bytes()) for size, p in paths]
    header = struct.pack("<HHH", 0, 1, len(payload))
    offset = 6 + 16 * len(payload)
    entries = b""
    blobs = b""
    for size, data in payload:
        dimension = 0 if size >= 256 else size
        entries += struct.pack("<BBBBHHII", dimension, dimension, 0, 0, 1, 32, len(data), offset)
        offset += len(data)
        blobs += data
    out_ico.write_bytes(header + entries + blobs)


def main() -> None:
    browser = find_browser()
    ICONS.mkdir(exist_ok=True)

    with tempfile.TemporaryDirectory() as tmp:
        tmpdir = Path(tmp)
        large_png = tmpdir / "large.png"
        small_png = tmpdir / "small.png"
        render(browser, "mark.svg", large_png)
        render(browser, "mark-small.svg", small_png)

        large = square(cut_out(large_png))
        small = square(cut_out(small_png))

        written: dict[int, Path] = {}
        for size in LARGE_SIZES:
            target = ICONS / f"icon-{size}.png"
            export(large, size, target)
            written[size] = target
        for size in SMALL_SIZES:
            target = ICONS / f"icon-{size}.png"
            export(small, size, target)
            written[size] = target

        write_ico([(size, written[size]) for size in ICO_SIZES], ICONS / "dock.ico")

        # contact sheet: same icon at every shipped size, dark band over light band
        band_h = 300
        sheet = Image.new("RGBA", (1240, band_h * 2 + 48), (11, 14, 21, 255))
        dark = Image.new("RGBA", (1240, band_h), (16, 20, 29, 255))
        light = Image.new("RGBA", (1240, band_h), (244, 246, 251, 255))
        sheet.paste(dark, (0, 0))
        sheet.paste(light, (0, band_h + 48))
        sheet.paste(Image.new("RGBA", (1240, 48), (11, 14, 21, 255)), (0, band_h))

        x = 40
        for size in [256, 128, 64, 48, 32, 24, 16]:
            icon = Image.open(written[size]).convert("RGBA")
            y = band_h // 2 - size // 2
            sheet.paste(icon, (x, y), icon)
            y2 = band_h + 48 + band_h // 2 - size // 2
            sheet.paste(icon, (x, y2), icon)
            x += size + 56

        sheet.save(ICONS / "preview.png", "PNG", optimize=True)

    for size in sorted(written):
        print(f"icons/icon-{size}.png")
    print("icons/dock.ico")
    print("icons/preview.png")


if __name__ == "__main__":
    sys.exit(main())
