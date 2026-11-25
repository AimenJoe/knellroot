//! 교체 공격과 멘 쇠뇌의 장전(0272 1b): 내 타가 닿은 직후·받아친 직후의 바꾸기는 들어오는 무기의 한 타와 함께 나간다 —
//! 서서 바꾸면 그냥 바꾼다, 잠금 동안에도 그냥 바꾼다, 빈 사격 무기도 그냥 바꾼다. 무적·넓어진 판정은 없다.
use super::*;
use crate::tuning::{SWAP_LINK, SWAP_STRIKE_LOCK};
use crate::weapons::{Id, DASH};

fn armed(main: Id, sub: Id) -> World {
    let mut w = World::new("test", 20.0);
    w.set_weapon(main);
    w.set_sub(Some(sub));
    w
}

/// 세워 둔(치지 않는) 튼튼한 적 하나를 바로 앞에.
fn dummy_in_front(w: &mut World) -> usize {
    let i = mob_with(w, vec3(0.0, 0.0, -1.2), 10_000);
    idle(w);
    w.player.facing = std::f32::consts::PI;
    i
}

fn swap_press() -> Actions {
    Actions {
        swap: true,
        ..Default::default()
    }
}

fn attack_press() -> Actions {
    Actions {
        attack: true,
        ..Default::default()
    }
}

fn has(ev: &[Event], f: fn(&Event) -> bool) -> bool {
    ev.iter().any(f)
}

fn swapped(e: &Event) -> bool {
    matches!(e, Event::WeaponSwap { .. })
}

fn struck(e: &Event) -> bool {
    matches!(e, Event::SwapStrike { .. })
}

/// 평타를 눌러 처음 닿는 틱까지 — 그 틱의 사건.
fn swing_until_hit(w: &mut World, t: &Tuning) -> Vec<Event> {
    let mut ev = w.step(&attack_press(), t);
    for _ in 0..60 {
        if has(&ev, |e| matches!(e, Event::Hit { .. } | Event::Crit { .. })) {
            return ev;
        }
        ev = w.step(&Actions::default(), t);
    }
    panic!("the swing never landed: {:?}", w.player.state);
}

/// 바꾸기를 누르고, 바뀌는 틱까지(틱 수, 그 틱의 사건).
fn swap_and_wait(w: &mut World, t: &Tuning) -> (usize, Vec<Event>) {
    let mut ev = w.step(&swap_press(), t);
    for n in 0..120 {
        if has(&ev, swapped) {
            return (n, ev);
        }
        ev = w.step(&Actions::default(), t);
    }
    panic!("never swapped: {:?}", w.player.state);
}

/// 맞힌 직후의 바꾸기 = 교체 공격: 맞힌 순간의 멈춤 동안 눌러도 받아 두고, 검 휘두름의 캔슬 시점(다음 타로 잇는 자리)에서
/// 후딜을 끊어 망치가 제 대시 공격으로 들어온다 — 피해도 망치의 것, 보통 무기 타라 큰 기술 게이지를 채운다.
#[test]
fn a_swap_right_after_a_hit_lands_brings_the_new_weapon_in_striking() {
    let t = Tuning::default();
    let mut w = armed(Id::Sword, Id::Hammer);
    let foe = dummy_in_front(&mut w);
    w.player.skill.ult_rank = 1;
    swing_until_hit(&mut w, &t);
    assert!(w.hitstop > 0.0, "the hit froze the world");
    assert!(w.swap_strikes(), "the window is open");
    let PState::Attack { hit: 0, .. } = w.player.state else {
        panic!("{:?}", w.player.state);
    };
    // 멈춘 동안 누른 바꾸기 — 버려지지 않는다.
    let (n, ev) = swap_and_wait(&mut w, &t);
    assert!(has(&ev, struck), "a swap strike: {ev:?}");
    assert_eq!(w.player.weapon, Id::Hammer);
    assert_eq!(w.player.sub, Some(Id::Sword));
    let PState::Attack { hit, swing, .. } = w.player.state else {
        panic!("the entry strike: {:?}", w.player.state);
    };
    assert_eq!(hit, DASH, "the hammer's dash move");
    // 검의 첫 타는 다 휘두르기 전에(후딜의 `combo.link` 자리에서) 끊겼다 — 서기를 기다리지 않았다.
    let (s0, a0, r0) = hit_times(&t, Id::Sword.def(), 0);
    let whole = ((s0 + a0 + r0) / TICK_DT) as usize;
    assert!(n < whole, "cut at the link point: {n} of {whole} ticks");
    // 닿으면 망치 대시 공격의 피해 그대로(덤 없음), 게이지는 평타 한 번만큼.
    let (hp0, gauge0) = (w.enemies[foe].hp, w.player.skill.gauge);
    for _ in 0..90 {
        w.step(&Actions::default(), &t);
        if w.player.connected == swing {
            break;
        }
    }
    assert_eq!(w.player.connected, swing, "the entry strike landed");
    let dash = Id::Hammer.def().dash.unwrap();
    assert_eq!(hp0 - w.enemies[foe].hp, dash.damage);
    assert!(
        w.player.skill.gauge > gauge0,
        "a weapon hit fills the gauge"
    );
    // 기력은 대시 공격 한 번 값(공짜가 아니다).
    assert!(w.player.stamina < t[P::StaminaMax]);
}

