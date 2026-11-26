//! 스테이지 그림(0019): 레벨(`stage::Level`)을 한 번 굽는다 — 바닥 석판·벽 덩어리·소품 모델을 정점으로 만들고 빛(달빛 + 화로)을
//! 정점 색에 미리 곱해 둔다(움직이지 않으니 매 프레임 다시 셀 까닭이 없다). 기본 셰이더가 법선을 안 읽어 빛은 정점 색이 전부다
//! (KB reference/macroquad/3d-rendering.md).
//!
//! 가림: 벽 칸·소품마다 상자를 들고 있어, 카메라와 플레이어 사이를 막으면 그 덩어리만 옅게 그린다(0008의 기둥 옅어짐 그대로).
use crate::det::Det;
use macroquad::prelude::*;

use crate::ground::Ground;
use crate::models::Library;
use crate::render::{lit, Batch};
use crate::shade::Occluders;
use crate::stage::{Level, Light, PropKind, CELL, WALL_H};
use crate::tuning::GROUND_STEP;
use crate::world::Aabb;

/// 굽힌 덩어리 하나(정점은 이미 빛을 받았다).
pub struct Piece {
    pub verts: Vec<Vertex>,
    pub idx: Vec<u16>,
    pub tex: Option<Texture2D>,
    /// 가림 판정 상자(None이면 가리지 않는다 — 바닥).
    pub aabb: Option<Aabb>,
    pub fade: f32,
    /// 빛 지도 바닥(0044) — 캐릭터 그림자 재질로 그린다(0045).
    pub lightmap: bool,
}

pub struct Scenery {
    pub pieces: Vec<Piece>,
    pub lights: Vec<Light>,
    /// 바깥 허공 판의 반폭.
    pub extent: f32,
    /// 허공 판의 높이(가장 깊은 꺼진 땅보다 아래, 0035).
    pub void_y: f32,
    /// 걸을 수 있는 가장 낮은 땅(높이 안개는 이보다 아래에만, 0063).
    pub walk_min: f32,
    /// 달빛 가림 높이 지도(0044) — 구운 뒤에도 둔다: 그늘에 선 배우가 달빛을 덜 받게(0140, `render::actor_moon`).
    pub occ: std::rc::Rc<Occluders>,
    /// 덩어리마다 시야 거르기 상자(0258 — `pieces`와 같은 차례): 구운 정점의 범위 + 바람에 흔들리는 몫(`SWAY_PAD`).
    pub bounds: Vec<(Vec3, Vec3)>,
}

/// 바람에 흔들리는 정점이 구운 자리에서 벗어나는 가장 큰 거리(m, `WIND_VS` — 밟혀 눕기 0.34 + 돌풍 0.17)보다 넉넉히.
pub const SWAY_PAD: f32 = 0.6;

/// 칸 자리로 정하는 0..1 값(같은 지도 = 같은 그림).
fn hash(x: usize, z: usize, k: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(73_856_093)
        ^ (z as u32).wrapping_mul(19_349_663)
        ^ k.wrapping_mul(83_492_791);
    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1_e995);
    h ^= h >> 15;
    (h & 0xffff) as f32 / 65535.0
}

/// 꺼진 화로의 불씨 빛 반지름 몫(2026-11-23).
const COLD_REACH: f32 = 0.35;

/// 조용한 바닥(0140): 줄눈(`FloorLook::mortar_calm`)을 석판의 2/3 밝기로(예전 0.3배 — 화면에서 가장 센 대비가 바닥 격자였다),
/// 석판마다의 밝기 차는 ±6%로 줄이고 대신 여러 칸에 걸친 큰 얼룩(±10%)으로, 석판 옆면도 덜 어둡게.
const SLAB_SIDE_CALM: f32 = 0.82;
/// 큰 얼룩의 한 마디(칸).
const PATCH_CELLS: f32 = 5.0;
/// 빛 대비(0140): 화로 빛 반지름 배율, 소품 밑 그늘(세기·닿는 거리 m), 그림자 칸(m — 예전 0.25).
const FIRE_REACH: f32 = 1.3;
const PROP_AO: f32 = 0.4;
const PROP_AO_R: f32 = 0.45;
const SHADE_RES: f32 = 0.125;

/// 칸 하나(±1m)의 석판들(0268): 칸을 칼질로 나눈다 — 첫 칼은 가로·세로를 칸마다 고르고, 자리는 조각 길이의 36~64%, 남은 조각이
/// 1.3m를 넘으면 또 자르고(1.2m를 넘으면 가끔), 판 사이에 줄눈 0.05m씩. 이웃 칸과 칼 자리가 달라 줄눈이 바닥 전체로 곧게 이어지지
/// 않는다(같은 1m 넷이면 1m 격자였다). (판 가운데, 반폭, 판 번호 — 번호는 판마다의 굴림 씨앗).
fn slab_cuts(x: usize, z: usize) -> Vec<(Vec3, Vec2, u32)> {
    let mut out = Vec::new();
    let mut todo = vec![(vec2(-1.0, -1.0), vec2(1.0, 1.0), 0u32)];
    let mut n = 0u32;
    while let Some((lo, hi, depth)) = todo.pop() {
        let size = hi - lo;
        let long = size.x.max(size.y);
        let cut = depth < 4 && (long > 1.3 || (long > 1.2 && hash(x, z, 200 + n * 3) < 0.4));
        if cut {
            let along_x = if (size.x - size.y).abs() < 0.05 {
                hash(x, z, 201 + n * 3) < 0.5
            } else {
                size.x > size.y
            };
            let t = 0.36 + 0.28 * hash(x, z, 202 + n * 3);
            n += 1;
            if along_x {
                let m = lo.x + size.x * t;
                todo.push((lo, vec2(m, hi.y), depth + 1));
                todo.push((vec2(m, lo.y), hi, depth + 1));
            } else {
                let m = lo.y + size.y * t;
                todo.push((lo, vec2(hi.x, m), depth + 1));
                todo.push((vec2(lo.x, m), hi, depth + 1));
            }
            continue;
        }
        let mid = (lo + hi) * 0.5;
        let i = out.len() as u32;
        out.push((vec3(mid.x, 0.0, mid.y), size * 0.5 - vec2(0.05, 0.05), i));
    }
    out
}

/// 여러 칸에 걸쳐 천천히 바뀌는 0..1 값(격자 마디의 `hash`를 부드럽게 이음) — 석판 색의 큰 얼룩.
fn patch(x: usize, z: usize) -> f32 {
    let (fx, fz) = (x as f32 / PATCH_CELLS, z as f32 / PATCH_CELLS);
    let (ix, iz) = (fx.floor(), fz.floor());
    let s = |t: f32| t * t * (3.0 - 2.0 * t);
    let (tx, tz) = (s(fx - ix), s(fz - iz));
    let h = |dx: usize, dz: usize| hash(ix as usize + dx, iz as usize + dz, 77);
    let a = h(0, 0) + (h(1, 0) - h(0, 0)) * tx;
    let b = h(0, 1) + (h(1, 1) - h(0, 1)) * tx;
    a + (b - a) * tz
}

/// 지역마다의 땅 빛(0185 바닥 → 0249 바닥·벽·바위·턱까지): 석판 빛·이끼 빛, 석판 자리의 몇이 이끼인가(`moss`), 몇이 빠져 흙 + 풀인가(`bare`),
/// 줄눈(옛 그림·조용한 바닥), 벽(옆면·윗면), 배경 바위 단(옆면·이끼 낀 윗면), 꺼진 땅 바닥, 걸을 수 있는 층의 옹벽(0043)과 턱 가장자리의
/// 밝은 테(쿼터뷰에서 턱이 읽히게).
///
/// **색상대(0249)**: 주인공의 망토·적의 뿌리(보라 250~340°)·위험 예고(빨강 345~15°)는 세계가 쓰지 않는다. 불빛·눈빛·갈 곳의 호박색
/// (15~55°)은 채도 0.25 아래로만(흙빛) — 시험 `world_colors_keep_off_the_creatures_hues`. 값은 옛 남보라 값의 밝기(선형 휘도)를
/// 그대로 두고 색상·채도만 바꿨다(0140의 대비를 잃지 않게).
#[derive(Clone, Copy, Debug)]
pub struct FloorLook {
    pub slab: Color,
    pub moss_slab: Color,
    pub moss: f32,
    pub bare: f32,
    pub dirt: Color,
    pub mortar: Color,
    pub mortar_calm: Color,
    pub wall: Color,
    pub wall_top: Color,
    pub rock: Color,
    pub rock_top: Color,
    pub pit: Color,
    pub ledge: Color,
    pub edge: Color,
    /// 땅(0270 A — 사장 2026-11-23 "바닥은 무조건 타일이 있어야 하는거야?"): 있으면 석판 대신 칸 경계 없이 이어진 땅
    /// (얼룩 · 섬처럼 남은 석판 · 자갈 · 풀). 없으면 석판(0268).
    pub ground: Option<Soil>,
}

/// 타일 없는 땅의 꼴(0270 A): 흙(`FloorLook::dirt`) 위에 이끼 얼룩(`moss` 빛, 몫 `moss_amt`), 자갈(`gravel` 빛, 칸마다 평균 `pebbles`개),
/// 풀 다발(칸마다 확률 `grass`), 부서진 석판이 남은 칸(확률 `islands` — 그 칸의 판 몇만 남는다).
#[derive(Clone, Copy, Debug)]
pub struct Soil {
    pub moss: Color,
    pub moss_amt: f32,
    pub gravel: Color,
    pub pebbles: f32,
    pub grass: f32,
    pub islands: f32,
}

/// 성소(땅 위, 달빛 해 질 녘): 푸른 잿빛 석판(212°) — 다듬은 석판(0060 W2 그대로 — 이끼 두 칸에 하나쯤, 빠진 판 7%), 이끼는 청록, 흙은 옅은 갈색.
pub const SANCTUARY_FLOOR: FloorLook = FloorLook {
    moss: 0.2,
    bare: 0.07,
    slab: Color::new(0.282, 0.315, 0.353, 1.0),
    moss_slab: Color::new(0.218, 0.311, 0.287, 1.0),
    dirt: Color::new(0.214, 0.198, 0.182, 1.0),
    mortar: Color::new(0.084, 0.096, 0.11, 1.0),
    mortar_calm: Color::new(0.177, 0.203, 0.233, 1.0),
    wall: Color::new(0.337, 0.377, 0.422, 1.0),
    wall_top: Color::new(0.209, 0.234, 0.262, 1.0),
    rock: Color::new(0.245, 0.274, 0.307, 1.0),
    rock_top: Color::new(0.19, 0.272, 0.251, 1.0),
    pit: Color::new(0.108, 0.128, 0.155, 1.0),
    ledge: Color::new(0.301, 0.336, 0.377, 1.0),
    edge: Color::new(0.58, 0.618, 0.645, 1.0),
    ground: Some(Soil {
        moss: Color::new(0.2, 0.29, 0.262, 1.0),
        moss_amt: 0.4,
        gravel: Color::new(0.33, 0.35, 0.37, 1.0),
        pebbles: 0.8,
        grass: 0.12,
        islands: 0.45,
    }),
};

