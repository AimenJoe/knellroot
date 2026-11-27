//! 완드(0274 ⑥ — 흑마법사): 짧게 치는 3연타(셋째 찌르기와 강공격은 균열을 새긴다) · RB 저주탄(곧게 날아 균열을 쌓는 주된 공격 — 칸 다섯이
//! 빨리 찬다) · 주특성 갚지 않은 빚(금의 덤 ×1.5 · 금 간 채 쓰러지면 둘레에 울림) · 보조특성 뜻 모를 말(원거리 공격 피해) · 빈 손의 결
//! "계약의 표"(저주탄의 표가 남은 적이 쓰러지면 체력 · 칸이 돌아온다).
use super::*;
use crate::det::Det;
use crate::status::Kind;
use crate::weapons::{
    Id as W, Look, ART_HEX, BARE_PACT_CHARGE, BARE_PACT_HEAL, DEATH_KNELL_R, SUB_TOSS, WAND_CHARGES,
};

fn wait(w: &mut World, t: &Tuning, secs: f32) -> Vec<Event> {
    let mut ev = Vec::new();
    for _ in 0..(secs / TICK_DT).round() as usize {
        idle(w);
        ev.extend(w.step(&Actions::default(), t));
    }
    ev
}

fn gauge(en: &crate::enemy::Enemy, k: Kind) -> f32 {
    en.ail.gauge[k.slot().unwrap()]
}

/// 표: 한 손에 들고 0281부터 방패와 든다(계열 `One` — 방패를 들어도 완드 판이 먼저 · 방패는 RB를 맡지 않는다) · 짧은 3연타(타마다 10 — 검의 끝 타
/// 20이 없다, 닿는 거리는 검보다 짧다) · 셋째 찌르기와 강공격만 균열을 싣는다 · 주특성 갚지 않은 빚만 · RB = 저주탄(곧게 · 던지는 것이
/// 아니다 · 내려 꽂히지 않는다 · 균열) — 칸이 다섯이고 빨리 찬다.
#[test]
fn the_wand_is_short_and_its_third_jab_and_charge_etch_a_crack() {
    let s = W::Sword.def();
    let k = W::Wand.def();
    assert_eq!(W::Wand.hand(), weapons::Hand::One);
    assert_eq!(
        W::Wand.grip(true),
        weapons::Grip::Shield(Some(weapons::Class::Wand))
    );
    let mut w = World::new("test", 30.0);
    w.set_weapon(W::Wand);
    w.set_shield(Some(crate::shield::Id::Pilgrim));
    assert!(!w.shield_throws(), "the curse stays in the hand");
    assert_eq!(w.rb_bolts().0.look, Look::Curse);
    assert_eq!(k.combo.len(), 3);
    assert!(k.combo.iter().all(|h| h.damage == 10));
    assert!(k.combo.iter().all(|h| h.reach < s.combo[0].reach));
    for h in &k.combo[..2] {
        assert_eq!(h.dose.1, 0.0, "{}", h.anim);
    }
    assert_eq!(k.combo[2].dose.0, Kind::Rift);
    let heavy = k.heavy.unwrap();
    assert_eq!(heavy.dose, (Kind::Rift, 1.0));
    assert!(s.combo.iter().all(|h| h.dose.1 == 0.0) && s.heavy.unwrap().dose.1 == 0.0);
    let art = W::Wand.art();
    assert_eq!(
        (
            art.hex(),
            art.force(),
            art.sneak(),
            art.fury(),
            art.channel(),
            art.sap()
        ),
        (ART_HEX, 1.0, 1.0, 1.0, 1.0, 1.0)
    );
    assert_eq!(W::Sword.art().hex(), 1.0, "only the wand");
    let (bolt, charged) = W::Wand.bolts();
    assert_eq!((bolt.look, charged.look), (Look::Curse, Look::Curse));
    assert!(bolt.straight() && !bolt.thrown() && !bolt.smites() && !bolt.is_shot());
    assert!(
        !bolt.pierce && charged.pierce,
        "a gathered curse goes through"
    );
    assert_eq!((bolt.dose.0, charged.dose), (Kind::Rift, (Kind::Rift, 1.0)));
    let toss = W::Wand.toss().unwrap();
    assert!(toss.charges > crate::tuning::CAST_CHARGES && toss.refill < 1.0);
    assert_eq!(w.cast_charges(), WAND_CHARGES);
    w.set_weapon(W::Sword);
    assert_eq!(w.cast_charges(), crate::tuning::CAST_CHARGES);
}

