//! 강공격: 모으기·떼기, 다 모은 강타의 휘청, 딱 맞춰 떼기, 망치 여진, 모으는 동안의 걸음·캔슬.
use super::*;

/// 0030: 쥐고 있으면 평타의 활성이 끝난 뒤 모으기, 떼면 늘 강타 — 다 모았으면 `full`, 다 모이는 틱에 `Charged` 한 번.
#[test]
fn holding_charges_and_letting_go_always_swings_a_heavy() {
    let t = Tuning::default();
    let into = ((t[P::AttackStartup] + t[P::AttackActive]).max(t[P::HeavyHoldStart]) / TICK_DT)
        .ceil() as usize
        + 1;
    let full = (t[P::HeavyCharge] / TICK_DT).ceil() as usize;
    for (extra, want_full) in [(3, false), (full + 5, true)] {
        let mut w = World::new("test", 20.0);
        let (st, ev) = hold_attack(&mut w, &t, into + extra, 3);
        assert!(
            st.iter().any(|s| matches!(s, PState::Charging { .. })),
            "charged"
        );
        let heavy = st.iter().find_map(|s| match s {
            PState::Heavy { full, .. } => Some(*full),
            _ => None,
        });
        assert_eq!(heavy, Some(want_full), "extra {extra}");
        let charged = ev.iter().filter(|e| matches!(e, Event::Charged)).count();
        assert_eq!(charged, want_full as usize);
    }
}

/// 0051: 망치 강타는 친 자리에 금을 내고, 늦게 여진이 둘레를 다시 친다.
#[test]
fn the_hammer_slam_leaves_an_aftershock() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    w.set_weapon(weapons::Id::Hammer);
    let far = mob_with(&mut w, vec3(0.0, 0.0, 8.0), 500);
    idle(&mut w);
    w.step(
        &Actions {
            attack: true,
            attack_held: true,
            ..Default::default()
        },
        &t,
    );
    let hold = Actions {
        attack_held: true,
        ..Default::default()
    };
    for _ in 0..200 {
        w.player.stamina = 100.0;
        w.step(&hold, &t);
        if matches!(w.player.state, PState::Charging { t: e } if e > 1.0) {
            break;
        }
    }
    let mut ev = Vec::new();
    for _ in 0..30 {
        ev.extend(w.step(&Actions::default(), &t));
    }
    assert_eq!(w.cracks.len(), 1, "a crack under the slam");
    // 여진 자리로 적을 옮겨 두고 기다린다.
    w.enemies[far].pos = w.cracks[0].pos + vec3(0.8, 0.0, 0.0);
    idle(&mut w);
    for _ in 0..150 {
        ev.extend(w.step(&Actions::default(), &t));
        idle(&mut w);
    }
    assert!(ev.iter().any(|e| matches!(e, Event::Aftershock { .. })));
    assert!(w.enemies[far].hp < 500, "aftershock hits");
    assert!(w.cracks.is_empty());
}

/// 0051: 지팡이는 다 모인 뒤 짧은 창 안에 떼야 "딱 맞춤", 늦게 떼면 보통 강타.
#[test]
fn every_weapon_rewards_a_perfectly_timed_heavy_release() {
    // 0078(사장 2026-11-07): 딱 맞춰 떼기는 지팡이만이 아니라 강타가 있는 모든 무기의 규칙.
    let t = Tuning::default();
    for id in weapons::IDS.into_iter().filter(|i| i.def().heavy.is_some()) {
        let w0 = id.def();
        let full = t[P::HeavyCharge] * w0.heavy.unwrap().charge;
        for (extra, want) in [(0.05, true), (0.6, false)] {
            let mut w = World::new("test", 30.0);
            w.set_weapon(id);
            w.step(
                &Actions {
                    attack: true,
                    attack_held: true,
                    ..Default::default()
                },
                &t,
            );
            let hold = Actions {
                attack_held: true,
                ..Default::default()
            };
            let mut charged_for = 0.0;
            for _ in 0..400 {
                w.player.stamina = 100.0;
                w.step(&hold, &t);
                if let PState::Charging { t: e } = w.player.state {
                    charged_for = e;
                    if e >= full + extra {
                        break;
                    }
                }
            }
            assert!(charged_for >= full);
            w.step(&Actions::default(), &t);
            let got = matches!(w.player.state, PState::Heavy { perfect: true, .. });
            assert_eq!(got, want, "{id:?}: released {extra}s after full");
        }
    }
}

