//! 앱(0019): 장면(타이틀 → 싸움 → 큰 창(멈춤·화로, 0164) → 끝 → 결과)과 프레임 루프. 흐름은 `flow.rs`의 순수 함수가 정하고 여기서는 그대로 한다.
//! 월드는 메뉴를 모르고, 스테이지(`stage`)는 월드를 지켜보며 진행만 하고, 앱이 둘을 묶어 카메라·그림·글로 보낸다.
//!
//! 루프: 프레임마다 입력 폴링 → 고정 60Hz 틱 N번(장면 → 월드 → 스테이지 → 카메라) → 가변 렌더(보간) → 글·메뉴(화면 패스).
mod ask;
mod codex;
pub mod controls;
mod controls_screen;
#[cfg(feature = "dev")]
mod dev;
mod equip;
pub mod flow;
mod frame;
#[cfg(feature = "dev")]
mod lab_screen;
mod menu;
mod play;
mod progress;
mod settings;
mod shop;
mod skills;
mod slots;
mod stack;
mod status;
mod tabs;
mod title;

use macroquad::prelude::*;

use crate::camera::{Camera, Mode};
use crate::hud;
use crate::i18n::{self, fill, s, tr};
use crate::input::{self, Act, Actions, Input};
#[cfg(feature = "dev")]
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
use equip::next_charm;
use flow::{Go, Layer, ResultItem, Scene, SettingsRow, TitleItem};
use menu::Hit;

/// 타이틀 메뉴 첫 줄의 자리(화면 높이 비율) — `tools/test/touch.js`가 같은 값으로 탭한다.
const TITLE_MENU_Y: f32 = 0.37;

/// 가장자리 화살(0060 W1): 갈 곳이 화면 밖에 이만큼(초) 있으면 띄운다(문이 열리는 순간·잠깐 돌아본 것엔 안 뜨게), 테두리에서 들인 몫(NDC).
const GOAL_ARROW_DELAY: f32 = 1.0;
const GOAL_ARROW_INSET: f32 = 0.12;

/// 판의 첫 지역(0118 — 차례는 `stage::STAGES`).
const FIRST_STAGE: &StageDef = &sanctuary::DEF;

