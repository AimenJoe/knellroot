use super::*;
use crate::skills::Id as Sk;
use std::f32::consts::PI;

/// 멈춘 적 셋·화면 위(-z)를 보는 방랑자.
fn world() -> (World, Tuning) {
    let mut w = World::new("test", 30.0);
    w.player.facing = PI;
    (w, Tuning::default())
}

fn still(w: &mut World) {
    for e in &mut w.enemies {
        e.hp = 90;
        e.max_hp = 90;
        e.state = EState::Recover { t: -1000.0 };
    }
}

fn skill(slot: usize) -> Actions {
    Actions {
        skill1: slot == 0,
        skill2: slot == 1,
        ..Default::default()
    }
}

fn run(w: &mut World, t: &Tuning, a: Actions, ticks: usize) -> Vec<Event> {
    let mut ev = w.step(&a, t);
    for _ in 1..ticks {
        ev.extend(w.step(&Actions::default(), t));
    }
    ev
}

fn flat(a: Vec3, b: Vec3) -> f32 {
    vec3(a.x - b.x, 0.0, a.z - b.z).length()
}

/// 0166: 안 배운 스킬이 놓인 칸은 빈 칸(단 0)이다 — 눌러도 시작하지 않고, 받아 두지도 "안 됨"을 알리지도 않는다.
/// 다른 칸의 배운 스킬은 그대로 나가고, 두 버튼 "함께"의 큰 기술도 그대로다(큰 기술은 처음부터 가진다).
#[test]
fn an_empty_slot_does_nothing_when_pressed() {
    use crate::skills::{Loadout, Pick, Slot, Ult};
    let (mut w, t) = world();
    w.spawn_mob(vec3(0.0, 0.0, -2.0));
    still(&mut w);
    let slot = |id, rank| Slot {
        id,
        rank,
        pick: Pick::A,
    };
    w.set_skills(Loadout {
        slots: [slot(Sk::Whirl, 0), slot(Sk::Cut, 1)],
        ult: 1,
        great: Ult::Upend,
    });
    assert_eq!(w.player.skill.rank, [0, 1]);
    let ev = run(&mut w, &t, skill(0), 30);
    assert!(
        !ev.iter().any(|e| matches!(
            e,
            Event::SkillStart { .. } | Event::Skill { .. } | Event::SkillDenied
        )),
        "{ev:?}"
    );
    assert!(matches!(w.player.state, PState::Idle));
    assert_eq!(w.player.skill.cd[Sk::Whirl.index()], 0.0, "no cooldown");
    assert_eq!(w.enemies[0].hp, 90);
    // 배운 칸은 그대로.
    let ev = run(&mut w, &t, skill(1), 5);
    assert!(ev
        .iter()
        .any(|e| matches!(e, Event::SkillStart { id: Sk::Cut })));
    // 둘 다 빈 칸이어도 게이지가 차면 "함께" = 큰 기술.
    let (mut w, t) = world();
    w.set_skills(Loadout {
        slots: [slot(Sk::Whirl, 0), slot(Sk::Cut, 0)],
        ult: 1,
        great: Ult::Upend,
    });
    w.player.skill.gauge = 1.0;
    let both = Actions {
        skill1: true,
        skill2: true,
        ..Default::default()
    };
    run(&mut w, &t, both, 3);
    assert!(
        matches!(w.player.state, PState::Ult { .. }),
        "{:?}",
        w.player.state
    );
}

/// 휘돌이: 둘레의 적을 **모두** 두 번 치고(바퀴마다 한 번) 바깥으로 민다. 기력은 쓰지 않고 대기가 돈다.
#[test]
fn whirl_cuts_everyone_around_twice_and_pushes_them_out() {
    let (mut w, t) = world();
    for k in 0..6 {
        let a = k as f32 / 6.0 * std::f32::consts::TAU;
        w.spawn_mob(vec3(a.cos() * 2.2, 0.0, a.sin() * 2.2));
    }
    w.spawn_mob(vec3(0.0, 0.0, -6.0));
    still(&mut w);
    let stamina = w.player.stamina;
    let ev = run(&mut w, &t, skill(0), 80);
    assert!(ev
        .iter()
        .any(|e| matches!(e, Event::SkillStart { id: Sk::Whirl })));
    let rounds = ev
        .iter()
        .filter(|e| matches!(e, Event::Skill { id: Sk::Whirl, .. }))
        .count();
    assert_eq!(rounds, 2, "one flash per turn");
    let dmg = Sk::Whirl.def().damage + w.stats.dmg;
    for e in &w.enemies[..6] {
        assert_eq!(e.hp, 90 - 2 * dmg, "hit once per turn");
        assert!(flat(e.pos, w.player.pos) > 2.8, "pushed out: {:?}", e.pos);
    }
    assert_eq!(w.enemies[6].hp, 90, "out of reach");
    assert_eq!(w.player.stamina, stamina, "skills cost no stamina");
    assert!(w.player.skill.cd[Sk::Whirl.index()] > 4.0, "cooling down");
    assert!(matches!(w.player.state, PState::Idle));
    // 두 바퀴를 돌고 처음 보던 쪽으로 돌아온다.
    assert!(w.player.facing_dir().dot(vec3(0.0, 0.0, -1.0)) > 0.99);
}

/// 가로지르기: 바라보는 쪽으로 6m를 지나가며 길 위의 적을 한 번씩 베고 반대편에 선다 — 적의 몸에 막히지 않는다.
#[test]
fn cut_through_passes_the_pack_and_cuts_everyone_on_the_way_once() {
    let (mut w, t) = world();
    for z in [-1.6, -3.0, -4.4] {
        w.spawn_mob(vec3(0.0, 0.0, z));
    }
    w.spawn_mob(vec3(4.0, 0.0, -3.0));
    still(&mut w);
    run(&mut w, &t, skill(1), 60);
    let d = Sk::Cut.def();
    let Form::Lane { dist, .. } = d.form else {
        panic!()
    };
    assert!(
        (w.player.pos.z + dist).abs() < 0.8 && w.player.pos.x.abs() < 0.3,
        "ends {dist} m ahead: {:?}",
        w.player.pos
    );
    for e in &w.enemies[..3] {
        assert_eq!(e.hp, 90 - d.damage - w.stats.dmg);
        assert!(e.pos.z > w.player.pos.z, "left behind");
    }
    assert_eq!(w.enemies[3].hp, 90, "off the lane");
}

/// 끌어모으기: 앞 부채꼴의 잡몹이 방랑자 앞 한 점으로 모이고 휘청인다 — 다치지는 않는다. 뒤의 적·보스·엘리트는 그대로.
#[test]
fn gather_draws_the_mobs_ahead_to_one_spot_without_hurting_them() {
    let (mut w, t) = world();
    w.set_skills([Sk::Gather, Sk::Quake]);
    for p in [(-2.5, -4.5), (2.5, -4.5), (0.0, -6.0)] {
        w.spawn_mob(vec3(p.0, 0.0, p.1));
    }
    w.spawn_mob(vec3(0.0, 0.0, 9.0));
    // 부채꼴 안이지만 끌려오는 길에서는 비켜 선 보스와 엘리트.
    w.spawn_boss(vec3(4.5, 0.0, -4.5));
    let elite = w.spawn(crate::species::Kind::Husk, vec3(-4.5, 0.0, -4.5));
    w.enemies[elite].make_elite(Affix::Hardened);
    still(&mut w);
    let was: Vec<Vec3> = w.enemies.iter().map(|e| e.pos).collect();
    let ev = run(&mut w, &t, skill(0), 14);
    assert!(ev
        .iter()
        .any(|e| matches!(e, Event::Skill { id: Sk::Gather, .. })));
    assert!(
        w.enemies[..3]
            .iter()
            .all(|e| matches!(e.state, EState::Stagger { .. })),
        "the gathered stagger"
    );
    run(&mut w, &t, Actions::default(), 50);
    let spot = vec3(0.0, 0.0, -1.8);
    for e in &w.enemies[..3] {
        assert!(flat(e.pos, spot) < 1.7, "gathered: {:?}", e.pos);
        assert_eq!(e.hp, 90, "gather does no harm");
    }
    for i in [3, 4, elite] {
        assert!(flat(w.enemies[i].pos, was[i]) < 0.05, "{i} stays");
    }
    assert!(!ev.iter().any(|e| matches!(e, Event::Hit { .. })));
}

/// 땅울림: 앞 부채꼴만 친다 — 맞은 잡몹은 휘청이고, 보스는 다치되 휘청이지 않는다(보스전의 박자를 무너뜨리지 않는다).
#[test]
fn quake_slams_the_cone_ahead_and_a_boss_does_not_stagger() {
    let (mut w, t) = world();
    // 땅울림은 양손 무기의 스킬(0223 V3).
    w.set_weapon(crate::weapons::Id::Hammer);
    w.set_skills([Sk::Quake, Sk::Gather]);
    // 겨누는 쪽 = 닿는 거리의 가장 가까운 적(앞의 것). 뒤·옆의 적은 부채꼴 밖.
    w.spawn_mob(vec3(0.5, 0.0, -3.0));
    w.spawn_mob(vec3(0.0, 0.0, 4.2));
    w.spawn_mob(vec3(4.0, 0.0, 0.5));
    w.spawn_boss(vec3(-1.0, 0.0, -3.5));
    still(&mut w);
    let boss_hp = w.enemies[3].hp;
    let ev = run(&mut w, &t, skill(0), 22);
    let d = Sk::Quake.def();
    assert_eq!(w.enemies[0].hp, 90 - forced(d.damage) - w.stats.dmg);
    assert!(matches!(w.enemies[0].state, EState::Stagger { .. }));
    assert_eq!(w.enemies[1].hp, 90, "behind");
    assert_eq!(w.enemies[2].hp, 90, "to the side");
    assert!(w.enemies[3].hp < boss_hp, "the boss is hurt");
    assert!(!matches!(w.enemies[3].state, EState::Stagger { .. }));
    assert_eq!(
        ev.iter().filter(|e| matches!(e, Event::Hit { .. })).count(),
        2
    );
}

/// 기세(0151): 스킬이 아닌 것으로 쓰러뜨리면 모든 대기가 `skill.momentum s`씩 당겨지고, 스킬로 쓰러뜨리면 그 반.
#[test]
fn felling_enemies_brings_skills_back_sooner_and_skill_kills_count_half() {
    let (mut w, t) = world();
    let down = Event::EnemyDown { at: Vec3::ZERO };
    w.player.skill.cd = [5.0; skills::N];
    let mut ev = vec![down, down];
    w.skill_books(&t, &mut ev);
    let want = 5.0 - 2.0 * t[P::SkillMomentum];
    assert!(w.player.skill.cd.iter().all(|c| (c - want).abs() < 1e-5));
    w.player.skill.cd = [5.0; skills::N];
    w.skill_hit_tick = true;
    w.skill_books(&t, &mut vec![down]);
    let want = 5.0 - t[P::SkillMomentumOwn];
    assert!(w.player.skill.cd.iter().all(|c| (c - want).abs() < 1e-5));
    assert!(t[P::SkillMomentumOwn] < t[P::SkillMomentum]);
    // 실제 판: 휘돌이가 한 대 몫(체력 10)짜리 둘을 쓰러뜨리면 다른 스킬의 대기가 "스킬 몫"만큼 당겨진다.
    let (mut w, t) = world();
    w.spawn_mob(vec3(1.5, 0.0, 0.0));
    w.spawn_mob(vec3(-1.5, 0.0, 0.0));
    still(&mut w);
    for e in &mut w.enemies {
        e.hp = 10;
    }
    w.player.skill.cd[Sk::Cut.index()] = 5.0;
    let ticks = 12;
    let ev = run(&mut w, &t, skill(0), ticks);
    let downs = ev
        .iter()
        .filter(|e| matches!(e, Event::EnemyDown { .. }))
        .count();
    assert_eq!(downs, 2);
    let cd = w.player.skill.cd[Sk::Cut.index()];
    assert!(cd < 5.0 - 2.0 * t[P::SkillMomentumOwn] + 1e-3, "{cd}");
    assert!(
        cd > 5.0 - 2.0 * t[P::SkillMomentum],
        "less than a plain kill"
    );
}

/// 게이지(0151): 맞힌 휘두르기 한 번에 한 번만 찬다(셋을 한꺼번에 베어도), 패링은 크게, 스킬의 타는 채우지 않는다.
/// 가득 차는 틱에 한 번 알린다.
#[test]
fn the_gauge_fills_once_per_swing_more_per_parry_and_never_from_skills() {
    let (mut w, t) = world();
    for x in [-0.7, 0.0, 0.7] {
        w.spawn_mob(vec3(x, 0.0, -1.4));
    }
    still(&mut w);
    let ev = run(
        &mut w,
        &t,
        Actions {
            attack: true,
            ..Default::default()
        },
        40,
    );
    let hits = ev.iter().filter(|e| matches!(e, Event::Hit { .. })).count();
    assert!(hits >= 2, "a wide swing: {hits}");
    assert!((w.player.skill.gauge - t[P::SkillGaugeHit]).abs() < 1e-5);
    // 스킬의 타는 채우지 않는다.
    let g = w.player.skill.gauge;
    let ev = run(&mut w, &t, skill(0), 60);
    assert!(ev.iter().any(|e| matches!(e, Event::Hit { .. })));
    assert_eq!(w.player.skill.gauge, g);
    // 패링 한 번 = 맞힘 여러 번.
    let parry = Event::Parry { at: Vec3::ZERO };
    w.skill_books(&t, &mut vec![parry]);
    assert!((w.player.skill.gauge - g - t[P::SkillGaugeParry]).abs() < 1e-5);
    assert!(t[P::SkillGaugeParry] >= 3.0 * t[P::SkillGaugeHit]);
    // 가득: 한 번만 알린다.
    w.player.skill.gauge = 1.0 - t[P::SkillGaugeParry] * 0.5;
    let mut ev = vec![parry];
    w.skill_books(&t, &mut ev);
    assert_eq!(w.player.skill.gauge, 1.0);
    assert!(ev.contains(&Event::UltReady));
    let mut ev = vec![parry];
    w.skill_books(&t, &mut ev);
    assert!(!ev.contains(&Event::UltReady), "already full");
}

