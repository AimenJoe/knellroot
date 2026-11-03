# miniquad 0.4.11 — nail-under-the-roots 패치본(Knellroot가 그대로 가져옴, 0022)

crates.io `miniquad 0.4.11` 원본(MIT OR Apache-2.0, 라이선스 파일 그대로)에서 아래를 고쳤다(docs/decisions/0075·0143).

- `src/native/android/keycodes.rs`: Android 게임패드 버튼 키코드(KEYCODE_BUTTON_A 96 … BUTTON_MODE 110)를
  `KeyCode::F13`~`F25`로 넘긴다. 원본은 이 키코드들을 `Unknown`으로 버려서, Android에서 D패드(화살표 키로 들어온다)
  말고는 패드 버튼이 하나도 들어오지 않았다.

루트 `Cargo.toml`의 `[patch.crates-io]`가 이 폴더를 쓴다. cargo-quad-apk도 `java/MainActivity.java`를 이 폴더에서 가져간다.
miniquad를 올릴 때는 새 원본을 받아 같은 패치를 다시 넣는다(`git diff`로 이 파일의 변경을 본다).
- `src/lib.rs` 맨 위 `#![allow(warnings)]`: 경로 의존성은 cargo가 경고를 숨기지 않아(crates.io 의존성은 `--cap-lints`로 숨긴다) 원본의 dead_code 경고가 우리 빌드·clippy 출력에 섞였다.
- 패드 축(0143): `java/MainActivity.java` `QuadSurface.onGenericMotionEvent`(조이스틱 축 여섯을 넘기고 true, import `InputDevice`),
  `java/QuadNative.java` `surfaceOnJoystick`, `src/native/android.rs`의 JNI 진입점 `surfaceOnJoystick` + `pub fn joystick_axes()`.
  cargo-quad-apk는 `QuadNative.java`도 이 폴더에서 가져간다(build.rs의 `miniquad_root_path.join("java")`).
- 키 반복(0171): 원본은 Android가 버튼을 누르고 있는 동안 보내는 반복 `ACTION_DOWN`(`getRepeatCount() > 0`)을 전부 새 누름으로
  넘겨(`key_down_event(…, false)`) macroquad의 `is_key_pressed`가 누르고 있는 내내 다시 참이 됐다 — 땅파기·강타 모으기가 안 됐다.
  `java/MainActivity.java` `onKey`가 `surfaceOnKeyDown(keyCode, repeat)`로 플래그를 넘기고, `QuadNative.java`·`src/native/android.rs`
  (`Message::KeyDown { repeat }`)가 `key_down_event`까지 전한다. 웹(`event.repeat`)·데스크톱과 같아졌다.
