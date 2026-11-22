//! 식생(0063 E3): 굽는 지형(`scenery`)에 얹는 나무·덤불·벽을 타는 뿌리. 모두 정점 빛을 미리 받는 낮은 다각형.
//!
//! - 잎 덩어리는 면이 몇 개 안 되는 둥근 조약돌(정이십면체를 칸마다 조금 흔든 것) — 면마다 툰 빛 띠가 갈려 t3ssel8r 풍 잎 무더기가 된다
//!   (0061: 큰 면 몇 개의 둥근 덩어리. 상자는 "마인크래프트" 인상이라 쓰지 않는다).
//! - 뿌리만 각지다(0037): 네모 단면의 가는 줄기가 벽 꼭대기에서 바닥으로 구불구불 내려와 바닥에서 갈라진다.
//! - 색(0037 "색 = 생명"): 성소 둘레 나무는 대부분 바랜 잿빛 잎(뿌리에 먹힘), 몇 그루만 살아 있는 민트빛. 뿌리는 짙은 보라.
use macroquad::prelude::*;

use crate::render::lit;
use crate::scenery::{with_normal, Piece};
use crate::stage::Light;
use crate::world::Aabb;

// 0249: 죽은 나무는 잿빛 갈색 줄기·푸른 잿빛 잎(옛 보랏빛은 주인공 망토와 같은 색상이었다). 보라 뿌리(`ROOT`)는 침식의 표지라 그대로.
const TRUNK: Color = Color::new(0.241, 0.212, 0.192, 1.0);
const LEAF_DEAD: Color = Color::new(0.292, 0.322, 0.356, 1.0);
const LEAF_LIVE: Color = Color::new(0.34, 0.50, 0.45, 1.0);
const ROOT: Color = Color::new(0.27, 0.15, 0.34, 1.0);
/// 쓰러진 통나무(0262): 껍질(잿빛 갈색 — 줄기보다 조금 밝게, 바닥 흙과 갈리게)·잘린 면의 겉재·나이테·심재·이끼.
const BARK: Color = Color::new(0.41, 0.365, 0.322, 1.0);
const SAP: Color = Color::new(0.56, 0.5, 0.42, 1.0);
const RING: Color = Color::new(0.42, 0.37, 0.31, 1.0);
const HEART: Color = Color::new(0.5, 0.44, 0.36, 1.0);
const MOSS: Color = Color::new(0.25, 0.4, 0.27, 1.0);
/// 살아 있는 나무의 몫(성소 — 지역마다는 `scenery::Woods::live`).
pub const LIVE_SHARE: f32 = 0.3;

/// 0..1 값(같은 씨앗 = 같은 모양).
fn h(seed: u32, k: u32) -> f32 {
    let mut v = seed.wrapping_mul(0x9E37_79B9) ^ k.wrapping_mul(0x85EB_CA6B);
    v ^= v >> 15;
    v = v.wrapping_mul(0x2C1B_3C6D);
    v ^= v >> 12;
    (v & 0xffff) as f32 / 65535.0
}

fn mul(c: Color, k: f32) -> Color {
    Color::new(c.r * k, c.g * k, c.b * k, c.a)
}

/// 면 하나(면 법선으로 빛을 받는다).
fn tri(p: &mut Piece, a: Vec3, b: Vec3, c: Vec3, col: Color, lights: &[Light]) {
    let n = (b - a).cross(c - a).normalize_or_zero();
    if n == Vec3::ZERO {
        return;
    }
    let base = p.verts.len() as u16;
    for v in [a, b, c] {
        let l = lit(v, n, lights);
        p.verts.push(with_normal(
            Vertex::new(
                v.x,
                v.y,
                v.z,
                0.0,
                0.0,
                Color::new(col.r * l.x, col.g * l.y, col.b * l.z, 1.0),
            ),
            n,
        ));
    }
    p.idx.extend_from_slice(&[base, base + 1, base + 2]);
}

