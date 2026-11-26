//! 쌍단검(0274 ③ — 도적): 빠른 4연타 · 가벼운 자세 피해 · 주특성 급소(등 뒤 · 휘청 · 기절 · 묶인 적에게 피해 배) · 강공격은 적을 꿰뚫고
//! 지나 등 뒤에 선다 · RB 투척 단검 셋(좁은 부채꼴) · 날카로움의 덤은 이제 쌍단검의 것.
use super::*;
use crate::weapons::{Id as W, ART_SNEAK, KNIFE, SNEAK_BACK_DEG};

/// 표: 4연타, 타마다 검보다 빠르고(시간 배 < 1) 자세 피해는 검의 반 이하, 닿는 거리는 맨손보다 조금 길고 검보다 짧다. 두 손이 다 차
/// 방패와 들지 않는다. 강공격은 곧은 길(창처럼 꿰뚫고 지나간다).
#[test]
fn the_daggers_are_quick_short_and_light_on_poise() {
    let t = Tuning::default();
    let d = W::Daggers.def();
    let s = W::Sword.def();
    assert_eq!(d.combo.len(), 4);
    let each =
        |w: &weapons::Weapon| w.combo.iter().map(|h| h.time).sum::<f32>() / w.combo.len() as f32;
    assert!(
        each(d) < each(s) * 0.85,
        "quicker hits: {} vs {}",
        each(d),
        each(s)
    );
    for (k, h) in d.combo.iter().enumerate() {
        let sw = s.combo[k.min(s.combo.len() - 1)];
        assert!(h.time < sw.time, "hit {k}: {}", h.time);
        assert!(h.poise <= sw.poise * 0.5 + 1e-4, "hit {k}");
    }
    let reach = |id: W| t[P::AttackRange] * id.def().combo[0].reach;
    assert!(reach(W::Fists) < reach(W::Daggers) && reach(W::Daggers) < reach(W::Sword));
    assert_eq!(W::Daggers.hand(), weapons::Hand::Two);
    assert!(d.heavy.unwrap().shape.lane_half().is_some());
    assert_eq!(W::Daggers.art().sneak, ART_SNEAK);
    assert_eq!(W::Sword.art().sneak, 1.0, "only the daggers");
}

/// 급소(`hurt_by` 한 곳): 쌍단검의 손(`Knack::sneak`)이 친 자리가 적의 등 뒤면 × 주특성의 배 — 앞 · 옆은 그대로. 휘청 · 기절 · 묶인
/// 적이면 앞에서도. 덤 없는 피해(새는 피 · 화상)에는 없다.
#[test]
fn the_sneak_attack_lands_from_behind_or_on_a_reeling_foe() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    let i = mob_with(&mut w, vec3(0.0, 0.0, -2.0), 500);
    w.enemies[i].state = EState::Recover { t: -1000.0 };
    w.enemies[i].facing = 0.0; // +z를 본다
    let knack = |from: Vec3| Knack {
        keen: false,
        sneak: Some((from, ART_SNEAK)),
        sure: false,
    };
    let en = &w.enemies[i];
    let front = vec3(0.0, 0.0, 0.0);
    let back = vec3(0.0, 0.0, -4.0);
    let side = vec3(2.0, 0.0, -2.0);
    let edge = SNEAK_BACK_DEG.to_radians();
    let just_in = vec3(-(edge + 0.05).sin(), 0.0, (edge + 0.05).cos()) * 2.0 + en.pos;
    let just_out = vec3(-(edge - 0.05).sin(), 0.0, (edge - 0.05).cos()) * 2.0 + en.pos;
    let sneak = crate::skills::scaled(100, ART_SNEAK);
    assert_eq!(hurt_by(en, 100, false, 0, knack(front), &t), 100);
    assert_eq!(hurt_by(en, 100, false, 0, knack(side), &t), 100, "aside");
    assert_eq!(hurt_by(en, 100, false, 0, knack(back), &t), sneak);
    assert_eq!(hurt_by(en, 100, false, 0, knack(just_in), &t), sneak);
    assert_eq!(hurt_by(en, 100, false, 0, knack(just_out), &t), 100);
    assert_eq!(hurt_by(en, 100, false, 0, Knack::NONE, &t), 100, "no hand");
    for reel in 0..3 {
        let mut w2 = World::new("test", 30.0);
        let j = mob_with(&mut w2, vec3(0.0, 0.0, -2.0), 500);
        let en = &mut w2.enemies[j];
        en.facing = 0.0;
        match reel {
            0 => en.state = EState::Stagger { t: 0.0, dur: 1.0 },
            1 => en.ail.stun = 1.0,
            _ => en.ail.bound = 1.0,
        }
        assert_eq!(
            hurt_by(&w2.enemies[j], 100, false, 0, knack(front), &t),
            sneak,
            "reeling {reel}"
        );
    }
    // 월드의 손: 쌍단검을 들면 급소가 실리고 다른 무기는 아니다.
    w.set_weapon(W::Daggers);
    assert_eq!(w.knack().sneak.map(|s| s.1), Some(ART_SNEAK));
    assert!(w.keen(), "the bleeding bonus is the daggers' now");
    w.set_weapon(W::Sword);
    assert_eq!(w.knack().sneak, None);
    assert!(!w.keen(), "the sword gave it up (0274 ③)");
}

