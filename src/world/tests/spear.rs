//! 창(0272 2단계): 가장 긴 근접 거리 · 좁은 길을 꿰뚫는 찌르기 · 벽 너머로는 닿지 않음 · 옆에 붙은 적에게는 약함 ·
//! 내달려 꿰뚫는 강타(길 위의 적을 모두 — 벼랑 앞에서 멈춘다).
use super::*;
use crate::weapons::{Id as W, Shape};

/// 창을 든 월드, 적은 세워 둔 채(쫓지도 치지도 않게).
fn spear_world() -> World {
    let mut w = World::new("test", 20.0);
    w.set_weapon(W::Spear);
    w
}

/// 공격 한 번(누름) 뒤 `ticks`틱 — 맞은 적의 번호들.
fn thrust_once(w: &mut World, t: &Tuning, ticks: usize) -> Vec<usize> {
    let before: Vec<i32> = w.enemies.iter().map(|e| e.hp).collect();
    idle(w);
    let mut a = Actions {
        attack: true,
        ..Default::default()
    };
    for _ in 0..ticks {
        w.step(&a, t);
        a.attack = false;
        idle(w);
    }
    (0..w.enemies.len())
        .filter(|&i| w.enemies[i].hp < before[i])
        .collect()
}

/// 표: 창의 평타는 셋 다 곧은 길(좁다), 닿는 거리는 근접 무기 가운데 가장 길고(검 1.8 · 망치 2.1 · 창 2.7m 남짓), 빠르기는 검과 망치
/// 사이, 한 적에게의 피해는 검과 같다(빠르기가 느려 붙어 치면 검보다 덜 낸다). 강타·대시 공격도 길이다.
#[test]
fn the_spear_reaches_farthest_in_a_narrow_lane_between_sword_and_hammer_speed() {
    let t = Tuning::default();
    let reach = |id: W| {
        id.def()
            .combo
            .iter()
            .map(|h| t[P::AttackRange] * h.reach)
            .fold(0.0, f32::max)
    };
    let (sword, hammer, spear) = (reach(W::Sword), reach(W::Hammer), reach(W::Spear));
    assert!(spear > hammer.max(sword), "{sword} {hammer} {spear}");
    assert!((2.6..=2.8).contains(&spear), "{spear}");
    let s = W::Spear.def();
    for h in s.combo.iter().chain(s.dash.iter()) {
        let half = h.shape.lane_half().expect("every spear hit is a lane");
        assert!(half <= 0.5, "narrow: {half}");
        assert!((1.0..=1.4).contains(&h.time), "{}", h.time);
    }
    assert!(matches!(s.heavy.unwrap().shape, Shape::Lane { .. }));
    let dmg = |id: W| id.def().combo.iter().map(|h| h.damage).collect::<Vec<_>>();
    assert_eq!(dmg(W::Spear), dmg(W::Sword));
    let swing = |id: W| -> f32 {
        let w = id.def();
        (0..w.combo.len() as u8)
            .map(|k| {
                let (a, b, c) = hit_times(&t, w, k);
                a + b + c
            })
            .sum::<f32>()
            / w.combo.len() as f32
    };
    assert!(swing(W::Sword) < swing(W::Spear) && swing(W::Spear) < swing(W::Hammer));
    assert!(s.stamina > W::Sword.def().stamina && s.stamina < W::Hammer.def().stamina);
}

/// 한 번 찌르면 길 위의 적이 모두 맞는다(꿰뚫음) — 검은 맨 앞의 둘레만, 가장 먼 적(2.9m)에게는 검이 닿지 않는다.
#[test]
fn a_thrust_pierces_every_foe_on_the_line_and_outreaches_the_sword() {
    let t = Tuning::default();
    let line = [1.2, 2.0, 2.9];
    let mut w = spear_world();
    for z in line {
        w.spawn_mob(vec3(0.0, 0.0, -z));
    }
    assert_eq!(thrust_once(&mut w, &t, 40), vec![0, 1, 2]);
    let mut w = World::new("test", 20.0);
    for z in line {
        w.spawn_mob(vec3(0.0, 0.0, -z));
    }
    let hit = thrust_once(&mut w, &t, 40);
    assert!(!hit.contains(&2), "the sword does not reach 2.9 m: {hit:?}");
}

