//! 수치는 전부 여기(nail 규칙 그대로). 랩(`lab.rs`)이 실행 중에 바꾸는 값은 `P`로 이름 붙은 것들이고,
//! `SPECS`의 `def`가 곧 기본값이다. 랩에서 "export"하면 `pub const` 줄을 콘솔에 찍는다 → 사람이 여기에 붙여 넣고 커밋한다
//! (저장 파일을 게임이 읽게 하지 않는다: 진실은 한 곳, KB guides/poc-and-lab-before-content.md).
use std::ops::Index;

/// 로직 고정 타임스텝(60Hz).
pub const TICK_DT: f32 = 1.0 / 60.0;
/// 한 프레임에 돌리는 로직 시간 상한(탭 전환 뒤 폭주 방지).
pub const MAX_FRAME_DT: f32 = 0.1;
/// 내부 해상도(PS1 느낌). 정수 배율로 키운다. 0041: 480×270 → 640×360 — 720p(×2)·1080p(×3)·1440p(×4)·4K(×6)
/// 모두 정수 배율로 꽉 차고(480×270은 720p에서 ×2로 둘레가 검었다), 캐릭터가 20 → 27px(쿼터뷰)이라 동작이 읽힌다.
pub const INTERNAL_W: u32 = 640;
pub const INTERNAL_H: u32 = 360;
/// UI(글·HUD)는 480×270 기준으로 짰다 — UI 배율 = 렌더 배율 × INTERNAL_H / 이 값이라 화면 속 글 크기는 해상도를 올려도 같다(0041).
pub const UI_REF_H: f32 = 270.0;
/// 플레이어 키(m). 카메라 높이·주시점의 기준.
pub const PLAYER_HEIGHT: f32 = 1.8;
// 걸을 수 있는 높낮이(0043, `ground.rs`).
/// 걸어서 오르내리는 높이(m) — 이보다 높은 칸은 벽(점프로 발이 거기까지 오르면 넘는다), 적에게는 이보다 낮은 칸도 벽.
pub const GROUND_STEP: f32 = 0.3;
/// 적이 뛰어오르는 가장 높은 턱·뛰어내리는 가장 깊은 낙차(m, 0100). 플레이어는 점프로 0.5m 턱을 오르고(1m는 못 오름) 어디서든 뛰어내린다.
pub const ENEMY_CLIMB: f32 = 1.0;
pub const ENEMY_DROP: f32 = 3.0;
/// 치고 맞는 높이 차 한도(m): 이보다 위·아래에 있는 것은 칼도 발톱도 안 닿는다(0.5m 턱 너머는 닿고, 1m 단 너머는 안 닿는다).
pub const HIT_DY: f32 = 0.75;

