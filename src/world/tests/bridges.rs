//! 0215: 다리 칸 넷(두 갈래의 시너지)과 숫자 칸 셋의 고르기 II.
use super::*;
use crate::progression::{cell, link, Cell, Link, Tree};

fn tree(cells: &[Cell], links: &[Link], b_picks: &[Cell]) -> Tree {
    let mut t = Tree::with(cells);
    for &l in links {
        t.open_link(l);
    }
    for &c in b_picks {
        assert!(t.flip_cell(c), "{c:?} is an open number cell");
    }
    t
}

/// 앞의 내려치는 잡몹을 한 번 패링하고 히트스톱이 풀릴 때까지(튕겼으면 참).
fn parry_once(w: &mut World, t: &Tuning) -> bool {
    let en = &mut w.enemies[0];
    en.state = EState::Tele { t: en.tele_dur() };
    en.landed = false;
    w.player.state = PState::Parry { t: 0.0 };
    let mut hit = false;
    for _ in 0..60 {
        hit |= w
            .step(&Actions::default(), t)
            .iter()
            .any(|e| matches!(e, Event::Parry { .. }));
        if hit {
            break;
        }
    }
    while w.hitstop > 0.0 {
        w.step(&Actions::default(), t);
    }
    hit
}

/// 앞(-z)의 적을 이 스윙으로 친다 — 활성 첫 틱의 `apply_strike` 한 번.
fn strike_once(w: &mut World, t: &Tuning, state: PState) {
    w.player.facing = std::f32::consts::PI;
    w.player.state = state;
    let mut ev = Vec::new();
    w.apply_strike(t, &mut ev);
}

fn light_at_active(t: &Tuning, w: &World, swing: u32) -> PState {
    let (s0, _, _) = hit_times(t, w.player.weapon.def(), 0);
    PState::Attack {
        t: s0 + 1e-4,
        swing,
        dir: vec3(0.0, 0.0, -1.0),
        hit: 0,
    }
}

fn heavy_at_active(t: &Tuning, w: &World, swing: u32) -> PState {
    let (s0, _, _) = heavy_times(t, w.player.weapon.def());
    PState::Heavy {
        t: s0 + 1e-4,
        swing,
        dir: vec3(0.0, 0.0, -1.0),
        full: true,
        perfect: false,
    }
}

/// 받아치는 박: 패링 반격이 박자 타로 친다 — 박자의 이음에 센다(값은 그대로 치른다).
#[test]
fn riposte_beat_makes_the_riposte_a_beat_hit() {
    let t = Tuning::default();
    let after = |linked: bool| {
        let mut w = mob_swinging_in_front(&t);
        if linked {
            w.tree = tree(&[], &[Link::RiposteBeat], &[]);
        }
        assert!(parry_once(&mut w, &t));
        let s0 = w.player.stamina;
        w.step(
            &Actions {
                attack: true,
                ..Default::default()
            },
            &t,
        );
        assert!(matches!(w.player.state, PState::Heavy { full: true, .. }));
        (
            w.player.beat.swing_beat,
            w.player.beat.chain,
            w.player.stamina < s0,
        )
    };
    assert_eq!(after(false), (false, 0, true));
    assert_eq!(after(true), (true, 1, true), "a beat, and still paid for");
}

/// 디딘 받기: 패링하면 다음 회피가 기력 없이.
#[test]
fn parry_step_frees_the_next_dodge() {
    let t = Tuning::default();
    for linked in [false, true] {
        let mut w = mob_swinging_in_front(&t);
        if linked {
            w.tree = tree(&[], &[Link::ParryStep], &[]);
        }
        assert!(parry_once(&mut w, &t));
        assert_eq!(w.player.beat.free_dodge, linked);
    }
}

/// 끊는 박: 휘청이거나 드러난 적에게 박자 타의 자세 ×1.5 — 박자 밖이거나 서 있는 적은 그대로.
#[test]
fn breaking_beat_hits_a_staggered_foe_harder_on_the_beat() {
    let t = Tuning::default();
    let poise = |linked: bool, beat: bool, staggered: bool| {
        let mut w = World::new("test", 20.0);
        if linked {
            w.tree = tree(&[], &[Link::BreakingBeat], &[]);
        }
        let i = mob_with(&mut w, vec3(0.0, 0.0, -1.2), 990);
        idle(&mut w);
        if staggered {
            w.enemies[i].state = EState::Stagger { t: 0.0, dur: 5.0 };
        }
        w.player.beat.swing_beat = beat;
        let st = light_at_active(&t, &w, 3);
        strike_once(&mut w, &t, st);
        w.enemies[i].stance.poise
    };
    let plain = poise(false, true, true);
    assert!(plain > 0.0);
    assert!((poise(true, true, true) / plain - link::BREAK_POISE).abs() < 1e-3);
    assert!(
        (poise(true, false, true) - poise(false, false, true)).abs() < 1e-4,
        "off beat"
    );
    assert!(
        (poise(true, true, false) - poise(false, true, false)).abs() < 1e-4,
        "standing"
    );
}

/// 뿌리로 받기: 엉키거나 묶인 적에게 반격 타 ×1.5 — 반격이 아니거나 풀린 적은 그대로.
#[test]
fn rooted_riposte_hits_a_snared_foe_harder() {
    let t = Tuning::default();
    let dealt = |linked: bool, snared: bool, riposte: bool| {
        let mut w = World::new("test", 20.0);
        if linked {
            w.tree = tree(&[], &[Link::RootedRiposte], &[]);
        }
        let i = mob_with(&mut w, vec3(0.0, 0.0, -1.2), 990);
        idle(&mut w);
        if snared {
            w.enemies[i].marks.snare = 1.0;
        }
        w.player.parry.riposte_swing = if riposte { 5 } else { 0 };
        let st = heavy_at_active(&t, &w, 5);
        strike_once(&mut w, &t, st);
        990 - w.enemies[i].hp
    };
    let plain = dealt(false, true, true);
    assert_eq!(
        dealt(true, true, true),
        (plain as f32 * link::ROOTED_RIPOSTE).round() as i32
    );
    assert_eq!(dealt(true, false, true), dealt(false, false, true));
    assert_eq!(dealt(true, true, false), dealt(false, true, false));
}

