//! 화면 셈: 렌더 타깃·내부 해상도 맞추기, 법선 패스와 화면 공간 선 둘(0061), 마무리 패스(0140), 정수 배율로 올리기, 지평선 아지랑이.
use super::*;

/// 화면 공간 선 둘(0061 t3ssel8r 풍): 불투명 기하를 한 번 더 그린 법선 타깃(rg = 화면 법선 xy, ba = 선형 깊이 16비트)과 재질 둘.
pub(super) struct EdgeFx {
    rt: RenderTarget,
    size: (u32, u32),
    normal_mat: Material,
    edge_mat: Material,
}

/// 작은 렌더 타깃을 화면 `dest`에(뒤집어서 — 렌더 타깃은 위아래가 바뀌어 나온다).
pub(super) fn blit(rt: &RenderTarget, dest: Rect) {
    draw_texture_ex(
        &rt.texture,
        dest.x,
        dest.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(dest.w, dest.h)),
            flip_y: true,
            ..Default::default()
        },
    );
}

/// 깊이 버퍼가 붙은 렌더 타깃(0033).
pub(super) fn make_rt(w: u32, h: u32) -> RenderTarget {
    // 깊이 버퍼를 붙인다(0033): `render_target`의 기본값은 깊이 없음이라, `Camera3D`가 깊이 시험을 켜도 붙은 버퍼가 없어
    // 그린 순서로만 덮였다 — 모델 부위 상자끼리도. 외곽선(키운 뒷면)도 깊이가 있어야 몸 뒤로 숨는다.
    let rt = render_target_ex(
        w,
        h,
        RenderTargetParams {
            depth: true,
            ..Default::default()
        },
    );
    rt.texture.set_filter(FilterMode::Nearest);
    rt
}

pub(super) fn make_post(w: u32, h: u32) -> RenderTarget {
    let rt = render_target(w, h);
    rt.texture.set_filter(FilterMode::Nearest);
    rt
}

/// 화면(물리 px)에 맞춘 내부 해상도와 정수 배율(0046): 배율은 높이가 `INTERNAL_H`에 가장 가깝게(반올림) 고르고,
/// 내부 해상도는 그 배율로 화면을 채우는 크기 — 픽셀은 늘 정수배로 고르고 검은 테두리는 배율보다 작다.
/// 폭은 높이의 4:3~21:9 사이로 묶는다(세로 폰·아주 넓은 창은 나머지가 테두리). 세로 시야각은 그대로라 넓은 화면은 옆을 더 본다.
#[cfg(test)]
pub fn fit_internal(sw: f32, sh: f32) -> (u32, u32, u32) {
    fit_internal_rows(sw, sh, INTERNAL_H as f32)
}

/// `fit_internal`을 줄 수 `rows`에 맞춰(0190 — 연구소 `render.rows`).
pub fn fit_internal_rows(sw: f32, sh: f32, rows: f32) -> (u32, u32, u32) {
    let rows = rows.max(90.0);
    let by_h = (sh / rows).round();
    let by_w = (sw / (rows * 4.0 / 3.0)).floor();
    let s = by_h.min(by_w).max(1.0) as u32;
    let mut w = ((sw / s as f32).floor() as u32).max(1);
    let mut h = ((sh / s as f32).floor() as u32).max(1);
    w = w.min(h * 21 / 9).max(1);
    h = h.min(w * 3 / 4).max(1);
    (w, h, s)
}

