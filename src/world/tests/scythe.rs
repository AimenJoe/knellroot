//! 낫(0274 ⑤ — 드루이드): 한 손의 휜 날로 걸어 베는 3연타(셋째와 강공격은 맞은 적을 몸 앞으로 끌어온다 — 문지기는 아니다) · 주특성
//! 되살아나는 숨(묶인 적을 쳐도 흡수처럼 되돌아오고 되돌아오는 몫이 크다) · 보조특성 거두는 손(쓰러뜨릴 때마다 체력) · 빈 손의 결
//! "땅을 짚는 손"(뿌리 스킬 · 가시 씨앗의 범위) · RB 가시 씨앗(겨눈 자리에 떨어져 가시 뿌리가 솟는다 — 뿌리).
use super::*;
use crate::status::Kind;
use crate::weapons::{
    Id as W, Look, ART_SAP, BARE_GROUND_REACH, PULL_STOP, SCYTHE_HEAVY_PULL, SCYTHE_PULL,
    SUB_KILL_HEAL,
};

fn wait(w: &mut World, t: &Tuning, secs: f32) -> Vec<Event> {
    let mut ev = Vec::new();
    for _ in 0..(secs / TICK_DT).round() as usize {
        idle(w);
        ev.extend(w.step(&Actions::default(), t));
    }
    ev
}

fn flat_gap(w: &World, i: usize) -> f32 {
    let (p, q) = (w.player.pos, w.enemies[i].pos);
    vec3(q.x - p.x, 0.0, q.z - p.z).length()
}

/// 표: 한손(방패와 든다 — 방패를 들면 방패 판) · 평타 셋(검과 같은 피해 — 앞의 둘은 조금 넓고, 셋째는 넓고 길고 느리다) · 셋째 타와
/// 강공격만 끌어온다(넉백 대신) · 기력은 검과 같다 · 주특성 되살아나는 숨만(물리력 · 급소 · 분노는 없다).
#[test]
fn the_sickle_is_one_handed_and_its_third_cut_and_charge_hook_foes_in() {
    let s = W::Sword.def();
    let k = W::Scythe.def();
    assert_eq!(W::Scythe.hand(), weapons::Hand::One);
    assert_eq!(k.combo.len(), s.combo.len());
    let dmg = |w: &weapons::Weapon| w.combo.iter().map(|h| h.damage).collect::<Vec<_>>();
    assert_eq!(dmg(k), dmg(s), "the sword's numbers");
    for h in &k.combo[..2] {
        assert!(h.arc > s.combo[0].arc && h.pull == 0.0, "{}", h.anim);
    }
    let hook = k.combo[2];
    assert_eq!((hook.pull, hook.knock), (SCYTHE_PULL, 0.0));
    assert!(hook.time > 1.0 && hook.arc > 1.0 && hook.reach > 1.0);
    assert_eq!(k.heavy.unwrap().pull, SCYTHE_HEAVY_PULL);
    assert!(
        s.combo.iter().all(|h| h.pull == 0.0),
        "the sword knocks back"
    );
    assert_eq!(k.stamina, s.stamina);
    let art = W::Scythe.art();
    assert_eq!(
        (art.sap, art.force, art.sneak, art.fury, art.channel),
        (ART_SAP, 1.0, 1.0, 1.0, 1.0)
    );
    assert_eq!(W::Sword.art().sap, 1.0, "only the sickle");
}

/// 끌어오기: 몸 앞(방랑자 · 적의 몸 반지름 + `PULL_STOP`)까지 — 넘어서 끌지 않는다, 멀리서는 많아야 `pull` m.
#[test]
fn a_hook_drags_no_further_than_the_feet() {
    let near = PLAYER_RADIUS + 0.4 + PULL_STOP;
    assert_eq!(pull_slide(near - 0.2, 0.4, 1.2), 0.0, "already at the feet");
    assert!((pull_slide(near + 0.5, 0.4, 1.2) - 0.5).abs() < 1e-5);
    assert!(
        (pull_slide(near + 3.0, 0.4, 1.2) - 1.2).abs() < 1e-5,
        "at most the pull"
    );
}

