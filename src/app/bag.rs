//! 큰 창의 가방 탭(0226 — 사장 2026-11-17 "인벤토리도 제한이 있어야겠지"): 가진 무기·방패·장신구 아이템 전부(낀 것 포함).
//! 왼쪽 = 칸 6×5(빈 칸이 보인다 — 한도는 숫자보다 빈 칸으로 읽힌다) + 아래 "가방 n/30" · 오른쪽 = 고른 것의 상세(장비 탭과 같은 셈 —
//! 그 칸에 지금 낀 것과 견준다) · 확인 = 칸 곁의 작은 목록(끼기 · 지키기 · 내려놓기 · 잔불로).
//! - 차례는 저절로(무기 → 방패 → 장신구, 바탕, 값진 것 먼저) — 손으로 옮기지 않는다.
//! - 가득이면 떨어진 것을 **줍지 못한다**(바닥에 남는다) — 자리를 만드는 길은 둘: 여벌을 내려놓기(어디서나 — 발치에 놓이고
//!   기록에도 남는다) · 잔불로 바꾸기(화로에서만, 하나씩 또는 "여벌 정리"로 한꺼번에).
//! - 내려놓거나 거둘 수 있는 것은 여벌뿐(낀 것·그 바탕의 마지막 하나는 아니다), 지키는 것은 빼고.
use super::equip::{slot_name, Cand, SUB_SLOT};
use super::frame::Window;
use super::tabs::{Ln, PaneHead, HEAD_H, K_KIND, LEFT_W};
use super::*;
use crate::items::{Bag, Base, Item, BAG_CAP};
use flow::Host;

/// 칸의 격자(기준 px): 한 줄 여섯, 칸의 한 변과 틈. 가방이 한도를 넘으면(문지기가 처음 주는 장신구 — `BAG_CAP`의 풀이)
/// 줄이 하나 늘고 칸이 작아진다.
pub(super) const COLS: usize = 6;
pub(super) const CELL: f32 = 30.0;
const CELL_SMALL: f32 = 24.0;
pub(super) const GAP: f32 = 4.0;
const GRID_Y: f32 = 2.0;
/// 격자 아래 한 줄(수 · 여벌 정리 단추)의 자리.
const FOOT_Y: f32 = 174.0;
pub(super) const TIDY_W: f32 = 84.0;
const TIDY_H: f32 = 16.0;
/// 내려놓는 자리: 바라보는 쪽 앞으로 이만큼(m — 몸에 가려 안 보이지 않게, 0131). 그 자리가 벽·턱 위거나 낭떠러지면 발치에.
const PUT_AHEAD: f32 = 0.9;
/// 문지기가 떨구는 것이 쓰러진 자리(핵이 남는다)에서 옆으로 비키는 몫.
pub(super) const BESIDE: Vec3 = Vec3::new(1.3, 0.0, 0.0);

/// 덧옷의 칸(2026-11-23 사장 "갑옷은 장비창에만 나타나고 가방에는 안 나타남"): 가방 아이템 뒤에 가진 덧옷을 세운다. 아이템이
/// 아니라(숨은 물건 — 한도 밖, 거두기·내려놓기·잠그기 없음) 칸 번호는 이 바닥 위 — 할 일은 걸치기/벗기뿐.
const GARB_ID: u32 = 1 << 30;

fn garb_of(id: u32) -> Option<crate::garb::Id> {
    id.checked_sub(GARB_ID)
        .and_then(|i| crate::garb::TABLE.get(i as usize))
        .map(|d| d.id)
}

/// 덧옷 칸의 할 일.
fn garb_acts(worn: bool) -> Vec<Do> {
    vec![if worn { Do::Doff } else { Do::Don }]
}

/// 칸 수: 한도(넘쳤으면 가진 수)를 줄에 맞춰 올린 것.
pub(super) fn cells(len: usize) -> usize {
    len.max(BAG_CAP).div_ceil(COLS) * COLS
}

/// 칸에 서는 차례(순수): 무기 → 방패 → 장신구(종류의 칸 차례), 같은 바탕끼리, 값진 것(거두는 값) 먼저, 같으면 먼저 얻은 것.
pub(super) fn order(bag: &Bag) -> Vec<u32> {
    let rank = |b: Base| match b {
        Base::Weapon(w) => (
            0,
            crate::weapons::IDS
                .iter()
                .position(|&i| i == w)
                .unwrap_or(0),
        ),
        Base::Shield(s) => (1, s as usize),
        Base::Charm(c) => (2 + c.kind().slot(), c as usize),
    };
    let mut v: Vec<(u32, Item)> = bag.items.clone();
    v.sort_by_key(|(id, it)| (rank(it.base), std::cmp::Reverse(it.salvage()), *id));
    v.into_iter().map(|(id, _)| id).collect()
}

/// 커서 옮기기(순수): 격자 안에서 위·아래·좌·우 — 가장자리에서 돌지 않는다. `tidy`(화로)면 맨 아랫줄 아래에 "여벌 정리"
/// 단추 하나(자리 = 칸 수)가 있다.
pub(super) fn nav(cur: usize, up: i32, right: i32, n: usize, tidy: bool) -> usize {
    if cur >= n {
        // 단추에서: 위 = 맨 아랫줄의 오른쪽 끝(단추가 그 아래 있다).
        return if up > 0 { n - 1 } else { n };
    }
    let (row, col) = ((cur / COLS) as i32, (cur % COLS) as i32);
    let rows = (n / COLS) as i32;
    let to_row = row - up;
    if to_row >= rows {
        return if tidy { n } else { cur };
    }
    let (r, c) = (to_row.max(0), (col + right).clamp(0, COLS as i32 - 1));
    (r * COLS as i32 + c) as usize
}

/// 고른 아이템에 할 수 있는 일(작은 목록의 줄).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Do {
    /// 주무기로 든다.
    Main,
    /// 보조무기로 든다(무기 · 방패).
    Sub,
    /// 장신구를 제 종류의 칸에 낀다.
    Wear,
    Keep,
    Unkeep,
    /// 발치에 내려놓는다(여벌만).
    Drop,
    /// 잔불로 바꾼다(화로에서, 여벌만).
    Salvage,
    /// 잔불로 바꾸기는 화로에서만 — 흐린 줄(못 하는 까닭을 말한다, 0173).
    Away,
    /// 덧옷을 걸친다 · 벗는다(덧옷 칸).
    Don,
    Doff,
}

