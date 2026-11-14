//! 평타: 한 스윙 한 번·히트스톱, 콤보·캔슬·받아 두기, 박자 타와 박자 부채꼴, 대시 공격, 자동 조준·내딛기, 자세 막대.
use super::*;

#[test]
fn a_swing_hits_once_per_enemy_and_triggers_hitstop() {
    let t = Tuning::default();
    let mut w = World::new("test", 20.0);
    w.spawn_mob(vec3(0.0, 0.0, -1.0));
    let mut hits = 0;
    let mut a = Actions {
        attack: true,
        ..Default::default()
    };
    for _ in 0..60 {
        let ev = w.step(&a, &t);
        a.attack = false;
        hits += ev.iter().filter(|e| matches!(e, Event::Hit { .. })).count();
    }
    assert_eq!(hits, 1);
    assert_eq!(w.enemies[0].hp, w.enemies[0].max_hp - 10);
}

/// 0030: 톡 누름(사람 0.08~0.15초)은 평타 하나 — 모으기로 들어가지 않는다(평타는 누르는 순간, KB melee 노트).
#[test]
fn a_tap_is_a_light_attack_and_never_charges() {
    let t = Tuning::default();
    for ticks in 1..=9 {
        let mut w = World::new("test", 20.0);
        let (st, _) = hold_attack(&mut w, &t, ticks, 60);
        assert!(
            matches!(st[0], PState::Attack { .. }),
            "pressed {ticks}: {:?}",
            st[0]
        );
        assert!(
            !st.iter()
                .any(|s| matches!(s, PState::Charging { .. } | PState::Heavy { .. })),
            "held {ticks} ticks"
        );
    }
}

/// 0051: 연타하면 검은 1 → 2 → 3타로 이어지고, 마지막 타 뒤에는 처음으로 돌아간다.
#[test]
fn mashing_the_sword_runs_its_three_hit_combo() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    mob_with(&mut w, vec3(0.0, 0.0, -1.3), 990);
    idle(&mut w);
    let mut seen = Vec::new();
    for i in 0..240 {
        let a = Actions {
            attack: i % 6 == 0,
            ..Default::default()
        };
        w.step(&a, &t);
        idle(&mut w);
        w.player.stamina = 100.0;
        if let PState::Attack { hit, .. } = w.player.state {
            if seen.last() != Some(&hit) {
                seen.push(hit);
            }
        }
    }
    assert!(seen.windows(3).any(|x| x == [0, 1, 2]), "{seen:?}");
    assert!(seen.windows(2).any(|x| x == [2, 0]), "wraps: {seen:?}");
}

/// 0057: 평타의 후딜레이는 다음 행동(회피·패링·점프·마법)으로 끊긴다 — 활성 구간이 끝나면 바로, 그 전에 누르면 받아 뒀다가 그때.
#[test]
fn attack_recovery_cancels_into_any_next_action() {
    let t = Tuning::default();
    let w0 = weapons::Id::Sword.def();
    let (s0, a0, r0) = hit_times(&t, w0, 0);
    let active_end = ((s0 + a0) / TICK_DT).ceil() as usize;
    let checks: [Check; 4] = [
        (
            Actions {
                dodge: true,
                ..Default::default()
            },
            |s| matches!(s, PState::Dodge { .. }),
        ),
        (
            Actions {
                parry: true,
                ..Default::default()
            },
            |s| matches!(s, PState::Parry { .. }),
        ),
        (
            Actions {
                jump: true,
                ..Default::default()
            },
            |s| matches!(s, PState::Jump { .. }),
        ),
        (
            Actions {
                cast: true,
                ..Default::default()
            },
            |s| matches!(s, PState::Cast { .. }),
        ),
    ];
    for (press, want) in checks {
        let mut w = World::new("test", 30.0);
        w.step(
            &Actions {
                attack: true,
                ..Default::default()
            },
            &t,
        );
        // 활성 중에 누른다 → 활성이 끝나는 틱 즈음에 나간다(후딜레이 전부를 기다리지 않는다).
        let got = ticks_until(&mut w, &t, press, want, 60).expect("cancelled");
        assert!(got <= active_end + 1, "{press:?}: {got} ticks");
        assert!(
            ((got as f32) * TICK_DT) < s0 + a0 + r0 - TICK_DT,
            "before the recovery ends"
        );
    }
}

