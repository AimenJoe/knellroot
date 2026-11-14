//! 큰 창의 상태 탭(0164 — 0160 시안 2. 옛 화로 목록과 레벨 창 0104가 여기로): 멈춤과 화로가 이 탭에서 열린다.
//! 왼쪽 = 방랑자 인형과 체력·기력 막대 + **고르는 줄**(레벨 셋 — 생명·기력·힘: 단의 점 다섯과 다음 단의 값, 그 아래 끼운 기술 —
//! 스킬 두 칸·큰 기술·패시브, 0173: 그전에는 장신구 세 칸이었다 — 장신구는 장비 탭이 이미 보여 준다, 사장 "차라리 패시브나 선택되어
//! 있는 기술이"), 오른쪽 = 고른 줄의 상세(올리면 바뀌는 "지금 → 다음", 스킬의 설명, 연 패시브)와 지금의 수치(`stats::lines`).
//! 화로에서는 레벨 줄에서 확인 = 올리기(묻는 창), 멈춤에서는 읽기만. 기술 줄에서 확인 = 기술 탭의 그 칸으로(탭 사이의 이음).
//! 값은 `progression.rs`.
use super::equip::{stat_name, stat_value};
use super::frame::Window;
use super::menu::Hit;
use super::tabs::{Ln, PaneHead, HEAD_H};
use super::*;
use crate::progression::{Level, Line as LvLine, LINES as LV_LINES, MAX_RANK};
use crate::skills::{self as sk, Entry, ENTRIES};
use crate::stats::Change;
use flow::Host;

/// 기술 묶음의 줄: 스킬 두 칸 · 큰 기술 · 패시브.
pub(super) const SKILL_ROWS: usize = 4;
/// 고르는 줄의 수: 레벨 셋 + 기술 묶음.
pub(super) const ROWS: usize = LV_LINES.len() + SKILL_ROWS;

/// 자리(480×270 기준 px, 몸의 왼쪽 위가 원점): 인형 단(왼쪽)의 폭, 고르는 줄 단의 왼쪽 끝과 폭.
const DOLL_W: f32 = 92.0;
const PICK_X: f32 = 100.0;
const PICK_W: f32 = tabs::LEFT_W - PICK_X - 6.0;
/// 묶음 머리글(레벨 n · 기술)의 높이, 레벨 줄·기술 줄의 높이, 두 묶음 사이.
const HEAD: f32 = 14.0;
const LEVEL_H: f32 = 29.0;
const CHARM_H: f32 = 19.0;
const GROUP_GAP: f32 = 4.0;
/// 단의 점: 한 변, 사이.
const PIP: f32 = 6.0;
const PIP_GAP: f32 = 2.0;
/// 막대(체력·기력): 폭·높이, 이름표까지 한 묶음의 높이.
const BAR_W: f32 = 80.0;
const BAR_H: f32 = 5.0;
const BAR_ROW: f32 = 17.0;

/// 기력 막대의 풀빛(HUD의 기력 막대와 같은 결), 빈 막대.
const STAMINA: Color = Color::new(0.55, 0.78, 0.5, 1.0);
const BAR_OFF: Color = Color::new(0.2, 0.18, 0.17, 0.9);

/// 고르는 줄 하나.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Row {
    Level(LvLine),
    /// 끼운 스킬 칸(0·1).
    Skill(usize),
    /// 끼운 큰 기술.
    Ult,
    /// 연 패시브(모두).
    Passives,
}

/// 커서 자리 → 줄(레벨 셋이 먼저, 그 아래 스킬 두 칸 · 큰 기술 · 패시브).
pub(super) fn row(i: usize) -> Option<Row> {
    match LV_LINES.get(i) {
        Some(&l) => Some(Row::Level(l)),
        None => match i.checked_sub(LV_LINES.len())? {
            j @ 0..=1 => Some(Row::Skill(j)),
            2 => Some(Row::Ult),
            3 => Some(Row::Passives),
            _ => None,
        },
    }
}

/// 기술 줄에 놓인 것(배운 것만 — 안 배운 스킬이 놓인 칸·큰 기술이 없는 판은 빈 칸).
pub(super) fn worn_entry(
    what: Row,
    board: &sk::Board,
    worn: [sk::Id; 2],
    great: sk::Ult,
) -> Option<Entry> {
    match what {
        Row::Skill(j) => board.learned(worn[j]).then_some(Entry::Skill(worn[j])),
        Row::Ult => {
            let u = board.wearable_ult(great);
            board.has_ult(u).then_some(Entry::Ult(u))
        }
        Row::Level(_) | Row::Passives => None,
    }
}

