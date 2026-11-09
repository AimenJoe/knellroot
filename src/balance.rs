//! 보스 싸움 길이 모의(0126): 실제 `World`로 종지기와 싸우는 봇을 돌려 성공률별 싸움 시간·받은 피해를 잰다.
//! 봇은 보스 상태를 읽는다(사람이 예고를 읽는 것의 대역). 공격마다 한 번 굴려 `p`면 맞는 순간에 패링(되받을 수 있는 것)·
//! 점프(울림 고리 — 고리마다 굴림)를 정확히 하고, 아니면 아무것도 안 해 맞는다(늦거나 이른 패링 = 맞음과 같게 본다).
//! 틈(보스가 예고 중이 아니거나 예고가 많이 남았을 때)에는 붙어서 평타 콤보를 넣는다. 체력 2 이하면 틈에 등잔을 마신다.
//! 돌리기: `cargo test bellkeeper_fight_length -- --ignored --nocapture`
#![cfg(test)]

use macroquad::prelude::*;

use crate::charms::Id as Charm;
use crate::input::Actions;
use crate::progression::{Level, Tree};
use crate::species::Kind;
use crate::tuning::{Tuning, PLAYER_RADIUS};
use crate::world::{Atk, EState, PState, World};

/// 작은 결정적 난수(xorshift).
struct Rng(u64);
impl Rng {
    fn f(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 40) as f32 / (1u64 << 24) as f32
    }
}

/// 도달 시점의 몸(성장·장신구·나무·숨은 물건).
#[derive(Clone)]
pub struct Build {
    pub name: &'static str,
    pub level: Level,
    pub worn: Vec<Charm>,
    pub finds: Vec<String>,
    pub tree: Tree,
    pub weapon: crate::weapons::Id,
}

/// 한 판의 결과.
#[derive(Clone, Copy, Debug, Default)]
pub struct Fight {
    pub won: bool,
    pub secs: f32,
    pub taken: i32,
    pub flasks: i32,
    pub boss_hp_left: i32,
    pub parries: u32,
    /// 보스에게 준 피해의 출처: 보통 타 · 패링 뒤 드러남/자세 깨짐 동안의 타 · 되받은 조각.
    pub dmg_plain: i32,
    pub dmg_opened: i32,
    pub dmg_reflect: i32,
    /// 내 공격 동작 중에 맞아 끊긴 횟수.
    pub interrupted: u32,
}

/// 성공률 `p`로 한 판. `seed`마다 굴림이 다르다.
pub fn fight(b: &Build, p: f32, seed: u64, t: &Tuning) -> Fight {
    fight_vs(Kind::Bellkeeper, false, b, p, seed, t)
}

