//! 앱(0019): 장면(타이틀 → 싸움 → 큰 창(멈춤·화로, 0164) → 끝 → 결과)과 프레임 루프. 흐름은 `flow.rs`의 순수 함수가 정하고 여기서는 그대로 한다.
//! 월드는 메뉴를 모르고, 스테이지(`stage`)는 월드를 지켜보며 진행만 하고, 앱이 둘을 묶어 카메라·그림·글로 보낸다.
//!
//! 루프: 프레임마다 입력 폴링 → 고정 60Hz 틱 N번(장면 → 월드 → 스테이지 → 카메라) → 가변 렌더(보간) → 글·메뉴(화면 패스).
mod ask;
mod away;
mod bag;
mod codex;
pub mod controls;
mod controls_screen;
mod cue;
#[cfg(feature = "dev")]
mod dev;
mod equip;
pub mod flow;
mod frame;
mod interlude;
mod journey;
#[cfg(feature = "dev")]
mod lab_screen;
mod menu;
#[cfg(feature = "dev")]
mod motions;
mod play;
mod progress;
mod settings;
mod shop;
mod skills;
mod slots;
mod stack;
mod status;
mod tabs;
#[cfg(test)]
mod text_fit;
mod title;

use crate::det::Det;
use macroquad::prelude::*;

use crate::camera::{Camera, Mode};
use crate::hud;
use crate::i18n::{self, fill, s, tr};
use crate::input::{self, Act, Actions, Input};
#[cfg(feature = "dev")]
use crate::lab::Lab;
use crate::render::{AimShow, Frame, Perf, Renderer, View};
#[cfg(feature = "dev")]
use crate::scenario::{self, Scenario};
use crate::stage::StageRun;
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

/// 판의 첫 지역(0118 — 차례는 `stage::STAGES`): 성소. 아직 숲을 지나지 않은 새 기록은 서막 숲에서(0262 — `first_stage`).
const FIRST_STAGE: usize = 0;

/// 헤드리스 시험이 서막을 건너뛰라고 했나(0262 — 개발판 웹 `window.knellTestFlags` 2, `tools/test/scene.js`가 기본으로 건다:
/// 옛 스크립트는 성소에서 시작한다). 운영판은 늘 거짓.
fn skip_prologue() -> bool {
    #[cfg(feature = "dev")]
    return crate::screen::test_flags() & 2 != 0;
    #[cfg(not(feature = "dev"))]
    false
}

/// 새 판이 열리는 지역(0262): 숲을 지나지 않은 기록이면 서막 숲, 아니면 성소. 개발 창의 판·시험 깃발은 성소에서.
fn first_stage(progress: &crate::save::Progress, dev_run: bool) -> usize {
    if progress.prologue || dev_run || skip_prologue() {
        FIRST_STAGE
    } else {
        crate::stage::PROLOGUE
    }
}

/// 메뉴 누름을 붙잡는 시간(초, 0207) — 눌린 꼴이 보이고 장면이 넘어간다. UI 누름 피드백 100~160 ms 안쪽 끝(늦게 느껴지지 않게).
const PRESS_HOLD: f32 = 0.09;

/// 메뉴에서 줄 세울 입력(0207): 누르는 순간의 것만 — 쥐고 있는 스틱·버튼은 매 틱 같은 값이라 줄에 쌓이면 지연이 남는다.
fn is_edge(a: &Actions) -> bool {
    a.menu_up
        || a.menu_down
        || a.menu_left
        || a.menu_right
        || a.confirm
        || a.back
        || a.pause
        || a.tap.is_some()
        || a.tab_prev
        || a.tab_next
        || a.attack
        || a.turn_left
        || a.turn_right
        || a.skill1
        || a.skill2
        || a.equip
        || a.pressed_key.is_some()
        || a.pressed_pad.is_some()
        || a.wheel != 0
}

/// 누른 자리에 눌린 꼴을 보일까(0207 → 2026-11-19 사장 "컨텍스트 메뉴가 보이는 상태에서 바깥을 터치해서 닫을 때 그 바깥도 눌리는 듯한
/// 하이라이트가 생김"): 창 밖을 덮는 닫기 칸(`push_outside` — 화면보다 큰 칸)은 닫기일 뿐 단추가 아니고, 작은 창(칸 곁의 목록·묻는 창)이
/// 떠 있으면 그 창의 줄만 빛난다 — 아래 화면의 칸·탭을 눌러 창을 닫을 때 그 칸이 눌린 듯 보이지 않게. 그런 탭은 붙잡지 않고 바로 넘긴다.
fn press_shows(popup: bool, h: Hit, r: Rect) -> bool {
    let outside = r.w > 1.0e4 || r.h > 1.0e4;
    if popup {
        return matches!(h, Hit::Row(_));
    }
    matches!(h, Hit::Row(_) | Hit::Cell(..) | Hit::Tab(_)) || (h == Hit::Close && !outside)
}

/// 붙잡아 둔 메뉴 누름(0207).
#[derive(Clone, Copy, Debug)]
struct Press {
    left: f32,
    confirm: bool,
    tap: Option<Vec2>,
    /// 탭이면 누른 자리(지난 프레임의 누를 자리), 키·패드 확인이면 없음(고른 줄이 눌린 꼴).
    rect: Option<Rect>,
}

/// 헤드리스 시험에 알리는 기술 탭의 자리: 보이는 판 · 고른 칸 · 판 줄에 섰나(0270 C2).
type SkillFocus = (crate::plates::Plate, Option<crate::skills::Entry>, bool);

