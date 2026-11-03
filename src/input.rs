//! 입력 레이어(nail 규칙 그대로): 이 파일 밖에서는 `KeyCode`도 패드 버튼도 보지 않는다.
//! 프레임마다 `poll()`로 읽고 눌림(edge)은 래치에 모아 두며, 로직 틱의 `take()`가 소비한다.
//!
//! 기본 배치(패드 W3C 표준 자리 / 키보드):
//!   이동 = 왼쪽 스틱·십자 / WASD·화살표      조준 = 오른쪽 스틱(바라보는 쪽, 공격은 가까운 적 자동 조준, 0014)
//!   공격 = X(왼쪽) / J·Space      회피 = A(아래) / K·LShift      락온 = 오른쪽 스틱 클릭 / L·Tab
//!   땅 돌리기(쟁반, 0012) = LB·Q 왼쪽 / RB·E 오른쪽 — 누르는 쪽으로 카메라가 돈다(0017: 오른쪽 = 땅은 화면에서 시계 방향, 먼 쪽이 오른쪽으로).
//!   Fez처럼 어깨 버튼, 락온은 다크소울처럼 R3로 옮겼다.
//!   멈춤 = Start / Esc·P · 메뉴 = 십자·왼쪽 스틱(한 번씩)·A 확인·B 뒤로 / 화살표·WASD·Enter·Esc(0019)
//!   랩 열기 = Select / F1          랩 안: 십자(↑↓ 고르기, ←→ 바꾸기), LB = 10배(랩이 열려 있으면 왼쪽 돌리기 대신), Y = 초기화, B·X키 = 내보내기,
//!   LT = 슬롯에 담기, RT = A/B 바꾸기, 1~7 = 시나리오 바꾸기(dev 기능)
//!   터치(`touch.rs`, 0006) = 왼쪽 반 가상 스틱 + 오른쪽 아래 공격·회피·락온·↶↷, 오른쪽 위 랩·시나리오(dev).
//!
//! 지금 쓰는 장치(`Device`)는 마지막으로 만진 것. 터치 오버레이는 장치가 터치일 때만 그린다(nail 0036·0172).
use std::sync::atomic::{AtomicU8, Ordering};

use macroquad::prelude::*;

#[cfg(not(target_os = "android"))]
use gamepads::{Button, Gamepads};

use crate::touch::{Overlay, Touch};
use crate::tuning;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Device {
    Keys,
    Pad,
    Touch,
}

static DEVICE: AtomicU8 = AtomicU8::new(u8::MAX);

pub fn device() -> Device {
    match DEVICE.load(Ordering::Relaxed) {
        u8::MAX => initial_device(),
        1 => Device::Pad,
        2 => Device::Touch,
        _ => Device::Keys,
    }
}

fn set_device(d: Device) {
    DEVICE.store(d as u8, Ordering::Relaxed);
}

/// 아무것도 만지기 전: 손가락이 주 포인터인 기기(폰·태블릿 브라우저)면 터치 — 첫 화면부터 버튼이 보이게.
fn initial_device() -> Device {
    #[cfg(target_arch = "wasm32")]
    {
        extern "C" {
            fn knell_coarse_pointer() -> u32;
        }
        // SAFETY: 인자 없이 정수를 돌려주는 JS 함수(web/index.html의 knell_device 플러그인).
        if unsafe { knell_coarse_pointer() } != 0 {
            return Device::Touch;
        }
        Device::Keys
    }
    #[cfg(target_os = "android")]
    {
        Device::Touch
    }
    #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
    {
        Device::Keys
    }
}

const DEV: bool = cfg!(feature = "dev");

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Actions {
    /// 이동. x = 오른쪽, y = 위(앞). 길이 0..=1.
    pub move_dir: Vec2,
    /// 조준(오른쪽 스틱, 0014). 이동과 같은 축. 0이면 겨누지 않음.
    pub aim: Vec2,
    pub attack: bool,
    pub dodge: bool,
    pub lock: bool,
    /// 땅을 시계 방향(화면에서 보아)으로 90° 돌리기 / 반시계(0012). 오른쪽 버튼 = 카메라가 오른쪽으로 = 땅은 시계 방향(`turn_cw`),
    /// 왼쪽 버튼 = `turn_ccw`(0017 — 사장이 "카메라가 오른쪽으로"를 골랐다. 0015의 "가까운 쪽 기준"을 되돌림).
    pub turn_cw: bool,
    pub turn_ccw: bool,
    pub lab_toggle: bool,
    pub lab_up: bool,
    pub lab_down: bool,
    pub lab_left: bool,
    pub lab_right: bool,
    /// 10배 단계(누르고 있는 동안).
    pub lab_fast: bool,
    pub lab_reset: bool,
    pub lab_export: bool,
    pub lab_store: bool,
    pub lab_swap: bool,
    /// 개발 시나리오 번호(1부터). 0 = 다음 시나리오(터치 SC).
    pub scenario: Option<u8>,
    /// 메뉴(0019): 위·아래·왼·오른(눌림, 스틱은 한 번 넘을 때만), 확인·뒤로, 멈춤, 터치 탭 자리.
    pub menu_up: bool,
    pub menu_down: bool,
    pub menu_left: bool,
    pub menu_right: bool,
    pub confirm: bool,
    pub back: bool,
    pub pause: bool,
    pub tap: Option<Vec2>,
    /// 개발: 측정·안내 줄 켜고 끄기(F3).
    pub dev_overlay: bool,
}

