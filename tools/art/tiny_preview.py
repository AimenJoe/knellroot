"""그림 한 장을 게임 화면 크기로 줄여 본다(0038): 이 캐릭터가 쿼터뷰(키 약 20px)·락온(약 50px)에서 무엇이 남는가.

    python3 tools/art/tiny_preview.py <그림.png> [-o 결과.png] [--colors 16] [--heights 60 150]   # 문지기(방랑자 키 약 3배)는 60·150

바탕은 네 귀퉁이 색으로 보고 지운다(흰 바탕·회색 바탕). 줄이기 → 색 수 줄이기 → 1px 자두빛 외곽선 → 픽셀 그대로 키우기.
Meshy에 넣기 전에 Gemini 그림을 거르는 용도. 렌더러의 진짜 모습(빛 띠·원근)은 아니다 — 크기 감만.
"""
import argparse

import numpy as np
from PIL import Image, ImageDraw

OUTLINE = (33, 18, 41)  # render OUTLINE (0.13, 0.07, 0.16)
HEIGHTS = (20, 50)  # 쿼터뷰·락온에서 방랑자(1.4m) 키, 지금 카메라 값으로 계산
SHOW = 360  # 결과에서 한 칸 높이


def cut_out(im, tol=24):
    """귀퉁이 색과 가깝고 바깥에서 이어진 칸만 바탕으로 본다(몸 안쪽의 비슷한 색은 남긴다)."""
    im = im.convert("RGB")
    w, h = im.size
    corners = [im.getpixel(p) for p in ((2, 2), (w - 3, 2), (2, h - 3), (w - 3, h - 3))]
    bg = tuple(sorted(c[i] for c in corners)[1] for i in range(3))
    d = np.abs(np.asarray(im, dtype=np.int16) - np.array(bg, dtype=np.int16)).max(axis=2)
    near = Image.fromarray(np.where(d <= tol, 255, 0).astype(np.uint8)).copy()  # fromarray는 배열을 빌려 써서 floodfill이 안 먹는다
    for p in ((0, 0), (w - 1, 0), (0, h - 1), (w - 1, h - 1)):
        if near.getpixel(p) == 255:
            ImageDraw.floodfill(near, p, 128)
    mask = near.point(lambda v: 0 if v == 128 else 255)
    rgba = im.convert("RGBA")
    rgba.putalpha(mask)
    return rgba.crop(mask.getbbox())


def tiny(sub, height, colors):
    w = max(1, round(sub.width * height / sub.height))
    small = sub.resize((w, height), Image.Resampling.BOX)
    alpha = small.getchannel("A").point(lambda v: 255 if v >= 128 else 0)
    rgb = small.convert("RGB").quantize(colors, method=Image.Quantize.MAXCOVERAGE).convert("RGB")
    out = Image.new("RGBA", (w + 2, height + 2), (0, 0, 0, 0))
    a = Image.new("L", out.size, 0)
    a.paste(alpha, (1, 1))
    for dx, dy in ((-1, 0), (1, 0), (0, -1), (0, 1)):  # 1px 외곽선: 바깥 이웃
        out.paste(Image.new("RGBA", (w, height), OUTLINE + (255,)), (1 + dx, 1 + dy), alpha)
    body = rgb.convert("RGBA")
    body.putalpha(alpha)
    out.alpha_composite(body, (1, 1))
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("image")
    ap.add_argument("-o", "--out", default=None)
    ap.add_argument("--colors", type=int, default=16)
    ap.add_argument("--heights", type=int, nargs="+", default=list(HEIGHTS))
    a = ap.parse_args()
    heights = tuple(a.heights)
    sub = cut_out(Image.open(a.image))
    tiles = [sub.resize((round(sub.width * SHOW / sub.height), SHOW), Image.Resampling.LANCZOS)]
    k = max(1, SHOW // (max(heights) + 2))  # 두 크기를 같은 배율로 — 실제 크기 차이가 보이게
    for h in heights:
        t = tiny(sub, h, a.colors)
        tiles.append(t.resize((t.width * k, t.height * k), Image.Resampling.NEAREST))
    bg = (44, 36, 66)  # 해 질 녘 남보라 바닥 비슷하게
    sheet = Image.new("RGB", (sum(t.width for t in tiles) + 40 * len(tiles), SHOW + 40), bg)
    x = 20
    for t in tiles:
        sheet.paste(t, (x, 20 + SHOW - t.height), t)
        x += t.width + 40
    out = a.out or a.image.rsplit(".", 1)[0] + "_tiny.png"
    sheet.save(out)
    print(out)


if __name__ == "__main__":
    main()
