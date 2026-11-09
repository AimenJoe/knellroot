//! 스테이지(0019): 지도(글자 격자) → 레벨(벽·소품·불빛·스폰·문) + 진행 대본(구역·물결·문·보스·핵). 새 스테이지 = 파일 하나
//! (`StageDef` 한 개). 진행은 월드 밖에서 월드를 지켜보며 적을 세우고 문을 여닫는다 — 월드는 스테이지를 모른다(경계 그대로).
//! 앱은 `StageRun::tick`의 결과(`StageOut`)로 장면만 바꾼다(흐름은 `app::flow`의 순수 함수).
pub mod sanctuary;
pub mod village;

use macroquad::prelude::*;

use crate::ground::{Cell, Ground};
use crate::i18n::Tr;
use crate::species::Kind;
use crate::world::{Aabb, EState, PState, World};

/// 지도 한 칸(m).
pub const CELL: f32 = 2.0;
/// 벽 높이(m, 굽기에서 칸마다 조금씩 다르게 깎는다).
pub const WALL_H: f32 = 3.0;

/// 소품 종류 — 모델 이름(`assets/models/<이름>.glb`)과 부딪힘 상자.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PropKind {
    Pillar,
    Root,
    Brazier,
    Rubble,
    WallPiece,
    Bell,
}

impl PropKind {
    pub fn model(self) -> &'static str {
        match self {
            PropKind::Pillar => "pillar_broken",
            PropKind::Root => "root_clump",
            PropKind::Brazier => "brazier",
            PropKind::Rubble => "rubble",
            PropKind::WallPiece => "wall_segment",
            PropKind::Bell => "bell",
        }
    }

    /// 부딪힘 상자 반폭(XZ, m) — 0이면 지나간다.
    pub fn half(self) -> Vec2 {
        match self {
            PropKind::Pillar => vec2(0.6, 0.6),
            PropKind::Root => vec2(0.5, 0.5),
            PropKind::Brazier => vec2(0.35, 0.35),
            PropKind::Rubble => Vec2::ZERO,
            PropKind::WallPiece => vec2(1.0, 0.4),
            PropKind::Bell => vec2(0.8, 0.8),
        }
    }

    pub fn height(self) -> f32 {
        match self {
            PropKind::Pillar => 3.0,
            PropKind::Root => 1.0,
            PropKind::Brazier => 1.0,
            PropKind::Rubble => 0.6,
            PropKind::WallPiece => 3.0,
            PropKind::Bell => 2.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Prop {
    pub kind: PropKind,
    pub pos: Vec3,
    pub yaw: f32,
}

/// 불빛(화로) — 렌더가 정점에 더한다.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Light {
    pub pos: Vec3,
    pub color: Color,
    pub radius: f32,
    /// 천장 틈 빛줄기(0119 → 0124): 둥글게 퍼지지 않고 위에서 내리꽂는 기둥 — `radius`는 기둥의 가로 반지름,
    /// 그 안의 것은 높이와 상관없이 위를 보는 면부터 밝다(서면 몸이 비친다).
    pub shaft: bool,
}

/// 지도에서 읽은 레벨(정적인 것).
pub struct Level {
    pub w: usize,
    pub h: usize,
    /// 칸: '#' 벽, '.' 바닥, ' ' 바깥.
    pub cells: Vec<u8>,
    pub walls: Vec<Aabb>,
    pub props: Vec<Prop>,
    pub lights: Vec<Light>,
    /// 표지 글자 → 그 칸들의 가운데.
    pub marks: Vec<(char, Vec3)>,
    pub player_start: Vec3,
    /// 문(닫히면 막는 상자)과 그 가운데.
    pub gate: Option<(Aabb, Vec3)>,
    /// 높낮이(0035 그림 → 0043 걸을 수 있게): 칸마다 높이(m). 바닥 칸은 땅 높이, 바깥 칸은 바위 단(+)·꺼진 땅(-),
    /// 벽 칸은 벽 꼭대기. None = 기본(바닥 0·벽 무작위·바깥 허공).
    pub heights: Vec<Option<f32>>,
    /// 경사로 칸(0043): 이웃 높이를 이어 준다(`ground`).
    pub ramps: Vec<bool>,
}

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
    pub fn obstacles(&self) -> Vec<Aabb> {
        let mut v = self.walls.clone();
        for p in &self.props {
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

/// 스테이지 정의(데이터).
pub struct StageDef {
    pub name: Tr,
    pub map: &'static str,
    /// 배경 높낮이 격자(0035, 그림만 — `Level::with_heights`). 없으면 평평.
    pub heights: Option<&'static str>,
    /// 구역: (이름, 이 행보다 위(작은 z)로 들어가면).
    pub zones: &'static [(Tr, usize)],
    /// 물결: (스폰 표지, 종류) 묶음 — 앞 물결이 이만큼 남으면 다음 물결.
    pub waves: &'static [&'static [(char, Kind)]],
    /// 보스 표지.
    pub boss: (char, Kind),
    /// 시작 이야기 두 줄.
    pub intro: (Tr, Tr),
    pub obj_waves: Tr,
    pub obj_gate: Tr,
    pub obj_core: Tr,
    pub boss_wakes: Tr,
    /// 상자(지도 `C`, 위에서부터 차례)마다 든 것(0064 M3 → 0117: 장신구 또는 무기). 이미 가졌으면 잔불 `CHEST_EMBERS`.
    pub chest_loot: &'static [Loot],
    /// 문지기를 처음 이겼을 때 주는 장신구.
    pub boss_charm: Option<crate::charms::Id>,
    /// 새긴 돌(0129 C4, 지도 `N`, 위에서부터 차례)마다 새긴 글 — 곁에서 상호작용하면 화면 아래 글 판(평범한 삶의 흔적, lore 0116).
    pub stones: &'static [Tr],
    /// 숨은 물건 기록 이름의 앞(0105): `<앞>.<종류>.<그 종류의 몇째>` — 스테이지가 늘어도 섞이지 않게.
    pub find_prefix: &'static str,
    /// 지도에 서 있는 적(0113): (지도 표지, 종류) — 제자리에서 제 일을 하다 알아차리면 쫓는다. 화로에서 쉬면 다시 선다.
    /// 물결(`waves`)은 시험·PoC·특별한 기믹 스테이지에만 — 보통 스테이지는 넓은 지역 + 밀집도(사장 2026-11-08).
    pub spawns: &'static [(char, Kind)],
    /// 청소자 자리(0120 — 0116 ECO-10 "같은 종, 다른 역할"): 이 표지에 선 적은 보고도 덤비지 않고 먹는다(맞거나 동료가 깨면 싸운다).
    pub docile: &'static [char],
    /// 엘리트(0112 → 0113): (지도 표지, 종류, 성질) — 서 있는 적과 같은 꼴, 단 그 판에 한 번 쓰러뜨리면 쉬어도 다시 서지 않는다.
    /// 기록 이름은 `<find_prefix>.elite.<이 표의 몇째>`(처음 쓰러뜨린 것만 덤).
    pub elites: &'static [(char, Kind, crate::enemy::Affix)],
    /// 처음(문이 닫혀 있을 때)의 목표 글 — 서 있는 적 스테이지는 문을 여는 장치를 찾게.
    pub obj_explore: Tr,
    /// 장치를 당겼을 때의 카드(성소 "어딘가에서 문이 열렸다", 마을 "사다리가 내려왔다").
    pub card_lever: Tr,
    /// 지하 몇 m(0118, 연출 표 — 지층 단면 전환·카드가 같이 쓴다). 차례대로 깊어진다(시험).
    pub depth_m: u32,
    /// 구역 등급(0112·0116, 1부터): 잡몹의 체력·자세·잔불(등급 3부터 피해 2). 깊을수록 높다(시험).
    pub tier: u8,
    /// 지역 환경(0118 T2): 원경·달빛·안개·떠다니는 것·식생 — `backdrop::SURFACE`·`UNDERGROUND`.
    pub env: &'static crate::backdrop::Env,
}

/// 판의 차례(0118, v0.5.0 T1): 성소 → 떠난 마을. 지도 `D`(내려가는 길, 핵을 거둔 뒤)로 다음, `U`(오르는 길)로 앞.
pub static STAGES: [&StageDef; 2] = [&sanctuary::DEF, &village::DEF];
/// 내려가는 길·오르는 길 표지(0118)와 밟는 거리(m). 표지에서 이만큼 벗어난 뒤에야 다시 밟을 수 있다.
pub const DOWN_GLYPH: char = 'D';
pub const UP_GLYPH: char = 'U';
pub const PASSAGE_REACH: f32 = 1.2;
/// 천장 틈 빛줄기 표지(0118 T2)와 기둥의 가로 반지름(m — 바닥 웅덩이와 같게).
pub const SHAFT_GLYPH: char = 'S';
pub const SHAFT_R: f32 = 1.7;
/// 상자 기록 번호(0118): 지역마다 `지역 × 이것 + 상자 번호`(성소는 그대로 — 옛 기록이 이어진다).
pub const CHESTS_PER_STAGE: u32 = 1000;
const PASSAGE_ARM: f32 = 2.6;

/// 문을 여는 장치(0113): 지도 `V`. 곁에서 상호작용 = 당기기 — 문이 열리고 기록에 남는다(지름길, 소울 고리).
pub const LEVER_GLYPH: char = 'V';
pub const LEVER_REACH: f32 = 1.8;

/// 숨은 물건(0105, v0.4.0 G3 — 0102 6절): 핵 고르기의 등잔·한 모금이 탐험의 보상으로. 지도 글자 `F` 등잔 +1회 · `H` 한 모금 +1.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Find {
    Lamp,
    Sip,
    /// 종소리 조각(0106, v0.4.0 G2): 기술 나무 점수 +1. 이름이 출처를 정하지 않는다(lore — 종에서 떨어진 것인지 말하지 않는다).
    Shard,
}

impl Find {
    pub const ALL: [Find; 3] = [Find::Lamp, Find::Sip, Find::Shard];

    pub fn glyph(self) -> char {
        match self {
            Find::Lamp => 'F',
            Find::Sip => 'H',
            Find::Shard => 'K',
        }
    }

    /// 기록 이름 가운데 칸(바꾸지 않는다).
    pub fn key(self) -> &'static str {
        match self {
            Find::Lamp => "lamp",
            Find::Sip => "sip",
            Find::Shard => "shard",
        }
    }
}

/// 숨은 물건 하나의 기록 이름: `sanctuary.lamp.0`.
pub fn find_id(prefix: &str, kind: Find, nth: usize) -> String {
    format!("{prefix}.{}.{nth}", kind.key())
}

/// 엘리트 기록 이름(0112): `<앞>.elite.<표의 몇째>`.
pub fn elite_id(prefix: &str, nth: usize) -> String {
    format!("{prefix}.elite.{nth}")
}

/// 숨은 물건 곁(m) — 상자와 같다.
pub const FIND_REACH: f32 = 1.8;

