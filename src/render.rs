//! 그리기. 월드를 읽기만 한다. 3D 패스는 내부 해상도 렌더 타깃(PS1 느낌)에, HUD는 화면 패스에.
//! 상자·구만 쓴다(모델 로더는 PoC P4의 일). 바닥 예고는 삼각형 부채꼴 `Mesh`를 매 프레임 만든다(정점은 어차피 매 프레임 올라간다).
use macroquad::prelude::*;

use crate::camera::{Camera, Mode, CAM_MIN_DIST};
use crate::scenario::Entry;
use crate::tuning::{Tuning, INTERNAL_H, INTERNAL_W, PLAYER_HEIGHT, PLAYER_RADIUS};
use crate::world::{EState, PState, World};

pub struct Renderer {
    rt: RenderTarget,
    /// 상자마다 지금 진하기(1 = 불투명). 가리면 부드럽게 옅어지고 풀리면 돌아온다(0008).
    fade: Vec<f32>,
}

impl Renderer {
    pub fn new() -> Self {
        let rt = render_target(INTERNAL_W, INTERNAL_H);
        rt.texture.set_filter(FilterMode::Nearest);
        Renderer {
            rt,
            fade: Vec::new(),
        }
    }

    /// `touch`면 HUD의 키 안내 줄을 빼고(손가락에는 안 맞는 글), 랩 터치 줄 자리를 비운다.
    pub fn draw(
        &mut self,
        w: &World,
        cam: &Camera,
        alpha: f32,
        t: &Tuning,
        entry: &Entry,
        touch: bool,
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
            draw_cube_wires(b.center(), b.size(), Color::new(0.12, 0.1, 0.1, 1.0));
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
            let crouch = matches!(en.state, EState::Tele { .. });
            let h = if crouch { en.height * 0.85 } else { en.height };
            actor(p, en.facing_dir(), en.radius, h, col, shade);
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
        let pcol = match w.player.state {
            _ if w.player.hurt > 0.0 && (w.time * 20.0) as i32 % 2 == 0 => WHITE,
            PState::Dodge { .. } => Color::new(0.6, 0.9, 1.0, 0.6),
            PState::Attack { .. } => Color::new(1.0, 0.85, 0.5, 1.0),
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
        self.hud(w, cam, ox, oy, entry, touch);
    }

    fn hud(&self, w: &World, cam: &Camera, ox: f32, oy: f32, entry: &Entry, touch: bool) {
        let mode = match cam.mode {
            Mode::Quarter => "QUARTER",
            Mode::Lock => "LOCK-ON",
        };
        let blend = cam
            .blending()
            .map(|e| format!(" blending {e:.2}s"))
            .unwrap_or_default();
        draw_text(
            format!("{} [{}]{}   fps {}", entry.name, mode, blend, get_fps()),
            ox + 10.0,
            oy + 22.0,
            20.0,
            WHITE,
        );
        draw_text(
            format!(
                "{}: hp {}   kills {}   enemies {}",
                w.name,
                w.player.hp,
                w.kills,
                w.enemies.iter().filter(|e| e.alive()).count()
            ),
            ox + 10.0,
            oy + 44.0,
            20.0,
            LIGHTGRAY,
        );
        if touch {
            return;
        }
        draw_text(
            format!("checks: {}", entry.checks),
            ox + 10.0,
            oy + 66.0,
            16.0,
            GRAY,
        );
        draw_text(
            "WASD move  J/Space attack  K/Shift dodge  L/Tab lock-on  F1 lab  1-4 scenario",
            ox + 10.0,
            oy + 86.0,
            16.0,
            GRAY,
        );
    }
}

/// 가린 기둥의 면 진하기(0 = 안 보임). 윤곽선은 따로 남는다.
const FADED: f32 = 0.08;

/// 상자 몸 + 작은 코. `dir`은 바라보는 단위 벡터(XZ).
fn actor(p: Vec3, dir: Vec3, r: f32, h: f32, col: Color, shade: f32) {
    let body = vec3(r * 2.0, h, r * 2.0);
    shaded_box(p + Vec3::Y * (h * 0.5), body, col, shade);
    draw_cube_wires(
        p + Vec3::Y * (h * 0.5),
        body,
        Color::new(0.05, 0.05, 0.05, 1.0),
    );
    shaded_box(
        p + dir * (r + 0.1) + Vec3::Y * (h * 0.8),
        vec3(0.2, 0.2, 0.2),
        Color::new(0.1, 0.1, 0.1, 1.0),
        shade,
    );
}

/// 고정 빛 방향(위·약간 앞·오른쪽). 면마다 `1 - shade + shade * max(0, n·l)`을 곱한다 — 기본 셰이더가 normal을 안 읽어 정점 색으로 굽는다
/// (KB reference/macroquad/3d-rendering.md). shade 0이면 민짜(전처럼), 1이면 그늘진 면이 검정에 가깝다.
const LIGHT_DIR: Vec3 = Vec3::new(0.35, 0.8, 0.45);

/// 면마다 음영이 다른 상자. 축 정렬. 정점 24개(면마다 넷), 매 프레임 만들어 보낸다(정점은 어차피 매 프레임 올라간다).
fn shaded_box(center: Vec3, size: Vec3, col: Color, shade: f32) {
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
        let k = 1.0 - shade + shade * n.dot(l).max(0.0);
        let c = Color::new(col.r * k, col.g * k, col.b * k, col.a);
        let base = vertices.len() as u16;
        for (su, sv) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
            let p = center + n * h + u * h * su + v * h * sv;
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
            draw_line_3d(vec3(c.x, y, c.z), vec3(c.x + s * r, y, c.z + k * r), col);
        }
    }
    draw_mesh(&Mesh {
        vertices,
        indices,
        texture: None,
    });
}
