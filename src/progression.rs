//! 성장 v0.4.0 "자라는 방랑자"(0095·0096·0102): **잔불 레벨** — 생명·기력·힘 셋, 각 다섯 단(0104 G1),
//! **기술 나무** — 흐름·맞받기·자리 셋 × 다섯 칸(0106 G2), 칸의 동작은 G5.
//!
//! **0161 — 나무 하나**: 패시브 15칸은 기술 판(`skills::Board`)의 칸이 되었다 — 칸마다 기술 점수 1, 잠금은 줄(쓴 점수)뿐이다
//! (갈래마다 첫 둘은 첫 줄, 셋째·넷째는 둘째 줄, 다섯째는 셋째 줄 `CELL_ROWS`). "앞 칸을 열어야 다음 칸"과 화폐 "마디"는 걷었다 —
//! 핵과 종소리 조각은 기술 점수의 샘이다(`skills::points`). 여기에는 칸의 표·연 칸(`Tree`)·칸 동작의 값만 남는다.
//!
//! 뜻(lore CANON, `docs/lore/growth.md`): 힘을 빨아들이는 게 아니라 몸과 싸우는 법을 되찾는 것 — 숫자 성장은 여기,
//! 동작 성장은 나무. 값·단은 판정(0096 L1·L2)에서 고친다 — 표 한 곳.
use crate::i18n::Tr;
use crate::stats::Mods;

/// 레벨 줄.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Line {
    /// 버티는 힘 — 최대 체력 +10/단(`LIFE_PER_RANK`).
    Life,
    /// 움직이는 힘 — 최대 스태미나 +12/단, 3·5단에 차는 빠르기 +10%.
    Stamina,
    /// 맞서는 힘 — 자세 피해 +12%/단, 3단 강타 +10, 5단 평타 +10(`MIGHT_DMG` — 옛 +1, 0095 원칙 3 → 눈금 0158 N0).
    Might,
}

pub const LINES: [Line; 3] = [Line::Life, Line::Stamina, Line::Might];

/// 줄마다 단 수.
pub const MAX_RANK: u8 = 5;
/// 다음 레벨 값(잔불, 0109): 어느 줄이든 **총 레벨**로 — `LEVEL_BASE + LEVEL_STEP × 지금 레벨`(소울 꼴).
/// 줄마다 따로 매기면 세 줄 1단이 다 싸서 두 판에 레벨 3이 몰렸다(0095·0102의 30→120 표).
/// 한 판(다 깸) 잔불 80이면 첫 판 1레벨, 넷째 판쯤 4, 열한 판쯤 8 — 이 구역 상한 15까지 2,700(≈ 34판, 둘째 구역에서 채운다).
pub const LEVEL_BASE: u32 = 50;
pub const LEVEL_STEP: u32 = 25;
pub const STAMINA_PER_RANK: f32 = 8.0;
/// 차는 빠르기가 오르는 단(+10%씩, 곱).
pub const REGEN_RANKS: [u8; 2] = [3, 5];
pub const REGEN_STEP: f32 = 0.06;
pub const POISE_PER_RANK: f32 = 0.08;
/// 강타 덤이 붙는 단·평타 덤이 붙는 단, 그 덤(피해).
pub const HEAVY_RANK: u8 = 3;
pub const LIGHT_RANK: u8 = 5;
pub const MIGHT_DMG: i32 = 10;
/// 생명 한 단의 최대 체력·숨은 한 모금 하나의 회복(0105) — 눈금 0158 N0(옛 +1).
pub const LIFE_PER_RANK: i32 = 10;
pub const SIP_HEAL: i32 = 10;

/// 문지기 핵(0102 5절·0105): 이길 때마다 하나, 문지기마다 셋까지 — 하나가 기술 점수 1(0161, `skills::points`).
pub const CORE_CAP: u32 = 3;

/// 숨은 물건(0105)의 보정: 등잔 +1회 · 한 모금 +`SIP_HEAL` — 기록 이름(`<스테이지>.<종류>.<n>`)의 가운데 칸으로 센다.
pub fn find_mods(ids: &[String]) -> Mods {
    let count = |k: &str| {
        ids.iter()
            .filter(|id| id.split('.').nth(1) == Some(k))
            .count() as i32
    };
    Mods {
        flask: count("lamp"),
        heal: count("sip") * SIP_HEAL,
        ..Mods::default()
    }
}

/// 올린 단(기록 `lv.*`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Level {
    pub life: u8,
    pub stamina: u8,
    pub might: u8,
}

impl Level {
    pub fn rank(&self, l: Line) -> u8 {
        match l {
            Line::Life => self.life,
            Line::Stamina => self.stamina,
            Line::Might => self.might,
        }
    }

    /// 레벨 숫자 = 세 줄 단의 합(0~15).
    pub fn total(&self) -> u32 {
        LINES.iter().map(|&l| self.rank(l) as u32).sum()
    }

    /// 다음 단 값(다 올렸으면 없음) — 줄과 상관없이 총 레벨로 오른다(0109).
    pub fn next_cost(&self, l: Line) -> Option<u32> {
        (self.rank(l) < MAX_RANK).then(|| LEVEL_BASE + LEVEL_STEP * self.total())
    }

    /// 한 단 올린 사본(다 올렸으면 그대로).
    pub fn raised(&self, l: Line) -> Level {
        let mut o = *self;
        let r = match l {
            Line::Life => &mut o.life,
            Line::Stamina => &mut o.stamina,
            Line::Might => &mut o.might,
        };
        *r = (*r + 1).min(MAX_RANK);
        o
    }

    /// 잔불로 올릴 수 있는 줄이 있나(쉼터가 빛난다 — KB "쓸 곳이 보여야").
    pub fn can_raise(&self, embers: u32) -> bool {
        LINES
            .iter()
            .any(|&l| self.next_cost(l).is_some_and(|c| embers >= c))
    }