/// 그 아이템의 줄들(순수): 끼기(안 낀 것만 — 무기는 주무기·보조무기, 든 무기와 같은 바탕은 주무기만) · 지키기/풀기 ·
/// 여벌이고 지키지 않으면 내려놓기 + 잔불로(`edits` = 화로).
pub(super) fn acts(bag: &Bag, id: u32, main: weapons::Id, edits: bool) -> Vec<Do> {
    let Some(it) = bag.get(id) else {
        return Vec::new();
    };
    let mut v = Vec::new();
    if !bag.worn(id) {
        match it.base {
            Base::Weapon(w) => {
                v.push(Do::Main);
                if w != main {
                    v.push(Do::Sub);
                }
            }
            Base::Shield(_) => v.push(Do::Sub),
            Base::Charm(_) => v.push(Do::Wear),
        }
    }
    v.push(if bag.locked(id) { Do::Unkeep } else { Do::Keep });
    if bag.loose().contains(&id) {
        v.push(Do::Drop);
        v.push(if edits { Do::Salvage } else { Do::Away });
    }
    v
}

/// 아이템의 그림(없는 바탕은 빈 칸).
fn view_of(base: Base) -> Option<crate::item_view::Item> {
    crate::item_view::of_base(base)
}

/// 그 아이템이 설 장비 칸: 낀 것이면 낀 칸, 아니면 제 갈래의 칸(무기 = 주무기 · 방패 = 보조무기 · 장신구 = 종류의 칸).
fn slot_of(bag: &Bag, id: u32, base: Base) -> usize {
    let g = &bag.gear;
    if g.main == Some(id) {
        return 0;
    }
    if g.sub == Some(id) || g.shield == Some(id) {
        return SUB_SLOT;
    }
    if let Some(i) = g.charms.iter().position(|c| *c == Some(id)) {
        return i + 1;
    }
    match base {
        Base::Weapon(_) => 0,
        Base::Shield(_) => SUB_SLOT,
        Base::Charm(c) => c.kind().slot() + 1,
    }
}

/// 아이템 칸의 상태 줄(0278 R22 — 차례: 낀 것 > 새 것 > 잠근 것 > 여벌 > 그 바탕의 마지막 하나).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum BagState {
    Worn,
    New,
    Kept,
    Spare,
    Last,
}

impl BagState {
    fn of(bag: &Bag, id: u32) -> BagState {
        if bag.worn(id) {
            BagState::Worn
        } else if bag.fresh(id) {
            BagState::New
        } else if bag.locked(id) {
            BagState::Kept
        } else if bag.extras().contains(&id) {
            BagState::Spare
        } else {
            BagState::Last
        }
    }

    fn line(self) -> (String, Color) {
        let s = s();
        match self {
            BagState::Worn => (s.equip_worn.to_string(), AMBER),
            BagState::New => (s.bag_st_new.to_string(), ui::MINT),
            BagState::Kept => (s.bag_st_kept.to_string(), INK),
            BagState::Spare => (s.bag_st_spare.to_string(), DIM),
            BagState::Last => (s.bag_st_last.to_string(), DIM),
        }
    }
}

/// 가방 격자의 한 칸(0278 R22): 빈 칸 · 덧옷 · 아이템과 그 표지(낀 것 — 주무기는 찬 모 · 잠금 · 새 것).
#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum BagCell {
    Empty,
    Garb {
        garb: crate::garb::Id,
        worn: bool,
    },
    Item {
        id: u32,
        item: Item,
        worn: bool,
        main: bool,
        locked: bool,
        fresh: bool,
    },
}

/// 오른쪽 칸에 보일 것(0278 R22).
#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum BagPane {
    /// 여벌 정리 단추: 나갈 (수, 잔불).
    Tidy {
        count: usize,
        embers: u32,
    },
    Garb {
        garb: crate::garb::Id,
        worn: bool,
    },
    Item {
        id: u32,
        item: Item,
        /// 그 아이템이 설 장비 칸(`slot_of`).
        slot: usize,
        state: BagState,
        /// 오른쪽 몸의 줄: 견줄 칸의 것과 "후보로 견주나"(낀 것은 그 칸의 지금 것 그대로, 안 낀 것은 그 칸에 낀 것과 견준다).
        focus: Cand,
        picking: bool,
        /// 거둘 수 있는 값을 보이나(안 낀 것)와 잠갔나(값이 흐리다).
        unworn: bool,
        locked: bool,
    },
    /// 빈 칸.
    Empty,
}

/// 가방 탭이 그리는 것(0278 R22 — 순수: 기록 · 든 무기 · 커서에서, GL 없이).
#[derive(Clone, PartialEq, Debug)]
pub(super) struct BagView {
    pub cells: Vec<BagCell>,
    /// 칸 커서(단추면 칸 수) — 범위 안으로 당긴 것.
    pub cursor: usize,
    /// 여벌 정리 단추가 있나(화로).
    pub tidy: bool,
    pub count: usize,
    pub full: bool,
    pub pane: BagPane,
}

