//! 패링: 창·앞쪽 반원·헛침, 돌진 미리 받기, 드러난 적의 치명타·반격, 이어 패링, 모으기 패링, 받아 둔 패링.
use super::*;

/// 0025: 창 안에 앞에서 오는 근접 공격은 튕긴다 — 피해 없음, 적은 휘청·드러남, 플레이어는 바로 풀린다.
#[test]
fn a_parry_in_the_window_deflects_a_blow_from_the_front() {
    let t = Tuning::default();
    let mut w = mob_swinging_in_front(&t);
    let mut ev = w.step(
        &Actions {
            parry: true,
            ..Default::default()
        },
        &t,
    );
    for _ in 0..30 {
        if ev.iter().any(|e| matches!(e, Event::Parry { .. })) {
            break;
        }
        ev = w.step(&Actions::default(), &t);
    }
    assert!(
        ev.iter().any(|e| matches!(e, Event::Parry { .. })),
        "parried"
    );
    assert_eq!(w.player.hp, PLAYER_HP);
    assert!(matches!(w.enemies[0].state, EState::Stagger { .. }));
    assert!(w.enemies[0].exposed > 0.0);
    assert_eq!(w.player.state, PState::Idle, "free to riposte");
}

/// 2026-11-06: 달리다 패링하면 그 자리에서 0이 아니라 걸음을 멈출 때의 감속으로 선다(뒤로 출렁이지 않게) — 앞으로만 조금 미끄러진다.
#[test]
fn a_parry_from_a_run_brakes_like_a_stop_not_a_wall() {
    let t = Tuning::default();
    let mut w = World::new("test", 20.0);
    w.player.vel = vec3(0.0, 0.0, -5.0);
    w.player.state = PState::Parry { t: 0.0 };
    let z0 = w.player.pos.z;
    w.step(&Actions::default(), &t);
    assert!(
        w.player.vel.z < -1.0,
        "still moving the first tick: {:?}",
        w.player.vel
    );
    for _ in 0..20 {
        w.step(&Actions::default(), &t);
    }
    assert!(
        w.player.vel.length() < 1e-4,
        "stopped within a third of a second"
    );
    let slid = z0 - w.player.pos.z;
    assert!(slid > 0.1 && slid < 0.6, "slides forward a little: {slid}");
}

#[test]
fn a_parry_does_not_cover_your_back_or_a_root_wave() {
    let t = Tuning::default();
    // 등 뒤.
    let mut w = mob_swinging_in_front(&t);
    w.player.facing = 0.0; // 적을 등진다
    w.player.state = PState::Parry { t: 0.0 };
    let mut hurt = false;
    for _ in 0..30 {
        hurt |= w.step(&Actions::default(), &t).contains(&Event::PlayerHurt);
    }
    assert!(hurt, "a blow from behind lands");
    // 파동(못 받는 공격).
    let mut w = world_with_boss();
    let en = &mut w.enemies[0];
    en.pos = vec3(0.0, 0.0, -2.0);
    en.atk = Atk::Wave;
    en.state = EState::Wave { t: 0.0 };
    en.landed = false;
    w.player.facing = std::f32::consts::PI;
    let mut hurt = false;
    for _ in 0..60 {
        if !matches!(w.player.state, PState::Parry { .. } | PState::Hurt { .. }) {
            w.player.state = PState::Parry { t: 0.0 };
        }
        hurt |= w.step(&Actions::default(), &t).contains(&Event::PlayerHurt);
    }
    assert!(hurt, "the root wave cannot be parried");
}

/// 헛치면 `parry.time s`까지 묶인다(연타가 공짜가 되지 않게) — 스태미나도 쓴다.
#[test]
fn a_whiffed_parry_roots_you_and_costs_stamina() {
    let t = Tuning::default();
    let mut w = World::new("test", 20.0);
    let before = w.player.stamina;
    w.step(
        &Actions {
            parry: true,
            ..Default::default()
        },
        &t,
    );
    assert!(matches!(w.player.state, PState::Parry { .. }));
    assert!(w.player.stamina < before);
    let mv = Actions {
        move_dir: vec2(0.0, 1.0),
        ..Default::default()
    };
    let start = w.player.pos;
    for _ in 0..((t[P::ParryTime] / TICK_DT) as usize - 2) {
        w.step(&mv, &t);
    }
    assert!(
        (w.player.pos - start).length() < 1e-4,
        "rooted while recovering"
    );
    for _ in 0..6 {
        w.step(&mv, &t);
    }
    assert_eq!(w.player.state, PState::Idle);
}

