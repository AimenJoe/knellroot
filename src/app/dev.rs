//! 개발 창(0083, nail 0269 `Scene::Dev` 구조 그대로): 위에 묶음 탭이 한 줄로 다 보이고(좌우·Q/E·탭), 왼쪽은 그 묶음의 줄(위아래·굴리기),
//! 오른쪽은 고른 줄의 이름·설명, 아래 안내 한 줄. 줄은 시작할 것만(돌아가기 줄·무기 줄 없음 — 뒤로 = 나가기, 무기는 장비 창).
//! 줄마다 묶음·시작할 것을 표 한 곳(`dev_list` — 손으로 적는 `FIXED_ROWS` + 무기마다의 줄)에 두고 묶음별 목록은 거기서 거른다(nail `scenario::Entry { cat }`).
//! 개발판에만. 여기서 시작한 스테이지 판은 기록 슬롯에 쓰지 않는다(`App::dev_run`, 0075).
use super::menu::Hit;
use super::*;
use crate::i18n::dev::{self as dv, scenario_text, t};

/// 묶음(탭 순서 = 이 순서, 도구가 앞 — nail 0269). 2026-11-15 사장 "체계적으로 정리된 게 아닌 듯": 하는 일로 가른다 —
/// 도구(연구소·놀이터) · 모션(무기별 동작 보기) · 기술(끼우기·판 바꾸기·기술 아레나) · 스테이지 · 전투(적·보스) · 성능(성능 줄·부하).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum DevCat {
    Tools,
    Motion,
    Skills,
    Stage,
    Combat,
    Perf,
}

pub(super) const DEV_CATS: [DevCat; 6] = [
    DevCat::Tools,
    DevCat::Motion,
    DevCat::Skills,
    DevCat::Stage,
    DevCat::Combat,
    DevCat::Perf,
];

impl DevCat {
    fn name(self) -> &'static str {
        t(match self {
            DevCat::Tools => dv::TAB_TOOLS,
            DevCat::Motion => dv::TAB_MOTION,
            DevCat::Skills => dv::TAB_SKILLS,
            DevCat::Stage => dv::TAB_STAGE,
            DevCat::Combat => dv::TAB_COMBAT,
            DevCat::Perf => dv::TAB_PERF,
        })
    }
}

/// 개발 창에서 시작할 것 하나.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum DevStart {
    /// 첫 스테이지 처음부터(입구) — 기록은 쓰지 않는다.
    Entrance,
    /// 문 앞(문지기 결투) — 기록은 쓰지 않는다.
    Gate,
    /// 둘째 지역 어귀(0118 — 이 판에 첫 문지기를 이긴 셈) — 기록은 쓰지 않는다.
    Village,
    /// 셋째 지역 어귀(0185 — 이 판에 두 문지기를 이긴 셈) — 기록은 쓰지 않는다.
    Garden,
    /// 넷째 지역 어귀(0208 — 이 판에 세 문지기를 이긴 셈) — 기록은 쓰지 않는다.
    Works,
    /// 서막(0262): 숲에서 깨어나는 장면부터 — 숲 끝에서 "성소에 들다". 기록은 쓰지 않는다.
    Prologue,
    /// 둘째 지역의 첫 주민 곁(0122 — 말풍선·돌아봄 판정) — 기록은 쓰지 않는다.
    Residents,
    /// 둘째 지역의 새긴 돌 곁(0129 — 글 판 판정) — 기록은 쓰지 않는다.
    Stones,
    /// 둘째 지역의 첫 상자 곁(0131 — 떨어진 아이템·빛기둥 판정) — 기록은 쓰지 않는다.
    Chest,
    /// 그 무기가 나오는 상자 곁(0272 2단계 · 0274 ③~⑦ — 나오는 곳 판정): 지역 · 상자는 스테이지 표의 `chest_loot`에서 찾는다
    /// (`DevStart::chest`), 줄은 상자에 든 무기마다 하나(`dev_list`). 기록은 쓰지 않는다.
    WeaponChest(crate::weapons::Id),
    /// 첫 지역 성벽길의 상자 곁(0274 ⑦ — 옛 쇠뇌 상자, 이제 숫돌) — 기록은 쓰지 않는다.
    WallChest,
    /// 첫 지역 뜰 남동 단의 상자 곁(0274 ⑦ — 옛 지팡이 상자, 이제 여벌) — 기록은 쓰지 않는다.
    SpareChest,
    /// 첫 지역의 필드 보스 터 들머리(0270 D — 엎드린 고요한 사슴·끈·떨굼·쉬어도 다시 서지 않음 판정) — 기록은 쓰지 않는다.
    Field,
    /// PoC·개발 아레나(`scenario::LIST`).
    Scenario(Scenario),
    /// 연구소(메뉴의 설정 아래 줄과 같은 창, 0079).
    Lab,
    /// 기술 화면(0151 → 0161 — 멈춤의 기술 줄과 같은 화면, 읽기·끼우기): 아레나에는 멈춤이 없어 여기서 끼운다.
    Skills,
    /// 기술 판을 모두 끝 단으로·패시브 칸을 모두 열림으로(0158·0161 — 켬/끔): 개발 판·아레나에서 단과 고르기를 시험한다. 기록에 쓰지 않는다.
    SkillsMax,
    /// 끝 단으로 올린 판의 3단 고르기를 모두 뒤집는다(ㄱ ↔ ㄴ). 기록에 쓰지 않는다.
    SkillsFlip,
    /// 든 무기 바꾸기(0223 V3 — 스킬은 무기 갈래를 탄다: 아레나에는 장비 탭이 없어 여기서 든다). 기록에 쓰지 않는다.
    Arm,
    /// 보조무기 바꾸기(0272 1b — 보조특성·교체 공격을 아레나에서 본다). 기록에 쓰지 않는다.
    ArmSub,
    /// 보조무기 비우기(0276 — 켬/끔: 아레나의 보조무기 슬롯을 비워 빈 손의 결을 본다). 기록에 쓰지 않는다.
    ArmBare,
    /// 성능 줄(F3과 같은 개발 줄) 켬/끔 — 키보드가 없는 기기(Android·iOS)에서 프레임 시간을 본다(0186 상주 지형 A/B).
    PerfLine,
    /// 모션 보기(2026-11-15): 이 무기(`weapons::IDS`의 자리)를 든 방랑자의 동작을 하나씩.
    Motion(u8),
}