impl BagView {
    /// `entries` = 격자의 칸 번호(`App::bag_entries` — 아이템 뒤에 덧옷).
    pub(super) fn build(
        p: &crate::save::Progress,
        weapon: weapons::Id,
        entries: &[u32],
        tidy: bool,
        cursor: usize,
    ) -> BagView {
        let bag = &p.bag;
        let n = cells(entries.len());
        let cursor = cursor.min(if tidy { n } else { n - 1 });
        let cell = |i: usize| match entries.get(i) {
            Some(&id) => match garb_of(id) {
                Some(g) => BagCell::Garb {
                    garb: g,
                    worn: p.garb == Some(g),
                },
                None => match bag.get(id) {
                    Some(it) => BagCell::Item {
                        id,
                        item: *it,
                        worn: bag.worn(id),
                        main: bag.gear.main == Some(id),
                        locked: bag.locked(id),
                        fresh: bag.fresh(id),
                    },
                    None => BagCell::Empty,
                },
            },
            None => BagCell::Empty,
        };
        let cells: Vec<BagCell> = (0..n).map(cell).collect();
        let pane = if cursor >= n {
            let ids = bag.clearable();
            BagPane::Tidy {
                count: ids.len(),
                embers: ids
                    .iter()
                    .filter_map(|id| bag.get(*id))
                    .map(Item::salvage)
                    .sum(),
            }
        } else {
            match cells[cursor] {
                BagCell::Garb { garb, worn } => BagPane::Garb { garb, worn },
                BagCell::Item {
                    id,
                    item,
                    worn,
                    locked,
                    ..
                } => {
                    let slot = slot_of(bag, id, item.base);
                    let (focus, picking) = if worn {
                        (super::equip::slot_now_of(p, weapon, slot), false)
                    } else {
                        (Cand::Item(id), true)
                    };
                    BagPane::Item {
                        id,
                        item,
                        slot,
                        state: BagState::of(bag, id),
                        focus,
                        picking,
                        unworn: bag.spares().contains(&id),
                        locked,
                    }
                }
                BagCell::Empty => BagPane::Empty,
            }
        };
        BagView {
            cells,
            cursor,
            tidy,
            count: bag.items.len(),
            full: bag.full(),
            pane,
        }
    }
}

impl App {
    /// 가방 탭을 떠난다: 지금 있는 것은 모두 본 것(새 것의 점이 꺼진다).
    /// 가방 격자의 칸들: 아이템(`order`) 뒤에 가진 덧옷(`GARB_ID` 위 번호).
    pub(super) fn bag_entries(&self) -> Vec<u32> {
        let mut v = order(&self.progress.bag);
        let have = self.garbs();
        v.extend(
            crate::garb::TABLE
                .iter()
                .enumerate()
                .filter(|(_, d)| have.contains(&d.id))
                .map(|(i, _)| GARB_ID + i as u32),
        );
        v
    }

    pub(super) fn leave_bag(&mut self) {
        self.progress.bag.mark_seen();
        self.persist();
    }

    /// 여벌 정리로 나갈 것의 (수, 잔불).
    pub(super) fn tidy_count(&self) -> (usize, u32) {
        let bag = &self.progress.bag;
        let ids = bag.clearable();
        let embers = ids
            .iter()
            .filter_map(|id| bag.get(*id))
            .map(Item::salvage)
            .sum();
        (ids.len(), embers)
    }

    /// 여벌 정리(묻는 창의 답): 지키지 않는 여벌을 모두 잔불로 — 바탕마다 가장 값진 하나는 남는다(`Bag::clearable`).
    pub(super) fn tidy_bag(&mut self) {
        let ids = self.progress.bag.clearable();
        let mut embers = 0;
        for id in &ids {
            if let Some(it) = self.progress.bag.remove(*id) {
                embers += it.salvage();
            }
        }
        if ids.is_empty() {
            self.snd(crate::sfx::Id::Deny);
            return;
        }
        self.world.earn(embers);
        self.progress.embers = self.world.embers();
        self.progress.settle_items();
        self.snd(crate::sfx::Id::Recall);
        self.notice.toast = Some((
            fill(
                s().toast_tidy,
                &[&ids.len().to_string(), &embers.to_string()],
            ),
            TOAST_TIME,
        ));
        self.persist();
    }

    /// 여벌을 발치에 내려놓는다(0226): 가방에서 빼고 바닥에 — 다시 주울 수 있다(가방에 자리가 있으면), 기록에도 남는다.
    fn put_down(&mut self, id: u32) {
        if !self.progress.bag.loose().contains(&id) {
            self.snd(crate::sfx::Id::Deny);
            return;
        }
        let name = self.item_name(id);
        let Some(item) = self.progress.bag.remove(id) else {
            return;
        };
        // 그 바탕의 마지막 하나였으면 가진 바탕에서도 뺀다(0266 — 다시 주우면 `take_item`이 되돌린다).
        self.progress.part_with(item.base);
        // 바라보는 쪽 앞의 땅에(뛰는 중에 멈췄어도) — 한 번 곁을 떠나야 다시 줍는다(`ItemDrop::wait`).
        // 앞이 벽·소품 칸이면 걸을 수 있는 땅까지 물러난다(`floor_toward` — 벽 속·건너편에 놓이지 않게).
        let pl = &self.world.player;
        let here = pl.pos;
        let ahead = self
            .run
            .level
            .floor_toward(here, here + pl.facing_dir() * PUT_AHEAD);
        let (y0, y1) = (self.world.ground_y(here), self.world.ground_y(ahead));
        let at = if (-1.0..=0.5).contains(&(y1 - y0)) {
            vec3(ahead.x, y1, ahead.z)
        } else {
            vec3(here.x, y0, here.z)
        };
        self.run.drop_item(at, item, true);
        self.fx
            .loot_drop(at, super::progress::glow_of(&item), item.rarity.rank());
        self.note_ground();
        self.progress.settle_items();
        self.snd(crate::sfx::Id::UiMove);
        self.notice.toast = Some((fill(s().toast_put_down, &[&name]), TOAST_TIME));
        self.persist();
    }

