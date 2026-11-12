//! 성장(0064 M4, `docs/design/items.md` 0·F): 판을 넘어 남는 큰 보상.
//!
//! - 핵 고르기(문지기를 이길 때마다 체력·등잔·한 모금 중 하나)는 0105에서 걷었다 — 체력은 레벨 생명, 등잔·한 모금은 숨은 물건,
//!   핵은 기술 나무 점수(`progression.rs`). 옛 기록의 몫은 `save::migrate_v04`가 옮기고 남는 것만 여기.
//! - **쉼터 화로의 가게**: 잔불로 사는 것 — 상자·문지기에 없는 장신구 셋과 등잔 기름(등잔 +1회, 한 번). 쓸 곳이 생기면
//!   쉼터가 빛난다(KB `roguelite-healing-and-currency` "쓸 곳이 보여야").
//!
//! 능력치는 `Growth::mods`가 `Stats`에 더한다(장신구와 같은 합치기).
use crate::charms::Id as Charm;
use crate::stats::Mods;

/// 쌓인 성장(0064 M4). 핵 고르기는 0105(v0.4.0 G3)에서 걷었다 — `hp`·`flask`·`heal`은 옛 기록을 옮기고 **남은 몫**
/// (생명 5단을 넘은 체력, 숨은 등잔 하나를 넘은 등잔)만 들고 있다(잃지 않게, `save::migrate_v04`). 새로 늘지 않는다.
/// `hp`·`heal`은 **몫의 수**(기록 `grow.hp`·`grow.heal` 그대로) — 한 몫이 체력·회복 한 눈금(`HP_UNIT` = 10, 0158 N0).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Growth {
    pub hp: u8,
    pub flask: u8,
    pub heal: u8,
    /// 등잔 기름(가게, 한 번).
    pub oil: bool,
}

impl Growth {
    pub fn mods(&self) -> Mods {
        Mods {
            hp: self.hp as i32 * crate::tuning::HP_UNIT,
            flask: self.flask as i32 + self.oil as i32,
            heal: self.heal as i32 * crate::tuning::HP_UNIT,
            ..Mods::default()
        }
    }
}

/// 가게 물건.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ware {
    Charm(Charm),
    /// 등잔 기름: 등잔 +1회(한 번).
    Oil,
}

/// 쉼터 화로의 가게(값은 잔불). 0117: 서 있는 적 39(잔재 2·가시짐승 3)·엘리트·문지기 40으로 한 판 ≈ 150 —
/// 레벨(40 + 20×L)과 나눠 쓰니 한 판에 하나쯤 되게 값을 올렸다(초반에 강해지는 것이 몰리지 않게). 단단한 껍질은 입구 상자에서 옮겨 왔다.
pub const SHOP: [(Ware, u32); 5] = [
    (Ware::Charm(Charm::Pouch), 60),
    (Ware::Charm(Charm::Feet), 90),
    (Ware::Charm(Charm::Shell), 110),
    (Ware::Charm(Charm::Hunger), 120),
    (Ware::Oil, 150),
];

/// 아직 살 수 있는 것(가진 장신구·산 기름은 빠진다).
pub fn wares(owned: &[Charm], g: &Growth) -> Vec<(Ware, u32)> {
    SHOP.into_iter()
        .filter(|(w, _)| match w {
            Ware::Charm(id) => !owned.contains(id),
            Ware::Oil => !g.oil,
        })
        .collect()
}

/// 잔불로 살 수 있는 것이 있나(쉼터가 빛난다).
pub fn can_buy(embers: u32, owned: &[Charm], g: &Growth) -> bool {
    wares(owned, g).iter().any(|&(_, price)| embers >= price)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::Stats;

    #[test]
    fn leftover_growth_still_feeds_the_stats() {
        let g = Growth {
            hp: 1,
            flask: 1,
            heal: 0,
            oil: true,
        };
        let s = Stats::compute(&[g.mods()]);
        let base = Stats::default();
        assert_eq!(
            s.max_hp,
            base.max_hp + 10,
            "one leftover share = one old point"
        );
        assert_eq!(s.flask, base.flask + 2, "leftover lamp + oil");
    }

    #[test]
    fn the_shop_sells_what_you_lack_and_glows_only_when_you_can_pay() {
        let g = Growth::default();
        assert_eq!(wares(&[], &g).len(), SHOP.len());
        // 가진 장신구는 안 판다, 산 기름도.
        let owned = [Charm::Pouch];
        let got = Growth { oil: true, ..g };
        let left = wares(&owned, &got);
        assert!(left
            .iter()
            .all(|(w, _)| *w != Ware::Charm(Charm::Pouch) && *w != Ware::Oil));
        // 가장 싼 것보다 적으면 빛나지 않는다.
        let cheapest = SHOP.iter().map(|s| s.1).min().unwrap();
        assert!(!can_buy(cheapest - 1, &[], &g));
        assert!(can_buy(cheapest, &[], &g));
        // 상자·문지기에서 얻는 장신구는 팔지 않는다(얻는 곳이 겹치지 않게). 등껍질은 0117에서 가게로 옮겼다.
        for id in [
            Charm::Ear,
            Charm::Hand,
            Charm::Cloth,
            Charm::Jar,
            Charm::BellShard,
        ] {
            assert!(SHOP.iter().all(|(w, _)| *w != Ware::Charm(id)));
        }
    }
}
