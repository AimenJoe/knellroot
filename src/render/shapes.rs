//! 모양 도우미: 색 투명도, 카메라를 보는 판(마름모·별), 모아 긋는 선, 음영 상자·면 덩이, 빛기둥·빛무리.
use super::*;

pub(super) fn with_a(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a)
}

thread_local! {
    /// 이 프레임에 그을 선(나중에 한꺼번에). 면(삼각형)과 선을 번갈아 그리면 macroquad가 그리기 호출을 그때마다 끊는다 —
    /// 배우마다 면 → 윤곽선 → 코라서 잡몹 80이면 호출 ~180, 다 모여 예고를 띄우면 더. macroquad는 호출 칸마다 GPU 버퍼
    /// (정점 10,000개분 ≈ 400KB)를 잡고 줄이지 않아, 오래 두면 핸드헬드가 죽었다(사장 2026-11-03, 0016). 선을 모아 한 번에 그린다.
    static LINES: std::cell::RefCell<Vec<(Vec3, Vec3, Color)>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// 카메라를 보는 판의 오른쪽·위(길이 1) — 그 자리에서 눈을 보게.
fn facing_axes(p: Vec3) -> (Vec3, Vec3) {
    let f = (p - look().eye).normalize_or(Vec3::NEG_Z);
    let r = f.cross(Vec3::Y).normalize_or(Vec3::X);
    (r, r.cross(f).normalize_or(Vec3::Y))
}

pub(super) fn flat_mesh(pts: &[(Vec3, Color)], indices: &[u16]) {
    draw_mesh(&Mesh {
        vertices: pts
            .iter()
            .map(|(p, c)| Vertex::new(p.x, p.y, p.z, 0.0, 0.0, *c))
            .collect(),
        indices: indices.to_vec(),
        texture: None,
    });
}

/// 카메라를 보는 마름모(0141 — 정육면체 대신: 탄·조각·모으는 빛의 덩이). `size` = 꼭짓점에서 꼭짓점.
pub(super) fn gem(p: Vec3, size: f32, col: Color) {
    let (r, u) = facing_axes(p);
    let h = size * 0.5;
    flat_mesh(
        &[
            (p + u * h, col),
            (p + r * h, col),
            (p - u * h, col),
            (p - r * h, col),
        ],
        &[0, 1, 2, 0, 2, 3],
    );
}

/// 카메라를 보는 네 갈래 별(0141 — 휘청의 별·도화선 불꽃): 끝이 뾰족한 마름모 둘. `size` = 끝에서 끝.
pub(super) fn spark4(p: Vec3, size: f32, col: Color) {
    let (r, u) = facing_axes(p);
    let (h, t) = (size * 0.5, size * 0.15);
    flat_mesh(
        &[
            (p - r * h, col),
            (p - u * t, col),
            (p + r * h, col),
            (p + u * t, col),
            (p - u * h, col),
            (p - r * t, col),
            (p + u * h, col),
            (p + r * t, col),
        ],
        &[0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7],
    );
}

pub(super) fn line(a: Vec3, b: Vec3, col: Color) {
    LINES.with(|l| l.borrow_mut().push((a, b, col)));
}

pub(super) fn flush_lines() {
    LINES.with(|l| {
        for (a, b, c) in l.borrow_mut().drain(..) {
            draw_line_3d(a, b, c);
        }
    });
}

/// 돌린 상자의 모서리 12개(`draw_cube_wires`는 축 정렬뿐). 바로 긋지 않고 `line`으로 모은다.
pub(super) fn box_wires(center: Vec3, size: Vec3, col: Color, rot: Quat, pivot: Vec3) {
    let h = size * 0.5;
    let at = |x: f32, y: f32, z: f32| pivot + rot * (center + h * vec3(x, y, z) - pivot);
    for (a, b) in [
        ((-1., -1., -1.), (1., -1., -1.)),
        ((-1., 1., -1.), (1., 1., -1.)),
        ((-1., -1., 1.), (1., -1., 1.)),
        ((-1., 1., 1.), (1., 1., 1.)),
        ((-1., -1., -1.), (-1., 1., -1.)),
        ((1., -1., -1.), (1., 1., -1.)),
        ((-1., -1., 1.), (-1., 1., 1.)),
        ((1., -1., 1.), (1., 1., 1.)),
        ((-1., -1., -1.), (-1., -1., 1.)),
        ((1., -1., -1.), (1., -1., 1.)),
        ((-1., 1., -1.), (-1., 1., 1.)),
        ((1., 1., -1.), (1., 1., 1.)),
    ] {
        line(at(a.0, a.1, a.2), at(b.0, b.1, b.2), col);
    }
}

/// `shaded_box`를 `pivot` 둘레로 `rot`만큼 돌린 것. 음영도 돌린 면의 방향으로(빛은 고정).
pub(super) fn shaded_box_rot(
    center: Vec3,
    size: Vec3,
    col: Color,
    shade: f32,
    rot: Quat,
    pivot: Vec3,
) {
    let h = size * 0.5;
    let l = LIGHT_DIR.normalize();
    let faces: [(Vec3, Vec3, Vec3); 6] = [
        (Vec3::Y, Vec3::X, Vec3::Z),
        (Vec3::NEG_Y, Vec3::Z, Vec3::X),
        (Vec3::Z, Vec3::X, Vec3::NEG_Y),
        (Vec3::NEG_Z, Vec3::NEG_Y, Vec3::X),
        (Vec3::X, Vec3::NEG_Y, Vec3::Z),
        (Vec3::NEG_X, Vec3::Z, Vec3::NEG_Y),
    ];
    let mut vertices = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);
    for (n, u, v) in faces {
        let k = 1.0 - shade + shade * (rot * n).dot(l).max(0.0);
        let c = Color::new(col.r * k, col.g * k, col.b * k, col.a);
        let base = vertices.len() as u16;
        for (su, sv) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
            let p = pivot + rot * (center + n * h + u * h * su + v * h * sv - pivot);
            vertices.push(Vertex::new(p.x, p.y, p.z, 0.0, 0.0, c));
        }
        // 뒷면 컬링이 없어 감기 방향은 상관없다.
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    draw_mesh(&Mesh {
        vertices,
        indices,
        texture: None,
    });
}