/// 서서 바꾸면 그냥 바꾼다 — 맞힌 적이 없으면, 그리고 맞힌 지 `SWAP_LINK`가 지났으면.
#[test]
fn a_swap_from_idle_or_after_the_window_is_a_plain_swap() {
    let t = Tuning::default();
    let mut w = armed(Id::Sword, Id::Hammer);
    dummy_in_front(&mut w);
    assert!(!w.swap_strikes());
    let ev = w.step(&swap_press(), &t);
    assert!(has(&ev, swapped) && !has(&ev, struck));
    assert_eq!(w.player.state, PState::Idle);
    // 맞히고, 창이 닫힐 때까지 기다렸다 바꾼다.
    let mut w = armed(Id::Sword, Id::Hammer);
    dummy_in_front(&mut w);
    swing_until_hit(&mut w, &t);
    for _ in 0..((SWAP_LINK + 0.2) / TICK_DT) as usize + 30 {
        w.step(&Actions::default(), &t);
    }
    assert_eq!(w.player.state, PState::Idle);
    let ev = w.step(&swap_press(), &t);
    assert!(has(&ev, swapped) && !has(&ev, struck), "too late: {ev:?}");
    assert_eq!(w.player.state, PState::Idle);
    // 빗나간 휘두름은 창을 열지 않는다: 휘두르는 동안 누른 바꾸기는 지금까지처럼 설 때까지 받아 두거나 버려진다.
    let mut w = armed(Id::Sword, Id::Hammer);
    w.step(&attack_press(), &t);
    let mut ev = w.step(&swap_press(), &t);
    for _ in 0..60 {
        assert!(!has(&ev, struck), "a whiff opens nothing");
        assert!(!has(&ev, swapped) || w.player.state == PState::Idle);
        ev = w.step(&Actions::default(), &t);
    }
}

/// 잠금: 교체 공격이 한 번 나가면 `SWAP_STRIKE_LOCK` 동안의 바꾸기는 그냥 바꾼다 — 그 타가 닿아 창이 다시 열려도.
/// 잠금이 풀리고 다시 맞히면 또 나간다.
#[test]
fn swap_strikes_do_not_chain_inside_the_lockout() {
    let t = Tuning::default();
    let mut w = armed(Id::Sword, Id::Hammer);
    dummy_in_front(&mut w);
    swing_until_hit(&mut w, &t);
    let (_, ev) = swap_and_wait(&mut w, &t);
    assert!(has(&ev, struck));
    let PState::Attack { swing, .. } = w.player.state else {
        panic!()
    };
    for _ in 0..90 {
        w.step(&Actions::default(), &t);
        if w.player.connected == swing {
            break;
        }
    }
    assert_eq!(w.player.connected, swing);
    assert!(w.player.swap_link > 0.0, "the entry strike landed too");
    assert!(!w.swap_strikes(), "but the lock holds");
    // 곧바로 다시 바꾸기: 망치 휘두름이 끝나 설 때까지 기다렸다(받아 두기 안이면) 그냥 바꾸거나, 버려진다 — 교체 공격은 아니다.
    let mut ev = w.step(&swap_press(), &t);
    for _ in 0..60 {
        assert!(!has(&ev, struck), "no second strike inside the lock");
        ev = w.step(&Actions::default(), &t);
    }
    // 잠금이 풀린 뒤: 다시 맞히고 바꾸면 또 나간다.
    for _ in 0..(SWAP_STRIKE_LOCK / TICK_DT) as usize {
        w.step(&Actions::default(), &t);
    }
    assert_eq!(w.player.swap_lock, 0.0);
    swing_until_hit(&mut w, &t);
    let (_, ev) = swap_and_wait(&mut w, &t);
    assert!(has(&ev, struck), "after the lock: {ev:?}");
}

