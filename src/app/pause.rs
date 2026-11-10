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
        let rows = flow::pause_items()
            .iter()
            .map(|i| match i {
                PauseItem::Resume => ui::Row::plain(s.menu_resume),
                // 장비 창으로 들어가는 줄 — 든 무기 이름·좌우 바꾸기는 뺐다(사장 2026-11-07, 0081: 장비 창이 보여 준다).
                PauseItem::Weapon => ui::Row::go(s.menu_equip),
                PauseItem::Tree => ui::Row::go(s.tree_title),
                PauseItem::Codex => ui::Row::go(s.menu_codex),
                PauseItem::Settings => ui::Row::go(s.menu_settings),
                PauseItem::Restart => ui::Row::plain(s.menu_restart),
                PauseItem::ToTitle => ui::Row::plain(s.menu_to_title),
                #[cfg(feature = "dev")]
                PauseItem::Dev => ui::Row::go(crate::i18n::dev::t(crate::i18n::dev::DEV_TITLE)),
            })
            .collect();
        Menu::new(s.pause_title, 260.0, rows)
    }

    pub(super) fn pause_act(&mut self, out: Out) {
        match out {
            Out::Step(..) => {}
            Out::Pick(i) => {
                let (next, rebuild) = flow::choose_pause(flow::pause_items()[i]);
                if rebuild {
                    self.restart();
                }
                self.go(next);
            }
        }
    }
}
