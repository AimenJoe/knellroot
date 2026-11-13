//! 싸움 화면의 글·막대(0019). 그린 화면(`render::Frame`) 안에 같은 정수 배율로 붙인다. 월드·스테이지를 읽기만 한다.
//!
//! 자리: 왼쪽 위 체력(막대, 0158 N0)·스태미나 · 오른쪽 위 목표 · 가운데 위 구역 카드 · 아래 가운데 보스 띠 · 가운데 시작 이야기.
use macroquad::prelude::*;

use crate::i18n::{self, tr};
use crate::render::Frame;
use crate::stage::{StageRun, CARD_TIME, INTRO_TIME};
use crate::tuning::{idx, CAST_CHARGES, P, SPECS};
use crate::tutorial::Hint;
use crate::ui::{Ui, AMBER, DIM, INK};
use crate::weapons::Rb;
use crate::world::World;

const HP_ON: Color = Color::new(0.88, 0.6, 0.23, 1.0);
const HP_OFF: Color = Color::new(0.2, 0.18, 0.17, 0.9);
const STAMINA: Color = Color::new(0.55, 0.78, 0.5, 1.0);
const DENIED: Color = Color::new(1.0, 0.3, 0.25, 1.0);
/// 등잔(0031): 놋쇠 몸 + 불.
const FLASK_ON: Color = Color::new(0.62, 0.5, 0.32, 1.0);
const FLASK_FLAME: Color = Color::new(1.0, 0.8, 0.35, 1.0);
/// 마법 칸(0048): 라벤더(탄과 같은 색). 쇠뇌 탄창·수류탄(0051): 밝은 나무.
const MANA: Color = Color::new(0.75, 0.66, 1.0, 1.0);
/// 큰 기술 게이지(0151): 차는 동안은 가라앉은 금빛, 가득 차면 밝게 숨 쉰다.
const GAUGE: Color = Color::new(0.78, 0.62, 0.3, 1.0);
/// 이어지는 것의 남은 시간(0158 G2 — 몰아치기의 줄어드는 막대, 스킬 칸 아래의 띠): 게이지보다 밝은 금빛.
const GAUGE_FULL: Color = Color::new(1.0, 0.9, 0.55, 1.0);
const AMMO: Color = Color::new(0.95, 0.82, 0.55, 1.0);
const BOSS: Color = Color::new(0.72, 0.62, 0.42, 1.0);
const BOSS_ROUSED: Color = Color::new(0.95, 0.55, 0.2, 1.0);
/// 자세 막대(0055): 밝은 크림, 깨지면 금빛.
const POISE: Color = Color::new(0.92, 0.88, 0.78, 0.9);
const POISE_BROKEN: Color = Color::new(1.0, 0.82, 0.3, 1.0);
/// 잔불 수(0064 M1)와 주웠을 때 커지는 시간(초).
const EMBER: Color = Color::new(1.0, 0.74, 0.32, 1.0);
const EMBER_POP: f32 = 0.35;

const BAR_BG: Color = Color::new(0.05, 0.05, 0.06, 0.85);

/// 0..1 → 들어올 때·나갈 때 `edge`초씩 부드럽게(카드·이야기).
fn fade_in_out(left: f32, total: f32, edge: f32) -> f32 {
    let shown = total - left;
    (shown / edge).min(left / edge).clamp(0.0, 1.0)
}

/// UI px 하나(480×270 기준 UI 배율, 0041).
fn u(f: &Frame) -> f32 {
    f.ui
}

/// `under_menu`: 메뉴(멈춤)가 위에 뜬다 — 아래 가운데 보스 띠는 메뉴 안내와 겹쳐 그리지 않는다(KB hud-readability: 창이 덮는 HUD는 뺀다).
/// 갈 곳이 화면 밖일 때 가장자리 화살(0060 W1, nail 0159 출구 표시와 같은 장치): `at` 화면 자리, `ang` 화면 방향(라디안, y 아래),
/// 삼각형이 바깥을 가리키고 안쪽에 작은 마름모(갈 곳), 어두운 테두리로 어느 바닥 위에서도, 맥박. 색만이 아니라 모양으로 읽힌다.
/// 빛을 받는다: 호박색 = 갈 곳(0060 — 그 뜻은 이 색 하나가 끝까지 지닌다), 잿빛 `FOE_INK` = "저쪽에 적이 있다"(0170 — 튜토리얼이
/// 벨 상대를 가리킬 때. 갈 길의 색을 쓰지 않는다).
pub fn draw_edge_arrow(f: &Frame, at: Vec2, ang: f32, time: f32, ink: Color) {
    let s = f.ui * (7.0 + 1.0 * (time * 4.0).sin());
    let d = vec2(ang.cos(), ang.sin());
    let n = vec2(-d.y, d.x);
    // 길쭉한 화살촉(길이 2 : 폭 1.1) — 작은 크기에서도 어느 쪽인지 읽히게.
    let tip = at + d * s * 1.2;
    let (l, r) = (
        at - d * s * 0.8 + n * s * 0.55,
        at - d * s * 0.8 - n * s * 0.55,
    );
    let shade = Color::new(0.05, 0.04, 0.06, 0.8);
    let grow = |p: Vec2| at + (p - at) * 1.35;
    draw_triangle(grow(tip), grow(l), grow(r), shade);
    draw_triangle(tip, l, r, ink);
    let c = at - d * s * 1.9;
    let k = s * 0.45;
    draw_poly(c.x, c.y, 4, k + f.ui * 1.2, 0.0, shade);
    draw_poly(c.x, c.y, 4, k, 0.0, ink);
}

/// 깨어나는 첫 장면의 덮개(0171 — HUD 대신): 눈꺼풀(위아래에서 닫힌 어둠 — 가장자리는 부드럽게) + 덜 뜬 눈의 흐린 어둠 +
/// 장면 동안 남는 위아래 띠 + 아래쪽 서술 두 줄(방랑자의 몸을 가리지 않게 가운데가 아니라 아래 3분의 1) + 구석의 건너뛰기 안내.
/// 시간표는 `wake::Wake`가 낸다. `skip` = (버튼 이름 — 터치는 빈 글, 글).
pub fn draw_wake(ui: &Ui, f: &Frame, w: &crate::wake::Wake, lines: [&str; 2], skip: (&str, &str)) {
    let s = u(f);
    let (open, bars) = (w.lids(), w.bars() * crate::wake::BARS);
    // 덜 뜬 눈은 어둡다 — 열릴수록 걷힌다.
    let veil = (1.0 - open).powi(2) * 0.85;
    if veil > 0.003 {
        draw_rectangle(f.ox, f.oy, f.w, f.h, Color::new(0.0, 0.0, 0.0, veil));
    }
    // 눈꺼풀: 다 뜬 뒤에도 띠만큼은 남는다(장면의 표지). 가장자리 `soft`는 여러 겹으로 옅어진다.
    let lid = ((1.0 - open) * 0.5).max(bars) * f.h;
    let soft = if open < 1.0 { 14.0 * s } else { 0.0 };
    for (y, dir) in [(f.oy, 1.0), (f.oy + f.h, -1.0)] {
        let (top, h) = if dir > 0.0 { (y, lid) } else { (y - lid, lid) };
        draw_rectangle(f.ox, top, f.w, h, BLACK);
        const STEPS: usize = 7;
        for i in 0..STEPS {
            let k = 1.0 - (i as f32 + 0.5) / STEPS as f32;
            let band = soft / STEPS as f32;
            let y0 = y + dir * (lid + band * i as f32);
            let top = if dir > 0.0 { y0 } else { y0 - band };
            draw_rectangle(f.ox, top, f.w, band, Color::new(0.0, 0.0, 0.0, k * k * 0.9));
        }
    }
    // 서술 두 줄(시작 글과 같은 빛: 첫 줄 본문 · 둘째 줄 흐리게).
    let cx = f.ox + f.w * 0.5;
    let mut y = f.oy + f.h * WAKE_TEXT_Y;
    for (i, (line, ink)) in lines.into_iter().zip([INK, DIM]).enumerate() {
        let a = w.line(i);
        let col = Color::new(ink.r, ink.g, ink.b, a);
        let y2 = if a > 0.004 {
            ui.text_center_wrapped(line, cx, y, WAKE_TEXT_K, col, f.w * 0.84)
        } else {
            y + ui.px(WAKE_TEXT_K)
        };
        y = y2 + ui.px(WAKE_TEXT_K) * 0.35;
    }
    // 건너뛰기: 눈이 뜨인 뒤 오른쪽 아래 띠 안에 작게(터치는 왼쪽 위 멈춤 버튼이 같은 일을 한다 — 글만).
    let a = (open * w.bars()).min(1.0) * 0.75;
    if a > 0.05 && !skip.1.is_empty() {
        let k = 0.75;
        let text = if skip.0.is_empty() {
            skip.1.to_string()
        } else {
            format!("{} {}", crate::ui::key(skip.0), skip.1)
        };
        let tw = ui.width(&text, k);
        let y = f.oy + f.h - (bars * f.h + ui.px(k)) * 0.5 - s;
        ui.text(
            &text,
            f.ox + f.w - tw - 8.0 * s,
            y.min(f.oy + f.h - ui.px(k) - 3.0 * s),
            k,
            Color::new(DIM.r, DIM.g, DIM.b, a),
        );
    }
}