// 2차 동작 층의 고정값(0039). 랩 값은 `anim.*`.
/// 머리: 조금 늦고 덜 출렁인다(Hz, 감쇠비).
pub const SEC_HEAD_HZ: f32 = 3.5;
pub const SEC_HEAD_DAMP: f32 = 0.55;
/// 몸 기울기: 가속하는 쪽으로 — 거의 출렁이지 않게.
/// 박자 창(2026-11-07): 그 무기의 한 스윙 시간 ± 이 비율 안에 누른 평타 콤보는 스태미나를 쓰지 않는다(KB: 창은 박자에 비례, ±60~90ms).
pub const BEAT_WINDOW: f32 = 0.15;
/// 딱 맞춰 떼기(0078 — 0051에선 지팡이만, 사장 2026-11-07 "2"로 모든 무기의 규칙): 강타가 다 모인 뒤 이 초 안에 떼면
/// 피해 +1·자세 피해 ×1.5·넉백 ×1.5. 다 모은 뒤 더 쥐고 있어도 손해는 없다(덤만 없다).
pub const PERFECT_RELEASE: f32 = 0.15;
/// 박자 창의 가장 좁은 반폭(초, 0077): 쇠뇌 한 발 0.24초 × 15%는 ±36ms라 사람 손으로는 못 맞춘다 — KB 창 범위 ±60~90ms의 아래 끝.
pub const BEAT_WINDOW_MIN: f32 = 0.06;
/// 경직 이음 창(0099): 경직이 끝난 뒤 이만큼(초) 안에 또 맞아도 경직이 그전의 반 — 이어 패링(0081)의 반대 방향.
pub const STUN_CHAIN_GRACE: f32 = 0.5;
/// 경직 이음의 바닥(0099, 사장 "바닥을 두자"): 이은 수가 이만큼에서 멈춘다 — 경직은 처음의 0.5^2 = 1/4 아래로 줄지 않는다.
pub const STUN_CHAIN_MAX: i32 = 2;
/// 기력이 모자라 막혔을 때 알리는 시간(초, 0088): 막대 깜빡·떨림, 땀방울.
pub const STAMINA_DENIED_SHOW: f32 = 0.5;
/// 박자 부채꼴(0087): 박자 순간에 남는 몫(가장 작음 — 0이면 맞출 것이 안 보인다), 사격의 박자 부채꼴 거리(m)·각(도).
pub const BEAT_CUE_MIN: f32 = 0.12;
pub const BEAT_CUE_SHOT_REACH: f32 = 2.0;
pub const BEAT_CUE_SHOT_ARC: f32 = 30.0;
/// 달리기(0076, 사장 2026-11-07 "회피 꾹 눌렀을때 달리는거"): 회피가 끝났을 때 회피 버튼을 아직 쥐고 있으면 달린다 —
/// 걷기 × 이 배율, 달리는 동안 초당 이만큼 스태미나를 쓰고(차지 않는다) 0이면 걷는다. 소울류 달리기(KB nail 0209 회피 쥐기).
pub const RUN_SPEED_X: f32 = 1.45;
pub const RUN_STAMINA: f32 = 18.0;
/// 대시 공격(2026-11-06): 회피가 끝난 뒤 이 초 안에 누른 공격도 대시 공격.
pub const DASH_ATTACK_LATE: f32 = 0.15;
/// 이어 패링(0081, 사장 2026-11-07 "패링 성공하면 패링시간이 조금 연장 되면서 비슷한 타이밍의 공격들도 같이"):
/// 튕긴 뒤 이 초 동안 패링 창이 열려 있다 — 그 사이 앞쪽에서 오는 받을 수 있는 공격도 튕긴다(튕길 때마다 다시 이만큼).
/// 반격(공격)을 시작하면 닫힌다(`since_parry`를 반격이 쓴다).
pub const PARRY_CHAIN: f32 = 0.2;
/// 이어 패링 쌓기(0086, 사장 "연장 중에 또 성공하면 남은 창이 또 늘되 늘어나는 몫은 50%씩 줄고, 범위는 늘어나게 — 둘러싸였을 때도
/// 빠져나가게"): 이어지는 동안 n번째 더 튕기면 남은 창에 `PARRY_CHAIN × PARRY_CHAIN_DECAY^n`을 더하고(0.2 → +0.1 → +0.05 …),
/// 받는 반원은 한 번 더 튕길 때마다 반각 `PARRY_CHAIN_WIDEN_DEG`도씩 넓어진다(앞 ±78° → ±123° → 둘레 전부).
pub const PARRY_CHAIN_DECAY: f32 = 0.5;
pub const PARRY_CHAIN_WIDEN_DEG: f32 = 45.0;
/// 패링 반격(2026-11-06): 튕긴 뒤 이 초 안에 누른 공격은 다 모은 강타로 바로 나간다.
pub const RIPOSTE_WINDOW: f32 = 0.6;
/// 락온 대상이 쓰러지면 그 자리에서 이 반지름 안의 가장 가까운 적으로 넘어간다(없으면 풀림, 2026-11-06).
pub const LOCK_HANDOFF: f32 = 8.0;
pub const SEC_LEAN_HZ: f32 = 2.5;
pub const SEC_LEAN_DAMP: f32 = 0.8;
/// 늦음(m) → 기울기 변위 배율. 몸은 반대쪽(가속하는 쪽)으로.
pub const SEC_LEAN_K: f32 = 0.5;
/// 덩어리가 늦게 따라올 수 있는 거리 한도(덩어리까지 거리에 대한 비).
pub const SEC_LAG_MAX: f32 = 0.7;
/// 찌그러짐 스프링: 착지에 찌그러졌다 한 번 늘어나며 선다(부피 그대로, Lasseter 1987).
pub const SEC_SQUASH_HZ: f32 = 3.5;
pub const SEC_SQUASH_DAMP: f32 = 0.3;
/// 착지 속도(m/s)마다 찌그러짐 속도(1/s).
pub const SEC_SQUASH_IMPULSE: f32 = 1.4;
/// 세로 속도(m/s)마다 늘어남(공중).
pub const SEC_STRETCH_PER_SPEED: f32 = 0.018;
/// 찌그러짐·늘어남 한도(세로 배율 1 ± 이 값).
pub const SEC_SQUASH_MAX: f32 = 0.3;