    /// 능력치 보정(장신구·핵과 같은 `Stats::compute`로).
    pub fn mods(&self) -> Mods {
        let regen = REGEN_RANKS.iter().filter(|&&r| self.stamina >= r).count() as f32;
        Mods {
            hp: self.life as i32 * LIFE_PER_RANK,
            stamina_max: STAMINA_PER_RANK * self.stamina as f32,
            stamina_regen: 1.0 + REGEN_STEP * regen,
            poise: 1.0 + POISE_PER_RANK * self.might as f32,
            heavy_dmg: (self.might >= HEAVY_RANK) as i32 * MIGHT_DMG,
            light_dmg: (self.might >= LIGHT_RANK) as i32 * MIGHT_DMG,
            ..Mods::default()
        }
    }
}

/// 나무 갈래(0102 4절 — 싸움을 대하는 태도). 화면 이름은 흐름·맞받기·자리(0161) — 스킬·큰 기술도 이 셋에 선다.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Branch {
    /// 흐름(박자)을 읽는 방랑자 — 평타 콤보·박자·딱 맞춰 떼기.
    Beat,
    /// 맞서는 방랑자 — 패링·이어 패링·모으기 패링·반격.
    Riposte,
    /// 자리(발걸음)를 찾는 방랑자 — 회피·달리기·땅 돌리기·점프·내려찍기.
    Footing,
}

pub const BRANCHES: [Branch; 3] = [Branch::Beat, Branch::Riposte, Branch::Footing];
/// 갈래마다 패시브 칸 수(칸마다 기술 점수 1). 0214: 다섯 → 일곱(넷째 줄에 둘씩 — 뒤에만 붙인다, 기록은 칸의 자리).
pub const DEPTH: u8 = 7;
/// 0214 전의 칸 수 — 옛 꼴의 기록(`tree.beat` = 앞에서부터 연 수)은 이만큼까지만 읽는다.
pub const LEGACY_DEPTH: u8 = 5;
/// 칸의 줄(0161 — `skills::ROW_NEEDS`의 자리): 갈래마다 첫 둘은 첫 줄, 셋째·넷째는 둘째 줄, 다섯째는 셋째 줄, 여섯째·일곱째는
/// 넷째 줄(0214).
pub const CELL_ROWS: [u8; DEPTH as usize] = [0, 0, 1, 1, 2, 3, 3];

/// 맺음(0214 — Keystone): 갈래마다 하나, 다섯째 줄. **하나만** 켠다(1점, 켠 뒤 화로에서 값 없이 바꾼다). 덤과 값이 함께 있는 규칙 —
/// 판정 창을 넓히는 맺음은 없다(0095 원칙 2). 기록 `tree.key` = 키 이름(바꾸지 않는다).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Key {
    /// 박자의 사람(흐름): 박자 타 피해 ×`key::MEASURE_ON`, 박자 밖의 평타 ×`key::MEASURE_OFF`.
    Measure,
    /// 벼린 손(맞받기): 패링이 주는 자세 피해 ×`key::EDGE_POISE`, 패링 창 ×`key::EDGE_WINDOW`.
    Edge,
    /// 뿌리내린 몸(자리): `key::ROOTED_STILL`초 넘게 서 있으면 받는 피해 ×`key::ROOTED_TAKEN`·밀리지 않는다, 회피 거리 ×`key::ROOTED_DODGE`.
    Rooted,
}

pub const KEYS: [Key; 3] = [Key::Measure, Key::Edge, Key::Rooted];

/// 다리 칸(0215): 맞닿은 두 갈래의 경계에 서서 둘을 잇는다 — 판의 길로는 양쪽 갈래와 이어지고, 하는 일은 두 갈래의 시너지.
/// 기록 `tree.links` = 연 다리의 키들(바꾸지 않는다).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Link {
    /// 받아치는 박(둘째 줄 · 흐름 ↔ 맞받기): 패링 반격 타가 박자 타로 친다.
    RiposteBeat,
    /// 디딘 받기(둘째 줄 · 맞받기 ↔ 자리): 패링에 성공하면 다음 회피가 기력을 쓰지 않는다.
    ParryStep,
    /// 끊는 박(넷째 줄 · 흐름 ↔ 맞받기): 패링·막아치기로 드러난(휘청인) 적에게 박자 타 자세 피해 ×`link::BREAK_POISE`.
    BreakingBeat,
    /// 뿌리로 받기(넷째 줄 · 맞받기 ↔ 자리): 엉키거나 묶인 적에게 반격 타 피해 ×`link::ROOTED_RIPOSTE`.
    RootedRiposte,
}

pub const LINKS: [Link; 4] = [
    Link::RiposteBeat,
    Link::ParryStep,
    Link::BreakingBeat,
    Link::RootedRiposte,
];

/// 결의 칸(0223 11절): 피해 타입마다 하나 — 그 타입의 스킬을 배워야 연다(판의 길 밖, 어느 갈래의 점수도 아니다). 타입마다 다른
/// **자원**을 돌려준다(숫자만 다른 넷이 아니게): 잔불 = 번짐 · 뿌리 = 기력 · 울림 = 큰 기술 게이지 · 균열 = 스킬 대기.
/// 받는 피해·패링 창·회피를 건드리지 않는다(0095 원칙 2). 기록 `tree.veins` = 연 결의 키들(바꾸지 않는다).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Vein {
    /// 옮겨붙는 불(잔불): 타는 적이 쓰러지면 남은 불이 `vein::EMBER_REACH` m 안의 가장 가까운 적 하나에 옮겨붙는다.
    Ember,
    /// 뿌리의 숨(뿌리): 묶인 적을 스킬이 아닌 타로 치면 휘두르기 한 번에 기력 +`vein::ROOT_BREATH`.
    Root,
    /// 울림 모으기(울림): 울림이 터질 때마다 큰 기술 게이지가 평타 한 번만큼 찬다(한 틱에 `vein::KNELL_TOLLS` 번까지).
    Knell,
    /// 금을 따라(균열): 금 간 적을 스킬이 아닌 타로 치면 휘두르기 한 번에 모든 스킬 대기가 `vein::RIFT_PULL`초 당겨진다.
    Rift,
}

pub const VEINS: [Vein; 4] = [Vein::Ember, Vein::Root, Vein::Knell, Vein::Rift];

impl Vein {
    pub fn index(self) -> usize {
        self as usize
    }

