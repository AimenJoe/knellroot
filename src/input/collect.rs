//! 장치마다 이번 프레임에 모은 것(0278 R23 — 옛 `Input::poll` 445줄을 나눈 것): 키보드 · 패드(하나마다) · 터치 · 마우스가
//! 저마다 평범한 값을 내고, `merge`가 옛 `poll`의 차례 그대로 하나의 `Actions`로 접는다 — 눌림은 OR, 이동 · 조준은 뒤 장치가 덮고,
//! 패드의 큰 기술 조합은 **그때까지 모은** 눌림에서 그 버튼의 동작을 거둔다(옛 `drop_pads`). 래치에 얹는 것은 `fold_latch`.
//! 엔진(`is_key_pressed` · `gamepads` · `touches` · `mouse_position`)을 부르는 것은 모으는 함수들뿐이고, 합치기 둘은 순수하다(시험).

#[cfg(not(pad_keys))]
use gamepads::Button;

use super::*;
use crate::touch::Read;

/// 키보드 또는 패드 하나가 이번 프레임에 모은 것.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct Frame {
    /// 눌림 · 쥐고 있음 · 메뉴 · 개발 키 · 시점 거리(`or_buttons`가 접는 칸만 쓴다).
    pub acts: Actions,
    /// 있으면 앞 장치의 이동 · 조준을 덮는다(키보드는 늘 있다 — 0이어도, 패드는 데드존을 넘었을 때만).
    pub move_dir: Option<Vec2>,
    pub aim: Option<Vec2>,
    /// 이 패드가 이룬 큰 기술 조합의 버튼에 붙은 동작(`BINDS`의 자리) — 그때까지 모은 눌림에서 거둔다(0255).
    pub chord: [bool; NB],
    /// 이 장치를 만졌다(지금 쓰는 장치를 정한다).
    pub used: bool,
}

/// 마우스가 이번 프레임에 모은 것(터치 다음에 읽는다 — 터치 직후의 마우스 사건은 흉내일 수 있다).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct MouseFrame {
    pub used: bool,
    /// 메뉴의 휠(부호만 한 칸)과 그 자리.
    pub wheel: i32,
    pub wheel_at: Option<Vec2>,
    /// 싸움의 휠 = 시점 당기기 · 밀기(0181).
    pub zoom: i32,
    /// 잡힌 커서의 움직임(자유 숄더뷰 시점).
    pub look: Option<Vec2>,
    /// 메뉴의 끌기(0235): 가로 거리와 누른 자리.
    pub drag_x: Option<f32>,
    pub drag_at: Option<Vec2>,
    /// 메뉴에서 왼쪽을 뗀 자리(끌지 않았을 때) — 터치의 탭이 없을 때만 센다.
    pub tap: Option<Vec2>,
    /// 지금 커서 자리(조준 커서).
    pub pos: Vec2,
}

