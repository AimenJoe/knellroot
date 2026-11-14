//! 회피·달리기·기력: 무적과 지나가기, 회피에서 잇기, 쥐고 달리기, 기력이 막는 것과 알림, 무기 무게의 값.
use super::*;

#[test]
fn dodging_player_is_not_hurt_by_a_boss_swing() {
    let t = Tuning::default();
    let mut w = world_with_boss();
    w.enemies[0].pos = vec3(0.0, 0.0, -2.0);
    w.enemies[0].facing = 0.0; // +z = 플레이어 쪽
    w.enemies[0].state = EState::Swing { t: 0.0 };
    w.player.state = PState::Dodge {
        t: 0.0,
        dir: vec3(1.0, 0.0, 0.0),
    };
    let ev = w.step(&Actions::default(), &t);
    assert!(!ev.contains(&Event::PlayerHurt));
    assert_eq!(w.player.hp, PLAYER_HP);
}

#[test]
fn a_dodge_passes_through_enemies_but_walking_does_not() {
    let t = Tuning::default();
    let mut w = World::new("test", 20.0);
    w.view_yaw = Some(0.0);
    w.spawn_mob(vec3(0.0, 0.0, -1.5));
    idle(&mut w);
    let a = Actions {
        move_dir: vec2(0.0, 1.0),
        dodge: true,
        ..Default::default()
    };
    w.step(&a, &t);
    for _ in 0..30 {
        w.step(
            &Actions {
                move_dir: vec2(0.0, 1.0),
                ..Default::default()
            },
            &t,
        );
        idle(&mut w);
    }
    assert!(
        w.player.pos.z < -1.5 - 0.3,
        "dodged through: z {}",
        w.player.pos.z
    );
    // 걸어서는 못 지나간다.
    let mut w = World::new("test", 20.0);
    w.view_yaw = Some(0.0);
    w.spawn_mob(vec3(0.0, 0.0, -1.5));
    for _ in 0..60 {
        w.step(
            &Actions {
                move_dir: vec2(0.0, 1.0),
                ..Default::default()
            },
            &t,
        );
        idle(&mut w);
    }
    assert!(
        w.player.pos.z > -1.5,
        "walking is blocked: z {}",
        w.player.pos.z
    );
}

/// 0117: 평타 기력은 무기 배율만큼 — 망치는 검보다 많이, 맨손은 적게.
#[test]
fn heavier_weapons_spend_more_stamina_per_swing() {
    let t = Tuning::default();
    let spent = |id: weapons::Id| {
        let mut w = World::new("test", 20.0);
        w.set_weapon(id);
        let s0 = w.player.stamina;
        w.step(
            &Actions {
                attack: true,
                ..Default::default()
            },
            &t,
        );
        s0 - w.player.stamina
    };
    let sword = spent(weapons::Id::Sword);
    assert!((sword - t[P::StaminaAttack]).abs() < 0.5, "{sword}");
    assert!((spent(weapons::Id::Hammer) - sword * 1.7).abs() < 0.5);
    assert!(spent(weapons::Id::Fists) < sword);
}

/// 0057: 회피의 뒷부분(`dodge.cancel x` 뒤)은 공격·점프로 끊긴다 — 회피 초반에 누른 것은 받아 뒀다가 그때.
#[test]
fn a_dodge_flows_into_an_attack_or_a_jump() {
    let t = Tuning::default();
    let cancel_at = (t[P::DodgeTime] * t[P::DodgeCancel] / TICK_DT).ceil() as usize;
    let checks: [Check; 2] = [
        (
            Actions {
                attack: true,
                ..Default::default()
            },
            |s| matches!(s, PState::Attack { .. }),
        ),
        (
            Actions {
                jump: true,
                ..Default::default()
            },
            |s| matches!(s, PState::Jump { .. }),
        ),
    ];
    for (press, want) in checks {
        let mut w = World::new("test", 30.0);
        w.step(
            &Actions {
                dodge: true,
                ..Default::default()
            },
            &t,
        );
        assert!(matches!(w.player.state, PState::Dodge { .. }));
        let got = ticks_until(&mut w, &t, press, want, 60).expect("flows");
        assert!(
            got + 1 >= cancel_at.saturating_sub(1) && got <= cancel_at + 1,
            "{press:?}: {got} vs {cancel_at}"
        );
    }
}

/// 회피·공격은 스태미나를 쓰고, 없으면 막히며(막대가 깜빡인다) 마지막 사용 뒤 잠깐 쉬면 다시 찬다(0018).
#[test]
fn stamina_gates_dodges_and_comes_back_after_a_pause() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    let full = w.player.stamina;
    let dodge = Actions {
        dodge: true,
        ..Default::default()
    };
    let mut dodges = 0;
    for _ in 0..600 {
        if w.player.state == PState::Idle && w.player.dodge_cd <= 0.0 {
            let before = w.player.stamina;
            w.step(&dodge, &t);
            if matches!(w.player.state, PState::Dodge { .. }) {
                dodges += 1;
                assert!(w.player.stamina < before);
            } else {
                break;
            }
        } else {
            w.step(&Actions::default(), &t);
        }
        if w.player.stamina <= 0.0 {
            break;
        }
    }
    let max_dodges = (full / t[P::StaminaDodge]).ceil() as i32;
    assert!(dodges <= max_dodges && dodges >= 2, "{dodges}");
    // 다 쓰면 막히고 막대가 깜빡인다.
    while !(w.player.state == PState::Idle && w.player.dodge_cd <= 0.0) {
        w.step(&Actions::default(), &t);
    }
    if w.player.stamina <= 0.0 {
        w.step(&dodge, &t);
        assert!(!matches!(w.player.state, PState::Dodge { .. }));
        assert!(w.player.stamina_denied > 0.0);
    }
    // 쉬면 찬다.
    for _ in 0..((t[P::StaminaDelay] + full / t[P::StaminaRegen]) / TICK_DT) as usize + 10 {
        w.step(&Actions::default(), &t);
    }
    assert!((w.player.stamina - t[P::StaminaMax]).abs() < 1e-3);
}

