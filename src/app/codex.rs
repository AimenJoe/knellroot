//! 도감 창(0130 C5 — 사장 2026-11-09 인터뷰: 멈춤 메뉴 줄, 깨어난 것을 가까이 본 때 적힘, 본 만큼 관찰 줄이 열림, 모델 + 실루엣).
//! 꼴은 개발 창(0083, nail 0269): 위 탭 줄(적 · 장신구 — 좌우·스킬 버튼(0151)·탭) · 왼쪽 목록(위아래·굴리기) · 오른쪽 그림과 글 · 아래 안내.
//! 적: 본 것 = 돌아가는 모델 + 이름 + 관찰(`·`)·추측(`?`) 줄, 못 본 것 = 검은 실루엣 + "???" + 나오는 곳.
//! 장신구: 가진 것 = 돌아가는 그림 + 이름 + 수치 + 세계 안의 말(0129), 못 가진 것 = "???" + 나오는 곳.
use super::menu::Hit;
use super::*;
use crate::codex::{self, ENTRIES};

const TABS: usize = 2;

impl App {
    /// 도감 한 틱: 뒤로·✕·창 밖 = 닫기, 좌우·스킬 버튼(0151)·탭 = 탭, 위아래·굴리기·줄 누르기 = 고르기(고름은 보기만 — 할 일 없음).
    pub(super) fn tick_codex(&mut self, a: &Actions, tab: usize) {
        let tap = a.tap.and_then(|p| self.hit_at(p));
        if a.back || a.pause || tap == Some(Hit::Close) {
            self.go(Go::Back);
            return;
        }
        let step = (a.menu_right || a.skill2) as i32 - (a.menu_left || a.skill1) as i32;
        let to = match tap {
            Some(Hit::Cell(1, i)) => Some(i),
            _ if step != 0 => Some((tab as i32 + step).rem_euclid(TABS as i32) as usize),
            _ => None,
        };
        if let Some(to) = to.filter(|&i| i != tab) {
            // 탭은 장면 칸의 상태(0066) — 지금 칸을 새 탭으로 바꿔 연다(커서는 첫 줄).
            self.go(Go::Back);
            self.go(Go::Open(Scene::Codex { tab: to as u8 }));
            self.snd(crate::sfx::Id::UiMove);
            return;
        }
        let n = if tab == 0 {
            ENTRIES.len()
        } else {
            crate::charms::TABLE.len()
        };
        let _ = self.menu_pick(a, n);
    }

