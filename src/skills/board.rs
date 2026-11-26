//! 스킬 트리의 셈 `Board`(점수 · 사기 · 뒤집기 · 되돌리기 — 순수, 0158 G0·G1 → 0161).
use super::*;
use crate::progression::{Branch, Cell, Tree, Vein, VEINS};

/// 기술 점수(0158 나 → 0161): 줍는 화폐가 아니라 셈 — 총 레벨 + 처음 쓰러뜨린 엘리트 수 + **핵**(보스를 이길 때마다, 보스마다
/// `progression::CORE_CAP`까지 — 부르는 쪽이 자른 수) + 주운 **종소리 조각**. 핵·조각은 옛 나무 화폐 "마디"였다 — 이제 점수의 샘이다.
/// 0158의 "보스 첫 쓰러뜨림 +1"은 핵의 수로 바뀌었다. 지금 내용으로 15 + 6 + 6 + 3 = 30.
/// 처음부터 가진 점수는 없다(0167) — 옛 기록의 몫 `GIFT_POINTS`는 기록이 따로 더한다(`Progress::skill_points`).
pub fn points(level: u32, elites: u32, cores: u32, shards: u32) -> u32 {
    level + elites + cores + shards
}

/// 되돌리기 값(잔불): `RESPEC_BASE + RESPEC_PER × 쓴 점수`.
pub const RESPEC_BASE: u32 = 30;
pub const RESPEC_PER: u32 = 5;

/// 칸 하나의 다음 단을 지금 살 수 있나.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Offer {
    /// 살 수 있다(점수 `Entry::price`).
    Ready,
    /// 점수가 모자란다.
    Short,
    /// 줄이 아직 안 열렸다 — 점수를 이만큼 더 써야 열린다.
    Locked(u32),
    /// 끝 단.
    Full,
    /// 다른 맺음이 켜져 있다 — 값 없이 바꿀 수 있다(0214). "살 것"으로 치지 않는다(쉼터의 빛·민트 점·반짝임은 없다).
    Swap,
    /// 길이 끊겼다(0215 B) — 바로 위 줄에서 이어진 칸을 먼저 가져야 한다.
    Cut,
    /// 맺음: 그 갈래에 점수를 이만큼 더 써야 한다(0215 A — `KEY_BRANCH`).
    Branch(u32),
}

/// 기술 판(기록 `skill.rank.*`·`skill.pick.*`·`ult.rank.*`·`tree.cells.*`): 스킬마다의 단과 고르기, 큰 기술마다의 단
/// (`ULTS`의 자리), 연 패시브 칸(0161 — 같은 점수로 산다).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Board {
    pub rank: [u8; N],
    pub pick: [Pick; N],
    pub ult: [u8; NU],
    pub cells: Tree,
}

impl Default for Board {
    fn default() -> Self {
        Board {
            rank: IDS.map(|id| Entry::Skill(id).free()),
            pick: [Pick::A; N],
            ult: ULTS.map(|u| Entry::Ult(u).free()),
            cells: Tree::default(),
        }
    }
}

impl Board {
    /// 칸의 단(패시브 칸은 열렸으면 1, 맺음은 켰으면 1).
    pub fn rank(&self, e: Entry) -> u8 {
        match e {
            Entry::Skill(id) => self.rank[id.index()],
            Entry::Ult(u) => self.ult[u.index()],
            Entry::Cell(c) => self.cells.has(c) as u8,
            Entry::Key(k) => self.cells.keyed(k) as u8,
            Entry::Link(l) => self.cells.linked(l) as u8,
            Entry::Vein(v) => self.cells.veined(v) as u8,
        }
    }

    /// 갈래 `b`에 쓴 점수(0215 A): 그 갈래의 스킬·큰 기술·패시브 칸 — 맺음·다리는 세지 않는다.
    pub fn branch_spent(&self, b: Branch) -> u32 {
        ENTRIES
            .iter()
            .filter(|e| {
                e.branch() == b && !matches!(e, Entry::Key(_) | Entry::Link(_) | Entry::Vein(_))
            })
            .map(|&e| e.paid(self.rank(e)))
            .sum()
    }