/// 깨어나는 장면의 서술 글: 자리(화면 높이의 몫)·크기.
const WAKE_TEXT_Y: f32 = 0.68;
const WAKE_TEXT_K: f32 = 1.1;

/// 종소리의 물결(0171): 종이 운 뒤 `k` 0..1 동안. 소리가 화면 밖에서 오면(`out` = 바깥을 향한 화면 방향, 라디안·y 아래)
/// 가장자리 `at`에서 화면 안쪽으로 번져 오는 호 셋, 화면 안이면 그 자리에서 퍼지는 고리 둘. 화살이 아니다 — 가리키지 않고
/// 번진다. 빛은 종의 쇠(창백한 푸른 흰빛): 갈 곳의 호박색·적 쪽의 잿빛과 갈린다. 어두운 밑줄로 어느 바닥 위에서도.
pub fn draw_knell(f: &Frame, at: Vec2, out: Option<f32>, k: f32) {
    let s = f.ui;
    let shade = |a: f32| Color::new(0.04, 0.04, 0.07, 0.55 * a);
    let ink = |a: f32| Color::new(KNELL_INK.r, KNELL_INK.g, KNELL_INK.b, a);
    // 호 하나: 가운데 `c`·반지름 `r`·가운데 방향 `mid`에서 양쪽으로 `half` 라디안.
    let arc = |c: Vec2, r: f32, mid: f32, half: f32, a: f32| {
        const SEGS: usize = 24;
        let pt = |i: usize| {
            let t = mid - half + 2.0 * half * i as f32 / SEGS as f32;
            c + vec2(t.cos(), t.sin()) * r
        };
        for (w, col) in [(3.6 * s, shade(a)), (1.8 * s, ink(a))] {
            for i in 0..SEGS {
                let (p, q) = (pt(i), pt(i + 1));
                draw_line(p.x, p.y, q.x, q.y, w, col);
            }
        }
    };
    for i in 0..3 {
        // 물결마다 조금씩 늦게 떠나 차례로 번진다.
        let t = (k * 1.5 - i as f32 * 0.18).clamp(0.0, 1.0);
        if t <= 0.0 || t >= 1.0 {
            continue;
        }
        let a = (t / 0.15).min((1.0 - t) / 0.5).clamp(0.0, 1.0) * 0.85;
        match out {
            Some(ang) => {
                // 소리의 샘은 화면 밖 — 가장자리 너머의 점에서 원이 자라 화면 안으로 들어온다.
                let d = vec2(ang.cos(), ang.sin());
                let c = at + d * KNELL_BACK * s;
                let r = (KNELL_BACK + 4.0 + t * KNELL_REACH) * s;
                arc(c, r, ang + std::f32::consts::PI, KNELL_HALF, a);
            }
            None if i < 2 => arc(
                at,
                (5.0 + t * KNELL_REACH * 0.6) * s,
                0.0,
                std::f32::consts::PI,
                a,
            ),
            None => {}
        }
    }
}

/// 종소리 물결의 빛·크기(480×270 기준 px): 샘이 가장자리 너머로 물러난 거리 · 번지는 거리 · 호의 반각(라디안).
pub const KNELL_INK: Color = Color::new(0.80, 0.88, 1.0, 1.0);
const KNELL_BACK: f32 = 64.0;
const KNELL_REACH: f32 = 46.0;
const KNELL_HALF: f32 = 0.5;

/// 적 쪽 화살의 빛(0170): 적의 잿빛(`light::enemy_tint`이 만드는 몸빛)과 같은 줄기 — 색 없는 밝은 회색.
pub const FOE_INK: Color = Color::new(0.82, 0.80, 0.78, 1.0);

/// 박자 타의 스태미나 막대 반짝임(초).
const BEAT_GLINT: f32 = 0.4;
/// 박자 타의 음표가 머리 위로 떠오르는 시간(초).
pub const BEAT_NOTE_LIFE: f32 = 0.55;

/// 박자 타(0076, KB nail 0168 "음표"): 머리 위(`at` 화면 자리)에서 음표가 떠오르며 사라진다. `k` 0..1 진행, `chain` 잇단 수 —
/// 둘째부터 음표 둘(♫), 셋째부터 크게. 어두운 테두리로 어느 바닥 위에서도.
pub fn draw_beat_note(f: &Frame, at: Vec2, k: f32, chain: u32) {
    let s = f.ui * if chain >= 3 { 1.25 } else { 1.0 };
    let rise = 10.0 * f.ui * (1.0 - (1.0 - k) * (1.0 - k));
    let a = if k < 0.7 { 1.0 } else { (1.0 - k) / 0.3 };
    let gold = Color::new(1.0, 0.91, 0.6, a);
    let shade = Color::new(0.08, 0.05, 0.08, 0.8 * a);
    let base = at - vec2(0.0, rise);
    let heads: &[Vec2] = if chain >= 2 {
        &[vec2(-3.0, 0.0), vec2(3.0, -1.0)]
    } else {
        &[vec2(0.0, 0.0)]
    };
    for pass in 0..2 {
        let (col, grow) = if pass == 0 {
            (shade, f.ui)
        } else {
            (gold, 0.0)
        };
        let w = 1.0 * s + grow;
        let mut tops = Vec::new();
        for h in heads {
            let c = base + *h * s;
            draw_ellipse(c.x, c.y, 2.0 * s + grow, 1.5 * s + grow, -20.0, col);
            let top = c + vec2(1.6 * s, -7.0 * s);
            draw_line(c.x + 1.6 * s, c.y, top.x, top.y - grow, w, col);
            tops.push(top);
        }
        if let [a0, b0] = tops[..] {
            draw_line(a0.x, a0.y, b0.x, b0.y, 1.6 * s + grow, col);
        } else if let [t0] = tops[..] {
            draw_line(t0.x, t0.y, t0.x + 2.5 * s, t0.y + 3.0 * s, w, col);
        }
    }
}

