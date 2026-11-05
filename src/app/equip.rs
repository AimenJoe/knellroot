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

/// 칸의 후보 목록(표 순서): 무기 칸 = 무기 넷, 장신구 칸 = 빈 칸 + **가진** 장신구만.
/// 못 얻은 것은 장비 창에 보이지 않는다 — 검은 실루엣 같은 "찾을 것" 표시는 도감 몫(사장 2026-11-05, 0068).
pub(super) fn candidates(slot: usize, owned: &[Charm]) -> Vec<Cand> {
    if slot == 0 {
        weapons::IDS.into_iter().map(Cand::Weapon).collect()
    } else {
        std::iter::once(Cand::Empty)
            .chain(
                charms::TABLE
                    .iter()
                    .filter(|d| owned.contains(&d.id))
                    .map(|d| Cand::Charm(d.id)),
            )
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

/// 칸 후보 → 그림으로 보일 아이템(빈 칸은 없음).
fn item_of(c: Cand) -> Option<crate::item_view::Item> {
    match c {
        Cand::Weapon(w) => Some(crate::item_view::Item::Weapon(w)),
        Cand::Charm(id) => Some(crate::item_view::Item::Charm(id)),
        Cand::Empty => None,
    }
}

/// 타깃 한 변 `native` px을 화면 `want` px 안에 넣는 정수 배율(도트가 고르게 — 1 아래면 1).
fn icon_scale(native: u32, want: f32) -> f32 {
    (want / native as f32).floor().max(1.0)
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

    /// 고른 것의 설명. 빈 칸은 — 아직 장신구가 하나도 없으면 얻는 곳(옛 장비 창의 안내 그대로).
    fn cand_desc(&self, c: Cand) -> &'static str {
        let s = s();
        match c {
            Cand::Weapon(w) => weapon_desc(w),
            Cand::Empty if self.progress.owned.is_empty() => s.equip_none_yet,
            Cand::Empty => s.equip_empty_d,
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
            self.cursor = candidates(slot, &self.progress.owned)
                .iter()
                .position(|&c| c == now)
                .unwrap_or(0);
        }
    }

    /// 후보 목록 한 틱: 위아래·굴리기, 확인·탭 = 끼우고 칸으로(못 얻은 것은 "안 됨"), 뒤로 = 그냥 칸으로.
    pub(super) fn tick_equip_pick(&mut self, a: &Actions, slot: usize) {
        let tap = a.tap.and_then(|p| self.hit_at(p));
        if a.back || a.pause || tap == Some(Hit::Close) {
            self.go(Go::Back);
            return;
        }
        let list = candidates(slot, &self.progress.owned);
        let Some(i) = self.menu_pick(a, list.len()) else {
            return;
        };
        match list[i] {
            Cand::Weapon(w) => self.set_weapon(w),
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
        let split = panel.x + panel.w * 0.55;
        draw_line(split, top, split, bottom, u.max(1.0), ui::BORDER);
        let (lx, lw) = (panel.x + u * 8.0, split - panel.x - u * 16.0);
        let (rx, rw) = (split + u * 10.0, panel.x + panel.w - split - u * 18.0);

        // 칸 상자(0068 — 디아블로 인형처럼 칸에는 아이템 그림, 이름은 아래 설명의 머리글): 무기는 인형 왼쪽, 장신구 셋은 오른쪽.
        let (ww, bw, gap) = (u * 40.0, u * 34.0, u * 6.0);
        let boxes_h = bw * 3.0 + gap * 2.0;
        let mut boxes = [Rect::default(); SLOTS];
        boxes[0] = Rect::new(lx + u * 6.0, top + (boxes_h - ww) * 0.5, ww, ww);
        for (i, b) in boxes.iter_mut().enumerate().skip(1) {
            *b = Rect::new(
                lx + lw - bw - u * 6.0,
                top + (bw + gap) * (i - 1) as f32,
                bw,
                bw,
            );
        }
        // 인형: 두 칸 사이 가운데, 정수 배율(게임 화면과 같은 도트 굵기).
        let room_x = boxes[0].x + ww + u * 4.0;
        let room_w = boxes[1].x - u * 4.0 - room_x;
        let m = ((boxes_h + u * 10.0) / crate::render::DOLL_H as f32)
            .min(room_w / crate::render::DOLL_W as f32)
            .floor()
            .max(1.0);
        let (dw, dh) = (
            crate::render::DOLL_W as f32 * m,
            crate::render::DOLL_H as f32 * m,
        );
        let doll = Rect::new(room_x + (room_w - dw) * 0.5, top - u * 4.0, dw, dh);
        let t = get_time() as f32;
        self.renderer
            .draw_doll(self.weapon, doll, 0.55 * (t * 0.7).sin(), t);
        for (i, r) in boxes.iter().enumerate() {
            let sel = i == slot_cursor;
            self.ui.button(*r, "", sel);
            if let Some(item) = item_of(self.slot_now(i)) {
                let k = icon_scale(crate::render::ICON_PX, r.w - u * 4.0);
                let side = crate::render::ICON_PX as f32 * k;
                self.renderer.draw_icon(
                    item,
                    Rect::new(
                        r.x + (r.w - side) * 0.5,
                        r.y + (r.h - side) * 0.5,
                        side,
                        side,
                    ),
                );
            }
            if pick.is_none() {
                self.hits.push((*r, Hit::Row(i)));
            }
        }

        // 아래 왼쪽: 고른 것(후보 목록이면 그 후보) — 도는 그림 + 이름(머리글) + 설명.
        let list = pick.map(|slot| candidates(slot, &self.progress.owned));
        let focus = match (&list, pick) {
            (Some(l), Some(_)) => l.get(self.cursor).copied().unwrap_or(Cand::Empty),
            _ => self.slot_now(slot_cursor),
        };
        let desc_y = (doll.y + doll.h).max(top + boxes_h) + u * 6.0;
        let room = (bottom - desc_y).max(px);
        let mut text_x = lx;
        if let Some(item) = item_of(focus) {
            let k = icon_scale(crate::render::ITEM_PX, room.min(u * 64.0));
            let side = crate::render::ITEM_PX as f32 * k;
            self.renderer.draw_item(
                item,
                Rect::new(lx, desc_y, side, side),
                0.7 * (t * 0.8).sin(),
            );
            text_x = lx + side + u * 8.0;
        }
        let text_w = lx + lw - text_x;
        let name = self.ui.fit(self.cand_name(focus), text_w, 1.0);
        self.ui.text(&name, text_x, desc_y, 1.0, AMBER);
        for (k, line) in ui::wrap(&self.ui, self.cand_desc(focus), text_w, 0.9)
            .iter()
            .take(3)
            .enumerate()
        {
            let y = desc_y + px * 1.5 + px * 1.3 * k as f32;
            if y + px <= bottom {
                self.ui.text(line, text_x, y, 0.9, DIM);
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
                            Cand::Charm(id) => self
                                .progress
                                .worn
                                .iter()
                                .position(|w| *w == Some(id))
                                .map(|j| fill(s.menu_charm_slot, &[&(j + 1).to_string()]))
                                .unwrap_or_default(),
                            _ => String::new(),
                        };
                        ui::Row::value(self.cand_name(c), mark)
                            .with_icon(true)
                            .tagged()
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
                // 줄마다 아이템 그림(빈 칸은 비워 둔다) — 아이콘은 한 번 찍어 둔 것(0068).
                for (r, c) in hits.rows.iter().zip(list.iter()) {
                    if let (true, Some(item)) = (r.w > 0.0, item_of(*c)) {
                        let slot_r = self.ui.list_icon(*r);
                        let k = icon_scale(crate::render::ICON_PX, slot_r.w);
                        let side = crate::render::ICON_PX as f32 * k;
                        self.renderer.draw_icon(
                            item,
                            Rect::new(
                                slot_r.x + (slot_r.w - side) * 0.5,
                                slot_r.y + (slot_r.h - side) * 0.5,
                                side,
                                side,
                            ),
                        );
                    }
                }
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
    fn slots_list_only_what_you_own() {
        assert_eq!(candidates(0, &[]).len(), weapons::IDS.len());
        // 장신구가 없으면 빈 칸 하나 — 못 얻은 것은 장비 창에 없다(도감 몫).
        assert_eq!(candidates(1, &[]), vec![Cand::Empty]);
        let owned = [Charm::Ear, Charm::Shell];
        assert_eq!(
            candidates(3, &owned),
            vec![
                Cand::Empty,
                Cand::Charm(Charm::Shell),
                Cand::Charm(Charm::Ear)
            ],
            "table order, owned only"
        );
    }
}
