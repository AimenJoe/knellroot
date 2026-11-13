//! 장면 스택(0066 — Nystrom *Game Programming Patterns* "pushdown automata"): 열고 닫는 창은 아래 장면을 멈춰 두고,
//! 닫으면 아래가 **그 커서 그대로** 이어진다(Microsoft 패드 지침: 창을 닫으면 포커스를 연 자리로). 돌아갈 곳 깃발
//! (`from_pause` 같은)을 들고 다니지 않는다. 순수 — 앱은 `go`가 돌려준 커서·창을 그대로 쓴다.
use super::flow::{self, Go, Scene};
use crate::ui::ListView;

/// 스택 칸: 장면 + 그 화면의 커서·굴림 창(위 칸이 열려 있는 동안 맡아 둔다).
#[derive(Clone, Copy, Debug)]
pub struct Entry {
    pub scene: Scene,
    pub cursor: usize,
    pub view: ListView,
}

impl Entry {
    fn new(scene: Scene) -> Entry {
        Entry {
            scene,
            cursor: flow::focus(scene),
            view: ListView::default(),
        }
    }
}

/// 비지 않는 스택. 맨 위가 지금 장면.
#[derive(Clone, Debug)]
pub struct Stack {
    v: Vec<Entry>,
}

impl Stack {
    pub fn new(scene: Scene) -> Stack {
        Stack {
            v: vec![Entry::new(scene)],
        }
    }

    pub fn top(&self) -> Scene {
        self.v.last().expect("never empty").scene
    }

    /// 맨 아래(판인가 타이틀인가).
    pub fn base(&self) -> Scene {
        self.v[0].scene
    }

    /// 바로 아래 칸(묻는 창이 깔 화면).
    pub fn under(&self) -> Option<&Entry> {
        self.v.len().checked_sub(2).map(|i| &self.v[i])
    }

    /// 스택에 이런 장면이 있나(멈춤에서 연 창 아래인가 등 — 장면이 값을 들고 다녀 조건으로 묻는다).
    pub fn any(&self, f: impl Fn(Scene) -> bool) -> bool {
        self.v.iter().any(|e| f(e.scene))
    }

    /// 바꾼다. 지금 칸의 커서·창(`cursor`·`view`)을 받아 맡겨 두고, 바뀌었으면 새 맨 위의 커서·창을 돌려준다.
    pub fn go(&mut self, go: Go, cursor: usize, view: ListView) -> Option<(usize, ListView)> {
        let top = self.v.last_mut().expect("never empty");
        top.cursor = cursor;
        top.view = view;
        match go {
            Go::Stay => return None,
            Go::Open(s) => self.v.push(Entry::new(s)),
            // 탭 넘기기(0164): 맨 위 칸만 새 장면으로 — 아래 칸(연 자리)은 그대로라 닫으면 한 번에 거기로 간다.
            Go::Swap(s) => *self.v.last_mut().expect("never empty") = Entry::new(s),
            Go::Back if self.v.len() > 1 => {
                self.v.pop();
            }
            Go::Back => return None,
            Go::Root(s) => self.v = vec![Entry::new(s)],
        }
        let e = self.v.last().expect("never empty");
        Some((e.cursor, e.view))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flow::{Ask, Host};

    const PAUSE_SETTINGS: Scene = Scene::Settings { host: Host::Pause };

    fn paused(s: &Stack) -> bool {
        s.any(|sc| matches!(flow::tab_of(sc), Some((Host::Pause, _))))
    }

    #[test]
    fn closing_a_window_returns_to_the_row_that_opened_it() {
        let mut s = Stack::new(Scene::Title);
        // 타이틀(설정 줄 1) → 설정(조작 줄 5) → 조작.
        assert_eq!(
            s.go(
                Go::Open(Scene::Settings { host: Host::Title }),
                1,
                ListView::default()
            )
            .map(|c| c.0),
            Some(0)
        );
        s.go(Go::Open(Scene::Controls), 5, ListView::default());
        assert_eq!(s.top(), Scene::Controls);
        assert_eq!(s.base(), Scene::Title);
        // 닫으면 연 줄로: 조작 → 설정(5) → 타이틀(1).
        assert_eq!(s.go(Go::Back, 0, ListView::default()).map(|c| c.0), Some(5));
        assert_eq!(s.top(), Scene::Settings { host: Host::Title });
        assert_eq!(s.go(Go::Back, 5, ListView::default()).map(|c| c.0), Some(1));
        assert_eq!(s.top(), Scene::Title);
        // 맨 아래에서 뒤로는 그대로.
        assert!(s.go(Go::Back, 0, ListView::default()).is_none());
        assert_eq!(s.top(), Scene::Title);
    }

    /// 0164: 탭 넘기기는 맨 위 칸을 바꾼다 — 몇 번을 넘겨도 스택은 자라지 않고, 뒤로 한 번에 싸움으로 돌아간다.
    #[test]
    fn swapping_tabs_never_grows_the_stack_and_one_back_closes_the_window() {
        let mut s = Stack::new(Scene::Play);
        s.go(
            Go::Open(flow::pause_opens(flow::Tab::Status)),
            0,
            ListView::default(),
        );
        let mut seen = vec![s.top()];
        for _ in 0..12 {
            let go = flow::tab_step(s.top(), 1);
            // 새 탭은 처음 자리에서(지난 탭의 커서 3은 버린다).
            assert_eq!(s.go(go, 3, ListView::default()).map(|c| c.0), Some(0));
            seen.push(s.top());
            assert_eq!(s.under().map(|e| e.scene), Some(Scene::Play));
            assert!(paused(&s));
        }
        // 다섯 탭을 다 지났다.
        for &t in flow::tabs(Host::Pause) {
            assert!(seen.contains(&flow::tab_scene(Host::Pause, t)), "{t:?}");
        }
        s.go(Go::Back, 0, ListView::default());
        assert_eq!(s.top(), Scene::Play);
        assert!(!paused(&s));
    }

    #[test]
    fn asks_open_on_their_focus_and_lie_over_the_window_below() {
        let mut s = Stack::new(Scene::Play);
        s.go(Go::Open(PAUSE_SETTINGS), 0, ListView::default());
        let got = s.go(Go::Open(Scene::Ask(Ask::ToTitle)), 9, ListView::default());
        assert_eq!(
            got.map(|c| c.0),
            Some(1),
            "destructive asks start on cancel"
        );
        let under = s.under().expect("settings is below");
        assert_eq!((under.scene, under.cursor), (PAUSE_SETTINGS, 9));
        assert!(paused(&s), "still under the pause window");
        // 예 → 타이틀로: 스택이 비고 타이틀 하나.
        s.go(Go::Root(Scene::Title), 0, ListView::default());
        assert_eq!(s.top(), Scene::Title);
        assert!(s.under().is_none());
        assert!(!paused(&s));
    }
}
