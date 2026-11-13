//! 원거리: 마법 탄과 칸, 커서 조준, 공중 RB, 쇠뇌(탄창·모은 사격·박자), 수류탄(던지기·겨누기·튕김), 기력 값.
use super::*;

/// 0048: RB 마법 — 한 칸을 쓰고 탄이 날아가 앞의 적을 친다(자동 조준).
#[test]
fn a_cast_spends_a_charge_and_the_bolt_hits() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    w.spawn_mob(vec3(1.0, 0.0, -6.0));
    w.enemies[0].hp = 90;
    idle(&mut w);
    let ev = press(
        &t,
        &mut w,
        Actions {
            cast: true,
            ..Default::default()
        },
        90,
    );
    assert!(ev.iter().any(|e| matches!(e, Event::Cast { .. })));
    // 한 칸을 쓰고, 그 사이(90틱) 시간으로 찬 몫만 더해졌다(0111).
    let refill = 90.0 * TICK_DT / t[P::CastCooldown];
    assert!(
        (w.player.ranged.mana - (CAST_CHARGES - 1.0 + refill)).abs() < 0.02,
        "{}",
        w.player.ranged.mana
    );
    let magic = match w.player.weapon.def().rb {
        weapons::Rb::Magic(b) => b.damage,
        weapons::Rb::Shot { bolt, .. } => bolt.damage,
    };
    assert_eq!(w.enemies[0].hp, 90 - magic);
    assert!(w.bolts.is_empty());
    assert!(matches!(w.player.state, PState::Idle));
}

/// 0058: 커서를 비춘 땅 위의 점은 오른쪽 스틱처럼 겨눈다 — 가까운 적(자동 조준)이 아니라 커서 쪽 적을 맞힌다.
/// 락온 중이면 커서는 쓰지 않는다(대상을 본다 — 스틱과 같다).
#[test]
fn the_cursor_point_aims_like_the_right_stick_and_lock_on_ignores_it() {
    let t = Tuning::default();
    let cast_at = |lock: bool| {
        let mut w = World::new("test", 30.0);
        w.spawn_mob(vec3(0.0, 0.0, -4.0)); // 가까운 앞쪽(자동 조준·락온이 고를 적)
        w.spawn_mob(vec3(7.0, 0.0, 0.0)); // 커서 쪽
        for e in &mut w.enemies {
            e.hp = 90;
        }
        idle(&mut w);
        if lock {
            press(
                &t,
                &mut w,
                Actions {
                    lock: true,
                    ..Default::default()
                },
                2,
            );
            assert!(w.lock.is_some());
        }
        press(
            &t,
            &mut w,
            Actions {
                cast: true,
                aim_at: Some(vec2(7.0, 0.0)),
                ..Default::default()
            },
            90,
        );
        (w.enemies[0].hp, w.enemies[1].hp)
    };
    let (near, cursor) = cast_at(false);
    assert_eq!(near, 90, "the near enemy is left alone");
    assert!(cursor < 90, "the bolt went the cursor's way");
    let (near, cursor) = cast_at(true);
    assert!(
        near < 90 && cursor == 90,
        "locked: the target, not the cursor"
    );
}

/// 0058: 커서는 자리까지 준다 — 수류탄 톡은 커서 자리에(2m..먼 거리로 자른다).
#[test]
fn a_tossed_grenade_lands_at_the_cursor() {
    let t = Tuning::default();
    let g = weapons::Id::Crossbow.def().grenade.unwrap();
    let mut w = World::new("test", 40.0);
    w.set_weapon(weapons::Id::Crossbow);
    w.view_yaw = Some(0.0);
    let p0 = w.player.pos;
    let want = vec2(p0.x + 4.0, p0.z + 3.0);
    w.step(
        &Actions {
            cast: true,
            aim_at: Some(want),
            ..Default::default()
        },
        &t,
    );
    let to = match w.player.state {
        PState::Throw { to, .. } => to,
        ref s => panic!("{s:?}"),
    };
    assert!(vec2(to.x, to.z).distance(want) < 0.05, "{to:?}");
    assert!(want.distance(vec2(p0.x, p0.z)) < g.far);
}

