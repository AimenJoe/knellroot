//! 스킬 트리의 칸 `Entry` · 차례 `ENTRIES` · 줄이 열리는 점수 · 가로 자리(0161 · 0214 · 0215).
use super::*;
use crate::progression::{Branch, Cell, Key, Link, Vein, DEPTH, KEYS, LINKS, VEINS};

/// 줄이 열리는 때(쓴 점수 합): 첫 줄 · 둘째 줄 · 큰 기술 줄(0158 라) · 넷째 줄(새 스킬·패시브) · 맺음 줄(0214 — Diablo IV의
/// 묶음처럼 깊은 줄일수록 더 써야 열린다. 있는 줄의 열림은 그대로 — 이미 산 칸이 잠기지 않게).
pub const ROW_NEEDS: [u32; 6] = [0, 4, 8, 14, 22, 0];
/// 맺음의 줄(`ROW_NEEDS`의 자리).
pub const KEY_ROW: usize = 4;
/// 결의 줄(0223 11절 — 판의 맨 아래 띠): 쓴 점수로 열리지 않는다(`ROW_NEEDS` 0) — 그 타입의 스킬을 배웠나만 본다
/// (`Board::reachable`). 가로 자리는 판의 폭을 넷으로 나눈 가운데들.
pub const VEIN_ROW: usize = 5;
pub const LANE_VEIN: [f32; 8] = [17.0, 51.0, 84.0, 118.0, 152.0, 186.0, 219.0, 253.0];
/// 맺음을 켜는 데 그 갈래에 써 둔 점수(0215 A — 갈래의 스킬 단 + 큰 기술 단 + 패시브 칸, 다리 칸은 어느 갈래에도 세지 않는다).
pub const KEY_BRANCH: u32 = 10;
/// 판의 가로 자리(0215 B — 화면의 칸 자리 `app::skills::spots`도 이것을 쓴다): 갈래 폭 안에서 칸 둘이면 `LANE_2`, 하나면
/// `LANE_1`, 큰 기술 줄은 `LANE_ULT`(큰 기술 · 패시브). 다리 칸은 두 갈래의 경계(`LANE_W`의 배수).
pub const LANE_W: f32 = 90.0;
pub const LANE_2: [f32; 2] = [23.0, 67.0];
pub const LANE_1: f32 = 45.0;
pub const LANE_ULT: [f32; 2] = [30.0, 66.0];
/// 이어지는 가로 거리: 갈래 안의 왼쪽(23)·가운데(45)·오른쪽(67) 가운데 왼쪽↔오른쪽(44)만 안 이어진다.
pub const LANE_DX: f32 = 30.0;

/// 판의 칸: 스킬 하나 · 큰 기술 하나 · 패시브 칸 하나(0161) · 맺음 하나(0214 — 셋 가운데 하나만 켠다).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Entry {
    Skill(Id),
    Ult(Ult),
    Cell(Cell),
    Key(Key),
    /// 다리 칸(0215 C — 두 갈래의 경계, 시너지).
    Link(Link),
    /// 결의 칸(0223 11절 — 피해 타입마다 하나).
    Vein(Vein),
}

/// 판의 칸 수: 스킬 + 큰 기술 + 패시브(갈래마다 `DEPTH`) + 맺음 셋 + 다리 넷.
pub const NE: usize = N + NU + 3 * DEPTH as usize + KEYS.len() + LINKS.len() + VEINS.len();