/// 저주탄은 땅을 따라 흐르지 않고 곧게 난다(볼트처럼 겨눈 줄 위 첫 적의 몸통으로 — 한 단 아래 · 위의 적도 맞는다), 벼랑 위(3 m)의 적에게는
/// 벼랑 면에 막힌다. 맞으면 균열이 쌓인다(셋이면 금).
#[test]
fn the_curse_bolt_flies_straight_to_foes_above_and_below_and_cracks() {
    use super::ranged::split_yard;
    let t = Tuning::default();
    let shoot = |w: &mut World| {
        idle(w);
        press(
            &t,
            w,
            Actions {
                cast: true,
                ..Default::default()
            },
            90,
        )
    };
    for (far, tag) in [(-1.0, "one metre below"), (1.0, "on a one-metre ledge")] {
        let mut w = split_yard(far);
        w.set_weapon(W::Wand);
        w.spawn_mob(vec3(0.0, far, -2.0));
        w.enemies[0].hp = 900;
        shoot(&mut w);
        assert!(w.enemies[0].hp < 900, "{tag}");
        assert!(gauge(&w.enemies[0], Kind::Rift) > 0.0 || w.enemies[0].ail.rift > 0.0);
    }
    let mut w = split_yard(3.0);
    w.set_weapon(W::Wand);
    w.spawn_mob(vec3(0.0, 3.0, -2.0));
    w.enemies[0].hp = 900;
    shoot(&mut w);
    assert_eq!(w.enemies[0].hp, 900, "a cliff face stops it");
    // 셋이면 금이 간다.
    let mut w = World::new("test", 30.0);
    w.set_weapon(W::Wand);
    w.player.facing = std::f32::consts::PI;
    mob_with(&mut w, vec3(0.0, 0.0, -5.0), 900);
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
        ev.extend(wait(&mut w, &t, 0.5));
    }
    assert!(w.enemies[0].ail.rift > 0.0, "cracked");
    assert!(ev.iter().any(|e| matches!(
        e,
        Event::Ail {
            kind: Kind::Rift,
            ..
        }
    )));
}

/// 갚지 않은 빚(주특성): 금 간 적이 받는 피해의 덤이 ×`ART_HEX`(잡몹 +30% → +45% · 문지기 +15% → +22.5%) — 완드를 든 손의 타만(다른 무기 ·
/// 타입이 스스로 낸 피해는 그대로). 금 간 채 쓰러진 적은 둘레 `DEATH_KNELL_R` m의 적에게 울림을 남긴다(완드를 들었을 때만) — 둘레 밖은 아니다.
#[test]
fn an_unpaid_debt_deepens_the_crack_and_a_cracked_death_tolls_around() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    let i = mob_with(&mut w, vec3(0.0, 0.0, -3.0), 900);
    w.enemies[i].ail.rift = 3.0;
    let plain = Knack::NONE;
    let hex = Knack {
        hex: ART_HEX,
        ..Knack::NONE
    };
    let en = &w.enemies[i];
    assert_eq!(hurt_by(en, 100, false, 0, plain, &t), 130);
    assert_eq!(hurt_by(en, 100, false, 0, hex, &t), 145);
    w.enemies[i].ail.rift = 0.0;
    assert_eq!(
        hurt_by(&w.enemies[i], 100, false, 0, hex, &t),
        100,
        "no crack, no debt"
    );
    w.set_weapon(W::Wand);
    assert_eq!(w.knack().hex, ART_HEX);
    w.set_weapon(W::Sword);
    assert_eq!(w.knack().hex, 1.0);
    let knell = |id: W| {
        let mut w = World::new("test", 30.0);
        w.set_weapon(id);
        let a = mob_with(&mut w, vec3(0.0, 0.0, -3.0), 900);
        let b = mob_with(&mut w, vec3(2.0, 0.0, -3.0), 900);
        let c = mob_with(&mut w, vec3(0.0, 0.0, -3.0 - DEATH_KNELL_R - 3.0), 900);
        idle(&mut w);
        w.enemies[a].ail.rift = 3.0;
        w.enemies[a].hp = 0;
        w.enemies[a].state = EState::Dead;
        let ev = w.step(&Actions::default(), &t);
        (
            gauge(&w.enemies[b], Kind::Knell),
            gauge(&w.enemies[c], Kind::Knell),
            ev.iter().any(|e| matches!(e, Event::DeathKnell { .. })),
        )
    };
    let (near, far, rang) = knell(W::Wand);
    assert!(near > 0.0 && rang, "the debt tolls around the fallen");
    assert_eq!(far, 0.0, "only around it");
    let (near, _, rang) = knell(W::Sword);
    assert!(near == 0.0 && !rang, "only the wand's hand");
}

