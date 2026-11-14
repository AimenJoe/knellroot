//! 스테이지 0의 튜토리얼(0167 → 0169): 한 번에 하나, 화면 아래 힌트 두 줄("[n/N] 키 할 일" + 작은 풀이) — nail 0023의 꼴에서
//! 안내자와 대화창을 뺀 것. 싸움을 멈추지 않는다(상황이 가르친다 — KB `action-roguelite-for-solo-dev`).
//!
//! 규칙은 순수하다: 본 것(`Seen` — 걸은 거리·맞힌 타·회피·패링·쓴 스킬)과 지금의 사실(`Facts` — 잔불·레벨·기술 판)에서
//! 지금 걸음이 끝났는지를 본다. 걸음은 차례대로지만 본 것은 처음부터 쌓이므로 앞질러 한 일은 그 걸음에 닿는 순간 지나간다.
//!
//! **상(0169)**: 몸의 걸음 넷(걷기·공격·회피·패링)을 마치면 기술 점수 하나를 받는다(`REWARD_POINTS` — 앱이 기록의 몫에 적는다).
//! 새 판은 스킬도 점수도 없이 서는데(0166·0167) 잔불로 사는 첫 레벨은 한 판이 걸린다(0109·0123 "한 판 ≈ 한 레벨") — 첫 스킬을
//! 그 뒤로 미루지 않으려고 튜토리얼이 준다. 레벨의 값은 건드리지 않는다. 그 뒤의 걸음은 쉬고 배우기 → 쓰기, 그리고 잔불이
//! 첫 레벨 값만큼 모였을 때에야 뜨는 "한 단 올리기".
//!
//! 지난 걸음 수는 기록에 남는다(`save::key::TUTORIAL`) — 이어하면 그 걸음부터. 개발 창에서 시작한 판·놀이터에서는 돌지 않는다.

use crate::i18n::Strings;
use crate::input::Bind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Move,
    Attack,
    Dodge,
    /// 실패가 잦다 — 싸움 속에서 `PARRY_GIVE_UP`초를 못 하면 넘어간다(막히면 뒤의 걸음이 다 멈춘다, nail 0023).
    Parry,
    /// 쉼터 화로에서 쉬고 기술 판에서 하나를 배운다(상으로 받은 점수로).
    Learn,
    /// 배운 스킬을 한 번 쓴다.
    Skill,
    /// 쉼터 화로에서 쉬고 한 단을 올린다 — 잔불이 그 값만큼 모였을 때에야 뜬다(그 전에는 조용하다).
    Level,
}

pub const STEPS: [Step; 7] = [
    Step::Move,
    Step::Attack,
    Step::Dodge,
    Step::Parry,
    Step::Learn,
    Step::Skill,
    Step::Level,
];

/// 다 지났다(기록의 값).
pub const DONE: u8 = STEPS.len() as u8;
/// 몸의 걸음(걷기·공격·회피·패링)을 지난 자리 — 상을 받는 자리이고, 옛 기록은 여기부터(`start_for_old`).
pub const AFTER_BASICS: u8 = 4;
/// 몸의 걸음 넷을 마친 상: 기술 점수(0169). 기록의 몫(`Progress::gift`)에 적힌다 — 판마다 한 번.
pub const REWARD_POINTS: u32 = 1;

/// 걷기: 이만큼(m).
pub const MOVE_M: f32 = 4.0;
/// 공격: 맞힌 타 수.
pub const HITS: u32 = 3;
/// 패링: 깨어 있는 적 곁에서 이만큼(초) 못 하면 넘어간다. 0169에서 40 → 20: 이 걸음 뒤에 상(첫 기술 점수)이 걸려 있다 —
/// 적 두엇과 싸우는 길이면 넉넉하다(연습 상대를 따로 세우지 않으니 nail의 30초보다 짧게).
pub const PARRY_GIVE_UP: f32 = 20.0;
/// "싸움 속"의 거리(m): 깨어 있는 적이 이 안에 있을 때만 패링의 시계가 간다.
pub const FIGHT_RANGE: f32 = 10.0;
/// 걸음이 바뀐 뒤 힌트가 밝게 뜨는 시간(초).
pub const FRESH_TIME: f32 = 1.2;

