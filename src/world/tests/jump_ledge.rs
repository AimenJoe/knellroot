//! 점프·내려찍기와 높낮이: 턱·벼랑·경사로, 적의 턱 뛰기, 다른 층끼리는 안 닿는다, 기둥, 점프 버튼의 상호작용.
use super::*;

/// 0027: 점프는 체공 시간 뒤에 내려앉고(포물선), 스태미나를 쓰고, 공중에서 또 뛸 수 없다.
#[test]
fn a_jump_rises_and_lands_after_its_air_time() {
    let t = Tuning::default();
    let mut w = World::new("test", 20.0);
    let st = w.player.stamina;
    w.step(
        &Actions {
            jump: true,
            ..Default::default()
        },
        &t,
    );
    assert!(w.player.airborne());
    assert!(w.player.stamina < st);
    let mut peak = 0.0f32;
    let mut ticks = 1;
    while w.player.airborne() && ticks < 200 {
        // 공중에서 또 눌러도 다시 뛰지 않는다.
        w.step(
            &Actions {
                jump: true,
                ..Default::default()
            },
            &t,
        );
        peak = peak.max(w.player.air);
        ticks += 1;
    }
    let air = ticks as f32 * TICK_DT;
    assert!((peak - t[P::JumpHeight]).abs() < 0.05, "peak {peak}");
    assert!((air - t[P::JumpTime]).abs() < 0.05, "air {air}s");
    assert!(matches!(w.player.state, PState::Land { .. }));
    assert_eq!(w.player.air, 0.0);
}

#[test]
fn a_running_jump_carries_its_speed() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    w.view_yaw = Some(0.0);
    let run = Actions {
        move_dir: vec2(0.0, 1.0),
        ..Default::default()
    };
    for _ in 0..60 {
        w.step(&run, &t);
    }
    let v = w.player.vel.length();
    let z0 = w.player.pos.z;
    w.step(&Actions { jump: true, ..run }, &t);
    // 손을 놓아도 공중에서는 거의 그대로 간다.
    while w.player.airborne() {
        w.step(&Actions::default(), &t);
    }
    let flew = z0 - w.player.pos.z;
    assert!(flew > v * t[P::JumpTime] * 0.6, "flew {flew} m at {v} m/s");
}

/// 높이 떠 있으면 바닥을 훑는 뿌리 파동을 넘는다(엘든 링: 점프로 낮은 공격 피하기). 땅에 있으면 맞는다.
#[test]
fn a_high_jump_clears_the_root_wave() {
    let t = Tuning::default();
    for jump in [true, false] {
        let mut w = world_with_boss();
        let en = &mut w.enemies[0];
        en.pos = vec3(0.0, 0.0, -3.0);
        en.atk = Atk::Wave;
        en.state = EState::Wave { t: 0.0 };
        en.landed = false;
        let mut hurt = false;
        for i in 0..40 {
            // 앞(-3m)에서 퍼지는 파동이 닿기 조금 전(9 m/s → 약 0.3초)에 뛴다.
            let a = Actions {
                jump: jump && i == 8,
                ..Default::default()
            };
            hurt |= w.step(&a, &t).contains(&Event::PlayerHurt);
        }
        assert_eq!(hurt, !jump, "jump {jump}");
    }
}

/// 공중에서 공격 = 내려찍기: 내려앉는 순간 둘레의 적을 `plunge.damage`만큼.
#[test]
fn a_plunge_hits_everyone_around_the_landing_spot() {
    let t = Tuning::default();
    let mut w = World::new("test", 20.0);
    w.spawn_mob(vec3(1.0, 0.0, 0.0));
    w.spawn_mob(vec3(-1.0, 0.0, 0.3));
    w.spawn_mob(vec3(0.0, 0.0, 6.0)); // 멀다
    idle(&mut w);
    w.step(
        &Actions {
            jump: true,
            ..Default::default()
        },
        &t,
    );
    w.step(
        &Actions {
            attack: true,
            ..Default::default()
        },
        &t,
    );
    let mut ev = Vec::new();
    for _ in 0..120 {
        idle(&mut w);
        ev.extend(w.step(&Actions::default(), &t));
    }
    assert!(ev.iter().any(|e| matches!(e, Event::Plunge { .. })));
    let dmg = t[P::PlungeDamage] as i32;
    assert_eq!(w.enemies[0].hp, w.enemies[0].max_hp - dmg);
    assert_eq!(w.enemies[1].hp, w.enemies[1].max_hp - dmg);
    assert_eq!(w.enemies[2].hp, w.enemies[2].max_hp, "out of range");
}