/// 0057: 캔슬 시점은 무기 성격 — 망치는 후딜레이의 앞부분이 지나야 끊긴다(검보다 늦게).
#[test]
fn a_hammer_commits_longer_than_a_sword() {
    let t = Tuning::default();
    let when = |id: weapons::Id| {
        let mut w = World::new("test", 30.0);
        w.set_weapon(id);
        w.step(
            &Actions {
                attack: true,
                ..Default::default()
            },
            &t,
        );
        // 회피를 연타한다(너무 일찍 한 번 누른 것은 `attack.buffer s`가 지나면 사라진다 — 무거운 무기의 대가).
        (1..120)
            .find(|_| {
                w.step(
                    &Actions {
                        dodge: true,
                        ..Default::default()
                    },
                    &t,
                );
                matches!(w.player.state, PState::Dodge { .. })
            })
            .expect("cancels at some point")
    };
    let (sword, hammer) = (when(weapons::Id::Sword), when(weapons::Id::Hammer));
    let h = weapons::Id::Hammer.def();
    let (s0, a0, r0) = hit_times(&t, h, 0);
    let want = ((s0 + a0 + r0 * h.combo[0].cancel) / TICK_DT).ceil() as usize;
    assert!(hammer > sword, "hammer {hammer} vs sword {sword}");
    assert!(hammer + 2 >= want, "{hammer} vs {want}");
}

/// 0057: 한 번 누른 회피는 한 번만 — 받아 둔 누름이 쓰이면 지운다.
#[test]
fn one_press_is_one_action() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    let mut dodges = 0;
    let mut was = false;
    for i in 0..90 {
        w.step(
            &Actions {
                dodge: i == 0,
                ..Default::default()
            },
            &t,
        );
        let now = matches!(w.player.state, PState::Dodge { .. });
        if now && !was {
            dodges += 1;
        }
        was = now;
    }
    assert_eq!(dodges, 1);
}

