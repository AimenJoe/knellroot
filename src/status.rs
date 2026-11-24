//! 피해 타입(0223 V2 → 0270 B): 물리 + 여덟 — 잔불(화상)·뿌리(묶임)·울림(퍼짐)·균열(취약) + 충격(기절)·날카로움(움직이면 샌다)·
//! 서리(느려짐 → 얼어붙음)·흡수(치면 돌아온다). 타입마다 **동사가 다르다**(숫자만 다른 색이 아니다).
//! 걸리는 법은 하나다: 타입이 있는 타가 적의 **게이지**를 쌓고, 가득 차면(1) 그 타입의 일이 일어난다 — 면역은 없다(배율만),
//! 보스·엘리트는 덜 쌓이고, 묶임·균열·기절·얼어붙음은 걸릴수록 다음이 더디다(되풀이 묶기를 막는다).
//!
//! 이 파일은 순수한 규칙이다(적 하나의 상태 `Ail` — 쌓기 `build`·시간 `tick`). 이웃에게 번지는 일(울림·터짐)은 월드가 한다
//! (`world/ail_tick.rs`). 숫자는 `tuning::AIL_*`.
//!
//! 겹침 덤 넷(늘 켜져 있다 — 타입 둘을 섞으면 일어나는 일):
//! - 묶인 적에게 붙는 화상은 한 겹 더(`build`).
//! - 화상 중인 적이 묶이면 터진다 — 겹을 다 쓰고 둘레를 친다(`Proc::Burst`).
//! - 타는 적이 울리면 둘레에 불이 옮는다 · 금 간 적이 울리면 둘레에도 금이 번진다(월드 `toll`).
//! - 얼어붙은 적을 치면 깨진다 — 그 한 대가 크게 들어가고 얼음이 풀린다(월드 `hurt_by`·`shatter`).
//! - 타는 적에게는 서리가 덜 붙고(녹는다), 언 적에게 붙는 불은 얼음을 녹인다 — 둘은 서로 지운다(`build`).

use crate::tuning::{
    AIL_BOSS, AIL_DECAY, AIL_ELITE, AIL_HOLD, AIL_WEAR, AIL_WEAR_MIN, BLEED_SECS, BLEED_STEP,
    BURN_BOSS_STACKS, BURN_EVERY, BURN_SECS, BURN_STACKS, DRAIN_SECS, FREEZE_BOSS_SECS,
    FREEZE_SECS, FROST_EMBER, FROST_SLOW, RIFT_SECS, ROOT_SECS, STUN_BOSS_SECS, STUN_SECS,
    TOLL_DELAY, TOLL_REST,
};

/// 타의 타입. 물리는 게이지가 없다(자세를 깬다 — 지금 그대로).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Kind {
    #[default]
    Physical,
    /// 잔불: 겹쳐 타는 화상(초마다, 겹은 셋까지).
    Ember,
    /// 뿌리: 묶는다(걷지 못한다 — 보스는 느려지기만), 묶인 적은 자세가 더 깎인다.
    Root,
    /// 울림: 울려서 둘레를 치고 게이지를 나눠 준다(떼에 강하다).
    Knell,
    /// 균열: 금이 가서 모든 피해를 더 받는다.
    Rift,
    /// 충격(0270 B — 망치): 기절시킨다(잠깐 아무것도 못 한다 — 보스는 짧게), 자세가 더 깎인다.
    Shock,
    /// 날카로움(0270 B — 결투): 걸린 동안 움직이는 만큼 피가 샌다(서 있으면 멎는다). 피 흘리는 동안 결투의 손이 치면 더 다친다
    /// (C5 — `tuning::SHARP_CUT_MULT`, 월드 `hurt_by`).
    Sharp,
    /// 서리(0270 B — 지팡이·원거리): 쌓일수록 느려지고, 가득 차면 얼어붙는다(얼면 다음 한 대가 크다).
    Frost,
    /// 흡수(0270 B — 방패·결투): 걸린 적을 치면 체력·기력이 조금 돌아온다(한 틱에 한 번).
    Drain,
}