    pub(super) fn draw_codex(&mut self, f: &Frame, tab: usize) {
        let s = s();
        self.ui.dim(0.45);
        let u = self.ui.u(1.0);
        let px = self.ui.px(1.0);
        let panel = self.panel_rect(f, 440.0, 236.0);
        let top = self.ui.panel(panel, Some(s.codex_title));
        let x = self.ui.close(panel);
        self.hits.push((x, Hit::Close));
        let (lx, rx_end) = (panel.x + u * 10.0, panel.x + panel.w - u * 10.0);
        // 탭 둘: 글자 폭 + 여유, 고른 탭은 밝은 단추.
        let tab_h = px * 1.6;
        let names = [s.codex_tab_foes, s.codex_tab_charms];
        let mut tx = lx;
        for (i, name) in names.iter().enumerate() {
            let r = Rect::new(tx, top, self.ui.width(name, 1.0) + u * 24.0, tab_h);
            self.ui.button(r, name, i == tab);
            self.hits.push((r, Hit::Cell(1, i)));
            tx += r.w + u * 4.0;
        }
        let body = top + tab_h + u * 8.0;
        let hint_y = panel.y + panel.h - u * 8.0 - px;
        let split = panel.x + panel.w * 0.36;
        draw_line(split, body, split, hint_y - u * 4.0, u.max(1.0), ui::BORDER);
        // 왼쪽 목록: 본 것·가진 것은 이름, 아니면 "???".
        let rec = &self.progress.codex.clone();
        let rows: Vec<ui::Row> = if tab == 0 {
            ENTRIES
                .iter()
                .map(|e| {
                    ui::Row::plain(if codex::seen(rec, e.kind) {
                        tr(crate::species::def(e.kind).name)
                    } else {
                        s.codex_unknown
                    })
                })
                .collect()
        } else {
            crate::charms::TABLE
                .iter()
                .map(|d| {
                    ui::Row::plain(if self.progress.owned.contains(&d.id) {
                        tr(d.name)
                    } else {
                        s.codex_unknown
                    })
                })
                .collect()
        };
        let area = Rect::new(
            lx - u * 4.0,
            body,
            split - lx - u * 4.0,
            hint_y - u * 6.0 - body,
        );
        let cursor = self.cursor.min(rows.len().saturating_sub(1));
        let hits = self.ui.list(&rows, cursor, &mut self.view, area);
        self.push_hits(&hits);
        // 오른쪽: 위에 그림, 그 오른쪽에 이름, 아래로 글.
        let dx = split + u * 10.0;
        let w = rx_end - dx;
        let t = get_time() as f32;
        let pic = Rect::new(dx, body, u * 52.0, u * 78.0);
        let text_x = pic.x + pic.w + u * 8.0;
        let text_w = rx_end - text_x;
        let line_h = px * 1.3;
        let mut lines: Vec<(String, Color)> = Vec::new();
        let mut name = s.codex_unknown.to_string();
        if tab == 0 {
            let e = &ENTRIES[cursor.min(ENTRIES.len() - 1)];
            let seen = codex::seen(rec, e.kind);
            self.renderer
                .draw_creature(e.kind, pic, 0.6 * (t * 0.6).sin(), t, !seen);
            if seen {
                name = tr(crate::species::def(e.kind).name).to_string();
                for (guess, l) in codex::open_lines(rec, e) {
                    let head = if guess { "?" } else { "·" };
                    let col = if guess { GUESS } else { DIM };
                    lines.push((format!("{head} {}", tr(l)), col));
                }
            } else {
                lines.push((s.codex_not_seen.to_string(), DIM));
            }
            lines.push((fill(s.codex_place, &[tr(e.place)]), PLACE));
        } else {
            let d = &crate::charms::TABLE[cursor.min(crate::charms::TABLE.len() - 1)];
            let owned = self.progress.owned.contains(&d.id);
            if owned {
                let item = crate::item_view::Item::Charm(d.id);
                let side = pic.w.min(pic.h);
                self.renderer
                    .draw_item(item, Rect::new(pic.x, pic.y, side, side), t * 0.8);
                name = tr(d.name).to_string();
                lines.push((tr(d.desc).to_string(), DIM));
                lines.push((tr(d.flavor).to_string(), GUESS));
            } else {
                self.ui.text_center(
                    "?",
                    pic.x + pic.w * 0.5,
                    pic.y + pic.h * 0.35,
                    2.0,
                    ui::BORDER,
                );
                lines.push((s.codex_not_owned.to_string(), DIM));
            }
            lines.push((fill(s.codex_place, &[tr(codex::charm_place(d.id))]), PLACE));
        }
        let name = self.ui.fit(&name, text_w, 1.0);
        self.ui.text(&name, text_x, body, 1.0, AMBER);
        // 글은 이름 오른쪽에서 시작해 그림 아래로 넘어가면 칸 전체 폭을 쓴다.
        let mut y = body + px * 1.6;
        let bottom = hint_y - u * 6.0;
        for (l, col) in lines {
            let (lx2, lw) = if y < pic.y + pic.h {
                (text_x, text_w)
            } else {
                (dx, w)
            };
            for part in ui::wrap(&self.ui, &l, lw, 0.9) {
                if y + px > bottom {
                    break;
                }
                let x0 = if y < pic.y + pic.h { text_x } else { lx2 };
                self.ui.text(&part, x0, y, 0.9, col);
                y += line_h;
            }
            y += line_h * 0.3;
        }
        let hint = self.ui.fit(s.codex_hint, panel.w - u * 20.0, 0.85);
        self.ui.text(&hint, lx, hint_y, 0.85, DIM);
        self.push_outside(panel);
    }
}

/// 추측 줄·세계 안의 말: 관찰보다 옅고 푸르스름하게(0129 장비 창과 같은 결).
const GUESS: Color = Color::new(0.55, 0.56, 0.62, 1.0);
/// 나오는 곳: 옅은 호박빛(길 안내 색의 결).
const PLACE: Color = Color::new(0.78, 0.66, 0.46, 1.0);
