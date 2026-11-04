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

use crate::i18n::{self, Strings};

/// 본문 글 크기(UI 1배 = 480×270 기준 px, 0041). 명조라 픽셀 격자가 아니다 — 1배(480×270 창)에서도 읽히게 13.
pub const GLYPH: f32 = 13.0;

/// 글자 하나를 굽는 크기의 상한(물리 px, 0049). 더 크게 보일 글은 이 크기로 구워 `font_scale`로 늘린다.
/// 화면 크기로 다 구우면 한글 수백 자 × 크기 셋이 아틀라스를 4096²~8192²로 키웠고, 폰 GPU(낮은 정밀도 uv)에서
/// 글자가 검게 나왔다(KB `reference/macroquad/text-rendering-and-browser-frame-rate.md`). 1080p 본문(52px)은 그대로 굽는다.
///
/// 0054: 상한만으로는 모자랐다 — 전체 화면을 켜고 끌 때마다(창 높이 → UI 배율이 바뀜) 새 크기로 또 구워 아틀라스가 다시 4096²로 컸고
/// 폰에서 글자가 다시 검어졌다. 그래서 **화면 크기와 상관없이 한 크기로만** 굽는다(물리 `BAKE`px 가까이, 글자 크기 정수) —
/// 창이 바뀌어도 새로 굽는 글자가 없다. 보이는 크기는 모두 `font_scale`.
const BAKE: f32 = 48.0;

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

/// 키캡(nail 0153): 글 속 키·버튼 이름을 이 두 표식 사이에 두면(`key`) 테두리 있는 작은 칸 + 밝은 글로 그린다.
/// 표식은 사용 영역 글자라 폰트에 없고 그리지 않는다. `text`·`width`가 알아보니 가운데 맞춤·`fit`·`wrap`이 그대로 된다.
pub const KEY_OPEN: char = '\u{E000}';
pub const KEY_CLOSE: char = '\u{E001}';
pub const KEY_TEXT: Color = Color::new(1.0, 0.93, 0.78, 1.0);
const KEY_FILL: Color = Color::new(0.08, 0.06, 0.10, 0.85);
const KEY_EDGE: Color = Color::new(0.62, 0.50, 0.38, 1.0);
/// 키캡 안쪽 여백·칸 사이 틈(글 크기에 대한 비).
const KEY_PAD: f32 = 0.28;
const KEY_GAP: f32 = 0.12;

/// 키·버튼 이름을 키캡 표식으로 감싼다.
pub fn key(name: &str) -> String {
    format!("{KEY_OPEN}{name}{KEY_CLOSE}")
}

/// 표식을 뺀 글(넘침 검사·시험 — 운영판에는 없다).
#[cfg(any(feature = "dev", test))]
pub fn plain(s: &str) -> String {
    s.chars()
        .filter(|c| *c != KEY_OPEN && *c != KEY_CLOSE)
        .collect()
}

