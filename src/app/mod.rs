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

/// 가장자리 화살(0060 W1): 갈 곳이 화면 밖에 이만큼(초) 있으면 띄운다(문이 열리는 순간·잠깐 돌아본 것엔 안 뜨게), 테두리에서 들인 몫(NDC).
const GOAL_ARROW_DELAY: f32 = 1.0;
const GOAL_ARROW_INSET: f32 = 0.12;

/// 지금 만든 스테이지(나중에 스테이지 고르기가 생기면 여기서 고른다).
const FIRST_STAGE: &StageDef = &sanctuary::DEF;

pub struct App {
    scene: Scene,
    cursor: usize,
    /// 지난 프레임에 그린 메뉴 줄·▲▼ 자리(탭 판정).
    menu_hits: ui::MenuHits,
    /// 지금 화면의 굴리는 목록 창(메뉴·조작 설정 공통, 장면이 바뀌면 처음부터).
    view: ui::ListView,
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
    /// 지난 프레임에 그린 화면 자리(커서 → 땅, 0058).
    frame: Option<Frame>,
    /// OS 커서를 숨겼나(싸움 중 마우스 조준이면 숨기고 조준 표시를 그린다, 0058).
    mouse_hidden: bool,
    /// 갈 곳이 화면 밖에 있은 초(0060 W1 — 잠깐 뒤에 가장자리 화살).
    goal_off: f32,
}

