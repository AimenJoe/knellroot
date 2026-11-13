//! 목록 화면 몰이꾼(0066): 화면은 줄 표(`Menu`) 하나를 내고, 입력과 그림이 같은 표를 쓴다 — 줄을 더하면 한 곳.
//! 입력은 `Out`(고름 · 좌우 · 뒤로)으로 바꿔 화면의 `act`에 넘긴다. 누를 자리는 그림이 채우고 입력이 읽는
//! `hits: Vec<(Rect, Hit)>`(nail `app.rs`의 꼴) — 먼저 넣은 칸이 이긴다(▲▼ · ◀▶ · ✕ · 줄 · 창 밖 순).
use super::*;

/// 창 틀(0066, 480×270 기준 px): 제목 줄 높이, 아래 여백, 아래 설명 두 줄 자리, 제목 아래 한 줄.
pub(super) const PANEL_HEAD: f32 = 30.0;
pub(super) const PANEL_FOOT: f32 = 8.0;
/// 아래 설명 한 줄의 높이·설명 칸 위 틈. 줄 수는 화면이 정한다(`Menu::note_lines` — 보통 둘, 기술 판은 셋).
pub(super) const NOTE_LINE: f32 = ui::GLYPH * 1.45;
pub(super) const NOTE_GAP: f32 = 6.0;
pub(super) const PANEL_SUB: f32 = ui::GLYPH * 1.6;

/// 목록 화면 하나(그림과 입력이 같이 쓴다).
pub(super) struct Menu {
    pub title: String,
    pub rows: Vec<ui::Row>,
    /// 창 폭(480×270 기준 px).
    pub width: f32,
    /// ✕·창 밖·뒤로로 닫힌다. 골라야 끝나는 창(핵 고르기·결과)은 거짓.
    pub closable: bool,
    /// 제목 오른쪽(가진 잔불).
    pub purse: Option<String>,
    /// 제목 아래 한 줄.
    pub sub: Option<String>,
    /// 아래 설명 자리를 두나 · 지금 고른 줄의 설명 · 그 줄 수(넘치는 글은 잘린다 — 개발판 F4가 알린다).
    pub note_room: bool,
    pub note: Option<String>,
    pub note_lines: usize,
    /// 설명 자리에 잠깐 대신 뜨는 알림(밝게) — 창이 화면을 다 써서 위쪽 알림 자리가 없는 화면(기술 판)이 쓴다.
    pub alert: Option<String>,
}

impl Menu {
    pub fn new(title: &str, width: f32, rows: Vec<ui::Row>) -> Menu {
        Menu {
            title: title.to_string(),
            rows,
            width,
            closable: true,
            purse: None,
            sub: None,
            note_room: false,
            note: None,
            note_lines: 2,
            alert: None,
        }
    }
}

/// 목록 화면의 입력 결과.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Out {
    /// 확인·탭: 이 줄.
    Pick(usize),
    /// 좌우·◀▶: 이 줄의 값을 ±1.
    Step(usize, i32),
}

/// 누를 자리(0066, nail `Hit`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Hit {
    Row(usize),
    Step(usize, i32),
    /// ▲(-1)·▼(+1).
    Scroll(i32),
    /// ✕ · 창 밖.
    Close,
    /// 조작 표의 칸(줄, 칸).
    Cell(usize, usize),
    /// 글 칸(설명) — 휠·끌기가 여기서 일어나면 목록 대신 이 칸을 굴린다(0068).
    Pane,
    /// 큰 창의 탭 머리(0164) — 누르면 그 탭으로(`flow::tab_pick`).
    Tab(usize),
    /// 굴리는 판(기술 화면의 칸 격자, 0163)의 보이는 자리 — 휠·끌기가 여기서 일어나면 판을 굴린다. 칸들보다 뒤에 넣는다.
    Field,
}

/// `ui` 목록의 자리 → 누를 자리, 먼저 이기는 순서로: ▲▼ → ◀▶ → 줄(◀▶·▲▼는 줄 칸 안에 겹쳐 있다).
pub(super) fn hits_of(h: &ui::MenuHits) -> Vec<(Rect, Hit)> {
    let mut v = Vec::new();
    for (r, d) in h.arrows.iter().zip([-1, 1]) {
        if let Some(r) = r {
            v.push((*r, Hit::Scroll(d)));
        }
    }
    for (i, s) in h.steps.iter().enumerate() {
        if let Some([l, r]) = s {
            v.push((*l, Hit::Step(i, -1)));
            v.push((*r, Hit::Step(i, 1)));
        }
    }
    for (i, r) in h.rows.iter().enumerate() {
        if r.w > 0.0 {
            v.push((*r, Hit::Row(i)));
        }
    }
    v
}

