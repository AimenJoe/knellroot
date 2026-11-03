//! 그리기. 월드를 읽기만 한다. 3D 패스는 내부 해상도 렌더 타깃(PS1 느낌)에, HUD는 화면 패스에.
//! 상자·구, 그리고 P4 스파이크의 glTF 시험 모델(`model.rs`, CPU에서 부위 행렬을 곱해 한 배치로 — `ModelBatch`). 바닥 예고는 삼각형 부채꼴 `Mesh`를 매 프레임 만든다(정점은 어차피 매 프레임 올라간다).
use macroquad::prelude::*;

use crate::camera::{Camera, Mode, CAM_MIN_DIST};
use crate::model::Model;
use crate::scenario::Entry;
use crate::tuning::{Tuning, INTERNAL_H, INTERNAL_W, PLAYER_HEIGHT, PLAYER_RADIUS};
use crate::world::{EState, PState, World};

pub struct Renderer {
    rt: RenderTarget,
    /// 상자마다 지금 진하기(1 = 불투명). 가리면 부드럽게 옅어지고 풀리면 돌아온다(0008).
    fade: Vec<f32>,
    /// P4 시험 모델과 그 텍스처(처음 쓸 때 올린다).
    critter: Option<(Model, Texture2D)>,
    pose: Vec<Mat4>,
    batch: ModelBatch,
}

/// 측정값(HUD 한 줄, P5): 우리 코드가 한 프레임에 쓴 CPU 시간(평균·최대, 지난 1초), 그리기 호출 수, 모델 정점 수.
#[derive(Clone, Copy, Default, Debug)]
pub struct Perf {
    pub cpu_avg_ms: f32,
    pub cpu_max_ms: f32,
    pub frame_avg_ms: f32,
    pub draws: usize,
    /// 장면을 띄운 뒤 잰 가장 많은 그리기 호출(macroquad 호출 칸 = GPU 버퍼 수, 줄지 않는다).
    pub draws_max: usize,
}

impl Renderer {
    pub fn new() -> Self {
        let rt = render_target(INTERNAL_W, INTERNAL_H);
        rt.texture.set_filter(FilterMode::Nearest);
        Renderer {
            rt,
            fade: Vec::new(),
            critter: None,
            pose: Vec::new(),
            batch: ModelBatch::default(),
        }
    }

    fn critter(&mut self) -> Option<&(Model, Texture2D)> {
        if self.critter.is_none() {
            match Model::from_bytes(crate::model::TEST_CRITTER.as_bytes()) {
                Ok(m) => {
                    let png = m.texture_png.clone().unwrap_or_default();
                    let tex = Texture2D::from_file_with_format(&png, Some(ImageFormat::Png));
                    tex.set_filter(FilterMode::Nearest);
                    self.critter = Some((m, tex));
                }
                Err(e) => {
                    error!("test model: {e}");
                    return None;
                }
            }
        }
        self.critter.as_ref()
    }