/// 손으로 적는 줄(묶음, 시작할 것) — 묶음 안 순서 = 표 순서. 무기마다의 줄(모션 보기 · 무기가 나오는 상자)은 표에서 선다(`dev_list`). 시험 스크립트는 줄을 세지 않고 이름으로 부른다(0278 R13 — `DevCmd`,
/// `tools/test/scene.js` `dev`·`devRow`): 줄을 어디에 끼워도 스크립트의 길은 그대로다(아래 "끝에 둔다"는 그 전의 까닭).
const FIXED_ROWS: [(DevCat, DevStart); 41] = [
    (DevCat::Tools, DevStart::Lab),
    (DevCat::Tools, DevStart::Scenario(Scenario::Empty)),
    (DevCat::Skills, DevStart::Skills),
    (DevCat::Skills, DevStart::SkillsMax),
    (DevCat::Skills, DevStart::SkillsFlip),
    (DevCat::Skills, DevStart::Scenario(Scenario::Skills)),
    (DevCat::Skills, DevStart::Scenario(Scenario::Tray)),
    // 끝에 둔다 — 시험 스크립트가 세는 앞 줄들의 자리를 밀지 않는다.
    (DevCat::Skills, DevStart::Arm),
    // 0272 1b: 역시 끝에.
    (DevCat::Skills, DevStart::ArmSub),
    // 0276: 역시 끝에.
    (DevCat::Skills, DevStart::ArmBare),
    (DevCat::Stage, DevStart::Entrance),
    (DevCat::Stage, DevStart::Gate),
    (DevCat::Stage, DevStart::Village),
    (DevCat::Stage, DevStart::Residents),
    (DevCat::Stage, DevStart::Stones),
    (DevCat::Stage, DevStart::Chest),
    (DevCat::Stage, DevStart::Garden),
    (DevCat::Stage, DevStart::Works),
    // 0262: 묶음의 끝에 둔다(시험 스크립트가 세는 앞 줄을 밀지 않는다).
    (DevCat::Stage, DevStart::Prologue),
    // 0270 D: 역시 끝에.
    (DevCat::Stage, DevStart::Field),
    // 0274 ⑦: 역시 끝에.
    (DevCat::Stage, DevStart::WallChest),
    (DevCat::Stage, DevStart::SpareChest),
    (DevCat::Combat, DevStart::Scenario(Scenario::Boss)),
    (DevCat::Combat, DevStart::Scenario(Scenario::Pillars)),
    (DevCat::Combat, DevStart::Scenario(Scenario::Duel)),
    (DevCat::Combat, DevStart::Scenario(Scenario::Elites)),
    (DevCat::Combat, DevStart::Scenario(Scenario::Newcomers)),
    (DevCat::Combat, DevStart::Scenario(Scenario::Bellkeeper)),
    (DevCat::Combat, DevStart::Scenario(Scenario::WardenTwo)),
    (DevCat::Combat, DevStart::Scenario(Scenario::BellkeeperTwo)),
    (DevCat::Combat, DevStart::Scenario(Scenario::Mossward)),
    (DevCat::Combat, DevStart::Scenario(Scenario::MosswardTwo)),
    (DevCat::Combat, DevStart::Scenario(Scenario::Horde)),
    (DevCat::Combat, DevStart::Scenario(Scenario::Workers)),
    // 0234: 끝에 둔다(시험 스크립트가 세는 앞 줄을 밀지 않는다).
    (DevCat::Combat, DevStart::Scenario(Scenario::Overseer)),
    // 0270 D: 필드 보스 — 역시 끝에.
    (DevCat::Combat, DevStart::Scenario(Scenario::Stag)),
    (DevCat::Combat, DevStart::Scenario(Scenario::StagTwo)),
    // 0272 2단계: 창의 한 줄 — 역시 끝에.
    (DevCat::Combat, DevStart::Scenario(Scenario::Line)),
    (DevCat::Perf, DevStart::PerfLine),
    (DevCat::Perf, DevStart::Scenario(Scenario::Stress40)),
    (DevCat::Perf, DevStart::Scenario(Scenario::Stress80)),
];

/// 개발 창의 줄 전부: 손으로 적는 줄 + 무기마다의 줄(0278 R19 — 무기 표 `weapons::IDS`의 차례): 모션 보기는 무기마다 하나, 무기가
/// 나오는 상자는 스테이지 표의 상자에 든 무기마다 하나(스테이지 묶음 끝). 새 무기는 여기에 손대지 않는다.
pub(super) fn dev_list() -> Vec<(DevCat, DevStart)> {
    let motions =
        (0..crate::weapons::IDS.len() as u8).map(|w| (DevCat::Motion, DevStart::Motion(w)));
    let chests = crate::weapons::IDS
        .into_iter()
        .filter(|&w| chest_region(w).is_some())
        .map(|w| (DevCat::Stage, DevStart::WeaponChest(w)));
    FIXED_ROWS
        .into_iter()
        .chain(motions)
        .chain(chests)
        .collect()
}

/// 무기 `w`가 든 상자의 지역(`stage::STAGES`의 앞 `REGIONS`개 — 서막 숲은 상자가 없다), 없으면 없음.
fn chest_region(w: crate::weapons::Id) -> Option<usize> {
    crate::stage::STAGES[..crate::stage::REGIONS]
        .iter()
        .position(|d| d.chest_loot.contains(&crate::stage::Loot::Weapon(w)))
}

