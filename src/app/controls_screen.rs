//! 조작 설정 화면(0024·0058): 동작 | 키보드 | 패드 표. 표의 규칙은 `controls.rs`, 여기는 그 화면의 입력·그림(0066 R3).
use super::*;

impl App {
    /// 조작 설정(0024): 칸 고르기·다시 붙이기, 굴리기·탭은 메뉴와 같다.
    pub(super) fn tick_controls(&mut self, a: Actions) {
        let b = input::bindings();
        let mut out = self.ctl.tick(&a, b, TICK_DT);
        if self.ctl.capture.is_none() {
            // 굴리기(끌기·휠·▲▼)는 메뉴와 같다 — 커서 = 고른 줄.
            let tap = a.tap.and_then(|p| self.hit_at(p));
            let arrow = match tap {
                Some(Hit::Scroll(d)) => d,
                _ => 0,
            };
            let n = controls::rows().len();
            self.view
                .scroll((a.drag, a.wheel + arrow), &mut self.ctl.row, n);
            if let Some(Hit::Cell(row, col)) = tap {
                out = self.ctl.pick(row, col, b);
            }
        }
        match out {
            controls::Out::Stay => {}
            controls::Out::Changed(nb) => {
                input::set_bindings(nb);
                self.save_prefs();
            }
            controls::Out::Back => self.go(flow::back(self.scene())),
        }
    }

    /// 조작 설정(0024): 동작 | 키보드 | 패드 표, 고른 칸은 호박색, 기다리는 칸은 "누르세요… n".
    pub(super) fn draw_controls(&mut self, f: &Frame) {
        let s = s();
        self.ui.dim(0.7);
        let cx = f.ox + f.w * 0.5;
        let px = self.ui.px(1.0);
        self.ui
            .text_center(s.controls_title, cx, f.oy + f.h * 0.07, 2.0, INK);
        let (x_name, x_key, x_pad) = (f.ox + f.w * 0.16, f.ox + f.w * 0.56, f.ox + f.w * 0.78);
        let mut y = f.oy + f.h * 0.17;
        self.ui.text_center(s.controls_keyboard, x_key, y, 1.0, DIM);
        self.ui.text_center(s.controls_pad, x_pad, y, 1.0, DIM);
        y += px * 1.4;
        let b = input::bindings();
        let rows = controls::rows();
        // 줄 간격은 아래 고정 안내(`controls_fixed`) 위까지 들어가게 조금 줄이고(1.32 → 1.15줄), 그래도 넘치면 굴린다(메뉴와 같은 `ListView`).
        let bottom = f.oy + f.h - px * 3.2 - px * 1.2;
        let n = rows.len();
        let gap = (px * 1.32)
            .min((bottom - y) / (n as f32 - 1.0 + 0.4))
            .max(px * 1.15);
        let fits = if gap * (n as f32 - 1.0 + 0.4) <= bottom - y + 1.0 {
            n
        } else {
            y += px * 0.9; // ▲ 자리
            ((bottom - y - px * 0.9) / gap).floor().max(1.0) as usize
        };
        let first = self.view.window(self.ctl.row, n, fits, gap);
        let y_top = y;
        for (i, row) in rows.iter().enumerate().skip(first).take(fits) {
            let sel = i == self.ctl.row;
            match row {
                controls::Row::Bind(bind) => {
                    let name = match bind {
                        // 꾹 눌러 강공격(0030)은 여기서 알린다 — 타이틀의 긴 조작 줄을 걷었다(nail 0153).
                        input::Bind::Attack => s.verb_attack_hint,
                        input::Bind::Dodge => s.verb_dodge_hint,
                        input::Bind::Parry => s.verb_parry,
                        input::Bind::Lock => s.verb_lock,
                        input::Bind::TurnLeft => s.verb_turn_left,
                        input::Bind::TurnRight => s.verb_turn_right,
                        input::Bind::Jump => s.verb_jump_interact,
                        input::Bind::Flask => s.verb_flask,
                        input::Bind::Cast => s.verb_cast,
                        input::Bind::Skill1 => s.verb_skill1,
                        input::Bind::Skill2 => s.verb_skill2,
                    };
                    let name = self.ui.fit(name, x_key - x_name - px * 2.5, 1.0);
                    self.ui
                        .text(&name, x_name, y, 1.0, if sel { AMBER } else { INK });
                    for (col, x) in [(0usize, x_key), (1usize, x_pad)] {
                        let on = sel && self.ctl.col == col;
                        let label = match (on, self.ctl.capture) {
                            (true, Some(left)) => {
                                fill(s.controls_press, &[&format!("{}", left.ceil() as i32)])
                            }
                            _ if col == 0 => input::key_name(b.keys[*bind as usize]).to_string(),
                            _ => b.pad(*bind).name().to_string(),
                        };
                        let w = self.ui.width(&label, 1.0).max(px * 3.0);
                        let r = Rect::new(x - w * 0.5 - px * 0.5, y - px * 0.2, w + px, px * 1.4);
                        if on {
                            draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.88, 0.6, 0.23, 0.18));
                        }
                        self.ui
                            .text_center(&label, x, y, 1.0, if on { AMBER } else { INK });
                        self.hits.push((r, Hit::Cell(i, col)));
                    }
                }
                controls::Row::Preset(_) | controls::Row::Back => {
                    let label = match row {
                        controls::Row::Preset(controls::Preset::Mouse) => s.controls_preset_mouse,
                        controls::Row::Preset(controls::Preset::Keys) => s.controls_preset_keys,
                        _ => s.controls_back,
                    };
                    let yy = y + px * 0.4;
                    let w = self.ui.width(label, 1.0);
                    let r = Rect::new(cx - w * 0.5 - px, yy - px * 0.2, w + px * 2.0, px * 1.4);
                    if sel {
                        draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.88, 0.6, 0.23, 0.18));
                    }
                    self.ui
                        .text_center(label, cx, yy, 1.0, if sel { AMBER } else { INK });
                    self.hits.push((r, Hit::Cell(i, 0)));
                }
            }
            y += gap;
        }
        let arrows = self.ui.scroll_arrows(
            &self.view,
            n,
            cx,
            y_top - px * 0.95,
            y_top + gap * fits as f32 - px * 0.15,
        );
        // ▲▼는 칸보다 먼저(겹치면 굴리기가 이긴다).
        for (r, d) in arrows.iter().zip([-1, 1]) {
            if let Some(r) = r {
                self.hits.insert(0, (*r, Hit::Scroll(d)));
            }
        }
        self.ui
            .text_center(s.controls_fixed, cx, f.oy + f.h - px * 3.2, 1.0, DIM);
        self.menu_hint(cx, f.oy + f.h - px * 1.8, f.w * 0.94);
    }
}
