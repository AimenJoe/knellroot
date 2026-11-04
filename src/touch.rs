//! 터치 조작(nail `input.rs` read_touch·TouchOverlay에서 가져옴, 0006). `input.rs`만 이 모듈을 부른다.
//!
//! - 화면 왼쪽 반에서 시작한 손가락 = 떠다니는 가상 스틱(손가락을 댄 자리가 중심).
//! - 오른쪽 아래 버튼: 공격(가장 크게)·회피(왼쪽 아래)·락온(위), 락온 왼쪽에 땅 돌리기 왼쪽·오른쪽(0012·0017 — 누르는 쪽으로 카메라가 돈다).
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
    /// 패링(0025): 방패 그림, 공격 왼쪽 위.
    Parry,
    /// 점프(0027): 위 화살표, 회피 왼쪽.
    Jump,
    /// 등잔(0031): 등잔 그림, 점프 위(돌리기 줄과 점프 사이).
    Flask,
    /// 마법(0048): 별 그림, 공격 위 호. 터치는 조준 스틱이 없어 자동 조준.
    Cast,
    /// 그림만(0053): 쉼터 곁의 점프 버튼에 그리는 모닥불.
    Rest,
    Lock,
    /// 땅 돌리기(0023): 누르기 전 캐릭터의 오른쪽(왼쪽) 면이 새 정면.
    TurnLeft,
    TurnRight,
    /// 멈춤(0019): 오른쪽 위.
    Pause,
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
    /// 쉼터 곁(0053): 점프 버튼에 모닥불을 그린다.
    pub rest: bool,
}

/// 터치가 이번 프레임에 만든 입력.
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Read {
    pub move_dir: Vec2,
    pub attack: bool,
    /// 공격 버튼을 짚고 있나(강공격 모으기, 0030).
    pub attack_held: bool,
    pub dodge: bool,
    pub parry: bool,
    pub jump: bool,
    /// 등잔(0031).
    pub flask: bool,
    /// 상호작용(0032).
    pub interact: bool,
    pub cast: bool,
    pub lock: bool,
    pub turn_right: bool,
    pub turn_left: bool,
    pub lab_toggle: bool,
    pub scenario_next: bool,
    pub lab_up: bool,
    pub lab_down: bool,
    pub lab_left: bool,
    pub lab_right: bool,
    pub lab_reset: bool,
    pub lab_store: bool,
    pub lab_swap: bool,
    pub pause: bool,
    /// 메뉴 모드에서 손가락을 거의 움직이지 않고 뗀 자리(논리 px) — 메뉴는 뗄 때 누른다(nail 0199).
    pub tap: Option<Vec2>,
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
    /// 공격 버튼을 짚은 손가락(강공격 모으기, 0030).
    attack_finger: Option<u64>,
    /// 상황 버튼(0053, 앱이 매 프레임 정한다).
    pub ctx: Ctx,
    /// 메뉴 모드: 손가락마다 (id, 처음 자리, 멀리 움직였나).
    taps: Vec<(u64, Vec2, bool)>,
}

/// 버튼 크기·거리의 기준(짧은 변)의 상한(논리 px) — 태블릿에서 버튼이 너무 커지지 않게.
const TOUCH_U_MAX: f32 = 520.0;

/// 이보다 덜 움직이고 떼면 탭(논리 px).
const TAP_SLOP: f32 = 24.0;

/// 상황 버튼(0053): 쓸 수 있을 때만 정해 둔 자리에 뜬다 — 다른 버튼을 밀지 않는다(Apple HIG Game controls: 쓸 수 없으면 숨겨 어수선함을 줄인다).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Ctx {
    /// 쉼터 곁이고 안전하다 — 점프 버튼이 모닥불 그림이 되고 쉬기를 한다(0048: 점프 = 상호작용).
    pub rest: bool,
    /// 락온할 적이 가까이 있거나 이미 락온 중.
    pub lock: bool,
    /// 마실 수 있다(체력이 가득이 아니고 등잔이 남았다).
    pub flask: bool,
}

impl Ctx {
    #[cfg(test)]
    pub const ALL: Ctx = Ctx {
        rest: true,
        lock: true,
        flask: true,
    };
}

