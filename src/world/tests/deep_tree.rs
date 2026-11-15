//! 0214: 기술 판 넷째 줄 패시브 여섯과 다섯째 줄 맺음 셋의 동작.
use super::*;
use crate::progression::{cell, key, Cell, Key, Tree};

fn with(cells: &[Cell], k: Option<Key>) -> Tree {
    Tree {
        key: k,
        ..Tree::with(cells)
    }
}

/// 앞의 내려치는 잡몹을 한 번 패링한다(튕겼으면 참).
fn parry_once(w: &mut World, t: &Tuning) -> bool {
    let en = &mut w.enemies[0];
    en.state = EState::Tele { t: en.tele_dur() };
    en.landed = false;
    w.player.state = PState::Parry { t: 0.0 };
    for _ in 0..60 {
        if w.step(&Actions::default(), t)
            .iter()
            .any(|e| matches!(e, Event::Parry { .. }))
        {
            return true;
        }
    }
    false
}

/// 숨 고르기: 패링마다 기력 +20. 되찾는 살: 패링 셋마다 체력 한 눈금(가득이면 그대로).
#[test]
fn second_wind_and_mend_pay_for_parries() {
    let t = Tuning::default();
    let mut w = mob_swinging_in_front(&t);
    w.tree = with(&[Cell::SecondWind, Cell::Mend], None);
    w.player.hp = w.player.max_hp - 3 * HP_UNIT;
    let hp0 = w.player.hp;
    w.player.stamina = 10.0;
    w.player.stamina_idle = 0.0;
    assert!(parry_once(&mut w, &t));
    assert!(
        w.player.stamina >= 10.0 + cell::SECOND_WIND - 0.01,
        "{}",
        w.player.stamina
    );
    assert_eq!(w.player.hp, hp0, "one parry is not yet a notch");
    assert!(parry_once(&mut w, &t));
    let mut ev = Vec::new();
    let en = &mut w.enemies[0];
    en.state = EState::Tele { t: en.tele_dur() };
    en.landed = false;
    w.player.state = PState::Parry { t: 0.0 };
    for _ in 0..60 {
        ev.extend(w.step(&Actions::default(), &t));
        if ev.iter().any(|e| matches!(e, Event::Parry { .. })) {
            break;
        }
    }
    assert_eq!(w.player.hp, hp0 + HP_UNIT, "the third parry mends a notch");
    assert!(ev.iter().any(|e| matches!(e, Event::Mend { .. })));
    // 칸이 없으면 아무 일도 없다.
    let mut plain = mob_swinging_in_front(&t);
    plain.player.hp = plain.player.max_hp - 3 * HP_UNIT;
    let hp1 = plain.player.hp;
    for _ in 0..3 {
        assert!(parry_once(&mut plain, &t));
    }
    assert_eq!(plain.player.hp, hp1);
}

/// 이어지는 숨: 박자 셋을 이은 뒤의 회피 하나는 기력을 쓰지 않는다 — 한 번뿐.
#[test]
fn held_breath_makes_one_dodge_free() {
    let t = Tuning::default();
    let mut w = World::new("test", 20.0);
    w.tree = with(&[Cell::HeldBreath], None);
    w.player.beat.free_dodge = true;
    let s0 = w.player.stamina;
    let dodge = Actions {
        dodge: true,
        move_dir: vec2(1.0, 0.0),
        ..Default::default()
    };
    w.step(&dodge, &t);
    assert!(matches!(w.player.state, PState::Dodge { .. }));
    assert!(
        (w.player.stamina - s0).abs() < 0.5,
        "free: {s0} → {}",
        w.player.stamina
    );
    assert!(!w.player.beat.free_dodge, "used up");
    for _ in 0..60 {
        w.step(&Actions::default(), &t);
    }
    let s1 = w.player.stamina;
    w.step(&dodge, &t);
    assert!(w.player.stamina < s1 - 1.0, "the next one costs again");
}

/// 다잡은 박: 박자 타가 맞으면 게이지가 반 번 더 찬다.
#[test]
fn gathering_beat_fills_the_gauge_more_on_beat_hits() {
    let t = Tuning::default();
    let gain = |cell: bool, beat: bool| {
        let mut w = World::new("test", 20.0);
        if cell {
            w.tree = with(&[Cell::GatheringBeat], None);
        }
        w.player.skill.ult_rank = 1;
        w.player.skill.gauge = 0.0;
        w.player.connected = 7;
        w.player.beat.swing_beat = beat;
        let mut ev = Vec::new();
        w.skill_books(&t, &mut ev);
        w.player.skill.gauge
    };
    let hit = t[P::SkillGaugeHit];
    assert!((gain(false, true) - hit).abs() < 1e-5);
    assert!((gain(true, false) - hit).abs() < 1e-5, "only beat hits");
    assert!((gain(true, true) - hit * (1.0 + cell::GATHERING_BEAT)).abs() < 1e-5);
}

/// 디딘 발: 스킬의 시동 동안 맞아도 끊기지 않는다(피해는 받는다). 칸이 없으면 끊긴다.
#[test]
fn planted_feet_keep_a_skill_going_through_a_hit() {
    let t = Tuning::default();
    let after_hit = |cell: bool| {
        let mut w = World::new("test", 20.0);
        if cell {
            w.tree = with(&[Cell::Planted], None);
        }
        w.step(
            &Actions {
                skill1: true,
                ..Default::default()
            },
            &t,
        );
        assert!(matches!(w.player.state, PState::Skill { .. }));
        let hp = w.player.hp;
        hit_by_swing(&mut w, &t);
        (w.player.state, w.player.hp < hp)
    };
    let (st, hurt) = after_hit(true);
    assert!(matches!(st, PState::Skill { .. }) && hurt, "{st:?}");
    let (st, hurt) = after_hit(false);
    assert!(!matches!(st, PState::Skill { .. }) && hurt, "{st:?}");
}

