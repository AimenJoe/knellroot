//! 몽크(0286 — 맨손): 보조무기 슬롯이 잠긴 종류 · 주특성 흐름 · 빈 손의 결 열린 손.
use super::*;
use crate::weapons::{self, Bare, Class, Id as W, ART_FLOW, FLOW_FADE, FLOW_HOLD, FLOW_STEP};

/// 앞(-z) 1.2 m에 선 맞기만 하는 적(체력 넉넉히).
fn monk_with_a_dummy() -> (World, Tuning) {
    let mut w = World::new("test", 30.0);
    w.set_weapon(W::Fists);
    w.player.facing = std::f32::consts::PI;
    let i = w.spawn(crate::species::Kind::Husk, vec3(0.0, 0.0, -1.2));
    let en = &mut w.enemies[i];
    en.hp = 9000;
    en.max_hp = 9000;
    en.state = EState::Recover { t: -1000.0 };
    (w, Tuning::default())
}

fn hold_still(w: &mut World) {
    for e in &mut w.enemies {
        if e.active() && !matches!(e.state, EState::Stagger { .. }) {
            e.state = EState::Recover { t: -1000.0 };
        }
    }
}

/// 평타를 `n`번 눌러 맞힌다(누를 때마다 그 휘두르기가 끝날 때까지).
fn punch(w: &mut World, t: &Tuning, n: usize) {
    for _ in 0..n {
        w.step(
            &Actions {
                attack: true,
                ..Default::default()
            },
            t,
        );
        for _ in 0..24 {
            w.step(&Actions::default(), t);
            hold_still(w);
        }
        // 밀려난 적은 다시 손 앞으로(셋째 타의 넉백).
        let at = w.player.pos + vec3(0.0, 0.0, -1.2);
        for e in &mut w.enemies {
            e.pos = at;
            e.prev = at;
        }
    }
}

/// 맨손은 몽크의 무기다 — 종류 표의 한 줄(타격 · 충격과 날카로움 · 흐름 · 열린 손 · 돌), 양손 계열(두 주먹)이라 방패와 들지 않고,
/// 주무기만 든다(`Quirks::one_slot`). 다른 종류는 그대로 보조무기로 멘다.
#[test]
fn fists_are_the_monks_weapon() {
    assert_eq!(W::Fists.class(), Some(Class::Monk));
    let d = Class::Monk.def();
    assert_eq!(
        d.pair,
        [crate::status::Kind::Shock, crate::status::Kind::Sharp]
    );
    assert_eq!(d.great, crate::skills::Ult::Palms);
    assert_eq!(d.bare, Bare::OpenHand);
    assert_eq!(d.toss, Some(weapons::STONE_TOSS));
    assert!(d.quirks.one_slot && d.quirks.keen);
    assert!(!W::Fists.takes_shield() && !W::Fists.slings());
    for c in weapons::CLASSES.iter().filter(|c| c.class != Class::Monk) {
        assert!(
            !c.quirks.one_slot,
            "{:?}: other classes are unchanged",
            c.class
        );
    }
}

/// 몽크의 보조무기 슬롯은 아무것도 들지 않는다(월드에서도): 보조무기를 메려 해도 비고, 방패를 들어도 막지도 던지지도 않으며 쥔 꼴도
/// 몽크 그대로다. 바꾸기 버튼은 바꿀 것이 없어 아무 일도 없다. 빈 손의 결(열린 손)은 앱이 슬롯을 말하지 않아도 늘 켜진다.
#[test]
fn the_monks_off_hand_slot_holds_nothing() {
    let (mut w, t) = monk_with_a_dummy();
    w.set_sub(Some(W::Sword));
    assert_eq!(w.player.sub, None, "no off-hand weapon");
    w.set_shield(Some(crate::shield::Id::Pilgrim));
    assert!(!w.shield_guards() && !w.can_block() && !w.shield_throws());
    assert_eq!(w.grip(), weapons::Grip::Class(Class::Monk));
    assert_eq!(w.bare(), Some(Bare::OpenHand), "always open");
    let ev = w.step(
        &Actions {
            swap: true,
            ..Default::default()
        },
        &t,
    );
    assert!(!ev.iter().any(|e| matches!(e, Event::WeaponSwap { .. })));
    assert_eq!(w.player.weapon, W::Fists);
    // 맨손은 다른 무기의 보조무기도 되지 않는다.
    let mut s = World::new("test", 30.0);
    s.set_weapon(W::Sword);
    s.set_sub(Some(W::Fists));
    assert_eq!(s.player.sub, None, "fists are never an off-hand weapon");
    s.set_sub(Some(W::Daggers));
    assert_eq!(
        s.player.sub,
        Some(W::Daggers),
        "other classes are unchanged"
    );
}

