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
                // 스킬(0151 — 금·민트·흰빛, 빨강 없음). 판정이 나간 자리의 번쩍임 + 땅의 먼지 고리, 모양마다 한 겹씩 다르다.
                Event::Skill { id, at, dir } => {
                    use crate::skills::Id as Sk;
                    let g = vec3(at.x, w.ground_y(at), at.z);
                    match id {
                        // 휘돌이: 둘레로 퍼지는 먼지 고리와 허리 높이의 빛줄.
                        Sk::Whirl => {
                            self.dust_ring(g, 1.6, 12, 5.0, 0.55, DUST);
                            self.spray(
                                g + Vec3::Y * 0.9,
                                Vec3::Y,
                                1.0,
                                10,
                                8.0,
                                Shape::Streak,
                                0.07,
                                0.2,
                                2.0,
                                WHITE_HOT,
                                GOLD,
                            );
                        }
                        // 가로지르기: 떠난 자리의 별과 뒤로 차이는 먼지.
                        Sk::Cut => {
                            self.star(g + Vec3::Y * 0.9, 1.2, 0.09, WHITE_HOT);
                            self.dust_kick(g, -dir, 4, 0.5);
                        }
                        // 끌어모으기: 모이는 점의 민트빛 별과 안쪽으로 감기는 먼지.
                        Sk::Gather => {
                            self.star(g + Vec3::Y * 0.6, 1.5, 0.14, MINT);
                            self.dust_ring(g, 2.4, 10, -3.2, 0.5, DUST);
                        }
                        // 땅울림: 찍은 자리의 별 + 앞으로 터지는 돌 부스러기 + 넓은 먼지 고리.
                        Sk::Quake => {
                            let c = g + dir * 1.6;
                            self.star(c + Vec3::Y * 0.3, 1.8, 0.12, WHITE_HOT);
                            self.dust_ring(c, 0.8, 12, 4.4, 0.7, DUST);
                            self.spray(
                                c + Vec3::Y * 0.1,
                                dir + Vec3::Y * 0.8,
                                0.6,
                                9,
                                6.0,
                                Shape::Dot,
                                0.12,
                                0.5,
                                14.0,
                                STONE,
                                STONE,
                            );
                        }
                    }
                }
                // 큰 기술: 시동은 발밑에 모이는 금빛, 뒤집히는 순간은 가장 큰 별 + 두 겹 먼지 고리 + 사방의 금빛 줄.
                Event::UltStart => {
                    self.star(pp + Vec3::Y * 0.4, 1.4, 0.3, GOLD);
                    self.dust_ring(vec3(pp.x, w.ground_y(pp), pp.z), 2.6, 12, -2.6, 0.45, DUST);
                }
                Event::Ult { at } => {
                    let g = vec3(at.x, w.ground_y(at), at.z);
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
        for e in all {
            let mut fx = Fx::default();
            fx.on_events(&[e], &w);
            assert!(!fx.parts.is_empty(), "{e:?}: nothing to see");
            for p in &fx.parts {
                for c in [p.color.0, p.color.1] {
                    assert!(
                        !(c.r > 0.75 && c.g < 0.45 && c.b < 0.45),
                        "{e:?}: red {c:?}"
                    );
                }
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
