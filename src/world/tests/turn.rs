//! 땅 돌리기(0012): 90° 한 번, 멈출 때의 쏟아짐·휘청·벽 쿵, 대기와 락온, 돌린 뒤의 걸음.
use super::*;

/// 0034: 오른쪽 버튼의 몸 돌기는 `turn_is_right`(렌더가 `turn_r`을 고른다), 왼쪽은 아니다.
#[test]
fn the_turn_animation_side_follows_the_button() {
    use std::f32::consts::PI;
    for facing in [PI, 0.0, 1.0, -2.5] {
        let r = plan_turn(true, facing);
        assert!(turn_is_right(facing, r.facing), "{facing}");
        let l = plan_turn(false, facing);
        assert!(!turn_is_right(facing, l.facing), "{facing}");
    }
}

/// 0028: 캐릭터 기준 90° — 오른쪽 버튼이면 몸이 제 오른쪽으로 90°, 화면도 늘 오른쪽으로 90°(요 -90°). 바라보는 쪽과 무관.
#[test]
fn a_turn_is_always_ninety_degrees_the_buttons_way_for_body_and_screen() {
    use std::f32::consts::{FRAC_PI_2, PI};
    let close = |a: f32, b: f32| angle_diff(a, b).abs() < 1e-3;
    for facing in [PI, 0.0, FRAC_PI_2, -FRAC_PI_2, PI - 0.5, 0.7] {
        let r = plan_turn(true, facing);
        assert_eq!(r.quarters, -1, "{facing}");
        assert!(close(r.facing, facing - FRAC_PI_2), "{facing}: {r:?}");
        let l = plan_turn(false, facing);
        assert_eq!(l.quarters, 1, "{facing}");
        assert!(close(l.facing, facing + FRAC_PI_2), "{facing}: {l:?}");
    }
    // 북(π)을 보고 오른쪽 → 동쪽을 보고, 화면이 오른쪽으로 90°라 동쪽이 화면 위.
    assert!(close(plan_turn(true, PI).facing, FRAC_PI_2));
    assert!(crate::camera::quarter_forward(-90.0).dot(vec3(1.0, 0.0, 0.0)) > 0.999);
}

#[test]
fn turning_turns_the_body_to_the_new_front() {
    let t = Tuning::default();
    let mut w = World::new("test", 20.0);
    w.view_yaw = Some(0.0);
    w.player.facing = 0.0; // 화면 아래(카메라 쪽)를 본다
    let ev = w.step(
        &Actions {
            turn_right: true,
            ..Default::default()
        },
        &t,
    );
    assert!(ev.contains(&Event::Turn { quarters: -1 }), "{ev:?}");
    for _ in 0..((t[P::TurnPlant] / TICK_DT).ceil() as usize + 2) {
        w.step(&Actions::default(), &t);
    }
    // 캐릭터의 오른쪽이던 서쪽(-x)을 본다.
    assert!(
        w.player.facing_dir().dot(vec3(-1.0, 0.0, 0.0)) > 0.999,
        "{}",
        w.player.facing
    );
}