/// 곧게 오는 돌진은 몸에 닿기 전(`parry.reach m`)에 받는다.
#[test]
fn a_charge_is_parried_just_before_it_lands() {
    let t = Tuning::default();
    let mut w = World::new("test", 20.0);
    w.spawn(Kind::Thorn, vec3(0.0, 0.0, -3.0));
    let en = &mut w.enemies[0];
    en.atk = Atk::Charge;
    en.state = EState::Charge {
        t: 0.0,
        dir: vec3(0.0, 0.0, 1.0),
    };
    en.facing = 0.0;
    w.player.facing = std::f32::consts::PI;
    let mut parried = false;
    for _ in 0..40 {
        let near =
            (w.enemies[0].pos - w.player.pos).length() < 0.45 + PLAYER_RADIUS + t[P::ParryReach];
        let a = Actions {
            parry: near && !matches!(w.player.state, PState::Parry { .. }),
            ..Default::default()
        };
        let ev = w.step(&a, &t);
        if ev.iter().any(|e| matches!(e, Event::Parry { .. })) {
            parried = true;
            break;
        }
    }
    assert!(parried);
    assert_eq!(w.player.hp, PLAYER_HP);
}

/// 드러난 적에게는 한 대가 치명타(+`parry.crit`), 그 뒤엔 보통.
#[test]
fn an_exposed_enemy_takes_a_critical_hit_once() {
    let t = Tuning::default();
    let mut w = World::new("test", 20.0);
    w.spawn_boss(vec3(0.0, 0.0, -1.5));
    idle(&mut w);
    w.enemies[0].exposed = 1.0;
    let hp = w.enemies[0].hp;
    let mut ev = w.step(
        &Actions {
            attack: true,
            ..Default::default()
        },
        &t,
    );
    for _ in 0..30 {
        ev.extend(w.step(&Actions::default(), &t));
    }
    assert!(ev.iter().any(|e| matches!(e, Event::Crit { .. })));
    assert_eq!(w.enemies[0].hp, hp - 10 - t[P::ParryCrit] as i32);
    assert_eq!(
        t[P::ParryCrit] as i32,
        20,
        "the crit bonus is two old points"
    );
    assert_eq!(w.enemies[0].exposed, 0.0);
}

/// 2026-11-06: 패링으로 튕긴 직후의 공격은 모으기 없이 다 모은 강타(반격). 창이 지나면 평타.
#[test]
fn attacking_right_after_a_parry_is_a_full_heavy_riposte() {
    let t = Tuning::default();
    let mut w = mob_swinging_in_front(&t);
    w.player.state = PState::Parry { t: 0.0 };
    let mut parried = false;
    for _ in 0..60 {
        parried |= w
            .step(&Actions::default(), &t)
            .iter()
            .any(|e| matches!(e, Event::Parry { .. }));
        if parried {
            break;
        }
    }
    assert!(parried);
    // 세상이 한 박자 멈춘 동안(히트스톱)은 틱이 돌지 않는다 — 풀리고 누른다.
    while w.hitstop > 0.0 {
        w.step(&Actions::default(), &t);
    }
    w.step(
        &Actions {
            attack: true,
            ..Default::default()
        },
        &t,
    );
    assert!(
        matches!(w.player.state, PState::Heavy { full: true, .. }),
        "riposte: {:?}",
        w.player.state
    );
    // 창이 지난 뒤면 평타.
    let mut w = World::new("test", 20.0);
    w.player.since_parry = RIPOSTE_WINDOW + 0.1;
    w.step(
        &Actions {
            attack: true,
            ..Default::default()
        },
        &t,
    );
    assert!(matches!(w.player.state, PState::Attack { .. }));
}

/// 0110: 공격 후딜레이에 눌러 받아 둔 패링이 나가기 전에 맞으면 버린다 — 경직이 풀린 뒤 뒤늦게 패링(과 헛침 굳음)이 나가지 않는다.
#[test]
fn a_hit_before_the_parry_comes_out_drops_the_buffered_parry() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    // 평타 하나를 내고 곧바로 패링을 누른다 → 캔슬 시점까지 받아 둔다.
    w.step(
        &Actions {
            attack: true,
            ..Default::default()
        },
        &t,
    );
    assert!(matches!(w.player.state, PState::Attack { .. }));
    w.step(
        &Actions {
            parry: true,
            ..Default::default()
        },
        &t,
    );
    assert_eq!(
        w.player.pending,
        Some(Next::Parry),
        "buffered behind the swing"
    );
    hit_by_swing(&mut w, &t);
    assert!(matches!(w.player.state, PState::Hurt { .. }));
    assert_eq!(w.player.pending, None, "the hit drops the parry");
    for _ in 0..90 {
        w.step(&Actions::default(), &t);
        assert!(
            !matches!(w.player.state, PState::Parry { .. }),
            "no late parry after the stun"
        );
    }
    assert_eq!(w.player.state, PState::Idle);
}

