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
    }
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
