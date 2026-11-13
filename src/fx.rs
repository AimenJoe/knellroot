//! 그림 이펙트(0141): 월드 사건·상태 변화 → 입자(번쩍임·불똥·먼지·재). 월드는 이것을 모른다 — 소리(`sfx`)와 같은 길로 앱이 듣고
//! 여기에 넘긴다. 로직은 순수하다(난수 대신 번호 해시, 그림 핸들 없음) — 렌더(`Backdrop::fx`)는 `Fx::parts`를 읽어 그리기만.
//!
//! 원칙(KB `game-feel-particles-and-light` + 0141): 타격은 겹으로(번쩍임 1~2프레임 → 불똥 → 남는 먼지), 한 이펙트는 2~3색,
//! 타격은 짧고 밝게·몸짓(먼지)은 흙빛으로 낮게, 꾸밈은 많으면 건너뛰고 타격은 건너뛰지 않는다, 빨강은 쓰지 않는다(위험 예고 전용).
use macroquad::prelude::*;

use crate::enemy::EState;
use crate::world::{Event, PState, World};

/// 입자의 꼴(렌더가 고른다).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// 카메라를 보는 네모 화소 덩이(불똥·부스러기·빛 알갱이).
    Dot,
    /// 날아가는 쪽으로 늘어난 빛줄.
    Streak,
    /// 커지며 옅어지는 먼지·재(계단진 원판).
    Puff,
    /// 네 갈래 번쩍임(타격의 한가운데 — 아주 짧다).
    Star,
}

#[derive(Clone, Copy, Debug)]
pub struct Part {
    pub pos: Vec3,
    pub vel: Vec3,
    pub age: f32,
    pub life: f32,
    /// 처음·끝 크기(m).
    pub size: (f32, f32),
    /// 처음·끝 빛.
    pub color: (Color, Color),
    /// 아래로 당기는 세기(m/s²)와 속도가 주는 빠르기(/s).
    pub gravity: f32,
    pub drag: f32,
    pub shape: Shape,
    /// 꾸밈(몸짓·분위기) — 입자가 많으면 건너뛴다. 타격은 거짓.
    pub decor: bool,
}

impl Part {
    /// 수명의 몇 분의 몇(0..1).
    pub fn k(&self) -> f32 {
        (self.age / self.life.max(1e-3)).clamp(0.0, 1.0)
    }

    pub fn size_now(&self) -> f32 {
        self.size.0 + (self.size.1 - self.size.0) * self.k()
    }

    pub fn color_now(&self) -> Color {
        let (a, b, k) = (self.color.0, self.color.1, self.k());
        Color::new(
            a.r + (b.r - a.r) * k,
            a.g + (b.g - a.g) * k,
            a.b + (b.b - a.b) * k,
            a.a + (b.a - a.a) * k,
        )
    }
}

/// 바닥·허공에 잠깐 남는 빛의 꼴(0158 G2 — 스킬마다 다른 서명): 사건이 만들고(`Fx::on_events`) 렌더가 그린다(`Renderer::marks`).
/// 입자가 아니라 **모양**이라 입자 수 상한을 쓰지 않는다. `k` = 수명의 몫(0..1)으로 자라고 옅어진다.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MarkKind {
    /// 바닥 고리: 반지름이 `r0` → `r1`(퍼지거나 조여든다).
    Ring { r0: f32, r1: f32 },
    /// 길을 따라 남는 빛 띠(가로지르기): `dir` 쪽 `len` m, 반 폭 `half` — 떠난 자리(꼬리)부터 가늘어지며 사라진다.
    Streak { dir: Vec3, len: f32, half: f32 },
    /// 부채꼴을 가로질러 바깥으로 달리는 충격파 호(땅울림): `r0` → `r1`.
    Shock {
        dir: Vec3,
        r0: f32,
        r1: f32,
        arc_deg: f32,
    },
    /// 땅의 금 `n`줄(`dir` 쪽 `arc_deg`° 안에서 `r` m까지 뻗는다 — 360이면 사방).
    Cracks {
        dir: Vec3,
        r: f32,
        n: u8,
        arc_deg: f32,
    },
    /// 한 점으로 모여드는 빛줄 `n`개(끌어모으기): `r` m 밖에서 안으로.
    Converge { r: f32, n: u8 },
    /// 길을 가르는 빛의 면(한 줄기): `dir` 쪽 `len` m, 반 폭 `half` — 흰 속 + 금빛 겉, 길게 남는다.
    Slash { dir: Vec3, len: f32, half: f32 },
}

/// 남는 빛 하나. `delay` 뒤에 나타나 `life` 동안 산다. `follow` = 방랑자를 따라간다(몸에 두른 것),
/// `hold` = 버티는 동안에만(버팀이 끊기면 지운다).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mark {
    pub pos: Vec3,
    pub kind: MarkKind,
    pub age: f32,
    pub life: f32,
    pub delay: f32,
    pub color: Color,
    pub follow: bool,
    pub hold: bool,
}

impl Mark {
    pub fn k(&self) -> f32 {
        (self.age / self.life.max(1e-3)).clamp(0.0, 1.0)
    }

    pub fn live(&self) -> bool {
        self.delay <= 0.0
    }
}

/// 남는 빛의 수 상한(넘으면 가장 오래된 것부터 버린다).
pub const MARK_CAP: usize = 40;

/// 잔불 흩기의 불티(0158 G2 — 상태 없음, 화로 불티와 같은 꼴): 불티 `i`가 시각 `time`에 있는 자리(가운데·바닥 기준, 반지름 1의
/// 원판 안 — 렌더가 반지름을 곱한다)와 진하기(0..1). 주기마다 바닥에서 떠올라 흔들리며 식는다.
pub fn field_mote(i: usize, time: f32) -> (Vec3, f32) {
    let h = |n: u32| {
        let mut x = (i as u32).wrapping_mul(19_349_663) ^ n.wrapping_mul(83_492_791) ^ 0x9e37;
        x ^= x >> 13;
        x = x.wrapping_mul(0x5bd1_e995);
        x ^= x >> 15;
        (x & 0xffff) as f32 / 65535.0
    };
    let period = 0.9 + 0.9 * h(1);
    let ph = ((time + h(2) * period) / period).fract();
    let a = h(3) * std::f32::consts::TAU;
    // 넓이에 고르게(√) — 가장자리까지.
    let r = h(4).sqrt() * 0.96;
    let sway = (time * (2.0 + 2.0 * h(5)) + h(6) * 6.0).sin() * 0.05 * ph;
    let pos = vec3(
        a.sin() * r + sway,
        0.08 + (0.7 + 0.9 * h(7)) * ph,
        a.cos() * r,
    );
    let k = (ph / 0.12).min(1.0) * ((1.0 - ph) / 0.35).min(1.0);
    (pos, k)
}
/// 불티 하나의 수(반지름 4m 기준 — 렌더가 넓이에 맞춰 늘린다, 상한 `FIELD_MOTES_MAX`).
pub const FIELD_MOTES: usize = 44;
pub const FIELD_MOTES_MAX: usize = 72;

/// 입자 수 상한: 꾸밈은 `DECOR_CAP`을 넘으면 새로 내지 않고, 타격은 `HARD_CAP`까지(그 뒤엔 가장 오래된 것부터 버린다).
pub const DECOR_CAP: usize = 220;
pub const HARD_CAP: usize = 420;

// 빛깔(2~3색씩 골라 쓴다). 따뜻한 흰빛·호박빛은 마무리 패스(0140)의 번짐을 받는다.
const WHITE_HOT: Color = Color::new(1.0, 0.97, 0.86, 1.0);
const AMBER: Color = Color::new(1.0, 0.72, 0.32, 1.0);
const GOLD: Color = Color::new(1.0, 0.86, 0.45, 1.0);
const COOL: Color = Color::new(0.80, 0.92, 1.0, 1.0);
const STEEL: Color = Color::new(0.74, 0.78, 0.88, 1.0);
const LAVENDER: Color = Color::new(0.82, 0.74, 1.0, 1.0);
/// 스킬의 민트(0151 — 땅을 다루는 기술의 빛).
const MINT: Color = Color::new(0.66, 1.0, 0.86, 1.0);
const HEAL: Color = Color::new(1.0, 0.86, 0.56, 1.0);
/// 먼지 = 바닥 석판보다 한 단 밝은 남보라 — 바닥과 같은 빛이면(처음 0.50·0.48·0.62) 깔리는 게 아니라 안 보였다.
const DUST: Color = Color::new(0.63, 0.61, 0.74, 0.72);
/// 쓰러진 적에게서 풀려나는 재(바랜 잿빛 라벤더, 0037)와 뿌리의 보라.
const ASH: Color = Color::new(0.72, 0.70, 0.80, 0.75);
const ROOT: Color = Color::new(0.56, 0.38, 0.70, 1.0);
const SMOKE: Color = Color::new(0.22, 0.21, 0.30, 0.8);
const STONE: Color = Color::new(0.40, 0.38, 0.50, 1.0);