/// 죽음의 울림은 떼의 크기로 곱절이 되지 않는다(0223 8절): 한 틱에 여럿이 쓰러져도 한 적은 한 번만 울림을 받는다.
#[test]
fn many_cracked_deaths_in_one_tick_toll_a_survivor_once() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    w.set_weapon(W::Wand);
    let s = mob_with(&mut w, vec3(0.0, 0.0, -3.0), 900);
    for k in 0..6 {
        let a = k as f32 / 6.0 * std::f32::consts::TAU;
        mob_with(
            &mut w,
            vec3(a.sin_d() * 1.5, 0.0, -3.0 + a.cos_d() * 1.5),
            900,
        );
    }
    idle(&mut w);
    for en in &mut w.enemies[1..] {
        en.ail.rift = 3.0;
        en.hp = 0;
        en.state = EState::Dead;
    }
    w.step(&Actions::default(), &t);
    let one = weapons::DEATH_KNELL_DOSE;
    let g = gauge(&w.enemies[s], Kind::Knell);
    assert!(g > 0.0 && g <= one + 1e-4, "once: {g}");
}

/// 뜻 모를 말(보조특성 — 멘 완드): 종류의 원거리 수단(손도끼 · 빛 · 마법 탄 …)의 피해 +`SUB_TOSS` — 사격 무기의 볼트 · 화살은 아니다.
/// 든 동안은 없다(종류 표의 시험 — 보조특성은 멘 것만).
#[test]
fn words_of_unknown_meaning_add_to_the_toss_and_not_to_shots() {
    let with = |id: W, toss_dmg: i32| {
        let mut w = World::new("test", 30.0);
        w.set_weapon(id);
        w.stats = crate::stats::Stats::compute(&[crate::stats::Mods {
            toss_dmg,
            ..crate::stats::Mods::NONE
        }]);
        let (a, b) = w.rb_bolts();
        (a.damage, b.damage)
    };
    for id in [W::Sword, W::Hammer, W::Staff, W::Fists] {
        let (a, b) = with(id, 0);
        assert_eq!(with(id, SUB_TOSS), (a + SUB_TOSS, b + SUB_TOSS), "{id:?}");
    }
    for id in [W::Crossbow, W::Bow] {
        assert_eq!(
            with(id, SUB_TOSS),
            with(id, 0),
            "{id:?}: a shot is no curse"
        );
    }
    assert_eq!(W::Wand.sub_trait().unwrap().mods.toss_dmg, SUB_TOSS);
}

