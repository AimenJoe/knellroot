use super::*;

/// 0034 → 0197: 락온 중 옆으로 걸으면 걷기의 다리를 가는 쪽으로 돌리고(+ = 모델의 왼쪽, 한도 `LEG_TURN_MAX`), 뒤로 걸으면 뒷걸음
/// (다리는 가는 쪽의 반대), 락온이 아니면 늘 걷기·돌림 없음.
#[test]
fn locked_on_sideways_steps_turn_the_legs_and_backwards_walks_back() {
    let facing = vec3(0.0, 0.0, -1.0); // 북을 본다 → 오른쪽 = +x
    let (n, a) = move_anim(vec3(1.0, 0.0, 0.0), facing, true);
    assert_eq!(n[0], "walk");
    assert!(
        (a + LEG_TURN_MAX).abs() < 1e-5,
        "right: legs turn right (model −) to the limit: {a}"
    );
    let (n, a) = move_anim(vec3(-1.0, 0.0, -1.0), facing, true);
    assert_eq!(n[0], "walk");
    assert!(
        (a - std::f32::consts::FRAC_PI_4).abs() < 1e-4,
        "front-left 45°: {a}"
    );
    let (n, a) = move_anim(vec3(0.0, 0.0, 1.0), facing, true);
    assert!(n[0] == "walk_back" && a.abs() < 1e-5);
    let (n, a) = move_anim(vec3(0.5, 0.0, 1.0), facing, true);
    assert!(
        n[0] == "walk_back" && a > 0.0,
        "back-right: legs turn so their back points there: {a}"
    );
    assert_eq!(
        move_anim(vec3(0.0, 0.0, -1.0), facing, true),
        (&["walk"][..], 0.0)
    );
    assert_eq!(
        move_anim(vec3(1.0, 0.0, 0.0), facing, false),
        (&["walk"][..], 0.0)
    );
}

/// 가짜 모델: 가진 동작 이름(길이는 모두 1초).
struct Has(&'static [&'static str]);
impl Clips for Has {
    fn dur(&self, name: &str) -> Option<f32> {
        self.0.contains(&name).then_some(1.0)
    }
}

/// 방랑자 모델이 가진 동작(이 시험이 보는 것만).
const FULL: Has = Has(&[
    "idle",
    "walk",
    "walk_back",
    "run",
    "dodge",
    "dodge_back",
    "dodge_rise",
    "parry_push",
    "xbow_shoot",
    "xbow_charge",
    "bow_shoot",
    "bow_draw",
    "sk_throw",
    "cast",
    "attack",
]);

fn world() -> World {
    let mut w = World::new("test", 30.0);
    w.player.parry.since = 99.0;
    w.player.prev = w.player.pos;
    w
}

/// 한 프레임(1/60초): 월드 시계를 옮기고 고른다 — 고른 동작 가운데 모델에 있는 첫 이름과 시각.
fn frame(m: &mut Memory, w: &mut World, clips: &Has) -> (&'static str, f32) {
    w.time += TICK_DT;
    let p = m.pick(w, &Tuning::default(), clips);
    let name = p
        .names()
        .iter()
        .copied()
        .find(|n| clips.has(n))
        .unwrap_or("-");
    (name, p.at)
}

fn walk(w: &mut World) {
    w.player.prev = w.player.pos - vec3(0.05, 0.0, 0.0);
}

/// 0197: 몸이 보는 쪽의 반대로 피하면 뒤로 뛰기, 아니면 앞구르기. 뒤로 뛰는 동작이 없는 모델이면 구르기로.
#[test]
fn a_dodge_against_the_facing_backsteps_and_a_forward_one_rolls() {
    let mut w = world();
    let mut m = Memory::default();
    let back = -w.player.facing_dir();
    w.player.state = PState::Dodge { t: 0.1, dir: back };
    assert_eq!(frame(&mut m, &mut w, &FULL).0, "dodge_back");
    assert_eq!(frame(&mut m, &mut w, &Has(&["dodge", "idle"])).0, "dodge");
    w.player.state = PState::Dodge {
        t: 0.1,
        dir: w.player.facing_dir(),
    };
    assert_eq!(frame(&mut m, &mut w, &FULL).0, "dodge");
}