    /// 이 결의 피해 타입.
    pub fn kind(self) -> crate::status::Kind {
        crate::status::TYPED[self.index()]
    }

    /// 기록 값(바꾸지 않는다).
    pub fn key(self) -> &'static str {
        match self {
            Vein::Ember => "ember",
            Vein::Root => "root",
            Vein::Knell => "knell",
            Vein::Rift => "rift",
        }
    }

    pub fn by_key(k: &str) -> Option<Vein> {
        VEINS.into_iter().find(|x| x.key() == k)
    }

    /// 화면의 빛깔이 기대는 갈래(그 타입의 스킬이 많이 선 곳 — 갈래의 점수에는 세지 않는다).
    pub fn branch(self) -> Branch {
        match self {
            Vein::Rift => Branch::Beat,
            Vein::Knell => Branch::Riposte,
            Vein::Ember | Vein::Root => Branch::Footing,
        }
    }

    pub fn name(self) -> Tr {
        match self {
            Vein::Ember => |s| s.vein_ember,
            Vein::Root => |s| s.vein_root,
            Vein::Knell => |s| s.vein_knell,
            Vein::Rift => |s| s.vein_rift,
        }
    }

    pub fn desc(self) -> Tr {
        match self {
            Vein::Ember => |s| s.vein_ember_d,
            Vein::Root => |s| s.vein_root_d,
            Vein::Knell => |s| s.vein_knell_d,
            Vein::Rift => |s| s.vein_rift_d,
        }
    }
}

/// 결의 값(0223 11절 — 판정에서 고친다).
pub mod vein {
    /// 옮겨붙는 불: 닿는 거리(m).
    pub const EMBER_REACH: f32 = 4.0;
    /// 뿌리의 숨: 묶인 적을 맞힌 휘두르기 한 번에 돌아오는 기력.
    pub const ROOT_BREATH: f32 = 6.0;
    /// 울림 모으기: 한 틱에 세는 울림의 수(떼가 한꺼번에 울려도 게이지가 통째로 차지 않게).
    pub const KNELL_TOLLS: u8 = 2;
    /// 금을 따라: 금 간 적을 맞힌 휘두르기 한 번에 당겨지는 스킬 대기(초).
    pub const RIFT_PULL: f32 = 0.5;
}

impl Link {
    pub fn index(self) -> usize {
        self as usize
    }

    /// 기록 값(바꾸지 않는다).
    pub fn key(self) -> &'static str {
        match self {
            Link::RiposteBeat => "riposte_beat",
            Link::ParryStep => "parry_step",
            Link::BreakingBeat => "breaking_beat",
            Link::RootedRiposte => "rooted_riposte",
        }
    }

    pub fn by_key(k: &str) -> Option<Link> {
        LINKS.into_iter().find(|x| x.key() == k)
    }

    /// 판의 줄(`skills::ROW_NEEDS`의 자리): 둘째 줄 또는 넷째 줄.
    pub fn row(self) -> usize {
        match self {
            Link::RiposteBeat | Link::ParryStep => 1,
            Link::BreakingBeat | Link::RootedRiposte => 3,
        }
    }

    /// 잇는 두 갈래 가운데 왼쪽(`BRANCHES`의 자리) — 오른쪽은 그다음.
    pub fn left(self) -> usize {
        match self {
            Link::RiposteBeat | Link::BreakingBeat => 0,
            Link::ParryStep | Link::RootedRiposte => 1,
        }
    }

    pub fn name(self) -> Tr {
        match self {
            Link::RiposteBeat => |s| s.link_riposte_beat,
            Link::ParryStep => |s| s.link_parry_step,
            Link::BreakingBeat => |s| s.link_breaking_beat,
            Link::RootedRiposte => |s| s.link_rooted_riposte,
        }
    }

    pub fn desc(self) -> Tr {
        match self {
            Link::RiposteBeat => |s| s.link_riposte_beat_d,
            Link::ParryStep => |s| s.link_parry_step_d,
            Link::BreakingBeat => |s| s.link_breaking_beat_d,
            Link::RootedRiposte => |s| s.link_rooted_riposte_d,
        }
    }
}

/// 다리 칸의 값(0215).
pub mod link {
    /// 끊는 박: 드러난 적에게 박자 타 자세 배.
    pub const BREAK_POISE: f32 = 1.5;
    /// 뿌리로 받기: 엉키거나 묶인 적에게 반격 타 피해 배.
    pub const ROOTED_RIPOSTE: f32 = 1.5;
}

impl Key {
    pub fn branch(self) -> Branch {
        BRANCHES[self as usize]
    }

    /// 기록 값(바꾸지 않는다).
    pub fn key(self) -> &'static str {
        match self {
            Key::Measure => "measure",
            Key::Edge => "edge",
            Key::Rooted => "rooted",
        }
    }

    pub fn by_key(k: &str) -> Option<Key> {
        KEYS.into_iter().find(|x| x.key() == k)
    }

    pub fn name(self) -> Tr {
        match self {
            Key::Measure => |s| s.key_measure,
            Key::Edge => |s| s.key_edge,
            Key::Rooted => |s| s.key_rooted,
        }
    }

    pub fn desc(self) -> Tr {
        match self {
            Key::Measure => |s| s.key_measure_d,
            Key::Edge => |s| s.key_edge_d,
            Key::Rooted => |s| s.key_rooted_d,
        }
    }
}

/// 맺음의 값(0214 — 판정에서 고친다).
pub mod key {
    /// 박자의 사람: 박자 타 · 박자 밖의 평타 피해 배.
    pub const MEASURE_ON: f32 = 1.3;
    pub const MEASURE_OFF: f32 = 0.75;
    /// 벼린 손: 패링이 주는 자세 피해 배 · 패링 창 배.
    pub const EDGE_POISE: f32 = 2.0;
    pub const EDGE_WINDOW: f32 = 0.75;
    /// 뿌리내린 몸: 이만큼(초) 움직이지 않으면 서 있다 — 받는 피해 배·회피 거리 배.
    pub const ROOTED_STILL: f32 = 0.4;
    pub const ROOTED_TAKEN: f32 = 0.7;
    pub const ROOTED_DODGE: f32 = 0.65;
}