/// 키보드(Android · iOS에서는 패드의 십자 키가 화살표 키코드로 온다 — 키보드로 세지 않는다).
pub(super) fn keys_frame(menu: bool, bind: &Bindings) -> Frame {
    let mut a = Actions::default();
    // Android·iOS에 키보드를 붙이는 일은 드물고, 패드 버튼·D패드도 키로 들어온다 — 키보드로 세지 않는다(안내 글이 키 글자로 바뀌지 않게).
    let used_keys = !cfg!(pad_keys) && get_last_key_pressed().is_some();
    // 키보드 이동.
    let mut m = Vec2::ZERO;
    #[allow(unused_mut)]
    let mut aim = Vec2::ZERO;
    // 화살표는 걷지 않는다(0181 — 패드의 십자 키와 같은 일): ↑·↓ = 시점 당기기·밀기, ←·→ = 자유 숄더뷰에서 시점 돌리기.
    // Android·iOS에서는 화살표 키코드가 곧 패드의 십자 키다(0151).
    if is_key_down(KeyCode::A) {
        m.x -= 1.0;
    }
    if is_key_down(KeyCode::D) {
        m.x += 1.0;
    }
    if is_key_down(KeyCode::W) {
        m.y += 1.0;
    }
    if is_key_down(KeyCode::S) {
        m.y -= 1.0;
    }
    if !menu {
        a.zoom += is_key_pressed(KeyCode::Up) as i32 - is_key_pressed(KeyCode::Down) as i32;
    }
    #[cfg(not(pad_keys))]
    if free_look() && !menu {
        aim.x +=
            is_key_down(KeyCode::Right) as i32 as f32 - is_key_down(KeyCode::Left) as i32 as f32;
    }
    // 게임 동작은 배치표에서(0024).
    let kp = |x: Bind| bind.key(x).pressed();
    a.attack = kp(Bind::Attack);
    a.attack_held = bind.key(Bind::Attack).down();
    a.dodge = kp(Bind::Dodge);
    a.parry = kp(Bind::Parry);
    a.jump = kp(Bind::Jump);
    a.flask = kp(Bind::Flask);
    a.cast = kp(Bind::Cast);
    a.cast_held = bind.key(Bind::Cast).down();
    a.dodge_held = bind.key(Bind::Dodge).down();
    a.parry_held = bind.key(Bind::Parry).down();
    a.lock = kp(Bind::Lock);
    a.turn_left = kp(Bind::TurnLeft);
    a.turn_right = kp(Bind::TurnRight);
    a.skill1 = kp(Bind::Skill1);
    a.skill2 = kp(Bind::Skill2);
    a.swap = kp(Bind::Swap);
    // 큰 기술의 따로 붙인 키(0255): 이룬 키의 제 동작은 이번 틱에 거둔다(조합의 둘째 키가 스킬·공격을 같이 내지 않게).
    if let Some(hit) = chord_fired(
        bind.ult.keys,
        |k| Slot::of(k).pressed(),
        |k| Slot::of(k).down(),
    ) {
        a.ult = true;
        for k in hit.into_iter().flatten() {
            for x in BINDS {
                if bind.keys[x as usize] == k {
                    a.drop_press(x);
                }
            }
        }
    }
    a.tab_prev = a.skill1;
    a.tab_next = a.skill2;
    a.pressed_key = KEYS
        .iter()
        .position(|(k, _)| is_key_pressed(*k))
        .or_else(|| {
            MICE.iter()
                .position(|(m, _)| is_mouse_button_pressed(*m))
                .map(|i| KEYS.len() + i)
        })
        .map(|i| i as u8);
    a.lab_toggle = is_key_pressed(KeyCode::F1);
    // Tab은 배치표에 올릴 수 있는 키라, 게임 동작에 붙이지 않았을 때만 장비 창.
    a.equip = is_key_pressed(KeyCode::Tab) && !bind.keys.contains(&key_index(KeyCode::Tab));
    a.dev_overlay = is_key_pressed(KeyCode::F3);
    a.ui_check = is_key_pressed(KeyCode::F4);
    a.step_clock = is_key_pressed(KeyCode::F6);
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
    for (i, k) in [
        KeyCode::Key1,
        KeyCode::Key2,
        KeyCode::Key3,
        KeyCode::Key4,
        KeyCode::Key5,
        KeyCode::Key6,
        KeyCode::Key7,
        KeyCode::Key8,
        KeyCode::Key9,
    ]
    .iter()
    .enumerate()
    {
        if is_key_pressed(*k) {
            a.scenario = Some(i as u8 + 1);
        }
    }
    Frame {
        acts: a,
        move_dir: Some(m),
        aim: Some(aim),
        chord: [false; NB],
        used: used_keys,
    }
}

/// 큰 기술 조합을 이룬 패드 버튼 → 그 버튼에 붙은 동작(`BINDS`의 자리).
fn chord_binds(bind: &Bindings, hit: [Option<Pad>; 2]) -> [bool; NB] {
    let mut out = [false; NB];
    for p in hit.into_iter().flatten() {
        for x in BINDS {
            if bind.pad(x) == p {
                out[x as usize] = true;
            }
        }
    }
    out
}

/// 터치의 원거리 무기: 든 무기 그림의 공격 버튼 = 사격, 원거리 버튼 = 수류탄(0223 — 패드·키보드는 맞바꾸지 않는다, 0255).
pub(super) fn ranged_swap(mut t: Read, ranged: bool) -> Read {
    if ranged {
        std::mem::swap(&mut t.attack, &mut t.cast);
        std::mem::swap(&mut t.attack_held, &mut t.cast_held);
    }
    t
}

