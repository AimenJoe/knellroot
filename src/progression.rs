//! 성장 v0.4.0 "자라는 방랑자"(0095·0096·0102): **잔불 레벨** — 생명·기력·힘 셋, 각 다섯 단(0104 G1).
//! 기술 나무(G2)는 뒤에 이 파일에 더한다.
//!
//! 뜻(lore CANON, `docs/lore/growth.md`): 힘을 빨아들이는 게 아니라 몸과 싸우는 법을 되찾는 것 — 숫자 성장은 여기,
//! 동작 성장은 나무. 값·단은 판정(0096 L1·L2)에서 고친다 — 표 한 곳.
use crate::stats::Mods;

/// 레벨 줄.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Line {
    /// 버티는 힘 — 최대 체력 +1/단.
    Life,
    /// 움직이는 힘 — 최대 스태미나 +12/단, 3·5단에 차는 빠르기 +10%.
    Stamina,
    /// 맞서는 힘 — 자세 피해 +12%/단, 3단 강타 +1, 5단 평타 +1(정수 피해를 지킨다, 0095 원칙 3).
    Might,
}

pub const LINES: [Line; 3] = [Line::Life, Line::Stamina, Line::Might];

/// 줄마다 단 수.
pub const MAX_RANK: u8 = 5;
/// 다음 단 값(잔불, 줄마다 같은 표 — 0095·0102): 줄당 350, 셋 1,050. 한 판 벌이 ≈ 70~100.
pub const COSTS: [u32; MAX_RANK as usize] = [30, 45, 65, 90, 120];
pub const STAMINA_PER_RANK: f32 = 12.0;
/// 차는 빠르기가 오르는 단(+10%씩, 곱).
pub const REGEN_RANKS: [u8; 2] = [3, 5];
pub const REGEN_STEP: f32 = 0.10;
pub const POISE_PER_RANK: f32 = 0.12;
/// 강타 +1이 붙는 단·평타 +1이 붙는 단.
pub const HEAVY_RANK: u8 = 3;
pub const LIGHT_RANK: u8 = 5;

/// 문지기 핵 점수(0102 5절·0105): 이길 때마다 1점, 문지기마다 셋까지 — 기술 나무(G2) 점수의 한 몫(나머지는 종소리 조각).
pub const CORE_CAP: u32 = 3;

/// 숨은 물건(0105)의 보정: 등잔 +1회 · 한 모금 +1 — 기록 이름(`<스테이지>.<종류>.<n>`)의 가운데 칸으로 센다.
pub fn find_mods(ids: &[String]) -> Mods {
    let count = |k: &str| {
        ids.iter()
            .filter(|id| id.split('.').nth(1) == Some(k))
            .count() as i32
    };
    Mods {
        flask: count("lamp"),
        heal: count("sip"),
        ..Mods::default()
    }
}

/// 올린 단(기록 `lv.*`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Level {
    pub life: u8,
    pub stamina: u8,
    pub might: u8,
}

impl Level {
    pub fn rank(&self, l: Line) -> u8 {
        match l {
            Line::Life => self.life,
            Line::Stamina => self.stamina,
            Line::Might => self.might,
        }
    }

    /// 레벨 숫자 = 세 줄 단의 합(0~15).
    pub fn total(&self) -> u32 {
        LINES.iter().map(|&l| self.rank(l) as u32).sum()
    }

    /// 다음 단 값(다 올렸으면 없음).
    pub fn next_cost(&self, l: Line) -> Option<u32> {
        COSTS.get(self.rank(l) as usize).copied()
    }

    /// 한 단 올린 사본(다 올렸으면 그대로).
    pub fn raised(&self, l: Line) -> Level {
        let mut o = *self;
        let r = match l {
            Line::Life => &mut o.life,
            Line::Stamina => &mut o.stamina,
            Line::Might => &mut o.might,
        };
        *r = (*r + 1).min(MAX_RANK);
        o
    }

    /// 잔불로 올릴 수 있는 줄이 있나(쉼터가 빛난다 — KB "쓸 곳이 보여야").
    pub fn can_raise(&self, embers: u32) -> bool {
        LINES
            .iter()
            .any(|&l| self.next_cost(l).is_some_and(|c| embers >= c))
    }

