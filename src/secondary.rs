//! 2차 동작 층(0039): 키프레임 동작 위에 얹는 절차 움직임 — 동작 표는 그대로 두고, 렌더가 그릴 때만 더한다.
//! 월드·로직은 모른다(프레임 시간으로 돈다, 그림만 바뀐다).
//!
//! 원칙(KB `concepts/believable-cute-character-motion.md`): 무거운·부드러운 부분은 늦게 따라오고 더 천천히 멈춘다(따라가기·겹치기),
//! 몸은 가속하는 쪽으로 기운다, 늘어남·찌그러짐은 부피를 지킨다(Lasseter 1987). 플레이어 입력의 반응 시간은 건드리지 않는다.
//!
//! - 꼬리·머리: 그 덩어리 자리를 감쇠 스프링으로 따라가는 점이 늦는 만큼(모델 공간) 덩어리 쪽으로 돌린다.
//! - 몸: 같은 늦음의 반대쪽으로 기운다(가속하는 쪽).
//! - 뿌리(발): 공중에서는 세로 속도만큼 늘어나고, 내려앉으면 찌그러졌다 한 번 튀며 선다.
//!
//! 스프링은 암시적 오일러(Allen Chou 2015) — 프레임 시간이 튀어도 터지지 않는다.
use macroquad::prelude::*;

use crate::model::Model;
use crate::tuning::{
    Tuning, P, SEC_HEAD_DAMP, SEC_HEAD_HZ, SEC_LAG_MAX, SEC_LEAN_DAMP, SEC_LEAN_HZ, SEC_LEAN_K,
    SEC_SQUASH_DAMP, SEC_SQUASH_HZ, SEC_SQUASH_IMPULSE, SEC_SQUASH_MAX, SEC_STRETCH_PER_SPEED,
};

/// 1차원 감쇠 스프링(위치 x, 속도 v). f = 진동수(Hz), zeta = 감쇠비(1 = 임계, 작을수록 출렁).
#[derive(Clone, Copy, Debug, Default)]
pub struct Spring {
    pub x: f32,
    pub v: f32,
}

impl Spring {
    pub fn step(&mut self, target: f32, f: f32, zeta: f32, h: f32) {
        let w = std::f32::consts::TAU * f;
        let a = 1.0 + 2.0 * h * zeta * w;
        let d = a + h * h * w * w;
        let x = (a * self.x + h * self.v + h * h * w * w * target) / d;
        let v = (self.v + h * w * w * (target - self.x)) / d;
        self.x = x;
        self.v = v;
    }
}

/// 3차원 따라가는 점: 기준점을 스프링으로 쫓는다. 돌려주는 것 = 늦음(점 − 기준점).
#[derive(Clone, Copy, Debug, Default)]
pub struct Follow {
    s: [Spring; 3],
    ready: bool,
}

impl Follow {
    pub fn step(&mut self, anchor: Vec3, f: f32, zeta: f32, h: f32) -> Vec3 {
        let a = anchor.to_array();
        // 처음이거나 순간 이동(되살아남·장면 바뀜)이면 그 자리에서 다시.
        let far = self.s.iter().zip(a).any(|(s, x)| (s.x - x).abs() > 2.0);
        if !self.ready || far {
            for (s, x) in self.s.iter_mut().zip(a) {
                *s = Spring { x, v: 0.0 };
            }
            self.ready = true;
        }
        for (s, x) in self.s.iter_mut().zip(a) {
            s.step(x, f, zeta, h);
        }
        Vec3::new(self.s[0].x, self.s[1].x, self.s[2].x) - anchor
    }
}

