//! 싸움 화면의 연출 칸(0251 R5): 시간이 흐르며 세계를 멈추거나 빈 입력으로 흘리는 장면 다섯 — 뿌리 굴 전환(0174)·마지막 핵의
//! 반응(0216 W3)·쓰러짐(0175)·깨어나는 첫 장면(0171)·꺼진 화로에 불이 붙는 순간(0171).
//!
//! 예전에는 앱에 칸이 다섯 따로 있었고(`passage`·`echo`·`fall`·`wake`·`kindle`), 무엇이 먼저인지는 `tick_play` 머리의 if 차례에만
//! 있었다. 이제 한 칸(`Option<Interlude>`)이라 둘이 함께 돌 수 없고, 한 틱의 일은 순수한 `Interlude::step`이 `Beat`로 말한다 —
//! 앱(`App::tick_interlude`)은 그대로 한다(소리·카메라·월드·장면). 각 장면의 시간표는 제 모듈(`descent`·`core_echo`·`fall`·`wake`)에 그대로.
use crate::input::Actions;

/// 꺼진 화로에 불이 붙고 창이 열리기까지(초) — 붙는 것을 본 뒤에 창이 화면을 덮는다.
pub const KINDLE_TIME: f32 = 1.5;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Interlude {
    /// 지역 사이 뿌리 굴(0174) — 아무 버튼 = 건너뛰기, 다 어두운 순간에 지역을 바꾼다.
    Passage(crate::descent::Passage),
    /// 마지막 핵의 반응(0216 W3): 지난 초 — 세계가 멈춘 채 잔상·먼 종소리 → 끝 장면.
    Echo(f32),
    /// 쓰러짐(0175) — 다 어두울 때 쉼터 곁으로, 일어나는 동안만 건너뛴다.
    Fall(crate::fall::Fall),
    /// 깨어나는 첫 장면(0171) — 멈춤 버튼(터치는 싸움 버튼 자리) = 건너뛰기.
    Wake(crate::wake::Wake),
    /// 꺼진 화로에 불이 붙는 순간(0171): 남은 초.
    Kindle(f32),
    /// "성소에 들다"(0262 — 서막 숲 끝에서 한 번): 다 어두울 때 지역을 성소로, 높이 보는 카메라·몇 걸음·이름 카드. 멈춤 버튼 = 건너뛰기.
    Arrive(crate::arrive::Arrive),
}

/// 이 틱에 세계를 어떻게 하나.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Hold {
    /// 건드리지 않는다(굴 — 그동안 월드는 그리지도 않는다).
    #[default]
    Untouched,
    /// 멈춘다: 보간이 지난 틱과 이번 틱 사이를 오가지 않게 서 있는 자리로 + 카메라만 따라간다(히트스톱과 같다).
    Freeze,
    /// 빈 입력으로 한 틱 흘린다(방랑자는 서 있다).
    Idle,
    /// 성소 쪽(북, 월드 −z)으로 이 세기만큼 걷는 입력으로 한 틱(0262 "성소에 들다" — 앱이 지금 입력 기준의 스틱으로 바꾼다).
    Walk(f32),
    /// 빈 입력으로 한 틱 + 스테이지도 한 틱(화로 곁 — 불이 붙는 동안 쉼터가 빛난다).
    IdleStage,
}

/// 연출이 끝나며 앱이 할 일.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Then {
    #[default]
    Nothing,
    /// 쓰러짐의 어둠 속: 쉼터 곁으로 옮긴다(`App::rise_after_fall`) — 연출은 남는다(일어나는 몫).
    Rise,
    /// 화로의 창을 연다.
    Hearth,
    /// 끝 장면으로.
    Ending,
    /// 깨어나는 장면이 끝났다 — 기록에 적는다.
    Woke,
}

