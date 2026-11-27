//! 방패 막기(0223 — 한손 무기 + 방패, 패링 버튼을 쥔 동안): 패링 창 뒤에 막기로 잇기, 기력으로 받기, 깨짐, 못 막는 것, 풀기.
use super::*;
use crate::shield;
use crate::weapons::Id;

fn hold() -> Actions {
    Actions {
        parry_held: true,
        ..Default::default()
    }
}

/// 방패를 든 방랑자 앞(-z)에서 잡몹이 내려치기를 막 휘두른다 — 방랑자는 이미 막고 있다.
fn guarding_a_blow(t: &Tuning) -> World {
    let mut w = mob_swinging_in_front(t);
    w.set_shield(Some(shield::Id::Pilgrim));
    w.player.state = PState::Block { t: 0.5 };
    w
}

fn step_until(
    w: &mut World,
    t: &Tuning,
    a: Actions,
    n: usize,
    hit: impl Fn(&Event) -> bool,
) -> bool {
    for _ in 0..n {
        if w.step(&a, t).iter().any(&hit) {
            return true;
        }
    }
    false
}

/// 누르는 순간은 패링 그대로, 창이 지나도 쥐고 있으면 막기 — 헛친 패링의 굳음 대신 방패 뒤에 선다. 떼면 풀린다.
#[test]
fn holding_parry_with_a_shield_goes_from_the_parry_window_into_a_guard() {
    let t = Tuning::default();
    let mut w = World::new("test", 20.0);
    w.set_shield(Some(shield::Id::Pilgrim));
    w.step(
        &Actions {
            parry: true,
            parry_held: true,
            ..Default::default()
        },
        &t,
    );
    assert!(
        matches!(w.player.state, PState::Parry { .. }),
        "a press is a parry first"
    );
    let window = (t[P::ParryWindow] / TICK_DT).ceil() as usize + 1;
    for _ in 0..window {
        w.step(&hold(), &t);
    }
    assert!(w.guarding(), "{:?}", w.player.state);
    w.step(&Actions::default(), &t);
    assert_eq!(w.player.state, PState::Idle, "letting go lowers the shield");
}

/// 방패가 없거나 든 무기가 한손이 아니면 쥐어도 패링뿐이다(양손·원거리는 막지 못한다).
#[test]
fn no_shield_or_a_two_handed_weapon_means_no_guard() {
    let t = Tuning::default();
    let long = (t[P::ParryTime] / TICK_DT).ceil() as usize + 4;
    let run = |shield: bool, weapon: Id| {
        let mut w = World::new("test", 20.0);
        w.set_weapon(weapon);
        w.set_shield(shield.then_some(shield::Id::Pilgrim));
        w.step(
            &Actions {
                parry: true,
                parry_held: true,
                ..Default::default()
            },
            &t,
        );
        let mut guarded = false;
        for _ in 0..long {
            w.step(&hold(), &t);
            guarded |= w.guarding();
        }
        guarded
    };
    assert!(run(true, Id::Sword));
    // 0286: 맨손은 몽크 — 두 주먹이 다 차 방패로 막지 않는다.
    assert!(!run(true, Id::Fists), "the monk holds no shield");
    assert!(!run(false, Id::Sword), "no shield");
    // 0274: 망치는 한 손에 드는 메 — 방패와 함께 막는다. 두 손(창)은 막지 못한다.
    assert!(run(true, Id::Hammer), "the one-handed hammer");
    assert!(!run(true, Id::Spear), "two hands");
    assert!(!run(true, Id::Staff));
    assert!(!run(true, Id::Crossbow));
}

/// 막은 타는 체력이 아니라 기력을 깎는다 — 적은 휘청이지 않는다(튕긴 것이 아니다).
#[test]
fn a_guarded_blow_costs_stamina_not_health() {
    let t = Tuning::default();
    let mut w = guarding_a_blow(&t);
    let s0 = w.player.stamina;
    assert!(step_until(&mut w, &t, hold(), 40, |e| matches!(
        e,
        Event::Guard { .. }
    )));
    assert_eq!(w.player.hp, PLAYER_HP);
    assert!(
        (s0 - w.player.stamina - crate::tuning::BLOCK_STAMINA).abs() < 1e-3,
        "{} → {}",
        s0,
        w.player.stamina
    );
    assert!(w.guarding(), "still behind the shield");
    assert!(
        !matches!(w.enemies[0].state, EState::Stagger { .. }),
        "a guard is not a parry"
    );
    assert_eq!(w.enemies[0].exposed, 0.0);
}

