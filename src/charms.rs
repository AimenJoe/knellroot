//! 장신구 표(0064 M3, `docs/design/items.md` 3): 한 줄 = (기록 키, 이름, 설명, 보정). 세 칸에 낀 것이 `Stats`에 더해진다.
//! 기록 키(`key`)는 바꾸지 않는다(save.rs). 새 장신구 = `Id` 한 줄 + 표 한 줄 + 글 둘.
use crate::i18n::{Strings, Tr};
use crate::stats::Mods;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Id {
    Shell,
    Feet,
    Ear,
    Hunger,
    Hand,
    Pouch,
    /// 떠난 마을(0127 C6): 꿰맨 천 · 물 받는 항아리(상자) · 깨진 종 조각(종지기 첫 승리).
    Cloth,
    Jar,
    BellShard,
}

/// 낄 수 있는 칸 수.
pub const SLOTS: usize = 3;

pub struct Def {
    pub id: Id,
    /// 기록 키(바꾸지 않는다).
    pub key: &'static str,
    pub name: Tr,
    pub desc: Tr,
    /// 세계 안의 말 한 줄(0129 C4 — 출처와 기능을 잇는다, lore ecology "장신구 = 옛 흔적"). 장비 창 설명 아래 옅게.
    pub flavor: Tr,
    pub mods: Mods,
}