/// 0048: 칸이 모자라면 쏘지 않고(몸짓 없음) 칸이 깜빡인다. 0111: 칸은 시간으로만 찬다(`cast.cooldown s`에 한 칸) —
/// 근접으로 맞혀도 더 차지 않는다.
#[test]
fn magic_refills_with_time_not_hits_and_an_empty_cast_is_denied() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    w.player.ranged.mana = 0.5;
    let ev = press(
        &t,
        &mut w,
        Actions {
            cast: true,
            ..Default::default()
        },
        2,
    );
    assert!(ev.iter().any(|e| matches!(e, Event::CastDenied)));
    assert!(matches!(w.player.state, PState::Idle));
    assert!(w.player.ranged.cast_denied > 0.0);
    w.spawn_mob(vec3(0.0, 0.0, -1.2));
    w.enemies[0].hp = 500;
    w.enemies[0].max_hp = 500;
    idle(&mut w);
    let m0 = w.player.ranged.mana;
    press(
        &t,
        &mut w,
        Actions {
            attack: true,
            ..Default::default()
        },
        40,
    );
    assert_eq!(w.enemies[0].hp, 490);
    let by_time = 41.0 * TICK_DT / t[P::CastCooldown];
    assert!(
        w.player.ranged.mana - m0 <= by_time + 1e-4,
        "a hit adds nothing: {} -> {}",
        m0,
        w.player.ranged.mana
    );
    // 한 칸 = `cast.cooldown s`.
    w.enemies[0].state = EState::Dead;
    w.player.ranged.mana = 0.0;
    for _ in 0..(t[P::CastCooldown] / TICK_DT).round() as usize {
        w.step(&Actions::default(), &t);
    }
    assert!(
        (w.player.ranged.mana - 1.0).abs() < 0.02,
        "{}",
        w.player.ranged.mana
    );
    for _ in 0..600 {
        w.step(&Actions::default(), &t);
    }
    assert_eq!(w.player.ranged.mana, CAST_CHARGES, "stops at the cap");
}

/// 0111: 공중 RB(근접 무기) = 마법 한 칸을 비스듬히 내려 쏘고, 잠깐 떠 있다. 한 점프에 한 번.
#[test]
fn an_air_cast_fires_down_once_per_jump_and_hovers() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    w.step(
        &Actions {
            jump: true,
            ..Default::default()
        },
        &t,
    );
    for _ in 0..6 {
        w.step(&Actions::default(), &t);
    }
    let mana = w.player.ranged.mana;
    w.step(
        &Actions {
            cast: true,
            ..Default::default()
        },
        &t,
    );
    assert_eq!(w.bolts.len(), 1);
    assert!(
        w.bolts[0].dir.y < -0.05,
        "fired downward: {:?}",
        w.bolts[0].dir
    );
    assert!((w.player.ranged.mana - (mana - 1.0)).abs() < 0.01);
    // 체공: 떨어지지 않는다.
    let air = w.player.air;
    assert!(air > 0.2);
    for _ in 0..((AIR_SHOT_HOVER / TICK_DT) as usize - 2) {
        w.step(&Actions::default(), &t);
        assert!((w.player.air - air).abs() < 1e-4, "hovering");
    }
    // 같은 점프에 또 누르면 안 나간다.
    w.step(
        &Actions {
            cast: true,
            ..Default::default()
        },
        &t,
    );
    let mut n = 0;
    let mut landed = false;
    for _ in 0..120 {
        n = n.max(w.bolts.len());
        w.step(&Actions::default(), &t);
        landed |= matches!(w.player.state, PState::Land { .. });
    }
    assert!(n <= 1, "once per jump");
    assert!(landed);
    assert!(w.bolts.is_empty(), "the bolt sank into the floor");
}

