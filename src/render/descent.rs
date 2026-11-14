//! 지역 사이 전환의 굴 장면(0174): 뿌리가 얽힌 수직 굴 — 벽을 따라 도는 큰 뿌리를 방랑자가 밟고 내려간다(올라간다).
//! 규칙(시간표·길·카메라)은 `descent.rs`. 여기는 그림: 굴 벽(깊이마다의 흙빛 지층 — 둥글게 부푼 띠)·뿌리들(휘고 가늘어지는 둥근 관,
//! 매끈한 법선 — 0177 다시 빚음, 사장 "모델링이 너무 허접": 면마다 빛을 받던 조각 벽·네모 막대 뿌리를 걷었다)·
//! 방랑자(걷기 동작)·등잔 빛·위의 달빛과 아래의 빛·떠다니는 먼지. 월드와 같은 타깃·같은 선·같은 마무리 패스로 그린다.
//! 굴의 면은 전환마다 한 번 짓고(깊이가 흙빛을 정한다), 빛은 프레임마다 다시 받는다 — 등잔이 방랑자와 같이 움직인다.
use super::*;
use crate::descent::{self as dsc, Passage, DROP, ROOT_R, SHAFT_R};

const TAU_F: f32 = std::f32::consts::TAU;

/// 정점 하나(굴 안의 자리). 법선은 정점마다 — 둥근 덩어리는 매끈하게(KB `3d-pixel-art-look` 0139: 면마다 따로 빛을 받으면 툰 띠
/// 대신 깨진 유리 같은 조각이 되고 면마다 밝은 선이 선다). `glow` = 스스로 빛남(빛을 받지 않는다).
#[derive(Clone, Copy)]
struct SV {
    p: Vec3,
    n: Vec3,
    col: Color,
    glow: bool,
}

/// 덩어리 하나(정점을 같이 쓰는 면 묶음 — 한 그리기 호출의 한도 안).
#[derive(Default)]
struct Part {
    v: Vec<SV>,
    idx: Vec<u16>,
}

/// 지어 둔 굴(전환 하나의 것).
pub(super) struct ShaftMesh {
    key: (u32, u32),
    parts: Vec<Part>,
}

/// 굴 벽이 이어지는 높이(위·아래 — 카메라가 보는 끝보다 넉넉히).
const WALL_TOP: f32 = 7.0;
const WALL_BOTTOM: f32 = -DROP - 8.0;
/// 벽 한 바퀴의 칸 수.
const WALL_COLS: usize = 56;
/// 뿌리의 빛(식생의 뿌리와 같은 보라 — `flora::ROOT`), 스스로 빛나는 옹이(호박).
const ROOT: Color = Color::new(0.27, 0.15, 0.34, 1.0);
const KNOT: Color = Color::new(1.0, 0.72, 0.36, 1.0);
/// 이끼(큰 뿌리 옆구리), 썩은 뿌리에 핀 희미한 버섯(갓이 스스로 빛남 — 푸른 빛, 뿌리의 호박빛 옹이와 갈린다 · 갓 밑은 짙게),
/// 벽을 기는 검은 균사 줄(뽕나무버섯의 "신발끈" 균사다발 — KB 동물·생태 사실).
const MOSS: Color = Color::new(0.22, 0.36, 0.29, 1.0);
const CAP: Color = Color::new(0.55, 0.95, 0.82, 1.0);
const GILL: Color = Color::new(0.24, 0.52, 0.5, 1.0);
const STALK: Color = Color::new(0.62, 0.58, 0.52, 1.0);
const CORD: Color = Color::new(0.07, 0.05, 0.08, 1.0);
/// 등잔 빛·달빛(굴 위의 틈)의 색.
const LANTERN: Color = Color::new(1.0, 0.70, 0.36, 1.0);
const MOONBEAM: Color = Color::new(0.52, 0.58, 0.92, 1.0);
/// 굴 속 바탕 어둠(달빛이 닿지 않는다 — `lit`의 바탕빛에 곱한다).
const CAVE_DIM: f32 = 0.8;

fn h(a: u32, b: u32) -> f32 {
    let mut v = a.wrapping_mul(0x9E37_79B9) ^ b.wrapping_mul(0x85EB_CA6B);
    v ^= v >> 15;
    v = v.wrapping_mul(0x2C1B_3C6D);
    v ^= v >> 12;
    (v & 0xFFFF) as f32 / 65535.0
}

fn mul(c: Color, k: f32) -> Color {
    Color::new(c.r * k, c.g * k, c.b * k, c.a)
}

/// 격자 면(행 × 열, `closed`면 열이 한 바퀴 돈다): 이웃 정점의 차로 매끈한 법선을 내고, `facing(자리, 행)`과 같은 쪽을 보게 한다.
fn grid(
    pos: &[Vec<Vec3>],
    closed: bool,
    col: impl Fn(usize, usize) -> Color,
    glow: bool,
    facing: impl Fn(Vec3, usize) -> Vec3,
) -> Part {
    let rows = pos.len();
    let cols = pos[0].len();
    let mut part = Part::default();
    for i in 0..rows {
        for j in 0..cols {
            let (jm, jp) = if closed {
                ((j + cols - 1) % cols, (j + 1) % cols)
            } else {
                (j.saturating_sub(1), (j + 1).min(cols - 1))
            };
            let (im, ip) = (i.saturating_sub(1), (i + 1).min(rows - 1));
            let p = pos[i][j];
            let want = facing(p, i);
            let mut n = (pos[i][jp] - pos[i][jm])
                .cross(pos[ip][j] - pos[im][j])
                .normalize_or_zero();
            if n == Vec3::ZERO {
                n = want.normalize_or(Vec3::Y);
            }
            if n.dot(want) < 0.0 {
                n = -n;
            }
            part.v.push(SV {
                p,
                n,
                col: col(i, j),
                glow,
            });
        }
    }
    let span = if closed { cols } else { cols - 1 };
    for i in 0..rows - 1 {
        for j in 0..span {
            let j1 = (j + 1) % cols;
            let (a, b, c, d) = (
                i * cols + j,
                i * cols + j1,
                (i + 1) * cols + j,
                (i + 1) * cols + j1,
            );
            part.idx.extend([a, b, c, b, d, c].map(|x| x as u16));
        }
    }
    part
}