/// 좁다: 몸 곁(옆 0.8m)에 붙은 적에게는 찌르기가 닿지 않는다 — 검의 부채꼴은 닿는다. 에워싸였을 때의 약함(그 답은 장대 휩쓸기).
#[test]
fn a_thrust_misses_foes_pressed_at_the_side_that_a_sword_would_cut() {
    let t = Tuning::default();
    let side = [vec3(-0.8, 0.0, -1.1), vec3(0.8, 0.0, -1.1)];
    let mut w = spear_world();
    w.spawn_mob(vec3(0.0, 0.0, -1.0));
    for p in side {
        w.spawn_mob(p);
    }
    assert_eq!(thrust_once(&mut w, &t, 40), vec![0], "only the one ahead");
    let mut w = World::new("test", 20.0);
    w.spawn_mob(vec3(0.0, 0.0, -1.0));
    for p in side {
        w.spawn_mob(p);
    }
    assert!(
        thrust_once(&mut w, &t, 40).len() >= 2,
        "the sword's arc takes the side"
    );
}

/// 벽 너머로는 꿰뚫지 않는다: 길 위에 벽 상자가 서면 그 뒤의 적은 닿는 거리 안이어도 맞지 않는다(`lane_reach`).
#[test]
fn a_thrust_stops_at_a_wall() {
    let t = Tuning::default();
    let mut w = spear_world();
    w.spawn_mob(vec3(0.0, 0.0, -1.2));
    w.spawn_mob(vec3(0.0, 0.0, -2.6));
    w.obstacles.push(Aabb {
        min: vec3(-1.0, 0.0, -2.05),
        max: vec3(1.0, 3.0, -1.85),
    });
    assert_eq!(thrust_once(&mut w, &t, 40), vec![0]);
    let len = w.lane_reach(Vec3::ZERO, -Vec3::Z, 5.0);
    assert!((1.7..=1.86).contains(&len), "{len}");
    // 걸을 수 없는 칸(높은 단)도 막는다.
    let mut y = ledge_yard(1.0);
    y.player.pos = vec3(0.0, 0.0, -1.0);
    let len = y.lane_reach(y.player.pos, -Vec3::Z, 5.0);
    assert!(
        (2.85..=3.0).contains(&len),
        "the ledge face at z = -4: {len}"
    );
}

/// 강타 = 내달려 꿰뚫기: 시동 · 활성 동안 앞으로 3m 남짓 내달리며 지나는 길과 그 앞의 적을 모두 친다.
#[test]
fn the_heavy_lunge_travels_and_strikes_everyone_on_the_line() {
    let t = Tuning::default();
    let mut w = spear_world();
    for z in [1.5, 3.2, 4.6] {
        w.spawn_mob(vec3(0.0, 0.0, -z));
    }
    for e in &mut w.enemies {
        e.hp = 1000;
        e.max_hp = 1000;
    }
    let start = w.player.pos;
    idle(&mut w);
    let (st, ev) = hold_attack(&mut w, &t, 50, 60);
    assert!(st.iter().any(|s| matches!(s, PState::Heavy { .. })));
    // 강타의 타(`Smash`)가 셋 — 몸이 적을 뚫고 지나가며(내달리는 동안 적에게 막히지 않는다) 줄 끝의 적까지.
    let smash = ev
        .iter()
        .filter(|e| matches!(e, Event::Smash { .. }))
        .count();
    assert_eq!(smash, 3, "{ev:?}");
    let moved = (w.player.pos - start).length();
    assert!(moved > 2.0, "it lunges: {moved}");
}

/// 내달림은 벼랑 끝에서 멈춘다 — 높은 단에서 아래로 내달려도 떨어지지 않고 단 위에 선다(벽은 부딪힘이 막는다).
#[test]
fn the_heavy_lunge_stops_at_a_ledge() {
    let t = Tuning::default();
    let mut w = ledge_yard(1.0);
    w.set_weapon(W::Spear);
    // 단(맨 윗줄 z ∈ [-6, -4]) 위 가장자리 곁에서 아래(+z)를 본다.
    w.player.pos = vec3(0.0, 1.0, -4.8);
    w.player.facing = 0.0;
    let (st, _) = hold_attack(&mut w, &t, 50, 60);
    assert!(st.iter().any(|s| matches!(s, PState::Heavy { .. })));
    assert!(
        (w.player.pos.y - 1.0).abs() < 1e-3 && w.player.air < 1e-3,
        "still on the ledge: {:?} air {}",
        w.player.pos,
        w.player.air
    );
    // 같은 자리에서 검의 강타는 내딛기가 짧아 그대로 — 창만 멈추는 규칙이 있어야 하는 길이다(3m).
    assert!(t[P::HeavyLunge] * W::Spear.def().heavy.unwrap().lunge > 2.5);
}
