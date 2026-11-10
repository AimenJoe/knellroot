//! 지역 사이 지층 단면 전환(0118, v0.5.0 T1 — KB `conveying-descent`, nail 0156·0204 꼴): 검정 대신 땅 단면을 덮고,
//! 가운데에서 지역을 바꾼다(지형 굽기가 그 뒤에 숨는다). 깊이 숫자가 떠난 곳 → 닿을 곳으로 흐른다.
//! 내려가면 띠가 위로 흐르고 방랑자는 머리를 아래로, 올라가면 거꾸로. 렌더는 화면 패스(HUD처럼), 로직은 순수 함수(시험).
use macroquad::prelude::*;

/// 전환 길이(초): 내려가기·올라가기(올라가기가 조금 길다 — nail 0204 "먼 길").
pub const DOWN_TIME: f32 = 1.6;
pub const UP_TIME: f32 = 1.9;
/// 이 진행(0..1)에서 지역을 바꾼다(화면이 다 덮인 가운데).
pub const SWAP_AT: f32 = 0.5;
/// 덮는 데·걷는 데 쓰는 진행 몫.
const COVER: f32 = 0.2;

/// 지역 사이 전환 하나.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Passage {
    /// 닿을 지역(`stage::STAGES` 차례).
    pub to: usize,
    pub up: bool,
    pub from_m: u32,
    pub to_m: u32,
    /// 지난 초.
    pub t: f32,
    /// 지역을 이미 바꿨나.
    pub swapped: bool,
}

impl Passage {
    pub fn new(to: usize, up: bool, from_m: u32, to_m: u32) -> Self {
        Self {
            to,
            up,
            from_m,
            to_m,
            t: 0.0,
            swapped: false,
        }
    }

    pub fn length(&self) -> f32 {
        if self.up {
            UP_TIME
        } else {
            DOWN_TIME
        }
    }

    /// 진행 0..1.
    pub fn progress(&self) -> f32 {
        (self.t / self.length()).clamp(0.0, 1.0)
    }

    /// 화면을 덮는 정도 0..1: 처음에 덮고, 끝에 걷는다.
    pub fn cover(&self) -> f32 {
        let p = self.progress();
        (p / COVER).min((1.0 - p) / COVER).clamp(0.0, 1.0)
    }

    /// 지금 보이는 깊이(m): 덮인 동안 떠난 곳 → 닿을 곳으로(부드럽게).
    pub fn depth(&self) -> f32 {
        let p = ((self.progress() - COVER) / (1.0 - 2.0 * COVER)).clamp(0.0, 1.0);
        let e = p * p * (3.0 - 2.0 * p);
        self.from_m as f32 + (self.to_m as f32 - self.from_m as f32) * e
    }

    /// 한 틱 나아간다. 지역을 바꿀 때면 참(한 번), 다 끝났으면 `done`.
    pub fn step(&mut self, dt: f32) -> (bool, bool) {
        self.t += dt;
        let swap = !self.swapped && self.progress() >= SWAP_AT;
        if swap {
            self.swapped = true;
        }
        (swap, self.progress() >= 1.0)
    }
}

/// 깊이별 흙 색 키(m → 색): 얕을수록 따뜻한 갈색, 깊을수록 남보라(0037 "해 질 녘 남보라" 세계).
const SOIL: [(f32, [u8; 3]); 5] = [
    (0.0, [0x6b, 0x55, 0x4a]),
    (15.0, [0x57, 0x46, 0x47]),
    (35.0, [0x45, 0x3a, 0x4a]),
    (60.0, [0x35, 0x2e, 0x47]),
    (100.0, [0x24, 0x20, 0x38]),
];

fn soil(m: f32) -> Color {
    let mut a = SOIL[0];
    for &b in &SOIL[1..] {
        if m <= b.0 {
            let k = ((m - a.0) / (b.0 - a.0)).clamp(0.0, 1.0);
            let c = |i: usize| (a.1[i] as f32 + (b.1[i] as f32 - a.1[i] as f32) * k) / 255.0;
            return Color::new(c(0), c(1), c(2), 1.0);
        }
        a = b;
    }
    let c = SOIL[SOIL.len() - 1].1;
    Color::from_rgba(c[0], c[1], c[2], 255)
}

fn hash(a: i32, b: i32) -> f32 {
    let h = ((a as f32) * 12.9898 + (b as f32) * 78.233).sin() * 43758.547;
    h - h.floor()
}