/// 0051: 자세 막대가 다 차면 깨진다 — 휘청 + 드러남(다음 한 대 치명타), 깬 뒤 잠깐은 다시 안 찬다.
#[test]
fn filling_the_poise_bar_breaks_the_enemy() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    let i = mob_with(&mut w, vec3(0.0, 0.0, -3.0), 990);
    let max = poise_max(&w.enemies[i], &t);
    let mut ev = Vec::new();
    add_poise(&mut w.enemies[i], max * 0.5, None, &t, &mut ev);
    assert!(ev.is_empty());
    add_poise(&mut w.enemies[i], max * 0.6, None, &t, &mut ev);
    assert!(ev.iter().any(|e| matches!(e, Event::Break { .. })));
    assert!(matches!(w.enemies[i].state, EState::Stagger { .. }));
    assert!(w.enemies[i].exposed > 0.0);
    // 0055: 깨진 동안은 모든 타격이 × `poise.break damage x`(폭딜 창) — 첫 대는 치명타 덤까지.
    assert!(w.enemies[i].broken > 0.0);
    let base = 20;
    let want = crate::tuning::scale_up(base, t[P::PoiseBreakDamage]);
    assert_eq!(hurt_by(&w.enemies[i], base, false, 0, &t), want);
    assert!(hurt_by(&w.enemies[i], base, true, 0, &t) > want);
    // 올림은 옛 눈금으로(0158 N0): 옛 1 → 2·2 → 3·3 → 5(4.5 올림)가 열 배 그대로 — 15·45가 아니다.
    assert_eq!(t[P::PoiseBreakDamage], 1.5);
    for (d, hurt) in [(10, 20), (20, 30), (30, 50), (40, 60), (50, 80)] {
        assert_eq!(hurt_by(&w.enemies[i], d, false, 0, &t), hurt, "{d}");
    }
    // 0127 깨진 종 조각: 열린 틈(깨짐·드러남)에만 +10 — 깨진 동안은 배율 앞에서 더한다.
    let open = crate::tuning::scale_up(base + 10, t[P::PoiseBreakDamage]);
    assert_eq!((want, open), (30, 50));
    assert_eq!(hurt_by(&w.enemies[i], base, false, 10, &t), open);
    w.enemies[i].broken = 0.0;
    assert_eq!(hurt_by(&w.enemies[i], base, false, 0, &t), base);
    assert_eq!(
        hurt_by(&w.enemies[i], base, false, 10, &t),
        base,
        "closed: no bonus"
    );
    let crit = base + t[P::ParryCrit] as i32;
    assert_eq!(
        hurt_by(&w.enemies[i], base, true, 10, &t),
        crit + 10,
        "exposed: bonus"
    );
    let before = w.enemies[i].poise;
    add_poise(&mut w.enemies[i], max, None, &t, &mut ev);
    assert_eq!(w.enemies[i].poise, before, "locked after a break");
    // 망치처럼 "깨지기 직전"을 깨는 타는 반만 차 있어도 깬다.
    let j = mob_with(&mut w, vec3(2.0, 0.0, -3.0), 990);
    let mut ev = Vec::new();
    add_poise(&mut w.enemies[j], max * 0.7, None, &t, &mut ev);
    add_poise(&mut w.enemies[j], 0.1, Some(0.6), &t, &mut ev);
    assert!(ev.iter().any(|e| matches!(e, Event::Break { .. })));
}

#[test]
fn a_press_during_hitstop_is_kept_for_when_the_world_moves_again() {
    let t = Tuning::default();
    let mut w = world_with_boss();
    w.hitstop = 0.1;
    w.step(
        &Actions {
            lock: true,
            ..Default::default()
        },
        &t,
    );
    assert!(w.lock.is_none(), "frozen");
    let mut ev = Vec::new();
    for _ in 0..10 {
        ev.extend(w.step(&Actions::default(), &t));
    }
    assert!(ev.contains(&Event::LockOn) && w.lock.is_some());
}

/// 2026-11-06(타격감, KB melee-light…): 평타가 맞으면 6프레임쯤 멈추고(0.06초 × `HIT_HITSTOP`), 맞은 쪽으로 불똥이 튀고
/// 몸이 젖고, 공격자는 남은 내딛기를 멈춘다(빗나감과 맞힘이 손에서 다르게).
#[test]
fn a_landed_light_hit_stops_the_world_sprays_sparks_and_ends_the_lunge() {
    let t = Tuning::default();
    let mut w = World::new("test", 20.0);
    w.spawn_mob(vec3(0.0, 0.0, -1.3));
    w.player.facing = std::f32::consts::PI;
    let mut landed = false;
    for i in 0..20 {
        let ev = w.step(
            &Actions {
                attack: i == 0,
                ..Default::default()
            },
            &t,
        );
        if ev.iter().any(|e| matches!(e, Event::Hit { .. })) {
            landed = true;
            break;
        }
    }
    assert!(landed);
    assert!((w.hitstop - t[P::Hitstop] * HIT_HITSTOP).abs() < 1e-5);
    assert!(w.hitstop >= 0.09, "about six frames, not three");
    assert!(!w.sparks.is_empty(), "sparks fly");
    let away = w.sparks.iter().map(|s| s.vel.z).sum::<f32>();
    assert!(away < 0.0, "they fly the way the blow went (-z)");
    assert!(w.enemies[0].sway.length() > 0.2, "the body gives");
    while w.hitstop > 0.0 {
        w.step(&Actions::default(), &t);
    }
    w.step(&Actions::default(), &t);
    assert!(w.player.vel.length() < 1e-4, "the lunge stops on contact");
}

