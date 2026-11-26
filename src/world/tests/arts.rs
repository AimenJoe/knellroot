//! 무기의 특색(0225): 한손 무기는 방패로 갈린다(방패 기술 · 결투 기술), 망치는 물리력(타입 없는 스킬의 피해·자세 피해),
//! 지팡이는 마법력(스킬이 쌓는 타입 게이지).
use super::*;
use crate::skills::{Id as Sk, Loadout, Pick, Slot};
use crate::status::Kind as Ty;
use crate::weapons::{Class, Grip, Id as W, ART_CHANNEL, ART_FORCE};

fn worn(w: &mut World, a: Sk, b: Sk) {
    w.set_skills(Loadout {
        slots: [a, b].map(|id| Slot {
            id,
            rank: 1,
            pick: Pick::A,
        }),
        ult: 1,
        great: crate::skills::Ult::Upend,
    });
}

/// 한손검: 방패를 들면 방패 기술(막아치기)이 살고 한손검 기술(자루 치기 — 0274 ③부터 가로지르기는 쌍단검의 것)이 빈 칸이 된다 —
/// 방패를 내리면 거꾸로. 배운 단은 그대로 남는다.
#[test]
fn a_shield_turns_the_sword_from_duelling_arts_to_shield_arts() {
    let mut w = World::new("test", 30.0);
    assert_eq!(w.player.weapon, W::Sword);
    worn(&mut w, Sk::Bash, Sk::Pommel);
    assert_eq!(w.grip(), Grip::Class(Class::Sword));
    assert_eq!(w.player.skill.rank, [0, 1], "no shield: the duel art only");
    w.set_shield(Some(crate::shield::Id::Pilgrim));
    assert_eq!(w.grip(), Grip::Shield);
    assert_eq!(w.player.skill.rank, [1, 0], "with a shield: the shield art");
    assert_eq!(w.player.skill.known, [1, 1], "what was learned stays");
    w.set_shield(None);
    assert_eq!(w.player.skill.rank, [0, 1]);
    // 한손 어느 쪽이든 되는 것(휘돌이)은 방패와 상관없다.
    worn(&mut w, Sk::Whirl, Sk::Gather);
    assert_eq!(w.player.skill.rank, [1, 1]);
    w.set_shield(Some(crate::shield::Id::Pilgrim));
    assert_eq!(w.player.skill.rank, [1, 1]);
    // 0274: 망치도 한손 — 방패와 함께면 방패 판의 기술(한손검과 같다).
    w.set_weapon(W::Hammer);
    assert_eq!(w.grip(), Grip::Shield);
    // 두 손 무기(창 · 쌍단검)를 들면 방패는 쉰다 — 쥔 꼴은 그 무기의 것.
    w.set_weapon(W::Spear);
    assert_eq!(w.grip(), Grip::Class(Class::Spear));
    w.set_weapon(W::Daggers);
    assert_eq!(w.grip(), Grip::Class(Class::Dagger));
    assert!(!w.can_block(), "no hand left for the shield");
}

/// 망치의 물리력: 타입 없는 스킬(휘돌이)의 피해·자세 피해가 무기의 배만큼 — 타입 있는 스킬(던지기)은 그대로. 검은 배가 없다.
#[test]
fn the_hammer_adds_force_to_untyped_skills_only() {
    let mut w = World::new("test", 30.0);
    worn(&mut w, Sk::Whirl, Sk::Throw);
    let (whirl, throw) = (w.skill_eff(Sk::Whirl).skill, w.skill_eff(Sk::Throw).skill);
    assert_eq!(whirl.damage, Sk::Whirl.def().damage);
    w.set_weapon(W::Hammer);
    let (hw, ht) = (w.skill_eff(Sk::Whirl).skill, w.skill_eff(Sk::Throw).skill);
    assert_eq!(hw.damage, (whirl.damage as f32 * ART_FORCE).round() as i32);
    assert!((hw.poise - whirl.poise * ART_FORCE).abs() < 1e-4);
    assert_eq!((ht.damage, ht.poise), (throw.damage, throw.poise), "typed");
    assert_eq!(w.dosed((Ty::Rift, 1.0)).1, 1.0, "no channel in a hammer");
}

