//! 화면 글·메뉴(0019·0021). 플레이어가 보는 글은 고운바탕(명조, OFL — 표 글자만 남긴 서브셋 `tools/fonts/subset_ui.py`)으로,
//! 큰 제목(2배 이상)은 굵게. 크기는 렌더 배율에 맞춘다 —
//! 렌더 배율(`Ui::scale`)과 같은 배수라 게임 화면의 픽셀과 굵기가 맞는다. 개발 화면(랩·측정 줄)은 기본 글꼴 그대로(`text.rs`).
//!
//! 메뉴는 커서 없이(위·아래 + 확인), 터치는 항목을 뗄 때 누른다(`Actions::tap`). 그리는 함수가 항목 자리를 돌려주고
//! 앱이 탭을 맞춰 본다 — 그림과 판정이 같은 자리를 쓴다.
//!
//! 글자 아틀라스: 쓰는 크기의 모든 글자(두 언어 표의 글자 + ASCII)를 그리기 전에 굽는다(`warm`) — 프레임 중간에 아틀라스가
//! 커지면 옛 텍스처가 지워져 WebGL 에러가 났다(`text.rs`, 2026-11-03).
use macroquad::prelude::*;

use crate::i18n::{self, Lang, Strings};

/// 본문 글 크기(UI 1배 = 480×270 기준 px, 0041). 명조라 픽셀 격자가 아니다 — 1배(480×270 창)에서도 읽히게 13.
pub const GLYPH: f32 = 13.0;

pub struct Ui {
    font: Font,
    bold: Font,
    /// 정수 배율(렌더 타깃과 같은 값). 글 크기는 `GLYPH × 배수 × scale`.
    pub scale: f32,
    warmed: (u32, f32),
}

pub const INK: Color = Color::new(0.88, 0.84, 0.76, 1.0);
pub const DIM: Color = Color::new(0.62, 0.58, 0.52, 1.0);
pub const AMBER: Color = Color::new(0.88, 0.6, 0.23, 1.0);
pub const SHADOW: Color = Color::new(0.03, 0.03, 0.04, 0.9);

impl Ui {
    pub fn new() -> Self {
        let load = |bytes: &[u8]| {
            let mut f = load_ttf_font_from_bytes(bytes).expect("UI font");
            // 명조는 픽셀 격자가 아니라 부드럽게(고밀도 화면의 소수 배율에서도 매끈하게).
            f.set_filter(FilterMode::Linear);
            f
        };
        let font = load(include_bytes!("../assets/fonts/GowunBatang-UI-Regular.ttf"));
        let bold = load(include_bytes!("../assets/fonts/GowunBatang-UI-Bold.ttf"));
        Ui {
            font,
            bold,
            scale: 1.0,
            warmed: (0, 0.0),
        }
    }

    /// 프레임마다(그리기 전). 배율·언어·DPI가 바뀌었을 때만 굽는다.
    pub fn begin(&mut self, scale: f32) {
        self.scale = scale.max(1.0);
        let key = (
            (self.scale * 10.0) as u32 * 10 + i18n::lang() as u32,
            screen_dpi_scale(),
        );
        if key == self.warmed {
            return;
        }
        self.warmed = key;
        let mut chars: Vec<char> = (32u8..127).map(char::from).collect();
        for l in Lang::ALL {
            for c in all_text(l.strings()).chars() {
                if !chars.contains(&c) {
                    chars.push(c);
                }
            }
        }
        let dpi = screen_dpi_scale();
        for k in [1.0, 2.0, 3.0] {
            let size = (self.px(k) * dpi).ceil() as u16;
            self.font.populate_font_cache(&chars, size);
            if k >= 2.0 {
                self.bold.populate_font_cache(&chars, size);
            }
        }
    }

    /// 큰 제목(2배 이상)은 굵게.
    fn face(&self, k: f32) -> &Font {
        if k >= 2.0 {
            &self.bold
        } else {
            &self.font
        }
    }

    /// 글 크기 `k`배(1 = 본문, 2 = 큰 제목)의 논리 px.
    pub fn px(&self, k: f32) -> f32 {
        GLYPH * k * self.scale
    }