/// 떠난 마을(지하 40m): 흙빛 동굴 — 바랜 황토 석판(38°, 채도 0.16), 마른 올리브 이끼, 갈색 흙.
pub const VILLAGE_FLOOR: FloorLook = FloorLook {
    moss: 0.2,
    bare: 0.07,
    slab: Color::new(0.329, 0.31, 0.276, 1.0),
    moss_slab: Color::new(0.274, 0.302, 0.236, 1.0),
    dirt: Color::new(0.223, 0.196, 0.174, 1.0),
    mortar: Color::new(0.102, 0.094, 0.081, 1.0),
    mortar_calm: Color::new(0.215, 0.198, 0.172, 1.0),
    wall: Color::new(0.393, 0.37, 0.33, 1.0),
    wall_top: Color::new(0.244, 0.23, 0.205, 1.0),
    rock: Color::new(0.286, 0.269, 0.24, 1.0),
    rock_top: Color::new(0.24, 0.264, 0.206, 1.0),
    pit: Color::new(0.139, 0.125, 0.106, 1.0),
    ledge: Color::new(0.351, 0.33, 0.295, 1.0),
    edge: Color::new(0.628, 0.611, 0.577, 1.0),
    ground: Some(Soil {
        moss: Color::new(0.255, 0.28, 0.215, 1.0),
        moss_amt: 0.2,
        gravel: Color::new(0.37, 0.35, 0.31, 1.0),
        pebbles: 1.6,
        grass: 0.08,
        islands: 0.25,
    }),
};

/// 잔뿌리의 정원(0185, 지하 75m): 옛 석재는 조각만 남고(판의 반이 빠져 흙·풀), 남은 판의 반 넘게 이끼 — 물기 머금은 청록(155~185°).
pub const GARDEN_FLOOR: FloorLook = FloorLook {
    moss: 0.6,
    bare: 0.5,
    slab: Color::new(0.284, 0.32, 0.323, 1.0),
    moss_slab: Color::new(0.206, 0.317, 0.271, 1.0),
    dirt: Color::new(0.168, 0.191, 0.187, 1.0),
    mortar: Color::new(0.08, 0.098, 0.098, 1.0),
    mortar_calm: Color::new(0.17, 0.208, 0.208, 1.0),
    wall: Color::new(0.338, 0.381, 0.384, 1.0),
    wall_top: Color::new(0.21, 0.236, 0.239, 1.0),
    rock: Color::new(0.246, 0.277, 0.28, 1.0),
    rock_top: Color::new(0.179, 0.275, 0.235, 1.0),
    pit: Color::new(0.103, 0.132, 0.131, 1.0),
    ledge: Color::new(0.302, 0.34, 0.343, 1.0),
    edge: Color::new(0.573, 0.623, 0.615, 1.0),
    ground: Some(Soil {
        moss: Color::new(0.19, 0.3, 0.25, 1.0),
        moss_amt: 0.65,
        gravel: Color::new(0.3, 0.33, 0.33, 1.0),
        pebbles: 0.4,
        grass: 0.35,
        islands: 0.15,
    }),
};

/// 작업층(0208, 지하 110m): 닳은 철판·돌판 — 올리브빛 쇳빛(70°, 채도 0.08), 이끼 대신 녹(22°)·녹 슨 흙(빛이 없다).
pub const WORKS_FLOOR: FloorLook = FloorLook {
    moss: 0.25,
    bare: 0.22,
    slab: Color::new(0.309, 0.313, 0.288, 1.0),
    moss_slab: Color::new(0.32, 0.274, 0.248, 1.0),
    dirt: Color::new(0.213, 0.182, 0.162, 1.0),
    mortar: Color::new(0.094, 0.096, 0.084, 1.0),
    mortar_calm: Color::new(0.199, 0.203, 0.178, 1.0),
    wall: Color::new(0.37, 0.375, 0.345, 1.0),
    wall_top: Color::new(0.23, 0.233, 0.214, 1.0),
    rock: Color::new(0.269, 0.273, 0.251, 1.0),
    rock_top: Color::new(0.258, 0.258, 0.219, 1.0),
    pit: Color::new(0.125, 0.129, 0.109, 1.0),
    ledge: Color::new(0.331, 0.335, 0.309, 1.0),
    edge: Color::new(0.615, 0.615, 0.578, 1.0),
    ground: Some(Soil {
        moss: Color::new(0.3, 0.262, 0.236, 1.0),
        moss_amt: 0.25,
        gravel: Color::new(0.34, 0.34, 0.32, 1.0),
        pebbles: 2.2,
        grass: 0.0,
        islands: 0.2,
    }),
};

/// 서막 숲(0262, 땅 위): 석판은 옛 길의 조각만(판의 반이 빠져 흙 + 풀), 남은 판은 이끼. 흙은 어두운 부엽토(채도 낮게 —
/// 호박색대 밖), 바위 단 윗면은 이끼(나무가 선다) — 갈색 길이 초록 둘레와 갈려 읽힌다. 밝기는 성소 값의 휘도 그대로.
pub const FOREST_FLOOR: FloorLook = FloorLook {
    moss: 0.65,
    bare: 0.5,
    slab: Color::new(0.276, 0.31, 0.296, 1.0),
    moss_slab: Color::new(0.214, 0.31, 0.25, 1.0),
    dirt: Color::new(0.226, 0.204, 0.178, 1.0),
    mortar: Color::new(0.088, 0.098, 0.09, 1.0),
    mortar_calm: Color::new(0.186, 0.205, 0.19, 1.0),
    wall: Color::new(0.33, 0.37, 0.35, 1.0),
    wall_top: Color::new(0.2, 0.236, 0.214, 1.0),
    rock: Color::new(0.235, 0.262, 0.245, 1.0),
    rock_top: Color::new(0.176, 0.262, 0.2, 1.0),
    pit: Color::new(0.098, 0.124, 0.114, 1.0),
    ledge: Color::new(0.3, 0.33, 0.31, 1.0),
    edge: Color::new(0.57, 0.62, 0.59, 1.0),
    ground: Some(Soil {
        moss: Color::new(0.2, 0.29, 0.232, 1.0),
        moss_amt: 0.55,
        gravel: Color::new(0.33, 0.33, 0.31, 1.0),
        pebbles: 0.5,
        grass: 0.4,
        islands: 0.22,
    }),
};

/// 고인 물(`~`, 0185): 어둡고 푸른 수면, 가장자리는 젖은 흙.
const WATER: Color = Color::new(0.13, 0.23, 0.30, 1.0);
/// 수면의 달빛 결(0185): 칸을 가로지르는 가는 밝은 띠 — 평평한 판이 아니라 물로 읽히게.
const WATER_SHEEN: Color = Color::new(0.30, 0.40, 0.52, 1.0);
/// 물 위의 이끼 잎(0185).
const WATER_PAD: Color = Color::new(0.20, 0.33, 0.25, 1.0);
const WATER_Y: f32 = -0.04;
/// 마른 물길(`=`, 0185): 물이 지나던 자리 — 석판 없이 마른 흙과 잔돌.
const CHANNEL: Color = Color::new(0.26, 0.23, 0.22, 1.0);
const PEBBLE: Color = Color::new(0.40, 0.38, 0.40, 1.0);
/// 레일(`|`, 0208): 자갈 바닥 · 가로 침목 · 세로 쇠 레일 한 줄(두 칸이 한 짝 — 수레 폭 2m).
const BALLAST: Color = Color::new(0.20, 0.19, 0.20, 1.0);
const SLEEPER: Color = Color::new(0.27, 0.20, 0.16, 1.0);
const RAIL: Color = Color::new(0.46, 0.44, 0.46, 1.0);
// 0242(사장 설문 12-6 "풀이 있는지도 잘 모르겠다"): 끝을 노란 연두로 밝게 — 남보라 달빛 아래 바닥과 같은 청록이라 묻혔다.
const GRASS_ROOT: Color = Color::new(0.19, 0.30, 0.25, 1.0);
const GRASS_TIP: Color = Color::new(0.66, 0.84, 0.50, 1.0);
/// 석판 옆면 어둡기, 벽 밑 그늘(세기·닿는 거리 m).
/// 바닥은 걷는 면(0) 아래로 판다 — 예고·그림자·고리(바닥 위 2~3cm)가 석판에 묻히지 않게. 회반죽 깊이, 석판 꼭대기 범위(음수), 들린 변(0을 넘지 않는다).
const MORTAR_Y: f32 = -0.08;
const SLAB_LOW: f32 = -0.05;
/// 땅의 면(0270 A): 줄눈판 위, 석판 꼭대기 아래 — 섬처럼 남은 판이 땅에서 솟는다.
const GROUND_Y: f32 = -0.056;
/// 섬(남은 석판 무리)의 얼룩 크기, m(0270 A).
const ISLAND_SCALE: f32 = 6.0;
const SLAB_HIGH: f32 = -0.015;
const SLAB_LIFT: f32 = 0.03;
const SLAB_SIDE: f32 = 0.62;
const WALL_AO: f32 = 0.45;
const WALL_AO_R: f32 = 0.6;
/// 식생(0063 E3): 바위 단 칸마다 나무·덤불이 설 몫, 걸을 수 있는 쪽 벽 면마다 뿌리가 탈 몫.
pub const TREE_SHARE: f32 = 0.22;

/// 쓰러진 통나무(0262)의 반지름·표지 너머로 뻗는 길이(m — 길 양옆 덤불 단에 조금 걸치고 잘린 면은 보이게)·턱 쪽으로 붙는 몫(칸 가장자리에서 m).
/// 처음 0.36m는 쿼터뷰에서 턱의 테처럼 가늘게 읽혔다(스크린숏).
const LOG_R: f32 = 0.48;
const LOG_PAST: f32 = 0.9;
const LOG_IN: f32 = 0.22;

