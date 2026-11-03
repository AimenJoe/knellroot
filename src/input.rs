//! 입력 레이어(nail 규칙 그대로): 이 파일 밖에서는 `KeyCode`도 패드 버튼도 보지 않는다.
//! 프레임마다 `poll()`로 읽고 눌림(edge)은 래치에 모아 두며, 로직 틱의 `take()`가 소비한다.
//!
//! 기본 배치(패드 W3C 표준 자리 / 키보드):
//!   이동 = 왼쪽 스틱·십자 / WASD·화살표
//!   공격 = X(왼쪽) / J·Space      회피 = A(아래) / K·LShift      락온 = RB·오른쪽 스틱 클릭 / L·Tab
//!   랩 열기 = Select / F1          랩 안: 십자(↑↓ 고르기, ←→ 바꾸기), LB = 10배, Y = 초기화, B = 내보내기,
//!   LT = 슬롯에 담기, RT = A/B 바꾸기, Start/1~4 = 시나리오 바꾸기(dev 기능)
use macroquad::prelude::*;

#[cfg(not(target_os = "android"))]
use gamepads::{Button, Gamepads};

use crate::tuning;

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Actions {
    /// 이동. x = 오른쪽, y = 위(앞). 길이 0..=1.
    pub move_dir: Vec2,
    pub attack: bool,
    pub dodge: bool,
    pub lock: bool,
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
    /// 개발 시나리오 번호(1부터). `Start`는 다음 시나리오(0).
    pub scenario: Option<u8>,
}

pub struct Input {
    latch: Actions,
    #[cfg(not(target_os = "android"))]
    pads: Gamepads,
    /// 이동 반복 억제용: 십자키는 틱마다 반복이 아니라 눌림만 받는다(랩 커서).
    _pad_seen: bool,
}

impl Input {
    pub fn new() -> Self {
        Input {
            latch: Actions::default(),
            #[cfg(not(target_os = "android"))]
            pads: Gamepads::new(),
            _pad_seen: false,
        }
    }

    /// 프레임마다 한 번. 눌림은 OR로 모으고, 이동(상태)은 마지막 값으로 덮는다.
    pub fn poll(&mut self) {
        let mut a = Actions::default();
        // 키보드 이동.
        let mut m = Vec2::ZERO;
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
        a.lab_toggle = is_key_pressed(KeyCode::F1);
        a.lab_up = is_key_pressed(KeyCode::Up);
        a.lab_down = is_key_pressed(KeyCode::Down);
        a.lab_left = is_key_pressed(KeyCode::Left);
        a.lab_right = is_key_pressed(KeyCode::Right);
        a.lab_fast = is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl);
        a.lab_reset = is_key_pressed(KeyCode::R);
        a.lab_export = is_key_pressed(KeyCode::E);
        a.lab_store = is_key_pressed(KeyCode::C);
        a.lab_swap = is_key_pressed(KeyCode::V);
        for (i, k) in [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4]
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
                let (lx, ly) = pad.left_stick();
                let st = vec2(lx, -ly);
                if st.length() > tuning::STICK_DEADZONE {
                    m = st;
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
                a.lock |= pad.is_just_pressed(Button::FrontRightUpper)
                    || pad.is_just_pressed(Button::RightStick);
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
                if pad.is_just_pressed(Button::RightCenterCluster) {
                    a.scenario = Some(0);
                }
            }
        }

        if m.length() > 1.0 {
            m = m.normalize();
        }
        a.move_dir = m;
        self.latch = Actions {
            move_dir: m,
            attack: self.latch.attack | a.attack,
            dodge: self.latch.dodge | a.dodge,
            lock: self.latch.lock | a.lock,
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
        };
    }

    /// 로직 틱이 소비. 눌림은 지워지고 상태(이동·누르고 있음)는 남는다.
    pub fn take(&mut self) -> Actions {
        let a = self.latch;
        self.latch = Actions {
            move_dir: a.move_dir,
            lab_fast: a.lab_fast,
            ..Actions::default()
        };
        a
    }
}