fn fade(c: Color) -> Color {
    Color::new(c.r, c.g, c.b, 0.0)
}

#[derive(Default)]
pub struct Fx {
    pub parts: Vec<Part>,
    /// 남는 빛(0158 G2 — 렌더가 그린다).
    pub marks: Vec<Mark>,
    /// 조금 뒤에 나올 입자(한 줄기의 늦은 불똥): (남은 초, 입자).
    later: Vec<(f32, Part)>,
    /// 몰아치기의 기운을 낸 뒤 흐른 시간.
    aura_t: f32,
    /// 번호 해시의 씨(게임 난수와 따로 — 이펙트가 놀이를 바꾸지 않는다).
    seed: u32,
    was_dodging: bool,
    was_air: bool,
    /// 적마다: 지난 틱에 돌진 중이었나·예고 중이었나, 돌진 먼지를 낸 뒤 흐른 시간.
    enemy_was: Vec<(bool, bool, f32)>,
    run_dist: f32,
    last_pos: Option<Vec3>,
}

impl Fx {
    /// 0..1(번호마다 다른 값, 같은 차례 = 같은 값).
    fn r(&mut self) -> f32 {
        self.seed = self
            .seed
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        ((self.seed >> 8) & 0xffff) as f32 / 65535.0
    }

    /// −1..1.
    fn rs(&mut self) -> f32 {
        self.r() * 2.0 - 1.0
    }

    fn push(&mut self, p: Part) {
        let cap = if p.decor { DECOR_CAP } else { HARD_CAP };
        if self.parts.len() >= cap {
            if p.decor {
                return;
            }
            self.parts.remove(0);
        }
        self.parts.push(p);
    }

    fn mark(&mut self, pos: Vec3, kind: MarkKind, life: f32, color: Color) {
        self.mark_with(pos, kind, life, color, 0.0, false, false);
    }

    #[allow(clippy::too_many_arguments)]
    fn mark_with(
        &mut self,
        pos: Vec3,
        kind: MarkKind,
        life: f32,
        color: Color,
        delay: f32,
        follow: bool,
        hold: bool,
    ) {
        if self.marks.len() >= MARK_CAP {
            self.marks.remove(0);
        }
        self.marks.push(Mark {
            pos,
            kind,
            age: 0.0,
            life,
            delay,
            color,
            follow,
            hold,
        });
    }

    /// 바깥 고리 `r`에서 가운데로 빨려드는 빛 알갱이 `n`개(끌어모으기의 모임).
    fn implode(&mut self, at: Vec3, r: f32, n: usize, col: Color) {
        let turn = self.r() * std::f32::consts::TAU;
        for i in 0..n {
            let a = turn + i as f32 / n as f32 * std::f32::consts::TAU + self.rs() * 0.25;
            let d = vec3(a.sin(), 0.0, a.cos());
            let rr = r * (0.55 + 0.45 * self.r());
            let life = 0.22 + 0.08 * self.r();
            let h = 0.3 + 0.6 * self.r();
            self.push(Part {
                pos: at + d * rr + Vec3::Y * h,
                vel: -d * (rr / life) * 1.6,
                age: 0.0,
                life,
                size: (0.11, 0.05),
                color: (col, fade(WHITE_HOT)),
                gravity: 0.0,
                drag: 3.0,
                shape: Shape::Streak,
                decor: false,
            });
        }
    }

    /// 반지름 `r`의 고리에서 바깥으로 튀는 빛줄 `n`개(허리 높이 — 휘돌이·터뜨림·울림).
    #[allow(clippy::too_many_arguments)]
    fn ring_sparks(
        &mut self,
        at: Vec3,
        r: f32,
        n: usize,
        speed: f32,
        life: f32,
        from: Color,
        to: Color,
    ) {
        let turn = self.r() * std::f32::consts::TAU;
        for i in 0..n {
            let a = turn + i as f32 / n as f32 * std::f32::consts::TAU + self.rs() * 0.15;
            let d = vec3(a.sin(), 0.0, a.cos());
            let sp = speed * (0.7 + 0.6 * self.r());
            let l = life * (0.7 + 0.5 * self.r());
            let up = 0.5 + 1.5 * self.r();
            self.push(Part {
                pos: at + d * r,
                vel: d * sp + Vec3::Y * up,
                age: 0.0,
                life: l,
                size: (0.08, 0.04),
                color: (from, fade(to)),
                gravity: 3.0,
                drag: 2.5,
                shape: Shape::Streak,
                decor: false,
            });
        }
    }

    /// 한가운데 번쩍임 하나(네 갈래 별).
    fn star(&mut self, at: Vec3, size: f32, life: f32, col: Color) {
        self.push(Part {
            pos: at,
            vel: Vec3::ZERO,
            age: 0.0,
            life,
            size: (size, size * 0.25),
            color: (col, col),
            gravity: 0.0,
            drag: 0.0,
            shape: Shape::Star,
            decor: false,
        });
    }

    /// `dir` 쪽 원뿔(벌어짐 `spread` — 0 = 한 줄, 1 = 사방)로 튀는 불똥 `n`개.
    #[allow(clippy::too_many_arguments)]
    fn spray(
        &mut self,
        at: Vec3,
        dir: Vec3,
        spread: f32,
        n: usize,
        speed: f32,
        shape: Shape,
        size: f32,
        life: f32,
        gravity: f32,
        from: Color,
        to: Color,
    ) {
        let dir = dir.normalize_or(Vec3::Y);
        for _ in 0..n {
            let wob = vec3(self.rs(), self.rs() * 0.7 + 0.25, self.rs());
            let v = (dir * (1.0 - spread) + wob * spread).normalize_or(Vec3::Y);
            let sp = speed * (0.55 + 0.75 * self.r());
            let l = life * (0.7 + 0.5 * self.r());
            self.push(Part {
                pos: at,
                vel: v * sp,
                age: 0.0,
                life: l,
                size: (size, size * 0.5),
                color: (from, fade(to)),
                gravity,
                drag: 2.5,
                shape,
                decor: false,
            });
        }
    }

    /// 꺼진 화로에 불이 붙는다(0171 — 사건이 아니라 앱이 부른다: 월드는 화로를 모른다): 그릇 위의 번쩍임 + 솟는 불티 +
    /// 둘레로 튀는 금빛 줄. `at` = 화로의 발밑.
    pub fn kindle(&mut self, at: Vec3) {
        let bowl = at + Vec3::Y * 0.9;
        self.star(bowl + Vec3::Y * 0.15, 1.5, 0.16, WHITE_HOT);
        self.spray(
            bowl,
            Vec3::Y,
            0.45,
            26,
            3.4,
            Shape::Dot,
            0.09,
            1.1,
            1.2,
            WHITE_HOT,
            AMBER,
        );
        self.ring_sparks(bowl, 0.3, 12, 2.6, 0.5, GOLD, AMBER);
    }

    /// 바닥에서 둘레로 퍼지는 먼지 `n`덩이(반지름 `r`의 고리에서 바깥으로 `speed`).
    fn dust_ring(&mut self, at: Vec3, r: f32, n: usize, speed: f32, size: f32, col: Color) {
        let turn = self.r() * std::f32::consts::TAU;
        for i in 0..n {
            let a = turn + i as f32 / n as f32 * std::f32::consts::TAU + self.rs() * 0.2;
            let d = vec3(a.sin(), 0.0, a.cos());
            let s = size * (0.8 + 0.5 * self.r());
            let vel = d * speed * (0.7 + 0.5 * self.r()) + Vec3::Y * (0.3 + 0.5 * self.r());
            let life = 0.42 + 0.2 * self.r();
            self.push(Part {
                pos: at + d * r + Vec3::Y * 0.12,
                vel,
                age: 0.0,
                life,
                size: (s * 0.55, s * 1.5),
                color: (col, fade(col)),
                gravity: 0.0,
                drag: 3.5,
                shape: Shape::Puff,
                decor: true,
            });
        }
    }

    /// 한쪽으로 밀리는 먼지 `n`덩이(회피·돌진의 발밑).
    fn dust_kick(&mut self, at: Vec3, away: Vec3, n: usize, size: f32) {
        let away = away.normalize_or_zero();
        for _ in 0..n {
            let side = vec3(-away.z, 0.0, away.x) * self.rs() * 0.6;
            let s = size * (0.8 + 0.5 * self.r());
            let vel = (away + side) * (1.4 + 1.2 * self.r()) + Vec3::Y * (0.4 + 0.4 * self.r());
            let life = 0.36 + 0.18 * self.r();
            self.push(Part {
                pos: at + side * 0.3 + Vec3::Y * 0.1,
                vel,
                age: 0.0,
                life,
                size: (s * 0.5, s * 1.4),
                color: (DUST, fade(DUST)),
                gravity: 0.0,
                drag: 4.0,
                shape: Shape::Puff,
                decor: true,
            });
        }
    }

