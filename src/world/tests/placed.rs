//! 방랑자가 놓거나 던진 것 치우기(0278 고침 — `World::clear_placed`, 쉬기 · 쓰러져 다시 서기가 부른다).
use super::golden::{arm_for, arm_for_ult, yard};
use super::*;
use crate::skills::{Loadout, Pick, Slot, Ult, IDS, MAX_RANK, ULTS, ULT_MAX_RANK};

/// 가짓수의 이름(`placed_counts`의 차례).
const KINDS: [&str; 11] = [
    "bolts", "smites", "bombs", "blasts", "cracks", "sparks", "bursts", "zones", "blades",
    "fields", "spikes",
];

/// 고정 마당에서 쓰는 것 하나: (이름, 준비된 월드, 첫 틱의 누름, 그 뒤 쥐고 있는 틱 수 — 쥐는 것은 첫 누름의 쥐고 있음).
type Case = (String, World, Actions, usize);

fn cases() -> Vec<Case> {
    let mut out = Vec::new();
    for (id, pick) in IDS
        .into_iter()
        .flat_map(|id| [(id, Pick::A), (id, Pick::B)])
    {
        let mut w = yard();
        arm_for(&mut w, id);
        let other = if id == crate::skills::Id::Whirl {
            crate::skills::Id::Cut
        } else {
            crate::skills::Id::Whirl
        };
        w.set_skills(Loadout {
            slots: [
                Slot {
                    id,
                    rank: MAX_RANK,
                    pick,
                },
                Slot {
                    id: other,
                    rank: 1,
                    pick: Pick::A,
                },
            ],
            ult: 1,
            great: Ult::Upend,
        });
        let a = Actions {
            skill1: true,
            ..Default::default()
        };
        out.push((format!("skill {id:?} {pick:?}"), w, a, 0));
    }
    for u in ULTS {
        let mut w = yard();
        arm_for_ult(&mut w, u);
        w.set_skills(Loadout {
            slots: [crate::skills::Id::Whirl, crate::skills::Id::Cut].map(|id| Slot {
                id,
                rank: 1,
                pick: Pick::A,
            }),
            ult: ULT_MAX_RANK,
            great: u,
        });
        w.player.skill.gauge = 1.0;
        let a = Actions {
            ult: true,
            ..Default::default()
        };
        out.push((format!("ult {u:?}"), w, a, 0));
    }
    for arm in weapons::IDS {
        // RB(던진 창 · 신성한 빛 · 땅 내려찍기 …) · 평타(쇠뇌 · 활은 수류탄 — 터지면 폭발 고리) · 쥐었다 떼기(망치의 강타는 금 간 땅).
        for (what, a, held) in [
            (
                "rb",
                Actions {
                    cast: true,
                    cast_held: true,
                    ..Default::default()
                },
                0,
            ),
            (
                "attack",
                Actions {
                    attack: true,
                    ..Default::default()
                },
                0,
            ),
            (
                "heavy",
                Actions {
                    attack: true,
                    attack_held: true,
                    ..Default::default()
                },
                90,
            ),
        ] {
            let mut w = yard();
            w.set_weapon(arm);
            out.push((format!("{what} {arm:?}"), w, a, held));
        }
    }
    out
}

/// 스킬 · 궁극기 · 무기의 RB와 평타를 쓰고 몇 틱 뒤에 치우면, 그때 놓여 있던 것이 무엇이든 하나도 남지 않고 던진 창 · 방패 · 무기는
/// 손에 있다 — 모든 가짓수가 적어도 한 번은 치울 때 놓여 있었다(빠진 가짓수가 없게).
#[test]
fn clearing_what_the_wanderer_placed_leaves_nothing_and_brings_thrown_weapons_home() {
    let t = Tuning::default();
    let mut cleared = [false; 11];
    let mut came_home = false;
    for after in [6, 24, 50, 100, 150] {
        for (name, mut w, first, held) in cases() {
            let hold = Actions {
                attack_held: first.attack_held,
                cast_held: first.cast_held,
                ..Default::default()
            };
            for k in 0..after + held {
                let a = if k == 0 {
                    first
                } else if k < held {
                    hold
                } else {
                    Actions::default()
                };
                w.step(&a, &t);
            }
            let before = w.placed_counts();
            let away = w.rb_out() || w.weapon_away();
            w.clear_placed();
            for (i, n) in before.iter().enumerate() {
                cleared[i] |= *n > 0;
            }
            assert_eq!(w.placed_counts(), [0; 11], "{name} after {after} ticks");
            assert!(!w.rb_out() && !w.shield_out() && !w.weapon_away(), "{name}");
            came_home |= away;
            // 치운 뒤 한 틱이 지나도 지난 것이 다시 나오지 않는다(남은 쪽지가 새로 세우지 않는다).
            w.step(&Actions::default(), &t);
            let after_step = w.placed_counts();
            assert_eq!(
                after_step[..5],
                [0; 5],
                "{name}: nothing thrown comes back after clearing"
            );
        }
    }
    let missed: Vec<&str> = KINDS
        .iter()
        .zip(cleared)
        .filter(|(_, c)| !c)
        .map(|(k, _)| *k)
        .collect();
    assert!(missed.is_empty(), "never placed in these cases: {missed:?}");
    assert!(came_home, "some case threw a weapon that had to come home");
}
