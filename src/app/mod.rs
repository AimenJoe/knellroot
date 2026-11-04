//! 앱(0019): 장면(타이틀 → 싸움 → 멈춤 → 끝 → 결과)과 프레임 루프. 흐름은 `flow.rs`의 순수 함수가 정하고 여기서는 그대로 한다.
//! 월드는 메뉴를 모르고, 스테이지(`stage`)는 월드를 지켜보며 진행만 하고, 앱이 둘을 묶어 카메라·그림·글로 보낸다.
//!
//! 루프: 프레임마다 입력 폴링 → 고정 60Hz 틱 N번(장면 → 월드 → 스테이지 → 카메라) → 가변 렌더(보간) → 글·메뉴(화면 패스).
pub mod controls;
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
use crate::weapons;
use crate::world::{Event, World};
use flow::{PauseItem, ResultItem, Scene, SettingsRow, TitleItem};

/// 타이틀 메뉴 첫 줄의 자리(화면 높이 비율) — `tools/test/touch.js`가 같은 값으로 탭한다.
const TITLE_MENU_Y: f32 = 0.37;

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
    /// 조작 설정 화면(0024)과 지난 프레임의 칸 자리(탭: (자리, 줄, 칸)).
    ctl: controls::Controls,
    ctl_rects: Vec<(Rect, usize, usize)>,
    perf: Perf,
    sums: (f64, f64, f64, u32),
    /// 든 무기(0051) — 다시 시작해도 그대로, 설정과 함께 저장.
    weapon: weapons::Id,
    /// 화면 흔들림 0..=10(설정, 0052).
    shake: u8,
}

