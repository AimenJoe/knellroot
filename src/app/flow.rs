//! 장면 흐름(0019) — 순수 함수. 앱은 여기서 정한 대로만 바꾼다(첫 게임 `app/flow.rs`, nail 0230 R15).
//! 흐름을 바꾸면 여기 시험을 더한다.
use crate::stage::StageOut;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scene {
    Title,
    Play,
    /// 스테이지가 끝나고 장면이 멈춰 보이는 동안(쓰러짐·종소리) — 이어서 `Result`.
    Ending {
        won: bool,
    },
    Result {
        won: bool,
    },
    /// 조작 설정(0024). 설정에서 연다(0052).
    Controls,
    /// 큰 창의 상태 탭(0164 — 옛 레벨 창 0104가 여기로): 인형·막대, 레벨 셋(생명·기력·힘), 지금의 수치와 끼운 장신구.
    /// 화로에서는 확인 = 올리기(묻는 창), 멈춤에서는 읽기만. 멈춤과 화로가 이 탭에서 열린다.
    Status {
        host: Host,
    },
    /// 큰 창의 장비 탭(0064 M3 → 0067 D 인형 → 0164): 무기 + 장신구 세 칸. Select는 싸움에서 이 탭을 바로 연다(0070).
    Equip {
        host: Host,
    },
    /// 장비 칸의 후보 목록(0067): 장비 탭 위에 얹는다 — 닫으면 그 칸으로.
    EquipPick {
        slot: u8,
    },
    /// 큰 창의 도감 탭(0130 C5 → 0164): 몸 안의 묶음 둘(`tab`: 적 · 장신구), 왼쪽 목록 · 오른쪽 그림과 관찰.
    Codex {
        host: Host,
        tab: u8,
    },
    /// 큰 창의 기술 탭(0161 → 0164): 갈래 셋 × 줄 셋의 칸 그림 + 고른 칸의 상세. 화로에서는 올리기·고르기 바꾸기·되돌리기까지
    /// (`Host::edits`), 멈춤에서는 읽기와 끼우기만.
    Skills {
        host: Host,
    },
    /// 기술 화면에서 고른 칸에 할 수 있는 일의 작은 목록(0161): 올리기·열기 / 칸에 끼우기 / 고르기 바꾸기 — 되는 줄만.
    /// 기술 화면 위에 얹는다(칸은 아래 칸의 커서) — 닫으면 그 칸으로.
    SkillAct,
    /// 큰 창의 가게 탭(0064 M4 → 0164): 잔불로 장신구·등잔 기름을 산다. 화로에만 있다.
    Shop,
    /// 묻는 창(KB `controller-layout-and-remapping` "되돌릴 수 없는 메뉴 항목은 한 번 묻는다", nail 0274·가게 묻는 창):
    /// 줄 둘(하는 일 이름 / 취소) + 무엇을 잃는지 한 줄. 뒤로 = 취소(온 곳).
    Ask(Ask),
    /// 큰 창의 설정 탭(0052 → 0164): 언어·전체 화면·화면 흔들림·조작. 멈춤에서는 끝에 [개발] · 처음부터 · 타이틀로.
    /// 타이틀에서 열면 탭이 이것 하나.
    Settings {
        host: Host,
    },
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
    /// 연구소(랩, 0070·0079, 개발판): 묶음 목록 + A/B·내보내기. 개발 창의 도구 탭·F1에서 연다. 게임은 멈추고 값은 바로 들어간다.
    #[cfg(feature = "dev")]
    Lab,
    /// 랩의 한 묶음(`lab::GROUPS[g]`): 줄 = 이름 ◀ 값 ▶.
    #[cfg(feature = "dev")]
    LabGroup {
        g: u8,
    },
}

/// 큰 창을 연 곳(0164 — 0160 3절 "멈춤과 화로가 같은 창을 연다"): 탭 목록과 켜지는 일이 여기서 갈린다. 장면 칸이 들고 다닌다 —
/// 탭을 넘기면 맨 위 칸을 바꾸므로(쌓지 않는다) 연 곳은 칸의 값이어야 남는다.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Host {
    /// 타이틀의 설정: 탭 하나.
    Title,
    /// 싸움 중 멈춤(Esc·Start): 읽기와 끼우기만. 처음부터·타이틀로는 여기 설정 탭의 끝 줄.
    Pause,
    /// 쉼터 화로(쉬면 열린다): 올리기·사기·되돌리기가 켜지고 가게 탭이 선다.
    Hearth,
}

