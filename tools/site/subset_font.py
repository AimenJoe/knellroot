"""소개 페이지(site/**/*.html)에 나오는 글자만 남겨 고운바탕 굵게를 woff2로 줄인다(SIL OFL 1.1, 예약 이름 없음, 0093).
페이지 글을 고치면 다시 돌린다. 원본은 Google Fonts(github.com/google/fonts ofl/gowunbatang) — 레포에 넣지 않는다.

    python3 tools/site/subset_font.py <GowunBatang-Bold.ttf>
"""
import re
import sys
from html import unescape
from pathlib import Path

from fontTools import subset

SITE = Path(__file__).resolve().parents[2] / "site"


def main(src: str) -> None:
    text = "".join(unescape(re.sub(r"<[^>]+>", " ", p.read_text(encoding="utf-8"))) for p in SITE.rglob("*.html"))
    opts = subset.Options()
    opts.flavor = "woff2"
    opts.layout_features = ["*"]
    font = subset.load_font(src, opts)
    sub = subset.Subsetter(opts)
    sub.populate(text=text)
    sub.subset(font)
    out = SITE / "fonts" / "gowun-bold-sub.woff2"
    subset.save_font(font, str(out), opts)
    print(out, out.stat().st_size, "bytes")


if __name__ == "__main__":
    main(sys.argv[1])