/// 표 순서 = 보여 주는 순서.
pub static TABLE: [Def; 9] = [
    Def {
        id: Id::Shell,
        key: "shell",
        name: |s: &Strings| s.charm_shell,
        desc: |s: &Strings| s.charm_shell_d,
        flavor: |s: &Strings| s.charm_shell_f,
        mods: Mods {
            hp: 10,
            flask: 0,
            heal: 0,
            dmg: 0,
            taken: 0,
            atk_speed: 1.0,
            dodge_cost: 1.0,
            parry: 0.0,
            kill_stamina: 0.0,
            ember: 1.0,
            stamina_max: 0.0,
            stamina_regen: 1.0,
            poise: 1.0,
            heavy_dmg: 0,
            light_dmg: 0,
            opened_dmg: 0,
        },
    },
    Def {
        id: Id::Feet,
        key: "feet",
        name: |s: &Strings| s.charm_feet,
        desc: |s: &Strings| s.charm_feet_d,
        flavor: |s: &Strings| s.charm_feet_f,
        mods: Mods {
            hp: 0,
            flask: 0,
            heal: 0,
            dmg: 0,
            taken: 10,
            atk_speed: 1.0,
            dodge_cost: 0.8,
            parry: 0.0,
            kill_stamina: 0.0,
            ember: 1.0,
            stamina_max: 0.0,
            stamina_regen: 1.0,
            poise: 1.0,
            heavy_dmg: 0,
            light_dmg: 0,
            opened_dmg: 0,
        },
    },
    Def {
        id: Id::Ear,
        key: "ear",
        name: |s: &Strings| s.charm_ear,
        desc: |s: &Strings| s.charm_ear_d,
        flavor: |s: &Strings| s.charm_ear_f,
        mods: Mods {
            hp: 0,
            flask: 0,
            heal: 0,
            dmg: 0,
            taken: 0,
            atk_speed: 1.0,
            dodge_cost: 1.0,
            parry: 0.04,
            kill_stamina: 0.0,
            ember: 1.0,
            stamina_max: 0.0,
            stamina_regen: 1.0,
            poise: 1.0,
            heavy_dmg: 0,
            light_dmg: 0,
            opened_dmg: 0,
        },
    },
    Def {
        id: Id::Hunger,
        key: "hunger",
        name: |s: &Strings| s.charm_hunger,
        desc: |s: &Strings| s.charm_hunger_d,
        flavor: |s: &Strings| s.charm_hunger_f,
        mods: Mods {
            hp: 0,
            flask: -1,
            heal: 0,
            dmg: 0,
            taken: 0,
            atk_speed: 1.0,
            dodge_cost: 1.0,
            parry: 0.0,
            kill_stamina: 25.0,
            ember: 1.0,
            stamina_max: 0.0,
            stamina_regen: 1.0,
            poise: 1.0,
            heavy_dmg: 0,
            light_dmg: 0,
            opened_dmg: 0,
        },
    },
    Def {
        id: Id::Hand,
        key: "hand",
        name: |s: &Strings| s.charm_hand,
        desc: |s: &Strings| s.charm_hand_d,
        flavor: |s: &Strings| s.charm_hand_f,
        mods: Mods {
            hp: 0,
            flask: 0,
            heal: 0,
            dmg: 0,
            taken: 0,
            atk_speed: 1.0,
            dodge_cost: 1.0,
            parry: 0.0,
            kill_stamina: 0.0,
            ember: 1.0,
            stamina_max: 0.0,
            stamina_regen: 1.0,
            poise: 1.0,
            heavy_dmg: 10,
            light_dmg: 0,
            opened_dmg: 0,
        },
    },
    Def {
        id: Id::Pouch,
        key: "pouch",
        name: |s: &Strings| s.charm_pouch,
        desc: |s: &Strings| s.charm_pouch_d,
        flavor: |s: &Strings| s.charm_pouch_f,
        mods: Mods {
            hp: 0,
            flask: 0,
            heal: 0,
            dmg: 0,
            taken: 0,
            atk_speed: 1.0,
            dodge_cost: 1.0,
            parry: 0.0,
            kill_stamina: 0.0,
            ember: 1.25,
            stamina_max: 0.0,
            stamina_regen: 1.0,
            poise: 1.0,
            heavy_dmg: 0,
            light_dmg: 0,
            opened_dmg: 0,
        },
    },
    Def {
        id: Id::Cloth,
        key: "cloth",
        name: |s: &Strings| s.charm_cloth,
        desc: |s: &Strings| s.charm_cloth_d,
        flavor: |s: &Strings| s.charm_cloth_f,
        mods: Mods {
            hp: 10,
            flask: 0,
            heal: 0,
            dmg: 0,
            taken: 0,
            atk_speed: 1.0,
            dodge_cost: 1.0,
            parry: 0.0,
            kill_stamina: 0.0,
            ember: 1.0,
            stamina_max: 0.0,
            stamina_regen: 0.9,
            poise: 1.0,
            heavy_dmg: 0,
            light_dmg: 0,
            opened_dmg: 0,
        },
    },
    Def {
        id: Id::Jar,
        key: "jar",
        name: |s: &Strings| s.charm_jar,
        desc: |s: &Strings| s.charm_jar_d,
        flavor: |s: &Strings| s.charm_jar_f,
        mods: Mods {
            hp: 0,
            flask: -1,
            heal: 10,
            dmg: 0,
            taken: 0,
            atk_speed: 1.0,
            dodge_cost: 1.0,
            parry: 0.0,
            kill_stamina: 0.0,
            ember: 1.0,
            stamina_max: 0.0,
            stamina_regen: 1.0,
            poise: 1.0,
            heavy_dmg: 0,
            light_dmg: 0,
            opened_dmg: 0,
        },
    },
    Def {
        id: Id::BellShard,
        key: "bellshard",
        name: |s: &Strings| s.charm_bellshard,
        desc: |s: &Strings| s.charm_bellshard_d,
        flavor: |s: &Strings| s.charm_bellshard_f,
        mods: Mods {
            hp: 0,
            flask: 0,
            heal: 0,
            dmg: 0,
            taken: 0,
            atk_speed: 1.0,
            dodge_cost: 1.0,
            parry: 0.0,
            kill_stamina: 0.0,
            ember: 1.0,
            stamina_max: 0.0,
            stamina_regen: 1.0,
            poise: 1.0,
            heavy_dmg: 0,
            light_dmg: 0,
            opened_dmg: 10,
        },
    },
];

pub fn def(id: Id) -> &'static Def {
    TABLE
        .iter()
        .find(|d| d.id == id)
        .expect("every charm has a row")
}

pub fn by_key(key: &str) -> Option<Id> {
    TABLE.iter().find(|d| d.key == key).map(|d| d.id)
}

/// 낀 장신구들의 보정.
pub fn mods(worn: &[Option<Id>; SLOTS]) -> Vec<Mods> {
    worn.iter().flatten().map(|&id| def(id).mods).collect()
}