/// 판의 칸 순서(0161 "갈래와 줄" — 화면의 차례이기도 하다): 줄(처음부터 · 쓴 점수 4 · 쓴 점수 8)마다 갈래(흐름 · 맞받기 · 자리)
/// 순으로, 갈래 안에서는 스킬(큰 기술) 먼저·패시브 다음. 대기 칸의 자리(`IDS`)와는 따로다. 묻는 창 `Ask::Skill`이 이 자리를 든다.
pub const ENTRIES: [Entry; NE] = [
    // 1 — 처음부터
    Entry::Skill(Id::Whirl),
    Entry::Skill(Id::Cut),
    Entry::Cell(Cell::EvenBreath),
    Entry::Cell(Cell::FlowingBeat),
    Entry::Skill(Id::Brace),
    Entry::Cell(Cell::Catch),
    Entry::Cell(Cell::TurnItBack),
    Entry::Skill(Id::Gather),
    Entry::Cell(Cell::UnbrokenStride),
    Entry::Cell(Cell::Sidestep),
    // 0270 C3 — 무기 판의 첫 줄(옛 판의 자리는 갈래의 첫 줄 끝)
    Entry::Skill(Id::Stitch),
    Entry::Skill(Id::Toll),
    Entry::Skill(Id::Swat),
    Entry::Skill(Id::Flare),
    Entry::Skill(Id::Snare),
    // 0272 — 쇠뇌 판의 충격 갈래 첫 줄
    Entry::Skill(Id::Fuse),
    // 0272 2단계 — 창 판의 첫 줄(가운데 · 뿌리 갈래)
    Entry::Skill(Id::Skewer),
    Entry::Skill(Id::Pin),
    // 0274 — 한손검 판의 충격 갈래 첫 줄
    Entry::Skill(Id::Pommel),
    // 0274 ④ — 대검 판의 잔불 갈래 첫 줄
    Entry::Skill(Id::Sear),
    // 0274 ⑤ — 낫 판의 뿌리 갈래 첫 줄
    Entry::Skill(Id::Reap),
    // 0274 ⑥ — 완드 판의 균열 갈래 첫 줄
    Entry::Skill(Id::Hex),
    // 2 — 쓴 점수 4
    Entry::Skill(Id::Quake),
    Entry::Cell(Cell::ClosingBlow),
    Entry::Cell(Cell::GapBeat),
    Entry::Skill(Id::Echo),
    Entry::Cell(Cell::GatheredRiposte),
    Entry::Cell(Cell::OpenBreach),
    Entry::Skill(Id::Throw),
    Entry::Skill(Id::Embers),
    Entry::Cell(Cell::TiltingGround),
    Entry::Cell(Cell::LandingBeat),
    Entry::Link(Link::RiposteBeat),
    Entry::Link(Link::ParryStep),
    // 0270 C3 — 무기 판의 둘째 줄
    Entry::Skill(Id::Ram),
    Entry::Skill(Id::Ripple),
    Entry::Skill(Id::Snow),
    // 0272 — 쇠뇌 판의 충격 갈래 둘째 줄
    Entry::Skill(Id::Mine),
    // 0272 2단계 — 창 판의 둘째 줄(가운데)
    Entry::Skill(Id::Pole),
    // 0274 — 한손검 판의 둘째 줄(충격 · 흡수)
    Entry::Skill(Id::Shove),
    Entry::Skill(Id::Rally),
    // 0274 ③ — 쌍단검 판의 둘째 줄(가운데)
    Entry::Skill(Id::Slip),
    // 0274 ④ — 대검 판의 둘째 줄(균열)
    Entry::Skill(Id::Sunder),
    // 0274 ⑤ — 낫 판의 둘째 줄(흡수)
    Entry::Skill(Id::Harvest),
    // 0274 ⑥ — 완드 판의 둘째 줄(균열)
    Entry::Skill(Id::Blight),
    // 0280 — 빈 노드 넷: 창(날카로움) · 대검(잔불) · 낫(가운데) · 완드(가운데)의 둘째 줄
    Entry::Skill(Id::Withdraw),
    Entry::Skill(Id::Plunge),
    Entry::Skill(Id::Boomerang),
    Entry::Skill(Id::Word),
    // 3 — 쓴 점수 8
    Entry::Ult(Ult::Flurry),
    Entry::Cell(Cell::FinalMeasure),
    Entry::Ult(Ult::Line),
    Entry::Cell(Cell::OneWhoStands),
    Entry::Ult(Ult::Upend),
    Entry::Cell(Cell::PathOpeningStep),
    // 0270 C3 — 무기 판마다 큰 기술 하나: 방패 판·지팡이 판
    Entry::Ult(Ult::Bell),
    Entry::Ult(Ult::Blizzard),
    // 0272 — 활 판의 큰 기술
    Entry::Ult(Ult::Rain),
    // 0272 2단계 — 창 판의 큰 기술
    Entry::Ult(Ult::Gale),
    // 0274 ③ — 쌍단검 판의 큰 기술
    Entry::Ult(Ult::Ambush),
    // 0274 ④ — 대검 판의 큰 기술
    Entry::Ult(Ult::Tempest),
    // 0274 ⑤ — 낫 판의 큰 기술
    Entry::Ult(Ult::Grove),
    // 0274 ⑥ — 완드 판의 큰 기술
    Entry::Ult(Ult::Price),
    // 4 — 쓴 점수 14(0214)
    Entry::Skill(Id::Swallow),
    Entry::Cell(Cell::HeldBreath),
    Entry::Cell(Cell::GatheringBeat),
    Entry::Skill(Id::Bash),
    Entry::Cell(Cell::SecondWind),
    Entry::Cell(Cell::Mend),
    Entry::Skill(Id::Roots),
    Entry::Cell(Cell::Planted),
    Entry::Cell(Cell::Snare),
    Entry::Link(Link::BreakingBeat),
    Entry::Link(Link::RootedRiposte),
    // 5 — 쓴 점수 22: 맺음(하나만)
    Entry::Key(Key::Measure),
    Entry::Key(Key::Edge),
    Entry::Key(Key::Rooted),
    // 결(0223 11절) — 그 타입의 스킬을 배우면
    Entry::Vein(Vein::Ember),
    Entry::Vein(Vein::Root),
    Entry::Vein(Vein::Knell),
    Entry::Vein(Vein::Rift),
    // 0270 C3: 새 타입 넷의 결
    Entry::Vein(Vein::Shock),
    Entry::Vein(Vein::Sharp),
    Entry::Vein(Vein::Frost),
    Entry::Vein(Vein::Drain),
];