/// 받아친 직후의 바꾸기 = 교체 공격(반격 대신 나간다 — 패링 하나에 보상 하나): 들어온 망치가 대시 공격으로 치고,
/// 그 뒤의 평타는 반격(다 모은 강타)이 아니다.
#[test]
fn a_swap_right_after_a_parry_strikes_and_closes_the_riposte_window() {
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
    while !has(&ev, |e| matches!(e, Event::Parry { .. })) && n < 30 {
        ev = w.step(&Actions::default(), &t);
        n += 1;
    }
    assert!(has(&ev, |e| matches!(e, Event::Parry { .. })), "parried");
    assert!(w.swap_strikes());
    let (_, ev) = swap_and_wait(&mut w, &t);
    assert!(has(&ev, struck), "{ev:?}");
    assert!(
        matches!(w.player.state, PState::Attack { hit: DASH, .. }),
        "{:?}",
        w.player.state
    );
    assert_eq!(w.player.weapon, Id::Hammer);
    assert!(w.player.parry.since > crate::tuning::RIPOSTE_WINDOW);
    assert_eq!(w.player.parry.chain_left, 0.0, "the chain closes too");
}

/// 사격 무기가 들어올 때: 지금 겨눈 쪽으로 보통 한 발(탄 하나) — 멘 탄창이 비었으면 그냥 바꾼다.
#[test]
fn an_incoming_ranged_weapon_fires_one_shot_and_an_empty_one_just_swaps() {
    let t = Tuning::default();
    let mut w = armed(Id::Sword, Id::Bow);
    dummy_in_front(&mut w);
    swing_until_hit(&mut w, &t);
    let (_, ev) = swap_and_wait(&mut w, &t);
    assert!(has(&ev, struck), "{ev:?}");
    assert_eq!(w.player.weapon, Id::Bow);
    let PState::Cast { dir, .. } = w.player.state else {
        panic!("a shot: {:?}", w.player.state);
    };
    assert!(dir.dot(vec3(0.0, 0.0, -1.0)) > 0.9, "at the foe ahead");
    assert_eq!(w.player.ranged.ammo, w.mag_of(Id::Bow) - 1, "one arrow");
    let before = w.bolts.len();
    for _ in 0..30 {
        w.step(&Actions::default(), &t);
    }
    assert!(w.bolts.len() > before || w.player.connected != 0, "it flew");
    // 빈 탄창으로 멘 활: 맞힌 직후에 바꿔도 그냥 바꾼다.
    let mut w = armed(Id::Bow, Id::Sword);
    dummy_in_front(&mut w);
    w.player.ranged.ammo = 0;
    assert_eq!(w.swap_weapons(), Some(Id::Sword));
    swing_until_hit(&mut w, &t);
    assert!(
        w.player.swap_link > 0.0 && !w.swap_strikes(),
        "nothing to shoot"
    );
    // 휘두르는 동안 누른 바꾸기는 지금까지처럼(설 때까지 받아 두거나 버려진다) — 끊고 들어오지 않는다.
    let mut ev = w.step(&swap_press(), &t);
    for _ in 0..60 {
        assert!(!has(&ev, struck));
        assert!(!has(&ev, swapped) || w.player.state == PState::Idle);
        ev = w.step(&Actions::default(), &t);
    }
    // 맞힌 직후 서서 눌러도(창 안) 그냥 바꾼다.
    w.set_weapon(Id::Sword);
    w.player.sub = Some(Id::Bow);
    w.player.swap_link = SWAP_LINK;
    let ev = w.step(&swap_press(), &t);
    assert!(has(&ev, swapped) && !has(&ev, struck), "{ev:?}");
    assert_eq!(w.player.weapon, Id::Bow);
    assert_eq!(w.player.state, PState::Idle);
    assert_eq!(w.player.ranged.ammo, 0);
}

/// 기력이 바닥이면 한 타의 값을 못 치른다 — 그냥 바꾼다(끊고 아무것도 안 나가는 일은 없다).
#[test]
fn without_stamina_the_swap_is_plain() {
    let t = Tuning::default();
    let mut w = armed(Id::Sword, Id::Hammer);
    dummy_in_front(&mut w);
    swing_until_hit(&mut w, &t);
    w.player.stamina = 0.0;
    w.player.stamina_idle = 0.0;
    assert!(!w.swap_strikes());
    let ev = w.step(&swap_press(), &t);
    assert!(!has(&ev, struck));
}

