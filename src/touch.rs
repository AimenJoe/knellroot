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
    /// 그림만(0147): 공격 버튼에 그리는 든 무기 — 검은 `Attack`의 칼 그림 그대로(`attack_icon`).
    Hammer,
    Staff,
    Crossbow,
    Fist,
    Lock,
    /// 땅 돌리기(0023): 누르기 전 캐릭터의 오른쪽(왼쪽) 면이 새 정면.
    TurnLeft,
    TurnRight,
    /// 스킬 버튼 둘(0151): 예전 돌리기 캡슐 자리. 그림은 끼운 스킬의 것(`View::skills`).
    Skill1,
    Skill2,
    /// 큰 기술(0151): 게이지가 차면 스킬 둘 위에 뜬다(상황 버튼).
    Ult,
    /// 그림만(0151): 스킬·큰 기술의 픽토그램, 원거리 무기의 RB(수류탄).
    Whirl,
    Cut,
    Gather,
    Quake,
    Upend,
    Bomb,
    /// 멈춤(0019): 왼쪽 위.
    Pause,
}

/// 그릴 때 앱이 넘기는 것(0151): 상태에 따라 바뀌는 그림·차오름.
#[derive(Clone, Copy, Debug)]
pub struct View {
    pub locked: bool,
    /// 땅 돌리기 대기(0..1, 1 = 준비).
    pub turn: f32,
    /// 공격 버튼의 그림(든 무기, 0147)과 RB 버튼의 그림(근접 = 별, 원거리 = 수류탄).
    pub attack: Kind,
    pub cast: Kind,
    /// 스킬 둘: (그림, 대기 0..1 — 1 = 준비).
    pub skills: [(Kind, f32); 2],
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
    /// 회피 버튼을 쥐고 있나(0076 — 쥐면 회피 뒤 달리기).
    pub dodge_held: bool,
    pub parry: bool,
    pub jump: bool,
    /// 등잔(0031).
    pub flask: bool,
    /// 상호작용(0032).
    pub interact: bool,
    pub cast: bool,
    pub cast_held: bool,
    pub lock: bool,
    pub turn_right: bool,
    pub turn_left: bool,
    pub skill1: bool,
    pub skill2: bool,
    pub ult: bool,
    pub pause: bool,
    /// 메뉴 모드에서 손가락을 거의 움직이지 않고 뗀 자리(논리 px) — 메뉴는 뗄 때 누른다(nail 0199).
    pub tap: Option<Vec2>,
    /// 메뉴 모드에서 손가락을 탭 범위 넘게 움직였으면 이번 프레임에 끈 거리(논리 px, 위로 끌면 + — 아래 줄이 드러난다). 끈 손가락은 뗄 때 탭이 아니다.
    pub drag: f32,
    /// 끈 손가락이 처음 닿은 자리(논리 px) — 무엇을 굴리나(목록·설명 칸, 0068).
    pub drag_at: Option<Vec2>,
    /// 화면에 손가락이 있었다(버튼·스틱이 아니어도) → 장치 = 터치.
    pub active: bool,
}

#[derive(Clone, Copy)]
struct Stick {
    id: u64,
    origin: Vec2,
    pos: Vec2,
}

/// 스틱 데드존(반지름 대비).
const DEAD: f32 = 0.15;

#[derive(Default)]
pub struct Touch {
    stick: Option<Stick>,
    known: Vec<u64>,
    /// 공격 버튼을 짚은 손가락(강공격 모으기, 0030).
    attack_finger: Option<u64>,
    /// 마법 버튼을 짚은 손가락(0056 — 쥐면 모으기).
    cast_finger: Option<u64>,
    /// 회피 버튼을 누른 손가락(0076).
    dodge_finger: Option<u64>,
    /// 상황 버튼(0053, 앱이 매 프레임 정한다).
    pub ctx: Ctx,
    /// 메뉴 모드: 손가락마다 (id, 처음 자리, 멀리 움직였나 = 끌기, 지난 자리).
    taps: Vec<(u64, Vec2, bool, Vec2)>,
    /// 싸움 중에 닿아 있던 손가락(멈춤 버튼을 누른 손가락 등) — 메뉴가 열린 뒤 떼도 탭이 아니다(2026-11-06: 멈춤 버튼을 뗀 자리가
    /// "창 밖 탭"이 되어 멈춤 창이 바로 닫혔다).
    stale: Vec<u64>,
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
    /// 큰 기술 게이지가 가득 찼다(0151).
    pub ult: bool,
}

impl Ctx {
    #[cfg(test)]
    pub const ALL: Ctx = Ctx {
        rest: true,
        lock: true,
        flask: true,
        ult: true,
    };
}

