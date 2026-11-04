//! 그리기. 월드를 읽기만 한다. 3D 패스는 내부 해상도 렌더 타깃(PS1 느낌)에, 글·메뉴는 화면 패스(`hud.rs`·`app`).
//!
//! 스테이지(0019): 굽힌 지형(`scenery.rs`) + 모델(`models.rs` 목록, 상태 → 동작 이름) + 빛(달빛 + 화로, 정점 색). 모델이 없는 종류는
//! 상자로. 개발 시나리오(스테이지 없음)는 PoC의 회색 아레나 그대로.
//! 바닥 예고는 모양마다(원판·부채꼴·띠·고리, `TeleShape`) 옅은 면 + 테두리, 채움이 자라 테두리에 닿는 순간 공격(nail 0186~0188).
use macroquad::camera::Camera as _;
use macroquad::prelude::*;
use miniquad::{UniformDesc, UniformType};

use crate::camera::{occlusion, Camera, Mode, CAM_MIN_DIST};
use crate::models::{Entry, Library};
use crate::scenery::Scenery;
use crate::secondary::{Rig, Secondary};
use crate::stage::{Light, Phase, StageRun};
use crate::tuning::{
    Tuning, INTERNAL_H, INTERNAL_W, PLAYER_HEIGHT, PLAYER_RADIUS, TICK_DT, UI_REF_H,
};
use crate::world::{Aabb, Atk, EState, Enemy, PState, TeleShape, World};

pub struct Renderer {
    rt: RenderTarget,
    /// 렌더 타깃 크기와 정수 배율(물리 px) — 화면에 맞춰 바뀐다(0046).
    rt_size: (u32, u32, u32),
    /// 시나리오 상자마다 지금 진하기(1 = 불투명). 가리면 부드럽게 옅어지고 풀리면 돌아온다(0008).
    fade: Vec<f32>,
    pub lib: Library,
    /// 굽힌 스테이지(어느 레벨인지 = 지도 글의 주소).
    scenery: Option<(usize, Scenery)>,
    pose: Vec<Mat4>,
    batch: Batch,
    /// 방랑자의 2차 동작 층(0039): 상태·부위(처음 그릴 때 모델에서 찾음)·이번 프레임 덧붙이기.
    sec: Secondary,
    sec_rig: Option<Rig>,
    sec_extra: Vec<(usize, Quat, Vec3)>,
    /// 락온 중 과녁을 가릴 때의 진하기(0040, 부드럽게 따라간다).
    lock_fade: f32,
    /// 캐릭터 실시간 그림자(0045): 달 쪽 정사영으로 찍은 가림판(저해상도, 가장 가까운 픽셀) + 그 재질 둘.
    shadow_rt: Option<RenderTarget>,
    mask_mat: Option<Material>,
    floor_mat: Option<Material>,
    shadow_tried: bool,
    /// 이번 프레임 가림판의 빛 공간 행렬(없으면 바닥은 그림자 없이).
    light_vp: Option<Mat4>,
    /// 가림판 가운데(지난 프레임의 플레이어 자리).
    shadow_center: Vec3,
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
    /// UI 배율(글·HUD): 그린 화면 높이(논리 px) / `UI_REF_H`(0046 — 전에는 렌더 정수 배율 × INTERNAL_H / `UI_REF_H`).
    /// 480×270 기준으로 짠 UI가 화면에서 같은 크기로 보인다(0041).
    pub ui: f32,
}

/// 한 프레임에 그릴 것.
pub struct View<'a> {
    pub world: &'a World,
    pub cam: &'a Camera,
    pub alpha: f32,
    pub t: &'a Tuning,
    pub stage: Option<&'a StageRun>,
}

/// 탄의 색(0048·0051, `weapons::Bolt::look`): 마법 라벤더, 무거운 탄 놋쇠, 꿰뚫는 탄 민트, 쇠뇌 볼트 밝은 나무 — 붉은색(위험)은 쓰지 않는다.
const BOLT_LOOKS: [(Color, Color); 4] = [
    (
        Color::new(0.92, 0.88, 1.0, 1.0),
        Color::new(0.7, 0.6, 1.0, 0.45),
    ),
    (
        Color::new(1.0, 0.86, 0.55, 1.0),
        Color::new(0.9, 0.65, 0.3, 0.45),
    ),
    (
        Color::new(0.85, 1.0, 0.92, 1.0),
        Color::new(0.45, 0.9, 0.7, 0.45),
    ),
    (
        Color::new(0.95, 0.85, 0.65, 1.0),
        Color::new(0.8, 0.65, 0.4, 0.3),
    ),
];

/// 예고 색(붉은 계열 하나 — 붉은색은 위험에만, 그림 팔레트에는 없다).
const TELE: Color = Color::new(0.85, 0.15, 0.1, 1.0);

fn with_a(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a)
}

/// 빛을 어떻게 굽나(0033) — 렌더가 프레임마다 정한다(지형 굽기도 같은 값을 본다).
#[derive(Clone, Copy, Debug, Default)]
pub struct Look {
    /// 빛을 단계로 끊는다(툰): 달빛 2단, 화로 3단.
    pub toon: bool,
    /// 배우 가장자리 빛 세기(카메라에서 비스듬한 면일수록, 차가운 달빛 색).
    pub rim: f32,
    /// 배우 외곽선 두께(내부 해상도 px). 0 = 끔.
    pub outline_px: f32,
    /// 카메라 자리와 세로 시야각(라디안) — 가장자리 빛·외곽선 두께.
    pub eye: Vec3,
    pub fovy: f32,
    /// 렌더 타깃의 줄 수(0046 — 화면에 따라 바뀐다, 외곽선 두께를 내부 px로 셀 때).
    pub rows: f32,
}

