//! 기술 창(0151 S0, 멈춤 → 기술): 스킬 칸 둘에 가진 스킬을 끼운다 — 줄 표 + 고름(0066). 좌우·확인으로 다음 스킬.
//! S0에서는 표의 스킬을 모두 가진 것으로 친다(얻는 법 — 엘리트·보스의 첫 쓰러뜨림 — 은 S3). 큰 기술은 하나라 보여 주기만 한다.
use super::menu::{Menu, Out};
use super::*;
use crate::skills;

/// 스킬 이름·설명(글 표).
pub(super) fn skill_text(id: skills::Id) -> (&'static str, &'static str) {
    let s = s();
    match id {
        skills::Id::Whirl => (s.skill_whirl, s.skill_whirl_d),
        skills::Id::Cut => (s.skill_cut, s.skill_cut_d),
        skills::Id::Gather => (s.skill_gather, s.skill_gather_d),
        skills::Id::Quake => (s.skill_quake, s.skill_quake_d),
    }
}

/// 칸 `slot`의 다음(`step` ±1) 스킬: 다른 칸에 끼운 것은 건너뛴다(같은 스킬 둘은 없다), 끝에서 감는다.
pub(super) fn next_skill(worn: [skills::Id; 2], slot: usize, step: i32) -> skills::Id {
    let opts: Vec<skills::Id> = skills::IDS
        .into_iter()
        .filter(|id| *id != worn[1 - slot])
        .collect();
    let at = opts.iter().position(|id| *id == worn[slot]).unwrap_or(0) as i32;
    opts[(at + step).rem_euclid(opts.len() as i32) as usize]
}

impl App {
    pub(super) fn skills_menu(&self) -> Menu {
        let s = s();
        let worn = self.progress.skills.0;
        let rows = vec![
            ui::Row::step(s.skills_slot1, skill_text(worn[0]).0),
            ui::Row::step(s.skills_slot2, skill_text(worn[1]).0),
            ui::Row::value(s.skills_ult, s.ult_upend),
            ui::Row::plain(s.controls_back),
        ];
        let note = match self.cursor {
            0 | 1 => skill_text(worn[self.cursor]).1,
            2 => s.ult_upend_d,
            _ => s.skills_hint,
        };
        Menu {
            note_room: true,
            note: Some(note.to_string()),
            ..Menu::new(s.skills_title, 340.0, rows)
        }
    }

    pub(super) fn skills_act(&mut self, out: Out) {
        let (i, step) = match out {
            Out::Step(i, step) => (i, step),
            Out::Pick(3) => return self.go(Go::Back),
            Out::Pick(i) => (i, 1),
        };
        if i > 1 {
            return;
        }
        let mut worn = self.progress.skills.0;
        worn[i] = next_skill(worn, i, step);
        self.progress.skills = skills::Worn(worn);
        self.world.set_skills(worn);
        self.persist();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skills::Id;

    /// 0151: 칸을 돌리면 다른 칸의 스킬은 건너뛰고(같은 스킬 둘은 못 끼운다), 끝에서 감는다.
    #[test]
    fn stepping_a_slot_skips_the_skill_in_the_other_slot_and_wraps() {
        let worn = [Id::Whirl, Id::Cut];
        assert_eq!(next_skill(worn, 0, 1), Id::Gather, "skips Cut");
        assert_eq!(next_skill(worn, 0, -1), Id::Quake, "wraps");
        assert_eq!(next_skill(worn, 1, 1), Id::Gather);
        assert_eq!(next_skill(worn, 1, -1), Id::Quake, "skips Whirl, wraps");
        // 어느 쪽으로 돌려도 표의 나머지를 다 지나 제자리로 온다.
        let mut w = worn;
        for _ in 0..skills::N - 1 {
            w[0] = next_skill(w, 0, 1);
            assert_ne!(w[0], w[1]);
        }
        assert_eq!(w, worn);
    }
}