    /// 위로 떠오르며 옅어지는 것 `n`개(재·회복의 빛 — 몸 둘레 `r` 안에서).
    #[allow(clippy::too_many_arguments)]
    fn rise(
        &mut self,
        at: Vec3,
        r: f32,
        n: usize,
        shape: Shape,
        size: (f32, f32),
        life: f32,
        from: Color,
        to: Color,
        decor: bool,
    ) {
        for _ in 0..n {
            let off = vec3(self.rs() * r, self.r() * r * 1.2, self.rs() * r);
            let l = life * (0.7 + 0.6 * self.r());
            let vel = vec3(self.rs() * 0.3, 0.7 + 0.9 * self.r(), self.rs() * 0.3);
            self.push(Part {
                pos: at + off,
                vel,
                age: 0.0,
                life: l,
                size,
                color: (from, fade(to)),
                gravity: -0.4,
                drag: 1.2,
                shape,
                decor,
            });
        }
    }

    /// 월드 사건 → 이펙트. `w`는 방금 그 틱을 마친 월드(읽기만).
    pub fn on_events(&mut self, events: &[Event], w: &World) {
        let pp = w.player.pos;
        // 맞은 쪽 = 플레이어에서 그 자리로(수평) — 불똥이 "밀어낸 쪽"으로 튄다.
        let from_player = |at: Vec3| vec3(at.x - pp.x, 0.0, at.z - pp.z).normalize_or(Vec3::Z);
        for e in events {
            match *e {
                // 평타: 번쩍임 한 번 — 불똥은 월드의 `sparks`(같은 스윙 = 같은 무늬)가 낸다.
                Event::Hit { at } => self.star(at, 0.95, 0.07, WHITE_HOT),
                // 치명타: 더 크고 금빛, 금 알갱이가 사방으로.
                Event::Crit { at } => {
                    self.star(at, 1.7, 0.11, GOLD);
                    self.spray(
                        at,
                        Vec3::Y,
                        1.0,
                        9,
                        5.5,
                        Shape::Dot,
                        0.10,
                        0.34,
                        9.0,
                        WHITE_HOT,
                        GOLD,
                    );
                }
                // 강타: 큰 번쩍임 + 돌 부스러기 + 발밑 먼지.
                Event::Smash { at } => {
                    let d = from_player(at);
                    self.star(at, 1.5, 0.10, WHITE_HOT);
                    self.spray(
                        at,
                        d + Vec3::Y * 0.6,
                        0.55,
                        7,
                        5.0,
                        Shape::Dot,
                        0.11,
                        0.45,
                        14.0,
                        STONE,
                        STONE,
                    );
                    let g = vec3(at.x, w.ground_y(at), at.z);
                    self.dust_ring(g, 0.3, 6, 2.2, 0.5, DUST);
                }
                // 패링: 차가운 흰빛 별 + 사방으로 빠른 빛줄("쨍").
                Event::Parry { at } => {
                    self.star(at, 1.9, 0.12, COOL);
                    self.spray(
                        at,
                        Vec3::Y,
                        1.0,
                        12,
                        9.5,
                        Shape::Streak,
                        0.07,
                        0.20,
                        4.0,
                        WHITE_HOT,
                        COOL,
                    );
                }
                // 튕김(등딱지): 작은 쇳빛 별 + 플레이어 쪽으로 되튀는 쇳빛 줄.
                Event::Deflect { at } => {
                    let back = -from_player(at);
                    self.star(at, 0.9, 0.08, STEEL);
                    self.spray(
                        at,
                        back + Vec3::Y * 0.4,
                        0.45,
                        6,
                        6.5,
                        Shape::Streak,
                        0.06,
                        0.18,
                        8.0,
                        WHITE_HOT,
                        STEEL,
                    );
                }
                // 자세 깨짐: 금빛 별 + 머리 높이에서 옆으로 퍼지는 금 알갱이.
                Event::Break { at } => {
                    self.star(at, 1.6, 0.12, GOLD);
                    self.spray(
                        at,
                        Vec3::Y,
                        1.0,
                        8,
                        4.5,
                        Shape::Dot,
                        0.09,
                        0.4,
                        5.0,
                        GOLD,
                        AMBER,
                    );
                }
                // 쓰러짐: 바랜 재가 풀려 오르고(0037 — 침식 = 색을 잃은 것), 뿌리의 보라 알갱이가 흩어진다.
                Event::EnemyDown { at } => {
                    let c = at + Vec3::Y * 0.5;
                    self.rise(c, 0.45, 6, Shape::Puff, (0.35, 0.75), 0.7, ASH, ASH, false);
                    self.rise(c, 0.4, 7, Shape::Dot, (0.09, 0.05), 0.8, ROOT, ROOT, false);
                }
                // 내려찍기·여진: 둘레로 먼지 고리 + 튀는 돌 부스러기.
                Event::Plunge { at } | Event::Aftershock { at } => {
                    let g = vec3(at.x, w.ground_y(at), at.z);
                    self.dust_ring(g, 0.5, 10, 3.6, 0.6, DUST);
                    self.spray(
                        g + Vec3::Y * 0.1,
                        Vec3::Y,
                        0.8,
                        6,
                        4.5,
                        Shape::Dot,
                        0.10,
                        0.45,
                        13.0,
                        STONE,
                        STONE,
                    );
                }
                // 벽에 부딪힌 적: 먼지 + 부스러기.
                Event::Slam { at } => {
                    self.star(at, 1.0, 0.08, WHITE_HOT);
                    self.dust_ring(vec3(at.x, w.ground_y(at), at.z), 0.2, 5, 1.8, 0.5, DUST);
                    self.spray(
                        at,
                        Vec3::Y,
                        0.9,
                        5,
                        3.5,
                        Shape::Dot,
                        0.10,
                        0.4,
                        13.0,
                        STONE,
                        STONE,
                    );
                }
                // 폭발: 큰 따뜻한 번쩍임 + 사방 불티 + 피어오르는 연기 + 바닥 먼지.
                Event::Boom { at } => {
                    let c = at + Vec3::Y * 0.4;
                    self.star(c, 2.6, 0.13, WHITE_HOT);
                    self.spray(
                        c,
                        Vec3::Y,
                        1.0,
                        14,
                        8.0,
                        Shape::Streak,
                        0.08,
                        0.38,
                        11.0,
                        WHITE_HOT,
                        AMBER,
                    );
                    self.rise(c, 0.6, 6, Shape::Puff, (0.5, 1.2), 0.8, SMOKE, SMOKE, false);
                    self.dust_ring(vec3(at.x, w.ground_y(at), at.z), 0.6, 8, 3.0, 0.6, DUST);
                }
                // 등잔: 따뜻한 빛 알갱이가 몸 둘레에서 떠오른다.
                Event::Gulp { at } => {
                    self.rise(
                        at - Vec3::Y * 0.7,
                        0.4,
                        9,
                        Shape::Dot,
                        (0.08, 0.04),
                        0.8,
                        HEAL,
                        WHITE_HOT,
                        false,
                    );
                }
                // 마법 탄 발사: 손끝의 작은 라벤더 번쩍임.
                Event::Cast { at } => {
                    self.star(at, 0.8, 0.07, LAVENDER);
                    self.spray(
                        at,
                        Vec3::Y,
                        1.0,
                        4,
                        2.5,
                        Shape::Dot,
                        0.07,
                        0.25,
                        2.0,
                        WHITE_HOT,
                        LAVENDER,
                    );
                }
                // 딱 맞춰 떼기: 금 알갱이가 사방으로.
                Event::Perfect { at } => {
                    self.star(at, 1.2, 0.09, GOLD);
                    self.spray(
                        at,
                        Vec3::Y,
                        1.0,
                        8,
                        5.0,
                        Shape::Dot,
                        0.08,
                        0.35,
                        5.0,
                        WHITE_HOT,
                        GOLD,
                    );
                }
                // 스킬(0151 → 0158 G2 — 금·민트·흰빛, 빨강 없음). 스킬마다 다른 서명: 한가운데 번쩍임 + 빛줄 + 먼지에
                // 바닥에 남는 빛(`marks`)이 그 스킬의 꼴을 그린다.
                Event::SkillStart { id } => self.on_skill_start(id, w),
                Event::Skill { id, at, dir } => self.on_skill(id, at, dir, w),
                // 모인 자리(0158 G1, 끌어모으기 ㄱ): 모인 점에서 터지는 민트빛 별 + 바깥으로 퍼지는 고리와 먼지 + 위로 튀는 빛줄.
                Event::SkillBurst { at } => {
                    let g = vec3(at.x, w.ground_y(at), at.z);
                    self.star(g + Vec3::Y * 0.7, 2.0, 0.12, MINT);
                    self.mark(g, MarkKind::Ring { r0: 0.4, r1: 2.5 }, 0.3, MINT);
                    self.dust_ring(g, 0.5, 10, 4.2, 0.55, DUST);
                    self.spray(
                        g + Vec3::Y * 0.5,
                        Vec3::Y,
                        1.0,
                        8,
                        6.0,
                        Shape::Streak,
                        0.07,
                        0.25,
                        4.0,
                        WHITE_HOT,
                        MINT,
                    );
                }
                // 버팀이 한 대를 받아냈다: 맞은 자리의 흰 별 + 사방으로 튕기는 금빛 줄(받아낼수록 많이) + 몸 둘레로 번지는 고리.
                Event::Soak { at, n } => {
                    self.star(at, 1.3, 0.1, WHITE_HOT);
                    self.spray(
                        at,
                        Vec3::Y,
                        1.0,
                        5 + 3 * n as usize,
                        7.5,
                        Shape::Streak,
                        0.07,
                        0.22,
                        6.0,
                        WHITE_HOT,
                        GOLD,
                    );
                    let g = vec3(at.x, w.ground_y(at), at.z);
                    self.mark(g, MarkKind::Ring { r0: 0.5, r1: 1.4 }, 0.2, GOLD);
                }
                // 되울림이 퍼졌다: 종처럼 번지는 고리 셋(가운데에서 닿는 거리까지) + 민트빛 줄.
                Event::EchoRing { at, radius } => {
                    let g = vec3(at.x, w.ground_y(at), at.z);
                    self.star(g + Vec3::Y * 1.0, 1.8, 0.12, WHITE_HOT);
                    for (i, col) in [WHITE_HOT, MINT, MINT].into_iter().enumerate() {
                        self.mark_with(
                            g,
                            MarkKind::Ring {
                                r0: 0.6,
                                r1: radius,
                            },
                            0.34,
                            col,
                            0.07 * i as f32,
                            false,
                            false,
                        );
                    }
                    self.ring_sparks(g + Vec3::Y * 0.9, 0.8, 12, 9.0, 0.24, WHITE_HOT, MINT);
                }
                // 던진 것이 끝에서 되돌았다: 금빛 별 + 둘레로 튀는 줄.
                Event::ThrowTurn { at } => {
                    self.star(at, 1.3, 0.1, GOLD);
                    self.ring_sparks(at - Vec3::Y * 0.2, 0.15, 8, 5.0, 0.2, WHITE_HOT, GOLD);
                }
                // 박힌 자리: 큰 별 + 둘레로 퍼지는 고리 둘 + 사방의 금빛 줄 + 바닥 먼지.
                Event::ThrowBoom { at, radius } => {
                    let g = vec3(at.x, w.ground_y(at), at.z);
                    self.star(g + Vec3::Y * 0.7, 2.6, 0.13, WHITE_HOT);
                    self.mark(
                        g,
                        MarkKind::Ring {
                            r0: 0.3,
                            r1: radius,
                        },
                        0.28,
                        GOLD,
                    );
                    self.mark_with(
                        g,
                        MarkKind::Ring {
                            r0: 0.2,
                            r1: radius * 0.7,
                        },
                        0.3,
                        WHITE_HOT,
                        0.05,
                        false,
                        false,
                    );
                    self.spray(
                        g + Vec3::Y * 0.5,
                        Vec3::Y,
                        1.0,
                        14,
                        8.5,
                        Shape::Streak,
                        0.08,
                        0.34,
                        9.0,
                        WHITE_HOT,
                        GOLD,
                    );
                    self.dust_ring(g, 0.6, 9, 3.4, 0.6, DUST);
                }
                // 손에 돌아왔다: 손 높이의 작은 금빛 별.
                Event::ThrowBack { at } => {
                    self.star(at, 0.9, 0.08, GOLD);
                    self.spray(
                        at,
                        Vec3::Y,
                        1.0,
                        4,
                        3.0,
                        Shape::Dot,
                        0.07,
                        0.22,
                        4.0,
                        WHITE_HOT,
                        GOLD,
                    );
                }
                // 불티의 타: 가장자리에서 가운데로 훑는 호박빛 고리 + 솟는 불티.
                Event::FieldPulse { at, radius } => {
                    let g = vec3(at.x, w.ground_y(at), at.z);
                    self.mark(
                        g,
                        MarkKind::Ring {
                            r0: radius,
                            r1: radius * 0.25,
                        },
                        0.3,
                        AMBER,
                    );
                    for _ in 0..12 {
                        let a = self.r() * std::f32::consts::TAU;
                        let rr = radius * self.r().sqrt();
                        let p = g + vec3(a.sin() * rr, 0.1, a.cos() * rr);
                        let life = 0.45 + 0.3 * self.r();
                        let up = 2.2 + 1.6 * self.r();
                        self.push(Part {
                            pos: p,
                            vel: vec3(0.0, up, 0.0),
                            age: 0.0,
                            life,
                            size: (0.1, 0.04),
                            color: (GOLD, fade(AMBER)),
                            gravity: -0.5,
                            drag: 1.5,
                            shape: Shape::Dot,
                            decor: false,
                        });
                    }
                }
                // 몰아치기가 시작됐다: 금빛 큰 별 + 위로 솟는 줄 + 발밑에서 퍼지는 고리 둘.
                Event::Flurry { at } => {
                    let g = vec3(at.x, w.ground_y(at), at.z);
                    self.star(g + Vec3::Y * 1.0, 2.6, 0.14, GOLD);
                    self.spray(
                        g + Vec3::Y * 0.4,
                        Vec3::Y,
                        0.45,
                        16,
                        9.0,
                        Shape::Streak,
                        0.08,
                        0.4,
                        4.0,
                        WHITE_HOT,
                        GOLD,
                    );
                    self.mark(g, MarkKind::Ring { r0: 0.4, r1: 2.6 }, 0.3, GOLD);
                    self.mark_with(
                        g,
                        MarkKind::Ring { r0: 0.3, r1: 1.8 },
                        0.32,
                        WHITE_HOT,
                        0.08,
                        false,
                        false,
                    );
                }
                // 몰아치기 동안의 평타: 휘두를 때마다 바라보는 쪽으로 금빛 줄이 흩어진다.
                Event::Swing if w.player.skill.flurry_left > 0.0 => {
                    let f = w.player.facing_dir();
                    self.spray(
                        pp + Vec3::Y * 1.0 + f * 0.8,
                        f + Vec3::Y * 0.2,
                        0.5,
                        5,
                        7.0,
                        Shape::Streak,
                        0.06,
                        0.18,
                        3.0,
                        WHITE_HOT,
                        GOLD,
                    );
                }
                // 한 줄기: 길을 가르는 빛의 면(흰 속 + 금빛 겉, 길게 남는다) + 선 자리의 큰 별 + 길을 따라 늦게 터지는 불똥.
                Event::LineCut { at, dir, len, half } => {
                    let g = vec3(at.x, w.ground_y(at), at.z);
                    self.mark(g, MarkKind::Slash { dir, len, half }, 0.75, GOLD);
                    self.star(g + Vec3::Y * 1.0 + dir * 0.8, 3.0, 0.14, WHITE_HOT);
                    self.dust_kick(g, -dir, 5, 0.6);
                    let side = vec3(dir.z, 0.0, -dir.x);
                    let n = (len / 1.1).ceil() as usize;
                    for i in 0..n {
                        let along = (i as f32 + 0.5) / n as f32 * len;
                        let delay = 0.03 + along / len * 0.2;
                        for sgn in [-1.0f32, 1.0] {
                            let life = 0.3 + 0.15 * self.r();
                            let part = Part {
                                pos: g + dir * along + Vec3::Y * 0.3,
                                vel: side * sgn * (2.5 + 2.5 * self.r())
                                    + Vec3::Y * (3.0 + 3.0 * self.r()),
                                age: 0.0,
                                life,
                                size: (0.09, 0.04),
                                color: (WHITE_HOT, fade(GOLD)),
                                gravity: 9.0,
                                drag: 2.0,
                                shape: Shape::Streak,
                                decor: false,
                            };
                            self.later.push((delay, part));
                        }
                    }
                }
                // 큰 기술의 시동(끼운 것마다): 쟁반 뒤집기는 발밑으로 모이는 금빛, 몰아치기는 솟는 금빛, 한 줄기는 흰 별.
                Event::UltStart => {
                    let g = vec3(pp.x, w.ground_y(pp), pp.z);
                    match w.player.skill.ult {
                        crate::skills::Ult::Upend => {
                            self.star(pp + Vec3::Y * 0.4, 1.4, 0.3, GOLD);
                            self.dust_ring(g, 2.6, 12, -2.6, 0.45, DUST);
                            self.mark(g, MarkKind::Ring { r0: 2.8, r1: 0.4 }, 0.3, GOLD);
                        }
                        crate::skills::Ult::Flurry => {
                            self.star(pp + Vec3::Y * 1.0, 1.2, 0.2, GOLD);
                            self.implode(g, 2.2, 10, GOLD);
                        }
                        crate::skills::Ult::Line => {
                            self.star(pp + Vec3::Y * 1.1, 1.6, 0.3, WHITE_HOT);
                            self.implode(g, 2.4, 12, WHITE_HOT);
                        }
                    }
                }
                // 쟁반 뒤집기: 가장 큰 별 + 두 겹 먼지 고리 + 사방의 금빛 줄 — 사방으로 뻗는 땅의 금과 테두리의 먼지 벽(0158 G2).
                Event::Ult { at } => {
                    let g = vec3(at.x, w.ground_y(at), at.z);
                    let r = crate::skills::UPEND.radius;
                    self.star(g + Vec3::Y * 0.8, 3.2, 0.15, GOLD);
                    self.dust_ring(g, 1.0, 14, 7.5, 0.9, DUST);
                    self.dust_ring(g, 2.5, 14, 5.0, 0.7, DUST);
                    self.spray(
                        g + Vec3::Y * 0.5,
                        Vec3::Y,
                        1.0,
                        18,
                        11.0,
                        Shape::Streak,
                        0.09,
                        0.4,
                        5.0,
                        WHITE_HOT,
                        GOLD,
                    );
                    self.mark(
                        g,
                        MarkKind::Cracks {
                            dir: Vec3::Z,
                            r: r * 0.85,
                            n: 11,
                            arc_deg: 360.0,
                        },
                        1.1,
                        GOLD,
                    );
                    self.mark(g, MarkKind::Ring { r0: 1.0, r1: r }, 0.4, GOLD);
                    // 테두리의 먼지 벽: 둘레를 따라 솟는 큰 먼지(꾸밈이 아니다 — 쟁반의 끝이 어디인지 보여 준다).
                    let turn = self.r() * std::f32::consts::TAU;
                    for i in 0..20 {
                        let a = turn + i as f32 / 20.0 * std::f32::consts::TAU;
                        let d = vec3(a.sin(), 0.0, a.cos());
                        let life = 0.7 + 0.3 * self.r();
                        let up = 1.6 + 1.4 * self.r();
                        self.later.push((
                            0.16,
                            Part {
                                pos: g + d * (r - 0.4) + Vec3::Y * 0.2,
                                vel: d * 1.2 + Vec3::Y * up,
                                age: 0.0,
                                life,
                                size: (0.7, 1.7),
                                color: (DUST, fade(DUST)),
                                gravity: 0.0,
                                drag: 2.2,
                                shape: Shape::Puff,
                                decor: false,
                            },
                        ));
                    }
                }
                // 게이지가 가득: 가슴 높이의 금빛 번쩍임.
                Event::UltReady => self.star(pp + Vec3::Y * 1.2, 1.2, 0.12, GOLD),
                // 다 모였다: 손 높이의 작은 금빛 번쩍임.
                Event::Charged => self.star(pp + Vec3::Y * 1.0, 0.9, 0.09, GOLD),
                // 보스가 깨어남(2단계): 호박빛 불티가 사방으로 + 넓은 먼지 고리.
                Event::Rouse { at } => {
                    self.star(at, 2.8, 0.14, AMBER);
                    self.spray(
                        at,
                        Vec3::Y,
                        1.0,
                        16,
                        8.5,
                        Shape::Streak,
                        0.09,
                        0.45,
                        6.0,
                        WHITE_HOT,
                        AMBER,
                    );
                    self.dust_ring(vec3(at.x, w.ground_y(at), at.z), 1.2, 12, 4.5, 0.8, DUST);
                }
                // 맞음: 가슴 높이의 흰 별 + 옅은 라벤더 알갱이(빨강은 쓰지 않는다 — 화면의 붉은 테는 HUD의 몫).
                Event::PlayerHurt => {
                    let c = pp + Vec3::Y * 0.9;
                    self.star(c, 1.3, 0.09, WHITE_HOT);
                    self.spray(
                        c,
                        Vec3::Y,
                        1.0,
                        6,
                        4.0,
                        Shape::Dot,
                        0.08,
                        0.3,
                        8.0,
                        WHITE_HOT,
                        LAVENDER,
                    );
                }
                _ => {}
            }
        }
    }

