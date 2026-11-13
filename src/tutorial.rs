//! 스테이지 0의 튜토리얼(0167): 한 번에 하나, 화면 아래 힌트 두 줄("[n/N] 키  할 일" + 작은 풀이) — nail 0023의 꼴에서
//! 안내자와 대화창을 뺀 것. 싸움을 멈추지 않는다(상황이 가르친다 — KB `action-roguelite-for-solo-dev`).
//!
//! 규칙은 순수하다: 본 것(`Seen` — 걸은 거리·맞힌 타·회피·패링·쓴 스킬)과 지금의 사실(`Facts` — 잔불·레벨·기술 판)에서
//! 지금 걸음이 끝났는지를 본다. 걸음은 차례대로지만 본 것은 처음부터 쌓이므로 앞질러 한 일은 그 걸음에 닿는 순간 지나간다.
//! 뒤쪽 넷(잔불 → 쉬고 레벨 → 기술 배우기 → 쓰기)이 이 튜토리얼의 까닭이다: 새 판은 스킬도 기술 점수도 없이 서고(0166·0167),
//! 첫 점수는 첫 레벨에서 나온다.
//!
//! 지난 걸음 수는 기록에 남는다(`save::key::TUTORIAL`) — 이어하면 그 걸음부터. 개발 창에서 시작한 판·놀이터에서는 돌지 않는다.

use crate::i18n::{fill, Strings};
use crate::input::Bind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Move,
    Attack,
    Dodge,
    /// 실패가 잦다 — 싸움 속에서 `PARRY_GIVE_UP`초를 못 하면 넘어간다(막히면 뒤의 걸음이 다 멈춘다, nail 0023).
    Parry,
    /// 첫 레벨 값만큼 잔불을 모은다.
    Gather,
    /// 쉼터 화로에서 쉬고 한 단을 올린다 — 기술 점수 +1.
    Level,
    /// 그 점수로 기술 판에서 하나를 익힌다.
    Learn,
    /// 배운 스킬을 한 번 쓴다.
    Skill,
}

pub const STEPS: [Step; 8] = [
    Step::Move,
    Step::Attack,
    Step::Dodge,
    Step::Parry,
    Step::Gather,
    Step::Level,
    Step::Learn,
    Step::Skill,
];

/// 다 지났다(기록의 값).
pub const DONE: u8 = STEPS.len() as u8;
/// 몸의 걸음(걷기·공격·회피·패링)을 지난 자리 — 옛 기록은 여기부터(`start_for_old`).
pub const AFTER_BASICS: u8 = 4;

/// 걷기: 이만큼(m).
pub const MOVE_M: f32 = 4.0;
/// 공격: 맞힌 타 수.
pub const HITS: u32 = 3;
/// 패링: 깨어 있는 적 곁에서 이만큼(초) 못 하면 넘어간다.
pub const PARRY_GIVE_UP: f32 = 40.0;
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
}

impl Step {
    /// 이 걸음이 끝났나.
    pub fn met(self, s: &Seen, f: &Facts) -> bool {
        match self {
            Step::Move => s.moved >= MOVE_M,
            Step::Attack => s.hits >= HITS,
            Step::Dodge => s.dodges >= 1,
            Step::Parry => s.parries >= 1 || s.parry_wait >= PARRY_GIVE_UP,
            Step::Gather => f.level >= 1 || f.cost.is_none_or(|c| f.embers >= c),
            Step::Level => f.level >= 1,
            Step::Learn => f.spent,
            // 패시브만 열었으면 쓸 스킬이 없다 — 붙잡지 않는다.
            Step::Skill => s.skills >= 1 || (f.spent && !f.knows_skill),
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
            Step::Move | Step::Gather | Step::Level | Step::Learn => None,
        }
    }