/// `kind`와 싸운다. `dummy`면 보스는 공격하지 않는다(순수 딜 시간 = 하한).
pub fn fight_vs(kind: Kind, dummy: bool, b: &Build, p: f32, seed: u64, t: &Tuning) -> Fight {
    let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
    let mut w = World::new("sim", 18.0);
    w.set_weapon(b.weapon);
    let mut prog = crate::save::Progress::default();
    prog.level = b.level;
    for (k, &c) in b.worn.iter().enumerate().take(crate::charms::SLOTS) {
        prog.worn[k] = Some(c);
    }
    prog.owned = b.worn.clone();
    prog.finds = b.finds.clone();
    prog.tree = b.tree;
    w.apply_stats(prog.stats(), true);
    w.tree = b.tree;
    let boss = w.spawn(kind, vec3(0.0, 0.0, -7.0));
    // 체력 훑기(0127): `SIM_BOSS_HP=60`이면 보스 체력을 바꿔 잰다(자세 막대는 보스 고정값이라 그대로).
    if let Some(hp) = std::env::var("SIM_BOSS_HP")
        .ok()
        .and_then(|v| v.parse().ok())
    {
        w.enemies[boss].max_hp = hp;
        w.enemies[boss].hp = hp;
    }
    let flask0 = w.player.flask;
    let (mut hp_seen, mut taken) = (w.player.hp, 0);
    // 이번 공격의 굴림(공격 번호 → 성공), 울림 고리는 고리마다.
    let mut roll_for: u32 = u32::MAX;
    let mut ok = false;
    let mut ring_ok = [false; 4];
    let mut parries = 0u32;
    let offbeat: f32 = std::env::var("SIM_OFFBEAT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1.0);
    let (mut dmg_plain, mut dmg_opened, mut dmg_reflect, mut interrupted) = (0, 0, 0, 0u32);
    // 기력 남김: 실제 싸움은 점프 두 번 몫, 허수아비는 한 번 칠 몫만.
    let reserve = if dummy { 12.0 } else { 36.0 };
    let limit = 60 * 300;
    let mut tick = 0;
    while tick < limit {
        tick += 1;
        if dummy
            && w.enemies[boss].alive()
            && !matches!(w.enemies[boss].state, EState::Stagger { .. })
        {
            w.enemies[boss].state = EState::Recover { t: -1000.0 };
        }
        let e = &w.enemies[boss];
        if !e.alive() {
            break;
        }
        if w.player.hp <= 0 {
            break;
        }
        let pp = w.player.pos;
        let to = vec3(e.pos.x - pp.x, 0.0, e.pos.z - pp.z);
        let dist = to.length();
        let gap = dist - e.radius - PLAYER_RADIUS;
        let mut a = Actions::default();
        if w.lock.is_none() {
            a.lock = true;
        }
        // 새 공격: 굴린다.
        if matches!(e.state, EState::Tele { .. }) && e.atk_count != roll_for {
            roll_for = e.atk_count;
            // 엇박(0128): 지연·빠름·연속의 이어지는 타·페인트 뒤의 공격은 사람이 덜 받아낸다고 볼 때 — `SIM_OFFBEAT=0.75`면 p × 0.75.
            // 몇 배인지는 확인하지 못했다(가정 — 판정에서 잴 것). 봇 자체는 상태를 읽으니 속지 않는다.
            let off = !matches!(e.tempo, crate::enemy::Tempo::Normal);
            let pen = if off { offbeat } else { 1.0 };
            ok = rng.f() < p * pen;
            for r in &mut ring_ok {
                *r = rng.f() < p;
            }
        }
        let tele_left = match e.state {
            EState::Tele { t: te } => e.tele_dur() - te,
            _ => f32::MAX,
        };
        let busy = !matches!(w.player.state, PState::Idle | PState::Attack { .. });
        let mut defending = false;
        match (e.state, e.atk) {
            // 휘두르기: 다 차기 직전에 패링.
            (EState::Tele { .. }, Atk::Slam | Atk::Smash) => {
                defending = tele_left < 0.5;
                if ok && tele_left <= 0.07 {
                    a.parry = true;
                }
            }
            // 돌진: 닿기 직전.
            (EState::Tele { .. }, Atk::Charge) => defending = tele_left < 0.5,
            (EState::Charge { dir, .. }, Atk::Charge) => {
                defending = true;
                let closing = dir.dot(-to.normalize_or_zero()) > 0.3;
                if ok && closing && dist - e.touch() <= 1.0 {
                    a.parry = true;
                }
            }
            // 울림 고리: 고리마다 닿기 직전(공중 판정이 서는 시간만큼 앞)에 뛴다 — 예고 중에도(가까우면 첫 고리가 곧 닿는다).
            (EState::Tele { .. } | EState::Wave { .. }, Atk::Toll | Atk::Wave) => {
                defending = tele_left < 0.5 || matches!(e.state, EState::Wave { .. });
                let elapsed = match e.state {
                    EState::Wave { t: wt } => wt,
                    _ => -tele_left,
                };
                let v = e.def().wave_speed;
                let grounded = w.player.air <= 0.0
                    && matches!(w.player.state, PState::Idle | PState::Attack { .. });
                for k in 0..e.ring_count() as usize {
                    let front = elapsed * v - k as f32 * crate::tuning::TOLL_GAP;
                    let arrive = (dist - 0.7 - front) / v;
                    let hit_already = e.rings_hit & (1 << k) != 0;
                    if ring_ok[k.min(3)]
                        && !hit_already
                        && grounded
                        && (0.07..0.13).contains(&arrive)
                    {
                        a.jump = true;
                    }
                }
            }
            (EState::Tele { .. }, Atk::Shards) => defending = tele_left < 0.5,
            _ => {}
        }
        // 날아오는 조각: 가장 가까운 것이 닿기 직전에 패링.
        if let Some(near) = w
            .shards
            .iter()
            .filter(|s| !s.reflected)
            .map(|s| vec2(s.pos.x - pp.x, s.pos.z - pp.z).length())
            .reduce(f32::min)
        {
            defending = true;
            if ok && near <= 1.3 {
                a.parry = true;
            }
        }
        // 틈이면 붙어서 친다(기력이 남았을 때 — 패링 몫은 남긴다).
        let opening = !defending && tele_left > 0.55;
        if opening && gap > 1.3 {
            a.move_dir = vec2(0.0, 1.0);
        }
        if opening && gap <= 1.6 && w.player.stamina >= reserve && !busy {
            a.attack = true;
        }
        // 등잔: 체력 2 이하·틈·조금 떨어져 있으면.
        if w.player.hp <= 2 && w.player.flask > 0 && opening && !busy {
            a.flask = true;
            a.attack = false;
            a.move_dir = Vec2::ZERO;
        }
        let (hp_b, opened_b) = (
            w.enemies[boss].hp,
            w.enemies[boss].exposed > 0.0 || w.enemies[boss].broken > 0.0,
        );
        let back_b = w.shards.iter().filter(|s| s.reflected).count();
        let attacking = matches!(w.player.state, PState::Attack { .. });
        let ev = w.step(&a, t);
        let lost = hp_b - w.enemies[boss].hp.max(0).min(hp_b);
        let back_a = w.shards.iter().filter(|s| s.reflected).count();
        let mut rest = lost;
        if back_a < back_b && rest >= crate::tuning::SHARD_REFLECT_DAMAGE {
            dmg_reflect += crate::tuning::SHARD_REFLECT_DAMAGE;
            rest -= crate::tuning::SHARD_REFLECT_DAMAGE;
        }
        if opened_b {
            dmg_opened += rest;
        } else {
            dmg_plain += rest;
        }
        if attacking && w.player.hp < hp_seen {
            interrupted += 1;
        }
        parries += ev
            .iter()
            .filter(|e| matches!(e, crate::world::Event::Parry { .. }))
            .count() as u32;
        if w.player.hp < hp_seen {
            taken += hp_seen - w.player.hp;
            if std::env::var("SIM_TRACE").is_ok() {
                let e = &w.enemies[boss];
                eprintln!(
                    "t={:.2} hurt: boss {:?}/{:?} dist {:.2} player {:?} air {:.2} stam {:.0} shards {} a.parry {} a.jump {}",
                    tick as f32 / 60.0, e.state, e.atk, dist, w.player.state, w.player.air, w.player.stamina, w.shards.len(), a.parry, a.jump
                );
            }
        }
        hp_seen = w.player.hp;
    }
    let e = &w.enemies[boss];
    Fight {
        won: !e.alive(),
        secs: tick as f32 / 60.0,
        taken,
        flasks: flask0 - w.player.flask,
        boss_hp_left: e.hp.max(0),
        parries,
        dmg_plain,
        dmg_opened,
        dmg_reflect,
        interrupted,
    }
}

/// 성공률 하나에서 여러 판의 평균.
pub struct Sum {
    pub win: f32,
    pub secs: f32,
    pub taken: f32,
    pub flasks: f32,
    pub left: f32,
    pub parries: f32,
    /// 이긴 판의 피해 몫(보통·열린 틈·되받은 조각, 0..1).
    pub share: (f32, f32, f32),
    pub interrupted: f32,
}

pub fn summary(kind: Kind, b: &Build, p: f32, runs: u64, t: &Tuning) -> Sum {
    let fs: Vec<Fight> = (0..runs)
        .map(|s| fight_vs(kind, false, b, p, s + 1, t))
        .collect();
    let n = fs.len() as f32;
    let avg = |f: &dyn Fn(&Fight) -> f32| fs.iter().map(f).sum::<f32>() / n;
    let wins: Vec<&Fight> = fs.iter().filter(|f| f.won).collect();
    let secs = if wins.is_empty() {
        f32::NAN
    } else {
        wins.iter().map(|f| f.secs).sum::<f32>() / wins.len() as f32
    };
    let (pl, op, rf) = fs.iter().fold((0, 0, 0), |a, f| {
        (a.0 + f.dmg_plain, a.1 + f.dmg_opened, a.2 + f.dmg_reflect)
    });
    let all = (pl + op + rf).max(1) as f32;
    Sum {
        win: wins.len() as f32 / n,
        secs,
        taken: avg(&|f| f.taken as f32),
        flasks: avg(&|f| f.flasks as f32),
        left: avg(&|f| f.boss_hp_left as f32),
        parries: avg(&|f| f.parries as f32),
        share: (pl as f32 / all, op as f32 / all, rf as f32 / all),
        interrupted: avg(&|f| f.interrupted as f32),
    }
}

pub fn builds() -> Vec<Build> {
    use crate::weapons::Id as W;
    let tree = |r: u8| Tree {
        open: [0, r, 0],
        pin: None,
    };
    let finds = || {
        vec![
            "sanctuary.lamp.1".to_string(),
            "sanctuary.sip.1".to_string(),
        ]
    };
    vec![
        Build {
            name: "A bare (Lv0, no charm, sword)",
            level: Level::default(),
            worn: vec![],
            finds: vec![],
            tree: tree(0),
            weapon: W::Sword,
        },
        // 첫 도착(0127): 잔불 ≈ 280 = 레벨 셋, 성소 탑의 귀 + 문지기의 손 + 마을 상자의 꿰맨 천, 등잔·한 모금, 마디 셋.
        Build {
            name: "B first arrival (Lv3 life1/might2, Ear+Hand+Cloth, lamp+sip, Riposte2)",
            level: Level {
                life: 1,
                stamina: 0,
                might: 2,
            },
            worn: vec![Charm::Ear, Charm::Hand, Charm::Cloth],
            finds: finds(),
            tree: tree(2),
            weapon: W::Sword,
        },
        // 둘째 판 이후(문지기 핵 둘·마디 넷): 힘 3단(강타 +1).
        Build {
            name: "C later run (Lv5 life2/might3, Ear+Hand+Cloth, lamp+sip, Riposte4)",
            level: Level {
                life: 2,
                stamina: 0,
                might: 3,
            },
            worn: vec![Charm::Ear, Charm::Hand, Charm::Cloth],
            finds: finds(),
            tree: tree(4),
            weapon: W::Sword,
        },
    ]
}

#[test]
#[ignore]
fn bellkeeper_fight_length() {
    let t = Tuning::default();
    let runs = 40;
    for (kind, bs) in [
        (Kind::Warden, &builds()[..3]),
        (Kind::Bellkeeper, &builds()[..]),
    ] {
        for b in bs {
            let dummy = fight_vs(kind, true, b, 1.0, 1, &t).secs;
            println!("\n## {kind:?} — {} (dummy: {dummy:.1}s)", b.name);
            println!("p    | win  | secs(win) | taken | flasks | hp left | parries | cut | dmg plain/opened/reflect");
            for p in [0.0, 0.25, 0.5, 0.7, 0.85, 1.0] {
                let m = summary(kind, b, p, runs, &t);
                println!(
                    "{p:.2} | {:>3.0}% | {:>9.1} | {:>5.1} | {:>6.2} | {:>7.1} | {:>7.1} | {:>3.1} | {:.0}%/{:.0}%/{:.0}%",
                    m.win * 100.0, m.secs, m.taken, m.flasks, m.left, m.parries, m.interrupted,
                    m.share.0 * 100.0, m.share.1 * 100.0, m.share.2 * 100.0
                );
            }
        }
    }
}

/// 모의 장치가 살아 있나: 다 받아내는 봇은 맨몸으로도 이긴다.
#[test]
fn a_perfect_bot_beats_the_bellkeeper_bare_handed_with_a_sword() {
    let t = Tuning::default();
    let f = fight(&builds()[0], 1.0, 1, &t);
    assert!(f.won, "{f:?}");
    assert!(f.parries > 0);
}

#[test]
#[ignore]
fn jump_curve() {
    let t = Tuning::default();
    let mut w = World::new("sim", 18.0);
    let mut a = Actions {
        jump: true,
        ..Default::default()
    };
    for i in 0..50 {
        w.step(&a, &t);
        a.jump = false;
        println!("{i:>2} {:.3} {:?}", w.player.air, w.player.state);
    }
}