/// 헤드리스 시험에 알리는 목록 창의 고른 줄(0278 R13 — 개발판): 개발 창은 줄(`DevStart`의 Debug), 모션 보기는 동작 이름(달리기
/// 공격 줄은 `dash:` 앞머리). 스크립트가 줄을 세지 않고 이름으로 찾는다(`tools/test/scene.js` `devRow`).
#[cfg(feature = "dev")]
#[derive(Clone, Copy, PartialEq, Eq)]
enum RowFocus {
    Dev(dev::DevStart),
    Motion(&'static str, bool),
}

#[cfg(feature = "dev")]
impl RowFocus {
    /// 장면 이름 끝에 붙는 말(`@Gate` · `@attack` · `@dash:attack`).
    fn tag(self) -> String {
        match self {
            RowFocus::Dev(e) => format!("{e:?}"),
            RowFocus::Motion(a, dash) => format!("{}{a}", if dash { "dash:" } else { "" }),
        }
    }
}

/// 운영판에는 개발 창이 없다 — 고른 줄도 없다.
#[cfg(not(feature = "dev"))]
type RowFocus = ();

/// 페이지에 알린 것: 맨 위 장면 · 기술 탭의 자리 · 목록 창의 고른 줄 · "성소에 들다"가 흐르나.
type Reported = (Scene, Option<SkillFocus>, Option<RowFocus>, bool);

thread_local! {
    /// 마지막으로 페이지에 알린 맨 위 장면(`App::frame` — 바뀐 프레임에만 알린다).
    static REPORTED: std::cell::Cell<Option<Reported>> = const { std::cell::Cell::new(None) };
}

/// 개발판만의 칸(0075 · 0158): 개발 창이 고른 시나리오, 개발 창에서 시작한 판(기록 슬롯에 쓰지 않는다 — 슬롯을 고르면
/// `enter_slot`이 푼다), 개발 창에서 올린 기술 판(있으면 기록의 판 대신 — 기록에 쓰지 않는다).
#[cfg(feature = "dev")]
struct DevState {
    sc: Scenario,
    run: bool,
    board: Option<crate::skills::Board>,
    /// 놀이터의 보조무기(0272 1b — 개발 창 "보조무기 바꾸기"): 아레나에는 장비 탭이 없다. 기록에 쓰지 않는다.
    sub: Option<weapons::Id>,
    /// 놀이터의 보조무기 슬롯을 비운다(0276 — 개발 창 "보조무기 비우기"): 멘 무기 · 방패를 내리고 빈 손의 결로 선다. 기록에 쓰지 않는다.
    bare: bool,
}

#[cfg(feature = "dev")]
impl Default for DevState {
    fn default() -> Self {
        DevState {
            sc: Scenario::Boss,
            run: false,
            board: None,
            sub: None,
            bare: false,
        }
    }
}

/// 소리가 듣는 상태 변화(0064 M5): 지난 틱에 회피 중이었나, 적마다 소리를 낸 예고 번호, 문지기가 깨어나는 중이었나 —
/// 시작한 순간만 소리를 낸다.
#[derive(Default)]
struct Heard {
    was_dodging: bool,
    tele_was: Vec<u32>,
    was_waking: bool,
}

/// 메뉴 입력(0278 R24 — `App::menu`): 지금 칸의 커서와 굴리는 창(스택이 칸마다 맡아 둔다), 지난 프레임에 그린 누를 자리,
/// 붙잡아 둔 누름.
struct MenuInput {
    cursor: usize,
    /// 지금 화면의 굴리는 목록 창(메뉴·조작 설정 공통, 장면이 바뀌면 처음부터).
    view: ui::ListView,
    /// 지난 프레임에 그린 메뉴 줄·▲▼ 자리(탭 판정).
    hits: Vec<(Rect, Hit)>,
    /// 붙잡아 둔 메뉴 누름(0207): 눌린 꼴을 `PRESS_HOLD` 동안 보인 뒤 그 확인·탭을 장면에 넘긴다.
    press: Option<Press>,
    /// 누름을 붙잡은 동안 들어온 입력(0207) — 버리지 않고 넘긴 뒤 차례로(빠르게 연달아 누른 것이 사라지지 않게).
    press_queue: std::collections::VecDeque<Actions>,
}

/// 큰 창 · 탭의 화면 상태(0278 R24 — `App::tab`): 판 줄 · 탭 줄 · 작은 탭 줄의 가로 창, 기술 탭의 판과 줄 열림, 인형 · 도감 그림을
/// 손으로 돌린 각, 화면의 둘째 축 · 설명 칸의 굴림.
struct TabUi {
    /// 기술 탭에서 격자를 떠나기 전의 칸(0261 — 되돌리기 단추에서 위·판 줄에서 아래로 가면 거기로, 0270 C2).
    respec_from: usize,
    /// 기술 탭에 보이는 판(0270 C2 — 탭을 열 때 든 무기의 판, `App::open_plate`).
    skill_plate: crate::plates::Plate,
    /// 기술 탭 줄의 열림 연출(`skills::RowFades` — 그리기 전에 `ready_skill_rows`가 본다).
    skill_rows: skills::RowFades,
    /// 큰 창의 탭 줄이 넘칠 때의 가로 창(0233 — 탭을 넘겨도 같은 창이라 장면 칸이 아니라 여기 하나).
    tab_strip: crate::ui::Strip,
    /// 창 안의 작은 탭 줄(개발 창·모션 보기·도감의 묶음, 0257) — 한 번에 하나만 보인다.
    small_strip: crate::ui::Strip,
    /// 기술 탭의 판 줄(0275 — 넘치면 굴린다): 기술 탭을 열 때마다 새로(든 무기의 판이 보이게).
    plate_strip: crate::ui::Strip,
    /// 상태·장비 탭의 인형(0235): 손으로 돌린 각(라디안 — `None`이면 저절로 천천히 좌우로 흔든다)과 그린 자리(끌기가 그 위에서 시작했나).
    doll_yaw: Option<f32>,
    doll_rect: Option<Rect>,
    /// 도감 그림을 손으로 보기(끌어 돌리기·크게 — 2026-11-19).
    codex_view: codex::CodexView,
    /// 지금 화면의 둘째 축(슬롯 창의 단추). 장면이 바뀌면 처음으로.
    col: usize,
    /// 지금 화면의 글 칸(설명)을 몇 줄 내렸나(0068 — 넘치는 설명은 오른쪽 스틱·휠·끌기로 굴린다). 장면이 바뀌면 처음으로.
    scroll: f32,
}

/// 이 판에 붙은 것(0278 R24 — `App::this_run`): 새 판을 세울 때(`App::build_run`) 모두 처음으로 — 지역 잇기 · 멈춤 탭 ·
/// 튜토리얼과 상황 힌트 · 종소리의 길 안내.
struct RunState {
    /// 판의 지역 잇기(0118): 지금 지역(`stage::STAGES` 차례), 이 판에 이긴 문지기 수, 마지막으로 쉰 곳(지역을 바꿔도 남는다),
    /// 지역 사이 전환(0118 → 0174 뿌리 굴).
    stage: usize,
    down: usize,
    rest_at: Option<crate::save::RunAt>,
    /// 이 판에 쓰러뜨린 필드 보스(0270 D — 지역마다 한 비트): 쉬어도·지역을 떠났다 와도·이어 해도 그 판에는 다시 서지 않는다
    /// (`RunAt::fields`로 기록에). 새 판이면 0.
    fields_down: u32,
    /// 이 판에서 멈춤 창이 마지막에 보던 탭(0164) — 다음에 멈추면 거기서 열린다(`flow::pause`). 판에 붙은 한 칸: 새 판이면 상태로.
    pause_tab: flow::Tab,
    /// 튜토리얼(0167): 지난 걸음 수(기록 `Progress::tutorial`과 같은 값) + 이 판에서 본 것. 판에 붙는다.
    tutor: crate::tutorial::Tutor,
    /// 떠 있는 상황 힌트와 남은 시간(0209 T2).
    tip: Option<(crate::tutorial::Tip, f32)>,
    /// 이 판에서 띄운 상황 힌트(기록에 안 남는 것 — 등잔은 마셔 볼 때까지 체력이 반 아래로 갈 때마다, 0213).
    tips_now: u16,
    /// 튜토리얼이 센 적의 공격 번호(0209 T3 — 예고마다 한 번 센다).
    tell_seen: Vec<u32>,
    /// 종소리의 길 안내(0171): 문을 찾는 동안 가끔 우는 종의 시계 + 물결. 판에 붙는다(기록에 남지 않는다).
    knell: crate::knell::Knell,
}

impl RunState {
    /// 새 판: 첫 지역 `stage`에서, 튜토리얼은 기록의 지난 걸음 수부터.
    fn fresh(stage: usize, tutorial: u8) -> RunState {
        RunState {
            stage,
            down: 0,
            rest_at: None,
            fields_down: 0,
            pause_tab: flow::Tab::Status,
            tutor: crate::tutorial::Tutor::new(tutorial),
            tip: None,
            tips_now: 0,
            tell_seen: Vec::new(),
            knell: crate::knell::Knell::default(),
        }
    }
}

/// 기록을 쓰는 쪽의 장부(0278 R24 — `App::book`): 마지막으로 쓴 것(바뀌었을 때만 쓴다), 슬롯, 옛 prefs 칸, 실패, 플레이 시간의 나머지.
struct SaveBook {
    saved: crate::save::Progress,
    /// 지금 슬롯(0066)과 슬롯 셋의 요약(타이틀 "이어하기"·슬롯 창, 쓸 때마다 고친다).
    slot: usize,
    slots: [Option<crate::save::Summary>; crate::save::SLOTS],
    /// 옛 prefs 칸 21(든 무기) — 읽은 값 그대로 다시 적는다(0066에서 기록으로 옮김).
    legacy_weapon: u8,
    /// 저장에 실패한 채인가(실패가 시작될 때 한 번만 알린다 — KB `roguelite-save-slots-and-continue`).
    save_failed: bool,
    /// 플레이 시간의 1초 아래 나머지(0081).
    play_acc: f32,
}

/// 잠깐 뜨는 알림(0278 R24 — `App::notice`).
struct Notices {
    /// 잠깐 뜨는 알림(0064 M3 — 상자에서 얻은 것 등)과 남은 초.
    toast: Option<(String, f32)>,
    /// 지금 알림이 끝난 뒤에 뜰 알림(0223 V4 — 여벌을 얻은 틱에 핵·엘리트의 알림도 뜬다: 한 칸뿐이라 덮어쓰면 하나가 안 보인다).
    toast_wait: Vec<String>,
}

pub struct App {
    /// 장면 스택(0066): 맨 위가 지금 장면, 아래 칸은 커서·창을 맡아 둔다. 지금 칸의 커서·창은 `cursor`·`view`.
    stack: stack::Stack,
    /// 지금 칸의 커서 · 굴리는 창과 누를 자리 · 붙잡은 누름(0278 R24 — 메뉴 입력의 묶음).
    menu: MenuInput,
    /// 큰 창 · 탭의 화면 상태(0278 R24 — 탭을 넘겨도 이어 가는 것: 스택 칸에 두지 않았다).
    tab: TabUi,
    /// 이 판에 붙은 것(0278 R24 — 새 판을 세울 때 함께 처음으로: `RunState::fresh`).
    this_run: RunState,
    /// 기록을 쓰는 쪽의 장부(0278 R24 — 마지막으로 쓴 것 · 슬롯 · 실패 · 플레이 시간의 나머지).
    book: SaveBook,
    /// 잠깐 뜨는 알림과 그 뒤에 뜰 것(0278 R24).
    notice: Notices,
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
    /// 싸움 화면의 연출 칸(0251 R5 — `interlude`): 뿌리 굴·핵의 반응·쓰러짐·깨어나는 첫 장면·불 붙는 화로 가운데 하나.
    /// 그동안 조작이 없고 세계는 멈추거나 빈 입력으로 흐른다. 깨어나는 장면은 타이틀 바탕에서 0초에 멈춰 있다(엎드린 방랑자).
    interlude: Option<interlude::Interlude>,
    /// 손으로 하는 일의 몸짓(0196 줍기·상자 열기 → 0197 장치 당기기)과 흐른 초 — 렌더만 안다(월드는 그 일을 바로 끝낸다).
    /// 서 있는 동안만, 움직이거나 다른 동작이 시작되면 그친다.
    gesture: Option<(&'static str, f32)>,
    /// 주민 말풍선(0122): (주민 — 지금 지역의 번호, 몇째 말, 남은 초). 멈추지 않는다.
    bubble: Option<(usize, usize, f32)>,
    /// 새긴 돌 글 판(0129): (지금 지역의 돌 번호, 남은 초).
    stone_card: Option<(usize, f32)>,
    /// 실제 시간(전환의 작은 움직임 — 지형을 굽는 동안에도 흐른다).
    clock: f32,
    /// 개발판만의 칸(시나리오·개발에서 시작한 판·개발 기술 판 — 0251 R10).
    #[cfg(feature = "dev")]
    dev: DevState,
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
    /// 설정 화면의 값(흔들림·소리·예고·시점 민감도·터치 크기 — prefs에 적는 기기의 것, 0251 R10).
    opts: settings::Opts,
    /// 소리 묶음(main이 올린 뒤 `set_sfx`), 지난 틱의 회피·적 예고(시작한 순간만 소리).
    sfx: Option<crate::sfx::Sfx>,
    /// 소리가 듣는 상태 변화(시작한 순간만 — 0251 R10).
    heard: Heard,
    /// 앱이 내려간 것을 엔진에서 듣는 귀(0257)와 아직 싸움 틱이 받지 않은 내려감.
    away: away::Away,
    went_away: bool,
    /// 그림 이펙트(0141): 월드 사건·상태 변화를 듣고 입자를 낸다 — 월드는 모른다(소리와 같은 길).
    fx: crate::fx::Fx,
    /// 지난 프레임에 그린 화면 자리(커서 → 땅, 0058).
    frame: Option<Frame>,
    /// OS 커서를 숨겼나(싸움 중 마우스 조준이면 숨기고 조준 표시를 그린다, 0058).
    mouse_hidden: bool,
    /// 갈 곳이 화면 밖에 있은 초(0060 W1 — 잠깐 뒤에 가장자리 화살).
    goal_off: f32,
    /// 여정(0183): 알린 목표(처음 본 기록은 조용히 채운다 — `None` = 아직 안 봤다).
    journey_told: Option<Vec<&'static str>>,
    /// 기록(0064 M2): 지금 것(마지막으로 쓴 것과 슬롯은 `book`).
    progress: crate::save::Progress,
}

/// 알림이 떠 있는 시간(초).
const TOAST_TIME: f32 = 3.5;

/// 깨어나는 첫 장면(0171)의 누운 자리: 몸이 화면에 옆으로 놓이게(뒤통수만 보이면 일어나는 몸짓이 두건에 가린다) — 머리는
/// 꺼진 화로 쪽 옆으로. 화면 위 = 월드 -z라서 화로까지의 앞뒤 몫을 줄여 거의 옆을 보게 한다. 화로가 없는 서막 숲(0262)은 숲 끝 쪽.
fn lay_by_hearth(run: &StageRun, world: &mut World) {
    let at = world.player.pos;
    let hearth = run
        .rests
        .iter()
        .copied()
        .min_by(|a, b| a.distance(at).total_cmp(&b.distance(at)))
        .or(run.exit_at);
    if let Some(h) = hearth {
        world.player.facing = (h.x - at.x).atan2_d((h.z - at.z) * 0.3);
    }
}

/// 터치로 끄는 버튼의 미리보기(0217): 버튼이 아니라 그 버튼의 지금 일로 고른다 — 원거리 무기는 평타 버튼(`Attack`)이 사격·
/// 특수(`Cast`)가 수류탄(0223 → 0227: 이 맞바꿈을 빠뜨려 쇠뇌로 사격을 끌면 수류탄 원이, 수류탄을 끌면 사격 띠가 그려졌다).
fn touch_aim_show(k: touch::Kind, ranged: bool) -> Option<AimShow> {
    match k {
        // 근접 무기의 평타는 끌지 않는다(`touch::Ctx::aim_attack`).
        touch::Kind::Attack => ranged.then_some(AimShow::Cast),
        touch::Kind::Cast if ranged => Some(AimShow::Grenade),
        touch::Kind::Cast => Some(AimShow::Cast),
        touch::Kind::Skill1 => Some(AimShow::Skill(0)),
        touch::Kind::Skill2 => Some(AimShow::Skill(1)),
        _ => None,
    }
}

impl App {
    /// 땅 위 조준 표시의 꼴(0217 → 0219): 터치로 끄는 버튼이면 그 일, 아니면 원거리가 겨누는 쪽이 있을 때 —
    /// 쏘거나 모으는 동안은 띠(어느 장치나), 숄더뷰는 가는 줄 + 멈추는 자리, 쿼터뷰의 패드 스틱은 띠. 마우스 커서는 그리지 않는다
    /// (커서 자체가 겨누는 자리다).
    fn aim_show(&self) -> Option<AimShow> {
        if let Some(k) = self.input.aiming() {
            return touch_aim_show(k, self.world.player.weapon.is_ranged());
        }
        self.world.aim_line?;
        let firing = matches!(
            self.world.player.state,
            crate::world::PState::Cast { .. } | crate::world::PState::CastCharge { .. }
        );
        // 숄더뷰는 바닥에 그리지 않는다 — 화면 가운데 크로스헤어(`draw_crosshair`, 0222). 쿼터뷰는 조준할 때만(쏘거나 모으는 동안·
        // 패드 스틱) 연한 띠.
        if self.cam.mode == Mode::Free {
            None
        } else if firing || input::device() == input::Device::Pad {
            Some(AimShow::Cast)
        } else {
            None
        }
    }

