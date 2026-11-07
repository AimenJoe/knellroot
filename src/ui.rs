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

/// 창 틀(0066, nail `ui::panel`의 꼴과 색): 어두운 면 + 청동 테두리, 고른 줄은 조금 밝은 면.
pub const PANEL: Color = Color::new(0.07, 0.05, 0.09, 0.94);
pub const PANEL_LIGHT: Color = Color::new(0.16, 0.11, 0.14, 1.0);
pub const BORDER: Color = Color::new(0.55, 0.40, 0.28, 1.0);

/// 창 안 목록의 한 줄(0066, nail `menu_list`·`setting_row`): 이름은 왼쪽, 값은 오른쪽.
#[derive(Debug, Clone, Default)]
pub struct Row {
    pub label: String,
    /// 오른쪽 값(값·값어치). 비면 그리지 않는다.
    pub value: String,
    pub kind: RowKind,
    /// 지금 고를 수 없음(흐리게 — 가게에서 잔불이 모자란 물건).
    pub dim: bool,
    /// 이름 앞에 그림 자리를 둔다(`list_icon`이 그 칸, 그림은 부른 쪽이 — 0068 아이템 아이콘).
    pub icon: bool,
    /// 오른쪽 값을 작은 표지로(조금 작은 글 — "장비 중" 같은 표시가 이름을 잘라 먹지 않게).
    pub tag: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RowKind {
    /// 누르는 줄(값이 있으면 오른쪽에 그대로).
    #[default]
    Plain,
    /// 좌우로 바꾸는 값: ◀ 값 ▶(화살표를 누르면 바뀐다).
    Step,
    /// 다른 창으로 가는 줄: 값 자리에 →.
    Go,
}

impl Row {
    pub fn plain(label: impl Into<String>) -> Self {
        Row {
            label: label.into(),
            ..Row::default()
        }
    }
    pub fn value(label: impl Into<String>, value: impl Into<String>) -> Self {
        Row {
            label: label.into(),
            value: value.into(),
            ..Row::default()
        }
    }
    pub fn step(label: impl Into<String>, value: impl Into<String>) -> Self {
        Row {
            kind: RowKind::Step,
            ..Row::value(label, value)
        }
    }
    pub fn go(label: impl Into<String>) -> Self {
        Row {
            kind: RowKind::Go,
            ..Row::plain(label)
        }
    }
    pub fn dimmed(self, dim: bool) -> Self {
        Row { dim, ..self }
    }
    pub fn with_icon(self, icon: bool) -> Self {
        Row { icon, ..self }
    }
    pub fn tagged(self) -> Self {
        Row { tag: true, ..self }
    }
}

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

    /// 세로 메뉴(`top`..`bottom` 안). 고른 줄은 호박색 + 앞에 표지. 줄이 많으면 간격을 조금 줄이고(1.9 → 1.6줄),
    /// 그래도 넘치면 굴린다 — 보이는 줄만 그리고 위·아래에 더 있으면 ▲▼(누를 수 있다). 창은 `view`가 기억한다(nail 0199).
    /// 돌려주는 줄 자리는 항목마다(안 보이는 줄은 빈 사각형 — 탭이 닿지 않는다).
    pub fn menu(
        &self,
        items: &[String],
        cursor: usize,
        view: &mut ListView,
        cx: f32,
        top: f32,
        bottom: f32,
    ) -> MenuHits {
        self.menu_dimmed(items, &[], cursor, view, cx, top, bottom)
    }

