//! 방패(0223 — 사장 "한손무기는 방패 착용 가능" · "쥐면 막기"): 왼손에 드는 것 하나. 한손 무기(검·맨손)를 들었을 때만 쓰인다 —
//! 패링 버튼을 누르는 순간은 누구나 하는 패링 그대로, 쥐고 있으면 방패 뒤에 선다(`world::PState::Block`): 앞에서 오는 받을 수 있는
//! 공격을 체력 대신 기력으로 받고, 기력이 바닥나면 막기가 깨진다.
//!
//! **패링 창은 건드리지 않는다**(0095 원칙 2 — 방어는 손). 방패가 바꾸는 것은 받는 값(`cost` — 한 대의 기력에 곱한다)뿐.
//! 가진 것은 기록 `shields`(키 목록), 든 것은 `shield.worn`(키). 새 방패 = `Id` 한 줄 + 표 한 줄 + 글 둘 + 모델 부위.
use crate::i18n::{Strings, Tr};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Id {
    /// 순례자의 방패 — 화로의 가게에서 산다(값은 `growth::SHOP`).
    Pilgrim,
}

pub struct Def {
    pub id: Id,
    /// 기록 키(바꾸지 않는다).
    pub key: &'static str,
    /// 방랑자 모델의 부위 이름(왼손 뼈의 단단한 조각 — 든 것만 보인다).
    pub node: &'static str,
    pub name: Tr,
    pub desc: Tr,
    /// 한 대를 받는 기력(`tuning::BLOCK_STAMINA`)에 곱한다 — 작을수록 잘 받는다.
    pub cost: f32,
}

pub static TABLE: [Def; 1] = [Def {
    id: Id::Pilgrim,
    key: "pilgrim",
    node: "shield",
    name: |s: &Strings| s.shield_pilgrim,
    desc: |s: &Strings| s.shield_pilgrim_d,
    cost: 1.0,
}];

impl Id {
    pub fn def(self) -> &'static Def {
        TABLE
            .iter()
            .find(|d| d.id == self)
            .expect("every shield is in the table")
    }

    pub fn key(self) -> &'static str {
        self.def().key
    }

    pub fn by_key(key: &str) -> Option<Id> {
        TABLE.iter().find(|d| d.key == key).map(|d| d.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_shield_has_its_own_key_and_a_sane_cost() {
        for (i, d) in TABLE.iter().enumerate() {
            assert_eq!(Id::by_key(d.key), Some(d.id));
            assert!(TABLE[..i].iter().all(|o| o.key != d.key), "{}", d.key);
            assert!(d.cost > 0.3 && d.cost <= 1.5, "{}: {}", d.key, d.cost);
        }
    }
}