/// 정이십면체의 꼭짓점·면(바깥을 보는 감김).
pub(crate) fn icosahedron() -> ([Vec3; 12], [[usize; 3]; 20]) {
    let t = (1.0 + 5f32.sqrt()) * 0.5;
    let v = [
        vec3(-1.0, t, 0.0),
        vec3(1.0, t, 0.0),
        vec3(-1.0, -t, 0.0),
        vec3(1.0, -t, 0.0),
        vec3(0.0, -1.0, t),
        vec3(0.0, 1.0, t),
        vec3(0.0, -1.0, -t),
        vec3(0.0, 1.0, -t),
        vec3(t, 0.0, -1.0),
        vec3(t, 0.0, 1.0),
        vec3(-t, 0.0, -1.0),
        vec3(-t, 0.0, 1.0),
    ]
    .map(|p| p.normalize());
    let f = [
        [0, 11, 5],
        [0, 5, 1],
        [0, 1, 7],
        [0, 7, 10],
        [0, 10, 11],
        [1, 5, 9],
        [5, 11, 4],
        [11, 10, 2],
        [10, 7, 6],
        [7, 1, 8],
        [3, 9, 4],
        [3, 4, 2],
        [3, 2, 6],
        [3, 6, 8],
        [3, 8, 9],
        [4, 9, 5],
        [2, 4, 11],
        [6, 2, 10],
        [8, 6, 7],
        [9, 8, 1],
    ];
    (v, f)
}

/// 둥근 조약돌 덩어리: 반지름 `r`(축마다), 꼭짓점을 씨앗으로 ±15% 흔든다. 면 20, 정점 60.
pub fn blob(p: &mut Piece, c: Vec3, r: Vec3, seed: u32, col: Color, lights: &[Light]) {
    let (v, f) = icosahedron();
    let pts: Vec<Vec3> = v
        .iter()
        .enumerate()
        .map(|(i, q)| c + *q * r * (0.85 + 0.3 * h(seed, i as u32)))
        .collect();
    let start = p.verts.len();
    for (k, [a, b, d]) in f.iter().enumerate() {
        // 면마다 빛을 조금씩(같은 띠 안에서도 잎 무더기가 얼룩지게).
        let shade = 0.9 + 0.2 * h(seed, 40 + k as u32);
        tri(p, pts[*a], pts[*b], pts[*d], mul(col, shade), lights);
    }
    // 잎 무더기는 바람에 조금 흔들린다(0142 — 줄기·뿌리는 그대로).
    for v in &mut p.verts[start..] {
        crate::scenery::sway(v, LEAF_SWAY);
    }
}
/// 잎 무더기가 바람을 받는 몫(풀 끝 = 1).
const LEAF_SWAY: f32 = 0.3;

