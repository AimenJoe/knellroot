//! 주위 환경(0063): 놀이 칸 밖·아래를 채우는 것. 월드를 읽기만 하고(렌더의 일부), 로직은 모른다.
//!
//! - **높이 안개(E1)**: 걸을 수 있는 가장 낮은 땅보다 아래에 옅은 안개 판을 몇 장 겹친다. 깊이 버퍼가 땅 위의 판을 가리므로
//!   낭떠러지·꺼진 바닥·벽 너머만 깊을수록 판을 더 많이 지나 안개에 묻힌다(셰이더 없이 높이 안개). 판마다 다른 쪽으로 천천히 흐르는
//!   점 무늬(가장 가까운 픽셀) — 검은 허공 대신 "깊다"로 읽히게.
//! - **떠다니는 것(E4)**: 반딧불(따뜻한 노랑, 깜빡이며 풀 높이에서 헤맨다)·달빛 먼지(옅은 라벤더, 천천히 가라앉는다).
//!   상태 없이 시각만으로 자리를 정한다 — 플레이어를 가운데 둔 상자에서 감아 돈다(끝없는 들판, 가장자리는 옅게).
//! - **원경(E2)**: 하늘 띠(카메라 둘레 원통, 높이각마다 한 빛 — 계단진 해 질 녘), 달빛이 오는 쪽의 달, 스테이지 둘레 세 겹 산등성이
//!   (멀수록 하늘빛에 가깝게 — 공기 원근, 꼭대기에 달빛 테)와 가운데 겹의 거대한 뿌리 아치("뿌리에 먹힌 땅"을 한 장에).
//!   먼저 그린다(땅·벽이 깊이로 가린다). 법선 패스에는 넣지 않는다 — 먼 것엔 선이 없다.
//!   하늘 원통은 먼 클립(200m) 안에 들게 높이각 34°까지만, 그 위는 지우는 빛(`ZENITH`)이 이어 준다.
use macroquad::prelude::*;

use crate::ground::Ground;
use crate::render::Batch;

/// 안개 빛(해 질 녘 남보라 — 0037 세계 빛, 아지랑이·하늘과 같은 계열).
pub const FOG: Color = Color::new(0.30, 0.28, 0.46, 1.0);
/// 허공 판 빛 = 안개 가장 깊은 곳(옛 검정 0.06·0.06·0.10 대신).
pub const FOG_DEEP: Color = Color::new(0.20, 0.19, 0.33, 1.0);
/// 걷는 땅보다 이만큼 아래부터 안개(걷는 땅·배우는 안개 밖).
const FOG_CLEAR: f32 = 0.3;
/// 판 사이 간격(m)·판 하나의 진하기.
const FOG_GAP: f32 = 0.5;
const FOG_ALPHA: f32 = 0.3;
/// 판은 이 크기의 바둑판(무늬 한 장)으로, 카메라가 보는 곳 둘레 `FOG_REACH` 안만 깐다(정점 한도).
const FOG_TILE: f32 = 8.0;
const FOG_REACH: f32 = 42.0;
/// 무늬 한 장의 텍셀 수(한 텍셀 = 25cm — 게임 거리에서 2px 안팎).
const FOG_TEX: u16 = 32;
/// 판마다 흐르는 빠르기(m/s)·쪽(판 번호로 돌린다).
const FOG_DRIFT: f32 = 0.25;

pub struct Backdrop {
    fog_tex: Option<Texture2D>,
    halo_tex: Option<Texture2D>,
    /// 먼지·재 덩이 한 장(0141 — 빛무리보다 진한 계단진 원판).
    puff_tex: Option<Texture2D>,
    /// 지금 지역의 환경(0118 T2 — 렌더가 프레임마다 스테이지에서 넣는다).
    pub env: &'static Env,
}

impl Default for Backdrop {
    fn default() -> Self {
        Self {
            fog_tex: None,
            halo_tex: None,
            puff_tex: None,
            env: &SURFACE,
        }
    }
}

/// 지역 환경(0118 T2, 0114 표): 땅 위(하늘·달·반딧불)와 땅속(동굴 원경·약한 달빛·떨어지는 흙)을 한 표로.
/// 원경 꼴(하늘 원통·산등성이 세 겹·뿌리 아치)은 같고 빛만 바꾼다 — 땅속에선 산등성이가 높아져 동굴 벽, 아치는 뿌리 천장.
pub struct Env {
    /// 하늘 띠 빛(`SKY_BANDS` 높이각 차례)·원통 위를 지우는 빛.
    pub sky: [Color; 6],
    pub zenith: Color,
    /// 산등성이 세 겹의 (몸빛, 꼭대기 테)와 높이 배율(땅속 = 동굴 벽).
    pub ridges: [(Color, Color); 3],
    pub ridge_h: f32,
    /// 달을 그리나, 달빛 세기(굽는 지형·배우의 `lit` 달빛 몫에 곱한다).
    pub moon: bool,
    pub moonlight: f32,
    /// 안개 빛·허공 판 빛.
    pub fog: Color,
    pub fog_deep: Color,
    /// 반딧불, 떨어지는 흙(천장에서), 나무·덤불(벽 뿌리는 둘 다).
    pub fireflies: bool,
    pub falling: bool,
    pub trees: bool,
}

/// 땅 위(성소): 해 질 녘 남보라 하늘·달(0063 그대로).
pub static SURFACE: Env = Env {
    sky: [
        Color::new(0.22, 0.20, 0.34, 1.0),
        Color::new(0.40, 0.30, 0.44, 1.0),
        Color::new(0.33, 0.26, 0.43, 1.0),
        Color::new(0.25, 0.21, 0.39, 1.0),
        Color::new(0.18, 0.16, 0.33, 1.0),
        Color::new(0.12, 0.11, 0.25, 1.0),
    ],
    zenith: ZENITH,
    ridges: [
        (
            Color::new(0.27, 0.22, 0.38, 1.0),
            Color::new(0.36, 0.30, 0.48, 1.0),
        ),
        (
            Color::new(0.19, 0.16, 0.30, 1.0),
            Color::new(0.28, 0.24, 0.40, 1.0),
        ),
        (
            Color::new(0.12, 0.11, 0.21, 1.0),
            Color::new(0.20, 0.18, 0.32, 1.0),
        ),
    ],
    ridge_h: 1.0,
    moon: true,
    moonlight: 1.0,
    fog: FOG,
    fog_deep: FOG_DEEP,
    fireflies: true,
    falling: false,
    trees: true,
};