/// 셋째 타(걸어 당기며 베기)는 앞의 적을 몸 앞으로 끌어온다 — 앞의 두 타는 검처럼 조금 밀어낸다. 문지기는 끌리지 않는다(무겁다).
#[test]
fn the_third_cut_drags_a_mob_in_but_not_a_warden() {
    let t = Tuning::default();
    let go = |boss: bool| {
        let mut w = World::new("test", 30.0);
        w.set_weapon(W::Scythe);
        w.player.facing = std::f32::consts::PI;
        let at = vec3(0.0, 0.0, -2.3);
        if boss {
            w.spawn_boss(at);
        } else {
            w.spawn_mob(at);
        }
        w.enemies[0].hp = 100_000;
        w.enemies[0].max_hp = 100_000;
        let mut before = None;
        let mut hits = Vec::new();
        // 0277: 끌려오는 동안 덩굴의 표지(`Enemy::tug`)와 소리 · 이펙트의 사건.
        let (mut pulls, mut tugged) = (0, 0.0f32);
        for k in 0..240 {
            idle(&mut w);
            w.player.stamina = t[P::StaminaMax];
            // 셋째 타가 시작하기 전까지 적을 같은 자리에 둔다(앞의 두 타의 넉백을 지운다).
            let third = matches!(w.player.state, PState::Attack { hit: 2, .. });
            if !third {
                w.enemies[0].pos = at;
                w.enemies[0].knock = Vec3::ZERO;
            } else if before.is_none() {
                before = Some(flat_gap(&w, 0));
            }
            let ev = w.step(
                &Actions {
                    attack: k % 2 == 0 && !third,
                    ..Default::default()
                },
                &t,
            );
            pulls += ev
                .iter()
                .filter(|e| matches!(e, Event::Pull { .. }))
                .count();
            tugged = tugged.max(w.enemies[0].tug);
            if let PState::Attack { hit, .. } = w.player.state {
                if hits.last() != Some(&hit) {
                    hits.push(hit);
                }
            }
            if third && !matches!(w.player.state, PState::Attack { hit: 2, .. }) {
                break;
            }
        }
        assert!(hits.contains(&2), "the third cut came: {hits:?}");
        for _ in 0..30 {
            idle(&mut w);
            w.step(&Actions::default(), &t);
        }
        // 덩굴은 미끄러짐 동안만 — 반 초 뒤에는 남지 않는다.
        assert_eq!(w.enemies[0].tug, 0.0, "the vine lets go");
        (
            before.unwrap(),
            flat_gap(&w, 0),
            w.enemies[0].radius,
            w.enemies[0].pos,
            (pulls, tugged),
        )
    };
    let (was, now, r, _, (pulls, tugged)) = go(false);
    // 0277: 끌린 잡몹에는 덩굴이 걸리고(`PULL_SHOW`초) 사건이 하나 이상 — 셋째 타 한 번에 한 번.
    assert!(pulls >= 1, "the hook says so");
    assert!(
        tugged > crate::weapons::PULL_SHOW - 2.0 * TICK_DT && tugged <= crate::weapons::PULL_SHOW
    );
    // (방랑자도 휘두르며 내딛는다 — 셋째 타가 시작할 때 이미 가깝다.) 끌려와 몸 앞에 멈춘다.
    let feet = PLAYER_RADIUS + r + PULL_STOP;
    assert!(now < was - 0.3, "dragged in: {was:.2} -> {now:.2}");
    assert!(
        (now - feet).abs() < 0.15,
        "stops at the feet ({feet:.2}): {now:.2}"
    );
    // 문지기: 방랑자가 내딛어 가까워질 뿐 몸은 제자리(방랑자 쪽으로 오지 않는다).
    let (_, _, _, pos, (pulls, tugged)) = go(true);
    assert!(pos.z <= -2.3 + 0.05, "a warden stays: {pos:?}");
    assert_eq!(
        (pulls, tugged),
        (0, 0.0),
        "no vine on a warden that does not move"
    );
}