/// 탭 자리 → 누를 것(먼저 넣은 칸이 이긴다).
pub(super) fn first_hit(hits: &[(Rect, Hit)], p: Vec2) -> Option<Hit> {
    hits.iter().find(|(r, _)| r.contains(p)).map(|(_, h)| *h)
}

impl App {
    /// 탭 자리 → 누를 것(지난 프레임 그림에서, 먼저 넣은 칸이 이긴다).
    pub(super) fn hit_at(&self, p: Vec2) -> Option<Hit> {
        first_hit(&self.hits, p)
    }

    /// `ui` 목록이 돌려준 자리를 누를 자리 목록에(`hits_of`).
    pub(super) fn push_hits(&mut self, h: &ui::MenuHits) {
        self.hits.extend(hits_of(h));
    }

    /// 창 밖 네 쪽을 닫기로(nail `close_button` — 창 안 빈 곳은 닫지 않는다). 줄을 넣은 뒤에 부른다.
    pub(super) fn push_outside(&mut self, panel: Rect) {
        let big = 1.0e5;
        for r in [
            Rect::new(-big, -big, big * 2.0, big + panel.y),
            Rect::new(-big, panel.y + panel.h, big * 2.0, big),
            Rect::new(-big, panel.y, big + panel.x, panel.h),
            Rect::new(panel.x + panel.w, panel.y, big, panel.h),
        ] {
            self.hits.push((r, Hit::Close));
        }
    }

    /// 메뉴 한 틱(제 그림을 갖는 화면 — 타이틀·묻는 창): 커서·굴리기·탭·확인 → 고른 줄.
    pub(super) fn menu_pick(&mut self, a: &Actions, n: usize) -> Option<usize> {
        ui::menu_input(&mut self.cursor, n, a.menu_up, a.menu_down);
        let tap = a.tap.and_then(|p| self.hit_at(p));
        let scroll = match tap {
            Some(Hit::Scroll(d)) => d,
            _ => 0,
        };
        self.view
            .scroll((a.drag, a.wheel + scroll), &mut self.cursor, n);
        if let Some(Hit::Row(i)) = tap {
            self.cursor = i;
            return Some(i);
        }
        a.confirm.then_some(self.cursor.min(n.saturating_sub(1)))
    }

    /// 목록 화면 한 틱: 뒤로(닫을 수 있는 창만) · 좌우(값 줄만) · 고름. 뒤로는 여기서 한다(`flow::back`).
    pub(super) fn menu_input(&mut self, a: &Actions, m: &Menu) -> Option<Out> {
        let tap = a.tap.and_then(|p| self.hit_at(p));
        if m.closable && (a.back || a.pause || tap == Some(Hit::Close)) {
            self.go(flow::back(self.scene()));
            return None;
        }
        if let Some(Hit::Step(i, d)) = tap {
            self.cursor = i;
            return Some(Out::Step(i, d));
        }
        let step = a.menu_right as i32 - a.menu_left as i32;
        let on_step = m
            .rows
            .get(self.cursor)
            .is_some_and(|r| r.kind == ui::RowKind::Step);
        if step != 0 && on_step {
            return Some(Out::Step(self.cursor, step));
        }
        self.menu_pick(a, m.rows.len()).map(Out::Pick)
    }

    /// 가운데 창 자리(0066): 480×270 기준 폭·높이를 화면 안(양옆 8)·안내 줄 위로 줄여 가운데에.
    pub(super) fn panel_rect(&self, f: &Frame, w: f32, h: f32) -> Rect {
        let u = self.ui.u(1.0);
        let top = f.oy + u * 6.0;
        let bottom = self.menu_bottom(f);
        let w = (w * u).min(f.w - u * 16.0);
        let h = (h * u).min(bottom - top);
        Rect::new(f.ox + (f.w - w) * 0.5, top + (bottom - top - h) * 0.5, w, h)
    }

    /// 창 안 아래쪽 설명(`lines`줄까지, nail 가게의 설명 줄). 넘쳐 잘린 글은 개발판 넘침 검사(F4)가 알린다.
    pub(super) fn panel_note(&self, panel: Rect, text: &str, lines: usize, ink: Color) {
        let u = self.ui.u(1.0);
        let px = self.ui.px(1.0);
        let y = panel.y + panel.h - u * PANEL_FOOT - px * 1.45 * lines as f32;
        let wrapped = ui::wrap(&self.ui, text, panel.w - u * 24.0, 1.0);
        #[cfg(feature = "dev")]
        if wrapped.len() > lines {
            ui::check::cut(text, panel.w - u * 24.0);
        }
        for (i, line) in wrapped.iter().take(lines).enumerate() {
            self.ui
                .text(line, panel.x + u * 12.0, y + px * 1.45 * i as f32, 1.0, ink);
        }
    }

