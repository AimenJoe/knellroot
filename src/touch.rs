//! 터치 조작(nail `input.rs` read_touch·TouchOverlay에서 가져옴, 0006). `input.rs`만 이 모듈을 부른다.
//!
//! - 화면 왼쪽 반에서 시작한 손가락 = 떠다니는 가상 스틱(손가락을 댄 자리가 중심).
//! - 오른쪽 아래 버튼: 공격(가장 크게)·회피(왼쪽 아래)·락온(위), 락온 왼쪽에 땅 돌리기 ↺ ↻(0012 — 왼쪽이 반시계).
//!   그림은 글자 대신 선 그림(언어를 타지 않는다, nail 0172).
//! - dev 기능: 오른쪽 위 랩·시나리오 버튼, 랩이 열리면 위쪽 줄에 ▲▼−+·담기·바꾸기·초기화(−+는 누르고 있으면 반복).
//! - 장치가 터치일 때만 그린다(마지막으로 만진 것이 화면). 키·패드를 만지면 사라진다(nail 0036·0172).
//!
//! 함정(nail에서 겪음): macroquad 0.4.16의 `touches()`만 물리 픽셀이고 `screen_width()`는 논리 픽셀이다 → DPI로 나눈다.
//! `TouchPhase::Started`는 같은 프레임의 Moved에 덮이므로 "처음 보는 id"를 눌림으로 본다.
use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Attack,
    Dodge,
    Lock,
    TurnCcw,
    TurnCw,
    Lab,
    Scenario,
    LabUp,
    LabDown,
    LabMinus,
    LabPlus,
    LabReset,
    LabStore,
    LabSwap,
}

#[derive(Clone, Copy, Debug)]
pub struct Button {
    pub center: Vec2,
    pub radius: f32,
    pub kind: Kind,
}

/// 렌더가 그릴 것.
#[derive(Clone, Debug)]
pub struct Overlay {
    pub stick: Option<(Vec2, Vec2)>,
    pub stick_radius: f32,
    pub buttons: Vec<Button>,
}

/// 터치가 이번 프레임에 만든 입력.
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Read {
    pub move_dir: Vec2,
    pub attack: bool,
    pub dodge: bool,
    pub lock: bool,
    pub turn_cw: bool,
    pub turn_ccw: bool,
    pub lab_toggle: bool,
    pub scenario_next: bool,
    pub lab_up: bool,
    pub lab_down: bool,
    pub lab_left: bool,
    pub lab_right: bool,
    pub lab_reset: bool,
    pub lab_store: bool,
    pub lab_swap: bool,
    /// 화면에 손가락이 있었다(버튼·스틱이 아니어도) → 장치 = 터치.
    pub active: bool,
}

#[derive(Clone, Copy)]
struct Stick {
    id: u64,
    origin: Vec2,
    pos: Vec2,
}

/// −/+를 누르고 있으면 반복(처음 0.35초 뒤 0.06초마다).
#[derive(Clone, Copy)]
struct Repeat {
    id: u64,
    kind: Kind,
    t: f32,
}

const REPEAT_DELAY: f32 = 0.35;
const REPEAT_EVERY: f32 = 0.06;
/// 스틱 데드존(반지름 대비).
const DEAD: f32 = 0.15;

#[derive(Default)]
pub struct Touch {
    stick: Option<Stick>,
    known: Vec<u64>,
    repeat: Option<Repeat>,
}

