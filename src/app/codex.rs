//! 큰 창의 도감 탭(0130 C5 — 사장 2026-11-09 인터뷰: 깨어난 것을 가까이 본 때 적힘, 본 만큼 관찰 줄이 열림, 모델 + 실루엣 → 0164 탭).
//! 왼쪽 = 묶음 머리(적 · 장신구 — 좌우·누르기; 위 탭과 버튼이 다르다) + 목록(위아래·굴리기), 오른쪽 = 고른 것의 그림과 글.
//! 적: 본 것 = 돌아가는 모델 + 이름 + 관찰(`·`)·추측(`?`) 줄, 못 본 것 = 검은 실루엣 + "???" + 나오는 곳.
//! 장신구: 가진 것 = 돌아가는 그림 + 이름 + 수치 + 세계 안의 말(0129), 못 가진 것 = "???" + 나오는 곳.
use super::menu::Hit;
use super::tabs::{Ln, PaneHead, HEAD_H};
use super::*;
use crate::codex::{self, ENTRIES};
use flow::Host;

/// 몸 안의 묶음 둘(적 · 장신구).
const GROUPS: usize = 2;
/// 묶음 머리 줄의 높이(기준 px) — 그 아래부터 목록.
const GROUP_H: f32 = 18.0;
/// 오른쪽 칸의 그림 자리(기준 px): 적은 선 몸(이름 아래의 세로로 긴 칸), 장신구는 머리의 그림 칸.
const PIC_TOP: f32 = 17.0;
const PIC_W: f32 = 52.0;
const PIC_H: f32 = 72.0;

impl App {
    /// 도감 한 틱: 뒤로·✕·창 밖 = 닫기, 좌우·묶음 머리 누르기 = 묶음(적 ↔ 장신구 — 위 탭은 스킬 버튼이 넘긴다),
    /// 위아래·굴리기·줄 누르기 = 고르기(고름은 보기만 — 할 일 없음).
    pub(super) fn tick_codex(&mut self, a: &Actions, host: Host, tab: usize) {
        let mut a = *a;
        let before = self.cursor;
        self.pane_scroll_lines(&mut a);
        let a = &a;
        let tap = a.tap.and_then(|p| self.hit_at(p));
        if a.back || a.pause || tap == Some(Hit::Close) {
            self.go(Go::Back);
            return;
        }
        let step = a.menu_right as i32 - a.menu_left as i32;
        let to = match tap {
            Some(Hit::Cell(1, i)) => Some(i),
            _ if step != 0 => Some((tab as i32 + step).rem_euclid(GROUPS as i32) as usize),
            _ => None,
        };
        if let Some(to) = to.filter(|&i| i != tab) {
            // 묶음은 장면 칸의 상태(0066) — 지금 칸을 새 묶음으로 바꾼다(커서는 첫 줄).
            self.go(Go::Swap(Scene::Codex {
                host,
                tab: to as u8,
            }));
            self.snd(crate::sfx::Id::UiMove);
            return;
        }
        let n = if tab == 0 {
            ENTRIES.len()
        } else {
            crate::charms::TABLE.len()
        };
        let _ = self.menu_pick(a, n);
        if self.cursor != before {
            self.scroll = 0.0;
        }
    }