    /// 개발 판(DEV에서 시작한 스테이지)인가 — 운영판은 늘 거짓.
    fn dev_run_now(&self) -> bool {
        #[cfg(feature = "dev")]
        return self.dev.run;
        #[cfg(not(feature = "dev"))]
        false
    }

    /// 들 수 있는 무기(0117): 기록에 가진 것 — 개발 판·놀이터(시나리오)는 다(무기를 시험하려고).
    fn arms(&self) -> Vec<weapons::Id> {
        #[cfg(feature = "dev")]
        if self.dev.run || self.stack.base() == Scene::Sandbox {
            return weapons::IDS.to_vec();
        }
        self.progress.owned_weapons()
    }

    pub fn new() -> Self {
        let t = Tuning::default();
        // 저장한 설정(0024·0052·0058): 언어·키 배치·흔들림·무기·마우스 조준.
        let mut legacy_weapon = 0;
        let mut opts = settings::Opts::default();
        if let Some(p) = crate::prefs::load() {
            if let Some(l) = p.lang {
                i18n::set_lang(l);
            }
            input::set_bindings(p.bindings);
            input::set_mouse_aim(p.mouse);
            opts = settings::Opts::of(&p);
            legacy_weapon = p.weapon;
        }
        crate::touch::set_size(crate::prefs::TOUCH_SIZES[opts.touch_size as usize]);
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
        // 가지지 않은 무기는 들 수 없다(0117) — 새 판은 맨손(0220).
        let weapon = if progress.owns_weapon(weapon) {
            weapon
        } else {
            weapons::Id::Fists
        };
        let first = first_stage(&progress, false);
        let (mut run, mut world) = StageRun::start(crate::stage::STAGES[first]);
        // 새 기록(0171): 깨어난 자리의 화로는 꺼져 있다 — 타이틀 바탕부터.
        if !progress.lit {
            run.chill_hearth();
        }
        if !progress.woke {
            lay_by_hearth(&run, &mut world);
        }
        world.set_weapon(weapon);
        world.set_skills(progress.board.loadout(progress.skills.0, progress.ult));
        world.set_sub_skills(progress.board.loadout(progress.sub_skills.0, progress.ult));
        world.tree = progress.board.cells;
        let cam = Camera::new(world.player.pos, &t);
        App {
            stack: stack::Stack::new(Scene::Title),
            menu: MenuInput {
                cursor: 0,
                view: ui::ListView::default(),
                hits: Vec::new(),
                press: None,
                press_queue: std::collections::VecDeque::new(),
            },
            tab: TabUi {
                respec_from: 0,
                skill_plate: crate::plates::Plate::Body,
                skill_rows: skills::RowFades::default(),
                tab_strip: crate::ui::Strip::default(),
                small_strip: crate::ui::Strip::default(),
                plate_strip: crate::ui::Strip::default(),
                doll_yaw: None,
                doll_rect: None,
                codex_view: Default::default(),
                col: 0,
                scroll: 0.0,
            },
            this_run: RunState::fresh(first, progress.tutorial),
            notice: Notices {
                toast: None,
                toast_wait: Vec::new(),
            },
            input: Input::new(),
            #[cfg(feature = "dev")]
            lab: Lab::new(),
            renderer: Renderer::new(),
            ui: Ui::new(),
            cam,
            world,
            run,
            interlude: (!progress.woke)
                .then(|| interlude::Interlude::Wake(crate::wake::Wake::default())),
            gesture: None,
            bubble: None,
            stone_card: None,
            clock: 0.0,
            t,
            #[cfg(feature = "dev")]
            dev: DevState::default(),
            ending: 0.0,
            acc: 0.0,
            dev_overlay: false,
            step_clock: false,
            ctl: controls::Controls::default(),
            perf: Perf::default(),
            sums: (0.0, 0.0, 0.0, 0),
            weapon,
            opts,
            sfx: None,
            heard: Heard::default(),
            away: away::Away::new(),
            went_away: false,
            fx: crate::fx::Fx::default(),
            frame: None,
            mouse_hidden: false,
            goal_off: 0.0,
            journey_told: None,
            progress: progress.clone(),
            book: SaveBook {
                saved: progress,
                slot,
                slots: crate::save::summaries(),
                legacy_weapon,
                save_failed: false,
                play_acc: 0.0,
            },
        }
    }

