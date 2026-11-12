//! 맞음: 맞은 순간의 멈춤(0143), 경직과 경직 이음(0099), 체력·피해 눈금 10(0158 N0 — 옛 대 수와 맞춰 본다).
use super::*;

// ── 눈금 옮기기(0158 N0) ─────────────────────────────────────────────────────────────────────────────

/// 옮기기 전의 수(작은 정수 — 고치지 않는다): 방랑자 체력, 한 대(등급 3부터 둘), 치명타 덤, 깨진 동안의 배율.
const OLD_PLAYER_HP: i32 = 5;
const OLD_BLOW: i32 = 1;
const OLD_BLOW_HEAVY_TIER: i32 = 2;
const OLD_CRIT: i32 = 2;
const OLD_BREAK_X: f32 = 1.5;

/// 0158 N0 판정 기준: 어느 종에게 맞아도(등급 1~3, 받는 피해 + 장신구까지) 몇 대에 쓰러지는가가 옮기기 전과 같다.
/// 종마다 실제로 한 대 맞아 잃은 체력을 재고, 옛 수(체력 5 · 한 대 1 · 등급 3부터 2 · 가벼운 발 +1)와 대 수를 견준다.
#[test]
fn hits_to_die_to_every_species_are_what_they_were_before_the_scale_move() {
    use crate::charms::{def, Id};
    use crate::stats::Stats;
    let t = Tuning::default();
    let mut cases = 0;
    for d in crate::species::ALL {
        for tier in [1u8, 2, crate::tuning::TIER_HEAVY_HITS] {
            for feet in [false, true] {
                let mut w = World::new("test", 30.0);
                if feet {
                    w.apply_stats(Stats::compute(&[def(Id::Feet).mods]), true);
                }
                let full = w.player.hp;
                assert_eq!(full, OLD_PLAYER_HP * HP_UNIT);
                let i = w.spawn(d.kind, w.player.pos + vec3(0.0, 0.0, -1.0));
                w.enemies[i].set_tier(tier);
                w.enemies[i].facing = 0.0; // +z = 플레이어 쪽
                                           // 꼬투리는 휘두르지 않고 심는다 — 심어 둔 원판이 발밑에서 터지는 틱.
                if d.behavior == crate::species::Behavior::Planter {
                    w.enemies[i].atk = Atk::Plant;
                    w.enemies[i].plant_at = Some(w.player.pos);
                }
                w.enemies[i].state = EState::Swing { t: 0.0 };
                let ev = w.step(&Actions::default(), &t);
                let tag = format!("{:?} tier {tier} feet {feet}", d.kind);
                assert!(ev.contains(&Event::PlayerHurt), "{tag}: the blow lands");
                let lost = full - w.player.hp;
                // 보스는 등급을 받지 않는다(늘 한 대 = 한 칸).
                let old = if !d.boss && tier >= crate::tuning::TIER_HEAVY_HITS {
                    OLD_BLOW_HEAVY_TIER
                } else {
                    OLD_BLOW
                } + feet as i32;
                assert_eq!(lost, old * HP_UNIT, "{tag}");
                assert_eq!(
                    w.player.hurt_blow, lost,
                    "{tag}: the HUD reads the same loss"
                );
                assert_eq!(
                    (full + lost - 1) / lost,
                    (OLD_PLAYER_HP + old - 1) / old,
                    "{tag}: hits to die"
                );
                cases += 1;
            }
        }
    }
    assert_eq!(cases, crate::species::ALL.len() * 6);
    // 등잔 한 모금·생명 한 단도 옛 칸 수 그대로: 한 모금 = 두 대 몫, 생명 한 단 = 한 대 몫.
    assert_eq!(t[P::FlaskHeal] as i32, 2 * OLD_BLOW * HP_UNIT);
    let life = crate::progression::Level {
        life: 1,
        ..Default::default()
    };
    assert_eq!(
        Stats::compute(&[life.mods()]).max_hp,
        (OLD_PLAYER_HP + 1) * HP_UNIT
    );
}

