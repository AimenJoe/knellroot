//! 캐릭터 그림자 가림판(0045): 지난 프레임의 배우 실루엣을 달 쪽에서 찍고, 빛 지도 바닥이 그것을 읽는다.
use super::*;

/// 그림자 가림판(0045): 한 변 칸 수와 덮는 넓이(m). 0.125m 칸 — 바닥 빛 지도(0.25m)보다 촘촘해 캐릭터 실루엣이 읽힌다.
const SHADOW_RES: u32 = 256;
const SHADOW_SPAN: f32 = 32.0;

impl Renderer {
    /// 지난 프레임의 배우 실루엣을 달 쪽 정사영으로 가림판에 찍는다. 재질을 못 만들면(Metal 등) 끈다 — 바닥은 그림자 없이.
    pub(super) fn shadow_pass(&mut self) {
        if !self.shadow_tried {
            self.shadow_tried = true;
            self.mask_mat = flat_material(MASK_FRAGMENT, &[], &[]);
            self.floor_mat = flat_material(
                FLOOR_FRAGMENT,
                &[
                    UniformDesc::new("LightVP", UniformType::Mat4),
                    UniformDesc::new("Shade", UniformType::Float1),
                ],
                &["ShadowMask"],
            );
            if self.mask_mat.is_some() && self.floor_mat.is_some() {
                let rt = render_target(SHADOW_RES, SHADOW_RES);
                rt.texture.set_filter(FilterMode::Nearest);
                self.shadow_rt = Some(rt);
            }
        }
        let (Some(rt), Some(mask)) = (&self.shadow_rt, &self.mask_mat) else {
            return;
        };
        let c = self.shadow_center;
        let lc = Camera3D {
            position: c + LIGHT_DIR.normalize() * 40.0,
            target: c,
            up: Vec3::Y,
            fovy: SHADOW_SPAN,
            aspect: Some(1.0),
            projection: Projection::Orthographics,
            render_target: Some(rt.clone()),
            ..Default::default()
        };
        set_camera(&lc);
        clear_background(BLACK);
        if !self.batch.casters.is_empty() {
            gl_use_material(mask);
            draw_mesh(&Mesh {
                vertices: self.batch.casters.clone(),
                indices: self.batch.caster_idx.clone(),
                texture: None,
            });
            gl_use_default_material();
        }
        self.light_vp = Some(lc.matrix());
    }
}

/// 가림판: 무엇이든 흰색.
const MASK_FRAGMENT: &str = r#"#version 100
precision mediump float;
varying lowp vec4 color;
varying lowp vec2 uv;
varying mediump vec3 sp;
void main() {
    gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
}"#;

/// 바닥: 색 × 빛 지도(0044) × 캐릭터 그림자(가림판, 빛 공간 자리로 읽는다).
const FLOOR_FRAGMENT: &str = r#"#version 100
precision mediump float;
varying lowp vec4 color;
varying lowp vec2 uv;
varying mediump vec3 sp;
uniform sampler2D Texture;
uniform sampler2D ShadowMask;
uniform float Shade;
void main() {
    vec4 c = color * texture2D(Texture, uv);
    float s = 0.0;
    if (sp.x > 0.0 && sp.x < 1.0 && sp.y > 0.0 && sp.y < 1.0) {
        s = texture2D(ShadowMask, sp.xy).r;
    }
    gl_FragColor = vec4(c.rgb * (1.0 - Shade * s), c.a);
}"#;

/// 두 재질이 같이 쓰는 꼭짓점 셰이더: 월드 자리를 빛 공간(`LightVP`, 없으면 단위)으로도 넘긴다.
const SHADOW_VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec4 color;
varying lowp vec2 uv;
varying mediump vec3 sp;
uniform mat4 Model;
uniform mat4 Projection;
uniform mat4 LightVP;
void main() {
    vec4 w = Model * vec4(position, 1.0);
    gl_Position = Projection * w;
    color = color0 / 255.0;
    uv = texcoord;
    vec4 l = LightVP * w;
    sp = vec3(l.xy * 0.5 + 0.5, l.z);
}"#;

fn flat_material(fragment: &str, uniforms: &[UniformDesc], textures: &[&str]) -> Option<Material> {
    let mut u = uniforms.to_vec();
    if !u.iter().any(|d| d.name == "LightVP") {
        u.push(UniformDesc::new("LightVP", UniformType::Mat4));
    }
    load_material(
        ShaderSource::Glsl {
            vertex: SHADOW_VERTEX,
            fragment,
        },
        MaterialParams {
            pipeline_params: PipelineParams {
                depth_write: true,
                depth_test: Comparison::LessOrEqual,
                ..Default::default()
            },
            uniforms: u,
            textures: textures.iter().map(|t| t.to_string()).collect(),
        },
    )
    .map_err(|e| {
        warn!("shadow material: {e:?}");
    })
    .ok()
}