/// 게이지가 있는 타입 여덟(자리 = `Kind::slot`).
pub const TYPED: [Kind; 8] = [
    Kind::Ember,
    Kind::Root,
    Kind::Knell,
    Kind::Rift,
    Kind::Shock,
    Kind::Sharp,
    Kind::Frost,
    Kind::Drain,
];
pub const N: usize = TYPED.len();

impl Kind {
    /// 게이지의 자리(물리는 없다).
    pub fn slot(self) -> Option<usize> {
        match self {
            Kind::Physical => None,
            Kind::Ember => Some(0),
            Kind::Root => Some(1),
            Kind::Knell => Some(2),
            Kind::Rift => Some(3),
            Kind::Shock => Some(4),
            Kind::Sharp => Some(5),
            Kind::Frost => Some(6),
            Kind::Drain => Some(7),
        }
    }

    /// 걸릴수록 다음이 더딘 타입(몸을 붙드는 것 — 되풀이로 묶어 두지 못하게).
    pub fn wears(self) -> bool {
        matches!(self, Kind::Root | Kind::Rift | Kind::Shock | Kind::Frost)
    }
}

/// 타입의 빛(잔불 호박 · 뿌리 민트 · 울림 쇳빛 · 균열 하늘빛 · 충격 레몬 · 날카로움 은백 · 서리 얼음 청록 · 흡수 풀빛 — `TYPED` 순, rgb).
/// 빨강은 쓰지 않는다(위험 예고 전용, 0141). 빛이 가까운 것끼리는 머리 위 표지의 모양이 다르다(`MARK`).
pub const INK: [[f32; 3]; N] = [
    [1.0, 0.72, 0.32],
    [0.66, 1.0, 0.86],
    [0.80, 0.84, 0.94],
    [0.50, 0.80, 1.0],
    [1.0, 0.96, 0.45],
    [0.95, 0.95, 1.0],
    [0.55, 0.95, 1.0],
    [0.55, 0.95, 0.45],
];

/// 머리 위 표지의 모양(`TYPED` 순) — 색약에도 갈리게(0150).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    Gem,
    Star,
    Shard,
    Drop,
}
pub const MARK: [Mark; N] = [
    Mark::Gem,
    Mark::Gem,
    Mark::Gem,
    Mark::Gem,
    Mark::Star,
    Mark::Shard,
    Mark::Shard,
    Mark::Drop,
];

/// 타 하나가 싣는 타입과 쌓는 양(보통 적의 게이지 1 가운데 몫).
pub type Dose = (Kind, f32);
pub const NO_DOSE: Dose = (Kind::Physical, 0.0);
/// 울린 직후의 게이지가 머무는 곳(가득 직전).
const TOLL_BRIM: f32 = 0.99;

/// 쌓이는 쪽의 사정: 보스·엘리트(덜 쌓인다)와 종의 배율(0.5~1.5 — 면역 없음).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Who {
    pub boss: bool,
    pub elite: bool,
    pub mult: [f32; N],
}

impl Who {
    #[cfg(test)]
    pub const PLAIN: Who = Who {
        boss: false,
        elite: false,
        mult: [1.0; N],
    };
}

/// 게이지가 가득 차서 일어난 일.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Proc {
    /// 화상이 붙었다(지금 겹).
    Burn(u8),
    /// 묶였다(보스는 느려진다).
    Bind,
    /// 화상 중에 묶여 터진다(쓴 겹 — 월드가 둘레를 친다).
    Burst(u8),
    /// 곧 울린다(`TOLL_DELAY` 뒤 — 월드가 둘레를 친다).
    Toll,
    /// 금이 갔다.
    Rift,
    /// 기절했다(초 — 월드가 휘청으로 세운다).
    Stun(f32),
    /// 피가 샌다.
    Bleed,
    /// 얼어붙었다(초 — 월드가 휘청으로 세운다).
    Freeze(f32),
    /// 흡수 표지가 붙었다.
    Drain,
}

