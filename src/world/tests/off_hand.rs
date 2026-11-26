//! 보조 손(0276): 보조무기 슬롯이 비었을 때 종류마다의 "빈 손의 결"과, 하데스처럼 던지고 들이받는 방패.
use super::*;
use crate::shield;
use crate::weapons::{Bare, Id, CLASSES};

fn wait(w: &mut World, t: &Tuning, secs: f32) -> Vec<Event> {
    let mut ev = Vec::new();
    for _ in 0..(secs / TICK_DT).round() as usize {
        ev.extend(w.step(&Actions::default(), t));
    }
    ev
}

/// 그 종류의 첫 무기.
fn weapon_of(c: crate::weapons::Class) -> Id {
    *crate::weapons::IDS
        .iter()
        .find(|w| w.class() == Some(c))
        .expect("every class has a weapon")
}

/// 보조무기 슬롯이 빈 방랑자(앱이 `arm_world`에서 넘기는 것과 같은 길).
fn bare_world(weapon: Id) -> World {
    let mut w = World::new("test", 30.0);
    w.set_weapon(weapon);
    w.set_sub(None);
    w.set_shield(None);
    w.set_open_hand(true);
    w
}

/// 0276: 종류마다 하나의 결이 켜지고, 보조무기 슬롯에 무엇이든(무기 · 방패) 들면 꺼진다. 앱이 비었다고 하지 않으면 없다(시험 ·
/// 시나리오의 `World::new`). 맨손은 종류가 아니라 없다.
#[test]
fn every_class_has_an_empty_hand_way_that_anything_in_the_off_hand_turns_off() {
    let mut seen = Vec::new();
    for c in CLASSES.iter() {
        let id = weapon_of(c.class);
        let mut w = bare_world(id);
        assert_eq!(w.bare(), Some(c.bare), "{id:?}");
        assert!(!seen.contains(&c.bare), "{:?} twice", c.bare);
        seen.push(c.bare);
        let other = if id == Id::Sword {
            Id::Hammer
        } else {
            Id::Sword
        };
        w.set_sub(Some(other));
        assert_eq!(w.bare(), None, "{id:?} with an off-hand weapon");
        w.set_sub(None);
        w.set_shield(Some(shield::Id::Pilgrim));
        assert_eq!(w.bare(), None, "{id:?} with a shield (even a resting one)");
        w.set_shield(None);
        assert_eq!(w.bare(), Some(c.bare));
        w.set_open_hand(false);
        assert_eq!(w.bare(), None, "the app did not say the slot is empty");
    }
    assert_eq!(
        bare_world(Id::Fists).bare(),
        None,
        "bare fists are not a class"
    );
    let mut w = World::new("test", 30.0);
    w.set_weapon(Id::Sword);
    assert_eq!(w.bare(), None, "a fresh world knows no slot");
}

/// 0276: 바꿔 들면 보조무기 슬롯에는 옛 주무기가 남는다 — 결은 없다(둘 다 바꿔 든 뒤에도). 앱이 슬롯을 비우면 돌아온다.
#[test]
fn after_a_swap_the_off_hand_holds_the_old_weapon_so_no_empty_hand() {
    let t = Tuning::default();
    let mut w = bare_world(Id::Sword);
    w.set_sub(Some(Id::Spear));
    assert_eq!(w.bare(), None);
    w.step(
        &Actions {
            swap: true,
            ..Default::default()
        },
        &t,
    );
    assert_eq!(w.player.weapon, Id::Spear);
    assert_eq!(w.player.sub, Some(Id::Sword));
    assert_eq!(w.bare(), None, "the sword now rides in the off hand");
    w.swap_weapons();
    assert_eq!(w.player.weapon, Id::Sword);
    assert_eq!(w.bare(), None);
    // 앱이 보조무기 슬롯을 비운다(장비 탭) — 같은 한 길.
    w.set_sub(None);
    w.set_open_hand(true);
    assert_eq!(w.bare(), Some(Bare::TwoHands));
    w.set_weapon(Id::Spear);
    assert_eq!(w.bare(), Some(Bare::LongGrip), "follows the held weapon");
}

