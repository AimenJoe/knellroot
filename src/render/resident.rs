//! 상주 지형(0186): 구운 지형 가운데 움직이지 않는 덩어리(빛 지도 바닥 + 가리지 않는 덩어리)를 GPU 버퍼에 **한 번** 올려 두고
//! 프레임마다 "그려라"만 보낸다. macroquad의 `draw_mesh`는 부를 때마다 정점을 다시 올린다(GPU에 머무는 메시가 없다, KB
//! `reference/macroquad/3d-rendering.md`) — 정원에서 프레임마다 약 30만 정점(≈ 12MB)을 색 패스·법선 패스에 두 번 복사·업로드했다.
//! macroquad는 그대로 두고 그 아래의 miniquad를 직접 부른다(`get_internal_gl`): 쌓인 macroquad 그리기를 먼저 보내고(`flush`),
//! 같은 렌더 패스·같은 투영으로 우리 파이프라인을 그린다. 셰이더는 원래 재질과 같은 원본(`WIND_VS`·`SHADOW_VERTEX`·`NORMAL_VS` …).
//! 가려지면 옅어지는 덩어리(벽·기둥 — 덩어리마다 투명도)는 지금처럼 배치로 간다. GL이 아닌 백엔드(Metal)는 만들지 않는다(옛 길).
use super::*;
use miniquad::{
    BlendFactor, BlendState, BlendValue, BufferId, BufferLayout, BufferSource, BufferType,
    BufferUsage, Equation, PassAction, Pipeline, ShaderMeta, ShaderSource as MqShader, TextureId,
    UniformBlockLayout, VertexAttribute, VertexFormat,
};

/// 한 번에 그리는 묶음: 같은 텍스처의 덩어리들을 u16 색인 한도 안에서 이어 붙인 것.
struct Chunk {
    vb: BufferId,
    ib: BufferId,
    n: i32,
    tex: Option<TextureId>,
}

struct Pipes {
    floor: Pipeline,
    wind: Pipeline,
    normal: Pipeline,
}

pub(super) struct Resident {
    /// 이 버퍼를 만든 지형의 열쇠(`Renderer::scenery`의 것과 같으면 그대로 쓴다).
    key: Option<usize>,
    floor: Vec<Chunk>,
    still: Vec<Chunk>,
    pipes: Option<Pipes>,
    tried: bool,
    white: Option<Texture2D>,
    /// 올려 둔 정점 수(개발 줄).
    pub verts: usize,
    /// 이 프레임에 상주 지형으로 그렸나(법선 패스가 같이 그린다) — 프레임 들머리에 끈다.
    pub active: bool,
}

/// 덩어리를 상주로 둘까: 빛 지도 바닥이거나, 가림 상자가 없는(옅어지지 않는) 덩어리.
pub(super) fn is_resident(p: &crate::scenery::Piece) -> bool {
    p.lightmap || p.aabb.is_none()
}

fn attrs() -> [VertexAttribute; 4] {
    [
        VertexAttribute::new("position", VertexFormat::Float3),
        VertexAttribute::new("texcoord", VertexFormat::Float2),
        VertexAttribute::new("color0", VertexFormat::Byte4),
        VertexAttribute::new("normal", VertexFormat::Float4),
    ]
}

fn pipeline(
    ctx: &mut dyn miniquad::RenderingBackend,
    vs: &str,
    fs: &str,
    uniforms: &[(&str, UniformType)],
    images: &[&str],
    blend: bool,
) -> Option<Pipeline> {
    let meta = ShaderMeta {
        uniforms: UniformBlockLayout {
            uniforms: uniforms
                .iter()
                .map(|(n, t)| UniformDesc::new(n, *t))
                .collect(),
        },
        images: images.iter().map(|s| s.to_string()).collect(),
    };
    let shader = ctx
        .new_shader(
            MqShader::Glsl {
                vertex: vs,
                fragment: fs,
            },
            meta,
        )
        .map_err(|e| {
            warn!("resident shader: {:?}", e);
        })
        .ok()?;
    Some(ctx.new_pipeline(
        &[BufferLayout::default()],
        &attrs(),
        shader,
        miniquad::PipelineParams {
            depth_write: true,
            depth_test: miniquad::Comparison::LessOrEqual,
            color_blend: blend.then(|| {
                BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )
            }),
            ..Default::default()
        },
    ))
}

fn mat(m: Mat4) -> [f32; 16] {
    m.to_cols_array()
}

impl Resident {
    pub(super) fn new() -> Resident {
        Resident {
            key: None,
            floor: Vec::new(),
            still: Vec::new(),
            pipes: None,
            tried: false,
            white: None,
            verts: 0,
            active: false,
        }
    }

