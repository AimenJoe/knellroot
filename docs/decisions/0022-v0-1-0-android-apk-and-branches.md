# 0022. v0.1.0 — Android APK와 갈래(운영·개발)

- 날짜: 2026-11-03
- 상태: 채택·구현(사장: "0.1.0으로 배포할 건데, Android APK도 배포될 수 있도록 해 줘. CI/CD는 Under the Roots 참조하되 여긴 비공개 에셋 같은 게 없으니 관련 설정은 무시해도 좋고")

## 0005와 달라진 것
- 0005는 v0.1.0을 "웹 판, 모델·Android 없음, P1~P3 실기 판정 뒤"로 정했다. 사장이 첫 스테이지 초안(0019)으로 v0.1.0을 내기로 정했다 — **P1~P3 사람 판정과 P5 실기 측정은 하지 않은 채**(phase-1의 "사람이 할 일"에 남아 있다).

## APK
- 첫 게임 nail의 `build-android.yml`·`release.yml`을 가져왔다(KB `guides/macroquad-android-apk-github-actions.md`, `reference/github-actions/release-from-version-bump.md`). 뺀 것: 비공개 에셋 레포 단계, 웹 전용 mp3 지우기, PC·iOS 묶음.
- 액션은 node24 메이저(checkout·upload-artifact v7, cache v6, action-gh-release v3).
- 패키지 `io.github.aimenjoe.knellroot`(0005에서 잡아 둔 것), 개발판 `….knellroot.dev`("Knellroot dev")는 운영 앱과 나란히. `version_code`는 릴리스마다 1씩(v0.1.0 = 1).
- `android:exported`·앱 아이콘이 없으면 CI가 실패한다(nail 0074·0087에서 배운 것). 아이콘은 `tools/icons/make_icons.py`가 32×32 픽셀 그림(호박색 종 + 뿌리)을 코드로 찍어 만든다.
- 서명: Secrets `ANDROID_DEBUG_KEYSTORE_B64` → Actions 캐시 → 새로 생성(경고). 키가 바뀌면 덮어 설치가 안 된다 — 시크릿을 넣어 고정한다(docs/release.md).
- **패드**: Android에서는 `gamepads` 크레이트가 컴파일되지 않아(nail 0005), nail의 패치한 miniquad 0.4.11(`third_party/miniquad`, 버튼 → F13~F24 키, 스틱·HAT → `joystick_axes`, 키 반복 플래그)을 그대로 가져와 `[patch.crates-io]`로 쓴다. `input.rs`의 Android 경로가 데스크톱 패드와 같은 배치로 읽는다(Android 축은 아래가 +y → 뒤집음). 시험: `android_sticks_push_up_for_forward_and_fall_back_to_the_hat`, `android_pad_keys_match_patched_miniquad`, `android_key_repeat_reaches_macroquad`. Android에서는 키 입력을 키보드로 세지 않는다(패드 버튼·D패드도 키로 오므로).

## 갈래(nail 0193, 0005에 적어 둔 대로)
- `main` = 운영판: 웹 운영 주소·Release APK 모두 `--no-default-features`(랩·개발 항목·F1 없음, F3 개발 줄만).
- `release/v0.2` = 개발판이자 작업 갈래: 웹 별칭 `dev`(https://dev--knellroot.netlify.app, noindex), APK 미리 보기 Release `dev`의 `knellroot-dev.apk`.
- 연기 시험은 개발 항목을 쓰므로 운영판 빌드에서도 개발판을 따로 빌드해 돈다(배포물은 운영판 그대로).
- 버전 글자: 타이틀 오른쪽 위 `v0.1.0`(개발판은 `v0.1.0 dev`).

## 다시 볼 조건
- 스토어(Play) 배포를 정하면 릴리스 키·AAB로.
- PC 묶음이 필요해지면 nail `build-desktop.yml`(0240)을 가져온다.