/// 화면에 키 그림을 쓸 행동(글에 키 이름을 박지 않는다 — 지금 장치에 맞춰 여기서, nail 0036).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Act {
    Move,
    Attack,
    Dodge,
    Turn,
    Lock,
    Pause,
    Choose,
    Confirm,
}

/// 지금 장치의 키 그림(ASCII — 글꼴 상관없이). 터치는 화면 버튼이 곧 안내라 빈 글.
pub fn prompt(act: Act) -> &'static str {
    match (device(), act) {
        (Device::Touch, _) => "",
        (Device::Pad, Act::Move) => "LS",
        (Device::Pad, Act::Attack) => "X",
        (Device::Pad, Act::Dodge) => "A",
        (Device::Pad, Act::Turn) => "LB/RB",
        (Device::Pad, Act::Lock) => "R3",
        (Device::Pad, Act::Pause) => "START",
        (Device::Pad, Act::Choose) => "D-PAD",
        (Device::Pad, Act::Confirm) => "A",
        (Device::Keys, Act::Move) => "WASD",
        (Device::Keys, Act::Attack) => "J",
        (Device::Keys, Act::Dodge) => "K",
        (Device::Keys, Act::Turn) => "Q/E",
        (Device::Keys, Act::Lock) => "L",
        (Device::Keys, Act::Pause) => "ESC",
        (Device::Keys, Act::Choose) => "W/S",
        (Device::Keys, Act::Confirm) => "ENTER",
    }
}

/// 메뉴용 스틱: 한 번 넘을 때만 눌림(히스테리시스 — 쥐고 있으면 반복하지 않는다).
const MENU_STICK_ON: f32 = 0.6;
const MENU_STICK_OFF: f32 = 0.3;

/// 패드 스틱(`gamepads`) → `Actions` 축(x 오른쪽, y 위). 그 크레이트는 이미 "위 = +"로 준다(웹 플러그인이 축 1·3을 뒤집는다,
/// 문서: "negative values representing down") — 그래서 그대로. 첫 게임은 화면 아래가 +인 축을 써서 `-ly`였고, 그 줄을 가져와
/// 여기서는 위아래가 뒤집혔다(Android 핸드헬드에서 왼쪽 스틱이 엉뚱한 쪽, D패드는 멀쩡 — 사장 2026-11-03, 0014).
pub fn stick((x, y): (f32, f32)) -> Vec2 {
    vec2(x, y)
}

pub struct Input {
    latch: Actions,
    /// 메뉴 스틱이 기울어 있는 쪽(히스테리시스).
    menu_stick: Vec2,
    touch: Touch,
    #[cfg(not(target_os = "android"))]
    pads: Gamepads,
    /// 이동 반복 억제용: 십자키는 틱마다 반복이 아니라 눌림만 받는다(랩 커서).
    _pad_seen: bool,
}

impl Input {
    pub fn new() -> Self {
        // 손가락을 마우스로 흉내 내지 않는다(nail 0199: 켜 두면 짚는 순간 클릭이 된다).
        simulate_mouse_with_touch(false);
        Input {
            latch: Actions::default(),
            menu_stick: Vec2::ZERO,
            touch: Touch::default(),
            #[cfg(not(target_os = "android"))]
            pads: Gamepads::new(),
            _pad_seen: false,
        }
    }

    /// 터치 장치일 때 렌더가 그릴 오버레이.
    pub fn overlay(&self, lab_open: bool) -> Option<Overlay> {
        (device() == Device::Touch).then(|| self.touch.overlay(DEV, lab_open))
    }

