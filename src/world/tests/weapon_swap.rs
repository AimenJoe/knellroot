//! 주/부무기 바꾸기(0223 — 패링 + 평타를 함께): 같은 틱·창 안의 둘째 누름, 시동 안의 첫 동작 거두기, 튕긴 뒤의 반격, 탄은 무기마다.
use super::*;
use crate::weapons::Id;

fn armed(main: Id, sub: Id) -> World {
    let mut w = World::new("test", 20.0);
    w.set_weapon(main);
    w.set_sub(Some(sub));
    w
}

fn swapped(ev: &[Event]) -> Option<Id> {
    ev.iter().find_map(|e| match e {
        Event::WeaponSwap { to } => Some(*to),
        _ => None,
    })
}

/// 같은 틱에 둘 다 = 바로 바꾼다. 패링도 휘두르기도 나가지 않는다.
#[test]
fn parry_and_attack_together_swap_the_weapons() {
    let t = Tuning::default();
    let mut w = armed(Id::Sword, Id::Hammer);
    let ev = w.step(
        &Actions {
            parry: true,
            attack: true,
            ..Default::default()
        },
        &t,
    );
    assert_eq!(swapped(&ev), Some(Id::Hammer));
    assert_eq!(
        (w.player.weapon, w.player.sub),
        (Id::Hammer, Some(Id::Sword))
    );
    assert_eq!(w.player.state, PState::Idle, "neither a parry nor a swing");
    // 다시 함께 → 되돌아온다.
    for _ in 0..10 {
        w.step(&Actions::default(), &t);
    }
    let ev = w.step(
        &Actions {
            parry: true,
            attack: true,
            ..Default::default()
        },
        &t,
    );
    assert_eq!(swapped(&ev), Some(Id::Sword));
}

/// 둘째 누름이 첫째 뒤 `WEAPON_CHORD` 안이면 첫째가 시작한 패링을 거두고 바꾼다. 늦으면 그냥 평타.
#[test]
fn a_second_press_inside_the_chord_takes_back_the_parry_and_swaps() {
    let t = Tuning::default();
    let mut w = armed(Id::Sword, Id::Staff);
    w.step(
        &Actions {
            parry: true,
            ..Default::default()
        },
        &t,
    );
    assert!(matches!(w.player.state, PState::Parry { .. }));
    w.step(&Actions::default(), &t);
    let ev = w.step(
        &Actions {
            attack: true,
            ..Default::default()
        },
        &t,
    );
    assert_eq!(swapped(&ev), Some(Id::Staff));
    assert_eq!(w.player.state, PState::Idle);
    // 늦게 누른 평타는 바꾸지 않는다.
    let mut w = armed(Id::Sword, Id::Staff);
    w.step(
        &Actions {
            attack: true,
            ..Default::default()
        },
        &t,
    );
    let late = (crate::tuning::WEAPON_CHORD / TICK_DT).ceil() as usize + 2;
    for _ in 0..late {
        w.step(&Actions::default(), &t);
    }
    let ev = w.step(
        &Actions {
            parry: true,
            ..Default::default()
        },
        &t,
    );
    assert_eq!(swapped(&ev), None);
    assert_eq!(w.player.weapon, Id::Sword);
}

/// 패링이 튕겨 낸 뒤의 평타는 반격이다 — 바꾸지 않는다(패링 → 반격의 손을 지킨다).
#[test]
fn the_attack_after_a_deflect_is_a_riposte_not_a_swap() {
    let t = Tuning::default();
    let mut w = mob_swinging_in_front(&t);
    w.set_sub(Some(Id::Hammer));
    let mut ev = w.step(
        &Actions {
            parry: true,
            ..Default::default()
        },
        &t,
    );
    let mut n = 0;
    while !ev.iter().any(|e| matches!(e, Event::Parry { .. })) && n < 30 {
        ev = w.step(&Actions::default(), &t);
        n += 1;
    }
    assert!(
        ev.iter().any(|e| matches!(e, Event::Parry { .. })),
        "parried"
    );
    // 튕긴 뒤의 히트스톱 동안 누른 평타는 받아 두었다가 풀리면 나간다.
    let mut ev = w.step(
        &Actions {
            attack: true,
            ..Default::default()
        },
        &t,
    );
    for _ in 0..30 {
        assert_eq!(swapped(&ev), None, "a riposte");
        if matches!(w.player.state, PState::Attack { .. } | PState::Heavy { .. }) {
            break;
        }
        ev = w.step(&Actions::default(), &t);
    }
    assert_eq!(w.player.weapon, Id::Sword);
    assert!(
        matches!(w.player.state, PState::Attack { .. } | PState::Heavy { .. }),
        "the riposte (a heavy after a parry): {:?}",
        w.player.state
    );
}

