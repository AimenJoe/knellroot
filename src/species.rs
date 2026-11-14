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
    /// 깨진 종지기(0116 이름, 0125 — 둘째 문지기): 울림 고리·종 조각·종 끌기 돌진.
    Bellkeeper,
    /// 이끼먹이(0185 — 비적대 생명체 한 종): 젖은 바닥에서 이끼를 뜯고, 다가오면 바위 틈 쪽으로 달아난다. 치지 않는다.
    Mossgrazer,
    /// 이끼 덮인 문지기(0185 — 셋째 문지기, 설계 내부 작업명 "문턱지기"): 쓸기·껍질 받기·틈새 돌진.
    Mossward,
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
    /// 풀 뜯기(이끼먹이, 0185): 공격이 없다. 제자리 둘레에서 뜯고, 방랑자가 `GRAZE_FLEE_R` 안이면 반대쪽으로 달아나고(제자리에서
    /// `GRAZE_LEASH`까지), 멀어지면 돌아온다. 락온·알아차림에 잡히지 않는다.
    Grazer,
    /// 깨진 종지기(0125): 가까우면 땅을 쳐 울림 고리(`rings`겹 — 사이에 서거나 뛰어넘는다), 멀면 종 조각 부채꼴(`shards`개 —
    /// 패링하면 되돌아가 종지기를 깎는다)과 종 끌기 돌진을 번갈아. 체력 반 아래 고리·조각이 하나·둘 는다.
    Bellkeeper,
}

/// 공격 박자 변주(0128, 사장 "정박에만 때리면 난이도가 확 내려간다"): 같은 공격을 다른 박자로.
/// 지연 = 들어 올림은 같고 정점에서 더 버틴다(몸이 더 젖히고 떨림 + 내리치기 직전 소리), 빠름 = 예고가 짧다,
/// 연속 = 첫 타 뒤 같은 공격이 고르지 않은 간격(`gaps` = 예고 배율)으로 더, 페인트 = 체력 반 아래 보스가 들다가 다른 공격으로.
/// 외울 수 있게 길이는 몇 가지로만(`tuning::DELAY_HOLDS`). 확률은 공격을 시작할 때 한 번 굴린다.
#[derive(Clone, Copy, Debug)]
pub struct Rhythm {
    pub delay: f32,
    pub fast: f32,
    pub chain: f32,
    pub gaps: &'static [f32],
    pub feint: f32,
}

/// 변주 없음(꼬투리 — 심는 것이 이미 늦게 터진다).
pub const STEADY: Rhythm = Rhythm {
    delay: 0.0,
    fast: 0.0,
    chain: 0.0,
    gaps: &[],
    feint: 0.0,
};

pub struct Def {
    /// 공격 박자 변주(0128).
    pub rhythm: Rhythm,
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
    /// 깨진 종지기(0125): 울림 고리 겹 수·종 조각 수(체력 반 아래 +1·+2).
    pub rings: u8,
    pub shards: u8,
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
    hp: 30,
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
    rings: 0,
    shards: 0,
    rhythm: Rhythm {
        delay: 0.25,
        fast: 0.0,
        chain: 0.0,
        gaps: &[],
        feint: 0.0,
    },
    behavior: Behavior::Melee,
    boss: false,
    embers: 1,
    model: "root_husk",
};

pub static THORN: Def = Def {
    kind: Kind::Thorn,
    name: |s: &Strings| s.enemy_thorn,
    hp: 20,
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
    rings: 0,
    shards: 0,
    rhythm: Rhythm {
        delay: 0.15,
        fast: 0.15,
        chain: 0.0,
        gaps: &[],
        feint: 0.0,
    },
    behavior: Behavior::Charger,
    boss: false,
    embers: 2,
    model: "thorn_beast",
};

pub static WARDEN: Def = Def {
    kind: Kind::Warden,
    name: |s: &Strings| s.boss_warden,
    hp: 400,
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
    rings: 0,
    shards: 0,
    rhythm: Rhythm {
        delay: 0.3,
        fast: 0.15,
        chain: 0.35,
        gaps: &[0.45, 1.25],
        feint: 0.3,
    },
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
    hp: 50,
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
    rings: 0,
    shards: 0,
    rhythm: Rhythm {
        delay: 0.25,
        fast: 0.0,
        chain: 0.0,
        gaps: &[],
        feint: 0.0,
    },
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
    hp: 20,
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
    rings: 0,
    shards: 0,
    rhythm: STEADY,
    behavior: Behavior::Planter,
    boss: false,
    embers: 2,
    model: "pod",
};

