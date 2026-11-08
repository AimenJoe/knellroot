//! 성장 v0.4.0 "자라는 방랑자"(0095·0096·0102): **잔불 레벨** — 생명·기력·힘 셋, 각 다섯 단(0104 G1),
//! **기술 나무** — 박자·맞받기·발걸음 셋 × 다섯 칸, 점수 = 문지기 핵 + 종소리 조각(0106 G2). 칸의 동작은 G5.
//!
//! 뜻(lore CANON, `docs/lore/growth.md`): 힘을 빨아들이는 게 아니라 몸과 싸우는 법을 되찾는 것 — 숫자 성장은 여기,
//! 동작 성장은 나무. 값·단은 판정(0096 L1·L2)에서 고친다 — 표 한 곳.
use crate::i18n::Tr;
use crate::stats::Mods;

/// 레벨 줄.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Line {
    /// 버티는 힘 — 최대 체력 +1/단.
    Life,
    /// 움직이는 힘 — 최대 스태미나 +12/단, 3·5단에 차는 빠르기 +10%.
    Stamina,
    /// 맞서는 힘 — 자세 피해 +12%/단, 3단 강타 +1, 5단 평타 +1(정수 피해를 지킨다, 0095 원칙 3).
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
/// 강타 +1이 붙는 단·평타 +1이 붙는 단.
pub const HEAVY_RANK: u8 = 3;
pub const LIGHT_RANK: u8 = 5;

/// 문지기 핵 점수(0102 5절·0105): 이길 때마다 1점, 문지기마다 셋까지 — 기술 나무(G2) 점수의 한 몫(나머지는 종소리 조각).
pub const CORE_CAP: u32 = 3;

/// 숨은 물건(0105)의 보정: 등잔 +1회 · 한 모금 +1 — 기록 이름(`<스테이지>.<종류>.<n>`)의 가운데 칸으로 센다.
pub fn find_mods(ids: &[String]) -> Mods {
    let count = |k: &str| {
        ids.iter()
            .filter(|id| id.split('.').nth(1) == Some(k))
            .count() as i32
    };
    Mods {
        flask: count("lamp"),
        heal: count("sip"),
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
            hp: self.life as i32,
            stamina_max: STAMINA_PER_RANK * self.stamina as f32,
            stamina_regen: 1.0 + REGEN_STEP * regen,
            poise: 1.0 + POISE_PER_RANK * self.might as f32,
            heavy_dmg: (self.might >= HEAVY_RANK) as i32,
            light_dmg: (self.might >= LIGHT_RANK) as i32,
            ..Mods::default()
        }
    }
}

/// 나무 갈래(0102 4절 — 싸움을 대하는 태도).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Branch {
    /// 흐름을 읽는 방랑자 — 평타 콤보·박자·딱 맞춰 떼기.
    Beat,
    /// 맞서는 방랑자 — 패링·이어 패링·모으기 패링·반격.
    Riposte,
    /// 자리를 찾는 방랑자 — 회피·달리기·땅 돌리기·점프·내려찍기.
    Footing,
}

pub const BRANCHES: [Branch; 3] = [Branch::Beat, Branch::Riposte, Branch::Footing];
/// 갈래마다 칸 수(앞 칸을 열어야 다음 칸, 칸마다 1점).
pub const DEPTH: u8 = 5;

/// 나무 칸(0102 4절). 기록은 갈래마다 연 칸 수라 칸 이름은 기록에 없다 — 이름·순서를 바꿔도 연 수는 그대로.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cell {
    /// 고른 호흡: 딱 맞춰 뗀 강타가 강타 기력을 돌려준다(숫자 칸).
    EvenBreath,
    /// 이어지는 박자: 박자 타 셋을 잇달아 맞히면 셋째 타에 작은 충격.
    FlowingBeat,
    /// 맺음타: 콤보 마지막 타에도 박자 창, 맞히면 자세 ×1.5.
    ClosingBlow,
    /// 빈틈의 박자: 헛친 적의 경직 동안 박자 타 자세 ×2.
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
}

/// 칸 하나의 표 줄: 칸·이름·설명·숫자 칸인가(갈래마다 하나까지 — 0095 원칙, 시험).
/// `number`는 표 규칙 시험만 읽는다(갈래마다 숫자 칸 하나).
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
    ],
];

impl Branch {
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
}

/// 나무 점수(0102 5절): 문지기 핵(이길 때마다·셋까지) + 주운 종소리 조각.
pub fn tree_points(cores: u32, finds: &[String]) -> u32 {
    cores
        + finds
            .iter()
            .filter(|id| id.split('.').nth(1) == Some("shard"))
            .count() as u32
}

