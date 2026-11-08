//! 적 종류 표(0019, nail 0021과 같은 꼴). 새 적 = 표 한 줄 + (새 행동이면) `enemy.rs`의 수 하나 + 모델 하나.
//! 종류별 수치는 여기 한 곳(손맛 공통 값은 `tuning.rs`). 컨셉: 뿌리의 잔재(근접·느림·수로 압박) → 가시짐승(돌진) →
//! 첫 보스 잠들지 않는 문지기(내려치기·돌진·뿌리 파동, 체력 반 아래 핵이 밝아지며 빨라짐) — `docs/concept.md`.
use crate::i18n::{Strings, Tr};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Husk,
    Thorn,
    Warden,
    /// 등딱지짐승(0116 이름, 0120): 앞을 등딱지로 막는다.
    Shell,
    /// 꼬투리(0116 이름, 0120): 멀리서 바닥에 예고를 심는다.
    Pod,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Behavior {
    /// 다가와 예고(둘레 원판) 뒤 내려친다.
    Melee,
    /// 거리를 두고 예고(앞으로 곧은 띠) 뒤 곧게 돌진한다.
    Charger,
    /// 문지기: 가까우면 내려치기(부채꼴), 멀면 돌진, 세 번째마다 뿌리 파동(퍼지는 고리). 체력 반 아래 빨라짐.
    Warden,
    /// 심기(꼬투리, 0120): `plant_range` 안이면 플레이어 자리에 원판 예고를 심고(끝 = 판정), 너무 가까우면(`retreat`) 깡충 물러난다.
    Planter,
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
    /// 앞 막기(등딱지, 0120): 바라보는 쪽 이 각(도) 안에서 온 평타·탄을 튕긴다. 0 = 막지 않는다.
    pub guard_arc: f32,
    /// 심기(꼬투리, 0120): 심는 거리(m)·원판 반지름(m), 이보다 가까우면 물러난다(m).
    pub plant_range: f32,
    pub plant_r: f32,
    pub retreat: f32,
    pub behavior: Behavior,
    pub boss: bool,
    /// 쓰러지면 떨구는 잔불(0064 M1).
    pub embers: u32,
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
    guard_arc: 0.0,
    plant_range: 0.0,
    plant_r: 0.0,
    retreat: 0.0,
    behavior: Behavior::Melee,
    boss: false,
    embers: 1,
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
    guard_arc: 0.0,
    plant_range: 0.0,
    plant_r: 0.0,
    retreat: 0.0,
    behavior: Behavior::Charger,
    boss: false,
    embers: 2,
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
    guard_arc: 0.0,
    plant_range: 0.0,
    plant_r: 0.0,
    retreat: 0.0,
    behavior: Behavior::Warden,
    boss: true,
    embers: 30,
    model: "gatekeeper",
};

/// 등딱지짐승(0120, 막는 적 — 0069 "웅크린 것"): 앞 120°에서 온 평타·탄을 튕긴다(방랑자는 짧게 튀고 콤보가 끊긴다).
/// 뚫는 길 셋 — 강타(자세), 땅 돌리기·돌아 들어가 옆·뒤, 내려찍기. 느리고 단단하다, 둘레 내려치기는 잔재보다 조금 넓고 느리게.
pub static SHELL: Def = Def {
    kind: Kind::Shell,
    name: |s: &Strings| s.enemy_shell,
    hp: 5,
    radius: 0.55,
    height: 1.2,
    speed: 1.9,
    reach: 1.6,
    tele: 0.7,
    recover: 0.9,
    arc: 360.0,
    charge_range: 0.0,
    charge_tele: 0.0,
    charge_speed: 0.0,
    charge_time: 0.0,
    wave_tele: 0.0,
    wave_radius: 0.0,
    wave_speed: 0.0,
    guard_arc: 120.0,
    plant_range: 0.0,
    plant_r: 0.0,
    retreat: 0.0,
    behavior: Behavior::Melee,
    boss: false,
    embers: 3,
    model: "shellback",
};

/// 꼬투리(0120, 심는 적 — 0069 "씨앗"): 몸은 약하고 느리다. 9m 안이면 플레이어 자리에 원판을 심고(1.1초 — 끝 = 판정),
/// 3.5m 안으로 오면 한 번 깡충 물러난다(다시 뛰기까지 쉼). 구석에 몰면 잡힌다 — "다가갈까 피할까".
pub static POD: Def = Def {
    kind: Kind::Pod,
    name: |s: &Strings| s.enemy_pod,
    hp: 2,
    radius: 0.35,
    height: 0.9,
    speed: 1.6,
    reach: 0.0,
    tele: 1.1,
    recover: 1.4,
    arc: 360.0,
    charge_range: 0.0,
    charge_tele: 0.0,
    charge_speed: 0.0,
    charge_time: 0.0,
    wave_tele: 0.0,
    wave_radius: 0.0,
    wave_speed: 0.0,
    guard_arc: 0.0,
    plant_range: 9.0,
    plant_r: 1.4,
    retreat: 3.5,
    behavior: Behavior::Planter,
    boss: false,
    embers: 2,
    model: "pod",
};

pub static ALL: [&Def; 5] = [&HUSK, &THORN, &WARDEN, &SHELL, &POD];

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
            match d.behavior {
                Behavior::Melee => assert!(d.reach > d.radius, "{:?}", d.kind),
                Behavior::Charger | Behavior::Warden => {
                    assert!(d.reach > d.radius, "{:?}", d.kind);
                    assert!(d.charge_speed > 0.0 && d.charge_time > 0.0 && d.charge_tele > 0.0);
                }
                // 심는 적(0120): 멀리서 심고, 물러나는 거리는 심는 거리 안·원판보다 바깥.
                Behavior::Planter => {
                    assert!(d.plant_range > d.retreat && d.retreat > d.plant_r && d.plant_r > 0.0);
                }
            }
            assert!((0.0..=180.0).contains(&d.guard_arc), "{:?}", d.kind);
        }
        assert!(ALL.iter().filter(|d| d.boss).count() == 1);
    }
}
