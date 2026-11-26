//! 대검(0274 ④ — 광전사): 끊김 없는 넓은 휩쓸기 다섯(앞의 셋까지 쪼갠다) · 휘두르는 동안 덜 휘청인다(받는 피해는 그대로) · 주특성 분노
//! (둘레의 적 · 잃은 체력만큼 평타가 빨라진다 — 상한) · 빈 손의 결 "남김없이 휘두른다"(분노가 두 배로 차고 가득이면 평타가 끝나지
//! 않는다) · RB 땅 내려찍기(땅을 따라 앞으로 달리는 짧은 충격파 — 길 위를 모두, 균열).
use super::*;
use crate::weapons::{
    Id as W, ART_FURY, BARE_FURY_RATE, FURY_CROWD, GREATSWORD_CLEAVE, GREATSWORD_FLINCH, QUAKE,
};

fn wait(w: &mut World, t: &Tuning, secs: f32) -> Vec<Event> {
    let mut ev = Vec::new();
    for _ in 0..(secs / TICK_DT).round() as usize {
        idle(w);
        ev.extend(w.step(&Actions::default(), t));
    }
    ev
}

/// 표: 평타 다섯(한손검 셋보다 길다) · 넓다(앞의 넷은 검의 1.5배 넘는 각) · 앞의 셋까지 · 한 타는 검보다 느리고 멀리 닿는다 · 두 손
/// (방패와 들지 않는다) · 기력은 검보다 많이 · 주특성 분노만(급소 · 물리력은 없다).
#[test]
fn the_greatsword_sweeps_wide_long_and_cleaves_three() {
    let t = Tuning::default();
    let g = W::Greatsword.def();
    let s = W::Sword.def();
    assert_eq!(g.combo.len(), 5);
    assert!(g.combo.len() > s.combo.len());
    for h in &g.combo[..4] {
        assert!(h.arc >= 1.5 * s.combo[0].arc, "{}", h.anim);
    }
    for h in g.combo.iter().chain(g.dash.iter()) {
        assert_eq!(h.cleave, GREATSWORD_CLEAVE, "{}", h.anim);
        assert!(h.time > 1.0 && h.reach > 1.0, "{}", h.anim);
    }
    assert!(
        s.combo.iter().all(|h| h.cleave == 0),
        "the sword cuts all it reaches"
    );
    let reach = |id: W| t[P::AttackRange] * id.def().combo[0].reach;
    assert!(reach(W::Greatsword) > reach(W::Sword));
    assert_eq!(W::Greatsword.hand(), weapons::Hand::Two);
    assert!(g.stamina > s.stamina);
    let art = W::Greatsword.art();
    assert_eq!((art.fury, art.force, art.sneak), (ART_FURY, 1.0, 1.0));
    assert_eq!(W::Sword.art().fury, 1.0, "only the greatsword");
}

/// 쪼개기: 앞의 부채꼴에 다섯이 서 있어도 한 휘두르기는 가까운 셋만 벤다 — 먼 둘은 다음 휘두르기에(다른 무기는 상한이 없다 — 표의
/// `cleave` 0, 위 시험).
#[test]
fn a_sweep_cuts_the_three_nearest_and_no_more() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    w.set_weapon(W::Greatsword);
    w.player.facing = std::f32::consts::PI;
    for k in 0..5 {
        let a = (k as f32 - 2.0) * 20f32.to_radians();
        let d = 1.0 + 0.25 * k as f32;
        mob_with(&mut w, vec3(a.sin() * d, 0.0, -a.cos() * d), 500);
    }
    idle(&mut w);
    w.step(
        &Actions {
            attack: true,
            ..Default::default()
        },
        &t,
    );
    wait(&mut w, &t, 0.4);
    let cut: Vec<bool> = w.enemies.iter().map(|e| e.hp < 500).collect();
    assert_eq!(cut, [true, true, true, false, false], "the three nearest");
}