/// 버튼 배치(0053, `docs/decisions/0053`). `w`·`h`는 논리 px, 기준은 짧은 변 `u`(태블릿에서 너무 커지지 않게 상한).
/// 오른손 엄지가 쉬는 오른쪽 아래에 공격(지름 ≈ 0.27u), 그 둘레 첫째 호(거리 0.26u)에 자주 쓰는 넷 — 회피·패링·마법·점프 —
/// 둘째 호(0.44u)에 돌리기 캡슐과 상황 버튼(락온·등잔). 각도는 공격 가운데에서 180° = 왼쪽, 90° = 위.
pub fn layout(w: f32, h: f32, ctx: Ctx) -> Vec<Button> {
    let u = w.min(h).min(TOUCH_U_MAX);
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
    // 스킬 둘(0151): 예전 돌리기 캡슐 자리(공격 위 바깥 호) — 엄지가 외운 자리를 스킬이 물려받는다.
    let skill = at(118.0, 0.44);
    let sr = u * 0.065;
    b.push(Button {
        center: skill - vec2(u * 0.08, 0.0),
        radius: sr,
        kind: Kind::Skill1,
    });
    b.push(Button {
        center: skill + vec2(u * 0.08, 0.0),
        radius: sr,
        kind: Kind::Skill2,
    });
    // 땅 돌리기(0151): 자주 누르지 않으니 엄지 호 밖, 오른쪽 위에 작게(왼쪽 위의 멈춤과 마주 본다).
    let (pc, ps) = pause_spot(w, h);
    b.push(Button {
        center: vec2(w - pc.x - ps * 2.5, pc.y),
        radius: ps,
        kind: Kind::TurnLeft,
    });
    b.push(Button {
        center: vec2(w - pc.x, pc.y),
        radius: ps,
        kind: Kind::TurnRight,
    });
    if ctx.ult {
        b.push(Button {
            center: at(118.0, 0.6),
            radius: u * 0.06,
            kind: Kind::Ult,
        });
    }
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
    // 멈춤: 왼쪽 위 끝(2026-11-06 — 사장, 메뉴 버튼은 왼쪽 위). HUD는 그만큼 오른쪽으로 비킨다(`hud_inset`).
    // 랩·시나리오 터치 버튼은 걷었다 — 랩은 멈춤 메뉴의 LAB, 시나리오는 DEV 메뉴에서.
    let (c, s) = pause_spot(w, h);
    b.push(Button {
        center: c,
        radius: s,
        kind: Kind::Pause,
    });
    b
}

/// 멈춤 버튼의 가운데와 반지름(논리 px).
fn pause_spot(w: f32, h: f32) -> (Vec2, f32) {
    let s = w.min(h).min(TOUCH_U_MAX) * 0.09 * 0.7;
    (vec2(s * 1.4, s * 1.4), s)
}

/// 터치일 때 HUD가 왼쪽 위 멈춤 버튼을 비키는 폭(논리 px).
pub fn hud_inset(w: f32, h: f32) -> f32 {
    let (c, s) = pause_spot(w, h);
    c.x + s + 6.0
}

pub fn stick_radius(w: f32, h: f32) -> f32 {
    w.min(h) * 0.14
}

