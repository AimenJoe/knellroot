#!/usr/bin/env bash
# Android 에뮬레이터에서 APK를 켜 보고 로그를 남긴다(2026-11-06 — 개발판이 폰에서 켜지지 않아 원인을 보려고:
# logcat "libdev.so not found" → 개발판 패키지 이름의 마지막 조각이 라이브러리 이름이 되던 것).
#   bash tools/test/android-smoke.sh <apk> <나갈 폴더> [패키지]
# 하는 일: 깔기 → 켜기 → 20초 뒤 화면·살아 있나 → Enter 둘(시작하기 → 빈 슬롯 = 시작) → 15초 뒤 화면·살아 있나 → logcat.
# Rust 패닉은 logcat의 RustStdoutStderr·"panicked", 네이티브 크래시는 "Fatal signal"·"backtrace"로 남는다.
set -u
APK=$1
OUT=$2
PKG=${3:-io.github.aimenjoe.dev.knellroot}
mkdir -p "$OUT"
adb install -r -g "$APK" > "$OUT/install.txt" 2>&1
cat "$OUT/install.txt"
adb logcat -c
adb shell monkey -p "$PKG" -c android.intent.category.LAUNCHER 1 > /dev/null 2>&1
alive() { if adb shell pidof "$PKG" > /dev/null 2>&1; then echo alive; else echo DEAD; fi; }
sleep 20
adb exec-out screencap -p > "$OUT/1-title.png"
echo "after launch: $(alive)" | tee "$OUT/alive.txt"
for _ in 1 2; do adb shell input keyevent 66; sleep 2; done
sleep 15
adb exec-out screencap -p > "$OUT/2-start.png"
echo "after start: $(alive)" | tee -a "$OUT/alive.txt"
adb logcat -d > "$OUT/logcat.txt"
grep -inE "panicked|RustStdoutStderr|Fatal signal|backtrace|SIGSEGV|SIGABRT|AndroidRuntime|knellroot" "$OUT/logcat.txt" | head -200 > "$OUT/crash.txt" || true
echo "----- crash lines -----"
cat "$OUT/crash.txt"
exit 0