    /// 새 판(0122 — 판 수를 센다: 주민 재방문 "처음 본 판이 끝난 뒤의 다른 판"). 이어하기는 `build_run`.
    fn restart(&mut self) {
        self.progress.runs = self.progress.runs.saturating_add(1);
        self.build_run();
    }

    /// 첫 지역을 새로 세운다(판 수는 그대로).
    fn build_run(&mut self) {
        let first = first_stage(&self.progress, self.dev_run_now());
        let (run, world) = StageRun::start(crate::stage::STAGES[first]);
        self.run = run;
        self.world = world;
        self.fx.clear();
        self.this_run = RunState::fresh(first, self.progress.tutorial);
        // 깨어나는 첫 장면(0171): 아직 본 적 없는 기록의 판은 그 장면으로 열린다(개발 창의 판은 아니다).
        self.interlude = (!self.progress.woke && !self.dev_run_now())
            .then(|| interlude::Interlude::Wake(crate::wake::Wake::default()));
        if self.interlude.is_some() {
            lay_by_hearth(&self.run, &mut self.world);
        }
        self.cam.close = 0.0;
        // 잔불은 판을 넘어 남는다(0064 M1). 장신구·성장은 능력치로, 연 상자는 비어 있다(M3).
        self.world.set_embers(self.progress.embers);
        self.world.apply_stats(self.progress.stats(), true);
        self.seed_run();
        self.cam = self.cam.renew(self.world.player.pos, &self.t);
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
        world.set_embers(self.world.embers());
        self.run = run;
        self.world = world;
        self.fx.clear();
        self.this_run.stage = s;
        self.world.apply_stats(self.progress.stats(), false);
        self.seed_run();
    }

    /// 지역을 세운 뒤 기록에서 채운다: 무기·기술(끼운 스킬과 연 패시브 칸)·연 상자(지역마다 번호 `지역 × 1000 + 번호`, 0118)·주운 것·쓰러뜨린 엘리트·문,
    /// 이 판에 이 지역 문지기를 이겼으면 바닥 길이 열린 채.
    fn seed_run(&mut self) {
        self.arm_world();
        // 깨어난 자리의 화로(0171): 불을 붙인 적 없는 기록이면 첫 지역의 그 화로가 꺼져 있다.
        if self.this_run.stage == 0 && !self.progress.lit && !self.dev_run_now() {
            self.run.chill_hearth();
        }
        self.apply_board();
        let base = self.this_run.stage as u32 * crate::stage::CHESTS_PER_STAGE;
        let opened: Vec<u32> = self
            .progress
            .chests
            .iter()
            .filter(|&&c| c / crate::stage::CHESTS_PER_STAGE == self.this_run.stage as u32)
            .map(|c| c - base)
            .collect();
        self.run.set_opened(&opened);
        // 깨어난 자리 곁의 검(0220): 아직 검이 없는 기록의 첫 지역에만 — 서막 숲(0262)에서 깨어나면 거기에도(줍지 않고 지나면 성소에).
        let woke_here = self.this_run.stage == 0 || self.this_run.stage == crate::stage::PROLOGUE;
        if woke_here && !self.progress.owns_weapon(weapons::Id::Sword) && !self.dev_run_now() {
            self.run.lay_start_sword();
        }
        self.run.set_found(&self.progress.finds);
        self.run.set_beaten(&self.progress.elites);
        // 필드 보스(0270 D): 기록에서 처음인지, 이 판에 이미 쓰러뜨렸는지(다시 서지 않는다).
        let felled = |f: &String| f == self.run.def.find_prefix;
        self.run
            .set_field_beaten(self.progress.fields.iter().any(felled));
        if self.this_run.fields_down >> self.this_run.stage & 1 == 1 {
            self.run.set_field_down(&mut self.world);
        }
        self.lay_ground();
        let open = self
            .progress
            .gates
            .iter()
            .any(|g| g == self.run.def.find_prefix);
        self.run.set_gate_opened(&mut self.world, open);
        // 주민 재방문(0122, 0116): 처음 본 판보다 뒤의 판이면 한 번 바뀐다(단방향, 기록).
        self.bubble = None;
        for k in 0..self.run.folk.list.len() {
            let id = crate::residents::RESIDENTS[self.run.folk.list[k].0].id;
            self.run.folk.changed[k] = crate::residents::settle(
                id,
                &self.progress.residents_seen,
                &mut self.progress.residents_changed,
                self.progress.runs,
            );
        }
        // 처음 이기는 판이면 핵이 스스로 다가온다(lore CANON, 0105) — 첫 지역 문지기만(둘째 문지기는 C3).
        self.run.first_core = match self.this_run.stage {
            0 => self.progress.clears == 0,
            1 => self.progress.bell_cores == 0,
            2 => self.progress.moss_cores == 0,
            _ => self.progress.overseer_cores == 0,
        };
        if self.this_run.down > self.this_run.stage {
            self.run.down_open = true;
            self.run.open_down(&mut self.world);
        }
    }

    /// 소리 묶음을 받는다(0064 M5 — main이 올린 뒤). 환경음을 튼다.
    pub fn set_sfx(&mut self, sfx: crate::sfx::Sfx) {
        sfx.set_volume(self.opts.volume as f32 / 10.0);
        sfx.ambience(true);
        self.sfx = Some(sfx);
    }

    fn snd(&self, id: crate::sfx::Id) {
        if let Some(s) = &self.sfx {
            s.play(id);
        }
    }

    fn set_volume(&mut self, v: u8) {
        self.opts.volume = v;
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
        let next = arms[(i + step).rem_euclid(arms.len() as i32) as usize];
        self.wield(next);
    }

    /// 무기 둘(주·부, 0223)을 월드에. 몽크(0286)를 들었으면 보조무기 칸은 비어 있다(기록에 남은 것도 여기서 내린다 — 장비가 바뀌는
    /// 길은 모두 여기를 지난다).
    pub(super) fn arm_world(&mut self) {
        if self.weapon.one_slot() {
            self.progress.sub = None;
            self.progress.shield = None;
        }
        self.world.set_weapon(self.weapon);
        self.world.set_sub(self.progress.sub);
        self.world.set_shield(self.progress.shield);
        // 빈 손의 결(0276): 보조무기 슬롯이 비었나 — 장비가 바뀌는 길은 모두 여기를 지난다(들기 · 보조무기 · 방패 · 판 세우기).
        self.world
            .set_open_hand(self.progress.sub.is_none() && self.progress.shield.is_none());
        self.renderer.shield = self.progress.shield;
        // 든 것이 바뀌면 능력치도 다시 합친다(벼린 단 0180 · 낀 아이템의 옵션 0223 V4 — 무기는 든 것만).
        self.world.apply_stats(self.progress.stats(), false);
    }

