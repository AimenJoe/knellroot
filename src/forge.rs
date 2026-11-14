//! 무기 강화 — 벼리기(0180, v0.6.1 W1): 무기마다 세 단, 값은 **숫돌**(정해진 수만 지도에 놓인 숨은 물건 `stage::Find::Stone`, 지도 `O`)과
//! 잔불. 화로의 가게 탭에서 벼리고, **거두면 숫돌은 돌아온다**(잔불은 안 돌아온다) — 무기를 바꿔도 쌓은 것에 갇히지 않게
//! (KB `concepts/weapon-upgrade-design.md` — 무기마다 따로 올리면 아끼는 하나에 갇힌다).
//!
//! 한 단의 크기는 "잡몹을 쓰러뜨리는 대 수가 하나 준다"(근접 타 +5 = 반 눈금 — +10은 보스 모의에서 너무 셌다). 무기마다의 몫은 `weapons::Weapon::forge` 표,
//! 여기는 셈만(순수). 가진 숫돌은 주운 숨은 물건(`finds` — 이름 `<스테이지>.stone.<n>`)에서 세고, 쓴 숫돌은 벼린 단에서 센다
//! (따로 적지 않는다 — 옛 기록·고친 지도에서도 어긋나지 않는다). 판정 창은 넓히지 않는다(0095 원칙 2): 피해·탄창만.
use crate::stats::Mods;
use crate::weapons::{self, Id};

/// 단의 수.
pub const RANKS: u8 = 3;

/// 그 단으로 올리는 값: (숫돌, 잔불). 첫 단은 싸고 가깝게(장치를 일찍 보인다).
pub const COST: [(u32, u32); RANKS as usize] = [(1, 60), (1, 100), (2, 140)];

/// 숨은 물건 기록 이름의 가운데 칸(`stage::Find::Stone::key`와 같다 — 바꾸지 않는다).
pub const STONE_KEY: &str = "stone";

/// 무기마다 벼린 단(`weapons::IDS` 차례). 기록 `forge.<무기 키>` = 단(0이면 쓰지 않는다).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Forged([u8; weapons::IDS.len()]);

fn slot(w: Id) -> usize {
    weapons::IDS.iter().position(|&i| i == w).unwrap_or(0)
}

/// 벼릴 수 있는 무기인가(맨손은 아니다).
pub fn forgeable(w: Id) -> bool {
    w.def().forge.is_some()
}

impl Forged {
    pub fn rank(&self, w: Id) -> u8 {
        self.0[slot(w)]
    }

    pub fn set(&mut self, w: Id, r: u8) {
        if forgeable(w) {
            self.0[slot(w)] = r.min(RANKS);
        }
    }

    /// 벼린 단에 쓴 숫돌.
    pub fn spent(&self) -> u32 {
        weapons::IDS.iter().map(|&w| cost_to(self.rank(w)).0).sum()
    }

    /// 그 무기의 지금 단이 더하는 몫(들고 있는 무기에만 — `save::Progress::stats`).
    pub fn mods(&self, w: Id) -> Mods {
        let r = self.rank(w);
        match (w.def().forge, r) {
            (Some(f), 1..) => {
                let f = f[(r - 1) as usize];
                Mods {
                    light_dmg: f.hit,
                    heavy_dmg: f.hit,
                    shot_dmg: f.shot,
                    mag: f.mag,
                    ..Mods::default()
                }
            }
            _ => Mods::default(),
        }
    }

    /// 가진 숫돌로 셀 수 있게 바로잡는다(쓴 것이 주운 것보다 많으면 높은 단부터 내린다 — 손으로 고친 기록·지도의 변화).
    pub fn sane(mut self, found: u32) -> Forged {
        while self.spent() > found {
            let Some(top) = weapons::IDS
                .iter()
                .copied()
                .filter(|&w| self.rank(w) > 0)
                .max_by_key(|&w| self.rank(w))
            else {
                break;
            };
            let r = self.rank(top);
            self.set(top, r - 1);
        }
        self
    }
}

/// 0단에서 `r`단까지 드는 (숫돌, 잔불).
pub fn cost_to(r: u8) -> (u32, u32) {
    COST.iter()
        .take(r.min(RANKS) as usize)
        .fold((0, 0), |a, c| (a.0 + c.0, a.1 + c.1))
}

