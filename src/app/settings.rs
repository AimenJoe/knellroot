//! 설정·개발 메뉴(0052, nail 설정 창 꼴 0066): 이름 왼쪽 · 값 오른쪽(◀ 값 ▶), 좌우·확인·화살표 탭으로 바꾼다.
use super::menu::{Menu, Out};
use super::*;
#[cfg(feature = "dev")]
use crate::i18n::dev::{self as dv, scenario_text, t};

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
                SettingsRow::Controls => self.go(Go::Open(Scene::Controls)),
                SettingsRow::Back => self.go(Go::Back),
            },
        }
    }

    /// 개발 메뉴(0052 → 0075, nail 0269의 묶음 탭 꼴): 묶음 줄(◀ 묶음 ▶) · 무기 · 그 묶음에서 시작할 것들 · 돌아가기.
    /// 아래 설명 칸에 고른 줄의 설명. 개발판에만, 글은 언어를 따른다(`i18n::dev`, 0080).
    #[cfg(feature = "dev")]
    pub(super) fn dev_menu(&self, tab: usize) -> Menu {
        let tab = tab.min(DEV_TABS.len() - 1);
        let items = dev_items(tab);
        let rows = items
            .iter()
            .map(|it| match it {
                DevItem::Tab => ui::Row::step(t(dv::GROUP), tab_name(tab)),
                DevItem::Weapon => ui::Row::step(t(dv::WEAPON), weapon_name(self.weapon)),
                DevItem::Start(e) => ui::Row::plain(e.name()),
                DevItem::Back => ui::Row::plain(t(dv::BACK)),
            })
            .collect();
        let note = match items.get(self.cursor) {
            Some(DevItem::Start(e)) => Some(e.note()),
            Some(DevItem::Tab) => Some(t(dv::DEV_TAB_NOTE).to_string()),
            _ => None,
        };
        Menu {
            note_room: true,
            note,
            ..Menu::new(t(dv::DEV_TITLE), 360.0, rows)
        }
    }

    #[cfg(feature = "dev")]
    pub(super) fn dev_act(&mut self, tab: usize, out: Out) {
        let items = dev_items(tab);
        let n = DEV_TABS.len();
        let to_tab = |app: &mut App, step: i32| {
            let t = (tab as i32 + step).rem_euclid(n as i32) as u8;
            // 묶음은 장면 칸의 상태(0066): 지금 칸을 새 묶음으로 바꿔 연다(커서는 묶음 줄).
            app.go(Go::Back);
            app.go(Go::Open(Scene::Dev { tab: t }));
        };
        match out {
            Out::Step(i, step) => match items.get(i) {
                Some(DevItem::Tab) => to_tab(self, step),
                Some(DevItem::Weapon) => self.next_weapon(step),
                _ => {}
            },
            Out::Pick(i) => match items[i] {
                DevItem::Tab => to_tab(self, 1),
                DevItem::Weapon => self.next_weapon(1),
                DevItem::Start(e) => self.dev_start(e),
                DevItem::Back => self.go(Go::Back),
            },
        }
    }

    /// 개발 메뉴에서 시작(0075): 스테이지 판은 기록 슬롯에 쓰지 않는다(`dev_run` — 슬롯을 고르면 풀린다).
    #[cfg(feature = "dev")]
    fn dev_start(&mut self, e: DevStart) {
        match e {
            DevStart::Entrance | DevStart::Gate => {
                self.dev_run = true;
                self.restart();
                if e == DevStart::Gate {
                    self.run.dev_skip_to_gate(&mut self.world);
                }
                self.cam = Camera::new(self.world.player.pos, &self.t);
                self.go(Go::Root(Scene::Play));
            }
            DevStart::Scenario(sc) => {
                self.sc = sc;
                self.world = scenario::build(sc);
                self.world.set_weapon(self.weapon);
                self.cam = Camera::new(self.world.player.pos, &self.t);
                self.go(Go::Root(Scene::Sandbox));
            }
            DevStart::Lab => self.go(Go::Open(Scene::Lab)),
        }
    }
}

/// 묶음 이름(한국어면 이 표, 영어면 `DEV_TABS`의 이름).
#[cfg(feature = "dev")]
fn tab_name(tab: usize) -> &'static str {
    t([dv::TAB_COMBAT, dv::TAB_STAGE, dv::TAB_TOOLS][tab.min(2)])
}