/// 기술 줄에서 확인하면 가는 기술 탭의 칸(`ENTRIES`의 자리): 놓인 것이 있으면 그 칸, 없으면 그 갈래의 첫 칸.
pub(super) fn skills_cursor(
    what: Row,
    board: &sk::Board,
    worn: [sk::Id; 2],
    great: sk::Ult,
) -> usize {
    let first = |f: fn(&Entry) -> bool| ENTRIES.iter().position(f).unwrap_or(0);
    match (worn_entry(what, board, worn, great), what) {
        (Some(e), _) => ENTRIES.iter().position(|x| *x == e).unwrap_or(0),
        (None, Row::Ult) => first(|e| matches!(e, Entry::Ult(_))),
        (None, Row::Passives) => first(|e| matches!(e, Entry::Cell(_))),
        (None, _) => first(|e| matches!(e, Entry::Skill(_))),
    }
}

/// 연 패시브 칸들(판의 차례).
pub(super) fn open_cells(board: &sk::Board) -> Vec<Entry> {
    ENTRIES
        .iter()
        .copied()
        .filter(|e| matches!(e, Entry::Cell(_)) && board.rank(*e) >= 1)
        .collect()
}

/// 패시브 칸의 수.
pub(super) fn cell_count() -> usize {
    ENTRIES
        .iter()
        .filter(|e| matches!(e, Entry::Cell(_)))
        .count()
}

/// 줄의 위(기준 px).
fn row_top(i: usize) -> f32 {
    let n = LV_LINES.len();
    if i < n {
        HEAD + LEVEL_H * i as f32
    } else {
        HEAD + LEVEL_H * n as f32 + GROUP_GAP + HEAD + CHARM_H * (i - n) as f32
    }
}

/// 레벨 줄에서 지금 할 수 있는 것.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum LevelState {
    /// 다 올렸다.
    Full,
    /// 쉬는 동안이 아니다(멈춤) — 읽기만. 값은 다음 단의 잔불.
    View(u32),
    /// 잔불이 모자란다.
    Poor(u32),
    /// 올릴 수 있다.
    Ready(u32),
}

pub(super) fn level_state(lv: &Level, l: LvLine, embers: u32, edits: bool) -> LevelState {
    match lv.next_cost(l) {
        None => LevelState::Full,
        Some(c) if !edits => LevelState::View(c),
        Some(c) if embers < c => LevelState::Poor(c),
        Some(c) => LevelState::Ready(c),
    }
}

/// 확인하면 할 일.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Do {
    /// 올릴까 묻는다(`progression::LINES`의 자리).
    Ask(u8),
    /// 안 된다 — 까닭(알림이 있으면 그 글).
    Deny(LevelState),
    /// 기술 탭으로(그 줄에 놓인 칸 — 앱이 `skills_cursor`로 자리를 찾는다).
    Skills(Row),
}

pub(super) fn confirm(i: usize, lv: &Level, embers: u32, edits: bool) -> Option<Do> {
    Some(match row(i)? {
        Row::Level(l) => match level_state(lv, l, embers, edits) {
            LevelState::Ready(_) => Do::Ask(i as u8),
            st => Do::Deny(st),
        },
        what => Do::Skills(what),
    })
}

/// 레벨 줄 이름 · 그 아래 작은 한마디.
fn line_name(l: LvLine) -> &'static str {
    let s = s();
    match l {
        LvLine::Life => s.lv_life,
        LvLine::Stamina => s.lv_stamina,
        LvLine::Might => s.lv_might,
    }
}

fn line_sub(l: LvLine) -> &'static str {
    let s = s();
    match l {
        LvLine::Life => s.lv_life_sub,
        LvLine::Stamina => s.lv_stamina_sub,
        LvLine::Might => s.lv_might_sub,
    }
}

