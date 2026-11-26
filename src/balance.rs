//! 보스 싸움 길이 모의(0126): 실제 `World`로 종지기와 싸우는 봇을 돌려 성공률별 싸움 시간·받은 피해를 잰다.
//! 봇은 보스 상태를 읽는다(사람이 예고를 읽는 것의 대역). 공격마다 한 번 굴려 `p`면 맞는 순간에 패링(되받을 수 있는 것)·
//! 점프(울림 고리 — 고리마다 굴림)를 정확히 하고, 아니면 아무것도 안 해 맞는다(늦거나 이른 패링 = 맞음과 같게 본다).
//! 틈(보스가 예고 중이 아니거나 예고가 많이 남았을 때)에는 붙어서 평타 콤보를 넣는다. 체력 20(두 대 몫) 이하면 틈에 등잔을 마신다.
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
    /// 든 무기를 벼린 단(0180 — 0~3). `SIM_FORGE`가 있으면 그것이 앞선다.
    pub forge: u8,
    /// 보조무기 슬롯에 멘 무기(0272 1b — 그 종류의 보조특성이 능력치에 든다). 봇은 바꿔 들지 않는다.
    pub sub: Option<crate::weapons::Id>,
    /// 왼손의 방패(0274 ② — 망치도 한손이 되어 방패와 든다). 봇은 막지 않는다(패링 버튼을 한 틱만 누른다) — 쥔 꼴만 바뀐다.
    pub shield: bool,
    /// 재기(0274 ② — 한손검의 스킬)의 레벨(0 = 없음): 끼워 두고, 틈에 체력이 한 번 돌 만큼 빠졌으면 쓴다.
    pub rally: u8,
    /// 보조무기 슬롯이 비었다고 월드에 말한다(0276 — 빈 손의 결이 켜진다. 부무기 · 방패가 있으면 결은 없다).
    pub bare: bool,
    /// 봇의 손(0276 — 빈 손의 결을 재려고): 보통(패링 · 평타 콤보) · 구르기(패링할 때 옆으로 구르고 곧장 친다 — 그림자 걸음) ·
    /// 강공격(틈에 평타 대신 다 모은 강공격 — 두 손 내려베기 · 이어 꿰뚫기).
    pub style: Style,
    /// 대검을 휘두르는 동안의 휘청 배를 바꾼다(0274 ④ — `Some(1.0)` = 휘청 줄임 없음: 그것이 보스전을 얼마나 줄이나 잰다).
    pub flinch: Option<f32>,
}

