//! "성소에 들다"(0262 — 새 기록에서 한 번, 기록 `prologue`): 서막 숲 끝의 표지를 밟은 방랑자가 침묵의 성소에 든다.
//! 숲이 어두워진다(방랑자는 그대로 걸어 든다) → **다 어두울 때** 지역을 성소로 바꾼다(0174 규칙 — 닿을 곳의 첫 굽기가 어둠 뒤에서)
//! → 위아래 띠 + 어둠이 걷히며 카메라가 성소를 높이·넓게 본다(무너진 입구 너머 다리·뜰·닫힌 성소 쪽) → 방랑자 뒤로 내려온다
//! → 그동안 방랑자가 몇 걸음 저절로 걸어 들어온다(앱이 넣는 걷기 입력 — 월드는 장면을 모른다) → 이름 카드 → 띠가 걷히며 조작이 돌아온다.
//!
//! 여기에는 시간표(순수 규칙)만 — 그림은 `hud::draw_arrive`, 카메라는 앱이 `shot`을 `Camera::shot`에 얹는다, 소리·기록은 앱이 맡는다.
//! 건너뛰기 = 멈춤 버튼(0171 깨어나는 장면과 같다). 그동안 스테이지 대본·튜토리얼·종소리의 시계는 멈춘다(앱).
use macroquad::prelude::*;

/// 숲이 어두워지는 초.
pub const LEAVE: f32 = 0.9;
/// 다 어두운 채 머무는 초(성소의 지형 굽기가 여기 숨는다).
pub const HOLD: f32 = 0.3;
/// 어둠이 걷히는 초(성소가 드러난다).
pub const REVEAL: f32 = 1.6;
/// 높이·넓게 보는 초(카메라는 천천히 미끄러진다), 그 뒤 방랑자 뒤로 내려오는 초.
pub const LOOK: f32 = 3.4;
pub const DESCEND: f32 = 2.6;
/// 방랑자가 저절로 걷는 때(성소에 닿은 뒤부터 잰다)·길이·스틱의 세기(천천히 걷는다).
pub const WALK_AT: f32 = 2.6;
pub const WALK_LEN: f32 = 3.2;
pub const WALK: f32 = 0.55;
/// 이름 카드가 뜨는 때(닿은 뒤부터)·길이·뜨고 지는 초.
pub const CARD_AT: f32 = 5.6;
pub const CARD_LEN: f32 = 3.0;
pub const CARD_FADE: f32 = 0.6;
/// 띠(화면 높이의 몫 — 깨어나는 장면과 같다)와 걷히는 초.
pub const BARS: f32 = crate::wake::BARS;
const BARS_OUT: f32 = 0.7;
/// 성소가 드러나며 종이 한 번(닿은 뒤부터).
const BELL_AT: f32 = 0.5;
/// 건너뛴 뒤 어둠이 걷히는 초(그 순간 화면을 덮어 카메라가 제자리로 튀는 것을 가린다).
const BLINK: f32 = 0.45;

/// 닿은 순간(지역을 바꾸는 때).
pub const SWAP: f32 = LEAVE;
/// 닿은 곳이 드러나기 시작하는 때.
const T0: f32 = LEAVE + HOLD;
/// 장면의 끝 — 조작이 돌아온다.
pub const END: f32 = T0 + CARD_AT + CARD_LEN + BARS_OUT;
const SKIP_TO: f32 = END - BARS_OUT;

/// 높이·넓게 보는 카메라(방랑자가 닿는 자리 `at`, 성소 쪽 한 점 `far`): 처음 자리(성소 쪽에서 등지고 높이) → 끝 자리(조금 낮게·가까이).
const EYE_BACK: [f32; 2] = [24.0, 10.0];
const EYE_UP: [f32; 2] = [14.0, 7.0];
/// 보는 곳: 닿는 자리에서 성소 쪽으로 이만큼(m), 땅에서 이만큼 위.
const LOOK_AHEAD: [f32; 2] = [30.0, 10.0];
const LOOK_UP: f32 = 1.5;

