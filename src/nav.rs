//! 적 길찾기(0076). 걸을 수 있는 땅(`Ground`, 0043) 위에 **부딪힘 상자(기둥·소품·상자·닫힌 문)가 덮는 칸**을 막힌 칸으로 얹은 격자.
//! 땅이 없는 아레나(개발 시나리오)는 1m 칸의 평평한 격자를 만든다 — 길찾기는 한 길로.
//!
//! 고친 것(사장 2026-11-07: "적들의 길찾기가 좀 멍청함"):
//! - 거리 지도·곧게 갈 수 있나가 땅 높이만 보고 기둥·소품을 몰랐다 → 적이 기둥을 향해 곧장 걸어 박혀 비비적거렸다.
//! - 길을 따라갈 때 칸 가운데를 하나씩 밟아 계단처럼 꺾였다 → 앞으로 이어진 칸 가운데 **곧게 갈 수 있는 가장 먼 칸**을 겨눈다
//!   (끈 당기기, string pulling). 곧게는 몸 폭(가운데 + 양옆 반지름)으로 본다 — 모서리를 긁지 않게.
//! - 돌진은 벽·기둥·턱에서 멈춘다(`charge_room`) — 예고 띠도 그 길이만큼.
use macroquad::prelude::*;

use crate::ground::{Cell, Ground};
use crate::world::Aabb;

/// 땅이 없는 아레나의 격자 칸(m).
const ARENA_CELL: f32 = 1.0;
/// 상자가 칸을 이만큼(m) 넘게 덮어야 막힌 칸(벽 조각처럼 칸 끝에 딱 닿는 것은 이웃을 막지 않는다).
const COVER: f32 = 0.05;
/// 끈 당기기로 앞을 내다보는 칸 수.
const LOOKAHEAD: usize = 8;

pub struct Nav {
    pub g: Ground,
    /// 칸마다 상자가 덮었나.
    blocked: Vec<bool>,
    /// 만든 때의 상자 열쇠(문이 열리고 닫히면 바뀐다).
    key: (usize, i64),
}

/// 상자 목록의 열쇠 — 수와 자리 합(문을 열고 닫으면 바뀐다).
fn key_of(obstacles: &[Aabb]) -> (usize, i64) {
    let s: f32 = obstacles
        .iter()
        .map(|o| o.min.x * 3.0 + o.min.z * 7.0 + o.max.x * 11.0 + o.max.z * 13.0)
        .sum();
    (obstacles.len(), (s * 100.0) as i64)
}

impl Nav {
    pub fn build(ground: Option<&Ground>, obstacles: &[Aabb], arena: f32) -> Nav {
        let g = match ground {
            Some(g) => g.clone(),
            None => {
                let n = ((arena * 2.0) / ARENA_CELL).ceil().max(1.0) as usize;
                Ground::new(n, n, ARENA_CELL, vec![Cell::Flat(0.0); n * n])
            }
        };
        let mut blocked = vec![false; g.w * g.h];
        for o in obstacles {
            let lo = vec2(o.min.x, o.min.z) - g.origin;
            let hi = vec2(o.max.x, o.max.z) - g.origin;
            let x0 = ((lo.x + COVER) / g.cell).floor().max(0.0) as usize;
            let z0 = ((lo.y + COVER) / g.cell).floor().max(0.0) as usize;
            let x1 = ((hi.x - COVER) / g.cell).floor();
            let z1 = ((hi.y - COVER) / g.cell).floor();
            if x1 < 0.0 || z1 < 0.0 {
                continue;
            }
            for z in z0..=(z1 as usize).min(g.h - 1) {
                for x in x0..=(x1 as usize).min(g.w - 1) {
                    blocked[z * g.w + x] = true;
                }
            }
        }
        Nav {
            g,
            blocked,
            key: key_of(obstacles),
        }
    }

    /// 상자가 바뀌었나(그러면 다시 만든다).
    pub fn stale(&self, obstacles: &[Aabb]) -> bool {
        self.key != key_of(obstacles)
    }

    /// 칸에 설 수 있나(벽·허공·상자가 아님).
    fn open(&self, c: (usize, usize)) -> bool {
        !self.blocked[c.1 * self.g.w + c.0] && !matches!(self.g.at(c.0, c.1), Cell::Solid)
    }

    /// 너비 우선 거리 지도: 칸마다 `target`까지 걸음 수(못 가면 u16::MAX). 목표 칸은 상자가 덮어도 연다(곁에 서 있을 수 있다).
    pub fn flow(&self, target: Vec3) -> Vec<u16> {
        let g = &self.g;
        let mut dist = vec![u16::MAX; g.w * g.h];
        let Some(t) = g.cell_of(target) else {
            return dist;
        };
        let mut q = std::collections::VecDeque::new();
        dist[t.1 * g.w + t.0] = 0;
        q.push_back(t);
        while let Some(c) = q.pop_front() {
            let d = dist[c.1 * g.w + c.0];
            for n in self.neighbors(c) {
                if dist[n.1 * g.w + n.0] == u16::MAX && self.open(n) && g.passable(c, n) {
                    dist[n.1 * g.w + n.0] = d + 1;
                    q.push_back(n);
                }
            }
        }
        dist
    }

