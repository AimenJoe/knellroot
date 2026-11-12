//! 등잔(0031): 한 모금, 가득·빈 등잔, 연타, 삼키기 전의 회피·맞음.
use super::*;

/// 등잔 시험용: 한 번 누른 뒤 `ticks`틱 그대로(누름 없음). 사건을 모은다.
fn drink(w: &mut World, t: &Tuning, ticks: usize) -> Vec<Event> {
    let mut ev = w.step(
        &Actions {
            flask: true,
            ..Default::default()
        },
        t,
    );
    for _ in 0..ticks {
        ev.extend(w.step(&Actions::default(), t));
    }
    ev
}

/// 0031: 한 모금 = 체력 + `flask.heal`(가득을 넘지 않는다), 등잔 −1, 삼키는 순간 한 번.
#[test]
fn a_sip_heals_and_spends_one_charge() {
    let t = Tuning::default();
    let mut w = World::new("test", 20.0);
    w.player.hp = 10;
    let ev = drink(&mut w, &t, 80); // 마시기 0.8초 + 누름 버리기 0.3초를 넘게
    assert_eq!(w.player.hp, 10 + t[P::FlaskHeal] as i32);
    assert_eq!(w.player.hp, 30, "a sip is two old points");
    assert_eq!(w.player.flask, FLASK_CHARGES - 1);
    assert_eq!(
        ev.iter()
            .filter(|e| matches!(e, Event::Gulp { .. }))
            .count(),
        1
    );
    w.player.hp = PLAYER_HP - 10;
    drink(&mut w, &t, 80);
    assert_eq!(w.player.hp, PLAYER_HP, "never above full");
}

/// 0031: 가득이거나 등잔이 0이면 몸짓 없이 깜빡만 — 등잔·자세 그대로.
#[test]
fn a_full_or_empty_flask_press_does_nothing_but_flash() {
    let t = Tuning::default();
    for (hp, charges) in [(PLAYER_HP, FLASK_CHARGES), (10, 0)] {
        let mut w = World::new("test", 20.0);
        w.player.hp = hp;
        w.player.flask = charges;
        let ev = drink(&mut w, &t, 0);
        assert_eq!(w.player.state, PState::Idle, "hp {hp} charges {charges}");
        assert_eq!(w.player.flask, charges);
        assert!(ev.contains(&Event::FlaskDenied));
        assert!(w.player.flask_denied > 0.0);
    }
}

/// 0031(사장 "물약이 더 큰 문제 아냐?"): RB 연타 — 누름 1~9틱 × 사이 2~14틱 × 3초. 마시기는 끊기지 않고(들어간 만큼 삼킨다),
/// 마신 수는 3초 ÷ (`flask.time` + `flask.lock`)을 넘지 않는다(누름이 쌓여 연달아 마시지 않는다). 가득이면 0번.
#[test]
fn mashing_the_flask_sips_one_at_a_time() {
    let t = Tuning::default();
    let ticks = 180;
    let most = (ticks as f32 * TICK_DT / (t[P::FlaskTime] + t[P::FlaskLock])).ceil() as usize;
    for press in 1..=9usize {
        for gap in 2..=14usize {
            for full in [false, true] {
                let mut w = World::new("test", 20.0);
                w.player.flask = 10;
                let (mut starts, mut gulps, mut was) = (0, 0, false);
                for i in 0..ticks {
                    if !full {
                        w.player.hp = 10; // 늘 다쳐 있다(급한 상황)
                    }
                    let k = i % (press + gap);
                    let ev = w.step(
                        &Actions {
                            flask: k == 0,
                            ..Default::default()
                        },
                        &t,
                    );
                    gulps += ev
                        .iter()
                        .filter(|e| matches!(e, Event::Gulp { .. }))
                        .count();
                    let now = matches!(w.player.state, PState::Drink { .. });
                    starts += (now && !was) as usize;
                    was = now;
                }
                let tag = format!("press {press} gap {gap} full {full}");
                if full {
                    assert_eq!(starts, 0, "{tag}");
                } else {
                    assert!(starts >= 1 && starts <= most, "{tag}: {starts} > {most}");
                    // 끝까지 간 마시기는 모두 삼켰다(다시 누르기로 끊기지 않는다) — 마지막 것은 아직 들고 있을 수 있다.
                    assert!(
                        gulps + 1 >= starts && gulps <= starts,
                        "{tag}: {starts} starts {gulps} gulps"
                    );
                }
            }
        }
    }
}

/// 0031: 삼키기 전 회피는 끊기 — 등잔·체력 그대로(어깨 버튼 실수를 무른다). 삼킨 뒤 회피는 안 먹는다.
#[test]
fn a_dodge_before_the_gulp_takes_the_flask_back() {
    let t = Tuning::default();
    let mut w = World::new("test", 20.0);
    w.player.hp = 10;
    drink(&mut w, &t, 5);
    w.player.dodge_cd = 0.0;
    w.step(
        &Actions {
            dodge: true,
            ..Default::default()
        },
        &t,
    );
    assert!(matches!(w.player.state, PState::Dodge { .. }));
    assert_eq!((w.player.flask, w.player.hp), (FLASK_CHARGES, 10));
    // 0057: 삼킨 뒤에는 내리는 동작(후딜레이)을 회피로 끊는다 — 등잔·체력은 이미 바뀌었다(0031의 "삼킨 뒤 회피는 안 먹는다"를 바꿈).
    let mut w = World::new("test", 20.0);
    w.player.hp = 10;
    drink(&mut w, &t, (t[P::FlaskRaise] / TICK_DT) as usize + 3);
    w.player.dodge_cd = 0.0;
    w.step(
        &Actions {
            dodge: true,
            ..Default::default()
        },
        &t,
    );
    assert!(matches!(w.player.state, PState::Dodge { .. }));
    assert_eq!(
        (w.player.flask, w.player.hp),
        (FLASK_CHARGES - 1, 10 + t[P::FlaskHeal].round() as i32)
    );
}

/// 0031: 삼키기 전에 맞으면 끊기고 등잔 그대로(대가는 맞은 피해와 시간).
#[test]
fn a_hit_before_the_gulp_keeps_the_flask() {
    let t = Tuning::default();
    let mut w = mob_swinging_in_front(&t);
    w.player.hp = 30;
    let mut hurt = drink(&mut w, &t, 0).contains(&Event::PlayerHurt);
    for _ in 0..((t[P::FlaskRaise] / TICK_DT) as usize - 2) {
        if hurt {
            break;
        }
        hurt |= w.step(&Actions::default(), &t).contains(&Event::PlayerHurt);
    }
    assert!(hurt, "the blow landed before the gulp");
    assert!(!matches!(w.player.state, PState::Drink { .. }));
    assert_eq!((w.player.flask, w.player.hp), (FLASK_CHARGES, 20));
}