/// 0110: 경직 중에 누른 패링은 풀린 뒤 나가지 않는다. 회피는 받아 둔다(경직에서 빠져나가기, 0099).
#[test]
fn a_parry_pressed_during_the_stun_is_not_kept_but_a_dodge_is() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    hit_by_swing(&mut w, &t);
    w.step(
        &Actions {
            parry: true,
            ..Default::default()
        },
        &t,
    );
    assert_eq!(w.player.pending, None);
    for _ in 0..90 {
        w.step(&Actions::default(), &t);
        assert!(!matches!(w.player.state, PState::Parry { .. }));
    }
    // 회피: 경직이 짧게 남았을 때 누르면 풀리자마자 나간다.
    hit_by_swing(&mut w, &t);
    while let PState::Hurt { t: e, dur } = w.player.state {
        if dur - e < t[P::AttackBuffer] * 0.5 {
            break;
        }
        w.step(&Actions::default(), &t);
    }
    w.step(
        &Actions {
            dodge: true,
            move_dir: vec2(1.0, 0.0),
            ..Default::default()
        },
        &t,
    );
    let mut dodged = false;
    for _ in 0..30 {
        w.step(&Actions::default(), &t);
        dodged |= matches!(w.player.state, PState::Dodge { .. });
    }
    assert!(dodged, "a dodge pressed late in the stun still comes out");
}

/// 0081(사장 2026-11-07): 튕기면 패링 창이 `PARRY_CHAIN`만큼 이어져, 비슷한 때 오는 앞쪽 공격도 같이 튕긴다.
/// 패링 동작의 창(0.18초)이 이미 닫힌 뒤에 오는 둘째 공격으로 본다. 이어지는 창보다 늦으면 맞는다.
#[test]
fn a_parry_extends_its_window_to_deflect_blows_close_behind() {
    let t = Tuning::default();
    let first = 9; // 패링을 누르고 9틱(0.15초) 뒤 첫 공격 — 창(0.18초) 끝 무렵
    for (gap, deflected) in [(9_usize, true), (18, false)] {
        let mut w = World::new("test", 20.0);
        w.player.facing = std::f32::consts::PI; // -z = 적 쪽
        for (x, at) in [(-0.5_f32, first), (0.5, first + gap)] {
            w.spawn_mob(vec3(x, 0.0, -1.2));
            let en = w.enemies.last_mut().unwrap();
            en.facing = 0.0;
            en.atk = Atk::Smash;
            // `at`틱 뒤에 내리친다(예고 끝 → 다음 틱 휘두름의 첫 틱에 맞음).
            en.state = EState::Tele {
                t: en.tele_dur() - (at as f32 - 1.0) * TICK_DT,
            };
        }
        let mut parries = 0;
        let mut ev = w.step(
            &Actions {
                parry: true,
                ..Default::default()
            },
            &t,
        );
        for _ in 0..120 {
            parries += ev
                .iter()
                .filter(|e| matches!(e, Event::Parry { .. }))
                .count();
            ev = w.step(&Actions::default(), &t);
        }
        parries += ev
            .iter()
            .filter(|e| matches!(e, Event::Parry { .. }))
            .count();
        if deflected {
            assert_eq!(parries, 2, "gap {gap}: both deflected");
            assert_eq!(w.player.hp, PLAYER_HP);
        } else {
            assert_eq!(parries, 1, "gap {gap}: only the first");
            assert!(w.player.hp < PLAYER_HP, "the late blow lands");
        }
    }
}

