//! 장면 흐름(0019) — 순수 함수. 앱은 여기서 정한 대로만 바꾼다(첫 게임 `app/flow.rs`, nail 0230 R15).
//! 흐름을 바꾸면 여기 시험을 더한다.
use crate::stage::StageOut;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scene {
    Title,
    Play,
    Pause,
    /// 스테이지가 끝나고 장면이 멈춰 보이는 동안(쓰러짐·종소리) — 이어서 `Result`.
    Ending {
        won: bool,
    },
    Result {
        won: bool,
    },
    /// 조작 설정(0024). 설정에서 연다(0052).
    Controls,
    /// 장비(0064 M3 → 0067 D 인형): 무기 + 장신구 세 칸. 멈춤 "장비"에서, 뒤로 = 멈춤.
    Equip,
    /// 장비 칸의 후보 목록(0067): 장비 창 위에 얹는다 — 닫으면 그 칸으로.
    EquipPick {
        slot: u8,
    },
    /// 쉼터 화로(0104, v0.4.0 G1 — 0102 7): 쉬면 열린다. 줄 = 레벨 · 가게 · 떠나기. 뒤로 = 싸움.
    Hearth,
    /// 레벨(0104): 생명·기력·힘을 잔불로 한 단씩. 화로에서 연다 — 닫으면 화로.
    Level,
    /// 기술 나무(0106): 화로에서 열면 칸을 연다(`view = false`), 멈춤에서 열면 읽기·목표 핀만(`view = true`). 닫으면 연 곳.
    Tree {
        view: bool,
    },
    /// 도감(0130 C5): 탭 둘(적 · 장신구), 왼쪽 목록 · 오른쪽 그림과 관찰. 멈춤에서 연다 — 닫으면 멈춤.
    Codex {
        tab: u8,
    },
    /// 기술(0151 S0): 스킬 칸 둘에 가진 스킬을 끼운다. 멈춤에서 연다 — 닫으면 멈춤.
    Skills,
    /// 가게(0064 M4): 잔불로 장신구·등잔 기름을 산다. 화로에서 연다(0104 전엔 쉬면 바로 열렸다) — 닫으면 화로.
    Shop,
    /// 묻는 창(KB `controller-layout-and-remapping` "되돌릴 수 없는 메뉴 항목은 한 번 묻는다", nail 0274·가게 묻는 창):
    /// 줄 둘(하는 일 이름 / 취소) + 무엇을 잃는지 한 줄. 뒤로 = 취소(온 곳).
    Ask(Ask),
    /// 설정(0052, nail 0269의 틀): 언어·전체 화면·화면 흔들림·조작. 타이틀·멈춤에서 연다 — 닫으면 연 곳(스택, 0066).
    Settings,
    /// 슬롯 창(0066, KB `roguelite-save-slots-and-continue`): `load`면 이어하기(기록 있는 슬롯만 · 이어하기/지우기), 아니면 새로 시작(빈 슬롯은 시작, 기록 있으면 덮어쓰기). 둘 다 두 번 눌러야 지운다.
    Slots {
        load: bool,
    },
    /// 개발 창(0052 → 0075 → 0083, nail `Scene::Dev`·0269 구조): 묶음 `tab`(`dev::DEV_CATS`)의 시작할 것 목록. 개발판에만.
    #[cfg(feature = "dev")]
    Dev {
        tab: u8,
    },
    /// 개발 시나리오(PoC 아레나). 개발판에만.
    #[cfg(feature = "dev")]
    Sandbox,
    /// 연구소(랩, 0070·0079, 개발판): 묶음 목록 + A/B·내보내기. 타이틀·멈춤 메뉴의 설정 바로 아래 줄·F1에서 연다. 게임은 멈추고 값은 바로 들어간다.
    #[cfg(feature = "dev")]
    Lab,
    /// 랩의 한 묶음(`lab::GROUPS[g]`): 줄 = 이름 ◀ 값 ▶.
    #[cfg(feature = "dev")]
    LabGroup {
        g: u8,
    },
}

