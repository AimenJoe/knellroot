//! 큰 창의 탭(0164 — 0160 3절 "창은 하나, 탭 여섯"): 멈춤과 화로가 같은 창을 연다. 여기는 탭 줄(이름·표시·지갑)과 넘기기,
//! 그리고 모든 탭이 같이 쓰는 몸의 문법 — **왼쪽 = 고르는 것(칸·줄) · 오른쪽 = 고른 것의 상세 · 아래 = 같은 자리의 버튼 안내**.
//! 탭마다의 몸은 제 파일(`status`·`equip`·`skills`·`codex`·`shop`·`settings`), 탭 목록과 넘기기의 규칙은 `flow`(순수, 시험).
//! 넘기면 스택의 맨 위 칸을 바꾼다(`Go::Swap` — 쌓지 않는다): 창의 자리·크기·탭 머리·지갑은 그대로고 몸만 바뀐다.
use super::frame::Window;
use super::menu::Hit;
use super::*;
use flow::{Host, Tab};

/// 몸의 두 칸(480×270 기준 px, 몸의 왼쪽 위가 원점 — 기술 화면이 먼저 쓴 자리 그대로): 왼쪽 칸의 폭, 오른쪽 칸의 자리와 폭.
/// 그 사이(`LEFT_W + 6`)에 세로 선.
pub(super) const LEFT_W: f32 = 270.0;
pub(super) const RIGHT_X: f32 = 282.0;
pub(super) const RIGHT_W: f32 = 166.0;
/// 왼쪽 칸의 줄 목록이 오른쪽 끝에서 들이는 몫(그 틈에 굴림 ▲▼).
const LIST_GUTTER: f32 = 6.0;
/// 오른쪽 칸의 머리: 그림 칸의 한 변, 머리의 높이(이름 · 갈래 한 줄 · 상태 한 줄 아래 몸이 시작한다).
pub(super) const HEAD_ICON: f32 = 30.0;
pub(super) const HEAD_H: f32 = 37.0;
/// 글의 배(창의 배 `Window::k`에 곱한다): 이름 · 갈래와 상태 줄 · 몸.
pub(super) const K_NAME: f32 = 1.0;
pub(super) const K_KIND: f32 = 0.66;
pub(super) const K_BODY: f32 = 0.72;
/// 몸 글의 줄 높이(기준 px).
pub(super) const LH_BODY: f32 = 10.4;

/// 오른쪽 칸의 몸 한 줄.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Ln {
    /// 글 한 줄(이미 폭에 맞춰 나눈 것 — `App::pane_wrap`).
    Text(String, Color),
    /// 이름 왼쪽 · 값 오른쪽(값의 색).
    Pair(String, String, Color),
    /// 묶음의 작은 머리글(금빛).
    Head(String),
    /// 반 줄 띄기.
    Gap,
}

impl Ln {
    fn height(&self) -> f32 {
        match self {
            Ln::Gap => LH_BODY * 0.5,
            Ln::Head(_) => LH_BODY * 1.15,
            _ => LH_BODY,
        }
    }
}

/// 오른쪽 칸의 머리에 쓸 것: 이름(가진 것·낀 것이면 금빛) · 갈래 한 줄 · 상태 한 줄(색) · 그림 칸을 비울까.
pub(super) struct PaneHead<'a> {
    pub name: &'a str,
    pub gold: bool,
    pub kind: String,
    pub state: (String, Color),
    pub icon: bool,
}

/// 탭 머리에 "지금 할 수 있는 것" 표시(민트 점)를 붙일까 — 쉬는 동안(화로)에만: 올릴 단이 있는 상태, 살 점수가 있는 기술,
/// 살 것이 있는 가게. 멈춤에서는 읽기·끼우기뿐이라 표시가 없다. 여정(0183)만은 멈춤에서도 — 받기는 어디서나 된다.
pub(super) fn tab_marked(
    host: Host,
    tab: Tab,
    level: bool,
    skills: bool,
    shop: bool,
    journey: bool,
) -> bool {
    match tab {
        Tab::Journey => journey,
        Tab::Status => host.edits() && level,
        Tab::Skills => host.edits() && skills,
        Tab::Shop => host.edits() && shop,
        Tab::Equip | Tab::Codex | Tab::Settings => false,
    }
}