    /// 스킬의 시동(0158 G2): 몸에 걸리는 조짐 — 버팀은 조여드는 고리(버티는 동안), 되울림·잔불 흩기는 손에 모이는 빛.
    fn on_skill_start(&mut self, id: crate::skills::Id, w: &World) {
        use crate::skills::{Form, Id as Sk};
        let pp = w.player.pos;
        let g = vec3(pp.x, w.ground_y(pp), pp.z);
        let d = w.skill_eff(id);
        match (id, d.skill.form) {
            (Sk::Brace, Form::Brace { hold, radius, .. }) => {
                // 반짝이며 조여드는 고리 둘(버티는 동안 몸을 따라간다 — 끊기면 지운다).
                for (delay, col) in [(0.0, GOLD), (hold * 0.5, WHITE_HOT)] {
                    self.mark_with(
                        g,
                        MarkKind::Ring {
                            r0: radius,
                            r1: 0.9,
                        },
                        d.skill.startup + hold - delay,
                        col,
                        delay,
                        true,
                        true,
                    );
                }
                self.dust_ring(g, 0.4, 6, 1.6, 0.45, DUST);
            }
            (Sk::Echo, _) => self.implode(g, 1.8, 8, MINT),
            (Sk::Embers, _) => self.implode(g, 1.6, 8, AMBER),
            (Sk::Gather, Form::Pull { .. }) => {}
            _ => {}
        }
    }