/// 본 것 — 이 판을 켠 뒤로 쌓인다(기록에 남지 않는다. 남는 것은 지난 걸음 수뿐).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Seen {
    pub moved: f32,
    pub hits: u32,
    pub dodges: u32,
    pub parries: u32,
    /// 패링 걸음에서 깨어 있는 적 곁에 있던 시간.
    pub parry_wait: f32,
    pub skills: u32,
}

/// 지금의 사실(앱이 기록·월드에서 모은다).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Facts {
    pub embers: u32,
    /// 다음 레벨 값(다 올렸으면 없음).
    pub cost: Option<u32>,
    /// 총 레벨.
    pub level: u32,
    /// 기술 판에 쓴 점수가 있다(스킬이든 패시브든).
    pub spent: bool,
    pub knows_skill: bool,
    /// 지금 기술 판에서 살 수 있는 것이 있다(남은 점수 + 열린 칸).
    pub can_learn: bool,
    /// 기록의 몫(옛 기록 2 · 튜토리얼의 상 1 · 아직이면 0).
    pub gift: u32,
}

impl Step {
    /// 이 걸음이 끝났나.
    pub fn met(self, s: &Seen, f: &Facts) -> bool {
        match self {
            Step::Move => s.moved >= MOVE_M,
            Step::Attack => s.hits >= HITS,
            Step::Dodge => s.dodges >= 1,
            Step::Parry => s.parries >= 1 || s.parry_wait >= PARRY_GIVE_UP,
            Step::Learn => f.spent,
            // 아는 스킬이 없으면(패시브만 열었다) 쓸 것이 없다 — 붙잡지 않는다.
            Step::Skill => s.skills >= 1 || !f.knows_skill,
            Step::Level => f.level >= 1 || f.cost.is_none(),
        }
    }

    /// 이 걸음의 힌트를 지금 보이나 — 할 수 없는 일을 시키지 않는다: 배울 점수가 없으면 "배운다"를, 잔불이 모자라면
    /// "올린다"를 띄우지 않는다(걸음은 그대로 기다린다).
    pub fn shown(self, f: &Facts) -> bool {
        match self {
            Step::Learn => f.can_learn,
            Step::Level => f.cost.is_some_and(|c| f.embers >= c),
            _ => true,
        }
    }

    /// 힌트에 붙는 버튼(지금 배치의 이름 — 글에 키 이름을 박지 않는다). `slot` = 배운 스킬이 놓인 칸.
    pub fn bind(self, slot: usize) -> Option<Bind> {
        match self {
            Step::Attack => Some(Bind::Attack),
            Step::Dodge => Some(Bind::Dodge),
            Step::Parry => Some(Bind::Parry),
            Step::Skill => Some(if slot == 1 {
                Bind::Skill2
            } else {
                Bind::Skill1
            }),
            Step::Move | Step::Learn | Step::Level => None,
        }
    }

    /// 힌트의 글 둘(버튼 이름 없이 — 동작의 이름으로): 할 일 한마디 + 작은 풀이. 풀이는 기능의 설명이 아니라 **몸의
    /// 기억**으로 말한다(0171 — 과거를 잃은 방랑자: 잊었지만 몸은 안다).
    pub fn text(self, st: &Strings) -> (&'static str, &'static str) {
        match self {
            Step::Move => (st.tut_move, st.tut_move_d),
            Step::Attack => (st.tut_attack, st.tut_attack_d),
            Step::Dodge => (st.tut_dodge, st.tut_dodge_d),
            Step::Parry => (st.tut_parry, st.tut_parry_d),
            // 쉬는 것까지는 같은 말, 화로에서 할 일이 다르다.
            Step::Learn => (st.tut_rest, st.tut_learn_d),
            Step::Skill => (st.tut_skill, st.tut_skill_d),
            Step::Level => (st.tut_rest, st.tut_level_d),
        }
    }