    /// 주무기를 바꾼다(0223): 부무기 칸에 있던 것이면 둘을 맞바꾼다(같은 무기 두 칸 금지 — 맨손은 부무기로 남기지 않는다). 지금 판에 바로, 저장.
    /// 주무기만 드는 종류(0286 몽크 — 맨손)를 들면 보조무기 칸에 있던 것(무기 · 방패)은 가방으로 돌아간다(가진 것은 그대로 — 칸만 빈다).
    pub(super) fn wield(&mut self, w: weapons::Id) {
        let old = self.weapon;
        if self.progress.sub == Some(w) {
            self.progress.sub = old.slings().then_some(old);
            self.swap_kits();
        }
        if w.one_slot() {
            self.progress.sub = None;
            self.progress.shield = None;
        }
        self.weapon = w;
        self.progress.weapon = Some(w);
        self.arm_world();
        self.persist();
    }

    /// 주·부무기를 장비 탭에서 맞바꿨다(0223 V3): 스킬 칸도 무기를 따라간다 — 싸움 중의 바꿔 들기(`on_weapon_swap`)와 같은 규칙
    /// (안 그러면 맞바꾼 두 무기의 칸 넷이 모두 안 맞는 스킬이 된다). 낀 아이템은 `settle_items`가 따라가게 한다.
    fn swap_kits(&mut self) {
        std::mem::swap(&mut self.progress.skills, &mut self.progress.sub_skills);
        self.apply_board();
    }

    /// 부무기를 정한다(0223): 지금 주무기를 고르면 둘을 맞바꾼다(주무기 자리에는 옛 부무기, 없으면 맨손). 주무기만 드는 종류(0286
    /// 몽크)를 든 동안은 칸이 잠겨 무기를 메지 못하고, 메지 못하는 무기(맨손)는 보조무기가 되지 않는다.
    pub(super) fn set_sub_weapon(&mut self, s: Option<weapons::Id>) {
        if s.is_some_and(|w| !w.slings()) || (s.is_some() && self.weapon.one_slot()) {
            return;
        }
        if s.is_some() && s == Some(self.weapon) {
            self.weapon = self.progress.sub.unwrap_or(weapons::Id::Fists);
            self.progress.weapon = Some(self.weapon);
            self.swap_kits();
        }
        self.progress.sub = s;
        // 보조무기 칸에는 하나만(0224): 무기를 들면 방패는 내린다.
        if s.is_some() {
            self.progress.shield = None;
        }
        self.arm_world();
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
            shake: self.opts.shake,
            volume: self.opts.volume,
            tells: self.opts.tells,
            tell_timing: self.opts.tell_timing,
            look: self.opts.look_sens,
            touch_size: self.opts.touch_size,
            // PC만 기억한다(0135) — 웹은 브라우저가 Esc로 끌 수 있고 다음에 켤 때 사용자 입력 없이 못 연다.
            fullscreen: crate::DESKTOP && crate::screen::active(),
            // 든 무기는 슬롯 기록으로 옮겼다(0066) — 옛 칸은 처음 읽은 값 그대로 둔다(옛 슬롯 1의 기본값).
            weapon: self.book.legacy_weapon,
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
        let go = self.codex_entry(go);
        if let Some((cursor, view)) = self.stack.go(go, self.menu.cursor, self.menu.view) {
            self.menu.cursor = cursor;
            self.menu.view = view;
            self.tab.col = 0;
            self.tab.scroll = 0.0;
            self.menu.hits.clear();
            self.ctl = controls::Controls::default();
        }
    }

    /// 한 프레임.
    /// 창 안의 작은 탭 줄(0257)이 넘쳐 굴리는 동안, 그 위의 끌기·휠은 탭 줄을 굴린다 — 몸의 목록에는 넘기지 않는다.
    pub(super) fn small_strip_input(&mut self, a: &Actions) -> Actions {
        strip_input(&mut self.tab.small_strip, a)
    }

    pub fn frame(&mut self) {
        crate::prof::frame_start();
        // 헤드리스 시험의 개발 명령(0278 R13 — 웹 개발판 `window.knellDev`, 쌓인 것을 이 프레임에 차례로).
        #[cfg(feature = "dev")]
        while let Some(name) = crate::screen::dev_command() {
            self.dev_command(&name);
        }
        let cpu_start = get_time();
        // 헤드리스 시험의 기다림(`screen::report_scene`): 맨 위 장면이 바뀐 프레임에만 이름을 알린다.
        // 기술 탭은 고른 칸도 붙인다(개발판 — `Skills { host: Dev } @Skill(Embers)`): 칸의 자리는 판이 바뀌면 낡으니 시험이 이름으로 찾는다.
        let top = self.scene();
        #[cfg(feature = "dev")]
        let focus = match top {
            Scene::Skills { .. } => Some((
                self.tab.skill_plate,
                crate::skills::ENTRIES.get(self.menu.cursor).copied(),
                self.menu.cursor == skills::STRIP,
            )),
            _ => None,
        };
        #[cfg(not(feature = "dev"))]
        let focus: Option<SkillFocus> = None;
        #[cfg(feature = "dev")]
        let row = match top {
            Scene::Dev { tab } => dev::dev_items(tab as usize)
                .get(self.menu.cursor)
                .map(|e| RowFocus::Dev(*e)),
            Scene::Motions { weapon, .. } => self
                .motion_focus(weapon)
                .map(|(a, d)| RowFocus::Motion(a, d)),
            _ => None,
        };
        #[cfg(not(feature = "dev"))]
        let row: Option<RowFocus> = None;
        // "성소에 들다"(0262)가 흐르는 동안은 싸움 화면 이름 뒤에 ` ~arrive`(앞머리 `Play`로 기다리는 스크립트는 그대로 맞는다).
        let cut = self.arriving().is_some();
        if REPORTED.with(|r| r.replace(Some((top, focus, row, cut))))
            != Some((top, focus, row, cut))
        {
            let base = match (focus, cut) {
                // 기술 탭(0270 C2): 보이는 판 + 고른 칸(판 줄이면 `@Strip`) — `Skills { host: Hearth } [Class(Sword)] @Skill(Cut)`.
                (Some((p, Some(e), _)), _) => format!("{top:?} [{p:?}] @{e:?}"),
                (Some((p, None, true)), _) => format!("{top:?} [{p:?}] @Strip"),
                (Some((p, None, false)), _) => format!("{top:?} [{p:?}] @Respec"),
                (None, true) => format!("{top:?} ~arrive"),
                (None, false) => format!("{top:?}"),
            };
            // 목록 창의 고른 줄(0278 R13): `Dev { tab: 3 } @Gate` · `Motions { … } @attack`.
            #[cfg(feature = "dev")]
            let name = match row {
                Some(r) => format!("{base} @{}", r.tag()),
                None => base,
            };
            #[cfg(not(feature = "dev"))]
            let name = base;
            crate::screen::report_scene(&name);
        }
        // 터치의 상황 버튼(0053): 쉼터 곁이면 점프가 쉬기, 락온할 적이 있거나 락온 중이면 락온, 마실 수 있으면 등잔.
        let play = self.scene().ticks_world();
        let pl = &self.world.player;
        self.input.set_touch_ctx(crate::touch::Ctx {
            rest: play && self.world.interact_here,
            near: self
                .run
                .near()
                .map_or(crate::touch::Near::Rest, |n| hud::near_button(n).icon),
            lock: play && (self.world.lock.is_some() || self.world.lock_candidate()),
            flask: play && pl.flask > 0 && pl.hp < pl.max_hp,
            ult: play && pl.skill.gauge >= 1.0,
            // 끌어 조준(0217): 그 버튼에 올라간 일이 쪽을 가지면 — 무기·끼운 스킬이 바뀌어도 같은 규칙.
            // 0223: 원거리 무기는 평타 버튼이 사격·특수가 수류탄이라 둘 다 겨눈다. 근접은 특수(마법 탄)만.
            aim_attack: pl.weapon.is_ranged(),
            aim_cast: true,
            swap: play && pl.sub.is_some(),
            aim_skills: [0, 1].map(|i| {
                pl.skill.rank[i] > 0 && self.world.skill_eff(pl.skill.worn[i]).skill.form.aims()
            }),
            charging: matches!(pl.state, crate::world::PState::CastCharge { .. }),
            ranged: pl.weapon.is_ranged(),
            // 숄더뷰(0229)에서는 한 대상을 겨누는 버튼을 끌면 시점이 돈다 — 락온 중은 대상을 보니 쿼터뷰처럼 쪽을 끈다(0217).
            look_aim: self.cam.mode == Mode::Free,
            sight_skills: [0, 1].map(|i| {
                pl.skill.rank[i] > 0 && self.world.skill_eff(pl.skill.worn[i]).skill.form.sighted()
            }),
            // 0236: 쿼터뷰가 아니면 땅 돌리기를 숨긴다(숄더뷰는 끌기, 락온은 월드가 막는다), 끼운 스킬이 없는 칸도.
            hide_turn: self.cam.mode != Mode::Quarter || self.world.lock.is_some(),
            bare: [0, 1].map(|i| pl.skill.known[i] == 0),
        });
        self.input.poll(self.scene().is_menu());
        // 앱이 내려갔으면(0257) 다음 싸움 틱이 멈춤 창을 연다 — 싸움이 아닌 장면은 이미 입력을 기다리니 버린다.
        if self.away.take() {
            self.went_away = self.scene().ticks_world();
        }
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
        {
            let _p = crate::prof::span("tick");
            while self.acc >= TICK_DT {
                let a = self.input.take();
                self.tick(a);
                self.acc -= TICK_DT;
            }
        }
        // 글자 아틀라스를 그리기 전에 다 굽는다(프레임 중간에 키우면 앞서 쌓은 글자가 지워진 텍스처를 가리킨다).
        crate::text::warm();
        {
            let _p = crate::prof::span("draw");
            self.draw();
        }
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
        crate::prof::frame_end(|| format!("{:?}", self.scene()));
    }

