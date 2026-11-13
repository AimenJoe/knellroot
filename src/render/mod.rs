//! 그리기. 월드를 읽기만 한다. 3D 패스는 내부 해상도 렌더 타깃(PS1 느낌)에, 글·메뉴는 화면 패스(`hud.rs`·`app`).
//!
//! 스테이지(0019): 굽힌 지형(`scenery.rs`) + 모델(`models.rs` 목록, 상태 → 동작 이름) + 빛(달빛 + 화로, 정점 색). 모델이 없는 종류는
//! 상자로. 개발 시나리오(스테이지 없음)는 PoC의 회색 아레나 그대로.
//! 바닥 예고는 모양마다(원판·부채꼴·띠·고리, `TeleShape`) 옅은 면 + 테두리, 채움이 자라 테두리에 닿는 순간 공격(nail 0186~0188).
use crate::progression::Cell;
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

mod actors;
mod batch;
mod descent;
mod drape;
mod items;
mod light;
mod overlays;
mod post;
mod props;
mod shadow;
mod shapes;
mod skill_fx;
mod telegraph;
mod terrain;
#[cfg(test)]
mod tests;

pub use actors::*;
pub use batch::*;
use drape::*;
pub use light::*;
pub use overlays::*;
pub use post::*;
use props::*;
pub use shapes::*;
use telegraph::*;
use terrain::*;

pub struct Renderer {
    rt: RenderTarget,
    /// 선·번짐·톤을 내부 해상도에서 한 번 셈해 담는 타깃(0140) — 화면에는 이것을 가장 가까운 픽셀로 키운다.
    post: RenderTarget,
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
    /// 서 있는 동안의 몸짓(0153)과 그 시계의 기준(월드 시간 — 멈춤 창 아래에서는 같이 선다).
    idle: crate::idle::Idle,
    idle_at: f32,
    sec_rig: Option<Rig>,
    sec_extra: Vec<(usize, Quat, Vec3)>,
    /// 락온 중 과녁을 가릴 때의 진하기(0040, 부드럽게 따라간다).
    lock_fade: f32,
    /// 휘돌이의 전용 동작(`sk_whirl` — 몸의 한 바퀴를 담고 있다)이 모델에 있나(0158 G2 — `player`가 적고 빛이 같이 읽는다).
    whirl_clip: bool,
    /// 스테이지 문의 진하기(0091): 지형 덩어리와 같은 규칙으로 플레이어·락온 대상을 가리면 옅어진다(문은 굽는 지형이 아니라 따로 그린다).
    gate_fade: f32,
    /// 캐릭터 실시간 그림자(0045): 달 쪽 정사영으로 찍은 가림판(저해상도, 가장 가까운 픽셀) + 그 재질 둘.
    shadow_rt: Option<RenderTarget>,
    mask_mat: Option<Material>,
    floor_mat: Option<Material>,
    shadow_tried: bool,
    /// 바람 재질(0142)과 이번 프레임의 바람 값 — 지형 덩어리와 법선 패스가 같이 읽는다.
    wind_mat: Option<Material>,
    wind_tried: bool,
    wind_uniform: Vec4,
    /// 이번 프레임 가림판의 빛 공간 행렬(없으면 바닥은 그림자 없이).
    light_vp: Option<Mat4>,
    /// 가림판 가운데(지난 프레임의 플레이어 자리).
    shadow_center: Vec3,
    /// 화면 공간 선 둘(0061): 법선 패스 타깃·재질과 선 재질. 못 만들면(Metal 등) 끈다.
    edge: Option<EdgeFx>,
    edge_tried: bool,
    /// 이번 프레임에 법선 패스를 그렸나(선 재질을 쓸까).
    edge_ready: bool,
    edge_uniform: Vec4,
    grade_uniform: Vec4,
    /// 주위 환경(0063): 높이 안개 등.
    backdrop: crate::backdrop::Backdrop,
    /// 장비 창 인형(0067 D): 방랑자만 그리는 작은 렌더 타깃(처음 쓸 때 만든다).
    doll_rt: Option<RenderTarget>,
    /// 도감 생명체(0130).
    creature_rt: Option<RenderTarget>,
    /// 아이템 그림(0068): 아이콘은 아이템마다 한 번 찍어 둔 타깃, 고른 것 하나는 도는 큰 타깃.
    icons: std::collections::HashMap<&'static str, RenderTarget>,
    item_rt: Option<RenderTarget>,
    /// 지역 사이 전환의 굴(0174) — 전환마다 한 번 짓는다.
    shaft: Option<descent::ShaftMesh>,
}

/// 아이템 아이콘·도는 그림의 타깃 크기(내부 px, 정사각).
pub const ICON_PX: u32 = 32;
pub const ITEM_PX: u32 = 56;