    /// 가방 한 틱: 뒤로·✕·창 밖 = 닫기, 십자·스틱 = 칸, 확인(또는 고른 칸 다시 누르기) = 그 아이템의 작은 목록 · 여벌 정리.
    pub(super) fn tick_bag(&mut self, a: &Actions, host: Host) {
        let mut a = *a;
        let before = self.menu.cursor;
        // 격자는 굴리지 않는다 — 휠·끌기는 어디서든 오른쪽 상세 칸을.
        if a.wheel != 0 || a.drag != 0.0 {
            let line = self.ui.u(tabs::LH_BODY).max(1.0);
            self.tab.scroll = super::equip::pane_step(
                self.tab.scroll,
                0.0,
                TICK_DT,
                a.wheel,
                a.drag / line,
                usize::MAX >> 1,
            );
            (a.wheel, a.drag) = (0, 0.0);
        }
        self.pane_scroll_lines(&mut a);
        let a = &a;
        let tap = a.tap.and_then(|p| self.hit_at(p));
        if a.back || a.pause || tap == Some(Hit::Close) {
            self.leave_bag();
            self.go(Go::Back);
            return;
        }
        let ids = self.bag_entries();
        let n = cells(ids.len());
        let tidy = host.edits();
        self.menu.cursor = nav(
            self.menu.cursor.min(if tidy { n } else { n - 1 }),
            a.menu_up as i32 - a.menu_down as i32,
            a.menu_right as i32 - a.menu_left as i32,
            n,
            tidy,
        );
        // 터치·마우스: 다른 칸을 누르면 고르기만, 고른 칸을 다시 누르면 확인(장비 탭과 같은 두 걸음).
        let pick = match tap {
            Some(Hit::Row(i)) => {
                let again = self.menu.cursor == i;
                self.menu.cursor = i;
                again
            }
            _ => a.confirm,
        };
        if self.menu.cursor != before {
            self.tab.scroll = 0.0;
        }
        if !pick {
            return;
        }
        if self.menu.cursor >= n {
            if self.tidy_count().0 == 0 {
                self.notice.toast = Some((s().toast_tidy_none.to_string(), 1.6));
                self.snd(crate::sfx::Id::Deny);
            } else {
                self.ask(flow::Ask::Tidy);
            }
        } else if ids.get(self.menu.cursor).is_some() {
            self.go(Go::Open(Scene::BagAct));
        } else {
            // 빈 칸(오른쪽이 가방의 규칙을 말한다).
            self.snd(crate::sfx::Id::Deny);
        }
    }

    /// 작은 목록 한 틱: 위아래, 확인·탭 = 그 일을 하고 가방으로, 뒤로 = 그냥 가방으로.
    pub(super) fn tick_bag_act(&mut self, a: &Actions) {
        let tap = a.tap.and_then(|p| self.hit_at(p));
        let under = self.stack.under().copied();
        let Some((at, host)) = under.and_then(|e| match e.scene {
            Scene::Bag { host } => Some((e.cursor, host)),
            _ => None,
        }) else {
            return self.go(Go::Back);
        };
        if a.back || a.pause || tap == Some(Hit::Close) {
            return self.go(Go::Back);
        }
        let Some(&id) = self.bag_entries().get(at) else {
            return self.go(Go::Back);
        };
        // 덧옷 칸: 걸치기/벗기만(장비 탭의 덧옷 칸과 같은 일).
        if let Some(g) = garb_of(id) {
            let list = garb_acts(self.progress.garb == Some(g));
            let Some(i) = self.menu_pick(a, list.len()) else {
                return;
            };
            self.go(Go::Back);
            self.progress.garb = (list[i] == Do::Don).then_some(g);
            self.world.apply_stats(self.progress.stats(), false);
            self.persist();
            return;
        }
        let list = acts(&self.progress.bag, id, self.weapon, host.edits());
        let Some(base) = self.progress.bag.get(id).map(|it| it.base) else {
            return self.go(Go::Back);
        };
        let Some(i) = self.menu_pick(a, list.len()) else {
            return;
        };
        self.go(Go::Back);
        let s = s();
        match list[i] {
            Do::Main => self.wear_item(0, id),
            Do::Sub => self.wear_item(SUB_SLOT, id),
            Do::Wear => {
                if let Base::Charm(c) = base {
                    self.wear_item(c.kind().slot() + 1, id);
                }
            }
            Do::Keep | Do::Unkeep => {
                let name = self.item_name(id);
                let line = if self.progress.bag.toggle_lock(id) {
                    s.toast_kept
                } else {
                    s.toast_unkept
                };
                self.notice.toast = Some((fill(line, &[&name]), 1.6));
                self.persist();
            }
            Do::Drop => self.put_down(id),
            Do::Salvage => self.ask(flow::Ask::Salvage(id)),
            Do::Away => {
                self.notice.toast = Some((s.toast_bag_away.to_string(), 1.4));
                self.snd(crate::sfx::Id::Deny);
            }
            Do::Don | Do::Doff => {}
        }
        // 끼면 차례가 그대로여도 낀 표지가 옮겨 간다 — 커서는 그 아이템을 따라간다(내려놓았으면 그 자리).
        if let Some(to) = self.bag_entries().iter().position(|x| *x == id) {
            if matches!(self.scene(), Scene::Bag { .. }) {
                self.menu.cursor = to;
            }
        }
    }

    /// 칸의 자리(화면 px): 격자는 왼쪽 칸의 가운데.
    fn bag_cell(w: &Window, i: usize, n: usize) -> Rect {
        let side = if n > BAG_CAP { CELL_SMALL } else { CELL };
        let width = COLS as f32 * side + (COLS - 1) as f32 * GAP;
        let x0 = (LEFT_W - width) * 0.5;
        w.rect(
            x0 + (side + GAP) * (i % COLS) as f32,
            GRID_Y + (side + GAP) * (i / COLS) as f32,
            side,
            side,
        )
    }

    /// 여벌 정리 단추: 격자의 오른쪽 끝에 맞춘다(칸이 작아져도).
    fn bag_tidy_rect(w: &Window, n: usize) -> Rect {
        let last = Self::bag_cell(w, COLS - 1, n);
        let (tw, th) = (TIDY_W * w.s, TIDY_H * w.s);
        Rect::new(last.x + last.w - tw, w.at(0.0, FOOT_Y).y, tw, th)
    }