/// 땅속(떠난 마을, 지하 40m): 하늘 대신 흙빛이 아래로 짙어지는 동굴, 높은 동굴 벽, 뿌리 천장(아치), 달 없음·달빛은 천장 틈으로 조금,
/// 흙이 떨어진다(KB conveying-descent "방 안의 위"). 색은 같은 남보라 계열을 흙빛 쪽으로 낮춘다(0037 — 세계의 빛은 이어진다).
pub static UNDERGROUND: Env = Env {
    sky: [
        Color::new(0.10, 0.08, 0.12, 1.0),
        Color::new(0.17, 0.13, 0.17, 1.0),
        Color::new(0.13, 0.10, 0.15, 1.0),
        Color::new(0.10, 0.08, 0.12, 1.0),
        Color::new(0.07, 0.06, 0.10, 1.0),
        Color::new(0.05, 0.045, 0.08, 1.0),
    ],
    zenith: Color::new(0.04, 0.035, 0.06, 1.0),
    ridges: [
        (
            Color::new(0.16, 0.12, 0.15, 1.0),
            Color::new(0.24, 0.18, 0.21, 1.0),
        ),
        (
            Color::new(0.11, 0.09, 0.12, 1.0),
            Color::new(0.18, 0.14, 0.17, 1.0),
        ),
        (
            Color::new(0.07, 0.06, 0.09, 1.0),
            Color::new(0.13, 0.10, 0.13, 1.0),
        ),
    ],
    ridge_h: 2.2,
    moon: false,
    moonlight: 0.4,
    fog: Color::new(0.16, 0.13, 0.20, 1.0),
    fog_deep: Color::new(0.09, 0.075, 0.12, 1.0),
    fireflies: false,
    falling: true,
    trees: false,
};

/// 이어 붙여도 이음매가 없는 점 무늬(정수 주파수 사인 합) → 세 단 알파(픽셀 그림처럼 계단).
fn fog_pixels(n: u16) -> Vec<u8> {
    let n = n as usize;
    let mut px = vec![255u8; n * n * 4];
    let tau = std::f32::consts::TAU;
    for y in 0..n {
        for x in 0..n {
            let (u, v) = (x as f32 / n as f32, y as f32 / n as f32);
            let f = 0.5
                + 0.22 * (tau * (2.0 * u + v)).sin()
                + 0.16 * (tau * (3.0 * u - 2.0 * v) + 1.3).sin()
                + 0.12 * (tau * (5.0 * v + u) + 2.1).sin();
            let a = if f > 0.74 {
                1.0
            } else if f > 0.5 {
                0.55
            } else {
                0.12
            };
            px[(y * n + x) * 4 + 3] = (a * 255.0) as u8;
        }
    }
    px
}

/// 판 하나: `y` 높이, 가운데 `c` 둘레를 바둑판으로, 무늬는 `shift`만큼 밀려 있다(바둑판째 민다 — 텍스처 감기 없이 흐름).
fn fog_layer(
    verts: &mut Vec<Vertex>,
    idx: &mut Vec<u16>,
    c: Vec2,
    y: f32,
    shift: Vec2,
    col: Color,
) {
    let n = (FOG_REACH / FOG_TILE).ceil() as i32;
    let ox = (c.x / FOG_TILE).floor() * FOG_TILE + shift.x.rem_euclid(FOG_TILE);
    let oz = (c.y / FOG_TILE).floor() * FOG_TILE + shift.y.rem_euclid(FOG_TILE);
    for j in -n..n {
        for i in -n..n {
            let x0 = ox + i as f32 * FOG_TILE;
            let z0 = oz + j as f32 * FOG_TILE;
            let base = verts.len() as u16;
            for (dx, dz) in [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)] {
                verts.push(Vertex::new(
                    x0 + dx * FOG_TILE,
                    y,
                    z0 + dz * FOG_TILE,
                    dx,
                    dz,
                    col,
                ));
            }
            idx.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }
}

/// 안개 판이 몇 장, 어느 높이에(위 → 아래). 걷는 땅 아래에 허공이 없으면 없다.
pub fn fog_heights(walk_min: f32, void_y: f32) -> Vec<f32> {
    let top = walk_min - FOG_CLEAR;
    let mut out = Vec::new();
    let mut y = top;
    while y > void_y + 0.05 {
        out.push(y);
        y -= FOG_GAP;
    }
    out
}

impl Backdrop {
    /// 높이 안개(E1). 불투명한 것을 다 그린 뒤에 — 깊은 판부터(위 판이 아래 판을 깊이로 막지 않게).
    pub fn fog(&mut self, batch: &mut Batch, center: Vec3, walk_min: f32, void_y: f32, time: f32) {
        let heights = fog_heights(walk_min, void_y);
        if heights.is_empty() {
            return;
        }
        let tex = self
            .fog_tex
            .get_or_insert_with(|| {
                let t = Texture2D::from_rgba8(FOG_TEX, FOG_TEX, &fog_pixels(FOG_TEX));
                t.set_filter(FilterMode::Nearest);
                t
            })
            .clone();
        let f = self.env.fog;
        let col = Color::new(f.r, f.g, f.b, FOG_ALPHA);
        for (k, &y) in heights.iter().enumerate().rev() {
            let a = k as f32 * 2.4 + 0.7;
            let shift = vec2(a.cos(), a.sin()) * FOG_DRIFT * time;
            let mut verts = Vec::new();
            let mut idx = Vec::new();
            fog_layer(
                &mut verts,
                &mut idx,
                vec2(center.x, center.z),
                y,
                shift,
                col,
            );
            batch.raw(&verts, &idx, Some(&tex), 1.0);
            batch.send();
        }
    }
}

/// 떠다니는 것 한 무리.
struct Drift {
    count: u32,
    /// 감아 도는 상자의 가로 반폭(m)·높이 범위.
    half: f32,
    y: (f32, f32),
    /// 흐름(m/s, 무리 전체)·흔들림 크기(m)·빠르기.
    flow: Vec3,
    wobble: f32,
    wobble_hz: f32,
    /// 점 크기(m)·빛무리 크기(0이면 없음)·빛.
    size: f32,
    halo: f32,
    color: Color,
    /// 깜빡임: 0이면 늘 켜짐, 아니면 켜지는 빠르기(Hz).
    blink_hz: f32,
    seed: u32,
    /// 바닥이 없는 칸(벽·허공) 위에서는 보이지 않는다(떨어지는 흙 — 천장에서 바닥으로 떨어지는 것).
    hide_off_ground: bool,
}

