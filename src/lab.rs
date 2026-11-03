//! Lab(KB guides/poc-and-lab-before-content.md): 실행 중에 `Tuning`을 바꾸고, 슬롯 A/B를 번갈아 느끼고,
//! `tuning.rs`에 붙여 넣을 줄을 내보낸다. 커서 없이(패드 십자키·키보드) 동작한다. 개발 전용 화면이라 글은 영어 ASCII.
use macroquad::prelude::*;

use crate::input::Actions;
use crate::tuning::{Tuning, SPECS};

pub struct Lab {
    pub open: bool,
    cursor: usize,
    slot: Option<Tuning>,
    pub using_b: bool,
    pub note: String,
}

impl Lab {
    pub fn new() -> Self {
        Lab {
            open: false,
            cursor: 0,
            slot: None,
            using_b: false,
            note: String::new(),
        }
    }

    /// 열려 있을 때 틱마다. 내보내기를 눌렀으면 그 글을 돌려준다(앱이 콘솔에 찍는다).
    pub fn update(&mut self, a: &Actions, t: &mut Tuning) -> Option<String> {
        if a.lab_up {
            self.cursor = (self.cursor + SPECS.len() - 1) % SPECS.len();
        }
        if a.lab_down {
            self.cursor = (self.cursor + 1) % SPECS.len();
        }
        let s = &SPECS[self.cursor];
        let step = if a.lab_fast { s.step * 10.0 } else { s.step };
        if a.lab_left {
            t.set(self.cursor, t.get(self.cursor) - step);
        }
        if a.lab_right {
            t.set(self.cursor, t.get(self.cursor) + step);
        }
        if a.lab_reset {
            t.reset(self.cursor);
        }
        if a.lab_store {
            self.slot = Some(t.clone());
            self.using_b = false;
            self.note = "stored current values as slot B (V to swap)".into();
        }
        if a.lab_swap {
            if let Some(b) = self.slot.as_mut() {
                std::mem::swap(t, b);
                self.using_b = !self.using_b;
                self.note = format!("now using {}", if self.using_b { "B" } else { "A" });
            } else {
                self.note = "no slot B yet (C stores one)".into();
            }
        }
        if a.lab_export {
            self.note = "exported to console (paste into src/tuning.rs)".into();
            return Some(t.export());
        }
        None
    }

    /// `touch`면 폰 화면에 맞춘 한 줄짜리(고른 값 하나 + 아래 줄 버튼은 `touch.rs`가 그린다).
    pub fn draw(&self, t: &Tuning, touch: bool) {
        if !self.open {
            return;
        }
        if touch {
            self.draw_compact(t);
            return;
        }
        let sw = screen_width();
        let w = 420.0_f32.min(sw * 0.45);
        let x = sw - w - 8.0;
        let line = 18.0;
        // 값이 화면보다 많으면(55개 — 720px에 38줄) 커서를 따라 창을 굴린다. 예전엔 위 38줄만 그려 아래 값을 골라도 안 보였다.
        let rows =
            (((screen_height() - 42.0 - line * 3.5) / line).floor() as usize).clamp(5, SPECS.len());
        let start = window_start(self.cursor, SPECS.len(), rows);
        let h = line * (rows as f32 + 4.0) + 16.0;
        draw_rectangle(x, 8.0, w, h, Color::new(0.0, 0.0, 0.0, 0.75));
        draw_text(
            "LAB  up/down pick  left/right change (Ctrl/LB x10)  R reset",
            x + 8.0,
            26.0,
            16.0,
            WHITE,
        );
        draw_text(
            "X export  C store B  V swap A/B  1-7/Start scenario  F1 close",
            x + 8.0,
            42.0,
            16.0,
            GRAY,
        );
        for (i, s) in SPECS.iter().enumerate().skip(start).take(rows) {
            let y = 42.0 + line * ((i - start) as f32 + 1.5);
            let sel = i == self.cursor;
            let v = t.get(i);
            let changed = (v - s.def).abs() > 1e-6;
            let col = if sel {
                YELLOW
            } else if changed {
                ORANGE
            } else {
                LIGHTGRAY
            };
            let mark = if sel { ">" } else { " " };
            draw_text(format!("{mark} {:<24}", s.name), x + 8.0, y, 16.0, col);
            let vs = if s.step >= 1.0 {
                format!("{v:.0}")
            } else if s.step >= 0.1 {
                format!("{v:.2}")
            } else {
                format!("{v:.3}")
            };
            draw_text(&vs, x + w - 70.0, y, 16.0, col);
        }
        let y = 42.0 + line * (rows as f32 + 2.5);
        let below = SPECS.len() - start - rows;
        draw_text(
            format!(
                "{}/{}{}{}   slot: {}   {}",
                self.cursor + 1,
                SPECS.len(),
                if start > 0 {
                    format!("  ^{start} more")
                } else {
                    String::new()
                },
                if below > 0 {
                    format!("  v{below} more")
                } else {
                    String::new()
                },
                if self.using_b { "B" } else { "A" },
                self.note
            ),
            x + 8.0,
            y,
            16.0,
            SKYBLUE,
        );
    }

    fn draw_compact(&self, t: &Tuning) {
        let (sw, sh) = (screen_width(), screen_height());
        let u = sw.min(sh);
        let s = &SPECS[self.cursor];
        let v = t.get(self.cursor);
        let text = format!(
            "{}/{}  {}  {}{}   [{}] {}",
            self.cursor + 1,
            SPECS.len(),
            s.name,
            value_text(v, s.step),
            if (v - s.def).abs() > 1e-6 { " *" } else { "" },
            if self.using_b { "B" } else { "A" },
            self.note
        );
        // 오른쪽 위 dev 버튼 앞까지 들어가게 글자를 줄인다(폰 가로 폭에서 메모가 길면 버튼에 닿았다).
        let avail = sw - crate::touch::dev_buttons_width(sw, sh) - 16.0;
        let mut size = crate::text::fit((u * 0.06).max(14.0));
        let mut m = measure_text(&text, None, size as u16, 1.0);
        if m.width > avail {
            size = crate::text::fit(size * avail / m.width);
            m = measure_text(&text, None, size as u16, 1.0);
        }
        let x = ((avail - m.width) * 0.5).max(8.0);
        let y = crate::touch::lab_row_y(sw, sh) - crate::touch::lab_button_radius(sw, sh) - 8.0;
        draw_rectangle(
            x - 8.0,
            y - size,
            m.width + 16.0,
            size * 1.4,
            Color::new(0.0, 0.0, 0.0, 0.7),
        );
        draw_text(&text, x, y, size, YELLOW);
    }
}

/// 굴리는 목록의 첫 줄: 커서를 가운데쯤에 두되 끝에서는 멈춘다.
fn window_start(cursor: usize, len: usize, rows: usize) -> usize {
    if len <= rows {
        return 0;
    }
    cursor.saturating_sub(rows / 2).min(len - rows)
}

fn value_text(v: f32, step: f32) -> String {
    if step >= 1.0 {
        format!("{v:.0}")
    } else if step >= 0.1 {
        format!("{v:.2}")
    } else {
        format!("{v:.3}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_lab_list_scrolls_so_the_picked_value_is_always_on_screen() {
        for rows in [5, 20, 38] {
            for cursor in 0..SPECS.len() {
                let start = window_start(cursor, SPECS.len(), rows);
                assert!(
                    cursor >= start && cursor < start + rows,
                    "{cursor} {start} {rows}"
                );
                assert!(start + rows <= SPECS.len());
            }
        }
        assert_eq!(window_start(3, 10, 20), 0);
    }
}