/// 0048: 점프 버튼이 상호작용도 한다 — 스테이지가 "여기서 쉴 수 있다"고 적은 자리면 쉬기(뛰지 않음), 아니면 점프.
#[test]
fn the_jump_button_interacts_where_the_stage_allows() {
    let t = Tuning::default();
    let jump = Actions {
        jump: true,
        ..Default::default()
    };
    let mut w = World::new("test", 30.0);
    w.interact_here = true;
    w.step(&jump, &t);
    assert!(w.interact);
    assert!(!matches!(w.player.state, PState::Jump { .. }));
    let mut w = World::new("test", 30.0);
    w.step(&jump, &t);
    assert!(!w.interact);
    assert!(matches!(w.player.state, PState::Jump { .. }));
}

/// 0047: 뛰어 있는 동안에도 스틱 쪽으로 몸을 돌린다(공중에서 내려찍기를 겨눈다). 내려찍기가 시작되면 고정.
#[test]
fn a_jump_turns_toward_the_stick() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    w.view_yaw = Some(0.0);
    let f0 = w.player.facing;
    w.step(
        &Actions {
            jump: true,
            ..Default::default()
        },
        &t,
    );
    let side = Actions {
        move_dir: vec2(1.0, 0.0),
        ..Default::default()
    };
    for _ in 0..20 {
        w.step(&side, &t);
    }
    assert!(w.player.airborne());
    let turned = angle_diff(w.player.facing, f0).abs();
    assert!(turned > 1.0, "turned {turned} rad in the air");
}

/// 높낮이 시험 마당(0043): 5×6칸(2m). 맨 윗줄(가장 -z)은 높이 `ledge`인 단, 나머지는 바닥 0.
fn ledge_yard(ledge: f32) -> World {
    use crate::ground::Cell::Flat;
    let mut cells = vec![Flat(0.0); 5 * 6];
    for c in cells.iter_mut().take(5) {
        *c = Flat(ledge);
    }
    let mut w = World::new("test", 30.0);
    w.ground = Some(Ground::new(5, 6, 2.0, cells));
    w.view_yaw = Some(0.0);
    w.player.pos = vec3(0.0, 0.0, 3.0);
    w
}

/// 걸어서는 0.5m 턱에 막히고, 달려와 뛰면 올라선다(사장: "경사로 + 낮은 턱 점프").
#[test]
fn a_low_ledge_blocks_walking_and_a_jump_climbs_it() {
    let t = Tuning::default();
    let mut w = ledge_yard(0.5);
    for _ in 0..150 {
        w.step(&up(), &t);
    }
    assert_eq!(w.player.pos.y, 0.0, "walking stays below");
    let edge = -6.0 + 2.0; // 단 칸의 아래 변(z)
    assert!(
        w.player.pos.z > edge,
        "stopped at the ledge: z {}",
        w.player.pos.z
    );
    // 조금 물러났다가 달려와 뛴다.
    w.player.pos.z = edge + 2.0;
    for _ in 0..10 {
        w.step(&up(), &t);
    }
    w.step(&Actions { jump: true, ..up() }, &t);
    while w.player.airborne() {
        w.step(&up(), &t);
    }
    // 내려앉은 뒤에는 손을 놓는다(시험 격자 밖으로 걸어 나가지 않게).
    for _ in 0..30 {
        w.step(&Actions::default(), &t);
    }
    assert!(
        (w.player.pos.y - 0.5).abs() < 1e-4 && w.player.air == 0.0,
        "stands on the ledge: y {} air {}",
        w.player.pos.y,
        w.player.air
    );
    assert!(w.player.pos.z < edge);
}

/// 1m 벼랑은 점프(0.9m)로도 못 오른다.
#[test]
fn a_tall_cliff_cannot_be_jumped() {
    let t = Tuning::default();
    let mut w = ledge_yard(1.0);
    let edge = -4.0;
    w.player.pos.z = edge + 2.0;
    for _ in 0..10 {
        w.step(&up(), &t);
    }
    w.step(&Actions { jump: true, ..up() }, &t);
    while w.player.airborne() {
        w.step(&up(), &t);
    }
    assert_eq!(w.player.pos.y, 0.0);
    assert!(w.player.pos.z > edge);
}

