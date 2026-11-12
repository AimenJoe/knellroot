//! 성장: 장신구·레벨의 능력치, 기술 나무 칸의 동작(0107 — 칸마다 `tree.has` 한 줄).
use super::*;

#[test]
fn charms_raise_the_health_cap_and_make_hits_hurt_more() {
    use crate::charms::{def, Id};
    use crate::stats::Stats;
    let mut w = World::new("t", 20.0);
    // 단단한 껍질 + 가벼운 발: 최대 체력 +10, 받는 피해 +10.
    w.apply_stats(
        Stats::compute(&[def(Id::Shell).mods, def(Id::Feet).mods]),
        true,
    );
    assert_eq!(w.player.max_hp, crate::tuning::PLAYER_HP + 10);
    assert_eq!(w.player.hp, w.player.max_hp, "refilled");
    // 장신구를 빼면 체력은 새 최대를 넘지 않는다.
    w.apply_stats(Stats::default(), false);
    assert_eq!(w.player.hp, crate::tuning::PLAYER_HP);
    // 받는 피해: 같은 한 대가 10 대신 20.
    w.apply_stats(Stats::compute(&[def(Id::Feet).mods]), true);
    let before = w.player.hp;
    w.spawn(crate::species::Kind::Husk, vec3(0.6, 0.0, 0.0));
    let t = crate::tuning::Tuning::default();
    for _ in 0..240 {
        w.step(&Actions::default(), &t);
        if w.player.hp < before {
            break;
        }
    }
    assert_eq!(before - w.player.hp, 20);
}

/// 0104(레벨 힘): 평타 한 대가 힘 5단이면 피해 +10(옛 +1)·자세 피해 ×1.6, 힘 0단은 그대로. 기력은 최대가 늘고 쉬면 가득.
#[test]
fn might_adds_whole_damage_and_poise_and_stamina_grows_the_bar() {
    let t = Tuning::default();
    let swing = |might: u8| {
        let mut w = World::new("test", 20.0);
        let i = mob_with(&mut w, vec3(0.0, 0.0, -1.2), 990);
        idle(&mut w);
        let lv = crate::progression::Level {
            might,
            ..Default::default()
        };
        let s = crate::stats::Stats::compute(&[lv.mods()]);
        w.apply_stats(s, true);
        let tt = w.stats.tune(&t);
        w.step(
            &Actions {
                attack: true,
                ..Default::default()
            },
            &tt,
        );
        for _ in 0..40 {
            if w.enemies[i].hp < 990 {
                break;
            }
            w.step(&Actions::default(), &tt);
        }
        (990 - w.enemies[i].hp, w.enemies[i].poise)
    };
    let (d0, p0) = swing(0);
    let (d5, p5) = swing(5);
    assert_eq!(d0, 10, "the first swing lands");
    assert_eq!(d5, d0 + 10, "might 5: light hits +10");
    assert!(p0 > 0.0);
    assert!(
        (p5 / p0 - 1.4).abs() < 1e-3,
        "might 5: poise x1.4 ({p0} -> {p5})"
    );

    let mut w = World::new("test", 20.0);
    let lv = crate::progression::Level {
        stamina: 5,
        ..Default::default()
    };
    let s = crate::stats::Stats::compute(&[lv.mods()]);
    w.apply_stats(s, true);
    let base = crate::tuning::SPECS[crate::tuning::idx(P::StaminaMax)].def;
    assert_eq!(w.player.stamina, base + 40.0, "rest fills the longer bar");
    let tt = w.stats.tune(&t);
    assert_eq!(tt[P::StaminaMax], t[P::StaminaMax] + 40.0);
    assert!((tt[P::StaminaRegen] - t[P::StaminaRegen] * 1.12).abs() < 1e-3);
    // 판정 창은 레벨이 건드리지 않는다(0095 원칙 2).
    assert_eq!(tt[P::ParryWindow], t[P::ParryWindow]);
}

// ── 나무 칸 동작(0107, v0.4.0 G5) ─────────────────────────────────────────────

/// 이 칸들(과 그 앞 칸들)을 연 나무.
fn tree_with(cells: &[Cell]) -> crate::progression::Tree {
    let mut tr = crate::progression::Tree::default();
    for (b, row) in crate::progression::TREE.iter().enumerate() {
        for (d, def) in row.iter().enumerate() {
            if cells.contains(&def.cell) {
                tr.open[b] = tr.open[b].max(d as u8 + 1);
            }
        }
    }
    tr
}