/// 0158 N0: 방랑자가 주는 피해의 셈(`hurt_by` — 치명타 덤 · 열린 틈 덤 · 깨진 동안 ×1.5 올림)은 옛 작은 정수로 센 결과의
/// 꼭 열 배다. 옛 피해 1~9 × 치명타 × 깨짐 × 열린 틈 덤 0·1을 다 돈다(올림이 걸리는 홀수도).
#[test]
fn every_damage_formula_gives_ten_times_what_the_old_small_numbers_gave() {
    let t = Tuning::default();
    assert_eq!(t[P::ParryCrit] as i32, OLD_CRIT * HP_UNIT);
    assert_eq!(t[P::PoiseBreakDamage], OLD_BREAK_X);
    let mut w = World::new("test", 30.0);
    let i = mob_with(&mut w, vec3(0.0, 0.0, -3.0), 990);
    for base in 1..=9 {
        for crit in [false, true] {
            for broken in [false, true] {
                for opened in [0, 1] {
                    let open = crit || broken;
                    let d = base + if crit { OLD_CRIT } else { 0 } + if open { opened } else { 0 };
                    let old = if broken {
                        (d as f32 * OLD_BREAK_X).ceil() as i32
                    } else {
                        d
                    };
                    w.enemies[i].broken = if broken { 1.0 } else { 0.0 };
                    let new = hurt_by(&w.enemies[i], base * HP_UNIT, crit, opened * HP_UNIT, &t);
                    assert_eq!(
                        new,
                        old * HP_UNIT,
                        "base {base} crit {crit} broken {broken} opened {opened}"
                    );
                }
            }
        }
    }
    // 자세 막대는 옮기지 않았다: 체력 ×10이어도 옛 칸 수로 센다(잔재 = 옛 세 칸).
    let mut w = World::new("test", 30.0);
    let h = w.spawn(Kind::Husk, vec3(0.0, 0.0, -3.0));
    assert_eq!(
        poise_max(&w.enemies[h], &t),
        t[P::PoiseBase] + 3.0 * t[P::PoisePerHp]
    );
}

/// 0143 "맞았는지 잘 모르겠음": 맞은 틱에 세계가 멈춘다 — 때릴 때(`HIT_HITSTOP`)보다 길게, HUD가 읽는 "맞은 뒤 지난 시간"은
/// 멈춘 동안 0에 머문다(붉은 테·흰 몸이 멈춘 화면과 함께 남는다). 경직 중 또 맞으면 멈춤도 반(떼 속에서 화면이 얼어붙지 않게).
#[test]
fn being_hit_freezes_the_world_longer_than_landing_a_hit_and_less_when_chained() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    let hp = w.player.hp;
    hit_by_swing(&mut w, &t);
    let first = w.hitstop;
    assert!(first > t[P::Hitstop] * HIT_HITSTOP, "{first}");
    assert!(
        (0.1..=0.2).contains(&first),
        "six to twelve frames: {first}"
    );
    assert_eq!(
        (w.player.since_hurt, w.player.hurt_blow),
        (0.0, hp - w.player.hp)
    );
    // 멈춘 동안: 시계가 가지 않는다 — 맞은 뒤 지난 시간도 0.
    w.step(&Actions::default(), &t);
    assert!(w.hitstop > 0.0 && w.player.since_hurt == 0.0);
    while w.hitstop > 0.0 {
        w.step(&Actions::default(), &t);
    }
    // 풀리면 흐른다.
    for _ in 0..3 {
        w.step(&Actions::default(), &t);
    }
    assert!(w.player.since_hurt > 0.0);
    // 무적이 끝나고 아직 경직 중일 때 또 맞는다 → 멈춤도 반.
    while w.player.hurt > 0.0 {
        w.step(&Actions::default(), &t);
    }
    assert!(matches!(w.player.state, PState::Hurt { .. }));
    hit_by_swing(&mut w, &t);
    assert!(
        (w.hitstop - first * 0.5).abs() < 1e-4,
        "{} vs {first}",
        w.hitstop
    );
    assert_eq!(w.player.since_hurt, 0.0);
}