/// 쓰러진 통나무(0262 — 서막 숲의 뛰어넘는 턱, 사장 2026-11-21 "코드 도형으로"): 0177 꼴 — 정점마다 매끈한 법선의 둥근 관.
/// 등뼈는 `a` → `b`를 캣멀-롬으로 살짝 휘고(옆·아래로 씨앗만큼), 굵기는 밑동(`a`) `r`에서 우듬지 쪽으로 가늘어진다. 껍질은 길이로 난
/// 얕은 골(반지름과 빛이 함께 꺼진다), 두 끝은 잘린 면 — 바깥 껍질 테 · 겉재 · 나이테 한 줄 · 심재. 위에 이끼 몇 덩이(매끈한 법선,
/// 납작). 밑동 쪽에 부러진 뿌리 셋만 각지다(0037). 바람 몫은 0(밑동·줄기 규칙, 0142) — 흔들리지 않는다.
pub fn log(p: &mut Piece, a: Vec3, b: Vec3, r: f32, seed: u32, lights: &[Light]) {
    let axis = b - a;
    let len = axis.length();
    if len < 1e-3 {
        return;
    }
    let dir = axis / len;
    let side = Vec3::Y.cross(dir).normalize_or(Vec3::X);
    // 등뼈: 넷의 조정점(가운데 둘은 옆·아래로 조금) → 캣멀-롬.
    let bend = |k: u32, f: f32| {
        side * (h(seed, k) - 0.5) * 0.5 * r * f - Vec3::Y * h(seed, k + 1) * 0.12 * r
    };
    let ctrl = [
        a,
        a + axis * 0.33 + bend(1, 1.0),
        a + axis * 0.66 + bend(3, 1.6),
        b,
    ];
    let cr = |t: f32| -> Vec3 {
        let seg = (t * 3.0).min(2.999);
        let i = seg as usize;
        let u = seg - i as f32;
        let p0 = if i == 0 {
            ctrl[0] * 2.0 - ctrl[1]
        } else {
            ctrl[i - 1]
        };
        let (p1, p2) = (ctrl[i], ctrl[i + 1]);
        let p3 = if i + 2 < 4 {
            ctrl[i + 2]
        } else {
            ctrl[3] * 2.0 - ctrl[2]
        };
        let (u2, u3) = (u * u, u * u * u);
        0.5 * (2.0 * p1
            + (p2 - p0) * u
            + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * u2
            + (3.0 * p1 - p0 - 3.0 * p2 + p3) * u3)
    };
    // 굵기: 밑동은 조금 부풀고(뿌리 쪽) 우듬지 쪽으로 가늘어진다.
    let rad = |t: f32| r * (1.0 - 0.32 * t + 0.12 * (1.0 - (t * 8.0).min(1.0)));
    const RINGS: usize = 12;
    const SIDES: usize = 14;
    let tau = std::f32::consts::TAU;
    let groove = |ang: f32, t: f32| {
        let g = (ang * 7.0 + t * 1.7 + h(seed, 9) * tau).cos();
        g.max(0.0).powi(3)
    };
    let frame = |t: f32| -> (Vec3, Vec3, Vec3) {
        let e = 0.01;
        let fwd = (cr((t + e).min(1.0)) - cr((t - e).max(0.0))).normalize_or(dir);
        let sd = Vec3::Y.cross(fwd).normalize_or(side);
        (fwd, sd, fwd.cross(sd))
    };
    let base = p.verts.len() as u16;
    let push = |p: &mut Piece, v: Vec3, n: Vec3, col: Color| {
        let l = lit(v, n, lights);
        p.verts.push(with_normal(
            Vertex::new(
                v.x,
                v.y,
                v.z,
                0.0,
                0.0,
                Color::new(col.r * l.x, col.g * l.y, col.b * l.z, 1.0),
            ),
            n,
        ));
    };
    for i in 0..=RINGS {
        let t = i as f32 / RINGS as f32;
        let (_, sd, up) = frame(t);
        let c = cr(t);
        for j in 0..=SIDES {
            let ang = j as f32 / SIDES as f32 * tau;
            let out = sd * ang.cos() + up * ang.sin();
            let g = groove(ang, t);
            let v = c + out * rad(t) * (1.0 - 0.08 * g);
            // 껍질 빛: 골은 어둡게, 큰 얼룩(씨앗)으로 조금씩.
            let k = (0.92 + 0.16 * h(seed, 20 + (i as u32 / 3))) * (1.0 - 0.3 * g);
            push(p, v, out, mul(BARK, k));
        }
    }
    let row = (SIDES + 1) as u16;
    for i in 0..RINGS as u16 {
        for j in 0..SIDES as u16 {
            let (a0, a1) = (base + i * row + j, base + i * row + j + 1);
            let (b0, b1) = (a0 + row, a1 + row);
            // 바깥을 보는 감김(앞 → 옆 둘레 = 바깥).
            p.idx.extend_from_slice(&[a0, b1, b0, a0, a1, b1]);
        }
    }
    // 잘린 면 둘: 가운데(심재) → 나이테 → 겉재 → 껍질 테, 면 법선은 축 쪽 밖으로.
    for (t, sign) in [(0.0f32, -1.0f32), (1.0, 1.0)] {
        let (fwd, sd, up) = frame(t);
        let n = fwd * sign;
        let c = cr(t);
        let r0 = rad(t);
        let start = p.verts.len() as u16;
        push(p, c, n, HEART);
        for (k, col) in [(0.5, RING), (0.62, SAP), (0.86, SAP), (1.0, BARK)] {
            for j in 0..SIDES {
                let ang = j as f32 / SIDES as f32 * tau;
                let v = c + (sd * ang.cos() + up * ang.sin()) * r0 * k + n * 0.002;
                push(p, v, n, col);
            }
        }
        let s = SIDES as u16;
        let wind = |x: u16, y: u16, z: u16, idx: &mut Vec<u16>| {
            if sign > 0.0 {
                idx.extend_from_slice(&[x, y, z]);
            } else {
                idx.extend_from_slice(&[x, z, y]);
            }
        };
        for j in 0..s {
            let jn = (j + 1) % s;
            wind(start, start + 1 + j, start + 1 + jn, &mut p.idx);
            for ring in 0..3u16 {
                let (o, q) = (start + 1 + ring * s, start + 1 + (ring + 1) * s);
                wind(o + j, q + j, q + jn, &mut p.idx);
                wind(o + j, q + jn, o + jn, &mut p.idx);
            }
        }
    }
    // 이끼 몇 덩이: 등 위에 납작하게(매끈한 법선 — 흔들리지 않는다).
    let (v12, f20) = icosahedron();
    for m in 0..4u32 {
        let t = 0.12 + 0.76 * (m as f32 + h(seed, 30 + m)) / 4.0;
        let (_, sd, up) = frame(t);
        let lean = (h(seed, 34 + m) - 0.5) * 0.9;
        let at = cr(t) + (up * lean.cos() + sd * lean.sin()) * rad(t) * 0.9;
        let size = rad(t) * (0.55 + 0.3 * h(seed, 38 + m));
        let start = p.verts.len() as u16;
        let col = mul(MOSS, 0.9 + 0.2 * h(seed, 42 + m));
        for q in v12.iter() {
            let d = vec3(q.x, q.y * 0.45, q.z);
            push(
                p,
                at + d * size,
                (*q * vec3(1.0, 2.2, 1.0)).normalize(),
                col,
            );
        }
        for f in f20.iter() {
            p.idx.extend_from_slice(&[
                start + f[0] as u16,
                start + f[1] as u16,
                start + f[2] as u16,
            ]);
        }
    }
    // 밑동의 부러진 뿌리 셋(각진 것은 뿌리만).
    let (fwd, sd, up) = frame(0.0);
    let c = cr(0.0);
    for k in 0..3u32 {
        let ang = (k as f32 / 3.0 + h(seed, 50 + k) * 0.2) * tau;
        let out = sd * ang.cos() + up * ang.sin().max(-0.2);
        let from = c + out * rad(0.0) * 0.7 + fwd * 0.05;
        let to = from - fwd * (0.35 + 0.25 * h(seed, 54 + k)) + out * 0.25 - Vec3::Y * 0.1;
        strand(
            p,
            &[from, (from + to) * 0.5 + out * 0.06, to],
            0.07,
            0.025,
            TRUNK,
            lights,
        );
    }
}

