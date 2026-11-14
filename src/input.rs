//! 입력 레이어(nail 규칙 그대로): 이 파일 밖에서는 `KeyCode`도 패드 버튼도 보지 않는다.
//! 프레임마다 `poll()`로 읽고 눌림(edge)은 래치에 모아 두며, 로직 틱의 `take()`가 소비한다.
//!
//! 기본 배치(패드 W3C 표준 자리 / 키보드+마우스 0058 / 키보드만 0024·0048) — 표는 `Bindings::default_layout`·`keyboard_only`:
//!   이동 = 왼쪽 스틱(십자 키는 싸움에서 걷지 않는다, 0151) / WASD(·화살표)      조준 = 오른쪽 스틱 / 커서(최근 만진 것만, 0058) — 안 쓰면 자동 조준(0014)
//!   공격 X / 왼쪽 클릭 / J      마법 RB / 오른쪽 클릭 / U      회피 B / Space      패링 LB / L-Shift / K
//!   락온 R3 / 휠 클릭 / L      땅 돌리기 십자 ←·→ / Z·C(0151)      스킬 LT·RT / Q·E(함께 = 큰 기술)      점프·상호작용 A / F      등잔 Y / R
//!   멈춤 = Start / Esc·P · 메뉴 = 십자·왼쪽 스틱(한 번씩)·A 확인·B 뒤로 / 화살표·WASD·Enter·Esc(0019)
//!   장비 창 = Select / Tab(0070) · 랩 열기 = F1(개발판)          랩 안: 십자(↑↓ 고르기, ←→ 바꾸기), LB = 10배(랩이 열려 있으면 왼쪽 돌리기 대신), Y = 초기화, B·X키 = 내보내기,
//!   LT = 슬롯에 담기, RT = A/B 바꾸기, 1~7 = 시나리오 바꾸기(dev 기능)
//!   터치(`touch.rs`, 0006) = 왼쪽 반 가상 스틱 + 오른쪽 아래 공격·회피·락온·↶↷, 오른쪽 위 랩·시나리오(dev).
//!
//! 지금 쓰는 장치(`Device`)는 마지막으로 만진 것. 터치 오버레이는 장치가 터치일 때만 그린다(nail 0036·0172).
use std::sync::atomic::{AtomicU8, Ordering};

use macroquad::prelude::*;

#[cfg(not(pad_keys))]
use gamepads::{Button, Gamepads};

use std::sync::Mutex;

use crate::touch::{Overlay, Touch};
use crate::tuning;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Device {
    Keys,
    Pad,
    Touch,
}

static DEVICE: AtomicU8 = AtomicU8::new(u8::MAX);

/// 이번 프레임에 전체 화면 키(F11, PC만 — 0135, nail 0240)를 눌렀나. 창 일이라 `Actions`(게임 동작)에 넣지 않고,
/// 키 배치(`KEYS`)에도 없다(어떤 배치에서도 같은 키).
pub fn fullscreen_pressed() -> bool {
    crate::DESKTOP && is_key_pressed(KeyCode::F11)
}

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
    // iOS 앱(0136)도 손가락이 주 포인터 — 첫 화면부터 터치 버튼이 보이게(패드는 iOS에서 아직 안 된다).
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        Device::Touch
    }
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    {
        Device::Keys
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Actions {
    /// 이동. x = 오른쪽, y = 위(앞). 길이 0..=1.
    pub move_dir: Vec2,
    /// 조준(오른쪽 스틱, 0014). 이동과 같은 축. 0이면 겨누지 않음.
    pub aim: Vec2,
    /// 커서(0058): 마우스 조준이 켜져 있고 최근(`MouseAim::rest_secs`) 마우스를 만졌을 때만 화면 자리(논리 px). 상태 — 틱이 지우지 않는다.
    pub cursor: Option<Vec2>,
    /// 커서를 비춘 땅 위의 점(월드 x·z) — 앱이 카메라로 `cursor`에서 채운다(0058). 있으면 월드는 오른쪽 스틱처럼 그쪽을 겨눈다.
    pub aim_at: Option<Vec2>,
    /// 시점 거리 단계(0181): + = 당기기(패드 십자 ↑ · 키보드 ↑ · 휠 위 · 두 손가락 벌리기), − = 밀기. 틱마다 소비.
    pub zoom: i32,
    /// 자유 숄더뷰의 시점 돌리기(0181, 논리 px — 잡힌 마우스의 움직임·화면 오른쪽 끌기): x + = 오른쪽을 본다, y + = 아래를 본다.
    /// 오른쪽 스틱(`aim`)·키보드 ←·→는 앱이 빠르기로 더한다.
    pub look: Vec2,
    pub attack: bool,
    /// 공격 버튼을 쥐고 있나(상태, 강공격 모으기 0030). 눌림 `attack`과 달리 틱이 지우지 않는다.
    pub attack_held: bool,
    pub dodge: bool,
    /// 회피 버튼을 쥐고 있나(상태, 0076): 회피가 끝났을 때 아직 쥐고 있으면 달린다(KB nail 0209).
    pub dodge_held: bool,
    /// 패링(0025): 누른 뒤 짧은 창 안에 앞쪽에서 오는 근접 공격을 튕긴다.
    pub parry: bool,
    /// 점프(0027). 공중에서 공격 = 내려찍기.
    pub jump: bool,
    /// 등잔 마시기(0031).
    pub flask: bool,
    /// 상호작용(0032): 쉼터에서 쉬기 — 패드·키보드는 점프 버튼이 같이 한다(0048, 월드가 가른다). 터치만 따로 버튼.
    pub interact: bool,
    /// 마법(0048): 조준 스틱 쪽(안 쥐면 락온·자동 조준·바라보는 쪽)으로 탄 하나.
    pub cast: bool,
    /// RB를 쥐고 있나(상태, 0056 원거리 강공격 모으기).
    pub cast_held: bool,
    pub lock: bool,
    /// 땅 돌리기 오른쪽·왼쪽 버튼(0012·0028): 캐릭터 기준 90° — 몸이 제 오른쪽(왼쪽)으로 돌아서고 화면도 같은 쪽으로 90°
    /// (`world::plan_turn`).
    pub turn_right: bool,
    pub turn_left: bool,
    /// 스킬 버튼 둘(0151): LT·RT / Q·E. 싸움에서는 끼운 스킬, 메뉴에서는 빠른 바꾸기(1 = 왼쪽, 2 = 오른쪽 — 예전 돌리기 버튼의 몫).
    /// 둘을 함께 누르면 큰 기술 — 월드가 가른다(시동 안의 바꿔치기).
    pub skill1: bool,
    pub skill2: bool,
    /// 큰 창의 탭 넘기기(0164 → 0175): 패드 = 어깨 LB·RB(고정 — 메뉴의 십자·A·B처럼, KB 조작 노트 "어깨 = 탭 넘기기"),
    /// 키보드 = 스킬 버튼 둘(배치 — Q·E). 0164는 패드도 스킬 버튼(LT·RT)이었다 — 사장 "LB, RB가 맞아"(2026-11-14).
    pub tab_prev: bool,
    pub tab_next: bool,
    /// 큰 기술 버튼(터치만 — 게이지가 차면 뜬다). 패드·키보드는 스킬 둘을 함께.
    pub ult: bool,
    /// 장비 창 바로 열기(0070 6 A, 2026-11-06): 패드 Select · 키보드 Tab(배치표에 올리지 않았을 때).
    pub equip: bool,
    pub lab_toggle: bool,
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
    /// 메뉴 굴리기: 손가락 끌기(논리 px, 위로 끌면 +)·마우스 휠(칸, 아래로 +). 틱마다 소비(`ui::ListView::scroll`).
    pub drag: f32,
    pub wheel: i32,
    /// 굴리기가 일어난 자리(논리 px): 휠이면 마우스 자리, 끌기면 손가락이 처음 닿은 자리 — 한 화면에 굴릴 것이 둘일 때 가른다(0068).
    pub point: Option<Vec2>,
    /// 개발: 측정·안내 줄 켜고 끄기(F3).
    pub dev_overlay: bool,
    /// 개발: 글자 넘침 검사 켜고 끄기(F4, `ui::check`).
    pub ui_check: bool,
    /// 개발: 한 프레임 = 한 틱 시계 켜고 끄기(F6, 0141) — 짧은 이펙트·동작을 틱마다 본다(느린 기기·헤드리스에서는 느린 화면이 된다).
    pub step_clock: bool,
    /// 키 다시 붙이기(0024): 이번에 처음 눌린, 배치표에 올릴 수 있는 키(`KEYS`의 자리)·패드 버튼(Start·Select 빼고).
    pub pressed_key: Option<u8>,
    pub pressed_pad: Option<Pad>,
}

/// 화면에 키 그림을 쓸 행동(글에 키 이름을 박지 않는다 — 지금 장치·지금 배치에서, nail 0036·0024).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Act {
    Interact,
    Choose,
    Confirm,
    /// 장비 칸 빨리 바꾸기 = 공격 버튼 하나(0164 — 다음 후보로 돈다. 스킬 버튼 둘은 큰 창의 탭을 넘긴다).
    /// 그 버튼이 메뉴에서 이미 다른 일을 하면 빈 글(`swap_free`).
    Swap,
    /// 큰 창의 탭 넘기기(0164 → 0175): 패드 LB·RB, 키보드 스킬 버튼 둘(Q·E).
    Tabs,
    /// 격자 화면(기술, 0161)의 네 쪽 걷기 — 메뉴의 위·아래·왼·오른.
    Walk,
    /// 뒤로·닫기(고정 — 배치표에 없다).
    Back,
    /// 멈춤(고정 — Esc · Start). 깨어나는 첫 장면(0171)에서는 건너뛰기.
    Pause,
}