    /// `menu`와 같고, `dim[i]`가 참인 줄은 흐리게(지금 고를 수 없는 것 — 가게에서 잔불이 모자란 물건, nail 가게와 같은 규칙).
    #[allow(clippy::too_many_arguments)]
    pub fn menu_dimmed(
        &self,
        items: &[String],
        dim: &[bool],
        cursor: usize,
        view: &mut ListView,
        cx: f32,
        top: f32,
        bottom: f32,
    ) -> MenuHits {
        let px = self.px(1.0);
        let n = items.len();
        let avail = (bottom - top).max(px);
        let gap = (px * 1.9).min(avail / n.max(1) as f32).max(px * 1.6);
        let (fits, y0) = if gap * n as f32 <= avail + 1.0 {
            (n, top)
        } else {
            // ▲▼ 자리를 빼고 들어가는 줄 수.
            let arrow = px * 0.9;
            (
                ((avail - arrow * 2.0) / gap).floor().max(1.0) as usize,
                top + arrow,
            )
        };
        let first = view.window(cursor, n, fits, gap);
        let mut rows = vec![Rect::new(0.0, 0.0, 0.0, 0.0); n];
        for (i, it) in items.iter().enumerate().skip(first).take(fits) {
            let yy = y0 + gap * (i - first) as f32;
            let w = self.width(it, 1.0);
            let sel = i == cursor;
            if sel {
                let pad = self.px(0.5);
                draw_rectangle(
                    cx - w * 0.5 - pad * 2.0,
                    yy - pad * 0.4,
                    w + pad * 4.0,
                    px + pad * 0.8,
                    Color::new(0.88, 0.6, 0.23, 0.14),
                );
                self.text("◆", cx - w * 0.5 - pad * 3.4, yy, 1.0, AMBER);
            }
            let off = dim.get(i).copied().unwrap_or(false);
            let col = match (sel, off) {
                (true, false) => AMBER,
                (true, true) => Color::new(AMBER.r, AMBER.g, AMBER.b, 0.5),
                (false, false) => INK,
                (false, true) => Color::new(INK.r, INK.g, INK.b, 0.4),
            };
            self.text_center(it, cx, yy, 1.0, col);
            rows[i] = Rect::new(cx - w * 0.5 - px * 1.5, yy - gap * 0.25, w + px * 3.0, gap);
        }
        let arrows = self.scroll_arrows(
            view,
            n,
            cx,
            y0 - px * 0.95,
            y0 + gap * fits as f32 - px * 0.15,
        );
        MenuHits {
            steps: vec![None; rows.len()],
            rows,
            arrows,
        }
    }

