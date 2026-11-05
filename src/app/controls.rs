//! 조작 설정 화면(0024) — 상태와 입력 규칙(순수, 시험). 그림은 `app/mod.rs`의 `draw_controls`.
//!
//! 줄 = 다시 붙일 동작 + 배치 둘("키보드+마우스 배치로"(기본) · "키보드만 배치로", 0058 — 키 칸만 바꾸고 패드 칸은 기본값) + "돌아가기".
//! 칸 = 키보드 | 패드(좌우로). 키보드 칸에는 마우스 버튼(왼쪽·오른쪽·휠 클릭)도 붙는다(0058). 확인 → "누르세요… 5":
//! 그 칸 장치의 다음 눌림을 붙인다(겹치면 맞바꾸기 — `Bindings::set_*`). 취소: 키보드 칸은 Esc, 패드 칸은 Start, 5초가 지나도.
//! 확인을 누른 그 틱의 눌림은 잡지 않는다(Space·A는 확인이면서 붙일 수 있는 키라서) — 다음 틱부터.
use crate::input::{Actions, Bind, Bindings, BINDS};

/// 기다리는 초.
pub const CAPTURE_SECS: f32 = 5.0;

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
    Preset(Preset),
    Back,
}

pub fn rows() -> Vec<Row> {
    let mut v: Vec<Row> = BINDS.iter().map(|b| Row::Bind(*b)).collect();
    v.push(Row::Preset(Preset::Mouse));
    v.push(Row::Preset(Preset::Keys));
    v.push(Row::Back);
    v
}

/// 칸: 0 = 키보드, 1 = 패드.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Controls {
    pub row: usize,
    pub col: usize,
    /// 기다리는 중이면 남은 초.
    pub capture: Option<f32>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Out {
    Stay,
    Changed(Bindings),
    Back,
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
            Some(Row::Bind(_)) => {
                self.capture = Some(CAPTURE_SECS);
                Out::Stay
            }
            Some(Row::Preset(p)) => Out::Changed(p.layout()),
            Some(Row::Back) => Out::Back,
            None => Out::Stay,
        }
        .min_changed(b)
    }

    pub fn tick(&mut self, a: &Actions, b: Bindings, dt: f32) -> Out {
        let rows = rows();
        if let Some(left) = self.capture {
            let Some(Row::Bind(bind)) = rows.get(self.row).copied() else {
                self.capture = None;
                return Out::Stay;
            };
            let left = left - dt;
            self.capture = (left > 0.0).then_some(left);
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
        if (a.menu_left || a.menu_right) && matches!(rows[self.row], Row::Bind(_)) {
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