    /// 가방 탭 그림: 왼쪽 칸 격자 + 수(화로면 여벌 정리 단추), 오른쪽 고른 것. `cursor` = 칸 커서(작은 목록이 얹혔으면 아래 칸의 것).
    pub(super) fn draw_bag(&mut self, f: &Frame, host: Host, cursor: usize) {
        let s = s();
        let scene = Scene::Bag { host };
        let w = self.draw_tab_window(f, scene);
        let u = w.s;
        // 무엇을 보이나(칸의 표지 · 상태 줄 · 오른쪽 칸)는 그리기 밖의 `BagView`가 정한다(0278 R22).
        let ids = self.bag_entries();
        let view = BagView::build(&self.progress, self.weapon, &ids, host.edits(), cursor);
        let n = view.cells.len();
        let tidy = view.tidy;
        let cursor = view.cursor;
        for (i, cell) in view.cells.iter().enumerate() {
            let r = Self::bag_cell(&w, i, n);
            let sel = i == cursor;
            match *cell {
                // 덧옷 칸: 그림 + 걸쳤으면 왼쪽 위 금빛 테.
                BagCell::Garb { garb: g, worn } => {
                    self.ui.button(r, "", sel);
                    let k = super::equip::icon_scale(crate::render::ICON_PX, r.w - u * 4.0);
                    let side = crate::render::ICON_PX as f32 * k;
                    self.renderer.draw_icon(
                        crate::item_view::Item::Garb(g),
                        Rect::new(
                            (r.x + (r.w - side) * 0.5).round(),
                            (r.y + (r.h - side) * 0.5).round(),
                            side,
                            side,
                        ),
                    );
                    if worn {
                        let d = (u * 4.0).max(2.0).round();
                        let (x, y) = (r.x + u, r.y + u);
                        draw_rectangle(x, y, d + 2.0, d + 2.0, Color::new(0.10, 0.07, 0.08, 1.0));
                        draw_rectangle(x + 1.0, y + 1.0, d, d, AMBER);
                    }
                    self.menu.hits.push((r, Hit::Row(i)));
                    continue;
                }
                BagCell::Item {
                    item: it,
                    worn,
                    main,
                    locked,
                    fresh,
                    ..
                } => {
                    self.ui.button(r, "", sel);
                    if let Some(v) = view_of(it.base) {
                        let k = super::equip::icon_scale(crate::render::ICON_PX, r.w - u * 4.0);
                        let side = crate::render::ICON_PX as f32 * k;
                        self.renderer.draw_icon(
                            v,
                            Rect::new(
                                (r.x + (r.w - side) * 0.5).round(),
                                (r.y + (r.h - side) * 0.5).round(),
                                side,
                                side,
                            ),
                        );
                    }
                    // 등급의 빛: 칸 아래의 띠(보통 것은 없다) — 이름의 등급 말과 함께 쓴다(색만으로 말하지 않는다).
                    if it.rarity != crate::items::Rarity::Plain {
                        let ink = super::equip::rarity_ink(it.rarity);
                        let t = (u * 2.0).max(1.0).round();
                        draw_rectangle(r.x + t, r.y + r.h - t * 2.0, r.w - t * 2.0, t, ink);
                    }
                    let d = (u * 4.0).max(2.0).round();
                    // 낀 것: 왼쪽 위 금빛 모(주무기 = 찬 것 · 그 밖 = 테).
                    if worn {
                        let (x, y) = (r.x + u, r.y + u);
                        draw_rectangle(x, y, d + 2.0, d + 2.0, Color::new(0.10, 0.07, 0.08, 1.0));
                        draw_rectangle(x + 1.0, y + 1.0, d, d, AMBER);
                        if !main {
                            draw_rectangle(
                                x + 2.0,
                                y + 2.0,
                                d - 2.0,
                                d - 2.0,
                                Color::new(0.10, 0.07, 0.08, 1.0),
                            );
                        }
                    }
                    // 지키는 것: 오른쪽 아래 자물쇠(몸 + 고리).
                    if locked {
                        let (x, y) = (r.x + r.w - d - u * 2.0, r.y + r.h - d - u * 3.0);
                        let p = (d / 4.0).max(1.0).round();
                        draw_rectangle(x + p, y - p * 2.0, p * 2.0, p, INK);
                        draw_rectangle(x + p, y - p * 2.0, p, p * 2.0, INK);
                        draw_rectangle(x + p * 2.0, y - p * 2.0, p, p * 2.0, INK);
                        draw_rectangle(x, y, p * 4.0, p * 3.0, INK);
                    }
                    // 새 것: 오른쪽 위 민트 점.
                    if fresh {
                        let p = (u * 3.0).max(2.0).round();
                        draw_rectangle(r.x + r.w - p - u * 2.0, r.y + u * 2.0, p, p, ui::MINT);
                    }
                }
                BagCell::Empty => {
                    // 빈 칸: 옅은 바닥과 테 — 한도가 보인다.
                    draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.0, 0.0, 0.0, 0.22));
                    let ink = if sel { AMBER } else { Color { a: 0.35, ..DIM } };
                    self.ui.edge(r, u, ink);
                }
            }
            self.menu.hits.push((r, Hit::Row(i)));
        }
        // 아래 한 줄: 수(가득이면 호박색) · 화로면 여벌 정리.
        let count = fill(
            s.bag_count,
            &[&view.count.to_string(), &BAG_CAP.to_string()],
        );
        let first = Self::bag_cell(&w, 0, n);
        let k = K_KIND * w.k * 1.15;
        let ty = w.at(0.0, FOOT_Y).y + (TIDY_H * u - self.ui.px(k)) * 0.5;
        self.ui
            .text(&count, first.x, ty, k, if view.full { AMBER } else { DIM });
        if tidy {
            let r = Self::bag_tidy_rect(&w, n);
            self.ui.button(r, "", cursor >= n);
            let tk = K_KIND * w.k;
            let label = self.ui.fit(s.bag_tidy, r.w - u * 6.0, tk);
            let tw = self.ui.width(&label, tk);
            let ink = if cursor >= n { AMBER } else { INK };
            self.ui.text(
                &label,
                (r.x + (r.w - tw) * 0.5).round(),
                r.y + (r.h - self.ui.px(tk)) * 0.5,
                tk,
                ink,
            );
            self.menu.hits.push((r, Hit::Row(n)));
        }

        // 오른쪽: 고른 것.
        let confirm = match view.pane {
            BagPane::Tidy { count: cnt, embers } => {
                self.draw_pane_head(
                    &w,
                    &PaneHead {
                        name: s.bag_tidy,
                        gold: cnt > 0,
                        kind: s.menu_bag.to_string(),
                        state: (
                            fill(s.toast_tidy, &[&cnt.to_string(), &embers.to_string()]),
                            if cnt > 0 { ui::MINT } else { DIM },
                        ),
                        icon: false,
                    },
                );
                let lines = self.pane_wrap(&w, s.bag_tidy_d, INK);
                self.draw_pane(&w, HEAD_H, &lines);
                cnt > 0
            }
            BagPane::Garb { garb: g, worn } => {
                // 덧옷: 이름 · 덧옷 · 걸쳤나, 그림, 설명과 세계 안의 말(장비 탭의 덧옷 칸과 같은 글).
                let d = crate::garb::def(g);
                let icon = self.draw_pane_head(
                    &w,
                    &PaneHead {
                        name: tr(d.name),
                        gold: worn,
                        kind: s.codex_tab_garb.to_string(),
                        state: if worn {
                            (s.equip_worn.to_string(), AMBER)
                        } else {
                            (String::new(), DIM)
                        },
                        icon: true,
                    },
                );
                self.draw_pane_item(crate::item_view::Item::Garb(g), icon);
                let mut lines = self.pane_wrap(&w, tr(d.desc), INK);
                lines.push(Ln::Gap);
                lines.extend(self.pane_wrap(&w, tr(d.flavor), DIM));
                self.draw_pane(&w, HEAD_H, &lines);
                true
            }
            BagPane::Item {
                item: it,
                slot,
                state,
                focus,
                picking,
                unworn,
                locked,
                ..
            } => {
                let v = view_of(it.base);
                let icon = self.draw_pane_head(
                    &w,
                    &PaneHead {
                        // 등급은 갈래 줄에(0275 — 이름 칸이 넘쳤다).
                        name: Self::base_name(&it),
                        gold: state == BagState::Worn,
                        kind: crate::i18n::with_rarity(Some(it.rarity), &slot_name(slot)),
                        state: state.line(),
                        icon: v.is_some(),
                    },
                );
                if let Some(v) = v {
                    self.draw_pane_item(v, icon);
                }
                // 장비 탭과 같은 줄: 낀 것은 그 칸의 지금 것으로, 안 낀 것은 그 칸에 지금 낀 것과 견줘.
                let mut lines = self.equip_lines(&w, slot, focus, picking);
                if unworn {
                    lines.push(Ln::Gap);
                    lines.push(Ln::Pair(
                        s.bag_value.to_string(),
                        format!("+{}", it.salvage()),
                        if locked { DIM } else { AMBER },
                    ));
                }
                self.draw_pane(&w, HEAD_H, &lines);
                true
            }
            BagPane::Empty => {
                self.draw_pane_head(
                    &w,
                    &PaneHead {
                        name: s.bag_empty,
                        gold: false,
                        kind: s.menu_bag.to_string(),
                        state: (count.clone(), if view.full { AMBER } else { DIM }),
                        icon: false,
                    },
                );
                let lines = self.pane_wrap(&w, s.bag_empty_d, INK);
                self.draw_pane(&w, HEAD_H, &lines);
                false
            }
        };
        let right = w.panel.x + w.panel.w;
        let mut pairs = vec![(Act::Walk, s.verb_choose)];
        if confirm {
            pairs.push((Act::Confirm, s.verb_confirm));
        }
        self.tab_foot(f, &w, scene, right, &pairs);
    }

    /// 작은 목록 그림: 가방 탭을 그 커서 그대로 깔고, 고른 칸 곁에 줄들.
    pub(super) fn draw_bag_act(&mut self, f: &Frame) {
        let Some(under) = self.stack.under().copied() else {
            return;
        };
        let Scene::Bag { host } = under.scene else {
            return;
        };
        let (cursor, view) = (self.menu.cursor, self.menu.view);
        (self.menu.cursor, self.menu.view) = (under.cursor, under.view);
        self.draw_layered(f, under.scene);
        (self.menu.cursor, self.menu.view) = (cursor, view);
        // 작은 창은 누를 자리를 제 줄과 창 밖만 남긴다(묻는 창과 같다).
        self.menu.hits.clear();
        #[cfg(feature = "dev")]
        crate::ui::check::forget();
        let s = s();
        let ids = self.bag_entries();
        let Some(&id) = ids.get(under.cursor) else {
            return;
        };
        let value = self.progress.bag.get(id).map_or(0, Item::salvage);
        let list = match garb_of(id) {
            Some(g) => garb_acts(self.progress.garb == Some(g)),
            None => acts(&self.progress.bag, id, self.weapon, host.edits()),
        };
        let rows: Vec<ui::Row> = list
            .into_iter()
            .map(|d| {
                let label = match d {
                    Do::Main => s.bag_do_main.to_string(),
                    Do::Sub => s.bag_do_sub.to_string(),
                    Do::Wear => s.bag_do_wear.to_string(),
                    Do::Keep => s.bag_do_keep.to_string(),
                    Do::Unkeep => s.bag_do_unkeep.to_string(),
                    Do::Drop => s.bag_do_drop.to_string(),
                    Do::Salvage => fill(s.bag_do_salvage, &[&value.to_string()]),
                    Do::Away => s.bag_do_away.to_string(),
                    Do::Don => s.bag_do_don.to_string(),
                    Do::Doff => s.bag_do_doff.to_string(),
                };
                ui::Row {
                    dim: d == Do::Away,
                    ..ui::Row::plain(label)
                }
            })
            .collect();
        if rows.is_empty() {
            return;
        }
        let u = self.ui.u(1.0);
        let px = self.ui.px(1.0);
        let row_h = px * 1.75;
        let widest = rows
            .iter()
            .map(|r| self.ui.width(&r.label, 1.0))
            .fold(0.0, f32::max);
        // 줄의 글 앞 여백(▶ 자리)·오른쪽 여백 + 창 안 여백 + 여유(기술 탭의 작은 목록과 같은 셈).
        let pw = widest + u * 24.0 + px * 1.1;
        let ph = row_h * rows.len() as f32 + u * 8.0;
        // 고른 칸의 오른쪽에(안 들어가면 왼쪽에), 창 안으로.
        let w = self.window_geo(f);
        let cell = Self::bag_cell(&w, under.cursor, cells(ids.len()));
        let gap = u * 5.0;
        let mut x = cell.x + cell.w + gap;
        if x + pw > f.ox + f.w - u * 6.0 {
            x = cell.x - gap - pw;
        }
        let x = x.max(f.ox + u * 6.0).round();
        let y = cell
            .y
            .min(w.panel.y + w.panel.h - ph - u * 4.0)
            .max(w.panel.y + u * 4.0)
            .round();
        let panel = Rect::new(x, y, pw, ph);
        draw_rectangle(
            x + u * 2.0,
            y + u * 2.0,
            pw,
            ph,
            Color::new(0.01, 0.0, 0.02, 0.7),
        );
        draw_rectangle(
            x,
            y,
            pw,
            ph,
            Color {
                a: 1.0,
                ..ui::PANEL
            },
        );
        self.ui.panel(panel, None);
        self.ui.edge(panel, u, AMBER);
        let area = Rect::new(x + u * 3.0, y + u * 4.0, pw - u * 6.0, ph - u * 8.0);
        let hits = self
            .ui
            .list(&rows, self.menu.cursor, &mut self.menu.view, area);
        self.push_hits(&hits);
        self.push_outside(panel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charms;
    use crate::items::{Rarity, Worn};
    use weapons::Id as W;

    fn bag_with(bases: &[Base]) -> Bag {
        let mut bag = Bag::default();
        for b in bases {
            bag.add(Item::plain(*b));
        }
        bag
    }

    fn rare(base: Base) -> Item {
        Item {
            rarity: Rarity::Rare,
            ..Item::plain(base)
        }
    }

    /// 0226: 칸은 한도만큼(넘쳤으면 줄이 는다), 커서는 가장자리에서 돌지 않고, 화로에서는 맨 아랫줄 아래에 단추 하나.
    #[test]
    fn the_grid_shows_the_cap_and_the_cursor_stops_at_the_edges() {
        assert_eq!(cells(0), BAG_CAP);
        assert_eq!(cells(BAG_CAP), BAG_CAP);
        assert_eq!(cells(BAG_CAP + 1), BAG_CAP + COLS, "one more row");
        let n = cells(0);
        assert_eq!(nav(0, 1, 0, n, false), 0, "top edge");
        assert_eq!(nav(0, 0, -1, n, false), 0, "left edge");
        assert_eq!(nav(5, 0, 1, n, false), 5, "right edge — no wrap");
        assert_eq!(nav(0, -1, 0, n, false), COLS);
        assert_eq!(nav(7, 0, 1, n, false), 8);
        assert_eq!(
            nav(n - 1, -1, 0, n, false),
            n - 1,
            "bottom edge while paused"
        );
        assert_eq!(nav(n - 3, -1, 0, n, true), n, "the tidy button below");
        assert_eq!(nav(n, 1, 0, n, true), n - 1, "and back up");
        assert_eq!(nav(n, 0, 1, n, true), n);
        assert_eq!(nav(n, -1, 0, n, true), n);
    }

    /// 차례: 무기 → 방패 → 장신구, 같은 바탕끼리 모이고 값진 것이 앞.
    #[test]
    fn items_stand_by_kind_then_base_then_worth() {
        let mut bag = bag_with(&[
            Base::Charm(charms::Id::Ear),
            Base::Weapon(W::Hammer),
            Base::Shield(crate::shield::Id::Pilgrim),
            Base::Weapon(W::Sword),
        ]);
        let best = bag.add(rare(Base::Weapon(W::Sword)));
        let got = order(&bag);
        let bases: Vec<Base> = got.iter().map(|id| bag.get(*id).unwrap().base).collect();
        assert_eq!(
            bases,
            [
                Base::Weapon(W::Sword),
                Base::Weapon(W::Sword),
                Base::Weapon(W::Hammer),
                Base::Shield(crate::shield::Id::Pilgrim),
                Base::Charm(charms::Id::Ear),
            ]
        );
        assert_eq!(got[0], best, "the rare sword stands before the plain one");
    }

    /// 할 수 있는 일: 낀 것은 끼기가 없고, 끼지 않은 것은 내려놓고 거둔다(하나뿐인 것도, 0266) — 지키면 둘 다 사라진다. 잔불로는 화로에서만(멈춤은 흐린 줄).
    /// 2026-11-23: 덧옷 칸은 아이템 번호와 겹치지 않는 자리에서 제 덧옷으로 읽히고, 할 일은 걸치기/벗기 하나.
    #[test]
    fn garment_cells_read_back_and_only_put_on_or_take_off() {
        for (i, d) in crate::garb::TABLE.iter().enumerate() {
            assert_eq!(garb_of(GARB_ID + i as u32), Some(d.id));
        }
        assert_eq!(garb_of(0), None);
        assert_eq!(garb_of(GARB_ID - 1), None);
        assert_eq!(garb_of(GARB_ID + crate::garb::TABLE.len() as u32), None);
        assert_eq!(garb_acts(false), [Do::Don]);
        assert_eq!(garb_acts(true), [Do::Doff]);
    }

    #[test]
    fn acts_follow_what_the_item_is() {
        let mut bag = bag_with(&[Base::Weapon(W::Sword), Base::Weapon(W::Hammer)]);
        let spare = bag.add(rare(Base::Weapon(W::Sword)));
        let owned = [Base::Weapon(W::Sword), Base::Weapon(W::Hammer)];
        bag.settle(
            &owned,
            &Worn {
                main: Some(W::Sword),
                ..Worn::default()
            },
        );
        let worn = bag.gear.main.unwrap();
        let hammer = bag.of(Base::Weapon(W::Hammer)).next().unwrap().0;
        // 낀 검: 지키기뿐.
        assert_eq!(acts(&bag, worn, W::Sword, true), [Do::Keep]);
        // 하나뿐인 망치: 들 수 있고, 끼지 않았으니 내려놓거나 거둘 수도 있다(0266 — 사장 "장비가 하나밖에 없을 때 팔 수 없었던 것").
        assert_eq!(
            acts(&bag, hammer, W::Sword, true),
            [Do::Main, Do::Sub, Do::Keep, Do::Drop, Do::Salvage]
        );
        // 검의 여벌: 주무기로만(든 것과 같은 바탕), 내려놓기, 화로면 잔불로.
        assert_ne!(spare, worn);
        assert_eq!(
            acts(&bag, spare, W::Sword, true),
            [Do::Main, Do::Keep, Do::Drop, Do::Salvage]
        );
        assert_eq!(
            acts(&bag, spare, W::Sword, false),
            [Do::Main, Do::Keep, Do::Drop, Do::Away]
        );
        assert!(bag.toggle_lock(spare));
        assert_eq!(acts(&bag, spare, W::Sword, true), [Do::Main, Do::Unkeep]);
        assert!(acts(&bag, 999, W::Sword, true).is_empty());
    }

    /// 0278 R22: 가방 칸의 상태 줄 차례는 낀 것 > 새 것 > 잠근 것 > 여벌 > 그 바탕의 마지막 하나 — 그리기 밖의 `BagView`가 정한다.
    /// 낀 것을 고르면 그 칸의 지금 것을 그대로, 안 낀 것은 그 칸에 낀 것과 견준다.
    #[test]
    fn bag_states_go_worn_new_kept_spare_last_and_the_pane_compares_the_unworn() {
        let mut p = crate::save::Progress::default();
        p.bag = bag_with(&[Base::Weapon(W::Sword), Base::Weapon(W::Hammer)]);
        let spare = p.bag.add(rare(Base::Weapon(W::Sword)));
        p.bag.settle(
            &[Base::Weapon(W::Sword), Base::Weapon(W::Hammer)],
            &Worn {
                main: Some(W::Sword),
                ..Worn::default()
            },
        );
        let worn = p.bag.gear.main.unwrap();
        let hammer = p.bag.of(Base::Weapon(W::Hammer)).next().unwrap().0;
        let ids = order(&p.bag);
        let state = |p: &crate::save::Progress, id: u32| {
            let at = ids.iter().position(|&i| i == id).unwrap();
            match BagView::build(p, W::Sword, &ids, false, at).pane {
                BagPane::Item { state, .. } => state,
                other => panic!("{other:?}"),
            }
        };
        // 다 새 것이다(가방 탭을 아직 안 봤다) — 낀 것만 "착용 중".
        assert_eq!(state(&p, worn), BagState::Worn);
        assert_eq!(state(&p, spare), BagState::New);
        p.bag.mark_seen();
        assert_eq!(state(&p, spare), BagState::Spare);
        assert_eq!(state(&p, hammer), BagState::Last);
        assert!(p.bag.toggle_lock(spare));
        assert_eq!(state(&p, spare), BagState::Kept);
        assert!(p.bag.toggle_lock(worn));
        assert_eq!(state(&p, worn), BagState::Worn);
        // 오른쪽 칸: 낀 검은 주무기 칸의 지금 것, 안 낀 망치는 후보로(값도 보인다).
        let at = |id: u32| ids.iter().position(|&i| i == id).unwrap();
        match BagView::build(&p, W::Sword, &ids, false, at(worn)).pane {
            BagPane::Item {
                slot,
                focus,
                picking,
                unworn,
                ..
            } => {
                assert_eq!(
                    (slot, focus, picking, unworn),
                    (0, Cand::Weapon(W::Sword), false, false)
                );
            }
            other => panic!("{other:?}"),
        }
        match BagView::build(&p, W::Sword, &ids, false, at(hammer)).pane {
            BagPane::Item {
                slot,
                focus,
                picking,
                unworn,
                locked,
                ..
            } => {
                assert_eq!(
                    (slot, focus, picking, unworn, locked),
                    (0, Cand::Item(hammer), true, true, false)
                );
            }
            other => panic!("{other:?}"),
        }
    }

    /// 0278 R22: 격자의 칸은 아이템 뒤에 덧옷, 나머지는 빈 칸 — 낀 표지는 주무기만 찬 모. 커서는 격자 안으로(화로면 단추까지),
    /// 단추에서는 여벌 정리로 나갈 수와 잔불.
    #[test]
    fn bag_cells_mark_main_and_others_and_the_cursor_reaches_the_tidy_button_only_at_the_hearth() {
        let mut p = crate::save::Progress::default();
        p.bag = bag_with(&[
            Base::Weapon(W::Sword),
            Base::Shield(crate::shield::Id::Pilgrim),
        ]);
        let extra = p.bag.add(Item::plain(Base::Weapon(W::Sword)));
        p.bag.settle(
            &[
                Base::Weapon(W::Sword),
                Base::Shield(crate::shield::Id::Pilgrim),
            ],
            &Worn {
                main: Some(W::Sword),
                shield: Some(crate::shield::Id::Pilgrim),
                ..Worn::default()
            },
        );
        let ids = order(&p.bag);
        let v = BagView::build(&p, W::Sword, &ids, false, 999);
        assert_eq!(v.cells.len(), cells(ids.len()));
        assert_eq!(v.cursor, v.cells.len() - 1);
        assert_eq!(v.pane, BagPane::Empty);
        let marks: Vec<(bool, bool)> = v
            .cells
            .iter()
            .filter_map(|c| match *c {
                BagCell::Item { worn, main, .. } => Some((worn, main)),
                _ => None,
            })
            .collect();
        assert_eq!(marks, [(true, true), (false, false), (true, false)]);
        assert!(v.cells[3..].iter().all(|c| *c == BagCell::Empty));
        let v = BagView::build(&p, W::Sword, &ids, true, 999);
        assert_eq!(v.cursor, v.cells.len());
        let salvage = p.bag.get(extra).unwrap().salvage();
        assert_eq!(
            v.pane,
            BagPane::Tidy {
                count: 1,
                embers: salvage
            }
        );
        assert_eq!((v.count, v.full), (3, false));
    }
}