    /// 굴리는 목록의 ▲(위에 더 있음)·▼(아래에 더 있음): `y_up`·`y_down` 줄에, 없는 쪽은 그리지 않는다. 누를 자리.
    pub fn scroll_arrows(
        &self,
        view: &ListView,
        n: usize,
        cx: f32,
        y_up: f32,
        y_down: f32,
    ) -> [Option<Rect>; 2] {
        let px = self.px(1.0);
        let (up, down) = view.more(n);
        let mut out = [None, None];
        for (k, (on, glyph, y)) in [(up, "▲", y_up), (down, "▼", y_down)]
            .into_iter()
            .enumerate()
        {
            if on {
                self.text_center(glyph, cx, y, 0.8, DIM);
                out[k] = Some(Rect::new(cx - px * 2.5, y - px * 0.3, px * 5.0, px * 1.2));
            }
        }
        out
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

    /// 480×270 기준 px `n`의 화면 px(창 틀·여백).
    pub fn u(&self, n: f32) -> f32 {
        n * self.scale
    }

    /// 창 틀(0066, nail `ui::panel`): 면 + 테두리, 제목이 있으면 왼쪽 위에 호박색 + 아래 선. 안쪽 글이 시작하는 y를 돌려준다.
    /// 제목은 오른쪽 닫기 단추 자리(`close`)를 비우고 `fit`한다.
    pub fn panel(&self, r: Rect, title: Option<&str>) -> f32 {
        let u = self.scale;
        draw_rectangle(r.x, r.y, r.w, r.h, PANEL);
        draw_rectangle_lines(r.x, r.y, r.w, r.h, u.max(1.0), BORDER);
        #[cfg(feature = "dev")]
        check::frame(r);
        match title {
            Some(t) => {
                let ty = r.y + u * 6.0;
                let t = self.fit(t, r.w - u * 40.0, 1.0);
                self.text(&t, r.x + u * 9.0, ty, 1.0, AMBER);
                let ly = (ty + self.px(1.0) + u * 5.0).round();
                draw_line(
                    r.x + u * 5.0,
                    ly,
                    r.x + r.w - u * 5.0,
                    ly,
                    u.max(1.0),
                    BORDER,
                );
                ly + u * 6.0
            }
            None => r.y + u * 8.0,
        }
    }

    /// 닫기 단추(nail `close_button`, 0198): 창 오른쪽 위 작은 칸에 ✕(선 둘). 누를 자리(그림보다 넉넉히)를 돌려준다 —
    /// 앱은 이 자리와 창 밖 탭을 "뒤로"로 읽는다(터치만으로도 나갈 길).
    pub fn close(&self, panel: Rect) -> Rect {
        let u = self.scale;
        let c = vec2(panel.x + panel.w - u * 13.0, panel.y + u * 13.0);
        let h = u * 7.0;
        draw_rectangle(c.x - h, c.y - h, h * 2.0, h * 2.0, PANEL_LIGHT);
        draw_rectangle_lines(c.x - h, c.y - h, h * 2.0, h * 2.0, u.max(1.0), BORDER);
        let a = u * 3.5;
        let w = (u * 1.2).max(1.0);
        draw_line(c.x - a, c.y - a, c.x + a, c.y + a, w, INK);
        draw_line(c.x - a, c.y + a, c.x + a, c.y - a, w, INK);
        Rect::new(c.x - h * 2.0, c.y - h * 2.0, h * 4.0, h * 4.0)
    }

    /// 눌리는 단추(nail `button`, 0157): 오른쪽 아래 그림자 위에 얹힌 면, 위·왼쪽 밝은 모서리. 고르면 면이 밝고 테두리가 호박색.
    pub fn button(&self, r: Rect, label: &str, sel: bool) {
        let u = self.scale;
        draw_rectangle(
            r.x + u * 2.0,
            r.y + u * 2.0,
            r.w,
            r.h,
            Color::new(0.02, 0.01, 0.03, 0.9),
        );
        draw_rectangle(r.x, r.y, r.w, r.h, if sel { PANEL_LIGHT } else { PANEL });
        let edge = if sel { AMBER } else { BORDER };
        let hi = Color { a: 0.45, ..edge };
        draw_rectangle(r.x + u, r.y + u, r.w - u * 2.0, u, hi);
        draw_rectangle(r.x + u, r.y + u, u, r.h - u * 2.0, hi);
        draw_rectangle_lines(r.x, r.y, r.w, r.h, u.max(1.0), edge);
        let label = self.fit(label, r.w - u * 6.0, 1.0);
        let px = self.px(1.0);
        self.text_center(
            &label,
            r.x + r.w * 0.5,
            r.y + (r.h - px) * 0.5,
            1.0,
            if sel { INK } else { DIM },
        );
    }

    /// 창 안 목록(0066, nail `menu_list`·`setting_row`): `area` 안에 줄을 왼쪽 맞춤으로, 고른 줄은 밝은 면 + ▶.
    /// 값은 오른쪽(`Step`이면 ◀ 값 ▶, `Go`면 →). 넘치면 `menu`처럼 굴린다(▲▼, 창은 `view`).
    pub fn list(&self, rows: &[Row], cursor: usize, view: &mut ListView, area: Rect) -> MenuHits {
        let px = self.px(1.0);
        let u = self.scale;
        let n = rows.len();
        let row_h = px * 1.75;
        // 넘치면 들어가는 만큼만 — ▲▼는 목록이 아니라 창 제목 줄에(`title_arrows`, nail 0239: 줄 자리를 먹지 않게).
        let fits = ((area.h + 1.0) / row_h).floor().max(1.0) as usize;
        let y0 = area.y;
        let first = view.window(cursor, n, fits, row_h);
        let none = Rect::new(0.0, 0.0, 0.0, 0.0);
        let mut hits = MenuHits {
            rows: vec![none; n],
            steps: vec![None; n],
            arrows: [None, None],
        };
        for (i, row) in rows.iter().enumerate().skip(first).take(fits) {
            let r = Rect::new(area.x, y0 + row_h * (i - first) as f32, area.w, row_h);
            let sel = i == cursor;
            let ty = r.y + (row_h - px) * 0.5;
            if sel {
                draw_rectangle(r.x, r.y, r.w, r.h, PANEL_LIGHT);
                self.text("▶", r.x + u * 4.0, ty, 1.0, AMBER);
            }
            let fade = |c: Color| if row.dim { Color { a: 0.4, ..c } } else { c };
            let ink = fade(if sel { INK } else { DIM });
            let mark = fade(if sel { AMBER } else { DIM });
            let right = r.x + r.w - u * 6.0;
            let label_x = r.x + u * 4.0 + px * 1.1 + if row.icon { px * 1.4 } else { 0.0 };
            // 값 칸의 왼쪽 끝(이름은 그 앞까지 `fit`).
            let value_left = match row.kind {
                RowKind::Step => {
                    let arrow = self.width("▶", 1.0) + u * 6.0;
                    let vw = self.width(&row.value, 1.0).max(px * 3.0) + u * 6.0;
                    let ra = Rect::new(right - arrow, r.y, arrow, row_h);
                    let la = Rect::new(ra.x - vw - arrow, r.y, arrow, row_h);
                    self.text("◀", la.x + u * 3.0, ty, 1.0, mark);
                    self.text_center(&row.value, la.x + arrow + vw * 0.5, ty, 1.0, ink);
                    self.text("▶", ra.x + u * 3.0, ty, 1.0, mark);
                    hits.steps[i] = Some([la, ra]);
                    la.x
                }
                RowKind::Go => {
                    let w = self.width("→", 1.0);
                    self.text("→", right - w, ty, 1.0, mark);
                    right - w
                }
                RowKind::Plain if row.value.is_empty() => right,
                RowKind::Plain => {
                    let k = if row.tag { 0.8 } else { 1.0 };
                    let w = self.width(&row.value, k);
                    let y = ty + (px - self.px(k)) * 0.5;
                    self.text(
                        &row.value,
                        right - w,
                        y,
                        k,
                        if row.tag { mark } else { ink },
                    );
                    right - w
                }
            };
            let label = self.fit(&row.label, value_left - label_x - u * 6.0, 1.0);
            self.text(&label, label_x, ty, 1.0, ink);
            hits.rows[i] = r;
        }
        hits
    }

    /// 창 제목 줄의 ▲▼(nail 0239): 목록이 넘칠 때만, 닫기 단추 왼쪽에. 더 갈 수 없는 쪽은 흐리게. 누를 자리(열린 쪽만).
    pub fn title_arrows(&self, view: &ListView, n: usize, panel: Rect) -> [Option<Rect>; 2] {
        let (up, down) = view.more(n);
        let mut out = [None, None];
        if !(up || down) {
            return out;
        }
        let u = self.scale;
        let y = panel.y + u * 6.0;
        for (k, (open, glyph, dx)) in [(up, "▲", 54.0), (down, "▼", 36.0)].into_iter().enumerate()
        {
            let x = panel.x + panel.w - u * dx;
            let c = if open { DIM } else { Color { a: 0.3, ..DIM } };
            self.text(glyph, x, y, 1.0, c);
            if open {
                out[k] = Some(Rect::new(x - u * 4.0, y - u * 4.0, u * 18.0, u * 22.0));
            }
        }
        out
    }

    /// 목록 줄 `r`(`list`가 돌려준 자리)의 그림 칸 — `Row::icon` 줄의 이름 앞, 정사각 한 글줄 높이.
    pub fn list_icon(&self, r: Rect) -> Rect {
        let px = self.px(1.0);
        let side = px * 1.25;
        Rect::new(
            r.x + self.scale * 4.0 + px * 1.1,
            r.y + (r.h - side) * 0.5,
            side,
            side,
        )
    }

    /// 단추 줄(묻는 창, nail `draw_shop_ask`): `labels`를 `area` 가로로 나란히. 누를 자리를 돌려준다.
    pub fn buttons(&self, labels: &[&str], cursor: usize, area: Rect) -> MenuHits {
        let u = self.scale;
        let n = labels.len().max(1) as f32;
        let gap = u * 10.0;
        let w = (area.w - gap * (n - 1.0)) / n;
        let rows: Vec<Rect> = labels
            .iter()
            .enumerate()
            .map(|(i, l)| {
                let r = Rect::new(area.x + (w + gap) * i as f32, area.y, w, area.h);
                self.button(r, l, i == cursor);
                r
            })
            .collect();
        MenuHits {
            steps: vec![None; rows.len()],
            rows,
            arrows: [None, None],
        }
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
/// 메뉴가 그린 자리: 항목마다 줄(안 보이면 빈 사각형), ▲·▼(있을 때).
#[derive(Debug, Default, Clone)]
pub struct MenuHits {
    pub rows: Vec<Rect>,
    /// 줄마다 ◀·▶ 누를 자리(`RowKind::Step`인 줄만, 0066).
    pub steps: Vec<Option<[Rect; 2]>>,
    pub arrows: [Option<Rect>; 2],
}

/// 굴리는 목록의 창(nail 0199를 가져옴, Knellroot 메뉴 공통): 보이는 첫 줄을 기억한다. 키·패드 커서는 창이 따라가고(`window`),
/// 손가락 끌기·휠·▲▼는 창을 직접 옮기고 커서를 창 안으로 데려온다(`scroll`).
/// 커서는 창 가장자리에서 한 줄 띄운다(목록 끝이 아니면) — 다음 줄이 보여야 더 있는 줄 안다(게임 UI 관례, 2차).
#[derive(Debug, Default, Clone, Copy)]
pub struct ListView {
    pub first: usize,
    /// 지난번 그릴 때 들어간 줄 수와 줄 높이(논리 px).
    fits: usize,
    row_h: f32,
    /// 줄 하나가 안 된 끌기 거리(논리 px).
    acc: f32,
    /// 손가락으로 고른 줄(0076): 이 줄에 커서가 있는 동안은 가장자리 여유로 창을 옮기지 않는다 — 두 걸음 탭(고르기 → 다시 눌러 끼우기)에서
    /// 첫 탭에 목록이 한 줄 밀려 같은 자리의 둘째 탭이 다른 줄을 누르던 것(장비 창 무기 후보).
    tapped: Option<usize>,
}

impl ListView {
    /// 손가락이 `row`를 골랐다(창은 그대로 — 그 줄은 이미 보인다).
    pub fn tapped(&mut self, row: usize) {
        self.tapped = Some(row);
    }

    fn margin(&self) -> usize {
        usize::from(self.fits >= 3)
    }

    /// 그릴 때: 이번에 들어가는 줄 수·줄 높이를 적고, 커서가 창(가장자리 여유 포함) 밖이면 창을 옮긴다. 보이는 첫 줄.
    pub fn window(&mut self, cursor: usize, n: usize, fits: usize, row_h: f32) -> usize {
        self.fits = fits.max(1);
        self.row_h = row_h;
        let m = if self.tapped == Some(cursor) {
            0
        } else {
            self.tapped = None;
            self.margin()
        };
        if cursor < self.first + m {
            self.first = cursor.saturating_sub(m);
        }
        if cursor + m >= self.first + self.fits {
            self.first = cursor + m + 1 - self.fits;
        }
        self.first = self.first.min(n.saturating_sub(self.fits));
        self.first
    }

    /// 위·아래에 가려진 줄이 있나(▲▼).
    pub fn more(&self, n: usize) -> (bool, bool) {
        (self.first > 0, self.first + self.fits.max(1) < n)
    }

    /// 끌기(논리 px, 위로 끌면 +)와 칸(휠·▲▼, 아래로 +)만큼 창을 옮기고 커서를 창 안(가장자리 여유 포함)에 둔다. 창이 움직였으면 참.
    pub fn scroll(&mut self, (drag, steps): (f32, i32), cursor: &mut usize, n: usize) -> bool {
        let row_h = self.row_h.max(1.0);
        // 방향을 바꾸면 남은 거리는 버린다(반대로 끄는 첫 줄이 늦게 오지 않게).
        if self.acc * drag < 0.0 {
            self.acc = 0.0;
        }
        self.acc += drag;
        let mut rows = steps;
        while self.acc >= row_h {
            self.acc -= row_h;
            rows += 1;
        }
        while self.acc <= -row_h {
            self.acc += row_h;
            rows -= 1;
        }
        if rows == 0 || n == 0 {
            return false;
        }
        let fits = self.fits.max(1);
        let max_first = n.saturating_sub(fits) as i32;
        let first = (self.first as i32 + rows).clamp(0, max_first) as usize;
        let moved = first != self.first;
        self.first = first;
        let m = self.margin();
        let lo = if first > 0 { first + m } else { first };
        let last = (first + fits - 1).min(n - 1);
        let hi = if first + fits < n { last - m } else { last };
        *cursor = (*cursor).clamp(lo, hi.max(lo));
        moved
    }
}

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

/// 논리 크기 `want`를 그릴 때 굽는 크기(논리 px, 정수)와 늘리는 배 — 굽는 크기는 DPI로만 정한다(물리 ≈ `BAKE`px, 0054).
pub fn bake_size(want: f32, dpi: f32) -> (u16, f32) {
    let fs = (BAKE / dpi.max(0.01)).round().max(1.0);
    (fs as u16, want / fs)
}

/// 표의 모든 글 + 기호(글자 굽기).
pub fn all_text(s: &Strings) -> String {
    let mut v = i18n::all(s).concat();
    v.push_str(i18n::UI_GLYPHS);
    // 개발판은 개발 창 글도(0106 — 빠져 있어 개발 창을 여는 프레임에 아틀라스가 커졌다).
    v.push_str(&i18n::dev_texts().concat());
    v
}

#[cfg(test)]
mod tests {
    /// 굴리는 목록(nail 0199 + 가장자리 한 줄 여유): 줄 높이만큼 끌면 한 줄, 끝에서 멈추고, 커서는 창 안(가장자리에서 한 줄 띄워)으로.
    /// 키 커서는 창이 따라가되 다음 줄이 보이게 한 줄 앞서 굴린다.
    #[test]
    fn list_view_scrolls_by_drag_and_keeps_a_row_of_margin() {
        let mut v = ListView::default();
        let mut cur = 0;
        assert_eq!(v.window(cur, 10, 4, 12.0), 0);
        assert_eq!(v.more(10), (false, true));
        assert!(!v.scroll((8.0, 0), &mut cur, 10), "less than a row: stays");
        assert!(v.scroll((8.0, 0), &mut cur, 10));
        assert_eq!(
            (v.first, cur),
            (1, 2),
            "the rest piles up; the cursor comes in one row from the top"
        );
        assert_eq!(
            v.window(cur, 10, 4, 12.0),
            1,
            "and the next draw keeps the window"
        );
        assert!(v.scroll((0.0, 99), &mut cur, 10));
        assert_eq!((v.first, cur), (6, 7), "stops at the end (10 - 4)");
        assert_eq!(v.more(10), (true, false));
        assert!(!v.scroll((0.0, 1), &mut cur, 10));
        assert!(v.scroll((-24.0, 0), &mut cur, 10));
        assert_eq!(v.first, 4);
        // 키로 내려가면 커서가 창 맨 아래 줄에 닿기 전에(한 줄 앞서) 창이 내려간다.
        let mut v = ListView::default();
        assert_eq!(v.window(2, 10, 4, 12.0), 0);
        assert_eq!(v.window(3, 10, 4, 12.0), 1);
        assert_eq!(
            v.window(9, 10, 4, 12.0),
            6,
            "the last row may sit on the edge"
        );
        assert_eq!(v.window(0, 10, 4, 12.0), 0);
        assert_eq!(v.window(9, 3, 4, 12.0), 0, "everything fits: 0");
        // 0076: 손가락으로 가장자리 줄을 고르면 창은 그대로(같은 자리를 다시 누르면 같은 줄), 키로 옮기면 다시 여유를 둔다.
        let mut v = ListView::default();
        assert_eq!(v.window(0, 5, 4, 12.0), 0);
        v.tapped(3);
        assert_eq!(
            v.window(3, 5, 4, 12.0),
            0,
            "a tapped edge row does not scroll"
        );
        assert_eq!(v.window(3, 5, 4, 12.0), 0, "nor on the next frame");
        assert_eq!(v.window(2, 5, 4, 12.0), 0);
        assert_eq!(
            v.window(3, 5, 4, 12.0),
            1,
            "a key move keeps the margin again"
        );
    }

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
        /// 창 틀(0066): 그 안에서 시작한 글이 테두리를 넘으면 알린다.
        frames: Vec<Rect>,
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

    /// 지금까지 모은 글을 잊는다 — 묻는 창이 아래 창을 덮을 때(가려진 글은 겹침이 아니다, 0066).
    pub fn forget() {
        S.with(|s| {
            let mut s = s.borrow_mut();
            s.texts.clear();
            s.frames.clear();
        });
    }

    pub(super) fn frame(r: Rect) {
        if on() {
            S.with(|s| s.borrow_mut().frames.push(r));
        }
    }

    pub fn cut(t: &str, max_w: f32) {
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
        let (texts, frames) = S.with(|s| {
            let mut s = s.borrow_mut();
            (std::mem::take(&mut s.texts), std::mem::take(&mut s.frames))
        });
        for (t, r) in &texts {
            let start = macroquad::prelude::vec2(r.x + 0.5, r.y + 0.5);
            if let Some(f) = frames.iter().find(|f| f.contains(start)) {
                if r.right() > f.right() + 0.5 || r.bottom() > f.bottom() + 0.5 {
                    report(format!(
                        "past-panel x{:.0} y{:.0} w{:.0} {t}",
                        r.x, r.y, r.w
                    ));
                }
            }
        }
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
