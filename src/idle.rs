//! 서 있는 동안의 몸짓(0153, 사장 "아이들 모션 같은 것도 넣어 주고"): 숨 쉬는 고리(`idle`) 위에, 가만히 서 있으면 가끔
//! 한 번짜리 몸짓(둘러보기 · 털기 · 종 만지기)을 튼다. 렌더가 들고 있는 그림 상태다 — 월드는 모른다(`secondary.rs`와 같은 자리).
//!
//! 규칙(KB `concepts/believable-cute-character-motion.md` "쉬기의 2차 동작은 1차 동작을 흐리지 않게"):
//! - 몸짓은 **조용할 때만**(가까이 깨어 있는 적이 없고 락온이 아닐 때) 시작한다 — 싸움 자세를 흐리지 않는다.
//! - 움직이거나 다른 동작에 들어가면 그 자리에서 끊긴다(입력이 먼저다). 다시 서면 처음부터 센다.
//! - 숨 고리의 **끝에서만** 시작한다: 몸짓은 숨 고리의 0초 자세에서 시작해 거기로 돌아오게 그렸다(동작 사이 보간이 없어도 안 튄다).
//! - 같은 몸짓을 되풀이하지 않고 표 순서로 돌린다, 사이 간격도 조금씩 다르게(정해진 차례 — 난수 없음, 시험이 본다).

/// 몸짓 동작 이름(`wanderer.glb`) — 이 차례로 돌린다. 모델에 없는 것은 건너뛴다(길이 0).
pub const FIDGETS: [&str; 3] = ["idle_look", "idle_shake", "idle_bell"];
/// 서고 나서 첫 몸짓까지(초)와 그 뒤 몸짓 사이(초 — `GAP + GAP_STEP × (몇째 % GAP_WRAP)`).
pub const FIRST: f32 = 5.0;
pub const GAP: f32 = 8.0;
pub const GAP_STEP: f32 = 2.5;
pub const GAP_WRAP: u32 = 3;

/// 이번 프레임에 틀 동작.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Pose {
    /// 숨 고리의 `t`초.
    Loop(f32),
    /// `FIDGETS[i]`의 `t`초.
    Fidget(usize, f32),
}

#[derive(Clone, Copy, Debug)]
pub struct Idle {
    /// 서 있은 지(초).
    still: f32,
    /// 숨 고리의 시계 — 설 때·몸짓이 끝날 때 0부터.
    clock: f32,
    /// 도는 몸짓(번호, 지난 초).
    act: Option<(usize, f32)>,
    /// 다음 몸짓을 틀 수 있는 때(`still` 기준).
    next_at: f32,
    /// 지금까지 튼 몸짓 수(차례·간격).
    count: u32,
}

impl Default for Idle {
    fn default() -> Self {
        Idle {
            still: 0.0,
            clock: 0.0,
            act: None,
            next_at: FIRST,
            count: 0,
        }
    }
}