/// 부무기가 없으면 함께 눌러도 바꾸지 않는다(그냥 패링·평타).
#[test]
fn without_an_off_weapon_the_chord_is_just_the_two_buttons() {
    let t = Tuning::default();
    let mut w = World::new("test", 20.0);
    let ev = w.step(
        &Actions {
            parry: true,
            attack: true,
            ..Default::default()
        },
        &t,
    );
    assert_eq!(swapped(&ev), None);
    assert_eq!(w.player.weapon, Id::Sword);
}

/// 탄은 무기마다 남는다: 쇠뇌를 비우고 바꿔 들었다 돌아와도 탄창이 차 있지 않다(바꾸기로 장전을 건너뛰지 못한다).
#[test]
fn swapping_back_does_not_refill_the_crossbow() {
    let t = Tuning::default();
    let mut w = armed(Id::Crossbow, Id::Sword);
    w.player.ranged.ammo = 1;
    w.player.ranged.reload = 0.0;
    assert_eq!(w.swap_weapons(), Some(Id::Sword));
    assert_eq!(w.swap_weapons(), Some(Id::Crossbow));
    assert_eq!(w.player.ranged.ammo, 1, "the magazine waited where it was");
    let _ = t;
}

/// 터치의 바꾸기 버튼(`Actions::swap`): 서 있으면 바로 바꾼다. 부무기가 없으면 아무 일도 없다.
#[test]
fn the_swap_button_swaps_at_once_when_standing() {
    let t = Tuning::default();
    let press = Actions {
        swap: true,
        ..Default::default()
    };
    let mut w = armed(Id::Sword, Id::Crossbow);
    assert_eq!(swapped(&w.step(&press, &t)), Some(Id::Crossbow));
    assert_eq!(w.player.sub, Some(Id::Sword));
    let mut w = World::new("test", 20.0);
    assert_eq!(swapped(&w.step(&press, &t)), None);
}

/// 바꿔 치기: 튕겨 낸 뒤 반격 창 안에 바꿔 들고 치면 **새 무기의** 반격(모으지 않은 다 모은 강타)이 나간다 — 바꾸기가 반격 창을 닫지 않는다.
#[test]
fn swapping_inside_the_riposte_window_ripostes_with_the_new_weapon() {
    let t = Tuning::default();
    let mut w = mob_swinging_in_front(&t);
    w.set_sub(Some(Id::Hammer));
    let mut ev = w.step(
        &Actions {
            parry: true,
            ..Default::default()
        },
        &t,
    );
    let mut n = 0;
    while !ev.iter().any(|e| matches!(e, Event::Parry { .. })) && n < 30 {
        ev = w.step(&Actions::default(), &t);
        n += 1;
    }
    assert!(
        ev.iter().any(|e| matches!(e, Event::Parry { .. })),
        "parried"
    );
    // 히트스톱이 풀릴 때까지 기다렸다가 패링 + 평타를 함께(바꾸기), 그리고 평타.
    while w.hitstop > 0.0 {
        w.step(&Actions::default(), &t);
    }
    let ev = w.step(
        &Actions {
            parry: true,
            attack: true,
            ..Default::default()
        },
        &t,
    );
    assert_eq!(swapped(&ev), Some(Id::Hammer));
    w.step(
        &Actions {
            attack: true,
            ..Default::default()
        },
        &t,
    );
    assert_eq!(w.player.weapon, Id::Hammer);
    assert!(
        matches!(w.player.state, PState::Heavy { full: true, .. }),
        "the hammer's riposte: {:?}",
        w.player.state
    );
}
