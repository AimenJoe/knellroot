//! 기술 판(0158 G0 — 화로 → 기술): 기술 점수로 스킬을 배우고 단을 올린다. 줄 표 + 고름(0066, 레벨 창 `hearth.rs`의 꼴).
//! 줄 = 스킬마다 "이름 … 단/5"(3단부터는 ◀ 고르기 ▶ — 좌우로 값 없이 바꾼다) · 큰 기술 · 되돌리기 · 돌아가기.
//! 확인 = 다음 단을 묻고 산다(점수 1). 잠긴 줄(쓴 점수가 모자람)도 이름·하는 일·고르기 둘이 다 보인다 — 살 수만 없다.
//! 셈은 `skills::Board`(순수), 줄과 설명 글은 여기의 순수 함수(`board_rows`·`board_note` — 시험).
use super::menu::{Menu, Out};
use super::skills::skill_text;
use super::*;
use crate::skills::{self as sk, Board, Entry, Form, Offer, Pick, ENTRIES, PICK_RANK, SIZE_RANK};

/// 판의 알림이 설명 자리에 떠 있는 초(짧게 — 그동안 다음 단의 설명이 가려진다).
const BOARD_ALERT: f32 = 1.4;

/// 줄: 판의 칸들 · 되돌리기 · 돌아가기.
const RESPEC_ROW: usize = ENTRIES.len();
const BACK_ROW: usize = ENTRIES.len() + 1;

/// 칸의 이름.
pub(super) fn entry_name(e: Entry) -> &'static str {
    match e {
        Entry::Skill(id) => skill_text(id).0,
        Entry::Ult => s().ult_upend,
    }
}

/// 고르기의 표지(ㄱ·ㄴ)·이름·한 줄.
pub(super) fn pick_text(id: sk::Id, p: Pick) -> (&'static str, &'static str, &'static str) {
    let s = s();
    let mark = match p {
        Pick::A => s.pick_mark_a,
        Pick::B => s.pick_mark_b,
    };
    let (name, desc) = match (id, p) {
        (sk::Id::Whirl, Pick::A) => (s.pick_whirl_a, s.pick_whirl_a_d),
        (sk::Id::Whirl, Pick::B) => (s.pick_whirl_b, s.pick_whirl_b_d),
        (sk::Id::Cut, Pick::A) => (s.pick_cut_a, s.pick_cut_a_d),
        (sk::Id::Cut, Pick::B) => (s.pick_cut_b, s.pick_cut_b_d),
        (sk::Id::Gather, Pick::A) => (s.pick_gather_a, s.pick_gather_a_d),
        (sk::Id::Gather, Pick::B) => (s.pick_gather_b, s.pick_gather_b_d),
        (sk::Id::Quake, Pick::A) => (s.pick_quake_a, s.pick_quake_a_d),
        (sk::Id::Quake, Pick::B) => (s.pick_quake_b, s.pick_quake_b_d),
    };
    (mark, name, desc)
}

/// 맺음의 한 줄.
fn cap_text(id: sk::Id) -> &'static str {
    let s = s();
    match id {
        sk::Id::Whirl => s.cap_whirl,
        sk::Id::Cut => s.cap_cut,
        sk::Id::Gather => s.cap_gather,
        sk::Id::Quake => s.cap_quake,
    }
}

/// 배율 → "+15" 같은 백분율 글(표의 수에서 — 글에 숫자를 박지 않는다).
fn pct(mult: f32) -> String {
    ((mult - 1.0).abs() * 100.0).round().to_string()
}

