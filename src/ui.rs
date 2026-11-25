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
    /// 본문·굵게 — 글꼴마다 아틀라스 여러 장(0254, `glyph_shards`).
    font: Face,
    bold: Face,
    /// 정수 배율(렌더 타깃과 같은 값). 글 크기는 `GLYPH × 배수 × scale`.
    pub scale: f32,
    warmed: (u32, f32),
    /// 눌림(0207, 사장 "버튼 눌리는 감이 없으니"): 0 = 없음, 1 = 막 눌림. 앱이 확인을 잠깐 붙잡는 동안(`press::HOLD`) 고른 줄·단추를
    /// 눌린 꼴로 그린다(면이 그림자 쪽으로 가라앉고 호박색으로 번쩍). 그린 위젯은 `pressed_drawn`을 켠다 — 앱은 아무도 안 그렸으면
    /// 고른 줄의 누를 자리에 덧그린다.
    pub press: f32,
    pub pressed_drawn: std::cell::Cell<bool>,
}

pub const INK: Color = Color::new(0.88, 0.84, 0.76, 1.0);
pub const DIM: Color = Color::new(0.62, 0.58, 0.52, 1.0);
pub const AMBER: Color = Color::new(0.88, 0.6, 0.23, 1.0);
pub const SHADOW: Color = Color::new(0.03, 0.03, 0.04, 0.9);

/// 창 틀(0066, nail `ui::panel`의 꼴과 색): 어두운 면 + 청동 테두리, 고른 줄은 조금 밝은 면.
pub const PANEL: Color = Color::new(0.07, 0.05, 0.09, 0.94);
pub const PANEL_LIGHT: Color = Color::new(0.16, 0.11, 0.14, 1.0);
pub const BORDER: Color = Color::new(0.55, 0.40, 0.28, 1.0);

/// "지금 할 수 있는 것"의 빛(0160 꾸밈의 규칙: 금빛 = 내 것·고른 것 / 민트 = 지금 할 수 있는 것 / 흐림 = 잠김. 빨강은 위험 전용).
pub const MINT: Color = Color::new(0.50, 0.90, 0.74, 1.0);
/// 큰 창(0161)의 머리 줄 높이(480×270 기준 px).
pub const WINDOW_HEAD: f32 = 22.0;

/// 목록 줄의 작은 둘째 줄의 배(`Row::sub`).
pub const LIST_SUB_K: f32 = 0.7;

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
    /// 이름 아래 작은 둘째 줄(0275 — 장비 칸의 후보: 등급 · 낀 칸). 줄 하나라도 있으면 목록의 모든 줄이 두 줄 높이(`list_row_h`).
    pub sub: String,
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
    pub fn with_sub(self, sub: impl Into<String>) -> Self {
        Row {
            sub: sub.into(),
            ..self
        }
    }
}

/// 키캡(nail 0153): 글 속 키·버튼 이름을 이 두 표식 사이에 두면(`key`) 테두리 있는 작은 칸 + 밝은 글로 그린다.
/// 표식은 사용 영역 글자라 폰트에 없고 그리지 않는다. `text`·`width`가 알아보니 가운데 맞춤·`fit`·`wrap`이 그대로 된다.
pub const KEY_OPEN: char = '\u{E000}';
pub const KEY_CLOSE: char = '\u{E001}';
pub const KEY_TEXT: Color = Color::new(1.0, 0.93, 0.78, 1.0);
const KEY_FILL: Color = Color::new(0.08, 0.06, 0.10, 0.85);
/// 테는 밝게, 아랫변은 두껍게(0168 — 사장 "키·버튼 표시는 더 눈에 띄게, 버튼처럼": 눌리는 키의 턱).
const KEY_EDGE: Color = Color::new(0.86, 0.70, 0.46, 1.0);
/// 아랫변 턱의 두께(테 두께의 배).
const KEY_LIP: f32 = 2.0;
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