/// 나무 칸(0102 4절). 기록은 갈래마다 연 칸의 자리(1~5)라 칸 이름은 기록에 없다 — 이름을 바꿔도 연 자리는 그대로.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cell {
    /// 고른 호흡: 딱 맞춰 뗀 강타가 강타 기력을 돌려준다(숫자 칸).
    EvenBreath,
    /// 이어지는 박자: 박자 타 셋을 잇달아 맞히면 셋째 타에 작은 충격.
    FlowingBeat,
    /// 맺음타: 콤보 마지막 타에도 박자 창, 맞히면 자세 ×1.5.
    ClosingBlow,
    /// 빈틈의 박자: 헛친 적의 경직 동안 박자 타 자세 ×1.5.
    GapBeat,
    /// 끝맺음: 박자 다섯 + 마지막 타 박자 → 다음 강타가 다 모인 강타.
    FinalMeasure,
    /// 받아내기: 패링이 적 자세에 주는 몫 +50%(숫자 칸).
    Catch,
    /// 되돌려 치기: 패링 반격 타가 둘레 2m도 휘청.
    TurnItBack,
    /// 모은 맞받기: 모으기 패링 뒤 그 강타가 맞으면 자세가 곧바로 깨짐(문지기는 반).
    GatheredRiposte,
    /// 무너진 틈: 깨진 적에게 다 모은 강타 = 깨짐을 처음부터.
    OpenBreach,
    /// 맞서는 자: 이어 패링 셋 동안 못 받는 공격도 패링 = 피해 반.
    OneWhoStands,
    /// 멈추지 않는 걸음: 달리기 기력 −30%(숫자 칸).
    UnbrokenStride,
    /// 비껴서기: 등 뒤에서 끝난 회피 → 대시 공격 자세 ×2.
    Sidestep,
    /// 기우는 땅: 땅 돌리기가 멈출 때 휘청인 적이 바깥으로 1m.
    TiltingGround,
    /// 착지의 순간: 내려찍기가 0.3초 뒤 바깥 고리 한 번 더.
    LandingBeat,
    /// 길을 여는 발: 대시 공격이 맞힌 적을 지나쳐 등 뒤에 선다.
    PathOpeningStep,
    /// 이어지는 숨(0214): 박자 타 셋을 잇달아 맞히면 다음 회피가 기력을 쓰지 않는다.
    HeldBreath,
    /// 다잡은 박(0214): 박자 타가 큰 기술 게이지를 반 번 더 채운다.
    GatheringBeat,
    /// 숨 고르기(0214): 패링에 성공하면 기력 +`cell::SECOND_WIND`.
    SecondWind,
    /// 되찾는 살(0214): 패링 `cell::MEND_EVERY`번마다 체력 한 눈금.
    Mend,
    /// 디딘 발(0214): 스킬 시동 동안 맞아도 끊기지 않는다(피해는 받는다).
    Planted,
    /// 엉킨 뿌리(0214): 스킬에 맞은 적은 `cell::SNARE_SECS`초 동안 `cell::SNARE_SLOW`배로 걷는다(보스는 아니다).
    Snare,
}

/// 칸 하나의 표 줄: 칸·이름·설명·숫자 칸인가(갈래마다 하나까지 — 0095 원칙, 시험).
/// `number` = 숫자 칸(갈래마다 하나) — 0215부터 둘 가운데 하나를 고른다(`Tree::picked_b`).
pub struct CellDef {
    pub cell: Cell,
    pub name: Tr,
    pub desc: Tr,
    #[cfg_attr(not(test), allow(dead_code))]
    pub number: bool,
}

const fn c(cell: Cell, name: Tr, desc: Tr, number: bool) -> CellDef {
    CellDef {
        cell,
        name,
        desc,
        number,
    }
}

/// 나무(0102 4절 — 이름은 장신구와 겹치지 않는다, 1막이라 종·울림 말 없음, 판정 창을 넓히는 칸 없음).
pub static TREE: [[CellDef; DEPTH as usize]; 3] = [
    [
        c(
            Cell::EvenBreath,
            |s| s.cell_even_breath,
            |s| s.cell_even_breath_d,
            true,
        ),
        c(
            Cell::FlowingBeat,
            |s| s.cell_flowing_beat,
            |s| s.cell_flowing_beat_d,
            false,
        ),
        c(
            Cell::ClosingBlow,
            |s| s.cell_closing_blow,
            |s| s.cell_closing_blow_d,
            false,
        ),
        c(
            Cell::GapBeat,
            |s| s.cell_gap_beat,
            |s| s.cell_gap_beat_d,
            false,
        ),
        c(
            Cell::FinalMeasure,
            |s| s.cell_final_measure,
            |s| s.cell_final_measure_d,
            false,
        ),
        c(
            Cell::HeldBreath,
            |s| s.cell_held_breath,
            |s| s.cell_held_breath_d,
            false,
        ),
        c(
            Cell::GatheringBeat,
            |s| s.cell_gathering_beat,
            |s| s.cell_gathering_beat_d,
            false,
        ),
    ],
    [
        c(Cell::Catch, |s| s.cell_catch, |s| s.cell_catch_d, true),
        c(
            Cell::TurnItBack,
            |s| s.cell_turn_it_back,
            |s| s.cell_turn_it_back_d,
            false,
        ),
        c(
            Cell::GatheredRiposte,
            |s| s.cell_gathered,
            |s| s.cell_gathered_d,
            false,
        ),
        c(
            Cell::OpenBreach,
            |s| s.cell_open_breach,
            |s| s.cell_open_breach_d,
            false,
        ),
        c(
            Cell::OneWhoStands,
            |s| s.cell_one_who_stands,
            |s| s.cell_one_who_stands_d,
            false,
        ),
        c(
            Cell::SecondWind,
            |s| s.cell_second_wind,
            |s| s.cell_second_wind_d,
            false,
        ),
        c(Cell::Mend, |s| s.cell_mend, |s| s.cell_mend_d, false),
    ],
    [
        c(
            Cell::UnbrokenStride,
            |s| s.cell_stride,
            |s| s.cell_stride_d,
            true,
        ),
        c(
            Cell::Sidestep,
            |s| s.cell_sidestep,
            |s| s.cell_sidestep_d,
            false,
        ),
        c(
            Cell::TiltingGround,
            |s| s.cell_tilting,
            |s| s.cell_tilting_d,
            false,
        ),
        c(
            Cell::LandingBeat,
            |s| s.cell_landing,
            |s| s.cell_landing_d,
            false,
        ),
        c(
            Cell::PathOpeningStep,
            |s| s.cell_path,
            |s| s.cell_path_d,
            false,
        ),
        c(
            Cell::Planted,
            |s| s.cell_planted,
            |s| s.cell_planted_d,
            false,
        ),
        c(Cell::Snare, |s| s.cell_snare, |s| s.cell_snare_d, false),
    ],
];