/// 0089(사장 "모으기 공격에는 패링도 달려있어야 함. 리스크가 있는 공격이라 보상"): 모은 강타를 떼는 순간 앞에서 오는 공격은 튕기고
/// 강타는 끊기지 않는다. 떼지 않고 서 있으면 같은 공격에 맞는다.
#[test]
fn releasing_a_charged_heavy_deflects_a_blow_from_the_front() {
    let t = Tuning::default();
    for release in [true, false] {
        let mut w = World::new("test", 20.0);
        w.set_weapon(weapons::Id::Sword);
        w.player.facing = std::f32::consts::PI;
        w.spawn_mob(vec3(0.0, 0.0, -1.2));
        let en = w.enemies.last_mut().unwrap();
        en.facing = 0.0;
        en.atk = Atk::Smash;
        // 두 틱 뒤 내리친다 — 강타의 활성(선딜 0.06초) 전이라 강타가 먼저 끊지 않는다.
        en.state = EState::Tele {
            t: en.tele_dur() - TICK_DT,
        };
        w.player.state = if release {
            PState::Charging { t: 2.0 }
        } else {
            PState::Idle
        };
        let mut parried = false;
        for _ in 0..12 {
            let ev = w.step(&Actions::default(), &t);
            parried |= ev.iter().any(|e| matches!(e, Event::Parry { .. }));
            if parried {
                assert!(
                    matches!(w.player.state, PState::Heavy { .. }),
                    "the heavy keeps going"
                );
            }
        }
        assert_eq!(parried, release, "release {release}");
        assert_eq!(w.player.hp == PLAYER_HP, release, "release {release}");
    }
}

/// 0089: 모으기 공격의 패링 창은 뗀 뒤 `parry.window s` 동안만, 근접 강타·쇠뇌 모은 사격 모두. 반격(모으지 않은 강타)·근접 무기의 모은 마법엔 없다.
#[test]
fn the_charged_release_guard_lasts_one_parry_window_and_only_for_charged_attacks() {
    let t = Tuning::default();
    let open_ticks = |w: &mut World| {
        let mut n = 0;
        for _ in 0..40 {
            w.step(&Actions::default(), &t);
            n += w.parry_open(&t) as usize;
        }
        n
    };
    let want = (t[P::ParryWindow] / TICK_DT).round() as usize;
    let mut w = World::new("test", 20.0);
    w.set_weapon(weapons::Id::Hammer);
    w.player.state = PState::Charging { t: 2.0 };
    let n = open_ticks(&mut w);
    assert!(
        n + 1 >= want && n <= want,
        "heavy guard {n} ticks, want ≈{want}"
    );
    // 쇠뇌: 모은 사격을 떼면.
    let mut w = World::new("test", 20.0);
    w.set_weapon(weapons::Id::Crossbow);
    w.player.state = PState::CastCharge { t: 2.0 };
    let n = open_ticks(&mut w);
    assert!(n + 1 >= want && n <= want, "shot guard {n}");
    // 검의 RB 마법을 모아 뗀 것 — 무기의 공격이 아니라 없다.
    let mut w = World::new("test", 20.0);
    w.set_weapon(weapons::Id::Sword);
    w.player.state = PState::CastCharge { t: 2.0 };
    assert_eq!(open_ticks(&mut w), 0, "charged magic has no guard");
    // 반격(튕긴 뒤 바로 강타 — 모으지 않았다)도 없다.
    let mut w = World::new("test", 20.0);
    w.set_weapon(weapons::Id::Sword);
    w.player.since_parry = 0.0;
    w.step(
        &Actions {
            attack: true,
            ..Default::default()
        },
        &t,
    );
    assert!(matches!(w.player.state, PState::Heavy { full: true, .. }));
    assert_eq!(open_ticks(&mut w), 0, "a riposte is not a charged attack");
}

/// 0088(사장 2026-11-07 "패링 성공시 어느정도 기력 회복이 되어야 반격 바로 할수 있을듯"): 기력 마지막 한 줌으로 튕겨도
/// 튕긴 순간 `parry.stamina gain`이 차서 반격(다 모은 강타)이 바로 나간다. 0으로 두면 반격이 막힌다.
#[test]
fn a_parry_on_the_last_breath_still_leaves_stamina_for_the_riposte() {
    for (gain, riposte) in [(None, true), (Some(0.0), false)] {
        let mut t = Tuning::default();
        if let Some(g) = gain {
            t.bend(P::ParryStaminaGain, g);
        }
        let mut w = World::new("test", 20.0);
        w.player.facing = std::f32::consts::PI;
        w.spawn_mob(vec3(0.0, 0.0, -1.2));
        let en = w.enemies.last_mut().unwrap();
        en.facing = 0.0;
        en.atk = Atk::Smash;
        en.state = EState::Tele {
            t: en.tele_dur() - 4.0 * TICK_DT,
        };
        w.player.stamina = 1.0;
        let mut ev = w.step(
            &Actions {
                parry: true,
                ..Default::default()
            },
            &t,
        );
        assert_eq!(w.player.stamina, 0.0, "the parry spent the last of it");
        let mut parried = false;
        for _ in 0..30 {
            if ev.iter().any(|e| matches!(e, Event::Parry { .. })) {
                parried = true;
                break;
            }
            ev = w.step(&Actions::default(), &t);
        }
        assert!(parried, "gain {gain:?}");
        assert!((w.player.stamina - t[P::ParryStaminaGain]).abs() < 1e-3);
        // 튕긴 멈춤(히트스톱) 동안 누른 것도 받아 둔다 — 몇 틱 안에 반격이 나간다.
        let mut ev = w.step(
            &Actions {
                attack: true,
                ..Default::default()
            },
            &t,
        );
        let mut heavy = false;
        let mut denied = false;
        for _ in 0..12 {
            denied |= ev.contains(&Event::StaminaDenied);
            heavy |= matches!(w.player.state, PState::Heavy { full: true, .. });
            ev = w.step(&Actions::default(), &t);
        }
        assert_eq!(heavy, riposte, "gain {gain:?}: riposte {heavy}");
        assert_eq!(denied, !riposte, "a blocked riposte says so");
    }
}