    /// 능력치 보정(장신구·핵과 같은 `Stats::compute`로).
    pub fn mods(&self) -> Mods {
        let regen = REGEN_RANKS.iter().filter(|&&r| self.stamina >= r).count() as f32;
        Mods {
            hp: self.life as i32,
            stamina_max: STAMINA_PER_RANK * self.stamina as f32,
            stamina_regen: 1.0 + REGEN_STEP * regen,
            poise: 1.0 + POISE_PER_RANK * self.might as f32,
            heavy_dmg: (self.might >= HEAVY_RANK) as i32,
            light_dmg: (self.might >= LIGHT_RANK) as i32,
            ..Mods::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::Stats;

    #[test]
    fn levels_cost_the_table_stop_at_five_and_sum_to_the_level_number() {
        let mut lv = Level::default();
        let mut paid = 0;
        for l in LINES {
            while let Some(c) = lv.next_cost(l) {
                paid += c;
                lv = lv.raised(l);
            }
            assert_eq!(lv.rank(l), MAX_RANK);
            assert_eq!(lv.raised(l), lv, "a full line stays full");
        }
        assert_eq!(paid, 3 * COSTS.iter().sum::<u32>());
        assert_eq!(paid, 1_050, "0102: three lines of 30·45·65·90·120");
        assert_eq!(lv.total(), 15);
        assert!(!lv.can_raise(u32::MAX));
        assert!(Level::default().can_raise(COSTS[0]));
        assert!(!Level::default().can_raise(COSTS[0] - 1));
    }

    #[test]
    fn each_line_feeds_its_own_stats_and_might_keeps_damage_whole() {
        let base = Stats::default();
        let life = Stats::compute(&[Level {
            life: 5,
            ..Level::default()
        }
        .mods()]);
        assert_eq!(life.max_hp, base.max_hp + 5);
        let st = |r| {
            Stats::compute(&[Level {
                stamina: r,
                ..Level::default()
            }
            .mods()])
        };
        assert_eq!(st(1).stamina_max, 12.0);
        assert_eq!(st(2).stamina_regen, 1.0, "regen waits for rank 3");
        assert!((st(3).stamina_regen - 1.1).abs() < 1e-6);
        assert!((st(5).stamina_regen - 1.2).abs() < 1e-6);
        assert_eq!(st(5).stamina_max, 60.0);
        let mt = |r| {
            Stats::compute(&[Level {
                might: r,
                ..Level::default()
            }
            .mods()])
        };
        assert!((mt(1).poise - 1.12).abs() < 1e-6);
        assert_eq!((mt(2).heavy_dmg, mt(2).light_dmg), (0, 0));
        assert_eq!((mt(3).heavy_dmg, mt(3).light_dmg), (1, 0));
        assert_eq!((mt(5).heavy_dmg, mt(5).light_dmg), (1, 1));
        // 레벨은 판정 창을 건드리지 않는다(0095 원칙 2, 0102).
        for r in 0..=MAX_RANK {
            let lv = Level {
                life: r,
                stamina: r,
                might: r,
            };
            let s = Stats::compute(&[lv.mods()]);
            assert_eq!(s.parry, 0.0);
            assert_eq!(s.atk_speed, 1.0);
        }
    }

    #[test]
    fn finds_count_by_kind_whatever_the_stage() {
        let ids = vec![
            "sanctuary.lamp.0".to_string(),
            "sanctuary.sip.0".to_string(),
            "hamlet.sip.0".to_string(),
        ];
        let s = Stats::compute(&[find_mods(&ids)]);
        assert_eq!(s.flask, Stats::default().flask + 1);
        assert_eq!(s.heal, 2);
        assert_eq!(find_mods(&[]).flask, 0);
    }

    #[test]
    fn the_shell_charm_still_counts_at_full_life() {
        // 0104: 생명 5단 + 껍질 = 상한 안.
        let full = Level {
            life: MAX_RANK,
            ..Level::default()
        };
        let shell = crate::stats::Mods {
            hp: 1,
            ..Mods::default()
        };
        let a = Stats::compute(&[full.mods()]);
        let b = Stats::compute(&[full.mods(), shell]);
        assert_eq!(b.max_hp, a.max_hp + 1);
    }
}