/// 0047: 모으는 동안 느리게 걷는다(제자리가 아니다) — 걷기의 `heavy.charge walk x`배쯤.
#[test]
fn charging_walks_slowly() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    w.view_yaw = Some(0.0);
    let walk = Actions {
        move_dir: vec2(0.0, 1.0),
        ..Default::default()
    };
    w.step(
        &Actions {
            attack: true,
            attack_held: true,
            ..walk
        },
        &t,
    );
    let hold = Actions {
        attack_held: true,
        ..walk
    };
    for _ in 0..120 {
        if matches!(w.player.state, PState::Charging { .. }) {
            break;
        }
        w.step(&hold, &t);
    }
    assert!(matches!(w.player.state, PState::Charging { .. }));
    for _ in 0..30 {
        w.step(&hold, &t);
    }
    assert!(matches!(w.player.state, PState::Charging { .. }));
    let v = w.player.vel.length();
    let want = t[P::MoveSpeed] * t[P::HeavyChargeWalk];
    assert!((v - want).abs() < want * 0.1, "{v} m/s, want {want}");
}

/// 0030: 다 모은 강타 = `heavy.damage` + 덤, 잡몹은 길게 휘청(예고를 끊는다), 보스는 다 모았을 때만 짧게.
#[test]
fn a_full_heavy_hits_hard_and_staggers() {
    let t = Tuning::default();
    let full = (t[P::HeavyCharge] / TICK_DT).ceil() as usize;
    for boss in [false, true] {
        for charged in [false, true] {
            let mut w = World::new("test", 20.0);
            if boss {
                w.spawn_boss(vec3(0.0, 0.0, -1.5));
            } else {
                w.spawn_mob(vec3(0.0, 0.0, -1.2));
                w.enemies[0].hp = 500;
                w.enemies[0].max_hp = 500;
            }
            idle(&mut w);
            let hp0 = w.enemies[0].hp;
            // 모으기에 들어설 때까지 쥔다(평타가 맞으면 히트스톱으로 틱이 멈추니 틱 수로 세지 않는다).
            let mut ev = w.step(
                &Actions {
                    attack: true,
                    attack_held: true,
                    ..Default::default()
                },
                &t,
            );
            let hold = Actions {
                attack_held: true,
                ..Default::default()
            };
            for _ in 0..120 {
                if matches!(w.player.state, PState::Charging { .. }) {
                    break;
                }
                ev.extend(w.step(&hold, &t));
            }
            // 다 모은 뒤 딱 맞춤 창(0078)을 지나서 뗀다 — 이 시험은 "다 모음"만 본다(딱 맞춤은 따로).
            let past_perfect = (PERFECT_RELEASE / TICK_DT).ceil() as usize + 2;
            for _ in 0..if charged { full + past_perfect } else { 3 } {
                ev.extend(w.step(&hold, &t));
            }
            assert!(matches!(w.player.state, PState::Charging { .. }));
            // 평타 한 대는 이미 맞았다. 이제 놓는다.
            let hp1 = w.enemies[0].hp;
            assert_eq!(hp0 - hp1, 10, "the light hit first");
            assert!(!ev.iter().any(|e| matches!(e, Event::Smash { .. })));
            let mut ev = Vec::new();
            let mut staggered = false;
            for _ in 0..30 {
                ev.extend(w.step(&Actions::default(), &t));
                staggered |= matches!(w.enemies[0].state, EState::Stagger { .. });
                idle_unless_staggered(&mut w);
            }
            assert!(
                ev.iter().any(|e| matches!(e, Event::Smash { .. })),
                "boss {boss} charged {charged}"
            );
            let want = w.player.weapon.def().heavy.unwrap().damage
                + if charged {
                    t[P::HeavyFullBonus] as i32
                } else {
                    0
                };
            assert_eq!(hp1 - w.enemies[0].hp, want, "boss {boss} charged {charged}");
            assert_eq!(staggered, !boss || charged, "boss {boss} charged {charged}");
        }
    }
}

