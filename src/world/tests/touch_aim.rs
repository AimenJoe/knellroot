//! 터치 끌어 조준(0217 — 브롤스타즈 꼴): 끈 쪽은 락온보다 앞서고, 톡은 가까운 적, 쥐고 끄는 동안 모은다.
//! 버튼은 하나의 규칙 — 쏘기·수류탄·끼운 스킬(쪽이 있는 꼴) 어느 것이 올라가도 같다.
use super::*;
use crate::skills;

/// 끈 쪽(`aim` + `aim_drag`)은 락온 대상보다 앞선다 — 그냥 스틱 조준은 락온이면 쓰지 않는다(0014 그대로).
#[test]
fn a_touch_drag_aims_over_the_lock_on() {
    let t = Tuning::default();
    let cast_at = |drag: bool| {
        let mut w = World::new("test", 30.0);
        w.spawn_mob(vec3(0.0, 0.0, -4.0)); // 앞 — 락온이 고를 적
        w.spawn_mob(vec3(7.0, 0.0, 0.0)); // 오른쪽 — 손가락이 가리킨 적
        for e in &mut w.enemies {
            e.hp = 90;
        }
        idle(&mut w);
        press(
            &t,
            &mut w,
            Actions {
                lock: true,
                ..Default::default()
            },
            2,
        );
        assert!(w.lock.is_some());
        press(
            &t,
            &mut w,
            Actions {
                cast: true,
                aim: vec2(1.0, 0.0),
                aim_drag: drag,
                ..Default::default()
            },
            90,
        );
        (w.enemies[0].hp, w.enemies[1].hp)
    };
    let (front, right) = cast_at(true);
    assert!(front == 90 && right < 90, "dragged: the finger's way");
    let (front, right) = cast_at(false);
    assert!(
        front < 90 && right == 90,
        "a plain stick under lock-on: the target"
    );
}

/// 톡(끌지 않고 뗌)은 쇠뇌도 가까운 적에게 — 패드·키보드의 쇠뇌는 찾지 않는다(0213 그대로).
#[test]
fn a_touch_tap_seeks_the_nearest_even_with_the_crossbow() {
    let t = Tuning::default();
    let shot = |tap: bool| {
        let mut w = World::new("test", 30.0);
        w.set_weapon(weapons::Id::Crossbow);
        w.view_yaw = Some(0.0);
        w.spawn_mob(vec3(6.0, 0.0, 0.0)); // 옆 — 바라보는 쪽(-z)이 아니다
        idle(&mut w);
        w.step(
            &Actions {
                cast: true,
                auto_aim: tap,
                ..Default::default()
            },
            &t,
        );
        for _ in 0..30 {
            if !w.bolts.is_empty() {
                break;
            }
            w.step(&Actions::default(), &t);
        }
        let b = w.bolts.first().expect("a bolt flies");
        vec3(b.dir.x, 0.0, b.dir.z).normalize()
    };
    assert!(shot(true).x > 0.9, "tap: toward the foe beside");
    assert!(shot(false).z < -0.99, "no tap: where it faces");
}

/// 쏘는 버튼을 쥐고 끄는 동안(`aim_hold`)은 첫 발 없이 모으기에 들고, 몸이 끈 쪽으로 돈다 — 떼면 모은 탄이 그 쪽으로.
#[test]
fn holding_a_dragged_cast_charges_without_a_first_shot() {
    let t = Tuning::default();
    let mut w = World::new("test", 40.0);
    w.view_yaw = Some(0.0);
    let mana = w.player.ranged.mana;
    let held = Actions {
        cast_held: true,
        aim_hold: true,
        aim_drag: true,
        aim: vec2(1.0, 0.0),
        ..Default::default()
    };
    let full = ((t[P::CastHoldStart] + t[P::CastCharge]) / TICK_DT).ceil() as usize + 4;
    for _ in 0..full {
        w.step(&held, &t);
    }
    assert!(
        matches!(w.player.state, PState::CastCharge { .. }),
        "{:?}",
        w.player.state
    );
    assert!(w.bolts.is_empty(), "no tap shot first");
    assert!(w.player.facing_dir().x > 0.9, "turned to the finger");
    // 뗀다 — 앱은 모으는 중에는 누름 없이 쪽만 넘긴다.
    w.step(
        &Actions {
            aim: vec2(1.0, 0.0),
            aim_drag: true,
            ..Default::default()
        },
        &t,
    );
    let b = w.bolts.first().expect("the charged bolt");
    assert_eq!(b.def, weapons::Id::Sword.def().charged);
    assert!(b.dir.x > 0.9);
    assert!(w.player.ranged.mana < mana - 0.9, "one charge spent");
    // 쥐지 않은(패드·키보드의) 꾹은 그대로 — 첫 발을 쏜 뒤에 모은다(0056).
    let mut w = World::new("test", 40.0);
    w.step(
        &Actions {
            cast_held: true,
            ..Default::default()
        },
        &t,
    );
    for _ in 0..full {
        w.step(
            &Actions {
                cast_held: true,
                ..Default::default()
            },
            &t,
        );
    }
    assert!(
        matches!(w.player.state, PState::Idle),
        "no press, no charge"
    );
}