impl App {
    /// 상태 탭 한 틱: 위아래 = 줄(끝에서 돈다), 확인·고른 줄 다시 누르기 = 그 줄의 일, 뒤로·✕·창 밖 = 닫기.
    pub(super) fn tick_status(&mut self, a: &Actions, host: Host) {
        let mut a = *a;
        let before = self.cursor;
        self.pane_scroll_lines(&mut a);
        let a = &a;
        let tap = a.tap.and_then(|p| self.hit_at(p));
        if a.back || a.pause || tap == Some(Hit::Close) {
            self.go(Go::Back);
            return;
        }
        ui::menu_input(&mut self.cursor, ROWS, a.menu_up, a.menu_down);
        // 터치·마우스: 다른 줄은 고르기만(상세가 바뀐다), 고른 줄을 다시 누르면 확인 — 장비·기술 탭과 같은 두 걸음.
        let confirm_now = match tap {
            Some(Hit::Row(i)) => {
                let again = self.cursor == i;
                self.cursor = i;
                again
            }
            _ => a.confirm,
        };
        if self.cursor != before {
            self.scroll = 0.0;
        }
        if !confirm_now {
            return;
        }
        let edits = host.edits();
        match confirm(self.cursor, &self.progress.level, self.world.embers, edits) {
            Some(Do::Ask(i)) => self.ask(flow::Ask::Level(i)),
            // 기술 줄 → 기술 탭의 그 칸(탭을 바꾼다 — 쌓지 않는다).
            Some(Do::Skills(what)) => {
                let at = skills_cursor(
                    what,
                    &self.board(),
                    self.progress.skills.0,
                    self.progress.ult,
                );
                self.show(Go::Swap(Scene::Skills { host }));
                self.cursor = at;
            }
            Some(Do::Deny(st)) => {
                // 까닭을 상태 줄 자리에 한 번 밝게(가게와 같다 — 0173: 멈춤에서는 소리만 나서 왜 안 오르는지 몰랐다).
                match st {
                    LevelState::Poor(_) => self.toast = Some((s().shop_poor.to_string(), 1.6)),
                    LevelState::View(_) => self.toast = Some((s().toast_away.to_string(), 1.6)),
                    LevelState::Full | LevelState::Ready(_) => {}
                }
                self.snd(crate::sfx::Id::Deny);
            }
            None => {}
        }
    }

    /// 상태 탭 그림.
    pub(super) fn draw_status(&mut self, f: &Frame, host: Host) {
        let s = s();
        let scene = Scene::Status { host };
        let w = self.draw_tab_window(f, scene);
        let at = self.cursor.min(ROWS - 1);
        let edits = host.edits();
        self.draw_status_doll(&w);
        self.draw_status_rows(&w, at, edits);
        self.draw_status_detail(&w, at, edits);
        // 아래 안내: 확인이 하는 일은 줄마다 다르다 — 레벨 줄(화로) = 올리기, 기술 줄 = 기술 탭으로. 멈춤의 레벨 줄은 읽기만
        // (오른쪽 끝의 "보기만" 표지가 말한다, 0173).
        let mut acts = vec![(Act::Choose, s.verb_choose)];
        match row(at) {
            Some(Row::Level(_)) if edits => acts.push((Act::Confirm, s.ask_level_yes)),
            Some(Row::Level(_)) | None => {}
            Some(_) => acts.push((Act::Confirm, s.menu_skills)),
        }
        let right = if edits || matches!(self.scene(), Scene::Ask(_)) {
            w.panel.x + w.panel.w
        } else {
            self.draw_view_note(f, &w)
        };
        self.tab_foot(f, &w, scene, right, &acts);
    }

