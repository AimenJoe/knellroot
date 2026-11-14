//! 배치(P4·P5): 모델·굽힌 덩어리를 그리기 호출 한도 아래로 모아 보낸다 + 뒤집은 껍데기 외곽선.
use super::*;

/// 외곽선 색(0037: 검정 대신 짙은 자두빛 — 부드러운 외곽선).
const OUTLINE: Color = Color::new(0.13, 0.07, 0.16, 1.0);
/// 외곽선 껍데기를 카메라에서 미는 거리(m) — 이보다 가까이 겹친 부위 사이에는 선을 긋지 않는다.
const OUTLINE_PUSH: f32 = 0.18;

/// 모델·굽힌 덩어리를 한 배치로 모은다(P4·P5): 부위 행렬 × 정점을 CPU에서 곱하고 빛(`lit`)을 정점 색에 굽는다.
/// macroquad는 그리기 호출 하나에 정점 10,000·색인 5,000 **미만**만 받고 넘치면 잘라 버린다
/// (0.4.16 `quad_gl.rs` `geometry`, `Conf::draw_call_*_capacity` 기본값 — 처음엔 40마리를 한 덩어리로 보내 아무것도 안 그려졌다).
/// 그래서 그 아래에서 끊어 보낸다. 텍스처가 바뀌어도 끊는다.
pub const MAX_DRAW_VERTICES: usize = 10_000;
/// 창 설정의 그리기 인덱스 용량(`main.rs window_conf`, 0036 — 기본 5,000에서 올림).
pub const DRAW_CALL_INDICES: usize = 15_000;
pub const MAX_DRAW_INDICES: usize = DRAW_CALL_INDICES;

/// 지금 모은 것(`v`·`i`)에 `nv`·`ni`를 더해도 한 그리기 호출에 들어가나 — 정점과 **색인 둘 다**(0135).
/// Meshy 모델은 정점을 나눠 써서 정점 하나에 색인이 2.4개쯤이라, 정점만 세면 색인이 15,000을 넘어 macroquad가 잘랐다
/// (외곽선·그림자 배치가 매 프레임 1만 개 가까이 잘림 — PC 빌드의 `geometry() exceeded` 경고로 찾음).
pub(super) fn fits(v: usize, i: usize, nv: usize, ni: usize) -> bool {
    v + nv < MAX_DRAW_VERTICES && i + ni < MAX_DRAW_INDICES
}

#[derive(Default)]
pub struct Batch {
    tex: Option<Texture2D>,
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    /// 지난 프레임에 보낸 정점 수·보낸 횟수(개발 줄).
    pub last_vertices: usize,
    this_frame: usize,
    pub last_sends: usize,
    sends: usize,
    /// 외곽선(0033): 배우 부위 상자를 키운 뒷면 — 앞면을 버리는 재질로 따로 보낸다.
    outline: Vec<Vertex>,
    outline_idx: Vec<u16>,
    outline_mat: Option<Material>,
    /// 켜져 있으면 `model`이 그리는 배우의 실루엣(월드 자리, 흰색)을 모은다 — 그림자 가림판(0045).
    pub casting: bool,
    pub casters: Vec<Vertex>,
    pub caster_idx: Vec<u16>,
    outline_tried: bool,
    /// 법선 패스에 다시 그릴 것을 모으나(0061), 모은 덩어리.
    pub record: bool,
    pub recorded: Vec<(Vec<Vertex>, Vec<u16>)>,
}

/// `bake_glow`의 빛남 값: 이 값이면 스스로 빛나는 부위를 아예 굽지 않는다(꺼진 것).
const GLOW_OFF: f32 = -1.0;

fn to_u8(x: f32) -> u8 {
    (x.clamp(0.0, 1.0) * 255.0) as u8
}

impl Batch {
    /// 모델 한 자세를 정점으로(빛을 구워서). `glow_` 부위는 빛 대신 `tint`만(스스로 빛남). 굽기(`scenery`)와 매 프레임이 같이 쓴다.
    pub fn bake_model(
        verts: &mut Vec<Vertex>,
        idx: &mut Vec<u16>,
        e: &Entry,
        pose: &[Mat4],
        inst: Mat4,
        tint: Color,
        lights: &[Light],
    ) {
        Self::bake_glow(verts, idx, e, pose, inst, tint, lights, 1.0, false);
    }