/// 앞(-z)의 적을 이 스윙으로 친다 — 활성 첫 틱의 `apply_strike` 한 번.
fn strike_once(w: &mut World, t: &Tuning, state: PState) -> Vec<Event> {
    w.player.facing = std::f32::consts::PI;
    w.player.state = state;
    let mut ev = Vec::new();
    w.apply_strike(t, &mut ev);
    ev
}

fn light_at_active(t: &Tuning, w: &World, swing: u32, hit: u8) -> PState {
    let (s0, _, _) = hit_times(t, w.player.weapon.def(), hit);
    PState::Attack {
        t: s0 + 1e-4,
        swing,
        dir: vec3(0.0, 0.0, -1.0),
        hit,
    }
}

fn heavy_at_active(t: &Tuning, w: &World, swing: u32, full: bool) -> PState {
    let (s0, _, _) = heavy_times(t, w.player.weapon.def());
    PState::Heavy {
        t: s0 + 1e-4,
        swing,
        dir: vec3(0.0, 0.0, -1.0),
        full,
        perfect: false,
    }
}

#[test]
fn even_breath_gives_a_perfect_heavy_its_stamina_back() {
    let t = Tuning::default();
    let left = |cells: &[Cell]| {
        let mut w = World::new("test", 20.0);
        w.tree = tree_with(cells);
        let h = w.player.weapon.def().heavy.unwrap();
        let full = t[P::HeavyCharge] * h.charge;
        w.player.state = PState::Charging { t: full + 0.05 };
        w.player.stamina = 50.0;
        w.step(&Actions::default(), &t);
        assert!(matches!(
            w.player.state,
            PState::Heavy { perfect: true, .. }
        ));
        w.player.stamina
    };
    assert!((left(&[]) - (50.0 - t[P::HeavyStamina])).abs() < 1e-3);
    assert!((left(&[Cell::EvenBreath]) - 50.0).abs() < 1e-3);
}

#[test]
fn flowing_beat_sends_a_shock_on_every_third_beat_hit() {
    let t = Tuning::default();
    let poise = |cells: &[Cell]| {
        let mut w = World::new("test", 20.0);
        w.tree = tree_with(cells);
        let i = mob_with(&mut w, vec3(0.0, 0.0, 1.0), 990);
        idle(&mut w);
        let pl = &mut w.player;
        (pl.beat, pl.since_swing) = (0.5, 0.5);
        (pl.combo_left, pl.combo_next, pl.beat_chain) = (0.3, 1, 2);
        w.step(
            &Actions {
                attack: true,
                ..Default::default()
            },
            &t,
        );
        assert_eq!(w.player.beat_chain, 3, "the third beat in a row");
        w.enemies[i].poise
    };
    assert_eq!(
        poise(&[]),
        0.0,
        "no shock without the cell (the swing has not landed yet)"
    );
    assert!(
        poise(&[Cell::FlowingBeat]) > 0.0,
        "the shock reaches the foe behind"
    );
}

#[test]
fn closing_blow_keeps_the_combo_after_the_last_hit_and_hits_harder_on_beat() {
    let t = Tuning::default();
    for cell in [false, true] {
        let mut w = World::new("test", 20.0);
        if cell {
            w.tree = tree_with(&[Cell::ClosingBlow]);
        }
        let wpn = w.player.weapon.def();
        let last = (0..8u8).find(|&h| wpn.after(h).is_none()).unwrap();
        let (s0, a0, r0) = hit_times(&t, wpn, last);
        w.player.state = PState::Attack {
            t: s0 + a0 + r0 - 1e-3,
            swing: 1,
            dir: vec3(0.0, 0.0, -1.0),
            hit: last,
        };
        w.step(&Actions::default(), &t);
        assert_eq!(w.player.state, PState::Idle);
        assert_eq!(
            w.player.combo_left > 0.0,
            cell,
            "the beat still counts after the last hit"
        );
        assert_eq!(w.player.closing_ready, cell);
    }
    // 맺음타로 이은 박자 타는 자세 ×1.5.
    let poise = |cell: bool| {
        let mut w = World::new("test", 20.0);
        if cell {
            w.tree = tree_with(&[Cell::ClosingBlow]);
        }
        let i = mob_with(&mut w, vec3(0.0, 0.0, -1.2), 990);
        idle(&mut w);
        w.player.swing_beat = true;
        w.player.closing_swing = 4;
        let st = light_at_active(&t, &w, 4, 0);
        strike_once(&mut w, &t, st);
        w.enemies[i].poise
    };
    assert!((poise(true) / poise(false) - cell::CLOSING_POISE).abs() < 1e-3);
}