/// 단면을 그린다(화면 패스). `f` = 게임 화면, `clock` = 실제 시간(굽는 동안에도 흐른다), `label` = 깊이 글.
pub fn draw(p: &Passage, u: &crate::ui::Ui, f: &crate::render::Frame, clock: f32, label: &str) {
    let (ox, oy, w, h, ui) = (f.ox, f.oy, f.w, f.h, f.ui);
    let a = p.cover();
    if a <= 0.0 {
        return;
    }
    // 화면 한 줄 = 깊이 몇 m: 화면 높이에 약 12m가 보인다. 띠 하나 = 4m.
    let m_per_px = 12.0 / h;
    let top_m = p.depth() - 6.0;
    let band = 4.0;
    let first = (top_m / band).floor() as i32;
    let cols = 24;
    let cw = w / cols as f32;
    for b in first..first + 5 {
        for c in 0..cols {
            // 띠 경계는 기둥마다 물결친다.
            let wave =
                |bb: i32| (hash(bb, c) - 0.5) * 0.9 + ((c as f32 * 0.7 + bb as f32).sin()) * 0.4;
            let y0 = ((b as f32 * band + wave(b)) - top_m) / m_per_px;
            let y1 = (((b + 1) as f32 * band + wave(b + 1)) - top_m) / m_per_px;
            let (y0, y1) = (y0.max(0.0), y1.min(h));
            if y1 <= y0 {
                continue;
            }
            let mut col = soil(b as f32 * band + band * 0.5);
            // 띠마다 한 빛(번갈아 밝고 어둡게) — 기둥마다 바꾸면 세로 줄무늬로 읽혔다(0118 헤드리스 확인).
            let shade = if b.rem_euclid(2) == 0 { 1.0 } else { 0.84 } + hash(b * 7, 3) * 0.06;
            col.r *= shade;
            col.g *= shade;
            col.b *= shade;
            col.a = a;
            draw_rectangle(ox + c as f32 * cw, oy + y0, cw + 1.0, y1 - y0, col);
        }
        // 큰 돌(먼저 — 작은 점만 흩으면 가루로 보인다)과 옆으로 긴 뿌리.
        for k in 0..3 {
            let x = hash(b, k * 31) * w;
            let ym = b as f32 * band + hash(b * 3, k) * band;
            let y = (ym - top_m) / m_per_px;
            if !(-40.0..h + 40.0).contains(&y) {
                continue;
            }
            let r = (10.0 + hash(k, b) * 14.0) * ui * 0.5;
            let mut stone = soil(ym + 20.0);
            stone.a = a * 0.9;
            draw_circle(ox + x, oy + y, r, stone);
            if hash(b, k + 9) > 0.55 {
                let mut root = Color::from_rgba(0x5a, 0x3f, 0x6e, 255);
                root.a = a * 0.8;
                let len = (60.0 + hash(k * 5, b) * 120.0) * ui * 0.5;
                draw_line(
                    ox + x,
                    oy + y + r,
                    ox + x + len,
                    oy + y + r + 6.0 * ui,
                    2.0 * ui,
                    root,
                );
            }
        }
    }
    // 가운데 굴(어두운 기둥)과 방랑자: 내려가면 머리가 아래, 올라가면 위. 부스러기는 위로(내려가면)·아래로(올라가면).
    let cx = ox + w * 0.5;
    let shaft = 26.0 * ui;
    draw_rectangle(
        cx - shaft * 0.5,
        oy,
        shaft,
        h,
        Color::new(0.06, 0.05, 0.09, 0.85 * a),
    );
    let body = Color::new(0.72, 0.62, 0.90, a);
    let cy = oy + h * 0.5;
    let head = if p.up { -1.0 } else { 1.0 } * 7.0 * ui;
    draw_circle(
        cx,
        cy - head * 0.4,
        6.0 * ui,
        Color::new(0.55, 0.85, 0.75, a),
    );
    draw_circle(cx, cy + head, 7.0 * ui, body);
    draw_circle(cx - 4.5 * ui, cy + head * 1.6, 2.5 * ui, body);
    draw_circle(cx + 4.5 * ui, cy + head * 1.6, 2.5 * ui, body);
    let fall = if p.up { 1.0 } else { -1.0 };
    for k in 0..10 {
        let s = hash(k, 77);
        let y = ((s * h + fall * clock * (60.0 + s * 50.0) * ui).rem_euclid(h)) + oy;
        let x = cx + (hash(k, 13) - 0.5) * shaft * 1.6;
        draw_rectangle(
            x,
            y,
            2.0 * ui,
            2.0 * ui,
            Color::new(0.75, 0.62, 0.5, a * 0.8),
        );
    }
    // 깊이 숫자(둥근 그늘 위).
    let k = 1.3;
    let (tw, th) = (u.width(label, k), u.px(k));
    let ty = oy + h * 0.74;
    draw_rectangle(
        cx - tw * 0.5 - 8.0 * ui,
        ty - 5.0 * ui,
        tw + 16.0 * ui,
        th + 10.0 * ui,
        Color::new(0.04, 0.03, 0.07, 0.6 * a),
    );
    u.text_center(label, cx, ty, k, Color::new(1.0, 0.86, 0.6, a));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 0118: 지역은 화면이 다 덮인 가운데에서 한 번 바뀌고, 깊이는 덮인 동안 떠난 곳 → 닿을 곳으로 늘 한쪽으로 흐른다.
    #[test]
    fn the_region_swaps_once_under_full_cover_and_depth_moves_one_way() {
        let mut p = Passage::new(1, false, 0, 40);
        let (mut swaps, mut last, mut done) = (0, p.depth(), false);
        while !done {
            let (swap, d) = p.step(1.0 / 60.0);
            if swap {
                swaps += 1;
                assert!((p.cover() - 1.0).abs() < 1e-3, "covered when swapping");
            }
            assert!(p.depth() >= last - 1e-4, "depth only grows going down");
            last = p.depth();
            done = d;
        }
        assert_eq!(swaps, 1);
        assert_eq!(p.depth(), 40.0);
        assert_eq!(p.cover(), 0.0, "uncovered at the end");
        let up = Passage::new(0, true, 40, 0);
        assert_eq!(up.depth(), 40.0);
        assert!(up.length() > p.length(), "the way up is longer");
    }
}
