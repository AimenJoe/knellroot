//! 무기 판(0270 C1 → 0272 — 순수 규칙): 기술 판을 **공통 판**(갈래 셋 — 흐름·맞받기·자리의 패시브·다리·맺음, 어느 무기든)과
//! **무기 종류마다의 판**(`weapons::CLASSES` — 한손검·망치·지팡이·쇠뇌·활), 보조 장비 **방패의 판**으로 나눈다. 무기 판은 그 종류의
//! 대표 타입 둘이 왼쪽·오른쪽 갈래, 가운데는 그 종류의 물리 속성(참격·타격 …)의 열.
//!
//! 판은 "칸이 어디에 서나"의 보기다 — 스킬의 단은 하나(`skills::Board::rank` 그대로, 한 스킬이 여러 무기 판에 서도 한 번 배우면 다),
//! 줄의 열림만 **그 판에 쓴 점수**로 센다(`Plate::spent` · `Plate::row_open`). 그래서 판을 가르고 이름을 바꿔도 기록은 그대로다
//! (판에 쓴 점수는 스킬마다의 단에서 다시 센다 — 못 서게 된 칸만 `drop_unseated`가 돌려준다).
//!
//! 살 수 있나는 `offer_on` — 공통 판은 `Board::offer` 그대로(쓴 점수 합의 줄·길), 무기 판은 줄·길 대신 **그 판의 줄**(`row_open`)로.
//! 사는 것도 `buy_on` 한 곳을 지나 단·기록은 같은 `Board`에 남는다.

use crate::i18n::Tr;
use crate::progression::{Vein, VEINS};
use crate::skills::{Board, Entry, Id, Offer, Ult, ENTRIES, IDS};
use crate::status::Kind;
use crate::weapons::{Class, Grip, CLASSES};

/// 판 하나: 공통 · 무기 종류 · 방패(보조 장비 — 한손 계열과 함께 든다).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Plate {
    Body,
    Class(Class),
    Shield,
}

/// 판의 수: 공통 + 종류마다 + 방패.
pub const NP: usize = CLASSES.len() + 2;

/// 기술 탭의 판 차례: 공통 먼저 · 종류(표의 차례) · 방패가 끝(보조 장비). 종류 표에 한 줄을 더하면 판이 하나 는다.
pub const PLATES: [Plate; NP] = {
    let mut out = [Plate::Body; NP];
    let mut i = 0;
    while i < CLASSES.len() {
        out[i + 1] = Plate::Class(CLASSES[i].class);
        i += 1;
    }
    out[NP - 1] = Plate::Shield;
    out
};

/// 방패 판의 대표 타입 둘 · 큰 기술 · 가운데 열의 머리(0270 C — 종류의 것은 `weapons::CLASSES`).
pub const SHIELD_PAIR: [Kind; 2] = [Kind::Drain, Kind::Knell];
pub const SHIELD_GREAT: Ult = Ult::Bell;

/// 대표 속성으로 어느 트리에도 서지 않는 속성(비어 있어야 한다 — 시험이 본다). 0274 ②에서 한손검이 균열을 내려놓은 동안 그 결은
/// 한손검 트리에 잠시 더 섰고(`CARRIED`), ③에서 쌍단검 트리(날카로움 · 균열)가 와 그 줄을 걷었다.
#[cfg(test)]
pub const HOMELESS: [Kind; 0] = [];

/// 무기 판의 줄이 열리는 때(그 판에 쓴 점수): 첫 스킬 · 둘째 스킬 · 결과 큰 기술.
pub const PLATE_ROWS: [u32; 3] = [0, 3, 6];

/// 무기 판의 칸 자리: 갈래(왼쪽 = 대표 타입 A · 가운데 · 오른쪽 = 대표 타입 B).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Col {
    Left,
    Mid,
    Right,
}

/// 무기 판의 칸 하나.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Seat {
    pub entry: Entry,
    pub row: u8,
    pub col: Col,
}

impl Plate {
    /// 쥔 꼴의 판(0272 — 판 줄의 금빛 점·탭을 열 때의 판): 종류의 판 또는 방패 판, 맨손은 무기 판이 없다.
    pub fn held(g: Grip) -> Option<Plate> {
        match g {
            Grip::Bare => None,
            Grip::Class(c) => Some(Plate::Class(c)),
            Grip::Shield => Some(Plate::Shield),
        }
    }

    /// 이 판을 켜는 쥔 꼴(`held`의 거꾸로 — 공통 판은 쥔 것을 묻지 않는다).
    pub fn grip(self) -> Option<Grip> {
        match self {
            Plate::Body => None,
            Plate::Class(c) => Some(Grip::Class(c)),
            Plate::Shield => Some(Grip::Shield),
        }
    }

    /// 판의 이름(판 줄).
    pub fn name(self) -> Tr {
        match self {
            Plate::Body => |s| s.plate_common,
            Plate::Class(c) => c.def().name,
            Plate::Shield => |s| s.plate_shield,
        }
    }

    /// 무기 판의 대표 타입 둘(왼쪽 · 오른쪽 갈래) — 공통 판은 없다.
    pub fn pair(self) -> Option<[Kind; 2]> {
        match self {
            Plate::Body => None,
            Plate::Class(c) => Some(c.def().pair),
            Plate::Shield => Some(SHIELD_PAIR),
        }
    }

