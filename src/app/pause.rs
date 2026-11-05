//! 멈춤·장비 창(0052·0064 M3) — 줄 표 + 고름(0066).
use super::menu::{Menu, Out};
use super::*;

/// 장비 창 줄(0064 M3): 무기, 장신구 세 칸, 돌아가기.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum EquipRow {
    Weapon,
    Charm(usize),
    Back,
}

pub(super) fn equip_rows() -> Vec<EquipRow> {
    let mut v = vec![EquipRow::Weapon];
    v.extend((0..crate::charms::SLOTS).map(EquipRow::Charm));
    v.push(EquipRow::Back);
    v
}

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

    /// 장비(0064 M3): 무기 줄 + 장신구 세 칸 + 돌아가기, 아래에 고른 칸의 설명.
    pub(super) fn equip_menu(&self) -> Menu {
        let s = s();
        let rows = equip_rows()
            .iter()
            .map(|r| match r {
                EquipRow::Weapon => ui::Row::step(s.menu_weapon, weapon_name(self.weapon)),
                EquipRow::Charm(i) => {
                    let name = self.progress.worn[*i]
                        .map_or(s.charm_empty, |id| tr(crate::charms::def(id).name));
                    ui::Row::step(fill(s.menu_charm_slot, &[&(i + 1).to_string()]), name)
                }
                EquipRow::Back => ui::Row::plain(s.controls_back),
            })
            .collect();
        // 설명 줄: 고른 칸의 장신구, 아직 하나도 없으면 얻는 곳.
        let note = match equip_rows().get(self.cursor) {
            _ if self.progress.owned.is_empty() => Some(s.equip_none_yet),
            Some(EquipRow::Charm(i)) => {
                self.progress.worn[*i].map(|id| tr(crate::charms::def(id).desc))
            }
            _ => None,
        };
        Menu {
            note_room: true,
            note: note.map(str::to_string),
            ..Menu::new(s.equip_title, 300.0, rows)
        }
    }

    pub(super) fn equip_act(&mut self, out: Out) {
        let rows = equip_rows();
        match out {
            Out::Step(i, d) => match rows.get(i) {
                Some(EquipRow::Weapon) => self.next_weapon(d),
                Some(EquipRow::Charm(slot)) => self.step_charm(*slot, d),
                _ => {}
            },
            Out::Pick(i) => match rows[i] {
                EquipRow::Weapon => self.next_weapon(1),
                EquipRow::Charm(slot) => self.step_charm(slot, 1),
                EquipRow::Back => self.go(Go::Back),
            },
        }
    }
}