/// 0238: 앞구르기가 끝나 서기로 들어가면 `dodge_rise`를 0초부터 `ROLL_RISE`까지 잇고, 그 뒤 숨 고리. 뒤로 뛴 뒤에는 잇지 않는다,
/// 일어나는 동작이 없는 모델이면 바로 서기.
#[test]
fn a_forward_roll_rises_into_standing_and_a_backstep_does_not() {
    let mut w = world();
    let mut m = Memory::default();
    let fwd = w.player.facing_dir();
    w.player.state = PState::Dodge { t: 0.3, dir: fwd };
    frame(&mut m, &mut w, &FULL);
    w.player.state = PState::Idle;
    assert_eq!(frame(&mut m, &mut w, &FULL), ("dodge_rise", 0.0));
    let mut last = 0.0;
    for _ in 0..((crate::gait::ROLL_RISE / TICK_DT) as usize + 2) {
        let (n, at) = frame(&mut m, &mut w, &FULL);
        if n != "dodge_rise" {
            assert_eq!(n, "idle");
            assert!(last > 0.15, "rose for the whole clip ({last})");
            break;
        }
        last = at;
    }
    assert_eq!(frame(&mut m, &mut w, &FULL).0, "idle");

    let mut m = Memory::default();
    w.player.state = PState::Dodge { t: 0.3, dir: -fwd };
    frame(&mut m, &mut w, &FULL);
    w.player.state = PState::Idle;
    assert_eq!(
        frame(&mut m, &mut w, &FULL).0,
        "idle",
        "no rise after a backstep"
    );

    let mut m = Memory::default();
    w.player.state = PState::Dodge { t: 0.3, dir: fwd };
    let plain = Has(&["idle", "dodge"]);
    frame(&mut m, &mut w, &plain);
    w.player.state = PState::Idle;
    assert_eq!(frame(&mut m, &mut w, &plain).0, "idle");
}

/// 0197: 쇠뇌를 쏜 뒤 서 있으면 `AIM_HOLD` 동안 쏘기의 끝 자세로 겨눈 채, 그 뒤 숨 고리. 움직이면 바로 내린다.
#[test]
fn after_a_crossbow_shot_standing_keeps_the_aim_for_a_while() {
    let mut w = world();
    w.player.weapon = crate::weapons::Id::Crossbow;
    let mut m = Memory::default();
    w.player.state = PState::Cast {
        t: 0.05,
        dir: w.player.facing_dir(),
        fired: true,
    };
    assert_eq!(frame(&mut m, &mut w, &FULL).0, "xbow_shoot");
    w.player.state = PState::Idle;
    assert_eq!(frame(&mut m, &mut w, &FULL), ("xbow_shoot", XBOW_HOLD_AT));
    for _ in 0..((AIM_HOLD / TICK_DT) as usize + 1) {
        frame(&mut m, &mut w, &FULL);
    }
    assert_eq!(
        frame(&mut m, &mut w, &FULL).0,
        "idle",
        "lowered after the hold"
    );

    let mut m = Memory::default();
    w.player.state = PState::Cast {
        t: 0.05,
        dir: w.player.facing_dir(),
        fired: true,
    };
    frame(&mut m, &mut w, &FULL);
    w.player.state = PState::Idle;
    walk(&mut w);
    assert_eq!(
        frame(&mut m, &mut w, &FULL).0,
        "walk",
        "moving drops the aim"
    );
    w.player.prev = w.player.pos;
    assert_eq!(
        frame(&mut m, &mut w, &FULL).0,
        "idle",
        "and it does not come back"
    );
}

/// 0231: 활은 제 동작(`bow_draw`·`bow_shoot`), 없는 모델이면 쇠뇌의 것으로.
#[test]
fn the_bow_uses_its_own_clips_and_falls_back_to_the_crossbows() {
    let mut w = world();
    w.player.weapon = crate::weapons::Id::Bow;
    let mut m = Memory::default();
    w.player.state = PState::CastCharge { t: 0.2 };
    assert_eq!(frame(&mut m, &mut w, &FULL).0, "bow_draw");
    let old = Has(&["idle", "xbow_charge", "xbow_shoot"]);
    assert_eq!(frame(&mut m, &mut w, &old).0, "xbow_charge");
    w.player.state = PState::Cast {
        t: 0.05,
        dir: w.player.facing_dir(),
        fired: true,
    };
    assert_eq!(frame(&mut m, &mut w, &FULL).0, "bow_shoot");
    assert_eq!(frame(&mut m, &mut w, &old).0, "xbow_shoot");
}