    /// 스스로 빛나는 부위(`glow_`)를 빼고 굽는다 — 꺼진 화로(0171: 불꽃 없는 그릇).
    pub fn bake_model_dark(
        verts: &mut Vec<Vertex>,
        idx: &mut Vec<u16>,
        e: &Entry,
        pose: &[Mat4],
        inst: Mat4,
        tint: Color,
        lights: &[Light],
    ) {
        Self::bake_glow(verts, idx, e, pose, inst, tint, lights, GLOW_OFF, false);
    }

    #[allow(clippy::too_many_arguments)]
    fn bake_glow(
        verts: &mut Vec<Vertex>,
        idx: &mut Vec<u16>,
        e: &Entry,
        pose: &[Mat4],
        inst: Mat4,
        tint: Color,
        lights: &[Light],
        glow: f32,
        actor: bool,
    ) {
        let m = &e.model;
        let look = look();
        for (i, node) in m.nodes.iter().enumerate() {
            let Some(mi) = node.mesh else { continue };
            let mesh = &m.meshes[mi];
            let mat = inst * pose[i];
            let base = verts.len() as u16;
            let bright = e.glow.get(i).copied().unwrap_or(false);
            if bright && glow == GLOW_OFF {
                continue;
            }
            // 구운 불꽃(0142): 지형에 굽는 스스로 빛나는 부위(화로 불꽃)는 밑동은 그대로, 끝으로 갈수록 일렁인다(바람 셰이더).
            let (y0, y1) = mesh
                .pos
                .iter()
                .map(|q| mat.transform_point3(*q).y)
                .fold((f32::MAX, f32::MIN), |(a, b), y| (a.min(y), b.max(y)));
            for k in 0..mesh.pos.len() {
                let p = mat.transform_point3(mesh.pos[k]);
                let n = mat.transform_vector3(mesh.nrm[k]).normalize_or_zero();
                let f = if bright {
                    Vec3::splat(glow)
                } else {
                    let r = if actor { rim(p, n, look) } else { Vec3::ZERO };
                    lit(p, n, lights) + r
                };
                let c = Color::from_rgba(
                    to_u8(tint.r * f.x),
                    to_u8(tint.g * f.y),
                    to_u8(tint.b * f.z),
                    to_u8(tint.a),
                );
                let mut v = crate::scenery::with_normal(
                    Vertex::new(p.x, p.y, p.z, mesh.uv[k].x, mesh.uv[k].y, c),
                    n,
                );
                if bright && !actor {
                    crate::scenery::sway(&mut v, flame_sway(p.y, y0, y1));
                }
                verts.push(v);
            }
            idx.extend(mesh.idx.iter().map(|&j| base + j as u16));
        }
    }

    fn room(&mut self, tex: Option<&Texture2D>, nv: usize, ni: usize) {
        if self.tex.as_ref() != tex {
            self.send();
            self.tex = tex.cloned();
        }
        if !fits(self.vertices.len(), self.indices.len(), nv, ni) {
            self.send();
        }
    }