/// 단 위에서 걸어 나가면 떨어져 아래에 선다(몸 높이는 이어진다).
#[test]
fn walking_off_a_ledge_falls_and_lands_below() {
    let t = Tuning::default();
    let mut w = ledge_yard(1.0);
    w.player.pos = vec3(0.0, 1.0, -5.0);
    w.step(&Actions::default(), &t);
    assert_eq!(w.player.pos.y, 1.0);
    let down = Actions {
        move_dir: vec2(0.0, -1.0),
        ..Default::default()
    };
    let mut flew = false;
    let mut lowest_body = f32::MAX;
    for _ in 0..90 {
        let body = w.player.pos.y + w.player.air;
        w.step(&down, &t);
        let now = w.player.pos.y + w.player.air;
        assert!(
            now <= body + 1e-4 && body - now < 0.5,
            "body height is continuous: {body} → {now}"
        );
        flew |= w.player.air > 0.2;
        lowest_body = lowest_body.min(now);
    }
    assert!(flew, "fell through the air");
    assert_eq!((w.player.pos.y, w.player.air), (0.0, 0.0));
}

/// 0100(사장 "적들도 높은 곳으로 점프하거나 낮은 곳으로 뛰어내릴 수 있어야"): 경사로 없는 1m 단 위의 플레이어에게 아래 잔재가
/// 뛰어올라 와 친다. 단 위의 잔재는 아래 플레이어에게 뛰어내려 온다. 뛰는 동안은 치지 않는다.
#[test]
fn enemies_hop_up_and_drop_down_ledges_to_reach_you() {
    let t = Tuning::default();
    // 뛰어오르기.
    let mut w = ledge_yard(1.0);
    w.player.pos = vec3(0.0, 1.0, -5.0);
    let i = w.spawn(Kind::Husk, vec3(0.0, 0.0, 3.0));
    let (mut hopped, mut on_top) = (false, false);
    for _ in 0..900 {
        w.step(&Actions::default(), &t);
        if let Some(h) = w.enemies[i].hop {
            hopped = true;
            assert!(h.to.y > h.from.y + 0.9, "up the 1m ledge: {h:?}");
        }
        on_top |= w.enemies[i].hop.is_none() && (w.enemies[i].pos.y - 1.0).abs() < 1e-3;
        if w.player.hp < PLAYER_HP {
            break;
        }
    }
    assert!(hopped && on_top, "hopped up: {:?}", w.enemies[i].pos);
    assert!(w.player.hp < PLAYER_HP, "and reached the player");
    // 뛰어내리기.
    let mut w = ledge_yard(1.0);
    w.player.pos = vec3(0.0, 0.0, 3.0);
    let i = w.spawn(Kind::Husk, vec3(0.0, 1.0, -5.0));
    let mut dropped = false;
    for _ in 0..900 {
        w.step(&Actions::default(), &t);
        if let Some(h) = w.enemies[i].hop {
            assert!(h.to.y < h.from.y - 0.9, "down: {h:?}");
            dropped = true;
        }
        if w.player.hp < PLAYER_HP {
            break;
        }
    }
    assert!(dropped, "dropped down: {:?}", w.enemies[i].pos);
    assert!(w.player.hp < PLAYER_HP, "and reached the player");
}

/// 0100: 뛰는 포물선은 두 끝을 지나고, 오를 때는 턱 위로 넘어간다(가운데 높이가 위 끝보다 높다).
#[test]
fn a_hop_arcs_over_the_ledge_and_lands_where_it_aimed() {
    let h = crate::enemy::Hop::new(vec3(0.0, 0.0, 0.0), vec3(0.0, 1.0, -1.2));
    assert_eq!(h.at(0.0), h.from);
    assert!((h.at(1.0) - h.to).length() < 1e-5);
    assert!(h.at(0.5).y > 1.0, "clears the lip: {:?}", h.at(0.5));
    let d = crate::enemy::Hop::new(vec3(0.0, 2.0, 0.0), vec3(0.0, 0.0, 1.0));
    assert!(d.dur < h.dur, "dropping is quicker");
}