impl Entry {
    /// 끝 단(패시브 칸은 열림 = 1).
    pub fn max(self) -> u8 {
        match self {
            Entry::Skill(_) => MAX_RANK,
            Entry::Ult(_) => ULT_MAX_RANK,
            Entry::Cell(_) | Entry::Key(_) | Entry::Link(_) | Entry::Vein(_) => 1,
        }
    }

    pub fn row(self) -> usize {
        match self {
            Entry::Skill(id) => id.def().row as usize,
            Entry::Ult(_) => ULT_ROW as usize,
            Entry::Cell(c) => c.row(),
            Entry::Key(_) => KEY_ROW,
            Entry::Link(l) => l.row(),
            Entry::Vein(_) => VEIN_ROW,
        }
    }

    /// 갈래 — 다리 칸은 잇는 둘 가운데 왼쪽(화면의 빛깔만 — 갈래의 점수 `Board::branch_spent`에는 세지 않는다).
    pub fn branch(self) -> Branch {
        match self {
            Entry::Skill(id) => id.def().branch,
            Entry::Ult(u) => u.branch(),
            Entry::Cell(c) => c.branch(),
            Entry::Key(k) => k.branch(),
            Entry::Link(l) => crate::progression::BRANCHES[l.left()],
            Entry::Vein(v) => v.branch(),
        }
    }

    /// 작은 칸(패시브·맺음·다리)인가 — 줄 안에서 스킬·큰 기술 아래에 선다.
    pub fn small(self) -> bool {
        matches!(
            self,
            Entry::Cell(_) | Entry::Key(_) | Entry::Link(_) | Entry::Vein(_)
        )
    }

    /// 판의 길의 줄(0215 B, 0~7 — 첫 줄 스킬 · 첫 줄 패시브 · 둘째 줄 스킬 · 둘째 줄 패시브(+ 다리) · 큰 기술 줄 · 넷째 줄 스킬 ·
    /// 넷째 줄 패시브(+ 다리) · 맺음): 줄(`row`)마다 스킬 줄과 패시브 줄 — 큰 기술 줄과 맺음 줄은 하나.
    pub fn line(self) -> usize {
        let row = self.row();
        let base = [0, 2, 4, 5, 7, 8][row];
        let one_line = row == ULT_ROW as usize || row == KEY_ROW || row == VEIN_ROW;
        base + usize::from(self.small() && !one_line)
    }

    /// 판의 가로 자리(0215 B — 셋째 갈래의 오른쪽 끝이 `3 × LANE_W`). 같은 줄·갈래의 같은 크기 칸 가운데 몇째인가로 정한다
    /// (`ENTRIES`의 차례), 큰 기술 줄은 큰 기술 먼저.
    pub fn lane(self) -> f32 {
        if let Entry::Link(l) = self {
            return LANE_W * (l.left() + 1) as f32;
        }
        if let Entry::Vein(v) = self {
            return LANE_VEIN[v.index()];
        }
        let (row, b) = (self.row(), self.branch());
        let x0 = LANE_W * b.index() as f32;
        let peers = |small: bool| {
            ENTRIES.iter().filter(move |e| {
                e.row() == row
                    && e.branch() == b
                    && !matches!(e, Entry::Link(_) | Entry::Vein(_))
                    && e.small() == small
            })
        };
        if row == ULT_ROW as usize {
            let k = peers(false).chain(peers(true)).position(|&e| e == self);
            return x0 + LANE_ULT[k.unwrap_or(0).min(1)];
        }
        let n = peers(self.small()).count();
        let k = peers(self.small()).position(|&e| e == self).unwrap_or(0);
        x0 + if n == 1 { LANE_1 } else { LANE_2[k.min(1)] }
    }

