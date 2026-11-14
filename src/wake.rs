//! 깨어나는 첫 장면(0171 — 새 기록에서 한 번): 과거를 잃은 방랑자가 꺼진 화로 곁에서 눈을 뜬다.
//! 검은 화면 + 종 한 번 → 눈꺼풀이 떨리다 열린다 → 엎드린 몸이 일어난다(`wake` 동작) → 두 줄의 서술 → 싸움 화면으로.
//!
//! 여기에는 시간표(순수 규칙)만 — 그림은 `hud::draw_wake`, 몸의 자세는 렌더(`View::wake`), 소리·기록은 앱이 맡는다.
//! 월드는 이 장면을 모른다(입력 없이 그냥 흐른다).

/// 종이 우는 때(초).
pub const BELL_AT: f32 = 0.8;
/// 눈꺼풀이 열리기 시작하는 때.
pub const OPEN_AT: f32 = 2.6;
/// 눈이 다 열리는 데 걸리는 시간(떨림까지).
pub const OPEN_LEN: f32 = 2.6;
/// 몸이 움직이기 시작하는 때(그 전에는 엎드린 채 — `wake` 동작의 0초 자세).
pub const RISE_AT: f32 = 5.6;
/// `wake` 동작의 길이(모델 원본 `wanderer.py`와 같다 — `check_glb.py`가 본다).
pub const CLIP: f32 = 4.0;
/// 서술 두 줄이 뜨는 때: 눈이 열리며 첫 줄, 몸이 일어설 때 둘째 줄.
pub const LINE_AT: [f32; 2] = [3.7, 7.0];
/// 글이 걷히기 시작하는 때·글이 뜨고 걷히는 시간.
pub const LINES_OUT: f32 = 9.6;
pub const LINE_FADE: f32 = 0.7;
/// 장면의 끝 — 조작이 돌아온다.
pub const END: f32 = 10.6;
/// 장면 동안 위아래에 남는 띠(화면 높이의 몫) — "지금은 보는 때"의 표지. 끝에서 걷힌다.
pub const BARS: f32 = 0.085;
/// 띠가 걷히는 시간.
const BARS_OUT: f32 = 0.7;
/// 카메라가 다가가 있는 몫(주시점까지 거리의) — 눈을 뜰 때 가깝고, 일어서며 제 거리로 물러난다.
pub const CLOSE: f32 = 0.42;
/// 건너뛰면 여기서부터(끝의 띠 걷힘만 남는다) — 그 순간 눈을 한 번 깜빡여 누운 몸 → 선 몸의 튐을 가린다.
const SKIP_TO: f32 = END - BARS_OUT;
const BLINK: f32 = 0.45;

