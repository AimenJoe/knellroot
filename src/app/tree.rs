//! 기술 나무 창(0106, v0.4.0 G2·G4 — 0102 4·7절, KB `meta-progression-unlock-planning`): 갈래 셋을 세로 다섯 칸으로 다 보이고,
//! 칸 밝기 네 단(열림 · 열 수 있음 · 점수 모자람 · 앞 칸 먼저 — 거기까지 n점), 아래에 고른 칸의 설명과 상태.
//! 화로에서 열면(`view = false`) 확인 = 묻는 창 → 연다. 멈춤에서 열면(`view = true`) 읽기 전용.
//! 아직 못 여는 칸에서 확인 = 목표 핀(꽂기·빼기) — 화로 줄과 이 창이 "목표까지 n점"을 보인다.
use super::menu::Hit;
use super::*;
use crate::progression::{CellState, BRANCHES, DEPTH, TREE};

/// 커서(`App::cursor`) = 갈래 × 다섯 + 깊이.
fn at(cursor: usize) -> (usize, usize) {
    let d = DEPTH as usize;
    ((cursor / d).min(2), cursor % d)
}

fn cursor_of(b: usize, d: usize) -> usize {
    b * DEPTH as usize + d
}

/// 십자 한 걸음: 좌우 = 갈래, 위아래 = 깊이(가장자리에서 멈춘다).
pub(super) fn tree_nav(cursor: usize, dx: i32, dy: i32) -> usize {
    let (b, d) = at(cursor);
    let b = (b as i32 + dx).clamp(0, 2) as usize;
    let d = (d as i32 + dy).clamp(0, DEPTH as i32 - 1) as usize;
    cursor_of(b, d)
}

impl App {
    pub(super) fn tick_tree(&mut self, a: &Actions, view: bool) {
        let tap = a.tap.and_then(|p| self.hit_at(p));
        if a.back || a.pause || tap == Some(Hit::Close) {
            self.go(Go::Back);
            return;
        }
        self.cursor = tree_nav(
            self.cursor,
            a.menu_right as i32 - a.menu_left as i32,
            a.menu_down as i32 - a.menu_up as i32,
        );
        // 터치·마우스: 다른 칸은 고르기만, 고른 칸을 다시 누르면 확인(장비 창과 같은 두 걸음).
        let confirm = match tap {
            Some(Hit::Cell(b, d)) => {
                let i = cursor_of(b, d);
                let again = self.cursor == i;
                self.cursor = i;
                again
            }
            _ => a.confirm,
        };
        if !confirm {
            return;
        }
        let (b, d) = at(self.cursor);
        let points = self.progress.tree_points();
        match self.progress.tree.state(b, d, points) {
            CellState::Open => self.snd(crate::sfx::Id::Deny),
            CellState::Ready if !view => self.ask(flow::Ask::Tree(b as u8)),
            CellState::Ready | CellState::Short | CellState::Far(_) => {
                let here = Some((b as u8, d as u8));
                let s = s();
                if self.progress.tree.pin == here {
                    self.progress.tree.pin = None;
                    self.toast = Some((s.toast_unpin.to_string(), 1.6));
                } else {
                    self.progress.tree.pin = here;
                    self.toast = Some((s.toast_pin.to_string(), 1.6));
                }
                self.snd(crate::sfx::Id::UiMove);
                self.persist();
            }
        }
    }

    /// 묻는 창에서 "열기": 그 갈래의 다음 칸(점수 1).
    pub(super) fn open_tree_cell(&mut self, b: usize) {
        let points = self.progress.tree_points();
        let d = self.progress.tree.open[b] as usize;
        if !self.progress.tree.open_next(b, points) {
            self.snd(crate::sfx::Id::Deny);
            return;
        }
        self.snd(crate::sfx::Id::Buy);
        let name = (TREE[b][d].name)(s());
        self.toast = Some((fill(s().toast_tree, &[name]), TOAST_TIME));
        self.persist();
    }

    /// 묻는 창의 글: "고른 호흡 — 1점. 열까요?".
    pub(super) fn ask_tree_text(&self, b: usize) -> String {
        let d = (self.progress.tree.open[b] as usize).min(DEPTH as usize - 1);
        fill(s().ask_tree, &[(TREE[b][d].name)(s())])
    }

