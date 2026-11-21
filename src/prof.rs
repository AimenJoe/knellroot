//! 프레임 구간 시간 재기(0258 — 최적화 전에 어디에 시간이 드는지). 개발판의 PC 실행만: 환경 변수 `KNELL_PROF=<초>`면
//! 그 초마다 구간별 프레임당 평균 ms를 표준 오류로 찍는다. `let _p = prof::span("이름");` — 가드가 사라질 때 잰다(겹치면 들여 쓴다).
//! 웹(`Instant`가 없다)·운영판·켜지 않은 실행에서는 아무것도 하지 않는다.

#[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
mod imp {
    use std::cell::RefCell;
    use std::time::Instant;

    struct Row {
        name: &'static str,
        depth: usize,
        total: f64,
        max: f64,
        hits: u32,
    }

    struct State {
        every: Option<f64>,
        rows: Vec<Row>,
        depth: usize,
        frames: u32,
        since: Instant,
        frame_end: Option<Instant>,
        engine: f64,
        engine_max: f64,
    }

    thread_local! {
        static S: RefCell<State> = RefCell::new(State {
            every: std::env::var("KNELL_PROF").ok().and_then(|v| v.parse().ok()),
            rows: Vec::new(),
            depth: 0,
            frames: 0,
            since: Instant::now(),
            frame_end: None,
            engine: 0.0,
            engine_max: 0.0,
        });
    }

    pub struct Span(Option<(&'static str, usize, Instant)>);

    pub fn span(name: &'static str) -> Span {
        S.with(|s| {
            let mut s = s.borrow_mut();
            if s.every.is_none() {
                return Span(None);
            }
            let d = s.depth;
            s.depth += 1;
            Span(Some((name, d, Instant::now())))
        })
    }

    impl Drop for Span {
        fn drop(&mut self) {
            let Some((name, depth, at)) = self.0 else {
                return;
            };
            let ms = at.elapsed().as_secs_f64() * 1000.0;
            S.with(|s| {
                let mut s = s.borrow_mut();
                s.depth = s.depth.saturating_sub(1);
                match s.rows.iter_mut().find(|r| r.name == name) {
                    Some(r) => {
                        r.total += ms;
                        r.max = r.max.max(ms);
                        r.hits += 1;
                    }
                    None => s.rows.push(Row {
                        name,
                        depth,
                        total: ms,
                        max: ms,
                        hits: 1,
                    }),
                }
            });
        }
    }

    /// 프레임 들머리: 지난 프레임 끝부터 지금까지 = 엔진(`next_frame` — GL 제출·스왑·기다림).
    pub fn frame_start() {
        S.with(|s| {
            let mut s = s.borrow_mut();
            if let Some(end) = s.frame_end.take() {
                let ms = end.elapsed().as_secs_f64() * 1000.0;
                s.engine += ms;
                s.engine_max = s.engine_max.max(ms);
            }
        })
    }

    /// 프레임 끝: 셈을 하나 올리고, 때가 되면 표를 찍고 비운다.
    pub fn frame_end(label: impl FnOnce() -> String) {
        S.with(|s| {
            let mut s = s.borrow_mut();
            let Some(every) = s.every else {
                return;
            };
            s.frames += 1;
            s.frame_end = Some(Instant::now());
            let secs = s.since.elapsed().as_secs_f64();
            if secs < every {
                return;
            }
            let n = s.frames.max(1) as f64;
            let mut out = format!(
                "[prof] {} — {:.1} fps · 프레임당 ms (평균 / 최대)\n",
                label(),
                n / secs
            );
            for r in &s.rows {
                out += &format!(
                    "  {:indent$}{:<22} {:>7.2} / {:>6.2}  ×{:.1}\n",
                    "",
                    r.name,
                    r.total / n,
                    r.max,
                    r.hits as f64 / n,
                    indent = r.depth * 2
                );
            }
            out += &format!(
                "  {:<22} {:>7.2} / {:>6.2}\n",
                "engine(next_frame)",
                s.engine / n,
                s.engine_max
            );
            eprint!("{out}");
            s.rows.clear();
            s.frames = 0;
            s.engine = 0.0;
            s.engine_max = 0.0;
            s.since = Instant::now();
        })
    }
}

#[cfg(not(all(feature = "dev", not(target_arch = "wasm32"))))]
mod imp {
    pub struct Span;
    // 빈 Drop — 재는 판과 같은 꼴로 `drop(가드)`를 쓸 수 있게(clippy `drop_non_drop`).
    impl Drop for Span {
        #[inline(always)]
        fn drop(&mut self) {}
    }
    #[inline(always)]
    pub fn span(_: &'static str) -> Span {
        Span
    }
    #[inline(always)]
    pub fn frame_start() {}
    #[inline(always)]
    pub fn frame_end(_: impl FnOnce() -> String) {}
}

pub use imp::*;