    /// 쓸 수 있나(GL에서 파이프라인 셋을 만들었나) — 처음 부를 때 한 번 만들어 본다.
    pub(super) fn ready(&mut self) -> bool {
        if !self.tried {
            self.tried = true;
            let ctx = unsafe { get_internal_gl().quad_context };
            if ctx.info().backend == miniquad::Backend::OpenGl {
                use UniformType::{Float1, Float4, Mat4 as M4};
                let floor = pipeline(
                    ctx,
                    super::shadow::SHADOW_VERTEX,
                    super::shadow::FLOOR_FRAGMENT,
                    &[
                        ("Projection", M4),
                        ("Model", M4),
                        ("LightVP", M4),
                        ("Shade", Float1),
                    ],
                    &["Texture", "ShadowMask"],
                    false,
                );
                let wind = pipeline(
                    ctx,
                    WIND_VS,
                    WIND_FS,
                    &[("Projection", M4), ("Model", M4), ("Wind", Float4)],
                    &["Texture"],
                    true,
                );
                let normal = pipeline(
                    ctx,
                    super::post::NORMAL_VS,
                    super::post::NORMAL_FS,
                    &[
                        ("Projection", M4),
                        ("Model", M4),
                        ("View", M4),
                        ("Wind", Float4),
                    ],
                    &[],
                    false,
                );
                if let (Some(floor), Some(wind), Some(normal)) = (floor, wind, normal) {
                    self.pipes = Some(Pipes {
                        floor,
                        wind,
                        normal,
                    });
                    self.white = Some(Texture2D::from_rgba8(1, 1, &[255, 255, 255, 255]));
                }
            }
        }
        self.pipes.is_some()
    }

    /// 지형이 바뀌었으면(열쇠) 버퍼를 지우고 새로 올린다.
    pub(super) fn sync(&mut self, key: usize, sc: &Scenery) {
        if self.key == Some(key) {
            return;
        }
        let ctx = unsafe { get_internal_gl().quad_context };
        for c in self.floor.drain(..).chain(self.still.drain(..)) {
            ctx.delete_buffer(c.vb);
            ctx.delete_buffer(c.ib);
        }
        self.key = Some(key);
        self.verts = 0;
        let floor: Vec<&crate::scenery::Piece> = sc.pieces.iter().filter(|p| p.lightmap).collect();
        let still: Vec<&crate::scenery::Piece> = sc
            .pieces
            .iter()
            .filter(|p| !p.lightmap && is_resident(p))
            .collect();
        self.floor = chunks(ctx, &floor, &mut self.verts);
        self.still = chunks(ctx, &still, &mut self.verts);
    }

    /// 빛 지도 바닥: 캐릭터 그림자 가림판을 읽는다(없으면 그늘 0 — 원래의 기본 재질과 같은 색).
    pub(super) fn draw_floor(&self, light_vp: Option<Mat4>, mask: Option<&Texture2D>, shade: f32) {
        let (Some(p), Some(white)) = (&self.pipes, &self.white) else {
            return;
        };
        let lv = light_vp.unwrap_or(Mat4::IDENTITY);
        let shade = if light_vp.is_some() && mask.is_some() {
            shade
        } else {
            0.0
        };
        let mask = mask.unwrap_or(white).raw_miniquad_id();
        self.draw(&p.floor, &self.floor, &[mask], |proj| {
            let mut u = Vec::with_capacity(49);
            u.extend(mat(proj));
            u.extend(mat(Mat4::IDENTITY));
            u.extend(mat(lv));
            u.push(shade);
            u
        });
    }

    /// 가리지 않는 덩어리: 바람 재질(풀 끝·잎·불꽃이 흔들린다).
    pub(super) fn draw_still(&self, wind: Vec4) {
        let Some(p) = &self.pipes else { return };
        self.draw(&p.wind, &self.still, &[], |proj| {
            let mut u = Vec::with_capacity(36);
            u.extend(mat(proj));
            u.extend(mat(Mat4::IDENTITY));
            u.extend(wind.to_array());
            u
        });
    }

    /// 법선 패스(0061): 상주한 것 모두를 법선·깊이 타깃에.
    pub(super) fn draw_normals(&self, view: Mat4, wind: Vec4) {
        let Some(p) = &self.pipes else { return };
        let u = |proj: Mat4| {
            let mut u = Vec::with_capacity(52);
            u.extend(mat(proj));
            u.extend(mat(Mat4::IDENTITY));
            u.extend(mat(view));
            u.extend(wind.to_array());
            u
        };
        self.draw(&p.normal, &self.floor, &[], u);
        self.draw(&p.normal, &self.still, &[], u);
    }

    /// 쌓인 macroquad 그리기를 먼저 보내고, 지금 카메라의 렌더 패스·투영으로 묶음들을 그린다.
    fn draw(
        &self,
        pipe: &Pipeline,
        list: &[Chunk],
        extra: &[TextureId],
        uniforms: impl Fn(Mat4) -> Vec<f32>,
    ) {
        if list.is_empty() {
            return;
        }
        let Some(white) = &self.white else { return };
        let mut gl = unsafe { get_internal_gl() };
        gl.flush();
        let proj = gl.quad_gl.get_projection_matrix();
        let pass = gl.quad_gl.get_active_render_pass();
        let ctx = gl.quad_context;
        let (w, h) = match pass {
            Some(rp) => {
                let t = ctx.render_pass_texture(rp);
                ctx.texture_size(t)
            }
            None => {
                let (w, h) = miniquad::window::screen_size();
                (w as u32, h as u32)
            }
        };
        ctx.begin_pass(pass, PassAction::Nothing);
        ctx.apply_pipeline(pipe);
        ctx.apply_viewport(0, 0, w as i32, h as i32);
        ctx.apply_scissor_rect(0, 0, w as i32, h as i32);
        let data = uniforms(proj);
        let white = white.raw_miniquad_id();
        for c in list {
            let mut images = vec![c.tex.unwrap_or(white)];
            images.extend_from_slice(extra);
            ctx.apply_bindings(&miniquad::Bindings {
                vertex_buffers: vec![c.vb],
                index_buffer: c.ib,
                images,
            });
            ctx.apply_uniforms_from_bytes(data.as_ptr() as *const u8, data.len() * 4);
            ctx.draw(0, c.n, 1);
        }
        ctx.end_render_pass();
    }
}