    fn neighbors(&self, c: (usize, usize)) -> impl Iterator<Item = (usize, usize)> + '_ {
        [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)]
            .into_iter()
            .filter_map(move |(dx, dz)| {
                let (x, z) = (c.0 as i32 + dx, c.1 as i32 + dz);
                (x >= 0 && z >= 0 && (x as usize) < self.g.w && (z as usize) < self.g.h)
                    .then_some((x as usize, z as usize))
            })
    }

    /// 한 칸 가까운 이웃(거리 지도를 따라).
    fn next_cell(&self, dist: &[u16], c: (usize, usize)) -> Option<(usize, usize)> {
        let here = dist[c.1 * self.g.w + c.0];
        if here == 0 || here == u16::MAX {
            return None;
        }
        self.neighbors(c)
            .filter(|n| dist[n.1 * self.g.w + n.0] < here && self.g.passable(c, *n))
            .min_by_key(|n| dist[n.1 * self.g.w + n.0])
    }

    /// 거리 지도를 따라 겨눌 자리(끈 당기기): 앞으로 `LOOKAHEAD`칸 가운데 몸 폭으로 곧게 갈 수 있는 가장 먼 칸의 가운데.
    /// 없으면(이미 목표 칸·길 없음) None.
    pub fn waypoint(&self, dist: &[u16], from: Vec3, r: f32) -> Option<Vec3> {
        let mut c = self.g.cell_of(from)?;
        let mut chain = Vec::with_capacity(LOOKAHEAD);
        while chain.len() < LOOKAHEAD {
            match self.next_cell(dist, c) {
                Some(n) => {
                    chain.push(n);
                    c = n;
                }
                None => break,
            }
        }
        let first = *chain.first()?;
        Some(
            chain
                .iter()
                .rev()
                .map(|n| self.g.center(n.0, n.1))
                .find(|p| self.clear(from, *p, r))
                .unwrap_or_else(|| self.g.center(first.0, first.1)),
        )
    }

    /// `from`에서 `to`까지 반지름 `r`인 몸으로 곧게 걸어 갈 수 있나: 가운데 줄은 칸을 이어 가며 높이·막힘을 보고,
    /// 양옆 줄(± r)은 막힌 칸·벽에 들지 않는지 본다(처음과 끝 r 안은 빼고 — 벽에 붙어 선 몸이 늘 막히지 않게).
    pub fn clear(&self, from: Vec3, to: Vec3, r: f32) -> bool {
        let g = &self.g;
        let (Some(start), Some(goal)) = (g.cell_of(from), g.cell_of(to)) else {
            return false;
        };
        let span = vec3(to.x - from.x, 0.0, to.z - from.z);
        let len = span.length();
        if len < 1e-4 {
            return true;
        }
        let dir = span / len;
        let side = vec3(-dir.z, 0.0, dir.x) * r;
        let steps = (len / (g.cell * 0.25)).ceil().max(1.0) as usize;
        let mut c = start;
        for i in 1..=steps {
            let k = i as f32 / steps as f32;
            let p = from + span * k;
            let Some(n) = g.cell_of(p) else {
                return false;
            };
            if n != c {
                let ok = |a: (usize, usize), b: (usize, usize)| {
                    (b == goal || self.open(b)) && g.passable(a, b)
                };
                let pass = if n.0 != c.0 && n.1 != c.1 {
                    (ok(c, (n.0, c.1)) && ok((n.0, c.1), n))
                        || (ok(c, (c.0, n.1)) && ok((c.0, n.1), n))
                } else {
                    ok(c, n)
                };
                if !pass {
                    return false;
                }
                c = n;
            }
            let along = len * k;
            if along > r && along < len - r {
                for s in [side, -side] {
                    match g.cell_of(p + s) {
                        Some(m)
                            if m == goal
                                || m == start
                                || (self.open(m) && (m == n || g.passable(n, m))) => {}
                        _ => return false,
                    }
                }
            }
        }
        c == goal
    }

    /// 돌진할 수 있는 길이(m): `from`에서 `dir`(단위)로 반지름 `r`인 몸이 `max`까지 가다 상자·벽·턱(걸음 넘는 높이 차)에 닿는 거리.
    pub fn charge_room(
        &self,
        obstacles: &[Aabb],
        arena: f32,
        from: Vec3,
        dir: Vec3,
        r: f32,
        max: f32,
    ) -> f32 {
        let mut room = max;
        // 상자: 몸 반지름만큼 부풀린 상자에 가운데 줄을 쏜다(높이는 보지 않는다 — 단 위 소품도 막는다).
        for o in obstacles {
            let fat = Aabb {
                min: vec3(o.min.x - r, -1e4, o.min.z - r),
                max: vec3(o.max.x + r, 1e4, o.max.z + r),
            };
            if let Some(d) = fat.ray(vec3(from.x, 0.0, from.z), dir) {
                if d > 1e-3 {
                    room = room.min(d);
                }
            }
        }
        // 아레나 경계.
        for (o, d) in [(from.x, dir.x), (from.z, dir.z)] {
            if d.abs() > 1e-6 {
                let wall = if d > 0.0 { arena - r } else { -arena + r };
                room = room.min(((wall - o) / d).max(0.0));
            }
        }
        // 땅: 몸 앞끝이 들어가는 칸이 벽·허공이거나 걸음 넘게 높낮이가 다르면 거기까지.
        let g = &self.g;
        let step = g.cell * 0.125;
        let mut s = 0.0;
        let mut c = g.cell_of(from + dir * r);
        while s + step <= room {
            let n = g.cell_of(from + dir * (s + step + r));
            if n != c {
                match (c, n) {
                    (Some(a), Some(b)) if g.passable(a, b) || diag_ok(g, a, b) => {}
                    _ => return s.min(room),
                }
                c = n;
            }
            s += step;
        }
        room
    }
}

