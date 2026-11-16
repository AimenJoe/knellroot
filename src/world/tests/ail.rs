//! 피해 타입(0223 V2 — 월드 쪽, 규칙은 `status.rs`의 시험): 화상이 타고, 묶인 적이 서고, 금 간 적이 더 다치고, 울림이 떼를 타고
//! 한 바퀴 번지고, 겹침 넷(묶인 적의 화상은 한 겹 더 — `status` · 화상 중에 묶이면 터진다 · 타는 적이 울리면 불이 옮는다 ·
//! 금 간 적이 울리면 금이 번진다).
use super::*;
use crate::skills::{Id as Sk, Loadout, Pick, Slot};
use crate::status::Kind as Ty;
use crate::tuning::{
    BURN_DMG, BURST_DMG, RIFT_BOSS_MULT, RIFT_MULT, ROOT_BOSS_SLOW, ROOT_POISE, TOLL_DMG,
    TOLL_RIFT_SECS, TOLL_SHARE,
};

/// 가만히 서 있는(쉬는) 튼튼한 잡몹 하나.
fn foe(w: &mut World, at: Vec3) -> usize {
    let i = mob_with(w, at, 500);
    w.enemies[i].state = EState::Recover { t: -1000.0 };
    i
}

fn run(w: &mut World, t: &Tuning, secs: f32) -> Vec<Event> {
    let mut ev = Vec::new();
    for _ in 0..(secs / TICK_DT).round() as usize {
        ev.extend(w.step(&Actions::default(), t));
    }
    ev
}

/// 게이지가 찰 때까지 쌓는다(종마다 배율이 달라도) — 걸린 일의 사건을 돌려준다.
fn fill(w: &mut World, i: usize, kind: Ty) -> Vec<Event> {
    let mut ev = Vec::new();
    afflict(&mut w.enemies[i], (kind, 10.0), &mut ev);
    ev
}

fn count(ev: &[Event], hit: impl Fn(&Event) -> bool) -> usize {
    ev.iter().filter(|e| hit(e)).count()
}

/// 잔불: 화상은 초마다 겹만큼 탄다 — 타마다의 덤 없이. 불로도 쓰러진다(스킬로 쓰러뜨린 것으로 센다 — 큰 기술 게이지는 그대로).
#[test]
fn a_burning_foe_loses_health_every_second_and_the_fire_can_kill() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    let i = foe(&mut w, vec3(6.0, 0.0, 0.0));
    let ev = fill(&mut w, i, Ty::Ember);
    assert!(matches!(
        ev[..],
        [Event::Ail {
            kind: Ty::Ember,
            ..
        }]
    ));
    assert_eq!(w.enemies[i].ail.burn, 1);
    let ev = run(&mut w, &t, 2.5);
    assert_eq!(count(&ev, |e| matches!(e, Event::Burn { .. })), 2);
    assert_eq!(w.enemies[i].hp, 500 - 2 * BURN_DMG);
    w.enemies[i].hp = 1;
    let ev = run(&mut w, &t, 1.0);
    assert!(!w.enemies[i].alive());
    assert_eq!(w.kills, 1);
    assert_eq!(count(&ev, |e| matches!(e, Event::EnemyDown { .. })), 1);
    assert_eq!(w.player.skill.gauge, 0.0, "a burn fills no ult gauge");
}

