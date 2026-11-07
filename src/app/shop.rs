//! 쉼터 화로(가게)·핵 고르기·결과(0064 M4, 0060) — 줄 표 + 고름(0066).
use super::menu::{Menu, Out};
use super::*;

impl App {
    /// 가게(nail 가게 꼴, 0104부터 화로 아래 창): 제목 줄 오른쪽에 가진 잔불, 줄 = 이름 … 값(모자라면 흐리게) + 떠나기, 아래에 설명.
    pub(super) fn shop_menu(&self) -> Menu {
        let s = s();
        let wares = crate::growth::wares(&self.progress.owned, &self.progress.growth);
        let name = |w: crate::growth::Ware| match w {
            crate::growth::Ware::Charm(id) => tr(crate::charms::def(id).name),
            crate::growth::Ware::Oil => s.shop_oil,
        };
        let have = self.world.embers;
        let mut rows: Vec<ui::Row> = wares
            .iter()
            .map(|&(w, price)| {
                ui::Row::value(name(w), fill(s.shop_item, &[&price.to_string()]))
                    .dimmed(have < price)
            })
            .collect();
        rows.push(ui::Row::plain(s.shop_leave));
        let note = match wares.get(self.cursor) {
            Some((crate::growth::Ware::Charm(id), _)) => Some(tr(crate::charms::def(*id).desc)),
            Some((crate::growth::Ware::Oil, _)) => Some(s.shop_oil_d),
            None if wares.is_empty() => Some(s.shop_empty),
            None => None,
        };
        Menu {
            purse: Some(fill(s.shop_embers, &[&have.to_string()])),
            note_room: true,
            note: note.map(str::to_string),
            ..Menu::new(s.hearth_shop, 300.0, rows)
        }
    }

    pub(super) fn shop_act(&mut self, out: Out) {
        let Out::Pick(i) = out else { return };
        let wares = crate::growth::wares(&self.progress.owned, &self.progress.growth);
        match wares.get(i).copied() {
            // 모자라면 묻지 않고 알림만(nail 가게와 같다 — 흐린 줄).
            Some((_, price)) if self.world.embers < price => {
                self.toast = Some((s().shop_poor.to_string(), 1.6));
                self.snd(crate::sfx::Id::Deny);
            }
            Some(_) => self.ask(flow::Ask::Buy(i)),
            None => self.go(Go::Back),
        }
    }

    /// 핵 고르기(0064 M4): 제목 + 한 줄 + 갈래(지금/상한). 골라야 끝나니 닫을 수 없다.
    pub(super) fn core_menu(&self) -> Menu {
        let s = s();
        let g = self.progress.growth;
        let rows = g
            .options()
            .iter()
            .map(|&p| {
                let name = match p {
                    crate::growth::Pick::Hp => s.grow_hp,
                    crate::growth::Pick::Flask => s.grow_flask,
                    crate::growth::Pick::Heal => s.grow_heal,
                };
                let n = (g.count(p) + 1).to_string();
                let m = crate::growth::cap(p).to_string();
                ui::Row::value(name, fill(s.grow_count, &[&n, &m]))
            })
            .collect();
        Menu {
            closable: false,
            sub: Some(s.core_sub.to_string()),
            ..Menu::new(s.core_title, 300.0, rows)
        }
    }

    pub(super) fn core_act(&mut self, out: Out) {
        let Out::Pick(i) = out else { return };
        let opts = self.progress.growth.options();
        self.progress.growth.add(opts[i]);
        self.world.apply_stats(self.progress.stats(), true);
        self.persist();
        self.go(Go::Root(Scene::Result { won: true }));
    }

    /// 결과(0060): 제목 + 한 줄(걸린 시간·쓰러뜨린 수 / 쓰러진 까닭) + 다시·타이틀로. 고르는 창이라 닫을 수 없다.
    pub(super) fn result_menu(&self, won: bool) -> Menu {
        let s = s();
        let rows = flow::RESULT_ITEMS
            .iter()
            .map(|i| {
                ui::Row::plain(match i {
                    ResultItem::Retry => s.menu_retry,
                    ResultItem::ToTitle => s.menu_to_title,
                })
            })
            .collect();
        let (title, line) = if won {
            (s.clear_title, hud::clear_line(&self.run, &self.world))
        } else {
            (s.defeat_title, s.defeat_sub.to_string())
        };
        Menu {
            closable: false,
            sub: Some(line),
            ..Menu::new(title, 340.0, rows)
        }
    }

    pub(super) fn result_act(&mut self, out: Out) {
        let Out::Pick(i) = out else { return };
        let (next, rebuild) = flow::choose_result(flow::RESULT_ITEMS[i]);
        if rebuild {
            self.restart();
        }
        self.go(next);
    }
}