    /// 세는 걸음의 (지금, 목표) — 머리 줄 곁에 따로 그린다(0171: 글 속의 "(0/3)"이 아니라 제 빛의 수).
    pub fn count(self, s: &Seen) -> Option<(u32, u32)> {
        match self {
            Step::Attack => Some((s.hits.min(HITS), HITS)),
            _ => None,
        }
    }
}

/// 힌트 한 장(0171 — 판 위에): 걸음 점의 자리, 버튼의 표시(없으면 빈 글), 할 일, 세는 수, 풀이.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hint {
    /// 지금 걸음의 자리(0부터) — 그 앞은 지난 점, 뒤는 남은 점. 모두 `DONE`개.
    pub at: u8,
    pub key: String,
    pub text: &'static str,
    pub count: Option<(u32, u32)>,
    pub more: &'static str,
}

/// 판에 붙은 튜토리얼의 상태: 지난 걸음 수(기록과 같은 값) + 본 것.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Tutor {
    pub at: u8,
    pub seen: Seen,
    /// 걸음이 막 바뀌었다(남은 시간 — 힌트가 잠깐 밝다).
    pub fresh: f32,
    /// 지난 틱에 구르고 있었나(회피의 시작을 센다 — 사건이 없는 몸짓이라 상태 변화로).
    pub dodging: bool,
}

impl Tutor {
    pub fn new(at: u8) -> Self {
        Tutor {
            at: at.min(DONE),
            ..Tutor::default()
        }
    }

    /// 지금 걸음(다 지났으면 없음).
    pub fn step(&self) -> Option<Step> {
        STEPS.get(self.at as usize).copied()
    }

    /// 한 틱: 패링 걸음의 기다림을 쌓고(`fight` = 깨어 있는 적이 곁에 있다), 끝난 걸음을 넘긴다 — 이미 끝나 있는 다음
    /// 걸음들도 한꺼번에. 걸음이 바뀌었으면 참.
    pub fn tick(&mut self, f: &Facts, fight: bool, dt: f32) -> bool {
        self.fresh = (self.fresh - dt).max(0.0);
        if self.step() == Some(Step::Parry) && fight {
            self.seen.parry_wait += dt;
        }
        let was = self.at;
        while self.step().is_some_and(|st| st.met(&self.seen, f)) {
            self.at += 1;
        }
        if self.at != was {
            self.fresh = FRESH_TIME;
        }
        self.at != was
    }

    /// 상을 줄 때인가(0169): 몸의 걸음을 지났고 아직 걸음이 남았고 기록의 몫이 비어 있다. 지난 일이 아니라 **지금의 상태**로
    /// 본다 — 이어한 판·옛 꼴의 기록에서도 한 번은 주고, 두 번은 주지 않는다(받으면 몫이 찬다. 옛 기록은 몫이 이미 있다).
    pub fn owes_reward(&self, f: &Facts) -> bool {
        self.at >= AFTER_BASICS && self.step().is_some() && f.gift == 0
    }

    /// 힌트(0171 — 걸음 점 + 키캡 + 할 일 + 세는 수 + 풀이): 다 지났거나 지금은 보일 것이 없으면(`Step::shown`) 없음.
    /// `key`가 버튼의 표시를 만든다 — 앱은 키캡(`ui::key`)으로 감싸 넘긴다(이름이 없는 걸음·장치는 빈 글).
    pub fn hint(
        &self,
        st: &Strings,
        f: &Facts,
        slot: usize,
        key: impl Fn(Bind) -> String,
    ) -> Option<Hint> {
        let step = self.step().filter(|s| s.shown(f))?;
        let (text, more) = step.text(st);
        Some(Hint {
            at: self.at,
            key: step.bind(slot).map(key).unwrap_or_default(),
            text,
            count: step.count(&self.seen),
            more,
        })
    }
}

