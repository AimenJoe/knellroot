//! 장비 창(0067 안 D — 디아블로 콘솔식 인형): 가운데 방랑자 인형(든 무기를 든 3D, `Renderer::draw_doll`), 왼쪽 무기 칸,
//! 오른쪽 장신구 칸 셋. 칸을 먼저 고르고(위아래, 좌우는 바로 돌려 끼우기 — 옛 창 그대로) 확인하면 그 칸의 후보 목록
//! (`Scene::EquipPick`, 스택 위 칸 — 닫으면 그 칸으로)에서 고른다. 오른쪽 칸은 상태(`stats::lines`), 후보에 커서가 있으면
//! 바뀌는 줄만 `값 → 값 ▲▼`(색 + 화살표 + 숫자). 아래 왼쪽은 고른 것의 설명.
use super::menu::Hit;
use super::*;
use crate::charms::{self, Id as Charm};
use crate::stats::{Change, Line, Stats};

/// 칸: 0 = 무기, 1..=3 = 장신구 칸.
pub(super) const SLOTS: usize = 1 + charms::SLOTS;

/// 칸의 후보.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Cand {
    Weapon(weapons::Id),
    /// 장신구 칸 비우기.
    Empty,
    Charm(Charm),
}

/// 칸의 후보 목록(표 순서): 무기 칸 = 무기 넷, 장신구 칸 = 빈 칸 + 장신구 여섯(못 얻은 것도 — 잠긴 줄, KB "다 보이는 칸").
pub(super) fn candidates(slot: usize) -> Vec<Cand> {
    if slot == 0 {
        weapons::IDS.into_iter().map(Cand::Weapon).collect()
    } else {
        std::iter::once(Cand::Empty)
            .chain(charms::TABLE.iter().map(|d| Cand::Charm(d.id)))
            .collect()
    }
}

/// 장신구 칸 `i`에 `pick`을 끼운 뒤의 세 칸: 다른 칸에 끼어 있던 것이면 그 칸과 맞바꾼다(같은 것 두 칸 금지).
pub(super) fn wear(
    worn: [Option<Charm>; charms::SLOTS],
    i: usize,
    pick: Option<Charm>,
) -> [Option<Charm>; charms::SLOTS] {
    let mut out = worn;
    if let Some(j) = (0..charms::SLOTS).find(|&j| j != i && pick.is_some() && worn[j] == pick) {
        out[j] = worn[i];
    }
    out[i] = pick;
    out
}

/// 장신구를 얻는 곳(잠긴 줄의 설명).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Source {
    Chest,
    Boss,
    Shop,
}

pub(super) fn source(id: Charm, def: &StageDef) -> Option<Source> {
    if def.chest_charms.contains(&id) {
        Some(Source::Chest)
    } else if def.boss_charm == Some(id) {
        Some(Source::Boss)
    } else if crate::growth::SHOP
        .iter()
        .any(|(w, _)| *w == crate::growth::Ware::Charm(id))
    {
        Some(Source::Shop)
    } else {
        None
    }
}

/// 상태 줄의 이름과 값 글.
fn stat_name(l: Line) -> &'static str {
    let s = s();
    match l {
        Line::MaxHp => s.stat_max_hp,
        Line::Flask => s.stat_flask,
        Line::Heal => s.stat_heal,
        Line::Dmg => s.stat_dmg,
        Line::Taken => s.stat_taken,
        Line::AtkSpeed => s.stat_atk_speed,
        Line::DodgeCost => s.stat_dodge,
        Line::Parry => s.stat_parry,
        Line::KillStamina => s.stat_kill,
        Line::Ember => s.stat_ember,
    }
}