impl Scene {
    /// 메뉴 입력을 받는 장면(터치는 탭만, 패드 십자키는 커서, 메뉴 소리) = **싸움 화면이 아닌 모든 장면**.
    /// 예전에는 장면을 하나씩 적은 목록이었고 도감(0130)이 빠져 있었다 — 도감에 들어가면 터치가 싸움 조작(스틱·버튼)으로 읽혀
    /// 탭이 안 먹었다(사장 2026-11-11). `layer()`는 `_` 없는 match라 새 장면을 빠뜨릴 수 없다 — 거기서 끌어온다.
    pub fn is_menu(self) -> bool {
        self.layer() != Layer::Game
    }

    /// 무엇 위에 그리나(0066, nail 0235 R21 — `_` 없는 match: 새 장면이면 컴파일러가 묻는다).
    pub fn layer(self) -> Layer {
        match self {
            Scene::Title => Layer::Screen,
            Scene::Play | Scene::Ending { .. } => Layer::Game,
            #[cfg(feature = "dev")]
            Scene::Sandbox => Layer::Game,
            Scene::Pause
            | Scene::Equip
            | Scene::EquipPick { .. }
            | Scene::Settings
            | Scene::Skills => Layer::Window { hud: true },
            // 가게는 잔불을 제가 보인다(KB hud-readability: 창이 같은 정보면 밑 HUD를 그리지 않는다).
            Scene::Hearth
            | Scene::Level
            | Scene::Tree { .. }
            | Scene::Codex { .. }
            | Scene::Shop
            | Scene::Result { .. }
            | Scene::Controls
            | Scene::Slots { .. } => Layer::Window { hud: false },
            #[cfg(feature = "dev")]
            Scene::Dev { .. } => Layer::Window { hud: false },
            #[cfg(feature = "dev")]
            Scene::Lab | Scene::LabGroup { .. } => Layer::Window { hud: true },
            Scene::Ask(_) => Layer::Modal,
        }
    }

    /// 월드가 도는 장면.
    pub fn ticks_world(self) -> bool {
        match self {
            Scene::Play => true,
            #[cfg(feature = "dev")]
            Scene::Sandbox => true,
            _ => false,
        }
    }
}

/// 장면이 그리는 자리(0066).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Layer {
    /// 화면 하나를 다 쓴다(타이틀: 키 아트 위 목록).
    Screen,
    /// 싸움 화면(HUD).
    Game,
    /// 창: 판 위에서 열렸으면 밑에 HUD를 깐다(`hud`일 때).
    Window { hud: bool },
    /// 묻는 창: 아래 칸을 그대로 깔고 어둡게, 그 위에.
    Modal,
}

/// 장면을 바꾸는 방법(0066, 장면 스택 — Nystrom "pushdown automata"): 위에 얹기 · 꺼내기(뒤로) · 처음부터.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Go {
    Stay,
    /// 지금 장면 위에 연다(닫으면 지금 장면 · 지금 커서로 돌아온다).
    Open(Scene),
    /// 위 칸을 닫는다. 맨 아래 칸이면 그대로.
    Back,
    /// 스택을 비우고 이 장면부터.
    Root(Scene),
}

/// 창을 열 때 처음 포커스(0066 — Microsoft 패드 지침 "처음 포커스는 뜻이 있는 자리"): 묻는 창은 `ask_default`, 그 밖은 첫 줄.
pub fn focus(scene: Scene) -> usize {
    match scene {
        Scene::Ask(k) => ask_default(k),
        _ => 0,
    }
}

/// 묻는 일.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ask {
    /// 멈춤 "처음부터": 쉼터 자리를 버린다.
    Restart,
    /// 멈춤 "타이틀로": 마지막 쉼 뒤의 진행을 버린다.
    ToTitle,
    /// 화로 가게: 이 물건(살 수 있는 것 목록의 자리)을 산다.
    Buy(usize),
    /// 화로 레벨(0104): 이 줄(`progression::LINES`의 자리)을 한 단 올린다.
    Level(u8),
    /// 나무(0106): 이 갈래의 다음 칸을 연다(점수 1).
    Tree(u8),
}

/// 묻는 창의 커서 처음 자리: 버리는 일은 "취소"(1)에서 — 연타가 실행되지 않게(KB). 사기는 nail 가게처럼 "사기"(0)에서.
pub fn ask_default(k: Ask) -> usize {
    match k {
        Ask::Buy(_) | Ask::Level(_) | Ask::Tree(_) => 0,
        _ => 1,
    }
}