    /// 화면 아래 안내 줄 위까지(메뉴가 굴리는 아래 끝).
    fn menu_bottom(&self, f: &Frame) -> f32 {
        f.oy + f.h - self.ui.px(1.0) * 2.8
    }

    fn tick(&mut self, a: Actions) {
        // 카메라의 지난 틱 = 지금(0173): 이 틱에 카메라가 안 움직이면(멈춤 창 아래) 그림이 틱 사이를 오가지 않는다.
        self.cam.settle();
        let (a, injected) = self.hold_press(a);
        let (scene0, cursor0, back0) = (self.scene(), self.menu.cursor, a.back || a.pause);
        let played0 = self.sfx.as_ref().map_or(0, |s| s.played());
        self.tick_scene(a);
        // 메뉴 소리(0064 M5, KB `game-audio-without-a-composer` "메뉴 소리는 한 곳에서"): 그 사이에 다른 소리(사기·안 됨 등)가
        // 나지 않았을 때만 — 뒤로 > 확인(장면이 바뀜) > 커서.
        let quiet = self.sfx.as_ref().map_or(0, |s| s.played()) == played0;
        if scene0.is_menu() && quiet {
            // 모션 보기는 재생 시각을 장면에 들고 다녀 틱마다 값이 바뀐다 — 그것은 장면이 바뀐 것이 아니다(0256: 확인 소리가 틱마다 났다).
            let (now, was) = (self.scene(), scene0);
            #[cfg(feature = "dev")]
            let moved =
                !matches!((now, was), (Scene::Motions { .. }, Scene::Motions { .. })) && now != was;
            #[cfg(not(feature = "dev"))]
            let moved = now != was;
            if moved && !(injected && !back0) {
                self.snd(if back0 {
                    crate::sfx::Id::UiBack
                } else {
                    crate::sfx::Id::UiConfirm
                });
            } else if self.menu.cursor != cursor0 {
                self.snd(crate::sfx::Id::UiMove);
            }
        }
    }

    /// 메뉴 누름 붙잡기(0207, 사장 "UI 메뉴에서 버튼 눌리는 감이 없으니"): 메뉴 장면의 확인·줄 탭은 바로 넘기지 않고
    /// `PRESS_HOLD` 동안 눌린 꼴(`Ui::press`)을 보인 뒤 그대로 넘긴다 — 누르는 순간 장면이 바뀌면 눌린 꼴이 한 프레임도 안 보인다
    /// (UI 피드백 100~160 ms, 넘어가기 전 짧은 지연). 소리는 누르는 순간에. 붙잡은 동안의 입력은 줄 세워 두었다
    /// 넘긴 뒤 차례로(그 사이 커서가 먼저 움직이지 않게).
    /// ◀▶·▲▼·끌기·휠은 붙잡지 않는다(값·창을 바로 옮기는 일). 돌려주는 참 = 이번 틱이 붙잡았던 누름을 넘긴 틱.
    fn hold_press(&mut self, a: Actions) -> (Actions, bool) {
        if let Some(p) = &mut self.menu.press {
            p.left -= TICK_DT;
            if is_edge(&a) {
                self.menu.press_queue.push_back(a);
            }
            if p.left > 0.0 {
                return (Actions::default(), false);
            }
            let p = self.menu.press.take().unwrap();
            return (
                Actions {
                    confirm: p.confirm,
                    tap: p.tap,
                    ..Actions::default()
                },
                true,
            );
        }
        // 붙잡은 동안 쌓인 입력부터(이번 틱의 것은 뒤로).
        let a = match self.menu.press_queue.pop_front() {
            Some(q) => {
                if is_edge(&a) {
                    self.menu.press_queue.push_back(a);
                }
                q
            }
            None => a,
        };
        let scene = self.scene();
        if !scene.is_menu() || matches!(scene, Scene::Controls) {
            return (a, false);
        }
        let popup = scene.is_popup();
        let tapped = a
            .tap
            .and_then(|p| self.menu.hits.iter().find(|(r, _)| r.contains(p)))
            .filter(|(r, h)| press_shows(popup, *h, *r))
            .map(|(r, _)| *r);
        if !(a.confirm || tapped.is_some()) {
            return (a, false);
        }
        self.menu.press = Some(Press {
            left: PRESS_HOLD,
            confirm: a.confirm,
            tap: a.tap,
            rect: tapped,
        });
        self.snd(crate::sfx::Id::UiConfirm);
        (
            Actions {
                confirm: false,
                tap: None,
                ..a
            },
            false,
        )
    }

    fn tick_scene(&mut self, mut a: Actions) {
        // 알림은 어느 장면에서나 줄어든다(가게의 "잔불이 모자란다"가 떠날 때까지 남았다).
        if let Some((_, t)) = &mut self.notice.toast {
            *t -= TICK_DT;
        }
        self.notice.toast = self.notice.toast.take().filter(|(_, t)| *t > 0.0);
        if self.notice.toast.is_none() && !self.notice.toast_wait.is_empty() {
            self.notice.toast = Some((self.notice.toast_wait.remove(0), TOAST_TIME));
        }
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
        let swapped = flow::tab_of(self.scene()).is_some() && self.tab_input(&mut a);
        if !swapped {
            self.tick_top(a);
        }
        // 창이 열린 동안 카메라는 계속 따라간다(타이틀 바탕·화로) — 판 위의 멈춤 아래에서는 그대로 선다.
        if self.scene().is_menu() && !self.paused() {
            self.follow_camera();
        }
    }

    /// 깨어나는 첫 장면(0171)이 지금 화면에 흐르나 — 싸움 화면이 맨 위일 때만(타이틀 바탕·멈춤 밑에서는 아니다).
    fn waking(&self) -> Option<crate::wake::Wake> {
        self.wake().filter(|_| self.scene() == Scene::Play)
    }

    /// 연출 칸 읽기(0251 R5) — 그 장면일 때만.
    fn wake(&self) -> Option<crate::wake::Wake> {
        match self.interlude {
            Some(interlude::Interlude::Wake(w)) => Some(w),
            _ => None,
        }
    }

    fn fall(&self) -> Option<crate::fall::Fall> {
        match self.interlude {
            Some(interlude::Interlude::Fall(f)) => Some(f),
            _ => None,
        }
    }

    /// "성소에 들다"(0262)가 지금 화면에 흐르나 — 싸움 화면이 맨 위일 때만.
    fn arriving(&self) -> Option<crate::arrive::Arrive> {
        match self.interlude {
            Some(interlude::Interlude::Arrive(a)) if self.scene() == Scene::Play => Some(a),
            _ => None,
        }
    }

    /// 보는 장면(깨어남 0171 · 성소에 들다 0262)이 흐르나 — 그동안 스테이지 대본·튜토리얼·종소리의 시계가 멈춘다.
    fn cutscene(&self) -> bool {
        matches!(
            self.interlude,
            Some(interlude::Interlude::Wake(_) | interlude::Interlude::Arrive(_))
        )
    }