/// 2026-11-07: 무기마다 박자 = 그 타의 한 스윙 시간. 박자 창(± `BEAT_WINDOW`) 안에 **누른** 다음 평타는 스태미나를 쓰지 않는다.
/// 일찍 눌러 받아 둔 것·연타는 박자에 나가도 쓴다(연타가 공짜가 되지 않게), 첫 타도 쓴다.
#[test]
fn an_attack_pressed_on_the_weapons_beat_costs_no_stamina_but_mashing_does() {
    assert!(on_beat(0.43, 0.43) && on_beat(0.43 * 1.14, 0.43) && !on_beat(0.43 * 1.2, 0.43));
    assert!(!on_beat(0.2, 0.43) && !on_beat(0.43, 0.0));
    let t = Tuning::default();
    for weapon in [weapons::Id::Sword, weapons::Id::Staff, weapons::Id::Hammer] {
        // 박자에 맞춰 누름.
        let mut w = World::new("test", 20.0);
        w.set_weapon(weapon);
        w.step(
            &Actions {
                attack: true,
                ..Default::default()
            },
            &t,
        );
        let after_first = w.player.stamina;
        assert!(after_first < t[P::StaminaMax], "the first swing costs");
        let beat = w.player.beat.period;
        assert!(beat > 0.25, "{weapon:?} beat {beat}");
        let mut free = false;
        for _ in 0..120 {
            let press = w.player.beat.since_swing >= beat - TICK_DT * 0.5
                && w.player.beat.since_swing < beat + TICK_DT;
            let before = w.player.stamina;
            let ev = w.step(
                &Actions {
                    attack: press,
                    ..Default::default()
                },
                &t,
            );
            if press {
                assert!(ev.contains(&Event::Beat), "{weapon:?}: on the beat");
                assert!(
                    w.player.stamina >= before,
                    "{weapon:?}: no stamina for the beat swing"
                );
                // 피드백(0076): 이 스윙은 박자 타, 잇단 수 1, 고리·음표 시계가 0부터.
                assert!(
                    w.player.beat.swing_beat && w.player.beat.chain == 1,
                    "{weapon:?}"
                );
                assert!(w.player.beat.since < TICK_DT * 2.0);
                free = true;
                break;
            }
        }
        assert!(free, "{weapon:?}");
        // 연타(매 틱 누름): 박자에 나가는 일이 있어도 받아 둔 이른 누름이라 늘 쓴다.
        let mut w = World::new("test", 20.0);
        w.set_weapon(weapon);
        let mut beats = 0;
        for _ in 0..90 {
            let ev = w.step(
                &Actions {
                    attack: true,
                    ..Default::default()
                },
                &t,
            );
            beats += ev.iter().filter(|e| **e == Event::Beat).count();
        }
        assert_eq!(beats, 0, "{weapon:?}: mashing never gets the beat for free");
        assert!(!w.player.beat.swing_beat && w.player.beat.chain == 0);
    }
}

/// 0087: 박자 부채꼴은 나타난 뒤 줄기만 하고, 박자 순간에 가장 작으며(그 뒤 창이 닫힐 때까지 그대로), 금빛(ripe) = 박자 창 안.
#[test]
fn the_beat_fan_shrinks_to_its_smallest_on_the_beat() {
    let (start, beat) = (0.08, 0.43);
    assert_eq!(
        beat_shrink(0.05, start, beat),
        None,
        "not before it appears"
    );
    assert_eq!(beat_shrink(start, start, beat).map(|c| c.0), Some(1.0));
    let mut last = 2.0;
    let mut k = start;
    while k <= beat {
        let (left, ripe) = beat_shrink(k, start, beat).unwrap();
        assert!(left <= last, "only shrinks");
        assert_eq!(
            ripe,
            on_beat(k, beat),
            "gold exactly inside the window at {k}"
        );
        last = left;
        k += 0.005;
    }
    let (min, ripe) = beat_shrink(beat, start, beat).unwrap();
    assert!((min - BEAT_CUE_MIN).abs() < 1e-5 && ripe);
    let (held, ripe) = beat_shrink(beat + 0.05, start, beat).unwrap();
    assert!(
        (held - BEAT_CUE_MIN).abs() < 1e-5 && ripe,
        "held smallest until the window closes"
    );
    assert_eq!(
        beat_shrink(beat + 0.1, start, beat),
        None,
        "gone once the window closes"
    );
    assert_eq!(beat_shrink(0.2, 0.0, 0.0), None, "no beat, no fan");
}