/// 엉킨 뿌리: 스킬에 맞은 잡몹은 한동안 느리게 걷는다 — 보스는 아니다.
#[test]
fn snaring_roots_slow_what_a_skill_hits_but_not_a_boss() {
    let t = Tuning::default();
    let mut w = World::new("test", 20.0);
    w.tree = with(&[Cell::Snare], None);
    w.player.facing = std::f32::consts::PI;
    mob_with(&mut w, vec3(0.0, 0.0, -1.5), 990);
    w.spawn_boss(vec3(1.5, 0.0, 0.0));
    idle(&mut w);
    w.step(
        &Actions {
            skill1: true,
            ..Default::default()
        },
        &t,
    );
    for _ in 0..40 {
        w.step(&Actions::default(), &t);
        idle(&mut w);
    }
    assert!(w.enemies[0].snare > 0.0, "the mob is snared");
    assert!((w.enemies[0].slow - cell::SNARE_SLOW).abs() < 1e-5);
    assert_eq!(w.enemies[1].snare, 0.0, "a boss is not");
    for _ in 0..(cell::SNARE_SECS / TICK_DT) as usize + 2 {
        w.step(&Actions::default(), &t);
        idle(&mut w);
    }
    assert_eq!(w.enemies[0].slow, 1.0, "it wears off");
}

/// 박자의 사람: 박자 밖의 평타는 ×0.75 — 맺음이 없으면 그대로.
#[test]
fn one_who_keeps_time_weakens_off_beat_swings() {
    let t = Tuning::default();
    let first_hit = |k: Option<Key>| {
        let mut w = World::new("test", 20.0);
        w.tree = with(&[], k);
        w.player.facing = std::f32::consts::PI;
        let i = mob_with(&mut w, vec3(0.0, 0.0, -1.2), 990);
        idle(&mut w);
        press(
            &t,
            &mut w,
            Actions {
                attack: true,
                ..Default::default()
            },
            40,
        );
        990 - w.enemies[i].hp
    };
    let plain = first_hit(None);
    let keyed = first_hit(Some(Key::Measure));
    assert!(plain > 0);
    assert_eq!(
        keyed,
        (plain as f32 * key::MEASURE_OFF).round() as i32,
        "an off-beat first swing"
    );
    // 다른 맺음은 평타를 건드리지 않는다.
    assert_eq!(first_hit(Some(Key::Edge)), plain);
}

/// 벼린 손: 패링 창이 ×0.75로 좁아지고, 패링이 자세 막대를 25% 더 채운다(받아내기와 함께면 두 배).
#[test]
fn honed_hand_narrows_the_window_and_breaks_more_posture() {
    let t = Tuning::default();
    let mut w = World::new("test", 20.0);
    let open = w.tuned(&t)[P::ParryWindow];
    w.tree = with(&[], Some(Key::Edge));
    assert!((w.tuned(&t)[P::ParryWindow] - open * key::EDGE_WINDOW).abs() < 1e-5);
    let poise = |cells: &[Cell], k: Option<Key>| {
        let mut w = mob_swinging_in_front(&t);
        w.tree = with(cells, k);
        assert!(parry_once(&mut w, &t));
        w.enemies[0].poise / poise_max(&w.enemies[0], &t)
    };
    assert_eq!(poise(&[], None), 0.0);
    assert!((poise(&[], Some(Key::Edge)) - cell::CATCH_POISE).abs() < 1e-3);
    assert!((poise(&[Cell::Catch], Some(Key::Edge)) - 2.0 * cell::CATCH_POISE).abs() < 1e-3);
}

/// 뿌리내린 몸: 서 있으면 덜 다치고 밀리지 않는다 — 움직이는 중이면 그대로. 회피는 ×0.65.
#[test]
fn a_rooted_body_takes_less_when_standing_and_dodges_short() {
    let t = Tuning::default();
    let hurt = |still: f32| {
        let mut w = World::new("test", 20.0);
        w.tree = with(&[], Some(Key::Rooted));
        w.player.hurt.still = still;
        let hp = w.player.hp;
        hit_by_swing(&mut w, &t);
        (hp - w.player.hp, w.player.vel.length())
    };
    let (rooted, knock) = hurt(1.0);
    assert_eq!(rooted, (HP_UNIT as f32 * key::ROOTED_TAKEN).round() as i32);
    assert!(knock < 1e-3, "not pushed");
    let (moving, knock) = hurt(0.0);
    assert_eq!(moving, HP_UNIT);
    assert!(knock > 0.1);
    let mut w = World::new("test", 20.0);
    let far = w.tuned(&t)[P::DodgeDist];
    w.tree = with(&[], Some(Key::Rooted));
    assert!((w.tuned(&t)[P::DodgeDist] - far * key::ROOTED_DODGE).abs() < 1e-5);
    // 서 있는 시간은 쌓이고, 걸으면 처음부터.
    for _ in 0..30 {
        w.step(&Actions::default(), &t);
    }
    assert!(w.player.hurt.still >= key::ROOTED_STILL);
    for _ in 0..10 {
        w.step(&up(), &t);
    }
    assert!(w.player.hurt.still < key::ROOTED_STILL);
}
