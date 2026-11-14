//! 랩 창(0070, 개발판): 0066 창 틀 안의 목록 둘 — 묶음 목록(+ A/B·내보내기·모두 되돌리기)과 묶음 하나(이름 ◀ 값 ▶).
//! 창이 열린 동안 게임은 멈추고, 바꾼 값은 바로 `App::t`에 들어가 닫은 뒤에도 계속 쓰인다. 글은 언어를 따른다(`i18n::dev`, 0080).
use super::menu::{Menu, Out};
use super::*;
use crate::i18n::dev::{self as dv, d, spec_label, t, GROUP_KO};
use crate::lab::{self, GROUPS};
use crate::tuning::SPECS;

/// 묶음 이름(한국어면 표, 영어면 `GROUPS`).
fn group_name(g: usize) -> &'static str {
    d(GROUP_KO[g], GROUPS[g].0)
}

/// 묶음 목록 위의 줄 넷.
const TOP: usize = 4;

/// 랩 창의 줄 배율(0190): 묶음 하나에 값이 스물 넘게 있어 보통 크기면 한 화면에 여섯 줄뿐이었다(사장 "하위메뉴가 다 나오지 않음").
const LAB_K: f32 = 0.7;

impl App {
    /// 싸움·아레나에서 F1·터치 LAB: 랩 창을 연다(이미 랩 안이면 그 창들을 닫는다).
    pub(super) fn toggle_lab(&mut self) {
        match self.scene() {
            Scene::Lab | Scene::LabGroup { .. } => {
                while matches!(self.scene(), Scene::Lab | Scene::LabGroup { .. }) {
                    self.go(Go::Back);
                }
            }
            s if s.ticks_world() || flow::tab_of(s).is_some() => self.go(Go::Open(Scene::Lab)),
            _ => {}
        }
    }

    pub(super) fn lab_menu(&self) -> Menu {
        let ab = if !self.lab.has_b() {
            t(dv::NO_B)
        } else if self.lab.using_b {
            "B"
        } else {
            "A"
        };
        let mut rows = vec![
            ui::Row::step(t(dv::USING), ab),
            ui::Row::plain(t(dv::STORE_B)),
            ui::Row::plain(t(dv::EXPORT)),
            ui::Row::plain(t(dv::RESET_ALL)),
        ];
        for g in 0..GROUPS.len() {
            let name = group_name(g);
            let n = lab::members(g)
                .into_iter()
                .filter(|&i| lab::changed(&self.t, i))
                .count();
            let label = if n > 0 {
                format!("{name}  *{n}")
            } else {
                name.to_string()
            };
            rows.push(ui::Row::go(label));
        }
        Menu {
            note_room: true,
            note: (!self.lab.note.is_empty()).then(|| self.lab.note.clone()),
            note_lines: 1,
            k: LAB_K,
            ..Menu::new(s().menu_lab, 360.0, rows)
        }
    }

    pub(super) fn lab_act(&mut self, out: Out) {
        match out {
            Out::Step(0, _) | Out::Pick(0) => self.lab.swap(&mut self.t),
            Out::Step(..) => {}
            Out::Pick(1) => self.lab.store(&self.t),
            Out::Pick(2) => {
                let text = self.lab.export(&self.t);
                info!("--- tuning export ---\n{}", text);
            }
            Out::Pick(3) => {
                for i in 0..SPECS.len() {
                    self.t.reset(i);
                }
                self.lab.note = t(dv::NOTE_RESET).into();
            }
            Out::Pick(i) => self.go(Go::Open(Scene::LabGroup { g: (i - TOP) as u8 })),
        }
    }

    pub(super) fn lab_group_menu(&self, g: usize) -> Menu {
        let rows = lab::members(g)
            .into_iter()
            .map(|i| {
                let s = &SPECS[i];
                let mark = if lab::changed(&self.t, i) { " *" } else { "" };
                ui::Row::step(
                    spec_label(s.name),
                    format!("{}{mark}", lab::value_text(self.t.get(i), s.step)),
                )
            })
            .collect();
        let note = lab::members(g).get(self.cursor).map(|&i| {
            let s = &SPECS[i];
            // 한국어여도 원래 이름(`tuning.rs`·내보낸 줄의 말)을 함께 — 값을 말로 주고받을 때 같은 이름으로(영어는 줄 이름이 곧 그것).
            fill(
                t(dv::LAB_HINT),
                &[
                    s.name,
                    &lab::value_text(s.def, s.step),
                    &input::prompt(Act::Swap),
                ],
            )
        });
        Menu {
            note_room: true,
            note,
            note_lines: 1,
            k: LAB_K,
            ..Menu::new(group_name(g), 400.0, rows)
        }
    }

    pub(super) fn lab_group_act(&mut self, g: usize, out: Out) {
        let ids = lab::members(g);
        match out {
            Out::Step(r, d) => self.lab_nudge(ids[r], d as f32),
            Out::Pick(r) => self.t.reset(ids[r]),
        }
    }

    /// 한 걸음(`k`배) 바꾼다 — 바로 `Tuning`에(창을 닫으면 그 값으로 싸운다).
    fn lab_nudge(&mut self, i: usize, k: f32) {
        self.t.set(i, self.t.get(i) + SPECS[i].step * k);
    }

    /// 묶음 창 한 틱: 스킬 버튼 둘(LT·RT / Q·E — 0151) = 10걸음, 나머지는 목록 몰이꾼.
    pub(super) fn tick_lab_group(&mut self, a: &Actions, g: usize) {
        let fast = a.skill2 as i32 - a.skill1 as i32;
        if fast != 0 {
            if let Some(&i) = lab::members(g).get(self.cursor) {
                self.lab_nudge(i, fast as f32 * 10.0);
            }
        }
        self.tick_menu(a, Scene::LabGroup { g: g as u8 });
    }
}