/// 상자 곁(m)과, 든 장신구를 이미 가졌을 때 대신 주는 잔불.
pub const CHEST_REACH: f32 = 1.8;
/// 새긴 돌을 읽는 거리(m, 0129).
const STONE_REACH: f32 = 2.2;
pub const CHEST_EMBERS: u32 = 10;

/// 상자에 든 것(0117): 장신구 또는 무기 — 무기는 얻어서 쓴다(검·맨손만 처음부터).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Loot {
    Charm(crate::charms::Id),
    Weapon(crate::weapons::Id),
}
/// 상자 부딪힘 상자 반폭(가로·세로 m).
const CHEST_HALF: Vec3 = Vec3::new(0.45, 0.5, 0.3);

/// 진행 단계.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Phase {
    /// 입구(아직 뜰에 안 들어감).
    Entrance,
    /// 뜰: n번째 물결.
    Waves(usize),
    /// 문이 열렸다(성소로 들어가기 전).
    GateOpen,
    /// 성소에 들어와 문이 닫히고 문지기가 깨어나는 중(초).
    Waking(f32),
    Duel,
    /// 문지기가 쓰러지고 핵이 남았다.
    Core,
    /// 핵을 거뒀고 아래 지역이 있다(0118): 바닥 길(`D`)이 열려 있다 — 이 판에서는 문지기가 다시 서지 않는다.
    Down,
    Cleared,
}

/// 진행 결과(앱이 장면을 바꾸는 데만 쓴다).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StageOut {
    Playing,
    /// 마지막이 아닌 지역의 핵을 거뒀다(0118) — 앱은 핵 보상만, 판은 이어진다(바닥 길이 열린다).
    CoreTaken,
    /// 열린 바닥 길·오르는 길을 밟았다(0118) — 앱이 지층 단면 전환을 열고 지역을 바꾼다.
    Descend,
    Ascend,
    Cleared,
    Defeated,
    /// 쓰러졌지만 마지막으로 쉰 쉼터에서 다시 일어섰다(0032) — 앱은 카메라만 그 자리로.
    Respawned,
}

/// 쉼터 곁(m): 이 안이면 "쉬기"가 뜬다.
pub const REST_REACH: f32 = 2.4;
/// 이 안에 깨어 있는 적이 있으면 쉴 수 없다(m).
pub const REST_SAFE: f32 = 10.0;

/// 문지기가 깨어나는 데 걸리는 시간(초) — 이름 카드·`wake` 동작.
pub const WAKE_TIME: f32 = 2.0;
/// 다음 물결: 앞 물결이 이만큼 이하로 남으면.
const WAVE_LEFT: usize = 1;
/// 핵을 거두는 거리(m).
const CORE_REACH: f32 = 1.4;
/// 처음 이긴 문지기의 핵(lore CANON — 0101·0105): 이 안에 들어서면 핵이 스스로 방랑자 쪽으로 다가온다(m, m/s).
const CORE_DRAW: f32 = 4.0;
const CORE_DRAW_SPEED: f32 = 2.2;
/// 이름 카드가 떠 있는 시간(초).
pub const CARD_TIME: f32 = 3.0;

pub struct StageRun {
    pub def: &'static StageDef,
    pub level: Level,
    pub phase: Phase,
    /// 지금 구역(이름 카드를 띄울 때 바뀐다)과 카드 남은 초.
    pub zone: usize,
    pub card: Option<(Tr, f32)>,
    pub objective: Option<Tr>,
    /// 시작 이야기 남은 초.
    pub intro: f32,
    /// 이번 물결의 적 번호들.
    wave_ids: Vec<usize>,
    pub boss: Option<usize>,
    pub gate_open: f32,
    /// 문이 열려 있나(부딪힘 상자).
    pub gate_closed: bool,
    pub core: Option<Vec3>,
    pub time: f32,
    first_card: bool,
    /// 쉼터 화로들(지도 `R`).
    pub rests: Vec<Vec3>,
    /// 마지막으로 쉰 쉼터와, 쓰러지면 되돌아갈 진행 단계(쉰 때의 단계 — 물결 전이면 입구, 물결을 끝냈으면 문 앞).
    pub rest_at: Option<(usize, Phase)>,
    /// 곁에 있는 쉼터(HUD·터치 버튼), 지금 쉴 수 있나(가까운 깨어 있는 적이 없다).
    pub near_rest: Option<usize>,
    pub rest_safe: bool,
    /// 마지막으로 쉰 뒤 지난 초(렌더의 퍼지는 빛).
    pub since_rest: f32,
    /// 장신구 상자(0064 M3): 자리, 열렸나(기록에서 — 한 번 연 상자는 판을 넘어 비어 있다), 곁의 닫힌 상자, 이번 틱에 연 상자(앱이 가져간다).
    pub chests: Vec<Vec3>,
    pub chest_open: Vec<bool>,
    pub near_chest: Option<usize>,
    opened: Option<usize>,
    /// 새긴 돌(0129): 자리, 곁의 돌, 이번 틱에 읽은 돌(앱이 가져간다).
    pub stones: Vec<Vec3>,
    pub near_stone: Option<usize>,
    read: Option<usize>,
    /// 이번 틱에 쉬었다(0064 M4 — 앱이 화로 가게를 연다), 잔불로 살 것이 있다(앱이 정한다 — 쉼터가 빛난다).
    rested: bool,
    pub shop_ready: bool,
    /// 숨은 물건(0105): 자리·종류·기록 이름, 주웠나(기록에서), 곁의 것, 이번 틱에 주운 것(앱이 가져간다).
    pub finds: Vec<(Vec3, Find, String)>,
    pub found: Vec<bool>,
    pub near_find: Option<usize>,
    picked: Option<usize>,
    /// 엘리트(0112): 지금 서 있는 (적 번호, 기록 이름), 기록에서 이미 쓰러뜨린 것, 이번 틱에 처음 쓰러뜨린 것(앱이 가져간다).
    elite_ids: Vec<(usize, String)>,
    /// 이 판에 쓰러뜨린 엘리트(표의 몇째, 0113) — 쉬어도 다시 서지 않는다.
    elites_down: Vec<usize>,
    /// 문 장치(0113): 자리, 곁인가, 문이 열려 있나(기록 — 한 번 열면 판을 넘어 열려 있다), 이번 틱에 처음 열었다(앱이 가져간다).
    pub lever: Option<Vec3>,
    pub near_lever: bool,
    pub gate_opened: bool,
    gate_new: bool,
    beaten: Vec<String>,
    elite_new: Vec<String>,
    /// 이 문지기를 처음 이기는 판인가(앱이 기록에서 — 핵이 스스로 다가온다, lore CANON 0101).
    pub first_core: bool,
    /// 내려가는 길·오르는 길 자리(0118, 지도 `D`·`U`), 이 판에 이 지역 문지기를 이겼나(바닥 길이 열린 채, 문지기 없음),
    /// 표지에서 한 번 벗어났나(도착하자마자 되돌아가지 않게).
    pub down_at: Option<Vec3>,
    pub up_at: Option<Vec3>,
    pub down_open: bool,
    passage_armed: bool,
    /// 남은 주민(0122): (표 번호, 자리), 지금 바라보는 쪽(가까이 오면 돌아본다), 재방문으로 바뀌었나(앱이 기록에서),
    /// 이 판에 봤나(처음 본 틱에 앱이 기록에), 곁의 주민, 이번 틱에 말 건 주민(앱이 가져간다 — 말풍선).
    pub residents: Vec<(usize, Vec3)>,
    pub resident_face: Vec<f32>,
    pub resident_changed: Vec<bool>,
    resident_seen: Vec<bool>,
    resident_seen_new: Vec<usize>,
    pub near_resident: Option<usize>,
    talked: Option<usize>,
}

/// 시작 이야기 길이(초).
pub const INTRO_TIME: f32 = 6.0;

impl StageRun {
    /// 레벨을 세우고 월드를 만든다.
    pub fn start(def: &'static StageDef) -> (StageRun, World) {
        let mut level = Level::parse(def.map);
        if let Some(g) = def.heights {
            level = level.with_heights(g).settle();
        }
        let mut w = World::new("stage", level.half_extent());
        w.obstacles = level.obstacles();
        w.ground = Some(level.ground());
        if let Some((g, _)) = level.gate {
            w.obstacles.push(g);
        }
        w.player.pos = level.player_start;
        w.player.prev = w.player.pos;
        w.player.facing = std::f32::consts::PI; // 위(북쪽, -z)를 본다
        let mut run = StageRun {
            def,
            level,
            phase: Phase::Entrance,
            zone: 0,
            card: None,
            objective: None,
            intro: INTRO_TIME,
            wave_ids: Vec::new(),
            boss: None,
            gate_open: 0.0,
            gate_closed: true,
            core: None,
            time: 0.0,
            first_card: false,
            rests: Vec::new(),
            rest_at: None,
            near_rest: None,
            rest_safe: true,
            since_rest: f32::MAX,
            chests: Vec::new(),
            chest_open: Vec::new(),
            near_chest: None,
            stones: Vec::new(),
            near_stone: None,
            read: None,
            opened: None,
            rested: false,
            shop_ready: false,
            finds: Vec::new(),
            found: Vec::new(),
            near_find: None,
            picked: None,
            elite_ids: Vec::new(),
            elites_down: Vec::new(),
            lever: None,
            near_lever: false,
            gate_opened: false,
            gate_new: false,
            beaten: Vec::new(),
            elite_new: Vec::new(),
            first_core: false,
            down_at: None,
            up_at: None,
            down_open: false,
            passage_armed: true,
            residents: Vec::new(),
            resident_face: Vec::new(),
            resident_changed: Vec::new(),
            resident_seen: Vec::new(),
            resident_seen_new: Vec::new(),
            near_resident: None,
            talked: None,
        };
        // 주민(0122): 표지마다 — 바라보는 쪽은 자리마다 정해진 값, 몸은 방랑자만 밀어낸다.
        for (c, p) in run.level.marks.clone() {
            if let Some(i) = crate::residents::by_mark(c) {
                run.residents.push((i, p));
                run.resident_face
                    .push(((p.x * 3.1 + p.z * 1.7).sin() * 2.0).rem_euclid(std::f32::consts::TAU));
                w.bodies.push((p, crate::residents::BODY_R));
            }
        }
        run.resident_changed = vec![false; run.residents.len()];
        run.resident_seen = vec![false; run.residents.len()];
        run.down_at = run.level.marks_of(DOWN_GLYPH).next();
        run.up_at = run.level.marks_of(UP_GLYPH).next();
        run.rests = run.level.marks_of('R').collect();
        run.chests = run.level.marks_of('C').collect();
        run.stones = run.level.marks_of('N').collect();
        run.chest_open = vec![false; run.chests.len()];
        for kind in Find::ALL {
            let at: Vec<Vec3> = run.level.marks_of(kind.glyph()).collect();
            for (n, p) in at.into_iter().enumerate() {
                run.finds.push((p, kind, find_id(def.find_prefix, kind, n)));
            }
        }
        run.found = vec![false; run.finds.len()];
        for c in &run.chests {
            w.obstacles.push(crate::world::Aabb {
                min: *c - vec3(CHEST_HALF.x, 0.0, CHEST_HALF.z),
                max: *c + vec3(CHEST_HALF.x, CHEST_HALF.y * 2.0, CHEST_HALF.z),
            });
        }
        // 보스는 처음부터 잠든 채 성소에 있다(보인다 — 기대감).
        if let Some(p) = run.level.marks_of(def.boss.0).next() {
            let i = w.spawn(def.boss.1, p);
            w.enemies[i].state = EState::Dormant;
            run.boss = Some(i);
        }
        // 서 있는 적(0113): 처음부터 제자리에. 문 장치 자리.
        run.lever = run.level.marks_of(LEVER_GLYPH).next();
        if def.waves.is_empty() {
            run.populate(&mut w);
            run.objective = Some(def.obj_explore);
        }
        // 문 장치도 물결도 없는 지역(0118)은 처음부터 문이 열린 단계.
        if run.lever.is_none() && def.waves.is_empty() {
            run.phase = Phase::GateOpen;
            run.set_gate(&mut w, false);
            run.objective = Some(def.obj_gate);
        }
        (run, w)
    }