/// 기력이 모자라 막혔다(0088): 머리 양옆에서 땀방울 둘이 튀어 오르다 떨어지며 사라진다(귀여운 "헉" — 몸에서 읽히게, KB action-reuse-gating).
/// `k` 0..1 = 알림 시간의 진행.
pub fn draw_sweat(f: &Frame, at: Vec2, k: f32) {
    let s = f.ui;
    let a = if k < 0.6 { 1.0 } else { (1.0 - k) / 0.4 };
    let drop = Color::new(0.62, 0.86, 1.0, a);
    let shade = Color::new(0.05, 0.06, 0.1, 0.8 * a);
    // 튀어 나가 포물선으로 떨어진다(화면 px, 머리 기준).
    let fly = |side: f32| {
        let x = side * (4.0 + 7.0 * k) * s;
        let y = (-4.0 - 10.0 * k + 14.0 * k * k) * s;
        at + vec2(x, y)
    };
    for side in [-1.0f32, 1.0] {
        let c = fly(side);
        for (col, grow) in [(shade, s), (drop, 0.0)] {
            let r = 1.6 * s + grow;
            draw_circle(c.x, c.y, r, col);
            // 물방울 꼭지는 날아가는 반대쪽 위로.
            let tip = c + vec2(-side * 1.2 * s, -3.6 * s - grow * 1.4);
            draw_triangle(
                c + vec2(-r * 0.92, -r * 0.3),
                c + vec2(r * 0.92, -r * 0.3),
                tip,
                col,
            );
        }
    }
}

/// 커서 조준 표시(0058): 고리 + 가운데 점, 어두운 테두리로 어느 바닥에서도 보이게. 크기 단계 0 작게·1 보통·2 크게.
/// 붉은색(위험)은 쓰지 않는다 — 적의 예고와 섞이지 않게.
pub fn draw_reticle(f: &Frame, at: Vec2, size: u8) {
    let r = [3.0, 4.5, 6.5][(size as usize).min(2)] * f.ui;
    let w = (f.ui * 0.9).max(1.0);
    let shade = Color::new(0.05, 0.04, 0.06, 0.75);
    draw_circle_lines(at.x, at.y, r, w + 2.0, shade);
    draw_circle_lines(at.x, at.y, r, w, INK);
    draw_circle(at.x, at.y, w + 1.0, shade);
    draw_circle(at.x, at.y, w * 0.6, INK);
}

/// 맞음 테의 세기(0..1): 맞은 순간부터 `HURT_EDGE_HOLD` 동안 가득, 그 뒤 `HURT_EDGE_FADE`에 걸쳐 제곱으로 사라진다.
/// 센 타격(잃은 체력 `HURT_HEAVY_BLOW` 이상)은 `HURT_HEAVY`배(1 안). 로직 시계라 히트스톱 동안은 가득한 채 멈춰 있다.
pub fn hurt_edge_k(since: f32, blow: i32) -> f32 {
    use crate::tuning::{HURT_EDGE_FADE, HURT_EDGE_HOLD, HURT_HEAVY, HURT_HEAVY_BLOW};
    if !(0.0..HURT_EDGE_HOLD + HURT_EDGE_FADE).contains(&since) {
        return 0.0;
    }
    let k = 1.0 - ((since - HURT_EDGE_HOLD) / HURT_EDGE_FADE).clamp(0.0, 1.0);
    let heavy = if blow >= HURT_HEAVY_BLOW {
        HURT_HEAVY
    } else {
        1.0
    };
    (k * k * heavy / HURT_HEAVY).min(1.0)
}

/// 체력이 위험한가(0143): `LOW_HP`(한 대 몫) 이하로 살아 있고, 최대가 그보다 많을 때만(처음부터 그만큼인 판에서 늘 울리지 않게).
pub fn low_hp(hp: i32, max_hp: i32) -> bool {
    hp > 0 && hp <= crate::tuning::LOW_HP && max_hp > crate::tuning::LOW_HP
}

/// 체력 막대(0158 N0)의 높이·체력 1의 길이·길이 상한(UI px). 기본 체력(50)이 기본 스태미나 막대와 같은 60px이고,
/// 최대 체력 +10마다 12px 길어진다(늘어난 몫이 눈에 보이게 — 스태미나 막대와 같은 꼴, 0104). 상한을 넘으면 더 길어지지 않는다.
const HP_BAR_H: f32 = 5.0;
const HP_BAR_PER_HP: f32 = 1.2;
const HP_BAR_MAX_W: f32 = 160.0;

/// 체력 막대의 길이(UI px): 최대 체력에 비례, `HP_BAR_MAX_W`까지.
pub fn hp_bar_w(max_hp: i32) -> f32 {
    (max_hp.max(0) as f32 * HP_BAR_PER_HP).min(HP_BAR_MAX_W)
}

/// 방금 잃은 몫(0143 "부서지는 체력 칸"의 막대 꼴, 0158 N0): 맞은 뒤 `HURT_PIP` 동안 채움 끝에 밝게 남는 체력 —
/// 처음 `HURT_PIP_HOLD`의 몫은 잃은 만큼 그대로 머물고, 그 뒤 끝까지 0으로 줄어든다. 로직 시계라 히트스톱 동안은 멈춰 있다.
pub fn hp_chip(since: f32, blow: i32) -> f32 {
    use crate::tuning::{HURT_PIP, HURT_PIP_HOLD};
    if !(0.0..HURT_PIP).contains(&since) || blow <= 0 {
        return 0.0;
    }
    let k = ((since / HURT_PIP - HURT_PIP_HOLD) / (1.0 - HURT_PIP_HOLD)).clamp(0.0, 1.0);
    blow as f32 * (1.0 - k)
}

/// 화면 가장자리의 붉은 테(0143): 가장자리에서 안으로 계단지게 옅어지는 띠 일곱 겹(도트 화면이라 매끈한 그러데이션 대신 계단).
/// `a` = 가장 바깥 띠의 불투명도. 붉은색은 바닥의 위험 예고와 같은 뜻("위험") — 예고는 바닥에, 이것은 화면 테두리에만.
fn draw_edge(s: f32, a: f32) {
    if a <= 0.004 {
        return;
    }
    const BANDS: usize = 7;
    let (sw, sh) = (screen_width(), screen_height());
    let step = (3.0 * s).max(2.0);
    for i in 0..BANDS {
        let k = 1.0 - i as f32 / BANDS as f32;
        let col = Color::new(0.86, 0.1, 0.16, a * k * k);
        let o = i as f32 * step;
        // 위·아래(가로 전체), 왼쪽·오른쪽(그 사이) — 겹치지 않게.
        draw_rectangle(o, o, sw - 2.0 * o, step, col);
        draw_rectangle(o, sh - o - step, sw - 2.0 * o, step, col);
        draw_rectangle(o, o + step, step, sh - 2.0 * (o + step), col);
        draw_rectangle(sw - o - step, o + step, step, sh - 2.0 * (o + step), col);
    }
}

