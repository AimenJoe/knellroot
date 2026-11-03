"""앱 아이콘(0022). 산출물(android/res/mipmap-*)을 커밋하고, 이 파일이 원본이다 — PNG를 손으로 고치지 않는다.
    python3 tools/icons/make_icons.py
그림: 어둠 속 호박색 종(knell) 아래로 뿌리(root)가 내려간다. 32×32 픽셀 격자에 코드로 찍고 크기마다 가장 가까운 픽셀로 키운다
(게임의 레트로 픽셀 느낌). 첫 게임 nail tools/icons/make_icons.py의 틀(밀도 표·적응형 XML)을 따랐다.
- 옛 아이콘: mipmap-<밀도>/ic_launcher.png(48~192px, 둥근 네모 바탕 + 그림).
- 적응형(Android 8+): mipmap-anydpi-v26/ic_launcher.xml = 배경(바탕색 한 장) + 앞 그림(108dp 칸의 가운데 66dp 안 — 원으로 깎여도 다 보이게).
Cargo.toml [package.metadata.android]의 res = "android/res", icon = "@mipmap/ic_launcher"."""
import os

from PIL import Image, ImageDraw

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..'))
ANDROID = [('mdpi', 48, 108), ('hdpi', 72, 162), ('xhdpi', 96, 216), ('xxhdpi', 144, 324), ('xxxhdpi', 192, 432)]
ADAPTIVE_XML = """<?xml version="1.0" encoding="utf-8"?>
<!-- 적응형 아이콘(Android 8+, 0022). tools/icons/make_icons.py가 만든다. -->
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
    <background android:drawable="@mipmap/ic_launcher_background" />
    <foreground android:drawable="@mipmap/ic_launcher_foreground" />
</adaptive-icon>
"""
BG = (27, 26, 31)          # charcoal(그림 팔레트와 같은 색)
BG2 = (42, 44, 51)         # slate
AMBER = (224, 154, 58)
EMBER = (242, 196, 109)
DARK_AMBER = (150, 92, 34)
ROOT_C = (110, 74, 92)     # 뿌리(보랏빛 갈색)
ROOT_D = (74, 51, 64)
# 앞 그림이 108dp 칸에서 차지할 크기(dp) — 66dp 안전 원 안에 들게.
ADAPTIVE_ART_DP = 60

# 32×32 그림(문자 = 색). 종: 머리 고리·몸·입술, 아래 추, 그 아래 뿌리가 갈라진다.
ART = [
    "................................",
    "...............ee...............",
    "..............e..e..............",
    "..............e..e..............",
    "...............ee...............",
    "............aaaaaaaa............",
    "...........aeeaaaaaaa...........",
    "..........aeeaaaaaaaad..........",
    "..........aeaaaaaaaaad..........",
    ".........aeeaaaaaaaaaad.........",
    ".........aeaaaaaaaaaaad.........",
    ".........aeaaaaaaaaaaad.........",
    "........aeeaaaaaaaaaaaad........",
    "........aeaaaaaaaaaaaaad........",
    ".......aeeaaaaaaaaaaaaaad.......",
    "......aaaaaaaaaaaaaaaaaaaa......",
    ".....dddddddddddddddddddddd.....",
    "...............ee...............",
    "...............rr...............",
    "...............rr...............",
    "..............rrrr..............",
    ".............rr..rr.............",
    "............rr.rr.rr............",
    "...........rr..rr..rr...........",
    "..........rr..rr....rr..........",
    ".........rr...r......rr.........",
    "........rr...rr.......rr........",
    ".......kr....r.........rk.......",
    "......k.....kr..........k.......",
    ".....k......k............k......",
    "............k...................",
    "................................",
]
COLORS = {'a': AMBER, 'e': EMBER, 'd': DARK_AMBER, 'r': ROOT_C, 'k': ROOT_D}


def art():
    img = Image.new('RGBA', (32, 32), (0, 0, 0, 0))
    for y, row in enumerate(ART):
        assert len(row) == 32, (y, len(row))
        for x, c in enumerate(row):
            if c in COLORS:
                img.putpixel((x, y), COLORS[c] + (255,))
    return img


def background(size):
    """위가 조금 밝은 슬레이트 → 숯(세로 계단 넷 — 픽셀 느낌)."""
    img = Image.new('RGBA', (size, size), BG + (255,))
    d = ImageDraw.Draw(img)
    for i in range(4):
        k = i / 4
        c = tuple(int(BG2[j] + (BG[j] - BG2[j]) * k) for j in range(3))
        d.rectangle([0, size * i // 4, size, size * (i + 1) // 4], fill=c + (255,))
    return img


def legacy(src, size):
    """옛 아이콘: 둥근 네모 바탕 + 그림(가운데 80%)."""
    out = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    mask = Image.new('L', (size, size), 0)
    ImageDraw.Draw(mask).rounded_rectangle([0, 0, size - 1, size - 1], radius=size // 6, fill=255)
    out.paste(background(size), (0, 0), mask)
    n = size * 8 // 10 // 32 * 32 or size * 8 // 10
    a = src.resize((n, n), Image.NEAREST)
    out.alpha_composite(a, ((size - n) // 2, (size - n) // 2))
    return out


def main():
    src = art()
    res = os.path.join(ROOT, 'android', 'res')
    for dens, icon, cell in ANDROID:
        d = os.path.join(res, f'mipmap-{dens}')
        os.makedirs(d, exist_ok=True)
        legacy(src, icon).save(os.path.join(d, 'ic_launcher.png'))
        background(cell).convert('RGB').save(os.path.join(d, 'ic_launcher_background.png'))
        fg = Image.new('RGBA', (cell, cell), (0, 0, 0, 0))
        n = round(cell * ADAPTIVE_ART_DP / 108)
        fg.alpha_composite(src.resize((n, n), Image.NEAREST), ((cell - n) // 2, (cell - n) // 2))
        fg.save(os.path.join(d, 'ic_launcher_foreground.png'))
    d = os.path.join(res, 'mipmap-anydpi-v26')
    os.makedirs(d, exist_ok=True)
    with open(os.path.join(d, 'ic_launcher.xml'), 'w') as f:
        f.write(ADAPTIVE_XML)
    # 미리보기(문서용): 옛 아이콘 192 + 원으로 깎은 적응형.
    prev = Image.new('RGBA', (192 * 2 + 24, 192), (0, 0, 0, 0))
    prev.alpha_composite(legacy(src, 192), (0, 0))
    cell = 432
    full = background(cell)
    n = round(cell * ADAPTIVE_ART_DP / 108)
    full.alpha_composite(src.resize((n, n), Image.NEAREST), ((cell - n) // 2, (cell - n) // 2))
    circ = Image.new('L', (cell, cell), 0)
    ImageDraw.Draw(circ).ellipse([0, 0, cell - 1, cell - 1], fill=255)
    full.putalpha(circ)
    prev.alpha_composite(full.resize((192, 192), Image.LANCZOS), (192 + 24, 0))
    os.makedirs(os.path.join(ROOT, 'docs', 'art'), exist_ok=True)
    prev.save(os.path.join(ROOT, 'docs', 'art', 'app-icon.png'))
    print('icons written')


if __name__ == '__main__':
    main()