    pub(super) fn draw_tree(&mut self, f: &Frame, view: bool) {
        let s = s();
        self.ui.dim(0.45);
        let u = self.ui.u(1.0);
        let px = self.ui.px(1.0);
        let panel = self.panel_rect(f, 440.0, 244.0);
        let top = self.ui.panel(panel, Some(s.tree_title));
        let x = self.ui.close(panel);
        self.hits.push((x, Hit::Close));
        let points = self.progress.tree_points();
        let tree = self.progress.tree;
        // 남은 점수(제목 줄 오른쪽, 가게의 잔불 자리).
        let purse = fill(s.tree_points, &[&tree.left(points).to_string()]);
        let pw = self.ui.width(&purse, 1.0);
        self.ui.text(
            &purse,
            panel.x + panel.w - u * 26.0 - pw,
            panel.y + u * 6.0,
            1.0,
            INK,
        );

        let (cur_b, cur_d) = at(self.cursor);
        let x0 = panel.x + u * 10.0;
        let col_w = (panel.w - u * 20.0) / 3.0;
        let cell_h = u * 19.0;
        let gap = u * 4.0;
        let t = get_time() as f32;
        let mut cells_bottom = top;
        for (b, br) in BRANCHES.iter().enumerate() {
            let cx = x0 + col_w * b as f32;
            let w = col_w - u * 8.0;
            let mid = cx + col_w * 0.5 - u * 4.0;
            let name = (br.name())(s);
            self.ui
                .text_center(name, mid, top, 1.0, if b == cur_b { AMBER } else { INK });
            // 태도 한 줄(0102) — 두 줄까지 접는다.
            let motto = ui::wrap(&self.ui, (br.motto())(s), w, 0.75);
            let mut y = top + px * 1.25;
            for line in motto.iter().take(2) {
                self.ui.text_center(line, mid, y, 0.75, DIM);
                y += px * 0.8;
            }
            let cells_top = top + px * 1.25 + px * 0.8 * 2.0 + u * 4.0;
            for (d, def) in TREE[b].iter().enumerate() {
                let r = Rect::new(cx, cells_top + (cell_h + gap) * d as f32, w, cell_h);
                let state = tree.state(b, d, points);
                let sel = b == cur_b && d == cur_d;
                self.draw_tree_cell(r, (def.name)(s), state, sel, t);
                if tree.pin == Some((b as u8, d as u8)) {
                    self.ui
                        .text("◆", r.x + r.w - u * 9.0, r.y + u * 1.5, 0.7, AMBER);
                }
                self.hits.push((r, Hit::Cell(b, d)));
                cells_bottom = cells_bottom.max(r.y + r.h);
            }
        }
        // 아래: 고른 칸 — 이름 · 상태, 설명 두 줄.
        let def = &TREE[cur_b][cur_d];
        let state = tree.state(cur_b, cur_d, points);
        let status = match state {
            CellState::Open => s.tree_open.to_string(),
            CellState::Ready if view => s.tree_view_only.to_string(),
            CellState::Ready => s.tree_ready.to_string(),
            CellState::Short => s.tree_short.to_string(),
            CellState::Far(n) => fill(s.tree_far, &[&n.to_string()]),
        };
        let goal = tree
            .cells_to_pin()
            .map(|n| fill(s.tree_goal, &[&n.to_string()]));
        let status = match goal {
            Some(g) if !matches!(state, CellState::Open) => format!("{status} · {g}"),
            _ => status,
        };
        let nx = panel.x + u * 12.0;
        let nw = panel.w - u * 24.0;
        let mut y = cells_bottom + u * 7.0;
        let name = (def.name)(s);
        self.ui.text(name, nx, y, 1.0, AMBER);
        let name_w = self.ui.width(name, 1.0) + u * 8.0;
        let status = self.ui.fit(&status, nw - name_w, 0.9);
        self.ui.text(&status, nx + name_w, y + u, 0.9, DIM);
        y += px * 1.35;
        for line in ui::wrap(&self.ui, (def.desc)(s), nw, 0.9).iter().take(2) {
            self.ui.text(line, nx, y, 0.9, INK);
            y += px * 0.95;
        }
        self.push_outside(panel);
        self.hint_line(
            &[(Act::Choose, s.verb_choose), (Act::Confirm, s.verb_confirm)],
            f.ox + f.w * 0.5,
            f.oy + f.h - px * 2.2,
            f.w * 0.94,
        );
    }

    /// 칸 하나(밝기 네 단): 열림 = 호박빛 면, 열 수 있음 = 호박 테가 숨 쉼, 모자람 = 보통 테, 앞 칸 먼저 = 어둡게.
    /// 고른 칸은 밝은 테 두 겹. 이름은 두 줄까지 접는다(잘라 내지 않는다 — F4 `fit` 검사).
    fn draw_tree_cell(&self, r: Rect, name: &str, state: CellState, sel: bool, t: f32) {
        let u = self.ui.u(1.0);
        let (fill_c, edge, ink) = match state {
            CellState::Open => (Color::new(AMBER.r, AMBER.g, AMBER.b, 0.32), AMBER, INK),
            CellState::Ready => {
                let k = 0.6 + 0.4 * (t * 3.0).sin();
                (
                    Color::new(0.16, 0.13, 0.2, 1.0),
                    Color::new(AMBER.r, AMBER.g, AMBER.b, k),
                    INK,
                )
            }
            CellState::Short => (Color::new(0.12, 0.1, 0.15, 1.0), ui::BORDER, DIM),
            CellState::Far(_) => (
                Color::new(0.07, 0.06, 0.09, 1.0),
                Color::new(ui::BORDER.r, ui::BORDER.g, ui::BORDER.b, 0.5),
                Color::new(DIM.r, DIM.g, DIM.b, 0.6),
            ),
        };
        draw_rectangle(r.x, r.y, r.w, r.h, fill_c);
        draw_rectangle_lines(r.x, r.y, r.w, r.h, u.max(1.0), edge);
        if sel {
            draw_rectangle_lines(
                r.x - u * 1.5,
                r.y - u * 1.5,
                r.w + u * 3.0,
                r.h + u * 3.0,
                (u * 1.2).max(1.0),
                INK,
            );
        }
        let k = 0.8;
        let lines = ui::wrap(&self.ui, name, r.w - u * 6.0, k);
        let px = self.ui.px(k);
        let n = lines.len().min(2) as f32;
        let mut y = r.y + (r.h - px * (0.95 * (n - 1.0) + 1.0)) * 0.5;
        for line in lines.iter().take(2) {
            self.ui.text_center(line, r.x + r.w * 0.5, y, k, ink);
            y += px * 0.95;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tree_cursor_walks_branches_sideways_and_depth_up_and_down() {
        assert_eq!(tree_nav(0, 1, 0), cursor_of(1, 0));
        assert_eq!(
            tree_nav(cursor_of(2, 0), 1, 0),
            cursor_of(2, 0),
            "stops at the edge"
        );
        assert_eq!(tree_nav(cursor_of(1, 4), 0, 1), cursor_of(1, 4));
        assert_eq!(tree_nav(cursor_of(1, 1), -1, 2), cursor_of(0, 3));
        assert_eq!(at(cursor_of(2, 4)), (2, 4));
    }
}
