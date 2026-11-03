//! Knellroot — 레트로 로우폴리 3D ARPG PoC. 쿼터뷰 떼싸움 ↔ 락온 보스전(숄더뷰).
//!
//! 루프: 프레임마다 입력 폴링 → 고정 타임스텝 60Hz 틱 N번(월드 → 카메라) → 가변 렌더(보간).
//! 경계(nail에서 가져온 규칙): 입력은 `Actions`만, 수치는 `tuning.rs`만, 월드는 카메라·소리·메뉴를 모른다.
//! 범위·판정 기준: docs/phase-0-poc.md. 랩: docs/lab.md.
#![allow(clippy::new_without_default)]

mod camera;
mod input;
mod lab;
mod model;
mod render;
mod scenario;
mod text;
mod touch;
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
    // 측정(P5, 0013): 우리 코드가 쓴 CPU 시간(틱 + 그리기)과 프레임 시간을 1초씩 모은다. 그리기 호출 수는 30프레임마다 한 프레임을 잡아 센다.
    let mut perf = render::Perf::default();
    let (mut cpu_sum, mut cpu_max, mut frame_sum, mut frames) = (0.0f64, 0.0f64, 0.0f64, 0u32);

    loop {
        let cpu_start = get_time();
        if frames % 30 == 0 {
            macroquad::telemetry::capture_frame();
        }
        input.poll(lab.open);
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
                    aim: a.aim,
                    attack: a.attack,
                    dodge: a.dodge,
                    lock: a.lock,
                    turn_cw: a.turn_cw,
                    turn_ccw: a.turn_ccw,
                    ..Actions::default()
                }
            } else {
                a
            };
            world.view_yaw = Some(cam.quarter_yaw_deg(&t));
            for e in world.step(&a, &t) {
                match e {
                    Event::Hit { .. } => cam.trauma = (cam.trauma + 0.25).min(1.0),
                    Event::PlayerHurt => cam.trauma = (cam.trauma + 0.5).min(1.0),
                    Event::EnemyDown { .. } => cam.trauma = (cam.trauma + 0.15).min(1.0),
                    // 땅 돌리기: 카메라가 90° 돈다(입력 기준은 다음 틱부터 새 방위 — 쥔 스틱은 월드가 그대로 둔다).
                    Event::Turn { cw } => {
                        cam.turn(cw, &t);
                        cam.trauma = (cam.trauma + 0.2).min(1.0);
                    }
                    Event::Slam { .. } => cam.trauma = (cam.trauma + 0.35).min(1.0),
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
        // 글자 아틀라스를 그리기 전에 다 굽는다(프레임 중간에 키우면 앞서 쌓은 글자가 지워진 텍스처를 가리킨다).
        text::warm();
        let alpha = (acc / TICK_DT).clamp(0.0, 1.0);
        let overlay = input.overlay(lab.open);
        if frames % 30 == 2 {
            perf.draws = macroquad::telemetry::drawcalls().len();
        }
        renderer.draw(
            &world,
            &cam,
            alpha,
            &t,
            scenario::entry(sc),
            overlay.is_some(),
            &perf,
        );
        lab.draw(&t, overlay.is_some());
        if let Some(o) = &overlay {
            touch::draw(o, world.lock_target().is_some(), world.turn_ready(&t));
        }
        let cpu = get_time() - cpu_start;
        cpu_sum += cpu;
        cpu_max = cpu_max.max(cpu);
        frame_sum += get_frame_time() as f64;
        frames += 1;
        if frames % 60 == 0 {
            perf.cpu_avg_ms = (cpu_sum / 60.0 * 1000.0) as f32;
            perf.cpu_max_ms = (cpu_max * 1000.0) as f32;
            perf.frame_avg_ms = (frame_sum / 60.0 * 1000.0) as f32;
            cpu_sum = 0.0;
            cpu_max = 0.0;
            frame_sum = 0.0;
        }
        next_frame().await;
    }
}