    /// 왼쪽 단: 이름 · 총 레벨 · 인형(든 무기를 든 3D — 장비 탭과 같은 것) · 체력과 기력의 막대.
    fn draw_status_doll(&mut self, w: &Window) {
        let s = s();
        let (k, u) = (w.k, w.s);
        let cx = w.at(DOLL_W * 0.5, 0.0).x;
        self.ui.text_center(s.status_name, cx, w.origin.y, k, AMBER);
        let lv = fill(s.level_title, &[&self.progress.level.total().to_string()]);
        self.ui
            .text_center(&lv, cx, w.at(0.0, 15.0).y, tabs::K_KIND * k, DIM);
        // 막대는 단의 바닥에, 인형은 이름과 막대 사이 가운데 — 정수 배율(게임 화면과 같은 도트 굵기).
        let bars_top = w.body_h - BAR_ROW * 2.0;
        let (top, room) = (25.0, bars_top - 25.0 - 2.0);
        let m = (DOLL_W * u / crate::render::DOLL_W as f32)
            .min(room * u / crate::render::DOLL_H as f32)
            .floor()
            .max(1.0);
        let (dw, dh) = (
            crate::render::DOLL_W as f32 * m,
            crate::render::DOLL_H as f32 * m,
        );
        let doll = Rect::new(
            (cx - dw * 0.5).round(),
            (w.at(0.0, top).y + (room * u - dh) * 0.5).round(),
            dw,
            dh,
        );
        let t = get_time() as f32;
        self.renderer
            .draw_doll(self.weapon, doll, 0.55 * (t * 0.7).sin(), t);
        let pl = &self.world.player;
        let smax = self.world.stats.max_stamina().max(1.0);
        let bars = [
            (
                fill(
                    s.status_hp,
                    &[&pl.hp.max(0).to_string(), &pl.max_hp.to_string()],
                ),
                pl.hp.max(0) as f32 / pl.max_hp.max(1) as f32,
                AMBER,
            ),
            (
                fill(
                    s.status_stamina,
                    &[
                        &(pl.stamina.round() as i32).to_string(),
                        &(smax.round() as i32).to_string(),
                    ],
                ),
                pl.stamina / smax,
                STAMINA,
            ),
        ];
        let x = (DOLL_W - BAR_W) * 0.5;
        for (i, (label, part, ink)) in bars.into_iter().enumerate() {
            let y = bars_top + BAR_ROW * i as f32;
            let p = w.at(x, y);
            let label = self.ui.fit(&label, BAR_W * u, tabs::K_KIND * k);
            self.ui.text(&label, p.x, p.y, tabs::K_KIND * k, DIM);
            let r = w.rect(x, y + 10.0, BAR_W, BAR_H);
            draw_rectangle(r.x, r.y, r.w, r.h, BAR_OFF);
            draw_rectangle(r.x, r.y, (r.w * part.clamp(0.0, 1.0)).round(), r.h, ink);
        }
    }

