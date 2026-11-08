//! 개발 시나리오(nail 규칙): 같은 상황에 몇 초 만에 다시. 시나리오 설명에 그것이 검증하는 PoC 항목을 적는다.
use macroquad::prelude::*;

use crate::world::{Aabb, World};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scenario {
    Boss,
    Horde,
    Pillars,
    Empty,
    Tray,
    Stress40,
    Stress80,
    Duel,
}

pub struct Entry {
    pub scenario: Scenario,
    pub name: &'static str,
    /// 무엇을 보는 시나리오인가(PoC 문서의 번호).
    pub checks: &'static str,
}

pub const LIST: [Entry; 8] = [
    Entry {
        scenario: Scenario::Boss,
        name: "boss arena",
        checks: "P2 camera switch, P3 lock-on move, boss telegraph",
    },
    Entry {
        scenario: Scenario::Horde,
        name: "horde 20",
        checks: "P1 quarter-view readability, draw cost, crowd feel",
    },
    Entry {
        scenario: Scenario::Pillars,
        name: "pillars + boss",
        checks: "P2 camera collision (pull-in), occlusion",
    },
    Entry {
        scenario: Scenario::Empty,
        name: "empty",
        checks: "movement lab (speed, accel, turn, dodge)",
    },
    Entry {
        scenario: Scenario::Tray,
        name: "tray spill",
        checks: "P6 turn the ground: spill, stagger, wall slam, no sickness",
    },
    Entry {
        scenario: Scenario::Stress40,
        name: "stress 40",
        checks: "P5 perf: 40 mobs, cpu ms / draws (F3)",
    },
    Entry {
        scenario: Scenario::Stress80,
        name: "stress 80",
        checks: "P5 perf: 80 mobs, cpu ms / draws (F3)",
    },
    Entry {
        scenario: Scenario::Duel,
        name: "husk + thorn",
        checks: "0073 windup poses one by one (rise, hold, snap; rear-up stomp)",
    },
];

/// 지금 시나리오의 줄(HUD가 이름·검증 항목을 보인다).
pub fn entry(s: Scenario) -> &'static Entry {
    LIST.iter().find(|e| e.scenario == s).unwrap_or(&LIST[0])
}

#[cfg(feature = "dev")]
pub fn next(s: Scenario) -> Scenario {
    let i = LIST.iter().position(|e| e.scenario == s).unwrap_or(0);
    LIST[(i + 1) % LIST.len()].scenario
}

#[cfg(feature = "dev")]
pub fn by_number(n: u8) -> Option<Scenario> {
    LIST.get(n.saturating_sub(1) as usize).map(|e| e.scenario)
}

fn pillar(x: f32, z: f32, w: f32, h: f32) -> Aabb {
    Aabb {
        min: vec3(x - w * 0.5, 0.0, z - w * 0.5),
        max: vec3(x + w * 0.5, h, z + w * 0.5),
    }
}

pub fn build(s: Scenario) -> World {
    match s {
        Scenario::Boss => {
            let mut w = World::new("boss arena", 14.0);
            w.spawn_boss(vec3(0.0, 0.0, -8.0));
            w.obstacles.push(pillar(-9.0, -9.0, 1.6, 4.0));
            w.obstacles.push(pillar(9.0, -9.0, 1.6, 4.0));
            w
        }
        Scenario::Horde => {
            let mut w = World::new("horde", 16.0);
            for i in 0..20 {
                let a = i as f32 / 20.0 * std::f32::consts::TAU;
                let r = 9.0 + (i % 3) as f32 * 1.5;
                w.spawn_mob(vec3(a.cos() * r, 0.0, a.sin() * r));
            }
            w
        }
        Scenario::Pillars => {
            let mut w = World::new("pillars", 12.0);
            w.spawn_boss(vec3(0.0, 0.0, -7.0));
            for (x, z) in [
                (-4.0, -3.0),
                (4.0, -3.0),
                (-4.0, 3.0),
                (4.0, 3.0),
                (0.0, 6.0),
                (-8.0, 0.0),
                (8.0, 0.0),
            ] {
                w.obstacles.push(pillar(x, z, 1.2, 3.5));
            }
            w
        }
        Scenario::Empty => World::new("empty", 16.0),
        Scenario::Stress40 => stress(40),
        Scenario::Stress80 => stress(80),
        Scenario::Duel => {
            // 준비 동작 보기(0073): 잡몹 둘을 따로 떨어뜨려 하나씩 — 잔재(앞)·가시짐승(옆, 돌진 거리 밖에서 시작).
            let mut w = World::new("duel", 12.0);
            w.spawn(crate::species::Kind::Husk, vec3(0.0, 0.0, -4.0));
            w.spawn(crate::species::Kind::Thorn, vec3(8.0, 0.0, 2.0));
            w
        }
        Scenario::Tray => {
            // 둘러싸인 자리: 안쪽(3m)·가운데(5m)·바깥(7m) 고리 + 바깥 고리 곁 기둥 넷(쏟아진 적이 부딪히게).
            let mut w = World::new("tray", 12.0);
            for (n, r) in [(4, 3.0), (6, 5.0), (8, 7.0)] {
                for i in 0..n {
                    let a = (i as f32 + 0.5) / n as f32 * std::f32::consts::TAU;
                    w.spawn_mob(vec3(a.cos() * r, 0.0, a.sin() * r));
                }
            }
            for (x, z) in [(6.5, 6.5), (-6.5, 6.5), (6.5, -6.5), (-6.5, -6.5)] {
                w.obstacles.push(pillar(x, z, 1.4, 3.0));
            }
            w
        }
    }
}