/// 묶음 나누기(순수): (텍스처, 정점 수) 목록 → 묶음마다 (텍스처, 덩어리 번호들). 같은 텍스처끼리만, 묶음의 정점 합은 u16 색인 한도
/// (65,535) 안 — 덩어리를 쪼개지 않는다(한 덩어리는 굽는 쪽이 이미 한도 아래로 나눴다, `Batch::raw`).
pub(super) fn plan<T: PartialEq + Copy>(pieces: &[(T, usize)]) -> Vec<(T, Vec<usize>)> {
    let mut groups: Vec<(T, Vec<usize>)> = Vec::new();
    for (i, (t, _)) in pieces.iter().enumerate() {
        match groups.iter_mut().find(|(g, _)| g == t) {
            Some((_, v)) => v.push(i),
            None => groups.push((*t, vec![i])),
        }
    }
    let mut out = Vec::new();
    for (t, list) in groups {
        let (mut cur, mut n) = (Vec::new(), 0usize);
        for i in list {
            let v = pieces[i].1;
            if n + v > u16::MAX as usize && !cur.is_empty() {
                out.push((t, std::mem::take(&mut cur)));
                n = 0;
            }
            cur.push(i);
            n += v;
        }
        if !cur.is_empty() {
            out.push((t, cur));
        }
    }
    out
}

/// 계획대로 이어 붙여 올린다.
fn chunks(
    ctx: &mut dyn miniquad::RenderingBackend,
    pieces: &[&crate::scenery::Piece],
    count: &mut usize,
) -> Vec<Chunk> {
    let keys: Vec<(Option<TextureId>, usize)> = pieces
        .iter()
        .map(|p| (p.tex.as_ref().map(|t| t.raw_miniquad_id()), p.verts.len()))
        .collect();
    plan(&keys)
        .into_iter()
        .filter_map(|(tex, list)| {
            let (mut vs, mut is): (Vec<Vertex>, Vec<u16>) = (Vec::new(), Vec::new());
            for i in list {
                let p = pieces[i];
                let base = vs.len() as u16;
                vs.extend_from_slice(&p.verts);
                is.extend(p.idx.iter().map(|&j| base + j));
            }
            if is.is_empty() {
                return None;
            }
            *count += vs.len();
            Some(Chunk {
                vb: ctx.new_buffer(
                    BufferType::VertexBuffer,
                    BufferUsage::Immutable,
                    BufferSource::slice(&vs[..]),
                ),
                ib: ctx.new_buffer(
                    BufferType::IndexBuffer,
                    BufferUsage::Immutable,
                    BufferSource::slice(&is[..]),
                ),
                n: is.len() as i32,
                tex,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::plan;

    /// 0186: 묶음은 같은 텍스처끼리, 정점 합이 u16 한도 안, 덩어리는 모두 정확히 한 번 — 순서는 텍스처 안에서 그대로.
    #[test]
    fn chunks_group_by_texture_stay_under_the_u16_limit_and_keep_every_piece_once() {
        let pieces: Vec<(Option<u8>, usize)> = (0..300)
            .map(|i| {
                (
                    if i % 3 == 0 {
                        None
                    } else {
                        Some((i % 2) as u8)
                    },
                    900 + (i * 37) % 6000,
                )
            })
            .collect();
        let out = plan(&pieces);
        let mut seen = vec![0; pieces.len()];
        for (t, list) in &out {
            let sum: usize = list.iter().map(|&i| pieces[i].1).sum();
            assert!(sum <= u16::MAX as usize, "{sum}");
            assert!(list.iter().all(|&i| pieces[i].0 == *t));
            assert!(list.windows(2).all(|w| w[0] < w[1]));
            for &i in list {
                seen[i] += 1;
            }
        }
        assert!(seen.iter().all(|&n| n == 1));
        // 묶음 수는 정점 합을 한도로 나눈 것에 가깝다(조각이 흩어지지 않는다).
        let total: usize = pieces.iter().map(|p| p.1).sum();
        assert!(out.len() <= total / u16::MAX as usize + 6, "{}", out.len());
        // 한 덩어리가 한도와 같아도 혼자 한 묶음.
        assert_eq!(plan(&[(1u8, 65_535), (1u8, 1)]).len(), 2);
    }
}