impl App {
    pub fn new() -> Self {
        let t = Tuning::default();
        // 저장한 설정(0024·0052·0058): 언어·키 배치·흔들림·무기·마우스 조준.
        let mut weapon = weapons::Id::Sword;
        let mut shake = crate::prefs::SHAKE_DEFAULT;
        if let Some(p) = crate::prefs::load() {
            if let Some(l) = p.lang {
                i18n::set_lang(l);
            }
            input::set_bindings(p.bindings);
            input::set_mouse_aim(p.mouse);
            shake = p.shake;
            weapon = weapons::IDS[(p.weapon as usize).min(weapons::IDS.len() - 1)];
        }
        let (run, mut world) = StageRun::start(FIRST_STAGE);
        world.set_weapon(weapon);
        let cam = Camera::new(world.player.pos, &t);
        App {
            scene: Scene::Title,
            cursor: 0,
            menu_hits: ui::MenuHits::default(),
            view: ui::ListView::default(),
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
            frame: None,
            mouse_hidden: false,
            goal_off: 0.0,
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
            mouse: input::mouse_aim(),
            shake: self.shake,
            weapon: weapons::IDS
                .iter()
                .position(|w| *w == self.weapon)
                .unwrap_or(0) as u8,
        });
    }

    /// 마우스 조준 줄(0058): 켬/끔은 뒤집기, 쉼 시간·표시 크기는 좌우로 한 칸(끝에서 돈다), 저장.
    fn step_mouse(&mut self, row: SettingsRow, step: i32) {
        let mut m = input::mouse_aim();
        let cycle = |v: u8, n: usize| (v as i32 + step).rem_euclid(n as i32) as u8;
        match row {
            SettingsRow::MouseAim => m.on = !m.on,
            SettingsRow::CursorRest => m.rest = cycle(m.rest, input::REST_SECS.len()),
            SettingsRow::Reticle => m.size = cycle(m.size, input::RETICLE_SIZES as usize),
            _ => return,
        }
        input::set_mouse_aim(m);
        self.save_prefs();
    }

    fn next_lang(&mut self) {
        i18n::set_lang(i18n::lang().next());
        self.save_prefs();
    }

    fn go(&mut self, scene: Scene) {
        if scene != self.scene {
            self.cursor = 0;
            self.menu_hits = ui::MenuHits::default();
            self.view = ui::ListView::default();
            self.ctl = controls::Controls::default();
            self.ctl_rects.clear();
        }
        self.scene = scene;
    }

    /// 한 프레임.
    pub fn frame(&mut self) {
        let cpu_start = get_time();
        // 터치의 상황 버튼(0053): 쉼터 곁이면 점프가 쉬기, 락온할 적이 있거나 락온 중이면 락온, 마실 수 있으면 등잔.
        let play = self.scene.ticks_world();
        let pl = &self.world.player;
        self.input.set_touch_ctx(crate::touch::Ctx {
            rest: play && self.world.interact_here,
            lock: play && (self.world.lock.is_some() || self.world.lock_candidate()),
            flask: play && pl.flask > 0 && pl.hp < crate::tuning::PLAYER_HP,
        });
        self.input.poll(self.lab.open, self.scene.is_menu());
        // 싸움 중 마우스 조준이면 OS 커서 대신 조준 표시(0058). 랩은 마우스를 쓰지 않지만 화살표가 보이게 둔다.
        let hide = play
            && !self.lab.open
            && input::mouse_aim().on
            && input::device() == input::Device::Keys;
        if hide != self.mouse_hidden {
            show_mouse(!hide);
            self.mouse_hidden = hide;
        }
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

    /// 메뉴 한 틱: 커서·굴리기·탭·확인 → 고른 줄. 굴리기 = 끌기·휠·▲▼ 탭(창을 옮기고 커서를 창 안으로).
    fn menu_pick(&mut self, a: &Actions, n: usize) -> Option<usize> {
        ui::menu_input(&mut self.cursor, n, a.menu_up, a.menu_down);
        let arrow = a.tap.and_then(|p| self.menu_hits.arrow_at(p));
        self.view
            .scroll((a.drag, a.wheel + arrow.unwrap_or(0)), &mut self.cursor, n);
        if let (Some(p), None) = (a.tap, arrow) {
            if let Some(i) = ui::hit(&self.menu_hits.rows, p) {
                self.cursor = i;
                return Some(i);
            }
        }
        a.confirm.then_some(self.cursor.min(n.saturating_sub(1)))
    }

    /// 화면 아래 안내 줄 위까지(메뉴가 굴리는 아래 끝).
    fn menu_bottom(&self, f: &Frame) -> f32 {
        f.oy + f.h - self.ui.px(1.0) * 2.8
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
                let rows = flow::settings_rows(crate::screen::supported(), input::has_mouse());
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
                        Some(
                            r @ (SettingsRow::MouseAim
                            | SettingsRow::CursorRest
                            | SettingsRow::Reticle),
                        ) => self.step_mouse(r, step),
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
                        r @ (SettingsRow::MouseAim
                        | SettingsRow::CursorRest
                        | SettingsRow::Reticle) => self.step_mouse(r, 1),
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
                    // 굴리기(끌기·휠·▲▼)는 메뉴와 같다 — 커서 = 고른 줄.
                    let arrow = a.tap.and_then(|p| self.menu_hits.arrow_at(p));
                    let n = controls::rows().len();
                    self.view
                        .scroll((a.drag, a.wheel + arrow.unwrap_or(0)), &mut self.ctl.row, n);
                    if let (Some(p), None) = (a.tap, arrow) {
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
                let a = self.cursor_aim(a);
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
                let a = self.cursor_aim(a);
                self.step_world(&a);
            }
        }
    }

    /// 커서 → 땅 위의 점(0058): 지난 프레임 화면 자리로 NDC, 카메라 리그의 시선이 플레이어 발 높이 면에 닿는 점.
    /// 락온 중이면 쓰지 않는다(오른쪽 스틱처럼 — 대상을 본다). 화면 밖이면 없음.
    fn cursor_aim(&self, mut a: Actions) -> Actions {
        a.aim_at = None;
        let (Some(c), Some(f)) = (a.cursor, self.frame) else {
            return a;
        };
        if self.world.lock_target().is_some() || f.w < 1.0 || f.h < 1.0 {
            return a;
        }
        let ndc = vec2(
            (c.x - f.ox) / f.w * 2.0 - 1.0,
            1.0 - (c.y - f.oy) / f.h * 2.0,
        );
        if ndc.x.abs() > 1.0 || ndc.y.abs() > 1.0 {
            return a;
        }
        let rig = &self.cam.rig;
        let dir = crate::camera::screen_ray(rig, f.w / f.h, ndc);
        a.aim_at = crate::camera::ground_hit(rig.pos, dir, self.world.player.pos.y)
            .map(|p| vec2(p.x, p.z));
        a
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
            cursor: a.cursor,
            attack: a.attack,
            attack_held: a.attack_held,
            cast: a.cast,
            cast_held: a.cast_held,
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
        self.frame = Some(f);
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
        if self.scene == Scene::Play {
            self.draw_goal_arrow(&f);
        }
        if self.scene.ticks_world() {
            if self.mouse_hidden && self.world.lock_target().is_none() {
                if let Some(c) = self.input.cursor() {
                    hud::draw_reticle(&f, c, input::mouse_aim().size);
                }
            }
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

    /// 갈 곳이 화면 밖이면(`GOAL_ARROW_DELAY` 넘게) 가장자리 화살(0060 W1). 화면 안에 들어오면 빛기둥이 맡는다.
    fn draw_goal_arrow(&mut self, f: &Frame) {
        let mark = self.run.goal().and_then(|g| {
            let (ndc, front) =
                crate::camera::world_to_ndc(&self.cam.rig, f.w / f.h.max(1.0), g + Vec3::Y);
            crate::camera::edge_marker(ndc, front, GOAL_ARROW_INSET)
        });
        self.goal_off = match mark {
            Some(_) => self.goal_off + get_frame_time(),
            None => 0.0,
        };
        if let (Some((at, ang)), true) = (mark, self.goal_off > GOAL_ARROW_DELAY) {
            let p = vec2(
                f.ox + (at.x + 1.0) * 0.5 * f.w,
                f.oy + (1.0 - at.y) * 0.5 * f.h,
            );
            hud::draw_goal_arrow(f, p, -ang, self.world.time);
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
        let bottom = self.menu_bottom(f);
        self.menu_hits = self.ui.menu(
            &items,
            self.cursor,
            &mut self.view,
            cx,
            f.oy + f.h * TITLE_MENU_Y,
            bottom,
        );
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
        let bottom = self.menu_bottom(f);
        self.menu_hits = self.ui.menu(
            &items,
            self.cursor,
            &mut self.view,
            cx,
            f.oy + f.h * 0.45,
            bottom,
        );
        self.menu_hint(cx, f.oy + f.h - self.ui.px(1.0) * 2.2, f.w * 0.94);
    }

    /// 설정(0052): 값 줄은 "이름: 값", 좌우·확인으로 바꾼다.
    fn draw_settings(&mut self, f: &Frame) {
        let s = s();
        let mouse = input::mouse_aim();
        self.ui.dim(0.6);
        let cx = f.ox + f.w * 0.5;
        let rows = flow::settings_rows(crate::screen::supported(), input::has_mouse());
        // 마우스 줄 셋(0058)이 붙으면 제목·메뉴를 올리고 줄 간격을 아래 안내 위까지로 줄인다.
        let many = rows.len() > 5;
        let title_y = f.oy + f.h * if many { 0.1 } else { 0.2 };
        self.ui.text_center(s.settings_title, cx, title_y, 2.0, INK);
        let items: Vec<String> = rows
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
                SettingsRow::MouseAim => if mouse.on {
                    s.menu_mouse_aim_on
                } else {
                    s.menu_mouse_aim_off
                }
                .to_string(),
                SettingsRow::CursorRest => {
                    fill(s.menu_cursor_rest, &[&format!("{}", mouse.rest_secs())])
                }
                SettingsRow::Reticle => fill(
                    s.menu_reticle,
                    &[[s.reticle_small, s.reticle_medium, s.reticle_large]
                        [(mouse.size as usize).min(2)]],
                ),
                SettingsRow::Controls => s.menu_controls.to_string(),
                SettingsRow::Back => s.controls_back.to_string(),
            })
            .collect();
        let hint_y = f.oy + f.h - self.ui.px(1.0) * 2.2;
        let top = f.oy + f.h * if many { 0.24 } else { 0.38 };
        let bottom = hint_y - self.ui.px(1.0) * 0.6;
        self.menu_hits = self
            .ui
            .menu(&items, self.cursor, &mut self.view, cx, top, bottom);
        self.menu_hint(cx, hint_y, f.w * 0.94);
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
        // 아래 PoC 설명 줄(0.78) 위까지.
        let bottom = f.oy + f.h * 0.76;
        self.menu_hits = self.ui.menu(
            &items,
            self.cursor,
            &mut self.view,
            cx,
            f.oy + f.h * 0.3,
            bottom,
        );
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
        // 줄 간격은 아래 고정 안내(`controls_fixed`) 위까지 들어가게 조금 줄이고(1.32 → 1.15줄), 그래도 넘치면 굴린다(메뉴와 같은 `ListView`).
        let bottom = f.oy + f.h - px * 3.2 - px * 1.2;
        let n = rows.len();
        let gap = (px * 1.32)
            .min((bottom - y) / (n as f32 - 1.0 + 0.4))
            .max(px * 1.15);
        let fits = if gap * (n as f32 - 1.0 + 0.4) <= bottom - y + 1.0 {
            n
        } else {
            y += px * 0.9; // ▲ 자리
            ((bottom - y - px * 0.9) / gap).floor().max(1.0) as usize
        };
        let first = self.view.window(self.ctl.row, n, fits, gap);
        let y_top = y;
        for (i, row) in rows.iter().enumerate().skip(first).take(fits) {
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
                controls::Row::Preset(_) | controls::Row::Back => {
                    let label = match row {
                        controls::Row::Preset(controls::Preset::Mouse) => s.controls_preset_mouse,
                        controls::Row::Preset(controls::Preset::Keys) => s.controls_preset_keys,
                        _ => s.controls_back,
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
        let arrows = self.ui.scroll_arrows(
            &self.view,
            n,
            cx,
            y_top - px * 0.95,
            y_top + gap * fits as f32 - px * 0.15,
        );
        self.menu_hits = ui::MenuHits {
            rows: Vec::new(),
            arrows,
        };
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
        let bottom = self.menu_bottom(f);
        self.menu_hits = self.ui.menu(
            &items,
            self.cursor,
            &mut self.view,
            cx,
            f.oy + f.h * 0.55,
            bottom,
        );
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