    /// 고르는 줄: 레벨 셋(이름 · 한마디 · 단의 점 다섯 · 다음 단의 값) + 기술 넷(그림 · 이름 → 기술 탭). 누를 자리를 채운다.
    fn draw_status_rows(&mut self, w: &Window, at: usize, edits: bool) {
        let s = s();
        let (k, u) = (w.k, w.s);
        let lv = self.progress.level;
        let embers = self.world.embers;
        let kk = tabs::K_KIND * k;
        // 묶음 머리글: 작은 글 + 가는 선.
        let n = LV_LINES.len();
        let heads = [
            (0.0, s.status_levels.to_string()),
            (
                HEAD + LEVEL_H * n as f32 + GROUP_GAP,
                s.status_skills.to_string(),
            ),
        ];
        let board = self.board();
        let (worn, great) = (self.progress.skills.0, self.progress.ult);
        for (y, text) in heads {
            let p = w.at(PICK_X, y);
            let text = self.ui.fit(&text, PICK_W * u, kk);
            self.ui.text(&text, p.x, p.y, kk, DIM);
            let line = w.rect(PICK_X, y + HEAD - 3.5, PICK_W, 0.0);
            draw_rectangle(
                line.x,
                line.y.round(),
                line.w,
                1.0,
                Color {
                    a: 0.6,
                    ..ui::BORDER
                },
            );
        }
        for i in 0..ROWS {
            let Some(what) = row(i) else { continue };
            let sel = i == at;
            let h = match what {
                Row::Level(_) => LEVEL_H,
                _ => CHARM_H,
            };
            let r = w.rect(PICK_X, row_top(i), PICK_W, h - 2.0);
            if sel {
                draw_rectangle(r.x, r.y, r.w, r.h, ui::PANEL_LIGHT);
                self.ui.edge(r, self.ui.u(1.0), ui::KEY_TEXT);
            }
            self.hits.push((r, Hit::Row(i)));
            let x = PICK_X + 6.0;
            let y = row_top(i);
            let right = PICK_X + PICK_W - 6.0;
            match what {
                Row::Level(l) => {
                    let st = level_state(&lv, l, embers, edits);
                    let pips = PIP * MAX_RANK as f32 + PIP_GAP * (MAX_RANK - 1) as f32;
                    let name = self.ui.fit(line_name(l), (right - pips - 6.0 - x) * u, k);
                    let p = w.at(x, y + 2.0);
                    self.ui
                        .text(&name, p.x, p.y, k, if sel { INK } else { DIM });
                    // 단의 점: 가진 단 = 금빛, 다음 단 = 올릴 수 있으면 민트 테, 나머지 = 흐린 테.
                    let have = lv.rank(l);
                    for n in 0..MAX_RANK {
                        let pr =
                            w.rect(right - pips + (PIP + PIP_GAP) * n as f32, y + 5.0, PIP, PIP);
                        let pr = Rect::new(pr.x.round(), pr.y.round(), pr.w.round(), pr.h.round());
                        if n < have {
                            draw_rectangle(pr.x, pr.y, pr.w, pr.h, AMBER);
                        } else if n == have && matches!(st, LevelState::Ready(_)) {
                            self.ui.edge(pr, 1.0, ui::MINT);
                        } else {
                            self.ui.edge(
                                pr,
                                1.0,
                                Color {
                                    a: 0.7,
                                    ..ui::BORDER
                                },
                            );
                        }
                    }
                    // 아랫줄: 왼쪽 한마디, 오른쪽 다음 단의 값(올릴 수 있으면 민트).
                    let (cost, ink) = match st {
                        LevelState::Full => (s.lv_full_d.to_string(), DIM),
                        LevelState::Ready(c) => (fill(s.shop_embers, &[&c.to_string()]), ui::MINT),
                        LevelState::Poor(c) | LevelState::View(c) => (
                            fill(s.shop_embers, &[&c.to_string()]),
                            Color { a: 0.6, ..DIM },
                        ),
                    };
                    let cw = self.ui.width(&cost, kk);
                    let cy = w.at(0.0, y + 17.0).y;
                    self.ui.text(&cost, w.at(right, 0.0).x - cw, cy, kk, ink);
                    let sub = self.ui.fit(line_sub(l), (right - x) * u - cw - u * 4.0, kk);
                    self.ui.text(&sub, p.x, cy, kk, Color { a: 0.75, ..DIM });
                }
                what => {
                    let entry = worn_entry(what, &board, worn, great);
                    let cells = open_cells(&board);
                    // 그림 칸: 기술 탭의 칸 그림 그대로(작게). 패시브 줄은 연 수.
                    if let Some(e) = entry {
                        let c = w.at(x + (CHARM_H - 4.0) * 0.5, y + (CHARM_H - 2.0) * 0.5);
                        super::skills::draw_node(
                            e,
                            c,
                            (CHARM_H - 5.0) * 0.5 * u,
                            u,
                            super::skills::Look::Owned,
                            None,
                        );
                    }
                    let tx = x + CHARM_H;
                    let arrow = self.ui.width("→", kk);
                    // 오른쪽 표지: 스킬 칸은 그 버튼(터치는 없다), 패시브는 "연 수/모두".
                    // 버튼 이름은 키캡에(0168 — 글과 같은 빛의 낱글자는 버튼으로 안 읽힌다).
                    let cap = |b: input::Bind| {
                        Some(input::bind_prompt(b))
                            .filter(|n| !n.is_empty())
                            .map_or(String::new(), |n| ui::key(&n))
                    };
                    let tag = match what {
                        Row::Skill(0) => cap(input::Bind::Skill1),
                        Row::Skill(_) => cap(input::Bind::Skill2),
                        Row::Passives => format!("{}/{}", cells.len(), cell_count()),
                        _ => String::new(),
                    };
                    let tag_w = if tag.is_empty() {
                        0.0
                    } else {
                        self.ui.width(&tag, kk) + u * 5.0
                    };
                    let (name, have) = match (what, entry) {
                        (Row::Passives, _) => (s.status_passives, !cells.is_empty()),
                        (_, Some(e)) => (super::skills::entry_text(e).0, true),
                        (_, None) => (s.charm_empty, false),
                    };
                    let tk = 0.85 * k;
                    let name = self
                        .ui
                        .fit(name, (right - tx) * u - arrow - tag_w - u * 4.0, tk);
                    let ty = w.at(0.0, y + (CHARM_H - 2.0 - ui::GLYPH * 0.85) * 0.5).y;
                    let ink = match (have, sel) {
                        (true, true) => INK,
                        (true, false) => DIM,
                        (false, _) => Color { a: 0.6, ..DIM },
                    };
                    self.ui.text(&name, w.at(tx, 0.0).x, ty, tk, ink);
                    // 기술 탭으로 이어진다는 표지 + 그 앞의 작은 표지.
                    let ay = w
                        .at(0.0, y + (CHARM_H - 2.0 - ui::GLYPH * tabs::K_KIND) * 0.5)
                        .y;
                    let mark = if sel { AMBER } else { Color { a: 0.6, ..DIM } };
                    let ax = w.at(right, 0.0).x - arrow;
                    self.ui.text("→", ax, ay, kk, mark);
                    if !tag.is_empty() {
                        self.ui
                            .text(&tag, ax - tag_w, ay, kk, Color { a: 0.8, ..DIM });
                    }
                }
            }
        }
    }

