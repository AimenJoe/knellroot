"""앱 아이콘(0022). 산출물(android/res/mipmap-*)을 커밋하고, 이 파일이 원본이다 — PNG를 손으로 고치지 않는다.
    python3 tools/icons/make_icons.py
그림: 어둠 속 호박색 종(knell) 아래로 뿌리(root)가 내려간다. 32×32 픽셀 격자에 코드로 찍고 크기마다 가장 가까운 픽셀로 키운다
(게임의 레트로 픽셀 느낌). 첫 게임 nail tools/icons/make_icons.py의 틀(밀도 표·적응형 XML)을 따랐다.
- 옛 아이콘: mipmap-<밀도>/ic_launcher.png(48~192px, 둥근 네모 바탕 + 그림).
- 적응형(Android 8+): mipmap-anydpi-v26/ic_launcher.xml = 배경(바탕색 한 장) + 앞 그림(108dp 칸의 가운데 66dp 안 — 원으로 깎여도 다 보이게).
Cargo.toml [package.metadata.android]의 res = "android/res", icon = "@mipmap/ic_launcher"."""
import math
import os
import random

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

# 32×32 그림 — 문자는 모양만(h 종 고리 · b 종 몸 · l 입술 · c 추). 뿌리는 `roots()`가 기른다. 음영·외곽선은 `art()`가 모양에서 계산한다:
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
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
]
ART = [row.ljust(32) for row in SHAPE]
OUTLINE = (12, 10, 14)
HI = (255, 222, 150)        # 종의 가장 밝은 빛
MID_HI = (238, 178, 82)
SHADE = (184, 116, 42)
ROOT_L = (150, 104, 124)    # 뿌리 빛 쪽
STYLE = 'clapper'  # 'clapper' = 또렷한 방울에서 뿌리가 돋음(사장 2026-11-03: 방울이 모호하다 → 둘을 비교해 고름) · 'knot' = 방울 대신 뿌리 매듭(촉수처럼 보여 안 씀)
ROOT_SEED = 23  # 시드 몇 개를 나란히 보고 고름


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
    raise ValueError(ch)


def clapper():
    """또렷한 방울(종의 추): 입술 밑 한 줄(외곽선 자리) 띄워 6×4 둥근 공, 왼쪽 위 빛. 뿌리가 이 공에서 돋는다."""
    out = {}
    shape = [" #### ", "######", "######", " #### "]
    for j, row in enumerate(shape):
        for i, ch in enumerate(row):
            if ch == '#':
                x, y = 13 + i, 18 + j
                col = HI if (i, j) in ((1, 0), (1, 1), (0, 1)) else EMBER if (i, j) in ((2, 0), (2, 1)) else SHADE if (i >= 4 or j == 3) else AMBER
                out[(x, y)] = col
    return out


def roots(seed=ROOT_SEED, style=None):
    """종 입술 밑에서 뿌리를 기른다: 여러 곳에서 나와 바깥으로 퍼지며 내려가고, 굵게 시작해 1px로 가늘어지며,
    조금씩 비틀리고 중간에 곁가지가 갈라진다. 시드 고정(같은 그림). 돌려주는 것은 칠할 칸의 집합."""
    rng = random.Random(seed)
    cells = set()

    def stamp(x, y, w):
        if w <= 1.15:
            xx, yy = int(math.floor(x)), int(math.floor(y))
            if 1 <= xx <= 30 and yy <= 29:
                cells.add((xx, yy))
            return
        r = w / 2
        for yy in range(int(math.floor(y - r)), int(math.ceil(y + r)) + 1):
            for xx in range(int(math.floor(x - r)), int(math.ceil(x + r)) + 1):
                if (xx + 0.5 - x) ** 2 + (yy + 0.5 - y) ** 2 <= r * r and 1 <= xx <= 30 and yy <= 29:
                    cells.add((xx, yy))

    def grow(x, y, ang, length, width, depth):
        steps = int(length / 0.25)
        branch_at = {int(steps * rng.uniform(0.3, 0.6))} if depth > 0 else set()
        for i in range(steps):
            t = i / steps
            stamp(x, y, max(width * (1 - t) ** 0.8, 1.0))
            # 흙을 더듬듯 조금씩 비틀리고, 아래(pi/2)로 살짝 끌린다.
            ang += rng.uniform(-0.14, 0.14) + (math.pi / 2 - ang) * 0.012
            x += math.cos(ang) * 0.25
            y += math.sin(ang) * 0.25
            if i in branch_at:
                side = 1 if math.cos(ang) >= 0 else -1  # 바깥쪽으로 갈라진다
                if rng.random() < 0.35:
                    side = -side
                grow(x, y, ang + side * rng.uniform(0.5, 0.8), length * (1 - t) * 0.6, max(width * (1 - t) * 0.6, 1.0), depth - 1)

    if style == 'knot':
        # 방울 자리의 뿌리 매듭 하나에서 사방(아래 반원)으로 퍼진다.
        cx, cy = 16.0, 20.0
        for xx in range(12, 20):
            for yy in range(18, 23):
                if ((xx + 0.5 - cx) / 2.7) ** 2 + ((yy + 0.5 - cy) / 2.1) ** 2 <= 1.0:
                    cells.add((xx, yy))
        # 매듭 아래 반원 둘레에서 바깥으로(나오는 각 = 뻗는 각).
        for da, length, width, depth in [(0.0, 9.5, 2.2, 2), (0.5, 9.5, 1.9, 1), (-0.5, 9.5, 1.9, 1), (1.05, 8.5, 1.6, 1), (-1.05, 8.5, 1.6, 1)]:
            ang = math.pi / 2 + da
            grow(cx + math.cos(ang) * 2.2, cy + math.sin(ang) * 1.7, ang, length, width, depth)
    else:
        # (나오는 x, 방향(아래 = pi/2에서 더하는 각), 길이, 굵기, 곁가지 깊이) — 'clapper'면 방울 밑에서 시작.
        y0 = 22.2 if style == 'clapper' else 17.6
        for x0, da, length, width, depth in [
            (15.5, 0.06, 8.0 if style == 'clapper' else 12.5, 2.6, 2),   # 곧은뿌리
            (12.5, 0.42, 11.5, 2.0, 1),
            (19.0, -0.42, 11.5, 2.0, 1),
            (10.0, 0.9, 9.5, 1.6, 1),
            (21.5, -0.9, 9.5, 1.6, 1),
        ]:
            if style == 'clapper':
                x0 = 16.0 + (x0 - 15.75) * 0.35
            grow(x0, y0, math.pi / 2 + da, length, width, depth)
    return cells


def art(seed=ROOT_SEED, style=None):
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
    # 뿌리: 빛은 왼쪽 위 — 왼쪽·위가 비면 밝게, 오른쪽·아래가 비면 그늘, 안쪽은 가운데 색. 종 위에는 칠하지 않는다.
    style = style or STYLE
    if style == 'clapper':
        # 방울: 입술 밑 둥근 공(5×4), 왼쪽 위 빛.
        for (x, y), c in clapper().items():
            img.putpixel((x, y), c + (255,))
    rc = roots(seed, style)
    for (x, y) in rc:
        if img.getpixel((x, y))[3]:
            continue
        lit = (x - 1, y) not in rc or (x, y - 1) not in rc
        dark = (x + 1, y) not in rc or (x, y + 1) not in rc
        col = ROOT_L if lit and not dark else ROOT_D if dark and not lit else ROOT_C
        img.putpixel((x, y), col + (255,))
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
