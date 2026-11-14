# 0135. PC 빌드 — Windows · macOS · Linux (v0.6 P1)

- 날짜: 2026-11-10
- 상태: **구현**(CI 묶음까지, 실기 시험은 사람 몫). 0134 P1, 사장 2026-11-10 "P1 PC 빌드부터".
- 바탕: nail 0240(같은 엔진·같은 꼴), KB `reference/macroquad/desktop-build.md`.

## 왜 nail보다 쉬웠나
- **에셋 경로 문제가 없다.** nail에서 가장 크게 막힌 것은 miniquad 데스크톱이 파일을 "지금 폴더" 기준으로 여는 것(바로가기·Finder로 켜면 그림·소리를 못 찾음)이었다. knellroot는 처음부터 모델(`build.rs`)·소리 wav·글꼴(`include_bytes!`)을 모두 바이너리에 넣어 실행 중에 디스크를 읽지 않는다 → `assets_dir()` 같은 찾기가 필요 없고, 묶음은 **실행 파일 하나 + 고지 셋**이다. 소리가 wav라 "네이티브는 mp3를 못 푼다"(nail 0235 R20)도 없다.
  - 확인: 리눅스 운영판을 작업 폴더 `/`에서 Xvfb로 켜서 타이틀(글꼴·3D·끝내기 줄)이 뜸.
- **창 ✕도 안전.** 기록은 판의 매 틱 `sync_progress` → `persist`가 바뀔 때마다 임시 파일에 쓰고 이름 바꾸기로 쓴다(0064 M2) — nail 0264처럼 `prevent_quit`로 닫기를 붙잡을 까닭이 없다.
- 저장 자리(`prefs::data_dir`: `%APPDATA%\knellroot`, `~/Library/Application Support/knellroot`, `$XDG_DATA_HOME` 또는 `~/.local/share/knellroot`)와 패드(gamepads → gilrs)·키보드+마우스 기본 배치(0058)는 이미 있었다.

## 바꾼 것
- **게임**
  - `crate::DESKTOP`(wasm·Android·iOS가 아니면 참)으로 PC 전용을 가른다.
  - 타이틀 끝 줄 **끝내기**(`TitleItem::Quit` → `order_quit`, PC만 — 웹·Android 줄은 그대로라 헤드리스 시험의 길이 같다). 글 둘(한국어 "끝내기"·영어 "Quit", 글자는 이미 글꼴에 있음).
  - **F11** = 창 ↔ 전체 화면(`input::fullscreen_pressed`, 키 배치 밖 — 창 일이라 `Actions`에 넣지 않는다, nail과 같은 꼴). 설정 "전체 화면" 줄은 이미 있었다(0046).
  - **전체 화면 기억**: prefs 칸 28(1 창·2 전체, 0 = 저장 전 = 창, 뒤에 붙인 칸이라 MAGIC 그대로). PC만 읽고(`main.rs window_conf`가 miniquad `conf.fullscreen`으로 연다 — Windows·macOS·X11 모두 시작 때 반영), 웹은 사용자 입력 없이 전체 화면을 못 열어 쓰지 않는다.
  - Windows 운영판은 콘솔 창 없음(`windows_subsystem = "windows"`, 디버그 빌드 빼고).
  - 창·작업 표시줄 아이콘(`window_icon`, `pc/window-16·32·64.png` — 없으면 miniquad 로고), 리눅스 `WM_CLASS` = `knellroot`.
- **아이콘**: `tools/icons/make_icons.py`가 `pc/knellroot.ico`(16~256, 크기마다 따로 — 큰 것을 줄이면 16·32가 뭉갬)·`pc/knellroot.icns`(16~1024)·창 아이콘 PNG 셋도 만든다(같은 종·뿌리 그림, 옛 산출물은 바이트 그대로).
- **고지**: `tools/licenses.py`(nail에서 가져옴) → `assets/THIRD_PARTY_LICENSES.txt` — Rust 크레이트 원문(MIT·Apache는 바이너리 배포에도 고지 의무), 글꼴 OFL, **Meshy 모델의 CC BY 4.0 표기**(`assets/LICENSES.md` 표에서 읽음), CC0 소리 크레딧. PC 묶음에 `LICENSES.md`·`GowunBatang-OFL.txt`와 함께 싣는다. 의존성·에셋 출처를 바꾸면 다시 돌린다.
- **CI** `build-desktop.yml`: Windows(windows-latest, x64, rcedit로 아이콘·버전 → zip), macOS(macos-latest, arm64 + x86_64 → `lipo` 겸용 `.app`, Info.plist, 임시 서명 → `ditto` zip), Linux(ubuntu-22.04 — glibc를 낮게, tar.gz). `release/**` push(개발판)·손으로·`release.yml`(태그 때 운영판을 Release에, APK 잡 뒤).