/// 캣멀-롬 곡선: 조절점 사이마다 `per`점(끝점 포함) — 꺾인 줄이 아니라 휘는 줄(뿌리가 막대로 읽히지 않게).
fn spline(c: &[Vec3], per: usize) -> Vec<Vec3> {
    let n = c.len();
    let mut out = Vec::with_capacity(n * per + 1);
    for i in 0..n - 1 {
        let (p0, p1, p2, p3) = (
            c[i.saturating_sub(1)],
            c[i],
            c[i + 1],
            c[(i + 2).min(n - 1)],
        );
        for k in 0..per {
            let t = k as f32 / per as f32;
            let (t2, t3) = (t * t, t * t * t);
            out.push(
                0.5 * (2.0 * p1
                    + (p2 - p0) * t
                    + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
                    + (3.0 * p1 - p0 - 3.0 * p2 + p3) * t3),
            );
        }
    }
    out.push(c[n - 1]);
    out
}

/// 관(뿌리): 가운데 줄 `pts`를 따라 `sides`모 단면, 반지름 `r(t)`(t = 0..1, 0이면 뾰족). 단면은 평행 이동 틀로 옮겨 꼬이지 않고,
/// 모마다 반지름을 `groove`만큼 번갈아(나무껍질의 골 — 법선이 매끈해 줄무늬가 아니라 결로 읽힌다) 길이를 따라 `twist`(라디안)만큼 돈다.
fn tube(
    out: &mut Vec<Part>,
    pts: &[Vec3],
    sides: usize,
    r: impl Fn(f32) -> f32,
    groove: f32,
    twist: f32,
    col: Color,
) {
    let n = pts.len();
    if n < 2 {
        return;
    }
    let dir = |i: usize| -> Vec3 {
        (pts[(i + 1).min(n - 1)] - pts[i.saturating_sub(1)]).normalize_or(Vec3::Y)
    };
    // 첫 틀: 위쪽에 가까운 수직(큰 뿌리의 골이 위·아래로 고르게).
    let t0 = dir(0);
    let mut nrm = (Vec3::Y - t0 * Vec3::Y.dot(t0)).normalize_or(Vec3::X);
    let mut rings: Vec<Vec<Vec3>> = Vec::with_capacity(n);
    for (i, &c) in pts.iter().enumerate() {
        let t = dir(i);
        nrm = (nrm - t * nrm.dot(t)).normalize_or(nrm);
        let bin = t.cross(nrm);
        let u = i as f32 / (n - 1) as f32;
        let rr = r(u).max(0.004);
        rings.push(
            (0..sides)
                .map(|k| {
                    let a = TAU_F * k as f32 / sides as f32 + twist * u;
                    let g = if k % 2 == 0 {
                        1.0 + groove
                    } else {
                        1.0 - groove
                    };
                    c + (nrm * a.cos() + bin * a.sin()) * rr * g
                })
                .collect(),
        );
    }
    out.push(grid(&rings, true, |_, _| col, false, |p, i| p - pts[i]));
}

/// 둥근 덩어리(돌·이끼·버섯 갓·옹이): 이십면체를 한 번 쪼갠 공(정점 42 · 면 80)을 축마다 `r`로, 씨앗으로 ±`wob` 흔든다. 법선은 매끈하게.
/// 색은 정점의 단위 방향(`u`)으로 — 위·아래를 다른 색으로(갓과 갓 밑).
fn blob(
    out: &mut Vec<Part>,
    c: Vec3,
    r: Vec3,
    seed: u32,
    wob: f32,
    col: impl Fn(Vec3) -> Color,
    glow: bool,
) {
    let (v12, f20) = crate::flora::icosahedron();
    let mut unit: Vec<Vec3> = v12.to_vec();
    let mut faces: Vec<[usize; 3]> = Vec::with_capacity(80);
    let mut mid = std::collections::HashMap::new();
    let mut half = |a: usize, b: usize, unit: &mut Vec<Vec3>| -> usize {
        *mid.entry((a.min(b), a.max(b))).or_insert_with(|| {
            unit.push(((unit[a] + unit[b]) * 0.5).normalize());
            unit.len() - 1
        })
    };
    for [a, b, d] in f20 {
        let (ab, bd, da) = (
            half(a, b, &mut unit),
            half(b, d, &mut unit),
            half(d, a, &mut unit),
        );
        faces.extend([[a, ab, da], [ab, b, bd], [da, bd, d], [ab, bd, da]]);
    }
    let mut part = Part::default();
    for (k, u) in unit.iter().enumerate() {
        let w = 1.0 + wob * (h(seed, k as u32) - 0.5) * 2.0;
        part.v.push(SV {
            p: c + *u * r * w,
            // 타원체의 법선(축 반지름으로 나눈 방향).
            n: (*u / r).normalize_or(*u),
            col: col(*u),
            glow,
        });
    }
    for f in faces {
        part.idx.extend(f.map(|x| x as u16));
    }
    out.push(part);
}