    /// 길이 이어졌나(0215 B → 0270 C3): 몸의 판의 첫 줄 패시브는 늘(뿌리), 그 밖은 가진 칸 가운데 이것으로 이어지는 것이 있어야.
    /// 이미 가진 칸(단을 올리는 것·옛 기록)은 묻지 않는다.
    pub fn reachable(&self, e: Entry) -> bool {
        // 결의 칸(0223 11절): 그 타입을 싣는 스킬을 하나라도 배웠어야 한다.
        if let Entry::Vein(v) = e {
            return self.rank(e) > 0
                || IDS
                    .into_iter()
                    .any(|id| self.learned(id) && id.def().kind == v.kind());
        }
        // 스킬·큰 기술은 무기 판의 줄로 연다(0270 C) — 몸의 판의 길을 묻지 않는다.
        !e.on_path()
            || e.path_line() == Some(0)
            || self.rank(e) > 0
            || ENTRIES
                .iter()
                .any(|&from| self.rank(from) > 0 && from.feeds(e))
    }

    /// 맺음을 바꾸는 일인가(0214): 다른 맺음을 이미 켰으면 이 맺음은 값 없이 바꿔 켠다(화로에서).
    pub fn key_swap(&self, e: Entry) -> bool {
        matches!(e, Entry::Key(k) if self.cells.key.is_some_and(|on| on != k))
    }

    pub fn learned(&self, id: Id) -> bool {
        self.rank[id.index()] >= 1
    }

    pub fn has_ult(&self, u: Ult) -> bool {
        self.ult[u.index()] >= 1
    }

    /// 쓴 점수: 산 단만 센다(처음부터 가진 단은 값이 없다, 큰 기술을 배운 첫 단은 2점) + 연 패시브 칸마다 1.
    pub fn spent(&self) -> u32 {
        ENTRIES.iter().map(|&e| e.paid(self.rank(e))).sum()
    }

    /// 남은 점수.
    pub fn left(&self, total: u32) -> u32 {
        total.saturating_sub(self.spent())
    }

    pub fn offer(&self, e: Entry, total: u32) -> Offer {
        let need = ROW_NEEDS[e.row()];
        if self.rank(e) >= e.max() {
            Offer::Full
        } else if self.spent() < need {
            Offer::Locked(need - self.spent())
        } else if !self.reachable(e) {
            Offer::Cut
        } else if let (Entry::Key(_), true) = (e, self.branch_spent(e.branch()) < KEY_BRANCH) {
            Offer::Branch(KEY_BRANCH - self.branch_spent(e.branch()))
        } else if self.key_swap(e) {
            // 켠 맺음이 있으면 값은 이미 냈다 — 바꾸기는 값이 없다(그 갈래의 점수는 바꿀 때도 본다).
            Offer::Swap
        } else if self.left(total) < e.price(self.rank(e)) {
            Offer::Short
        } else {
            Offer::Ready
        }
    }

    /// 다음 단을 산다(점수 `Entry::price`). 3단에 닿으면 고르기는 ㄱ.
    pub fn buy(&mut self, e: Entry, total: u32) -> bool {
        if !matches!(self.offer(e, total), Offer::Ready | Offer::Swap) {
            return false;
        }
        self.raise(e);
        true
    }

    /// 다음 단을 올린다 — 묻지 않는다(살 수 있나는 부른 쪽이: `buy` · 무기 판의 `plates::buy_on`, 0270 C2). 3단에 닿으면 고르기는 ㄱ.
    pub fn raise(&mut self, e: Entry) {
        match e {
            Entry::Skill(id) => {
                let i = id.index();
                self.rank[i] += 1;
                if self.rank[i] == PICK_RANK {
                    self.pick[i] = Pick::A;
                }
            }
            Entry::Ult(u) => self.ult[u.index()] += 1,
            Entry::Cell(c) => self.cells.open_cell(c),
            Entry::Key(k) => self.cells.key = Some(k),
            Entry::Link(l) => self.cells.open_link(l),
            Entry::Vein(v) => self.cells.open_vein(v),
        }
    }