/// 한 틱의 일(앱이 이 차례로 한다: 카메라 당김 → 시작 글 붙잡기 → 지역 바꾸기 → 소리 → 세계 → 당김 풀기 → 끝의 일).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Beat {
    /// 카메라 당김(`Camera::close`)을 이 값으로(세계를 움직이기 전에).
    pub close: Option<f32>,
    /// 스테이지의 시작 글을 붙잡아 둔다(깨어나는 동안 흐르지 않게).
    pub hold_intro: bool,
    /// 지역을 바꾼다(닿을 지역, 오르는 길인가) — 굴이 다 어두운 순간.
    pub cross: Option<(usize, bool)>,
    /// 서막 숲에서 성소로(0262) — 다 어두운 순간. 기록 `prologue`도 그때.
    pub arrive: bool,
    /// 숲 끝의 성소 바깥 문이 열린 몫 0..1(0265 — 문 앞 장면 동안만, 앱이 문의 여는 동작에).
    pub gate: Option<f32>,
    /// 그 문이 열리기 시작한다(한 번 — 막는 상자를 걷는다).
    pub open_gate: bool,
    pub sound: Option<crate::sfx::Id>,
    pub hold: Hold,
    /// 세계를 움직인 뒤 당김을 푼다(0).
    pub release_close: bool,
    pub then: Then,
}

/// 건너뛰기에 쓰는 누름과 장치(깨어나는 장면은 터치일 때만 싸움 버튼도 건너뛴다).
#[derive(Clone, Copy, Debug, Default)]
pub struct Input {
    pub actions: Actions,
    pub touch: bool,
}

impl Interlude {
    /// 한 틱. 돌려주는 것: 다음 틱의 연출(끝났으면 None)과 이 틱의 일.
    pub fn step(self, input: Input, dt: f32) -> (Option<Interlude>, Beat) {
        let a = &input.actions;
        let any = a.attack || a.jump || a.dodge || a.confirm || a.pause;
        match self {
            Interlude::Passage(mut p) => {
                if any {
                    p.skip();
                }
                let tick = p.step(dt);
                let beat = Beat {
                    cross: tick.swap.then_some((p.to, p.up)),
                    sound: tick.bell.then_some(crate::sfx::Id::Knell),
                    ..Beat::default()
                };
                ((!tick.done).then_some(Interlude::Passage(p)), beat)
            }
            Interlude::Echo(t) => {
                let beat = Beat {
                    hold: Hold::Freeze,
                    sound: crate::core_echo::bell(t, dt).then_some(crate::sfx::Id::Knell),
                    ..Beat::default()
                };
                let t = t + dt;
                if crate::core_echo::done(t) {
                    (
                        None,
                        Beat {
                            then: Then::Ending,
                            ..beat
                        },
                    )
                } else {
                    (Some(Interlude::Echo(t)), beat)
                }
            }
            Interlude::Fall(mut fl) => {
                if any {
                    fl.skip();
                }
                let cue = fl.tick(dt);
                let beat = Beat {
                    sound: cue.rumble.then_some(crate::sfx::Id::Wake),
                    close: Some(fl.close()),
                    hold: if fl.moved() { Hold::Idle } else { Hold::Freeze },
                    ..Beat::default()
                };
                if cue.moved {
                    (
                        Some(Interlude::Fall(fl)),
                        Beat {
                            then: Then::Rise,
                            ..beat
                        },
                    )
                } else if cue.done || fl.done() {
                    (
                        None,
                        Beat {
                            release_close: true,
                            ..beat
                        },
                    )
                } else {
                    (Some(Interlude::Fall(fl)), beat)
                }
            }
            Interlude::Wake(mut wk) => {
                if a.pause || (input.touch && (a.attack || a.jump || a.dodge)) {
                    wk.skip();
                }
                let cue = wk.tick(dt);
                let done = cue == crate::wake::Cue::Done;
                let beat = Beat {
                    sound: (cue == crate::wake::Cue::Bell).then_some(crate::sfx::Id::Knell),
                    close: Some(wk.close()),
                    hold_intro: true,
                    hold: Hold::Idle,
                    release_close: done,
                    then: if done { Then::Woke } else { Then::Nothing },
                    ..Beat::default()
                };
                ((!wk.done()).then_some(Interlude::Wake(wk)), beat)
            }
            Interlude::Arrive(mut ar) => {
                if a.pause || (input.touch && (a.attack || a.jump || a.dodge)) {
                    ar.skip();
                }
                let cue = ar.tick(dt);
                let beat = Beat {
                    arrive: cue.swap,
                    gate: (!ar.arrived()).then(|| ar.gate()),
                    open_gate: cue.open,
                    // 문이 열리는 소리는 땅울림(문지기가 깨는 소리와 같은 낮은 돌 소리), 종은 종.
                    sound: if cue.open {
                        Some(crate::sfx::Id::Wake)
                    } else {
                        cue.bell.then_some(crate::sfx::Id::Knell)
                    },
                    hold: Hold::Walk(ar.walk()),
                    ..Beat::default()
                };
                ((!ar.done()).then_some(Interlude::Arrive(ar)), beat)
            }
            Interlude::Kindle(left) => {
                let left = left - dt;
                let beat = Beat {
                    hold: Hold::IdleStage,
                    ..Beat::default()
                };
                if left <= 0.0 {
                    (
                        None,
                        Beat {
                            then: Then::Hearth,
                            ..beat
                        },
                    )
                } else {
                    (Some(Interlude::Kindle(left)), beat)
                }
            }
        }
    }

