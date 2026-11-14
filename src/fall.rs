//! 쓰러짐(0175): 체력이 다하면 세계가 멈추고 방랑자만 무너진다 → 화면 가장자리에서 뿌리가 감겨 들며 어두워진다 → 어둠 속에서
//! 쉼터 곁으로 옮겨진다 → 엎드린 몸이 화로 곁에서 일어난다. 사장 "죽었을 때 멈추는 것 없이 다시 살아나는 것 어색해"(2026-11-14) —
//! 그전에는 체력이 다한 그 틱에 쉼터 곁에 서 있었다.
//!
//! 세계관(lore README A·ECO-08): 삶과 죽음은 딱 갈리지 않고, 다시 일어나는 것이 부활인지는 정하지 않는다 — 그래서 글은 이미 있던
//! 쓰러짐의 한 줄("뿌리가 몸을 감는다…")만, 종은 울리지 않는다(종이 되살린다는 규칙처럼 읽힐 수 있다). 소리는 낮은 땅울림.
//!
//! 여기에는 시간표(순수 규칙)만 — 그림은 `hud::draw_fall`, 몸의 자세는 렌더(`View::pose` — `fall`·`wake` 동작), 소리·옮기기는 앱.
//! 월드는 이 장면을 모른다(무너지는 동안은 걷지 않고, 일어나는 동안은 빈 입력으로 흐른다).

/// 무너지는 동작(`fall`)의 길이 — 모델 원본 `wanderer.py`와 같다(`check_glb.py`가 본다).
pub const FALL_CLIP: f32 = 1.3;
/// 어둠이 들기 시작하는 때·다 드는 때.
pub const DARK_FROM: f32 = 0.9;
pub const DARK_FULL: f32 = 2.1;
/// 가장자리의 뿌리가 감겨 드는 구간.
pub const ROOTS_FROM: f32 = 0.6;
/// 땅울림(소리 하나).
pub const RUMBLE_AT: f32 = 1.0;
/// 글 한 줄이 떠 있는 구간(뜨고 걷히는 시간 `LINE_FADE`).
pub const LINE_FROM: f32 = 1.5;
pub const LINE_TO: f32 = 3.0;
const LINE_FADE: f32 = 0.45;
/// 어둠 속에서 쉼터 곁으로 옮기는 때(화면이 다 어둡다).
pub const MOVE_AT: f32 = 2.4;
/// 쉼터 곁에서 어둠이 걷히기 시작하는 때·다 걷히는 때.
pub const LIGHT_FROM: f32 = 3.1;
pub const LIGHT_FULL: f32 = 3.8;
/// 일어나기: `wake` 동작의 이 초부터 끝(`wake::CLIP`)까지를 이 빠르기로 — 숨 쉬는 엎드림(0~0.9초)은 건너뛴다.
pub const RISE_FROM_CLIP: f32 = 0.9;
pub const RISE_RATE: f32 = 1.3;
/// 일어나기 시작하는 때(어둠이 걷히기 조금 전 — 밝아질 때 이미 몸이 움직인다).
pub const RISE_AT: f32 = 3.0;
/// 장면의 끝 — 조작이 돌아온다.
pub const END: f32 = RISE_AT + (crate::wake::CLIP - RISE_FROM_CLIP) / RISE_RATE;
/// 카메라가 다가가는 몫(무너질 때 다가가고, 일어나며 제 거리로).
pub const CLOSE: f32 = 0.32;

fn smooth(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// 한 틱이 알리는 것.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cue {
    pub rumble: bool,
    /// 쉼터 곁으로 옮긴다(한 번 — 화면이 다 어두운 순간).
    pub moved: bool,
    pub done: bool,
}

/// 쓰러짐 하나.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Fall {
    pub t: f32,
}

impl Fall {
    pub fn tick(&mut self, dt: f32) -> Cue {
        let was = self.t;
        self.t += dt;
        let crossed = |at: f32| was < at && self.t >= at;
        Cue {
            rumble: crossed(RUMBLE_AT),
            moved: crossed(MOVE_AT),
            done: crossed(END),
        }
    }

    pub fn done(&self) -> bool {
        self.t >= END
    }

    /// 옮긴 뒤인가(그 뒤로 월드는 쉼터 곁에서 흐른다).
    pub fn moved(&self) -> bool {
        self.t >= MOVE_AT
    }

    /// 건너뛰기(옮긴 뒤 일어나는 동안만): 끝으로 — 무너지고 어두워지는 동안은 건너뛰지 않는다(쓰러짐이 읽혀야 한다).
    pub fn skip(&mut self) {
        if self.t >= LIGHT_FULL {
            self.t = END;
        }
    }