/// 0087: 검 1타(오른쪽 → 왼쪽)는 활성부터 박자 부채꼴, 박자에 금빛으로 가장 작고, 그때 누르면 박자 타.
/// 이을 타가 없는 마지막 타엔 없다. 쇠뇌는 쏜 순간부터 좁은 부채꼴이 가운데로 줄어든다.
#[test]
fn the_beat_fan_follows_the_swing_and_peaks_when_a_press_counts() {
    let t = Tuning::default();
    let press = |w: &mut World, attack: bool, cast: bool| {
        w.step(
            &Actions {
                attack,
                cast,
                ..Default::default()
            },
            &t,
        )
    };
    let mut w = World::new("test", 20.0);
    w.set_weapon(weapons::Id::Sword);
    assert_eq!(w.beat_cue(&t), None);
    press(&mut w, true, false);
    let (s0, _, _) = hit_times(&t, w.player.weapon.def(), 0);
    let mut seen = None;
    let mut beat = false;
    for _ in 0..90 {
        let cue = w.beat_cue(&t);
        if w.player.beat.since_swing < s0 - 1e-4 {
            assert_eq!(cue, None, "not before the hit is live");
        }
        if let Some(c) = cue {
            assert_eq!(c.sweep, weapons::Sweep::RightToLeft);
            seen = Some(c);
            if c.ripe && (w.player.beat.since_swing - w.player.beat.period).abs() < TICK_DT * 0.6 {
                assert!((c.left - BEAT_CUE_MIN).abs() < 0.05, "smallest on the beat");
                let ev = press(&mut w, true, false);
                assert!(
                    ev.contains(&Event::Beat),
                    "a press while it is smallest is on the beat"
                );
                beat = true;
                break;
            }
        }
        press(&mut w, false, false);
    }
    assert!(seen.is_some() && beat);
    // 둘째 타(왼쪽 → 오른쪽)를 박자로 쳤다 — 이제 부채꼴은 그 타의 쪽.
    for _ in 0..12 {
        press(&mut w, false, false);
    }
    assert_eq!(
        w.beat_cue(&t).map(|c| c.sweep),
        Some(weapons::Sweep::LeftToRight)
    );
    // 셋째(마지막) 타 찌르기: 이을 타가 없으니 박자도 없다.
    let mut w = World::new("test", 20.0);
    w.set_weapon(weapons::Id::Sword);
    w.player.combo_next = 2;
    w.player.combo_left = 1.0;
    press(&mut w, true, false);
    assert!(matches!(w.player.state, PState::Attack { hit: 2, .. }));
    for _ in 0..60 {
        assert_eq!(w.beat_cue(&t), None, "the finisher has no beat");
        press(&mut w, false, false);
    }
    // 쇠뇌: 쏜 순간부터, 가운데로.
    let mut w = World::new("test", 20.0);
    w.set_weapon(weapons::Id::Crossbow);
    press(&mut w, true, false);
    let c = w.beat_cue(&t).expect("a shot starts its beat fan");
    assert_eq!(c.sweep, weapons::Sweep::Center);
    assert!(c.left > 0.9 && !c.ripe);
}

