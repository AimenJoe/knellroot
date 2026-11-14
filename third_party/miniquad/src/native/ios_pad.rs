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
//! - 패드가 여럿이면 합친다(1인 게임 — 버튼은 어느 쪽이든, 스틱·D패드는 더 기운 쪽). 패드가 빠지면 눌린 것을 모두 뗀다.
//! - Home 버튼은 시스템 몫(게임 센터·스크린숏)이라 읽지 않는다. 최소 iOS 14(ios/Info.plist)라 `buttonMenu`·`buttonOptions`(13)·
//!   스틱 버튼(12.1)이 있다 — 그래도 nil이면 안 눌린 것으로 본다.
//!
//! 0137 손질(첫 게임 nail 0288과 견주어 가져온 것):
//! - 연결·끊김 알림을 한 번 구독한다(Apple 권장 — iOS 15부터 프레임워크는 `controllers`를 처음 읽거나 알림을 구독할 때 패드를
//!   찾기 시작한다는 개발자 포럼 답, 2차). 처리기는 비어 있다 — 붙고 떨어지는 것은 폴링이 본다.
//!   알림 이름 상수를 실제로 참조하므로 GameController 프레임워크가 **반드시 링크된다**(빈 `#[link]` 블록만으로는 장담 못 함 —
//!   `build-ios.yml`이 `otool -L`로도 본다).
//! - `class!` 대신 `Class::get`: 클래스를 못 찾으면 panic 대신 "패드 없음".
//! - 읽기를 `autoreleasepool`로 감싼다(`controllers`가 돌려주는 배열 등 — 그리기마다 쌓이지 않게).

use {
    crate::{
        event::{EventHandler, KeyCode, KeyMods},
        native::apple::frameworks::*,
    },
    std::sync::{
        atomic::{AtomicU32, Ordering},
        Once,
    },
};

#[link(name = "GameController", kind = "framework")]
extern "C" {
    static GCControllerDidConnectNotification: ObjcId;
    static GCControllerDidDisconnectNotification: ObjcId;
}

#[allow(clippy::declare_interior_mutable_const)]
const AXIS_ZERO: AtomicU32 = AtomicU32::new(0);
static JOYSTICK: [AtomicU32; 6] = [AXIS_ZERO; 6];
/// 지난 그리기에 눌려 있던 버튼(비트 i = `KEYS[i]`). 메인 스레드에서만 쓴다.
static HELD: AtomicU32 = AtomicU32::new(0);
static OBSERVE: Once = Once::new();

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

/// 연결·끊김 알림을 한 번 구독한다(빈 처리기, 관찰자는 앱이 끝날 때까지 둔다).
fn observe() {
    OBSERVE.call_once(|| {
        extern "C" fn changed(_: &Object, _: Sel, _: ObjcId) {}
        let Some(mut decl) = ClassDecl::new("KnellPadObserver", class!(NSObject)) else {
            return;
        };
        // SAFETY: NSObject 하위 클래스에 `padChanged:`(인자 하나, 반환 없음)를 붙여 등록하고, 알림 센터에 그 셀렉터로 건다.
        // 알림 이름은 GameController 프레임워크가 내보내는 NSString 상수. 메인 스레드에서 한 번만.
        unsafe {
            decl.add_method(
                sel!(padChanged:),
                changed as extern "C" fn(&Object, Sel, ObjcId),
            );
            let cls = decl.register();
            let observer: ObjcId = msg_send![cls, new];
            let center: ObjcId = msg_send![class!(NSNotificationCenter), defaultCenter];
            for name in [
                GCControllerDidConnectNotification,
                GCControllerDidDisconnectNotification,
            ] {
                let () = msg_send![center, addObserver: observer
                                           selector: sel!(padChanged:)
                                           name: name
                                           object: nil];
            }
        }
    });
}

unsafe fn pressed(button: ObjcId) -> bool {
    if button.is_null() {
        return false;
    }
    let p: BOOL = msg_send![button, isPressed];
    p != NO
}

/// `GCControllerDirectionPad`의 (x, y), 위가 +y(GameController 그대로).
unsafe fn axes(pad: ObjcId) -> (f32, f32) {
    if pad.is_null() {
        return (0.0, 0.0);
    }
    let x: ObjcId = msg_send![pad, xAxis];
    let y: ObjcId = msg_send![pad, yAxis];
    if x.is_null() || y.is_null() {
        return (0.0, 0.0);
    }
    let xv: f32 = msg_send![x, value];
    let yv: f32 = msg_send![y, value];
    (xv, yv)
}

/// 둘 중 더 기운 축 쌍.
fn stronger(a: (f32, f32), b: (f32, f32)) -> (f32, f32) {
    if b.0 * b.0 + b.1 * b.1 > a.0 * a.0 + a.1 * a.1 {
        b
    } else {
        a
    }
}

/// 연결된 확장 게임패드를 모두 합친 (버튼 비트, 축 — 위가 +y인 [L, R, D패드] 쌍).
unsafe fn read() -> (u32, [(f32, f32); 3]) {
    let mut mask = 0u32;
    let mut sticks = [(0.0, 0.0); 3];
    let Some(cls) = Class::get("GCController") else {
        return (mask, sticks);
    };
    let list: ObjcId = msg_send![cls, controllers];
    if list.is_null() {
        return (mask, sticks);
    }
    let n: usize = msg_send![list, count];
    for i in 0..n {
        let c: ObjcId = msg_send![list, objectAtIndex: i];
        if c.is_null() {
            continue;
        }
        let g: ObjcId = msg_send![c, extendedGamepad];
        if g.is_null() {
            continue;
        }
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
        for (b, button) in buttons.iter().enumerate() {
            if pressed(*button) {
                mask |= 1 << b;
            }
        }
        let now = [
            axes(msg_send![g, leftThumbstick]),
            axes(msg_send![g, rightThumbstick]),
            axes(msg_send![g, dpad]),
        ];
        for (s, v) in sticks.iter_mut().zip(now) {
            *s = stronger(*s, v);
        }
    }
    (mask, sticks)
}

/// 그리기마다 한 번(메인 스레드): 버튼 변화를 키 이벤트로, 축을 `JOYSTICK`에.
pub fn poll(handler: &mut dyn EventHandler) {
    observe();
    // SAFETY: GameController 클래스 메서드·속성만 부르고(메인 스레드), nil은 위에서 거른다.
    let (mask, [(lx, ly), (rx, ry), (dx, dy)]) =
        objc::rc::autoreleasepool(|| unsafe { read() });
    for (cell, v) in JOYSTICK.iter().zip([lx, -ly, rx, -ry, dx, -dy]) {
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
