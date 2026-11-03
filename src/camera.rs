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

/// 리그를 중심점(플레이어를 따라가는 점) 둘레의 궤도로 본 것. 전환은 이것을 보간한다(0009):
/// 위치를 직선으로 옮기면 카메라가 반대편으로 넘어갈 때 플레이어 머리 위를 가로질러 시선이 뒤집혔다(990°/s).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Orbit {
    /// 카메라 자리의 방위(라디안, atan2(dx, dz)).
    yaw: f32,
    /// 올려본 각(라디안).
    elev: f32,
    dist: f32,
    /// 주시점 − 중심점을 카메라 앞(수평)·오른쪽·위 축으로 잰 것 — 요와 함께 돈다.
    look: Vec3,
    fov: f32,
}

impl Orbit {
    fn axes(yaw: f32) -> (Vec3, Vec3) {
        // 카메라 → 중심 수평 방향 = 카메라 자리 방위의 반대.
        let fwd = -vec3(yaw.sin(), 0.0, yaw.cos());
        (fwd, fwd.cross(Vec3::Y))
    }

    fn from_rig(r: &Rig, pivot: Vec3) -> Orbit {
        let d = r.pos - pivot;
        let dist = d.length().max(1e-3);
        let yaw = d.x.atan2(d.z);
        let (fwd, right) = Orbit::axes(yaw);
        let l = r.look - pivot;
        Orbit {
            yaw,
            elev: (d.y / dist).clamp(-1.0, 1.0).asin(),
            dist,
            look: vec3(l.dot(fwd), l.dot(right), l.y),
            fov: r.fov,
        }
    }

    fn to_rig(self, pivot: Vec3) -> Rig {
        let (fwd, right) = Orbit::axes(self.yaw);
        let h = self.dist * self.elev.cos();
        let pos = pivot
            + vec3(
                self.yaw.sin() * h,
                self.dist * self.elev.sin(),
                self.yaw.cos() * h,
            );
        Rig {
            pos,
            look: pivot + fwd * self.look.x + right * self.look.y + Vec3::Y * self.look.z,
            fov: self.fov,
        }
    }
}

fn wrap(mut a: f32) -> f32 {
    while a > std::f32::consts::PI {
        a -= std::f32::consts::TAU;
    }
    while a < -std::f32::consts::PI {
        a += std::f32::consts::TAU;
    }
    a
}