/// 버튼 배치(0053, `docs/decisions/0053`). `w`·`h`는 논리 px, 기준은 짧은 변 `u`(태블릿에서 너무 커지지 않게 상한).
/// 오른손 엄지가 쉬는 오른쪽 아래에 공격(지름 ≈ 0.27u), 그 둘레 첫째 호(거리 0.26u)에 자주 쓰는 넷 — 회피·패링·마법·점프 —
/// 둘째 호(0.44u)에 돌리기 캡슐과 상황 버튼(락온·등잔). 각도는 공격 가운데에서 180° = 왼쪽, 90° = 위.
pub fn layout(w: f32, h: f32, dev: bool, lab_open: bool, ctx: Ctx) -> Vec<Button> {
    let u = w.min(h).min(TOUCH_U_MAX);
    let r = u * 0.09;
    let atk = vec2(w - u * 0.238, h - u * 0.21);
    let at = |deg: f32, dist: f32| {
        let a = deg.to_radians();
        atk + vec2(a.cos(), -a.sin()) * (u * dist)
    };
    let mut b = vec![
        Button {
            center: atk,
            radius: u * 0.135,
            kind: Kind::Attack,
        },
        Button {
            center: at(188.0, 0.26),
            radius: u * 0.085,
            kind: Kind::Dodge,
        },
        Button {
            center: at(146.0, 0.26),
            radius: u * 0.0775,
            kind: Kind::Parry,
        },
        Button {
            center: at(104.0, 0.26),
            radius: u * 0.0775,
            kind: Kind::Cast,
        },
        Button {
            center: at(62.0, 0.26),
            radius: u * 0.0725,
            kind: Kind::Jump,
        },
    ];
    // 땅 돌리기: 둘을 한 캡슐로(왼쪽 반 = 왼쪽으로, 오른쪽 반 = 오른쪽으로) — 그림이 캡슐 테두리로 묶는다.
    let turn = at(118.0, 0.44);
    let tr = u * 0.065;
    b.push(Button {
        center: turn - vec2(u * 0.08, 0.0),
        radius: tr,
        kind: Kind::TurnLeft,
    });
    b.push(Button {
        center: turn + vec2(u * 0.08, 0.0),
        radius: tr,
        kind: Kind::TurnRight,
    });
    if ctx.lock {
        b.push(Button {
            center: at(80.0, 0.44),
            radius: u * 0.065,
            kind: Kind::Lock,
        });
    }
    if ctx.flask {
        b.push(Button {
            center: at(150.0, 0.44),
            radius: u * 0.0625,
            kind: Kind::Flask,
        });
    }
    // 멈춤: 오른쪽 위 끝(개발 버튼은 그 왼쪽으로).
    let s = r * 0.7;
    b.push(Button {
        center: vec2(w - s * 1.4, s * 1.4),
        radius: s,
        kind: Kind::Pause,
    });
    if dev {
        b.push(Button {
            center: vec2(w - s * 4.0, s * 1.4),
            radius: s,
            kind: Kind::Lab,
        });
        b.push(Button {
            center: vec2(w - s * 6.6, s * 1.4),
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
    w.min(h).min(TOUCH_U_MAX) * 0.21
}

pub fn lab_button_radius(w: f32, h: f32) -> f32 {
    w.min(h).min(TOUCH_U_MAX) * 0.09 * 0.62
}

/// 오른쪽 위 dev 버튼(시나리오·랩)이 차지하는 폭 — 랩 글이 그 앞에서 멈춘다.
pub fn dev_buttons_width(w: f32, h: f32) -> f32 {
    let s = w.min(h) * 0.09 * 0.7;
    s * 7.6 + 8.0
}

pub fn stick_radius(w: f32, h: f32) -> f32 {
    w.min(h) * 0.14
}

impl Touch {
    /// `menu`면 스틱·버튼 없이 탭만(메뉴 화면).
    pub fn read(&mut self, dt: f32, dev: bool, lab_open: bool, menu: bool) -> Read {
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
        if menu {
            self.stick = None;
            self.repeat = None;
            self.attack_finger = None;
            self.known.clear();
            for t in &touches {
                let ended = matches!(t.phase, TouchPhase::Ended | TouchPhase::Cancelled);
                match self.taps.iter_mut().find(|x| x.0 == t.id) {
                    Some(x) => {
                        x.2 |= (t.position - x.1).length() > TAP_SLOP;
                        if ended {
                            if !x.2 {
                                a.tap = Some(t.position);
                            }
                            let id = t.id;
                            self.taps.retain(|x| x.0 != id);
                        }
                    }
                    None if !ended => self.taps.push((t.id, t.position, false)),
                    None => a.tap = Some(t.position),
                }
            }
            self.taps.retain(|x| touches.iter().any(|t| t.id == x.0));
            return a;
        }
        self.taps.clear();
        if touches.is_empty() && self.stick.is_none() && self.known.is_empty() {
            self.repeat = None;
            self.attack_finger = None;
            return a;
        }
        let (w, h) = (screen_width(), screen_height());
        let buttons = layout(w, h, dev, lab_open, self.ctx);
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
                    if b.kind == Kind::Attack && !ended {
                        self.attack_finger = Some(t.id);
                    }
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
        // 공격 손가락이 아직 화면에 있으면 쥐고 있는 것(버튼 밖으로 미끄러져도 — 떼야 강타가 나간다).
        self.attack_finger = self.attack_finger.filter(|id| {
            touches.iter().any(|t| {
                t.id == *id && !matches!(t.phase, TouchPhase::Ended | TouchPhase::Cancelled)
            })
        });
        a.attack_held = self.attack_finger.is_some();
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
            buttons: layout(w, h, dev, lab_open, self.ctx),
            rest: self.ctx.rest,
        }
    }
}

fn press(a: &mut Read, k: Kind) {
    match k {
        Kind::Attack => a.attack = true,
        Kind::Dodge => a.dodge = true,
        Kind::Parry => a.parry = true,
        Kind::Jump => a.jump = true,
        Kind::Flask => a.flask = true,
        Kind::Cast => a.cast = true,
        Kind::Rest => a.jump = true,
        Kind::Lock => a.lock = true,
        Kind::TurnLeft => a.turn_left = true,
        Kind::TurnRight => a.turn_right = true,
        Kind::Pause => a.pause = true,
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
    // 돌리기 둘을 한 캡슐로 묶는 테(0053).
    let tl = o.buttons.iter().find(|b| b.kind == Kind::TurnLeft);
    let trn = o.buttons.iter().find(|b| b.kind == Kind::TurnRight);
    if let (Some(l), Some(r)) = (tl, trn) {
        let pad = l.radius * 0.25;
        let rr = l.radius + pad;
        draw_rectangle(
            l.center.x,
            l.center.y - rr,
            r.center.x - l.center.x,
            rr * 2.0,
            Color::new(1.0, 1.0, 1.0, 0.06),
        );
        draw_line(
            l.center.x,
            l.center.y - rr,
            r.center.x,
            r.center.y - rr,
            2.0,
            line,
        );
        draw_line(
            l.center.x,
            l.center.y + rr,
            r.center.x,
            r.center.y + rr,
            2.0,
            line,
        );
        draw_arc_left_right(l.center, r.center, rr, line);
    }
    for b in &o.buttons {
        let on = b.kind == Kind::Lock && locked;
        let f = if on {
            Color::new(1.0, 0.85, 0.3, 0.35)
        } else {
            fill
        };
        let is_turn = matches!(b.kind, Kind::TurnLeft | Kind::TurnRight);
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
        let kind = if b.kind == Kind::Jump && o.rest {
            Kind::Rest
        } else {
            b.kind
        };
        icon(kind, b.center, b.radius, dim);
    }
}

/// 캡슐 양끝의 반원 테(선 조각으로).
fn draw_arc_left_right(l: Vec2, r: Vec2, rr: f32, col: Color) {
    let n = 10;
    for (c, from) in [(l, 90.0f32), (r, -90.0f32)] {
        for i in 0..n {
            let a0 = (from + 180.0 * i as f32 / n as f32).to_radians();
            let a1 = (from + 180.0 * (i + 1) as f32 / n as f32).to_radians();
            draw_line(
                c.x + rr * a0.cos(),
                c.y - rr * a0.sin(),
                c.x + rr * a1.cos(),
                c.y - rr * a1.sin(),
                2.0,
                col,
            );
        }
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
        Kind::Jump => {
            // 위 화살표 + 바닥 줄(뛰어오르기).
            draw_line(c.x, c.y + s * 0.5, c.x, c.y - s * 0.7, th, col);
            draw_line(c.x - s * 0.5, c.y - s * 0.2, c.x, c.y - s * 0.7, th, col);
            draw_line(c.x + s * 0.5, c.y - s * 0.2, c.x, c.y - s * 0.7, th, col);
            draw_line(
                c.x - s * 0.7,
                c.y + s * 0.8,
                c.x + s * 0.7,
                c.y + s * 0.8,
                th,
                col,
            );
        }
        Kind::Flask => {
            // 등잔: 둥근 몸 + 목 + 불꽃.
            let body = [
                vec2(-0.6, 0.15),
                vec2(0.6, 0.15),
                vec2(0.45, 0.8),
                vec2(-0.45, 0.8),
            ];
            for i in 0..body.len() {
                let (p, q) = (c + body[i] * s, c + body[(i + 1) % body.len()] * s);
                draw_line(p.x, p.y, q.x, q.y, th, col);
            }
            draw_line(
                c.x - s * 0.2,
                c.y + s * 0.15,
                c.x - s * 0.2,
                c.y - s * 0.1,
                th,
                col,
            );
            draw_line(
                c.x + s * 0.2,
                c.y + s * 0.15,
                c.x + s * 0.2,
                c.y - s * 0.1,
                th,
                col,
            );
            draw_line(c.x, c.y - s * 0.3, c.x - s * 0.22, c.y - s * 0.55, th, col);
            draw_line(c.x - s * 0.22, c.y - s * 0.55, c.x, c.y - s * 0.85, th, col);
            draw_line(c.x, c.y - s * 0.85, c.x + s * 0.22, c.y - s * 0.55, th, col);
            draw_line(c.x + s * 0.22, c.y - s * 0.55, c.x, c.y - s * 0.3, th, col);
        }
        Kind::Cast => {
            // 마법: 네 갈래 별(가는 마름모 둘).
            let pts = [
                vec2(0.0, -0.9),
                vec2(0.22, -0.22),
                vec2(0.9, 0.0),
                vec2(0.22, 0.22),
                vec2(0.0, 0.9),
                vec2(-0.22, 0.22),
                vec2(-0.9, 0.0),
                vec2(-0.22, -0.22),
            ];
            for i in 0..pts.len() {
                let (p, q) = (c + pts[i] * s, c + pts[(i + 1) % pts.len()] * s);
                draw_line(p.x, p.y, q.x, q.y, th, col);
            }
        }
        Kind::Parry => {
            // 방패: 위가 넓고 아래가 뾰족한 오각형 윤곽.
            let pts = [
                vec2(-0.7, -0.75),
                vec2(0.7, -0.75),
                vec2(0.7, 0.05),
                vec2(0.0, 0.85),
                vec2(-0.7, 0.05),
            ];
            for i in 0..pts.len() {
                let (p, q) = (c + pts[i] * s, c + pts[(i + 1) % pts.len()] * s);
                draw_line(p.x, p.y, q.x, q.y, th, col);
            }
        }
        Kind::Rest => {
            // 모닥불: 엇갈린 장작 둘 + 불꽃.
            draw_line(
                c.x - s * 0.7,
                c.y + s * 0.75,
                c.x + s * 0.7,
                c.y + s * 0.35,
                th,
                col,
            );
            draw_line(
                c.x + s * 0.7,
                c.y + s * 0.75,
                c.x - s * 0.7,
                c.y + s * 0.35,
                th,
                col,
            );
            draw_line(c.x, c.y + s * 0.25, c.x - s * 0.4, c.y - s * 0.2, th, col);
            draw_line(c.x - s * 0.4, c.y - s * 0.2, c.x, c.y - s * 0.85, th, col);
            draw_line(c.x, c.y - s * 0.85, c.x + s * 0.4, c.y - s * 0.2, th, col);
            draw_line(c.x + s * 0.4, c.y - s * 0.2, c.x, c.y + s * 0.25, th, col);
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
        Kind::TurnLeft | Kind::TurnRight => {
            // 위쪽 반원 화살표(↶ ↷): 반대쪽 끝에서 위를 지나 누르는 쪽 끝으로, 화살촉은 아래로 — 먼 쪽이 그쪽으로 = 카메라가 그쪽으로 돈다(0017).
            let side = if k == Kind::TurnRight { 1.0 } else { -1.0 };
            let rad = s * 0.75;
            let n = 14;
            let at = |a: f32| c + vec2(a.cos() * side, -a.sin()) * rad + vec2(0.0, rad * 0.35);
            for i in 0..n {
                let (u, v) = (
                    std::f32::consts::PI * (1.0 - i as f32 / n as f32),
                    std::f32::consts::PI * (1.0 - (i + 1) as f32 / n as f32),
                );
                let (p, q) = (at(u), at(v));
                draw_line(p.x, p.y, q.x, q.y, th, col);
            }
            let tip = at(0.0) + vec2(0.0, s * 0.15);
            let h = s * 0.45;
            draw_triangle(
                tip + vec2(0.0, h * 0.6),
                tip + vec2(-h * 0.55, -h * 0.4),
                tip + vec2(h * 0.55, -h * 0.4),
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
        Kind::Pause => {
            // 두 줄(멈춤).
            for dx in [-s * 0.3, s * 0.3] {
                draw_line(
                    c.x + dx,
                    c.y - s * 0.55,
                    c.x + dx,
                    c.y + s * 0.55,
                    th * 1.6,
                    col,
                );
            }
        }
        Kind::Lab | Kind::Scenario | Kind::LabReset | Kind::LabStore | Kind::LabSwap => {
            let text = match k {
                Kind::Lab => "LAB",
                Kind::Scenario => "SC",
                Kind::LabReset => "R",
                Kind::LabStore => "B",
                _ => "A/B",
            };
            let size = crate::text::fit((r * 0.7).max(12.0));
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
                let b = layout(w, h, true, lab, Ctx::ALL);
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
        let b = layout(667.0, 375.0, true, true, Ctx::ALL);
        let atk = b.iter().find(|x| x.kind == Kind::Attack).unwrap();
        assert!(b.iter().all(|x| x.radius <= atk.radius));
        assert!(
            b.iter().all(|x| x.radius * 2.0 >= 44.0 * 0.8),
            "smallest {:?}",
            b.iter().map(|x| x.radius).fold(f32::MAX, f32::min)
        );
    }

    /// 0053: 상황 버튼이 켜지고 꺼져도 늘 있는 버튼의 자리는 그대로다(엄지가 외운 자리가 밀리지 않게).
    #[test]
    fn contextual_buttons_never_move_the_others() {
        let all = layout(844.0, 390.0, false, false, Ctx::ALL);
        let none = layout(844.0, 390.0, false, false, Ctx::default());
        for x in &none {
            let y = all.iter().find(|y| y.kind == x.kind).unwrap();
            assert_eq!(x.center, y.center, "{:?}", x.kind);
        }
        assert!(!none
            .iter()
            .any(|x| matches!(x.kind, Kind::Lock | Kind::Flask)));
        assert_eq!(
            none.iter().filter(|x| x.kind != Kind::Pause).count(),
            7,
            "attack + 4 + turn pair"
        );
    }

    /// 오른쪽 버튼 = 카메라가 오른쪽으로 = 땅이 화면에서 시계 방향(0017) — 카메라 쪽 뜻은 camera 시험이 본다.
    #[test]
    fn the_right_turn_button_swings_the_camera_right() {
        let mut a = Read::default();
        press(&mut a, Kind::TurnRight);
        assert!(a.turn_right && !a.turn_left);
        let mut a = Read::default();
        press(&mut a, Kind::TurnLeft);
        assert!(a.turn_left && !a.turn_right);
        // 오른쪽 버튼이 화면 오른쪽에 있다.
        let b = layout(844.0, 390.0, false, false, Ctx::default());
        let x = |k| b.iter().find(|x| x.kind == k).unwrap().center.x;
        assert!(x(Kind::TurnRight) > x(Kind::TurnLeft));
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