impl Input {
    /// 데스크톱 · 웹의 패드(`gamepads`) — 패드마다 한 장(차례가 곧 옛 고리의 차례: 뒤 패드의 조합은 앞 패드의 눌림도 거둔다).
    #[cfg(not(pad_keys))]
    pub(super) fn pad_frames(&mut self, menu: bool, bind: &Bindings) -> Vec<Frame> {
        let mut out = Vec::new();
        self.pads.poll();
        for pad in self.pads.all() {
            let mut a = Actions::default();
            let mut f = Frame::default();
            let st = stick(pad.left_stick());
            if st.length() > tuning::STICK_DEADZONE {
                f.move_dir = Some(st);
                f.used = true;
            }
            let r = stick(pad.right_stick());
            if r.length() > tuning::STICK_DEADZONE {
                f.aim = Some(r);
                f.used = true;
            }
            let pj = |x: Bind| pad.is_just_pressed(bind.pad(x).button());
            a.attack |= pj(Bind::Attack);
            a.attack_held |= pad.is_currently_pressed(bind.pad(Bind::Attack).button());
            a.dodge |= pj(Bind::Dodge);
            a.dodge_held |= pad.is_currently_pressed(bind.pad(Bind::Dodge).button());
            a.jump |= pj(Bind::Jump);
            a.lock |= pj(Bind::Lock);
            // 랩이 열려 있으면 어깨·트리거는 랩 단계(LB 10배·LT 담기·RT 바꾸기)다 — 패링·돌리기로 같이 읽지 않는다.
            a.parry |= pj(Bind::Parry);
            a.parry_held |= pad.is_currently_pressed(bind.pad(Bind::Parry).button());
            a.flask |= pj(Bind::Flask);
            a.cast |= pj(Bind::Cast);
            a.cast_held |= pad.is_currently_pressed(bind.pad(Bind::Cast).button());
            a.turn_left |= pj(Bind::TurnLeft);
            a.turn_right |= pj(Bind::TurnRight);
            a.skill1 |= pj(Bind::Skill1);
            a.skill2 |= pj(Bind::Skill2);
            a.swap |= pj(Bind::Swap);
            if let Some(hit) = chord_fired(
                bind.ult.pads,
                |p| pad.is_just_pressed(p.button()),
                |p| pad.is_currently_pressed(p.button()),
            ) {
                a.ult = true;
                f.chord = chord_binds(bind, hit);
            }
            a.tab_prev |= pad.is_just_pressed(Pad::L1.button());
            a.tab_next |= pad.is_just_pressed(Pad::R1.button());
            if a.pressed_pad.is_none() {
                a.pressed_pad = PADS
                    .iter()
                    .copied()
                    .find(|p| p.bindable() && pad.is_just_pressed(p.button()));
            }
            a.equip |= pad.is_just_pressed(Button::LeftCenterCluster);
            a.menu_up |= pad.is_just_pressed(Button::DPadUp);
            a.menu_down |= pad.is_just_pressed(Button::DPadDown);
            // 십자 ↑·↓ = 시점 당기기·밀기(0181 — 싸움 화면에서만 쓴다).
            if !menu {
                a.zoom += pad.is_just_pressed(Button::DPadUp) as i32
                    - pad.is_just_pressed(Button::DPadDown) as i32;
            }
            a.menu_left |= pad.is_just_pressed(Button::DPadLeft);
            a.menu_right |= pad.is_just_pressed(Button::DPadRight);
            a.confirm |= pad.is_just_pressed(Button::ActionDown);
            a.back |= pad.is_just_pressed(Button::ActionRight);
            // 0019: Start = 멈춤(예전엔 개발 시나리오 넘기기 — 키보드 1~7·터치 SC로 남김).
            a.pause |= pad.is_just_pressed(Button::RightCenterCluster);
            // 메뉴 스틱: 한 번 넘을 때만.
            menu_from_stick(&mut self.menu_stick, stick(pad.left_stick()), &mut a);
            f.used |= pad.all_just_pressed().next().is_some();
            f.acts = a;
            out.push(f);
        }
        out
    }

