//! 지역 사이 전환(0118 → 0174): 뿌리가 얽힌 수직 굴을 방랑자가 큰 뿌리를 밟고 내려가는(올라가는) 3D 장면.
//! 예전(0118)은 nail의 2D 지층 단면을 화면에 덮었다 — 이 게임의 그림(3D 도트)과 어울리지 않았다(사장 2026-11-13).
//! 여기는 순수 규칙(시험): 시간표(떠남 → 굴 → 닿음)·지역을 바꾸는 순간·깊이 숫자·굴 속의 길과 카메라.
//! 그림은 `render/descent.rs`(굴·뿌리·방랑자), 덮는 어둠과 깊이 글은 앱의 화면 패스.
//! KB `conveying-descent`: 이동 자체를 보여 준다 · 깊이 숫자는 떠난 곳 → 닿을 곳으로 · 오르는 길은 조금 길게.
use macroquad::prelude::*;

/// 떠나는 곳이 어두워지는 초.
pub const LEAVE: f32 = 0.35;
/// 굴 장면의 길이: 내려가기·올라가기(올라가기가 조금 길다 — nail 0204 "먼 길").
pub const SHAFT_DOWN: f32 = 3.4;
pub const SHAFT_UP: f32 = 3.9;
/// 굴 장면이 어둠에서 나오고·어둠으로 들어가는 초.
pub const FADE: f32 = 0.3;
/// 닿은 곳을 다 어두운 채로 한 번 그리는 초(지형 굽기가 여기 숨는다) + 밝아지는 초.
pub const HOLD: f32 = 0.12;
pub const ARRIVE: f32 = 0.5;
/// 굴 속에서 종이 한 번 울리는 자리(굴 장면의 몫).
const BELL_AT: f32 = 0.45;

/// 굴의 꼴(m): 안쪽 반지름, 큰 뿌리의 길(굴 축에서의 거리)·반 굵기, 한 번에 도는 각(라디안)·내려가는 높이.
pub const SHAFT_R: f32 = 5.2;
pub const PATH_R: f32 = 3.7;
pub const ROOT_R: f32 = 0.6;
pub const SWEEP: f32 = 2.4;
pub const DROP: f32 = 5.0;
/// 카메라: 굴 축에서의 거리(방랑자의 맞은편), 방랑자 발보다 높은 만큼, 시야(라디안).
const EYE_R: f32 = 3.4;
const EYE_UP: f32 = 2.6;
pub const EYE_FOV: f32 = 0.82;
/// 카메라는 방랑자의 맞은편에서 이만큼 비켜 선다(라디안) — 길이 화면을 비스듬히 가로지르게.
const EYE_OFF: f32 = 0.5;

/// 지금 무엇을 그리나.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    /// 떠나는 곳(월드)이 어두워진다.
    Leave,
    /// 굴 장면(월드는 그리지 않는다 — 이미 닿을 곳으로 바뀌어 있다).
    Shaft,
    /// 닿은 곳(월드)이 밝아진다.
    Arrive,
}

/// 한 틱의 일.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Tick {
    /// 지역을 바꿀 때(한 번 — 화면이 다 어두운 순간).
    pub swap: bool,
    /// 굴 속의 종(한 번).
    pub bell: bool,
    pub done: bool,
}

/// 지역 사이 전환 하나.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Passage {
    /// 닿을 지역(`stage::STAGES` 차례).
    pub to: usize,
    pub up: bool,
    pub from_m: u32,
    pub to_m: u32,
    /// 지난 초.
    pub t: f32,
    /// 지역을 이미 바꿨나 · 종을 이미 울렸나.
    pub swapped: bool,
    belled: bool,
}

