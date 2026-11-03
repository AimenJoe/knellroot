//! Knellroot — 레트로 로우폴리 3D ARPG PoC. 쿼터뷰 떼싸움 ↔ 락온 보스전(숄더뷰).
//!
//! 루프: 프레임마다 입력 폴링 → 고정 타임스텝 60Hz 틱 N번(월드 → 카메라) → 가변 렌더(보간).
//! 경계(nail에서 가져온 규칙): 입력은 `Actions`만, 수치는 `tuning.rs`만, 월드는 카메라·소리·메뉴를 모른다.
//! 범위·판정 기준: docs/phase-0-poc.md. 랩: docs/lab.md.
#![allow(clippy::new_without_default)]

mod camera;
mod input;
mod lab;
mod render;
mod scenario;
mod tuning;
mod world;

use macroquad::prelude::*;

use camera::{Camera, Mode};
use input::{Actions, Input};
use lab::Lab;
use render::Renderer;
use scenario::Scenario;
use tuning::{Tuning, MAX_FRAME_DT, TICK_DT};
use world::Event;

fn window_conf() -> Conf {
    Conf {
        window_title: "Knellroot PoC".to_owned(),
        window_width: 1280,
        window_height: 720,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut t = Tuning::default();
    let mut input = Input::new();
    #[allow(unused_mut)]
    let mut sc = Scenario::Boss;
    let mut world = scenario::build(sc);
    let mut cam = Camera::new(world.player.pos, &t);
    let mut lab = Lab::new();
    let mut renderer = Renderer::new();
    let mut acc = 0.0f32;

    loop {
        input.poll();
        let dt = get_frame_time().min(MAX_FRAME_DT);
        acc += dt;
        while acc >= TICK_DT {
            let a = input.take();
            if a.lab_toggle {
                lab.open = !lab.open;
            }
            #[cfg(feature = "dev")]
            if let Some(n) = a.scenario {
                sc = if n == 0 {
                    scenario::next(sc)
                } else {
                    scenario::by_number(n).unwrap_or(sc)
                };
                world = scenario::build(sc);
                cam = Camera::new(world.player.pos, &t);
            }
            // 랩이 열려 있으면 커서 키는 랩이 먹고, 이동·공격·회피·락온은 그대로 게임으로(바꾸며 바로 느끼기).
            let a = if lab.open {
                if let Some(text) = lab.update(&a, &mut t) {
                    info!("--- tuning export ---\n{}", text);
                }
                Actions {
                    move_dir: a.move_dir,
                    attack: a.attack,
                    dodge: a.dodge,
                    lock: a.lock,
                    ..Actions::default()
                }
            } else {
                a
            };
            for e in world.step(&a, &t) {
                match e {
                    Event::Hit { .. } => cam.trauma = (cam.trauma + 0.25).min(1.0),
                    Event::PlayerHurt => cam.trauma = (cam.trauma + 0.5).min(1.0),
                    Event::EnemyDown { .. } => cam.trauma = (cam.trauma + 0.15).min(1.0),
                    Event::Swing | Event::LockOn | Event::LockOff => {}
                }
            }
            let target = world.lock_target();
            cam.set_mode(
                if target.is_some() {
                    Mode::Lock
                } else {
                    Mode::Quarter
                },
                target,
                world.player.pos,
            );
            cam.update(TICK_DT, &t, world.player.pos, target, &world.obstacles);
            acc -= TICK_DT;
        }
        let alpha = (acc / TICK_DT).clamp(0.0, 1.0);
        renderer.draw(&world, &cam, alpha, &t, scenario::entry(sc));
        lab.draw(&t);
        next_frame().await;
    }
}
