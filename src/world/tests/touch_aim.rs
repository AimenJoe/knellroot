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
            Form::Ring { .. }
                | Form::Brace { .. }
                | Form::Echo { .. }
                | Form::Field { .. }
                | Form::Wave { .. }
        );
        assert_eq!(form.aims(), !round, "{id:?}");
    }
}

/// 쇠뇌의 첫 볼트 쪽(수평).
fn crossbow_shot(w: &mut World, a: Actions) -> Vec3 {
    let t = Tuning::default();
    w.step(&Actions { cast: true, ..a }, &t);
    for _ in 0..30 {
        if !w.bolts.is_empty() {
            break;
        }
        w.step(&a, &t);
    }
    let b = w.bolts.first().expect("a bolt flies");
    vec3(b.dir.x, 0.0, b.dir.z).normalize()
}

/// 0219 패드 조준 보조: 스틱이 적에게서 5° 빗나가도 ±8° 안이면 그 적에게 붙는다. 보조가 없으면(마우스·터치 끌기) 스틱 그대로,
/// 원뿔 밖(20°)의 적은 찾지 않고, 스틱을 안 기울이면 바라보는 쪽(0213 그대로).
#[test]
fn a_pad_stick_snaps_to_a_foe_within_the_narrow_cone() {
    let off = |deg: f32| vec2(deg.to_radians().sin(), deg.to_radians().cos());
    let foe = vec3(0.0, 0.0, -10.0); // 앞(화면 위 = -z) 10m
    let shoot = |stick: Vec2, assist: f32| {
        let mut w = World::new("test", 40.0);
        w.set_weapon(weapons::Id::Crossbow);
        w.view_yaw = Some(0.0);
        w.spawn_mob(w.player.pos + foe);
        idle(&mut w);
        crossbow_shot(
            &mut w,
            Actions {
                aim: stick,
                aim_assist: assist,
                ..Default::default()
            },
        )
    };
    let d = shoot(off(5.0), 8.0);
    assert!(d.z < -0.999, "snapped onto the foe: {d:?}");
    let d = shoot(off(5.0), 0.0);
    assert!(
        (d.x - 5f32.to_radians().sin()).abs() < 0.01,
        "no assist: the stick as is {d:?}"
    );
    let d = shoot(off(20.0), 8.0);
    assert!(d.x > 0.3, "outside the cone: not pulled {d:?}");
    // 스틱을 안 기울이면 바라보는 쪽 — 적이 옆에 있어도 찾지 않는다.
    let mut w = World::new("test", 40.0);
    w.set_weapon(weapons::Id::Crossbow);
    w.view_yaw = Some(0.0);
    w.spawn_mob(w.player.pos + vec3(6.0, 0.0, 0.0));
    idle(&mut w);
    let face = w.player.facing_dir();
    let d = crossbow_shot(
        &mut w,
        Actions {
            aim_assist: 8.0,
            ..Default::default()
        },
    );
    assert!(d.dot(face) > 0.99, "no stick, no seeking (0213)");
}

/// 0219(사장 "숄더뷰에서의 원거리 무기 조준도 필요해 — 지금은 캐릭터가 움직이는 방향으로 쏘니까 이상함"): 숄더뷰의 원거리는
/// 카메라가 보는 쪽으로 — 옆으로 걸으며 쏴도 앞으로 간다. 근접의 자동 조준은 건드리지 않는다.
#[test]
fn in_the_shoulder_view_the_shot_goes_where_the_camera_looks() {
    let mut w = World::new("test", 40.0);
    w.set_weapon(weapons::Id::Crossbow);
    w.view_yaw = Some(0.0); // 카메라 앞 = 화면 위 = -z
    let t = Tuning::default();
    // 오른쪽으로 걸어 몸이 오른쪽을 본다.
    for _ in 0..20 {
        w.step(
            &Actions {
                move_dir: vec2(1.0, 0.0),
                aim_ahead: true,
                ..Default::default()
            },
            &t,
        );
    }
    assert!(w.player.facing_dir().x > 0.9, "walking right faces right");
    let d = crossbow_shot(
        &mut w,
        Actions {
            move_dir: vec2(1.0, 0.0),
            aim_ahead: true,
            ..Default::default()
        },
    );
    assert!(
        d.z < -0.99,
        "the camera's ahead, not the walking way: {d:?}"
    );
    assert!(w.aim_line.is_some(), "the reticle has a line to draw");
}