impl Branch {
    /// 표의 자리(`BRANCHES`·`TREE`의 순서).
    pub fn index(self) -> usize {
        self as usize
    }

    /// 갈래 이름과 태도 한 줄(0102 4절, 화면 머리).
    pub fn name(self) -> Tr {
        match self {
            Branch::Beat => |s| s.tree_beat,
            Branch::Riposte => |s| s.tree_riposte,
            Branch::Footing => |s| s.tree_footing,
        }
    }

    pub fn motto(self) -> Tr {
        match self {
            Branch::Beat => |s| s.tree_beat_d,
            Branch::Riposte => |s| s.tree_riposte_d,
            Branch::Footing => |s| s.tree_footing_d,
        }
    }
}

/// 칸 동작의 값(0107 G5 — 0102 4절, 판정에서 고친다).
pub mod cell {
    /// 이어지는 박자: 박자 타 셋마다 둘레 반지름(m)·자세 피해.
    pub const FLOW_EVERY: u32 = 3;
    pub const FLOW_RADIUS: f32 = 1.5;
    pub const FLOW_POISE: f32 = 1.5;
    /// 맺음타: 마지막 타 뒤 박자로 이은 첫 타의 자세 배.
    pub const CLOSING_POISE: f32 = 1.3;
    /// 빈틈의 박자: 공격 뒤 굳은(되돌아가는) 적에게 박자 타 자세 배.
    pub const GAP_POISE: f32 = 1.5;
    /// 끝맺음: 박자를 이만큼 이은 뒤의 마지막 타.
    pub const FINAL_CHAIN: u32 = 5;
    /// 받아내기: 패링이 적 자세 막대를 이만큼 채운다(문지기는 반).
    pub const CATCH_POISE: f32 = 0.25;
    /// 되돌려 치기: 반격이 맞으면 둘레(m)의 다른 적도 이만큼(초) 휘청.
    pub const TURN_BACK_RADIUS: f32 = 2.0;
    pub const TURN_BACK_STAGGER: f32 = 0.35;
    /// 맞서는 자: 이어 패링이 이만큼 쌓인 동안(첫 튕김 = 0) 못 받는 공격도 막는다.
    pub const STAND_CHAIN: u32 = 2;
    /// 멈추지 않는 걸음: 달리기 기력 배.
    pub const STRIDE_RUN: f32 = 0.7;
    /// 비껴서기: 회피가 끝날 때 이 안(m)의 적 등 뒤면, 그 대시 공격 자세 배.
    pub const SIDESTEP_RANGE: f32 = 3.5;
    pub const SIDESTEP_POISE: f32 = 1.5;
    /// 기우는 땅: 땅 돌리기가 멈출 때 바깥으로 더 미는 거리(m).
    pub const TILT_PUSH: f32 = 1.0;
    /// 착지의 순간: 내려찍기 뒤 늦은 바깥 고리(초, 반지름 배).
    pub const LANDING_DELAY: f32 = 0.3;
    pub const LANDING_RANGE: f32 = 1.8;
    /// 이어지는 숨(0214): 박자 타를 이만큼 잇달아 맞히면 다음 회피가 기력을 쓰지 않는다.
    pub const HELD_BREATH_CHAIN: u32 = 3;
    /// 다잡은 박(0214): 박자 타가 게이지를 이만큼 더(한 번 휘두름의 몫에 대해).
    pub const GATHERING_BEAT: f32 = 0.5;
    /// 숨 고르기(0214): 패링 성공의 기력.
    pub const SECOND_WIND: f32 = 20.0;
    /// 되찾는 살(0214): 패링 이만큼마다 체력 한 눈금(`HP_UNIT`).
    pub const MEND_EVERY: u32 = 3;
    /// 엉킨 뿌리(0214): 스킬에 맞은 적의 걸음 배·초.
    pub const SNARE_SLOW: f32 = 0.7;
    pub const SNARE_SECS: f32 = 2.0;
    /// 숫자 칸의 ㄴ(0215): 모으는 걸음(강타를 모으는 동안 걸음 배) · 받아 넘기기(패링 뒤 초·받는 피해 배) · 가벼운 구르기(회피 기력 배).
    pub const CHARGE_WALK: f32 = 1.4;
    pub const DEFLECT_SECS: f32 = 1.0;
    pub const DEFLECT_TAKEN: f32 = 0.5;
    pub const LIGHT_ROLL: f32 = 0.8;
}

impl Cell {
    /// 숫자 칸의 ㄴ(0215 D): 이름·하는 일. ㄱ은 칸 자체의 이름·하는 일. 숫자 칸이 아니면 없다.
    pub fn pick_b(self) -> Option<(Tr, Tr)> {
        match self {
            Cell::EvenBreath => Some((|s| s.cell_even_breath_b, |s| s.cell_even_breath_b_d)),
            Cell::Catch => Some((|s| s.cell_catch_b, |s| s.cell_catch_b_d)),
            Cell::UnbrokenStride => Some((|s| s.cell_stride_b, |s| s.cell_stride_b_d)),
            _ => None,
        }
    }

    /// 표의 자리: (갈래, 깊이 0~4).
    pub fn at(self) -> (usize, usize) {
        for (b, row) in TREE.iter().enumerate() {
            if let Some(d) = row.iter().position(|def| def.cell == self) {
                return (b, d);
            }
        }
        unreachable!("TREE has every Cell")
    }

