//! 걸을 수 있는 높낮이(0043, 0035의 2단계). 지도와 같은 격자의 칸마다 땅 높이(0.5m 단위)와 경사로.
//! 월드는 스테이지를 모른다 — 스테이지가 이 격자를 만들어 `World::ground`에 넣는다(없으면 평평한 땅 0).
//!
//! 규칙(사장 2026-11-04: "경사로 + 낮은 턱 점프"):
//! - 발보다 `GROUND_STEP`(0.3m) 넘게 높은 칸은 벽. 점프로 발이 그 높이까지 올라가 있으면 넘어간다 — 0.5m 턱은 점프(0.9m)로 오르고
//!   1m 넘는 벼랑은 못 오른다.
//! - 내려가기: 플레이어는 어디서든 뛰어내린다(떨어진다). 적은 경사로로만 오르내린다 — 낮은 칸도 벽(`can_drop` 거짓).
//! - 경사로 칸은 낮은 이웃에서 높은 이웃으로 칸 안에서 곧게 오른다(이음매에 턱이 없다).
//! - 적 길찾기: 칸 격자 너비 우선(이웃끼리 맞닿은 변의 높이 차 ≤ 걸음), 플레이어 칸에서 퍼진다.
use macroquad::prelude::*;

use crate::tuning::GROUND_STEP;

/// 칸 하나의 땅: 높이(경사로면 축의 낮은 변 → 높은 변).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cell {
    /// 벽·허공(걸을 수 없다 — 부딪힘은 벽 상자가 맡는다).
    Solid,
    Flat(f32),
    /// 경사로: `x_axis`면 x가 커질수록, 아니면 z가 커질수록 `from` → `to`.
    Ramp {
        x_axis: bool,
        from: f32,
        to: f32,
    },
}

#[derive(Clone, Debug)]
pub struct Ground {
    pub w: usize,
    pub h: usize,
    pub cell: f32,
    /// 칸 (0,0)의 최소 모서리(월드 x, z).
    pub origin: Vec2,
    pub cells: Vec<Cell>,
}

impl Ground {
    /// 지도 가운데 = 원점(스테이지 `Level::at`과 같은 놓임).
    pub fn new(w: usize, h: usize, cell: f32, cells: Vec<Cell>) -> Ground {
        assert_eq!(cells.len(), w * h);
        Ground {
            w,
            h,
            cell,
            origin: vec2(-(w as f32) * cell * 0.5, -(h as f32) * cell * 0.5),
            cells,
        }
    }

    pub fn at(&self, x: usize, z: usize) -> Cell {
        if x < self.w && z < self.h {
            self.cells[z * self.w + x]
        } else {
            Cell::Solid
        }
    }

    /// 월드 자리 → 칸.
    pub fn cell_of(&self, p: Vec3) -> Option<(usize, usize)> {
        let fx = ((p.x - self.origin.x) / self.cell).floor();
        let fz = ((p.z - self.origin.y) / self.cell).floor();
        (fx >= 0.0 && fz >= 0.0 && (fx as usize) < self.w && (fz as usize) < self.h)
            .then_some((fx as usize, fz as usize))
    }

    /// 칸의 최소 모서리.
    fn min_of(&self, x: usize, z: usize) -> Vec2 {
        self.origin + vec2(x as f32, z as f32) * self.cell
    }

    pub fn center(&self, x: usize, z: usize) -> Vec3 {
        let m = self.min_of(x, z) + Vec2::splat(self.cell * 0.5);
        vec3(
            m.x,
            self.height_in(x, z, vec3(m.x, 0.0, m.y)).unwrap_or(0.0),
            m.y,
        )
    }

    /// 칸 (x, z) 안의 점 `p`(칸 밖이면 칸 안 가장 가까운 점)의 땅 높이. 벽·허공이면 None.
    pub fn height_in(&self, x: usize, z: usize, p: Vec3) -> Option<f32> {
        match self.at(x, z) {
            Cell::Solid => None,
            Cell::Flat(h) => Some(h),
            Cell::Ramp { x_axis, from, to } => {
                let m = self.min_of(x, z);
                let k = if x_axis {
                    (p.x - m.x) / self.cell
                } else {
                    (p.z - m.y) / self.cell
                };
                Some(from + (to - from) * k.clamp(0.0, 1.0))
            }
        }
    }

    /// 월드 자리의 땅 높이(격자 밖·벽이면 0 — 그 자리엔 서지 못한다).
    pub fn height(&self, p: Vec3) -> f32 {
        self.cell_of(p)
            .and_then(|(x, z)| self.height_in(x, z, p))
            .unwrap_or(0.0)
    }

