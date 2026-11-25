//! 주/부무기 바꾸기(0223 → 0261 — 따로 붙인 바꾸기 버튼): 서 있으면 바로·동작 중엔 받아 두기, 조합은 바꾸지 않음, 튕긴 뒤의 반격, 탄은 무기마다.
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

/// 0261: 패링 + 평타를 함께 눌러도 이제 바꾸지 않는다 — 바꾸기는 따로 붙인 버튼 하나(패드 L3·키보드 X). 둘은 제 일을 한다.
#[test]
fn parry_and_attack_together_no_longer_swap() {
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
    assert_eq!(swapped(&ev), None);
    assert_eq!(w.player.weapon, Id::Sword);
    assert_ne!(
        w.player.state,
        PState::Idle,
        "the parry or the swing went out"
    );
}

/// 휘두르는 동안 누른 바꾸기 버튼은 사라지지 않고 받아 두었다가(`SWAP_BUFFER`) 서는 순간 바꾼다. 너무 이르면 버린다.
#[test]
fn a_swap_pressed_mid_swing_waits_for_the_stand() {
    let t = Tuning::default();
    let mut w = armed(Id::Sword, Id::Staff);
    w.step(
        &Actions {
            attack: true,
            ..Default::default()
        },
        &t,
    );
    assert!(matches!(w.player.state, PState::Attack { .. }));
    let mut ev = w.step(
        &Actions {
            swap: true,
            ..Default::default()
        },
        &t,
    );
    assert_eq!(swapped(&ev), None, "not in the middle of a swing");
    let mut n = 0;
    while swapped(&ev).is_none() && n < 120 {
        ev = w.step(&Actions::default(), &t);
        n += 1;
    }
    // 받아 둔 바꾸기가 나갔다면 그것은 휘두름이 끝나 선 틱이다.
    if swapped(&ev).is_some() {
        assert_eq!(w.player.weapon, Id::Staff);
        assert_eq!(w.player.state, PState::Idle);
    } else {
        // 휘두름이 받아 두기보다 길면 버려진다 — 그때도 무기는 그대로다.
        assert_eq!(w.player.weapon, Id::Sword);
    }
    // 서 있는 동안 누르면 바로.
    let ev = w.step(
        &Actions {
            swap: true,
            ..Default::default()
        },
        &t,
    );
    assert!(swapped(&ev).is_some());
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

/// 바꾸기 버튼(`Actions::swap` — 터치·패드 L3·키보드 X): 서 있으면 바로 바꾼다. 부무기가 없으면 아무 일도 없다.
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

/// 튕겨 낸 뒤 반격 창 안에 바꿔 들고 치면 **새 무기의** 반격(모으지 않은 다 모은 강타)이 나간다 — 그냥 바꾸기는 반격 창을 닫지
/// 않는다. 0272 1b부터 받아친 직후의 바꾸기는 교체 공격이 되므로(`swap_strike.rs`), 그냥 바꾸는 때 = 교체 공격의 잠금 동안.
#[test]
fn a_plain_swap_inside_the_riposte_window_ripostes_with_the_new_weapon() {
    let t = Tuning::default();
    let mut w = mob_swinging_in_front(&t);
    w.set_sub(Some(Id::Hammer));
    w.player.swap_lock = 1.0;
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
    // 히트스톱이 풀릴 때까지 기다렸다가 바꾸기 버튼, 그리고 평타.
    while w.hitstop > 0.0 {
        w.step(&Actions::default(), &t);
    }
    let ev = w.step(
        &Actions {
            swap: true,
            ..Default::default()
        },
        &t,
    );
    assert_eq!(swapped(&ev), Some(Id::Hammer));
    assert!(!ev.iter().any(|e| matches!(e, Event::SwapStrike { .. })));
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

/// 0223 V3: 스킬 칸은 무기 칸마다 둘 — 바꿔 들면 통째로 바뀐다. 든 무기의 갈래로 쓸 수 없는 스킬이 놓인 칸은 빈 칸이다
/// (눌러도 아무 일 없다 — 맞는 무기를 들면 되살아난다). 대기는 스킬마다라 바꿔도 그대로 흐른다.
#[test]
fn swapping_weapons_swaps_the_skill_slots_and_an_unfit_skill_is_an_empty_slot() {
    use crate::skills::Id as Sk;
    let t = Tuning::default();
    let mut w = armed(Id::Sword, Id::Hammer);
    // 검의 칸: 제비걸음(한손) · 땅울림(양손 — 검으로는 못 쓴다). 망치의 칸: 땅울림 · 휘돌이.
    w.set_skills([Sk::Swallow, Sk::Quake]);
    w.set_sub_skills([Sk::Quake, Sk::Whirl]);
    assert_eq!(w.player.skill.rank, [1, 0]);
    let quake = Actions {
        skill2: true,
        ..Default::default()
    };
    let ev = w.step(&quake, &t);
    assert_eq!(w.player.state, PState::Idle, "an empty slot");
    assert!(!ev.contains(&Event::SkillDenied));
    assert_eq!(w.swap_weapons(), Some(Id::Hammer));
    assert_eq!(w.player.skill.worn, [Sk::Quake, Sk::Whirl]);
    assert_eq!(w.player.skill.rank, [1, 1]);
    w.step(
        &Actions {
            skill1: true,
            ..Default::default()
        },
        &t,
    );
    assert!(
        matches!(w.player.state, PState::Skill { id: Sk::Quake, .. }),
        "{:?}",
        w.player.state
    );
    // 다시 검으로: 검의 칸이 돌아오고, 방금 쓴 땅울림의 대기는 흐르는 채다.
    w.player.state = PState::Idle;
    assert_eq!(w.swap_weapons(), Some(Id::Sword));
    assert_eq!(w.player.skill.worn, [Sk::Swallow, Sk::Quake]);
    assert_eq!(w.player.skill.rank, [1, 0]);
    assert!(w.player.skill.cd[Sk::Quake.index()] > 0.0);
    // 무기를 통째로 바꿔 들어도(장비 탭) 칸은 그 자리의 것 — 망치를 주무기로 들면 땅울림이 살아난다.
    w.set_weapon(Id::Hammer);
    assert_eq!(w.player.skill.rank, [0, 1], "swallow step needs one hand");
}