    /// Android(0022)·iOS(0137) = `cfg(pad_keys)`: `gamepads`가 빠지고, 패치한 miniquad가 패드 버튼을 F13~F24 키로, 스틱·HAT을
    /// `joystick_axes`로 넘긴다(third_party/miniquad/PATCHED.md, 첫 게임 nail 0075·0143 — iOS는 GameController를 매 프레임 읽어 같은 꼴로).
    /// D패드가 키코드로 오는 Android 기기는 화살표 키 — 키보드 줄이 받는다. iOS D패드는 HAT 축으로 온다.
    /// 버튼 배치는 데스크톱 패드와 같다. 패드는 늘 한 장.
    #[cfg(pad_keys)]
    pub(super) fn pad_frames(&mut self, menu: bool, bind: &Bindings) -> Vec<Frame> {
        let mut a = Actions::default();
        let mut f = Frame::default();
        let (mv, r, hat) = android_axes(macroquad::miniquad::native::joystick_axes());
        if mv != Vec2::ZERO {
            f.move_dir = Some(mv);
            f.used = true;
        }
        if r != Vec2::ZERO {
            f.aim = Some(r);
            f.used = true;
        }
        // 십자 키 좌·우: 키코드(Android) 또는 HAT 축이 새로 기운 프레임(iOS·일부 Android).
        let was = std::mem::replace(&mut self.hat, hat);
        f.used |= hat != Vec2::ZERO;
        let j = |p: Pad| {
            is_key_pressed(p.key())
                || (p == Pad::DLeft && hat.x < 0.0 && was.x >= 0.0)
                || (p == Pad::DRight && hat.x > 0.0 && was.x <= 0.0)
        };
        let pj = |x: Bind| j(bind.pad(x));
        a.attack |= pj(Bind::Attack);
        a.attack_held |= is_key_down(bind.pad(Bind::Attack).key());
        a.dodge |= pj(Bind::Dodge);
        a.dodge_held |= is_key_down(bind.pad(Bind::Dodge).key());
        a.jump |= pj(Bind::Jump);
        a.lock |= pj(Bind::Lock);
        a.parry |= pj(Bind::Parry);
        a.parry_held |= is_key_down(bind.pad(Bind::Parry).key());
        a.flask |= pj(Bind::Flask);
        a.cast |= pj(Bind::Cast);
        a.cast_held |= is_key_down(bind.pad(Bind::Cast).key());
        a.turn_left |= pj(Bind::TurnLeft);
        a.turn_right |= pj(Bind::TurnRight);
        a.skill1 |= pj(Bind::Skill1);
        a.skill2 |= pj(Bind::Skill2);
        a.swap |= pj(Bind::Swap);
        if let Some(hit) = chord_fired(bind.ult.pads, j, |p| is_key_down(p.key())) {
            a.ult = true;
            f.chord = chord_binds(bind, hit);
        }
        a.tab_prev |= j(Pad::L1);
        a.tab_next |= j(Pad::R1);
        if a.pressed_pad.is_none() {
            a.pressed_pad = PADS.iter().copied().find(|p| p.bindable() && j(*p));
        }
        a.equip |= j(Pad::Select);
        // 십자 ↑·↓ = 시점(0181): 키코드(Android — 키보드 줄이 이미 받았다)가 아니면 HAT 축이 새로 기운 프레임(iOS).
        if !menu {
            a.zoom += (hat.y > 0.0 && was.y <= 0.0) as i32 - (hat.y < 0.0 && was.y >= 0.0) as i32;
        }
        a.confirm |= j(Pad::South);
        a.back |= j(Pad::East) || is_key_pressed(KeyCode::Back);
        a.pause |= j(Pad::Start);
        menu_from_stick(
            &mut self.menu_stick,
            if hat != Vec2::ZERO { hat } else { mv },
            &mut a,
        );
        let arrows = [KeyCode::Up, KeyCode::Down, KeyCode::Left, KeyCode::Right];
        f.used |=
            PADS.iter().any(|p| is_key_down(p.key())) || arrows.iter().any(|k| is_key_down(*k));
        f.acts = a;
        vec![f]
    }

    /// 터치(`touch.rs`) — 원거리 무기면 공격 · 원거리 버튼을 맞바꾼 뒤.
    pub(super) fn touch_frame(&mut self, menu: bool) -> Read {
        let t = self.touch.read(menu);
        ranged_swap(t, self.touch.ctx.ranged)
    }