/// 지금 장치의 키 그림(ASCII — 글꼴 상관없이). 터치는 화면 버튼이 곧 안내라 빈 글.
pub fn prompt(act: Act) -> String {
    let b = bindings();
    let name = |x: Bind| match device() {
        Device::Pad => b.pad(x).name().to_string(),
        _ => key_name(b.keys[x as usize]).to_string(),
    };
    match (device(), act) {
        (Device::Touch, _) => String::new(),
        // 상호작용은 점프 버튼이 같이 한다(0048).
        (_, Act::Interact) => name(Bind::Jump),
        (Device::Pad, Act::Choose) => "D-PAD".into(),
        (Device::Pad, Act::Confirm) => "A".into(),
        (_, Act::Swap) if !swap_free() => String::new(),
        (_, Act::Swap) => name(Bind::Attack),
        (Device::Pad, Act::Tabs) => format!("{}/{}", Pad::L1.name(), Pad::R1.name()),
        (_, Act::Tabs) => format!("{}/{}", name(Bind::Skill1), name(Bind::Skill2)),
        (Device::Pad, Act::Walk) => "D-PAD".into(),
        (Device::Pad, Act::Back) => "B".into(),
        (Device::Pad, Act::Pause) => "START".into(),
        (_, Act::Choose) => "W/S".into(),
        (_, Act::Confirm) => "ENTER".into(),
        (_, Act::Walk) => "WASD".into(),
        (_, Act::Back) | (_, Act::Pause) => "ESC".into(),
    }
}

/// 싸움의 동작 하나의 지금 버튼 이름(튜토리얼의 힌트, 0167 — 지금 장치·지금 배치). 터치는 화면 버튼이 곧 안내라 빈 글.
pub fn bind_prompt(x: Bind) -> String {
    let b = bindings();
    match device() {
        Device::Touch => String::new(),
        Device::Pad => b.pad(x).name().to_string(),
        _ => key_name(b.keys[x as usize]).to_string(),
    }
}

/// 장비 칸 빨리 바꾸기(공격 버튼, 0164)를 지금 장치에서 쓸 수 있나: 공격 버튼이 메뉴에서 이미 다른 일을 하면 못 쓴다 —
/// 키보드의 J·Space는 확인이고(키보드만 배치의 공격 = J), 마우스 버튼은 누르기(기본 배치의 공격 = 왼쪽 클릭)다. 그때는 확인 →
/// 후보 목록으로 바꾼다(안내 줄에도 뜨지 않는다). 패드는 기본 배치(공격 = X)에서 된다. 터치는 메뉴에 공격 버튼이 없다.
pub fn swap_free() -> bool {
    swap_free_for(device(), &bindings())
}

fn swap_free_for(device: Device, b: &Bindings) -> bool {
    match device {
        Device::Touch => false,
        Device::Pad => !matches!(
            b.pad(Bind::Attack),
            Pad::South | Pad::East | Pad::Start | Pad::Select | Pad::DLeft | Pad::DRight
        ),
        Device::Keys => KEYS
            .get(b.keys[Bind::Attack as usize] as usize)
            .is_some_and(|(k, _)| !matches!(k, KeyCode::J | KeyCode::Space)),
    }
}

/// 다시 붙일 수 있는 게임 동작(0024). 이동·확인·뒤로·멈춤은 고정 — 배치를 망쳐도 메뉴로 돌아올 길이 남게
/// (KB concepts/controller-layout-and-remapping.md). 값 = 배치표의 자리(저장 칸 순서) — 바꾸지 않는다.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bind {
    Attack = 0,
    Dodge = 1,
    Parry = 2,
    Lock = 3,
    TurnLeft = 4,
    TurnRight = 5,
    /// 0027 — 저장 칸은 뒤에 붙는다(`Bindings::encode`).
    Jump = 6,
    /// 등잔 마시기(0031, RB·R).
    Flask = 7,
    /// 마법(0048, RB·U) — 0032의 상호작용 칸을 물려받았다(상호작용은 점프 버튼이 같이 한다). 저장 칸 뜻이 바뀌어 prefs MAGIC을 올렸다.
    Cast = 8,
    /// 스킬 둘(0151, LT·RT / Q·E) — 예전 땅 돌리기의 자리. 돌리기는 십자 ←·→ / Z·C로 갔다. 칸 뜻이 바뀌어 prefs MAGIC을 올렸다.
    Skill1 = 9,
    Skill2 = 10,
}

/// 다시 붙일 수 있는 동작 수.
pub const NB: usize = 11;

pub const BINDS: [Bind; NB] = [
    Bind::Attack,
    Bind::Dodge,
    Bind::Parry,
    Bind::Lock,
    Bind::TurnLeft,
    Bind::TurnRight,
    Bind::Jump,
    Bind::Flask,
    Bind::Cast,
    Bind::Skill1,
    Bind::Skill2,
];