    /// 스킬의 판정이 나간 순간(0151 → 0158 G2): 스킬마다의 서명.
    fn on_skill(&mut self, id: crate::skills::Id, at: Vec3, dir: Vec3, w: &World) {
        use crate::skills::{Form, Id as Sk};
        let g = vec3(at.x, w.ground_y(at), at.z);
        let d = w.skill_eff(id);
        match (id, d.skill.form) {
            // 휘돌이: 바퀴마다 닿는 거리까지 번지는 금빛 고리 + 둘레에서 바깥으로 튀는 빛줄 + 먼지 고리.
            (Sk::Whirl, Form::Ring { radius, gap, .. }) => {
                self.mark_with(
                    g,
                    MarkKind::Ring {
                        r0: radius * 0.45,
                        r1: radius,
                    },
                    gap,
                    GOLD,
                    0.0,
                    true,
                    false,
                );
                self.ring_sparks(
                    g + Vec3::Y * 0.9,
                    radius * 0.8,
                    14,
                    7.5,
                    0.22,
                    WHITE_HOT,
                    GOLD,
                );
                self.dust_ring(g, radius * 0.55, 12, 5.0, 0.55, DUST);
            }
            // 가로지르기: 떠난 자리의 잔상(큰 민트 별) + 길 전체에 오래 남는 빛 띠 + 뒤로 차이는 먼지.
            (Sk::Cut, Form::Lane { dist, half, .. }) => {
                self.star(g + Vec3::Y * 0.9, 1.9, 0.12, MINT);
                self.star(g + Vec3::Y * 0.9, 1.0, 0.09, WHITE_HOT);
                self.mark(
                    g,
                    MarkKind::Streak {
                        dir,
                        len: dist,
                        half,
                    },
                    0.55,
                    MINT,
                );
                self.dust_kick(g, -dir, 5, 0.5);
            }
            // 끌어모으기: 모이는 점으로 빨려드는 빛줄 + 조여드는 고리 + 한가운데의 작은 민트빛 터짐.
            (Sk::Gather, Form::Pull { range, .. }) => {
                self.mark(
                    g,
                    MarkKind::Converge {
                        r: range * 0.75,
                        n: 12,
                    },
                    0.3,
                    MINT,
                );
                self.mark(
                    g,
                    MarkKind::Ring {
                        r0: range * 0.5,
                        r1: 0.3,
                    },
                    0.28,
                    MINT,
                );
                self.implode(g, range * 0.6, 14, MINT);
                self.star(g + Vec3::Y * 0.6, 1.5, 0.14, MINT);
                self.dust_ring(g, 2.4, 10, -3.2, 0.5, DUST);
            }
            // 땅울림: 찍은 자리의 별 + 부채꼴을 가로질러 달리는 충격파 + 갈라지는 땅 + 튀어 오르는 돌과 먼지.
            (Sk::Quake, Form::Cone { range, arc_deg, .. }) => {
                let c = g + dir * 1.2;
                self.star(c + Vec3::Y * 0.3, 2.2, 0.13, WHITE_HOT);
                self.mark(
                    g,
                    MarkKind::Shock {
                        dir,
                        r0: 0.8,
                        r1: range,
                        arc_deg,
                    },
                    0.26,
                    GOLD,
                );
                self.mark(
                    g,
                    MarkKind::Cracks {
                        dir,
                        r: range * 0.92,
                        n: 5,
                        arc_deg: arc_deg * 0.8,
                    },
                    0.9,
                    GOLD,
                );
                self.dust_ring(c, 0.8, 12, 4.4, 0.7, DUST);
                // 돌 부스러기: 부채꼴 안 여러 자리에서 튀어 오른다.
                let half = (arc_deg * 0.4).to_radians();
                let yaw = dir.x.atan2(dir.z);
                for _ in 0..12 {
                    let a = yaw + self.rs() * half;
                    let rr = range * (0.25 + 0.7 * self.r());
                    let p = g + vec3(a.sin() * rr, 0.1, a.cos() * rr);
                    let up = 4.0 + 3.5 * self.r();
                    let life = 0.45 + 0.2 * self.r();
                    let size = 0.1 + 0.08 * self.r();
                    self.push(Part {
                        pos: p,
                        vel: vec3(a.sin() * 1.5, up, a.cos() * 1.5),
                        age: 0.0,
                        life,
                        size: (size, size * 0.7),
                        color: (STONE, STONE),
                        gravity: 16.0,
                        drag: 0.6,
                        shape: Shape::Dot,
                        decor: false,
                    });
                }
            }
            // 버팀의 터뜨림: 흰 큰 별 + 닿는 거리까지 터지는 고리 둘 + 받아낸 만큼 많아지는 금빛 줄 + 먼지.
            (Sk::Brace, Form::Brace { radius, .. }) => {
                let n = w.player.skill.brace_soaked as usize;
                self.star(g + Vec3::Y * 0.8, 2.0 + 0.4 * n as f32, 0.13, WHITE_HOT);
                self.mark(
                    g,
                    MarkKind::Ring {
                        r0: 0.5,
                        r1: radius,
                    },
                    0.24,
                    WHITE_HOT,
                );
                self.mark_with(
                    g,
                    MarkKind::Ring {
                        r0: 0.3,
                        r1: radius,
                    },
                    0.3,
                    GOLD,
                    0.05,
                    false,
                    false,
                );
                self.ring_sparks(
                    g + Vec3::Y * 0.7,
                    0.6,
                    10 + 5 * n,
                    9.0 + 1.5 * n as f32,
                    0.26,
                    WHITE_HOT,
                    GOLD,
                );
                self.dust_ring(g, 0.8, 10, 4.5, 0.6, DUST);
            }
            // 되울림을 둘렀다: 가운데에서 번지는 잔물결 셋(작게 — 닿는 거리의 반까지) + 가슴 높이의 민트빛 별.
            (Sk::Echo, Form::Echo { radius, .. }) => {
                self.star(g + Vec3::Y * 1.1, 1.4, 0.13, MINT);
                for i in 0..3 {
                    self.mark_with(
                        g,
                        MarkKind::Ring {
                            r0: 0.3,
                            r1: radius * 0.55,
                        },
                        0.4,
                        if i == 0 { WHITE_HOT } else { MINT },
                        0.09 * i as f32,
                        true,
                        false,
                    );
                }
            }
            // 던지기: 손을 떠나는 순간의 흰 별 + 던진 쪽으로 튀는 금빛 줄.
            (Sk::Throw, _) => {
                let hand = g + Vec3::Y * 1.0 + dir * 0.6;
                self.star(hand, 1.2, 0.09, WHITE_HOT);
                self.spray(
                    hand,
                    dir,
                    0.3,
                    6,
                    8.0,
                    Shape::Streak,
                    0.07,
                    0.18,
                    2.0,
                    WHITE_HOT,
                    GOLD,
                );
                self.dust_kick(g, -dir, 3, 0.4);
            }
            // 잔불 흩기: 가운데에서 가장자리로 번지는 호박빛 고리 + 사방으로 흩어지는 불티.
            (Sk::Embers, Form::Field { radius, .. }) => {
                self.star(g + Vec3::Y * 0.6, 1.5, 0.12, GOLD);
                self.mark(
                    g,
                    MarkKind::Ring {
                        r0: 0.4,
                        r1: radius,
                    },
                    0.3,
                    AMBER,
                );
                for _ in 0..22 {
                    let a = self.r() * std::f32::consts::TAU;
                    let sp = radius * (1.4 + 1.6 * self.r());
                    let life = 0.5 + 0.3 * self.r();
                    let up = 1.5 + 2.5 * self.r();
                    self.push(Part {
                        pos: g + Vec3::Y * 0.7,
                        vel: vec3(a.sin() * sp, up, a.cos() * sp),
                        age: 0.0,
                        life,
                        size: (0.11, 0.05),
                        color: (GOLD, fade(AMBER)),
                        gravity: 5.0,
                        drag: 3.0,
                        shape: Shape::Dot,
                        decor: false,
                    });
                }
            }
            _ => {}
        }
    }

