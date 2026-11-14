//! 땅: 구운 지형(가린 덩어리는 옅게, 바람 재질)과 개발 아레나의 바닥·기둥.
use super::*;

impl Renderer {
    /// 개발 시나리오의 바닥(PoC 아레나): 판 + 격자 + 낮은 턱 + 회색 기둥(가리면 옅게).
    pub(super) fn sandbox_floor(
        &mut self,
        w: &World,
        pts: &[Vec3],
        cam_pos: Vec3,
        k: f32,
        t: &Tuning,
    ) {
        let a = w.arena;
        draw_plane(
            Vec3::Y * -0.02,
            vec2(a * 4.0, a * 4.0),
            None,
            Color::new(0.11, 0.1, 0.09, 1.0),
        );
        draw_plane(
            Vec3::ZERO,
            vec2(a, a),
            None,
            Color::new(0.22, 0.2, 0.18, 1.0),
        );
        draw_grid_ex(
            ((a * 2.0) as u32).max(2),
            1.0,
            Color::new(0.3, 0.27, 0.24, 1.0),
            Color::new(0.26, 0.24, 0.21, 1.0),
            Vec3::Y * 0.005,
            Quat::IDENTITY,
        );
        for (x, z, sx, sz) in [
            (0.0, -a, a * 2.0, 0.3),
            (0.0, a, a * 2.0, 0.3),
            (-a, 0.0, 0.3, a * 2.0),
            (a, 0.0, 0.3, a * 2.0),
        ] {
            draw_cube(
                vec3(x, 0.3, z),
                vec3(sx, 0.6, sz),
                None,
                Color::new(0.16, 0.14, 0.13, 1.0),
            );
        }
        let shade = t[crate::tuning::P::Shade];
        let hidden = crate::camera::occluders(pts, cam_pos, &w.obstacles);
        if self.fade.len() != w.obstacles.len() {
            self.fade = vec![1.0; w.obstacles.len()];
        }
        for (i, f) in self.fade.iter_mut().enumerate() {
            let want = if hidden.contains(&i) { FADED } else { 1.0 };
            *f += (want - *f) * k;
        }
        for (i, b) in w.obstacles.iter().enumerate() {
            if self.fade[i] < 0.98 {
                continue;
            }
            shaded_box(
                b.center(),
                b.size(),
                Color::new(0.35, 0.32, 0.3, 1.0),
                shade,
            );
            box_wires(
                b.center(),
                b.size(),
                Color::new(0.12, 0.1, 0.1, 1.0),
                Quat::IDENTITY,
                b.center(),
            );
        }
    }

    pub(super) fn sandbox_faded(&self, w: &World, t: &Tuning) {
        let shade = t[crate::tuning::P::Shade];
        for (i, b) in w.obstacles.iter().enumerate() {
            let f = self.fade.get(i).copied().unwrap_or(1.0);
            if f >= 0.98 {
                continue;
            }
            shaded_box(b.center(), b.size(), Color::new(0.35, 0.32, 0.3, f), shade);
            let edge = (1.0 - f) / (1.0 - FADED);
            draw_cube_wires(
                b.center(),
                b.size(),
                Color::new(0.9, 0.85, 0.8, 0.12 + 0.2 * edge),
            );
        }
    }
}

/// 지형 재질(0142): 기본 재질과 같되 꼭짓점을 바람으로 민다(풀 끝·잎 무더기·불꽃 — `scenery::sway`가 실은 몫). 못 만들면 없음(바람 없이).
pub(super) fn wind_material() -> Option<Material> {
    const VS: &str = r#"#version 100
    attribute vec3 position;
    attribute vec2 texcoord;
    attribute vec4 color0;
    attribute vec4 normal;
    varying lowp vec4 color;
    varying lowp vec2 uv;
    uniform mat4 Model;
    uniform mat4 Projection;
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
        vec4 w = Model * vec4(position, 1.0);
        w.xyz = sway(w.xyz, normal.w - 1.0);
        gl_Position = Projection * w;
        color = color0 / 255.0;
        uv = texcoord;
    }"#;
    const FS: &str = r#"#version 100
    precision mediump float;
    varying lowp vec4 color;
    varying lowp vec2 uv;
    uniform sampler2D Texture;
    void main() {
        gl_FragColor = color * texture2D(Texture, uv);
    }"#;
    load_material(
        ShaderSource::Glsl {
            vertex: VS,
            fragment: FS,
        },
        MaterialParams {
            pipeline_params: PipelineParams {
                depth_write: true,
                depth_test: Comparison::LessOrEqual,
                color_blend: Some(miniquad::BlendState::new(
                    miniquad::Equation::Add,
                    miniquad::BlendFactor::Value(miniquad::BlendValue::SourceAlpha),
                    miniquad::BlendFactor::OneMinusValue(miniquad::BlendValue::SourceAlpha),
                )),
                ..Default::default()
            },
            uniforms: vec![UniformDesc::new("Wind", UniformType::Float4)],
            ..Default::default()
        },
    )
    .map_err(|e| {
        warn!("wind material: {e:?}");
    })
    .ok()
}

/// 가린 기둥의 면 진하기(0 = 안 보임). 윤곽선은 따로 남는다.
pub(super) const FADED: f32 = 0.08;