## 같이 찾아 고친 것 — 외곽선·그림자 배치가 색인 한도를 넘었다
- 리눅스 빌드를 켜자 `geometry() exceeded max drawcall size, clamping`이 프레임마다 둘씩 찍혔다(12초에 1,449번). 웹 콘솔에도 같은 경고가 있었다(헤드리스 shot.js 한 번에 2,350번 — 시험은 error만 모아 못 봤다).
- 까닭: 외곽선 껍데기(`send_outline`)와 그림자 배우 모음(`casters`)이 **정점만** 세어 끊었다. Meshy 모델은 정점을 나눠 써 색인이 정점의 2.4배쯤이라, 정점 8,000~9,800에서 색인이 19,000~25,000이 되어 15,000(0036)을 넘고 macroquad가 뒤를 잘랐다 → 뒤쪽 배우의 외곽선·그림자가 빠짐.
- 고침: `render::fits(v, i, nv, ni)` 한 곳(정점·색인 둘 다)을 몸 배치·외곽선·그림자가 같이 쓴다. 외곽선은 넣기 **전에** 넘칠지 보고 보낸다. 그림자 지도는 한 번에 그리므로 다 차면 그 뒤 배우는 그림자 없이(반쯤 잘린 것보다 낫다). 시험 `a_batch_counts_indices_too`. 고친 뒤 웹·리눅스 모두 경고 0, 화면에서 적의 외곽선이 끊겼던 자리(다리·날개)가 이어짐.

## 서명(사람 몫)
- **macOS**: 임시(ad-hoc) 서명만 — lipo로 합치면 arm64 링커 서명이 깨져 애플 실리콘에서 안 켜지므로 다시 서명한다. 공증이 없으면 macOS 15는 받은 앱의 첫 실행을 막는다 → "시스템 설정 → 개인정보 보호 및 보안 → 그래도 열기"(KB: 2차 출처). 공증은 Apple Developer($99/년) 뒤 — nail과 같은 계정을 쓸 수 있다.
- **Windows**: 서명 없음 → 직접 받으면 SmartScreen "추가 정보 → 실행".

## 확인한 것 / 못 한 것
- 확인: `cargo clippy`(Windows `x86_64-pc-windows-msvc`·macOS `aarch64-apple-darwin`, 운영판, 리눅스에서 링크 없이), 리눅스 운영판을 `/`에서 켜 타이틀(끝내기 줄 포함), `cargo test` 전부, 헤드리스 shot.js·save.js.
- 못 함(이 컨테이너에 창 관리자·소리 장치가 없음): 전체 화면으로 시작(X11 전체 화면은 창 관리자가 해 준다 — 칸 읽기·쓰기는 단위 시험), 소리(장치가 없으면 소리 스레드만 죽고 게임은 돈다). Windows·Mac 실기.
- 알려진 것: macOS에서 초록 버튼·Esc로 전체 화면을 나가면 게임이 아는 상태(`screen::active`)와 어긋난다 — 다음 F11이 한 번 헛돈다(miniquad가 지금 상태를 알려 주지 않음).

## 남은 것
- 실기 시험(사장): Windows PC·Mac에서 켜기, 패드, 전체 화면·F11·다시 켰을 때 기억, 저장 자리, 소리, 끝내기.
- Mac 공증, 소개 페이지에 PC 내려받기 칸(v0.6.0 릴리스 때 — Release에 셋이 붙은 뒤), Steam 데포는 판매 준비 마일스톤.
