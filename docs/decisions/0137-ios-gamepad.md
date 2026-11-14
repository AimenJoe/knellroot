# 0137. iOS 게임 컨트롤러 — Android 패치와 같은 꼴(F13~F24 키 + `joystick_axes`)

- 날짜: 2026-11-10
- 상태: **구현**(빌드까지, 실기 시험은 사람 몫). 사장 2026-11-10 "Android에서 했던것처럼, iOS에도 컨트롤러 지원 추가해줘." — 0136 "다음 단계"의 첫째.
- 바탕: Android 패드 패치(0022 — nail 0075 버튼·0143 축), KB `reference/macroquad/ios-build.md`("패드 다리가 필요하다")·`gamepad-and-android-input.md`.

## 왜 다리가 필요했나
- 원본 miniquad iOS 백엔드는 패드를 읽지 않는다. `gamepads` 크레이트는 iOS에서 gilrs를 쓰는데 gilrs-core에 iOS 플랫폼이 없어 늘 빈 패드였다(KB 1차). 그래서 iOS는 터치만 됐다.

## 정한 것
- **Android와 같은 꼴**: 패치한 miniquad가 패드를 키로 넘기고 게임은 같은 길로 읽는다 — 게임 쪽에 iOS 전용 입력 코드를 두지 않는다.
  - 새 파일 `third_party/miniquad/src/native/ios_pad.rs`: Apple GameController(`GCController.controllers` → 첫 `extendedGamepad`)를 그리기마다 한 번(메인 스레드, `draw_in_rect`가 `update()` 직전에) 읽는다. Apple 문서 "Handling input events"의 폴링 길(게임 루프가 고정 간격으로 읽을 때)이다.
  - 버튼 열둘 → Android 패치와 같은 `KeyCode::F13`~`F24` 눌림·뗌 이벤트(A·B·X·Y·LB·RB·LT·RT·LS·RS·Menu·Options). Apple의 A·B·X·Y는 위치 이름(A = 아래)이라 게임의 `Pad::South` 등과 같다.
  - 스틱·D패드 → `joystick_axes()`(Android와 같은 배열 [왼 X, 왼 Y, 오른 X, 오른 Y, D패드 X, D패드 Y], 아래가 +y — GameController는 위가 +라 뒤집는다). D패드는 HAT 칸으로 와서 Android처럼 스틱이 서 있으면 이동·메뉴가 된다.
  - 패드가 빠지면 눌린 것을 모두 뗀다(다음 그리기의 빈 마스크).
- **게임 쪽**: `build.rs`가 Android·iOS에서 `cfg(pad_keys)`를 켜고, `input.rs`의 Android 분기 열두 곳을 그 이름으로 바꿨다(패드 키 읽기·"패드 키를 키보드로 세지 않기"·마우스 없음·`gamepads` 빼기). `Cargo.toml`도 iOS에서 `gamepads`를 뺀다(빈 백엔드였다).
- **Info.plist**: Xcode "Game Controllers" 능력이 넣는 두 키를 손으로 — `GCSupportsControllerUserInteraction` = true, `GCSupportedGameControllers` = [ExtendedGamepad]. iOS에서 이 키 없이도 입력이 오는지는 확인 못 함(Apple 문서는 "시스템에 지원을 알린다"고만 한다) — 넣어 두는 쪽이 안전하고 App Store의 컨트롤러 표시에도 쓰인다.
- 시험 `ios_pad_keys_match_patched_miniquad`: `ios_pad.rs`의 버튼 읽는 차례와 키 표가 게임의 `Pad::key`와 같은 짝인지, 축 부호가 Android와 같은지.

## 확인한 것 / 못 한 것
- 확인: 리눅스에서 `cargo clippy --release --target aarch64-apple-ios`(운영판·개발판) 경고 없음, 다른 타깃·`cargo test`(355) 그대로. CI `build-ios`가 GameController 프레임워크를 링크해 `.ipa`를 만드는지(아래 커밋의 실행).
- 못 함: 실기(사장) — 패드 연결·버튼 열둘·스틱 세기·D패드 메뉴·패드를 뗐다 붙이기. Menu 버튼을 시스템이 가로채는지(iOS가 홈 버튼은 가로챈다 — 우리는 Home을 읽지 않는다).
- 알려진 것: 그리기 사이(1/60초 안)에 눌렀다 뗀 아주 짧은 누름은 놓칠 수 있다(폴링의 한계 — Apple 문서도 같은 말). 문제가 되면 `valueChangedHandler`(콜백)로 바꾼다.

## 손질 — 첫 게임 nail 0288과 견주어(2026-11-10, 사장 "서로 비교해볼래?" → "다섯 다")
nail은 같은 날 같은 요청을 **게임 쪽 모듈**(`src/input/ios_pad.rs` + 순수 `PadFrame` → `pad_frame_actions`, miniquad는 그대로)로 풀었다. 구조는 각자 맞는 쪽으로 둔다 — knellroot는 Android와 한 길(키·축), nail은 플랫폼마다 `Actions`를 돌려주는 입력 구조. 대신 nail에서 다섯을 가져왔다.
1. **링크 보장**: 알림 이름 상수(`GCControllerDidConnectNotification`·`DidDisconnect…`)를 실제로 참조한다. 전엔 빈 `#[link]` 블록만 있어, 링커가 프레임워크를 빼면 `class!(GCController)`가 첫 그리기에서 panic할 수 있었다(objc-rs `class!`는 못 찾으면 panic — 소스로 확인). `build-ios.yml`이 `otool -L`로 GameController.framework를 본다.
2. **`Class::get`**: 못 찾으면 panic 대신 "패드 없음".
3. **연결·끊김 알림 구독**(빈 처리기, 한 번) — Apple 권장, iOS 15부터 프레임워크가 이때 패드를 찾기 시작한다는 포럼 답(2차).
4. **`autoreleasepool`**로 읽기를 감싼다.
5. **패드 여럿 합치기**: 버튼은 어느 쪽이든, 스틱·D패드는 더 기운 쪽(데스크톱 `gamepads` 길과 같다).
