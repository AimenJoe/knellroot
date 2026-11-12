//! 쉼터 화로와 레벨(0104, v0.4.0 G1 — 0102 3·7절): 쉬면 화로가 열리고(레벨 · 나무 · 가게 · 기술 · 떠나기), 레벨은 생명·기력·힘을
//! 잔불로 한 단씩 올린다. 기술 줄(0158 G0)은 기술 판 `board.rs`로 — 가게 다음 줄이라 앞 줄들의 자리는 그대로다. 줄 표(0066) — 값은 `progression.rs`, 미리보기는 장비 창과 같은 상태 줄(`stats::lines`, 0067 D).
use super::equip::{stat_name, stat_value};
use super::menu::{Menu, Out};
use super::*;
use crate::progression::{Line as LvLine, LINES as LV_LINES, MAX_RANK};

/// 화로의 줄: 레벨 · 나무(0106) · 가게 · 기술(0158) · 떠나기.
const HEARTH_ROWS: usize = 5;

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
    /// 화로: 제목 줄 오른쪽에 잔불, 줄 = 레벨(지금 레벨) · 가게 · 떠나기, 아래에 고른 줄의 설명.
    pub(super) fn hearth_menu(&self) -> Menu {
        let s = s();
        let lv = self.progress.level.total().to_string();
        let points = self.progress.tree_points();
        let left = self.progress.tree.left(points).to_string();
        let skill_left = self
            .progress
            .board
            .left(self.progress.skill_points())
            .to_string();
        let rows = vec![
            ui::Row::value(s.hearth_level, lv),
            ui::Row::value(s.tree_title, fill(s.hearth_points, &[&left])),
            ui::Row::plain(s.hearth_shop),
            ui::Row::value(s.menu_skills, fill(s.hearth_skill_points, &[&skill_left])),
            ui::Row::plain(s.shop_leave),
        ];
        // 나무 줄: 목표 핀이 있으면 "목표까지 n점"(KB — 메뉴에서도 길 표시).
        let note = match self.cursor {
            0 => Some(s.hearth_level_d.to_string()),
            1 => Some(match self.progress.tree.cells_to_pin() {
                Some(n) => fill(s.tree_goal, &[&n.to_string()]),
                None => s.hearth_tree_d.to_string(),
            }),
            2 => Some(s.hearth_shop_d.to_string()),
            3 => Some(s.hearth_skills_d.to_string()),
            _ => None,
        };
        Menu {
            purse: Some(fill(s.shop_embers, &[&self.world.embers.to_string()])),
            note_room: true,
            note,
            ..Menu::new(s.shop_title, 300.0, rows)
        }
    }

    pub(super) fn hearth_act(&mut self, out: Out) {
        let Out::Pick(i) = out else { return };
        debug_assert!(i < HEARTH_ROWS);
        match i {
            0 => self.go(Go::Open(Scene::Level)),
            1 => self.go(Go::Open(Scene::Tree { view: false })),
            2 => self.go(Go::Open(Scene::Shop)),
            3 => self.go(Go::Open(Scene::SkillBoard)),
            _ => self.go(Go::Back),
        }
    }

    /// 레벨: 제목 "레벨 n", 줄 = 생명·기력·힘 … "단/5 · 잔불 값"(모자라면·다 올렸으면 흐리게),
    /// 아래에 고른 줄을 올리면 바뀌는 상태 줄 "지금 → 다음 ▲"(장비 창과 같은 줄·같은 값 글).
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
        let note = LV_LINES.get(self.cursor).map(|&l| self.level_preview(l));
        Menu {
            purse: Some(fill(s.shop_embers, &[&have.to_string()])),
            note_room: true,
            note,
            ..Menu::new(
                &fill(s.level_title, &[&lv.total().to_string()]),
                300.0,
                rows,
            )
        }
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

    pub(super) fn level_act(&mut self, out: Out) {
        let Out::Pick(i) = out else { return };
        let Some(&l) = LV_LINES.get(i) else { return };
        match self.progress.level.next_cost(l) {
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