/// 면마다 음영이 다른 상자. 축 정렬. 정점 24개(면마다 넷), 매 프레임 만들어 보낸다(정점은 어차피 매 프레임 올라간다).
pub(super) fn shaded_box(center: Vec3, size: Vec3, col: Color, shade: f32) {
    shaded_box_rot(center, size, col, shade, Quat::IDENTITY, center);
}

/// 세로 빛기둥(0060 W1): 열린 원통, 아래가 `col`의 투명도·위로 갈수록 사라진다. 앞뒤 면이 다 보이게 감김을 둘 다.
/// 빛줄기(0118 T2): 바닥 `a`(반지름 `ra`, 빛 `ca`) → 위 `b`(반지름 `rb`, 빛 `cb`)의 원뿔대 옆면(안팎 둘 다).
pub(super) fn shaft(a: Vec3, b: Vec3, ra: f32, rb: f32, ca: Color, cb: Color) {
    let n = 16;
    let mut vertices = Vec::with_capacity((n + 1) * 2);
    let mut indices = Vec::with_capacity(n * 12);
    for i in 0..=n {
        let t = std::f32::consts::TAU * i as f32 / n as f32;
        let (s, k) = t.sin_cos();
        vertices.push(Vertex::new(a.x + s * ra, a.y, a.z + k * ra, 0.0, 0.0, ca));
        vertices.push(Vertex::new(b.x + s * rb, b.y, b.z + k * rb, 0.0, 0.0, cb));
    }
    for i in 0..n {
        let q = (i * 2) as u16;
        indices.extend_from_slice(&[q, q + 1, q + 2, q + 1, q + 3, q + 2]);
        indices.extend_from_slice(&[q, q + 2, q + 1, q + 1, q + 2, q + 3]);
    }
    draw_mesh(&Mesh {
        vertices,
        indices,
        texture: None,
    });
}

/// 면마다 음영이 다른 삼각형 묶음 — `tris`는 가운데 `c` 기준 꼭짓점 셋씩, 법선은 바깥(가운데에서 먼 쪽)으로.
pub(super) fn shaded_tris(c: Vec3, tris: &[[Vec3; 3]], col: Color, shade: f32, rot: Quat) {
    let l = LIGHT_DIR.normalize();
    let mut vertices = Vec::with_capacity(tris.len() * 3);
    let mut indices = Vec::with_capacity(tris.len() * 3);
    // 바깥 = 꼴의 가운데(꼭짓점 평균)에서 먼 쪽 — 원점에서 재면 길쭉하게 솟은 꼴(뿌리 가시)의 옆면이 뒤집힌다.
    let n_pts = (tris.len() * 3).max(1) as f32;
    let mid = rot * (tris.iter().flatten().copied().sum::<Vec3>() / n_pts);
    for t in tris {
        let [a, b, d] = t.map(|p| rot * p);
        let mut n = (b - a).cross(d - a).normalize_or_zero();
        if n.dot((a + b + d) / 3.0 - mid) < 0.0 {
            n = -n;
        }
        let k = 1.0 - shade + shade * n.dot(l).max(0.0);
        let k = Color::new(col.r * k, col.g * k, col.b * k, col.a);
        let base = vertices.len() as u16;
        for p in [a, b, d] {
            let p = c + p;
            vertices.push(Vertex::new(p.x, p.y, p.z, 0.0, 0.0, k));
        }
        indices.extend_from_slice(&[base, base + 1, base + 2]);
    }
    draw_mesh(&Mesh {
        vertices,
        indices,
        texture: None,
    });
}

