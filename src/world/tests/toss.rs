//! 0274 ②b: RB를 종류마다(던지는 손도끼 · 신성한 빛 · 창 던지기 · 마법탄 · 돌 던지기)와 쇠뇌·활의 손 리듬(장전 / 당기기).
use super::*;

/// 재는 도구(0274 ②b): RB만 눌러 서 있는 적에게 `secs`초 동안 준 피해(초당) — 근접 무기는 종류의 RB(칸이 차는 대로),
/// 원거리 무기는 사격. `hold` = 쥐고 다 모은 뒤 뗀다(모은 RB · 끝까지 당긴 화살), 아니면 마구 누른다.
fn rb_dps(id: weapons::Id, secs: f32, dist: f32, hold: bool, boss: bool) -> f32 {
    let t = Tuning::default();
    let mut w = World::new("test", 40.0);
    w.set_weapon(id);
    let at = vec3(0.0, 0.0, -dist);
    let i = if boss {
        w.spawn(crate::species::Kind::Warden, at)
    } else {
        w.spawn_mob(at);
        w.enemies.len() - 1
    };
    let hp0 = 1_000_000;
    w.enemies[i].hp = hp0;
    w.enemies[i].max_hp = hp0;
    let n = (secs / TICK_DT) as usize;
    let mut released = false;
    for k in 0..n {
        idle(&mut w);
        w.enemies[i].pos = at;
        w.enemies[i].stance.poise = 0.0;
        w.player.pos = Vec3::ZERO;
        let pl = &w.player;
        let mut a = Actions::default();
        if hold {
            let full =
                matches!(pl.state, PState::CastCharge { t: e } if e >= t[P::CastCharge] + TICK_DT);
            if full || released {
                released = !released && full;
            } else {
                a.cast = !matches!(pl.state, PState::Cast { .. } | PState::CastCharge { .. });
                a.cast_held = true;
            }
        } else {
            a.cast = k % 2 == 0;
        }
        w.step(&a, &t);
    }
    (hp0 - w.enemies[0].hp.min(w.enemies[i].hp)) as f32 / secs
}

/// `cargo test rb_dps_probe -- --ignored --nocapture`
#[test]
#[ignore]
fn rb_dps_probe() {
    for id in weapons::IDS {
        for (secs, boss) in [(4.0, false), (20.0, false), (20.0, true)] {
            let mash = rb_dps(id, secs, 8.0, false, boss);
            let hold = rb_dps(id, secs, 8.0, true, boss);
            println!(
                "{id:?} {secs}s{}: RB mash {mash:.1}/s · hold {hold:.1}/s",
                if boss { " (warden)" } else { "" }
            );
        }
    }
}

/// 쥐었다 뗀다: 첫 틱에 누르고 `secs`초 쥔 뒤 뗀다 — 뗀 틱까지의 사건.
fn hold_release(w: &mut World, t: &Tuning, secs: f32) -> Vec<Event> {
    let mut ev = w.step(
        &Actions {
            cast: true,
            cast_held: true,
            ..Default::default()
        },
        t,
    );
    for _ in 0..(secs / TICK_DT).round() as usize {
        ev.extend(w.step(
            &Actions {
                cast_held: true,
                ..Default::default()
            },
            t,
        ));
    }
    ev.extend(w.step(&Actions::default(), t));
    ev
}

/// `n`틱 그냥 흐른다.
fn wait(w: &mut World, t: &Tuning, secs: f32) -> Vec<Event> {
    let mut ev = Vec::new();
    for _ in 0..(secs / TICK_DT).round() as usize {
        ev.extend(w.step(&Actions::default(), t));
    }
    ev
}

/// 0274 ②b: 한손검의 RB = 던지는 손도끼 — 짧다(7.2 m 안은 맞고 그 밖은 안 닿는다), 마법 탄보다 무겁다(피해 20 · 자세 피해 4배).
#[test]
fn the_hand_axe_is_short_and_heavy() {
    let t = Tuning::default();
    let hit = |dist: f32| {
        let mut w = World::new("test", 30.0);
        let i = mob_with(&mut w, vec3(0.0, 0.0, -dist), 900);
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
        assert!(ev
            .iter()
            .any(|e| matches!(e, Event::Cast { look, .. } if *look == weapons::AXE)));
        (900 - w.enemies[i].hp, w.enemies[i].stance.poise)
    };
    let (near, poise) = hit(5.0);
    assert_eq!(near, 20, "a heavy axe");
    assert!(
        poise >= 4.0 * weapons::Id::Staff.bolts().0.poise - 1e-3,
        "{poise}"
    );
    assert_eq!(hit(9.0).0, 0, "short: 7.2 m");
}