pub struct App {
    /// 장면 스택(0066): 맨 위가 지금 장면, 아래 칸은 커서·창을 맡아 둔다. 지금 칸의 커서·창은 `cursor`·`view`.
    stack: stack::Stack,
    cursor: usize,
    /// 지난 프레임에 그린 메뉴 줄·▲▼ 자리(탭 판정).
    hits: Vec<(Rect, Hit)>,
    /// 지금 화면의 굴리는 목록 창(메뉴·조작 설정 공통, 장면이 바뀌면 처음부터).
    view: ui::ListView,
    input: Input,
    t: Tuning,
    /// 랩(0070, 개발판): 슬롯 B·알림.
    #[cfg(feature = "dev")]
    lab: Lab,
    renderer: Renderer,
    ui: Ui,
    cam: Camera,
    world: World,
    run: StageRun,
    /// 판의 지역 잇기(0118): 지금 지역(`stage::STAGES` 차례), 이 판에 이긴 문지기 수, 마지막으로 쉰 곳(지역을 바꿔도 남는다),
    /// 지역 사이 지층 단면 전환.
    stage: usize,
    down: usize,
    rest_at: Option<crate::save::RunAt>,
    passage: Option<crate::descent::Passage>,
    /// 이 판에서 멈춤 창이 마지막에 보던 탭(0164) — 다음에 멈추면 거기서 열린다(`flow::pause`). 판에 붙은 한 칸: 새 판이면 상태로.
    pause_tab: flow::Tab,
    /// 주민 말풍선(0122): (주민 — 지금 지역의 번호, 몇째 말, 남은 초). 멈추지 않는다.
    bubble: Option<(usize, usize, f32)>,
    /// 새긴 돌 글 판(0129): (지금 지역의 돌 번호, 남은 초).
    stone_card: Option<(usize, f32)>,
    /// 실제 시간(전환의 작은 움직임 — 지형을 굽는 동안에도 흐른다).
    clock: f32,
    #[cfg(feature = "dev")]
    sc: Scenario,
    /// 개발 메뉴에서 시작한 스테이지 판(0075): 기록 슬롯에 쓰지 않는다. 슬롯을 고르면(`enter_slot`) 풀린다.
    #[cfg(feature = "dev")]
    dev_run: bool,
    /// 개발 창에서 올린 기술 판(0158 — 모두 끝 단): 있으면 기록의 판 대신 이것을 쓴다. 기록에 쓰지 않고, 슬롯을 고르면 풀린다.
    #[cfg(feature = "dev")]
    dev_board: Option<crate::skills::Board>,
    ending: f32,
    acc: f32,
    /// 개발 줄(F3).
    dev_overlay: bool,
    /// 개발(F6, 0141): 한 프레임에 꼭 한 틱 — 0.1초짜리 이펙트를 틱마다 찍어 보려고(`tools/test/fx.js`). 운영판에는 없다.
    step_clock: bool,
    /// 조작 설정 화면(0024).
    ctl: controls::Controls,
    perf: Perf,
    sums: (f64, f64, f64, u32),
    /// 든 무기(0051) — 다시 시작해도 그대로, 설정과 함께 저장.
    weapon: weapons::Id,
    /// 화면 흔들림 0..=10(설정, 0052).
    shake: u8,
    /// 소리(0064 M5): 크기 0..=10, 소리 묶음(main이 올린 뒤 `set_sfx`), 지난 틱의 회피·적 예고(시작한 순간만 소리).
    volume: u8,
    /// 바닥 공격 예고 진하기 0..=10(0 = 끔, 0074).
    tells: u8,
    tell_timing: bool,
    sfx: Option<crate::sfx::Sfx>,
    was_dodging: bool,
    /// 그림 이펙트(0141): 월드 사건·상태 변화를 듣고 입자를 낸다 — 월드는 모른다(소리와 같은 길).
    fx: crate::fx::Fx,
    tele_was: Vec<u32>,
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
    /// 지금 슬롯(0066)과 슬롯 셋의 요약(타이틀 "이어하기"·슬롯 창, 쓸 때마다 고친다).
    slot: usize,
    slots: [Option<crate::save::Summary>; crate::save::SLOTS],
    /// 지금 화면의 둘째 축(슬롯 창의 단추)·두 번째 누름을 기다리나(덮어쓰기·지우기). 장면이 바뀌면 처음으로.
    col: usize,
    armed: bool,
    /// 지금 화면의 글 칸(설명)을 몇 줄 내렸나(0068 — 넘치는 설명은 오른쪽 스틱·휠·끌기로 굴린다). 장면이 바뀌면 처음으로.
    scroll: f32,
    /// 옛 prefs 칸 21(든 무기) — 읽은 값 그대로 다시 적는다(0066에서 기록으로 옮김).
    legacy_weapon: u8,
    /// 저장에 실패한 채인가(실패가 시작될 때 한 번만 알린다 — KB `roguelite-save-slots-and-continue`).
    save_failed: bool,
    /// 플레이 시간의 1초 아래 나머지(0081).
    play_acc: f32,
    /// 잠깐 뜨는 알림(0064 M3 — 상자에서 얻은 것 등)과 남은 초.
    toast: Option<(String, f32)>,
}

/// 알림이 떠 있는 시간(초).
const TOAST_TIME: f32 = 3.5;

impl App {
    /// 개발 판(DEV에서 시작한 스테이지)인가 — 운영판은 늘 거짓.
    fn dev_run_now(&self) -> bool {
        #[cfg(feature = "dev")]
        return self.dev_run;
        #[cfg(not(feature = "dev"))]
        false
    }

    /// 들 수 있는 무기(0117): 기록에 가진 것 — 개발 판·놀이터(시나리오)는 다(무기를 시험하려고).
    fn arms(&self) -> Vec<weapons::Id> {
        #[cfg(feature = "dev")]
        if self.dev_run || self.stack.base() == Scene::Sandbox {
            return weapons::IDS.to_vec();
        }
        self.progress.owned_weapons()
    }