/// 한 틱에 적에게서 일어난 일(월드가 피해·번짐으로 옮긴다).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Tick {
    /// 화상이 탄 겹(0 = 이번 틱은 안 탔다).
    pub burn: u8,
    /// 울린다.
    pub toll: bool,
    /// 화상 중에 묶여 터진다(쓴 겹).
    pub burst: u8,
}

/// 날카로움이 새는 걸음(`Ail::walked` — 움직인 거리 `BLEED_STEP` m마다 한 번).
pub type Pulses = u8;

/// 적 하나의 타입 상태.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Ail {
    /// 타입마다의 게이지(0..1).
    pub gauge: [f32; N],
    /// 타입마다 마지막으로 쌓인 뒤의 초(`AIL_HOLD` 뒤부터 빠진다).
    quiet: [f32; N],
    /// 타입마다 걸린 횟수(묶임·균열은 걸릴수록 덜 쌓인다).
    procs: [u8; N],
    /// 화상 겹·남은 초·다음에 탈 때까지.
    pub burn: u8,
    pub burn_left: f32,
    burn_next: f32,
    /// 묶인 초(남은).
    pub bound: f32,
    /// 금 간 초(남은).
    pub rift: f32,
    /// 울리기까지(없으면 None).
    pub toll_in: Option<f32>,
    /// 울린 뒤 다시 울릴 수 있을 때까지(그동안 게이지는 가득 직전에 머문다 — 떼 안에서 서로 끝없이 울리지 않게).
    toll_rest: f32,
    /// 터지기를 기다리는 겹(다음 틱에 월드가 둘레를 친다).
    burst: u8,
    /// 기절한 초(남은 — 0270 B 충격).
    pub stun: f32,
    /// 피가 새는 초(남은)와 다음 새기까지 걸은 거리(날카로움).
    pub bleed: f32,
    walked: f32,
    /// 지난번 `walked_to`의 자리(x, z) — 그 사이 걸은 거리를 잰다.
    last_at: Option<(f32, f32)>,
    /// 얼어붙은 초(남은 — 서리).
    pub frozen: f32,
    /// 흡수 표지의 초(남은).
    pub drain: f32,
    /// 이번 틱에 막 얼었다(0270 C3) — 얼린 그 타가 곧바로 얼음을 깨지 않게(다음 타부터 깬다). 월드가 틱마다 `settle_ice`로 걷는다.
    fresh_ice: bool,
}

impl Ail {
    /// 이 적에게 `kind`가 쌓이는 배율: 종 × 보스·엘리트 × 닳음(묶임·균열만 — 걸린 횟수마다 `AIL_WEAR`, 바닥 `AIL_WEAR_MIN`).
    pub fn rate(&self, kind: Kind, who: &Who) -> f32 {
        let Some(i) = kind.slot() else {
            return 0.0;
        };
        let rank = if who.boss {
            AIL_BOSS
        } else if who.elite {
            AIL_ELITE
        } else {
            1.0
        };
        let wear = if kind.wears() {
            AIL_WEAR.powi(self.procs[i] as i32).max(AIL_WEAR_MIN)
        } else {
            1.0
        };
        who.mult[i] * rank * wear
    }