    /// 프레임마다 한 번. 눌림은 OR로 모으고, 이동(상태)은 마지막 값으로 덮는다. `menu`면 터치는 탭만.
    pub fn poll(&mut self, lab_open: bool, menu: bool) {
        let mut a = Actions::default();
        let used_keys = get_last_key_pressed().is_some();
        let mut used_pad = false;
        // 키보드 이동.
        let mut m = Vec2::ZERO;
        #[allow(unused_mut)]
        let mut aim = Vec2::ZERO;
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            m.x -= 1.0;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            m.x += 1.0;
        }
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            m.y += 1.0;
        }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            m.y -= 1.0;
        }
        a.attack = is_key_pressed(KeyCode::J) || is_key_pressed(KeyCode::Space);
        a.dodge = is_key_pressed(KeyCode::K) || is_key_pressed(KeyCode::LeftShift);
        a.lock = is_key_pressed(KeyCode::L) || is_key_pressed(KeyCode::Tab);
        a.turn_ccw = is_key_pressed(KeyCode::Q);
        a.turn_cw = is_key_pressed(KeyCode::E);
        a.lab_toggle = is_key_pressed(KeyCode::F1);
        a.dev_overlay = is_key_pressed(KeyCode::F3);
        a.menu_up = is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W);
        a.menu_down = is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S);
        a.menu_left = is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A);
        a.menu_right = is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D);
        a.confirm = is_key_pressed(KeyCode::Enter)
            || is_key_pressed(KeyCode::KpEnter)
            || is_key_pressed(KeyCode::Space)
            || is_key_pressed(KeyCode::J);
        a.back = is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Backspace);
        a.pause = is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::P);
        a.lab_up = is_key_pressed(KeyCode::Up);
        a.lab_down = is_key_pressed(KeyCode::Down);
        a.lab_left = is_key_pressed(KeyCode::Left);
        a.lab_right = is_key_pressed(KeyCode::Right);
        a.lab_fast = is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl);
        a.lab_reset = is_key_pressed(KeyCode::R);
        a.lab_export = is_key_pressed(KeyCode::X);
        a.lab_store = is_key_pressed(KeyCode::C);
        a.lab_swap = is_key_pressed(KeyCode::V);
        for (i, k) in [
            KeyCode::Key1,
            KeyCode::Key2,
            KeyCode::Key3,
            KeyCode::Key4,
            KeyCode::Key5,
            KeyCode::Key6,
            KeyCode::Key7,
        ]
        .iter()
        .enumerate()
        {
            if is_key_pressed(*k) {
                a.scenario = Some(i as u8 + 1);
            }
        }

        #[cfg(not(target_os = "android"))]
        {
            self.pads.poll();
            for pad in self.pads.all() {
                let st = stick(pad.left_stick());
                if st.length() > tuning::STICK_DEADZONE {
                    m = st;
                    used_pad = true;
                }
                let r = stick(pad.right_stick());
                if r.length() > tuning::STICK_DEADZONE {
                    aim = r;
                    used_pad = true;
                }
                if pad.is_currently_pressed(Button::DPadLeft) {
                    m.x -= 1.0;
                }
                if pad.is_currently_pressed(Button::DPadRight) {
                    m.x += 1.0;
                }
                if pad.is_currently_pressed(Button::DPadUp) {
                    m.y += 1.0;
                }
                if pad.is_currently_pressed(Button::DPadDown) {
                    m.y -= 1.0;
                }
                a.attack |= pad.is_just_pressed(Button::ActionLeft);
                a.dodge |= pad.is_just_pressed(Button::ActionDown);
                a.lock |= pad.is_just_pressed(Button::RightStick);
                a.turn_cw |= pad.is_just_pressed(Button::FrontRightUpper);
                // 랩이 열려 있으면 LB는 10배 단계다(누르고 십자) — 돌리기로 같이 읽지 않는다.
                a.turn_ccw |= !lab_open && pad.is_just_pressed(Button::FrontLeftUpper);
                a.lab_toggle |= pad.is_just_pressed(Button::LeftCenterCluster);
                a.lab_up |= pad.is_just_pressed(Button::DPadUp);
                a.lab_down |= pad.is_just_pressed(Button::DPadDown);
                a.lab_left |= pad.is_just_pressed(Button::DPadLeft);
                a.lab_right |= pad.is_just_pressed(Button::DPadRight);
                a.lab_fast |= pad.is_currently_pressed(Button::FrontLeftUpper);
                a.lab_reset |= pad.is_just_pressed(Button::ActionUp);
                a.lab_export |= pad.is_just_pressed(Button::ActionRight);
                a.lab_store |= pad.is_just_pressed(Button::FrontLeftLower);
                a.lab_swap |= pad.is_just_pressed(Button::FrontRightLower);
                a.menu_up |= pad.is_just_pressed(Button::DPadUp);
                a.menu_down |= pad.is_just_pressed(Button::DPadDown);
                a.menu_left |= pad.is_just_pressed(Button::DPadLeft);
                a.menu_right |= pad.is_just_pressed(Button::DPadRight);
                a.confirm |= pad.is_just_pressed(Button::ActionDown);
                a.back |= pad.is_just_pressed(Button::ActionRight);
                // 0019: Start = 멈춤(예전엔 개발 시나리오 넘기기 — 키보드 1~7·터치 SC로 남김).
                a.pause |= pad.is_just_pressed(Button::RightCenterCluster);
                // 메뉴 스틱: 한 번 넘을 때만.
                let st = stick(pad.left_stick());
                let was = self.menu_stick;
                let mut now = was;
                for (v, slot) in [(st.x, 0), (st.y, 1)] {
                    let cur = if slot == 0 { &mut now.x } else { &mut now.y };
                    if v.abs() > MENU_STICK_ON {
                        *cur = v.signum();
                    } else if v.abs() < MENU_STICK_OFF {
                        *cur = 0.0;
                    }
                }
                if now.y > 0.0 && was.y <= 0.0 {
                    a.menu_up = true;
                }
                if now.y < 0.0 && was.y >= 0.0 {
                    a.menu_down = true;
                }
                if now.x < 0.0 && was.x >= 0.0 {
                    a.menu_left = true;
                }
                if now.x > 0.0 && was.x <= 0.0 {
                    a.menu_right = true;
                }
                self.menu_stick = now;
                used_pad |= pad.all_just_pressed().next().is_some();
            }
        }

        // 터치: 스틱은 키·패드가 0일 때만, 눌림은 OR.
        let t = self.touch.read(get_frame_time(), DEV, lab_open, menu);
        a.pause |= t.pause;
        a.tap = t.tap;
        if m == Vec2::ZERO {
            m = t.move_dir;
        }
        a.attack |= t.attack;
        a.dodge |= t.dodge;
        a.lock |= t.lock;
        a.turn_cw |= t.turn_cw;
        a.turn_ccw |= t.turn_ccw;
        a.lab_toggle |= t.lab_toggle;
        a.lab_up |= t.lab_up;
        a.lab_down |= t.lab_down;
        a.lab_left |= t.lab_left;
        a.lab_right |= t.lab_right;
        a.lab_reset |= t.lab_reset;
        a.lab_store |= t.lab_store;
        a.lab_swap |= t.lab_swap;
        if t.scenario_next {
            a.scenario = Some(0);
        }
        // 지금 쓰는 장치 = 마지막으로 만진 것.
        if t.active {
            set_device(Device::Touch);
        } else if used_pad {
            set_device(Device::Pad);
        } else if used_keys {
            set_device(Device::Keys);
        }

        if m.length() > 1.0 {
            m = m.normalize();
        }
        a.move_dir = m;
        if aim.length() > 1.0 {
            aim = aim.normalize();
        }
        self.latch = Actions {
            move_dir: m,
            aim,
            attack: self.latch.attack | a.attack,
            dodge: self.latch.dodge | a.dodge,
            lock: self.latch.lock | a.lock,
            turn_cw: self.latch.turn_cw | a.turn_cw,
            turn_ccw: self.latch.turn_ccw | a.turn_ccw,
            lab_toggle: self.latch.lab_toggle | a.lab_toggle,
            lab_up: self.latch.lab_up | a.lab_up,
            lab_down: self.latch.lab_down | a.lab_down,
            lab_left: self.latch.lab_left | a.lab_left,
            lab_right: self.latch.lab_right | a.lab_right,
            lab_fast: a.lab_fast,
            lab_reset: self.latch.lab_reset | a.lab_reset,
            lab_export: self.latch.lab_export | a.lab_export,
            lab_store: self.latch.lab_store | a.lab_store,
            lab_swap: self.latch.lab_swap | a.lab_swap,
            scenario: a.scenario.or(self.latch.scenario),
            menu_up: self.latch.menu_up | a.menu_up,
            menu_down: self.latch.menu_down | a.menu_down,
            menu_left: self.latch.menu_left | a.menu_left,
            menu_right: self.latch.menu_right | a.menu_right,
            confirm: self.latch.confirm | a.confirm,
            back: self.latch.back | a.back,
            pause: self.latch.pause | a.pause,
            tap: a.tap.or(self.latch.tap),
            dev_overlay: self.latch.dev_overlay | a.dev_overlay,
        };
    }

    /// 로직 틱이 소비. 눌림은 지워지고 상태(이동·누르고 있음)는 남는다.
    pub fn take(&mut self) -> Actions {
        let a = self.latch;
        self.latch = Actions {
            move_dir: a.move_dir,
            aim: a.aim,
            lab_fast: a.lab_fast,
            ..Actions::default()
        };
        a
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// gamepads 크레이트 축: 위로 밀면 y > 0(문서) → 앞으로 걷는다.
    #[test]
    fn pushing_a_stick_up_means_forward() {
        assert_eq!(stick((0.0, 1.0)), vec2(0.0, 1.0));
        assert_eq!(stick((-1.0, 0.0)), vec2(-1.0, 0.0));
    }
}
