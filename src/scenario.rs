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
}

pub struct Entry {
    pub scenario: Scenario,
    pub name: &'static str,
    /// 무엇을 보는 시나리오인가(PoC 문서의 번호).
    pub checks: &'static str,
}

pub const LIST: [Entry; 5] = [
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
                    turn_cw: i % 150 == 10,
                    turn_ccw: i % 150 == 80,
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
}