/// 맞으면 잠깐 경직(못 움직임) — 경직 중 또 맞으면 반씩 짧아지고, 이음 창(0099)까지 지난 뒤 맞으면 다시 처음 길이(0018).
#[test]
fn a_hit_stuns_briefly_and_hits_during_the_stun_halve_it() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    let ev = hit_by_swing(&mut w, &t);
    assert!(ev.contains(&Event::PlayerHurt));
    let PState::Hurt { dur: d0, .. } = w.player.state else {
        panic!("{:?}", w.player.state)
    };
    assert!((d0 - t[P::HitStun]).abs() < 1e-4);
    // 경직 중엔 걷지 못한다(밀려난 것만 줄어든다).
    let before = w.player.pos;
    for _ in 0..3 {
        w.step(
            &Actions {
                move_dir: vec2(1.0, 0.0),
                ..Default::default()
            },
            &t,
        );
    }
    assert!((w.player.pos.x - before.x).abs() < 1e-3);
    // 무적이 끝나고 아직 경직 중일 때 또 맞는다 → 반.
    while w.player.hurt > 0.0 {
        w.step(&Actions::default(), &t);
    }
    assert!(matches!(w.player.state, PState::Hurt { .. }));
    hit_by_swing(&mut w, &t);
    let PState::Hurt { dur: d1, .. } = w.player.state else {
        panic!()
    };
    assert!((d1 - d0 * 0.5).abs() < 1e-4, "{d1} vs {d0}");
    // 풀린 뒤 맞으면 다시 처음 길이.
    for _ in 0..60 {
        w.step(&Actions::default(), &t);
    }
    assert_eq!(w.player.state, PState::Idle);
    hit_by_swing(&mut w, &t);
    let PState::Hurt { dur: d2, .. } = w.player.state else {
        panic!()
    };
    assert!((d2 - d0).abs() < 1e-4);
}

/// 0099(사장 "다구리 맞더라도 틈이 생겨서 도망갈 수 있게"): 경직이 풀린 뒤에도 `STUN_CHAIN_GRACE` 안에 또 맞으면 경직이 그전의 반,
/// 창이 지나면 다시 처음 길이. 일정한 박자로 계속 맞으면 경직이 줄어 맞는 사이사이 움직일 틈이 커진다.
#[test]
fn hits_soon_after_a_stun_keep_halving_it_so_a_mob_leaves_gaps() {
    let stun = 0.35;
    assert_eq!(
        stun_chain_next(0.0, 3, stun).0,
        0,
        "window closed: from the top"
    );
    let (n, w) = stun_chain_next(0.1, 0, stun);
    assert_eq!(n, 1);
    assert!((w - (stun * 0.5 + STUN_CHAIN_GRACE)).abs() < 1e-5);
    // 0.4초마다 맞는 몰매: 맞을 때마다 경직이 짧아지고, 맞은 사이에 움직일 수 있는 시간은 늘어난다.
    let (mut n, mut w) = (0, 0.0);
    let mut free = Vec::new();
    for _ in 0..4 {
        (n, w) = stun_chain_next(w, n, stun);
        let dur = stun * 0.5f32.powi(n);
        free.push(0.4 - dur);
        w -= 0.4;
    }
    assert!(free.windows(2).all(|f| f[1] >= f[0]), "{free:?}");
    assert!(free[0] > 0.0 && free[3] > 0.3);
    // 바닥(0099 사장 "바닥을 두자"): 두 번 이은 뒤로는 처음의 1/4에서 멈춘다 — 맞은 몸의 반응은 남는다.
    assert_eq!(n, STUN_CHAIN_MAX);
    assert!((stun * 0.5f32.powi(n) - stun * 0.25).abs() < 1e-6);

    // 월드에서: 경직이 풀린 뒤(서 있음) 창 안에 맞으면 반, 창이 지나면 처음 길이.
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    hit_by_swing(&mut w, &t);
    while !matches!(w.player.state, PState::Idle) {
        w.step(&Actions::default(), &t);
    }
    assert!(w.player.stun_window > 0.0, "still inside the grace");
    hit_by_swing(&mut w, &t);
    let PState::Hurt { dur, .. } = w.player.state else {
        panic!("{:?}", w.player.state)
    };
    assert!((dur - t[P::HitStun] * 0.5).abs() < 1e-4, "{dur}");
    for _ in 0..((t[P::HitStun] + STUN_CHAIN_GRACE) / TICK_DT) as usize + 5 {
        w.step(&Actions::default(), &t);
    }
    hit_by_swing(&mut w, &t);
    let PState::Hurt { dur, .. } = w.player.state else {
        panic!()
    };
    assert!(
        (dur - t[P::HitStun]).abs() < 1e-4,
        "fresh after the grace: {dur}"
    );
}