impl App {
    pub fn new() -> Self {
        let t = Tuning::default();
        // 저장한 설정(0024·0052): 언어·키 배치·흔들림·무기.
        let mut weapon = weapons::Id::Sword;
        let mut shake = crate::prefs::SHAKE_DEFAULT;
        if let Some(p) = crate::prefs::load() {
            if let Some(l) = p.lang {
                i18n::set_lang(l);
            }
            input::set_bindings(p.bindings);
            shake = p.shake;
            weapon = weapons::IDS[(p.weapon as usize).min(weapons::IDS.len() - 1)];
        }
        let (run, mut world) = StageRun::start(FIRST_STAGE);
        world.set_weapon(weapon);
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
            ctl: controls::Controls::default(),
            ctl_rects: Vec::new(),
            perf: Perf::default(),
            sums: (0.0, 0.0, 0.0, 0),
            weapon,
            shake,
        }
    }

    fn restart(&mut self) {
        let (run, world) = StageRun::start(FIRST_STAGE);
        self.run = run;
        self.world = world;
        self.world.set_weapon(self.weapon);
        self.cam = Camera::new(self.world.player.pos, &self.t);
    }

    /// 무기를 바꾼다(0051, 멈춤 메뉴): 지금 판에 바로, 저장.
    fn next_weapon(&mut self, step: i32) {
        self.weapon = self.weapon.next(step);
        self.world.set_weapon(self.weapon);
        self.save_prefs();
    }

    /// 설정을 저장한다(언어·키 배치를 바꿀 때마다).
    fn save_prefs(&self) {
        crate::prefs::save(&crate::prefs::Prefs {
            lang: Some(i18n::lang()),
            bindings: input::bindings(),
            shake: self.shake,
            weapon: weapons::IDS
                .iter()
                .position(|w| *w == self.weapon)
                .unwrap_or(0) as u8,
        });
    }

    fn next_lang(&mut self) {
        i18n::set_lang(i18n::lang().next());
        self.save_prefs();
    }

    fn go(&mut self, scene: Scene) {
        if scene != self.scene {
            self.cursor = 0;
            self.menu_rects.clear();
            self.ctl = controls::Controls::default();
            self.ctl_rects.clear();
        }
        self.scene = scene;
    }

    /// 한 프레임.
    pub fn frame(&mut self) {
        let cpu_start = get_time();
        // 쉼터 곁이면 터치에 상호작용 버튼(0032).
        self.input
            .set_interact_shown(self.scene == Scene::Play && self.run.near_rest.is_some());
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
        #[cfg(feature = "dev")]
        if a.ui_check {
            crate::ui::check::toggle();
        }
        if a.lab_toggle {
            self.lab.open = !self.lab.open;
        }
        match self.scene {
            Scene::Title => {
                let items = flow::title_items();
                if let Some(i) = self.menu_pick(&a, items.len()) {
                    match items[i] {
                        TitleItem::Start => {
                            crate::screen::touch_enter();
                            self.restart();
                            self.go(Scene::Play);
                        }
                        TitleItem::Settings => self.go(Scene::Settings { from_pause: false }),
                        #[cfg(feature = "dev")]
                        TitleItem::Dev => self.go(Scene::Dev),
                    }
                }
                self.follow_camera();
            }
            Scene::Settings { from_pause } => {
                let rows = flow::settings_rows(crate::screen::supported());
                let row = rows.get(self.cursor).copied();
                let step = a.menu_right as i32 - a.menu_left as i32;
                if a.back || a.pause {
                    self.go(flow::back(self.scene));
                    return;
                }
                if step != 0 {
                    match row {
                        Some(SettingsRow::Language) => self.next_lang(),
                        Some(SettingsRow::Shake) => {
                            self.shake = (self.shake as i32 + step).clamp(0, 10) as u8;
                            self.save_prefs();
                        }
                        _ => {}
                    }
                }
                if let Some(i) = self.menu_pick(&a, rows.len()) {
                    match rows[i] {
                        SettingsRow::Language => self.next_lang(),
                        SettingsRow::Fullscreen => crate::screen::toggle(),
                        SettingsRow::Shake => {
                            self.shake = if self.shake >= 10 { 0 } else { self.shake + 1 };
                            self.save_prefs();
                        }
                        SettingsRow::Controls => self.go(Scene::Controls { from_pause }),
                        SettingsRow::Back => self.go(flow::back(self.scene)),
                    }
                }
                if !from_pause {
                    self.follow_camera();
                }
            }
            #[cfg(feature = "dev")]
            Scene::Dev => {
                let items = dev_items();
                if a.back || a.pause {
                    self.go(Scene::Title);
                    return;
                }
                let step = a.menu_right as i32 - a.menu_left as i32;
                if step != 0 {
                    match items.get(self.cursor) {
                        Some(DevItem::Scenario) => {
                            self.sc = if step > 0 {
                                scenario::next(self.sc)
                            } else {
                                (0..scenario::LIST.len() - 1)
                                    .fold(self.sc, |s, _| scenario::next(s))
                            }
                        }
                        Some(DevItem::Weapon) => self.next_weapon(step),
                        _ => {}
                    }
                }
                if let Some(i) = self.menu_pick(&a, items.len()) {
                    match items[i] {
                        DevItem::Gate => {
                            self.restart();
                            self.run.dev_skip_to_gate(&mut self.world);
                            self.cam = Camera::new(self.world.player.pos, &self.t);
                            self.go(Scene::Play);
                        }
                        DevItem::Scenario => {
                            let sc = self.sc;
                            self.world = scenario::build(sc);
                            self.world.set_weapon(self.weapon);
                            self.cam = Camera::new(self.world.player.pos, &self.t);
                            self.go(Scene::Sandbox);
                        }
                        DevItem::Weapon => self.next_weapon(1),
                        DevItem::Back => self.go(Scene::Title),
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
                } else if (a.menu_left || a.menu_right)
                    && flow::PAUSE_ITEMS.get(self.cursor) == Some(&PauseItem::Weapon)
                {
                    self.next_weapon(if a.menu_right { 1 } else { -1 });
                } else if let Some(i) = self.menu_pick(&a, flow::PAUSE_ITEMS.len()) {
                    if flow::PAUSE_ITEMS[i] == PauseItem::Weapon {
                        self.next_weapon(1);
                    }
                    let (next, rebuild) = flow::choose_pause(flow::PAUSE_ITEMS[i]);
                    if rebuild {
                        self.restart();
                    }
                    self.go(next);
                }
            }
            Scene::Controls { .. } => {
                let b = input::bindings();
                let mut out = self.ctl.tick(&a, b, TICK_DT);
                if self.ctl.capture.is_none() {
                    if let Some(p) = a.tap {
                        if let Some(&(_, row, col)) =
                            self.ctl_rects.iter().find(|(r, _, _)| r.contains(p))
                        {
                            out = self.ctl.pick(row, col, b);
                        }
                    }
                }
                match out {
                    controls::Out::Stay => {}
                    controls::Out::Changed(nb) => {
                        input::set_bindings(nb);
                        self.save_prefs();
                    }
                    controls::Out::Back => self.go(flow::back(self.scene)),
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
                if out == crate::stage::StageOut::Respawned {
                    // 쉼터에서 다시 일어섰다(0032): 카메라를 그 자리로 바로(멀리서 날아오지 않게).
                    self.cam = Camera::new(self.world.player.pos, &self.t);
                }
                if let Some(next) = flow::after_stage(out) {
                    self.ending = 0.0;
                    self.go(next);
                }
            }
            #[cfg(feature = "dev")]
            Scene::Sandbox => {
                if a.pause {
                    self.go(Scene::Dev);
                    return;
                }
                if let Some(n) = a.scenario {
                    self.sc = if n == 0 {
                        scenario::next(self.sc)
                    } else {
                        scenario::by_number(n).unwrap_or(self.sc)
                    };
                    self.world = scenario::build(self.sc);
                    self.world.set_weapon(self.weapon);
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
            attack_held: a.attack_held,
            flask: a.flask,
            dodge: a.dodge,
            parry: a.parry,
            jump: a.jump,
            lock: a.lock,
            turn_right: a.turn_right,
            turn_left: a.turn_left,
            ..Actions::default()
        }
    }

    fn step_world(&mut self, a: &Actions) {
        let (cam, t) = (&mut self.cam, &self.t);
        self.world.view_yaw = Some(cam.quarter_yaw_deg(t));
        use crate::tuning::{
            SHAKE_CHARGED, SHAKE_CRIT, SHAKE_DOWN, SHAKE_HIT, SHAKE_HURT, SHAKE_PARRY,
            SHAKE_PLUNGE, SHAKE_ROUSE, SHAKE_SLAM, SHAKE_SMASH, SHAKE_TURN,
        };
        for e in self.world.step(a, t) {
            let kick = match e {
                Event::Hit { .. } => SHAKE_HIT,
                Event::PlayerHurt => SHAKE_HURT,
                Event::EnemyDown { .. } => SHAKE_DOWN,
                // 땅 돌리기: 카메라가 90° 돈다(입력 기준은 다음 틱부터 새 방위 — 쥔 스틱은 월드가 그대로 둔다).
                Event::Turn { quarters } => {
                    cam.turn(quarters, t);
                    SHAKE_TURN
                }
                Event::Slam { .. } => SHAKE_SLAM,
                Event::Rouse { .. } => SHAKE_ROUSE,
                Event::Parry { .. } => SHAKE_PARRY,
                Event::Crit { .. } => SHAKE_CRIT,
                Event::Plunge { .. } => SHAKE_PLUNGE,
                Event::Smash { .. } => SHAKE_SMASH,
                Event::Charged => SHAKE_CHARGED,
                Event::Gulp { .. } | Event::FlaskDenied => 0.0,
                Event::Cast { .. } | Event::CastDenied | Event::Throw => 0.0,
                Event::Break { .. } => SHAKE_SMASH,
                Event::Aftershock { .. } | Event::Boom { .. } => SHAKE_PLUNGE,
                Event::Perfect { .. } => SHAKE_CHARGED,
                Event::Swing | Event::LockOn | Event::LockOff => 0.0,
            };
            // 화면 흔들림 설정(0052): 0..=10 → 0..100%.
            cam.trauma = (cam.trauma + kick * self.shake as f32 / 10.0).min(1.0);
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
        self.ui.begin(f.ui);
        let overlay = self.input.overlay(self.lab.open);
        match self.scene {
            Scene::Title => self.draw_title(&f),
            Scene::Play | Scene::Ending { .. } => {
                hud::draw(&self.ui, &f, &self.world, Some(&self.run), false)
            }
            Scene::Pause => {
                hud::draw(&self.ui, &f, &self.world, Some(&self.run), true);
                self.draw_pause(&f);
            }
            Scene::Result { won } => self.draw_result(&f, won),
            Scene::Controls { .. } => self.draw_controls(&f),
            Scene::Settings { from_pause } => {
                if from_pause {
                    hud::draw(&self.ui, &f, &self.world, Some(&self.run), true);
                }
                self.draw_settings(&f);
            }
            #[cfg(feature = "dev")]
            Scene::Dev => self.draw_dev_menu(&f),
            #[cfg(feature = "dev")]
            Scene::Sandbox => hud::draw(&self.ui, &f, &self.world, None, false),
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
        #[cfg(feature = "dev")]
        crate::ui::check::end_frame(
            Rect::new(0.0, 0.0, screen_width(), screen_height()),
            Rect::new(f.ox, f.oy, f.w, f.h),
        );
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

    /// 키 안내 한 줄(지금 장치·지금 배치의 글자, 0024).
    /// 넓으면 `fit`으로 자른다(키는 키캡).
    fn hint_line(&self, pairs: &[(Act, &str)], cx: f32, y: f32, max_w: f32) {
        let named: Vec<(String, &str)> =
            pairs.iter().map(|(a, v)| (input::prompt(*a), *v)).collect();
        let refs: Vec<(&str, &str)> = named.iter().map(|(k, v)| (k.as_str(), *v)).collect();
        self.ui.hints(&refs, cx, y, max_w)
    }

    /// 메뉴 안내(고르기·확인). 전체 조작 줄은 두지 않는다 — 조작 설정 화면에 있다(nail 0153: 안내는 짧게).
    fn menu_hint(&self, cx: f32, y: f32, max_w: f32) {
        let s = s();
        self.hint_line(
            &[(Act::Choose, s.verb_choose), (Act::Confirm, s.verb_confirm)],
            cx,
            y,
            max_w,
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
                TitleItem::Settings => s.menu_settings.to_string(),
                // 개발 전용(영어 그대로, 0013).
                #[cfg(feature = "dev")]
                TitleItem::Dev => "DEV".to_string(),
            })
            .collect();
        self.menu_rects = self
            .ui
            .menu(&items, self.cursor, cx, f.oy + f.h * TITLE_MENU_Y);
        self.menu_hint(cx, f.oy + f.h - self.ui.px(1.0) * 2.2, f.w * 0.94);
        // 버전 글자(오른쪽 위) — 받은 판이 무엇인지 사진 한 장으로 알게. 개발판은 표시를 붙인다.
        let ver = if cfg!(feature = "dev") {
            concat!("v", env!("CARGO_PKG_VERSION"), " dev")
        } else {
            concat!("v", env!("CARGO_PKG_VERSION"))
        };
        let w = self.ui.width(ver, 1.0);
        self.ui.text(
            ver,
            f.ox + f.w - w - self.ui.px(0.6),
            f.oy + self.ui.px(0.5),
            1.0,
            DIM,
        );
    }

    fn draw_pause(&mut self, f: &Frame) {
        let s = s();
        self.ui.dim(0.55);
        let cx = f.ox + f.w * 0.5;
        self.ui
            .text_center(s.pause_title, cx, f.oy + f.h * 0.25, 2.0, INK);
        let items: Vec<String> = flow::PAUSE_ITEMS
            .iter()
            .map(|i| match i {
                PauseItem::Resume => s.menu_resume.to_string(),
                PauseItem::Weapon => fill(s.menu_weapon, &[weapon_name(self.weapon)]),
                PauseItem::Settings => s.menu_settings.to_string(),
                PauseItem::Restart => s.menu_restart.to_string(),
                PauseItem::ToTitle => s.menu_to_title.to_string(),
            })
            .collect();
        self.menu_rects = self.ui.menu(&items, self.cursor, cx, f.oy + f.h * 0.45);
        self.menu_hint(cx, f.oy + f.h - self.ui.px(1.0) * 2.2, f.w * 0.94);
    }

    /// 설정(0052): 값 줄은 "이름: 값", 좌우·확인으로 바꾼다.
    fn draw_settings(&mut self, f: &Frame) {
        let s = s();
        self.ui.dim(0.6);
        let cx = f.ox + f.w * 0.5;
        self.ui
            .text_center(s.settings_title, cx, f.oy + f.h * 0.2, 2.0, INK);
        let items: Vec<String> = flow::settings_rows(crate::screen::supported())
            .iter()
            .map(|r| match r {
                SettingsRow::Language => fill(s.menu_language, &[s.lang_name]),
                SettingsRow::Fullscreen => if crate::screen::active() {
                    s.menu_fullscreen_on
                } else {
                    s.menu_fullscreen_off
                }
                .to_string(),
                SettingsRow::Shake => {
                    fill(s.menu_shake, &[&format!("{}%", self.shake as u32 * 10)])
                }
                SettingsRow::Controls => s.menu_controls.to_string(),
                SettingsRow::Back => s.controls_back.to_string(),
            })
            .collect();
        self.menu_rects = self.ui.menu(&items, self.cursor, cx, f.oy + f.h * 0.38);
        self.menu_hint(cx, f.oy + f.h - self.ui.px(1.0) * 2.2, f.w * 0.94);
    }

    /// 개발 메뉴(0052): 개발판에만, 영어 그대로(0013).
    #[cfg(feature = "dev")]
    fn draw_dev_menu(&mut self, f: &Frame) {
        self.ui.dim(0.6);
        let cx = f.ox + f.w * 0.5;
        self.ui.text_center("DEV", cx, f.oy + f.h * 0.1, 2.0, AMBER);
        let items: Vec<String> = dev_items()
            .iter()
            .map(|d| match d {
                DevItem::Gate => "start at the sanctum gate (boss)".to_string(),
                DevItem::Scenario => format!("PoC: < {} >", scenario::entry(self.sc).name),
                DevItem::Weapon => format!("weapon: < {:?} >", self.weapon),
                DevItem::Back => "back".to_string(),
            })
            .collect();
        self.menu_rects = self.ui.menu(&items, self.cursor, cx, f.oy + f.h * 0.3);
        if let Some(DevItem::Scenario) = dev_items().get(self.cursor) {
            let e = scenario::entry(self.sc);
            let checks = self.ui.fit(e.checks, f.w * 0.9, 1.0);
            self.ui
                .text_center(&checks, cx, f.oy + f.h * 0.78, 1.0, DIM);
        }
        self.menu_hint(cx, f.oy + f.h - self.ui.px(1.0) * 2.2, f.w * 0.94);
    }

    /// 조작 설정(0024): 동작 | 키보드 | 패드 표, 고른 칸은 호박색, 기다리는 칸은 "누르세요… n".
    fn draw_controls(&mut self, f: &Frame) {
        let s = s();
        self.ui.dim(0.7);
        let cx = f.ox + f.w * 0.5;
        let px = self.ui.px(1.0);
        self.ui
            .text_center(s.controls_title, cx, f.oy + f.h * 0.07, 2.0, INK);
        let (x_name, x_key, x_pad) = (f.ox + f.w * 0.16, f.ox + f.w * 0.56, f.ox + f.w * 0.78);
        let mut y = f.oy + f.h * 0.17;
        self.ui.text_center(s.controls_keyboard, x_key, y, 1.0, DIM);
        self.ui.text_center(s.controls_pad, x_pad, y, 1.0, DIM);
        y += px * 1.4;
        let b = input::bindings();
        self.ctl_rects.clear();
        let rows = controls::rows();
        // 줄 간격은 아래 고정 안내(`controls_fixed`) 위까지 들어가게 줄인다 — 동작이 늘어도(0031 등잔) 겹치지 않게.
        let bottom = f.oy + f.h - px * 3.2 - px * 1.2;
        let fit = (bottom - y) / (rows.len() as f32 - 1.0 + 0.4);
        let gap = (px * 1.32).min(fit);
        for (i, row) in rows.iter().enumerate() {
            let sel = i == self.ctl.row;
            match row {
                controls::Row::Bind(bind) => {
                    let name = match bind {
                        // 꾹 눌러 강공격(0030)은 여기서 알린다 — 타이틀의 긴 조작 줄을 걷었다(nail 0153).
                        input::Bind::Attack => s.verb_attack_hint,
                        input::Bind::Dodge => s.verb_dodge,
                        input::Bind::Parry => s.verb_parry,
                        input::Bind::Lock => s.verb_lock,
                        input::Bind::TurnLeft => s.verb_turn_left,
                        input::Bind::TurnRight => s.verb_turn_right,
                        input::Bind::Jump => s.verb_jump_interact,
                        input::Bind::Flask => s.verb_flask,
                        input::Bind::Cast => s.verb_cast,
                    };
                    let name = self.ui.fit(name, x_key - x_name - px * 2.5, 1.0);
                    self.ui
                        .text(&name, x_name, y, 1.0, if sel { AMBER } else { INK });
                    for (col, x) in [(0usize, x_key), (1usize, x_pad)] {
                        let on = sel && self.ctl.col == col;
                        let label = match (on, self.ctl.capture) {
                            (true, Some(left)) => {
                                fill(s.controls_press, &[&format!("{}", left.ceil() as i32)])
                            }
                            _ if col == 0 => input::key_name(b.keys[*bind as usize]).to_string(),
                            _ => b.pad(*bind).name().to_string(),
                        };
                        let w = self.ui.width(&label, 1.0).max(px * 3.0);
                        let r = Rect::new(x - w * 0.5 - px * 0.5, y - px * 0.2, w + px, px * 1.4);
                        if on {
                            draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.88, 0.6, 0.23, 0.18));
                        }
                        self.ui
                            .text_center(&label, x, y, 1.0, if on { AMBER } else { INK });
                        self.ctl_rects.push((r, i, col));
                    }
                }
                controls::Row::Reset | controls::Row::Back => {
                    let label = if *row == controls::Row::Reset {
                        s.controls_reset
                    } else {
                        s.controls_back
                    };
                    let yy = y + px * 0.4;
                    let w = self.ui.width(label, 1.0);
                    let r = Rect::new(cx - w * 0.5 - px, yy - px * 0.2, w + px * 2.0, px * 1.4);
                    if sel {
                        draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.88, 0.6, 0.23, 0.18));
                    }
                    self.ui
                        .text_center(label, cx, yy, 1.0, if sel { AMBER } else { INK });
                    self.ctl_rects.push((r, i, 0));
                }
            }
            y += gap;
        }
        self.ui
            .text_center(s.controls_fixed, cx, f.oy + f.h - px * 3.2, 1.0, DIM);
        self.menu_hint(cx, f.oy + f.h - px * 1.8, f.w * 0.94);
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
                .text_center_wrapped(&line, cx, y + self.ui.px(2.0) * 1.6, 1.0, INK, f.w * 0.9);
        } else {
            self.ui.text_center(s.defeat_title, cx, y, 2.0, INK);
            self.ui.text_center_wrapped(
                s.defeat_sub,
                cx,
                y + self.ui.px(2.0) * 1.6,
                1.0,
                DIM,
                f.w * 0.9,
            );
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
        self.menu_hint(cx, f.oy + f.h - self.ui.px(1.0) * 2.0, f.w * 0.94);
    }
}

/// 무기 이름(0051, i18n 표).
fn weapon_name(id: weapons::Id) -> &'static str {
    let s = s();
    match id {
        weapons::Id::Sword => s.weapon_sword,
        weapons::Id::Hammer => s.weapon_hammer,
        weapons::Id::Staff => s.weapon_staff,
        weapons::Id::Crossbow => s.weapon_crossbow,
    }
}

/// 개발 메뉴 줄(0052).
#[cfg(feature = "dev")]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum DevItem {
    Gate,
    Weapon,
    /// PoC 시나리오(좌우로 고르고 확인으로 들어간다).
    Scenario,
    Back,
}

#[cfg(feature = "dev")]
fn dev_items() -> Vec<DevItem> {
    vec![
        DevItem::Gate,
        DevItem::Weapon,
        DevItem::Scenario,
        DevItem::Back,
    ]
}