/// 수류탄도 같은 버튼 규칙: 끈 쪽으로 떨구고(락온이어도), 톡이면 가까운 적 쪽으로. 거리는 그대로 발 앞(0178).
#[test]
fn a_grenade_follows_the_drag_and_a_tap_drops_toward_the_nearest() {
    let t = Tuning::default();
    let g = weapons::Id::Crossbow.def().grenade.unwrap();
    let throw = |a: Actions| {
        let mut w = World::new("test", 40.0);
        w.set_weapon(weapons::Id::Crossbow);
        w.view_yaw = Some(0.0);
        w.spawn_mob(vec3(-5.0, 0.0, 0.0)); // 왼쪽
        idle(&mut w);
        let p0 = w.player.pos;
        w.step(&Actions { attack: true, ..a }, &t);
        match w.player.state {
            PState::Throw { to, .. } => vec2(to.x - p0.x, to.z - p0.z),
            ref s => panic!("{s:?}"),
        }
    };
    let d = throw(Actions {
        aim: vec2(1.0, 0.0),
        aim_drag: true,
        ..Default::default()
    });
    assert!(
        d.x > 0.0 && (d.length() - g.drop).abs() < 0.05,
        "dragged right: {d:?}"
    );
    let d = throw(Actions {
        auto_aim: true,
        ..Default::default()
    });
    assert!(
        d.x < -0.9 * g.drop,
        "tap: toward the foe on the left: {d:?}"
    );
}

/// 쪽이 있는 스킬도 같은 버튼 규칙: 줄 스킬(가르기)은 걷는 쪽보다 끈 쪽이 먼저(0217) — 끌지 않으면 걷는 쪽(0151 그대로).
#[test]
fn a_dragged_lane_skill_goes_the_finger_way_not_the_walking_way() {
    let t = Tuning::default();
    let dash = |drag: bool| {
        let mut w = World::new("test", 40.0);
        w.set_skills([skills::Id::Cut, skills::Id::Whirl]);
        w.view_yaw = Some(0.0);
        let p0 = w.player.pos;
        let a = Actions {
            skill1: true,
            move_dir: vec2(0.0, 1.0),
            aim: vec2(1.0, 0.0),
            aim_drag: drag,
            ..Default::default()
        };
        w.step(&a, &t);
        for _ in 0..40 {
            w.step(&Actions::default(), &t);
        }
        vec2(w.player.pos.x - p0.x, w.player.pos.z - p0.z)
    };
    let d = dash(true);
    assert!(d.x > 1.0 && d.x > d.y.abs() * 3.0, "dragged right: {d:?}");
    let d = dash(false);
    assert!(d.y.abs() > d.x.abs() * 3.0, "walking way: {d:?}");
}

/// 둥근 꼴(쟁반·버티기·되울림·불티)은 쪽이 없다 — 그 스킬이 끼워진 버튼은 끌지 않는다(톡 = 바로).
#[test]
fn only_skills_with_a_direction_take_the_drag() {
    use skills::Form;
    for s in &skills::SKILLS {
        let (id, form) = (s.id, s.form);
        let round = matches!(
            form,
            Form::Ring { .. } | Form::Brace { .. } | Form::Echo { .. } | Form::Field { .. }
        );
        assert_eq!(form.aims(), !round, "{id:?}");
    }
}