/// 지팡이의 마법력: 스킬의 타가 쌓는 타입 게이지가 무기의 배만큼 — 쇠뇌로는 게이지만 차는 첫 불티가 지팡이로는 불을 붙인다.
#[test]
fn the_staff_channels_more_type_gauge_per_skill_hit() {
    let t = Tuning::default();
    let gauge = |arm: W| {
        let mut w = World::new("test", 30.0);
        w.set_weapon(arm);
        worn(&mut w, Sk::Embers, Sk::Gather);
        assert_eq!(w.player.skill.rank, [1, 1], "{arm:?} holds both");
        let i = mob_with(&mut w, vec3(0.0, 0.0, -1.5), 500);
        w.enemies[i].state = EState::Recover { t: -1000.0 };
        w.step(
            &Actions {
                skill1: true,
                ..Default::default()
            },
            &t,
        );
        // 첫 불티가 닿을 때까지.
        for _ in 0..90 {
            w.step(&Actions::default(), &t);
            if w.enemies[i].ail.gauge[0] > 0.0 || w.enemies[i].ail.burn > 0 {
                break;
            }
        }
        (
            w.enemies[i].ail.gauge[0],
            w.enemies[i].ail.burn,
            w.dosed((Ty::Ember, 1.0)).1,
        )
    };
    let (xbow, cold, base) = gauge(W::Crossbow);
    let (_, lit, more) = gauge(W::Staff);
    assert_eq!(base, 1.0);
    assert!((more - ART_CHANNEL).abs() < 1e-4);
    // 쇠뇌: 첫 불티로는 게이지만 찬다. 지팡이: 같은 불티가 가득 채워 불이 붙는다.
    assert!(xbow > 0.0 && xbow < 1.0 && cold == 0, "{xbow} / {cold}");
    assert!(xbow * ART_CHANNEL >= 1.0, "the test's premise: {xbow}");
    assert!(lit >= 1, "the staff's first pulse lights it");
}

/// 남아 있는 것(던진 무기·불티)은 쓴 순간의 무기를 따른다 — 날아가는 동안 지팡이로 바꿔 들어도 마법력이 얹히지 않는다.
#[test]
fn what_a_skill_leaves_behind_keeps_the_weapon_it_was_cast_with() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    // 0274 ③: 던지기는 쌍단검 · 망치의 스킬 — 망치로 던진다(망치에는 마법력이 없다).
    w.set_weapon(W::Hammer);
    worn(&mut w, Sk::Throw, Sk::Gather);
    let i = mob_with(&mut w, vec3(0.0, 0.0, -2.0), 500);
    w.enemies[i].state = EState::Recover { t: -1000.0 };
    w.step(
        &Actions {
            skill1: true,
            ..Default::default()
        },
        &t,
    );
    for _ in 0..120 {
        w.step(&Actions::default(), &t);
        if !w.skill_blades.is_empty() {
            break;
        }
    }
    let cast = w.skill_blades[0].dose;
    assert_eq!(cast, w.dosed(Sk::Throw.def().dose()), "the hammer's own");
    w.set_weapon(W::Staff);
    assert!(
        w.dosed(Sk::Throw.def().dose()).1 > cast.1,
        "a staff would add more"
    );
    assert_eq!(
        w.skill_blades[0].dose, cast,
        "the blade in the air does not change"
    );
}

/// 0272: 맨손은 무기 종류가 아니다 — 무기 판의 스킬이 놓인 칸은 빈 칸이 된다(배운 단은 남는다). 방패를 들면 방패 판의 것은 쓴다
/// (맨손도 한손 계열). 빈 손의 덤(날카로움)은 맨손도 받는다.
#[test]
fn bare_hands_hold_no_weapon_plate() {
    let mut w = World::new("test", 30.0);
    worn(&mut w, Sk::Whirl, Sk::Bash);
    assert_eq!(w.player.skill.rank, [1, 0]);
    w.set_weapon(W::Fists);
    assert_eq!(w.grip(), Grip::Bare);
    assert_eq!(w.player.skill.rank, [0, 0], "nothing fits bare hands");
    assert_eq!(w.player.skill.known, [1, 1], "what was learned stays");
    assert!(w.keen(), "a free hand all the same");
    w.set_shield(Some(crate::shield::Id::Pilgrim));
    assert_eq!(w.grip(), Grip::Shield);
    assert_eq!(w.player.skill.rank, [1, 1], "shield arts with a shield");
    assert!(!w.keen());
    w.set_shield(None);
    w.set_weapon(W::Sword);
    assert_eq!(w.player.skill.rank, [1, 0], "back in the sword's hands");
}

/// 0272: 쇠뇌와 활은 다른 종류다 — 스킬 칸이 종류를 따른다(불티는 쇠뇌의 것, 뿌리 솟기는 활의 것, 끌어모으기는 어느 것이나).
#[test]
fn the_crossbow_and_the_bow_wield_different_skills() {
    let mut w = World::new("test", 30.0);
    w.set_weapon(W::Crossbow);
    assert_eq!(w.grip(), Grip::Class(Class::Crossbow));
    worn(&mut w, Sk::Embers, Sk::Roots);
    assert_eq!(w.player.skill.rank, [1, 0]);
    w.set_weapon(W::Bow);
    assert_eq!(w.grip(), Grip::Class(Class::Bow));
    assert_eq!(w.player.skill.rank, [0, 1]);
    worn(&mut w, Sk::Gather, Sk::Snare);
    assert_eq!(w.player.skill.rank, [1, 1]);
    w.set_weapon(W::Crossbow);
    assert_eq!(w.player.skill.rank, [1, 0]);
    worn(&mut w, Sk::Fuse, Sk::Mine);
    assert_eq!(w.player.skill.rank, [1, 1]);
}