/// 0111: 쇠뇌의 공중 RB = 내려 쏘기 — 볼트 `AIR_VOLLEY`발을 부채꼴로, 탄창은 그대로. 아래의 적을 맞힌다.
#[test]
fn the_crossbow_shoots_a_downward_volley_from_the_air() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    w.set_weapon(weapons::Id::Crossbow);
    w.spawn_mob(vec3(0.0, 0.0, -3.0));
    let hp = w.enemies[0].hp;
    w.player.facing = std::f32::consts::PI; // -z
    w.step(
        &Actions {
            jump: true,
            ..Default::default()
        },
        &t,
    );
    for _ in 0..6 {
        w.step(&Actions::default(), &t);
    }
    let ammo = w.player.ranged.ammo;
    w.step(
        &Actions {
            attack: true,
            ..Default::default()
        },
        &t,
    );
    assert_eq!(w.bolts.len(), AIR_VOLLEY);
    assert!(w.bolts.iter().all(|b| b.dir.y < 0.0));
    assert_eq!(w.player.ranged.ammo, ammo, "no ammo spent");
    for _ in 0..60 {
        w.step(&Actions::default(), &t);
    }
    assert!(w.enemies[0].hp < hp, "the volley hits the enemy below");
}

/// 0117: 쇠뇌 사격·수류탄도 기력을 쓰고, 기력이 없으면 나가지 않는다(볼트·수류탄도 그대로).
#[test]
fn shots_and_grenades_cost_stamina_and_stop_when_empty() {
    let t = Tuning::default();
    let mut w = World::new("test", 20.0);
    w.set_weapon(weapons::Id::Crossbow);
    let (s0, ammo) = (w.player.stamina, w.player.ranged.ammo);
    w.step(
        &Actions {
            attack: true,
            ..Default::default()
        },
        &t,
    );
    assert!(w.player.stamina < s0, "a shot costs stamina");
    assert_eq!(w.player.ranged.ammo, ammo - 1);
    // 기력 0: 사격도 수류탄도 막힌다.
    for _ in 0..60 {
        w.step(&Actions::default(), &t);
    }
    w.player.stamina = 0.0;
    w.player.stamina_idle = 0.0;
    let ammo = w.player.ranged.ammo;
    let bombs = w.player.ranged.grenades;
    w.step(
        &Actions {
            attack: true,
            ..Default::default()
        },
        &t,
    );
    assert_eq!(w.player.ranged.ammo, ammo, "no stamina, no shot");
    w.player.stamina = 0.0;
    w.player.stamina_idle = 0.0;
    w.step(
        &Actions {
            cast: true,
            ..Default::default()
        },
        &t,
    );
    assert!(
        (w.player.ranged.grenades - bombs).abs() < 0.05,
        "no stamina, no throw"
    );
}

/// 0048: 탄은 기둥에 막힌다.
#[test]
fn a_bolt_stops_at_a_pillar() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    w.obstacles.push(Aabb {
        min: vec3(-1.0, 0.0, -3.5),
        max: vec3(1.0, 3.0, -2.5),
    });
    w.spawn_mob(vec3(0.0, 0.0, -6.0));
    w.enemies[0].hp = 90;
    idle(&mut w);
    press(
        &t,
        &mut w,
        Actions {
            cast: true,
            ..Default::default()
        },
        90,
    );
    assert_eq!(w.enemies[0].hp, 90);
    assert!(w.bolts.is_empty());
}