/// 0086(사장 2026-11-07): 이어 패링이 쌓이면 남은 창이 반씩 줄어드는 몫만큼 늘고(0.2 → +0.1 → +0.05), 받는 반원은 넓어진다
/// (앞 ±78° → ±123° → 둘레 전부).
#[test]
fn parry_chains_stack_halving_time_and_widen_their_arc() {
    let (l0, n0) = parry_chain_add(0.0, 0);
    assert_eq!((l0, n0), (PARRY_CHAIN, 0), "first parry opens the chain");
    let (l1, n1) = parry_chain_add(0.05, n0);
    assert!((l1 - (0.05 + PARRY_CHAIN * 0.5)).abs() < 1e-6 && n1 == 1);
    let (l2, n2) = parry_chain_add(l1, n1);
    assert!((l2 - (l1 + PARRY_CHAIN * 0.25)).abs() < 1e-6 && n2 == 2);
    assert_eq!(
        parry_front_cos(false, 3),
        PARRY_FRONT,
        "no chain: the front half"
    );
    assert_eq!(parry_front_cos(true, 0), PARRY_FRONT);
    let side = parry_front_cos(true, 1);
    assert!(side < 0.0 && side > -0.6, "±123°: the sides too ({side})");
    assert!(parry_front_cos(true, 3) < -1.0, "all around");
}

/// 0086: 앞에서 둘을 함께 튕기면(이음 +1) 0.1초 뒤 옆(90°)에서 오는 것도 튕긴다 — 하나만 튕겼으면 옆은 맞는다.
#[test]
fn a_stacked_parry_chain_catches_a_blow_from_the_side() {
    let t = Tuning::default();
    for (front_mobs, side_deflected) in [(2usize, true), (1, false)] {
        let mut w = World::new("test", 20.0);
        w.player.facing = std::f32::consts::PI; // -z = 앞
        let mut put = |w: &mut World, at: Vec3, face: f32, ticks: usize| {
            w.spawn_mob(at);
            let en = w.enemies.last_mut().unwrap();
            en.facing = face;
            en.atk = Atk::Smash;
            en.state = EState::Tele {
                t: en.tele_dur() - (ticks as f32 - 1.0) * TICK_DT,
            };
        };
        // 패링을 누르면 가장 가까운 적 쪽으로 돌아선다 — 앞 오른쪽(0.3, -1.0)이 가장 가깝게.
        let fronts = [vec3(0.3, 0.0, -1.0), vec3(-0.35, 0.0, -1.05)];
        for at in fronts.iter().take(front_mobs) {
            put(&mut w, *at, 0.0, 3);
        }
        // 왼쪽 옆(바라보는 쪽에서 ≈ 107° — 평소 반원 ±78° 밖, 한 번 쌓인 ±123° 안)에서 12틱 뒤:
        // 패링 동작 창(0.18초 = 11틱)이 닫힌 뒤라 이어 패링으로만 받는다.
        put(
            &mut w,
            vec3(-1.2, 0.0, 0.0),
            std::f32::consts::FRAC_PI_2,
            3 + 12,
        );
        let mut parries = 0;
        let mut ev = w.step(
            &Actions {
                parry: true,
                ..Default::default()
            },
            &t,
        );
        for _ in 0..90 {
            parries += ev
                .iter()
                .filter(|e| matches!(e, Event::Parry { .. }))
                .count();
            ev = w.step(&Actions::default(), &t);
        }
        parries += ev
            .iter()
            .filter(|e| matches!(e, Event::Parry { .. }))
            .count();
        let side_hit = w.player.hp < PLAYER_HP;
        assert_eq!(
            !side_hit, side_deflected,
            "{front_mobs} in front: parries {parries}, hp {}",
            w.player.hp
        );
    }
}
