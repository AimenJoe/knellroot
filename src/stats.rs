//! 능력치 합치기(0064 M3, KB `concepts/action-rpg-stats-and-equipment.md`): `Stats = 기본 + Σ장신구 + 성장` 한 곳.
//! 더하기(체력·등잔·피해)는 합, 배율(공격 속도·회피 비용·잔불)은 곱, 끝에 상한·하한으로 극단 조합을 막는다.
//! 월드는 `Stats`만 읽는다 — 체력·등잔 수는 여기서, 랩 값(회피 비용·패링 창·공격 시간)은 `tune`이 휘어서 넘긴다
//! (회피 비용을 쓰는 곳이 여덟이라 한 곳에서 휜다).
use crate::tuning::{Tuning, FLASK_CHARGES, P, PLAYER_HP};

/// 보정 하나(장신구 한 줄·성장 한 칸). 기본값 = 아무것도 안 바꿈.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mods {
    /// 최대 체력 +.
    pub hp: i32,
    /// 등잔 횟수 +.
    pub flask: i32,
    /// 등잔 한 모금 회복 +.
    pub heal: i32,
    /// 주는 피해 +(타마다).
    pub dmg: i32,
    /// 받는 피해 +(맞을 때마다).
    pub taken: i32,
    /// 공격 빠르기 배(1 = 그대로, 0.85 = 15% 느림 — 선딜·활성·후딜·모으기 시간이 1/배).
    pub atk_speed: f32,
    /// 회피 스태미나 배.
    pub dodge_cost: f32,
    /// 패링 창 +(초).
    pub parry: f32,
    /// 적이 쓰러질 때마다 스태미나 +.
    pub kill_stamina: f32,
    /// 잔불 배.
    pub ember: f32,
}

impl Default for Mods {
    fn default() -> Self {
        Mods {
            hp: 0,
            flask: 0,
            heal: 0,
            dmg: 0,
            taken: 0,
            atk_speed: 1.0,
            dodge_cost: 1.0,
            parry: 0.0,
            kill_stamina: 0.0,
            ember: 1.0,
        }
    }
}

/// 합친 능력치.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stats {
    pub max_hp: i32,
    pub flask: i32,
    pub heal: i32,
    pub dmg: i32,
    pub taken: i32,
    pub atk_speed: f32,
    pub dodge_cost: f32,
    pub parry: f32,
    pub kill_stamina: f32,
    pub ember: f32,
}

/// 상한·하한(극단 조합을 막는다 — KB).
const HP_RANGE: (i32, i32) = (1, 10);
const FLASK_RANGE: (i32, i32) = (0, 8);
const ATK_SPEED_RANGE: (f32, f32) = (0.6, 1.8);
const DODGE_COST_RANGE: (f32, f32) = (0.3, 2.0);
const PARRY_MAX: f32 = 0.15;

impl Default for Stats {
    fn default() -> Self {
        Stats::compute(&[])
    }
}

impl Stats {
    pub fn compute(mods: &[Mods]) -> Stats {
        let mut s = Stats {
            max_hp: PLAYER_HP,
            flask: FLASK_CHARGES,
            heal: 0,
            dmg: 0,
            taken: 0,
            atk_speed: 1.0,
            dodge_cost: 1.0,
            parry: 0.0,
            kill_stamina: 0.0,
            ember: 1.0,
        };
        for m in mods {
            s.max_hp += m.hp;
            s.flask += m.flask;
            s.heal += m.heal;
            s.dmg += m.dmg;
            s.taken += m.taken;
            s.atk_speed *= m.atk_speed;
            s.dodge_cost *= m.dodge_cost;
            s.parry += m.parry;
            s.kill_stamina += m.kill_stamina;
            s.ember *= m.ember;
        }
        s.max_hp = s.max_hp.clamp(HP_RANGE.0, HP_RANGE.1);
        s.flask = s.flask.clamp(FLASK_RANGE.0, FLASK_RANGE.1);
        s.heal = s.heal.max(0);
        s.dmg = s.dmg.max(0);
        s.taken = s.taken.max(0);
        s.atk_speed = s.atk_speed.clamp(ATK_SPEED_RANGE.0, ATK_SPEED_RANGE.1);
        s.dodge_cost = s.dodge_cost.clamp(DODGE_COST_RANGE.0, DODGE_COST_RANGE.1);
        s.parry = s.parry.clamp(0.0, PARRY_MAX);
        s.kill_stamina = s.kill_stamina.max(0.0);
        s.ember = s.ember.max(0.0);
        s
    }

