//! Knellroot — 레트로 로우폴리 3D ARPG. 쿼터뷰 떼싸움 ↔ 락온 보스전(숄더뷰).
//!
//! 장면·루프는 `app`(흐름은 `app/flow.rs` 순수 함수), 규칙은 `world`(+ `enemy`·`species` 표), 스테이지 진행은 `stage`,
//! 그림은 `render`(+ `scenery` 지형 굽기·`models` 모델 목록)와 `hud`·`ui`(글·메뉴), 글은 `i18n` 표.
//! 경계(첫 게임에서 가져온 규칙): 입력은 `Actions`만, 수치는 `tuning.rs`만, 월드는 카메라·소리·메뉴를 모른다.
//! 범위·판정 기준: docs/phase-1-first-stage.md(지금), docs/phase-0-poc.md(닫음). 랩: docs/lab.md.
#![allow(clippy::new_without_default)]

mod app;
mod camera;
mod enemy;
mod ground;
mod hud;
mod i18n;
mod input;
mod lab;
mod model;
mod models;
mod prefs;
mod render;
#[cfg(feature = "dev")]
mod scenario;
mod scenery;
mod screen;
mod secondary;
mod shade;
mod species;
mod stage;
mod text;
mod touch;
mod tuning;
mod ui;
mod weapons;
mod world;

use macroquad::prelude::*;

fn window_conf() -> macroquad::conf::Conf {
    macroquad::conf::Conf {
        miniquad_conf: Conf {
            window_title: "Knellroot".to_owned(),
            window_width: 1280,
            window_height: 720,
            high_dpi: true,
            // Android는 처음부터 몰입 모드(0046): miniquad는 이 값이 참일 때만 시작·복귀 때 바를 숨긴다
            // (`Cargo.toml`의 `fullscreen = true`는 매니페스트 테마뿐이라 내비게이션 바가 남았다).
            fullscreen: cfg!(target_os = "android"),
            ..Default::default()
        },
        // 그리기 한 번의 인덱스 용량(0036): 기본 5,000은 둥근 방랑자 한 벌(≈ 7,800)보다 작아 넘친 부위(몸·다리)가 조용히 잘렸다.
        // 정점 용량은 그대로(10,000). `render::MAX_DRAW_INDICES`와 같이 바꾼다.
        draw_call_index_capacity: render::DRAW_CALL_INDICES,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut app = app::App::new();
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