/// 주운 숫돌의 수.
pub fn stones(finds: &[String]) -> u32 {
    finds
        .iter()
        .filter(|id| id.split('.').nth(1) == Some(STONE_KEY))
        .count() as u32
}

/// 쓸 수 있는 숫돌(주운 것 − 벼린 단에 쓴 것).
pub fn free(finds: &[String], forged: &Forged) -> u32 {
    stones(finds).saturating_sub(forged.spent())
}

/// 다음 단으로 벼리는 값(다 벼렸거나 벼릴 수 없는 무기면 없음).
pub fn next_cost(forged: &Forged, w: Id) -> Option<(u32, u32)> {
    let r = forged.rank(w);
    (forgeable(w) && r < RANKS).then(|| COST[r as usize])
}

/// 지금 벼릴 수 있나: 숫돌과 잔불이 다 있다.
pub fn can_raise(finds: &[String], forged: &Forged, embers: u32, w: Id) -> bool {
    next_cost(forged, w).is_some_and(|(st, em)| free(finds, forged) >= st && embers >= em)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stones_n(n: usize) -> Vec<String> {
        (0..n).map(|i| format!("sanctuary.stone.{i}")).collect()
    }

    /// 0180: 벼리면 숫돌·잔불이 들고, 거두면(0단으로) 숫돌은 돌아온다 — 다른 무기를 같은 숫돌로 벼릴 수 있다(갇히지 않는다).
    #[test]
    fn forging_spends_stones_and_taking_back_returns_them() {
        let finds = stones_n(4);
        let mut f = Forged::default();
        assert_eq!(free(&finds, &f), 4);
        assert_eq!(next_cost(&f, Id::Sword), Some(COST[0]));
        f.set(Id::Sword, 3);
        assert_eq!(f.spent(), 4);
        assert_eq!(free(&finds, &f), 0);
        assert!(!can_raise(&finds, &f, 999, Id::Hammer), "no stones left");
        assert_eq!(next_cost(&f, Id::Sword), None, "fully forged");
        f.set(Id::Sword, 0);
        assert_eq!(free(&finds, &f), 4, "taking back returns every stone");
        assert!(can_raise(&finds, &f, 60, Id::Hammer));
        assert!(!can_raise(&finds, &f, 59, Id::Hammer), "embers too");
        // 맨손은 벼리지 않는다.
        assert!(!forgeable(Id::Fists) && next_cost(&f, Id::Fists).is_none());
        f.set(Id::Fists, 2);
        assert_eq!(f.rank(Id::Fists), 0);
    }

    /// 0180: 근접 무기의 단은 타마다 +5씩(대 수가 하나 준다), 쇠뇌는 탄창부터 — 판정 창·기력은 건드리지 않는다.
    #[test]
    fn ranks_add_hit_damage_for_melee_and_magazine_for_the_crossbow() {
        let mut f = Forged::default();
        for r in 1..=RANKS {
            f.set(Id::Sword, r);
            let m = f.mods(Id::Sword);
            assert_eq!((m.light_dmg, m.heavy_dmg), (5 * r as i32, 5 * r as i32));
            assert_eq!((m.parry, m.dodge_cost, m.atk_speed), (0.0, 1.0, 1.0));
        }
        f.set(Id::Crossbow, 1);
        let m = f.mods(Id::Crossbow);
        assert_eq!(
            (m.light_dmg, m.mag, m.shot_dmg),
            (0, 1, 0),
            "magazine first"
        );
        f.set(Id::Crossbow, 3);
        assert!(f.mods(Id::Crossbow).shot_dmg > 0);
        assert_eq!(Forged::default().mods(Id::Sword), Mods::default());
    }

    /// 0180: 쓴 숫돌이 주운 것보다 많은 기록은 높은 단부터 내려 맞춘다.
    #[test]
    fn a_record_that_spent_more_than_it_found_is_brought_down() {
        let mut f = Forged::default();
        f.set(Id::Sword, 3);
        f.set(Id::Hammer, 1);
        let f = f.sane(3);
        assert!(f.spent() <= 3);
        assert_eq!(f.rank(Id::Hammer), 1, "the lower one stays");
        assert_eq!(f.rank(Id::Sword), 2);
    }
}
