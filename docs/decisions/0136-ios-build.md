# 0136. iOS 빌드 1단계 — 서명하지 않은 .ipa(내 기기 시험)

- 날짜: 2026-11-10
- 상태: **구현**(CI 묶음까지, 실기 시험은 사람 몫). 사장 2026-11-10 "PC는 추가되었고, iOS는?" → "지금 1단계만" — v0.6 범위(0134)에 더한다.
- 바탕: nail 0086(같은 엔진·같은 꼴), KB `reference/macroquad/ios-build.md`. 개념 문서의 "iOS는 빌드만"(concept.md)과 같은 범위.

## 확인한 것
- 리눅스에서 `cargo clippy --release --target aarch64-apple-ios`(운영판·개발판) 경고 없음 — 링크·실행은 macOS 러너에서.
- 저장 자리는 이미 iOS 샌드박스에 맞다(`prefs::data_dir`: macOS·iOS = `~/Library/Application Support/knellroot`).
- 에셋이 모두 바이너리 안이라(0135) `.app`에 assets 폴더가 없다 — nail은 NSBundle 경로에 맞춰 assets를 .app 바로 아래에 넣어야 했다.
- 게임패드는 iOS에서 안 된다(gilrs에 iOS 백엔드 없음 — KB 1차). 터치는 된다.

## 바꾼 것
- **게임**: iOS 앱을 손가락 장치로 시작(`input::initial_device` → 터치 버튼이 첫 화면부터), 전체 화면 줄 없음(`screen` — iOS 앱은 Info.plist로 늘 전체 화면). 그동안 iOS는 이 두 곳에서 PC로 잡혔다(키보드로 시작·전체 화면 줄). `crate::DESKTOP`은 처음부터 iOS를 뺐다(끝내기·F11 없음).
- **`ios/Info.plist`**(nail 0086 그대로, 이름만): 번들 아이디 `io.github.aimenjoe.knellroot`(Android와 같음), 버전·빌드 번호는 `Cargo.toml`의 `version`·`version_code`, 가로만·상태 표시줄 숨김·`UILaunchScreen`(스토리보드 없이 전체 화면, iOS 14+)·`UIRequiresFullScreen`, 최소 iOS 14.
- **아이콘**: `tools/icons/make_icons.py` → `ios/icons/AppIcon*.png`(120·152·167·180·1024, 투명 없이 바탕 꽉 — iOS가 둥글게 깎는다, 같은 종·뿌리 그림).
- **CI `build-ios.yml`**(macos-latest): main = 운영판, `release/**`·손으로 = 개발판(번들 아이디 `.dev`·이름 "Knellroot dev" — 운영 앱과 나란히, 기록도 따로). `Payload/Knellroot.app`(실행 파일·아이콘·Info.plist·PkgInfo·고지 셋) → `knellroot-ios-unsigned.ipa`를 Actions 아티팩트로(7일). Release에는 붙이지 않는다(서명 없는 파일은 그대로 설치되지 않는다).

## 시험 방법(사장)
1. Actions → build-ios(이 갈래 push로 돈 것, 또는 Run workflow) → 아티팩트 `knellroot-ios-unsigned-dev-…`를 받아 압축을 푼다.
2. Sideloadly(또는 AltStore·SideStore)로 내 Apple ID 서명·설치 — 무료 계정은 7일마다 다시, 앱 셋까지(nail과 같은 칸을 나눠 쓴다).
3. iPhone 설정 → 일반 → VPN 및 기기 관리에서 개발자 앱을 신뢰.
4. 볼 것: 가로로 켜지나, 터치 버튼(첫 화면부터), 화면 배율·노치 둘레, 소리, 저장(끄고 다시 켜서 이어하기), 글꼴.

## 하지 않은 것(다음 단계)
- 게임패드 다리(Apple `GameController` → 키 코드, Android 패치 0022 꼴) — 핸드헬드·컨트롤러가 목표라 iOS 배포 전에 필요.
- 정식 서명·TestFlight(Apple Developer Program $99/년 — nail과 같은 계정을 쓸 수 있다, 인증서·프로필·App Store Connect API 키를 Secrets로) — 판매 준비 마일스톤.