    fn passage(&self) -> Option<crate::descent::Passage> {
        match self.interlude {
            Some(interlude::Interlude::Passage(p)) => Some(p),
            _ => None,
        }
    }

    fn echo(&self) -> Option<f32> {
        match self.interlude {
            Some(interlude::Interlude::Echo(t)) => Some(t),
            _ => None,
        }
    }

    /// 깨어나는 장면을 끝낸다(쉼터에서 잇는 판 — 0171). 다른 연출(쓰러짐의 어둠 속에서 다른 지역으로 다시 설 때)은 건드리지 않는다.
    fn end_wake(&mut self) {
        if self.wake().is_some() {
            self.interlude = None;
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
            Scene::Bag { host } => self.tick_bag(&a, host),
            Scene::BagAct => self.tick_bag_act(&a),
            s @ (Scene::Settings { .. } | Scene::Shop | Scene::Result { .. }) => {
                self.tick_menu(&a, s)
            }
            #[cfg(feature = "dev")]
            Scene::Dev { tab } => self.tick_dev(&a, tab as usize),
            #[cfg(feature = "dev")]
            Scene::Motions {
                weapon,
                speed,
                spin,
                t_ms,
            } => self.tick_motions(&a, weapon, speed, spin, t_ms),
            #[cfg(feature = "dev")]
            s @ (Scene::Lab | Scene::LabGroup { .. }) => self.tick_lab(a, s),
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
            | Scene::Bag { .. }
            | Scene::BagAct
            | Scene::EquipPick { .. } => None,
            #[cfg(feature = "dev")]
            Scene::Sandbox | Scene::Dev { .. } | Scene::Motions { .. } => None,
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
            | Scene::Bag { .. }
            | Scene::BagAct
            | Scene::EquipPick { .. } => {}
            #[cfg(feature = "dev")]
            Scene::Sandbox | Scene::Dev { .. } | Scene::Motions { .. } => {}
        }
    }