    /// 몸의 판의 길에 서는 칸인가(0270 C3): 패시브·다리·맺음. 스킬·큰 기술·결은 무기 판의 칸이라(0270 C) 길 밖이다 — 그 칸들의
    /// 열림은 무기 판의 줄(`plates::offer_on`)이 정한다.
    pub fn on_path(self) -> bool {
        matches!(self, Entry::Cell(_) | Entry::Link(_) | Entry::Key(_))
    }

    /// 몸의 판의 길의 줄(0270 C3, 0~4 — 첫 줄 패시브 · 둘째 줄 패시브(+ 다리) · 큰 기술 줄의 패시브 · 넷째 줄 패시브(+ 다리) · 맺음):
    /// 스킬·큰 기술이 빠진 `line`을 당긴 것. 길 밖의 칸은 `None`.
    pub fn path_line(self) -> Option<usize> {
        if !self.on_path() {
            return None;
        }
        [1, 3, 4, 6, 7].iter().position(|&l| l == self.line())
    }

    /// 몸의 판에서 같은 줄·갈래의 길 칸이 이것 하나뿐인가(다리 빼고) — 홀로 선 칸은 위·아래 줄의 갈래 칸 모두와 잇는다.
    fn alone(self) -> bool {
        let (row, b) = (self.row(), self.branch());
        ENTRIES
            .iter()
            .filter(|e| {
                e.on_path() && !matches!(e, Entry::Link(_)) && e.row() == row && e.branch() == b
            })
            .count()
            == 1
    }

    /// 판의 길(0215 B → 0270 C3 몸의 판): `self`를 가졌으면 `to`가 이어진다. 길은 몸의 판의 칸(패시브·다리·맺음)끼리만 — 첫 줄
    /// 패시브가 뿌리다(스킬이 무기 판으로 옮겨 가 몸의 판이 스킬 밑에서 끊기지 않게). 갈래 안에서는 바로 아래 줄의 가로로 가까운 칸으로
    /// (그 줄에 홀로 선 칸이면 갈래의 칸 모두와), 다리는 같은 줄의 맞닿은 칸에서 들어서고, 들어선 다리에서는 같은 줄·바로 아래 줄의
    /// 가까운 칸으로(양쪽 갈래) 간다.
    pub fn feeds(self, to: Entry) -> bool {
        let (Some(a), Some(b)) = (self.path_line(), to.path_line()) else {
            return false;
        };
        let near = (self.lane() - to.lane()).abs() <= LANE_DX;
        match (self, to) {
            (Entry::Link(_), Entry::Link(_)) => false,
            (Entry::Link(_), _) => near && (b == a || b == a + 1),
            (_, Entry::Link(_)) => near && b == a,
            _ => b == a + 1 && self.branch() == to.branch() && (near || self.alone() || to.alone()),
        }
    }

    /// 값 없이 가진 단: 없다(0172 — 쟁반 뒤집기도 점수로 배운다. 0166의 "처음부터 주는 것은 없다"를 큰 기술까지). 옛 기록이
    /// 값 없이 받았던 것은 기록의 몫으로 지닌다(`Board::with_upend` + `Progress::ult_gift`).
    pub fn free(self) -> u8 {
        0
    }

    /// `rank`단에서 다음 단으로 올리는 값(기술 점수): 단마다 1 — 큰 기술을 배우는 첫 단만 `ULT_LEARN`. 패시브 칸은 1.
    pub fn price(self, rank: u8) -> u32 {
        match self {
            Entry::Ult(_) if rank == 0 => ULT_LEARN,
            _ => 1,
        }
    }

    /// `rank`단까지 쓴 점수(값 없이 가진 단은 빼고).
    pub fn paid(self, rank: u8) -> u32 {
        (self.free()..rank).map(|r| self.price(r)).sum()
    }
}
