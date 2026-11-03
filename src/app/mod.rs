//! 앱(0019): 장면(타이틀 → 싸움 → 멈춤 → 끝 → 결과)과 프레임 루프. 흐름은 `flow.rs`의 순수 함수가 정하고 여기서는 그대로 한다.
//! 월드는 메뉴를 모르고, 스테이지(`stage`)는 월드를 지켜보며 진행만 하고, 앱이 둘을 묶어 카메라·그림·글로 보낸다.
//!
//! 루프: 프레임마다 입력 폴링 → 고정 60Hz 틱 N번(장면 → 월드 → 스테이지 → 카메라) → 가변 렌더(보간) → 글·메뉴(화면 패스).
pub mod flow;

use macroquad::prelude::*;

use crate::camera::{Camera, Mode};
use crate::hud;
use crate::i18n::{self, fill, s};
use crate::input::{self, Act, Actions, Input};
use crate::lab::Lab;
use crate::render::{Frame, Perf, Renderer, View};
#[cfg(feature = "dev")]
use crate::scenario::{self, Scenario};
use crate::stage::{sanctuary, StageDef, StageRun};
use crate::touch;
use crate::tuning::{Tuning, MAX_FRAME_DT, TICK_DT};
use crate::ui::{self, Ui, AMBER, DIM, INK};
use crate::world::{Event, World};
use flow::{PauseItem, ResultItem, Scene, TitleItem};

/// 타이틀 메뉴 첫 줄의 자리(화면 높이 비율) — `tools/test/touch.js`가 같은 값으로 탭한다.
const TITLE_MENU_Y: f32 = 0.42;

/// 지금 만든 스테이지(나중에 스테이지 고르기가 생기면 여기서 고른다).
const FIRST_STAGE: &StageDef = &sanctuary::DEF;

pub struct App {
    scene: Scene,
    cursor: usize,
    /// 지난 프레임에 그린 메뉴 줄 자리(탭 판정).
    menu_rects: Vec<Rect>,
    input: Input,
    t: Tuning,
    lab: Lab,
    renderer: Renderer,
    ui: Ui,
    cam: Camera,
    world: World,
    run: StageRun,
    #[cfg(feature = "dev")]
    sc: Scenario,
    ending: f32,
    acc: f32,
    /// 개발 줄(F3).
    dev_overlay: bool,
    perf: Perf,
    sums: (f64, f64, f64, u32),
}

impl App {
    pub fn new() -> Self {
        let t = Tuning::default();
        let (run, world) = StageRun::start(FIRST_STAGE);
        let cam = Camera::new(world.player.pos, &t);
        App {
            scene: Scene::Title,
            cursor: 0,
            menu_rects: Vec::new(),
            input: Input::new(),
            lab: Lab::new(),
            renderer: Renderer::new(),
            ui: Ui::new(),
            cam,
            world,
            run,
            t,
            #[cfg(feature = "dev")]
            sc: Scenario::Boss,
            ending: 0.0,
            acc: 0.0,
            dev_overlay: false,
            perf: Perf::default(),
            sums: (0.0, 0.0, 0.0, 0),
        }
    }

    fn restart(&mut self) {
        let (run, world) = StageRun::start(FIRST_STAGE);
        self.run = run;
        self.world = world;
        self.cam = Camera::new(self.world.player.pos, &self.t);
    }

    fn go(&mut self, scene: Scene) {
        if scene != self.scene {
            self.cursor = 0;
            self.menu_rects.clear();
        }
        self.scene = scene;
    }

    /// 한 프레임.
    pub fn frame(&mut self) {
        let cpu_start = get_time();
        self.input.poll(self.lab.open, self.scene.is_menu());
        let dt = get_frame_time().min(MAX_FRAME_DT);
        self.acc += dt;
        while self.acc >= TICK_DT {
            let a = self.input.take();
            self.tick(a);
            self.acc -= TICK_DT;
        }
        // 글자 아틀라스를 그리기 전에 다 굽는다(프레임 중간에 키우면 앞서 쌓은 글자가 지워진 텍스처를 가리킨다).
        crate::text::warm();
        self.draw();
        // 그리기 호출 수는 우리 배치가 보낸 수로 센다. macroquad `telemetry::capture_frame`은 잡은 프레임의 호출마다
        // 128×128 텍스처를 만들고 지우지 않아(0.4.16 `telemetry::track_drawcall`) 오래 켜 두면 GPU 메모리가 차서 꺼졌다(0020).
        self.perf.draws = self.renderer.batches();
        self.perf.draws_max = self.perf.draws_max.max(self.perf.draws);
        let cpu = get_time() - cpu_start;
        let (cs, cm, fs, n) = &mut self.sums;
        *cs += cpu;
        *cm = cm.max(cpu);
        *fs += get_frame_time() as f64;
        *n += 1;
        if n.is_multiple_of(60) {
            self.perf.cpu_avg_ms = (*cs / 60.0 * 1000.0) as f32;
            self.perf.cpu_max_ms = (*cm * 1000.0) as f32;
            self.perf.frame_avg_ms = (*fs / 60.0 * 1000.0) as f32;
            *cs = 0.0;
            *cm = 0.0;
            *fs = 0.0;
        }
    }

