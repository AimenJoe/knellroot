//! 무기 판(0270 C1 — 순수 규칙): 기술 판을 **몸의 판**(지금의 갈래 셋 — 흐름·맞받기·자리의 패시브·다리·맺음)과 **쥔 꼴마다의 무기 판**
//! 다섯으로 나눈다. 무기 판은 그 꼴의 대표 타입 둘이 왼쪽·오른쪽 갈래, 물리(또는 다른 타입) 스킬이 가운데.
//!
//! 판은 "칸이 어디에 서나"의 보기다 — 스킬의 단은 하나(`skills::Board::rank` 그대로, 한 스킬이 여러 무기 판에 서도 한 번 배우면 다),
//! 줄의 열림만 **그 판에 쓴 점수**로 센다(`Plate::spent` · `Plate::row_open`). 화면(C2)과 새 스킬(C3)은 이 표를 읽는다.
//!
//! C2(기술 탭의 판 고르기): 살 수 있나는 `offer_on` — 몸의 판은 지금 판 그대로(`Board::offer` — 쓴 점수 합의 줄·길), 무기 판은
//! 줄·길 대신 **그 판의 줄**(`row_open`)로. 사는 것도 `buy_on` 한 곳을 지나 단·기록은 같은 `Board`에 남는다.

use crate::progression::{Vein, VEINS};
use crate::skills::{Board, Entry, Id, Offer, Ult, ENTRIES, IDS};
use crate::status::Kind;
use crate::weapons::Grip;

/// 판 하나: 몸 또는 쥔 꼴.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Plate {
    Body,
    Grip(Grip),
}

/// 기술 탭의 판 차례(몸 먼저).
pub const PLATES: [Plate; 6] = [
    Plate::Body,
    Plate::Grip(Grip::Duel),
    Plate::Grip(Grip::Shield),
    Plate::Grip(Grip::Heavy),
    Plate::Grip(Grip::Focus),
    Plate::Grip(Grip::Ranged),
];

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

/// 쥔 꼴의 대표 타입 둘(왼쪽 · 오른쪽 — 0270, 사장 2026-11-23 확인).
pub fn pair(g: Grip) -> [Kind; 2] {
    match g {
        Grip::Duel => [Kind::Sharp, Kind::Rift],
        Grip::Shield => [Kind::Drain, Kind::Knell],
        Grip::Heavy => [Kind::Shock, Kind::Knell],
        Grip::Focus => [Kind::Frost, Kind::Ember],
        Grip::Ranged => [Kind::Frost, Kind::Root],
    }
}

/// 쥔 꼴의 큰 기술(판의 셋째 줄 가운데).
pub fn great(g: Grip) -> Ult {
    match g {
        Grip::Duel => Ult::Flurry,
        Grip::Shield => Ult::Bell,
        Grip::Heavy => Ult::Upend,
        Grip::Focus => Ult::Blizzard,
        Grip::Ranged => Ult::Line,
    }
}

impl Plate {
    /// 판의 차례(`PLATES`의 자리).
    pub fn index(self) -> usize {
        PLATES.iter().position(|p| *p == self).unwrap_or(0)
    }

    /// 칸 `e`가 이 판에 서나: 몸의 판 = 패시브·맺음·다리(지금 판의 갈래 셋 — 0270 C 세부 설계), 무기 판 = `seats`.
    pub fn has(self, e: Entry) -> bool {
        match self {
            Plate::Body => matches!(e, Entry::Cell(_) | Entry::Key(_) | Entry::Link(_)),
            Plate::Grip(_) => self.seats().iter().any(|s| s.entry == e),
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

    /// 판의 칸들: 그 꼴로 쓸 수 있는 스킬(타입으로 갈래, 첫 줄 스킬은 줄 0 · 나머지 줄 1) + 셋째 줄의 결(대표 타입의 결이 있으면)과
    /// 큰 기술. 몸의 판은 무기 판의 칸이 없다(지금의 판 — `skills::ENTRIES`의 패시브·다리·맺음).
    pub fn seats(self) -> Vec<Seat> {
        let Plate::Grip(g) = self else {
            return Vec::new();
        };
        let [a, b] = pair(g);
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
            .filter(|id| id.def().fits(g))
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
            entry: Entry::Ult(great(g)),
            row: 2,
            col: Col::Mid,
        });
        out
    }

    /// 이 판에 쓴 점수(판에 선 칸의 산 단 — 여러 판에 선 스킬은 판마다 센다: 줄의 열림만 보는 셈이다). 몸의 판은 그 패시브·맺음·다리.
    pub fn spent(self, board: &Board) -> u32 {
        ENTRIES
            .iter()
            .filter(|&&e| self.has(e))
            .map(|&e| e.paid(board.rank(e)))
            .sum()
    }