/// 두 버튼 함께 = 큰 기술(0151 3절): 같은 틱이면 바로, 한두 틱 어긋나면 먼저 나간 스킬을 시동 안에서 바꿔치고 그 대기는
/// 돌려준다. 게이지가 안 찼으면 그냥 스킬이 나간다. 낱 누름은 늦지 않는다(누른 틱에 시동).
#[test]
fn both_skill_buttons_make_the_great_skill_even_a_few_ticks_apart() {
    let both = Actions {
        skill1: true,
        skill2: true,
        ..Default::default()
    };
    let (mut w, t) = world();
    w.player.skill.gauge = 1.0;
    let ev = w.step(&both, &t);
    assert!(matches!(w.player.state, PState::Ult { .. }));
    assert!(ev.contains(&Event::UltStart));
    assert_eq!(w.player.skill.gauge, 0.0);
    assert!(w.player.skill.cd.iter().all(|c| *c == 0.0));
    // 두 틱 어긋나게.
    let (mut w, t) = world();
    w.player.skill.gauge = 1.0;
    w.step(&skill(0), &t);
    assert!(
        matches!(w.player.state, PState::Skill { id: Sk::Whirl, .. }),
        "a single press starts at once"
    );
    w.step(&Actions::default(), &t);
    w.step(&skill(1), &t);
    assert!(matches!(w.player.state, PState::Ult { .. }));
    assert_eq!(w.player.skill.cd[Sk::Whirl.index()], 0.0, "refunded");
    // 시동 창이 지난 뒤의 둘째 누름은 큰 기술이 아니다 — 다음 스킬로 받아 둔다.
    let (mut w, t) = world();
    w.player.skill.gauge = 1.0;
    w.step(&skill(0), &t);
    let late = (t[P::SkillChord] / TICK_DT).ceil() as usize + 1;
    for _ in 0..late {
        w.step(&Actions::default(), &t);
    }
    w.step(&skill(1), &t);
    assert!(matches!(
        w.player.state,
        PState::Skill { id: Sk::Whirl, .. }
    ));
    assert_eq!(w.player.skill.gauge, 1.0);
    // 게이지가 안 찼으면 함께 눌러도 스킬 하나.
    let (mut w, t) = world();
    w.step(&both, &t);
    assert!(matches!(
        w.player.state,
        PState::Skill { id: Sk::Whirl, .. }
    ));
}

/// 쟁반 뒤집기: 둘레의 적이 모두 바깥으로 쏟아지고 휘청인다 — 가까운 적일수록 멀리. 피해는 벽에 박힌 적만. 보스는 덜 밀리고
/// 짧게 휘청인다. 땅이 뒤집히기까지는 끊기지 않는다.
#[test]
fn upend_spills_everyone_outward_and_only_walls_hurt() {
    let mut w = World::new("test", 9.0);
    let t = Tuning::default();
    w.player.facing = PI;
    w.spawn_mob(vec3(0.0, 0.0, -2.0));
    w.spawn_mob(vec3(-3.0, 0.0, 0.0));
    // 벽(아레나 끝 x = 9)에 가까운 적 — 밀려나다 박힌다.
    w.spawn_mob(vec3(6.5, 0.0, 0.0));
    w.spawn_boss(vec3(0.0, 0.0, 4.0));
    still(&mut w);
    let boss_hp = w.enemies[3].hp;
    let was: Vec<f32> = w.enemies.iter().map(|e| flat(e.pos, Vec3::ZERO)).collect();
    w.player.skill.gauge = 1.0;
    w.step(
        &Actions {
            ult: true,
            ..Default::default()
        },
        &t,
    );
    assert!(w.player.invulnerable(), "not interrupted before it lands");
    let mut ev = Vec::new();
    let mut staggers = [0.0f32; 4];
    // 가장 멀리 밀려난 거리(휘청이 풀리면 다시 다가온다).
    let mut now = was.clone();
    for _ in 0..200 {
        ev.extend(w.step(&Actions::default(), &t));
        for (i, e) in w.enemies.iter().enumerate() {
            if let EState::Stagger { dur, .. } = e.state {
                staggers[i] = staggers[i].max(dur);
            }
            now[i] = now[i].max(flat(e.pos, Vec3::ZERO));
        }
    }
    assert!(ev.iter().any(|e| matches!(e, Event::Ult { .. })));
    assert!(
        now[0] - was[0] > 4.0,
        "the near one flies: {}",
        now[0] - was[0]
    );
    assert!(now[1] - was[1] > 1.0 && now[1] - was[1] < now[0] - was[0]);
    assert_eq!(w.enemies[0].hp, 90, "no direct damage");
    assert_eq!(w.enemies[1].hp, 90);
    assert!(ev.iter().any(|e| matches!(e, Event::Slam { .. })));
    assert_eq!(
        w.enemies[2].hp,
        90 - crate::tuning::ULT_SLAM_DAMAGE,
        "thrown into the wall"
    );
    assert_eq!(w.enemies[3].hp, boss_hp);
    assert!(now[3] - was[3] < now[1] - was[1] + 2.0 && now[3] - was[3] > 0.2);
    assert!(
        staggers[3] > 0.0 && staggers[3] < staggers[0],
        "a short boss stagger"
    );
    assert!((staggers[0] - UPEND.stagger).abs() < 1e-5);
    // 둘레 밖의 적은 그대로.
    let mut w = World::new("test", 30.0);
    w.spawn_mob(vec3(0.0, 0.0, -(UPEND.radius + 1.5)));
    still(&mut w);
    w.player.skill.gauge = 1.0;
    run(
        &mut w,
        &t,
        Actions {
            ult: true,
            ..Default::default()
        },
        60,
    );
    assert!(flat(w.enemies[0].pos, vec3(0.0, 0.0, -(UPEND.radius + 1.5))) < 0.05);
    assert!(!matches!(w.enemies[0].state, EState::Stagger { .. }));
}

/// 대기 중에 누른 스킬은 나가지 않고 칸만 깜빡인다(한 번 알림) — 몇 초 뒤에 저절로 나가지도 않는다.
#[test]
fn a_cooling_skill_blinks_and_is_not_buffered() {
    let (mut w, t) = world();
    run(&mut w, &t, skill(0), 60);
    let cd = w.player.skill.cd[Sk::Whirl.index()];
    assert!(cd > 1.0);
    let ev = w.step(&skill(0), &t);
    assert!(ev.contains(&Event::SkillDenied));
    assert!(matches!(w.player.state, PState::Idle));
    assert!(w.skill_ready(0, &t) < 1.0 && w.skill_ready(1, &t) == 1.0);
    let ev = run(&mut w, &t, Actions::default(), (cd / TICK_DT) as usize + 30);
    assert!(!ev.iter().any(|e| matches!(e, Event::SkillStart { .. })));
    assert_eq!(w.skill_ready(0, &t), 1.0);
    let ev = w.step(&skill(0), &t);
    assert!(ev.contains(&Event::SkillStart { id: Sk::Whirl }));
}

/// 스킬의 후딜은 회피로 끊는다 — 판정이 끝난 뒤부터.
#[test]
fn a_skill_recovers_into_a_dodge() {
    let (mut w, t) = world();
    // 땅울림은 양손 무기의 스킬(0223 V3).
    w.set_weapon(crate::weapons::Id::Hammer);
    w.set_skills([Sk::Quake, Sk::Whirl]);
    w.step(&skill(0), &t);
    let d = Sk::Quake.def();
    let dodge = Actions {
        dodge: true,
        ..Default::default()
    };
    // 시동 중의 회피는 끊지 못한다(받아 둘 뿐).
    w.step(&dodge, &t);
    assert!(matches!(w.player.state, PState::Skill { .. }));
    let to_hit = (d.startup / TICK_DT).ceil() as usize + 2;
    for _ in 0..to_hit {
        w.step(&dodge, &t);
    }
    assert!(
        matches!(w.player.state, PState::Dodge { .. }),
        "{:?}",
        w.player.state
    );
}

// ---- 기술 판의 단·고르기·맺음(0158 G0·G1) ----

use crate::skills::{Loadout, Pick, Slot, MAX_RANK};

/// 망치의 물리력(0225)이 얹힌 값 — 땅울림은 망치의 스킬이라 늘 이 배를 받는다.
fn forced(n: i32) -> i32 {
    (n as f32 * crate::weapons::ART_FORCE).round() as i32
}

/// 스킬 `id`를 쓸 수 있는 것을 든다(0225 — 이미 맞으면 그대로): 방패 → 망치 → 쇠뇌 → 지팡이 차례로.
fn hold_for(w: &mut World, id: Sk) {
    use crate::weapons::Id as W;
    let def = id.def();
    if def.fits(w.grip()) {
        return;
    }
    w.set_shield(Some(crate::shield::Id::Pilgrim));
    if def.fits(w.grip()) {
        return;
    }
    w.set_shield(None);
    // (쇠뇌가 지팡이보다 먼저 — 타입 스킬의 시험은 바탕의 게이지로 잰다: 지팡이는 마법력의 배가 붙는다.)
    for arm in [W::Hammer, W::Crossbow, W::Staff] {
        w.set_weapon(arm);
        if def.fits(w.grip()) {
            return;
        }
    }
    panic!("{id:?} fits nothing");
}

/// 칸 0에 `id`를 `rank`단·`pick`으로 끼운다(칸 1은 남는 스킬 1단), 큰 기술은 1단.
fn wear(w: &mut World, id: Sk, rank: u8, pick: Pick) {
    // 손에 든 검으로 못 쓰는 스킬(0223 V3 → 0225)은 맞는 것을 든다: 방패 기술이면 방패를, 망치·지팡이의 것이면 그 무기를.
    hold_for(w, id);
    let other = if id == Sk::Whirl { Sk::Cut } else { Sk::Whirl };
    w.set_skills(Loadout {
        slots: [
            Slot { id, rank, pick },
            Slot {
                id: other,
                rank: 1,
                pick: Pick::A,
            },
        ],
        ult: 1,
        great: skills::Ult::Upend,
    });
}

/// 0230(사장 "범위공격이면 바닥 표시가 맞고 단일타겟이라면 크로스헤어 방식"): 숄더뷰에서 한 대상을 겨누는 스킬(던지기)은
/// 걷는 쪽이 아니라 크로스헤어의 적으로 — 크로스헤어에 적이 없으면 카메라 앞. 범위 스킬(내리찍기)은 그대로(크로스헤어를 모른다).
#[test]
fn in_the_shoulder_view_a_single_target_skill_goes_to_the_crosshair() {
    let skill_dir = |id: Sk, sight: Option<Vec2>| {
        let (mut w, t) = world();
        w.view_yaw = Some(0.0); // 카메라 앞 = -z
        wear(&mut w, id, 1, Pick::A);
        let foe = w.player.pos + vec3(-4.0, 0.0, -9.0);
        w.spawn_mob(foe);
        still(&mut w);
        let go = Actions {
            move_dir: vec2(1.0, 0.0), // 오른쪽으로 걷는다
            aim_ahead: true,
            aim_sight: sight.map(|_| vec2(foe.x, foe.z)),
            ..Default::default()
        };
        for _ in 0..20 {
            w.step(&go, &t);
        }
        w.step(&Actions { skill1: true, ..go }, &t);
        let PState::Skill { dir, .. } = w.player.state else {
            panic!("{:?}", w.player.state)
        };
        (dir, (foe - w.player.pos) * vec3(1.0, 0.0, 1.0))
    };
    let (d, to_foe) = skill_dir(Sk::Throw, Some(Vec2::ZERO));
    assert!(
        d.dot(to_foe.normalize()) > 0.99,
        "thrown at the crosshair's foe: {d:?}"
    );
    let (d, _) = skill_dir(Sk::Throw, None);
    assert!(
        d.z < -0.99,
        "no foe under the crosshair: the camera's ahead, not the walking way: {d:?}"
    );
    let (d, _) = skill_dir(Sk::Quake, Some(Vec2::ZERO));
    assert!(
        d.z > -0.5,
        "an area skill does not follow the crosshair: {d:?}"
    );
}

/// 끼운 칸의 단·고르기가 월드에 들어간다 — 같은 스킬 둘이라 바꿔 끼운 칸은 1단. 준비 막대는 그 단의 대기로 잰다.
#[test]
fn worn_ranks_reach_the_world_and_the_cooldown_bar_uses_the_ranked_cooldown() {
    let (mut w, t) = world();
    // 땅울림은 양손 무기의 스킬(0223 V3).
    w.set_weapon(crate::weapons::Id::Hammer);
    w.set_skills(Loadout {
        slots: [
            Slot {
                id: Sk::Quake,
                rank: 5,
                pick: Pick::B,
            },
            Slot {
                id: Sk::Quake,
                rank: 3,
                pick: Pick::B,
            },
        ],
        ult: 3,
        great: skills::Ult::Upend,
    });
    assert_eq!(w.player.skill.worn, [Sk::Quake, Sk::Whirl]);
    assert_eq!(w.player.skill.rank, [5, 1]);
    assert_eq!(w.player.skill.pick, [Pick::B, Pick::A]);
    assert_eq!(w.player.skill.ult_rank, 3);
    assert_eq!(w.skill_eff(Sk::Quake).skill.damage, forced(26));
    assert_eq!(w.skill_eff(Sk::Whirl).skill.damage, forced(10));
    assert_eq!(w.skill_eff(Sk::Gather).twist, None, "not worn: rank 1");
    // 5단의 대기는 ×0.9 — 쓴 직후 남은 대기가 그만큼.
    w.step(&skill(0), &t);
    let cd = w.player.skill.cd[Sk::Quake.index()];
    assert!((cd - Sk::Quake.def().cooldown * 0.9).abs() < 0.05, "{cd}");
    assert!(w.skill_ready(0, &t) < 0.02);
}

/// 2단: 피해 +15%(10 → 12). 4단: 반지름 +20% — 1단에서는 안 닿던 적이 닿는다.
#[test]
fn rank_two_hits_harder_and_rank_four_reaches_further() {
    let hp_after = |rank: u8| {
        let (mut w, t) = world();
        wear(&mut w, Sk::Whirl, rank, Pick::A);
        w.spawn_mob(vec3(1.5, 0.0, 0.0));
        // 3.0m 반지름 밖·3.6m 안(몸 반지름을 빼고).
        w.spawn_mob(vec3(
            0.0,
            0.0,
            3.3 + Enemy::new(Kind::Husk, Vec3::ZERO).radius,
        ));
        still(&mut w);
        run(&mut w, &t, skill(0), 60);
        (w.enemies[0].hp, w.enemies[1].hp)
    };
    assert_eq!(hp_after(1), (90 - 2 * 10, 90));
    assert_eq!(hp_after(2), (90 - 2 * 12, 90));
    assert_eq!(hp_after(4), (90 - 2 * 12, 90 - 2 * 12), "the far one is in");
}