    /// 메뉴 한 틱: 커서·탭·확인 → 고른 줄.
    fn menu_pick(&mut self, a: &Actions, n: usize) -> Option<usize> {
        ui::menu_input(&mut self.cursor, n, a.menu_up, a.menu_down);
        if let Some(p) = a.tap {
            if let Some(i) = ui::hit(&self.menu_rects, p) {
                self.cursor = i;
                return Some(i);
            }
        }
        a.confirm.then_some(self.cursor.min(n.saturating_sub(1)))
    }

    fn tick(&mut self, a: Actions) {
        if a.dev_overlay {
            self.dev_overlay = !self.dev_overlay;
        }
        if a.lab_toggle {
            self.lab.open = !self.lab.open;
        }
        match self.scene {
            Scene::Title => {
                let items = flow::title_items();
                let lang_row = items.iter().position(|i| *i == TitleItem::Language);
                if (a.menu_left || a.menu_right) && Some(self.cursor) == lang_row {
                    i18n::set_lang(i18n::lang().next());
                }
                if let Some(i) = self.menu_pick(&a, items.len()) {
                    match items[i] {
                        TitleItem::Start => {
                            self.restart();
                            self.go(Scene::Play);
                        }
                        TitleItem::Language => i18n::set_lang(i18n::lang().next()),
                        #[cfg(feature = "dev")]
                        TitleItem::SkipToGate => {
                            self.restart();
                            self.run.dev_skip_to_gate(&mut self.world);
                            self.cam = Camera::new(self.world.player.pos, &self.t);
                            self.go(Scene::Play);
                        }
                        #[cfg(feature = "dev")]
                        TitleItem::Sandbox => {
                            self.world = scenario::build(self.sc);
                            self.cam = Camera::new(self.world.player.pos, &self.t);
                            self.go(Scene::Sandbox);
                        }
                    }
                }
                self.follow_camera();
            }
            Scene::Pause => {
                if a.pause || a.back {
                    let next = if a.pause {
                        flow::pause(self.scene)
                    } else {
                        flow::back(self.scene)
                    };
                    self.go(next);
                } else if let Some(i) = self.menu_pick(&a, flow::PAUSE_ITEMS.len()) {
                    let (next, rebuild) = flow::choose_pause(flow::PAUSE_ITEMS[i]);
                    if rebuild {
                        self.restart();
                    }
                    self.go(next);
                }
            }
            Scene::Result { .. } => {
                if let Some(i) = self.menu_pick(&a, flow::RESULT_ITEMS.len()) {
                    let (next, rebuild) = flow::choose_result(flow::RESULT_ITEMS[i]);
                    if rebuild {
                        self.restart();
                    }
                    self.go(next);
                }
            }
            Scene::Ending { won } => {
                self.ending += TICK_DT;
                self.follow_camera();
                if self.ending >= flow::ENDING_TIME {
                    self.go(Scene::Result { won });
                }
            }
            Scene::Play => {
                if a.pause {
                    self.go(flow::pause(self.scene));
                    return;
                }
                let a = self.lab_filter(a);
                self.step_world(&a);
                let out = self.run.tick(&mut self.world, TICK_DT);
                if let Some(next) = flow::after_stage(out) {
                    self.ending = 0.0;
                    self.go(next);
                }
            }
            #[cfg(feature = "dev")]
            Scene::Sandbox => {
                if a.pause {
                    self.go(Scene::Title);
                    return;
                }
                if let Some(n) = a.scenario {
                    self.sc = if n == 0 {
                        scenario::next(self.sc)
                    } else {
                        scenario::by_number(n).unwrap_or(self.sc)
                    };
                    self.world = scenario::build(self.sc);
                    self.cam = Camera::new(self.world.player.pos, &self.t);
                }
                let a = self.lab_filter(a);
                self.step_world(&a);
            }
        }
    }