/// 0276 원칙 3(0095 원칙 2): 빈 손의 결은 받는 피해 · 패링 판정 시간 · 회피 거리와 무적을 건드리지 않는다 — 종류마다, 슬롯이 빈
/// 방랑자와 막대를 든 방랑자를 같은 틱으로 돌려 견준다.
#[test]
fn no_empty_hand_way_touches_damage_taken_the_parry_window_or_the_dodge() {
    let t = Tuning::default();
    for c in CLASSES.iter() {
        let id = weapon_of(c.class);
        let run = |open: bool| {
            // 받는 피해: 앞의 잡몹이 내려친다.
            let mut w = mob_swinging_in_front(&t);
            w.set_weapon(id);
            w.set_open_hand(open);
            let hp0 = w.player.hp;
            wait(&mut w, &t, 0.6);
            let taken = hp0 - w.player.hp;
            // 패링 판정 시간: 누른 뒤 열려 있는 틱 수.
            let mut w = World::new("test", 30.0);
            w.set_weapon(id);
            w.set_open_hand(open);
            w.step(
                &Actions {
                    parry: true,
                    ..Default::default()
                },
                &t,
            );
            let mut open_ticks = 0;
            for _ in 0..60 {
                if w.parry_open(&t) {
                    open_ticks += 1;
                }
                w.step(&Actions::default(), &t);
            }
            // 회피: 간 거리 · 무적인 틱 수.
            let mut w = World::new("test", 30.0);
            w.set_weapon(id);
            w.set_open_hand(open);
            let p0 = w.player.pos;
            w.step(
                &Actions {
                    dodge: true,
                    move_dir: vec2(1.0, 0.0),
                    ..Default::default()
                },
                &t,
            );
            let mut inv = 0;
            for _ in 0..60 {
                if w.player.invulnerable() {
                    inv += 1;
                }
                w.step(&Actions::default(), &t);
            }
            (taken, open_ticks, inv, (w.player.pos - p0).length())
        };
        let (a, b) = (run(false), run(true));
        assert_eq!(a.0, b.0, "{id:?}: damage taken");
        assert_eq!(a.1, b.1, "{id:?}: parry window");
        assert_eq!(a.2, b.2, "{id:?}: dodge invulnerability");
        assert!((a.3 - b.3).abs() < 1e-3, "{id:?}: dodge distance");
    }
}

/// 0276 한손검 "두 손으로 쥔다": 강공격이 두 손 내려베기 — 망치의 내려치기 동작으로 바뀌고 자세 피해가 크다 · 평타 끝 타가 넓다.
/// 판정과 그림이 같은 셈(`hit_of` · `heavy_of`)을 읽는다.
#[test]
fn both_hands_on_the_sword_turn_the_heavy_into_a_poise_breaking_downward_cut() {
    let t = Tuning::default();
    let plain = Id::Sword.def();
    let w = bare_world(Id::Sword);
    let h = w.heavy_of().unwrap();
    assert_eq!(h.anim_heavy, "hammer_slam");
    assert!(h.reach > plain.heavy.unwrap().reach);
    let last = (plain.combo.len() - 1) as u8;
    assert!(w.hit_of(last).unwrap().arc > plain.hit(last).unwrap().arc * 1.4);
    assert_eq!(
        w.hit_of(0).unwrap().arc,
        plain.hit(0).unwrap().arc,
        "only the last"
    );
    // 같은 강공격을 서 있는 잡몹에 — 슬롯이 빈 쪽이 자세를 더 깎는다.
    let poise_after = |open: bool, boss: bool| {
        let mut w = World::new("test", 30.0);
        w.set_weapon(Id::Sword);
        w.set_open_hand(open);
        let i = mob_with(&mut w, vec3(0.0, 0.0, -1.6), 2000);
        w.enemies[i].boss = boss;
        idle(&mut w);
        let charge = ((t[P::HeavyCharge] + 0.6) / TICK_DT) as usize;
        hold_attack(&mut w, &t, charge, 60);
        w.enemies[i].stance.poise
    };
    let (mob, mob0) = (poise_after(true, false), poise_after(false, false));
    assert!(mob > mob0 * 1.3, "{mob} vs {mob0}");
    // 문지기의 자세 막대는 조금만(모의의 문턱).
    let (boss, boss0) = (poise_after(true, true), poise_after(false, true));
    assert!(boss > boss0 && boss < boss0 * 1.1, "{boss} vs {boss0}");
}