    /// 오른쪽: 고른 줄의 상세 — 레벨 줄이면 올리면 바뀌는 것("지금 → 다음"), 기술 줄이면 그 설명·연 패시브 — 그 아래 지금의 수치.
    fn draw_status_detail(&mut self, w: &Window, at: usize, edits: bool) {
        let s = s();
        let lv = self.progress.level;
        let now = self.progress.stats();
        let mut lines: Vec<Ln> = Vec::new();
        match row(at) {
            Some(Row::Level(l)) => {
                let st = level_state(&lv, l, self.world.embers, edits);
                let state = match st {
                    LevelState::Full => (s.lv_full_d.to_string(), DIM),
                    LevelState::View(_) => (s.sk_st_view.to_string(), INK),
                    LevelState::Poor(_) => (s.shop_poor.to_string(), DIM),
                    LevelState::Ready(c) => (fill(s.lv_ready, &[&c.to_string()]), ui::MINT),
                };
                let rank = fill(
                    s.lv_rank_of,
                    &[&lv.rank(l).to_string(), &MAX_RANK.to_string()],
                );
                self.draw_pane_head(
                    w,
                    &PaneHead {
                        name: line_name(l),
                        gold: true,
                        kind: format!("{} · {rank}", line_sub(l)),
                        state,
                        icon: false,
                    },
                );
                // 한 단 올리면 바뀌는 줄(올릴 수 있으면 민트 — 지금 할 수 있는 것).
                if st != LevelState::Full {
                    let mut then = self.progress.clone();
                    then.level = then.level.raised(l);
                    let then = then.stats();
                    let ink = if matches!(st, LevelState::Ready(_)) {
                        ui::MINT
                    } else {
                        INK
                    };
                    for (line, a, b) in crate::stats::lines(&now, Some(&then)) {
                        let Some((b, ch)) = b else { continue };
                        let mark = match ch {
                            Change::Same => continue,
                            Change::Better => "▲",
                            Change::Worse => "▼",
                        };
                        lines.push(Ln::Pair(
                            stat_name(line).to_string(),
                            format!("{} → {} {mark}", stat_value(line, a), stat_value(line, b)),
                            ink,
                        ));
                    }
                    lines.push(Ln::Gap);
                }
            }
            Some(Row::Passives) => {
                let board = self.board();
                let cells = open_cells(&board);
                self.draw_pane_head(
                    w,
                    &PaneHead {
                        name: s.status_passives,
                        gold: !cells.is_empty(),
                        kind: format!("{}/{}", cells.len(), cell_count()),
                        state: (String::new(), DIM),
                        icon: false,
                    },
                );
                if cells.is_empty() {
                    lines.extend(self.pane_wrap(w, s.status_passives_none, DIM));
                }
                for e in cells {
                    let (name, desc) = super::skills::entry_text(e);
                    lines.push(Ln::Head(name.to_string()));
                    lines.extend(self.pane_wrap(w, desc, DIM));
                }
                lines.push(Ln::Gap);
            }
            Some(what) => {
                let board = self.board();
                let entry = worn_entry(what, &board, self.progress.skills.0, self.progress.ult);
                let slot = match what {
                    Row::Skill(j) => fill(s.act_wear, &[&(j + 1).to_string()]),
                    _ => String::new(),
                };
                match entry {
                    Some(e) => {
                        let (name, desc) = super::skills::entry_text(e);
                        let rank = fill(
                            s.lv_rank_of,
                            &[&board.rank(e).to_string(), &e.max().to_string()],
                        );
                        self.draw_pane_head(
                            w,
                            &PaneHead {
                                name,
                                gold: true,
                                kind: super::skills::kind_line(&board, e),
                                state: (rank, DIM),
                                icon: false,
                            },
                        );
                        lines.extend(self.pane_wrap(w, desc, DIM));
                        // 3단의 고르기(스킬): 표지 · 이름 — 한 줄.
                        if let Entry::Skill(id) = e {
                            if board.rank(e) >= sk::PICK_RANK {
                                let (mark, pick, pick_d) =
                                    super::skills::pick_text(id, board.pick[id.index()]);
                                lines.push(Ln::Gap);
                                lines.push(Ln::Head(format!("{mark} {pick}")));
                                lines.extend(self.pane_wrap(w, pick_d, DIM));
                            }
                        }
                    }
                    None => {
                        self.draw_pane_head(
                            w,
                            &PaneHead {
                                name: s.charm_empty,
                                gold: false,
                                kind: slot,
                                state: (String::new(), DIM),
                                icon: false,
                            },
                        );
                        lines.extend(self.pane_wrap(w, s.status_skill_none, DIM));
                    }
                }
                lines.push(Ln::Gap);
            }
            None => {}
        }
        // 지금의 수치(레벨·장신구·주운 것이 다 든 값 — 보정이 있는 줄만 + 늘 보이는 줄).
        lines.push(Ln::Head(s.status_now.to_string()));
        for (line, v, _) in crate::stats::lines(&now, None) {
            lines.push(Ln::Pair(
                stat_name(line).to_string(),
                stat_value(line, v),
                INK,
            ));
        }
        self.draw_pane(w, HEAD_H, &lines);
    }

