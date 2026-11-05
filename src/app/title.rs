//! 타이틀(0019·0066): 키 아트 위 목록 — 창 틀 없이. [이어하기] · 시작하기 · 설정 · DEV — 이어하기·시작하기는 슬롯 창으로.
use super::*;

impl App {
    /// 타이틀(0066): [이어하기] · 시작하기 · 설정 · DEV.
    pub(super) fn tick_title(&mut self, a: Actions) {
        let items = flow::title_items(self.slots.iter().any(Option::is_some));
        if let Some(i) = self.menu_pick(&a, items.len()) {
            match items[i] {
                // 둘 다 슬롯 창으로(0066, KB): 고른 슬롯에서 판이 시작된다.
                TitleItem::Continue => self.open_slots(true),
                TitleItem::Start => self.open_slots(false),
                TitleItem::Settings => self.go(Go::Open(Scene::Settings)),
                #[cfg(feature = "dev")]
                TitleItem::Dev => self.go(Go::Open(Scene::Dev)),
            }
        }
    }

    pub(super) fn draw_title(&mut self, f: &Frame) {
        let s = s();
        self.ui.dim(0.45);
        let cx = f.ox + f.w * 0.5;
        let y = f.oy + f.h * 0.13;
        self.ui.text_center(s.game_title, cx, y, 3.0, AMBER);
        self.ui
            .text_center(s.title_subtitle, cx, y + self.ui.px(3.0) * 1.2, 1.0, DIM);
        let has = self.slots.iter().any(Option::is_some);
        let items: Vec<String> = flow::title_items(has)
            .iter()
            .map(|i| match i {
                TitleItem::Continue => s.menu_continue.to_string(),
                TitleItem::Start => s.menu_start.to_string(),
                TitleItem::Settings => s.menu_settings.to_string(),
                // 개발 전용(영어 그대로, 0013).
                #[cfg(feature = "dev")]
                TitleItem::Dev => "DEV".to_string(),
            })
            .collect();
        let bottom = self.menu_bottom(f);
        let hits = self.ui.menu(
            &items,
            self.cursor,
            &mut self.view,
            cx,
            f.oy + f.h * TITLE_MENU_Y,
            bottom,
        );
        self.push_hits(&hits);
        self.menu_hint(cx, f.oy + f.h - self.ui.px(1.0) * 2.2, f.w * 0.94);
        // 버전 글자(오른쪽 위) — 받은 판이 무엇인지 사진 한 장으로 알게. 개발판은 표시를 붙인다.
        let ver = if cfg!(feature = "dev") {
            concat!("v", env!("CARGO_PKG_VERSION"), " dev")
        } else {
            concat!("v", env!("CARGO_PKG_VERSION"))
        };
        let w = self.ui.width(ver, 1.0);
        self.ui.text(
            ver,
            f.ox + f.w - w - self.ui.px(0.6),
            f.oy + self.ui.px(0.5),
            1.0,
            DIM,
        );
    }
}
