//! 카메라 리그 둘(쿼터뷰·락온) + 하나의 상태. 원하는 리그를 매 프레임 계산하고, 모드가 바뀌면
//! "지금 카메라"에서 원하는 리그로 ease-in-out 보간한다(KB concepts/camera-mode-switch-and-lock-on.md).
//! 입력 기준은 여기 없다 — 월드가 `Basis`를 보고 즉시 바꾼다(카메라가 늦게 도착해도 조작은 바로).
use macroquad::prelude::*;

use crate::tuning::{Tuning, P, PLAYER_HEIGHT};

/// 쿼터뷰에서 화면 위가 가리키는 월드 방향(XZ 단위 벡터). 입력 기준(`world::Basis::Quarter`)과 카메라가 같은 식을 쓴다.
pub fn quarter_forward(yaw_deg: f32) -> Vec3 {
    let (s, c) = yaw_deg.to_radians().sin_cos();
    vec3(-s, 0.0, -c)
}

/// 당겨도 머리에서 이보다 가까이는 안 온다(몸 안에 들어가면 화면이 몸 색으로 찬다). 이보다 가까우면 렌더가 플레이어를 옅게 그린다.
pub const CAM_MIN_DIST: f32 = 1.2;
use crate::world::Aabb;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Mode {
    Quarter,
    /// 락온 대상의 월드 위치는 월드가 준다(여기는 "어떻게 볼까"만).
    Lock,
}

/// 카메라의 세 값. 셋을 같은 곡선으로 보간한다(피치·거리·FOV를 따로 바꾸지 않기 — Nesky 16).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rig {
    pub pos: Vec3,
    pub look: Vec3,
    /// 라디안.
    pub fov: f32,
}

impl Rig {
    fn lerp(a: Rig, b: Rig, t: f32) -> Rig {
        Rig {
            pos: a.pos.lerp(b.pos, t),
            look: a.look.lerp(b.look, t),
            fov: a.fov + (b.fov - a.fov) * t,
        }
    }
}

pub struct Camera {
    pub mode: Mode,
    pub rig: Rig,
    /// 전환 중이면 (출발 리그, 경과 시간).
    blend: Option<(Rig, f32)>,
    /// 쿼터뷰 추적점(플레이어를 지연 추적).
    follow: Vec3,
    /// 락온 요(라디안, 대상 방향을 지연 추적).
    yaw: f32,
    /// 화면 흔들림(trauma 0..1, 제곱으로 쓴다).
    pub trauma: f32,
    shake: Vec3,
}

impl Camera {
    pub fn new(player: Vec3, t: &Tuning) -> Self {
        let mut c = Camera {
            mode: Mode::Quarter,
            rig: Rig {
                pos: Vec3::ZERO,
                look: Vec3::ZERO,
                fov: 1.0,
            },
            blend: None,
            follow: player,
            yaw: 0.0,
            trauma: 0.0,
            shake: Vec3::ZERO,
        };
        c.rig = c.desired(t, player, None);
        c
    }

    /// 모드 전환. 같은 모드면 아무것도 안 한다. 출발점은 지금 카메라(고정 자리에서 시작하면 달리는 중 튄다).
    pub fn set_mode(&mut self, mode: Mode, target: Option<Vec3>, player: Vec3) {
        if mode == self.mode {
            return;
        }
        if mode == Mode::Lock {
            // 요를 대상 방향에서 시작시키면 "우왁" 회전이 없다. 큰 각도 차는 보간이 삼킨다.
            if let Some(tg) = target {
                let d = tg - player;
                self.yaw = d.x.atan2(d.z);
            }
        }
        self.mode = mode;
        self.blend = Some((self.rig, 0.0));
    }

    /// 카메라가 플레이어 머리에 얼마나 가까운가(가림 당기기 뒤). 렌더가 가까우면 플레이어를 옅게.
    pub fn closeness(&self, player: Vec3) -> f32 {
        self.rig
            .pos
            .distance(player + Vec3::Y * (PLAYER_HEIGHT * 0.8))
    }

    pub fn blending(&self) -> Option<f32> {
        self.blend.map(|(_, e)| e)
    }