/// 계약의 표(빈 손의 결): 보조무기 슬롯이 비면 저주탄에 맞은 표가 남은 적이 쓰러질 때 체력 `BARE_PACT_HEAL`과 RB 칸 `BARE_PACT_CHARGE`가
/// 돌아온다 — 표 없이 쓰러진 적(평타)은 아니다, 보조무기를 메면 없다. 받는 피해 · 패링 · 회피는 건드리지 않는다(`off_hand.rs`).
#[test]
fn the_pacts_mark_pays_back_health_and_a_charge_when_a_marked_foe_falls() {
    let t = Tuning::default();
    let go = |open: bool, sub: Option<W>| {
        let mut w = World::new("test", 30.0);
        w.set_weapon(W::Wand);
        w.set_open_hand(open);
        w.set_sub(sub);
        w.player.facing = std::f32::consts::PI;
        mob_with(&mut w, vec3(0.0, 0.0, -5.0), 5);
        idle(&mut w);
        w.player.hp = 10;
        w.player.ranged.mana = 2.0;
        let mut ev = w.step(
            &Actions {
                cast: true,
                ..Default::default()
            },
            &t,
        );
        ev.extend(wait(&mut w, &t, 0.6));
        assert!(w.enemies[0].hp <= 0, "felled by the curse");
        (w.player.hp - 10, w.player.ranged.mana, ev)
    };
    let (hp, mana, ev) = go(true, None);
    assert_eq!(hp, BARE_PACT_HEAL);
    assert!(ev.iter().any(|e| matches!(e, Event::Mend { .. })));
    let (_, plain_mana, _) = go(false, None);
    assert!(
        (mana - plain_mana - BARE_PACT_CHARGE).abs() < 0.05,
        "a charge back: {mana:.2} vs {plain_mana:.2}"
    );
    let (hp, _, _) = go(true, Some(W::Sword));
    assert_eq!(hp, 0, "an off-hand weapon takes it away");
    // 평타로 쓰러뜨린 적은 표가 없다.
    let mut w = World::new("test", 30.0);
    w.set_weapon(W::Wand);
    w.set_open_hand(true);
    w.player.facing = std::f32::consts::PI;
    mob_with(&mut w, vec3(0.0, 0.0, -1.1), 1);
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
    assert!(w.enemies[0].hp <= 0);
    assert_eq!(w.player.hp, 10, "no mark, no payment");
}

/// 초당 피해(20초, 1.2 m): 완드로 치는 것은 검보다 약하다(한 적 75~95%) — 주된 공격은 저주탄이다: RB 초당 피해가 근접 종류의 RB
/// 가운데 가장 크고(손도끼 · 빛보다), 완드 평타의 7할 밑이다(안전한 거리의 값 — 0173).
#[test]
fn the_wand_strikes_softer_than_a_sword_and_its_curse_is_the_strongest_toss() {
    for beat in [false, true] {
        let sword = super::daggers::foe_dps(W::Sword, false, 20.0, beat);
        let wand = super::daggers::foe_dps(W::Wand, false, 20.0, beat);
        assert!(
            wand > sword * 0.75 && wand < sword * 0.95,
            "beat {beat}: sword {sword:.1} wand {wand:.1}"
        );
    }
    let curse = super::toss::rb_dps(W::Wand, 20.0, 8.0, false, false);
    for id in weapons::IDS
        .into_iter()
        .filter(|i| !i.is_ranged() && *i != W::Wand)
    {
        let other = super::toss::rb_dps(id, 20.0, 8.0, false, false);
        assert!(curse > other, "{id:?}: {other:.1} vs the curse {curse:.1}");
    }
    let melee = super::ranged::dps(W::Wand, 20.0, true, 1.2).0;
    assert!(curse < melee * 0.7, "curse {curse:.1} vs melee {melee:.1}");
}

/// 재는 도구: `cargo test wand_probe -- --ignored --nocapture`
#[test]
#[ignore]
fn wand_probe() {
    for id in [W::Sword, W::Staff, W::Daggers, W::Scythe, W::Wand] {
        for beat in [false, true] {
            println!(
                "{id:?} {}: one {:.1}/s · huddle of 3 {:.1}/s · 4 s one {:.1}/s",
                if beat { "beat" } else { "mash" },
                super::daggers::foe_dps(id, false, 20.0, beat),
                super::daggers::huddle_dps(id, 20.0, beat),
                super::daggers::foe_dps(id, false, 4.0, beat),
            );
        }
    }
}

