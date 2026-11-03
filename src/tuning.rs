//! 수치는 전부 여기(nail 규칙 그대로). 랩(`lab.rs`)이 실행 중에 바꾸는 값은 `P`로 이름 붙은 것들이고,
//! `SPECS`의 `def`가 곧 기본값이다. 랩에서 "export"하면 `pub const` 줄을 콘솔에 찍는다 → 사람이 여기에 붙여 넣고 커밋한다
//! (저장 파일을 게임이 읽게 하지 않는다: 진실은 한 곳, KB guides/poc-and-lab-before-content.md).
use std::ops::Index;

/// 로직 고정 타임스텝(60Hz).
pub const TICK_DT: f32 = 1.0 / 60.0;
/// 한 프레임에 돌리는 로직 시간 상한(탭 전환 뒤 폭주 방지).
pub const MAX_FRAME_DT: f32 = 0.1;
/// 내부 해상도(PS1 느낌). 정수 배율로 키운다.
pub const INTERNAL_W: u32 = 480;
pub const INTERNAL_H: u32 = 270;
/// 플레이어 키(m). 카메라 높이·주시점의 기준.
pub const PLAYER_HEIGHT: f32 = 1.8;
pub const PLAYER_RADIUS: f32 = 0.35;
pub const PLAYER_HP: i32 = 5;
pub const STICK_DEADZONE: f32 = 0.2;

/// 랩에서 바꾸는 값. 새 값 = 여기 한 줄 + `SPECS` 한 줄.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum P {
    CamQPitch,
    CamQDist,
    CamQFov,
    CamQLag,
    CamQYaw,
    CamLDist,
    CamLHeight,
    CamLShoulder,
    CamLFov,
    CamLLookAhead,
    CamLFrame,
    CamLYawLag,
    CamBlend,
    CamCollideRadius,
    MoveSpeed,
    MoveAccel,
    TurnSpeed,
    DodgeDist,
    DodgeTime,
    AttackStartup,
    AttackActive,
    AttackRecovery,
    AttackRange,
    AttackArc,
    Hitstop,
    Knockback,
    LockRange,
    LockLose,
    MobSpeed,
    MobTele,
    BossTele,
    BossSwingArc,
    Shade,
    CamLYawMax,
    CamLNear,
    CamPullOut,
    CamQLead,
}

pub struct Spec {
    pub p: P,
    /// 랩 화면 이름(개발 전용 화면이라 영어 ASCII — 기본 폰트가 ASCII뿐).
    pub name: &'static str,
    /// export 때 찍을 상수 이름.
    pub konst: &'static str,
    pub def: f32,
    pub min: f32,
    pub max: f32,
    pub step: f32,
}

macro_rules! spec {
    ($p:ident, $name:literal, $konst:literal, $def:expr, $min:expr, $max:expr, $step:expr) => {
        Spec {
            p: P::$p,
            name: $name,
            konst: $konst,
            def: $def,
            min: $min,
            max: $max,
            step: $step,
        }
    };
}