    pub fn new() -> Self {
        let t = Tuning::default();
        // 저장한 설정(0024·0052·0058): 언어·키 배치·흔들림·무기·마우스 조준.
        let mut legacy_weapon = 0;
        let mut shake = crate::prefs::SHAKE_DEFAULT;
        let mut volume = crate::prefs::VOLUME_DEFAULT;
        let mut tells = crate::prefs::TELLS_DEFAULT;
        let mut tell_timing = false;
        if let Some(p) = crate::prefs::load() {
            if let Some(l) = p.lang {
                i18n::set_lang(l);
            }
            input::set_bindings(p.bindings);
            input::set_mouse_aim(p.mouse);
            shake = p.shake;
            volume = p.volume;
            tells = p.tells;
            tell_timing = p.tell_timing;
            legacy_weapon = p.weapon;
        }
        // 기록 슬롯(0066): 마지막으로 쓴 슬롯의 기록. 든 무기는 기록에 — 없으면(옛 슬롯 1) 예전 prefs 칸 21의 값.
        let slot = crate::save::last_slot();
        let progress = crate::save::load(slot);
        let weapon = progress.weapon.unwrap_or_else(|| {
            if slot == 0 {
                weapons::IDS[(legacy_weapon as usize).min(weapons::IDS.len() - 1)]
            } else {
                weapons::Id::Sword
            }
        });
        // 가지지 않은 무기는 들 수 없다(0117).
        let weapon = if progress.owns_weapon(weapon) {
            weapon
        } else {
            weapons::Id::Sword
        };
        let (run, mut world) = StageRun::start(FIRST_STAGE);
        world.set_weapon(weapon);
        world.set_skills(progress.board.loadout(progress.skills.0, progress.ult));
        world.tree = progress.board.cells;
        let cam = Camera::new(world.player.pos, &t);
        App {
            stack: stack::Stack::new(Scene::Title),
            cursor: 0,
            hits: Vec::new(),
            view: ui::ListView::default(),
            input: Input::new(),
            #[cfg(feature = "dev")]
            lab: Lab::new(),
            renderer: Renderer::new(),
            ui: Ui::new(),
            cam,
            world,
            run,
            stage: 0,
            down: 0,
            rest_at: None,
            passage: None,
            pause_tab: flow::Tab::Status,
            bubble: None,
            stone_card: None,
            clock: 0.0,
            t,
            #[cfg(feature = "dev")]
            sc: Scenario::Boss,
            #[cfg(feature = "dev")]
            dev_run: false,
            #[cfg(feature = "dev")]
            dev_board: None,
            ending: 0.0,
            acc: 0.0,
            dev_overlay: false,
            step_clock: false,
            ctl: controls::Controls::default(),
            perf: Perf::default(),
            sums: (0.0, 0.0, 0.0, 0),
            weapon,
            shake,
            volume,
            tells,
            tell_timing,
            sfx: None,
            was_dodging: false,
            fx: crate::fx::Fx::default(),
            tele_was: Vec::new(),
            was_waking: false,
            frame: None,
            mouse_hidden: false,
            goal_off: 0.0,
            progress: progress.clone(),
            saved: progress,
            slot,
            slots: crate::save::summaries(),
            col: 0,
            armed: false,
            scroll: 0.0,
            legacy_weapon,
            save_failed: false,
            play_acc: 0.0,
            toast: None,
        }
    }

    /// 새 판(0122 — 판 수를 센다: 주민 재방문 "처음 본 판이 끝난 뒤의 다른 판"). 이어하기는 `build_run`.
    fn restart(&mut self) {
        self.progress.runs = self.progress.runs.saturating_add(1);
        self.build_run();
    }

    /// 첫 지역을 새로 세운다(판 수는 그대로).
    fn build_run(&mut self) {
        let (run, world) = StageRun::start(FIRST_STAGE);
        self.run = run;
        self.world = world;
        self.fx.clear();
        self.stage = 0;
        self.down = 0;
        self.rest_at = None;
        self.passage = None;
        self.pause_tab = flow::Tab::Status;
        // 잔불은 판을 넘어 남는다(0064 M1). 장신구·성장은 능력치로, 연 상자는 비어 있다(M3).
        self.world.embers = self.progress.embers;
        self.world.apply_stats(self.progress.stats(), true);
        self.seed_run();
        self.cam = Camera::new(self.world.player.pos, &self.t);
    }