#[test]
fn gap_beat_doubles_poise_on_a_foe_recovering_from_its_swing() {
    let t = Tuning::default();
    let poise = |cell: bool, beat: bool| {
        let mut w = World::new("test", 20.0);
        if cell {
            w.tree = tree_with(&[Cell::GapBeat]);
        }
        let i = mob_with(&mut w, vec3(0.0, 0.0, -1.2), 990);
        w.enemies[i].state = EState::Recover { t: 0.0 };
        w.player.swing_beat = beat;
        let st = light_at_active(&t, &w, 3, 0);
        strike_once(&mut w, &t, st);
        w.enemies[i].poise
    };
    assert!((poise(true, true) / poise(false, true) - cell::GAP_POISE).abs() < 1e-3);
    assert!(
        (poise(true, false) - poise(false, false)).abs() < 1e-4,
        "only beat hits"
    );
}

#[test]
fn final_measure_primes_the_next_heavy_after_five_beats_and_a_beat_finish() {
    let t = Tuning::default();
    let mut w = World::new("test", 20.0);
    w.tree = tree_with(&[Cell::FinalMeasure]);
    let wpn = w.player.weapon.def();
    let last = (0..8u8).find(|&h| wpn.after(h).is_none()).unwrap();
    let pl = &mut w.player;
    (pl.beat, pl.since_swing) = (0.5, 0.5);
    (pl.combo_left, pl.combo_next, pl.beat_chain) = (0.3, last, 4);
    w.step(
        &Actions {
            attack: true,
            ..Default::default()
        },
        &t,
    );
    assert!(w.player.primed_heavy, "five beats and the last hit on beat");
    // 다음 모으기는 다 모인 채로 시작한다.
    let (s0, a0, _) = hit_times(&t, wpn, 0);
    w.player.state = PState::Attack {
        t: s0 + a0 + 0.01,
        swing: 9,
        dir: vec3(0.0, 0.0, -1.0),
        hit: 0,
    };
    w.player.attack_hold = t[P::HeavyHoldStart] + 0.1;
    w.step(
        &Actions {
            attack_held: true,
            ..Default::default()
        },
        &t,
    );
    let full = t[P::HeavyCharge] * wpn.heavy.unwrap().charge;
    assert!(
        matches!(w.player.state, PState::Charging { t: c } if c >= full - 1e-4),
        "{:?}",
        w.player.state
    );
    assert!(!w.player.primed_heavy, "used once");
}

#[test]
fn catch_makes_a_parry_fill_the_poise_bar() {
    let t = Tuning::default();
    let after_parry = |cell: bool| {
        let mut w = mob_swinging_in_front(&t);
        if cell {
            w.tree = tree_with(&[Cell::Catch]);
        }
        w.player.state = PState::Parry { t: 0.0 };
        for _ in 0..60 {
            if w.step(&Actions::default(), &t)
                .iter()
                .any(|e| matches!(e, Event::Parry { .. }))
            {
                break;
            }
        }
        (w.enemies[0].poise, poise_max(&w.enemies[0], &t))
    };
    assert_eq!(after_parry(false).0, 0.0);
    let (p, max) = after_parry(true);
    assert!((p - max * cell::CATCH_POISE).abs() < 1e-3, "{p} of {max}");
}

#[test]
fn turn_it_back_staggers_foes_around_a_riposte_that_lands() {
    let t = Tuning::default();
    let side_staggered = |cell: bool| {
        let mut w = World::new("test", 20.0);
        if cell {
            w.tree = tree_with(&[Cell::TurnItBack]);
        }
        mob_with(&mut w, vec3(0.0, 0.0, -1.2), 990);
        let j = mob_with(&mut w, vec3(1.6, 0.0, 0.6), 990);
        idle(&mut w);
        w.player.riposte_swing = 5;
        let st = heavy_at_active(&t, &w, 5, true);
        strike_once(&mut w, &t, st);
        matches!(w.enemies[j].state, EState::Stagger { .. })
    };
    assert!(!side_staggered(false));
    assert!(side_staggered(true));
}

#[test]
fn gathered_riposte_breaks_poise_when_the_charged_parrys_heavy_lands() {
    let t = Tuning::default();
    let broke = |cell: bool| {
        let mut w = World::new("test", 20.0);
        if cell {
            w.tree = tree_with(&[Cell::GatheredRiposte]);
        }
        mob_with(&mut w, vec3(0.0, 0.0, -1.2), 990);
        idle(&mut w);
        w.player.gathered_swing = 7;
        let st = heavy_at_active(&t, &w, 7, true);
        strike_once(&mut w, &t, st)
            .iter()
            .any(|e| matches!(e, Event::Break { .. }))
    };
    assert!(!broke(false), "one heavy does not break a 99-hp foe");
    assert!(broke(true));
}

