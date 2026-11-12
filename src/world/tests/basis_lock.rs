//! 입력 기준과 락온: 쿼터뷰·락온 기준, 락온 켜고 끄기·풀린 뒤의 걸음, 대상이 쓰러졌을 때.
use super::*;

#[test]
fn quarter_basis_moves_up_toward_minus_z_and_lock_basis_toward_target() {
    let m = move_to_world(vec2(0.0, 1.0), Basis::Quarter { yaw: 0.0 }, Vec3::ZERO);
    assert!(m.z < -0.99 && m.x.abs() < 1e-6);
    let m = move_to_world(vec2(0.0, 1.0), Basis::Lock(vec3(5.0, 0.0, 0.0)), Vec3::ZERO);
    assert!(m.x > 0.99, "forward is toward the target: {m:?}");
    let r = move_to_world(
        vec2(1.0, 0.0),
        Basis::Lock(vec3(0.0, 0.0, -5.0)),
        Vec3::ZERO,
    );
    assert!(r.x > 0.99, "right of a target in -z is +x: {r:?}");
}

#[test]
fn quarter_basis_follows_the_camera_yaw_so_stick_up_is_screen_up() {
    // 요 90°: 카메라가 +x 쪽에서 본다 → 화면 위 = -x.
    let m = move_to_world(vec2(0.0, 1.0), Basis::Quarter { yaw: 90.0 }, Vec3::ZERO);
    assert!(m.x < -0.99 && m.z.abs() < 1e-4, "{m:?}");
    let r = move_to_world(vec2(1.0, 0.0), Basis::Quarter { yaw: 90.0 }, Vec3::ZERO);
    assert!(
        r.z < -0.99,
        "right of screen is -z when looking from +x: {r:?}"
    );
}

#[test]
fn lock_button_toggles_and_the_basis_switches_the_same_tick() {
    let t = Tuning::default();
    let mut w = world_with_boss();
    let ev = w.step(
        &Actions {
            lock: true,
            ..Default::default()
        },
        &t,
    );
    assert!(ev.contains(&Event::LockOn));
    assert_eq!(w.basis(), Basis::Lock(w.enemies[0].pos));
    let ev = w.step(
        &Actions {
            lock: true,
            ..Default::default()
        },
        &t,
    );
    assert!(ev.contains(&Event::LockOff));
    assert!(matches!(w.basis(), Basis::Quarter { .. }));
}

#[test]
fn releasing_lock_while_holding_the_stick_keeps_walking_the_same_way_until_the_stick_is_let_go() {
    let t = Tuning::default();
    let mut w = world_with_boss();
    w.player.pos = vec3(0.0, 0.0, -14.0); // 보스(0,-8) 위쪽 — 쿼터뷰 기준과 락온 기준이 거의 반대
    w.step(
        &Actions {
            lock: true,
            ..Default::default()
        },
        &t,
    );
    let back = Actions {
        move_dir: vec2(0.0, -1.0),
        ..Default::default()
    }; // 보스에게서 물러나기
    for _ in 0..30 {
        w.step(&back, &t);
    }
    let before = w.player.vel.normalize();
    // 쥔 채로 풀기: 걷는 방향이 그대로.
    w.step(&Actions { lock: true, ..back }, &t);
    for _ in 0..5 {
        w.step(&back, &t);
    }
    assert!(w.lock.is_none());
    assert!(
        w.player.vel.normalize().dot(before) > 0.98,
        "{:?} vs {before:?}",
        w.player.vel
    );
    // 놓았다 다시 쥐면 쿼터뷰 기준(화면 아래 = 반대쪽).
    w.step(&Actions::default(), &t);
    for _ in 0..30 {
        w.step(&back, &t);
    }
    let quarter_down = move_to_world(
        vec2(0.0, -1.0),
        Basis::Quarter { yaw: t[P::CamQYaw] },
        w.player.pos,
    );
    assert!(w.player.vel.normalize().dot(quarter_down) > 0.98);
}

#[test]
fn the_unlock_hold_ends_by_itself_after_the_hold_time() {
    let t = Tuning::default();
    let mut w = world_with_boss();
    w.player.pos = vec3(0.0, 0.0, -14.0);
    w.step(
        &Actions {
            lock: true,
            ..Default::default()
        },
        &t,
    );
    let back = Actions {
        move_dir: vec2(0.0, -1.0),
        ..Default::default()
    };
    w.step(&back, &t);
    w.step(&Actions { lock: true, ..back }, &t);
    let ticks = (t[P::MoveHoldMax] / TICK_DT).ceil() as usize + 2;
    for _ in 0..ticks {
        w.step(&back, &t);
    }
    assert!(w.hold.is_none());
}

#[test]
fn lock_drops_when_the_target_dies() {
    let t = Tuning::default();
    let mut w = World::new("test", 20.0);
    w.spawn_mob(vec3(0.0, 0.0, -1.0));
    w.step(
        &Actions {
            lock: true,
            ..Default::default()
        },
        &t,
    );
    assert!(w.lock.is_some());
    w.enemies[0].state = EState::Dead;
    let ev = w.step(&Actions::default(), &t);
    assert!(ev.contains(&Event::LockOff) && w.lock.is_none());
}

/// 2026-11-06: 락온한 적이 쓰러지면 그 자리에서 가장 가까운 적으로 넘어가고, 둘레에 아무도 없으면 풀린다.
#[test]
fn a_fallen_lock_target_hands_off_to_the_nearest_enemy_or_lets_go() {
    let t = Tuning::default();
    let mut w = World::new("test", 20.0);
    w.spawn_mob(vec3(0.0, 0.0, -4.0)); // 0: 락온 대상
    w.spawn_mob(vec3(9.0, 0.0, 0.0)); // 1: 플레이어에게는 가깝지만 대상 자리에서 멀다
    w.spawn_mob(vec3(2.0, 0.0, -7.0)); // 2: 대상 자리에서 가장 가깝다
    w.lock = Some(0);
    w.enemies[0].hp = 0;
    w.enemies[0].state = EState::Dead;
    let ev = w.step(&Actions::default(), &t);
    assert_eq!(w.lock, Some(2), "next to where the target fell");
    assert!(!ev.contains(&Event::LockOff), "stays in lock-on");
    // 그 적도 쓰러지고, 남은 적(1)은 둘레 밖 → 풀린다.
    w.enemies[2].hp = 0;
    w.enemies[2].state = EState::Dead;
    let ev = w.step(&Actions::default(), &t);
    assert_eq!(w.lock, None);
    assert!(ev.contains(&Event::LockOff));
}