/// 0276 망치 "기도하는 손": 등잔 한 모금을 삼키는 순간 둘레에 울림이 쌓이고 밀려난다 — 회복은 그대로, 피해는 없다. 멀리 있는 적은 그대로.
#[test]
fn a_sip_with_praying_hands_rings_a_knell_around() {
    let t = Tuning::default();
    let run = |open: bool| {
        let mut w = World::new("test", 30.0);
        w.set_weapon(Id::Hammer);
        w.set_open_hand(open);
        let near = mob_with(&mut w, vec3(0.0, 0.0, -2.0), 500);
        let far = mob_with(&mut w, vec3(0.0, 0.0, -9.0), 500);
        idle(&mut w);
        w.player.hp = 10;
        let ev = press(
            &t,
            &mut w,
            Actions {
                flask: true,
                ..Default::default()
            },
            (t[P::FlaskTime] / TICK_DT) as usize + 2,
        );
        let knell = crate::status::Kind::Knell.slot().unwrap();
        (
            w.player.hp,
            w.enemies[near].ail.gauge[knell],
            w.enemies[near].pos.z,
            w.enemies[far].ail.gauge[knell],
            w.enemies[near].hp,
            ev.iter().any(|e| matches!(e, Event::Prayer { .. })),
        )
    };
    let (plain, pray) = (run(false), run(true));
    assert_eq!(plain.0, pray.0, "the heal is the same");
    assert!(!plain.5 && pray.5);
    assert!(plain.1 <= 0.0 && pray.1 > 0.3, "{pray:?}");
    assert!(
        pray.2 < plain.2 - 0.3,
        "pushed away: {} vs {}",
        pray.2,
        plain.2
    );
    assert_eq!(pray.3, 0.0, "the far one is untouched");
    assert_eq!(pray.4, 500, "no damage of its own");
}

/// 0276 쌍단검 "그림자 걸음": 회피가 끝난 뒤 첫 휘두르기는 앞에서 쳐도 급소(× 급소의 배) — 둘째부터는 아니다, 문지기는 등 뒤에서만
/// (`SHADOW_STEP_BOSS`).
#[test]
fn the_first_hit_after_a_dodge_with_a_shadow_step_finds_the_weak_spot() {
    let t = Tuning::default();
    let run = |open: bool, boss: bool| {
        let mut w = World::new("test", 30.0);
        w.set_weapon(Id::Daggers);
        w.set_open_hand(open);
        let i = mob_with(&mut w, vec3(0.0, 0.0, -1.3), 2000);
        w.enemies[i].boss = boss;
        w.enemies[i].facing = 0.0; // 방랑자 쪽을 본다(앞).
        idle(&mut w);
        // 옆으로 구르고(그 자리로 돌아오게 반대로 한 번 더) 선 뒤 앞에서 친다.
        for dir in [1.0, -1.0] {
            press(
                &t,
                &mut w,
                Actions {
                    dodge: true,
                    move_dir: vec2(dir, 0.0),
                    ..Default::default()
                },
                (t[P::DodgeTime] / TICK_DT) as usize + 2,
            );
        }
        w.player.pos = vec3(0.0, 0.0, 0.0);
        w.enemies[i].pos = vec3(0.0, 0.0, -1.3);
        w.enemies[i].facing = 0.0;
        let hp0 = w.enemies[i].hp;
        press(
            &t,
            &mut w,
            Actions {
                attack: true,
                ..Default::default()
            },
            20,
        );
        let first = hp0 - w.enemies[i].hp;
        let hp1 = w.enemies[i].hp;
        w.enemies[i].facing = 0.0;
        press(
            &t,
            &mut w,
            Actions {
                attack: true,
                ..Default::default()
            },
            20,
        );
        (first, hp1 - w.enemies[i].hp)
    };
    let (plain, shadow) = (run(false, false), run(true, false));
    assert!(plain.0 > 0);
    assert_eq!(
        shadow.0,
        crate::skills::scaled(plain.0, weapons::ART_SNEAK),
        "the first strike is a sneak attack"
    );
    assert_eq!(shadow.1, plain.1, "only the first");
    let boss = run(true, true);
    assert_eq!(
        boss.0 == crate::skills::scaled(run(false, true).0, weapons::ART_SNEAK),
        crate::world::SHADOW_STEP_BOSS,
        "a warden only from behind unless the rule says otherwise"
    );
}