    /// 상태 변화 → 몸짓 먼지(사건이 없는 것): 회피 시작, 뛰어오름·내려앉음, 달리기, 적의 돌진·내리침. 틱마다 한 번.
    pub fn watch(&mut self, w: &World, dt: f32) {
        let pl = &w.player;
        let feet = vec3(pl.pos.x, w.ground_y(pl.pos), pl.pos.z);
        let dodging = matches!(pl.state, PState::Dodge { .. });
        if dodging && !self.was_dodging {
            if let PState::Dodge { dir, .. } = pl.state {
                self.dust_kick(feet, -dir, 4, 0.5);
            }
        }
        self.was_dodging = dodging;
        let air = matches!(pl.state, PState::Jump { .. });
        if air && !self.was_air {
            self.dust_ring(feet, 0.15, 4, 1.4, 0.4, DUST);
        }
        // 내려찍기의 착지는 `Event::Plunge`가 더 크게 낸다 — 여기는 보통 착지만.
        if !air && self.was_air && matches!(pl.state, PState::Land { .. }) {
            self.dust_ring(feet, 0.2, 5, 1.8, 0.45, DUST);
        }
        self.was_air = air;
        // 달리기(회피를 쥔 채): 1.3m마다 발뒤로 한 덩이.
        if let Some(last) = self.last_pos {
            let step = vec3(pl.pos.x - last.x, 0.0, pl.pos.z - last.z);
            if pl.running && !air && step.length() < 1.0 {
                self.run_dist += step.length();
                if self.run_dist > 1.3 {
                    self.run_dist = 0.0;
                    self.dust_kick(feet, -step, 1, 0.38);
                }
            } else {
                self.run_dist = 0.0;
            }
        }
        self.last_pos = Some(pl.pos);
        // 0158 G2: 버팀이 끊기면(회피·맞음) 조여들던 고리를 지운다 — 버티는 동안에만 보이는 것.
        let bracing = matches!(
            pl.state,
            PState::Skill {
                id: crate::skills::Id::Brace,
                ..
            }
        );
        if !bracing {
            self.marks.retain(|m| !m.hold);
        }
        // 던진 무기의 꼬리: 날아가는 동안 틱마다 지나온 자리에 금빛 줄(굽은 길이 남는다).
        for b in &w.skill_blades {
            let side = vec3(b.dir.z, 0.0, -b.dir.x) * (b.age * 31.0).sin();
            let vel = -b.dir * 2.0 + side * 2.5;
            let part = Part {
                pos: b.pos + side * 0.35,
                vel,
                age: 0.0,
                life: 0.2,
                size: (0.1, 0.03),
                color: (GOLD, fade(WHITE_HOT)),
                gravity: 0.0,
                drag: 2.0,
                shape: Shape::Streak,
                decor: false,
            };
            self.push(part);
        }
        // 몰아치기의 기운: 이어지는 동안 몸 둘레에서 금빛 알갱이가 솟는다(0.05초마다 하나 — 적게, 오래).
        if pl.skill.flurry_left > 0.0 {
            self.aura_t += dt;
            if self.aura_t >= 0.05 {
                self.aura_t = 0.0;
                let a = self.r() * std::f32::consts::TAU;
                let rr = 0.35 + 0.35 * self.r();
                let up = 1.6 + 1.2 * self.r();
                let h = 0.2 + 0.8 * self.r();
                self.push(Part {
                    pos: feet + vec3(a.sin() * rr, h, a.cos() * rr),
                    vel: vec3(0.0, up, 0.0),
                    age: 0.0,
                    life: 0.45,
                    size: (0.09, 0.03),
                    color: (GOLD, fade(WHITE_HOT)),
                    gravity: -0.5,
                    drag: 1.0,
                    shape: Shape::Dot,
                    decor: false,
                });
            }
        }
        // 적: 돌진하는 동안 발밑 먼지(0.09초마다), 내리치는 틱(예고가 끝나고 휘두름·파동으로 넘어간 틱)에 발 앞 먼지.
        self.enemy_was.resize(w.enemies.len(), (false, false, 0.0));
        for (i, en) in w.enemies.iter().enumerate() {
            let (was_charge, was_tele, mut since) = self.enemy_was[i];
            let charge = matches!(en.state, EState::Charge { .. });
            let tele = matches!(en.state, EState::Tele { .. });
            let g = vec3(en.pos.x, w.ground_y(en.pos), en.pos.z);
            if charge {
                since += dt;
                if !was_charge || since > 0.09 {
                    since = 0.0;
                    self.dust_kick(g, -en.facing_dir(), 2, 0.3 + 0.35 * en.radius);
                }
            }
            let struck = was_tele && matches!(en.state, EState::Swing { .. } | EState::Wave { .. });
            if struck && en.alive() {
                let at = g + en.facing_dir() * (en.radius + 0.5);
                self.dust_ring(
                    at,
                    0.25,
                    if en.boss { 9 } else { 4 },
                    2.0 + en.radius,
                    0.35 + 0.3 * en.radius,
                    DUST,
                );
            }
            self.enemy_was[i] = (charge, tele, since);
        }
    }