/// 무적도, 넓어진 판정 시간도 없다: 교체 공격으로 들어오는 동안에도 적의 타는 그대로 맞는다.
#[test]
fn the_swap_strike_gives_no_invulnerability() {
    let t = Tuning::default();
    let mut w = armed(Id::Sword, Id::Hammer);
    dummy_in_front(&mut w);
    swing_until_hit(&mut w, &t);
    let (_, ev) = swap_and_wait(&mut w, &t);
    assert!(has(&ev, struck));
    let hp = w.player.hp;
    let ev = hit_by_swing(&mut w, &t);
    assert!(ev.contains(&Event::PlayerHurt), "{ev:?}");
    assert!(w.player.hp < hp);
}

/// 보스에게도 그냥 대시 공격 한 타다: 그 무기의 대시 공격을 회피 끝에 낸 것과 같은 피해(덤·휘청 없음).
#[test]
fn on_a_boss_the_swap_strike_is_just_the_dash_attack() {
    let t = Tuning::default();
    let mut w = World::new("test", 20.0);
    w.set_weapon(Id::Hammer);
    w.set_sub(Some(Id::Sword));
    w.spawn_boss(vec3(0.0, 0.0, -2.0));
    idle(&mut w);
    w.player.facing = std::f32::consts::PI;
    swing_until_hit(&mut w, &t);
    let (_, ev) = swap_and_wait(&mut w, &t);
    assert!(has(&ev, struck), "{ev:?}");
    let PState::Attack { swing, hit, .. } = w.player.state else {
        panic!()
    };
    assert_eq!(hit, DASH);
    let hp0 = w.enemies[0].hp;
    for _ in 0..90 {
        w.step(&Actions::default(), &t);
        if w.player.connected == swing {
            break;
        }
    }
    assert_eq!(w.player.connected, swing);
    assert_eq!(hp0 - w.enemies[0].hp, Id::Sword.def().dash.unwrap().damage);
    assert!(
        !matches!(w.enemies[0].state, EState::Stagger { .. }),
        "a boss does not flinch"
    );
}

/// 쇠뇌의 보조특성: 멘 동안에도 장전된다 — 비워서 메면 제 장전 시간 뒤 가득, 덜 찬 탄창으로 메도 장전이 돌아 가득.
/// 손에 든 동안은 그대로다(비워야 장전되고 빠르기도 같다). 다 차기 전에 꺼내면 등에서 시작한 장전은 버린다 —
/// 두 번 바꿔 손에서 미리 장전하는 길은 없다. 활은 멘 동안 멈춰 있다(쇠뇌의 특성).
#[test]
fn a_stowed_crossbow_reloads_on_the_back_and_a_bow_does_not() {
    let t = Tuning::default();
    let Rb::Shot { reload, mag, .. } = Id::Crossbow.def().rb else {
        panic!()
    };
    let wait = |w: &mut World, secs: f32| {
        for _ in 0..(secs / TICK_DT).ceil() as usize {
            w.step(&Actions::default(), &t);
        }
    };
    // 덜 찬 탄창(1/4)으로 멘다: 장전 시간 전에는 그대로, 지나면 가득.
    let mut w = armed(Id::Crossbow, Id::Sword);
    w.player.ranged.ammo = 1;
    assert_eq!(w.swap_weapons(), Some(Id::Sword));
    wait(&mut w, reload - 0.2);
    assert!(!w.player.sub_load.unwrap().full, "not yet");
    wait(&mut w, 0.3);
    assert!(w.player.sub_load.unwrap().full);
    assert_eq!(w.swap_weapons(), Some(Id::Crossbow));
    assert_eq!(w.player.ranged.ammo, mag, "out full");
    assert_eq!(w.player.ranged.reload, 0.0);
    assert!(w.swap_full);
    // 다 차기 전에 꺼내면: 남은 탄 그대로, 손에서는 장전이 돌지 않는다(비워야 장전된다).
    let mut w = armed(Id::Crossbow, Id::Sword);
    w.player.ranged.ammo = 1;
    w.swap_weapons();
    wait(&mut w, reload * 0.5);
    w.swap_weapons();
    assert_eq!((w.player.ranged.ammo, w.player.ranged.reload), (1, 0.0));
    assert!(!w.swap_full);
    wait(&mut w, reload + 0.5);
    assert_eq!(w.player.ranged.ammo, 1, "no reload in hand until empty");
    // 손에 든 쇠뇌의 장전은 그대로: 비우면 `reload`초 — 더 빠르지 않다.
    let mut w = armed(Id::Crossbow, Id::Sword);
    w.player.ranged.ammo = 0;
    w.player.ranged.reload = reload;
    wait(&mut w, reload - 0.1);
    assert_eq!(w.player.ranged.ammo, 0);
    wait(&mut w, 0.2);
    assert_eq!(w.player.ranged.ammo, mag);
    // 비워서 장전 중에 멘 것은 등에서 이어 가고, 도중에 꺼내면 손에서 이어 간다(비워 시작한 장전).
    let mut w = armed(Id::Crossbow, Id::Sword);
    w.player.ranged.ammo = 0;
    w.player.ranged.reload = reload;
    w.swap_weapons();
    wait(&mut w, 1.0);
    w.swap_weapons();
    assert_eq!(w.player.ranged.ammo, 0);
    assert!((w.player.ranged.reload - (reload - 1.0)).abs() < 0.05);
    // 활: 멘 동안 멈춰 있다.
    let Rb::Shot { reload: bow, .. } = Id::Bow.def().rb else {
        panic!()
    };
    let mut w = armed(Id::Bow, Id::Sword);
    w.player.ranged.ammo = 0;
    w.player.ranged.reload = bow;
    w.swap_weapons();
    wait(&mut w, bow + 1.0);
    let l = w.player.sub_load.unwrap();
    assert!(!l.full && l.reload == bow && l.ammo == 0, "{l:?}");
    // 쇠뇌를 주무기로 든 채(멘 것은 검)면 등의 장전 같은 것은 없다 — 멘 쇠뇌에만.
    let mut w = armed(Id::Sword, Id::Crossbow);
    assert!(w.player.sub_load.is_none(), "never held: full when drawn");
    w.swap_weapons();
    assert_eq!(w.player.ranged.ammo, mag);
}