    /// 움직이는 모델 하나(빛은 가까운 화로만 센다).
    pub fn model(
        &mut self,
        e: &Entry,
        pose: &[Mat4],
        inst: Mat4,
        tint: Color,
        lights: &[Light],
        glow: f32,
    ) {
        let at = inst.transform_point3(Vec3::ZERO);
        let near: Vec<Light> = lights
            .iter()
            .filter(|l| l.pos.distance(at) < l.radius + 3.0)
            .copied()
            .collect();
        let (nv, ni) = e
            .model
            .meshes
            .iter()
            .fold((0, 0), |(a, b), m| (a + m.pos.len(), b + m.idx.len()));
        self.room(Some(&e.tex), nv, ni);
        // 배우는 바닥보다 조금 밝게(어두운 성소에서 실루엣이 묻히지 않게 — 무대 조명처럼).
        let lift = Color::new(
            tint.r * ACTOR_LIFT,
            tint.g * ACTOR_LIFT,
            tint.b * ACTOR_LIFT,
            tint.a,
        );
        MOON_GATE.with(|g| g.set(actor_moon(at)));
        Self::bake_glow(
            &mut self.vertices,
            &mut self.indices,
            e,
            pose,
            inst,
            lift,
            &near,
            glow,
            true,
        );
        MOON_GATE.with(|g| g.set(1.0));
        // 그림자 지도는 한 번에 그린다 — 다 차면 그 뒤 배우는 그림자 없이(반쯤 잘린 그림자보다 낫다).
        if self.casting && tint.a > 0.5 && fits(self.casters.len(), self.caster_idx.len(), nv, ni) {
            let m = &e.model;
            for (i, node) in m.nodes.iter().enumerate() {
                let Some(mi) = node.mesh else { continue };
                let mesh = &m.meshes[mi];
                let mat = inst * pose[i];
                let base = self.casters.len() as u16;
                self.casters.extend(mesh.pos.iter().map(|q| {
                    let p = mat.transform_point3(*q);
                    Vertex::new(p.x, p.y, p.z, 0.0, 0.0, WHITE)
                }));
                self.caster_idx
                    .extend(mesh.idx.iter().map(|&j| base + j as u16));
            }
        }
        let k = look();
        if k.outline_px > 0.0 && tint.a > 0.99 {
            // 화면 1px = 그 거리에서 이만큼(m): 2·거리·tan(시야/2) / 내부 세로.
            let dist = k.eye.distance(at).max(0.5);
            let w = k.outline_px * 2.0 * dist * (k.fovy * 0.5).tan() / k.rows;
            // 껍데기는 모델과 정점·색인 수가 같다 — 넣기 전에 넘칠지 보고 먼저 보낸다.
            if !fits(self.outline.len(), self.outline_idx.len(), nv, ni) {
                self.send_outline();
            }
            Self::hull(
                &mut self.outline,
                &mut self.outline_idx,
                e,
                pose,
                inst,
                w,
                k.eye,
            );
        }
    }

    /// 외곽선 껍데기(0033): 부위마다 상자를 가운데에서 `w`(m)만큼 바깥으로 키운 것. 부위가 상자라 축마다 부호로 민다(면마다 정점이 따로라
    /// 법선으로 밀면 모서리가 벌어진다).
    /// 껍데기는 카메라에서 `OUTLINE_PUSH`만큼 뒤로 민다 — 부위 사이 선은 뒤 부위가 그만큼 떨어져 있을 때만 남고(작은 배우가
    /// 안쪽 선으로 까매지지 않게), 바닥·배경 앞의 바깥 테는 그대로.
    #[allow(clippy::too_many_arguments)]
    fn hull(
        verts: &mut Vec<Vertex>,
        idx: &mut Vec<u16>,
        e: &Entry,
        pose: &[Mat4],
        inst: Mat4,
        w: f32,
        eye: Vec3,
    ) {
        let m = &e.model;
        for (i, node) in m.nodes.iter().enumerate() {
            let Some(mi) = node.mesh else { continue };
            if e.glow.get(i).copied().unwrap_or(false) {
                continue;
            }
            let mesh = &m.meshes[mi];
            if mesh.pos.is_empty() {
                continue;
            }
            let mat = inst * pose[i];
            let c = mesh.pos.iter().copied().sum::<Vec3>() / mesh.pos.len() as f32;
            let base = verts.len() as u16;
            let sign = |x: f32| {
                if x > 1e-5 {
                    1.0
                } else if x < -1e-5 {
                    -1.0
                } else {
                    0.0
                }
            };
            for p in &mesh.pos {
                let d = *p - c;
                let q = *p + vec3(sign(d.x), sign(d.y), sign(d.z)) * w;
                let q = mat.transform_point3(q);
                let q = q + (q - eye).normalize_or_zero() * OUTLINE_PUSH;
                verts.push(Vertex::new(q.x, q.y, q.z, 0.0, 0.0, OUTLINE));
            }
            idx.extend(mesh.idx.iter().map(|&j| base + j as u16));
        }
    }