/// 0051: 쇠뇌 — RB는 탄창에서 쏘고, 비면 장전 뒤 가득, 회피하면 한 발 장전. 칸(마법)은 쓰지 않는다.
#[test]
fn the_crossbow_shoots_from_a_magazine_and_reloads() {
    let t = Tuning::default();
    let mut w = World::new("test", 40.0);
    w.set_weapon(weapons::Id::Crossbow);
    let (mag, reload) = match weapons::Id::Crossbow.def().rb {
        Rb::Shot { mag, reload, .. } => (mag, reload),
        _ => unreachable!(),
    };
    assert_eq!(w.player.ranged.ammo, mag);
    let mana = w.player.ranged.mana;
    let cast = Actions {
        attack: true,
        ..Default::default()
    };
    for _ in 0..mag {
        press(&t, &mut w, cast, 40);
    }
    assert_eq!(w.player.ranged.ammo, 0);
    assert_eq!(w.player.ranged.mana, mana, "no magic spent");
    let ev = press(&t, &mut w, cast, 2);
    assert!(ev.iter().any(|e| matches!(e, Event::CastDenied)), "empty");
    // 회피 한 번 = 한 발.
    press(
        &t,
        &mut w,
        Actions {
            dodge: true,
            ..Default::default()
        },
        2,
    );
    assert_eq!(w.player.ranged.ammo, 1);
    // 장전이 끝나면 가득.
    press(
        &t,
        &mut w,
        Actions::default(),
        (reload / TICK_DT) as usize + 10,
    );
    assert_eq!(w.player.ranged.ammo, mag);
}

/// 0051: 쇠뇌의 X = 수류탄 — 날아가 도화선이 다하면 둘레를 친다(셀 수 있고 다시 찬다).
#[test]
fn a_grenade_flies_bounces_and_blows_up() {
    let t = Tuning::default();
    let mut w = World::new("test", 40.0);
    w.set_weapon(weapons::Id::Crossbow);
    let g = weapons::Id::Crossbow.def().grenade.unwrap();
    let m = mob_with(&mut w, vec3(0.0, 0.0, -5.0), 200);
    idle(&mut w);
    let mut ev = w.step(
        &Actions {
            cast: true,
            ..Default::default()
        },
        &t,
    );
    assert!((w.player.ranged.grenades - (g.count as f32 - 1.0)).abs() < 0.05);
    for _ in 0..((g.fuse + 0.6) / TICK_DT) as usize {
        ev.extend(w.step(&Actions::default(), &t));
        idle(&mut w);
        w.enemies[m].pos = vec3(0.0, 0.0, -5.0);
    }
    assert!(ev.iter().any(|e| matches!(e, Event::Throw)));
    assert!(ev.iter().any(|e| matches!(e, Event::Boom { .. })));
    assert!(
        w.enemies[m].hp <= 200 - g.damage,
        "blast hits: {}",
        w.enemies[m].hp
    );
    assert!(w.bombs.is_empty());
}

/// 0055: 쇠뇌 X를 꾹 쥐면 손을 든 채 겨누다가 떼면 가장 먼 자리로 던진다(톡은 가까운 자동 조준 자리).
#[test]
fn holding_the_grenade_aims_far() {
    let t = Tuning::default();
    let g = weapons::Id::Crossbow.def().grenade.unwrap();
    let mut w = World::new("test", 40.0);
    w.set_weapon(weapons::Id::Crossbow);
    let hold = Actions {
        cast_held: true,
        move_dir: vec2(1.0, 0.0),
        ..Default::default()
    };
    w.view_yaw = Some(0.0);
    w.step(&Actions { cast: true, ..hold }, &t);
    for _ in 0..40 {
        w.step(&hold, &t);
    }
    let PState::Throw {
        to, thrown: false, ..
    } = w.player.state
    else {
        panic!("aiming: {:?}", w.player.state)
    };
    let d = vec3(to.x - w.player.pos.x, 0.0, to.z - w.player.pos.z).length();
    assert!((d - g.far).abs() < 0.2, "aims at the far ring: {d}");
    let mut ev = Vec::new();
    for _ in 0..3 {
        ev.extend(w.step(&Actions::default(), &t));
    }
    assert!(ev.iter().any(|e| matches!(e, Event::Throw)));
    assert_eq!(w.bombs.len(), 1);
}