/// 봇의 손(0276).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    Plain,
    Dodge,
    Heavy,
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
    // 장신구는 제 종류의 칸에(0224 — 같은 종류 둘은 앞의 것만).
    for &c in b.worn.iter() {
        let slot = &mut prog.worn[c.kind().slot()];
        if slot.is_none() {
            *slot = Some(c);
        }
    }
    prog.owned = b.worn.clone();
    prog.finds = b.finds.clone();
    prog.board.cells = b.tree;
    // 벼린 단 훑기(0180): `SIM_FORGE=2`면 든 무기를 그 단까지 벼린 것으로 잰다.
    prog.weapon = Some(b.weapon);
    prog.sub = b.sub;
    w.set_sub(b.sub);
    if b.shield {
        w.set_shield(Some(crate::shield::Id::Pilgrim));
    }
    w.set_open_hand(b.bare);
    w.flinch_probe = b.flinch;
    if b.rally > 0 {
        use crate::skills::{Id as Sk, Loadout, Pick, Slot, Ult};
        w.set_skills(Loadout {
            slots: [
                Slot {
                    id: Sk::Rally,
                    rank: b.rally,
                    pick: Pick::A,
                },
                Slot {
                    id: Sk::Whirl,
                    rank: 0,
                    pick: Pick::A,
                },
            ],
            ult: 0,
            great: Ult::Flurry,
        });
    }
    prog.forged.set(b.weapon, b.forge);
    if let Some(r) = std::env::var("SIM_FORGE").ok().and_then(|v| v.parse().ok()) {
        prog.forged.set(b.weapon, r);
    }
    // 옵션 훑기(0223 V4): `SIM_DMG=6`이면 "주는 피해" 옵션을 얹어 잰다.
    let mut st = prog.stats();
    if let Some(d) = std::env::var("SIM_DMG")
        .ok()
        .and_then(|v| v.parse::<i32>().ok())
    {
        st.dmg += d;
    }
    // 기력 회복 훑기(0272 1b — 보조특성의 값을 고를 때): `SIM_REGEN=1.05`면 기력 회복에 그 배를 얹어 잰다.
    if let Some(k) = std::env::var("SIM_REGEN")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
    {
        st.stamina_regen *= k;
    }
    // 자세 피해 훑기(같은 까닭): `SIM_POISE=1.05`.
    if let Some(k) = std::env::var("SIM_POISE")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
    {
        st.poise *= k;
    }
    // 최대 기력 훑기(0272 2단계 — 창의 보조특성): `SIM_STAMINA=3`이면 최대 기력에 그만큼을 얹어 잰다.
    if let Some(k) = std::env::var("SIM_STAMINA")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
    {
        st.stamina_max += k;
    }
    w.apply_stats(st, true);
    w.tree = b.tree;
    let boss = w.spawn(kind, vec3(0.0, 0.0, -7.0));
    // 체력 훑기(0127): `SIM_BOSS_HP=600`이면 보스 체력을 바꿔 잰다(자세 막대는 보스 고정값이라 그대로).
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
    // 구르는 손(0276): 이 공격 번호의 틈에 이미 굴렀다.
    let mut dodged_for: u32 = u32::MAX;
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
            let off = !matches!(e.rhythm.tempo, crate::enemy::Tempo::Normal);
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
            // 2단계(0133) 전환: 튕기니 치지 않는다.
            (EState::Rise { .. }, _) => defending = true,
            // 뿌리 기둥 줄: 줄 위면 옆으로 비킨다(성공이면).
            (EState::Tele { .. } | EState::Wave { .. }, Atk::Roots) => {
                let f = e.facing_dir();
                let rel = -to;
                let along = rel.dot(f);
                let side = (rel - f * along).length();
                let on_line = along > 0.0 && side <= crate::tuning::ROOTS_R + PLAYER_RADIUS + 0.4;
                defending = true;
                if ok && on_line {
                    a.move_dir = vec2(1.0, 0.0);
                }
            }
            // 내려 딛기(0270 D 고요한 사슴): 성공이면 원판 밖으로 물러나고 뿌리 줄(한 줄·세 줄)에서 옆으로 비킨다(줄 사이의
            // 빈 데로 — 가까운 줄에서 멀어지는 쪽) — 실패면 맞는다.
            (EState::Tele { .. } | EState::Swing { .. } | EState::Wave { .. }, Atk::Stomp) => {
                defending = true;
                let rel = -to;
                let inside = matches!(e.state, EState::Tele { .. })
                    && dist <= crate::tuning::STOMP_R + PLAYER_RADIUS + 0.5;
                let mut want = if inside {
                    rel.normalize_or_zero()
                } else {
                    Vec3::ZERO
                };
                for (yaw, len) in e.root_lines() {
                    let f = e.dir_at(yaw);
                    let along = rel.dot(f);
                    let off = rel - f * along;
                    if along > 0.0
                        && along <= len + 1.0
                        && off.length() <= crate::tuning::ROOTS_R + PLAYER_RADIUS + 0.3
                    {
                        want += off.normalize_or(vec3(f.z, 0.0, -f.x));
                    }
                }
                if ok && want.length_squared() > 1e-4 {
                    // 락온 기준(앞 = 보스 쪽)의 스틱으로.
                    let fwd = to.normalize_or(Vec3::Z);
                    let right = fwd.cross(Vec3::Y);
                    a.move_dir = vec2(want.dot(right), want.dot(fwd)).normalize_or_zero();
                }
            }
            // 종 내던지기: 떨어질 원판 안이면 빠져나오고(성공이면), 떨어진 자리의 고리는 울림처럼 뛴다.
            (EState::Tele { .. } | EState::Swing { .. }, Atk::Hurl) => {
                defending = true;
                if let Some(at) = e.plant_at {
                    let inside = vec2(pp.x - at.x, pp.z - at.z).length()
                        <= crate::tuning::HURL_R + PLAYER_RADIUS + 0.3;
                    if ok && inside {
                        a.move_dir = vec2(1.0, 0.0);
                    }
                }
            }
            (EState::Wave { t: wt }, Atk::Hurl) => {
                defending = true;
                let c = e.wave_center();
                let from_c = vec2(pp.x - c.x, pp.z - c.z).length();
                let v = e.def().wave_speed;
                let arrive = (from_c - 0.7 - wt * v) / v;
                let grounded = w.player.air <= 0.0
                    && matches!(w.player.state, PState::Idle | PState::Attack { .. });
                if ring_ok[0] && e.rings_hit == 0 && grounded && (0.07..0.13).contains(&arrive) {
                    a.jump = true;
                }
            }
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
        // 구르는 손(0276 — 그림자 걸음을 재려고): 틈마다 처음 한 번은 보스 쪽으로 굴러 들어가 친다(구른 뒤 첫 타). 기력은 회피 몫까지 남긴다.
        let dodge_cost = t[crate::tuning::P::StaminaDodge];
        if b.style == Style::Dodge
            && opening
            && dodged_for != e.atk_count
            && gap <= 4.0
            && gap > 0.6
            && !busy
            && matches!(w.player.state, PState::Idle)
            && w.player.stamina >= reserve + dodge_cost
        {
            dodged_for = e.atk_count;
            a.dodge = true;
            a.move_dir = vec2(0.0, 1.0);
        }
        if opening && gap <= 1.6 && w.player.stamina >= reserve && !busy && !a.dodge {
            a.attack = true;
        }
        // 강공격의 손(0276): 평타를 누른 채 다 모을 때까지 쥐고 뗀다(지키는 때가 오면 뗀다).
        if b.style == Style::Heavy {
            let full = t[crate::tuning::P::HeavyCharge]
                * w.player.weapon.def().heavy.map_or(1.0, |h| h.charge);
            a.attack_held = match w.player.state {
                PState::Attack { .. } => !defending,
                PState::Charging { t: e } => !defending && e < full,
                _ => a.attack,
            };
        }
        // 재기(0274 ②): 틈에 체력이 한 번 돌 만큼(두 눈금) 빠졌고 쿨타임이 끝났으면 — 등잔보다 먼저.
        let rally_ready = b.rally > 0 && w.player.skill.cd[crate::skills::Id::Rally.index()] <= 0.0;
        if rally_ready
            && w.player.hp <= w.player.max_hp - 2 * crate::tuning::HP_UNIT
            && opening
            && !busy
        {
            a.skill1 = true;
            a.attack = false;
        }
        // 등잔: 체력 두 눈금(20) 이하·틈·조금 떨어져 있으면.
        if w.player.hp <= 2 * crate::tuning::HP_UNIT && w.player.flask > 0 && opening && !busy {
            a.flask = true;
            a.attack = false;
            a.move_dir = Vec2::ZERO;
        }
        let (hp_b, opened_b) = (
            w.enemies[boss].hp,
            w.enemies[boss].exposed > 0.0 || w.enemies[boss].stance.broken > 0.0,
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
    let tree = |r: u8| Tree::from_counts([0, r, 0]);
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
            forge: 0,
            sub: None,
            shield: false,
            rally: 0,
            bare: false,
            style: Style::Plain,
            flinch: None,
        },
        // 첫 도착(0127): 잔불 ≈ 280 = 레벨 셋, 성소 탑의 귀 + 문지기의 손 + 마을 상자의 꿰맨 천, 등잔·한 모금, 패시브 칸 둘.
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
            forge: 0,
            sub: None,
            shield: false,
            rally: 0,
            bare: false,
            style: Style::Plain,
            flinch: None,
        },
        // 둘째 판 이후(문지기 핵 둘·패시브 칸 넷): 힘 3단(강타 +1).
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
            forge: 0,
            sub: None,
            shield: false,
            rally: 0,
            bare: false,
            style: Style::Plain,
            flinch: None,
        },
        // 정원 도착(0185): 문지기 둘·마을의 잔불 — 레벨 일곱, 패시브 칸 여섯.
        Build {
            name: "D garden arrival (Lv7 life3/might4, Ear+Hand+Cloth, lamp+sip, Riposte5)",
            level: Level {
                life: 3,
                stamina: 0,
                might: 4,
            },
            worn: vec![Charm::Ear, Charm::Hand, Charm::Cloth],
            finds: finds(),
            tree: tree(5),
            weapon: W::Sword,
            forge: 0,
            sub: None,
            shield: false,
            rally: 0,
            bare: false,
            style: Style::Plain,
            flinch: None,
        },
        // 작업층 도달(0216 W1): 문지기 셋 — 레벨 열, 힘 5단(평타 +10), 패시브 칸 여섯.
        Build {
            name: "E workings arrival (Lv10 life4/stamina1/might5, Ear+Hand+Cloth, lamp+sip, Riposte6)",
            level: Level {
                life: 4,
                stamina: 1,
                might: 5,
            },
            worn: vec![Charm::Ear, Charm::Hand, Charm::Cloth],
            finds: finds(),
            tree: tree(6),
            weapon: W::Sword,
            forge: 0,
            sub: None,
            shield: false,
            rally: 0,
            bare: false,
            style: Style::Plain,
            flinch: None,
        },
    ]
}