/// 서 있는 적 하나를 `secs`초 동안 쳐서 준 피해(초당) — 적은 방랑자를 보거나(`behind` 거짓) 등을 돌린다. 마구 눌러/박자에 맞춰.
pub(super) fn foe_dps(id: W, behind: bool, secs: f32, on_beat: bool) -> f32 {
    let t = Tuning::default();
    let mut w = World::new("test", 40.0);
    w.set_weapon(id);
    let hp0 = 1_000_000;
    let at = vec3(0.0, 0.0, -1.2);
    let i = mob_with(&mut w, at, hp0);
    for k in 0..(secs / TICK_DT) as usize {
        idle(&mut w);
        let en = &mut w.enemies[i];
        en.pos = at;
        en.stance.poise = 0.0;
        en.facing = if behind { std::f32::consts::PI } else { 0.0 };
        w.player.pos = Vec3::ZERO;
        let press = if on_beat {
            let beat = w.player.beat.period;
            beat <= 0.0 || w.player.beat.since_swing >= beat - TICK_DT * 0.5
        } else {
            k % 2 == 0
        };
        w.step(
            &Actions {
                attack: press,
                ..Default::default()
            },
            &t,
        );
    }
    (hp0 - w.enemies[0].hp) as f32 / secs
}

/// 방랑자 앞 1.2 m에 부채꼴로 선 셋(−35° · 0° · +35°)에게 준 피해의 합(초당) — 붙은 떼.
pub(super) fn huddle_dps(id: W, secs: f32, on_beat: bool) -> f32 {
    let t = Tuning::default();
    let mut w = World::new("test", 40.0);
    w.set_weapon(id);
    let hp0 = 1_000_000;
    let at = |k: usize| {
        let a = (k as f32 - 1.0) * 35f32.to_radians();
        vec3(a.sin(), 0.0, -a.cos()) * 1.3
    };
    for k in 0..3 {
        mob_with(&mut w, at(k), hp0);
    }
    for k in 0..(secs / TICK_DT) as usize {
        idle(&mut w);
        for (j, e) in w.enemies.iter_mut().enumerate() {
            e.pos = at(j);
            e.stance.poise = 0.0;
            e.facing = 0.0;
        }
        w.player.pos = Vec3::ZERO;
        let press = if on_beat {
            let beat = w.player.beat.period;
            beat <= 0.0 || w.player.beat.since_swing >= beat - TICK_DT * 0.5
        } else {
            k % 2 == 0
        };
        w.step(
            &Actions {
                attack: press,
                ..Default::default()
            },
            &t,
        );
    }
    w.enemies.iter().map(|e| (hp0 - e.hp) as f32).sum::<f32>() / secs
}

/// 재는 도구: 무기마다 한 적(앞 · 등 뒤)과 붙은 셋에게 초당 피해. `cargo test dagger_probe -- --ignored --nocapture`
#[test]
#[ignore]
fn dagger_probe() {
    for id in [W::Sword, W::Hammer, W::Spear, W::Staff, W::Daggers] {
        for beat in [false, true] {
            println!(
                "{id:?} {}: front {:.1}/s · behind {:.1}/s · huddle of 3 {:.1}/s",
                if beat { "beat" } else { "mash" },
                foe_dps(id, false, 20.0, beat),
                foe_dps(id, true, 20.0, beat),
                huddle_dps(id, 20.0, beat)
            );
        }
    }
}

/// 초당 피해(20초 — 근접의 띠 안): 앞에서 한 적에게는 검과 같은 띠(90~115%), **등 뒤에서는 급소로 검의 1.4배 넘게**, 붙은 셋에게는 검보다
/// 덜(좁은 찌르기 — 떼에 약하다). 쌍단검의 값은 자리다: 혼자 있는 적 · 옆을 내준 적.
#[test]
fn the_daggers_are_strongest_at_a_lone_foes_back() {
    for beat in [false, true] {
        let sword = foe_dps(W::Sword, false, 20.0, beat);
        let front = foe_dps(W::Daggers, false, 20.0, beat);
        let back = foe_dps(W::Daggers, true, 20.0, beat);
        assert!(
            front > sword * 0.9 && front < sword * 1.15,
            "front (beat {beat}): sword {sword:.1} daggers {front:.1}"
        );
        assert!(back > sword * 1.4, "behind (beat {beat}): {back:.1}");
        assert_eq!(
            foe_dps(W::Sword, true, 20.0, beat),
            sword,
            "no sneak for the sword"
        );
        let (sw3, dg3) = (
            huddle_dps(W::Sword, 20.0, beat),
            huddle_dps(W::Daggers, 20.0, beat),
        );
        assert!(
            dg3 < sw3,
            "a huddle (beat {beat}): sword {sw3:.1} daggers {dg3:.1}"
        );
    }
}