    fn params(&self, k: f32, color: Color) -> TextParams<'_> {
        TextParams {
            font: Some(self.face(k)),
            font_size: self.px(k) as u16,
            font_scale: 1.0,
            color,
            ..Default::default()
        }
    }

    pub fn width(&self, s: &str, k: f32) -> f32 {
        measure_text(s, Some(self.face(k)), self.px(k) as u16, 1.0).width
    }

    /// 그림자를 깐 글(어두운 배경·밝은 배경 어디서나 읽히게). `y`는 글줄의 위.
    pub fn text(&self, s: &str, x: f32, y: f32, k: f32, color: Color) {
        let base = y + self.px(k) * 0.85;
        let o = self.scale * k;
        draw_text_ex(s, x + o, base + o, self.params(k, SHADOW));
        draw_text_ex(s, x, base, self.params(k, color));
    }

    pub fn text_center(&self, s: &str, cx: f32, y: f32, k: f32, color: Color) {
        let w = self.width(s, k);
        self.text(s, cx - w * 0.5, y, k, color);
    }

    /// 세로 메뉴. 고른 줄은 호박색 + 앞에 표지. 항목 자리(탭 판정용)를 돌려준다.
    pub fn menu(&self, items: &[String], cursor: usize, cx: f32, y: f32) -> Vec<Rect> {
        let gap = self.px(1.0) * 1.9;
        let mut rects = Vec::new();
        for (i, it) in items.iter().enumerate() {
            let yy = y + gap * i as f32;
            let w = self.width(it, 1.0);
            let sel = i == cursor;
            if sel {
                let pad = self.px(0.5);
                draw_rectangle(
                    cx - w * 0.5 - pad * 2.0,
                    yy - pad * 0.4,
                    w + pad * 4.0,
                    self.px(1.0) + pad * 0.8,
                    Color::new(0.88, 0.6, 0.23, 0.14),
                );
                self.text("◆", cx - w * 0.5 - pad * 3.4, yy, 1.0, AMBER);
            }
            self.text_center(it, cx, yy, 1.0, if sel { AMBER } else { INK });
            rects.push(Rect::new(
                cx - w * 0.5 - self.px(1.5),
                yy - gap * 0.25,
                w + self.px(3.0),
                gap,
            ));
        }
        rects
    }

    /// 키 안내 한 줄: [(키, 동사)]. 터치(키 그림 없음)면 그리지 않는다.
    /// `max_w`보다 넓으면 여러 줄로 나눈다(마지막 줄이 `y`, 앞줄은 위로). 쓴 맨 윗줄의 y를 돌려준다 —
    /// 위에 다른 안내를 겹치지 않게 놓는다(0041: 테두리가 없어진 720p에서 한 줄 안내가 창 밖으로 잘렸다).
    pub fn hints(&self, pairs: &[(&str, &str)], cx: f32, y: f32, max_w: f32) -> f32 {
        let items: Vec<String> = pairs
            .iter()
            .filter(|(k, _)| !k.is_empty())
            .map(|(k, v)| format!("{k} {v}"))
            .collect();
        if items.is_empty() {
            return y;
        }
        let sep = "   ";
        let mut lines: Vec<String> = vec![String::new()];
        for it in items {
            let last = lines.last_mut().unwrap();
            let cand = if last.is_empty() {
                it.clone()
            } else {
                format!("{last}{sep}{it}")
            };
            if last.is_empty() || self.width(&cand, 1.0) <= max_w {
                *last = cand;
            } else {
                lines.push(it);
            }
        }
        let gap = self.px(1.0) * 1.25;
        let top = y - gap * (lines.len() - 1) as f32;
        for (i, l) in lines.iter().enumerate() {
            self.text_center(l, cx, top + gap * i as f32, 1.0, DIM);
        }
        top
    }

    /// 화면 전체를 어둡게(메뉴 뒤).
    pub fn dim(&self, a: f32) {
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.02, 0.02, 0.03, a),
        );
    }
}

/// 메뉴 입력 한 틱: 위·아래로 커서, 확인·탭이면 고른 줄.
pub fn menu_input(cursor: &mut usize, n: usize, up: bool, down: bool) {
    if n == 0 {
        return;
    }
    if up {
        *cursor = (*cursor + n - 1) % n;
    }
    if down {
        *cursor = (*cursor + 1) % n;
    }
}

/// 탭 자리가 어느 줄인가.
pub fn hit(rects: &[Rect], p: Vec2) -> Option<usize> {
    rects.iter().position(|r| r.contains(p))
}

/// 표의 모든 글 + 기호(글자 굽기).
pub fn all_text(s: &Strings) -> String {
    let mut v = i18n::all(s).concat();
    v.push_str(i18n::UI_GLYPHS);
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_cursor_wraps_both_ways() {
        let mut c = 0;
        menu_input(&mut c, 3, true, false);
        assert_eq!(c, 2);
        menu_input(&mut c, 3, false, true);
        assert_eq!(c, 0);
        assert_eq!(
            hit(&[Rect::new(0.0, 0.0, 10.0, 10.0)], vec2(5.0, 5.0)),
            Some(0)
        );
    }
}