/// 개발 메뉴 묶음(0075, nail 0269 `scenario::Cat`): 이름 · 시작할 것들. 첫 묶음 첫 줄 = 보스 아레나(시험 스크립트가 "아래로 둘 · 확인"으로 간다).
#[cfg(feature = "dev")]
pub(super) const DEV_TABS: [(&str, &[DevStart]); 3] = [
    (
        "combat",
        &[
            DevStart::Scenario(Scenario::Boss),
            DevStart::Scenario(Scenario::Pillars),
            DevStart::Scenario(Scenario::Duel),
            DevStart::Scenario(Scenario::Horde),
            DevStart::Scenario(Scenario::Tray),
        ],
    ),
    ("stage", &[DevStart::Entrance, DevStart::Gate]),
    (
        "perf & tools",
        &[
            DevStart::Scenario(Scenario::Empty),
            DevStart::Scenario(Scenario::Stress40),
            DevStart::Scenario(Scenario::Stress80),
            DevStart::Lab,
        ],
    ),
];

/// 개발 메뉴에서 시작할 것 하나.
#[cfg(feature = "dev")]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum DevStart {
    /// 첫 스테이지 처음부터(입구) — 기록은 쓰지 않는다.
    Entrance,
    /// 문 앞(문지기 결투) — 기록은 쓰지 않는다.
    Gate,
    /// PoC·개발 아레나(`scenario::LIST`).
    Scenario(Scenario),
    /// 연구소(메뉴의 설정 아래 줄과 같은 창, 0079).
    Lab,
}

#[cfg(feature = "dev")]
impl DevStart {
    fn name(self) -> &'static str {
        match self {
            DevStart::Entrance => t(dv::ENTRANCE),
            DevStart::Gate => t(dv::GATE),
            DevStart::Scenario(sc) => {
                let e = scenario::entry(sc);
                scenario_text(e.name, e.checks).0
            }
            DevStart::Lab => t(dv::LAB_START),
        }
    }

    fn note(self) -> String {
        match self {
            DevStart::Entrance | DevStart::Gate => t(dv::STAGE_NOTE).to_string(),
            DevStart::Scenario(sc) => {
                let n = scenario::LIST
                    .iter()
                    .position(|e| e.scenario == sc)
                    .unwrap_or(0)
                    + 1;
                let e = scenario::entry(sc);
                let checks = scenario_text(e.name, e.checks).1;
                fill(t(dv::SCENARIO_NOTE), &[&n.to_string(), checks])
            }
            DevStart::Lab => t(dv::LAB_NOTE).to_string(),
        }
    }
}

/// 개발 메뉴 줄(0052 → 0075).
#[cfg(feature = "dev")]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum DevItem {
    /// 묶음(좌우·확인으로 다음 묶음).
    Tab,
    Weapon,
    Start(DevStart),
    Back,
}

#[cfg(feature = "dev")]
pub(super) fn dev_items(tab: usize) -> Vec<DevItem> {
    let mut v = vec![DevItem::Tab, DevItem::Weapon];
    v.extend(
        DEV_TABS[tab.min(DEV_TABS.len() - 1)]
            .1
            .iter()
            .map(|&e| DevItem::Start(e)),
    );
    v.push(DevItem::Back);
    v
}

#[cfg(all(test, feature = "dev"))]
mod tests {
    use super::*;

    /// 0075: 시나리오는 모두 한 묶음에 꼭 한 번 — 새 시나리오를 표에 더하고 묶음에 안 넣으면 메뉴에서 못 고른다.
    #[test]
    fn every_scenario_sits_in_exactly_one_dev_group() {
        for e in scenario::LIST.iter() {
            let n = DEV_TABS
                .iter()
                .flat_map(|(_, v)| v.iter())
                .filter(|&&d| d == DevStart::Scenario(e.scenario))
                .count();
            assert_eq!(n, 1, "{}", e.name);
        }
        // 시험 스크립트(shot·mouse·leak): 첫 묶음 · 아래로 둘 = 보스 아레나.
        assert_eq!(
            dev_items(0)[2],
            DevItem::Start(DevStart::Scenario(Scenario::Boss))
        );
    }
}