/// 네모 단면 줄기(뿌리·가지): 점을 따라 굵기 `r0` → `r1`. 각진 것은 뿌리만(0037).
fn strand(p: &mut Piece, pts: &[Vec3], r0: f32, r1: f32, col: Color, lights: &[Light]) {
    let n = pts.len();
    if n < 2 {
        return;
    }
    let ring = |i: usize| -> [Vec3; 4] {
        let dir = if i + 1 < n {
            pts[i + 1] - pts[i]
        } else {
            pts[i] - pts[i - 1]
        }
        .normalize_or_zero();
        let side = if dir.y.abs() > 0.9 { Vec3::X } else { Vec3::Y }
            .cross(dir)
            .normalize_or_zero();
        let up = dir.cross(side);
        let r = r0 + (r1 - r0) * i as f32 / (n - 1) as f32;
        [
            pts[i] + (side + up) * r,
            pts[i] + (-side + up) * r,
            pts[i] + (-side - up) * r,
            pts[i] + (side - up) * r,
        ]
    };
    for i in 0..n - 1 {
        let (a, b) = (ring(i), ring(i + 1));
        for s in 0..4 {
            let t = (s + 1) % 4;
            tri(p, a[s], a[t], b[t], col, lights);
            tri(p, a[s], b[t], b[s], col, lights);
        }
    }
}

/// 나무 하나(`base` = 뿌리 자리 땅 위): 짧고 굵은 휜 줄기 + 둥근 잎 무더기(가운데 큰 덩어리 위에 하나, 둘레에 둘~넷).
/// 처음엔 줄기 2.6~4.2m·납작한 갓이라 막대 사탕처럼 보였다(스크린숏) — 줄기를 줄이고 무더기를 위로 쌓는다. 가림 상자를 돌려준다.
pub fn tree(p: &mut Piece, base: Vec3, seed: u32, live: f32, lights: &[Light]) -> Aabb {
    let height = 1.7 + 0.9 * h(seed, 1);
    let lean = vec3(h(seed, 2) - 0.5, 0.0, h(seed, 3) - 0.5) * 0.9;
    let mid = base + Vec3::Y * height * 0.5 + lean * 0.4;
    let top = base + Vec3::Y * height + lean;
    strand(
        p,
        &[base - Vec3::Y * 0.2, mid, top],
        0.3,
        0.15,
        TRUNK,
        lights,
    );
    let leaf = if h(seed, 4) < live {
        LEAF_LIVE
    } else {
        LEAF_DEAD
    };
    let clumps = 3 + (h(seed, 5) * 2.99) as u32;
    let mut lo = base - vec3(0.6, 0.2, 0.6);
    let mut hi = base + vec3(0.6, 0.0, 0.6);
    for k in 0..clumps {
        let a = h(seed, 10 + k) * std::f32::consts::TAU;
        // 0 = 가운데 큰 덩어리, 1 = 그 위, 나머지 = 둘레(조금 아래).
        let (d, up) = match k {
            0 => (0.0, 0.5),
            1 => (0.25, 1.35),
            _ => (0.7 + 0.3 * h(seed, 20 + k), 0.35),
        };
        let r = match k {
            0 => 1.25,
            1 => 0.85,
            _ => 0.75 + 0.25 * h(seed, 30 + k),
        } * (0.85 + 0.3 * h(seed, 6));
        let c = top + vec3(a.cos() * d, up, a.sin() * d);
        let rr = vec3(r, r * 0.9, r);
        blob(
            p,
            c,
            rr,
            seed * 31 + k,
            mul(leaf, 0.9 + 0.2 * h(seed, 50 + k)),
            lights,
        );
        lo = lo.min(c - rr * 1.15);
        hi = hi.max(c + rr * 1.15);
    }
    Aabb { min: lo, max: hi }
}