/// 순서 = 랩 화면 순서. 카메라 → 이동 → 공격 → 락온 → 적.
pub const SPECS: [Spec; 37] = [
    spec!(
        CamQPitch,
        "cam.quarter.pitch deg",
        "CAM_Q_PITCH_DEG",
        55.0,
        20.0,
        85.0,
        1.0
    ),
    spec!(
        CamQDist,
        "cam.quarter.dist",
        "CAM_Q_DIST",
        14.0,
        4.0,
        30.0,
        0.5
    ),
    spec!(
        CamQFov,
        "cam.quarter.fov deg",
        "CAM_Q_FOV_DEG",
        42.0,
        15.0,
        80.0,
        1.0
    ),
    spec!(
        CamQLag,
        "cam.quarter.follow 1/s",
        "CAM_Q_LAG",
        8.0,
        1.0,
        30.0,
        0.5
    ),
    spec!(
        CamQYaw,
        "cam.quarter.yaw deg",
        "CAM_Q_YAW_DEG",
        35.0,
        -60.0,
        60.0,
        1.0
    ),
    spec!(
        CamLDist,
        "cam.lock.dist",
        "CAM_L_DIST",
        4.5,
        1.5,
        12.0,
        0.25
    ),
    spec!(
        CamLHeight,
        "cam.lock.height",
        "CAM_L_HEIGHT",
        2.0,
        0.5,
        6.0,
        0.1
    ),
    spec!(
        CamLShoulder,
        "cam.lock.shoulder",
        "CAM_L_SHOULDER",
        0.8,
        -2.0,
        2.0,
        0.1
    ),
    spec!(
        CamLFov,
        "cam.lock.fov deg",
        "CAM_L_FOV_DEG",
        58.0,
        30.0,
        90.0,
        1.0
    ),
    spec!(
        CamLLookAhead,
        "cam.lock.look ahead m",
        "CAM_L_LOOK_AHEAD",
        3.0,
        0.0,
        10.0,
        0.25
    ),
    spec!(
        CamLFrame,
        "cam.lock.frame x",
        "CAM_L_FRAME",
        1.0,
        -3.0,
        3.0,
        0.1
    ),
    spec!(
        CamLYawLag,
        "cam.lock.yaw 1/s",
        "CAM_L_YAW_LAG",
        6.0,
        1.0,
        30.0,
        0.5
    ),
    spec!(CamBlend, "cam.blend s", "CAM_BLEND", 0.3, 0.05, 1.0, 0.05),
    spec!(
        CamCollideRadius,
        "cam.collide r",
        "CAM_COLLIDE_RADIUS",
        0.3,
        0.0,
        1.0,
        0.05
    ),
    spec!(MoveSpeed, "move.speed", "MOVE_SPEED", 6.0, 1.0, 14.0, 0.25),
    spec!(MoveAccel, "move.accel", "MOVE_ACCEL", 40.0, 5.0, 120.0, 5.0),
    spec!(
        TurnSpeed,
        "move.turn deg/s",
        "TURN_SPEED_DEG",
        720.0,
        90.0,
        1800.0,
        30.0
    ),
    spec!(DodgeDist, "dodge.dist", "DODGE_DIST", 3.0, 1.0, 8.0, 0.25),
    spec!(
        DodgeTime,
        "dodge.time s",
        "DODGE_TIME",
        0.25,
        0.1,
        0.6,
        0.01
    ),
    spec!(
        AttackStartup,
        "attack.startup s",
        "ATTACK_STARTUP",
        0.08,
        0.0,
        0.4,
        0.01
    ),
    spec!(
        AttackActive,
        "attack.active s",
        "ATTACK_ACTIVE",
        0.10,
        0.02,
        0.4,
        0.01
    ),
    spec!(
        AttackRecovery,
        "attack.recovery s",
        "ATTACK_RECOVERY",
        0.25,
        0.0,
        0.8,
        0.01
    ),
    spec!(
        AttackRange,
        "attack.range",
        "ATTACK_RANGE",
        1.8,
        0.5,
        5.0,
        0.1
    ),
    spec!(
        AttackArc,
        "attack.arc deg",
        "ATTACK_ARC_DEG",
        120.0,
        30.0,
        360.0,
        10.0
    ),
    spec!(Hitstop, "feel.hitstop s", "HITSTOP", 0.06, 0.0, 0.3, 0.01),
    spec!(
        Knockback,
        "feel.knockback",
        "KNOCKBACK",
        5.0,
        0.0,
        20.0,
        0.5
    ),
    spec!(LockRange, "lock.range", "LOCK_RANGE", 18.0, 4.0, 40.0, 1.0),
    spec!(
        LockLose,
        "lock.lose dist",
        "LOCK_LOSE",
        26.0,
        4.0,
        60.0,
        1.0
    ),
    spec!(MobSpeed, "mob.speed", "MOB_SPEED", 3.5, 0.5, 10.0, 0.25),
    spec!(MobTele, "mob.telegraph s", "MOB_TELE", 0.5, 0.1, 2.0, 0.05),
    spec!(
        BossTele,
        "boss.telegraph s",
        "BOSS_TELE",
        0.9,
        0.2,
        3.0,
        0.05
    ),
    spec!(
        BossSwingArc,
        "boss.swing arc deg",
        "BOSS_SWING_ARC_DEG",
        110.0,
        30.0,
        360.0,
        10.0
    ),
    spec!(Shade, "render.face shade", "SHADE", 0.45, 0.0, 1.0, 0.05),
    spec!(
        CamLYawMax,
        "cam.lock.yaw max deg/s",
        "CAM_L_YAW_MAX_DEG",
        90.0,
        20.0,
        360.0,
        5.0
    ),
    spec!(
        CamLNear,
        "cam.lock.near slow m",
        "CAM_L_NEAR",
        3.0,
        0.0,
        8.0,
        0.25
    ),
    spec!(
        CamPullOut,
        "cam.pull out 1/s",
        "CAM_PULL_OUT",
        4.0,
        0.5,
        20.0,
        0.5
    ),
    spec!(
        CamQLead,
        "cam.quarter.lead s",
        "CAM_Q_LEAD",
        0.22,
        0.0,
        0.6,
        0.02
    ),
];