/// 배치표에 올릴 수 있는 키(저장 번호 = 이 표의 자리 — 뒤에만 붙인다). 이동(WASD·화살표)·Enter·Esc·Backspace·P(멈춤)·
/// F1·F3·F4(개발)·숫자(개발 시나리오)는 고정이라 없다.
pub const KEYS: [(KeyCode, &str); 30] = [
    (KeyCode::J, "J"),
    (KeyCode::K, "K"),
    (KeyCode::L, "L"),
    (KeyCode::I, "I"),
    (KeyCode::U, "U"),
    (KeyCode::O, "O"),
    (KeyCode::H, "H"),
    (KeyCode::N, "N"),
    (KeyCode::M, "M"),
    (KeyCode::Q, "Q"),
    (KeyCode::E, "E"),
    (KeyCode::R, "R"),
    (KeyCode::F, "F"),
    (KeyCode::G, "G"),
    (KeyCode::T, "T"),
    (KeyCode::Y, "Y"),
    (KeyCode::Z, "Z"),
    (KeyCode::X, "X"),
    (KeyCode::C, "C"),
    (KeyCode::V, "V"),
    (KeyCode::B, "B"),
    (KeyCode::Space, "SPACE"),
    (KeyCode::LeftShift, "SHIFT"),
    (KeyCode::RightShift, "R-SHIFT"),
    (KeyCode::Tab, "TAB"),
    (KeyCode::LeftControl, "CTRL"),
    (KeyCode::LeftAlt, "ALT"),
    (KeyCode::Semicolon, ";"),
    (KeyCode::Comma, ","),
    (KeyCode::Period, "."),
];

/// 배치표에 올릴 수 있는 마우스 버튼(0058). 저장 번호는 `KEYS` 뒤에 이어진다(`KEYS.len()` + 이 표의 자리 — 뒤에만 붙인다).
/// 옆 버튼(M4·M5)은 miniquad `MouseButton`에 없다.
pub const MICE: [(MouseButton, &str); 3] = [
    (MouseButton::Left, "LMB"),
    (MouseButton::Right, "RMB"),
    (MouseButton::Middle, "MMB"),
];

/// 키 칸에 올릴 수 있는 것의 수(키 + 마우스 버튼).
pub const NSLOTS: usize = KEYS.len() + MICE.len();

/// 키 칸의 한 자리: 키 또는 마우스 버튼.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Slot {
    Key(KeyCode),
    Mouse(MouseButton),
}

impl Slot {
    pub fn of(i: u8) -> Slot {
        let i = (i as usize).min(NSLOTS - 1);
        match KEYS.get(i) {
            Some(k) => Slot::Key(k.0),
            None => Slot::Mouse(MICE[i - KEYS.len()].0),
        }
    }

    fn down(self) -> bool {
        match self {
            Slot::Key(k) => is_key_down(k),
            Slot::Mouse(m) => is_mouse_button_down(m),
        }
    }

    fn pressed(self) -> bool {
        match self {
            Slot::Key(k) => is_key_pressed(k),
            Slot::Mouse(m) => is_mouse_button_pressed(m),
        }
    }
}

fn key_index(k: KeyCode) -> u8 {
    KEYS.iter().position(|(c, _)| *c == k).unwrap_or(0) as u8
}

fn mouse_index(m: MouseButton) -> u8 {
    (KEYS.len() + MICE.iter().position(|(c, _)| *c == m).unwrap_or(0)) as u8
}

pub fn key_name(i: u8) -> &'static str {
    let i = i as usize;
    KEYS.get(i)
        .map(|k| k.1)
        .or_else(|| MICE.get(i.wrapping_sub(KEYS.len())).map(|m| m.1))
        .unwrap_or("?")
}

/// 동작 → (키 하나, 패드 버튼 하나). 겹치면 맞바꾼다(빈 동작이 생기지 않게).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bindings {
    pub keys: [u8; NB],
    pub pads: [Pad; NB],
}

impl Bindings {
    /// 기본 = 키보드+마우스(0058, 새로 설치한 판): 왼쪽 클릭 공격·오른쪽 클릭 마법(커서 쪽)·휠 클릭 락온·L-Shift 패링,
    /// 왼손은 그대로(Space 회피·F 점프·R 등잔·Q/E 스킬·Z/C 땅 돌리기 — 0151). 패드 칸은 `keyboard_only`와 같다.
    pub fn default_layout() -> Bindings {
        Bindings {
            keys: [
                mouse_index(MouseButton::Left),
                key_index(KeyCode::Space),
                key_index(KeyCode::LeftShift),
                mouse_index(MouseButton::Middle),
                key_index(KeyCode::Z),
                key_index(KeyCode::C),
                key_index(KeyCode::F),
                key_index(KeyCode::R),
                mouse_index(MouseButton::Right),
                key_index(KeyCode::Q),
                key_index(KeyCode::E),
            ],
            ..Bindings::keyboard_only()
        }
    }

    /// 키보드만(0024): 소울라이크식 — 회피 B·Space, 패링 LB, 스킬 LT·RT(0151 — 땅 돌리기는 십자 ←·→), 락온 R3, 점프 A·F(0027, 엘든 링). 공격만 X·J(사장: "공격만 우리식").
    /// 오른손은 J·K·L·U(패드의 오른쪽 넷처럼). 조작 설정의 "키보드만 배치로".
    pub fn keyboard_only() -> Bindings {
        Bindings {
            keys: [
                key_index(KeyCode::J),
                key_index(KeyCode::Space),
                key_index(KeyCode::K),
                key_index(KeyCode::L),
                key_index(KeyCode::Z),
                key_index(KeyCode::C),
                key_index(KeyCode::F),
                key_index(KeyCode::R),
                key_index(KeyCode::U),
                key_index(KeyCode::Q),
                key_index(KeyCode::E),
            ],
            pads: [
                Pad::West,
                Pad::East,
                Pad::L1,
                Pad::R3,
                Pad::DLeft,
                Pad::DRight,
                Pad::South,
                Pad::North,
                Pad::R1,
                Pad::L2,
                Pad::R2,
            ],
        }
    }

    /// 키 칸(키 또는 마우스 버튼, 0058).
    pub fn key(&self, b: Bind) -> Slot {
        Slot::of(self.keys[b as usize])
    }

    /// 키 칸 쓰는 것이 마우스 버튼인가(시험).
    #[cfg(test)]
    pub fn uses_mouse(&self) -> bool {
        self.keys.iter().any(|k| *k as usize >= KEYS.len())
    }

    pub fn pad(&self, b: Bind) -> Pad {
        self.pads[b as usize]
    }

    /// `b`에 키 `k`를 붙인다. 다른 동작이 쓰던 키면 그 동작은 `b`가 쓰던 키를 받는다.
    pub fn set_key(&mut self, b: Bind, k: u8) {
        let old = self.keys[b as usize];
        if let Some(other) = self.keys.iter().position(|x| *x == k) {
            self.keys[other] = old;
        }
        self.keys[b as usize] = k;
    }

    pub fn set_pad(&mut self, b: Bind, p: Pad) {
        let old = self.pads[b as usize];
        if let Some(other) = self.pads.iter().position(|x| *x == p) {
            self.pads[other] = old;
        }
        self.pads[b as usize] = p;
    }

    /// 저장 칸: 처음 여섯 동작의 키 여섯 + 패드 여섯(0024의 칸 그대로), 그 뒤에 붙인 동작마다 (키, 패드)(0027 점프).
    /// 0 = 없음이라 1을 더한다.
    pub fn encode(&self) -> [u32; 2 * NB] {
        let mut v = [0u32; 2 * NB];
        let pad_cell = |p: Pad| PADS.iter().position(|x| *x == p).unwrap_or(0) as u32 + 1;
        for i in 0..6 {
            v[i] = self.keys[i] as u32 + 1;
            v[6 + i] = pad_cell(self.pads[i]);
        }
        for i in 6..NB {
            v[12 + 2 * (i - 6)] = self.keys[i] as u32 + 1;
            v[13 + 2 * (i - 6)] = pad_cell(self.pads[i]);
        }
        v
    }