/// 재는 도구(0272 1b): 서 있는 적에게 `secs`초 동안 준 피해(초당) — 평타를 마구 누르며, `swapping`이면 교체 공격이 될 때마다
/// 바꾸기를 누른다. `cargo test swap_dps_probe -- --ignored --nocapture`
fn swap_dps(main: Id, sub: Id, secs: f32, swapping: bool) -> f32 {
    let t = Tuning::default();
    let mut w = armed(main, sub);
    let hp0 = 1_000_000;
    let i = mob_with(&mut w, vec3(0.0, 0.0, -1.2), hp0);
    for k in 0..(secs / TICK_DT) as usize {
        idle(&mut w);
        w.enemies[i].pos = vec3(0.0, 0.0, -1.2);
        w.enemies[i].stance.poise = 0.0;
        w.player.pos = Vec3::ZERO;
        let a = Actions {
            attack: k % 2 == 0,
            // 교체 공격이 될 때만 누른다(잠금 동안의 그냥 바꾸기는 콤보를 끊어 손해다 — 가장 잘 쓰는 손).
            swap: swapping && w.swap_strikes(),
            ..Default::default()
        };
        w.step(&a, &t);
    }
    (hp0 - w.enemies[i].hp) as f32 / secs
}

#[test]
#[ignore]
fn swap_dps_probe() {
    for (a, b) in [
        (Id::Sword, Id::Hammer),
        (Id::Sword, Id::Staff),
        (Id::Hammer, Id::Staff),
    ] {
        for secs in [4.0, 20.0] {
            println!(
                "{a:?}+{b:?} {secs}s: {a:?} only {:.1}/s · {b:?} only {:.1}/s · swap strike whenever ready {:.1}/s",
                swap_dps(a, b, secs, false),
                swap_dps(b, a, secs, false),
                swap_dps(a, b, secs, true),
            );
        }
    }
}

/// 잠금의 뜻: 교체 공격을 되는 대로 다 내도 한 무기로 치는 것보다 눈에 띄게 세지 않다 — 둘 가운데 센 무기의 평타 연타의
/// 1.15배 안(20초). 교체 공격은 초당 피해가 아니라 이어 치기의 한 갈래다.
#[test]
fn using_every_swap_strike_does_not_out_damage_one_weapon_by_much() {
    for (a, b) in [(Id::Sword, Id::Hammer), (Id::Sword, Id::Staff)] {
        let one = swap_dps(a, b, 20.0, false).max(swap_dps(b, a, 20.0, false));
        let both = swap_dps(a, b, 20.0, true);
        assert!(both <= one * 1.15, "{a:?}+{b:?}: {both:.1} vs {one:.1}");
    }
}