/// 0276 창 "자루를 길게": 평타 · 대시 공격의 닿는 거리 ×, 내달려 꿰뚫기가 한 번 더 이어진다(새 스윙 — 같은 적을 다시 꿰뚫는다 · 셋째는 없다).
#[test]
fn a_long_grip_reaches_farther_and_runs_the_lunge_twice() {
    let t = Tuning::default();
    let w = bare_world(Id::Spear);
    let plain = Id::Spear.def();
    for i in 0..plain.combo.len() as u8 {
        let r = w.hit_of(i).unwrap().reach / plain.hit(i).unwrap().reach;
        assert!((r - weapons::BARE_SPEAR_REACH).abs() < 1e-4);
    }
    let run = |open: bool| {
        let mut w = World::new("test", 60.0);
        w.set_weapon(Id::Spear);
        w.set_open_hand(open);
        mob_with(&mut w, vec3(0.0, 0.0, -2.2), 4000);
        let i = mob_with(&mut w, vec3(0.0, 0.0, -9.0), 4000);
        idle(&mut w);
        let p0 = w.player.pos;
        let charge = ((t[P::HeavyCharge] * 1.5 + 0.6) / TICK_DT) as usize;
        let (st, _) = hold_attack(&mut w, &t, charge, 120);
        let heavies: std::collections::BTreeSet<u32> = st
            .iter()
            .filter_map(|s| match s {
                PState::Heavy { swing, .. } => Some(*swing),
                _ => None,
            })
            .collect();
        (
            heavies.len(),
            (w.player.pos - p0).length(),
            4000 - w.enemies[i].hp,
        )
    };
    let (plain, long) = (run(false), run(true));
    assert_eq!(plain.0, 1);
    assert_eq!(long.0, 2, "once more, not more");
    assert!(
        long.1 > plain.1 * 1.6,
        "runs farther: {} vs {}",
        long.1,
        plain.1
    );
    assert_eq!(plain.2, 0, "one thrust does not reach the far one");
    assert!(long.2 > 0, "the second thrust pierces on to it");
}

/// 0276 지팡이 "두 손으로 엮는다": RB 칸이 하나 더(넷), 모은 탄을 빨리 모은다.
#[test]
fn woven_hands_hold_one_more_charge_and_gather_faster() {
    let t = Tuning::default();
    let mut w = bare_world(Id::Staff);
    assert_eq!(w.cast_charges(), weapons::BARE_STAFF_CHARGES);
    assert!(w.cast_full(&t) < t[P::CastCharge] * 0.7);
    w.player.ranged.mana = 0.0;
    wait(&mut w, &t, t[P::CastCooldown] * 6.0);
    assert!((w.player.ranged.mana - weapons::BARE_STAFF_CHARGES).abs() < 1e-3);
    w.set_open_hand(false);
    wait(&mut w, &t, 0.1);
    assert!(w.player.ranged.mana <= CAST_CHARGES + 1.0);
    assert_eq!(w.cast_charges(), CAST_CHARGES);
    assert_eq!(w.cast_full(&t), t[P::CastCharge]);
}

