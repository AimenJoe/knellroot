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
    /// 자유 숄더뷰(0181): 락온과 같은 어깨 리그를 대상 없이 — 방위·높이는 플레이어가 돌린다(오른쪽 스틱·마우스·화면 끌기).
    /// 쿼터뷰에서 가장 가까이 당긴 뒤 한 번 더 당기면, 또는 락온 대상을 잃으면(쓰러지고 넘길 적이 없음) 여기.
    Free,
}

/// 쿼터뷰 거리 단계(0181 — 십자 ↑ 당기기 · ↓ 밀기): `cam.quarter.dist`에 곱한다. 가장 가까운 단계에서 한 번 더 당기면 자유 숄더뷰.
/// 연속 줌이 아니라 단계인 까닭: 거리는 멀미와 타격감의 맞바꿈이라(KB camera-mode-switch) 사람이 판정한 범위 안에서 고르게.
pub const QUARTER_ZOOM: [f32; 3] = [1.3, 1.0, 0.75];
/// 처음 단계(= 랩 거리 그대로).
pub const ZOOM_HOME: u8 = 1;
/// 거리 단계를 바꿀 때 옮겨 가는 시간(초).
const ZOOM_BLEND: f32 = 0.3;
/// 자유 숄더뷰의 높이 각(라디안, 락온 리그에서 더하는 몫) 범위: 아래(올려보기)·위(내려다보기). 땅 밑·머리 위로 넘어가지 않게.
pub const FREE_PITCH_MIN: f32 = -0.25;
pub const FREE_PITCH_MAX: f32 = 0.65;

/// 눈에서 볼 때 가까운 공(가운데·반지름)이 먼 공을 얼마나 가리나(0..1, 각도 원판 둘의 겹침 근사). 0040: 락온 중
/// 방랑자가 과녁을 가리면 옅게 — 렌더가 쓴다. 가까운 쪽이 더 멀리 있으면 0.
pub fn occlusion(eye: Vec3, near_c: Vec3, near_r: f32, far_c: Vec3, far_r: f32) -> f32 {
    let (dn, df) = (near_c - eye, far_c - eye);
    let (ln, lf) = (dn.length(), df.length());
    if ln < 1e-3 || lf <= ln {
        return 0.0;
    }
    let sep = dn.angle_between(df);
    let an = (near_r / ln).min(1.0).asin();
    let af = (far_r / lf).min(1.0).asin();
    (1.0 - sep / (an + af)).clamp(0.0, 1.0)
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

#[derive(Clone, Copy, PartialEq, Debug)]
enum YawMode {
    Fixed,
    Follow,
    Snap90,
}

fn yaw_mode(t: &Tuning) -> YawMode {
    match t[P::CamQYawMode].round() as i32 {
        0 => YawMode::Fixed,
        1 => YawMode::Follow,
        _ => YawMode::Snap90,
    }
}

/// 모드 전환에서 방위가 위치보다 늦게 시작하는 몫(먼저 내려오고 돈다, 0009).
const YAW_DELAY: f32 = 0.15;

/// 전환에서 방위가 도는 모양.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Curve {
    /// 모드 전환: 전체 길이의 이 몫만큼 늦게 시작하는 smoothstep.
    Delay(f32),
    /// 땅 돌리기(0012): `go`초 동안 목표 + `over`(라디안)까지 smoothstep, 남은 시간 동안 목표로 되돌아온다
    /// (둘 다 끝에서 속도 0 — 한 번 멈췄다 되돌아오는 반동).
    /// `dir` = 도는 쪽(±1) — 180°에서 "짧은 쪽"이 정해지지 않아 따로 든다(0023).
    Over { go: f32, over: f32, dir: f32 },
}