/// 되살아나는 숨(주특성): 묶인 적을 치면 흡수처럼 체력 · 기력이 돌아오고(다른 무기는 아니다), 흡수가 걸린 적을 치면 다른 무기보다
/// `ART_SAP`배 돌아온다 — 한 틱에 한 번.
#[test]
fn the_sickle_draws_life_from_bound_foes_and_more_from_drained_ones() {
    let t = Tuning::default();
    let sip = |id: W, bound: bool, drained: bool| {
        let mut w = World::new("test", 30.0);
        w.set_weapon(id);
        for x in [-1.0, 1.0] {
            mob_with(&mut w, vec3(x, 0.0, -2.0), 500);
        }
        idle(&mut w);
        for en in &mut w.enemies {
            if bound {
                en.ail.bound = 1.0;
            }
            if drained {
                en.ail.drain = 3.0;
            }
            en.struck_now = true;
        }
        w.player.hp = 10;
        w.player.stamina = 0.0;
        w.step(&Actions::default(), &t);
        (w.player.hp - 10, w.player.stamina)
    };
    use crate::tuning::{DRAIN_HP, DRAIN_STAMINA};
    assert_eq!(
        sip(W::Sword, true, false).0,
        0,
        "a sword takes nothing from roots"
    );
    let (hp, _) = sip(W::Scythe, true, false);
    assert_eq!(
        hp,
        (DRAIN_HP as f32 * ART_SAP).round() as i32,
        "the sickle does"
    );
    assert_eq!(sip(W::Sword, false, true).0, DRAIN_HP);
    let (hp, st) = sip(W::Scythe, false, true);
    assert_eq!(
        hp,
        (DRAIN_HP as f32 * ART_SAP).round() as i32,
        "once a tick"
    );
    assert!(st >= DRAIN_STAMINA * ART_SAP - 1e-3);
}

/// 거두는 손(보조특성 — 멘 낫): 적을 쓰러뜨릴 때마다 체력 +`SUB_KILL_HEAL`. 받는 피해 · 방어는 건드리지 않는다(종류 표의 시험).
#[test]
fn a_reaping_hand_heals_for_each_foe_felled() {
    let t = Tuning::default();
    let felled = |kill_heal: i32| {
        let mut w = World::new("test", 30.0);
        w.set_weapon(W::Sword);
        w.stats = crate::stats::Stats::compute(&[crate::stats::Mods {
            kill_heal,
            ..crate::stats::Mods::NONE
        }]);
        w.player.facing = std::f32::consts::PI;
        mob_with(&mut w, vec3(0.0, 0.0, -1.2), 1);
        idle(&mut w);
        w.player.hp = 10;
        w.step(
            &Actions {
                attack: true,
                ..Default::default()
            },
            &t,
        );
        wait(&mut w, &t, 0.5);
        assert!(w.enemies[0].hp <= 0, "felled");
        w.player.hp - 10
    };
    assert_eq!(felled(0), 0);
    assert_eq!(felled(SUB_KILL_HEAL), SUB_KILL_HEAL);
    assert_eq!(W::Scythe.sub_trait().unwrap().mods.kill_heal, SUB_KILL_HEAL);
}

/// 땅을 짚는 손(빈 손의 결): 보조무기 슬롯이 비면 뿌리 스킬의 치수(반지름 · 길이 · 각)와 가시 씨앗이 솟는 둘레가 `BARE_GROUND_REACH`배 —
/// 뿌리가 아닌 스킬(생기 거두기)은 그대로, 보조무기를 메면 없다.
#[test]
fn a_hand_on_the_ground_widens_root_skills_and_the_seed() {
    use crate::skills::{Form, Id as Sk};
    let mut w = World::new("test", 30.0);
    w.set_weapon(W::Scythe);
    let roots = |w: &World| match w.skill_eff(Sk::Roots).skill.form {
        Form::Erupt { len, .. } => len,
        _ => unreachable!(),
    };
    let reap = |w: &World| match w.skill_eff(Sk::Reap).skill.form {
        Form::Cone { arc_deg, .. } => arc_deg,
        _ => unreachable!(),
    };
    let harvest = |w: &World| match w.skill_eff(Sk::Harvest).skill.form {
        Form::Reap { radius, .. } => radius,
        _ => unreachable!(),
    };
    let base = (roots(&w), reap(&w), harvest(&w), w.rb_bolts().0.scale);
    w.set_open_hand(true);
    assert_eq!(w.bare(), Some(weapons::Bare::GroundHand));
    let k = BARE_GROUND_REACH;
    assert!((roots(&w) - base.0 * k).abs() < 1e-4);
    assert!((reap(&w) - base.1 * k).abs() < 1e-4);
    assert_eq!(harvest(&w), base.2, "not a root skill");
    assert!((w.rb_bolts().0.scale - base.3 * k).abs() < 1e-4);
    assert!((w.rb_bolts().1.scale - W::Scythe.bolts().1.scale * k).abs() < 1e-4);
    w.set_sub(Some(W::Sword));
    assert_eq!(w.bare(), None);
    assert_eq!(roots(&w), base.0, "an off-hand weapon takes it away");
}