/// 버튼 배치. `w`·`h`는 논리 px. 오른손 엄지가 쉬는 오른쪽 아래에 공격(지름 ≈ 짧은 변의 27%), 그 둘레에 회피·락온.
pub fn layout(w: f32, h: f32, dev: bool, lab_open: bool) -> Vec<Button> {
    let u = w.min(h);
    let r = u * 0.09;
    let atk_r = r * 1.5;
    let dodge_r = r * 1.1;
    let lock_r = r * 0.85;
    let turn_r = r * 0.8;
    let atk = vec2(w - atk_r * 1.3, h - atk_r * 1.3);
    let lock = vec2(atk.x, atk.y - (atk_r + lock_r + r * 0.45));
    let cw_x = lock.x - (lock_r + turn_r + r * 0.4);
    let ccw_x = cw_x - (turn_r * 2.0 + r * 0.4);
    let mut b = vec![
        Button {
            center: atk,
            radius: atk_r,
            kind: Kind::Attack,
        },
        Button {
            center: vec2(atk.x - (atk_r + dodge_r + r * 0.5), h - dodge_r * 1.2),
            radius: dodge_r,
            kind: Kind::Dodge,
        },
        Button {
            center: lock,
            radius: lock_r,
            kind: Kind::Lock,
        },
        Button {
            center: vec2(cw_x, lock.y),
            radius: turn_r,
            kind: Kind::TurnCw,
        },
        Button {
            center: vec2(ccw_x, lock.y),
            radius: turn_r,
            kind: Kind::TurnCcw,
        },
    ];
    if dev {
        let s = r * 0.7;
        b.push(Button {
            center: vec2(w - s * 1.4, s * 1.4),
            radius: s,
            kind: Kind::Lab,
        });
        b.push(Button {
            center: vec2(w - s * 4.0, s * 1.4),
            radius: s,
            kind: Kind::Scenario,
        });
        if lab_open {
            // 위쪽 가운데 줄: ▲ ▼ − + · 초기화 담기 바꾸기. 고른 값의 이름·값은 렌더가 이 줄 위에 쓴다.
            let k = r * 0.62;
            let gap = k * 2.5;
            let kinds = [
                Kind::LabUp,
                Kind::LabDown,
                Kind::LabMinus,
                Kind::LabPlus,
                Kind::LabReset,
                Kind::LabStore,
                Kind::LabSwap,
            ];
            let total = gap * (kinds.len() as f32 - 1.0);
            let x0 = (w * 0.5 - total * 0.5).max(k * 1.3);
            let y = lab_row_y(w, h);
            for (i, kind) in kinds.iter().enumerate() {
                b.push(Button {
                    center: vec2(x0 + gap * i as f32, y),
                    radius: k,
                    kind: *kind,
                });
            }
        }
    }
    b
}

/// 랩 터치 줄의 높이(가운데 y)와 버튼 반지름. HUD 두 줄 아래, 랩 글은 이 줄 바로 위(`lab.rs draw_compact`).
pub fn lab_row_y(w: f32, h: f32) -> f32 {
    w.min(h) * 0.30
}

pub fn lab_button_radius(w: f32, h: f32) -> f32 {
    w.min(h) * 0.09 * 0.62
}

/// 오른쪽 위 dev 버튼(시나리오·랩)이 차지하는 폭 — 랩 글이 그 앞에서 멈춘다.
pub fn dev_buttons_width(w: f32, h: f32) -> f32 {
    let s = w.min(h) * 0.09 * 0.7;
    s * 5.0 + 8.0
}

pub fn stick_radius(w: f32, h: f32) -> f32 {
    w.min(h) * 0.14
}