    /// 다른 지역으로(0118): 그 지역을 새로 세우고 방랑자(체력·등잔·기력·탄…)와 잔불은 그대로 옮긴다.
    fn enter_stage(&mut self, s: usize) {
        let s = s.min(crate::stage::STAGES.len() - 1);
        // 말풍선·새긴 돌 글 판은 지역 안의 번호라 지역을 바꾸면 닫는다(0129).
        self.bubble = None;
        self.stone_card = None;
        let (run, mut world) = StageRun::start(crate::stage::STAGES[s]);
        let (start, facing) = (world.player.pos, world.player.facing);
        std::mem::swap(&mut self.world.player, &mut world.player);
        let pl = &mut world.player;
        pl.pos = start;
        pl.prev = start;
        pl.facing = facing;
        pl.vel = Vec3::ZERO;
        pl.state = crate::world::PState::Idle;
        pl.air = 0.0;
        world.embers = self.world.embers;
        self.run = run;
        self.world = world;
        self.fx.clear();
        self.stage = s;
        self.world.apply_stats(self.progress.stats(), false);
        self.seed_run();
    }

    /// 지역을 세운 뒤 기록에서 채운다: 무기·기술(끼운 스킬과 연 패시브 칸)·연 상자(지역마다 번호 `지역 × 1000 + 번호`, 0118)·주운 것·쓰러뜨린 엘리트·문,
    /// 이 판에 이 지역 문지기를 이겼으면 바닥 길이 열린 채.
    fn seed_run(&mut self) {
        self.world.set_weapon(self.weapon);
        self.apply_board();
        let base = self.stage as u32 * crate::stage::CHESTS_PER_STAGE;
        let opened: Vec<u32> = self
            .progress
            .chests
            .iter()
            .filter(|&&c| c / crate::stage::CHESTS_PER_STAGE == self.stage as u32)
            .map(|c| c - base)
            .collect();
        self.run.set_opened(&opened);
        self.run.set_found(&self.progress.finds);
        self.run.set_beaten(&self.progress.elites);
        let open = self
            .progress
            .gates
            .iter()
            .any(|g| g == self.run.def.find_prefix);
        self.run.set_gate_opened(&mut self.world, open);
        // 주민 재방문(0122, 0116): 처음 본 판보다 뒤의 판이면 한 번 바뀐다(단방향, 기록).
        self.bubble = None;
        for k in 0..self.run.residents.len() {
            let id = crate::residents::RESIDENTS[self.run.residents[k].0].id;
            self.run.resident_changed[k] = crate::residents::settle(
                id,
                &self.progress.residents_seen,
                &mut self.progress.residents_changed,
                self.progress.runs,
            );
        }
        // 처음 이기는 판이면 핵이 스스로 다가온다(lore CANON, 0105) — 첫 지역 문지기만(둘째 문지기는 C3).
        self.run.first_core = match self.stage {
            0 => self.progress.clears == 0,
            _ => self.progress.bell_cores == 0,
        };
        if self.down > self.stage {
            self.run.down_open = true;
            self.run.open_down(&mut self.world);
        }
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

    /// 묻는 창을 연다(커서는 버리는 일이면 취소에서).
    fn ask(&mut self, k: flow::Ask) {
        self.go(Go::Open(Scene::Ask(k)));
    }

    /// 무기를 바꾼다(0051, 장비 탭의 빨리 바꾸기): 지금 판에 바로, 저장.
    fn next_weapon(&mut self, step: i32) {
        // 가진 무기 사이에서만(0117).
        let arms = self.arms();
        let i = arms.iter().position(|&w| w == self.weapon).unwrap_or(0) as i32;
        self.weapon = arms[(i + step).rem_euclid(arms.len() as i32) as usize];
        self.world.set_weapon(self.weapon);
        self.progress.weapon = Some(self.weapon);
        self.persist();
    }

    /// 전체 화면을 켜고 끈다(설정 줄·F11) — PC는 다음에 켤 때도 그대로(0135).
    pub(super) fn toggle_fullscreen(&mut self) {
        crate::screen::toggle();
        if crate::DESKTOP {
            self.save_prefs();
        }
    }

    /// 설정을 저장한다(언어·키 배치를 바꿀 때마다).
    fn save_prefs(&self) {
        crate::prefs::save(&crate::prefs::Prefs {
            lang: Some(i18n::lang()),
            bindings: input::bindings(),
            mouse: input::mouse_aim(),
            shake: self.shake,
            volume: self.volume,
            tells: self.tells,
            tell_timing: self.tell_timing,
            // PC만 기억한다(0135) — 웹은 브라우저가 Esc로 끌 수 있고 다음에 켤 때 사용자 입력 없이 못 연다.
            fullscreen: crate::DESKTOP && crate::screen::active(),
            // 든 무기는 슬롯 기록으로 옮겼다(0066) — 옛 칸은 처음 읽은 값 그대로 둔다(옛 슬롯 1의 기본값).
            weapon: self.legacy_weapon,
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

    /// 지금 장면(스택 맨 위).
    fn scene(&self) -> Scene {
        self.stack.top()
    }

    /// 장면을 바꾼다(0066): 지금 칸의 커서·창을 스택에 맡기고, 새 맨 위의 것(연 창이면 처음 포커스, 닫았으면 연 자리)을 받는다.
    fn go(&mut self, go: Go) {
        if let Some((cursor, view)) = self.stack.go(go, self.cursor, self.view) {
            self.cursor = cursor;
            self.view = view;
            self.col = 0;
            self.armed = false;
            self.scroll = 0.0;
            self.hits.clear();
            self.ctl = controls::Controls::default();
        }
    }

    /// 한 프레임.
    pub fn frame(&mut self) {
        let cpu_start = get_time();
        // 터치의 상황 버튼(0053): 쉼터 곁이면 점프가 쉬기, 락온할 적이 있거나 락온 중이면 락온, 마실 수 있으면 등잔.
        let play = self.scene().ticks_world();
        let pl = &self.world.player;
        self.input.set_touch_ctx(crate::touch::Ctx {
            rest: play && self.world.interact_here,
            near: match self.run.near() {
                Some(crate::stage::Near::Pick) => crate::touch::Near::Pick,
                Some(crate::stage::Near::Open) => crate::touch::Near::Open,
                Some(crate::stage::Near::Pull) => crate::touch::Near::Pull,
                Some(crate::stage::Near::Talk) => crate::touch::Near::Talk,
                Some(crate::stage::Near::Read) => crate::touch::Near::Read,
                Some(crate::stage::Near::Rest { .. }) | None => crate::touch::Near::Rest,
            },
            lock: play && (self.world.lock.is_some() || self.world.lock_candidate()),
            flask: play && pl.flask > 0 && pl.hp < pl.max_hp,
            ult: play && pl.skill.gauge >= 1.0,
        });
        self.input.poll(self.scene().is_menu());
        // F11(PC, 0135): 어느 장면에서나 창 ↔ 전체 화면. 창 일이라 `Actions`(게임 동작)에 넣지 않는다.
        if input::fullscreen_pressed() {
            self.toggle_fullscreen();
        }
        // 싸움 중 마우스 조준이면 OS 커서 대신 조준 표시(0058). 랩은 마우스를 쓰지 않지만 화살표가 보이게 둔다.
        let hide = play && input::mouse_aim().on && input::device() == input::Device::Keys;
        if hide != self.mouse_hidden {
            show_mouse(!hide);
            self.mouse_hidden = hide;
        }
        let dt = get_frame_time().min(MAX_FRAME_DT);
        self.acc += dt;
        if self.step_clock {
            // 개발(F6): 실제 시간과 상관없이 프레임마다 한 틱.
            self.acc = 0.0;
            let a = self.input.take();
            self.tick(a);
        }
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

    /// 화면 아래 안내 줄 위까지(메뉴가 굴리는 아래 끝).
    fn menu_bottom(&self, f: &Frame) -> f32 {
        f.oy + f.h - self.ui.px(1.0) * 2.8
    }

    fn tick(&mut self, a: Actions) {
        let (scene0, cursor0, back0) = (self.scene(), self.cursor, a.back || a.pause);
        let played0 = self.sfx.as_ref().map_or(0, |s| s.played());
        self.tick_scene(a);
        // 메뉴 소리(0064 M5, KB `game-audio-without-a-composer` "메뉴 소리는 한 곳에서"): 그 사이에 다른 소리(사기·안 됨 등)가
        // 나지 않았을 때만 — 뒤로 > 확인(장면이 바뀜) > 커서.
        let quiet = self.sfx.as_ref().map_or(0, |s| s.played()) == played0;
        if scene0.is_menu() && quiet {
            if self.scene() != scene0 {
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
        // 알림은 어느 장면에서나 줄어든다(가게의 "잔불이 모자란다"가 떠날 때까지 남았다).
        if let Some((_, t)) = &mut self.toast {
            *t -= TICK_DT;
        }
        self.toast = self.toast.take().filter(|(_, t)| *t > 0.0);
        if a.dev_overlay {
            self.dev_overlay = !self.dev_overlay;
        }
        #[cfg(feature = "dev")]
        if a.ui_check {
            crate::ui::check::toggle();
        }
        #[cfg(feature = "dev")]
        if a.step_clock {
            self.step_clock = !self.step_clock;
        }
        #[cfg(feature = "dev")]
        if a.lab_toggle {
            self.toggle_lab();
            return;
        }
        // 큰 창의 탭(0164): 탭 넘기기(스킬 버튼 둘·탭 머리)가 먼저 — 넘겼으면 그 틱은 끝.
        let swapped = flow::tab_of(self.scene()).is_some() && self.tab_input(&a);
        if !swapped {
            self.tick_top(a);
        }
        // 창이 열린 동안 카메라는 계속 따라간다(타이틀 바탕·화로) — 판 위의 멈춤 아래에서는 그대로 선다.
        if self.scene().is_menu() && !self.paused() {
            self.follow_camera();
        }
    }

    /// 멈춤에서 연 창(과 그 위의 작은 창들)이 떠 있나.
    fn paused(&self) -> bool {
        self.stack
            .any(|sc| matches!(flow::tab_of(sc), Some((flow::Host::Pause, _))))
    }

    /// 맨 위 장면의 한 틱.
    fn tick_top(&mut self, a: Actions) {
        match self.scene() {
            Scene::Title => self.tick_title(a),
            Scene::Ask(k) => self.tick_ask(&a, k),
            Scene::Status { host } => self.tick_status(&a, host),
            Scene::Equip { .. } => self.tick_equip(&a),
            Scene::Skills { host } => self.tick_skills(&a, host.edits()),
            Scene::SkillAct => self.tick_skill_act(&a),
            Scene::Codex { host, tab } => self.tick_codex(&a, host, tab as usize),
            Scene::EquipPick { slot } => self.tick_equip_pick(&a, slot as usize),
            s @ (Scene::Settings { .. } | Scene::Shop | Scene::Result { .. }) => {
                self.tick_menu(&a, s)
            }
            #[cfg(feature = "dev")]
            Scene::Dev { tab } => self.tick_dev(&a, tab as usize),
            #[cfg(feature = "dev")]
            Scene::Lab => self.tick_menu(&a, Scene::Lab),
            #[cfg(feature = "dev")]
            Scene::LabGroup { g } => self.tick_lab_group(&a, g as usize),
            Scene::Controls => self.tick_controls(a),
            Scene::Slots { load } => self.tick_slots(&a, load),
            Scene::Ending { won } => {
                self.ending += TICK_DT;
                self.follow_camera();
                if self.ending >= flow::ENDING_TIME {
                    self.go(Go::Root(flow::after_ending(won)));
                }
            }
            Scene::Play => self.tick_play(a),
            #[cfg(feature = "dev")]
            Scene::Sandbox => self.tick_sandbox(a),
        }
    }

    /// 목록 화면 한 틱(0066): 줄 표 → 입력 → 화면의 고름.
    fn tick_menu(&mut self, a: &Actions, scene: Scene) {
        if let Some(m) = self.menu_of(scene) {
            if let Some(out) = self.menu_input(a, &m) {
                self.act(scene, out);
            }
        }
    }

    /// 장면의 줄 표(목록 화면만 — 제 그림을 갖는 화면은 `None`). `_` 없는 match: 새 장면이면 컴파일러가 묻는다.
    fn menu_of(&self, scene: Scene) -> Option<menu::Menu> {
        match scene {
            Scene::Settings { host } => Some(self.settings_menu(host)),
            Scene::Shop => Some(self.shop_menu()),
            Scene::Result { won } => Some(self.result_menu(won)),
            Scene::Slots { load } => Some(self.slots_menu(load)),
            #[cfg(feature = "dev")]
            Scene::Lab => Some(self.lab_menu()),
            #[cfg(feature = "dev")]
            Scene::LabGroup { g } => Some(self.lab_group_menu(g as usize)),
            Scene::Title
            | Scene::Play
            | Scene::Ending { .. }
            | Scene::Controls
            | Scene::Ask(_)
            | Scene::Status { .. }
            | Scene::Equip { .. }
            | Scene::Skills { .. }
            | Scene::SkillAct
            | Scene::Codex { .. }
            | Scene::EquipPick { .. } => None,
            #[cfg(feature = "dev")]
            Scene::Sandbox | Scene::Dev { .. } => None,
        }
    }

    /// 목록 화면의 고름을 그 화면에(뒤로는 몰이꾼이 한다).
    fn act(&mut self, scene: Scene, out: menu::Out) {
        match scene {
            Scene::Settings { host } => self.settings_act(host, out),
            Scene::Shop => self.shop_act(out),
            Scene::Result { .. } => self.result_act(out),
            // 슬롯 창은 제 틱(`tick_slots`)이 고름을 한다(빈 줄 건너뛰기·두 번째 누름).
            Scene::Slots { .. } => {}
            #[cfg(feature = "dev")]
            Scene::Lab => self.lab_act(out),
            #[cfg(feature = "dev")]
            Scene::LabGroup { g } => self.lab_group_act(g as usize, out),
            Scene::Title
            | Scene::Play
            | Scene::Ending { .. }
            | Scene::Controls
            | Scene::Ask(_)
            | Scene::Status { .. }
            | Scene::Equip { .. }
            | Scene::Skills { .. }
            | Scene::SkillAct
            | Scene::Codex { .. }
            | Scene::EquipPick { .. } => {}
            #[cfg(feature = "dev")]
            Scene::Sandbox | Scene::Dev { .. } => {}
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
            tells: self.tells as f32 / 10.0,
            tell_timing: self.tell_timing,
            fx: &self.fx,
        });
        self.frame = Some(f);
        self.ui.begin(f.ui);
        let overlay = self.input.overlay();
        self.hits.clear();
        self.draw_layered(&f, self.scene());
        // 큰 창은 화면을 다 써서 알림을 제 오른쪽 칸의 상태 줄에 보인다(`draw_pane_head`) — 여기서는 싸움·타이틀만.
        if matches!(
            self.scene(),
            Scene::Play | Scene::Ending { .. } | Scene::Title
        ) {
            self.draw_toast(&f);
        }
        if self.scene() == Scene::Play {
            self.draw_goal_arrow(&f);
            self.draw_bubble(&f);
            self.draw_stone_card(&f);
        }
        // 지역 사이 지층 단면(0118): 싸움 화면 위에(전환 중엔 멈춤·장비 창이 열리지 않는다).
        self.clock += get_frame_time();
        if let Some(p) = self
            .passage
            .as_ref()
            .filter(|_| self.scene() == Scene::Play)
        {
            let m = p.depth().round() as u32;
            let label = if m == 0 {
                s().depth_surface.to_string()
            } else {
                fill(s().depth_under, &[&m.to_string()])
            };
            crate::descent::draw(p, &self.ui, &f, self.clock, &label);
        }
        if self.scene().ticks_world() {
            self.draw_beat_note(&f);
        }
        if self.scene().ticks_world() {
            if self.mouse_hidden && self.world.lock_target().is_none() {
                if let Some(c) = self.input.cursor() {
                    hud::draw_reticle(&f, c, input::mouse_aim().size);
                }
            }
            if let Some(o) = &overlay {
                let pl = &self.world.player;
                let skill = |slot: usize| {
                    (
                        touch::skill_icon(pl.skill.worn[slot]),
                        self.world.skill_ready(slot, &self.t),
                    )
                };
                touch::draw(
                    o,
                    &touch::View {
                        locked: self.world.lock_target().is_some(),
                        turn: self.world.turn_ready(&self.t),
                        attack: touch::attack_icon(pl.weapon),
                        cast: touch::cast_icon(pl.weapon),
                        skills: [skill(0), skill(1)],
                        vacant: [0, 1].map(|i| pl.skill.rank[i] == 0),
                        active: [0, 1].map(|i| self.world.skill_active(i)),
                        ult: touch::ult_icon(pl.skill.ult),
                    },
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
    /// 장면을 제 자리(`Scene::layer`, 0066)에 그린다: 싸움 화면은 HUD, 판 위에서 연 창은 밑에 HUD를 깔고,
    /// 묻는 창은 아래 칸을 그대로 깐다(`draw_ask`). 목록 화면은 줄 표(`menu_of`)를 몰이꾼이 그린다.
    fn draw_layered(&mut self, f: &Frame, scene: Scene) {
        match scene.layer() {
            Layer::Game => {
                let run = matches!(scene, Scene::Play | Scene::Ending { .. }).then_some(&self.run);
                let edge = self.t[crate::tuning::P::HudHurtEdge];
                let skills = [0, 1].map(|i| self.world.skill_ready(i, &self.t));
                hud::draw(&self.ui, f, &self.world, run, false, edge, skills);
            }
            Layer::Window { hud: true } if self.stack.base() == Scene::Play => {
                let skills = [0, 1].map(|i| self.world.skill_ready(i, &self.t));
                hud::draw(&self.ui, f, &self.world, Some(&self.run), true, 0.0, skills);
            }
            Layer::Screen | Layer::Window { .. } | Layer::Modal => {}
        }
        match scene {
            Scene::Title => self.draw_title(f),
            Scene::Ask(k) => self.draw_ask(f, k),
            Scene::Controls => self.draw_controls(f),
            Scene::Status { host } => self.draw_status(f, host),
            Scene::Equip { host } => self.draw_equip(f, host, self.cursor, None),
            Scene::Skills { host } => self.draw_skills(f, self.cursor, host),
            Scene::SkillAct => self.draw_skill_act(f),
            Scene::Codex { host, tab } => self.draw_codex(f, host, tab as usize),
            Scene::Shop => self.draw_shop(f),
            Scene::Settings { host } => self.draw_settings(f, host),
            #[cfg(feature = "dev")]
            Scene::Dev { tab } => self.draw_dev_screen(f, tab as usize),
            Scene::EquipPick { slot } => {
                // 칸 커서·연 곳은 아래 칸(장비 탭)의 것.
                let under = self.stack.under().copied();
                let at = under.map_or(slot as usize, |e| e.cursor);
                let host = match under.map(|e| e.scene) {
                    Some(Scene::Equip { host }) => host,
                    _ => flow::Host::Pause,
                };
                self.draw_equip(f, host, at, Some(slot as usize));
            }
            s => {
                if let Some(m) = self.menu_of(s) {
                    self.draw_menu(f, &m);
                }
            }
        }
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

    /// 알림(0064 M3): 화면 위 가운데, 끝 0.6초에 옅어진다.
    fn draw_toast(&self, f: &Frame) {
        if let Some((text, t)) = &self.toast {
            let a = (t / 0.6).min(1.0);
            let cx = f.ox + f.w * 0.5;
            self.ui.text_center(
                text,
                cx,
                f.oy + f.h * 0.3,
                1.2,
                Color::new(AMBER.r, AMBER.g, AMBER.b, a),
            );
        }
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
        weapons::Id::Fists => s.weapon_fists,
    }
}