/// 분노: 둘레의 깨어 있는 적 수 ÷ `FURY_CROWD` + 잃은 체력의 몫(1에서 멈춘다) — 평타의 빠르기가 그만큼(상한 `ART_FURY`). 한 휘두르기
/// 동안은 처음의 빠르기에 머문다. 다른 무기는 없다.
#[test]
fn fury_speeds_the_combo_with_foes_around_and_wounds_up_to_a_cap() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    w.set_weapon(W::Greatsword);
    assert_eq!(w.fury(), 0.0, "alone and whole");
    w.step(&Actions::default(), &t);
    assert_eq!(w.swing_rate(), 1.0);
    mob_with(&mut w, vec3(0.0, 0.0, -1.5), 500);
    mob_with(&mut w, vec3(1.5, 0.0, 0.0), 500);
    mob_with(&mut w, vec3(0.0, 0.0, -9.0), 500);
    idle(&mut w);
    assert!(
        (w.fury() - 2.0 / FURY_CROWD).abs() < 1e-4,
        "two near, one far"
    );
    w.player.hp = w.player.max_hp / 2;
    assert!((w.fury() - (2.0 / FURY_CROWD + 0.5)).abs() < 1e-4);
    w.player.hp = 1;
    assert_eq!(w.fury(), 1.0, "capped");
    w.step(&Actions::default(), &t);
    assert!((w.swing_rate() - ART_FURY).abs() < 1e-4);
    // 빠른 휘두르기는 더 일찍 끝난다(같은 무기 · 같은 적 — 한 휘두르기의 틱 수).
    let swing_ticks = |hp: i32| {
        let mut w = World::new("test", 30.0);
        w.set_weapon(W::Greatsword);
        w.player.facing = std::f32::consts::PI;
        w.player.hp = hp;
        mob_with(&mut w, vec3(0.0, 0.0, -1.3), 500);
        idle(&mut w);
        let mut n = 0;
        w.step(
            &Actions {
                attack: true,
                ..Default::default()
            },
            &t,
        );
        while matches!(w.player.state, PState::Attack { .. }) && n < 120 {
            idle(&mut w);
            w.step(&Actions::default(), &t);
            n += 1;
        }
        n
    };
    let whole = swing_ticks(w.player.max_hp);
    let wounded = swing_ticks(1);
    // 하나 곁(분노 0.25 — ×1.05)과 체력이 바닥(가득 — ×1.2).
    let want = (1.05 / ART_FURY) * whole as f32;
    assert!(
        (wounded as f32 - want).abs() <= 1.5,
        "wounded swings end sooner: {wounded} vs {whole}"
    );
    // 한 휘두르기 안에서는 머문다: 휘두르는 도중에 적이 몰려와도 빠르기가 그대로.
    let mut w = World::new("test", 30.0);
    w.set_weapon(W::Greatsword);
    w.step(
        &Actions {
            attack: true,
            ..Default::default()
        },
        &t,
    );
    w.step(&Actions::default(), &t);
    let rate = w.swing_rate();
    for k in 0..4 {
        mob_with(&mut w, vec3(k as f32 - 1.5, 0.0, 2.0), 500);
    }
    idle(&mut w);
    w.step(&Actions::default(), &t);
    assert!(matches!(w.player.state, PState::Attack { .. }));
    assert_eq!(w.swing_rate(), rate, "latched for the swing");
    // 다른 무기는 분노가 없다.
    let mut w = World::new("test", 30.0);
    w.set_weapon(W::Sword);
    w.player.hp = 1;
    mob_with(&mut w, vec3(0.0, 0.0, -1.5), 500);
    assert_eq!(w.fury(), 0.0);
}

/// 휘두르는 동안 덜 휘청인다(덧옷의 `flinch`와 같은 손잡이): 대검을 모으는 동안 맞으면 휘청이는 시간이 `GREATSWORD_FLINCH`배 —
/// 받는 피해는 검과 같다(0095 원칙 2). 서 있을 때는 그대로.
#[test]
fn swinging_the_greatsword_flinches_less_but_takes_the_same_blow() {
    let t = Tuning::default();
    let hit = |id: W, charging: bool| {
        let mut w = mob_swinging_in_front(&t);
        w.set_weapon(id);
        if charging {
            w.player.state = PState::Charging { t: 0.0 };
        }
        let hp0 = w.player.hp;
        let held = Actions {
            attack_held: charging,
            ..Default::default()
        };
        for _ in 0..60 {
            w.step(&held, &t);
            if let PState::Hurt { dur, .. } = w.player.state {
                return (hp0 - w.player.hp, dur);
            }
        }
        panic!("{id:?} never hit");
    };
    let (sword_taken, sword_dur) = hit(W::Sword, true);
    let (gs_taken, gs_dur) = hit(W::Greatsword, true);
    assert_eq!(gs_taken, sword_taken, "the same damage");
    assert!(
        (gs_dur - sword_dur * GREATSWORD_FLINCH).abs() < 1e-4,
        "{gs_dur} vs {sword_dur}"
    );
    let (_, standing) = hit(W::Greatsword, false);
    assert!((standing - sword_dur).abs() < 1e-4, "not while standing");
}