    /// 외곽선을 앞면 버리기 재질로 보낸다(GL에서만 — Metal이면 만들지 않고 끈다).
    fn send_outline(&mut self) {
        if self.outline.is_empty() {
            return;
        }
        if !self.outline_tried {
            self.outline_tried = true;
            self.outline_mat = outline_material();
        }
        let verts = std::mem::take(&mut self.outline);
        let idx = std::mem::take(&mut self.outline_idx);
        if let Some(mat) = &self.outline_mat {
            gl_use_material(mat);
            self.this_frame += verts.len();
            self.sends += 1;
            draw_mesh(&Mesh {
                vertices: verts,
                indices: idx,
                texture: None,
            });
            gl_use_default_material();
        }
    }

    /// 이미 구운 덩어리(`alpha`를 곱해서 — 가린 벽을 옅게).
    pub fn raw(&mut self, verts: &[Vertex], idx: &[u16], tex: Option<&Texture2D>, alpha: f32) {
        if verts.len() >= MAX_DRAW_VERTICES || idx.len() >= MAX_DRAW_INDICES {
            // 한 덩어리가 한도를 넘으면 굽는 쪽이 나눠야 한다(바닥은 줄마다).
            warn!("piece too big: {} verts", verts.len());
            return;
        }
        self.room(tex, verts.len(), idx.len());
        let base = self.vertices.len() as u16;
        if alpha >= 0.999 {
            self.vertices.extend_from_slice(verts);
        } else {
            let a = to_u8(alpha);
            self.vertices.extend(verts.iter().map(|v| {
                let mut v = *v;
                v.color[3] = ((v.color[3] as u16 * a as u16) / 255) as u8;
                v
            }));
        }
        self.indices.extend(idx.iter().map(|&j| base + j));
    }

    pub fn send(&mut self) {
        if self.vertices.is_empty() {
            return;
        }
        self.this_frame += self.vertices.len();
        self.sends += 1;
        if self.record {
            self.recorded
                .push((self.vertices.clone(), self.indices.clone()));
        }
        draw_mesh(&Mesh {
            vertices: std::mem::take(&mut self.vertices),
            indices: std::mem::take(&mut self.indices),
            texture: self.tex.clone(),
        });
    }

    /// 프레임마다 한 번(모은 것을 보내고 수를 개발 줄로).
    pub fn flush(&mut self) {
        self.send();
        self.send_outline();
        self.last_vertices = self.this_frame;
        self.this_frame = 0;
        self.last_sends = self.sends;
        self.sends = 0;
    }
}

/// 외곽선 재질(0033): macroquad 기본 3D 셰이더 그대로, 앞면을 버리고(키운 상자의 뒷면만 남아 몸 둘레에 테가 된다) 깊이는 쓴다.
fn outline_material() -> Option<Material> {
    let ctx = unsafe { get_internal_gl().quad_context };
    if ctx.info().backend != miniquad::Backend::OpenGl {
        return None;
    }
    const VERTEX: &str = r#"#version 100
    attribute vec3 position;
    attribute vec2 texcoord;
    attribute vec4 color0;
    varying lowp vec4 color;
    uniform mat4 Model;
    uniform mat4 Projection;
    void main() {
        gl_Position = Projection * Model * vec4(position, 1);
        color = color0 / 255.0;
    }"#;
    const FRAGMENT: &str = r#"#version 100
    varying lowp vec4 color;
    void main() {
        gl_FragColor = color;
    }"#;
    load_material(
        ShaderSource::Glsl {
            vertex: VERTEX,
            fragment: FRAGMENT,
        },
        MaterialParams {
            pipeline_params: PipelineParams {
                depth_write: true,
                depth_test: Comparison::LessOrEqual,
                cull_face: OUTLINE_CULL,
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .map_err(|e| {
        warn!("outline material: {e:?}");
    })
    .ok()
}

/// 키운 상자에서 버릴 면. 렌더 타깃이 위아래를 뒤집으면 감김이 바뀌어 반대가 된다 — 스크린숏으로 정했다(0033).
const OUTLINE_CULL: miniquad::CullFace = miniquad::CullFace::Front;