    /// 칸이 비었거나(0) 범위 밖이거나 겹치면 기본값. 뒤에 붙인 동작(점프 0027·등잔 0031)의 칸만 비었으면(옛 저장) 그 동작만
    /// 기본값 — 그 기본 키·버튼을 사용자가 이미 다른 동작에 붙였으면 비어 있는 첫 키·버튼을 준다(배치가 통째로 날아가지 않게).
    pub fn decode(v: &[u32]) -> Bindings {
        let d = Bindings::default_layout();
        if v.len() < 12 {
            return d;
        }
        let read = |k: u32, p: u32| -> Option<(u8, Pad)> {
            (k != 0 && k as usize <= NSLOTS && p != 0 && p as usize <= PADS.len())
                .then(|| ((k - 1) as u8, PADS[(p - 1) as usize]))
        };
        let mut b = d;
        for i in 0..6 {
            let Some((k, p)) = read(v[i], v[6 + i]) else {
                return d;
            };
            b.keys[i] = k;
            b.pads[i] = p;
        }
        let mut fresh = Vec::new();
        for i in 6..NB {
            let (ck, cp) = (12 + 2 * (i - 6), 13 + 2 * (i - 6));
            match v.get(ck).zip(v.get(cp)).and_then(|(k, p)| read(*k, *p)) {
                Some((k, p)) => {
                    b.keys[i] = k;
                    b.pads[i] = p;
                }
                None => fresh.push(i),
            }
        }
        for &i in &fresh {
            let others = |j: usize| j != i && !(fresh.contains(&j) && j > i);
            if (0..NB).any(|j| others(j) && b.keys[j] == b.keys[i]) {
                if let Some(k) = (0..NSLOTS as u8).find(|k| !b.keys.contains(k)) {
                    b.keys[i] = k;
                }
            }
            if (0..NB).any(|j| others(j) && b.pads[j] == b.pads[i]) {
                if let Some(p) = PADS
                    .iter()
                    .copied()
                    .find(|p| p.bindable() && !b.pads.contains(p))
                {
                    b.pads[i] = p;
                }
            }
        }
        let dup = |xs: &[u8]| (0..xs.len()).any(|i| xs[i + 1..].contains(&xs[i]));
        let pads: Vec<u8> = b.pads.iter().map(|p| *p as u8).collect();
        if dup(&b.keys) || dup(&pads) {
            return d;
        }
        b
    }
}

/// 마우스 조준 설정(0058): 켬/끔, 커서가 쉰 뒤 자동 조준으로 돌아가기까지(`REST_SECS`의 자리), 조준 표시 크기(0 작게·1 보통·2 크게).
/// 쿼터뷰에서 마우스는 패드의 오른쪽 스틱과 같은 일을 한다 — 커서는 OS 커서 그대로라 감도 칸은 없다(빠르기는 OS·DPI).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MouseAim {
    pub on: bool,
    pub rest: u8,
    pub size: u8,
}

/// 커서 쉼 시간 고르기(초). 기본은 자리 1(1초).
pub const REST_SECS: [f32; 4] = [0.5, 1.0, 2.0, 4.0];
/// 조준 표시 크기 단계 수.
pub const RETICLE_SIZES: u8 = 3;

impl Default for MouseAim {
    fn default() -> Self {
        MouseAim {
            on: true,
            rest: 1,
            size: 1,
        }
    }
}

impl MouseAim {
    pub fn rest_secs(&self) -> f32 {
        REST_SECS[(self.rest as usize).min(REST_SECS.len() - 1)]
    }
}

static MOUSE_AIM: Mutex<Option<MouseAim>> = Mutex::new(None);

pub fn mouse_aim() -> MouseAim {
    MOUSE_AIM.lock().ok().and_then(|g| *g).unwrap_or_default()
}

pub fn set_mouse_aim(m: MouseAim) {
    if let Ok(mut g) = MOUSE_AIM.lock() {
        *g = Some(m);
    }
}

/// 마우스가 있을 법한 기기인가(설정의 마우스 줄을 보일지): Android·손가락이 주 포인터인 브라우저는 아니다.
pub fn has_mouse() -> bool {
    !cfg!(pad_keys) && initial_device() != Device::Touch
}

/// 터치 뒤 이만큼(초)은 마우스 이벤트를 터치가 흉내 낸 것으로 본다(모바일 브라우저의 호환 마우스 이벤트) — 장치·탭으로 세지 않는다.
const MOUSE_AFTER_TOUCH: f32 = 1.0;
/// 이만큼(논리 px) 움직여야 "마우스를 만졌다"로 센다(떨림·흔들림은 빼고).
const MOUSE_MOVE_PX: f32 = 2.0;

/// 커서를 조준에 쓸까(0058, 순수 — 시험): 마우스 조준이 켜져 있고, 지금 장치가 키보드(+마우스)이고, 마우스를 만진 지 쉼 시간 안.
/// 키보드만으로 하는 사람의 쉬는 마우스는 조준을 빼앗지 않는다 — 패드에서 오른쪽 스틱을 놓은 것과 같다.
pub fn cursor_live(on: bool, device: Device, since_mouse: f32, rest: f32) -> bool {
    on && device == Device::Keys && since_mouse < rest
}

static BINDINGS: Mutex<Option<Bindings>> = Mutex::new(None);

/// 자유 숄더뷰인가(0181 — 앱이 틱마다 적는다): 마우스면 커서를 잡아 움직임을 시점으로, 키보드 ←·→도 시점(`aim.x`)으로 읽는다.
static FREE_LOOK: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn set_free_look(on: bool) {
    FREE_LOOK.store(on, Ordering::Relaxed);
}

pub fn free_look() -> bool {
    FREE_LOOK.load(Ordering::Relaxed)
}

/// 지금 배치(설정에서 바꾼다).
pub fn bindings() -> Bindings {
    BINDINGS
        .lock()
        .ok()
        .and_then(|g| *g)
        .unwrap_or_else(Bindings::default_layout)
}

pub fn set_bindings(b: Bindings) {
    if let Ok(mut g) = BINDINGS.lock() {
        *g = Some(b);
    }
}

/// 메뉴용 스틱: 한 번 넘을 때만 눌림(히스테리시스 — 쥐고 있으면 반복하지 않는다).
const MENU_STICK_ON: f32 = 0.6;
const MENU_STICK_OFF: f32 = 0.3;

/// 패드 스틱(`gamepads`) → `Actions` 축(x 오른쪽, y 위). 그 크레이트는 이미 "위 = +"로 준다(웹 플러그인이 축 1·3을 뒤집는다,
/// 문서: "negative values representing down") — 그래서 그대로. 첫 게임은 화면 아래가 +인 축을 써서 `-ly`였고, 그 줄을 가져와
/// 여기서는 위아래가 뒤집혔다(Android 핸드헬드에서 왼쪽 스틱이 엉뚱한 쪽, D패드는 멀쩡 — 사장 2026-11-03, 0014).
#[cfg_attr(pad_keys, allow(dead_code))]
pub fn stick((x, y): (f32, f32)) -> Vec2 {
    vec2(x, y)
}

/// 패드 버튼(위치 이름 — A·B 글자는 기기마다 다르다). 데스크톱·웹은 `gamepads` 크레이트, Android·iOS는 패치한 miniquad의 키.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pad {
    South,
    East,
    West,
    North,
    L1,
    R1,
    L2,
    R2,
    L3,
    R3,
    Start,
    Select,
    /// 십자 키 왼쪽·오른쪽(0151 — 땅 돌리기의 기본 자리). 표 끝에 붙였다(저장 번호 = `PADS`의 자리).
    DLeft,
    DRight,
}

pub const PADS: [Pad; 14] = [
    Pad::South,
    Pad::East,
    Pad::West,
    Pad::North,
    Pad::L1,
    Pad::R1,
    Pad::L2,
    Pad::R2,
    Pad::L3,
    Pad::R3,
    Pad::Start,
    Pad::Select,
    Pad::DLeft,
    Pad::DRight,
];