fn stat_value(l: Line, v: f32) -> String {
    let s = s();
    let int = || (v.round() as i32).to_string();
    let pct = || format!("{}%", (v * 100.0).round() as i32);
    match l {
        Line::MaxHp => int(),
        Line::Flask => fill(s.stat_sips, &[&int()]),
        Line::Heal | Line::Dmg | Line::Taken | Line::KillStamina => format!("+{}", int()),
        Line::AtkSpeed | Line::DodgeCost | Line::Ember => pct(),
        // 0.06 → "0.06초", 0 → "0초"(뒤 0은 뺀다 — 미리보기 줄이 짧게).
        Line::Parry => {
            let n = format!("{v:.2}");
            let n = n.trim_end_matches('0').trim_end_matches('.');
            format!("+{}", fill(s.value_secs, &[n]))
        }
    }
}

fn weapon_desc(id: weapons::Id) -> &'static str {
    let s = s();
    match id {
        weapons::Id::Sword => s.weapon_sword_d,
        weapons::Id::Hammer => s.weapon_hammer_d,
        weapons::Id::Staff => s.weapon_staff_d,
        weapons::Id::Crossbow => s.weapon_crossbow_d,
    }
}

/// 나빠지는 값의 색(붉은 계열 — 화살표 ▼와 함께, 색만으로 말하지 않는다).
const WORSE: Color = Color::new(0.88, 0.42, 0.36, 1.0);

impl App {
    /// 칸의 지금 것.
    fn slot_now(&self, slot: usize) -> Cand {
        match slot {
            0 => Cand::Weapon(self.weapon),
            i => self.progress.worn[i - 1].map_or(Cand::Empty, Cand::Charm),
        }
    }

