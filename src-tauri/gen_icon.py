# -*- coding: utf-8 -*-
"""生成 Tauri 应用图标：圆角红底 + 木质棋子 + 楷体「帥」"""
import struct
import zlib
from PIL import Image, ImageDraw, ImageFont
from pathlib import Path

OUT = Path(__file__).parent / "icons"
OUT.mkdir(exist_ok=True)

SIZE = 1024

def rounded_mask(size, radius):
    m = Image.new("L", (size, size), 0)
    d = ImageDraw.Draw(m)
    d.rounded_rectangle([0, 0, size - 1, size - 1], radius=radius, fill=255)
    return m

def build_icon():
    img = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))

    # 背景圆角方块：暗红渐变
    bg = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    d = ImageDraw.Draw(bg)
    top = (216, 86, 60)
    bottom = (146, 44, 30)
    for y in range(SIZE):
        t = y / SIZE
        r = int(top[0] + (bottom[0] - top[0]) * t)
        g = int(top[1] + (bottom[1] - top[1]) * t)
        b = int(top[2] + (bottom[2] - top[2]) * t)
        d.line([(0, y), (SIZE, y)], fill=(r, g, b, 255))

    # 微妙的对角光晕
    glow = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    gd = ImageDraw.Draw(glow)
    gd.ellipse([-SIZE * 0.25, -SIZE * 0.3, SIZE * 0.8, SIZE * 0.55], fill=(255, 255, 255, 26))
    bg = Image.alpha_composite(bg, glow)

    img.paste(bg, (0, 0), rounded_mask(SIZE, int(SIZE * 0.22)))

    # 木质棋子圆盘
    cx = cy = SIZE / 2
    R = SIZE * 0.335
    disc = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    dd = ImageDraw.Draw(disc)
    # 外圈深红环
    dd.ellipse([cx - R, cy - R, cx + R, cy + R], fill=(184, 69, 47, 255))
    R2 = R * 0.9
    # 内盘木色
    dd.ellipse([cx - R2, cy - R2, cx + R2, cy + R2], fill=(244, 230, 200, 255))
    # 上部高光
    hl = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    hd = ImageDraw.Draw(hl)
    hd.ellipse([cx - R2 * 0.8, cy - R2 * 1.05, cx + R2 * 0.2, cy - R2 * 0.05], fill=(255, 255, 255, 40))
    disc = Image.alpha_composite(disc, hl)
    # 底部阴影圈
    dd.ellipse([cx - R2, cy + R2 * 0.55, cx + R2, cy + R2 * 1.25], outline=(0, 0, 0, 0))

    img = Image.alpha_composite(img, disc)

    # 环上刻度装饰（内圈细线）
    dec = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    kd = ImageDraw.Draw(dec)
    R3 = R2 * 0.86
    kd.ellipse([cx - R3, cy - R3, cx + R3, cy + R3], outline=(184, 69, 47, 90), width=int(SIZE * 0.006))
    img = Image.alpha_composite(img, dec)

    # 主字「帥」
    font = ImageFont.truetype("C:/Windows/Fonts/simkai.ttf", int(SIZE * 0.42))
    text = "帥"
    bbox = ImageDraw.Draw(img).textbbox((0, 0), text, font=font)
    tw, th = bbox[2] - bbox[0], bbox[3] - bbox[1]
    tx = cx - tw / 2 - bbox[0]
    ty = cy - th / 2 - bbox[1] * 1.0
    # 文字阴影
    sh = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    sd = ImageDraw.Draw(sh)
    sd.text((tx + SIZE * 0.006, ty + SIZE * 0.008), text, font=font, fill=(120, 40, 25, 140))
    img = Image.alpha_composite(img, sh)
    td = ImageDraw.Draw(img)
    td.text((tx, ty), text, font=font, fill=(178, 55, 36, 255))

    return img

icon = build_icon()

# 输出 PNG 尺寸
for name, s in [("32x32.png", 32), ("128x128.png", 128), ("128x128@2x.png", 256), ("icon.png", 512), ("Square30x30Logo.png", 30), ("Square44x44Logo.png", 44), ("Square150x150Logo.png", 150), ("Square310x310Logo.png", 310), ("StoreLogo.png", 50)]:
    icon.resize((s, s), Image.LANCZOS).save(OUT / name)

# ICO（含多尺寸）
icon.save(OUT / "icon.ico", format="ICO", sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])

# ICNS（嵌入 PNG 的最简实现）
def make_icns(img, path):
    entries = []
    for type_, s in [(b"icp4", 16), (b"icp5", 32), (b"ic07", 128), (b"ic08", 256), (b"ic09", 512)]:
        buf = __import__("io").BytesIO()
        img.resize((s, s), Image.LANCZOS).save(buf, "PNG")
        data = buf.getvalue()
        entries.append(type_ + struct.pack(">I", len(data) + 8) + data)
    body = b"".join(entries)
    with open(path, "wb") as f:
        f.write(b"icns" + struct.pack(">I", len(body) + 8) + body)

make_icns(icon, OUT / "icon.icns")
print("icons OK:", sorted(p.name for p in OUT.iterdir()))
