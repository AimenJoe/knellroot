//! 앱(0019): 장면(타이틀 → 싸움 → 멈춤 → 끝 → 결과)과 프레임 루프. 흐름은 `flow.rs`의 순수 함수가 정하고 여기서는 그대로 한다.
//! 월드는 메뉴를 모르고, 스테이지(`stage`)는 월드를 지켜보며 진행만 하고, 앱이 둘을 묶어 카메라·그림·글로 보낸다.
//!
//! 루프: 프레임마다 입력 폴링 → 고정 60Hz 틱 N번(장면 → 월드 → 스테이지 → 카메라) → 가변 렌더(보간) → 글·메뉴(화면 패스).
pub mod controls;
pub mod flow;

use macroquad::prelude::*;

use crate::camera::{Camera, Mode};
use crate::hud;
use crate::i18n::{self, fill, s, tr};
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
    /// 소리(0064 M5): 크기 0..=10, 소리 묶음(main이 올린 뒤 `set_sfx`), 지난 틱의 회피·적 예고(시작한 순간만 소리).
    volume: u8,
    sfx: Option<crate::sfx::Sfx>,
    was_dodging: bool,
    tele_was: Vec<bool>,
    was_waking: bool,
    /// 지난 프레임에 그린 화면 자리(커서 → 땅, 0058).
    frame: Option<Frame>,
    /// OS 커서를 숨겼나(싸움 중 마우스 조준이면 숨기고 조준 표시를 그린다, 0058).
    mouse_hidden: bool,
    /// 갈 곳이 화면 밖에 있은 초(0060 W1 — 잠깐 뒤에 가장자리 화살).
    goal_off: f32,
    /// 기록(0064 M2): 지금 것과 마지막으로 쓴 것(바뀌었을 때만 쓴다).
    progress: crate::save::Progress,
    saved: crate::save::Progress,
    /// 저장에 실패한 채인가(실패가 시작될 때 한 번만 알린다 — KB `roguelite-save-slots-and-continue`).
    save_failed: bool,
    /// 잠깐 뜨는 알림(0064 M3 — 상자에서 얻은 것 등)과 남은 초.
    toast: Option<(String, f32)>,
}

/// 알림이 떠 있는 시간(초).
const TOAST_TIME: f32 = 3.5;

/// 장비 창 줄(0064 M3): 무기, 장신구 세 칸, 돌아가기.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum EquipRow {
    Weapon,
    Charm(usize),
    Back,
}

fn equip_rows() -> Vec<EquipRow> {
    let mut v = vec![EquipRow::Weapon];
    v.extend((0..crate::charms::SLOTS).map(EquipRow::Charm));
    v.push(EquipRow::Back);
    v
}

/// 칸 `slot`에 낄 수 있는 다음(`step` ±1) 장신구: 빈 칸 + 가진 것 가운데 다른 칸에 없는 것(표 순서), 끝에서 감는다.
fn next_charm(
    owned: &[crate::charms::Id],
    worn: &[Option<crate::charms::Id>; crate::charms::SLOTS],
    slot: usize,
    step: i32,
) -> Option<crate::charms::Id> {
    let mut opts: Vec<Option<crate::charms::Id>> = vec![None];
    for d in crate::charms::TABLE.iter() {
        let elsewhere = worn
            .iter()
            .enumerate()
            .any(|(j, w)| j != slot && *w == Some(d.id));
        if owned.contains(&d.id) && !elsewhere {
            opts.push(Some(d.id));
        }
    }
    let at = opts.iter().position(|o| *o == worn[slot]).unwrap_or(0) as i32;
    opts[(at + step).rem_euclid(opts.len() as i32) as usize]
}