/// 굴 벽 위의 한 점(각 `a`, 높이 `y`) — 벽에서 `inset`만큼 안쪽(벽의 기복 `wall_r`을 따른다).
fn on_wall(a: f32, y: f32, inset: f32) -> Vec3 {
    let (k, top, bottom) = bands()
        .into_iter()
        .find(|b| y <= b.1 && y >= b.2)
        .unwrap_or((0, y, y - 1.0));
    let r = wall_r(a, y, (top - y) / (top - bottom), k) - inset;
    vec3(r * a.sin(), y, r * a.cos())
}

/// 지층 띠: 위에서부터 높이 1.3~2.3m(씨앗으로) — (띠 번호, 위, 아래).
fn bands() -> Vec<(u32, f32, f32)> {
    let mut v = Vec::new();
    let mut top = WALL_TOP;
    let mut k = 0u32;
    while top > WALL_BOTTOM {
        let bottom = (top - (1.3 + 1.0 * h(k, 1))).max(WALL_BOTTOM - 0.01);
        v.push((k, top, bottom));
        top = bottom;
        k += 1;
    }
    v
}

/// 벽의 반지름: 크게 굽이치는 기복(세 사인 — 큰 얼룩, 0140 "변주는 큰 덩어리로") − 띠마다 안으로 둥글게 부푼 지층(`u` = 띠 안의
/// 자리 0..1, 띠의 이음에서 0이라 이웃 띠와 꼭 맞는다 — 이음은 움푹한 금이 된다).
fn wall_r(a: f32, y: f32, u: f32, band: u32) -> f32 {
    let low = 0.2 * (3.0 * a + 0.45 * y + 1.3).sin()
        + 0.13 * (5.0 * a - 0.8 * y + 0.4).sin()
        + 0.06 * (9.0 * a + 1.7 * y).sin();
    let swell = 0.12 + 0.14 * (0.5 + 0.5 * (2.0 * a + band as f32 * 1.9).sin());
    let lip = (std::f32::consts::PI * u.clamp(0.0, 1.0))
        .sin()
        .max(0.0)
        .powf(0.6)
        * swell;
    SHAFT_R + low - lip
}

