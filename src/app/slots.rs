//! 슬롯 창(0066, KB `roguelite-save-slots-and-continue`, nail 0078): 타이틀 시작하기·이어하기가 둘 다 여기로.
//! 줄 = 슬롯 이름 + 요약(완주·잔불·장신구), 오른쪽은 할 일. 새로 시작: 빈 슬롯 = 시작, 기록 있으면 덮어쓰기(두 번).
//! 이어하기: 기록 있는 슬롯만(빈 줄은 건너뜀), 좌우로 이어하기 ↔ 지우기, 지우기는 두 번, 마지막 기록을 지우면 타이틀로.
//! 결정은 `flow::slot_focus`·`slot_step`·`slot_pick`(순수, 시험), 여기는 그대로 한다.
use super::menu::{Menu, Out};
use super::*;
use flow::SlotDo;

impl App {
    fn filled(&self) -> [bool; crate::save::SLOTS] {
        std::array::from_fn(|i| self.slots[i].is_some())
    }

    /// 슬롯 창을 연다: 요약을 새로 읽고, 처음 포커스는 KB대로(이어하기 = 마지막 슬롯, 새로 시작 = 첫 빈 슬롯).
    pub(super) fn open_slots(&mut self, load: bool) {
        self.slots = crate::save::summaries();
        self.go(Go::Open(Scene::Slots { load }));
        self.cursor = flow::slot_focus(load, &self.filled(), self.slot);
    }

    pub(super) fn slots_menu(&self, load: bool) -> Menu {
        let s = s();
        let rows = self
            .slots
            .iter()
            .enumerate()
            .map(|(i, sum)| {
                let name = fill(s.slot_name, &[&(i + 1).to_string()]);
                let label = match sum {
                    // 장 · 지역(스테이지) · 시간(사장 2026-11-07, 0081 — 완주·잔불·장신구 수는 뺌). 장마다 스테이지는 하나(지금 1장 = 성소).
                    Some(m) => format!(
                        "{name}   {}",
                        fill(
                            s.slot_summary,
                            &[
                                &m.chapter.to_string(),
                                tr(crate::stage::sanctuary::DEF.name),
                                &crate::save::play_text(m.play),
                            ]
                        )
                    ),
                    None => name,
                };
                let here = i == self.cursor;
                match (load, sum.is_some()) {
                    (true, false) => ui::Row::value(label, s.slot_empty).dimmed(true),
                    (true, true) => {
                        let act = match (here && self.col == 1, here && self.armed) {
                            (true, true) => s.slot_erase_sure,
                            (true, false) => s.slot_erase,
                            _ => s.slot_continue,
                        };
                        ui::Row::step(label, act)
                    }
                    (false, false) => ui::Row::value(label, s.slot_start),
                    (false, true) => ui::Row::value(
                        label,
                        if here && self.armed {
                            s.slot_overwrite_sure
                        } else {
                            s.slot_overwrite
                        },
                    ),
                }
            })
            .collect();
        Menu::new(if load { s.slots_load } else { s.slots_new }, 400.0, rows)
    }

    /// 슬롯 창 한 틱: 몰이꾼 입력 + 줄을 옮기면 두 번째 누름을 푼다 + 이어하기 창은 빈 줄을 건너뛴다.
    pub(super) fn tick_slots(&mut self, a: &Actions, load: bool) {
        let before = self.cursor;
        let m = self.slots_menu(load);
        let out = self.menu_input(a, &m);
        if self.scene() != (Scene::Slots { load }) {
            return;
        }
        if self.cursor != before {
            self.armed = false;
            self.col = 0;
            let filled = self.filled();
            if load && !filled[self.cursor] {
                let dir = if a.menu_up { -1 } else { 1 };
                self.cursor = flow::slot_step(before, dir, &filled);
            }
        }
        match out {
            Some(Out::Step(i, _)) if load && self.filled()[i] => {
                self.col = 1 - self.col.min(1);
                self.armed = false;
            }
            Some(Out::Pick(i)) => self.slot_do(load, i),
            _ => {}
        }
    }

    fn slot_do(&mut self, load: bool, i: usize) {
        let filled = self.filled();
        match flow::slot_pick(load, filled[i], self.col, self.armed) {
            SlotDo::Nothing => {}
            SlotDo::Arm => self.armed = true,
            SlotDo::Fresh => {
                crate::save::wipe(i);
                self.enter_slot(i, crate::save::Progress::default());
            }
            SlotDo::Load => self.enter_slot(i, crate::save::load(i)),
            SlotDo::Erase => {
                crate::save::wipe(i);
                self.slots = crate::save::summaries();
                (self.armed, self.col) = (false, 0);
                let filled = self.filled();
                if filled.iter().any(|&f| f) {
                    self.cursor = flow::slot_step(i, 1, &filled);
                } else {
                    // 마지막 기록을 지우면 타이틀로(KB).
                    self.go(Go::Back);
                }
            }
        }
    }

    /// 이 슬롯으로 판을 시작한다: 마지막 슬롯 칸을 적고, 기록·무기를 그 슬롯 것으로, 쉼터가 있으면 거기서.
    fn enter_slot(&mut self, i: usize, progress: crate::save::Progress) {
        #[cfg(feature = "dev")]
        {
            self.dev_run = false;
            self.dev_board = None;
        }
        self.slot = i;
        crate::save::set_last_slot(i);
        self.weapon = progress
            .weapon
            .filter(|&w| progress.owns_weapon(w))
            .unwrap_or(weapons::Id::Sword);
        self.saved = progress.clone();
        self.progress = progress;
        self.slots = crate::save::summaries();
        crate::screen::touch_enter();
        self.continue_run();
        self.go(Go::Root(Scene::Play));
    }
}