/// 묶음 `tab`의 줄.
pub(super) fn dev_items(tab: usize) -> Vec<DevStart> {
    let cat = DEV_CATS[tab.min(DEV_CATS.len() - 1)];
    dev_list()
        .into_iter()
        .filter(|(c, _)| *c == cat)
        .map(|(_, e)| e)
        .collect()
}

/// "든 무기 바꾸기"의 차례(0223 V3 → 0278 R19 — 무기 표 `weapons::IDS`에서 선다): 무기마다 그것만, 한손 종류는 바로 뒤에 순례자의
/// 방패와 함께(방패 트리의 스킬을 끼워 보게, 0270 C3). 시험 스크립트는 세지 않고 이름(`Arm(Wand)` · `Arm(Sword+Shield)`, `DevCmd`)으로
/// 든다(0278 R13) — 새 무기는 여기에 손대지 않는다.
fn arm_cycle() -> Vec<(crate::weapons::Id, bool)> {
    crate::weapons::IDS
        .into_iter()
        .flat_map(|w| {
            let shield = w.class().is_some() && w.hand() == crate::weapons::Hand::One;
            std::iter::once((w, false)).chain(shield.then_some((w, true)))
        })
        .collect()
}

/// 차례의 이름 줄(개발 창 설명 — `ARM_NOTE`의 `{1}` · `ARM_SUB_NOTE`의 `{1}`): 무기 이름을 화살표로.
fn arm_names(arms: impl Iterator<Item = Option<(crate::weapons::Id, bool)>>) -> String {
    arms.map(|a| match a {
        None => s().slot_empty.to_string(),
        Some((w, false)) => weapon_name(w).to_string(),
        Some((w, true)) => format!("{} + {}", weapon_name(w), s().menu_shield),
    })
    .collect::<Vec<_>>()
    .join(" → ")
}

/// "보조무기 바꾸기"가 도는 무기(종류가 있는 것 — 맨손은 메지 않는다, 표 `weapons::IDS`의 차례).
fn sub_ids() -> Vec<crate::weapons::Id> {
    crate::weapons::IDS
        .into_iter()
        .filter(|w| w.class().is_some())
        .collect()
}

/// 이름 있는 개발 명령(0278 R13 — 헤드리스 시험 · 재기가 개발 창을 세어 누르지 않고 이름으로 부른다): 웹은 페이지의
/// `window.knellDev`(이름의 배열 — 프레임마다 앞에서부터 꺼내 차례로 한다, `screen::dev_command`), PC는 `KNELL_BOOT`(쉼표로 여럿).
/// 이름은 모두 있는 표의 Debug에서 선다 — 따로 적은 목록이 없다: 개발 창 줄 `DevStart`(`Garden` · `Scenario(Stress80)` — 그 줄을
/// 고른 것과 같다), `Dev(<줄>)`(개발 창을 그 줄에 커서를 두고 연다 — 시작하지 않는다), `Arm(<무기>)` · `Arm(<무기>+Shield)`(`arm_cycle`의
/// 한 칸을 바로 든다), `ArmSub(<무기>)` · `ArmSub(None)`(`sub_ids`). 모르는 이름은 콘솔 에러 `knellDev: …`(스크립트가 실패로 본다).
#[cfg(any(feature = "dev", test))]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum DevCmd {
    Start(DevStart),
    Show(DevStart),
    Arm(crate::weapons::Id, bool),
    Sub(Option<crate::weapons::Id>),
}

#[cfg(any(feature = "dev", test))]
impl DevCmd {
    /// 받는 명령 전부(이름의 출처 = 표).
    fn all() -> Vec<DevCmd> {
        let rows = dev_list().into_iter().map(|(_, e)| e);
        rows.clone()
            .map(DevCmd::Start)
            .chain(rows.map(DevCmd::Show))
            .chain(arm_cycle().into_iter().map(|(w, s)| DevCmd::Arm(w, s)))
            .chain(std::iter::once(DevCmd::Sub(None)))
            .chain(sub_ids().into_iter().map(|w| DevCmd::Sub(Some(w))))
            .collect()
    }

    fn name(self) -> String {
        match self {
            DevCmd::Start(e) => format!("{e:?}"),
            DevCmd::Show(e) => format!("Dev({e:?})"),
            DevCmd::Arm(w, false) => format!("Arm({w:?})"),
            DevCmd::Arm(w, true) => format!("Arm({w:?}+Shield)"),
            DevCmd::Sub(None) => "ArmSub(None)".to_string(),
            DevCmd::Sub(Some(w)) => format!("ArmSub({w:?})"),
        }
    }

    fn parse(s: &str) -> Option<DevCmd> {
        Self::all().into_iter().find(|c| c.name() == s.trim())
    }
}

impl DevStart {
    /// 상자 곁 줄이면 (지역, 그 상자의 것 — `None` = 그 지역의 첫 상자)(와일드카드 없이 — 0278 R14). 무기가 나오는 상자는 그 무기를
    /// 든 지역(`chest_region` — 스테이지 표), 성소의 옛 무기 상자 둘(0274 ⑦)은 첫 지역(건너지 않는다), 첫 상자는 둘째 지역.
    fn chest(self) -> Option<(usize, Option<crate::stage::Loot>)> {
        use crate::stage::Loot;
        Some(match self {
            DevStart::Chest => (1, None),
            DevStart::WeaponChest(w) => (chest_region(w)?, Some(Loot::Weapon(w))),
            DevStart::WallChest => (0, Some(Loot::Stone)),
            DevStart::SpareChest => (0, Some(Loot::Spare)),
            DevStart::Entrance
            | DevStart::Gate
            | DevStart::Village
            | DevStart::Garden
            | DevStart::Works
            | DevStart::Prologue
            | DevStart::Residents
            | DevStart::Stones
            | DevStart::Field
            | DevStart::Scenario(_)
            | DevStart::Lab
            | DevStart::Skills
            | DevStart::SkillsMax
            | DevStart::SkillsFlip
            | DevStart::Arm
            | DevStart::ArmSub
            | DevStart::ArmBare
            | DevStart::PerfLine
            | DevStart::Motion(_) => return None,
        })
    }