fn smooth(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// 한 틱이 알리는 것.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cue {
    /// 지역을 성소로 바꾼다(한 번 — 화면이 다 어두운 순간).
    pub swap: bool,
    /// 종이 운다(한 번).
    pub bell: bool,
    /// 끝났다(한 번).
    pub done: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Arrive {
    pub t: f32,
    swapped: bool,
    belled: bool,
    /// 건너뛰기를 눌렀다(닿기 전이면 닿는 순간에 건너뛴다 — 지역은 다 어두울 때만 바꾼다).
    skip: bool,
    /// 건너뛴 때(그때부터 어둠이 걷힌다).
    skipped: Option<f32>,
}

impl Arrive {
    pub fn tick(&mut self, dt: f32) -> Cue {
        let was = self.t;
        self.t += dt;
        let mut cue = Cue::default();
        if !self.swapped && self.t >= SWAP {
            self.swapped = true;
            cue.swap = true;
            if self.skip {
                self.jump();
            }
        }
        if !self.belled && self.skipped.is_none() && self.t >= T0 + BELL_AT {
            self.belled = true;
            cue.bell = true;
        }
        if was < END && self.t >= END {
            cue.done = true;
        }
        cue
    }

    pub fn done(&self) -> bool {
        self.t >= END
    }

    /// 닿았나(성소로 바뀌었나).
    pub fn arrived(&self) -> bool {
        self.swapped
    }

    /// 건너뛰기: 끝의 띠 걷힘으로. 아직 숲이면 닿는 순간에(지역은 다 어두울 때만 바꾼다). 두 번 눌러도 같다.
    pub fn skip(&mut self) {
        if !self.swapped {
            self.skip = true;
        } else if self.t < SKIP_TO {
            self.jump();
        }
    }

    fn jump(&mut self) {
        self.t = self.t.max(SKIP_TO);
        self.skipped = Some(self.t);
        self.belled = true;
    }

    /// 닿은 뒤 지난 초(닿기 전 0).
    fn since(&self) -> f32 {
        (self.t - T0).max(0.0)
    }

    /// 화면을 덮는 어둠 0..1: 숲에서 짙어지고, 다 어두운 채 잠깐, 성소가 드러나며 걷힌다. 건너뛰면 그 순간 덮였다가 걷힌다.
    pub fn dark(&self) -> f32 {
        let d = if self.t < LEAVE {
            smooth(self.t / LEAVE)
        } else if self.t < T0 {
            1.0
        } else {
            1.0 - smooth((self.t - T0) / REVEAL)
        };
        match self.skipped {
            Some(at) => d.max(1.0 - smooth((self.t - at) / BLINK)),
            None => d,
        }
    }

    /// 위아래 띠의 몫 0..1(1 = `BARS` 가득): 숲이 어두워지며 들어오고 끝에서 걷힌다.
    pub fn bars(&self) -> f32 {
        smooth(self.t / LEAVE) * (1.0 - smooth((self.t - SKIP_TO) / BARS_OUT))
    }

    /// 카메라를 장면의 자리로 당기는 몫 0..1: 닿은 뒤 높이·넓게(1) → 방랑자 뒤로 내려와 0. 숲(닿기 전)·건너뛴 뒤는 0.
    pub fn weight(&self) -> f32 {
        if !self.swapped || self.skipped.is_some() {
            return 0.0;
        }
        1.0 - smooth((self.since() - LOOK) / DESCEND)
    }

    /// 장면의 카메라 자리(눈, 보는 곳): `at` = 방랑자가 닿은 자리, `far` = 성소 쪽 한 점. 보는 동안 천천히 낮게·가까이 미끄러진다.
    pub fn eye(&self, at: Vec3, far: Vec3) -> (Vec3, Vec3) {
        let d = vec3(far.x - at.x, 0.0, far.z - at.z);
        let dir = if d.length_squared() > 1e-4 {
            d.normalize()
        } else {
            Vec3::NEG_Z
        };
        let g = smooth(self.since() / (LOOK + DESCEND));
        let mix = |a: [f32; 2]| a[0] + (a[1] - a[0]) * g;
        let eye = at - dir * mix(EYE_BACK) + Vec3::Y * mix(EYE_UP);
        let look = at + dir * mix(LOOK_AHEAD) + Vec3::Y * LOOK_UP;
        (eye, look)
    }

    /// 앱이 넣는 걷기 입력의 세기 0..`WALK`(성소 쪽으로): 숲 끝에서 그대로 걸어 들고, 닿은 뒤 잠깐 서 있다가 몇 걸음. 건너뛰면 없다.
    pub fn walk(&self) -> f32 {
        if self.skipped.is_some() {
            return 0.0;
        }
        if !self.swapped {
            return WALK;
        }
        let u = self.since() - WALK_AT;
        let ramp = 0.35;
        WALK * smooth(u / ramp) * (1.0 - smooth((u - WALK_LEN + ramp) / ramp))
    }

    /// 이름 카드의 진하기 0..1.
    pub fn card(&self) -> f32 {
        if self.skipped.is_some() {
            return 0.0;
        }
        let u = self.since() - CARD_AT;
        smooth(u / CARD_FADE) * (1.0 - smooth((u - CARD_LEN + CARD_FADE) / CARD_FADE))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 60.0;

    fn run(mut a: Arrive, mut each: impl FnMut(&Arrive, Cue)) -> Arrive {
        let mut n = 0;
        while !a.done() {
            let c = a.tick(DT);
            each(&a, c);
            n += 1;
            assert!(n < 60 * 60, "it ends");
        }
        a
    }

    /// 차례: 숲이 어두워진다 → 다 어두운 순간에 한 번 지역을 바꾼다 → 잠깐 어둠(굽기) → 드러나며 종 한 번 → 높이 보다가 내려온다 →
    /// 몇 걸음 → 이름 카드 → 띠가 걷히며 끝. 끝은 한 번만 알린다.
    #[test]
    fn the_scene_darkens_swaps_once_at_full_dark_then_looks_walks_and_names() {
        let (mut swaps, mut bells, mut ends) = (Vec::new(), 0, 0);
        let a = run(Arrive::default(), |a, c| {
            if c.swap {
                swaps.push(a.dark());
            }
            bells += c.bell as u32;
            ends += c.done as u32;
        });
        assert_eq!(swaps, vec![1.0], "swaps once, only when fully dark");
        assert_eq!((bells, ends), (1, 1));
        let mut b = a;
        assert_eq!(b.tick(DT), Cue::default(), "the end is told once");
        // 다 어두운 동안은 닿은 곳을 그대로 덮는다(굽기가 숨는다).
        let at = |t: f32| Arrive {
            t,
            swapped: t >= SWAP,
            ..Default::default()
        };
        assert_eq!(at(SWAP + HOLD * 0.5).dark(), 1.0);
        assert_eq!(at(T0 + REVEAL).dark(), 0.0);
        // 카메라: 숲에서는 그대로, 닿으면 장면의 자리, 내려오며 0으로 — 끝에 튀지 않는다.
        assert_eq!(at(SWAP * 0.5).weight(), 0.0);
        assert_eq!(at(T0).weight(), 1.0);
        assert_eq!(at(T0 + LOOK).weight(), 1.0);
        assert_eq!(at(T0 + LOOK + DESCEND).weight(), 0.0);
        // 걷기: 숲 끝에서 그대로 걸어 들고, 닿은 뒤 서 있다가 몇 걸음, 카드 전에 멈춘다.
        assert_eq!(at(SWAP * 0.5).walk(), WALK);
        assert_eq!(at(T0 + 1.0).walk(), 0.0);
        assert!((at(T0 + WALK_AT + WALK_LEN * 0.5).walk() - WALK).abs() < 1e-4);
        assert_eq!(at(T0 + WALK_AT + WALK_LEN).walk(), 0.0);
        assert!(
            WALK_AT + WALK_LEN <= CARD_AT + CARD_FADE,
            "stops by the card"
        );
        // 카드는 내려온 뒤 — 끝 전에 다 진다. 띠는 끝에서 걷힌다.
        assert!(CARD_AT >= LOOK + DESCEND - 1.5);
        assert!((at(T0 + CARD_AT + CARD_LEN * 0.5).card() - 1.0).abs() < 1e-4);
        assert_eq!(at(T0 + CARD_AT + CARD_LEN).card(), 0.0);
        assert_eq!((at(T0 + 1.0).bars(), at(END).bars()), (1.0, 0.0));
    }

    /// 높이 보는 눈은 닿는 자리 뒤 위에서 성소 쪽을 본다 — 내려오며 낮아진다.
    #[test]
    fn the_establishing_eye_looks_from_high_behind_toward_the_sanctum() {
        let (at, far) = (vec3(0.0, -1.0, 50.0), vec3(0.0, 0.0, -50.0));
        let a = Arrive {
            t: T0,
            swapped: true,
            ..Default::default()
        };
        let (eye, look) = a.eye(at, far);
        assert!(
            eye.z > at.z + 20.0 && eye.y > at.y + 12.0,
            "behind and high: {eye:?}"
        );
        assert!(look.z < at.z - 20.0, "toward the sanctum: {look:?}");
        let later = Arrive {
            t: T0 + LOOK + DESCEND,
            ..a
        };
        assert!(later.eye(at, far).0.y < eye.y);
    }

    /// 건너뛰기(멈춤 버튼): 숲에서 누르면 지역은 그래도 다 어두울 때 한 번 바꾸고 곧장 끝의 띠 걷힘으로. 닿은 뒤 누르면 그 순간 덮였다
    /// 걷힌다 — 카메라는 제자리·걷기와 카드는 없다. 짧고, 두 번 눌러도 같다.
    #[test]
    fn skipping_still_swaps_at_full_dark_and_ends_short() {
        for press_at in [0.2, T0 + 2.0] {
            let mut a = Arrive::default();
            let mut swaps = Vec::new();
            while a.t < press_at {
                if a.tick(DT).swap {
                    swaps.push(a.dark());
                }
            }
            a.skip();
            a.skip();
            let mut ticks = 0;
            let end = run(a, |a, c| {
                ticks += 1;
                if c.swap {
                    swaps.push(a.dark());
                }
                if a.arrived() {
                    assert_eq!((a.weight(), a.walk(), a.card()), (0.0, 0.0, 0.0));
                }
            });
            assert_eq!(swaps, vec![1.0], "pressed at {press_at}");
            let left = ticks as f32 * DT;
            assert!(left < LEAVE + 1.0, "short: {left}");
            assert_eq!(end.bars(), 0.0);
        }
    }
}
