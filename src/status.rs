//! 피해 타입(0223 V2): 물리 + 넷 — 잔불(화상)·뿌리(묶임)·울림(퍼짐)·균열(취약). 타입마다 **동사가 다르다**(숫자만 다른 색이 아니다).
//! 걸리는 법은 하나다: 타입이 있는 타가 적의 **게이지**를 쌓고, 가득 차면(1) 그 타입의 일이 일어난다 — 면역은 없다(배율만),
//! 보스·엘리트는 덜 쌓이고, 묶임·균열은 걸릴수록 다음이 더디다(되풀이 묶기를 막는다).
//!
//! 이 파일은 순수한 규칙이다(적 하나의 상태 `Ail` — 쌓기 `build`·시간 `tick`). 이웃에게 번지는 일(울림·터짐)은 월드가 한다
//! (`world/ail_tick.rs`). 숫자는 `tuning::AIL_*`.
//!
//! 겹침 덤 넷(늘 켜져 있다 — 타입 둘을 섞으면 일어나는 일):
//! - 묶인 적에게 붙는 화상은 한 겹 더(`build`).
//! - 화상 중인 적이 묶이면 터진다 — 겹을 다 쓰고 둘레를 친다(`Proc::Burst`).
//! - 타는 적이 울리면 둘레에 불이 옮는다 · 금 간 적이 울리면 둘레에도 금이 번진다(월드 `toll`).

use crate::tuning::{
    AIL_BOSS, AIL_DECAY, AIL_ELITE, AIL_HOLD, AIL_WEAR, AIL_WEAR_MIN, BURN_BOSS_STACKS, BURN_EVERY,
    BURN_SECS, BURN_STACKS, RIFT_SECS, ROOT_SECS, TOLL_DELAY, TOLL_REST,
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
}

/// 게이지가 있는 타입 넷(자리 = `Kind::slot`).
pub const TYPED: [Kind; 4] = [Kind::Ember, Kind::Root, Kind::Knell, Kind::Rift];
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
        }
    }
}

/// 타입의 빛(잔불 호박 · 뿌리 민트 · 울림 쇳빛 · 균열 하늘빛 — `TYPED` 순, rgb). 빨강은 쓰지 않는다(위험 예고 전용, 0141).
pub const INK: [[f32; 3]; N] = [
    [1.0, 0.72, 0.32],
    [0.66, 1.0, 0.86],
    [0.80, 0.84, 0.94],
    [0.50, 0.80, 1.0],
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
        let wear = if matches!(kind, Kind::Root | Kind::Rift) {
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
        self.gauge[i] += amount * self.rate(kind, who);
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
        })
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