    /// 마우스(0058): 움직이거나 누르면 "만졌다". 터치 직후의 마우스 사건은 터치가 흉내 낸 것일 수 있어 세지 않는다.
    /// `touched` = 이번 프레임에 화면에 손가락이 있다, `tapped` = 터치가 이미 탭을 냈다(마우스의 탭은 그때 세지 않는다).
    pub(super) fn mouse_frame(&mut self, menu: bool, touched: bool, tapped: bool) -> MouseFrame {
        let mut o = MouseFrame::default();
        let fdt = get_frame_time();
        self.since_touch = if touched {
            0.0
        } else {
            (self.since_touch + fdt).min(f32::MAX)
        };
        let mp = Vec2::from(mouse_position());
        o.pos = mp;
        let mouse_real = !cfg!(pad_keys) && self.since_touch > MOUSE_AFTER_TOUCH;
        let clicked = MICE.iter().any(|(m, _)| is_mouse_button_pressed(*m));
        // 첫 프레임의 자리는 기준만 잡는다(켜자마자 "만졌다"가 되지 않게).
        let anchor = *self.mouse_anchor.get_or_insert(mp);
        let used_mouse = mouse_real && ((mp - anchor).length() > MOUSE_MOVE_PX || clicked);
        o.used = used_mouse;
        let prev_mouse = self.mouse_pos;
        if mouse_real {
            self.mouse_pos = mp;
        }
        if used_mouse {
            self.since_mouse = 0.0;
            self.mouse_anchor = Some(mp);
        } else {
            self.since_mouse = (self.since_mouse + fdt).min(f32::MAX);
        }
        // 메뉴 굴리기: 휠은 부호만 한 칸(기기마다 값 크기가 다르다, KB gamepad-and-android-input). 싸움에서는 시점 당기기·밀기(0181).
        if mouse_real {
            let wy = mouse_wheel().1;
            if wy.abs() > 0.0 {
                if menu {
                    o.wheel = if wy > 0.0 { -1 } else { 1 };
                    o.wheel_at = Some(mp);
                } else {
                    o.zoom += if wy > 0.0 { 1 } else { -1 };
                }
            }
        }
        // 자유 숄더뷰(0181): 마우스로 하는 사람이면 커서를 잡아(웹은 포인터 잠금) 움직임을 시점으로 — 패드·터치·메뉴에서는 놓는다.
        let grab = free_look() && !menu && !cfg!(pad_keys) && device() == Device::Keys;
        // 잡은 첫 프레임은 자리가 튈 수 있어 세지 않는다.
        if grab && self.grabbed && mouse_real {
            o.look = Some(mp - prev_mouse);
        }
        if grab != self.grabbed {
            self.grabbed = grab;
            set_cursor_grab(grab);
            show_mouse(!grab);
        } else if grab && clicked {
            // 웹: Esc는 브라우저가 포인터 잠금을 먼저 풀어 버린다(게임은 아직 잡은 줄 안다) — 누를 때마다 다시 청한다(누름 = 사용자 동작).
            set_cursor_grab(true);
        }
        // 메뉴의 마우스 끌기(0235): 왼쪽을 쥐고 `MOUSE_DRAG_PX`를 넘게 움직이면 터치 끌기와 같은 가로 끌기(`drag_x` — 인형 돌리기·
        // 탭 줄 굴리기). 끈 뒤 뗀 것은 탭이 아니다(터치와 같은 규칙).
        if menu && mouse_real {
            if is_mouse_button_pressed(MouseButton::Left) {
                (self.mouse_down_at, self.mouse_dragged) = (Some(mp), false);
            }
            if let Some(at) = self
                .mouse_down_at
                .filter(|_| is_mouse_button_down(MouseButton::Left))
            {
                self.mouse_dragged |= (mp - at).length() > MOUSE_DRAG_PX;
                if self.mouse_dragged {
                    o.drag_x = Some(prev_mouse.x - mp.x);
                    o.drag_at = Some(at);
                }
            }
        }
        // 메뉴는 마우스 왼쪽을 뗄 때 탭(터치와 같은 규칙, nail 0199).
        if menu && mouse_real && !tapped && is_mouse_button_released(MouseButton::Left) {
            if !std::mem::take(&mut self.mouse_dragged) {
                o.tap = Some(mp);
            }
            self.mouse_down_at = None;
        }
        o
    }
}

/// 키보드 · 패드가 낸 칸을 OR로 접는다(시점 거리는 더하고, 고른 키 · 시나리오 번호는 먼저 온 것).
fn or_buttons(a: &mut Actions, f: &Actions) {
    a.attack |= f.attack;
    a.attack_held |= f.attack_held;
    a.dodge |= f.dodge;
    a.dodge_held |= f.dodge_held;
    a.parry |= f.parry;
    a.parry_held |= f.parry_held;
    a.jump |= f.jump;
    a.flask |= f.flask;
    a.cast |= f.cast;
    a.cast_held |= f.cast_held;
    a.lock |= f.lock;
    a.turn_left |= f.turn_left;
    a.turn_right |= f.turn_right;
    a.skill1 |= f.skill1;
    a.skill2 |= f.skill2;
    a.swap |= f.swap;
    a.ult |= f.ult;
    a.tab_prev |= f.tab_prev;
    a.tab_next |= f.tab_next;
    a.pressed_key = a.pressed_key.or(f.pressed_key);
    a.pressed_pad = a.pressed_pad.or(f.pressed_pad);
    a.lab_toggle |= f.lab_toggle;
    a.equip |= f.equip;
    a.dev_overlay |= f.dev_overlay;
    a.ui_check |= f.ui_check;
    a.step_clock |= f.step_clock;
    a.menu_up |= f.menu_up;
    a.menu_down |= f.menu_down;
    a.menu_left |= f.menu_left;
    a.menu_right |= f.menu_right;
    a.confirm |= f.confirm;
    a.back |= f.back;
    a.pause |= f.pause;
    a.scenario = a.scenario.or(f.scenario);
    a.zoom += f.zoom;
}