/// 실행 중 값. `SPECS`의 기본값으로 시작하고 랩이 바꾼다. 로직·카메라·렌더는 이것만 읽는다.
#[derive(Clone, PartialEq, Debug)]
pub struct Tuning {
    v: [f32; SPECS.len()],
}

impl Default for Tuning {
    fn default() -> Self {
        let mut v = [0.0; SPECS.len()];
        for (i, s) in SPECS.iter().enumerate() {
            v[i] = s.def;
        }
        Tuning { v }
    }
}

impl Index<P> for Tuning {
    type Output = f32;
    fn index(&self, p: P) -> &f32 {
        &self.v[idx(p)]
    }
}

impl Tuning {
    pub fn get(&self, i: usize) -> f32 {
        self.v[i]
    }
    pub fn set(&mut self, i: usize, value: f32) {
        let s = &SPECS[i];
        self.v[i] = value.clamp(s.min, s.max);
    }
    pub fn reset(&mut self, i: usize) {
        self.v[i] = SPECS[i].def;
    }
    /// `tuning.rs`에 붙여 넣을 줄들. 기본값과 같은 값은 "(default)"를 달아 구분한다.
    pub fn export(&self) -> String {
        let mut out = String::new();
        for (i, s) in SPECS.iter().enumerate() {
            let same = if (self.v[i] - s.def).abs() < 1e-6 {
                "  // (default)"
            } else {
                ""
            };
            out.push_str(&format!(
                "pub const {}: f32 = {:.3};{}\n",
                s.konst, self.v[i], same
            ));
        }
        out
    }
}

fn idx(p: P) -> usize {
    // SPECS 순서로 찾는다. 30개라 선형이어도 된다(틱마다 수십 번).
    SPECS.iter().position(|s| s.p == p).expect("P not in SPECS")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_p_has_exactly_one_spec_and_defaults_are_in_range() {
        for s in SPECS.iter() {
            assert!(
                s.min <= s.def && s.def <= s.max,
                "{} default out of range",
                s.konst
            );
            assert!(s.step > 0.0);
            assert_eq!(
                SPECS.iter().filter(|o| o.p == s.p).count(),
                1,
                "{:?} listed twice",
                s.p
            );
        }
        let names: std::collections::HashSet<_> = SPECS.iter().map(|s| s.konst).collect();
        assert_eq!(names.len(), SPECS.len(), "duplicate const names");
    }

    #[test]
    fn set_clamps_and_export_marks_defaults() {
        let mut t = Tuning::default();
        t.set(0, 1000.0);
        assert_eq!(t.get(0), SPECS[0].max);
        let e = t.export();
        assert!(e.contains("CAM_Q_PITCH_DEG: f32 = 85.000;\n"));
        assert!(e.contains("CAM_Q_DIST: f32 = 14.000;  // (default)"));
    }
}
