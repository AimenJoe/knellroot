//! 피해 타입(0223 V2 — 월드 쪽, 규칙은 `status.rs`의 시험): 화상이 타고, 묶인 적이 서고, 금 간 적이 더 다치고, 울림이 떼를 타고
//! 한 바퀴 번지고, 겹침 넷(묶인 적의 화상은 한 겹 더 — `status` · 화상 중에 묶이면 터진다 · 타는 적이 울리면 불이 옮는다 ·
//! 금 간 적이 울리면 금이 번진다).
use super::*;
use crate::skills::{Id as Sk, Loadout, Pick, Slot};
use crate::status::Kind as Ty;
use crate::tuning::{
    BURN_DMG, BURST_DMG, RIFT_BOSS_MULT, RIFT_MULT, ROOT_BOSS_SLOW, ROOT_POISE, TOLL_DMG,
    TOLL_RIFT_SECS, TOLL_SHARE, TOLL_STACK,
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
    assert_eq!(w.enemies[0].marks.slow, ROOT_BOSS_SLOW);
    assert_eq!(w.enemies[0].marks.bind, 0.0);
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
    w.enemies[i].marks.frail = 1.0;
    w.enemies[i].marks.frail_mult = 1.5;
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
    w.enemies[i].marks.frail = 0.0;
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
    assert!((w.enemies[a].stance.poise - w.enemies[b].stance.poise * ROOT_POISE).abs() < 1e-4);
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
        // 첫 울림 하나 + 뒤따라 한꺼번에 울린 다섯 가운데 아픈 것은 `TOLL_STACK` 번까지.
        assert_eq!(
            w.enemies[i].hp,
            500 - (1 + TOLL_STACK as i32) * TOLL_DMG,
            "six tolls: the first, then the five at once — capped"
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
    assert_eq!(
        (ail.gauge, ail.rift, ail.burn),
        ([0.0; crate::status::N], 0.0, 0)
    );
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

/// 떼가 한꺼번에 울려도 한 적이 받는 울림의 피해는 `TOLL_STACK` 번까지 — 떼가 클수록 스킬 한 번이 곱절로 세지지 않는다.
#[test]
fn a_whole_pack_tolling_at_once_hurts_each_foe_only_a_few_tolls_worth() {
    let t = Tuning::default();
    for n in [2usize, 4, 10, 24] {
        let mut w = World::new("test", 30.0);
        let ids: Vec<usize> = (0..n)
            .map(|k| {
                let (a, r) = (k as f32 * 2.4, 0.6 + 0.35 * (k as f32).sqrt());
                foe(&mut w, vec3(6.0 + r * a.cos(), 0.0, r * a.sin()))
            })
            .collect();
        for &i in &ids {
            fill(&mut w, i, Ty::Knell);
        }
        let ev = run(&mut w, &t, 8.0);
        assert_eq!(count(&ev, |e| matches!(e, Event::Toll { .. })), n);
        let most = TOLL_DMG * (n as i32).min(TOLL_STACK as i32);
        for &i in &ids {
            assert_eq!(500 - w.enemies[i].hp, most, "pack of {n}");
        }
    }
}

/// 이 타에 쓰러진 적에게는 걸리지 않는다 — 주검 위에 상태 표지·소리가 뜨지 않는다(터짐도 남지 않는다).
#[test]
fn a_foe_the_hit_just_killed_takes_no_affliction() {
    let mut w = World::new("test", 30.0);
    let i = foe(&mut w, vec3(6.0, 0.0, 0.0));
    fill(&mut w, i, Ty::Ember);
    w.enemies[i].hp = 0;
    let before = w.enemies[i].ail;
    let ev = fill(&mut w, i, Ty::Root);
    assert!(ev.is_empty(), "{ev:?}");
    assert_eq!(w.enemies[i].ail, before);
}

/// 0270 B 충격: 가득 차면 잡몹은 기절해 선다(휘청 — 하던 것을 끊는다), 문지기는 휘청이지 않고 그 짧은 초 동안 느려지기만.
/// 기절한 적은 자세가 더 깎인다.
#[test]
fn a_shocked_foe_is_stunned_and_a_boss_only_slows_for_a_moment() {
    use crate::tuning::{SHOCK_POISE, STUN_SECS};
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    let a = mob_with(&mut w, vec3(6.0, 0.0, 0.0), 500);
    let b = foe(&mut w, vec3(-6.0, 0.0, 0.0));
    let ev = fill(&mut w, a, Ty::Shock);
    assert!(matches!(
        ev[..],
        [Event::Ail {
            kind: Ty::Shock,
            ..
        }]
    ));
    assert!(
        matches!(w.enemies[a].state, EState::Stagger { dur, .. } if (dur - STUN_SECS).abs() < 1e-5)
    );
    let mut ev = Vec::new();
    add_poise(&mut w.enemies[a], 1.0, None, &t, &mut ev);
    add_poise(&mut w.enemies[b], 1.0, None, &t, &mut ev);
    assert!((w.enemies[a].stance.poise - w.enemies[b].stance.poise * SHOCK_POISE).abs() < 1e-4);

    let mut w = world_with_boss();
    fill(&mut w, 0, Ty::Shock);
    assert!(!matches!(w.enemies[0].state, EState::Stagger { .. }));
    w.step(&Actions::default(), &t);
    assert_eq!(w.enemies[0].marks.slow, ROOT_BOSS_SLOW);
}

/// 0270 B 날카로움: 상처 난 적은 움직이는 만큼 샌다 — 가만히 선 적은 새지 않는다. 새는 피로도 쓰러진다.
#[test]
fn a_cut_foe_bleeds_as_it_walks_and_not_while_it_stands() {
    use crate::tuning::BLEED_DMG;
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    let walker = mob_with(&mut w, vec3(9.0, 0.0, 0.0), 500);
    let stander = foe(&mut w, vec3(-9.0, 0.0, 0.0));
    fill(&mut w, walker, Ty::Sharp);
    fill(&mut w, stander, Ty::Sharp);
    let ev = run(&mut w, &t, 2.0);
    let lost = 500 - w.enemies[walker].hp;
    assert!(lost >= BLEED_DMG && lost % BLEED_DMG == 0, "{lost}");
    assert!(count(&ev, |e| matches!(e, Event::Bleed { .. })) >= 1);
    assert_eq!(w.enemies[stander].hp, 500, "standing still stops it");
    w.enemies[walker].hp = 1;
    run(&mut w, &t, 2.0);
    assert!(!w.enemies[walker].alive(), "it can bleed out");
}

/// 0270 B 서리: 쌓이는 만큼 느려지고(문지기는 아니다), 가득 차면 얼어 선다. 언 적을 치면 그 한 대는 크게 들어가고 깨진다(한 번).
#[test]
fn frost_slows_then_freezes_and_the_next_blow_shatters_it() {
    use crate::tuning::{FREEZE_SECS, SHATTER_MULT};
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    let i = mob_with(&mut w, vec3(6.0, 0.0, 0.0), 500);
    let mut ev = Vec::new();
    afflict(&mut w.enemies[i], (Ty::Frost, 0.5), &mut ev);
    w.step(&Actions::default(), &t);
    assert!(w.enemies[i].marks.slow < 1.0, "chilled");
    fill(&mut w, i, Ty::Frost);
    assert!(
        matches!(w.enemies[i].state, EState::Stagger { dur, .. } if (dur - FREEZE_SECS).abs() < 1e-5)
    );
    assert_eq!(
        hurt_by(&w.enemies[i], 100, false, 0, &t),
        (100.0 * SHATTER_MULT) as i32
    );
    // 얼린 그 타는 깨지 않는다(0270 C3) — 한 틱 지나 굳은 얼음을 다음 타가 깬다.
    w.enemies[i].struck();
    let ev = w.step(&Actions::default(), &t);
    assert_eq!(count(&ev, |e| matches!(e, Event::Shatter { .. })), 0);
    assert!(
        w.enemies[i].ail.frozen > 0.0,
        "the freezing blow does not break it"
    );
    w.enemies[i].struck();
    let ev = w.step(&Actions::default(), &t);
    assert_eq!(count(&ev, |e| matches!(e, Event::Shatter { .. })), 1);
    assert_eq!(w.enemies[i].ail.frozen, 0.0);
    assert!(
        !matches!(w.enemies[i].state, EState::Stagger { .. }),
        "free again"
    );
    assert_eq!(hurt_by(&w.enemies[i], 100, false, 0, &t), 100);

    let mut b = world_with_boss();
    afflict(&mut b.enemies[0], (Ty::Frost, 0.5), &mut Vec::new());
    b.step(&Actions::default(), &t);
    assert_eq!(
        b.enemies[0].marks.slow, 1.0,
        "a boss is not chilled by the gauge"
    );
}

/// 0270 B 흡수: 표지가 붙은 적을 치면 체력·기력이 조금 돌아온다 — 표지가 붙은 떼를 한꺼번에 쳐도 틱에 한 번만.
#[test]
fn striking_drained_foes_heals_a_little_once_a_tick_however_many() {
    use crate::tuning::{DRAIN_HP, DRAIN_STAMINA};
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    let pack: Vec<usize> = (0..5)
        .map(|k| foe(&mut w, vec3(4.0 + k as f32, 0.0, 0.0)))
        .collect();
    let plain = foe(&mut w, vec3(-4.0, 0.0, 0.0));
    w.player.hp = 10;
    w.player.stamina = 0.0;
    // 표지 없는 적을 쳐도 아무것도.
    w.enemies[plain].struck();
    let ev = w.step(&Actions::default(), &t);
    assert_eq!(count(&ev, |e| matches!(e, Event::Drain { .. })), 0);
    assert_eq!(w.player.hp, 10);
    for &i in &pack {
        fill(&mut w, i, Ty::Drain);
        w.enemies[i].struck();
    }
    let st = w.player.stamina;
    let ev = w.step(&Actions::default(), &t);
    assert_eq!(count(&ev, |e| matches!(e, Event::Drain { .. })), 1);
    assert_eq!(w.player.hp, 10 + DRAIN_HP);
    assert!(w.player.stamina >= st + DRAIN_STAMINA - 1e-3);
    // 다음 틱에 안 쳤으면 그대로.
    w.step(&Actions::default(), &t);
    assert_eq!(w.player.hp, 10 + DRAIN_HP);
}

/// 0270 B: 새 타입 넷을 싣는 스킬이 하나씩은 있다(무기의 길 — 판은 0270 C에서 무기마다).
#[test]
fn every_new_type_rides_on_some_skill() {
    for k in [Ty::Shock, Ty::Sharp, Ty::Frost, Ty::Drain] {
        assert!(
            crate::skills::SKILLS
                .iter()
                .any(|s| s.kind == k && s.build > 0.0),
            "{k:?}"
        );
    }
}