/// 고른 호흡 II(모으는 걸음): 강타를 모으는 동안 ×1.4로 걷는다 — 그때는 딱 맞춘 강타의 기력은 돌려주지 않는다.
#[test]
fn gathering_stride_walks_faster_while_charging_instead_of_the_stamina_refund() {
    let t = Tuning::default();
    let speed = |b: bool| {
        let mut w = World::new("test", 20.0);
        w.tree = tree(
            &[Cell::EvenBreath],
            &[],
            if b { &[Cell::EvenBreath] } else { &[] },
        );
        w.player.state = PState::Charging { t: 0.0 };
        let a = Actions {
            attack_held: true,
            move_dir: vec2(0.0, 1.0),
            ..Default::default()
        };
        for _ in 0..40 {
            w.step(&a, &t);
            if let PState::Charging { .. } = w.player.state {
            } else {
                panic!("{:?}", w.player.state);
            }
        }
        vec3(w.player.vel.x, 0.0, w.player.vel.z).length()
    };
    let (a, b) = (speed(false), speed(true));
    assert!((b / a - cell::CHARGE_WALK).abs() < 0.02, "{a} → {b}");
    // II에서는 딱 맞춘 강타가 기력을 돌려주지 않는다.
    let mut w = World::new("test", 20.0);
    w.tree = tree(&[Cell::EvenBreath], &[], &[Cell::EvenBreath]);
    let h = w.player.weapon.def().heavy.unwrap();
    w.player.state = PState::Charging {
        t: t[P::HeavyCharge] * h.charge + 0.05,
    };
    w.player.stamina = 50.0;
    w.step(&Actions::default(), &t);
    assert!(matches!(
        w.player.state,
        PState::Heavy { perfect: true, .. }
    ));
    assert!((w.player.stamina - (50.0 - t[P::HeavyStamina])).abs() < 1e-3);
}

/// 받아내기 II(받아 넘기기): 패링 뒤 1초 동안 받는 피해 −50% — 자세 막대는 채우지 않는다. 지나면 그대로.
#[test]
fn turn_aside_halves_damage_for_a_moment_after_a_parry() {
    let t = Tuning::default();
    let mut w = mob_swinging_in_front(&t);
    w.tree = tree(&[Cell::Catch], &[], &[Cell::Catch]);
    assert!(parry_once(&mut w, &t));
    assert_eq!(w.enemies[0].stance.poise, 0.0, "II does not fill the bar");
    assert!(w.player.parry.deflect > 0.0);
    w.enemies.clear();
    // 이어 패링의 창은 닫는다 — 맞는 것을 본다.
    w.player.parry.chain_left = 0.0;
    w.player.state = PState::Idle;
    let hp = w.player.hp;
    hit_by_swing(&mut w, &t);
    assert_eq!(
        hp - w.player.hp,
        (HP_UNIT as f32 * cell::DEFLECT_TAKEN).round() as i32
    );
    // 1초가 지나면 그대로.
    let mut w = World::new("test", 20.0);
    w.tree = tree(&[Cell::Catch], &[], &[Cell::Catch]);
    w.player.parry.deflect = cell::DEFLECT_SECS;
    for _ in 0..(cell::DEFLECT_SECS / TICK_DT) as usize + 2 {
        w.step(&Actions::default(), &t);
    }
    let hp = w.player.hp;
    hit_by_swing(&mut w, &t);
    assert_eq!(hp - w.player.hp, HP_UNIT);
}

/// 멈추지 않는 걸음 II(가벼운 구르기): 회피 기력 ×0.8 — 달리기 값은 그대로.
#[test]
fn light_roll_makes_dodges_cheaper_instead_of_running() {
    let t = Tuning::default();
    let dodge_cost = |b: bool| {
        let mut w = World::new("test", 20.0);
        w.tree = tree(
            &[Cell::UnbrokenStride],
            &[],
            if b { &[Cell::UnbrokenStride] } else { &[] },
        );
        let s0 = w.player.stamina;
        w.step(
            &Actions {
                dodge: true,
                move_dir: vec2(1.0, 0.0),
                ..Default::default()
            },
            &t,
        );
        assert!(matches!(w.player.state, PState::Dodge { .. }));
        s0 - w.player.stamina
    };
    let base = t[P::StaminaDodge];
    assert!((dodge_cost(false) - base).abs() < 0.5);
    assert!((dodge_cost(true) - base * cell::LIGHT_ROLL).abs() < 0.5);
    // II의 달리기는 옛 값 그대로(멈추지 않는 걸음 I의 덤이 없다).
    let run = |b: bool| {
        let mut w = World::new("test", 20.0);
        w.tree = tree(
            &[Cell::UnbrokenStride],
            &[],
            if b { &[Cell::UnbrokenStride] } else { &[] },
        );
        w.player.running = true;
        w.player.stamina = 100.0;
        let a = Actions {
            dodge_held: true,
            move_dir: vec2(0.0, 1.0),
            ..Default::default()
        };
        for _ in 0..30 {
            w.step(&a, &t);
        }
        100.0 - w.player.stamina
    };
    assert!((run(false) / run(true) - cell::STRIDE_RUN).abs() < 0.02);
}
