//! 결의 칸(0223 11절 — 타입마다 하나): 타입이 저마다 다른 자원을 돌려준다. 잔불 = 남은 불이 옮겨붙는다 · 뿌리 = 기력 ·
//! 울림 = 큰 기술 게이지 · 균열 = 스킬 대기. 칸을 안 열었으면 아무 일도 없다(같은 싸움을 칸만 달리해 견준다).
use super::*;
use crate::progression::{vein, Tree, Vein};
use crate::status::Kind as Ty;

fn veined(v: Option<Vein>) -> Tree {
    let mut t = Tree::default();
    if let Some(v) = v {
        t.open_vein(v);
    }
    t
}

/// 가만히 서 있는(쉬는) 튼튼한 잡몹 하나.
fn foe(w: &mut World, at: Vec3) -> usize {
    let i = mob_with(w, at, 500);
    w.enemies[i].state = EState::Recover { t: -1000.0 };
    i
}

fn fill(w: &mut World, i: usize, kind: Ty) {
    afflict(&mut w.enemies[i], (kind, 10.0), &mut Vec::new());
}

fn run(w: &mut World, t: &Tuning, secs: f32) {
    for _ in 0..(secs / TICK_DT).round() as usize {
        w.step(&Actions::default(), t);
    }
}

/// 평타 한 번(앞의 적에 닿을 때까지).
fn swing(w: &mut World, t: &Tuning) {
    press(
        t,
        w,
        Actions {
            attack: true,
            ..Default::default()
        },
        30,
    );
}

/// 옮겨붙는 불: 타다 쓰러진 적의 남은 겹이 가까운 적 하나에 옮는다 — 먼 적·칸을 안 연 판은 그대로. 쓰러진 몸의 불은 꺼진다.
#[test]
fn ember_vein_passes_a_fallen_foes_fire_to_the_nearest_one() {
    let t = Tuning::default();
    let after = |open: bool| {
        let mut w = World::new("test", 30.0);
        w.tree = veined(open.then_some(Vein::Ember));
        let a = foe(&mut w, vec3(6.0, 0.0, 0.0));
        let near = foe(&mut w, vec3(8.0, 0.0, 0.0));
        let far = foe(&mut w, vec3(6.0 + vein::EMBER_REACH + 3.0, 0.0, 0.0));
        fill(&mut w, a, Ty::Ember);
        w.enemies[a].ail.kindle(1, false);
        assert_eq!(w.enemies[a].ail.burn, 2);
        w.enemies[a].hp = 1;
        run(&mut w, &t, 1.2);
        assert!(!w.enemies[a].alive());
        assert_eq!(w.enemies[a].ail.burn, 0, "the body's fire is out");
        (w.enemies[near].ail.burn, w.enemies[far].ail.burn)
    };
    assert_eq!(after(false), (0, 0));
    assert_eq!(after(true), (2, 0), "both layers, to the nearest only");
}

/// 옮겨붙은 불은 제자리에 서 있던 적을 깨운다(모르는 채 서서 타 죽지 않는다).
#[test]
fn ember_vein_wakes_the_foe_the_fire_reaches() {
    let t = Tuning::default();
    let mut w = World::new("test", 60.0);
    w.tree = veined(Some(Vein::Ember));
    let a = foe(&mut w, vec3(20.0, 0.0, 20.0));
    let post = mob_with(&mut w, vec3(22.0, 0.0, 20.0), 500);
    w.enemies[post].place(0.0);
    assert_eq!(w.enemies[post].state, EState::Idle);
    fill(&mut w, a, Ty::Ember);
    w.enemies[a].state = EState::Dead;
    w.step(&Actions::default(), &t);
    assert_eq!(w.enemies[post].ail.burn, 1);
    assert_ne!(w.enemies[post].state, EState::Idle, "it noticed");
}

/// 뿌리의 숨: 묶인 적을 무기로 맞히면 기력이 조금 돌아온다 — 휘두르기 한 번에 한 번(둘을 한꺼번에 베어도), 안 묶인 적은 아니다.
#[test]
fn root_vein_returns_stamina_for_a_weapon_hit_on_a_bound_foe() {
    let t = Tuning::default();
    let after = |open: bool, bound: bool| {
        let mut w = World::new("test", 30.0);
        w.tree = veined(open.then_some(Vein::Root));
        let a = foe(&mut w, vec3(-0.5, 0.0, -1.5));
        let b = foe(&mut w, vec3(0.5, 0.0, -1.5));
        if bound {
            fill(&mut w, a, Ty::Root);
            fill(&mut w, b, Ty::Root);
            assert!(w.enemies[a].ail.bound > 0.0);
        }
        w.player.stamina = 50.0;
        swing(&mut w, &t);
        assert!(w.enemies[a].hp < 500 && w.enemies[b].hp < 500, "both cut");
        w.player.stamina
    };
    let base = after(false, true);
    assert!((after(true, true) - base - vein::ROOT_BREATH).abs() < 1e-3);
    assert!((after(true, false) - after(false, false)).abs() < 1e-3);
}