/// 반딧불: 풀 높이(0.3~1.8m), 바람 없이 헤매고, 몇 초마다 켜졌다 꺼진다. 빛무리로 둘레를 살짝 밝힌다.
const FIREFLIES: Drift = Drift {
    count: 40,
    half: 13.0,
    y: (0.3, 1.8),
    flow: Vec3::new(0.05, 0.0, 0.03),
    wobble: 0.6,
    wobble_hz: 0.18,
    size: 0.12,
    halo: 0.7,
    color: Color::new(1.0, 0.86, 0.42, 1.0),
    blink_hz: 0.22,
    seed: 11,
    hide_off_ground: false,
};
/// 달빛 먼지: 공중 높이까지(0.2~4m), 옅고 작게, 달 반대쪽으로 천천히 흐르며 가라앉는다.
const MOTES: Drift = Drift {
    count: 70,
    half: 14.0,
    y: (0.2, 4.0),
    flow: Vec3::new(0.12, -0.05, 0.06),
    wobble: 0.25,
    wobble_hz: 0.1,
    size: 0.07,
    halo: 0.0,
    color: Color::new(0.82, 0.82, 1.0, 0.7),
    blink_hz: 0.0,
    seed: 23,
    hide_off_ground: false,
};

/// 떨어지는 흙(땅속, 0118 T2): 천장 높이(6m)에서 곧게 떨어지는 작은 흙빛 점 — 위에 땅이 있다는 신호.
const FALLING: Drift = Drift {
    count: 36,
    half: 13.0,
    y: (0.1, 6.0),
    flow: Vec3::new(0.0, -0.9, 0.0),
    wobble: 0.05,
    wobble_hz: 0.3,
    size: 0.06,
    halo: 0.0,
    color: Color::new(0.62, 0.50, 0.42, 0.85),
    blink_hz: 0.0,
    seed: 31,
    hide_off_ground: true,
};

fn h01(i: u32, k: u32) -> f32 {
    let mut h = i.wrapping_mul(0x9E37_79B9) ^ k.wrapping_mul(0x85EB_CA6B);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    (h & 0xffff) as f32 / 65535.0
}

/// 무리의 `i`번째가 시각 `t`에 어디·얼마나 밝은가(0..1). 플레이어 `c`를 가운데 둔 상자에서 감아 돈다.
fn drift_at(d: &Drift, i: u32, c: Vec3, t: f32, ground: Option<&Ground>) -> (Vec3, f32) {
    let s = d.seed.wrapping_mul(1000) + i;
    let span = d.half * 2.0;
    let base = vec3(h01(s, 1) * span, 0.0, h01(s, 2) * span);
    let ph = h01(s, 3) * std::f32::consts::TAU;
    let w = d.wobble_hz * std::f32::consts::TAU * t;
    let wob = vec3((w + ph).sin(), 0.0, (w * 0.83 + ph * 1.7).cos()) * d.wobble;
    let moved = base + d.flow * t * (0.7 + 0.6 * h01(s, 4)) + wob;
    // 가로는 상자에서 감고(플레이어 기준), 높이는 범위 안에서 감는다(가라앉으면 위에서 다시).
    let lo = vec2(c.x - d.half, c.z - d.half);
    let x = lo.x + (moved.x - lo.x).rem_euclid(span);
    let z = lo.y + (moved.z - lo.y).rem_euclid(span);
    let hspan = d.y.1 - d.y.0;
    let y0 = h01(s, 5) * hspan + d.flow.y * t + 0.2 * (w * 1.3 + ph).sin();
    // 높이는 그 자리의 바닥에서 잰다(마을은 구역마다 바닥이 −1 ~ +4m, 성소 성벽길 +3m — 월드 0에서 재면 바닥에 묻혔다).
    // 바닥이 없는 칸(벽·허공): 떨어지는 흙은 만들지 않고(`hide_off_ground`), 다른 것은 월드 0 기준(허공 위 먼지·반딧불).
    let floor = ground.and_then(|g| {
        let p = vec3(x, 0.0, z);
        let (gx, gz) = g.cell_of(p)?;
        g.height_in(gx, gz, p)
    });
    if d.hide_off_ground && ground.is_some() && floor.is_none() {
        return (vec3(x, 0.0, z), 0.0);
    }
    let y = floor.unwrap_or(0.0) + d.y.0 + y0.rem_euclid(hspan);
    // 가장자리는 옅게(감아 돌 때 튀지 않게): 상자 끝 15%·높이 끝에서 0으로.
    let edge =
        |v: f32, a: f32, span: f32| ((v - a).min(a + span - v) / (span * 0.15)).clamp(0.0, 1.0);
    let mut k = edge(x, lo.x, span) * edge(z, lo.y, span) * edge(y, d.y.0, hspan);
    if d.blink_hz > 0.0 {
        let b = (d.blink_hz * std::f32::consts::TAU * t + ph * 3.0).sin();
        k *= ((b - 0.1) / 0.5).clamp(0.0, 1.0);
    }
    (vec3(x, y, z), k)
}