/// 뿌리: 묶인 적은 걷지 못한다(풀리면 다시 온다). 보스는 묶이지 않고 느려지기만 한다.
#[test]
fn a_bound_foe_stands_where_it_is_and_a_boss_only_slows() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    let bound = mob_with(&mut w, vec3(8.0, 0.0, 0.0), 500);
    let free = mob_with(&mut w, vec3(-8.0, 0.0, 0.0), 500);
    let ev = fill(&mut w, bound, Ty::Root);
    assert!(matches!(ev[..], [Event::Ail { kind: Ty::Root, .. }]));
    let (b0, f0) = (w.enemies[bound].pos, w.enemies[free].pos);
    run(&mut w, &t, 1.0);
    assert!(w.enemies[bound].pos.distance(b0) < 0.05, "rooted");
    assert!(w.enemies[free].pos.distance(f0) > 1.0, "the other walks");
    run(&mut w, &t, 1.5);
    assert!(w.enemies[bound].pos.distance(b0) > 0.5, "let go");

    let mut w = world_with_boss();
    fill(&mut w, 0, Ty::Root);
    w.step(&Actions::default(), &t);
    assert_eq!(w.enemies[0].slow, ROOT_BOSS_SLOW);
    assert_eq!(w.enemies[0].bind, 0.0);
}

/// 균열: 금 간 적은 모든 피해를 더 받는다(보스는 덜) — 끌어모으기 맺음의 "더 다치는" 표지와는 곱하지 않고 큰 쪽 하나만.
#[test]
fn a_cracked_foe_takes_more_from_every_blow_and_a_boss_less() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    let i = foe(&mut w, vec3(3.0, 0.0, 0.0));
    assert_eq!(hurt_by(&w.enemies[i], 100, false, 0, &t), 100);
    fill(&mut w, i, Ty::Rift);
    assert_eq!(
        hurt_by(&w.enemies[i], 100, false, 0, &t),
        (100.0 * RIFT_MULT) as i32
    );
    w.enemies[i].frail = 1.0;
    w.enemies[i].frail_mult = 1.5;
    assert_eq!(
        hurt_by(&w.enemies[i], 100, false, 0, &t),
        150,
        "the larger one, not both"
    );
    let mut b = world_with_boss();
    fill(&mut b, 0, Ty::Rift);
    assert_eq!(
        hurt_by(&b.enemies[0], 100, false, 0, &t),
        (100.0 * RIFT_BOSS_MULT).round() as i32
    );
    // 금은 때가 되면 닫힌다.
    run(&mut w, &t, crate::tuning::RIFT_SECS + 0.1);
    w.enemies[i].frail = 0.0;
    assert_eq!(hurt_by(&w.enemies[i], 100, false, 0, &t), 100);
}

/// 뿌리: 묶인 적은 자세가 더 깎인다.
#[test]
fn a_bound_foe_loses_more_poise() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    let a = foe(&mut w, vec3(3.0, 0.0, 0.0));
    let b = foe(&mut w, vec3(-3.0, 0.0, 0.0));
    fill(&mut w, a, Ty::Root);
    let mut ev = Vec::new();
    add_poise(&mut w.enemies[a], 1.0, None, &t, &mut ev);
    add_poise(&mut w.enemies[b], 1.0, None, &t, &mut ev);
    assert!((w.enemies[a].poise - w.enemies[b].poise * ROOT_POISE).abs() < 1e-4);
}

/// 울림: 게이지가 찬 적이 한 박 뒤에 울린다 — 둘레의 적(울린 적도)이 다치고 이웃은 울림 게이지를 나눠 받는다. 먼 적은 그대로.
#[test]
fn a_toll_strikes_everyone_around_and_shares_the_ringing() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    let a = foe(&mut w, vec3(6.0, 0.0, 0.0));
    let b = foe(&mut w, vec3(8.0, 0.0, 0.0));
    let far = foe(&mut w, vec3(-12.0, 0.0, 0.0));
    assert!(
        fill(&mut w, a, Ty::Knell).is_empty(),
        "the toll speaks for itself"
    );
    w.step(&Actions::default(), &t);
    assert_eq!(w.enemies[a].hp, 500, "a beat later");
    let ev = run(&mut w, &t, 0.3);
    assert_eq!(count(&ev, |e| matches!(e, Event::Toll { .. })), 1);
    assert_eq!(w.enemies[a].hp, 500 - TOLL_DMG);
    assert_eq!(w.enemies[b].hp, 500 - TOLL_DMG);
    assert_eq!(w.enemies[far].hp, 500);
    assert!((w.enemies[b].ail.gauge[2] - TOLL_SHARE).abs() < 1e-4);
    assert_eq!(w.enemies[far].ail.gauge[2], 0.0);
    assert_eq!(w.player.skill.gauge, 0.0, "a toll fills no ult gauge");
}