/// 쏘는 버튼을 누른 채 쥔다(0151: 근접 무기 = RB 마법, 원거리 무기 = X 사격): 한 발 쏘고 모으기로 들어가 `ticks`틱 쥔 뒤 뗀다 —
/// 뗀 직후의 탄들.
fn charge_cast(w: &mut World, t: &Tuning, ticks: usize) -> Vec<Bolt> {
    let ranged = w.player.weapon.def().grenade.is_some();
    let held = Actions {
        attack_held: ranged,
        cast_held: !ranged,
        ..Default::default()
    };
    w.step(
        &Actions {
            attack: ranged,
            cast: !ranged,
            ..held
        },
        t,
    );
    for _ in 0..200 {
        if matches!(w.player.state, PState::CastCharge { .. }) {
            break;
        }
        w.step(&held, t);
    }
    assert!(
        matches!(w.player.state, PState::CastCharge { .. }),
        "{:?}",
        w.player.state
    );
    for _ in 0..ticks {
        w.step(&held, t);
    }
    w.bolts.clear();
    w.step(&Actions::default(), t);
    w.bolts.clone()
}

/// 0056: RB를 쥐면 한 발 쏜 뒤 모으기 — 다 모아 떼면 무기 표의 모은 탄(크고 세다), 칸을 하나 더 쓴다.
#[test]
fn holding_rb_charges_a_stronger_magic_bolt() {
    let t = Tuning::default();
    let full = (t[P::CastCharge] / TICK_DT).ceil() as usize + 2;
    let mut w = World::new("test", 40.0);
    let got = charge_cast(&mut w, &t, full);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].def, weapons::Id::Sword.def().charged);
    // 톡 한 칸 + 모은 탄 한 칸 — 그 사이 시간으로 찬 몫(0111)은 한 칸에 못 미친다.
    assert!(
        w.player.ranged.mana >= CAST_CHARGES - 2.0 && w.player.ranged.mana < CAST_CHARGES - 1.0,
        "one for the tap, one for the charge: {}",
        w.player.ranged.mana
    );
    // 덜 모으고 떼면 약하게(반 피해, 꿰뚫기 없음).
    let mut w = World::new("test", 40.0);
    let got = charge_cast(&mut w, &t, 3);
    let c = weapons::Id::Sword.def().charged;
    // 반은 옛 눈금으로 올림(0158 N0): 30 → 옛 (3 + 1) / 2 = 2의 열 배.
    assert_eq!((c.damage, got[0].def.damage), (30, 20));
    assert!(!got[0].def.pierce);
}

/// 0056: 쇠뇌는 모은 사격 — 볼트 하나를 쓰고 꿰뚫는다. 볼트가 없으면 모으지 않는다.
#[test]
fn the_crossbow_charges_a_piercing_shot() {
    let t = Tuning::default();
    let full = (t[P::CastCharge] / TICK_DT).ceil() as usize;
    let past_perfect = (PERFECT_RELEASE / TICK_DT).ceil() as usize + 2;
    let charged = weapons::Id::Crossbow.def().charged;
    let mut w = World::new("test", 40.0);
    w.set_weapon(weapons::Id::Crossbow);
    let mag = w.player.ranged.ammo;
    let got = charge_cast(&mut w, &t, full + past_perfect);
    assert!(got[0].def.pierce && got[0].def.damage == charged.damage);
    assert_eq!(w.player.ranged.ammo, mag - 2);
    // 딱 맞춰 떼기(0078): 다 모인 바로 뒤에 떼면 피해 +10(옛 +1) — 강타와 같은 규칙.
    let mut w = World::new("test", 40.0);
    w.set_weapon(weapons::Id::Crossbow);
    let got = charge_cast(&mut w, &t, full + 2);
    assert!(got[0].def.pierce && got[0].def.damage == charged.damage + 10);
    // 근접 무기의 RB 마법은 무기의 모으기 공격이 아니다 — 딱 맞춤 없음.
    let mut w = World::new("test", 40.0);
    let got = charge_cast(&mut w, &t, full + 2);
    assert_eq!(got[0].def.damage, weapons::Id::Sword.def().charged.damage);
    // 볼트가 다 떨어지면 쥐어도 모으기로 가지 않는다.
    w.player.ranged.ammo = 1;
    let held = Actions {
        attack_held: true,
        ..Default::default()
    };
    w.step(
        &Actions {
            attack: true,
            ..held
        },
        &t,
    );
    for _ in 0..80 {
        w.step(&held, &t);
        assert!(!matches!(w.player.state, PState::CastCharge { .. }));
    }
}