/// 글 폭의 셈(키캡이면 칸 여백까지): `px` = 글 크기, `plain` = 표식 없는 조각의 폭. 그리는 `Ui::width`와 글꼴만 읽어 재는
/// 시험(0275 `app::text_fit`)이 같은 셈을 쓴다.
pub fn width_with(s: &str, px: f32, plain: impl Fn(&str) -> f32) -> f32 {
    if !s.contains(KEY_OPEN) {
        return plain(s);
    }
    key_segments(s)
        .iter()
        .map(|(seg, is_key)| {
            if *is_key {
                plain(seg) + px * (2.0 * KEY_PAD + KEY_GAP)
            } else {
                plain(seg)
            }
        })
        .sum()
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
        let font = Face::new(include_bytes!("../assets/fonts/GowunBatang-UI-Regular.ttf"));
        let bold = Face::new(include_bytes!("../assets/fonts/GowunBatang-UI-Bold.ttf"));
        Ui {
            font,
            bold,
            scale: 1.0,
            warmed: (0, 0.0),
            press: 0.0,
            pressed_drawn: std::cell::Cell::new(false),
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
        self.font.warm(&chars, (fs as f32 * dpi).ceil() as u16);
    }

    /// 큰 제목(2배 이상)은 굵게.
    fn face(&self, k: f32) -> &Face {
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

    fn plain_w(&self, s: &str, k: f32) -> f32 {
        let (fs, scale) = self.sizing(k);
        self.face(k).width(s, fs, scale)
    }

    /// 글 폭(키캡이면 칸 여백까지).
    pub fn width(&self, s: &str, k: f32) -> f32 {
        width_with(s, self.px(k), |seg| self.plain_w(seg, k))
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
                let line = self.scale.max(1.0);
                let edge = Color {
                    a: color.a,
                    ..KEY_EDGE
                };
                draw_rectangle_lines(cx, ry, w, rh, line, edge);
                // 아랫변의 턱(칸 안쪽으로 — 칸의 크기는 그대로다).
                let lip = line * KEY_LIP;
                draw_rectangle(cx, ry + rh - lip, w, lip, edge);
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

    /// 넘침 검사 없이 그리는 글 — 잘라 그리는 가장자리의 글(굴리는 탭 줄에 걸친 이름)만 쓴다. 보통은 `text`.
    pub fn plain_text(&self, s: &str, x: f32, y: f32, k: f32, color: Color) {
        let base = y + self.px(k) * 0.85;
        let o = self.scale * k;
        let (fs, scale) = self.sizing(k);
        let face = self.face(k);
        face.draw(s, x + o, base + o, fs, scale, SHADOW);
        face.draw(s, x, base, fs, scale, color);
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
            let r = Rect::new(cx - w * 0.5 - px * 1.5, yy - gap * 0.25, w + px * 3.0, gap);
            // 눌린 줄(0207): 글이 한 칸 가라앉고 번쩍.
            let sink = if sel && self.press > 0.0 {
                self.scale
            } else {
                0.0
            };
            self.text_center(it, cx, yy + sink, 1.0, col);
            if sink > 0.0 {
                self.press_overlay(r);
            }
            rows[i] = r;
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

    /// `hints`와 같은 줄을 왼쪽 맞춤으로(큰 창 아래 — 오른쪽은 화면의 단추가 쓴다, 0161).
    pub fn hints_left(&self, pairs: &[(&str, &str)], x: f32, y: f32, max_w: f32) {
        let line = pairs
            .iter()
            .filter(|(k, _)| !k.is_empty())
            .map(|(k, v)| format!("{} {v}", key(k)))
            .collect::<Vec<_>>()
            .join("   ");
        if line.is_empty() {
            return;
        }
        // 좁은 화면(4:3)에서는 글을 자르기 전에 먼저 줄인다 — 안내의 끝이 "…"이면 닫는 법을 못 읽는다.
        let k = (max_w / self.width(&line, 1.0).max(1.0)).clamp(0.6, 1.0);
        let line = self.fit(&line, max_w, k);
        self.text(&line, x, y + (self.px(1.0) - self.px(k)) * 0.5, k, DIM);
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

    /// 테 한 겹(굵기 `t` 화면 px — `draw_rectangle_lines`는 준 굵기의 반만 안쪽에 긋는다).
    pub fn edge(&self, r: Rect, t: f32, c: Color) {
        let t = t.max(1.0).round();
        draw_rectangle(r.x, r.y, r.w, t, c);
        draw_rectangle(r.x, r.y + r.h - t, r.w, t, c);
        draw_rectangle(r.x, r.y + t, t, r.h - t * 2.0, c);
        draw_rectangle(r.x + r.w - t, r.y + t, t, r.h - t * 2.0, c);
    }

    /// 큰 창의 틀(0161 — 0160 "창 하나, 늘 같은 자리·같은 크기": 기술 화면부터, 0164에 모든 탭이 같은 틀에 선다).
    /// 두 겹 테 + 머리 줄(탭들 — 고른 탭은 금빛 테의 칸 · 오른쪽에 지갑 `(이름, 값)`들) + 머리 아래 선. `k` = 좁은 화면에서 줄인 배
    /// (자리·글이 함께 준다). 몸의 자리는 부른 쪽이 `WINDOW_HEAD`로 센다. 닫기 단추는 `close`가 같은 자리에 그린다.
    /// 탭 머리의 자리(누를 자리 — 가려진 탭은 화면 밖)를 돌려준다. 탭이 지갑까지 닿으면 탭의 옆 여백부터 줄이고, 그래도 넘치면
    /// 탭 줄을 굴린다(`window_tabs` · `Strip` — 0233: 끌기·휠, 고른 탭이 바뀌면 보이게. 지갑의 폭이 같으면 탭 자리도 같다).
    /// 탭 = (이름, 표시) — 표시가 참이면 이름 오른쪽 위에 민트 점("지금 할 수 있는 것이 있다").
    pub fn window(
        &self,
        r: Rect,
        k: f32,
        tabs: &[(&str, bool)],
        active: usize,
        purse: &[(&str, String)],
        strip: &mut Strip,
    ) -> Vec<Rect> {
        let u = self.scale * k;
        let line = self.scale.max(1.0).round();
        draw_rectangle(r.x, r.y, r.w, r.h, PANEL);
        #[cfg(feature = "dev")]
        check::frame(r);
        let head = (u * WINDOW_HEAD).round();
        // 머리 줄은 몸보다 한 톤 어둡게(탭이 몸에서 솟아 보이게).
        draw_rectangle(r.x, r.y, r.w, head, Color::new(0.0, 0.0, 0.01, 0.38));
        self.edge(r, line, BORDER);
        let inset = (u * 2.5).round();
        self.edge(
            Rect::new(
                r.x + inset,
                r.y + inset,
                r.w - inset * 2.0,
                r.h - inset * 2.0,
            ),
            1.0,
            Color { a: 0.4, ..BORDER },
        );
        // 지갑: 닫기 단추 왼쪽에 오른쪽 맞춤 — 이름은 본문 색, 값은 금빛.
        let px = self.px(k);
        let ty = r.y + (head - px) * 0.5 + u;
        let mut right = r.x + r.w - self.scale * 28.0;
        for (n, (name, value)) in purse.iter().enumerate().rev() {
            let vw = self.width(value, k);
            self.text(value, right - vw, ty, k, AMBER);
            let lead = format!("{name} ");
            let lw = self.width(&lead, k);
            self.text(&lead, right - vw - lw, ty, k, INK);
            right -= vw + lw;
            if n > 0 {
                let dot = "  ·  ";
                let dw = self.width(dot, k);
                self.text(dot, right - dw, ty, k, DIM);
                right -= dw;
            }
        }
        // 탭: 고른 탭은 몸과 같은 면 + 금빛 테(위·양옆), 머리 아래 선은 그 밑에서 끊긴다.
        // 넘치면 굴린다(0232 → 0233): 글을 줄이지 않고 탭 줄을 가로로 — 손가락으로 끌고(휠도), 고른 탭이 바뀌면 그 탭이 보이게
        // 가장 적게 옮긴다. 양끝 ◀▶는 "더 있다"(가려진 쪽에 할 것 — 민트 점 — 이 있으면 민트), 누르는 단추가 아니다.
        let x0 = r.x + u * 10.0;
        let room = right - u * 8.0 - x0;
        let widths: Vec<f32> = tabs.iter().map(|t| self.width(t.0, 1.0)).collect();
        let (pad, over) = window_tabs(&widths, room, u, k);
        let tpx = self.px(k);
        let tab_y = r.y + (head - tpx) * 0.5 + u;
        let gap_w = u * 2.0;
        let ws: Vec<f32> = widths.iter().map(|w| (w * k + pad).round()).collect();
        let mut at = Vec::with_capacity(tabs.len());
        let mut acc = 0.0;
        for w in &ws {
            at.push(acc);
            acc += w + gap_w;
        }
        let content = (acc - gap_w).max(0.0);
        let arrow = if over {
            self.width("◀", k) + u * 6.0
        } else {
            0.0
        };
        let view = Rect::new(
            x0 + arrow,
            r.y,
            if over { room - 2.0 * arrow } else { room },
            head,
        );
        strip.fit(if over { view.w } else { 0.0 }, content, u * 40.0, view);
        if strip.follow(active) {
            if let (Some(&a0), Some(&w)) = (at.get(active), ws.get(active)) {
                strip.reveal(a0, a0 + w);
            }
        }
        let off = if over { strip.offset() } else { 0.0 };
        let hidden = Rect::new(-1.0e4, -1.0e4, 0.0, 0.0);
        let mut rects = vec![hidden; tabs.len()];
        let mut gap = (view.x, view.x);
        if over {
            clip(Some(view));
        }
        for (i, (t, mark)) in tabs.iter().enumerate() {
            let x = (view.x + at[i] - off).round();
            let w = ws[i];
            if x + w <= view.x || x >= view.x + view.w {
                continue;
            }
            let whole = x >= view.x - 0.5 && x + w <= view.x + view.w + 0.5;
            let top = r.y + u * 4.0;
            if i == active {
                draw_rectangle(x, top, w, head - u * 4.0, PANEL);
                draw_rectangle(
                    x,
                    top,
                    w,
                    head - u * 4.0,
                    Color {
                        a: 0.5,
                        ..PANEL_LIGHT
                    },
                );
                draw_rectangle(x, top, w, line, AMBER);
                draw_rectangle(x, top, line, head - u * 4.0, AMBER);
                draw_rectangle(x + w - line, top, line, head - u * 4.0, AMBER);
                gap = (
                    (x + line).clamp(view.x, view.x + view.w),
                    (x + w - line).clamp(view.x, view.x + view.w),
                );
            }
            let ink = if i == active { AMBER } else { DIM };
            if whole {
                self.text_center(t, x + w * 0.5, tab_y, k, ink);
            } else {
                // 가장자리에 걸친 탭: 잘린 채로 그린다(넘침 검사는 자르기를 모른다 — 걸친 글은 검사 밖으로).
                let tw = self.width(t, k);
                self.plain_text(t, x + (w - tw) * 0.5, tab_y, k, ink);
            }
            if *mark {
                // 민트 점: 글의 오른쪽 위 구석(여백 안), 어두운 한 칸 테로 어느 바탕에서도 보이게.
                let d = (u * 3.0).round().max(2.0);
                let dx = (x + w * 0.5 + widths[i] * k * 0.5 + u).min(x + w - d - line);
                let dy = tab_y - u;
                draw_rectangle(dx - line, dy - line, d + line * 2.0, d + line * 2.0, PANEL);
                draw_rectangle(dx, dy, d, d, MINT);
            }
            let l = x.max(view.x);
            rects[i] = Rect::new(l, r.y, (x + w).min(view.x + view.w) - l, head);
        }
        if over {
            clip(None);
            let (left, right_more) = strip.more();
            let dim = Color { a: 0.35, ..DIM };
            let hid_l = tabs
                .iter()
                .enumerate()
                .any(|(i, t)| t.1 && view.x + at[i] + ws[i] - off <= view.x + 1.0);
            let hid_r = tabs
                .iter()
                .enumerate()
                .any(|(i, t)| t.1 && view.x + at[i] - off >= view.x + view.w - 1.0);
            let ink = |more: bool, mark: bool| {
                if mark {
                    MINT
                } else if more {
                    INK
                } else {
                    dim
                }
            };
            self.text_center("◀", x0 + arrow * 0.5, tab_y, k, ink(left, hid_l));
            self.text_center(
                "▶",
                view.x + view.w + arrow * 0.5,
                tab_y,
                k,
                ink(right_more, hid_r),
            );
        }
        let (l, rr, y) = (r.x + inset, r.x + r.w - inset, r.y + head);
        draw_rectangle(l, y, gap.0 - l, line, BORDER);
        draw_rectangle(gap.1, y, rr - gap.1, line, BORDER);
        rects
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

    /// 눌림의 덧그림(0207): 사각형 `r`을 가라앉히고(위·왼쪽 어두운 턱) 호박색으로 번쩍 — 줄·칸 어디에나.
    pub fn press_overlay(&self, r: Rect) {
        let k = self.press.clamp(0.0, 1.0);
        if k <= 0.0 {
            return;
        }
        let u = self.scale;
        draw_rectangle(
            r.x,
            r.y,
            r.w,
            r.h,
            Color {
                a: 0.28 * k,
                ..AMBER
            },
        );
        draw_rectangle(r.x, r.y, r.w, u * 2.0, Color::new(0.0, 0.0, 0.0, 0.45 * k));
        draw_rectangle(r.x, r.y, u * 2.0, r.h, Color::new(0.0, 0.0, 0.0, 0.45 * k));
        draw_rectangle_lines(
            r.x,
            r.y,
            r.w,
            r.h,
            u.max(1.0),
            Color {
                a: 0.9 * k,
                ..AMBER
            },
        );
        self.pressed_drawn.set(true);
    }

    /// 눌리는 단추(nail `button`, 0157): 오른쪽 아래 그림자 위에 얹힌 면, 위·왼쪽 밝은 모서리. 고르면 면이 밝고 테두리가 호박색.
    /// 단추 탭 줄(0257 — 개발 창·모션 보기): `strip_tabs`로 자리를 정하고 단추로 그린다. 넘치면 창으로 자르고 양끝에 ◀▶("더 있다").
    /// 누를 자리를 돌려준다(보이는 몫만 — 가려진 탭은 화면 밖).
    #[cfg_attr(not(feature = "dev"), allow(dead_code))]
    #[allow(clippy::too_many_arguments)]
    pub fn button_strip(
        &self,
        strip: &mut Strip,
        labels: &[&str],
        active: usize,
        x0: f32,
        x1: f32,
        top: f32,
        h: f32,
    ) -> Vec<Rect> {
        let u = self.scale;
        let widths: Vec<f32> = labels
            .iter()
            .map(|n| self.width(n, 1.0) + u * 14.0)
            .collect();
        let arrow = self.width("◀", 1.0) + u * 6.0;
        let (at, view) = strip_tabs(strip, &widths, x0, x1, top, h, u * 4.0, arrow, active);
        let hidden = Rect::new(-1.0e4, -1.0e4, 0.0, 0.0);
        let mut rects = vec![hidden; labels.len()];
        if let Some(v) = view {
            clip(Some(Rect::new(v.x, v.y - u * 2.0, v.w, v.h + u * 6.0)));
        }
        for (i, (&(x, w), n)) in at.iter().zip(labels).enumerate() {
            let r = Rect::new(x, top, w, h);
            if let Some(v) = view {
                if x + w <= v.x || x >= v.x + v.w {
                    continue;
                }
                // 가장자리에 걸친 단추: 넘침 검사는 자르기를 모른다 — 글 없이 면만(검사 밖으로).
                if x < v.x - 0.5 || x + w > v.x + v.w + 0.5 {
                    self.button(r, "", i == active);
                    let l = x.max(v.x);
                    rects[i] = Rect::new(l, top, (x + w).min(v.x + v.w) - l, h);
                    continue;
                }
            }
            self.button(r, n, i == active);
            rects[i] = r;
        }
        if let Some(v) = view {
            clip(None);
            let (l, r) = strip.more();
            let ink = |more: bool| if more { INK } else { Color { a: 0.35, ..DIM } };
            let ty = top + (h - self.px(1.0)) * 0.5;
            self.text_center("◀", x0 + arrow * 0.5, ty, 1.0, ink(l));
            self.text_center("▶", v.x + v.w + arrow * 0.5, ty, 1.0, ink(r));
        }
        rects
    }

    pub fn button(&self, r: Rect, label: &str, sel: bool) {
        let u = self.scale;
        draw_rectangle(
            r.x + u * 2.0,
            r.y + u * 2.0,
            r.w,
            r.h,
            Color::new(0.02, 0.01, 0.03, 0.9),
        );
        // 눌린 단추(0207): 면이 그림자 자리로 가라앉는다.
        let pressed = sel && self.press > 0.0;
        let r = if pressed {
            Rect::new(r.x + u * 2.0, r.y + u * 2.0, r.w, r.h)
        } else {
            r
        };
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
        if pressed {
            self.press_overlay(r);
        }
    }

    /// 창 안 목록(0066, nail `menu_list`·`setting_row`): `area` 안에 줄을 왼쪽 맞춤으로, 고른 줄은 밝은 면 + ▶.
    /// 값은 오른쪽(`Step`이면 ◀ 값 ▶, `Go`면 →). 넘치면 `menu`처럼 굴린다(▲▼, 창은 `view`).
    pub fn list(&self, rows: &[Row], cursor: usize, view: &mut ListView, area: Rect) -> MenuHits {
        self.list_k(rows, cursor, view, area, 1.0)
    }

    /// `list`를 `k`배로(큰 창의 몸 — 좁은 화면에서 창이 줄면 줄·글도 같은 배로 준다, 0164).
    pub fn list_k(
        &self,
        rows: &[Row],
        cursor: usize,
        view: &mut ListView,
        area: Rect,
        k: f32,
    ) -> MenuHits {
        let px = self.px(k);
        let u = self.scale * k;
        let n = rows.len();
        let row_h = self.list_row_h(rows, k);
        let two = rows.iter().any(|r| !r.sub.is_empty());
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
            // 두 줄 목록(0275): 이름은 위로 비키고 그 아래에 작은 둘째 줄.
            let sk = LIST_SUB_K * k;
            let ty = if two {
                r.y + (row_h - px - self.px(sk) * 1.1) * 0.5
            } else {
                r.y + (row_h - px) * 0.5
            };
            if sel {
                draw_rectangle(r.x, r.y, r.w, r.h, PANEL_LIGHT);
                self.text("▶", r.x + u * 4.0, ty, k, AMBER);
            }
            let fade = |c: Color| if row.dim { Color { a: 0.4, ..c } } else { c };
            let ink = fade(if sel { INK } else { DIM });
            let mark = fade(if sel { AMBER } else { DIM });
            let right = r.x + r.w - u * 6.0;
            let label_x = r.x + u * 4.0 + px * 1.1 + if row.icon { px * 1.4 } else { 0.0 };
            // 값 칸의 왼쪽 끝(이름은 그 앞까지 `fit`).
            let value_left = match row.kind {
                RowKind::Step => {
                    let arrow = self.width("▶", k) + u * 6.0;
                    let vw = self.width(&row.value, k).max(px * 3.0) + u * 6.0;
                    let ra = Rect::new(right - arrow, r.y, arrow, row_h);
                    let la = Rect::new(ra.x - vw - arrow, r.y, arrow, row_h);
                    self.text("◀", la.x + u * 3.0, ty, k, mark);
                    self.text_center(&row.value, la.x + arrow + vw * 0.5, ty, k, ink);
                    self.text("▶", ra.x + u * 3.0, ty, k, mark);
                    hits.steps[i] = Some([la, ra]);
                    la.x
                }
                RowKind::Go => {
                    let w = self.width("→", k);
                    self.text("→", right - w, ty, k, mark);
                    right - w
                }
                RowKind::Plain if row.value.is_empty() => right,
                RowKind::Plain => {
                    let vk = if row.tag { 0.8 * k } else { k };
                    let w = self.width(&row.value, vk);
                    let y = ty + (px - self.px(vk)) * 0.5;
                    self.text(
                        &row.value,
                        right - w,
                        y,
                        vk,
                        if row.tag { mark } else { ink },
                    );
                    right - w
                }
            };
            let label = self.fit(&row.label, value_left - label_x - u * 6.0, k);
            self.text(&label, label_x, ty, k, ink);
            if !row.sub.is_empty() {
                let sub = self.fit(&row.sub, right - label_x, sk);
                self.text(
                    &sub,
                    label_x,
                    ty + px * 1.1,
                    sk,
                    Color {
                        a: ink.a * 0.85,
                        ..DIM
                    },
                );
            }
            if sel && self.press > 0.0 {
                self.press_overlay(r);
            }
            hits.rows[i] = r;
        }
        hits
    }

    /// 목록 한 줄의 높이(화면 px): 둘째 줄(`Row::sub`)이 있는 목록은 두 줄 높이.
    pub fn list_row_h(&self, rows: &[Row], k: f32) -> f32 {
        let px = self.px(k);
        if rows.iter().any(|r| !r.sub.is_empty()) {
            px * 1.75 + self.px(LIST_SUB_K * k) * 1.1
        } else {
            px * 1.75
        }
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

    /// 목록 줄 `r`(`list_k`가 돌려준 자리)의 그림 칸 — `Row::icon` 줄의 이름 앞, 정사각 한 글줄 높이. `k` = 목록의 배.
    pub fn list_icon(&self, r: Rect, k: f32) -> Rect {
        let px = self.px(k);
        let side = px * 1.25;
        Rect::new(
            r.x + self.scale * k * 4.0 + px * 1.1,
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
    /// 반짝이는 별 ✦(0209 — 튜토리얼이 창 안에서 가리키는 곳): 네 끝 별이 숨 쉬듯 커졌다 작아지고, 가운데 흰 점.
    /// `r` = 화면 px 반지름. 어두운 테를 먼저 그려 어느 바탕에서도 보이게.
    pub fn sparkle(&self, c: Vec2, r: f32) {
        let t = get_time() as f32;
        let p = 0.5 + 0.5 * (t * 4.2).sin();
        let r = r * (0.8 + 0.35 * p);
        let star = |r: f32, w: f32, col: Color| {
            for (dx, dy) in [(1.0f32, 0.0f32), (0.0, 1.0)] {
                let (a, b) = (vec2(dx, dy) * r, vec2(-dy, dx) * w);
                draw_triangle(c + a, c + b, c - b, col);
                draw_triangle(c - a, c + b, c - b, col);
            }
        };
        star(r + 1.5, r * 0.32 + 1.5, Color::new(0.08, 0.06, 0.05, 0.6));
        star(r, r * 0.3, Color::new(1.0, 0.86, 0.45, 0.75 + 0.25 * p));
        draw_circle(
            c.x,
            c.y,
            (r * 0.22).max(1.0),
            Color::new(1.0, 1.0, 0.95, 0.9),
        );
    }

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
    /// 줄 표가 아닌 판(기술 화면의 칸 격자, 0163)의 창: 보이는 첫 자리 `off`, 지난번 그릴 때의 보이는 높이·내용 높이(판의 기준 px)와
    /// 기준 px 하나의 화면 px. 한 화면은 줄 창(`first`)과 판 창 가운데 하나만 쓴다.
    off: f32,
    room: f32,
    content: f32,
    unit: f32,
}

/// 화면 패스의 그림을 `r`(논리 px) 안으로 자른다 — `None`이면 푼다. 굴리는 판의 위·아래 끝에 걸친 칸을 자를 때(0163).
/// 풀기를 잊으면 그 뒤의 모든 그림이 잘린다: 자른 함수 안에서 푼다.
pub fn clip(r: Option<Rect>) {
    let dpi = screen_dpi_scale();
    let px = r.map(|r| {
        (
            (r.x * dpi).round() as i32,
            (r.y * dpi).round() as i32,
            (r.w * dpi).round() as i32,
            (r.h * dpi).round() as i32,
        )
    });
    unsafe { get_internal_gl() }.quad_gl.scissor(px);
}

impl ListView {
    /// 판 창(0163): 그릴 때 보이는 높이 `room`·내용 높이 `content`(기준 px)·기준 px의 화면 크기 `unit`을 적고, 보이는 첫 자리를
    /// 돌려준다(내용이 다 들어가면 0). 처음 그리는 창이면 참도 함께 — 부르는 쪽이 커서 자리를 `reveal`로 보이게 한다.
    pub fn span(&mut self, room: f32, content: f32, unit: f32) -> (f32, bool) {
        let fresh = self.room <= 0.0;
        (self.room, self.content, self.unit) = (room, content, unit);
        self.off = self.off.clamp(0.0, self.span_max());
        (self.off, fresh)
    }

    fn span_max(&self) -> f32 {
        (self.content - self.room).max(0.0)
    }

    /// 지금 보이는 첫 자리(기준 px).
    pub fn offset(&self) -> f32 {
        self.off
    }

    /// 내용의 `y0..y1`이 보이게 창을 가장 적게 옮긴다(커서가 창을 끈다). 창보다 큰 구간이면 위쪽을 보인다. 움직인 뒤의 자리.
    pub fn reveal(&mut self, y0: f32, y1: f32) -> f32 {
        if self.room > 0.0 {
            let low = (y1 - self.room).min(y0);
            self.off = self.off.clamp(low, y0.max(low)).clamp(0.0, self.span_max());
        }
        self.off
    }

    /// 끌기(화면 px, 위로 끌면 +)와 칸(휠·▲▼, 아래로 + — 한 칸 = `step` 기준 px)만큼 판 창을 옮긴다. 움직였으면 참.
    pub fn nudge(&mut self, drag: f32, steps: i32, step: f32) -> bool {
        let before = self.off;
        let d = drag / self.unit.max(0.01) + steps as f32 * step;
        self.off = (self.off + d).clamp(0.0, self.span_max());
        self.off != before
    }

    /// 판 창의 위·아래에 가려진 것이 있나(▲▼).
    pub fn span_more(&self) -> (bool, bool) {
        (self.off > 0.5, self.off < self.span_max() - 0.5)
    }

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

/// 큰 창의 탭 줄(0164 → 0233): 글 폭들(`widths` — 배율 1의 화면 px)을 `room`(화면 px) 안에. 돌려주는 것 = (탭 하나의 옆 여백 합,
/// 넘치나). 넉넉하면 여백 18·글은 창의 배 `k` 그대로. 모자라면 여백을 6까지 줄이고, 그래도 넘치면 **글을 줄이지 않는다** —
/// 탭 줄을 굴린다(`Strip`, 여백 10). 글을 줄이거나 자르면 무슨 탭인지 모른다(nail 0247 — 일시정지 탭 여덟에서 "メ…").
/// `u` = 기준 px 하나의 화면 크기(탭 사이 틈 2).
/// 창 안의 작은 탭 줄(개발 창의 묶음·모션 보기의 무기·도감의 묶음, 0256 → 0257): 다 들면 남는 폭을 고르게 나눠 `x0..x1`을 꽉 채우고,
/// 넘치면 큰 창의 탭 줄(0233)처럼 **가로로 굴린다** — 양끝에 ◀▶ 자리(`arrow` 폭)를 비우고 `strip`의 창으로 보며, 고른 탭이 바뀌면
/// 그 탭이 보이게 가장 적게 옮긴다(사장 2026-11-19 "좌우 스크롤되게 할 수 있으면 그게 더 좋은 거지" — 0256의 여러 줄을 걷음).
/// `widths` = 탭마다 폭(여백 포함). 돌려주는 것 = (탭마다 화면의 왼쪽 x · 폭, 보이는 창 — 넘칠 때만 `Some`).
/// 끌기·휠은 부른 쪽이 `strip.nudge`로(그 창 위에서만).
#[allow(clippy::too_many_arguments)]
pub fn strip_tabs(
    strip: &mut Strip,
    widths: &[f32],
    x0: f32,
    x1: f32,
    y: f32,
    h: f32,
    gap: f32,
    arrow: f32,
    active: usize,
) -> (Vec<(f32, f32)>, Option<Rect>) {
    let room = (x1 - x0).max(1.0);
    let n = widths.len().max(1) as f32;
    let sum: f32 = widths.iter().sum::<f32>() + gap * (n - 1.0);
    if sum <= room {
        strip.fit(0.0, sum, 0.0, Rect::new(x0, y, room, h));
        let extra = (room - sum) / n;
        let mut x = x0;
        let out = widths
            .iter()
            .map(|&w| {
                let at = (x, w + extra);
                x += w + extra + gap;
                at
            })
            .collect();
        return (out, None);
    }
    let view = Rect::new(x0 + arrow, y, (room - 2.0 * arrow).max(1.0), h);
    let mut at = Vec::with_capacity(widths.len());
    let mut acc = 0.0;
    for &w in widths {
        at.push(acc);
        acc += w + gap;
    }
    strip.fit(
        view.w,
        sum,
        widths.iter().copied().fold(0.0, f32::max).max(1.0),
        view,
    );
    if strip.follow(active) {
        if let (Some(&a0), Some(&w)) = (at.get(active), widths.get(active)) {
            strip.reveal(a0, a0 + w);
        }
    }
    let off = strip.offset();
    let out = at
        .iter()
        .zip(widths)
        .map(|(&a0, &w)| ((view.x + a0 - off).round(), w))
        .collect();
    (out, Some(view))
}

pub fn window_tabs(widths: &[f32], room: f32, u: f32, k: f32) -> (f32, bool) {
    let n = widths.len().max(1) as f32;
    let text: f32 = widths.iter().sum::<f32>() * k;
    let gaps = (n - 1.0) * u * 2.0;
    let pad = ((room - text - gaps) / n).clamp(u * 6.0, u * 18.0);
    if widths.is_empty() || text + gaps + pad * n <= room {
        return (pad, false);
    }
    (u * 10.0, true)
}

/// 넘치는 탭 줄의 가로 창(0233 — 사장 "터치스크린에서는 스크롤되어야지, 탭으로 옮기는 건 이상할 듯"): 보이는 첫 자리 `off`
/// (화면 px) · 지난번 그릴 때의 보이는 폭·내용 폭·휠 한 칸의 폭 · 탭 줄의 자리(끌기·휠이 여기서 일어나면 굴린다) · 마지막으로
/// 보인 고른 탭(바뀌면 그 탭이 보이게 `reveal` — 손으로 굴린 창을 그대로 두다가, 탭을 넘기면 따라온다). 큰 창 하나에 하나.
#[derive(Clone, Copy, Debug, Default)]
pub struct Strip {
    off: f32,
    room: f32,
    content: f32,
    step: f32,
    rect: Rect,
    shown: Option<usize>,
}

impl Strip {
    /// 그릴 때: 보이는 폭(넘치지 않으면 0 — 굴리지 않는다)·내용 폭·휠 한 칸·탭 줄의 자리.
    pub fn fit(&mut self, room: f32, content: f32, step: f32, rect: Rect) {
        (self.room, self.content, self.step, self.rect) = (room, content, step, rect);
        self.off = self.off.clamp(0.0, self.max());
    }

    fn max(&self) -> f32 {
        if self.room <= 0.0 {
            return 0.0;
        }
        (self.content - self.room).max(0.0)
    }

    pub fn offset(&self) -> f32 {
        self.off
    }

    /// 고른 탭이 지난번과 다른가(다르면 기억한다 — 부른 쪽이 그 탭을 `reveal`).
    pub fn follow(&mut self, active: usize) -> bool {
        let changed = self.shown != Some(active);
        self.shown = Some(active);
        changed
    }

    /// 내용의 `x0..x1`이 보이게 창을 가장 적게 옮긴다.
    pub fn reveal(&mut self, x0: f32, x1: f32) {
        if self.room > 0.0 {
            let low = (x1 - self.room).min(x0);
            self.off = self.off.clamp(low, x0.max(low)).clamp(0.0, self.max());
        }
    }

    /// 끌기(화면 px, 왼쪽으로 끌면 +)·휠(칸)만큼 옮긴다 — `at`(끌기가 시작된 자리·마우스 자리)이 탭 줄 위일 때만. 옮겼으면 참.
    pub fn nudge(&mut self, at: Option<Vec2>, drag: f32, steps: i32) -> bool {
        if self.room <= 0.0 || !at.is_some_and(|p| self.rect.contains(p)) {
            return false;
        }
        let before = self.off;
        self.off = (self.off + drag + steps as f32 * self.step).clamp(0.0, self.max());
        self.off != before
    }

    /// 탭 줄 위인가(넘쳐서 굴리는 동안만 — 끌기·휠을 몸의 목록에 넘기지 않으려고).
    pub fn over(&self, at: Option<Vec2>) -> bool {
        self.room > 0.0 && at.is_some_and(|p| self.rect.contains(p))
    }

    /// 왼쪽·오른쪽에 가려진 것이 있나(◀▶).
    pub fn more(&self) -> (bool, bool) {
        (self.off > 0.5, self.off < self.max() - 0.5)
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

/// 글꼴 하나 = 같은 글꼴을 읽은 `Font` 여러 장(0254): 장마다 아틀라스 하나가 1024² 안에 머물게 글자를 나눠 담는다
/// (`glyph_shards::Router`). 글은 장마다의 토막으로 그리고 잰다 — macroquad는 글자마다 advance로 나아가니 토막의 폭을 더하면
/// 한 번에 그린 것과 같다(장 사이의 커닝만 빠진다).
struct Face {
    bytes: &'static [u8],
    shards: std::cell::RefCell<Vec<Font>>,
    router: std::cell::RefCell<crate::glyph_shards::Router>,
}

impl Face {
    fn new(bytes: &'static [u8]) -> Self {
        let f = Face {
            bytes,
            shards: Default::default(),
            router: Default::default(),
        };
        f.grow(1);
        f
    }

    /// 장이 `n`개가 되게 읽는다.
    fn grow(&self, n: usize) {
        let mut v = self.shards.borrow_mut();
        while v.len() < n {
            let mut f = load_ttf_font_from_bytes(self.bytes).expect("UI font");
            // 명조는 픽셀 격자가 아니라 부드럽게(고밀도 화면의 소수 배율에서도 매끈하게).
            f.set_filter(FilterMode::Linear);
            v.push(f);
        }
    }

    /// 미리 굽기 — 글자를 장에 나눠 그 장에 굽는다.
    fn warm(&self, chars: &[char], size: u16) {
        let parts = self.router.borrow_mut().split(chars);
        self.grow(parts.len());
        let v = self.shards.borrow();
        for (i, part) in parts.iter().enumerate() {
            v[i].populate_font_cache(part, size);
        }
    }

    fn runs(&self, s: &str) -> Vec<(usize, std::ops::Range<usize>)> {
        let runs = self.router.borrow_mut().runs(s);
        self.grow(self.router.borrow().shards());
        runs
    }

    fn width(&self, s: &str, fs: u16, scale: f32) -> f32 {
        let runs = self.runs(s);
        let v = self.shards.borrow();
        runs.into_iter()
            .map(|(i, r)| measure_text(&s[r], Some(&v[i]), fs, scale).width)
            .sum()
    }

    fn draw(&self, s: &str, x: f32, y: f32, fs: u16, scale: f32, color: Color) {
        let runs = self.runs(s);
        let v = self.shards.borrow();
        let mut cx = x;
        for (i, r) in runs {
            let p = TextParams {
                font: Some(&v[i]),
                font_size: fs,
                font_scale: scale,
                color,
                ..Default::default()
            };
            cx += draw_text_ex(&s[r], cx, y, p).width;
        }
    }
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
    /// 0164 → 0233: 큰 창의 탭 줄 — 넉넉하면 여백 18, 모자라면 여백부터(6까지), 그래도 넘치면 글을 줄이지 않고 굴린다(여백 10).
    #[test]
    fn small_tab_strips_fill_when_they_fit_and_scroll_when_they_do_not() {
        use super::{strip_tabs, Strip};
        use macroquad::prelude::vec2;
        // 다 든다: 남는 폭을 나눠 꽉 채우고 굴리지 않는다.
        let mut st = Strip::default();
        let (at, view) = strip_tabs(&mut st, &[40.0, 60.0], 0.0, 200.0, 0.0, 12.0, 4.0, 10.0, 0);
        assert!(view.is_none());
        assert!((at[1].0 + at[1].1 - 200.0).abs() < 0.01);
        assert!(!st.over(Some(vec2(50.0, 5.0))));
        // 넘친다: 한 줄 그대로(줄을 늘리지 않는다), 양끝 ◀▶ 자리 안이 창 — 마지막 탭을 고르면 그 탭이 창 안으로.
        let w = [70.0; 7];
        let (at, view) = strip_tabs(&mut st, &w, 0.0, 200.0, 0.0, 12.0, 4.0, 10.0, 0);
        let v = view.expect("scrolls");
        assert!(at.iter().all(|x| x.0.is_finite()));
        assert_eq!((v.x, v.w), (10.0, 180.0));
        assert!(st.over(Some(vec2(50.0, 5.0))));
        let (at, _) = strip_tabs(&mut st, &w, 0.0, 200.0, 0.0, 12.0, 4.0, 10.0, 6);
        assert!(
            at[6].0 >= v.x - 0.5 && at[6].0 + at[6].1 <= v.x + v.w + 0.5,
            "{:?}",
            at[6]
        );
        // 끌기로 처음으로 돌아간다.
        assert!(st.nudge(Some(vec2(50.0, 5.0)), -1000.0, 0));
        let (at, _) = strip_tabs(&mut st, &w, 0.0, 200.0, 0.0, 12.0, 4.0, 10.0, 6);
        assert_eq!(at[0].0, v.x);
    }

    #[test]
    fn window_tabs_give_up_padding_then_roll_instead_of_shrinking_text() {
        use super::window_tabs;
        let w = [26.0; 6];
        // 넉넉: 6 × (26 + 18) + 5 × 2 = 274.
        assert_eq!(window_tabs(&w, 400.0, 1.0, 1.0), (18.0, false));
        // 조금 모자람: 여백만 준다 — 합이 꼭 방에 든다.
        let (pad, over) = window_tabs(&w, 250.0, 1.0, 1.0);
        assert!(!over && (6.0..18.0).contains(&pad));
        assert!((156.0 + 10.0 + pad * 6.0 - 250.0).abs() < 0.01);
        // 많이 모자람: 굴린다 — 글의 배는 그대로(돌려주지도 않는다).
        assert_eq!(window_tabs(&w, 190.0, 1.0, 1.0), (10.0, true));
        // 화면 배율·좁은 창의 배가 같이 곱해진다.
        assert_eq!(window_tabs(&w, 2000.0, 3.0, 0.8), (54.0, false));
    }

    /// 0233: 넘치는 탭 줄의 창 — 끌기·휠은 탭 줄 위에서만, 끝에서 멈춘다. 고른 탭이 바뀌면 그 탭이 보이게 가장 적게 옮기고,
    /// 같은 탭이면 손으로 굴린 자리를 그대로 둔다. 넘치지 않으면 굴리지 않는다.
    #[test]
    fn a_tab_strip_scrolls_by_finger_and_follows_a_new_tab() {
        use super::Strip;
        use macroquad::prelude::{vec2, Rect};
        let head = Rect::new(10.0, 0.0, 200.0, 20.0);
        let on = Some(vec2(50.0, 10.0));
        let mut s = Strip::default();
        s.fit(200.0, 500.0, 40.0, head);
        assert!(s.follow(0));
        s.reveal(0.0, 60.0);
        assert_eq!(s.offset(), 0.0);
        assert_eq!(s.more(), (false, true));
        // 몸 위의 끌기는 탭 줄을 굴리지 않는다.
        assert!(!s.nudge(Some(vec2(50.0, 100.0)), 80.0, 0));
        assert!(!s.over(Some(vec2(50.0, 100.0))) && s.over(on));
        // 왼쪽으로 80 끌면 80, 휠 두 칸이면 +80, 끝(300)에서 멈춘다.
        assert!(s.nudge(on, 80.0, 0));
        assert_eq!(s.offset(), 80.0);
        assert!(s.nudge(on, 0.0, 2));
        assert_eq!(s.offset(), 160.0);
        s.nudge(on, 999.0, 0);
        assert_eq!(s.offset(), 300.0);
        assert_eq!(s.more(), (true, false));
        // 같은 탭이면 그대로(손으로 굴린 자리를 지킨다).
        assert!(!s.follow(0));
        // 다른 탭으로 넘기면 그 탭(120..180)이 보이게 가장 적게: 왼쪽 끝이 120.
        assert!(s.follow(2));
        s.reveal(120.0, 180.0);
        assert_eq!(s.offset(), 120.0);
        // 이미 보이는 탭이면 움직이지 않는다.
        assert!(s.follow(3));
        s.reveal(200.0, 260.0);
        assert_eq!(s.offset(), 120.0);
        // 넘치지 않으면(보이는 폭 0) 굴리지 않는다.
        s.fit(0.0, 500.0, 40.0, head);
        assert_eq!(s.offset(), 0.0);
        assert!(!s.nudge(on, 80.0, 0) && !s.over(on));
    }

    /// 굴리는 목록(nail 0199 + 가장자리 한 줄 여유): 줄 높이만큼 끌면 한 줄, 끝에서 멈추고, 커서는 창 안(가장자리에서 한 줄 띄워)으로.
    /// 키 커서는 창이 따라가되 다음 줄이 보이게 한 줄 앞서 굴린다.
    /// 0163: 판 창(줄 표가 아닌 격자) — 내용이 다 들어가면 움직이지 않고, 넘치면 끌기(화면 px)·칸(휠·▲▼)으로 0..=넘친 만큼,
    /// `reveal`은 구간이 보이게 가장 적게 옮긴다. 그리기 전(보이는 높이를 모를 때)에는 움직이지 않는다.
    #[test]
    fn a_span_view_scrolls_only_what_overflows_and_reveals_with_the_least_move() {
        let mut v = ListView::default();
        assert_eq!(v.reveal(150.0, 190.0), 0.0, "not drawn yet");
        let (off, fresh) = v.span(100.0, 80.0, 2.0);
        assert!(fresh && off == 0.0);
        assert!(!v.nudge(40.0, 1, 24.0), "everything fits");
        assert_eq!(v.span_more(), (false, false));
        let (_, fresh) = v.span(100.0, 160.0, 2.0);
        assert!(!fresh);
        assert_eq!(v.span_more(), (false, true));
        // 끌기 40 화면 px = 20 기준 px, 휠 한 칸 = 24.
        assert!(v.nudge(40.0, 0, 24.0));
        assert_eq!(v.offset(), 20.0);
        assert!(v.nudge(0.0, 1, 24.0));
        assert_eq!(v.offset(), 44.0);
        assert_eq!(v.span_more(), (true, true));
        v.nudge(0.0, 5, 24.0);
        assert_eq!(v.offset(), 60.0, "stops at the end");
        assert_eq!(v.span_more(), (true, false));
        // 보이는 구간(60..160) 안이면 그대로, 위로 벗어나면 그 위가 창의 위 끝, 아래로 벗어나면 그 아래가 창의 아래 끝.
        assert_eq!(v.reveal(70.0, 120.0), 60.0);
        assert_eq!(v.reveal(30.0, 80.0), 30.0);
        assert_eq!(v.reveal(100.0, 150.0), 50.0);
        // 창보다 큰 구간은 위쪽을 보인다.
        assert_eq!(v.reveal(10.0, 150.0), 10.0);
        // 내용이 줄면(창이 커지면) 자리도 따라 줄어든다.
        assert_eq!(v.span(100.0, 120.0, 2.0).0, 10.0);
        v.nudge(0.0, 3, 24.0);
        assert_eq!(v.span(130.0, 120.0, 2.0).0, 0.0);
    }

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

    pub fn frame(r: Rect) {
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