/// 울림은 떼를 타고 번지되 한 바퀴로 끝난다: 가득 직전인 여섯이 차례로 한 번씩만 울린다(서로 끝없이 울리지 않는다).
#[test]
fn ringing_runs_once_through_a_pack_and_stops() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    let pack: Vec<usize> = (0..6)
        .map(|k| {
            let a = k as f32 * std::f32::consts::TAU / 6.0;
            let i = foe(&mut w, vec3(8.0 + a.cos() * 1.5, 0.0, a.sin() * 1.5));
            w.enemies[i].ail.gauge[2] = 0.9;
            i
        })
        .collect();
    fill(&mut w, pack[0], Ty::Knell);
    let ev = run(&mut w, &t, 9.0);
    assert_eq!(count(&ev, |e| matches!(e, Event::Toll { .. })), 6);
    for &i in &pack {
        assert_eq!(
            w.enemies[i].hp,
            500 - 6 * TOLL_DMG,
            "six tolls, each heard by all"
        );
    }
}

/// 겹침: 화상 중인 적이 묶이면 겹을 다 쓰고 터진다 — 둘레의 적이 쓴 겹만큼 다친다.
#[test]
fn overlap_binding_a_burning_foe_bursts_on_those_around() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    let a = foe(&mut w, vec3(6.0, 0.0, 0.0));
    let b = foe(&mut w, vec3(7.5, 0.0, 0.0));
    let far = foe(&mut w, vec3(-12.0, 0.0, 0.0));
    fill(&mut w, a, Ty::Ember);
    fill(&mut w, a, Ty::Ember);
    assert_eq!(w.enemies[a].ail.burn, 2);
    assert!(fill(&mut w, a, Ty::Root).is_empty());
    let ev = run(&mut w, &t, 0.1);
    assert_eq!(count(&ev, |e| matches!(e, Event::AilBurst { .. })), 1);
    assert_eq!(w.enemies[b].hp, 500 - 2 * BURST_DMG);
    assert_eq!(w.enemies[far].hp, 500);
    assert_eq!(w.enemies[a].ail.burn, 0, "the fire is spent");
    assert!(w.enemies[a].ail.bound > 0.0);
}

/// 겹침: 타는 적이 울리면 이웃에 불이 한 겹 옮고, 금 간 적이 울리면 이웃에도 금이 번진다(짧게).
#[test]
fn overlap_a_ringing_foe_passes_on_its_fire_and_its_crack() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    let a = foe(&mut w, vec3(6.0, 0.0, 0.0));
    let b = foe(&mut w, vec3(8.0, 0.0, 0.0));
    fill(&mut w, a, Ty::Ember);
    fill(&mut w, a, Ty::Rift);
    fill(&mut w, a, Ty::Knell);
    run(&mut w, &t, 0.3);
    assert_eq!(w.enemies[b].ail.burn, 1);
    assert!(w.enemies[b].ail.rift > 0.0 && w.enemies[b].ail.rift <= TOLL_RIFT_SECS);
    // 타지도 금 가지도 않은 적이 울리면 아무것도 옮지 않는다.
    let mut w = World::new("test", 30.0);
    let a = foe(&mut w, vec3(6.0, 0.0, 0.0));
    let b = foe(&mut w, vec3(8.0, 0.0, 0.0));
    fill(&mut w, a, Ty::Knell);
    run(&mut w, &t, 0.3);
    assert_eq!((w.enemies[b].ail.burn, w.enemies[b].ail.rift), (0, 0.0));
}

