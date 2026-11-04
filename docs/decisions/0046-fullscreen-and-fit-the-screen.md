# 0046. 전체 화면: Android 몰입 모드, 화면에 맞춘 내부 해상도, 웹 전체 화면 줄

- 날짜: 2026-11-04
- 상태: 채택·구현(사장: v0.1.2를 보고 "안드로이드든 웹이든 풀스크린이 아닌데?")

## 왜 — 원인 셋
1. **Android: 내비게이션 바가 남았다.** `Cargo.toml`의 `fullscreen = true`(cargo-quad-apk)는 매니페스트 테마뿐이고, miniquad는 `Conf::fullscreen`이 참일 때만 시작·복귀 때 `setFullScreen`을 부르는데 우리 `window_conf`는 그 값을 켜지 않았다. 켜도 원본 miniquad 0.4.11의 `setFullScreen`은 API 30+(Android 11+)에서 `setDecorFitsSystemWindows(false)`만 해서 **바가 숨지 않고 내용이 바 밑까지 그려질 뿐**이다 — 바를 숨기는 것은 `WindowInsetsController.hide(…)`다(공식 문서 developer.android.com/develop/ui/views/layout/immersive). 핸드헬드는 대부분 Android 11+.
2. **웹: 정수 배율이 브라우저 창에 안 맞는다.** 0041의 고정 640×360은 720p·1080p "화면"에는 꽉 차지만, 브라우저 창은 주소창·탭만큼 낮다(1080p 모니터에서 보통 1920×950 안팎) → ×2(1280×720)로 떨어져 둘레가 넓게 검었다. 480×270 때(×3 = 1440×810)보다 오히려 나빠졌다.
3. **웹: 전체 화면으로 들어갈 길이 없었다.** 브라우저는 사용자 입력 직후에만 전체 화면을 허락하는데, 게임에 그런 줄이 없었다.

## 정한 것
1. **Android 몰입 모드**: `main.rs` `fullscreen: cfg!(target_os = "android")`. 패치한 miniquad(`third_party/miniquad/java/MainActivity.java` `setFullScreen`)가 API 30+에서 `getInsetsController()`로 상태·내비게이션 바를 숨기고 `BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE`(가장자리를 쓸면 잠깐 보였다 다시 숨는다). 29 이하는 원본 그대로(`IMMERSIVE_STICKY`). `PATCHED.md`에 적고, 시험 `android_fullscreen_hides_the_system_bars`가 패치가 남았는지 본다.
2. **내부 해상도를 화면에 맞춘다(0041 고정 크기를 대신함)**: 픽셀은 늘 정수 배율로 고르게 두고, 렌더 타깃 크기를 화면에 맞춰 바꾼다(`render::fit_internal`).
   - 배율 = 화면 높이(물리 px) / 360을 반올림(1 이상). 내부 해상도 = 화면 / 배율(버림). 남는 검은 테두리는 배율보다 작다(1080p ×3에서 2px 이하).
   - 폭은 높이의 4:3~21:9로 묶는다(세로 폰·아주 넓은 창은 나머지가 테두리).
   - 예: 1920×1080 → 640×360 ×3(그대로), 1920×969 창 → 640×323 ×3, 2400×1080 폰 → 800×360 ×3, 1920×1200 → 640×400 ×3.
   - 세로 시야각은 그대로라 **같은 것은 같은 크기**로 보이고, 넓은 화면은 옆을, 16:10은 위아래를 조금 더 본다(Hor+). 렌더 타깃은 크기가 바뀐 프레임에만 새로 만든다.
   - 외곽선 두께는 지금 줄 수로 센다(`Look::rows`). UI 배율 = 그린 화면 높이 / 270(전에는 정수 배율 × 360/270 — 1080p에서 같은 값 4).
3. **전체 화면 줄**: 타이틀 "전체 화면: 켬/끔"(쓸 수 있을 때만 — 웹 `document.fullscreenEnabled`, 데스크톱은 창, Android는 늘 전체 화면이라 없음). 웹은 nail 0269와 같은 방식(문서 전체를 올리고, 들어간 뒤 가로 잠금 시도, 안 되면 조용히). **터치 기기 웹에서 "시작"을 누르면 바로 전체 화면 + 가로 잠금**(폰에서 주소창이 화면을 먹지 않게). 코드는 `screen.rs` + `web/index.html` `knell_fs_*`.

## 대가·남은 것
- 화면마다 보이는 범위가 조금 다르다(21:9까지). 싸움 공정성에 문제가 되면 폭을 16:9로 묶는다.
- iPhone Safari는 요소 전체 화면 API가 없어 줄이 안 보인다(KB `reference/web/pwa-install-fullscreen-orientation.md`) — 홈 화면에 추가 안내는 PWA를 할 때.
- 실기기 확인 전(사람이 할 일): Android 11+ 핸드헬드에서 바가 숨는지, 쓸어서 잠깐 나왔다 들어가는지.
