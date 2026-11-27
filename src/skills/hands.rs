//! 스킬이 무엇을 들었을 때 되나 — 종류 · 방패의 비트(0223 V3 → 0272, 0278 R19).
use crate::weapons::{Class, Grip, Hand, CLASSES};

/// 스킬이 "무엇을 들었을 때 되나"(0223 V3 → 0225 → 0272): 비트의 합 — 무기 종류마다 하나(`class_bit`)와 방패(`SHIELD` — 한손 계열과
/// 방패를 함께 든 꼴). 0281부터 한손 종류의 비트는 방패를 들어도 선다: 한손 무기와 방패 = 그 종류의 스킬 **과** 방패 판의 스킬.
pub type Hands = u16;
/// 종류 하나의 비트(`Class::index`의 자리 — 새 종류는 표에 한 줄이면 비트가 생긴다).
pub const fn class_bit(c: Class) -> Hands {
    1 << c.index()
}
/// 한손검(방패가 있든 없든 — 0281).
pub const SWORD: Hands = class_bit(Class::Sword);
pub const HAMMER: Hands = class_bit(Class::Hammer);
pub const STAFF: Hands = class_bit(Class::Staff);
pub const CROSSBOW: Hands = class_bit(Class::Crossbow);
pub const BOW: Hands = class_bit(Class::Bow);
pub const SPEAR: Hands = class_bit(Class::Spear);
/// 쌍단검(0274 ③ — 두 손에 날 하나씩: 방패와 들지 않는다).
pub const DAGGER: Hands = class_bit(Class::Dagger);
/// 대검(0274 ④ — 두 손으로 드는 긴 날: 방패와 들지 않는다).
pub const GREATSWORD: Hands = class_bit(Class::Greatsword);
/// 낫(0274 ⑤ — 한 손의 휜 날: 한손검 · 망치처럼 방패와 든다 — 방패를 들면 낫 판과 방패 판, 0281).
pub const SCYTHE: Hands = class_bit(Class::Scythe);
/// 완드(0274 ⑥ — 한 손에 드는 짧은 막대. 0281부터 한손: 방패와 든다 — RB는 그대로 저주탄, `Quirks::rb_main`).
pub const WAND: Hands = class_bit(Class::Wand);
/// 한손 계열과 방패(방패 판의 기술).
pub const SHIELD: Hands = 1 << 15;
/// 묶음: 한손(방패가 있든 없든 — 0274부터 망치도, ⑤부터 낫도, 0281부터 완드도) · 양손 근접(지팡이 · 창 · 쌍단검 · 대검 — 두 손이 다 찬다) · 근접 ·
/// 원거리 · 어느 것이나.
/// 셋 다 종류 표의 계열(`ClassDef::hand`)에서 선다(0278 R19 — `hand_bits`): 새 종류는 계열을 고르면 그 묶음의 스킬이 그 트리에도 선다.
pub const ONE: Hands = hand_bits(Hand::One) | SHIELD;
pub const TWO: Hands = hand_bits(Hand::Two);
pub const MELEE: Hands = ONE | TWO;
pub const RANGED: Hands = hand_bits(Hand::Ranged);

/// 계열 `h`인 종류들의 비트(종류 표 `CLASSES`를 돈다).
pub const fn hand_bits(h: Hand) -> Hands {
    let mut bits = 0;
    let mut i = 0;
    while i < CLASSES.len() {
        if CLASSES[i].hand as u8 == h as u8 {
            bits |= class_bit(CLASSES[i].class);
        }
        i += 1;
    }
    bits
}
pub const ANY: Hands = MELEE | RANGED;

/// 쥔 꼴 하나의 비트 — 맨손은 어느 무기 판의 기술도 쓰지 못한다(0272: 무기 판이 없다). 한손 무기와 방패는 그 종류의 비트와 방패의
/// 비트를 다(0281 — 두 트리의 스킬을 섞어 장착한다).
pub const fn grip_bit(g: Grip) -> Hands {
    match g {
        Grip::Bare => 0,
        Grip::Class(c) => class_bit(c),
        Grip::Shield(None) => SHIELD,
        Grip::Shield(Some(c)) => SHIELD | class_bit(c),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::weapons::CLASSES;

    /// 0278 R19: 계열의 묶음 비트는 종류 표에서 선다 — 그 전부터 있던 종류에서는 손으로 적던 합(낫은 한손, 0274 ⑤ · 완드는 0281부터
    /// 한손)과 같다(뒤에 오는 종류는 제 비트만 더한다 — 종류의 계열을 옮길 때만 고친다). 셋은 겹치지 않고 모든 종류 + 방패를 덮는다.
    #[test]
    fn hand_masks_from_the_class_table_equal_the_old_unions() {
        let known = SWORD
            | HAMMER
            | STAFF
            | CROSSBOW
            | BOW
            | SPEAR
            | DAGGER
            | GREATSWORD
            | SCYTHE
            | WAND
            | SHIELD;
        assert_eq!(ONE & known, SWORD | HAMMER | SCYTHE | WAND | SHIELD);
        assert_eq!(TWO & known, STAFF | SPEAR | DAGGER | GREATSWORD);
        assert_eq!(RANGED & known, CROSSBOW | BOW);
        assert_eq!(ONE & TWO, 0);
        assert_eq!((ONE | TWO) & RANGED, 0);
        assert_eq!(MELEE | RANGED, ANY);
        assert_eq!(
            ANY.count_ones() as usize,
            CLASSES.len() + 1,
            "every class and the shield"
        );
    }
}