/// 0274 ②b · CLAUDE.md "새 탄·던지는 것을 만들면 높낮이가 다른 마당에서": 던지는 것(손도끼 · 창 · 돌)은 곧게 날아 겨눈 줄 위 첫 적의
/// 몸통으로 비스듬히 간다 — 한 단 아래·위의 적도 맞는다. 벼랑 위(3 m)의 적에게는 벼랑 면에 막힌다.
#[test]
fn thrown_rbs_fly_straight_to_foes_above_and_below() {
    use super::ranged::split_yard;
    let t = Tuning::default();
    for id in [weapons::Id::Sword, weapons::Id::Spear, weapons::Id::Fists] {
        for (far, tag) in [
            (-1.0, "a foe one metre below"),
            (1.0, "a foe on a one-metre ledge"),
        ] {
            let mut w = split_yard(far);
            w.set_weapon(id);
            w.spawn_mob(vec3(0.0, far, -2.0));
            w.enemies[0].hp = 900;
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
            assert!(w.enemies[0].hp < 900, "{id:?}: {tag} ({ev:?})");
        }
        let mut w = split_yard(3.0);
        w.set_weapon(id);
        w.spawn_mob(vec3(0.0, 3.0, -2.0));
        w.enemies[0].hp = 900;
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
        assert_eq!(w.enemies[0].hp, 900, "{id:?}: a cliff face stops it");
    }
}

/// 0274 ②b: 망치의 RB = 신성한 빛 — 날지 않고 겨눈 자리(미리보기와 같은 셈 `smite_spot` — 줄 위의 첫 적)에 `SMITE_DELAY`초 뒤
/// 내려 꽂혀 둘레를 친다: 곁에 붙은 둘째 적도 맞고, 울림이 쌓인다(모은 빛은 잡몹을 한 번에 울린다). 기둥 너머로는 가지 않는다.
#[test]
fn holy_light_falls_where_the_preview_says_and_rings_the_crowd() {
    use crate::tuning::{SMITE_DELAY, SMITE_RADIUS};
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    w.set_weapon(weapons::Id::Hammer);
    let a = mob_with(&mut w, vec3(0.0, 0.0, -6.0), 900);
    let b = mob_with(&mut w, vec3(0.9, 0.0, -6.6), 900);
    idle(&mut w);
    let face = vec3(0.0, 0.0, -1.0);
    w.player.facing = std::f32::consts::PI;
    let said = w.smite_spot(face, weapons::Id::Hammer.bolts().0, &t);
    assert!(said.distance(vec3(0.0, 0.0, -6.0)) < 0.05, "{said:?}");
    let mut ev = w.step(
        &Actions {
            cast: true,
            ..Default::default()
        },
        &t,
    );
    let mut fell = None;
    let mut ticks = 0;
    for k in 0..90 {
        let e = w.step(&Actions::default(), &t);
        if let Some(at) = e.iter().find_map(|e| match e {
            Event::Smite { at, radius } => Some((*at, *radius)),
            _ => None,
        }) {
            fell = Some(at);
            ticks = k + 1;
        }
        ev.extend(e);
    }
    let (at, radius) = fell.expect("the light falls");
    assert!(at.distance(said) < 0.05, "said {said:?}, fell {at:?}");
    assert!((radius - SMITE_RADIUS).abs() < 1e-4);
    assert!(
        ticks as f32 * TICK_DT >= SMITE_DELAY - 0.2,
        "it waits a beat"
    );
    assert_eq!(w.enemies[a].hp, 880);
    assert_eq!(w.enemies[b].hp, 880, "the one beside it too");
    assert!(w.bolts.is_empty(), "nothing flies");
    // 모은 빛: 넓고(×1.5) 울림 가득 — 잡몹은 울린다.
    let mut w = World::new("test", 30.0);
    w.set_weapon(weapons::Id::Hammer);
    let a = mob_with(&mut w, vec3(0.0, 0.0, -5.0), 900);
    idle(&mut w);
    let mut ev = hold_release(&mut w, &t, t[P::CastHoldStart] + t[P::CastCharge] + 0.1);
    ev.extend(wait(&mut w, &t, 1.5));
    // 쥐면 보통 빛 하나를 부르고(0056 — 쏜 뒤에도 쥐면 모으기) 떼면 모은 빛.
    let r = ev
        .iter()
        .filter_map(|e| match e {
            Event::Smite { radius, .. } => Some(*radius),
            _ => None,
        })
        .fold(0.0, f32::max);
    assert!(r > SMITE_RADIUS * 1.4, "{r}");
    assert!(
        ev.iter().any(|e| matches!(e, Event::Toll { .. })) || w.enemies[a].ail.toll_in.is_some(),
        "a full knell dose rings a plain foe"
    );
    // 기둥(벽) 너머의 적은 빛의 자리가 되지 못한다 — 빛은 기둥 앞에 꽂힌다.
    let mut w = World::new("test", 30.0);
    w.set_weapon(weapons::Id::Hammer);
    let p0 = w.player.pos;
    w.obstacles.push(Aabb {
        min: vec3(p0.x - 1.5, 0.0, p0.z - 4.0),
        max: vec3(p0.x + 1.5, 3.0, p0.z - 3.0),
    });
    mob_with(&mut w, p0 + vec3(0.0, 0.0, -6.0), 900);
    idle(&mut w);
    let spot = w.smite_spot(face, weapons::Id::Hammer.bolts().0, &t);
    assert!((p0.z - spot.z) < 3.2, "stops at the pillar: {spot:?}");
}