    /// `touch`면 HUD의 키 안내 줄을 빼고(손가락에는 안 맞는 글), 랩 터치 줄 자리를 비운다.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        w: &World,
        cam: &Camera,
        alpha: f32,
        t: &Tuning,
        entry: &Entry,
        touch: bool,
        perf: &Perf,
    ) {
        let aspect = INTERNAL_W as f32 / INTERNAL_H as f32;
        set_camera(&cam.to_camera3d(self.rt.clone(), aspect));
        clear_background(Color::new(0.07, 0.06, 0.08, 1.0));

        // 바닥 + 격자(거리감). 경기장 밖도 어두운 땅으로 — 끝에 가면 검은 허공이 화면을 채워 어색했다(0008).
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
        // 아레나 벽(낮은 턱).
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
        // 카메라에서 플레이어 몸(발·허리·머리 × 좌우)과 락온 대상을 가리는 기둥은 배우 뒤에 옅게(카메라를 당기지 않는다, 0007·0008).
        let tg = w
            .lock
            .and_then(|i| w.enemies.get(i))
            .filter(|e| e.alive())
            .map(|e| (e.pos, e.height));
        let pts = crate::camera::view_points(
            w.player.pos,
            PLAYER_HEIGHT,
            PLAYER_RADIUS + 0.15,
            cam.rig.pos,
            tg,
        );
        let hidden = crate::camera::occluders(&pts, cam.rig.pos, &w.obstacles);
        if self.fade.len() != w.obstacles.len() {
            self.fade = vec![1.0; w.obstacles.len()];
        }
        let k = 1.0 - (-10.0 * get_frame_time()).exp();
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

        // 바닥 예고·그림자(먼저, 배우보다 아래).
        for en in &w.enemies {
            if !en.alive() {
                continue;
            }
            let p = en.prev.lerp(en.pos, alpha);
            shadow(p, en.radius * 1.2);
            if let Some(k) = en.telegraph(t) {
                let arc = if en.boss {
                    t[crate::tuning::P::BossSwingArc]
                } else {
                    360.0
                };
                let r = en.reach + PLAYER_RADIUS;
                // 옅은 면 + 테두리, 채움이 가운데서 자라 테두리에 닿는 순간 공격(nail 0186~0188).
                sector(p, r, en.facing, arc, Color::new(0.85, 0.15, 0.1, 0.18));
                sector(p, r * k, en.facing, arc, Color::new(0.85, 0.15, 0.1, 0.45));
                ring(p, r, en.facing, arc, Color::new(0.85, 0.15, 0.1, 0.6));
            }
        }
        let pp = w.player.prev.lerp(w.player.pos, alpha);
        shadow(pp, PLAYER_RADIUS * 1.3);

        // 배우: 상자 몸 + 코(바라보는 쪽).
        for en in &w.enemies {
            if !en.alive() {
                continue;
            }
            let p = en.prev.lerp(en.pos, alpha);
            let base = if en.boss {
                Color::new(0.45, 0.2, 0.5, 1.0)
            } else {
                Color::new(0.5, 0.35, 0.25, 1.0)
            };
            let col = if en.hurt > 0.0 { WHITE } else { base };
            // 예고 중에는 웅크리고, 땅에 실려 가는 동안엔 버티느라 조금 웅크린다.
            let crouch = matches!(en.state, EState::Tele { .. } | EState::Carried);
            let mut h = if crouch { en.height * 0.85 } else { en.height };
            let mut tilt = Quat::IDENTITY;
            if let EState::Stagger { t: e, dur } = en.state {
                let left = ((dur - e) / 0.25).clamp(0.0, 1.0);
                let v = lean(en.sway, en.sway_t, w.time, left);
                if v.length() > 1e-4 {
                    tilt = Quat::from_axis_angle(Vec3::Y.cross(v).normalize(), v.length());
                }
                // 움찔: 처음 순간 눌렸다 펴진다.
                h *= 1.0 - 0.2 * en.sway.length() * (-12.0 * en.sway_t).exp();
            }
            // 끌린 자국: 미끄러지는 동안 발 뒤에 흙.
            let speed = en.knock.length();
            if speed > 1.5 {
                let back = -en.knock / speed;
                for i in 1..=3 {
                    let k = i as f32;
                    draw_cube(
                        p + back * (0.22 * k) + Vec3::Y * 0.03,
                        vec3(0.22, 0.05, 0.22) * (1.0 - 0.2 * k),
                        None,
                        Color::new(0.55, 0.48, 0.38, 0.8 - 0.2 * k),
                    );
                }
            }
            let as_model = !en.boss && t[crate::tuning::P::RenderModel] >= 0.5;
            if as_model && self.critter().is_some() {
                let (m, tex) = self.critter.as_ref().unwrap();
                // 발을 축으로 기울기 → 바라보는 쪽(모델 앞 = +Z) → 키에 맞춘 크기. 걷기는 적마다 위상을 어긋나게.
                let inst = Mat4::from_translation(p)
                    * Mat4::from_quat(tilt)
                    * Mat4::from_rotation_y(en.facing)
                    * Mat4::from_scale(Vec3::splat(h / 1.4));
                let phase = (en.pos.x * 0.37 + en.pos.z * 0.61).rem_euclid(1.0);
                m.pose(m.anim("walk"), w.time + phase, &mut self.pose);
                let tint = if en.hurt > 0.0 {
                    WHITE
                } else {
                    Color::new(1.0, 1.0, 1.0, 1.0)
                };
                self.batch.push(m, tex, &self.pose, inst, tint, shade);
            } else {
                actor(p, en.facing_dir(), en.radius, h, col, shade, tilt);
            }
            if en.boss {
                // 체력 띠(머리 위).
                let k = en.hp as f32 / en.max_hp as f32;
                draw_cube(
                    p + Vec3::Y * (en.height + 0.4),
                    vec3(2.0, 0.08, 0.08),
                    None,
                    Color::new(0.1, 0.1, 0.1, 1.0),
                );
                draw_cube(
                    p + Vec3::Y * (en.height + 0.4) - Vec3::X * (1.0 - k),
                    vec3(2.0 * k, 0.1, 0.1),
                    None,
                    RED,
                );
            }
        }
        self.batch.flush();
        let pcol = match w.player.state {
            _ if w.player.hurt > 0.0 && (w.time * 20.0) as i32 % 2 == 0 => WHITE,
            PState::Dodge { .. } => Color::new(0.6, 0.9, 1.0, 0.6),
            PState::Attack { .. } => Color::new(1.0, 0.85, 0.5, 1.0),
            PState::Turn { .. } => Color::new(0.7, 0.95, 0.75, 1.0),
            PState::Idle => Color::new(0.85, 0.75, 0.55, 1.0),
        };
        // 카메라가 몸에 닿을 만큼 가까우면 옅게(가림 당기기의 마지막 수단).
        let close = (cam.closeness(pp) - CAM_MIN_DIST).clamp(0.0, 1.0);
        let pcol = Color::new(pcol.r, pcol.g, pcol.b, pcol.a * (0.25 + 0.75 * close));
        actor(
            pp,
            w.player.facing_dir(),
            PLAYER_RADIUS,
            PLAYER_HEIGHT,
            pcol,
            shade,
            Quat::IDENTITY,
        );
        // 스윙 부채꼴(활성 구간만).
        if let PState::Attack { t: e, .. } = w.player.state {
            let (s0, s1) = (
                t[crate::tuning::P::AttackStartup],
                t[crate::tuning::P::AttackStartup] + t[crate::tuning::P::AttackActive],
            );
            if e >= s0 && e < s1 {
                sector(
                    pp + Vec3::Y * 0.9,
                    t[crate::tuning::P::AttackRange],
                    w.player.facing,
                    t[crate::tuning::P::AttackArc],
                    Color::new(1.0, 0.95, 0.7, 0.5),
                );
            }
        }
        // 땅 돌리기(0012): 쟁반의 테 — 멈추는 순간 범위 고리가 밝게 떴다 옅어지고, 안쪽 물결이 바깥으로 퍼진다(어디까지 쏟아지는지 배우게).
        // 멈추는 순간(쏟는 틱)부터 0.5초.
        let shown = 0.5;
        if w.player.since_spill < shown {
            let k = (w.player.since_spill / shown).clamp(0.0, 1.0);
            let range = t[crate::tuning::P::SpillRange];
            ring(
                pp,
                range,
                0.0,
                360.0,
                Color::new(0.7, 0.95, 0.75, 0.8 * (1.0 - k)),
            );
            ring(
                pp,
                range * (0.25 + 0.75 * k),
                0.0,
                360.0,
                Color::new(0.7, 0.95, 0.75, 0.5 * (1.0 - k)),
            );
        }
        // 휘청: 머리 위를 도는 작은 별 셋.
        for en in &w.enemies {
            if let EState::Stagger { t: e, dur } = en.state {
                let p = en.prev.lerp(en.pos, alpha);
                let fade = ((dur - e) / 0.2).clamp(0.0, 1.0);
                for i in 0..3 {
                    let a = w.time * 6.0 + i as f32 * std::f32::consts::TAU / 3.0;
                    let r = en.radius + 0.15;
                    draw_cube(
                        p + vec3(a.cos() * r, en.height + 0.3, a.sin() * r),
                        Vec3::splat(0.16),
                        None,
                        Color::new(1.0, 0.92, 0.35, fade),
                    );
                }
            }
        }
        // 락온 표시: 대상 머리 위 작은 구 + 땅의 고리.
        if let Some(tg) = w.lock.and_then(|i| w.enemies.get(i)).filter(|e| e.alive()) {
            let p = tg.prev.lerp(tg.pos, alpha);
            let bob = (w.time * 4.0).sin() * 0.1;
            draw_sphere_wires(p + Vec3::Y * (tg.height + 0.9 + bob), 0.25, None, YELLOW);
            ring(
                p,
                tg.radius + 0.3,
                0.0,
                360.0,
                Color::new(1.0, 0.9, 0.3, 0.8),
            );
        }

        // 모아 둔 선(윤곽선·예고 옆선)을 한꺼번에 — 옅은 기둥보다 먼저(그 뒤의 선이 가려지지 않게).
        flush_lines();

        // 옅어진(옅어지는) 기둥은 맨 나중에: 면은 거의 투명하게, 윤곽만 밝게 남겨 자리가 보이게.
        for (i, b) in w.obstacles.iter().enumerate() {
            let f = self.fade[i];
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

        // 지평선 아지랑이(0012): 렌더 타깃 위쪽을 하늘빛으로 옅게 — 위 끝은 짙고 아래로 사라진다. 화면에 고정된 수평 띠라
        // 땅 돌리기(수직축 회전) 중에도 기울거나 움직이지 않는다(진짜 지평선과 같은 움직임).
        let haze = t[crate::tuning::P::Haze];
        if haze > 0.0 {
            set_camera(&Camera2D {
                zoom: vec2(2.0 / INTERNAL_W as f32, 2.0 / INTERNAL_H as f32),
                target: vec2(INTERNAL_W as f32 * 0.5, INTERNAL_H as f32 * 0.5),
                render_target: Some(self.rt.clone()),
                ..Default::default()
            });
            draw_haze(INTERNAL_W as f32, INTERNAL_H as f32, haze);
        }

        // 화면 패스: 정수 배율로 가운데에.
        set_default_camera();
        clear_background(BLACK);
        let (sw, sh) = (screen_width(), screen_height());
        // 정수 배율은 물리 픽셀로 센다: 폰(DPR 3)의 논리 844×390에서는 논리로 세면 1배(480×270)라 화면 반이 검정이었다.
        // 물리 2532×1170이면 4배 = 1920×1080 물리 = 논리 640×360. 데스크톱(DPR 1)은 전과 같다.
        let dpi = screen_dpi_scale().max(0.01);
        let scale = ((sw * dpi) / INTERNAL_W as f32)
            .min((sh * dpi) / INTERNAL_H as f32)
            .floor()
            .max(1.0)
            / dpi;
        let (dw, dh) = (INTERNAL_W as f32 * scale, INTERNAL_H as f32 * scale);
        let (ox, oy) = (((sw - dw) * 0.5).floor(), ((sh - dh) * 0.5).floor());
        draw_texture_ex(
            &self.rt.texture,
            ox,
            oy,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(dw, dh)),
                flip_y: true,
                ..Default::default()
            },
        );
        self.hud(w, cam, ox, oy, entry, touch, perf);
    }

    #[allow(clippy::too_many_arguments)]
    fn hud(
        &self,
        w: &World,
        cam: &Camera,
        ox: f32,
        oy: f32,
        entry: &Entry,
        touch: bool,
        perf: &Perf,
    ) {
        let mode = match cam.mode {
            Mode::Quarter => "QUARTER",
            Mode::Lock => "LOCK-ON",
        };
        let blend = cam
            .blending()
            .map(|e| format!(" blending {e:.2}s"))
            .unwrap_or_default();
        draw_text(
            format!(
                "{} [{}]{}   cam {:.0}deg   fps {}",
                entry.name,
                mode,
                blend,
                cam.azimuth_deg(),
                get_fps()
            ),
            ox + 10.0,
            oy + 22.0,
            20.0,
            WHITE,
        );
        draw_text(
            format!(
                "{}: hp {}   kills {}   enemies {}   turn {}",
                w.name,
                w.player.hp,
                w.kills,
                w.enemies.iter().filter(|e| e.alive()).count(),
                if w.lock_target().is_some() {
                    "-".to_string()
                } else if w.player.turn_cd > 0.0 {
                    format!("{:.1}s", w.player.turn_cd)
                } else {
                    "ready".to_string()
                }
            ),
            ox + 10.0,
            oy + 44.0,
            20.0,
            LIGHTGRAY,
        );
        // 측정 줄(P5): 폰에서도 보이게 터치 판정 위에.
        draw_text(
            format!(
                "cpu {:.1}ms (max {:.1})  frame {:.1}ms  draws {} (max {})  verts {}",
                perf.cpu_avg_ms,
                perf.cpu_max_ms,
                perf.frame_avg_ms,
                perf.draws,
                perf.draws_max,
                self.batch.last_vertices
            ),
            ox + 10.0,
            oy + 62.0,
            16.0,
            YELLOW,
        );
        if touch {
            return;
        }
        draw_text(
            format!("checks: {}", entry.checks),
            ox + 10.0,
            oy + 80.0,
            16.0,
            GRAY,
        );
        draw_text(
            "WASD move  J/Space attack  K/Shift dodge  Q/E turn  L/Tab lock-on  F1 lab  1-7 scenario",
            ox + 10.0,
            oy + 98.0,
            16.0,
            GRAY,
        );
    }
}