/// (조각, 키캡인가). 닫는 표식이 없으면(잘린 글) 끝까지 키캡.
fn key_segments(s: &str) -> Vec<(String, bool)> {
    let mut out: Vec<(String, bool)> = Vec::new();
    let mut cur = String::new();
    let mut in_key = false;
    for c in s.chars() {
        match c {
            KEY_OPEN if !in_key => {
                if !cur.is_empty() {
                    out.push((std::mem::take(&mut cur), false));
                }
                in_key = true;
            }
            KEY_CLOSE if in_key => {
                out.push((std::mem::take(&mut cur), true));
                in_key = false;
            }
            _ => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push((cur, in_key));
    }
    out
}

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
        // 굽는 크기는 DPI만 따른다(0054) — 배율이 바뀌어도 다시 굽지 않는다.
        let key = (i18n::lang() as u32, screen_dpi_scale());
        if key == self.warmed {
            return;
        }
        self.warmed = key;
        // 본문(1배)만, 지금 언어의 표 글자 + ASCII만 미리 굽는다(0049). 큰 제목(굵게)은 글자가 적어 처음 그릴 때 굽는다 —
        // 본문이 다 들어간 아틀라스에 자리가 남아 커지지 않는다(커지면 그 프레임의 글이 지운 텍스처를 본다, `text.rs`).
        let mut chars: Vec<char> = (32u8..127).map(char::from).collect();
        for c in all_text(i18n::lang().strings()).chars() {
            if !chars.contains(&c) {
                chars.push(c);
            }
        }
        let (fs, _) = self.sizing(1.0);
        let dpi = screen_dpi_scale();
        // macroquad는 `ceil(font_size × dpi)`로 굽는다 — 그리는 크기와 같은 셈이어야 구운 것을 쓴다.
        self.font
            .populate_font_cache(&chars, (fs as f32 * dpi).ceil() as u16);
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

    /// 굽는 크기(논리 px, 정수)와 늘리는 배(0049·0054): 굽는 크기는 DPI로만 정하고 보이는 크기는 배로.
    fn sizing(&self, k: f32) -> (u16, f32) {
        bake_size(self.px(k), screen_dpi_scale())
    }

    fn params(&self, k: f32, color: Color) -> TextParams<'_> {
        let (fs, scale) = self.sizing(k);
        TextParams {
            font: Some(self.face(k)),
            font_size: fs,
            font_scale: scale,
            color,
            ..Default::default()
        }
    }

    fn plain_w(&self, s: &str, k: f32) -> f32 {
        let (fs, scale) = self.sizing(k);
        measure_text(s, Some(self.face(k)), fs, scale).width
    }

    /// 글 폭(키캡이면 칸 여백까지).
    pub fn width(&self, s: &str, k: f32) -> f32 {
        if !s.contains(KEY_OPEN) {
            return self.plain_w(s, k);
        }
        let px = self.px(k);
        key_segments(s)
            .iter()
            .map(|(seg, is_key)| {
                if *is_key {
                    self.plain_w(seg, k) + px * (2.0 * KEY_PAD + KEY_GAP)
                } else {
                    self.plain_w(seg, k)
                }
            })
            .sum()
    }

    /// 그림자를 깐 글(어두운 배경·밝은 배경 어디서나 읽히게). `y`는 글줄의 위.
    /// 키캡(`key`)이 든 글은 키를 칸 + 밝은 글로, 나머지가 흐린 색이면 본문 색으로 올린다(nail 0153: 밝은 바닥에서 안내가 묻혔다).
    pub fn text(&self, s: &str, x: f32, y: f32, k: f32, color: Color) {
        #[cfg(feature = "dev")]
        if color.a > 0.05 {
            check::text(&plain(s), Rect::new(x, y, self.width(s, k), self.px(k)));
        }
        if !s.contains(KEY_OPEN) {
            self.plain_text(s, x, y, k, color);
            return;
        }
        let dim = (color.r + color.g + color.b) / 3.0 < 0.7;
        let body = if dim {
            Color { a: color.a, ..INK }
        } else {
            color
        };
        let px = self.px(k);
        let mut cx = x;
        for (seg, is_key) in key_segments(s) {
            if is_key {
                let w = self.plain_w(&seg, k) + 2.0 * KEY_PAD * px;
                let (ry, rh) = (y - px * 0.08, px * 1.16);
                draw_rectangle(
                    cx,
                    ry,
                    w,
                    rh,
                    Color {
                        a: KEY_FILL.a * color.a,
                        ..KEY_FILL
                    },
                );
                draw_rectangle_lines(
                    cx,
                    ry,
                    w,
                    rh,
                    self.scale.max(1.0),
                    Color {
                        a: color.a,
                        ..KEY_EDGE
                    },
                );
                self.plain_text(
                    &seg,
                    cx + KEY_PAD * px,
                    y,
                    k,
                    Color {
                        a: color.a,
                        ..KEY_TEXT
                    },
                );
                cx += w + KEY_GAP * px;
            } else {
                self.plain_text(&seg, cx, y, k, body);
                cx += self.plain_w(&seg, k);
            }
        }
    }

    fn plain_text(&self, s: &str, x: f32, y: f32, k: f32, color: Color) {
        let base = y + self.px(k) * 0.85;
        let o = self.scale * k;
        draw_text_ex(s, x + o, base + o, self.params(k, SHADOW));
        draw_text_ex(s, x, base, self.params(k, color));
    }

    /// 폭에 안 맞으면 뒤를 잘라 "…"를 붙인다(nail `fit`). 언어마다 길이가 달라 고정 칸에서 넘칠 때의 마지막 안전장치.
    pub fn fit(&self, s: &str, max_w: f32, k: f32) -> String {
        if self.width(s, k) <= max_w {
            return s.to_string();
        }
        #[cfg(feature = "dev")]
        check::cut(&plain(s), max_w);
        let mut cut: Vec<char> = s.chars().collect();
        while !cut.is_empty() {
            cut.pop();
            let t: String = cut.iter().collect::<String>().trim_end().to_string() + "…";
            if self.width(&t, k) <= max_w {
                return t;
            }
        }
        "…".to_string()
    }

    /// 가운데 맞춘 긴 글: `max_w`에서 줄을 바꿔(`wrap`) 아래로 쌓는다. 다음 줄의 y를 돌려준다.
    pub fn text_center_wrapped(
        &self,
        s: &str,
        cx: f32,
        y: f32,
        k: f32,
        color: Color,
        max_w: f32,
    ) -> f32 {
        let gap = self.px(k) * 1.45;
        let mut yy = y;
        for line in wrap(self, s, max_w, k) {
            self.text_center(&line, cx, yy, k, color);
            yy += gap;
        }
        yy
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

    /// 키 안내 한 줄: [(키, 동사)] — 키는 키캡. 터치(키 그림 없음)면 그리지 않는다. 넓으면 `fit`으로 자른다
    /// (nail: 안내는 짧게 — 타이틀·멈춤은 고르기·확인·뒤로만, 전체 조작은 조작 설정 화면에).
    pub fn hints(&self, pairs: &[(&str, &str)], cx: f32, y: f32, max_w: f32) {
        let line = pairs
            .iter()
            .filter(|(k, _)| !k.is_empty())
            .map(|(k, v)| format!("{} {v}", key(k)))
            .collect::<Vec<_>>()
            .join("   ");
        if line.is_empty() {
            return;
        }
        let line = self.fit(&line, max_w, 1.0);
        self.text_center(&line, cx, y, 1.0, DIM);
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

/// 논리 크기 `want`를 그릴 때 굽는 크기(논리 px, 정수)와 늘리는 배 — 굽는 크기는 DPI로만 정한다(물리 ≈ `BAKE`px, 0054).
pub fn bake_size(want: f32, dpi: f32) -> (u16, f32) {
    let fs = (BAKE / dpi.max(0.01)).round().max(1.0);
    (fs as u16, want / fs)
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

    /// 0049: 굽는 물리 크기는 상한을 넘지 않고, 늘린 결과는 원하는 크기와 같다. 1080p 본문(52px)은 늘리지 않는다.
    #[test]
    fn glyphs_bake_at_one_size_whatever_the_screen() {
        // 0054: 같은 DPI면 어떤 크기를 그리든 굽는 크기는 하나(창을 바꿔도 새로 굽지 않는다), 물리 ≈ 48px.
        for dpi in [1.0, 1.5, 2.0, 2.625, 3.0] {
            let sizes: Vec<u16> = [13.0, 17.3, 20.0, 34.7, 52.0, 104.0, 156.0]
                .iter()
                .map(|w| bake_size(*w, dpi).0)
                .collect();
            assert!(sizes.windows(2).all(|x| x[0] == x[1]), "{dpi}: {sizes:?}");
            let phys = (sizes[0] as f32 * dpi).ceil();
            assert!((phys - BAKE).abs() <= dpi + 1.0, "{dpi}: bakes {phys}px");
            for w in [17.3, 52.0] {
                let (fs, k) = bake_size(w, dpi);
                assert!((fs as f32 * k - w).abs() < 1e-3);
            }
        }
    }

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

/// 줄바꿈(nail `wrap`). 띄어쓰기가 있는 글(한국어·영어)은 단어 단위, 띄어쓰기가 없는 글(일본어·중국어)은 글자 단위 + 금칙.
pub fn wrap(ui: &Ui, s: &str, max_w: f32, k: f32) -> Vec<String> {
    wrap_with(s, max_w, |t| ui.width(t, k))
}

/// 한자·가나·전각 문장부호: 이 글자 앞뒤에서는 띄어쓰기 없이도 줄을 바꿀 수 있다.
fn is_cjk(c: char) -> bool {
    matches!(c as u32, 0x3000..=0x30FF | 0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF | 0xFF00..=0xFFEF)
        || c == '…'
}

/// 금칙: 줄 맨 앞에 오면 안 되는 글자(닫는 부호·구두점·작은 가나·장음).
fn no_line_start(c: char) -> bool {
    "、。，．！？：；）」』】〕…ー〜・ぁぃぅぇぉっゃゅょゎァィゥェォッャュョヮ!?,.:;)".contains(c)
}

/// 금칙: 줄 맨 끝에 오면 안 되는 글자(여는 부호).
fn no_line_end(c: char) -> bool {
    "（「『【〔(".contains(c)
}

/// 측정 함수를 받는 줄바꿈(시험용으로 분리).
pub fn wrap_with(s: &str, max_w: f32, measure: impl Fn(&str) -> f32) -> Vec<String> {
    let mut units: Vec<(String, bool)> = Vec::new();
    let mut space = false;
    for c in s.chars() {
        if c == ' ' {
            space = true;
            continue;
        }
        let joins_prev = !space
            && units
                .last()
                .is_some_and(|(u, _)| !is_cjk(c) && !u.chars().last().is_some_and(is_cjk));
        if joins_prev {
            units.last_mut().unwrap().0.push(c);
        } else {
            units.push((c.to_string(), space));
        }
        space = false;
    }
    // 한 조각이 혼자서도 줄보다 길면 글자 단위로 쪼갠다.
    let units: Vec<(String, bool)> = units
        .into_iter()
        .flat_map(|(u, sp)| {
            if measure(&u) <= max_w {
                return vec![(u, sp)];
            }
            let mut parts: Vec<(String, bool)> = Vec::new();
            for c in u.chars() {
                match parts.last_mut() {
                    Some((p, _)) if measure(&format!("{p}{c}")) <= max_w => p.push(c),
                    _ => parts.push((c.to_string(), parts.is_empty() && sp)),
                }
            }
            parts
        })
        .collect();
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    for (u, sp) in units {
        let joined = if cur.is_empty() {
            u.clone()
        } else if sp {
            format!("{cur} {u}")
        } else {
            format!("{cur}{u}")
        };
        let starts_badly = u.chars().next().is_some_and(no_line_start);
        if cur.is_empty() || measure(&joined) <= max_w || starts_badly {
            cur = joined;
            continue;
        }
        let mut carry = String::new();
        if let Some(last) = cur.chars().last().filter(|c| no_line_end(*c)) {
            cur.pop();
            carry.push(last);
        }
        lines.push(cur.trim_end().to_string());
        cur = carry + &u;
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

/// 개발 빌드의 글자 넘침 검사(nail 0089, F4로 켬/끔). 그린 글 칸을 프레임마다 모아 화면 밖·게임 화면 밖·겹침·`fit` 잘림을
/// 콘솔에 한 번씩 찍는다(웹은 브라우저 콘솔). 헤드리스로 화면 × 언어를 돌며 모으면 사람 눈 전에 거른다.
#[cfg(feature = "dev")]
pub mod check {
    use macroquad::logging::warn;
    use macroquad::prelude::Rect;
    use std::cell::RefCell;
    use std::collections::HashSet;

    #[derive(Default)]
    struct State {
        on: bool,
        texts: Vec<(String, Rect)>,
        seen: HashSet<String>,
    }

    thread_local! {
        static S: RefCell<State> = RefCell::new(State::default());
    }

    pub fn on() -> bool {
        S.with(|s| s.borrow().on)
    }

    pub fn toggle() {
        S.with(|s| {
            let mut s = s.borrow_mut();
            s.on = !s.on;
            s.seen.clear();
            warn!("[ui-check] {}", if s.on { "on" } else { "off" });
        });
    }

    pub(super) fn text(t: &str, r: Rect) {
        if t.trim().is_empty() || !on() {
            return;
        }
        S.with(|s| s.borrow_mut().texts.push((t.to_string(), r)));
    }

    pub(super) fn cut(t: &str, max_w: f32) {
        if on() {
            report(format!("cut ({max_w:.0}px) {t}"));
        }
    }

    fn report(msg: String) {
        S.with(|s| {
            let mut s = s.borrow_mut();
            if s.seen.insert(msg.clone()) {
                warn!("[ui-check] {}", msg);
            }
        });
    }

    /// 두 칸이 1px 넘게 겹치는가(맞닿은 줄은 겹침이 아니다).
    fn overlap(a: Rect, b: Rect) -> bool {
        let w = a.right().min(b.right()) - a.x.max(b.x);
        let h = a.bottom().min(b.bottom()) - a.y.max(b.y);
        w > 1.5 && h > 1.5
    }

    /// 프레임 끝에: 모은 칸을 화면(`screen`)·게임 화면(`game`, 정수 배율로 올린 렌더 타깃)과 견주고 비운다.
    pub fn end_frame(screen: Rect, game: Rect) {
        if !on() {
            return;
        }
        let texts = S.with(|s| std::mem::take(&mut s.borrow_mut().texts));
        for (i, (t, r)) in texts.iter().enumerate() {
            for (name, f) in [("off-screen", screen), ("past-game", game)] {
                if r.x < f.x - 0.5
                    || r.y < f.y - 0.5
                    || r.right() > f.right() + 0.5
                    || r.bottom() > f.bottom() + 0.5
                {
                    report(format!("{name} x{:.0} y{:.0} w{:.0} {t}", r.x, r.y, r.w));
                    break;
                }
            }
            for (u, q) in texts.iter().skip(i + 1) {
                if u != t && overlap(*r, *q) {
                    report(format!("overlap {t} <> {u}"));
                }
            }
        }
    }
}

#[cfg(test)]
mod wrap_tests {
    use super::*;

    fn w(s: &str) -> f32 {
        s.chars().count() as f32
    }

    #[test]
    fn wraps_words_with_spaces() {
        assert_eq!(wrap_with("aa bb cc", 5.0, w), vec!["aa bb", "cc"]);
        assert_eq!(wrap_with("abcdefgh", 3.0, w), vec!["abc", "def", "gh"]);
        assert_eq!(
            wrap_with("뿌리가 닿은 곳마다", 6.0, w),
            vec!["뿌리가 닿은", "곳마다"]
        );
    }

    #[test]
    fn wraps_cjk_by_character_with_kinsoku() {
        assert_eq!(
            wrap_with("あいうえお。かき", 5.0, w),
            vec!["あいうえお。", "かき"]
        );
        assert_eq!(
            wrap_with("あいうえ（かき）", 5.0, w),
            vec!["あいうえ", "（かき）"]
        );
    }

    /// 키캡 조각(nail 0153): 표식 사이가 키, 잘려 닫는 표식이 없으면 끝까지 키.
    #[test]
    fn key_segments_split_keys() {
        let t = format!("{} 고르기   {} 확인", key("W/S"), key("ENTER"));
        let segs = key_segments(&t);
        assert_eq!(segs[0], ("W/S".to_string(), true));
        assert_eq!(segs[1], (" 고르기   ".to_string(), false));
        assert_eq!(segs[2], ("ENTER".to_string(), true));
        assert_eq!(
            key_segments(&format!("A {KEY_OPEN}En")),
            vec![("A ".into(), false), ("En".into(), true)]
        );
        assert_eq!(plain(&t), "W/S 고르기   ENTER 확인");
    }
}