/// `edge`: 맞음 테의 세기 배율(랩 `hurt.edge`, 0 = 끔 — 창이 위에 뜬 동안은 0).
/// 스킬 칸(0151): 오른쪽 아래 — 끼운 둘의 그림(터치 버튼과 같은 픽토그램) + 대기(위에서 덮였다 걷힌다) + 버튼 이름,
/// 그 위에 큰 기술 게이지(가득 차면 금빛으로 숨 쉰다). 터치는 화면 버튼이 같은 것을 보여 주므로 그리지 않는다.
/// `ready` = 칸마다 준비 0..1(`World::skill_ready`).
fn draw_skills(ui: &Ui, f: &Frame, w: &World, ready: [f32; 2]) {
    use crate::input::{self, Bind, Device};
    if input::device() == Device::Touch {
        return;
    }
    let s = u(f);
    let pl = &w.player;
    let (side, gap) = (15.0 * s, 4.0 * s);
    let label = 0.7;
    let x1 = f.ox + f.w - 8.0 * s - side;
    let x0 = x1 - gap - side;
    let y = f.oy + f.h - 6.0 * s - ui.px(label) - side;
    let blink = pl.skill.denied > 0.0 && (w.time * 16.0) as i32 % 2 == 0;
    let b = input::bindings();
    for (i, (x, bind)) in [(x0, Bind::Skill1), (x1, Bind::Skill2)]
        .into_iter()
        .enumerate()
    {
        let k = ready[i].clamp(0.0, 1.0);
        draw_rectangle(x - s, y - s, side + 2.0 * s, side + 2.0 * s, BAR_BG);
        let back = if blink && k < 1.0 { DENIED } else { HP_OFF };
        draw_rectangle(x, y, side, side, back);
        // 픽토그램 한 칸 = UI 한 px(13칸 = 13 px).
        let c = vec2(x + side * 0.5, y + side * 0.5);
        let dim = if k < 1.0 { 0.45 } else { 1.0 };
        let tint = Color::new(1.0, 0.99, 0.95, 1.0);
        let r = crate::touch::ICON_N as f32 * s / 1.25 + 0.01;
        // 빈 칸(0166 — 아직 배운 스킬이 없다): 그림 없이 어두운 칸과 키 이름만(자리는 지킨다 — KB "빈 칸도 자리를 지킨다").
        let vacant = pl.skill.rank[i] == 0;
        if !vacant {
            crate::touch::icon(crate::touch::skill_icon(pl.skill.worn[i]), c, r, dim, tint);
        }
        if k < 1.0 && !vacant {
            draw_rectangle(x, y, side, side * (1.0 - k), Color::new(0.0, 0.0, 0.0, 0.5));
        }
        // 스킬이 남긴 것이 이어지는 동안(0158 G2 — 되울림의 울림·잔불 흩기의 불티): 칸 아래에 줄어드는 밝은 띠.
        let live = w.skill_active(i);
        if live > 0.0 {
            draw_rectangle(x, y + side - 2.0 * s, side * live, 2.0 * s, GAUGE_FULL);
        }
        let name = match input::device() {
            Device::Pad => b.pad(bind).name(),
            _ => input::key_name(b.keys[bind as usize]),
        };
        // 칸의 버튼 이름도 키캡으로(0168 — 힌트·곁의 할 일과 같은 꼴).
        let cap = crate::ui::key(name);
        ui.text_center(&cap, x + side * 0.5, y + side + 2.5 * s, label, INK);
    }
    // 큰 기술 게이지: 칸 둘의 폭. 큰 기술을 아직 안 배웠으면(0172 — 단 0) 그리지 않는다.
    if pl.skill.ult_rank == 0 {
        return;
    }
    let (gx, gw, gh) = (x0, x1 + side - x0, 2.5 * s);
    let gy = y - 3.0 * s - gh;
    draw_rectangle(gx - s, gy - s, gw + 2.0 * s, gh + 2.0 * s, BAR_BG);
    // 몰아치기(0158 G2)가 이어지는 동안은 같은 막대가 남은 시간으로 줄어든다(밝은 금빛 — 그동안 차오르는 게이지는 끝난 뒤에 보인다).
    let (fill, col) = gauge_bar(pl.skill.gauge, w.ult_active(), w.time);
    draw_rectangle(gx, gy, gw * fill, gh, col);
}

/// 큰 기술 막대의 (채움 0..1, 빛깔): 몰아치기가 이어지는 동안(`active` > 0)은 남은 몫, 아니면 게이지 — 가득이면 금빛으로 숨 쉰다.
pub fn gauge_bar(gauge: f32, active: f32, time: f32) -> (f32, Color) {
    if active > 0.0 {
        (active.clamp(0.0, 1.0), GAUGE_FULL)
    } else if gauge >= 1.0 {
        let p = 0.75 + 0.25 * (time * 5.0).sin();
        (1.0, Color::new(1.0, 0.86 * p + 0.1, 0.45 * p, 1.0))
    } else {
        (gauge.clamp(0.0, 1.0), GAUGE)
    }
}

/// 튜토리얼의 힌트 판(0167 → 0171 "잘 눈에 안 띈다"): 화면 아래 가운데의 어두운 판 + 위쪽 호박색 줄. 안에 걸음 점
/// (지난 것 민트 · 지금 호박 · 남은 것 빈 고리 — "[2/7]"을 대신한다), 큰 머리 줄(키캡 + 할 일 + 세는 수는 호박색으로
/// 따로), 작은 풀이 줄. 걸음이 막 바뀐 동안(`fresh` 초)은 머리 줄이 민트로 밝았다가 가라앉는다. 터치의 엄지 자리
/// (왼쪽 스틱·오른쪽 버튼 호)를 피해 가운데 폭 안에서 — 넘치는 풀이는 줄을 바꾸고 판이 위로 자란다.
/// 판의 자리는 `hint_box`가 정한다(곁의 할 일 글이 그 위로 비킨다).
fn draw_hint(ui: &Ui, f: &Frame, h: &Hint, fresh: f32) {
    let s = u(f);
    let lay = hint_box(ui, f, h);
    let r = lay.rect;
    draw_rectangle(r.x, r.y, r.w, r.h, HINT_PANEL);
    draw_rectangle(r.x, r.y, r.w, (1.5 * s).round().max(1.0), AMBER);
    #[cfg(feature = "dev")]
    crate::ui::check::frame(r);
    let cx = r.x + r.w * 0.5;
    // 걸음 점.
    let n = crate::tutorial::DONE as usize;
    let pitch = HINT_DOT_PITCH * s;
    let mut y = r.y + HINT_PAD_TOP * s;
    let dy = y + HINT_DOT * s;
    for i in 0..n {
        let dx = cx + (i as f32 - (n as f32 - 1.0) * 0.5) * pitch;
        match i.cmp(&(h.at as usize)) {
            std::cmp::Ordering::Less => draw_circle(dx, dy, HINT_DOT * s * 0.8, crate::ui::MINT),
            std::cmp::Ordering::Equal => {
                draw_circle(dx, dy, HINT_DOT * s * 1.25, crate::ui::SHADOW);
                draw_circle(dx, dy, HINT_DOT * s, AMBER);
            }
            std::cmp::Ordering::Greater => {
                draw_circle_lines(dx, dy, HINT_DOT * s * 0.8, s.max(1.0), DIM)
            }
        }
    }
    y += (HINT_DOT * 2.0 + HINT_GAP) * s;
    // 머리 줄: 키캡 + 할 일 + 세는 수.
    let k = (fresh / crate::tutorial::FRESH_TIME).clamp(0.0, 1.0);
    let mix = |a: f32, b: f32| a + (b - a) * k;
    let ink = Color::new(
        mix(INK.r, crate::ui::MINT.r),
        mix(INK.g, crate::ui::MINT.g),
        mix(INK.b, crate::ui::MINT.b),
        1.0,
    );
    let hk = lay.head_k;
    let mut x = cx - lay.head_w * 0.5;
    if !h.key.is_empty() {
        ui.text(&h.key, x, y, hk, ink);
        x += ui.width(&h.key, hk) + lay.gap;
    }
    ui.text(h.text, x, y, hk, ink);
    x += ui.width(h.text, hk);
    if let Some(c) = &lay.count {
        let ck = hk * HINT_COUNT_K;
        // 작은 글을 큰 글의 밑줄 가까이에 앉힌다.
        let cy = y + (ui.px(hk) - ui.px(ck)) * 0.7;
        ui.text(c, x + lay.gap * 1.6, cy, ck, AMBER);
    }
    y += ui.px(hk) + HINT_GAP * 0.5 * s;
    let soft = Color::new(INK.r, INK.g, INK.b, 0.8);
    for line in &lay.more {
        ui.text_center(line, cx, y, HINT_MORE_SCALE, soft);
        y += ui.px(HINT_MORE_SCALE) * HINT_LINE;
    }
}