/// 흐름(주특성): 맞힌 손의 타마다 한 칸씩(`FLOW_STEP`) 차 네 대에 가득 — 한 휘두르기는 여러 틱 · 여러 적이어도 한 번. 흐름만큼 평타가
/// 빨라진다(가득이면 ×`ART_FLOW`). 맞으면 끊기고, 손이 닿지 않으면 `FLOW_HOLD`초 뒤부터 빠진다. 다른 종류에는 흐름이 없다.
#[test]
fn landed_bare_hand_hits_build_the_flow_and_quicken_the_swings() {
    let (mut w, t) = monk_with_a_dummy();
    assert_eq!(w.flow(), 0.0);
    punch(&mut w, &t, 1);
    assert!(
        (w.flow() - FLOW_STEP).abs() < 1e-4,
        "one swing: {}",
        w.flow()
    );
    punch(&mut w, &t, 3);
    assert!((w.flow() - 1.0).abs() < 1e-4, "four swings fill it");
    w.player.state = PState::Idle;
    w.step(&Actions::default(), &t);
    assert!(
        (w.swing_rate() - ART_FLOW).abs() < 1e-3,
        "{}",
        w.swing_rate()
    );
    // 손이 닿지 않으면 잠시 머물다 빠진다.
    let ticks = |s: f32| (s / TICK_DT).round() as usize;
    for _ in 0..ticks(FLOW_HOLD * 0.8) {
        w.step(&Actions::default(), &t);
        hold_still(&mut w);
    }
    assert!((w.flow() - 1.0).abs() < 1e-4, "it holds a moment");
    for _ in 0..ticks(FLOW_HOLD * 0.2 + 0.5 / FLOW_FADE) {
        w.step(&Actions::default(), &t);
        hold_still(&mut w);
    }
    assert!(w.flow() < 0.6, "then it fades: {}", w.flow());
    // 다른 종류: 맞혀도 흐름이 없고 빠르기는 그대로.
    let mut s = World::new("test", 30.0);
    s.set_weapon(W::Sword);
    s.player.facing = std::f32::consts::PI;
    let i = s.spawn(crate::species::Kind::Husk, vec3(0.0, 0.0, -1.2));
    s.enemies[i].hp = 9000;
    punch(&mut s, &t, 3);
    assert_eq!(s.flow(), 0.0);
    assert_eq!(s.player.class.flow.level, 0.0, "the sword builds nothing");
}

/// 흐름은 맞으면 끊긴다(피해를 받은 타 — `wanderer_takes_hit`). 받는 피해 · 맞음 무적 · 경직은 흐름과 상관없이 그대로다.
#[test]
fn getting_hit_breaks_the_flow_and_nothing_else_changes() {
    let taken = |flowing: bool| {
        let (mut w, t) = monk_with_a_dummy();
        if flowing {
            punch(&mut w, &t, 4);
            assert!(w.flow() >= 1.0 - 1e-4);
        }
        w.player.state = PState::Idle;
        let hp = w.player.hp;
        let mut ev = Vec::new();
        w.wanderer_takes_hit(&t, &mut ev, Some(vec3(0.0, 0.0, -1.0)), 0);
        (hp - w.player.hp, w.flow(), w.player.hurt.left)
    };
    let (a, flow, left_a) = taken(true);
    let (b, _, left_b) = taken(false);
    assert_eq!(flow, 0.0, "broken");
    assert_eq!(a, b, "the same damage taken");
    assert_eq!(left_a, left_b, "the same iframes");
}

/// 열린 손(빈 손의 결 — 몽크는 늘): 패링하면 흐름이 단번에 가득 찬다. 다른 종류의 패링은 흐름을 만들지 않는다.
#[test]
fn a_parry_fills_the_flow_at_once() {
    let parried = |arm: W| {
        let mut w = World::new("test", 30.0);
        let t = Tuning::default();
        w.set_weapon(arm);
        w.spawn_mob(vec3(0.0, 0.0, -1.2));
        w.player.facing = std::f32::consts::PI;
        let en = &mut w.enemies[0];
        en.facing = 0.0;
        en.state = EState::Tele { t: en.tele_dur() };
        en.landed = false;
        w.player.state = PState::Parry { t: 0.0 };
        let mut hit = false;
        for _ in 0..60 {
            hit |= w
                .step(&Actions::default(), &t)
                .iter()
                .any(|e| matches!(e, Event::Parry { .. }));
            if hit {
                break;
            }
        }
        assert!(hit, "{arm:?} parried");
        (w.flow(), w.player.class.flow.level)
    };
    assert_eq!(parried(W::Fists).0, 1.0);
    assert_eq!(parried(W::Sword).1, 0.0, "not the sword's way");
}