    /// 원(`p`, `r`)을 발 높이 `feet`에서 걸을 수 없는 칸 밖으로 밀어낸다: 발보다 `step_up` 넘게 높은 칸(걸을 때 `GROUND_STEP`,
    /// 공중이면 0 — 발이 턱 높이를 넘어야 올라선다), `can_drop`이 거짓이면 걸음 넘게 낮은 칸도.
    /// 높이는 그 칸에서 원에 가장 가까운 점의 높이(경사로는 맞닿은 변).
    pub fn push_circle(&self, p: Vec3, r: f32, feet: f32, step_up: f32, can_drop: bool) -> Vec3 {
        let Some((cx, cz)) = self.cell_of(p) else {
            return p;
        };
        let mut q = p;
        for dz in -1i32..=1 {
            for dx in -1i32..=1 {
                if dx == 0 && dz == 0 {
                    continue;
                }
                let (x, z) = (cx as i32 + dx, cz as i32 + dz);
                // 격자 밖은 벽처럼 막는다(스테이지는 벽이 두르지만, 격자 끝에서 땅 0으로 걸어 나가지 않게).
                let outside = x < 0 || z < 0 || x as usize >= self.w || z as usize >= self.h;
                let m = self.origin + vec2(x as f32, z as f32) * self.cell;
                if outside {
                    let near = vec2(
                        q.x.clamp(m.x, m.x + self.cell),
                        q.z.clamp(m.y, m.y + self.cell),
                    );
                    let d = vec2(q.x, q.z) - near;
                    let len = d.length();
                    if len < r && len > 1e-5 {
                        let push = d / len * (r - len);
                        q.x += push.x;
                        q.z += push.y;
                    }
                    continue;
                }
                let (x, z) = (x as usize, z as usize);
                let near = vec3(
                    q.x.clamp(m.x, m.x + self.cell),
                    0.0,
                    q.z.clamp(m.y, m.y + self.cell),
                );
                let Some(gh) = self.height_in(x, z, near) else {
                    continue; // 벽 칸은 벽 상자가 막는다.
                };
                let blocks = gh > feet + step_up || (!can_drop && gh < feet - GROUND_STEP);
                if !blocks {
                    continue;
                }
                let d = vec2(q.x - near.x, q.z - near.z);
                let len = d.length();
                if len < r && len > 1e-5 {
                    let push = d / len * (r - len);
                    q.x += push.x;
                    q.z += push.y;
                }
            }
        }
        q
    }

    /// 두 칸이 이웃끼리 걸어서(적처럼) 오갈 수 있나: 맞닿은 변 가운데에서 높이 차 ≤ 걸음.
    fn passable(&self, a: (usize, usize), b: (usize, usize)) -> bool {
        let ma = self.min_of(a.0, a.1);
        let mb = self.min_of(b.0, b.1);
        let edge = (ma + mb) * 0.5 + Vec2::splat(self.cell * 0.5);
        let e = vec3(edge.x, 0.0, edge.y);
        match (self.height_in(a.0, a.1, e), self.height_in(b.0, b.1, e)) {
            (Some(ha), Some(hb)) => (ha - hb).abs() <= GROUND_STEP,
            _ => false,
        }
    }

    /// `from`에서 `to`까지 곧게 걸어(적처럼) 갈 수 있나 — 지나는 칸을 이어 가며 이웃끼리 오갈 수 있는지 본다.
    pub fn straight_walkable(&self, from: Vec3, to: Vec3) -> bool {
        let (Some(mut c), Some(goal)) = (self.cell_of(from), self.cell_of(to)) else {
            return false;
        };
        let steps = ((to - from).length() / (self.cell * 0.25)).ceil().max(1.0) as usize;
        for i in 1..=steps {
            let p = from.lerp(to, i as f32 / steps as f32);
            let Some(n) = self.cell_of(p) else {
                return false;
            };
            if n != c {
                // 대각선으로 넘어가면 두 길 가운데 하나라도 이어져야 한다.
                let ok = if n.0 != c.0 && n.1 != c.1 {
                    (self.passable(c, (n.0, c.1)) && self.passable((n.0, c.1), n))
                        || (self.passable(c, (c.0, n.1)) && self.passable((c.0, n.1), n))
                } else {
                    self.passable(c, n)
                };
                if !ok {
                    return false;
                }
                c = n;
            }
        }
        c == goal
    }