/// 0281(멀티클래스 — 사장 2026-11-27 "완드도 사실 방패 가능하긴 해야할것 같은데"): 완드는 방패와 든다 — 쥔 꼴은 완드 트리와 방패
/// 트리 둘, RB는 **그대로 저주탄**(칸을 쓴다 · 방패는 날지 않는다 — `Quirks::rb_main`), 방패는 길게 눌러 막고 놓으면 들이받는다.
#[test]
fn the_wand_takes_a_shield_that_guards_and_bashes_but_its_rb_stays_the_curse_bolt() {
    let t = Tuning::default();
    let mut w = World::new("test", 40.0);
    w.set_weapon(W::Wand);
    let alone_icon = crate::touch::rb_icon(&w);
    w.set_shield(Some(crate::shield::Id::Pilgrim));
    assert_eq!(
        w.grip(),
        weapons::Grip::Shield(Some(weapons::Class::Wand)),
        "both trees"
    );
    assert!(w.shield_guards() && w.can_block());
    assert!(!w.shield_throws(), "the curse stays the ranged attack");
    assert_eq!(w.rb_toss().map(|r| r.bolt.look), Some(Look::Curse));
    assert_eq!(w.rb_bolts().0.look, Look::Curse);
    assert_eq!(crate::touch::rb_icon(&w), alone_icon, "the RB icon too");
    assert_eq!(w.bare(), None, "no empty-hand way with a shield");
    let foe = mob_with(&mut w, vec3(0.0, 0.0, -6.0), 900);
    idle(&mut w);
    let mana = w.player.ranged.mana;
    let mut ev = w.step(
        &Actions {
            cast: true,
            ..Default::default()
        },
        &t,
    );
    ev.extend(wait(&mut w, &t, 0.8));
    assert!(!w.shield_out(), "the shield never leaves the arm");
    assert!(w.player.ranged.mana < mana, "a curse charge was spent");
    assert!(
        !ev.iter().any(|e| matches!(e, Event::ShieldBounce { .. })),
        "no shield in the air"
    );
    assert!(w.enemies[foe].hp < 900, "the curse bolt struck");
    // 길게 누르면 막고, 놓으면 모인 만큼 들이받는다.
    let hold = Actions {
        parry: true,
        parry_held: true,
        ..Default::default()
    };
    w.step(&hold, &t);
    let hold = Actions {
        parry_held: true,
        ..Default::default()
    };
    for _ in 0..60 {
        w.step(&hold, &t);
    }
    assert!(w.guarding(), "the wand's shield guards");
    w.step(&Actions::default(), &t);
    assert!(
        matches!(w.player.state, PState::Rush { .. }),
        "and bashes on release: {:?}",
        w.player.state
    );
}

/// 0281: 한손검 · 망치 · 낫(과 맨손)은 방패와 들면 그 전처럼 RB가 방패 던지기다 — 바뀐 것은 무기 트리를 함께 쓰는 것뿐.
#[test]
fn a_sword_hammer_or_scythe_with_a_shield_still_throws_it() {
    for arm in [W::Sword, W::Hammer, W::Scythe, W::Fists] {
        let mut w = World::new("test", 40.0);
        w.set_weapon(arm);
        w.set_shield(Some(crate::shield::Id::Pilgrim));
        assert!(w.shield_throws() && w.can_block(), "{arm:?}");
        assert_eq!(
            w.rb_toss().map(|r| r.bolt.look),
            Some(Look::Shield),
            "{arm:?}"
        );
        assert_eq!(
            crate::touch::rb_icon(&w),
            crate::touch::Kind::ShieldToss,
            "{arm:?}"
        );
        assert_eq!(w.grip(), weapons::Grip::Shield(arm.class()), "{arm:?}");
    }
}