/// 휘돌이 3단 ㄱ "도는 걸음": 도는 동안 스틱 쪽으로 걷는다(걷는 빠르기보다 느리게) — 고르기가 없으면 제자리.
#[test]
fn whirl_pick_a_walks_while_spinning() {
    let moved = |rank: u8, pick: Pick| {
        let (mut w, t) = world();
        wear(&mut w, Sk::Whirl, rank, pick);
        // 제자리에서는 반 걸음 모자란 자리(반지름 3m + 몸 + 0.5m).
        let r = Enemy::new(Kind::Husk, Vec3::ZERO).radius;
        w.spawn_mob(vec3(0.0, 0.0, -(3.5 + r)));
        still(&mut w);
        let go = Actions {
            move_dir: vec2(0.0, 1.0),
            ..Default::default()
        };
        w.step(&Actions { skill1: true, ..go }, &t);
        let mut ticks = 1;
        while matches!(w.player.state, PState::Skill { .. }) {
            w.step(&go, &t);
            ticks += 1;
        }
        (flat(w.player.pos, Vec3::ZERO), ticks as f32 * TICK_DT, w, t)
    };
    let (d, secs, w, t) = moved(3, Pick::A);
    assert!(d > 1.0, "walked while spinning: {d}");
    assert!(
        d < t[P::MoveSpeed] * secs * 0.75,
        "but slower than a walk: {d}"
    );
    assert!(w.player.pos.z < -1.0, "towards the stick (screen up)");
    // 걸어 들어가서 닿았다: 반 걸음 밖의 적은 제자리에서는 안 닿는다.
    assert!(w.enemies[0].hp < 90, "the walk carried the ring to it");
    let (d, _, w, _) = moved(2, Pick::A);
    assert!(d < 0.05, "no pick yet: stands still ({d})");
    assert_eq!(w.enemies[0].hp, 90);
    let (d, ..) = moved(3, Pick::B);
    assert!(d < 0.05, "the other pick stands still ({d})");
}

/// 휘돌이 3단 ㄴ "셋째 바퀴": 세 번 돈다 — 둘레의 적이 세 번 맞고, 미는 것은 여전히 마지막 바퀴.
#[test]
fn whirl_pick_b_spins_a_third_time() {
    let (mut w, t) = world();
    wear(&mut w, Sk::Whirl, 3, Pick::B);
    w.spawn_mob(vec3(2.0, 0.0, 0.0));
    w.spawn_mob(vec3(-2.0, 0.0, 0.0));
    still(&mut w);
    let ev = run(&mut w, &t, skill(0), 80);
    let spins = ev
        .iter()
        .filter(|e| matches!(e, Event::Skill { id: Sk::Whirl, .. }))
        .count();
    assert_eq!(spins, 3);
    let dmg = w.skill_eff(Sk::Whirl).skill.damage;
    assert_eq!(dmg, 12);
    for e in &w.enemies {
        assert_eq!(e.hp, 90 - 3 * dmg, "once per spin");
        assert!(flat(e.pos, Vec3::ZERO) > 2.8, "pushed out by the last spin");
    }
    assert!(matches!(w.player.state, PState::Idle));
    assert!(w.player.facing_dir().dot(vec3(0.0, 0.0, -1.0)) > 0.99);
}

/// 가로지르기 3단 ㄱ "되돌아 베기": 끝에서 온 길을 되돌아와 처음 자리 곁에 서고, 길 위의 적은 두 번 베인다.
#[test]
fn cut_pick_a_dashes_back_and_cuts_again() {
    let (mut w, t) = world();
    wear(&mut w, Sk::Cut, 3, Pick::A);
    for z in [-1.6, -3.0, -4.4] {
        w.spawn_mob(vec3(0.0, 0.0, z));
    }
    w.spawn_mob(vec3(4.0, 0.0, -3.0));
    still(&mut w);
    let mut ev = w.step(&skill(0), &t);
    let mut far: f32 = 0.0;
    // 맞힐 때마다 히트스톱이 끼니 끝날 때까지 돈다.
    for _ in 0..400 {
        ev.extend(w.step(&Actions::default(), &t));
        far = far.max(-w.player.pos.z);
        if matches!(w.player.state, PState::Idle) {
            break;
        }
    }
    assert!(far > 5.0, "went the whole lane first: {far}");
    assert!(
        flat(w.player.pos, Vec3::ZERO) < 0.9,
        "back at the start: {:?}",
        w.player.pos
    );
    // 가는 길의 타가 금을 내고(균열 타입 — 0223 V2) 오는 길의 타는 그 금에 더 아프다.
    let dmg = w.skill_eff(Sk::Cut).skill.damage + w.stats.dmg;
    let back = skills::scaled(dmg, crate::tuning::RIFT_MULT);
    for e in &w.enemies[..3] {
        assert!(e.ail.rift > 0.0);
        assert_eq!(e.hp, 90 - dmg - back, "cut going and coming");
    }
    assert_eq!(w.enemies[3].hp, 90, "off the lane");
    let flashes: Vec<Vec3> = ev
        .iter()
        .filter_map(|e| match e {
            Event::Skill {
                id: Sk::Cut, dir, ..
            } => Some(*dir),
            _ => None,
        })
        .collect();
    assert_eq!(flashes.len(), 2);
    assert!(
        flashes[0].dot(flashes[1]) < -0.99,
        "the second one faces back"
    );
    assert!(matches!(w.player.state, PState::Idle));
    // 고르기가 없으면 한 번만, 반대편에 선다.
    let (mut w, t) = world();
    wear(&mut w, Sk::Cut, 2, Pick::A);
    w.spawn_mob(vec3(0.0, 0.0, -3.0));
    still(&mut w);
    run(&mut w, &t, skill(0), 120);
    assert!(w.player.pos.z < -5.0);
    assert_eq!(w.enemies[0].hp, 90 - 23);
}

/// 가로지르기 3단 ㄴ "스치는 몸": 지나는 동안에만 맞지 않는다 — 시동과 후딜은 맞는다. ㄱ을 골랐으면 내내 맞는다.
#[test]
fn cut_pick_b_cannot_be_hit_while_dashing() {
    let safe_ticks = |pick: Pick| {
        let (mut w, t) = world();
        wear(&mut w, Sk::Cut, 3, pick);
        w.step(&skill(0), &t);
        let mut log = vec![w.player.invulnerable()];
        while matches!(w.player.state, PState::Skill { .. }) {
            w.step(&Actions::default(), &t);
            log.push(w.player.invulnerable());
        }
        log
    };
    let b = safe_ticks(Pick::B);
    let d = Sk::Cut.def();
    let dash = (d.active() / TICK_DT).round() as usize;
    let safe = b.iter().filter(|x| **x).count();
    assert!(
        safe + 2 >= dash && safe <= dash + 2,
        "safe for the dash: {safe} of {dash}"
    );
    assert!(!b[0], "the startup can be hit");
    assert!(!b[b.len() - 1] && !b[b.len() - 3], "so can the recovery");
    assert!(safe_ticks(Pick::A).iter().all(|x| !x), "the other pick");
    // 실제로 맞지 않는다: 지나는 길에서 내리치는 적 — ㄱ이면 같은 자리에서 맞는다.
    let hurt = |pick: Pick| {
        let (mut w, t) = world();
        wear(&mut w, Sk::Cut, 3, pick);
        let into_dash = (d.startup / TICK_DT).ceil() as usize + 3;
        run(&mut w, &t, skill(0), into_dash);
        assert!(matches!(w.player.state, PState::Skill { .. }));
        let hp = w.player.hp;
        w.spawn_mob(w.player.pos + vec3(0.9, 0.0, -1.0));
        w.enemies[0].state = EState::Swing { t: 0.0 };
        let ev = w.step(&Actions::default(), &t);
        (hp - w.player.hp, ev.contains(&Event::PlayerHurt))
    };
    assert_eq!(hurt(Pick::B), (0, false), "slips through the blow");
    let (lost, ev) = hurt(Pick::A);
    assert!(lost > 0 && ev, "the same blow lands without the pick");
}

/// 끌어모으기 3단 ㄱ "모인 자리": 0.5초 뒤 모인 점이 터져 그 둘레의 적이 한 눈금(10) 다친다 — 멀리 있는 적은 그대로.
/// 터짐도 스킬의 타라 게이지를 채우지 않는다. 고르기가 없으면 터지지 않는다.
#[test]
fn gather_pick_a_bursts_where_they_gathered() {
    let (mut w, t) = world();
    wear(&mut w, Sk::Gather, 3, Pick::A);
    for p in [(-2.5, -4.5), (2.5, -4.5), (0.0, -6.0)] {
        w.spawn_mob(vec3(p.0, 0.0, p.1));
    }
    w.spawn_mob(vec3(0.0, 0.0, 9.0));
    still(&mut w);
    let d = Sk::Gather.def();
    let pull = (d.startup / TICK_DT).ceil() as usize + 1;
    let ev = run(&mut w, &t, skill(0), pull);
    let spot = ev
        .iter()
        .find_map(|e| match e {
            Event::Skill {
                id: Sk::Gather, at, ..
            } => Some(*at),
            _ => None,
        })
        .expect("it pulls");
    assert!(
        w.enemies.iter().all(|e| e.hp == 90),
        "the pull does no harm"
    );
    // 0.5초가 다 되기 전에는 아직.
    let ev = run(&mut w, &t, Actions::default(), 24);
    assert!(!ev.iter().any(|e| matches!(e, Event::SkillBurst { .. })));
    let ev = run(&mut w, &t, Actions::default(), 20);
    let at = ev
        .iter()
        .find_map(|e| match e {
            Event::SkillBurst { at } => Some(*at),
            _ => None,
        })
        .expect("it bursts");
    assert!(flat(at, spot) < 1e-3, "at the gather point");
    for e in &w.enemies[..3] {
        assert_eq!(e.hp, 90 - crate::tuning::HP_UNIT);
    }
    assert_eq!(w.enemies[3].hp, 90, "far behind");
    assert_eq!(w.player.skill.gauge, 0.0, "a skill hit: no gauge");
    let ev = run(&mut w, &t, Actions::default(), 120);
    assert!(
        !ev.iter().any(|e| matches!(e, Event::SkillBurst { .. })),
        "once"
    );
    // 2단(고르기 없음)·3단 ㄴ은 터지지 않는다.
    for (rank, pick) in [(2, Pick::A), (3, Pick::B)] {
        let (mut w, t) = world();
        wear(&mut w, Sk::Gather, rank, pick);
        w.spawn_mob(vec3(0.0, 0.0, -5.0));
        still(&mut w);
        let ev = run(&mut w, &t, skill(0), 120);
        assert!(!ev.iter().any(|e| matches!(e, Event::SkillBurst { .. })));
        assert_eq!(w.enemies[0].hp, 90);
    }
}

/// 끌어모으기 3단 ㄴ "무거운 것도": 엘리트도 끌려온다 — 보스는 여전히 끌리지 않는다. ㄱ이면 엘리트는 그대로.
#[test]
fn gather_pick_b_pulls_elites_too_but_never_a_boss() {
    let pulled = |pick: Pick| {
        let (mut w, t) = world();
        wear(&mut w, Sk::Gather, 3, pick);
        // 가장 가까운 잡몹 쪽(곧장 앞)을 겨눈다 — 보스와 엘리트는 그 부채꼴 안, 끌려오는 길 밖.
        w.spawn_mob(vec3(0.0, 0.0, -3.0));
        w.spawn_boss(vec3(3.0, 0.0, -5.0));
        let elite = w.spawn(crate::species::Kind::Husk, vec3(-3.0, 0.0, -5.0));
        w.enemies[elite].make_elite(Affix::Repeating);
        still(&mut w);
        let was: Vec<Vec3> = w.enemies.iter().map(|e| e.pos).collect();
        let mut staggered = [false; 3];
        w.step(&skill(0), &t);
        for _ in 0..70 {
            w.step(&Actions::default(), &t);
            for (i, e) in w.enemies.iter().enumerate() {
                staggered[i] |= matches!(e.state, EState::Stagger { .. });
            }
        }
        let moved: Vec<f32> = w
            .enemies
            .iter()
            .zip(&was)
            .map(|(e, p)| flat(e.pos, *p))
            .collect();
        (moved, staggered)
    };
    let (moved, staggered) = pulled(Pick::B);
    assert!(moved[0] > 0.5 && staggered[0], "the mob comes: {moved:?}");
    assert!(
        moved[2] > 2.0 && staggered[2],
        "the elite comes too: {moved:?}"
    );
    assert!(moved[1] < 0.05 && !staggered[1], "the boss never");
    let (moved, staggered) = pulled(Pick::A);
    assert!(moved[0] > 0.5);
    assert!(
        moved[2] < 0.05 && !staggered[2],
        "without the pick the elite stays"
    );
    assert!(moved[1] < 0.05 && !staggered[1]);
}

/// 땅울림 3단 ㄱ "멀리 뛰기": 겨눈 쪽으로 6m까지 뛰어들어 찍는다 — 그 길의 가장 가까운 적 1.5m쯤 앞에서 멈추고(먼 적에게
/// 닿는다), 바로 앞에 적이 있으면 거의 뛰지 않는다. 길에 적이 없으면 6m를 다 뛴다. 고르기가 없으면 제자리.
#[test]
fn quake_pick_a_leaps_in_and_stops_short_of_the_nearest_enemy() {
    let leap = |rank: u8, pick: Pick, enemy_z: Option<f32>| {
        let (mut w, t) = world();
        wear(&mut w, Sk::Quake, rank, pick);
        if let Some(z) = enemy_z {
            w.spawn_mob(vec3(0.0, 0.0, z));
        }
        still(&mut w);
        let d = Sk::Quake.def();
        let land = (d.startup / TICK_DT).ceil() as usize + 2;
        let ev = run(&mut w, &t, skill(0), land);
        let hit = ev.iter().any(|e| matches!(e, Event::Hit { .. }));
        (-w.player.pos.z, w.enemies.first().map(|e| e.pos), hit)
    };
    // 9m 앞의 적: 1단의 부채꼴(5m)은 못 닿는다 — 뛰어들면 닿는다.
    let (went, en, hit) = leap(3, Pick::A, Some(-9.0));
    let gap = flat(vec3(0.0, 0.0, -went), en.unwrap());
    assert!(went > 4.5 && went <= 6.05, "a long leap: {went}");
    assert!(hit, "and the slam reaches it");
    assert!(gap > 1.2 && gap < 4.0, "lands short of it: {gap}");
    // 7m 앞: 6m를 다 뛰지 않고 적 앞에서 멈춘다.
    let (went, en, hit) = leap(3, Pick::A, Some(-7.0));
    let r = Enemy::new(Kind::Husk, Vec3::ZERO).radius;
    let want = 7.0 - r - 1.5;
    assert!(
        (went - want).abs() < 0.6,
        "stops 1.5 m short: {went} vs {want}"
    );
    assert!(hit && flat(vec3(0.0, 0.0, -went), en.unwrap()) > 1.3);
    // 바로 앞(2m): 큰 뜀이 없다.
    let (went, _, hit) = leap(3, Pick::A, Some(-2.0));
    assert!(went < 0.4, "a close target: no big leap ({went})");
    assert!(hit);
    // 길에 아무도 없으면 끝까지.
    let (went, ..) = leap(3, Pick::A, None);
    assert!(went > 5.2 && went <= 6.05, "{went}");
    // 고르기가 없거나 ㄴ이면 제자리에서 찍는다 — 9m의 적은 못 닿는다.
    for (rank, pick) in [(2, Pick::A), (3, Pick::B)] {
        let (went, _, hit) = leap(rank, pick, Some(-9.0));
        assert!(went.abs() < 0.05 && !hit, "{rank} {pick:?}: {went}");
    }
}

