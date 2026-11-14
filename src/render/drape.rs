//! 땅 높낮이를 따라 접는 바닥 그림(0083): 원판·부채꼴·고리·띠·선.
use super::*;

/// 바닥 그림을 땅 높낮이에 접는 것(2026-11-06): 점마다 그 자리의 땅 높이(경사로·턱 — `Ground::height_in`, 벽·격자 밖이면 적의 발 높이)
/// 위로 들어 올리고, 면은 `DRAPE_STEP`마다 잘게 나눠 턱에서 꺾이게 한다. 땅이 없으면(아레나) 발 높이 평면 그대로.
pub(super) struct Drape<'a> {
    pub(super) ground: Option<&'a crate::ground::Ground>,
}

/// 접는 칸 크기(m) — 턱 둘레의 기울어진 삼각형이 이보다 좁다.
const DRAPE_STEP: f32 = 0.3;

impl Drape<'_> {
    /// 점 (x, z)의 땅 높이(벽·격자 밖·땅 없음이면 None).
    fn ground_at(&self, x: f32, z: f32) -> Option<f32> {
        let p = vec3(x, 0.0, z);
        self.ground
            .and_then(|g| g.cell_of(p).and_then(|(cx, cz)| g.height_in(cx, cz, p)))
    }

    /// 그림 가운데 `c`가 그 자리 땅보다 얼마나 위인가(0083): 발밑 표시는 0, 가슴 높이 휘두름 부채꼴은 0.9 — 그 높이를 지키며 땅을 따라 접는다.
    pub(super) fn rise(&self, c: Vec3) -> f32 {
        self.ground_at(c.x, c.z).map_or(0.0, |g| c.y - g)
    }

    /// 점 (x, z)의 땅 높이 + `lift`. `base` = 그 그림의 기준(적의 발) — 벽·격자 밖이면 그것.
    pub(super) fn at(&self, x: f32, z: f32, base: f32, lift: f32) -> Vec3 {
        let y = self.ground_at(x, z).unwrap_or(base);
        vec3(x, y + lift, z)
    }

    /// 그림자(땅을 따라, 진하기 `k`배).
    pub(super) fn shadow_k(&self, p: Vec3, r: f32, k: f32) {
        self.sector(p, r, 0.0, 360.0, Color::new(0.0, 0.0, 0.0, 0.35 * k));
    }

    pub(super) fn shadow(&self, p: Vec3, r: f32) {
        self.shadow_k(p, r, 1.0);
    }

    fn mesh(vertices: Vec<Vertex>, indices: Vec<u16>) {
        draw_mesh(&Mesh {
            vertices,
            indices,
            texture: None,
        });
    }

    /// 부채꼴 면(가운데 `c`, 반지름 `r`): 반지름·각을 잘게 나눈 격자.
    pub(super) fn sector(&self, c: Vec3, r: f32, yaw: f32, arc: f32, col: Color) {
        if r <= 1e-3 {
            return;
        }
        let n = ((arc / 10.0).ceil() as usize).clamp(3, 48);
        let m = ((r / DRAPE_STEP).ceil() as usize).clamp(1, 24);
        let half = arc.to_radians() * 0.5;
        let mut v = Vec::with_capacity((n + 1) * m + 1);
        let mut idx = Vec::with_capacity(n * m * 6);
        let up = 0.02 + self.rise(c);
        let cv = self.at(c.x, c.z, c.y, up);
        v.push(Vertex::new(cv.x, cv.y, cv.z, 0.0, 0.0, col));
        for j in 1..=m {
            let rr = r * j as f32 / m as f32;
            for i in 0..=n {
                let a = yaw - half + arc.to_radians() * (i as f32 / n as f32);
                let (s, k) = a.sin_cos();
                let q = self.at(c.x + s * rr, c.z + k * rr, c.y, up);
                v.push(Vertex::new(q.x, q.y, q.z, 0.0, 0.0, col));
            }
        }
        let row = |j: usize, i: usize| (1 + (j - 1) * (n + 1) + i) as u16;
        for i in 0..n {
            idx.extend_from_slice(&[0, row(1, i), row(1, i + 1)]);
        }
        for j in 2..=m {
            for i in 0..n {
                let (a, b, c2, d) = (row(j - 1, i), row(j - 1, i + 1), row(j, i), row(j, i + 1));
                idx.extend_from_slice(&[a, c2, d, a, d, b]);
            }
        }
        Self::mesh(v, idx);
    }

    /// 부채꼴 테두리 띠(폭 0.12m). 부채꼴이면 양 옆 변도.
    pub(super) fn ring(&self, c: Vec3, r: f32, yaw: f32, arc: f32, col: Color) {
        if r <= 1e-3 {
            return;
        }
        let w = TELE_RING_W;
        let n = ((arc / 10.0).ceil() as usize)
            .max((arc.to_radians() * r / DRAPE_STEP).ceil() as usize)
            .clamp(3, 96);
        let half = arc.to_radians() * 0.5;
        let mut v = Vec::with_capacity((n + 1) * 2);
        let mut idx = Vec::with_capacity(n * 6);
        let up = 0.03 + self.rise(c);
        for i in 0..=n {
            let a = yaw - half + arc.to_radians() * (i as f32 / n as f32);
            let (s, k) = a.sin_cos();
            for rr in [r - w, r] {
                let q = self.at(c.x + s * rr, c.z + k * rr, c.y, up);
                v.push(Vertex::new(q.x, q.y, q.z, 0.0, 0.0, col));
            }
        }
        for i in 0..n {
            let b = (i * 2) as u16;
            idx.extend_from_slice(&[b, b + 1, b + 2, b + 1, b + 3, b + 2]);
        }
        if arc < 360.0 {
            for a in [yaw - half, yaw + half] {
                let (s, k) = a.sin_cos();
                self.line(c, c + vec3(s * r, 0.0, k * r), col);
            }
        }
        Self::mesh(v, idx);
    }

    /// 띠 면(시작 `p`, 방향 `dir`, 길이 `len`, 폭 `w`): 길이를 잘게 나눈 띠.
    pub(super) fn lane(&self, p: Vec3, dir: Vec3, len: f32, w: f32, col: Color) {
        if w <= 1e-3 || len <= 1e-3 {
            return;
        }
        let side = vec3(-dir.z, 0.0, dir.x) * (w * 0.5);
        let n = ((len / DRAPE_STEP).ceil() as usize).clamp(1, 96);
        let mut v = Vec::with_capacity((n + 1) * 2);
        let mut idx = Vec::with_capacity(n * 6);
        let up = 0.02 + self.rise(p);
        for i in 0..=n {
            let o = p + dir * (len * i as f32 / n as f32);
            for q in [o - side, o + side] {
                let q = self.at(q.x, q.z, p.y, up);
                v.push(Vertex::new(q.x, q.y, q.z, 0.0, 0.0, col));
            }
        }
        for i in 0..n {
            let b = (i * 2) as u16;
            idx.extend_from_slice(&[b, b + 1, b + 3, b, b + 3, b + 2]);
        }
        Self::mesh(v, idx);
    }

    pub(super) fn lane_edge(&self, p: Vec3, dir: Vec3, len: f32, w: f32, col: Color) {
        let side = vec3(-dir.z, 0.0, dir.x) * (w * 0.5);
        let (a, b) = (p - side, p + side);
        let (c, d) = (b + dir * len, a + dir * len);
        // 높이는 띠의 시작 `p`(적의 발)에서 — 모서리가 낮은 단 위에 걸려도 그 자리 땅을 따른다.
        let up = 0.04 + self.rise(p);
        for (x, z) in [(a, b), (b, c), (c, d), (d, a)] {
            self.line_up(x, z, col, up);
        }
    }

    /// 땅에 접힌 선(잘게 나눈 선분들). 땅 위 높이는 `a`에서 잰다.
    pub(super) fn line(&self, a: Vec3, b: Vec3, col: Color) {
        self.line_up(a, b, col, 0.04 + self.rise(a));
    }

    /// 땅에 접힌 선, 땅 위 `up`만큼.
    fn line_up(&self, a: Vec3, b: Vec3, col: Color, up: f32) {
        let n = (((b - a).length() / DRAPE_STEP).ceil() as usize).clamp(1, 96);
        let mut prev = self.at(a.x, a.z, a.y, up);
        for i in 1..=n {
            let q = a.lerp(b, i as f32 / n as f32);
            let cur = self.at(q.x, q.z, a.y, up);
            line(prev, cur, col);
            prev = cur;
        }
    }
}