impl Pad {
    /// 화면 글(Xbox 글자 — 위치 기준이라 닌텐도 패드는 A/B가 반대로 보인다, KB 함정).
    pub fn name(self) -> &'static str {
        match self {
            Pad::South => "A",
            Pad::East => "B",
            Pad::West => "X",
            Pad::North => "Y",
            Pad::L1 => "LB",
            Pad::R1 => "RB",
            Pad::L2 => "LT",
            Pad::R2 => "RT",
            Pad::L3 => "LS",
            Pad::R3 => "RS",
            Pad::Start => "START",
            Pad::Select => "SELECT",
            // 십자 키는 글자 대신 세모(`i18n::UI_GLYPHS`에 있다).
            Pad::DLeft => "◀",
            Pad::DRight => "▶",
        }
    }

    /// 배치표에 올릴 수 있나(Start·Select는 멈춤·개발 랩으로 고정).
    pub fn bindable(self) -> bool {
        !matches!(self, Pad::Start | Pad::Select)
    }

    #[cfg(not(pad_keys))]
    fn button(self) -> Button {
        match self {
            Pad::South => Button::ActionDown,
            Pad::East => Button::ActionRight,
            Pad::West => Button::ActionLeft,
            Pad::North => Button::ActionUp,
            Pad::L1 => Button::FrontLeftUpper,
            Pad::R1 => Button::FrontRightUpper,
            Pad::L2 => Button::FrontLeftLower,
            Pad::R2 => Button::FrontRightLower,
            Pad::L3 => Button::LeftStick,
            Pad::R3 => Button::RightStick,
            Pad::Start => Button::RightCenterCluster,
            Pad::Select => Button::LeftCenterCluster,
            Pad::DLeft => Button::DPadLeft,
            Pad::DRight => Button::DPadRight,
        }
    }
}

/// Android·iOS: 패치한 miniquad가 패드 버튼(Android KEYCODE_BUTTON_*, iOS GCExtendedGamepad)을 쓰지 않는 F13~F24로 넘긴다(PATCHED.md).
/// 시험이 패치 표와 맞는지 보려고 시험 빌드에서도 컴파일한다.
#[cfg(any(pad_keys, test))]
impl Pad {
    fn key(self) -> KeyCode {
        match self {
            Pad::South => KeyCode::F13,
            Pad::East => KeyCode::F14,
            Pad::West => KeyCode::F15,
            Pad::North => KeyCode::F16,
            Pad::L1 => KeyCode::F17,
            Pad::R1 => KeyCode::F18,
            Pad::L2 => KeyCode::F19,
            Pad::R2 => KeyCode::F20,
            Pad::L3 => KeyCode::F21,
            Pad::R3 => KeyCode::F22,
            Pad::Start => KeyCode::F23,
            Pad::Select => KeyCode::F24,
            // 십자 키는 Android에서 화살표 키코드(또는 HAT 축 — `Input::hat`)로 온다.
            Pad::DLeft => KeyCode::Left,
            Pad::DRight => KeyCode::Right,
        }
    }
}

/// Android·iOS 패드 축(패치한 miniquad의 `joystick_axes`: [왼쪽 X, 왼쪽 Y, 오른쪽 X(Z), 오른쪽 Y(RZ), HAT X, HAT Y], **아래가 +y**)
/// → (이동, 조준, 십자 키), 우리 축(위가 +y). 십자 키(HAT)는 걷지 않는다(0151) — 메뉴 고르기와 땅 돌리기(좌·우)만.
#[cfg_attr(not(any(pad_keys, test)), allow(dead_code))]
fn android_axes(ax: [f32; 6]) -> (Vec2, Vec2, Vec2) {
    let dead = |v: Vec2| {
        if v.length() > tuning::STICK_DEADZONE {
            v
        } else {
            Vec2::ZERO
        }
    };
    (
        dead(vec2(ax[0], -ax[1])),
        dead(vec2(ax[2], -ax[3])),
        vec2(ax[4].round(), -ax[5].round()),
    )
}

/// 메뉴 스틱: 한 번 넘을 때만 눌림(히스테리시스 — 쥐고 있으면 반복하지 않는다). `held`는 지금 기울어 있는 쪽.
fn menu_from_stick(held: &mut Vec2, st: Vec2, a: &mut Actions) {
    let was = *held;
    let mut now = was;
    for (v, cur) in [(st.x, &mut now.x), (st.y, &mut now.y)] {
        if v.abs() > MENU_STICK_ON {
            *cur = v.signum();
        } else if v.abs() < MENU_STICK_OFF {
            *cur = 0.0;
        }
    }
    a.menu_up |= now.y > 0.0 && was.y <= 0.0;
    a.menu_down |= now.y < 0.0 && was.y >= 0.0;
    a.menu_left |= now.x < 0.0 && was.x >= 0.0;
    a.menu_right |= now.x > 0.0 && was.x <= 0.0;
    *held = now;
}

pub struct Input {
    latch: Actions,
    /// 메뉴 스틱이 기울어 있는 쪽(히스테리시스).
    menu_stick: Vec2,
    /// 십자 키(HAT 축)가 지난 프레임에 기울어 있던 쪽(Android·iOS — 좌·우의 눌림을 가장자리로 읽는다, 0151).
    #[cfg_attr(not(pad_keys), allow(dead_code))]
    hat: Vec2,
    /// 마우스(0058): 지금 자리, 마지막으로 "만졌다"고 센 자리(천천히 끌어도 쌓여서 넘게), 만진 뒤 지난 초, 마지막 터치 뒤 지난 초.
    mouse_pos: Vec2,
    mouse_anchor: Option<Vec2>,
    since_mouse: f32,
    since_touch: f32,
    touch: Touch,
    #[cfg(not(pad_keys))]
    pads: Gamepads,
    /// 이동 반복 억제용: 십자키는 틱마다 반복이 아니라 눌림만 받는다(랩 커서).
    _pad_seen: bool,
    /// 자유 숄더뷰에서 커서를 잡았나(0181 — 웹은 포인터 잠금).
    grabbed: bool,
}

impl Input {
    pub fn new() -> Self {
        // 손가락을 마우스로 흉내 내지 않는다(nail 0199: 켜 두면 짚는 순간 클릭이 된다).
        simulate_mouse_with_touch(false);
        Input {
            latch: Actions::default(),
            menu_stick: Vec2::ZERO,
            hat: Vec2::ZERO,
            mouse_pos: Vec2::ZERO,
            mouse_anchor: None,
            since_mouse: f32::MAX,
            since_touch: f32::MAX,
            touch: Touch::default(),
            #[cfg(not(pad_keys))]
            pads: Gamepads::new(),
            _pad_seen: false,
            grabbed: false,
        }
    }

    /// 터치에 상호작용 버튼을 띄울까(0032 — 쉼터 곁일 때만, 앱이 틱마다 알려 준다).
    /// 터치의 상황 버튼(0053): 앱이 매 프레임 넘긴다.
    pub fn set_touch_ctx(&mut self, ctx: crate::touch::Ctx) {
        self.touch.ctx = ctx;
    }

    /// 조준에 쓰는 커서 자리(0058) — 조준 표시를 그릴 때. `poll`과 같은 규칙.
    pub fn cursor(&self) -> Option<Vec2> {
        // 자유 숄더뷰(0181)에서 잡힌 커서는 시점이지 조준이 아니다 — 조준 표시를 그리지 않는다.
        if self.grabbed {
            return None;
        }
        let m = mouse_aim();
        cursor_live(m.on, device(), self.since_mouse, m.rest_secs()).then_some(self.mouse_pos)
    }