/// 0076(사장 2026-11-07 "회피 꾹 눌렀을때 달리는거"): 톡 누름은 회피만, 회피가 끝날 때까지 쥐고 있으면 달린다 —
/// 걷기보다 빠르고 스태미나를 쓰며, 떼면 걷는다. 달리다 치면 대시 공격.
#[test]
fn holding_dodge_runs_after_the_dodge_and_a_tap_does_not() {
    let t = Tuning::default();
    let up = vec2(0.0, 1.0);
    let run = |held_ticks: usize| -> World {
        let mut w = World::new("test", 40.0);
        for i in 0..90 {
            w.step(
                &Actions {
                    dodge: i == 0,
                    dodge_held: i < held_ticks,
                    move_dir: up,
                    ..Default::default()
                },
                &t,
            );
        }
        w
    };
    // 톡(0.1초): 회피(0.3초 남짓)보다 짧다 — 예전과 같이 걷는다.
    let tap = run(6);
    assert!(!tap.player.running);
    let walk = tap.player.vel.length();
    assert!((walk - t[P::MoveSpeed]).abs() < 0.3, "walks {walk}");
    // 쥠: 회피 뒤 달린다.
    let held = run(90);
    assert!(held.player.running, "runs while held");
    let speed = held.player.vel.length();
    assert!(speed > walk * 1.3, "faster: {speed} vs {walk}");
    assert!(
        held.player.stamina < tap.player.stamina,
        "running costs stamina"
    );
    // 떼면 다음 틱에 걷는다.
    let mut w = held;
    w.step(
        &Actions {
            move_dir: up,
            ..Default::default()
        },
        &t,
    );
    assert!(!w.player.running);
    // 달리다 치면 대시 공격.
    let mut w = run(90);
    w.step(
        &Actions {
            attack: true,
            dodge_held: true,
            move_dir: up,
            ..Default::default()
        },
        &t,
    );
    assert!(
        matches!(
            w.player.state,
            PState::Attack {
                hit: weapons::DASH,
                ..
            }
        ),
        "{:?}",
        w.player.state
    );
    assert!(!w.player.running);
}

/// 0076: 스태미나가 다하면 달리기가 끝난다(쥐고 있어도).
#[test]
fn running_stops_when_stamina_runs_out() {
    let t = Tuning::default();
    let mut w = World::new("test", 200.0);
    let secs = t[P::StaminaMax] / RUN_STAMINA + 1.0;
    let mut was = false;
    let mut ended_at = None;
    for i in 0..((secs / TICK_DT) as usize) {
        w.step(
            &Actions {
                dodge: i == 0,
                dodge_held: true,
                move_dir: vec2(0.0, 1.0),
                ..Default::default()
            },
            &t,
        );
        if was && !w.player.running && ended_at.is_none() {
            ended_at = Some(w.player.stamina);
        }
        was = w.player.running;
    }
    let left = ended_at.expect("the run ended while still held");
    assert!(left <= 1e-3, "it ended because stamina ran out: {left}");
}

/// 0088(사장 "기력이 없어 행동하지 못할때의 피드백이 부족함"): 막히면 `Event::StaminaDenied`(소리)·막대 알림이 한 번 —
/// 받아 둔 누름이 틱마다 다시 막혀도 다시 알리지 않고, 알림이 끝난 뒤 새로 누르면 다시.
#[test]
fn running_out_of_stamina_says_so_once_per_press() {
    let t = Tuning::default();
    let mut w = World::new("test", 20.0);
    w.player.stamina = 0.0;
    w.player.stamina_idle = 0.0;
    let press = Actions {
        attack: true,
        ..Default::default()
    };
    let count = |ev: &[Event]| ev.iter().filter(|e| **e == Event::StaminaDenied).count();
    let mut n = count(&w.step(&press, &t));
    assert_eq!(n, 1, "the blocked press is announced");
    assert!(w.player.stamina_denied > 0.0 && matches!(w.player.state, PState::Idle));
    for _ in 0..((STAMINA_DENIED_SHOW / TICK_DT) as usize - 2) {
        n += count(&w.step(&Actions::default(), &t));
    }
    assert_eq!(n, 1, "the buffered retry stays quiet");
    for _ in 0..4 {
        w.step(&Actions::default(), &t);
    }
    w.player.stamina = 0.0;
    assert_eq!(
        count(&w.step(&press, &t)),
        1,
        "a new press later is announced again"
    );
}
