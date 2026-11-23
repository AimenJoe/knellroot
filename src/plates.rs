//! 무기 판(0270 C1 — 순수 규칙): 기술 판을 **몸의 판**(지금의 갈래 셋 — 흐름·맞받기·자리의 패시브·다리·맺음)과 **쥔 꼴마다의 무기 판**
//! 다섯으로 나눈다. 무기 판은 그 꼴의 대표 타입 둘이 왼쪽·오른쪽 갈래, 물리(또는 다른 타입) 스킬이 가운데.
//!
//! 판은 "칸이 어디에 서나"의 보기다 — 스킬의 단은 하나(`skills::Board::rank` 그대로, 한 스킬이 여러 무기 판에 서도 한 번 배우면 다),
//! 줄의 열림만 **그 판에 쓴 점수**로 센다(`Plate::spent` · `Plate::row_open`). 화면(C2)과 새 스킬(C3)은 이 표를 읽는다.
// C2(기술 탭의 판 고르기)가 읽기 전까지는 시험만 부른다.
#![allow(dead_code)]

use crate::progression::{Vein, VEINS};
use crate::skills::{Board, Entry, Id, Ult, IDS};
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
        Grip::Shield | Grip::Heavy => Ult::Upend,
        Grip::Focus | Grip::Ranged => Ult::Line,
    }
}

impl Plate {
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

    /// 이 판에 쓴 점수(판에 선 칸의 산 단 — 여러 판에 선 스킬은 판마다 센다: 줄의 열림만 보는 셈이다).
    pub fn spent(self, board: &Board) -> u32 {
        self.seats()
            .iter()
            .map(|s| s.entry.paid(board.rank(s.entry)))
            .sum()
    }

    /// `row`줄이 이 판에서 열렸나.
    pub fn row_open(self, board: &Board, row: u8) -> bool {
        self.spent(board) >= PLATE_ROWS[(row as usize).min(PLATE_ROWS.len() - 1)]
    }
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
}