/// 굴 하나를 짓는다(0174 → 0177 다시 빚음 — 사장 "모델링이 너무 허접"): 둥근 지층 벽 · 박힌 바위 · 둥근 큰 뿌리(골이 감기는 껍질) ·
/// 받치는 뿌리 · 벽을 타는 뿌리 · 늘어진 잔뿌리 · 곁뿌리 · 이끼 · 버섯 · 균사 줄 · 자갈 · 빛나는 옹이. 모두 매끈한 법선의 휘는 꼴.
fn build(from_m: u32, to_m: u32) -> ShaftMesh {
    let mut f: Vec<Part> = Vec::new();
    // 위 끝 = 얕은 쪽, 아래 끝 = 깊은 쪽(올라가는 전환도 같은 굴을 거꾸로 걷는다).
    let (top_m, bottom_m) = (from_m.min(to_m) as f32, from_m.max(to_m) as f32);
    let depth_at = |y: f32| top_m + (bottom_m - top_m) * (-y / DROP).clamp(-0.6, 1.6);
    // 벽: 지층 띠마다 덩어리 하나(띠마다 한 흙빛 — 번갈아 밝고 어둡게, 부푼 가운데는 밝고 이음의 금은 어둡게).
    for (k, top, bottom) in bands() {
        const ROWS: usize = 7;
        let shade = if k % 2 == 0 { 1.0 } else { 0.86 } + 0.05 * h(k, 2);
        let base = mul(dsc::soil(depth_at((top + bottom) * 0.5)), shade);
        let pos: Vec<Vec<Vec3>> = (0..ROWS)
            .map(|i| {
                let u = i as f32 / (ROWS - 1) as f32;
                let y = top - (top - bottom) * u;
                (0..WALL_COLS)
                    .map(|j| {
                        let a = TAU_F * j as f32 / WALL_COLS as f32;
                        let r = wall_r(a, y, u, k);
                        vec3(r * a.sin(), y, r * a.cos())
                    })
                    .collect()
            })
            .collect();
        f.push(grid(
            &pos,
            true,
            |i, _| {
                let u = i as f32 / (ROWS - 1) as f32;
                mul(base, 0.78 + 0.22 * (std::f32::consts::PI * u).sin())
            },
            false,
            |p, _| vec3(-p.x, 0.0, -p.z),
        ));
    }
    // 박힌 바위: 지층의 이음에 반쯤 묻힌 큰 덩어리(작은 점만 흩으면 가루로 보인다 — 몇 개를 크게).
    let bs = bands();
    for k in 0..14u32 {
        let b = bs[(k as usize * 5 + 1) % bs.len()];
        let a = TAU_F * h(k, 51);
        let y = b.2 + 0.1 * (h(k, 52) - 0.5);
        let r = 0.45 + 0.45 * h(k, 53);
        let stone = mul(dsc::soil(depth_at(y) + 22.0), 1.3);
        blob(
            &mut f,
            on_wall(a, y, 0.05),
            vec3(r, r * (0.55 + 0.2 * h(k, 54)), r * 0.9),
            k,
            0.12,
            |_| stone,
            false,
        );
    }
    // 큰 뿌리: 걷는 구간 밖으로도 이어진다(화면 밖에서 오고 간다), 두 끝은 가늘어진다. 둥근 단면 + 감기는 껍질의 골.
    let (s0, s1) = (-0.45, 1.45);
    let n = 96;
    let pts: Vec<Vec3> = (0..=n)
        .map(|i| dsc::path(s0 + (s1 - s0) * i as f32 / n as f32))
        .collect();
    tube(
        &mut f,
        &pts,
        14,
        |u| {
            let s = s0 + (s1 - s0) * u;
            let end = ((s + 0.45) / 0.35).min((1.45 - s) / 0.35).clamp(0.0, 1.0);
            ROOT_R * (0.4 + 0.6 * end)
        },
        0.08,
        5.0,
        ROOT,
    );
    // 받치는 뿌리: 큰 뿌리 밑에서 벽으로 비스듬히 아래 — 두 끝이 굵게 퍼지고(붙은 자리) 가운데가 가늘다.
    for k in 0..8u32 {
        let s = -0.3 + 0.2 * k as f32 + (h(k, 1) - 0.5) * 0.06;
        let from = dsc::path(s) - Vec3::Y * ROOT_R * 0.55;
        let a = dsc::angle(s) + (h(k, 2) - 0.5) * 0.5;
        let to = on_wall(a, from.y - 1.4 - 1.4 * h(k, 3), -0.15);
        let bend = (from + to) * 0.5 - Vec3::Y * (0.2 + 0.2 * h(k, 4));
        let col = mul(ROOT, 0.82 + 0.15 * h(k, 5));
        tube(
            &mut f,
            &spline(&[from, bend, to], 8),
            9,
            |u| 0.32 * (1.0 - 0.45 * (std::f32::consts::PI * u).sin()) * (1.0 + 0.45 * u * u),
            0.09,
            2.0,
            col,
        );
    }
    // 곁뿌리: 큰 뿌리의 옆구리에서 나와 휘어 늘어진다(위로 솟은 가시가 아니다), 끝이 뾰족하다.
    for k in 0..8u32 {
        let s = -0.15 + 1.3 * (k as f32 + h(k, 91)) / 8.0;
        let a = dsc::angle(s);
        let out = vec3(a.sin(), 0.0, a.cos()) * if k % 2 == 0 { 1.0 } else { -1.0 };
        let p0 = dsc::path(s) + out * ROOT_R * 0.7;
        let len = 0.8 + 0.6 * h(k, 93);
        let c = [
            p0,
            p0 + out * len * 0.45 + Vec3::Y * 0.05,
            p0 + out * len * 0.8 - Vec3::Y * len * 0.35,
            p0 + out * len - Vec3::Y * len * 0.9,
        ];
        tube(
            &mut f,
            &spline(&c, 6),
            7,
            |u| 0.16 * (1.0 - u).powf(0.8),
            0.0,
            0.0,
            mul(ROOT, 0.88 + 0.12 * h(k, 94)),
        );
    }
    // 늘어진 잔뿌리: 큰 뿌리 밑에서 아래로 가늘게 굽이치며(곧은 굵은 가시는 고드름으로 읽혔다), 끝은 실처럼.
    for k in 0..18u32 {
        let s = -0.2 + 1.4 * h(k, 11);
        let from = dsc::path(s) - Vec3::Y * ROOT_R * 0.85
            + vec3(h(k, 12) - 0.5, 0.0, h(k, 13) - 0.5) * 0.5;
        let len = 1.0 + 1.4 * h(k, 14);
        let side = |i: u32| vec3(h(k, 15 + i) - 0.5, 0.0, h(k, 18 + i) - 0.5) * 0.35;
        let c = [
            from,
            from - Vec3::Y * len * 0.33 + side(0),
            from - Vec3::Y * len * 0.66 + side(1),
            from - Vec3::Y * len + side(2),
        ];
        tube(
            &mut f,
            &spline(&c, 5),
            5,
            |u| 0.045 * (1.0 - u * 0.9),
            0.0,
            0.0,
            mul(ROOT, 0.8 + 0.3 * h(k, 17)),
        );
    }
    // 벽을 타는 뿌리: 위에서 아래로 굽이치며 벽에 반쯤 묻혀 내려간다, 아래로 가늘어진다. 몇은 중간에 갈래 하나.
    for k in 0..14u32 {
        let a0 = TAU_F * (k as f32 + h(k, 21)) / 14.0;
        let y0 = WALL_TOP - 1.0 - (WALL_TOP - WALL_BOTTOM - 6.0) * h(k, 22);
        let len = 3.5 + 4.5 * h(k, 23);
        let r = 0.2 + 0.18 * h(k, 24);
        let c: Vec<Vec3> = (0..=5)
            .map(|i| {
                let t = i as f32 / 5.0;
                let wig = (h(k, 30 + i) - 0.5) * 0.14;
                // 위 끝은 벽 속에서 나온다(잘린 관 끝이 보이지 않게).
                // 몸은 반 넘게 벽에 묻는다(드러난 앞쪽만 — 벽에서 떨어진 기둥으로 읽히지 않게).
                let inset = if i == 0 { -r * 1.2 } else { r * 0.15 };
                on_wall(a0 + wig, y0 - len * t, inset)
            })
            .collect();
        let line = spline(&c, 5);
        let col = mul(ROOT, 0.62 + 0.2 * h(k, 25));
        tube(
            &mut f,
            &line,
            7,
            |u| r * (1.15 - 0.85 * u) * (u / 0.12).min(1.0).sqrt(),
            0.09,
            2.0,
            col,
        );
        if k % 2 == 0 {
            let at = line[line.len() / 2];
            let a1 = at.x.atan2(at.z) + 0.35 * if k % 4 == 0 { 1.0 } else { -1.0 };
            let c2 = [
                at,
                on_wall(a1 * 0.5 + at.x.atan2(at.z) * 0.5, at.y - 0.5, r * 0.4),
                on_wall(a1, at.y - 1.3, r * 0.3),
            ];
            tube(
                &mut f,
                &spline(&c2, 5),
                6,
                |u| r * 0.6 * (1.0 - u),
                0.0,
                0.0,
                col,
            );
        }
    }
    // 가로지르는 뿌리: 벽에서 벽으로(가까운 두 점 — 굴 한가운데는 비워 둔다, 카메라와 방랑자 사이를 막지 않게), 가운데가 처진다.
    for k in 0..4u32 {
        let a0 = TAU_F * h(k, 41);
        let span = 0.45 + 0.3 * h(k, 42);
        let y = WALL_TOP - 2.0 - (WALL_TOP - WALL_BOTTOM - 4.0) * h(k, 43);
        let (a, b) = (
            on_wall(a0, y, -0.1),
            on_wall(a0 + span, y - 0.8 + 1.6 * h(k, 44), -0.1),
        );
        let mid = (a + b) * 0.5 * 0.99 - Vec3::Y * 0.3;
        tube(
            &mut f,
            &spline(&[a, mid, b], 8),
            8,
            |u| 0.2 * (1.0 - 0.3 * (std::f32::consts::PI * u).sin()),
            0.09,
            1.0,
            mul(ROOT, 0.8),
        );
    }
    // 이끼: 큰 뿌리 옆구리의 낮고 둥근 방석(걷는 줄 곁 — 발밑은 비운다).
    for k in 0..14u32 {
        let s = -0.2 + 1.4 * h(k, 101);
        let a = dsc::angle(s);
        let side = vec3(a.sin(), 0.0, a.cos()) * if k % 2 == 0 { 1.0 } else { -1.0 };
        let c = dsc::path(s) + side * ROOT_R * 0.72 + Vec3::Y * ROOT_R * 0.55;
        let r = 0.2 + 0.14 * h(k, 103);
        let col = mul(MOSS, 0.85 + 0.3 * h(k, 105));
        blob(
            &mut f,
            c,
            vec3(r, r * 0.32, r * 0.8),
            200 + k,
            0.2,
            |_| col,
            false,
        );
    }
    // 버섯 무리: 벽과 큰 뿌리의 옆구리에 셋~다섯씩(대 + 둥근 갓 — 갓 위는 밝게, 갓 밑은 짙게, 스스로 빛난다).
    let cluster = |f: &mut Vec<Part>, k: u32, foot0: Vec3, into: Vec3| {
        let n = 3 + (h(k, 113) * 3.0) as u32;
        let along = Vec3::Y.cross(into).normalize_or_zero();
        for j in 0..n {
            let off = along * (h(k * 13 + j, 114) - 0.5) * 0.5
                + Vec3::Y * (h(k * 13 + j, 115) - 0.5) * 0.3;
            let foot = foot0 + off;
            let tall = 0.14 + 0.14 * h(k * 13 + j, 116);
            let top = foot + into * tall * 0.6 + Vec3::Y * tall;
            let bend = (foot + top) * 0.5 + into * tall * 0.15;
            tube(
                f,
                &spline(&[foot, bend, top], 3),
                5,
                |u| 0.032 - 0.008 * u,
                0.0,
                0.0,
                STALK,
            );
            let r = 0.08 + 0.08 * h(k * 13 + j, 117);
            blob(
                f,
                top + Vec3::Y * r * 0.15,
                vec3(r, r * 0.5, r),
                300 + k * 13 + j,
                0.06,
                |u| if u.y > -0.15 { CAP } else { GILL },
                true,
            );
        }
    };
    for k in 0..10u32 {
        let a = TAU_F * h(k, 111);
        let y = WALL_TOP - 2.0 - (WALL_TOP - WALL_BOTTOM - 4.0) * h(k, 112);
        let foot = on_wall(a, y, 0.05);
        cluster(
            &mut f,
            k,
            foot,
            vec3(-foot.x, 0.0, -foot.z).normalize_or_zero(),
        );
    }
    for k in 0..6u32 {
        let s = -0.05 + 1.1 * (k as f32 + 0.5) / 6.0;
        let a = dsc::angle(s);
        let out = vec3(a.sin(), 0.0, a.cos()) * if k % 2 == 0 { 1.0 } else { -1.0 };
        let foot = dsc::path(s) + out * (ROOT_R * 0.95);
        cluster(&mut f, 40 + k, foot, out);
    }
    // 균사 줄: 벽을 길게 기는 검고 가는 줄(갈래 하나씩).
    for k in 0..8u32 {
        let a0 = TAU_F * h(k, 121);
        let y0 = WALL_TOP - (WALL_TOP - WALL_BOTTOM - 3.0) * h(k, 122);
        let c: Vec<Vec3> = (0..=5)
            .map(|i| {
                let t = i as f32 / 5.0;
                let wig = (h(k, 130 + i) - 0.5) * 0.3;
                on_wall(a0 + t * 0.9 + wig, y0 - t * 3.2, 0.02)
            })
            .collect();
        let line = spline(&c, 4);
        tube(&mut f, &line, 4, |u| 0.035 - 0.015 * u, 0.0, 0.0, CORD);
        let mid = line[line.len() / 2];
        let a1 = mid.x.atan2(mid.z) + 0.35;
        let c2 = [mid, on_wall(a1, mid.y - 1.0, 0.02)];
        tube(
            &mut f,
            &spline(&c2, 4),
            4,
            |u| 0.025 * (1.0 - u),
            0.0,
            0.0,
            CORD,
        );
    }
    // 자갈: 벽에 박힌 작은 돌(바위 사이의 크기).
    for k in 0..26u32 {
        let a = TAU_F * h(k, 141);
        let y = WALL_TOP - (WALL_TOP - WALL_BOTTOM) * h(k, 142);
        let r = 0.1 + 0.12 * h(k, 143);
        let col = mul(dsc::soil(depth_at(y) + 30.0), 1.1 + 0.3 * h(k, 144));
        blob(
            &mut f,
            on_wall(a, y, 0.03),
            vec3(r, r * 0.75, r),
            400 + k,
            0.15,
            |_| col,
            false,
        );
    }
    // 빛나는 옹이: 큰 뿌리의 옆구리(굴 축 쪽·벽 쪽 번갈아).
    for k in 0..7u32 {
        let s = -0.15 + 1.3 * (k as f32 + h(k, 61)) / 7.0;
        let side = if k % 2 == 0 { 1.0 } else { -1.0 };
        let a = dsc::angle(s);
        let c = dsc::path(s) + vec3(a.sin(), 0.0, a.cos()) * (ROOT_R * 0.92) * side - Vec3::Y * 0.1;
        let r = 0.06 + 0.03 * h(k, 62);
        blob(&mut f, c, Vec3::splat(r), 500 + k, 0.1, |_| KNOT, true);
    }
    ShaftMesh {
        key: (from_m, to_m),
        parts: f,
    }
}