    /// 위 지역에서 내려와 닿았다(0118): 시작 이야기 없이 시작 자리(`P`)에, 이름 카드.
    pub fn arrive_from_above(&mut self, w: &mut World) {
        self.intro = 0.0;
        self.first_card = true;
        self.passage_armed = false;
        let p = self.level.player_start;
        w.player.pos = p;
        w.player.prev = p;
        w.player.facing = std::f32::consts::PI;
        self.zone = self.zone_at(p);
        self.card = Some((self.def.name, CARD_TIME));
    }

    /// 아래 지역에서 올라왔다(0118): 이 판에 이긴 문지기는 없고 바닥 길은 열린 채, 바닥 길 곁(남쪽)에 선다.
    pub fn arrive_from_below(&mut self, w: &mut World) {
        self.intro = 0.0;
        self.first_card = true;
        self.open_down(w);
        self.passage_armed = false;
        if let Some(d) = self.down_at {
            let p = d + vec3(0.0, 0.0, PASSAGE_ARM - 0.4);
            w.player.pos = p;
            w.player.prev = p;
            w.player.facing = 0.0;
        }
        self.zone = self.zone_at(w.player.pos);
        self.card = Some((self.def.zones[self.zone].0, CARD_TIME));
    }

    /// 이 판에 이 지역 문지기를 이겼다(0118): 문지기를 지우고 문을 열고 바닥 길 단계로.
    pub fn open_down(&mut self, w: &mut World) {
        self.down_open = true;
        w.enemies.retain(|e| !e.boss);
        self.boss = None;
        self.core = None;
        self.phase = Phase::Down;
        self.objective = Some(|s| s.obj_descend);
        self.set_gate(w, false);
    }

    fn zone_at(&self, p: Vec3) -> usize {
        let row = self.level.row_of(p);
        let mut z = 0;
        for (i, (_, edge)) in self.def.zones.iter().enumerate() {
            if row < *edge as f32 {
                z = i;
            }
        }
        z
    }

    fn set_gate(&mut self, w: &mut World, closed: bool) {
        if closed == self.gate_closed {
            return;
        }
        self.gate_closed = closed;
        if let Some((g, _)) = self.level.gate {
            if closed {
                w.obstacles.push(g);
            } else {
                w.obstacles.retain(|o| *o != g);
            }
        }
    }

    fn spawn_wave(&mut self, w: &mut World, n: usize) {
        self.wave_ids.clear();
        self.elite_ids.clear();
        if let Some(list) = self.def.waves.get(n) {
            for (mark, kind) in list.iter() {
                let spots: Vec<Vec3> = self.level.marks_of(*mark).collect();
                for p in spots {
                    let i = w.spawn(*kind, p);
                    w.enemies[i].set_tier(self.def.tier);
                    self.wave_ids.push(i);
                }
            }
        }
    }

    /// 서 있는 적을 세운다(0113): 보스 말고 모두 지우고 표대로 제자리에 — 엘리트는 이 판에 쓰러뜨리지 않은 것만.
    /// 시작할 때와 화로에서 쉴 때(쓰러져 다시 설 때 포함, 소울 화톳불).
    fn populate(&mut self, w: &mut World) {
        if !self.def.waves.is_empty() {
            return;
        }
        w.enemies.retain(|e| e.boss);
        self.boss = (!w.enemies.is_empty()).then_some(0);
        w.lock = None;
        w.lingers.clear();
        w.shards.clear();
        self.elite_ids.clear();
        // 바라보는 쪽은 자리마다 정해진 값(자리로 섞는다) — 늘어선 적이 한 쪽만 보지 않게.
        let face = |p: Vec3| {
            let h = (p.x * 12.9898 + p.z * 78.233).sin() * 43758.547;
            (h - h.floor()) * std::f32::consts::TAU
        };
        for &(mark, kind) in self.def.spawns {
            let spots: Vec<Vec3> = self.level.marks_of(mark).collect();
            for p in spots {
                let i = w.spawn(kind, p);
                w.enemies[i].set_tier(self.def.tier);
                w.enemies[i].place(face(p));
                w.enemies[i].docile = self.def.docile.contains(&mark);
            }
        }
        for (k, &(mark, kind, affix)) in self.def.elites.iter().enumerate() {
            if self.elites_down.contains(&k) {
                continue;
            }
            if let Some(p) = self.level.marks_of(mark).next() {
                let i = w.spawn(kind, p);
                w.enemies[i].set_tier(self.def.tier);
                w.enemies[i].make_elite(affix);
                w.enemies[i].place(face(p));
                self.elite_ids.push((i, elite_id(self.def.find_prefix, k)));
            }
        }
    }

    /// 기록의 문(0113): 이미 열었으면 처음부터 열린 채(지름길).
    pub fn set_gate_opened(&mut self, w: &mut World, open: bool) {
        if !open || self.gate_opened {
            return;
        }
        self.gate_opened = true;
        if self.phase == Phase::Entrance {
            self.phase = Phase::GateOpen;
            self.objective = Some(self.def.obj_gate);
        }
        self.set_gate(w, false);
    }

    /// 이번 틱에 처음 문을 열었다(0113) — 앱이 한 번 가져가 기록에.
    pub fn take_gate_new(&mut self) -> bool {
        std::mem::take(&mut self.gate_new)
    }

    /// 쓰러진 엘리트(0112): 처음이면 잔불 `ELITE_FIRST_EMBERS`를 더 흩고 앱에 알린다(기록·알림).
    fn tick_elites(&mut self, w: &mut World) {
        let mut down = Vec::new();
        self.elite_ids.retain(|(i, id)| match w.enemies.get(*i) {
            Some(e) if e.alive() => true,
            Some(e) => {
                down.push((e.pos, id.clone()));
                false
            }
            None => false,
        });
        for (pos, id) in down {
            if let Some(k) =
                (0..self.def.elites.len()).find(|&k| elite_id(self.def.find_prefix, k) == id)
            {
                self.elites_down.push(k);
            }
            if self.beaten.contains(&id) {
                continue;
            }
            let seed = (self.time * 1000.0) as u32 ^ 0x5eed_e11e;
            w.drops.extend(crate::ember::scatter(
                pos,
                crate::tuning::ELITE_FIRST_EMBERS,
                seed,
            ));
            self.beaten.push(id.clone());
            self.elite_new.push(id);
        }
    }

    /// 이번 틱에 처음 쓰러뜨린 엘리트들(0112, 한 틱에 여럿 — 수류탄) — 앱이 한 번 가져가 기록·알림에.
    pub fn take_elites_new(&mut self) -> Vec<String> {
        std::mem::take(&mut self.elite_new)
    }

    /// 기록의 쓰러뜨린 엘리트들(처음 덤은 한 번만).
    pub fn set_beaten(&mut self, ids: &[String]) {
        self.beaten = ids.to_vec();
    }

    fn alive_in_wave(&self, w: &World) -> usize {
        self.wave_ids
            .iter()
            .filter(|&&i| w.enemies.get(i).is_some_and(|e| e.alive()))
            .count()
    }