/// 성소에 닿아 문지기 앞에 선 몸(0270 D 필드 보스 모의 — "성소에 갓 닿은 몸"): 잔불 ≈ 150 = 레벨 둘, 탑의 귀, 등잔·한 모금,
/// 패시브 칸 하나. 첫 도착 몸(B — 마을에 닿은 몸)보다 한 걸음 앞.
pub fn sanctuary_build() -> Build {
    Build {
        name: "S sanctuary, at the gate (Lv2 life1/might1, Ear, lamp+sip, Riposte1)",
        level: Level {
            life: 1,
            stamina: 0,
            might: 1,
        },
        worn: vec![Charm::Ear],
        finds: vec![
            "sanctuary.lamp.1".to_string(),
            "sanctuary.sip.1".to_string(),
        ],
        tree: Tree::from_counts([0, 1, 0]),
        weapon: crate::weapons::Id::Sword,
        forge: 0,
        sub: None,
        shield: false,
        rally: 0,
        bare: false,
        style: Style::Plain,
        flinch: None,
    }
}

/// 필드 보스 모의(0270 D): 고요한 사슴 — 성소에 갓 닿은 몸(S)은 드물게(25~40%), 둘째 지역을 마친 몸(D — 정원 도착)은
/// 대개(70~85%) 이기게, 싸움은 60~90초. 돌리기: `cargo test stag_fight_length -- --ignored --nocapture`
/// (`SIM_BOSS_HP`로 체력을 훑는다).
#[test]
#[ignore]
fn stag_fight_length() {
    let t = Tuning::default();
    let runs = 120;
    let all = builds();
    // 둘째 지역을 마친 몸(D)은 숫돌 넷(성소 1 + 마을 3)을 지나왔다 — 벼림 0·1·2단으로 나눠 본다(0216: 정원 도착 = 벼림 1~2).
    let forged = |b: &Build, forge: u8, name: &'static str| Build {
        forge,
        name,
        ..b.clone()
    };
    let bs = [
        all[0].clone(),
        sanctuary_build(),
        forged(&sanctuary_build(), 1, "S+1 sanctuary, forged once"),
        all[1].clone(),
        all[3].clone(),
        forged(&all[3], 1, "D+1 garden arrival, forged once"),
        forged(&all[3], 2, "D+2 garden arrival, forged twice"),
    ];
    for b in &bs {
        let dummy = fight_vs(Kind::Stag, true, b, 1.0, 1, &t).secs;
        println!("\n## Stag — {} (dummy: {dummy:.1}s)", b.name);
        println!(
            "p    | win  | secs(win) | taken | flasks | hp left | parries | cut | dmg plain/opened"
        );
        let ps: &[f32] = if std::env::var("SIM_QUICK").is_ok() {
            &[0.5, 0.7]
        } else {
            &[0.25, 0.5, 0.7, 0.85, 1.0]
        };
        for &p in ps {
            let m = summary(Kind::Stag, b, p, runs, &t);
            println!(
                "{p:.2} | {:>3.0}% | {:>9.1} | {:>5.1} | {:>6.2} | {:>7.1} | {:>7.1} | {:>3.1} | {:.0}%/{:.0}%",
                m.win * 100.0, m.secs, m.taken, m.flasks, m.left, m.parries, m.interrupted,
                m.share.0 * 100.0, m.share.1 * 100.0
            );
        }
    }
}

