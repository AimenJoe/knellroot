//! 스테이지 그림(0019): 레벨(`stage::Level`)을 한 번 굽는다 — 바닥 석판·벽 덩어리·소품 모델을 정점으로 만들고 빛(달빛 + 화로)을
//! 정점 색에 미리 곱해 둔다(움직이지 않으니 매 프레임 다시 셀 까닭이 없다). 기본 셰이더가 법선을 안 읽어 빛은 정점 색이 전부다
//! (KB reference/macroquad/3d-rendering.md).
//!
//! 가림: 벽 칸·소품마다 상자를 들고 있어, 카메라와 플레이어 사이를 막으면 그 덩어리만 옅게 그린다(0008의 기둥 옅어짐 그대로).
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
}

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

// 0037 "색 = 생명": 세계는 해 질 녘 남보라(살아 있는 것만 파스텔·불빛만 따뜻하게) — 옛 숯빛·돌빛에서.
const MORTAR: Color = Color::new(0.09, 0.09, 0.14, 1.0);
const SLAB: Color = Color::new(0.31, 0.30, 0.41, 1.0);
const SLAB_MOSS: Color = Color::new(0.21, 0.31, 0.31, 1.0);
const WALL: Color = Color::new(0.37, 0.36, 0.48, 1.0);
const WALL_TOP: Color = Color::new(0.23, 0.22, 0.32, 1.0);
/// 배경 높낮이(0035): 바위 단 옆면·윗면(이끼 낀 흙), 꺼진 땅 바닥(어둡게).
const ROCK: Color = Color::new(0.27, 0.26, 0.36, 1.0);
const ROCK_TOP: Color = Color::new(0.19, 0.27, 0.27, 1.0);
const PIT: Color = Color::new(0.12, 0.12, 0.19, 1.0);
/// 걸을 수 있는 층의 옹벽(0043)과 턱 가장자리의 밝은 테(쿼터뷰에서 턱이 읽히게 — 0035 "다음"의 플레이 후기).
const LEDGE: Color = Color::new(0.33, 0.32, 0.44, 1.0);
const RIM: Color = Color::new(0.62, 0.60, 0.70, 1.0);

fn mul(c: Color, k: f32) -> Color {
    Color::new(c.r * k, c.g * k, c.b * k, c.a)
}