    /// 도감 탭 그림(0164): 왼쪽 = 묶음 머리(◀ 적 · 장신구 ▶) + 목록, 오른쪽 = 고른 것의 그림 · 이름 · 글.
    pub(super) fn draw_codex(&mut self, f: &Frame, host: Host, tab: usize) {
        let s = s();
        let scene = Scene::Codex {
            host,
            tab: tab as u8,
        };
        let w = self.draw_tab_window(f, scene);
        let (k, u) = (w.k, w.s);
        // 묶음 머리: 위 탭과 다른 꼴 — 밑줄 하나짜리 작은 글 둘, 양 끝에 ◀ ▶(좌우로 넘긴다).
        let names = [s.codex_tab_foes, s.codex_tab_charms];
        let tk = 0.85 * k;
        let ty = w.at(0.0, 2.0).y;
        let mut x = w.at(4.0, 0.0).x;
        self.ui.text("◀", x, ty, tk, DIM);
        x += self.ui.width("◀", tk) + u * 6.0;
        for (i, name) in names.iter().enumerate() {
            let tw = self.ui.width(name, tk);
            let on = i == tab;
            self.ui.text(name, x, ty, tk, if on { AMBER } else { DIM });
            if on {
                draw_rectangle(
                    x,
                    (ty + self.ui.px(tk) + u * 1.5).round(),
                    tw,
                    u.round().max(1.0),
                    AMBER,
                );
            }
            self.hits.push((
                Rect::new(x - u * 5.0, w.origin.y, tw + u * 10.0, GROUP_H * u),
                Hit::Cell(1, i),
            ));
            x += tw + u * 12.0;
        }
        self.ui.text("▶", x - u * 6.0, ty, tk, DIM);
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
        let cursor = self.cursor.min(rows.len().saturating_sub(1));
        self.draw_left_list(&w, &rows, GROUP_H);
        // 오른쪽: 고른 것.
        let t = get_time() as f32;
        let mut lines: Vec<Ln> = Vec::new();
        if tab == 0 {
            let e = &ENTRIES[cursor.min(ENTRIES.len() - 1)];
            let seen = codex::seen(rec, e.kind);
            // 선 몸은 세로로 길다 — 머리의 그림 칸 대신 이름 아래 왼쪽에 세워 두고, 나오는 곳은 그 오른쪽에, 관찰은 그 아래로 흐른다.
            let name = if seen {
                tr(crate::species::def(e.kind).name)
            } else {
                s.codex_unknown
            };
            let p = w.at(tabs::RIGHT_X, 0.0);
            let fit = self.ui.fit(name, tabs::RIGHT_W * u, tabs::K_NAME * k);
            let ink = if seen { AMBER } else { INK };
            self.ui.text(&fit, p.x, p.y, tabs::K_NAME * k, ink);
            let pic = w.rect(tabs::RIGHT_X, PIC_TOP, PIC_W, PIC_H);
            self.renderer
                .draw_creature(e.kind, pic, 0.6 * (t * 0.6).sin(), t, !seen);
            let tx = tabs::RIGHT_X + PIC_W + 6.0;
            let room = (tabs::RIGHT_X + tabs::RIGHT_W - tx) * u;
            let kk = tabs::K_KIND * k;
            let mut y = PIC_TOP + 2.0;
            for part in ui::wrap(&self.ui, &fill(s.codex_place, &[tr(e.place)]), room, kk)
                .iter()
                .take(6)
            {
                self.ui
                    .text(part, w.at(tx, 0.0).x, w.at(0.0, y).y, kk, PLACE);
                y += 9.5;
            }
            if seen {
                for (guess, l) in codex::open_lines(rec, e) {
                    let head = if guess { "?" } else { "·" };
                    let col = if guess { GUESS } else { DIM };
                    lines.extend(self.pane_wrap(&w, &format!("{head} {}", tr(l)), col));
                    lines.push(Ln::Gap);
                }
            } else {
                lines.extend(self.pane_wrap(&w, s.codex_not_seen, DIM));
            }
            self.draw_pane(&w, PIC_TOP + PIC_H + 4.0, &lines);
        } else {
            let d = &crate::charms::TABLE[cursor.min(crate::charms::TABLE.len() - 1)];
            let owned = self.progress.owned.contains(&d.id);
            let icon = self.draw_pane_head(
                &w,
                &PaneHead {
                    name: if owned { tr(d.name) } else { s.codex_unknown },
                    gold: owned,
                    kind: s.codex_tab_charms.to_string(),
                    state: (String::new(), DIM),
                    icon: true,
                },
            );
            if owned {
                self.draw_pane_item(crate::item_view::Item::Charm(d.id), icon);
                lines.extend(self.pane_wrap(&w, tr(d.desc), DIM));
                lines.push(Ln::Gap);
                lines.extend(self.pane_wrap(&w, tr(d.flavor), GUESS));
            } else {
                self.ui.text_center(
                    "?",
                    icon.x + icon.w * 0.5,
                    icon.y + (icon.h - self.ui.px(1.6 * k)) * 0.5,
                    1.6 * k,
                    ui::BORDER,
                );
                lines.extend(self.pane_wrap(&w, s.codex_not_owned, DIM));
            }
            lines.push(Ln::Gap);
            lines.extend(self.pane_wrap(
                &w,
                &fill(s.codex_place, &[tr(codex::charm_place(d.id))]),
                PLACE,
            ));
            self.draw_pane(&w, HEAD_H, &lines);
        }
        let right = w.panel.x + w.panel.w;
        self.tab_foot(f, &w, scene, right, &[(Act::Walk, s.verb_choose)]);
    }
}

/// 추측 줄·세계 안의 말: 관찰보다 옅고 푸르스름하게(0129 장비 창과 같은 결).
const GUESS: Color = Color::new(0.55, 0.56, 0.62, 1.0);
/// 나오는 곳: 옅은 호박빛(길 안내 색의 결).
const PLACE: Color = Color::new(0.78, 0.66, 0.46, 1.0);