/// 힌트 판이 차지하는 자리 — 가장자리 화살·종소리 물결이 그 위로 비킨다(판에 겹치면 글도 화살도 안 읽힌다).
pub fn hint_rect(ui: &Ui, f: &Frame, h: &Hint) -> Rect {
    hint_box(ui, f, h).rect
}

/// 힌트 판의 자리와 그 안의 글 배치(그리기와 곁의 할 일 글이 같이 읽는다).
struct HintBox {
    rect: Rect,
    head_k: f32,
    head_w: f32,
    gap: f32,
    count: Option<String>,
    more: Vec<String>,
}

fn hint_box(ui: &Ui, f: &Frame, h: &Hint) -> HintBox {
    let s = u(f);
    let pad = HINT_PAD_X * s;
    let room = f.w * HINT_W - pad * 2.0;
    let count = h.count.map(|(a, b)| format!("{a} / {b}"));
    let head = |k: f32| {
        let gap = ui.px(k) * 0.35;
        let kw = if h.key.is_empty() {
            0.0
        } else {
            ui.width(&h.key, k) + gap
        };
        let cw = count
            .as_ref()
            .map_or(0.0, |c| gap * 1.6 + ui.width(c, k * HINT_COUNT_K));
        (kw + ui.width(h.text, k) + cw, gap)
    };
    // 머리 줄이 폭을 넘으면 글을 줄인다(줄을 바꾸지 않는다 — 키캡과 할 일이 갈라지면 안 읽힌다).
    let (w1, _) = head(HINT_SCALE);
    let head_k = HINT_SCALE * (room / w1.max(1.0)).min(1.0);
    let (head_w, gap) = head(head_k);
    let more = if h.more.is_empty() {
        Vec::new()
    } else {
        crate::ui::wrap(ui, h.more, room, HINT_MORE_SCALE)
    };
    let more_w = more
        .iter()
        .map(|l| ui.width(l, HINT_MORE_SCALE))
        .fold(0.0, f32::max);
    let dots_w = HINT_DOT_PITCH * s * crate::tutorial::DONE as f32;
    let w = (head_w.max(more_w).max(dots_w) + pad * 2.0)
        .max(HINT_MIN_W * s)
        .min(f.w * HINT_W);
    let hh = (HINT_PAD_TOP + HINT_DOT * 2.0 + HINT_GAP + HINT_GAP * 0.5 + HINT_PAD_BOTTOM) * s
        + ui.px(head_k)
        + ui.px(HINT_MORE_SCALE) * HINT_LINE * more.len() as f32;
    let (cx, bottom) = (f.ox + f.w * 0.5, f.oy + f.h - HINT_FLOOR * s);
    HintBox {
        rect: Rect::new(
            (cx - w * 0.5).round(),
            (bottom - hh).round(),
            w.round(),
            hh.round(),
        ),
        head_k,
        head_w,
        gap,
        count,
        more,
    }
}

/// 버튼 + 할 일 한 줄을 가운데에: 버튼 이름은 키캡(`ui::key` — 어두운 칸 + 테 + 밝은 글, 버튼처럼 보인다), 할 일은
/// 제 빛으로. 이름이 없는 장치(터치)는 글만.
fn key_line(ui: &Ui, key: &str, text: &str, cx: f32, y: f32, k: f32, color: Color) {
    if key.is_empty() {
        ui.text_center(text, cx, y, k, color);
        return;
    }
    let cap = crate::ui::key(key);
    let gap = ui.px(k) * 0.35;
    let (kw, tw) = (ui.width(&cap, k), ui.width(text, k));
    let x = cx - (kw + gap + tw) * 0.5;
    ui.text(&cap, x, y, k, color);
    ui.text(text, x + kw + gap, y, k, color);
}

/// 힌트 판(0171): 폭의 상한(화면 폭의 몫 — 터치의 엄지 자리 사이)·가장 좁은 폭·글 크기(머리 줄 · 세는 수는 그 몫 ·
/// 풀이 줄)·줄 높이. 나머지는 480×270 기준 px: 옆·위·아래 여백, 걸음 점의 반지름·간격, 줄 사이, 화면 바닥에서 뜬 높이.
const HINT_W: f32 = 0.5;
const HINT_MIN_W: f32 = 132.0;
const HINT_SCALE: f32 = 1.2;
const HINT_COUNT_K: f32 = 0.85;
const HINT_MORE_SCALE: f32 = 0.8;
const HINT_LINE: f32 = 1.12;
const HINT_PAD_X: f32 = 9.0;
const HINT_PAD_TOP: f32 = 4.5;
const HINT_PAD_BOTTOM: f32 = 3.5;
const HINT_DOT: f32 = 1.7;
const HINT_DOT_PITCH: f32 = 7.0;
const HINT_GAP: f32 = 3.0;
const HINT_FLOOR: f32 = 3.0;
/// 힌트 판의 면(큰 창의 `ui::PANEL`보다 조금 비친다 — 싸움 화면 위라 바닥이 아주 가리지는 않게).
const HINT_PANEL: Color = Color::new(0.07, 0.05, 0.09, 0.86);

/// HUD가 판 밖에서 받는 것(앱이 정한다): 창이 위에 떴나 · 맞음 테의 세기(설정) · 스킬 칸 둘의 준비 몫 · 튜토리얼의 힌트와 그
/// 밝은 시간(0171 — 곁의 할 일 글이 힌트 판 위로 비키게 HUD가 같이 놓는다).
pub struct Opts<'a> {
    pub under_menu: bool,
    pub edge: f32,
    pub skills: [f32; 2],
    pub hint: Option<(&'a Hint, f32)>,
}