/// 땅울림 3단 ㄴ "갈라진 땅": 찍은 부채꼴이 2초 남는다 — 그 안의 적은 반의 빠르기로 걷고(밖의 적·보스는 그대로), 2초 뒤 사라진다.
#[test]
fn quake_pick_b_leaves_cracked_ground_that_slows() {
    let (mut w, t) = world();
    wear(&mut w, Sk::Quake, 3, Pick::B);
    w.spawn_mob(vec3(0.0, 0.0, -4.0));
    w.spawn_mob(vec3(0.0, 0.0, 4.0));
    w.spawn_boss(vec3(2.0, 0.0, -3.0));
    still(&mut w);
    let d = Sk::Quake.def();
    let land = (d.startup / TICK_DT).ceil() as usize + 1;
    run(&mut w, &t, skill(0), land);
    assert_eq!(w.skill_zones.len(), 1, "the slam leaves a zone");
    let z = w.skill_zones[0];
    assert!(z.holds(vec3(0.0, 0.0, -4.0), 0.0) && !z.holds(vec3(0.0, 0.0, 4.0), 0.0));
    w.step(&Actions::default(), &t);
    assert_eq!(w.enemies[0].marks.slow, 0.5, "inside");
    assert_eq!(w.enemies[1].marks.slow, 1.0, "behind: outside the cone");
    assert_eq!(w.enemies[2].marks.slow, 1.0, "a boss is not slowed");
    // 실제 걸음: 안의 적과 밖의 적을 같은 거리에서 쫓게 하면 안쪽이 반만 간다.
    for e in &mut w.enemies[..2] {
        e.state = EState::Chase;
        e.knock = Vec3::ZERO;
    }
    let was = [w.enemies[0].pos, w.enemies[1].pos];
    for _ in 0..10 {
        w.step(&Actions::default(), &t);
    }
    let (slow, free) = (
        flat(w.enemies[0].pos, was[0]),
        flat(w.enemies[1].pos, was[1]),
    );
    assert!(free > 0.1, "the free one walks: {free}");
    assert!(
        (slow / free - 0.5).abs() < 0.12,
        "half speed on the cracked ground: {slow} vs {free}"
    );
    // 2초 뒤: 땅이 아물고 빠르기가 돌아온다.
    still(&mut w);
    run(&mut w, &t, Actions::default(), 125);
    assert!(w.skill_zones.is_empty());
    assert!(w.enemies.iter().all(|e| e.marks.slow == 1.0));
    // 고르기가 없으면 남는 땅이 없다.
    let (mut w, t) = world();
    wear(&mut w, Sk::Quake, 3, Pick::A);
    w.spawn_mob(vec3(0.0, 0.0, -3.0));
    still(&mut w);
    run(&mut w, &t, skill(0), 40);
    assert!(w.skill_zones.is_empty());
}

/// 휘돌이 5단 맺음: 마지막 바퀴에 맞은 적이 0.4초 휘청인다 — 보스는 휘청이지 않는다. 4단까지는 휘청이 없다.
#[test]
fn whirl_capstone_staggers_on_the_last_spin_only() {
    let staggers = |rank: u8| {
        let (mut w, t) = world();
        wear(&mut w, Sk::Whirl, rank, Pick::A);
        w.spawn_mob(vec3(2.0, 0.0, 0.0));
        w.spawn_boss(vec3(-2.0, 0.0, 0.0));
        still(&mut w);
        let d = w.skill_eff(Sk::Whirl);
        w.step(&skill(0), &t);
        let mut first: Option<(f32, f32)> = None;
        let mut boss = false;
        let mut secs = TICK_DT;
        while matches!(w.player.state, PState::Skill { .. }) {
            w.step(&Actions::default(), &t);
            secs += TICK_DT;
            if let (EState::Stagger { dur, .. }, None) = (w.enemies[0].state, first) {
                first = Some((secs, dur));
            }
            boss |= matches!(w.enemies[1].state, EState::Stagger { .. });
        }
        (first, boss, d)
    };
    let (first, boss, d) = staggers(MAX_RANK);
    let (at, dur) = first.expect("the mob staggers");
    assert!((dur - 0.4).abs() < 1e-6);
    assert!(
        at > d.skill.startup + d.active() - 0.05,
        "on the last spin, not the first: {at}"
    );
    assert!(!boss, "a boss does not stagger");
    let (first, boss, _) = staggers(4);
    assert!(first.is_none() && !boss, "no capstone below rank 5");
}

/// 가로지르기 5단 맺음: 이 스킬로 쓰러뜨리면 대기의 반을 돌려받는다 — 둘을 쓰러뜨려도 한 번. 못 쓰러뜨리면 그대로.
#[test]
fn cut_capstone_refunds_half_the_cooldown_on_a_kill_once() {
    let cd_after = |kills: usize, rank: u8| {
        let (mut w, t) = world();
        wear(&mut w, Sk::Cut, rank, Pick::B);
        for z in [-2.0, -4.0] {
            w.spawn_mob(vec3(0.0, 0.0, z));
        }
        still(&mut w);
        for e in w.enemies.iter_mut().take(kills) {
            e.hp = 10;
        }
        let ticks = 40;
        let ev = run(&mut w, &t, skill(0), ticks);
        let downs = ev
            .iter()
            .filter(|e| matches!(e, Event::EnemyDown { .. }))
            .count();
        assert_eq!(downs, kills);
        (w.player.skill.cd[Sk::Cut.index()], w, t)
    };
    let (none, w, t) = cd_after(0, MAX_RANK);
    let full = w.skill_eff(Sk::Cut).skill.cooldown * t[P::SkillCooldownX];
    assert!((full - Sk::Cut.def().cooldown * 0.9).abs() < 1e-4);
    let own = t[P::SkillMomentumOwn];
    let (one, ..) = cd_after(1, MAX_RANK);
    assert!(
        (none - one - (full * 0.5 + own)).abs() < 0.05,
        "half back (+ the usual momentum): {none} -> {one}"
    );
    let (two, ..) = cd_after(2, MAX_RANK);
    assert!(
        (one - two - own).abs() < 0.05,
        "the second kill only adds momentum: {one} -> {two}"
    );
    // 4단: 쓰러뜨려도 기세만.
    let (none4, ..) = cd_after(0, 4);
    let (one4, ..) = cd_after(1, 4);
    assert!((none4 - one4 - own).abs() < 0.05, "{none4} -> {one4}");
}

/// 끌어모으기 5단 맺음: 끌려온 적은 2초 동안 피해를 20% 더 받는다(10 → 12) — 끌려오지 않은 적은 그대로, 2초 뒤에는 풀린다.
#[test]
fn gather_capstone_makes_the_gathered_take_more_for_two_seconds() {
    let (mut w, t) = world();
    w.set_skills(Loadout {
        slots: [
            Slot {
                id: Sk::Gather,
                rank: MAX_RANK,
                pick: Pick::B,
            },
            Slot {
                id: Sk::Whirl,
                rank: 1,
                pick: Pick::A,
            },
        ],
        ult: 1,
        great: skills::Ult::Upend,
    });
    // 앞의 적이 더 가깝다(겨누는 쪽) — 뒤의 적은 부채꼴 밖.
    w.spawn_mob(vec3(0.0, 0.0, -2.2));
    w.spawn_mob(vec3(0.0, 0.0, 2.6));
    still(&mut w);
    run(&mut w, &t, skill(0), 40);
    assert!(w.enemies[0].marks.frail > 1.0 && w.enemies[0].marks.frail <= 2.0);
    assert_eq!(w.enemies[1].marks.frail, 0.0, "behind: not gathered");
    assert_eq!(w.enemies[0].hp, 90);
    // 모인 적 곁에서 휘돌이(1단, 한 타 10): 모인 적은 12씩, 뒤의 적은 10씩.
    run(&mut w, &t, skill(1), 60);
    assert_eq!(w.enemies[0].hp, 90 - 2 * 12, "{:?}", w.enemies[0].pos);
    assert_eq!(w.enemies[1].hp, 90 - 2 * 10);
    // 2초가 지나면 풀린다.
    still(&mut w);
    run(&mut w, &t, Actions::default(), 90);
    assert_eq!(w.enemies[0].marks.frail, 0.0);
    assert_eq!(hurt_by(&w.enemies[0], 10, false, 0, &t), 10);
    // 4단에는 없다.
    let (mut w, t) = world();
    wear(&mut w, Sk::Gather, 4, Pick::A);
    w.spawn_mob(vec3(0.0, 0.0, -4.0));
    still(&mut w);
    run(&mut w, &t, skill(0), 20);
    assert_eq!(w.enemies[0].marks.frail, 0.0);
}

/// 땅울림 5단 맺음: 자세가 깨져 있는 적에게 두 배 — 멀쩡한 적은 그대로(26). 4단에는 없다.
#[test]
fn quake_capstone_doubles_against_broken_poise() {
    let hits = |rank: u8| {
        let (mut w, t) = world();
        wear(&mut w, Sk::Quake, rank, Pick::B);
        w.spawn_mob(vec3(-1.0, 0.0, -3.0));
        w.spawn_mob(vec3(1.0, 0.0, -3.0));
        still(&mut w);
        for e in &mut w.enemies {
            e.hp = 500;
            e.max_hp = 500;
        }
        w.enemies[0].stance.broken = 5.0;
        run(&mut w, &t, skill(0), 30);
        (500 - w.enemies[0].hp, 500 - w.enemies[1].hp, t)
    };
    let (broken, whole, t) = hits(MAX_RANK);
    assert_eq!(whole, forced(26));
    // 깨진 동안의 ×1.5(옛 눈금 올림)에 맺음의 두 배.
    let opened = crate::tuning::scale_up(forced(26), t[P::PoiseBreakDamage]);
    assert_eq!(broken, 2 * opened, "double on top of the open-poise bonus");
    let (broken, whole, t) = hits(4);
    assert_eq!(whole, forced(23));
    assert_eq!(
        broken,
        crate::tuning::scale_up(forced(23), t[P::PoiseBreakDamage])
    );
}

/// 큰 기술의 단: 2단 = 휘청 +0.6초(보스의 짧은 휘청은 그대로), 3단 = 벽에 박힌 피해 ×1.5(20 → 30). 직접 피해는 여전히 없다.
#[test]
fn upend_ranks_lengthen_the_stagger_and_harden_the_wall() {
    let cast = |rank: u8| {
        let mut w = World::new("test", 9.0);
        let t = Tuning::default();
        w.player.facing = PI;
        w.set_skills(Loadout {
            ult: rank,
            ..Loadout::from(skills::START)
        });
        w.spawn_mob(vec3(0.0, 0.0, -2.0));
        w.spawn_mob(vec3(6.5, 0.0, 0.0));
        w.spawn_boss(vec3(0.0, 0.0, 4.0));
        still(&mut w);
        let boss_hp = w.enemies[2].hp;
        w.player.skill.gauge = 1.0;
        w.step(
            &Actions {
                ult: true,
                ..Default::default()
            },
            &t,
        );
        let mut durs = [0.0f32; 3];
        for _ in 0..200 {
            w.step(&Actions::default(), &t);
            for (i, e) in w.enemies.iter().enumerate() {
                if let (EState::Stagger { dur, .. }, true) = (e.state, durs[i] == 0.0) {
                    durs[i] = dur;
                }
            }
        }
        assert_eq!(w.enemies[0].hp, 90, "no direct damage at rank {rank}");
        assert_eq!(w.enemies[2].hp, boss_hp);
        (durs, 90 - w.enemies[1].hp)
    };
    let (d1, wall1) = cast(1);
    let (d2, wall2) = cast(2);
    let (d3, wall3) = cast(3);
    assert!((d1[0] - UPEND.stagger).abs() < 1e-5);
    assert!((d2[0] - UPEND.stagger - 0.6).abs() < 1e-5, "{d2:?}");
    assert!((d3[0] - d2[0]).abs() < 1e-5);
    for d in [d1, d2, d3] {
        assert!(
            (d[2] - UPEND.boss_stagger).abs() < 1e-5,
            "a boss's short stagger does not grow: {d:?}"
        );
    }
    assert_eq!((wall1, wall2, wall3), (20, 20, 30));
}