impl Host {
    /// 올리기·사기·되돌리기가 켜지나(쉬는 동안에만 — 0104·0161의 규칙 그대로).
    pub fn edits(self) -> bool {
        self == Host::Hearth
    }
}

/// 큰 창의 탭(0164). 차례는 `tabs`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tab {
    Status,
    Equip,
    Skills,
    Codex,
    Shop,
    Settings,
}

/// 연 곳마다의 탭 줄: 멈춤 = 상태 · 장비 · 기술 · 도감 · 설정, 화로 = 거기에 가게(설정 앞), 타이틀 = 설정 하나.
pub fn tabs(host: Host) -> &'static [Tab] {
    match host {
        Host::Title => &[Tab::Settings],
        Host::Pause => &[
            Tab::Status,
            Tab::Equip,
            Tab::Skills,
            Tab::Codex,
            Tab::Settings,
        ],
        Host::Hearth => &[
            Tab::Status,
            Tab::Equip,
            Tab::Skills,
            Tab::Codex,
            Tab::Shop,
            Tab::Settings,
        ],
    }
}

/// 탭의 장면(가게는 화로에만 있어 연 곳을 들지 않는다).
pub fn tab_scene(host: Host, tab: Tab) -> Scene {
    match tab {
        Tab::Status => Scene::Status { host },
        Tab::Equip => Scene::Equip { host },
        Tab::Skills => Scene::Skills { host },
        Tab::Codex => Scene::Codex { host, tab: 0 },
        Tab::Shop => Scene::Shop,
        Tab::Settings => Scene::Settings { host },
    }
}

/// 이 장면이 큰 창의 탭이면 (연 곳, 탭). `_` 없는 match — 새 장면이면 컴파일러가 묻는다.
pub fn tab_of(scene: Scene) -> Option<(Host, Tab)> {
    match scene {
        Scene::Status { host } => Some((host, Tab::Status)),
        Scene::Equip { host } => Some((host, Tab::Equip)),
        Scene::Skills { host } => Some((host, Tab::Skills)),
        Scene::Codex { host, .. } => Some((host, Tab::Codex)),
        Scene::Shop => Some((Host::Hearth, Tab::Shop)),
        Scene::Settings { host } => Some((host, Tab::Settings)),
        Scene::Title
        | Scene::Play
        | Scene::Ending { .. }
        | Scene::Result { .. }
        | Scene::Controls
        | Scene::EquipPick { .. }
        | Scene::SkillAct
        | Scene::Ask(_)
        | Scene::Slots { .. } => None,
        #[cfg(feature = "dev")]
        Scene::Dev { .. } | Scene::Sandbox | Scene::Lab | Scene::LabGroup { .. } => None,
    }
}

/// 탭 줄에서의 자리(그림의 "지금 탭").
pub fn tab_index(scene: Scene) -> Option<usize> {
    let (host, tab) = tab_of(scene)?;
    tabs(host).iter().position(|&t| t == tab)
}

/// 탭 넘기기(스킬 버튼 둘 — `dir` ±1): 맨 위 칸을 옆 탭의 장면으로 **바꾼다**(쌓지 않는다 — 뒤로 한 번에 닫힌다).
/// 한 줄 목록이라 끝에서 돈다(KB: Xbox 112). 탭이 하나면(타이틀의 설정) 그대로.
pub fn tab_step(scene: Scene, dir: i32) -> Go {
    let Some((host, _)) = tab_of(scene) else {
        return Go::Stay;
    };
    let (list, at) = (tabs(host), tab_index(scene).unwrap_or(0));
    let to = (at as i32 + dir).rem_euclid(list.len() as i32) as usize;
    tab_pick(scene, to)
}

/// 탭 머리를 눌렀다(터치·마우스): 그 탭으로 바꾼다. 지금 탭이면 그대로.
pub fn tab_pick(scene: Scene, i: usize) -> Go {
    let Some((host, now)) = tab_of(scene) else {
        return Go::Stay;
    };
    match tabs(host).get(i) {
        Some(&t) if t != now => Go::Swap(tab_scene(host, t)),
        _ => Go::Stay,
    }
}