/// 화면 시계 방향(요 35°, 화면 위 = `quarter_forward(35)`): 돌다가 **멈추는 순간** 화면 위에 선 적이 돌던 쪽(화면 오른쪽)으로
/// 쏟아진다(급정거 — 관성으로 계속 간다). 도는 동안엔 그대로.
#[test]
fn turning_spills_enemies_along_the_turn_when_it_stops_and_farther_ones_slide_more() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    w.view_yaw = Some(35.0);
    let up = crate::camera::quarter_forward(35.0);
    let right = up.cross(Vec3::Y);
    w.spawn_mob(up * 3.0);
    w.spawn_mob(up * 7.0);
    idle(&mut w);
    let ev = w.step(&turn(true), &t);
    assert!(ev.contains(&Event::Turn { quarters: 1 }));
    idle(&mut w); // 시작 휘청(0085)이 세워 둔 적을 깨우지 않게
                  // 도는 동안(멈추기 전)엔 아무도 밀리지 않는다.
    let half = (crate::tuning::turn_land_secs_q(&t, 1) * 0.5 / TICK_DT) as usize;
    for _ in 0..half {
        w.step(&Actions::default(), &t);
    }
    assert!(!matches!(w.enemies[1].state, EState::Stagger { .. }));
    for _ in 0..600 {
        if w.spill_at.is_none() {
            break;
        }
        w.step(&Actions::default(), &t);
    }
    assert!(matches!(w.enemies[1].state, EState::Stagger { .. }));
    // 0029: 밀림은 조금(먼 적도 `spill.slide max m`까지) — 휘청(0.3초)이 끝나기 전에 잰다.
    let (near0, far0) = (w.enemies[0].pos, w.enemies[1].pos);
    for _ in 0..15 {
        w.step(&Actions::default(), &t);
    }
    let near = w.enemies[0].pos - near0;
    let far = w.enemies[1].pos - far0;
    assert!(
        near.dot(right) > 0.05 && far.dot(right) > 0.1,
        "{near:?} {far:?}"
    );
    assert!(far.length() > near.length() * 1.5, "{near:?} {far:?}");
    assert!(far.length() <= t[P::SpillMax] + 1e-3, "{far:?}");
    // 반시계면 반대쪽.
    let mut w2 = World::new("test", 30.0);
    w2.view_yaw = Some(35.0);
    w2.spawn_mob(up * 7.0);
    idle(&mut w2);
    turn_until_spill(&mut w2, false, &t);
    let at = w2.enemies[0].pos;
    for _ in 0..15 {
        w2.step(&Actions::default(), &t);
    }
    assert!((w2.enemies[0].pos - at).dot(right) < -0.1);
}

#[test]
fn turning_staggers_everyone_in_range_when_it_stops_and_cuts_their_telegraph() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    w.spawn_mob(vec3(0.0, 0.0, -1.2));
    w.spawn_mob(vec3(0.0, 0.0, t[P::SpillRange] + 6.0));
    idle(&mut w);
    w.step(&turn(true), &t);
    while w.spill_at.is_some_and(|(_, left)| left > 2.0 * TICK_DT) {
        w.step(&Actions::default(), &t);
    }
    w.enemies[0].state = EState::Tele { t: 0.4 };
    for _ in 0..600 {
        if w.spill_at.is_none() {
            break;
        }
        w.step(&Actions::default(), &t);
    }
    assert!(matches!(w.enemies[0].state, EState::Stagger { .. }));
    assert!(!matches!(w.enemies[1].state, EState::Stagger { .. }));
    // 휘청 동안은 치지 못한다(가까이 붙어 있어도).
    let mut hurt = false;
    for _ in 0..((t[P::SpillStagger] / TICK_DT) as usize - 2) {
        hurt |= w.step(&Actions::default(), &t).contains(&Event::PlayerHurt);
    }
    assert!(!hurt);
}

/// 0029: 도는 동안 적을 붙잡지 않는다 — 그대로 쫓아온다. 멈출 때 범위 안이면 짧게 휘청할 뿐.
#[test]
fn while_the_ground_turns_the_enemies_keep_coming() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    w.spawn_mob(vec3(0.0, 0.0, -5.0));
    w.step(&turn(true), &t);
    while w.spill_at.is_some_and(|(_, left)| left > TICK_DT) {
        w.step(&Actions::default(), &t);
    }
    assert!(
        w.enemies[0].pos.z > -5.0 + 1.0,
        "closed in: {:?}",
        w.enemies[0].pos
    );
    for _ in 0..3 {
        w.step(&Actions::default(), &t);
    }
    assert!(matches!(w.enemies[0].state, EState::Stagger { .. }));
    // 휘청은 짧다.
    for _ in 0..((t[P::SpillStagger] / TICK_DT) as usize + 2) {
        w.step(&Actions::default(), &t);
    }
    assert!(!matches!(w.enemies[0].state, EState::Stagger { .. }));
}

