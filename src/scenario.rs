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
    Elites,
    /// 떠난 마을의 새 적(0120): 등딱지짐승·꼬투리 둘·청소자.
    Newcomers,
    /// 깨진 종지기(0125): 울림 고리·종 조각·돌진.
    Bellkeeper,
    /// 보스 2단계(0133): 문지기 체력 반 — 시작하자마자 전환, 이어 뿌리 기둥 줄·돌진 뒤 내려치기.
    WardenTwo,
    /// 보스 2단계(0133): 종지기 체력 반 — 시작하자마자 전환, 이어 종 내던지기·돌진 뒤 울림.
    BellkeeperTwo,
    /// 스킬 연습(0151 S0 판정): 두 겹으로 둘러싼 떼 + 큰 기술 게이지 가득 — "모으고 → 쓸고"와 쟁반 뒤집기를 바로.
    Skills,
}

pub struct Entry {
    pub scenario: Scenario,
    pub name: &'static str,
    /// 무엇을 보는 시나리오인가(PoC 문서의 번호).
    pub checks: &'static str,
}

pub const LIST: [Entry; 14] = [
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
    Entry {
        scenario: Scenario::Elites,
        name: "elites",
        checks: "0112 elite affixes: hardened, repeating, lingering",
    },
    Entry {
        scenario: Scenario::Newcomers,
        name: "shellback + pods + scavenger",
        checks:
            "0120 guard (front light hits bounce), planted discs, hop away, scavenger ignores you",
    },
    Entry {
        scenario: Scenario::Bellkeeper,
        name: "broken bellkeeper",
        checks:
            "0125 toll rings (jump or dodge through), shard fan (parry sends it back), bell charge",
    },
    Entry {
        scenario: Scenario::WardenTwo,
        name: "warden phase two",
        checks: "0133 rise at once (untouchable, burst push), root pillar row, charge then slam",
    },
    Entry {
        scenario: Scenario::BellkeeperTwo,
        name: "bellkeeper phase two",
        checks: "0133 rise at once (untouchable, burst push), bell hurled to your spot + ring, charge then toll",
    },
    Entry {
        scenario: Scenario::Skills,
        name: "skills: horde 24 + full gauge",
        checks: "0151 gather then sweep, momentum (kills bring skills back), both buttons = great skill",
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
        Scenario::Elites => {
            // 엘리트 셋(0112)을 따로 떨어뜨려: 굳은 잔재(앞)·되풀이 가시짐승(오른쪽, 돌진 거리 밖)·남는 잔재(왼쪽).
            use crate::enemy::Affix;
            use crate::species::Kind;
            let mut w = World::new("elites", 14.0);
            for (kind, at, affix) in [
                (Kind::Husk, vec3(0.0, 0.0, -5.0), Affix::Hardened),
                (Kind::Thorn, vec3(9.0, 0.0, 2.0), Affix::Repeating),
                (Kind::Husk, vec3(-7.0, 0.0, 3.0), Affix::Lingering),
            ] {
                let i = w.spawn(kind, at);
                w.enemies[i].make_elite(affix);
            }
            w
        }
        Scenario::Bellkeeper => {
            let mut w = World::new("bellkeeper arena", 16.0);
            w.spawn(crate::species::Kind::Bellkeeper, vec3(0.0, 0.0, -9.0));
            w.obstacles.push(pillar(-10.0, -10.0, 1.6, 4.0));
            w.obstacles.push(pillar(10.0, -10.0, 1.6, 4.0));
            w
        }
        // 2단계(0133): 같은 아레나, 체력만 반 — 시작하자마자 전환(잠든 문지기는 깨어난 뒤).
        Scenario::WardenTwo | Scenario::BellkeeperTwo => {
            let mut w = build(if s == Scenario::WardenTwo {
                Scenario::Boss
            } else {
                Scenario::Bellkeeper
            });
            for en in &mut w.enemies {
                if en.boss {
                    en.hp = en.max_hp / 2;
                }
            }
            w
        }
        Scenario::Newcomers => {
            // 0120: 앞의 등딱지짐승(이쪽을 본다)·먼 꼬투리 둘·뒤의 청소자(제자리에서 먹는다 — 쳐야 싸운다).
            use crate::species::Kind;
            let mut w = World::new("newcomers", 16.0);
            let s = w.spawn(Kind::Shell, vec3(0.0, 0.0, -5.0));
            w.enemies[s].facing = 0.0;
            w.spawn(Kind::Pod, vec3(6.0, 0.0, -9.0));
            w.spawn(Kind::Pod, vec3(-6.0, 0.0, -9.0));
            let e = w.spawn(Kind::Husk, vec3(0.0, 0.0, 8.0));
            w.enemies[e].place(std::f32::consts::PI);
            w.enemies[e].docile = true;
            w
        }
        Scenario::Skills => {
            // 두 겹의 떼(8m·11m — 다가오기까지 숨 돌릴 틈) — 벽이 가까워(14m) 큰 기술로 쏟긴 적이 박힌다. 게이지는 가득.
            let mut w = World::new("skills", 14.0);
            for (n, r) in [(10, 8.0), (14, 11.0)] {
                for i in 0..n {
                    let a = (i as f32 + 0.5) / n as f32 * std::f32::consts::TAU;
                    w.spawn_mob(vec3(a.cos() * r, 0.0, a.sin() * r));
                }
            }
            w.player.gauge = 1.0;
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
        let full = crate::progression::Tree::full();
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