/// 성능 시험(P5): 넓은 아레나에 잡몹 `n`마리를 고리 몇 겹으로, 기둥 넷. 쫓아와 뭉치는 것까지 재는 게 실제 싸움에 가깝다.
fn stress(n: usize) -> World {
    let mut w = World::new("stress", 22.0);
    let rings = n.div_ceil(20);
    for i in 0..n {
        let ring = i % rings;
        let k = i / rings;
        let per = n.div_ceil(rings);
        let a = (k as f32 + 0.5 * ring as f32) / per as f32 * std::f32::consts::TAU;
        let r = 8.0 + ring as f32 * 3.0;
        w.spawn_mob(vec3(a.cos() * r, 0.0, a.sin() * r));
    }
    for (x, z) in [(10.0, 10.0), (-10.0, 10.0), (10.0, -10.0), (-10.0, -10.0)] {
        w.obstacles.push(pillar(x, z, 1.6, 4.0));
    }
    w
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Actions;
    use crate::tuning::Tuning;

    /// 모든 시나리오가 600틱(10초) 동안 패닉 없이 돈다 — 소리·그림 핸들이 로직에 없어서 가능한 시험(nail 0169 R1).
    #[test]
    fn every_scenario_runs_ten_seconds_with_lock_and_attacks() {
        let t = Tuning::default();
        for e in LIST.iter() {
            let mut w = build(e.scenario);
            for i in 0..600 {
                let a = Actions {
                    move_dir: vec2(0.0, 1.0),
                    attack: i % 20 == 0,
                    lock: i == 30,
                    dodge: i % 90 == 0,
                    turn_right: i % 150 == 10,
                    turn_left: i % 150 == 80,
                    ..Default::default()
                };
                w.step(&a, &t);
            }
            assert!(
                w.player.pos.x.abs() <= w.arena && w.player.pos.z.abs() <= w.arena,
                "{}",
                e.name
            );
        }
    }

    /// 0107: 나무 15칸을 다 연 채로 시나리오마다 무기마다 오래 싸워도 깨지지 않고 아레나 안에 남는다(대시 공격이 적을 지나치는 칸 등).
    #[test]
    fn every_scenario_survives_a_full_tree() {
        let t = Tuning::default();
        let full = crate::progression::Tree {
            open: [crate::progression::DEPTH; 3],
            pin: None,
        };
        for e in LIST.iter() {
            for id in [
                crate::weapons::Id::Sword,
                crate::weapons::Id::Hammer,
                crate::weapons::Id::Fists,
            ] {
                let mut w = build(e.scenario);
                w.set_weapon(id);
                w.tree = full;
                for i in 0..900u32 {
                    let a = Actions {
                        move_dir: vec2(((i / 40) as f32).sin(), ((i / 40) as f32).cos()),
                        attack: i % 9 == 0,
                        attack_held: (i / 60) % 3 == 1,
                        dodge: i % 70 == 0,
                        dodge_held: (i / 50) % 4 == 2,
                        parry: i % 23 == 0,
                        jump: i % 110 == 5,
                        lock: i == 30,
                        turn_right: i % 240 == 10,
                        ..Default::default()
                    };
                    w.step(&a, &t);
                    assert!(
                        w.player.pos.x.abs() <= w.arena + 0.5
                            && w.player.pos.z.abs() <= w.arena + 0.5,
                        "{} {:?} tick {i}: {:?}",
                        e.name,
                        id,
                        w.player.pos
                    );
                    assert!(w.player.pos.is_finite() && w.player.stamina.is_finite());
                }
            }
        }
    }
}
