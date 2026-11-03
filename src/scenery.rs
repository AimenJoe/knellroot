//! 스테이지 그림(0019): 레벨(`stage::Level`)을 한 번 굽는다 — 바닥 석판·벽 덩어리·소품 모델을 정점으로 만들고 빛(달빛 + 화로)을
//! 정점 색에 미리 곱해 둔다(움직이지 않으니 매 프레임 다시 셀 까닭이 없다). 기본 셰이더가 법선을 안 읽어 빛은 정점 색이 전부다
//! (KB reference/macroquad/3d-rendering.md).
//!
//! 가림: 벽 칸·소품마다 상자를 들고 있어, 카메라와 플레이어 사이를 막으면 그 덩어리만 옅게 그린다(0008의 기둥 옅어짐 그대로).
use macroquad::prelude::*;

use crate::models::Library;
use crate::render::{lit, Batch};
use crate::stage::{Level, Light, PropKind, CELL, WALL_H};
use crate::world::Aabb;

/// 굽힌 덩어리 하나(정점은 이미 빛을 받았다).
pub struct Piece {
    pub verts: Vec<Vertex>,
    pub idx: Vec<u16>,
    pub tex: Option<Texture2D>,
    /// 가림 판정 상자(None이면 가리지 않는다 — 바닥).
    pub aabb: Option<Aabb>,
    pub fade: f32,
}

pub struct Scenery {
    pub pieces: Vec<Piece>,
    pub lights: Vec<Light>,
    /// 바깥 허공 판의 반폭.
    pub extent: f32,
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

const MORTAR: Color = Color::new(0.09, 0.085, 0.09, 1.0);
const SLAB: Color = Color::new(0.30, 0.29, 0.28, 1.0);
const SLAB_MOSS: Color = Color::new(0.20, 0.25, 0.18, 1.0);
const WALL: Color = Color::new(0.33, 0.31, 0.30, 1.0);
const WALL_TOP: Color = Color::new(0.20, 0.19, 0.19, 1.0);

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
    }
}

impl Scenery {
    pub fn bake(lv: &Level, lib: &mut Library) -> Scenery {
        let lights = lv.lights.clone();
        let mut pieces = Vec::new();
        // 바닥: 칸마다 1m 석판 넷(틈 = 회반죽 판), 몇 칸은 이끼. 정점 한도 아래로 줄마다 덩어리.
        for z in 0..lv.h {
            let mut p = empty(None);
            for x in 0..lv.w {
                if lv.cell(x, z) != b'.' {
                    continue;
                }
                let c = lv.at(x, z);
                let h = CELL * 0.5;
                quad(
                    &mut p,
                    [
                        c + vec3(-h, 0.0, -h),
                        c + vec3(h, 0.0, -h),
                        c + vec3(h, 0.0, h),
                        c + vec3(-h, 0.0, h),
                    ],
                    Vec3::Y,
                    MORTAR,
                    &lights,
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
                    quad(
                        &mut p,
                        [
                            o + vec3(-g, 0.0, -g),
                            o + vec3(g, 0.0, -g),
                            o + vec3(g, 0.0, g),
                            o + vec3(-g, 0.0, g),
                        ],
                        Vec3::Y,
                        col,
                        &lights,
                    );
                }
            }
            if !p.verts.is_empty() {
                pieces.push(p);
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
                let top = WALL_H * (0.8 + 0.35 * hash(x, z, 3));
                let min = vec3(c.x - h, 0.0, c.z - h);
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
        // 소품: 모델이 있으면 첫 자세를 굽고, 없으면 상자.
        let mut pose = Vec::new();
        for pr in &lv.props {
            let half = pr.kind.half();
            let hh = pr.kind.height();
            let aabb = (half != Vec2::ZERO).then(|| Aabb {
                min: vec3(pr.pos.x - half.x, 0.0, pr.pos.z - half.y),
                max: vec3(pr.pos.x + half.x, hh, pr.pos.z + half.y),
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
                    vec3(pr.pos.x - s.x, 0.0, pr.pos.z - s.y),
                    vec3(pr.pos.x + s.x, hh, pr.pos.z + s.y),
                    prop_color(pr.kind),
                    WALL_TOP,
                    &lights,
                );
            }
            pieces.push(p);
        }
        Scenery {
            pieces,
            lights,
            extent: lv.half_extent() * 3.0,
        }
    }
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
