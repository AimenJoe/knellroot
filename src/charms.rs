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
    /// 고요의 종(0270 D — 성소의 필드 보스 고요한 사슴을 처음 쓰러뜨리면): 스킬 대기가 짧아진다.
    StillBell,
}

/// 낄 수 있는 칸 수.
pub const SLOTS: usize = 3;

/// 장신구의 종류(0224 — 사장 "조합이 가능/불가능할 수 있도록 목걸이·반지 등등 나눠야"): 칸마다 종류 하나, 장신구는 제 종류의
/// 칸에만 낀다 — 같은 종류 둘은 함께 낄 수 없다. 묶음은 하는 일로 갈랐다(같은 일을 겹쳐 쌓지 못하게):
/// 목걸이 = 살림과 감각(패링 창·등잔·잔불) · 반지 = 손의 힘(강타·열린 틈·쓰러뜨림의 기력) · 팔찌 = 몸(체력·회피).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Neck,
    Ring,
    Bracelet,
}

/// 칸의 종류(칸 번호 = 기록 `charm.worn`의 자리).
pub const SLOT_KINDS: [Kind; SLOTS] = [Kind::Neck, Kind::Ring, Kind::Bracelet];

impl Kind {
    /// 이 종류의 칸.
    pub fn slot(self) -> usize {
        self as usize
    }

    pub fn name(self) -> Tr {
        match self {
            Kind::Neck => |s| s.charm_kind_neck,
            Kind::Ring => |s| s.charm_kind_ring,
            Kind::Bracelet => |s| s.charm_kind_bracelet,
        }
    }
}

impl Id {
    /// 종류(`_` 없는 match — 새 장신구는 여기서 종류를 정한다).
    pub fn kind(self) -> Kind {
        match self {
            // 고요의 종(0270 D)도 목걸이 — 목에 거는 종이고, 다른 목걸이(패링 창·등잔·잔불)와 하는 일이 겹치지 않는다.
            Id::Ear | Id::Jar | Id::Pouch | Id::StillBell => Kind::Neck,
            Id::Hand | Id::BellShard | Id::Hunger => Kind::Ring,
            Id::Shell | Id::Cloth | Id::Feet => Kind::Bracelet,
        }
    }
}

