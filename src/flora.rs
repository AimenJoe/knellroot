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

const TRUNK: Color = Color::new(0.25, 0.20, 0.27, 1.0);
const LEAF_DEAD: Color = Color::new(0.31, 0.31, 0.41, 1.0);
const LEAF_LIVE: Color = Color::new(0.34, 0.50, 0.45, 1.0);
const ROOT: Color = Color::new(0.27, 0.15, 0.34, 1.0);
/// 살아 있는 나무의 몫.
const LIVE_SHARE: f32 = 0.3;

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
fn icosahedron() -> ([Vec3; 12], [[usize; 3]; 20]) {
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
pub fn tree(p: &mut Piece, base: Vec3, seed: u32, lights: &[Light]) -> Aabb {
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
    let live = h(seed, 4) < LIVE_SHARE;
    let leaf = if live { LEAF_LIVE } else { LEAF_DEAD };
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
pub fn bush(p: &mut Piece, base: Vec3, seed: u32, size: f32, lights: &[Light]) {
    let n = 2 + (h(seed, 1) * 1.99) as u32;
    let live = h(seed, 2) < LIVE_SHARE;
    let leaf = if live { LEAF_LIVE } else { LEAF_DEAD };
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
        tree(&mut p, base, 3, &[]);
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
            let b = tree(&mut p, base, seed, &[]);
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