impl Renderer {
    /// 굴 장면 한 프레임(0174): 월드 대신 그린다 — 같은 타깃·같은 선·같은 마무리 패스. `clock` = 실제 시간(걷기 동작·먼지).
    /// `upper`·`lower` = 굴의 위·아래 지역의 환경(달이 있나 · 안개 빛).
    pub fn draw_shaft(
        &mut self,
        p: &Passage,
        weapon: crate::weapons::Id,
        t: &Tuning,
        clock: f32,
        (upper, lower): (&crate::backdrop::Env, &crate::backdrop::Env),
    ) -> Frame {
        use crate::tuning::P;
        if self.shaft.as_ref().map(|m| m.key) != Some((p.from_m, p.to_m)) {
            self.shaft = Some(build(p.from_m, p.to_m));
        }
        // 그림자 가림판은 월드의 것 — 이 프레임에는 없다.
        self.light_vp = None;
        self.batch.casters.clear();
        self.batch.caster_idx.clear();
        let (iw, ih) = self.fit_screen();
        let (eye, target) = dsc::eye(p);
        let c3 = Camera3D {
            position: eye,
            target,
            up: Vec3::Y,
            fovy: dsc::EYE_FOV,
            aspect: Some(iw as f32 / ih as f32),
            projection: Projection::Perspective,
            render_target: Some(self.rt.clone()),
            viewport: None,
            z_near: 0.1,
            z_far: 200.0,
        };
        set_look(Look {
            toon: t[P::RenderToon] >= 0.5,
            rim: t[P::RenderRim],
            outline_px: t[P::RenderOutline],
            eye,
            fovy: c3.fovy,
            rows: ih as f32,
            calm: t[P::RenderFloorCalm] >= 0.5,
            key: t[P::RenderKeyLight] >= 0.5,
            enemy_lift: t[P::RenderEnemyLift],
            enemy_ash: t[P::RenderEnemyAsh],
        });
        set_camera(&c3);
        set_actor_shade(None);
        // 굴 속에는 달빛(한쪽에서 오는 빛)이 없다 — 위의 틈에서 떨어지는 빛줄기와 등잔뿐.
        set_moonlight(0.0);
        let s = p.travel();
        let dark = mul(lower.fog_deep, 0.35 + 0.25 * (1.0 - s));
        clear_background(Color::new(dark.r, dark.g, dark.b, 1.0));

        let (feet, yaw) = dsc::walker(p);
        let fwd = vec3(yaw.sin(), 0.0, yaw.cos());
        // 빛 셋: 등잔(방랑자 곁) · 위의 달빛(위 지역에 달이 있을 때, 위에 가까울수록) · 아래에서 올라오는 닿을 곳의 빛.
        let near_top = (1.0 - s * 1.6).clamp(0.0, 1.0);
        let near_bottom = ((s - 0.45) * 1.8).clamp(0.0, 1.0);
        // 등잔은 살짝 일렁인다(불꽃 — 두 사인의 합, 실제 시간).
        let flick = 1.0 + 0.06 * (clock * 9.3).sin() + 0.04 * (clock * 23.7 + 1.3).sin();
        let mut lights = vec![Light {
            pos: feet + Vec3::Y * 1.25 + fwd * 0.45,
            color: mul(LANTERN, flick),
            radius: 7.5 * (0.97 + 0.03 * flick),
            shaft: false,
        }];
        if upper.moon && near_top > 0.0 {
            lights.push(Light {
                pos: vec3(0.0, WALL_TOP, 0.0),
                color: mul(MOONBEAM, 0.75 * near_top),
                radius: SHAFT_R * 1.25,
                shaft: true,
            });
        }
        if near_bottom > 0.0 {
            lights.push(Light {
                pos: vec3(0.0, -DROP - 3.5, 0.0),
                color: mul(lower.fog, 1.6 * near_bottom),
                radius: 9.0,
                shaft: false,
            });
        }

        let edges_on = t[P::RenderEdge] >= 0.5;
        self.batch.record = edges_on;
        self.batch.recorded.clear();
        // 굴의 덩어리: 정점마다 빛을 다시 받아 보낸다(한 그리기 호출의 한도 안에서 끊는다 — `batch::fits`).
        if let Some(mesh) = &self.shaft {
            let mut verts: Vec<Vertex> = Vec::with_capacity(10_000);
            let mut idx: Vec<u16> = Vec::with_capacity(15_000);
            for part in &mesh.parts {
                if !batch::fits(verts.len(), idx.len(), part.v.len(), part.idx.len()) {
                    self.batch.raw(&verts, &idx, None, 1.0);
                    verts.clear();
                    idx.clear();
                }
                let b = verts.len() as u16;
                for v in &part.v {
                    let k = if v.glow {
                        Vec3::splat(1.1)
                    } else {
                        // 바탕빛은 굴 속이라 조금 낮게, 등잔·빛줄기의 몫은 그대로.
                        let base = lit(v.p, v.n, &[]) * CAVE_DIM;
                        base + (lit(v.p, v.n, &lights) - lit(v.p, v.n, &[]))
                    };
                    let col = Color::new(v.col.r * k.x, v.col.g * k.y, v.col.b * k.z, 1.0);
                    verts.push(crate::scenery::with_normal(
                        Vertex::new(v.p.x, v.p.y, v.p.z, 0.0, 0.0, col),
                        v.n,
                    ));
                }
                idx.extend(part.idx.iter().map(|i| i + b));
            }
            if !verts.is_empty() {
                self.batch.raw(&verts, &idx, None, 1.0);
            }
        }
        // 방랑자: 걷기 동작, 든 무기만 보인다(0051).
        if let Some(e) = self.lib.get("wanderer") {
            let mut hide: Vec<(usize, Quat, Vec3)> = Vec::new();
            for w in crate::weapons::WEAPONS.iter() {
                if w.id != weapon {
                    if let Some(n) = e.model.node(w.node) {
                        hide.push((n, Quat::IDENTITY, Vec3::ZERO));
                    }
                }
            }
            e.model
                .pose_with(e.pick(&["walk", "idle"]), clock, &hide, &mut self.pose);
            self.batch.model(
                e,
                &self.pose,
                Mat4::from_translation(feet) * Mat4::from_rotation_y(yaw),
                WHITE,
                &lights,
                1.0,
            );
        }
        self.batch.flush();
        self.batch.record = false;

        // 빛줄기: 위의 틈에서 떨어지는 달빛(위에 가까울 때), 아래에서 올라오는 닿을 곳의 빛(아래에 가까울 때).
        if upper.moon && near_top > 0.0 {
            shaft(
                vec3(0.0, feet.y - 4.0, 0.0),
                vec3(0.0, WALL_TOP + 2.0, 0.0),
                0.6,
                2.6,
                with_a(MOONBEAM, 0.0),
                with_a(MOONBEAM, 0.22 * near_top),
            );
        }
        if near_bottom > 0.0 {
            beam(
                vec3(0.0, -DROP - 7.0, 0.0),
                SHAFT_R * 0.75,
                7.0,
                with_a(mul(lower.fog, 1.8), 0.2 * near_bottom),
            );
        }
        // 등잔의 빛무리와 떠다니는 먼지(내려가면 위로 흐르는 듯, 실제 시간으로).
        glow_disc(
            feet + Vec3::Y * 1.2 + fwd * 0.4,
            0.9 * flick,
            with_a(LANTERN, 0.16 * flick),
        );
        // 발걸음(0175): 걷기 동작(0.8초)의 두 디딤마다 발밑에서 흙 부스러기가 튀어 떨어진다 — 뿌리를 밟는 무게.
        let step = 0.4;
        let since = clock.rem_euclid(step);
        let n_step = (clock / step).floor() as u32;
        let side = vec3(fwd.z, 0.0, -fwd.x)
            * if n_step.is_multiple_of(2) {
                0.14
            } else {
                -0.14
            };
        for k in 0..5u32 {
            let life = 0.45;
            if since > life {
                break;
            }
            let u = since / life;
            let dir = vec3(
                h(n_step * 5 + k, 151) - 0.5,
                0.0,
                h(n_step * 5 + k, 152) - 0.5,
            ) * 0.9;
            let p = feet + side + dir * u + Vec3::Y * (0.12 * u * 4.0 * (1.0 - u) - 0.25 * u * u);
            gem(
                p,
                0.035 + 0.02 * h(k, 153),
                with_a(Color::new(0.62, 0.5, 0.42, 1.0), 0.8 * (1.0 - u)),
            );
        }
        // 위에서 떨어지는 흙 알갱이 몇(굴이 살아 있다 — 가끔, 곧게).
        for k in 0..6u32 {
            let period = 2.2 + 1.6 * h(k, 161);
            let ph = (clock / period + h(k, 162)).rem_euclid(1.0);
            let a = TAU_F * h(k, 163) + (clock / period).floor() * 1.7;
            let r = SHAFT_R * (0.55 + 0.35 * h(k, 164));
            let y = feet.y + 5.0 - ph * 9.0;
            gem(
                vec3(r * a.sin(), y, r * a.cos()),
                0.045,
                with_a(Color::new(0.72, 0.6, 0.5, 1.0), 0.7 * (1.0 - ph)),
            );
        }
        let drift = if p.up { -0.25 } else { 0.25 };
        for k in 0..36u32 {
            let a = std::f32::consts::TAU * h(k, 71) + clock * 0.05 * (h(k, 72) - 0.5);
            let r = SHAFT_R * (0.15 + 0.75 * h(k, 73));
            let span = 9.0;
            let y = feet.y + 4.5
                - (h(k, 74) * span + clock * drift * (0.4 + h(k, 75))).rem_euclid(span);
            let near = 1.0 - ((y - feet.y - 1.0).abs() / 5.0).clamp(0.0, 1.0);
            gem(
                vec3(r * a.sin(), y, r * a.cos()),
                0.05 + 0.04 * h(k, 76),
                with_a(Color::new(0.9, 0.82, 0.7, 1.0), 0.15 + 0.45 * near),
            );
        }
        flush_lines();
        self.edge_ready = edges_on && self.normal_pass(&c3, (iw, ih));
        self.edge_uniform = vec4(
            t[P::RenderEdgeDepth],
            t[P::RenderEdgeDark],
            t[P::RenderEdgeLight],
            t[P::RenderEdgeCrease],
        );
        self.grade_uniform = vec4(
            t[P::RenderGrade],
            t[P::RenderBloom],
            t[P::RenderVignette],
            if self.edge_ready { 1.0 } else { 0.0 },
        );
        self.present()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::descent::{PATH_R, SWEEP};

    fn tris(m: &ShaftMesh) -> Vec<[SV; 3]> {
        m.parts
            .iter()
            .flat_map(|p| {
                p.idx
                    .chunks(3)
                    .map(move |t| [p.v[t[0] as usize], p.v[t[1] as usize], p.v[t[2] as usize]])
            })
            .collect()
    }

    /// 0174 → 0177: 굴의 벽은 모두 굴 안(축)을 보고, 큰 뿌리의 윗면은 방랑자가 걷는 높이에 있다 — 발이 뿌리에 묻히거나 뜨지 않는다.
    /// 덩어리마다 한 그리기 호출에 들어가고, 정점 법선은 모두 길이 1이다.
    #[test]
    fn the_wall_faces_inward_and_the_great_root_carries_the_walker() {
        let m = build(0, 40);
        let verts: usize = m.parts.iter().map(|p| p.v.len()).sum();
        assert!((5_000..60_000).contains(&verts), "{verts}");
        for p in &m.parts {
            assert!(
                batch::fits(0, 0, p.v.len(), p.idx.len()),
                "a part too big for one draw"
            );
            assert!(
                p.v.iter().all(|v| (v.n.length() - 1.0).abs() < 1e-3),
                "unit normals"
            );
        }
        // 벽(지층 띠마다의 덩어리 — 먼저 지은 것들)은 축을 본다. 벽에 반쯤 묻힌 바위·뿌리의 뒷면은 바깥을 보는 게 맞다.
        for p in m.parts.iter().take(bands().len()) {
            for v in &p.v {
                if v.n.y.abs() < 0.5 {
                    assert!(
                        v.n.dot(vec3(-v.p.x, 0.0, -v.p.z)) > 0.0,
                        "wall faces the axis at {:?}",
                        v.p
                    );
                }
            }
        }
        // 걷는 구간의 여러 자리에서: 발 바로 아래 큰 뿌리의 윗면이 있다(위를 보는 면, 발보다 조금 아래).
        let all = tris(&m);
        for i in 0..=10 {
            let s = i as f32 / 10.0;
            let feet = dsc::path(s) + Vec3::Y * ROOT_R;
            let under = all.iter().any(|t| {
                let mid = (t[0].p + t[1].p + t[2].p) / 3.0;
                let n = (t[0].n + t[1].n + t[2].n).normalize();
                n.y > 0.8
                    && vec2(mid.x - feet.x, mid.z - feet.z).length() < 0.6
                    && (-0.05..0.25).contains(&(feet.y - mid.y))
            });
            assert!(under, "no root top under the walker at s={s}");
        }
        // 굴 한가운데(카메라와 방랑자 사이)는 비어 있다: 축 둘레 1.2m 안에는 정점이 없다. 벽은 큰 뿌리에 닿지 않는다.
        assert!(m
            .parts
            .iter()
            .flat_map(|p| &p.v)
            .all(|v| vec2(v.p.x, v.p.z).length() > 1.2));
        for k in 0..WALL_COLS {
            let a = TAU_F * k as f32 / WALL_COLS as f32;
            for u in [0.0, 0.5, 1.0] {
                assert!(wall_r(a, 0.0, u, 3) > PATH_R + ROOT_R * 1.1);
            }
        }
        // 길은 굴을 `SWEEP`만큼 돈다(상수가 서로 맞는지 — 길의 반지름은 벽 안).
        assert!(PATH_R + ROOT_R < SHAFT_R - 0.4 && SWEEP > 1.5);
    }

    /// 0177: 둥근 꼴은 매끈하다 — 큰 뿌리의 이웃 정점 법선이 크게 꺾이지 않는다(면마다 빛을 받던 조각 꼴이 아니다), 지층의 이웃 띠는 이음에서 꼭 맞는다.
    #[test]
    fn the_shaft_is_smooth_round_masses_not_shards() {
        let mut parts = Vec::new();
        let pts: Vec<Vec3> = (0..=40).map(|i| dsc::path(i as f32 / 40.0)).collect();
        tube(&mut parts, &pts, 14, |_| ROOT_R, 0.045, 5.0, ROOT);
        let p = &parts[0];
        for t in p.idx.chunks(3) {
            let [a, b, c] = [t[0], t[1], t[2]].map(|i| p.v[i as usize].n);
            assert!(
                a.dot(b) > 0.7 && b.dot(c) > 0.7 && a.dot(c) > 0.7,
                "a sharp crease"
            );
        }
        for (k, top, bottom) in bands().windows(2).map(|w| w[0]) {
            let _ = top;
            for j in 0..8 {
                let a = TAU_F * j as f32 / 8.0;
                assert!((wall_r(a, bottom, 1.0, k) - wall_r(a, bottom, 0.0, k + 1)).abs() < 1e-4);
            }
        }
    }
}