/// 통나무 자리(0262): 지도 `J` 짝마다 (밑동, 우듬지, 씨앗) — 두 표지를 잇는 줄을 턱(높은 이웃 칸) 쪽 칸 가장자리로 옮기고, 낮은 땅에
/// 반지름의 0.8만큼 묻히게 눕힌다. 높은 이웃이 없으면 표지 줄 그대로.
fn log_spans(lv: &Level, gnd: &Ground) -> Vec<(Vec3, Vec3, u32)> {
    let marks: Vec<Vec3> = lv.marks_of(crate::stage::LOG_GLYPH).collect();
    let mut out = Vec::new();
    for (k, pair) in marks.chunks(2).enumerate() {
        let [a, b] = pair else { continue };
        let Some((x, z)) = gnd.cell_of(*a) else {
            continue;
        };
        let here = gnd.center(x, z).y;
        let higher = |nz: usize| lv.walkable(x, nz) && gnd.center(x, nz).y > here + 0.2;
        let toward = if z > 0 && higher(z - 1) {
            -1.0
        } else if higher(z + 1) {
            1.0
        } else {
            0.0
        };
        let shift = vec3(0.0, LOG_R * 0.8, toward * (CELL * 0.5 - LOG_IN));
        let d = (*b - *a).normalize_or_zero();
        out.push((
            vec3(a.x, here, a.z) - d * LOG_PAST + shift,
            vec3(b.x, here, b.z) + d * LOG_PAST + shift,
            (x * 131 + z * 977 + k) as u32,
        ));
    }
    out
}

/// 지역의 나무(0262 — `backdrop::Env::woods`): 바위 단 칸에 나무가 서는 몫(0 = 없음 — 땅속), 살아 있는(푸른 잎) 나무의 몫.
/// 빽빽한 숲(`share`가 `TREE_SHARE`보다 크면)은 걷는 땅 바로 곁 칸에 덤불만 — 나무는 한 칸 뒤에 서서 길을 덮지 않고 둘레를 이룬다.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Woods {
    pub share: f32,
    pub live: f32,
}

impl Woods {
    pub const NONE: Woods = Woods {
        share: 0.0,
        live: 0.0,
    };

    pub fn dense(&self) -> bool {
        self.share > TREE_SHARE
    }
}
const BUSH_SHARE: f32 = 0.35;
const ROOT_SHARE: f32 = 0.16;

fn mul(c: Color, k: f32) -> Color {
    Color::new(c.r * k, c.g * k, c.b * k, c.a)
}

