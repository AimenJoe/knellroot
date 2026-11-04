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
mod secondary;
mod species;
mod stage;
mod text;
mod touch;
mod tuning;
mod ui;
mod world;

use macroquad::prelude::*;

fn window_conf() -> macroquad::conf::Conf {
    macroquad::conf::Conf {
        miniquad_conf: Conf {
            window_title: "Knellroot".to_owned(),
            window_width: 1280,
            window_height: 720,
            high_dpi: true,
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
    loop {
        app.frame();
        next_frame().await;
    }
}
