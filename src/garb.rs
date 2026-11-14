//! 덧옷(0176): 몸에 걸치는 것 하나 — 줍는 물건(지도 `M` 숨은 물건 `stage::Find::Garb`)이고, 칸은 하나(장비 탭 다섯째 칸).
//! 사장 "방어구는 아이템이고 줍는거지, 거기에 뭐가 효과가 있을수도 있고. 다만, lore 에 맞게 너무 rpg 식은 말고"(2026-11-14).
//!
//! 그래서 방어력·받는 피해 숫자는 없다 — 덧옷은 **맞을 때 몸이 어떻게 되는가**(밀림 `knock`·휘청임 `flinch`)와 그 무게(구르기 기력)를
//! 바꾼다. 설명은 말로만(숫자는 장비 탭이 견줄 때만 보인다). 세계관: 떠난 사람들이 남긴 옷 — 누가 입었는지는 말하지 않는다.
//!
//! 가졌는지는 기록의 주운 숨은 물건(`finds` — 이름 `<스테이지>.garb.<n>`)으로 센다(조각과 같은 꼴 — 따로 가진 목록이 없다).
//! 걸친 것은 기록 `garb.worn`(키). 새 덧옷 = `Id` 한 줄 + 표 한 줄 + 지도 `M` 하나(그 스테이지의 n째) + 글 셋 + 모델 부위.
//! 몸에 걸친 모습은 아직 그리지 않는다(0157 — 생김새는 그림 먼저, 사장 확인 뒤).
use crate::i18n::{Strings, Tr};
use crate::stats::Mods;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Id {
    /// 해진 순례 망토 — 가볍다.
    Cloak,
    /// 뿌리껍질 덧옷 — 무겁다.
    Bark,
    /// 그을린 가죽 앞치마 — 덜 휘청인다.
    Apron,
}

pub struct Def {
    pub id: Id,
    /// 기록 키(바꾸지 않는다).
    pub key: &'static str,
    /// 숨은 물건 기록 이름(`stage::find_id` — 그 스테이지 지도의 몇째 `M`).
    pub find: &'static str,
    /// 모델 부위(`charms.glb`, `item_view`).
    pub node: &'static str,
    pub name: Tr,
    pub desc: Tr,
    pub flavor: Tr,
    /// 놓인 곳(도감 — 아직 못 찾았을 때도 보인다, 장신구의 `codex::charm_place`와 같은 결).
    pub place: Tr,
    pub mods: Mods,
}

/// 표 순서 = 보여 주는 순서(찾는 차례 — 성소 성벽길 · 성소 뿌리 굴 · 떠난 마을 거주지).
pub static TABLE: [Def; 3] = [
    Def {
        id: Id::Cloak,
        key: "cloak",
        find: "sanctuary.garb.0",
        node: "garb_cloak",
        name: |s: &Strings| s.garb_cloak,
        desc: |s: &Strings| s.garb_cloak_d,
        flavor: |s: &Strings| s.garb_cloak_f,
        place: |s: &Strings| s.cx_garb_cloak,
        mods: Mods {
            dodge_cost: 0.85,
            stamina_regen: 1.1,
            knock: 1.35,
            flinch: 1.2,
            ..MODS0
        },
    },
    Def {
        id: Id::Bark,
        key: "bark",
        find: "sanctuary.garb.1",
        node: "garb_bark",
        name: |s: &Strings| s.garb_bark,
        desc: |s: &Strings| s.garb_bark_d,
        flavor: |s: &Strings| s.garb_bark_f,
        place: |s: &Strings| s.cx_garb_bark,
        mods: Mods {
            dodge_cost: 1.3,
            knock: 0.45,
            flinch: 0.55,
            ..MODS0
        },
    },
    Def {
        id: Id::Apron,
        key: "apron",
        find: "village.garb.0",
        node: "garb_apron",
        name: |s: &Strings| s.garb_apron,
        desc: |s: &Strings| s.garb_apron_d,
        flavor: |s: &Strings| s.garb_apron_f,
        place: |s: &Strings| s.cx_garb_apron,
        mods: Mods {
            dodge_cost: 1.1,
            knock: 0.8,
            flinch: 0.6,
            ..MODS0
        },
    },
];

/// 아무것도 안 바꾸는 보정(표의 `..` 자리 — `Mods::default()`는 const가 아니다).
const MODS0: Mods = Mods {
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
};