impl App {
    pub fn new() -> Self {
        let t = Tuning::default();
        // 저장한 설정(0024·0052·0058): 언어·키 배치·흔들림·무기·마우스 조준.
        let mut weapon = weapons::Id::Sword;
        let mut shake = crate::prefs::SHAKE_DEFAULT;
        let mut volume = crate::prefs::VOLUME_DEFAULT;
        if let Some(p) = crate::prefs::load() {
            if let Some(l) = p.lang {
                i18n::set_lang(l);
            }
            input::set_bindings(p.bindings);
            input::set_mouse_aim(p.mouse);
            shake = p.shake;
            volume = p.volume;
            weapon = weapons::IDS[(p.weapon as usize).min(weapons::IDS.len() - 1)];
        }
        let (run, mut world) = StageRun::start(FIRST_STAGE);
        world.set_weapon(weapon);
        let cam = Camera::new(world.player.pos, &t);
        let progress = crate::save::load();
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
            volume,
            sfx: None,
            was_dodging: false,
            tele_was: Vec::new(),
            was_waking: false,
            frame: None,
            mouse_hidden: false,
            goal_off: 0.0,
            progress: progress.clone(),
            saved: progress,
            save_failed: false,
            toast: None,
        }
    }

    fn restart(&mut self) {
        let (run, world) = StageRun::start(FIRST_STAGE);
        self.run = run;
        self.world = world;
        self.world.set_weapon(self.weapon);
        // 잔불은 판을 넘어 남는다(0064 M1). 장신구·성장은 능력치로, 연 상자는 비어 있다(M3).
        self.world.embers = self.progress.embers;
        self.world.apply_stats(self.progress.stats(), true);
        self.run.set_opened(&self.progress.chests);
        self.cam = Camera::new(self.world.player.pos, &self.t);
    }

    /// 소리 묶음을 받는다(0064 M5 — main이 올린 뒤). 환경음을 튼다.
    pub fn set_sfx(&mut self, sfx: crate::sfx::Sfx) {
        sfx.set_volume(self.volume as f32 / 10.0);
        sfx.ambience(true);
        self.sfx = Some(sfx);
    }

    fn snd(&self, id: crate::sfx::Id) {
        if let Some(s) = &self.sfx {
            s.play(id);
        }
    }

    fn set_volume(&mut self, v: u8) {
        self.volume = v;
        if let Some(s) = &self.sfx {
            s.set_volume(v as f32 / 10.0);
        }
        self.save_prefs();
        self.snd(crate::sfx::Id::UiMove);
    }

    /// 월드 사건·상태 변화 → 소리(0064 M5). 월드는 소리를 모른다 — 여기서만 듣는다.
    fn world_sounds(&mut self, events: &[Event]) {
        use crate::sfx::Id;
        for e in events {
            let id = match e {
                Event::Swing => Id::Swing,
                Event::Hit { .. } | Event::Crit { .. } => Id::Hit,
                Event::Plunge { .. } | Event::Smash { .. } | Event::Aftershock { .. } => Id::Hit,
                Event::Break { .. } | Event::Boom { .. } => Id::Hit,
                Event::EnemyDown { .. } => Id::Die,
                Event::PlayerHurt => Id::Hurt,
                Event::Parry { .. } => Id::Parry,
                Event::Ember { .. } => Id::Ember,
                Event::Gulp { .. } => Id::Gulp,
                Event::FlaskDenied | Event::CastDenied => Id::Deny,
                _ => continue,
            };
            self.snd(id);
        }
        // 회피를 시작한 틱.
        let dodging = matches!(self.world.player.state, crate::world::PState::Dodge { .. });
        if dodging && !self.was_dodging {
            self.snd(Id::Dodge);
        }
        self.was_dodging = dodging;
        // 적의 예고가 시작한 틱: 모양마다 다른 소리(소리만 듣고도 가르게, 0064 R4).
        let n = self.world.enemies.len();
        self.tele_was.resize(n, false);
        let mut cues = Vec::new();
        for (i, en) in self.world.enemies.iter().enumerate() {
            let tele = en.telegraph();
            if let (Some((_, shape)), false) = (tele, self.tele_was[i]) {
                cues.push(match shape {
                    crate::enemy::TeleShape::Disc { .. } => Id::TeleDisk,
                    crate::enemy::TeleShape::Sector { .. } => Id::TeleFan,
                    crate::enemy::TeleShape::Lane { .. } => Id::TeleLane,
                    crate::enemy::TeleShape::Ring { .. } => Id::TeleRing,
                });
            }
            self.tele_was[i] = tele.is_some();
        }
        for id in cues {
            self.snd(id);
        }
    }

    /// 이어하기(0064 M2): 새 판을 세우고, 기록에 쉼터가 있으면 거기서.
    fn continue_run(&mut self) {
        self.restart();
        if let Some(at) = self.progress.run {
            self.run.resume(&mut self.world, at);
            self.cam = Camera::new(self.world.player.pos, &self.t);
        }
    }

    /// 판에서 기록할 것을 옮기고(쉰 쉼터·끝낸 판), 바뀌었으면 쓴다.
    fn sync_progress(&mut self, out: crate::stage::StageOut) {
        self.progress.run = self.run.run_at();
        // 상자(0064 M3): 든 장신구를 얻는다(이미 가졌으면 잔불).
        if let Some(i) = self.run.take_opened() {
            self.snd(crate::sfx::Id::Chest);
            self.progress.chests.push(i as u32);
            match self.run.def.chest_charms.get(i).copied() {
                Some(id) if !self.progress.owned.contains(&id) => self.gain_charm(id),
                _ => {
                    self.world.embers += crate::stage::CHEST_EMBERS;
                    let n = crate::stage::CHEST_EMBERS.to_string();
                    self.toast = Some((fill(s().toast_embers, &[&n]), TOAST_TIME));
                }
            }
        }
        if out == crate::stage::StageOut::Cleared {
            self.progress.run = None;
            // 문지기를 처음 이기면 장신구 하나.
            if self.progress.clears == 0 {
                if let Some(id) = self.run.def.boss_charm {
                    if !self.progress.owned.contains(&id) {
                        self.gain_charm(id);
                    }
                }
            }
            self.progress.clears += 1;
        }
        self.progress.embers = self.world.embers;
        self.persist();
    }

    /// 장신구를 얻는다: 가진 것에 넣고, 빈 칸이 있으면 바로 끼운다(얻자마자 효과가 보이게). 알림.
    fn gain_charm(&mut self, id: crate::charms::Id) {
        self.progress.owned.push(id);
        self.progress.owned.sort();
        if let Some(slot) = self.progress.worn.iter_mut().find(|w| w.is_none()) {
            *slot = Some(id);
            self.world.apply_stats(self.progress.stats(), false);
        }
        let name = tr(crate::charms::def(id).name);
        self.toast = Some((fill(s().toast_charm, &[name]), TOAST_TIME));
    }

    /// 묻는 창을 연다(커서는 버리는 일이면 취소에서).
    fn ask(&mut self, k: flow::Ask) {
        self.go(Scene::Ask(k));
        self.cursor = flow::ask_default(k);
    }

    /// 화로 가게에서 산다(0064 M4): 잔불이 모자라면 알림만.
    fn buy(&mut self, (ware, price): (crate::growth::Ware, u32)) {
        let s = s();
        if self.world.embers < price {
            self.toast = Some((s.shop_poor.to_string(), 1.6));
            self.snd(crate::sfx::Id::Deny);
            return;
        }
        self.snd(crate::sfx::Id::Buy);
        self.world.embers -= price;
        self.progress.embers = self.world.embers;
        match ware {
            crate::growth::Ware::Charm(id) => self.gain_charm(id),
            crate::growth::Ware::Oil => {
                self.progress.growth.oil = true;
                // 쉼터에서 막 쉬었으니 늘어난 등잔도 가득.
                self.world.apply_stats(self.progress.stats(), true);
                self.toast = Some((fill(s.toast_bought, &[s.shop_oil]), TOAST_TIME));
            }
        }
        self.persist();
    }

    /// 장비 창에서 칸을 바꾼다.
    fn step_charm(&mut self, slot: usize, step: i32) {
        self.progress.worn[slot] =
            next_charm(&self.progress.owned, &self.progress.worn, slot, step);
        self.world.apply_stats(self.progress.stats(), false);
        self.persist();
    }

    /// 바뀐 것이 있으면 쓴다. 못 쓰면(사생활 보호 모드 등) 다음에 또 바뀔 때까지 다시 시도하지 않는다.
    fn persist(&mut self) {
        if self.progress != self.saved {
            let ok = crate::save::save(&self.progress);
            // 실패가 시작될 때 한 번만 알린다(저장할 때마다 알리면 화면을 덮는다).
            if !ok && !self.save_failed {
                warn!("save failed");
                self.toast = Some((s().save_failed.to_string(), TOAST_TIME));
            }
            self.save_failed = !ok;
            self.saved = self.progress.clone();
        }
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
            volume: self.volume,
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
            flask: play && pl.flask > 0 && pl.hp < pl.max_hp,
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
        let (scene0, cursor0, back0) = (self.scene, self.cursor, a.back || a.pause);
        let played0 = self.sfx.as_ref().map_or(0, |s| s.played());
        self.tick_scene(a);
        // 메뉴 소리(0064 M5, KB `game-audio-without-a-composer` "메뉴 소리는 한 곳에서"): 그 사이에 다른 소리(사기·안 됨 등)가
        // 나지 않았을 때만 — 뒤로 > 확인(장면이 바뀜) > 커서.
        let quiet = self.sfx.as_ref().map_or(0, |s| s.played()) == played0;
        if scene0.is_menu() && quiet {
            if self.scene != scene0 {
                self.snd(if back0 {
                    crate::sfx::Id::UiBack
                } else {
                    crate::sfx::Id::UiConfirm
                });
            } else if self.cursor != cursor0 {
                self.snd(crate::sfx::Id::UiMove);
            }
        }
    }

    fn tick_scene(&mut self, a: Actions) {
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
                let items = flow::title_items(self.progress.has_any());
                if let Some(i) = self.menu_pick(&a, items.len()) {
                    match items[i] {
                        TitleItem::Start => {
                            crate::screen::touch_enter();
                            self.continue_run();
                            self.go(Scene::Play);
                        }
                        TitleItem::NewGame => self.ask(flow::Ask::NewGame),
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
                        Some(SettingsRow::Volume) => {
                            self.set_volume((self.volume as i32 + step).clamp(0, 10) as u8)
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
                        SettingsRow::Volume => self.set_volume(if self.volume >= 10 {
                            0
                        } else {
                            self.volume + 1
                        }),
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
                    let (next, rebuild) = flow::choose_pause(flow::PAUSE_ITEMS[i]);
                    if rebuild {
                        self.restart();
                    }
                    match next {
                        Scene::Ask(k) => self.ask(k),
                        _ => self.go(next),
                    }
                }
            }
            Scene::Core => {
                // 핵 고르기(0064 M4): 하나를 골라야 결과로(뒤로는 건너뛰지 않는다).
                let opts = self.progress.growth.options();
                if let Some(i) = self.menu_pick(&a, opts.len()) {
                    self.progress.growth.add(opts[i]);
                    self.world.apply_stats(self.progress.stats(), true);
                    self.persist();
                    self.go(Scene::Result { won: true });
                }
                self.follow_camera();
            }
            Scene::Shop => {
                let wares = crate::growth::wares(&self.progress.owned, &self.progress.growth);
                if a.back || a.pause {
                    self.go(flow::back(self.scene));
                    return;
                }
                if let Some(i) = self.menu_pick(&a, wares.len() + 1) {
                    match wares.get(i).copied() {
                        // 모자라면 묻지 않고 알림만(nail 가게와 같다 — 흐린 줄).
                        Some((_, price)) if self.world.embers < price => {
                            self.toast = Some((s().shop_poor.to_string(), 1.6));
                            self.snd(crate::sfx::Id::Deny);
                        }
                        Some(_) => self.ask(flow::Ask::Buy(i)),
                        None => self.go(flow::back(self.scene)),
                    }
                }
                self.follow_camera();
            }
            Scene::Ask(k) => {
                if a.back || a.pause {
                    self.go(flow::back(self.scene));
                    return;
                }
                if let Some(i) = self.menu_pick(&a, 2) {
                    let yes = i == 0;
                    let (next, rebuild) = flow::answer(k, yes);
                    if yes {
                        match k {
                            flow::Ask::NewGame => {
                                crate::save::wipe();
                                self.progress = crate::save::Progress::default();
                                self.saved = self.progress.clone();
                                crate::screen::touch_enter();
                            }
                            flow::Ask::Buy(w) => {
                                let wares = crate::growth::wares(
                                    &self.progress.owned,
                                    &self.progress.growth,
                                );
                                if let Some(&w) = wares.get(w) {
                                    self.buy(w);
                                }
                            }
                            flow::Ask::Restart | flow::Ask::ToTitle => {}
                        }
                    }
                    if rebuild {
                        self.restart();
                    }
                    self.go(next);
                }
                if matches!(k, flow::Ask::Buy(_) | flow::Ask::NewGame) {
                    self.follow_camera();
                }
            }
            Scene::Equip => {
                let rows = equip_rows();
                let step = a.menu_right as i32 - a.menu_left as i32;
                if a.back || a.pause {
                    self.go(flow::back(self.scene));
                    return;
                }
                if step != 0 {
                    match rows.get(self.cursor) {
                        Some(EquipRow::Weapon) => self.next_weapon(step),
                        Some(EquipRow::Charm(i)) => self.step_charm(*i, step),
                        _ => {}
                    }
                } else if let Some(i) = self.menu_pick(&a, rows.len()) {
                    match rows[i] {
                        EquipRow::Weapon => self.next_weapon(1),
                        EquipRow::Charm(slot) => self.step_charm(slot, 1),
                        EquipRow::Back => self.go(flow::back(self.scene)),
                    }
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
                    let grow = !self.progress.growth.options().is_empty();
                    self.go(flow::after_ending(won, grow));
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
                self.sync_progress(out);
                // 쉼터 화로(0064 M4): 쉬면 가게가 열린다. 살 것이 있으면 쉼터가 빛난다.
                self.run.shop_ready = crate::growth::can_buy(
                    self.world.embers,
                    &self.progress.owned,
                    &self.progress.growth,
                );
                if self.run.take_rested() {
                    self.snd(crate::sfx::Id::Rest);
                    self.go(Scene::Shop);
                }
                // 문지기가 깨어나는 순간(0064 M5): 낮은 땅울림.
                let waking = matches!(self.run.phase, crate::stage::Phase::Waking(_));
                if waking && !self.was_waking {
                    self.snd(crate::sfx::Id::Wake);
                }
                self.was_waking = waking;
                if let Some((_, t)) = &mut self.toast {
                    *t -= TICK_DT;
                }
                self.toast = self.toast.take().filter(|(_, t)| *t > 0.0);
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
        let events = self.world.step(a, t);
        for &e in &events {
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
                Event::Gulp { .. } | Event::FlaskDenied | Event::Ember { .. } => 0.0,
                Event::Cast { .. } | Event::CastDenied | Event::Throw => 0.0,
                Event::Break { .. } => SHAKE_SMASH,
                Event::Aftershock { .. } | Event::Boom { .. } => SHAKE_PLUNGE,
                Event::Perfect { .. } => SHAKE_CHARGED,
                Event::Swing | Event::LockOn | Event::LockOff => 0.0,
            };
            // 화면 흔들림 설정(0052): 0..=10 → 0..100%.
            cam.trauma = (cam.trauma + kick * self.shake as f32 / 10.0).min(1.0);
        }
        self.world_sounds(&events);
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
            Scene::Equip => {
                hud::draw(&self.ui, &f, &self.world, Some(&self.run), true);
                self.draw_equip(&f);
            }
            Scene::Core => self.draw_core(&f),
            // 가게 창이 잔불을 보여 주니 밑의 HUD는 그리지 않는다(KB `hud-readability-and-quick-slots`).
            Scene::Shop => self.draw_shop(&f),
            Scene::Ask(k) => self.draw_ask(&f, k),
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
        if matches!(
            self.scene,
            Scene::Play | Scene::Ending { .. } | Scene::Shop | Scene::Title
        ) {
            self.draw_toast(&f);
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
        let has = self.progress.has_any();
        let items: Vec<String> = flow::title_items(has)
            .iter()
            .map(|i| match i {
                TitleItem::Start if has => s.menu_continue.to_string(),
                TitleItem::Start => s.menu_start.to_string(),
                TitleItem::NewGame => s.menu_new_game.to_string(),
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
                PauseItem::Weapon => fill(s.menu_equip, &[weapon_name(self.weapon)]),
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

    /// 장비(0064 M3): 무기 줄 + 장신구 세 칸 + 돌아가기, 아래에 고른 칸의 설명.
    fn draw_equip(&mut self, f: &Frame) {
        let s = s();
        self.ui.dim(0.6);
        let cx = f.ox + f.w * 0.5;
        self.ui
            .text_center(s.equip_title, cx, f.oy + f.h * 0.14, 2.0, INK);
        let rows = equip_rows();
        let items: Vec<String> = rows
            .iter()
            .map(|r| match r {
                EquipRow::Weapon => fill(s.menu_weapon, &[weapon_name(self.weapon)]),
                EquipRow::Charm(i) => {
                    let name = self.progress.worn[*i]
                        .map_or(s.charm_empty, |id| tr(crate::charms::def(id).name));
                    fill(s.menu_charm_slot, &[&(i + 1).to_string(), name])
                }
                EquipRow::Back => s.controls_back.to_string(),
            })
            .collect();
        let bottom = self.menu_bottom(f) - self.ui.px(1.0) * 2.4;
        self.menu_hits = self.ui.menu(
            &items,
            self.cursor,
            &mut self.view,
            cx,
            f.oy + f.h * 0.3,
            bottom,
        );
        // 설명 줄: 고른 칸의 장신구, 아직 하나도 없으면 얻는 곳.
        let desc = match rows.get(self.cursor) {
            _ if self.progress.owned.is_empty() => Some(s.equip_none_yet),
            Some(EquipRow::Charm(i)) => {
                self.progress.worn[*i].map(|id| tr(crate::charms::def(id).desc))
            }
            _ => None,
        };
        if let Some(d) = desc {
            self.ui.text_center_wrapped(
                d,
                cx,
                bottom + self.ui.px(1.0) * 0.6,
                1.0,
                AMBER,
                f.w * 0.9,
            );
        }
        self.menu_hint(cx, f.oy + f.h - self.ui.px(1.0) * 2.2, f.w * 0.94);
    }

    /// 핵 고르기(0064 M4): 제목 + 갈래 셋(지금/상한).
    fn draw_core(&mut self, f: &Frame) {
        let s = s();
        self.ui.dim(0.6);
        let cx = f.ox + f.w * 0.5;
        let y = f.oy + f.h * 0.2;
        self.ui.text_center(s.core_title, cx, y, 2.0, AMBER);
        self.ui
            .text_center(s.core_sub, cx, y + self.ui.px(2.0) * 1.3, 1.0, DIM);
        let g = self.progress.growth;
        let items: Vec<String> = g
            .options()
            .iter()
            .map(|&p| {
                let name = match p {
                    crate::growth::Pick::Hp => s.grow_hp,
                    crate::growth::Pick::Flask => s.grow_flask,
                    crate::growth::Pick::Heal => s.grow_heal,
                };
                let n = (g.count(p) + 1).to_string();
                let m = crate::growth::cap(p).to_string();
                fill(s.grow_line, &[name, &n, &m])
            })
            .collect();
        let bottom = self.menu_bottom(f);
        self.menu_hits = self.ui.menu(
            &items,
            self.cursor,
            &mut self.view,
            cx,
            f.oy + f.h * 0.42,
            bottom,
        );
        self.menu_hint(cx, f.oy + f.h - self.ui.px(1.0) * 2.2, f.w * 0.94);
    }

    /// 쉼터 화로(0064 M4): 가진 잔불 + 살 것(값) + 떠나기, 아래에 고른 것의 설명.
    fn draw_shop(&mut self, f: &Frame) {
        let s = s();
        self.ui.dim(0.6);
        let cx = f.ox + f.w * 0.5;
        let y = f.oy + f.h * 0.14;
        self.ui.text_center(s.shop_title, cx, y, 2.0, AMBER);
        let have = self.world.embers.to_string();
        self.ui.text_center(
            &fill(s.shop_embers, &[&have]),
            cx,
            y + self.ui.px(2.0) * 1.3,
            1.0,
            INK,
        );
        let wares = crate::growth::wares(&self.progress.owned, &self.progress.growth);
        let name = |w: crate::growth::Ware| match w {
            crate::growth::Ware::Charm(id) => tr(crate::charms::def(id).name),
            crate::growth::Ware::Oil => s.shop_oil,
        };
        let mut items: Vec<String> = wares
            .iter()
            .map(|&(w, price)| fill(s.shop_item, &[name(w), &price.to_string()]))
            .collect();
        items.push(s.shop_leave.to_string());
        let bottom = self.menu_bottom(f) - self.ui.px(1.0) * 2.4;
        let have = self.world.embers;
        let dim: Vec<bool> = wares.iter().map(|&(_, price)| have < price).collect();
        self.menu_hits = self.ui.menu_dimmed(
            &items,
            &dim,
            self.cursor,
            &mut self.view,
            cx,
            f.oy + f.h * 0.34,
            bottom,
        );
        let desc = match wares.get(self.cursor) {
            Some((crate::growth::Ware::Charm(id), _)) => Some(tr(crate::charms::def(*id).desc)),
            Some((crate::growth::Ware::Oil, _)) => Some(s.shop_oil_d),
            None if wares.is_empty() => Some(s.shop_empty),
            None => None,
        };
        if let Some(d) = desc {
            self.ui.text_center_wrapped(
                d,
                cx,
                bottom + self.ui.px(1.0) * 0.6,
                1.0,
                AMBER,
                f.w * 0.9,
            );
        }
        self.menu_hint(cx, f.oy + f.h - self.ui.px(1.0) * 2.2, f.w * 0.94);
    }

    /// 묻는 창: 무엇을 잃는지 한 줄 + 줄 둘(하는 일 이름 / 취소). 뒤에 온 곳을 흐리게 깐다.
    fn draw_ask(&mut self, f: &Frame, k: flow::Ask) {
        let s = s();
        // 멈춤에서 왔으면 HUD를 깔고 흐리게. 가게 목록은 깔지 않는다(글이 겹쳐 읽기 어려웠다 — knellroot에는 nail의 창 틀이 없다).
        if flow::back(Scene::Ask(k)) == Scene::Pause {
            hud::draw(&self.ui, f, &self.world, Some(&self.run), true);
        }
        self.ui.dim(0.7);
        let cx = f.ox + f.w * 0.5;
        let (q, yes) = match k {
            flow::Ask::NewGame => (s.ask_new_game.to_string(), s.ask_new_game_yes),
            flow::Ask::Restart => (s.ask_restart.to_string(), s.menu_restart),
            flow::Ask::ToTitle => (s.ask_to_title.to_string(), s.menu_to_title),
            flow::Ask::Buy(i) => {
                let wares = crate::growth::wares(&self.progress.owned, &self.progress.growth);
                let (name, price) = match wares.get(i) {
                    Some((crate::growth::Ware::Charm(id), p)) => {
                        (tr(crate::charms::def(*id).name), *p)
                    }
                    Some((crate::growth::Ware::Oil, p)) => (s.shop_oil, *p),
                    None => ("", 0),
                };
                (fill(s.ask_buy, &[name, &price.to_string()]), s.ask_buy_yes)
            }
        };
        self.ui
            .text_center_wrapped(&q, cx, f.oy + f.h * 0.3, 1.0, INK, f.w * 0.8);
        let items = vec![yes.to_string(), s.ask_cancel.to_string()];
        let bottom = self.menu_bottom(f);
        self.menu_hits = self.ui.menu(
            &items,
            self.cursor,
            &mut self.view,
            cx,
            f.oy + f.h * 0.48,
            bottom,
        );
        self.menu_hint(cx, f.oy + f.h - self.ui.px(1.0) * 2.2, f.w * 0.94);
    }

    /// 알림(0064 M3): 화면 위 가운데, 끝 0.6초에 옅어진다.
    fn draw_toast(&self, f: &Frame) {
        if let Some((text, t)) = &self.toast {
            let a = (t / 0.6).min(1.0);
            let cx = f.ox + f.w * 0.5;
            // 화로 가게에서는 목록을 가리지 않게 맨 위에.
            let y = if self.scene == Scene::Shop { 0.04 } else { 0.3 };
            self.ui.text_center(
                text,
                cx,
                f.oy + f.h * y,
                1.2,
                Color::new(AMBER.r, AMBER.g, AMBER.b, a),
            );
        }
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
                SettingsRow::Volume => {
                    fill(s.menu_volume, &[&format!("{}%", self.volume as u32 * 10)])
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