    /// 입자를 `dt`만큼 움직이고 다한 것을 치운다.
    pub fn tick(&mut self, dt: f32) {
        for p in &mut self.parts {
            p.age += dt;
            p.vel.y -= p.gravity * dt;
            p.vel *= (1.0 - p.drag * dt).max(0.0);
            p.pos += p.vel * dt;
        }
        self.parts.retain(|p| p.age < p.life);
        // 늦게 나올 입자·남는 빛(0158 G2).
        let mut due = Vec::new();
        self.later.retain_mut(|(left, p)| {
            *left -= dt;
            if *left <= 0.0 {
                due.push(*p);
                false
            } else {
                true
            }
        });
        for p in due {
            self.push(p);
        }
        for m in &mut self.marks {
            if m.delay > 0.0 {
                m.delay -= dt;
            } else {
                m.age += dt;
            }
        }
        self.marks.retain(|m| m.age < m.life);
    }

    /// 판이 바뀔 때(다른 스테이지·시나리오): 남은 입자와 지켜보던 상태를 비운다.
    pub fn clear(&mut self) {
        let seed = self.seed;
        *self = Fx {
            seed,
            ..Default::default()
        };
    }
}

/// 화로에서 오르는 불티(상태 없음 — KB "환경 입자"): 화로 `k`의 `i`번째 불티가 시각 `time`에 있는 자리(화로 위 높이 0 기준)와
/// 진하기(0..1). 주기마다 그릇에서 떠올라 흔들리며 식는다.
pub fn brazier_ember(k: usize, i: usize, time: f32) -> (Vec3, f32) {
    let h = |n: u32| {
        let mut x = (k as u32).wrapping_mul(73_856_093)
            ^ (i as u32).wrapping_mul(19_349_663)
            ^ n.wrapping_mul(83_492_791);
        x ^= x >> 13;
        x = x.wrapping_mul(0x5bd1_e995);
        x ^= x >> 15;
        (x & 0xffff) as f32 / 65535.0
    };
    let period = 1.5 + 1.3 * h(1);
    let ph = ((time + h(2) * period) / period).fract();
    let a = h(3) * std::f32::consts::TAU;
    let r = 0.05 + 0.16 * h(4);
    let sway = (time * (1.5 + 2.0 * h(5)) + h(6) * 6.0).sin() * 0.10 * ph;
    let pos = vec3(
        a.sin() * r + sway,
        0.15 + (0.9 + 0.7 * h(7)) * ph,
        a.cos() * r,
    );
    // 떠오르자마자 밝고, 위로 갈수록 식는다(끝 30%에서 사라짐).
    let k = (ph / 0.1).min(1.0) * ((1.0 - ph) / 0.3).min(1.0);
    (pos, k)
}
/// 화로 하나의 불티 수.
pub const BRAZIER_EMBERS: usize = 5;

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> World {
        World::new("empty", 16.0)
    }

    /// 스킬·큰 기술이 내는 모든 사건(0158 G2) — 스킬 여덟의 시동과 판정, 남긴 것들의 사건, 큰 기술 셋.
    fn skill_events() -> Vec<Event> {
        let (at, dir) = (vec3(0.0, 0.0, 1.0), Vec3::Z);
        let mut ev = Vec::new();
        for id in crate::skills::IDS {
            ev.push(Event::SkillStart { id });
            ev.push(Event::Skill { id, at, dir });
        }
        ev.extend([
            Event::SkillBurst { at },
            Event::Soak { at, n: 2 },
            Event::EchoRing { at, radius: 4.0 },
            Event::ThrowTurn { at },
            Event::ThrowBoom { at, radius: 2.5 },
            Event::ThrowBack { at },
            Event::FieldPulse { at, radius: 4.0 },
            Event::Flurry { at },
            Event::LineCut {
                at,
                dir,
                len: 12.0,
                half: 1.2,
            },
            Event::UltStart,
            Event::Ult { at },
        ]);
        ev
    }

    /// 0158 G2: 스킬마다 서명이 다르다 — 판정이 나간 순간에 남는 빛(`marks`)의 꼴·수가 스킬 여덟에서 서로 다르고,
    /// 타격인 스킬은 한가운데 번쩍임(0.15초 안)을 낸다. 큰 기술 셋도 저마다의 꼴을 남긴다.
    #[test]
    fn every_skill_leaves_its_own_signature() {
        let w = world();
        let (at, dir) = (vec3(0.0, 0.0, 1.0), Vec3::Z);
        let sign = |e: Event| {
            let mut fx = Fx::default();
            fx.on_events(&[e], &w);
            for p in fx.parts.iter().filter(|p| p.shape == Shape::Star) {
                assert!(p.life <= 0.15, "{e:?}: the flash lingers {}", p.life);
            }
            let kinds: Vec<String> = fx
                .marks
                .iter()
                .map(|m| format!("{:?}", std::mem::discriminant(&m.kind)))
                .collect();
            let stars = fx.parts.iter().filter(|p| p.shape == Shape::Star).count();
            assert!(!fx.marks.is_empty() || !fx.parts.is_empty(), "{e:?}");
            (
                kinds,
                stars,
                fx.parts.len(),
                fx.marks.first().map(|m| m.color),
            )
        };
        let mut seen = Vec::new();
        for id in crate::skills::IDS {
            let s = sign(Event::Skill { id, at, dir });
            assert!(s.1 >= 1 || !s.0.is_empty(), "{id:?}: a flash or a mark");
            assert!(!seen.contains(&s), "{id:?} looks like another skill");
            seen.push(s);
        }
        // 큰 기술: 쟁반 뒤집기 = 땅의 금, 한 줄기 = 빛의 면, 몰아치기 = 고리 — 서로 다른 꼴.
        let has = |e: Event, f: fn(&MarkKind) -> bool| {
            let mut fx = Fx::default();
            fx.on_events(&[e], &w);
            fx.marks.iter().any(|m| f(&m.kind))
        };
        assert!(has(Event::Ult { at }, |k| matches!(
            k,
            MarkKind::Cracks { .. }
        )));
        let line = Event::LineCut {
            at,
            dir,
            len: 12.0,
            half: 1.2,
        };
        assert!(has(line, |k| matches!(k, MarkKind::Slash { .. })));
        assert!(has(Event::Flurry { at }, |k| matches!(
            k,
            MarkKind::Ring { .. }
        )));
        // 한 줄기의 불똥은 늦게 — 길을 따라 차례로 터진다.
        let mut fx = Fx::default();
        fx.on_events(&[line], &w);
        let now = fx.parts.len();
        assert!(!fx.later.is_empty());
        for _ in 0..20 {
            fx.tick(crate::tuning::TICK_DT);
        }
        assert!(fx.later.is_empty(), "all out within a third of a second");
        assert!(fx.parts.len() > now / 2, "the late sparks came out");
    }

    /// 남는 빛은 다 사라지고(가장 긴 것도 1.5초 안), 수는 상한을 넘지 않는다. 버티는 동안의 고리는 버팀이 끊기면 지워진다.
    #[test]
    fn marks_fade_out_are_capped_and_the_brace_ring_goes_when_the_hold_ends() {
        let mut w = world();
        let mut fx = Fx::default();
        fx.on_events(&skill_events(), &w);
        assert!(!fx.marks.is_empty());
        for _ in 0..120 {
            fx.on_events(&[Event::Ult { at: Vec3::ZERO }], &w);
        }
        assert!(fx.marks.len() <= MARK_CAP);
        for _ in 0..100 {
            fx.tick(crate::tuning::TICK_DT);
        }
        assert!(fx.marks.is_empty(), "{} left", fx.marks.len());
        // 버팀: 시동에 조여드는 고리가 걸리고, 버티는 동안은 남는다 — 상태가 풀리면 다음 틱에 지워진다.
        let id = crate::skills::Id::Brace;
        w.player.state = PState::Skill {
            id,
            t: 0.0,
            dir: Vec3::Z,
            swing: 1,
        };
        let mut fx = Fx::default();
        fx.on_events(&[Event::SkillStart { id }], &w);
        fx.watch(&w, crate::tuning::TICK_DT);
        assert!(fx.marks.iter().any(|m| m.hold && m.follow));
        w.player.state = PState::Idle;
        fx.watch(&w, crate::tuning::TICK_DT);
        assert!(fx.marks.iter().all(|m| !m.hold));
    }

    /// 잔불 흩기의 불티(상태 없음): 원판 안에서만 놀고(반지름 1·높이 2m 안) 주기 끝에서는 꺼져 있다.
    #[test]
    fn field_motes_stay_inside_the_disc_and_fade_out() {
        for i in 0..FIELD_MOTES_MAX {
            let mut seen = 0.0f32;
            for s in 0..300 {
                let (p, a) = field_mote(i, s as f32 * 0.02);
                assert!((0.0..=1.0).contains(&a));
                assert!(
                    p.y > 0.0 && p.y < 2.0 && vec2(p.x, p.z).length() < 1.05,
                    "{p}"
                );
                seen = seen.max(a);
            }
            assert!(seen > 0.9);
        }
        assert!(FIELD_MOTES <= FIELD_MOTES_MAX);
    }

    /// 타격은 겹이다: 한가운데 번쩍임(아주 짧다)이 꼭 있고, 치명타·강타·패링은 평타보다 크다.
    #[test]
    fn every_hit_event_flashes_and_bigger_hits_flash_bigger() {
        let w = world();
        let at = vec3(0.0, 1.0, 2.0);
        let star = |e: Event| {
            let mut fx = Fx::default();
            fx.on_events(&[e], &w);
            let s = fx
                .parts
                .iter()
                .find(|p| p.shape == Shape::Star)
                .copied()
                .unwrap_or_else(|| panic!("{e:?}: no flash"));
            assert!(s.life <= 0.15, "{e:?}: the flash lingers {}", s.life);
            s.size.0
        };
        let hit = star(Event::Hit { at });
        for e in [
            Event::Crit { at },
            Event::Smash { at },
            Event::Parry { at },
            Event::Break { at },
            Event::Boom { at },
        ] {
            assert!(star(e) > hit, "{e:?}");
        }
        star(Event::Deflect { at });
        star(Event::PlayerHurt);
    }

    /// 빨강은 위험 예고 전용(0037) — 어떤 사건의 입자도 붉지 않다. 패링은 차갑고(파랑 ≥ 빨강) 평타는 따뜻하다.
    #[test]
    fn no_effect_is_red_and_a_parry_reads_cool() {
        let w = world();
        let at = vec3(0.0, 1.0, 2.0);
        let all = [
            Event::Hit { at },
            Event::Crit { at },
            Event::Smash { at },
            Event::Parry { at },
            Event::Deflect { at },
            Event::Break { at },
            Event::EnemyDown { at },
            Event::Plunge { at },
            Event::Aftershock { at },
            Event::Slam { at },
            Event::Boom { at },
            Event::Gulp { at },
            Event::Cast { at },
            Event::Perfect { at },
            Event::Charged,
            Event::Rouse { at },
            Event::PlayerHurt,
        ];
        let red = |c: Color| c.r > 0.75 && c.g < 0.45 && c.b < 0.45;
        for e in all.into_iter().chain(skill_events()) {
            let mut fx = Fx::default();
            fx.on_events(&[e], &w);
            // 늦게 나오는 입자까지 본다.
            let mut seen = fx.parts.clone();
            seen.extend(fx.later.iter().map(|(_, p)| *p));
            // 시동의 조짐은 일부 스킬에만 있다(나머지는 판정의 순간에 보인다).
            assert!(
                !seen.is_empty() || !fx.marks.is_empty() || matches!(e, Event::SkillStart { .. }),
                "{e:?}: nothing to see"
            );
            for p in &seen {
                for c in [p.color.0, p.color.1] {
                    assert!(!red(c), "{e:?}: red {c:?}");
                }
            }
            for m in &fx.marks {
                assert!(!red(m.color), "{e:?}: red mark {:?}", m.color);
            }
        }
        let mut fx = Fx::default();
        fx.on_events(&[Event::Parry { at }], &w);
        let s = fx.parts.iter().find(|p| p.shape == Shape::Star).unwrap();
        assert!(s.color.0.b >= s.color.0.r);
    }

    /// 입자는 다 사라진다(가장 긴 것도 1.5초 안), 같은 사건 = 같은 그림(난수 없음).
    #[test]
    fn particles_die_and_the_same_events_draw_the_same_picture() {
        let w = world();
        let ev = [
            Event::Boom { at: Vec3::ZERO },
            Event::EnemyDown { at: Vec3::ZERO },
            Event::Gulp { at: Vec3::Y },
        ];
        let run = || {
            let mut fx = Fx::default();
            fx.on_events(&ev, &w);
            for _ in 0..10 {
                fx.tick(crate::tuning::TICK_DT);
            }
            fx.parts.iter().map(|p| p.pos).collect::<Vec<_>>()
        };
        assert_eq!(run(), run());
        let mut fx = Fx::default();
        fx.on_events(&ev, &w);
        for _ in 0..90 {
            fx.tick(crate::tuning::TICK_DT);
        }
        assert!(fx.parts.is_empty(), "{} left", fx.parts.len());
    }

    /// 꾸밈(먼지)은 많으면 건너뛰지만 타격은 건너뛰지 않는다 — 떼싸움 한가운데서도 맞힌 번쩍임은 뜬다. 그래도 끝없이 늘지는 않는다.
    #[test]
    fn dust_is_capped_but_hits_always_show() {
        let w = world();
        let mut fx = Fx::default();
        for _ in 0..80 {
            fx.on_events(&[Event::Plunge { at: Vec3::ZERO }], &w);
        }
        let dust = fx.parts.iter().filter(|p| p.decor).count();
        assert!(dust <= DECOR_CAP, "{dust}");
        fx.on_events(&[Event::Hit { at: Vec3::Y }], &w);
        assert!(fx.parts.last().is_some_and(|p| p.shape == Shape::Star));
        for _ in 0..200 {
            fx.on_events(&[Event::Crit { at: Vec3::Y }], &w);
        }
        assert!(fx.parts.len() <= HARD_CAP);
    }

    /// 회피를 시작한 틱에 발밑 먼지가 반대쪽으로 밀리고, 가만히 서 있으면 아무것도 안 난다(목적이 있을 때만).
    #[test]
    fn a_dodge_kicks_dust_backwards_and_standing_still_is_clean() {
        let mut w = world();
        let mut fx = Fx::default();
        for _ in 0..30 {
            fx.watch(&w, crate::tuning::TICK_DT);
        }
        assert!(fx.parts.is_empty(), "idle dust");
        w.player.state = PState::Dodge {
            t: 0.0,
            dir: Vec3::Z,
        };
        fx.watch(&w, crate::tuning::TICK_DT);
        assert!(!fx.parts.is_empty());
        assert!(fx.parts.iter().all(|p| p.decor && p.shape == Shape::Puff));
        assert!(
            fx.parts.iter().map(|p| p.vel.z).sum::<f32>() < 0.0,
            "dust trails behind"
        );
        let n = fx.parts.len();
        fx.watch(&w, crate::tuning::TICK_DT);
        assert_eq!(fx.parts.len(), n, "once per dodge");
    }

    /// 화로 불티: 그릇 위에서만 놀고(반지름 0.4m·높이 2m 안) 주기 끝에서는 꺼져 있다 — 튀어 사라지지 않는다.
    #[test]
    fn brazier_embers_rise_above_the_bowl_and_fade_out() {
        for k in 0..6 {
            for i in 0..BRAZIER_EMBERS {
                let mut seen = 0.0f32;
                for s in 0..400 {
                    let (p, a) = brazier_ember(k, i, s as f32 * 0.02);
                    assert!((0.0..=1.0).contains(&a));
                    assert!(
                        p.y > 0.0 && p.y < 2.0 && vec2(p.x, p.z).length() < 0.4,
                        "{p}"
                    );
                    if p.y > 1.4 {
                        assert!(a < 0.75);
                    }
                    seen = seen.max(a);
                }
                assert!(seen > 0.9);
            }
        }
    }
}
