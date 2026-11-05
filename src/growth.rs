//! 성장(0064 M4, `docs/design/items.md` 0·F): 판을 넘어 남는 큰 보상.
//!
//! - **핵 고르기**: 문지기를 이길 때마다 핵 하나 → 셋 가운데 하나(최대 체력 +1 · 등잔 +1회 · 한 모금 회복 +1), 갈래마다 상한.
//! - **쉼터 화로의 가게**: 잔불로 사는 것 — 상자·문지기에 없는 장신구 셋과 등잔 기름(등잔 +1회, 한 번). 쓸 곳이 생기면
//!   쉼터가 빛난다(KB `roguelite-healing-and-currency` "쓸 곳이 보여야").
//!
//! 능력치는 `Growth::mods`가 `Stats`에 더한다(장신구와 같은 합치기).
use crate::charms::Id as Charm;
use crate::stats::Mods;

/// 핵 고르기의 갈래.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pick {
    Hp,
    Flask,
    Heal,
}

pub const PICKS: [Pick; 3] = [Pick::Hp, Pick::Flask, Pick::Heal];

/// 갈래마다 고를 수 있는 횟수.
pub fn cap(p: Pick) -> u8 {
    match p {
        Pick::Hp => 3,
        Pick::Flask => 2,
        Pick::Heal => 2,
    }
}

/// 쌓인 성장.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Growth {
    pub hp: u8,
    pub flask: u8,
    pub heal: u8,
    /// 등잔 기름(가게, 한 번).
    pub oil: bool,
}

impl Growth {
    pub fn count(&self, p: Pick) -> u8 {
        match p {
            Pick::Hp => self.hp,
            Pick::Flask => self.flask,
            Pick::Heal => self.heal,
        }
    }

    /// 아직 고를 수 있는 갈래(상한에 닿지 않은 것). 비면 핵 고르기를 건너뛴다.
    pub fn options(&self) -> Vec<Pick> {
        PICKS
            .into_iter()
            .filter(|&p| self.count(p) < cap(p))
            .collect()
    }

    pub fn add(&mut self, p: Pick) {
        let c = match p {
            Pick::Hp => &mut self.hp,
            Pick::Flask => &mut self.flask,
            Pick::Heal => &mut self.heal,
        };
        *c = (*c + 1).min(cap(p));
    }

    pub fn mods(&self) -> Mods {
        Mods {
            hp: self.hp as i32,
            flask: self.flask as i32 + self.oil as i32,
            heal: self.heal as i32,
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

/// 쉼터 화로의 가게(값은 잔불). 잔재 2·가시짐승 3·문지기 40이라 한 판(≈ 70)에 하나쯤.
pub const SHOP: [(Ware, u32); 4] = [
    (Ware::Charm(Charm::Pouch), 25),
    (Ware::Charm(Charm::Feet), 35),
    (Ware::Charm(Charm::Hunger), 45),
    (Ware::Oil, 80),
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
    fn core_picks_stop_at_their_caps_and_feed_the_stats() {
        let mut g = Growth::default();
        assert_eq!(g.options(), PICKS.to_vec());
        for _ in 0..10 {
            g.add(Pick::Hp);
        }
        assert_eq!(g.hp, cap(Pick::Hp));
        assert!(!g.options().contains(&Pick::Hp), "capped");
        g.add(Pick::Heal);
        g.oil = true;
        let s = Stats::compute(&[g.mods()]);
        let base = Stats::default();
        assert_eq!(s.max_hp, base.max_hp + cap(Pick::Hp) as i32);
        assert_eq!(s.flask, base.flask + 1, "oil adds a sip");
        assert_eq!(s.heal, 1);
        // 다 고르면 고를 것이 없다(핵 고르기를 건너뛴다).
        for p in PICKS {
            for _ in 0..5 {
                g.add(p);
            }
        }
        assert!(g.options().is_empty());
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
        // 상자·문지기에서 얻는 장신구는 팔지 않는다(얻는 곳이 겹치지 않게).
        for id in [Charm::Shell, Charm::Ear, Charm::Hand] {
            assert!(SHOP.iter().all(|(w, _)| *w != Ware::Charm(id)));
        }
    }
}