pub const PLAYER_RADIUS: f32 = 0.35;
/// 화면 흔들림(0020): trauma 1이면 카메라가 이 각(도)만큼 흔들린다 — 거리에 곱해 쿼터뷰(20m)·락온(7m)에서 화면 크기가 같다.
/// 예전엔 월드 ±0.25m 고정이라 20m 쿼터뷰에서 1px 남짓(사장: "피격 시에도 흔들림 있어야지").
pub const SHAKE_DEG: f32 = 1.6;
/// 사건마다 더하는 trauma(크기는 제곱으로 쓴다).
pub const SHAKE_HIT: f32 = 0.25;
/// 카메라 킥(2026-11-06, KB: 작은 타격은 흔들림 대신 킥 — 2~3px): 평타·치명타, 강타·자세 깨기(도), 돌아오는 빠르기(1/s).
pub const KICK_HIT_DEG: f32 = 0.3;
pub const KICK_SMASH_DEG: f32 = 0.55;
pub const KICK_RETURN: f32 = 22.0;
/// 평타 히트스톱 배율(`feel.hitstop s` × 이것, 2026-11-06): 0.06초(≈3.6프레임)는 "없는 것과 같다"(KB 사다리 — 가장 약한 타도 6프레임쯤).
pub const HIT_HITSTOP: f32 = 1.6;
/// 맞은 쪽: 몸이 맞은 쪽으로 젖는 세기(평타·강타, `Enemy::sway`), 멈춤 동안 떨림(m).
pub const HIT_SWAY: f32 = 0.35;
pub const HEAVY_SWAY: f32 = 0.65;
pub const HIT_TREMBLE: f32 = 0.05;
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
/// 등잔 횟수(0031). 화로에서 다시 찬다 — 화로 전까지는 스테이지를 새로 시작할 때. 성장으로 늘린다(items.md 2).
pub const FLASK_CHARGES: i32 = 2;
/// 마법 칸(0048 → 0111): 한 칸이 `cast.cooldown s`마다 차고(수류탄과 같은 꼴), 쉼터에서 가득. 한 번 쏘면 한 칸.
pub const CAST_CHARGES: f32 = 3.0;
/// 잔불(0064 M1, `ember.rs`): 한 번에 흩는 조각 수 상한, 튀는 빠르기(옆·위 m/s)·중력·튀어 오름 비, 자석이 켜지는 때(초)·
/// 닿는 거리(m)·끌림 가속(m/s²)·최고 빠르기, 줍는 거리(m). 적마다 떨구는 값은 `species.rs`의 `embers`.
pub const EMBER_PIECES: u32 = 6;
pub const EMBER_POP_SIDE: f32 = 2.2;
pub const EMBER_POP_UP: f32 = 4.5;
pub const EMBER_GRAVITY: f32 = 16.0;
pub const EMBER_BOUNCE: f32 = 0.35;
pub const EMBER_DELAY: f32 = 0.35;
pub const EMBER_MAGNET_R: f32 = 6.0;
pub const EMBER_PULL: f32 = 40.0;
pub const EMBER_PULL_MAX: f32 = 16.0;
pub const EMBER_PICK_R: f32 = 0.7;
/// 마법 탄 반지름(m)과 쏘는 높이(몸 위 m).
pub const BOLT_RADIUS: f32 = 0.35;
pub const BOLT_HEIGHT: f32 = 1.0;
/// 마법을 쏜 직후 땅 돌리기 누름을 버리는 시간(초) — RB 바로 밑이 RT(돌리기)라 잘못 눌러 세상이 도는 것을 막는다(0048).
pub const CAST_TURN_GUARD: f32 = 0.25;
/// 공중 RB(0111): 쏘는 순간 떨어지기를 멈추는 초(체공), 겨눌 적이 없을 때 앞으로 내려 쏘는 거리(m)·가장 가까운 거리(m),
/// 쇠뇌의 내려 쏘기 볼트 수·부채꼴 반각(도). 한 점프에 한 번.
pub const AIR_SHOT_HOVER: f32 = 0.2;
pub const AIR_SHOT_REACH: f32 = 4.0;
pub const AIR_SHOT_MIN: f32 = 1.5;
pub const AIR_VOLLEY: usize = 3;
pub const AIR_VOLLEY_SPREAD_DEG: f32 = 18.0;
/// 알아차림(0113): 지도에 서 있는 적은 제자리에서 제 일을 하다가 플레이어가 시야(m, 벽·턱 너머는 못 봄) 안에 들거나
/// 아주 가까이(m, 벽 너머도 — 발소리) 오거나 맞으면 쫓는다. 높이 차가 이보다 크면 못 본다. 깨면 곁(m)의 잠든 동료도 깬다.
/// 제자리에서 끈(m)보다 멀어지고 플레이어도 그만큼 멀면 제자리로 돌아가 가득 찬다.
pub const SIGHT_RANGE: f32 = 9.0;
pub const HEAR_RANGE: f32 = 3.0;
pub const SIGHT_DY: f32 = 2.5;
pub const ALERT_RADIUS: f32 = 4.5;
pub const LEASH_RANGE: f32 = 18.0;
/// 엘리트(0112): 체력 배율(올림)·몸 크기 배율·떨구는 잔불 배율, 처음 쓰러뜨렸을 때 더 흩는 잔불.
pub const ELITE_HP: f32 = 2.0;
pub const ELITE_SCALE: f32 = 1.25;
pub const ELITE_EMBERS: u32 = 2;
pub const ELITE_FIRST_EMBERS: u32 = 15;
/// 구역 등급(0112 1절 — 잡몹 레벨은 지역마다 고정 등급, 숫자는 화면에 없다): 체력·자세 막대·떨구는 잔불 배율(올림).
/// 예고 길이·움직임은 바꾸지 않는다(읽히는 싸움 0071·0075). 등급 2 값은 0116 3절. 등급 3 값은 셋째 지역 때 정한다.
pub struct Tier {
    pub hp: f32,
    pub poise: f32,
    pub embers: f32,
}
pub const TIERS: [Tier; 2] = [
    Tier {
        hp: 1.0,
        poise: 1.0,
        embers: 1.0,
    },
    Tier {
        hp: 1.5,
        poise: 1.3,
        // 0123: 1.6 → 1.3(아이템·성장 덜 후하게).
        embers: 1.3,
    },
];
/// 청소자(0120): 싸움이 없을 때 이 초마다 체력 1을 먹는다.
pub const EAT_TIME: f32 = 2.5;
/// 등딱지(0120): 앞을 친 평타가 튕기면 방랑자가 튀는 초·뒤로 밀리는 빠르기(m/s). 패링당한 것보다 짧게.
pub const RECOIL_TIME: f32 = 0.35;
pub const RECOIL_PUSH: f32 = 3.0;
/// 꼬투리(0120): 깡충 물러나는 거리(m)·다시 뛰기까지 쉼(초).
pub const POD_HOP: f32 = 3.2;
pub const POD_HOP_CD: f32 = 2.6;
/// 이 등급부터 잡몹 한 대 피해가 2(0112).
pub const TIER_HEAVY_HITS: u8 = 3;

