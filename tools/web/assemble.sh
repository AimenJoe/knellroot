#!/usr/bin/env sh
# 웹 배포 폴더 조립: sh tools/web/assemble.sh dist [빌드 아이디] [release|dev]
# web/의 페이지·JS·_headers + knellroot.wasm + build.txt(어느 커밋이 올라갔는지 — 배포 확인용).
# dev(개발판 별칭)면 제목에 "(dev)"를 붙이고 검색에 안 잡히게 noindex.
set -e
out="${1:-dist}"
build="${2:-local}"
flavor="${3:-dev}"
root="$(cd "$(dirname "$0")/../.." && pwd)"
rm -rf "$out"; mkdir -p "$out"
cp "$root"/web/index.html "$root"/web/*.js "$root"/web/_headers "$out"/
cp "$root"/target/wasm32-unknown-unknown/release/knellroot.wasm "$out"/
echo "$build" > "$out"/build.txt
if [ "$flavor" = "dev" ]; then
  sed -i -e 's|<title>\(.*\)</title>|<title>\1 (dev)</title>\n<meta name="robots" content="noindex">|' "$out"/index.html
fi
echo "assembled $out ($build)"
