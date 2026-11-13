//! 큰 창의 상태 탭(0164 — 옛 화로 목록과 레벨 창 0104가 여기로): 레벨은 생명·기력·힘을 잔불로 한 단씩. 화로에서는 확인 = 올리기
//! (묻는 창), 멈춤에서는 읽기만. 값은 `progression.rs`, 미리보기는 장비 탭과 같은 상태 줄(`stats::lines`, 0067 D).
use super::equip::{stat_name, stat_value};
use super::menu::{Menu, Out};
use super::tabs::{Ln, PaneHead, HEAD_H};
use super::*;
use crate::progression::{Line as LvLine, LINES as LV_LINES, MAX_RANK};
use flow::Host;

/// 레벨 줄 이름.
fn line_name(l: LvLine) -> &'static str {
    let s = s();
    match l {
        LvLine::Life => s.lv_life,
        LvLine::Stamina => s.lv_stamina,
        LvLine::Might => s.lv_might,
    }
}

impl App {
    /// 레벨 줄: 생명·기력·힘 … "단/5 · 잔불 값"(모자라면·다 올렸으면 흐리게).
    pub(super) fn level_menu(&self) -> Menu {
        let s = s();
        let lv = self.progress.level;
        let have = self.world.embers;
        let max = MAX_RANK.to_string();
        let rows = LV_LINES
            .iter()
            .map(|&l| {
                let rank = lv.rank(l).to_string();
                match lv.next_cost(l) {
                    Some(c) => ui::Row::value(
                        line_name(l),
                        fill(s.lv_rank, &[&rank, &max, &c.to_string()]),
                    )
                    .dimmed(have < c),
                    None => {
                        ui::Row::value(line_name(l), fill(s.lv_full, &[&rank, &max])).dimmed(true)
                    }
                }
            })
            .collect();
        Menu::new(s.equip_status, tabs::LEFT_W, rows)
    }

    /// 상태 탭 그림: 왼쪽 레벨 줄, 오른쪽에 고른 줄을 올리면 바뀌는 것.
    pub(super) fn draw_status(&mut self, f: &Frame, host: Host) {
        let s = s();
        let scene = Scene::Status { host };
        let m = self.level_menu();
        let w = self.draw_tab_window(f, scene);
        self.draw_left_list(&w, &m.rows, 0.0);
        let lv = self.progress.level;
        if let Some(&l) = LV_LINES.get(self.cursor) {
            let state = match lv.next_cost(l) {
                None => (s.lv_full_d.to_string(), DIM),
                Some(_) if !host.edits() => (s.lv_rest_only.to_string(), DIM),
                Some(c) if self.world.embers < c => (s.shop_poor.to_string(), DIM),
                Some(c) => (fill(s.shop_embers, &[&c.to_string()]), ui::MINT),
            };
            self.draw_pane_head(
                &w,
                &PaneHead {
                    name: line_name(l),
                    gold: true,
                    kind: fill(s.level_title, &[&lv.total().to_string()]),
                    state,
                    icon: false,
                },
            );
            let lines: Vec<Ln> = self.pane_wrap(&w, &self.level_preview(l), DIM);
            self.draw_pane(&w, HEAD_H, &lines);
        }
        let right = w.panel.x + w.panel.w;
        let mut acts = vec![(Act::Choose, s.verb_choose)];
        if host.edits() {
            acts.push((Act::Confirm, s.ask_level_yes));
        }
        self.tab_foot(f, &w, scene, right, &acts);
    }

    /// 이 줄을 한 단 올리면 바뀌는 상태 줄 — "최대 기력 +12 → +24 ▲ · …". 다 올렸으면 그렇다고.
    fn level_preview(&self, l: LvLine) -> String {
        if self.progress.level.next_cost(l).is_none() {
            return s().lv_full_d.to_string();
        }
        let now = self.progress.stats();
        let mut then = self.progress.clone();
        then.level = then.level.raised(l);
        let then = then.stats();
        crate::stats::lines(&now, Some(&then))
            .into_iter()
            .filter_map(|(line, a, b)| {
                let (b, ch) = b?;
                let mark = match ch {
                    crate::stats::Change::Same => return None,
                    crate::stats::Change::Better => "▲",
                    crate::stats::Change::Worse => "▼",
                };
                Some(format!(
                    "{} {} → {} {mark}",
                    stat_name(line),
                    stat_value(line, a),
                    stat_value(line, b)
                ))
            })
            .collect::<Vec<_>>()
            .join(" · ")
    }

    pub(super) fn level_act(&mut self, host: Host, out: Out) {
        let Out::Pick(i) = out else { return };
        let Some(&l) = LV_LINES.get(i) else { return };
        match self.progress.level.next_cost(l) {
            // 올리는 것은 쉬는 동안에만(멈춤에서는 읽기) — 까닭은 오른쪽 칸의 상태 줄이 말하고 있다.
            Some(_) if !host.edits() => self.snd(crate::sfx::Id::Deny),
            None => {
                self.toast = Some((s().lv_full_d.to_string(), 1.6));
                self.snd(crate::sfx::Id::Deny);
            }
            // 모자라면 묻지 않고 알림만(가게와 같다 — 흐린 줄).
            Some(c) if self.world.embers < c => {
                self.toast = Some((s().shop_poor.to_string(), 1.6));
                self.snd(crate::sfx::Id::Deny);
            }
            Some(_) => self.ask(flow::Ask::Level(i as u8)),
        }
    }

    /// 묻는 창에서 "올리기"(0104): 잔불을 내고 한 단 — 막 쉬었으니 체력·스태미나 가득(가게 기름과 같다).
    /// 연출은 작게(lore CANON — 빛 터짐·뿌리 없이): 소리 하나 + 알림. 무기를 고쳐 쥐는 동작(`regrip`)은 모델 동작이 생기면.
    pub(super) fn raise_level(&mut self, i: usize) {
        let Some(&l) = LV_LINES.get(i) else { return };
        let Some(cost) = self.progress.level.next_cost(l) else {
            return;
        };
        if self.world.embers < cost {
            self.toast = Some((s().shop_poor.to_string(), 1.6));
            self.snd(crate::sfx::Id::Deny);
            return;
        }
        self.world.embers -= cost;
        self.progress.embers = self.world.embers;
        self.progress.level = self.progress.level.raised(l);
        self.world.apply_stats(self.progress.stats(), true);
        self.snd(crate::sfx::Id::Buy);
        let rank = self.progress.level.rank(l).to_string();
        self.toast = Some((fill(s().toast_level, &[line_name(l), &rank]), TOAST_TIME));
        self.persist();
    }

    /// 묻는 창의 글(0104): "기력 3단 — 잔불 65. 올릴까요?".
    pub(super) fn ask_level_text(&self, i: usize) -> String {
        let Some(&l) = LV_LINES.get(i) else {
            return String::new();
        };
        let next = (self.progress.level.rank(l) + 1).to_string();
        let cost = self.progress.level.next_cost(l).unwrap_or(0).to_string();
        fill(s().ask_level, &[line_name(l), &next, &cost])
    }
}
