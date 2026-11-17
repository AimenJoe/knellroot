//! 마지막 핵의 둘째 반응(0208 6절 · 0216 W3): 핵을 쥐는 순간 세계가 잠깐 멈추고, 화면 가장자리에 아주 짧은 잔상(차가운 흰빛이 두 번)과
//! 먼 종소리 한 번 — 그다음 끝 장면. 무엇인지는 말하지 않는다(lore — 글이 없다). 첫 핵의 반응(스스로 다가옴, 0105)과 다른 꼴.
//! 순수한 시간표: 앱이 틱마다 `t`를 올리고 빛의 세기·종의 순간·끝을 묻는다(`fall.rs`와 같은 꼴).

/// 반응 전체의 초(멈춘 세계 → 끝 장면).
pub const ECHO_TIME: f32 = 2.4;
/// 먼 종소리가 울리는 때(첫 잔상이 스러질 무렵).
pub const BELL_AT: f32 = 0.45;
/// 잔상 둘: (시작, 길이, 세기) — 둘째는 더 옅고 짧다("아주 짧은").
const FLASHES: [(f32, f32, f32); 2] = [(0.05, 0.42, 1.0), (1.05, 0.3, 0.55)];

/// `t`초의 잔상 세기(0..1): 빠르게 오르고 천천히 스러진다.
pub fn glow(t: f32) -> f32 {
    FLASHES
        .iter()
        .map(|&(at, len, k)| {
            let x = (t - at) / len;
            if !(0.0..1.0).contains(&x) {
                0.0
            } else if x < 0.15 {
                k * x / 0.15
            } else {
                k * (1.0 - (x - 0.15) / 0.85).powi(2)
            }
        })
        .fold(0.0, f32::max)
}

/// 이번 틱(`t` → `t + dt`)에 종이 울리나.
pub fn bell(t: f32, dt: f32) -> bool {
    t < BELL_AT && t + dt >= BELL_AT
}

pub fn done(t: f32) -> bool {
    t >= ECHO_TIME
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 잔상은 둘 — 처음은 밝고 둘째는 옅다, 그 사이와 끝은 꺼져 있다. 종은 한 번, 끝은 그 뒤.
    #[test]
    fn two_flashes_one_bell_then_the_end() {
        let dt = 1.0 / 60.0;
        let samples: Vec<f32> = (0..(ECHO_TIME / dt) as usize)
            .map(|i| glow(i as f32 * dt))
            .collect();
        let peak = |a: f32, b: f32| {
            samples[(a / dt) as usize..(b / dt) as usize]
                .iter()
                .cloned()
                .fold(0.0, f32::max)
        };
        let (first, second) = (peak(0.0, 0.6), peak(1.0, 1.5));
        assert!(
            first > 0.9 && second > 0.4 && second < first,
            "{first} {second}"
        );
        assert_eq!(glow(0.8), 0.0, "dark between");
        assert_eq!(glow(ECHO_TIME - 0.1), 0.0, "dark before the end");
        let bells = (0..(ECHO_TIME / dt) as usize)
            .filter(|&i| bell(i as f32 * dt, dt))
            .count();
        assert_eq!(bells, 1);
        assert!(!done(ECHO_TIME - 0.01) && done(ECHO_TIME));
        assert!(samples.iter().all(|g| (0.0..=1.0).contains(g)));
    }
}
