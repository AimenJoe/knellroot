//! 큰 창의 설정 탭(0052 → 0164): 왼쪽 줄 표 — 이름 왼쪽 · 값 오른쪽(◀ 값 ▶), 좌우·확인·화살표 탭으로 바꾼다 — 오른쪽에 고른 줄이
//! 무엇을 하는지. 멈춤에서 열면 끝에 [개발] · 처음부터 · 타이틀로(묻는 창). 줄의 차례는 `flow::settings_rows`. 개발 창은 `dev.rs`(0083).
use super::menu::{Menu, Out};
use super::tabs::{Ln, PaneHead, HEAD_H};
use super::*;
use flow::Host;

/// 설정 줄이 하는 일(오른쪽 칸의 설명).
fn row_desc(r: SettingsRow) -> &'static str {
    let s = s();
    match r {
        SettingsRow::Language => s.set_language_d,
        SettingsRow::Fullscreen => s.set_fullscreen_d,
        SettingsRow::Shake => s.set_shake_d,
        SettingsRow::Volume => s.set_volume_d,
        SettingsRow::Tells => s.set_tells_d,
        SettingsRow::TellTiming => s.set_tell_timing_d,
        SettingsRow::MouseAim => s.set_mouse_aim_d,
        SettingsRow::CursorRest => s.set_cursor_rest_d,
        SettingsRow::Reticle => s.set_reticle_d,
        SettingsRow::Controls => s.set_controls_d,
        #[cfg(feature = "dev")]
        SettingsRow::Dev => crate::i18n::dev::t(crate::i18n::dev::DEV_ROW_D),
        SettingsRow::Restart => s.set_restart_d,
        SettingsRow::ToTitle => s.set_to_title_d,
    }
}

impl App {
    fn settings_rows(host: Host) -> Vec<SettingsRow> {
        flow::settings_rows(crate::screen::supported(), input::has_mouse(), host)
    }

    pub(super) fn settings_menu(&self, host: Host) -> Menu {
        let s = s();
        let mouse = input::mouse_aim();
        let on_off = |b: bool| if b { s.value_on } else { s.value_off };
        let rows = Self::settings_rows(host)
            .iter()
            .map(|r| match r {
                SettingsRow::Language => ui::Row::step(s.menu_language, s.lang_name),
                SettingsRow::Fullscreen => {
                    ui::Row::value(s.menu_fullscreen, on_off(crate::screen::active()))
                }
                SettingsRow::Shake => {
                    ui::Row::step(s.menu_shake, format!("{}%", self.shake as u32 * 10))
                }
                SettingsRow::Volume => {
                    ui::Row::step(s.menu_volume, format!("{}%", self.volume as u32 * 10))
                }
                SettingsRow::Tells if self.tells == 0 => ui::Row::step(s.menu_tells, s.value_off),
                SettingsRow::Tells => {
                    ui::Row::step(s.menu_tells, format!("{}%", self.tells as u32 * 10))
                }
                SettingsRow::TellTiming => {
                    ui::Row::step(s.menu_tell_timing, on_off(self.tell_timing))
                }
                SettingsRow::MouseAim => ui::Row::step(s.menu_mouse_aim, on_off(mouse.on)),
                SettingsRow::CursorRest => ui::Row::step(
                    s.menu_cursor_rest,
                    fill(s.value_secs, &[&format!("{}", mouse.rest_secs())]),
                ),
                SettingsRow::Reticle => ui::Row::step(
                    s.menu_reticle,
                    [s.reticle_small, s.reticle_medium, s.reticle_large]
                        [(mouse.size as usize).min(2)],
                ),
                SettingsRow::Controls => ui::Row::go(s.menu_controls),
                #[cfg(feature = "dev")]
                SettingsRow::Dev => ui::Row::go(crate::i18n::dev::t(crate::i18n::dev::DEV_TITLE)),
                SettingsRow::Restart => ui::Row::plain(s.menu_restart),
                SettingsRow::ToTitle => ui::Row::plain(s.menu_to_title),
            })
            .collect();
        Menu::new(s.menu_settings, tabs::LEFT_W, rows)
    }

    /// 설정 탭 그림: 왼쪽 줄 표, 오른쪽에 고른 줄의 이름 · 지금 값 · 하는 일.
    pub(super) fn draw_settings(&mut self, f: &Frame, host: Host) {
        let s = s();
        let scene = Scene::Settings { host };
        let m = self.settings_menu(host);
        let w = self.draw_tab_window(f, scene);
        self.draw_left_list(&w, &m.rows, 0.0);
        let rows = Self::settings_rows(host);
        let at = self.cursor.min(rows.len().saturating_sub(1));
        if let (Some(&r), Some(row)) = (rows.get(at), m.rows.get(at)) {
            self.draw_pane_head(
                &w,
                &PaneHead {
                    name: &row.label,
                    gold: true,
                    kind: row.value.clone(),
                    state: (String::new(), DIM),
                    icon: false,
                },
            );
            let lines: Vec<Ln> = self.pane_wrap(&w, row_desc(r), DIM);
            self.draw_pane(&w, HEAD_H, &lines);
        }
        let right = w.panel.x + w.panel.w;
        self.tab_foot(
            f,
            &w,
            scene,
            right,
            &[(Act::Choose, s.verb_choose), (Act::Confirm, s.verb_confirm)],
        );
    }

    pub(super) fn settings_act(&mut self, host: Host, out: Out) {
        let rows = Self::settings_rows(host);
        match out {
            Out::Step(i, step) => match rows.get(i) {
                Some(SettingsRow::Language) => self.next_lang(),
                Some(SettingsRow::Shake) => {
                    self.shake = (self.shake as i32 + step).clamp(0, 10) as u8;
                    self.save_prefs();
                }
                Some(SettingsRow::Tells) => {
                    self.tells = (self.tells as i32 + step).clamp(0, 10) as u8;
                    self.save_prefs();
                }
                Some(SettingsRow::TellTiming) => {
                    self.tell_timing = !self.tell_timing;
                    self.save_prefs();
                }
                Some(SettingsRow::Volume) => {
                    self.set_volume((self.volume as i32 + step).clamp(0, 10) as u8)
                }
                Some(
                    &r @ (SettingsRow::MouseAim | SettingsRow::CursorRest | SettingsRow::Reticle),
                ) => self.step_mouse(r, step),
                _ => {}
            },
            Out::Pick(i) => match rows[i] {
                SettingsRow::Language => self.next_lang(),
                SettingsRow::Fullscreen => self.toggle_fullscreen(),
                SettingsRow::Shake => {
                    self.shake = if self.shake >= 10 { 0 } else { self.shake + 1 };
                    self.save_prefs();
                }
                SettingsRow::Tells => {
                    self.tells = if self.tells >= 10 { 0 } else { self.tells + 1 };
                    self.save_prefs();
                }
                SettingsRow::TellTiming => {
                    self.tell_timing = !self.tell_timing;
                    self.save_prefs();
                }
                SettingsRow::Volume => self.set_volume(if self.volume >= 10 {
                    0
                } else {
                    self.volume + 1
                }),
                r @ (SettingsRow::MouseAim | SettingsRow::CursorRest | SettingsRow::Reticle) => {
                    self.step_mouse(r, 1)
                }
                r => {
                    if let Some(go) = flow::choose_settings(r) {
                        self.go(go);
                    }
                }
            },
        }
    }
}