#[test]
fn the_boss_is_heavy_it_barely_slides_and_staggers_briefly() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    w.spawn_boss(vec3(0.0, 0.0, -6.0));
    w.spawn_mob(vec3(0.0, 0.0, 6.0));
    idle(&mut w);
    turn_until_spill(&mut w, true, &t);
    // 멈춘 자리에서 잰다(도는 동안 둘 다 쫓아왔다). 보스 휘청(0.1초)이 끝나기 전에.
    let (b0, m0) = (w.enemies[0].pos, w.enemies[1].pos);
    for _ in 0..5 {
        w.step(&Actions::default(), &t);
    }
    let boss = (w.enemies[0].pos - b0).length();
    let mob = (w.enemies[1].pos - m0).length();
    assert!(boss * 3.0 < mob, "boss {boss} mob {mob}");
    for _ in 0..8 {
        w.step(&Actions::default(), &t);
    }
    let EState::Stagger { .. } = w.enemies[1].state else {
        panic!("mob should still be staggered: {:?}", w.enemies[1].state)
    };
    assert!(!matches!(w.enemies[0].state, EState::Stagger { .. }));
}

#[test]
fn turning_has_a_cooldown_plants_the_player_and_is_off_while_locked() {
    let t = Tuning::default();
    let mut w = world_with_boss();
    let walk = Actions {
        move_dir: vec2(0.0, 1.0),
        ..Default::default()
    };
    w.step(
        &Actions {
            turn_right: true,
            ..walk
        },
        &t,
    );
    assert!(matches!(w.player.state, PState::Turn { .. }));
    w.step(&walk, &t);
    assert_eq!(w.player.vel, Vec3::ZERO, "planted while twisting");
    let ev = w.step(&turn(true), &t);
    assert!(
        !ev.iter().any(|e| matches!(e, Event::Turn { .. })),
        "cooldown"
    );
    // 대기가 끝나도 락온 중엔 안 된다.
    w.player.turn_cd = 0.0;
    w.player.state = PState::Idle;
    w.hitstop = 0.0;
    w.step(
        &Actions {
            lock: true,
            ..Default::default()
        },
        &t,
    );
    assert!(w.lock.is_some());
    let ev = w.step(&turn(true), &t);
    assert!(!ev.iter().any(|e| matches!(e, Event::Turn { .. })));
    assert_eq!(w.turn_ready(&t), 0.0);
}

/// 기본값(0029)은 밀림이 짧아 거의 부딪히지 않는다 — 랩에서 밀림·벽 피해를 키웠을 때의 규칙을 본다.
#[test]
fn a_spilled_enemy_that_hits_a_pillar_is_slammed_once() {
    let mut t = Tuning::default();
    for (p, v) in [
        (P::SpillPerM, 0.35),
        (P::SpillMax, 3.0),
        (P::SlamDamage, 10.0),
    ] {
        let i = crate::tuning::SPECS.iter().position(|s| s.p == p).unwrap();
        t.set(i, v);
    }
    let mut w = World::new("test", 30.0);
    w.view_yaw = Some(0.0); // 화면 위 = -z, ↻이 멈추면 쏟아지는 쪽 = +x(화면 오른쪽, + 바깥)
    w.spawn_mob(vec3(0.0, 0.0, -7.0));
    w.obstacles.push(Aabb {
        min: vec3(1.0, 0.0, -9.0),
        max: vec3(3.0, 3.0, -6.0),
    });
    idle(&mut w);
    let mut slams = 0;
    let mut ev = turn_until_spill(&mut w, true, &t);
    for _ in 0..60 {
        slams += ev
            .iter()
            .filter(|e| matches!(e, Event::Slam { .. }))
            .count();
        ev = w.step(&Actions::default(), &t);
    }
    assert_eq!(slams, 1);
    assert_eq!(
        w.enemies[0].hp,
        w.enemies[0].max_hp - t[P::SlamDamage] as i32
    );
}