    fn name(self) -> &'static str {
        match self {
            DevStart::Entrance => t(dv::ENTRANCE),
            DevStart::Gate => t(dv::GATE),
            DevStart::Village => t(dv::VILLAGE),
            DevStart::Garden => t(dv::GARDEN),
            DevStart::Works => t(dv::WORKS),
            DevStart::Prologue => t(dv::PROLOGUE),
            DevStart::Residents => t(dv::RESIDENTS),
            DevStart::Stones => t(dv::STONES),
            DevStart::Chest => t(dv::CHEST),
            // 나오는 곳 · 무기 이름은 `App::dev_name`이 채운다(무기 표의 줄 `Weapon::place` · `name`) — 여기는 바탕.
            DevStart::WeaponChest(_) => t(dv::WEAPON_CHEST),
            DevStart::WallChest => t(dv::WALL_CHEST),
            DevStart::SpareChest => t(dv::SPARE_CHEST),
            DevStart::Field => t(dv::FIELD),
            DevStart::Scenario(sc) => {
                let e = scenario::entry(sc);
                scenario_text(e.name, e.checks).0
            }
            DevStart::Lab => t(dv::LAB_START),
            DevStart::Skills => t(dv::SKILLS_START),
            DevStart::SkillsMax => t(dv::SKILLS_MAX),
            DevStart::SkillsFlip => t(dv::SKILLS_FLIP),
            DevStart::Arm => t(dv::ARM),
            DevStart::ArmSub => t(dv::ARM_SUB),
            DevStart::ArmBare => t(dv::ARM_BARE),
            DevStart::PerfLine => t(dv::PERF_LINE),
            // 이름은 무기를 채워야 해서 `App::dev_name`이 만든다 — 여기는 바탕.
            DevStart::Motion(_) => t(dv::MOTION_START),
        }
    }

    fn note(self) -> String {
        match self {
            DevStart::Entrance
            | DevStart::Gate
            | DevStart::Village
            | DevStart::Garden
            | DevStart::Works
            | DevStart::Prologue
            | DevStart::Residents
            | DevStart::Stones
            | DevStart::Chest
            | DevStart::WeaponChest(_)
            | DevStart::WallChest
            | DevStart::SpareChest
            | DevStart::Field => t(dv::STAGE_NOTE).to_string(),
            DevStart::Scenario(sc) => {
                let n = scenario::LIST
                    .iter()
                    .position(|e| e.scenario == sc)
                    .unwrap_or(0)
                    + 1;
                let e = scenario::entry(sc);
                let checks = scenario_text(e.name, e.checks).1;
                fill(t(dv::SCENARIO_NOTE), &[&n.to_string(), checks])
            }
            DevStart::Lab => t(dv::LAB_NOTE).to_string(),
            DevStart::Skills => t(dv::SKILLS_NOTE).to_string(),
            // 지금 상태({0})는 앱이 채운다(`App::dev_note`).
            DevStart::SkillsMax => t(dv::SKILLS_MAX_NOTE).to_string(),
            DevStart::SkillsFlip => t(dv::SKILLS_FLIP_NOTE).to_string(),
            // 지금 든 무기({0})는 앱이 채운다.
            DevStart::Arm => t(dv::ARM_NOTE).to_string(),
            DevStart::ArmSub => t(dv::ARM_SUB_NOTE).to_string(),
            DevStart::ArmBare => t(dv::ARM_BARE_NOTE).to_string(),
            DevStart::PerfLine => t(dv::PERF_LINE_NOTE).to_string(),
            DevStart::Motion(_) => t(dv::MOTION_NOTE).to_string(),
        }
    }
}

/// 개발 창의 기술 판 줄을 고르면(0158): "다 올리기"는 켬/끔(켜면 끝 단 · 고르기 ㄱ), "고르기 바꾸기"는 끝 단 판의 고르기를
/// 모두 뒤집는다(꺼져 있으면 ㄴ으로 켠다). 순수 — 시험.
pub(super) fn dev_board_step(
    now: Option<crate::skills::Board>,
    e: DevStart,
) -> Option<crate::skills::Board> {
    use crate::skills::{Board, Pick};
    match (e, now) {
        (DevStart::SkillsMax, None) => Some(Board::maxed(Pick::A)),
        (DevStart::SkillsMax, Some(_)) => None,
        (DevStart::SkillsFlip, None) => Some(Board::maxed(Pick::B)),
        (DevStart::SkillsFlip, Some(b)) => Some(Board::maxed(b.pick[0].other())),
        (_, now) => now,
    }
}

impl App {
    /// 개발 창 한 틱: 뒤로·✕·창 밖 = 나가기, 좌우·스킬 버튼(Q/E, 0151)·탭 누르기 = 묶음, 위아래·굴리기 = 줄, 확인·줄 누르기 = 시작.
    pub(super) fn tick_dev(&mut self, a: &Actions, tab: usize) {
        let a = &self.small_strip_input(a);
        let tap = a.tap.and_then(|p| self.hit_at(p));
        if a.back || a.pause || tap == Some(Hit::Close) {
            self.go(Go::Back);
            return;
        }
        let n = DEV_CATS.len();
        let step = (a.menu_right || a.tab_next) as i32 - (a.menu_left || a.tab_prev) as i32;
        let to = match tap {
            Some(Hit::Cell(1, i)) => Some(i),
            _ if step != 0 => Some((tab as i32 + step).rem_euclid(n as i32) as usize),
            _ => None,
        };
        if let Some(to) = to.filter(|&i| i != tab) {
            // 묶음은 장면 칸의 상태(0066): 지금 칸을 새 묶음으로 바꿔 연다(커서는 첫 줄).
            self.go(Go::Back);
            self.go(Go::Open(Scene::Dev { tab: to as u8 }));
            self.snd(crate::sfx::Id::UiMove);
            return;
        }
        let items = dev_items(tab);
        if let Some(i) = self.menu_pick(a, items.len()) {
            if let Some(&e) = items.get(i) {
                self.dev_start(e);
            }
        }
    }