/// 이 값이 없는 옛 기록이 설 자리: 이미 레벨을 올려 본 방랑자에게는 가르칠 것이 없다. 한 번도 안 올렸으면 몸의 걸음은
/// 건너뛴다 — 스킬은 이미 가졌으니 한 번 써 보고, 잔불이 모이면 "한 단 올리기"가 뜬다(그 길을 못 찾은 판이다).
pub fn start_for_old(level: u32) -> u8 {
    if level >= 1 {
        DONE
    } else {
        AFTER_BASICS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 새 판의 사실: 잔불도 레벨도 점수도 없다.
    fn facts() -> Facts {
        Facts {
            embers: 0,
            cost: Some(50),
            level: 0,
            spent: false,
            knows_skill: false,
            can_learn: false,
            gift: 0,
        }
    }

    fn at(t: &Tutor) -> Option<Step> {
        t.step()
    }

    /// 걸음은 차례대로 — 걷기 → 공격 세 타 → 회피 → 패링 → (상) → 쉬고 배우기 → 쓰기 → 한 단 올리기. 걸음마다 그 일을
    /// 해야 넘어간다.
    #[test]
    fn the_steps_go_in_order_and_each_waits_for_its_deed() {
        let mut t = Tutor::new(0);
        let mut f = facts();
        assert_eq!(at(&t), Some(Step::Move));
        assert!(!t.tick(&f, false, 0.1));
        t.seen.moved = MOVE_M;
        assert!(t.tick(&f, false, 0.1));
        assert_eq!(at(&t), Some(Step::Attack));
        assert_eq!(t.fresh, FRESH_TIME, "a new step shows bright");
        t.seen.hits = HITS - 1;
        assert!(!t.tick(&f, false, 0.1));
        t.seen.hits = HITS;
        t.tick(&f, false, 0.1);
        assert_eq!(at(&t), Some(Step::Dodge));
        t.seen.dodges = 1;
        t.tick(&f, false, 0.1);
        assert_eq!(at(&t), Some(Step::Parry));
        assert!(!t.owes_reward(&f), "not before the body steps are done");
        t.seen.parries = 1;
        t.tick(&f, false, 0.1);
        assert_eq!((at(&t), t.at), (Some(Step::Learn), AFTER_BASICS));
        // 상: 한 번 — 받으면(몫이 찬다) 다시 주지 않는다.
        assert!(t.owes_reward(&f));
        f.gift = REWARD_POINTS;
        f.can_learn = true;
        assert!(!t.owes_reward(&f));
        assert!(
            !t.tick(&f, false, 0.1),
            "resting and learning is up to them"
        );
        f.spent = true;
        f.knows_skill = true;
        f.can_learn = false;
        t.tick(&f, false, 0.1);
        assert_eq!(at(&t), Some(Step::Skill));
        t.seen.skills = 1;
        t.tick(&f, false, 0.1);
        assert_eq!(at(&t), Some(Step::Level));
        f.embers = 50;
        assert!(!t.tick(&f, false, 0.1), "having the embers is not raising");
        f.embers = 0;
        f.level = 1;
        f.cost = Some(75);
        t.tick(&f, false, 0.1);
        assert_eq!((at(&t), t.at), (None, DONE));
        assert!(!t.tick(&f, false, 0.1), "done stays done");
    }

    /// 앞질러 한 일은 그 걸음에 닿는 순간 지나간다(본 것은 처음부터 쌓인다) — 걷다 말고 구르고 벤 사람에게 다시 시키지 않는다.
    #[test]
    fn deeds_done_early_are_not_asked_again() {
        let mut t = Tutor::new(0);
        t.seen = Seen {
            moved: 1.0,
            hits: 5,
            dodges: 2,
            ..Seen::default()
        };
        assert!(!t.tick(&facts(), true, 0.1), "still has to walk");
        t.seen.moved = 9.0;
        assert!(t.tick(&facts(), true, 0.1));
        assert_eq!(at(&t), Some(Step::Parry), "attack and dodge went by");
    }

    /// 패링은 막히지 않는다: 깨어 있는 적 곁에서 `PARRY_GIVE_UP`초를 못 하면 넘어간다(상도 받는다 — 해 본 것으로 친다).
    /// 적이 없는 동안은 세지 않는다.
    #[test]
    fn parry_gives_up_after_a_while_in_a_fight() {
        let mut t = Tutor::new(3);
        assert_eq!(at(&t), Some(Step::Parry));
        for _ in 0..1000 {
            t.tick(&facts(), false, 0.1);
        }
        assert_eq!(at(&t), Some(Step::Parry), "no fight, no clock");
        let ticks = (PARRY_GIVE_UP / 0.1) as usize;
        for _ in 0..ticks - 2 {
            t.tick(&facts(), true, 0.1);
        }
        assert_eq!(at(&t), Some(Step::Parry));
        for _ in 0..4 {
            t.tick(&facts(), true, 0.1);
        }
        assert_eq!(at(&t), Some(Step::Learn));
        assert!(t.owes_reward(&facts()));
    }

    /// 0169 상의 규칙: 지금의 상태로 본다 — 몸의 걸음을 지났고(이어한 판도), 걸음이 남았고, 기록의 몫이 비었을 때 한 번.
    /// 옛 기록(몫 둘)·다 지난 판에는 주지 않는다.
    #[test]
    fn the_reward_is_owed_once_after_the_body_steps() {
        assert_eq!(REWARD_POINTS, 1);
        let f = facts();
        for i in 0..AFTER_BASICS {
            assert!(!Tutor::new(i).owes_reward(&f), "step {i}");
        }
        for i in AFTER_BASICS..DONE {
            assert!(Tutor::new(i).owes_reward(&f), "step {i} — a resumed run");
            assert!(!Tutor::new(i).owes_reward(&Facts { gift: 1, ..f }));
            assert!(!Tutor::new(i).owes_reward(&Facts { gift: 2, ..f }));
        }
        assert!(!Tutor::new(DONE).owes_reward(&f));
    }

    /// 할 수 없는 일은 시키지 않는다: 배울 점수가 없으면 "배운다" 힌트가, 잔불이 첫 레벨 값에 못 미치면 "올린다" 힌트가
    /// 뜨지 않는다(걸음은 그대로 기다린다 — 0123: 첫 레벨은 한 판이 걸린다, 그동안 힌트가 화면에 머물지 않게).
    #[test]
    fn a_hint_shows_only_when_its_deed_can_be_done() {
        let st = &crate::i18n::KO;
        let key = |_: Bind| String::new();
        let f = facts();
        let learn = Tutor::new(4);
        assert_eq!(learn.hint(st, &f, 0, key), None, "no point yet");
        let ready = Facts {
            can_learn: true,
            gift: 1,
            ..f
        };
        assert!(learn.hint(st, &ready, 0, key).is_some());
        let level = Tutor::new(6);
        assert_eq!(level.step(), Some(Step::Level));
        assert_eq!(level.hint(st, &Facts { embers: 49, ..f }, 0, key), None);
        assert!(level.hint(st, &Facts { embers: 50, ..f }, 0, key).is_some());
        // 다 올린 판(값이 없다)은 지난 것.
        assert!(Step::Level.met(&Seen::default(), &Facts { cost: None, ..f }));
        // 몸의 걸음은 늘 보인다.
        for i in 0..AFTER_BASICS {
            assert!(Tutor::new(i).hint(st, &f, 0, key).is_some(), "step {i}");
        }
    }

    /// 뒤쪽 걸음은 사실로 본다: 이미 배운 기술·이미 올린 레벨은 지나가고, 패시브만 연 판은 "스킬 쓰기"에 붙잡히지 않는다.
    #[test]
    fn the_later_steps_read_the_facts() {
        let f = facts();
        let passive_only = Facts { spent: true, ..f };
        let mut t = Tutor::new(AFTER_BASICS);
        t.tick(&passive_only, false, 0.1);
        assert_eq!(at(&t), Some(Step::Level), "nothing to use — on to the last");
        let knows = Facts {
            spent: true,
            knows_skill: true,
            ..f
        };
        let mut t = Tutor::new(AFTER_BASICS);
        t.tick(&knows, false, 0.1);
        assert_eq!(at(&t), Some(Step::Skill), "use it once");
        let leveled = Facts {
            level: 2,
            cost: Some(100),
            ..knows
        };
        t.seen.skills = 1;
        t.tick(&leveled, false, 0.1);
        assert_eq!(t.at, DONE);
    }

    /// 옛 기록: 레벨을 올려 본 판은 다 지난 것으로, 한 번도 안 올린 판은 몸의 걸음을 건너뛴다(스킬은 이미 가졌다).
    #[test]
    fn an_old_record_skips_what_it_already_knows() {
        assert_eq!(start_for_old(3), DONE);
        assert_eq!(start_for_old(0), AFTER_BASICS);
        assert_eq!(STEPS[AFTER_BASICS as usize], Step::Learn);
        assert_eq!(Tutor::new(200).at, DONE, "a hand-edited record");
        // 옛 기록의 사실(처음의 둘을 가졌고 몫이 둘): 상은 없고, 스킬을 한 번 쓰면 마지막 걸음이 조용히 기다린다.
        let old = Facts {
            spent: true,
            knows_skill: true,
            gift: 2,
            ..facts()
        };
        let mut t = Tutor::new(start_for_old(0));
        assert!(!t.owes_reward(&old));
        t.tick(&old, false, 0.1);
        assert_eq!(at(&t), Some(Step::Skill));
    }

    /// 힌트 한 장(0171): 걸음 점의 자리 · 버튼(키캡) · 할 일 · 세는 수는 글 밖에 따로 · 풀이. 글에 "{0}"·"[n/N]"이 남지
    /// 않는다 — 걸음은 점이, 세는 수는 제 칸이 말한다.
    #[test]
    fn the_hint_carries_the_step_the_button_the_deed_and_the_count() {
        let key = |b: Bind| format!("<{}>", b as usize);
        // 어느 걸음이든 보이게: 배울 점수도, 올릴 잔불도 있다.
        let f = Facts {
            can_learn: true,
            embers: 50,
            ..facts()
        };
        for st in [&crate::i18n::KO, &crate::i18n::EN] {
            let mut t = Tutor::new(0);
            let h = t.hint(st, &f, 0, key).unwrap();
            assert!((h.at, h.key.as_str(), h.count) == (0, "", None), "{h:?}");
            t.at = 1;
            t.seen.hits = 9;
            let h = t.hint(st, &f, 0, key).unwrap();
            assert_eq!(h.key, format!("<{}>", Bind::Attack as usize));
            assert_eq!((h.at, h.count), (1, Some((HITS, HITS))), "capped");
            assert!(!h.more.is_empty());
            // 터치: 버튼 이름이 비면 글만.
            let bare = t.hint(st, &f, 0, |_| String::new()).unwrap();
            assert!(bare.key.is_empty() && bare.text == h.text);
            t.at = 4;
            let learn = t.hint(st, &f, 0, key).unwrap();
            t.at = 6;
            let level = t.hint(st, &f, 0, key).unwrap();
            assert_eq!(level.text, learn.text, "both say: rest at a hearth");
            assert_ne!(level.more, learn.more);
            t.at = 5;
            for (slot, b) in [(0, Bind::Skill1), (1, Bind::Skill2)] {
                let h = t.hint(st, &f, slot, key).unwrap();
                assert_eq!(h.key, format!("<{}>", b as usize));
            }
            for i in 0..DONE {
                t.at = i;
                let h = t.hint(st, &f, 0, key).unwrap();
                let clean = |s: &str| !s.contains('{') && !s.contains('[') && !s.contains('(');
                assert!(clean(h.text) && h.text.chars().count() >= 2, "{h:?}");
                assert!(clean(h.more) && !h.more.is_empty(), "{h:?}");
                assert_eq!(h.count.is_some(), STEPS[i as usize] == Step::Attack);
            }
            t.at = DONE;
            assert_eq!(t.hint(st, &f, 0, key), None);
        }
    }
}