/// 어느 단·어느 고르기에서도(0151의 규칙): 보스는 끌리지 않고·스킬로 휘청이지 않고·느려지지 않고·더 다치는 표지도 붙지 않는다.
/// 스킬의 타(되돌아 베기·셋째 바퀴·모인 자리의 터짐까지)는 큰 기술 게이지를 채우지 않고, 스킬은 기력을 쓰지 않는다.
#[test]
fn at_max_rank_with_every_pick_a_boss_is_never_pulled_or_staggered_and_skills_stay_free() {
    for id in skills::IDS {
        for pick in [Pick::A, Pick::B] {
            let (mut w, t) = world();
            wear(&mut w, id, MAX_RANK, pick);
            // 보스는 어느 모양에도 닿는 자리(앞 2.5m), 잡몹 하나는 그 곁. 끌어모으기는 보스가 부채꼴 안·끌려오는 길 밖에
            // 서게(끌려온 잡몹의 몸에 밀리는 것은 끌림이 아니다).
            if id == Sk::Gather {
                w.spawn_boss(vec3(3.0, 0.0, -5.0));
                w.spawn_mob(vec3(0.0, 0.0, -3.0));
            } else {
                w.spawn_boss(vec3(0.6, 0.0, -2.5));
                w.spawn_mob(vec3(-1.2, 0.0, -2.5));
            }
            still(&mut w);
            let (boss_was, boss_hp) = (w.enemies[0].pos, w.enemies[0].hp);
            let stamina = w.player.stamina;
            let tag = format!("{id:?} {pick:?}");
            let mut ev = w.step(&skill(0), &t);
            for k in 0..150 {
                // 되울림(G2)은 패링이 있어야 울린다: 울림을 두른 뒤 잡몹이 앞에서 내려치게 하고 받아친다(보스는 둘레 안).
                let a = if id == Sk::Echo && k == 30 {
                    let en = &mut w.enemies[1];
                    en.pos = vec3(0.0, 0.0, -1.2);
                    en.facing = 0.0;
                    en.atk = Atk::Smash;
                    en.state = EState::Tele { t: en.tele_dur() };
                    w.player.facing = PI;
                    Actions {
                        parry: true,
                        ..Default::default()
                    }
                } else {
                    Actions::default()
                };
                ev.extend(w.step(&a, &t));
                let b = &w.enemies[0];
                assert!(
                    !matches!(b.state, EState::Stagger { .. }),
                    "{tag}: staggered"
                );
                assert_eq!(b.marks.slow, 1.0, "{tag}: slowed");
                assert_eq!(b.marks.frail, 0.0, "{tag}: marked");
            }
            assert!(
                ev.iter().any(|e| matches!(e, Event::Skill { .. })),
                "{tag}: it went off"
            );
            // 끌려오지 않았다: 밀려난 것(넉백)은 있어도 방랑자 쪽으로 오지는 않는다.
            let (was, now) = (
                flat(boss_was, Vec3::ZERO),
                flat(w.enemies[0].pos, w.player.pos),
            );
            if id == Sk::Gather {
                assert!(flat(w.enemies[0].pos, boss_was) < 0.05, "{tag}: pulled");
                assert_eq!(w.enemies[0].hp, boss_hp, "{tag}: gather alone does no harm");
            } else if id == Sk::Swallow || id == Sk::Ram || (id == Sk::Stitch && pick == Pick::A) {
                // 제비걸음(0214)은 방랑자가 뛰어든다 — 보스가 처음 자리에서 끌려오지 않았는지만.
                let moved = flat(w.enemies[0].pos, Vec3::ZERO);
                assert!(moved > was - 0.6, "{tag}: drawn in ({was} -> {moved})");
            } else if !(id == Sk::Quake && pick == Pick::A) {
                assert!(now > was - 0.6, "{tag}: drawn in ({was} -> {now})");
            }
            // 패링 한 번은 패링의 몫만 채운다(울림의 타는 채우지 않는다).
            let parries = ev
                .iter()
                .filter(|e| matches!(e, Event::Parry { .. }))
                .count();
            if id == Sk::Echo {
                assert_eq!(parries, 1, "{tag}: the echo test parries once");
                assert!(
                    ev.iter().any(|e| matches!(e, Event::EchoRing { .. })),
                    "{tag}"
                );
            }
            let want = parries as f32 * t[P::SkillGaugeParry];
            assert!(
                (w.player.skill.gauge - want).abs() < 1e-5,
                "{tag}: skill hits fill no gauge ({})",
                w.player.skill.gauge
            );
            assert_eq!(w.player.stamina, stamina, "{tag}: no stamina");
            assert!(w.player.skill.cd[id.index()] > 1.0, "{tag}: cooling down");
        }
    }
}

// ---- G2(0158): 나머지 스킬 넷 · 큰 기술 둘 ----

use crate::skills::Ult;

/// 칸 0에 `id`를 끼우고 큰 기술 `great`를 `ult`단으로.
fn wear_ult(w: &mut World, great: Ult, ult: u8) {
    w.set_skills(Loadout {
        slots: [
            Slot {
                id: Sk::Whirl,
                rank: 1,
                pick: Pick::A,
            },
            Slot {
                id: Sk::Cut,
                rank: 1,
                pick: Pick::A,
            },
        ],
        ult,
        great,
    });
}

/// 0172: 큰 기술을 아직 안 배웠으면(단 0 — 새 판) 게이지가 없다 — 패링을 해도 차지 않고 "가득"도 알리지 않으며, 두 스킬
/// 버튼을 함께 눌러도·터치의 큰 기술 버튼을 눌러도 큰 기술은 나가지 않는다(끼운 스킬이 나간다). 배우면(단 1) 그때부터 찬다.
#[test]
fn without_a_great_skill_there_is_no_gauge_and_the_chord_does_nothing() {
    let (mut w, t) = world();
    wear_ult(&mut w, Ult::Upend, 0);
    assert_eq!(w.player.skill.ult_rank, 0);
    let parry = Event::Parry { at: Vec3::ZERO };
    for _ in 0..40 {
        let mut ev = vec![parry];
        w.skill_books(&t, &mut ev);
        assert!(!ev.contains(&Event::UltReady));
    }
    assert_eq!(w.player.skill.gauge, 0.0, "nothing to fill");
    // 함께 누름: 큰 기술이 아니라 칸의 스킬.
    still(&mut w);
    let both = Actions {
        skill1: true,
        skill2: true,
        ..Default::default()
    };
    let ev = run(&mut w, &t, both, 30);
    assert!(!ev.iter().any(|e| matches!(e, Event::UltStart)));
    assert!(ev.iter().any(|e| matches!(e, Event::SkillStart { .. })));
    let touch = Actions {
        ult: true,
        ..Default::default()
    };
    let mut w2 = world().0;
    wear_ult(&mut w2, Ult::Upend, 0);
    still(&mut w2);
    let ev = run(&mut w2, &t, touch, 30);
    assert!(!ev.iter().any(|e| matches!(e, Event::UltStart)));
    // 배우면 찬다.
    wear_ult(&mut w, Ult::Upend, 1);
    w.skill_books(&t, &mut vec![parry]);
    assert!(w.player.skill.gauge > 0.0);
}

/// 앞 1.2m에서 이 틱에 내려치는 잔재(맞으면 한 눈금).
fn swing_at_player(w: &mut World, i: usize) {
    let p = w.player.pos;
    let en = &mut w.enemies[i];
    en.pos = p + vec3(0.0, 0.0, -1.2);
    en.prev = en.pos;
    en.facing = 0.0;
    en.atk = Atk::Smash;
    en.state = EState::Swing { t: 0.0 };
}

fn ticks(secs: f32) -> usize {
    (secs / TICK_DT).ceil() as usize
}

/// 버팀: 버티는 동안 맞아도 밀리지도 끊기지도 않고 한 대 몫(10)을 덜어 다치지 않는다 — 받아낸 타는 셋까지 센다.
/// 끝의 터뜨림은 둘레 3m에 10 + 받아낸 타마다 10(셋이면 40). 아무것도 안 받아냈으면 10.
#[test]
fn brace_soaks_blows_without_being_moved_and_bursts_harder_for_each() {
    let burst = |blows: usize| {
        let (mut w, t) = world();
        wear(&mut w, Sk::Brace, 1, Pick::A);
        let hitter = w.spawn(Kind::Husk, vec3(0.0, 0.0, -1.2));
        let side = w.spawn(Kind::Husk, vec3(2.0, 0.0, 1.0));
        w.spawn_mob(vec3(0.0, 0.0, 6.0));
        still(&mut w);
        let (hp, at) = (w.player.hp, w.player.pos);
        let mut ev = run(&mut w, &t, skill(0), ticks(0.12));
        for _ in 0..blows {
            swing_at_player(&mut w, hitter);
            ev.extend(run(&mut w, &t, Actions::default(), 2));
            assert!(
                matches!(w.player.state, PState::Skill { id: Sk::Brace, .. }),
                "not interrupted: {:?}",
                w.player.state
            );
            w.enemies[hitter].state = EState::Recover { t: -1000.0 };
            ev.extend(run(&mut w, &t, Actions::default(), ticks(0.2)));
        }
        ev.extend(run(&mut w, &t, Actions::default(), 120));
        let soaks = ev
            .iter()
            .filter(|e| matches!(e, Event::Soak { .. }))
            .count();
        assert_eq!(soaks, blows, "every blow is soaked");
        assert_eq!(w.player.hp, hp, "a blow of 10 minus 10");
        assert!(flat(w.player.pos, at) < 0.05, "not pushed");
        assert!(!ev.contains(&Event::PlayerHurt), "no hurt feedback");
        // 터뜨림은 잔불 타입(0223 V2)이라 맞은 적에 화상이 붙는다 — 그 뒤에 탄 몫은 빼고 터뜨림만 센다.
        let side_at = w.enemies[side].pos;
        let burns = ev
            .iter()
            .filter(|e| matches!(e, Event::Burn { at } if flat(*at, side_at) < 0.1))
            .count() as i32;
        (
            90 - w.enemies[side].hp - burns * crate::tuning::BURN_DMG,
            w.enemies[2].hp,
            w.player.skill.brace_soaked,
        )
    };
    assert_eq!(burst(0), (10, 90, 0), "the far one is out of reach");
    assert_eq!(burst(1), (20, 90, 1));
    assert_eq!(burst(3), (40, 90, 3));
    assert_eq!(burst(4), (40, 90, 3), "three at most");
    // 버티지 않으면 같은 타가 맞히고 경직시킨다.
    let (mut w, t) = world();
    w.spawn_mob(vec3(0.0, 0.0, -1.2));
    still(&mut w);
    swing_at_player(&mut w, 0);
    let ev = run(&mut w, &t, Actions::default(), 2);
    assert!(ev.contains(&Event::PlayerHurt) && w.player.hp < PLAYER_HP);
}

/// 버팀: 버티는 동안 회피는 터뜨림 없이 버팀을 끊는다 — 대기는 쓴 채. 시동 동안·후딜의 회피 받기는 다른 스킬과 같다.
#[test]
fn a_dodge_breaks_the_brace_without_a_burst_and_the_cooldown_stays_spent() {
    let (mut w, t) = world();
    wear(&mut w, Sk::Brace, 1, Pick::A);
    w.spawn_mob(vec3(1.5, 0.0, 0.0));
    still(&mut w);
    let mut ev = run(&mut w, &t, skill(0), ticks(0.4));
    assert!(matches!(w.player.state, PState::Skill { .. }));
    ev.extend(w.step(
        &Actions {
            dodge: true,
            ..Default::default()
        },
        &t,
    ));
    assert!(
        matches!(w.player.state, PState::Dodge { .. }),
        "{:?}",
        w.player.state
    );
    ev.extend(run(&mut w, &t, Actions::default(), 120));
    assert!(!ev
        .iter()
        .any(|e| matches!(e, Event::Skill { id: Sk::Brace, .. })));
    assert_eq!(w.enemies[0].hp, 90, "no burst");
    assert!(w.player.skill.cd[Sk::Brace.index()] > 5.0, "spent");
}

/// 버팀 3단 ㄱ "버티는 걸음": 버티는 동안 스틱 쪽으로 느리게 걷는다 — 고르기가 없거나 ㄴ이면 제자리.
#[test]
fn brace_pick_a_walks_while_bracing() {
    let moved = |rank: u8, pick: Pick| {
        let (mut w, t) = world();
        wear(&mut w, Sk::Brace, rank, pick);
        let go = Actions {
            move_dir: vec2(0.0, 1.0),
            ..Default::default()
        };
        w.step(&Actions { skill1: true, ..go }, &t);
        let mut n = 1;
        while matches!(w.player.state, PState::Skill { .. }) {
            w.step(&go, &t);
            n += 1;
        }
        (flat(w.player.pos, Vec3::ZERO), n as f32 * TICK_DT, t)
    };
    let (d, secs, t) = moved(3, Pick::A);
    assert!(d > 1.0, "walked: {d}");
    assert!(d < t[P::MoveSpeed] * secs * 0.5, "slowly: {d}");
    for (rank, pick) in [(2, Pick::A), (3, Pick::B)] {
        let (d, ..) = moved(rank, pick);
        assert!(d < 0.05, "{rank} {pick:?}: {d}");
    }
}

/// 버팀 3단 ㄴ "되쏘기": 버티는 동안 받아낸 종 조각을 터뜨릴 때 던진 종지기 쪽으로 되쏜다 — 되돌린 조각이 종지기를 깎는다.
/// ㄱ이면 받아낼 뿐(종지기는 그대로).
#[test]
fn brace_pick_b_sends_soaked_shards_back_at_their_thrower() {
    let hurt_keeper = |pick: Pick| {
        let (mut w, t) = world();
        wear(&mut w, Sk::Brace, 3, pick);
        let b = w.spawn(Kind::Bellkeeper, vec3(0.0, 0.0, -8.0));
        still(&mut w);
        let hp = w.enemies[b].hp;
        run(&mut w, &t, skill(0), ticks(0.15));
        w.shards.push(Shard {
            pos: vec3(0.0, 1.1, -1.6),
            dir: vec3(0.0, 0.0, 1.0),
            went: 0.0,
            reflected: false,
            from: b,
        });
        let mut soaked = false;
        for _ in 0..200 {
            soaked |= w
                .step(&Actions::default(), &t)
                .iter()
                .any(|e| matches!(e, Event::Soak { .. }));
            w.enemies[b].state = EState::Recover { t: -1000.0 };
        }
        assert!(soaked, "{pick:?}: the shard was soaked");
        hp - w.enemies[b].hp
    };
    assert_eq!(hurt_keeper(Pick::B), crate::tuning::SHARD_REFLECT_DAMAGE);
    assert_eq!(hurt_keeper(Pick::A), 0);
}

/// 버팀 5단 맺음: 터뜨림에 맞은 적이 1초 휘청인다 — 보스는 휘청이지 않는다. 4단에는 없다.
#[test]
fn brace_capstone_staggers_the_burst_but_not_a_boss() {
    let staggers = |rank: u8| {
        let (mut w, t) = world();
        wear(&mut w, Sk::Brace, rank, Pick::A);
        w.spawn_mob(vec3(2.0, 0.0, 0.0));
        w.spawn_boss(vec3(-2.2, 0.0, 0.0));
        still(&mut w);
        let mut most = [0.0f32; 2];
        w.step(&skill(0), &t);
        for _ in 0..150 {
            w.step(&Actions::default(), &t);
            for (i, e) in w.enemies.iter().enumerate() {
                if let EState::Stagger { dur, .. } = e.state {
                    most[i] = most[i].max(dur);
                }
            }
        }
        most
    };
    let five = staggers(MAX_RANK);
    assert!((five[0] - 1.0).abs() < 1e-5, "{five:?}");
    assert_eq!(five[1], 0.0, "a boss does not stagger");
    assert_eq!(staggers(4)[0], 0.0);
}

/// 앞에서 이 틱에 내려치는 잔재(쪽 −z)와 둘레의 적들 — 되울림의 판.
fn echo_world(rank: u8, pick: Pick) -> (World, Tuning) {
    let (mut w, t) = world();
    wear(&mut w, Sk::Echo, rank, pick);
    w.spawn_mob(vec3(0.0, 0.0, -1.2)); // 0: 내려친다(패링당한다)
    w.spawn_mob(vec3(3.0, 0.0, 0.5)); // 1: 둘레 4m 안
    w.spawn_mob(vec3(0.0, 0.0, 7.0)); // 2: 밖
    w.spawn_boss(vec3(-2.6, 0.0, 1.0)); // 3: 안의 보스
    still(&mut w);
    (w, t)
}