    /// 목록 화면을 그린다: 어둡게 → 창 틀(제목·✕) → 지갑·제목 아래 한 줄 → 목록 → 설명 → 아래 안내. 누를 자리를 채운다.
    pub(super) fn draw_menu(&mut self, f: &Frame, m: &Menu) {
        self.ui.dim(0.45);
        let u = self.ui.u(1.0);
        let sub = if m.sub.is_some() { PANEL_SUB } else { 0.0 };
        let note = if m.note_room {
            NOTE_LINE * m.note_lines as f32 + NOTE_GAP
        } else {
            0.0
        };
        let h = PANEL_HEAD + m.rows.len() as f32 * ui::GLYPH * 1.75 + sub + note + PANEL_FOOT;
        let panel = self.panel_rect(f, m.width, h);
        let top = self.ui.panel(panel, Some(&m.title));
        let close = m.closable.then(|| self.ui.close(panel));
        if let Some(t) = &m.sub {
            let t = self.ui.fit(t, panel.w - u * 24.0, 1.0);
            self.ui.text(&t, panel.x + u * 12.0, top, 1.0, DIM);
        }
        let list_top = top + u * sub;
        let bottom = panel.y + panel.h - u * (PANEL_FOOT + note);
        let area = Rect::new(
            panel.x + u * 6.0,
            list_top,
            panel.w - u * 12.0,
            bottom - list_top,
        );
        let mut hits = self.ui.list(&m.rows, self.cursor, &mut self.view, area);
        hits.arrows = self.ui.title_arrows(&self.view, m.rows.len(), panel);
        // 지갑은 제목 줄 오른쪽 — 목록이 넘쳐 ▲▼가 뜨면 그 왼쪽으로 비킨다(겹치지 않게).
        if let Some(p) = &m.purse {
            let (up, down) = self.view.more(m.rows.len());
            let arrows = if up || down { 30.0 } else { 0.0 };
            let pw = self.ui.width(p, 1.0);
            self.ui.text(
                p,
                panel.x + panel.w - u * (30.0 + arrows) - pw,
                panel.y + u * 6.0,
                1.0,
                INK,
            );
        }
        if let Some(x) = close {
            self.hits.push((x, Hit::Close));
        }
        self.push_hits(&hits);
        if m.closable {
            self.push_outside(panel);
        }
        match (&m.alert, &m.note) {
            (Some(t), _) => self.panel_note(panel, t, m.note_lines, AMBER),
            (None, Some(t)) => self.panel_note(panel, t, m.note_lines, DIM),
            (None, None) => {}
        }
        self.menu_hint(
            f.ox + f.w * 0.5,
            f.oy + f.h - self.ui.px(1.0) * 2.2,
            f.w * 0.94,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrows_and_steps_win_over_the_row_they_sit_on() {
        // 줄 하나(0..100 × 0..20), 그 안의 ◀(60..70)·▶(90..100), 위 ▲(40..60 × -10..0).
        let h = ui::MenuHits {
            rows: vec![Rect::new(0.0, 0.0, 100.0, 20.0), Rect::default()],
            steps: vec![
                Some([
                    Rect::new(60.0, 0.0, 10.0, 20.0),
                    Rect::new(90.0, 0.0, 10.0, 20.0),
                ]),
                None,
            ],
            arrows: [Some(Rect::new(40.0, -10.0, 20.0, 10.0)), None],
        };
        let hits = hits_of(&h);
        assert_eq!(first_hit(&hits, vec2(10.0, 10.0)), Some(Hit::Row(0)));
        assert_eq!(first_hit(&hits, vec2(65.0, 10.0)), Some(Hit::Step(0, -1)));
        assert_eq!(first_hit(&hits, vec2(95.0, 10.0)), Some(Hit::Step(0, 1)));
        assert_eq!(first_hit(&hits, vec2(50.0, -5.0)), Some(Hit::Scroll(-1)));
        // 안 보이는 줄(빈 칸)은 누를 수 없다.
        assert_eq!(hits.iter().filter(|(_, h)| *h == Hit::Row(1)).count(), 0);
    }
}