/// 0276 활 "가벼운 걸음": 당긴 채 걷는 빠르기가 0.6 → 0.85, 끝까지 당긴 화살의 오금이 길다.
#[test]
fn a_light_step_walks_with_the_string_drawn_and_lames_longer() {
    let t = Tuning::default();
    let run = |open: bool| {
        let mut w = World::new("test", 60.0);
        w.set_weapon(Id::Bow);
        w.set_open_hand(open);
        w.view_yaw = Some(0.0);
        let i = mob_with(&mut w, vec3(0.0, 0.0, -8.0), 4000);
        idle(&mut w);
        w.player.facing = std::f32::consts::PI;
        w.step(
            &Actions {
                cast: true,
                cast_held: true,
                ..Default::default()
            },
            &t,
        );
        let x0 = w.player.pos.x;
        for _ in 0..50 {
            w.step(
                &Actions {
                    cast_held: true,
                    move_dir: vec2(1.0, 0.0),
                    ..Default::default()
                },
                &t,
            );
        }
        let walked = w.player.pos.x - x0;
        // 다 당겼다 — 적 쪽으로 놓는다.
        w.player.pos = vec3(0.0, 0.0, 0.0);
        w.player.facing = std::f32::consts::PI;
        w.step(
            &Actions {
                aim: vec2(0.0, 1.0),
                ..Default::default()
            },
            &t,
        );
        wait(&mut w, &t, 1.0);
        (walked, w.enemies[i].marks.hamstring)
    };
    let (plain, light) = (run(false), run(true));
    let k = light.0 / plain.0;
    assert!(
        (k - weapons::BARE_BOW_WALK / weapons::BOW_DRAW_WALK).abs() < 0.1,
        "{} vs {}",
        light.0,
        plain.0
    );
    assert!(plain.1 > 0.0, "the full draw lames");
    assert!(light.1 > plain.1 + 0.8, "{} vs {}", light.1, plain.1);
}

/// 0276 쇠뇌 "장치를 하나 더": 빈 손이면 공격 버튼(수류탄 자리)이 쇠 덫을 놓는다 — 수류탄 하나를 쓰고, 적이 들어서면 한 번 물어
/// 묶는다(잡몹은 속박). 놓인 덫은 `SNARE_MAX`개까지.
#[test]
fn an_empty_handed_crossbow_sets_iron_snares_that_bind() {
    let t = Tuning::default();
    let mut w = bare_world(Id::Crossbow);
    w.view_yaw = Some(0.0);
    let g0 = w.player.ranged.grenades;
    let ev = press(
        &t,
        &mut w,
        Actions {
            attack: true,
            ..Default::default()
        },
        (t[P::ThrowTime] / TICK_DT) as usize + 2,
    );
    assert!(ev.iter().any(|e| matches!(e, Event::SnareSet { .. })));
    assert!(w.bombs.is_empty(), "no grenade");
    assert!(w.player.ranged.grenades < g0);
    let snare = *w
        .skill_fields
        .iter()
        .find(|f| f.iron())
        .expect("a snare lies");
    // 잡몹이 덫 위로 걸어 든다.
    let i = mob_with(&mut w, snare.pos + vec3(3.0, 0.0, 0.0), 500);
    idle(&mut w);
    w.enemies[i].pos = snare.pos + vec3(0.4, 0.0, 0.0);
    let ev = wait(&mut w, &t, 0.2);
    assert!(ev.iter().any(|e| matches!(e, Event::SnareSnap { .. })));
    assert!(w.enemies[i].marks.bind > 0.0, "bound");
    assert!(w.enemies[i].hp < 500);
    assert!(
        !w.skill_fields.iter().any(|f| f.iron()),
        "a snare bites once"
    );
    // 덫은 둘까지 — 셋째를 놓으면 가장 오래된 것이 사라진다(걸어 들 적은 멀리).
    w.enemies[i].pos = vec3(25.0, 0.0, 25.0);
    for k in 0..3 {
        w.player.pos = vec3(k as f32 * 4.0, 0.0, 0.0);
        w.player.ranged.grenades = 2.0;
        w.player.stamina = 100.0;
        press(
            &t,
            &mut w,
            Actions {
                attack: true,
                ..Default::default()
            },
            (t[P::ThrowTime] / TICK_DT) as usize + 2,
        );
    }
    assert_eq!(
        w.skill_fields.iter().filter(|f| f.iron()).count(),
        crate::tuning::SNARE_MAX
    );
    // 보조무기가 있으면 수류탄 그대로.
    let mut w = bare_world(Id::Crossbow);
    w.set_sub(Some(Id::Sword));
    press(
        &t,
        &mut w,
        Actions {
            attack: true,
            ..Default::default()
        },
        (t[P::ThrowStartup] / TICK_DT) as usize + 2,
    );
    assert_eq!(w.bombs.len(), 1);
}

