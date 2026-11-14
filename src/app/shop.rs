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

/// 가게 탭의 한 줄(0180): 진열(살 것·산 것 — `growth::shelf`의 자리) 뒤에 벼리기(가진 무기마다) · 거두기(벼린 무기마다).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum ShopRow {
    Ware(usize),
    Forge(weapons::Id),
    Recall(weapons::Id),
}

/// 가게의 줄 차례(순수): 진열 `shelf`개 → 가진 무기 가운데 벼릴 수 있는 것 → 그 가운데 벼린 것.
/// 진열을 맨 앞에 두는 까닭: 앞의 줄(살 것)은 자리 = `wares`의 자리라 고름이 그대로 산다(`Ask::Buy`).
pub(super) fn shop_rows(
    shelf: usize,
    arms: &[weapons::Id],
    forged: &crate::forge::Forged,
) -> Vec<ShopRow> {
    let forge: Vec<weapons::Id> = arms
        .iter()
        .copied()
        .filter(|&w| crate::forge::forgeable(w))
        .collect();
    (0..shelf)
        .map(ShopRow::Ware)
        .chain(forge.iter().map(|&w| ShopRow::Forge(w)))
        .chain(
            forge
                .iter()
                .filter(|&&w| forged.rank(w) > 0)
                .map(|&w| ShopRow::Recall(w)),
        )
        .collect()
}

/// `weapons::IDS`의 자리(묻는 창에 싣는다).
fn weapon_index(w: weapons::Id) -> u8 {
    weapons::IDS.iter().position(|&i| i == w).unwrap_or(0) as u8
}

impl App {
    fn shop_list(&self) -> Vec<ShopRow> {
        let shelf = crate::growth::shelf(&self.progress.owned, &self.progress.growth);
        shop_rows(shelf.len(), &self.arms(), &self.progress.forged)
    }

    /// 벼릴 수 있는 무기가 하나라도 있나(가게 탭의 민트 점·쉼터의 빛 — 0180).
    pub(super) fn can_forge(&self) -> bool {
        self.arms().into_iter().any(|w| {
            crate::forge::can_raise(
                &self.progress.finds,
                &self.progress.forged,
                self.world.embers,
                w,
            )
        })
    }

    /// 가게의 줄: 이름 … 값(모자라면 흐리게), 그 뒤에 이미 산 것(흐리게 · "샀다" — `growth::shelf`), 벼리기(무기 +n → 값)·거두기.
    /// 닫는 줄은 없다 — 창을 닫으면 떠난다(0164).
    pub(super) fn shop_menu(&self) -> Menu {
        let s = s();
        let shelf = crate::growth::shelf(&self.progress.owned, &self.progress.growth);
        let have = self.world.embers;
        let (finds, forged) = (&self.progress.finds, &self.progress.forged);
        let rows: Vec<ui::Row> = self
            .shop_list()
            .into_iter()
            .map(|r| match r {
                ShopRow::Ware(i) => {
                    let (w, price, sold) = shelf[i];
                    if sold {
                        ui::Row::value(ware_name(w), s.shop_sold).dimmed(true)
                    } else {
                        ui::Row::value(ware_name(w), fill(s.shop_item, &[&price.to_string()]))
                            .dimmed(have < price)
                    }
                }
                ShopRow::Forge(w) => match crate::forge::next_cost(forged, w) {
                    Some((st, em)) => ui::Row::value(
                        fill(s.forge_row, &[&forged_label(w, forged.rank(w) + 1)]),
                        fill(s.forge_cost, &[&st.to_string(), &em.to_string()]),
                    )
                    .dimmed(!crate::forge::can_raise(finds, forged, have, w)),
                    None => ui::Row::value(self.forged_name(w), s.forge_full).dimmed(true),
                },
                ShopRow::Recall(w) => ui::Row::value(
                    fill(s.forge_recall_row, &[&self.forged_name(w)]),
                    fill(
                        s.forge_back,
                        &[&crate::forge::cost_to(forged.rank(w)).0.to_string()],
                    ),
                ),
            })
            .collect();
        Menu::new(s.hearth_shop, tabs::LEFT_W, rows)
    }