/// 하늘빛 아지랑이 띠: 위 끝 진하기 0.9에서 `frac × h` 아래로 0이 된다. 렌더 타깃은 화면에 뒤집혀(`flip_y`) 나가므로
/// 이 2D 좌표의 y = h 쪽이 화면 위다(처음엔 y = 0에 그려 화면 아래에 깔렸다 — 스크린숏으로 잡음).
fn draw_haze(w: f32, h: f32, frac: f32) {
    let sky = Color::new(0.2, 0.2, 0.26, 0.9);
    let clear = Color::new(0.2, 0.2, 0.26, 0.0);
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

/// 모델 인스턴스를 한 배치로 모은다(P4·P5): 부위 행렬 × 정점을 CPU에서 곱하고, 면마다 고정 빛 음영을 정점 색에 굽는다
/// (`shaded_box`와 같은 식). macroquad는 그리기 호출 하나에 정점 10,000·색인 5,000 **미만**만 받고 넘치면 잘라 버린다
/// (0.4.16 `quad_gl.rs` `geometry`, `Conf::draw_call_*_capacity` 기본값 — 처음엔 40마리를 한 덩어리로 보내 아무것도 안 그려졌다).
/// 그래서 메시 단위로 그 아래에서 끊어 보낸다.
const MAX_DRAW_VERTICES: usize = 10_000;
const MAX_DRAW_INDICES: usize = 5_000;

#[derive(Default)]
pub struct ModelBatch {
    tex: Option<Texture2D>,
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    /// 지난 프레임에 보낸 모델 정점 수(HUD).
    pub last_vertices: usize,
    this_frame: usize,
}

impl ModelBatch {
    pub fn push(
        &mut self,
        m: &Model,
        tex: &Texture2D,
        pose: &[Mat4],
        inst: Mat4,
        tint: Color,
        shade: f32,
    ) {
        if self.tex.as_ref() != Some(tex) {
            self.send();
            self.tex = Some(tex.clone());
        }
        let l = LIGHT_DIR.normalize();
        for (i, node) in m.nodes.iter().enumerate() {
            let Some(mi) = node.mesh else { continue };
            let mesh = &m.meshes[mi];
            if self.vertices.len() + mesh.pos.len() >= MAX_DRAW_VERTICES
                || self.indices.len() + mesh.idx.len() >= MAX_DRAW_INDICES
            {
                self.send();
            }
            let mat = inst * pose[i];
            let base = self.vertices.len() as u16;
            for k in 0..mesh.pos.len() {
                let p = mat.transform_point3(mesh.pos[k]);
                let n = mat.transform_vector3(mesh.nrm[k]).normalize_or_zero();
                let f = 1.0 - shade + shade * n.dot(l).max(0.0);
                let c = Color::new(tint.r * f, tint.g * f, tint.b * f, tint.a);
                self.vertices
                    .push(Vertex::new(p.x, p.y, p.z, mesh.uv[k].x, mesh.uv[k].y, c));
            }
            self.indices
                .extend(mesh.idx.iter().map(|&j| base + j as u16));
        }
    }

    fn send(&mut self) {
        if self.vertices.is_empty() {
            return;
        }
        self.this_frame += self.vertices.len();
        draw_mesh(&Mesh {
            vertices: std::mem::take(&mut self.vertices),
            indices: std::mem::take(&mut self.indices),
            texture: self.tex.clone(),
        });
    }

    /// 프레임마다 한 번(모은 것을 보내고 수를 HUD로).
    pub fn flush(&mut self) {
        self.send();
        self.last_vertices = self.this_frame;
        self.this_frame = 0;
    }
}

/// 가린 기둥의 면 진하기(0 = 안 보임). 윤곽선은 따로 남는다.
const FADED: f32 = 0.08;

/// 휘청 기울기(가장 큰 각, 라디안). 세기 1(가장 멀리 미끄러진 적)이 이만큼 꺾인다.
const LEAN_MAX: f32 = 0.6;
/// 비틀비틀 작은 원 흔들림(라디안).
const DIZZY: f32 = 0.1;

/// 휘청 몸짓: 머리가 기우는 쪽(XZ) × 각(라디안). 버스가 급정거할 때처럼 발은 바닥에 끌리고 몸은 미끄러지는 쪽으로 꺾인다 —
/// 빠르게 기울었다 앞뒤로 두어 번 흔들리며 잦아들고(감쇠 진동), 휘청이 끝날 때까지 작게 원을 그린다(`left` 1 → 0이면 사라짐).
/// `sway` = 쏟아진 쪽 × 세기, `t` = 그때부터 지난 초, `clock` = 원 흔들림 위상.
pub fn lean(sway: Vec3, t: f32, clock: f32, left: f32) -> Vec3 {
    let k = sway.length();
    if k < 1e-4 {
        return Vec3::ZERO;
    }
    let dir = sway / k;
    // 감쇠 진동(초당 두 번): 0.125초에 가장 크게(≈ 0.7), 0.375초에 반대로 한 번, 1초면 거의 선다.
    let swing = (-3.0 * t).exp() * (std::f32::consts::TAU * 2.0 * t).sin();
    let (s, c) = (clock * 9.0).sin_cos();
    dir * (LEAN_MAX * k * swing) + vec3(c, 0.0, s) * (DIZZY * left)
}

/// 상자 몸 + 작은 코. `dir`은 바라보는 단위 벡터(XZ). `tilt`는 발(바닥 가운데)을 축으로 몸 전체를 기울인다.
fn actor(p: Vec3, dir: Vec3, r: f32, h: f32, col: Color, shade: f32, tilt: Quat) {
    let body = vec3(r * 2.0, h, r * 2.0);
    let center = p + Vec3::Y * (h * 0.5);
    shaded_box_rot(center, body, col, shade, tilt, p);
    box_wires(center, body, Color::new(0.05, 0.05, 0.05, 1.0), tilt, p);
    shaded_box_rot(
        p + dir * (r + 0.1) + Vec3::Y * (h * 0.8),
        vec3(0.2, 0.2, 0.2),
        Color::new(0.1, 0.1, 0.1, 1.0),
        shade,
        tilt,
        p,
    );
}

thread_local! {
    /// 이 프레임에 그을 선(나중에 한꺼번에). 면(삼각형)과 선을 번갈아 그리면 macroquad가 그리기 호출을 그때마다 끊는다 —
    /// 배우마다 면 → 윤곽선 → 코라서 잡몹 80이면 호출 ~180, 다 모여 예고를 띄우면 더. macroquad는 호출 칸마다 GPU 버퍼
    /// (정점 10,000개분 ≈ 400KB)를 잡고 줄이지 않아, 오래 두면 핸드헬드가 죽었다(사장 2026-11-03, 0016). 선을 모아 한 번에 그린다.
    static LINES: std::cell::RefCell<Vec<(Vec3, Vec3, Color)>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn line(a: Vec3, b: Vec3, col: Color) {
    LINES.with(|l| l.borrow_mut().push((a, b, col)));
}

fn flush_lines() {
    LINES.with(|l| {
        for (a, b, c) in l.borrow_mut().drain(..) {
            draw_line_3d(a, b, c);
        }
    });
}

/// 돌린 상자의 모서리 12개(`draw_cube_wires`는 축 정렬뿐). 바로 긋지 않고 `line`으로 모은다.
fn box_wires(center: Vec3, size: Vec3, col: Color, rot: Quat, pivot: Vec3) {
    let h = size * 0.5;
    let at = |x: f32, y: f32, z: f32| pivot + rot * (center + h * vec3(x, y, z) - pivot);
    for (a, b) in [
        ((-1., -1., -1.), (1., -1., -1.)),
        ((-1., 1., -1.), (1., 1., -1.)),
        ((-1., -1., 1.), (1., -1., 1.)),
        ((-1., 1., 1.), (1., 1., 1.)),
        ((-1., -1., -1.), (-1., 1., -1.)),
        ((1., -1., -1.), (1., 1., -1.)),
        ((-1., -1., 1.), (-1., 1., 1.)),
        ((1., -1., 1.), (1., 1., 1.)),
        ((-1., -1., -1.), (-1., -1., 1.)),
        ((1., -1., -1.), (1., -1., 1.)),
        ((-1., 1., -1.), (-1., 1., 1.)),
        ((1., 1., -1.), (1., 1., 1.)),
    ] {
        line(at(a.0, a.1, a.2), at(b.0, b.1, b.2), col);
    }
}

/// 고정 빛 방향(위·약간 앞·오른쪽). 면마다 `1 - shade + shade * max(0, n·l)`을 곱한다 — 기본 셰이더가 normal을 안 읽어 정점 색으로 굽는다
/// (KB reference/macroquad/3d-rendering.md). shade 0이면 민짜(전처럼), 1이면 그늘진 면이 검정에 가깝다.
const LIGHT_DIR: Vec3 = Vec3::new(0.35, 0.8, 0.45);

/// 면마다 음영이 다른 상자. 축 정렬. 정점 24개(면마다 넷), 매 프레임 만들어 보낸다(정점은 어차피 매 프레임 올라간다).
fn shaded_box(center: Vec3, size: Vec3, col: Color, shade: f32) {
    shaded_box_rot(center, size, col, shade, Quat::IDENTITY, center);
}

/// `shaded_box`를 `pivot` 둘레로 `rot`만큼 돌린 것. 음영도 돌린 면의 방향으로(빛은 고정).
fn shaded_box_rot(center: Vec3, size: Vec3, col: Color, shade: f32, rot: Quat, pivot: Vec3) {
    let h = size * 0.5;
    let l = LIGHT_DIR.normalize();
    let faces: [(Vec3, Vec3, Vec3); 6] = [
        (Vec3::Y, Vec3::X, Vec3::Z),
        (Vec3::NEG_Y, Vec3::Z, Vec3::X),
        (Vec3::Z, Vec3::X, Vec3::NEG_Y),
        (Vec3::NEG_Z, Vec3::NEG_Y, Vec3::X),
        (Vec3::X, Vec3::NEG_Y, Vec3::Z),
        (Vec3::NEG_X, Vec3::Z, Vec3::NEG_Y),
    ];
    let mut vertices = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);
    for (n, u, v) in faces {
        let k = 1.0 - shade + shade * (rot * n).dot(l).max(0.0);
        let c = Color::new(col.r * k, col.g * k, col.b * k, col.a);
        let base = vertices.len() as u16;
        for (su, sv) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
            let p = pivot + rot * (center + n * h + u * h * su + v * h * sv - pivot);
            vertices.push(Vertex::new(p.x, p.y, p.z, 0.0, 0.0, c));
        }
        // 뒷면 컬링이 없어 감기 방향은 상관없다.
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    draw_mesh(&Mesh {
        vertices,
        indices,
        texture: None,
    });
}

fn shadow(p: Vec3, r: f32) {
    sector(
        p + Vec3::Y * 0.01,
        r,
        0.0,
        360.0,
        Color::new(0.0, 0.0, 0.0, 0.35),
    );
}

/// 바닥 부채꼴(가운데 `c`, 반지름 `r`, 가운데 각 `yaw`, 벌림 `arc`도). 삼각형 팬 Mesh 하나.
fn sector(c: Vec3, r: f32, yaw: f32, arc: f32, col: Color) {
    let n = ((arc / 10.0).ceil() as usize).clamp(3, 48);
    let half = arc.to_radians() * 0.5;
    let mut vertices = Vec::with_capacity(n + 2);
    let mut indices = Vec::with_capacity(n * 3);
    let y = c.y + 0.02;
    vertices.push(Vertex::new(c.x, y, c.z, 0.0, 0.0, col));
    for i in 0..=n {
        let a = yaw - half + arc.to_radians() * (i as f32 / n as f32);
        let (s, k) = a.sin_cos();
        vertices.push(Vertex::new(c.x + s * r, y, c.z + k * r, 0.0, 0.0, col));
    }
    for i in 0..n {
        indices.extend_from_slice(&[0, (i + 1) as u16, (i + 2) as u16]);
    }
    draw_mesh(&Mesh {
        vertices,
        indices,
        texture: None,
    });
}

/// 부채꼴 테두리(띠). `arc` 360이면 고리.
fn ring(c: Vec3, r: f32, yaw: f32, arc: f32, col: Color) {
    let n = ((arc / 10.0).ceil() as usize).clamp(3, 48);
    let half = arc.to_radians() * 0.5;
    let y = c.y + 0.03;
    let w = 0.12;
    let mut vertices = Vec::with_capacity((n + 1) * 2);
    let mut indices = Vec::with_capacity(n * 6);
    for i in 0..=n {
        let a = yaw - half + arc.to_radians() * (i as f32 / n as f32);
        let (s, k) = a.sin_cos();
        vertices.push(Vertex::new(
            c.x + s * (r - w),
            y,
            c.z + k * (r - w),
            0.0,
            0.0,
            col,
        ));
        vertices.push(Vertex::new(c.x + s * r, y, c.z + k * r, 0.0, 0.0, col));
    }
    for i in 0..n {
        let b = (i * 2) as u16;
        indices.extend_from_slice(&[b, b + 1, b + 2, b + 1, b + 3, b + 2]);
    }
    if arc < 360.0 {
        // 부채꼴이면 양 옆 변도 긋는다.
        for a in [yaw - half, yaw + half] {
            let (s, k) = a.sin_cos();
            line(vec3(c.x, y, c.z), vec3(c.x + s * r, y, c.z + k * r), col);
        }
    }
    draw_mesh(&Mesh {
        vertices,
        indices,
        texture: None,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_spilled_enemy_leans_the_way_it_slides_then_rocks_back_and_settles() {
        let sway = vec3(1.0, 0.0, 0.0);
        // 처음엔 미끄러지는 쪽으로 크게(머리가 그쪽으로).
        let peak = (1..30)
            .map(|i| lean(sway, i as f32 / 60.0, 0.0, 0.0).x)
            .fold(f32::MIN, f32::max);
        assert!(peak > 0.3, "visible lean: {peak} rad");
        // 그다음 반대로 한 번 흔들린다.
        let back = (15..60)
            .map(|i| lean(sway, i as f32 / 60.0, 0.0, 0.0).x)
            .fold(f32::MAX, f32::min);
        assert!(back < -0.1, "rocks back: {back}");
        // 휘청이 끝날 즈음엔 거의 서 있다.
        assert!(lean(sway, 1.0, 0.0, 0.0).length() < 0.05);
        // 세기가 약하면(가까운 적·보스) 작게.
        let small = (1..30)
            .map(|i| lean(sway * 0.35, i as f32 / 60.0, 0.0, 0.0).x)
            .fold(f32::MIN, f32::max);
        assert!(small < peak * 0.5);
    }
}