/// 2026-11-06: 회피를 끊고(또는 막 끝나고) 누른 공격은 무기의 대시 공격 — 멀리 내딛고, 다음 타는 콤보 둘째 타.
#[test]
fn attacking_out_of_a_dodge_is_a_dash_attack_that_chains_into_the_second_hit() {
    let t = Tuning::default();
    let mut w = World::new("test", 20.0);
    w.step(
        &Actions {
            dodge: true,
            move_dir: vec2(0.0, 1.0),
            ..Default::default()
        },
        &t,
    );
    assert!(matches!(w.player.state, PState::Dodge { .. }));
    let mut dashed = false;
    for i in 0..40 {
        let a = Actions {
            attack: i == 8,
            ..Default::default()
        };
        w.step(&a, &t);
        if let PState::Attack { hit, .. } = w.player.state {
            assert_eq!(
                hit,
                weapons::DASH,
                "the attack out of a dodge is the dash attack"
            );
            dashed = true;
            break;
        }
    }
    assert!(dashed);
    let wpn = w.player.weapon.def();
    assert_eq!(
        wpn.after(weapons::DASH),
        Some(1),
        "then the combo goes on from its second hit"
    );
    // 그냥 선 채로 누른 공격은 첫 타.
    let mut w = World::new("test", 20.0);
    w.step(
        &Actions {
            attack: true,
            ..Default::default()
        },
        &t,
    );
    assert!(matches!(w.player.state, PState::Attack { hit: 0, .. }));
}

fn swing_at(w: &mut World, aim: Vec2) {
    let t = Tuning::default();
    w.step(
        &Actions {
            attack: true,
            aim,
            ..Default::default()
        },
        &t,
    );
}

#[test]
fn an_attack_turns_to_the_nearest_enemy_within_reach() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    w.view_yaw = Some(0.0);
    w.spawn_mob(vec3(2.0, 0.0, 0.0)); // 가까운 적(오른쪽)
    w.spawn_mob(vec3(0.0, 0.0, -2.5)); // 조금 먼 적(앞)
    w.player.facing = std::f32::consts::PI; // -z를 본다
    swing_at(&mut w, Vec2::ZERO);
    assert!(
        w.player.facing_dir().x > 0.99,
        "{:?}",
        w.player.facing_dir()
    );
    // 범위 밖이면 돌지 않는다.
    let mut w = World::new("test", 30.0);
    w.spawn_mob(vec3(t[P::AimRange] + 2.0, 0.0, 0.0));
    w.player.facing = std::f32::consts::PI;
    swing_at(&mut w, Vec2::ZERO);
    assert!(w.player.facing_dir().z < -0.99);
}

#[test]
fn with_the_aim_stick_held_the_attack_picks_the_nearest_enemy_that_way() {
    let mut w = World::new("test", 30.0);
    w.view_yaw = Some(0.0); // 화면 위 = -z
    w.spawn_mob(vec3(1.5, 0.0, 0.0)); // 가장 가깝지만 오른쪽
    w.spawn_mob(vec3(0.0, 0.0, -2.5)); // 스틱 쪽(위)
    swing_at(&mut w, vec2(0.0, 1.0));
    assert!(
        w.player.facing_dir().z < -0.99,
        "{:?}",
        w.player.facing_dir()
    );
    // 그 쪽에 적이 없으면 스틱 쪽 그대로.
    let mut w = World::new("test", 30.0);
    w.view_yaw = Some(0.0);
    w.spawn_mob(vec3(1.5, 0.0, 0.0));
    swing_at(&mut w, vec2(-1.0, 0.0));
    assert!(
        w.player.facing_dir().x < -0.99,
        "{:?}",
        w.player.facing_dir()
    );
}

#[test]
fn the_right_stick_turns_the_player_without_moving() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    w.view_yaw = Some(0.0);
    w.player.facing = std::f32::consts::PI;
    for _ in 0..30 {
        w.step(
            &Actions {
                aim: vec2(1.0, 0.0),
                ..Default::default()
            },
            &t,
        );
    }
    assert!(w.player.facing_dir().x > 0.99);
    assert!(w.player.pos.length() < 1e-4);
    // 걸으면서 다른 쪽을 본다.
    for _ in 0..30 {
        w.step(
            &Actions {
                move_dir: vec2(0.0, 1.0),
                aim: vec2(-1.0, 0.0),
                ..Default::default()
            },
            &t,
        );
    }
    assert!(w.player.facing_dir().x < -0.99 && w.player.vel.z < -1.0);
}

