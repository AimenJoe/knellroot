//! 조작 설정 화면(0024) — 상태와 입력 규칙(순수, 시험). 그림은 `controls_screen.rs`.
//!
//! 줄 = 다시 붙일 동작(하는 일의 차례로 — 공격 둘·몸·기술·큰 기술·땅 돌리기, 0255) + 배치 둘("키보드+마우스 배치로"(기본) ·
//! "키보드만 배치로", 0058) + "돌아가기". 칸 = 키보드 | 패드(좌우로). 키보드 칸에는 마우스 버튼도 붙는다(0058).
//! 확인 → "누르세요… 5": 그 칸 장치의 다음 눌림을 붙인다(겹치면 맞바꾸기 — `Bindings::set_*`). 취소: 키보드 칸은 Esc, 패드 칸은
//! Start, 5초가 지나도. 확인을 누른 그 틱의 눌림은 잡지 않는다(Space·A는 확인이면서 붙일 수 있는 키라서) — 다음 틱부터.
//! 큰 기술 줄(0255)은 버튼 하나 또는 두 버튼 조합: 첫 눌림 뒤 `CHORD_WAIT` 안에 다른 버튼이 오면 조합, 아니면 하나. 하나는
//! 다른 동작이 쓰지 않는 버튼만(겹치면 알림 — 조합은 겹쳐도 된다), 스킬 두 버튼 조합이면 비운다(그것은 늘 큰 기술이다).
use crate::input::{Actions, Bind, Bindings, Pad};

/// 기다리는 초.
pub const CAPTURE_SECS: f32 = 5.0;
/// 큰 기술 조합의 둘째 버튼을 기다리는 초(첫 눌림 뒤).
pub const CHORD_WAIT: f32 = 0.4;
/// "다른 동작이 쓰는 버튼" 알림이 떠 있는 초.
pub const NOTE_SECS: f32 = 2.5;

/// 줄의 차례(0255 — 하는 일로 묶는다. 저장 칸 차례 `BINDS`와 다르다).
pub const ORDER: [Bind; crate::input::NB] = [
    Bind::Attack,
    Bind::Cast,
    Bind::Dodge,
    Bind::Parry,
    Bind::Jump,
    Bind::Flask,
    Bind::Lock,
    Bind::Skill1,
    Bind::Skill2,
    Bind::TurnLeft,
    Bind::TurnRight,
];

/// 통째로 되돌리는 배치(0058).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Preset {
    /// 키보드+마우스(새로 설치한 판의 기본값).
    Mouse,
    /// 키보드만(0024·0048 — 오른손 J·K·L·U).
    Keys,
}

