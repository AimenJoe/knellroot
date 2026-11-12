//! 기술 창(0151 S0, 멈춤 → 기술): 스킬 칸 둘에 **배운** 스킬을 끼운다 — 줄 표 + 고름(0066). 좌우·확인으로 다음 스킬.
//! 배우고 올리는 것은 쉼터 화로의 기술 판(`board.rs`, 0158) — 여기서는 끼우기와 단·고르기 읽기만("휘돌이 3 · ㄱ").
use super::board::pick_text;
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
        skills::Id::Brace => (s.skill_brace, s.skill_brace_d),
        skills::Id::Echo => (s.skill_echo, s.skill_echo_d),
        skills::Id::Throw => (s.skill_throw, s.skill_throw_d),
        skills::Id::Embers => (s.skill_embers, s.skill_embers_d),
    }
}

/// 큰 기술 이름·설명(글 표).
pub(super) fn ult_text(u: skills::Ult) -> (&'static str, &'static str) {
    let s = s();
    match u {
        skills::Ult::Upend => (s.ult_upend, s.ult_upend_d),
        skills::Ult::Flurry => (s.ult_flurry, s.ult_flurry_d),
        skills::Ult::Line => (s.ult_line, s.ult_line_d),
    }
}

/// 끼울 다음(`step` ±1) 큰 기술(0158 G2 — 하나만 끼운다): 배운 것만, 끝에서 감는다. 쟁반 뒤집기뿐이면 그대로.
pub(super) fn next_ult(board: &skills::Board, worn: skills::Ult, step: i32) -> skills::Ult {
    let opts: Vec<skills::Ult> = skills::ULTS
        .into_iter()
        .filter(|u| board.has_ult(*u))
        .collect();
    if opts.is_empty() {
        return worn;
    }
    let at = opts.iter().position(|u| *u == worn).unwrap_or(0) as i32;
    opts[(at + step).rem_euclid(opts.len() as i32) as usize]
}

/// 칸 `slot`의 다음(`step` ±1) 스킬: 배운 것만, 다른 칸에 끼운 것은 건너뛴다(같은 스킬 둘은 없다), 끝에서 감는다.
/// 배운 것이 둘뿐이면(처음의 둘) 바꿀 것이 없어 그대로다.
pub(super) fn next_skill(
    board: &skills::Board,
    worn: [skills::Id; 2],
    slot: usize,
    step: i32,
) -> skills::Id {
    let opts: Vec<skills::Id> = skills::IDS
        .into_iter()
        .filter(|id| board.learned(*id) && *id != worn[1 - slot])
        .collect();
    if opts.is_empty() {
        return worn[slot];
    }
    let at = opts.iter().position(|id| *id == worn[slot]).unwrap_or(0) as i32;
    opts[(at + step).rem_euclid(opts.len() as i32) as usize]
}

/// 끼운 스킬의 표시: "휘돌이 2", 3단부터는 고르기 표지까지 "휘돌이 3 · ㄱ".
pub(super) fn worn_label(board: &skills::Board, id: skills::Id) -> String {
    let s = s();
    let i = id.index();
    let rank = board.rank[i].to_string();
    let name = skill_text(id).0;
    if board.rank[i] >= skills::PICK_RANK {
        fill(
            s.skills_worn_pick,
            &[name, &rank, pick_text(id, board.pick[i]).0],
        )
    } else {
        fill(s.skills_worn, &[name, &rank])
    }
}

impl App {
    pub(super) fn skills_menu(&self) -> Menu {
        let s = s();
        let board = self.board();
        let worn = self.progress.skills.0;
        // 끼운 큰 기술(0158 G2): 배운 것들 사이를 좌우로 — 하나만 끼운다.
        let great = board.wearable_ult(self.progress.ult);
        let rows = vec![
            ui::Row::step(s.skills_slot1, worn_label(&board, worn[0])),
            ui::Row::step(s.skills_slot2, worn_label(&board, worn[1])),
            ui::Row::step(
                s.skills_ult,
                fill(
                    s.skills_worn,
                    &[ult_text(great).0, &board.ult[great.index()].to_string()],
                ),
            ),
            ui::Row::plain(s.controls_back),
        ];
        // 칸의 설명: 하는 일 + (3단부터) 지금 고르기 한 줄.
        let slot_note = |id: skills::Id| {
            let i = id.index();
            if board.rank[i] >= skills::PICK_RANK {
                let (mark, name, desc) = pick_text(id, board.pick[i]);
                format!(
                    "{} {}",
                    skill_text(id).1,
                    fill(s.board_pick_now, &[mark, name, desc])
                )
            } else {
                skill_text(id).1.to_string()
            }
        };
        let note = match self.cursor {
            0 | 1 => slot_note(worn[self.cursor]),
            2 => ult_text(great).1.to_string(),
            _ => s.skills_hint.to_string(),
        };
        Menu {
            sub: Some(s.skills_view_only.to_string()),
            note_room: true,
            note: Some(note),
            note_lines: 3,
            ..Menu::new(s.skills_title, 360.0, rows)
        }
    }