    /// `row`줄이 이 판에서 열렸나(무기 판의 줄 — 몸의 판은 쓴 점수 합의 `ROW_NEEDS`로 연다, `offer_on`).
    pub fn row_open(self, board: &Board, row: u8) -> bool {
        self.spent(board) >= row_need(row)
    }
}

/// 무기 판의 `row`줄이 열리는 그 판의 점수.
pub fn row_need(row: u8) -> u32 {
    PLATE_ROWS[(row as usize).min(PLATE_ROWS.len() - 1)]
}

/// 판 `plate`에서 칸 `e`의 다음 단을 지금 살 수 있나(0270 C2). 몸의 판(또는 그 판에 서지 않은 칸)은 `Board::offer` 그대로.
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

/// 칸 `e`가 어느 판에든 서나(몸의 판 또는 무기 판 하나 — 0270 C5).
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
    use crate::weapons::GRIPS;

    #[test]
    fn every_grip_pairs_two_different_types_and_every_type_has_a_home() {
        let mut homes = Vec::new();
        for g in GRIPS {
            let [a, b] = pair(g);
            assert!(
                a != b && a != Kind::Physical && b != Kind::Physical,
                "{g:?}"
            );
            homes.extend([a, b]);
        }
        for k in crate::status::TYPED {
            assert!(homes.contains(&k), "{k:?} belongs to no weapon");
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
            let Plate::Grip(g) = p else { unreachable!() };
            let seats = p.seats();
            for id in IDS {
                let n = seats.iter().filter(|s| s.entry == Entry::Skill(id)).count();
                assert_eq!(n, usize::from(id.def().fits(g)), "{g:?} {id:?}");
            }
            let [a, b] = pair(g);
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
        let duel = Plate::Grip(Grip::Duel);
        let staff = Plate::Grip(Grip::Focus);
        // 가로지르기(결투만) 셋.
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
        let duel = Plate::Grip(Grip::Duel);
        let mut b = Board::default();
        let throw = Entry::Skill(Id::Throw);
        let flurry = Entry::Ult(Ult::Flurry);
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
        assert!(buy_on(duel, &mut b, Entry::Skill(Id::Whirl), 9));
        assert!(buy_on(duel, &mut b, Entry::Skill(Id::Whirl), 9));
        // 큰 기술의 첫 단은 2점 — 남은 점수 1이면 모자란다.
        assert_eq!(offer_on(duel, &b, flurry, b.spent() + 1), Offer::Short);
        let total = b.spent() + 2;
        assert!(buy_on(duel, &mut b, flurry, total));
        assert_eq!(b.rank(flurry), 1);
        // 결은 그 타입의 스킬을 배웠어야(지금 규칙 그대로): 균열의 결은 가로지르기를 안다 — 산다.
        let rift = Entry::Vein(Vein::Rift);
        assert_eq!(offer_on(duel, &b, rift, 99), Offer::Ready);
        let mut fresh = Board::default();
        fresh.rank[Id::Whirl.index()] = 5;
        fresh.raise(Entry::Skill(Id::Gather));
        assert!(duel.row_open(&fresh, 2));
        assert_eq!(
            offer_on(duel, &fresh, rift, 99),
            Offer::Cut,
            "no rift skill yet"
        );
        // 다른 판의 점수는 세지 않는다: 지팡이 판의 둘째 줄은 아직.
        let staff = Plate::Grip(Grip::Focus);
        assert_eq!(
            offer_on(staff, &b, Entry::Skill(Id::Embers), 99),
            Offer::Locked(3)
        );
        // 몸의 판(과 그 판에 서지 않은 칸)은 지금 판 그대로.
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

    /// 0270 C5: 지금은 판의 칸 모두가 어느 판에든 선다 — 돌려줄 것이 없다(새 칸을 표에 더하고 판에 안 앉히면 여기서 걸린다).
    #[test]
    fn every_entry_stands_on_some_plate_today() {
        for e in ENTRIES {
            assert!(stands(e), "{e:?} stands on no plate");
        }
        let all = Board::maxed(crate::skills::Pick::B);
        assert_eq!(drop_unseated(all), all, "nothing is refunded");
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

    /// 칸을 보여 줄 판: 든 무기 · 부무기의 판 먼저, 없으면 판의 차례 — 몸의 칸은 몸의 판.
    #[test]
    fn an_entry_is_shown_on_the_held_plate_first() {
        let heavy = Plate::Grip(Grip::Heavy);
        let shield = Plate::Grip(Grip::Shield);
        let whirl = Entry::Skill(Id::Whirl);
        assert_eq!(home(whirl, &[heavy]), heavy);
        assert_eq!(home(whirl, &[]), Plate::Grip(Grip::Duel));
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