    /// 가게 탭 그림(0164): 왼쪽 진열의 줄(살 것 · 산 것 · 벼리기 · 거두기), 오른쪽에 고른 것 — 도는 그림 · 이름 · 갈래 · 할 수 있나 ·
    /// 하는 일(장신구는 세계 안의 말까지, 벼리기는 단이 바꾸는 것과 가진 숫돌).
    pub(super) fn draw_shop(&mut self, f: &Frame) {
        let scene = Scene::Shop;
        let m = self.shop_menu();
        let w = self.draw_tab_window(f, scene);
        self.draw_left_list(&w, &m.rows, 0.0);
        let list = self.shop_list();
        let row = list[self.cursor.min(list.len() - 1)];
        let lines = match row {
            ShopRow::Ware(i) => self.ware_pane(&w, i),
            ShopRow::Forge(wp) | ShopRow::Recall(wp) => {
                self.forge_pane(&w, wp, matches!(row, ShopRow::Recall(_)))
            }
        };
        self.draw_pane(&w, HEAD_H, &lines);
        let s = s();
        let yes = match row {
            ShopRow::Ware(_) => s.ask_buy_yes,
            ShopRow::Forge(_) => s.forge_do,
            ShopRow::Recall(_) => s.forge_recall,
        };
        let right = w.panel.x + w.panel.w;
        self.tab_foot(
            f,
            &w,
            scene,
            right,
            &[(Act::Choose, s.verb_choose), (Act::Confirm, yes)],
        );
    }

    /// 진열의 물건: 머리(그림 · 이름 · 갈래와 값 · 살 수 있나) + 하는 일.
    fn ware_pane(&mut self, w: &super::frame::Window, i: usize) -> Vec<Ln> {
        let s = s();
        let shelf = crate::growth::shelf(&self.progress.owned, &self.progress.growth);
        let have = self.world.embers;
        let (ware, price, sold) = shelf[i];
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
            w,
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
        let mut lines = self.pane_wrap(w, desc, DIM);
        if let Some(fl) = flavor {
            lines.push(Ln::Gap);
            lines.extend(self.pane_wrap(w, fl, FLAVOR));
        }
        lines
    }

    /// 벼리기·거두기(0180): 머리(무기 그림 · 다음 단의 이름 · 값 · 할 수 있나) + 단마다 하는 일(지금 단까지 금빛) + 숫돌 셈 + 말.
    fn forge_pane(&mut self, w: &super::frame::Window, wp: weapons::Id, recall: bool) -> Vec<Ln> {
        let s = s();
        let forged = self.progress.forged;
        let finds = self.progress.finds.clone();
        let finds = &finds;
        let rank = forged.rank(wp);
        let next = crate::forge::next_cost(&forged, wp);
        // 머리의 이름은 무기 이름만 — 단은 갈래 줄에(긴 이름 + "+n"은 머리 칸을 넘는다).
        let name = weapon_name(wp);
        let (kind, state) = if recall {
            let (st, _) = crate::forge::cost_to(rank);
            (
                fill(s.forge_kind_recall, &[&st.to_string(), &rank.to_string()]),
                (s.forge_recall_note.to_string(), DIM),
            )
        } else {
            match next {
                Some((st, em)) => {
                    let state = if crate::forge::free(finds, &forged) < st {
                        (s.forge_no_stone.to_string(), DIM)
                    } else if self.world.embers < em {
                        (s.shop_poor.to_string(), DIM)
                    } else {
                        (s.forge_can.to_string(), ui::MINT)
                    };
                    (
                        fill(
                            s.forge_kind,
                            &[&st.to_string(), &em.to_string(), &(rank + 1).to_string()],
                        ),
                        state,
                    )
                }
                None => (
                    s.forge_kind_full.to_string(),
                    (s.forge_full.to_string(), AMBER),
                ),
            }
        };
        let icon = self.draw_pane_head(
            w,
            &PaneHead {
                name,
                gold: next.is_none() && !recall,
                kind,
                state,
                icon: true,
            },
        );
        self.draw_pane_item(crate::item_view::Item::Weapon(wp), icon);
        let mut lines = Vec::new();
        if recall {
            lines.extend(self.pane_wrap(w, s.forge_recall_d, INK));
            lines.push(Ln::Gap);
        }
        if let Some(table) = wp.def().forge {
            for (i, f) in table.iter().enumerate() {
                let r = i as u8 + 1;
                let col = if r <= rank { AMBER } else { INK };
                lines.push(Ln::Pair(
                    fill(s.forge_rank, &[&r.to_string()]),
                    forge_effect(f),
                    col,
                ));
            }
        }
        lines.push(Ln::Gap);
        lines.push(Ln::Pair(
            s.forge_stones.to_string(),
            fill(
                s.forge_stones_v,
                &[
                    &crate::forge::free(finds, &forged).to_string(),
                    &crate::forge::stones(finds).to_string(),
                ],
            ),
            INK,
        ));
        lines.push(Ln::Gap);
        lines.extend(self.pane_wrap(w, s.forge_flavor, FLAVOR));
        lines
    }