    pub(super) fn skills_act(&mut self, out: Out) {
        let (i, step) = match out {
            Out::Step(i, step) => (i, step),
            Out::Pick(3) => return self.go(Go::Back),
            Out::Pick(i) => (i, 1),
        };
        if i == 2 {
            let board = self.board();
            let now = board.wearable_ult(self.progress.ult);
            let next = next_ult(&board, now, step);
            if next == now {
                return;
            }
            self.progress.ult = next;
            self.world.set_skills(self.loadout());
            #[cfg(feature = "dev")]
            if self.dev_board.is_some() {
                return;
            }
            self.persist();
            return;
        }
        if i > 1 {
            return;
        }
        let mut worn = self.progress.skills.0;
        let next = next_skill(&self.board(), worn, i, step);
        if next == worn[i] {
            return;
        }
        worn[i] = next;
        self.progress.skills = skills::Worn(worn);
        self.world.set_skills(self.loadout());
        // 개발 창에서 올린 판으로 끼운 것(기록의 판에서는 안 배운 스킬일 수 있다)은 슬롯에 쓰지 않는다.
        #[cfg(feature = "dev")]
        if self.dev_board.is_some() {
            return;
        }
        self.persist();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skills::{Board, Id};

    /// 0151: 칸을 돌리면 다른 칸의 스킬은 건너뛰고(같은 스킬 둘은 못 끼운다), 끝에서 감는다 — 다 배운 판에서.
    #[test]
    fn stepping_a_slot_skips_the_skill_in_the_other_slot_and_wraps() {
        let all = Board::maxed(skills::Pick::A);
        let worn = [Id::Whirl, Id::Cut];
        assert_eq!(next_skill(&all, worn, 0, 1), Id::Gather, "skips Cut");
        assert_eq!(next_skill(&all, worn, 0, -1), Id::Embers, "wraps");
        assert_eq!(next_skill(&all, worn, 1, 1), Id::Gather);
        assert_eq!(
            next_skill(&all, worn, 1, -1),
            Id::Embers,
            "skips Whirl, wraps"
        );
        // 어느 쪽으로 돌려도 표의 나머지를 다 지나 제자리로 온다.
        let mut w = worn;
        for _ in 0..skills::N - 1 {
            w[0] = next_skill(&all, w, 0, 1);
            assert_ne!(w[0], w[1]);
        }
        assert_eq!(w, worn);
    }

    /// 0158: 끼울 수 있는 것은 배운 스킬뿐 — 처음의 둘만 배웠으면 돌려도 그대로, 하나를 더 배우면 그것만 돈다.
    #[test]
    fn only_learned_skills_can_be_worn() {
        let mut b = Board::default();
        let worn = skills::START;
        for slot in 0..2 {
            for step in [-1, 1] {
                assert_eq!(next_skill(&b, worn, slot, step), worn[slot]);
            }
        }
        b.rank[Id::Gather.index()] = 1;
        assert_eq!(next_skill(&b, worn, 0, 1), Id::Gather);
        assert_eq!(next_skill(&b, [Id::Gather, Id::Cut], 0, 1), Id::Whirl);
        assert_eq!(
            next_skill(&b, worn, 1, -1),
            Id::Gather,
            "Quake is not learned"
        );
    }

    /// 멈춤 창은 단을 읽기만: "이름 단", 3단부터는 고르기 표지까지.
    #[test]
    fn a_worn_skill_shows_its_rank_and_from_rank_three_its_pick() {
        let s = s();
        let mut b = Board::default();
        assert_eq!(
            worn_label(&b, Id::Whirl),
            fill(s.skills_worn, &[s.skill_whirl, "1"])
        );
        b.rank[Id::Whirl.index()] = 3;
        assert_eq!(
            worn_label(&b, Id::Whirl),
            fill(s.skills_worn_pick, &[s.skill_whirl, "3", s.pick_mark_a])
        );
        b.flip(Id::Whirl);
        assert!(worn_label(&b, Id::Whirl).ends_with(s.pick_mark_b));
    }

    /// 0158 G2: 끼우는 큰 기술은 하나 — 배운 것들 사이만 돈다(쟁반 뒤집기뿐이면 그대로), 끝에서 감는다. 이름이 셋 다 다르다.
    #[test]
    fn the_worn_great_skill_steps_through_the_learned_ones_only() {
        use skills::Ult;
        let mut b = Board::default();
        for step in [-1, 1] {
            assert_eq!(next_ult(&b, Ult::Upend, step), Ult::Upend);
        }
        b.ult[Ult::Line.index()] = 1;
        assert_eq!(next_ult(&b, Ult::Upend, 1), Ult::Line, "skips Flurry");
        assert_eq!(next_ult(&b, Ult::Line, 1), Ult::Upend, "wraps");
        assert_eq!(next_ult(&b, Ult::Upend, -1), Ult::Line);
        b.ult[Ult::Flurry.index()] = 2;
        assert_eq!(next_ult(&b, Ult::Upend, 1), Ult::Flurry);
        assert_eq!(next_ult(&b, Ult::Flurry, 1), Ult::Line);
        let names: Vec<_> = skills::ULTS.iter().map(|u| ult_text(*u).0).collect();
        assert!(names[0] != names[1] && names[1] != names[2] && names[0] != names[2]);
        for id in skills::IDS {
            let (name, desc) = skill_text(id);
            assert!(!name.is_empty() && !desc.is_empty(), "{id:?}");
        }
    }
}