/// 깨진 종지기(0116 이름·3절 싸움 꼴, 0125): 둘째 문지기 — 거리·공간형. 마을의 깨진 종을 끌고 다닌다(종이 왜 깨졌는지는 말하지 않는다).
/// 가까우면(5m 안) 울림 고리 두 겹(7m/s, 9m까지 — 사이 2.6m에 서거나 뛰어넘기), 멀면 종 조각 부채꼴 셋(50°, 되받아치기)과 종 끌기 돌진(띠)을 번갈아.
pub static BELLKEEPER: Def = Def {
    kind: Kind::Bellkeeper,
    name: |s: &Strings| s.boss_bellkeeper,
    hp: 580,
    radius: 1.0,
    height: 3.0,
    speed: 2.2,
    reach: 5.0,
    tele: 0.9,
    recover: 1.1,
    arc: 50.0,
    charge_range: 13.0,
    charge_tele: 0.85,
    charge_speed: 10.0,
    charge_time: 0.75,
    wave_tele: 1.0,
    wave_radius: 9.0,
    wave_speed: 7.0,
    guard_arc: 0.0,
    plant_range: 0.0,
    plant_r: 0.0,
    retreat: 0.0,
    rings: 2,
    shards: 3,
    rhythm: Rhythm {
        delay: 0.35,
        fast: 0.2,
        chain: 0.4,
        gaps: &[0.4, 1.3],
        feint: 0.35,
    },
    behavior: Behavior::Bellkeeper,
    boss: true,
    embers: 30,
    model: "bellkeeper",
};

/// 이끼 덮인 문지기(0185 — 셋째 문지기, 잔뿌리의 정원 문턱 아래). 지역 생태의 큰 몸: 낮게 나아가 앞을 쓸고, 웅크려 정면을 흘리고
/// (등딱지짐승의 앞 막기 — 옆·뒤·강타로 연다), 거리가 벌어지면 틈새 돌진. 체력 반 아래 간격이 짧아지고 쓸기 뒤 돌진을 잇는다.
pub static MOSSWARD: Def = Def {
    kind: Kind::Mossward,
    name: |s: &Strings| s.boss_mossward,
    hp: 640,
    radius: 1.1,
    height: 2.6,
    speed: 2.4,
    reach: 3.6,
    tele: 0.85,
    recover: 1.0,
    arc: 150.0,
    charge_range: 10.0,
    charge_tele: 0.8,
    charge_speed: 12.0,
    charge_time: 0.55,
    wave_tele: 1.0,
    wave_radius: 8.0,
    wave_speed: 8.0,
    guard_arc: 0.0,
    plant_range: 0.0,
    plant_r: 0.0,
    retreat: 0.0,
    rings: 0,
    shards: 0,
    rhythm: Rhythm {
        delay: 0.35,
        fast: 0.2,
        chain: 0.35,
        gaps: &[0.45, 1.2],
        feint: 0.3,
    },
    behavior: Behavior::Warden,
    boss: true,
    embers: 30,
    model: "gatekeeper",
};

/// 이끼먹이(0185 — 정상 피식자): 작고 둥근 몸, 공격·보상 없음(잔불 0). 맞으면 아프지만 싸우지 않고 달아난다.
pub static MOSSGRAZER: Def = Def {
    kind: Kind::Mossgrazer,
    name: |s: &Strings| s.creature_mossgrazer,
    hp: 20,
    radius: 0.32,
    height: 0.6,
    speed: 3.4,
    reach: 0.0,
    tele: 0.0,
    recover: 0.0,
    arc: 0.0,
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
    rings: 0,
    shards: 0,
    rhythm: STEADY,
    behavior: Behavior::Grazer,
    boss: false,
    embers: 0,
    model: "mossgrazer",
};

pub static ALL: [&Def; 8] = [
    &HUSK,
    &THORN,
    &WARDEN,
    &SHELL,
    &POD,
    &BELLKEEPER,
    &MOSSWARD,
    &MOSSGRAZER,
];

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
                Behavior::Charger | Behavior::Warden | Behavior::Bellkeeper => {
                    assert!(d.reach > d.radius, "{:?}", d.kind);
                    assert!(d.charge_speed > 0.0 && d.charge_time > 0.0 && d.charge_tele > 0.0);
                }
                // 심는 적(0120): 멀리서 심고, 물러나는 거리는 심는 거리 안·원판보다 바깥.
                Behavior::Planter => {
                    assert!(d.plant_range > d.retreat && d.retreat > d.plant_r && d.plant_r > 0.0);
                }
                // 이끼먹이(0185): 치지 않고(닿는 거리 0) 잔불도 없다.
                Behavior::Grazer => assert!(d.reach == 0.0 && d.embers == 0 && !d.boss),
            }
            assert!((0.0..=180.0).contains(&d.guard_arc), "{:?}", d.kind);
        }
        // 문지기는 지역마다 하나(성소 문지기·깨진 종지기·이끼 덮인 문지기 0185).
        assert_eq!(
            ALL.iter().filter(|d| d.boss).count(),
            crate::stage::STAGES.len()
        );
    }
}