/// 강공격 = 꿰뚫고 지나가기: 한 걸음 앞의 적을 뚫고 지나 그 등 뒤에 선다(몸이 막히지 않는다 — 창의 내달려 꿰뚫기와 같은 길) — 다음 타가 급소.
#[test]
fn the_heavy_lunges_through_to_the_foes_back() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    w.set_weapon(W::Daggers);
    w.player.facing = std::f32::consts::PI;
    let i = mob_with(&mut w, vec3(0.0, 0.0, -1.3), 500);
    w.enemies[i].facing = 0.0;
    idle(&mut w);
    let hold = Actions {
        attack: true,
        attack_held: true,
        ..Default::default()
    };
    w.step(&hold, &t);
    for _ in 0..40 {
        idle(&mut w);
        w.step(
            &Actions {
                attack_held: true,
                ..Default::default()
            },
            &t,
        );
    }
    for _ in 0..60 {
        idle(&mut w);
        w.step(&Actions::default(), &t);
    }
    assert!(w.enemies[i].hp < 500, "the lunge cuts");
    assert!(
        w.player.pos.z < w.enemies[i].pos.z,
        "through to its back: player {:?} foe {:?}",
        w.player.pos,
        w.enemies[i].pos
    );
    assert!(crate::world::sneak_open(&w.enemies[i], w.player.pos));
}

/// RB = 투척 단검 셋: 좁은 부채꼴(`KNIFE_SPREAD`°)로 셋이 곧게 난다 — 가까운 적에게는 셋이 다 박히고(날마다 날카로움 — 보통 적은 피를
/// 흘린다), 먼 적에게는 가운데 하나만.
#[test]
fn three_knives_fan_out_narrowly() {
    let t = Tuning::default();
    let throw = |dist: f32| {
        let mut w = World::new("test", 30.0);
        w.set_weapon(W::Daggers);
        w.player.facing = std::f32::consts::PI;
        // `dist`가 0이면 적 없이(날아가는 날을 센다) — 적은 그 쪽 멀리 비켜 둔다.
        let at = if dist > 0.0 {
            vec3(0.0, 0.0, -dist)
        } else {
            vec3(25.0, 0.0, 25.0)
        };
        let i = mob_with(&mut w, at, 500);
        idle(&mut w);
        let mut ev = w.step(
            &Actions {
                cast: true,
                ..Default::default()
            },
            &t,
        );
        let mut flying: Vec<Vec3> = Vec::new();
        for _ in 0..30 {
            idle(&mut w);
            ev.extend(w.step(&Actions::default(), &t));
            if !w.bolts.is_empty() {
                flying = w
                    .bolts
                    .iter()
                    .filter(|b| b.def.look == KNIFE)
                    .map(|b| b.dir)
                    .collect();
                break;
            }
        }
        for _ in 0..60 {
            idle(&mut w);
            ev.extend(w.step(&Actions::default(), &t));
        }
        let hits = ev.iter().filter(|e| matches!(e, Event::Hit { .. })).count();
        (flying, hits, w.enemies[i].ail.bleed > 0.0)
    };
    let (dirs, _, _) = throw(0.0);
    assert_eq!(dirs.len(), 3, "three knives");
    let spread = dirs[0].angle_between(dirs[2]).to_degrees();
    assert!(
        (spread - 2.0 * weapons::KNIFE_SPREAD).abs() < 1.0,
        "{spread}°"
    );
    let (_, hits, bled) = throw(1.6);
    assert_eq!(hits, 3, "all three up close");
    assert!(bled, "three knives bleed even a remnant");
    let (_, hits, _) = throw(6.5);
    assert_eq!(hits, 1, "only the middle one far off");
}

/// 보스(문지기)의 급소는 등 뒤에서만 — 자세가 깨져 휘청이는 동안은 이미 깨진 틈의 배(`poise.break damage x`)라 급소를 겹치지 않는다.
#[test]
fn a_boss_opens_its_weak_spot_only_from_behind() {
    let t = Tuning::default();
    let mut w = world_with_boss();
    let en = &mut w.enemies[0];
    en.facing = 0.0;
    en.state = EState::Stagger { t: 0.0, dur: 2.0 };
    let at = en.pos;
    let front = at + vec3(0.0, 0.0, 3.0);
    let back = at - vec3(0.0, 0.0, 3.0);
    assert!(
        !crate::world::sneak_open(&w.enemies[0], front),
        "reeling, but a boss"
    );
    assert!(crate::world::sneak_open(&w.enemies[0], back));
    let k = |from| Knack {
        keen: false,
        sneak: Some((from, ART_SNEAK)),
        sure: false,
    };
    let en = &w.enemies[0];
    assert!(hurt_by(en, 100, false, 0, k(back), &t) > hurt_by(en, 100, false, 0, k(front), &t));
}