/// 금을 따라: 금 간 적을 무기로 맞히면 모든 스킬 대기가 조금 당겨진다 — 금이 없으면 아니다.
#[test]
fn rift_vein_pulls_skill_cooldowns_for_a_weapon_hit_on_a_cracked_foe() {
    let t = Tuning::default();
    let after = |open: bool, cracked: bool| {
        let mut w = World::new("test", 30.0);
        w.tree = veined(open.then_some(Vein::Rift));
        let a = foe(&mut w, vec3(0.0, 0.0, -1.5));
        if cracked {
            fill(&mut w, a, Ty::Rift);
            assert!(w.enemies[a].ail.rift > 0.0);
        }
        w.player.skill.cd = [9.0; crate::skills::N];
        swing(&mut w, &t);
        assert!(w.enemies[a].hp < 500);
        w.player.skill.cd
    };
    let (base, pulled) = (after(false, true), after(true, true));
    for (b, p) in base.iter().zip(pulled) {
        assert!((b - p - vein::RIFT_PULL).abs() < 1e-3, "{b} -> {p}");
    }
    assert_eq!(after(true, false), after(false, false));
}

/// 울림 모으기: 울림 하나가 큰 기술 게이지를 평타 한 번만큼 채운다 — 떼가 한꺼번에 울려도 한 틱에 `KNELL_TOLLS` 번까지.
#[test]
fn knell_vein_fills_the_ult_gauge_per_toll_up_to_a_cap() {
    let t = Tuning::default();
    let after = |open: bool, pack: usize| {
        let mut w = World::new("test", 30.0);
        w.tree = veined(open.then_some(Vein::Knell));
        for k in 0..pack {
            let i = foe(&mut w, vec3(8.0 + k as f32, 0.0, 0.0));
            fill(&mut w, i, Ty::Knell);
        }
        run(&mut w, &t, 0.4);
        w.player.skill.gauge
    };
    let hit = t[P::SkillGaugeHit];
    assert_eq!(after(false, 1), 0.0);
    assert!((after(true, 1) - hit).abs() < 1e-4);
    assert!(
        (after(true, 5) - hit * vein::KNELL_TOLLS as f32).abs() < 1e-4,
        "five tolling in one tick count as {}",
        vein::KNELL_TOLLS
    );
}

/// 전환 중인 문지기(0133)에 닿은 다 모은 강타: 피해가 없으니 결의 몫도 없고, 전환을 끊지도 못한다.
#[test]
fn a_heavy_on_a_boss_mid_transition_pays_no_vein_and_does_not_cut_the_rise() {
    let t = Tuning::default();
    let mut w = world_with_boss();
    w.tree = veined(Some(Vein::Rift));
    w.enemies[0].pos = vec3(0.0, 0.0, -1.5);
    w.enemies[0].ail.crack(4.0);
    w.enemies[0].state = EState::Rise { t: 0.0 };
    w.player.skill.cd = [9.0; crate::skills::N];
    let (s0, _, _) = heavy_times(&t, w.player.weapon.def());
    w.player.facing = std::f32::consts::PI;
    w.player.state = PState::Heavy {
        t: s0 + 1e-4,
        swing: 7,
        dir: vec3(0.0, 0.0, -1.0),
        full: true,
        perfect: false,
    };
    let hp = w.enemies[0].hp;
    let mut ev = Vec::new();
    w.apply_strike(&t, &mut ev);
    w.skill_books(&t, &mut ev);
    assert_eq!(w.enemies[0].hp, hp);
    assert!(w.enemies[0].rising(), "still rising");
    assert_eq!(w.player.skill.cd, [9.0; crate::skills::N]);
}

// ---- 0270 C3: 새 타입 넷의 결 ----