    pub fn def(self) -> &'static CellDef {
        let (b, d) = self.at();
        &TREE[b][d]
    }

    pub fn branch(self) -> Branch {
        BRANCHES[self.at().0]
    }

    /// 판의 줄(0161): 쓴 점수 합이 `skills::ROW_NEEDS[row]`에 닿아야 열 수 있다.
    pub fn row(self) -> usize {
        CELL_ROWS[self.at().1] as usize
    }
}

/// 주운 종소리 조각의 수(0105 — 기록 이름 `<스테이지>.shard.<n>`). 하나가 기술 점수 1(0161).
pub fn shards(finds: &[String]) -> u32 {
    finds
        .iter()
        .filter(|id| id.split('.').nth(1) == Some("shard"))
        .count() as u32
}

/// 갈래 하나의 칸을 다 켠 비트.
const FULL_BRANCH: u8 = (1 << DEPTH) - 1;

/// 연 패시브 칸(0161 — 칸마다 따로: 어느 칸이든 열 수 있다). 기록은 `tree.cells.<갈래>` = 연 자리들(1~5),
/// 옛 기록 `tree.beat`·`tree.riposte`·`tree.footing`(갈래마다 연 수 N)은 "앞에서부터 N칸"으로 읽는다(`from_counts`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tree {
    /// 옛 꼴: 갈래마다 **앞에서부터 연 수**. 월드 시험의 `tree_with`가 이 칸에 적는다 — 그래서 남겨 두고 `cells`와 함께 읽는다
    /// (`mask`). 새 코드는 쓰지 않는다(`with`·`open_cell`). 월드 쪽 시험을 `Tree::with`로 옮기면 걷는다.
    pub open: [u8; 3],
    /// 갈래마다 연 칸의 비트(비트 d = 깊이 d의 칸).
    pub cells: [u8; 3],
    /// 켠 맺음(0214 — 하나만, 기록 `tree.key`).
    pub key: Option<Key>,
    /// 연 다리 칸(0215 — 비트 = `Link::index`, 기록 `tree.links`).
    pub links: u8,
    /// 숫자 칸의 고르기(0215 — 갈래마다 비트 d = 깊이 d의 칸이 ㄴ, 기록 `tree.picks.<갈래>`).
    pub picks: [u8; 3],
    /// 연 결의 칸(0223 11절 — 비트 = `Vein::index`, 기록 `tree.veins`).
    pub veins: u8,
}

impl Tree {
    /// 갈래 `b`의 연 칸 비트(옛 꼴의 앞 칸들 + 칸마다 연 것).
    pub fn mask(&self, b: usize) -> u8 {
        let prefix = (1u8 << self.open[b].min(DEPTH)) - 1;
        (prefix | self.cells[b]) & FULL_BRANCH
    }

    /// 이 칸이 열렸나 — 칸 동작(0107)이 읽는다.
    pub fn has(&self, cell: Cell) -> bool {
        let (b, d) = cell.at();
        self.mask(b) & (1 << d) != 0
    }

    /// 연 칸 수(= 패시브에 쓴 기술 점수 — 맺음은 따로, `skills::Board::spent`가 센다).
    pub fn count(&self) -> u32 {
        (0..3).map(|b| self.mask(b).count_ones()).sum()
    }

    /// 이 맺음을 켰나 — 맺음 동작(0214)이 읽는다.
    pub fn keyed(&self, k: Key) -> bool {
        self.key == Some(k)
    }

    /// 이 다리 칸을 열었나(0215).
    pub fn linked(&self, l: Link) -> bool {
        self.links & (1 << l.index()) != 0
    }

    /// 다리 칸을 연다.
    pub fn open_link(&mut self, l: Link) {
        self.links |= 1 << l.index();
    }

    /// 이 결을 열었나(0223 11절) — 결의 동작이 읽는다.
    pub fn veined(&self, v: Vein) -> bool {
        self.veins & (1 << v.index()) != 0
    }

    /// 결의 칸을 연다.
    pub fn open_vein(&mut self, v: Vein) {
        self.veins |= 1 << v.index();
    }

    /// 숫자 칸을 열었고 그 고르기가 ㄴ인가(0215). 고르기가 없는 칸은 늘 거짓.
    pub fn picked_b(&self, c: Cell) -> bool {
        let (b, d) = c.at();
        c.def().number && self.has(c) && self.picks[b] & (1 << d) != 0
    }

    /// 숫자 칸을 열었고 ㄱ인가 — 옛 동작(고른 호흡·받아내기·멈추지 않는 걸음)이 읽는다.
    pub fn picked_a(&self, c: Cell) -> bool {
        self.has(c) && !self.picked_b(c)
    }

    /// 숫자 칸의 고르기를 뒤집는다(값 없이 — 열린 숫자 칸만).
    pub fn flip_cell(&mut self, c: Cell) -> bool {
        if !c.def().number || !self.has(c) {
            return false;
        }
        let (b, d) = c.at();
        self.picks[b] ^= 1 << d;
        true
    }

    /// 칸 하나를 연다(이미 열렸으면 그대로).
    pub fn open_cell(&mut self, cell: Cell) {
        let (b, d) = cell.at();
        self.cells[b] |= 1 << d;
    }

    /// 이 칸들만 연 나무(시험·월드 시험이 옮겨 올 자리 — 게임은 `open_cell`로 연다).
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn with(cells: &[Cell]) -> Tree {
        let mut t = Tree::default();
        for &c in cells {
            t.open_cell(c);
        }
        t
    }

    /// 옛 기록의 꼴(갈래마다 연 수) → 앞에서부터 그만큼 연 나무. 다섯을 넘는 수는 다섯.
    pub fn from_counts(n: [u8; 3]) -> Tree {
        Tree {
            open: [0; 3],
            cells: n.map(|n| (1u8 << n.min(DEPTH)) - 1),
            key: None,
            links: 0,
            picks: [0; 3],
            veins: 0,
        }
    }