/// 모의 장치가 사슴의 수를 읽는다: 다 받아내는 봇은 문지기 앞의 몸으로도 이긴다(원판에서 물러나고 뿌리 줄을 비킨다).
#[test]
fn a_perfect_bot_beats_the_stag() {
    let t = Tuning::default();
    let f = fight_vs(Kind::Stag, false, &sanctuary_build(), 1.0, 1, &t);
    assert!(f.won, "{f:?}");
    assert!(f.parries > 0);
}

#[test]
#[ignore]
fn bellkeeper_fight_length() {
    let t = Tuning::default();
    let runs = 40;
    for (kind, bs) in [
        (Kind::Warden, &builds()[..3]),
        (Kind::Bellkeeper, &builds()[..4]),
        (Kind::Mossward, &builds()[2..4]),
        (Kind::Overseer, &builds()[3..]),
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

/// 보조특성 모의(0272 1b): 같은 몸·같은 씨앗으로 보조무기 슬롯만 바꿔 싸움 길이를 견준다 — 어느 보조특성도 싸움을 4% 넘게
/// 줄이지 않아야 한다. 봇은 평타 콤보만 친다(스킬·바꿔 들기 없음): 기력 회복(검)·자세 피해(망치)는 이 모의에 그대로 들고,
/// 속성 게이지(지팡이)·스킬 쿨타임(활)·등의 장전(쇠뇌)은 0으로 나온다(쓰는 길이 없다 — 상한은 문서에 셈으로 적는다).
/// 돌리기: `cargo test sub_trait_fight_length -- --ignored --nocapture`
#[test]
#[ignore]
fn sub_trait_fight_length() {
    use crate::weapons::Id as W;
    let t = Tuning::default();
    // 짧은 싸움(문지기 15~20초)은 판마다 한두 번의 주고받음이 몇 %라 60판으로는 흔들린다 — `SIM_RUNS=300`으로 좁힌다.
    let runs = std::env::var("SIM_RUNS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(60);
    let all = builds();
    let with = |b: &Build, weapon: W, sub: Option<W>| Build {
        weapon,
        sub,
        ..b.clone()
    };
    for (kind, bi) in [
        (Kind::Warden, 1),
        (Kind::Bellkeeper, 1),
        (Kind::Bellkeeper, 3),
        (Kind::Mossward, 3),
    ] {
        let b = &all[bi];
        for (main, subs) in [
            (
                W::Sword,
                &[
                    W::Hammer,
                    W::Staff,
                    W::Bow,
                    W::Crossbow,
                    W::Spear,
                    W::Daggers,
                ][..],
            ),
            (W::Hammer, &[W::Sword, W::Spear][..]),
            (W::Staff, &[W::Sword, W::Hammer, W::Spear][..]),
            // 0272 2단계: 창(주무기로도).
            (W::Spear, &[W::Sword, W::Hammer][..]),
            // 0274 ③: 쌍단검(주무기로) · 쌍단검을 멘 검(손버릇 — 잔불만이라 0%여야 한다).
            (W::Daggers, &[W::Sword][..]),
        ] {
            let base = with(b, main, None);
            let d0 = fight_vs(kind, true, &base, 1.0, 1, &t).secs;
            println!(
                "\n## {kind:?} — {} — main {main:?} (dummy {d0:.1}s)",
                b.name
            );
            println!("sub      | dummy    | p    | win  | secs(win) | Δsecs");
            for p in [0.5, 0.7, 0.85] {
                let m0 = summary(kind, &base, p, runs, &t);
                println!(
                    "none     | {d0:>6.1}s  | {p:.2} | {:>3.0}% | {:>9.1} |",
                    m0.win * 100.0,
                    m0.secs
                );
                for &sub in subs {
                    let sb = with(b, main, Some(sub));
                    let d = fight_vs(kind, true, &sb, 1.0, 1, &t).secs;
                    let m = summary(kind, &sb, p, runs, &t);
                    println!(
                        "{:<8} | {d:>6.1}s {:>+5.1}% | {p:.2} | {:>3.0}% | {:>9.1} | {:>+5.1}%",
                        format!("{sub:?}"),
                        (d / d0 - 1.0) * 100.0,
                        m.win * 100.0,
                        m.secs,
                        (m.secs / m0.secs - 1.0) * 100.0
                    );
                }
            }
        }
    }
}

/// 보조특성은 모의에 든다: 망치를 멘 검의 몸은 자세 피해가 늘어 있다(능력치를 합치는 한 곳을 지난다).
#[test]
fn the_sim_carries_the_sub_weapons_trait() {
    let t = Tuning::default();
    let b = Build {
        sub: Some(crate::weapons::Id::Hammer),
        ..builds()[0].clone()
    };
    let with = fight_vs(Kind::Warden, true, &b, 1.0, 1, &t);
    let without = fight_vs(Kind::Warden, true, &builds()[0], 1.0, 1, &t);
    assert!(with.won && without.won);
    assert!(
        with.secs <= without.secs,
        "{} vs {}",
        with.secs,
        without.secs
    );
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

/// 보조특성의 값 고르기(0272 1b — 한손검의 기력 회복 `SIM_REGEN` · 망치의 자세 피해 `SIM_POISE` · 2단계 창의 최대 기력 `SIM_STAMINA`): 무기 셋으로, 가장 짧은 싸움
/// (문지기)까지 — 배를 바꿔 가며 같은 씨앗으로 견준다(보조무기 없이 능력치에 바로 얹는다).
/// 돌리기: `SIM_REGEN=1.04 SIM_RUNS=300 cargo test --release sub_knob_probe -- --ignored --nocapture`
#[test]
#[ignore]
fn sub_knob_probe() {
    use crate::weapons::Id as W;
    let t = Tuning::default();
    let runs = std::env::var("SIM_RUNS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(120);
    let all = builds();
    for (kind, bi) in [
        (Kind::Warden, 1),
        (Kind::Bellkeeper, 1),
        (Kind::Mossward, 3),
    ] {
        for main in [W::Sword, W::Hammer, W::Staff] {
            let b = Build {
                weapon: main,
                ..all[bi].clone()
            };
            let d = fight_vs(kind, true, &b, 1.0, 1, &t).secs;
            let row: Vec<String> = [0.5, 0.7, 0.85]
                .into_iter()
                .map(|p| {
                    let m = summary(kind, &b, p, runs, &t);
                    format!(
                        "{:.2}/{:.0}% (parries {:.1}, opened {:.0}%)",
                        m.secs,
                        m.win * 100.0,
                        m.parries,
                        m.share.1 * 100.0
                    )
                })
                .collect();
            println!("{kind:?} {main:?} dummy {d:.2} | {}", row.join(" | "));
        }
    }
}

/// 0274 ② 종류 모의: 한손검 · 망치(한손 3연타) · 망치와 방패 · 재기를 쓰는 한손검 · 망치를 멘 한손검(보조특성 = 등잔 회복량) · 쌍단검(0274 ③)을
/// 같은 몸·같은 씨앗으로 — 문지기(첫 도착 B) · 종지기(B) · 이끼 덮인 문지기(정원 도착 D). 성공률 0.5 · 0.7 · 0.85.
/// 돌리기: `SIM_RUNS=300 cargo test --release class_fight_probe -- --ignored --nocapture`
#[test]
#[ignore]
fn class_fight_probe() {
    use crate::weapons::Id as W;
    let t = Tuning::default();
    let runs = std::env::var("SIM_RUNS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(120);
    let all = builds();
    for (kind, bi) in [
        (Kind::Warden, 1),
        (Kind::Bellkeeper, 1),
        (Kind::Mossward, 3),
    ] {
        let b = &all[bi];
        let v = |name: &'static str, weapon: W, sub: Option<W>, shield: bool, rally: u8| Build {
            name,
            weapon,
            sub,
            shield,
            rally,
            ..b.clone()
        };
        let variants = [
            v("sword", W::Sword, None, false, 0),
            v("sword+rally1", W::Sword, None, false, 1),
            v("sword+rally5", W::Sword, None, false, 5),
            v("sword/sub hammer", W::Sword, Some(W::Hammer), false, 0),
            v("hammer", W::Hammer, None, false, 0),
            v("hammer+shield", W::Hammer, None, true, 0),
            v("hammer/sub sword", W::Hammer, Some(W::Sword), false, 0),
            // 0274 ③: 쌍단검(주무기 · 보조무기로).
            v("daggers", W::Daggers, None, false, 0),
            v("daggers/sub sword", W::Daggers, Some(W::Sword), false, 0),
            v("sword/sub daggers", W::Sword, Some(W::Daggers), false, 0),
            // 0274 ④: 대검(주무기 · 보조무기로) · 휘청 줄임을 걷은 대검(그것이 싸움을 얼마나 줄이나).
            v("greatsword", W::Greatsword, None, false, 0),
            Build {
                flinch: Some(1.0),
                ..v("gs no steady", W::Greatsword, None, false, 0)
            },
            v(
                "greatsword/sub sword",
                W::Greatsword,
                Some(W::Sword),
                false,
                0,
            ),
            v(
                "sword/sub greatsw.",
                W::Sword,
                Some(W::Greatsword),
                false,
                0,
            ),
            // 다 모은 강공격의 덤(대검을 멘 보조특성)은 쥐어 모으는 손에서만 — 강공격의 손으로도 잰다.
            Build {
                style: Style::Heavy,
                ..v("sword heavy-hand", W::Sword, None, false, 0)
            },
            Build {
                style: Style::Heavy,
                ..v(
                    "sword heavy/sub gs",
                    W::Sword,
                    Some(W::Greatsword),
                    false,
                    0,
                )
            },
        ];
        println!("\n## {kind:?} — {}", b.name);
        println!("variant          | dummy  | p    | win  | secs(win) | taken | flasks");
        for bv in &variants {
            let d = fight_vs(kind, true, bv, 1.0, 1, &t).secs;
            for p in [0.5, 0.7, 0.85] {
                let m = summary(kind, bv, p, runs, &t);
                println!(
                    "{:<16} | {d:>5.1}s | {p:.2} | {:>3.0}% | {:>9.1} | {:>5.1} | {:>5.2}",
                    bv.name,
                    m.win * 100.0,
                    m.secs,
                    m.taken,
                    m.flasks
                );
            }
        }
    }
}

/// 0276 빈 손의 결 모의: 같은 무기로 보조무기 슬롯을 비운 몸(빈 손) · 무기를 멘 몸 · 방패를 든 몸을 같은 씨앗으로 — 기준은 "빈 손이
/// 가장 좋은 보조무기 조합보다 어느 싸움도 4% 넘게 빠르지 않다". 봇의 손을 결에 맞춘다(그림자 걸음 = 구르는 손 · 두 손 내려베기와
/// 이어 꿰뚫기 = 강공격의 손 · 기도하는 손 = 보통 손이 마시는 등잔). 지팡이의 칸 · 활 · 쇠뇌는 봇이 원거리를 쓰지 않아 재지 못한다.
/// 돌리기: `SIM_RUNS=300 cargo test --release bare_hand_probe -- --ignored --nocapture`
#[test]
#[ignore]
fn bare_hand_probe() {
    use crate::weapons::Id as W;
    let t = Tuning::default();
    let runs = std::env::var("SIM_RUNS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(60);
    let only = std::env::var("SIM_ONLY").ok();
    let all = builds();
    // (주무기, 봇의 손, 견줄 보조무기 슬롯 — None = 방패)
    let rows: [(W, Style, &[Option<W>]); 10] = [
        (
            W::Sword,
            Style::Plain,
            &[Some(W::Hammer), Some(W::Spear), Some(W::Daggers), None],
        ),
        (
            W::Sword,
            Style::Heavy,
            &[Some(W::Hammer), Some(W::Spear), Some(W::Daggers), None],
        ),
        (
            W::Hammer,
            Style::Plain,
            &[Some(W::Sword), Some(W::Spear), None],
        ),
        (W::Spear, Style::Plain, &[Some(W::Sword), Some(W::Hammer)]),
        (W::Spear, Style::Heavy, &[Some(W::Sword), Some(W::Hammer)]),
        (W::Daggers, Style::Plain, &[Some(W::Sword), Some(W::Hammer)]),
        (W::Daggers, Style::Dodge, &[Some(W::Sword), Some(W::Hammer)]),
        (W::Staff, Style::Plain, &[Some(W::Sword), Some(W::Hammer)]),
        // 0274 ④: 대검(남김없이 휘두른다 — 분노가 두 배로 차고 가득이면 평타가 끝나지 않는다).
        (
            W::Greatsword,
            Style::Plain,
            &[Some(W::Sword), Some(W::Hammer), Some(W::Daggers)],
        ),
        (
            W::Greatsword,
            Style::Heavy,
            &[Some(W::Sword), Some(W::Hammer), Some(W::Daggers)],
        ),
    ];
    let mut worst: f32 = f32::MIN;
    for (kind, bi) in [
        (Kind::Warden, 1),
        (Kind::Bellkeeper, 1),
        (Kind::Mossward, 3),
    ] {
        let b = &all[bi];
        for (main, style, subs) in rows {
            if only
                .as_deref()
                .is_some_and(|o| !format!("{main:?}").eq_ignore_ascii_case(o))
            {
                continue;
            }
            let mk = |sub: Option<W>, shield: bool, bare: bool| Build {
                weapon: main,
                sub,
                shield,
                bare,
                style,
                ..b.clone()
            };
            println!("\n## {kind:?} — {} — {main:?} / {style:?}", b.name);
            println!("off hand       | p    | win  | secs(win) | Δ vs best");
            for p in [0.5, 0.7, 0.85] {
                let bare = summary(kind, &mk(None, false, true), p, runs, &t);
                let mut best = f32::MAX;
                let mut lines = Vec::new();
                for &sub in subs {
                    let m = summary(kind, &mk(sub, sub.is_none(), false), p, runs, &t);
                    best = best.min(m.secs);
                    let name = sub.map_or("shield".to_string(), |w| format!("{w:?}"));
                    lines.push(format!(
                        "{name:<14} | {p:.2} | {:>3.0}% | {:>9.1} |",
                        m.win * 100.0,
                        m.secs
                    ));
                }
                let d = (best - bare.secs) / best * 100.0;
                worst = worst.max(d);
                for l in lines {
                    println!("{l}");
                }
                println!(
                    "{:<14} | {p:.2} | {:>3.0}% | {:>9.1} | {:+.1}% faster",
                    "EMPTY",
                    bare.win * 100.0,
                    bare.secs,
                    d
                );
            }
        }
    }
    println!("\nworst: the empty hand is {worst:+.1}% faster than the best off hand");
}