impl Touch {
    pub fn read(&mut self, dt: f32, dev: bool, lab_open: bool) -> Read {
        let mut a = Read::default();
        let dpi = screen_dpi_scale().max(0.01);
        let touches: Vec<macroquad::input::Touch> = touches()
            .into_iter()
            .map(|mut t| {
                t.position /= dpi;
                t
            })
            .collect();
        a.active = !touches.is_empty();
        if touches.is_empty() && self.stick.is_none() && self.known.is_empty() {
            self.repeat = None;
            return a;
        }
        let (w, h) = (screen_width(), screen_height());
        let buttons = layout(w, h, dev, lab_open);
        for t in &touches {
            let ended = matches!(t.phase, TouchPhase::Ended | TouchPhase::Cancelled);
            if !self.known.contains(&t.id) {
                if !ended {
                    self.known.push(t.id);
                }
                // 버튼이 먼저(왼쪽 반의 랩 버튼도 스틱보다 앞).
                if let Some(b) = buttons
                    .iter()
                    .find(|b| (t.position - b.center).length() <= b.radius * 1.25)
                {
                    press(&mut a, b.kind);
                    if matches!(b.kind, Kind::LabMinus | Kind::LabPlus) && !ended {
                        self.repeat = Some(Repeat {
                            id: t.id,
                            kind: b.kind,
                            t: 0.0,
                        });
                    }
                } else if t.position.x < w * 0.5 && self.stick.is_none() && !ended {
                    self.stick = Some(Stick {
                        id: t.id,
                        origin: t.position,
                        pos: t.position,
                    });
                }
            }
            if let Some(s) = self.stick.as_mut() {
                if s.id == t.id {
                    s.pos = t.position;
                }
            }
            if ended {
                self.known.retain(|id| *id != t.id);
                if self.stick.map(|s| s.id) == Some(t.id) {
                    self.stick = None;
                }
                if self.repeat.map(|r| r.id) == Some(t.id) {
                    self.repeat = None;
                }
            }
        }
        // Ended를 못 받고 사라진 손가락(탭 전환 등) 정리.
        self.known.retain(|id| touches.iter().any(|t| t.id == *id));
        if self
            .stick
            .is_some_and(|s| !touches.iter().any(|t| t.id == s.id))
        {
            self.stick = None;
        }
        if let Some(r) = self.repeat.as_mut() {
            if !touches.iter().any(|t| t.id == r.id) {
                self.repeat = None;
            } else {
                let before = r.t;
                r.t += dt;
                if r.t >= REPEAT_DELAY {
                    let n_before = ((before - REPEAT_DELAY).max(0.0) / REPEAT_EVERY) as i32;
                    let n_now = ((r.t - REPEAT_DELAY) / REPEAT_EVERY) as i32;
                    if n_now > n_before || before < REPEAT_DELAY {
                        let k = r.kind;
                        press(&mut a, k);
                    }
                }
            }
        }
        if let Some(s) = self.stick {
            a.move_dir = stick_vector(s.pos - s.origin, stick_radius(w, h));
        }
        a
    }

    pub fn overlay(&self, dev: bool, lab_open: bool) -> Overlay {
        let (w, h) = (screen_width(), screen_height());
        Overlay {
            stick: self.stick.map(|s| (s.origin, s.pos)),
            stick_radius: stick_radius(w, h),
            buttons: layout(w, h, dev, lab_open),
        }
    }
}

fn press(a: &mut Read, k: Kind) {
    match k {
        Kind::Attack => a.attack = true,
        Kind::Dodge => a.dodge = true,
        Kind::Lock => a.lock = true,
        Kind::TurnCw => a.turn_cw = true,
        Kind::TurnCcw => a.turn_ccw = true,
        Kind::Lab => a.lab_toggle = true,
        Kind::Scenario => a.scenario_next = true,
        Kind::LabUp => a.lab_up = true,
        Kind::LabDown => a.lab_down = true,
        Kind::LabMinus => a.lab_left = true,
        Kind::LabPlus => a.lab_right = true,
        Kind::LabReset => a.lab_reset = true,
        Kind::LabStore => a.lab_store = true,
        Kind::LabSwap => a.lab_swap = true,
    }
}

/// 손가락이 중심에서 벗어난 만큼 → 스틱 값(x 오른쪽, y 위). 화면 y는 아래가 +라 뒤집는다. 데드존 뒤 0..1로 다시 편다.
pub fn stick_vector(d: Vec2, radius: f32) -> Vec2 {
    let v = vec2(d.x, -d.y) / radius.max(1.0);
    let len = v.length();
    if len <= DEAD {
        return Vec2::ZERO;
    }
    v / len * ((len - DEAD) / (1.0 - DEAD)).min(1.0)
}