    /// 무기 판의 큰 기술(셋째 줄 가운데).
    pub fn great(self) -> Option<Ult> {
        match self {
            Plate::Body => None,
            Plate::Class(c) => Some(c.def().great),
            Plate::Shield => Some(SHIELD_GREAT),
        }
    }

    /// 무기 판 가운데 열의 머리: 그 종류의 물리 속성(참격·타격·관통·마법), 방패 판은 방어.
    pub fn attribute(self) -> Option<Tr> {
        match self {
            Plate::Body => None,
            Plate::Class(c) => Some(c.def().attribute),
            Plate::Shield => Some(|s| s.attr_guard),
        }
    }

    /// 판의 차례(`PLATES`의 자리).
    pub fn index(self) -> usize {
        PLATES.iter().position(|p| *p == self).unwrap_or(0)
    }

    /// 칸 `e`가 이 판에 서나: 공통 판 = 패시브·맺음·다리(지금 판의 갈래 셋 — 0270 C 세부 설계), 무기 판 = `seats`.
    pub fn has(self, e: Entry) -> bool {
        match self {
            Plate::Body => matches!(e, Entry::Cell(_) | Entry::Key(_) | Entry::Link(_)),
            Plate::Class(_) | Plate::Shield => self.seats().iter().any(|s| s.entry == e),
        }
    }

    /// 이 판에 선 칸(`ENTRIES`의 자리, 그 차례).
    pub fn entries(self) -> Vec<usize> {
        (0..ENTRIES.len())
            .filter(|&i| self.has(ENTRIES[i]))
            .collect()
    }

    /// 무기 판의 칸 `e`의 자리.
    pub fn seat(self, e: Entry) -> Option<Seat> {
        self.seats().into_iter().find(|s| s.entry == e)
    }

    /// 판의 칸들: 그 판의 비트를 가진 스킬(타입으로 갈래, 첫 줄 스킬은 줄 0 · 나머지 줄 1) + 셋째 줄의 결(대표 타입의 결이 있으면)과
    /// 큰 기술. 공통 판은 무기 판의 칸이 없다(`skills::ENTRIES`의 패시브·다리·맺음).
    pub fn seats(self) -> Vec<Seat> {
        let (Some([a, b]), Some(great)) = (self.pair(), self.great()) else {
            return Vec::new();
        };
        let Some(grip) = self.grip() else {
            return Vec::new();
        };
        let col_of = |k: Kind| {
            if k == a {
                Col::Left
            } else if k == b {
                Col::Right
            } else {
                Col::Mid
            }
        };
        let mut out: Vec<Seat> = IDS
            .iter()
            .filter(|id| id.def().fits(grip))
            .map(|&id: &Id| Seat {
                entry: Entry::Skill(id),
                row: u8::from(id.def().row > 0),
                col: col_of(id.def().kind),
            })
            .collect();
        for (k, col) in [(a, Col::Left), (b, Col::Right)] {
            if let Some(v) = VEINS.iter().copied().find(|v: &Vein| v.kind() == k) {
                out.push(Seat {
                    entry: Entry::Vein(v),
                    row: 2,
                    col,
                });
            }
        }
        out.push(Seat {
            entry: Entry::Ult(great),
            row: 2,
            col: Col::Mid,
        });
        out
    }

    /// 이 판에 쓴 점수(판에 선 칸의 산 단 — 여러 판에 선 스킬은 판마다 센다: 줄의 열림만 보는 셈이다). 공통 판은 그 패시브·맺음·다리.
    pub fn spent(self, board: &Board) -> u32 {
        ENTRIES
            .iter()
            .filter(|&&e| self.has(e))
            .map(|&e| e.paid(board.rank(e)))
            .sum()
    }

    /// `row`줄이 이 판에서 열렸나(무기 판의 줄 — 공통 판은 쓴 점수 합의 `ROW_NEEDS`로 연다, `offer_on`).
    pub fn row_open(self, board: &Board, row: u8) -> bool {
        self.spent(board) >= row_need(row)
    }
}

/// 무기 판의 `row`줄이 열리는 그 판의 점수.
pub fn row_need(row: u8) -> u32 {
    PLATE_ROWS[(row as usize).min(PLATE_ROWS.len() - 1)]
}

/// 판 `plate`에서 칸 `e`의 다음 단을 지금 살 수 있나(0270 C2). 공통 판(또는 그 판에 서지 않은 칸)은 `Board::offer` 그대로.
/// 무기 판은 쓴 점수 합의 줄(`ROW_NEEDS`)과 판의 길(0215)을 묻지 않고 **그 판의 줄**로 연다 — `Locked(n)` = 이 판에 n점 더.
/// 결의 칸은 지금 규칙 그대로(그 타입의 스킬을 먼저 배워야 — `Offer::Cut`), 큰 기술은 첫 단의 값 `ULT_LEARN`.
pub fn offer_on(plate: Plate, board: &Board, e: Entry, total: u32) -> Offer {
    let Some(seat) = plate.seat(e) else {
        return board.offer(e, total);
    };
    if board.rank(e) >= e.max() {
        Offer::Full
    } else if !plate.row_open(board, seat.row) {
        Offer::Locked(row_need(seat.row) - plate.spent(board))
    } else if matches!(e, Entry::Vein(_)) && !board.reachable(e) {
        Offer::Cut
    } else if board.left(total) < e.price(board.rank(e)) {
        Offer::Short
    } else {
        Offer::Ready
    }
}