    /// 칸 하나를 처음(안 배움)으로 — 묻지 않는다(0270 C5: 어느 판에도 서지 않는 칸, `plates::drop_unseated`). 그 칸에 쓴 점수는
    /// 남은 점수로 돌아간다(`spent`는 산 단만 센다). 스킬의 고르기·숫자 칸의 고르기도 지운다.
    pub fn forget(&mut self, e: Entry) {
        match e {
            Entry::Skill(id) => {
                self.rank[id.index()] = 0;
                self.pick[id.index()] = Pick::A;
            }
            Entry::Ult(u) => self.ult[u.index()] = 0,
            Entry::Cell(c) => self.cells.close_cell(c),
            Entry::Key(k) => {
                if self.cells.key == Some(k) {
                    self.cells.key = None;
                }
            }
            Entry::Link(l) => self.cells.links &= !(1 << l.index()),
            Entry::Vein(v) => {
                self.cells.veins &= !(1 << v.index());
                self.cells.vein_picks &= !(1 << v.index());
            }
        }
    }

    /// 숫자 칸의 고르기를 뒤집는다(0215 D — 값 없이, 화로에서). 연 숫자 칸만.
    pub fn flip_cell(&mut self, c: Cell) -> bool {
        self.cells.flip_cell(c)
    }

    /// 속성 패시브의 고르기를 뒤집는다(0274 ② — 값 없이, 화로에서). 연 결 가운데 고를 것이 있는 것만.
    pub fn flip_vein(&mut self, v: Vein) -> bool {
        self.cells.flip_vein(v)
    }

    /// 살 수 있는 칸이 있나 — 판 하나의 셈(시험). 앱(쉼터의 빛·탭 머리의 점·튜토리얼)은 판마다 보는 `plates::can_buy`(0270 C2).
    #[cfg(test)]
    pub fn can_buy(&self, total: u32) -> bool {
        ENTRIES
            .iter()
            .any(|&e| self.offer(e, total) == Offer::Ready)
    }

    /// 3단의 고르기를 뒤집는다(값 없이) — 3단이 안 됐으면 그대로.
    pub fn flip(&mut self, id: Id) -> bool {
        let i = id.index();
        if self.rank[i] < PICK_RANK {
            return false;
        }
        self.pick[i] = self.pick[i].other();
        true
    }

    /// 되돌리기 값(잔불).
    pub fn respec_cost(&self) -> u32 {
        RESPEC_BASE + RESPEC_PER * self.spent()
    }

    /// 전부 되돌린다: 스킬·큰 기술은 모두 안 배운 것으로(0166 — 처음의 둘도, 0172 — 쟁반 뒤집기도), 고르기는 지우고 패시브 칸도
    /// 모두 닫는다(0161). 끼운 자리는 그대로 둔다(빈 칸이 된다 — 다시 배우면 그 자리에 있다. 큰 기술도).
    pub fn respec(&mut self, great: &mut Ult) {
        *self = Board::default();
        *great = self.wearable_ult(*great);
    }

    /// 잔불을 내고 되돌린다(화로): 쓴 점수가 없거나 잔불이 모자라면 아무 일도 없다(거짓).
    pub fn respec_paid(&mut self, embers: &mut u32, great: &mut Ult) -> bool {
        let cost = self.respec_cost();
        if self.spent() == 0 || *embers < cost {
            return false;
        }
        *embers -= cost;
        self.respec(great);
        true
    }

    /// 막 배운 스킬 `id`를 빈 칸에(0166): 칸 1부터 보아 안 배운 스킬이 놓인 칸이 있으면 거기에 끼운다 — 처음 배운 스킬이
    /// 기술 화면에서 한 번 더 끼우지 않아도 버튼에 있다. 빈 칸이 없거나 이미 끼워져 있으면 그대로.
    pub fn wear_on_learn(&self, worn: [Id; 2], id: Id) -> [Id; 2] {
        if worn.contains(&id) {
            return worn;
        }
        match worn.iter().position(|&w| !self.learned(w)) {
            Some(slot) => {
                let mut out = worn;
                out[slot] = id;
                out
            }
            None => worn,
        }
    }

    /// 끼울 수 있는 큰 기술: 배운 것이면 그대로, 아니면 배운 것 가운데 첫째(표 순서). 하나도 안 배웠으면(0172 — 새 판) 그대로
    /// 돌려준다 — 그 단은 0이라 월드에서는 큰 기술이 없다(`Loadout::ult` 0: 게이지가 차지 않는다).
    pub fn wearable_ult(&self, great: Ult) -> Ult {
        if self.has_ult(great) {
            great
        } else {
            ULTS.into_iter().find(|&u| self.has_ult(u)).unwrap_or(great)
        }
    }

