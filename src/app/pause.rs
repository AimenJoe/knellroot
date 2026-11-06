//! 멈춤 창(0052·0066) — 줄 표 + 고름. 장비 창은 `equip.rs`(0067), 장신구 돌려 끼우기 `next_charm`은 여기.
use super::menu::{Menu, Out};
use super::*;

/// 칸 `slot`에 낄 수 있는 다음(`step` ±1) 장신구: 빈 칸 + 가진 것 가운데 다른 칸에 없는 것(표 순서), 끝에서 감는다.
pub(super) fn next_charm(
    owned: &[crate::charms::Id],
    worn: &[Option<crate::charms::Id>; crate::charms::SLOTS],
    slot: usize,
    step: i32,
) -> Option<crate::charms::Id> {
    let mut opts: Vec<Option<crate::charms::Id>> = vec![None];
    for d in crate::charms::TABLE.iter() {
        let elsewhere = worn
            .iter()
            .enumerate()
            .any(|(j, w)| j != slot && *w == Some(d.id));
        if owned.contains(&d.id) && !elsewhere {
            opts.push(Some(d.id));
        }
    }
    let at = opts.iter().position(|o| *o == worn[slot]).unwrap_or(0) as i32;
    opts[(at + step).rem_euclid(opts.len() as i32) as usize]
}

impl App {
    pub(super) fn pause_menu(&self) -> Menu {
        let s = s();
        let rows = flow::PAUSE_ITEMS
            .iter()
            .map(|i| match i {
                PauseItem::Resume => ui::Row::plain(s.menu_resume),
                PauseItem::Weapon => ui::Row::step(s.menu_equip, weapon_name(self.weapon)),
                PauseItem::Settings => ui::Row::go(s.menu_settings),
                PauseItem::Restart => ui::Row::plain(s.menu_restart),
                PauseItem::ToTitle => ui::Row::plain(s.menu_to_title),
                #[cfg(feature = "dev")]
                PauseItem::Lab => ui::Row::go("LAB"),
            })
            .collect();
        Menu::new(s.pause_title, 260.0, rows)
    }

    pub(super) fn pause_act(&mut self, out: Out) {
        match out {
            Out::Step(i, d) => {
                if flow::PAUSE_ITEMS.get(i) == Some(&PauseItem::Weapon) {
                    self.next_weapon(d);
                }
            }
            Out::Pick(i) => {
                let (next, rebuild) = flow::choose_pause(flow::PAUSE_ITEMS[i]);
                if rebuild {
                    self.restart();
                }
                self.go(next);
            }
        }
    }
}