    fn draw(&mut self) {
        self.clock += get_frame_time();
        // 그리기 전에 정할 것(0278 R22): 기술 탭의 판과 줄의 열림 — 그리기는 읽기만.
        self.ready_skill_rows();
        // 멈춤 창 아래의 세계는 틱이 없다 — 틱 사이를 오가지 않게 끝에 세운다(0173).
        let alpha = if self.paused() {
            1.0
        } else {
            (self.acc / TICK_DT).clamp(0.0, 1.0)
        };
        self.cam.present(alpha);
        let stage = self.in_stage().then_some(&self.run);
        // 지역 사이 전환의 굴 장면(0174): 월드 대신 굴을 그린다 — 지역은 이미 닿을 곳으로 바뀌어 있고, 그 월드는 닿는 쪽의
        // 어둠 뒤에서 처음 그려진다(지형 굽기가 거기 숨는다). HUD·화살·터치 버튼도 그동안은 걷는다.
        let in_shaft = self
            .passage()
            .filter(|p| self.scene() == Scene::Play && p.phase() == crate::descent::Phase::Shaft);
        let f = match in_shaft {
            Some(p) => {
                // 굴의 위·아래 지역(얕은 쪽·깊은 쪽)의 환경: 달이 있나 · 닿을 곳의 안개 빛.
                let (a, b) = if p.up {
                    (p.to, p.to + 1)
                } else {
                    (p.to.saturating_sub(1), p.to)
                };
                let env = |i: usize| {
                    crate::stage::STAGES
                        .get(i)
                        .map_or(&crate::backdrop::SURFACE, |d| d.env)
                };
                self.renderer
                    .draw_shaft(&p, self.weapon, &self.t, self.clock, (env(a), env(b)))
            }
            None => self.renderer.draw(&View {
                world: &self.world,
                cam: &self.cam,
                alpha,
                t: &self.t,
                stage,
                tells: self.opts.tells as f32 / 10.0,
                tell_timing: self.opts.tell_timing,
                fx: &self.fx,
                pose: match (self.fall(), self.wake()) {
                    (Some(f), _) if self.scene() == Scene::Play => Some(f.pose()),
                    (_, Some(w)) if stage.is_some() => Some(("wake", w.pose())),
                    _ => self.gesture.filter(|_| stage.is_some()),
                },
                aim_show: self.aim_show(),
            }),
        };
        self.frame = Some(f);
        let _p = crate::prof::span("ui·hud");
        self.ui.begin(f.ui);
        let overlay = self.input.overlay().filter(|_| in_shaft.is_none());
        // 눌린 꼴(0207): 키·패드 확인은 고른 줄·단추가 그리고(`Ui::press`), 탭은 누른 자리에 덧그린다.
        let pressing = self
            .menu
            .press
            .map(|p| (p.left / PRESS_HOLD).clamp(0.0, 1.0).sqrt());
        self.ui.press = match self.menu.press {
            Some(p) if p.rect.is_none() => pressing.unwrap_or(0.0),
            _ => 0.0,
        };
        self.ui.pressed_drawn.set(false);
        let cursor_row = self
            .menu
            .hits
            .iter()
            .find(|(_, h)| *h == Hit::Row(self.menu.cursor))
            .map(|(r, _)| *r);
        self.menu.hits.clear();
        if in_shaft.is_none() {
            self.draw_layered(&f, self.scene());
        }
        if let (Some(p), Some(k)) = (self.menu.press, pressing) {
            // 줄 표가 아닌 화면(칸 격자 등)에서 키로 눌렀으면 고른 줄의 자리에, 탭이면 누른 자리에.
            let at = p
                .rect
                .or(cursor_row.filter(|_| !self.ui.pressed_drawn.get()));
            if let Some(r) = at {
                self.ui.press = k;
                self.ui.press_overlay(r);
            }
        }
        self.ui.press = 0.0;
        // 큰 창은 화면을 다 써서 알림을 제 오른쪽 칸의 상태 줄에 보인다(`draw_pane_head`) — 여기서는 싸움·타이틀만.
        if matches!(
            self.scene(),
            Scene::Play | Scene::Ending { .. } | Scene::Title
        ) {
            self.draw_toast(&f);
        }
        // 깨어나는 첫 장면(0171) 동안은 싸움 화면의 덧그림(화살·조준점·터치 버튼)을 걷는다 — 덮개만.
        let waking = self.waking().is_some()
            || self.arriving().is_some()
            || (self.fall().is_some() && self.scene() == Scene::Play);
        if self.scene() == Scene::Play && !waking && in_shaft.is_none() {
            self.draw_goal_arrow(&f);
            self.draw_knell(&f);
            self.draw_bubble(&f);
            self.draw_stone_card(&f);
        }
        // 지역 사이 전환(0174)의 화면 패스: 덮는 어둠(떠남·굴의 두 끝·닿음) + 굴을 지나는 동안의 깊이 글.
        if let Some(p) = self.passage().filter(|_| self.scene() == Scene::Play) {
            let dark = p.dark();
            if dark > 0.0 {
                draw_rectangle(f.ox, f.oy, f.w, f.h, Color::new(0.02, 0.015, 0.035, dark));
            }
            if in_shaft.is_some() {
                let m = p.depth().round() as u32;
                let label = if m == 0 {
                    s().depth_surface.to_string()
                } else {
                    fill(s().depth_under, &[&m.to_string()])
                };
                hud::draw_depth(&self.ui, &f, &label, 1.0 - dark);
            }
        }
        if self.scene().ticks_world() && in_shaft.is_none() {
            self.draw_beat_note(&f);
        }
        if self.scene().ticks_world() && !waking && in_shaft.is_none() {
            if self.mouse_hidden
                && self.world.lock_target().is_none()
                && self.cam.mode != Mode::Free
            {
                if let Some(c) = self.input.cursor() {
                    hud::draw_reticle(&f, c, input::mouse_aim().size);
                }
            }
            // 숄더뷰 크로스헤어(0222): 화면 가운데 — 원거리는 여기로 간다(가운데의 적이 있으면 호박빛).
            if self.cam.mode == Mode::Free && self.world.lock_target().is_none() {
                let at = vec2(f.ox + f.w * 0.5, f.oy + f.h * 0.5);
                hud::draw_crosshair(&f, at, self.crosshair_target().is_some());
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
                        cast: touch::rb_icon(&self.world),
                        swap: pl.sub.map_or(touch::Kind::Swap, touch::attack_icon),
                        skills: [skill(0), skill(1)],
                        vacant: [0, 1].map(|i| pl.skill.rank[i] == 0),
                        unfit: [0, 1].map(|i| pl.skill.rank[i] == 0 && pl.skill.known[i] > 0),
                        active: [0, 1].map(|i| self.world.skill_active(i)),
                        ult: touch::ult_icon(pl.skill.ult),
                        swap_strike: pl.sub.is_some() && self.world.swap_strikes(),
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
                // 깨어나는 첫 장면(0171): HUD 대신 눈꺼풀·띠·서술 글. 싸움 화면에서만(타이틀 바탕은 그냥 엎드린 방랑자).
                if let Some(fl) = self.fall().filter(|_| scene == Scene::Play) {
                    hud::draw_fall(&self.ui, f, &fl, crate::i18n::s().defeat_sub);
                } else if let Some(ar) = self.arriving() {
                    let st = crate::i18n::s();
                    let skip = input::prompt(input::Act::Pause);
                    hud::draw_arrive(&self.ui, f, &ar, st.stage1_name, (&skip, st.wake_skip));
                } else if let Some(wk) = self.waking() {
                    let st = crate::i18n::s();
                    let skip = input::prompt(input::Act::Pause);
                    hud::draw_wake(
                        &self.ui,
                        f,
                        &wk,
                        [st.wake_1, st.wake_2],
                        (&skip, st.wake_skip),
                    );
                } else {
                    // 튜토리얼의 힌트 판(0167·0171) — 싸움 화면에서만, 창 밑에서는 그리지 않는다.
                    let hint = self.tutor_hint().filter(|_| scene == Scene::Play);
                    let hint = hint.as_ref().map(|h| (h, self.this_run.tutor.fresh));
                    let o = hud::Opts {
                        under_menu: false,
                        edge,
                        skills,
                        hint,
                    };
                    hud::draw(&self.ui, f, &self.world, run, &o);
                    // 마지막 핵의 잔상(0216 W3): HUD 위 화면 가장자리에.
                    if let Some(t) = self.echo().filter(|_| scene == Scene::Play) {
                        hud::draw_echo(f, crate::core_echo::glow(t));
                    }
                }
            }
            Layer::Window { hud: true } if self.stack.base() == Scene::Play => {
                let skills = [0, 1].map(|i| self.world.skill_ready(i, &self.t));
                let o = hud::Opts {
                    under_menu: true,
                    edge: 0.0,
                    skills,
                    hint: None,
                };
                hud::draw(&self.ui, f, &self.world, Some(&self.run), &o);
            }
            Layer::Screen | Layer::Window { .. } | Layer::Modal => {}
        }
        match scene {
            Scene::Title => self.draw_title(f),
            Scene::Ask(k) => self.draw_ask(f, k),
            Scene::Controls => self.draw_controls(f),
            Scene::Status { host } => self.draw_status(f, host),
            Scene::Equip { host } => self.draw_equip(f, host, self.menu.cursor, None),
            Scene::Skills { host } => self.draw_skills(f, self.menu.cursor, host),
            Scene::SkillAct => self.draw_skill_act(f),
            Scene::Codex { host, tab } => self.draw_codex(f, host, tab as usize),
            Scene::Bag { host } => self.draw_bag(f, host, self.menu.cursor),
            Scene::BagAct => self.draw_bag_act(f),
            Scene::Shop => self.draw_shop(f),
            Scene::Settings { host } => self.draw_settings(f, host),
            #[cfg(feature = "dev")]
            Scene::Dev { tab } => self.draw_dev_screen(f, tab as usize),
            #[cfg(feature = "dev")]
            s @ (Scene::Lab | Scene::LabGroup { .. }) => self.draw_lab(f, s),
            #[cfg(feature = "dev")]
            Scene::Motions {
                weapon,
                speed,
                spin,
                t_ms,
            } => self.draw_motions(f, weapon, speed, spin, t_ms),
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
        if let Some((text, t)) = &self.notice.toast {
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

impl App {
    /// 벼린 단이 붙은 무기 이름(0180 — "검 +2", 0단이면 이름만).
    pub(super) fn forged_name(&self, w: weapons::Id) -> String {
        forged_label(w, self.progress.forged.rank(w))
    }
}

/// 넘쳐 굴리는 가로 탭 줄(`ui::Strip`) 위의 끌기·휠은 그 줄을 굴린다 — 몸의 목록에는 넘기지 않는다(0257 작은 탭 줄 · 0275 판 줄).
pub(super) fn strip_input(strip: &mut crate::ui::Strip, a: &Actions) -> Actions {
    let mut a = *a;
    if strip.over(a.point) {
        strip.nudge(a.point, a.drag_x, a.wheel);
        (a.drag, a.drag_x, a.wheel) = (0.0, 0.0, 0);
    }
    a
}

/// 무기 이름 + 단(0180).
fn forged_label(w: weapons::Id, rank: u8) -> String {
    match rank {
        0 => weapon_name(w).to_string(),
        r => fill(s().forge_named, &[weapon_name(w), &r.to_string()]),
    }
}

/// 무기 이름(0051 — 무기 표의 줄 `Weapon::name`).
fn weapon_name(id: weapons::Id) -> &'static str {
    tr(id.def().name)
}

#[cfg(test)]
mod press_tests {
    use super::*;

    /// 2026-11-19: 작은 창 밖을 눌러 닫을 때 그 바깥(아래 화면의 칸·창 밖 전체)은 눌린 꼴을 보이지 않는다 — 작은 창의 줄만.
    /// 창 밖을 덮는 닫기 칸(화면보다 큰 칸)은 큰 창에서도 단추가 아니다. ✕(작은 칸)는 그대로 눌린다.
    #[test]
    fn closing_a_popup_by_tapping_outside_lights_nothing_underneath() {
        let small = Rect::new(10.0, 10.0, 40.0, 20.0);
        let outside = Rect::new(-1.0e5, -1.0e5, 2.0e5, 1.0e5 + 10.0);
        assert!(press_shows(true, Hit::Row(1), small), "the popup's own row");
        for h in [Hit::Cell(0, 2), Hit::Tab(1), Hit::Close] {
            assert!(!press_shows(true, h, small), "{h:?} under a popup");
        }
        assert!(!press_shows(true, Hit::Close, outside));
        assert!(
            !press_shows(false, Hit::Close, outside),
            "the outside of a big window"
        );
        assert!(press_shows(false, Hit::Close, small), "the ✕ button");
        assert!(press_shows(false, Hit::Cell(0, 2), small));
        assert!(Scene::EquipPick { slot: 0 }.is_popup() && Scene::BagAct.is_popup());
        assert!(!Scene::Title.is_popup());
    }

    /// 0207: 누름을 붙잡은 동안 줄 세우는 것은 누르는 순간의 입력뿐 — 쥐고 있는 스틱은 매 틱 같은 값이라 쌓이면 지연이 남는다.
    #[test]
    fn only_edges_queue_while_a_press_is_held() {
        let held = Actions {
            move_dir: Vec2::new(0.0, 1.0),
            attack_held: true,
            ..Actions::default()
        };
        assert!(!is_edge(&held));
        assert!(!is_edge(&Actions::default()));
        for a in [
            Actions {
                menu_down: true,
                ..Actions::default()
            },
            Actions {
                confirm: true,
                ..Actions::default()
            },
            Actions {
                back: true,
                ..Actions::default()
            },
            Actions {
                tap: Some(Vec2::ZERO),
                ..Actions::default()
            },
        ] {
            assert!(is_edge(&a), "{a:?}");
        }
        // 눌린 꼴은 UI 누름 피드백(100~160 ms)보다 길지 않게 — 늦게 넘어간다고 느끼지 않게.
        const { assert!(PRESS_HOLD > 0.05 && PRESS_HOLD <= 0.12) };
    }
}

#[cfg(test)]
mod aim_show_tests {
    use super::*;

    /// 0227: 쇠뇌로 평타(사격)를 끌면 사격 띠, 특수(수류탄)를 끌면 터지는 원 — 근접 무기의 특수는 마법 탄 띠.
    #[test]
    fn a_dragged_touch_button_previews_what_it_now_does() {
        assert_eq!(
            touch_aim_show(touch::Kind::Attack, true),
            Some(AimShow::Cast)
        );
        assert_eq!(
            touch_aim_show(touch::Kind::Cast, true),
            Some(AimShow::Grenade)
        );
        assert_eq!(
            touch_aim_show(touch::Kind::Cast, false),
            Some(AimShow::Cast)
        );
        assert_eq!(touch_aim_show(touch::Kind::Attack, false), None);
        assert_eq!(
            touch_aim_show(touch::Kind::Skill2, false),
            Some(AimShow::Skill(1))
        );
    }
}