    /// `kind`를 `amount`만큼 쌓는다. 가득 차면 게이지를 비우고 그 타입의 일을 돌려준다(남은 양은 버린다 — 한 타에 한 번).
    pub fn build(&mut self, kind: Kind, amount: f32, who: &Who) -> Option<Proc> {
        let i = kind.slot()?;
        if amount <= 0.0 {
            return None;
        }
        // 서리와 불은 서로 지운다: 타는 적에게는 서리가 덜 붙고, 언 적에게 붙는 불은 얼음을 녹인다.
        let melt = if kind == Kind::Frost && self.burn > 0 {
            FROST_EMBER
        } else {
            1.0
        };
        if kind == Kind::Ember && self.frozen > 0.0 {
            self.frozen = 0.0;
        }
        self.gauge[i] += amount * self.rate(kind, who) * melt;
        self.quiet[i] = 0.0;
        if kind == Kind::Knell && (self.toll_rest > 0.0 || self.toll_in.is_some()) {
            self.gauge[i] = self.gauge[i].min(TOLL_BRIM);
        }
        if self.gauge[i] < 1.0 {
            return None;
        }
        self.gauge[i] = 0.0;
        self.procs[i] = self.procs[i].saturating_add(1);
        Some(match kind {
            Kind::Physical => return None,
            Kind::Ember => {
                // 겹침: 묶인 적에게는 한 겹 더.
                let more = if self.bound > 0.0 { 2 } else { 1 };
                Proc::Burn(self.kindle(more, who.boss))
            }
            Kind::Root => {
                self.bound = ROOT_SECS;
                // 겹침: 화상 중이면 겹을 다 쓰고 터진다.
                if self.burn > 0 {
                    let n = self.burn;
                    self.burn = 0;
                    self.burn_left = 0.0;
                    self.burst = n;
                    Proc::Burst(n)
                } else {
                    Proc::Bind
                }
            }
            Kind::Knell => {
                self.toll_in = Some(TOLL_DELAY);
                Proc::Toll
            }
            Kind::Rift => {
                self.rift = RIFT_SECS;
                Proc::Rift
            }
            Kind::Shock => {
                let secs = if who.boss { STUN_BOSS_SECS } else { STUN_SECS };
                self.stun = secs;
                Proc::Stun(secs)
            }
            Kind::Sharp => {
                self.bleed = BLEED_SECS;
                Proc::Bleed
            }
            Kind::Frost => {
                let secs = if who.boss {
                    FREEZE_BOSS_SECS
                } else {
                    FREEZE_SECS
                };
                self.frozen = secs;
                self.fresh_ice = true;
                Proc::Freeze(secs)
            }
            Kind::Drain => {
                self.drain = DRAIN_SECS;
                Proc::Drain
            }
        })
    }

    /// 서리가 걷는 빠르기에 곱하는 배(쌓인 게이지만큼 느려진다 — 얼면 0, 1 = 그대로).
    pub fn chill(&self) -> f32 {
        if self.frozen > 0.0 {
            0.0
        } else {
            1.0 - FROST_SLOW * self.gauge[6].min(1.0)
        }
    }

    /// 날카로움: 이번 틱에 `dist` m 움직였다 — 피가 새는 동안 `BLEED_STEP` m마다 한 번씩 돌려준다(서 있으면 멎는다).
    pub fn walked(&mut self, dist: f32) -> Pulses {
        if self.bleed <= 0.0 {
            self.walked = 0.0;
            return 0;
        }
        self.walked += dist.max(0.0);
        let n = (self.walked / BLEED_STEP).floor();
        self.walked -= n * BLEED_STEP;
        n.min(3.0) as Pulses
    }

    /// `walked`를 자리로: 지난번 부른 자리에서 (x, z)까지 걸은 만큼(틱의 차례와 상관없이 — 월드는 걷기 전에 부른다).
    pub fn walked_to(&mut self, x: f32, z: f32) -> (Pulses, f32, f32) {
        let (dx, dz) = match self.last_at.replace((x, z)) {
            Some((px, pz)) => (x - px, z - pz),
            None => (0.0, 0.0),
        };
        (self.walked((dx * dx + dz * dz).sqrt()), dx, dz)
    }

    /// 언 적이 깨진다(친 한 대가 크게 들어간 뒤 — 월드가 부른다). 얼어 있었으면 참.
    pub fn shatter(&mut self) -> bool {
        if self.fresh_ice {
            return false;
        }
        let was = self.frozen > 0.0;
        self.frozen = 0.0;
        was
    }

    /// 막 언 얼음이 굳는다(틱마다 — `shatter` 뒤에).
    pub fn settle_ice(&mut self) {
        self.fresh_ice = false;
    }

    /// 화상 `more`겹을 붙인다(상한까지 — 보스는 둘) — 남은 시간은 처음부터. 지금 겹을 돌려준다.
    pub fn kindle(&mut self, more: u8, boss: bool) -> u8 {
        let cap = if boss { BURN_BOSS_STACKS } else { BURN_STACKS };
        if self.burn == 0 {
            self.burn_next = BURN_EVERY;
        }
        self.burn = (self.burn + more).min(cap);
        self.burn_left = BURN_SECS;
        self.burn
    }