/// 높이가 다른 단: 아래 잔재의 발톱도, 플레이어의 칼도 닿지 않는다. 뛰어오를 수 없는 높이(1.5m — `ENEMY_CLIMB` 1m 위)면
/// 경사로가 없을 때 적은 아래에서 기다린다(0100 전에는 1m 단도 그랬다).
#[test]
fn different_levels_do_not_trade_blows() {
    let t = Tuning::default();
    // 아래 잔재는 1.5m 단 위에 가만히 선 플레이어를 못 친다.
    let mut w = ledge_yard(1.5);
    w.player.pos = vec3(0.0, 1.5, -4.4);
    let i = w.spawn(Kind::Husk, vec3(0.0, 0.0, -3.0));
    for _ in 0..400 {
        w.step(&Actions::default(), &t);
    }
    assert_eq!(w.player.hp, PLAYER_HP);
    assert!(
        w.enemies[i].pos.y == 0.0 && w.enemies[i].pos.z > -4.0,
        "the husk stays below: {:?}",
        w.enemies[i].pos
    );
    // 단 끝에서 아래 잔재 쪽으로 휘둘러도 안 닿는다(수평으로는 닿는 거리).
    let mut w = ledge_yard(1.0);
    w.player.pos = vec3(0.0, 1.0, -4.5);
    w.player.facing = 0.0;
    let i = w.spawn(Kind::Husk, vec3(0.0, 0.0, -3.2));
    w.enemies[i].state = EState::Dormant;
    w.step(
        &Actions {
            attack: true,
            ..Default::default()
        },
        &t,
    );
    for _ in 0..40 {
        w.step(&Actions::default(), &t);
    }
    assert_eq!(w.enemies[i].hp, w.enemies[i].max_hp);
}

/// 경사로가 있으면 적은 돌아서 단 위로 올라온다.
#[test]
fn enemies_take_the_ramp_up_to_the_player() {
    use crate::ground::Cell::{Flat, Ramp};
    let t = Tuning::default();
    let mut w = ledge_yard(1.0);
    // 맨 오른쪽 열 둘째 줄을 경사로(0 → 1, -z 쪽이 높다)로.
    if let Some(g) = &mut w.ground {
        g.cells[5 + 4] = Ramp {
            x_axis: false,
            from: 1.0,
            to: 0.0,
        };
        g.cells[4] = Flat(1.0);
    }
    w.player.pos = vec3(-4.0, 1.0, -5.0);
    let i = w.spawn(Kind::Husk, vec3(-4.0, 0.0, 3.0));
    let mut climbed = false;
    for _ in 0..900 {
        w.step(&Actions::default(), &t);
        climbed |= w.enemies[i].pos.y > 0.99 && w.enemies[i].pos.z < -4.0;
        if w.player.hp < PLAYER_HP {
            break;
        }
    }
    assert!(climbed, "climbed the ramp: {:?}", w.enemies[i].pos);
    assert!(w.player.hp < PLAYER_HP, "and reached the player");
}

#[test]
fn pillars_push_actors_out() {
    let b = Aabb {
        min: vec3(-1.0, 0.0, -1.0),
        max: vec3(1.0, 2.0, 1.0),
    };
    let p = b.push_circle(vec3(0.9, 0.0, 0.0), 0.3);
    assert!((p.x - 1.3).abs() < 1e-5, "{p:?}");
    assert_eq!(b.ray(vec3(0.0, 1.0, 5.0), vec3(0.0, 0.0, -1.0)), Some(4.0));
    assert_eq!(b.ray(vec3(3.0, 1.0, 5.0), vec3(0.0, 0.0, -1.0)), None);
}

/// 사장 2026-11-07 "적들의 길찾기가 좀 멍청함"(0076): 기둥이 사이에 있으면 박혀 비비지 않고 돌아와 붙는다.
#[test]
fn an_enemy_walks_around_a_pillar_to_reach_you() {
    let t = Tuning::default();
    let mut w = World::new("test", 12.0);
    w.obstacles.push(Aabb {
        min: vec3(-1.5, 0.0, -3.0),
        max: vec3(1.5, 3.0, -1.5),
    });
    w.spawn(Kind::Husk, vec3(0.0, 0.0, -6.0));
    let mut reached = None;
    for tick in 0..600 {
        w.step(&Actions::default(), &t);
        let en = &w.enemies[0];
        if matches!(en.state, EState::Tele { .. }) {
            reached = Some(tick);
            break;
        }
    }
    // 잔재 2.6m/s, 돌아가는 길 ≈ 7m → 3초 남짓.
    let tick = reached.expect("got to the player and wound up");
    assert!(tick < 240, "took {tick} ticks");
}