/// 묻는 창의 답 → (어디로, 판을 다시 세우나). 취소 = 닫기(온 곳).
pub fn answer(k: Ask, yes: bool) -> (Go, bool) {
    if !yes {
        return (Go::Back, false);
    }
    match k {
        Ask::Restart => (Go::Root(Scene::Play), true),
        Ask::ToTitle => (Go::Root(Scene::Title), true),
        Ask::Buy(_) | Ask::Level(_) | Ask::Tree(_) => (Go::Back, false),
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TitleItem {
    /// 이어하기(기록 있는 슬롯이 하나라도 있을 때만, 맨 위) → 슬롯 창(이어하기).
    Continue,
    /// 시작하기(늘) → 슬롯 창(새로 시작). KB: 처음 하는 사람은 이어하기가 안 보여 헷갈릴 일이 없다.
    Start,
    /// 설정(0052): 언어·전체 화면·흔들림·조작이 여기로 모였다(타이틀에 흩어져 있었다).
    Settings,
    /// 개발 메뉴(0052). 개발판에만 — 설정 바로 아래(0094: 연구소는 바깥 메뉴에 없고 개발 창의 도구 탭에서만).
    #[cfg(feature = "dev")]
    Dev,
    /// 끝내기(PC만, 0135 — nail 0240): 맨 끝. 전체 화면·패드로 하면 창 ✕가 없다. 기록은 바뀔 때마다 이미 썼다(`persist`).
    Quit,
}

/// 타이틀 줄(0066·0094): [이어하기(기록이 있으면)] · 시작하기 · 설정 · [개발(개발판)] · [끝내기(PC)]. 기록이 없으면 설정은 둘째(헤드리스 시험의 길).
pub fn title_items(has_save: bool) -> Vec<TitleItem> {
    let mut v = Vec::new();
    if has_save {
        v.push(TitleItem::Continue);
    }
    v.extend([TitleItem::Start, TitleItem::Settings]);
    #[cfg(feature = "dev")]
    v.push(TitleItem::Dev);
    if crate::DESKTOP {
        v.push(TitleItem::Quit);
    }
    v
}

/// 슬롯 창을 열 때의 포커스(KB): 이어하기는 마지막 슬롯(비었으면 첫 기록), 새로 시작은 첫 빈 슬롯(다 찼으면 첫째).
pub fn slot_focus(load: bool, filled: &[bool], last: usize) -> usize {
    if load {
        if filled.get(last).copied().unwrap_or(false) {
            last
        } else {
            filled.iter().position(|&f| f).unwrap_or(0)
        }
    } else {
        filled.iter().position(|&f| !f).unwrap_or(0)
    }
}

/// 이어하기 창에서 위아래: 빈 슬롯은 건너뛴다(고를 수 없는 줄에 서지 않게). 기록이 하나도 없으면 그대로.
pub fn slot_step(from: usize, dir: i32, filled: &[bool]) -> usize {
    let n = filled.len() as i32;
    let mut i = from as i32;
    for _ in 0..n {
        i = (i + dir).rem_euclid(n);
        if filled[i as usize] {
            return i as usize;
        }
    }
    from
}

/// 슬롯 줄에서 고르면.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SlotDo {
    /// 아무것도(이어하기 창의 빈 슬롯).
    Nothing,
    /// 한 번 더 눌러야 한다(덮어쓰기·지우기 — 단추 글이 "정말 …?"로 바뀐다).
    Arm,
    /// 이 슬롯을 비우고(이미 비었으면 그대로) 새로 시작.
    Fresh,
    /// 이 슬롯의 기록으로 이어한다.
    Load,
    /// 이 슬롯을 지운다.
    Erase,
}

/// 고름 → 할 일(`col`: 이어하기 창의 단추 0 = 이어하기 · 1 = 지우기, `armed`: 이미 한 번 눌렀나).
pub fn slot_pick(load: bool, filled: bool, col: usize, armed: bool) -> SlotDo {
    match (load, filled, col, armed) {
        (true, false, _, _) => SlotDo::Nothing,
        (true, true, 0, _) => SlotDo::Load,
        (true, true, _, false) => SlotDo::Arm,
        (true, true, _, true) => SlotDo::Erase,
        (false, false, _, _) => SlotDo::Fresh,
        (false, true, _, false) => SlotDo::Arm,
        (false, true, _, true) => SlotDo::Fresh,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PauseItem {
    Resume,
    /// 장비(0064 M3, 0051의 무기 줄): 좌우 = 다음 무기(전처럼), 확인 = 장비 창(무기 + 장신구 세 칸).
    Weapon,
    /// 기술 나무(0106) — 읽기·목표 핀만(칸은 쉼터 화로에서 연다).
    Tree,
    /// 도감(0130) — 나무 다음 줄.
    Codex,
    /// 기술(0151) — 도감 다음 줄.
    Skills,
    Settings,
    /// 개발 메뉴(0075 → 0090) — 개발판에만, 설정 바로 아래 줄(연구소가 있던 자리 — 연구소는 개발 창 도구 탭·타이틀에서, 사장 2026-11-07).
    #[cfg(feature = "dev")]
    Dev,
    Restart,
    ToTitle,
}

/// 멈춤 창의 줄: 계속 · 장비 · 나무(0106) · 도감(0130) · 설정 · [개발] · 처음부터 · 타이틀로 — 대괄호는 개발판에만(0090).
pub fn pause_items() -> Vec<PauseItem> {
    let mut v = vec![
        PauseItem::Resume,
        PauseItem::Weapon,
        PauseItem::Tree,
        PauseItem::Codex,
        PauseItem::Skills,
        PauseItem::Settings,
    ];
    #[cfg(feature = "dev")]
    v.push(PauseItem::Dev);
    v.extend([PauseItem::Restart, PauseItem::ToTitle]);
    v
}

/// 설정 줄(0052). 값 줄은 좌우·확인으로 바꾸고, 누르는 줄은 확인으로 들어간다.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SettingsRow {
    Language,
    /// 쓸 수 있는 기기에서만(웹·데스크톱, 0046).
    Fullscreen,
    /// 화면 흔들림 0..=10.
    Shake,
    /// 소리 크기 0..=10(0064 M5).
    Volume,
    /// 바닥 공격 예고 진하기 0..=10, 0 = 끔(0074, nail 0198 꼴).
    Tells,
    /// 바닥 예고의 "언제"(채움) 켬/끔(0128).
    TellTiming,
    /// 마우스 조준 켬/끔·커서 쉼 시간·조준 표시 크기(0058) — 마우스가 있을 법한 기기에서만.
    MouseAim,
    CursorRest,
    Reticle,
    Controls,
    Back,
}

/// 조작 설정은 늘 끝에서 둘째(`tools/test/shot.js`가 위로 두 번 간다).
pub fn settings_rows(fullscreen: bool, mouse: bool) -> Vec<SettingsRow> {
    let mut v = vec![SettingsRow::Language];
    if fullscreen {
        v.push(SettingsRow::Fullscreen);
    }
    v.push(SettingsRow::Shake);
    v.push(SettingsRow::Tells);
    v.push(SettingsRow::TellTiming);
    v.push(SettingsRow::Volume);
    if mouse {
        v.extend([
            SettingsRow::MouseAim,
            SettingsRow::CursorRest,
            SettingsRow::Reticle,
        ]);
    }
    v.extend([SettingsRow::Controls, SettingsRow::Back]);
    v
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResultItem {
    Retry,
    ToTitle,
}

pub const RESULT_ITEMS: [ResultItem; 2] = [ResultItem::Retry, ResultItem::ToTitle];

/// 스테이지가 이렇게 끝나면 어디로. 이어지는 중이면 그대로.
pub fn after_stage(out: StageOut) -> Option<Scene> {
    match out {
        // 지역 잇기(0118)는 앱이 판 안에서 한다 — 장면은 그대로.
        StageOut::Playing
        | StageOut::Respawned
        | StageOut::CoreTaken
        | StageOut::Descend
        | StageOut::Ascend => None,
        StageOut::Cleared => Some(Scene::Ending { won: true }),
        StageOut::Defeated => Some(Scene::Ending { won: false }),
    }
}

/// 끝 장면을 이만큼(초) 보여 준 뒤 결과 창.
pub const ENDING_TIME: f32 = 1.6;

/// 끝 장면 뒤: 결과(핵 고르기는 0105에서 걷었다 — 핵은 기술 나무 점수, 알림만).
pub fn after_ending(won: bool) -> Scene {
    Scene::Result { won }
}

/// 뒤로(B·Esc): 연 창은 닫는다(스택에서 꺼내 연 곳으로). 골라야 끝나는 창(핵 고르기·결과)·타이틀·싸움은 그대로(0066).
pub fn back(scene: Scene) -> Go {
    match scene {
        Scene::Pause
        | Scene::Equip
        | Scene::EquipPick { .. }
        | Scene::Hearth
        | Scene::Level
        | Scene::Tree { .. }
        | Scene::Codex { .. }
        | Scene::Skills
        | Scene::Shop
        | Scene::Ask(_)
        | Scene::Controls
        | Scene::Settings
        | Scene::Slots { .. } => Go::Back,
        #[cfg(feature = "dev")]
        Scene::Dev { .. } | Scene::Lab | Scene::LabGroup { .. } => Go::Back,
        Scene::Title | Scene::Play | Scene::Ending { .. } | Scene::Result { .. } => Go::Stay,
        #[cfg(feature = "dev")]
        Scene::Sandbox => Go::Stay,
    }
}

/// 멈춤 버튼: 싸움 중이면 멈춤을 열고, 멈춤이면 닫는다.
pub fn pause(scene: Scene) -> Go {
    match scene {
        Scene::Play => Go::Open(Scene::Pause),
        Scene::Pause => Go::Back,
        _ => Go::Stay,
    }
}

/// 이 장면에서 고르면: (어디로, 스테이지를 새로 세우나).
pub fn choose_pause(item: PauseItem) -> (Go, bool) {
    match item {
        PauseItem::Resume => (Go::Back, false),
        PauseItem::Weapon => (Go::Open(Scene::Equip), false),
        PauseItem::Tree => (Go::Open(Scene::Tree { view: true }), false),
        PauseItem::Codex => (Go::Open(Scene::Codex { tab: 0 }), false),
        PauseItem::Skills => (Go::Open(Scene::Skills), false),
        PauseItem::Settings => (Go::Open(Scene::Settings), false),
        PauseItem::Restart => (Go::Open(Scene::Ask(Ask::Restart)), false),
        PauseItem::ToTitle => (Go::Open(Scene::Ask(Ask::ToTitle)), false),
        #[cfg(feature = "dev")]
        PauseItem::Dev => (Go::Open(Scene::Dev { tab: 0 }), false),
    }
}

pub fn choose_result(item: ResultItem) -> (Go, bool) {
    match item {
        ResultItem::Retry => (Go::Root(Scene::Play), true),
        ResultItem::ToTitle => (Go::Root(Scene::Title), true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_finished_stage_shows_its_ending_before_the_result() {
        assert_eq!(after_stage(StageOut::Playing), None);
        assert_eq!(
            after_stage(StageOut::Cleared),
            Some(Scene::Ending { won: true })
        );
        assert_eq!(
            after_stage(StageOut::Defeated),
            Some(Scene::Ending { won: false })
        );
        assert!(!Scene::Ending { won: true }.ticks_world());
        assert!(!Scene::Ending { won: true }.is_menu());
    }

    #[test]
    fn pause_opens_and_back_closes() {
        assert_eq!(pause(Scene::Play), Go::Open(Scene::Pause));
        assert_eq!(pause(Scene::Pause), Go::Back);
        assert_eq!(back(Scene::Pause), Go::Back);
        assert_eq!(pause(Scene::Title), Go::Stay);
        // 골라야 끝나는 창은 뒤로로 건너뛰지 않는다.
        assert_eq!(back(Scene::Result { won: false }), Go::Stay);
        assert_eq!(back(Scene::Title), Go::Stay);
    }

    #[test]
    fn menus_rebuild_the_stage_except_resume() {
        assert_eq!(choose_pause(PauseItem::Resume), (Go::Back, false));
        // 버리는 일은 한 번 묻는다(KB): 처음부터·타이틀로 → 묻는 창, 답해야 판을 다시 세운다.
        assert_eq!(
            choose_pause(PauseItem::Restart),
            (Go::Open(Scene::Ask(Ask::Restart)), false)
        );
        assert_eq!(
            choose_pause(PauseItem::ToTitle),
            (Go::Open(Scene::Ask(Ask::ToTitle)), false)
        );
        assert_eq!(answer(Ask::Restart, true), (Go::Root(Scene::Play), true));
        assert_eq!(answer(Ask::ToTitle, true), (Go::Root(Scene::Title), true));
        assert_eq!(
            answer(Ask::Buy(2), true),
            (Go::Back, false),
            "back to the shop"
        );
        // 0104: 화로 레벨도 가게처럼 "올리기"에서, 답하면 레벨 창으로.
        assert_eq!(answer(Ask::Level(1), true), (Go::Back, false));
        assert_eq!(ask_default(Ask::Level(0)), 0);
        // 도감(0130)·기술(0151)도 메뉴다 — 터치가 탭으로 읽힌다(도감이 목록에서 빠져 터치가 안 먹던 것).
        for sc in [
            Scene::Codex { tab: 0 },
            Scene::Codex { tab: 1 },
            Scene::Skills,
        ] {
            assert!(sc.is_menu(), "{sc:?} takes taps");
            assert_eq!(back(sc), Go::Back);
        }
        // 싸움 화면만 메뉴가 아니다.
        assert!(!Scene::Play.is_menu());
        for sc in [Scene::Hearth, Scene::Level, Scene::Shop] {
            assert_eq!(back(sc), Go::Back, "{sc:?} closes to where it opened");
            assert!(sc.is_menu());
        }
        for k in [Ask::Restart, Ask::ToTitle, Ask::Buy(0), Ask::Level(2)] {
            assert_eq!(answer(k, false), (Go::Back, false), "cancel closes");
            assert_eq!(back(Scene::Ask(k)), Go::Back);
            assert_eq!(Scene::Ask(k).layer(), Layer::Modal);
        }
        // 커서: 버리는 일은 취소에서, 사기는 사기에서 — 여는 순간의 포커스(0066).
        assert_eq!(focus(Scene::Ask(Ask::Restart)), 1);
        assert_eq!(focus(Scene::Ask(Ask::Buy(0))), 0);
        assert_eq!(focus(Scene::Pause), 0);
        assert_eq!(
            choose_result(ResultItem::Retry),
            (Go::Root(Scene::Play), true)
        );
        // 0066: 기록이 없으면 시작하기·설정(설정은 둘째), 있으면 이어하기가 맨 위에.
        assert_eq!(
            title_items(false)[..2],
            [TitleItem::Start, TitleItem::Settings]
        );
        assert!(!title_items(false).contains(&TitleItem::Continue));
        assert_eq!(
            title_items(true)[..3],
            [TitleItem::Continue, TitleItem::Start, TitleItem::Settings]
        );
        // 0135: 끝내기는 PC에만, 늘 맨 끝(시험은 PC에서 돈다). 웹·Android 줄은 그대로라 헤드리스 시험의 길이 같다.
        for has in [false, true] {
            let v = title_items(has);
            assert_eq!(v.last() == Some(&TitleItem::Quit), crate::DESKTOP);
            assert_eq!(
                v.iter().filter(|&&i| i == TitleItem::Quit).count(),
                crate::DESKTOP as usize
            );
        }
        assert_eq!(
            choose_pause(PauseItem::Settings),
            (Go::Open(Scene::Settings), false),
            "the run is kept"
        );
        assert_eq!(
            choose_pause(PauseItem::Weapon),
            (Go::Open(Scene::Equip), false)
        );
        // 0105: 핵 고르기는 없다 — 이겨도 져도 결과.
        assert_eq!(after_ending(true), Scene::Result { won: true });
        assert_eq!(after_ending(false), Scene::Result { won: false });
        assert!(Scene::Controls.is_menu());
        assert!(Scene::Settings.is_menu());
        assert!(!settings_rows(false, true).contains(&SettingsRow::Fullscreen));
        assert!(!settings_rows(true, false).contains(&SettingsRow::MouseAim));
        for (f, m) in [(false, false), (true, true), (true, false)] {
            let r = settings_rows(f, m);
            assert_eq!(r[r.len() - 2..], [SettingsRow::Controls, SettingsRow::Back]);
        }
    }

    #[test]
    fn layers_put_the_hud_only_under_windows_that_lack_it() {
        assert_eq!(Scene::Title.layer(), Layer::Screen);
        assert_eq!(Scene::Play.layer(), Layer::Game);
        assert_eq!(Scene::Pause.layer(), Layer::Window { hud: true });
        assert_eq!(
            Scene::Shop.layer(),
            Layer::Window { hud: false },
            "shows its own embers"
        );
    }

    #[test]
    fn slot_windows_focus_skip_and_need_two_presses_to_destroy() {
        let filled = [true, false, true];
        // 이어하기: 마지막 슬롯(기록이 있으면), 없으면 첫 기록. 새로 시작: 첫 빈 슬롯.
        assert_eq!(slot_focus(true, &filled, 2), 2);
        assert_eq!(slot_focus(true, &filled, 1), 0);
        assert_eq!(slot_focus(false, &filled, 0), 1);
        assert_eq!(slot_focus(false, &[true; 3], 2), 0, "all full: first");
        // 이어하기 창의 위아래는 빈 슬롯을 건너뛴다(끝에서 돈다).
        assert_eq!(slot_step(0, 1, &filled), 2);
        assert_eq!(slot_step(2, 1, &filled), 0);
        assert_eq!(slot_step(0, -1, &filled), 2);
        assert_eq!(slot_step(1, 1, &[false; 3]), 1, "nothing to stand on");
        // 덮어쓰기·지우기는 두 번, 이어하기·빈 슬롯 시작은 한 번.
        assert_eq!(slot_pick(false, false, 0, false), SlotDo::Fresh);
        assert_eq!(slot_pick(false, true, 0, false), SlotDo::Arm);
        assert_eq!(slot_pick(false, true, 0, true), SlotDo::Fresh);
        assert_eq!(slot_pick(true, true, 0, false), SlotDo::Load);
        assert_eq!(slot_pick(true, true, 1, false), SlotDo::Arm);
        assert_eq!(slot_pick(true, true, 1, true), SlotDo::Erase);
        assert_eq!(slot_pick(true, false, 0, true), SlotDo::Nothing);
        assert_eq!(back(Scene::Slots { load: true }), Go::Back);
        assert_eq!(
            Scene::Slots { load: false }.layer(),
            Layer::Window { hud: false }
        );
    }

    /// 0094(사장 2026-11-07 "연구소 메뉴가 바깥(첫 화면 메뉴나 멈춤 화면 메뉴)에서도 표시됨"): 바깥 메뉴엔 연구소가 없고
    /// 설정 바로 아래가 개발 — 연구소는 개발 창 도구 탭에서만(`app/dev.rs`).
    #[cfg(feature = "dev")]
    #[test]
    fn menus_have_dev_below_settings_and_no_lab() {
        for has_save in [false, true] {
            let t = title_items(has_save);
            let i = t.iter().position(|x| *x == TitleItem::Settings).unwrap();
            assert_eq!(t.get(i + 1), Some(&TitleItem::Dev));
            // + 끝내기(PC, 0135 — 맨 끝).
            assert_eq!(t.len(), 3 + has_save as usize + crate::DESKTOP as usize);
        }
        // 멈춤엔 연구소가 없고 그 자리(설정 아래)에 개발 줄(0090) — 연구소는 개발 창의 도구 탭에서.
        let p = pause_items();
        let i = p.iter().position(|x| *x == PauseItem::Settings).unwrap();
        assert_eq!(p.get(i + 1), Some(&PauseItem::Dev));
        // 계속 · 장비 · 나무(0106) · 도감(0130) · 기술(0151) · 설정 · 개발 · 처음부터 · 타이틀로(헤드리스 스크립트가 이 차례로 간다).
        assert_eq!(p.len(), 9);
        assert_eq!(p[4], PauseItem::Skills);
        assert_eq!(
            choose_pause(PauseItem::Skills),
            (Go::Open(Scene::Skills), false)
        );
        assert_eq!(p[2], PauseItem::Tree);
        assert_eq!(p[3], PauseItem::Codex);
        assert_eq!(
            choose_pause(PauseItem::Codex),
            (Go::Open(Scene::Codex { tab: 0 }), false)
        );
        assert_eq!(
            choose_pause(PauseItem::Tree),
            (Go::Open(Scene::Tree { view: true }), false)
        );
        for fs in [false, true] {
            for mouse in [false, true] {
                assert_eq!(settings_rows(fs, mouse).len() >= 4, true);
            }
        }
    }
}
