#!/usr/bin/env bash
# 배포한 주소의 build.txt가 이 커밋을 가리키는지 본다(배포 직후, deploy-web.yml publish — 0246). 아니면 실패.
#   tools/web/check_live.sh <주소> <커밋 SHA>
set -u
url=${1%/}; sha=$2
for i in 1 2 3 4 5 6; do
  got=$(curl -fsS "$url/build.txt?t=$(date +%s)" || true)
  echo "try $i: $got"
  case "$got" in "$sha"*) echo "ok: $url = $(printf %.7s "$sha")"; exit 0;; esac
  sleep 10
done
echo "::error::$url 이 이 커밋($sha)을 가리키지 않는다"
exit 1