    /// 개발 창에서 시작(0075): 스테이지 판은 기록 슬롯에 쓰지 않는다(`dev_run` — 슬롯을 고르면 풀린다).
    fn dev_start(&mut self, e: DevStart) {
        match e {
            DevStart::Entrance | DevStart::Gate => {
                self.dev.run = true;
                self.restart();
                if e == DevStart::Gate {
                    self.run.dev_skip_to_gate(&mut self.world);
                }
                self.cam = self.cam.renew(self.world.player.pos, &self.t);
                self.go(Go::Root(Scene::Play));
            }
            // 필드 보스의 터(0270 D): 첫 지역을 세우고 터의 들머리(보스에서 서쪽 12m — 일어나는 거리 밖)에 동쪽을 보고 선다.
            DevStart::Field => {
                self.dev.run = true;
                self.restart();
                let home = self.run.level.marks_of(crate::stage::FIELD_GLYPH).next();
                if let Some(at) = home {
                    let pl = &mut self.world.player;
                    pl.pos = at + vec3(-12.0, 0.0, 0.0);
                    pl.prev = pl.pos;
                    pl.facing = std::f32::consts::FRAC_PI_2;
                }
                self.run.intro = 0.0;
                self.cam = self.cam.renew(self.world.player.pos, &self.t);
                self.go(Go::Root(Scene::Play));
            }
            DevStart::Village => {
                self.dev.run = true;
                self.restart();
                // 이 판에 첫 문지기를 이긴 셈 — 내려가는 뿌리 굴(0174)부터 보인다.
                self.down = 1;
                self.interlude = Some(super::interlude::Interlude::Passage(
                    crate::descent::Passage::new(
                        1,
                        false,
                        self.run.def.depth_m,
                        crate::stage::STAGES[1].depth_m,
                    ),
                ));
                self.go(Go::Root(Scene::Play));
            }
            DevStart::Garden => {
                self.dev.run = true;
                self.restart();
                // 이 판에 두 문지기를 이긴 셈 — 마을에서 내려가는 뿌리 굴부터.
                self.down = 2;
                self.cross_to(1, false);
                self.interlude = Some(super::interlude::Interlude::Passage(
                    crate::descent::Passage::new(
                        2,
                        false,
                        crate::stage::STAGES[1].depth_m,
                        crate::stage::STAGES[2].depth_m,
                    ),
                ));
                self.go(Go::Root(Scene::Play));
            }
            DevStart::Works => {
                self.dev.run = true;
                self.restart();
                // 이 판에 세 문지기를 이긴 셈 — 정원에서 내려가는 뿌리 굴부터.
                self.down = 3;
                self.cross_to(2, false);
                self.interlude = Some(super::interlude::Interlude::Passage(
                    crate::descent::Passage::new(
                        3,
                        false,
                        crate::stage::STAGES[2].depth_m,
                        crate::stage::STAGES[3].depth_m,
                    ),
                ));
                self.go(Go::Root(Scene::Play));
            }
            DevStart::Prologue => {
                self.dev.run = true;
                self.restart();
                // 서막 숲의 깨어난 빈터에서 — 깨어나는 장면부터(개발 판이라 기록에는 쓰지 않는다).
                self.enter_stage(crate::stage::PROLOGUE);
                self.interlude = Some(super::interlude::Interlude::Wake(Default::default()));
                lay_by_hearth(&self.run, &mut self.world);
                self.cam = self.cam.renew(self.world.player.pos, &self.t);
                self.go(Go::Root(Scene::Play));
            }
            DevStart::Residents => {
                self.dev.run = true;
                self.restart();
                self.down = 1;
                self.cross_to(1, false);
                if let Some(&(_, at)) = self.run.folk.list.first() {
                    self.world.player.pos = at + vec3(0.0, 0.0, 1.6);
                    self.world.player.prev = self.world.player.pos;
                    self.cam = self.cam.renew(self.world.player.pos, &self.t);
                }
                self.go(Go::Root(Scene::Play));
            }
            DevStart::Stones => {
                self.dev.run = true;
                self.restart();
                self.down = 1;
                self.cross_to(1, false);
                if let Some(&at) = self.run.stones.last() {
                    self.world.player.pos = at + vec3(0.0, 0.0, 1.4);
                    self.world.player.prev = self.world.player.pos;
                    self.cam = self.cam.renew(self.world.player.pos, &self.t);
                }
                self.go(Go::Root(Scene::Play));
            }
            DevStart::Chest
            | DevStart::WeaponChest(_)
            | DevStart::WallChest
            | DevStart::SpareChest => {
                self.dev.run = true;
                self.restart();
                // 지역은 그 상자를 든 곳(`DevStart::chest` — 둘째 지역부터는 그 앞 문지기를 이긴 셈으로 건넌다).
                let Some((region, loot)) = e.chest() else {
                    return;
                };
                if region > 0 {
                    self.down = region;
                    self.cross_to(region, false);
                }
                let which = match loot {
                    Some(l) => self.run.def.chest_loot.iter().position(|x| *x == l),
                    None => Some(0),
                };
                if let Some(&at) = which.and_then(|i| self.run.chest.at.get(i)) {
                    // 서쪽 곁부터 — 벽 · 다른 높이면 동 · 남 · 북(0274 ⑦: 작업대 구석의 상자는 서쪽이 벽이다).
                    let g = self.run.level.ground();
                    let floor = |p: Vec3| {
                        g.cell_of(p).is_some_and(|(x, z)| {
                            !matches!(g.at(x, z), crate::ground::Cell::Solid)
                                && (g.height(p) - g.height(at)).abs() < 0.1
                        })
                    };
                    let side = [
                        vec3(-1.2, 0.0, 0.0),
                        vec3(1.2, 0.0, 0.0),
                        vec3(0.0, 0.0, 1.2),
                        vec3(0.0, 0.0, -1.2),
                    ]
                    .into_iter()
                    .find(|d| floor(at + *d))
                    .unwrap_or(vec3(-1.2, 0.0, 0.0));
                    self.world.player.pos = at + side;
                    self.world.player.prev = self.world.player.pos;
                    self.cam = self.cam.renew(self.world.player.pos, &self.t);
                }
                self.go(Go::Root(Scene::Play));
            }
            DevStart::Scenario(sc) => {
                self.dev.sc = sc;
                self.world = scenario::build(sc);
                self.fx.clear();
                self.arm_sandbox();
                self.cam = self.cam.renew(self.world.player.pos, &self.t);
                self.go(Go::Root(Scene::Sandbox));
                // 놀이터가 선 뒤에: 판(`App::board`)이 놀이터인지를 보고 처음의 둘을 배운 것으로 친다(0166).
                self.apply_board();
            }
            DevStart::Lab => self.go(Go::Open(Scene::Lab)),
            DevStart::Skills => self.show(Go::Open(Scene::Skills {
                host: flow::Host::Dev,
            })),
            // 기술 판(0158): 메모리의 덮어쓰기만 바꾼다 — 기록(`progress`)과 슬롯은 건드리지 않는다. 창은 그대로(설명 줄이 상태를 보인다).
            DevStart::SkillsMax | DevStart::SkillsFlip => {
                self.dev.board = dev_board_step(self.dev.board, e);
                self.apply_board();
                self.snd(crate::sfx::Id::Buy);
            }
            DevStart::PerfLine => {
                self.dev_overlay = !self.dev_overlay;
                self.snd(crate::sfx::Id::Buy);
            }
            // 든 무기를 표의 다음 것으로(0223 V3 — 차례는 `arm_cycle`) — 이 창에서 시작하는 아레나와 "기술 끼우기"가 그 무기로 선다. 기록은 그대로.
            DevStart::Arm => {
                let now = (self.weapon, self.progress.shield.is_some());
                let cycle = arm_cycle();
                let at = cycle.iter().position(|c| *c == now).unwrap_or(0);
                let (w, s) = cycle[(at + 1) % cycle.len()];
                self.dev_arm(w, s);
                self.snd(crate::sfx::Id::Buy);
            }
            // 놀이터의 보조무기를 표의 다음 것으로(없음 → `sub_ids()`의 차례 → 없음 — 맨손은 메지 않는다).
            // 아레나에서 든 무기와 같으면 비워진다(`arm_sandbox`). 기록은 그대로.
            DevStart::ArmSub => {
                let ids = sub_ids();
                self.dev.sub = match self.dev.sub {
                    None => ids.first().copied(),
                    Some(w) => ids
                        .iter()
                        .position(|i| *i == w)
                        .and_then(|at| ids.get(at + 1).copied()),
                };
                self.snd(crate::sfx::Id::Buy);
            }
            // 놀이터의 보조무기 슬롯을 비운다 · 되돌린다(0276). 기록은 그대로.
            DevStart::ArmBare => {
                self.dev.bare = !self.dev.bare;
                self.snd(crate::sfx::Id::Buy);
            }
            DevStart::Motion(w) => self.go(Go::Open(Scene::Motions {
                weapon: w,
                speed: 0,
                spin: 0,
                t_ms: 0,
            })),
        }
    }