impl Renderer {
    /// 법선 패스(0061): 모은 불투명 기하를 같은 카메라로 법선 타깃에 다시 그린다. 재질을 못 만들면 거짓(선 없이).
    pub(super) fn normal_pass(&mut self, c3: &Camera3D, size: (u32, u32)) -> bool {
        if !self.edge_tried {
            self.edge_tried = true;
            self.edge = edge_fx(size);
        }
        let Some(fx) = &mut self.edge else {
            self.batch.recorded.clear();
            return false;
        };
        if fx.size != size {
            fx.rt = edge_rt(size);
            fx.size = size;
        }
        let cam = Camera3D {
            position: c3.position,
            target: c3.target,
            up: c3.up,
            fovy: c3.fovy,
            aspect: c3.aspect,
            projection: c3.projection,
            render_target: Some(fx.rt.clone()),
            viewport: None,
            z_near: c3.z_near,
            z_far: c3.z_far,
        };
        set_camera(&cam);
        // 빈 곳 = 아주 먼 깊이(ba = 1)·화면을 보는 법선.
        clear_background(Color::new(0.5, 0.5, 1.0, 1.0));
        fx.normal_mat
            .set_uniform("View", Mat4::look_at_rh(c3.position, c3.target, c3.up));
        fx.normal_mat.set_uniform("Wind", self.wind_uniform);
        gl_use_material(&fx.normal_mat);
        for (vertices, indices) in self.batch.recorded.drain(..) {
            draw_mesh(&Mesh {
                vertices,
                indices,
                texture: None,
            });
        }
        gl_use_default_material();
        // 상주 지형(0186)은 배치에 없으니 여기서 같은 카메라로.
        if self.resident.active {
            self.resident.draw_normals(
                Mat4::look_at_rh(c3.position, c3.target, c3.up),
                self.wind_uniform,
            );
        }
        true
    }

    /// 렌더 타깃을 지금 화면에 맞춘다(0046) — 크기가 바뀐 프레임에만 새로 만든다. 돌려주는 것은 내부 해상도.
    pub(super) fn fit_screen(&mut self) -> (u32, u32) {
        // 정수 배율은 물리 픽셀로 센다: 폰(DPR 3)의 논리 844×390에서는 논리로 세면 1배라 화면 반이 검정이었다(0006).
        let dpi = screen_dpi_scale().max(0.01);
        let (w, h, s) = fit_internal_rows(
            screen_width() * dpi,
            screen_height() * dpi,
            self.rows_target,
        );
        // 2배로 그려 줄이기(0190): 그리는 타깃은 두 배, 화면에 올리기 전에 한 번 반으로 줄인다(2×2 평균) — 도트 크기는 그대로, 경계만 부드럽게.
        let k = if self.ssaa { 2 } else { 1 };
        let fit = (w * k, h * k, s);
        if fit != self.rt_size || self.ssaa != self.half.is_some() {
            if (fit.0, fit.1) != (self.rt_size.0, self.rt_size.1) {
                self.rt = make_rt(fit.0, fit.1);
                self.post = make_post(fit.0, fit.1);
            }
            self.half = self.ssaa.then(|| make_post(w, h));
            self.rt_size = fit;
        }
        (fit.0, fit.1)
    }