/// 0276 방패 — 하데스처럼: RB가 방패를 던진다(종류의 RB보다 앞선다 · 칸을 쓰지 않는다). 맞힌 적에서 가까운 적으로 튕기고
/// (`SHIELD_BOUNCES`번까지) 손에 돌아온다. 날아간 동안은 막지 못하고 RB도 막히지만 **누르는 순간의 패링은 그대로**다(창 같은 틱 수).
/// 평타를 누르면 곧바로 손에 돌아온다.
#[test]
fn the_shield_flies_bounces_comes_back_and_the_parry_press_stays_while_it_is_out() {
    let t = Tuning::default();
    let mut w = World::new("test", 40.0);
    w.set_weapon(Id::Sword);
    w.set_shield(Some(shield::Id::Pilgrim));
    assert!(w.shield_throws() && w.can_block());
    let ids: Vec<usize> = [
        vec3(0.0, 0.0, -4.0),
        vec3(2.5, 0.0, -6.0),
        vec3(-2.0, 0.0, -7.5),
        vec3(0.0, 0.0, -16.0),
    ]
    .iter()
    .map(|p| mob_with(&mut w, *p, 900))
    .collect();
    idle(&mut w);
    let mana = w.player.ranged.mana;
    let mut ev = w.step(
        &Actions {
            cast: true,
            ..Default::default()
        },
        &t,
    );
    ev.extend(wait(&mut w, &t, 0.4));
    assert!(w.shield_out(), "flying");
    assert!(!w.can_block(), "no guard while it flies");
    assert!(!w.weapon_away(), "the sword stays in hand");
    assert!(
        (w.player.ranged.mana - mana).abs() < 1e-3,
        "no charge spent"
    );
    // 패링 판정 시간은 그대로.
    let window = |w: &mut World, ev: &mut Vec<Event>| {
        w.player.state = PState::Idle;
        ev.extend(w.step(
            &Actions {
                parry: true,
                parry_held: true,
                ..Default::default()
            },
            &t,
        ));
        let mut n = 0;
        for _ in 0..30 {
            if w.parry_open(&t) {
                n += 1;
            }
            ev.extend(w.step(
                &Actions {
                    parry_held: true,
                    ..Default::default()
                },
                &t,
            ));
        }
        n
    };
    assert!(w.shield_out() && !w.can_block());
    let out = window(&mut w, &mut ev);
    ev.extend(wait(&mut w, &t, 3.0));
    assert!(!w.shield_out(), "back in hand");
    assert!(ev.iter().any(|e| matches!(e, Event::JavelinBack { .. })));
    let bounces = ev
        .iter()
        .filter(|e| matches!(e, Event::ShieldBounce { .. }))
        .count();
    assert_eq!(bounces, crate::tuning::SHIELD_BOUNCES as usize);
    let hit: Vec<bool> = ids.iter().map(|&i| w.enemies[i].hp < 900).collect();
    assert_eq!(
        hit,
        [true, true, true, false],
        "three bounced, the far one not"
    );
    let mut fresh = World::new("test", 40.0);
    fresh.set_weapon(Id::Sword);
    fresh.set_shield(Some(shield::Id::Pilgrim));
    assert_eq!(
        out,
        window(&mut fresh, &mut Vec::new()),
        "the parry window is untouched"
    );
    // 평타가 부른다.
    w.player.state = PState::Idle;
    w.step(
        &Actions {
            cast: true,
            ..Default::default()
        },
        &t,
    );
    wait(&mut w, &t, t[P::CastTime] + 0.05);
    assert!(w.shield_out());
    w.step(
        &Actions {
            attack: true,
            ..Default::default()
        },
        &t,
    );
    assert!(!w.shield_out(), "called home");
    // 두 손 무기면 방패는 쉰다 — RB는 그 종류의 것.
    let mut w = World::new("test", 40.0);
    w.set_weapon(Id::Spear);
    w.set_shield(Some(shield::Id::Pilgrim));
    assert!(!w.shield_throws());
    assert!(w.rb_bolts().0.returns() && w.rb_bolts().0.look == weapons::Look::Javelin);
}