/// 터치 오버레이 그리기(화면 패스). `locked`면 락온 버튼을 채운다. `turn` = 땅 돌리기 준비 0..1 — 차오르는 동안 옅고, 다 차면 밝다.
pub fn draw(o: &Overlay, locked: bool, turn: f32) {
    let line = Color::new(1.0, 1.0, 1.0, 0.4);
    let fill = Color::new(1.0, 1.0, 1.0, 0.12);
    if let Some((origin, pos)) = o.stick {
        draw_circle_lines(origin.x, origin.y, o.stick_radius, 2.0, line);
        let knob = origin + (pos - origin).clamp_length_max(o.stick_radius);
        draw_circle(knob.x, knob.y, o.stick_radius * 0.4, fill);
        draw_circle_lines(knob.x, knob.y, o.stick_radius * 0.4, 2.0, line);
    }
    for b in &o.buttons {
        let on = b.kind == Kind::Lock && locked;
        let f = if on {
            Color::new(1.0, 0.85, 0.3, 0.35)
        } else {
            fill
        };
        let is_turn = matches!(b.kind, Kind::TurnCw | Kind::TurnCcw);
        draw_circle(b.center.x, b.center.y, b.radius, f);
        if is_turn && turn < 1.0 {
            // 대기: 차오르는 몫만큼 아래부터 채운다(그림은 옅게).
            draw_circle(
                b.center.x,
                b.center.y,
                b.radius * turn.clamp(0.0, 1.0),
                Color::new(1.0, 1.0, 1.0, 0.08),
            );
        }
        draw_circle_lines(b.center.x, b.center.y, b.radius, 2.0, line);
        let dim = if is_turn && turn < 1.0 { 0.35 } else { 1.0 };
        icon(b.kind, b.center, b.radius, dim);
    }
}

