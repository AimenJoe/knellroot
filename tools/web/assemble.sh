#!/usr/bin/env sh
# 웹 배포 폴더 조립: sh tools/web/assemble.sh dist
set -e
out="${1:-dist}"
root="$(cd "$(dirname "$0")/../.." && pwd)"
rm -rf "$out"; mkdir -p "$out"
cp "$root"/web/index.html "$root"/web/*.js "$out"/
cp "$root"/target/wasm32-unknown-unknown/release/blight.wasm "$out"/
echo "assembled $out"
