"""UI 글꼴 서브셋(0021): 고운바탕(OFL) 원본에서 게임이 쓸 글자만 남긴다.
    python3 tools/fonts/subset_ui.py <GowunBatang-Regular.ttf> <GowunBatang-Bold.ttf>
넣는 글자: i18n 표의 모든 글자 + ASCII + 라틴-1 + 라틴 확장 A + 쓰는 기호. 고운바탕은 글자 하나가 무거워
KS X 1001 한글 2,350자를 다 넣으면 두 파일 2.7MB — 표 글자만이면 수백 KB(KB `reference/fonts/cjk-pixel-fonts.md`의 서브셋 방식).
`--ks`를 붙이면 2,350자를 다 넣는다.
원본(각 8MB)은 레포에 넣지 않는다 — https://github.com/google/fonts/tree/main/ofl/gowunbatang
표에 새 글자를 쓰면 다시 돌린다(`cargo test`의 글꼴 시험 `every_string_is_drawable_with_the_font`가 잡는다).
가로 줄 기호(— – −)는 굵힌다(`thicken_rules`): 원본의 줄은 0.05 em 두께라 화면 글 크기(9~11 px)에서 반 화소 밑으로 녹아
빈칸처럼 보였다(0278 — 영어 큰 창 아래 줄의 "View only — raise at a hearth"). 시험 `rules_are_thick_enough_to_draw`."""
import re
import sys
from pathlib import Path

from fontTools import subset

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "assets" / "fonts"


def ks_x_1001_hangul():
    out = []
    for b1 in range(0xB0, 0xC9):
        for b2 in range(0xA1, 0xFF):
            try:
                out.append(bytes([b1, b2]).decode("euc-kr"))
            except UnicodeDecodeError:
                pass
    return out


def table_chars():
    text = ""
    for f in (ROOT / "src" / "i18n").glob("*.rs"):
        text += "".join(re.findall(r'"((?:[^"\\]|\\.)*)"', f.read_text(encoding="utf-8")))
    # 표 밖 기호(`i18n::UI_GLYPHS`).
    m = re.search(r'UI_GLYPHS: &str = "([^"]*)"', (ROOT / "src" / "i18n" / "mod.rs").read_text(encoding="utf-8"))
    return text + (m.group(1) if m else "")


# 가로 줄 기호(네 점 직사각형): 엠 대시 · 엔 대시 · 빼기. 두께를 이만큼(1000 단위 em)까지 위아래로 같이 늘린다 — 가운데는 그대로.
RULES = (0x2014, 0x2013, 0x2212)
RULE_MIN = 90


def thicken_rules(font):
    glyf = font["glyf"]
    cmap = font.getBestCmap()
    for u in RULES:
        name = cmap.get(u)
        if not name:
            continue
        g = glyf[name]
        coords = g.getCoordinates(glyf)[0]
        if len(coords) != 4:
            raise SystemExit(f"U+{u:04X}: not a plain rule ({len(coords)} points)")
        lo = min(y for _, y in coords)
        hi = max(y for _, y in coords)
        if hi - lo >= RULE_MIN:
            continue
        mid = (lo + hi) / 2
        new_lo, new_hi = round(mid - RULE_MIN / 2), round(mid + RULE_MIN / 2)
        for i, (x, y) in enumerate(coords):
            coords[i] = (x, new_hi if y == hi else new_lo)
        g.recalcBounds(glyf)


def main(regular, bold):
    chars = set(ks_x_1001_hangul()) if "--ks" in sys.argv else set()
    chars |= {chr(c) for c in range(0x20, 0x7F)}
    chars |= {chr(c) for c in range(0xA0, 0x180)}
    chars |= set("‘’“”–—…·•※◆◇○●▲▼▶◀←→↑↓")
    chars |= set(table_chars())
    unicodes = sorted(ord(c) for c in chars if not c.isspace() or c == " ")
    for src, name in [(regular, "GowunBatang-UI-Regular.ttf"), (bold, "GowunBatang-UI-Bold.ttf")]:
        opts = subset.Options()
        opts.layout_features = ["kern", "liga"]
        opts.name_IDs = ["*"]
        opts.notdef_outline = True
        opts.hinting = False
        font = subset.load_font(src, opts)
        s = subset.Subsetter(opts)
        s.populate(unicodes=unicodes)
        s.subset(font)
        thicken_rules(font)
        out = OUT / name
        subset.save_font(font, str(out), opts)
        print(f"{out.relative_to(ROOT)}: {len(unicodes)} chars, {out.stat().st_size // 1024} KB")


if __name__ == "__main__":
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    main(args[0], args[1])