/// 연 칸(기록 `tree.beat`·`tree.riposte`·`tree.footing` = 갈래마다 연 수)과 목표 핀(`tree.pin` = "갈래.깊이").
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tree {
    pub open: [u8; 3],
    pub pin: Option<(u8, u8)>,
}

/// 칸 하나의 지금 모양(화면 밝기 네 단 — KB `meta-progression-unlock-planning`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CellState {
    Open,
    /// 다음 칸이고 점수가 있다.
    Ready,
    /// 다음 칸인데 점수가 모자란다.
    Short,
    /// 앞 칸을 먼저 — 거기까지 n점.
    Far(u32),
}

impl Tree {
    /// 쓴 점수.
    pub fn spent(&self) -> u32 {
        self.open.iter().map(|&n| n as u32).sum()
    }

    /// 남은 점수.
    pub fn left(&self, points: u32) -> u32 {
        points.saturating_sub(self.spent())
    }

    /// 이 칸이 열렸나 — 칸 동작(0107)이 읽는다.
    pub fn has(&self, cell: Cell) -> bool {
        TREE.iter().enumerate().any(|(b, row)| {
            row.iter()
                .take(self.open[b] as usize)
                .any(|d| d.cell == cell)
        })
    }

    pub fn state(&self, b: usize, d: usize, points: u32) -> CellState {
        let open = self.open[b] as usize;
        if d < open {
            CellState::Open
        } else if d == open {
            if self.left(points) >= 1 {
                CellState::Ready
            } else {
                CellState::Short
            }
        } else {
            CellState::Far((d - open + 1) as u32)
        }
    }

    /// 다음 칸을 연다(점수가 있고 갈래가 덜 찼을 때). 그 칸이 목표였으면 핀을 푼다.
    pub fn open_next(&mut self, b: usize, points: u32) -> bool {
        if self.open[b] >= DEPTH || self.left(points) == 0 {
            return false;
        }
        self.open[b] += 1;
        if self
            .pin
            .is_some_and(|(pb, pd)| pb as usize == b && pd < self.open[b])
        {
            self.pin = None;
        }
        true
    }

    /// 목표까지 남은 칸 수(핀이 있고 아직 안 열렸을 때).
    pub fn cells_to_pin(&self) -> Option<u32> {
        let (b, d) = self.pin?;
        let open = self.open[b as usize];
        (d >= open).then(|| (d - open + 1) as u32)
    }

    /// 열 수 있는 칸이 있나(쉼터가 빛난다).
    pub fn can_open(&self, points: u32) -> bool {
        self.left(points) >= 1 && self.open.iter().any(|&n| n < DEPTH)
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
        assert_eq!(life.max_hp, base.max_hp + 5);
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
        assert_eq!((mt(3).heavy_dmg, mt(3).light_dmg), (1, 0));
        assert_eq!((mt(5).heavy_dmg, mt(5).light_dmg), (1, 1));
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
        assert_eq!(seen.len(), 15);
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

    #[test]
    fn cells_open_in_order_one_point_each_and_the_pin_counts_down() {
        let mut t = Tree::default();
        assert_eq!(t.state(0, 0, 0), CellState::Short);
        assert_eq!(t.state(0, 0, 1), CellState::Ready);
        assert_eq!(t.state(0, 3, 1), CellState::Far(4));
        t.pin = Some((2, 2));
        assert_eq!(t.cells_to_pin(), Some(3));
        assert!(t.open_next(2, 6));
        assert_eq!(t.cells_to_pin(), Some(2));
        assert!(t.open_next(2, 6) && t.open_next(2, 6));
        assert_eq!(t.pin, None, "reaching the goal clears it");
        assert!(t.has(Cell::TiltingGround));
        assert!(!t.has(Cell::LandingBeat));
        assert!(t.open_next(0, 6) && t.open_next(0, 6) && t.open_next(0, 6));
        assert_eq!(t.left(6), 0);
        assert!(!t.open_next(1, 6), "no points left");
        assert!(!t.can_open(6));
        assert_eq!(
            tree_points(3, &["sanctuary.shard.0".into(), "sanctuary.lamp.0".into()]),
            4
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
        assert_eq!(s.heal, 2);
        assert_eq!(find_mods(&[]).flask, 0);
    }

    #[test]
    fn the_shell_charm_still_counts_at_full_life() {
        // 0104: 생명 5단 + 껍질 = 상한 안.
        let full = Level {
            life: MAX_RANK,
            ..Level::default()
        };
        let shell = crate::stats::Mods {
            hp: 1,
            ..Mods::default()
        };
        let a = Stats::compute(&[full.mods()]);
        let b = Stats::compute(&[full.mods(), shell]);
        assert_eq!(b.max_hp, a.max_hp + 1);
    }
}