/// 0274 ②b: 창의 RB = 창 던지기 — 줄 위의 적을 모두 꿰뚫고(20씩) 멀리(15.6 m) 날아, 멈춘 자리에서 손으로 돌아온다(오는 길은
/// 치지 않는다). 날고 있는 동안 RB는 막히고 손에는 창이 없다(`weapon_away`), 평타를 누르면 곧바로 손에 돌아온다.
#[test]
fn the_javelin_pierces_a_line_comes_back_and_answers_the_attack_button() {
    let t = Tuning::default();
    let mut w = World::new("test", 40.0);
    w.set_weapon(weapons::Id::Spear);
    let ids: Vec<usize> = [-3.0, -7.0, -12.0, -17.5]
        .iter()
        .map(|z| mob_with(&mut w, vec3(0.0, 0.0, *z), 900))
        .collect();
    idle(&mut w);
    let mut ev = w.step(
        &Actions {
            cast: true,
            ..Default::default()
        },
        &t,
    );
    ev.extend(wait(&mut w, &t, 0.4));
    assert!(w.javelin_out() && w.weapon_away(), "flying");
    let mana = w.player.ranged.mana;
    ev.extend(w.step(
        &Actions {
            cast: true,
            ..Default::default()
        },
        &t,
    ));
    assert!(
        ev.iter().any(|e| matches!(e, Event::CastDenied)),
        "no RB while it flies"
    );
    assert!(w.player.ranged.mana >= mana - 1e-3);
    ev.extend(wait(&mut w, &t, 2.0));
    assert!(!w.javelin_out(), "back in hand");
    assert!(ev.iter().any(|e| matches!(e, Event::JavelinBack { .. })));
    let hps: Vec<i32> = ids.iter().map(|&i| w.enemies[i].hp).collect();
    assert_eq!(
        hps,
        [880, 880, 880, 900],
        "the line once each, not beyond 15.6 m"
    );
    // 평타가 부른다.
    idle(&mut w);
    w.player.state = PState::Idle;
    w.step(
        &Actions {
            cast: true,
            ..Default::default()
        },
        &t,
    );
    wait(&mut w, &t, 0.25);
    assert!(w.javelin_out());
    let ev = w.step(
        &Actions {
            attack: true,
            ..Default::default()
        },
        &t,
    );
    assert!(!w.javelin_out(), "called home");
    assert!(ev.iter().any(|e| matches!(e, Event::JavelinBack { .. })));
    assert!(
        matches!(w.player.state, PState::Attack { .. }),
        "and the swing goes"
    );
}