/// 스킬 `id`의 `rank`단이 주는 것(한 줄).
fn rank_gives(id: sk::Id, rank: u8) -> String {
    let s = s();
    let d = id.def();
    let harmless = d.damage == 0;
    match rank {
        0 | 1 => skill_text(id).1.to_string(),
        2 if harmless => fill(s.rk_stagger, &[&format!("{:.1}", sk::RANK2_STAGGER)]),
        2 => fill(s.rk_damage, &[&pct(sk::RANK2_DAMAGE)]),
        r if r == PICK_RANK => {
            let (a, b) = (pick_text(id, Pick::A), pick_text(id, Pick::B));
            fill(s.board_pick_both, &[a.1, a.2, b.1, b.2])
        }
        r if r == SIZE_RANK => {
            let t = match d.form {
                Form::Ring { .. } => s.rk_radius,
                Form::Lane { .. } => s.rk_length,
                Form::Pull { .. } => s.rk_range,
                Form::Cone { .. } => s.rk_arc,
            };
            fill(t, &[&pct(sk::RANK4_SIZE)])
        }
        _ => {
            let cool = fill(s.rk_cooldown, &[&pct(sk::RANK5_COOLDOWN)]);
            let mut parts = Vec::new();
            if !harmless {
                // 5단의 피해는 2단의 몫에 더한 것(합 +30%).
                let more = pct(1.0 + sk::RANK5_DAMAGE - sk::RANK2_DAMAGE);
                parts.push(fill(s.rk_damage_more, &[&more]));
            }
            parts.push(cool);
            parts.push(cap_text(id).to_string());
            parts.join(" · ")
        }
    }
}

/// 큰 기술의 `rank`단이 주는 것.
fn ult_gives(rank: u8) -> String {
    let s = s();
    match rank {
        0 | 1 => s.ult_upend_d.to_string(),
        2 => fill(s.rk_stagger, &[&format!("{:.1}", sk::ULT_RANK2_STAGGER)]),
        _ => fill(s.rk_slam, &[&pct(sk::ULT_RANK3_SLAM)]),
    }
}

/// 고른 칸의 설명(세 줄 안): **다음 단**이 주는 것 — 안 배운 스킬은 하는 일 + 3단의 둘, 3단을 넘겼으면 지금 고르기 한 줄을 앞에.
pub(super) fn board_note(board: &Board, e: Entry) -> String {
    let s = s();
    let rank = board.rank(e);
    let next = rank + 1;
    let step = |text: String| fill(s.board_next, &[&next.to_string(), &text]);
    match e {
        Entry::Ult if rank >= e.max() => s.board_full.to_string(),
        Entry::Ult => step(ult_gives(next)),
        Entry::Skill(id) => {
            let now = (rank >= PICK_RANK).then(|| {
                let (mark, name, desc) = pick_text(id, board.pick[id.index()]);
                fill(s.board_pick_now, &[mark, name, desc])
            });
            let then = if rank >= e.max() {
                s.board_full.to_string()
            } else {
                step(rank_gives(id, next))
            };
            match (now, rank) {
                (Some(now), _) => format!("{now} {then}"),
                // 안 배운 스킬: 하는 일 + 3단에서 고를 둘의 이름(잠긴 줄도 다 보인다).
                (None, 0) => {
                    let (a, b) = (pick_text(id, Pick::A).1, pick_text(id, Pick::B).1);
                    format!("{then} {}", fill(s.board_picks_short, &[a, b]))
                }
                (None, _) => then,
            }
        }
    }
}

/// 판의 줄 표(순수 — 시험): 칸마다 "이름 … 단/끝"(잠겼으면 "쓴 점수 n부터", 3단부터는 ◀ 고르기 ▶), 살 수 없으면 흐리게.
/// 되돌리기는 쓴 점수가 없거나 잔불이 모자라면 흐리게.
pub(super) fn board_rows(board: &Board, total: u32, embers: u32) -> Vec<ui::Row> {
    let s = s();
    let mut rows: Vec<ui::Row> = ENTRIES
        .iter()
        .map(|&e| {
            let offer = board.offer(e, total);
            let (rank, max) = (board.rank(e).to_string(), e.max().to_string());
            let row = match (e, offer) {
                (_, Offer::Locked(_)) => {
                    let need = sk::ROW_NEEDS[e.row()].to_string();
                    ui::Row::value(entry_name(e), fill(s.board_locked, &[&need]))
                }
                (Entry::Skill(id), _) if board.rank(e) >= PICK_RANK => {
                    let (mark, name, _) = pick_text(id, board.pick[id.index()]);
                    let pick = format!("{mark} {name}");
                    ui::Row::step(
                        entry_name(e),
                        fill(s.board_rank_pick, &[&rank, &max, &pick]),
                    )
                }
                _ => ui::Row::value(entry_name(e), fill(s.board_rank, &[&rank, &max])),
            };
            row.dimmed(offer != Offer::Ready)
        })
        .collect();
    let cost = board.respec_cost();
    rows.push(
        ui::Row::value(s.board_respec, fill(s.shop_embers, &[&cost.to_string()]))
            .dimmed(board.spent() == 0 || embers < cost),
    );
    rows.push(ui::Row::plain(s.controls_back));
    rows
}

