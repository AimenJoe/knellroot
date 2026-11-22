//! 글자 지도의 땅(0251 R11 — `stage/mod.rs`에서 뺐다): 읽기·높이·걸을 수 있는 칸·떼 자리·땅 높이 지도·장애물.
use super::*;

impl Level {
    pub fn cell(&self, x: usize, z: usize) -> u8 {
        if x < self.w && z < self.h {
            self.cells[z * self.w + x]
        } else {
            b' '
        }
    }

    /// 칸 가운데의 월드 자리(지도 가운데 = 원점, 행이 늘면 +z).
    pub fn at(&self, x: usize, z: usize) -> Vec3 {
        vec3(
            (x as f32 - self.w as f32 * 0.5 + 0.5) * CELL,
            0.0,
            (z as f32 - self.h as f32 * 0.5 + 0.5) * CELL,
        )
    }

    pub fn marks_of(&self, c: char) -> impl Iterator<Item = Vec3> + '_ {
        self.marks.iter().filter(move |m| m.0 == c).map(|m| m.1)
    }

    /// 월드 자리 → 칸 행(z).
    pub fn row_of(&self, p: Vec3) -> f32 {
        p.z / CELL + self.h as f32 * 0.5 - 0.5
    }

    /// 아레나 반폭(월드의 경계 — 벽이 안쪽에서 막는다).
    pub fn half_extent(&self) -> f32 {
        (self.w.max(self.h) as f32) * CELL * 0.5
    }

    /// 글자 지도 → 레벨. 표지: `#` 벽 · `.` 바닥 · ` ` 바깥 · `P` 시작 · `G` 문 · `i` 기둥 · `r` 뿌리 · `b` 화로 · `R` 쉼터 화로(0032, 화로 + 표지) · `u` 돌무더기 ·
    /// `w` 무너진 벽 조각 · `L` 종 · 그 밖의 글자는 바닥 + 표지(스폰 등, `marks`).
    pub fn parse(map: &str) -> Level {
        let rows: Vec<&str> = map.lines().filter(|l| !l.trim().is_empty()).collect();
        let h = rows.len();
        let w = rows.iter().map(|r| r.chars().count()).max().unwrap_or(0);
        let mut lv = Level {
            w,
            h,
            cells: vec![b' '; w * h],
            walls: Vec::new(),
            props: Vec::new(),
            lights: Vec::new(),
            marks: Vec::new(),
            player_start: Vec3::ZERO,
            cold: None,
            gate: None,
            heights: vec![None; w * h],
            ramps: vec![false; w * h],
        };
        let mut gate_cells = Vec::new();
        for (z, row) in rows.iter().enumerate() {
            for (x, c) in row.chars().enumerate() {
                let p = lv.at(x, z);
                let cell = match c {
                    '#' => b'#',
                    ' ' => b' ',
                    _ => b'.',
                };
                lv.cells[z * w + x] = cell;
                let prop = |kind| Prop {
                    kind,
                    pos: p,
                    yaw: ((x * 7 + z * 13) % 8) as f32 * std::f32::consts::FRAC_PI_4,
                };
                match c {
                    '#' | '.' | ' ' => {}
                    'P' => lv.player_start = p,
                    'G' => gate_cells.push(p),
                    'i' => lv.props.push(prop(PropKind::Pillar)),
                    'r' => lv.props.push(prop(PropKind::Root)),
                    'u' => lv.props.push(prop(PropKind::Rubble)),
                    'w' => lv.props.push(Prop {
                        kind: PropKind::WallPiece,
                        pos: p,
                        yaw: 0.0,
                    }),
                    ARCH_GLYPH => lv.props.push(Prop {
                        kind: PropKind::Arch,
                        pos: p,
                        yaw: 0.0,
                    }),
                    'L' => lv.props.push(Prop {
                        kind: PropKind::Bell,
                        pos: p,
                        yaw: 0.0,
                    }),
                    'b' | 'R' => {
                        if c == 'R' {
                            lv.marks.push(('R', p));
                        }
                        lv.props.push(Prop {
                            kind: PropKind::Brazier,
                            pos: p,
                            yaw: 0.0,
                        });
                        lv.lights.push(Light {
                            pos: p + Vec3::Y * 1.2,
                            color: Color::new(1.0, 0.62, 0.25, 1.0),
                            radius: 7.0,
                            shaft: false,
                        });
                    }
                    // 천장 틈 빛줄기(0118 T2, 땅속): 표지 + 옅은 빛(굽는 지형·배우가 받는다). 그림은 렌더가 표지로.
                    SHAFT_GLYPH => {
                        lv.marks.push((SHAFT_GLYPH, p));
                        lv.lights.push(Light {
                            pos: p + Vec3::Y * 1.0,
                            color: Color::new(0.78, 0.82, 1.0, 1.0),
                            radius: SHAFT_R,
                            shaft: true,
                        });
                    }
                    other => lv.marks.push((other, p)),
                }
            }
        }
        // 벽: 줄마다 이어진 '#'를 상자 하나로(부딪힘 상자 수를 줄인다).
        for z in 0..h {
            let mut x = 0;
            while x < w {
                if lv.cell(x, z) == b'#' {
                    let x0 = x;
                    while x < w && lv.cell(x, z) == b'#' {
                        x += 1;
                    }
                    let a = lv.at(x0, z);
                    let b = lv.at(x - 1, z);
                    lv.walls.push(Aabb {
                        min: vec3(a.x - CELL * 0.5, 0.0, a.z - CELL * 0.5),
                        max: vec3(b.x + CELL * 0.5, WALL_H, b.z + CELL * 0.5),
                    });
                } else {
                    x += 1;
                }
            }
        }
        if !gate_cells.is_empty() {
            let min_x = gate_cells.iter().map(|p| p.x).fold(f32::MAX, f32::min) - CELL * 0.5;
            let max_x = gate_cells.iter().map(|p| p.x).fold(f32::MIN, f32::max) + CELL * 0.5;
            let z = gate_cells[0].z;
            lv.gate = Some((
                Aabb {
                    min: vec3(min_x, 0.0, z - 0.4),
                    max: vec3(max_x, 4.5, z + 0.4),
                },
                vec3((min_x + max_x) * 0.5, 0.0, z),
            ));
        }
        lv
    }

    /// 높이 격자를 얹는다(0035·0043): 지도와 같은 크기의 글자 격자. 한 글자 = 0.5m 단위 높이 — 36진수 `v`(`0`~`9`, `a`~`z`)
    /// → (v − 4) × 0.5m(`4` = 바닥 0, `5` = +0.5, `6` = +1, `8` = +2, `c` = +4, `0` = −2). `/` = 경사로(높이는 이웃이 정한다),
    /// `.` = 기본.
    pub fn with_heights(mut self, grid: &str) -> Level {
        let rows: Vec<&str> = grid.lines().filter(|l| !l.trim().is_empty()).collect();
        for (z, row) in rows.iter().enumerate().take(self.h) {
            for (x, c) in row.chars().enumerate().take(self.w) {
                if c == '/' {
                    self.ramps[z * self.w + x] = true;
                } else if let Some(v) = c.to_digit(36) {
                    self.heights[z * self.w + x] = Some((v as f32 - 4.0) * 0.5);
                }
            }
        }
        self
    }

    /// 소품·불빛·시작 자리를 땅 높이에 올려놓는다(0043 — 높이 격자를 얹은 뒤 한 번).
    pub fn settle(mut self) -> Level {
        let g = self.ground();
        for p in &mut self.props {
            p.pos.y = g.height(p.pos);
        }
        for l in &mut self.lights {
            l.pos.y += g.height(l.pos);
        }
        self.player_start.y = g.height(self.player_start);
        for m in &mut self.marks {
            m.1.y = g.height(m.1);
        }
        // 문 가운데(갈 곳의 빛기둥·고리 자리, 0083)도 땅 높이에. 문 상자도 같이 올린다(0091 — 상자만 0에 남아 있으면 높은 땅의 문은
        // 위 한 자락이 상자 밖이라 시야 가림 판정이 놓쳤고, 막는 상자도 그만큼 낮았다).
        if let Some((b, c)) = &mut self.gate {
            c.y = g.height(*c);
            b.min.y += c.y;
            b.max.y += c.y;
        }
        self
    }

    /// 걸을 수 있는 칸인가.
    pub fn walkable(&self, x: usize, z: usize) -> bool {
        self.cell(x, z) == b'.'
    }

    /// 떼 마당의 자리(0244): 걸을 수 있는 칸마다 "벽·턱까지의 거리"를 재서, 비킬 자리(`avoid` — 자리·거리)와 `ok`가 거르는
    /// 곳을 빼고 가장 넓은 칸을 가운데로 고른다. 그 둘레 `HORDE_RADIUS` 칸 안에서 가운데와 같은 높이의 칸들에 가까운 차례로
    /// 칸마다 넷(±0.5m)씩 `n`개까지. 같은 지도 = 같은 자리. 넓은 곳이 없으면 빈 목록.
    pub fn horde_spots(
        &self,
        n: usize,
        avoid: &[(Vec3, f32)],
        ok: impl Fn(Vec3) -> bool,
    ) -> Vec<Vec3> {
        let (w, h) = (self.w as i32, self.h as i32);
        let height = |x: i32, z: i32| {
            self.heights
                .get((z * w + x) as usize)
                .copied()
                .flatten()
                .unwrap_or(0.0)
        };
        let flat = |x: i32, z: i32| {
            x >= 0
                && z >= 0
                && x < w
                && z < h
                && self.walkable(x as usize, z as usize)
                && !self
                    .ramps
                    .get((z * w + x) as usize)
                    .copied()
                    .unwrap_or(false)
        };
        // 벽·턱(높이 차 0.3m 넘게)까지의 칸 거리 — 여러 곳에서 퍼지는 BFS(8방향).
        let mut dist = vec![i32::MAX; (w * h) as usize];
        let mut q = std::collections::VecDeque::new();
        for z in 0..h {
            for x in 0..w {
                let edge = !flat(x, z)
                    || [(-1, 0), (1, 0), (0, -1), (0, 1)].iter().any(|(dx, dz)| {
                        !flat(x + dx, z + dz) || (height(x + dx, z + dz) - height(x, z)).abs() > 0.3
                    });
                if edge {
                    dist[(z * w + x) as usize] = 0;
                    q.push_back((x, z));
                }
            }
        }
        while let Some((x, z)) = q.pop_front() {
            let d = dist[(z * w + x) as usize] + 1;
            for dz in -1..=1 {
                for dx in -1..=1 {
                    let (nx, nz) = (x + dx, z + dz);
                    if nx < 0 || nz < 0 || nx >= w || nz >= h {
                        continue;
                    }
                    let k = (nz * w + nx) as usize;
                    if dist[k] > d {
                        dist[k] = d;
                        q.push_back((nx, nz));
                    }
                }
            }
        }
        let clear = |p: Vec3| {
            avoid
                .iter()
                .all(|(a, r)| vec2(p.x - a.x, p.z - a.z).length() >= *r)
                && ok(p)
        };
        let mut best: Option<(i32, i32, i32)> = None;
        for z in 0..h {
            for x in 0..w {
                let d = dist[(z * w + x) as usize];
                if !flat(x, z) || d < 2 || !clear(self.at(x as usize, z as usize)) {
                    continue;
                }
                if best.is_none_or(|(bd, _, _)| d > bd) {
                    best = Some((d, x, z));
                }
            }
        }
        let Some((_, cx, cz)) = best else {
            return Vec::new();
        };
        let h0 = height(cx, cz);
        let mut cells: Vec<(i32, i32)> = Vec::new();
        for dz in -HORDE_RADIUS..=HORDE_RADIUS {
            for dx in -HORDE_RADIUS..=HORDE_RADIUS {
                let (x, z) = (cx + dx, cz + dz);
                if dx * dx + dz * dz <= HORDE_RADIUS * HORDE_RADIUS
                    && flat(x, z)
                    && (height(x, z) - h0).abs() <= 0.3
                    && clear(self.at(x as usize, z as usize))
                {
                    cells.push((x, z));
                }
            }
        }
        cells.sort_by_key(|(x, z)| ((x - cx).pow(2) + (z - cz).pow(2), *z, *x));
        let mut out = Vec::with_capacity(n);
        for (x, z) in cells {
            let c = self.at(x as usize, z as usize);
            for o in [
                vec3(-0.5, 0.0, -0.5),
                vec3(0.5, 0.0, 0.5),
                vec3(0.5, 0.0, -0.5),
                vec3(-0.5, 0.0, 0.5),
            ] {
                if out.len() == n {
                    return out;
                }
                out.push(c + o);
            }
        }
        out
    }

    /// 월드 자리 → 칸(`at`의 거꾸로). 지도 밖이면 없음.
    pub fn cell_of(&self, p: Vec3) -> Option<(usize, usize)> {
        let fx = (p.x / CELL + self.w as f32 * 0.5).floor();
        let fz = (p.z / CELL + self.h as f32 * 0.5).floor();
        (fx >= 0.0 && fz >= 0.0 && (fx as usize) < self.w && (fz as usize) < self.h)
            .then_some((fx as usize, fz as usize))
    }

    /// 그 자리가 걸을 수 있는 땅인가.
    pub fn on_floor(&self, p: Vec3) -> bool {
        self.cell_of(p).is_some_and(|(x, z)| self.walkable(x, z))
    }

    /// 떨굼을 놓을 자리(0226 — 줍지 못할 자리에 떨어지지 않게): `to`가 걸을 수 있는 땅이면 그대로, 아니면 `from`(쓰러진 자리·
    /// 방랑자)으로 곧게 물러나며 처음 닿는 땅. 그 줄에 땅이 없으면(`from`도 벽 속 — 밀려 들어간 몸) `to` 둘레에서 가장 가까운
    /// 땅 칸의 가운데. 지도의 모든 땅은 이어져 있다(`every_floor_is_reachable…` 시험) — 땅이면 주울 수 있다. 높이는 부른 쪽이.
    pub fn floor_toward(&self, from: Vec3, to: Vec3) -> Vec3 {
        const STEPS: usize = 12;
        for k in 0..=STEPS {
            let p = to.lerp(from, k as f32 / STEPS as f32);
            if self.on_floor(p) {
                return p;
            }
        }
        let (cx, cz) = self.cell_of(to).unwrap_or((self.w / 2, self.h / 2));
        let mut best: Option<(f32, Vec3)> = None;
        for r in 1..=6usize {
            for z in cz.saturating_sub(r)..=(cz + r).min(self.h.saturating_sub(1)) {
                for x in cx.saturating_sub(r)..=(cx + r).min(self.w.saturating_sub(1)) {
                    if !self.walkable(x, z) {
                        continue;
                    }
                    let c = self.at(x, z);
                    let d = vec2(c.x - to.x, c.z - to.z).length();
                    if best.is_none_or(|(b, _)| d < b) {
                        best = Some((d, c));
                    }
                }
            }
            if let Some((_, c)) = best {
                return vec3(c.x, to.y, c.z);
            }
        }
        to
    }

    /// 걸을 수 있는 땅(0043): 바닥 칸은 높이(없으면 0), 경사로 칸은 같은 줄(가로 또는 세로)로 이어진 경사로 묶음의
    /// 양 끝 이웃 높이를 곧게 잇는다(높이 차가 큰 쪽 축). 벽·바깥은 `Solid`.
    pub fn ground(&self) -> Ground {
        let flat = |x: usize, z: usize| -> Option<f32> {
            (self.walkable(x, z) && !self.ramps[z * self.w + x])
                .then(|| self.heights[z * self.w + x].unwrap_or(0.0))
        };
        let mut cells = Vec::with_capacity(self.w * self.h);
        for z in 0..self.h {
            for x in 0..self.w {
                if !self.walkable(x, z) {
                    cells.push(Cell::Solid);
                    continue;
                }
                if !self.ramps[z * self.w + x] {
                    cells.push(Cell::Flat(flat(x, z).unwrap_or(0.0)));
                    continue;
                }
                // 경사로 묶음의 양 끝 이웃(축마다): (낮은 쪽 끝 높이, 높은 쪽 끝 높이, 이 칸의 자리, 묶음 길이).
                let run = |x_axis: bool| -> Option<(f32, f32, usize, usize)> {
                    let (len, pos) = if x_axis { (self.w, x) } else { (self.h, z) };
                    let idx = |i: usize| if x_axis { (i, z) } else { (x, i) };
                    let is_ramp = |i: usize| {
                        let (cx, cz) = idx(i);
                        self.walkable(cx, cz) && self.ramps[cz * self.w + cx]
                    };
                    let mut a = pos;
                    while a > 0 && is_ramp(a - 1) {
                        a -= 1;
                    }
                    let mut b = pos;
                    while b + 1 < len && is_ramp(b + 1) {
                        b += 1;
                    }
                    let lo = (a > 0)
                        .then(|| idx(a - 1))
                        .and_then(|(cx, cz)| flat(cx, cz))?;
                    let hi = (b + 1 < len)
                        .then(|| idx(b + 1))
                        .and_then(|(cx, cz)| flat(cx, cz))?;
                    Some((lo, hi, pos - a, b - a + 1))
                };
                let pick = match (run(true), run(false)) {
                    (Some(a), Some(b)) => Some(if (a.1 - a.0).abs() >= (b.1 - b.0).abs() {
                        (true, a)
                    } else {
                        (false, b)
                    }),
                    (Some(a), None) => Some((true, a)),
                    (None, Some(b)) => Some((false, b)),
                    (None, None) => None,
                };
                cells.push(match pick {
                    Some((x_axis, (lo, hi, i, n))) => Cell::Ramp {
                        x_axis,
                        from: lo + (hi - lo) * i as f32 / n as f32,
                        to: lo + (hi - lo) * (i + 1) as f32 / n as f32,
                    },
                    None => Cell::Flat(0.0),
                });
            }
        }
        Ground::new(self.w, self.h, CELL, cells)
    }

    /// 칸의 배경 높이(0035).
    pub fn height(&self, x: usize, z: usize) -> Option<f32> {
        if x < self.w && z < self.h {
            self.heights[z * self.w + x]
        } else {
            None
        }
    }

    /// 부딪힘 상자: 벽 + 소품(+ 닫힌 문은 따로).
    /// 무너진 아치의 자리(0265 — 지도의 `ARCH_GLYPH`, 땅 높이).
    pub fn arch(&self) -> Option<Vec3> {
        self.props
            .iter()
            .find(|p| p.kind == PropKind::Arch)
            .map(|p| p.pos)
    }

    pub fn obstacles(&self) -> Vec<Aabb> {
        let mut v = self.walls.clone();
        for p in &self.props {
            if p.kind == PropKind::Arch {
                for sx in [-1.0, 1.0] {
                    let x = p.pos.x + sx * ARCH_JAMB_X;
                    v.push(Aabb {
                        min: vec3(x - ARCH_JAMB_HALF.x, 0.0, p.pos.z - ARCH_JAMB_HALF.y),
                        max: vec3(
                            x + ARCH_JAMB_HALF.x,
                            ARCH_JAMB_H,
                            p.pos.z + ARCH_JAMB_HALF.y,
                        ),
                    });
                }
            }
            let h = p.kind.half();
            if h != Vec2::ZERO {
                v.push(Aabb {
                    min: vec3(p.pos.x - h.x, 0.0, p.pos.z - h.y),
                    max: vec3(p.pos.x + h.x, p.kind.height(), p.pos.z + h.y),
                });
            }
        }
        v
    }
}