    /// 렌더 타깃을 화면 가운데에 정수 배율로.
    pub(super) fn present(&self) -> Frame {
        set_default_camera();
        clear_background(BLACK);
        let (sw, sh) = (screen_width(), screen_height());
        let dpi = screen_dpi_scale().max(0.01);
        let (iw, ih, s) = self.rt_size;
        let scale = s as f32 / dpi;
        // 2배로 그렸으면 화면의 크기는 반(줄인 타깃)으로 센다.
        let k = if self.half.is_some() { 2 } else { 1 };
        let (dw, dh) = ((iw / k) as f32 * scale, (ih / k) as f32 * scale);
        let (ox, oy) = (((sw - dw) * 0.5).floor(), ((sh - dh) * 0.5).floor());
        // 화면 공간 선 둘(0061): 법선 패스를 그렸으면 선 재질로 올린다.
        // 화면 공간 선 둘(0061) + 마무리 패스(0140): 내부 해상도에서 한 번만 셈해 `post`에 담고, 화면에는 그것을 가장 가까운 픽셀로 키운다
        // (화면 해상도로 셈하면 배율² 배 — 2배 화면에서 화소마다 텍스처를 서른 번 가까이 읽었다). 선을 긋지 않아도 마무리 패스가
        // 켜져 있으면 같은 재질로(선은 `Grade.w`로 끈다).
        let g = self.grade_uniform;
        let edge = self
            .edge
            .as_ref()
            .filter(|_| self.edge_ready || g.x > 0.0 || g.y > 0.0 || g.z > 0.0);
        let src = match edge {
            Some(fx) => {
                set_camera(&Camera2D {
                    zoom: vec2(2.0 / iw as f32, 2.0 / ih as f32),
                    target: vec2(iw as f32 * 0.5, ih as f32 * 0.5),
                    render_target: Some(self.post.clone()),
                    ..Default::default()
                });
                fx.edge_mat.set_texture("Normals", fx.rt.texture.clone());
                // 2배로 그렸으면 선의 이웃을 두 칸 건너 본다 — 줄인 뒤에 지금과 같은 1px 두께(부드러운 가장자리로).
                fx.edge_mat
                    .set_uniform("Texel", vec2(k as f32 / iw as f32, k as f32 / ih as f32));
                fx.edge_mat.set_uniform("Edge", self.edge_uniform);
                fx.edge_mat.set_uniform("Grade", g);
                gl_use_material(&fx.edge_mat);
                draw_texture_ex(
                    &self.rt.texture,
                    0.0,
                    0.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(iw as f32, ih as f32)),
                        ..Default::default()
                    },
                );
                gl_use_default_material();
                set_default_camera();
                &self.post.texture
            }
            None => &self.rt.texture,
        };
        // 2배로 그렸으면 반 크기 타깃에 한 번 줄여 옮긴다: 정확히 2:1에서 선형 거르기 = 2×2 평균.
        let src = match &self.half {
            Some(half) => {
                set_camera(&Camera2D {
                    zoom: vec2(2.0 / (iw / 2) as f32, 2.0 / (ih / 2) as f32),
                    target: vec2((iw / 2) as f32 * 0.5, (ih / 2) as f32 * 0.5),
                    render_target: Some(half.clone()),
                    ..Default::default()
                });
                src.set_filter(FilterMode::Linear);
                draw_texture_ex(
                    src,
                    0.0,
                    0.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2((iw / 2) as f32, (ih / 2) as f32)),
                        ..Default::default()
                    },
                );
                src.set_filter(FilterMode::Nearest);
                set_default_camera();
                &half.texture
            }
            None => src,
        };
        draw_texture_ex(
            src,
            ox,
            oy,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(dw, dh)),
                // 렌더 타깃은 위아래가 바뀌어 나온다 — `post`에 옮길 때는 그대로 두고(법선 타깃과 같은 방향으로 읽는다) 화면에서 한 번만 뒤집는다.
                flip_y: true,
                ..Default::default()
            },
        );
        Frame {
            ox,
            oy,
            w: dw,
            h: dh,
            // 그린 화면 높이에 비례(0046): 화면에 맞춘 내부 해상도는 높이가 360이 아닐 수 있다(브라우저 창 323줄 등).
            ui: dh / UI_REF_H,
        }
    }
}

/// 아지랑이 세기 배(0063): 카메라 앞 방향의 y(내려다볼수록 −)로 — 쿼터뷰(숙임 ≈ 35°, −0.57) 1, 숄더뷰(≈ 19°, −0.33) 0.
pub(super) fn haze_by_pitch(fwd_y: f32) -> f32 {
    ((-fwd_y - HAZE_PITCH.0) / (HAZE_PITCH.1 - HAZE_PITCH.0)).clamp(0.0, 1.0)
}
/// 아지랑이가 0 → 다 켜지는 카메라 숙임(앞 방향 −y).
const HAZE_PITCH: (f32, f32) = (0.36, 0.52);