fn quad(p: &mut Piece, corners: [Vec3; 4], n: Vec3, col: Color, lights: &[Light]) {
    let base = p.verts.len() as u16;
    for c in corners {
        let l = lit(c, n, lights);
        p.verts.push(with_normal(
            Vertex::new(
                c.x,
                c.y,
                c.z,
                0.0,
                0.0,
                Color::new(col.r * l.x, col.g * l.y, col.b * l.z, col.a),
            ),
            n,
        ));
    }
    p.idx
        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

/// 축 정렬 상자(바닥면 없음). 윗면은 `top` 색.
fn block(p: &mut Piece, min: Vec3, max: Vec3, side: Color, top: Color, lights: &[Light]) {
    let (a, b) = (min, max);
    quad(
        p,
        [
            vec3(a.x, b.y, a.z),
            vec3(b.x, b.y, a.z),
            vec3(b.x, b.y, b.z),
            vec3(a.x, b.y, b.z),
        ],
        Vec3::Y,
        top,
        lights,
    );
    // 옆면은 아래로 갈수록 어둡게(바닥에 닿는 그늘) — 위 모서리와 아래 모서리를 따로 빛 계산하고 아래만 곱한다.
    let faces = [
        (
            Vec3::Z,
            [
                vec3(a.x, a.y, b.z),
                vec3(b.x, a.y, b.z),
                vec3(b.x, b.y, b.z),
                vec3(a.x, b.y, b.z),
            ],
        ),
        (
            Vec3::NEG_Z,
            [
                vec3(b.x, a.y, a.z),
                vec3(a.x, a.y, a.z),
                vec3(a.x, b.y, a.z),
                vec3(b.x, b.y, a.z),
            ],
        ),
        (
            Vec3::X,
            [
                vec3(b.x, a.y, b.z),
                vec3(b.x, a.y, a.z),
                vec3(b.x, b.y, a.z),
                vec3(b.x, b.y, b.z),
            ],
        ),
        (
            Vec3::NEG_X,
            [
                vec3(a.x, a.y, a.z),
                vec3(a.x, a.y, b.z),
                vec3(a.x, b.y, b.z),
                vec3(a.x, b.y, a.z),
            ],
        ),
    ];
    for (n, c) in faces {
        let base = p.verts.len() as u16;
        for (i, v) in c.iter().enumerate() {
            let l = lit(*v, n, lights) * if i < 2 { 0.55 } else { 1.0 };
            p.verts.push(with_normal(
                Vertex::new(
                    v.x,
                    v.y,
                    v.z,
                    0.0,
                    0.0,
                    Color::new(side.r * l.x, side.g * l.y, side.b * l.z, 1.0),
                ),
                n,
            ));
        }
        p.idx
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
}

fn empty(aabb: Option<Aabb>) -> Piece {
    Piece {
        verts: Vec::new(),
        idx: Vec::new(),
        tex: None,
        aabb,
        fade: 1.0,
        lightmap: false,
    }
}

impl Scenery {
    /// `woods` = 바위 단 칸에 나무가 서는 몫(0118 T2 — 땅속은 0: 벽 뿌리만, 0262 — 서막 숲은 빽빽하게).
    pub fn bake(
        lv: &Level,
        lib: &mut Library,
        flora: bool,
        woods: Woods,
        floor: &FloorLook,
    ) -> Scenery {
        let look = crate::render::look();
        // 빛 대비(0140): 화로 빛 웅덩이를 넓게(빛줄기는 그대로) — 따뜻한 자리가 화면에서 더 큰 덩어리가 되게.
        // 꺼진 화로(0171): 그 자리의 빛은 굽지 않는다(불꽃 부위도 — 아래 소품).
        let cold = |p: Vec3| {
            lv.cold
                .is_some_and(|c| vec2(c.x - p.x, c.z - p.z).length() < 0.1)
        };
        let lights: Vec<Light> = lv
            .lights
            .iter()
            .map(|l| {
                let reach = if look.key && !l.shaft {
                    l.radius * FIRE_REACH
                } else {
                    l.radius
                };
                // 꺼진 화로(0171)는 꺼져 가는 불씨의 작은 빛만(2026-11-23 — 그 전엔 빛을 뺐다: 어둠 속의 돌 고리가 보이지 않았다).
                if cold(l.pos) {
                    Light {
                        radius: reach * COLD_REACH,
                        color: Color::new(l.color.r * 0.8, l.color.g * 0.55, l.color.b * 0.4, 1.0),
                        ..*l
                    }
                } else {
                    Light {
                        radius: reach,
                        ..*l
                    }
                }
            })
            .collect();
        let mut pieces = Vec::new();
        let gnd = lv.ground();
        // 벽 칸 꼭대기(곁 바닥이 올라가 있으면 그만큼 높게, 0043) — 벽 그림과 달빛 가림이 같이 쓴다.
        let wall_top = |x: usize, z: usize| -> f32 {
            let floor = neighbors(x, z)
                .filter(|&(nx, nz)| lv.walkable(nx, nz))
                .map(|(nx, nz)| gnd.center(nx, nz).y)
                .fold(0.0f32, f32::max);
            lv.height(x, z)
                .unwrap_or(floor + WALL_H * (0.8 + 0.35 * hash(x, z, 3)))
                .max(floor + 0.5)
        };
        // 달빛 가림 높이 지도(0044): 0.25m 칸마다 가장 높은 단단한 것. 빛 대비(0140)면 0.125m — 그림자 가장자리의 계단이
        // 쿼터뷰에서 6px → 3px(캐릭터 그림자 가림판과 같은 칸).
        let res = if look.key { SHADE_RES } else { 0.25 };
        let origin = vec2(-(lv.w as f32) * CELL * 0.5, -(lv.h as f32) * CELL * 0.5);
        let (ow, oh) = (
            (lv.w as f32 * CELL / res) as usize,
            (lv.h as f32 * CELL / res) as usize,
        );
        let mut occ = Occluders::new(origin, res, ow, oh);
        for j in 0..oh {
            for i in 0..ow {
                let c = occ.center(i, j);
                let (x, z) = (
                    ((c.x - origin.x) / CELL) as usize,
                    ((c.y - origin.y) / CELL) as usize,
                );
                let y = match lv.cell(x, z) {
                    _ if lv.walkable(x, z) => gnd.height_in(x, z, vec3(c.x, 0.0, c.y)),
                    b'#' => Some(wall_top(x, z)),
                    _ => lv.height(x, z),
                };
                if let Some(y) = y {
                    occ.raise(i, j, y);
                }
            }
        }
        for pr in &lv.props {
            let half = pr.kind.half();
            if half != Vec2::ZERO {
                let c = vec2(pr.pos.x, pr.pos.z);
                occ.raise_box(c - half, c + half, pr.pos.y + pr.kind.height());
            }
        }
        // 통나무(0262)도 달빛을 가린다(그림자 · 그 곁에 선 배우).
        let logs = log_spans(lv, &gnd);
        for (a, b, _) in &logs {
            let r = Vec2::splat(LOG_R);
            let (lo, hi) = (
                vec2(a.x.min(b.x), a.z.min(b.z)),
                vec2(a.x.max(b.x), a.z.max(b.z)),
            );
            occ.raise_box(lo - r, hi + r, a.y.max(b.y) + LOG_R);
        }
        let prop_boxes: Vec<(Vec2, Vec2)> = lv
            .props
            .iter()
            .filter(|pr| pr.kind.half() != Vec2::ZERO)
            .map(|pr| {
                let c = vec2(pr.pos.x, pr.pos.z);
                (c - pr.kind.half(), c + pr.kind.half())
            })
            .collect();
        // 바닥 빛 지도(0044): 칸마다 달빛(가림)·화로를 미리 — 바닥은 색 × 이 텍스처(가장 가까운 픽셀 → 또렷한 그림자 경계).
        // 텍스처는 0..1이라 빛의 반을 담고 바닥 색을 두 배로 칠한다(화로 곁은 1보다 밝다).
        crate::render::set_bake_shade(Some(occ.clone()));
        let mut lm = vec![0u8; ow * oh * 4];
        for j in 0..oh {
            for i in 0..ow {
                let c = occ.center(i, j);
                let (x, z) = (
                    ((c.x - origin.x) / CELL) as usize,
                    ((c.y - origin.y) / CELL) as usize,
                );
                if !lv.walkable(x, z) {
                    continue;
                }
                let p = vec3(
                    c.x,
                    gnd.height_in(x, z, vec3(c.x, 0.0, c.y)).unwrap_or(0.0),
                    c.y,
                );
                // 벽 밑 그늘(0060 W2): 이웃이 벽 칸이면 그 변까지의 거리로 어둡게 — 바닥이 벽에 닿는 자리가 읽히게.
                let cc = lv.at(x, z);
                let (fx, fz) = (c.x - (cc.x - CELL * 0.5), c.y - (cc.z - CELL * 0.5));
                let wall = |dx: i32, dz: i32| {
                    let (nx, nz) = (x as i32 + dx, z as i32 + dz);
                    nx >= 0 && nz >= 0 && lv.cell(nx as usize, nz as usize) == b'#'
                };
                let mut near = f32::MAX;
                for (dx, dz, d) in [
                    (-1, 0, fx),
                    (1, 0, CELL - fx),
                    (0, -1, fz),
                    (0, 1, CELL - fz),
                ] {
                    if wall(dx, dz) {
                        near = near.min(d);
                    }
                }
                let mut ao = 1.0 - WALL_AO * (1.0 - (near / WALL_AO_R).min(1.0));
                // 소품 밑 그늘(0140): 부딪힘 상자까지의 거리로 — 기둥·벽 조각·화로가 바닥에 "놓여" 보이게.
                if look.key {
                    for (lo, hi) in &prop_boxes {
                        let d = (*lo - c).max(c - *hi).max(Vec2::ZERO).length();
                        if d < PROP_AO_R {
                            ao *= 1.0 - PROP_AO * (1.0 - d / PROP_AO_R);
                        }
                    }
                }
                let l = lit(p, slope_normal(&gnd, x, z), &lights) * 0.5 * ao;
                let k = (j * ow + i) * 4;
                lm[k] = to_u8(l.x);
                lm[k + 1] = to_u8(l.y);
                lm[k + 2] = to_u8(l.z);
                lm[k + 3] = 255;
            }
        }
        // 0242(사장 설문 10-2 "그림자가 실제처럼 보이지 않음"): 가장 가까운 픽셀의 계단 경계 → 걷는 칸끼리만 [1 2 1] 흐리기 +
        // 선형 거르기 — 툰의 두 단은 그대로 두고 경계만 반그림자처럼 부드럽게(0.25m 남짓). 걷지 못하는 칸(검정)은 섞지 않는다.
        let lm = soften_lightmap(&lm, ow, oh);
        let lightmap = Texture2D::from_rgba8(ow as u16, oh as u16, &lm);
        lightmap.set_filter(FilterMode::Linear);
        let uv_of = |v: Vec3| {
            vec2(
                (v.x - origin.x) / (ow as f32 * res),
                (v.z - origin.y) / (oh as f32 * res),
            )
        };
        // 바닥: 칸마다 1m 석판 넷(틈 = 회반죽 판), 몇 칸은 이끼. 땅 높이를 따른다(경사로는 기운다, 0043). 정점 한도 아래로 줄마다 덩어리.
        // 빛은 빛 지도 텍스처(0044) — 바닥 판은 색만(두 배), 옹벽·테는 정점 빛(따로 덩어리).
        let gy = |x: usize, z: usize, p: Vec3| gnd.height_in(x, z, p).unwrap_or(0.0);
        // 바닥 표지(0185): 고인 물 `~`·마른 물길 `=` 칸은 석판 대신 제 그림.
        let mut wet = vec![0u8; lv.w * lv.h];
        for (c, at) in &lv.marks {
            if let (Some(k), Some((x, z))) = (
                match c {
                    '~' => Some(1u8),
                    '=' => Some(2u8),
                    '|' => Some(3u8),
                    _ => None,
                },
                gnd.cell_of(*at),
            ) {
                wet[z * lv.w + x] = k;
            }
        }
        for z in 0..lv.h {
            let mut p = empty(None);
            p.tex = Some(lightmap.clone());
            p.lightmap = true;
            let mut sides = empty(None);
            for x in 0..lv.w {
                if !lv.walkable(x, z) {
                    continue;
                }
                let c = lv.at(x, z);
                let h = CELL * 0.5;
                let on = |v: Vec3| vec3(v.x, v.y + gy(x, z, v), v.z);
                quad_lm(
                    &mut p,
                    [
                        on(c + vec3(-h, MORTAR_Y, -h)),
                        on(c + vec3(h, MORTAR_Y, -h)),
                        on(c + vec3(h, MORTAR_Y, h)),
                        on(c + vec3(-h, MORTAR_Y, h)),
                    ],
                    if look.calm {
                        floor.mortar_calm
                    } else {
                        floor.mortar
                    },
                    &uv_of,
                );
                let mark = wet[z * lv.w + x];
                if mark == 1 {
                    // 고인 물: 칸 전체가 고른 수면(걷는 면 아래 4cm) — 빛 지도가 천장 틈 빛을 받는다.
                    let w = mul(WATER, 0.92 + 0.16 * patch(x, z));
                    quad_lm(
                        &mut p,
                        [
                            on(c + vec3(-h, WATER_Y, -h)),
                            on(c + vec3(h, WATER_Y, -h)),
                            on(c + vec3(h, WATER_Y, h)),
                            on(c + vec3(-h, WATER_Y, h)),
                        ],
                        w,
                        &uv_of,
                    );
                    // 달빛 결: 칸의 몇에 가는 띠 하나(물결 방향은 한쪽으로 — 띠가 이어져 보이게).
                    if hash(x, z, 81) < 0.45 {
                        let o = (hash(x, z, 82) - 0.5) * 0.7;
                        let (a, b) = (0.06, 0.32 + 0.12 * hash(x, z, 83));
                        quad_lm(
                            &mut p,
                            [
                                on(c + vec3(-b, WATER_Y + 0.006, o - a)),
                                on(c + vec3(b, WATER_Y + 0.006, o - a)),
                                on(c + vec3(b, WATER_Y + 0.006, o + a)),
                                on(c + vec3(-b, WATER_Y + 0.006, o + a)),
                            ],
                            WATER_SHEEN,
                            &uv_of,
                        );
                    }
                    // 떠 있는 이끼 잎.
                    if hash(x, z, 84) < 0.22 {
                        let o = vec3(
                            (hash(x, z, 85) - 0.5) * 0.5,
                            0.0,
                            (hash(x, z, 86) - 0.5) * 0.5,
                        );
                        let r = 0.14 + 0.08 * hash(x, z, 87);
                        quad_lm(
                            &mut p,
                            [
                                on(c + o + vec3(0.0, WATER_Y + 0.01, -r)),
                                on(c + o + vec3(r, WATER_Y + 0.01, 0.0)),
                                on(c + o + vec3(0.0, WATER_Y + 0.01, r)),
                                on(c + o + vec3(-r, WATER_Y + 0.01, 0.0)),
                            ],
                            WATER_PAD,
                            &uv_of,
                        );
                    }
                    continue;
                }
                if mark == 3 {
                    // 레일(0208): 자갈 바닥 + 침목 셋(가로) + 쇠 레일(세로, 칸 가운데).
                    let flat = |y: f32, x0: f32, x1: f32, z0: f32, z1: f32| {
                        [
                            on(c + vec3(x0, y, z0)),
                            on(c + vec3(x1, y, z0)),
                            on(c + vec3(x1, y, z1)),
                            on(c + vec3(x0, y, z1)),
                        ]
                    };
                    quad_lm(
                        &mut p,
                        flat(MORTAR_Y + 0.02, -h, h, -h, h),
                        mul(BALLAST, 0.9 + 0.2 * hash(x, z, 63)),
                        &uv_of,
                    );
                    for t in 0..3 {
                        let zc = -h + (t as f32 + 0.5) * (2.0 * h / 3.0);
                        quad_lm(
                            &mut p,
                            flat(SLAB_LOW, -h * 0.95, h * 0.95, zc - 0.13, zc + 0.13),
                            mul(SLEEPER, 0.85 + 0.25 * hash(x, z, 64 + t)),
                            &uv_of,
                        );
                    }
                    quad_lm(
                        &mut p,
                        flat(SLAB_HIGH + 0.005, -0.07, 0.07, -h, h),
                        RAIL,
                        &uv_of,
                    );
                    continue;
                }
                if mark == 2 {
                    // 마른 물길: 마른 흙 + 잔돌 몇(석판 없음).
                    quad_lm(
                        &mut p,
                        [
                            on(c + vec3(-h, MORTAR_Y + 0.02, -h)),
                            on(c + vec3(h, MORTAR_Y + 0.02, -h)),
                            on(c + vec3(h, MORTAR_Y + 0.02, h)),
                            on(c + vec3(-h, MORTAR_Y + 0.02, h)),
                        ],
                        mul(CHANNEL, 0.9 + 0.2 * hash(x, z, 61)),
                        &uv_of,
                    );
                    for t in 0..4u32 {
                        let o = c + vec3(
                            (hash(x, z, 70 + t) - 0.5) * 1.6,
                            0.0,
                            (hash(x, z, 74 + t) - 0.5) * 1.6,
                        );
                        let r = 0.08 + 0.1 * hash(x, z, 78 + t);
                        quad_lm(
                            &mut p,
                            [
                                on(o + vec3(-r, MORTAR_Y + 0.05, -r)),
                                on(o + vec3(r, MORTAR_Y + 0.05, -r)),
                                on(o + vec3(r, MORTAR_Y + 0.05, r)),
                                on(o + vec3(-r, MORTAR_Y + 0.05, r)),
                            ],
                            mul(PEBBLE, 0.8 + 0.3 * hash(x, z, 82 + t)),
                            &uv_of,
                        );
                    }
                    continue;
                }
                // 땅(0270 A): 석판 대신 칸 경계 없이 이어진 얼룩 — 칸을 2×2로 나눠 정점 색을 월드 자리에서 고른다(이웃 칸의 같은 자리 =
                // 같은 색이라 이음이 없다). 그 위에 자갈 몇·풀 다발, 몇 칸에만 부서진 석판이 섬처럼 남는다(아래 석판 길이 `island`만).
                let island = match floor.ground {
                    Some(g) => {
                        for (i, j) in [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)] {
                            let (x0, z0) = (-h + i * h, -h + j * h);
                            let corners = [(x0, z0), (x0 + h, z0), (x0 + h, z0 + h), (x0, z0 + h)]
                                .map(|(a, b)| on(c + vec3(a, GROUND_Y, b)));
                            let cols = corners.map(|v| ground_color(floor, &g, v));
                            quad_lm_cols(&mut p, corners, cols, &uv_of);
                        }
                        let n = (g.pebbles * 2.0 * hash(x, z, 210)).round() as u32;
                        for t in 0..n.min(6) {
                            let o = c + vec3(
                                (hash(x, z, 211 + t) - 0.5) * 1.8,
                                0.0,
                                (hash(x, z, 217 + t) - 0.5) * 1.8,
                            );
                            let r = 0.05 + 0.09 * hash(x, z, 223 + t);
                            let y = GROUND_Y + 0.02;
                            quad_lm(
                                &mut p,
                                [
                                    on(o + vec3(0.0, y, -r)),
                                    on(o + vec3(r, y, 0.0)),
                                    on(o + vec3(0.0, y, r)),
                                    on(o + vec3(-r, y, 0.0)),
                                ],
                                mul(g.gravel, 0.8 + 0.35 * hash(x, z, 229 + t)),
                                &uv_of,
                            );
                        }
                        if hash(x, z, 235) < g.grass {
                            let a = vec3(hash(x, z, 236) - 0.5, 0.0, hash(x, z, 237) - 0.5);
                            tuft(
                                &mut sides,
                                on(c + a * 1.6 + Vec3::Y * GROUND_Y),
                                x * 13 + 3,
                                z * 7 + 1,
                                &lights,
                            );
                        }
                        // 섬은 칸마다 따로가 아니라 큰 얼룩 안에 무리로(폐허의 깔린 터) — 판 하나하나는 아래에서 얼룩 가장자리를 따라 거른다.
                        field(c.x, c.z, ISLAND_SCALE, 239) > 1.0 - g.islands - 0.06
                    }
                    None => true,
                };
                // 석판(0060 W2·0061): 칸마다 1m 넷 — 짝 둘은 가끔 하나로 붙고(긴 판), 꼭대기는 걷는 면 아래 1.5~5cm로 들쭉날쭉, 몇은 한쪽이 들리고,
                // 몇은 빠져 흙 + 풀. 옆면이 있어 카메라 쪽 모서리에 밝은 선(0061 화면 공간 선)이 붙는다.
                // 0268(사장 2026-11-22 "바닥의 격자… 좀 어떻게 못하나"): 칸마다 같은 1m 넷이면 줄눈이 바닥 전체에 곧은 격자로 이어졌다 —
                // 칸마다 칼 자리가 다른 크고 작은 판으로(`slab_cuts`).
                let slabs = if island { slab_cuts(x, z) } else { Vec::new() };
                for (off, half, i) in slabs {
                    // 땅 위의 섬(0270 A): 칸의 판 몇만 남는다(부서진 폐허).
                    if let Some(g) = floor.ground {
                        let o = c + off;
                        let inside = field(o.x, o.z, ISLAND_SCALE, 239) - (1.0 - g.islands);
                        if inside < 0.1 * hash(x, z, i + 240) || hash(x, z, i + 241) < 0.15 {
                            continue;
                        }
                    }
                    let r = hash(x, z, i);
                    let o = c + off;
                    let g = half - vec2(0.03, 0.03) * hash(x, z, i + 7);
                    if hash(x, z, i + 31) > 1.0 - floor.bare {
                        // 빠진 석판: 흙 + 풀 두 다발.
                        quad_lm(
                            &mut p,
                            [
                                on(o + vec3(-g.x, MORTAR_Y + 0.01, -g.y)),
                                on(o + vec3(g.x, MORTAR_Y + 0.01, -g.y)),
                                on(o + vec3(g.x, MORTAR_Y + 0.01, g.y)),
                                on(o + vec3(-g.x, MORTAR_Y + 0.01, g.y)),
                            ],
                            floor.dirt,
                            &uv_of,
                        );
                        for t in 0..3 {
                            let a = vec3(
                                hash(x, z, i + 40 + t) - 0.5,
                                0.0,
                                hash(x, z, i + 44 + t) - 0.5,
                            );
                            tuft(
                                &mut sides,
                                on(o + a * g.x + Vec3::Y * MORTAR_Y),
                                x * 7 + i as usize,
                                z * 3 + t as usize,
                                &lights,
                            );
                        }
                        continue;
                    }
                    let base = if hash(x / 2, z / 2, 99) > 1.0 - floor.moss && r > 0.4 {
                        floor.moss_slab
                    } else {
                        floor.slab
                    };
                    let col = if look.calm {
                        mul(base, (0.94 + 0.12 * r) * (0.90 + 0.20 * patch(x, z)))
                    } else {
                        mul(base, 0.8 + 0.35 * r)
                    };
                    // 0242(사장 설문 10-5 "전체 색이 너무 단조롭다"): 큰 얼룩(3칸 묶음)으로 따뜻한 돌(모래빛)을 섞는다 —
                    // 칸마다의 밝기 차는 키우지 않고(0140) 색의 결만 남보라 하나에서 둘로.
                    let col = if hash(x / 3, z / 3, 77) > 0.62 {
                        Color::new(col.r * 1.16, col.g * 1.02, col.b * 0.84, col.a)
                    } else {
                        col
                    };
                    let top = SLAB_LOW + (SLAB_HIGH - SLAB_LOW) * hash(x, z, i + 13);
                    // 몇은 한쪽 변이 들린다(빛 띠가 달라지고 모서리가 선다).
                    let lift = if hash(x, z, i + 17) > 0.85 {
                        SLAB_LIFT
                    } else {
                        0.0
                    };
                    let tilt_x = hash(x, z, i + 19) > 0.5;
                    let ys = |sx: f32, sz: f32| {
                        let up = if tilt_x { sx } else { sz };
                        (top + if up > 0.0 { lift } else { 0.0 }).min(0.0)
                    };
                    let corner = |sx: f32, sz: f32| on(o + vec3(sx * g.x, ys(sx, sz), sz * g.y));
                    let foot = |sx: f32, sz: f32| on(o + vec3(sx * g.x, MORTAR_Y, sz * g.y));
                    let tops = [
                        corner(-1.0, -1.0),
                        corner(1.0, -1.0),
                        corner(1.0, 1.0),
                        corner(-1.0, 1.0),
                    ];
                    quad_lm(&mut p, tops, col, &uv_of);
                    let side = mul(col, if look.calm { SLAB_SIDE_CALM } else { SLAB_SIDE });
                    for (a, b, n) in [
                        ((-1.0, -1.0), (1.0, -1.0), Vec3::NEG_Z),
                        ((1.0, -1.0), (1.0, 1.0), Vec3::X),
                        ((1.0, 1.0), (-1.0, 1.0), Vec3::Z),
                        ((-1.0, 1.0), (-1.0, -1.0), Vec3::NEG_X),
                    ] {
                        quad_lm_n(
                            &mut p,
                            [
                                foot(a.0, a.1),
                                foot(b.0, b.1),
                                corner(b.0, b.1),
                                corner(a.0, a.1),
                            ],
                            side,
                            &uv_of,
                            n,
                        );
                    }
                }
                // 벽 밑 풀(0060 W2): 벽 칸에 닿은 변마다 반쯤, 변을 따라 아무 자리에.
                for (k, (dx, dz)) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)]
                    .into_iter()
                    .enumerate()
                {
                    let (nx, nz) = (x as i32 + dx, z as i32 + dz);
                    if nx < 0 || nz < 0 || lv.cell(nx as usize, nz as usize) != b'#' {
                        continue;
                    }
                    if hash(x, z, 60 + k as u32) > 0.5 {
                        continue;
                    }
                    let along = (hash(x, z, 64 + k as u32) - 0.5) * (CELL - 0.4);
                    let d = vec3(dx as f32, 0.0, dz as f32);
                    let pos = c + d * (h - 0.15) + vec3(d.z, 0.0, d.x) * along;
                    tuft(
                        &mut sides,
                        on(pos + Vec3::Y * MORTAR_Y),
                        x * 5 + k,
                        z * 11 + k,
                        &lights,
                    );
                }
                // 옹벽과 턱 테(0043): 이웃 칸(걷는 칸·허공)보다 높으면 그 변에 이웃을 바라보는 면, 걸음보다 높으면 위 모서리에 밝은 테.
                for (nx, nz) in neighbors(x, z) {
                    let d = vec3(nx as f32 - x as f32, 0.0, nz as f32 - z as f32);
                    let e = c + d * h;
                    let side = vec3(d.z, 0.0, -d.x) * h;
                    let (a, b) = (e - side, e + side);
                    let below = |v: Vec3| -> Option<f32> {
                        if lv.walkable(nx, nz) {
                            Some(gy(nx, nz, v))
                        } else if lv.cell(nx, nz) == b' ' && lv.height(nx, nz).is_none() {
                            Some(0.0f32.min(gy(x, z, v)) - 0.5)
                        } else {
                            None // 벽·바위 단·꺼진 땅은 제 그림이 맡는다.
                        }
                    };
                    let (Some(la), Some(lb)) = (below(a), below(b)) else {
                        continue;
                    };
                    let (ha, hb) = (gy(x, z, a), gy(x, z, b));
                    if ha <= la + 1e-3 && hb <= lb + 1e-3 {
                        continue;
                    }
                    wall_face(
                        &mut sides,
                        [
                            a + Vec3::Y * la,
                            b + Vec3::Y * lb,
                            b + Vec3::Y * hb,
                            a + Vec3::Y * ha,
                        ],
                        d,
                        floor.ledge,
                        &lights,
                    );
                    if (ha - la).max(hb - lb) > GROUND_STEP {
                        let inset = -d * 0.12;
                        quad(
                            &mut sides,
                            [
                                a + Vec3::Y * (ha + 0.03),
                                b + Vec3::Y * (hb + 0.03),
                                b + inset + Vec3::Y * (hb + 0.03),
                                a + inset + Vec3::Y * (ha + 0.03),
                            ],
                            Vec3::Y,
                            floor.edge,
                            &lights,
                        );
                    }
                }
            }
            if !p.verts.is_empty() {
                pieces.push(p);
            }
            if !sides.verts.is_empty() {
                pieces.push(sides);
            }
        }
        // 벽: 칸마다 덩어리 하나(높이를 조금씩 — 무너진 성벽). 칸마다 가림 상자.
        for z in 0..lv.h {
            for x in 0..lv.w {
                if lv.cell(x, z) != b'#' {
                    continue;
                }
                let c = lv.at(x, z);
                let h = CELL * 0.5;
                let top = wall_top(x, z);
                // 곁에 꺼진 땅이 있으면 벽도 그 바닥까지 내려 그린다(0035).
                let foot = neighbors(x, z)
                    .filter_map(|(nx, nz)| {
                        (lv.cell(nx, nz) == b' ')
                            .then(|| lv.height(nx, nz))
                            .flatten()
                    })
                    .fold(0.0f32, f32::min);
                let min = vec3(c.x - h, foot, c.z - h);
                let max = vec3(c.x + h, top, c.z + h);
                let mut p = empty(Some(Aabb { min, max }));
                let shade = 0.85 + 0.25 * hash(x, z, 5);
                block(
                    &mut p,
                    min,
                    max,
                    mul(floor.wall, shade),
                    floor.wall_top,
                    &lights,
                );
                // 위에 얹힌 돌 하나(몇 칸만).
                if hash(x, z, 9) > 0.6 {
                    let s = 0.35 + 0.3 * hash(x, z, 11);
                    let o = vec3(
                        c.x + (hash(x, z, 12) - 0.5),
                        top,
                        c.z + (hash(x, z, 13) - 0.5),
                    );
                    block(
                        &mut p,
                        o - vec3(s, 0.0, s),
                        o + vec3(s, s * 0.9, s),
                        mul(floor.wall, shade * 0.9),
                        floor.wall_top,
                        &lights,
                    );
                }
                // 벽을 타는 뿌리(0063 E3): 걸을 수 있는 쪽 면에 몇 줄기 — 벽과 같은 덩어리라 같이 옅어진다.
                for (i, (nx, nz)) in neighbors(x, z).enumerate() {
                    if !flora || !lv.walkable(nx, nz) || hash(x, z, 40 + i as u32) > ROOT_SHARE {
                        continue;
                    }
                    let out = (lv.at(nx, nz) - c).normalize_or_zero();
                    let along = Vec3::Y.cross(out) * (hash(x, z, 50 + i as u32) - 0.5) * CELL * 0.6;
                    let face = c + out * h + along;
                    crate::flora::wall_root(
                        &mut p,
                        vec3(face.x, top, face.z),
                        out,
                        gnd.center(nx, nz).y,
                        (x * 131 + z * 17 + i) as u32,
                        &lights,
                    );
                }
                pieces.push(p);
            }
        }
        // 배경 높낮이(0035): 바깥 칸마다 높으면 바위 단(가림 상자 — 앞을 막으면 옅게), 낮으면 꺼진 바닥 + 이웃보다 낮은 쪽 벼랑면.
        let mut void_y: f32 = -0.03;
        for z in 0..lv.h {
            for x in 0..lv.w {
                if lv.cell(x, z) != b' ' {
                    continue;
                }
                let Some(hgt) = lv.height(x, z) else { continue };
                let c = lv.at(x, z);
                let hh = CELL * 0.5;
                if hgt > 0.0 {
                    let min = vec3(c.x - hh, 0.0, c.z - hh);
                    let max = vec3(c.x + hh, hgt, c.z + hh);
                    let mut p = empty(Some(Aabb { min, max }));
                    let shade = 0.85 + 0.25 * hash(x, z, 21);
                    block(
                        &mut p,
                        min,
                        max,
                        mul(floor.rock, shade),
                        mul(floor.rock_top, shade),
                        &lights,
                    );
                    if hash(x, z, 22) > 0.55 {
                        let s = 0.25 + 0.35 * hash(x, z, 23);
                        let o = vec3(
                            c.x + (hash(x, z, 24) - 0.5),
                            hgt,
                            c.z + (hash(x, z, 25) - 0.5),
                        );
                        block(
                            &mut p,
                            o - vec3(s, 0.0, s),
                            o + vec3(s, s, s),
                            mul(floor.rock, shade * 0.9),
                            floor.rock_top,
                            &lights,
                        );
                    }
                    // 식생(0063 E3): 바위 단 위에 나무 몇 그루·덤불. 나무는 가림 상자를 넓혀 앞을 막으면 같이 옅어진다.
                    let on = vec3(
                        c.x + (hash(x, z, 32) - 0.5) * CELL * 0.5,
                        hgt,
                        c.z + (hash(x, z, 33) - 0.5) * CELL * 0.5,
                    );
                    let seed = (x * 977 + z * 131) as u32;
                    // 빽빽한 숲(0262): 걷는 땅 바로 곁(8방향) 칸은 덤불만.
                    let by_path = woods.dense()
                        && (-1i32..=1).any(|dz| {
                            (-1i32..=1).any(|dx| {
                                let (nx, nz) = (x as i32 + dx, z as i32 + dz);
                                nx >= 0 && nz >= 0 && lv.walkable(nx as usize, nz as usize)
                            })
                        });
                    if !flora || woods.share <= 0.0 {
                    } else if !by_path && hash(x, z, 30) < woods.share {
                        let b = crate::flora::tree(&mut p, on, seed, woods.live, &lights);
                        if let Some(a) = p.aabb.as_mut() {
                            a.min = a.min.min(b.min);
                            a.max = a.max.max(b.max);
                        }
                    } else if hash(x, z, 31) < BUSH_SHARE {
                        let size = 0.5 + 0.4 * hash(x, z, 34);
                        crate::flora::bush(&mut p, on, seed, size, woods.live, &lights);
                    }
                    pieces.push(p);
                } else {
                    void_y = void_y.min(hgt - 0.5);
                    let mut p = empty(None);
                    let k = 0.7 + 0.3 * hash(x, z, 26);
                    quad(
                        &mut p,
                        [
                            vec3(c.x - hh, hgt, c.z - hh),
                            vec3(c.x + hh, hgt, c.z - hh),
                            vec3(c.x + hh, hgt, c.z + hh),
                            vec3(c.x - hh, hgt, c.z + hh),
                        ],
                        Vec3::Y,
                        mul(floor.pit, k),
                        &lights,
                    );
                    // 떨어진 돌덩이 몇 개(깊이 단서).
                    if hash(x, z, 27) > 0.5 {
                        let s = 0.2 + 0.3 * hash(x, z, 28);
                        let o = vec3(
                            c.x + (hash(x, z, 29) - 0.5) * 1.2,
                            hgt,
                            c.z + (hash(x, z, 30) - 0.5) * 1.2,
                        );
                        block(
                            &mut p,
                            o - vec3(s, 0.0, s),
                            o + vec3(s, s * 0.8, s),
                            mul(floor.wall, 0.6 * k),
                            mul(floor.wall_top, k),
                            &lights,
                        );
                    }
                    // 벼랑면: 이웃이 더 높으면(걷는 바닥 0·덜 꺼진 땅) 그 경계에 이 칸을 바라보는 면.
                    for (nx, nz) in neighbors(x, z) {
                        let top = match lv.cell(nx, nz) {
                            b' ' => match lv.height(nx, nz) {
                                Some(v) => v.min(0.0),
                                None => continue,
                            },
                            _ if lv.walkable(nx, nz) => gnd.center(nx, nz).y,
                            _ => 0.0,
                        };
                        if top <= hgt + 1e-3 {
                            continue;
                        }
                        let d = vec3(nx as f32 - x as f32, 0.0, nz as f32 - z as f32);
                        let e = c + d * hh;
                        let side = vec3(d.z, 0.0, -d.x) * hh;
                        // 위 모서리는 벽돌빛 그대로, 바닥 쪽은 어둡게 — 깊이가 읽히게.
                        let base = p.verts.len() as u16;
                        let corners = [
                            e - side + Vec3::Y * hgt,
                            e + side + Vec3::Y * hgt,
                            e + side + Vec3::Y * top,
                            e - side + Vec3::Y * top,
                        ];
                        for (i, v) in corners.iter().enumerate() {
                            let l = lit(*v, -d, &lights) * if i < 2 { 0.3 } else { 1.0 };
                            let col = mul(floor.wall, 0.9 * k);
                            p.verts.push(with_normal(
                                Vertex::new(
                                    v.x,
                                    v.y,
                                    v.z,
                                    0.0,
                                    0.0,
                                    Color::new(col.r * l.x, col.g * l.y, col.b * l.z, 1.0),
                                ),
                                -d,
                            ));
                        }
                        p.idx.extend_from_slice(&[
                            base,
                            base + 1,
                            base + 2,
                            base,
                            base + 2,
                            base + 3,
                        ]);
                    }
                    pieces.push(p);
                }
            }
        }
        // 소품: 모델이 있으면 첫 자세를 굽고, 없으면 상자.
        let mut pose = Vec::new();
        for pr in &lv.props {
            let half = pr.kind.half();
            let hh = pr.kind.height();
            let aabb = (half != Vec2::ZERO).then(|| Aabb {
                min: vec3(pr.pos.x - half.x, pr.pos.y, pr.pos.z - half.y),
                max: vec3(pr.pos.x + half.x, pr.pos.y + hh, pr.pos.z + half.y),
            });
            let mut p = empty(aabb);
            if let Some(e) = lib.get(pr.kind.model()) {
                e.model.pose(e.pick(&["idle"]), 0.0, &mut pose);
                let inst = Mat4::from_translation(pr.pos) * Mat4::from_rotation_y(pr.yaw);
                if matches!(pr.kind, PropKind::Brazier | PropKind::Hearth) && cold(pr.pos) {
                    Batch::bake_model_dark(
                        &mut p.verts,
                        &mut p.idx,
                        e,
                        &pose,
                        inst,
                        WHITE,
                        &lights,
                    );
                } else {
                    Batch::bake_model(&mut p.verts, &mut p.idx, e, &pose, inst, WHITE, &lights);
                }
                p.tex = Some(e.tex.clone());
            } else {
                let s = if half == Vec2::ZERO {
                    vec2(0.4, 0.4)
                } else {
                    half
                };
                block(
                    &mut p,
                    vec3(pr.pos.x - s.x, pr.pos.y, pr.pos.z - s.y),
                    vec3(pr.pos.x + s.x, pr.pos.y + hh, pr.pos.z + s.y),
                    prop_color(pr.kind),
                    floor.wall_top,
                    &lights,
                );
            }
            // 가림 상자(0094) = 구운 정점의 실제 범위 — 부딪힘 상자(종류별 반폭)는 회전(yaw)을 모르고 모델보다 좁을 수 있다
            // (벽 조각 깊이 0.4 대 0.83m, 부러진 기둥 조각 0.6 대 1.07m). 지나가는 소품(돌무더기)도 가리면 옅어진다.
            p.aabb = vert_bounds(&p.verts).or(p.aabb);
            pieces.push(p);
        }
        // 쓰러진 통나무(0262): 정점마다 매끈한 법선의 관(`flora::log`) — 식생을 꺼도 굽는다(뛰어넘는 턱의 그림이다).
        for (a, b, seed) in logs {
            let mut p = empty(None);
            crate::flora::log(&mut p, a, b, LOG_R, seed, &lights);
            p.aabb = vert_bounds(&p.verts);
            pieces.push(p);
        }
        crate::render::set_bake_shade(None);
        let bounds = pieces
            .iter()
            .map(|p| {
                let (lo, hi) = crate::render::bounds(p.verts.iter().map(|v| v.position));
                (lo - Vec3::splat(SWAY_PAD), hi + Vec3::splat(SWAY_PAD))
            })
            .collect();
        Scenery {
            bounds,
            pieces,
            occ: std::rc::Rc::new(occ),
            lights,
            extent: lv.half_extent() * 3.0,
            void_y,
            walk_min: walk_min(&lv.ground()),
        }
    }
}