/// 2차 동작을 줄 부위: (노드, 피벗, 덩어리 가운데) — 쉬는 자세의 모델 공간.
#[derive(Clone, Copy, Debug)]
pub struct Part {
    pub node: usize,
    pub pivot: Vec3,
    pub mass: Vec3,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Rig {
    pub tail: Option<Part>,
    pub head: Option<Part>,
    pub body: Option<Part>,
    pub root: Option<usize>,
}

impl Rig {
    /// 뼈 이름으로 찾는다(없는 부위는 건너뜀). 덩어리 = 그 뼈 아래 메시 정점의 가운데.
    pub fn of(m: &Model) -> Rig {
        let mut rest = Vec::new();
        m.pose(None, 0.0, &mut rest);
        let part = |name: &str| {
            let node = m.node(name)?;
            Some(Part {
                node,
                pivot: rest[node].w_axis.truncate(),
                mass: m.rest_centroid_under(node)?,
            })
        };
        Rig {
            tail: part("tail"),
            head: part("head"),
            body: part("body"),
            root: m.node("root"),
        }
    }
}

/// 한 캐릭터의 2차 동작 상태(렌더가 들고 있다).
#[derive(Clone, Debug, Default)]
pub struct Secondary {
    tail: Follow,
    head: Follow,
    lean: Follow,
    squash: Spring,
    was_air: bool,
    vy_air: f32,
}

/// 모델 공간에서 덩어리(피벗 → mass)가 `lag`만큼 밀린 쪽으로 도는 회전. 밀림은 덩어리 거리의 `SEC_LAG_MAX`배까지.
fn swing(p: &Part, lag: Vec3) -> Quat {
    let r = p.mass - p.pivot;
    let len = r.length();
    if len < 1e-4 {
        return Quat::IDENTITY;
    }
    let lag = lag.clamp_length_max(len * SEC_LAG_MAX);
    Quat::from_rotation_arc(r / len, (r + lag).normalize_or(r / len))
}

impl Secondary {
    /// 프레임마다. `pos` = 발 자리(그릴 때 보간한 월드), `facing` = 모델을 돌리는 요(라디안), `air`·`vy` = 높이·세로 속도.
    /// 돌려주는 것 = `Model::pose_with`의 덧붙이기.
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        rig: &Rig,
        dt: f32,
        pos: Vec3,
        facing: f32,
        air: f32,
        vy: f32,
        t: &Tuning,
        out: &mut Vec<(usize, Quat, Vec3)>,
    ) {
        out.clear();
        let h = dt.clamp(0.0, 0.05);
        let rot = Quat::from_rotation_y(facing);
        let inv = rot.inverse();
        let base = pos + Vec3::Y * air;
        // 모델 공간 점 → 월드, 월드 늦음 → 모델 공간.
        let world = |p: Vec3| base + rot * p;
        if let Some(p) = rig.tail {
            let lag = inv
                * self
                    .tail
                    .step(world(p.mass), t[P::AnimTailHz], t[P::AnimTailDamp], h);
            out.push((p.node, swing(&p, lag * t[P::AnimTailGain]), Vec3::ONE));
        }
        if let Some(p) = rig.head {
            let lag = inv * self.head.step(world(p.mass), SEC_HEAD_HZ, SEC_HEAD_DAMP, h);
            out.push((p.node, swing(&p, lag * t[P::AnimHeadGain]), Vec3::ONE));
        }
        if let Some(p) = rig.body {
            let lag = inv * self.lean.step(world(p.mass), SEC_LEAN_HZ, SEC_LEAN_DAMP, h);
            // 세로 늦음은 기울기가 아니다(찌그러짐이 맡는다).
            let lag = Vec3::new(lag.x, 0.0, lag.z);
            out.push((
                p.node,
                swing(&p, -lag * SEC_LEAN_K * t[P::AnimLeanGain]),
                Vec3::ONE,
            ));
        }
        // 늘어남·찌그러짐: 공중에서는 세로 속도만큼 늘어나고, 내려앉는 순간 그 속도로 찌그러짐을 친다.
        let airborne = air > 0.01;
        if airborne {
            self.vy_air = vy;
        } else if self.was_air {
            self.squash.v -= SEC_SQUASH_IMPULSE * self.vy_air.abs();
        }
        self.was_air = airborne;
        let target = if airborne {
            SEC_STRETCH_PER_SPEED * vy.abs()
        } else {
            0.0
        };
        self.squash.step(target, SEC_SQUASH_HZ, SEC_SQUASH_DAMP, h);
        if let Some(n) = rig.root {
            let k = 1.0 + (self.squash.x * t[P::AnimSquash]).clamp(-SEC_SQUASH_MAX, SEC_SQUASH_MAX);
            let side = 1.0 / k.sqrt();
            out.push((n, Quat::IDENTITY, Vec3::new(side, k, side)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(s: &mut Spring, target: f32, f: f32, z: f32, secs: f32, h: f32) -> (f32, f32) {
        let mut peak = f32::MIN;
        for _ in 0..(secs / h) as usize {
            s.step(target, f, z, h);
            peak = peak.max(s.x);
        }
        (s.x, peak)
    }

    #[test]
    fn spring_settles_on_target_and_only_underdamped_overshoots() {
        let (end, peak) = run(&mut Spring::default(), 1.0, 3.0, 1.0, 3.0, 1.0 / 60.0);
        assert!(
            (end - 1.0).abs() < 1e-3 && peak <= 1.0 + 1e-4,
            "critical: end {end} peak {peak}"
        );
        let (end, peak) = run(&mut Spring::default(), 1.0, 3.0, 0.3, 4.0, 1.0 / 60.0);
        assert!(
            (end - 1.0).abs() < 1e-2 && peak > 1.2,
            "bouncy: end {end} peak {peak}"
        );
    }

    #[test]
    fn spring_stays_stable_when_a_frame_takes_forever() {
        // 탭을 돌아오면 프레임 시간이 1초씩 튄다 — 암시적 오일러는 그래도 목표를 넘어 터지지 않는다.
        let mut s = Spring::default();
        for _ in 0..20 {
            s.step(1.0, 8.0, 0.1, 1.0);
            assert!(s.x.is_finite() && s.x.abs() < 2.0, "{s:?}");
        }
    }

    fn rig() -> Rig {
        // 모델 공간: +Z 앞, +X 캐릭터 왼쪽, +Y 위. 꼬리는 뒤·왼쪽 위, 머리는 위, 몸은 허리 위.
        Rig {
            tail: Some(Part {
                node: 1,
                pivot: vec3(0.1, 0.24, -0.16),
                mass: vec3(0.35, 0.38, -0.25),
            }),
            head: Some(Part {
                node: 2,
                pivot: vec3(0.0, 0.62, 0.0),
                mass: vec3(0.0, 0.95, 0.0),
            }),
            body: Some(Part {
                node: 3,
                pivot: vec3(0.0, 0.13, 0.0),
                mass: vec3(0.0, 0.4, 0.0),
            }),
            root: Some(0),
        }
    }

    fn rotated_mass(out: &[(usize, Quat, Vec3)], p: &Part) -> Vec3 {
        let q = out.iter().find(|e| e.0 == p.node).unwrap().1;
        p.pivot + q * (p.mass - p.pivot)
    }

    /// 앞(+Z)으로 갑자기 달리기 시작하면 꼬리·머리는 뒤로 처지고 몸은 앞으로 숙는다. 멈추면 꼬리가 앞으로 한 번 넘었다 제자리로.
    #[test]
    fn starting_to_run_drags_tail_and_head_back_and_leans_the_body_in() {
        let t = Tuning::default();
        let r = rig();
        let mut sec = Secondary::default();
        let mut out = Vec::new();
        let h = 1.0 / 60.0;
        let mut pos = Vec3::ZERO;
        sec.update(&r, h, pos, 0.0, 0.0, 0.0, &t, &mut out);
        for _ in 0..6 {
            pos.z += 5.0 * h;
            sec.update(&r, h, pos, 0.0, 0.0, 0.0, &t, &mut out);
        }
        let (tail, head, body) = (r.tail.unwrap(), r.head.unwrap(), r.body.unwrap());
        assert!(
            rotated_mass(&out, &tail).z < tail.mass.z - 0.02,
            "tail drags back"
        );
        assert!(
            rotated_mass(&out, &head).z < head.mass.z - 0.01,
            "head lags back"
        );
        assert!(
            rotated_mass(&out, &body).z > body.mass.z + 0.01,
            "body leans into the run"
        );
        // 멈춤: 꼬리가 앞으로 넘어갔다가(겹치기) 돌아온다.
        let mut ahead = f32::MIN;
        for _ in 0..240 {
            sec.update(&r, h, pos, 0.0, 0.0, 0.0, &t, &mut out);
            ahead = ahead.max(rotated_mass(&out, &tail).z - tail.mass.z);
        }
        assert!(
            ahead > 0.01,
            "tail overshoots forward after stopping: {ahead}"
        );
        assert!(
            (rotated_mass(&out, &tail) - tail.mass).length() < 1e-3,
            "tail settles"
        );
    }

    /// 같은 움직임을 캐릭터가 오른쪽(요 90°)을 보고 해도 모델 공간에서는 같은 쪽으로 처진다.
    #[test]
    fn lag_is_measured_in_the_characters_own_frame() {
        let t = Tuning::default();
        let r = rig();
        let tail = r.tail.unwrap();
        let mut out = Vec::new();
        for yaw in [0.0f32, 1.3, -2.4] {
            let mut sec = Secondary::default();
            let fwd = Quat::from_rotation_y(yaw) * Vec3::Z;
            let mut pos = Vec3::ZERO;
            sec.update(&r, 1.0 / 60.0, pos, yaw, 0.0, 0.0, &t, &mut out);
            for _ in 0..6 {
                pos += fwd * 5.0 / 60.0;
                sec.update(&r, 1.0 / 60.0, pos, yaw, 0.0, 0.0, &t, &mut out);
            }
            assert!(
                rotated_mass(&out, &tail).z < tail.mass.z - 0.02,
                "yaw {yaw}"
            );
        }
    }

    /// 공중에서는 늘어나고, 내려앉으면 찌그러졌다 한 번 늘어나며 선다 — 부피(세 배율의 곱)는 늘 1.
    #[test]
    fn landing_squashes_then_bounces_and_keeps_volume() {
        let t = Tuning::default();
        let r = rig();
        let mut sec = Secondary::default();
        let mut out = Vec::new();
        let h = 1.0 / 60.0;
        let scale = |out: &[(usize, Quat, Vec3)]| out.iter().find(|e| e.0 == 0).unwrap().2;
        // 점프 한 번(0027: 높이 0.9m·체공 0.55초 → 뛰는 속도 ±6.5m/s).
        let n = (0.55 / h) as usize;
        let mut stretch = f32::MIN;
        for k in 0..n {
            let vy = 6.5 - 13.0 * k as f32 / n as f32;
            sec.update(&r, h, Vec3::ZERO, 0.0, 0.5, vy, &t, &mut out);
            stretch = stretch.max(scale(&out).y);
        }
        assert!(stretch > 1.03, "stretched in the air: {:?}", scale(&out));
        let (mut lo, mut hi_after_lo) = (f32::MAX, f32::MIN);
        for _ in 0..90 {
            sec.update(&r, h, Vec3::ZERO, 0.0, 0.0, 0.0, &t, &mut out);
            let s = scale(&out);
            assert!((s.x * s.y * s.z - 1.0).abs() < 1e-4, "volume {s:?}");
            if s.y < lo {
                lo = s.y;
                hi_after_lo = f32::MIN;
            } else {
                hi_after_lo = hi_after_lo.max(s.y);
            }
        }
        assert!(lo < 0.85, "squash on landing: {lo}");
        assert!(
            hi_after_lo > 1.0,
            "bounces back up past rest: {hi_after_lo}"
        );
        assert!((scale(&out).y - 1.0).abs() < 0.01, "settles");
    }
}