/// 이번 프레임의 장치들 → 하나의 입력(커서 · 래치 밖)과 지금 쓰는 장치(만진 것이 있으면): 키보드 → 패드들(차례대로 — 조합이 그때까지의
/// 눌림을 거둔다) → 터치(이동은 키 · 패드가 0일 때만, 눌림은 OR) → 마우스(휠 · 잡힌 커서 · 메뉴 끌기 · 탭). `dp` = 터치 끌기를 dp로(0252).
pub(super) fn merge(
    keys: &Frame,
    pads: &[Frame],
    t: &Read,
    mouse: &MouseFrame,
    dp: f32,
) -> (Actions, Option<Device>) {
    let mut a = Actions::default();
    let mut m = Vec2::ZERO;
    let mut aim = Vec2::ZERO;
    for f in std::iter::once(keys).chain(pads) {
        or_buttons(&mut a, &f.acts);
        a.drop_chord(&f.chord);
        if let Some(v) = f.move_dir {
            m = v;
        }
        if let Some(v) = f.aim {
            aim = v;
        }
    }
    let used_pad = pads.iter().any(|f| f.used);
    // 터치: 스틱은 키·패드가 0일 때만, 눌림은 OR.
    a.pause |= t.pause;
    a.tap = t.tap;
    if m == Vec2::ZERO {
        m = t.move_dir;
    }
    a.attack |= t.attack;
    a.attack_held |= t.attack_held;
    a.dodge |= t.dodge;
    a.dodge_held |= t.dodge_held;
    a.parry |= t.parry;
    a.parry_held |= t.parry_held;
    a.jump |= t.jump;
    a.flask |= t.flask;
    a.interact |= t.interact;
    a.cast |= t.cast;
    a.cast_held |= t.cast_held;
    // 끌어 조준(0217): 터치의 조준은 오른쪽 스틱처럼 — 끈 쪽이 있으면 그것.
    a.aim_drag |= t.aim_drag;
    a.auto_aim |= t.auto_aim;
    a.aim_hold |= t.aim_hold;
    // 0048의 "마법 직후 돌리기 누름 버리기"(RB 바로 밑이 RT)는 걷었다(0151): RT는 이제 스킬이고 RB → RT로 잇는 것이 바라는 쓰임이다.
    a.lock |= t.lock;
    a.turn_right |= t.turn_right;
    a.turn_left |= t.turn_left;
    a.skill1 |= t.skill1;
    a.skill2 |= t.skill2;
    a.ult |= t.ult;
    // 마우스의 휠 · 잡힌 커서.
    a.wheel = mouse.wheel;
    a.zoom += mouse.zoom;
    if let Some(l) = mouse.look {
        a.look += l;
    }
    a.drag = t.drag;
    a.drag_x = t.drag_x;
    // 굴리기의 자리: 휠 > 터치 끌기 > 마우스 끌기.
    a.point = mouse.wheel_at.or(t.drag_at);
    // 터치로 끈 거리는 dp로(Android는 화면 단위가 화소 — 같은 손가락 거리에 시점이 2~3배 돌았다, `screen::dp`).
    a.look += t.look / dp;
    a.zoom += t.zoom;
    a.view |= t.view;
    a.swap |= t.swap;
    if let Some(d) = mouse.drag_x {
        a.drag_x += d;
        a.point = a.point.or(mouse.drag_at);
    }
    if a.tap.is_none() {
        a.tap = mouse.tap;
    }
    // 지금 쓰는 장치 = 마지막으로 만진 것.
    let used = if t.active {
        Some(Device::Touch)
    } else if used_pad {
        Some(Device::Pad)
    } else if keys.used || mouse.used {
        Some(Device::Keys)
    } else {
        None
    };
    if m.length() > 1.0 {
        m = m.normalize();
    }
    a.move_dir = m;
    if t.aim != Vec2::ZERO {
        aim = t.aim;
    }
    if aim.length() > 1.0 {
        aim = aim.normalize();
    }
    a.aim = aim;
    (a, used)
}