/// 인형 렌더 타깃 크기(내부 px) — 게임 화면과 같은 도트 굵기로 보이게 작게 굽고 정수배로 늘린다.
pub const DOLL_W: u32 = 64;
pub const DOLL_H: u32 = 96;

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
    /// 바닥 공격 예고 진하기(설정 0..1, 0 = 끔 — 0074).
    pub tells: f32,
    /// 바닥 예고의 "언제"(0128): 켬 = 채움이 테두리에 닿는 순간(0186 꼴), 끔(기본) = 자리가 점점 짙어지다 정해진 농도에서 맞는다.
    pub tell_timing: bool,
    /// 그림 이펙트(0141 — 앱이 사건을 듣고 채운다).
    pub fx: &'a crate::fx::Fx,
    /// 깨어나는 첫 장면(0171)이 흐르는 동안의 `wake` 동작 시각 — 방랑자는 상태와 상관없이 그 자세로 그린다(월드는 모른다).
    pub wake: Option<f32>,
}
/// 화로의 빛은 바닥 1.2m 위(`stage` 지도 읽기), 그릇 테는 0.86m(`props.py`) — 불티가 그릇에서 오르게.
const BRAZIER_LIGHT_ABOVE_BOWL: f32 = 1.2 - 0.86;

impl Renderer {
    pub fn new() -> Self {
        let rt = make_rt(INTERNAL_W, INTERNAL_H);
        Renderer {
            rt,
            post: make_post(INTERNAL_W, INTERNAL_H),
            rt_size: (INTERNAL_W, INTERNAL_H, 1),
            fade: Vec::new(),
            lib: Library::default(),
            scenery: None,
            pose: Vec::new(),
            batch: Batch::default(),
            sec: Secondary::default(),
            idle: crate::idle::Idle::default(),
            idle_at: 0.0,
            sec_rig: None,
            sec_extra: Vec::new(),
            lock_fade: 1.0,
            whirl_clip: false,
            gate_fade: 1.0,
            shadow_rt: None,
            mask_mat: None,
            floor_mat: None,
            shadow_tried: false,
            wind_mat: None,
            wind_tried: false,
            wind_uniform: Vec4::ZERO,
            light_vp: None,
            shadow_center: Vec3::ZERO,
            edge: None,
            edge_tried: false,
            edge_ready: false,
            edge_uniform: Vec4::ZERO,
            grade_uniform: Vec4::ZERO,
            backdrop: Default::default(),
            doll_rt: None,
            creature_rt: None,
            icons: Default::default(),
            item_rt: None,
            shaft: None,
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
        let (iw, ih, c3) = self.begin_frame(v, w, cam, t);
        // 원경(0063 E2): 하늘 띠·달·산등성이 — 스테이지에서만, 땅보다 먼저(법선 패스에는 넣지 않는다).
        let fwd = (c3.target - c3.position).normalize_or_zero();
        let cam_right = fwd.cross(Vec3::Y).normalize_or_zero();
        let cam_up = cam_right.cross(fwd);
        if let Some(run) = v.stage.filter(|_| t[crate::tuning::P::RenderSky] >= 0.5) {
            let base = self.scenery.as_ref().map_or(-2.5, |(_, sc)| sc.void_y);
            self.backdrop.sky(
                &mut self.batch,
                c3.position,
                cam_right,
                cam_up,
                run.level.half_extent(),
                base,
            );
        }

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
            cam.view.pos,
            tg,
        );
        let k = 1.0 - (-10.0 * get_frame_time()).exp();
        // 법선 패스에 다시 그릴 불투명 기하를 모은다(0061) — 지형·소품·배우까지, 옅은 덩어리·효과는 빼고.
        let edges_on = t[crate::tuning::P::RenderEdge] >= 0.5;
        self.batch.record = edges_on;
        self.batch.recorded.clear();
        let lights: Vec<Light> = match v.stage {
            Some(run) => self.terrain(run, w, cam, &pts, k, t),
            None => {
                self.sandbox_floor(w, &pts, cam.view.pos, k, t);
                Vec::new()
            }
        };

        ground_marks(v, w, alpha);
        linger_tells(v, w);
        let pp = w.player.prev.lerp(w.player.pos, alpha);
        // 그림자는 땅에, 높이 뜰수록 작고 옅게 — 쿼터뷰에서 높이를 읽는 단서(0027).
        let lift = (w.player.air / JUMP_SHADOW_H).clamp(0.0, 1.0);
        Drape {
            ground: w.ground.as_ref(),
        }
        .shadow_k(
            pp,
            PLAYER_RADIUS * 1.3 * (1.0 - 0.45 * lift),
            1.0 - 0.5 * lift,
        );

        // 스테이지 물건: 문·핵.
        if let Some(run) = v.stage {
            // 문이 시야를 가리면 옅게(0091, 사장 "문은 투명화되지 않아서 시야를 가림") — 지형 덩어리와 같은 판정점·같은 빠르기.
            let want = match gate_box(run) {
                Some(b) if !crate::camera::occluders(&pts, cam.view.pos, &[b]).is_empty() => FADED,
                _ => 1.0,
            };
            self.gate_fade += (want - self.gate_fade) * k;
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
        self.player(w, cam, pp, t, &lights, v.wake);
        // 던진 무기(0158 G2): 날아가는 자리에 돌며.
        self.blades(w, alpha, &lights);
        self.batch.casting = false;
        self.batch.flush();
        self.batch.record = false;

        self.overlays(w, pp, alpha, t);
        // 스킬이 남긴 빛(0158 G2 — `fx::Mark`)과 이어지는 것들(울림·불티·몰아치기의 기운).
        if t[crate::tuning::P::RenderFx] >= 0.5 {
            self.marks(w, &v.fx.marks, pp);
        }
        self.skill_auras(w, pp, alpha, t);
        // 모아 둔 선(윤곽선·예고 옆선)을 한꺼번에 — 옅은 덩어리보다 먼저(그 뒤의 선이 가려지지 않게).
        flush_lines();

        // 옅어진 덩어리는 맨 나중에: 면은 거의 투명하게.
        match v.stage {
            Some(run) => {
                self.faded_terrain(run, w, t, &lights, cam_right, cam_up);
            }
            None => self.sandbox_faded(w, t),
        }
        self.particles(v, w, t, &lights, cam_right, cam_up);

        self.horizon_haze(t, fwd, iw, ih);
        set_actor_shade(None);
        self.edge_ready = edges_on && self.normal_pass(&c3, (iw, ih));
        self.edge_uniform = vec4(
            t[crate::tuning::P::RenderEdgeDepth],
            t[crate::tuning::P::RenderEdgeDark],
            t[crate::tuning::P::RenderEdgeLight],
            t[crate::tuning::P::RenderEdgeCrease],
        );
        // 마무리 패스(0140): 톤 곡선·빛 번짐·가장자리 어둡게 — 끝 칸은 "선을 긋나"(법선 패스를 그린 프레임만).
        self.grade_uniform = vec4(
            t[crate::tuning::P::RenderGrade],
            t[crate::tuning::P::RenderBloom],
            t[crate::tuning::P::RenderVignette],
            if self.edge_ready { 1.0 } else { 0.0 },
        );
        self.present()
    }

    /// 프레임 머리: 그림자 가림판, 렌더 타깃 맞추기, 카메라·보는 눈(`Look`)·바람·지역 환경. 돌려주는 것은 내부 해상도와 카메라.
    fn begin_frame(
        &mut self,
        v: &View,
        w: &World,
        cam: &Camera,
        t: &Tuning,
    ) -> (u32, u32, Camera3D) {
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
            calm: t[crate::tuning::P::RenderFloorCalm] >= 0.5,
            key: t[crate::tuning::P::RenderKeyLight] >= 0.5,
            enemy_lift: t[crate::tuning::P::RenderEnemyLift],
            enemy_ash: t[crate::tuning::P::RenderEnemyAsh],
        });
        set_camera(&c3);
        // 바람(0142): 시각은 초당 열두 번으로 끊는다(낮은 해상도에서 풀이 기어가지 않고 "도트 움직임"으로), 땅속은 약하게.
        if !self.wind_tried {
            self.wind_tried = true;
            self.wind_mat = wind_material();
        }
        let breeze = t[crate::tuning::P::RenderWind]
            * v.stage
                .map_or(1.0, |r| if r.def.env.moon { 1.0 } else { 0.4 });
        self.wind_uniform = vec4(
            (w.time * 12.0).floor() / 12.0,
            breeze,
            w.player.pos.x,
            w.player.pos.z,
        );
        // 지역 환경(0118 T2): 스테이지면 그 지역의 것, 개발 놀이터는 땅 위.
        self.backdrop.env = v.stage.map_or(&crate::backdrop::SURFACE, |r| r.def.env);
        set_moonlight(self.backdrop.env.moonlight);
        clear_background(self.backdrop.env.zenith);
        (iw, ih, c3)
    }

    fn particles(
        &mut self,
        v: &View,
        w: &World,
        t: &Tuning,
        lights: &[Light],
        cam_right: Vec3,
        cam_up: Vec3,
    ) {
        // 잔불 조각(0064 M1) — 스테이지·개발 시나리오 둘 다.
        self.backdrop
            .embers(&mut self.batch, &w.drops, cam_right, cam_up, w.time);
        // 이펙트 입자(0141): 사건의 입자·화로 불티는 랩 `render.fx`로 끄고 켠다, 맞힌 자리의 불똥은 늘.
        let fx_on = t[crate::tuning::P::RenderFx] >= 0.5;
        let braziers: Vec<Vec3> = lights
            .iter()
            .filter(|l| fx_on && !l.shaft && l.pos.distance(w.player.pos) < 26.0)
            .map(|l| l.pos - Vec3::Y * BRAZIER_LIGHT_ABOVE_BOWL)
            .collect();
        self.backdrop.fx(
            &mut self.batch,
            if fx_on { &v.fx.parts } else { &[] },
            &w.sparks,
            &braziers,
            cam_right,
            cam_up,
            w.time,
        );
    }
}

/// 이 높이(m)에서 그림자가 가장 작다.
const JUMP_SHADOW_H: f32 = 1.2;

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