impl App {
    /// 지금 쓰는 판: 기록의 것 — 개발 창에서 끝 단으로 올렸으면 그것(기록에 쓰지 않는다, 0075).
    pub(super) fn board(&self) -> Board {
        #[cfg(feature = "dev")]
        if let Some(b) = self.dev_board {
            return b;
        }
        self.progress.board
    }

    /// 월드에 넘길 끼운 둘(그 단·고르기) + 큰 기술의 단.
    pub(super) fn loadout(&self) -> sk::Loadout {
        self.board().loadout(self.progress.skills.0)
    }

    pub(super) fn board_menu(&self) -> Menu {
        let s = s();
        let board = self.progress.board;
        let total = self.progress.skill_points();
        let rows = board_rows(&board, total, self.world.embers);
        let note = match self.cursor {
            i if i < ENTRIES.len() => board_note(&board, ENTRIES[i]),
            RESPEC_ROW => s.board_respec_d.to_string(),
            _ => s.board_hint.to_string(),
        };
        let (left, embers) = (board.left(total).to_string(), self.world.embers.to_string());
        Menu {
            purse: Some(fill(s.board_purse, &[&left, &embers])),
            note_room: true,
            note: Some(note),
            note_lines: 3,
            // 알림은 설명 자리에 잠깐(창이 화면 높이를 다 써서 위쪽 알림 자리가 없다 — 판 밖에서는 평소대로 위에 뜬다).
            alert: self.toast.as_ref().map(|(t, _)| t.clone()),
            ..Menu::new(s.board_title, 420.0, rows)
        }
    }

    pub(super) fn board_act(&mut self, out: Out) {
        let s = s();
        let deny = |app: &mut App, text: String| {
            app.toast = Some((text, BOARD_ALERT));
            app.snd(crate::sfx::Id::Deny);
        };
        match out {
            // 좌우: 3단의 고르기를 값 없이 바꾼다(지금 판에 바로).
            Out::Step(i, _) => {
                let Some(&Entry::Skill(id)) = ENTRIES.get(i) else {
                    return;
                };
                if self.progress.board.flip(id) {
                    // 알림은 띄우지 않는다 — 줄의 값과 아래 설명이 바로 새 고르기를 보인다(알림은 그 설명을 가린다).
                    self.toast = None;
                    self.snd(crate::sfx::Id::UiMove);
                    self.world.set_skills(self.loadout());
                    self.persist();
                }
            }
            Out::Pick(BACK_ROW) => self.go(Go::Back),
            Out::Pick(RESPEC_ROW) => {
                let board = &self.progress.board;
                if board.spent() == 0 {
                    deny(self, s.board_respec_none.to_string());
                } else if self.world.embers < board.respec_cost() {
                    deny(self, s.shop_poor.to_string());
                } else {
                    self.ask(flow::Ask::Respec);
                }
            }
            Out::Pick(i) => {
                let Some(&e) = ENTRIES.get(i) else { return };
                // 살 수 없으면 묻지 않고 알림만(레벨·가게와 같다 — 흐린 줄).
                match self.progress.board.offer(e, self.progress.skill_points()) {
                    Offer::Ready => self.ask(flow::Ask::Skill(i as u8)),
                    Offer::Short => deny(self, s.board_short.to_string()),
                    Offer::Locked(n) => {
                        deny(self, fill(s.board_locked_toast, &[&n.to_string()]));
                    }
                    Offer::Full => deny(self, s.board_full.to_string()),
                }
            }
        }
    }

    /// 묻는 창에서 "올리기": 기술 점수 1로 그 칸의 다음 단 — 지금 판의 월드에 바로 들어간다.
    pub(super) fn buy_skill(&mut self, i: usize) {
        let Some(&e) = ENTRIES.get(i) else { return };
        let total = self.progress.skill_points();
        if !self.progress.board.buy(e, total) {
            self.snd(crate::sfx::Id::Deny);
            return;
        }
        self.snd(crate::sfx::Id::Buy);
        let rank = self.progress.board.rank(e).to_string();
        self.toast = Some((fill(s().toast_level, &[entry_name(e), &rank]), BOARD_ALERT));
        self.world.set_skills(self.loadout());
        self.persist();
    }