/// 하늘빛 아지랑이 띠: 위 끝 진하기 0.9에서 `frac × h` 아래로 0이 된다. 렌더 타깃은 화면에 뒤집혀(`flip_y`) 나가므로
/// 이 2D 좌표의 y = h 쪽이 화면 위다(처음엔 y = 0에 그려 화면 아래에 깔렸다 — 스크린숏으로 잡음).
pub(super) fn draw_haze(w: f32, h: f32, frac: f32) {
    let sky = Color::new(0.22, 0.21, 0.33, 0.9);
    let clear = Color::new(0.22, 0.21, 0.33, 0.0);
    let band = h * (1.0 - frac);
    let v = |x: f32, y: f32, c: Color| Vertex::new(x, y, 0.0, 0.0, 0.0, c);
    draw_mesh(&Mesh {
        vertices: vec![
            v(0.0, h, sky),
            v(w, h, sky),
            v(w, band, clear),
            v(0.0, band, clear),
        ],
        indices: vec![0, 1, 2, 0, 2, 3],
        texture: None,
    });
}

/// 법선 타깃(0061): 내부 해상도 그대로, 자기 깊이(가림)를 붙이고, 가장 가까운 픽셀로 읽는다.
fn edge_rt((w, h): (u32, u32)) -> RenderTarget {
    let rt = render_target_ex(
        w,
        h,
        RenderTargetParams {
            depth: true,
            ..Default::default()
        },
    );
    rt.texture.set_filter(FilterMode::Nearest);
    rt
}

/// 선 패스 둘(0061, GL에서만 — 외곽선 재질과 같은 조건).
/// 법선 재질: 화면(뷰) 법선 xy와 선형 깊이(0..64m를 두 바이트로 — 약 1mm)를 쓴다.
/// 선 재질: 이웃 넷(1px)을 본다 — 뒤가 거리의 `Edge.x`배 넘게 멀면 이 픽셀은 앞 물체의 가장자리 → 어둡게(`Edge.y`).
/// 같은 면 근처에서 이웃 법선과의 차가 `Edge.w`를 넘고 그 차가 이웃 쪽을 향하면(법선이 바깥으로 벌어짐 = 볼록) 가까운 쪽 픽셀만 밝게(`Edge.z`).
/// 오목 모서리는 긋지 않는다(t3ssel8r의 밝은 선은 볼록에만 — 2차).
/// 법선 패스 셰이더(0061) — 상주 지형(0186)도 같은 것을 쓴다.
pub(super) const NORMAL_VS: &str = r#"#version 100
    attribute vec3 position;
    attribute vec2 texcoord;
    attribute vec4 color0;
    attribute vec4 normal;
    varying highp vec3 vn;
    varying highp float vd;
    uniform mat4 Model;
    uniform mat4 Projection;
    uniform mat4 View;
    // 바람(0142): 법선 칸 넷째 값 − 1 = 흔들리는 몫. 양수 = 풀·잎(느린 돌풍 + 플레이어가 밟으면 눕는다), 음수 = 불꽃(빠른 일렁임).
    // Wind = (끊은 시각, 세기, 플레이어 x, 플레이어 z).
    uniform vec4 Wind;
    vec3 sway(vec3 w, float s) {
        float ph = w.x * 0.9 + w.z * 0.7;
        if (s > 0.0) {
            float g = sin(Wind.x * 1.7 + ph) * 0.6 + sin(Wind.x * 3.3 + ph * 1.9) * 0.4;
            w.x += g * 0.075 * s * Wind.y;
            w.z += g * 0.045 * s * Wind.y;
            vec2 d = w.xz - Wind.zw;
            float push = clamp(1.0 - length(d) / 0.9, 0.0, 1.0) * min(s, 1.0);
            w.xz += normalize(d + vec2(0.0001, 0.0)) * push * 0.28;
            w.y -= push * 0.14;
        } else if (s < 0.0) {
            float f = -s;
            float g = sin(Wind.x * 9.0 + ph * 5.0) * 0.6 + sin(Wind.x * 14.3 + ph * 3.0) * 0.4;
            w.x += g * 0.07 * f;
            w.z += g * 0.05 * f;
            w.y += (sin(Wind.x * 11.0 + ph) * 0.5 + 0.5) * 0.12 * f;
        }
        return w;
    }
    void main() {
        vec4 wp = Model * vec4(position, 1.0);
        wp.xyz = sway(wp.xyz, normal.w - 1.0);
        gl_Position = Projection * wp;
        vd = -(View * wp).z;
        vn = (View * vec4(normal.xyz, 0.0)).xyz;
    }"#;
