//! 적 길찾기(0076). 걸을 수 있는 땅(`Ground`, 0043) 위에 **부딪힘 상자(기둥·소품·상자·닫힌 문)가 덮는 칸**을 막힌 칸으로 얹은 격자.
//! 땅이 없는 아레나(개발 시나리오)는 1m 칸의 평평한 격자를 만든다 — 길찾기는 한 길로.
//!
//! 고친 것(사장 2026-11-07: "적들의 길찾기가 좀 멍청함"):
//! - 거리 지도·곧게 갈 수 있나가 땅 높이만 보고 기둥·소품을 몰랐다 → 적이 기둥을 향해 곧장 걸어 박혀 비비적거렸다.
//! - 길을 따라갈 때 칸 가운데를 하나씩 밟아 계단처럼 꺾였다 → 앞으로 이어진 칸 가운데 **곧게 갈 수 있는 가장 먼 칸**을 겨눈다
//!   (끈 당기기, string pulling). 곧게는 몸 폭(가운데 + 양옆 반지름)으로 본다 — 모서리를 긁지 않게.
//! - 돌진은 벽·기둥·턱에서 멈춘다(`charge_room`) — 예고 띠도 그 길이만큼.
//! - 높낮이(0100, 사장 "적들도 높은 곳으로 점프하거나 낮은 곳으로 뛰어내릴 수 있어야"): 이웃 칸으로 가는 법 셋 —
//!   걷기(높이 차 ≤ 걸음), 뛰어오르기(≤ `ENEMY_CLIMB`), 뛰어내리기(≤ `ENEMY_DROP`). 거리 지도는 값이 다른 다익스트라
//!   (경사로가 가까우면 걸어서 돈다), 끈 당기기는 걷는 칸까지만, 뛰어야 하는 변 앞에서는 `hop`이 내려앉을 자리를 준다.
use macroquad::prelude::*;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::ground::{Cell, Ground};
use crate::tuning::{ENEMY_CLIMB, ENEMY_DROP, GROUND_STEP};
use crate::world::Aabb;

/// 이웃 칸으로 가는 법(0100).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Step {
    Walk,
    Up,
    Down,
}

impl Step {
    /// 거리 지도의 값: 뛰는 것은 걷기 몇 칸만큼 — 경사로가 몇 칸 안이면 그리로 돈다.
    fn cost(self) -> u16 {
        match self {
            Step::Walk => 2,
            Step::Down => 5,
            Step::Up => 8,
        }
    }
}

