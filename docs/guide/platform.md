# 플랫폼 · 빌드

`CLAUDE.md`에서 옮긴 규칙이다(0278) — 문장은 그대로이고 같은 힘을 가진다. 고치는 곳: `rust-toolchain.toml` · `.github/` · `web/` · `ios/` · `pc/` · `third_party/miniquad`.
이 영역의 파일을 고치기 전에 읽고, 구조를 바꿨으면 여기를 고친다. 굵은 머리말에서 문단만 갈랐다.

## Rust 버전

Rust 버전은 `rust-toolchain.toml`의 숫자 한 줄로 고정한다(0162 — `stable`은 새 버전이 나온 날 손댄 적 없는 코드의 clippy 지적으로 CI를 깬다). CI는 `.github/actions/rust`가 그 줄을 읽어 깐다 — 워크플로에서 `dtolnay/rust-toolchain`을 직접 부르지 않는다. 올릴 때는 그 줄만 바꾸고 커밋 전 검사를 다 돌린 뒤 커밋.

## PC

PC(0135, nail 0240 꼴): PC 전용은 `crate::DESKTOP`으로 가른다(타이틀 끝내기 `TitleItem::Quit`, F11 `input::fullscreen_pressed` — 키 배치 밖, 전체 화면 기억 prefs 칸 28, Windows 콘솔 창 없음, 창 아이콘 `pc/window-*.png`). 에셋은 늘 바이너리 안에 넣는다(디스크에서 읽지 않는다 — 바로가기·Finder로 켜도 된다). 묶음은 `build-desktop.yml`(release/** push·손으로·릴리스 때), 아이콘은 `tools/icons/make_icons.py`(`pc/knellroot.ico`·`.icns`). 커밋 전에 `cargo clippy --release --target x86_64-pc-windows-msvc --no-default-features -- -D warnings`(macOS는 `aarch64-apple-darwin`, 둘 다 리눅스에서 링크 없이). 의존성·에셋 출처를 바꾸면 `python3 tools/licenses.py`(→ `assets/THIRD_PARTY_LICENSES.txt`, PC 묶음에 실린다).

## iOS

iOS(0136, nail 0086 꼴): `build-ios.yml`(main = 운영판, release/** = 개발판 `.dev`) → 서명 안 한 `.ipa` 아티팩트, `ios/Info.plist`(`__VERSION__`·`__BUILD__`는 CI가 채움)·아이콘 `ios/icons/`(`make_icons.py`). iOS는 `DESKTOP`이 아니고 Android처럼 터치로 시작·전체 화면 줄 없음(`input::initial_device`·`screen`). 패드는 Android와 같은 꼴(0137): 패치한 miniquad `src/native/ios_pad.rs`가 GameController를 그리기마다 읽어 F13~F24 키·`joystick_axes`로 넘기고, 게임은 `cfg(pad_keys)`(build.rs — Android·iOS) 한 길로 읽는다(`gamepads` 크레이트는 둘 다 뺌, 시험 `ios_pad_keys_match_patched_miniquad`). 커밋 전에 `cargo clippy --release --target aarch64-apple-ios --no-default-features -- -D warnings`(리눅스에서 됨).

## 웹 로딩 · PWA

웹 로딩·PWA(0062): 덮개는 `web/index.html`(글 없음, wasm 바이트로 진행, 첫 프레임 뒤 `main.rs` `loaded`), 캔버스 크기는 번들이 resize 때 한 번만 재므로 index.html의 감시가 회전·전체 화면 뒤 몇 번 더 재서 어긋나면 번들 처리기를 다시 부른다(0187 — 시험 `touch.js` 늦은 resize), 서비스 워커 `web/sw.js`는 캐시 = 빌드 아이디·`skipWaiting` 없음. `web/`에 파일을 더하면 `sw.js` `PRECACHE`와 `assemble.sh`도, 시험은 `tools/test/pwa.js`. 아이콘은 `tools/icons/make_icons.py`.