    fn cand_name(&self, c: Cand) -> &'static str {
        match c {
            Cand::Weapon(w) => weapon_name(w),
            Cand::Empty => s().charm_empty,
            Cand::Charm(id) => tr(charms::def(id).name),
        }
    }

    /// 고른 것의 설명(잠긴 장신구는 얻는 곳).
    fn cand_desc(&self, c: Cand) -> &'static str {
        let s = s();
        match c {
            Cand::Weapon(w) => weapon_desc(w),
            Cand::Empty => s.equip_empty_d,
            Cand::Charm(id) if !self.progress.owned.contains(&id) => {
                match source(id, self.run.def) {
                    Some(Source::Chest) => s.charm_from_chest,
                    Some(Source::Boss) => s.charm_from_boss,
                    Some(Source::Shop) => s.charm_from_shop,
                    None => s.charm_locked,
                }
            }
            Cand::Charm(id) => tr(charms::def(id).desc),
        }
    }

    /// 칸 `slot`에 `c`를 끼우면 능력치(무기는 능력치를 바꾸지 않는다).
    fn stats_with(&self, slot: usize, c: Cand) -> Stats {
        let mut p = self.progress.clone();
        if slot > 0 {
            let pick = match c {
                Cand::Charm(id) => Some(id),
                _ => None,
            };
            p.worn = wear(p.worn, slot - 1, pick);
        }
        p.stats()
    }

    fn set_weapon(&mut self, w: weapons::Id) {
        self.weapon = w;
        self.world.set_weapon(w);
        self.progress.weapon = Some(w);
        self.persist();
    }

    /// 장비 창 한 틱: 위아래 = 칸, 좌우 = 바로 돌려 끼우기, 확인·탭 = 후보 목록, 뒤로·✕·창 밖 = 닫기.
    pub(super) fn tick_equip(&mut self, a: &Actions) {
        let tap = a.tap.and_then(|p| self.hit_at(p));
        if a.back || a.pause || tap == Some(Hit::Close) {
            self.go(Go::Back);
            return;
        }
        ui::menu_input(&mut self.cursor, SLOTS, a.menu_up, a.menu_down);
        let step = a.menu_right as i32 - a.menu_left as i32;
        if step != 0 {
            match self.cursor {
                0 => self.next_weapon(step),
                i => self.step_charm(i - 1, step),
            }
        }
        let pick = match tap {
            Some(Hit::Row(i)) => {
                self.cursor = i;
                true
            }
            _ => a.confirm,
        };
        if pick {
            let slot = self.cursor;
            let now = self.slot_now(slot);
            self.go(Go::Open(Scene::EquipPick { slot: slot as u8 }));
            self.cursor = candidates(slot).iter().position(|&c| c == now).unwrap_or(0);
        }
    }

    /// 후보 목록 한 틱: 위아래·굴리기, 확인·탭 = 끼우고 칸으로(못 얻은 것은 "안 됨"), 뒤로 = 그냥 칸으로.
    pub(super) fn tick_equip_pick(&mut self, a: &Actions, slot: usize) {
        let tap = a.tap.and_then(|p| self.hit_at(p));
        if a.back || a.pause || tap == Some(Hit::Close) {
            self.go(Go::Back);
            return;
        }
        let list = candidates(slot);
        let Some(i) = self.menu_pick(a, list.len()) else {
            return;
        };
        match list[i] {
            Cand::Weapon(w) => self.set_weapon(w),
            Cand::Charm(id) if !self.progress.owned.contains(&id) => {
                self.snd(crate::sfx::Id::Deny);
                return;
            }
            c => {
                let pick = match c {
                    Cand::Charm(id) => Some(id),
                    _ => None,
                };
                self.progress.worn = wear(self.progress.worn, slot - 1, pick);
                self.world.apply_stats(self.progress.stats(), false);
                self.persist();
            }
        }
        self.go(Go::Back);
    }

    /// 장비 창 그림: `pick`이 있으면 그 칸의 후보 목록(커서 = 지금 커서)을 오른쪽에, 칸 커서는 `slot_cursor`.
    pub(super) fn draw_equip(&mut self, f: &Frame, slot_cursor: usize, pick: Option<usize>) {
        let s = s();
        self.ui.dim(0.45);
        let u = self.ui.u(1.0);
        let px = self.ui.px(1.0);
        let panel = self.panel_rect(f, 440.0, 226.0);
        let top = self.ui.panel(panel, Some(s.equip_title));
        let x = self.ui.close(panel);
        self.hits.push((x, Hit::Close));
        let bottom = panel.y + panel.h - u * 8.0;
        // 왼쪽(인형·칸)과 오른쪽(상태·후보) 사이 세로 선.
        let split = panel.x + panel.w * 0.58;
        draw_line(split, top, split, bottom, u.max(1.0), ui::BORDER);
        let (lx, lw) = (panel.x + u * 8.0, split - panel.x - u * 16.0);
        let (rx, rw) = (split + u * 10.0, panel.x + panel.w - split - u * 18.0);

        // 칸 상자: 무기는 인형 왼쪽, 장신구 셋은 오른쪽.
        // 무기 칸은 이름이 길어(영어 "Pilgrim's Sword") 조금 넓게.
        let (bw, ww, bh, gap) = (u * 66.0, u * 86.0, u * 30.0, u * 6.0);
        let boxes_h = bh * 3.0 + gap * 2.0;
        let mut boxes = [Rect::default(); SLOTS];
        boxes[0] = Rect::new(lx, top + (boxes_h - bh) * 0.5, ww, bh);
        for (i, b) in boxes.iter_mut().enumerate().skip(1) {
            *b = Rect::new(lx + lw - bw, top + (bh + gap) * (i - 1) as f32, bw, bh);
        }
        // 인형: 두 칸 사이 가운데, 정수 배율(게임 화면과 같은 도트 굵기).
        let room_w = lw - bw - ww - u * 8.0;
        let m = ((boxes_h + u * 10.0) / crate::render::DOLL_H as f32)
            .min(room_w / crate::render::DOLL_W as f32)
            .floor()
            .max(1.0);
        let (dw, dh) = (
            crate::render::DOLL_W as f32 * m,
            crate::render::DOLL_H as f32 * m,
        );
        let doll = Rect::new(
            lx + ww + u * 4.0 + (room_w - dw) * 0.5,
            top - u * 4.0,
            dw,
            dh,
        );
        let t = get_time() as f32;
        self.renderer
            .draw_doll(self.weapon, doll, 0.55 * (t * 0.7).sin(), t);
        for (i, r) in boxes.iter().enumerate() {
            let sel = i == slot_cursor;
            self.ui.button(*r, "", sel);
            let cap = if i == 0 {
                s.menu_weapon.to_string()
            } else {
                fill(s.menu_charm_slot, &[&i.to_string()])
            };
            self.ui.text(&cap, r.x + u * 5.0, r.y + u * 3.0, 0.75, DIM);
            let now = self.slot_now(i);
            let name = self.ui.fit(self.cand_name(now), r.w - u * 10.0, 0.8);
            let col = match (now, sel) {
                (Cand::Empty, _) => DIM,
                (_, true) => AMBER,
                _ => INK,
            };
            self.ui
                .text(&name, r.x + u * 5.0, r.y + u * 3.0 + px * 0.85, 0.8, col);
            if pick.is_none() {
                self.hits.push((*r, Hit::Row(i)));
            }
        }

        // 아래 왼쪽: 고른 것의 설명(후보 목록이면 그 후보).
        let list = pick.map(candidates);
        let focus = match (&list, pick) {
            (Some(l), Some(_)) => l.get(self.cursor).copied().unwrap_or(Cand::Empty),
            _ => self.slot_now(slot_cursor),
        };
        let desc_y = (doll.y + doll.h).max(top + boxes_h) + u * 6.0;
        for (k, line) in ui::wrap(&self.ui, self.cand_desc(focus), lw, 1.0)
            .iter()
            .take(3)
            .enumerate()
        {
            let y = desc_y + px * 1.4 * k as f32;
            if y + px <= bottom {
                self.ui.text(line, lx, y, 1.0, DIM);
            }
        }

        // 오른쪽: 상태(칸 고르기) 또는 후보 목록 + 바뀌는 줄.
        let now_stats = self.progress.stats();
        let row_h = px * 1.3;
        let line_row = |this: &Self, y: f32, l: Line, v: f32, then: Option<(f32, Change)>| {
            let mut val = stat_value(l, v);
            let mut col = INK;
            if let Some((t2, ch)) = then {
                let (mark, c) = match ch {
                    Change::Better => ("▲", AMBER),
                    Change::Worse => ("▼", WORSE),
                    Change::Same => ("", INK),
                };
                val = format!("{val} → {} {mark}", stat_value(l, t2));
                col = c;
            }
            // 값 폭을 먼저 재고 이름은 남는 폭에(긴 값 "+0.00s → +0.06s ▲"와 겹치지 않게).
            let w = this.ui.width(&val, 0.9);
            this.ui.text(&val, rx + rw - w, y, 0.9, col);
            let name = this.ui.fit(stat_name(l), rw - w - this.ui.u(6.0), 0.9);
            this.ui.text(&name, rx, y, 0.9, DIM);
        };
        match (pick, list) {
            (Some(slot), Some(list)) => {
                let head = if slot == 0 {
                    s.menu_weapon.to_string()
                } else {
                    fill(s.menu_charm_slot, &[&slot.to_string()])
                };
                self.ui.text(&head, rx, top, 1.0, AMBER);
                let rows: Vec<ui::Row> = list
                    .iter()
                    .map(|&c| {
                        let mark = match c {
                            Cand::Weapon(w) if w == self.weapon => s.equip_held.to_string(),
                            Cand::Charm(id) if !self.progress.owned.contains(&id) => {
                                s.charm_locked.to_string()
                            }
                            Cand::Charm(id) => self
                                .progress
                                .worn
                                .iter()
                                .position(|w| *w == Some(id))
                                .map(|j| fill(s.menu_charm_slot, &[&(j + 1).to_string()]))
                                .unwrap_or_default(),
                            _ => String::new(),
                        };
                        let locked =
                            matches!(c, Cand::Charm(id) if !self.progress.owned.contains(&id));
                        ui::Row::value(self.cand_name(c), mark).dimmed(locked)
                    })
                    .collect();
                let list_top = top + px * 1.5;
                let list_h = (px * 1.75 * 5.0).min(bottom - list_top - px * 4.5);
                let area = Rect::new(rx - u * 4.0, list_top, rw + u * 8.0, list_h);
                let mut hits = self.ui.list(&rows, self.cursor, &mut self.view, area);
                hits.arrows = self.ui.title_arrows(
                    &self.view,
                    rows.len(),
                    Rect::new(rx, top - u * 6.0, rw + u * 18.0, px),
                );
                self.push_hits(&hits);
                // 바뀌는 줄만(무기는 능력치를 바꾸지 않는다 — 설명이 대신).
                let then =
                    self.stats_with(slot, list.get(self.cursor).copied().unwrap_or(Cand::Empty));
                let mut y = list_top + list_h + u * 6.0;
                let changed: Vec<_> = crate::stats::lines(&now_stats, Some(&then))
                    .into_iter()
                    .filter(|l| l.2.is_some_and(|(_, c)| c != Change::Same))
                    .collect();
                if changed.is_empty() && slot > 0 {
                    self.ui.text(s.equip_no_change, rx, y, 0.9, DIM);
                }
                for (l, v, then) in changed {
                    if y + px > bottom {
                        break;
                    }
                    line_row(self, y, l, v, then);
                    y += row_h;
                }
            }
            _ => {
                self.ui.text(s.equip_status, rx, top, 1.0, AMBER);
                let mut y = top + px * 1.6;
                for (l, v, _) in crate::stats::lines(&now_stats, None) {
                    if y + px > bottom {
                        break;
                    }
                    line_row(self, y, l, v, None);
                    y += row_h;
                }
            }
        }
        self.push_outside(panel);
        self.menu_hint(
            f.ox + f.w * 0.5,
            f.oy + f.h - self.ui.px(1.0) * 2.2,
            f.w * 0.94,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wearing_a_charm_worn_elsewhere_swaps_the_two_slots() {
        let worn = [Some(Charm::Shell), None, Some(Charm::Ear)];
        // 칸 2에 귀(칸 3에 있던 것) → 칸 3은 칸 2의 것(빈 칸)과 맞바꿈.
        assert_eq!(
            wear(worn, 1, Some(Charm::Ear)),
            [Some(Charm::Shell), Some(Charm::Ear), None]
        );
        // 칸 1에 귀 → 칸 3에 껍데기.
        assert_eq!(
            wear(worn, 0, Some(Charm::Ear)),
            [Some(Charm::Ear), None, Some(Charm::Shell)]
        );
        // 비우기는 그 칸만.
        assert_eq!(wear(worn, 2, None), [Some(Charm::Shell), None, None]);
        // 같은 것 두 칸은 생기지 않는다.
        for i in 0..3 {
            for pick in charms::TABLE.iter().map(|d| Some(d.id)) {
                let out = wear(worn, i, pick);
                let n = out.iter().filter(|w| **w == pick).count();
                assert_eq!(n, 1, "{pick:?} in slot {i}");
            }
        }
    }

    #[test]
    fn every_slot_lists_its_candidates_and_every_charm_has_a_source() {
        assert_eq!(candidates(0).len(), weapons::IDS.len());
        assert_eq!(candidates(1)[0], Cand::Empty);
        assert_eq!(candidates(3).len(), 1 + charms::TABLE.len());
        // 잠긴 줄의 설명이 비지 않게: 모든 장신구는 얻는 곳이 있다(상자·문지기·화로).
        for d in charms::TABLE.iter() {
            assert!(
                source(d.id, &crate::stage::sanctuary::DEF).is_some(),
                "{:?} has nowhere to be found",
                d.id
            );
        }
    }
}