/// 덤불: 땅에 붙은 조약돌 둘~셋(`size` = 대략 높이 m). 바랜 잎.
pub fn bush(p: &mut Piece, base: Vec3, seed: u32, size: f32, live: f32, lights: &[Light]) {
    let n = 2 + (h(seed, 1) * 1.99) as u32;
    let leaf = if h(seed, 2) < live {
        LEAF_LIVE
    } else {
        LEAF_DEAD
    };
    for k in 0..n {
        let a = h(seed, 10 + k) * std::f32::consts::TAU;
        let d = if k == 0 { 0.0 } else { size * 0.6 };
        let r = size * (0.55 + 0.25 * h(seed, 20 + k));
        let c = base + vec3(a.cos() * d, r * 0.55, a.sin() * d);
        blob(
            p,
            c,
            vec3(r, r * 0.75, r),
            seed * 17 + k,
            mul(leaf, 0.85 + 0.2 * h(seed, 30 + k)),
            lights,
        );
    }
}

/// 벽을 타는 뿌리: 벽 윗변의 점 `top`에서 바깥 법선 `out` 쪽 면을 따라 바닥 `floor_y`까지 구불구불, 바닥에서 두 갈래로 퍼진다.
pub fn wall_root(p: &mut Piece, top: Vec3, out: Vec3, floor_y: f32, seed: u32, lights: &[Light]) {
    let side = Vec3::Y.cross(out).normalize_or_zero();
    let r = 0.11 + 0.07 * h(seed, 1);
    let lift = out * (r + 0.02);
    let steps = 5;
    let mut pts = Vec::with_capacity(steps + 1);
    let span = top.y - floor_y;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let wig = (h(seed, 10 + i as u32) - 0.5) * 0.7 * (1.0 - (2.0 * t - 1.0).powi(2) * 0.5);
        pts.push(top + lift - Vec3::Y * span * t + side * wig);
    }
    // 위에서는 벽 꼭대기를 넘어 살짝 걸친다.
    let over = top - out * 0.25 + Vec3::Y * r;
    let mut all = vec![over];
    all.extend(pts.iter().copied());
    let col = mul(ROOT, 0.9 + 0.25 * h(seed, 2));
    strand(p, &all, r * 0.7, r * 1.3, col, lights);
    // 바닥에서 두 갈래.
    let foot = *pts.last().unwrap();
    for (k, s) in [(0u32, 1.0f32), (1, -1.0)] {
        let a = (h(seed, 20 + k) - 0.5) * 0.8;
        let dir = (out * a.cos() + side * s * (0.4 + a.sin().abs())).normalize_or_zero();
        let len = 0.5 + 0.5 * h(seed, 30 + k);
        let tip = foot + dir * len + Vec3::Y * 0.02;
        strand(
            p,
            &[foot, (foot + tip) * 0.5 + Vec3::Y * 0.05, tip],
            r * 1.1,
            r * 0.35,
            col,
            lights,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn piece() -> Piece {
        Piece {
            verts: Vec::new(),
            idx: Vec::new(),
            tex: None,
            aabb: None,
            fade: 1.0,
            lightmap: false,
        }
    }

    /// 0142 바람: 나무는 잎 무더기만 살랑이고 줄기는 가만히 선다(줄기가 흔들리면 뿌리째 미끄러져 보인다).
    #[test]
    fn only_the_leaves_of_a_tree_sway() {
        let mut p = piece();
        let base = vec3(0.0, 0.0, 0.0);
        tree(&mut p, base, 3, LIVE_SHARE, &[]);
        let swaying: Vec<&Vertex> = p.verts.iter().filter(|v| v.normal.w != 1.0).collect();
        assert!(!swaying.is_empty() && swaying.len() < p.verts.len());
        for v in swaying {
            assert_eq!(v.normal.w, 1.0 + LEAF_SWAY);
            assert!(
                v.position.y > base.y + 1.0,
                "a leaf at the foot of the trunk"
            );
        }
    }

    /// 0262 쓰러진 통나무: 바람 몫 0(흔들리지 않는다) · 관의 면은 바깥을 본다(감김 = 정점 법선 쪽) · 밑동이 우듬지보다 굵다 ·
    /// 두 끝 사이에 눕는다(높이는 굵기 안 — 솟은 가시·가지가 없다).
    #[test]
    fn a_fallen_log_lies_still_round_and_outward() {
        let mut p = piece();
        let (a, b) = (vec3(-4.0, 0.3, 0.0), vec3(4.0, 0.3, 0.0));
        log(&mut p, a, b, 0.36, 7, &[]);
        assert!(p.verts.len() > 200 && p.verts.len() < u16::MAX as usize);
        assert!(p.verts.iter().all(|v| v.normal.w == 1.0), "no wind");
        assert!(p
            .verts
            .iter()
            .all(|v| (v.normal.truncate().length() - 1.0).abs() < 1e-3));
        let mut outward = 0;
        for t in p.idx.chunks(3) {
            let [x, y, z] = [0, 1, 2].map(|k| p.verts[t[k] as usize].position);
            let n = (y - x).cross(z - x);
            let vn = p.verts[t[0] as usize].normal.truncate();
            if n.dot(vn) > 0.0 {
                outward += 1;
            }
        }
        // 뿌리(면 법선의 네모 줄기)는 제 감김이 따로다 — 거의 모든 면이 바깥.
        assert!(outward * 10 >= p.idx.len() / 3 * 9, "{outward}");
        let thick = |x: f32| {
            p.verts
                .iter()
                .filter(|v| (v.position.x - x).abs() < 0.5)
                .map(|v| -v.position.y)
                .fold(f32::MIN, f32::max)
        };
        // 아랫면의 깊이로 잰다(이끼는 등 위에만 — 윗면은 이끼가 올린다).
        assert!(thick(-3.0) > thick(3.0), "the butt is thicker");
        assert!(p
            .verts
            .iter()
            .all(|v| v.position.y < 0.3 + 0.36 * 1.6 && v.position.y > 0.3 - 0.36 * 1.6));
    }

    #[test]
    fn the_icosahedron_faces_point_outward() {
        let (v, f) = icosahedron();
        for [a, b, c] in f {
            let n = (v[b] - v[a]).cross(v[c] - v[a]);
            let centre = (v[a] + v[b] + v[c]) / 3.0;
            assert!(n.dot(centre) > 0.0);
        }
    }

    #[test]
    fn a_tree_is_a_few_hundred_vertices_stands_on_its_base_and_its_box_holds_the_crown() {
        for seed in 0..40 {
            let mut p = piece();
            let base = vec3(3.0, 2.0, -5.0);
            let b = tree(&mut p, base, seed, LIVE_SHARE, &[]);
            assert!(p.verts.len() < 500, "{}", p.verts.len());
            let ys: Vec<f32> = p.verts.iter().map(|v| v.position.y).collect();
            let top = ys.iter().cloned().fold(f32::MIN, f32::max);
            assert!(top > base.y + 2.5 && top < base.y + 6.0, "{top}");
            for v in &p.verts {
                let q = v.position;
                assert!(
                    q.x >= b.min.x - 0.3
                        && q.x <= b.max.x + 0.3
                        && q.z >= b.min.z - 0.3
                        && q.z <= b.max.z + 0.3
                );
            }
        }
    }

    #[test]
    fn a_wall_root_hangs_on_the_wall_face_down_to_the_floor() {
        let mut p = piece();
        let out = Vec3::Z;
        wall_root(&mut p, vec3(0.0, 3.0, 1.0), out, 0.0, 7, &[]);
        let lo = p
            .verts
            .iter()
            .map(|v| v.position.y)
            .fold(f32::MAX, f32::min);
        assert!(lo < 0.1, "{lo}");
        // 벽 면(z = 1) 안으로 거의 들어가지 않는다(꼭대기에 걸친 끝만).
        let inside = p
            .verts
            .iter()
            .filter(|v| v.position.z < 0.9 && v.position.y < 2.8)
            .count();
        assert_eq!(inside, 0);
    }
}
