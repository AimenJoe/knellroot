//! 묻는 창(0065, nail `draw_shop_ask` 꼴 0066): 아래 칸을 그대로 깔고 어둡게, 그 위에 작은 창 — 무엇을 잃는지 + 단추 둘
//! (하는 일 이름 / 취소). 처음 포커스는 `flow::focus`(버리는 일은 취소), 창 밖 탭·뒤로 = 취소.
use super::menu::Hit;
use super::*;

impl App {
    pub(super) fn tick_ask(&mut self, a: &Actions, k: flow::Ask) {
        let tap = a.tap.and_then(|p| self.hit_at(p));
        if a.back || a.pause || tap == Some(Hit::Close) {
            self.go(Go::Back);
            return;
        }
        // 단추가 가로라 좌우로도 고른다(위·아래도 그대로).
        ui::menu_input(&mut self.cursor, 2, a.menu_left, a.menu_right);
        let Some(i) = self.menu_pick(a, 2) else {
            return;
        };
        let yes = i == 0;
        // 슬롯 창의 물음(0192): 닫고 나서 한다 — 덮어쓰기는 판을 열고(장면을 바꾼다), 지우기는 슬롯 창에 남는다.
        match (k, yes) {
            (flow::Ask::Overwrite(slot), true) => {
                self.go(Go::Back);
                self.overwrite_slot(slot as usize);
                return;
            }
            (flow::Ask::Erase(slot), true) => {
                self.go(Go::Back);
                self.erase_slot(slot as usize);
                return;
            }
            _ => {}
        }
        let (next, rebuild) = flow::answer(k, yes);
        if yes {
            match k {
                flow::Ask::Buy(w) => {
                    let wares = crate::growth::wares(
                        &self.progress.owned,
                        &self.progress.growth,
                        &self.progress.shields,
                    );
                    if let Some(&w) = wares.get(w) {
                        self.buy(w);
                    }
                }
                flow::Ask::Level(l) => self.raise_level(l as usize),
                flow::Ask::Skill(i) => self.buy_skill(i as usize),
                flow::Ask::Respec => self.respec_skills(),
                flow::Ask::Forge(i) => self.forge_weapon(crate::weapons::IDS[i as usize]),
                flow::Ask::Recall(i) => self.recall_weapon(crate::weapons::IDS[i as usize]),
                flow::Ask::Salvage(id) => self.salvage_item(id),
                flow::Ask::Tidy => self.tidy_bag(),
                flow::Ask::Restart
                | flow::Ask::ToTitle
                | flow::Ask::Overwrite(_)
                | flow::Ask::Erase(_) => {}
            }
        }
        if rebuild {
            self.restart();
        }
        self.go(next);
        // 줄이 준 가게(벼린 단을 거둠): 돌아온 커서는 묻기 전의 자리다 — 줄 안으로.
        if yes && matches!(k, flow::Ask::Recall(_)) {
            self.cursor = self.cursor.min(self.shop_list().len().saturating_sub(1));
        }
    }

    pub(super) fn draw_ask(&mut self, f: &Frame, k: flow::Ask) {
        let s = s();
        // 아래 칸을 그 커서 그대로 깐다(무엇을 묻는지 보이게).
        if let Some(e) = self.stack.under().copied() {
            let (cursor, view) = (self.cursor, self.view);
            (self.cursor, self.view) = (e.cursor, e.view);
            self.draw_layered(f, e.scene);
            (self.cursor, self.view) = (cursor, view);
        }
        // 묻는 창은 누를 자리를 제 단추와 창 밖만 남긴다(nail).
        self.hits.clear();
        #[cfg(feature = "dev")]
        crate::ui::check::forget();
        self.ui.dim(0.55);
        let (q, yes) = match k {
            flow::Ask::Restart => (s.ask_restart.to_string(), s.menu_restart),
            flow::Ask::ToTitle => (s.ask_to_title.to_string(), s.menu_to_title),
            flow::Ask::Overwrite(i) => (
                fill(s.ask_overwrite, &[&self.slot_label(i as usize)]),
                s.slot_overwrite,
            ),
            flow::Ask::Erase(i) => (
                fill(s.ask_erase, &[&self.slot_label(i as usize)]),
                s.slot_erase,
            ),
            flow::Ask::Buy(i) => {
                let wares = crate::growth::wares(
                    &self.progress.owned,
                    &self.progress.growth,
                    &self.progress.shields,
                );
                let (name, price) = match wares.get(i) {
                    Some((crate::growth::Ware::Charm(id), p)) => {
                        (tr(crate::charms::def(*id).name), *p)
                    }
                    Some((crate::growth::Ware::Oil, p)) => (s.shop_oil, *p),
                    Some((crate::growth::Ware::Shield(id), p)) => (tr(id.def().name), *p),
                    None => ("", 0),
                };
                (fill(s.ask_buy, &[name, &price.to_string()]), s.ask_buy_yes)
            }
            flow::Ask::Level(l) => (self.ask_level_text(l as usize), s.ask_level_yes),
            flow::Ask::Skill(i) => self.ask_skill_text(i as usize),
            flow::Ask::Respec => (self.ask_respec_text(), s.board_respec),
            flow::Ask::Forge(i) => {
                let w = crate::weapons::IDS[i as usize];
                let (st, em) = crate::forge::next_cost(&self.progress.forged, w).unwrap_or((0, 0));
                let to = self.progress.forged.rank(w) + 1;
                (
                    fill(
                        s.ask_forge,
                        &[&forged_label(w, to), &st.to_string(), &em.to_string()],
                    ),
                    s.forge_do,
                )
            }
            flow::Ask::Salvage(id) => {
                let n = self.progress.bag.get(id).map_or(0, |it| it.salvage());
                (
                    fill(s.ask_salvage, &[&self.item_name(id), &n.to_string()]),
                    s.salvage_do,
                )
            }
            flow::Ask::Tidy => {
                let (n, embers) = self.tidy_count();
                (
                    fill(s.ask_tidy, &[&n.to_string(), &embers.to_string()]),
                    s.bag_tidy,
                )
            }
            flow::Ask::Recall(i) => {
                let w = crate::weapons::IDS[i as usize];
                let r = self.progress.forged.rank(w);
                let (st, em) = crate::forge::cost_to(r);
                (
                    fill(
                        s.ask_recall,
                        &[&forged_label(w, r), &st.to_string(), &em.to_string()],
                    ),
                    s.forge_recall,
                )
            }
        };
        let u = self.ui.u(1.0);
        let w = 280.0;
        let lines = ui::wrap(&self.ui, &q, (w - 24.0) * u, 1.0)
            .len()
            .clamp(1, 3);
        let btn_h = ui::GLYPH * 1.9;
        let h = 14.0 + lines as f32 * ui::GLYPH * 1.45 + 12.0 + btn_h + 12.0;
        let panel = self.panel_rect(f, w, h);
        self.ui.panel(panel, None);
        let cx = panel.x + panel.w * 0.5;
        self.ui
            .text_center_wrapped(&q, cx, panel.y + u * 14.0, 1.0, INK, panel.w - u * 24.0);
        let area = Rect::new(
            panel.x + u * 16.0,
            panel.y + panel.h - u * (12.0 + btn_h),
            panel.w - u * 32.0,
            u * btn_h,
        );
        let hits = self.ui.buttons(&[yes, s.ask_cancel], self.cursor, area);
        self.push_hits(&hits);
        self.push_outside(panel);
        self.menu_hint(
            f.ox + f.w * 0.5,
            f.oy + f.h - self.ui.px(1.0) * 2.2,
            f.w * 0.94,
        );
    }
}