/// 판 `plate`에서 칸 `e`의 다음 단을 산다(`offer_on`이 살 수 있다고 할 때만) — 단은 같은 `Board`에(기록 키 그대로).
pub fn buy_on(plate: Plate, board: &mut Board, e: Entry, total: u32) -> bool {
    if plate.seat(e).is_none() {
        return board.buy(e, total);
    }
    if offer_on(plate, board, e, total) != Offer::Ready {
        return false;
    }
    board.raise(e);
    true
}

/// 어느 판에서든 지금 살 수 있는 칸이 있나(쉼터가 빛난다 · 탭 머리의 민트 점 · 튜토리얼의 배우기 걸음).
pub fn can_buy(board: &Board, total: u32) -> bool {
    PLATES.iter().any(|&p| {
        ENTRIES
            .iter()
            .any(|&e| p.has(e) && offer_on(p, board, e, total) == Offer::Ready)
    })
}

/// 칸 `e`가 어느 판에든 서나(공통 판 또는 무기 판 하나 — 0270 C5).
pub fn stands(e: Entry) -> bool {
    PLATES.iter().any(|p| p.has(e))
}

/// 0270 C5 — 판에 못 서는 칸의 점수 돌려주기: 어느 판에도 서지 않는 칸(판이 바뀌어 자리를 잃은 스킬·큰 기술·패시브·다리·맺음·결)은
/// 처음으로 되돌려 그 점수가 남은 점수로 돌아가게 한다(`Board::forget`). 나머지 산 것은 그대로. 기록을 읽을 때 한 번(`save`의
/// `migrate_after_read` — `Board::sane` 앞: 되돌린 스킬의 결은 `sane`이 닫는다). 그 칸이 끼워져 있던 스킬 칸은 빈 칸이 되고
/// (안 배운 스킬이 놓인 칸 = 빈 칸, 0166), 끼운 큰 기술은 `wearable_ult`가 배운 다른 것으로 옮긴다.
pub fn drop_unseated(board: Board) -> Board {
    drop_where(board, stands)
}

/// `drop_unseated`의 몸: `stands`가 거짓인 칸을 잊는다(시험은 가짜 판으로 부른다 — 지금은 모든 칸이 어느 판에든 선다).
fn drop_where(mut board: Board, stands: impl Fn(Entry) -> bool) -> Board {
    for e in ENTRIES {
        if !stands(e) && board.rank(e) > 0 {
            board.forget(e);
        }
    }
    board
}