/// 이번 프레임의 입력을 래치에 얹는다: 눌림은 틱이 가져갈 때까지 OR(수는 더한다, 자리는 이번 것이 먼저), 이동 · 조준 · 쥐고 있음 ·
/// 커서는 이번 것. 끌어 조준은 `merge_drag`(틱이 아직 안 가져간 지난 프레임의 끈 쪽을 남긴다). 월드 쪽 칸(`aim_ahead` …)은 앱이 채운다.
pub(super) fn fold_latch(prev: &Actions, taken: bool, a: &Actions) -> Actions {
    // 끌어 조준(0217): 이번 프레임의 손가락 + 아직 틱이 안 가져간 지난 프레임의 것(뗀 프레임의 끈 쪽은 그 누름과 함께 남는다).
    let (aim, aim_drag, auto_aim) = merge_drag(prev, taken, a.aim, a);
    Actions {
        move_dir: a.move_dir,
        aim,
        aim_drag,
        auto_aim,
        aim_hold: a.aim_hold,
        aim_ahead: false,
        aim_sight: None,
        aim_assist: 0.0,
        cursor: a.cursor,
        aim_at: None,
        zoom: prev.zoom + a.zoom,
        look: prev.look + a.look,
        view: prev.view | a.view,
        swap: prev.swap | a.swap,
        attack: prev.attack | a.attack,
        attack_held: a.attack_held,
        cast_held: a.cast_held,
        dodge_held: a.dodge_held,
        parry_held: a.parry_held,
        dodge: prev.dodge | a.dodge,
        parry: prev.parry | a.parry,
        jump: prev.jump | a.jump,
        flask: prev.flask | a.flask,
        interact: prev.interact | a.interact,
        cast: prev.cast | a.cast,
        lock: prev.lock | a.lock,
        turn_right: prev.turn_right | a.turn_right,
        turn_left: prev.turn_left | a.turn_left,
        skill1: prev.skill1 | a.skill1,
        skill2: prev.skill2 | a.skill2,
        tab_prev: prev.tab_prev | a.tab_prev,
        tab_next: prev.tab_next | a.tab_next,
        ult: prev.ult | a.ult,
        equip: prev.equip | a.equip,
        lab_toggle: prev.lab_toggle | a.lab_toggle,
        scenario: a.scenario.or(prev.scenario),
        menu_up: prev.menu_up | a.menu_up,
        menu_down: prev.menu_down | a.menu_down,
        menu_left: prev.menu_left | a.menu_left,
        menu_right: prev.menu_right | a.menu_right,
        confirm: prev.confirm | a.confirm,
        back: prev.back | a.back,
        pause: prev.pause | a.pause,
        tap: a.tap.or(prev.tap),
        drag: prev.drag + a.drag,
        drag_x: prev.drag_x + a.drag_x,
        wheel: prev.wheel + a.wheel,
        point: a.point.or(prev.point),
        dev_overlay: prev.dev_overlay | a.dev_overlay,
        ui_check: prev.ui_check | a.ui_check,
        step_clock: prev.step_clock | a.step_clock,
        pressed_key: a.pressed_key.or(prev.pressed_key),
        pressed_pad: a.pressed_pad.or(prev.pressed_pad),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(acts: Actions) -> Frame {
        Frame {
            acts,
            move_dir: Some(Vec2::ZERO),
            aim: Some(Vec2::ZERO),
            ..Default::default()
        }
    }

    fn pad(acts: Actions) -> Frame {
        Frame {
            acts,
            used: true,
            ..Default::default()
        }
    }

    #[test]
    fn presses_from_two_devices_in_one_frame_both_count_and_the_pad_is_the_device() {
        let k = keys(Actions {
            attack: true,
            ..Default::default()
        });
        let p = pad(Actions {
            dodge: true,
            zoom: 1,
            ..Default::default()
        });
        let t = Read {
            jump: true,
            zoom: 1,
            ..Default::default()
        };
        let (a, used) = merge(&k, &[p], &t, &MouseFrame::default(), 1.0);
        assert!(a.attack && a.dodge && a.jump);
        assert_eq!(a.zoom, 2, "zoom steps add up across devices");
        assert_eq!(used, Some(Device::Pad));
        // 손가락이 있으면 터치가 이긴다, 아무것도 안 만지면 장치는 그대로.
        let touched = Read {
            active: true,
            ..Default::default()
        };
        assert_eq!(
            merge(&k, &[p], &touched, &MouseFrame::default(), 1.0).1,
            Some(Device::Touch)
        );
        assert_eq!(
            merge(
                &keys(Actions::default()),
                &[],
                &Read::default(),
                &MouseFrame::default(),
                1.0
            )
            .1,
            None
        );
    }

    #[test]
    fn a_pad_chord_takes_back_the_bound_presses_gathered_so_far_but_not_later_ones() {
        let mut chord = [false; NB];
        chord[Bind::Skill1 as usize] = true;
        chord[Bind::Skill2 as usize] = true;
        // 키보드가 먼저 스킬 1을 눌렀고, 패드 하나가 스킬 둘로 큰 기술을 이뤘다 — 둘 다 거둔다(옛 `drop_pads`가 그때까지의 눌림을 거뒀다).
        let k = keys(Actions {
            skill1: true,
            attack: true,
            ..Default::default()
        });
        let p = Frame {
            chord,
            ..pad(Actions {
                skill1: true,
                skill2: true,
                ult: true,
                ..Default::default()
            })
        };
        // 그 뒤 패드의 스킬 2는 남는다, 터치의 스킬 1도.
        let later = pad(Actions {
            skill2: true,
            ..Default::default()
        });
        let (a, _) = merge(&k, &[p], &Read::default(), &MouseFrame::default(), 1.0);
        assert!(a.ult && a.attack && !a.skill1 && !a.skill2);
        let (a, _) = merge(
            &k,
            &[p, later],
            &Read::default(),
            &MouseFrame::default(),
            1.0,
        );
        assert!(a.ult && !a.skill1 && a.skill2);
        let t = Read {
            skill1: true,
            ..Default::default()
        };
        assert!(merge(&k, &[p], &t, &MouseFrame::default(), 1.0).0.skill1);
    }

    #[test]
    fn the_touch_stick_moves_only_when_keys_and_pads_stand_still_and_the_last_stick_wins() {
        let t = Read {
            move_dir: vec2(0.0, 0.5),
            aim: vec2(1.0, 0.0),
            ..Default::default()
        };
        let still = keys(Actions::default());
        let (a, _) = merge(&still, &[], &t, &MouseFrame::default(), 1.0);
        assert_eq!(a.move_dir, vec2(0.0, 0.5));
        // 키보드가 걸으면 터치 스틱은 안 쓴다, 데드존을 넘은 패드가 키보드를 덮는다. 터치의 끈 조준은 늘 이긴다.
        let walking = Frame {
            move_dir: Some(vec2(1.0, 1.0)),
            ..still
        };
        let (a, _) = merge(&walking, &[], &t, &MouseFrame::default(), 1.0);
        assert!((a.move_dir - vec2(1.0, 1.0).normalize()).length() < 1e-6);
        let stick = Frame {
            move_dir: Some(vec2(-0.4, 0.0)),
            aim: Some(vec2(0.0, -1.0)),
            ..pad(Actions::default())
        };
        let (a, _) = merge(&walking, &[stick], &t, &MouseFrame::default(), 1.0);
        assert_eq!(a.move_dir, vec2(-0.4, 0.0));
        assert_eq!(a.aim, vec2(1.0, 0.0));
    }

    #[test]
    fn the_wheel_point_beats_the_touch_drag_and_the_mouse_tap_waits_for_no_touch_tap() {
        let t = Read {
            drag_x: 3.0,
            drag_at: Some(vec2(10.0, 10.0)),
            look: vec2(6.0, 0.0),
            ..Default::default()
        };
        let m = MouseFrame {
            wheel: 1,
            wheel_at: Some(vec2(1.0, 1.0)),
            drag_x: Some(2.0),
            drag_at: Some(vec2(5.0, 5.0)),
            tap: Some(vec2(7.0, 7.0)),
            ..Default::default()
        };
        let (a, _) = merge(&keys(Actions::default()), &[], &t, &m, 3.0);
        assert_eq!(a.point, Some(vec2(1.0, 1.0)));
        assert_eq!(a.drag_x, 5.0);
        assert_eq!(a.tap, Some(vec2(7.0, 7.0)));
        assert_eq!(a.look, vec2(2.0, 0.0), "touch look is divided by dp");
        let no_wheel = MouseFrame {
            wheel_at: None,
            ..m
        };
        let tapped = Read {
            tap: Some(vec2(9.0, 9.0)),
            ..t
        };
        let (a, _) = merge(&keys(Actions::default()), &[], &tapped, &no_wheel, 1.0);
        assert_eq!(a.point, Some(vec2(10.0, 10.0)));
        assert_eq!(a.tap, Some(vec2(9.0, 9.0)));
    }

    #[test]
    fn the_ranged_touch_layout_swaps_attack_and_cast_only_with_a_ranged_weapon() {
        let t = Read {
            attack: true,
            attack_held: true,
            ..Default::default()
        };
        let s = ranged_swap(t, true);
        assert!(s.cast && s.cast_held && !s.attack && !s.attack_held);
        assert_eq!(ranged_swap(t, false), t);
    }

    #[test]
    fn the_latch_keeps_presses_until_a_tick_takes_them_and_holds_follow_the_frame() {
        let first = Actions {
            attack: true,
            attack_held: true,
            zoom: 1,
            ..Default::default()
        };
        let l = fold_latch(&Actions::default(), true, &first);
        // 틱이 오기 전 다음 프레임: 눌림은 남고(OR), 쥐고 있음은 이번 프레임의 것, 거리 단계는 쌓인다.
        let second = Actions {
            zoom: 1,
            ..Default::default()
        };
        let l = fold_latch(&l, false, &second);
        assert!(l.attack && !l.attack_held);
        assert_eq!(l.zoom, 2);
        // 틱이 가져가면(`carry`) 눌림은 사라진다.
        let l = fold_latch(&carry(&l), true, &Actions::default());
        assert!(!l.attack && l.zoom == 0);
    }
}