impl Touch {
    /// `menu`면 스틱·버튼 없이 탭만(메뉴 화면).
    pub fn read(&mut self, menu: bool) -> Read {
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
            self.attack_finger = None;
            self.cast_finger = None;
            self.dodge_finger = None;
            self.stale.append(&mut self.known);
            for t in &touches {
                let ended = matches!(t.phase, TouchPhase::Ended | TouchPhase::Cancelled);
                if self.stale.contains(&t.id) {
                    continue;
                }
                match self.taps.iter_mut().find(|x| x.0 == t.id) {
                    Some(x) => {
                        let was = x.2;
                        x.2 |= (t.position - x.1).length() > TAP_SLOP;
                        // 끌기는 탭 범위를 넘은 뒤부터 — 넘는 순간엔 처음 자리부터 센다(손가락과 목록이 같이 간다).
                        if x.2 {
                            let from = if was { x.3.y } else { x.1.y };
                            a.drag += from - t.position.y;
                            a.drag_at = Some(x.1);
                        }
                        x.3 = t.position;
                        if ended {
                            if !x.2 {
                                a.tap = Some(t.position);
                            }
                            let id = t.id;
                            self.taps.retain(|x| x.0 != id);
                        }
                    }
                    None if !ended => self.taps.push((t.id, t.position, false, t.position)),
                    None => a.tap = Some(t.position),
                }
            }
            self.taps.retain(|x| touches.iter().any(|t| t.id == x.0));
            // 뗀 손가락은 묵은 목록에서 뺀다(같은 id가 다시 쓰여도 새 탭은 받게).
            self.stale.retain(|id| {
                touches.iter().any(|t| {
                    t.id == *id && !matches!(t.phase, TouchPhase::Ended | TouchPhase::Cancelled)
                })
            });
            return a;
        }
        self.taps.clear();
        self.stale.clear();
        if touches.is_empty() && self.stick.is_none() && self.known.is_empty() {
            self.attack_finger = None;
            self.cast_finger = None;
            self.dodge_finger = None;
            return a;
        }
        let (w, h) = (screen_width(), screen_height());
        let buttons = layout(w, h, self.ctx);
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
                    if b.kind == Kind::Cast && !ended {
                        self.cast_finger = Some(t.id);
                    }
                    if b.kind == Kind::Dodge && !ended {
                        self.dodge_finger = Some(t.id);
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
            }
        }
        // Ended를 못 받고 사라진 손가락(탭 전환 등) 정리.
        self.known.retain(|id| touches.iter().any(|t| t.id == *id));
        // 버튼을 누른 손가락이 아직 화면에 있으면 쥐고 있는 것(버튼 밖으로 미끄러져도 — 떼야 강타·모은 탄이 나간다).
        // 0070 7(`995c189`)에서 랩 줄과 함께 지워져 터치로는 강공격·원거리 모으기가 안 됐다(0076에서 되살림).
        let alive = |f: Option<u64>| {
            f.filter(|id| {
                touches.iter().any(|t| {
                    t.id == *id && !matches!(t.phase, TouchPhase::Ended | TouchPhase::Cancelled)
                })
            })
        };
        self.attack_finger = alive(self.attack_finger);
        self.cast_finger = alive(self.cast_finger);
        self.dodge_finger = alive(self.dodge_finger);
        a.attack_held = self.attack_finger.is_some();
        a.cast_held = self.cast_finger.is_some();
        a.dodge_held = self.dodge_finger.is_some();
        if self
            .stick
            .is_some_and(|s| !touches.iter().any(|t| t.id == s.id))
        {
            self.stick = None;
        }
        if let Some(s) = self.stick {
            a.move_dir = stick_vector(s.pos - s.origin, stick_radius(w, h));
        }
        a
    }

    pub fn overlay(&self) -> Overlay {
        let (w, h) = (screen_width(), screen_height());
        Overlay {
            stick: self.stick.map(|s| (s.origin, s.pos)),
            stick_radius: stick_radius(w, h),
            buttons: layout(w, h, self.ctx),
            rest: self.ctx.rest,
        }
    }
}