    /// 랩이 열려 있으면 커서 키는 랩이 먹고, 이동·공격·회피·락온은 그대로 게임으로(바꾸며 바로 느끼기).
    fn lab_filter(&mut self, a: Actions) -> Actions {
        if !self.lab.open {
            return a;
        }
        if let Some(text) = self.lab.update(&a, &mut self.t) {
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
    }

    fn step_world(&mut self, a: &Actions) {
        let (cam, t) = (&mut self.cam, &self.t);
        self.world.view_yaw = Some(cam.quarter_yaw_deg(t));
        use crate::tuning::{
            SHAKE_DOWN, SHAKE_HIT, SHAKE_HURT, SHAKE_ROUSE, SHAKE_SLAM, SHAKE_TURN,
        };
        for e in self.world.step(a, t) {
            let kick = match e {
                Event::Hit { .. } => SHAKE_HIT,
                Event::PlayerHurt => SHAKE_HURT,
                Event::EnemyDown { .. } => SHAKE_DOWN,
                // 땅 돌리기: 카메라가 90° 돈다(입력 기준은 다음 틱부터 새 방위 — 쥔 스틱은 월드가 그대로 둔다).
                Event::Turn { cw } => {
                    cam.turn(cw, t);
                    SHAKE_TURN
                }
                Event::Slam { .. } => SHAKE_SLAM,
                Event::Rouse { .. } => SHAKE_ROUSE,
                Event::Swing | Event::LockOn | Event::LockOff => 0.0,
            };
            cam.trauma = (cam.trauma + kick).min(1.0);
        }
        self.follow_camera();
    }

    fn follow_camera(&mut self) {
        let target = self.world.lock_target();
        self.cam.set_mode(
            if target.is_some() {
                Mode::Lock
            } else {
                Mode::Quarter
            },
            target,
            self.world.player.pos,
        );
        self.cam.update(
            TICK_DT,
            &self.t,
            self.world.player.pos,
            target,
            &self.world.obstacles,
        );
    }

    fn in_stage(&self) -> bool {
        match self.scene {
            #[cfg(feature = "dev")]
            Scene::Sandbox => false,
            _ => true,
        }
    }

    fn draw(&mut self) {
        let alpha = (self.acc / TICK_DT).clamp(0.0, 1.0);
        let stage = self.in_stage().then_some(&self.run);
        let f = self.renderer.draw(&View {
            world: &self.world,
            cam: &self.cam,
            alpha,
            t: &self.t,
            stage,
        });
        self.ui.begin(f.scale);
        let overlay = self.input.overlay(self.lab.open);
        match self.scene {
            Scene::Title => self.draw_title(&f),
            Scene::Play | Scene::Ending { .. } => {
                hud::draw(&self.ui, &f, &self.world, Some(&self.run))
            }
            Scene::Pause => {
                hud::draw(&self.ui, &f, &self.world, Some(&self.run));
                self.draw_pause(&f);
            }
            Scene::Result { won } => self.draw_result(&f, won),
            #[cfg(feature = "dev")]
            Scene::Sandbox => hud::draw(&self.ui, &f, &self.world, None),
        }
        if self.scene.ticks_world() {
            self.lab.draw(&self.t, overlay.is_some());
            if let Some(o) = &overlay {
                touch::draw(
                    o,
                    self.world.lock_target().is_some(),
                    self.world.turn_ready(&self.t),
                );
            }
        }
        if self.dev_overlay || self.dev_sandbox() {
            self.draw_dev(&f);
        }
    }

    fn dev_sandbox(&self) -> bool {
        #[cfg(feature = "dev")]
        if self.scene == Scene::Sandbox {
            return true;
        }
        false
    }

    /// 개발 줄(기본 글꼴, 영어 — 개발 전용 화면은 예외, 첫 게임 0013): 측정·시나리오.
    fn draw_dev(&self, f: &Frame) {
        let p = &self.perf;
        let mode = match self.cam.mode {
            Mode::Quarter => "QUARTER",
            Mode::Lock => "LOCK-ON",
        };
        let y = f.oy + f.h - 40.0;
        #[cfg(feature = "dev")]
        if self.scene == Scene::Sandbox {
            let e = scenario::entry(self.sc);
            draw_text(
                format!(
                    "{} [{}]  checks: {}  (1-7 scenario, ESC title)",
                    e.name, mode, e.checks
                ),
                f.ox + 10.0,
                y - 20.0,
                16.0,
                GRAY,
            );
        }
        draw_text(
            format!(
                "{} {mode}{} cam {:.0}deg fps {}  cpu {:.1}ms (max {:.1})  frame {:.1}ms  batches {} (max {})  verts {}",
                self.world.name,
                self.cam.blending().map(|e| format!(" blend {e:.2}s")).unwrap_or_default(),
                self.cam.azimuth_deg(),
                get_fps(),
                p.cpu_avg_ms,
                p.cpu_max_ms,
                p.frame_avg_ms,
                p.draws,
                p.draws_max,
                self.renderer.vertices()
            ),
            f.ox + 10.0,
            y,
            16.0,
            YELLOW,
        );
    }

    fn controls_hint(&self, cx: f32, y: f32) {
        let s = s();
        self.ui.hints(
            &[
                (input::prompt(Act::Move), s.verb_move),
                (input::prompt(Act::Attack), s.verb_attack),
                (input::prompt(Act::Dodge), s.verb_dodge),
                (input::prompt(Act::Turn), s.verb_turn),
                (input::prompt(Act::Lock), s.verb_lock),
                (input::prompt(Act::Pause), s.verb_pause),
            ],
            cx,
            y,
        );
    }

    fn menu_hint(&self, cx: f32, y: f32) {
        let s = s();
        self.ui.hints(
            &[
                (input::prompt(Act::Choose), s.verb_choose),
                (input::prompt(Act::Confirm), s.verb_confirm),
            ],
            cx,
            y,
        );
    }

    fn draw_title(&mut self, f: &Frame) {
        let s = s();
        self.ui.dim(0.45);
        let cx = f.ox + f.w * 0.5;
        let y = f.oy + f.h * 0.13;
        self.ui.text_center(s.game_title, cx, y, 3.0, AMBER);
        self.ui
            .text_center(s.title_subtitle, cx, y + self.ui.px(3.0) * 1.2, 1.0, DIM);
        let items: Vec<String> = flow::title_items()
            .iter()
            .map(|i| match i {
                TitleItem::Start => s.menu_start.to_string(),
                TitleItem::Language => fill(s.menu_language, &[s.lang_name]),
                // 개발 전용(영어 그대로, 0013).
                #[cfg(feature = "dev")]
                TitleItem::SkipToGate => "DEV: skip to the sanctum gate".to_string(),
                #[cfg(feature = "dev")]
                TitleItem::Sandbox => "DEV: PoC scenarios".to_string(),
            })
            .collect();
        self.menu_rects = self
            .ui
            .menu(&items, self.cursor, cx, f.oy + f.h * TITLE_MENU_Y);
        self.menu_hint(cx, f.oy + f.h - self.ui.px(1.0) * 3.4);
        self.controls_hint(cx, f.oy + f.h - self.ui.px(1.0) * 2.0);
    }

    fn draw_pause(&mut self, f: &Frame) {
        let s = s();
        self.ui.dim(0.55);
        let cx = f.ox + f.w * 0.5;
        self.ui
            .text_center(s.pause_title, cx, f.oy + f.h * 0.25, 2.0, INK);
        let items: Vec<String> = flow::PAUSE_ITEMS
            .iter()
            .map(|i| {
                match i {
                    PauseItem::Resume => s.menu_resume,
                    PauseItem::Restart => s.menu_restart,
                    PauseItem::ToTitle => s.menu_to_title,
                }
                .to_string()
            })
            .collect();
        self.menu_rects = self.ui.menu(&items, self.cursor, cx, f.oy + f.h * 0.45);
        self.menu_hint(cx, f.oy + f.h - self.ui.px(1.0) * 3.4);
        self.controls_hint(cx, f.oy + f.h - self.ui.px(1.0) * 2.0);
    }

    fn draw_result(&mut self, f: &Frame, won: bool) {
        let s = s();
        self.ui.dim(0.6);
        let cx = f.ox + f.w * 0.5;
        let y = f.oy + f.h * 0.28;
        if won {
            self.ui.text_center(s.clear_title, cx, y, 2.0, AMBER);
            let line = hud::clear_line(&self.run, &self.world);
            self.ui
                .text_center(&line, cx, y + self.ui.px(2.0) * 1.6, 1.0, INK);
        } else {
            self.ui.text_center(s.defeat_title, cx, y, 2.0, INK);
            self.ui
                .text_center(s.defeat_sub, cx, y + self.ui.px(2.0) * 1.6, 1.0, DIM);
        }
        let items: Vec<String> = flow::RESULT_ITEMS
            .iter()
            .map(|i| {
                match i {
                    ResultItem::Retry => s.menu_retry,
                    ResultItem::ToTitle => s.menu_to_title,
                }
                .to_string()
            })
            .collect();
        self.menu_rects = self.ui.menu(&items, self.cursor, cx, f.oy + f.h * 0.55);
        self.menu_hint(cx, f.oy + f.h - self.ui.px(1.0) * 2.0);
    }
}