    /// 힌트의 글 둘(버튼 이름 없이 — 동작의 이름으로): 할 일 한마디 + 작은 풀이(없으면 빈 글). 세는 걸음은 "지금/목표"를 채운다.
    pub fn text(self, st: &Strings, s: &Seen, f: &Facts) -> (String, &'static str) {
        match self {
            Step::Move => (st.tut_move.to_string(), st.tut_move_d),
            Step::Attack => (
                fill(
                    st.tut_attack,
                    &[&s.hits.min(HITS).to_string(), &HITS.to_string()],
                ),
                st.tut_attack_d,
            ),
            Step::Dodge => (st.tut_dodge.to_string(), st.tut_dodge_d),
            Step::Parry => (st.tut_parry.to_string(), st.tut_parry_d),
            Step::Gather => {
                let c = f.cost.unwrap_or(0);
                (
                    fill(
                        st.tut_gather,
                        &[&f.embers.min(c).to_string(), &c.to_string()],
                    ),
                    st.tut_gather_d,
                )
            }
            // 쉬는 것까지는 같은 말, 화로에서 할 일이 다르다.
            Step::Level => (st.tut_rest.to_string(), st.tut_level_d),
            Step::Learn => (st.tut_rest.to_string(), st.tut_learn_d),
            Step::Skill => (st.tut_skill.to_string(), st.tut_skill_d),
        }
    }
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

    /// 힌트: 머리 줄 "[n/N] 키 할 일"(버튼 이름이 없는 걸음·장치는 글만) + 작은 풀이 줄. 다 지났으면 없음.
    /// `key`가 버튼의 표시를 만든다 — 앱은 키캡(`ui::key`)으로 감싸 넘긴다(글 속에서 버튼처럼 보이게).
    pub fn hint(
        &self,
        st: &Strings,
        f: &Facts,
        slot: usize,
        key: impl Fn(Bind) -> String,
    ) -> Option<(String, &'static str)> {
        let step = self.step()?;
        let (text, more) = step.text(st, &self.seen, f);
        let k = step.bind(slot).map(key).unwrap_or_default();
        let n = self.at + 1;
        let head = if k.is_empty() {
            format!("[{n}/{DONE}] {text}")
        } else {
            format!("[{n}/{DONE}] {k} {text}")
        };
        Some((head, more))
    }
}