    /// 랩 값을 이 능력치로 휜 사본(로직은 이것을 읽는다). 기본 능력치면 그대로.
    pub fn tune(&self, t: &Tuning) -> Tuning {
        let mut o = t.clone();
        if *self == Stats::default() {
            return o;
        }
        o.bend(P::StaminaDodge, t[P::StaminaDodge] * self.dodge_cost);
        o.bend(P::ParryWindow, t[P::ParryWindow] + self.parry);
        o.bend(P::FlaskHeal, t[P::FlaskHeal] + self.heal as f32);
        for p in [
            P::AttackStartup,
            P::AttackActive,
            P::AttackRecovery,
            P::HeavyCharge,
            P::HeavyStartup,
            P::HeavyActive,
            P::HeavyRecovery,
        ] {
            o.bend(p, t[p] / self.atk_speed);
        }
        o
    }
}

/// 장비 창의 상태 줄(0067): 무엇을 보이나·어느 쪽이 좋은가. 글·단위는 앱이(i18n).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Line {
    MaxHp,
    Flask,
    Heal,
    Dmg,
    Taken,
    AtkSpeed,
    DodgeCost,
    Parry,
    KillStamina,
    Ember,
}

pub const LINES: [Line; 10] = [
    Line::MaxHp,
    Line::Flask,
    Line::Heal,
    Line::Dmg,
    Line::Taken,
    Line::AtkSpeed,
    Line::DodgeCost,
    Line::Parry,
    Line::KillStamina,
    Line::Ember,
];

/// 바꾸면 어떻게 되나(좋아짐·나빠짐 — 화면은 색 + 화살표 + 숫자로, 색만으로 말하지 않는다).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Change {
    Same,
    Better,
    Worse,
}

impl Line {
    pub fn value(self, s: &Stats) -> f32 {
        match self {
            Line::MaxHp => s.max_hp as f32,
            Line::Flask => s.flask as f32,
            Line::Heal => s.heal as f32,
            Line::Dmg => s.dmg as f32,
            Line::Taken => s.taken as f32,
            Line::AtkSpeed => s.atk_speed,
            Line::DodgeCost => s.dodge_cost,
            Line::Parry => s.parry,
            Line::KillStamina => s.kill_stamina,
            Line::Ember => s.ember,
        }
    }

    /// 늘 보이는 줄(나머지는 보정이 있을 때만 — KB: 줄이 늘면 칸이 넘친다).
    pub fn always(self) -> bool {
        // 공격 속도는 무기 숫자 줄(한 스윙 초 = 박자)이 맡는다 — 여기는 장신구 배율이 있을 때만(2026-11-07).
        matches!(self, Line::MaxHp | Line::Flask)
    }

    /// 작을수록 좋은 줄(받는 피해·회피 기력).
    fn lower_is_better(self) -> bool {
        matches!(self, Line::Taken | Line::DodgeCost)
    }

    pub fn change(self, now: &Stats, then: &Stats) -> Change {
        let (a, b) = (self.value(now), self.value(then));
        if (a - b).abs() < 1e-4 {
            Change::Same
        } else if (b > a) != self.lower_is_better() {
            Change::Better
        } else {
            Change::Worse
        }
    }
}

/// 보일 줄: 늘 보이는 줄 + 지금(또는 바꾼 뒤) 기본값과 다른 줄. `then`이 있으면 줄마다 바꾼 뒤 값과 변화.
/// 상태 한 줄: 줄 · 지금 값 · (바꾼 뒤 값, 변화).
pub type Row = (Line, f32, Option<(f32, Change)>);