    /// 화면을 덮는 어둠 0..1.
    pub fn dark(&self) -> f32 {
        if self.t < LIGHT_FROM {
            smooth((self.t - DARK_FROM) / (DARK_FULL - DARK_FROM))
        } else {
            1.0 - smooth((self.t - LIGHT_FROM) / (LIGHT_FULL - LIGHT_FROM))
        }
    }

    /// 가장자리에서 감겨 드는 뿌리의 길이 0..1(어둠이 걷힐 때 같이 물러난다).
    pub fn roots(&self) -> f32 {
        if self.t < LIGHT_FROM {
            smooth((self.t - ROOTS_FROM) / (DARK_FULL - ROOTS_FROM))
        } else {
            self.dark()
        }
    }

    /// 글 한 줄의 진하기 0..1.
    pub fn line(&self) -> f32 {
        smooth((self.t - LINE_FROM) / LINE_FADE).min(smooth((LINE_TO - self.t) / LINE_FADE))
    }

    /// 몸의 자세: (동작 이름, 그 동작의 초). 무너지는 동안 `fall`, 어둠 속(옮기기 전후)은 엎드린 채, 그 뒤 `wake`로 일어난다.
    pub fn pose(&self) -> (&'static str, f32) {
        if self.t < MOVE_AT {
            ("fall", self.t.min(FALL_CLIP * 0.999))
        } else if self.t < RISE_AT {
            ("wake", RISE_FROM_CLIP)
        } else {
            (
                "wake",
                (RISE_FROM_CLIP + (self.t - RISE_AT) * RISE_RATE).min(crate::wake::CLIP * 0.999),
            )
        }
    }

    /// 카메라가 다가가 있는 몫.
    pub fn close(&self) -> f32 {
        if self.t < MOVE_AT {
            CLOSE * smooth(self.t / FALL_CLIP)
        } else {
            CLOSE * (1.0 - smooth((self.t - RISE_AT) / (END - RISE_AT)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 60.0;

    /// 0175: 무너짐 → 어둠 → (다 어두울 때 한 번) 옮김 → 밝아짐 → 일어남 → 끝. 옮기는 순간은 다 어둡고, 다 무너진 뒤에 어둠이 다 든다.
    #[test]
    fn the_wanderer_falls_in_view_and_is_moved_only_in_full_dark() {
        let mut f = Fall::default();
        let (mut moved, mut rumbles, mut done) = (0, 0, 0);
        while !f.done() {
            let c = f.tick(DT);
            if c.moved {
                moved += 1;
                assert!(f.dark() > 0.999, "dark when moving: {}", f.dark());
                assert_eq!(f.roots(), 1.0);
            }
            rumbles += c.rumble as u32;
            done += c.done as u32;
        }
        assert_eq!((moved, rumbles, done), (1, 1, 1));
        assert_eq!(f.dark(), 0.0);
        // 무너지는 동작은 어둠이 다 들기 전에 끝난다(쓰러지는 것이 보인다).
        assert!(FALL_CLIP < DARK_FULL);
        // 글은 다 어두운 동안에 뜨고 밝아지기 전에 걷힌다.
        assert!(LINE_FROM > DARK_FROM && LINE_TO <= LIGHT_FROM);
        // 몸이 일어나기 시작할 때는 아직 어둡다(밝아질 때 이미 움직인다) — 그리고 길이는 6초 안.
        assert!(RISE_AT < LIGHT_FROM && END < 6.0, "{END}");
    }

    /// 0175: 몸의 자세는 끊기지 않는다 — 무너짐(`fall`)은 늘 앞으로, 일어남(`wake`)은 엎드림에서 끝까지 늘 앞으로.
    #[test]
    fn the_pose_runs_fall_then_lies_still_then_rises_to_the_end() {
        let mut f = Fall::default();
        let mut last: Option<(&str, f32)> = None;
        while !f.done() {
            let p = f.pose();
            if let Some(l) = last {
                if l.0 == p.0 {
                    assert!(p.1 >= l.1, "{l:?} → {p:?}");
                } else {
                    assert_eq!((l.0, p.0), ("fall", "wake"));
                    // `fall`의 끝 자세 = `wake`의 엎드림(모델 원본이 맞춘다) — 어둠 속에서 바뀐다.
                    assert!(f.dark() > 0.999);
                }
            }
            last = Some(p);
            f.tick(DT);
        }
        let end = last.unwrap();
        assert_eq!(end.0, "wake");
        assert!(end.1 > crate::wake::CLIP * 0.98);
    }

    /// 0175: 건너뛰기는 밝아진 뒤(일어나는 동안)에만 — 무너지고 어두워지는 것은 늘 본다.
    #[test]
    fn only_the_rise_can_be_skipped() {
        let mut f = Fall::default();
        while f.t < MOVE_AT + 0.2 {
            f.skip();
            assert!(!f.done());
            f.tick(DT);
        }
        while f.t < LIGHT_FULL {
            f.tick(DT);
        }
        f.skip();
        assert!(f.done());
    }
}