impl Preset {
    pub fn layout(self) -> Bindings {
        match self {
            Preset::Mouse => Bindings::default_layout(),
            Preset::Keys => Bindings::keyboard_only(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Row {
    Bind(Bind),
    /// 큰 기술(0255).
    Ult,
    Preset(Preset),
    Back,
}

pub fn rows() -> Vec<Row> {
    let mut v = Vec::new();
    for b in ORDER {
        v.push(Row::Bind(b));
        if b == Bind::Skill2 {
            v.push(Row::Ult);
        }
    }
    v.push(Row::Preset(Preset::Mouse));
    v.push(Row::Preset(Preset::Keys));
    v.push(Row::Back);
    v
}

/// 큰 기술 줄의 첫 눌림(조합의 둘째를 기다린다).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum First {
    Key(u8),
    Pad(Pad),
}

/// 칸: 0 = 키보드, 1 = 패드.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Controls {
    pub row: usize,
    pub col: usize,
    /// 기다리는 중이면 남은 초.
    pub capture: Option<f32>,
    /// 큰 기술 줄: 첫 눌림과 둘째를 기다릴 남은 초.
    pub first: Option<(First, f32)>,
    /// "다른 동작이 쓰는 버튼" 알림의 남은 초.
    pub note: f32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Out {
    Stay,
    Changed(Bindings),
    Back,
}

/// 큰 기술 칸에 키(하나·둘)를 붙인 배치 — 하나가 다른 동작의 키면 `None`.
pub fn ult_keys(b: Bindings, keys: [Option<u8>; 2]) -> Option<Bindings> {
    let mut nb = b;
    let skills = [b.keys[Bind::Skill1 as usize], b.keys[Bind::Skill2 as usize]];
    nb.ult.keys = match keys {
        [Some(k), None] if b.keys.contains(&k) => return None,
        [Some(x), Some(y)] if skills.contains(&x) && skills.contains(&y) => [None, None],
        k => k,
    };
    Some(nb)
}

/// 큰 기술 칸에 패드 버튼(하나·둘)을 붙인 배치 — 하나가 다른 동작의 버튼이면 `None`.
pub fn ult_pads(b: Bindings, pads: [Option<Pad>; 2]) -> Option<Bindings> {
    let mut nb = b;
    let skills = [b.pad(Bind::Skill1), b.pad(Bind::Skill2)];
    nb.ult.pads = match pads {
        [Some(p), None] if b.pads.contains(&p) => return None,
        [Some(x), Some(y)] if skills.contains(&x) && skills.contains(&y) => [None, None],
        p => p,
    };
    Some(nb)
}

impl Controls {
    /// 줄·칸을 고르고 확인한 것처럼(터치 탭).
    pub fn pick(&mut self, row: usize, col: usize, b: Bindings) -> Out {
        self.row = row;
        self.col = col;
        self.confirm(b)
    }

    fn confirm(&mut self, b: Bindings) -> Out {
        match rows().get(self.row) {
            Some(Row::Bind(_) | Row::Ult) => {
                self.capture = Some(CAPTURE_SECS);
                self.first = None;
                Out::Stay
            }
            Some(Row::Preset(p)) => Out::Changed(p.layout()),
            Some(Row::Back) => Out::Back,
            None => Out::Stay,
        }
        .min_changed(b)
    }

    /// 큰 기술 줄을 붙인 결과: 바뀐 배치, 또는 겹쳐서 알림.
    fn settle(&mut self, got: Option<Bindings>, b: Bindings) -> Out {
        self.capture = None;
        self.first = None;
        match got {
            Some(nb) => Out::Changed(nb).min_changed(b),
            None => {
                self.note = NOTE_SECS;
                Out::Stay
            }
        }
    }

    /// 큰 기술 줄의 기다림(0255).
    fn tick_ult(&mut self, a: &Actions, b: Bindings, dt: f32) -> Out {
        let cancel = if self.col == 0 { a.back } else { a.pause };
        if cancel {
            self.capture = None;
            self.first = None;
            return Out::Stay;
        }
        let now = match (self.col, a.pressed_key, a.pressed_pad) {
            (0, Some(k), _) => Some(First::Key(k)),
            (1, _, Some(p)) => Some(First::Pad(p)),
            _ => None,
        };
        match (self.first, now) {
            (None, Some(f)) => self.first = Some((f, CHORD_WAIT)),
            (Some((First::Key(x), _)), Some(First::Key(y))) if x != y => {
                return self.settle(ult_keys(b, [Some(x), Some(y)]), b);
            }
            (Some((First::Pad(x), _)), Some(First::Pad(y))) if x != y => {
                return self.settle(ult_pads(b, [Some(x), Some(y)]), b);
            }
            (Some((f, left)), _) => {
                let left = left - dt;
                if left > 0.0 {
                    self.first = Some((f, left));
                } else {
                    let got = match f {
                        First::Key(k) => ult_keys(b, [Some(k), None]),
                        First::Pad(p) => ult_pads(b, [Some(p), None]),
                    };
                    return self.settle(got, b);
                }
            }
            (None, None) => {}
        }
        Out::Stay
    }

    pub fn tick(&mut self, a: &Actions, b: Bindings, dt: f32) -> Out {
        let rows = rows();
        self.note = (self.note - dt).max(0.0);
        if let Some(left) = self.capture {
            let left = left - dt;
            // 조합의 둘째를 기다리는 동안은 시간이 다 돼도 끝을 맺는다.
            self.capture = (left > 0.0 || self.first.is_some()).then_some(left.max(0.0));
            let bind = match rows.get(self.row).copied() {
                Some(Row::Bind(bind)) => bind,
                Some(Row::Ult) => return self.tick_ult(a, b, dt),
                _ => {
                    self.capture = None;
                    return Out::Stay;
                }
            };
            let mut nb = b;
            if self.col == 0 {
                if a.back {
                    self.capture = None;
                } else if let Some(k) = a.pressed_key {
                    nb.set_key(bind, k);
                    self.capture = None;
                    return Out::Changed(nb);
                }
            } else if a.pause {
                self.capture = None;
            } else if let Some(p) = a.pressed_pad {
                nb.set_pad(bind, p);
                self.capture = None;
                return Out::Changed(nb);
            }
            return Out::Stay;
        }
        let n = rows.len();
        if a.menu_up {
            self.row = (self.row + n - 1) % n;
        }
        if a.menu_down {
            self.row = (self.row + 1) % n;
        }
        if (a.menu_left || a.menu_right) && matches!(rows[self.row], Row::Bind(_) | Row::Ult) {
            self.col = 1 - self.col;
        }
        if a.back {
            return Out::Back;
        }
        if a.confirm {
            return self.confirm(b);
        }
        Out::Stay
    }
}

trait MinChanged {
    fn min_changed(self, b: Bindings) -> Out;
}

impl MinChanged for Out {
    /// 고른 배치가 이미 지금 배치면 바뀐 것이 없다(저장하지 않게).
    fn min_changed(self, b: Bindings) -> Out {
        match self {
            Out::Changed(nb) if nb == b => Out::Stay,
            o => o,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{Pad, Slot, KEYS};
    use macroquad::prelude::{KeyCode, MouseButton};

    fn press_confirm() -> Actions {
        Actions {
            confirm: true,
            pressed_key: Some(KEYS.iter().position(|k| k.1 == "SPACE").unwrap() as u8),
            ..Default::default()
        }
    }

    #[test]
    fn confirm_waits_for_the_next_key_and_swaps_on_conflict() {
        let b = Bindings::keyboard_only();
        let mut c = Controls::default(); // 첫 줄 = 공격, 키보드
                                         // 확인(Space)을 누른 그 틱엔 잡지 않는다.
        assert_eq!(c.tick(&press_confirm(), b, 1.0 / 60.0), Out::Stay);
        assert!(c.capture.is_some());
        // 다음 눌림 = K(패링이 쓰던 키) → 공격 = K, 패링 = 공격이 쓰던 J.
        let k = KEYS.iter().position(|x| x.1 == "K").unwrap() as u8;
        let out = c.tick(
            &Actions {
                pressed_key: Some(k),
                ..Default::default()
            },
            b,
            1.0 / 60.0,
        );
        let Out::Changed(nb) = out else {
            panic!("{out:?}")
        };
        assert_eq!(nb.keys[Bind::Attack as usize], k);
        assert_eq!(nb.keys[Bind::Parry as usize], b.keys[Bind::Attack as usize]);
        assert!(c.capture.is_none());
    }

    /// 0058: 두 배치는 키 칸만 다르고 패드 칸은 같다. 키보드+마우스는 왼쪽 클릭 공격·오른쪽 클릭 마법·휠 클릭 락온·Shift 패링,
    /// 땅 돌리기는 둘 다 Q·E로 남는다. 키보드 칸은 마우스 버튼 눌림도 받는다.
    #[test]
    fn the_mouse_preset_moves_only_the_key_column_and_keeps_the_turn() {
        let (m, k) = (Bindings::default_layout(), Bindings::keyboard_only());
        assert_eq!(m.pads, k.pads);
        assert_eq!(m.key(Bind::Attack), Slot::Mouse(MouseButton::Left));
        assert_eq!(m.key(Bind::Cast), Slot::Mouse(MouseButton::Right));
        assert_eq!(m.key(Bind::Lock), Slot::Mouse(MouseButton::Middle));
        assert_eq!(m.key(Bind::Parry), Slot::Key(KeyCode::LeftShift));
        for t in [
            Bind::TurnLeft,
            Bind::TurnRight,
            Bind::Dodge,
            Bind::Jump,
            Bind::Flask,
        ] {
            assert_eq!(m.key(t), k.key(t), "{t:?} stays with the left hand");
        }
        assert!(m.uses_mouse() && !k.uses_mouse());
        let mut c = Controls::default();
        c.tick(&press_confirm(), k, 0.016);
        let rmb = (KEYS.len() + 1) as u8;
        let out = c.tick(
            &Actions {
                pressed_key: Some(rmb),
                ..Default::default()
            },
            k,
            0.016,
        );
        assert!(
            matches!(out, Out::Changed(nb) if nb.key(Bind::Attack) == Slot::Mouse(MouseButton::Right))
        );
    }

    #[test]
    fn the_pad_column_takes_a_pad_button_and_start_cancels() {
        let b = Bindings::default_layout();
        let mut c = Controls {
            col: 1,
            ..Default::default()
        };
        c.tick(
            &Actions {
                confirm: true,
                pressed_pad: Some(Pad::South),
                ..Default::default()
            },
            b,
            0.016,
        );
        assert!(c.capture.is_some());
        // 키보드 눌림은 무시, Start면 취소.
        assert_eq!(
            c.tick(
                &Actions {
                    pressed_key: Some(0),
                    ..Default::default()
                },
                b,
                0.016
            ),
            Out::Stay
        );
        assert_eq!(
            c.tick(
                &Actions {
                    pause: true,
                    ..Default::default()
                },
                b,
                0.016
            ),
            Out::Stay
        );
        assert!(c.capture.is_none());
        // 다시: RB.
        c.tick(
            &Actions {
                confirm: true,
                ..Default::default()
            },
            b,
            0.016,
        );
        let out = c.tick(
            &Actions {
                pressed_pad: Some(Pad::R1),
                ..Default::default()
            },
            b,
            0.016,
        );
        assert!(matches!(out, Out::Changed(nb) if nb.pads[0] == Pad::R1));
    }

    /// 0255: 줄은 하는 일의 차례 — 모든 동작이 한 번씩, 첫 줄은 근접 공격, 큰 기술은 기술 2 다음.
    #[test]
    fn rows_list_every_bind_once_in_the_order_of_use() {
        let r = rows();
        for b in crate::input::BINDS {
            assert_eq!(r.iter().filter(|x| **x == Row::Bind(b)).count(), 1, "{b:?}");
        }
        assert_eq!(r[0], Row::Bind(Bind::Attack));
        let ult = r.iter().position(|x| *x == Row::Ult).unwrap();
        assert_eq!(r[ult - 1], Row::Bind(Bind::Skill2));
    }

    fn ult_row() -> usize {
        rows().iter().position(|x| *x == Row::Ult).unwrap()
    }

    fn key(name: &str) -> u8 {
        KEYS.iter().position(|x| x.1 == name).unwrap() as u8
    }

    /// 0255: 큰 기술 줄 — 다른 동작이 쓰지 않는 키 하나는 그 키, 둘을 잇달아 누르면 조합, 다른 동작의 키 하나는 알림만.
    #[test]
    fn the_ult_row_takes_a_free_key_a_two_key_combo_and_refuses_a_taken_single() {
        let b = Bindings::keyboard_only();
        let dt = 1.0 / 60.0;
        let mut c = Controls::default();
        c.pick(ult_row(), 0, b);
        let press = |k: u8| Actions {
            pressed_key: Some(k),
            ..Default::default()
        };
        // 하나(V — 아무도 안 쓴다): 기다림이 지나면 붙는다.
        assert_eq!(c.tick(&press(key("V")), b, dt), Out::Stay);
        let mut out = Out::Stay;
        for _ in 0..40 {
            out = c.tick(&Actions::default(), b, dt);
            if out != Out::Stay {
                break;
            }
        }
        let Out::Changed(nb) = out else {
            panic!("{out:?}")
        };
        assert_eq!(nb.ult.keys, [Some(key("V")), None]);
        assert_eq!(nb.keys, b.keys, "nothing else moves");
        // 조합(K = 패링 + L = 락온): 겹쳐도 된다.
        c.pick(ult_row(), 0, b);
        c.tick(&press(key("K")), b, dt);
        let out = c.tick(&press(key("L")), b, dt);
        assert!(matches!(out, Out::Changed(nb) if nb.ult.keys == [Some(key("K")), Some(key("L"))]));
        // 하나인데 다른 동작의 키(J = 근접 공격): 그대로 + 알림.
        c.pick(ult_row(), 0, b);
        c.tick(&press(key("J")), b, dt);
        for _ in 0..40 {
            assert_eq!(c.tick(&Actions::default(), b, dt), Out::Stay);
        }
        assert!(c.note > 0.0 && c.capture.is_none());
        // 스킬 두 키의 조합(Q+E)이면 비운다 — 그것은 늘 큰 기술이다.
        let mut set = b;
        set.ult.keys = [Some(key("V")), None];
        c.pick(ult_row(), 0, set);
        c.tick(&press(key("Q")), set, dt);
        let out = c.tick(&press(key("E")), set, dt);
        assert!(matches!(out, Out::Changed(nb) if nb.ult.keys == [None, None]));
    }

    /// 0255: 패드 칸도 같다 — L3(아무도 안 쓴다) 하나, LB+RB 조합, Start는 취소.
    #[test]
    fn the_ult_row_on_the_pad_takes_l3_or_a_combo_and_start_cancels() {
        let b = Bindings::default_layout();
        let dt = 1.0 / 60.0;
        let mut c = Controls::default();
        c.pick(ult_row(), 1, b);
        let press = |p: Pad| Actions {
            pressed_pad: Some(p),
            ..Default::default()
        };
        c.tick(&press(Pad::L1), b, dt);
        let out = c.tick(&press(Pad::R1), b, dt);
        assert!(matches!(out, Out::Changed(nb) if nb.ult.pads == [Some(Pad::L1), Some(Pad::R1)]));
        c.pick(ult_row(), 1, b);
        c.tick(&press(Pad::L3), b, dt);
        let mut out = Out::Stay;
        for _ in 0..40 {
            out = c.tick(&Actions::default(), b, dt);
            if out != Out::Stay {
                break;
            }
        }
        assert!(matches!(out, Out::Changed(nb) if nb.ult.pads == [Some(Pad::L3), None]));
        c.pick(ult_row(), 1, b);
        let stop = Actions {
            pause: true,
            ..Default::default()
        };
        assert_eq!(c.tick(&stop, b, dt), Out::Stay);
        assert!(c.capture.is_none());
    }

    #[test]
    fn waiting_times_out_and_reset_and_back_rows_work() {
        let b = Bindings::default_layout();
        let mut c = Controls::default();
        c.tick(
            &Actions {
                confirm: true,
                ..Default::default()
            },
            b,
            0.016,
        );
        for _ in 0..(CAPTURE_SECS * 60.0) as usize + 2 {
            c.tick(&Actions::default(), b, 1.0 / 60.0);
        }
        assert!(c.capture.is_none(), "gave up after {CAPTURE_SECS}s");
        let mut changed = b;
        changed.set_key(Bind::Dodge, 0);
        let reset = rows()
            .iter()
            .position(|r| *r == Row::Preset(Preset::Mouse))
            .unwrap();
        assert_eq!(
            c.pick(reset, 0, changed),
            Out::Changed(Bindings::default_layout())
        );
        assert_eq!(c.pick(reset, 0, b), Out::Stay, "already default");
        let keys = rows()
            .iter()
            .position(|r| *r == Row::Preset(Preset::Keys))
            .unwrap();
        assert_eq!(c.pick(keys, 0, b), Out::Changed(Bindings::keyboard_only()));
        let back = rows().iter().position(|r| *r == Row::Back).unwrap();
        assert_eq!(c.pick(back, 0, b), Out::Back);
        assert_eq!(
            Controls::default().tick(
                &Actions {
                    back: true,
                    ..Default::default()
                },
                b,
                0.016
            ),
            Out::Back
        );
    }
}