impl Idle {
    /// 프레임마다. `standing` = 서서 아무것도 안 한다, `calm` = 몸짓을 틀어도 되는 때, `loop_dur` = 숨 고리 길이,
    /// `durs` = 몸짓마다 길이(없으면 0).
    pub fn step(
        &mut self,
        dt: f32,
        standing: bool,
        calm: bool,
        loop_dur: f32,
        durs: [f32; FIDGETS.len()],
    ) -> Pose {
        if !standing {
            *self = Idle {
                count: self.count,
                ..Idle::default()
            };
            return Pose::Loop(0.0);
        }
        self.still += dt;
        if let Some((i, t)) = self.act {
            let t = t + dt;
            if calm && t < durs[i] {
                self.act = Some((i, t));
                return Pose::Fidget(i, t);
            }
            // 끝났거나(숨 고리의 0초로 돌아온다) 적이 다가와 끊겼다.
            self.act = None;
            self.clock = 0.0;
            self.next_at = self.still + GAP + GAP_STEP * (self.count % GAP_WRAP) as f32;
            return Pose::Loop(0.0);
        }
        let before = self.clock;
        self.clock += dt;
        let lp = loop_dur.max(1e-3);
        let wrapped = (self.clock / lp).floor() > (before / lp).floor();
        if calm && wrapped && self.still >= self.next_at {
            // 표 순서로, 모델에 있는 다음 몸짓.
            let n = FIDGETS.len();
            let first = self.count as usize % n;
            if let Some(i) = (0..n).map(|k| (first + k) % n).find(|i| durs[*i] > 0.0) {
                self.count += 1;
                self.act = Some((i, 0.0));
                return Pose::Fidget(i, 0.0);
            }
        }
        Pose::Loop(self.clock)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 60.0;
    const LOOP: f32 = 2.4;
    const DURS: [f32; 3] = [2.6, 1.3, 2.2];

    /// `secs`초 동안 돌려 시작된 몸짓의 (번호, 시작한 초)를 모은다.
    fn run(idle: &mut Idle, secs: f32, calm: bool) -> Vec<(usize, f32)> {
        let mut out = Vec::new();
        let mut was = None;
        for k in 0..(secs / DT) as usize {
            let p = idle.step(DT, true, calm, LOOP, DURS);
            let now = match p {
                Pose::Fidget(i, _) => Some(i),
                Pose::Loop(_) => None,
            };
            if now.is_some() && was.is_none() {
                out.push((now.unwrap(), k as f32 * DT));
            }
            was = now;
        }
        out
    }

    /// 가만히 서 있으면 몇 초 뒤 첫 몸짓이 나오고, 그 뒤로는 표 순서로 하나씩 — 같은 것을 잇달아 틀지 않는다.
    /// 몸짓은 숨 고리의 끝에서만 시작한다(이음매).
    #[test]
    fn standing_still_plays_fidgets_in_turn_at_loop_ends() {
        let mut idle = Idle::default();
        let got = run(&mut idle, 60.0, true);
        assert!(got.len() >= 4, "{got:?}");
        assert!(
            got[0].1 >= FIRST && got[0].1 < FIRST + LOOP + 0.1,
            "{got:?}"
        );
        for (k, (i, _)) in got.iter().enumerate() {
            assert_eq!(*i, k % FIDGETS.len(), "in table order");
        }
        for w in got.windows(2) {
            let gap = w[1].1 - w[0].1 - DURS[w[0].0];
            assert!(gap >= GAP - 0.1, "rests between fidgets: {gap}");
        }
        // 첫 몸짓은 숨 고리가 온 바퀴를 채운 때(서자마자 0부터 센 고리의 배수).
        let phase = got[0].1 % LOOP;
        assert!(phase < 2.0 * DT || LOOP - phase < 2.0 * DT, "{phase}");
    }

    /// 움직이면 그 자리에서 끊기고, 다시 서면 처음부터 센다(다음 몸짓은 이어지는 차례).
    #[test]
    fn moving_cuts_a_fidget_and_standing_again_starts_the_wait_over() {
        let mut idle = Idle::default();
        let got = run(&mut idle, 9.0, true);
        assert_eq!(got.len(), 1);
        assert!(matches!(
            idle.step(DT, true, true, LOOP, DURS),
            Pose::Fidget(0, _)
        ));
        assert_eq!(idle.step(DT, false, true, LOOP, DURS), Pose::Loop(0.0));
        let again = run(&mut idle, FIRST - 0.2, true);
        assert!(again.is_empty(), "waits again: {again:?}");
        let next = run(&mut idle, LOOP + 0.5, true);
        assert_eq!(next.first().map(|x| x.0), Some(1), "the next one in turn");
    }

    /// 조용하지 않으면(적이 가까이 깨어 있다·락온) 몸짓을 시작하지 않고, 돌던 몸짓도 끊는다 — 숨 고리는 계속 돈다.
    #[test]
    fn no_fidgets_while_enemies_are_near() {
        let mut idle = Idle::default();
        assert!(run(&mut idle, 40.0, false).is_empty());
        assert!(matches!(idle.step(DT, true, false, LOOP, DURS), Pose::Loop(t) if t > 0.0));
        let mut idle = Idle::default();
        run(&mut idle, 9.0, true);
        assert!(matches!(
            idle.step(DT, true, true, LOOP, DURS),
            Pose::Fidget(..)
        ));
        assert_eq!(idle.step(DT, true, false, LOOP, DURS), Pose::Loop(0.0));
    }

    /// 모델에 없는 몸짓(길이 0)은 건너뛴다 — 하나도 없으면 숨 고리만.
    #[test]
    fn missing_fidgets_are_skipped() {
        let mut idle = Idle::default();
        let mut started = Vec::new();
        for _ in 0..(40.0 / DT) as usize {
            if let Pose::Fidget(i, t) = idle.step(DT, true, true, LOOP, [0.0, 1.3, 0.0]) {
                if t == 0.0 {
                    started.push(i);
                }
            }
        }
        assert!(started.len() >= 2 && started.iter().all(|i| *i == 1));
        let mut idle = Idle::default();
        for _ in 0..(30.0 / DT) as usize {
            assert!(matches!(
                idle.step(DT, true, true, LOOP, [0.0; 3]),
                Pose::Loop(_)
            ));
        }
    }
}
