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
            self.menu
                .view
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

    /// 조작 설정(0024 → 0255): 창 틀 안에 동작 | 키보드 | 패드 표 — 줄은 하는 일의 차례로 묶고(묶음 사이 옅은 선), 버튼은 키캡,
    /// 고른 줄의 설명·기다림·알림은 창 아래 칸에. 고른 칸은 호박색, 기다리는 칸은 "누르세요… n".
    pub(super) fn draw_controls(&mut self, f: &Frame) {
        let s = s();
        self.ui.dim(0.7);
        let cx = f.ox + f.w * 0.5;
        let px = self.ui.px(1.0);
        let u = self.ui.u(1.0);
        let pw = (f.w * 0.9).min(px * 46.0);
        let panel = Rect::new(cx - pw * 0.5, f.oy + f.h * 0.04, pw, f.h * 0.8);
        let top = self.ui.panel(panel, Some(s.controls_title));
        let (x_name, x_key, x_pad) = (
            panel.x + u * 10.0,
            panel.x + panel.w * 0.63,
            panel.x + panel.w * 0.85,
        );
        let col_w = panel.w * 0.2;
        let mut y = top;
        self.ui.text_center(s.controls_keyboard, x_key, y, 1.0, DIM);
        self.ui.text_center(s.controls_pad, x_pad, y, 1.0, DIM);
        y += px * 1.5;
        let b = input::bindings();
        let rows = controls::rows();
        // 아래 칸 = 고른 줄의 설명(두 줄).
        let desc_top = panel.y + panel.h - px * 2.9;
        draw_line(
            panel.x + u * 5.0,
            desc_top - px * 0.35,
            panel.x + panel.w - u * 5.0,
            desc_top - px * 0.35,
            u.max(1.0),
            Color {
                a: 0.5,
                ..ui::BORDER
            },
        );
        let bottom = desc_top - px * 0.6;
        let n = rows.len();
        let gap = px * 1.36;
        let fits = (((bottom - y) / gap).floor() as usize).clamp(1, n);
        let first = self.menu.view.window(self.ctl.row, n, fits, gap);
        let y_top = y;
        for (i, row) in rows.iter().enumerate().skip(first).take(fits) {
            let sel = i == self.ctl.row;
            // 묶음의 첫 줄 위에 옅은 선(공격 · 몸 · 기술 · 땅 돌리기 · 배치).
            let group = matches!(
                row,
                controls::Row::Bind(
                    input::Bind::Dodge | input::Bind::Skill1 | input::Bind::TurnLeft
                ) | controls::Row::Preset(controls::Preset::Mouse)
            );
            if group && i != first {
                let ly = (y - px * 0.28).round();
                draw_line(
                    x_name,
                    ly,
                    panel.x + panel.w - u * 10.0,
                    ly,
                    u.max(1.0),
                    Color {
                        a: 0.22,
                        ..ui::BORDER
                    },
                );
            }
            match row {
                controls::Row::Bind(_) | controls::Row::Ult => {
                    let name = match row {
                        controls::Row::Bind(bind) => match bind {
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
                            input::Bind::Swap => s.verb_weapon_swap,
                        },
                        _ => s.ctl_ult,
                    };
                    if sel {
                        draw_rectangle(
                            panel.x + u * 4.0,
                            y - px * 0.22,
                            panel.w - u * 8.0,
                            px * 1.4,
                            Color::new(0.88, 0.6, 0.23, 0.07),
                        );
                    }
                    let name = self.ui.fit(name, x_key - col_w * 0.5 - x_name - px, 1.0);
                    self.ui
                        .text(&name, x_name, y, 1.0, if sel { AMBER } else { INK });
                    for (col, x) in [(0usize, x_key), (1usize, x_pad)] {
                        let on = sel && self.ctl.col == col;
                        let r = Rect::new(x - col_w * 0.5, y - px * 0.22, col_w, px * 1.4);
                        if on {
                            draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.88, 0.6, 0.23, 0.18));
                        }
                        match (on, self.ctl.capture) {
                            (true, Some(left)) => {
                                let t =
                                    fill(s.controls_press, &[&format!("{}", left.ceil() as i32)]);
                                let t = self.ui.fit(&t, col_w, 1.0);
                                self.ui.text_center(&t, x, y, 1.0, AMBER);
                            }
                            _ => {
                                let (label, faint) = self.binding_label(b, *row, col);
                                // 붙인 것이 없는 큰 기술: 스킬 두 버튼 + "함께"를 흐리게(키캡 밖에).
                                let (t, c) = if faint {
                                    (
                                        format!("{} {}", ui::key(&label), s.controls_both),
                                        Color { a: 0.5, ..INK },
                                    )
                                } else {
                                    (ui::key(&label), INK)
                                };
                                let t = self.ui.fit(&t, col_w, 1.0);
                                self.ui.text_center(&t, x, y, 1.0, c);
                            }
                        }
                        self.menu.hits.push((r, Hit::Cell(i, col)));
                    }
                }
                controls::Row::Preset(_) | controls::Row::Back => {
                    let label = match row {
                        controls::Row::Preset(controls::Preset::Mouse) => s.controls_preset_mouse,
                        controls::Row::Preset(controls::Preset::Keys) => s.controls_preset_keys,
                        _ => s.controls_back,
                    };
                    let w = self.ui.width(label, 1.0);
                    let r = Rect::new(cx - w * 0.5 - px, y - px * 0.2, w + px * 2.0, px * 1.4);
                    if sel {
                        draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.88, 0.6, 0.23, 0.18));
                    }
                    self.ui
                        .text_center(label, cx, y, 1.0, if sel { AMBER } else { INK });
                    self.menu.hits.push((r, Hit::Cell(i, 0)));
                }
            }
            y += gap;
        }
        let arrows = self.ui.scroll_arrows(
            &self.menu.view,
            n,
            cx,
            y_top - px * 0.95,
            y_top + gap * fits as f32 - px * 0.15,
        );
        // ▲▼는 칸보다 먼저(겹치면 굴리기가 이긴다).
        for (r, d) in arrows.iter().zip([-1, 1]) {
            if let Some(r) = r {
                self.menu.hits.insert(0, (*r, Hit::Scroll(d)));
            }
        }
        // 아래 칸: 알림 > 큰 기술의 기다림 > 고른 줄의 설명.
        let row = rows.get(self.ctl.row).copied();
        let (text, color) = if self.ctl.note > 0.0 {
            (s.controls_taken.to_string(), AMBER)
        } else if let (Some(controls::Row::Ult), Some(_)) = (row, self.ctl.capture) {
            (s.controls_press_chord.to_string(), AMBER)
        } else {
            let d = match row {
                Some(controls::Row::Bind(bind)) => match bind {
                    input::Bind::Attack => s.ctl_attack_d,
                    input::Bind::Cast => s.ctl_cast_d,
                    input::Bind::Dodge => s.ctl_dodge_d,
                    input::Bind::Parry => s.ctl_parry_d,
                    input::Bind::Jump => s.ctl_jump_d,
                    input::Bind::Flask => s.ctl_flask_d,
                    input::Bind::Lock => s.ctl_lock_d,
                    input::Bind::Skill1 | input::Bind::Skill2 => s.ctl_skill_d,
                    input::Bind::TurnLeft | input::Bind::TurnRight => s.ctl_turn_d,
                    input::Bind::Swap => s.ctl_swap_d,
                },
                Some(controls::Row::Ult) => s.ctl_ult_d,
                Some(controls::Row::Preset(_)) => s.ctl_preset_d,
                _ => "",
            };
            (d.to_string(), DIM)
        };
        self.ui
            .text_center_wrapped(&text, cx, desc_top, 1.0, color, panel.w - u * 20.0);
        self.ui
            .text_center(s.controls_fixed, cx, f.oy + f.h - px * 3.2, 1.0, DIM);
        self.menu_hint(cx, f.oy + f.h - px * 1.8, f.w * 0.94);
    }

    /// 칸의 버튼 이름(0255): 동작은 그 버튼, 큰 기술은 붙인 것(하나·조합) — 없으면 스킬 두 버튼 "함께"를 흐리게(늘 되는 길).
    fn binding_label(&self, b: input::Bindings, row: controls::Row, col: usize) -> (String, bool) {
        match row {
            controls::Row::Bind(bind) if col == 0 => {
                (input::key_name(b.keys[bind as usize]).to_string(), false)
            }
            controls::Row::Bind(bind) => (b.pad(bind).name().to_string(), false),
            _ => {
                let names: Vec<String> = if col == 0 {
                    b.ult
                        .keys
                        .iter()
                        .flatten()
                        .map(|k| input::key_name(*k).to_string())
                        .collect()
                } else {
                    b.ult
                        .pads
                        .iter()
                        .flatten()
                        .map(|p| p.name().to_string())
                        .collect()
                };
                if !names.is_empty() {
                    return (names.join("+"), false);
                }
                let (x, y) = if col == 0 {
                    (
                        input::key_name(b.keys[input::Bind::Skill1 as usize]).to_string(),
                        input::key_name(b.keys[input::Bind::Skill2 as usize]).to_string(),
                    )
                } else {
                    (
                        b.pad(input::Bind::Skill1).name().to_string(),
                        b.pad(input::Bind::Skill2).name().to_string(),
                    )
                };
                (format!("{x}+{y}"), true)
            }
        }
    }
}