/// 앞의 잔재가 내려치게 하고 패링한다 — 이 판의 사건과 패링 창이 열려 있던 틱 수.
fn parry_the_front(w: &mut World, t: &Tuning) -> (Vec<Event>, usize) {
    let en = &mut w.enemies[0];
    en.facing = 0.0;
    en.atk = Atk::Smash;
    en.state = EState::Tele { t: en.tele_dur() };
    w.player.facing = PI;
    let mut ev = w.step(
        &Actions {
            parry: true,
            ..Default::default()
        },
        t,
    );
    let mut open = w.parry_open(t) as usize;
    for _ in 0..40 {
        ev.extend(w.step(&Actions::default(), t));
        open += w.parry_open(t) as usize;
    }
    (ev, open)
}

/// 되울림: 4초 동안 성공한 패링마다 둘레 4m의 모든 적이 0.8초 휘청인다 — 밖의 적·보스는 아니다, 피해는 없다.
/// **패링 창은 그대로**(0095 원칙 2): 울림이 있어도 없어도 창이 열린 틱 수가 같다. 울림이 없으면 둘레는 그대로.
#[test]
fn echo_makes_each_parry_stagger_everyone_around_and_leaves_the_window_alone() {
    let (mut w, t) = echo_world(1, Pick::A);
    run(&mut w, &t, skill(0), ticks(0.3));
    assert!(
        (w.player.skill.echo_left - 4.0).abs() < 0.1,
        "{}",
        w.player.skill.echo_left
    );
    still(&mut w);
    let (ev, open_echo) = parry_the_front(&mut w, &t);
    assert!(ev.iter().any(|e| matches!(e, Event::Parry { .. })));
    assert!(ev.iter().any(|e| matches!(e, Event::EchoRing { .. })));
    assert!(
        matches!(w.enemies[1].state, EState::Stagger { dur, .. } if (dur - 0.8).abs() < 1e-5),
        "{:?}",
        w.enemies[1].state
    );
    assert!(
        !matches!(w.enemies[2].state, EState::Stagger { .. }),
        "outside"
    );
    assert!(
        !matches!(w.enemies[3].state, EState::Stagger { .. }),
        "a boss"
    );
    // 울림 자체의 피해는 없다 — 울림 타입(0223 V2)이라 닿은 잡몹이 한 박 뒤에 울리고, 그 울림이 둘레를 조금 다치게 한다.
    let tolls = ev
        .iter()
        .filter(|e| matches!(e, Event::Toll { .. }))
        .count() as i32;
    assert!(tolls >= 1);
    assert!(w
        .enemies
        .iter()
        .skip(1)
        .all(|e| e.max_hp - e.hp <= tolls * crate::tuning::TOLL_DMG));
    // 울림 없이 같은 패링.
    let (mut w, t) = echo_world(1, Pick::A);
    let (ev, open_plain) = parry_the_front(&mut w, &t);
    assert!(ev.iter().any(|e| matches!(e, Event::Parry { .. })));
    assert!(!ev.iter().any(|e| matches!(e, Event::EchoRing { .. })));
    assert!(!matches!(w.enemies[1].state, EState::Stagger { .. }));
    assert_eq!(open_echo, open_plain, "the parry window is untouched");
    // 2단: 5초. 4초가 지나면 끝(1단).
    let (mut w, t) = echo_world(2, Pick::A);
    run(&mut w, &t, skill(0), ticks(0.3));
    assert!((w.player.skill.echo_total - 5.0).abs() < 1e-5);
    let (mut w, t) = echo_world(1, Pick::A);
    run(&mut w, &t, skill(0), ticks(0.3) + ticks(4.1));
    assert_eq!(w.player.skill.echo_left, 0.0);
    still(&mut w);
    let (ev, _) = parry_the_front(&mut w, &t);
    assert!(
        !ev.iter().any(|e| matches!(e, Event::EchoRing { .. })),
        "over"
    );
}

/// 되울림 3단 ㄱ "울리는 틈": 울림이 닿은 적의 자세가 깎인다 · ㄴ "당기는 박자": 울린 패링마다 모든 스킬의 대기 −0.6초.
#[test]
fn echo_picks_add_poise_damage_or_pull_cooldowns() {
    let echo = |pick: Pick| {
        let (mut w, t) = echo_world(3, pick);
        run(&mut w, &t, skill(0), ticks(0.3));
        w.player.skill.cd[Sk::Whirl.index()] = 5.0;
        still(&mut w);
        let cd0 = w.player.skill.cd[Sk::Whirl.index()];
        let (_, _) = parry_the_front(&mut w, &t);
        let spent = cd0 - w.player.skill.cd[Sk::Whirl.index()];
        (w.enemies[1].stance.poise, w.enemies[3].stance.poise, spent)
    };
    let (poise_a, boss_a, spent_a) = echo(Pick::A);
    let (poise_b, _, spent_b) = echo(Pick::B);
    assert!(poise_a > 0.0 && boss_a > 0.0, "{poise_a} {boss_a}");
    assert_eq!(poise_b, 0.0);
    assert!(
        (spent_b - spent_a - 0.6).abs() < 0.02,
        "{spent_a} vs {spent_b}"
    );
}

/// 되울림 5단 맺음: 울림이 닿은 적에게 피해 10 — 보스도 다치되 휘청이지 않는다. 4단에는 없다.
#[test]
fn echo_capstone_hurts_everyone_echoed_but_staggers_no_boss() {
    let hurt = |rank: u8| {
        let (mut w, t) = echo_world(rank, Pick::B);
        run(&mut w, &t, skill(0), ticks(0.3));
        still(&mut w);
        let boss = w.enemies[3].hp;
        parry_the_front(&mut w, &t);
        assert!(!matches!(w.enemies[3].state, EState::Stagger { .. }));
        (90 - w.enemies[1].hp, boss - w.enemies[3].hp)
    };
    // 맺음 없이도 울림 타입(0223 V2)의 울림이 조금 다치게 한다 — 맺음의 몫은 그 위의 10.
    let (plain, capped) = (hurt(4), hurt(MAX_RANK));
    assert_eq!((capped.0 - plain.0, capped.1 - plain.1), (10, 10));
    assert!(plain.0 > 0 && plain.0 % crate::tuning::TOLL_DMG == 0);
}

/// 던지고 다 돌아올 때까지(또는 `n`틱).
fn throw_all(w: &mut World, t: &Tuning, a: Actions) -> Vec<Event> {
    let mut ev = w.step(&a, t);
    for _ in 0..400 {
        ev.extend(w.step(&Actions::default(), t));
        if matches!(w.player.state, PState::Idle) && w.skill_blades.is_empty() {
            break;
        }
    }
    ev
}

/// 던지기: 든 것이 8m를 곧게 날아가 뚫고 돌아온다 — 길 위의 적은 가는 길·오는 길에 10씩, 길 밖은 그대로. 후딜 뒤에는 무기가
/// 날고 있어도 움직이고 평타를 낼 수 있다(무기는 손에 없는 것으로 그린다). 돌아오면 손에.
#[test]
fn throw_flies_out_through_the_lane_and_back_cutting_both_ways() {
    let (mut w, t) = world();
    wear(&mut w, Sk::Throw, 1, Pick::A);
    for z in [-3.0, -6.5] {
        w.spawn_mob(vec3(0.0, 0.0, z));
    }
    w.spawn_mob(vec3(3.0, 0.0, -4.0));
    w.spawn_mob(vec3(0.0, 0.0, -10.5));
    still(&mut w);
    let d = Sk::Throw.def();
    let mut ev = run(&mut w, &t, skill(0), ticks(d.startup + 0.4 + 0.05));
    assert!(
        matches!(w.player.state, PState::Idle),
        "free after the recover"
    );
    assert!(w.weapon_away(), "still flying");
    // 무기가 없는 동안에도 평타.
    ev.extend(w.step(
        &Actions {
            attack: true,
            ..Default::default()
        },
        &t,
    ));
    assert!(matches!(w.player.state, PState::Attack { .. }));
    ev.extend(throw_all(&mut w, &t, Actions::default()));
    assert!(!w.weapon_away(), "back in hand");
    assert!(ev.iter().any(|e| matches!(e, Event::ThrowTurn { .. })));
    assert!(ev.iter().any(|e| matches!(e, Event::ThrowBack { .. })));
    assert_eq!(w.enemies[0].hp, 90 - 20, "out and back");
    assert_eq!(w.enemies[1].hp, 90 - 20);
    assert_eq!(w.enemies[2].hp, 90, "off the lane");
    assert_eq!(w.enemies[3].hp, 90, "beyond 8 m");
    assert_eq!(w.player.skill.gauge, 0.0, "a skill hit: no gauge");
}

/// 던지기 3단 ㄱ "박힌 자리": 돌아오지 않고 끝에서 터진다(둘레 2.5m · 피해 20) — 길 위의 적은 한 번만.
#[test]
fn throw_pick_a_sticks_at_the_far_end_and_bursts() {
    let (mut w, t) = world();
    wear(&mut w, Sk::Throw, 3, Pick::A);
    w.spawn_mob(vec3(0.0, 0.0, -3.0));
    w.spawn_mob(vec3(1.8, 0.0, -8.0)); // 끝 곁
    still(&mut w);
    let ev = throw_all(&mut w, &t, skill(0));
    assert!(ev.iter().any(|e| matches!(e, Event::ThrowBoom { .. })));
    assert!(!ev.iter().any(|e| matches!(e, Event::ThrowBack { .. })));
    assert_eq!(w.enemies[0].hp, 90 - 12, "once on the way out (rank 3)");
    assert_eq!(w.enemies[1].hp, 90 - 20, "the burst");
    assert!(!w.weapon_away());
}

/// 던지기 3단 ㄴ "끌고 오기": 돌아오는 길에 맞은 적이 방랑자 쪽으로 2m 끌려온다 — 보스는 아니다. 2단(고르기 없음)이면 그대로.
#[test]
fn throw_pick_b_drags_the_hit_back_two_metres_but_never_a_boss() {
    let pulled = |rank: u8, pick: Pick| {
        let (mut w, t) = world();
        wear(&mut w, Sk::Throw, rank, pick);
        // 보스는 끌려오는 잡몹의 길 밖·던진 길 위(뒤쪽)에.
        w.spawn_mob(vec3(0.0, 0.0, -4.5));
        w.spawn_boss(vec3(0.0, 0.0, -7.6));
        still(&mut w);
        let was: Vec<Vec3> = w.enemies.iter().map(|e| e.pos).collect();
        throw_all(&mut w, &t, skill(0));
        run(&mut w, &t, Actions::default(), 40);
        (
            flat(was[0], Vec3::ZERO) - flat(w.enemies[0].pos, Vec3::ZERO),
            flat(was[1], Vec3::ZERO) - flat(w.enemies[1].pos, Vec3::ZERO),
        )
    };
    let (mob, boss) = pulled(3, Pick::B);
    assert!((mob - 2.0).abs() < 0.5, "dragged ~2 m: {mob}");
    assert!(boss < 0.2, "a boss is not dragged: {boss}");
    let (mob, _) = pulled(2, Pick::A);
    assert!(mob < 0.5, "no pick: {mob}");
}

/// 던지기 5단 맺음: 돌아오는 길의 피해 ×1.5(13 → 20)·자세 피해 두 배. 4단은 오는 길도 같은 피해.
#[test]
fn throw_capstone_makes_the_way_back_hit_harder() {
    let legs = |rank: u8| {
        let (mut w, t) = world();
        wear(&mut w, Sk::Throw, rank, Pick::B);
        w.spawn_mob(vec3(0.0, 0.0, -4.0));
        still(&mut w);
        for e in &mut w.enemies {
            e.hp = 500;
            e.max_hp = 500;
        }
        let mut hits = Vec::new();
        let mut poise = Vec::new();
        let mut hp = 500;
        let mut last_poise = 0.0;
        let mut ev = w.step(&skill(0), &t);
        for _ in 0..300 {
            ev.extend(w.step(&Actions::default(), &t));
            let e = &w.enemies[0];
            if e.hp != hp {
                hits.push(hp - e.hp);
                poise.push(e.stance.poise - last_poise);
                hp = e.hp;
            }
            last_poise = e.stance.poise;
        }
        (hits, poise)
    };
    let (hits, poise) = legs(MAX_RANK);
    assert_eq!(hits, [13, 20]);
    assert!((poise[1] - 2.0 * poise[0]).abs() < 1e-3, "{poise:?}");
    assert_eq!(legs(4).0, [12, 12]);
}

/// 잔불 흩기: 선 자리 둘레 4m에 3초 — 그 안의 적이 1초마다 10(첫 타는 1초 뒤), 밖의 적은 그대로. 잔불(재화)은 쓰지 않는다.
/// 방랑자가 떠나도 불티는 그 자리에 남는다.
#[test]
fn embers_burn_everyone_inside_every_second_for_three_seconds_for_free() {
    let (mut w, t) = world();
    wear(&mut w, Sk::Embers, 1, Pick::A);
    w.spawn_mob(vec3(2.5, 0.0, 0.0));
    w.spawn_mob(vec3(0.0, 0.0, 6.5));
    still(&mut w);
    w.embers = 50;
    let d = Sk::Embers.def();
    run(&mut w, &t, skill(0), ticks(d.startup + 0.9));
    assert_eq!(w.enemies[0].hp, 90, "not yet");
    assert_eq!(w.skill_active(0) > 0.0, true, "the HUD cell shows it");
    // 떠난다.
    let away = Actions {
        move_dir: vec2(1.0, 0.0),
        ..Default::default()
    };
    let mut pulses = 0;
    for _ in 0..ticks(3.0) {
        pulses += w
            .step(&away, &t)
            .iter()
            .filter(|e| matches!(e, Event::FieldPulse { .. }))
            .count();
    }
    assert_eq!(pulses, 3);
    // 불티는 잔불 타입(0223 V2): 둘째 타에 화상 한 겹이 붙어 셋째 타와 같은 때 한 번 탄다(겹마다 3).
    assert_eq!(w.enemies[0].ail.burn, 1);
    assert_eq!(w.enemies[0].hp, 90 - 30 - crate::tuning::BURN_DMG);
    assert_eq!(w.enemies[1].hp, 90, "outside");
    assert!(w.skill_fields.is_empty() && w.skill_active(0) == 0.0);
    assert_eq!(w.embers, 50, "costs no embers");
    assert_eq!(w.player.skill.gauge, 0.0);
}

/// 잔불 흩기 3단 ㄱ "따르는 불티": 불티가 방랑자를 따라와 옮겨 간 자리의 적을 태운다 — ㄴ이면 처음 자리에 남는다.
#[test]
fn embers_pick_a_follows_the_wanderer() {
    let burnt = |pick: Pick| {
        let (mut w, t) = world();
        wear(&mut w, Sk::Embers, 3, pick);
        w.spawn_mob(vec3(0.0, 0.0, -9.0));
        still(&mut w);
        let d = Sk::Embers.def();
        run(&mut w, &t, skill(0), ticks(d.startup + d.recover + 0.05));
        assert_eq!(w.skill_fields.len(), 1);
        // 방랑자가 적 곁으로 옮겨 간다(걸음과 상관없이 — 따라오는지만 본다).
        w.player.pos = vec3(0.0, 0.0, -7.0);
        for _ in 0..ticks(3.0) {
            w.step(&Actions::default(), &t);
            w.enemies[0].state = EState::Recover { t: -1000.0 };
        }
        90 - w.enemies[0].hp
    };
    // 세 타 + 그 사이에 붙은 화상 한 겹이 한 번 탄 몫(0223 V2).
    assert_eq!(
        burnt(Pick::A),
        36 + crate::tuning::BURN_DMG,
        "three pulses of 12 and one burn"
    );
    assert_eq!(burnt(Pick::B), 0);
}