    /// 묻는 창에서 "올리기"(0104): 잔불을 내고 한 단 — 막 쉬었으니 체력·스태미나 가득(가게 기름과 같다).
    /// 연출은 작게(lore CANON — 빛 터짐·뿌리 없이): 소리 하나 + 알림. 무기를 고쳐 쥐는 동작(`regrip`)은 모델 동작이 생기면.
    pub(super) fn raise_level(&mut self, i: usize) {
        let Some(&l) = LV_LINES.get(i) else { return };
        let Some(cost) = self.progress.level.next_cost(l) else {
            return;
        };
        if self.world.embers < cost {
            self.toast = Some((s().shop_poor.to_string(), 1.6));
            self.snd(crate::sfx::Id::Deny);
            return;
        }
        self.world.embers -= cost;
        self.progress.embers = self.world.embers;
        self.progress.level = self.progress.level.raised(l);
        self.world.apply_stats(self.progress.stats(), true);
        self.snd(crate::sfx::Id::Buy);
        let rank = self.progress.level.rank(l).to_string();
        self.toast = Some((fill(s().toast_level, &[line_name(l), &rank]), TOAST_TIME));
        self.persist();
    }

    /// 묻는 창의 글(0104): "기력 3단 — 잔불 65. 올릴까요?".
    pub(super) fn ask_level_text(&self, i: usize) -> String {
        let Some(&l) = LV_LINES.get(i) else {
            return String::new();
        };
        let next = (self.progress.level.rank(l) + 1).to_string();
        let cost = self.progress.level.next_cost(l).unwrap_or(0).to_string();
        fill(s().ask_level, &[line_name(l), &next, &cost])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 0164 → 0173: 상태 탭의 줄 — 레벨 셋이 먼저(커서 0 = 생명: 화로가 열리면 바로 올릴 수 있는 자리), 그 아래 기술 묶음
    /// (스킬 두 칸 · 큰 기술 · 패시브 — 그전에는 장신구 세 칸).
    #[test]
    fn rows_are_three_levels_then_the_worn_skills_and_passives() {
        assert_eq!(ROWS, 7);
        assert_eq!(row(0), Some(Row::Level(LvLine::Life)));
        assert_eq!(row(1), Some(Row::Level(LvLine::Stamina)));
        assert_eq!(row(2), Some(Row::Level(LvLine::Might)));
        assert_eq!(row(3), Some(Row::Skill(0)));
        assert_eq!(row(4), Some(Row::Skill(1)));
        assert_eq!(row(5), Some(Row::Ult));
        assert_eq!(row(6), Some(Row::Passives));
        assert_eq!(row(ROWS), None);
        // 줄은 위에서 아래로 겹치지 않고, 묶음 사이에는 머리글 자리가 있다.
        for i in 1..ROWS {
            let h = if i <= 3 { LEVEL_H } else { CHARM_H };
            assert!(row_top(i) >= row_top(i - 1) + h, "row {i}");
        }
        assert!(row_top(3) >= row_top(2) + LEVEL_H + HEAD);
        // 몸(창 228 − 머리 22 − 여백 10 = 196)에 다 들어간다.
        assert!(row_top(ROWS - 1) + CHARM_H <= 196.0);
    }

    /// 0164: 레벨은 쉬는 동안(화로)에만 오른다 — 멈춤에서는 읽기만. 화로에서도 잔불이 모자라거나 다 올렸으면 묻지 않는다.
    #[test]
    fn levels_rise_only_at_the_hearth_with_enough_embers() {
        let lv = Level::default();
        let cost = lv.next_cost(LvLine::Life).expect("not maxed");
        assert_eq!(
            level_state(&lv, LvLine::Life, cost, true),
            LevelState::Ready(cost)
        );
        assert_eq!(
            level_state(&lv, LvLine::Life, cost - 1, true),
            LevelState::Poor(cost)
        );
        // 멈춤: 잔불이 넉넉해도 읽기만.
        assert_eq!(
            level_state(&lv, LvLine::Life, cost * 10, false),
            LevelState::View(cost)
        );
        let mut full = lv;
        for _ in 0..MAX_RANK {
            full = full.raised(LvLine::Might);
        }
        for edits in [false, true] {
            assert_eq!(
                level_state(&full, LvLine::Might, u32::MAX, edits),
                LevelState::Full
            );
        }
        // 확인: 올릴 수 있을 때만 묻는 창, 아니면 까닭.
        assert_eq!(confirm(0, &lv, cost, true), Some(Do::Ask(0)));
        assert_eq!(
            confirm(1, &lv, cost - 1, true),
            Some(Do::Deny(LevelState::Poor(cost)))
        );
        assert_eq!(
            confirm(0, &lv, cost, false),
            Some(Do::Deny(LevelState::View(cost)))
        );
        assert_eq!(
            confirm(2, &full, u32::MAX, true),
            Some(Do::Deny(LevelState::Full))
        );
    }

    /// 0164 → 0173: 탭 사이의 이음 — 기술 줄에서 확인하면 기술 탭의 그 칸으로. 멈춤에서도 된다.
    #[test]
    fn a_skill_row_leads_to_that_cell_on_the_skills_tab() {
        let lv = Level::default();
        for edits in [false, true] {
            for i in 3..ROWS {
                assert_eq!(confirm(i, &lv, 0, edits), Some(Do::Skills(row(i).unwrap())));
            }
        }
        assert_eq!(confirm(ROWS, &lv, 0, true), None);
        let at = |e: Entry| ENTRIES.iter().position(|x| *x == e).unwrap();
        let worn = [sk::Id::Whirl, sk::Id::Cut];
        // 새 판: 아무것도 안 배웠다 — 놓인 것이 없고, 가는 곳은 그 갈래의 첫 칸.
        let fresh = sk::Board::default();
        for what in [Row::Skill(0), Row::Skill(1), Row::Ult] {
            assert_eq!(worn_entry(what, &fresh, worn, sk::Ult::Upend), None);
        }
        assert!(matches!(
            ENTRIES[skills_cursor(Row::Skill(1), &fresh, worn, sk::Ult::Upend)],
            Entry::Skill(_)
        ));
        assert!(matches!(
            ENTRIES[skills_cursor(Row::Ult, &fresh, worn, sk::Ult::Upend)],
            Entry::Ult(_)
        ));
        assert!(matches!(
            ENTRIES[skills_cursor(Row::Passives, &fresh, worn, sk::Ult::Upend)],
            Entry::Cell(_)
        ));
        assert!(open_cells(&fresh).is_empty());
        // 처음의 둘과 쟁반 뒤집기를 지닌 판(옛 기록): 놓인 칸으로 간다.
        let old = sk::Board::default().with_starters().with_upend();
        assert_eq!(
            worn_entry(Row::Skill(1), &old, worn, sk::Ult::Upend),
            Some(Entry::Skill(sk::Id::Cut))
        );
        assert_eq!(
            skills_cursor(Row::Skill(0), &old, worn, sk::Ult::Upend),
            at(Entry::Skill(sk::Id::Whirl))
        );
        assert_eq!(
            skills_cursor(Row::Ult, &old, worn, sk::Ult::Upend),
            at(Entry::Ult(sk::Ult::Upend))
        );
        assert_eq!(cell_count(), 15);
    }
}
