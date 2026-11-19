//! 앱이 내려갔다(0257, nail 0298 꼴): 엔진이 내는 "최소화" 사건 — Android onPause, iOS 비활성, 웹 탭 숨김·창 포커스 잃음,
//! PC 창 최소화. macroquad 입력 구독 하나로 듣고, 앱은 프레임마다 `take`로 한 번 묻는다(구독은 매 프레임 비워야 쌓이지 않는다).
//! 싸움 중이면 멈춤 창을 연다(`tick_play`) — 돌아왔을 때 적이 먼저 움직이지 않는다. 기록은 바뀔 때마다 이미 적는다(`persist`).

pub(super) struct Away {
    sub: usize,
    went: bool,
}

impl Away {
    pub(super) fn new() -> Self {
        Self {
            sub: macroquad::input::utils::register_input_subscriber(),
            went: false,
        }
    }

    /// 지난 물음 뒤로 내려간 적이 있나.
    pub(super) fn take(&mut self) -> bool {
        let sub = self.sub;
        macroquad::input::utils::repeat_all_miniquad_input(self, sub);
        std::mem::take(&mut self.went)
    }
}

impl macroquad::miniquad::EventHandler for Away {
    fn update(&mut self) {}
    fn draw(&mut self) {}
    fn window_minimized_event(&mut self) {
        self.went = true;
    }
}