fn smooth(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

impl Passage {
    pub fn new(to: usize, up: bool, from_m: u32, to_m: u32) -> Self {
        Self {
            to,
            up,
            from_m,
            to_m,
            t: 0.0,
            swapped: false,
            belled: false,
        }
    }

    /// 굴 장면의 길이.
    pub fn shaft_len(&self) -> f32 {
        if self.up {
            SHAFT_UP
        } else {
            SHAFT_DOWN
        }
    }

    pub fn length(&self) -> f32 {
        LEAVE + self.shaft_len() + HOLD + ARRIVE
    }

    pub fn phase(&self) -> Phase {
        if self.t < LEAVE {
            Phase::Leave
        } else if self.t < LEAVE + self.shaft_len() {
            Phase::Shaft
        } else {
            Phase::Arrive
        }
    }

    /// 굴 장면 안의 시각(초, 0..`shaft_len`).
    fn in_shaft(&self) -> f32 {
        (self.t - LEAVE).clamp(0.0, self.shaft_len())
    }

    /// 화면을 덮는 어둠 0..1: 떠나며 짙어지고, 굴은 어둠에서 나왔다 어둠으로 들어가고, 닿은 곳은 잠깐 어두웠다가 밝아진다.
    pub fn dark(&self) -> f32 {
        match self.phase() {
            Phase::Leave => smooth(self.t / LEAVE),
            Phase::Shaft => {
                let u = self.in_shaft();
                smooth(1.0 - u / FADE).max(smooth((u - (self.shaft_len() - FADE)) / FADE))
            }
            Phase::Arrive => {
                let u = self.t - LEAVE - self.shaft_len() - HOLD;
                smooth(1.0 - u / ARRIVE)
            }
        }
    }

    /// 굴 장면의 진행 0..1(시간).
    pub fn along(&self) -> f32 {
        self.in_shaft() / self.shaft_len()
    }

    /// 큰 뿌리 위의 자리 0(위 끝)..1(아래 끝): 내려가면 0 → 1, 올라가면 1 → 0.
    pub fn travel(&self) -> f32 {
        if self.up {
            1.0 - self.along()
        } else {
            self.along()
        }
    }

    /// 지금 보이는 깊이(m): 굴을 지나는 동안 떠난 곳 → 닿을 곳으로(부드럽게, 늘 한쪽으로).
    pub fn depth(&self) -> f32 {
        let e = smooth(self.along());
        self.from_m as f32 + (self.to_m as f32 - self.from_m as f32) * e
    }

    /// 건너뛴다(굴 장면에서만): 굴이 어둠으로 들어가는 데로 — 닿는 쪽의 밝아짐은 그대로 본다.
    pub fn skip(&mut self) {
        let out = LEAVE + self.shaft_len() - FADE;
        if self.phase() == Phase::Shaft && self.in_shaft() >= FADE && self.t < out {
            self.t = out;
        }
    }

    /// 한 틱 나아간다.
    pub fn step(&mut self, dt: f32) -> Tick {
        self.t += dt;
        // 지역은 떠나는 곳이 다 어두워진 순간에 바꾼다 — 그 뒤 굴 장면 동안 월드는 그리지 않는다.
        let swap = !self.swapped && self.t >= LEAVE;
        self.swapped |= swap;
        let bell = !self.belled && self.phase() == Phase::Shaft && self.along() >= BELL_AT;
        self.belled |= bell;
        Tick {
            swap,
            bell,
            done: self.t >= self.length(),
        }
    }
}

/// 큰 뿌리의 가운데 줄(굴 안의 자리, y = 0이 위 끝): `s` 0..1이 걷는 구간, 그 밖으로도 이어진다(뿌리는 화면 밖에서 오고 간다).
pub fn path(s: f32) -> Vec3 {
    let a = SWEEP * s;
    vec3(PATH_R * a.sin(), -DROP * s, PATH_R * a.cos())
}

/// 굴 축 둘레의 각(라디안) — `path(s)`가 선 쪽.
pub fn angle(s: f32) -> f32 {
    SWEEP * s
}

/// 방랑자: (발의 자리 — 큰 뿌리의 윗면, 바라보는 쪽 `atan2(x, z)`). 내려가면 길의 아래쪽을, 올라가면 위쪽을 본다.
pub fn walker(p: &Passage) -> (Vec3, f32) {
    let s = p.travel();
    let ahead = if p.up { -0.02 } else { 0.02 };
    let d = path(s + ahead) - path(s);
    (path(s) + Vec3::Y * ROOT_R, d.x.atan2(d.z))
}

/// 카메라: (자리, 보는 곳). 방랑자의 맞은편 벽 쪽에서 조금 비켜, 조금 위에서 — 가는 쪽(내려가면 아래)을 살짝 앞서 본다.
pub fn eye(p: &Passage) -> (Vec3, Vec3) {
    let s = p.travel();
    let (feet, _) = walker(p);
    let a = angle(s) + std::f32::consts::PI - EYE_OFF;
    let pos = vec3(EYE_R * a.sin(), feet.y + EYE_UP, EYE_R * a.cos());
    let lead = if p.up { 0.5 } else { -0.3 };
    (pos, feet + Vec3::Y * (0.9 + lead))
}

/// 깊이별 흙 색 키(m → 색): 얕을수록 따뜻한 갈색, 깊을수록 남보라(0037 "해 질 녘 남보라" 세계).
const SOIL: [(f32, [u8; 3]); 5] = [
    (0.0, [0x6b, 0x55, 0x4a]),
    (15.0, [0x57, 0x46, 0x47]),
    (35.0, [0x45, 0x3a, 0x4a]),
    (60.0, [0x35, 0x2e, 0x47]),
    (100.0, [0x24, 0x20, 0x38]),
];

/// 깊이 `m`의 흙빛(굴 벽).
pub fn soil(m: f32) -> Color {
    let mut a = SOIL[0];
    for &b in &SOIL[1..] {
        if m <= b.0 {
            let k = ((m - a.0) / (b.0 - a.0)).clamp(0.0, 1.0);
            let c = |i: usize| (a.1[i] as f32 + (b.1[i] as f32 - a.1[i] as f32) * k) / 255.0;
            return Color::new(c(0), c(1), c(2), 1.0);
        }
        a = b;
    }
    let c = SOIL[SOIL.len() - 1].1;
    Color::from_rgba(c[0], c[1], c[2], 255)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 60.0;

    /// 0174: 떠남 → 굴 → 닿음. 지역은 화면이 다 어두운 순간에 한 번 바뀌고, 그 뒤로 닿는 쪽이 밝아질 때까지 월드는 보이지 않는다
    /// (굴 장면이거나 다 어둡다). 깊이는 굴을 지나는 동안 떠난 곳 → 닿을 곳으로 늘 한쪽으로 흐른다.
    #[test]
    fn the_region_swaps_once_in_the_dark_and_the_world_stays_hidden_until_arrival() {
        let mut p = Passage::new(1, false, 0, 40);
        let (mut swaps, mut bells, mut last, mut phases) = (0, 0, p.depth(), vec![p.phase()]);
        loop {
            let tick = p.step(DT);
            if tick.swap {
                swaps += 1;
                assert!(p.dark() > 0.999, "dark when swapping: {}", p.dark());
                assert_eq!(p.phase(), Phase::Shaft);
            }
            bells += tick.bell as u32;
            if phases.last() != Some(&p.phase()) {
                phases.push(p.phase());
            }
            // 굴 장면이 시작될 때(지역을 바꾼 직후)와 닿는 쪽의 첫 틱들은 다 어둡다 — 굽기가 숨는다.
            if p.phase() == Phase::Arrive && p.t < LEAVE + p.shaft_len() + HOLD {
                assert!(p.dark() > 0.999);
            }
            assert!(p.depth() >= last - 1e-4, "depth only grows going down");
            last = p.depth();
            if tick.done {
                break;
            }
        }
        assert_eq!((swaps, bells), (1, 1));
        assert_eq!(phases, [Phase::Leave, Phase::Shaft, Phase::Arrive]);
        assert_eq!(p.depth(), 40.0);
        assert!(p.dark() < 1e-3, "clear at the end");
        // 굴 장면의 끝은 다 어둡다(닿는 쪽의 첫 그림이 어둠 뒤에서 나온다).
        let mut q = Passage::new(1, false, 0, 40);
        q.t = LEAVE + q.shaft_len() - 1e-3;
        assert!(q.dark() > 0.999);
        // 굴 한가운데는 어둠이 걷혀 있다.
        q.t = LEAVE + q.shaft_len() * 0.5;
        assert_eq!(q.dark(), 0.0);
        let up = Passage::new(0, true, 40, 0);
        assert_eq!(up.depth(), 40.0);
        assert!(up.length() > p.length(), "the way up is longer");
    }

    /// 0174: 내려가면 방랑자는 큰 뿌리를 따라 늘 아래로·길의 아래쪽을 보고 걷는다, 올라가면 거꾸로. 발은 뿌리의 윗면에 있다.
    #[test]
    fn the_wanderer_walks_the_great_root_down_or_up_facing_the_way() {
        for up in [false, true] {
            let mut p = Passage::new(1, up, 0, 40);
            p.t = LEAVE;
            let (mut last, mut n) = (walker(&p).0, 0);
            while p.phase() != Phase::Arrive {
                p.step(DT);
                if p.phase() != Phase::Shaft {
                    continue;
                }
                let (feet, yaw) = walker(&p);
                let moved = feet - last;
                if up {
                    assert!(feet.y >= last.y - 1e-4, "rises going up");
                } else {
                    assert!(feet.y <= last.y + 1e-4, "sinks going down");
                }
                // 바라보는 쪽 = 걷는 쪽.
                if moved.length() > 1e-4 {
                    let dir = vec3(yaw.sin(), 0.0, yaw.cos());
                    assert!(dir.dot(vec3(moved.x, 0.0, moved.z).normalize()) > 0.95);
                    n += 1;
                }
                // 발은 뿌리 윗면에, 길은 굴 안에(벽과 뿌리 굵기만큼 떨어져).
                let s = p.travel();
                assert!((feet.y - (path(s).y + ROOT_R)).abs() < 1e-4);
                assert!(vec2(feet.x, feet.z).length() + ROOT_R < SHAFT_R - 0.5);
                last = feet;
            }
            assert!(n > 100, "{n}");
        }
        // 걷는 빠르기는 걸음으로 읽힐 만큼(초당 2~5m).
        let len: f32 = (0..100)
            .map(|i| path(i as f32 / 100.0).distance(path((i + 1) as f32 / 100.0)))
            .sum();
        for t in [SHAFT_DOWN, SHAFT_UP] {
            assert!((2.0..5.0).contains(&(len / t)), "{}", len / t);
        }
    }

    /// 0174: 카메라는 굴 안에 서고(벽을 뚫지 않는다) 방랑자를 화면에 둔다 — 한 틱에 튀지 않는다.
    #[test]
    fn the_camera_stays_inside_the_shaft_and_keeps_the_wanderer_in_view() {
        for up in [false, true] {
            let mut p = Passage::new(1, up, 0, 40);
            p.t = LEAVE;
            let mut prev = eye(&p).0;
            while p.phase() == Phase::Shaft {
                let (pos, look) = eye(&p);
                assert!(vec2(pos.x, pos.z).length() < SHAFT_R - 1.0);
                let (feet, _) = walker(&p);
                let d = pos.distance(feet);
                assert!((5.0..10.0).contains(&d), "distance {d}");
                // 방랑자의 몸통이 시야 한가운데 가까이.
                let fwd = (look - pos).normalize();
                let to = (feet + Vec3::Y * 0.9 - pos).normalize();
                assert!(fwd.dot(to) > (EYE_FOV * 0.3).cos(), "{}", fwd.dot(to));
                assert!(pos.distance(prev) < 0.2, "no jump: {}", pos.distance(prev));
                prev = pos;
                p.step(DT);
            }
        }
    }

    /// 0174: 건너뛰기는 굴 장면에서만 — 굴이 어둠으로 들어가는 데로 가고, 지역은 이미 바뀌어 있다(닿는 쪽의 밝아짐은 본다).
    #[test]
    fn skipping_jumps_to_the_shafts_fade_out_and_never_past_the_swap() {
        let mut p = Passage::new(1, false, 0, 40);
        p.skip();
        assert_eq!(p.t, 0.0, "not while leaving");
        while p.along() < 0.3 {
            p.step(DT);
        }
        assert!(p.swapped);
        p.skip();
        assert!((p.t - (LEAVE + p.shaft_len() - FADE)).abs() < 1e-5);
        let left = p.length() - p.t;
        assert!(left < 1.0, "{left}");
        let t = p.t;
        p.skip();
        assert_eq!(p.t, t, "once");
        assert_eq!(p.phase(), Phase::Shaft);
    }
}