#[test]
fn open_breach_restarts_the_break_with_a_full_heavy() {
    let t = Tuning::default();
    let broken_after = |cell: bool| {
        let mut w = World::new("test", 20.0);
        if cell {
            w.tree = tree_with(&[Cell::OpenBreach]);
        }
        let i = mob_with(&mut w, vec3(0.0, 0.0, -1.2), 990);
        w.enemies[i].broken = 0.1;
        w.enemies[i].poise_lock = 5.0;
        let st = heavy_at_active(&t, &w, 8, true);
        strike_once(&mut w, &t, st);
        w.enemies[i].broken
    };
    assert!((broken_after(false) - 0.1).abs() < 1e-4);
    assert!((broken_after(true) - t[P::PoiseBreak]).abs() < 1e-4);
}

#[test]
fn one_who_stands_blocks_a_root_wave_while_three_parries_are_chained() {
    let t = Tuning::default();
    let hurt = |cell: bool| {
        let mut w = world_with_boss();
        if cell {
            w.tree = tree_with(&[Cell::OneWhoStands]);
        }
        let en = &mut w.enemies[0];
        en.pos = vec3(0.0, 0.0, -2.0);
        en.atk = Atk::Wave;
        en.state = EState::Wave { t: 0.0 };
        en.landed = false;
        w.player.facing = std::f32::consts::PI;
        let mut hurt = false;
        for _ in 0..60 {
            w.player.parry_chain_left = 0.5;
            w.player.parry_chain_n = cell::STAND_CHAIN;
            hurt |= w.step(&Actions::default(), &t).contains(&Event::PlayerHurt);
        }
        hurt
    };
    assert!(hurt(false), "the wave still cannot be parried");
    assert!(!hurt(true), "but a long chain stands against it");
}

#[test]
fn unbroken_stride_makes_running_cheaper() {
    let t = Tuning::default();
    let spent = |cell: bool| {
        let mut w = World::new("test", 20.0);
        if cell {
            w.tree = tree_with(&[Cell::UnbrokenStride]);
        }
        w.player.running = true;
        w.player.stamina = 100.0;
        let a = Actions {
            dodge_held: true,
            move_dir: vec2(0.0, 1.0),
            ..Default::default()
        };
        for _ in 0..30 {
            w.step(&a, &t);
        }
        100.0 - w.player.stamina
    };
    assert!((spent(true) / spent(false) - cell::STRIDE_RUN).abs() < 0.02);
}

#[test]
fn sidestep_marks_a_dodge_that_ends_behind_a_foe_and_doubles_that_dash_attacks_poise() {
    let t = Tuning::default();
    let flank = |behind: bool| {
        let mut w = World::new("test", 20.0);
        w.tree = tree_with(&[Cell::Sidestep]);
        let i = mob_with(&mut w, vec3(0.0, 0.0, -1.5), 990);
        idle(&mut w);
        // 적이 +z(플레이어 쪽)를 보면 플레이어는 앞, -z를 보면 등 뒤.
        w.enemies[i].facing = if behind { std::f32::consts::PI } else { 0.0 };
        w.player.state = PState::Dodge {
            t: t[P::DodgeTime] - 1e-3,
            dir: vec3(0.0, 0.0, 1.0),
        };
        w.step(&Actions::default(), &t);
        w.player.flank
    };
    assert!(flank(true));
    assert!(!flank(false));
    let poise = |cell: bool| {
        let mut w = World::new("test", 20.0);
        if cell {
            w.tree = tree_with(&[Cell::Sidestep]);
        }
        let i = mob_with(&mut w, vec3(0.0, 0.0, -1.2), 990);
        idle(&mut w);
        w.player.flank_swing = 6;
        let st = light_at_active(&t, &w, 6, weapons::DASH);
        strike_once(&mut w, &t, st);
        w.enemies[i].poise
    };
    assert!((poise(true) / poise(false) - cell::SIDESTEP_POISE).abs() < 1e-3);
}

