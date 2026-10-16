#!/usr/bin/env python3
"""Generate assets/icons/droidkraft.{png,ico,icns} (needs Pillow)."""
import io, struct
from PIL import Image, ImageDraw

def draw(n=1024):
    im = Image.new("RGBA", (n, n), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    d.rounded_rectangle((0, 0, n - 1, n - 1), n // 5, fill=(17, 20, 28, 255))
    g = (61, 220, 132, 255)  # android green
    # phone body
    x0, x1, y0, y1 = n * .30, n * .70, n * .15, n * .85
    d.rounded_rectangle((x0, y0, x1, y1), n * .07, outline=g, width=int(n * .04))
    # screen "bars" (stats)
    bw = n * .05
    for i, h in enumerate((.12, .22, .17, .30)):
        bx = n * .36 + i * n * .075
        d.rounded_rectangle((bx, n * .70 - n * h, bx + bw, n * .70), n * .012, fill=g)
    d.rounded_rectangle((n * .45, n * .20, n * .55, n * .22), n * .01, fill=g)
    return im

im = draw()
im.resize((256, 256), Image.LANCZOS).save("assets/icons/droidkraft.png")
im.save("assets/icons/droidkraft.ico", sizes=[(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
# .icns: PNG-compressed entries
entries = []
for tag, px in ((b"icp4", 16), (b"icp5", 32), (b"icp6", 64), (b"ic07", 128), (b"ic08", 256), (b"ic09", 512), (b"ic10", 1024)):
    b = io.BytesIO(); im.resize((px, px), Image.LANCZOS).save(b, "PNG")
    entries.append(tag + struct.pack(">I", len(b.getvalue()) + 8) + b.getvalue())
body = b"".join(entries)
open("assets/icons/droidkraft.icns", "wb").write(b"icns" + struct.pack(">I", len(body) + 8) + body)
print("ok")