/// 0274 ②b: 공중 RB(내려 쏘기)는 종류마다 꼴만 다르고 규칙은 그대로 — 한 점프에 한 번, 칸 하나. 신성한 빛은 공중에서도 날지 않고
/// 겨눈 땅에 꽂힌다.
#[test]
fn every_kind_keeps_one_air_rb_per_jump() {
    let t = Tuning::default();
    for id in weapons::IDS.into_iter().filter(|i| !i.is_ranged()) {
        let mut w = World::new("test", 30.0);
        w.set_weapon(id);
        w.spawn_mob(vec3(0.0, 0.0, -3.0));
        idle(&mut w);
        w.step(
            &Actions {
                jump: true,
                ..Default::default()
            },
            &t,
        );
        wait(&mut w, &t, 0.1);
        let cast = Actions {
            cast: true,
            ..Default::default()
        };
        let first = w.step(&cast, &t);
        let n = |ev: &[Event]| {
            ev.iter()
                .filter(|e| matches!(e, Event::Cast { .. }))
                .count()
        };
        let mut ev = first.clone();
        ev.extend(w.step(&Actions::default(), &t));
        assert_eq!(n(&ev), 1, "{id:?}: one air RB");
        assert!(
            (w.player.ranged.mana - (CAST_CHARGES - 1.0)).abs() < 0.05,
            "{id:?}"
        );
        let again = w.step(&cast, &t);
        assert_eq!(n(&again), 0, "{id:?}: once per jump");
        if id == weapons::Id::Hammer {
            assert_eq!(w.smites.len(), 1, "the light falls on the ground below");
        }
    }
}

/// 0274 ②b · 0173: 근접 무기의 RB(칸 셋이 시간으로 찬다)는 안전한 거리의 값 — 20초 동안 RB만 마구·모아 쏴도 그 무기 평타(박자)의
/// 일곱 할 밑. 사격 무기는 근접 가장 약한 것(창)의 일곱 할 밑(톡 · 끝까지 당김 — 활의 당기기, 쇠뇌의 모은 사격).
#[test]
fn every_rb_stays_under_seven_tenths_of_melee_at_a_safe_distance() {
    for id in weapons::IDS {
        let melee = if id.is_ranged() {
            super::ranged::dps(weapons::Id::Spear, 20.0, true, 1.2).0
        } else {
            super::ranged::dps(id, 20.0, true, 1.2).0
        };
        for hold in [false, true] {
            let rb = rb_dps(id, 20.0, 8.0, hold, false);
            assert!(rb > 0.0, "{id:?}");
            assert!(
                rb < melee * 0.7,
                "{id:?} hold {hold}: RB {rb:.1}/s vs melee {melee:.1}/s"
            );
        }
    }
}

/// 0218 → 0274 ②b: 미리보기는 진실하다 — 던지는 것(손도끼 · 창 · 돌)이 실제로 멈추는 수평 거리 = `shot_reach`(앞 3.5 m의 기둥).
#[test]
fn thrown_rb_previews_match_where_they_stop() {
    let t = Tuning::default();
    for id in [weapons::Id::Sword, weapons::Id::Spear, weapons::Id::Fists] {
        let mut w = World::new("test", 40.0);
        w.set_weapon(id);
        w.view_yaw = Some(0.0);
        let p0 = w.player.pos;
        w.obstacles.push(Aabb {
            min: vec3(p0.x - 1.0, 0.0, p0.z - 4.5),
            max: vec3(p0.x + 1.0, 3.0, p0.z - 3.5),
        });
        let face = w.player.facing_dir();
        let said = w.shot_reach(face, id.bolts().0, &t);
        w.step(
            &Actions {
                cast: true,
                ..Default::default()
            },
            &t,
        );
        let mut last = None;
        for _ in 0..120 {
            match w.bolts.first() {
                Some(b) if !b.back => last = Some(b.pos),
                _ if last.is_some() => break,
                _ => {}
            }
            w.step(&Actions::default(), &t);
        }
        let end = last.expect("it flew");
        let went = vec2(end.x - p0.x, end.z - p0.z).length();
        assert!(
            (3.2..3.7).contains(&said),
            "{id:?}: stops at the pillar: {said}"
        );
        assert!(
            (went - said).abs() < 0.8,
            "{id:?}: said {said}, went {went}"
        );
    }
}

