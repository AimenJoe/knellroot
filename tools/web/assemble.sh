#!/usr/bin/env sh
# 웹 배포 폴더 조립: sh tools/web/assemble.sh dist [빌드 아이디]
# web/의 페이지·JS·_headers + knellroot.wasm + build.txt(어느 커밋이 올라갔는지 — 배포 확인용).
set -e
out="${1:-dist}"
build="${2:-local}"
root="$(cd "$(dirname "$0")/../.." && pwd)"
rm -rf "$out"; mkdir -p "$out"
cp "$root"/web/index.html "$root"/web/*.js "$root"/web/_headers "$out"/
cp "$root"/target/wasm32-unknown-unknown/release/knellroot.wasm "$out"/
echo "$build" > "$out"/build.txt
echo "assembled $out ($build)"
