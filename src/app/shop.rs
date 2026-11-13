//! 큰 창의 가게 탭(0064 M4 → 0164)과 결과 창(0060) — 줄 표 + 고름(0066). 핵 고르기는 0105에서 걷었다.
use super::equip::FLAVOR;
use super::menu::{Menu, Out};
use super::tabs::{Ln, PaneHead, HEAD_H};
use super::*;

/// 가게 물건의 이름.
pub(super) fn ware_name(w: crate::growth::Ware) -> &'static str {
    match w {
        crate::growth::Ware::Charm(id) => tr(crate::charms::def(id).name),
        crate::growth::Ware::Oil => s().shop_oil,
    }
}

impl App {
    /// 가게의 줄: 이름 … 값(모자라면 흐리게), 그 뒤에 이미 산 것(흐리게 · "샀다" — `growth::shelf`). 닫는 줄은 없다 — 창을 닫으면 떠난다(0164).
    pub(super) fn shop_menu(&self) -> Menu {
        let s = s();
        let shelf = crate::growth::shelf(&self.progress.owned, &self.progress.growth);
        let have = self.world.embers;
        let rows: Vec<ui::Row> = shelf
            .iter()
            .map(|&(w, price, sold)| {
                if sold {
                    ui::Row::value(ware_name(w), s.shop_sold).dimmed(true)
                } else {
                    ui::Row::value(ware_name(w), fill(s.shop_item, &[&price.to_string()]))
                        .dimmed(have < price)
                }
            })
            .collect();
        Menu::new(s.hearth_shop, tabs::LEFT_W, rows)
    }

    /// 가게 탭 그림(0164): 왼쪽 진열의 줄(살 것 · 산 것), 오른쪽에 고른 것 — 도는 그림 · 이름 · 갈래 · 살 수 있나 · 하는 일(장신구는 세계 안의 말까지).
    pub(super) fn draw_shop(&mut self, f: &Frame) {
        let s = s();
        let scene = Scene::Shop;
        let m = self.shop_menu();
        let w = self.draw_tab_window(f, scene);
        let shelf = crate::growth::shelf(&self.progress.owned, &self.progress.growth);
        let have = self.world.embers;
        self.draw_left_list(&w, &m.rows, 0.0);
        let (ware, price, sold) = shelf[self.cursor.min(shelf.len() - 1)];
        let (kind, desc, flavor, item) = match ware {
            crate::growth::Ware::Charm(id) => {
                let d = crate::charms::def(id);
                (
                    s.codex_tab_charms,
                    tr(d.desc),
                    Some(tr(d.flavor)),
                    Some(crate::item_view::Item::Charm(id)),
                )
            }
            crate::growth::Ware::Oil => (s.stat_flask, s.shop_oil_d, None, None),
        };
        // 상태 한 줄: 산 것은 금빛("샀다" — 내 것), 살 수 있으면 민트, 모자라면 흐림.
        let state = if sold {
            (s.shop_sold.to_string(), AMBER)
        } else if have >= price {
            (s.shop_can.to_string(), ui::MINT)
        } else {
            (s.shop_poor.to_string(), DIM)
        };
        let icon = self.draw_pane_head(
            &w,
            &PaneHead {
                name: ware_name(ware),
                gold: sold,
                kind: fill(s.shop_kind, &[kind, &price.to_string()]),
                state,
                icon: item.is_some(),
            },
        );
        if let Some(item) = item {
            self.draw_pane_item(item, icon);
        }
        let mut lines = self.pane_wrap(&w, desc, DIM);
        if let Some(fl) = flavor {
            lines.push(Ln::Gap);
            lines.extend(self.pane_wrap(&w, fl, FLAVOR));
        }
        self.draw_pane(&w, HEAD_H, &lines);
        let right = w.panel.x + w.panel.w;
        self.tab_foot(
            f,
            &w,
            scene,
            right,
            &[(Act::Choose, s.verb_choose), (Act::Confirm, s.ask_buy_yes)],
        );
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
            // 진열의 뒤쪽(이미 산 것): 까닭은 오른쪽 상태 줄("샀다")이 말하고 있다.
            None => self.snd(crate::sfx::Id::Deny),
        }
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
