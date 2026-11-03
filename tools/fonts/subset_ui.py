"""UI 글꼴 서브셋(0021): 고운바탕(OFL) 원본에서 게임이 쓸 글자만 남긴다.
    python3 tools/fonts/subset_ui.py <GowunBatang-Regular.ttf> <GowunBatang-Bold.ttf>
넣는 글자: i18n 표의 모든 글자 + ASCII + 라틴-1 + 라틴 확장 A + 쓰는 기호. 고운바탕은 글자 하나가 무거워
KS X 1001 한글 2,350자를 다 넣으면 두 파일 2.7MB — 표 글자만이면 수백 KB(KB `reference/fonts/cjk-pixel-fonts.md`의 서브셋 방식).
`--ks`를 붙이면 2,350자를 다 넣는다.
원본(각 8MB)은 레포에 넣지 않는다 — https://github.com/google/fonts/tree/main/ofl/gowunbatang
표에 새 글자를 쓰면 다시 돌린다(`cargo test`의 글꼴 시험 `every_string_is_drawable_with_the_font`가 잡는다)."""
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
        out = OUT / name
        subset.save_font(font, str(out), opts)
        print(f"{out.relative_to(ROOT)}: {len(unicodes)} chars, {out.stat().st_size // 1024} KB")


if __name__ == "__main__":
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    main(args[0], args[1])
