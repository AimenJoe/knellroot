//! Knellroot — 레트로 로우폴리 3D ARPG. 쿼터뷰 떼싸움 ↔ 락온 보스전(숄더뷰).
//!
//! 장면·루프는 `app`(흐름은 `app/flow.rs` 순수 함수), 규칙은 `world`(+ `enemy`·`species` 표), 스테이지 진행은 `stage`,
//! 그림은 `render`(+ `scenery` 지형 굽기·`models` 모델 목록)와 `hud`·`ui`(글·메뉴), 글은 `i18n` 표.
//! 경계(첫 게임에서 가져온 규칙): 입력은 `Actions`만, 수치는 `tuning.rs`만, 월드는 카메라·소리·메뉴를 모른다.
//! 범위·판정 기준: docs/phase-1-first-stage.md(지금), docs/phase-0-poc.md(닫음). 랩: docs/lab.md.
#![allow(clippy::new_without_default)]
// Windows 운영판(0135, nail 0240): 콘솔 창을 띄우지 않는다. 디버그 빌드는 eprintln!·warn!을 보게 그대로.
#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod app;
mod backdrop;
mod balance;
mod camera;
mod charms;
mod codex;
mod descent;
mod ember;
mod enemy;
mod flora;
mod fx;
mod ground;
mod growth;
mod hud;
mod i18n;
mod icons;
mod idle;
mod input;
mod item_view;
mod knell;
#[cfg(feature = "dev")]
mod lab;
mod model;
mod models;
mod nav;
mod prefs;
mod progression;
mod render;
mod residents;
mod save;
#[cfg(feature = "dev")]
mod scenario;
mod scenery;
mod screen;
mod secondary;
mod sfx;
mod shade;
mod skills;
mod species;
mod stage;
mod stats;
mod text;
mod touch;
mod tuning;
mod tutorial;
mod ui;
mod wake;
mod weapons;
mod world;

use macroquad::prelude::*;

/// 데스크톱(Windows·macOS·Linux) 빌드인가(0135): 끝내기·F11·전체 화면 기억처럼 PC에만 있는 것을 가른다.
/// 에셋은 모두 바이너리 안(모델 `build.rs`, 소리·글꼴 `include_bytes!`)이라 nail 0240의 에셋 경로 문제는 없다.
pub const DESKTOP: bool = cfg!(not(any(
    target_arch = "wasm32",
    target_os = "android",
    target_os = "ios"
)));

/// 창·작업 표시줄 아이콘(0135, `tools/icons/make_icons.py` → `pc/window-*.png`). 없으면 miniquad 로고가 뜬다.
/// macOS Dock은 실행 중 이것을, Finder는 `.app`의 `.icns`를 쓴다.
fn window_icon() -> Option<macroquad::miniquad::conf::Icon> {
    fn rgba<const N: usize>(png: &[u8]) -> Option<[u8; N]> {
        let img = Image::from_file_with_format(png, Some(ImageFormat::Png)).ok()?;
        img.bytes.try_into().ok()
    }
    Some(macroquad::miniquad::conf::Icon {
        small: rgba(include_bytes!("../pc/window-16.png"))?,
        medium: rgba(include_bytes!("../pc/window-32.png"))?,
        big: rgba(include_bytes!("../pc/window-64.png"))?,
    })
}

fn window_conf() -> macroquad::conf::Conf {
    macroquad::conf::Conf {
        miniquad_conf: Conf {
            window_title: "Knellroot".to_owned(),
            window_width: 1280,
            window_height: 720,
            high_dpi: true,
            // Android는 처음부터 몰입 모드(0046): miniquad는 이 값이 참일 때만 시작·복귀 때 바를 숨긴다
            // (`Cargo.toml`의 `fullscreen = true`는 매니페스트 테마뿐이라 내비게이션 바가 남았다).
            fullscreen: cfg!(target_os = "android") || desktop_fullscreen(),
            icon: if DESKTOP { window_icon() } else { None },
            platform: macroquad::miniquad::conf::Platform {
                linux_wm_class: "knellroot",
                ..Default::default()
            },
            ..Default::default()
        },
        // 그리기 한 번의 인덱스 용량(0036): 기본 5,000은 둥근 방랑자 한 벌(≈ 7,800)보다 작아 넘친 부위(몸·다리)가 조용히 잘렸다.
        // 정점 용량은 그대로(10,000). `render::MAX_DRAW_INDICES`와 같이 바꾼다.
        draw_call_index_capacity: render::DRAW_CALL_INDICES,
        ..Default::default()
    }
}

/// PC(0135): 마지막에 고른 대로 전체 화면으로 연다(prefs 칸 28). 웹은 사용자 입력 없이 못 켜고, Android는 늘 몰입 모드.
fn desktop_fullscreen() -> bool {
    let on = DESKTOP && prefs::load().is_some_and(|p| p.fullscreen);
    screen::start_desktop(on);
    on
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut app = app::App::new();
    // 소리(0064 M5): 파일은 바이너리 안, 합성은 여기서 — 첫 프레임 전에 올린다(웹은 브라우저가 뒤에서 푼다).
    app.set_sfx(sfx::Sfx::load().await);
    app.frame();
    next_frame().await;
    // 첫 프레임이 그려졌다 — 웹 로딩 화면을 걷는다(0062, web/index.html `knell_loading`).
    loaded();
    loop {
        app.frame();
        next_frame().await;
    }
}

/// 웹: 로딩 화면(종·막대)을 걷어 달라고 페이지에 알린다. 다른 플랫폼은 할 일 없음.
fn loaded() {
    #[cfg(target_arch = "wasm32")]
    {
        extern "C" {
            fn knell_loaded();
        }
        // SAFETY: 인자·반환 없는 JS 함수(web/index.html의 knell_loading 플러그인).
        unsafe { knell_loaded() };
    }
}