/// 등급 `n`(1부터)의 배율 — 표 밖이면 가장 높은 줄.
pub fn tier(n: u8) -> &'static Tier {
    &TIERS[(n.max(1) as usize - 1).min(TIERS.len() - 1)]
}

/// 굳은(0112): 자세 막대 배율·밀려나는 배율.
pub const ELITE_HARD_POISE: f32 = 2.5;
pub const ELITE_HARD_KNOCK: f32 = 0.35;
/// 남는(0112): 쓰러진 자리에 남는 고리 — 터지기까지의 초(그동안 예고)·반지름(m).
pub const ELITE_LINGER_DELAY: f32 = 1.0;
pub const ELITE_LINGER_RADIUS: f32 = 2.6;
/// 수류탄(0051): 중력(m/s²), 땅에서 튕길 때 남는 위·옆 속도 비, 쇠뇌 볼트가 수류탄을 맞히는 거리(m), 폭발 고리가 보이는 초.
pub const BOMB_GRAVITY: f32 = 22.0;
pub const BOMB_BOUNCE: f32 = 0.4;
pub const BOMB_SKID: f32 = 0.6;
pub const BOMB_SHOT_REACH: f32 = 0.8;
pub const BLAST_SHOW: f32 = 0.45;
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
    HeavyFullBonus,
    HeavyRange,
    HeavyArc,
    HeavyLunge,
    HeavyKnock,
    HeavyStagger,
    HeavyBossStagger,
    HeavyStamina,
    HeavyHoldStart,
    FlaskHeal,
    FlaskRaise,
    FlaskTime,
    FlaskLock,
    FlaskWalk,
    RenderToon,
    RenderRim,
    RenderOutline,
    AnimTailGain,
    AnimTailHz,
    AnimTailDamp,
    AnimHeadGain,
    AnimLeanGain,
    AnimSquash,
    RenderLockFade,
    RenderActorShadow,
    HeavyChargeWalk,
    CastSpeed,
    CastRange,
    CastCooldown,
    CastStartup,
    CastTime,
    CastWalk,
    ComboKeep,
    ComboLink,
    AttackBuffer,
    PoiseBase,
    PoisePerHp,
    PoiseBoss,
    PoiseBreak,
    PoiseBreakBoss,
    PoiseLock,
    PoiseDelay,
    PoiseDecay,
    ThrowStartup,
    ThrowTime,
    PoiseBreakDamage,
    CastHoldStart,
    CastCharge,
    DodgeCancel,
    LandCancel,
    RenderEdge,
    RenderEdgeDark,
    RenderEdgeLight,
    RenderEdgeDepth,
    RenderEdgeCrease,
    RenderFog,
    RenderMotes,
    RenderSky,
    RenderFlora,
    TurnWobbleIn,
    TurnWobbleOut,
    TurnStartStagger,
    ParryStaminaGain,
}