    /// 터치 장치일 때 렌더가 그릴 오버레이.
    pub fn overlay(&self) -> Option<Overlay> {
        (device() == Device::Touch).then(|| self.touch.overlay())
    }

    /// 프레임마다 한 번. 눌림은 OR로 모으고, 이동(상태)은 마지막 값으로 덮는다. `menu`면 터치는 탭만.
    pub fn poll(&mut self, menu: bool) {
        let mut a = Actions::default();
        // Android·iOS에 키보드를 붙이는 일은 드물고, 패드 버튼·D패드도 키로 들어온다 — 키보드로 세지 않는다(안내 글이 키 글자로 바뀌지 않게).
        let used_keys = !cfg!(pad_keys) && get_last_key_pressed().is_some();
        let mut used_pad = false;
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
            aim.x += is_key_down(KeyCode::Right) as i32 as f32
                - is_key_down(KeyCode::Left) as i32 as f32;
        }
        // 게임 동작은 배치표에서(0024).
        let bind = bindings();
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
        a.lock = kp(Bind::Lock);
        a.turn_left = kp(Bind::TurnLeft);
        a.turn_right = kp(Bind::TurnRight);
        a.skill1 = kp(Bind::Skill1);
        a.skill2 = kp(Bind::Skill2);
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

        #[cfg(not(pad_keys))]
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
                let pj = |x: Bind| pad.is_just_pressed(bind.pad(x).button());
                a.attack |= pj(Bind::Attack);
                a.attack_held |= pad.is_currently_pressed(bind.pad(Bind::Attack).button());
                a.dodge |= pj(Bind::Dodge);
                a.dodge_held |= pad.is_currently_pressed(bind.pad(Bind::Dodge).button());
                a.jump |= pj(Bind::Jump);
                a.lock |= pj(Bind::Lock);
                // 랩이 열려 있으면 어깨·트리거는 랩 단계(LB 10배·LT 담기·RT 바꾸기)다 — 패링·돌리기로 같이 읽지 않는다.
                a.parry |= pj(Bind::Parry);
                a.flask |= pj(Bind::Flask);
                a.cast |= pj(Bind::Cast);
                a.cast_held |= pad.is_currently_pressed(bind.pad(Bind::Cast).button());
                a.turn_left |= pj(Bind::TurnLeft);
                a.turn_right |= pj(Bind::TurnRight);
                a.skill1 |= pj(Bind::Skill1);
                a.skill2 |= pj(Bind::Skill2);
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
                used_pad |= pad.all_just_pressed().next().is_some();
            }
        }

        // Android(0022)·iOS(0137) = `cfg(pad_keys)`: `gamepads`가 빠지고, 패치한 miniquad가 패드 버튼을 F13~F24 키로, 스틱·HAT을
        // `joystick_axes`로 넘긴다(third_party/miniquad/PATCHED.md, 첫 게임 nail 0075·0143 — iOS는 GameController를 매 프레임 읽어 같은 꼴로).
        // D패드가 키코드로 오는 Android 기기는 화살표 키 — 위의 키보드 줄이 받는다. iOS D패드는 HAT 축으로 온다.
        // 버튼 배치는 데스크톱 패드와 같다.
        #[cfg(pad_keys)]
        {
            let (mv, r, hat) = android_axes(macroquad::miniquad::native::joystick_axes());
            if mv != Vec2::ZERO {
                m = mv;
                used_pad = true;
            }
            if r != Vec2::ZERO {
                aim = r;
                used_pad = true;
            }
            // 십자 키 좌·우: 키코드(Android) 또는 HAT 축이 새로 기운 프레임(iOS·일부 Android).
            let was = std::mem::replace(&mut self.hat, hat);
            used_pad |= hat != Vec2::ZERO;
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
            a.flask |= pj(Bind::Flask);
            a.cast |= pj(Bind::Cast);
            a.cast_held |= is_key_down(bind.pad(Bind::Cast).key());
            a.turn_left |= pj(Bind::TurnLeft);
            a.turn_right |= pj(Bind::TurnRight);
            a.skill1 |= pj(Bind::Skill1);
            a.skill2 |= pj(Bind::Skill2);
            a.tab_prev |= j(Pad::L1);
            a.tab_next |= j(Pad::R1);
            if a.pressed_pad.is_none() {
                a.pressed_pad = PADS.iter().copied().find(|p| p.bindable() && j(*p));
            }
            a.equip |= j(Pad::Select);
            // 십자 ↑·↓ = 시점(0181): 키코드(Android — 위의 키보드 줄이 이미 받았다)가 아니면 HAT 축이 새로 기운 프레임(iOS).
            if !menu {
                a.zoom +=
                    (hat.y > 0.0 && was.y <= 0.0) as i32 - (hat.y < 0.0 && was.y >= 0.0) as i32;
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
            used_pad |=
                PADS.iter().any(|p| is_key_down(p.key())) || arrows.iter().any(|k| is_key_down(*k));
        }

        // 터치: 스틱은 키·패드가 0일 때만, 눌림은 OR.
        let t = self.touch.read(menu);
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
        a.jump |= t.jump;
        a.flask |= t.flask;
        a.interact |= t.interact;
        a.cast |= t.cast;
        a.cast_held |= t.cast_held;
        // 0048의 "마법 직후 돌리기 누름 버리기"(RB 바로 밑이 RT)는 걷었다(0151): RT는 이제 스킬이고 RB → RT로 잇는 것이 바라는 쓰임이다.
        a.lock |= t.lock;
        a.turn_right |= t.turn_right;
        a.turn_left |= t.turn_left;
        a.skill1 |= t.skill1;
        a.skill2 |= t.skill2;
        a.ult |= t.ult;
        // 마우스(0058): 움직이거나 누르면 "만졌다". 터치 직후의 마우스 이벤트는 터치가 흉내 낸 것일 수 있어 세지 않는다.
        let fdt = get_frame_time();
        self.since_touch = if t.active {
            0.0
        } else {
            (self.since_touch + fdt).min(f32::MAX)
        };
        let mp = Vec2::from(mouse_position());
        let mouse_real = !cfg!(pad_keys) && self.since_touch > MOUSE_AFTER_TOUCH;
        let clicked = MICE.iter().any(|(m, _)| is_mouse_button_pressed(*m));
        // 첫 프레임의 자리는 기준만 잡는다(켜자마자 "만졌다"가 되지 않게).
        let anchor = *self.mouse_anchor.get_or_insert(mp);
        let used_mouse = mouse_real && ((mp - anchor).length() > MOUSE_MOVE_PX || clicked);
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
                    a.wheel = if wy > 0.0 { -1 } else { 1 };
                    a.point = Some(mp);
                } else {
                    a.zoom += if wy > 0.0 { 1 } else { -1 };
                }
            }
        }
        // 자유 숄더뷰(0181): 마우스로 하는 사람이면 커서를 잡아(웹은 포인터 잠금) 움직임을 시점으로 — 패드·터치·메뉴에서는 놓는다.
        let grab = free_look() && !menu && !cfg!(pad_keys) && device() == Device::Keys;
        // 잡은 첫 프레임은 자리가 튈 수 있어 세지 않는다.
        if grab && self.grabbed && mouse_real {
            a.look += mp - prev_mouse;
        }
        if grab != self.grabbed {
            self.grabbed = grab;
            set_cursor_grab(grab);
            show_mouse(!grab);
        } else if grab && clicked {
            // 웹: Esc는 브라우저가 포인터 잠금을 먼저 풀어 버린다(게임은 아직 잡은 줄 안다) — 누를 때마다 다시 청한다(누름 = 사용자 동작).
            set_cursor_grab(true);
        }
        a.drag = t.drag;
        a.point = a.point.or(t.drag_at);
        a.look += t.look;
        a.zoom += t.zoom;
        // 메뉴는 마우스 왼쪽을 뗄 때 탭(터치와 같은 규칙, nail 0199).
        if menu && mouse_real && a.tap.is_none() && is_mouse_button_released(MouseButton::Left) {
            a.tap = Some(mp);
        }
        // 지금 쓰는 장치 = 마지막으로 만진 것.
        if t.active {
            set_device(Device::Touch);
        } else if used_pad {
            set_device(Device::Pad);
        } else if used_keys || used_mouse {
            set_device(Device::Keys);
        }
        let ma = mouse_aim();
        a.cursor = (!self.grabbed)
            .then(|| cursor_live(ma.on, device(), self.since_mouse, ma.rest_secs()).then_some(mp))
            .flatten();

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
            cursor: a.cursor,
            aim_at: None,
            zoom: self.latch.zoom + a.zoom,
            look: self.latch.look + a.look,
            attack: self.latch.attack | a.attack,
            attack_held: a.attack_held,
            cast_held: a.cast_held,
            dodge_held: a.dodge_held,
            dodge: self.latch.dodge | a.dodge,
            parry: self.latch.parry | a.parry,
            jump: self.latch.jump | a.jump,
            flask: self.latch.flask | a.flask,
            interact: self.latch.interact | a.interact,
            cast: self.latch.cast | a.cast,
            lock: self.latch.lock | a.lock,
            turn_right: self.latch.turn_right | a.turn_right,
            turn_left: self.latch.turn_left | a.turn_left,
            skill1: self.latch.skill1 | a.skill1,
            skill2: self.latch.skill2 | a.skill2,
            tab_prev: self.latch.tab_prev | a.tab_prev,
            tab_next: self.latch.tab_next | a.tab_next,
            ult: self.latch.ult | a.ult,
            equip: self.latch.equip | a.equip,
            lab_toggle: self.latch.lab_toggle | a.lab_toggle,
            scenario: a.scenario.or(self.latch.scenario),
            menu_up: self.latch.menu_up | a.menu_up,
            menu_down: self.latch.menu_down | a.menu_down,
            menu_left: self.latch.menu_left | a.menu_left,
            menu_right: self.latch.menu_right | a.menu_right,
            confirm: self.latch.confirm | a.confirm,
            back: self.latch.back | a.back,
            pause: self.latch.pause | a.pause,
            tap: a.tap.or(self.latch.tap),
            drag: self.latch.drag + a.drag,
            wheel: self.latch.wheel + a.wheel,
            point: a.point.or(self.latch.point),
            dev_overlay: self.latch.dev_overlay | a.dev_overlay,
            ui_check: self.latch.ui_check | a.ui_check,
            step_clock: self.latch.step_clock | a.step_clock,
            pressed_key: a.pressed_key.or(self.latch.pressed_key),
            pressed_pad: a.pressed_pad.or(self.latch.pressed_pad),
        };
    }

    /// 로직 틱이 소비. 눌림은 지워지고 상태(이동·누르고 있음)는 남는다.
    pub fn take(&mut self) -> Actions {
        let a = self.latch;
        self.latch = carry(&a);
        a
    }
}