    /// 묻는 창에서 "되돌리기": 잔불을 내고 쓴 점수를 모두 돌려받는다 — 끼운 칸에 안 배운 스킬이 남으면 처음의 둘로.
    pub(super) fn respec_skills(&mut self) {
        let mut worn = self.progress.skills.0;
        if !self
            .progress
            .board
            .respec_paid(&mut self.world.embers, &mut worn)
        {
            self.snd(crate::sfx::Id::Deny);
            return;
        }
        self.progress.embers = self.world.embers;
        self.progress.skills = sk::Worn(worn);
        self.snd(crate::sfx::Id::Buy);
        self.toast = Some((s().toast_respec.to_string(), BOARD_ALERT));
        self.world.set_skills(self.loadout());
        self.persist();
    }

    /// 묻는 창의 글: "땅울림 1단 — 기술 점수 1. 올릴까요?".
    pub(super) fn ask_skill_text(&self, i: usize) -> String {
        let Some(&e) = ENTRIES.get(i) else {
            return String::new();
        };
        let next = (self.progress.board.rank(e) + 1).min(e.max()).to_string();
        fill(s().ask_skill, &[entry_name(e), &next])
    }

    /// "쓴 점수 6을 모두 돌려받는다 — 잔불 60. 되돌릴까요?".
    pub(super) fn ask_respec_text(&self) -> String {
        let b = &self.progress.board;
        fill(
            s().ask_respec,
            &[&b.spent().to_string(), &b.respec_cost().to_string()],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sk::Id;

    /// 0158 라: 잠긴 줄도 이름과 "쓴 점수 n부터"가 보인다 — 흐리고 살 수 없을 뿐. 열린 줄은 "단/5", 점수가 없으면 흐리다.
    #[test]
    fn locked_rows_show_but_are_dim_and_open_rows_dim_only_without_points() {
        let b = Board::default();
        // 점수 3: 첫 줄 셋은 살 수 있고(밝다), 땅울림·큰 기술은 잠겨 흐리다.
        let rows = board_rows(&b, 3, 0);
        assert_eq!(rows.len(), ENTRIES.len() + 2, "+ respec + back");
        for (i, e) in ENTRIES.iter().enumerate() {
            let open = e.row() == 0;
            assert_eq!(rows[i].dim, !open, "{e:?}");
            assert_eq!(rows[i].label, entry_name(*e));
            assert!(!rows[i].value.is_empty());
            assert_eq!(rows[i].kind, ui::RowKind::Plain, "no pick below rank 3");
        }
        assert_eq!(rows[0].value, fill(s().board_rank, &["1", "5"]));
        assert_eq!(rows[2].value, fill(s().board_rank, &["0", "5"]));
        assert_eq!(rows[3].value, fill(s().board_locked, &["4"]));
        assert_eq!(rows[4].value, fill(s().board_locked, &["8"]));
        // 점수가 없으면 열린 줄도 흐리다.
        assert!(board_rows(&b, 0, 0)
            .iter()
            .take(ENTRIES.len())
            .all(|r| r.dim));
        // 돌아가기는 늘 고를 수 있다.
        assert!(!rows[BACK_ROW].dim);
    }

    /// 3단부터 그 줄은 좌우로 바꾸는 값 줄(◀ 고르기 ▶)이 되고 지금 고르기의 이름을 보인다 — 끝 단이어도(살 수 없어 흐려도) 그대로.
    #[test]
    fn a_rank_three_row_becomes_a_step_row_naming_its_pick() {
        let mut b = Board::default();
        b.rank[Id::Whirl.index()] = 3;
        let rows = board_rows(&b, 20, 0);
        assert_eq!(rows[0].kind, ui::RowKind::Step);
        assert!(rows[0].value.contains(s().pick_whirl_a));
        assert_eq!(rows[1].kind, ui::RowKind::Plain);
        b.flip(Id::Whirl);
        assert!(board_rows(&b, 20, 0)[0].value.contains(s().pick_whirl_b));
        b.rank[Id::Whirl.index()] = 5;
        let rows = board_rows(&b, 20, 0);
        assert_eq!(rows[0].kind, ui::RowKind::Step, "still flips at max");
        assert!(rows[0].dim, "nothing more to buy");
    }

    /// 되돌리기 줄: 쓴 점수가 없거나 잔불이 모자라면 흐리고, 값은 `30 + 5 × 쓴 점수`.
    #[test]
    fn the_respec_row_is_dim_with_nothing_spent_or_too_few_embers() {
        let mut b = Board::default();
        assert!(board_rows(&b, 10, 999)[RESPEC_ROW].dim, "nothing spent");
        assert!(b.buy(Entry::Skill(Id::Gather), 10) && b.buy(Entry::Skill(Id::Gather), 10));
        let cost = b.respec_cost();
        assert_eq!(cost, 40);
        assert!(board_rows(&b, 10, cost - 1)[RESPEC_ROW].dim, "too poor");
        let row = &board_rows(&b, 10, cost)[RESPEC_ROW];
        assert!(!row.dim);
        assert!(row.value.contains("40"));
    }

    /// 설명은 **다음 단**을 말한다: 2단 = 피해(끌어모으기는 휘청), 3단 = 고르기 둘 다, 4단 = 제 치수, 5단 = 맺음, 끝 단 = 다 익혔다.
    /// 3단을 넘기면 지금 고르기 한 줄이 앞에 붙는다. 큰 기술은 2단 휘청·3단 벽 피해.
    #[test]
    fn the_note_tells_what_the_next_rank_gives() {
        let s = s();
        let mut b = Board::default();
        let whirl = Entry::Skill(Id::Whirl);
        assert!(board_note(&b, whirl).contains(&fill(s.rk_damage, &["15"])));
        // 안 배운 스킬: 하는 일 + 3단에서 고를 둘의 이름.
        let quake = board_note(&b, Entry::Skill(Id::Quake));
        assert!(quake.contains(s.skill_quake_d));
        assert!(quake.contains(s.pick_quake_a) && quake.contains(s.pick_quake_b));
        b.rank[Id::Gather.index()] = 1;
        assert!(board_note(&b, Entry::Skill(Id::Gather)).contains(&fill(s.rk_stagger, &["0.3"])));
        b.rank[Id::Whirl.index()] = 2;
        let n = board_note(&b, whirl);
        for t in [
            s.pick_whirl_a,
            s.pick_whirl_a_d,
            s.pick_whirl_b,
            s.pick_whirl_b_d,
        ] {
            assert!(n.contains(t), "{n}");
        }
        b.rank[Id::Whirl.index()] = 3;
        b.pick[Id::Whirl.index()] = Pick::B;
        let n = board_note(&b, whirl);
        assert!(n.contains(s.pick_whirl_b_d) && !n.contains(s.pick_whirl_a_d));
        assert!(n.contains(&fill(s.rk_radius, &["20"])), "{n}");
        b.rank[Id::Whirl.index()] = 4;
        let n = board_note(&b, whirl);
        assert!(n.contains(s.cap_whirl) && n.contains(&fill(s.rk_cooldown, &["10"])));
        assert!(n.contains(&fill(s.rk_damage_more, &["15"])), "{n}");
        b.rank[Id::Whirl.index()] = 5;
        assert!(board_note(&b, whirl).contains(s.board_full));
        // 끌어모으기의 5단에는 피해 글이 없다.
        b.rank[Id::Gather.index()] = 4;
        let n = board_note(&b, Entry::Skill(Id::Gather));
        assert!(n.contains(s.cap_gather) && !n.contains(&fill(s.rk_damage_more, &["15"])));
        // 치수의 이름은 모양에서.
        for (id, t) in [
            (Id::Cut, s.rk_length),
            (Id::Gather, s.rk_range),
            (Id::Quake, s.rk_arc),
        ] {
            assert!(rank_gives(id, 4).contains(&fill(t, &["20"])), "{id:?}");
        }
        assert!(board_note(&b, Entry::Ult).contains(&fill(s.rk_stagger, &["0.6"])));
        b.ult = 2;
        assert!(board_note(&b, Entry::Ult).contains(&fill(s.rk_slam, &["50"])));
        b.ult = 3;
        assert_eq!(board_note(&b, Entry::Ult), s.board_full);
    }
}