/// 대각선으로 넘어가는 칸 사이: 두 길 가운데 하나라도 이어지면.
fn diag_ok(g: &Ground, a: (usize, usize), b: (usize, usize)) -> bool {
    if a.0 == b.0 || a.1 == b.1 {
        return false;
    }
    (g.passable(a, (b.0, a.1)) && g.passable((b.0, a.1), b))
        || (g.passable(a, (a.0, b.1)) && g.passable((a.0, b.1), b))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pillar(x: f32, z: f32, half: f32) -> Aabb {
        Aabb {
            min: vec3(x - half, 0.0, z - half),
            max: vec3(x + half, 3.0, z + half),
        }
    }

    /// 사장 2026-11-07: 기둥이 사이에 있으면 곧게 박히지 않고 돌아간다 — 거리 지도·곧게 보기가 상자를 안다.
    #[test]
    fn a_pillar_between_is_walked_around_not_into() {
        let obs = [pillar(0.0, 0.0, 1.0)];
        let nav = Nav::build(None, &obs, 8.0);
        let (from, to) = (vec3(0.0, 0.0, -5.0), vec3(0.0, 0.0, 5.0));
        assert!(!nav.clear(from, to, 0.45), "the pillar is in the way");
        let dist = nav.flow(to);
        let mut p = from;
        let mut went_wide = false;
        for _ in 0..40 {
            let Some(w) = nav.waypoint(&dist, p, 0.45) else {
                break;
            };
            assert!(nav.clear(p, w, 0.45), "every leg is clear: {p:?} → {w:?}");
            // 한 걸음(0.5m)씩 그쪽으로.
            let d = w - p;
            p += d.normalize_or_zero() * d.length().min(0.5);
            went_wide |= p.x.abs() > 1.0;
            let out = obs[0].push_circle(p, 0.45);
            assert!(out.distance(p) < 0.15, "never walks into the pillar: {p:?}");
        }
        assert!(went_wide, "went around the side");
        assert_eq!(nav.g.cell_of(p), nav.g.cell_of(to), "and arrived");
    }

    /// 끈 당기기: 열린 땅에서는 칸 가운데를 하나씩 밟지 않고 멀리 곧게 겨눈다.
    #[test]
    fn on_open_ground_the_waypoint_is_far_and_straight() {
        let nav = Nav::build(None, &[], 10.0);
        let to = vec3(6.0, 0.0, 6.0);
        let dist = nav.flow(to);
        let from = vec3(-6.0, 0.0, -6.0);
        let w = nav.waypoint(&dist, from, 0.45).unwrap();
        assert!(w.distance(from) > 4.0, "looks ahead: {w:?}");
    }

    /// 돌진은 기둥 앞에서 멈추고(몸 반지름만큼 앞), 열린 길이면 끝까지.
    #[test]
    fn a_charge_stops_at_a_pillar_or_the_arena_edge() {
        let obs = [pillar(0.0, 4.0, 0.6)];
        let nav = Nav::build(None, &obs, 8.0);
        let room = nav.charge_room(&obs, 8.0, Vec3::ZERO, Vec3::Z, 0.45, 6.5);
        assert!((room - (4.0 - 0.6 - 0.45)).abs() < 0.05, "{room}");
        let room = nav.charge_room(&obs, 8.0, Vec3::ZERO, Vec3::X, 0.45, 10.0);
        assert!((room - (8.0 - 0.45)).abs() < 0.05, "edge {room}");
        let room = nav.charge_room(&obs, 20.0, Vec3::ZERO, -Vec3::Z, 0.45, 6.5);
        assert!((room - 6.5).abs() < 1e-4, "open {room}");
    }
}