/// 흔들린 머리: 기절한 적을 무기로 맞히면 그 둘레의 다른 적들의 자세가 흔들린다 — 칸을 안 열었거나 기절하지 않았으면 아니다.
#[test]
fn shock_vein_rattles_the_posture_of_foes_around_a_stunned_one() {
    let t = Tuning::default();
    let after = |open: bool, stunned: bool| {
        let mut w = World::new("test", 30.0);
        w.tree = veined(open.then_some(Vein::Shock));
        let a = foe(&mut w, vec3(0.0, 0.0, -1.5));
        let b = foe(&mut w, vec3(0.0, 0.0, -4.5));
        if stunned {
            fill(&mut w, a, Ty::Shock);
            assert!(w.enemies[a].ail.stun > 0.0);
        }
        swing(&mut w, &t);
        assert!(w.enemies[a].hp < 500);
        assert_eq!(w.enemies[b].hp, 500, "out of the blade's reach");
        w.enemies[b].stance.poise
    };
    let base = after(false, true);
    assert!(after(true, true) > base + vein::SHOCK_RATTLE * 0.5);
    assert_eq!(after(true, false), after(false, false));
}

/// 흐르는 걸음: 피 흘리는 적을 무기로 맞히면 잠깐 빨리 걷는다.
#[test]
fn sharp_vein_quickens_the_stride_after_a_hit_on_a_bleeding_foe() {
    let t = Tuning::default();
    let after = |open: bool| {
        let mut w = World::new("test", 30.0);
        w.tree = veined(open.then_some(Vein::Sharp));
        let a = foe(&mut w, vec3(0.0, 0.0, -1.5));
        fill(&mut w, a, Ty::Sharp);
        assert!(w.enemies[a].ail.bleed > 0.0);
        swing(&mut w, &t);
        let stride = w.player.skill.stride;
        // 걸어 본다: 같은 초에 더 멀리.
        let p0 = w.player.pos;
        for _ in 0..20 {
            w.step(
                &Actions {
                    move_dir: vec2(1.0, 0.0),
                    ..Default::default()
                },
                &t,
            );
        }
        (stride, (w.player.pos - p0).length())
    };
    let (none, slow) = after(false);
    let (some, fast) = after(true);
    assert_eq!(none, 0.0);
    assert!(some > 0.0);
    assert!(fast > slow * 1.1, "{slow} -> {fast}");
}

/// 얼음 속 불씨: 언 적이 깨질 때 잔불이 흩어진다.
#[test]
fn frost_vein_scatters_an_ember_when_the_ice_shatters() {
    let t = Tuning::default();
    let after = |open: bool| {
        let mut w = World::new("test", 30.0);
        w.tree = veined(open.then_some(Vein::Frost));
        let a = foe(&mut w, vec3(0.0, 0.0, -1.5));
        fill(&mut w, a, Ty::Frost);
        assert!(w.enemies[a].ail.frozen > 0.0);
        swing(&mut w, &t);
        assert!(
            w.enemies[a].ail.frozen <= 0.0,
            "shattered: {} {}",
            w.enemies[a].hp,
            w.enemies[a].ail.frozen
        );
        w.drops.len()
    };
    assert_eq!(after(false), 0);
    assert_eq!(after(true), vein::FROST_EMBERS as usize);
}

/// 한 모금씩: 흡수가 걸린 적을 무기로 맞힐 때마다 등잔이 조금 찬다 — 다 차면 하나, 가득이면 넘치지 않는다.
#[test]
fn drain_vein_fills_the_lamp_sip_by_sip() {
    let t = Tuning::default();
    let after = |open: bool, sip: f32, flask: i32| {
        let mut w = World::new("test", 30.0);
        w.tree = veined(open.then_some(Vein::Drain));
        let a = foe(&mut w, vec3(0.0, 0.0, -1.5));
        fill(&mut w, a, Ty::Drain);
        assert!(w.enemies[a].ail.drain > 0.0);
        w.player.skill.sip = sip;
        w.player.flask = flask;
        swing(&mut w, &t);
        (w.player.skill.sip, w.player.flask)
    };
    let max = World::new("test", 30.0).player.flask_max;
    assert_eq!(after(false, 0.0, 0), (0.0, 0));
    let (sip, flask) = after(true, 0.0, 0);
    assert!((sip - vein::DRAIN_SIP).abs() < 1e-5 && flask == 0);
    assert_eq!(
        after(true, 1.0 - vein::DRAIN_SIP, 0),
        (0.0, 1),
        "a full sip is a draught"
    );
    let (sip, flask) = after(true, 1.0 - vein::DRAIN_SIP, max);
    assert_eq!(flask, max, "never over the brim");
    assert!(sip >= 1.0 - 1e-4, "the sip waits, full");
}