    /// 금을 `secs`초 낸다(이미 더 길게 금 가 있으면 그대로) — 울림이 번지게 하는 금.
    pub fn crack(&mut self, secs: f32) {
        self.rift = self.rift.max(secs);
    }

    /// 시간이 흐른다: 화상이 타고, 묶임·금이 풀리고, 울릴 때가 오고, 한동안 안 쌓인 게이지가 빠진다.
    pub fn tick(&mut self, dt: f32) -> Tick {
        let mut out = Tick::default();
        if self.burn > 0 {
            self.burn_next -= dt;
            if self.burn_next <= 0.0 {
                self.burn_next += BURN_EVERY;
                out.burn = self.burn;
            }
            self.burn_left -= dt;
            if self.burn_left <= 0.0 {
                self.burn = 0;
                self.burn_left = 0.0;
            }
        }
        self.bound = (self.bound - dt).max(0.0);
        self.rift = (self.rift - dt).max(0.0);
        self.stun = (self.stun - dt).max(0.0);
        self.bleed = (self.bleed - dt).max(0.0);
        self.frozen = (self.frozen - dt).max(0.0);
        self.drain = (self.drain - dt).max(0.0);
        self.toll_rest = (self.toll_rest - dt).max(0.0);
        out.burst = std::mem::take(&mut self.burst);
        if let Some(left) = self.toll_in {
            if left - dt <= 0.0 {
                self.toll_in = None;
                self.toll_rest = TOLL_REST;
                out.toll = true;
            } else {
                self.toll_in = Some(left - dt);
            }
        }
        for i in 0..N {
            self.quiet[i] += dt;
            if self.quiet[i] > AIL_HOLD {
                self.gauge[i] = (self.gauge[i] - AIL_DECAY * dt).max(0.0);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 60.0;

    fn run(a: &mut Ail, secs: f32) -> (u32, u32) {
        let (mut burns, mut tolls) = (0, 0);
        for _ in 0..(secs / DT).round() as u32 {
            let t = a.tick(DT);
            burns += t.burn as u32;
            tolls += t.toll as u32;
        }
        (burns, tolls)
    }

    #[test]
    fn physical_blows_build_nothing() {
        let mut a = Ail::default();
        assert_eq!(a.build(Kind::Physical, 5.0, &Who::PLAIN), None);
        assert_eq!(a, Ail::default());
    }

    #[test]
    fn a_full_gauge_fires_once_and_empties() {
        let mut a = Ail::default();
        assert_eq!(a.build(Kind::Rift, 0.6, &Who::PLAIN), None);
        assert!(a.rift == 0.0 && a.gauge[3] > 0.5);
        assert_eq!(a.build(Kind::Rift, 0.6, &Who::PLAIN), Some(Proc::Rift));
        assert_eq!(a.gauge[3], 0.0);
        assert_eq!(a.rift, RIFT_SECS);
    }

    #[test]
    fn burn_stacks_to_three_and_ticks_every_second_until_it_runs_out() {
        let mut a = Ail::default();
        for want in [1, 2, 3, 3] {
            assert_eq!(
                a.build(Kind::Ember, 1.0, &Who::PLAIN),
                Some(Proc::Burn(want))
            );
        }
        // 세 겹이 `BURN_SECS` 동안 초마다: 겹 × 번.
        let (burns, _) = run(&mut a, BURN_SECS + 1.0);
        let pulses = (BURN_SECS / BURN_EVERY).floor() as u32;
        assert_eq!(burns, 3 * pulses);
        assert_eq!(a.burn, 0);
    }

    #[test]
    fn a_boss_burns_two_deep_at_most_and_builds_slower() {
        let boss = Who {
            boss: true,
            ..Who::PLAIN
        };
        let mut a = Ail::default();
        assert_eq!(a.build(Kind::Ember, 1.0, &boss), None, "half as fast");
        assert_eq!(a.build(Kind::Ember, 1.0, &boss), Some(Proc::Burn(1)));
        for _ in 0..6 {
            a.build(Kind::Ember, 1.0, &boss);
        }
        assert_eq!(a.burn, BURN_BOSS_STACKS);
    }

    #[test]
    fn binding_and_cracking_wear_thin_but_never_to_nothing() {
        let mut a = Ail::default();
        let mut last = f32::MAX;
        for _ in 0..12 {
            let r = a.rate(Kind::Root, &Who::PLAIN);
            assert!(r <= last && r >= AIL_WEAR_MIN - 1e-6);
            last = r;
            while a.build(Kind::Root, 0.5, &Who::PLAIN).is_none() {}
        }
        assert!((last - AIL_WEAR_MIN).abs() < 1e-6, "settles at the floor");
        // 화상·울림은 닳지 않는다.
        for _ in 0..5 {
            a.build(Kind::Ember, 1.0, &Who::PLAIN);
        }
        assert_eq!(a.rate(Kind::Ember, &Who::PLAIN), 1.0);
    }

    #[test]
    fn no_species_is_immune() {
        // 배율은 0.5~1.5(시험 `species`가 표를 본다) — 가장 낮은 배율의 보스도 쌓다 보면 걸린다.
        let hard = Who {
            boss: true,
            elite: false,
            mult: [0.5; N],
        };
        for k in TYPED {
            let mut a = Ail::default();
            let mut n = 0;
            while a.build(k, 0.5, &hard).is_none() {
                n += 1;
                assert!(n < 40, "{k:?}");
            }
        }
    }

    #[test]
    fn a_gauge_left_alone_drains_after_a_hold() {
        let mut a = Ail::default();
        a.build(Kind::Knell, 0.5, &Who::PLAIN);
        run(&mut a, AIL_HOLD - 0.1);
        assert!((a.gauge[2] - 0.5).abs() < 1e-4, "held");
        run(&mut a, 0.5 / AIL_DECAY + 0.3);
        assert_eq!(a.gauge[2], 0.0);
    }

    #[test]
    fn the_toll_comes_a_beat_after_the_gauge_fills() {
        let mut a = Ail::default();
        assert_eq!(a.build(Kind::Knell, 1.0, &Who::PLAIN), Some(Proc::Toll));
        let (_, early) = run(&mut a, TOLL_DELAY - 2.0 * DT);
        assert_eq!(early, 0);
        let (_, tolls) = run(&mut a, 4.0 * DT);
        assert_eq!(tolls, 1);
        assert_eq!(a.toll_in, None);
    }

    #[test]
    fn a_foe_that_just_rang_cannot_ring_again_until_it_has_rested() {
        let mut a = Ail::default();
        a.build(Kind::Knell, 1.0, &Who::PLAIN);
        assert_eq!(
            a.build(Kind::Knell, 5.0, &Who::PLAIN),
            None,
            "already about to ring"
        );
        assert_eq!(run(&mut a, 0.5).1, 1);
        for _ in 0..5 {
            assert_eq!(a.build(Kind::Knell, 1.0, &Who::PLAIN), None);
        }
        assert!(a.gauge[2] < 1.0);
        run(&mut a, TOLL_REST);
        assert_eq!(a.build(Kind::Knell, 1.0, &Who::PLAIN), Some(Proc::Toll));
    }

    #[test]
    fn overlap_a_bound_foe_catches_fire_two_deep() {
        let mut a = Ail::default();
        assert_eq!(a.build(Kind::Root, 1.0, &Who::PLAIN), Some(Proc::Bind));
        assert_eq!(a.build(Kind::Ember, 1.0, &Who::PLAIN), Some(Proc::Burn(2)));
    }

    #[test]
    fn shock_stuns_and_a_boss_shakes_it_off_sooner() {
        let mut a = Ail::default();
        assert_eq!(
            a.build(Kind::Shock, 1.0, &Who::PLAIN),
            Some(Proc::Stun(STUN_SECS))
        );
        let boss = Who {
            boss: true,
            ..Who::PLAIN
        };
        let mut b = Ail::default();
        while b.build(Kind::Shock, 1.0, &boss).is_none() {}
        assert!(b.stun < a.stun && b.stun > 0.0);
        run(&mut a, STUN_SECS + 0.1);
        assert_eq!(a.stun, 0.0);
        // 몸을 붙드는 타입은 닳는다.
        assert!(a.rate(Kind::Shock, &Who::PLAIN) < 1.0);
    }

    #[test]
    fn a_bleeding_foe_leaks_as_it_moves_and_stops_when_it_stands() {
        let mut a = Ail::default();
        assert_eq!(a.walked(10.0), 0, "no cut, no leak");
        assert_eq!(a.build(Kind::Sharp, 1.0, &Who::PLAIN), Some(Proc::Bleed));
        assert_eq!(a.walked(0.0), 0, "standing still");
        let mut n = 0;
        for _ in 0..60 {
            n += a.walked(BLEED_STEP / 20.0) as u32;
        }
        assert_eq!(n, 3, "three steps' worth of walking");
        run(&mut a, BLEED_SECS + 0.1);
        assert_eq!(a.walked(BLEED_STEP * 2.0), 0, "the cut has closed");
    }

    #[test]
    fn frost_slows_as_it_builds_then_freezes_and_shatters_once() {
        let mut a = Ail::default();
        assert_eq!(a.chill(), 1.0);
        a.build(Kind::Frost, 0.5, &Who::PLAIN);
        let half = a.chill();
        assert!(half < 1.0 && half > 1.0 - FROST_SLOW);
        assert_eq!(
            a.build(Kind::Frost, 0.6, &Who::PLAIN),
            Some(Proc::Freeze(FREEZE_SECS))
        );
        assert_eq!(a.chill(), 0.0, "frozen stiff");
        // 막 언 얼음은 얼린 그 타에 깨지지 않는다(0270 C3) — 굳은 뒤에.
        assert!(!a.shatter(), "not by the freezing blow");
        a.settle_ice();
        assert!(a.shatter());
        assert!(!a.shatter(), "only once");
        assert_eq!(a.chill(), 1.0, "the gauge emptied when it froze");
    }

    #[test]
    fn fire_and_frost_undo_each_other() {
        let mut a = Ail::default();
        a.build(Kind::Ember, 1.0, &Who::PLAIN);
        a.build(Kind::Frost, 0.5, &Who::PLAIN);
        assert!(
            (a.gauge[6] - 0.5 * FROST_EMBER).abs() < 1e-5,
            "a burning foe chills slower"
        );
        let mut b = Ail::default();
        b.build(Kind::Frost, 1.0, &Who::PLAIN);
        assert!(b.frozen > 0.0);
        b.build(Kind::Ember, 0.3, &Who::PLAIN);
        assert_eq!(b.frozen, 0.0, "fire thaws the ice");
    }

    #[test]
    fn drain_marks_a_foe_for_a_while() {
        let mut a = Ail::default();
        assert_eq!(a.build(Kind::Drain, 1.0, &Who::PLAIN), Some(Proc::Drain));
        assert_eq!(a.drain, DRAIN_SECS);
        run(&mut a, DRAIN_SECS + 0.1);
        assert_eq!(a.drain, 0.0);
        assert_eq!(a.rate(Kind::Drain, &Who::PLAIN), 1.0, "drain does not wear");
    }

    #[test]
    fn overlap_binding_a_burning_foe_spends_the_fire_in_a_burst() {
        let mut a = Ail::default();
        a.build(Kind::Ember, 1.0, &Who::PLAIN);
        a.build(Kind::Ember, 1.0, &Who::PLAIN);
        assert_eq!(a.build(Kind::Root, 1.0, &Who::PLAIN), Some(Proc::Burst(2)));
        assert_eq!(a.burn, 0);
        assert!(a.bound > 0.0, "still bound");
        assert_eq!(a.tick(DT).burst, 2, "the world bursts it on the next tick");
        assert_eq!(a.tick(DT).burst, 0);
    }
}
