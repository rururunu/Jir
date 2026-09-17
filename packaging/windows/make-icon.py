#!/usr/bin/env python3
"""Render assets/jir.ico from the wordmark.

The wordmark is drawn entirely with axis-aligned blocks, so the artwork is read
back as plain rectangles and redrawn at each icon size. Nothing is scaled down
from a master bitmap, which keeps the block edges hard at 16px as well as 256px.

The icon is the whole wordmark. It is 3.5:1, so in a square canvas it sits in a
band across the middle and its 14-unit strokes are the thinnest thing that has to
survive the smallest size; `--preview` is there for judging that.

    python packaging/windows/make-icon.py [--preview out.png]
"""

from __future__ import annotations

import argparse
import io
import re
import struct
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[2]
SVG = ROOT / "assets" / "wordmark.svg"
ICO = ROOT / "assets" / "jir.ico"

BRAND = (206, 32, 41, 255)
SIZES = (16, 20, 24, 32, 40, 48, 64, 128, 256)

# Fraction of the canvas kept clear so the mark never touches the edge.
PADDING = 0.08

# Every subpath is `M<x> <y>h<w>v<h>h<-w>z` — one block. Artwork of any other
# shape means the parser below has to grow, so it refuses rather than guessing.
BLOCK = re.compile(r"M(-?\d+)[ ,](-?\d+)h(-?\d+)v(-?\d+)h(-?\d+)z")


def blocks(svg: str) -> list[tuple[int, int, int, int]]:
    """The wordmark as `(x, y, width, height)` rectangles."""
    found: list[tuple[int, int, int, int]] = []
    for path in re.findall(r'<path[^>]*?\sd="([^"]+)"', svg):
        rest = BLOCK.sub("", path).strip()
        if rest:
            raise SystemExit(f"unsupported path data in wordmark: {rest[:120]}")
        for x, y, w, h, back in BLOCK.findall(path):
            if int(back) != -int(w):
                raise SystemExit(f"subpath is not a rectangle: M{x} {y}h{w}v{h}h{back}z")
            found.append((int(x), int(y), int(w), int(h)))
    if not found:
        raise SystemExit(f"no blocks found in {SVG}")
    return found


def encodings(path: Path) -> list[tuple[int, str]]:
    """`(size, "DIB" | "PNG")` for each frame, in directory order."""
    data = path.read_bytes()
    _, kind, count = struct.unpack_from("<HHH", data, 0)
    if kind != 1:
        raise SystemExit(f"{path} is not an icon file")
    found = []
    for index in range(count):
        width, _, _, _, _, _, length, offset = struct.unpack_from(
            "<BBBBHHII", data, 6 + index * 16
        )
        fmt = "PNG" if data[offset : offset + 4] == b"\x89PNG" else "DIB"
        found.append((width or 256, fmt))
    return found


def write_ico(path: Path, frames: dict[int, Image.Image]) -> None:
    """Assemble the container: DIB for the small sizes, PNG for 256.

    Pillow picks one encoding for the whole file, and neither choice alone is
    right. DIB everywhere satisfies GDI+ but costs ~360 KB, because a DIB frame is
    never compressed. PNG everywhere is 2.8 KB but GDI+ cannot read a
    PNG-compressed frame below 256px — `new Icon(path, 32, 32).ToBitmap()` throws.
    So each frame is encoded on its own and the directory is written here.
    """
    encoded: list[tuple[int, bytes]] = []
    for size in sorted(frames):
        buffer = io.BytesIO()
        if size >= 256:
            frames[size].save(buffer, format="PNG")
            encoded.append((size, buffer.getvalue()))
            continue
        # Let Pillow build the DIB — it writes the AND mask too — by way of a
        # one-frame icon, then keep only the frame bytes.
        frames[size].save(buffer, format="ICO", sizes=[(size, size)], bitmap_format="bmp")
        data = buffer.getvalue()
        length = struct.unpack_from("<I", data, 14)[0]
        offset = struct.unpack_from("<I", data, 18)[0]
        encoded.append((size, data[offset : offset + length]))

    directory = b""
    offset = 6 + 16 * len(encoded)
    blobs = b""
    for size, blob in encoded:
        directory += struct.pack("<BBBBHHII", size % 256, size % 256, 0, 0, 1, 32, len(blob), offset)
        offset += len(blob)
        blobs += blob

    path.write_bytes(struct.pack("<HHH", 0, 1, len(encoded)) + directory + blobs)


