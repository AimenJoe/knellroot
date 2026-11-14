//! 방랑자의 걸음 시계와 동작 섞기(0175) — 렌더가 들고 있는 순수 규칙(월드는 모른다).
//!
//! 사장 "가만히 있다가 첫 몇 프레임 움직일 때 덜그덕거리는 느낌이 있어. 길게 움직일 땐 괜찮아"(2026-11-14). 까닭 둘:
//! 1. 걷기 동작의 시각을 `월드 시간 × 빠르기`로 셈했다 — 빠르기가 그대로면 고르게 흐르지만, 가속하는 첫 몇 프레임에 빠르기가
//!    바뀌면 시각이 `월드 시간 × 바뀐 몫`만큼(월드 시간이 100초면 수십 초) 걸음 주기의 엉뚱한 자리로 튀었다. 다 빨라진 뒤에는
//!    빠르기가 그대로라 멀쩡했다 — "길게 움직일 땐 괜찮다". 여기서는 시각을 **쌓는다**(지난 시각 + 이번 틱 × 빠르기).
//! 2. 서기 → 걷기에 섞임이 없어 한 프레임에 자세가 바뀌었다. 조용한 동작(서기·걷기·달리기·옆걸음·몸짓)으로 들어갈 때만
//!    짧게 섞는다 — 공격·회피·패링처럼 누르는 순간 보여야 하는 동작은 섞지 않는다(늦춰 보이면 손맛이 무너진다).

/// 조용한 동작으로 들어갈 때 섞는 초.
pub const BLEND: f32 = 0.14;

/// 섞어 들어가는 동작: 서기·걷기·달리기·옆걸음·뒷걸음·서 있는 몸짓.
pub fn calm(name: &str) -> bool {
    matches!(
        name,
        "idle" | "walk" | "run" | "strafe_l" | "strafe_r" | "walk_back"
    ) || name.starts_with("idle_")
}

/// 걸음 시계: 걷는 동안 이번 틱 × 빠르기를 쌓는다. 걷기 시작하는 프레임에는 걸음의 첫 자세(0초)에서.
#[derive(Clone, Copy, Debug, Default)]
pub struct Gait {
    pub phase: f32,
    walking: bool,
}

impl Gait {
    /// 한 프레임: `dt` = 월드 시계로 지난 초(히트스톱·멈춤 동안 0), `moving` = 걷거나 달리나, `rate` = 동작 빠르기 배.
    /// 돌려주는 것 = 걷기 동작의 시각(초).
    pub fn advance(&mut self, dt: f32, moving: bool, rate: f32) -> f32 {
        if moving {
            if !self.walking {
                self.phase = 0.0;
            }
            self.phase += dt.max(0.0) * rate.max(0.0);
        }
        self.walking = moving;
        self.phase
    }
}

/// 섞이는 동안의 무게(0 = 앞 자세, 1 = 새 동작): 남은 초 `left`에서. 부드럽게(시작·끝에서 느리게).
pub fn weight(left: f32) -> f32 {
    let x = (1.0 - left / BLEND).clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 0175: 가속하는 동안(빠르기가 프레임마다 바뀐다) 걸음 시각은 한 프레임에 그 프레임의 몫만큼만 간다 — 예전 셈(월드 시간 ×
    /// 빠르기)은 월드 시간이 크면 한 프레임에 몇 초씩 걸음 주기를 건너뛰었다.
    #[test]
    fn the_walk_clock_never_jumps_while_speeding_up() {
        let mut g = Gait::default();
        let dt = 1.0 / 60.0;
        let mut world_time = 100.0f32;
        let (mut last, mut old_last) = (g.advance(dt, true, 0.6), world_time * 0.6);
        assert!(last < dt, "a fresh walk starts at the first pose: {last}");
        let mut old_jump = 0.0f32;
        for i in 1..30 {
            world_time += dt;
            let rate = (0.6 + 0.05 * i as f32).min(1.6);
            let now = g.advance(dt, true, rate);
            assert!(
                now > last && now - last <= dt * 1.6 + 1e-5,
                "{last} → {now}"
            );
            last = now;
            // 예전 셈.
            let old = world_time * rate;
            old_jump = old_jump.max((old - old_last).abs());
            old_last = old;
        }
        assert!(
            old_jump > 1.0,
            "the old clock jumped {old_jump}s in one frame"
        );
        // 멈췄다 다시 걸으면 처음 자세부터.
        g.advance(dt, false, 0.0);
        assert!(g.advance(dt, true, 1.0) <= dt + 1e-6);
        // 히트스톱(월드 시계가 멈춤): 시각도 선다.
        let t = g.phase;
        assert_eq!(g.advance(0.0, true, 1.0), t);
    }

    /// 0175: 섞는 것은 조용한 동작으로 들어갈 때만 — 누르는 순간 보여야 하는 동작은 바로.
    #[test]
    fn only_calm_clips_blend_in_and_the_weight_runs_zero_to_one() {
        for n in [
            "idle",
            "walk",
            "run",
            "strafe_l",
            "walk_back",
            "idle_look",
            "idle_bell",
        ] {
            assert!(calm(n), "{n}");
        }
        for n in [
            "attack", "attack2", "dodge", "parry", "heavy", "hurt", "cast", "jump", "wake",
        ] {
            assert!(!calm(n), "{n}");
        }
        assert_eq!(weight(BLEND), 0.0);
        assert_eq!(weight(0.0), 1.0);
        let mid = weight(BLEND * 0.5);
        assert!((mid - 0.5).abs() < 1e-5);
    }
}