pub fn lines(now: &Stats, then: Option<&Stats>) -> Vec<Row> {
    let base = Stats::default();
    LINES
        .into_iter()
        .filter(|l| {
            let differs = |s: &Stats| (l.value(s) - l.value(&base)).abs() > 1e-4;
            l.always() || differs(now) || then.is_some_and(differs)
        })
        .map(|l| {
            (
                l,
                l.value(now),
                then.map(|t| (l.value(t), l.change(now, t))),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_add_sums_multiply_rates_and_clamp_extremes() {
        let base = Stats::default();
        assert_eq!((base.max_hp, base.flask), (PLAYER_HP, FLASK_CHARGES));
        let a = Mods {
            hp: 1,
            atk_speed: 0.85,
            ..Mods::default()
        };
        let b = Mods {
            hp: 2,
            dmg: 1,
            atk_speed: 0.85,
            ..Mods::default()
        };
        let s = Stats::compute(&[a, b]);
        assert_eq!(s.max_hp, PLAYER_HP + 3);
        assert_eq!(s.dmg, 1);
        assert!((s.atk_speed - 0.85 * 0.85).abs() < 1e-6, "rates multiply");
        // 극단은 묶인다.
        let silly = Mods {
            hp: 50,
            flask: -9,
            atk_speed: 0.1,
            parry: 1.0,
            ..Mods::default()
        };
        let s = Stats::compute(&[silly]);
        assert_eq!((s.max_hp, s.flask), (HP_RANGE.1, 0));
        assert_eq!(s.atk_speed, ATK_SPEED_RANGE.0);
        assert_eq!(s.parry, PARRY_MAX);
    }

    #[test]
    fn tuning_bends_dodge_cost_parry_window_and_attack_times() {
        let t = Tuning::default();
        assert_eq!(Stats::default().tune(&t), t, "default stats change nothing");
        let s = Stats::compute(&[Mods {
            dodge_cost: 0.7,
            parry: 0.06,
            atk_speed: 0.85,
            ..Mods::default()
        }]);
        let o = s.tune(&t);
        assert!((o[P::StaminaDodge] - t[P::StaminaDodge] * 0.7).abs() < 1e-4);
        assert!((o[P::ParryWindow] - (t[P::ParryWindow] + 0.06)).abs() < 1e-6);
        assert!(
            o[P::AttackRecovery] > t[P::AttackRecovery],
            "slower swings last longer"
        );
        assert_eq!(o[P::AttackRange], t[P::AttackRange], "reach untouched");
    }

    #[test]
    fn status_lines_show_the_basics_and_only_lines_that_a_change_touches() {
        let base = Stats::default();
        let shown: Vec<Line> = lines(&base, None).iter().map(|l| l.0).collect();
        // 공격 속도는 무기 숫자 줄(초)이 맡고, 배율은 장신구가 바꿀 때만(2026-11-07).
        assert_eq!(shown, vec![Line::MaxHp, Line::Flask]);
        // 맞바꿈 장신구(피해 +1 · 공속 −15%): 둘 다 보이고 하나는 좋아짐, 하나는 나빠짐.
        let hand = Stats::compute(&[crate::charms::def(crate::charms::Id::Hand).mods]);
        let preview = lines(&base, Some(&hand));
        let dmg = preview.iter().find(|l| l.0 == Line::Dmg).unwrap();
        assert_eq!(dmg.2.unwrap().1, Change::Better);
        let spd = preview.iter().find(|l| l.0 == Line::AtkSpeed).unwrap();
        assert_eq!(spd.2.unwrap().1, Change::Worse);
        // 작을수록 좋은 줄: 받는 피해가 늘면 나빠짐, 회피 기력이 줄면 좋아짐.
        let feet = Stats::compute(&[crate::charms::def(crate::charms::Id::Feet).mods]);
        assert_eq!(Line::Taken.change(&base, &feet), Change::Worse);
        assert_eq!(Line::DodgeCost.change(&base, &feet), Change::Better);
        assert_eq!(Line::MaxHp.change(&base, &feet), Change::Same);
    }
}