fn smoothstep(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

pub struct Camera {
    pub mode: Mode,
    /// 이번 틱의 리그(로직 — 조준 광선·시험이 읽는다). 그림은 `view`를 쓴다.
    pub rig: Rig,
    /// 그릴 때의 리그(0173): 지난 틱과 이번 틱 사이(`present`). 방랑자는 틱 사이를 보간해 그리는데 카메라만 틱에서 뚝뚝 옮기면,
    /// 화면 주사율이 60Hz가 아니거나 프레임 간격이 흔들릴 때 방랑자가 화면에서 한 틱어치씩 앞뒤로 떨린다.
    pub view: Rig,
    /// 지난 틱의 (리그, 흔들림 + 킥, 굴림) · 그릴 때의 흔들림 + 킥, 굴림.
    prev: (Rig, Vec3, f32),
    view_off: Vec3,
    view_roll: f32,
    /// 전환 중이면 (출발 궤도, 경과 시간, 이번 전환 길이 — 없으면 첫 갱신에 돌 각도로 정한다, 방위 늦춤 몫).
    blend: Option<(Orbit, f32, Option<f32>, Curve)>,
    /// 쿼터뷰 추적점(플레이어를 지연 추적).
    follow: Vec3,
    /// 락온 요(라디안, 대상 방향을 지연 추적).
    yaw: f32,
    /// 상자에서 밀어낸 만큼(부드럽게 따라간다). 0 = 밀 것 없음.
    push: Vec3,
    /// 쿼터뷰 카메라 방위(라디안). None = 랩 값 `cam.quarter.yaw` 그대로(고정 방식, 또는 아직 락온을 푼 적 없음).
    q_azimuth: Option<f32>,
    /// 쿼터뷰 거리 단계(`QUARTER_ZOOM`의 자리, 0181).
    pub zoom_step: u8,
    /// 자유 숄더뷰의 높이 각(라디안, `FREE_PITCH_*` 안) — 락온 리그에 더한다.
    pitch: f32,
    /// 락온을 버튼으로 풀면 돌아갈 곳(0181): 자유 숄더뷰에서 락온했으면 거기로, 아니면 쿼터뷰.
    back_free: bool,
    /// 방금 락온을 풀었다: 풀던 순간의 락온 카메라 방위(라디안). 다음 갱신에서 방식에 따라 `q_azimuth`로.
    unlock_azimuth: Option<f32>,
    /// 직전 플레이어 자리와 부드럽게 한 속도(쿼터뷰 앞서 보기, 0008).
    last_player: Option<Vec3>,
    vel: Vec3,
    /// 락온 주시점(부드럽게 따라간다). 대상에 바짝 붙어 돌면 플레이어↔대상 사이 점이 초당 수백 도로 돌았다(0007).
    look: Option<Vec3>,
    /// 화면 흔들림(trauma 0..1, 제곱으로 쓴다).
    pub trauma: f32,
    /// 카메라 킥(2026-11-06): 맞은 쪽으로 찼다가 `KICK_RETURN`으로 돌아오는 월드 오프셋(각 → 거리는 `kick`이 잰다).
    kick: Vec3,
    kick_deg: Vec3,
    /// 타격 줌(0145): 주시점 쪽으로 다가가 있는 몫(거리의 0..`ZOOM_MAX`). `freeze`(히트스톱)인 동안은 줄지 않는다.
    zoom: f32,
    /// 장면이 당겨 둔 몫(0171 — 깨어나는 첫 장면): 주시점 쪽으로 거리의 0..1. 앱이 틱마다 적는다(0 = 제 거리). 타격 줌과 곱해진다.
    pub close: f32,
    pub freeze: bool,
    /// 땅 돌리기 휘청(0084): (지난 초, 굴림 크기 도 — + = 화면이 시계 방향으로 기욺). 줄어드는 흔들림 몇 개의 합.
    wobbles: Vec<(f32, f32)>,
    /// 이번 프레임의 굴림 각(라디안) — `to_camera3d`가 위쪽 벡터를 앞 축 둘레로 돌린다.
    roll: f32,
    shake: Vec3,
}

impl Camera {
    pub fn new(player: Vec3, t: &Tuning) -> Self {
        let zero = Rig {
            pos: Vec3::ZERO,
            look: Vec3::ZERO,
            fov: 1.0,
        };
        let mut c = Camera {
            mode: Mode::Quarter,
            rig: zero,
            view: zero,
            prev: (zero, Vec3::ZERO, 0.0),
            view_off: Vec3::ZERO,
            view_roll: 0.0,
            blend: None,
            follow: player,
            yaw: 0.0,
            push: Vec3::ZERO,
            q_azimuth: None,
            zoom_step: ZOOM_HOME,
            pitch: 0.0,
            back_free: false,
            unlock_azimuth: None,
            last_player: None,
            vel: Vec3::ZERO,
            look: None,
            trauma: 0.0,
            kick: Vec3::ZERO,
            kick_deg: Vec3::ZERO,
            zoom: 0.0,
            close: 0.0,
            freeze: false,
            wobbles: Vec::new(),
            roll: 0.0,
            shake: Vec3::ZERO,
        };
        c.rig = c.desired(t, None);
        c.settle();
        c.present(1.0);
        c
    }

    /// 틱 들머리에: 지난 틱의 리그 = 지금 리그. 이 틱에 `update`가 안 불리면(멈춤 창 아래) 그림이 틱 사이를 오가지 않는다.
    pub fn settle(&mut self) {
        self.prev = (self.rig, self.shake + self.kick, self.roll);
    }

    /// 그리기 전에: 지난 틱과 이번 틱 사이 `alpha`의 리그를 `view`에(0173). 방랑자의 보간과 같은 `alpha`를 준다.
    pub fn present(&mut self, alpha: f32) {
        let a = alpha.clamp(0.0, 1.0);
        let (p, off, roll) = self.prev;
        self.view = Rig {
            pos: p.pos.lerp(self.rig.pos, a),
            look: p.look.lerp(self.rig.look, a),
            fov: p.fov + (self.rig.fov - p.fov) * a,
        };
        self.view_off = off.lerp(self.shake + self.kick, a);
        self.view_roll = roll + (self.roll - roll) * a;
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
            self.back_free = self.mode == Mode::Free;
            self.pitch = 0.0;
        }
        if mode == Mode::Free && self.mode == Mode::Quarter {
            // 쿼터뷰에서 당겨 들어가면 화면 위였던 쪽을 그대로 앞으로(카메라 자리 방위 + 180° = 앞).
            let d = self.rig.pos - self.rig.look;
            self.yaw = d.x.atan2(d.z) + std::f32::consts::PI;
            self.pitch = 0.0;
        }
        if mode == Mode::Quarter && self.mode != Mode::Quarter {
            // 숄더 리그(락온·자유)의 카메라는 앞의 반대편(요 + 180°)에 있다.
            self.unlock_azimuth = Some(self.yaw + std::f32::consts::PI);
        }
        self.mode = mode;
        self.blend = Some((
            Orbit::from_rig(&self.rig, self.follow),
            0.0,
            None,
            Curve::Delay(YAW_DELAY),
        ));
    }

    /// 땅 돌리기(0012·0023): 쿼터뷰 방위를 `quarters` × 90° 돌린다. + = 요 +90° — 화면 위에 있던 것이 화면 오른쪽으로 간다
    /// (화면 왼쪽에 있던 것이 위로). 0이면 돌지 않고(몸만 돌아선다), ±2면 180°를 그 쪽으로.
    /// 플레이어가 누른 것이라 예측된다: 길이는 `turn.camera s`로 고정(180°는 `TURN_HALF_MUL`배), 방위만 돈다.
    /// 락온 중에는 부르지 않는다(월드가 막는다).
    pub fn turn(&mut self, quarters: i32, t: &Tuning) {
        if quarters == 0 {
            return;
        }
        let q = std::f32::consts::FRAC_PI_2;
        if self.mode == Mode::Free {
            // 자유 숄더뷰(0181): 같은 쪽으로 앞을 90°씩(카메라 자리 방위 = 앞 + 180°라 같은 각을 더한다).
            self.yaw = wrap(self.yaw + q * quarters as f32);
        } else {
            let cur = self.quarter_yaw_deg(t).to_radians();
            self.q_azimuth = Some(wrap(cur + q * quarters as f32));
        }
        // 지나쳤다 돌아오기(`turn.overshoot deg`): 목표+α까지 같은 빠르기로 가고(그래서 그만큼 길게), `turn.settle s` 동안 목표로 되돌아와 선다.
        let over = t[P::TurnOver].to_radians();
        let (go, settle) = crate::tuning::turn_phases_q(t, quarters);
        // 시작 휘청(0084): 도는 쪽으로 살짝 기울었다 흔들리며 선다.
        self.wobble(quarters.signum() as f32 * t[P::TurnWobbleIn]);
        self.blend = Some((
            Orbit::from_rig(&self.rig, self.follow),
            0.0,
            Some(go + settle),
            Curve::Over {
                go,
                over,
                dir: quarters.signum() as f32,
            },
        ));
    }

    /// 지금 쿼터뷰 방위(도). 입력 기준(`world::Basis::Quarter`)도 이것을 쓴다 — 월드에 매 틱 넘긴다(0011).
    pub fn quarter_yaw_deg(&self, t: &Tuning) -> f32 {
        self.q_azimuth.map_or(t[P::CamQYaw], f32::to_degrees)
    }

    /// 입력 기준의 방위(도, 0181): 쿼터뷰면 그 방위, 자유 숄더뷰면 카메라가 선 방위(앞의 반대) — 스틱의 위 = 카메라가 보는 쪽.
    pub fn basis_yaw_deg(&self, t: &Tuning) -> f32 {
        match self.mode {
            Mode::Free => (self.yaw + std::f32::consts::PI).to_degrees(),
            _ => self.quarter_yaw_deg(t),
        }
    }

    /// 거리 단계 한 칸(0181 — `dir` + = 당기기). 쿼터뷰의 가장 가까운 단계에서 더 당기면 자유 숄더뷰로, 자유 숄더뷰에서 밀면
    /// 쿼터뷰의 가장 가까운 단계로. 락온 중에는 아무것도 하지 않는다(카메라는 대상이 정한다). 바뀌었으면 참.
    pub fn step_zoom(&mut self, dir: i32, player: Vec3) -> bool {
        let last = QUARTER_ZOOM.len() as u8 - 1;
        match (self.mode, dir.signum()) {
            (Mode::Quarter, 1) if self.zoom_step >= last => {
                self.set_mode(Mode::Free, None, player);
                true
            }
            (Mode::Quarter, s) if s != 0 => {
                let to = (self.zoom_step as i32 + s).clamp(0, last as i32) as u8;
                if to == self.zoom_step {
                    return false;
                }
                self.zoom_step = to;
                self.blend = Some((
                    Orbit::from_rig(&self.rig, self.follow),
                    0.0,
                    Some(ZOOM_BLEND),
                    Curve::Delay(0.0),
                ));
                true
            }
            (Mode::Free, -1) => {
                self.zoom_step = last;
                self.set_mode(Mode::Quarter, None, player);
                true
            }
            _ => false,
        }
    }

    /// 시점 바꾸기(0184 — 터치의 시점 버튼): 쿼터뷰 ↔ 자유 숄더뷰. 거리 단계는 그대로 두어 돌아오면 같은 거리다.
    /// 락온 중에는 아무것도 하지 않는다(카메라는 대상이 정한다). 바뀌었으면 참.
    pub fn toggle_view(&mut self, player: Vec3) -> bool {
        match self.mode {
            Mode::Quarter => self.set_mode(Mode::Free, None, player),
            Mode::Free => self.set_mode(Mode::Quarter, None, player),
            Mode::Lock => return false,
        }
        true
    }

    /// 자유 숄더뷰의 시점 돌리기(0181): 방위 `dyaw`(라디안, + = 오른쪽을 본다)·높이 `dpitch`(+ = 내려다본다). 다른 모드에서는 없다.
    pub fn look(&mut self, dyaw: f32, dpitch: f32) {
        if self.mode != Mode::Free {
            return;
        }
        // 앞 = (sin 요, cos 요): 요가 줄면 오른쪽(앞 × 위 = 오른쪽 축 쪽)으로 돈다.
        self.yaw = wrap(self.yaw - dyaw);
        self.pitch = (self.pitch + dpitch).clamp(FREE_PITCH_MIN, FREE_PITCH_MAX);
    }

    /// 락온 대상을 잃었다(0181 — `Event::LockLost`): 풀린 뒤 쿼터뷰로 돌아가지 않고 자유 숄더뷰에 머문다.
    pub fn lose_lock(&mut self) {
        self.back_free = true;
    }

    /// 새 판·쉼터·지역 전환의 새 카메라(0181): 고른 거리 단계와 자유 숄더뷰(그 방위·높이)는 이어 간다 — 쉴 때마다 쿼터뷰로 돌아가지 않게.
    pub fn renew(&self, player: Vec3, t: &Tuning) -> Camera {
        let mut c = Camera::new(player, t);
        c.zoom_step = self.zoom_step;
        if self.mode == Mode::Free || (self.mode == Mode::Lock && self.back_free) {
            c.mode = Mode::Free;
            c.yaw = self.yaw;
            c.pitch = self.pitch;
        }
        c.rig = c.desired(t, None);
        c.settle();
        c.present(1.0);
        c
    }

    /// 락온을 버튼으로 풀었을 때 돌아갈 모드(0181).
    pub fn unlock_mode(&self) -> Mode {
        if self.back_free {
            Mode::Free
        } else {
            Mode::Quarter
        }
    }

    /// 카메라가 플레이어 머리에 얼마나 가까운가(가림 당기기 뒤). 렌더가 가까우면 플레이어를 옅게.
    pub fn closeness(&self, player: Vec3) -> f32 {
        self.rig
            .pos
            .distance(player + Vec3::Y * (PLAYER_HEIGHT * 0.8))
    }

    /// 지금 카메라가 선 방위(도, 0..360). HUD가 보인다 — 랩 값이 먹는지 눈으로 재기.
    pub fn azimuth_deg(&self) -> f32 {
        let d = self.rig.pos - self.rig.look;
        d.x.atan2(d.z).to_degrees().rem_euclid(360.0)
    }

    pub fn blending(&self) -> Option<f32> {
        self.blend.map(|(_, e, _, _)| e)
    }

    /// 숄더 리그(락온·자유 숄더뷰가 같이 쓴다): 요 = 앞, `pitch` = 주시점 둘레로 더 내려다보는 각(자유 숄더뷰만, 락온은 0).
    fn shoulder(&self, t: &Tuning, pitch: f32) -> Rig {
        // 자리는 부드럽게 따라가는 점(`follow`)에서: 대상 곁을 빠르게 돌 때 카메라가 플레이어와 함께 옆으로 홱 밀리지 않게(0007).
        let player = self.follow;
        let (s, c) = self.yaw.sin_cos();
        let fwd = vec3(s, 0.0, c);
        let right = fwd.cross(Vec3::Y);
        let pos =
            player - fwd * t[P::CamLDist] + Vec3::Y * t[P::CamLHeight] + right * t[P::CamLShoulder];
        // 주시점 = 플레이어 앞 일정 거리(대상과의 사이 점이 아니다): 대상에 다가가도 카메라가 고개를 숙이며 플레이어가
        // 화면 위로 떠오르지 않는다(예전 사이 점: 다가가면 −5° → −11°, 0008). 옆으로 밀어야 플레이어가 화면 1/3에 선다.
        let look = player
            + Vec3::Y * (PLAYER_HEIGHT * 0.6)
            + fwd * t[P::CamLLookAhead]
            + right * t[P::CamLFrame];
        // 높이 각(0181): 카메라를 주시점 둘레로 오른쪽 축을 따라 돌린다(+ = 위로 올라 내려다본다).
        let pos = if pitch.abs() > 1e-5 {
            look + Quat::from_axis_angle(right.normalize_or_zero(), -pitch) * (pos - look)
        } else {
            pos
        };
        Rig {
            pos,
            look,
            fov: t[P::CamLFov].to_radians(),
        }
    }

    /// 원하는 리그(전환·충돌 전).
    fn desired(&self, t: &Tuning, target: Option<Vec3>) -> Rig {
        match (self.mode, target) {
            // 방향은 요(대상 쪽)가 정한다.
            (Mode::Lock, Some(_)) => self.shoulder(t, 0.0),
            (Mode::Free, _) => self.shoulder(t, self.pitch),
            _ => {
                let pitch = t[P::CamQPitch].to_radians();
                let d = t[P::CamQDist]
                    * QUARTER_ZOOM[(self.zoom_step as usize).min(QUARTER_ZOOM.len() - 1)];
                // 요를 돌려 상자의 두 면이 보이게(축 정렬이면 정면 한 면만 보여 납작하다). 화면 위 = `quarter_forward`.
                let (sy, cy) = self.quarter_yaw_deg(t).to_radians().sin_cos();
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
        self.settle();
        // 락온을 풀면 쿼터뷰 방위를 방식에 따라 정한다(0011): 고정 = 랩 값, 따라감 = 락온하던 방향 그대로(돌아가며 돌지 않는다),
        // 90° 단위 = 랩 값 + 90°×k 중 락온 방향에 가장 가까운 것(최대 45°만 돈다, 지도 방향은 넷뿐).
        if let Some(a) = self.unlock_azimuth.take() {
            // 고정 = 락온 해제로는 돌지 않는다(땅 돌리기로 정한 방위는 그대로, 0012).
            self.q_azimuth = match yaw_mode(t) {
                YawMode::Fixed => self.q_azimuth,
                YawMode::Follow => Some(wrap(a)),
                YawMode::Snap90 => {
                    let base = t[P::CamQYaw].to_radians();
                    let q = std::f32::consts::FRAC_PI_2;
                    let k = (wrap(a - base) / q).round();
                    Some(wrap(base + k * q))
                }
            };
        }
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
                // 따라잡기(0173): 대상 방향과의 차가 크면 — 방랑자가 대상 뒤로 돌아가 대상이 카메라와 방랑자 사이에 섰다 — 가까움의
                // 늦춤을 걷고 상한을 올린다. 입력 기준은 그 틱에 대상 쪽으로 바뀌었는데 카메라만 반대편을 보고 있으면 스틱이 화면과
                // 거꾸로다(예전: 2m에서 뒤로 돌면 초당 36° — 다시 보기까지 4초). 차가 작을 때(곁에서 옆걸음)는 예전 그대로 조용하다.
                let at = t[P::CamLCatchAt].to_radians().max(1e-3);
                let far = smoothstep((diff.abs() - at) / at);
                let slow_k = slow + (1.0 - slow) * far;
                let ky = 1.0 - (-t[P::CamLYawLag] * slow_k * dt).exp();
                // 상한도 가까울수록 낮춘다(바짝 붙어 원을 그리면 대상 방향이 초당 200° 넘게 바뀐다 — 다 따라가면 멀미).
                // 바닥 0.4: 대상이 화면 밖으로 영영 빠지지는 않게.
                let calm = t[P::CamLYawMax] * slow.max(0.4);
                let max = (calm + (t[P::CamLCatch] - calm).max(0.0) * far).to_radians() * dt;
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
        if let Some((from, elapsed, dur, curve)) = self.blend {
            // 궤도로 보간: 방위는 짧은 쪽으로 돌고, 높이 각·거리·주시점·FOV는 따로. 돌 각도가 크면 그만큼 길게
            // (`cam.blend s` × (1 + 각/180° × `cam.blend turn`)), 방위는 위치보다 살짝 늦게 시작한다(먼저 내려오고 돈다).
            let to = Orbit::from_rig(&want, self.follow);
            let mut dyaw = wrap(to.yaw - from.yaw);
            // 땅 돌리기는 정한 쪽으로(180°면 짧은 쪽이 매 프레임 뒤집힐 수 있다).
            if let Curve::Over { dir, .. } = curve {
                if dyaw.abs() > 1.0 && dyaw.signum() != dir {
                    dyaw += dir * std::f32::consts::TAU;
                }
            }
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
                // 땅 돌리기가 90°에 닿는 순간 "탁" — 되돌아오기만으로는 끝이 무르다(사장 "마지막 순간 스냅이 약하다").
                if let Curve::Over { over, dir, .. } = curve {
                    if over > 0.0 {
                        self.trauma = (self.trauma + t[P::TurnSnap]).min(1.0);
                    }
                    // 끝 휘청(0084): 멈추는 순간 관성으로 반대쪽으로.
                    self.wobble(-dir * t[P::TurnWobbleOut]);
                }
            } else {
                let x = e / d.max(1e-3);
                let s = smoothstep(x);
                let yaw = match curve {
                    Curve::Delay(delay) => dyaw * smoothstep((x - delay) / (1.0 - delay)),
                    Curve::Over { go, over, .. } => {
                        let o = over * dyaw.signum();
                        if e < go {
                            (dyaw + o) * smoothstep(e / go)
                        } else {
                            dyaw + o - o * smoothstep((e - go) / (d - go).max(1e-3))
                        }
                    }
                };
                let o = Orbit {
                    yaw: from.yaw + yaw,
                    elev: from.elev + (to.elev - from.elev) * s,
                    dist: from.dist + (to.dist - from.dist) * s,
                    look: from.look.lerp(to.look, s),
                    fov: from.fov + (to.fov - from.fov) * s,
                };
                rig = o.to_rig(self.follow);
                self.blend = Some((from, e, Some(d), curve));
            }
        }
        // 충돌: 카메라 구가 상자 **안에** 들 때만 가장 얕은 면 밖으로 밀어낸다(머리 쪽으로 당기지 않는다 — 옆으로 미끄러져
        // 들어갈 때 당길 거리가 한 틱에 몇 m씩 바뀌어 튄다). 가리기만 하는 것은 렌더가 옅게 그린다(`occluders`).
        // 예전엔 선분이 막히면 그 앞으로 한 틱에 당겨 600 m/s로 튀었다(0007). 풀릴 때만 천천히.
        let r = t[P::CamCollideRadius];
        // 가는 기둥은 밀지 않는다 — 카메라가 뚫고 지나가고 렌더가 옅게 그린다(`occluders`). 락온 거리 7m에서 카메라가 기둥을 돌아
        // 나가며 한 틱 133~169°/s로 꺾였다(0012). 밀어낼 것은 벽처럼 넓은 상자뿐.
        let solid: Vec<Aabb> = obstacles.iter().copied().filter(|b| !is_thin(b)).collect();
        let want_push = push_out(rig.pos, r, &solid, self.rig.pos) - rig.pos;
        // 미는 것은 바로(늦으면 카메라 중심이 상자 안에 들어가 면 고르기로 튄다 — 밖에서 미는 것은 연속이라 바로 해도 매끄럽다),
        // 풀리는 것만 천천히.
        if want_push.length_squared() >= self.push.length_squared() {
            self.push = want_push;
        } else {
            self.push += (want_push - self.push) * (1.0 - (-t[P::CamPullOut] * dt).exp());
        }
        rig.pos += self.push;
        // 타격 줌(0145): 주시점 쪽으로 거리의 몫만큼 — 리그 자체를 옮겨 그림·커서 조준·가림이 같은 눈을 쓴다.
        // 세계가 멈춘 동안(히트스톱)은 다가간 채 머물고, 풀리면 돌아온다.
        if !self.freeze {
            self.zoom *= (-crate::tuning::ZOOM_RETURN * dt).exp();
            if self.zoom < 1e-4 {
                self.zoom = 0.0;
            }
        }
        rig.pos = rig.look
            + (rig.pos - rig.look) * (1.0 - self.zoom) * (1.0 - self.close.clamp(0.0, 0.9));
        // 흔들림: trauma² 크기의 무작위 오프셋(로직 시계를 쓰므로 히트스톱 중엔 멈춘다).
        self.trauma = (self.trauma - dt * 2.5).max(0.0);
        // 크기는 각으로(0020): 카메라 거리 × tan(각) — 먼 쿼터뷰에서도 화면에서 보이게. 카메라의 오른쪽·위로 민다(위치·주시점 같이 = 화면 평행 이동).
        let fwd = rig.look - rig.pos;
        let dist = fwd.length().max(1.0);
        let right = fwd.cross(Vec3::Y).normalize_or_zero();
        let up = right.cross(fwd / dist);
        let a = self.trauma * self.trauma * dist * crate::tuning::SHAKE_DEG.to_radians().tan();
        self.shake = right * rand::gen_range(-a, a) + up * rand::gen_range(-a, a);
        // 킥: 각(도 × 방향)을 지금 거리로 — 쿼터뷰·락온 어디서나 화면에서 같은 px쯤. 로직 시계로 줄어 히트스톱 동안 찬 채로 머문다.
        self.kick_deg *= (-crate::tuning::KICK_RETURN * dt).exp();
        self.kick = self.kick_deg * dist * 1f32.to_radians().tan();
        // 휘청(0084): 줄어드는 사인파들의 합 — 로직 시계라 히트스톱 동안 멈춘다.
        for w in &mut self.wobbles {
            w.0 += dt;
        }
        self.wobbles.retain(|w| w.0 < 1.0);
        self.roll = self
            .wobbles
            .iter()
            .map(|&(e, a)| wobble_at(e, a))
            .sum::<f32>()
            .to_radians();
        self.rig = rig;
    }

    /// 땅 돌리기 휘청 하나(굴림 `deg`도, + = 화면이 시계 방향으로 기욺).
    fn wobble(&mut self, deg: f32) {
        if deg.abs() > 1e-3 {
            self.wobbles.push((0.0, deg));
        }
    }

    /// 지금 굴림 각(도, 시험).
    #[cfg(test)]
    pub fn roll_deg(&self) -> f32 {
        self.roll.to_degrees()
    }

    /// 타격 줌(0145): 주시점 쪽으로 거리의 `k` 몫만큼 다가간다 — 겹치면 더 큰 쪽(더하지 않는다), `ZOOM_MAX`까지.
    pub fn punch_in(&mut self, k: f32) {
        self.zoom = self.zoom.max(k).clamp(0.0, crate::tuning::ZOOM_MAX);
    }

    /// 카메라 킥: 수평 방향 `dir` 쪽으로 `deg`도만큼 찬다(겹치면 더하되 한 번의 두 배까지).
    pub fn kick(&mut self, dir: Vec3, deg: f32) {
        let d = dir.normalize_or_zero();
        self.kick_deg = (self.kick_deg + d * deg).clamp_length_max(deg * 2.0);
    }

    /// macroquad 카메라로. 렌더 타깃을 쓰면 `aspect`를 꼭 준다(안 주면 화면 비율로 찌그러진다).
    pub fn to_camera3d(&self, rt: RenderTarget, aspect: f32) -> Camera3D {
        Camera3D {
            position: self.view.pos + self.view_off,
            target: self.view.look + self.view_off,
            // 기본값은 z-up — 매번 적는다. 땅 돌리기 휘청(0084)이면 앞 축 둘레로 살짝 돌린 위.
            up: if self.view_roll.abs() > 1e-5 {
                Quat::from_axis_angle(
                    (self.view.look - self.view.pos).normalize_or(Vec3::NEG_Z),
                    self.view_roll,
                ) * Vec3::Y
            } else {
                Vec3::Y
            },
            fovy: self.view.fov,
            aspect: Some(aspect),
            projection: Projection::Perspective,
            render_target: Some(rt),
            viewport: None,
            z_near: 0.1,
            z_far: 200.0,
        }
    }
}

/// 휘청 하나의 지금 굴림(도): 0에서 시작해 첫 반주기에 `amp` 쪽으로 기울었다(꼭대기 ≈ `amp`) `TURN_WOBBLE_HZ`로 흔들리며
/// `TURN_WOBBLE_DAMP`로 줄어든다 — 갑자기 기운 채로 시작하지 않게 사인(0에서 출발).
pub fn wobble_at(elapsed: f32, amp: f32) -> f32 {
    use crate::tuning::{TURN_WOBBLE_DAMP, TURN_WOBBLE_HZ};
    // 줄어드는 사인의 첫 꼭대기는 1보다 작다(3.5Hz·7/초에서 ≈ 0.62) — 그만큼 키워 꼭대기가 `amp`가 되게.
    const PEAK: f32 = 1.6;
    let x = std::f32::consts::TAU * TURN_WOBBLE_HZ * elapsed;
    PEAK * amp * (-TURN_WOBBLE_DAMP * elapsed).exp() * x.sin()
}

/// 화면 점(NDC: x 오른쪽·y 위, -1..1)을 지나는 시선의 방향(0058 커서 조준) — `to_camera3d`와 같은 원근(세로 `fov`, 가로는 `aspect`배).
/// 흔들림은 빼고 리그로만(겨누는 점이 흔들림을 따라 떨지 않게).
pub fn screen_ray(rig: &Rig, aspect: f32, ndc: Vec2) -> Vec3 {
    let f = (rig.look - rig.pos).normalize_or(Vec3::NEG_Z);
    let r = f.cross(Vec3::Y).normalize_or(Vec3::X);
    let u = r.cross(f);
    let k = (rig.fov * 0.5).tan();
    (f + r * (ndc.x * k * aspect) + u * (ndc.y * k)).normalize()
}

/// 월드 점 → 화면 NDC(`screen_ray`의 거꾸로, 0060 W1 길 안내). 카메라 뒤면 `false`와 함께, 방향만 맞게 뒤집은 값.
pub fn world_to_ndc(rig: &Rig, aspect: f32, p: Vec3) -> (Vec2, bool) {
    let f = (rig.look - rig.pos).normalize_or(Vec3::NEG_Z);
    let r = f.cross(Vec3::Y).normalize_or(Vec3::X);
    let u = r.cross(f);
    let k = (rig.fov * 0.5).tan();
    let d = p - rig.pos;
    let z = d.dot(f);
    let v = vec2(d.dot(r) / (k * aspect), d.dot(u) / k);
    if z > 1e-3 {
        (v / z, true)
    } else {
        // 뒤에 있으면 화면 밖 먼 곳으로 — 가장자리 화살이 그쪽을 가리키게(위아래는 가까운 바닥 쪽이 아래).
        (v.normalize_or(Vec2::NEG_Y) * 1e3, false)
    }
}

/// 화면 밖 표시 자리(0060 W1, nail 0159 `exit_at`과 같은 장치): 점이 화면 안(가장자리 `inset` 안쪽)이면 없음,
/// 밖이면 가운데에서 그 점으로 가는 선이 안쪽 테두리에 닿는 자리와 그 방향(라디안, NDC — x 오른쪽·y 위).
pub fn edge_marker(ndc: Vec2, in_front: bool, inset: f32) -> Option<(Vec2, f32)> {
    let lim = 1.0 - inset;
    if in_front && ndc.x.abs() <= lim && ndc.y.abs() <= lim {
        return None;
    }
    let d = ndc.normalize_or(Vec2::NEG_Y);
    let t = (lim / d.x.abs().max(1e-6)).min(lim / d.y.abs().max(1e-6));
    Some((d * t, d.y.atan2(d.x)))
}

/// 시선이 높이 `y`의 수평면에 닿는 점. 위로 가거나 평행이면 없음.
pub fn ground_hit(origin: Vec3, dir: Vec3, y: f32) -> Option<Vec3> {
    (dir.y < -1e-4)
        .then(|| (y - origin.y) / dir.y)
        .filter(|t| *t > 0.0)
        .map(|t| origin + dir * t)
}

#[cfg(test)]
mod ray_tests {
    use super::*;

    fn rig() -> Rig {
        Rig {
            pos: vec3(0.0, 10.0, 10.0),
            look: Vec3::ZERO,
            fov: 40f32.to_radians(),
        }
    }

    /// 화면 가운데는 카메라가 보는 점, 오른쪽은 +x, 위는 먼 쪽(-z) — 쿼터뷰의 화면 위 = 월드 -z(좌표 규칙).
    #[test]
    fn the_screen_centre_hits_the_look_point_and_up_is_farther() {
        let r = rig();
        let at = |n: Vec2| ground_hit(r.pos, screen_ray(&r, 16.0 / 9.0, n), 0.0).unwrap();
        assert!(at(Vec2::ZERO).length() < 1e-3);
        assert!(at(vec2(0.5, 0.0)).x > 1.0);
        assert!(at(vec2(0.0, 0.5)).z < -1.0);
        assert!(at(vec2(0.0, -0.5)).z > 1.0);
    }

    /// 0060 W1: 화면으로 다시 비추면 시선의 점으로 돌아온다. 화면 안이면 화살 없음, 밖이면 테두리 위 그쪽, 뒤에 있어도 그쪽.
    #[test]
    fn a_goal_off_screen_gets_an_edge_arrow_pointing_at_it() {
        let r = rig();
        let n = vec2(0.4, -0.3);
        let p = ground_hit(r.pos, screen_ray(&r, 1.5, n), 0.0).unwrap();
        let (back, front) = world_to_ndc(&r, 1.5, p);
        assert!(front && (back - n).length() < 1e-3, "{back:?}");
        assert_eq!(edge_marker(back, true, 0.1), None, "on screen: no arrow");
        let (far_right, _) = world_to_ndc(&r, 1.5, vec3(40.0, 0.0, 0.0));
        let (at, ang) = edge_marker(far_right, true, 0.1).unwrap();
        assert!((at.x - 0.9).abs() < 1e-3 && at.y.abs() <= 0.9, "{at:?}");
        assert!(ang.abs() < 0.6, "points right: {ang}");
        // 카메라 뒤(쿼터뷰 화면 아래 = 월드 +z 쪽 멀리)는 아래 가장자리.
        let (behind, front) = world_to_ndc(&r, 1.5, vec3(0.0, 0.0, 40.0));
        assert!(!front);
        let (at, _) = edge_marker(behind, front, 0.1).unwrap();
        assert!((at.y + 0.9).abs() < 1e-3, "{at:?}");
    }

    #[test]
    fn a_ray_above_the_horizon_hits_nothing() {
        let r = rig();
        assert!(ground_hit(r.pos, screen_ray(&r, 1.0, vec2(0.0, 1.0)), 0.0).is_some());
        assert!(ground_hit(r.pos, vec3(0.0, 0.2, -1.0).normalize(), 0.0).is_none());
    }
}

/// 바닥 면이 이 폭(m) 이하인 상자는 "가는 기둥" — 카메라를 밀지 않고 옅게 뚫고 지나간다.
pub const THIN: f32 = 2.5;

pub fn is_thin(b: &Aabb) -> bool {
    let s = b.size();
    s.x <= THIN && s.z <= THIN
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

    fn settle(c: &mut Camera, t: &Tuning, p: Vec3, target: Option<Vec3>) {
        for _ in 0..240 {
            c.update(1.0 / 60.0, t, p, target, &[]);
        }
    }

    /// 0181: 십자 ↑ = 당기기 — 쿼터뷰 거리 단계를 지나 가장 가까운 단계에서 한 번 더 당기면 자유 숄더뷰, ↓로 쿼터뷰에 돌아온다.
    #[test]
    fn zooming_in_steps_the_quarter_distance_then_enters_the_free_shoulder_view() {
        let t = Tuning::default();
        let p = Vec3::ZERO;
        let mut c = Camera::new(p, &t);
        settle(&mut c, &t, p, None);
        let far = c.rig.pos.distance(c.rig.look);
        assert!(c.step_zoom(1, p));
        settle(&mut c, &t, p, None);
        let near = c.rig.pos.distance(c.rig.look);
        assert!(near < far * 0.85, "{near} < {far}");
        assert_eq!(c.mode, Mode::Quarter);
        assert!(c.step_zoom(1, p));
        assert_eq!(c.mode, Mode::Free, "past the nearest step = shoulder view");
        settle(&mut c, &t, p, None);
        assert!(c.rig.pos.distance(c.rig.look) < near);
        assert!(!c.step_zoom(1, p), "nothing nearer than the shoulder");
        assert!(c.step_zoom(-1, p));
        assert_eq!((c.mode, c.zoom_step), (Mode::Quarter, 2));
        assert!(c.step_zoom(-1, p) && c.step_zoom(-1, p));
        assert_eq!(c.zoom_step, 0);
        assert!(!c.step_zoom(-1, p), "farthest step");
        // 락온 중에는 카메라를 대상이 정한다.
        c.set_mode(Mode::Lock, Some(vec3(0.0, 0.0, -5.0)), p);
        assert!(!c.step_zoom(1, p) && !c.step_zoom(-1, p));
    }

    /// 0181: 자유 숄더뷰로 들어가도 스틱의 위는 그대로(화면 위였던 쪽이 앞) — 들어가는 순간 걷던 쪽이 꺾이지 않는다.
    #[test]
    fn entering_the_free_view_keeps_stick_up_pointing_the_same_way() {
        let t = Tuning::default();
        let p = Vec3::ZERO;
        let mut c = Camera::new(p, &t);
        c.turn(1, &t);
        settle(&mut c, &t, p, None);
        let before = quarter_forward(c.basis_yaw_deg(&t));
        c.step_zoom(1, p);
        c.step_zoom(1, p);
        assert_eq!(c.mode, Mode::Free);
        let after = quarter_forward(c.basis_yaw_deg(&t));
        assert!(before.dot(after) > 0.99, "{before} vs {after}");
        // 그리고 화면도 그쪽을 본다(카메라 앞 = 스틱 위).
        settle(&mut c, &t, p, None);
        let view = (c.rig.look - c.rig.pos) * vec3(1.0, 0.0, 1.0);
        assert!(view.normalize().dot(after) > 0.95);
    }

    /// 0181: 자유 숄더뷰의 시점 돌리기 — + 방위는 오른쪽을 보고, + 높이는 위로 올라 내려다보며, 높이는 범위 안에 묶인다.
    /// 쿼터뷰·락온에서는 돌지 않는다. 땅 돌리기는 자유 숄더뷰에서도 앞을 90° 돌린다.
    #[test]
    fn free_look_turns_right_and_tilts_within_limits() {
        let t = Tuning::default();
        let p = Vec3::ZERO;
        let mut c = Camera::new(p, &t);
        let q = c.basis_yaw_deg(&t);
        c.look(1.0, 1.0);
        assert_eq!(c.basis_yaw_deg(&t), q, "quarter view does not free-look");
        c.step_zoom(1, p);
        c.step_zoom(1, p);
        settle(&mut c, &t, p, None);
        let fwd0 = quarter_forward(c.basis_yaw_deg(&t));
        let right0 = fwd0.cross(Vec3::Y);
        let h0 = c.rig.pos.y;
        c.look(0.6, 0.0);
        let fwd1 = quarter_forward(c.basis_yaw_deg(&t));
        assert!(fwd1.dot(right0) > 0.3, "turned toward the right");
        c.look(0.0, 0.4);
        settle(&mut c, &t, p, None);
        assert!(c.rig.pos.y > h0 + 0.5, "higher, looking down");
        c.look(0.0, 10.0);
        assert!(c.pitch <= FREE_PITCH_MAX);
        c.look(0.0, -10.0);
        assert!(c.pitch >= FREE_PITCH_MIN);
        // 땅 돌리기(0012)는 자유 숄더뷰에서 앞을 90°.
        let a = c.basis_yaw_deg(&t);
        c.turn(1, &t);
        let d = (c.basis_yaw_deg(&t) - a).rem_euclid(360.0);
        assert!((d - 90.0).abs() < 0.01, "{d}");
    }

    /// 0184: 시점 버튼은 쿼터뷰 ↔ 자유 숄더뷰를 오가고 거리 단계를 기억한다(먼 쿼터뷰에서 숄더뷰로 갔다 오면 다시 먼 쿼터뷰).
    /// 락온 중에는 아무 일도 없다.
    #[test]
    fn the_view_button_flips_between_quarter_and_shoulder_and_keeps_the_distance() {
        let t = Tuning::default();
        let p = Vec3::ZERO;
        let mut c = Camera::new(p, &t);
        c.step_zoom(-1, p);
        let far = c.zoom_step;
        assert!(c.toggle_view(p));
        assert_eq!(c.mode, Mode::Free);
        assert!(c.toggle_view(p));
        assert_eq!((c.mode, c.zoom_step), (Mode::Quarter, far));
        c.set_mode(Mode::Lock, Some(vec3(0.0, 0.0, -5.0)), p);
        assert!(!c.toggle_view(p));
        assert_eq!(c.mode, Mode::Lock);
    }

    /// 0181: 락온을 버튼으로 풀면 락온한 곳으로(쿼터뷰 → 쿼터뷰, 자유 → 자유), 대상을 잃으면 숄더뷰에 머문다.
    /// 새 카메라(쉼터·지역 전환)는 고른 거리와 자유 숄더뷰를 이어 간다.
    #[test]
    fn unlocking_returns_where_it_came_from_and_losing_the_target_stays_in_the_shoulder() {
        let t = Tuning::default();
        let p = Vec3::ZERO;
        let tg = Some(vec3(0.0, 0.0, -5.0));
        let mut c = Camera::new(p, &t);
        c.set_mode(Mode::Lock, tg, p);
        assert_eq!(c.unlock_mode(), Mode::Quarter);
        c.lose_lock();
        assert_eq!(
            c.unlock_mode(),
            Mode::Free,
            "lost target keeps the shoulder view"
        );
        c.set_mode(Mode::Free, None, p);
        c.set_mode(Mode::Lock, tg, p);
        assert_eq!(c.unlock_mode(), Mode::Free, "locked from the free view");
        // 이어 가기.
        c.set_mode(Mode::Free, None, p);
        c.zoom_step = 0;
        let n = c.renew(vec3(3.0, 0.0, 3.0), &t);
        assert_eq!((n.mode, n.zoom_step), (Mode::Free, 0));
        let mut q = Camera::new(p, &t);
        q.step_zoom(-1, p);
        let n = q.renew(p, &t);
        assert_eq!((n.mode, n.zoom_step), (Mode::Quarter, 0));
    }

    /// 0040: 바로 뒤에서 보면 가리고, 옆으로 비키면 안 가리고, 가까운 쪽이 더 멀면(대상 뒤) 안 가린다.
    #[test]
    fn occlusion_is_about_the_eye_line_and_order() {
        let eye = vec3(0.0, 2.0, 7.0);
        let boss = vec3(0.0, 1.5, -6.0);
        assert!(occlusion(eye, vec3(0.0, 1.6, 0.0), 0.5, boss, 1.5) > 0.6);
        assert_eq!(occlusion(eye, vec3(4.0, 1.6, 0.0), 0.5, boss, 1.5), 0.0);
        assert_eq!(occlusion(eye, vec3(0.0, 1.6, -9.0), 0.5, boss, 1.5), 0.0);
        let half = occlusion(eye, vec3(0.9, 1.6, 0.0), 0.5, boss, 1.5);
        assert!(half > 0.0 && half < 0.6, "partly: {half}");
    }

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
        let ticks = |c: &mut Camera, mode: Mode, tg: Option<Vec3>| {
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

    /// 재는 도구(0173): 대상 `r`m 앞에 서 있다가 한 틱에 뒤(반대편 `r`m)로 넘어갔을 때, 락온 카메라가 대상 쪽을 다시 볼 때까지의
    /// 초와 그동안의 가장 빠른 회전(°/s). `cargo test behind_probe -- --nocapture --ignored`
    fn behind(t: &Tuning, r: f32) -> (f32, f32) {
        let tg = Vec3::ZERO;
        let mut c = Camera::new(vec3(0.0, 0.0, r), t);
        c.set_mode(Mode::Lock, Some(tg), vec3(0.0, 0.0, r));
        for _ in 0..240 {
            c.update(1.0 / 60.0, t, vec3(0.0, 0.0, r), Some(tg), &[]);
        }
        let p = vec3(0.0, 0.0, -r);
        let (mut secs, mut peak) = (f32::MAX, 0f32);
        for i in 0..900 {
            let y0 = c.yaw;
            c.update(1.0 / 60.0, t, p, Some(tg), &[]);
            peak = peak.max(wrap(c.yaw - y0).abs().to_degrees() * 60.0);
            let want = (tg.x - p.x).atan2(tg.z - p.z);
            if wrap(want - c.yaw).abs().to_degrees() < 20.0 {
                secs = (i + 1) as f32 / 60.0;
                break;
            }
        }
        (secs, peak)
    }

    /// 대상 둘레 `r`m를 `v` m/s로 3초 돈 뒤의 (카메라 회전 °/s, 대상 방향과의 차 °).
    fn circle(t: &Tuning, r: f32, v: f32) -> (f32, f32) {
        let tg = Vec3::ZERO;
        let at = |a: f32| vec3(a.sin() * r, 0.0, a.cos() * r);
        let mut c = Camera::new(at(0.0), t);
        c.set_mode(Mode::Lock, Some(tg), at(0.0));
        for _ in 0..240 {
            c.update(1.0 / 60.0, t, at(0.0), Some(tg), &[]);
        }
        let (mut a, mut rate, mut err) = (0.0f32, 0.0, 0.0);
        for _ in 0..180 {
            a += v / r / 60.0;
            let y0 = c.yaw;
            c.update(1.0 / 60.0, t, at(a), Some(tg), &[]);
            rate = wrap(c.yaw - y0).abs().to_degrees() * 60.0;
            let p = at(a);
            err = wrap((tg.x - p.x).atan2(tg.z - p.z) - c.yaw)
                .abs()
                .to_degrees();
        }
        (rate, err)
    }

    /// 0173: 방랑자가 대상 뒤로 돌아가면(대상이 카메라와 방랑자 사이) 카메라가 곧 따라 돈다 — 예전엔 2m에서 4초였다.
    /// 곁에서 옆걸음하는 동안(차가 작다)은 예전의 조용한 상한 그대로.
    #[test]
    fn the_lock_camera_catches_up_when_the_wanderer_gets_behind_the_target() {
        let t = Tuning::default();
        for r in [1.2, 2.0, 3.0, 5.0] {
            let (secs, peak) = behind(&t, r);
            assert!(secs < 2.0, "{r}m: {secs}s");
            assert!(peak <= t[P::CamLCatch] + 1.0, "{r}m: {peak}°/s");
        }
        // 따라잡기를 끄면(0) 예전 그대로 느리다 — 이 시험이 재는 것이 그 값이라는 확인.
        let mut off = Tuning::default();
        off.bend(P::CamLCatch, 0.0);
        assert!(behind(&off, 2.0).0 > 3.0);
        // 멀찍이서 도는 동안은 차가 따라잡기 각에 못 미친다 — 상한은 조용한 쪽.
        let (rate, err) = circle(&t, 4.0, 4.0);
        assert!(
            err < t[P::CamLCatchAt] && rate <= t[P::CamLYawMax],
            "{rate} {err}"
        );
    }

    #[test]
    #[ignore]
    fn behind_probe() {
        let t = Tuning::default();
        for r in [1.2, 2.0, 3.0, 5.0] {
            let (s, p) = behind(&t, r);
            println!("behind r={r}: {s:.2}s peak {p:.0} deg/s");
        }
        for (r, v) in [(1.6, 4.0), (2.5, 4.0), (4.0, 4.0), (6.0, 4.0)] {
            let (rate, err) = circle(&t, r, v);
            println!(
                "circle r={r} v={v}: target {:.0} deg/s, cam {rate:.0} deg/s, err {err:.0}",
                (v / r).to_degrees()
            );
        }
    }

    /// 0173: 방랑자는 틱 사이를 보간해 그린다 — 카메라도 같은 `alpha`로 보간해야 걷는 동안 방랑자가 화면에서 떨리지 않는다.
    /// 120Hz 화면(틱 60Hz)을 흉내 낸다: 프레임이 틱의 한가운데와 끝에 번갈아 온다.
    #[test]
    fn a_walking_wanderer_does_not_judder_on_screen_between_ticks() {
        let t = Tuning::default();
        let dt = 1.0 / 60.0;
        let v = vec3(5.0, 0.0, 0.0);
        let mut c = Camera::new(Vec3::ZERO, &t);
        let mut pos = Vec3::ZERO;
        // 따라가기가 자리 잡을 때까지.
        for _ in 0..240 {
            pos += v * dt;
            c.update(dt, &t, pos, None, &[]);
        }
        let (mut shown, mut stepped) = (Vec::new(), Vec::new());
        for _ in 0..30 {
            let prev = pos;
            pos += v * dt;
            c.update(dt, &t, pos, None, &[]);
            for alpha in [0.5, 1.0] {
                c.present(alpha);
                let p = prev.lerp(pos, alpha);
                shown.push(ndc_x(&c.view, p));
                // 예전: 카메라는 이번 틱의 리그 그대로.
                stepped.push(ndc_x(&c.rig, p));
            }
        }
        let spread = |xs: &[f32]| {
            let (lo, hi) = xs
                .iter()
                .fold((f32::MAX, f32::MIN), |(l, h), &x| (l.min(x), h.max(x)));
            hi - lo
        };
        // 화면 폭 2(NDC) 가운데 — 보간하면 방랑자는 화면의 한 자리에 선다.
        assert!(spread(&shown) < 1e-3, "보간한 카메라: {}", spread(&shown));
        // 보간하지 않으면 반 틱어치(약 4cm)씩 앞뒤로 흔들린다 — 이 시험이 그 떨림을 본다.
        assert!(
            spread(&stepped) > 3.0 * spread(&shown).max(1e-4),
            "{}",
            spread(&stepped)
        );
    }

    /// 0173: 카메라가 안 움직인 틱(멈춤 창 아래)에는 그림이 지난 틱과 이번 틱 사이를 오가지 않는다.
    #[test]
    fn a_camera_that_did_not_move_this_tick_stands_still_whatever_the_alpha() {
        let t = Tuning::default();
        let mut c = Camera::new(Vec3::ZERO, &t);
        for i in 0..30 {
            c.update(1.0 / 60.0, &t, vec3(i as f32 * 0.1, 0.0, 0.0), None, &[]);
        }
        c.settle();
        c.present(0.2);
        let a = c.view;
        c.present(0.9);
        assert_eq!(a, c.view);
        assert_eq!(a, c.rig);
    }

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
        // 0050: 조금만 왼쪽(사장: "너무 왼쪽으로 치우쳐 있어") — 가운데 가까이, 그래도 과녁을 덜 가리게 왼쪽.
        assert!(
            px < -0.05 && px > -0.2,
            "player should sit a little left of center: {px}"
        );
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
                    // 쿼터뷰는 플레이어 속도(6)를 크게 넘지 않고 거의 돌지 않는다. 락온은 따라잡기 상한(0173 — 대상 곁을 돌아
                    // 뒤로 가면 `cam.lock.catch deg/s`까지 따라 돈다. 예전 상한은 90°/s × 1.4, 카메라 25 m/s)까지만.
                    let (vlim, wlim) = if lock {
                        (30.0, (t[P::CamLYawMax] * 1.4).max(t[P::CamLCatch] * 1.05))
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
        // 0040: 어깨를 비켜 둔 만큼(1.6m) 먼 쪽으로 도는 전환이 길어져 1.8배 → 2.1배(최악 절대값은 위 340°/s 그대로).
        // 0050: 높이 4.5·어깨 0.8에서 최악은 258 → 237°/s로 줄었는데 가장 순한 쪽이 124 → 98°/s로 더 순해져 배수만 2.4가 됐다 —
        // 배수 2.5 + 최악 260°/s(0040 때 값)로 본다.
        assert!(worst < 260.0, "turn max {worst} (0040: 258)");
        assert!(worst < best * 2.5, "side matters too much: {best}..{worst}");
        assert!(
            rows.iter().all(|r| r.3 > 4.0),
            "camera grazes the head: {rows:?}"
        );
    }
}

#[cfg(test)]
mod yaw_mode_probe {
    //! 0011: 쿼터뷰 방위 방식(고정 / 락온 방향 따라감 / 90° 단위)을 보스 둘레 8방향에서 비교.
    //! `cargo test yaw_mode_probe -- --nocapture`로 표를 본다. 락온 → 해제 → 다시 락온 → 해제(두 번째 들어감은 돌아간 쿼터뷰에서).
    use super::*;
    use crate::input::Actions;
    use crate::scenario::{build, Scenario};

    #[derive(Default, Debug, Clone, Copy)]
    struct Row {
        enter: f32,
        exit: f32,
        exit_secs: f32,
        enter2: f32,
    }

    fn one(mode: f32, deg: f32) -> Row {
        let mut t = Tuning::default();
        let i = crate::tuning::SPECS
            .iter()
            .position(|s| s.p == P::CamQYawMode)
            .unwrap();
        t.set(i, mode);
        let a = deg.to_radians();
        let mut w = build(Scenario::Boss);
        let boss = w.enemies[0].pos;
        w.player.pos = boss + vec3(a.sin(), 0.0, a.cos()) * 6.0;
        w.player.prev = w.player.pos;
        w.enemies[0].state = crate::world::EState::Recover { t: -1000.0 };
        let mut c = Camera::new(w.player.pos, &t);
        for _ in 0..60 {
            c.update(1.0 / 60.0, &t, w.player.pos, None, &w.obstacles);
        }
        let mut row = Row::default();
        let mut prev = c.rig;
        for tick in 0..300 {
            let lock = tick % 75 == 0;
            w.view_yaw = Some(c.quarter_yaw_deg(&t));
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
            let turn = d0.dot(d1).clamp(-1.0, 1.0).acos().to_degrees() * 60.0;
            match tick / 75 {
                0 => row.enter = row.enter.max(turn),
                1 => {
                    row.exit = row.exit.max(turn);
                    if c.blending().is_some() {
                        row.exit_secs += 1.0 / 60.0;
                    }
                }
                2 => row.enter2 = row.enter2.max(turn),
                _ => {}
            }
            prev = c.rig;
        }
        row
    }

    fn table(mode: f32) -> Vec<Row> {
        (0..8).map(|k| one(mode, k as f32 * 45.0)).collect()
    }

    #[test]
    fn snap_and_follow_turn_less_on_release_than_fixed() {
        let names = ["fixed", "follow", "snap90"];
        let mut worst_exit = [0.0f32; 3];
        for (m, name) in names.iter().enumerate() {
            let rows = table(m as f32);
            let max = |f: fn(&Row) -> f32| rows.iter().map(f).fold(0.0, f32::max);
            worst_exit[m] = max(|r| r.exit);
            println!(
                "{name:>7}: enter max {:5.0}°/s | exit max {:5.0}°/s, exit takes up to {:.2}s | 2nd enter max {:5.0}°/s",
                max(|r| r.enter),
                max(|r| r.exit),
                max(|r| r.exit_secs),
                max(|r| r.enter2)
            );
        }
        assert!(
            worst_exit[2] < worst_exit[0] * 0.8,
            "snap90 {:?} vs fixed",
            worst_exit
        );
        assert!(
            worst_exit[1] <= worst_exit[2] + 1.0,
            "follow {:?}",
            worst_exit
        );
    }
}

#[cfg(test)]
mod turn_probe {
    //! 0012: 땅 돌리기 카메라. `cargo test turn_probe -- --nocapture`로 회전 시간별 표를 본다.
    use super::*;
    use crate::input::Actions;
    use crate::scenario::{build, Scenario};

    /// 0084(사장 2026-11-07): 땅 돌리기는 시작에 도는 쪽으로, 끝에 반대쪽으로 작게 휘청인다. 휘청 하나는 꼭대기가 설정값,
    /// 0.5초 안에 거의 사라진다.
    #[test]
    fn a_turn_wobbles_into_the_turn_at_the_start_and_back_at_the_end() {
        let peak = (0..60)
            .map(|i| wobble_at(i as f32 / 120.0, 1.5))
            .fold(0.0f32, f32::max);
        assert!((peak - 1.5).abs() < 0.15, "peak {peak}");
        assert!(wobble_at(0.0, 1.5).abs() < 1e-6, "starts level");
        assert!(
            wobble_at(0.5, 1.5).abs() < 0.1,
            "almost gone in half a second"
        );
        let t = Tuning::default();
        let mut c = Camera::new(Vec3::ZERO, &t);
        let step = |c: &mut Camera| c.update(1.0 / 60.0, &t, Vec3::ZERO, None, &[]);
        for _ in 0..30 {
            step(&mut c);
        }
        c.turn(1, &t);
        let (go, settle) = crate::tuning::turn_phases_q(&t, 1);
        let mut early = 0.0f32;
        let mut late = 0.0f32;
        let mut e = 0.0;
        while e < go + settle + 0.5 {
            step(&mut c);
            e += 1.0 / 60.0;
            if e < 0.2 {
                early = if early.abs() > c.roll_deg().abs() {
                    early
                } else {
                    c.roll_deg()
                };
            } else if e > go + settle && e < go + settle + 0.2 {
                late = if late.abs() > c.roll_deg().abs() {
                    late
                } else {
                    c.roll_deg()
                };
            }
        }
        assert!(early > 0.5, "leans into the turn first: {early}");
        assert!(late < -0.5, "and back the other way when it stops: {late}");
    }

    /// 오른쪽 버튼(0017) = `turn(true)`: 카메라가 자기 오른쪽으로 돈다(카메라 자리가 돌기 전 화면 오른쪽 쪽으로 간다).
    /// 180°(0023, 오른쪽을 보고 오른쪽 버튼)는 정한 쪽으로 끝까지 돌고 반대로 튀지 않는다 — "짧은 쪽"이 매 프레임 뒤집힐 수 있다.
    #[test]
    fn a_half_turn_goes_the_chosen_way_and_ends_half_round() {
        let t = Tuning::default();
        for q in [2, -2] {
            let mut c = Camera::new(Vec3::ZERO, &t);
            for _ in 0..30 {
                c.update(1.0 / 60.0, &t, Vec3::ZERO, None, &[]);
            }
            let azi = |c: &Camera| {
                let d = c.rig.pos - c.rig.look;
                d.x.atan2(d.z)
            };
            let start = azi(&c);
            c.turn(q, &t);
            let (mut total, mut prev, mut against) = (0.0f32, start, 0.0f32);
            for _ in 0..240 {
                c.update(1.0 / 60.0, &t, Vec3::ZERO, None, &[]);
                let step = wrap(azi(&c) - prev);
                prev = azi(&c);
                total += step;
                if step.signum() != q.signum() as f32 {
                    against += step.abs();
                }
            }
            assert!(
                (total.abs().to_degrees() - 180.0).abs() < 2.0
                    && total.signum() == q.signum() as f32,
                "q {q}: turned {}°",
                total.to_degrees()
            );
            // 반대쪽으로 간 것은 지나쳤다 돌아오는 몫(`turn.overshoot deg`)뿐.
            assert!(
                against.to_degrees() < t[P::TurnOver] + 2.0,
                "q {q}: back {}°",
                against.to_degrees()
            );
        }
    }

    #[test]
    fn a_clockwise_turn_swings_the_camera_to_its_right() {
        let t = Tuning::default();
        let mut c = Camera::new(Vec3::ZERO, &t);
        for _ in 0..30 {
            c.update(1.0 / 60.0, &t, Vec3::ZERO, None, &[]);
        }
        let before = c.rig.pos;
        let right = quarter_forward(c.quarter_yaw_deg(&t)).cross(Vec3::Y);
        c.turn(1, &t);
        for _ in 0..10 {
            c.update(1.0 / 60.0, &t, Vec3::ZERO, None, &[]);
        }
        assert!(
            (c.rig.pos - before).dot(right) > 0.5,
            "{:?} -> {:?}",
            before,
            c.rig.pos
        );
    }

    /// 왼쪽 버튼 = `turn(false)`: 플레이어 앞(화면 아래, 가까운 쪽)에 있던 땅이 화면 오른쪽으로 간다(0015 때의 뜻, 시험은 남긴다).
    #[test]
    fn a_counter_clockwise_turn_moves_the_near_side_of_the_ground_to_the_right() {
        let t = Tuning::default();
        let mut c = Camera::new(Vec3::ZERO, &t);
        let before = c.quarter_yaw_deg(&t);
        let near = -quarter_forward(before); // 화면 아래 쪽 땅
        c.turn(-1, &t);
        let right = quarter_forward(c.quarter_yaw_deg(&t)).cross(Vec3::Y);
        assert!(
            near.dot(right) > 0.999,
            "{before} -> {}",
            c.quarter_yaw_deg(&t)
        );
    }

    #[test]
    fn a_clockwise_turn_puts_what_was_screen_up_on_screen_right() {
        let t = Tuning::default();
        let mut c = Camera::new(Vec3::ZERO, &t);
        let before = c.quarter_yaw_deg(&t);
        c.turn(1, &t);
        let after = c.quarter_yaw_deg(&t);
        let right = quarter_forward(after).cross(Vec3::Y);
        assert!(
            right.dot(quarter_forward(before)) > 0.999,
            "{before} -> {after}"
        );
        c.turn(-1, &t);
        assert!((wrap((c.quarter_yaw_deg(&t) - before).to_radians())).abs() < 1e-4);
    }

    /// 두 리그 사이 카메라 회전을 카메라 축으로 나눈 것(°/s): (팬 = 위축 둘레, 기울기 = 오른쪽 축 둘레, 롤 = 시선축 둘레).
    /// 롤은 화면이 제자리에서 도는 것 — 수평축 회전이라 멀미를 더 부른다(KB 카메라 노트). 시선 회전(°/s)은 롤을 못 본다.
    pub fn axis_rates(a: &Rig, b: &Rig, dt: f32) -> (f32, f32, f32) {
        let basis = |r: &Rig| {
            let f = (r.look - r.pos).normalize();
            let x = f.cross(Vec3::Y).normalize();
            let u = x.cross(f);
            Mat3::from_cols(x, u, -f)
        };
        let (m0, m1) = (basis(a), basis(b));
        let q = Quat::from_mat3(&(m1 * m0.transpose()));
        let (axis, ang) = q.to_axis_angle();
        let w = axis * (ang.to_degrees() / dt);
        // 카메라 축(m0의 열)으로 나눈다.
        (
            w.dot(m0.y_axis).abs(),
            w.dot(m0.x_axis).abs(),
            w.dot(m0.z_axis).abs(),
        )
    }

    struct Probe {
        /// 가장 빠른 시선 회전(°/s).
        peak: f32,
        /// 가장 빠른 롤(화면이 제자리에서 도는 것, °/s)과 팬(옆으로 훑기, °/s).
        roll: f32,
        pan: f32,
        /// 돌기 전 시선이 실제로 숙인 각(라디안) — 카메라는 발이 아니라 몸통을 봐서 랩의 지상각보다 조금 얕고, 가까울수록 더 얕다.
        dip: f32,
        /// 되돌아올 때 반대 방향으로 가장 빠른 방위 회전(°/s).
        back: f32,
        /// 가장 멀리 간 방위(°, 90이면 지나침 없음).
        reach: f32,
        /// 다 도는 데 걸린 초.
        secs: f32,
        /// 카메라-주시점 거리 흔들림(m).
        wobble: f32,
    }

    fn one(turn_time: f32, over: f32, settle: f32) -> Probe {
        one_at(Tuning::default()[P::CamQPitch], turn_time, over, settle)
    }

    fn one_at(pitch: f32, turn_time: f32, over: f32, settle: f32) -> Probe {
        let mut t = Tuning::default();
        for (p, v) in [
            (P::CamQPitch, pitch),
            (P::TurnTime, turn_time),
            (P::TurnOver, over),
            (P::TurnSettle, settle),
        ] {
            let i = crate::tuning::SPECS.iter().position(|s| s.p == p).unwrap();
            t.set(i, v);
        }
        let mut w = build(Scenario::Tray);
        let mut c = Camera::new(w.player.pos, &t);
        let mut prev = c.rig;
        let azi = |r: &Rig| {
            let d = r.pos - r.look;
            d.x.atan2(d.z)
        };
        let start = azi(&c.rig);
        let mut pr = Probe {
            peak: 0.0,
            roll: 0.0,
            pan: 0.0,
            dip: (-(c.rig.look - c.rig.pos).normalize().y).asin(),
            back: 0.0,
            reach: 0.0,
            secs: 0.0,
            wobble: 0.0,
        };
        let (mut dmin, mut dmax) = (f32::MAX, 0.0f32);
        for tick in 0..120 {
            w.view_yaw = Some(c.quarter_yaw_deg(&t));
            // 기본 자세(화면 위를 봄)에서 왼쪽 버튼 = 화면 왼쪽이 정면 = 방위 +90°(0023).
            for e in w.step(
                &Actions {
                    turn_left: tick == 10,
                    ..Default::default()
                },
                &t,
            ) {
                if let crate::world::Event::Turn { quarters } = e {
                    c.turn(quarters, &t);
                }
            }
            c.update(1.0 / 60.0, &t, w.player.pos, None, &w.obstacles);
            let d0 = (prev.look - prev.pos).normalize();
            let d1 = (c.rig.look - c.rig.pos).normalize();
            pr.peak = pr
                .peak
                .max(d0.dot(d1).clamp(-1.0, 1.0).acos().to_degrees() * 60.0);
            let (pan, _, roll) = axis_rates(&prev, &c.rig, 1.0 / 60.0);
            pr.pan = pr.pan.max(pan);
            pr.roll = pr.roll.max(roll);
            // ↻ = 방위 +: 반대(−)로 도는 빠르기가 되돌아오기.
            let step = wrap(azi(&c.rig) - azi(&prev)).to_degrees() * 60.0;
            pr.back = pr.back.max(-step);
            pr.reach = pr.reach.max(wrap(azi(&c.rig) - start).to_degrees());
            if c.blending().is_some() {
                pr.secs += 1.0 / 60.0;
            }
            if tick > 10 {
                let d = c.rig.pos.distance(c.rig.look);
                dmin = dmin.min(d);
                dmax = dmax.max(d);
            }
            prev = c.rig;
        }
        pr.wobble = dmax - dmin;
        pr
    }

    #[test]
    fn the_turn_is_no_faster_than_the_lock_switch_and_keeps_its_distance() {
        let d = Tuning::default();
        for tt in [0.25, 0.3, 0.4, 0.5, 0.6] {
            let p = one(tt, 0.0, d[P::TurnSettle]);
            println!(
                "turn {tt:.2}s: peak {:5.0}°/s, takes {:.2}s, distance wobble {:.3} m",
                p.peak, p.secs, p.wobble
            );
        }
        for (over, settle) in [
            (5.0, 0.2),
            (10.0, 0.2),
            (10.0, 0.3),
            (15.0, 0.25),
            (20.0, 0.3),
        ] {
            let p = one(d[P::TurnTime], over, settle);
            println!(
                "overshoot {over:4.0}° settle {settle:.2}s: peak {:5.0}°/s, reaches {:5.1}°, back {:4.0}°/s, takes {:.2}s",
                p.peak, p.reach, p.back, p.secs
            );
        }
        let p = one(d[P::TurnTime], d[P::TurnOver], d[P::TurnSettle]);
        // 기준: 락온 전환 최악 260°/s(0009). 플레이어가 누른 회전이라 예측되지만, 그보다 빠르지는 않게.
        assert!(p.peak < 260.0, "peak {}", p.peak);
        assert!(p.wobble < 0.05, "orbit keeps the distance: {}", p.wobble);
        assert!(
            (p.reach - 90.0 - d[P::TurnOver]).abs() < 1.5,
            "reaches {}",
            p.reach
        );
    }

    /// 0145 타격 줌: 센 타격의 순간 카메라가 주시점 쪽으로 거리의 몫만큼 다가가고, 세계가 멈춘 동안(히트스톱)은 그대로 머물다
    /// 풀리면 0.5초 안에 제자리로 돌아온다. 겹치면 더 큰 쪽 — 한도(`ZOOM_MAX`)를 넘지 않는다.
    #[test]
    fn a_heavy_blow_pulls_the_camera_in_holds_through_hitstop_and_lets_go() {
        use crate::tuning::{ZOOM_MAX, ZOOM_SMASH};
        let t = Tuning::default();
        let w = build(Scenario::Tray);
        let mut c = Camera::new(w.player.pos, &t);
        let step = |c: &mut Camera| c.update(1.0 / 60.0, &t, w.player.pos, None, &w.obstacles);
        for _ in 0..30 {
            step(&mut c);
        }
        let far = c.rig.pos.distance(c.rig.look);
        c.punch_in(ZOOM_SMASH);
        c.freeze = true;
        for _ in 0..8 {
            step(&mut c);
        }
        let near = c.rig.pos.distance(c.rig.look);
        assert!(
            (near / far - (1.0 - ZOOM_SMASH)).abs() < 1e-3,
            "{near} of {far}"
        );
        c.freeze = false;
        step(&mut c);
        assert!(
            c.rig.pos.distance(c.rig.look) > near,
            "eases out once the world moves"
        );
        for _ in 0..30 {
            step(&mut c);
        }
        assert!((c.rig.pos.distance(c.rig.look) - far).abs() < 0.02);
        // 겹쳐도 한도까지.
        for _ in 0..5 {
            c.punch_in(ZOOM_MAX);
        }
        c.freeze = true;
        step(&mut c);
        assert!((c.rig.pos.distance(c.rig.look) / far - (1.0 - ZOOM_MAX)).abs() < 1e-3);
    }

    /// 지상각을 낮추면 롤(화면이 제자리에서 도는 것)이 줄고 팬이 는다 — 45°에서 같아진다(롤 = 방위 회전 × sin 지상각).
    #[test]
    fn a_lower_pitch_turns_roll_into_pan() {
        let d = Tuning::default();
        let mut last_roll = f32::MAX;
        for pitch in [70.0, 60.0, 55.0, 50.0, 45.0, 44.0, 40.0, 30.0] {
            let p = one_at(pitch, d[P::TurnTime], d[P::TurnOver], d[P::TurnSettle]);
            println!(
                "pitch {pitch:4.0}°: roll {:5.0}°/s, pan {:5.0}°/s (look {:5.0}°/s)",
                p.roll, p.pan, p.peak
            );
            assert!(p.roll < last_roll, "roll shrinks as the pitch drops");
            last_roll = p.roll;
            // 견주는 각은 랩의 지상각이 아니라 시선이 실제로 숙인 각(0144: 거리를 20 → 17로 줄이자 둘의 차가 커져
            // 지상각 70°에서 허용 폭을 넘었다 — 식이 틀린 것이 아니라 견준 각이 달랐다).
            assert!(p.dip <= f32::to_radians(pitch) + 1e-3 && p.dip > f32::to_radians(pitch - 4.0));
            assert!(
                (p.roll / p.pan - p.dip.tan()).abs() < 0.08,
                "roll/pan = tan(dip): {} vs {}",
                p.roll / p.pan,
                p.dip.tan()
            );
        }
        // 기본 지상각(44°)에서는 롤이 팬보다 작다.
        let p = one(d[P::TurnTime], d[P::TurnOver], d[P::TurnSettle]);
        assert!(p.roll < p.pan, "roll {} pan {}", p.roll, p.pan);
    }

    #[test]
    fn the_default_turn_lands_with_a_kick() {
        let d = Tuning::default();
        let p = one(d[P::TurnTime], d[P::TurnOver], d[P::TurnSettle]);
        println!(
            "default: reaches {:.1}°, back {:.0}°/s, roll {:.0}°/s, pan {:.0}°/s, takes {:.2}s",
            p.reach, p.back, p.roll, p.pan, p.secs
        );
        let mut c = Camera::new(Vec3::ZERO, &d);
        c.turn(1, &d);
        let mut kicked = 0.0f32;
        for _ in 0..120 {
            let before = c.trauma;
            c.update(1.0 / 60.0, &d, Vec3::ZERO, None, &[]);
            kicked = kicked.max(c.trauma - before);
        }
        assert!(kicked > d[P::TurnSnap] * 0.8, "lands with a kick: {kicked}");
    }

    /// 카메라가 90°에 닿는 "탁"과 월드가 적을 쏟는 틱이 같다(±1틱) — 앱과 같은 순서로 돌려 본다.
    #[test]
    fn the_enemies_spill_on_the_same_tick_the_camera_lands() {
        let t = Tuning::default();
        let mut w = build(Scenario::Tray);
        let mut c = Camera::new(w.player.pos, &t);
        let (mut kick_tick, mut spill_tick) = (None, None);
        for tick in 0..200u32 {
            w.view_yaw = Some(c.quarter_yaw_deg(&t));
            for e in w.step(
                &Actions {
                    turn_right: tick == 5,
                    ..Default::default()
                },
                &t,
            ) {
                if let crate::world::Event::Turn { quarters } = e {
                    c.turn(quarters, &t);
                }
            }
            if w.player.since_spill == 0.0 && spill_tick.is_none() {
                spill_tick = Some(tick);
            }
            let before = c.trauma;
            c.update(1.0 / 60.0, &t, w.player.pos, None, &w.obstacles);
            if c.trauma > before + 0.1 && kick_tick.is_none() {
                kick_tick = Some(tick);
            }
        }
        let (k, s) = (kick_tick.unwrap(), spill_tick.unwrap());
        assert!(
            k.abs_diff(s) <= 1,
            "camera lands at tick {k}, enemies spill at {s}"
        );
    }

    #[test]
    fn an_overshooting_turn_goes_past_ninety_comes_back_gently_and_ends_on_ninety() {
        let p = one(Tuning::default()[P::TurnTime], 10.0, 0.25);
        assert!(p.reach > 99.0 && p.reach < 101.0, "{}", p.reach);
        assert!(
            p.back > 10.0 && p.back < 100.0,
            "comes back, slowly: {}",
            p.back
        );
        assert!(p.peak < 260.0);
        let mut t = Tuning::default();
        let i = crate::tuning::SPECS
            .iter()
            .position(|s| s.p == P::TurnOver)
            .unwrap();
        t.set(i, 10.0);
        let mut c = Camera::new(Vec3::ZERO, &t);
        let before = c.quarter_yaw_deg(&t);
        c.turn(1, &t);
        for _ in 0..90 {
            c.update(1.0 / 60.0, &t, Vec3::ZERO, None, &[]);
        }
        assert!(c.blending().is_none());
        let d = c.rig.pos - c.rig.look;
        let end = d.x.atan2(d.z).to_degrees();
        assert!(
            (wrap((end - before - 90.0).to_radians())).abs() < 0.01,
            "{before} -> {end}"
        );
    }
}