pub fn draw(ui: &Ui, f: &Frame, w: &World, run: Option<&StageRun>, o: &Opts) {
    let Opts {
        under_menu,
        edge,
        skills,
        hint,
    } = *o;
    let s = u(f);
    // 맞음 테(0143 — "맞았는지 잘 모르겠음"): 맞은 순간 화면 가장자리가 붉게 번쩍였다 사라지고, 체력이 한 대 몫만 남으면 옅게 숨 쉰다.
    // HUD 글·막대보다 먼저 그려 그 아래에 깔린다.
    {
        use crate::tuning::HURT_EDGE_ALPHA;
        let pl = &w.player;
        let hit = hurt_edge_k(pl.hurt.since, pl.hurt.blow) * HURT_EDGE_ALPHA;
        let low = if low_hp(pl.hp, pl.max_hp) {
            0.1 + 0.08 * (0.5 + 0.5 * (w.time * 4.2).sin())
        } else {
            0.0
        };
        draw_edge(s, hit.max(low) * edge.clamp(0.0, 2.0));
    }
    // 터치면 왼쪽 위 멈춤 버튼을 비킨다(2026-11-06).
    let inset = if crate::input::device() == crate::input::Device::Touch {
        crate::touch::hud_inset(screen_width(), screen_height())
    } else {
        0.0
    };
    let (x0, y0) = (f.ox + 8.0 * s + inset, f.oy + 8.0 * s);
    // 체력: 막대(0158 N0 — 하트 칸에서). 길이 = 최대 체력, 어두운 바탕 위에 남은 몫.
    let pl = &w.player;
    let max_hp = pl.max_hp.max(pl.hp).max(1);
    // 맞으면(0143) 체력 막대가 잠깐 떨리고, 잃은 몫은 채움 끝에 밝게 남았다 줄어든다. 한 대 남으면 채움이 붉게 뛴다.
    use crate::tuning::HURT_HP_SHAKE;
    let jolt = (1.0 - pl.hurt.since / HURT_HP_SHAKE).clamp(0.0, 1.0);
    let hx = x0 + (w.time * 80.0).sin() * 2.5 * s * jolt;
    let (hy, hw, hh) = (y0 + 1.5 * s, hp_bar_w(max_hp) * s, HP_BAR_H * s);
    let per = hw / max_hp as f32;
    let beat = if low_hp(pl.hp, pl.max_hp) {
        0.5 + 0.5 * (w.time * 4.2).sin()
    } else {
        0.0
    };
    draw_rectangle(hx - s, hy - s, hw + 2.0 * s, hh + 2.0 * s, BAR_BG);
    draw_rectangle(hx, hy, hw, hh, HP_OFF);
    let fill = pl.hp.clamp(0, max_hp) as f32 * per;
    draw_rectangle(
        hx,
        hy,
        fill,
        hh,
        Color::new(
            HP_ON.r + (1.0 - HP_ON.r) * beat,
            HP_ON.g * (1.0 - 0.55 * beat),
            HP_ON.b * (1.0 - 0.3 * beat),
            1.0,
        ),
    );
    // 방금 잃은 몫: 채움 끝에서 최대까지만(그 사이 회복했으면 남은 자리만큼).
    let chip = (hp_chip(pl.hurt.since, pl.hurt.blow) * per).min(hw - fill);
    if chip > 0.0 {
        draw_rectangle(hx + fill, hy, chip, hh, Color::new(1.0, 0.96, 0.9, 1.0));
    }
    // 스태미나(0018): 모자라서 막히면 잠깐 붉게 깜빡인다(막은 이유가 보이게 — KB action-reuse-gating).
    // 레벨 기력(0104)이 오르면 막대가 길어진다(소울류 — 늘어난 몫이 눈에 보이게, 0096 L2).
    let base = SPECS[idx(P::StaminaMax)].def.max(1.0);
    let smax = w.stats.max_stamina().max(1.0);
    let k = (pl.stamina / smax).clamp(0.0, 1.0);
    let (bx, by, bw, bh) = (x0, y0 + 11.0 * s, 60.0 * s * smax / base, 2.0 * s);
    // 막혔을 때(0088): 깜빡임에 더해 막대가 좌우로 떨린다(줄어들며). 다 쓴 동안은 붉은 테가 천천히 숨 쉰다(지금 막히는 상태).
    let shake = (pl.stamina_denied / crate::tuning::STAMINA_DENIED_SHOW).clamp(0.0, 1.0);
    let bx = bx + (w.time * 70.0).sin() * 2.0 * s * shake;
    draw_rectangle(bx, by, bw, bh, BAR_BG);
    if pl.stamina <= 0.0 {
        let p = 0.5 + 0.5 * (w.time * 5.0).sin();
        let o = s;
        draw_rectangle_lines(
            bx - o,
            by - o,
            bw + 2.0 * o,
            bh + 2.0 * o,
            s.max(1.0),
            Color::new(1.0, 0.3, 0.25, 0.35 + 0.45 * p),
        );
    }
    let blink = pl.stamina_denied > 0.0 && (w.time * 16.0) as i32 % 2 == 0;
    draw_rectangle(bx, by, bw * k, bh, if blink { DENIED } else { STAMINA });
    // 박자 타(0076): 스태미나를 안 썼다 — 막대가 금빛으로 반짝였다 식는다(막대 위아래로 한 줄씩 넓게).
    let shots = matches!(pl.weapon.def().rb, Rb::Shot { .. });
    if pl.beat.since < BEAT_GLINT && !shots {
        let g = 1.0 - pl.beat.since / BEAT_GLINT;
        let col = Color::new(1.0, 0.92, 0.64, 0.7 * g);
        draw_rectangle(bx, by - s, bw * k, bh + 2.0 * s, col);
    }
    // 등잔(0031): 남은 것은 불 켠 등잔, 쓴 것은 꺼진 등잔. 못 마시면(가득·0) 잠깐 깜빡인다.
    let fblink = pl.flask_denied > 0.0 && (w.time * 16.0) as i32 % 2 == 0;
    for i in 0..pl.flask_max.max(pl.flask) {
        let (lx, ly) = (x0 + i as f32 * 9.0 * s, y0 + 16.0 * s);
        let lit = i < pl.flask;
        let body = match (fblink, lit) {
            (true, _) => DENIED,
            (false, true) => FLASK_ON,
            (false, false) => HP_OFF,
        };
        draw_rectangle(lx, ly + 3.0 * s, 5.0 * s, 4.0 * s, body);
        draw_rectangle(lx + 1.0 * s, ly + 2.0 * s, 3.0 * s, 1.0 * s, body);
        if lit {
            let c = vec2(lx + 2.5 * s, ly + 0.5 * s);
            draw_triangle(
                c + vec2(0.0, -1.5 * s),
                c + vec2(1.3 * s, 1.5 * s),
                c + vec2(-1.3 * s, 1.5 * s),
                FLASK_FLAME,
            );
        }
    }
    // 잔불(0064 M1): 등잔 줄 아래에 호박 마름모 + 수. 주우면 잠깐 커진다. 하나도 없으면 그리지 않는다.
    if w.embers > 0 {
        let pop = 1.0 + 0.35 * (1.0 - ((w.time - w.ember_at) / EMBER_POP).clamp(0.0, 1.0));
        let c = vec2(x0 + 3.0 * s, y0 + 33.0 * s);
        let r = 3.0 * s * pop;
        draw_triangle(
            c + vec2(0.0, -r),
            c + vec2(r * 0.8, 0.0),
            c + vec2(0.0, r),
            EMBER,
        );
        draw_triangle(
            c + vec2(0.0, -r),
            c + vec2(-r * 0.8, 0.0),
            c + vec2(0.0, r),
            EMBER,
        );
        let k = 0.8 * pop.min(1.15);
        ui.text(
            &w.embers.to_string(),
            x0 + 9.0 * s,
            c.y - ui.px(k) * 0.55,
            k,
            EMBER,
        );
    }
    // 마법 칸(0048): 등잔 오른쪽에 둥근 칸 셋 — 찬 만큼 채워지고, 못 쏘면 깜빡인다. 근접으로 맞히면 찬다.
    // 원거리 무기(0051)는 그 자리에 탄창(세운 막대 — 장전 중이면 아래에서 차오른다) + 수류탄(둥근 칸).
    let mblink = pl.ranged.cast_denied > 0.0 && (w.time * 16.0) as i32 % 2 == 0;
    let mx0 = x0 + pl.flask_max.max(pl.flask) as f32 * 9.0 * s + 6.0 * s;
    let wpn = pl.weapon.def();
    let pip = |i: i32, fill: f32, col: Color| {
        let c = vec2(mx0 + 3.0 * s + i as f32 * 8.0 * s, y0 + 20.5 * s);
        draw_circle(c.x, c.y, 3.0 * s, if mblink { DENIED } else { HP_OFF });
        if fill > 0.0 && !mblink {
            let col = if fill >= 1.0 {
                col
            } else {
                Color::new(col.r, col.g, col.b, 0.55)
            };
            draw_circle(c.x, c.y, 3.0 * s * fill.sqrt(), col);
        }
    };
    match (wpn.rb, wpn.grenade) {
        (Rb::Shot { mag, reload, .. }, g) => {
            for i in 0..mag as i32 {
                let (bx, by) = (mx0 + i as f32 * 4.0 * s, y0 + 17.0 * s);
                draw_rectangle(bx, by, 2.5 * s, 7.0 * s, HP_OFF);
                let k = if (i as u8) < pl.ranged.ammo {
                    1.0
                } else if pl.ranged.reload > 0.0 {
                    1.0 - (pl.ranged.reload / reload.max(0.01)).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                if k > 0.0 {
                    draw_rectangle(bx, by + 7.0 * s * (1.0 - k), 2.5 * s, 7.0 * s * k, AMMO);
                }
                // 박자 사격(0077): 볼트를 안 썼다 — 탄창이 금빛으로 반짝였다 식는다.
                if pl.beat.since < BEAT_GLINT {
                    let g = 1.0 - pl.beat.since / BEAT_GLINT;
                    draw_rectangle(
                        bx - 0.5 * s,
                        by - 0.5 * s,
                        3.5 * s,
                        8.0 * s,
                        Color::new(1.0, 0.92, 0.64, 0.7 * g),
                    );
                }
            }
            if let Some(g) = g {
                let gx = mag as f32 * 4.0 * s + 4.0 * s;
                for i in 0..g.count as i32 {
                    let c = vec2(mx0 + gx + 3.0 * s + i as f32 * 8.0 * s, y0 + 20.5 * s);
                    draw_circle(c.x, c.y, 3.0 * s, if mblink { DENIED } else { HP_OFF });
                    let fill = (pl.ranged.grenades - i as f32).clamp(0.0, 1.0);
                    if fill > 0.0 && !mblink {
                        let col = if fill >= 1.0 {
                            AMMO
                        } else {
                            Color::new(AMMO.r, AMMO.g, AMMO.b, 0.5)
                        };
                        draw_circle(c.x, c.y, 3.0 * s * fill.sqrt(), col);
                    }
                }
            }
        }
        (Rb::Magic(_), _) => {
            for i in 0..CAST_CHARGES as i32 {
                pip(i, (pl.ranged.mana - i as f32).clamp(0.0, 1.0), MANA);
            }
        }
    }

    draw_skills(ui, f, w, skills);

    let Some(run) = run else { return };
    // 곁의 할 일(0105·0064 M3 → 0166 `StageRun::near` — 터치 버튼의 그림과 같은 답): "키 줍기"·"키 열기"·… ·"키 쉬기".
    // 쉼터 곁인데 적이 가까우면 쉴 수 없다는 한 줄.
    // 튜토리얼의 힌트 판(0171)이 떠 있으면 그 위로 비킨다. 보스 띠가 같은 자리를 쓰는 동안은 힌트를 그리지 않는다.
    let hint = hint.filter(|_| !under_menu && run.boss_bar(w).is_none());
    let hint_top = hint.map(|(h, _)| hint_box(ui, f, h).rect.y);
    if let (Some(near), false) = (run.near(), under_menu) {
        use crate::stage::Near;
        let y = f.oy + f.h * 0.78;
        let y = hint_top.map_or(y, |top| y.min(top - ui.px(1.0) - 3.0 * s));
        let cx = f.ox + f.w * 0.5;
        let verb = match near {
            Near::Pick => Some(i18n::s().find_verb),
            Near::Open => Some(i18n::s().chest_verb),
            Near::Pull => Some(i18n::s().lever_verb),
            Near::Talk => Some(i18n::s().talk_verb),
            Near::Read => Some(i18n::s().read_verb),
            Near::Rest { safe: true } => Some(s_rest()),
            Near::Rest { safe: false } => None,
        };
        match verb {
            Some(verb) => {
                let key = crate::input::prompt(crate::input::Act::Interact);
                key_line(ui, &key, verb, cx, y, 1.0, AMBER);
            }
            None => ui.text_center(i18n::s().rest_blocked, cx, y, 1.0, DIM),
        }
    }
    if let Some((h, fresh)) = hint {
        draw_hint(ui, f, h, fresh);
    }
    let cx = f.ox + f.w * 0.5;
    // 목표(오른쪽 위). 창 밑에서는 그리지 않는다 — 구역 카드·시작 이야기와 같다(0066, 장비 창 테두리를 넘었다 0104 F4).
    if let Some(o) = run.objective.filter(|_| !under_menu) {
        let text = format!("◇ {}", tr(o));
        let tw = ui.width(&text, 1.0);
        ui.text(&text, f.ox + f.w - tw - 8.0 * s, y0, 1.0, INK);
    }
    // 구역 카드(가운데 위): 이름 + 가는 줄.
    if let Some((name, left)) = run.card.filter(|_| !under_menu) {
        let a = fade_in_out(left, CARD_TIME.max(left), 0.5);
        let y = f.oy + f.h * 0.16;
        let text = tr(name);
        let col = Color::new(INK.r, INK.g, INK.b, a);
        // 긴 언어(영어 "Rested — health and flasks restored")는 왼쪽 위 HUD(잔불 수)에 닿았다(0104 F4) — 가운데 62% 안으로 줄인다.
        let k = (2.0 * f.w * 0.62 / ui.width(text, 2.0).max(1.0)).min(2.0);
        ui.text_center(text, cx, y, k, col);
        let tw = ui.width(text, k);
        let ly = y + ui.px(k) * 1.15;
        draw_rectangle(
            cx - tw * 0.5,
            ly,
            tw,
            s,
            Color::new(AMBER.r, AMBER.g, AMBER.b, a * 0.7),
        );
    }
    // 시작 이야기(가운데): 스테이지 이름 + 두 줄.
    // 시작 이야기도 창 밑에서는 그리지 않는다(창 면이 비쳐 글이 겹쳤다, 0066).
    if run.intro > 0.0 && !under_menu {
        let a = fade_in_out(run.intro, INTRO_TIME, 0.8);
        let y = f.oy + f.h * 0.34;
        ui.text_center(
            tr(run.def.name),
            cx,
            y,
            2.0,
            Color::new(AMBER.r, AMBER.g, AMBER.b, a),
        );
        // 긴 언어에서도 화면 안에(nail `wrap`): 넘치면 줄을 바꿔 아래로 쌓는다.
        let (l1, l2) = run.def.intro;
        let max_w = f.w * 0.9;
        let y2 = ui.text_center_wrapped(
            tr(l1),
            cx,
            y + ui.px(2.0) * 1.6,
            1.0,
            Color::new(INK.r, INK.g, INK.b, a),
            max_w,
        );
        ui.text_center_wrapped(
            tr(l2),
            cx,
            y2 + ui.px(1.0) * 0.15,
            1.0,
            Color::new(DIM.r, DIM.g, DIM.b, a),
            max_w,
        );
    }
    // 보스 띠(아래 가운데): 이름 + 막대(깨어나면 색이 달아오른다).
    if let Some((k, roused)) = run.boss_bar(w).filter(|_| !under_menu) {
        let boss = run.boss.and_then(|i| w.enemies.get(i));
        let name = boss.map(|b| tr(b.def().name)).unwrap_or("");
        let bw = f.w * 0.5;
        let bh = 3.0 * s;
        let by = f.oy + f.h - 18.0 * s;
        let bx = cx - bw * 0.5;
        ui.text(name, bx, by - ui.px(1.0) - 2.0 * s, 1.0, INK);
        draw_rectangle(bx - s, by - s, bw + 2.0 * s, bh + 2.0 * s, BAR_BG);
        draw_rectangle(bx, by, bw * k, bh, if roused { BOSS_ROUSED } else { BOSS });
        // 자세 막대(0051·0055): 체력 띠 아래 가는 줄 — 차오를수록 밝게, 깨지면(폭딜 창) 금빛으로 깜빡인다.
        if let Some(b) = boss {
            let py = by + bh + 2.0 * s;
            let ph = 1.5 * s;
            draw_rectangle(bx - s, py - s * 0.5, bw + 2.0 * s, ph + s, BAR_BG);
            if b.broken > 0.0 {
                let on = (w.time * 10.0) as i32 % 2 == 0;
                draw_rectangle(bx, py, bw, ph, if on { POISE_BROKEN } else { POISE });
            } else {
                draw_rectangle(bx, py, bw * b.poise_frac, ph, POISE);
            }
        }
    }
}

/// 끝 화면 아래 줄: 걸린 시간·쓰러뜨린 수.
pub fn clear_line(run: &StageRun, w: &World) -> String {
    let secs = run.time as u32;
    let time = format!("{}:{:02}", secs / 60, secs % 60);
    i18n::fill(i18n::s().clear_sub, &[&time, &w.kills.to_string()])
}

/// 쉼터 동사(0032).
fn s_rest() -> &'static str {
    i18n::s().rest_verb
}