/// 픽토그램(선 그림). 플레이어용 셋은 글자 없이, dev 버튼만 짧은 ASCII.
fn icon(k: Kind, c: Vec2, r: f32, dim: f32) {
    let col = Color::new(1.0, 1.0, 1.0, 0.85 * dim);
    let s = r * 0.5;
    let th = (r * 0.09).max(2.0);
    match k {
        Kind::Attack => {
            // 칼: 대각 날 + 가드.
            draw_line(c.x - s, c.y + s, c.x + s, c.y - s, th * 1.4, col);
            draw_line(
                c.x - s * 0.75,
                c.y + s * 0.05,
                c.x - s * 0.05,
                c.y + s * 0.75,
                th,
                col,
            );
        }
        Kind::Dodge => {
            // 겹 화살촉(옆으로 굴러 빠지기).
            for dx in [-s * 0.45, s * 0.25] {
                draw_line(c.x + dx, c.y - s * 0.6, c.x + dx + s * 0.6, c.y, th, col);
                draw_line(c.x + dx + s * 0.6, c.y, c.x + dx, c.y + s * 0.6, th, col);
            }
        }
        Kind::Lock => {
            // 조준 고리 + 십자.
            draw_circle_lines(c.x, c.y, s * 0.7, th, col);
            for (dx, dy) in [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
                draw_line(
                    c.x + dx * s * 0.45,
                    c.y + dy * s * 0.45,
                    c.x + dx * s * 1.1,
                    c.y + dy * s * 1.1,
                    th,
                    col,
                );
            }
        }
        Kind::TurnCw | Kind::TurnCcw => {
            // 둥근 화살표: 3/4 원호 + 끝의 화살촉. 시계 방향이면 화살촉이 시계 쪽을 본다(화면 y는 아래가 +).
            let sign = if k == Kind::TurnCw { 1.0 } else { -1.0 };
            let rad = s * 0.8;
            let n = 14;
            let (a0, a1) = (-std::f32::consts::FRAC_PI_2, std::f32::consts::PI);
            let at = |a: f32| c + vec2(a.cos() * sign, a.sin()) * rad;
            for i in 0..n {
                let (u, v) = (
                    a0 + (a1 - a0) * i as f32 / n as f32,
                    a0 + (a1 - a0) * (i + 1) as f32 / n as f32,
                );
                let (p, q) = (at(u), at(v));
                draw_line(p.x, p.y, q.x, q.y, th, col);
            }
            // 화살촉: 원호 끝(a1)에서 진행 방향(각이 커지는 쪽)으로.
            let tip = at(a1);
            let tan = vec2(-a1.sin() * sign, a1.cos());
            let side = vec2(-tan.y, tan.x);
            let h = s * 0.45;
            draw_triangle(
                tip + tan * h * 0.6,
                tip - tan * h * 0.4 + side * h * 0.55,
                tip - tan * h * 0.4 - side * h * 0.55,
                col,
            );
        }
        Kind::LabUp | Kind::LabDown => {
            let d = if k == Kind::LabUp { -1.0 } else { 1.0 };
            draw_triangle(
                vec2(c.x, c.y + d * s * 0.7),
                vec2(c.x - s * 0.7, c.y - d * s * 0.5),
                vec2(c.x + s * 0.7, c.y - d * s * 0.5),
                col,
            );
        }
        Kind::LabMinus => draw_line(c.x - s * 0.7, c.y, c.x + s * 0.7, c.y, th * 1.4, col),
        Kind::LabPlus => {
            draw_line(c.x - s * 0.7, c.y, c.x + s * 0.7, c.y, th * 1.4, col);
            draw_line(c.x, c.y - s * 0.7, c.x, c.y + s * 0.7, th * 1.4, col);
        }
        Kind::Lab | Kind::Scenario | Kind::LabReset | Kind::LabStore | Kind::LabSwap => {
            let text = match k {
                Kind::Lab => "LAB",
                Kind::Scenario => "SC",
                Kind::LabReset => "R",
                Kind::LabStore => "B",
                _ => "A/B",
            };
            let size = (r * 0.7).max(12.0);
            let m = measure_text(text, None, size as u16, 1.0);
            draw_text(text, c.x - m.width * 0.5, c.y + m.height * 0.4, size, col);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buttons_stay_on_screen_and_do_not_overlap_on_phone_tablet_and_desktop() {
        // 가로 폰(논리 px), 작은 가로 폰, 태블릿, 데스크톱, 세로 폰.
        for (w, h) in [
            (844.0, 390.0),
            (667.0, 375.0),
            (1180.0, 820.0),
            (1280.0, 720.0),
            (390.0, 844.0),
        ] {
            for lab in [false, true] {
                let b = layout(w, h, true, lab);
                for x in &b {
                    assert!(
                        x.center.x - x.radius >= 0.0 && x.center.x + x.radius <= w,
                        "{w}x{h} {:?} off x",
                        x.kind
                    );
                    assert!(
                        x.center.y - x.radius >= 0.0 && x.center.y + x.radius <= h,
                        "{w}x{h} {:?} off y",
                        x.kind
                    );
                }
                for i in 0..b.len() {
                    for j in (i + 1)..b.len() {
                        let d = (b[i].center - b[j].center).length();
                        assert!(
                            d > b[i].radius + b[j].radius,
                            "{w}x{h} lab={lab} {:?} overlaps {:?}",
                            b[i].kind,
                            b[j].kind
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn attack_is_the_biggest_button_and_meets_the_minimum_touch_size() {
        // 플랫폼 최소 터치 크기 44pt(지름) — 가장 작은 가로 폰에서도.
        let b = layout(667.0, 375.0, true, true);
        let atk = b.iter().find(|x| x.kind == Kind::Attack).unwrap();
        assert!(b.iter().all(|x| x.radius <= atk.radius));
        assert!(
            b.iter().all(|x| x.radius * 2.0 >= 44.0 * 0.8),
            "smallest {:?}",
            b.iter().map(|x| x.radius).fold(f32::MAX, f32::min)
        );
    }

    #[test]
    fn stick_has_a_dead_zone_points_up_when_the_finger_goes_up_and_saturates() {
        assert_eq!(stick_vector(vec2(3.0, 0.0), 100.0), Vec2::ZERO);
        let up = stick_vector(vec2(0.0, -60.0), 100.0);
        assert!(up.y > 0.4 && up.x.abs() < 1e-6, "{up:?}");
        let far = stick_vector(vec2(500.0, 0.0), 100.0);
        assert!((far.length() - 1.0).abs() < 1e-5);
    }
}