/// 0028: 어느 쪽으로 걷다 누르든 쥔 스틱 그대로 새 정면(돌기 전 캐릭터의 오른쪽·왼쪽)으로 직진한다 — 몸과 화면이 같이 90° 도니까.
/// (0023은 위로 걸을 때만 그랬다: 오른쪽으로 걷다 오른쪽 버튼이면 화면이 180° 돌고 쥔 스틱이 왔던 길로 되돌아갔다.)
#[test]
fn after_a_turn_the_held_stick_walks_straight_to_the_new_front() {
    let t = Tuning::default();
    let yaw0 = 35.0;
    for stick in [
        vec2(0.0, 1.0),
        vec2(1.0, 0.0),
        vec2(0.0, -1.0),
        vec2(-1.0, 0.0),
    ] {
        for right in [true, false] {
            let mut w = World::new("test", 30.0);
            w.view_yaw = Some(yaw0);
            let go = Actions {
                move_dir: stick,
                ..Default::default()
            };
            for _ in 0..30 {
                w.step(&go, &t);
            }
            let f = w.player.facing_dir();
            let want = if right {
                f.cross(Vec3::Y)
            } else {
                -f.cross(Vec3::Y)
            };
            let ev = w.step(
                &Actions {
                    turn_right: right,
                    turn_left: !right,
                    ..go
                },
                &t,
            );
            let q = if right { -1 } else { 1 };
            assert!(ev.contains(&Event::Turn { quarters: q }), "{ev:?}");
            w.view_yaw = Some(yaw0 + 90.0 * q as f32); // 앱이 카메라에서 넘기는 새 방위(입력 기준은 즉시)
            let plant = (t[P::TurnPlant] / TICK_DT).ceil() as usize;
            for _ in 0..plant + 12 {
                w.step(&go, &t);
            }
            let v = w.player.vel.normalize();
            assert!(
                v.dot(want) > 0.98 && w.player.facing_dir().dot(want) > 0.98,
                "stick {stick} right {right}: walks {v:?}, wants {want:?}"
            );
        }
    }
}

/// 0085(사장 2026-11-07 "화면돌리기의 휘청은 cc 포함"): 돌기 시작하는 순간 범위 안의 적이 짧게 휘청이고(예고가 끊긴다),
/// 범위 밖은 그대로. 보스는 무게만큼 짧게.
#[test]
fn turning_staggers_enemies_in_range_briefly_when_it_starts() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    w.spawn_mob(vec3(0.0, 0.0, -2.0));
    w.spawn_mob(vec3(0.0, 0.0, t[P::SpillRange] + 6.0));
    w.spawn_boss(vec3(2.5, 0.0, 0.0));
    for en in &mut w.enemies {
        en.state = EState::Tele { t: 0.1 };
    }
    w.step(&turn(true), &t);
    let dur = |e: &Enemy| match e.state {
        EState::Stagger { dur, .. } => Some(dur),
        _ => None,
    };
    assert_eq!(
        dur(&w.enemies[0]),
        Some(t[P::TurnStartStagger]),
        "near: staggered, telegraph cut"
    );
    assert_eq!(dur(&w.enemies[1]), None, "out of range: untouched");
    let boss = dur(&w.enemies[2]).expect("the boss too");
    assert!(boss < t[P::TurnStartStagger], "but shorter: {boss}");
    // 짧다 — 땅이 멈추기(쏟기) 전에 풀린다.
    for _ in 0..((t[P::TurnStartStagger] / TICK_DT) as usize + 2) {
        w.step(&Actions::default(), &t);
    }
    assert_eq!(dur(&w.enemies[0]), None);
    assert!(w.spill_at.is_some(), "still turning");
}