/// 뛰기 전, 몸이 변에서 이만큼(m) 안이면 뛴다.
const HOP_NEAR: f32 = 0.3;
/// 내려앉는 자리는 변 너머 몸 반지름 + 이만큼(m).
const HOP_LAND: f32 = 0.4;

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

    /// 칸 `a`에서 이웃 `b`로 적이 가는 법(맞닿은 변의 높이 차로) — 못 가면 없음.
    pub fn step(&self, a: (usize, usize), b: (usize, usize)) -> Option<Step> {
        let (ha, hb) = self.g.edge_heights(a, b)?;
        let dh = hb - ha;
        if dh.abs() <= GROUND_STEP {
            Some(Step::Walk)
        } else if dh > 0.0 && dh <= ENEMY_CLIMB {
            Some(Step::Up)
        } else if dh < 0.0 && -dh <= ENEMY_DROP {
            Some(Step::Down)
        } else {
            None
        }
    }

    /// 거리 지도(다익스트라): 칸마다 `target`까지의 값(걷기 2·뛰어내리기 5·뛰어오르기 8, 못 가면 u16::MAX).
    /// 목표 칸은 상자가 덮어도 연다(곁에 서 있을 수 있다). 적은 이웃 `n`에서 `c`로 오므로 `step(n, c)`를 본다.
    pub fn flow(&self, target: Vec3) -> Vec<u16> {
        let g = &self.g;
        let mut dist = vec![u16::MAX; g.w * g.h];
        let Some(t) = g.cell_of(target) else {
            return dist;
        };
        let mut heap = BinaryHeap::new();
        dist[t.1 * g.w + t.0] = 0;
        heap.push(Reverse((0u16, t)));
        while let Some(Reverse((d, c))) = heap.pop() {
            if d > dist[c.1 * g.w + c.0] {
                continue;
            }
            for n in self.neighbors(c) {
                if !self.open(n) {
                    continue;
                }
                let Some(s) = self.step(n, c) else {
                    continue;
                };
                let nd = d.saturating_add(s.cost());
                if nd < dist[n.1 * g.w + n.0] {
                    dist[n.1 * g.w + n.0] = nd;
                    heap.push(Reverse((nd, n)));
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

    /// 한 칸 가까운 이웃(거리 지도를 따라)과 가는 법 — 이웃의 값 + 가는 값이 가장 작은 쪽.
    fn next_cell(&self, dist: &[u16], c: (usize, usize)) -> Option<((usize, usize), Step)> {
        let here = dist[c.1 * self.g.w + c.0];
        if here == 0 || here == u16::MAX {
            return None;
        }
        self.neighbors(c)
            .filter(|n| dist[n.1 * self.g.w + n.0] < here)
            .filter_map(|n| self.step(c, n).map(|s| (n, s)))
            .min_by_key(|(n, s)| dist[n.1 * self.g.w + n.0].saturating_add(s.cost()))
    }

    /// 뛰어야 하는 변 앞에 왔나(0100): 다음 칸이 뛰어오르기·뛰어내리기이고 몸이 변에서 `HOP_NEAR` 안이면
    /// (내려앉을 자리, 가는 법). 내려앉는 자리는 옆으로는 지금 자리 그대로, 변 너머 몸 반지름 + `HOP_LAND`.
    pub fn hop(&self, dist: &[u16], from: Vec3, r: f32) -> Option<(Vec3, Step)> {
        let c = self.g.cell_of(from)?;
        let (n, s) = self.next_cell(dist, c)?;
        if s == Step::Walk {
            return None;
        }
        let (dir, to_edge) = self.edge_dir(c, n, from);
        if to_edge > r + HOP_NEAR {
            return None;
        }
        let mut land = from + dir * (to_edge + r + HOP_LAND);
        land.y = self.g.height(land);
        Some((land, s))
    }

    /// 칸 `a` → 이웃 `b` 쪽(xz 단위)과 `from`에서 맞닿은 변까지 그쪽으로 잰 거리.
    fn edge_dir(&self, a: (usize, usize), b: (usize, usize), from: Vec3) -> (Vec3, f32) {
        let (ca, cb) = (self.g.center(a.0, a.1), self.g.center(b.0, b.1));
        let dir = vec3(cb.x - ca.x, 0.0, cb.z - ca.z).normalize_or_zero();
        let m = self.g.edge_mid(a, b);
        (dir, vec3(m.x - from.x, 0.0, m.z - from.z).dot(dir))
    }

    /// 거리 지도를 따라 겨눌 자리(끈 당기기): 앞으로 `LOOKAHEAD`칸 가운데 몸 폭으로 곧게 갈 수 있는 가장 먼 칸의 가운데.
    /// 없으면(이미 목표 칸·길 없음) None.
    pub fn waypoint(&self, dist: &[u16], from: Vec3, r: f32) -> Option<Vec3> {
        let start = self.g.cell_of(from)?;
        let mut c = start;
        let mut chain = Vec::with_capacity(LOOKAHEAD);
        let mut jump = None;
        while chain.len() < LOOKAHEAD {
            match self.next_cell(dist, c) {
                Some((n, Step::Walk)) => {
                    chain.push(n);
                    c = n;
                }
                // 뛰어야 하는 변(0100): 걷는 길은 거기까지.
                Some((n, _)) => {
                    jump = Some((c, n));
                    break;
                }
                None => break,
            }
        }
        // 바로 앞이 뛰어야 하는 변이면 그 변 앞(몸 반지름만큼 안쪽, 옆으로는 지금 자리)으로 — 거기서 `hop`이 뛴다.
        if chain.is_empty() {
            let (a, b) = jump?;
            let (dir, to_edge) = self.edge_dir(a, b, from);
            let mut at = from + dir * (to_edge - r - 0.05).max(0.0);
            at.y = self.g.height(at);
            return Some(at);
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