    pub(super) fn shop_act(&mut self, out: Out) {
        let Out::Pick(i) = out else { return };
        let s = s();
        let list = self.shop_list();
        match list.get(i).copied() {
            Some(ShopRow::Ware(i)) => {
                let wares = crate::growth::wares(&self.progress.owned, &self.progress.growth);
                match wares.get(i).copied() {
                    // 모자라면 묻지 않고 알림만(nail 가게와 같다 — 흐린 줄).
                    Some((_, price)) if self.world.embers < price => {
                        self.toast = Some((s.shop_poor.to_string(), 1.6));
                        self.snd(crate::sfx::Id::Deny);
                    }
                    Some(_) => self.ask(flow::Ask::Buy(i)),
                    // 진열의 뒤쪽(이미 산 것): 까닭은 오른쪽 상태 줄("샀다")이 말하고 있다.
                    None => self.snd(crate::sfx::Id::Deny),
                }
            }
            Some(ShopRow::Forge(w)) => {
                let (finds, forged) = (&self.progress.finds, &self.progress.forged);
                match crate::forge::next_cost(forged, w) {
                    None => self.snd(crate::sfx::Id::Deny),
                    Some((st, _)) if crate::forge::free(finds, forged) < st => {
                        self.toast = Some((s.forge_no_stone.to_string(), 1.6));
                        self.snd(crate::sfx::Id::Deny);
                    }
                    Some((_, em)) if self.world.embers < em => {
                        self.toast = Some((s.shop_poor.to_string(), 1.6));
                        self.snd(crate::sfx::Id::Deny);
                    }
                    Some(_) => self.ask(flow::Ask::Forge(weapon_index(w))),
                }
            }
            Some(ShopRow::Recall(w)) => self.ask(flow::Ask::Recall(weapon_index(w))),
            None => {}
        }
    }

    /// 한 단 벼린다(0180 — 묻는 창의 "벼리기"): 숫돌·잔불을 쓰고, 든 무기면 능력치를 바로.
    pub(super) fn forge_weapon(&mut self, w: weapons::Id) {
        let (finds, forged) = (&self.progress.finds, &self.progress.forged);
        if !crate::forge::can_raise(finds, forged, self.world.embers, w) {
            self.snd(crate::sfx::Id::Deny);
            return;
        }
        let (_, em) = crate::forge::next_cost(forged, w).unwrap_or((0, 0));
        let r = forged.rank(w) + 1;
        self.world.embers -= em;
        self.progress.embers = self.world.embers;
        self.progress.forged.set(w, r);
        self.world.apply_stats(self.progress.stats(), false);
        self.snd(crate::sfx::Id::Buy);
        self.toast = Some((fill(s().toast_forged, &[&forged_label(w, r)]), TOAST_TIME));
        self.persist();
    }

    /// 단을 모두 거둔다(0180): 숫돌은 돌아오고(벼린 단에서 세니 단을 0으로 하면 된다) 잔불은 안 돌아온다.
    pub(super) fn recall_weapon(&mut self, w: weapons::Id) {
        let n = crate::forge::cost_to(self.progress.forged.rank(w)).0;
        self.progress.forged.set(w, 0);
        self.world.apply_stats(self.progress.stats(), false);
        self.snd(crate::sfx::Id::Recall);
        self.toast = Some((
            fill(s().toast_recalled, &[weapon_name(w), &n.to_string()]),
            TOAST_TIME,
        ));
        // 줄이 하나 줄었다 — 커서를 안에.
        self.cursor = self.cursor.min(self.shop_list().len().saturating_sub(1));
        self.persist();
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

/// 한 단이 하는 일(0180): 근접 = 타마다 피해, 쇠뇌 = 탄창·사격 피해.
fn forge_effect(f: &weapons::Forge) -> String {
    let s = s();
    let mut parts = Vec::new();
    if f.hit > 0 {
        parts.push(fill(s.forge_fx_hit, &[&f.hit.to_string()]));
    }
    if f.mag > 0 {
        parts.push(fill(s.forge_fx_mag, &[&f.mag.to_string()]));
    }
    if f.shot > 0 {
        parts.push(fill(s.forge_fx_shot, &[&f.shot.to_string()]));
    }
    parts.join(" · ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use weapons::Id;

    /// 0180: 가게 줄은 진열(자리 = `wares`의 자리) → 벼릴 무기(맨손 빼고) → 벼린 무기의 거두기.
    #[test]
    fn shop_rows_keep_wares_first_then_forging_then_recall() {
        let mut f = crate::forge::Forged::default();
        let arms = [Id::Fists, Id::Sword, Id::Crossbow];
        assert_eq!(
            shop_rows(2, &arms, &f),
            vec![
                ShopRow::Ware(0),
                ShopRow::Ware(1),
                ShopRow::Forge(Id::Sword),
                ShopRow::Forge(Id::Crossbow)
            ]
        );
        f.set(Id::Crossbow, 2);
        assert_eq!(
            shop_rows(0, &arms, &f).last(),
            Some(&ShopRow::Recall(Id::Crossbow))
        );
        assert!(!shop_rows(0, &arms, &f).contains(&ShopRow::Recall(Id::Sword)));
    }
}