/// 지역 사이 전환의 깊이 글(0118 → 0174): 굴을 지나는 동안 화면 아래 가운데, 둥근 그늘 위에 — `a` = 진하기(굴이 어둠에서
/// 나오고 들어갈 때 같이 옅어진다).
pub fn draw_depth(u: &Ui, f: &Frame, label: &str, a: f32) {
    if a <= 0.0 {
        return;
    }
    let k = 1.2;
    let (tw, th) = (u.width(label, k), u.px(k));
    let cx = f.ox + f.w * 0.5;
    let ty = f.oy + f.h * 0.8;
    let pad = 7.0 * f.ui;
    draw_rectangle(
        cx - tw * 0.5 - pad * 1.4,
        ty - pad * 0.6,
        tw + pad * 2.8,
        th + pad * 1.2,
        Color::new(0.04, 0.03, 0.07, 0.55 * a),
    );
    u.text_center(label, cx, ty, k, Color::new(1.0, 0.86, 0.6, a));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 0143 맞음 테: 맞은 순간 가득(센 타격) — 보통 타격은 그보다 옅게 — 잠깐 머문 뒤 사라지고, 맞은 적이 없으면(무한대) 없다.
    #[test]
    fn the_hurt_edge_peaks_at_the_blow_holds_briefly_and_fades_out() {
        use crate::tuning::{HURT_EDGE_FADE, HURT_EDGE_HOLD};
        assert_eq!(hurt_edge_k(f32::MAX, 10), 0.0);
        let (light, heavy) = (hurt_edge_k(0.0, 10), hurt_edge_k(0.0, 20));
        assert!(light > 0.5 && (heavy - 1.0).abs() < 1e-6 && heavy > light);
        assert_eq!(
            hurt_edge_k(HURT_EDGE_HOLD * 0.9, 20),
            heavy,
            "held at full first"
        );
        let mid = hurt_edge_k(HURT_EDGE_HOLD + HURT_EDGE_FADE * 0.5, 20);
        assert!(mid > 0.0 && mid < 0.5, "{mid}");
        assert_eq!(hurt_edge_k(HURT_EDGE_HOLD + HURT_EDGE_FADE + 0.01, 20), 0.0);
    }

    /// 0143 위험한 체력: 마지막 한 대 몫일 때만 — 쓰러진 뒤(0)나 처음부터 그만큼인 판에서는 울리지 않는다.
    #[test]
    fn low_health_is_the_last_blow_of_a_longer_bar() {
        assert!(low_hp(10, 50) && !low_hp(20, 50) && !low_hp(0, 50) && !low_hp(10, 10));
        // 눈금 사이의 값(나중의 % 피해)도 한 대 몫 안이면 위험하다.
        assert!(low_hp(5, 50) && !low_hp(11, 50));
    }

    /// 0158 N0 체력 막대: 길이는 최대 체력에 비례해 늘고(+10이 눈에 보인다) 상한에서 멈춘다 — 기본 체력은 기본 스태미나 막대와 같은 길이.
    #[test]
    fn the_health_bar_grows_with_max_health_up_to_a_cap() {
        use crate::tuning::{HP_UNIT, PLAYER_HP};
        let base = hp_bar_w(PLAYER_HP);
        assert!((base - 60.0).abs() < 1e-4, "{base}");
        let more = hp_bar_w(PLAYER_HP + HP_UNIT);
        assert!(more - base >= 8.0, "+10 max health is plainly longer");
        // 능력치 상한(120)의 막대는 상한 안이고, 그 위로는 더 길어지지 않는다.
        assert!(hp_bar_w(120) > more && hp_bar_w(120) <= HP_BAR_MAX_W);
        assert_eq!(hp_bar_w(100_000), HP_BAR_MAX_W);
        assert_eq!(hp_bar_w(-5), 0.0);
    }

    /// 0158 N0(0143의 부서지는 칸을 막대로): 잃은 몫은 맞은 순간 그대로 남아 잠깐 머문 뒤 줄어들어 `HURT_PIP`에 없어진다.
    #[test]
    fn the_lost_health_stays_bright_briefly_then_drains() {
        use crate::tuning::{HURT_PIP, HURT_PIP_HOLD};
        assert_eq!(hp_chip(0.0, 20), 20.0);
        assert_eq!(hp_chip(HURT_PIP * HURT_PIP_HOLD * 0.9, 20), 20.0, "held");
        let mid = hp_chip(HURT_PIP * (HURT_PIP_HOLD + 1.0) * 0.5, 20);
        assert!(mid > 5.0 && mid < 15.0, "{mid}");
        let late = hp_chip(HURT_PIP * 0.95, 20);
        assert!(late > 0.0 && late < mid, "drains, never grows back");
        assert_eq!(hp_chip(HURT_PIP, 20), 0.0);
        // 맞은 적이 없으면(무한대)·잃은 것이 없으면 없다.
        assert_eq!(hp_chip(f32::MAX, 20), 0.0);
        assert_eq!(hp_chip(0.1, 0), 0.0);
    }

    #[test]
    fn cards_fade_in_and_out() {
        assert_eq!(fade_in_out(3.0, 3.0, 0.5), 0.0);
        assert_eq!(fade_in_out(1.5, 3.0, 0.5), 1.0);
        assert!(fade_in_out(0.1, 3.0, 0.5) < 0.3);
    }
}