/// 0222(사장 "조준 전 바닥의 헤어라인 필요없음"): 쇠뇌도 손이 겨누지 않으면 겨누는 쪽이 없다(0221을 걷음) — 표시도 없다.
#[test]
fn nothing_held_means_no_aim_line_even_with_the_crossbow() {
    let t = Tuning::default();
    let mut w = World::new("test", 40.0);
    w.set_weapon(weapons::Id::Crossbow);
    w.step(&Actions::default(), &t);
    assert!(w.aim_line.is_none());
    w.step(
        &Actions {
            aim: vec2(1.0, 0.0),
            ..Default::default()
        },
        &t,
    );
    assert!(w.aim_line.is_some(), "the stick aims");
}

/// 숄더뷰 크로스헤어(0222)가 적을 겨눠도 그것은 원거리의 쪽일 뿐 — 몸은 걷는 쪽을 본다(0219 "근접의 쪽은 섞지 않는다").
/// 쏘면 그 적에게 간다.
#[test]
fn the_shoulder_crosshair_steers_the_shot_not_the_body() {
    let t = Tuning::default();
    let mut w = World::new("test", 40.0);
    w.set_weapon(weapons::Id::Crossbow);
    w.view_yaw = Some(0.0);
    let foe = w.player.pos + vec3(-3.0, 0.0, -8.0); // 앞 왼쪽
    w.spawn_mob(foe);
    let walk = Actions {
        move_dir: vec2(1.0, 0.0),
        aim_ahead: true,
        aim_sight: Some(vec2(foe.x, foe.z)),
        ..Default::default()
    };
    for _ in 0..30 {
        w.step(&walk, &t);
    }
    assert!(
        w.player.facing_dir().x > 0.9,
        "walking right still faces right: {:?}",
        w.player.facing_dir()
    );
    let d = crossbow_shot(&mut w, walk);
    let want = vec3(foe.x - w.player.pos.x, 0.0, foe.z - w.player.pos.z).normalize();
    assert!(
        d.dot(want) > 0.995,
        "the shot goes to the crosshair's foe: {d:?}"
    );
}

/// 0228(사장 "벽 너머의 적에게 조준 보정이 들어가는 건 아니지?"): 패드 보조(±8°)와 마법 탄의 가까운 적 찾기는 탄이 닿는 적에게만 —
/// 벽 너머의 적에게는 붙지 않는다. 벽이 없으면 그대로 붙는다.
#[test]
fn aim_help_skips_foes_behind_a_wall() {
    let t = Tuning::default();
    let wall = Aabb {
        min: vec3(-3.0, 0.0, -5.2),
        max: vec3(3.0, 4.0, -4.8),
    };
    let off = |deg: f32| vec2(deg.to_radians().sin(), deg.to_radians().cos());
    // 패드 보조: 앞 10m의 적, 스틱은 5° 빗나감.
    let assisted = |walled: bool| {
        let mut w = World::new("test", 40.0);
        w.set_weapon(weapons::Id::Crossbow);
        w.view_yaw = Some(0.0);
        if walled {
            w.obstacles.push(wall);
        }
        let foe = w.player.pos + vec3(0.0, 0.0, -10.0);
        w.spawn_mob(foe);
        idle(&mut w);
        let d = crossbow_shot(
            &mut w,
            Actions {
                aim: off(5.0),
                aim_assist: 8.0,
                ..Default::default()
            },
        );
        d.x.abs() < 0.01
    };
    assert!(assisted(false), "open ground: the stick snaps to the foe");
    assert!(
        !assisted(true),
        "a wall between: the stick stays where it points"
    );
    // 마법 탄의 찾기: 앞(벽 너머)의 가까운 적이 아니라 옆의 트인 적에게.
    let mut w = World::new("test", 40.0);
    w.set_weapon(weapons::Id::Staff);
    w.view_yaw = Some(0.0);
    w.obstacles.push(wall);
    let p = w.player.pos;
    w.spawn_mob(p + vec3(0.0, 0.0, -6.0)); // 벽 너머, 더 가깝다
    w.spawn_mob(p + vec3(7.0, 0.0, 0.0)); // 옆, 트였다
    idle(&mut w);
    assert!(!w.shot_clear(w.enemies[0].pos, w.enemies[0].radius, &t));
    assert!(w.shot_clear(w.enemies[1].pos, w.enemies[1].radius, &t));
    let d = w
        .nearest_shootable(t[P::CastRange], &t)
        .expect("the open foe");
    assert!(d.x > 0.99, "seeks the foe it can hit: {d:?}");
}