/// 기력이 바닥나는 타에 막기가 깨진다: 그 타의 피해는 없지만 길게 휘청인다.
#[test]
fn the_blow_that_empties_the_stamina_breaks_the_guard() {
    let t = Tuning::default();
    let mut w = guarding_a_blow(&t);
    w.player.stamina = crate::tuning::BLOCK_STAMINA * 0.5;
    assert!(step_until(&mut w, &t, hold(), 40, |e| matches!(
        e,
        Event::GuardBreak { .. }
    )));
    assert_eq!(
        w.player.hp, PLAYER_HP,
        "the breaking blow itself does not wound"
    );
    assert_eq!(w.player.stamina, 0.0);
    assert!(
        matches!(w.player.state, PState::Hurt { dur, .. } if dur == crate::tuning::BLOCK_BREAK_STUN),
        "{:?}",
        w.player.state
    );
}

/// 방패는 앞만 막는다 — 등 뒤에서 맞으면 그대로 다친다.
#[test]
fn a_guard_does_not_cover_the_back() {
    let t = Tuning::default();
    let mut w = guarding_a_blow(&t);
    w.player.facing = 0.0; // +z = 적을 등진다
    assert!(step_until(&mut w, &t, hold(), 40, |e| matches!(
        e,
        Event::PlayerHurt
    )));
    assert!(w.player.hp < PLAYER_HP);
}

/// 막는 동안은 기력이 차지 않고, 느리게 걷고, 걷는 쪽으로 돌지 않는다(방패를 든 채 물러선다).
#[test]
fn guarding_holds_the_stamina_walks_slowly_and_keeps_facing() {
    let t = Tuning::default();
    let mut w = World::new("test", 20.0);
    w.set_shield(Some(shield::Id::Pilgrim));
    w.player.stamina = 40.0;
    w.player.state = PState::Block { t: 0.0 };
    let facing = w.player.facing;
    let back = Actions {
        move_dir: vec2(0.0, -1.0),
        ..hold()
    };
    for _ in 0..120 {
        w.step(&back, &t);
    }
    assert!(w.guarding());
    assert_eq!(w.player.stamina, 40.0, "no regen behind the shield");
    assert_eq!(w.player.facing, facing, "backs away without turning");
    let speed = w.player.vel.length();
    let want = t[P::MoveSpeed] * crate::tuning::BLOCK_WALK;
    assert!((speed - want).abs() < 0.05, "{speed} vs {want}");
}

/// 막다가 누른 것은 그 틱에 나간다(막기에서 치기·피하기가 늦지 않다).
#[test]
fn a_press_out_of_the_guard_acts_on_that_tick() {
    let t = Tuning::default();
    for (a, want) in [
        (
            Actions {
                attack: true,
                ..hold()
            },
            "attack",
        ),
        (
            Actions {
                dodge: true,
                ..hold()
            },
            "dodge",
        ),
    ] {
        let mut w = World::new("test", 20.0);
        w.set_shield(Some(shield::Id::Pilgrim));
        w.player.state = PState::Block { t: 0.3 };
        w.step(&a, &t);
        let ok = match want {
            "attack" => matches!(w.player.state, PState::Attack { .. }),
            _ => matches!(w.player.state, PState::Dodge { .. }),
        };
        assert!(ok, "{want}: {:?}", w.player.state);
    }
}

/// 방패를 들어도 패링 창은 그대로다(0095 원칙 2): 창 안의 타는 여전히 튕긴다 — 막기가 아니라.
#[test]
fn the_parry_window_is_untouched_by_the_shield() {
    let t = Tuning::default();
    let mut w = mob_swinging_in_front(&t);
    w.set_shield(Some(shield::Id::Pilgrim));
    let mut ev = w.step(
        &Actions {
            parry: true,
            parry_held: true,
            ..Default::default()
        },
        &t,
    );
    for _ in 0..30 {
        if ev
            .iter()
            .any(|e| matches!(e, Event::Parry { .. } | Event::Guard { .. }))
        {
            break;
        }
        ev = w.step(&hold(), &t);
    }
    assert!(
        ev.iter().any(|e| matches!(e, Event::Parry { .. })),
        "{ev:?}"
    );
    assert!(matches!(w.enemies[0].state, EState::Stagger { .. }));
}
