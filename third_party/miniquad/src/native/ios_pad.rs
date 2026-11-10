//! Knellroot 패치(0137): iOS 게임 컨트롤러 — Android 패치(0075·0143)와 같은 꼴로 넘긴다.
//!
//! 원본 miniquad iOS 백엔드는 패드를 전혀 읽지 않고, `gamepads`(gilrs)에도 iOS 백엔드가 없다. Apple GameController 프레임워크를
//! 그리기마다(메인 스레드, `draw_in_rect`가 `update` 직전에) 한 번 읽는다 — Apple 문서 "Handling input events"의 폴링 길
//! ("Use polling when input handling occurs at a fixed interval as part of your game's loop").
//! - 버튼: `GCExtendedGamepad`의 버튼을 Android 패치와 같은 짝으로 `KeyCode::F13`~`F24` 키 눌림·뗌 이벤트로
//!   (A F13 · B F14 · X F15 · Y F16 · LB F17 · RB F18 · LT F19 · RT F20 · LS F21 · RS F22 · Menu F23 · Options F24).
//!   A·B·X·Y는 Apple 정의상 위치 이름이다(A = 아래, B = 오른쪽 …) — 게임의 `Pad::South` 등과 같다.
//! - 축: `joystick_axes()` = [왼쪽 X, 왼쪽 Y, 오른쪽 X, 오른쪽 Y, D패드 X, D패드 Y], **아래가 +y**(Android 패치와 같은 뜻 —
//!   GameController는 위가 +라 뒤집는다).
//! 연결된 패드 가운데 `extendedGamepad`가 있는 첫 패드만 읽는다. 패드가 빠지면 눌린 것을 모두 뗀다.
//! 최소 iOS 14(ios/Info.plist)라 `buttonMenu`(13)·`buttonOptions`(13)·스틱 버튼(12.1)이 있다 — 그래도 nil이면 안 눌린 것으로 본다.

use {
    crate::{
        event::{EventHandler, KeyCode, KeyMods},
        native::apple::frameworks::*,
    },
    std::sync::atomic::{AtomicU32, Ordering},
};

#[link(name = "GameController", kind = "framework")]
extern "C" {}

#[allow(clippy::declare_interior_mutable_const)]
const AXIS_ZERO: AtomicU32 = AtomicU32::new(0);
static JOYSTICK: [AtomicU32; 6] = [AXIS_ZERO; 6];
/// 지난 그리기에 눌려 있던 버튼(비트 i = `KEYS[i]`). 메인 스레드에서만 쓴다.
static HELD: AtomicU32 = AtomicU32::new(0);

const KEYS: [KeyCode; 12] = [
    KeyCode::F13,
    KeyCode::F14,
    KeyCode::F15,
    KeyCode::F16,
    KeyCode::F17,
    KeyCode::F18,
    KeyCode::F19,
    KeyCode::F20,
    KeyCode::F21,
    KeyCode::F22,
    KeyCode::F23,
    KeyCode::F24,
];

/// 마지막으로 읽은 패드 축: [왼쪽 X, 왼쪽 Y, 오른쪽 X, 오른쪽 Y, D패드 X, D패드 Y]. 아래가 +y. 패드가 없으면 0.
pub fn joystick_axes() -> [f32; 6] {
    let mut out = [0.0; 6];
    for (o, cell) in out.iter_mut().zip(JOYSTICK.iter()) {
        *o = f32::from_bits(cell.load(Ordering::Relaxed));
    }
    out
}

unsafe fn pressed(button: ObjcId) -> bool {
    if button.is_null() {
        return false;
    }
    let p: BOOL = msg_send![button, isPressed];
    p == YES
}

/// `GCControllerDirectionPad`의 (x, y), 위가 +y(GameController 그대로).
unsafe fn axes(pad: ObjcId) -> (f32, f32) {
    if pad.is_null() {
        return (0.0, 0.0);
    }
    let x: ObjcId = msg_send![pad, xAxis];
    let y: ObjcId = msg_send![pad, yAxis];
    let xv: f32 = msg_send![x, value];
    let yv: f32 = msg_send![y, value];
    (xv, yv)
}

/// 첫 확장 게임패드(`GCExtendedGamepad`), 없으면 nil.
unsafe fn first_gamepad() -> ObjcId {
    let list: ObjcId = msg_send![class!(GCController), controllers];
    if list.is_null() {
        return nil;
    }
    let n: usize = msg_send![list, count];
    for i in 0..n {
        let c: ObjcId = msg_send![list, objectAtIndex: i];
        let g: ObjcId = msg_send![c, extendedGamepad];
        if !g.is_null() {
            return g;
        }
    }
    nil
}

/// 그리기마다 한 번(메인 스레드): 버튼 변화를 키 이벤트로, 축을 `JOYSTICK`에.
pub fn poll(handler: &mut dyn EventHandler) {
    let (mask, ax) = unsafe {
        let g = first_gamepad();
        if g.is_null() {
            (0u32, [0.0f32; 6])
        } else {
            let buttons: [ObjcId; 12] = [
                msg_send![g, buttonA],
                msg_send![g, buttonB],
                msg_send![g, buttonX],
                msg_send![g, buttonY],
                msg_send![g, leftShoulder],
                msg_send![g, rightShoulder],
                msg_send![g, leftTrigger],
                msg_send![g, rightTrigger],
                msg_send![g, leftThumbstickButton],
                msg_send![g, rightThumbstickButton],
                msg_send![g, buttonMenu],
                msg_send![g, buttonOptions],
            ];
            let mut mask = 0u32;
            for (i, b) in buttons.iter().enumerate() {
                if pressed(*b) {
                    mask |= 1 << i;
                }
            }
            let (lx, ly) = axes(msg_send![g, leftThumbstick]);
            let (rx, ry) = axes(msg_send![g, rightThumbstick]);
            let (dx, dy) = axes(msg_send![g, dpad]);
            (mask, [lx, -ly, rx, -ry, dx, -dy])
        }
    };
    for (cell, v) in JOYSTICK.iter().zip(ax) {
        cell.store(v.to_bits(), Ordering::Relaxed);
    }
    let was = HELD.swap(mask, Ordering::Relaxed);
    let changed = was ^ mask;
    for (i, key) in KEYS.iter().enumerate() {
        if changed & (1 << i) == 0 {
            continue;
        }
        if mask & (1 << i) != 0 {
            handler.key_down_event(*key, KeyMods::default(), false);
        } else {
            handler.key_up_event(*key, KeyMods::default());
        }
    }
}
