//! 큰 창의 틀(0161 — 0160 3절 "창은 하나, 늘 같은 자리·같은 크기"): 기술 화면이 처음 썼고, 0164부터 모든 탭(상태·장비·기술·도감·
//! 가게·설정)이 같은 틀에 선다. 틀 = 두 겹 테 + 머리 줄(탭 · 지갑 · ✕) + 창 아래 버튼 안내 한 줄. 그림은 `ui::Ui::window`,
//! 여기는 자리와 누를 자리. 탭 줄·넘기기는 `tabs.rs`.
use super::menu::Hit;
use super::*;

/// 창의 크기(480×270 기준 px). 화면이 좁으면(4:3 등) 같은 비로 줄인다(`Window::k`).
pub(super) const WINDOW_W: f32 = 464.0;
pub(super) const WINDOW_H: f32 = 228.0;
/// 창 안 옆 여백.
pub(super) const WINDOW_PAD: f32 = 8.0;

/// 그린 창: 틀의 자리, 몸의 왼쪽 위(화면 px), 기준 px 하나의 화면 크기 `s`(= UI 배율 × 줄인 배 `k`).
#[derive(Clone, Copy, Debug)]
pub(super) struct Window {
    pub panel: Rect,
    pub origin: Vec2,
    pub s: f32,
    pub k: f32,
    /// 몸의 높이(기준 px).
    pub body_h: f32,
}

impl Window {
    /// 몸 안의 기준 자리 → 화면 자리.
    pub fn at(&self, x: f32, y: f32) -> Vec2 {
        self.origin + vec2(x, y) * self.s
    }

    pub fn rect(&self, x: f32, y: f32, w: f32, h: f32) -> Rect {
        let p = self.at(x, y);
        Rect::new(p.x, p.y, w * self.s, h * self.s)
    }
}

impl App {
    /// 창의 자리만(그리지 않는다 — 위에 얹는 작은 창이 아래 창의 칸 자리를 찾을 때도 쓴다).
    pub(super) fn window_geo(&self, f: &Frame) -> Window {
        let u = self.ui.u(1.0);
        let panel = self.panel_rect(f, WINDOW_W, WINDOW_H);
        let k = (panel.w / (WINDOW_W * u)).min(1.0);
        let s = u * k;
        let top = panel.y + (s * (ui::WINDOW_HEAD + 5.0)).round();
        Window {
            panel,
            origin: vec2(panel.x + s * WINDOW_PAD, top),
            s,
            k,
            body_h: (panel.y + panel.h - s * 5.0 - top) / s,
        }
    }

    /// 창을 그린다: 어둡게 → 틀·탭·지갑 → ✕(닫기 자리) · 탭 머리(누르면 그 탭 — 탭이 둘 이상일 때). `tabs`의 `active`가 지금 화면.
    pub(super) fn draw_window(
        &mut self,
        f: &Frame,
        tabs: &[(&str, bool)],
        active: usize,
        purse: &[(&str, String)],
    ) -> Window {
        self.ui.dim(0.45);
        let w = self.window_geo(f);
        let heads = self.ui.window(w.panel, w.k, tabs, active, purse);
        let x = self.ui.close(w.panel);
        self.hits.push((x, Hit::Close));
        if heads.len() > 1 {
            self.hits
                .extend(heads.into_iter().enumerate().map(|(i, r)| (r, Hit::Tab(i))));
        }
        w
    }

    /// 창 아래 버튼 안내 한 줄(지금 장치의 글자 — 터치는 화면이 곧 안내라 비어 있다): 창의 왼쪽 끝에서 시작해 `right`까지
    /// (오른쪽은 화면의 단추가 쓸 수 있다 — 기술 화면의 되돌리기). 늘 같은 자리.
    pub(super) fn window_hints(&self, f: &Frame, w: &Window, right: f32, pairs: &[(Act, &str)]) {
        let named: Vec<(String, &str)> =
            pairs.iter().map(|(a, v)| (input::prompt(*a), *v)).collect();
        let refs: Vec<(&str, &str)> = named.iter().map(|(k, v)| (k.as_str(), *v)).collect();
        let x = w.panel.x + w.s * 2.0;
        self.ui
            .hints_left(&refs, x, f.oy + f.h - self.ui.px(1.0) * 2.2, right - x);
    }
}
