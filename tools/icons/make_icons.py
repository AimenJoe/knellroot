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

# 32×32 그림 — 문자는 모양만(h 종 고리 · b 종 몸 · l 입술 · c 추 · r 뿌리). 음영·외곽선은 `art()`가 모양에서 계산한다:
# 빛은 왼쪽 위에서 — 줄마다 그 덩어리의 왼쪽 끝은 밝게, 오른쪽은 그늘, 입술은 한 단 어둡게. 둘레에 1px 외곽선.
SHAPE = [
    "",
    "               hh",
    "              h  h",
    "              h  h",
    "               hh",
    "            bbbbbbbb",
    "           bbbbbbbbbb",
    "          bbbbbbbbbbbb",
    "          bbbbbbbbbbbb",
    "         bbbbbbbbbbbbbb",
    "         bbbbbbbbbbbbbb",
    "         bbbbbbbbbbbbbb",
    "        bbbbbbbbbbbbbbbb",
    "        bbbbbbbbbbbbbbbb",
    "       bbbbbbbbbbbbbbbbbb",
    "      llllllllllllllllllll",
    "      llllllllllllllllllll",
    "              cccc",
    "               rr",
    "               rr",
    "              rrrr",
    "             rr  rr",
    "            rr rr rr",
    "           rr  rr  rr",
    "          rr   rr   rr",
    "         rr    r     rr",
    "        rr    rr      rr",
    "       rr     r        rr",
    "      r      rr         r",
    "     r       r           r",
    "",
    "",
]
ART = [row.ljust(32) for row in SHAPE]
OUTLINE = (12, 10, 14)
HI = (255, 222, 150)        # 종의 가장 밝은 빛
MID_HI = (238, 178, 82)
SHADE = (184, 116, 42)
ROOT_L = (150, 104, 124)    # 뿌리 빛 쪽


def lerp(c0, c1, t):
    return tuple(int(c0[i] + (c1[i] - c0[i]) * t) for i in range(3))


def shade_of(ch, x, x0, x1, y, rows):
    """그 줄에서 이 칸이 속한 덩어리의 [x0, x1] 안 자리로 색을 고른다."""
    n = max(x1 - x0, 1)
    t = (x - x0) / n
    if ch == 'b':
        top = rows.index(y) / max(len(rows) - 1, 1)  # 위가 조금 밝다
        if x == x0 or (t < 0.16 and top < 0.6):
            return HI if t < 0.08 else EMBER
        if t < 0.3:
            return MID_HI
        if t > 0.8:
            return DARK_AMBER
        if t > 0.62:
            return SHADE
        return lerp(AMBER, MID_HI, 0.25 * (1 - top))
    if ch == 'l':
        last = y == rows[-1]
        if t < 0.12:
            return SHADE if last else AMBER
        return lerp(DARK_AMBER, (96, 56, 22), 0.6) if last else (t > 0.6 and DARK_AMBER or SHADE)
    if ch == 'h':
        return EMBER if t < 0.5 else SHADE
    if ch == 'c':
        return EMBER if x == x0 else AMBER
    if ch == 'r':
        if x1 == x0:
            return ROOT_C
        return ROOT_L if x == x0 else ROOT_D
    raise ValueError(ch)


def art():
    img = Image.new('RGBA', (32, 32), (0, 0, 0, 0))
    rows_of = {}
    for y, row in enumerate(ART):
        assert len(row) == 32, (y, len(row))
        for ch in set(row) - {' '}:
            rows_of.setdefault(ch, []).append(y)
    for y, row in enumerate(ART):
        x = 0
        while x < 32:
            ch = row[x]
            if ch == ' ':
                x += 1
                continue
            x0 = x
            while x < 32 and row[x] == ch:
                x += 1
            for xx in range(x0, x):
                img.putpixel((xx, y), shade_of(ch, xx, x0, x - 1, y, rows_of[ch]) + (255,))
    # 1px 외곽선: 그림 칸에 닿는(상하좌우) 빈 칸.
    filled = {(x, y) for y in range(32) for x in range(32) if img.getpixel((x, y))[3]}
    for y in range(32):
        for x in range(32):
            if (x, y) in filled:
                continue
            if any((x + dx, y + dy) in filled for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1))):
                img.putpixel((x, y), OUTLINE + (255,))
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