impl App {
    fn tab_name(tab: Tab) -> &'static str {
        let s = s();
        match tab {
            Tab::Status => s.equip_status,
            Tab::Equip => s.menu_equip,
            Tab::Skills => s.menu_skills,
            Tab::Codex => s.menu_codex,
            Tab::Journey => s.menu_journey,
            Tab::Shop => s.hearth_shop,
            Tab::Settings => s.menu_settings,
        }
    }

    /// 창을 연다·탭을 넘긴다: 장면을 바꾸고, 그 탭이 처음 설 자리를 정한다(기술 = 화로면 살 수 있는 첫 칸·아니면 배운 첫 스킬).
    /// 멈춤에서 본 탭은 판에 적어 둔다 — 다음에 멈추면 거기서 열린다(`flow::pause`).
    pub(super) fn show(&mut self, go: Go) {
        self.go(go);
        let scene = self.scene();
        if let Scene::Skills { host } = scene {
            self.cursor = self
                .board()
                .first_focus(self.progress.skill_points(), self.can_edit(host));
        }
        if let Some(tab) = flow::remembered(scene) {
            self.pause_tab = tab;
        }
    }

    /// 이 창에서 올리기·사기·되돌리기가 켜지나: 화로에서만 — 개발 창에서 올린 판(메모리의 덮어쓰기)은 사지 않는다.
    pub(super) fn can_edit(&self, host: Host) -> bool {
        host.edits() && !self.dev_board_on()
    }

    /// 탭 넘기기 한 틱: 패드 LB·RB / 키보드 Q·E(0175) = 옆 탭, 탭 머리 누르기 = 그 탭. 넘겼으면 참(그 틱은 여기서 끝난다).
    pub(super) fn tab_input(&mut self, a: &Actions) -> bool {
        let scene = self.scene();
        let go = match a.tap.and_then(|p| self.hit_at(p)) {
            Some(Hit::Tab(i)) => flow::tab_pick(scene, i),
            _ => match a.tab_next as i32 - a.tab_prev as i32 {
                0 => Go::Stay,
                dir => flow::tab_step(scene, dir),
            },
        };
        if go == Go::Stay {
            return false;
        }
        self.show(go);
        self.snd(crate::sfx::Id::UiMove);
        true
    }

    /// 지갑(창 머리 오른쪽): 판 위의 창은 어느 탭에서나 기술 점수 · 잔불 — 탭을 넘겨도 머리 줄이 움직이지 않는다. 타이틀은 없다.
    fn purse(&self, host: Host) -> Vec<(&'static str, String)> {
        if host == Host::Title {
            return Vec::new();
        }
        let s = s();
        let left = self.board().left(self.progress.skill_points());
        vec![
            (s.sk_purse_points, left.to_string()),
            (s.sk_purse_embers, self.world.embers.to_string()),
        ]
    }

    /// 탭의 창을 그린다: 틀 · 탭 줄(지금 탭 = 금빛, 할 일이 있는 탭 = 민트 점) · 지갑 · ✕ · 두 칸 사이 세로 선. 몸은 부른 쪽이.
    pub(super) fn draw_tab_window(&mut self, f: &Frame, scene: Scene) -> Window {
        let (host, _) = flow::tab_of(scene).unwrap_or((Host::Pause, Tab::Status));
        let level = self.progress.level.can_raise(self.world.embers);
        let skills = !self.dev_board_on() && self.board().can_buy(self.progress.skill_points());
        let shop = crate::growth::can_buy(
            self.world.embers,
            &self.progress.owned,
            &self.progress.growth,
        ) || self.can_forge();
        let journey = crate::journey::any_ready(&self.progress);
        let tabs: Vec<(&str, bool)> = flow::tabs(host)
            .iter()
            .map(|&t| {
                let on = tab_marked(host, t, level, skills, shop, journey);
                (Self::tab_name(t), on)
            })
            .collect();
        let purse = self.purse(host);
        let w = self.draw_window(f, &tabs, flow::tab_index(scene).unwrap_or(0), &purse);
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
        w
    }

    /// 창 아래 버튼 안내(늘 같은 자리): 탭의 일(`pairs`) · 탭 넘기기(탭이 둘 이상이면) · 닫기. 창 밖 닫기 자리도 여기서 넣는다 —
    /// 창 밖에 단추를 두는 탭(기술의 되돌리기)은 그 단추를 먼저 넣고 부른다. 묻는 창이 위에 있으면 안내는 그 창이 그린다.
    pub(super) fn tab_foot(
        &mut self,
        f: &Frame,
        w: &Window,
        scene: Scene,
        right: f32,
        pairs: &[(Act, &str)],
    ) {
        self.push_outside(w.panel);
        if matches!(self.scene(), Scene::Ask(_)) {
            return;
        }
        let s = s();
        let mut all = pairs.to_vec();
        // 탭 넘기기는 탭이 맨 위일 때만 듣는다(작은 목록이 얹혀 있으면 안내하지 않는다) — 탭이 하나면(타이틀의 설정) 없다.
        if self.scene() == scene
            && flow::tab_of(scene).is_some_and(|(h, _)| flow::tabs(h).len() > 1)
        {
            all.push((Act::Tabs, s.verb_tab));
        }
        all.push((Act::Back, s.verb_close));
        self.window_hints(f, w, right, &all);
    }

    /// 멈춤의 큰 창(0173): 올리는 일이 있는 탭(상태·기술)의 아래 줄 오른쪽 끝에 "보기만 — 올리기는 쉼터 화로에서".
    /// 화로에서 되돌리기 단추가 서는 자리다. 안내 줄이 쓸 수 있는 오른쪽 끝을 돌려준다.
    pub(super) fn draw_view_note(&mut self, f: &Frame, w: &Window) -> f32 {
        let k = K_KIND * w.k;
        let room = w.panel.w * 0.5;
        let text = self.ui.fit(s().view_note, room, k);
        let x = (w.panel.x + w.panel.w - self.ui.width(&text, k) - w.s * 2.0).round();
        let y = f.oy + f.h - self.ui.px(1.0) * 2.2 + (self.ui.px(1.0) - self.ui.px(k)) * 0.5;
        self.ui.text(&text, x, y, k, AMBER);
        x - w.s * 8.0
    }

    /// 왼쪽 칸의 줄 목록(줄 표 그대로 — 창의 배로): `top`(기준 px)부터 몸의 끝까지. 넘치면 굴리고 ▲▼는 줄 끝과 세로 선 사이에.
    /// 누를 자리를 채운다.
    pub(super) fn draw_left_list(&mut self, w: &Window, rows: &[ui::Row], top: f32) {
        // 줄은 칸의 오른쪽 끝에서 조금 들여 끝난다 — 그 틈(줄 끝 ~ 세로 선)에 ▲▼가 선다.
        let area = w.rect(0.0, top, LEFT_W - LIST_GUTTER, w.body_h - top);
        let cursor = self.cursor.min(rows.len().saturating_sub(1));
        let mut hits = self.ui.list_k(rows, cursor, &mut self.view, area, w.k);
        let (up, down) = self.view.more(rows.len());
        let tk = 0.7 * w.k;
        let ax = w.at(LEFT_W, 0.0).x - self.ui.width("▼", tk) * 0.5;
        for (k, (on, glyph, y)) in [
            (up, "▲", w.at(0.0, top).y),
            (down, "▼", w.at(0.0, w.body_h).y - self.ui.px(tk)),
        ]
        .into_iter()
        .enumerate()
        {
            if on {
                self.ui.text(glyph, ax, y, tk, DIM);
                let grow = self.ui.px(tk);
                hits.arrows[k] = Some(Rect::new(
                    ax - grow,
                    y - grow,
                    self.ui.width(glyph, tk) + grow * 2.0,
                    grow * 3.0,
                ));
            }
        }
        self.push_hits(&hits);
    }

    /// 오른쪽 칸의 머리: (그림 칸) 이름 · 갈래 한 줄 · 상태 한 줄. 그림 칸의 자리를 돌려준다(부른 쪽이 그 안에 그린다).
    /// 잠깐 뜨는 알림(산 것·올린 것·안 되는 까닭)이 있으면 상태 줄 자리에 대신 뜬다 — 창이 화면을 다 써서 알림 자리가 따로 없다.
    pub(super) fn draw_pane_head(&mut self, w: &Window, h: &PaneHead) -> Rect {
        let (k, u) = (w.k, w.s);
        let icon = w.rect(RIGHT_X, 1.0, HEAD_ICON, HEAD_ICON);
        let x = if h.icon {
            RIGHT_X + HEAD_ICON + 6.0
        } else {
            RIGHT_X
        };
        let room = (RIGHT_X + RIGHT_W - x) * u;
        let p = w.at(x, 0.0);
        let name = self.ui.fit(h.name, room, K_NAME * k);
        let ink = if h.gold { AMBER } else { INK };
        self.ui.text(&name, p.x, p.y, K_NAME * k, ink);
        let kind = self.ui.fit(&h.kind, room, K_KIND * k);
        self.ui.text(&kind, p.x, w.at(0.0, 15.0).y, K_KIND * k, DIM);
        let (state, ink) = match &self.toast {
            Some((t, _)) => (t.clone(), AMBER),
            None => h.state.clone(),
        };
        let state = self.ui.fit(&state, room, K_KIND * k);
        self.ui
            .text(&state, p.x, w.at(0.0, 25.0).y, K_KIND * k, ink);
        icon
    }

    /// 머리의 그림 칸에 아이템을: 칸이 넉넉하면 도는 그림(`draw_item`), 좁으면(작은 화면) 찍어 둔 그림(`draw_icon`) — 정수배로만.
    pub(super) fn draw_pane_item(&mut self, item: crate::item_view::Item, r: Rect) {
        let (native, spin) = if r.w >= crate::render::ITEM_PX as f32 {
            (crate::render::ITEM_PX, true)
        } else {
            (crate::render::ICON_PX, false)
        };
        let side = native as f32 * (r.w / native as f32).floor().max(1.0);
        let dest = Rect::new(
            (r.x + (r.w - side) * 0.5).round(),
            (r.y + (r.h - side) * 0.5).round(),
            side,
            side,
        );
        if spin {
            let t = get_time() as f32;
            self.renderer.draw_item(item, dest, 0.7 * (t * 0.8).sin());
        } else {
            self.renderer.draw_icon(item, dest);
        }
    }

    /// 글을 오른쪽 칸의 폭에 맞춰 줄로 나눈다.
    pub(super) fn pane_wrap(&self, w: &Window, text: &str, ink: Color) -> Vec<Ln> {
        ui::wrap(&self.ui, text, RIGHT_W * w.s, K_BODY * w.k)
            .into_iter()
            .map(|l| Ln::Text(l, ink))
            .collect()
    }

    /// 오른쪽 칸의 몸: `top`(기준 px)부터 몸의 끝까지 줄들을. 넘치면 **읽는 사람이 굴린다**(0068 — 오른쪽 스틱, 칸 위의 휠·끌기,
    /// 값은 `App::scroll` 줄 수) + ▲▼. 굴리는 자리(`Hit::Pane`)를 넣는다.
    pub(super) fn draw_pane(&mut self, w: &Window, top: f32, lines: &[Ln]) {
        let (k, u) = (w.k, w.s);
        let room = w.body_h - top;
        // 보이는 첫 줄: 뒤의 줄들이 다 들어가는 가장 앞자리까지만 내려간다.
        let heights: Vec<f32> = lines.iter().map(Ln::height).collect();
        let mut max_first = 0;
        while max_first + 1 < lines.len() && heights[max_first..].iter().sum::<f32>() > room + 0.5 {
            max_first += 1;
        }
        self.scroll = self.scroll.clamp(0.0, max_first as f32);
        let first = self.scroll.round() as usize;
        let (x0, x1) = (w.at(RIGHT_X, 0.0).x, w.at(RIGHT_X + RIGHT_W, 0.0).x);
        // 넘칠 때만 오른쪽 끝에 ▲▼ 자리를 비운다.
        let arrow_w = if max_first > 0 {
            self.ui.width("▼", 0.7 * k) + u * 2.0
        } else {
            0.0
        };
        let mut y = top;
        let mut shown = first;
        for l in lines.iter().skip(first) {
            let h = l.height();
            if y + h > w.body_h + 0.5 {
                break;
            }
            let py = w.at(0.0, y).y;
            match l {
                Ln::Text(t, ink) => self.ui.text(t, x0, py, K_BODY * k, *ink),
                Ln::Head(t) => {
                    let t = self.ui.fit(t, x1 - x0 - arrow_w, K_BODY * k);
                    self.ui.text(&t, x0, py + u, K_BODY * k, AMBER);
                }
                Ln::Pair(name, value, ink) => {
                    let vw = self.ui.width(value, K_BODY * k);
                    self.ui.text(value, x1 - arrow_w - vw, py, K_BODY * k, *ink);
                    let name = self
                        .ui
                        .fit(name, x1 - x0 - arrow_w - vw - u * 4.0, K_BODY * k);
                    self.ui.text(&name, x0, py, K_BODY * k, DIM);
                }
                Ln::Gap => {}
            }
            y += h;
            shown += 1;
        }
        if max_first > 0 {
            let ax = x1 - self.ui.width("▼", 0.7 * k);
            if first > 0 {
                self.ui.text("▲", ax, w.at(0.0, top).y, 0.7 * k, DIM);
            }
            if shown < lines.len() {
                let ay = w.at(0.0, w.body_h).y - self.ui.px(0.7 * k);
                self.ui.text("▼", ax, ay, 0.7 * k, DIM);
            }
        }
        self.hits
            .push((w.rect(RIGHT_X, top, RIGHT_W, room), Hit::Pane));
    }

    /// 오른쪽 칸 굴리기 한 틱(0068): 오른쪽 스틱은 늘, 휠·끌기는 그 칸 위에서 일어났을 때만(그때는 목록에 넘기지 않는다).
    /// 커서가 옮겨 가면 부른 쪽이 `scroll`을 0으로.
    pub(super) fn pane_scroll_lines(&mut self, a: &mut Actions) {
        let over = a.point.and_then(|p| self.hit_at(p)) == Some(Hit::Pane);
        let (wheel, drag) = if over {
            let wd = (a.wheel, a.drag);
            (a.wheel, a.drag) = (0, 0.0);
            wd
        } else {
            (0, 0.0)
        };
        let line = self.ui.u(LH_BODY).max(1.0);
        self.scroll = super::equip::pane_step(
            self.scroll,
            a.aim.y,
            TICK_DT,
            wheel,
            drag / line,
            usize::MAX >> 1,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 0164: 탭 머리의 민트 점은 "지금 할 수 있는 것" — 쉬는 동안에만, 올릴 단·살 점수·살 물건이 있는 탭에만.
    #[test]
    fn tab_marks_show_only_what_can_be_done_now_and_only_at_the_hearth() {
        for &t in flow::tabs(Host::Hearth) {
            assert!(
                !tab_marked(Host::Hearth, t, false, false, false, false),
                "{t:?}"
            );
            assert!(
                !tab_marked(Host::Pause, t, true, true, true, false),
                "{t:?}: nothing to buy while paused"
            );
        }
        assert!(tab_marked(
            Host::Hearth,
            Tab::Status,
            true,
            false,
            false,
            false
        ));
        assert!(tab_marked(
            Host::Hearth,
            Tab::Skills,
            false,
            true,
            false,
            false
        ));
        assert!(tab_marked(
            Host::Hearth,
            Tab::Shop,
            false,
            false,
            true,
            false
        ));
        // 저마다 제 것만 본다.
        assert!(!tab_marked(
            Host::Hearth,
            Tab::Status,
            false,
            true,
            true,
            false
        ));
        assert!(!tab_marked(
            Host::Hearth,
            Tab::Skills,
            true,
            false,
            true,
            false
        ));
        assert!(!tab_marked(
            Host::Hearth,
            Tab::Shop,
            true,
            true,
            false,
            false
        ));
        for t in [Tab::Equip, Tab::Codex, Tab::Settings] {
            assert!(
                !tab_marked(Host::Hearth, t, true, true, true, false),
                "{t:?}"
            );
        }
        // 여정(0183): 받을 것이 있으면 멈춤에서도 점이 붙는다(받기는 어디서나).
        assert!(tab_marked(
            Host::Pause,
            Tab::Journey,
            false,
            false,
            false,
            true
        ));
        assert!(!tab_marked(
            Host::Hearth,
            Tab::Journey,
            true,
            true,
            true,
            false
        ));
    }

    #[test]
    fn the_two_panes_and_the_rule_between_them_fit_the_window_body() {
        let body = super::super::frame::WINDOW_W - super::super::frame::WINDOW_PAD * 2.0;
        assert!(LEFT_W + 6.0 < RIGHT_X);
        assert_eq!(RIGHT_X + RIGHT_W, body);
    }
}