fn press(a: &mut Read, k: Kind) {
    match k {
        Kind::Attack | Kind::Hammer | Kind::Staff | Kind::Crossbow | Kind::Fist => a.attack = true,
        Kind::Dodge => a.dodge = true,
        Kind::Parry => a.parry = true,
        Kind::Jump => a.jump = true,
        Kind::Flask => a.flask = true,
        Kind::Cast => a.cast = true,
        Kind::Rest => a.jump = true,
        Kind::Lock => a.lock = true,
        Kind::TurnLeft => a.turn_left = true,
        Kind::TurnRight => a.turn_right = true,
        Kind::Skill1 => a.skill1 = true,
        Kind::Skill2 => a.skill2 = true,
        Kind::Ult | Kind::Upend => a.ult = true,
        // 그림만인 것은 버튼이 되지 않는다.
        Kind::Whirl | Kind::Cut | Kind::Gather | Kind::Quake | Kind::Bomb => {}
        Kind::Pause => a.pause = true,
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

/// 터치 UI의 잉크(어두운 바탕·테)·따뜻한 강조(공격 테·락온) — HUD의 호박빛과 같은 줄기.
const INK: Color = Color::new(0.07, 0.05, 0.1, 1.0);
const WARM: Color = Color::new(1.0, 0.8, 0.42, 1.0);

/// 공격 버튼에 그릴 그림(0147 — 사장 "어차피 종류는 정해졌으니까"): 든 무기마다 하나. `_` 없는 match라 새 무기는 여기서 그림을 고르게 된다.
pub fn attack_icon(w: crate::weapons::Id) -> Kind {
    use crate::weapons::Id;
    match w {
        Id::Sword => Kind::Attack,
        Id::Hammer => Kind::Hammer,
        Id::Staff => Kind::Staff,
        Id::Crossbow => Kind::Crossbow,
        Id::Fists => Kind::Fist,
    }
}

/// 터치 오버레이 그리기(화면 패스). `attack` = 공격 버튼의 그림(`attack_icon`). `locked`면 락온 버튼을 채운다. `turn` = 땅 돌리기 준비 0..1 — 차오르는 동안 옅고, 다 차면 밝다.
/// 버튼의 몸(0146): 어두운 반투명 바탕 + 밝은 테 — 흰 반투명만으로는 밝은 바닥·불빛 위에서 그림이 묻혔다. 공격만 굵은 호박빛 테
/// (가장 큰 버튼이 가장 먼저 읽히게), 나머지는 흰 테.
/// 스킬 버튼의 그림(0151) — 새 스킬 = 그림 하나(`_` 없는 match).
pub fn skill_icon(id: crate::skills::Id) -> Kind {
    use crate::skills::Id;
    match id {
        Id::Whirl => Kind::Whirl,
        Id::Cut => Kind::Cut,
        Id::Gather => Kind::Gather,
        Id::Quake => Kind::Quake,
    }
}

/// RB 버튼의 그림(0151): 근접 무기 = 마법 탄의 별, 원거리 무기 = 수류탄.
pub fn cast_icon(w: crate::weapons::Id) -> Kind {
    if w.def().grenade.is_some() {
        Kind::Bomb
    } else {
        Kind::Cast
    }
}

pub fn draw(o: &Overlay, v: &View) {
    let (locked, turn, attack) = (v.locked, v.turn, v.attack);
    let line = Color::new(1.0, 1.0, 1.0, 0.5);
    let back = Color::new(INK.r, INK.g, INK.b, 0.42);
    if let Some((origin, pos)) = o.stick {
        draw_circle(
            origin.x,
            origin.y,
            o.stick_radius,
            Color::new(INK.r, INK.g, INK.b, 0.22),
        );
        draw_circle_lines(origin.x, origin.y, o.stick_radius, 2.0, line);
        let knob = origin + (pos - origin).clamp_length_max(o.stick_radius);
        draw_circle(knob.x, knob.y, o.stick_radius * 0.4, back);
        draw_circle(
            knob.x,
            knob.y,
            o.stick_radius * 0.4,
            Color::new(1.0, 1.0, 1.0, 0.18),
        );
        draw_circle_lines(knob.x, knob.y, o.stick_radius * 0.4, 2.0, line);
    }
    // 돌리기 둘을 한 캡슐로 묶는 테(0053).
    let tl = o.buttons.iter().find(|b| b.kind == Kind::TurnLeft);
    let trn = o.buttons.iter().find(|b| b.kind == Kind::TurnRight);
    if let (Some(l), Some(r)) = (tl, trn) {
        let pad = l.radius * 0.25;
        let rr = l.radius + pad;
        let faint = Color::new(INK.r, INK.g, INK.b, 0.2);
        draw_rectangle(
            l.center.x,
            l.center.y - rr,
            r.center.x - l.center.x,
            rr * 2.0,
            faint,
        );
        let soft = Color::new(1.0, 1.0, 1.0, 0.3);
        draw_line(
            l.center.x,
            l.center.y - rr,
            r.center.x,
            r.center.y - rr,
            2.0,
            soft,
        );
        draw_line(
            l.center.x,
            l.center.y + rr,
            r.center.x,
            r.center.y + rr,
            2.0,
            soft,
        );
        draw_arc_left_right(l.center, r.center, rr, soft);
    }
    for b in &o.buttons {
        let on = b.kind == Kind::Lock && locked;
        let is_turn = matches!(b.kind, Kind::TurnLeft | Kind::TurnRight);
        draw_circle(b.center.x, b.center.y, b.radius, back);
        if on {
            draw_circle(
                b.center.x,
                b.center.y,
                b.radius,
                Color::new(WARM.r, WARM.g, WARM.b, 0.3),
            );
        }
        if is_turn && turn < 1.0 {
            // 대기: 차오르는 몫만큼 가운데부터 채운다(그림은 옅게).
            draw_circle(
                b.center.x,
                b.center.y,
                b.radius * turn.clamp(0.0, 1.0),
                Color::new(1.0, 1.0, 1.0, 0.1),
            );
        }
        // 스킬(0151): 대기는 돌리기와 같은 꼴 — 가운데부터 차오르고 그림은 옅게.
        let skill = match b.kind {
            Kind::Skill1 => Some(v.skills[0]),
            Kind::Skill2 => Some(v.skills[1]),
            _ => None,
        };
        if let Some((_, ready)) = skill.filter(|(_, r)| *r < 1.0) {
            draw_circle(
                b.center.x,
                b.center.y,
                b.radius * ready.clamp(0.0, 1.0),
                Color::new(1.0, 1.0, 1.0, 0.1),
            );
        }
        let ult = b.kind == Kind::Ult;
        if ult {
            draw_circle(
                b.center.x,
                b.center.y,
                b.radius,
                Color::new(WARM.r, WARM.g, WARM.b, 0.3),
            );
        }
        let (ring, thick) = match b.kind {
            Kind::Attack => (Color::new(WARM.r, WARM.g, WARM.b, 0.8), 3.0),
            _ if on || ult => (Color::new(WARM.r, WARM.g, WARM.b, 0.9), 2.0),
            _ => (line, 2.0),
        };
        draw_circle_lines(b.center.x, b.center.y, b.radius, thick, ring);
        let cooling = (is_turn && turn < 1.0) || skill.is_some_and(|(_, r)| r < 1.0);
        let dim = if cooling { 0.4 } else { 1.0 };
        let kind = match b.kind {
            Kind::Jump if o.rest => Kind::Rest,
            Kind::Attack => attack,
            Kind::Cast => v.cast,
            Kind::Skill1 => v.skills[0].0,
            Kind::Skill2 => v.skills[1].0,
            Kind::Ult => Kind::Upend,
            k => k,
        };
        let tint = if on || ult || kind == Kind::Rest {
            Color::new(1.0, 0.9, 0.66, 1.0)
        } else {
            Color::new(1.0, 0.99, 0.95, 1.0)
        };
        icon(kind, b.center, b.radius, dim, tint);
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

/// 픽토그램의 한 변(칸).
pub const ICON_N: usize = 13;

/// 터치 픽토그램(0146·0147 — 사장 "터치스크린의 아이콘도 좀 거시기 하네"): 13×13 도트 그림(`#` = 칠한 칸). 가는 선 그림은 도트 화면 위에서
/// 다른 게임의 UI처럼 떠 보였다 — 첫 게임도 같은 판정을 도트 그림으로 풀었다(nail 0172 판정 11). 돌리기 둘은 한 그림을 좌우로 뒤집는다
/// (`icon_cell`). 고치면 시험 `touch_icons_are_13_by_13_and_tell_each_other_apart`.
pub fn icon_mask(k: Kind) -> [&'static str; ICON_N] {
    match k {
        Kind::Hammer => [
            "......##.....",
            ".....####....",
            "....######...",
            "...########..",
            "....########.",
            ".....########",
            "....#.######.",
            "...###.####..",
            "..###...##...",
            ".###.........",
            "###..........",
            "##...........",
            ".............",
        ],
        Kind::Staff => [
            ".........###.",
            "........#####",
            "........#####",
            "........#####",
            ".......#.###.",
            "......###....",
            ".....###.....",
            "....###......",
            "...###.......",
            "..###........",
            ".###.........",
            "###..........",
            "##...........",
        ],
        Kind::Crossbow => [
            "......#......",
            "..#########..",
            ".###..#..###.",
            "##....#....##",
            "#.#...#...#.#",
            "...#..#..#...",
            "....#.#.#....",
            ".....###.....",
            "......#......",
            ".....###.....",
            ".....###.....",
            ".....###.....",
            ".............",
        ],
        Kind::Fist => [
            ".............",
            "...##.##.##..",
            "..###.##.###.",
            "..##########.",
            ".###########.",
            "############.",
            "####.......#.",
            "####.#######.",
            ".###########.",
            ".###########.",
            "..#########..",
            "...#######...",
            ".............",
        ],
        Kind::Attack => [
            "..........###",
            ".........####",
            "........####.",
            ".......####..",
            "......####...",
            ".##..####....",
            ".###.###.....",
            "..#####......",
            "...####......",
            "..######.....",
            ".###..###....",
            "###....##....",
            "##...........",
        ],
        Kind::Dodge => [
            ".............",
            ".............",
            "###...###....",
            ".###...###...",
            "..###...###..",
            "...###...###.",
            "....###...###",
            "...###...###.",
            "..###...###..",
            ".###...###...",
            "###...###....",
            ".............",
            ".............",
        ],
        Kind::Parry => [
            ".............",
            ".###########.",
            ".###########.",
            ".##.......##.",
            ".##...#...##.",
            ".##..###..##.",
            ".##...#...##.",
            ".##.......##.",
            "..##.....##..",
            "...##...##...",
            "....##.##....",
            ".....###.....",
            "......#......",
        ],
        Kind::Cast => [
            "......#......",
            "......#......",
            ".....###.....",
            ".....###.....",
            "....#####....",
            "..#########..",
            "#############",
            "..#########..",
            "....#####....",
            ".....###.....",
            ".....###.....",
            "......#......",
            "......#......",
        ],
        Kind::Jump => [
            "......#......",
            ".....###.....",
            "....#####....",
            "...#######...",
            "..#########..",
            ".....###.....",
            ".....###.....",
            ".....###.....",
            ".....###.....",
            ".............",
            "..#########..",
            "..#########..",
            ".............",
        ],
        Kind::Rest => [
            "......#......",
            ".....##......",
            ".....###.....",
            "....####.#...",
            "...#####.##..",
            "...########..",
            "..#########..",
            "..####.####..",
            "...###.###...",
            "....#####....",
            "##.........##",
            ".###########.",
            "##.........##",
        ],
        Kind::Lock => [
            "......#......",
            "......#......",
            "...#######...",
            "..#########..",
            "..##.....##..",
            "..##.....##..",
            "####..#..####",
            "..##.....##..",
            "..##.....##..",
            "..#########..",
            "...#######...",
            "......#......",
            "......#......",
        ],
        Kind::Flask => [
            "......#......",
            ".....##......",
            ".....###.....",
            "....#####....",
            "....#####....",
            ".....###.....",
            ".............",
            ".###########.",
            "..#########..",
            "...#######...",
            "....#####....",
            "...#######...",
            "..#########..",
        ],
        Kind::Pause => [
            ".............",
            ".............",
            "...###.###...",
            "...###.###...",
            "...###.###...",
            "...###.###...",
            "...###.###...",
            "...###.###...",
            "...###.###...",
            "...###.###...",
            "...###.###...",
            ".............",
            ".............",
        ],
        // 스킬(0151). 휘돌이: 바람개비 날 넷.
        Kind::Skill1 | Kind::Whirl => [
            "......#......",
            ".....##......",
            "....###......",
            "....###...#..",
            ".....##.###..",
            "..#..######..",
            ".###########.",
            "..######..#..",
            "..###.##.....",
            "..#...###....",
            "......###....",
            "......##.....",
            "......#......",
        ],
        // 가로지르기: 선 하나를 꿰뚫고 지나가는 화살.
        Kind::Skill2 | Kind::Cut => [
            "...##........",
            "...##........",
            "...##....#...",
            "...##....##..",
            "...##....###.",
            "############.",
            "#############",
            "############.",
            "...##....###.",
            "...##....##..",
            "...##....#...",
            "...##........",
            "...##........",
        ],
        // 끌어모으기: 한 점으로 모이는 화살 넷.
        Kind::Gather => [
            "......#......",
            "....#####....",
            ".....###.....",
            "......#......",
            ".#.........#.",
            ".##..###..##.",
            "####.###.####",
            ".##..###..##.",
            ".#.........#.",
            "......#......",
            ".....###.....",
            "....#####....",
            "......#......",
        ],
        // 땅울림: 내리찍는 화살과 갈라지는 땅.
        Kind::Quake => [
            ".....###.....",
            ".....###.....",
            ".....###.....",
            "..#########..",
            "...#######...",
            "....#####....",
            ".....###.....",
            "......#......",
            "#############",
            "#############",
            "..#...#...#..",
            ".#....#....#.",
            "#.....#.....#",
        ],
        // 큰 기술 쟁반 뒤집기: 바깥으로 터지는 살과 가운데 판.
        Kind::Ult | Kind::Upend => [
            "#.....#.....#",
            ".#....#....#.",
            "..#...#...#..",
            ".............",
            "....#####....",
            "....#####....",
            "###.#####.###",
            "....#####....",
            "....#####....",
            ".............",
            "..#...#...#..",
            ".#....#....#.",
            "#.....#.....#",
        ],
        // 수류탄(원거리 무기의 RB): 둥근 몸과 도화선.
        Kind::Bomb => [
            ".........##..",
            "........#..#.",
            ".......#.....",
            "....#####....",
            "...#######...",
            "..####..###..",
            "..###...###..",
            "..#########..",
            "..#########..",
            "..#########..",
            "...#######...",
            "....#####....",
            ".............",
        ],
        Kind::TurnLeft | Kind::TurnRight => [
            ".............",
            ".............",
            "...######....",
            "..########...",
            ".###....###..",
            ".##......##..",
            "##.......##..",
            "##....#######",
            "##.....#####.",
            "........###..",
            ".........#...",
            ".............",
            ".............",
        ],
    }
}

/// 픽토그램의 칸 `(x, y)`가 칠해졌나 — 왼쪽 돌리기는 오른쪽 돌리기의 거울상.
pub fn icon_cell(k: Kind, x: usize, y: usize) -> bool {
    if x >= ICON_N || y >= ICON_N {
        return false;
    }
    let x = if k == Kind::TurnLeft {
        ICON_N - 1 - x
    } else {
        x
    };
    icon_mask(k)[y].as_bytes()[x] == b'#'
}

/// 버튼 반지름 `r`에 맞춘 픽토그램 한 칸의 크기(논리 px, 정수 — 칸이 고르게 떨어지게).
pub fn icon_px(r: f32) -> f32 {
    (r * 1.25 / ICON_N as f32).floor().max(1.0)
}

/// 픽토그램 그리기: 어두운 한 칸 테(밝은 바닥·불빛 위에서도 읽히게, KB hud-readability) 위에 밝은 몸. `tint` = 몸의 빛깔.
pub fn icon(k: Kind, c: Vec2, r: f32, dim: f32, tint: Color) {
    let px = icon_px(r);
    let size = ICON_N as f32 * px;
    let (x0, y0) = ((c.x - size / 2.0).round(), (c.y - size / 2.0).round());
    let body = Color::new(tint.r, tint.g, tint.b, 0.95 * dim);
    let edge = Color::new(INK.r, INK.g, INK.b, 0.7 * dim);
    let n = ICON_N as i32;
    // 테: 칠한 칸의 이웃 여덟 칸 가운데 비어 있는 칸을 한 번씩만(겹쳐 그리면 반투명이 얼룩진다).
    for y in -1..=n {
        for x in -1..=n {
            let at = |dx: i32, dy: i32| {
                let (u, v) = (x + dx, y + dy);
                u >= 0 && v >= 0 && icon_cell(k, u as usize, v as usize)
            };
            if at(0, 0) {
                continue;
            }
            let near = (-1..=1).any(|dy| (-1..=1).any(|dx| at(dx, dy)));
            if near {
                draw_rectangle(x0 + x as f32 * px, y0 + y as f32 * px, px, px, edge);
            }
        }
    }
    for y in 0..ICON_N {
        for x in 0..ICON_N {
            if icon_cell(k, x, y) {
                draw_rectangle(x0 + x as f32 * px, y0 + y as f32 * px, px, px, body);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 0146 도트 픽토그램: 그림마다 13×13, `#`·`.`만, 서로 다르고(같은 그림 둘이면 버튼을 못 가른다), 칸이 너무 적지도(안 보임)
    /// 너무 꽉 차지도(덩어리) 않다. 왼쪽 돌리기는 오른쪽 돌리기의 거울상. 칸 크기는 정수 px이고 그림이 버튼 안에 든다.
    #[test]
    fn touch_icons_are_13_by_13_and_tell_each_other_apart() {
        let kinds = [
            Kind::Attack,
            Kind::Dodge,
            Kind::Parry,
            Kind::Jump,
            Kind::Flask,
            Kind::Cast,
            Kind::Rest,
            Kind::Lock,
            Kind::TurnLeft,
            Kind::TurnRight,
            Kind::Pause,
            Kind::Hammer,
            Kind::Staff,
            Kind::Crossbow,
            Kind::Fist,
            Kind::Whirl,
            Kind::Cut,
            Kind::Gather,
            Kind::Quake,
            Kind::Upend,
            Kind::Bomb,
        ];
        // 0147: 무기마다 공격 그림이 다르다(같으면 무엇을 들었는지 버튼이 말하지 못한다).
        let icons: Vec<Kind> = crate::weapons::IDS
            .iter()
            .map(|w| attack_icon(*w))
            .collect();
        for (i, a) in icons.iter().enumerate() {
            assert!(!icons[i + 1..].contains(a), "{a:?} twice");
        }
        assert_eq!(attack_icon(crate::weapons::Id::Sword), Kind::Attack);
        let grid = |k: Kind| -> Vec<bool> {
            (0..ICON_N * ICON_N)
                .map(|i| icon_cell(k, i % ICON_N, i / ICON_N))
                .collect()
        };
        for k in kinds {
            let m = icon_mask(k);
            for row in m {
                assert_eq!(row.len(), ICON_N, "{k:?}: {row}");
                assert!(row.bytes().all(|b| b == b'#' || b == b'.'), "{k:?}");
            }
            let on = grid(k).iter().filter(|c| **c).count();
            assert!((30..=110).contains(&on), "{k:?}: {on} cells");
        }
        for (i, a) in kinds.iter().enumerate() {
            for b in &kinds[i + 1..] {
                let differ = grid(*a)
                    .iter()
                    .zip(grid(*b))
                    .filter(|(x, y)| **x != *y)
                    .count();
                assert!(
                    differ >= 12,
                    "{a:?} and {b:?} look alike ({differ} cells apart)"
                );
            }
        }
        for y in 0..ICON_N {
            for x in 0..ICON_N {
                assert_eq!(
                    icon_cell(Kind::TurnLeft, x, y),
                    icon_cell(Kind::TurnRight, ICON_N - 1 - x, y)
                );
            }
        }
        assert!(!icon_cell(Kind::Attack, ICON_N, 0) && !icon_cell(Kind::Attack, 0, ICON_N));
        for (w, h) in [(844.0, 390.0), (667.0, 375.0), (1180.0, 820.0)] {
            for b in layout(w, h, Ctx::ALL) {
                let px = icon_px(b.radius);
                assert!(px >= 2.0 && px.fract() == 0.0, "{w}x{h} {:?}: {px}", b.kind);
                // 그림의 대각 반(테 한 칸 포함)이 버튼 안에.
                let half = (ICON_N as f32 / 2.0 + 1.0) * px;
                assert!(half * 1.3 <= b.radius * 1.25, "{w}x{h} {:?}", b.kind);
            }
        }
    }

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
            {
                let b = layout(w, h, Ctx::ALL);
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
                            "{w}x{h} {:?} overlaps {:?}",
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
        let b = layout(667.0, 375.0, Ctx::ALL);
        let atk = b.iter().find(|x| x.kind == Kind::Attack).unwrap();
        assert!(b.iter().all(|x| x.radius <= atk.radius));
        assert!(
            b.iter().all(|x| x.radius * 2.0 >= 44.0 * 0.8),
            "smallest {:?}",
            b.iter().map(|x| x.radius).fold(f32::MAX, f32::min)
        );
    }

    /// 2026-11-06: 멈춤은 왼쪽 위(오른쪽 위엔 아무것도 없다), HUD는 그만큼 비킨다.
    #[test]
    fn the_pause_button_sits_top_left_and_the_hud_steps_aside() {
        for (w, h) in [(844.0, 390.0), (1280.0, 720.0)] {
            let b = layout(w, h, Ctx::ALL);
            let p = b.iter().find(|x| x.kind == Kind::Pause).unwrap();
            assert!(p.center.x < w * 0.1 && p.center.y < h * 0.15);
            assert!(hud_inset(w, h) > p.center.x + p.radius);
        }
    }

    /// 0053: 상황 버튼이 켜지고 꺼져도 늘 있는 버튼의 자리는 그대로다(엄지가 외운 자리가 밀리지 않게).
    #[test]
    fn contextual_buttons_never_move_the_others() {
        let all = layout(844.0, 390.0, Ctx::ALL);
        let none = layout(844.0, 390.0, Ctx::default());
        for x in &none {
            let y = all.iter().find(|y| y.kind == x.kind).unwrap();
            assert_eq!(x.center, y.center, "{:?}", x.kind);
        }
        assert!(!none
            .iter()
            .any(|x| matches!(x.kind, Kind::Lock | Kind::Flask)));
        assert_eq!(
            none.iter().filter(|x| x.kind != Kind::Pause).count(),
            9,
            "attack + 4 + skill pair + turn pair"
        );
        assert!(!none.iter().any(|x| x.kind == Kind::Ult));
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
        let b = layout(844.0, 390.0, Ctx::default());
        let x = |k| b.iter().find(|x| x.kind == k).unwrap().center.x;
        assert!(x(Kind::TurnRight) > x(Kind::TurnLeft));
    }

    /// 0151: 스킬 둘이 예전 돌리기 자리(엄지 호)를 물려받고, 돌리기는 오른쪽 위로 갔다. 큰 기술은 게이지가 찼을 때만 스킬 둘 위에 뜬다.
    #[test]
    fn skills_take_the_thumb_arc_and_turning_moves_to_the_top_right() {
        for (w, h) in [(844.0, 390.0), (667.0, 375.0), (1180.0, 820.0)] {
            let b = layout(w, h, Ctx::ALL);
            let at = |k| b.iter().find(|x| x.kind == k).unwrap();
            let (s1, s2, atk) = (at(Kind::Skill1), at(Kind::Skill2), at(Kind::Attack));
            assert!(s1.center.x < s2.center.x && s1.center.y == s2.center.y);
            assert!(s1.center.y < atk.center.y, "above the attack button");
            assert!(
                (s1.center - atk.center).length() < w.min(h) * 0.6,
                "in reach"
            );
            let (tl, tr) = (at(Kind::TurnLeft), at(Kind::TurnRight));
            assert!(tl.center.x > w * 0.7 && tr.center.y < h * 0.2, "top right");
            assert!(tl.radius < s1.radius, "turning is the smaller pair now");
            let u = at(Kind::Ult);
            assert!(u.center.y < s1.center.y && u.center.x > s1.center.x - 1.0);
            assert!(u.center.x < s2.center.x + 1.0, "between and above the two");
        }
        let mut a = Read::default();
        press(&mut a, Kind::Skill1);
        assert!(a.skill1 && !a.skill2 && !a.ult);
        let mut a = Read::default();
        press(&mut a, Kind::Ult);
        assert!(a.ult && !a.skill1);
        use crate::weapons::Id;
        assert_eq!(cast_icon(Id::Crossbow), Kind::Bomb);
        assert_eq!(cast_icon(Id::Sword), Kind::Cast);
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