fn quad(p: &mut Piece, corners: [Vec3; 4], n: Vec3, col: Color, lights: &[Light]) {
    let base = p.verts.len() as u16;
    for c in corners {
        let l = lit(c, n, lights);
        p.verts.push(Vertex::new(
            c.x,
            c.y,
            c.z,
            0.0,
            0.0,
            Color::new(col.r * l.x, col.g * l.y, col.b * l.z, col.a),
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
            p.verts.push(Vertex::new(
                v.x,
                v.y,
                v.z,
                0.0,
                0.0,
                Color::new(side.r * l.x, side.g * l.y, side.b * l.z, 1.0),
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
    pub fn bake(lv: &Level, lib: &mut Library) -> Scenery {
        let lights = lv.lights.clone();
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
        // 달빛 가림 높이 지도(0044): 0.25m 칸마다 가장 높은 단단한 것.
        let res = 0.25;
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
                let l = lit(p, slope_normal(&gnd, x, z), &lights) * 0.5;
                let k = (j * ow + i) * 4;
                lm[k] = to_u8(l.x);
                lm[k + 1] = to_u8(l.y);
                lm[k + 2] = to_u8(l.z);
                lm[k + 3] = 255;
            }
        }
        let lightmap = Texture2D::from_rgba8(ow as u16, oh as u16, &lm);
        lightmap.set_filter(FilterMode::Nearest);
        let uv_of = |v: Vec3| {
            vec2(
                (v.x - origin.x) / (ow as f32 * res),
                (v.z - origin.y) / (oh as f32 * res),
            )
        };
        // 바닥: 칸마다 1m 석판 넷(틈 = 회반죽 판), 몇 칸은 이끼. 땅 높이를 따른다(경사로는 기운다, 0043). 정점 한도 아래로 줄마다 덩어리.
        // 빛은 빛 지도 텍스처(0044) — 바닥 판은 색만(두 배), 옹벽·테는 정점 빛(따로 덩어리).
        let gy = |x: usize, z: usize, p: Vec3| gnd.height_in(x, z, p).unwrap_or(0.0);
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
                        on(c + vec3(-h, 0.0, -h)),
                        on(c + vec3(h, 0.0, -h)),
                        on(c + vec3(h, 0.0, h)),
                        on(c + vec3(-h, 0.0, h)),
                    ],
                    MORTAR,
                    &uv_of,
                );
                for (i, (sx, sz)) in [(-0.5, -0.5), (0.5, -0.5), (-0.5, 0.5), (0.5, 0.5)]
                    .into_iter()
                    .enumerate()
                {
                    let r = hash(x, z, i as u32);
                    let base = if hash(x / 2, z / 2, 99) > 0.8 && r > 0.4 {
                        SLAB_MOSS
                    } else {
                        SLAB
                    };
                    let col = mul(base, 0.8 + 0.35 * r);
                    let o = c + vec3(sx, 0.012 + 0.01 * r, sz);
                    let g = 0.46 - 0.03 * hash(x, z, i as u32 + 7);
                    quad_lm(
                        &mut p,
                        [
                            on(o + vec3(-g, 0.0, -g)),
                            on(o + vec3(g, 0.0, -g)),
                            on(o + vec3(g, 0.0, g)),
                            on(o + vec3(-g, 0.0, g)),
                        ],
                        col,
                        &uv_of,
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
                        LEDGE,
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
                            RIM,
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
                block(&mut p, min, max, mul(WALL, shade), WALL_TOP, &lights);
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
                        mul(WALL, shade * 0.9),
                        WALL_TOP,
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
                        mul(ROCK, shade),
                        mul(ROCK_TOP, shade),
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
                            mul(ROCK, shade * 0.9),
                            ROCK_TOP,
                            &lights,
                        );
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
                        mul(PIT, k),
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
                            mul(WALL, 0.6 * k),
                            mul(WALL_TOP, k),
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
                            let col = mul(WALL, 0.9 * k);
                            p.verts.push(Vertex::new(
                                v.x,
                                v.y,
                                v.z,
                                0.0,
                                0.0,
                                Color::new(col.r * l.x, col.g * l.y, col.b * l.z, 1.0),
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
                Batch::bake_model(&mut p.verts, &mut p.idx, e, &pose, inst, WHITE, &lights);
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
                    WALL_TOP,
                    &lights,
                );
            }
            pieces.push(p);
        }
        crate::render::set_bake_shade(None);
        Scenery {
            pieces,
            lights,
            extent: lv.half_extent() * 3.0,
            void_y,
        }
    }
}

/// 빛 지도를 입은 바닥 판(0044): 정점 색 = 바탕색 × 2(빛 지도가 빛의 반을 담는다), UV = 월드 XZ.
fn quad_lm(p: &mut Piece, corners: [Vec3; 4], col: Color, uv: &dyn Fn(Vec3) -> Vec2) {
    let base = p.verts.len() as u16;
    let c2 = Color::new(
        (col.r * 2.0).min(1.0),
        (col.g * 2.0).min(1.0),
        (col.b * 2.0).min(1.0),
        col.a,
    );
    for c in corners {
        let t = uv(c);
        p.verts.push(Vertex::new(c.x, c.y, c.z, t.x, t.y, c2));
    }
    p.idx
        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
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
        p.verts.push(Vertex::new(
            v.x,
            v.y,
            v.z,
            0.0,
            0.0,
            Color::new(col.r * l.x, col.g * l.y, col.b * l.z, 1.0),
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
        PropKind::Brazier => Color::new(0.9, 0.6, 0.25, 1.0),
        PropKind::Bell => Color::new(0.45, 0.42, 0.30, 1.0),
    }
}
