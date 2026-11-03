//! 그리기. 월드를 읽기만 한다. 3D 패스는 내부 해상도 렌더 타깃(PS1 느낌)에, 글·메뉴는 화면 패스(`hud.rs`·`app`).
//!
//! 스테이지(0019): 굽힌 지형(`scenery.rs`) + 모델(`models.rs` 목록, 상태 → 동작 이름) + 빛(달빛 + 화로, 정점 색). 모델이 없는 종류는
//! 상자로. 개발 시나리오(스테이지 없음)는 PoC의 회색 아레나 그대로.
//! 바닥 예고는 모양마다(원판·부채꼴·띠·고리, `TeleShape`) 옅은 면 + 테두리, 채움이 자라 테두리에 닿는 순간 공격(nail 0186~0188).
use macroquad::prelude::*;

use crate::camera::{Camera, CAM_MIN_DIST};
use crate::models::{Entry, Library};
use crate::scenery::Scenery;
use crate::stage::{Light, Phase, StageRun};
use crate::tuning::{Tuning, INTERNAL_H, INTERNAL_W, PLAYER_HEIGHT, PLAYER_RADIUS, TICK_DT};
use crate::world::{Aabb, Atk, EState, Enemy, PState, TeleShape, World};

pub struct Renderer {
    rt: RenderTarget,
    /// 시나리오 상자마다 지금 진하기(1 = 불투명). 가리면 부드럽게 옅어지고 풀리면 돌아온다(0008).
    fade: Vec<f32>,
    pub lib: Library,
    /// 굽힌 스테이지(어느 레벨인지 = 지도 글의 주소).
    scenery: Option<(usize, Scenery)>,
    pose: Vec<Mat4>,
    batch: Batch,
}

/// 측정값(개발 줄, P5): 우리 코드가 한 프레임에 쓴 CPU 시간(평균·최대, 지난 1초), 그리기 호출 수, 모델 정점 수.
#[derive(Clone, Copy, Default, Debug)]
pub struct Perf {
    pub cpu_avg_ms: f32,
    pub cpu_max_ms: f32,
    pub frame_avg_ms: f32,
    /// 배치가 보낸 횟수(모델·지형). 예전엔 macroquad telemetry로 셌다 — 그게 텍스처를 흘렸다(0020).
    pub draws: usize,
    pub draws_max: usize,
}

/// 그린 화면의 자리(글·메뉴가 같은 배율로 붙는다).
#[derive(Clone, Copy, Debug)]
pub struct Frame {
    pub ox: f32,
    pub oy: f32,
    pub w: f32,
    pub h: f32,
    /// 정수 배율(논리 px 기준 — 물리 픽셀로 센 정수를 DPI로 나눈 값).
    pub scale: f32,
}

/// 한 프레임에 그릴 것.
pub struct View<'a> {
    pub world: &'a World,
    pub cam: &'a Camera,
    pub alpha: f32,
    pub t: &'a Tuning,
    pub stage: Option<&'a StageRun>,
}

/// 예고 색(붉은 계열 하나 — 붉은색은 위험에만, 그림 팔레트에는 없다).
const TELE: Color = Color::new(0.85, 0.15, 0.1, 1.0);

fn with_a(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a)
}

/// 빛(정점 색에 곱할 rgb): 바탕 + 달빛(고정 방향) + 가까운 화로(거리 제곱으로 준다, 면이 그쪽을 볼수록 밝다).
pub fn lit(p: Vec3, n: Vec3, lights: &[Light]) -> Vec3 {
    let l = LIGHT_DIR.normalize();
    let mut c = AMBIENT + MOON * n.dot(l).max(0.0);
    for li in lights {
        let d = li.pos - p;
        let dist = d.length();
        if dist >= li.radius {
            continue;
        }
        let att = (1.0 - dist / li.radius).powi(2);
        let facing = 0.35 + 0.65 * n.dot(d / dist.max(1e-3)).max(0.0);
        c += vec3(li.color.r, li.color.g, li.color.b) * (FIRE * att * facing);
    }
    c
}