    /// 배운 큰 기술의 수(0172 — 둘 이상일 때만 기술 탭이 끼운 것을 표시한다).
    pub fn ults_known(&self) -> usize {
        ULTS.into_iter().filter(|&u| self.has_ult(u)).count()
    }

    /// 쟁반 뒤집기를 배운 판 — 0172 전에는 새 판이 이 꼴이었다(큰 기술 하나는 처음부터). 옛 기록을 읽을 때·개발 시나리오 판이
    /// 쓴다(이미 올린 단은 그대로).
    pub fn with_upend(mut self) -> Board {
        let i = Ult::Upend.index();
        self.ult[i] = self.ult[i].max(1);
        self
    }

    /// 기록에서 읽은 판을 바로잡는다: 단은 범위 안으로(처음의 둘·쟁반 뒤집기는 1단 아래로 내려가지 않는다), 3단이 안 된 고르기는
    /// 지우고, 가진 점수보다 많이 썼으면 처음으로 — 점수는 그대로 남으니 다시 찍으면 된다. 처음으로 돌리는 차례(0161): 먼저 스킬·
    /// 큰 기술만(연 패시브 칸은 남긴다 — 옛 기록의 칸은 핵·조각으로 열었고 그 둘은 지금도 점수라 칸만으로는 넘지 않는다. 0158의
    /// "보스 첫 쓰러뜨림 +1"이 핵의 수로 바뀌어 개발판 기록이 한두 점 넘칠 수 있다), 그래도 넘으면(손댄 기록) 칸까지.
    pub fn sane(self, total: u32) -> Board {
        let mut b = self.sane_ranks(total);
        // 결의 칸(0223 11절): 그 타입의 스킬을 모르면 닫는다(넘쳐서 스킬을 돌려받은 기록·손댄 기록).
        for v in VEINS {
            if !IDS
                .into_iter()
                .any(|id| b.learned(id) && id.def().kind == v.kind())
            {
                b.cells.veins &= !(1 << v.index());
            }
        }
        b
    }

    fn sane_ranks(mut self, total: u32) -> Board {
        self.cells = self.cells.sane();
        for (i, id) in IDS.into_iter().enumerate() {
            let e = Entry::Skill(id);
            self.rank[i] = self.rank[i].clamp(e.free(), e.max());
            if self.rank[i] < PICK_RANK {
                self.pick[i] = Pick::A;
            }
        }
        for (i, u) in ULTS.into_iter().enumerate() {
            self.ult[i] = self.ult[i].clamp(Entry::Ult(u).free(), ULT_MAX_RANK);
        }
        if self.spent() <= total {
            return self;
        }
        let cells = Board {
            cells: self.cells,
            ..Board::default()
        };
        if cells.spent() <= total {
            cells
        } else {
            Board::default()
        }
    }

    /// 배운 스킬이 하나라도 있나(0166 — 없으면 스킬 칸 둘이 다 비어 있다).
    pub fn knows_a_skill(&self) -> bool {
        IDS.into_iter().any(|id| self.learned(id))
    }

    /// 처음의 둘(`START`)을 배운 판 — 0166 전에는 새 판이 이 꼴이었다. 옛 기록을 읽을 때·개발 시나리오 판·시험이 쓴다
    /// (이미 올린 단은 그대로).
    pub fn with_starters(mut self) -> Board {
        for id in START {
            self.rank[id.index()] = self.rank[id.index()].max(1);
        }
        self
    }

    /// 모두 끝 단·패시브 칸 모두 열림(개발 창 — 기록에 쓰지 않는다).
    #[cfg(any(test, feature = "dev"))]
    pub fn maxed(pick: Pick) -> Board {
        Board {
            rank: [MAX_RANK; N],
            pick: [pick; N],
            ult: [ULT_MAX_RANK; NU],
            // 맺음은 하나만(0214) — 첫째를 켠다.
            cells: Tree {
                key: Some(crate::progression::KEYS[0]),
                ..Tree::full()
            },
        }
    }

    /// 끼운 둘과 끼운 큰 기술을 월드에 넘길 꼴로.
    pub fn loadout(&self, worn: [Id; 2], great: Ult) -> Loadout {
        let great = self.wearable_ult(great);
        Loadout {
            slots: worn.map(|id| Slot {
                id,
                rank: self.rank[id.index()],
                pick: self.pick[id.index()],
            }),
            ult: self.ult[great.index()],
            great,
        }
    }
}

#[cfg(test)]
mod tests;
