//! 랩 창(0070, 개발판 — 0235에서 큰 창의 문법으로): 멈춤·화로와 같은 창 하나 + 탭. 탭 = "도구"(A/B·담기·내보내기·모두 되돌리기)와
//! 묶음 열여섯(넘치면 탭 줄을 끌어 굴린다 — 0233), 왼쪽 = 값 줄(이름 ◀ 값 ▶), 오른쪽 = 고른 값의 원래 이름·지금·기본·한 걸음, 아래 = 안내.
//! 탭 넘기기는 다른 큰 창과 같다(LB·RB / Q·E · 탭 머리 누르기 — 맨 위 칸을 바꾼다, `Go::Swap`). 열 걸음은 특수(X · U)를 쥐고 ◀▶.
//! 창이 열린 동안 게임은 멈추고, 바꾼 값은 바로 `App::t`에 들어가 닫은 뒤에도 계속 쓰인다. 글은 언어를 따른다(`i18n::dev`, 0080).
//! 0190의 줄 배율(0.7)은 걷었다 — 폰에서 글이 뭉개졌다(사장 "연구소 메뉴가 아직도 이상하다", 0235).
use super::frame::Window;
use super::menu::{Hit, Menu, Out};
use super::tabs::{Ln, PaneHead, HEAD_H, LEFT_W};
use super::*;
use crate::i18n::dev::{self as dv, d, spec_label, t, GROUP_KO};
use crate::lab::{self, GROUPS};
use crate::tuning::SPECS;

/// 묶음 이름(한국어면 표, 영어면 `GROUPS`).
fn group_name(g: usize) -> &'static str {
    d(GROUP_KO[g], GROUPS[g].0)
}

/// 도구 탭의 줄 수(쓰는 값 · B에 담기 · 내보내기 · 모두 되돌리기).
const TOOLS: usize = 4;
/// 특수를 쥐고 ◀▶ = 이만큼 걸음.
const FAST: f32 = 10.0;

/// 탭 차례(순수): 0 = 도구, 1.. = 묶음.
pub(super) fn lab_tab(scene: Scene) -> usize {
    match scene {
        Scene::LabGroup { g } => g as usize + 1,
        _ => 0,
    }
}

pub(super) fn lab_scene(tab: usize) -> Scene {
    match tab {
        0 => Scene::Lab,
        n => Scene::LabGroup { g: (n - 1) as u8 },
    }
}

/// 탭 수(도구 + 묶음).
fn lab_tabs() -> usize {
    GROUPS.len() + 1
}

impl App {
    /// 싸움·아레나에서 F1·터치 LAB: 랩 창을 연다(이미 랩 안이면 닫는다 — 탭은 쌓지 않으니 한 칸).
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