/// 화로가 여는 탭(상태 — 화로의 첫 일은 레벨 올리기) · Select가 여는 탭(장비, 0070).
pub const HEARTH_OPENS: Scene = Scene::Status { host: Host::Hearth };
pub const SELECT_OPENS: Scene = Scene::Equip { host: Host::Pause };

/// 멈춤이 여는 탭: 이 판에서 멈춤 창이 마지막에 보던 탭(`last` — 판에 붙은 한 칸, 새 판이면 상태). 멈춤에 없는 탭이면 상태.
pub fn pause_opens(last: Tab) -> Scene {
    let tab = if tabs(Host::Pause).contains(&last) {
        last
    } else {
        Tab::Status
    };
    tab_scene(Host::Pause, tab)
}

/// 이 장면을 보았으면 적어 둘 탭: 멈춤에서 연 창의 탭만(화로·타이틀의 창은 멈춤이 열 곳을 바꾸지 않는다).
pub fn remembered(scene: Scene) -> Option<Tab> {
    match tab_of(scene) {
        Some((Host::Pause, tab)) => Some(tab),
        _ => None,
    }
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
            // 큰 창(0164)은 화면을 거의 다 쓰고 잔불·기술 점수를 제가 보인다(KB hud-readability: 창이 같은 정보면 밑 HUD를 그리지 않는다).
            Scene::Status { .. }
            | Scene::Equip { .. }
            | Scene::EquipPick { .. }
            | Scene::Skills { .. }
            | Scene::Codex { .. }
            | Scene::Shop
            | Scene::Settings { .. }
            | Scene::Result { .. }
            | Scene::Controls
            | Scene::Slots { .. } => Layer::Window { hud: false },
            #[cfg(feature = "dev")]
            Scene::Dev { .. } => Layer::Window { hud: false },
            #[cfg(feature = "dev")]
            Scene::Lab | Scene::LabGroup { .. } => Layer::Window { hud: true },
            Scene::Ask(_) | Scene::SkillAct => Layer::Modal,
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
    /// 맨 위 칸을 이 장면으로 바꾼다(0164 — 큰 창의 탭 넘기기: 쌓지 않으니 뒤로 한 번에 닫힌다). 커서는 처음 자리에서.
    Swap(Scene),
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
    /// 멈춤의 설정 탭 "처음부터": 쉼터 자리를 버린다.
    Restart,
    /// 멈춤의 설정 탭 "타이틀로": 마지막 쉼 뒤의 진행을 버린다.
    ToTitle,
    /// 화로 가게: 이 물건(살 수 있는 것 목록의 자리)을 산다.
    Buy(usize),
    /// 화로의 상태 탭(0104 → 0164): 이 줄(`progression::LINES`의 자리)을 한 단 올린다.
    Level(u8),
    /// 기술(0158 → 0161): 이 칸(`skills::ENTRIES`의 자리 — 스킬·큰 기술·패시브)의 다음 단을 산다·칸을 연다(기술 점수).
    Skill(u8),
    /// 기술: 잔불을 내고 쓴 점수를 모두 돌려받는다(패시브 칸까지).
    Respec,
}