/// 틱이 눌림을 가져간 뒤 다음 틱에 남는 것: 이동·조준·쥐고 있음(상태). 눌림은 한 번만.
/// 0056: RB를 쥐고 있음을 빠뜨려 한 프레임의 둘째 틱부터 원거리 모으기가 끊겼다.
fn carry(a: &Actions) -> Actions {
    Actions {
        move_dir: a.move_dir,
        aim: a.aim,
        cursor: a.cursor,
        attack_held: a.attack_held,
        cast_held: a.cast_held,
        dodge_held: a.dodge_held,
        ..Actions::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 0056: 쥐고 있는 상태(공격·RB)는 틱이 눌림을 가져가도 남는다 — 한 프레임에 틱이 여럿이어도 모으기가 끊기지 않게.
    #[test]
    fn held_buttons_survive_take() {
        let first = Actions {
            attack_held: true,
            cast_held: true,
            dodge_held: true,
            cast: true,
            ..Default::default()
        };
        let second = carry(&first);
        assert!(!second.cast, "a press is taken once");
        assert!(
            second.cast_held && second.attack_held && second.dodge_held,
            "held stays"
        );
    }

    /// 0164: 장비 칸 빨리 바꾸기는 공격 버튼 — 그 버튼이 메뉴에서 이미 다른 일을 하는 장치·배치에서는 꺼진다.
    /// 패드 기본(X)은 되고, 키보드만 배치(J = 확인)·키보드+마우스(왼쪽 클릭 = 누르기)·터치는 안 된다. 공격을 다른 키로 옮기면 된다.
    #[test]
    fn quick_swap_is_off_where_the_attack_button_already_means_something_in_menus() {
        let (only, mouse) = (Bindings::keyboard_only(), Bindings::default_layout());
        assert!(swap_free_for(Device::Pad, &only));
        assert!(swap_free_for(Device::Pad, &mouse));
        assert!(!swap_free_for(Device::Keys, &only), "J confirms");
        assert!(!swap_free_for(Device::Keys, &mouse), "a click is a tap");
        assert!(!swap_free_for(Device::Touch, &only));
        let mut moved = only;
        moved.set_key(Bind::Attack, key_index(KeyCode::U));
        assert!(swap_free_for(Device::Keys, &moved));
        // 패드에서 공격을 확인(A) 자리로 옮기면 꺼진다.
        let mut pad = only;
        pad.pads[Bind::Attack as usize] = Pad::South;
        assert!(!swap_free_for(Device::Pad, &pad));
    }

    /// 0058: 커서는 마우스 조준이 켜져 있고, 장치가 키보드(+마우스)이고, 만진 지 쉼 시간 안일 때만 조준한다.
    /// 키보드만으로 하는 사람의 쉬는 마우스·패드·터치는 조준을 빼앗지 않는다.
    #[test]
    fn a_recently_touched_cursor_aims_and_a_resting_one_falls_back_to_auto() {
        let rest = MouseAim::default().rest_secs();
        assert_eq!(rest, 1.0);
        assert!(cursor_live(true, Device::Keys, 0.0, rest));
        assert!(cursor_live(true, Device::Keys, 0.9, rest));
        assert!(!cursor_live(true, Device::Keys, 1.0, rest), "rested");
        assert!(
            !cursor_live(true, Device::Keys, f32::MAX, rest),
            "never touched"
        );
        assert!(
            !cursor_live(false, Device::Keys, 0.0, rest),
            "mouse aim off"
        );
        assert!(!cursor_live(true, Device::Pad, 0.0, rest));
        assert!(!cursor_live(true, Device::Touch, 0.0, rest));
    }

    /// 0058: 키 칸 번호는 키 표 뒤에 마우스 버튼이 이어지고, 이름은 ASCII(어느 글꼴로도).
    #[test]
    fn mouse_buttons_follow_the_keys_in_the_slot_numbers() {
        assert_eq!(Slot::of(0), Slot::Key(KEYS[0].0));
        assert_eq!(Slot::of(KEYS.len() as u8), Slot::Mouse(MouseButton::Left));
        assert_eq!(key_name(KEYS.len() as u8 + 1), "RMB");
        assert_eq!(key_name(NSLOTS as u8), "?");
        let mut b = Bindings::default_layout();
        b.set_key(Bind::Dodge, mouse_index(MouseButton::Left));
        assert_eq!(b.key(Bind::Dodge), Slot::Mouse(MouseButton::Left));
        assert_eq!(b.key(Bind::Attack), Slot::Key(KeyCode::Space), "swapped");
        assert_eq!(Bindings::decode(&b.encode()), b);
    }

    /// gamepads 크레이트 축: 위로 밀면 y > 0(문서) → 앞으로 걷는다.
    #[test]
    fn pushing_a_stick_up_means_forward() {
        assert_eq!(stick((0.0, 1.0)), vec2(0.0, 1.0));
        assert_eq!(stick((-1.0, 0.0)), vec2(-1.0, 0.0));
    }

    /// Android 스틱(0022): 패치한 miniquad는 아래가 +y — 위로 밀면 앞(+y)으로. 데드존 안 떨림은 0, 스틱이 서 있으면 HAT.
    #[test]
    fn android_sticks_push_up_for_forward_and_fall_back_to_the_hat() {
        assert_eq!(android_axes([0.0; 6]), (Vec2::ZERO, Vec2::ZERO, Vec2::ZERO));
        assert_eq!(
            android_axes([0.1, -0.1, 0.15, 0.0, 0.0, 0.0]),
            (Vec2::ZERO, Vec2::ZERO, Vec2::ZERO)
        );
        let (mv, aim, _) = android_axes([0.0, -1.0, 1.0, 0.0, 0.0, 0.0]);
        assert_eq!(mv, vec2(0.0, 1.0), "left stick up = forward");
        assert_eq!(aim, vec2(1.0, 0.0), "right stick right = aim right");
        // 0151: 십자 키(HAT)는 걷지 않는다 — 따로 나온다(메뉴·땅 돌리기), 위가 +y.
        let (mv, _, hat) = android_axes([0.0, 0.0, 0.0, 0.0, -1.0, 1.0]);
        assert_eq!(mv, Vec2::ZERO, "the hat does not walk");
        assert_eq!(hat, vec2(-1.0, -1.0), "hat down-left");
        let (mv, _, hat) = android_axes([0.5, 0.0, 0.0, 0.0, 0.0, -1.0]);
        assert_eq!((mv, hat), (vec2(0.5, 0.0), vec2(0.0, 1.0)));
    }

    /// Android 패드 버튼: 패치한 miniquad 표(KEYCODE_BUTTON_* → F13~F24)와 `Pad::key`가 같은 짝이고 겹치지 않는다(nail 0075).
    #[test]
    fn android_pad_keys_match_patched_miniquad() {
        let table = include_str!("../third_party/miniquad/src/native/android/keycodes.rs");
        let name = |p: Pad| match p {
            Pad::South => "BUTTON_A",
            Pad::East => "BUTTON_B",
            Pad::West => "BUTTON_X",
            Pad::North => "BUTTON_Y",
            Pad::L1 => "BUTTON_L1",
            Pad::R1 => "BUTTON_R1",
            Pad::L2 => "BUTTON_L2",
            Pad::R2 => "BUTTON_R2",
            Pad::L3 => "BUTTON_THUMBL",
            Pad::R3 => "BUTTON_THUMBR",
            Pad::Start => "BUTTON_START",
            Pad::Select => "BUTTON_SELECT",
            Pad::DLeft => "DPAD_LEFT",
            Pad::DRight => "DPAD_RIGHT",
        };
        let mut seen = Vec::new();
        for p in PADS {
            let k = p.key();
            assert!(!seen.contains(&k), "{p:?} key clash");
            seen.push(k);
            let line = table
                .lines()
                .find(|l| l.trim_end().ends_with(&format!("// KEYCODE_{}", name(p))))
                .unwrap_or_else(|| panic!("KEYCODE_{} not in the patched table", name(p)));
            assert!(line.contains(&format!("KeyCode::{k:?}")), "{p:?}: {line}");
        }
    }

    /// iOS 패드(0137): 패치한 miniquad `ios_pad.rs`가 GameController 버튼을 `KEYS` 차례로 읽어 F13~F24로 넘긴다 —
    /// 그 차례(버튼 이름·키)가 게임의 `Pad::key`와 같은 짝이고, 축은 Android와 같은 배열(아래가 +y, 위로 민 스틱 → 앞으로).
    #[test]
    fn ios_pad_keys_match_patched_miniquad() {
        let src = include_str!("../third_party/miniquad/src/native/ios_pad.rs");
        let name = |p: Pad| match p {
            Pad::South => "buttonA",
            Pad::East => "buttonB",
            Pad::West => "buttonX",
            Pad::North => "buttonY",
            Pad::L1 => "leftShoulder",
            Pad::R1 => "rightShoulder",
            Pad::L2 => "leftTrigger",
            Pad::R2 => "rightTrigger",
            Pad::L3 => "leftThumbstickButton",
            Pad::R3 => "rightThumbstickButton",
            Pad::Start => "buttonMenu",
            Pad::Select => "buttonOptions",
            // 십자 키는 버튼이 아니라 HAT 축(dx·dy)으로 넘어온다.
            Pad::DLeft | Pad::DRight => "",
        };
        // 버튼 읽는 차례(msg_send![g, <이름>])와 KEYS 표의 차례가 같은 자리끼리 짝이다.
        let buttons: Vec<&str> = src
            .lines()
            .filter_map(|l| l.trim().strip_prefix("msg_send![g, "))
            .filter_map(|l| l.strip_suffix("],"))
            .collect();
        let keys: Vec<&str> = src
            .lines()
            .filter_map(|l| l.trim().strip_prefix("KeyCode::"))
            .filter_map(|l| l.strip_suffix(','))
            .collect();
        assert_eq!(buttons.len(), 12, "{buttons:?}");
        assert_eq!(keys.len(), 12, "{keys:?}");
        for p in PADS.into_iter().filter(|p| !name(*p).is_empty()) {
            let i = buttons.iter().position(|b| *b == name(p)).unwrap();
            assert_eq!(keys[i], format!("{:?}", p.key()), "{p:?}");
        }
        // 축: GameController는 위가 +라 뒤집어 Android와 같은 뜻으로 넘긴다.
        assert!(src.contains("[lx, -ly, rx, -ry, dx, -dy]"));
        let (mv, _, _) = android_axes([0.0, -1.0, 0.0, 0.0, 0.0, 0.0]);
        assert!(mv.y > 0.9, "stick pushed up walks forward");
        // D패드(스틱 가운데): HAT 칸은 걷지 않고 십자 키로 나온다(0151 — 오른쪽 = 땅 돌리기 오른쪽).
        let (mv, _, hat) = android_axes([0.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
        assert_eq!((mv, hat), (Vec2::ZERO, vec2(1.0, 0.0)));
    }

    /// Android 키 반복: 패치한 miniquad가 반복 이벤트의 repeat 플래그를 넘긴다(안 그러면 누르고 있는 동안 계속 새 누름, nail 0171).
    #[test]
    fn android_key_repeat_reaches_macroquad() {
        let java = include_str!("../third_party/miniquad/java/MainActivity.java");
        assert!(java.contains("QuadNative.surfaceOnKeyDown(keyCode, event.getRepeatCount() > 0)"));
        let rs = include_str!("../third_party/miniquad/src/native/android.rs");
        assert!(rs.contains(".key_down_event(keycode, self.keymods, repeat)"));
    }

    /// Android 몰입 모드(0046): API 30+에서 바를 실제로 숨기는 패치가 남아 있고, 앱이 전체 화면을 켠다.
    #[test]
    fn android_fullscreen_hides_the_system_bars() {
        let java = include_str!("../third_party/miniquad/java/MainActivity.java");
        assert!(java.contains(
            "c.hide(WindowInsets.Type.statusBars() | WindowInsets.Type.navigationBars())"
        ));
        assert!(java.contains("BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE"));
        let main = include_str!("main.rs");
        assert!(main.contains(r#"fullscreen: cfg!(target_os = "android")"#));
    }
}