    /// 재기용 바로 시작(0258 — PC 개발판): 환경 변수 `KNELL_BOOT`가 개발 명령의 이름(`Garden`·`Scenario(Stress80)` … — 쉼표로
    /// 여럿, `DevCmd`)이면 타이틀을 거치지 않고 차례로 한다. 구간 시간 재기(`prof`)를 같은 장면에서 되풀이하려고. 웹의 짝은
    /// `window.knellDev`(`App::frame`이 프레임마다 꺼낸다, 0278 R13).
    #[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
    pub fn dev_boot(&mut self) {
        // 연구소 값 덮어쓰기(`KNELL_LAB=render.cull=0,render.ssaa=0`) — 이름은 연구소 줄의 머리(빈칸 앞)로 찾는다.
        for kv in std::env::var("KNELL_LAB").unwrap_or_default().split(',') {
            let Some((name, v)) = kv.split_once('=') else {
                continue;
            };
            let at = crate::tuning::SPECS
                .iter()
                .position(|sp| sp.name.split(' ').next() == Some(name));
            match (at, v.parse::<f32>()) {
                (Some(i), Ok(v)) => self.t.set(i, v),
                _ => eprintln!("KNELL_LAB: 모르는 값 {kv}"),
            }
        }
        let Ok(want) = std::env::var("KNELL_BOOT") else {
            return;
        };
        for name in want.split(',') {
            self.dev_command(name);
        }
    }

    /// 이름 있는 개발 명령 하나(0278 R13 — `DevCmd`). 모르는 이름은 콘솔 에러 한 줄(웹 스크립트가 실패로 본다).
    #[cfg(feature = "dev")]
    pub(super) fn dev_command(&mut self, name: &str) {
        let Some(c) = DevCmd::parse(name) else {
            macroquad::logging::error!("knellDev: 모르는 명령 {name}");
            return;
        };
        match c {
            DevCmd::Start(e) => self.dev_start(e),
            DevCmd::Arm(w, s) => self.dev_arm(w, s),
            DevCmd::Sub(w) => self.dev.sub = w,
            DevCmd::Show(e) => {
                let Some(tab) = DEV_CATS
                    .iter()
                    .position(|c| dev_list().iter().any(|(k, d)| k == c && *d == e))
                else {
                    return;
                };
                let row = dev_items(tab).iter().position(|d| *d == e).unwrap_or(0);
                if matches!(self.scene(), Scene::Dev { .. }) {
                    self.go(Go::Back);
                }
                self.go(Go::Open(Scene::Dev { tab: tab as u8 }));
                self.cursor = row;
            }
        }
    }

