//! 끼운 칸(`Worn`)과 월드에 넘기는 것(`Slot` · `Loadout`), 시험의 단 · 고르기 도우미.
use super::*;

/// 시험: 앞의 몇만 적은 스킬의 단(나머지 0) — 스킬이 늘어도 시험의 표가 그대로 읽히게.
#[cfg(test)]
pub fn ranks<const K: usize>(head: [u8; K]) -> [u8; N] {
    let mut out = [0; N];
    out[..K].copy_from_slice(&head);
    out
}

/// 시험: 앞의 몇만 적은 큰 기술의 단(나머지 0).
#[cfg(test)]
pub fn ult_ranks<const K: usize>(head: [u8; K]) -> [u8; NU] {
    let mut out = [0; NU];
    out[..K].copy_from_slice(&head);
    out
}

/// 시험: 앞의 몇만 적은 고르기(나머지 ㄱ).
#[cfg(test)]
pub fn picks<const K: usize>(head: [Pick; K]) -> [Pick; N] {
    let mut out = [Pick::A; N];
    out[..K].copy_from_slice(&head);
    out
}

/// 끼운 칸 둘(기록·앱이 들고 다닌다) — 기본은 처음의 둘.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Worn(pub [Id; 2]);

impl Default for Worn {
    fn default() -> Self {
        Worn(START)
    }
}

/// 끼운 칸 둘을 바로잡는다(기록에서 읽을 때): 같은 스킬이 두 칸이면 둘째를 표에서 비어 있는 첫 스킬로.
pub fn sane(slots: [Id; 2]) -> [Id; 2] {
    if slots[0] != slots[1] {
        return slots;
    }
    let other = IDS
        .into_iter()
        .find(|i| *i != slots[0])
        .expect("more than one skill");
    [slots[0], other]
}

/// 월드에 넘기는 끼운 칸 하나(0158): 스킬·그 단·그 고르기. 월드는 기록을 모른다 — 이것만 받는다.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slot {
    pub id: Id,
    pub rank: u8,
    pub pick: Pick,
}

/// 끼운 둘 + 끼운 큰 기술(`great`)과 그 단(`ult`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Loadout {
    pub slots: [Slot; 2],
    pub ult: u8,
    pub great: Ult,
}

/// 스킬만 주면 1단·쟁반 뒤집기(시험·옛 부름).
impl From<[Id; 2]> for Loadout {
    fn from(ids: [Id; 2]) -> Self {
        Loadout {
            slots: ids.map(|id| Slot {
                id,
                rank: 1,
                pick: Pick::A,
            }),
            ult: 1,
            great: Ult::Upend,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 월드에 넘기는 끼운 둘: 그 스킬의 단·고르기 + 큰 기술의 단. 안 배운 스킬이 놓인 칸은 단 0 — 빈 칸이다(0166).
    #[test]
    fn the_loadout_carries_the_worn_ranks_and_an_unlearned_slot_is_empty() {
        let mut b = Board::default();
        b.rank[..4].copy_from_slice(&[3, 1, 0, 5]);
        b.pick[Id::Whirl.index()] = Pick::B;
        b.ult[0] = 2;
        let l = b.loadout([Id::Quake, Id::Whirl], Ult::Upend);
        assert_eq!(
            l.slots,
            [
                Slot {
                    id: Id::Quake,
                    rank: 5,
                    pick: Pick::A
                },
                Slot {
                    id: Id::Whirl,
                    rank: 3,
                    pick: Pick::B
                }
            ]
        );
        assert_eq!((l.ult, l.great), (2, Ult::Upend));
        // 끼운 큰 기술이 안 배운 것이면 쟁반 뒤집기로, 배웠으면 그것과 그 단.
        assert_eq!(b.loadout(START, Ult::Line).great, Ult::Upend);
        b.ult[Ult::Line.index()] = 3;
        let l = b.loadout(START, Ult::Line);
        assert_eq!((l.great, l.ult), (Ult::Line, 3));
        let l = b.loadout([Id::Gather, Id::Whirl], Ult::Upend);
        assert_eq!(
            (l.slots[0].id, l.slots[0].rank),
            (Id::Gather, 0),
            "unlearned: an empty slot"
        );
        assert_eq!(l.slots[1].rank, 3);
        let plain: Loadout = [Id::Cut, Id::Gather].into();
        assert!(plain.slots.iter().all(|s| s.rank == 1) && plain.ult == 1);
    }

    #[test]
    fn two_slots_never_hold_the_same_skill() {
        assert_eq!(sane([Id::Cut, Id::Quake]), [Id::Cut, Id::Quake]);
        assert_eq!(sane([Id::Cut, Id::Cut]), [Id::Cut, Id::Whirl]);
        assert_eq!(sane([Id::Whirl, Id::Whirl]), [Id::Whirl, Id::Cut]);
    }
}