/// 랩이 읽는 칸(이름·걸음·범위·내보내기 이름)은 개발판에만 쓰인다(운영판에는 랩이 없다, 0070).
#[cfg_attr(not(feature = "dev"), allow(dead_code))]
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
pub const SPECS: [Spec; 147] = [
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
        4.5,
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
    // 0117(사장 "기력 충전이 너무 빨라 행동에 제약이 없다"): 45 → 28/s, 차기 시작 0.6 → 0.8초. 빈 막대가 다 차는 데 약 2.8 → 4.4초.
    spec!(
        StaminaRegen,
        "stamina.regen /s",
        "STAMINA_REGEN",
        28.0,
        5.0,
        200.0,
        5.0
    ),
    spec!(
        StaminaDelay,
        "stamina.regen delay s",
        "STAMINA_DELAY",
        0.8,
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
        // 15 → 5(0097, 사장 2026-11-07 "이미 타이밍 손해 및 뎀지 손해가 있는데 기력까지 소모하는건 너무 가혹해").
        5.0,
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
    // 한 번 누른 채 이만큼 쥐어야 모으기로(0030). 사람 톡 누름 0.08~0.15초(KB)보다 넉넉히 — 연타가 새지 않게.
    spec!(
        HeavyHoldStart,
        "heavy.hold start s",
        "HEAVY_HOLD_START",
        0.25,
        0.1,
        0.6,
        0.01
    ),
    // 등잔(0031): 한 모금에 체력, 들기(이 전에 회피·피격이면 끊기고 등잔 그대로), 전체 길이, 끝난 뒤 누름을 버리는 시간, 마시며 걷는 빠르기 배율.
    spec!(FlaskHeal, "flask.heal", "FLASK_HEAL", 2.0, 1.0, 5.0, 1.0),
    spec!(
        FlaskRaise,
        "flask.raise s",
        "FLASK_RAISE",
        0.45,
        0.05,
        1.0,
        0.05
    ),
    spec!(FlaskTime, "flask.time s", "FLASK_TIME", 1.0, 0.2, 2.0, 0.05),
    spec!(FlaskLock, "flask.lock s", "FLASK_LOCK", 0.3, 0.0, 1.0, 0.05),
    spec!(
        FlaskWalk,
        "flask.walk x",
        "FLASK_WALK",
        0.35,
        0.0,
        1.0,
        0.05
    ),
    // 모델 다듬기 실험(0033): 빛을 단계로 끊기(달빛 2단·화로 3단 — 지형은 다시 굽는다), 배우 가장자리 빛(카메라에서 비스듬한 면),
    // 배우 외곽선 두께(내부 해상도 px, 부위 상자를 키운 뒷면 — 0 = 끔).
    spec!(
        RenderToon,
        "render.toon 0off 1on",
        "RENDER_TOON",
        1.0,
        0.0,
        1.0,
        1.0
    ),
    spec!(RenderRim, "render.rim", "RENDER_RIM", 0.35, 0.0, 1.5, 0.05),
    spec!(
        RenderOutline,
        "render.outline px",
        "RENDER_OUTLINE",
        1.0,
        0.0,
        3.0,
        0.25
    ),
    // 2차 동작 층(0039, `secondary.rs`): 꼬리·머리가 늦게 따라오고, 몸은 가속하는 쪽으로 기울고, 점프·착지에 늘어나고 찌그러진다.
    // 이득 0 = 끔. 꼬리는 출렁이게(감쇠비 낮게).
    spec!(
        AnimTailGain,
        "anim.tail gain",
        "ANIM_TAIL_GAIN",
        1.0,
        0.0,
        3.0,
        0.1
    ),
    spec!(
        AnimTailHz,
        "anim.tail hz",
        "ANIM_TAIL_HZ",
        2.4,
        0.5,
        8.0,
        0.1
    ),
    spec!(
        AnimTailDamp,
        "anim.tail damp",
        "ANIM_TAIL_DAMP",
        0.3,
        0.05,
        1.5,
        0.05
    ),
    spec!(
        AnimHeadGain,
        "anim.head gain",
        "ANIM_HEAD_GAIN",
        1.0,
        0.0,
        3.0,
        0.1
    ),
    spec!(
        AnimLeanGain,
        "anim.lean gain",
        "ANIM_LEAN_GAIN",
        1.0,
        0.0,
        3.0,
        0.1
    ),
    spec!(AnimSquash, "anim.squash", "ANIM_SQUASH", 1.0, 0.0, 3.0, 0.1),
    // 락온 중 방랑자가 과녁을 가리면 이만큼 옅게(0040, 1 = 끔). 카메라를 비켜 둬도 가까운 큰 대상은 다 못 비켜난다.
    spec!(
        RenderLockFade,
        "render.lock fade",
        "RENDER_LOCK_FADE",
        0.45,
        0.1,
        1.0,
        0.05
    ),
    // 캐릭터 실시간 그림자(0045): 1 = 달 쪽 가림판으로 바닥에 그림자, 0 = 끔(원판만 — 핸드헬드에서 무거우면).
    spec!(
        RenderActorShadow,
        "render.actor shadow",
        "RENDER_ACTOR_SHADOW",
        1.0,
        0.0,
        1.0,
        1.0
    ),
    // 강공격을 모으는 동안 걷는 빠르기(0047): 걷기의 배율 — 0이면 예전처럼 제자리(0030).
    spec!(
        HeavyChargeWalk,
        "heavy.charge walk x",
        "HEAVY_CHARGE_WALK",
        0.4,
        0.0,
        1.0,
        0.05
    ),
    // 마법(0048, RB): 피해·탄 빠르기·닿는 거리, 근접 한 대가 채우는 칸, 쏘기까지·끝까지의 시간, 쏘는 동안 걷기 배율.
    spec!(
        CastSpeed,
        "cast.speed m/s",
        "CAST_SPEED",
        16.0,
        4.0,
        40.0,
        1.0
    ),
    spec!(
        CastRange,
        "cast.range m",
        "CAST_RANGE",
        12.0,
        3.0,
        30.0,
        0.5
    ),
    // 마법 한 칸이 다시 차는 초(0111 — 수류탄과 같은 꼴, 사장 2026-11-08. 예전 `cast.per hit` 근접 명중 충전 0048을 갈음).
    spec!(
        CastCooldown,
        "cast.cooldown s",
        "CAST_COOLDOWN",
        3.0,
        0.5,
        10.0,
        0.25
    ),
    spec!(
        CastStartup,
        "cast.startup s",
        "CAST_STARTUP",
        0.12,
        0.0,
        0.6,
        0.02
    ),
    spec!(CastTime, "cast.time s", "CAST_TIME", 0.4, 0.1, 1.5, 0.05),
    spec!(CastWalk, "cast.walk x", "CAST_WALK", 0.5, 0.0, 1.0, 0.05),
    // 콤보(0051): 콤보가 끝난 뒤 이 초 안에 누르면 다음 타부터, 후딜의 이만큼(비율)이 지나야 다음 타로 잇는다, 누름을 미리 받아 두는 초.
    spec!(
        ComboKeep,
        "combo.keep s",
        "COMBO_KEEP",
        0.45,
        0.0,
        1.5,
        0.05
    ),
    spec!(ComboLink, "combo.link x", "COMBO_LINK", 0.3, 0.0, 1.0, 0.05),
    spec!(
        AttackBuffer,
        "attack.buffer s",
        "ATTACK_BUFFER",
        0.3,
        0.0,
        1.0,
        0.05
    ),
    // 자세 막대(0051): 잡몹 크기 = 바탕 + 체력 × 배, 보스는 따로. 깨지면 휘청(잡몹·보스), 깬 뒤 안 차는 초, 줄기 시작까지·줄어드는 빠르기.
    spec!(PoiseBase, "poise.base", "POISE_BASE", 3.0, 0.5, 20.0, 0.5),
    spec!(
        PoisePerHp,
        "poise.per hp",
        "POISE_PER_HP",
        0.6,
        0.0,
        3.0,
        0.1
    ),
    spec!(PoiseBoss, "poise.boss", "POISE_BOSS", 20.0, 2.0, 80.0, 1.0),
    spec!(
        PoiseBreak,
        "poise.break s",
        "POISE_BREAK",
        1.5,
        0.2,
        4.0,
        0.1
    ),
    spec!(
        PoiseBreakBoss,
        "poise.break boss s",
        "POISE_BREAK_BOSS",
        1.0,
        0.2,
        4.0,
        0.1
    ),
    spec!(PoiseLock, "poise.lock s", "POISE_LOCK", 3.0, 0.0, 10.0, 0.5),
    spec!(
        PoiseDelay,
        "poise.delay s",
        "POISE_DELAY",
        2.0,
        0.0,
        10.0,
        0.25
    ),
    spec!(
        PoiseDecay,
        "poise.decay /s",
        "POISE_DECAY",
        1.0,
        0.0,
        10.0,
        0.25
    ),
    // 수류탄 던지기(0051, 쇠뇌의 X): 손을 떠나는 순간, 끝.
    spec!(
        ThrowStartup,
        "throw.startup s",
        "THROW_STARTUP",
        0.2,
        0.0,
        0.8,
        0.02
    ),
    spec!(ThrowTime, "throw.time s", "THROW_TIME", 0.5, 0.1, 1.5, 0.05),
    // 자세가 깨진 동안 받는 피해 배율(0055 — 폭딜 창, 올림).
    spec!(
        PoiseBreakDamage,
        "poise.break damage x",
        "POISE_BREAK_DAMAGE",
        1.5,
        1.0,
        4.0,
        0.25
    ),
    // 원거리 강공격(0056): RB를 한 번 누른 채 이만큼 쥐면(쏜 뒤) 모으기로, 다 모이는 시간.
    spec!(
        CastHoldStart,
        "cast.hold start s",
        "CAST_HOLD_START",
        0.25,
        0.05,
        1.0,
        0.05
    ),
    spec!(
        CastCharge,
        "cast.charge s",
        "CAST_CHARGE",
        0.7,
        0.1,
        2.0,
        0.05
    ),
    // 캔슬(0057): 회피는 이 비율이 지나면(무적이 그만큼 일찍 끝난다) 다음 행동으로, 착지 빈틈도 이 비율 뒤에.
    spec!(
        DodgeCancel,
        "dodge.cancel x",
        "DODGE_CANCEL",
        0.7,
        0.0,
        1.0,
        0.05
    ),
    spec!(
        LandCancel,
        "land.cancel x",
        "LAND_CANCEL",
        0.5,
        0.0,
        1.0,
        0.05
    ),
    // 화면 공간 선 둘(0061 t3ssel8r 풍): 법선 패스에서 — 깊이가 끊기는 앞쪽 픽셀은 어둡게(물체 가장자리), 법선이 꺾이는 볼록 모서리는 밝게.
    // 깊이 문턱은 거리에 대한 비(0.06 = 그 거리의 6% 넘게 뒤가 멀면 가장자리), 꺾임 문턱은 이웃 법선(화면 xy) 차의 길이.
    spec!(
        RenderEdge,
        "render.edge 0off 1on",
        "RENDER_EDGE",
        1.0,
        0.0,
        1.0,
        1.0
    ),
    spec!(
        RenderEdgeDark,
        "render.edge dark",
        "RENDER_EDGE_DARK",
        0.75,
        0.0,
        1.0,
        0.05
    ),
    spec!(
        RenderEdgeLight,
        "render.edge light",
        "RENDER_EDGE_LIGHT",
        0.45,
        0.0,
        1.0,
        0.05
    ),
    spec!(
        RenderEdgeDepth,
        "render.edge depth",
        "RENDER_EDGE_DEPTH",
        0.06,
        0.01,
        0.3,
        0.01
    ),
    spec!(
        RenderEdgeCrease,
        "render.edge crease",
        "RENDER_EDGE_CREASE",
        0.6,
        0.05,
        1.5,
        0.05
    ),
    // 주위 환경(0063): 핸드헬드 fps(P5)를 보며 하나씩 끄고 켜 본다 — 안개 판·떠다니는 것·하늘과 원경·식생(식생은 끄면 지형을 다시 굽는다).
    spec!(
        RenderFog,
        "render.fog 0off 1on",
        "RENDER_FOG",
        1.0,
        0.0,
        1.0,
        1.0
    ),
    spec!(
        RenderMotes,
        "render.motes 0off 1on",
        "RENDER_MOTES",
        1.0,
        0.0,
        1.0,
        1.0
    ),
    spec!(
        RenderSky,
        "render.sky 0off 1on",
        "RENDER_SKY",
        1.0,
        0.0,
        1.0,
        1.0
    ),
    spec!(
        RenderFlora,
        "render.flora 0off 1on",
        "RENDER_FLORA",
        1.0,
        0.0,
        1.0,
        1.0
    ),
    // 땅 돌리기 휘청(0084, 사장 2026-11-07 "회전 시작시에 작은 휘청임, 마지막에 작은 휘청임"): 화면이 돌기 시작할 때 도는 쪽으로,
    // 멈출 때 반대쪽으로 카메라가 살짝 기울었다 흔들리며 선다(굴림 각, 도). 0 = 없음.
    spec!(
        TurnWobbleIn,
        "turn.wobble in deg",
        "TURN_WOBBLE_IN",
        1.5,
        0.0,
        6.0,
        0.25
    ),
    spec!(
        TurnWobbleOut,
        "turn.wobble out deg",
        "TURN_WOBBLE_OUT",
        1.5,
        0.0,
        6.0,
        0.25
    ),
    // 땅 돌리기 시작의 휘청은 CC도(0085, 사장 "화면돌리기의 휘청은 cc 포함"): 돌기 시작하는 순간 `spill.range m` 안의 적이
    // 이만큼(초) 휘청인다(예고가 끊긴다). 보스는 `spill.boss mass`로 나눈다. 끝의 휘청은 그대로 쏟기(`spill.stagger s`). 0 = 없음.
    spec!(
        TurnStartStagger,
        "turn.start stagger s",
        "TURN_START_STAGGER",
        0.25,
        0.0,
        1.0,
        0.05
    ),
    // 패링 성공의 기력(0088, 사장 2026-11-07 "패링 성공시 어느정도 기력 회복이 되어야 반격 바로 할수 있을듯"): 튕긴 틱마다 이만큼 찬다
    // (기본 25 = 패링 값 15 + 평타 값 12 남짓 — 0까지 쓰고 튕겨도 반격·회피 하나가 남는다). Wuchang·Lies of P처럼 맞받기가 기력을 돌려준다.
    spec!(
        ParryStaminaGain,
        "parry.stamina gain",
        "PARRY_STAMINA_GAIN",
        25.0,
        0.0,
        100.0,
        5.0
    ),
];

/// 땅 돌리기 휘청의 흔들림 빠르기(Hz)·줄어듦(/초) — 0.5초 안에 3% 아래로.
pub const TURN_WOBBLE_HZ: f32 = 3.5;
pub const TURN_WOBBLE_DAMP: f32 = 7.0;

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

#[cfg_attr(not(feature = "dev"), allow(dead_code))]
impl Tuning {
    pub fn get(&self, i: usize) -> f32 {
        self.v[i]
    }
    pub fn set(&mut self, i: usize, value: f32) {
        let s = &SPECS[i];
        self.v[i] = value.clamp(s.min, s.max);
    }
    /// 능력치가 휜 값(0064 M3, `stats::Stats::tune`) — 랩 범위로 묶지 않는다(장신구가 범위를 넘을 수 있다).
    pub fn bend(&mut self, p: P, value: f32) {
        self.v[idx(p)] = value;
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