/// 0077(사장 2026-11-07 "원거리 무기도 박자 있는거야. 근데 버튼이 다를뿐") → 0151(이제 버튼도 같다 — 사격이 X): 쇠뇌는 사격이
/// 박자를 탄다 — 앞 사격에서 한 발의 시간(= 공격 속도) ± 창 안에 누른 사격은 볼트를 쓰지 않는다, 연사는 쓴다.
#[test]
fn a_crossbow_shot_on_the_beat_keeps_its_bolt_and_mashing_spends_them() {
    let t = Tuning::default();
    let shoot = |w: &mut World, press: bool| {
        w.step(
            &Actions {
                attack: press,
                ..Default::default()
            },
            &t,
        )
    };
    let mut w = World::new("test", 20.0);
    w.set_weapon(weapons::Id::Crossbow);
    let full = w.player.ranged.ammo;
    shoot(&mut w, true);
    assert_eq!(
        w.player.ranged.ammo,
        full - 1,
        "the first shot spends a bolt"
    );
    let beat = w.player.beat.period;
    assert!(beat > 0.2, "beat {beat}");
    let mut got = false;
    for _ in 0..120 {
        let press = w.player.beat.since_swing >= beat - TICK_DT * 0.5;
        let ev = shoot(&mut w, press);
        if press {
            assert!(ev.contains(&Event::Beat), "on the beat");
            assert_eq!(w.player.ranged.ammo, full - 1, "no bolt for the beat shot");
            assert!(w.player.beat.chain == 1 && w.player.beat.since < TICK_DT * 2.0);
            got = true;
            break;
        }
    }
    assert!(got);
    // 연사(매 틱 누름)는 박자를 공짜로 얻지 못한다.
    let mut w = World::new("test", 20.0);
    w.set_weapon(weapons::Id::Crossbow);
    let mut beats = 0;
    for _ in 0..30 {
        beats += shoot(&mut w, true)
            .iter()
            .filter(|e| **e == Event::Beat)
            .count();
    }
    assert_eq!(beats, 0);
    assert!(w.player.ranged.ammo < full, "mashing spends bolts");
    // 창은 아무리 빨라도 ±60ms 아래로 좁아지지 않는다.
    assert!(on_beat(0.24 + 0.055, 0.24) && !on_beat(0.24 + 0.07, 0.24));
}

/// 원거리 시험 마당: 5×8칸(2m), z가 음수인 위쪽 네 줄의 높이가 `far`, 아래 네 줄은 0. 방랑자는 아래쪽(z = +4)에서 위(-z)를 본다.
fn split_yard(far: f32) -> World {
    use crate::ground::{Cell::Flat, Ground};
    let mut cells = vec![Flat(0.0); 5 * 8];
    for c in cells.iter_mut().take(5 * 4) {
        *c = Flat(far);
    }
    let mut w = World::new("test", 30.0);
    w.ground = Some(Ground::new(5, 8, 2.0, cells));
    w.view_yaw = Some(0.0);
    w.player.pos = vec3(0.0, 0.0, 4.0);
    w.player.prev = w.player.pos;
    w.player.facing = std::f32::consts::PI;
    w
}