/// 낀 것을 제 종류의 칸에 앉힌다(0224 — 옛 기록의 낀 것을 옮긴다): 같은 종류가 둘이면 앞 칸의 것이 남고 뒤의 것은 벗는다
/// (가진 것에는 그대로 있다).
pub fn seat(worn: [Option<Id>; SLOTS]) -> [Option<Id>; SLOTS] {
    let mut out = [None; SLOTS];
    for id in worn.into_iter().flatten() {
        let k = id.kind().slot();
        if out[k].is_none() {
            out[k] = Some(id);
        }
    }
    out
}

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
pub static TABLE: [Def; 10] = [
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
            knock: 1.0,
            flinch: 1.0,
            shot_dmg: 0,
            mag: 0,
            skill_cd: 1.0,
            ail: 1.0,
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
            knock: 1.0,
            flinch: 1.0,
            shot_dmg: 0,
            mag: 0,
            skill_cd: 1.0,
            ail: 1.0,
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
            knock: 1.0,
            flinch: 1.0,
            shot_dmg: 0,
            mag: 0,
            skill_cd: 1.0,
            ail: 1.0,
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
            knock: 1.0,
            flinch: 1.0,
            shot_dmg: 0,
            mag: 0,
            skill_cd: 1.0,
            ail: 1.0,
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
            knock: 1.0,
            flinch: 1.0,
            shot_dmg: 0,
            mag: 0,
            skill_cd: 1.0,
            ail: 1.0,
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
            knock: 1.0,
            flinch: 1.0,
            shot_dmg: 0,
            mag: 0,
            skill_cd: 1.0,
            ail: 1.0,
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
            knock: 1.0,
            flinch: 1.0,
            shot_dmg: 0,
            mag: 0,
            skill_cd: 1.0,
            ail: 1.0,
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
            knock: 1.0,
            flinch: 1.0,
            shot_dmg: 0,
            mag: 0,
            skill_cd: 1.0,
            ail: 1.0,
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
            knock: 1.0,
            flinch: 1.0,
            shot_dmg: 0,
            mag: 0,
            skill_cd: 1.0,
            ail: 1.0,
        },
    },
    // 고요의 종(0270 D): 스킬 대기 −10% — 장신구 가운데 스킬 대기를 건드리는 것은 이것뿐이다(아이템 옵션 `affix_cd` 한 줄의
    // 가장 큰 단과 같은 크기). 받는 피해·패링 창·회피는 건드리지 않는다.
    Def {
        id: Id::StillBell,
        key: "stillbell",
        name: |s: &Strings| s.charm_stillbell,
        desc: |s: &Strings| s.charm_stillbell_d,
        flavor: |s: &Strings| s.charm_stillbell_f,
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
            opened_dmg: 0,
            knock: 1.0,
            flinch: 1.0,
            shot_dmg: 0,
            mag: 0,
            skill_cd: 0.9,
            ail: 1.0,
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

/// 낀 것은 가진 것만, 같은 것은 한 칸에만, 제 종류의 칸에만.
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
    // 제 종류의 칸으로(0224 — 옛 기록은 아무 칸에나 끼워져 있다).
    seat(out)
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
            [Some(Id::Ear), None, None],
            "the ear is a necklace: it moves to the necklace slot"
        );
        assert_eq!(decode_worn("ear,ear,", &owned), [Some(Id::Ear), None, None]);
        assert_eq!(decode_owned("ear,ear,,zzz,shell"), vec![Id::Shell, Id::Ear]);
    }

    /// 0224: 종류는 셋, 종류마다 칸 하나, 장신구는 종류마다 고르게 — 같은 종류 둘은 함께 못 낀다(앞 칸의 것이 남는다).
    /// 같은 일을 하는 것끼리 한 종류다: 체력을 주는 둘(껍질·천)·피해를 더하는 둘(손·종 조각)은 겹쳐 쌓지 못한다.
    /// 0270 D: 장신구가 열이 되어(고요의 종) 셋씩 꼭 같을 수 없다 — 종류끼리의 수는 하나까지만 다르다.
    #[test]
    fn charms_have_kinds_and_only_one_of_a_kind_is_worn() {
        let count = |k: &Kind| TABLE.iter().filter(|d| d.id.kind() == *k).count();
        for (i, k) in SLOT_KINDS.iter().enumerate() {
            assert_eq!(k.slot(), i);
            assert!(count(k) >= 3, "{k:?}");
        }
        let (most, least) = (
            SLOT_KINDS.iter().map(count).max().unwrap(),
            SLOT_KINDS.iter().map(count).min().unwrap(),
        );
        assert!(most - least <= 1, "kinds stay even: {most} vs {least}");
        assert_eq!(Id::StillBell.kind(), Kind::Neck);
        assert_eq!(Id::Shell.kind(), Id::Cloth.kind());
        assert_eq!(Id::Hand.kind(), Id::BellShard.kind());
        assert_ne!(
            Id::Ear.kind(),
            Id::BellShard.kind(),
            "parry and its payoff still combine"
        );
        assert_eq!(
            seat([Some(Id::Shell), Some(Id::Cloth), Some(Id::Ear)]),
            [Some(Id::Ear), None, Some(Id::Shell)]
        );
        assert_eq!(
            seat([None, Some(Id::Hand), None]),
            [None, Some(Id::Hand), None]
        );
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
        // 고요의 종(0270 D): 스킬 대기만 — 받는 피해·패링 창·회피는 그대로(0095 원칙 2 · 0176), 다른 장신구는 스킬 대기를 안 건드린다.
        let bell = Stats::compute(&[def(Id::StillBell).mods]);
        assert!(bell.skill_cd < 1.0 && bell.taken == 0 && bell.parry == base.parry);
        assert_eq!(bell.dodge_cost, base.dodge_cost);
        assert!(TABLE
            .iter()
            .filter(|d| d.id != Id::StillBell)
            .all(|d| d.mods.skill_cd == 1.0));
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