    /// 원하는 리그(전환·충돌 전).
    fn desired(&self, t: &Tuning, player: Vec3, target: Option<Vec3>) -> Rig {
        match (self.mode, target) {
            (Mode::Lock, Some(tg)) => {
                let (s, c) = self.yaw.sin_cos();
                let fwd = vec3(s, 0.0, c);
                let right = fwd.cross(Vec3::Y);
                let pos = player - fwd * t[P::CamLDist]
                    + Vec3::Y * t[P::CamLHeight]
                    + right * t[P::CamLShoulder];
                let a = player + Vec3::Y * (PLAYER_HEIGHT * 0.6);
                let b = tg + Vec3::Y * (PLAYER_HEIGHT * 0.6);
                // 주시점을 옆으로 밀어야 플레이어가 화면 1/3로 간다(카메라만 어깨로 옮기면 몇 도밖에 안 돈다).
                let look = a.lerp(b, t[P::CamLLookMix]) + right * t[P::CamLFrame];
                Rig {
                    pos,
                    look,
                    fov: t[P::CamLFov].to_radians(),
                }
            }
            _ => {
                let pitch = t[P::CamQPitch].to_radians();
                let d = t[P::CamQDist];
                // 요를 돌려 상자의 두 면이 보이게(축 정렬이면 정면 한 면만 보여 납작하다). 화면 위 = `quarter_forward`.
                let (sy, cy) = t[P::CamQYaw].to_radians().sin_cos();
                let pos =
                    self.follow + vec3(d * pitch.cos() * sy, d * pitch.sin(), d * pitch.cos() * cy);
                Rig {
                    pos,
                    look: self.follow + Vec3::Y * 0.5,
                    fov: t[P::CamQFov].to_radians(),
                }
            }
        }
    }

    /// 틱마다. `target`은 락온 대상 위치(없으면 쿼터뷰로 돌아온 것).
    pub fn update(
        &mut self,
        dt: f32,
        t: &Tuning,
        player: Vec3,
        target: Option<Vec3>,
        obstacles: &[Aabb],
    ) {
        // 추적점·요는 전환과 무관하게 늘 따라간다(전환이 끝난 뒤 튀지 않게).
        let k = 1.0 - (-t[P::CamQLag] * dt).exp();
        self.follow = self.follow.lerp(player, k);
        if let Some(tg) = target {
            let d = tg - player;
            if d.length_squared() > 1e-4 {
                let want = d.x.atan2(d.z);
                let mut diff = want - self.yaw;
                while diff > std::f32::consts::PI {
                    diff -= std::f32::consts::TAU;
                }
                while diff < -std::f32::consts::PI {
                    diff += std::f32::consts::TAU;
                }
                let ky = 1.0 - (-t[P::CamLYawLag] * dt).exp();
                self.yaw += diff * ky;
            }
        }
        let want = self.desired(t, player, target);
        let mut rig = want;
        if let Some((from, elapsed)) = self.blend {
            let e = elapsed + dt;
            let d = t[P::CamBlend].max(1e-3);
            if e >= d {
                self.blend = None;
            } else {
                let x = e / d;
                let s = x * x * (3.0 - 2.0 * x); // smoothstep = ease in-out
                rig = Rig::lerp(from, want, s);
                self.blend = Some((from, e));
            }
        }
        // 충돌: 플레이어 머리 → 카메라 선분이 상자에 막히면 그 앞으로 당긴다(벽 안으로는 절대 안 들어간다).
        let head = player + Vec3::Y * (PLAYER_HEIGHT * 0.8);
        let r = t[P::CamCollideRadius];
        rig.pos = pull_in(head, rig.pos, r, obstacles);
        // 흔들림: trauma² 크기의 무작위 오프셋(로직 시계를 쓰므로 히트스톱 중엔 멈춘다).
        self.trauma = (self.trauma - dt * 2.5).max(0.0);
        let a = self.trauma * self.trauma * 0.25;
        self.shake = vec3(rand::gen_range(-a, a), rand::gen_range(-a, a), 0.0);
        self.rig = rig;
    }

    /// macroquad 카메라로. 렌더 타깃을 쓰면 `aspect`를 꼭 준다(안 주면 화면 비율로 찌그러진다).
    pub fn to_camera3d(&self, rt: RenderTarget, aspect: f32) -> Camera3D {
        Camera3D {
            position: self.rig.pos + self.shake,
            target: self.rig.look + self.shake,
            up: Vec3::Y, // 기본값은 z-up — 매번 적는다.
            fovy: self.rig.fov,
            aspect: Some(aspect),
            projection: Projection::Perspective,
            render_target: Some(rt),
            viewport: None,
            z_near: 0.1,
            z_far: 200.0,
        }
    }
}