/// 기록 글: 가진 것 = "shell,ear", 낀 것 = "shell,,ear"(빈 칸은 비움).
pub fn encode_owned(owned: &[Id]) -> String {
    owned
        .iter()
        .map(|&i| def(i).key)
        .collect::<Vec<_>>()
        .join(",")
}

pub fn decode_owned(s: &str) -> Vec<Id> {
    let mut v: Vec<Id> = s.split(',').filter_map(|k| by_key(k.trim())).collect();
    v.sort();
    v.dedup();
    v
}

pub fn encode_worn(worn: &[Option<Id>; SLOTS]) -> String {
    worn.iter()
        .map(|w| w.map_or("", |i| def(i).key))
        .collect::<Vec<_>>()
        .join(",")
}

/// 낀 것은 가진 것만, 같은 것은 한 칸에만.
pub fn decode_worn(s: &str, owned: &[Id]) -> [Option<Id>; SLOTS] {
    let mut out = [None; SLOTS];
    for (slot, k) in out.iter_mut().zip(s.split(',')) {
        *slot = by_key(k.trim()).filter(|i| owned.contains(i));
    }
    for i in 1..SLOTS {
        if out[..i].contains(&out[i]) && out[i].is_some() {
            out[i] = None;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_has_unique_keys_and_every_id_once() {
        let mut keys: Vec<&str> = TABLE.iter().map(|d| d.key).collect();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), TABLE.len());
        for d in &TABLE {
            assert_eq!(def(d.id).key, d.key);
            assert_eq!(by_key(d.key), Some(d.id));
        }
    }

    #[test]
    fn owned_and_worn_round_trip_and_bad_records_are_cleaned() {
        let owned = vec![Id::Shell, Id::Ear, Id::Pouch];
        assert_eq!(decode_owned(&encode_owned(&owned)), owned);
        let worn = [Some(Id::Ear), None, Some(Id::Shell)];
        assert_eq!(decode_worn(&encode_worn(&worn), &owned), worn);
        // 가지지 않은 것·모르는 키·같은 것 두 칸은 빈다.
        assert_eq!(
            decode_worn("hand,nope,ear", &owned),
            [None, None, Some(Id::Ear)]
        );
        assert_eq!(decode_worn("ear,ear,", &owned), [Some(Id::Ear), None, None]);
        assert_eq!(decode_owned("ear,ear,,zzz,shell"), vec![Id::Shell, Id::Ear]);
    }

    #[test]
    fn every_charm_trades_or_helps_and_the_tradeoffs_are_real() {
        use crate::stats::Stats;
        let base = Stats::default();
        for d in &TABLE {
            let s = Stats::compute(&[d.mods]);
            assert_ne!(s, base, "{:?} changes something", d.id);
        }
        let feet = Stats::compute(&[def(Id::Feet).mods]);
        assert!(feet.dodge_cost < 1.0 && feet.taken == 10);
        let hunger = Stats::compute(&[def(Id::Hunger).mods]);
        assert!(hunger.kill_stamina > 0.0 && hunger.flask == base.flask - 1);
        // 무거운 손(0127, 0126 모의 — 평타 +한 눈금(옛 +1)이 싸움 시간을 반으로 줄였다): 강타(패링 반격 포함)에만.
        let hand = Stats::compute(&[def(Id::Hand).mods]);
        assert!(hand.heavy_dmg == 10 && hand.dmg == 0 && hand.atk_speed == 1.0);
        // 떠난 마을(0127): 맞바꿈 둘 + 열린 틈.
        let cloth = Stats::compute(&[def(Id::Cloth).mods]);
        assert!(cloth.max_hp == base.max_hp + 10 && cloth.stamina_regen < 1.0);
        let jar = Stats::compute(&[def(Id::Jar).mods]);
        assert!(jar.heal == 10 && jar.flask == base.flask - 1);
        assert_eq!(Stats::compute(&[def(Id::BellShard).mods]).opened_dmg, 10);
        // 0129 C4: 모든 장신구에 세계 안의 말 한 줄 — 수치 글과 다르다.
        for d in &TABLE {
            let s = crate::i18n::s();
            assert!(
                !(d.flavor)(s).is_empty() && (d.flavor)(s) != (d.desc)(s),
                "{:?}",
                d.id
            );
        }
    }
}