/// 0274 ②b 쇠뇌(장전을 다룬다): 빈 탄창은 오래(표의 `reload`) 걸려 가득, 남은 탄이 있으면 쏘지 않고 멈춰 선 채 `wait`초 뒤부터
/// `each`초마다 한 발 — 넷을 다 채워도 빈 탄창의 장전보다 짧다. 걸으면 덧장전은 서지 않는다. 장전하는 동안에도 수류탄을 던지고
/// (던지는 동안에도 장전이 간다), 회피 장전은 한 발 그대로.
#[test]
fn the_crossbow_reloads_empty_slowly_and_tops_up_fast_standing_still() {
    let t = Tuning::default();
    let weapons::Rb::Shot { reload, mag, .. } = weapons::Id::Crossbow.def().rb else {
        panic!()
    };
    let Some(weapons::Rhythm::Crank { wait: w0, each }) = weapons::Id::Crossbow.rhythm() else {
        panic!()
    };
    assert!(
        w0 + each * (mag as f32 - 1.0) < reload * 0.75,
        "topping up beats emptying"
    );
    let fresh = || {
        let mut w = World::new("test", 30.0);
        w.set_weapon(weapons::Id::Crossbow);
        w
    };
    // 덧장전: 한 발 남긴 탄창 — 서 있으면 셋을 채운다.
    let mut w = fresh();
    w.player.ranged.ammo = 1;
    wait(&mut w, &t, w0 + each * 0.5);
    assert_eq!(w.player.ranged.ammo, 1, "waits a moment first");
    wait(&mut w, &t, each * (mag as f32 - 1.0) + 0.05);
    assert_eq!(w.player.ranged.ammo, mag, "one by one");
    // 걸으면 서지 않는다.
    let mut w = fresh();
    w.player.ranged.ammo = 1;
    for _ in 0..120 {
        w.step(
            &Actions {
                move_dir: vec2(1.0, 0.0),
                ..Default::default()
            },
            &t,
        );
    }
    assert_eq!(w.player.ranged.ammo, 1, "no top-up on the move");
    // 빈 탄창: `reload`초 — 그동안 수류탄을 던져도 장전은 이어 간다.
    let mut w = fresh();
    w.player.ranged.ammo = 0;
    w.player.ranged.reload = reload;
    w.player.ranged.grenades = 2.0;
    let ev = w.step(
        &Actions {
            attack: true,
            ..Default::default()
        },
        &t,
    );
    assert!(matches!(w.player.state, PState::Throw { .. }), "{ev:?}");
    wait(&mut w, &t, reload - 0.2);
    assert_eq!(w.player.ranged.ammo, 0, "still long");
    wait(&mut w, &t, 0.3);
    assert_eq!(w.player.ranged.ammo, mag);
    // 회피 장전은 한 발.
    let mut w = fresh();
    w.player.ranged.ammo = 0;
    w.player.ranged.reload = reload;
    w.step(
        &Actions {
            dodge: true,
            move_dir: vec2(1.0, 0.0),
            ..Default::default()
        },
        &t,
    );
    assert_eq!(w.player.ranged.ammo, 1, "a dodge loads one");
}

