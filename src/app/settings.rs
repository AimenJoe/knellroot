//! 설정·개발 메뉴(0052, nail 설정 창 꼴 0066): 이름 왼쪽 · 값 오른쪽(◀ 값 ▶), 좌우·확인·화살표 탭으로 바꾼다.
use super::menu::{Menu, Out};
use super::*;

impl App {
    pub(super) fn settings_menu(&self) -> Menu {
        let s = s();
        let mouse = input::mouse_aim();
        let on_off = |b: bool| if b { s.value_on } else { s.value_off };
        let rows = flow::settings_rows(crate::screen::supported(), input::has_mouse())
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
                #[cfg(feature = "dev")]
                SettingsRow::Lab => ui::Row::go(s.menu_lab),
                SettingsRow::Controls => ui::Row::go(s.menu_controls),
                SettingsRow::Back => ui::Row::plain(s.controls_back),
            })
            .collect();
        Menu::new(s.settings_title, 340.0, rows)
    }

    pub(super) fn settings_act(&mut self, out: Out) {
        let rows = flow::settings_rows(crate::screen::supported(), input::has_mouse());
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
                SettingsRow::Fullscreen => crate::screen::toggle(),
                SettingsRow::Shake => {
                    self.shake = if self.shake >= 10 { 0 } else { self.shake + 1 };
                    self.save_prefs();
                }
                SettingsRow::Tells => {
                    self.tells = if self.tells >= 10 { 0 } else { self.tells + 1 };
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
                #[cfg(feature = "dev")]
                SettingsRow::Lab => self.go(Go::Open(Scene::Lab)),
                SettingsRow::Controls => self.go(Go::Open(Scene::Controls)),
                SettingsRow::Back => self.go(Go::Back),
            },
        }
    }

    /// 개발 메뉴(0052): 개발판에만, 영어 그대로(0013).
    #[cfg(feature = "dev")]
    pub(super) fn dev_menu(&self) -> Menu {
        let rows = dev_items()
            .iter()
            .map(|d| match d {
                DevItem::Gate => ui::Row::plain("start at the sanctum gate (boss)"),
                DevItem::Scenario => ui::Row::step("PoC", scenario::entry(self.sc).name),
                DevItem::Weapon => ui::Row::step("weapon", format!("{:?}", self.weapon)),
                DevItem::Back => ui::Row::plain("back"),
            })
            .collect();
        let note = (dev_items().get(self.cursor) == Some(&DevItem::Scenario))
            .then(|| scenario::entry(self.sc).checks.to_string());
        Menu {
            note_room: true,
            note,
            ..Menu::new("DEV", 340.0, rows)
        }
    }

    #[cfg(feature = "dev")]
    pub(super) fn dev_act(&mut self, out: Out) {
        let items = dev_items();
        match out {
            Out::Step(i, step) => match items.get(i) {
                Some(DevItem::Scenario) => {
                    self.sc = if step > 0 {
                        scenario::next(self.sc)
                    } else {
                        (0..scenario::LIST.len() - 1).fold(self.sc, |s, _| scenario::next(s))
                    }
                }
                Some(DevItem::Weapon) => self.next_weapon(step),
                _ => {}
            },
            Out::Pick(i) => match items[i] {
                DevItem::Gate => {
                    self.restart();
                    self.run.dev_skip_to_gate(&mut self.world);
                    self.cam = Camera::new(self.world.player.pos, &self.t);
                    self.go(Go::Root(Scene::Play));
                }
                DevItem::Scenario => {
                    let sc = self.sc;
                    self.world = scenario::build(sc);
                    self.world.set_weapon(self.weapon);
                    self.cam = Camera::new(self.world.player.pos, &self.t);
                    self.go(Go::Root(Scene::Sandbox));
                }
                DevItem::Weapon => self.next_weapon(1),
                DevItem::Back => self.go(Go::Back),
            },
        }
    }
}

/// 개발 메뉴 줄(0052).
#[cfg(feature = "dev")]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum DevItem {
    Gate,
    Weapon,
    /// PoC 시나리오(좌우로 고르고 확인으로 들어간다).
    Scenario,
    Back,
}

#[cfg(feature = "dev")]
pub(super) fn dev_items() -> Vec<DevItem> {
    vec![
        DevItem::Gate,
        DevItem::Weapon,
        DevItem::Scenario,
        DevItem::Back,
    ]
}