const AMBIENT: Vec3 = Vec3::new(0.26, 0.27, 0.35);
const MOON: Vec3 = Vec3::new(0.42, 0.45, 0.58);
const FIRE: f32 = 1.5;
/// 움직이는 모델에 곱하는 밝기(배경과 떼어 읽히게).
const ACTOR_LIFT: f32 = 1.35;

impl Renderer {
    pub fn new() -> Self {
        let rt = render_target(INTERNAL_W, INTERNAL_H);
        rt.texture.set_filter(FilterMode::Nearest);
        Renderer {
            rt,
            fade: Vec::new(),
            lib: Library::default(),
            scenery: None,
            pose: Vec::new(),
            batch: Batch::default(),
        }
    }

    /// 지난 프레임에 보낸 모델 정점 수(개발 줄).
    pub fn vertices(&self) -> usize {
        self.batch.last_vertices
    }

    /// 지난 프레임에 배치가 보낸 횟수(그리기 호출의 큰 몫, 개발 줄). 진짜 GL 호출 수는 `tools/test/leak.js`가 잰다.
    pub fn batches(&self) -> usize {
        self.batch.last_sends
    }

    /// 3D 패스를 렌더 타깃에 그리고 화면에 정수 배율로 올린다. 글은 돌려준 `Frame`에 맞춰 바깥에서.
    pub fn draw(&mut self, v: &View) -> Frame {
        let (w, cam, alpha, t) = (v.world, v.cam, v.alpha, v.t);
        let aspect = INTERNAL_W as f32 / INTERNAL_H as f32;
        set_camera(&cam.to_camera3d(self.rt.clone(), aspect));
        clear_background(Color::new(0.035, 0.035, 0.05, 1.0));

        // 가림 판정점: 플레이어 몸(발·허리·머리 × 좌우)과 락온 대상.
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
        let k = 1.0 - (-10.0 * get_frame_time()).exp();
        let lights: Vec<Light> = match v.stage {
            Some(run) => {
                let key = run.def.map.as_ptr() as usize;
                if self.scenery.as_ref().is_none_or(|(k, _)| *k != key) {
                    self.scenery = Some((key, Scenery::bake(&run.level, &mut self.lib)));
                }
                let sc = &mut self.scenery.as_mut().unwrap().1;
                draw_plane(
                    Vec3::Y * -0.03,
                    vec2(sc.extent, sc.extent),
                    None,
                    Color::new(0.03, 0.03, 0.045, 1.0),
                );
                let boxes: Vec<Aabb> = sc.pieces.iter().filter_map(|p| p.aabb).collect();
                let hidden = crate::camera::occluders(&pts, cam.rig.pos, &boxes);
                let mut bi = 0;
                for p in &mut sc.pieces {
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
                sc.lights.clone()
            }
            None => {
                self.sandbox_floor(w, &pts, cam.rig.pos, k, t);
                Vec::new()
            }
        };

        // 바닥 예고·그림자(배우보다 먼저).
        for en in &w.enemies {
            if !en.alive() {
                continue;
            }
            let p = en.prev.lerp(en.pos, alpha);
            shadow(p, en.radius * 1.2);
            if let Some((k, shape)) = en.telegraph() {
                telegraph(p, en.facing, k, shape);
            }
            if let Some(front) = en.wave_front() {
                ring(p, front, 0.0, 360.0, with_a(TELE, 0.75));
                ring(p, (front - 0.5).max(0.1), 0.0, 360.0, with_a(TELE, 0.3));
            }
        }
        let pp = w.player.prev.lerp(w.player.pos, alpha);
        shadow(pp, PLAYER_RADIUS * 1.3);

        // 스테이지 물건: 문·핵.
        if let Some(run) = v.stage {
            self.stage_props(run, w, &lights);
        }

        // 적.
        for en in &w.enemies {
            let fall = if en.alive() {
                0.0
            } else {
                (en.dead_t / DEATH_TIME).min(1.0)
            };
            if fall >= 1.0 {
                continue;
            }
            self.enemy(en, w, alpha, v.stage, &lights, fall);
        }
        // 플레이어.
        self.player(w, cam, pp, t, &lights);
        self.batch.flush();

        self.overlays(w, pp, alpha, t);
        // 모아 둔 선(윤곽선·예고 옆선)을 한꺼번에 — 옅은 덩어리보다 먼저(그 뒤의 선이 가려지지 않게).
        flush_lines();

        // 옅어진 덩어리는 맨 나중에: 면은 거의 투명하게.
        match v.stage {
            Some(_) => {
                let sc = &self.scenery.as_ref().unwrap().1;
                for p in sc.pieces.iter().filter(|p| p.fade < 0.98) {
                    self.batch.raw(&p.verts, &p.idx, p.tex.as_ref(), p.fade);
                }
                self.batch.send();
            }
            None => self.sandbox_faded(w, t),
        }

        // 지평선 아지랑이(0012): 렌더 타깃 위쪽을 하늘빛으로 옅게 — 화면에 고정된 수평 띠라 땅 돌리기 중에도 기울지 않는다.
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
        self.present()
    }

    /// 렌더 타깃을 화면 가운데에 정수 배율로.
    fn present(&self) -> Frame {
        set_default_camera();
        clear_background(BLACK);
        let (sw, sh) = (screen_width(), screen_height());
        // 정수 배율은 물리 픽셀로 센다: 폰(DPR 3)의 논리 844×390에서는 논리로 세면 1배(480×270)라 화면 반이 검정이었다(0006).
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
        Frame {
            ox,
            oy,
            w: dw,
            h: dh,
            scale,
        }
    }

    /// 문(열리면 땅으로 가라앉거나 `open` 동작)과 핵(문지기가 남긴 것, 떠서 돈다).
    fn stage_props(&mut self, run: &StageRun, w: &World, lights: &[Light]) {
        if let Some((g, c)) = run.level.gate {
            let k = (run.gate_open / GATE_TIME).clamp(0.0, 1.0);
            if k < 1.0 {
                match self.lib.get("gate") {
                    Some(e) => {
                        let open = e.pick(&["open"]);
                        let sink = if open.is_some() { 0.0 } else { k * 4.6 };
                        e.model
                            .pose(open, k * e.duration(open) * 0.999, &mut self.pose);
                        let inst = Mat4::from_translation(c - Vec3::Y * sink);
                        self.batch.model(e, &self.pose, inst, WHITE, lights, 1.0);
                    }
                    None => {
                        let mut b = g;
                        b.max.y -= k * 4.5;
                        shaded_box(b.center(), b.size(), Color::new(0.25, 0.2, 0.22, 1.0), 0.6);
                    }
                }
            }
        }
        if let (Some(core), Phase::Core) = (run.core, run.phase) {
            let bob = (w.time * 2.0).sin() * 0.15;
            let p = core + Vec3::Y * (0.9 + bob);
            match self.lib.get("core_crystal") {
                Some(e) => {
                    let a = e.pick(&["pulse", "idle"]);
                    e.model.pose(a, w.time, &mut self.pose);
                    let inst = Mat4::from_translation(p) * Mat4::from_rotation_y(w.time * 1.2);
                    self.batch.model(e, &self.pose, inst, WHITE, lights, 1.0);
                }
                None => shaded_box(p, Vec3::splat(0.45), Color::new(1.0, 0.7, 0.3, 1.0), 0.2),
            }
            ring(
                core,
                1.0 + 0.15 * (w.time * 3.0).sin(),
                0.0,
                360.0,
                Color::new(1.0, 0.75, 0.35, 0.6),
            );
        }
    }

    fn enemy(
        &mut self,
        en: &Enemy,
        w: &World,
        alpha: f32,
        stage: Option<&StageRun>,
        lights: &[Light],
        fall: f32,
    ) {
        let p = en.prev.lerp(en.pos, alpha);
        let mut tilt = Quat::IDENTITY;
        let mut squash = 1.0;
        if let EState::Stagger { t: e, dur } = en.state {
            let left = ((dur - e) / 0.25).clamp(0.0, 1.0);
            let v = lean(en.sway, en.sway_t, w.time, left);
            if v.length() > 1e-4 {
                tilt = Quat::from_axis_angle(Vec3::Y.cross(v).normalize(), v.length());
            }
            // 움찔: 처음 순간 눌렸다 펴진다.
            squash = 1.0 - 0.2 * en.sway.length() * (-12.0 * en.sway_t).exp();
        }
        // 죽음: 뒤로 넘어지며 가라앉는다.
        if fall > 0.0 {
            let back = -en.facing_dir();
            tilt = Quat::from_axis_angle(Vec3::Y.cross(back).normalize(), fall.min(0.5) * 2.6);
        }
        let sink = Vec3::Y * (-0.6 * (fall - 0.5).max(0.0) * 2.0);
        // 끌린 자국: 미끄러지는 동안 발 뒤에 흙.
        let speed = en.knock.length();
        if speed > 1.5 && fall == 0.0 {
            let back = -en.knock / speed;
            for i in 1..=3 {
                let k = i as f32;
                draw_cube(
                    p + back * (0.22 * k) + Vec3::Y * 0.03,
                    vec3(0.22, 0.05, 0.22) * (1.0 - 0.2 * k),
                    None,
                    Color::new(0.45, 0.4, 0.33, 0.8 - 0.2 * k),
                );
            }
        }
        let flash = en.hurt > 0.0;
        let model = en.def().model;
        // 문지기가 깨어나는 동안 눈·핵이 밝아진다.
        let glow = match (en.state, stage) {
            (EState::Dormant, Some(run)) => 0.15 + 0.85 * run.wake_progress(),
            _ if en.roused => 1.0,
            _ => 0.85,
        };
        if let Some(e) = self.lib.get(model) {
            let (anim, at) = enemy_anim(en, e, w.time, stage);
            e.model.pose(anim, at, &mut self.pose);
            let inst = Mat4::from_translation(p + sink)
                * Mat4::from_quat(tilt)
                * Mat4::from_rotation_y(en.facing)
                * Mat4::from_scale(vec3(1.0, squash, 1.0));
            let tint = if flash {
                Color::new(1.8, 1.8, 1.8, 1.0)
            } else {
                WHITE
            };
            self.batch.model(e, &self.pose, inst, tint, lights, glow);
        } else {
            let base = if en.boss {
                Color::new(0.45, 0.2, 0.5, 1.0)
            } else {
                Color::new(0.5, 0.35, 0.25, 1.0)
            };
            let col = if flash { WHITE } else { base };
            let crouch = matches!(en.state, EState::Tele { .. } | EState::Carried);
            let h = en.height * if crouch { 0.85 } else { 1.0 } * squash;
            actor(p + sink, en.facing_dir(), en.radius, h, col, 0.6, tilt);
        }
    }

    fn player(&mut self, w: &World, cam: &Camera, pp: Vec3, t: &Tuning, lights: &[Light]) {
        let pl = &w.player;
        // 무적 깜빡임: 짝수 프레임은 밝게.
        let blink = pl.hurt > 0.0 && (w.time * 20.0) as i32 % 2 == 0;
        // 카메라가 몸에 닿을 만큼 가까우면 옅게(가림 당기기의 마지막 수단).
        let close = (cam.closeness(pp) - CAM_MIN_DIST).clamp(0.0, 1.0);
        let a = 0.25 + 0.75 * close;
        if let Some(e) = self.lib.get("wanderer") {
            let speed = (pl.pos - pl.prev).length() / TICK_DT;
            let (names, at): (&[&str], f32) = match pl.state {
                PState::Attack { t: e, .. } => (&["attack"], e),
                PState::Dodge { t: e, .. } => (&["dodge", "walk"], e),
                PState::Hurt { t: e, .. } => (&["hurt", "idle"], e),
                PState::Turn { t: e, .. } => (&["turn", "idle"], e),
                PState::Idle if speed > 0.5 => (&["walk"], w.time * (speed / 4.0).clamp(0.6, 1.6)),
                PState::Idle => (&["idle"], w.time),
            };
            let anim = e.pick(names);
            let looped = matches!(pl.state, PState::Idle);
            let dur = e.duration(anim);
            let at = if looped { at } else { at.min(dur * 0.999) };
            e.model.pose(anim, at, &mut self.pose);
            // 맞으면 몸이 밀리는 쪽으로 젖혀졌다 떨며 돌아온다(0020) — 발을 축으로, 경직 동안.
            let recoil = match pl.state {
                PState::Hurt { t: e, dur } => {
                    let left = (1.0 - e / dur.max(1e-3)).clamp(0.0, 1.0);
                    let away = vec3(pl.vel.x, 0.0, pl.vel.z).normalize_or_zero();
                    let jitter = (w.time * 70.0).sin() * 0.08 * left;
                    let ang = RECOIL * left * left + jitter;
                    if away.length() > 0.5 {
                        Quat::from_axis_angle(Vec3::Y.cross(away).normalize(), ang)
                    } else {
                        Quat::IDENTITY
                    }
                }
                _ => Quat::IDENTITY,
            };
            let inst = Mat4::from_translation(pp)
                * Mat4::from_quat(recoil)
                * Mat4::from_rotation_y(pl.facing);
            let tint = if blink {
                Color::new(1.8, 1.8, 1.8, a)
            } else {
                Color::new(1.0, 1.0, 1.0, a)
            };
            self.batch.model(e, &self.pose, inst, tint, lights, 1.0);
        } else {
            let pcol = match pl.state {
                _ if blink => WHITE,
                PState::Dodge { .. } => Color::new(0.6, 0.9, 1.0, 0.6),
                PState::Hurt { .. } => Color::new(0.95, 0.45, 0.4, 1.0),
                PState::Attack { .. } => Color::new(1.0, 0.85, 0.5, 1.0),
                PState::Turn { .. } => Color::new(0.7, 0.95, 0.75, 1.0),
                PState::Idle => Color::new(0.85, 0.75, 0.55, 1.0),
            };
            actor(
                pp,
                pl.facing_dir(),
                PLAYER_RADIUS,
                PLAYER_HEIGHT,
                with_a(pcol, pcol.a * a),
                t[crate::tuning::P::Shade],
                Quat::IDENTITY,
            );
        }
    }

    /// 배우 위에 얹는 것: 스윙 부채꼴·쟁반 테·휘청 별·락온 표시.
    fn overlays(&self, w: &World, pp: Vec3, alpha: f32, t: &Tuning) {
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
                    Color::new(1.0, 0.95, 0.7, 0.35),
                );
            }
        }
        // 땅 돌리기(0012): 쟁반의 테 — 멈추는 순간 범위 고리가 밝게 떴다 옅어지고, 안쪽 물결이 바깥으로 퍼진다.
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
        // 락온 표시: 대상 머리 위 작은 마름모 + 땅의 고리.
        if let Some(tg) = w.lock.and_then(|i| w.enemies.get(i)).filter(|e| e.alive()) {
            let p = tg.prev.lerp(tg.pos, alpha);
            let bob = (w.time * 4.0).sin() * 0.1;
            let top = p + Vec3::Y * (tg.height + 0.9 + bob);
            let s = 0.25;
            for (a, b) in [
                (Vec3::X, Vec3::Y),
                (Vec3::Y, -Vec3::X),
                (-Vec3::X, -Vec3::Y),
                (-Vec3::Y, Vec3::X),
            ] {
                line(top + a * s, top + b * s, YELLOW);
            }
            ring(
                p,
                tg.radius + 0.3,
                0.0,
                360.0,
                Color::new(1.0, 0.9, 0.3, 0.8),
            );
        }
    }

    /// 개발 시나리오의 바닥(PoC 아레나): 판 + 격자 + 낮은 턱 + 회색 기둥(가리면 옅게).
    fn sandbox_floor(&mut self, w: &World, pts: &[Vec3], cam_pos: Vec3, k: f32, t: &Tuning) {
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

    fn sandbox_faded(&self, w: &World, t: &Tuning) {
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

/// 맞았을 때 몸이 젖혀지는 각(라디안, 경직 처음).
const RECOIL: f32 = 0.35;
/// 죽은 적이 넘어져 사라지는 데 걸리는 초.
const DEATH_TIME: f32 = 1.2;
/// 문이 다 열리는 초(스테이지가 `gate_open`을 이만큼까지 올린다).
const GATE_TIME: f32 = 1.5;

/// 적 상태 → (동작, 그 동작의 몇 초). 후보 이름을 차례로 찾고(모델마다 가진 동작이 다르다), 예고 동안은 진행에 맞춰 든다:
/// `_windup` 동작이면 예고 전체에, 한 동작(`attack`)이면 앞 절반이 예고 · 뒤 절반이 치기.
fn enemy_anim(en: &Enemy, e: &Entry, clock: f32, stage: Option<&StageRun>) -> (Option<usize>, f32) {
    let phase = (en.pos.x * 0.37 + en.pos.z * 0.61).rem_euclid(1.0);
    let moving = (en.pos - en.prev).length() > 0.005;
    let windup: &[&str] = match en.atk {
        Atk::Charge => &["charge_windup", "attack"],
        Atk::Slam => &["slam", "attack"],
        Atk::Wave => &["wave", "slam", "attack"],
        Atk::Smash => &["attack"],
    };
    let once = |names: &[&str], k: f32| {
        let a = e.pick(names);
        (a, k.clamp(0.0, 0.999) * e.duration(a))
    };
    let is_windup =
        |a: Option<usize>| a.is_some_and(|i| e.model.anims[i].name.ends_with("_windup"));
    match en.state {
        EState::Dormant => {
            let w = stage.map(|s| s.wake_progress()).unwrap_or(0.0);
            match e.pick(&["wake"]) {
                Some(a) => (Some(a), w.clamp(0.0, 0.999) * e.duration(Some(a))),
                None => (e.pick(&["idle"]), 0.0),
            }
        }
        EState::Chase if moving => (e.pick(&["walk", "run", "idle"]), clock * 1.0 + phase),
        EState::Chase => (e.pick(&["idle", "walk"]), clock + phase),
        EState::Tele { t } => {
            let k = t / en.tele_dur().max(1e-3);
            let a = e.pick(windup);
            if is_windup(a) {
                once(windup, k)
            } else {
                once(windup, k * 0.5)
            }
        }
        EState::Swing { t } => {
            let a = e.pick(windup);
            if is_windup(a) {
                once(&["attack"], 0.5 + t * 2.0)
            } else {
                once(windup, 0.5 + t * 2.0)
            }
        }
        EState::Charge { .. } => (e.pick(&["charge", "run", "walk"]), clock * 1.5),
        EState::Wave { t } => once(&["wave", "slam", "attack"], 0.5 + t),
        EState::Recover { t } => match e.pick(&["recover"]) {
            Some(a) => (Some(a), t.min(e.duration(Some(a)) * 0.999)),
            None => (e.pick(&["idle"]), clock + phase),
        },
        EState::Carried | EState::Stagger { .. } => once(&["hurt", "idle"], 0.3),
        EState::Dead => once(&["hurt", "idle"], 0.999),
    }
}

/// 바닥 예고 하나: 옅은 면 + 테두리, 채움이 자라 테두리에 닿는 순간 공격. 원판·부채꼴·고리는 가운데부터, 띠는 가운데 줄에서 양옆으로.
fn telegraph(p: Vec3, facing: f32, k: f32, shape: TeleShape) {
    let bg = with_a(TELE, 0.16);
    let fill = with_a(TELE, 0.42);
    let edge = with_a(TELE, 0.7);
    match shape {
        TeleShape::Disc { r } => {
            sector(p, r, facing, 360.0, bg);
            sector(p, r * k, facing, 360.0, fill);
            ring(p, r, facing, 360.0, edge);
        }
        TeleShape::Sector { r, arc } => {
            sector(p, r, facing, arc, bg);
            sector(p, r * k, facing, arc, fill);
            ring(p, r, facing, arc, edge);
        }
        TeleShape::Lane { len, w } => {
            let (s, c) = facing.sin_cos();
            let dir = vec3(s, 0.0, c);
            lane(p, dir, len, w, bg);
            lane(p, dir, len, w * k, fill);
            lane_edge(p, dir, len, w, edge);
        }
        TeleShape::Ring { r } => {
            sector(p, r, facing, 360.0, with_a(TELE, 0.1));
            ring(p, r * k, facing, 360.0, fill);
            ring(p, r, facing, 360.0, edge);
        }
    }
}

/// 바닥 띠(시작 `p`, 방향 `dir`, 길이 `len`, 폭 `w`).
fn lane(p: Vec3, dir: Vec3, len: f32, w: f32, col: Color) {
    let side = vec3(-dir.z, 0.0, dir.x) * (w * 0.5);
    let y = Vec3::Y * 0.02;
    let a = p + y - side;
    let b = p + y + side;
    let v = |q: Vec3| Vertex::new(q.x, q.y, q.z, 0.0, 0.0, col);
    draw_mesh(&Mesh {
        vertices: vec![v(a), v(b), v(b + dir * len), v(a + dir * len)],
        indices: vec![0, 1, 2, 0, 2, 3],
        texture: None,
    });
}

fn lane_edge(p: Vec3, dir: Vec3, len: f32, w: f32, col: Color) {
    let side = vec3(-dir.z, 0.0, dir.x) * (w * 0.5);
    let y = Vec3::Y * 0.03;
    let (a, b) = (p + y - side, p + y + side);
    let (c, d) = (b + dir * len, a + dir * len);
    for (x, z) in [(a, b), (b, c), (c, d), (d, a)] {
        line(x, z, col);
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

/// 모델·굽힌 덩어리를 한 배치로 모은다(P4·P5): 부위 행렬 × 정점을 CPU에서 곱하고 빛(`lit`)을 정점 색에 굽는다.
/// macroquad는 그리기 호출 하나에 정점 10,000·색인 5,000 **미만**만 받고 넘치면 잘라 버린다
/// (0.4.16 `quad_gl.rs` `geometry`, `Conf::draw_call_*_capacity` 기본값 — 처음엔 40마리를 한 덩어리로 보내 아무것도 안 그려졌다).
/// 그래서 그 아래에서 끊어 보낸다. 텍스처가 바뀌어도 끊는다.
const MAX_DRAW_VERTICES: usize = 10_000;
const MAX_DRAW_INDICES: usize = 5_000;

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
}

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
        Self::bake_glow(verts, idx, e, pose, inst, tint, lights, 1.0);
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
    ) {
        let m = &e.model;
        for (i, node) in m.nodes.iter().enumerate() {
            let Some(mi) = node.mesh else { continue };
            let mesh = &m.meshes[mi];
            let mat = inst * pose[i];
            let base = verts.len() as u16;
            let bright = e.glow.get(i).copied().unwrap_or(false);
            for k in 0..mesh.pos.len() {
                let p = mat.transform_point3(mesh.pos[k]);
                let f = if bright {
                    Vec3::splat(glow)
                } else {
                    let n = mat.transform_vector3(mesh.nrm[k]).normalize_or_zero();
                    lit(p, n, lights)
                };
                let c = Color::from_rgba(
                    to_u8(tint.r * f.x),
                    to_u8(tint.g * f.y),
                    to_u8(tint.b * f.z),
                    to_u8(tint.a),
                );
                verts.push(Vertex::new(p.x, p.y, p.z, mesh.uv[k].x, mesh.uv[k].y, c));
            }
            idx.extend(mesh.idx.iter().map(|&j| base + j as u16));
        }
    }

    fn room(&mut self, tex: Option<&Texture2D>, nv: usize, ni: usize) {
        if self.tex.as_ref() != tex {
            self.send();
            self.tex = tex.cloned();
        }
        if self.vertices.len() + nv >= MAX_DRAW_VERTICES
            || self.indices.len() + ni >= MAX_DRAW_INDICES
        {
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
        Self::bake_glow(
            &mut self.vertices,
            &mut self.indices,
            e,
            pose,
            inst,
            lift,
            &near,
            glow,
        );
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
        draw_mesh(&Mesh {
            vertices: std::mem::take(&mut self.vertices),
            indices: std::mem::take(&mut self.indices),
            texture: self.tex.clone(),
        });
    }

    /// 프레임마다 한 번(모은 것을 보내고 수를 개발 줄로).
    pub fn flush(&mut self) {
        self.send();
        self.last_vertices = self.this_frame;
        self.this_frame = 0;
        self.last_sends = self.sends;
        self.sends = 0;
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