/// 잔불 흩기 3단 ㄴ "남는 불씨": 불티 속에서 쓰러진 적이 잔불을 하나 더 남긴다 — ㄱ이면 여느 때와 같다.
#[test]
fn embers_pick_b_makes_the_fallen_drop_one_more_ember() {
    let dropped = |pick: Pick| {
        let (mut w, t) = world();
        wear(&mut w, Sk::Embers, 3, pick);
        w.spawn_mob(vec3(2.0, 0.0, 0.0));
        still(&mut w);
        w.enemies[0].hp = 5;
        for _ in 0..ticks(1.5) {
            w.step(&skill(0), &t);
            if !w.drops.is_empty() {
                break;
            }
        }
        assert!(!w.enemies[0].alive());
        w.drops.iter().map(|d| d.value).sum::<u32>()
    };
    assert_eq!(dropped(Pick::B), dropped(Pick::A) + 1);
}

/// 잔불 흩기 5단 맺음: 1초 더(넷째 타) + 불티가 자세도 깎는다. 4단은 세 타·자세 그대로.
#[test]
fn embers_capstone_lasts_a_second_longer_and_wears_down_poise() {
    let burn = |rank: u8| {
        let (mut w, t) = world();
        wear(&mut w, Sk::Embers, rank, Pick::B);
        w.spawn_mob(vec3(2.0, 0.0, 0.0));
        still(&mut w);
        w.enemies[0].hp = 500;
        let ev = run(&mut w, &t, skill(0), ticks(5.5));
        let pulses = ev
            .iter()
            .filter(|e| matches!(e, Event::FieldPulse { .. }))
            .count();
        (
            pulses,
            w.enemies[0].stance.poise > 0.0 || w.enemies[0].stance.broken > 0.0,
        )
    };
    assert_eq!(burn(MAX_RANK), (4, true));
    assert_eq!(burn(4), (3, false));
}

/// 기세(0151)는 새 스킬들의 쓰러뜨림에도: 던진 무기·불티로 쓰러뜨리면 다른 스킬의 대기가 "스킬 몫"만큼 당겨진다.
#[test]
fn kills_by_the_new_skills_pull_cooldowns_as_skill_kills() {
    for id in [Sk::Throw, Sk::Embers] {
        let (mut w, t) = world();
        wear(&mut w, id, 1, Pick::A);
        w.spawn_mob(vec3(0.0, 0.0, -2.5));
        still(&mut w);
        w.enemies[0].hp = 5;
        w.player.skill.cd[Sk::Quake.index()] = 5.0;
        let mut ev = Vec::new();
        let mut n = 0;
        ev.extend(w.step(&skill(0), &t));
        while !ev.iter().any(|e| matches!(e, Event::EnemyDown { .. })) && n < 200 {
            ev.extend(w.step(&Actions::default(), &t));
            n += 1;
        }
        let cd = w.player.skill.cd[Sk::Quake.index()];
        let want = 5.0 - (n + 1) as f32 * TICK_DT - t[P::SkillMomentumOwn];
        assert!((cd - want).abs() < 0.02, "{id:?}: {cd} vs {want}");
    }
}

/// 큰 기술 몰아치기: 시동 0.3초 뒤 5초 동안 평타가 기력을 쓰지 않고 1.4배 빠르며 모든 타가 박자 타(`Event::Beat`)다.
/// 끝나면 여느 평타(기력을 쓴다). 끼운 큰 기술이 나간다(쟁반 뒤집기가 아니다).
#[test]
fn flurry_makes_basic_attacks_free_fast_and_on_the_beat_for_five_seconds() {
    let swing_ticks = |w: &mut World, t: &Tuning| {
        w.step(
            &Actions {
                attack: true,
                ..Default::default()
            },
            t,
        );
        let mut n = 1;
        while matches!(w.player.state, PState::Attack { .. }) {
            w.step(&Actions::default(), t);
            n += 1;
        }
        n
    };
    let (mut w, t) = world();
    let plain = swing_ticks(&mut w, &t);
    let (mut w, t) = world();
    wear_ult(&mut w, Ult::Flurry, 1);
    w.player.skill.gauge = 1.0;
    let ev = run(
        &mut w,
        &t,
        Actions {
            ult: true,
            ..Default::default()
        },
        ticks(0.35),
    );
    assert!(ev.iter().any(|e| matches!(e, Event::Flurry { .. })));
    assert!(
        !ev.iter().any(|e| matches!(e, Event::Ult { .. })),
        "not Upend"
    );
    assert!((w.player.skill.flurry_left - 5.0).abs() < 0.1);
    assert_eq!(w.player.skill.gauge, 0.0);
    let stamina = w.player.stamina;
    let mut beats = 0;
    let fast = swing_ticks(&mut w, &t);
    for _ in 0..3 {
        let ev = w.step(
            &Actions {
                attack: true,
                ..Default::default()
            },
            &t,
        );
        beats += ev.contains(&Event::Beat) as usize;
        run(&mut w, &t, Actions::default(), 30);
    }
    assert_eq!(beats, 3, "every swing is a beat hit");
    assert_eq!(w.player.stamina, stamina, "no stamina");
    let k = plain as f32 / fast as f32;
    assert!((k - 1.4).abs() < 0.15, "1.4x faster: {plain} vs {fast}");
    assert!(w.ult_active() > 0.0);
    // 끝나면 여느 평타.
    run(&mut w, &t, Actions::default(), ticks(5.0));
    assert_eq!((w.player.skill.flurry_left, w.ult_active()), (0.0, 0.0));
    let s0 = w.player.stamina;
    swing_ticks(&mut w, &t);
    assert!(w.player.stamina < s0);
}

/// 몰아치기의 단: 2단 6.5초, 3단은 그동안 쓰러뜨릴 때마다 +0.5초 — 더해지는 몫은 3초까지.
#[test]
fn flurry_ranks_lengthen_it_and_rank_three_feeds_on_kills_up_to_three_seconds() {
    let (mut w, t) = world();
    wear_ult(&mut w, Ult::Flurry, 2);
    w.player.skill.gauge = 1.0;
    let ult = Actions {
        ult: true,
        ..Default::default()
    };
    run(&mut w, &t, ult, ticks(0.32));
    assert!((w.player.skill.flurry_total - 6.5).abs() < 1e-4);
    let down = Event::EnemyDown { at: Vec3::ZERO };
    let left = w.player.skill.flurry_left;
    w.skill_books(&t, &mut vec![down]);
    assert_eq!(w.player.skill.flurry_left, left, "rank 2 does not feed");
    let (mut w, t) = world();
    wear_ult(&mut w, Ult::Flurry, 3);
    w.player.skill.gauge = 1.0;
    run(&mut w, &t, ult, ticks(0.32));
    let left = w.player.skill.flurry_left;
    w.skill_books(&t, &mut vec![down, down]);
    assert!((w.player.skill.flurry_left - left - 1.0).abs() < 1e-4);
    for _ in 0..10 {
        w.skill_books(&t, &mut vec![down]);
    }
    assert!(
        (w.player.skill.flurry_left - left - 3.0).abs() < 1e-4,
        "capped"
    );
    assert!((w.player.skill.flurry_added - 3.0).abs() < 1e-4);
}

/// 큰 기술 한 줄기: 시동에 세계가 멈추고(0.3초) 앞 12m 한 줄(반 폭 1.2m)을 가른다 — 줄 위의 적은 40·자세 크게, 보스도 다치고
/// 자세가 깎이되 휘청이지 않는다(막대가 안 깨지면). 줄 밖·12m 너머는 그대로. 2단 15m, 3단 60.
#[test]
fn line_cuts_one_long_lane_ahead_hurting_bosses_too() {
    let cut = |rank: u8| {
        let mut w = World::new("test", 30.0);
        let t = Tuning::default();
        w.player.facing = PI;
        wear_ult(&mut w, Ult::Line, rank);
        for z in [-3.0, -10.5, -13.5] {
            w.spawn_mob(vec3(0.0, 0.0, z));
        }
        w.spawn_mob(vec3(3.0, 0.0, -5.0));
        w.spawn_boss(vec3(0.5, 0.0, -6.5));
        still(&mut w);
        let boss = w.enemies[4].hp;
        w.player.skill.gauge = 1.0;
        w.step(
            &Actions {
                ult: true,
                ..Default::default()
            },
            &t,
        );
        assert!(w.hitstop >= LINE.freeze - 1e-4, "the world stops");
        let mut ev = Vec::new();
        let mut staggered = false;
        for _ in 0..120 {
            ev.extend(w.step(&Actions::default(), &t));
            staggered |= matches!(w.enemies[4].state, EState::Stagger { .. });
        }
        assert!(ev.iter().any(|e| matches!(e, Event::LineCut { .. })));
        assert!(!staggered, "a boss is not staggered");
        assert!(w.enemies[4].stance.poise > 0.0, "but its poise is worn");
        assert_eq!(w.player.skill.gauge, 0.0);
        let lost: Vec<i32> = w.enemies[..4].iter().map(|e| 90 - e.hp).collect();
        (lost, boss - w.enemies[4].hp)
    };
    assert_eq!(cut(1), (vec![40, 40, 0, 0], 40));
    assert_eq!(cut(2), (vec![40, 40, 40, 0], 40), "15 m");
    assert_eq!(cut(3), (vec![60, 60, 60, 0], 60));
}

/// 끼우는 큰 기술은 하나: 게이지가 차면 끼운 것이 나간다 — 쟁반 뒤집기를 끼웠으면 몰아치기·한 줄기는 나가지 않는다.
#[test]
fn only_the_worn_great_skill_goes_off() {
    for (great, want) in [(Ult::Upend, 0), (Ult::Flurry, 1), (Ult::Line, 2)] {
        let (mut w, t) = world();
        wear_ult(&mut w, great, 1);
        w.spawn_mob(vec3(0.0, 0.0, -3.0));
        still(&mut w);
        w.player.skill.gauge = 1.0;
        let ev = run(
            &mut w,
            &t,
            Actions {
                ult: true,
                ..Default::default()
            },
            90,
        );
        let got = [
            ev.iter().any(|e| matches!(e, Event::Ult { .. })),
            ev.iter().any(|e| matches!(e, Event::Flurry { .. })),
            ev.iter().any(|e| matches!(e, Event::LineCut { .. })),
        ];
        for (i, g) in got.into_iter().enumerate() {
            assert_eq!(g, i == want, "{great:?}");
        }
    }
}

// ---- 0214 넷째 줄: 제비걸음·막아치기·뿌리 솟기 ----

/// 스킬이 다 끝날 때까지 돈다(적은 멈춘 채).
fn finish(w: &mut World, t: &Tuning, slot: usize) -> Vec<Event> {
    let mut ev = w.step(&skill(slot), t);
    for _ in 0..120 {
        still_keep(w);
        ev.extend(w.step(&Actions::default(), t));
        if !matches!(w.player.state, PState::Skill { .. }) {
            break;
        }
    }
    ev
}

/// 체력은 두고 멈춰 세운다.
fn still_keep(w: &mut World) {
    for e in &mut w.enemies {
        if e.active() && !matches!(e.state, EState::Stagger { .. }) {
            e.state = EState::Recover { t: -1000.0 };
        }
    }
}

/// 제비걸음: 6 m 안의 적 셋에게 차례로 뛰어들어 한 번씩 벤다 — 방랑자는 마지막 적 곁에 선다. 돌아오는 걸음(ㄴ)이면 처음 자리로.
#[test]
fn swallow_step_darts_to_each_foe_in_turn() {
    let (mut w, t) = world();
    wear(&mut w, Sk::Swallow, 1, Pick::A);
    for p in [
        vec3(0.0, 0.0, -3.0),
        vec3(3.0, 0.0, -4.0),
        vec3(-3.0, 0.0, -5.0),
    ] {
        w.spawn_mob(p);
    }
    still(&mut w);
    let p0 = w.player.pos;
    finish(&mut w, &t, 0);
    let hit: Vec<bool> = w.enemies.iter().map(|e| e.hp < 90).collect();
    assert_eq!(hit, [true, true, true], "every foe once");
    assert!(
        flat(w.player.pos, p0) > 2.0,
        "the wanderer moved: {:?}",
        w.player.pos
    );
    // 한 곳만(ㄱ — 3단): 첫 적만 거듭.
    let (mut w, t) = world();
    wear(&mut w, Sk::Swallow, 3, Pick::A);
    w.spawn_mob(vec3(0.0, 0.0, -3.0));
    w.spawn_mob(vec3(4.0, 0.0, -3.0));
    still(&mut w);
    finish(&mut w, &t, 0);
    let (a, b) = (90 - w.enemies[0].hp, 90 - w.enemies[1].hp);
    assert!(
        a > b * 2,
        "focus cuts the first again and again: {a} vs {b}"
    );
    // 돌아오는 걸음(ㄴ): 끝에 처음 자리로.
    let (mut w, t) = world();
    wear(&mut w, Sk::Swallow, 3, Pick::B);
    w.spawn_mob(vec3(0.0, 0.0, -4.0));
    still(&mut w);
    let p0 = w.player.pos;
    finish(&mut w, &t, 0);
    assert!(
        flat(w.player.pos, p0) < 0.6,
        "home again: {:?}",
        w.player.pos
    );
}