/// 선분 `from → to`가 상자에 막히면 막힌 지점에서 `radius`만큼 앞으로 당긴 점. 안 막히면 `to`.
pub fn pull_in(from: Vec3, to: Vec3, radius: f32, obstacles: &[Aabb]) -> Vec3 {
    let d = to - from;
    let len = d.length();
    if len < 1e-4 {
        return to;
    }
    let dir = d / len;
    let mut best = len;
    for b in obstacles {
        if let Some(t) = b.ray(from, dir) {
            if t < best {
                best = t;
            }
        }
    }
    if best >= len {
        to
    } else {
        from + dir * (best - radius).max(CAM_MIN_DIST)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_is_pulled_in_front_of_a_wall_but_never_behind_the_head() {
        let wall = Aabb {
            min: vec3(-5.0, 0.0, 4.0),
            max: vec3(5.0, 3.0, 5.0),
        };
        let head = vec3(0.0, 1.4, 0.0);
        let p = pull_in(head, vec3(0.0, 1.4, 8.0), 0.3, &[wall]);
        assert!((p.z - 3.7).abs() < 1e-3, "{p:?}");
        let p = pull_in(head, vec3(0.0, 1.4, 3.0), 0.3, &[wall]);
        assert_eq!(p, vec3(0.0, 1.4, 3.0));
        let p = pull_in(
            head,
            vec3(0.0, 1.4, 0.2),
            0.3,
            &[Aabb {
                min: vec3(-1.0, 0.0, 0.1),
                max: vec3(1.0, 3.0, 0.3),
            }],
        );
        assert!(
            p.z >= CAM_MIN_DIST - 1e-6,
            "never closer than CAM_MIN_DIST: {p:?}"
        );
    }

    #[test]
    fn lock_mode_blends_from_the_current_rig_and_ends_at_the_wanted_one() {
        let t = Tuning::default();
        let player = Vec3::ZERO;
        let boss = vec3(0.0, 0.0, -6.0);
        let mut c = Camera::new(player, &t);
        let start = c.rig;
        c.set_mode(Mode::Lock, Some(boss), player);
        c.update(1.0 / 60.0, &t, player, Some(boss), &[]);
        assert!(c.blending().is_some());
        // 첫 틱은 출발 리그 근처.
        assert!(c.rig.pos.distance(start.pos) < 1.0);
        for _ in 0..120 {
            c.update(1.0 / 60.0, &t, player, Some(boss), &[]);
        }
        assert!(c.blending().is_none());
        // 플레이어 뒤(+z 쪽), 높이 CamLHeight, 대상(-z)을 본다.
        assert!(
            c.rig.pos.z > 0.0 && (c.rig.pos.y - t[P::CamLHeight]).abs() < 1e-3,
            "{:?}",
            c.rig
        );
        assert!(c.rig.look.z < 0.0);
        assert!((c.rig.fov - t[P::CamLFov].to_radians()).abs() < 1e-5);
    }
}

#[cfg(test)]
mod framing_tests {
    use super::*;

    fn ndc_x(rig: &Rig, p: Vec3) -> f32 {
        let m = Mat4::perspective_rh_gl(rig.fov, 16.0 / 9.0, 0.1, 200.0)
            * Mat4::look_at_rh(rig.pos, rig.look, Vec3::Y);
        let c = m * p.extend(1.0);
        c.x / c.w
    }

    #[test]
    fn lock_on_frames_the_player_left_of_center_and_the_target_right_of_the_player() {
        let t = Tuning::default();
        let player = vec3(0.0, 0.0, -4.5);
        let boss = vec3(0.0, 0.0, -7.1);
        let mut c = Camera::new(player, &t);
        c.set_mode(Mode::Lock, Some(boss), player);
        for _ in 0..180 {
            c.update(1.0 / 60.0, &t, player, Some(boss), &[]);
        }
        let px = ndc_x(&c.rig, player + Vec3::Y * 0.9);
        let bx = ndc_x(&c.rig, boss + Vec3::Y * 1.6);
        println!("player ndc x {px}, boss ndc x {bx}, rig {:?}", c.rig);
        assert!(px < -0.15, "player should sit left of center: {px}");
        assert!(bx > px, "target right of the player: {bx} vs {px}");
    }
}