/// 사장 2026-11-13 "고도 차이가 있을 때 원거리 공격이 안 맞는다": 수평으로 쏜 탄은 땅을 따라 흐른다 — 낮은 곳에 선 적의 머리
/// 위로 지나가지 않고(전에는 쏜 높이로 곧게 날았다), 턱 위에 선 적 앞에서 턱에 박히지도 않는다. 벼랑처럼 높은 단에서는 사라진다.
#[test]
fn a_level_shot_follows_the_ground_to_foes_above_and_below() {
    let t = Tuning::default();
    let shoot = |w: &mut World| {
        w.enemies[0].hp = 900;
        idle(w);
        press(
            &t,
            w,
            Actions {
                cast: true,
                ..Default::default()
            },
            90,
        )
    };
    let hit = |ev: &[Event]| ev.iter().any(|e| matches!(e, Event::Hit { .. }));
    for (far, mine, tag) in [
        (-1.0, 0.0, "a foe one metre below"),
        (-2.5, 0.0, "a foe down a tall drop"),
        (1.0, 0.0, "a foe up a one-metre ledge"),
        (0.0, 0.0, "flat ground still hits"),
    ] {
        let mut w = split_yard(far);
        w.player.pos.y = mine;
        w.spawn_mob(vec3(0.0, far, -4.0));
        let ev = shoot(&mut w);
        assert!(hit(&ev), "{tag}");
        assert!(w.enemies[0].hp < 900, "{tag}");
    }
    // 방랑자가 위에 서서 아래로: 같은 일(땅 높이는 음수만이 아니다).
    let mut w = split_yard(0.0);
    {
        use crate::ground::{Cell::Flat, Ground};
        let mut cells = vec![Flat(0.0); 5 * 8];
        for c in cells.iter_mut().skip(5 * 4) {
            *c = Flat(1.5);
        }
        w.ground = Some(Ground::new(5, 8, 2.0, cells));
    }
    w.player.pos.y = 1.5;
    w.player.prev = w.player.pos;
    w.spawn_mob(vec3(0.0, 0.0, -4.0));
    assert!(hit(&shoot(&mut w)), "shooting down off a terrace");
    // 벼랑(넘는 턱보다 높은 단) 위의 적에게는 닿지 않는다 — 탄은 벼랑 면에서 사라진다.
    let mut w = split_yard(3.0);
    w.spawn_mob(vec3(0.0, 3.0, -4.0));
    assert!(!hit(&shoot(&mut w)), "a cliff face stops the bolt");
    assert!(w.bolts.is_empty());
}

/// 재는 도구(0173): 무기마다 서 있는 적에게 `secs`초 동안 준 피해(초당) — 평타를 박자에 맞춰/마구 눌러.
/// `cargo test weapon_dps_probe -- --ignored --nocapture`
fn dps(id: weapons::Id, secs: f32, on_beat: bool, dist: f32) -> (f32, f32) {
    let t = Tuning::default();
    let mut w = World::new("test", 40.0);
    w.set_weapon(id);
    let hp0 = 1_000_000;
    let i = mob_with(&mut w, vec3(0.0, 0.0, -dist), hp0);
    let n = (secs / TICK_DT) as usize;
    let mut starved = 0;
    for k in 0..n {
        idle(&mut w);
        w.enemies[i].pos = vec3(0.0, 0.0, -dist);
        w.enemies[i].poise = 0.0;
        w.player.pos = Vec3::ZERO;
        let press = if on_beat {
            let beat = w.player.beat.period;
            beat <= 0.0 || w.player.beat.since_swing >= beat - TICK_DT * 0.5
        } else {
            k % 2 == 0
        };
        if w.player.stamina < 1.0 {
            starved += 1;
        }
        w.step(
            &Actions {
                attack: press,
                ..Default::default()
            },
            &t,
        );
    }
    (
        (hp0 - w.enemies[i].hp) as f32 / secs,
        starved as f32 / n as f32,
    )
}

#[test]
#[ignore]
fn weapon_dps_probe() {
    for id in weapons::IDS {
        let d = if id == weapons::Id::Crossbow {
            8.0
        } else {
            1.2
        };
        for secs in [4.0, 20.0] {
            let (mash, s1) = dps(id, secs, false, d);
            let (beat, s2) = dps(id, secs, true, d);
            println!(
                "{:?} {secs}s: mash {mash:.1}/s (기력 바닥 {:.0}%) · beat {beat:.1}/s (기력 바닥 {:.0}%)",
                id,
                s1 * 100.0,
                s2 * 100.0
            );
        }
    }
}