pub fn def(id: Id) -> &'static Def {
    TABLE
        .iter()
        .find(|d| d.id == id)
        .expect("every garment is in the table")
}

pub fn by_key(k: &str) -> Option<Id> {
    TABLE.iter().find(|d| d.key == k).map(|d| d.id)
}

/// 숨은 물건 이름 → 덧옷.
pub fn of_find(find: &str) -> Option<Id> {
    TABLE.iter().find(|d| d.find == find).map(|d| d.id)
}

/// 가진 덧옷(표 순서) — 주운 숨은 물건에서.
pub fn owned(finds: &[String]) -> Vec<Id> {
    TABLE
        .iter()
        .filter(|d| finds.iter().any(|f| f == d.find))
        .map(|d| d.id)
        .collect()
}

/// 걸친 덧옷의 보정(없으면 아무것도).
pub fn mods(worn: Option<Id>) -> Mods {
    worn.map_or(MODS0, |id| def(id).mods)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::Stats;

    /// 0176: 덧옷은 받는 피해·체력 같은 "방어력" 숫자를 쓰지 않는다 — 몸이 맞음을 받는 꼴(밀림·휘청임)과 무게만 바꾼다.
    /// 그리고 셋은 서로 다른 몸이다(가벼움 = 덜 드는 구르기 + 더 밀림, 무거움 = 덜 밀림 + 더 드는 구르기).
    #[test]
    fn garments_change_how_the_body_takes_a_blow_not_a_defense_number() {
        for d in TABLE.iter() {
            let m = d.mods;
            assert_eq!((m.taken, m.hp, m.dmg), (0, 0, 0), "{}", d.key);
            assert!(
                m.knock != 1.0 || m.flinch != 1.0,
                "{} must change the blow",
                d.key
            );
            // 덜 밀리는 옷은 구르기가 무겁고, 더 밀리는 옷은 가볍다 — 공짜는 없다.
            assert_eq!(
                m.knock < 1.0,
                m.dodge_cost > 1.0,
                "{}: steadier must be heavier",
                d.key
            );
        }
        let s = |id| Stats::compute(&[mods(Some(id))]);
        assert!(s(Id::Bark).knock < s(Id::Apron).knock && s(Id::Apron).knock < s(Id::Cloak).knock);
        assert!(s(Id::Cloak).dodge_cost < s(Id::Apron).dodge_cost);
        assert_eq!(Stats::compute(&[mods(None)]), Stats::default());
    }

    /// 0176: 가진 것은 주운 숨은 물건에서 — 표의 기록 이름은 그 스테이지의 `M` 차례와 맞고(`stage` 시험이 지도를 본다), 키는 겹치지 않는다.
    #[test]
    fn owned_garments_come_from_finds_and_keys_are_unique() {
        assert!(owned(&[]).is_empty());
        let finds = vec![
            "village.garb.0".to_string(),
            "sanctuary.shard.0".to_string(),
        ];
        assert_eq!(owned(&finds), vec![Id::Apron]);
        for (i, a) in TABLE.iter().enumerate() {
            assert_eq!(by_key(a.key), Some(a.id));
            assert_eq!(of_find(a.find), Some(a.id));
            for b in TABLE.iter().skip(i + 1) {
                assert!(a.key != b.key && a.find != b.find && a.node != b.node);
            }
            assert!(
                a.node.starts_with("garb_"),
                "part names must not collide with charm keys"
            );
        }
    }

    /// 0176: 표의 기록 이름마다 그 스테이지 지도에 `M`이 있고(`<스테이지>.garb.<n>` = 그 지도의 n째 `M`), 지도의 `M`은 다 표에 있다 —
    /// 지도를 고치다 덧옷이 조용히 사라지거나 이름 없는 덧옷이 생기지 않게.
    #[test]
    fn every_garment_lies_on_its_stage_map() {
        use crate::stage::{find_id, Find, Level, STAGES};
        let mut on_maps = 0;
        for def in STAGES.iter() {
            let n = Level::parse(def.map).marks_of(Find::Garb.glyph()).count();
            on_maps += n;
            for k in 0..n {
                let id = find_id(def.find_prefix, Find::Garb, k);
                assert!(of_find(&id).is_some(), "{id} on the map has no garment");
            }
        }
        assert_eq!(on_maps, TABLE.len());
    }
}