    /// 한 틱(월드 틱 뒤). 구역 카드·물결·문·보스·핵을 진행한다.
    pub fn tick(&mut self, w: &mut World, dt: f32) -> StageOut {
        self.time += dt;
        self.tick_elites(w);
        self.intro = (self.intro - dt).max(0.0);
        if let Some((tr, t)) = self.card {
            self.card = (t > dt).then_some((tr, t - dt));
        }
        if self.gate_closed {
            self.gate_open = (self.gate_open - dt).max(0.0);
        } else {
            self.gate_open = (self.gate_open + dt).min(1.5);
        }
        self.since_rest = (self.since_rest + dt).min(f32::MAX);
        if w.player.hp <= 0 {
            // 쉼터에서 쉰 적이 있으면 거기서 다시(0032) — 없으면 끝.
            return match self.rest_at {
                Some((i, back)) => {
                    self.respawn(w, i, back);
                    StageOut::Respawned
                }
                None => StageOut::Defeated,
            };
        }
        // 쉼터(0032): 곁이면 "쉬기", 깨어 있는 적이 가까우면 막는다.
        let pp = w.player.pos;
        // 내려가는 길·오르는 길(0118): 표지에서 한 번 벗어나야 다시 밟힌다.
        let flat = |a: Vec3| vec2(a.x - pp.x, a.z - pp.z).length();
        let near = |at: Option<Vec3>| at.is_some_and(|a| flat(a) <= PASSAGE_REACH);
        if !self.passage_armed
            && [self.down_at, self.up_at]
                .iter()
                .flatten()
                .all(|&a| flat(a) >= PASSAGE_ARM)
        {
            self.passage_armed = true;
        }
        self.near_rest = self
            .rests
            .iter()
            .position(|r| vec2(r.x - pp.x, r.z - pp.z).length() <= REST_REACH);
        // 쉴 수 있나: 싸우는 중인 적이 가까우면 막는다 — 제자리에서 제 일을 하는 적(0113)은 세지 않는다.
        self.rest_safe = !w.enemies.iter().any(|e| {
            e.active()
                && !matches!(e.state, EState::Idle | EState::Return)
                && vec2(e.pos.x - pp.x, e.pos.z - pp.z).length() <= REST_SAFE
        });
        // 상자(0064 M3): 곁의 닫힌 상자는 적이 가까워도 연다(쉬기와 달리 몸이 묶이지 않는다).
        self.near_chest = self.chests.iter().enumerate().position(|(i, c)| {
            !self.chest_open[i] && vec2(c.x - pp.x, c.z - pp.z).length() <= CHEST_REACH
        });
        // 새긴 돌(0129): 곁이면 읽기(싸움 중에도 — 글 판은 멈추지 않는다).
        self.near_stone = self
            .stones
            .iter()
            .position(|s| vec2(s.x - pp.x, s.z - pp.z).length() <= STONE_REACH);
        // 숨은 물건(0105): 상자와 같은 꼴 — 곁이면 점프 버튼이 줍기.
        self.near_find = self.finds.iter().enumerate().position(|(i, (p, _, _))| {
            !self.found[i] && vec2(p.x - pp.x, p.z - pp.z).length() <= FIND_REACH
        });
        // 문 장치(0113): 문이 닫혀 있을 때만 곁이면 당기기.
        self.near_lever = !self.gate_opened
            && self
                .lever
                .is_some_and(|l| vec2(l.x - pp.x, l.z - pp.z).length() <= LEVER_REACH);
        // 주민(0122): 보면 기록, 가까이 오면 돌아보고(무심한 경계 — 천천히), 멀어지면 제 일 쪽으로. 곁이면 말 걸기.
        self.near_resident = None;
        for (k, &(_, at)) in self.residents.iter().enumerate() {
            let d = flat(at);
            if d <= crate::residents::SEE_RANGE && !self.resident_seen[k] {
                self.resident_seen[k] = true;
                self.resident_seen_new.push(k);
            }
            let home = ((at.x * 3.1 + at.z * 1.7).sin() * 2.0).rem_euclid(std::f32::consts::TAU);
            let want = if d <= crate::residents::GLANCE_RANGE {
                (pp.x - at.x).atan2(pp.z - at.z)
            } else {
                home
            };
            let f = &mut self.resident_face[k];
            let diff = (want - *f + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
                - std::f32::consts::PI;
            *f += diff.clamp(-1.5 * dt, 1.5 * dt);
            if d <= crate::residents::TALK_RANGE && self.near_resident.is_none() {
                self.near_resident = Some(k);
            }
        }
        // 점프 버튼이 다음 틱부터 쉬기·열기·줍기가 되는 자리(0048): 곁이고(쉬기는 안전할 때만) — 아니면 그대로 점프.
        w.interact_here = (self.near_rest.is_some() && self.rest_safe)
            || self.near_chest.is_some()
            || self.near_find.is_some()
            || self.near_lever
            || self.near_resident.is_some()
            || self.near_stone.is_some();
        // 같은 버튼: 줍기 > 열기 > 당기기 > 쉬기(HUD 안내와 같은 차례).
        if let (Some(i), true) = (self.near_find, w.interact) {
            self.found[i] = true;
            self.picked = Some(i);
        } else if let (Some(i), true) = (self.near_chest, w.interact) {
            self.chest_open[i] = true;
            self.opened = Some(i);
        } else if let (true, true) = (self.near_lever, w.interact) {
            self.gate_opened = true;
            self.gate_new = true;
            self.near_lever = false;
            self.set_gate(w, false);
            if self.phase == Phase::Entrance {
                self.phase = Phase::GateOpen;
                self.objective = Some(self.def.obj_gate);
            }
            self.card = Some((self.def.card_lever, CARD_TIME));
        } else if let (Some(k), true) = (self.near_resident, w.interact) {
            // 말 걸기(0122): 멈추지 않는다 — 앱이 머리 위 말풍선을 띄운다(다시 누르면 다음 줄).
            self.talked = Some(k);
        } else if let (Some(k), true) = (self.near_stone, w.interact) {
            self.read = Some(k);
        } else if let (Some(i), true) = (self.near_rest, w.interact) {
            if self.rest_safe {
                self.rest(w, i);
                self.rested = true;
            } else {
                self.card = Some((|s| s.rest_blocked, 1.6));
            }
        }
        // 구역 카드: 시작 이야기가 끝나면 첫 구역, 그다음은 새 구역에 처음 들어설 때.
        let z = self.zone_at(w.player.pos);
        if !self.first_card && self.intro <= 0.0 {
            self.first_card = true;
            self.card = Some((self.def.zones[self.zone].0, CARD_TIME));
        }
        if z > self.zone {
            self.zone = z;
            self.card = Some((self.def.zones[z].0, CARD_TIME));
        }
        match self.phase {
            // 물결 스테이지(시험·기믹)만 둘째 구역에 들어서면 물결 — 서 있는 적 스테이지는 문 장치가 문을 연다(0113).
            Phase::Entrance => {
                if z >= 1 && !self.def.waves.is_empty() {
                    self.phase = Phase::Waves(0);
                    self.spawn_wave(w, 0);
                    self.objective = Some(self.def.obj_waves);
                }
            }
            Phase::Waves(n) => {
                if self.alive_in_wave(w) <= WAVE_LEFT && n + 1 < self.def.waves.len() {
                    let keep: Vec<usize> = self.wave_ids.clone();
                    self.phase = Phase::Waves(n + 1);
                    self.spawn_wave(w, n + 1);
                    self.wave_ids.extend(keep);
                } else if self.alive_in_wave(w) == 0 {
                    self.phase = Phase::GateOpen;
                    self.set_gate(w, false);
                    self.objective = Some(self.def.obj_gate);
                }
            }
            Phase::GateOpen => {
                // 마지막 구역(성소)에 들어서면.
                if z + 1 >= self.def.zones.len() {
                    // 문이 닫히고 문지기가 깨어난다(플레이어가 문 안으로 다 들어온 뒤 닫는다).
                    let inside = self
                        .level
                        .gate
                        .is_none_or(|(g, _)| w.player.pos.z < g.min.z - 0.8);
                    if inside {
                        self.set_gate(w, true);
                        self.phase = Phase::Waking(0.0);
                        self.objective = None;
                        self.card = Some((self.def.boss_wakes, WAKE_TIME + 1.0));
                    }
                }
            }
            Phase::Waking(t) => {
                if t + dt >= WAKE_TIME {
                    if let Some(b) = self.boss.and_then(|i| w.enemies.get_mut(i)) {
                        b.state = EState::Chase;
                    }
                    self.phase = Phase::Duel;
                } else {
                    self.phase = Phase::Waking(t + dt);
                }
            }
            Phase::Duel => {
                let dead = self
                    .boss
                    .and_then(|i| w.enemies.get(i))
                    .is_none_or(|b| !b.alive());
                if dead {
                    self.core = self.boss.and_then(|i| w.enemies.get(i)).map(|b| b.pos);
                    self.phase = Phase::Core;
                    self.objective = Some(self.def.obj_core);
                    w.lock = None;
                }
            }
            Phase::Core => {
                // 처음 이긴 문지기의 핵은 손을 뻗으면 스스로 다가온다(lore CANON, 0101·0105) — 그다음부터는 평소대로 거둔다.
                if let (Some(c), true) = (self.core.as_mut(), self.first_core) {
                    let to = vec3(w.player.pos.x - c.x, 0.0, w.player.pos.z - c.z);
                    let d = to.length();
                    if d <= CORE_DRAW && d > 1e-3 {
                        *c += to / d * (CORE_DRAW_SPEED * dt).min(d);
                    }
                }
                if let Some(c) = self.core {
                    let d = vec2(c.x - w.player.pos.x, c.z - w.player.pos.z).length();
                    if d <= CORE_REACH {
                        // 아래 지역이 있으면 바닥 길이 열린다(0118) — 판은 이어진다.
                        if self.down_at.is_some() {
                            self.open_down(w);
                            self.card = Some((|s| s.card_way_down, CARD_TIME));
                            return StageOut::CoreTaken;
                        }
                        self.phase = Phase::Cleared;
                        self.objective = None;
                        return StageOut::Cleared;
                    }
                }
            }
            Phase::Down => {
                if self.passage_armed && near(self.down_at) {
                    return StageOut::Descend;
                }
            }
            Phase::Cleared => return StageOut::Cleared,
        }
        // 오르는 길(0118): 문지기와 싸우는 동안 말고는 언제나.
        if self.passage_armed
            && near(self.up_at)
            && !matches!(self.phase, Phase::Waking(_) | Phase::Duel)
        {
            return StageOut::Ascend;
        }
        StageOut::Playing
    }

    /// 쉬기(0032): 체력·등잔·스태미나 가득, 이 쉼터가 다시 일어설 곳. 되돌아갈 단계 = 지금(물결 중이면 물결 전부터).
    fn rest(&mut self, w: &mut World, i: usize) {
        let full = w.stats.max_stamina();
        let pl = &mut w.player;
        pl.hp = pl.max_hp;
        pl.flask = pl.flask_max;
        pl.mana = crate::tuning::CAST_CHARGES;
        pl.stamina = full;
        let back = match self.phase {
            Phase::Entrance | Phase::Waves(_) => Phase::Entrance,
            _ => Phase::GateOpen,
        };
        self.rest_at = Some((i, back));
        // 화로에서 쉬면 서 있는 적이 다시 선다(0113, 소울 화톳불 — 엘리트·보스 빼고).
        self.populate(w);
        self.since_rest = 0.0;
        self.card = Some((|s| s.rested, 2.4));
    }

    /// 쓰러진 뒤 쉼터에서 다시(0032): 플레이어는 쉼터 곁에 가득 찬 채로, 진행은 쉬던 때로 — 물결은 처음부터(뜰에 들어서면
    /// 다시 온다), 문지기는 잠든 채 가득. 문지기만 남기고 다른 적은 지운다(문지기는 시작할 때 처음 세워 번호 0).
    fn respawn(&mut self, w: &mut World, i: usize, back: Phase) {
        let at = self.rests[i];
        // 이 판에 이긴 문지기는 다시 서지 않는다(0118).
        if self.down_open {
            w.enemies.retain(|e| !e.boss);
        }
        // 기록의 문(0113)이 열려 있으면 문 앞 단계로.
        let back = if self.gate_opened {
            Phase::GateOpen
        } else {
            back
        };
        w.enemies.retain(|e| e.boss);
        self.boss = (!w.enemies.is_empty()).then_some(0);
        if let (Some(b), Some(home)) = (self.boss, self.level.marks_of(self.def.boss.0).next()) {
            let max = w.enemies[b].max_hp;
            let e = &mut w.enemies[b];
            e.hp = max;
            e.state = EState::Dormant;
            e.roused = false;
            e.exposed = 0.0;
            e.knock = Vec3::ZERO;
            e.pos = home;
            e.prev = home;
        }
        self.wave_ids.clear();
        self.elite_ids.clear();
        w.lingers.clear();
        w.shards.clear();
        w.lock = None;
        w.hitstop = 0.0;
        let pl = &mut w.player;
        pl.pos = at + vec3(0.0, 0.0, 1.4);
        pl.prev = pl.pos;
        pl.vel = Vec3::ZERO;
        pl.state = PState::Idle;
        pl.air = 0.0;
        pl.hurt = 1.0;
        self.rest(w, i);
        self.since_rest = f32::MAX;
        match back {
            Phase::Entrance => {
                self.phase = Phase::Entrance;
                self.set_gate(w, true);
                self.objective = self.def.waves.is_empty().then_some(self.def.obj_explore);
            }
            _ => {
                self.phase = Phase::GateOpen;
                self.set_gate(w, false);
                self.objective = Some(self.def.obj_gate);
            }
        }
        self.rest_at = Some((i, back));
        self.zone = self.zone_at(w.player.pos);
        self.core = None;
        if self.down_open {
            self.open_down(w);
        }
        self.passage_armed = false;
        self.card = Some((|s| s.respawned, CARD_TIME));
    }

    /// 이번 틱에 말 건 주민(0122) — 앱이 한 번 가져간다.
    /// 이번 틱에 읽은 새긴 돌(0129 — 앱이 글 판을 띄운다).
    pub fn take_read(&mut self) -> Option<usize> {
        self.read.take()
    }

    pub fn take_talked(&mut self) -> Option<usize> {
        self.talked.take()
    }

    /// 이 판에 처음 본 주민들(0122) — 앱이 한 번 가져가 기록에.
    pub fn take_residents_seen(&mut self) -> Vec<usize> {
        std::mem::take(&mut self.resident_seen_new)
    }

    /// 이번 틱에 쉬었나(0064 M4) — 앱이 한 번 가져가 화로 가게를 연다(쓰러진 뒤 다시 서는 것은 아니다).
    pub fn take_rested(&mut self) -> bool {
        std::mem::take(&mut self.rested)
    }

    /// 이번 틱에 연 상자(0064 M3) — 앱이 한 번 가져가 장신구·잔불을 준다.
    pub fn take_opened(&mut self) -> Option<usize> {
        self.opened.take()
    }

    /// 이번 틱에 주운 숨은 물건(0105): (종류, 기록 이름) — 앱이 한 번 가져가 기록·능력치에.
    pub fn take_picked(&mut self) -> Option<(Find, String)> {
        let i = self.picked.take()?;
        self.finds.get(i).map(|(_, k, id)| (*k, id.clone()))
    }

    /// 기록의 주운 숨은 물건들(판을 넘어 없다).
    pub fn set_found(&mut self, ids: &[String]) {
        for (i, (_, _, id)) in self.finds.iter().enumerate() {
            if ids.contains(id) {
                self.found[i] = true;
            }
        }
    }

    /// 기록의 연 상자들(판을 넘어 비어 있다).
    pub fn set_opened(&mut self, opened: &[u32]) {
        for &i in opened {
            if let Some(o) = self.chest_open.get_mut(i as usize) {
                *o = true;
            }
        }
    }

    /// 기록할 판 안의 자리(0064 M2): 마지막으로 쉰 쉼터와 되돌아갈 단계. 쉰 적이 없으면 없다(다음은 입구부터).
    pub fn run_at(&self) -> Option<crate::save::RunAt> {
        self.rest_at.map(|(i, back)| crate::save::RunAt {
            rest: i,
            gate_open: back != Phase::Entrance || self.gate_opened,
            ..Default::default()
        })
    }

    /// 이어하기(0064 M2): 시작 이야기 없이 기록의 쉼터에서 다시 선다 — 쓰러진 뒤 다시 서는 것과 같은 규칙(0032),
    /// 글만 "쉬었다". 기록의 쉼터 번호가 지도에 없으면(지도가 바뀌었다) 입구부터.
    pub fn resume(&mut self, w: &mut World, at: crate::save::RunAt) {
        if at.rest >= self.rests.len() {
            return;
        }
        self.intro = 0.0;
        self.first_card = true;
        // 옛 기록(물결 시절, 0113 전): 판 안의 "문이 열렸다"는 이제 기록의 문 — 한 번 옮겨 둔다.
        if at.gate_open && !self.gate_opened && self.def.waves.is_empty() {
            self.gate_opened = true;
            self.gate_new = true;
        }
        let back = if at.gate_open || self.gate_opened {
            Phase::GateOpen
        } else {
            Phase::Entrance
        };
        self.respawn(w, at.rest, back);
        self.card = Some((self.def.zones[self.zone].0, CARD_TIME));
    }

    /// 개발: 물결·서 있는 적을 건너뛰고 문 앞으로(문이 열린 채) — 보스만 다시 볼 때. 보스 말고는 지운다(0113 — 문 앞 가시짐승이 바로 덮쳤다).
    #[cfg(feature = "dev")]
    pub fn dev_skip_to_gate(&mut self, w: &mut World) {
        w.enemies.retain(|e| e.boss);
        self.boss = (!w.enemies.is_empty()).then_some(0);
        self.elite_ids.clear();
        w.lock = None;
        self.intro = 0.0;
        self.first_card = true;
        self.phase = Phase::GateOpen;
        self.objective = Some(self.def.obj_gate);
        self.set_gate(w, false);
        if let Some((g, _)) = self.level.gate {
            w.player.pos = vec3(g.center().x, 0.0, g.max.z + 4.0);
            w.player.prev = w.player.pos;
        }
        self.zone = self.def.zones.len().saturating_sub(2);
    }

    /// 보스 체력 띠를 띄울까(깨어나는 중부터 쓰러질 때까지).
    pub fn boss_bar(&self, w: &World) -> Option<(f32, bool)> {
        match self.phase {
            Phase::Waking(_) | Phase::Duel => self
                .boss
                .and_then(|i| w.enemies.get(i))
                .map(|b| (b.hp.max(0) as f32 / b.max_hp as f32, b.roused)),
            _ => None,
        }
    }

    /// 지금 가야 할 곳(0060 W1 길 안내): 문이 열렸으면 문, 핵이 남았으면 핵. 싸우는 동안·입구 길은 없음.
    /// 렌더는 여기에 호박색 빛기둥을, HUD는 화면 밖이면 가장자리 화살을 그린다.
    pub fn goal(&self) -> Option<Vec3> {
        match self.phase {
            Phase::GateOpen => self.level.gate.map(|(_, c)| c),
            Phase::Core => self.core,
            Phase::Down => self.down_at,
            _ => None,
        }
    }

    /// 문지기가 깨어나는 진행(0..1, 렌더의 `wake` 동작). 깨어나기 전 0, 다 깨면 1.
    pub fn wake_progress(&self) -> f32 {
        match self.phase {
            Phase::Entrance | Phase::Waves(_) | Phase::GateOpen => 0.0,
            Phase::Waking(t) => (t / WAKE_TIME).clamp(0.0, 1.0),
            _ => 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Actions;
    use crate::tuning::Tuning;
    use crate::tuning::{FLASK_CHARGES, PLAYER_HP};

    /// 0118(PR #10 검토 뒤 분명히): C3 전까지 마을 끝은 **싸우는 임시 문지기**(첫 문지기 모델) — 종탑 터에 들어서면 깨고,
    /// 이기고 핵을 거두면 마지막 지역이라 판이 끝난다(결과 화면). 바닥 길은 없다.
    #[test]
    fn the_village_bellkeeper_fights_and_its_core_ends_the_run() {
        let t = Tuning::default();
        let (mut run, mut w) = StageRun::start(STAGES[STAGES.len() - 1]);
        assert!(run.down_at.is_none(), "the last region has no way down");
        let home = run.level.marks_of('W').next().expect("bellkeeper mark");
        w.player.pos = home + vec3(0.0, 0.0, 4.0);
        w.player.prev = w.player.pos;
        run.tick(&mut w, 1.0 / 60.0);
        assert!(matches!(run.phase, Phase::Waking(_)), "{:?}", run.phase);
        for _ in 0..((WAKE_TIME * 60.0) as usize + 2) {
            w.player.hp = 5;
            w.step(&Actions::default(), &t);
            run.tick(&mut w, 1.0 / 60.0);
        }
        assert_eq!(run.phase, Phase::Duel);
        let b = run.boss.unwrap();
        assert_eq!(
            w.enemies[b].kind,
            Kind::Bellkeeper,
            "the second warden (0125)"
        );
        w.enemies[b].state = EState::Dead;
        w.step(&Actions::default(), &t);
        run.tick(&mut w, 1.0 / 60.0);
        assert_eq!(run.phase, Phase::Core);
        w.player.pos = run.core.unwrap();
        assert_eq!(
            run.tick(&mut w, 1.0 / 60.0),
            StageOut::Cleared,
            "the run ends here"
        );
    }

    /// 0091: 높은 땅에 선 문은 상자도 그 땅 위에 — 가운데만 땅에 올리고 상자를 0에 두면 위 한 자락이 시야 가림 판정·막기 밖이었다.
    #[test]
    fn the_gate_box_stands_on_the_ground_under_the_gate() {
        let (run, _) = StageRun::start(&sanctuary::DEF);
        let (b, c) = run.level.gate.expect("sanctuary has a gate");
        assert!(
            c.y > 0.5,
            "the sanctuary gate stands on raised ground ({})",
            c.y
        );
        assert!(
            (b.min.y - c.y).abs() < 1e-4,
            "box floor {} vs ground {}",
            b.min.y,
            c.y
        );
        assert!((b.max.y - (c.y + 4.5)).abs() < 1e-4);
    }

    #[test]
    fn the_sanctuary_map_parses_into_walls_props_lights_gate_and_marks() {
        let lv = Level::parse(sanctuary::DEF.map);
        assert!(lv.w > 10 && lv.h > 20);
        assert!(!lv.walls.is_empty() && !lv.props.is_empty() && !lv.lights.is_empty());
        assert!(lv.gate.is_some());
        assert!(lv.marks_of('W').count() == 1);
        // 줄마다 폭이 같다(글자 지도 실수 잡기).
        let rows: Vec<&str> = sanctuary::DEF
            .map
            .lines()
            .filter(|l| !l.trim().is_empty())
            .collect();
        assert!(
            rows.iter().all(|r| r.chars().count() == lv.w),
            "ragged map rows"
        );
        // 시작 자리는 바닥.
        let start = lv.player_start;
        assert!(lv.walls.iter().all(|wb| !(start.x > wb.min.x
            && start.x < wb.max.x
            && start.z > wb.min.z
            && start.z < wb.max.z)));
    }

    /// 0035: 배경 높이 격자는 지도와 같은 크기이고, 걸을 수 있는 칸·문·표지 칸에는 높이가 없다(월드는 평평한 땅 그대로).
    #[test]
    fn heights_match_the_map_and_every_ramp_joins_two_levels() {
        let g = sanctuary::DEF.heights.expect("sanctuary has heights");
        let map: Vec<&str> = sanctuary::DEF
            .map
            .lines()
            .filter(|l| !l.trim().is_empty())
            .collect();
        let hs: Vec<&str> = g.lines().filter(|l| !l.trim().is_empty()).collect();
        assert_eq!(map.len(), hs.len(), "rows");
        for (z, (m, h)) in map.iter().zip(&hs).enumerate() {
            assert_eq!(m.chars().count(), h.chars().count(), "row {z} width");
            for (x, (mc, hc)) in m.chars().zip(h.chars()).enumerate() {
                assert!(
                    hc == '.' || hc == '/' || hc.is_digit(36),
                    "row {z} col {x}: {hc:?}"
                );
                if hc == '/' {
                    assert!(mc != ' ' && mc != '#', "row {z} col {x}: ramp on {mc:?}");
                }
            }
        }
        let (run, _) = StageRun::start(&sanctuary::DEF);
        let ground = run.level.ground();
        for z in 0..run.level.h {
            for x in 0..run.level.w {
                if run.level.ramps[z * run.level.w + x] {
                    assert!(
                        matches!(ground.at(x, z), Cell::Ramp { from, to, .. } if from != to),
                        "ramp at row {z} col {x} joins two different levels: {:?}",
                        ground.at(x, z)
                    );
                }
            }
        }
        assert!(
            run.level.heights.iter().any(|h| h.is_some_and(|v| v > 0.0)),
            "some rise"
        );
        assert!(
            run.level.heights.iter().any(|h| h.is_some_and(|v| v < 0.0)),
            "some sink"
        );
    }

    /// 0043: 플레이어는 모든 바닥 칸에 갈 수 있고(걸음·경사로 + 0.5m 턱 점프 + 뛰어내리기), 모든 스폰·보스 표지에서
    /// 적이 걸어서(경사로로만) 플레이어 시작 자리·문 앞까지 온다 — 높낮이가 길을 끊지 않았나.
    #[test]
    fn every_floor_is_reachable_and_every_spawn_can_walk_to_the_fight() {
        // 0118: 모든 지역.
        for def in STAGES {
            let (run, _) = StageRun::start(def);
            let lv = &run.level;
            let g = lv.ground();
            // 플레이어 규칙: 이웃 칸 가운데 높이로 — 오르기는 점프 높이(0.9m) 안의 0.5m까지, 내리기는 아무리 깊어도.
            let start = g.cell_of(lv.player_start).unwrap();
            let mut seen = vec![false; lv.w * lv.h];
            let mut q = vec![start];
            seen[start.1 * lv.w + start.0] = true;
            while let Some((x, z)) = q.pop() {
                let here = g.center(x, z).y;
                for (dx, dz) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                    let (nx, nz) = (x as i32 + dx, z as i32 + dz);
                    if nx < 0 || nz < 0 || nx as usize >= lv.w || nz as usize >= lv.h {
                        continue;
                    }
                    let (nx, nz) = (nx as usize, nz as usize);
                    if seen[nz * lv.w + nx] || matches!(g.at(nx, nz), Cell::Solid) {
                        continue;
                    }
                    if g.center(nx, nz).y - here <= 0.5 + 1e-3 {
                        seen[nz * lv.w + nx] = true;
                        q.push((nx, nz));
                    }
                }
            }
            for z in 0..lv.h {
                for x in 0..lv.w {
                    if lv.walkable(x, z) {
                        assert!(seen[z * lv.w + x], "player cannot reach row {z} col {x}");
                    }
                }
            }
            // 적 규칙: 길찾기 거리 지도(`Nav::flow`, 소품·상자가 덮은 칸은 막힘 — 0076)로 — 시작 자리와 문 앞(뜰 쪽) 어디서든.
            // 문이 없는 지역(0118 마을)은 시작 자리만.
            let gate = lv.gate.map(|g| g.1 + vec3(0.0, 0.0, CELL * 2.0));
            let nav = crate::nav::Nav::build(Some(&g), &lv.obstacles(), lv.half_extent());
            for goal in std::iter::once(lv.player_start).chain(gate) {
                let dist = nav.flow(goal);
                for (c, at) in &lv.marks {
                    if c.is_ascii_lowercase() || c.is_ascii_uppercase() && *c != 'R' {
                        let (x, z) = g.cell_of(*at).unwrap();
                        assert!(
                            dist[z * lv.w + x] != u16::MAX,
                            "spawn {c} at row {z} col {x} cannot walk to {goal:?}"
                        );
                    }
                }
            }
        }
    }

    /// 끝까지 걸어 본다(0113): 입구(서 있는 적, 문 닫힘) → 문 장치 당기기(문 열림·기록 알림 한 번) → 성소(문 닫힘·깨어남) → 결투 → 핵 → 클리어.
    #[test]
    fn the_stage_script_runs_from_entrance_to_clear() {
        let t = Tuning::default();
        let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
        let step = |run: &mut StageRun, w: &mut World| {
            w.step(&Actions::default(), &t);
            run.tick(w, 1.0 / 60.0)
        };
        assert_eq!(run.phase, Phase::Entrance);
        assert!(run.gate_closed);
        assert_eq!(run.goal(), None, "the way to the gate is to be found");
        assert!(run.objective.is_some());
        // 문 장치를 당긴다.
        pull_lever(&mut run, &mut w);
        assert_eq!(run.phase, Phase::GateOpen);
        assert!(!run.gate_closed && run.gate_opened);
        assert!(run.take_gate_new(), "the app records it");
        assert!(!run.take_gate_new(), "once");
        // 0060 W1: 문이 열리면 갈 곳 = 문. 싸우는 동안은 없다.
        assert_eq!(run.goal(), run.level.gate.map(|(_, c)| c));
        // 성소 안으로.
        let (g, _) = run.level.gate.unwrap();
        w.player.pos = vec3(g.center().x, w.player.pos.y, g.min.z - 3.0);
        step(&mut run, &mut w);
        assert!(matches!(run.phase, Phase::Waking(_)), "{:?}", run.phase);
        assert!(run.gate_closed);
        assert_eq!(run.goal(), None, "no guide while the fight is on");
        for _ in 0..((WAKE_TIME * 60.0) as usize + 2) {
            w.player.hp = 5;
            step(&mut run, &mut w);
        }
        assert_eq!(run.phase, Phase::Duel);
        let b = run.boss.unwrap();
        assert!(w.enemies[b].active(), "awake: {:?}", w.enemies[b].state);
        assert!(run.boss_bar(&w).is_some());
        w.enemies[b].state = EState::Dead;
        step(&mut run, &mut w);
        assert_eq!(run.phase, Phase::Core);
        assert_eq!(run.goal(), run.core, "then the core");
        w.player.pos = run.core.unwrap();
        // 0118: 아래 지역이 있으면 핵은 판을 끝내지 않고 바닥 길을 연다 — 길이 갈 곳, 밟으면 내려간다.
        assert_eq!(step(&mut run, &mut w), StageOut::CoreTaken);
        assert_eq!(run.phase, Phase::Down);
        assert!(!run.gate_closed, "the way back out is open");
        assert_eq!(run.goal(), run.down_at);
        assert!(w.enemies.iter().all(|e| !e.boss), "the warden is gone");
        w.player.pos = run.down_at.unwrap();
        assert_eq!(step(&mut run, &mut w), StageOut::Descend);
    }

    /// 0118: 판의 차례 — 지역은 깊어지고, 마지막 지역만 바닥 길이 없다(핵 = 판의 끝). 첫 지역 말고는 오르는 길이 있다.
    #[test]
    fn stages_go_deeper_and_link_down_and_up() {
        for (i, def) in STAGES.iter().enumerate() {
            let (run, _) = StageRun::start(def);
            let last = i + 1 == STAGES.len();
            assert_eq!(
                run.down_at.is_none(),
                last,
                "stage {i}: a way down unless last"
            );
            assert_eq!(
                run.up_at.is_some(),
                i > 0,
                "stage {i}: a way up unless first"
            );
            if i > 0 {
                assert!(def.depth_m > STAGES[i - 1].depth_m, "stage {i} is deeper");
                assert_ne!(def.find_prefix, STAGES[i - 1].find_prefix);
            }
        }
    }

    /// 0121(0115 전투 안전 설계 — "AI보다 레벨 배치로 먼저"): 주민 자리(`1`·`2`·`3`)는 모든 서 있는 적·엘리트·문지기의 제자리에서
    /// 끈(`LEASH_RANGE`) + 3m 밖 — 쫓아오는 적도 주민 곁까지는 오지 않는다. 화로와도 겹치지 않는다.
    #[test]
    fn residents_live_beyond_every_enemy_leash() {
        use crate::tuning::LEASH_RANGE;
        for def in STAGES {
            let (run, _) = StageRun::start(def);
            let foes: Vec<Vec3> = def
                .spawns
                .iter()
                .map(|s| s.0)
                .chain(def.elites.iter().map(|e| e.0))
                .chain(std::iter::once(def.boss.0))
                .flat_map(|c| run.level.marks_of(c).collect::<Vec<_>>())
                .collect();
            for r in ['1', '2', '3'] {
                for home in run.level.marks_of(r) {
                    for f in &foes {
                        let d = vec2(home.x - f.x, home.z - f.z).length();
                        assert!(
                            d >= LEASH_RANGE + 3.0,
                            "{} resident {r} {d:.1} m from a foe",
                            def.find_prefix
                        );
                    }
                }
            }
        }
        let (run, _) = StageRun::start(STAGES[1]);
        assert_eq!(
            ['1', '2', '3']
                .iter()
                .filter(|c| run.level.marks_of(**c).next().is_some())
                .count(),
            3,
            "three residents in the village"
        );
    }

    /// 0129 C4: 새긴 돌은 지도 표지 수 = 글 수, 곁에서 상호작용 = 읽기(다른 일을 하지 않는다), 멀면 아무것도 없다.
    #[test]
    fn carved_stones_have_one_text_each_and_read_with_the_interact_button() {
        for def in STAGES {
            let (run, _) = StageRun::start(def);
            assert_eq!(run.stones.len(), def.stones.len(), "marks = texts");
        }
        let (mut run, mut w) = StageRun::start(STAGES[1]);
        assert_eq!(run.stones.len(), 3);
        let at = run.stones[2];
        w.player.pos = at + vec3(0.0, 0.0, 6.0);
        w.player.prev = w.player.pos;
        tick(&mut run, &mut w, Actions::default());
        assert_eq!(run.near_stone, None);
        w.player.pos = at + vec3(0.0, 0.0, 1.2);
        w.player.prev = w.player.pos;
        tick(&mut run, &mut w, Actions::default());
        assert_eq!(run.near_stone, Some(2));
        assert!(w.interact_here, "the jump button reads it");
        tick(
            &mut run,
            &mut w,
            Actions {
                interact: true,
                ..Default::default()
            },
        );
        assert_eq!(run.take_read(), Some(2));
        assert_eq!(run.take_read(), None, "once per press");
    }

    /// 0122: 주민은 월드의 적이 아니다(맞지도·락온되지도 않는다 — 공격이 지나간다), 몸은 방랑자만 밀어낸다.
    /// 곁에서 점프 버튼 = 말 걸기(다른 일을 하지 않는다), 가까이 오면 돌아보고, 보면 한 번 기록한다.
    #[test]
    fn residents_talk_glance_and_stay_out_of_the_fight() {
        let (mut run, mut w) = StageRun::start(STAGES[1]);
        assert_eq!(run.residents.len(), 3);
        assert_eq!(w.bodies.len(), 3);
        for &(_, at) in &run.residents {
            assert!(
                w.enemies.iter().all(|e| e.pos.distance(at) > 1.0),
                "not an enemy"
            );
        }
        let (_, at) = run.residents[0];
        // 멀리서는 안 보인다 → 다가가면 기록 한 번.
        w.player.pos = at + vec3(0.0, 0.0, crate::residents::SEE_RANGE + 6.0);
        tick(&mut run, &mut w, Actions::default());
        assert!(run.take_residents_seen().is_empty());
        w.player.pos = at + vec3(0.0, 0.0, 1.6);
        w.player.prev = w.player.pos;
        let face0 = run.resident_face[0];
        for _ in 0..90 {
            tick(&mut run, &mut w, Actions::default());
        }
        assert_eq!(run.take_residents_seen(), vec![0], "seen once");
        assert!(run.take_residents_seen().is_empty());
        let want = (w.player.pos.x - at.x).atan2(w.player.pos.z - at.z);
        let off = |f: f32| {
            ((want - f + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
                - std::f32::consts::PI)
                .abs()
        };
        assert!(
            off(run.resident_face[0]) <= off(face0).max(0.05),
            "turned toward the wanderer"
        );
        assert_eq!(run.near_resident, Some(0));
        assert!(w.interact_here);
        tick(
            &mut run,
            &mut w,
            Actions {
                interact: true,
                ..Default::default()
            },
        );
        assert_eq!(run.take_talked(), Some(0));
        assert!(run.rest_at.is_none(), "talking does nothing else");
        // 몸이 방랑자를 밀어낸다.
        w.player.pos = at + vec3(0.05, 0.0, 0.0);
        tick(&mut run, &mut w, Actions::default());
        assert!(
            w.player.pos.distance(at)
                >= crate::residents::BODY_R + crate::tuning::PLAYER_RADIUS - 0.05
        );
    }

    /// 0112·0116 T3: 구역 등급은 깊을수록 높고, 서 있는 적·엘리트가 그 등급으로 선다(보스는 그대로).
    #[test]
    fn deeper_regions_field_higher_tier_mobs() {
        for (i, def) in STAGES.iter().enumerate() {
            if i > 0 {
                assert!(def.tier >= STAGES[i - 1].tier);
            }
            let (_, w) = StageRun::start(def);
            for e in &w.enemies {
                assert_eq!(
                    e.tier,
                    if e.boss { 1 } else { def.tier },
                    "{}",
                    def.find_prefix
                );
            }
        }
        assert_eq!(STAGES[1].tier, 2, "0116: the village is tier 2");
    }

    /// 0118 T2: 땅 위 지역만 하늘·달·반딧불·나무, 땅속은 달이 없고 달빛이 약하며 흙이 떨어진다. 땅속엔 천장 틈 빛줄기가 있다.
    #[test]
    fn regions_below_ground_have_no_moon_and_light_through_ceiling_cracks() {
        for def in STAGES {
            let e = def.env;
            let below = def.depth_m > 0;
            assert_eq!(e.moon, !below, "{}", def.find_prefix);
            assert_eq!(e.fireflies || e.trees, !below);
            if below {
                assert!(e.falling && e.moonlight < 1.0);
                let (run, _) = StageRun::start(def);
                assert!(
                    run.level.marks_of(SHAFT_GLYPH).next().is_some(),
                    "light comes from somewhere"
                );
            }
        }
    }

    /// 0118: 내려와 닿으면 오르는 길 곁이지만 바로 올라가지 않는다 — 한 번 벗어났다 다시 밟아야. 장치 없는 지역은 문이 열린 단계.
    #[test]
    fn arriving_does_not_bounce_back_and_the_way_up_works_after_leaving_it() {
        let t = Tuning::default();
        let (mut run, mut w) = StageRun::start(STAGES[1]);
        assert_eq!(run.phase, Phase::GateOpen, "no lever, no waves: open");
        run.arrive_from_above(&mut w);
        let up = run.up_at.unwrap();
        w.player.pos = up;
        w.step(&Actions::default(), &t);
        assert_eq!(
            run.tick(&mut w, 1.0 / 60.0),
            StageOut::Playing,
            "not right away"
        );
        w.player.pos = up + vec3(0.0, 0.0, -PASSAGE_ARM - 0.5);
        w.step(&Actions::default(), &t);
        run.tick(&mut w, 1.0 / 60.0);
        w.player.pos = up;
        w.step(&Actions::default(), &t);
        assert_eq!(run.tick(&mut w, 1.0 / 60.0), StageOut::Ascend);
    }

    /// 0118: 아래에서 올라오면 이 판에 이긴 문지기는 없고 바닥 길이 열린 채 — 쉬었다 쓰러져도 그대로.
    #[test]
    fn coming_back_up_finds_the_warden_gone_and_the_floor_open_even_after_a_fall() {
        let (mut run, mut w) = StageRun::start(STAGES[0]);
        run.arrive_from_below(&mut w);
        assert_eq!(run.phase, Phase::Down);
        assert!(run.boss.is_none() && w.enemies.iter().all(|e| !e.boss));
        assert!(!run.gate_closed);
        let d = run.down_at.unwrap();
        assert!(
            w.player.pos.distance(d) > PASSAGE_REACH,
            "beside the hole, not in it"
        );
        rest_now(&mut run, &mut w, 0);
        w.player.hp = 0;
        assert_eq!(run.tick(&mut w, 1.0 / 60.0), StageOut::Respawned);
        assert_eq!(run.phase, Phase::Down);
        assert!(w.enemies.iter().all(|e| !e.boss), "still gone");
    }

    /// 문 장치 곁에서 상호작용(0113).
    fn pull_lever(run: &mut StageRun, w: &mut World) {
        let l = run.lever.expect("the sanctuary has a lever");
        w.player.pos = l + vec3(1.0, 0.0, 0.0);
        w.player.prev = w.player.pos;
        tick(run, w, Actions::default());
        tick(
            run,
            w,
            Actions {
                interact: true,
                ..Default::default()
            },
        );
    }

    /// 지도에 서 있는 보통 적의 수(엘리트 빼고).
    fn placed_count(run: &StageRun) -> usize {
        run.def
            .spawns
            .iter()
            .map(|(c, _)| run.level.marks_of(*c).count())
            .sum()
    }

    /// 0113: 지도의 적은 처음부터 제자리에(알아차리기 전) — 보통 적은 표의 표지 수만큼, 엘리트는 표의 차례·성질대로.
    #[test]
    fn placed_enemies_stand_at_their_marks_from_the_start() {
        let (run, w) = StageRun::start(&sanctuary::DEF);
        let plain: Vec<_> = w
            .enemies
            .iter()
            .filter(|e| !e.boss && e.elite.is_none())
            .collect();
        assert_eq!(plain.len(), placed_count(&run));
        assert!(plain.len() >= 20, "a real zone: {}", plain.len());
        assert!(plain
            .iter()
            .all(|e| e.state == EState::Idle && e.home.is_some()));
        let elites: Vec<_> = w
            .enemies
            .iter()
            .filter_map(|e| e.elite.map(|a| (e.kind, a)))
            .collect();
        let want: Vec<_> = sanctuary::DEF.elites.iter().map(|e| (e.1, e.2)).collect();
        assert_eq!(elites, want);
        assert!(sanctuary::DEF.waves.is_empty(), "no waves in a normal zone");
    }

    /// 0113: 화로 곁은 처음부터 안전하다 — 서 있는 적이 화로 곁의 플레이어를 알아차리지 않는다(쉴 수 있다).
    #[test]
    fn every_hearth_starts_safe_from_the_placed_enemies() {
        // 0118: 모든 지역.
        for def in STAGES {
            for i in 0..StageRun::start(def).0.rests.len() {
                let (mut run, mut w) = StageRun::start(def);
                // 쉬기 전에 곁에 잠깐 서 있는다(쉬면 적이 제자리로 돌아가 알아차림이 지워진다).
                w.player.pos = run.rests[i] + vec3(0.0, 0.0, 1.0);
                w.player.prev = w.player.pos;
                for _ in 0..10 {
                    tick(&mut run, &mut w, Actions::default());
                }
                let woke: Vec<_> = w
                    .enemies
                    .iter()
                    .filter(|e| !e.boss && e.state != EState::Idle)
                    .map(|e| e.pos)
                    .collect();
                assert!(woke.is_empty(), "hearth {i}: {woke:?} noticed");
                rest_now(&mut run, &mut w, i);
                assert!(run.rest_safe, "hearth {i} at {:?}", run.rests[i]);
                assert_eq!(run.rest_at.map(|r| r.0), Some(i));
            }
        }
    }

    /// 0113: 기록에 문이 열려 있으면 처음부터 열린 채(지름길) — 목표는 문.
    #[test]
    fn a_gate_opened_before_stays_open() {
        let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
        run.set_gate_opened(&mut w, true);
        assert_eq!(run.phase, Phase::GateOpen);
        assert!(!run.gate_closed);
        assert!(!run.take_gate_new(), "nothing new to record");
        assert_eq!(run.goal(), run.level.gate.map(|(_, c)| c));
    }

    /// 시험용: 한 틱(월드 → 스테이지).
    fn tick(run: &mut StageRun, w: &mut World, a: Actions) -> StageOut {
        w.step(&a, &Tuning::default());
        run.tick(w, 1.0 / 60.0)
    }

    fn rest_now(run: &mut StageRun, w: &mut World, i: usize) -> StageOut {
        w.player.pos = run.rests[i] + vec3(0.0, 0.0, 1.0);
        w.player.prev = w.player.pos;
        tick(
            run,
            w,
            Actions {
                interact: true,
                ..Default::default()
            },
        )
    }

    /// 입구 쪽 쉼터(시작 자리에서 가장 가까운 것).
    fn entrance_rest(run: &StageRun) -> usize {
        let s = run.level.player_start;
        (0..run.rests.len())
            .min_by(|a, b| {
                run.rests[*a]
                    .distance(s)
                    .total_cmp(&run.rests[*b].distance(s))
            })
            .unwrap()
    }

    /// 0032: 성소 지도에 쉼터 둘(입구·문 앞), 곁에서 상호작용 = 체력·등잔 가득 + 다시 일어설 곳.
    #[test]
    fn a_chest_opens_once_with_the_interact_button_and_stays_open_from_the_record() {
        let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
        assert_eq!(
            run.chests.len(),
            sanctuary::DEF.chest_loot.len(),
            "loot per chest"
        );
        run.intro = 0.0;
        let c = run.chests[1];
        w.player.pos = c + vec3(0.0, 0.0, 1.2);
        w.player.prev = w.player.pos;
        run.tick(&mut w, 0.016);
        assert_eq!(run.near_chest, Some(1));
        assert!(w.interact_here, "the jump button opens it");
        w.interact = true;
        run.tick(&mut w, 0.016);
        assert_eq!(run.take_opened(), Some(1));
        assert_eq!(run.take_opened(), None, "taken once");
        w.interact = true;
        run.tick(&mut w, 0.016);
        assert_eq!(run.take_opened(), None, "an open chest stays open");
        assert_eq!(run.near_chest, None);
        // 기록에서 연 상자는 새 판에서도 열려 있다.
        let (mut run, _) = StageRun::start(&sanctuary::DEF);
        run.set_opened(&[0, 9]);
        assert_eq!(run.chest_open, vec![true, false, false, false]);
    }

    /// 0105·0106 → 0123: 숨은 물건 넷(등잔 하나 · 한 모금 하나 · 종소리 조각 둘 — 덜 후하게)은 걸어서 닿는 바닥에 있고, 점프 버튼으로 한 번 줍고, 기록의 것은 새 판에 없다.
    #[test]
    fn hidden_finds_are_reachable_picked_once_and_gone_from_the_record() {
        let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
        let kinds: Vec<Find> = run.finds.iter().map(|f| f.1).collect();
        assert_eq!(kinds, vec![Find::Lamp, Find::Sip, Find::Shard, Find::Shard]);
        let ids: Vec<&str> = run.finds.iter().map(|f| f.2.as_str()).collect();
        assert_eq!(
            ids,
            vec![
                "sanctuary.lamp.0",
                "sanctuary.sip.0",
                "sanctuary.shard.0",
                "sanctuary.shard.1"
            ]
        );
        for (p, _, id) in &run.finds {
            // `Level::at`의 거꾸로.
            let cell = (
                (p.x / CELL + run.level.w as f32 * 0.5 - 0.5).round() as usize,
                (p.z / CELL + run.level.h as f32 * 0.5 - 0.5).round() as usize,
            );
            assert_eq!(run.level.at(cell.0, cell.1), vec3(p.x, 0.0, p.z));
            assert!(run.level.walkable(cell.0, cell.1), "{id} on a floor");
        }
        run.intro = 0.0;
        let (p, kind, id) = run.finds[1].clone();
        w.player.pos = p + vec3(0.0, 0.0, 1.0);
        w.player.prev = w.player.pos;
        run.tick(&mut w, 0.016);
        assert_eq!(run.near_find, Some(1));
        assert!(w.interact_here, "the jump button picks it up");
        w.interact = true;
        run.tick(&mut w, 0.016);
        assert_eq!(run.take_picked(), Some((kind, id.clone())));
        assert_eq!(run.take_picked(), None, "taken once");
        w.interact = true;
        run.tick(&mut w, 0.016);
        assert_eq!(run.take_picked(), None, "gone after picking");
        let (mut run, _) = StageRun::start(&sanctuary::DEF);
        run.set_found(&[id, "elsewhere.sip.0".to_string()]);
        assert_eq!(run.found, vec![false, true, false, false]);
    }

    /// 0105(lore CANON 0101): 처음 이긴 문지기의 핵은 다가오면 스스로 방랑자 쪽으로 온다. 그다음 판부터는 그 자리에 있다.
    #[test]
    fn the_first_core_drifts_to_the_wanderer_later_ones_wait() {
        for first in [true, false] {
            let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
            run.intro = 0.0;
            run.first_core = first;
            let c0 = w.player.pos + vec3(3.0, 0.0, 0.0);
            run.core = Some(c0);
            run.phase = Phase::Core;
            for _ in 0..30 {
                run.tick(&mut w, 1.0 / 60.0);
            }
            let moved = run.core.map_or(0.0, |c| c.distance(c0));
            if first {
                assert!(moved > 0.5, "the first core comes to you ({moved})");
            } else {
                assert!(moved < 1e-4, "later cores wait");
            }
        }
    }

    #[test]
    fn resuming_from_a_record_stands_at_that_rest_site_with_the_same_progress() {
        // 문 앞 쉼터에서 쉰 기록(문이 열린 뒤) → 새 판에서 이어하기 = 그 쉼터 곁, 문은 열려 있다.
        let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
        let last = run.rests.len() - 1;
        run.resume(
            &mut w,
            crate::save::RunAt {
                rest: last,
                gate_open: true,
                ..Default::default()
            },
        );
        assert_eq!(run.phase, Phase::GateOpen);
        assert!(!run.gate_closed);
        assert_eq!(run.intro, 0.0, "no opening story when continuing");
        let at = run.rests[last];
        assert!(vec2(w.player.pos.x - at.x, w.player.pos.z - at.z).length() < 2.0);
        assert_eq!(
            run.run_at(),
            Some(crate::save::RunAt {
                rest: last,
                gate_open: true,
                ..Default::default()
            })
        );
        // 서 있는 적은 쉼터에서 다시 섰다(0113).
        assert_eq!(
            w.enemies
                .iter()
                .filter(|e| !e.boss && e.elite.is_none())
                .count(),
            placed_count(&run)
        );
        assert!(
            run.gate_opened,
            "an old run record moves the open gate into the record"
        );
        assert!(run.take_gate_new());
        // 입구 쉼터·물결 전 기록이면 입구 단계.
        let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
        run.resume(
            &mut w,
            crate::save::RunAt {
                rest: 0,
                gate_open: false,
                ..Default::default()
            },
        );
        assert_eq!(run.phase, Phase::Entrance);
        assert!(run.gate_closed);
        // 지도에 없는 쉼터 번호면 아무것도 하지 않는다(처음부터).
        let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
        run.resume(
            &mut w,
            crate::save::RunAt {
                rest: 99,
                gate_open: true,
                ..Default::default()
            },
        );
        assert_eq!(run.phase, Phase::Entrance);
        assert_eq!(run.run_at(), None);
    }

    #[test]
    fn resting_refills_and_marks_the_rest_site() {
        let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
        assert_eq!(run.rests.len(), 3, "gate, root hollow, entrance");
        let i = entrance_rest(&run);
        w.player.hp = 1;
        w.player.flask = 0;
        rest_now(&mut run, &mut w, i);
        assert_eq!(run.near_rest, Some(i));
        assert_eq!((w.player.hp, w.player.flask), (PLAYER_HP, FLASK_CHARGES));
        assert_eq!(run.rest_at, Some((i, Phase::Entrance)));
    }

    /// 0032: 깨어 있는 적이 `REST_SAFE` 안이면 쉴 수 없다(잠든 문지기는 괜찮다).
    #[test]
    fn you_cannot_rest_with_an_awake_enemy_close() {
        let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
        let i = entrance_rest(&run);
        w.spawn(Kind::Husk, run.rests[i] + vec3(3.0, 0.0, 0.0));
        w.player.hp = 1;
        rest_now(&mut run, &mut w, i);
        assert!(!run.rest_safe);
        assert_eq!(run.rest_at, None);
        assert_eq!(w.player.hp, 1);
    }

    /// 0032 → 0113: 쉰 뒤 쓰러지면 그 쉼터에서 — 가득 찬 채, 서 있는 적은 다시 서고(소울 화톳불), 이 판에 쓰러뜨린 엘리트는
    /// 다시 서지 않는다, 문지기는 잠든 채.
    #[test]
    fn falling_after_a_rest_rises_there_and_the_zone_stands_again_but_not_downed_elites() {
        let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
        let i = entrance_rest(&run);
        rest_now(&mut run, &mut w, i);
        // 보통 적 둘과 첫 엘리트를 쓰러뜨린다.
        let mut down = 0;
        for e in w
            .enemies
            .iter_mut()
            .filter(|e| !e.boss && e.elite.is_none())
        {
            e.state = EState::Dead;
            down += 1;
            if down == 2 {
                break;
            }
        }
        let first = w.enemies.iter().position(|e| e.elite.is_some()).unwrap();
        w.enemies[first].state = EState::Dead;
        tick(&mut run, &mut w, Actions::default());
        w.player.hp = 0;
        w.player.flask = 0;
        assert_eq!(
            tick(&mut run, &mut w, Actions::default()),
            StageOut::Respawned
        );
        assert_eq!(run.phase, Phase::Entrance);
        assert!(w.player.pos.distance(run.rests[i]) < REST_REACH);
        assert_eq!((w.player.hp, w.player.flask), (PLAYER_HP, FLASK_CHARGES));
        let plain = w
            .enemies
            .iter()
            .filter(|e| e.alive() && !e.boss && e.elite.is_none())
            .count();
        assert_eq!(plain, placed_count(&run), "everyone stands again");
        assert_eq!(
            w.enemies.iter().filter(|e| e.elite.is_some()).count(),
            sanctuary::DEF.elites.len() - 1,
            "the downed elite stays down this run"
        );
        assert_eq!(w.enemies[run.boss.unwrap()].state, EState::Dormant);
        assert!(run.gate_closed);
    }

    /// 0112 → 0113: 엘리트를 처음 쓰러뜨리면 잔불을 더 흩고 앱에 한 번 알린다 — 기록에 있으면(다음 판) 덤은 없다.
    /// 한 틱에 여럿이 쓰러져도(수류탄) 다 센다.
    #[test]
    fn elites_pay_extra_the_first_time_only() {
        let first_drop = |beaten: &[String]| {
            let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
            run.set_beaten(beaten);
            for e in w.enemies.iter_mut().filter(|e| e.elite.is_some()) {
                e.state = EState::Dead;
            }
            w.drops.clear();
            tick(&mut run, &mut w, Actions::default());
            let extra = w.drops.iter().map(|d| d.value).sum::<u32>();
            (run.take_elites_new(), extra)
        };
        let (news, extra) = first_drop(&[]);
        assert_eq!(news.len(), sanctuary::DEF.elites.len());
        assert_eq!(news[0], elite_id("sanctuary", 0));
        // 쓰러진 적의 제 잔불은 두 판이 같다 — 차이가 처음 덤.
        let (again, plain) = first_drop(&news);
        assert!(again.is_empty(), "the first-time bonus is once");
        assert_eq!(
            extra - plain,
            crate::tuning::ELITE_FIRST_EMBERS * news.len() as u32
        );
    }

    /// 0032 → 0113: 문을 연 뒤 문 앞 쉼터에서 쉬었으면 결투에서 쓰러져도 문 앞부터 — 문은 열려 있고, 문지기는 잠든 채 가득.
    #[test]
    fn falling_in_the_duel_after_the_gate_rest_restarts_the_duel_only() {
        let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
        pull_lever(&mut run, &mut w);
        assert_eq!(run.phase, Phase::GateOpen);
        // 문 앞 쉼터 = 문에 가장 가까운 것.
        let gc = run.level.gate.unwrap().1;
        let gate_rest = (0..run.rests.len())
            .min_by(|a, b| {
                run.rests[*a]
                    .distance(gc)
                    .total_cmp(&run.rests[*b].distance(gc))
            })
            .unwrap();
        rest_now(&mut run, &mut w, gate_rest);
        assert_eq!(run.rest_at, Some((gate_rest, Phase::GateOpen)));
        // 성소로 → 깨어남 → 결투.
        let (g, _) = run.level.gate.unwrap();
        w.player.pos = vec3(g.center().x, w.player.pos.y, g.min.z - 3.0);
        for _ in 0..((WAKE_TIME * 60.0) as usize + 4) {
            w.player.hp = PLAYER_HP;
            tick(&mut run, &mut w, Actions::default());
        }
        assert_eq!(run.phase, Phase::Duel);
        let b = run.boss.unwrap();
        w.enemies[b].hp = 3;
        w.player.hp = 0;
        assert_eq!(
            tick(&mut run, &mut w, Actions::default()),
            StageOut::Respawned
        );
        assert_eq!(run.phase, Phase::GateOpen);
        assert!(!run.gate_closed);
        let boss = &w.enemies[run.boss.unwrap()];
        assert_eq!((boss.hp, boss.state), (boss.max_hp, EState::Dormant));
        assert!(w.player.pos.distance(run.rests[gate_rest]) < REST_REACH);
    }

    #[test]
    fn falling_ends_the_stage() {
        let t = Tuning::default();
        let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
        w.player.hp = 0;
        w.step(&Actions::default(), &t);
        assert_eq!(run.tick(&mut w, 1.0 / 60.0), StageOut::Defeated);
    }
}