/// 칸 `e`를 보여 줄 판: `prefer`(든 무기·부무기의 판 차례) 가운데 그 칸이 선 첫 판, 없으면 `PLATES` 차례의 첫 판.
pub fn home(e: Entry, prefer: &[Plate]) -> Plate {
    prefer
        .iter()
        .chain(PLATES.iter())
        .copied()
        .find(|p| p.has(e))
        .unwrap_or(Plate::Body)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SWORD: Plate = Plate::Class(Class::Sword);
    const HAMMER: Plate = Plate::Class(Class::Hammer);
    const STAFF: Plate = Plate::Class(Class::Staff);
    const CROSSBOW: Plate = Plate::Class(Class::Crossbow);
    const BOW: Plate = Plate::Class(Class::Bow);
    const SPEAR: Plate = Plate::Class(Class::Spear);
    const DAGGER: Plate = Plate::Class(Class::Dagger);
    const GREATSWORD: Plate = Plate::Class(Class::Greatsword);

    #[test]
    fn every_weapon_plate_pairs_two_different_types_and_every_type_has_a_home() {
        let mut homes = Vec::new();
        for p in PLATES.into_iter().skip(1) {
            let [a, b] = p.pair().unwrap();
            assert!(
                a != b && a != Kind::Physical && b != Kind::Physical,
                "{p:?}"
            );
            homes.extend([a, b]);
        }
        for k in crate::status::TYPED {
            assert_eq!(
                homes.contains(&k),
                !HOMELESS.contains(&k),
                "{k:?} belongs to no weapon"
            );
        }
    }

    /// 0272: 판 줄 = 공통 · 종류(표의 차례) · 방패. 종류마다 판 하나 — 쥔 꼴과 판은 서로의 거꾸로다(맨손은 무기 판이 없다).
    #[test]
    fn the_strip_is_common_then_every_class_then_the_shield() {
        assert_eq!(
            PLATES,
            [
                Plate::Body,
                SWORD,
                HAMMER,
                STAFF,
                CROSSBOW,
                BOW,
                SPEAR,
                DAGGER,
                GREATSWORD,
                Plate::Shield
            ]
        );
        assert_eq!(PLATES.len(), CLASSES.len() + 2);
        for (i, c) in CLASSES.iter().enumerate() {
            assert_eq!(PLATES[i + 1], Plate::Class(c.class));
        }
        for p in PLATES {
            assert_eq!(PLATES[p.index()], p);
            assert_eq!(
                p.grip().and_then(Plate::held),
                (p != Plate::Body).then_some(p)
            );
            assert_eq!(p.attribute().is_some(), p != Plate::Body);
        }
        assert_eq!(Plate::held(Grip::Bare), None);
        use crate::weapons::Id as W;
        assert_eq!(Plate::held(W::Fists.grip(false)), None, "bare hands");
        assert_eq!(Plate::held(W::Fists.grip(true)), Some(Plate::Shield));
        assert_eq!(Plate::held(W::Sword.grip(false)), Some(SWORD));
        assert_eq!(Plate::held(W::Sword.grip(true)), Some(Plate::Shield));
        // 0274: 망치도 한손 — 방패와 함께면 방패 판(한손검과 같다).
        assert_eq!(Plate::held(W::Hammer.grip(true)), Some(Plate::Shield));
        assert_eq!(Plate::held(W::Hammer.grip(false)), Some(HAMMER));
        assert_eq!(Plate::held(W::Spear.grip(true)), Some(SPEAR), "two hands");
        // 0274 ③: 쌍단검은 두 손이 다 찬다 — 방패와 들지 않는다(방패를 메어도 단검 트리).
        assert_eq!(
            Plate::held(W::Daggers.grip(true)),
            Some(DAGGER),
            "two blades"
        );
        // 0274 ④: 대검도 두 손 — 방패를 메어도 대검 트리.
        assert_eq!(
            Plate::held(W::Greatsword.grip(true)),
            Some(GREATSWORD),
            "two hands on the hilt"
        );
        assert_eq!(Plate::held(W::Crossbow.grip(false)), Some(CROSSBOW));
        assert_eq!(Plate::held(W::Bow.grip(false)), Some(BOW));
    }

    /// 0274 ②: 한손검 판 = 전사의 결 — 왼쪽 충격(자루 치기 · 밀치기 · 결) · 오른쪽 흡수(버팀 · 재기 · 결) · 가운데는 휘돌이 · 끌어모으기.
    /// 0274 ③: 옛 짝(날카로움 · 균열)의 스킬 넷(가로지르기 · 바늘땀 · 던지기 · 제비걸음)과 균열의 결은 쌍단검 트리로 옮겼다 — 전사의
    /// 기술만 남는다. 궁극기는 몰아치기 그대로.
    #[test]
    fn the_sword_plate_is_shock_and_drain_and_only_fighter_arts() {
        assert_eq!(SWORD.pair(), Some([Kind::Shock, Kind::Drain]));
        assert_eq!(SWORD.great(), Some(Ult::Flurry));
        let at = |col: Col, row: u8| -> Vec<Entry> {
            SWORD
                .seats()
                .into_iter()
                .filter(|s| s.col == col && s.row == row)
                .map(|s| s.entry)
                .collect()
        };
        let sk = |ids: &[Id]| ids.iter().map(|&i| Entry::Skill(i)).collect::<Vec<_>>();
        assert_eq!(at(Col::Left, 0), sk(&[Id::Pommel]));
        assert_eq!(at(Col::Left, 1), sk(&[Id::Shove]));
        assert_eq!(at(Col::Right, 0), sk(&[Id::Brace]));
        assert_eq!(at(Col::Right, 1), sk(&[Id::Rally]));
        assert_eq!(at(Col::Mid, 0), sk(&[Id::Whirl, Id::Gather]));
        // 0274 ④: 비어 있던 가운데 둘째 줄에 되울림(전사의 받아치기 — 방패 없이 받아치면 둘레가 울린다, 있는 스킬의 `hands`).
        assert_eq!(at(Col::Mid, 1), sk(&[Id::Echo]));
        assert_eq!(at(Col::Left, 2), [Entry::Vein(Vein::Shock)]);
        assert_eq!(at(Col::Mid, 2), [Entry::Ult(Ult::Flurry)]);
        assert_eq!(at(Col::Right, 2), [Entry::Vein(Vein::Drain)]);
        for id in [Id::Cut, Id::Stitch, Id::Throw, Id::Swallow] {
            assert!(!SWORD.has(Entry::Skill(id)), "{id:?} left for the daggers");
        }
        assert!(!SWORD.has(Entry::Vein(Vein::Rift)));
        assert_eq!(SWORD.seats().len(), 10);
        // 새 셋은 한손검만의 것(방패 없이 — 방패를 들면 방패 트리).
        for id in [Id::Pommel, Id::Shove, Id::Rally] {
            assert!(
                !SHIELD_PLATE_HAS(id) && SWORD.has(Entry::Skill(id)),
                "{id:?}"
            );
        }
    }

    /// 0274 ③: 쌍단검 판 = 도적의 결 — 왼쪽 날카로움(바늘땀 · 제비걸음 · 결), 오른쪽 균열(가로지르기 · 던지기 · 결), 가운데 관통
    /// (끌어모으기 / 빠지며 베기(새) / 궁극기 그림자 습격). 한손검 트리에서 옮긴 넷이 모두 여기 서고, 버팀(발을 박는 기술)은 서지 않는다.
    #[test]
    fn the_dagger_plate_seats_sharp_pierce_and_rift() {
        assert_eq!(DAGGER.pair(), Some([Kind::Sharp, Kind::Rift]));
        assert_eq!(DAGGER.great(), Some(Ult::Ambush));
        let at = |col: Col, row: u8| -> Vec<Entry> {
            DAGGER
                .seats()
                .into_iter()
                .filter(|s| s.col == col && s.row == row)
                .map(|s| s.entry)
                .collect()
        };
        let sk = |ids: &[Id]| ids.iter().map(|&i| Entry::Skill(i)).collect::<Vec<_>>();
        assert_eq!(at(Col::Left, 0), sk(&[Id::Stitch]));
        assert_eq!(at(Col::Left, 1), sk(&[Id::Swallow]));
        assert_eq!(at(Col::Left, 2), [Entry::Vein(Vein::Sharp)]);
        assert_eq!(at(Col::Mid, 0), sk(&[Id::Gather]));
        assert_eq!(at(Col::Mid, 1), sk(&[Id::Slip]));
        assert_eq!(at(Col::Mid, 2), [Entry::Ult(Ult::Ambush)]);
        assert_eq!(at(Col::Right, 0), sk(&[Id::Cut]));
        assert_eq!(at(Col::Right, 1), sk(&[Id::Throw]));
        assert_eq!(at(Col::Right, 2), [Entry::Vein(Vein::Rift)]);
        assert_eq!(DAGGER.seats().len(), 9);
        assert!(!DAGGER.has(Entry::Skill(Id::Brace)), "rogues don't brace");
        for p in PLATES {
            assert_eq!(p.has(Entry::Skill(Id::Slip)), p == DAGGER, "{p:?}");
        }
    }

    /// 0274 ④: 대검 판 = 광전사의 결 — 왼쪽 균열(가로지르기 / 땅 가르기(새) / 결), 오른쪽 잔불(불씨 휩쓸기(새) / 결), 가운데 참격
    /// (휘돌이 · 끌어모으기 · 버팀 / 궁극기 칼바람(새)). 9노드 — 첫 줄 스킬은 다섯.
    #[test]
    fn the_greatsword_plate_seats_rift_slash_and_ember() {
        assert_eq!(GREATSWORD.pair(), Some([Kind::Rift, Kind::Ember]));
        assert_eq!(GREATSWORD.great(), Some(Ult::Tempest));
        let at = |col: Col, row: u8| -> Vec<Entry> {
            GREATSWORD
                .seats()
                .into_iter()
                .filter(|s| s.col == col && s.row == row)
                .map(|s| s.entry)
                .collect()
        };
        let sk = |ids: &[Id]| ids.iter().map(|&i| Entry::Skill(i)).collect::<Vec<_>>();
        assert_eq!(at(Col::Left, 0), sk(&[Id::Cut]));
        assert_eq!(at(Col::Left, 1), sk(&[Id::Sunder]));
        assert_eq!(at(Col::Left, 2), [Entry::Vein(Vein::Rift)]);
        assert_eq!(at(Col::Mid, 0), sk(&[Id::Whirl, Id::Gather, Id::Brace]));
        assert_eq!(at(Col::Mid, 1), sk(&[]));
        assert_eq!(at(Col::Mid, 2), [Entry::Ult(Ult::Tempest)]);
        assert_eq!(at(Col::Right, 0), sk(&[Id::Sear]));
        assert_eq!(at(Col::Right, 1), sk(&[]));
        assert_eq!(at(Col::Right, 2), [Entry::Vein(Vein::Ember)]);
        assert_eq!(GREATSWORD.seats().len(), 9);
        for p in PLATES {
            for id in [Id::Sunder, Id::Sear] {
                assert_eq!(p.has(Entry::Skill(id)), p == GREATSWORD, "{p:?} {id:?}");
            }
        }
        let s = crate::i18n::s();
        assert_eq!(
            GREATSWORD.attribute().map(|a| a(s)),
            SWORD.attribute().map(|a| a(s)),
            "both blades cut"
        );
    }

    #[allow(non_snake_case)]
    fn SHIELD_PLATE_HAS(id: Id) -> bool {
        Plate::Shield.has(Entry::Skill(id))
    }

    /// 0272: 원거리 판을 쇠뇌와 활로 갈랐다 — 종류마다 제 타입 짝과 큰 기술 하나, 타입 갈래마다 첫 줄과 둘째 줄의 스킬.
    #[test]
    fn the_crossbow_and_the_bow_have_their_own_plates() {
        assert_eq!(CROSSBOW.pair(), Some([Kind::Ember, Kind::Shock]));
        assert_eq!(BOW.pair(), Some([Kind::Root, Kind::Frost]));
        assert_eq!(CROSSBOW.great(), Some(Ult::Line));
        assert_eq!(BOW.great(), Some(Ult::Rain));
        let at = |p: Plate, col: Col, row: u8| -> Vec<Entry> {
            p.seats()
                .into_iter()
                .filter(|s| s.col == col && s.row == row)
                .map(|s| s.entry)
                .collect()
        };
        let sk = |ids: &[Id]| ids.iter().map(|&i| Entry::Skill(i)).collect::<Vec<_>>();
        assert_eq!(at(CROSSBOW, Col::Left, 0), sk(&[Id::Flare]));
        assert_eq!(at(CROSSBOW, Col::Left, 1), sk(&[Id::Embers]));
        assert_eq!(at(CROSSBOW, Col::Mid, 0), sk(&[Id::Gather]));
        assert_eq!(at(CROSSBOW, Col::Mid, 1), sk(&[Id::Echo]));
        assert_eq!(at(CROSSBOW, Col::Right, 0), sk(&[Id::Fuse]));
        assert_eq!(at(CROSSBOW, Col::Right, 1), sk(&[Id::Mine]));
        assert_eq!(at(CROSSBOW, Col::Left, 2), [Entry::Vein(Vein::Ember)]);
        assert_eq!(at(CROSSBOW, Col::Mid, 2), [Entry::Ult(Ult::Line)]);
        assert_eq!(at(CROSSBOW, Col::Right, 2), [Entry::Vein(Vein::Shock)]);
        assert_eq!(at(BOW, Col::Left, 0), sk(&[Id::Snare]));
        assert_eq!(at(BOW, Col::Left, 1), sk(&[Id::Roots]));
        assert_eq!(at(BOW, Col::Mid, 0), []);
        assert_eq!(at(BOW, Col::Mid, 1), sk(&[Id::Echo]));
        assert_eq!(at(BOW, Col::Right, 0), sk(&[Id::Gather]));
        assert_eq!(at(BOW, Col::Right, 1), sk(&[Id::Snow]));
        assert_eq!(at(BOW, Col::Left, 2), [Entry::Vein(Vein::Root)]);
        assert_eq!(at(BOW, Col::Mid, 2), [Entry::Ult(Ult::Rain)]);
        assert_eq!(at(BOW, Col::Right, 2), [Entry::Vein(Vein::Frost)]);
        // 판마다의 칸 수(스킬 · 결 · 큰 기술 — `tools/test/hearth.js`가 같은 수를 센다).
        let n = |p: Plate| p.seats().len();
        assert_eq!(
            [
                n(SWORD),
                n(HAMMER),
                n(STAFF),
                n(CROSSBOW),
                n(BOW),
                n(Plate::Shield),
                n(SPEAR),
                n(DAGGER),
                n(GREATSWORD)
            ],
            // 0274 ④: 한손검 9 → 10(가운데 둘째 줄의 되울림), 대검 9.
            [10, 11, 10, 9, 8, 10, 10, 9, 9]
        );
        // 옛 원거리 판에 섰던 스킬은 모두 두 판 가운데 하나에는 선다(0270 C5 — 돌려줄 것이 없다).
        for id in [
            Id::Gather,
            Id::Echo,
            Id::Embers,
            Id::Roots,
            Id::Snow,
            Id::Snare,
        ] {
            let e = Entry::Skill(id);
            assert!(CROSSBOW.has(e) || BOW.has(e), "{id:?}");
        }
    }

    /// 0272 2단계: 창 판 — 왼쪽 뿌리(못 박기 · 뿌리 솟기 · 뿌리의 속성 패시브), 가운데 관통(꿰뚫기 · 끌어모으기 · 버팀 /
    /// 장대 휩쓸기 / 궁극기 질풍 찌르기), 오른쪽 날카로움(바늘땀 · 날카로움의 속성 패시브). 새 스킬 셋은 창에만 선다.
    #[test]
    fn the_spear_plate_seats_root_pierce_and_sharp() {
        assert_eq!(SPEAR.pair(), Some([Kind::Root, Kind::Sharp]));
        assert_eq!(SPEAR.great(), Some(Ult::Gale));
        let at = |col: Col, row: u8| -> Vec<Entry> {
            SPEAR
                .seats()
                .into_iter()
                .filter(|s| s.col == col && s.row == row)
                .map(|s| s.entry)
                .collect()
        };
        let sk = |ids: &[Id]| ids.iter().map(|&i| Entry::Skill(i)).collect::<Vec<_>>();
        assert_eq!(at(Col::Left, 0), sk(&[Id::Pin]));
        assert_eq!(at(Col::Left, 1), sk(&[Id::Roots]));
        assert_eq!(at(Col::Left, 2), [Entry::Vein(Vein::Root)]);
        assert_eq!(at(Col::Mid, 0), sk(&[Id::Gather, Id::Brace, Id::Skewer]));
        assert_eq!(at(Col::Mid, 1), sk(&[Id::Pole]));
        assert_eq!(at(Col::Mid, 2), [Entry::Ult(Ult::Gale)]);
        assert_eq!(at(Col::Right, 0), sk(&[Id::Stitch]));
        assert_eq!(at(Col::Right, 1), sk(&[]));
        assert_eq!(at(Col::Right, 2), [Entry::Vein(Vein::Sharp)]);
        assert_eq!(SPEAR.seats().len(), 10);
        for p in PLATES {
            for id in [Id::Skewer, Id::Pole, Id::Pin] {
                assert_eq!(p.has(Entry::Skill(id)), p == SPEAR, "{p:?} {id:?}");
            }
        }
        // 무기 판마다 궁극기는 하나, 판끼리 겹치지 않는다.
        let greats: Vec<Ult> = PLATES.iter().filter_map(|p| p.great()).collect();
        for (i, g) in greats.iter().enumerate() {
            assert!(!greats[i + 1..].contains(g), "{g:?}");
        }
    }

    /// 새 판의 첫 스킬이 막히지 않게(0166 — 판마다): 무기 판마다 처음부터 서는 스킬이 있다.
    #[test]
    fn every_weapon_plate_has_a_skill_open_from_the_start() {
        for p in PLATES.into_iter().skip(1) {
            assert!(
                p.seats()
                    .iter()
                    .any(|s| s.row == 0 && matches!(s.entry, Entry::Skill(_))),
                "{p:?}"
            );
            assert!(p.row_open(&Board::default(), 0));
            assert!(!p.row_open(&Board::default(), 1));
        }
    }

    /// 쓸 수 있는 스킬은 모두 그 꼴의 판에 한 번씩 서고, 대표 타입의 스킬은 제 갈래에 선다.
    #[test]
    fn a_skill_stands_once_on_every_plate_it_fits_on_its_types_side() {
        for p in PLATES.into_iter().skip(1) {
            let g = p.grip().unwrap();
            let seats = p.seats();
            for id in IDS {
                let n = seats.iter().filter(|s| s.entry == Entry::Skill(id)).count();
                assert_eq!(n, usize::from(id.def().fits(g)), "{g:?} {id:?}");
            }
            let [a, b] = p.pair().unwrap();
            for s in &seats {
                if let Entry::Skill(id) = s.entry {
                    let k = id.def().kind;
                    let want = if k == a {
                        Col::Left
                    } else if k == b {
                        Col::Right
                    } else {
                        Col::Mid
                    };
                    assert_eq!(s.col, want, "{g:?} {id:?}");
                }
            }
            assert_eq!(
                seats
                    .iter()
                    .filter(|s| matches!(s.entry, Entry::Ult(_)))
                    .count(),
                1
            );
        }
    }

    /// 판에 쓴 점수가 줄을 연다 — 다른 판의 점수는 세지 않는다.
    #[test]
    fn points_spent_on_one_plate_open_its_rows_and_not_anothers() {
        let mut b = Board::default();
        let duel = DAGGER;
        let staff = STAFF;
        // 가로지르기(쌍단검만, 0274 ③) 셋.
        for _ in 0..3 {
            b.rank[Id::Cut.index()] += 1;
        }
        assert_eq!(duel.spent(&b), 3);
        assert!(duel.row_open(&b, 1));
        assert_eq!(staff.spent(&b), 0);
        assert!(!staff.row_open(&b, 1));
    }

    /// 0270 C2: 무기 판의 칸은 쓴 점수 합의 줄·판의 길을 묻지 않고 **그 판의 줄**로 산다 — 둘째 줄은 그 판에 3점, 셋째 줄은 6점.
    /// 산 단은 같은 판(`Board`)에 남는다(기록 키 그대로).
    #[test]
    fn a_weapon_plate_sells_by_its_own_rows_and_the_body_keeps_the_old_rules() {
        // 0274 ③: 가로지르기 · 던지기 · 균열의 결은 쌍단검 판에 — 그 판의 궁극기는 그림자 습격.
        let duel = DAGGER;
        let mut b = Board::default();
        let throw = Entry::Skill(Id::Throw);
        let flurry = Entry::Ult(Ult::Ambush);
        // 던지기(둘째 줄): 옛 판은 쓴 점수 합 4부터 — 결투 판은 그 판의 3부터.
        assert_eq!(offer_on(duel, &b, throw, 9), Offer::Locked(3));
        for _ in 0..3 {
            assert!(buy_on(duel, &mut b, Entry::Skill(Id::Cut), 9));
        }
        assert_eq!(
            b.offer(throw, 9),
            Offer::Locked(1),
            "the old board still waits"
        );
        assert_eq!(offer_on(duel, &b, throw, 9), Offer::Ready);
        assert!(buy_on(duel, &mut b, throw, 9));
        assert_eq!(b.rank(throw), 1);
        // 셋째 줄(결·큰 기술)은 그 판에 6.
        assert_eq!(offer_on(duel, &b, flurry, 9), Offer::Locked(2));
        assert!(!buy_on(duel, &mut b, flurry, 9));
        assert!(buy_on(duel, &mut b, Entry::Skill(Id::Stitch), 9));
        assert!(buy_on(duel, &mut b, Entry::Skill(Id::Stitch), 9));
        // 큰 기술의 첫 단은 2점 — 남은 점수 1이면 모자란다.
        assert_eq!(offer_on(duel, &b, flurry, b.spent() + 1), Offer::Short);
        let total = b.spent() + 2;
        assert!(buy_on(duel, &mut b, flurry, total));
        assert_eq!(b.rank(flurry), 1);
        // 결은 그 타입의 스킬을 배웠어야(지금 규칙 그대로): 균열의 결은 가로지르기를 안다 — 산다.
        let rift = Entry::Vein(Vein::Rift);
        assert_eq!(offer_on(duel, &b, rift, 99), Offer::Ready);
        let mut fresh = Board::default();
        fresh.rank[Id::Stitch.index()] = 5;
        fresh.raise(Entry::Skill(Id::Gather));
        assert!(duel.row_open(&fresh, 2));
        assert_eq!(
            offer_on(duel, &fresh, rift, 99),
            Offer::Cut,
            "no rift skill yet"
        );
        // 다른 판의 점수는 세지 않는다: 지팡이 판의 둘째 줄은 아직.
        let staff = STAFF;
        assert_eq!(
            offer_on(staff, &b, Entry::Skill(Id::Embers), 99),
            Offer::Locked(3)
        );
        // 공통 판(과 그 판에 서지 않은 칸)은 지금 판 그대로.
        for e in ENTRIES {
            assert_eq!(offer_on(Plate::Body, &b, e, 99), b.offer(e, 99), "{e:?}");
        }
        assert_eq!(
            offer_on(staff, &b, Entry::Skill(Id::Cut), 99),
            b.offer(Entry::Skill(Id::Cut), 99)
        );
    }

    /// 살 것이 있나(쉼터의 빛·탭 머리의 점)는 판마다 본다: 옛 판으로는 잠긴 둘째 줄 스킬도 그 판의 줄이 열렸으면 살 것이다.
    #[test]
    fn something_to_buy_is_seen_plate_by_plate() {
        let mut b = Board::default();
        assert!(!can_buy(&b, 0));
        assert!(can_buy(&b, 1), "a first skill on any plate");
        b.rank[Id::Quake.index()] = 3;
        // 땅울림(망치 판 둘째 줄)만 배운 판: 그 판의 3점 — 망치의 다른 칸·결은 살 수 있다.
        assert!(can_buy(&b, 4));
        assert!(!can_buy(&b, 3), "all spent");
        let all = Board::maxed(crate::skills::Pick::A);
        assert!(!can_buy(&all, 99));
    }

    /// 0270 C5: 판의 칸 모두가 어느 판에든 선다 — 돌려줄 것이 없다(새 칸을 표에 더하고 판에 안 앉히면 여기서 걸린다). 0274 ③:
    /// 한손검이 내려놓은 균열의 결과 날카로움 · 균열의 스킬 넷은 쌍단검 트리에 선다 — 다 산 판에서 돌려받는 것이 없다.
    #[test]
    fn every_entry_stands_on_some_plate_today() {
        use crate::progression::Vein;
        assert!(HOMELESS.is_empty());
        for e in ENTRIES {
            assert!(stands(e), "{e:?} stands on no plate");
        }
        let all = Board::maxed(crate::skills::Pick::B);
        assert_eq!(drop_unseated(all), all, "nothing is refunded");
        assert!(
            Plate::Class(Class::Dagger).has(Entry::Vein(Vein::Rift)),
            "on the dagger tree"
        );
        for id in [Id::Cut, Id::Throw, Id::Swallow, Id::Stitch] {
            assert!(
                Plate::Class(Class::Dagger).has(Entry::Skill(id)),
                "{id:?} moved to the dagger tree"
            );
        }
    }

    /// 0270 C5: 어느 판에도 서지 않는 칸만 처음으로 — 그 점수는 남은 점수로 돌아오고, 나머지 산 것은 그대로.
    /// 끼운 스킬 칸은 빈 칸이 되고(안 배운 스킬), 끼운 큰 기술은 배운 다른 것으로 옮긴다.
    #[test]
    fn entries_standing_on_no_plate_are_refunded_and_the_rest_is_kept() {
        use crate::progression::{Cell, Key, Link};
        use crate::skills::{Pick, PICK_RANK};
        let mut b = Board::default();
        for _ in 0..PICK_RANK {
            b.raise(Entry::Skill(Id::Cut));
            b.raise(Entry::Skill(Id::Whirl));
        }
        b.flip(Id::Cut);
        b.raise(Entry::Ult(Ult::Flurry));
        b.raise(Entry::Ult(Ult::Bell));
        b.raise(Entry::Cell(Cell::EvenBreath));
        b.raise(Entry::Cell(Cell::Catch));
        b.cells.flip_cell(Cell::EvenBreath);
        b.raise(Entry::Key(Key::Edge));
        b.raise(Entry::Link(Link::RiposteBeat));
        b.raise(Entry::Vein(Vein::Rift));
        b.raise(Entry::Vein(Vein::Ember));
        let before = b.spent();
        // 가짜 판: 가로지르기 · 몰아치기 · 고른 숨 · 날의 맺음 · 다리 · 균열의 결이 자리를 잃었다.
        let lost = [
            Entry::Skill(Id::Cut),
            Entry::Ult(Ult::Flurry),
            Entry::Cell(Cell::EvenBreath),
            Entry::Key(Key::Edge),
            Entry::Link(Link::RiposteBeat),
            Entry::Vein(Vein::Rift),
        ];
        let out = drop_where(b, |e| !lost.contains(&e));
        for e in lost {
            assert_eq!(out.rank(e), 0, "{e:?} refunded");
        }
        let refund: u32 = lost.iter().map(|&e| e.paid(b.rank(e))).sum();
        assert_eq!(out.spent(), before - refund);
        assert_eq!(out.pick[Id::Cut.index()], Pick::A, "its pick is gone too");
        assert_eq!(out.cells.picks, [0; 3], "and the number cell's pick");
        // 나머지는 그대로.
        for e in [
            Entry::Skill(Id::Whirl),
            Entry::Ult(Ult::Bell),
            Entry::Cell(Cell::Catch),
            Entry::Vein(Vein::Ember),
        ] {
            assert_eq!(out.rank(e), b.rank(e), "{e:?} kept");
        }
        // 끼운 것: 가로지르기의 칸은 빈 칸(안 배운 스킬), 몰아치기를 끼웠으면 배운 큰 종으로.
        let load = out.loadout([Id::Cut, Id::Whirl], Ult::Flurry);
        assert_eq!(load.slots[0].rank, 0, "an empty slot");
        assert_eq!(load.slots[1].rank, PICK_RANK);
        assert_eq!(load.great, Ult::Bell);
        // 옛 꼴(앞에서부터 연 수)로 연 칸도 닫힌다 — 같은 갈래의 다른 칸은 남는다.
        let mut old = Board::default();
        old.cells.open = [2, 0, 0];
        let shut = drop_where(old, |e| e != Entry::Cell(Cell::EvenBreath));
        assert!(!shut.cells.has(Cell::EvenBreath));
        assert_eq!(shut.cells.count(), 1);
    }

    /// 칸을 보여 줄 판: 든 무기 · 부무기의 판 먼저, 없으면 판의 차례 — 몸의 칸은 공통 판.
    #[test]
    fn an_entry_is_shown_on_the_held_plate_first() {
        let heavy = HAMMER;
        let shield = Plate::Shield;
        let whirl = Entry::Skill(Id::Whirl);
        assert_eq!(home(whirl, &[heavy]), heavy);
        assert_eq!(home(whirl, &[]), SWORD);
        assert_eq!(home(Entry::Skill(Id::Quake), &[shield]), heavy);
        assert_eq!(
            home(Entry::Cell(crate::progression::Cell::Catch), &[heavy]),
            Plate::Body
        );
        for p in PLATES {
            assert_eq!(PLATES[p.index()], p);
        }
    }
}