fn smoothstep(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

pub struct Camera {
    pub mode: Mode,
    pub rig: Rig,
    /// 전환 중이면 (출발 궤도, 경과 시간, 이번 전환 길이 — 첫 갱신에 돌 각도로 정한다).
    blend: Option<(Orbit, f32, Option<f32>)>,
    /// 쿼터뷰 추적점(플레이어를 지연 추적).
    follow: Vec3,
    /// 락온 요(라디안, 대상 방향을 지연 추적).
    yaw: f32,
    /// 상자에서 밀어낸 만큼(부드럽게 따라간다). 0 = 밀 것 없음.
    push: Vec3,
    /// 직전 플레이어 자리와 부드럽게 한 속도(쿼터뷰 앞서 보기, 0008).
    last_player: Option<Vec3>,
    vel: Vec3,
    /// 락온 주시점(부드럽게 따라간다). 대상에 바짝 붙어 돌면 플레이어↔대상 사이 점이 초당 수백 도로 돌았다(0007).
    look: Option<Vec3>,
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
            push: Vec3::ZERO,
            last_player: None,
            vel: Vec3::ZERO,
            look: None,
            trauma: 0.0,
            shake: Vec3::ZERO,
        };
        c.rig = c.desired(t, None);
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
        self.blend = Some((Orbit::from_rig(&self.rig, self.follow), 0.0, None));
    }

    /// 카메라가 플레이어 머리에 얼마나 가까운가(가림 당기기 뒤). 렌더가 가까우면 플레이어를 옅게.
    pub fn closeness(&self, player: Vec3) -> f32 {
        self.rig
            .pos
            .distance(player + Vec3::Y * (PLAYER_HEIGHT * 0.8))
    }

    pub fn blending(&self) -> Option<f32> {
        self.blend.map(|(_, e, _)| e)
    }

    /// 원하는 리그(전환·충돌 전).
    fn desired(&self, t: &Tuning, target: Option<Vec3>) -> Rig {
        match (self.mode, target) {
            (Mode::Lock, Some(tg)) => {
                // 자리는 부드럽게 따라가는 점(`follow`)에서: 대상 곁을 빠르게 돌 때 카메라가 플레이어와 함께 옆으로 홱 밀리지 않게(0007).
                let player = self.follow;
                let (s, c) = self.yaw.sin_cos();
                let fwd = vec3(s, 0.0, c);
                let right = fwd.cross(Vec3::Y);
                let pos = player - fwd * t[P::CamLDist]
                    + Vec3::Y * t[P::CamLHeight]
                    + right * t[P::CamLShoulder];
                let _ = tg; // 방향은 요(대상 쪽)가 정한다.
                            // 주시점 = 플레이어 앞 일정 거리(대상과의 사이 점이 아니다): 대상에 다가가도 카메라가 고개를 숙이며 플레이어가
                            // 화면 위로 떠오르지 않는다(예전 사이 점: 다가가면 −5° → −11°, 0008). 옆으로 밀어야 플레이어가 화면 1/3에 선다.
                let look = player
                    + Vec3::Y * (PLAYER_HEIGHT * 0.6)
                    + fwd * t[P::CamLLookAhead]
                    + right * t[P::CamLFrame];
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
        // 쿼터뷰는 가는 쪽을 앞서 본다: 따라가기 지연(속도/지연율만큼 뒤)을 넘게 앞을 노려, 걷는 동안 플레이어가 가는 반대편에 머문다.
        // 예전엔 위로 걸으면 플레이어가 화면 위로 밀려(+0.15) 가는 쪽이 덜 보였다(0008). 회피 같은 짧은 돌진은 부드러운 속도라 덜 탄다.
        let raw = self
            .last_player
            .map_or(Vec3::ZERO, |l| (player - l) / dt.max(1e-4));
        self.last_player = Some(player);
        self.vel = self.vel.lerp(raw, 1.0 - (-6.0 * dt).exp());
        let lead = if self.mode == Mode::Quarter {
            (self.vel * t[P::CamQLead]).clamp_length_max(1.5)
        } else {
            Vec3::ZERO
        };
        let k = 1.0 - (-t[P::CamQLag] * dt).exp();
        self.follow = self.follow.lerp(player + lead, k);
        if let Some(tg) = target {
            let d = tg - player;
            let dist = vec2(d.x, d.z).length();
            if dist > 1e-2 {
                let want = d.x.atan2(d.z);
                let mut diff = want - self.yaw;
                while diff > std::f32::consts::PI {
                    diff -= std::f32::consts::TAU;
                }
                while diff < -std::f32::consts::PI {
                    diff += std::f32::consts::TAU;
                }
                // 대상에 바짝 붙으면 같은 옆걸음에도 각도가 빨리 바뀐다(각속도 = 속도/거리) → 가까울수록 덜 따라가고,
                // 어떤 경우에도 초당 CamLYawMax도까지만 돈다(측정: 2.5m에서 130°/s였다, 0007).
                let near = t[P::CamLNear];
                let slow = if near > 0.0 {
                    (dist / near).clamp(0.0, 1.0).powi(2)
                } else {
                    1.0
                };
                let ky = 1.0 - (-t[P::CamLYawLag] * slow * dt).exp();
                // 상한도 가까울수록 낮춘다(바짝 붙어 원을 그리면 대상 방향이 초당 200° 넘게 바뀐다 — 다 따라가면 멀미).
                // 바닥 0.4: 대상이 화면 밖으로 영영 빠지지는 않게.
                let max = t[P::CamLYawMax].to_radians() * slow.max(0.4) * dt;
                self.yaw += (diff * ky).clamp(-max, max);
            }
        }
        let mut want = self.desired(t, target);
        if self.mode == Mode::Lock && target.is_some() {
            let cur = self.look.unwrap_or(want.look);
            let k = 1.0 - (-t[P::CamLYawLag] * dt).exp();
            want.look = cur.lerp(want.look, k);
            self.look = Some(want.look);
        } else {
            self.look = None;
        }
        let mut rig = want;
        if let Some((from, elapsed, dur)) = self.blend {
            // 궤도로 보간: 방위는 짧은 쪽으로 돌고, 높이 각·거리·주시점·FOV는 따로. 돌 각도가 크면 그만큼 길게
            // (`cam.blend s` × (1 + 각/180° × `cam.blend turn`)), 방위는 위치보다 살짝 늦게 시작한다(먼저 내려오고 돈다).
            let to = Orbit::from_rig(&want, self.follow);
            let dyaw = wrap(to.yaw - from.yaw);
            // 쿼터뷰로 돌아갈 때는 더 빨리(`cam.blend exit x`, 0.7 — 0.5는 464°/s까지 올라 멀미 쪽): 넓게 보는 것은 빨리 돌아와야 한다(0010).
            let exit = if self.mode == Mode::Quarter {
                t[P::CamBlendExit]
            } else {
                1.0
            };
            let d = dur.unwrap_or_else(|| {
                t[P::CamBlend]
                    * exit
                    * (1.0 + dyaw.abs() / std::f32::consts::PI * t[P::CamBlendTurn])
            });
            let e = elapsed + dt;
            if e >= d {
                self.blend = None;
            } else {
                let x = e / d.max(1e-3);
                let s = smoothstep(x);
                let sy = smoothstep((x - 0.15) / 0.85);
                let o = Orbit {
                    yaw: from.yaw + dyaw * sy,
                    elev: from.elev + (to.elev - from.elev) * s,
                    dist: from.dist + (to.dist - from.dist) * s,
                    look: from.look.lerp(to.look, s),
                    fov: from.fov + (to.fov - from.fov) * s,
                };
                rig = o.to_rig(self.follow);
                self.blend = Some((from, e, Some(d)));
            }
        }
        // 충돌: 카메라 구가 상자 **안에** 들 때만 가장 얕은 면 밖으로 밀어낸다(머리 쪽으로 당기지 않는다 — 옆으로 미끄러져
        // 들어갈 때 당길 거리가 한 틱에 몇 m씩 바뀌어 튄다). 가리기만 하는 것은 렌더가 옅게 그린다(`occluders`).
        // 예전엔 선분이 막히면 그 앞으로 한 틱에 당겨 600 m/s로 튀었다(0007). 풀릴 때만 천천히.
        let r = t[P::CamCollideRadius];
        let want_push = push_out(rig.pos, r, obstacles, self.rig.pos) - rig.pos;
        // 미는 것은 바로(늦으면 카메라 중심이 상자 안에 들어가 면 고르기로 튄다 — 밖에서 미는 것은 연속이라 바로 해도 매끄럽다),
        // 풀리는 것만 천천히.
        if want_push.length_squared() >= self.push.length_squared() {
            self.push = want_push;
        } else {
            self.push += (want_push - self.push) * (1.0 - (-t[P::CamPullOut] * dt).exp());
        }
        rig.pos += self.push;
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

/// 구(`p`, `r`)를 겹친 상자 밖으로 민 자리. 구의 중심이 상자 밖이면 가장 가까운 점에서 멀어지는 쪽으로 민다 —
/// 모서리에서 둥글게 돌아 연속이다(가장 얕은 면으로 밀면 모서리에서 x면↔z면이 바뀌며 한 틱에 튀었다, 0007).
/// 중심이 상자 안이면 직전 카메라(`prev`)가 있던 쪽 면으로 — 그때그때 가장 얕은 면을 고르면 상자 한가운데를 지날 때 반대편으로
/// 튄다(기둥에 등지고 다가갈 때 78 m/s, 0007). 두 번 돌려 붙은 상자도 푼다.
pub fn push_out(p: Vec3, r: f32, obstacles: &[Aabb], prev: Vec3) -> Vec3 {
    let mut q = p;
    for _ in 0..2 {
        for b in obstacles {
            let c = q.clamp(b.min, b.max);
            let d2 = c.distance_squared(q);
            if d2 >= r * r {
                continue;
            }
            if d2 > 1e-8 {
                let d = d2.sqrt();
                q += (q - c) / d * (r - d);
                continue;
            }
            // 직전 카메라가 상자 밖으로 가장 많이 나가 있던 축·쪽.
            let sides = [
                (prev.x - b.max.x, Vec3::X, b.max.x + r - q.x),
                (b.min.x - prev.x, Vec3::NEG_X, q.x - (b.min.x - r)),
                (prev.z - b.max.z, Vec3::Z, b.max.z + r - q.z),
                (b.min.z - prev.z, Vec3::NEG_Z, q.z - (b.min.z - r)),
                (prev.y - b.max.y, Vec3::Y, b.max.y + r - q.y),
            ];
            let (_, n, depth) = sides
                .iter()
                .copied()
                .fold(
                    (f32::MIN, Vec3::ZERO, 0.0),
                    |a, o| if o.0 > a.0 { o } else { a },
                );
            q += n * depth.max(0.0);
        }
    }
    q
}

/// 카메라에서 `points`(플레이어 몸 여러 점, 락온 대상) 중 하나라도 가리는 상자들(렌더가 옅게 그린다).
/// 머리 한 점만 보면 다리·몸통·대상을 가리는 기둥이 그대로였다(사장 실기, 0008).
pub fn occluders(points: &[Vec3], cam: Vec3, obstacles: &[Aabb]) -> Vec<usize> {
    obstacles
        .iter()
        .enumerate()
        .filter(|(_, b)| {
            points.iter().any(|&p| {
                let len = p.distance(cam);
                len > 1e-4 && b.ray(cam, (p - cam) / len).is_some_and(|t| t < len)
            })
        })
        .map(|(i, _)| i)
        .collect()
}

/// 가림을 볼 점들: 발·허리·머리 × 가운데·좌우(카메라 기준 옆으로 `half`), 그리고 대상이 있으면 대상의 허리·머리.
pub fn view_points(
    player: Vec3,
    height: f32,
    half: f32,
    cam: Vec3,
    target: Option<(Vec3, f32)>,
) -> Vec<Vec3> {
    let side = {
        let f = vec3(player.x - cam.x, 0.0, player.z - cam.z);
        if f.length_squared() > 1e-6 {
            f.normalize().cross(Vec3::Y) * half
        } else {
            Vec3::X * half
        }
    };
    let mut v = Vec::with_capacity(12);
    for h in [0.15, 0.5, 0.95] {
        let c = player + Vec3::Y * (height * h);
        v.extend_from_slice(&[c, c + side, c - side]);
    }
    if let Some((tg, th)) = target {
        v.push(tg + Vec3::Y * (th * 0.5));
        v.push(tg + Vec3::Y * (th * 0.9));
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_is_pushed_out_only_when_inside_a_box_and_a_box_that_merely_hides_the_view_is_an_occluder(
    ) {
        let head = vec3(0.0, 1.4, 0.0);
        // 낮은 기둥이 머리→카메라를 가리기만 한다(카메라는 기둥보다 위): 밀지 않고, 가리는 상자로 잡힌다.
        let low = Aabb {
            min: vec3(-1.0, 0.0, 2.0),
            max: vec3(1.0, 3.0, 3.0),
        };
        let cam = vec3(0.0, 6.0, 7.0);
        assert_eq!(push_out(cam, 0.3, &[low], cam), cam);
        assert_eq!(occluders(&[head], cam, &[low]), vec![0]);
        // 발만 가리는 낮은 상자도 몸 여러 점으로 보면 잡힌다(머리 한 점으로는 못 잡는다).
        let ankle = Aabb {
            min: vec3(-1.0, 0.0, 0.4),
            max: vec3(1.0, 0.9, 0.6),
        };
        let pts = view_points(Vec3::ZERO, 1.8, 0.3, cam, None);
        assert!(occluders(&[head], cam, &[ankle]).is_empty());
        assert_eq!(occluders(&pts, cam, &[ankle]), vec![0]);
        // 카메라 구가 벽 면에 닿았다(중심은 밖): 가장 가까운 점에서 멀어지는 쪽으로 반지름만큼.
        let wall = Aabb {
            min: vec3(-5.0, 0.0, 4.0),
            max: vec3(5.0, 3.0, 5.0),
        };
        let p = push_out(vec3(0.0, 1.4, 3.9), 0.3, &[wall], vec3(0.0, 1.4, 3.0));
        assert!((p.z - 3.7).abs() < 1e-4, "{p:?}");
        // 중심이 상자 안: 직전 카메라가 있던 쪽(-z)으로 나간다 — 더 얕은 +z 쪽이 아니라.
        let p = push_out(vec3(0.0, 1.4, 4.7), 0.3, &[wall], vec3(0.0, 1.4, 3.0));
        assert!((p.z - 3.7).abs() < 1e-4, "{p:?}");
    }

    #[test]
    fn going_back_to_quarter_view_is_faster_than_locking_on() {
        let t = Tuning::default();
        let player = Vec3::ZERO;
        let boss = vec3(0.0, 0.0, 6.0); // 화면 아래쪽 — 크게 돈다
        let mut c = Camera::new(player, &t);
        let mut ticks = |c: &mut Camera, mode: Mode, tg: Option<Vec3>| {
            c.set_mode(mode, tg, player);
            let mut n = 0;
            c.update(1.0 / 60.0, &t, player, tg, &[]);
            while c.blending().is_some() && n < 600 {
                c.update(1.0 / 60.0, &t, player, tg, &[]);
                n += 1;
            }
            n
        };
        let enter = ticks(&mut c, Mode::Lock, Some(boss));
        let exit = ticks(&mut c, Mode::Quarter, None);
        assert!(
            exit * 10 <= enter * 8,
            "enter {enter} ticks, exit {exit} ticks"
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

#[cfg(test)]
mod motion_probe {
    //! 카메라 움직임 측정: `cargo test motion_probe -- --nocapture`로 표를 본다.
    use super::*;
    use crate::input::Actions;
    use crate::scenario::{build, Scenario};

    fn dir_of(r: &Rig) -> Vec3 {
        (r.look - r.pos).normalize()
    }

    /// 한 시나리오에서 시작 자리·스틱 방향마다 3초 걷고, 카메라 위치 속도(m/s)와 시선 회전(°/s)의 최댓값을 찍는다.
    fn probe(sc: Scenario, lock: bool, strict: bool) {
        let t = Tuning::default();
        let starts = [
            (-6.0, 4.0),
            (6.0, 4.0),
            (-6.0, -4.0),
            (6.0, -4.0),
            (0.0, -5.5),
            (-3.0, -8.0),
            (3.0, -8.0),
        ];
        for (sx, sz) in starts {
            for stick in [
                vec2(1.0, 0.0),
                vec2(0.0, 1.0),
                vec2(-1.0, 0.0),
                vec2(0.0, -1.0),
            ] {
                let mut w = build(sc);
                w.player.pos = vec3(sx, 0.0, sz);
                w.player.prev = w.player.pos;
                for e in &mut w.enemies {
                    e.state = crate::world::EState::Dead; // 적이 끼어들지 않게(락온 대상은 아래서 살린다)
                }
                if lock {
                    w.enemies[0].state = crate::world::EState::Recover { t: -1000.0 };
                }
                let mut c = Camera::new(w.player.pos, &t);
                if lock {
                    w.step(
                        &Actions {
                            lock: true,
                            ..Default::default()
                        },
                        &t,
                    );
                    let tg = w.lock_target();
                    c.set_mode(Mode::Lock, tg, w.player.pos);
                }
                for _ in 0..60 {
                    c.update(TICK, &t, w.player.pos, w.lock_target(), &w.obstacles);
                }
                let (mut vmax, mut wmax, mut dmin) = (0.0f32, 0.0f32, f32::MAX);
                let mut prev = c.rig;
                for _ in 0..180 {
                    w.step(
                        &Actions {
                            move_dir: stick,
                            ..Default::default()
                        },
                        &t,
                    );
                    c.update(TICK, &t, w.player.pos, w.lock_target(), &w.obstacles);
                    vmax = vmax.max(c.rig.pos.distance(prev.pos) / TICK);
                    let ang = dir_of(&prev)
                        .dot(dir_of(&c.rig))
                        .clamp(-1.0, 1.0)
                        .acos()
                        .to_degrees();
                    wmax = wmax.max(ang / TICK);
                    if let Some(tg) = w.lock_target() {
                        dmin =
                            dmin.min(vec2(tg.x - w.player.pos.x, tg.z - w.player.pos.z).length());
                    }
                    prev = c.rig;
                }
                if strict {
                    // 쿼터뷰는 플레이어 속도(6)를 크게 넘지 않고 거의 돌지 않는다. 락온은 상한(90°/s) 근처까지만.
                    let (vlim, wlim) = if lock {
                        (25.0, t[P::CamLYawMax] * 1.4)
                    } else {
                        (15.0, 30.0)
                    };
                    assert!(vmax < vlim && wmax < wlim, "{sc:?} lock={lock} start=({sx},{sz}) stick={stick:?}: {vmax:.1} m/s, {wmax:.1} deg/s");
                }
                println!(
                    "{:?} lock={lock} start=({sx:>4},{sz:>4}) stick=({:>2},{:>2}) cam speed max {vmax:6.1} m/s  turn max {wmax:6.1} deg/s  closest {}",
                    sc, stick.x, stick.y,
                    if dmin < f32::MAX { format!("{dmin:.1} m") } else { "-".into() }
                );
            }
        }
    }

    const TICK: f32 = 1.0 / 60.0;

    /// 0007: 위치에 따라 카메라가 순간이동하거나(기둥에 당김) 휙 돌던 것(사장 실기 2026-11-03). 시나리오의 여러 자리에서 걸어 본다.
    #[test]
    fn camera_never_jumps_or_whips_anywhere_in_the_arenas() {
        probe(Scenario::Boss, false, true);
        probe(Scenario::Pillars, false, true);
        probe(Scenario::Boss, true, true);
        probe(Scenario::Pillars, true, true);
    }
}

#[cfg(test)]
mod frame_probe {
    //! 0008: 걷는 방향·자리에 따라 플레이어가 화면에서 밀리던 것. `cargo test frame_probe -- --nocapture`로 표를 본다.
    use super::*;
    use crate::input::Actions;
    use crate::scenario::{build, Scenario};

    fn ndc(r: &Rig, p: Vec3) -> Vec2 {
        let m = Mat4::perspective_rh_gl(r.fov, 16.0 / 9.0, 0.1, 200.0)
            * Mat4::look_at_rh(r.pos, r.look, Vec3::Y);
        let c = m * p.extend(1.0);
        vec2(c.x / c.w, c.y / c.w)
    }

    /// 2.5초 걸은 뒤의 (화면 위치, 카메라 기울기°)와 걷는 동안 기울기의 폭.
    fn walk(lock: bool, stick: Vec2) -> (Vec2, f32) {
        let t = Tuning::default();
        let mut w = build(Scenario::Boss);
        w.player.pos = vec3(0.0, 0.0, 6.0);
        w.player.prev = w.player.pos;
        w.enemies[0].state = crate::world::EState::Recover { t: -1000.0 };
        let mut c = Camera::new(w.player.pos, &t);
        if lock {
            w.step(
                &Actions {
                    lock: true,
                    ..Default::default()
                },
                &t,
            );
            c.set_mode(Mode::Lock, w.lock_target(), w.player.pos);
        }
        for _ in 0..60 {
            c.update(1.0 / 60.0, &t, w.player.pos, w.lock_target(), &w.obstacles);
        }
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for _ in 0..150 {
            w.step(
                &Actions {
                    move_dir: stick,
                    ..Default::default()
                },
                &t,
            );
            c.update(1.0 / 60.0, &t, w.player.pos, w.lock_target(), &w.obstacles);
            let pitch = (c.rig.look - c.rig.pos).normalize().y.asin().to_degrees();
            lo = lo.min(pitch);
            hi = hi.max(pitch);
        }
        let n = ndc(&c.rig, w.player.pos + Vec3::Y * 0.9);
        println!(
            "lock={lock} stick={stick:?}: screen ({:+.2}, {:+.2}), pitch range {:.1}°",
            n.x,
            n.y,
            hi - lo
        );
        (n, hi - lo)
    }

    #[test]
    fn quarter_view_looks_ahead_and_lock_on_keeps_its_tilt_while_approaching() {
        // 쿼터뷰: 위로 걸으면 플레이어가 가운데보다 아래(가는 쪽이 보인다), 아래로 걸으면 위.
        let (up, _) = walk(false, vec2(0.0, 1.0));
        let (down, _) = walk(false, vec2(0.0, -1.0));
        assert!(up.y < 0.02 && down.y > 0.05, "up {up:?} down {down:?}");
        // 락온: 보스 쪽으로 걸어 다가가도 카메라 기울기가 거의 그대로(예전 −5° → −11°).
        let (_, range) = walk(true, vec2(0.0, 1.0));
        assert!(range < 2.5, "pitch range {range}");
    }
}

#[cfg(test)]
mod switch_probe {
    //! 0009: 락온·해제 전환이 플레이어 자리(대상과의 방향)에 따라 부자연스럽던 것. `cargo test switch_probe -- --nocapture`.
    use super::*;
    use crate::input::Actions;
    use crate::scenario::{build, Scenario};

    /// 보스(0,0,-8) 둘레의 여러 방향에서 락온 → 1초 → 해제. 전환 동안 시선 회전 최대(°/s), 카메라 속도 최대(m/s),
    /// 카메라와 플레이어 머리의 가장 가까운 거리(m).
    pub fn run(print: bool) -> Vec<(f32, f32, f32, f32)> {
        let t = Tuning::default();
        let mut out = Vec::new();
        for deg in (0..360).step_by(45) {
            let a = (deg as f32).to_radians();
            let mut w = build(Scenario::Boss);
            let boss = w.enemies[0].pos;
            w.player.pos = boss + vec3(a.sin(), 0.0, a.cos()) * 6.0;
            w.player.prev = w.player.pos;
            w.enemies[0].state = crate::world::EState::Recover { t: -1000.0 };
            let mut c = Camera::new(w.player.pos, &t);
            for _ in 0..60 {
                c.update(1.0 / 60.0, &t, w.player.pos, None, &w.obstacles);
            }
            let (mut wmax, mut vmax, mut dmin) = (0.0f32, 0.0f32, f32::MAX);
            let mut prev = c.rig;
            for tick in 0..150 {
                let lock = tick == 0 || tick == 75;
                w.step(
                    &Actions {
                        lock,
                        ..Default::default()
                    },
                    &t,
                );
                let tg = w.lock_target();
                c.set_mode(
                    if tg.is_some() {
                        Mode::Lock
                    } else {
                        Mode::Quarter
                    },
                    tg,
                    w.player.pos,
                );
                c.update(1.0 / 60.0, &t, w.player.pos, tg, &w.obstacles);
                let d0 = (prev.look - prev.pos).normalize();
                let d1 = (c.rig.look - c.rig.pos).normalize();
                wmax = wmax.max(d0.dot(d1).clamp(-1.0, 1.0).acos().to_degrees() * 60.0);
                vmax = vmax.max(c.rig.pos.distance(prev.pos) * 60.0);
                dmin = dmin.min(
                    c.rig
                        .pos
                        .distance(w.player.pos + Vec3::Y * (PLAYER_HEIGHT * 0.8)),
                );
                prev = c.rig;
            }
            if print {
                println!("player at {deg:>3}° around boss: turn max {wmax:6.1} deg/s  cam speed max {vmax:5.1} m/s  closest to head {dmin:4.2} m");
            }
            out.push((deg as f32, wmax, vmax, dmin));
        }
        out
    }

    /// 플레이어가 대상의 어느 쪽에 있든 전환이 비슷하게 부드럽다(고치기 전: 270~990°/s, 머리에서 3.3m까지 스침).
    #[test]
    fn lock_switch_is_smooth_from_every_side_of_the_target() {
        let rows = run(true);
        let worst = rows.iter().map(|r| r.1).fold(0.0, f32::max);
        let best = rows.iter().map(|r| r.1).fold(f32::MAX, f32::min);
        // 해제가 더 빨라(0010, exit 0.7) 최악은 해제 쪽 — 들어갈 때는 260°/s 안쪽.
        assert!(worst < 340.0, "turn max {worst}");
        assert!(worst < best * 1.8, "side matters too much: {best}..{worst}");
        assert!(
            rows.iter().all(|r| r.3 > 4.0),
            "camera grazes the head: {rows:?}"
        );
    }
}