def render(art: list[tuple[int, int, int, int]], size: int) -> Image.Image:
    box = (
        min(b[0] for b in art),
        min(b[1] for b in art),
        max(b[0] + b[2] for b in art),
        max(b[1] + b[3] for b in art),
    )
    width, height = box[2] - box[0], box[3] - box[1]

    inner = size * (1 - 2 * PADDING)
    scale = min(inner / width, inner / height)
    offset_x = (size - width * scale) / 2
    offset_y = (size - height * scale) / 2

    image = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(image)
    for x, y, w, h in art:
        left = offset_x + (x - box[0]) * scale
        top = offset_y + (y - box[1]) * scale
        # `- 1` on the far edge (Pillow fills both ends, so touching blocks would
        # otherwise overlap and fatten the mark), then clamp to one pixel: the
        # 14-unit strokes of the letterforms are far under a pixel at 16x16 and
        # would otherwise come out empty — or inverted, which Pillow rejects.
        x0, y0 = round(left), round(top)
        x1 = max(x0, round(left + w * scale) - 1)
        y1 = max(y0, round(top + h * scale) - 1)
        draw.rectangle([x0, y0, x1, y1], fill=BRAND)
    return image


def verify(path: Path) -> None:
    """Read the icon back. Frame list, encoding and fill colour are the three
    things that go wrong quietly."""
    formats = encodings(path)
    print(f"  frames: {[size for size, _ in formats]}")
    print(f"  encoding: {', '.join(f'{size}={fmt}' for size, fmt in formats)}")
    early_png = [size for size, fmt in formats if fmt == "PNG" and size < 256]
    if early_png:
        raise SystemExit(
            f"frames {early_png} were written as PNG; GDI+ cannot read a "
            "PNG-compressed frame below 256px"
        )
    with Image.open(path) as icon:
        for size in (16, 32, 256):
            icon.size = (size, size)
            frame = icon.convert("RGBA")
            brand = sum(n for n, rgba in frame.getcolors(size * size) if rgba == BRAND)
            print(f"  {size}x{size}: {brand} px of {size * size} in brand red")


def preview(frames: dict[int, Image.Image], path: Path) -> None:
    """A sheet of every size on white and on dark, for judging the icon."""
    gap = 12
    row = max(SIZES) + gap * 2
    sheet = Image.new("RGBA", (sum(SIZES) + gap * (len(SIZES) + 1), row * 2), (255, 255, 255, 255))
    ImageDraw.Draw(sheet).rectangle([0, row, sheet.width, row * 2], fill=(32, 32, 32, 255))

    x = gap
    for size in SIZES:
        for band in (0, 1):
            sheet.alpha_composite(frames[size], (x, band * row + (row - size) // 2))
        x += size + gap
    sheet.save(path)
    print(f"preview: {path}")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--preview", type=Path, help="also write a PNG sheet of every size")
    args = parser.parse_args()

    art = blocks(SVG.read_text(encoding="utf-8"))
    frames = {size: render(art, size) for size in SIZES}
    largest = max(SIZES)

    # Each size is drawn at its own pixel dimensions, so nothing is resampled
    # from a master bitmap.
    write_ico(ICO, frames)

    box_w = max(b[0] + b[2] for b in art) - min(b[0] for b in art)
    box_h = max(b[1] + b[3] for b in art) - min(b[1] for b in art)
    print(f"{ICO.relative_to(ROOT)}: {len(art)} blocks, art {box_w}x{box_h}, sizes {list(SIZES)}")
    print(f"  {ICO.stat().st_size} bytes")
    verify(ICO)

    if args.preview:
        preview(frames, args.preview)


if __name__ == "__main__":
    main()