fn smooth(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// 틱이 알리는 것.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cue {
    None,
    /// 종이 운다(소리 하나).
    Bell,
    /// 장면이 끝났다(기록에 적는다 · 시작 글이 흐른다).
    Done,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Wake {
    pub t: f32,
    /// 건너뛴 때(그때부터 한 번 깜빡인다).
    skipped: Option<f32>,
}

impl Wake {
    pub fn tick(&mut self, dt: f32) -> Cue {
        let was = self.t;
        self.t += dt;
        if was < BELL_AT && self.t >= BELL_AT {
            Cue::Bell
        } else if was < END && self.t >= END {
            Cue::Done
        } else {
            Cue::None
        }
    }

    pub fn done(&self) -> bool {
        self.t >= END
    }

    /// 건너뛰기: 끝의 띠 걷힘으로. 이미 거기면 그대로.
    pub fn skip(&mut self) {
        if self.t < SKIP_TO {
            self.t = SKIP_TO;
            self.skipped = Some(SKIP_TO);
        }
    }

    /// 눈꺼풀이 열린 몫 0(감김)..1(다 뜸): 조금 열렸다 도로 감기고(떨림) 천천히 다 열린다.
    pub fn lids(&self) -> f32 {
        let x = (self.t - OPEN_AT) / OPEN_LEN;
        let open = if x <= 0.0 {
            0.0
        } else if x < 0.2 {
            0.24 * smooth(x / 0.2)
        } else if x < 0.36 {
            0.24 - 0.17 * smooth((x - 0.2) / 0.16)
        } else {
            0.07 + 0.93 * smooth((x - 0.36) / 0.64)
        };
        match self.skipped {
            Some(at) => open.min(smooth((self.t - at) / BLINK)),
            None => open,
        }
    }

    /// 위아래 띠의 몫 0..1(1 = `BARS` 가득). 끝에서 걷힌다.
    pub fn bars(&self) -> f32 {
        1.0 - smooth((self.t - (END - BARS_OUT)) / BARS_OUT)
    }

    /// 서술 줄 `i`의 진하기 0..1.
    pub fn line(&self, i: usize) -> f32 {
        let Some(&at) = LINE_AT.get(i) else {
            return 0.0;
        };
        let a =
            smooth((self.t - at) / LINE_FADE) * (1.0 - smooth((self.t - LINES_OUT) / LINE_FADE));
        if self.skipped.is_some() {
            0.0
        } else {
            a
        }
    }

    /// `wake` 동작의 시각(0 = 엎드린 채, `CLIP` = 다 일어섬).
    pub fn pose(&self) -> f32 {
        (self.t - RISE_AT).clamp(0.0, CLIP)
    }

    /// 카메라가 다가가 있는 몫: 눈을 뜰 때 `CLOSE`, 몸이 일어서는 동안 0으로.
    pub fn close(&self) -> f32 {
        CLOSE * (1.0 - smooth(self.pose() / CLIP))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(t: f32) -> Wake {
        Wake { t, skipped: None }
    }

    /// 차례: 어둠 속의 종 → 눈이 열린다 → 첫 줄 → 몸이 일어난다 → 둘째 줄 → 글이 걷히고 띠가 걷히며 끝.
    #[test]
    fn the_scene_runs_dark_bell_eyes_rise_and_ends() {
        let mut w = Wake::default();
        let dt = 1.0 / 60.0;
        let mut cues = Vec::new();
        while !w.done() {
            match w.tick(dt) {
                Cue::None => {}
                c => cues.push((c, w.t)),
            }
        }
        assert_eq!(cues.len(), 2, "one bell, one end: {cues:?}");
        assert!(cues[0].0 == Cue::Bell && (cues[0].1 - BELL_AT).abs() < 0.02);
        assert!(cues[1].0 == Cue::Done && (cues[1].1 - END).abs() < 0.02);
        assert_eq!(w.tick(dt), Cue::None, "the end is told once");
        // 종은 어둠 속에서 운다 · 눈이 다 열린 뒤에야 몸이 움직인다 · 몸이 다 선 뒤에 끝난다.
        assert!(BELL_AT < OPEN_AT && OPEN_AT + OPEN_LEN <= RISE_AT && RISE_AT + CLIP <= END);
        assert_eq!((at(0.0).lids(), at(OPEN_AT).lids()), (0.0, 0.0));
        assert_eq!(at(OPEN_AT + OPEN_LEN).lids(), 1.0);
        assert_eq!((at(RISE_AT).pose(), at(END).pose()), (0.0, CLIP));
        // 글: 첫 줄은 눈이 열리는 동안, 둘째 줄은 몸이 일어나는 동안 뜨고, 끝 전에 다 걷힌다.
        assert!(LINE_AT[0] > OPEN_AT && LINE_AT[1] > RISE_AT);
        assert_eq!((at(LINE_AT[0]).line(0), at(LINE_AT[1]).line(1)), (0.0, 0.0));
        let mid = at(LINE_AT[1] + LINE_FADE + 0.1);
        assert_eq!((mid.line(0), mid.line(1)), (1.0, 1.0));
        assert_eq!(
            (at(END).line(0), at(END).line(1), at(END).line(2)),
            (0.0, 0.0, 0.0)
        );
        assert_eq!((at(0.0).bars(), at(END).bars()), (1.0, 0.0));
    }

    /// 눈꺼풀은 한 번 떨린다: 조금 열렸다가 도로 감기고(다 감기지는 않는다) 그 뒤로는 줄곧 열린다.
    #[test]
    fn the_lids_flutter_once_before_opening() {
        let mut v = Vec::new();
        let mut t = OPEN_AT;
        while t <= OPEN_AT + OPEN_LEN {
            v.push(at(t).lids());
            t += 0.02;
        }
        let peak = v.iter().position(|&x| x >= 0.239).expect("a first opening");
        let dip = (peak..v.len())
            .min_by(|&a, &b| v[a].total_cmp(&v[b]))
            .unwrap();
        assert!(dip > peak && v[dip] > 0.0 && v[dip] < 0.1, "{}", v[dip]);
        assert!(v[dip..].windows(2).all(|w| w[1] >= w[0]), "then only opens");
    }

    /// 카메라는 눈을 뜰 때 가깝고 몸이 서면 제 거리 — 장면이 끝나는 순간 튀지 않는다.
    #[test]
    fn the_camera_starts_close_and_is_home_by_the_end() {
        assert_eq!(at(0.0).close(), CLOSE);
        assert_eq!(at(RISE_AT).close(), CLOSE);
        assert!(at(RISE_AT + CLIP * 0.5).close() < CLOSE);
        assert_eq!(at(RISE_AT + CLIP).close(), 0.0);
    }

    /// 건너뛰면 끝의 띠 걷힘만 남는다: 글은 없고 몸은 서 있고, 그 순간 눈이 감겼다 뜬다(튐을 가린다). 두 번 눌러도 같다.
    #[test]
    fn skipping_blinks_to_the_standing_body() {
        let mut w = at(3.0);
        w.skip();
        assert!(w.t > RISE_AT + CLIP && !w.done());
        assert_eq!((w.pose(), w.close(), w.lids()), (CLIP, 0.0, 0.0));
        assert_eq!((w.line(0), w.line(1)), (0.0, 0.0));
        let t0 = w.t;
        w.skip();
        assert_eq!(w.t, t0);
        let mut n = 0;
        while w.tick(1.0 / 60.0) != Cue::Done {
            n += 1;
            assert!(n < 600);
        }
        assert_eq!((w.lids(), w.bars()), (1.0, 0.0));
        assert!((n as f32 / 60.0) < 1.0, "a skip is short");
    }
}