/// 카메라를 보는 판 하나.
fn billboard(
    verts: &mut Vec<Vertex>,
    idx: &mut Vec<u16>,
    p: Vec3,
    r: Vec3,
    u: Vec3,
    size: f32,
    col: Color,
) {
    let base = verts.len() as u16;
    let (r, u) = (r * size * 0.5, u * size * 0.5);
    for (q, uv) in [
        (p - r - u, vec2(0.0, 1.0)),
        (p + r - u, vec2(1.0, 1.0)),
        (p + r + u, vec2(1.0, 0.0)),
        (p - r + u, vec2(0.0, 0.0)),
    ] {
        verts.push(Vertex::new(q.x, q.y, q.z, uv.x, uv.y, col));
    }
    idx.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

/// 날아가는 쪽으로 늘어난 빛줄 하나(0141): `a`(머리, 빛 `ca`)에서 `b`(꼬리, `cb`)까지, 폭 `w` — 카메라를 본다(`view` = 화면 안쪽).
#[allow(clippy::too_many_arguments)]
fn streak(
    verts: &mut Vec<Vertex>,
    idx: &mut Vec<u16>,
    a: Vec3,
    b: Vec3,
    w: f32,
    view: Vec3,
    ca: Color,
    cb: Color,
) {
    let side = (a - b).cross(view).normalize_or(Vec3::Y) * (w * 0.5);
    let base = verts.len() as u16;
    for (q, c) in [
        (a - side, ca),
        (a + side, ca),
        (b + side, cb),
        (b - side, cb),
    ] {
        verts.push(Vertex::new(q.x, q.y, q.z, 0.0, 0.0, c));
    }
    idx.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

/// 네 갈래 번쩍임(0141): 끝이 뾰족한 가로·세로 마름모 둘(가운데가 두껍다 — 네모 팔보다 "반짝"으로 읽힌다). `size` = 끝에서 끝.
fn star(
    verts: &mut Vec<Vertex>,
    idx: &mut Vec<u16>,
    p: Vec3,
    r: Vec3,
    u: Vec3,
    size: f32,
    col: Color,
) {
    let (h, t) = (size * 0.5, size * 0.13);
    for (long, thin) in [(r, u), (u, r)] {
        let base = verts.len() as u16;
        for q in [p - long * h, p - thin * t, p + long * h, p + thin * t] {
            verts.push(Vertex::new(q.x, q.y, q.z, 0.0, 0.0, col));
        }
        idx.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
}

/// 먼지·재 덩이 한 장(0141): 속이 찬 원판 + 한 단 옅은 테 — 빛무리(가운데도 0.7)로 그리면 어두운 바닥에서 안 보였다.
fn puff_pixels(n: u16) -> Vec<u8> {
    let n = n as usize;
    let mut px = vec![255u8; n * n * 4];
    for y in 0..n {
        for x in 0..n {
            let d = vec2(
                x as f32 + 0.5 - n as f32 * 0.5,
                y as f32 + 0.5 - n as f32 * 0.5,
            )
            .length()
                / (n as f32 * 0.5);
            let a = if d < 0.62 {
                1.0
            } else if d < 0.95 {
                0.55
            } else {
                0.0
            };
            px[(y * n + x) * 4 + 3] = (a * 255.0) as u8;
        }
    }
    px
}

/// 빛무리 한 장(가운데 진하고 가장자리 0, 세 단 계단 — 픽셀 그림처럼).
fn halo_pixels(n: u16) -> Vec<u8> {
    let n = n as usize;
    let mut px = vec![255u8; n * n * 4];
    for y in 0..n {
        for x in 0..n {
            let d = vec2(
                x as f32 + 0.5 - n as f32 * 0.5,
                y as f32 + 0.5 - n as f32 * 0.5,
            )
            .length()
                / (n as f32 * 0.5);
            let a = if d < 0.3 {
                0.7
            } else if d < 0.6 {
                0.35
            } else if d < 1.0 {
                0.14
            } else {
                0.0
            };
            px[(y * n + x) * 4 + 3] = (a * 255.0) as u8;
        }
    }
    px
}

impl Backdrop {
    /// 떠다니는 것(E4): 반딧불·먼지. `right`·`up`은 카메라 화면의 오른쪽·위(월드).
    pub fn motes(
        &mut self,
        batch: &mut Batch,
        c: Vec3,
        right: Vec3,
        up: Vec3,
        time: f32,
        ground: Option<&Ground>,
    ) {
        let mut verts = Vec::new();
        let mut idx = Vec::new();
        let mut halos = Vec::new();
        let mut halo_idx = Vec::new();
        let env = self.env;
        let kinds = [
            Some(&MOTES),
            env.fireflies.then_some(&FIREFLIES),
            env.falling.then_some(&FALLING),
        ];
        for d in kinds.into_iter().flatten() {
            for i in 0..d.count {
                let (p, k) = drift_at(d, i, c, time, ground);
                if k <= 0.02 {
                    continue;
                }
                let col = Color::new(d.color.r, d.color.g, d.color.b, d.color.a * k);
                billboard(&mut verts, &mut idx, p, right, up, d.size, col);
                if d.halo > 0.0 {
                    billboard(&mut halos, &mut halo_idx, p, right, up, d.halo, col);
                }
            }
        }
        if !halos.is_empty() {
            let tex = self
                .halo_tex
                .get_or_insert_with(|| {
                    let t = Texture2D::from_rgba8(8, 8, &halo_pixels(8));
                    t.set_filter(FilterMode::Nearest);
                    t
                })
                .clone();
            batch.raw(&halos, &halo_idx, Some(&tex), 1.0);
            batch.send();
        }
        batch.raw(&verts, &idx, None, 1.0);
        batch.send();
    }
}

/// 화로 불티의 빛(밝게 떠올라 호박빛으로 식는다).
const SPARK_HOT: Color = Color::new(1.0, 0.92, 0.62, 1.0);
const SPARK_COOL: Color = Color::new(1.0, 0.55, 0.2, 1.0);

impl Backdrop {
    /// 이펙트 입자(0141): 사건의 입자(`parts` — 번쩍임·불티·먼지·재), 맞힌 자리의 불똥(월드 `sparks`), 화로에서 오르는 불티
    /// (`braziers` = 그릇 꼭대기 자리, 상태 없음). 모두 카메라를 보는 판 — 먼지·재만 계단진 원판 텍스처, 나머지는 네모 화소.
    /// 법선 패스 밖(선이 붙지 않는다). 따뜻하고 밝은 것은 마무리 패스(0140)가 번지게 한다.
    #[allow(clippy::too_many_arguments)]
    pub fn fx(
        &mut self,
        batch: &mut Batch,
        parts: &[crate::fx::Part],
        sparks: &[crate::world::Spark],
        braziers: &[Vec3],
        right: Vec3,
        up: Vec3,
        time: f32,
    ) {
        use crate::fx::Shape;
        let view = right.cross(up);
        let (mut verts, mut idx) = (Vec::new(), Vec::new());
        let (mut puffs, mut puff_idx) = (Vec::new(), Vec::new());
        for p in parts {
            let (size, col) = (p.size_now(), p.color_now());
            match p.shape {
                Shape::Dot => billboard(&mut verts, &mut idx, p.pos, right, up, size, col),
                Shape::Puff => billboard(&mut puffs, &mut puff_idx, p.pos, right, up, size, col),
                Shape::Star => star(&mut verts, &mut idx, p.pos, right, up, size, col),
                Shape::Streak => {
                    let tail = p.pos - p.vel * 0.045;
                    let end = Color::new(col.r, col.g, col.b, 0.0);
                    streak(&mut verts, &mut idx, p.pos, tail, size, view, col, end);
                }
            }
        }
        // 불똥(2026-11-06 → 0141): 1px 선이던 것을 두 화소 굵기의 빛줄로 — 흰빛에서 호박빛으로 식으며 꼬리가 옅어진다.
        for sp in sparks {
            let k = (sp.age / crate::world::SPARK_LIFE).clamp(0.0, 1.0);
            let col = Color::new(1.0, 1.0 - 0.35 * k, 0.85 - 0.6 * k, 1.0 - k * k);
            let (w, tail) = if sp.big { (0.11, 0.055) } else { (0.08, 0.04) };
            let end = Color::new(col.r, col.g * 0.8, col.b * 0.6, 0.0);
            streak(
                &mut verts,
                &mut idx,
                sp.pos,
                sp.pos - sp.vel * tail,
                w,
                view,
                col,
                end,
            );
        }
        for (k, b) in braziers.iter().enumerate() {
            for i in 0..crate::fx::BRAZIER_EMBERS {
                let (off, a) = crate::fx::brazier_ember(k, i, time);
                if a <= 0.02 {
                    continue;
                }
                // 위로 갈수록(주기의 뒤쪽) 식는다.
                let cool = (off.y / 1.6).clamp(0.0, 1.0);
                let col = Color::new(
                    SPARK_HOT.r + (SPARK_COOL.r - SPARK_HOT.r) * cool,
                    SPARK_HOT.g + (SPARK_COOL.g - SPARK_HOT.g) * cool,
                    SPARK_HOT.b + (SPARK_COOL.b - SPARK_HOT.b) * cool,
                    a,
                );
                billboard(&mut verts, &mut idx, *b + off, right, up, 0.07, col);
            }
        }
        if !puffs.is_empty() {
            let tex = self
                .puff_tex
                .get_or_insert_with(|| {
                    // 16칸: 락온 거리(1m ≈ 46px)에서 0.7m 덩이의 텍셀이 2px — 8칸이면 네모 벽돌처럼 보였다.
                    let t = Texture2D::from_rgba8(16, 16, &puff_pixels(16));
                    t.set_filter(FilterMode::Nearest);
                    t
                })
                .clone();
            // 입자 수 상한(`fx::HARD_CAP`)이라 한 번의 그리기 용량 안이다.
            batch.raw(&puffs, &puff_idx, Some(&tex), 1.0);
            batch.send();
        }
        if !verts.is_empty() {
            batch.raw(&verts, &idx, None, 1.0);
            batch.send();
        }
    }
}

/// 하늘 꼭대기 = 렌더 타깃을 지우는 빛(원통 위는 이것).
pub const ZENITH: Color = Color::new(0.08, 0.08, 0.18, 1.0);
/// 하늘 띠: (아래 높이각°, 위 높이각°) — 빛은 지역 환경(`Env::sky`). 땅 위는 지평선 바로 위가 가장 밝은 해 질 녘 띠.
const SKY_BANDS: [(f32, f32); 6] = [
    (-25.0, -3.0),
    (-3.0, 2.0),
    (2.0, 6.0),
    (6.0, 12.0),
    (12.0, 20.0),
    (20.0, 34.0),
];
const SKY_R: f32 = 160.0;
const SKY_SEGS: usize = 32;
/// 달: 거리·크기(m)·높이각(달빛이 오는 쪽 — 높이는 낮춘다: 빛 방향 45°는 어느 카메라에도 화면 밖이다).
/// 산등성이(가장 먼 겹 100m, 꼭대기 ≈ 11°)보다 멀어 그 뒤에 숨으므로, 달 쪽 둘레는 골짜기로 판다(`MOON_VALLEY`).
const MOON_DIST: f32 = 150.0;
const MOON_SIZE: f32 = 24.0;
const MOON_ELEV: f32 = 6.5;
/// 달 쪽 골짜기: 이 각(°) 안에서 산등성이가 낮아진다(가운데에서 높이 × `MOON_VALLEY_K`).
const MOON_VALLEY: f32 = 26.0;
const MOON_VALLEY_K: f32 = 0.25;

/// 달이 뜬 둘레 각(라디안, x = cos·z = sin) — 달빛이 오는 쪽.
fn moon_azimuth() -> f32 {
    let l = crate::render::LIGHT_DIR;
    l.z.atan2(l.x)
}
/// 산등성이 세 겹(스테이지 가운데 둘레): 반지름 = 스테이지 반폭 × 배, 땅 위 높이 범위(m). 빛은 `Env::ridges`.
struct Ridge {
    radius_k: f32,
    low: f32,
    high: f32,
    seed: u32,
}
const RIDGES: [Ridge; 3] = [
    Ridge {
        radius_k: 2.7,
        low: 11.0,
        high: 24.0,
        seed: 3,
    },
    Ridge {
        radius_k: 1.95,
        low: 7.0,
        high: 17.0,
        seed: 5,
    },
    Ridge {
        radius_k: 1.38,
        low: 3.0,
        high: 9.0,
        seed: 7,
    },
];
const RIDGE_SEGS: usize = 120;
const RIDGE_RIM: f32 = 0.7;
/// 뿌리 아치(가운데 겹): (가운데 쪽°, 벌어진 폭°, 높이 m, 굵기 m).
/// 높이는 숄더뷰 화면 위 끝(높이각 ≈ 12°, 72m에서 ≈ 20m) 안에 아치 꼭대기가 들게.
const ARCHES: [(f32, f32, f32, f32); 4] = [
    (-100.0, 24.0, 18.0, 4.8),
    (-66.0, 14.0, 12.0, 3.6),
    (40.0, 26.0, 20.0, 5.4),
    (160.0, 18.0, 15.0, 4.2),
];

/// 고리 둘레 높이: 주기 `n`칸의 값 잡음(이웃 칸 부드럽게 잇기) 두 겹 — 0..1.
fn ridge_noise(seed: u32, a: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    let u = a.rem_euclid(tau) / tau;
    let oct = |n: u32, k: u32| {
        let x = u * n as f32;
        let i = x.floor() as u32 % n;
        let f = x - x.floor();
        let f = f * f * (3.0 - 2.0 * f);
        let a = h01(seed * 97 + k, i);
        let b = h01(seed * 97 + k, (i + 1) % n);
        a + (b - a) * f
    };
    (0.65 * oct(14, 1) + 0.35 * oct(41, 2)).clamp(0.0, 1.0)
}

/// 달(카메라를 보는 판 위의 다각형 — 텍스처 없이): 바깥부터 빛무리 세 단(옅게), 원판, 얼룩 둘. 지름 = `size`의 40%.
/// (처음엔 32칸 텍스처 판이었는데 이 자리(하늘 원통 바로 앞)에서 그려지지 않았다 — 같은 판을 텍스처 없이 그리면 보였다. 까닭은 못 찾음.)
fn moon_mesh(
    verts: &mut Vec<Vertex>,
    idx: &mut Vec<u16>,
    c: Vec3,
    right: Vec3,
    up: Vec3,
    size: f32,
) {
    let n = 16;
    let tau = std::f32::consts::TAU;
    let at = |o: Vec3, r: f32, k: usize| {
        let a = k as f32 / n as f32 * tau;
        o + (right * a.cos() + up * a.sin()) * r
    };
    // 고리(r0..r1)를 띠로.
    let mut ring = |o: Vec3, r0: f32, r1: f32, col: Color| {
        for k in 0..n {
            quad(
                verts,
                idx,
                [
                    at(o, r0, k),
                    at(o, r1, k),
                    at(o, r1, k + 1),
                    at(o, r0, k + 1),
                ],
                col,
            );
        }
    };
    let h = size * 0.5;
    ring(c, h * 0.75, h, Color::new(0.55, 0.50, 0.80, 0.07));
    ring(c, h * 0.55, h * 0.75, Color::new(0.60, 0.55, 0.85, 0.16));
    ring(c, h * 0.4, h * 0.55, Color::new(0.70, 0.66, 0.90, 0.35));
    ring(c, 0.0, h * 0.4, Color::new(0.95, 0.93, 1.0, 1.0));
    // 얼룩(원판 위, 카메라 쪽으로 조금 — 150m에서는 깊이 한 칸이 몇 m라 같은 칸이면 그리는 차례로 덮는다).
    let toward = right.cross(up);
    for (dx, dy, r) in [(-0.16, -0.09, 0.16), (0.19, 0.19, 0.11)] {
        let o = c + (right * dx + up * dy) * h + toward * 2.0;
        ring(o, 0.0, h * r, Color::new(0.80, 0.80, 0.92, 1.0));
    }
}

fn quad(verts: &mut Vec<Vertex>, idx: &mut Vec<u16>, q: [Vec3; 4], col: Color) {
    let base = verts.len() as u16;
    for p in q {
        verts.push(Vertex::new(p.x, p.y, p.z, 0.0, 0.0, col));
    }
    idx.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

/// 뿌리 아치 하나: 원 둘레(반지름 `rad`)를 따라 펼친 면에서 사인 곡선, 굵기는 곡선에 수직으로(가파른 다리가 실처럼 가늘어지지 않게),
/// 밑동이 굵고 꼭대기가 가늘다. 두 다리는 `bottom`까지 내린다. 위쪽 가장자리에 달빛 테.
fn arch_quads(
    verts: &mut Vec<Vertex>,
    idx: &mut Vec<u16>,
    (c, span, h, w): (f32, f32, f32, f32),
    rad: f32,
    bottom: f32,
    body: Color,
    rim: Color,
) {
    let pi = std::f32::consts::PI;
    let len = rad * span.to_radians();
    let n = 24;
    // 곡선 위 점(둘레 거리 x, 높이 y)과 위쪽 법선.
    let at = |s: f32| -> (Vec2, Vec2, f32) {
        let p = vec2((s - 0.5) * len, (pi * s).sin() * h);
        let t = vec2(len, pi * h * (pi * s).cos()).normalize();
        let nrm = vec2(-t.y, t.x);
        let th = w * (1.0 - 0.45 * (pi * s).sin());
        (p, nrm, th)
    };
    let to3 = |q: Vec2| {
        let a = c.to_radians() + q.x / rad;
        vec3(a.cos() * rad, q.y, a.sin() * rad)
    };
    let edge = |s: f32| {
        let (p, nrm, th) = at(s);
        let mut lo = p - nrm * th * 0.5;
        if s <= 0.0 || s >= 1.0 {
            lo.y = bottom;
        }
        let hi = p + nrm * th * 0.5;
        let rim_lo = p + nrm * (th * 0.5 - 0.6);
        (to3(lo), to3(rim_lo), to3(hi))
    };
    let mut prev = edge(0.0);
    for j in 1..=n {
        let cur = edge(j as f32 / n as f32);
        quad(verts, idx, [prev.0, cur.0, cur.1, prev.1], body);
        quad(verts, idx, [prev.1, cur.1, cur.2, prev.2], rim);
        prev = cur;
    }
}

/// 산등성이 한 겹의 꼭대기 높이(땅 위 m) — `a`는 둘레 각(라디안).
fn ridge_top(r: &Ridge, a: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    let d = ((a - moon_azimuth()).rem_euclid(tau) - tau * 0.5).abs();
    let off = (tau * 0.5 - d).to_degrees(); // 달 쪽에서 몇 도
    let t = (1.0 - off / MOON_VALLEY).clamp(0.0, 1.0);
    let valley = 1.0 - (1.0 - MOON_VALLEY_K) * t * t * (3.0 - 2.0 * t);
    (r.low + (r.high - r.low) * ridge_noise(r.seed, a)) * valley
}

impl Backdrop {
    /// 원경(E2). 땅·벽보다 먼저 그린다. `eye`는 카메라 자리, `right`·`up`은 화면 오른쪽·위, `half`는 스테이지 반폭(가운데 = 원점).
    pub fn sky(
        &mut self,
        batch: &mut Batch,
        eye: Vec3,
        right: Vec3,
        up: Vec3,
        half: f32,
        base_y: f32,
    ) {
        let tau = std::f32::consts::TAU;
        // 하늘 띠.
        let mut verts = Vec::new();
        let mut idx = Vec::new();
        let env = self.env;
        for (b, &(e0, e1)) in SKY_BANDS.iter().enumerate() {
            let col = env.sky[b];
            let (y0, y1) = (SKY_R * e0.to_radians().tan(), SKY_R * e1.to_radians().tan());
            for i in 0..SKY_SEGS {
                let (a0, a1) = (
                    i as f32 / SKY_SEGS as f32 * tau,
                    (i + 1) as f32 / SKY_SEGS as f32 * tau,
                );
                let p = |a: f32, y: f32| eye + vec3(a.cos() * SKY_R, y, a.sin() * SKY_R);
                quad(
                    &mut verts,
                    &mut idx,
                    [p(a0, y0), p(a1, y0), p(a1, y1), p(a0, y1)],
                    col,
                );
            }
        }
        batch.raw(&verts, &idx, None, 1.0);
        batch.send();
        // 달(달빛이 오는 쪽) — 땅속엔 없다.
        if !env.moon {
            self.ridges(batch, half, base_y);
            return;
        }
        let az = moon_azimuth();
        let flat = vec2(az.cos(), az.sin());
        let e = MOON_ELEV.to_radians();
        let dir = vec3(flat.x * e.cos(), e.sin(), flat.y * e.cos());
        let mut mv = Vec::new();
        let mut mi = Vec::new();
        moon_mesh(
            &mut mv,
            &mut mi,
            eye + dir * MOON_DIST,
            right,
            up,
            MOON_SIZE,
        );
        batch.raw(&mv, &mi, None, 1.0);
        batch.send();
        self.ridges(batch, half, base_y);
    }

    /// 산등성이 세 겹(먼 것부터) + 뿌리 아치(가운데 겹 앞). 빛·높이는 지역 환경에서.
    fn ridges(&mut self, batch: &mut Batch, half: f32, base_y: f32) {
        let tau = std::f32::consts::TAU;
        let env = self.env;
        let bottom = base_y - 6.0;
        for (k, r) in RIDGES.iter().enumerate() {
            let mut verts = Vec::new();
            let mut idx = Vec::new();
            let rad = half * r.radius_k;
            for i in 0..RIDGE_SEGS {
                let (a0, a1) = (
                    i as f32 / RIDGE_SEGS as f32 * tau,
                    (i + 1) as f32 / RIDGE_SEGS as f32 * tau,
                );
                let at = |a: f32, y: f32| vec3(a.cos() * rad, y, a.sin() * rad);
                let (t0, t1) = (
                    ridge_top(r, a0) * env.ridge_h,
                    ridge_top(r, a1) * env.ridge_h,
                );
                quad(
                    &mut verts,
                    &mut idx,
                    [
                        at(a0, bottom),
                        at(a1, bottom),
                        at(a1, t1 - RIDGE_RIM),
                        at(a0, t0 - RIDGE_RIM),
                    ],
                    env.ridges[k].0,
                );
                quad(
                    &mut verts,
                    &mut idx,
                    [
                        at(a0, t0 - RIDGE_RIM),
                        at(a1, t1 - RIDGE_RIM),
                        at(a1, t1),
                        at(a0, t0),
                    ],
                    env.ridges[k].1,
                );
            }
            if k == 1 {
                for &arch in &ARCHES {
                    arch_quads(
                        &mut verts,
                        &mut idx,
                        arch,
                        rad * 0.97,
                        bottom,
                        env.ridges[k].0,
                        env.ridges[k].1,
                    );
                }
            }
            batch.raw(&verts, &idx, None, 1.0);
            batch.send();
        }
    }
}

/// 잔불 조각(0064 M1): 도는 작은 팔면체(위는 밝은 호박, 아래는 짙게) + 빛무리 판. 스스로 빛나는 것이라 빛을 받지 않는다.
const EMBER_SIZE: f32 = 0.13;
const EMBER_HALO: f32 = 0.55;
const EMBER_TOP: Color = Color::new(1.0, 0.78, 0.32, 1.0);
const EMBER_BOTTOM: Color = Color::new(0.85, 0.42, 0.14, 1.0);
const EMBER_GLOW: Color = Color::new(1.0, 0.7, 0.3, 0.9);

impl Backdrop {
    /// 떨어진 잔불 조각들(월드 `drops`).
    pub fn embers(
        &mut self,
        batch: &mut Batch,
        drops: &[crate::ember::Ember],
        right: Vec3,
        up: Vec3,
        time: f32,
    ) {
        if drops.is_empty() {
            return;
        }
        let mut verts = Vec::new();
        let mut idx = Vec::new();
        let mut halos = Vec::new();
        let mut halo_idx = Vec::new();
        for (k, e) in drops.iter().enumerate() {
            let spin = time * 3.0 + k as f32 * 1.7;
            let (sx, sz) = (spin.cos() * EMBER_SIZE, spin.sin() * EMBER_SIZE);
            let ring = [
                e.pos + vec3(sx, 0.0, sz),
                e.pos + vec3(-sz, 0.0, sx),
                e.pos + vec3(-sx, 0.0, -sz),
                e.pos + vec3(sz, 0.0, -sx),
            ];
            let (top, bottom) = (
                e.pos + Vec3::Y * EMBER_SIZE * 1.4,
                e.pos - Vec3::Y * EMBER_SIZE * 1.4,
            );
            for i in 0..4 {
                let (a, b) = (ring[i], ring[(i + 1) % 4]);
                let base = verts.len() as u16;
                for (p, c) in [
                    (top, EMBER_TOP),
                    (a, EMBER_TOP),
                    (b, EMBER_TOP),
                    (bottom, EMBER_BOTTOM),
                    (b, EMBER_BOTTOM),
                    (a, EMBER_BOTTOM),
                ] {
                    verts.push(Vertex::new(p.x, p.y, p.z, 0.0, 0.0, c));
                }
                idx.extend_from_slice(&[base, base + 1, base + 2, base + 3, base + 4, base + 5]);
            }
            billboard(
                &mut halos,
                &mut halo_idx,
                e.pos,
                right,
                up,
                EMBER_HALO,
                EMBER_GLOW,
            );
        }
        let tex = self
            .halo_tex
            .get_or_insert_with(|| {
                let t = Texture2D::from_rgba8(8, 8, &halo_pixels(8));
                t.set_filter(FilterMode::Nearest);
                t
            })
            .clone();
        batch.raw(&halos, &halo_idx, Some(&tex), 1.0);
        batch.send();
        batch.raw(&verts, &idx, None, 1.0);
        batch.send();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn height_fog_stays_below_the_lowest_walkable_ground_and_above_the_void() {
        // 걷는 땅 0, 허공 −2.5: 판은 −0.3부터 0.5m마다, 허공 판 위에서 멈춘다.
        let h = fog_heights(0.0, -2.5);
        assert_eq!(h.len(), 5);
        assert!((h[0] + FOG_CLEAR).abs() < 1e-6);
        assert!(h.iter().all(|&y| y < 0.0 && y > -2.5));
        assert!(h.windows(2).all(|w| w[0] > w[1]));
        // 꺼진 광장(−1m)을 걸을 수 있으면 그 아래부터.
        assert!(fog_heights(-1.0, -2.5)
            .iter()
            .all(|&y| y <= -1.0 - FOG_CLEAR + 1e-6));
        // 걷는 땅 바로 아래가 허공 판이면 없다.
        assert!(fog_heights(0.0, -0.03).is_empty());
    }

    #[test]
    fn the_fog_pattern_tiles_without_a_seam() {
        let n = FOG_TEX as usize;
        let px = fog_pixels(FOG_TEX);
        let a = |x: usize, y: usize| px[((y % n) * n + (x % n)) * 4 + 3] as i32;
        // 오른쪽 끝 → 왼쪽 처음, 아래 끝 → 위 처음이 이웃끼리처럼 이어진다(세 단 계단이라 한 단까지만 차이).
        let jump = (0..n).map(|y| (a(n - 1, y) - a(0, y)).abs()).max().unwrap();
        let inner = (0..n)
            .map(|y| (a(n / 2, y) - a(n / 2 + 1, y)).abs())
            .max()
            .unwrap();
        assert!(jump <= inner.max(120), "seam {jump} vs inner {inner}");
        // 세 단만.
        let mut levels: Vec<i32> = (0..n * n).map(|i| px[i * 4 + 3] as i32).collect();
        levels.sort();
        levels.dedup();
        assert!(levels.len() <= 3);
    }

    #[test]
    fn drifting_specks_stay_in_a_box_around_the_player_and_fade_at_its_edges() {
        for d in [&MOTES, &FIREFLIES, &FALLING] {
            for c in [Vec3::ZERO, vec3(37.0, 0.0, -12.5)] {
                for t in [0.0, 13.7, 600.0] {
                    for i in 0..d.count {
                        let (p, k) = drift_at(d, i, c, t, None);
                        assert!(
                            (p.x - c.x).abs() <= d.half + 1e-3
                                && (p.z - c.z).abs() <= d.half + 1e-3
                        );
                        assert!(p.y >= d.y.0 - 1e-3 && p.y <= d.y.1 + 1e-3);
                        assert!((0.0..=1.0).contains(&k));
                    }
                }
            }
        }
        // 감아 도는 자리(상자 끝)에서는 보이지 않는다 — 튀어 나타나지 않게.
        let edge_k: f32 = (0..2000)
            .map(|n| drift_at(&MOTES, 3, Vec3::ZERO, n as f32 * 0.37, None))
            .filter(|(p, _)| p.x.abs() > MOTES.half * 0.99)
            .map(|(_, k)| k)
            .fold(0.0, f32::max);
        assert!(edge_k < 0.1, "{edge_k}");
    }

    #[test]
    fn falling_dust_tracks_each_walkable_surface_and_stays_below_its_ceiling() {
        for floor in [-1.0, 0.0, 2.5, 4.0] {
            let ground = Ground::new(1, 1, 100.0, vec![crate::ground::Cell::Flat(floor)]);
            for t in [0.0, 13.7, 600.0] {
                for i in 0..FALLING.count {
                    let (p, k) = drift_at(&FALLING, i, Vec3::ZERO, t, Some(&ground));
                    assert!(p.y >= floor + 0.1 - 1e-4, "floor={floor}, y={}", p.y);
                    assert!(p.y <= floor + 6.0 + 1e-4, "floor={floor}, y={}", p.y);
                    assert!((0.0..=1.0).contains(&k));
                }
            }
        }
    }

    /// 달빛 먼지·반딧불도 바닥을 따른다(높은 단에서 바닥에 묻히지 않게) — 바닥 없는 칸에서도 보인다(허공 위 먼지).
    #[test]
    fn every_drift_rises_from_the_local_floor() {
        let high = Ground::new(1, 1, 100.0, vec![crate::ground::Cell::Flat(4.0)]);
        let solid = Ground::new(1, 1, 100.0, vec![crate::ground::Cell::Solid]);
        for d in [&MOTES, &FIREFLIES] {
            for i in 0..d.count {
                let (p, _) = drift_at(d, i, Vec3::ZERO, 7.3, Some(&high));
                assert!(p.y >= 4.0 + d.y.0 - 1e-4, "seed {}: {}", d.seed, p.y);
            }
            let shown = (0..d.count).any(|i| drift_at(d, i, Vec3::ZERO, 7.3, Some(&solid)).1 > 0.0);
            assert!(shown, "seed {}: not hidden over solid cells", d.seed);
        }
    }

    #[test]
    fn falling_dust_is_hidden_over_solid_cells() {
        let ground = Ground::new(1, 1, 100.0, vec![crate::ground::Cell::Solid]);
        let (_, alpha) = drift_at(&FALLING, 0, Vec3::ZERO, 0.0, Some(&ground));
        assert_eq!(alpha, 0.0);
    }

    #[test]
    fn fireflies_blink_and_are_not_all_lit_at_once() {
        let lit = |t: f32| {
            (0..FIREFLIES.count)
                .filter(|&i| drift_at(&FIREFLIES, i, vec3(0.0, 0.0, 0.0), t, None).1 > 0.5)
                .count()
        };
        let counts: Vec<usize> = (0..40).map(|n| lit(n as f32 * 0.5)).collect();
        assert!(
            counts.iter().all(|&n| n < FIREFLIES.count as usize),
            "{counts:?}"
        );
        assert!(counts.iter().any(|&n| n > 0), "{counts:?}");
    }

    #[test]
    fn the_backdrop_stays_inside_the_far_clip_from_anywhere_on_the_stage() {
        // 먼 클립 200m(camera.rs). 스테이지 끝(반폭 37m) + 카메라 거리 ~12m에서도 가장 먼 산등성이·하늘 원통 위 끝·달이 안에.
        let half = 37.0;
        let cam_off = half * 2f32.sqrt() + 12.0;
        let far = RIDGES.iter().map(|r| half * r.radius_k).fold(0.0, f32::max) + cam_off;
        assert!(far < 195.0, "{far}");
        let top = SKY_BANDS
            .iter()
            .map(|b| b.1)
            .fold(-90.0, f32::max)
            .to_radians();
        assert!(SKY_R / top.cos() < 198.0);
        assert!(MOON_DIST + MOON_SIZE < SKY_R + 20.0 && MOON_DIST < SKY_R);
        // 산등성이는 겹마다 멀수록 높다(뒤 겹이 앞 겹 위로 보인다), 하늘 띠는 빈틈 없이 이어진다.
        assert!(RIDGES
            .windows(2)
            .all(|w| w[0].radius_k > w[1].radius_k && w[0].high > w[1].high));
        assert!(SKY_BANDS.windows(2).all(|w| (w[0].1 - w[1].0).abs() < 1e-6));
        // 가장 가까운 겹도 스테이지 모서리 밖.
        let corner = (25.0f32.powi(2) + 37.0f32.powi(2)).sqrt();
        assert!(half * RIDGES[2].radius_k > corner);
    }

    #[test]
    fn the_moon_rises_in_a_valley_above_every_ridge() {
        // 달 아래 끝(높이각 − 반지름)이 그 쪽 산등성이 꼭대기들보다 높다(카메라 높이 4.5m, 스테이지 가운데에서).
        let az = moon_azimuth();
        let moon_low = MOON_ELEV - (MOON_SIZE * 0.2 / MOON_DIST).atan().to_degrees();
        for r in &RIDGES {
            let rad = 37.0 * r.radius_k;
            let top = ((ridge_top(r, az) - 4.5) / rad).atan().to_degrees();
            assert!(
                top < moon_low,
                "ridge {} top {top}° vs moon {moon_low}°",
                r.seed
            );
        }
        // 골짜기 밖은 원래 높이.
        let far = &RIDGES[0];
        let a = az + std::f32::consts::PI;
        assert!(
            (ridge_top(far, a) - (far.low + (far.high - far.low) * ridge_noise(far.seed, a))).abs()
                < 1e-4
        );
    }
}
