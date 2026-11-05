#!/usr/bin/env sh
# 웹 배포 폴더 조립: sh tools/web/assemble.sh dist [빌드 아이디] [release|dev]
# web/의 페이지·JS·_headers·매니페스트·아이콘 + knellroot.wasm + build.txt(어느 커밋이 올라갔는지 — 배포 확인용), sw.js에 빌드 아이디(0062).
# dev(개발판 별칭)면 제목에 "(dev)"를 붙이고 검색에 안 잡히게 noindex.
set -e
out="${1:-dist}"
build="${2:-local}"
flavor="${3:-dev}"
root="$(cd "$(dirname "$0")/../.." && pwd)"
rm -rf "$out"; mkdir -p "$out"
cp "$root"/web/index.html "$root"/web/*.js "$root"/web/_headers "$root"/web/manifest.webmanifest "$out"/
cp -r "$root"/web/icons "$out"/icons
cp "$root"/target/wasm32-unknown-unknown/release/knellroot.wasm "$out"/
echo "$build" > "$out"/build.txt
# 서비스 워커(0062): 캐시 이름 = 빌드 아이디 → 배포마다 sw.js가 바뀌어 새 워커가 깔린다.
grep -q "__BUILD__" "$out"/sw.js || { echo "sw.js 자리표 __BUILD__ 가 없다" >&2; exit 1; }
sed -i -e "s|__BUILD__|$build|" "$out"/sw.js
if [ "$flavor" = "dev" ]; then
  sed -i -e 's|<title>\(.*\)</title>|<title>\1 (dev)</title>\n<meta name="robots" content="noindex">|' "$out"/index.html
  # 설치 이름도 따로 — 운영과 나란히 깔린다(주소가 달라 기록도 따로).
  sed -i -e 's|"name": "Knellroot"|"name": "Knellroot (dev)"|' -e 's|"short_name": "Knellroot"|"short_name": "Knell dev"|' "$out"/manifest.webmanifest
fi
echo "assembled $out ($build)"