    /// 모든 칸을 다 연 나무(개발 창 "기술 다 올리기"·시험) — 다리 칸까지(0215), 숫자 칸은 I. 맺음은 켜지 않는다(하나만이라
    /// 고를 것이 없다).
    #[cfg(any(test, feature = "dev"))]
    pub fn full() -> Tree {
        Tree {
            links: (1 << LINKS.len()) - 1,
            veins: (1 << VEINS.len()) - 1,
            ..Tree::from_counts([DEPTH; 3])
        }
    }

    /// 갈래 `b`에서 앞에서부터 이어 연 칸 수 — 옛 키(`tree.beat` …)에 적는 값(옛 판이 읽어도 가진 것보다 많이 열리지 않는다).
    pub fn prefix(&self, b: usize) -> u8 {
        self.mask(b).trailing_ones() as u8
    }

    /// 기록에서 읽은 것을 한 꼴로: 옛 칸(`open`)을 비트로 옮기고 범위 밖 비트를 지운다.
    pub fn sane(self) -> Tree {
        let cells = [self.mask(0), self.mask(1), self.mask(2)];
        Tree {
            open: [0; 3],
            cells,
            key: self.key,
            links: self.links & ((1 << LINKS.len()) - 1),
            veins: self.veins & ((1 << VEINS.len()) - 1),
            // 고르기는 열린 숫자 칸에만.
            picks: [0, 1, 2].map(|b| {
                let numbers = TREE[b]
                    .iter()
                    .enumerate()
                    .filter(|(_, d)| d.number)
                    .fold(0u8, |m, (i, _)| m | 1 << i);
                self.picks[b] & cells[b] & numbers
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::Stats;

    #[test]
    fn levels_cost_by_total_level_stop_at_five_and_sum_to_the_level_number() {
        let mut lv = Level::default();
        let mut paid = 0;
        for l in LINES {
            while let Some(c) = lv.next_cost(l) {
                assert_eq!(c, LEVEL_BASE + LEVEL_STEP * lv.total());
                paid += c;
                lv = lv.raised(l);
            }
            assert_eq!(lv.rank(l), MAX_RANK);
            assert_eq!(lv.raised(l), lv, "a full line stays full");
        }
        assert_eq!(paid, 3_375, "0123: 50 + 25 × level, levels 0..15");
        assert_eq!(lv.total(), 15);
        assert!(!lv.can_raise(u32::MAX));
        assert!(Level::default().can_raise(LEVEL_BASE));
        assert!(!Level::default().can_raise(LEVEL_BASE - 1));
    }

    /// 0109: 값은 줄이 아니라 총 레벨을 본다 — 다른 줄의 1단도 싸지 않다(세 줄 1단이 몰리던 구멍).
    #[test]
    fn a_fresh_line_costs_as_much_as_the_level_says() {
        let lv = Level {
            life: 2,
            ..Level::default()
        };
        assert_eq!(lv.next_cost(Line::Might), lv.next_cost(Line::Life));
        assert_eq!(lv.next_cost(Line::Might), Some(LEVEL_BASE + LEVEL_STEP * 2));
    }

    /// 0109 → 0123 빠르기: 한 판(다 깸) 잔불 약 60 — 첫 판에 레벨 1, 두 판에도 1, 넷째 판에 3(덜 후하게).
    #[test]
    fn a_full_clear_buys_about_one_level_early_on() {
        let levels_after = |clears: u32| {
            let (mut lv, mut purse) = (Level::default(), 60 * clears);
            while let Some(c) = lv.next_cost(Line::Life).or(lv.next_cost(Line::Might)) {
                if purse < c {
                    break;
                }
                purse -= c;
                lv = if lv.life < MAX_RANK {
                    lv.raised(Line::Life)
                } else {
                    lv.raised(Line::Might)
                };
            }
            lv.total()
        };
        assert_eq!(levels_after(1), 1);
        assert_eq!(levels_after(2), 1);
        assert_eq!(levels_after(4), 3);
    }

    #[test]
    fn each_line_feeds_its_own_stats_and_might_keeps_damage_whole() {
        let base = Stats::default();
        let life = Stats::compute(&[Level {
            life: 5,
            ..Level::default()
        }
        .mods()]);
        assert_eq!(life.max_hp, base.max_hp + 50);
        let st = |r| {
            Stats::compute(&[Level {
                stamina: r,
                ..Level::default()
            }
            .mods()])
        };
        assert_eq!(st(1).stamina_max, 8.0);
        assert_eq!(st(2).stamina_regen, 1.0, "regen waits for rank 3");
        assert!((st(3).stamina_regen - 1.06).abs() < 1e-6);
        assert!((st(5).stamina_regen - 1.12).abs() < 1e-6);
        assert_eq!(st(5).stamina_max, 40.0);
        let mt = |r| {
            Stats::compute(&[Level {
                might: r,
                ..Level::default()
            }
            .mods()])
        };
        assert!((mt(1).poise - 1.08).abs() < 1e-6);
        assert_eq!((mt(2).heavy_dmg, mt(2).light_dmg), (0, 0));
        assert_eq!((mt(3).heavy_dmg, mt(3).light_dmg), (10, 0));
        assert_eq!((mt(5).heavy_dmg, mt(5).light_dmg), (10, 10));
        // 레벨은 판정 창을 건드리지 않는다(0095 원칙 2, 0102).
        for r in 0..=MAX_RANK {
            let lv = Level {
                life: r,
                stamina: r,
                might: r,
            };
            let s = Stats::compute(&[lv.mods()]);
            assert_eq!(s.parry, 0.0);
            assert_eq!(s.atk_speed, 1.0);
        }
    }

    #[test]
    fn the_tree_has_three_branches_of_five_one_number_cell_each_and_no_charm_names() {
        let mut seen = Vec::new();
        for row in &TREE {
            assert_eq!(
                row.iter().filter(|d| d.number).count(),
                1,
                "one number cell per branch"
            );
            for d in row {
                assert!(!seen.contains(&d.cell), "{:?} twice", d.cell);
                seen.push(d.cell);
            }
        }
        assert_eq!(seen.len(), 3 * DEPTH as usize);
        // 칸 이름은 장신구 이름과 겹치지 않는다(0102 — "가벼운 발").
        for lang in crate::i18n::Lang::ALL {
            let s = lang.strings();
            let charms: Vec<&str> = crate::charms::TABLE.iter().map(|d| (d.name)(s)).collect();
            for row in &TREE {
                for d in row {
                    assert!(
                        !charms.contains(&(d.name)(s)),
                        "{} is a charm name",
                        (d.name)(s)
                    );
                }
            }
        }
    }

    /// 0161: 패시브 칸은 칸마다 따로 열린다 — 앞 칸을 열지 않아도 뒤 칸이 열리고(순서 규칙 없음), 연 칸 수가 곧 쓴 점수다.
    #[test]
    fn any_cell_opens_on_its_own_and_the_open_count_is_the_points_spent() {
        let mut t = Tree::default();
        assert_eq!(t.count(), 0);
        t.open_cell(Cell::LandingBeat);
        assert!(t.has(Cell::LandingBeat));
        assert!(
            !t.has(Cell::UnbrokenStride) && !t.has(Cell::TiltingGround),
            "the cells before it stay shut"
        );
        t.open_cell(Cell::LandingBeat);
        assert_eq!(t.count(), 1, "opening twice is one cell");
        t.open_cell(Cell::EvenBreath);
        t.open_cell(Cell::OneWhoStands);
        assert_eq!(t.count(), 3);
        assert_eq!(
            t,
            Tree::with(&[Cell::EvenBreath, Cell::OneWhoStands, Cell::LandingBeat])
        );
        assert_eq!(Tree::full().count(), 3 * DEPTH as u32);
        for row in &TREE {
            for d in row {
                assert!(Tree::full().has(d.cell));
                assert!(Tree::with(&[d.cell]).has(d.cell));
                assert_eq!(Tree::with(&[d.cell]).count(), 1);
            }
        }
    }

    /// 0161: 옛 기록의 "갈래마다 연 수 N"은 앞에서부터 N칸 — 옛 꼴의 칸(`open`, 월드 시험이 쓴다)과 새 비트가 같은 나무로 읽힌다.
    #[test]
    fn an_old_count_reads_as_the_first_n_cells() {
        let t = Tree::from_counts([2, 0, 9]);
        assert!(t.has(Cell::EvenBreath) && t.has(Cell::FlowingBeat));
        assert!(!t.has(Cell::ClosingBlow));
        assert!(!t.has(Cell::Catch));
        assert!(t.has(Cell::Snare), "a count past the depth is the depth");
        assert_eq!(t.count(), 2 + DEPTH as u32);
        assert_eq!([t.prefix(0), t.prefix(1), t.prefix(2)], [2, 0, DEPTH]);
        // 옛 꼴로 적은 나무(월드 시험의 `tree_with`)도 같은 칸을 가진다.
        let legacy = Tree {
            open: [2, 0, DEPTH],
            cells: [0; 3],
            key: None,
            links: 0,
            picks: [0; 3],
            veins: 0,
        };
        assert_eq!(legacy.sane(), t);
        assert_eq!(legacy.count(), 2 + DEPTH as u32);
        // 옛 판의 수(다섯까지)는 그 다섯 칸 — 0214의 넷째 줄 칸은 열리지 않는다.
        let old = Tree::from_counts([5, 0, 0]);
        assert!(old.has(Cell::FinalMeasure) && !old.has(Cell::HeldBreath));
        assert!(legacy.has(Cell::FlowingBeat) && !legacy.has(Cell::ClosingBlow));
        // 사이가 빈 나무의 "앞에서부터 이어 연 수"는 빈 칸 앞까지.
        let gap = Tree::with(&[Cell::EvenBreath, Cell::ClosingBlow]);
        assert_eq!(gap.prefix(0), 1);
        // 범위 밖 비트는 지워진다.
        let odd = Tree {
            open: [0; 3],
            cells: [0xff, 0, 0],
            key: None,
            links: 0,
            picks: [0; 3],
            veins: 0,
        };
        assert_eq!(odd.count(), DEPTH as u32);
        assert_eq!(odd.sane().cells[0], 0b111_1111);
    }

    /// 0161 "갈래와 줄": 갈래마다 첫 둘은 첫 줄, 셋째·넷째는 둘째 줄, 다섯째는 셋째 줄 — 0214: 여섯째·일곱째는 넷째 줄.
    /// 맺음은 갈래마다 하나, 키는 서로 다르고 되읽힌다.
    #[test]
    fn cells_sit_two_two_one_two_on_the_four_rows() {
        for k in KEYS {
            assert_eq!(Key::by_key(k.key()), Some(k));
            assert_eq!(KEYS.iter().filter(|x| x.branch() == k.branch()).count(), 1);
        }
        for (b, row) in TREE.iter().enumerate() {
            let rows: Vec<usize> = row.iter().map(|d| d.cell.row()).collect();
            assert_eq!(rows, [0, 0, 1, 1, 2, 3, 3]);
            for (d, def) in row.iter().enumerate() {
                assert_eq!(def.cell.at(), (b, d));
                assert_eq!(def.cell.branch(), BRANCHES[b]);
                assert_eq!(def.cell.def().cell, def.cell);
            }
            assert_eq!(BRANCHES[b].index(), b);
        }
        assert_eq!(
            shards(&["sanctuary.shard.0".into(), "sanctuary.lamp.0".into()]),
            1
        );
    }

    #[test]
    fn finds_count_by_kind_whatever_the_stage() {
        let ids = vec![
            "sanctuary.lamp.0".to_string(),
            "sanctuary.sip.0".to_string(),
            "hamlet.sip.0".to_string(),
        ];
        let s = Stats::compute(&[find_mods(&ids)]);
        assert_eq!(s.flask, Stats::default().flask + 1);
        assert_eq!(s.heal, 20);
        assert_eq!(find_mods(&[]).flask, 0);
    }

    #[test]
    fn the_shell_charm_still_counts_at_full_life() {
        // 0104: 생명 5단 + 껍질 = 상한 안.
        let full = Level {
            life: MAX_RANK,
            ..Level::default()
        };
        let shell = crate::charms::def(crate::charms::Id::Shell).mods;
        let a = Stats::compute(&[full.mods()]);
        let b = Stats::compute(&[full.mods(), shell]);
        assert_eq!((a.max_hp, b.max_hp), (100, 110));
    }
}