    fn lab_ab(&self) -> &'static str {
        if !self.lab.has_b() {
            t(dv::NO_B)
        } else if self.lab.using_b {
            "B"
        } else {
            "A"
        }
    }

    pub(super) fn lab_menu(&self) -> Menu {
        let rows = vec![
            ui::Row::step(t(dv::USING), self.lab_ab()),
            ui::Row::plain(t(dv::STORE_B)),
            ui::Row::plain(t(dv::EXPORT)),
            ui::Row::plain(t(dv::RESET_ALL)),
        ];
        Menu::new(t(dv::TAB_TOOLS), LEFT_W, rows)
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
            Out::Pick(_) => {
                for i in 0..SPECS.len() {
                    self.t.reset(i);
                }
                self.lab.note = t(dv::NOTE_RESET).into();
            }
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
        Menu::new(group_name(g), LEFT_W, rows)
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

    /// 랩 창 한 틱: 탭 넘기기(탭 줄 끌기·휠 · 탭 머리 · LB·RB / Q·E) → 줄 목록. 특수를 쥐고 ◀▶ = 열 걸음.
    pub(super) fn tick_lab(&mut self, mut a: Actions, scene: Scene) {
        if self.tab_strip.over(a.point) {
            self.tab_strip.nudge(a.point, a.drag_x, a.wheel);
            (a.drag, a.drag_x, a.wheel) = (0.0, 0.0, 0);
        }
        let at = lab_tab(scene);
        let n = lab_tabs() as i32;
        let to = match a.tap.and_then(|p| self.hit_at(p)) {
            Some(Hit::Tab(i)) => Some(i),
            _ => match a.tab_next as i32 - a.tab_prev as i32 {
                0 => None,
                dir => Some((at as i32 + dir).rem_euclid(n) as usize),
            },
        };
        if let Some(to) = to.filter(|&i| i != at) {
            self.go(Go::Swap(lab_scene(to)));
            self.snd(crate::sfx::Id::UiMove);
            return;
        }
        let Some(m) = self.menu_of(scene) else { return };
        match (self.menu_input(&a, &m), scene) {
            (Some(Out::Step(r, d)), Scene::LabGroup { g }) if a.cast_held => {
                if let Some(&i) = lab::members(g as usize).get(r) {
                    self.lab_nudge(i, d as f32 * FAST);
                }
            }
            (Some(out), _) => self.act(scene, out),
            (None, _) => {}
        }
    }

    /// 랩 창 그림: 큰 창(탭 = 도구 + 묶음, 바꾼 값이 있는 묶음은 민트 점) · 지갑 자리에 쓰는 값(A/B) · 왼쪽 값 줄 · 오른쪽 상세 · 안내.
    pub(super) fn draw_lab(&mut self, f: &Frame, scene: Scene) {
        let names: Vec<(String, bool)> = std::iter::once((t(dv::TAB_TOOLS).to_string(), false))
            .chain((0..GROUPS.len()).map(|g| {
                let changed = lab::members(g)
                    .into_iter()
                    .any(|i| lab::changed(&self.t, i));
                (group_name(g).to_string(), changed)
            }))
            .collect();
        let tabs: Vec<(&str, bool)> = names.iter().map(|(n, m)| (n.as_str(), *m)).collect();
        let purse = [(t(dv::USING), self.lab_ab().to_string())];
        let w = self.draw_window(f, &tabs, lab_tab(scene), &purse, None);
        let x = w.at(LEFT_W + 6.0, 0.0).x.round();
        draw_rectangle(
            x,
            w.origin.y,
            1.0,
            w.body_h * w.s,
            Color {
                a: 0.6,
                ..ui::BORDER
            },
        );
        let Some(m) = self.menu_of(scene) else { return };
        self.draw_left_list(&w, &m.rows, 0.0);
        let at = self.cursor.min(m.rows.len().saturating_sub(1));
        match scene {
            Scene::LabGroup { g } => self.draw_lab_value(&w, g as usize, at),
            _ => self.draw_lab_tool(&w, &m, at),
        }
        let s = s();
        let mut hints = vec![(Act::Choose, s.verb_choose)];
        if matches!(scene, Scene::LabGroup { .. }) {
            hints.push((Act::Special, t(dv::LAB_FAST)));
            hints.push((Act::Confirm, t(dv::LAB_RESET)));
        } else {
            hints.push((Act::Confirm, s.verb_confirm));
        }
        hints.extend([(Act::Tabs, s.verb_tab), (Act::Back, s.verb_close)]);
        self.push_outside(w.panel);
        let right = w.panel.x + w.panel.w;
        self.window_hints(f, &w, right, &hints);
    }

    /// 도구 줄의 상세: 이름 · 지금 쓰는 값 · (방금 한 일) · 하는 일.
    fn draw_lab_tool(&mut self, w: &Window, m: &Menu, at: usize) {
        let Some(row) = m.rows.get(at) else { return };
        let note = (self.lab.note.clone(), AMBER);
        self.draw_pane_head(
            w,
            &PaneHead {
                name: &row.label,
                gold: true,
                kind: format!("{} {}", t(dv::USING), self.lab_ab()),
                state: note,
                icon: false,
            },
        );
        let desc = match at.min(TOOLS - 1) {
            0 => dv::USING_D,
            1 => dv::STORE_B_D,
            2 => dv::EXPORT_D,
            _ => dv::RESET_ALL_D,
        };
        let lines = self.pane_wrap(w, t(desc), DIM);
        self.draw_pane(w, HEAD_H, &lines);
    }

    /// 값 줄의 상세: 화면 이름 · 원래 이름(`tuning.rs`·내보낸 줄의 말) · 지금/기본 · 한 걸음.
    fn draw_lab_value(&mut self, w: &Window, g: usize, at: usize) {
        let Some(&i) = lab::members(g).get(at) else {
            return;
        };
        let sp = &SPECS[i];
        let now = lab::value_text(self.t.get(i), sp.step);
        let def = lab::value_text(sp.def, sp.step);
        let changed = lab::changed(&self.t, i);
        self.draw_pane_head(
            w,
            &PaneHead {
                name: spec_label(sp.name),
                gold: changed,
                kind: sp.name.to_string(),
                state: (
                    if changed {
                        t(dv::LAB_CHANGED).to_string()
                    } else {
                        String::new()
                    },
                    AMBER,
                ),
                icon: false,
            },
        );
        let lines = vec![
            Ln::Pair(
                t(dv::LAB_NOW).into(),
                now,
                if changed { AMBER } else { INK },
            ),
            Ln::Pair(t(dv::LAB_DEF).into(), def, DIM),
            Ln::Pair(
                t(dv::LAB_STEP).into(),
                lab::value_text(sp.step, sp.step),
                DIM,
            ),
        ];
        self.draw_pane(w, HEAD_H, &lines);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lab_tabs_are_tools_then_every_group_and_map_back() {
        assert_eq!(lab_scene(0), Scene::Lab);
        assert_eq!(lab_tab(Scene::Lab), 0);
        for g in 0..GROUPS.len() {
            let s = lab_scene(g + 1);
            assert_eq!(s, Scene::LabGroup { g: g as u8 });
            assert_eq!(lab_tab(s), g + 1);
        }
        assert_eq!(lab_tabs(), GROUPS.len() + 1);
    }
}