/// 이 값이 없는 옛 기록이 설 자리: 이미 레벨을 올려 본 방랑자에게는 가르칠 것이 없다. 한 번도 안 올렸으면 몸의 걸음은
/// 건너뛰고 잔불 → 레벨부터(그 길을 못 찾은 판이다).
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

    fn facts() -> Facts {
        Facts {
            embers: 0,
            cost: Some(50),
            level: 0,
            spent: false,
            knows_skill: false,
        }
    }

    fn at(t: &Tutor) -> Option<Step> {
        t.step()
    }

    /// 걸음은 차례대로 — 걷기 → 공격 세 타 → 회피 → 패링 → 잔불 → 레벨 → 배우기 → 쓰기. 걸음마다 그 일을 해야 넘어간다.
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
        t.seen.parries = 1;
        t.tick(&f, false, 0.1);
        assert_eq!(at(&t), Some(Step::Gather));
        f.embers = 49;
        assert!(!t.tick(&f, false, 0.1));
        f.embers = 50;
        t.tick(&f, false, 0.1);
        assert_eq!(at(&t), Some(Step::Level));
        // 올리면 잔불은 줄지만 돌아가지 않는다.
        f.embers = 0;
        f.level = 1;
        f.cost = Some(75);
        t.tick(&f, false, 0.1);
        assert_eq!(at(&t), Some(Step::Learn));
        f.spent = true;
        f.knows_skill = true;
        t.tick(&f, false, 0.1);
        assert_eq!(at(&t), Some(Step::Skill));
        t.seen.skills = 1;
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

    /// 패링은 막히지 않는다: 깨어 있는 적 곁에서 `PARRY_GIVE_UP`초를 못 하면 넘어간다. 적이 없는 동안은 세지 않는다.
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
        assert_eq!(at(&t), Some(Step::Gather));
    }

    /// 뒤쪽 걸음은 사실로 본다: 이미 올린 레벨·이미 익힌 기술은 지나가고, 패시브만 연 판은 "스킬 쓰기"에 붙잡히지 않는다.
    /// 레벨을 다 올린 판(값이 없다)은 잔불을 모으라고 하지 않는다.
    #[test]
    fn the_later_steps_read_the_facts() {
        let leveled = Facts {
            level: 2,
            cost: Some(100),
            ..facts()
        };
        let mut t = Tutor::new(AFTER_BASICS);
        t.tick(&leveled, false, 0.1);
        assert_eq!(at(&t), Some(Step::Learn));
        let passive_only = Facts {
            spent: true,
            ..leveled
        };
        t.tick(&passive_only, false, 0.1);
        assert_eq!(t.at, DONE);
        let maxed = Facts {
            cost: None,
            ..facts()
        };
        assert!(Step::Gather.met(&Seen::default(), &maxed));
        // 스킬을 익혔으면 한 번 쓸 때까지.
        let knows = Facts {
            spent: true,
            knows_skill: true,
            ..leveled
        };
        let mut t = Tutor::new(AFTER_BASICS);
        t.tick(&knows, false, 0.1);
        assert_eq!(at(&t), Some(Step::Skill));
    }

    /// 옛 기록: 레벨을 올려 본 판은 다 지난 것으로, 한 번도 안 올린 판은 잔불 → 레벨부터.
    #[test]
    fn an_old_record_skips_what_it_already_knows() {
        assert_eq!(start_for_old(3), DONE);
        assert_eq!(start_for_old(0), AFTER_BASICS);
        assert_eq!(STEPS[AFTER_BASICS as usize], Step::Gather);
        assert_eq!(Tutor::new(200).at, DONE, "a hand-edited record");
    }

    /// 힌트: 머리 줄 "[n/N] 키  할 일" + 작은 풀이. 버튼이 없는 걸음·이름이 없는 장치(터치)는 글만, 세는 걸음은 "지금/목표"
    /// (목표를 넘지 않는다), 스킬 걸음의 버튼은 배운 스킬이 놓인 칸. 두 언어 모두 걸음마다 글이 있고, 쉬는 두 걸음은 머리 줄이
    /// 같고 풀이가 다르다.
    #[test]
    fn the_hint_shows_the_count_the_button_and_the_deed() {
        let key = |b: Bind| format!("<{}>", b as usize);
        for st in [&crate::i18n::KO, &crate::i18n::EN] {
            let mut t = Tutor::new(0);
            let f = facts();
            let (h, _) = t.hint(st, &f, 0, key).unwrap();
            assert!(h.starts_with("[1/8] ") && !h.contains('<'), "{h}");
            t.at = 1;
            t.seen.hits = 9;
            let (h, more) = t.hint(st, &f, 0, key).unwrap();
            assert!(
                h.starts_with(&format!("[2/8] <{}> ", Bind::Attack as usize)),
                "{h}"
            );
            assert!(h.contains("3/3") && !h.contains('{'), "{h}");
            assert!(!more.is_empty());
            // 터치: 버튼 이름이 비면 글만.
            let (bare, _) = t.hint(st, &f, 0, |_| String::new()).unwrap();
            assert!(bare.starts_with("[2/8] ") && !bare.contains("  "), "{bare}");
            t.at = 4;
            let rich = Facts { embers: 70, ..f };
            assert!(t.hint(st, &rich, 0, key).unwrap().0.contains("50/50"));
            let poor = Facts { embers: 20, ..f };
            assert!(t.hint(st, &poor, 0, key).unwrap().0.contains("20/50"));
            t.at = 5;
            let level = t.hint(st, &f, 0, key).unwrap();
            t.at = 6;
            let learn = t.hint(st, &f, 0, key).unwrap();
            assert_eq!(level.0[5..], learn.0[5..], "both say: rest at a hearth");
            assert_ne!(level.1, learn.1);
            t.at = 7;
            for (slot, b) in [(0, Bind::Skill1), (1, Bind::Skill2)] {
                let (h, _) = t.hint(st, &f, slot, key).unwrap();
                assert!(h.contains(&format!("<{}>", b as usize)), "{h}");
            }
            for i in 0..DONE {
                t.at = i;
                let (h, more) = t.hint(st, &f, 0, key).unwrap();
                assert!(!h.contains('{') && h.chars().count() > 8, "{h}");
                assert!(!more.contains('{'), "{more}");
            }
            t.at = DONE;
            assert_eq!(t.hint(st, &f, 0, key), None);
        }
    }
}