/// 막아치기: 공격을 준비하던 적은 그 공격이 끊기고(휘청) 자세 피해를 더 받는다 — 서 있는 적은 그냥 맞는다. 보스는 끊기지 않는다.
/// 무너뜨림(맺음)이면 끊긴 적의 자세가 곧바로 깨진다.
#[test]
fn bash_cuts_off_a_winding_attack_but_not_a_boss() {
    let run = |rank: u8, boss: bool| {
        let (mut w, t) = world();
        wear(&mut w, Sk::Bash, rank, Pick::A);
        if boss {
            w.spawn_boss(vec3(0.0, 0.0, -2.0));
        } else {
            w.spawn_mob(vec3(0.0, 0.0, -1.8));
        }
        w.spawn_mob(vec3(0.8, 0.0, -1.8));
        still(&mut w);
        let en = &mut w.enemies[0];
        en.facing = 0.0;
        en.atk = Atk::Smash;
        en.state = EState::Tele { t: 0.0 };
        let ev = w.step(&skill(0), &t);
        let mut ev = ev;
        for _ in 0..30 {
            ev.extend(w.step(&Actions::default(), &t));
        }
        let cut = ev
            .iter()
            .filter(|e| matches!(e, Event::Interrupt { .. }))
            .count();
        (w, t, cut)
    };
    let (w, t, cut) = run(1, false);
    assert_eq!(cut, 1, "only the winding foe is cut off");
    assert!(
        w.enemies[0].stance.poise > w.enemies[1].stance.poise * 2.0,
        "more posture damage"
    );
    assert!(w.enemies[1].hp < 90, "the other is simply hit");
    let _ = t;
    let (w, _, cut) = run(1, true);
    assert_eq!(cut, 0, "a boss is not cut off");
    assert!(!matches!(w.enemies[0].state, EState::Stagger { .. }));
    let (w, t, _) = run(5, false);
    assert!(
        w.enemies[0].stance.broken > 0.0
            || w.enemies[0].stance.poise >= poise_max(&w.enemies[0], &t) - 1e-3
    );
}

/// 뿌리 솟기: 앞 줄의 적은 한 번씩 찔리고 휘청인다, 옆의 적은 아니다. 묶는 뿌리(ㄴ)면 묶여 걷지 못한다. 솟은 뿌리는 그림으로 남았다 사라진다.
#[test]
fn root_eruption_runs_ahead_and_binding_roots_hold() {
    let (mut w, t) = world();
    wear(&mut w, Sk::Roots, 1, Pick::A);
    w.spawn_mob(vec3(0.0, 0.0, -2.0));
    w.spawn_mob(vec3(0.0, 0.0, -6.0));
    w.spawn_mob(vec3(5.0, 0.0, 0.0));
    still(&mut w);
    let ev = finish(&mut w, &t, 0);
    assert!(
        w.enemies[0].hp < 90 && w.enemies[1].hp < 90,
        "the line ahead"
    );
    assert_eq!(w.enemies[2].hp, 90, "not the one beside");
    let spikes = ev
        .iter()
        .filter(|e| matches!(e, Event::RootSpike { .. }))
        .count();
    assert_eq!(spikes as u8, skills::erupt_count(7.0, 1.0));
    assert!(!w.skill_spikes.is_empty());
    for _ in 0..90 {
        w.step(&Actions::default(), &t);
    }
    assert!(w.skill_spikes.is_empty(), "the roots sink back");
    // 묶는 뿌리(ㄴ — 3단): 맞은 잡몹은 묶인다.
    let (mut w, t) = world();
    wear(&mut w, Sk::Roots, 3, Pick::B);
    w.spawn_mob(vec3(0.0, 0.0, -3.0));
    still(&mut w);
    finish(&mut w, &t, 0);
    assert!(w.enemies[0].marks.bind > 0.0);
    w.step(&Actions::default(), &t);
    assert_eq!(w.enemies[0].marks.slow, 0.0, "held in place");
    // 둘레로(ㄱ): 옆의 적도 찔린다.
    let (mut w, t) = world();
    wear(&mut w, Sk::Roots, 3, Pick::A);
    w.spawn_mob(vec3(2.6, 0.0, 0.0));
    still(&mut w);
    finish(&mut w, &t, 0);
    assert!(w.enemies[0].hp < 90, "the ring reaches beside");
}

// ---- 0270 C3: 무기 판을 채운 새 스킬 ----

/// 바늘땀: 좁은 앞을 세 번 찌른다 — 앞의 적은 땀마다 맞고 날카로움이 쌓여 피가 새기 시작한다, 옆의 적은 아니다.
/// 파고드는 걸음(ㄱ)이면 앞으로 내딛고, 촘촘한 땀(ㄴ)이면 두 번 더, 꿰매기(맺음)는 마지막 땀에 휘청.
#[test]
fn needle_stitch_thrusts_a_narrow_line_three_times_and_makes_it_bleed() {
    let (mut w, t) = world();
    wear(&mut w, Sk::Stitch, 1, Pick::A);
    w.spawn_mob(vec3(0.0, 0.0, -2.0));
    w.spawn_mob(vec3(2.2, 0.0, -0.6));
    still(&mut w);
    let ev = finish(&mut w, &t, 0);
    let stabs = ev
        .iter()
        .filter(|e| matches!(e, Event::Skill { id: Sk::Stitch, .. }))
        .count();
    assert_eq!(stabs, 3);
    let dmg = 90 - w.enemies[0].hp;
    assert!(dmg >= 30, "three thrusts: {dmg}");
    assert_eq!(w.enemies[1].hp, 90, "not the one beside");
    assert!(w.enemies[0].ail.bleed > 0.0, "it bleeds");
    // 파고드는 걸음(ㄱ — 3단): 앞으로 내딛는다.
    let (mut w, t) = world();
    wear(&mut w, Sk::Stitch, 3, Pick::A);
    let p0 = w.player.pos;
    finish(&mut w, &t, 0);
    assert!(flat(w.player.pos, p0) > 1.5, "{:?}", w.player.pos);
    // 촘촘한 땀(ㄴ): 다섯 번. 꿰매기(맺음 — 5단): 마지막 땀에 휘청.
    let (mut w, t) = world();
    wear(&mut w, Sk::Stitch, 5, Pick::B);
    w.spawn_mob(vec3(0.0, 0.0, -2.0));
    still(&mut w);
    let mut ev = w.step(&skill(0), &t);
    let mut staggered = false;
    for _ in 0..60 {
        ev.extend(w.step(&Actions::default(), &t));
        staggered |= matches!(w.enemies[0].state, EState::Stagger { .. });
    }
    let stabs = ev
        .iter()
        .filter(|e| matches!(e, Event::Skill { id: Sk::Stitch, .. }))
        .count();
    assert_eq!(stabs, 5);
    assert!(staggered, "the last thrust pins");
}

/// 방패 돌진: 방패를 앞세워 내달린다 — 길 위의 적을 치고 밀어 내며 흡수를 건다(그 적을 치면 체력이 돌아온다). 벽에 박기(ㄱ)면
/// 밀려난 적이 벽에 박혀 더 다치고, 방패 뒤(ㄴ)면 내달리는 동안 맞지 않는다.
#[test]
fn shield_rush_charges_through_shoving_and_draining() {
    let (mut w, t) = world();
    wear(&mut w, Sk::Ram, 1, Pick::A);
    assert_eq!(w.grip(), crate::weapons::Grip::Shield);
    w.spawn_mob(vec3(0.3, 0.0, -2.0));
    still(&mut w);
    let p0 = w.player.pos;
    let was = w.enemies[0].pos;
    finish(&mut w, &t, 0);
    assert!(flat(w.player.pos, p0) > 3.5, "charged: {:?}", w.player.pos);
    assert!(w.enemies[0].hp < 90);
    assert!(w.enemies[0].ail.drain > 0.0, "drained");
    assert!(flat(w.enemies[0].pos, was) > 0.8, "shoved aside");
    // 벽에 박기(ㄱ — 3단): 밀려난 적이 아레나 끝 벽에 박힌다.
    let (mut w, t) = world();
    w.arena = 3.0;
    wear(&mut w, Sk::Ram, 3, Pick::A);
    w.spawn_mob(vec3(0.6, 0.0, -1.6));
    still(&mut w);
    let mut ev = finish(&mut w, &t, 0);
    for _ in 0..60 {
        ev.extend(w.step(&Actions::default(), &t));
    }
    assert!(
        ev.iter().any(|e| matches!(e, Event::Slam { .. })),
        "into the wall"
    );
    // 방패 뒤(ㄴ): 내달리는 동안 맞지 않는다.
    let (mut w, _t) = world();
    wear(&mut w, Sk::Ram, 3, Pick::B);
    let d = w.skill_eff(Sk::Ram);
    assert_eq!(d.twist, Some(Twist::Ghost));
}

/// 종 치기: 앞의 적 하나를 치고 종을 건다 — 곧 그 적이 울려 둘레의 적이 모두 맞는다(울림이 실린다). 두 번 울림(ㄱ)이면 두 번,
/// 넓은 울림(ㄴ)이면 더 멀리. 앞이 비었으면 허공에서 울린다(헛된 누름이 아니다). 보스에게도 걸리지만 휘청이지 않는다.
#[test]
fn toll_hangs_a_bell_on_one_foe_and_it_rings_around_it() {
    let (mut w, t) = world();
    wear(&mut w, Sk::Toll, 1, Pick::A);
    w.spawn_mob(vec3(0.0, 0.0, -2.0));
    w.spawn_mob(vec3(2.0, 0.0, -3.0));
    w.spawn_mob(vec3(0.0, 0.0, 6.0));
    still(&mut w);
    let mut ev = finish(&mut w, &t, 0);
    assert!(w.enemies[0].hp < 90, "the struck foe");
    assert_eq!(w.enemies[1].hp, 90, "not yet");
    for _ in 0..ticks(1.0) {
        still_keep(&mut w);
        ev.extend(w.step(&Actions::default(), &t));
    }
    let tolls = ev
        .iter()
        .filter(|e| matches!(e, Event::Toll { .. }))
        .count();
    assert!(tolls >= 1, "the bell rang");
    assert!(w.enemies[1].hp < 90, "the ring reached the one beside it");
    assert_eq!(w.enemies[2].hp, 90, "not the far one behind");
    // 두 번 울림(ㄱ — 3단).
    let (mut w, t) = world();
    wear(&mut w, Sk::Toll, 3, Pick::A);
    w.spawn_mob(vec3(0.0, 0.0, -2.0));
    still(&mut w);
    finish(&mut w, &t, 0);
    let hung = |w: &World| w.skill_bursts.iter().filter(|b| b.toll).count();
    assert_eq!(hung(&w), 2, "two rings hang on it");
    for _ in 0..ticks(2.0) {
        still_keep(&mut w);
        w.step(&Actions::default(), &t);
    }
    assert_eq!(hung(&w), 0, "both rang");
    // 넓은 울림(ㄴ): 둘레가 더 넓다.
    let reach = |pick| match w.skill_eff(Sk::Toll).skill.form {
        Form::Bell { radius, .. } => {
            let _ = pick;
            radius
        }
        _ => unreachable!(),
    };
    let a = reach(Pick::A);
    wear(&mut w, Sk::Toll, 3, Pick::B);
    let b = match w.skill_eff(Sk::Toll).skill.form {
        Form::Bell { radius, .. } => radius,
        _ => unreachable!(),
    };
    assert!(b > a * 1.4, "{a} -> {b}");
    // 앞이 비었으면 허공에서 울린다.
    let (mut w, t) = world();
    wear(&mut w, Sk::Toll, 1, Pick::A);
    finish(&mut w, &t, 0);
    assert_eq!(hung(&w), 1, "it rings in the air");
    assert!(w.skill_bursts[0].on.is_none());
}

/// 후려치기: 망치로 앞 넓은 부채꼴을 친다 — 맞은 적은 멀리 밀리고 충격으로 기절한다(보통 적은 한 번에). 뒤의 적은 아니다.
/// 크게 휘두르기(ㄴ)면 부채꼴이 넓어진다.
#[test]
fn swat_knocks_the_front_far_back_and_stuns() {
    let (mut w, t) = world();
    wear(&mut w, Sk::Swat, 1, Pick::A);
    assert_eq!(w.grip(), crate::weapons::Grip::Heavy);
    w.spawn_mob(vec3(0.0, 0.0, -2.0));
    w.spawn_mob(vec3(0.0, 0.0, 2.0));
    still(&mut w);
    let was = w.enemies[0].pos;
    let mut stunned = false;
    let mut ev = w.step(&skill(0), &t);
    for _ in 0..60 {
        ev.extend(w.step(&Actions::default(), &t));
        stunned |= w.enemies[0].ail.stun > 0.0;
    }
    assert!(w.enemies[0].hp < 90);
    assert!(stunned, "the shock stuns a plain foe in one swing");
    assert!(
        flat(w.enemies[0].pos, was) > 1.5,
        "knocked back: {:?}",
        w.enemies[0].pos
    );
    assert_eq!(w.enemies[1].hp, 90, "not the one behind");
    let arc = |w: &World| match w.skill_eff(Sk::Swat).skill.form {
        Form::Cone { arc_deg, .. } => arc_deg,
        _ => unreachable!(),
    };
    let a = arc(&w);
    wear(&mut w, Sk::Swat, 3, Pick::B);
    assert!(arc(&w) > a + 60.0);
}

/// 파문: 물결이 몸에서 둘레로 번진다 — 가까운 적이 먼저, 먼 적이 나중에(같은 물결에 한 번씩), 둘레 밖은 아니다.
/// 밀어내는 물결(ㄱ)은 바깥으로 밀고, 갈라진 땅(ㄴ)은 둘레를 남겨 늦춘다.
#[test]
fn ripple_spreads_outward_hitting_near_then_far_once_each() {
    let (mut w, t) = world();
    wear(&mut w, Sk::Ripple, 1, Pick::A);
    w.spawn_mob(vec3(0.0, 0.0, -1.5));
    w.spawn_mob(vec3(5.0, 0.0, 0.0));
    w.spawn_mob(vec3(0.0, 0.0, 9.0));
    still(&mut w);
    let mut first: [Option<usize>; 3] = [None; 3];
    w.step(&skill(0), &t);
    for k in 0..90 {
        still_keep(&mut w);
        w.step(&Actions::default(), &t);
        for (i, f) in first.iter_mut().enumerate() {
            if f.is_none() && w.enemies[i].hp < 90 {
                *f = Some(k);
            }
        }
    }
    let (near, far) = (first[0].unwrap(), first[1].unwrap());
    assert!(near < far, "near first: {near} {far}");
    assert_eq!(first[2], None, "beyond the ripple");
    assert_eq!(
        90 - w.enemies[0].hp,
        90 - w.enemies[1].hp,
        "once each, the same"
    );
    // 밀어내는 물결(ㄱ — 3단).
    let (mut w, t) = world();
    wear(&mut w, Sk::Ripple, 3, Pick::A);
    w.spawn_mob(vec3(0.0, 0.0, -2.0));
    still(&mut w);
    finish(&mut w, &t, 0);
    for _ in 0..20 {
        w.step(&Actions::default(), &t);
    }
    assert!(flat(w.enemies[0].pos, w.player.pos) > 3.0, "pushed out");
    // 갈라진 땅(ㄴ): 둘레가 남아 늦춘다.
    let (mut w, t) = world();
    wear(&mut w, Sk::Ripple, 3, Pick::B);
    w.spawn_mob(vec3(3.0, 0.0, 3.0));
    still(&mut w);
    finish(&mut w, &t, 0);
    w.step(&Actions::default(), &t);
    assert!(w.enemies[0].marks.slow < 1.0, "slowed behind the ripple");
}