    /// 세로로 든 폰(0082)이면 이 연출은 이 틱을 쉬고 멈춤 창이 먼저인가 — 깨어나는 장면만(예전 차례 그대로: 다른 연출은 세로여도 흐른다).
    pub fn waits_for_landscape(&self) -> bool {
        matches!(self, Interlude::Wake(_) | Interlude::Arrive(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tuning::TICK_DT;

    fn run(mut i: Interlude, input: Input, max: usize) -> (Vec<Beat>, Option<Interlude>) {
        let mut beats = Vec::new();
        for _ in 0..max {
            let (next, b) = i.step(input, TICK_DT);
            beats.push(b);
            match next {
                Some(n) => i = n,
                None => return (beats, None),
            }
        }
        (beats, Some(i))
    }

    fn press(f: impl Fn(&mut Actions)) -> Input {
        let mut a = Actions::default();
        f(&mut a);
        Input {
            actions: a,
            touch: false,
        }
    }

    /// 굴: 지역은 한 번만, 다 어두운 순간에 바꾸고, 세계는 건드리지 않는다. 아무 버튼이나 건너뛴다(더 빨리 끝난다).
    #[test]
    fn a_passage_swaps_once_leaves_the_world_alone_and_any_button_hurries_it() {
        let p = crate::descent::Passage::new(1, false, 0, 40);
        let (beats, left) = run(Interlude::Passage(p), Input::default(), 100_000);
        assert!(left.is_none(), "the passage ends");
        assert_eq!(beats.iter().filter(|b| b.cross.is_some()).count(), 1);
        assert!(beats
            .iter()
            .all(|b| b.hold == Hold::Untouched && b.then == Then::Nothing));
        let (skipped, _) = run(Interlude::Passage(p), press(|a| a.jump = true), 100_000);
        assert!(
            skipped.len() < beats.len(),
            "{} < {}",
            skipped.len(),
            beats.len()
        );
        assert_eq!(skipped.iter().filter(|b| b.cross.is_some()).count(), 1);
    }

    /// 핵의 반응: 세계는 멈추고, 종은 한 번, 끝나면 끝 장면.
    #[test]
    fn the_core_echo_freezes_rings_once_and_ends_in_the_ending() {
        let (beats, left) = run(Interlude::Echo(0.0), Input::default(), 100_000);
        assert!(left.is_none());
        assert!(beats.iter().all(|b| b.hold == Hold::Freeze));
        assert_eq!(beats.iter().filter(|b| b.sound.is_some()).count(), 1);
        assert_eq!(beats.last().unwrap().then, Then::Ending);
        assert_eq!(beats.iter().filter(|b| b.then != Then::Nothing).count(), 1);
    }

    /// 쓰러짐: 옮기기 전에는 세계가 멈추고(방랑자만 무너진다), 쉼터 곁으로 한 번 옮긴 뒤로는 흐른다. 끝나면 당김을 푼다.
    #[test]
    fn a_fall_freezes_until_it_moves_once_then_lets_the_world_run() {
        let (beats, left) = run(
            Interlude::Fall(crate::fall::Fall::default()),
            Input::default(),
            100_000,
        );
        assert!(left.is_none());
        let rise = beats
            .iter()
            .position(|b| b.then == Then::Rise)
            .expect("moves once");
        assert_eq!(beats.iter().filter(|b| b.then == Then::Rise).count(), 1);
        assert!(
            beats[..rise].iter().all(|b| b.hold == Hold::Freeze),
            "the world waits while the roots close in"
        );
        assert!(
            beats[rise + 1..].iter().all(|b| b.hold == Hold::Idle),
            "then rises by the hearth"
        );
        assert!(beats.last().unwrap().release_close);
        assert!(beats.iter().all(|b| b.close.is_some()));
    }

    /// 깨어나는 장면: 시작 글을 붙잡고 세계는 빈 입력으로 흐른다. 키보드의 공격은 건너뛰지 않고, 멈춤 버튼은 건너뛴다.
    /// 터치(버튼이 걷혀 있다)는 싸움 버튼 자리 어디든 건너뛴다. 끝나는 틱에 한 번 "깨었다".
    #[test]
    fn waking_skips_on_pause_and_on_touch_fight_buttons_only() {
        let w = Interlude::Wake(crate::wake::Wake::default());
        let (plain, left) = run(w, Input::default(), 100_000);
        assert!(left.is_none());
        assert!(plain.iter().all(|b| b.hold_intro && b.hold == Hold::Idle));
        assert_eq!(plain.iter().filter(|b| b.then == Then::Woke).count(), 1);
        assert_eq!(plain.last().unwrap().then, Then::Woke);
        let (keys, _) = run(w, press(|a| a.attack = true), 100_000);
        assert_eq!(keys.len(), plain.len(), "a keyboard attack does not skip");
        let (paused, _) = run(w, press(|a| a.pause = true), 100_000);
        assert!(paused.len() < plain.len());
        let touch = Input {
            touch: true,
            ..press(|a| a.attack = true)
        };
        let (touched, _) = run(w, touch, 100_000);
        assert!(touched.len() < plain.len());
        assert!(w.waits_for_landscape());
        assert!(!Interlude::Echo(0.0).waits_for_landscape());
    }

    /// 성소에 들다(0262): 지역은 한 번만 바꾸고(다 어두운 순간), 세계는 걷는 입력으로 흐른다. 멈춤 버튼이 건너뛴다.
    #[test]
    fn arriving_swaps_once_walks_the_world_and_skips_on_pause() {
        let a = Interlude::Arrive(crate::arrive::Arrive::default());
        let (plain, left) = run(a, Input::default(), 100_000);
        assert!(left.is_none());
        assert_eq!(plain.iter().filter(|b| b.arrive).count(), 1);
        assert!(plain.iter().all(|b| matches!(b.hold, Hold::Walk(_))));
        // 0265: 문 앞의 종 · 문이 열리는 땅울림 · 성소의 종. 문은 한 번 열리고, 문 앞 동안만 여는 몫을 넘긴다.
        assert_eq!(plain.iter().filter(|b| b.sound.is_some()).count(), 3);
        assert_eq!(plain.iter().filter(|b| b.open_gate).count(), 1);
        let swap = plain.iter().position(|b| b.arrive).unwrap();
        assert!(plain[..swap].iter().all(|b| b.gate.is_some()));
        assert!(plain[swap + 1..].iter().all(|b| b.gate.is_none()));
        let (keys, _) = run(a, press(|a| a.attack = true), 100_000);
        assert_eq!(keys.len(), plain.len(), "a keyboard attack does not skip");
        let (paused, _) = run(a, press(|a| a.pause = true), 100_000);
        assert!(paused.len() < plain.len());
        assert_eq!(paused.iter().filter(|b| b.arrive).count(), 1);
        assert!(a.waits_for_landscape());
    }

    /// 불 붙는 순간: 세계와 스테이지가 흐르고, `KINDLE_TIME` 뒤에 한 번 화로의 창.
    #[test]
    fn kindling_runs_the_stage_then_opens_the_hearth_once() {
        let (beats, left) = run(Interlude::Kindle(KINDLE_TIME), Input::default(), 10_000);
        assert!(left.is_none());
        assert!(beats.iter().all(|b| b.hold == Hold::IdleStage));
        assert_eq!(beats.iter().filter(|b| b.then == Then::Hearth).count(), 1);
        let ticks = (KINDLE_TIME / TICK_DT).round() as usize;
        assert!(
            beats.len().abs_diff(ticks) <= 1,
            "{} ≈ {ticks}",
            beats.len()
        );
    }
}