/// 솟는 뿌리 가시의 면(밑동 가운데 기준) — `(몸 여덟, 끝 넷)`: 반폭 `base`의 네모 밑동에서 높이 `h`의 목(반폭 `neck`)까지 가늘어지고
/// 그 위로 `tip`만큼 뾰족하다. `lean` = 높이 1마다 옆으로 기우는 양.
pub fn spike_faces(
    base: f32,
    neck: f32,
    h: f32,
    tip: f32,
    lean: Vec3,
) -> (Vec<[Vec3; 3]>, Vec<[Vec3; 3]>) {
    let corner = |k: usize, r: f32, y: f32| {
        let (sx, sz) = [(1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)][k % 4];
        vec3(sx * r, y, sz * r) + lean * y
    };
    let top = Vec3::Y * (h + tip) + lean * (h + tip) * 1.4;
    let (mut body, mut point) = (Vec::with_capacity(8), Vec::with_capacity(4));
    for k in 0..4 {
        let (a, b) = (corner(k, base, 0.0), corner(k + 1, base, 0.0));
        let (c, d) = (corner(k, neck, h), corner(k + 1, neck, h));
        body.push([a, b, d]);
        body.push([a, d, c]);
        point.push([c, d, top]);
    }
    (body, point)
}

/// 팔면체의 면 여덟(가운데 기준): 가로 반지름 `r`(x·z), 위·아래 꼭짓점까지 `up`·`down`.
pub fn octa_faces(r: Vec2, up: f32, down: f32) -> Vec<[Vec3; 3]> {
    let ring = [
        vec3(r.x, 0.0, 0.0),
        vec3(0.0, 0.0, r.y),
        vec3(-r.x, 0.0, 0.0),
        vec3(0.0, 0.0, -r.y),
    ];
    let (top, bottom) = (Vec3::Y * up, Vec3::NEG_Y * down);
    let mut out = Vec::with_capacity(8);
    for i in 0..4 {
        let (a, b) = (ring[i], ring[(i + 1) % 4]);
        out.push([top, a, b]);
        out.push([bottom, b, a]);
    }
    out
}

/// 둥근 덩이의 면 서른둘(가운데 기준): 팔면체의 면을 넷으로 쪼개 반지름 `r`의 공에 붙인다.
pub fn ball_faces(r: f32) -> Vec<[Vec3; 3]> {
    let on = |p: Vec3| p.normalize() * r;
    let mut out = Vec::with_capacity(32);
    for [a, b, d] in octa_faces(Vec2::ONE, 1.0, 1.0) {
        let (ab, bd, da) = (on(a + b), on(b + d), on(d + a));
        let (a, b, d) = (on(a), on(b), on(d));
        out.extend_from_slice(&[[a, ab, da], [ab, b, bd], [da, bd, d], [ab, bd, da]]);
    }
    out
}

/// 카메라를 보는 둥근 빛무리: 가운데 `col` → 가장자리 투명(0142 — 겹 정육면체의 반투명 겉 대신).
pub(super) fn glow_disc(p: Vec3, r: f32, col: Color) {
    const N: usize = 10;
    let (right, up) = facing_axes(p);
    let rim = with_a(col, 0.0);
    let mut pts = Vec::with_capacity(N + 1);
    let mut idx = Vec::with_capacity(N * 3);
    pts.push((p, col));
    for i in 0..N {
        let (s, k) = (std::f32::consts::TAU * i as f32 / N as f32).sin_cos();
        pts.push((p + (right * k + up * s) * r, rim));
        idx.extend_from_slice(&[0, 1 + i as u16, 1 + ((i + 1) % N) as u16]);
    }
    flat_mesh(&pts, &idx);
}

pub(super) fn beam(c: Vec3, r: f32, h: f32, col: Color) {
    let n = 16;
    let mut vertices = Vec::with_capacity((n + 1) * 2);
    let mut indices = Vec::with_capacity(n * 12);
    let top = Color::new(col.r, col.g, col.b, 0.0);
    for i in 0..=n {
        let a = std::f32::consts::TAU * i as f32 / n as f32;
        let (s, k) = a.sin_cos();
        vertices.push(Vertex::new(c.x + s * r, c.y, c.z + k * r, 0.0, 0.0, col));
        vertices.push(Vertex::new(
            c.x + s * r,
            c.y + h,
            c.z + k * r,
            0.0,
            0.0,
            top,
        ));
    }
    for i in 0..n {
        let b = (i * 2) as u16;
        indices.extend_from_slice(&[b, b + 1, b + 2, b + 1, b + 3, b + 2]);
        indices.extend_from_slice(&[b, b + 2, b + 1, b + 1, b + 2, b + 3]);
    }
    draw_mesh(&Mesh {
        vertices,
        indices,
        texture: None,
    });
}