thread_local! {
    static LOOK: std::cell::Cell<Look> = const {
        std::cell::Cell::new(Look {
            toon: false,
            rim: 0.0,
            outline_px: 0.0,
            eye: Vec3::ZERO,
            fovy: 1.0,
            rows: INTERNAL_H as f32,
        })
    };
}

thread_local! {
    /// 지형 굽기 동안만 있는 달빛 가림 높이 지도(0044, `Scenery::bake`가 넣고 뺀다).
    static BAKE_SHADE: std::cell::RefCell<Option<crate::shade::Occluders>> = const { std::cell::RefCell::new(None) };
}

/// 굽기 동안 달빛 그림자를 켠다(None = 끔).
pub fn set_bake_shade(o: Option<crate::shade::Occluders>) {
    BAKE_SHADE.with(|s| *s.borrow_mut() = o);
}

pub fn look() -> Look {
    LOOK.with(|l| l.get())
}

pub fn set_look(l: Look) {
    LOOK.with(|c| c.set(l));
}

/// `x`(0..1)를 `n`단으로(반올림).
fn band(x: f32, n: f32) -> f32 {
    (x * n).round() / n
}

/// 빛(정점 색에 곱할 rgb): 바탕 + 달빛(고정 방향) + 가까운 화로(거리 제곱으로 준다, 면이 그쪽을 볼수록 밝다).
/// 툰(0033)이면 달빛은 2단, 화로 세기는 3단으로 끊는다 — 둥근 그러데이션 대신 또렷한 빛 웅덩이.
pub fn lit(p: Vec3, n: Vec3, lights: &[Light]) -> Vec3 {
    let toon = look().toon;
    let l = LIGHT_DIR.normalize();
    let mut d = n.dot(l).max(0.0);
    // 지형을 굽는 동안은 달빛 그림자(0044): 막히면 달빛 몫만 뺀다(화로 빛은 그대로).
    if d > 0.0 {
        d *= BAKE_SHADE.with(|s| s.borrow().as_ref().map_or(1.0, |o| o.vis(p + n * 0.05, l)));
    }
    if toon {
        d = band(d, 2.0);
    }
    let mut c = AMBIENT + MOON * d;
    for li in lights {
        let dv = li.pos - p;
        let dist = dv.length();
        if dist >= li.radius {
            continue;
        }
        let mut att = (1.0 - dist / li.radius).powi(2);
        if toon {
            att = band(att, 3.0);
        }
        let facing = 0.35 + 0.65 * n.dot(dv / dist.max(1e-3)).max(0.0);
        c += vec3(li.color.r, li.color.g, li.color.b) * (FIRE * att * facing);
    }
    c
}

/// 가장자리 빛(0033): 카메라에서 비스듬히 돌아선 면일수록(`1 - n·v`의 제곱) 차가운 빛을 더한다 — 어두운 바닥 위 실루엣.
fn rim(p: Vec3, n: Vec3, k: Look) -> Vec3 {
    if k.rim <= 0.0 {
        return Vec3::ZERO;
    }
    let v = (k.eye - p).normalize_or_zero();
    let e = 1.0 - n.dot(v).max(0.0);
    RIM * (k.rim * e * e)
}

const RIM: Vec3 = Vec3::new(0.55, 0.62, 0.85);
/// 외곽선 색(0037: 검정 대신 짙은 자두빛 — 부드러운 외곽선).
const OUTLINE: Color = Color::new(0.13, 0.07, 0.16, 1.0);
/// 외곽선 껍데기를 카메라에서 미는 거리(m) — 이보다 가까이 겹친 부위 사이에는 선을 긋지 않는다.
const OUTLINE_PUSH: f32 = 0.18;

// 0037: 해 질 녘 — 바탕빛은 보랏빛을 조금, 달빛은 차갑게(파스텔 주인공이 그 위에서 선다).
const AMBIENT: Vec3 = Vec3::new(0.29, 0.28, 0.40);
const MOON: Vec3 = Vec3::new(0.44, 0.45, 0.62);
const FIRE: f32 = 1.5;
/// 움직이는 모델에 곱하는 밝기(배경과 떼어 읽히게).
const ACTOR_LIFT: f32 = 1.35;