/// `idle`과 같지만 휘청은 그대로 둔다(강타의 휘청을 보려고).
fn idle_unless_staggered(w: &mut World) {
    for e in &mut w.enemies {
        if !matches!(e.state, EState::Stagger { .. }) {
            e.state = EState::Recover { t: -1000.0 };
        }
    }
}

/// 0030(사장 "급하면 막 연타할텐데"): 연타는 모으기로 새지 않는다 — 누름 길이 1~9틱(≤0.15초) × 사이 2~14틱,
/// 누르는 때가 휘두르기의 어느 자리에 걸리든. 예전(쥐고 있나만 봄)엔 휘두르기 도중의 짧은 누름이 활성 끝에 걸려 강타가 나갔다.
#[test]
fn mashing_never_charges_a_heavy() {
    let t = Tuning::default();
    for press in 1..=9usize {
        for gap in 2..=14usize {
            let mut w = World::new("test", 20.0);
            let period = press + gap;
            let mut swings = 0;
            for i in 0..240 {
                let k = i % period;
                w.player.stamina = 100.0; // 스태미나가 바닥나 멈추지 않게(연타만 본다)
                w.step(
                    &Actions {
                        attack: k == 0,
                        attack_held: k < press,
                        ..Default::default()
                    },
                    &t,
                );
                assert!(
                    !matches!(
                        w.player.state,
                        PState::Charging { .. } | PState::Heavy { .. }
                    ),
                    "press {press} gap {gap} tick {i}"
                );
                if matches!(w.player.state, PState::Attack { t: e, .. } if e == 0.0) {
                    swings += 1;
                }
            }
            assert!(swings >= 3, "press {press} gap {gap}: {swings} swings");
        }
    }
}

/// 0030: 모으는 중에도 회피로 끊는다(강타는 나가지 않는다).
#[test]
fn a_dodge_cancels_the_charge() {
    let t = Tuning::default();
    let into = ((t[P::AttackStartup] + t[P::AttackActive]).max(t[P::HeavyHoldStart]) / TICK_DT)
        .ceil() as usize
        + 1;
    let mut w = World::new("test", 20.0);
    hold_attack(&mut w, &t, into + 5, 0);
    assert!(matches!(w.player.state, PState::Charging { .. }));
    w.player.dodge_cd = 0.0;
    w.step(
        &Actions {
            dodge: true,
            attack_held: true,
            ..Default::default()
        },
        &t,
    );
    assert!(matches!(w.player.state, PState::Dodge { .. }));
}

/// 0030(KB melee 노트 "모으기는 시간을 내준 만큼"): 평타 + 다 모은 강타의 초당 피해가 같은 시간 평타만 친 것보다 확실히 크다.
#[test]
fn a_full_heavy_pays_back_the_time_it_takes() {
    let t = Tuning::default();
    let light = t[P::AttackStartup] + t[P::AttackActive] + t[P::AttackRecovery];
    let heavy_time = (t[P::AttackStartup] + t[P::AttackActive]).max(t[P::HeavyHoldStart])
        + t[P::HeavyCharge]
        + t[P::HeavyStartup]
        + t[P::HeavyActive]
        + t[P::HeavyRecovery];
    // 평타 한 대(쥐기 전에 나간 것) + 다 모은 강타 대 평타만(둘 다 피해/초).
    let jab = weapons::Id::Sword.def().combo[0].damage as f32;
    let heavy_dmg =
        jab + weapons::Id::Sword.def().heavy.unwrap().damage as f32 + t[P::HeavyFullBonus];
    let (h, l) = (heavy_dmg / heavy_time, jab / light);
    assert!(h > l * 1.2, "heavy {h:.2}/s vs light {l:.2}/s");
}