pub(super) const NORMAL_FS: &str = r#"#version 100
    #ifdef GL_FRAGMENT_PRECISION_HIGH
    precision highp float;
    #else
    precision mediump float;
    #endif
    varying highp vec3 vn;
    varying highp float vd;
    void main() {
        vec3 n = length(vn) > 0.0001 ? normalize(vn) : vec3(0.0, 0.0, 1.0);
        float v = clamp(vd / 64.0, 0.0, 0.99998);
        float hi = floor(v * 255.0) / 255.0;
        float lo = fract(v * 255.0);
        gl_FragColor = vec4(n.xy * 0.5 + 0.5, hi, lo);
    }"#;

fn edge_fx(size: (u32, u32)) -> Option<EdgeFx> {
    let ctx = unsafe { get_internal_gl().quad_context };
    if ctx.info().backend != miniquad::Backend::OpenGl {
        return None;
    }
    const EDGE_VS: &str = r#"#version 100
    attribute vec3 position;
    attribute vec2 texcoord;
    attribute vec4 color0;
    varying lowp vec4 color;
    varying highp vec2 uv;
    uniform mat4 Model;
    uniform mat4 Projection;
    void main() {
        gl_Position = Projection * Model * vec4(position, 1.0);
        color = color0 / 255.0;
        uv = texcoord;
    }"#;
    const EDGE_FS: &str = r#"#version 100
    #ifdef GL_FRAGMENT_PRECISION_HIGH
    precision highp float;
    #else
    precision mediump float;
    #endif
    varying lowp vec4 color;
    varying highp vec2 uv;
    uniform sampler2D Texture;
    uniform sampler2D Normals;
    uniform vec2 Texel;
    uniform vec4 Edge;
    uniform vec4 Grade;
    // 번질 빛(0140): 밝고 따뜻한 화소만(불꽃·핵·눈·자물쇠) — 파스텔 옷·달빛 웅덩이는 번지지 않는다.
    vec3 hot(vec2 o) {
        vec3 s = texture2D(Texture, uv + o * Texel).rgb;
        float l = dot(s, vec3(0.3, 0.6, 0.1));
        return s * clamp((s.r - s.b) * 4.0, 0.0, 1.0) * clamp((l - 0.42) * 4.0, 0.0, 1.0);
    }
    float dep(vec2 c) {
        vec4 n = texture2D(Normals, c);
        return (n.b + n.a / 255.0) * 64.0;
    }
    vec2 nrm(vec2 c) {
        return texture2D(Normals, c).rg * 2.0 - 1.0;
    }
    void probe(vec2 dir, float d, vec2 n, inout float dark, inout float light) {
        vec2 c = uv + dir * Texel;
        float dn = dep(c);
        float gap = Edge.x * d;
        if (dn - d > gap) { dark = 1.0; return; }
        if (abs(dn - d) > gap) return;
        vec2 df = nrm(c) - n;
        if (length(df) > Edge.w && dot(df, dir) > 0.0 && d <= dn) light = 1.0;
    }
    void main() {
        vec4 col = texture2D(Texture, uv) * color;
        float d = dep(uv);
        vec2 n = nrm(uv);
        float dark = 0.0;
        float light = 0.0;
        if (d < 63.0 && Grade.w > 0.5) {
            probe(vec2(1.0, 0.0), d, n, dark, light);
            probe(vec2(-1.0, 0.0), d, n, dark, light);
            probe(vec2(0.0, 1.0), d, n, dark, light);
            probe(vec2(0.0, -1.0), d, n, dark, light);
        }
        vec3 rgb = col.rgb;
        if (dark > 0.5) {
            rgb = mix(rgb, vec3(0.05, 0.04, 0.08), Edge.y);
        } else if (light > 0.5) {
            rgb = mix(rgb, min(rgb * 1.7 + 0.06, vec3(1.0)), Edge.z);
        }
        // 마무리 패스(0140). 번짐: 이웃 열여섯 칸(내부 px 단위 — 도트가 뭉개지지 않는다)의 밝고 따뜻한 빛을 더한다.
        if (Grade.y > 0.0) {
            vec3 g = hot(vec2(1.0, 0.0)) + hot(vec2(-1.0, 0.0)) + hot(vec2(0.0, 1.0)) + hot(vec2(0.0, -1.0));
            g += 0.7 * (hot(vec2(2.0, 1.0)) + hot(vec2(-2.0, -1.0)) + hot(vec2(-1.0, 2.0)) + hot(vec2(1.0, -2.0)));
            g += 0.4 * (hot(vec2(3.0, -2.0)) + hot(vec2(-3.0, 2.0)) + hot(vec2(2.0, 3.0)) + hot(vec2(-2.0, -3.0)));
            g += 0.22 * (hot(vec2(5.0, 1.0)) + hot(vec2(-5.0, -1.0)) + hot(vec2(-1.0, 5.0)) + hot(vec2(1.0, -5.0)));
            rgb += g * (0.085 * Grade.y);
        }
        // 톤 곡선: 어두운 중간을 올리고 밝은 끝은 그대로(x(1+a)/(1+ax) — 0과 1은 제자리).
        float a = 0.5 * Grade.x;
        rgb = rgb * (1.0 + a) / (1.0 + a * rgb);
        // 가장자리 어둡게: 가운데(주인공)로 눈이 모이게.
        vec2 q = (uv - 0.5) * vec2(1.0, 0.85);
        rgb *= 1.0 - Grade.z * smoothstep(0.28, 0.72, length(q));
        gl_FragColor = vec4(min(rgb, vec3(1.0)), col.a);
    }"#;
    let normal_mat = load_material(
        ShaderSource::Glsl {
            vertex: NORMAL_VS,
            fragment: NORMAL_FS,
        },
        MaterialParams {
            pipeline_params: PipelineParams {
                depth_write: true,
                depth_test: Comparison::LessOrEqual,
                ..Default::default()
            },
            uniforms: vec![
                UniformDesc::new("View", UniformType::Mat4),
                UniformDesc::new("Wind", UniformType::Float4),
            ],
            ..Default::default()
        },
    )
    .map_err(|e| {
        warn!("edge normal material: {e:?}");
    })
    .ok()?;
    let edge_mat = load_material(
        ShaderSource::Glsl {
            vertex: EDGE_VS,
            fragment: EDGE_FS,
        },
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("Texel", UniformType::Float2),
                UniformDesc::new("Edge", UniformType::Float4),
                UniformDesc::new("Grade", UniformType::Float4),
            ],
            textures: vec!["Normals".to_string()],
            ..Default::default()
        },
    )
    .map_err(|e| {
        warn!("edge material: {e:?}");
    })
    .ok()?;
    Some(EdgeFx {
        rt: edge_rt(size),
        size,
        normal_mat,
        edge_mat,
    })
}

impl Renderer {
    pub(super) fn horizon_haze(&self, t: &Tuning, fwd: Vec3, iw: u32, ih: u32) {
        // 지평선 아지랑이(0012): 렌더 타깃 위쪽을 하늘빛으로 옅게 — 화면에 고정된 수평 띠라 땅 돌리기 중에도 기울지 않는다.
        // 하늘이 보이는 숄더뷰에서는 아지랑이를 걷는다(0063 E2): 화면 위 = 하늘이라 띠가 달·산등성이를 지운다.
        // 내려다보는 쿼터뷰(화면 위 = 먼 땅)에서만 다 — 카메라 숙임으로 부드럽게(전환 중에도 튀지 않게).
        let haze = t[crate::tuning::P::Haze] * haze_by_pitch(fwd.y);
        if haze > 0.0 {
            set_camera(&Camera2D {
                zoom: vec2(2.0 / iw as f32, 2.0 / ih as f32),
                target: vec2(iw as f32 * 0.5, ih as f32 * 0.5),
                render_target: Some(self.rt.clone()),
                ..Default::default()
            });
            draw_haze(iw as f32, ih as f32, haze);
        }
    }
}
