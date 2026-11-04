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
/// 화면 흔들림(0020): trauma 1이면 카메라가 이 각(도)만큼 흔들린다 — 거리에 곱해 쿼터뷰(20m)·락온(7m)에서 화면 크기가 같다.
/// 예전엔 월드 ±0.25m 고정이라 20m 쿼터뷰에서 1px 남짓(사장: "피격 시에도 흔들림 있어야지").
pub const SHAKE_DEG: f32 = 1.6;
/// 사건마다 더하는 trauma(크기는 제곱으로 쓴다).
pub const SHAKE_HIT: f32 = 0.25;
pub const SHAKE_HURT: f32 = 0.75;
pub const SHAKE_DOWN: f32 = 0.15;
pub const SHAKE_TURN: f32 = 0.2;
pub const SHAKE_SLAM: f32 = 0.35;
pub const SHAKE_ROUSE: f32 = 0.6;
pub const SHAKE_PARRY: f32 = 0.5;
pub const SHAKE_CRIT: f32 = 0.45;
pub const SHAKE_PLUNGE: f32 = 0.4;
/// 강타가 맞았을 때(0030) — 평타·내려찍기보다 크게. 다 모였을 때는 작은 "탁".
pub const SHAKE_SMASH: f32 = 0.5;
pub const SHAKE_CHARGED: f32 = 0.12;

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
    Shade,
    CamLYawMax,
    CamLNear,
    CamPullOut,
    CamQLead,
    CamBlendTurn,
    CamBlendExit,
    MoveHoldMax,
    CamQYawMode,
    TurnTime,
    TurnOver,
    TurnSettle,
    TurnSnap,
    TurnPlant,
    TurnCooldown,
    SpillRange,
    SpillPerM,
    SpillMax,
    SpillOut,
    SpillStagger,
    SpillBossMass,
    SpillBossStagger,
    SlamDamage,
    SlamStagger,
    Haze,
    AimRange,
    AimCone,
    AttackLunge,
    HitStun,
    HitIframes,
    HitKnock,
    StaminaMax,
    StaminaRegen,
    StaminaDelay,
    StaminaDodge,
    StaminaAttack,
    ParryWindow,
    ParryTime,
    ParryStamina,
    ParryStagger,
    ParryBossStagger,
    ParryReach,
    ParryCrit,
    JumpHeight,
    JumpTime,
    JumpStamina,
    JumpAirControl,
    JumpClear,
    JumpLand,
    PlungeDamage,
    PlungeRange,
    PlungeLand,
    HeavyCharge,
    HeavyStartup,
    HeavyActive,
    HeavyRecovery,
    HeavyDamage,
    HeavyFullBonus,
    HeavyRange,
    HeavyArc,
    HeavyLunge,
    HeavyKnock,
    HeavyStagger,
    HeavyBossStagger,
    HeavyStamina,
    RenderModel,
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
pub const SPECS: [Spec; 94] = [
    spec!(
        CamQPitch,
        "cam.quarter.pitch deg",
        "CAM_Q_PITCH_DEG",
        40.0,
        20.0,
        85.0,
        1.0
    ),
    spec!(
        CamQDist,
        "cam.quarter.dist",
        "CAM_Q_DIST",
        20.0,
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
        7.0,
        1.5,
        12.0,
        0.25
    ),
    spec!(
        CamLHeight,
        "cam.lock.height",
        "CAM_L_HEIGHT",
        2.5,
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
        1.3,
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
    spec!(CamBlend, "cam.blend s", "CAM_BLEND", 0.4, 0.05, 1.0, 0.05),
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
    spec!(
        CamBlendTurn,
        "cam.blend turn x",
        "CAM_BLEND_TURN",
        2.0,
        0.0,
        3.0,
        0.1
    ),
    spec!(
        CamBlendExit,
        "cam.blend exit x",
        "CAM_BLEND_EXIT",
        0.7,
        0.1,
        1.5,
        0.05
    ),
    spec!(
        MoveHoldMax,
        "move.hold after unlock s",
        "MOVE_HOLD_MAX",
        0.6,
        0.0,
        2.0,
        0.05
    ),
    spec!(
        CamQYawMode,
        "cam.quarter.yaw mode 0fix 1fol 2snap",
        "CAM_Q_YAW_MODE",
        2.0,
        0.0,
        2.0,
        1.0
    ),
    // 쟁반 돌리기(P6, 0012): 주인공이 땅을 90° 비튼다 — 화면이 돌고, 둘레의 적은 관성으로 반대쪽·바깥으로 쏟아져 휘청인다.
    spec!(TurnTime, "turn.camera s", "TURN_TIME", 0.8, 0.1, 1.0, 0.05),
    spec!(
        TurnOver,
        "turn.overshoot deg",
        "TURN_OVER",
        12.0,
        0.0,
        30.0,
        1.0
    ),
    spec!(
        TurnSettle,
        "turn.settle s",
        "TURN_SETTLE",
        0.15,
        0.05,
        0.6,
        0.05
    ),
    // 되돌아와 90°에 닿는 순간의 "탁"(카메라 흔들림 trauma). 0 = 없음.
    spec!(
        TurnSnap,
        "turn.snap kick",
        "TURN_SNAP",
        0.35,
        0.0,
        1.0,
        0.05
    ),
    spec!(
        TurnPlant,
        "turn.plant s",
        "TURN_PLANT",
        0.25,
        0.0,
        0.8,
        0.05
    ),
    spec!(
        TurnCooldown,
        "turn.cooldown s",
        "TURN_COOLDOWN",
        4.0,
        0.0,
        10.0,
        0.25
    ),
    spec!(
        SpillRange,
        "spill.range m",
        "SPILL_RANGE",
        8.0,
        2.0,
        20.0,
        0.5
    ),
    spec!(
        SpillPerM,
        "spill.slide per m",
        "SPILL_PER_M",
        0.05,
        0.0,
        1.5,
        0.05
    ),
    spec!(
        SpillMax,
        "spill.slide max m",
        "SPILL_MAX",
        0.25,
        0.0,
        8.0,
        0.25
    ),
    spec!(
        SpillOut,
        "spill.outward share",
        "SPILL_OUT",
        0.35,
        0.0,
        1.0,
        0.05
    ),
    spec!(
        SpillStagger,
        "spill.stagger s",
        "SPILL_STAGGER",
        0.3,
        0.0,
        3.0,
        0.1
    ),
    spec!(
        SpillBossMass,
        "spill.boss mass",
        "SPILL_BOSS_MASS",
        6.0,
        1.0,
        20.0,
        0.5
    ),
    spec!(
        SpillBossStagger,
        "spill.boss stagger s",
        "SPILL_BOSS_STAGGER",
        0.1,
        0.0,
        2.0,
        0.05
    ),
    spec!(
        SlamDamage,
        "spill.slam dmg",
        "SLAM_DAMAGE",
        0.0,
        0.0,
        5.0,
        1.0
    ),
    spec!(
        SlamStagger,
        "spill.slam stagger +s",
        "SLAM_STAGGER",
        0.0,
        0.0,
        3.0,
        0.1
    ),
    // 지평선 아지랑이(0012): 화면 위쪽 이 몫만큼 먼 곳이 하늘빛으로 옅어진다. 카메라는 기울지 않고 수직축으로만 돌아서
    // 이 띠는 진짜 지평선처럼 늘 수평이고 회전 중에도 제자리 — 땅에 고정된 지평선이 멀미를 줄인다(Hemmerich 2020). 0 = 끔.
    spec!(Haze, "render.horizon haze", "HAZE", 0.3, 0.0, 0.6, 0.05),
    // 조준(0014): 공격 순간 이 거리(적 몸 가장자리까지, m) 안의 가장 가까운 적을 본다. 조준 스틱을 쥐면 그 쪽 부채꼴(전체 각) 안에서만.
    spec!(
        AimRange,
        "aim.auto range m",
        "AIM_RANGE",
        3.0,
        0.0,
        8.0,
        0.25
    ),
    spec!(
        AimCone,
        "aim.auto cone deg",
        "AIM_CONE",
        90.0,
        10.0,
        360.0,
        5.0
    ),
    // 손맛·생존(0018): 공격은 입력한 쪽으로 이만큼(m) 내딛으며(선딜+활성 동안), 맞으면 경직(초) — 경직 중 또 맞으면 반씩 줄고,
    // 맞은 뒤 무적(초)·밀림(m). 스태미나: 최대·초당 회복·마지막 사용 뒤 회복이 시작되기까지(초)·회피/공격 값.
    spec!(
        AttackLunge,
        "attack.lunge m",
        "ATTACK_LUNGE",
        0.5,
        0.0,
        2.0,
        0.05
    ),
    spec!(HitStun, "hurt.stun s", "HIT_STUN", 0.35, 0.0, 1.0, 0.05),
    spec!(
        HitIframes,
        "hurt.iframes s",
        "HIT_IFRAMES",
        0.15,
        0.0,
        1.0,
        0.05
    ),
    spec!(HitKnock, "hurt.knock m", "HIT_KNOCK", 0.6, 0.0, 3.0, 0.1),
    spec!(
        StaminaMax,
        "stamina.max",
        "STAMINA_MAX",
        100.0,
        10.0,
        300.0,
        5.0
    ),
    spec!(
        StaminaRegen,
        "stamina.regen /s",
        "STAMINA_REGEN",
        45.0,
        5.0,
        200.0,
        5.0
    ),
    spec!(
        StaminaDelay,
        "stamina.regen delay s",
        "STAMINA_DELAY",
        0.6,
        0.0,
        3.0,
        0.1
    ),
    spec!(
        StaminaDodge,
        "stamina.dodge cost",
        "STAMINA_DODGE",
        30.0,
        0.0,
        100.0,
        1.0
    ),
    spec!(
        StaminaAttack,
        "stamina.attack cost",
        "STAMINA_ATTACK",
        12.0,
        0.0,
        100.0,
        1.0
    ),
    // 패링(0025, KB concepts/timing-parry.md): 누른 뒤 `parry.window s` 안에 앞쪽에서 닿는 근접 공격을 튕긴다. 창은 짧게
    // (KB: 0.1~0.2초, Sekiro ≈ 0.2초는 커뮤니티 측정). 헛치면 `parry.time s`까지 못 움직인다 — 연타가 공짜가 되지 않게.
    spec!(
        ParryWindow,
        "parry.window s",
        "PARRY_WINDOW",
        0.18,
        0.05,
        0.5,
        0.01
    ),
    spec!(ParryTime, "parry.time s", "PARRY_TIME", 0.5, 0.1, 1.5, 0.05),
    spec!(
        ParryStamina,
        "parry.stamina cost",
        "PARRY_STAMINA",
        15.0,
        0.0,
        100.0,
        1.0
    ),
    // 튕긴 적은 휘청(예고 끊김)하고 그동안 다음 한 대가 치명타(+`parry.crit`).
    spec!(
        ParryStagger,
        "parry.stagger s",
        "PARRY_STAGGER",
        1.2,
        0.0,
        3.0,
        0.05
    ),
    spec!(
        ParryBossStagger,
        "parry.boss stagger s",
        "PARRY_BOSS_STAGGER",
        0.8,
        0.0,
        3.0,
        0.05
    ),
    // 곧게 오는 돌진은 몸에 닿기 이만큼(m) 앞에서부터 받는다(nail PARRY_REACH 0258 — 닿는 순간만 받으면 늦다).
    spec!(
        ParryReach,
        "parry.reach m",
        "PARRY_REACH",
        1.0,
        0.0,
        3.0,
        0.1
    ),
    spec!(
        ParryCrit,
        "parry.crit bonus",
        "PARRY_CRIT",
        2.0,
        0.0,
        10.0,
        1.0
    ),
    // 점프(0027): 높이·체공 시간에서 처음 속도·중력을 만든다(포물선). 뛰기 전 속도를 잇고 공중에선 조금만 조종.
    // `jump.clear m`보다 높이 떠 있으면 바닥을 훑는 뿌리 파동을 넘는다(엘든 링: 점프로 낮은 공격 피하기).
    spec!(
        JumpHeight,
        "jump.height m",
        "JUMP_HEIGHT",
        0.9,
        0.2,
        3.0,
        0.05
    ),
    spec!(JumpTime, "jump.air s", "JUMP_TIME", 0.55, 0.2, 1.5, 0.05),
    spec!(
        JumpStamina,
        "jump.stamina cost",
        "JUMP_STAMINA",
        12.0,
        0.0,
        100.0,
        1.0
    ),
    spec!(
        JumpAirControl,
        "jump.air control x",
        "JUMP_AIR_CONTROL",
        0.3,
        0.0,
        1.0,
        0.05
    ),
    spec!(
        JumpClear,
        "jump.clear m",
        "JUMP_CLEAR",
        0.35,
        0.0,
        2.0,
        0.05
    ),
    spec!(JumpLand, "jump.land s", "JUMP_LAND", 0.1, 0.0, 1.0, 0.02),
    // 내려찍기: 공중에서 공격 → 내려앉는 순간 둘레를 친다(묵직하게 — 피해 2, 넉백, 히트스톱).
    spec!(
        PlungeDamage,
        "plunge.damage",
        "PLUNGE_DAMAGE",
        2.0,
        1.0,
        10.0,
        1.0
    ),
    spec!(
        PlungeRange,
        "plunge.range m",
        "PLUNGE_RANGE",
        1.6,
        0.5,
        4.0,
        0.1
    ),
    spec!(
        PlungeLand,
        "plunge.land s",
        "PLUNGE_LAND",
        0.35,
        0.0,
        1.5,
        0.05
    ),
    // 강공격(0030): 평타 뒤 쥐고 있으면 모으기, 떼면 강타. 다 모으는 시간.
    spec!(
        HeavyCharge,
        "heavy.charge s",
        "HEAVY_CHARGE",
        0.5,
        0.1,
        2.0,
        0.05
    ),
    spec!(
        HeavyStartup,
        "heavy.startup s",
        "HEAVY_STARTUP",
        0.06,
        0.0,
        0.4,
        0.02
    ),
    spec!(
        HeavyActive,
        "heavy.active s",
        "HEAVY_ACTIVE",
        0.12,
        0.02,
        0.4,
        0.02
    ),
    spec!(
        HeavyRecovery,
        "heavy.recovery s",
        "HEAVY_RECOVERY",
        0.4,
        0.0,
        1.2,
        0.05
    ),
    // 덜 모으고 떼도 이만큼, 다 모으면 + `heavy.full bonus`.
    spec!(
        HeavyDamage,
        "heavy.damage",
        "HEAVY_DAMAGE",
        2.0,
        1.0,
        8.0,
        1.0
    ),
    spec!(
        HeavyFullBonus,
        "heavy.full bonus",
        "HEAVY_FULL_BONUS",
        1.0,
        0.0,
        5.0,
        1.0
    ),
    spec!(HeavyRange, "heavy.range", "HEAVY_RANGE", 2.2, 0.5, 5.0, 0.1),
    spec!(
        HeavyArc,
        "heavy.arc deg",
        "HEAVY_ARC",
        160.0,
        30.0,
        360.0,
        10.0
    ),
    spec!(
        HeavyLunge,
        "heavy.lunge m",
        "HEAVY_LUNGE",
        0.6,
        0.0,
        3.0,
        0.1
    ),
    // 넉백 배율(평타 `knockback` 기준).
    spec!(
        HeavyKnock,
        "heavy.knock x",
        "HEAVY_KNOCK",
        2.0,
        0.0,
        5.0,
        0.25
    ),
    // 다 모은 강타의 휘청. 덜 모으면 반.
    spec!(
        HeavyStagger,
        "heavy.stagger s",
        "HEAVY_STAGGER",
        0.8,
        0.0,
        3.0,
        0.1
    ),
    // 보스는 다 모은 강타에만.
    spec!(
        HeavyBossStagger,
        "heavy.boss stagger s",
        "HEAVY_BOSS_STAGGER",
        0.25,
        0.0,
        2.0,
        0.05
    ),
    // 떼는 순간 든다(평타 값은 이미 냈다).
    spec!(
        HeavyStamina,
        "heavy.stamina",
        "HEAVY_STAMINA",
        15.0,
        0.0,
        60.0,
        1.0
    ),
    // P4·P5 스파이크(0013): 잡몹을 상자 대신 glTF 시험 모델(부위 강체 + 걷기 애니메이션, 한 배치)로 그린다. 측정용 — 0 = 상자.
    spec!(
        RenderModel,
        "render.model 0box 1gltf",
        "RENDER_MODEL",
        0.0,
        0.0,
        1.0,
        1.0
    ),
];

/// 땅 돌리기의 두 구간(초): 90°+지나침까지 가는 시간, 90°로 되돌아오는 시간. 카메라(`Camera::turn`)와 월드(쏟는 때)가
/// 같은 식을 쓴다 — 월드는 카메라를 모르지만 "멈추는 순간"을 같이 알아야 한다(0012).
pub fn turn_phases(t: &Tuning) -> (f32, f32) {
    let over = t[P::TurnOver];
    let go = t[P::TurnTime] * (90.0 + over) / 90.0;
    let settle = if over > 0.0 { t[P::TurnSettle] } else { 0.0 };
    (go, settle)
}

/// 점프 포물선(0027): (처음 위 속도, 중력). 높이 h·체공 T → v = 4h/T, g = 8h/T².
pub fn jump_ballistics(t: &Tuning) -> (f32, f32) {
    let (h, air) = (t[P::JumpHeight], t[P::JumpTime].max(0.05));
    (4.0 * h / air, 8.0 * h / (air * air))
}

/// 180°로 돌 때(0023) 가는 시간을 이만큼 늘린다(90°와 같은 빠르기면 너무 빠르다 — 0.8초에 180°는 어지럽다).
pub const TURN_HALF_MUL: f32 = 1.6;
/// 카메라는 그대로고 몸만 돌아설 때(0023, 왼쪽을 보고 오른쪽 버튼): 쏟기까지 `turn.camera s`의 이만큼.
pub const TURN_SPIN_ONLY: f32 = 0.4;

/// 카메라가 `quarters`칸(90°씩) 도는 한 번의 (가는 초, 되돌아오는 초). 0칸이면 몸만(되돌아오기 없음).
pub fn turn_phases_q(t: &Tuning, quarters: i32) -> (f32, f32) {
    let (go, settle) = turn_phases(t);
    match quarters.abs() {
        0 => (t[P::TurnTime] * TURN_SPIN_ONLY, 0.0),
        1 => (go, settle),
        _ => (go * TURN_HALF_MUL, settle),
    }
}

/// `quarters`칸 돌기 시작에서 멈추기(쏟기)까지(초).
pub fn turn_land_secs_q(t: &Tuning, quarters: i32) -> f32 {
    let (go, settle) = turn_phases_q(t, quarters);
    go + settle
}

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

pub fn idx(p: P) -> usize {
    // SPECS 순서로 찾는다. 57개라 선형이어도 된다(틱마다 수십 번).
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

    /// 값 얼리기(0013): 기본값은 `docs/tuning-frozen.txt`와 같아야 한다. 바꾸려면 랩 A/B → export → 판정(사람) → 결정 기록 →
    /// 이 파일을 함께 고친다(`FREEZE_WRITE=1 cargo test frozen` 이 지금 기본값으로 다시 쓴다). 대화로 기본값이 줄줄이 바뀌던 것을 막는다.
    #[test]
    fn frozen_defaults_match_the_snapshot() {
        let now = Tuning::default().export();
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/docs/tuning-frozen.txt");
        if std::env::var("FREEZE_WRITE").is_ok() {
            std::fs::write(path, &now).unwrap();
        }
        let frozen = std::fs::read_to_string(path).expect("docs/tuning-frozen.txt");
        let diff: Vec<_> = now
            .lines()
            .zip(frozen.lines())
            .filter(|(a, b)| a != b)
            .map(|(a, b)| format!("now    {a}\nfrozen {b}"))
            .collect();
        assert!(
            diff.is_empty() && now.lines().count() == frozen.lines().count(),
            "기본값이 얼린 값과 다르다 — 판정·결정 기록 없이 바꾸지 않는다(0013):\n{}",
            diff.join("\n")
        );
    }

    /// 사람에게 "랩을 열고 ↑ 한 번 = `render.model`"로 안내한다(0013) — 새 값은 그 앞에 넣는다.
    #[test]
    fn render_model_stays_the_last_lab_entry() {
        assert_eq!(SPECS.last().unwrap().p, P::RenderModel);
    }

    #[test]
    fn set_clamps_and_export_marks_defaults() {
        let mut t = Tuning::default();
        t.set(0, 1000.0);
        assert_eq!(t.get(0), SPECS[0].max);
        let e = t.export();
        assert!(e.contains("CAM_Q_PITCH_DEG: f32 = 85.000;\n"));
        let dist = SPECS[idx(P::CamQDist)].def;
        assert!(e.contains(&format!("CAM_Q_DIST: f32 = {dist:.3};  // (default)")));
    }
}