/// 전환 중인 보스(0133)에게는 쌓이지 않는다.
#[test]
fn nothing_builds_on_a_boss_mid_transition() {
    let mut w = world_with_boss();
    w.enemies[0].state = EState::Rise { t: 0.0 };
    for k in crate::status::TYPED {
        assert!(fill(&mut w, 0, k).is_empty());
    }
    assert_eq!(w.enemies[0].ail, crate::status::Ail::default());
}

/// 스킬의 타가 제 타입을 싣는다: 가로지르기(균열)는 지나간 적에 금을 내고, 그 금에 다음 평타가 더 아프다. 휘돌이(물리)는 아무것도.
#[test]
fn a_typed_skill_leaves_its_mark_and_a_physical_one_none() {
    let t = Tuning::default();
    let cast = |id: Sk| {
        let mut w = World::new("test", 30.0);
        w.player.facing = std::f32::consts::PI;
        w.set_skills(Loadout {
            slots: [
                Slot {
                    id,
                    rank: 1,
                    pick: Pick::A,
                },
                Slot {
                    id: if id == Sk::Whirl { Sk::Cut } else { Sk::Whirl },
                    rank: 1,
                    pick: Pick::A,
                },
            ],
            ult: 1,
            great: crate::skills::Ult::Upend,
        });
        let i = foe(&mut w, vec3(0.0, 0.0, -2.0));
        let mut ev = w.step(
            &Actions {
                skill1: true,
                ..Default::default()
            },
            &t,
        );
        ev.extend(run(&mut w, &t, 1.0));
        (w.enemies[i].ail, ev)
    };
    let (ail, ev) = cast(Sk::Cut);
    assert!(ail.rift > 0.0);
    assert_eq!(
        count(&ev, |e| matches!(e, Event::Ail { kind: Ty::Rift, .. })),
        1
    );
    let (ail, ev) = cast(Sk::Whirl);
    assert_eq!((ail.gauge, ail.rift, ail.burn), ([0.0; 4], 0.0, 0));
    assert_eq!(count(&ev, |e| matches!(e, Event::Ail { .. })), 0);
}

/// 0223 V4: 아이템 옵션 둘이 월드에 닿는다 — "타입 게이지 +%"는 스킬의 타가 쌓는 양을, "스킬 대기 −%"는 쓴 뒤의 대기를 바꾼다.
#[test]
fn item_affixes_scale_the_type_build_and_the_skill_cooldown() {
    use crate::stats::{Mods, Stats};
    let t = Tuning::default();
    let cast = |mods: Mods| {
        let mut w = World::new("test", 30.0);
        w.player.facing = std::f32::consts::PI;
        w.apply_stats(Stats::compute(&[mods]), true);
        w.set_skills([Sk::Cut, Sk::Whirl]);
        let i = foe(&mut w, vec3(0.0, 0.0, -2.0));
        w.step(
            &Actions {
                skill1: true,
                ..Default::default()
            },
            &t,
        );
        let cd = w.player.skill.cd[Sk::Cut.index()];
        run(&mut w, &t, 1.0);
        (w.enemies[i].ail, cd)
    };
    let (plain, cd) = cast(Mods::default());
    assert!(plain.rift > 0.0, "one cut cracks a plain foe");
    // 게이지가 반만 쌓이면 한 번으로는 금이 가지 않는다(반만 찬 채 남는다).
    let (half, _) = cast(Mods {
        ail: 0.5,
        ..Mods::default()
    });
    assert_eq!(half.rift, 0.0);
    assert!((half.gauge[3] - 0.5).abs() < 0.05, "{:?}", half.gauge);
    let (_, short) = cast(Mods {
        skill_cd: 0.9,
        ..Mods::default()
    });
    assert!((short - cd * 0.9).abs() < 1e-3, "{short} vs {cd}");
}
