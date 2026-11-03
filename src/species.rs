//! 적 종류 표(0019, nail 0021과 같은 꼴). 새 적 = 표 한 줄 + (새 행동이면) `enemy.rs`의 수 하나 + 모델 하나.
//! 종류별 수치는 여기 한 곳(손맛 공통 값은 `tuning.rs`). 컨셉: 뿌리의 잔재(근접·느림·수로 압박) → 가시짐승(돌진) →
//! 첫 보스 잠들지 않는 문지기(내려치기·돌진·뿌리 파동, 체력 반 아래 핵이 밝아지며 빨라짐) — `docs/concept.md`.
use crate::i18n::{Strings, Tr};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Husk,
    Thorn,
    Warden,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Behavior {
    /// 다가와 예고(둘레 원판) 뒤 내려친다.
    Melee,
    /// 거리를 두고 예고(앞으로 곧은 띠) 뒤 곧게 돌진한다.
    Charger,
    /// 문지기: 가까우면 내려치기(부채꼴), 멀면 돌진, 세 번째마다 뿌리 파동(퍼지는 고리). 체력 반 아래 빨라짐.
    Warden,
}

pub struct Def {
    pub kind: Kind,
    pub name: Tr,
    pub hp: i32,
    pub radius: f32,
    pub height: f32,
    /// 걷는 빠르기(m/s).
    pub speed: f32,
    /// 근접 공격이 닿는 거리(m, 몸 가운데에서).
    pub reach: f32,
    /// 근접 예고(초).
    pub tele: f32,
    /// 공격 뒤 쉬기(초).
    pub recover: f32,
    /// 문지기 내려치기 부채꼴(도). 360 = 둘레.
    pub arc: f32,
    /// 돌진: 시작 거리(m)·예고(초)·빠르기(m/s)·길이(초).
    pub charge_range: f32,
    pub charge_tele: f32,
    pub charge_speed: f32,
    pub charge_time: f32,
    /// 뿌리 파동(문지기): 예고(초)·닿는 반지름(m)·퍼지는 빠르기(m/s).
    pub wave_tele: f32,
    pub wave_radius: f32,
    pub wave_speed: f32,
    pub behavior: Behavior,
    pub boss: bool,
    /// 모델(`assets/models/<이름>.glb`).
    pub model: &'static str,
}

pub static HUSK: Def = Def {
    kind: Kind::Husk,
    name: |s: &Strings| s.enemy_husk,
    hp: 3,
    radius: 0.4,
    height: 1.5,
    speed: 2.6,
    reach: 1.4,
    tele: 0.55,
    recover: 0.7,
    arc: 360.0,
    charge_range: 0.0,
    charge_tele: 0.0,
    charge_speed: 0.0,
    charge_time: 0.0,
    wave_tele: 0.0,
    wave_radius: 0.0,
    wave_speed: 0.0,
    behavior: Behavior::Melee,
    boss: false,
    model: "root_husk",
};

pub static THORN: Def = Def {
    kind: Kind::Thorn,
    name: |s: &Strings| s.enemy_thorn,
    hp: 2,
    radius: 0.45,
    height: 1.1,
    speed: 3.8,
    reach: 1.2,
    tele: 0.5,
    recover: 0.9,
    arc: 360.0,
    charge_range: 7.0,
    charge_tele: 0.65,
    charge_speed: 13.0,
    charge_time: 0.5,
    wave_tele: 0.0,
    wave_radius: 0.0,
    wave_speed: 0.0,
    behavior: Behavior::Charger,
    boss: false,
    model: "thorn_beast",
};

pub static WARDEN: Def = Def {
    kind: Kind::Warden,
    name: |s: &Strings| s.boss_warden,
    hp: 40,
    radius: 1.0,
    height: 3.2,
    speed: 2.6,
    reach: 3.4,
    tele: 0.9,
    recover: 1.0,
    arc: 110.0,
    charge_range: 11.0,
    charge_tele: 0.8,
    charge_speed: 11.0,
    charge_time: 0.6,
    wave_tele: 1.0,
    wave_radius: 9.0,
    wave_speed: 9.0,
    behavior: Behavior::Warden,
    boss: true,
    model: "gatekeeper",
};

pub static ALL: [&Def; 3] = [&HUSK, &THORN, &WARDEN];

/// 종류의 줄(표 `ALL`에서 찾는다 — 새 종류 = `Kind` 한 줄 + 표 한 줄, 빠뜨리면 시험이 잡는다).
pub fn def(kind: Kind) -> &'static Def {
    ALL.iter()
        .find(|d| d.kind == kind)
        .copied()
        .unwrap_or(&HUSK)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_has_one_row_with_sane_numbers() {
        for d in ALL {
            assert_eq!(def(d.kind).kind, d.kind);
            assert!(d.hp > 0 && d.radius > 0.0 && d.height > 0.0 && d.speed > 0.0);
            assert!(d.reach > d.radius, "{:?}", d.kind);
            if d.behavior != Behavior::Melee {
                assert!(d.charge_speed > 0.0 && d.charge_time > 0.0 && d.charge_tele > 0.0);
            }
        }
        assert!(ALL.iter().filter(|d| d.boss).count() == 1);
    }
}