/// 0274 ②b 활(당기는 길이를 고른다): 누르면 화살을 메겨 당기고 떼면 쏜다 — 톡 = 보통 화살, 반쯤(`mid`초) = 더 빠르고 멀리·자세
/// 피해 둘 배, 끝까지(`cast.charge s`) = 꿰뚫고 오금(맞은 적이 느리게). 화살은 떼는 순간 하나(당기다 구르면 잃지 않는다), 끝까지 당긴
/// 것은 기력을 더 쓴다. 당기는 동안은 천천히 걷는다(쇠뇌를 모을 때보다 덜 느리다).
#[test]
fn the_bow_draws_in_stages_and_a_full_draw_hamstrings() {
    let t = Tuning::default();
    let Some(weapons::Rhythm::Draw { mid, mid_bolt, .. }) = weapons::Id::Bow.rhythm() else {
        panic!()
    };
    let (plain, full) = weapons::Id::Bow.bolts();
    assert!(mid_bolt.speed > plain.speed && mid_bolt.poise > plain.poise);
    assert!(full.pierce && full.lame > 0.0 && !mid_bolt.pierce);
    let shot = |secs: f32| {
        let mut w = World::new("test", 30.0);
        w.set_weapon(weapons::Id::Bow);
        let mag = w.player.ranged.ammo;
        let st = w.player.stamina;
        hold_release(&mut w, &t, secs);
        let b = w.bolts.first().copied().expect("an arrow");
        (b.def, mag - w.player.ranged.ammo, st - w.player.stamina)
    };
    let (tap, n0, s0) = shot(0.0);
    assert_eq!((tap, n0), (plain, 1), "a tap");
    let (half, n1, _) = shot(mid + 0.05);
    assert_eq!((half, n1), (mid_bolt, 1), "half drawn");
    let (whole, n2, s2) = shot(t[P::CastCharge] + 0.3);
    assert_eq!(
        (whole.pierce, whole.lame, n2),
        (true, full.lame, 1),
        "full draw"
    );
    assert!(
        s2 > s0 + 1.0,
        "a full draw costs more stamina: {s0} vs {s2}"
    );
    // 당기다 구르면 화살은 그대로.
    let mut w = World::new("test", 30.0);
    w.set_weapon(weapons::Id::Bow);
    let mag = w.player.ranged.ammo;
    w.step(
        &Actions {
            cast: true,
            cast_held: true,
            ..Default::default()
        },
        &t,
    );
    assert!(matches!(w.player.state, PState::CastCharge { .. }));
    w.step(
        &Actions {
            cast_held: true,
            dodge: true,
            move_dir: vec2(1.0, 0.0),
            ..Default::default()
        },
        &t,
    );
    assert!(matches!(w.player.state, PState::Dodge { .. }));
    assert_eq!(w.player.ranged.ammo, mag.min(w.mag_of(weapons::Id::Bow)));
    // 오금: 잡몹은 `HAMSTRING_SLOW`, 문지기는 `ROOT_BOSS_SLOW`로 느려지기만(휘청이지 않는다).
    for boss in [false, true] {
        let mut w = World::new("test", 30.0);
        w.set_weapon(weapons::Id::Bow);
        let i = if boss {
            w.spawn_boss(vec3(0.0, 0.0, -6.0));
            0
        } else {
            mob_with(&mut w, vec3(0.0, 0.0, -6.0), 900)
        };
        idle(&mut w);
        hold_release(&mut w, &t, t[P::CastCharge] + 0.05);
        wait(&mut w, &t, 0.3);
        let en = &w.enemies[i];
        assert!(en.marks.hamstring > 0.0, "boss {boss}");
        let want = if boss {
            crate::tuning::ROOT_BOSS_SLOW
        } else {
            crate::tuning::HAMSTRING_SLOW
        };
        assert!(
            (en.marks.slow - want).abs() < 1e-4,
            "boss {boss}: {}",
            en.marks.slow
        );
        if boss {
            assert!(
                !matches!(en.state, EState::Stagger { .. }),
                "a boss only slows"
            );
        }
        wait(&mut w, &t, full.lame);
        assert_eq!(w.enemies[i].marks.hamstring, 0.0, "it wears off");
    }
    // 당기며 걷기: 쇠뇌를 모을 때보다 빠르다(둘 다 서 있을 때보다는 느리다).
    let pace = |id: weapons::Id| {
        let mut w = World::new("test", 30.0);
        w.set_weapon(id);
        w.player.state = PState::CastCharge { t: 0.0 };
        let held = Actions {
            cast_held: true,
            move_dir: vec2(1.0, 0.0),
            ..Default::default()
        };
        let x0 = w.player.pos.x;
        for _ in 0..30 {
            w.step(&held, &t);
        }
        (w.player.pos.x - x0).abs()
    };
    let (bow, xbow) = (pace(weapons::Id::Bow), pace(weapons::Id::Crossbow));
    assert!(
        bow > xbow * 1.1 && bow > 0.1,
        "bow {bow} vs crossbow {xbow}"
    );
}

/// 0274 ②b 활의 화살 묶음: 작고(셋) 다시 채우기가 짧다 — 쇠뇌의 빈 탄창 장전의 반 밑.
#[test]
fn the_bow_carries_a_small_quiver_that_refills_quickly() {
    let (weapons::Rb::Shot { mag, reload, .. }, weapons::Rb::Shot { reload: xbow, .. }) =
        (weapons::Id::Bow.def().rb, weapons::Id::Crossbow.def().rb)
    else {
        panic!()
    };
    assert!(
        mag <= 3 && reload < xbow * 0.5,
        "{mag} arrows, {reload}s vs {xbow}s"
    );
}