/// 묻는 창의 커서 처음 자리: 버리는 일은 "취소"(1)에서 — 연타가 실행되지 않게(KB). 사기는 nail 가게처럼 "사기"(0)에서.
/// 되돌리기(0158)는 찍은 것을 버리는 일이라 취소에서.
pub fn ask_default(k: Ask) -> usize {
    match k {
        Ask::Buy(_) | Ask::Level(_) | Ask::Skill(_) => 0,
        Ask::Restart | Ask::ToTitle | Ask::Respec => 1,
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
        Ask::Buy(_) | Ask::Level(_) | Ask::Skill(_) | Ask::Respec => (Go::Back, false),
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
    /// 개발 메뉴(0075 → 0090 → 0164) — 개발판의 멈춤에서만, 조작 설정 바로 아래.
    #[cfg(feature = "dev")]
    Dev,
    /// 처음부터 · 타이틀로(0164 — 옛 멈춤 목록의 끝 두 줄): 멈춤에서만, 맨 끝. 둘 다 묻는 창을 거친다.
    Restart,
    ToTitle,
}

/// 설정 탭의 줄: 값 줄들 · 조작 설정, 멈춤에서 열었으면 끝에 [개발(개발판)] · 처음부터 · 타이틀로(0164 — 화로·타이틀에는 없다).
/// "돌아가기" 줄은 없다 — 창의 ✕·뒤로·창 밖이 닫는다. 조작 설정은 값 줄들 바로 다음(타이틀·화로에서는 맨 끝 = 위로 한 번).
pub fn settings_rows(fullscreen: bool, mouse: bool, host: Host) -> Vec<SettingsRow> {
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
    v.push(SettingsRow::Controls);
    if host == Host::Pause {
        #[cfg(feature = "dev")]
        v.push(SettingsRow::Dev);
        v.extend([SettingsRow::Restart, SettingsRow::ToTitle]);
    }
    v
}

/// 설정 탭에서 다른 창을 여는 줄 → 어디로(값 줄은 `None` — 화면이 값을 바꾼다). 버리는 일은 묻는 창을 거친다(KB).
pub fn choose_settings(row: SettingsRow) -> Option<Go> {
    match row {
        SettingsRow::Controls => Some(Go::Open(Scene::Controls)),
        #[cfg(feature = "dev")]
        SettingsRow::Dev => Some(Go::Open(Scene::Dev { tab: 0 })),
        SettingsRow::Restart => Some(Go::Open(Scene::Ask(Ask::Restart))),
        SettingsRow::ToTitle => Some(Go::Open(Scene::Ask(Ask::ToTitle))),
        SettingsRow::Language
        | SettingsRow::Fullscreen
        | SettingsRow::Shake
        | SettingsRow::Volume
        | SettingsRow::Tells
        | SettingsRow::TellTiming
        | SettingsRow::MouseAim
        | SettingsRow::CursorRest
        | SettingsRow::Reticle => None,
    }
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

/// 뒤로(B·Esc): 연 창은 닫는다(스택에서 꺼내 연 곳으로 — 큰 창의 탭은 어느 탭에서든 한 번에 닫힌다, 0164).
/// 골라야 끝나는 창(결과)·타이틀·싸움은 그대로(0066).
pub fn back(scene: Scene) -> Go {
    match scene {
        Scene::Status { .. }
        | Scene::Equip { .. }
        | Scene::EquipPick { .. }
        | Scene::Skills { .. }
        | Scene::SkillAct
        | Scene::Codex { .. }
        | Scene::Shop
        | Scene::Ask(_)
        | Scene::Controls
        | Scene::Settings { .. }
        | Scene::Slots { .. } => Go::Back,
        #[cfg(feature = "dev")]
        Scene::Dev { .. } | Scene::Lab | Scene::LabGroup { .. } => Go::Back,
        Scene::Title | Scene::Play | Scene::Ending { .. } | Scene::Result { .. } => Go::Stay,
        #[cfg(feature = "dev")]
        Scene::Sandbox => Go::Stay,
    }
}

/// 멈춤 버튼: 싸움 중이면 큰 창을 바로 열고(목록을 거치지 않는다 — 마지막에 보던 탭 `last`), 멈춤에서 연 창이면 닫는다
/// ("계속" = 닫기, 0164).
pub fn pause(scene: Scene, last: Tab) -> Go {
    match (scene, tab_of(scene)) {
        (Scene::Play, _) => Go::Open(pause_opens(last)),
        (_, Some((Host::Pause, _))) => Go::Back,
        _ => Go::Stay,
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
    fn pause_opens_the_window_and_back_closes_it_from_any_tab() {
        // 멈춤 = 큰 창을 바로(0164 — 목록이 없다). 어느 탭에서든 뒤로·멈춤 버튼 한 번에 닫힌다("계속" 줄이 없다).
        assert_eq!(
            pause(Scene::Play, Tab::Status),
            Go::Open(Scene::Status { host: Host::Pause })
        );
        for &t in tabs(Host::Pause) {
            let sc = tab_scene(Host::Pause, t);
            assert_eq!(pause(sc, Tab::Status), Go::Back, "{sc:?}");
            assert_eq!(back(sc), Go::Back, "{sc:?}");
        }
        // 화로·타이틀에서 연 창은 멈춤 버튼으로 닫는 것이 아니다(뒤로가 닫는다 — 몰이꾼은 둘 다 닫기로 읽는다).
        assert_eq!(pause(HEARTH_OPENS, Tab::Status), Go::Stay);
        assert_eq!(back(HEARTH_OPENS), Go::Back);
        assert_eq!(pause(Scene::Title, Tab::Status), Go::Stay);
        // 골라야 끝나는 창은 뒤로로 건너뛰지 않는다.
        assert_eq!(back(Scene::Result { won: false }), Go::Stay);
        assert_eq!(back(Scene::Title), Go::Stay);
        // Select는 장비 탭을 바로(0070).
        assert_eq!(SELECT_OPENS, Scene::Equip { host: Host::Pause });
    }

    /// 0164: 멈춤은 이 판에서 마지막에 보던 탭에서 다시 열린다 — 적어 두는 것은 멈춤의 탭뿐(화로에서 가게를 보았다고
    /// 멈춤이 가게에서 열리지는 않는다 — 멈춤에는 가게가 없다).
    #[test]
    fn pause_reopens_on_the_tab_last_seen_while_paused() {
        for &t in tabs(Host::Pause) {
            assert_eq!(pause_opens(t), tab_scene(Host::Pause, t));
            assert_eq!(remembered(tab_scene(Host::Pause, t)), Some(t));
            assert_eq!(pause(Scene::Play, t), Go::Open(tab_scene(Host::Pause, t)));
        }
        assert_eq!(pause_opens(Tab::Shop), Scene::Status { host: Host::Pause });
        for &t in tabs(Host::Hearth) {
            assert_eq!(remembered(tab_scene(Host::Hearth, t)), None, "{t:?}");
        }
        assert_eq!(remembered(Scene::Settings { host: Host::Title }), None);
        assert_eq!(remembered(Scene::Play), None);
        // 도감은 안의 묶음이 어디든 도감으로 적힌다.
        let codex = Scene::Codex {
            host: Host::Pause,
            tab: 1,
        };
        assert_eq!(remembered(codex), Some(Tab::Codex));
        // Select로 연 장비 탭도 멈춤의 탭이다.
        assert_eq!(remembered(SELECT_OPENS), Some(Tab::Equip));
    }

    /// 0164: 탭 줄 — 멈춤 다섯, 화로는 거기에 가게, 타이틀은 설정 하나. 탭마다 장면이 하나고 장면에서 탭이 되읽힌다.
    #[test]
    fn each_host_has_its_tab_row_and_every_tab_maps_to_one_scene() {
        use Tab::*;
        assert_eq!(tabs(Host::Pause), [Status, Equip, Skills, Codex, Settings]);
        assert_eq!(
            tabs(Host::Hearth),
            [Status, Equip, Skills, Codex, Shop, Settings]
        );
        assert_eq!(tabs(Host::Title), [Settings]);
        for host in [Host::Title, Host::Pause, Host::Hearth] {
            for (i, &t) in tabs(host).iter().enumerate() {
                let sc = tab_scene(host, t);
                assert_eq!(tab_of(sc), Some((host, t)), "{sc:?}");
                assert_eq!(tab_index(sc), Some(i));
                assert!(sc.is_menu(), "{sc:?} takes taps");
                assert_eq!(
                    sc.layer(),
                    Layer::Window { hud: false },
                    "{sc:?}: one window, always the same"
                );
                assert_eq!(focus(sc), 0);
            }
        }
        // 가게는 화로에만 선다.
        assert_eq!(tab_of(Scene::Shop), Some((Host::Hearth, Shop)));
        assert!(!tabs(Host::Pause).contains(&Shop));
        // 탭 위에 얹는 작은 창·싸움 화면은 탭이 아니다(넘기는 버튼을 먹지 않는다).
        for sc in [
            Scene::Play,
            Scene::Title,
            Scene::SkillAct,
            Scene::EquipPick { slot: 1 },
            Scene::Ask(Ask::Respec),
            Scene::Controls,
        ] {
            assert_eq!(tab_of(sc), None, "{sc:?}");
            assert_eq!(tab_step(sc, 1), Go::Stay);
            assert_eq!(tab_pick(sc, 0), Go::Stay);
        }
    }

    /// 0164: 넘기면 맨 위 칸을 바꾼다(쌓지 않는다) — 끝에서 돌고, 연 곳은 그대로 따라가고, 탭이 하나면 그대로.
    #[test]
    fn stepping_tabs_swaps_the_top_scene_wraps_and_keeps_the_host() {
        for host in [Host::Pause, Host::Hearth] {
            let list = tabs(host);
            let n = list.len();
            for (i, &t) in list.iter().enumerate() {
                let sc = tab_scene(host, t);
                let next = tab_scene(host, list[(i + 1) % n]);
                let prev = tab_scene(host, list[(i + n - 1) % n]);
                assert_eq!(tab_step(sc, 1), Go::Swap(next));
                assert_eq!(tab_step(sc, -1), Go::Swap(prev));
                // 탭 머리 누르기: 다른 탭이면 그 탭으로, 지금 탭·없는 자리면 그대로.
                assert_eq!(tab_pick(sc, i), Go::Stay);
                assert_eq!(tab_pick(sc, n), Go::Stay);
                assert_eq!(tab_pick(sc, (i + 1) % n), Go::Swap(next));
            }
        }
        // 도감은 안의 묶음(적·장신구)이 어디에 있든 같은 탭이다 — 넘기면 옆 탭으로.
        let codex = Scene::Codex {
            host: Host::Pause,
            tab: 1,
        };
        assert_eq!(tab_of(codex), Some((Host::Pause, Tab::Codex)));
        assert_eq!(
            tab_step(codex, 1),
            Go::Swap(Scene::Settings { host: Host::Pause })
        );
        assert_eq!(
            tab_step(codex, -1),
            Go::Swap(Scene::Skills { host: Host::Pause })
        );
        // 멈춤의 끝(설정)에서 오른쪽 = 처음(상태).
        assert_eq!(
            tab_step(Scene::Settings { host: Host::Pause }, 1),
            Go::Swap(Scene::Status { host: Host::Pause })
        );
        // 타이틀의 설정은 탭이 하나 — 넘길 곳이 없다.
        let only = Scene::Settings { host: Host::Title };
        assert_eq!(tab_step(only, 1), Go::Stay);
        assert_eq!(tab_step(only, -1), Go::Stay);
    }

    /// 0164: 올리기·사기·되돌리기는 화로에서만 켜진다(멈춤은 읽기·끼우기). 처음부터·타이틀로·개발은 멈춤의 설정 탭 끝에만.
    #[test]
    fn only_the_hearth_edits_and_only_pause_settings_end_the_run() {
        assert!(Host::Hearth.edits());
        assert!(!Host::Pause.edits());
        assert!(!Host::Title.edits());
        for (f, m) in [(false, false), (true, true), (true, false)] {
            let pause = settings_rows(f, m, Host::Pause);
            assert_eq!(
                pause[pause.len() - 2..],
                [SettingsRow::Restart, SettingsRow::ToTitle]
            );
            for host in [Host::Title, Host::Hearth] {
                let r = settings_rows(f, m, host);
                assert_eq!(r.last(), Some(&SettingsRow::Controls), "{host:?}");
                assert!(!r.contains(&SettingsRow::Restart));
                assert!(!r.contains(&SettingsRow::ToTitle));
                // 멈춤의 줄 = 다른 곳의 줄 + 끝 줄들(값 줄의 차례는 어디서 열어도 같다).
                assert_eq!(pause[..r.len()], r[..]);
            }
        }
        assert!(!settings_rows(false, true, Host::Title).contains(&SettingsRow::Fullscreen));
        assert!(!settings_rows(true, false, Host::Title).contains(&SettingsRow::MouseAim));
        // 버리는 일은 한 번 묻는다(KB): 처음부터·타이틀로 → 묻는 창, 답해야 판을 다시 세운다. 값 줄은 어디로도 가지 않는다.
        assert_eq!(
            choose_settings(SettingsRow::Restart),
            Some(Go::Open(Scene::Ask(Ask::Restart)))
        );
        assert_eq!(
            choose_settings(SettingsRow::ToTitle),
            Some(Go::Open(Scene::Ask(Ask::ToTitle)))
        );
        assert_eq!(
            choose_settings(SettingsRow::Controls),
            Some(Go::Open(Scene::Controls)),
            "the run is kept"
        );
        assert_eq!(choose_settings(SettingsRow::Volume), None);
        assert_eq!(choose_settings(SettingsRow::Language), None);
    }

    #[test]
    fn asks_answer_and_results_rebuild_the_stage() {
        assert_eq!(answer(Ask::Restart, true), (Go::Root(Scene::Play), true));
        assert_eq!(answer(Ask::ToTitle, true), (Go::Root(Scene::Title), true));
        assert_eq!(
            answer(Ask::Buy(2), true),
            (Go::Back, false),
            "back to the shop"
        );
        // 0104: 레벨도 가게처럼 "올리기"에서, 답하면 상태 탭으로.
        assert_eq!(answer(Ask::Level(1), true), (Go::Back, false));
        assert_eq!(ask_default(Ask::Level(0)), 0);
        // 싸움 화면만 메뉴가 아니다.
        assert!(!Scene::Play.is_menu());
        // 0158 → 0161: 기술 화면 — 단을 사는 것·칸을 여는 것은 "올리기"에서, 되돌리기는 "취소"에서. 답하면 화면으로 돌아온다.
        assert_eq!(ask_default(Ask::Skill(0)), 0);
        assert_eq!(ask_default(Ask::Respec), 1);
        assert_eq!(answer(Ask::Skill(3), true), (Go::Back, false));
        assert_eq!(answer(Ask::Respec, true), (Go::Back, false));
        // 칸의 동작 목록·후보 목록은 탭 위에 얹는 창(닫으면 그 칸으로).
        assert_eq!(Scene::SkillAct.layer(), Layer::Modal);
        assert_eq!(focus(Scene::SkillAct), 0, "the primary row");
        assert!(Scene::SkillAct.is_menu());
        assert_eq!(back(Scene::SkillAct), Go::Back);
        assert_eq!(back(Scene::EquipPick { slot: 0 }), Go::Back);
        for k in [
            Ask::Restart,
            Ask::ToTitle,
            Ask::Buy(0),
            Ask::Level(2),
            Ask::Skill(1),
            Ask::Respec,
        ] {
            assert_eq!(answer(k, false), (Go::Back, false), "cancel closes");
            assert_eq!(back(Scene::Ask(k)), Go::Back);
            assert_eq!(Scene::Ask(k).layer(), Layer::Modal);
        }
        // 커서: 버리는 일은 취소에서, 사기는 사기에서 — 여는 순간의 포커스(0066).
        assert_eq!(focus(Scene::Ask(Ask::Restart)), 1);
        assert_eq!(focus(Scene::Ask(Ask::Buy(0))), 0);
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
        // 0105: 핵 고르기는 없다 — 이겨도 져도 결과.
        assert_eq!(after_ending(true), Scene::Result { won: true });
        assert_eq!(after_ending(false), Scene::Result { won: false });
        assert!(Scene::Controls.is_menu());
    }

    #[test]
    fn layers_put_the_hud_only_under_windows_that_lack_it() {
        assert_eq!(Scene::Title.layer(), Layer::Screen);
        assert_eq!(Scene::Play.layer(), Layer::Game);
        assert_eq!(
            Scene::Shop.layer(),
            Layer::Window { hud: false },
            "shows its own embers"
        );
        #[cfg(feature = "dev")]
        assert_eq!(Scene::Lab.layer(), Layer::Window { hud: true });
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
    /// 타이틀은 설정 바로 아래가 개발 — 연구소는 개발 창 도구 탭에서만(`app/dev.rs`). 멈춤의 개발 줄은 설정 탭의 끝 묶음(0164).
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
        // 멈춤의 설정 탭: … 조작 설정 · 개발 · 처음부터 · 타이틀로(헤드리스 스크립트가 끝에서 위로 센다).
        let p = settings_rows(true, true, Host::Pause);
        assert_eq!(
            p[p.len() - 4..],
            [
                SettingsRow::Controls,
                SettingsRow::Dev,
                SettingsRow::Restart,
                SettingsRow::ToTitle
            ]
        );
        assert_eq!(
            choose_settings(SettingsRow::Dev),
            Some(Go::Open(Scene::Dev { tab: 0 }))
        );
        // 화로·타이틀의 설정에는 개발 줄이 없다(타이틀은 제 줄이 있다).
        for host in [Host::Title, Host::Hearth] {
            assert!(!settings_rows(true, true, host).contains(&SettingsRow::Dev));
        }
    }
}