    /// 너비 우선 거리 지도: 칸마다 `target`까지 걸음 수(못 가면 u16::MAX).
    pub fn flow(&self, target: Vec3) -> Vec<u16> {
        let mut dist = vec![u16::MAX; self.w * self.h];
        let Some(t) = self.cell_of(target) else {
            return dist;
        };
        let mut q = std::collections::VecDeque::new();
        dist[t.1 * self.w + t.0] = 0;
        q.push_back(t);
        while let Some(c) = q.pop_front() {
            let d = dist[c.1 * self.w + c.0];
            for (dx, dz) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                let (x, z) = (c.0 as i32 + dx, c.1 as i32 + dz);
                if x < 0 || z < 0 || x as usize >= self.w || z as usize >= self.h {
                    continue;
                }
                let n = (x as usize, z as usize);
                if dist[n.1 * self.w + n.0] == u16::MAX && self.passable(c, n) {
                    dist[n.1 * self.w + n.0] = d + 1;
                    q.push_back(n);
                }
            }
        }
        dist
    }

    /// 거리 지도(`flow`)를 따라 `from`에서 한 칸 가까운 이웃의 가운데. 이미 목표 칸이거나 길이 없으면 None.
    pub fn next_step(&self, dist: &[u16], from: Vec3) -> Option<Vec3> {
        let c = self.cell_of(from)?;
        let here = dist[c.1 * self.w + c.0];
        if here == 0 || here == u16::MAX {
            return None;
        }
        [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)]
            .iter()
            .filter_map(|(dx, dz)| {
                let (x, z) = (c.0 as i32 + dx, c.1 as i32 + dz);
                (x >= 0 && z >= 0 && (x as usize) < self.w && (z as usize) < self.h)
                    .then_some((x as usize, z as usize))
            })
            .filter(|n| dist[n.1 * self.w + n.0] < here && self.passable(c, *n))
            .min_by_key(|n| dist[n.1 * self.w + n.0])
            .map(|n| self.center(n.0, n.1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 5×3 격자(칸 2m): 왼쪽 두 칸 바닥 0, 가운데 경사로 0 → 1, 오른쪽 두 칸 단 1m. 가운데 줄 아래는 0.5m 턱.
    fn yard() -> Ground {
        use Cell::*;
        let r = Ramp {
            x_axis: true,
            from: 0.0,
            to: 1.0,
        };
        Ground::new(
            5,
            3,
            2.0,
            vec![
                Flat(0.0),
                Flat(0.0),
                Flat(0.0),
                Flat(0.0),
                Flat(0.0), //
                Flat(0.0),
                Flat(0.0),
                r,
                Flat(1.0),
                Flat(1.0), //
                Flat(0.5),
                Flat(0.5),
                Solid,
                Flat(1.0),
                Flat(1.0),
            ],
        )
    }

    #[test]
    fn a_ramp_rises_smoothly_across_its_cell() {
        let g = yard();
        let c = g.center(2, 1);
        assert!((c.y - 0.5).abs() < 1e-5, "ramp middle {}", c.y);
        let left = vec3(c.x - 0.99, 0.0, c.z);
        let right = vec3(c.x + 0.99, 0.0, c.z);
        assert!(g.height(left) < 0.02 && g.height(right) > 0.98);
    }

    #[test]
    fn a_ledge_above_the_step_blocks_walking_but_not_a_high_enough_jump() {
        let g = yard();
        // 바닥(0)에서 아래 줄 0.5m 턱 쪽으로 걷기: 막힌다.
        let edge = g.min_of(0, 2).y; // 턱 칸의 위 변(z)
        let p = vec3(-3.0, 0.0, edge - 0.1);
        let q = g.push_circle(p, 0.35, 0.0, GROUND_STEP, true);
        assert!(q.z < p.z - 0.2, "pushed back from the ledge: {q:?}");
        // 발이 0.5m 위(점프)면 그대로 넘어간다 — 0.4m(모자람)면 막힌다.
        assert!(g.push_circle(p, 0.35, 0.4, 0.0, true).z < p.z - 0.2);
        assert_eq!(g.push_circle(p, 0.35, 0.5, 0.0, true), p);
    }

    #[test]
    fn players_may_drop_down_but_enemies_stay_on_their_level() {
        let g = yard();
        // 0.5m 턱 위(아래 줄)에서 바닥 쪽 변으로: 플레이어는 지나가고 적은 막힌다.
        let edge = g.min_of(0, 2).y;
        let p = vec3(-3.0, 0.0, edge + 0.1);
        assert_eq!(g.push_circle(p, 0.35, 0.5, 0.0, true), p);
        assert!(g.push_circle(p, 0.35, 0.5, GROUND_STEP, false).z > p.z + 0.2);
    }

    #[test]
    fn enemies_find_the_ramp_to_reach_the_platform() {
        let g = yard();
        let start = g.center(0, 0);
        let goal = g.center(4, 1);
        assert!(
            !g.straight_walkable(g.center(1, 2), goal),
            "the 0.5 ledge row cannot walk straight onto the 1m platform"
        );
        let dist = g.flow(goal);
        let mut p = start;
        let mut seen = vec![];
        for _ in 0..12 {
            match g.next_step(&dist, p) {
                Some(n) => {
                    seen.push(g.cell_of(n).unwrap());
                    p = n;
                }
                None => break,
            }
        }
        assert_eq!(g.cell_of(p), Some((4, 1)), "reached the platform: {seen:?}");
        assert!(seen.contains(&(2, 1)), "via the ramp: {seen:?}");
        // 0.5m 턱 칸에서는 길이 없다(오르려면 점프 — 적은 못 한다).
        assert_eq!(dist[2 * 5], u16::MAX);
    }
}