impl Renderer {
    /// 구운 지형(0033): 필요하면 다시 굽고, 빛 지도 바닥 → 나머지(가린 덩어리는 옅어지게 두고 건너뛴다). 돌려주는 것은 이 프레임의 빛(화로 + 갈 곳).
    pub(super) fn terrain(
        &mut self,
        run: &StageRun,
        w: &World,
        cam: &Camera,
        pts: &[Vec3],
        k: f32,
        t: &Tuning,
    ) -> Vec<Light> {
        // 툰을 켜고 끄면 지형을 다시 굽는다(빛이 정점에 구워져 있다, 0033).
        // 식생(0063)을 끄고 켜도 다시 굽는다.
        let flora = t[crate::tuning::P::RenderFlora] >= 0.5;
        // 조용한 바닥·빛 대비(0140)도 굽는 것이라 바꾸면 다시 굽는다.
        let key = run.def.map.as_ptr() as usize
            + look().toon as usize
            + 2 * flora as usize
            + 4 * look().calm as usize
            + 8 * look().key as usize
            // 꺼진 화로(0171)에 불이 붙으면 다시 굽는다(빛·불꽃이 정점에 구워져 있다).
            + 16 * run.level.cold.is_some() as usize;
        if self.scenery.as_ref().is_none_or(|(k, _)| *k != key) {
            let trees = run.def.env.trees;
            self.scenery = Some((key, Scenery::bake(&run.level, &mut self.lib, flora, trees)));
        }
        let sc = &mut self.scenery.as_mut().unwrap().1;
        // 그늘에 선 배우는 달빛을 덜 받는다(0140) — 구운 그림자와 같은 가림 지도.
        set_actor_shade(look().key.then(|| sc.occ.clone()));
        draw_plane(
            Vec3::Y * sc.void_y,
            vec2(sc.extent, sc.extent),
            None,
            self.backdrop.env.fog_deep,
        );
        let boxes: Vec<Aabb> = sc.pieces.iter().filter_map(|p| p.aabb).collect();
        let hidden = crate::camera::occluders(pts, cam.view.pos, &boxes);
        // 빛 지도 바닥(0044)은 캐릭터 그림자 가림판을 읽는 재질로(0045). 가림판이 없으면 기본 재질 그대로.
        let floor = match (&self.floor_mat, &self.shadow_rt, self.light_vp) {
            (Some(m), Some(rt), Some(vp)) => {
                m.set_uniform("LightVP", vp);
                m.set_uniform("Shade", floor_shade());
                m.set_texture("ShadowMask", rt.texture.clone());
                Some(m)
            }
            _ => None,
        };
        if let Some(m) = floor {
            gl_use_material(m);
        }
        for p in sc.pieces.iter().filter(|p| p.lightmap) {
            self.batch.raw(&p.verts, &p.idx, p.tex.as_ref(), 1.0);
        }
        self.batch.send();
        if floor.is_some() {
            gl_use_default_material();
        }
        // 바람(0142): 풀 끝·잎·불꽃이 흔들린다 — 그 몫이 실린 정점만 움직이고 나머지 지형은 그대로.
        if let Some(m) = &self.wind_mat {
            m.set_uniform("Wind", self.wind_uniform);
            gl_use_material(m);
        }
        let mut bi = 0;
        for p in sc.pieces.iter_mut().filter(|p| !p.lightmap) {
            let want = match p.aabb {
                Some(_) => {
                    let h = hidden.contains(&bi);
                    bi += 1;
                    if h {
                        FADED
                    } else {
                        1.0
                    }
                }
                None => 1.0,
            };
            p.fade += (want - p.fade) * k;
            if p.fade >= 0.98 {
                self.batch.raw(&p.verts, &p.idx, p.tex.as_ref(), 1.0);
            }
        }
        self.batch.send();
        if self.wind_mat.is_some() {
            gl_use_default_material();
        }
        let mut lights = sc.lights.clone();
        // 갈 곳(0060 W1)에 호박색 빛 — 곁의 모델이 같이 물든다("호박색 불빛 = 갈 길").
        if let Some(g) = run.goal() {
            let pulse = 0.85 + 0.15 * (w.time * GUIDE_PULSE).sin();
            lights.push(Light {
                pos: g + Vec3::Y * 1.5,
                color: Color::new(
                    GUIDE_COLOR.r * pulse,
                    GUIDE_COLOR.g * pulse,
                    GUIDE_COLOR.b * pulse,
                    1.0,
                ),
                radius: GUIDE_LIGHT_R,
                shaft: false,
            });
        }
        lights
    }

    /// 옅어진 문·지형 덩어리(맨 나중에 — 면은 거의 투명하게) + 높이 안개 + 떠다니는 것.
    pub(super) fn faded_terrain(
        &mut self,
        run: &StageRun,
        w: &World,
        t: &Tuning,
        lights: &[Light],
        cam_right: Vec3,
        cam_up: Vec3,
    ) {
        if self.gate_fade < 0.98 {
            let a = self.gate_fade;
            self.draw_gate(run, lights, a);
        }
        let sc = &self.scenery.as_ref().unwrap().1;
        if let Some(m) = &self.wind_mat {
            gl_use_material(m);
        }
        for p in sc.pieces.iter().filter(|p| p.fade < 0.98) {
            self.batch.raw(&p.verts, &p.idx, p.tex.as_ref(), p.fade);
        }
        self.batch.send();
        if self.wind_mat.is_some() {
            gl_use_default_material();
        }
        // 높이 안개(0063 E1): 걷는 땅 아래만 — 카메라가 보는 곳(플레이어) 둘레.
        if t[crate::tuning::P::RenderFog] >= 0.5 {
            self.backdrop.fog(
                &mut self.batch,
                w.player.pos,
                sc.walk_min,
                sc.void_y,
                w.time,
            );
        }
        // 떠다니는 것(0063 E4): 반딧불·달빛 먼지 — 카메라를 보는 작은 판.
        if t[crate::tuning::P::RenderMotes] >= 0.5 {
            self.backdrop.motes(
                &mut self.batch,
                w.player.pos,
                cam_right,
                cam_up,
                w.time,
                w.ground.as_ref(),
            );
        }
    }
}