#[test]
fn tilting_ground_pushes_spilled_foes_farther_out() {
    let t = Tuning::default();
    let out = |cell: bool| {
        let mut w = World::new("test", 20.0);
        if cell {
            w.tree = tree_with(&[Cell::TiltingGround]);
        }
        let i = mob_with(&mut w, vec3(0.0, 0.0, -3.0), 990);
        turn_until_spill(&mut w, true, &t);
        let en = &w.enemies[i];
        let d = vec3(en.pos.x, 0.0, en.pos.z).normalize_or_zero();
        en.knock.dot(d)
    };
    assert!(
        // 그 틱의 감속이 조금 먹는다 — 더한 밀림의 절반 넘게 남는다.
        out(true) - out(false) > cell::TILT_PUSH * KNOCK_DECAY * 0.5,
        "{} vs {}",
        out(true),
        out(false)
    );
}

#[test]
fn landing_beat_rings_again_farther_out_a_moment_after_a_plunge() {
    let t = Tuning::default();
    let hp = |cell: bool| {
        let mut w = World::new("test", 20.0);
        if cell {
            w.tree = tree_with(&[Cell::LandingBeat]);
        }
        let r = t[P::PlungeRange] * 1.4;
        let i = mob_with(&mut w, vec3(0.0, 0.0, -1.0), 990);
        w.enemies[i].pos.z -= r + w.enemies[i].radius - 1.0;
        idle(&mut w);
        w.plunge_now = true;
        for _ in 0..40 {
            w.step(&Actions::default(), &t);
        }
        w.enemies[i].hp
    };
    assert_eq!(hp(false), 990, "outside the first ring");
    assert_eq!(
        hp(true),
        990 - t[P::PlungeDamage] as i32,
        "the late outer ring reaches it"
    );
}

#[test]
fn path_opening_step_carries_a_dash_attack_past_the_foe() {
    let t = Tuning::default();
    let z = |cell: bool| {
        let mut w = World::new("test", 20.0);
        if cell {
            w.tree = tree_with(&[Cell::PathOpeningStep]);
        }
        mob_with(&mut w, vec3(0.0, 0.0, -1.2), 990);
        idle(&mut w);
        let st = light_at_active(&t, &w, 2, weapons::DASH);
        strike_once(&mut w, &t, st);
        w.player.pos.z
    };
    assert!(z(false).abs() < 1e-4);
    assert!(z(true) < -1.2 - 0.3, "now behind the foe: {}", z(true));
}

/// 0108: 맺음타는 실제 누름 길로도 걸린다 — 마지막 타 중 박자에 누르면 후딜레이를 끊고(캔슬) 첫 타가 박자 타로, 그 타가 맺음타.
/// 박자 부채꼴도 마지막 타에 뜬다(칸이 없으면 뜨지 않는다).
#[test]
fn closing_blow_works_through_the_cancel_and_shows_the_beat_fan_on_the_last_hit() {
    let t = Tuning::default();
    for cell in [false, true] {
        let mut w = World::new("test", 20.0);
        if cell {
            w.tree = tree_with(&[Cell::ClosingBlow]);
        }
        let wpn = w.player.weapon.def();
        let last = (0..8u8).find(|&h| wpn.after(h).is_none()).unwrap();
        let (s0, a0, r0) = hit_times(&t, wpn, last);
        let total = s0 + a0 + r0;
        let pl = &mut w.player;
        pl.state = PState::Attack {
            t: total - 0.01,
            swing: 3,
            dir: vec3(0.0, 0.0, -1.0),
            hit: last,
        };
        (pl.swings, pl.swing_hit, pl.beat, pl.since_swing) = (3, last, total, total - 0.01);
        assert_eq!(
            w.beat_cue(&t).is_some(),
            cell,
            "the beat fan on the last hit"
        );
        w.step(
            &Actions {
                attack: true,
                ..Default::default()
            },
            &t,
        );
        let pl = &w.player;
        assert!(
            matches!(pl.state, PState::Attack { hit: 0, .. }),
            "{:?}",
            pl.state
        );
        assert_eq!(pl.swing_beat, cell, "a beat swing only with the cell");
        assert_eq!(
            pl.closing_swing == pl.swings,
            cell,
            "and that swing is the closing blow"
        );
    }
}

/// 0108: 비껴서기 표지(민트 고리)는 대시 공격 창이 닫히면 꺼진다.
#[test]
fn the_sidestep_mark_fades_with_the_dash_window() {
    let t = Tuning::default();
    let mut w = World::new("test", 20.0);
    w.tree = tree_with(&[Cell::Sidestep]);
    w.player.flank = true;
    w.player.dash_left = DASH_ATTACK_LATE;
    w.step(&Actions::default(), &t);
    assert!(w.player.flank, "still open");
    for _ in 0..20 {
        w.step(&Actions::default(), &t);
    }
    assert!(!w.player.flank, "gone with the dash window");
}