/// 깊이 버퍼가 붙은 렌더 타깃(0033).
fn make_rt(w: u32, h: u32) -> RenderTarget {
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

/// 화면(물리 px)에 맞춘 내부 해상도와 정수 배율(0046): 배율은 높이가 `INTERNAL_H`에 가장 가깝게(반올림) 고르고,
/// 내부 해상도는 그 배율로 화면을 채우는 크기 — 픽셀은 늘 정수배로 고르고 검은 테두리는 배율보다 작다.
/// 폭은 높이의 4:3~21:9 사이로 묶는다(세로 폰·아주 넓은 창은 나머지가 테두리). 세로 시야각은 그대로라 넓은 화면은 옆을 더 본다.
pub fn fit_internal(sw: f32, sh: f32) -> (u32, u32, u32) {
    let by_h = (sh / INTERNAL_H as f32).round();
    let by_w = (sw / (INTERNAL_H as f32 * 4.0 / 3.0)).floor();
    let s = by_h.min(by_w).max(1.0) as u32;
    let mut w = ((sw / s as f32).floor() as u32).max(1);
    let mut h = ((sh / s as f32).floor() as u32).max(1);
    w = w.min(h * 21 / 9).max(1);
    h = h.min(w * 3 / 4).max(1);
    (w, h, s)
}

impl Renderer {
    pub fn new() -> Self {
        let rt = make_rt(INTERNAL_W, INTERNAL_H);
        Renderer {
            rt,
            rt_size: (INTERNAL_W, INTERNAL_H, 1),
            fade: Vec::new(),
            lib: Library::default(),
            scenery: None,
            pose: Vec::new(),
            batch: Batch::default(),
            sec: Secondary::default(),
            sec_rig: None,
            sec_extra: Vec::new(),
            lock_fade: 1.0,
            shadow_rt: None,
            mask_mat: None,
            floor_mat: None,
            shadow_tried: false,
            light_vp: None,
            shadow_center: Vec3::ZERO,
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
        // 캐릭터 그림자 가림판(0045): 지난 프레임에 모은 배우 실루엣을 달 쪽에서 찍는다(한 프레임 늦음 — 16ms는 안 보인다).
        self.light_vp = None;
        if t[crate::tuning::P::RenderActorShadow] >= 0.5 && v.stage.is_some() {
            self.shadow_pass();
        }
        self.batch.casters.clear();
        self.batch.caster_idx.clear();
        let (iw, ih) = self.fit_screen();
        let aspect = iw as f32 / ih as f32;
        let c3 = cam.to_camera3d(self.rt.clone(), aspect);
        set_look(Look {
            toon: t[crate::tuning::P::RenderToon] >= 0.5,
            rim: t[crate::tuning::P::RenderRim],
            outline_px: t[crate::tuning::P::RenderOutline],
            eye: c3.position,
            fovy: c3.fovy,
            rows: ih as f32,
        });
        set_camera(&c3);
        clear_background(Color::new(0.07, 0.07, 0.12, 1.0));

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
                // 툰을 켜고 끄면 지형을 다시 굽는다(빛이 정점에 구워져 있다, 0033).
                let key = run.def.map.as_ptr() as usize + look().toon as usize;
                if self.scenery.as_ref().is_none_or(|(k, _)| *k != key) {
                    self.scenery = Some((key, Scenery::bake(&run.level, &mut self.lib)));
                }
                let sc = &mut self.scenery.as_mut().unwrap().1;
                draw_plane(
                    Vec3::Y * sc.void_y,
                    vec2(sc.extent, sc.extent),
                    None,
                    Color::new(0.06, 0.06, 0.10, 1.0),
                );
                let boxes: Vec<Aabb> = sc.pieces.iter().filter_map(|p| p.aabb).collect();
                let hidden = crate::camera::occluders(&pts, cam.rig.pos, &boxes);
                // 빛 지도 바닥(0044)은 캐릭터 그림자 가림판을 읽는 재질로(0045). 가림판이 없으면 기본 재질 그대로.
                let floor = match (&self.floor_mat, &self.shadow_rt, self.light_vp) {
                    (Some(m), Some(rt), Some(vp)) => {
                        m.set_uniform("LightVP", vp);
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
        // 그림자는 땅에, 높이 뜰수록 작고 옅게 — 쿼터뷰에서 높이를 읽는 단서(0027).
        let lift = (w.player.air / JUMP_SHADOW_H).clamp(0.0, 1.0);
        shadow_k(
            pp,
            PLAYER_RADIUS * 1.3 * (1.0 - 0.45 * lift),
            1.0 - 0.5 * lift,
        );

        // 스테이지 물건: 문·핵.
        if let Some(run) = v.stage {
            self.stage_props(run, w, &lights);
        }

        // 적·플레이어는 그림자 가림판의 실루엣도 남긴다(0045 — 다음 프레임 가림판).
        self.batch.casting = true;
        self.shadow_center = w.player.pos;
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
        self.batch.casting = false;
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
                zoom: vec2(2.0 / iw as f32, 2.0 / ih as f32),
                target: vec2(iw as f32 * 0.5, ih as f32 * 0.5),
                render_target: Some(self.rt.clone()),
                ..Default::default()
            });
            draw_haze(iw as f32, ih as f32, haze);
        }
        self.present()
    }

    /// 렌더 타깃을 지금 화면에 맞춘다(0046) — 크기가 바뀐 프레임에만 새로 만든다. 돌려주는 것은 내부 해상도.
    fn fit_screen(&mut self) -> (u32, u32) {
        // 정수 배율은 물리 픽셀로 센다: 폰(DPR 3)의 논리 844×390에서는 논리로 세면 1배라 화면 반이 검정이었다(0006).
        let dpi = screen_dpi_scale().max(0.01);
        let fit = fit_internal(screen_width() * dpi, screen_height() * dpi);
        if fit != self.rt_size {
            if (fit.0, fit.1) != (self.rt_size.0, self.rt_size.1) {
                self.rt = make_rt(fit.0, fit.1);
            }
            self.rt_size = fit;
        }
        (fit.0, fit.1)
    }

    /// 렌더 타깃을 화면 가운데에 정수 배율로.
    fn present(&self) -> Frame {
        set_default_camera();
        clear_background(BLACK);
        let (sw, sh) = (screen_width(), screen_height());
        let dpi = screen_dpi_scale().max(0.01);
        let (iw, ih, s) = self.rt_size;
        let scale = s as f32 / dpi;
        let (dw, dh) = (iw as f32 * scale, ih as f32 * scale);
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
            // 그린 화면 높이에 비례(0046): 화면에 맞춘 내부 해상도는 높이가 360이 아닐 수 있다(브라우저 창 323줄 등).
            ui: dh / UI_REF_H,
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
        // 쉼터(0032): 발밑 테 — 아직 안 쉰 곳은 옅게, 다시 일어설 곳은 금빛으로 숨 쉬듯. 쉬는 순간 빛이 퍼진다.
        for (i, r) in run.rests.iter().enumerate() {
            let home = run.rest_at.is_some_and(|(j, _)| j == i);
            let col = if home {
                Color::new(1.0, 0.8, 0.4, 0.55 + 0.25 * (w.time * 2.0).sin())
            } else {
                Color::new(0.9, 0.7, 0.45, 0.25)
            };
            ring(*r, 1.0, 0.0, 360.0, col);
            if home && run.since_rest < 0.8 {
                let k = run.since_rest / 0.8;
                ring(
                    *r + Vec3::Y * (0.6 * k),
                    1.0 + 2.5 * k,
                    0.0,
                    360.0,
                    Color::new(1.0, 0.85, 0.5, 0.9 * (1.0 - k)),
                );
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
            let crouch = matches!(en.state, EState::Tele { .. });
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
        // 락온 중 몸이 과녁을 가리면 옅게(0040): 카메라를 어깨 너머로 비켜 둬도 가까운 큰 대상은 다 비켜나지 못한다.
        let hide = match w.lock.and_then(|i| w.enemies.get(i)).filter(|e| e.alive()) {
            Some(e) if cam.mode == Mode::Lock => occlusion(
                cam.rig.pos,
                pp + Vec3::Y * (pl.air + PLAYER_HEIGHT * 0.45),
                PLAYER_RADIUS * 1.2,
                e.pos + Vec3::Y * (e.height * 0.5),
                e.radius.max(e.height * 0.4),
            ),
            _ => 0.0,
        };
        let want =
            1.0 - (1.0 - t[crate::tuning::P::RenderLockFade]) * (hide / 0.35).clamp(0.0, 1.0);
        self.lock_fade += (want - self.lock_fade) * (1.0 - (-10.0 * get_frame_time()).exp());
        let a = (0.25 + 0.75 * close) * self.lock_fade;
        if let Some(e) = self.lib.get("wanderer") {
            let speed = (pl.pos - pl.prev).length() / TICK_DT;
            let wpn = pl.weapon.def();
            let hv = wpn.heavy;
            let combo_anim = |hit: u8| wpn.combo.get(hit as usize).map_or("attack", |h| h.anim);
            let owned: [&str; 3];
            let (names, at): (&[&str], f32) = match pl.state {
                // 콤보(0051): 무기 표의 타마다 동작(없는 모델이면 평타).
                PState::Attack { t: e, hit, .. } => {
                    owned = [combo_anim(hit), "attack", "attack"];
                    (&owned, e)
                }
                // 강공격(0030): 모으는 동안 칼을 뒤로 끌어 든 자세(없는 모델이면 공격 첫 자세), 떼면 크게 내려친다 — 무기마다(0051).
                PState::Charging { t: e } => {
                    owned = [hv.map_or("charge", |h| h.anim_charge), "charge", "attack"];
                    (&owned, e)
                }
                PState::Heavy { t: e, .. } => {
                    owned = [hv.map_or("heavy", |h| h.anim_heavy), "heavy", "attack"];
                    (&owned, e)
                }
                // 수류탄(0051): 오른손으로 아래에서 위로 던진다.
                PState::Throw { t: e, .. } => (&["throw", "cast", "attack"], e),
                // 등잔(0031): 왼손으로 들어 입에 대고(들기 = `flask.raise s`) 내린다.
                PState::Drink { t: e, .. } => (&["drink", "idle"], e),
                PState::Dodge { t: e, .. } => (&["dodge", "walk"], e),
                PState::Hurt { t: e, .. } => (&["hurt", "idle"], e),
                // 0034: 도는 쪽으로 윗몸이 먼저 감긴다(오른쪽 = 요가 준다).
                PState::Turn { t: e, from, to } => (
                    if crate::world::turn_is_right(from, to) {
                        &["turn_r", "turn", "idle"]
                    } else {
                        &["turn_l", "turn", "idle"]
                    },
                    e,
                ),
                PState::Parry { t: e } => (&["parry", "attack"], e),
                // 마법(0048): 손을 앞으로 내미는 자세(전용 동작이 생기기 전엔 패링 자세).
                PState::Cast { t: e, .. } if matches!(wpn.rb, crate::weapons::Rb::Shot { .. }) => {
                    (&["xbow_shoot", "cast", "attack"], e)
                }
                PState::Cast { t: e, .. } => (&["cast", "parry", "attack"], e),
                // 공중(0027): 방랑자의 jump·plunge·land 동작(없는 모델이면 회피·공격 자세로).
                PState::Jump { plunge: false, .. } => (&["jump", "dodge", "idle"], 0.1),
                // 내려찍기: 떨어지는 동안 칼을 들고 있다가 땅에 가까우면 꽂는 자세로.
                PState::Jump { plunge: true, .. } => (
                    &["plunge", "attack"],
                    if pl.air < 0.35 { 0.29 } else { 0.12 },
                ),
                PState::Land { t: e, .. } => (&["land", "idle"], e),
                PState::Idle if speed > 0.5 => (
                    move_anim(pl.pos - pl.prev, pl.facing_dir(), w.lock.is_some()),
                    w.time * (speed / 4.0).clamp(0.6, 1.6),
                ),
                PState::Idle => (&["idle"], w.time),
            };
            let anim = e.pick(names);
            let looped = matches!(pl.state, PState::Idle);
            let dur = e.duration(anim);
            let at = if looped { at } else { at.min(dur * 0.999) };
            // 키프레임 동작 위에 2차 동작(0039): 꼬리·머리가 늦게 따라오고 몸은 가속하는 쪽으로, 점프·착지에 늘어나고 찌그러진다.
            let rig = *self.sec_rig.get_or_insert_with(|| Rig::of(&e.model));
            self.sec.update(
                &rig,
                get_frame_time(),
                pp,
                pl.facing,
                pl.air,
                pl.vy,
                t,
                &mut self.sec_extra,
            );
            // 든 무기만 보인다(0051): 다른 무기 부위는 크기 0.
            for w2 in crate::weapons::WEAPONS.iter() {
                if w2.id != pl.weapon {
                    if let Some(n) = e.model.node(w2.node) {
                        self.sec_extra.push((n, Quat::IDENTITY, Vec3::ZERO));
                    }
                }
            }
            e.model.pose_with(anim, at, &self.sec_extra, &mut self.pose);
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
            // 강공격이 다 모이면 몸이 잘게 떤다(떼라는 신호, 0034).
            let tremble = match pl.state {
                PState::Charging { t: e } if e >= t[crate::tuning::P::HeavyCharge] => {
                    vec3((w.time * 61.0).sin(), 0.0, (w.time * 47.0).cos()) * 0.018
                }
                _ => Vec3::ZERO,
            };
            let inst = Mat4::from_translation(pp + Vec3::Y * pl.air + tremble)
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
                PState::Charging { .. } => Color::new(1.0, 0.7, 0.35, 1.0),
                PState::Heavy { .. } => Color::new(1.0, 0.6, 0.3, 1.0),
                PState::Drink { .. } => Color::new(0.75, 0.9, 0.6, 1.0),
                PState::Turn { .. } => Color::new(0.7, 0.95, 0.75, 1.0),
                PState::Parry { .. } => Color::new(0.7, 0.85, 1.0, 1.0),
                PState::Cast { .. } | PState::Throw { .. } => Color::new(0.75, 0.7, 1.0, 1.0),
                PState::Jump { .. } | PState::Land { .. } => Color::new(0.9, 0.8, 0.6, 1.0),
                PState::Idle => Color::new(0.85, 0.75, 0.55, 1.0),
            };
            actor(
                pp + Vec3::Y * pl.air,
                pl.facing_dir(),
                PLAYER_RADIUS,
                PLAYER_HEIGHT,
                with_a(pcol, pcol.a * a),
                t[crate::tuning::P::Shade],
                Quat::IDENTITY,
            );
        }
    }

    /// 배우 위에 얹는 것: 스윙 부채꼴·쟁반 테·휘청 별·락온 표시·마법 탄.
    fn overlays(&self, w: &World, pp: Vec3, alpha: f32, t: &Tuning) {
        // 마법 탄(0048): 라벤더 빛 덩이 + 짧은 꼬리(보간 자리).
        for b in &w.bolts {
            let p = b.prev.lerp(b.pos, alpha);
            let (core, glow) = BOLT_LOOKS[(b.def.look as usize).min(BOLT_LOOKS.len() - 1)];
            let k = if b.def.look == 3 { 0.6 } else { 1.0 };
            draw_cube(p, Vec3::splat(0.32 * k), None, core);
            draw_cube(p, Vec3::splat(0.5 * k), None, glow);
            draw_line_3d(p - b.dir * 0.9, p, glow);
        }
        // 수류탄(0051): 어두운 덩이 + 도화선이 짧아질수록 빨리 깜빡이는 불.
        for bm in &w.bombs {
            let p = bm.prev.lerp(bm.pos, alpha);
            draw_cube(p, Vec3::splat(0.3), None, Color::new(0.25, 0.2, 0.18, 1.0));
            let hz = 4.0 + 10.0 * (1.0 - (bm.fuse / 1.5).clamp(0.0, 1.0));
            if (w.time * hz) as i32 % 2 == 0 {
                draw_cube(
                    p + Vec3::Y * 0.2,
                    Vec3::splat(0.14),
                    None,
                    Color::new(1.0, 0.75, 0.3, 1.0),
                );
            }
        }
        // 수류탄 겨누기(0055): 떨굴 자리에 폭발 반지름 고리.
        if let PState::Throw {
            to,
            thrown: false,
            t: e,
        } = w.player.state
        {
            if let Some(g) = w.player.weapon.def().grenade {
                if e >= t[crate::tuning::P::ThrowStartup] * 0.99 {
                    let y = w.ground.as_ref().map_or(0.0, |gr| gr.height(to));
                    let c = vec3(to.x, y + 0.08, to.z);
                    ring(c, g.radius, 0.0, 360.0, Color::new(1.0, 0.82, 0.45, 0.8));
                    draw_line_3d(pp + Vec3::Y * 1.0, c, Color::new(1.0, 0.82, 0.45, 0.35));
                }
            }
        }
        // 금 간 땅(망치 여진): 터질 때가 가까울수록 진해지는 고리.
        for c in &w.cracks {
            let k = (1.0 - c.left / 1.5).clamp(0.0, 1.0);
            ring(
                c.pos + Vec3::Y * 0.06,
                c.radius,
                0.0,
                360.0,
                Color::new(1.0, 0.75, 0.4, 0.2 + 0.5 * k),
            );
            ring(
                c.pos + Vec3::Y * 0.06,
                c.radius * k,
                0.0,
                360.0,
                Color::new(1.0, 0.85, 0.5, 0.35),
            );
        }
        // 폭발 고리: 퍼지며 옅어진다.
        for b in &w.blasts {
            let k = (b.age / crate::tuning::BLAST_SHOW).clamp(0.0, 1.0);
            ring(
                b.pos + Vec3::Y * 0.1,
                b.radius * (0.4 + 0.6 * k),
                0.0,
                360.0,
                Color::new(1.0, 0.8, 0.45, 0.9 * (1.0 - k)),
            );
        }
        let wpn = w.player.weapon.def();
        if let PState::Attack { t: e, hit, .. } = w.player.state {
            let (s0, a0, _) = crate::world::hit_times(t, wpn, hit);
            if let Some(h) = wpn.combo.get(hit as usize) {
                if e >= s0 && e < s0 + a0 {
                    let arc = if h.shape == crate::weapons::Shape::Circle {
                        360.0
                    } else {
                        t[crate::tuning::P::AttackArc] * h.arc
                    };
                    sector(
                        pp + Vec3::Y * 0.9,
                        t[crate::tuning::P::AttackRange] * h.reach,
                        w.player.facing,
                        arc,
                        Color::new(1.0, 0.95, 0.7, 0.35),
                    );
                }
            }
        }
        // 강공격(0030): 모으는 동안 발밑 고리가 안쪽에서 차오르고, 다 차면 금빛으로 깜빡인다. 강타의 활성 구간엔 넓은 부채꼴.
        if let PState::Charging { t: e } = w.player.state {
            let full =
                (t[crate::tuning::P::HeavyCharge] * wpn.heavy.map_or(1.0, |h| h.charge)).max(1e-3);
            let k = (e / full).clamp(0.0, 1.0);
            let col = if k >= 1.0 {
                let on = (w.time * 12.0) as i32 % 2 == 0;
                Color::new(1.0, 0.85, 0.35, if on { 0.95 } else { 0.55 })
            } else {
                Color::new(1.0, 0.7, 0.4, 0.35 + 0.4 * k)
            };
            ring(
                pp + Vec3::Y * 0.05,
                0.9,
                0.0,
                360.0,
                with_a(col, col.a * 0.5),
            );
            ring(pp + Vec3::Y * 0.05, 0.2 + 0.7 * k, 0.0, 360.0, col);
        }
        if let (PState::Heavy { t: e, full, .. }, Some(h)) = (w.player.state, wpn.heavy) {
            let (s0, a0, _) = crate::world::heavy_times(t, wpn);
            let (s1, arc) = (
                s0 + a0,
                if h.shape == crate::weapons::Shape::Circle {
                    360.0
                } else {
                    t[crate::tuning::P::HeavyArc] * h.arc
                },
            );
            if e >= s0 && e < s1 {
                sector(
                    pp + Vec3::Y * 0.9,
                    t[crate::tuning::P::HeavyRange] * h.reach,
                    w.player.facing,
                    arc,
                    if full {
                        Color::new(1.0, 0.8, 0.4, 0.45)
                    } else {
                        Color::new(1.0, 0.9, 0.65, 0.35)
                    },
                );
            }
        }
        // 등잔(0031): 삼키는 순간 따뜻한 빛이 발밑에서 퍼진다.
        if let PState::Drink { t: e, gulped: true } = w.player.state {
            let k = ((e - t[crate::tuning::P::FlaskRaise]) / 0.35).clamp(0.0, 1.0);
            if k < 1.0 {
                ring(
                    pp + Vec3::Y * (0.05 + 0.8 * k),
                    0.5 + 0.6 * k,
                    0.0,
                    360.0,
                    Color::new(1.0, 0.82, 0.45, 0.8 * (1.0 - k)),
                );
            }
        }
        // 패링(0025): 창이 열린 동안 앞에 옅은 푸른 반달, 성공하면 둘레로 퍼지는 흰 고리.
        if let PState::Parry { t: e } = w.player.state {
            if e < t[crate::tuning::P::ParryWindow] {
                sector(
                    pp + Vec3::Y * 0.8,
                    1.4,
                    w.player.facing,
                    150.0,
                    Color::new(0.6, 0.85, 1.0, 0.35),
                );
                ring(
                    pp + Vec3::Y * 0.8,
                    1.4,
                    w.player.facing,
                    150.0,
                    Color::new(0.75, 0.9, 1.0, 0.8),
                );
            }
        }
        // 내려찍기(0027): 내려앉은 자리에서 범위만큼 퍼지는 고리(`plunge.range m`).
        if let PState::Land { t: e, dur } = w.player.state {
            if (dur - t[crate::tuning::P::PlungeLand]).abs() < 1e-4 && e < 0.25 {
                let k = e / 0.25;
                let r = t[crate::tuning::P::PlungeRange] * (0.4 + 0.6 * k);
                ring(
                    pp,
                    r,
                    0.0,
                    360.0,
                    Color::new(1.0, 0.85, 0.55, 0.9 * (1.0 - k)),
                );
            }
        }
        if w.player.since_parry < 0.35 {
            let k = (w.player.since_parry / 0.35).clamp(0.0, 1.0);
            ring(
                pp + Vec3::Y * 0.05,
                0.6 + 2.4 * k,
                0.0,
                360.0,
                Color::new(0.85, 0.95, 1.0, 0.9 * (1.0 - k)),
            );
        }
        // 드러난 적(치명타 기회): 발밑에 금빛 고리가 깜빡인다.
        for en in &w.enemies {
            if en.exposed > 0.0 && en.alive() && (w.time * 10.0) as i32 % 2 == 0 {
                let p = en.prev.lerp(en.pos, alpha);
                ring(
                    p,
                    en.radius + 0.45,
                    0.0,
                    360.0,
                    Color::new(1.0, 0.82, 0.35, 0.85),
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
        EState::Stagger { .. } => once(&["hurt", "idle"], 0.3),
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
            // 패링으로 못 받는다 = 가운데 X(색이 아니라 모양으로, nail 0187 · KB timing-parry).
            let x = r.min(2.0) * 0.45;
            let y = Vec3::Y * 0.04;
            line(p + y + vec3(-x, 0.0, -x), p + y + vec3(x, 0.0, x), edge);
            line(p + y + vec3(-x, 0.0, x), p + y + vec3(x, 0.0, -x), edge);
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

/// 모델·굽힌 덩어리를 한 배치로 모은다(P4·P5): 부위 행렬 × 정점을 CPU에서 곱하고 빛(`lit`)을 정점 색에 굽는다.
/// macroquad는 그리기 호출 하나에 정점 10,000·색인 5,000 **미만**만 받고 넘치면 잘라 버린다
/// (0.4.16 `quad_gl.rs` `geometry`, `Conf::draw_call_*_capacity` 기본값 — 처음엔 40마리를 한 덩어리로 보내 아무것도 안 그려졌다).
/// 그래서 그 아래에서 끊어 보낸다. 텍스처가 바뀌어도 끊는다.
pub const MAX_DRAW_VERTICES: usize = 10_000;
/// 창 설정의 그리기 인덱스 용량(`main.rs window_conf`, 0036 — 기본 5,000에서 올림).
pub const DRAW_CALL_INDICES: usize = 15_000;
pub const MAX_DRAW_INDICES: usize = DRAW_CALL_INDICES;

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
        Self::bake_glow(verts, idx, e, pose, inst, tint, lights, 1.0, false);
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
            for k in 0..mesh.pos.len() {
                let p = mat.transform_point3(mesh.pos[k]);
                let f = if bright {
                    Vec3::splat(glow)
                } else {
                    let n = mat.transform_vector3(mesh.nrm[k]).normalize_or_zero();
                    let r = if actor { rim(p, n, look) } else { Vec3::ZERO };
                    lit(p, n, lights) + r
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
            true,
        );
        if self.casting && tint.a > 0.5 && self.casters.len() + nv < MAX_DRAW_VERTICES {
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
            Self::hull(
                &mut self.outline,
                &mut self.outline_idx,
                e,
                pose,
                inst,
                w,
                k.eye,
            );
            if self.outline.len() >= MAX_DRAW_VERTICES - 2000 {
                self.send_outline();
            }
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

/// 걷는 동작 고르기(0034): 락온 중엔 몸이 대상을 보니 걷는 쪽이 옆이면 게걸음(`strafe_l`·`strafe_r`), 뒤면 뒷걸음.
/// 락온이 아니면 늘 앞으로 걷는다(몸이 걷는 쪽을 본다). 없는 모델이면 `walk`로.
pub fn move_anim(step: Vec3, facing: Vec3, locked: bool) -> &'static [&'static str] {
    if !locked {
        return &["walk"];
    }
    let v = vec3(step.x, 0.0, step.z);
    let fwd = v.dot(facing);
    // 오른쪽 = 앞 × 위(카메라·입력과 같은 축).
    let side = v.dot(facing.cross(Vec3::Y));
    if side.abs() > fwd.abs() * 1.2 {
        if side > 0.0 {
            &["strafe_r", "walk"]
        } else {
            &["strafe_l", "walk"]
        }
    } else if fwd < 0.0 {
        &["walk_back", "walk"]
    } else {
        &["walk"]
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

/// 그림자 가림판(0045): 한 변 칸 수와 덮는 넓이(m). 0.125m 칸 — 바닥 빛 지도(0.25m)보다 촘촘해 캐릭터 실루엣이 읽힌다.
const SHADOW_RES: u32 = 256;
const SHADOW_SPAN: f32 = 32.0;

impl Renderer {
    /// 지난 프레임의 배우 실루엣을 달 쪽 정사영으로 가림판에 찍는다. 재질을 못 만들면(Metal 등) 끈다 — 바닥은 그림자 없이.
    fn shadow_pass(&mut self) {
        if !self.shadow_tried {
            self.shadow_tried = true;
            self.mask_mat = flat_material(MASK_FRAGMENT, &[], &[]);
            self.floor_mat = flat_material(
                FLOOR_FRAGMENT,
                &[UniformDesc::new("LightVP", UniformType::Mat4)],
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
void main() {
    vec4 c = color * texture2D(Texture, uv);
    float s = 0.0;
    if (sp.x > 0.0 && sp.x < 1.0 && sp.y > 0.0 && sp.y < 1.0) {
        s = texture2D(ShadowMask, sp.xy).r;
    }
    gl_FragColor = vec4(c.rgb * (1.0 - 0.5 * s), c.a);
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

/// 키운 상자에서 버릴 면. 렌더 타깃이 위아래를 뒤집으면 감김이 바뀌어 반대가 된다 — 스크린숏으로 정했다(0033).
const OUTLINE_CULL: miniquad::CullFace = miniquad::CullFace::Front;

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

/// 고정 빛 방향 = 달(0044: 왼쪽 앞 위 45° — 벽·단·기둥의 그림자가 뜰 바닥 오른쪽·뒤로 떨어져 쿼터뷰에서 보이게;
/// 예전 오른쪽 앞에서는 그림자가 벽 바깥으로 떨어졌다). 면마다 `1 - shade + shade * max(0, n·l)`을 곱한다 — 기본 셰이더가 normal을 안 읽어 정점 색으로 굽는다
/// (KB reference/macroquad/3d-rendering.md). shade 0이면 민짜(전처럼), 1이면 그늘진 면이 검정에 가깝다.
const LIGHT_DIR: Vec3 = Vec3::new(-0.55, 0.65, 0.35);

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
    shadow_k(p, r, 1.0);
}

/// 그림자(진하기 `k`배).
fn shadow_k(p: Vec3, r: f32, k: f32) {
    sector(
        p + Vec3::Y * 0.01,
        r,
        0.0,
        360.0,
        Color::new(0.0, 0.0, 0.0, 0.35 * k),
    );
}

/// 이 높이(m)에서 그림자가 가장 작다.
const JUMP_SHADOW_H: f32 = 1.2;

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

    /// 0046: 흔한 화면은 테두리 없이 정수 배율로 꽉 차고, 브라우저 창(주소창만큼 낮은 높이)도 테두리가 배율보다 작다.
    #[test]
    fn the_internal_resolution_fills_the_screen_with_integer_pixels() {
        assert_eq!(fit_internal(1920.0, 1080.0), (640, 360, 3));
        assert_eq!(fit_internal(1280.0, 720.0), (640, 360, 2));
        assert_eq!(fit_internal(3840.0, 2160.0), (640, 360, 6));
        // 0041의 고정 640×360은 이 창에서 ×2(1280×720)라 둘레가 넓게 검었다.
        let (w, h, s) = fit_internal(1920.0, 969.0);
        assert_eq!(s, 3);
        assert!(1920 - w * s < s && 969 - h * s < s, "{w}×{h}×{s}");
        // 20:9 폰·16:10 핸드헬드는 옆·위아래를 더 본다(세로 시야각 그대로).
        assert_eq!(fit_internal(2400.0, 1080.0), (800, 360, 3));
        assert_eq!(fit_internal(1920.0, 1200.0), (640, 400, 3));
        // 세로 폰은 4:3까지만 — 나머지는 테두리.
        let (w, h, _) = fit_internal(1170.0, 2532.0);
        assert!(h * 4 <= w * 3 + 4, "{w}×{h}");
        // 아주 작은 창도 0이 되지 않는다.
        assert!(fit_internal(10.0, 10.0).0 >= 1);
    }

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

#[cfg(test)]
mod move_anim_tests {
    use super::*;

    /// 0034: 락온 중 옆으로 걸으면 게걸음(오른쪽 = 앞 × 위), 뒤로 걸으면 뒷걸음, 락온이 아니면 늘 걷기.
    #[test]
    fn locked_on_sideways_steps_strafe_and_backwards_walks_back() {
        let facing = vec3(0.0, 0.0, -1.0); // 북을 본다 → 오른쪽 = +x
        assert_eq!(move_anim(vec3(1.0, 0.0, 0.0), facing, true)[0], "strafe_r");
        assert_eq!(move_anim(vec3(-1.0, 0.0, 0.1), facing, true)[0], "strafe_l");
        assert_eq!(move_anim(vec3(0.1, 0.0, 1.0), facing, true)[0], "walk_back");
        assert_eq!(move_anim(vec3(0.0, 0.0, -1.0), facing, true)[0], "walk");
        assert_eq!(move_anim(vec3(1.0, 0.0, 0.0), facing, false)[0], "walk");
    }
}