    /// 든 무기를 정한다(개발 창 — 기록은 그대로): 방패는 "검과 방패"의 칸에서만.
    fn dev_arm(&mut self, w: crate::weapons::Id, shield: bool) {
        self.weapon = w;
        self.progress.shield = shield.then_some(crate::shield::Id::Pilgrim);
    }

    /// 줄 이름 — 모션 보기 줄은 무기 이름을 채운다.
    fn dev_name(&self, e: DevStart) -> String {
        match e {
            DevStart::Motion(w) => fill(
                e.name(),
                &[weapon_name(
                    crate::weapons::IDS[w as usize % crate::weapons::IDS.len()],
                )],
            ),
            DevStart::WeaponChest(w) => fill(e.name(), &[tr(w.def().place), weapon_name(w)]),
            _ => e.name().to_string(),
        }
    }

    /// 고른 줄의 설명 — 기술 판 줄은 지금 상태를 채운다.
    fn dev_note(&self, e: DevStart) -> String {
        match e {
            DevStart::SkillsMax | DevStart::SkillsFlip => {
                let state = match self.dev.board {
                    None => dv::BOARD_SLOT,
                    Some(b) if b.pick[0] == crate::skills::Pick::A => dv::BOARD_MAX_A,
                    Some(_) => dv::BOARD_MAX_B,
                };
                fill(&e.note(), &[t(state)])
            }
            DevStart::Arm => fill(
                &e.note(),
                &[
                    weapon_name(self.weapon),
                    &arm_names(arm_cycle().into_iter().map(Some)),
                ],
            ),
            DevStart::ArmSub => fill(
                &e.note(),
                &[
                    self.dev.sub.map_or(s().slot_empty, weapon_name),
                    &arm_names(
                        std::iter::once(None)
                            .chain(sub_ids().into_iter().map(|w| Some((w, false)))),
                    ),
                ],
            ),
            DevStart::ArmBare => fill(
                &e.note(),
                &[t(if self.dev.bare {
                    dv::ARM_BARE_ON
                } else {
                    dv::ARM_BARE_OFF
                })],
            ),
            _ => e.note(),
        }
    }