/// RB = 가시 씨앗: 날지 않고 겨눈 자리(줄 위의 첫 적)에 떨어져 둘레를 찌른다(신성한 빛과 같은 길 — 그 자리의 둘레만) · 뿌리를 반 쌓아
/// 묶는다 · 솟은 가시 뿌리가 그림으로 남는다(`Event::Sprout`).
#[test]
fn the_thorn_seed_falls_where_aimed_and_binds() {
    let t = Tuning::default();
    let (bolt, charged) = W::Scythe.bolts();
    assert_eq!((bolt.look, charged.look), (Look::Seed, Look::Seed));
    assert!(bolt.smites() && !bolt.straight());
    assert_eq!(bolt.dose.0, Kind::Root);
    assert!(charged.dose.1 >= 1.0, "a full seed binds at once");
    let mut w = World::new("test", 30.0);
    w.set_weapon(W::Scythe);
    w.player.facing = std::f32::consts::PI;
    // 가까운 적을 찾아 그 발밑에 떨어진다(겨누지 않으면) — 곁(4 m 옆)의 적은 둘레 밖.
    mob_with(&mut w, vec3(0.0, 0.0, -5.0), 500);
    mob_with(&mut w, vec3(4.0, 0.0, -5.0), 500);
    idle(&mut w);
    // 뿌리 반(0.5)씩 — 보통 적은 둘이면, 뿌리에 덜 걸리는 잔재(×0.75)는 셋이면 묶인다.
    let mut ev = Vec::new();
    for _ in 0..3 {
        idle(&mut w);
        ev.extend(w.step(
            &Actions {
                cast: true,
                ..Default::default()
            },
            &t,
        ));
        ev.extend(wait(&mut w, &t, 0.6));
    }
    assert_eq!(
        ev.iter()
            .filter(|e| matches!(e, Event::Sprout { .. }))
            .count(),
        3
    );
    assert!(w.enemies[0].hp < 500, "struck where it fell");
    assert_eq!(w.enemies[1].hp, 500, "only around where it fell");
    assert!(
        ev.iter().any(|e| matches!(
            e,
            Event::Ail {
                kind: Kind::Root,
                ..
            }
        )),
        "the seeds bind"
    );
}

/// 초당 피해(20초): 한 적에게는 검과 같거나 조금 아래(85~100%), 앞에 붙은 셋에게는 검 위(셋째 타가 넓게 걸어 당긴다 — 1.3배 넘게) ·
/// 대검 · 망치보다는 아래(떼를 베는 무기가 아니라 묶고 되살아나는 손이다).
#[test]
fn the_sickle_is_a_sword_alone_and_more_on_a_huddle_but_under_the_greatsword() {
    for beat in [false, true] {
        let sword = super::daggers::foe_dps(W::Sword, false, 20.0, beat);
        let sickle = super::daggers::foe_dps(W::Scythe, false, 20.0, beat);
        assert!(
            sickle > sword * 0.85 && sickle <= sword * 1.0,
            "one foe (beat {beat}): sword {sword:.1} sickle {sickle:.1}"
        );
        let (sw3, sc3, gs3) = (
            super::daggers::huddle_dps(W::Sword, 20.0, beat),
            super::daggers::huddle_dps(W::Scythe, 20.0, beat),
            super::daggers::huddle_dps(W::Greatsword, 20.0, beat),
        );
        assert!(
            sc3 > sw3 * 1.3 && sc3 < gs3,
            "a huddle (beat {beat}): sword {sw3:.1} sickle {sc3:.1} greatsword {gs3:.1}"
        );
    }
}

/// 재는 도구: `cargo test scythe_probe -- --ignored --nocapture`
#[test]
#[ignore]
fn scythe_probe() {
    for id in [W::Sword, W::Hammer, W::Greatsword, W::Scythe] {
        for beat in [false, true] {
            println!(
                "{id:?} {}: one {:.1}/s · huddle of 3 {:.1}/s · 4 s one {:.1}/s",
                if beat { "beat" } else { "mash" },
                super::daggers::foe_dps(id, false, 20.0, beat),
                super::daggers::huddle_dps(id, 20.0, beat),
                super::daggers::foe_dps(id, false, 4.0, beat)
            );
        }
    }
}