/// 정점들을 감싸는 상자(없으면 없음).
fn vert_bounds(v: &[Vertex]) -> Option<Aabb> {
    let mut it = v.iter().map(|q| q.position);
    let first = it.next()?;
    let (mut lo, mut hi) = (first, first);
    for q in it {
        lo = lo.min(q);
        hi = hi.max(q);
    }
    Some(Aabb { min: lo, max: hi })
}

/// 걸을 수 있는 칸 가운데 가장 낮은 땅(경사로는 낮은 끝). 걸을 칸이 없으면 0.
fn walk_min(g: &Ground) -> f32 {
    use crate::ground::Cell;
    g.cells
        .iter()
        .filter_map(|c| match *c {
            Cell::Solid => None,
            Cell::Flat(h) => Some(h),
            Cell::Ramp { from, to, .. } => Some(from.min(to)),
        })
        .reduce(f32::min)
        .unwrap_or(0.0)
}

/// 빛 지도를 입은 세운 면(석판 옆면, 0060 W2): 법선을 주는 판.
fn quad_lm_n(p: &mut Piece, corners: [Vec3; 4], col: Color, uv: &dyn Fn(Vec3) -> Vec2, n: Vec3) {
    let base = p.verts.len() as u16;
    let c2 = Color::new(
        (col.r * 2.0).min(1.0),
        (col.g * 2.0).min(1.0),
        (col.b * 2.0).min(1.0),
        col.a,
    );
    for c in corners {
        let t = uv(c);
        p.verts
            .push(with_normal(Vertex::new(c.x, c.y, c.z, t.x, t.y, c2), n));
    }
    p.idx
        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

/// 풀 한 다발(0060 W2, t3ssel8r 풍): 굵은 세모 잎 일곱(화면에서 1px = 약 3.5cm라 잎 폭 10cm·키 30~50cm) — 밑동은 어둡고 끝은 밝게, 법선은 위(바닥과 같은 빛, 잎끼리 선이 서지 않게).
/// `sx`·`sz`는 모양을 고르는 씨앗(같은 지도 = 같은 풀).
fn tuft(p: &mut Piece, at: Vec3, sx: usize, sz: usize, lights: &[Light]) {
    for b in 0..7u32 {
        let ang = std::f32::consts::TAU * (b as f32 / 7.0 + hash(sx, sz, 200 + b) * 0.3);
        let (s, c) = ang.sin_cos_d();
        let dir = vec3(c, 0.0, s);
        let side = vec3(-s, 0.0, c) * 0.07;
        let r = 0.03 + 0.12 * hash(sx, sz, 210 + b);
        // 0242: 키 30~50cm → 40~65cm(쿼터뷰에서 바닥 결이 아니라 풀로 읽히게).
        let hgt = 0.4 + 0.25 * hash(sx, sz, 220 + b);
        let lean = 0.08 + 0.12 * hash(sx, sz, 230 + b);
        let foot = at + dir * r;
        let tip = foot + dir * lean + Vec3::Y * hgt;
        let base = p.verts.len() as u16;
        for (q, col) in [
            (foot - side, GRASS_ROOT),
            (foot + side, GRASS_ROOT),
            (tip, GRASS_TIP),
        ] {
            let l = lit(q, Vec3::Y, lights);
            // 풀 끝만 바람에 흔들린다(밑동은 땅에 박혀 있다, 0142).
            let tip_sway = if col == GRASS_TIP { 1.0 } else { 0.0 };
            let mut v = with_normal(
                Vertex::new(
                    q.x,
                    q.y,
                    q.z,
                    0.0,
                    0.0,
                    Color::new(col.r * l.x, col.g * l.y, col.b * l.z, 1.0),
                ),
                Vec3::Y,
            );
            sway(&mut v, tip_sway);
            p.verts.push(v);
        }
        p.idx.extend_from_slice(&[base, base + 1, base + 2]);
    }
}

/// 빛 지도를 입은 바닥 판(0044): 정점 색 = 바탕색 × 2(빛 지도가 빛의 반을 담는다), UV = 월드 XZ.
/// `quad_lm`에 꼭짓점마다의 색(땅의 얼룩 — 0270 A).
fn quad_lm_cols(p: &mut Piece, corners: [Vec3; 4], cols: [Color; 4], uv: &dyn Fn(Vec3) -> Vec2) {
    let base = p.verts.len() as u16;
    let mut n = (corners[1] - corners[0])
        .cross(corners[2] - corners[0])
        .normalize_or(Vec3::Y);
    if n.y < 0.0 {
        n = -n;
    }
    for (c, col) in corners.into_iter().zip(cols) {
        let t = uv(c);
        let c2 = Color::new(
            (col.r * 2.0).min(1.0),
            (col.g * 2.0).min(1.0),
            (col.b * 2.0).min(1.0),
            col.a,
        );
        p.verts
            .push(with_normal(Vertex::new(c.x, c.y, c.z, t.x, t.y, c2), n));
    }
    p.idx
        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

/// 월드 자리의 매끈한 0..1 값(`scale` m 격자의 `hash`를 부드럽게 이음 — 칸과 상관없이 이어진다, 0270 A).
fn field(x: f32, z: f32, scale: f32, k: u32) -> f32 {
    let (fx, fz) = (x / scale + 1000.0, z / scale + 1000.0);
    let (ix, iz) = (fx.floor(), fz.floor());
    let s = |t: f32| t * t * (3.0 - 2.0 * t);
    let (tx, tz) = (s(fx - ix), s(fz - iz));
    let h = |dx: usize, dz: usize| hash(ix as usize + dx, iz as usize + dz, k);
    let a = h(0, 0) + (h(1, 0) - h(0, 0)) * tx;
    let b = h(0, 1) + (h(1, 1) - h(0, 1)) * tx;
    a + (b - a) * tz
}

/// 땅의 색(0270 A): 흙에 자갈 빛을 조금 섞고, 큰 얼룩으로 이끼(몫 `moss_amt` 남짓), 작은 얼룩으로 밝기 — 칸마다 튀지 않는다(0140).
fn ground_color(f: &FloorLook, g: &Soil, v: Vec3) -> Color {
    let lerp = |a: Color, b: Color, t: f32| {
        Color::new(
            a.r + (b.r - a.r) * t,
            a.g + (b.g - a.g) * t,
            a.b + (b.b - a.b) * t,
            1.0,
        )
    };
    let base = lerp(f.dirt, g.gravel, 0.3 * field(v.x, v.z, 3.0, 253));
    let t = field(v.x, v.z, 5.5, 251);
    let m = ((t - (1.0 - g.moss_amt)) / 0.18 + 0.5).clamp(0.0, 1.0);
    let m = m * m * (3.0 - 2.0 * m);
    mul(lerp(base, g.moss, m), 0.9 + 0.2 * field(v.x, v.z, 1.6, 252))
}

fn quad_lm(p: &mut Piece, corners: [Vec3; 4], col: Color, uv: &dyn Fn(Vec3) -> Vec2) {
    let base = p.verts.len() as u16;
    let c2 = Color::new(
        (col.r * 2.0).min(1.0),
        (col.g * 2.0).min(1.0),
        (col.b * 2.0).min(1.0),
        col.a,
    );
    // 법선은 판 모서리에서(경사로는 기운다) — 위를 보게.
    let mut n = (corners[1] - corners[0])
        .cross(corners[2] - corners[0])
        .normalize_or(Vec3::Y);
    if n.y < 0.0 {
        n = -n;
    }
    for c in corners {
        let t = uv(c);
        p.verts
            .push(with_normal(Vertex::new(c.x, c.y, c.z, t.x, t.y, c2), n));
    }
    p.idx
        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

/// 정점에 면 법선을 싣는다(0061 화면 공간 선 — 법선 패스가 읽는다, macroquad 기본 셰이더는 안 읽는 칸).
pub fn with_normal(mut v: Vertex, n: Vec3) -> Vertex {
    v.normal = n.normalize_or(Vec3::Y).extend(1.0);
    v
}

/// 바람에 흔들리는 몫(0142)을 정점에 싣는다 — 법선 칸의 넷째 값: 1 = 가만히, 1 + `s` = 바람에 `s`만큼(풀 끝 1·잎 무더기 0.3),
/// 1 − `f` = 불꽃처럼 빠르게 일렁임(`f` 0..0.9). 지형 재질(`render` 바람 셰이더)과 법선 패스가 같은 식으로 읽는다.
pub fn sway(v: &mut Vertex, s: f32) {
    v.normal.w = 1.0 + s;
}

fn to_u8(x: f32) -> u8 {
    (x.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// 칸 바닥의 법선(경사로는 기운다).
fn slope_normal(g: &Ground, x: usize, z: usize) -> Vec3 {
    match g.at(x, z) {
        crate::ground::Cell::Ramp { x_axis, from, to } => {
            let s = (to - from) / CELL;
            if x_axis {
                vec3(-s, 1.0, 0.0).normalize()
            } else {
                vec3(0.0, 1.0, -s).normalize()
            }
        }
        _ => Vec3::Y,
    }
}

/// 세운 면 하나(바깥 `out` 쪽을 본다): 아래 두 점이 어둡고 위가 밝게 — 깊이가 읽히게(꺼진 땅 벼랑과 같은 식).
fn wall_face(p: &mut Piece, c: [Vec3; 4], out: Vec3, col: Color, lights: &[Light]) {
    let base = p.verts.len() as u16;
    for (i, v) in c.iter().enumerate() {
        let l = lit(*v, out, lights) * if i < 2 { 0.55 } else { 1.0 };
        p.verts.push(with_normal(
            Vertex::new(
                v.x,
                v.y,
                v.z,
                0.0,
                0.0,
                Color::new(col.r * l.x, col.g * l.y, col.b * l.z, 1.0),
            ),
            out,
        ));
    }
    p.idx.extend_from_slice(&[
        base,
        base + 1,
        base + 2,
        base,
        base + 2,
        base + 3,
        base,
        base + 2,
        base + 1,
        base,
        base + 3,
        base + 2,
    ]);
}

/// 위·아래·왼·오른 이웃 칸(지도 밖은 뺀다 — `cell`이 바깥을 ' '로 돌려주지만 높이가 없어 건너뛴다).
fn neighbors(x: usize, z: usize) -> impl Iterator<Item = (usize, usize)> {
    [(0i32, -1i32), (0, 1), (-1, 0), (1, 0)]
        .into_iter()
        .filter_map(move |(dx, dz)| {
            let (nx, nz) = (x as i32 + dx, z as i32 + dz);
            (nx >= 0 && nz >= 0).then_some((nx as usize, nz as usize))
        })
}

fn prop_color(k: PropKind) -> Color {
    match k {
        PropKind::Pillar | PropKind::WallPiece | PropKind::Rubble => {
            Color::new(0.38, 0.36, 0.34, 1.0)
        }
        PropKind::Root => Color::new(0.30, 0.20, 0.25, 1.0),
        PropKind::Brazier | PropKind::Hearth => Color::new(0.9, 0.6, 0.25, 1.0),
        PropKind::Bell => Color::new(0.45, 0.42, 0.30, 1.0),
        PropKind::Arch => Color::new(0.38, 0.36, 0.34, 1.0),
    }
}

/// 빛 지도 흐리기(0242): 가로·세로 [1 2 1]을 걷는 칸(투명도 255)끼리만. 걷지 못하는 칸은 그대로 0.
fn soften_lightmap(lm: &[u8], w: usize, h: usize) -> Vec<u8> {
    let pass = |src: &[u8], dx: usize, dy: usize| -> Vec<u8> {
        let mut out = src.to_vec();
        for j in 0..h {
            for i in 0..w {
                let k = (j * w + i) * 4;
                if src[k + 3] == 0 {
                    continue;
                }
                let mut sum = [0u32; 3];
                let mut wt = 0u32;
                let taps = [
                    (i.checked_sub(dx), j.checked_sub(dy), 1u32),
                    (Some(i), Some(j), 2),
                    (Some(i + dx), Some(j + dy), 1),
                ];
                for (x, y, f) in taps {
                    let (Some(x), Some(y)) = (x, y) else { continue };
                    if x >= w || y >= h {
                        continue;
                    }
                    let q = (y * w + x) * 4;
                    if src[q + 3] == 0 {
                        continue;
                    }
                    for c in 0..3 {
                        sum[c] += src[q + c] as u32 * f;
                    }
                    wt += f;
                }
                for c in 0..3 {
                    out[k + c] = ((sum[c] + wt / 2) / wt) as u8;
                }
            }
        }
        out
    };
    let a = pass(lm, 1, 0);
    let mut out = pass(&a, 0, 1);
    // 선형 거르기는 이웃 칸을 섞는다: 걷지 못하는 칸이 검정·투명(바닥 셰이더는 빛 지도의 투명도도 곱한다)이면 바닥 끝이
    // 어둡고 반투명해진다 — 걷는 칸에 닿은 걷지 못하는 칸은 이웃 걷는 칸의 빛으로 채우고 불투명하게.
    let src = out.clone();
    for j in 0..h {
        for i in 0..w {
            let k = (j * w + i) * 4;
            if src[k + 3] != 0 {
                continue;
            }
            let (mut sum, mut n) = ([0u32; 3], 0u32);
            for (dx, dy) in [
                (-1i32, 0i32),
                (1, 0),
                (0, -1),
                (0, 1),
                (-1, -1),
                (1, -1),
                (-1, 1),
                (1, 1),
            ] {
                let (x, y) = (i as i32 + dx, j as i32 + dy);
                if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
                    continue;
                }
                let q = (y as usize * w + x as usize) * 4;
                if src[q + 3] == 0 {
                    continue;
                }
                for c in 0..3 {
                    sum[c] += src[q + c] as u32;
                }
                n += 1;
            }
            for c in 0..3 {
                if let Some(v) = sum[c].checked_div(n) {
                    out[k + c] = v as u8;
                    out[k + 3] = 255;
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn softening_the_light_map_blurs_edges_but_never_pulls_in_unwalkable_black() {
        // 3×1: 밝은 칸 · 어두운 칸 · 걷지 못하는 칸(검정, 투명도 0).
        let lm = [200, 200, 200, 255, 40, 40, 40, 255, 0, 0, 0, 0];
        let out = soften_lightmap(&lm, 3, 1);
        assert!(
            out[0] < 200 && out[0] > 120,
            "밝은 쪽 경계가 조금 어두워진다: {}",
            out[0]
        );
        assert!(
            out[4] > 40 && out[4] < 120,
            "어두운 쪽 경계가 조금 밝아진다: {}",
            out[4]
        );
        // 걷지 못하는 칸의 검정은 어두운 칸에 섞이지 않는다(두 칸만의 평균 = (200 + 2·40)/3 ≈ 93),
        // 그 칸은 이웃 걷는 칸의 빛으로 채워 불투명하게 — 선형 거르기에서 바닥 끝이 검게·반투명해지지 않게.
        assert_eq!(&out[8..12], &[out[4], out[5], out[6], 255]);
        assert_eq!(out[4], 93);
    }
    use super::*;

    /// 0268: 칸의 석판들은 칸 안에서 겹치지 않고 줄눈만 남기고 덮으며(넓이), 너무 가는 판이 없고, 이웃 칸과 칼 자리가 대개 다르다
    /// (곧은 줄눈이 칸을 넘어 이어지지 않게) — 1m 넷의 격자가 아니다.
    #[test]
    fn slabs_tile_each_cell_with_offset_cuts() {
        let (mut same, mut total) = (0, 0);
        for x in 0..30 {
            for z in 0..30 {
                let sl = slab_cuts(x, z);
                assert!((2..=8).contains(&sl.len()), "{x},{z}: {}", sl.len());
                let mut area = 0.0;
                for (i, (c, h, _)) in sl.iter().enumerate() {
                    assert!(h.x >= 0.15 && h.y >= 0.15, "thin slab {h:?}");
                    assert!(
                        c.x.abs() + h.x <= 1.0 && c.z.abs() + h.y <= 1.0,
                        "inside the cell"
                    );
                    area += (h.x + 0.05) * (h.y + 0.05) * 4.0;
                    for (d, g, _) in &sl[i + 1..] {
                        let apart =
                            (c.x - d.x).abs() >= h.x + g.x || (c.z - d.z).abs() >= h.y + g.y;
                        assert!(apart, "overlap at {x},{z}");
                    }
                }
                assert!((area - 4.0).abs() < 1e-3, "covers the cell: {area}");
                // 이웃(오른쪽) 칸과 판 모양이 같은가 — 같은 칼 자리가 이어지면 격자가 된다.
                let key = |v: &Vec<(Vec3, Vec2, u32)>| {
                    let mut k: Vec<(i32, i32)> = v
                        .iter()
                        .map(|(c, _, _)| ((c.x * 20.0) as i32, (c.z * 20.0) as i32))
                        .collect();
                    k.sort();
                    k
                };
                total += 1;
                same += (key(&sl) == key(&slab_cuts(x + 1, z))) as u32;
            }
        }
        assert!(
            same * 10 < total,
            "neighbours mostly cut differently: {same}/{total}"
        );
    }

    /// 0140 조용한 바닥: 석판 색의 큰 얼룩은 0..1 안이고 이웃 칸끼리는 조금씩만 바뀐다(칸마다 튀면 다시 격자 잡음이 된다),
    /// 줄눈은 석판의 반보다 밝다(예전 0.3배 — 화면에서 가장 센 대비가 바닥이었다).
    #[test]
    fn the_calm_floor_varies_in_big_patches_and_keeps_its_joints_soft() {
        let mut spread = 0.0f32;
        for z in 0..60 {
            for x in 0..60 {
                let a = patch(x, z);
                assert!((0.0..=1.0).contains(&a));
                assert!((a - patch(x + 1, z)).abs() < 0.3 && (a - patch(x, z + 1)).abs() < 0.3);
                spread = spread.max((a - patch(30, 30)).abs());
            }
        }
        assert!(spread > 0.3, "얼룩이 거의 없다: {spread}");
        for f in [SANCTUARY_FLOOR, VILLAGE_FLOOR, GARDEN_FLOOR, WORKS_FLOOR] {
            assert!(f.mortar_calm.g / f.slab.g > 0.5 && f.mortar.g / f.slab.g < 0.35);
        }
    }

    /// 0142 바람: 풀 다발은 끝만 흔들린다(밑동은 땅에 박혀 있다) — 바람 몫은 법선 칸의 넷째 값, 1 = 가만히.
    #[test]
    fn grass_sways_at_the_tip_and_stays_rooted_at_the_foot() {
        let mut p = Piece {
            verts: Vec::new(),
            idx: Vec::new(),
            tex: None,
            aabb: None,
            fade: 1.0,
            lightmap: false,
        };
        let at = vec3(4.0, 1.0, -3.0);
        tuft(&mut p, at, 4, 3, &[]);
        let (mut tips, mut feet) = (0, 0);
        for v in &p.verts {
            if v.normal.w > 1.5 {
                tips += 1;
                assert!(
                    v.position.y > at.y + 0.25,
                    "a swaying vertex near the ground"
                );
            } else {
                feet += 1;
                assert_eq!(v.normal.w, 1.0);
                assert!(v.position.y < at.y + 0.01, "a still vertex up in the air");
            }
        }
        assert!(tips >= 7 && feet >= 14, "{tips} tips, {feet} feet");
        // 도우미: 몫은 1에 더한다(음수 = 불꽃의 일렁임).
        let mut v = Vertex::new(0.0, 0.0, 0.0, 0.0, 0.0, WHITE);
        sway(&mut v, -0.5);
        assert_eq!(v.normal.w, 0.5);
    }

    /// 0094: 소품의 가림 상자는 구운 정점을 다 감싼다(돌린 모델·상자보다 큰 모델도).
    #[test]
    fn prop_occlusion_box_wraps_every_baked_vertex() {
        assert_eq!(vert_bounds(&[]), None);
        let v: Vec<Vertex> = [
            vec3(1.0, 0.0, -2.0),
            vec3(-0.5, 3.0, 0.8),
            vec3(0.2, 1.0, 0.0),
        ]
        .iter()
        .map(|p| Vertex::new(p.x, p.y, p.z, 0.0, 0.0, WHITE))
        .collect();
        let b = vert_bounds(&v).unwrap();
        assert_eq!(b.min, vec3(-0.5, 0.0, -2.0));
        assert_eq!(b.max, vec3(1.0, 3.0, 0.8));
    }
}