/// 0274 ②b: 던지는 RB(손도끼 · 창 · 돌)는 머리 위로 던지는 동작 — 손을 떠나는 자리가 탄이 나가는 때(`cast.startup`)에 온다.
/// 빛·마법은 손을 내미는 그대로. 활을 당기는 동작은 끝까지 당기는 시간(`cast.charge`)에 맞춰 늘어난다.
#[test]
fn thrown_rbs_throw_overhand_and_the_bow_draw_spans_the_full_draw() {
    let t = Tuning::default();
    let mut w = world();
    let mut m = Memory::default();
    let at_release = PState::Cast {
        t: t[crate::tuning::P::CastStartup],
        dir: w.player.facing_dir(),
        fired: true,
    };
    for (id, want) in [
        (crate::weapons::Id::Sword, "sk_throw"),
        (crate::weapons::Id::Spear, "sk_throw"),
        (crate::weapons::Id::Fists, "sk_throw"),
        (crate::weapons::Id::Hammer, "cast"),
        (crate::weapons::Id::Staff, "cast"),
    ] {
        w.player.weapon = id;
        w.player.state = at_release;
        let (name, at) = frame(&mut m, &mut w, &FULL);
        assert_eq!(name, want, "{id:?}");
        if want == "sk_throw" {
            assert!((at - TOSS_RELEASE).abs() < 1e-4, "{id:?}: {at}");
        }
    }
    w.player.weapon = crate::weapons::Id::Bow;
    w.player.state = PState::CastCharge {
        t: t[crate::tuning::P::CastCharge],
    };
    let (name, at) = frame(&mut m, &mut w, &FULL);
    assert_eq!(name, "bow_draw");
    assert!(
        (at - BOW_DRAW_CLIP).abs() < 1e-4,
        "fully drawn at the end of the clip: {at}"
    );
}

/// 0199 → 0202: 튕긴 뒤 `PARRY_PUSH_FIRM` 안에는 걸어도 거두는 팔, 그 뒤 `PARRY_PUSH_CLIP`까지는 서 있을 때만.
#[test]
fn the_parry_push_shows_while_walking_only_at_first() {
    let mut w = world();
    let mut m = Memory::default();
    w.player.parry.since = PARRY_PUSH_FIRM * 0.5;
    walk(&mut w);
    assert_eq!(frame(&mut m, &mut w, &FULL).0, "parry_push");
    w.player.parry.since = (PARRY_PUSH_FIRM + PARRY_PUSH_CLIP) * 0.5;
    assert_eq!(frame(&mut m, &mut w, &FULL).0, "walk");
    w.player.prev = w.player.pos;
    assert_eq!(frame(&mut m, &mut w, &FULL).0, "parry_push");
    w.player.parry.since = PARRY_PUSH_CLIP;
    assert_eq!(frame(&mut m, &mut w, &FULL).0, "idle");
}

/// 0175: 걸음 시계는 쌓는다 — 걷기 시작하는 프레임은 첫 자세에서, 서면 멈추고, 다시 걸으면 또 첫 자세에서.
/// 0197 → 0198: 뒤로 걸을 때(몸이 걷는 쪽이 아닌 것을 본다) 뒷걸음을 `WALK_BACK_RATE` 배로.
#[test]
fn the_step_clock_accumulates_and_restarts_and_backsteps_run_faster() {
    let mut w = world();
    let mut m = Memory::default();
    walk(&mut w);
    let (n0, a0) = frame(&mut m, &mut w, &FULL);
    let (_, a1) = frame(&mut m, &mut w, &FULL);
    assert_eq!(n0, "walk");
    assert!(a0 > 0.0 && a1 > a0, "it accumulates ({a0} → {a1})");
    w.player.prev = w.player.pos;
    frame(&mut m, &mut w, &FULL);
    walk(&mut w);
    let (_, b0) = frame(&mut m, &mut w, &FULL);
    assert!((b0 - a0).abs() < 1e-6, "restarts from the first pose");

    let mut m = Memory::default();
    w.player.prev = w.player.pos;
    frame(&mut m, &mut w, &FULL); // 시계를 맞춘다(새 기억의 첫 프레임은 지난 초를 모른다)
    w.player.faces_away = true;
    w.player.prev = w.player.pos + w.player.facing_dir() * 0.05;
    let (n, back) = frame(&mut m, &mut w, &FULL);
    assert_eq!(n, "walk_back");
    assert!((back / a0 - WALK_BACK_RATE).abs() < 1e-3);
}
