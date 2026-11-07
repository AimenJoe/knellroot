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
                TitleItem::Dev => self.go(Go::Open(Scene::Dev { tab: 0 })),
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
                // 개발 창 이름(언어를 따른다, 0080).
                #[cfg(feature = "dev")]
                TitleItem::Dev => crate::i18n::dev::t(crate::i18n::dev::DEV_TITLE).to_string(),
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
        // 버전 글자(오른쪽 위) — 받은 판이 무엇인지 사진 한 장으로 알게. 개발판은 표시 + 커밋 해시 앞 7자(0079 — 개발판끼리 가르게,
        // 개발 웹 `build.txt`와 앞이 같다).
        let ver = version_label(cfg!(feature = "dev"), env!("BUILD_HASH"));
        let ver = ver.as_str();
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

/// 타이틀 오른쪽 위 버전 글자: 운영판 "v0.3.3", 개발판 "v0.3.3 dev 674d4c8"(해시를 모르면 "v0.3.3 dev").
pub(super) fn version_label(dev: bool, hash: &str) -> String {
    let v = concat!("v", env!("CARGO_PKG_VERSION"));
    match (dev, hash.is_empty()) {
        (false, _) => v.to_string(),
        (true, true) => format!("{v} dev"),
        (true, false) => format!("{v} dev {hash}"),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn dev_builds_show_the_commit_next_to_the_version() {
        let v = concat!("v", env!("CARGO_PKG_VERSION"));
        assert_eq!(
            super::version_label(false, "674d4c8"),
            v,
            "production: version only"
        );
        assert_eq!(
            super::version_label(true, "674d4c8"),
            format!("{v} dev 674d4c8")
        );
        assert_eq!(super::version_label(true, ""), format!("{v} dev"));
        assert_eq!(
            env!("BUILD_HASH").len(),
            7,
            "the build knows its commit here"
        );
    }
}
