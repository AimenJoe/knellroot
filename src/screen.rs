//! 전체 화면(0046). 웹은 브라우저 전체 화면 API(web/index.html `knell_fs_*`, 사용자가 누른 입력의 프레임에서만 허락된다),
//! 데스크톱은 창 전체 화면, Android는 처음부터 몰입 모드·iOS 앱은 늘 전체 화면(Info.plist, 0136)이라 줄이 없다(main.rs `fullscreen`, 패치한 miniquad).

#[cfg(target_arch = "wasm32")]
mod js {
    extern "C" {
        pub fn knell_fs_enabled() -> u32;
        pub fn knell_fs_active() -> u32;
        pub fn knell_fs_toggle();
        pub fn knell_fs_touch_enter();
        pub fn knell_portrait() -> u32;
        pub fn knell_safe_inset(i: u32) -> u32;
        pub fn knell_scene(ptr: *const u8, len: usize);
        #[cfg(feature = "dev")]
        pub fn knell_test_flags() -> u32;
    }
}

/// 맨 위 장면의 이름을 페이지에 알린다(웹만 — `window.knellScene`). 헤드리스 시험이 시간 대신 이것을 기다린다:
/// CI의 소프트웨어 GL은 10 fps 안팎이라 "300ms 기다리기"가 몇 프레임인지 정해지지 않았고, 창이 뜨기 전에 키가 들어가
/// `save.js`의 덮어쓰기가 네 번 빠졌다(2026-11-18·19). 장면이 바뀐 프레임에만 부른다.
pub fn report_scene(name: &str) {
    #[cfg(target_arch = "wasm32")]
    // SAFETY: 바이트를 읽기만 하는 JS 함수(web/index.html) — 이 호출 동안만 살아 있는 글을 넘긴다.
    unsafe {
        js::knell_scene(name.as_ptr(), name.len())
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = name;
}

/// 헤드리스 시험이 페이지에 건 깃발(`window.knellTestFlags`, 웹만 — 그 밖은 0). 1 = 방랑자가 맞지 않는다(`World::unhurt`).
/// 개발판에만 있다.
#[cfg(feature = "dev")]
pub fn test_flags() -> u32 {
    #[cfg(target_arch = "wasm32")]
    // SAFETY: 인자 없이 정수를 돌려주는 JS 함수(web/index.html).
    return unsafe { js::knell_test_flags() };
    #[cfg(not(target_arch = "wasm32"))]
    0
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
static DESKTOP_FS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// 이 기기에서 전체 화면 줄을 보일까(iPhone Safari는 요소 전체 화면 API가 없다, Android 앱은 늘 전체 화면).
pub fn supported() -> bool {
    #[cfg(target_arch = "wasm32")]
    // SAFETY: 인자 없이 정수를 돌려주는 JS 함수(web/index.html).
    return unsafe { js::knell_fs_enabled() } != 0;
    #[cfg(any(target_os = "android", target_os = "ios"))]
    return false;
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    true
}

/// 지금 전체 화면인가(웹은 Esc·뒤로로 브라우저가 끌 수 있어 늘 브라우저에 묻는다).
pub fn active() -> bool {
    #[cfg(target_arch = "wasm32")]
    // SAFETY: 위와 같다.
    return unsafe { js::knell_fs_active() } != 0;
    #[cfg(any(target_os = "android", target_os = "ios"))]
    return true;
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    DESKTOP_FS.load(std::sync::atomic::Ordering::Relaxed)
}

/// PC(0135): 창을 전체 화면으로 열었으면 그 상태를 적어 둔다(main.rs `window_conf`가 prefs에서 읽어 연다).
pub fn start_desktop(on: bool) {
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    DESKTOP_FS.store(on, std::sync::atomic::Ordering::Relaxed);
    #[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
    let _ = on;
}

/// 켜거나 끈다 — 메뉴에서 고른 프레임에 부른다.
pub fn toggle() {
    #[cfg(target_arch = "wasm32")]
    // SAFETY: 위와 같다.
    unsafe {
        js::knell_fs_toggle()
    }
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    {
        let on = !active();
        DESKTOP_FS.store(on, std::sync::atomic::Ordering::Relaxed);
        macroquad::window::set_fullscreen(on);
    }
}

/// 터치 기기의 웹에서 "시작"을 누르면 전체 화면 + 가로 잠금(폰에서 주소창이 화면을 먹지 않게). 그 밖에는 아무것도 안 한다.
pub fn touch_enter() {
    #[cfg(target_arch = "wasm32")]
    // SAFETY: 위와 같다.
    unsafe {
        js::knell_fs_touch_enter()
    }
}

/// 세로로 든 터치 기기의 웹인가(0082 — 회전 안내가 덮여 있다, 싸움은 멈춤으로). 웹 밖은 늘 거짓(Android 앱은 가로 고정).
pub fn portrait_blocked() -> bool {
    #[cfg(target_arch = "wasm32")]
    // SAFETY: 위와 같다.
    return unsafe { js::knell_portrait() } != 0;
    #[cfg(not(target_arch = "wasm32"))]
    false
}

/// 화면의 안전 영역(노치·둥근 모서리·홈 막대 — iPhone 가로) 네 변의 폭: [왼쪽, 위, 오른쪽, 아래], `screen_width()`와 같은 화소.
/// 웹은 `viewport-fit=cover`로 캔버스가 화면을 다 쓰니(2026-11-19) 터치 버튼·HUD가 이만큼 비킨다. 웹 밖은 0.
pub fn safe_insets() -> [f32; 4] {
    #[cfg(target_arch = "wasm32")]
    // SAFETY: 칸 번호 0..4를 받아 정수를 돌려주는 JS 함수(web/index.html).
    return [0, 1, 2, 3].map(|i| unsafe { js::knell_safe_inset(i) } as f32);
    #[cfg(not(target_arch = "wasm32"))]
    [0.0; 4]
}