/// 휘두르며 입력한 쪽으로 `attack.lunge m` 내딛는다(0018) — 휘두르는 쪽(자동 조준)과 달라도.
#[test]
fn an_attack_steps_toward_the_held_direction() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    w.view_yaw = Some(0.0); // 화면 오른쪽 = +x
    w.step(
        &Actions {
            attack: true,
            move_dir: vec2(1.0, 0.0),
            ..Default::default()
        },
        &t,
    );
    let reach = t[P::AttackStartup] + t[P::AttackActive];
    for _ in 0..((reach / TICK_DT) as usize + 5) {
        w.step(
            &Actions {
                move_dir: vec2(1.0, 0.0),
                ..Default::default()
            },
            &t,
        );
    }
    let lunge = t[P::AttackLunge];
    assert!(
        (w.player.pos.x - lunge).abs() < 0.1 && w.player.pos.z.abs() < 1e-3,
        "{:?}",
        w.player.pos
    );
}

/// 진행 방향으로 내딛는다(0018): 옆으로 걷다 치면 옆으로 — 휘두르는 쪽(가까운 적)은 앞이어도. 입력을 막 놓아도 움직이던 쪽으로.
#[test]
fn the_lunge_follows_the_travel_direction_not_the_swing_direction() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    w.view_yaw = Some(0.0); // 화면 위 = -z, 오른쪽 = +x
    w.spawn_mob(vec3(0.0, 0.0, -2.0)); // 앞에 적 → 휘두르는 쪽은 앞
    w.enemies[0].state = EState::Recover { t: -1000.0 };
    let right = Actions {
        move_dir: vec2(1.0, 0.0),
        ..Default::default()
    };
    for _ in 0..20 {
        w.step(&right, &t);
    }
    let x0 = w.player.pos.x;
    let to_enemy = (w.enemies[0].pos - w.player.pos).normalize();
    // 입력을 놓는 틱에 공격: 아직 오른쪽으로 움직이던 중.
    w.step(
        &Actions {
            attack: true,
            ..Default::default()
        },
        &t,
    );
    assert!(
        w.player.facing_dir().dot(to_enemy) > 0.99,
        "swing faces the enemy"
    );
    for _ in 0..12 {
        w.step(&Actions::default(), &t);
    }
    assert!(
        w.player.pos.x - x0 > t[P::AttackLunge] * 0.8,
        "{} -> {}",
        x0,
        w.player.pos.x
    );
    assert!(w.player.pos.z.abs() < 0.05);
}

/// 0076(사장 2026-11-07 "무기 장착하지 않았을 경우에는 맨손 전투"): 맨손도 콤보로 친다 — 가깝고 가볍다.
#[test]
fn bare_hands_still_fight() {
    let t = Tuning::default();
    let mut w = World::new("test", 20.0);
    w.set_weapon(weapons::Id::Fists);
    let i = mob_with(&mut w, vec3(0.0, 0.0, 1.0), 100);
    w.enemies[i].state = EState::Dormant;
    w.enemies[i].state = EState::Chase;
    let hp = w.enemies[i].hp;
    for k in 0..30 {
        w.step(
            &Actions {
                attack: k == 0,
                ..Default::default()
            },
            &t,
        );
    }
    assert_eq!(w.enemies[i].hp, hp - 10, "a punch lands for 10");
    let fists = weapons::Id::Fists.def();
    let sword = weapons::Id::Sword.def();
    assert!(
        fists.combo[0].reach < sword.combo[0].reach && fists.combo[0].time < sword.combo[0].time
    );
}
