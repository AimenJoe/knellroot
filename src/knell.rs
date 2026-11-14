//! 종소리의 길 안내(0171 — 세계 법칙 D "방랑자는 땅 밑 종소리를 따라간다"): 문을 찾는 동안, 가끔 종이 울리고 그 소리가
//! 온 쪽의 화면 가장자리가 잠깐 물결친다. 화살(갈 곳 — 호박색)이 아니다: 늘 떠 있지 않고, 가리키는 것이 아니라 들린다.
//!
//! 순수 규칙(시계 하나)만 여기에 — 무엇이 "찾는 중"인지는 스테이지(`StageRun::bell_from`)가, 소리·그림은 앱이 맡는다.

/// 찾기 시작한 뒤 첫 종까지(초) — 시작 글·튜토리얼의 첫 걸음이 지난 뒤.
pub const FIRST: f32 = 12.0;
/// 종과 종 사이(초). 환경음의 먼 종(12초 고리)과 맞물리지 않게 다른 길이.
pub const EVERY: f32 = 17.0;
/// 물결이 보이는 시간(초).
pub const SHOW: f32 = 3.2;
/// 이 거리(m) 안이면 울리지 않는다 — 장치가 눈앞이다.
pub const NEAR: f32 = 9.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Knell {
    /// 다음 종까지 남은 시간.
    wait: f32,
    /// 물결이 남은 시간(0이면 안 보인다).
    pub show: f32,
}

impl Default for Knell {
    fn default() -> Self {
        Knell {
            wait: FIRST,
            show: 0.0,
        }
    }
}

impl Knell {
    /// 한 틱. `lost` = 찾을 문 장치가 있고 다른 길 안내가 없다(멀리 있다), `fight` = 깨어 있는 적이 곁에 있다.
    /// 종이 울린 틱이면 참. 찾는 중이 아니면 시계가 처음으로 돌아가고(다시 찾게 되면 `FIRST` 뒤에), 싸우는 동안은 시계가
    /// 멈춘다(싸움 한가운데에 울리지 않는다 — 끝나면 남은 만큼 뒤에).
    pub fn tick(&mut self, dt: f32, lost: bool, fight: bool) -> bool {
        self.show = (self.show - dt).max(0.0);
        if !lost {
            *self = Knell::default();
            return false;
        }
        if fight {
            return false;
        }
        self.wait -= dt;
        if self.wait > 0.0 {
            return false;
        }
        self.wait = EVERY;
        self.show = SHOW;
        true
    }

    /// 물결의 진행 0..1(안 보이면 없음).
    pub fn wave(&self) -> Option<f32> {
        (self.show > 0.0).then(|| 1.0 - self.show / SHOW)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(k: &mut Knell, secs: f32, lost: bool, fight: bool) -> u32 {
        let dt = 1.0 / 60.0;
        (0..(secs / dt) as usize)
            .filter(|_| k.tick(dt, lost, fight))
            .count() as u32
    }

    /// 찾는 동안: 처음은 `FIRST` 뒤, 그다음부터 `EVERY`마다 — 울릴 때마다 물결이 `SHOW`초.
    #[test]
    fn the_bell_tolls_now_and_then_while_the_door_is_unfound() {
        let mut k = Knell::default();
        assert_eq!(run(&mut k, FIRST - 0.5, true, false), 0);
        assert_eq!(k.wave(), None);
        assert_eq!(run(&mut k, 1.0, true, false), 1);
        assert!(k.wave().is_some_and(|w| w < 0.3));
        assert_eq!(run(&mut k, SHOW, true, false), 0);
        assert_eq!(k.wave(), None, "the ripple fades");
        assert_eq!(run(&mut k, EVERY * 3.0, true, false), 3);
    }

    /// 싸우는 동안은 울리지 않고 시계도 멈춘다 — 싸움이 끝나면 남은 만큼 뒤에 운다.
    #[test]
    fn no_toll_in_a_fight_and_the_clock_holds() {
        let mut k = Knell::default();
        run(&mut k, FIRST - 1.0, true, false);
        assert_eq!(run(&mut k, 60.0, true, true), 0);
        assert_eq!(run(&mut k, 0.5, true, false), 0, "not the moment it ends");
        assert_eq!(run(&mut k, 1.0, true, false), 1);
    }

    /// 찾는 중이 아니면(문이 열렸다·다른 길 안내가 있다·장치가 눈앞이다) 울리지 않고, 다시 찾게 되면 처음부터 센다.
    #[test]
    fn it_is_quiet_when_there_is_nothing_to_seek() {
        let mut k = Knell::default();
        assert_eq!(run(&mut k, 120.0, false, false), 0);
        run(&mut k, FIRST + 0.5, true, false);
        assert!(k.wave().is_some());
        k.tick(1.0 / 60.0, false, false);
        assert_eq!(k, Knell::default(), "the ripple goes with the search");
        assert_eq!(run(&mut k, FIRST - 0.5, true, false), 0);
    }
}