    /// 개발 창 그림(nail `draw_dev`): 창 틀 · 묶음 탭 줄 · 왼쪽 줄 목록 · 오른쪽 이름·설명 · 아래 안내.
    pub(super) fn draw_dev_screen(&mut self, f: &Frame, tab: usize) {
        let tab = tab.min(DEV_CATS.len() - 1);
        self.ui.dim(0.45);
        let u = self.ui.u(1.0);
        let px = self.ui.px(1.0);
        let panel = self.panel_rect(f, 440.0, 226.0);
        let top = self.ui.panel(panel, Some(t(dv::DEV_TITLE)));
        let x = self.ui.close(panel);
        self.hits.push((x, Hit::Close));
        let (lx, rx_end) = (panel.x + u * 10.0, panel.x + panel.w - u * 10.0);
        // 묶음 탭: 고른 탭은 밝은 단추 — 다 들면 꽉 채우고, 넘치면 가로로 굴린다(0257, `ui::button_strip`).
        let tab_h = px * 1.6;
        let names: Vec<&str> = DEV_CATS.iter().map(|c| c.name()).collect();
        let rects =
            self.ui
                .button_strip(&mut self.small_strip, &names, tab, lx, rx_end, top, tab_h);
        for (i, r) in rects.into_iter().enumerate() {
            self.hits.push((r, Hit::Cell(1, i)));
        }
        let tabs_end = top + tab_h;
        // 왼쪽 줄 목록 · 오른쪽 이름·설명(세로 선으로 가름).
        let body = tabs_end + u * 8.0;
        let hint_y = panel.y + panel.h - u * 8.0 - px;
        let split = panel.x + panel.w * 0.45;
        draw_line(split, body, split, hint_y - u * 4.0, u.max(1.0), ui::BORDER);
        let items = dev_items(tab);
        let names: Vec<String> = items.iter().map(|e| self.dev_name(*e)).collect();
        let rows: Vec<ui::Row> = names
            .iter()
            .map(String::as_str)
            .map(ui::Row::plain)
            .collect();
        let area = Rect::new(
            lx - u * 4.0,
            body,
            split - lx - u * 4.0,
            hint_y - u * 6.0 - body,
        );
        let cursor = self.cursor.min(items.len().saturating_sub(1));
        let hits = self.ui.list(&rows, cursor, &mut self.view, area);
        self.push_hits(&hits);
        if let Some(e) = items.get(cursor) {
            let dx = split + u * 10.0;
            let w = rx_end - dx;
            let name = self.ui.fit(&self.dev_name(*e), w, 1.0);
            self.ui.text(&name, dx, body, 1.0, AMBER);
            let line_h = px * 1.3;
            let fit = ((hint_y - u * 6.0 - body - px * 1.6) / line_h)
                .floor()
                .max(1.0) as usize;
            for (k, l) in ui::wrap(&self.ui, &self.dev_note(*e), w, 0.9)
                .iter()
                .take(fit)
                .enumerate()
            {
                self.ui
                    .text(l, dx, body + px * 1.6 + line_h * k as f32, 0.9, DIM);
            }
        }
        let hint = self.ui.fit(t(dv::DEV_HINT), panel.w - u * 20.0, 0.85);
        self.ui.text(&hint, lx, hint_y, 0.85, DIM);
        self.push_outside(panel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 0075·0083: 시나리오는 모두 개발 창의 줄에 꼭 한 번, 묶음은 비지 않는다.
    #[test]
    fn every_scenario_sits_once_and_no_group_is_empty() {
        for e in scenario::LIST.iter() {
            let n = dev_list()
                .iter()
                .filter(|(_, d)| *d == DevStart::Scenario(e.scenario))
                .count();
            assert_eq!(n, 1, "{}", e.name);
        }
        for (i, c) in DEV_CATS.iter().enumerate() {
            assert!(!dev_items(i).is_empty(), "{c:?} is empty");
        }
        // 줄의 자리는 자유다 — 시험 스크립트는 줄을 세지 않고 이름으로 찾는다(0278 R13 — `every_dev_command_name_round_trips`).
        // 모션: 무기마다 한 줄(무기 표의 차례).
        assert_eq!(
            dev_items(1),
            (0..crate::weapons::IDS.len() as u8)
                .map(DevStart::Motion)
                .collect::<Vec<_>>()
        );
    }

    /// 0278 R13: 이름 있는 개발 명령은 모두 이름 → 명령 → 이름으로 돌아오고 이름이 겹치지 않는다 — 줄·무기의 이름을 바꾸면
    /// 헤드리스 스크립트가 아니라 이 시험이 먼저 깨진다. 스크립트가 부르는 이름은 여기 박아 둔다(`tools/test/*.js`).
    #[test]
    fn every_dev_command_name_round_trips() {
        let all = DevCmd::all();
        let mut seen = std::collections::HashSet::new();
        for c in &all {
            let n = c.name();
            assert_eq!(DevCmd::parse(&n), Some(*c), "{n}");
            assert!(seen.insert(n.clone()), "duplicate name {n}");
        }
        for (_, e) in dev_list() {
            assert_eq!(DevCmd::parse(&format!("{e:?}")), Some(DevCmd::Start(e)));
        }
        assert_eq!(DevCmd::parse("NoSuchRow"), None);
        // 시험 스크립트가 쓰는 이름(tools/test/*.js — 하나라도 바뀌면 그 스크립트를 같이 고친다).
        for n in [
            "Gate",
            "Entrance",
            "Lab",
            "Skills",
            "SkillsMax",
            "Scenario(Skills)",
            "Scenario(Boss)",
            "Scenario(Duel)",
            "Scenario(Elites)",
            "Scenario(Horde)",
            "Scenario(Line)",
            "Scenario(Stress40)",
            "Scenario(Stress80)",
            "Motion(0)",
            "Dev(Arm)",
            "Dev(ArmSub)",
            "Dev(ArmBare)",
            "ArmBare",
            "Arm(Fists)",
            "Arm(Bow)",
            "Arm(Sword)",
            "Arm(Hammer)",
            "Arm(Staff)",
            "Arm(Crossbow)",
            "Arm(Sword+Shield)",
            "Arm(Spear)",
            "Arm(Daggers)",
            "Arm(Greatsword)",
            "Arm(Scythe)",
            "Arm(Wand)",
            "ArmSub(None)",
            "ArmSub(Hammer)",
            "ArmSub(Bow)",
        ] {
            assert!(DevCmd::parse(n).is_some(), "{n}");
        }
    }

    /// 0278 R19: 무기가 나오는 상자의 줄은 스테이지 표에서 선다 — 상자에 든 무기마다 한 줄, 지역은 그 무기를 든 곳(옛 줄
    /// 일곱의 지역 그대로: 창 · 쌍단검 둘째 · 대검 · 낫 셋째 · 완드 · 쇠뇌 · 지팡이 넷째 — 망치 · 활의 상자도 이제 선다).
    #[test]
    fn weapon_chest_rows_come_from_the_stage_tables() {
        use crate::stage::Loot;
        use crate::weapons::Id as W;
        let rows: Vec<W> = dev_items(DEV_CATS.iter().position(|c| *c == DevCat::Stage).unwrap())
            .into_iter()
            .filter_map(|e| match e {
                DevStart::WeaponChest(w) => Some(w),
                _ => None,
            })
            .collect();
        for (w, region) in [
            (W::Spear, 1),
            (W::Daggers, 1),
            (W::Greatsword, 2),
            (W::Scythe, 2),
            (W::Wand, 3),
            (W::Crossbow, 3),
            (W::Staff, 3),
        ] {
            assert!(rows.contains(&w), "{w:?}");
            assert_eq!(
                DevStart::WeaponChest(w).chest(),
                Some((region, Some(Loot::Weapon(w)))),
                "{w:?}"
            );
        }
        for w in &rows {
            let (region, _) = DevStart::WeaponChest(*w).chest().expect("a chest");
            assert!(crate::stage::STAGES[region]
                .chest_loot
                .contains(&Loot::Weapon(*w)));
        }
        assert!(!rows.contains(&W::Sword) && !rows.contains(&W::Fists));
        assert_eq!(
            DevCmd::parse("WeaponChest(Wand)"),
            Some(DevCmd::Start(DevStart::WeaponChest(W::Wand)))
        );
    }

    /// 0158: 개발 창의 "다 올리기"는 켬/끔, "고르기 바꾸기"는 끝 단 판의 고르기를 모두 뒤집는다(꺼져 있으면 ㄴ으로 켠다).
    /// 다른 줄은 판을 건드리지 않는다.
    #[test]
    fn the_dev_board_rows_toggle_max_ranks_and_flip_every_pick() {
        use crate::skills::{Board, Pick, MAX_RANK, ULT_MAX_RANK};
        let on = dev_board_step(None, DevStart::SkillsMax).expect("on");
        assert!(
            on.rank.iter().all(|&r| r == MAX_RANK) && on.ult == [ULT_MAX_RANK; crate::skills::NU]
        );
        assert!(on.pick.iter().all(|&p| p == Pick::A));
        assert_eq!(dev_board_step(Some(on), DevStart::SkillsMax), None, "off");
        let b = dev_board_step(Some(on), DevStart::SkillsFlip).expect("stays on");
        assert!(b.pick.iter().all(|&p| p == Pick::B));
        assert_eq!(dev_board_step(Some(b), DevStart::SkillsFlip), Some(on));
        assert_eq!(
            dev_board_step(None, DevStart::SkillsFlip),
            Some(Board::maxed(Pick::B))
        );
        assert_eq!(dev_board_step(Some(on), DevStart::Lab), Some(on));
    }
}