/// 0276 들이받기: 막는 동안 힘이 모이고(`RUSH_FULL`초에 가득), 놓으면 모인 만큼 멀리 · 세게 들이받는다 — 잡몹은 밀려 넘어지고,
/// 문지기는 자세만 깎이고 거기서 멈춘다. 톡 막기(덜 모임)는 그냥 내린다. 막기의 기력 셈은 그대로(막는 동안 기력이 차지 않는다).
#[test]
fn holding_the_guard_gathers_a_bash_that_goes_as_far_as_it_gathered() {
    let t = Tuning::default();
    let hold = Actions {
        parry_held: true,
        ..Default::default()
    };
    let run = |secs: f32, boss: bool| {
        let mut w = World::new("test", 60.0);
        w.set_weapon(Id::Sword);
        w.set_shield(Some(shield::Id::Pilgrim));
        let i = mob_with(&mut w, vec3(0.0, 0.0, -2.6), 2000);
        w.enemies[i].boss = boss;
        idle(&mut w);
        w.step(
            &Actions {
                parry: true,
                parry_held: true,
                ..Default::default()
            },
            &t,
        );
        for _ in 0..60 {
            if w.guarding() {
                break;
            }
            w.step(&hold, &t);
        }
        assert!(w.guarding());
        let stamina = w.player.stamina;
        for _ in 0..(secs / TICK_DT) as usize {
            w.step(&hold, &t);
        }
        assert!(w.player.stamina <= stamina, "no regen while guarding");
        let power = w.player.class.rush;
        let p0 = w.player.pos;
        let mut ev = w.step(&Actions::default(), &t);
        let rushed = matches!(w.player.state, PState::Rush { .. });
        let mut toppled = false;
        let mut poise: f32 = 0.0;
        for _ in 0..48 {
            ev.extend(w.step(&Actions::default(), &t));
            toppled |= matches!(w.enemies[i].state, EState::Stagger { .. });
            poise = poise.max(w.enemies[i].stance.poise);
        }
        let en = &w.enemies[i];
        (
            rushed,
            power,
            (w.player.pos - p0).length(),
            2000 - en.hp,
            poise,
            toppled,
            ev.iter().any(|e| matches!(e, Event::Rush { .. })),
        )
    };
    let tap = run(0.1, false);
    assert!(!tap.0 && !tap.6, "a short guard just lowers");
    let half = run(0.5, false);
    let full = run(1.4, false);
    assert!(half.0 && full.0);
    assert!((full.1 - 1.0).abs() < 1e-3);
    assert!(full.2 > half.2 + 1.0, "farther: {} vs {}", full.2, half.2);
    assert!(
        full.3 > half.3 && half.3 > 0,
        "harder: {} vs {}",
        full.3,
        half.3
    );
    assert!(half.5 && full.5, "a mob topples");
    let boss = run(1.4, true);
    assert!(
        boss.0 && boss.3 > 0 && boss.4 > 0.0,
        "a warden: hurt and poise"
    );
    assert!(!boss.5, "a warden does not topple");
    assert!(
        boss.2 < full.2,
        "stops at a warden: {} vs {}",
        boss.2,
        full.2
    );
}