/// 빈 손의 결 "남김없이 휘두른다": 분노가 두 배로 차고(`BARE_FURY_RATE`), 가득이면 끝 타 다음에 둘째 타로 이어진다(평타가 끝나지 않는다).
/// 보조무기를 메면 둘 다 없다.
#[test]
fn nothing_held_back_fills_fury_twice_as_fast_and_never_ends_the_combo() {
    let mut w = World::new("test", 30.0);
    w.set_weapon(W::Greatsword);
    w.set_sub(None);
    w.set_shield(None);
    w.set_open_hand(true);
    assert_eq!(w.bare(), Some(weapons::Bare::Reckless));
    mob_with(&mut w, vec3(0.0, 0.0, -1.5), 500);
    idle(&mut w);
    assert!((w.fury() - BARE_FURY_RATE / FURY_CROWD).abs() < 1e-4);
    let last = (W::Greatsword.def().combo.len() - 1) as u8;
    assert_eq!(w.next_hit(last), None, "not full yet");
    w.player.hp = w.player.max_hp / 2;
    assert_eq!(w.fury(), 1.0);
    assert_eq!(w.next_hit(last), Some(1), "back to the second sweep");
    assert_eq!(w.next_hit(0), Some(1), "the rest as the table");
    assert_eq!(w.next_hit(weapons::DASH), Some(1));
    w.set_sub(Some(W::Sword));
    assert_eq!(w.bare(), None);
    assert!((w.fury() - (1.0 / FURY_CROWD + 0.5)).abs() < 1e-4);
    assert_eq!(w.next_hit(last), None, "an off-hand weapon ends it");
    // 끝나지 않는 평타: 가득 찬 채 쥐고 누르면 다섯을 넘어 이어진다.
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    w.set_weapon(W::Greatsword);
    w.set_open_hand(true);
    w.player.facing = std::f32::consts::PI;
    w.player.hp = 1;
    mob_with(&mut w, vec3(0.0, 0.0, -1.3), 100_000);
    let mut hits = Vec::new();
    for k in 0..600 {
        idle(&mut w);
        w.player.stamina = t[P::StaminaMax];
        w.step(
            &Actions {
                attack: k % 2 == 0,
                ..Default::default()
            },
            &t,
        );
        if let PState::Attack { hit, .. } = w.player.state {
            if hits.last() != Some(&hit) {
                hits.push(hit);
            }
        }
    }
    let wraps = hits
        .windows(2)
        .filter(|p| p[0] == last && p[1] == 1)
        .count();
    assert!(
        wraps >= 2,
        "the fifth sweep runs on into the second: {hits:?}"
    );
    assert!(!hits.windows(2).any(|p| p[0] == last && p[1] == 0));
}

/// RB = 땅 내려찍기: 땅을 따라 앞으로 달리는 충격파(곧게 나는 것이 아니다 — 마법 탄처럼 땅을 탄다) · 길 위의 적을 모두 친다(꿰뚫음) ·
/// 균열을 쌓는다 · 짧다(손도끼만큼 — 10 m 밖에는 닿지 않는다).
#[test]
fn the_ground_slam_runs_along_the_ground_through_a_line() {
    let t = Tuning::default();
    let (bolt, charged) = W::Greatsword.bolts();
    assert_eq!(bolt.look, QUAKE);
    assert!(!bolt.straight() && bolt.pierce && charged.pierce);
    assert_eq!(bolt.dose.0, crate::status::Kind::Rift);
    let mut w = World::new("test", 30.0);
    w.set_weapon(W::Greatsword);
    w.player.facing = std::f32::consts::PI;
    for z in [2.0, 4.0, 6.0, 11.0] {
        mob_with(&mut w, vec3(0.0, 0.0, -z), 500);
    }
    idle(&mut w);
    w.step(
        &Actions {
            cast: true,
            ..Default::default()
        },
        &t,
    );
    wait(&mut w, &t, 1.5);
    let cut: Vec<bool> = w.enemies.iter().map(|e| e.hp < 500).collect();
    assert_eq!(cut, [true, true, true, false], "the line, short");
    assert!(
        w.enemies[0].ail.gauge[crate::status::Kind::Rift.slot().unwrap()] > 0.0
            || w.enemies[0].ail.rift > 0.0
    );
}

/// 초당 피해(20초): 한 적에게는 검과 비슷하거나 조금 아래(80~105%), 앞에 붙은 셋에게는 검보다 크게(1.4배 넘게 — 떼에서 센다).
#[test]
fn the_greatsword_is_a_sword_on_one_foe_and_more_on_a_crowd() {
    for beat in [false, true] {
        let sword = super::daggers::foe_dps(W::Sword, false, 20.0, beat);
        let gs = super::daggers::foe_dps(W::Greatsword, false, 20.0, beat);
        assert!(
            gs > sword * 0.8 && gs < sword * 1.05,
            "one foe (beat {beat}): sword {sword:.1} greatsword {gs:.1}"
        );
        let (sw3, gs3) = (
            super::daggers::huddle_dps(W::Sword, 20.0, beat),
            super::daggers::huddle_dps(W::Greatsword, 20.0, beat),
        );
        assert!(
            gs3 > sw3 * 1.4,
            "a huddle (beat {beat}): sword {sw3:.1} greatsword {gs3:.1}"
        );
    }
}

/// 재는 도구: `cargo test greatsword_probe -- --ignored --nocapture`
#[test]
#[ignore]
fn greatsword_probe() {
    for id in [W::Sword, W::Hammer, W::Spear, W::Daggers, W::Greatsword] {
        for beat in [false, true] {
            println!(
                "{id:?} {}: one {:.1}/s · huddle of 3 {:.1}/s",
                if beat { "beat" } else { "mash" },
                super::daggers::foe_dps(id, false, 20.0, beat),
                super::daggers::huddle_dps(id, 20.0, beat)
            );
        }
    }
}
