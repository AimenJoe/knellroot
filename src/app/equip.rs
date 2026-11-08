//! 장비 창(0067 안 D — 디아블로 콘솔식 인형): 가운데 방랑자 인형(든 무기를 든 3D, `Renderer::draw_doll`), 왼쪽 무기 칸,
//! 오른쪽 장신구 칸 셋. 칸을 먼저 고르고(십자는 칸 자리대로 커서만, 빨리 바꾸기는 땅 돌리기 버튼 — 2026-11-06) 확인하면 그 칸의 후보 목록
//! (`Scene::EquipPick`, 스택 위 칸 — 닫으면 그 칸으로)에서 고른다. 오른쪽 칸은 상태(`stats::lines`), 후보에 커서가 있으면
//! 바뀌는 줄만 `값 → 값 ▲▼`(색 + 화살표 + 숫자). 아래 왼쪽은 고른 것의 설명.
use super::menu::Hit;
use super::*;
use crate::charms::{self, Id as Charm};
use crate::stats::{Change, Line, Stats};

/// 칸: 0 = 무기, 1..=3 = 장신구 칸.
pub(super) const SLOTS: usize = 1 + charms::SLOTS;

/// 칸 커서의 공간 이동: `up`(+1 = 위)은 장신구 칸 셋 안에서 돌고(무기 칸에서는 그대로), `right`(+1 = 오른쪽)는
/// 무기 칸 → 가운데 장신구 칸(무기 상자와 같은 높이), 장신구 칸 → 무기 칸.
pub(super) fn slot_nav(cur: usize, up: i32, right: i32) -> usize {
    let mut c = cur;
    if right > 0 && c == 0 {
        c = 2;
    } else if right < 0 && c > 0 {
        c = 0;
    }
    if up != 0 && c > 0 {
        let n = (SLOTS - 1) as i32;
        c = 1 + ((c as i32 - 1 - up).rem_euclid(n)) as usize;
    }
    c
}

/// 칸의 후보.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Cand {
    Weapon(weapons::Id),
    /// 장신구 칸 비우기.
    Empty,
    Charm(Charm),
}

/// 칸의 후보 목록(표 순서): 무기 칸 = 맨손 + 무기 넷, 장신구 칸 = 빈 칸 + **가진** 장신구만.
/// 못 얻은 것은 장비 창에 보이지 않는다 — 검은 실루엣 같은 "찾을 것" 표시는 도감 몫(사장 2026-11-05, 0068).
/// 그 칸에 **지금 낀 것도** 목록에 있고("착용 중") 목록을 열면 커서가 거기서 시작한다(`pick_start`) — 사장 2026-11-07
/// "오른쪽 목록에서 장착된 장비에 커서가 가 있는 상태여야". 0070 8 가(낀 것 빼기)를 걷음. 비우기(맨손·빈 칸)는 맨 위.
pub(super) fn candidates(slot: usize, owned: &[Charm], arms: &[weapons::Id]) -> Vec<Cand> {
    if slot == 0 {
        // 무기 칸 = 맨손 + **가진** 무기만(0117 — 무기는 얻어서 쓴다).
        return std::iter::once(weapons::Id::Fists)
            .chain(
                weapons::IDS
                    .into_iter()
                    .filter(|&w| w != weapons::Id::Fists && arms.contains(&w)),
            )
            .map(Cand::Weapon)
            .collect();
    }
    std::iter::once(Cand::Empty)
        .chain(
            charms::TABLE
                .iter()
                .filter(|d| owned.contains(&d.id))
                .map(|d| Cand::Charm(d.id)),
        )
        .collect()
}

/// 바꿀 것이 있나(목록에 지금 것 말고도 있나) — 없으면 목록을 열지 않는다(설명 칸이 얻는 곳을 말한다).
pub(super) fn has_choice(list: &[Cand], now: Cand) -> bool {
    list.iter().any(|&c| c != now)
}

/// 무언가를 낀 후보인가(빈 칸·맨손은 아니다 — "착용 중" 표시를 붙이지 않는다).
pub(super) fn wears_something(c: Cand) -> bool {
    !matches!(c, Cand::Empty | Cand::Weapon(weapons::Id::Fists))
}

/// 후보 목록을 열 때 커서 자리: 지금 든 것(목록에 있으면), 아니면 맨 위.
pub(super) fn pick_start(list: &[Cand], now: Cand) -> usize {
    list.iter().position(|&c| c == now).unwrap_or(0)
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
pub(super) fn stat_name(l: Line) -> &'static str {
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
        Line::StaminaMax => s.stat_stamina,
        Line::StaminaRegen => s.stat_regen,
        Line::Poise => s.stat_poise,
        Line::HeavyDmg => s.stat_heavy,
        Line::LightDmg => s.stat_light,
    }
}

pub(super) fn stat_value(l: Line, v: f32) -> String {
    let s = s();
    let int = || (v.round() as i32).to_string();
    let pct = || format!("{}%", (v * 100.0).round() as i32);
    match l {
        Line::MaxHp => int(),
        Line::Flask => fill(s.stat_sips, &[&int()]),
        Line::Heal
        | Line::Dmg
        | Line::Taken
        | Line::KillStamina
        | Line::StaminaMax
        | Line::HeavyDmg
        | Line::LightDmg => format!("+{}", int()),
        Line::AtkSpeed | Line::DodgeCost | Line::Ember | Line::StaminaRegen | Line::Poise => pct(),
        // 0.06 → "0.06초", 0 → "0초"(뒤 0은 뺀다 — 미리보기 줄이 짧게).
        Line::Parry => {
            let n = format!("{v:.2}");
            let n = n.trim_end_matches('0').trim_end_matches('.');
            format!("+{}", fill(s.value_secs, &[n]))
        }
    }
}

/// 무기 숫자 한 줄(2026-11-07): 무기의 "모양"을 숫자로 — 무기 표 + 랩 기준값(장신구 공속까지 휜 사본) + 장신구 피해에서 계산.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum WLine {
    /// 콤보 타마다 피해 "1·1·2".
    Damage,
    /// 첫 타 한 스윙 시간(초) — 곧 그 무기의 박자.
    Speed,
    Reach,
    /// 강타 피해: 덜 모음 / 다 모음.
    Heavy,
    Shot,
    Mag,
    Reload,
    Grenade,
}

impl WLine {
    fn lower_is_better(self) -> bool {
        matches!(self, WLine::Speed | WLine::Reload)
    }

    fn name(self) -> &'static str {
        let s = s();
        match self {
            WLine::Damage => s.wstat_damage,
            WLine::Speed => s.wstat_speed,
            WLine::Reach => s.wstat_reach,
            WLine::Heavy => s.wstat_heavy,
            WLine::Shot => s.wstat_shot,
            WLine::Mag => s.wstat_mag,
            WLine::Reload => s.wstat_reload,
            WLine::Grenade => s.wstat_grenade,
        }
    }
}

/// 무기 숫자 한 줄의 값: 보이는 글과 견줄 수(클수록 좋다 — 작을수록 좋은 줄은 `WLine`이 뒤집는다).
#[derive(Clone, PartialEq, Debug)]
pub(super) struct WVal {
    pub line: WLine,
    pub text: String,
    pub score: f32,
}

/// 무기의 숫자들. `t` = 장신구까지 휜 랩 값(`Stats::tune`), `st` = 장신구·성장의 피해 +
/// (장신구 `dmg`는 모든 타, 레벨 힘은 평타 `light_dmg`·강타 `heavy_dmg`만 — 월드 `apply_strike`와 같다, 0104).
pub(super) fn weapon_numbers(
    w: &weapons::Weapon,
    t: &Tuning,
    st: &crate::stats::Stats,
) -> Vec<WVal> {
    use crate::tuning::P;
    let (dmg, light, heavy) = (st.dmg, st.dmg + st.light_dmg, st.dmg + st.heavy_dmg);
    let secs = |v: f32| fill(s().value_secs, &[&format!("{v:.2}")]);
    let mut out = Vec::new();
    if let Some(h0) = w.combo.first() {
        let dmgs: Vec<i32> = w.combo.iter().map(|h| h.damage + light).collect();
        out.push(WVal {
            line: WLine::Damage,
            text: dmgs
                .iter()
                .map(|d| d.to_string())
                .collect::<Vec<_>>()
                .join("·"),
            score: dmgs.iter().sum::<i32>() as f32 / dmgs.len() as f32,
        });
        let (s0, a0, r0) = crate::world::hit_times(t, w, 0);
        out.push(WVal {
            line: WLine::Speed,
            text: secs(s0 + a0 + r0),
            score: s0 + a0 + r0,
        });
        let reach = t[P::AttackRange] * h0.reach;
        out.push(WVal {
            line: WLine::Reach,
            text: format!("{reach:.1}m"),
            score: reach,
        });
        if let Some(h) = w.heavy {
            let (lo, hi) = (
                h.damage + heavy,
                h.damage + heavy + t[P::HeavyFullBonus] as i32,
            );
            out.push(WVal {
                line: WLine::Heavy,
                text: format!("{lo} / {hi}"),
                score: hi as f32,
            });
        }
    }
    if let weapons::Rb::Shot {
        bolt,
        mag,
        reload,
        time,
    } = w.rb
    {
        out.push(WVal {
            line: WLine::Shot,
            text: (bolt.damage + dmg).to_string(),
            score: (bolt.damage + dmg) as f32,
        });
        // 공격 속도 = 한 발의 시간 = 박자(0077 — 근접과 같은 줄).
        let shot = t[P::CastTime] * time;
        out.push(WVal {
            line: WLine::Speed,
            text: secs(shot),
            score: shot,
        });
        out.push(WVal {
            line: WLine::Mag,
            text: mag.to_string(),
            score: mag as f32,
        });
        out.push(WVal {
            line: WLine::Reload,
            text: secs(reload),
            score: reload,
        });
    }
    if let Some(g) = w.grenade {
        out.push(WVal {
            line: WLine::Grenade,
            text: g.count.to_string(),
            score: g.count as f32,
        });
    }
    out
}

/// 지금 무기 `now`와 후보 `then`의 같은 줄끼리 견줌(줄이 다르면 — 근접 ↔ 쇠뇌 — 견주지 않고 후보만).
pub(super) fn weapon_compare(now: &[WVal], then: &[WVal]) -> Vec<(WVal, Option<Change>)> {
    then.iter()
        .map(|b| {
            let ch = now.iter().find(|a| a.line == b.line).map(|a| {
                let d = (b.score - a.score) * if b.line.lower_is_better() { -1.0 } else { 1.0 };
                if d.abs() < 1e-4 {
                    Change::Same
                } else if d > 0.0 {
                    Change::Better
                } else {
                    Change::Worse
                }
            });
            (b.clone(), ch)
        })
        .collect()
}

fn weapon_desc(id: weapons::Id) -> &'static str {
    let s = s();
    match id {
        weapons::Id::Sword => s.weapon_sword_d,
        weapons::Id::Hammer => s.weapon_hammer_d,
        weapons::Id::Staff => s.weapon_staff_d,
        weapons::Id::Crossbow => s.weapon_crossbow_d,
        weapons::Id::Fists => s.weapon_fists_d,
    }
}

/// 칸 후보 → 그림으로 보일 아이템(빈 칸은 없음).
fn item_of(c: Cand) -> Option<crate::item_view::Item> {
    match c {
        Cand::Weapon(w) => {
            Some(crate::item_view::Item::Weapon(w)).filter(|&i| crate::item_view::has_view(i))
        }
        Cand::Charm(id) => Some(crate::item_view::Item::Charm(id)),
        Cand::Empty => None,
    }
}

/// 타깃 한 변 `native` px을 화면 `want` px 안에 넣는 정수 배율(도트가 고르게 — 1 아래면 1).
fn icon_scale(native: u32, want: f32) -> f32 {
    (want / native as f32).floor().max(1.0)
}

/// 넘치는 설명 칸 굴리기(0068): 오른쪽 스틱(위 = +, 초당 `PANE_LINES_PER_SEC`줄 × 기울기) · 휠(칸 = 한 줄) · 끌기(줄 수)를
/// 더해 0..=`max`로. 저절로 흐르지 않는다 — 읽는 속도는 읽는 사람이(WCAG 2.2.2, 사장 2026-11-05 "왜 흘러야해?").
pub(super) fn pane_step(
    scroll: f32,
    stick_y: f32,
    dt: f32,
    wheel: i32,
    drag_lines: f32,
    max: usize,
) -> f32 {
    (scroll - stick_y * PANE_LINES_PER_SEC * dt + wheel as f32 + drag_lines).clamp(0.0, max as f32)
}

const PANE_LINES_PER_SEC: f32 = 6.0;

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
    /// 설명 칸 굴리기 한 틱: 오른쪽 스틱은 늘, 휠·끌기는 `anywhere`이거나 설명 칸 위에서 일어났을 때만(그때는 목록에 넘기지 않는다).
    fn pane_input(&mut self, a: &mut Actions, anywhere: bool) {
        let over = anywhere || a.point.and_then(|p| self.hit_at(p)) == Some(Hit::Pane);
        let (wheel, drag) = if over {
            let wd = (a.wheel, a.drag);
            (a.wheel, a.drag) = (0, 0.0);
            wd
        } else {
            (0, 0.0)
        };
        let line = self.ui.px(1.0) * 0.9 * 1.3;
        // 칸에 넘치는 줄 수는 그림이 정한다 — 여기서는 크게 열어 두고 그림이 자른다.
        self.scroll = pane_step(
            self.scroll,
            a.aim.y,
            TICK_DT,
            wheel,
            drag / line.max(1.0),
            usize::MAX >> 1,
        );
    }

    pub(super) fn tick_equip(&mut self, a: &Actions) {
        let mut a = *a;
        let before = self.cursor;
        // 칸 고르기에는 굴릴 목록이 없다 — 휠·끌기는 어디서든 설명 칸을.
        self.pane_input(&mut a, true);
        let a = &a;
        let tap = a.tap.and_then(|p| self.hit_at(p));
        if a.back || a.pause || a.equip || tap == Some(Hit::Close) {
            self.go(Go::Back);
            return;
        }
        // 십자·스틱은 커서만 옮긴다 — 위아래는 장신구 칸 셋 안, 좌우는 무기 칸 ↔ 장신구 칸(칸이 놓인 자리대로,
        // 사장 2026-11-06 "좌우로는 커서가 움직이는게 더 직관적"). 빨리 바꾸기는 땅 돌리기 버튼(LT·RT / Q·E)으로.
        self.cursor = slot_nav(
            self.cursor,
            a.menu_up as i32 - a.menu_down as i32,
            a.menu_right as i32 - a.menu_left as i32,
        );
        if self.cursor != before {
            self.scroll = 0.0;
        }
        let step = a.turn_right as i32 - a.turn_left as i32;
        if step != 0 {
            match self.cursor {
                0 => self.next_weapon(step),
                i => self.step_charm(i - 1, step),
            }
            self.scroll = 0.0;
        }
        // 터치·마우스(2026-11-06): 다른 칸을 누르면 그 칸을 고르기만 하고(설명·상태가 바뀐다), 고른 칸을 다시 누르면 후보를 연다 —
        // 패드의 "커서 → 확인"과 같은 두 걸음.
        let pick = match tap {
            Some(Hit::Row(i)) => {
                let again = self.cursor == i;
                if !again {
                    self.cursor = i;
                    self.scroll = 0.0;
                }
                again
            }
            _ => a.confirm,
        };
        if pick {
            let slot = self.cursor;
            if !has_choice(
                &candidates(slot, &self.progress.owned, &self.arms()),
                self.slot_now(slot),
            ) {
                // 바꿀 것이 없다(설명 칸이 얻는 곳을 말한다).
                self.snd(crate::sfx::Id::Deny);
                return;
            }
            self.go(Go::Open(Scene::EquipPick { slot: slot as u8 }));
            self.cursor = pick_start(
                &candidates(slot, &self.progress.owned, &self.arms()),
                self.slot_now(slot),
            );
        }
    }

    /// 후보 목록 한 틱: 위아래·굴리기, 확인·탭 = 끼우고 칸으로(못 얻은 것은 "안 됨"), 뒤로 = 그냥 칸으로.
    pub(super) fn tick_equip_pick(&mut self, a: &Actions, slot: usize) {
        let mut a = *a;
        let before = self.cursor;
        // 휠·끌기가 설명 칸 위면 설명을, 아니면 후보 목록을 굴린다.
        self.pane_input(&mut a, false);
        let a = &a;
        let tap = a.tap.and_then(|p| self.hit_at(p));
        if a.back || a.pause || tap == Some(Hit::Close) {
            self.go(Go::Back);
            return;
        }
        // 칸 상자를 눌렀다(목록이 열린 채): 같은 칸이면 닫고, 다른 칸이면 그 칸을 고르고 그 칸의 목록을(바꿀 것이 있으면).
        if let Some(Hit::Cell(0, other)) = tap {
            self.go(Go::Back);
            self.cursor = other;
            self.scroll = 0.0;
            if other != slot
                && has_choice(
                    &candidates(other, &self.progress.owned, &self.arms()),
                    self.slot_now(other),
                )
            {
                self.go(Go::Open(Scene::EquipPick { slot: other as u8 }));
                self.cursor = pick_start(
                    &candidates(other, &self.progress.owned, &self.arms()),
                    self.slot_now(other),
                );
            }
            return;
        }
        let list = candidates(slot, &self.progress.owned, &self.arms());
        if !has_choice(&list, self.slot_now(slot)) {
            self.go(Go::Back);
            return;
        }
        // 후보도 두 걸음: 다른 줄을 누르면 고르기만(바뀌는 값 ▲▼를 미리 본다), 고른 줄을 다시 누르면 끼운다.
        let mut a = *a;
        if let Some(Hit::Row(i)) = tap {
            if i != self.cursor && i < list.len() {
                self.cursor = i;
                self.view.tapped(i);
                self.scroll = 0.0;
                a.tap = None;
            }
        }
        let a = &a;
        let picked = self.menu_pick(a, list.len());
        if self.cursor != before {
            self.scroll = 0.0;
        }
        let Some(i) = picked else {
            return;
        };
        match list[i] {
            // 지금 낀 것을 다시 고르면 그대로 닫는다(기록도 그대로).
            c if c == self.slot_now(slot) => {}
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
            // 후보 목록이 열려 있어도 칸을 누를 수 있다: 같은 칸 = 목록 닫기, 다른 칸 = 그 칸의 목록으로(`tick_equip_pick`).
            self.hits.push((
                *r,
                if pick.is_none() {
                    Hit::Row(i)
                } else {
                    Hit::Cell(0, i)
                },
            ));
        }

        // 아래 왼쪽: 고른 것(후보 목록이면 그 후보) — 도는 그림 + 이름(머리글) + 설명.
        let list = pick.map(|slot| candidates(slot, &self.progress.owned, &self.arms()));
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
        // 설명은 칸에 들어가는 줄만큼 보이고, 넘치면 굴린다(오른쪽 스틱·휠·끌기 — `pane_input`) + ▲▼ 표시.
        // 먼저 칸 폭 그대로 나누고, 넘칠 때만 오른쪽에 ▲▼ 자리를 비워 다시 나눈다(넘치지 않는 글은 폭을 다 쓴다).
        let line_h = px * 1.3;
        let body_y = desc_y + px * 1.5;
        let fit = ((bottom - body_y - px) / line_h + 1.0).floor().max(1.0) as usize;
        let mut lines = ui::wrap(&self.ui, self.cand_desc(focus), text_w, 0.9);
        if lines.len() > fit {
            let arrow_w = self.ui.width("▼", 0.8) + u * 3.0;
            lines = ui::wrap(&self.ui, self.cand_desc(focus), text_w - arrow_w, 0.9);
        }
        let max = lines.len().saturating_sub(fit);
        self.scroll = self.scroll.clamp(0.0, max as f32);
        let first = self.scroll.round() as usize;
        for (k, line) in lines.iter().skip(first).take(fit).enumerate() {
            self.ui
                .text(line, text_x, body_y + line_h * k as f32, 0.9, DIM);
        }
        if max > 0 {
            let ax = text_x + text_w - self.ui.width("▼", 0.8);
            if first > 0 {
                self.ui.text("▲", ax, body_y, 0.8, DIM);
            }
            if first < max {
                self.ui
                    .text("▼", ax, body_y + line_h * (fit as f32 - 1.0), 0.8, DIM);
            }
        }
        self.hits.push((
            Rect::new(text_x, desc_y, text_w, bottom - desc_y),
            Hit::Pane,
        ));

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
        // 무기 숫자 한 줄(2026-11-07): 이름 · 값(견주면 "값 → 값 ▲▼").
        let tuned = now_stats.tune(&self.t);
        let num_row = |this: &Self, y: f32, now: Option<&WVal>, v: &WVal, ch: Option<Change>| {
            let (val, col) = match (now, ch) {
                (Some(a), Some(c)) if c != Change::Same => {
                    let (mark, col) = if c == Change::Better {
                        ("▲", AMBER)
                    } else {
                        ("▼", WORSE)
                    };
                    (format!("{} → {} {mark}", a.text, v.text), col)
                }
                _ => (v.text.clone(), INK),
            };
            let w = this.ui.width(&val, 0.9);
            this.ui.text(&val, rx + rw - w, y, 0.9, col);
            let name = this.ui.fit(v.line.name(), rw - w - this.ui.u(6.0), 0.9);
            this.ui.text(&name, rx, y, 0.9, DIM);
        };
        let numbers = |id: weapons::Id| weapon_numbers(id.def(), &tuned, &now_stats);
        match (pick, list) {
            (Some(slot), Some(list)) => {
                let head = if slot == 0 {
                    s.menu_weapon.to_string()
                } else {
                    fill(s.menu_charm_slot, &[&slot.to_string()])
                };
                self.ui.text(&head, rx, top, 1.0, AMBER);
                let here = self.slot_now(slot);
                let rows: Vec<ui::Row> = list
                    .iter()
                    .map(|&c| {
                        // "착용 중"은 낀 물건에만(사장 2026-11-07, 0085) — 빈 칸·맨손은 "아무것도 안 낌"이라 커서만 거기 둔다.
                        let mark = match c {
                            _ if c == here && wears_something(c) => s.equip_worn.to_string(),
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
                // 목록 칸은 후보 수만큼(다섯까지 — 넘치면 굴린다), 남는 자리는 아래 견주는 줄에. 무기 칸은 후보 가운데 가장 긴 숫자 줄 수만큼
                // 비운다 — 커서를 옮겨도 목록이 들썩이지 않게(0077: 쇠뇌 다섯 줄·검 넷째 줄 "강타"가 잘렸다).
                let need = if slot == 0 {
                    list.iter()
                        .filter_map(|c| match c {
                            Cand::Weapon(w) => Some(numbers(*w).len()),
                            _ => None,
                        })
                        .max()
                        .unwrap_or(0) as f32
                        * row_h
                        + u * 8.0
                } else {
                    px * 4.5
                };
                let list_h = (px * 1.75 * list.len().clamp(1, 5) as f32)
                    .min(bottom - list_top - need)
                    .max(px * 1.75);
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
                // 무기 칸: 고른 후보의 숫자를 지금 무기와 견준다.
                if let (0, Some(Cand::Weapon(cand))) = (slot, list.get(self.cursor).copied()) {
                    let now = numbers(self.weapon);
                    for (v, ch) in weapon_compare(&now, &numbers(cand)) {
                        if y + px > bottom {
                            break;
                        }
                        let a = now.iter().find(|a| a.line == v.line);
                        num_row(self, y, a, &v, ch);
                        y += row_h;
                    }
                }
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
                let mut y = top;
                // 무기 칸에 커서가 있으면 그 무기의 숫자를 먼저(2026-11-07).
                if slot_cursor == 0 {
                    self.ui.text(s.menu_weapon, rx, y, 1.0, AMBER);
                    y += px * 1.6;
                    for v in numbers(self.weapon) {
                        num_row(self, y, None, &v, None);
                        y += row_h;
                    }
                    y += px * 0.5;
                }
                self.ui.text(s.equip_status, rx, y, 1.0, AMBER);
                y += px * 1.6;
                // 장비 창의 상태 = 장신구가 바꾸는 줄(+ 늘 보이는 줄) — 레벨·핵 몫은 값에는 들어가되 줄을 늘리지 않는다
                // (레벨 화면이 보인다, 0104 — 힘 5단이면 줄이 일곱이라 창 밑에서 잘렸다).
                let bare = self.progress.stats_without_charms();
                for (l, v, _) in crate::stats::lines_from(&bare, &now_stats, None) {
                    if y + px > bottom {
                        break;
                    }
                    line_row(self, y, l, v, None);
                    y += row_h;
                }
            }
        }
        self.push_outside(panel);
        let mut acts = vec![(Act::Choose, s.verb_choose), (Act::Confirm, s.verb_confirm)];
        if pick.is_none() {
            acts.push((Act::Swap, s.verb_swap));
        }
        self.hint_line(
            &acts,
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
    fn weapon_numbers_read_the_table_and_compare_the_right_way() {
        let t = Tuning::default();
        let none = crate::stats::Stats::default();
        let sword = weapon_numbers(weapons::Id::Sword.def(), &t, &none);
        let hammer = weapon_numbers(weapons::Id::Hammer.def(), &t, &none);
        let line = |v: &[WVal], l: WLine| v.iter().find(|x| x.line == l).unwrap().clone();
        assert_eq!(line(&sword, WLine::Damage).text, "1·1·2");
        assert_eq!(line(&hammer, WLine::Damage).text, "2·3");
        assert!(line(&hammer, WLine::Speed).score > line(&sword, WLine::Speed).score);
        let cmp = weapon_compare(&sword, &hammer);
        let ch = |l: WLine| cmp.iter().find(|(v, _)| v.line == l).unwrap().1;
        assert_eq!(
            ch(WLine::Damage),
            Some(Change::Better),
            "more damage per hit"
        );
        assert_eq!(
            ch(WLine::Speed),
            Some(Change::Worse),
            "a slower swing is worse"
        );
        // 장신구 피해 +1은 모든 타에.
        assert_eq!(
            line(
                &weapon_numbers(
                    weapons::Id::Sword.def(),
                    &t,
                    &crate::stats::Stats::compute(&[crate::stats::Mods {
                        dmg: 1,
                        ..Default::default()
                    }]),
                ),
                WLine::Damage
            )
            .text,
            "2·2·3"
        );
        // 쇠뇌는 사격 줄 — 근접 줄과는 견주지 않는다. 공격 속도(= 박자, 0077)만은 둘 다 있어 견준다.
        let xbow = weapon_numbers(weapons::Id::Crossbow.def(), &t, &none);
        assert!(xbow.iter().all(|v| v.line != WLine::Damage));
        assert!(weapon_compare(&sword, &xbow)
            .iter()
            .all(|(v, c)| c.is_none() == (v.line != WLine::Speed)));
    }

    #[test]
    fn the_slot_cursor_moves_by_where_the_boxes_sit() {
        // 무기(0)는 왼쪽, 장신구 1..=3은 오른쪽 위에서 아래로.
        assert_eq!(
            slot_nav(0, 0, 1),
            2,
            "right from the weapon lands level with it"
        );
        assert_eq!(slot_nav(3, 0, -1), 0);
        assert_eq!(slot_nav(1, 0, 1), 1, "nothing further right");
        assert_eq!(slot_nav(0, 0, -1), 0);
        assert_eq!(slot_nav(1, -1, 0), 2, "down");
        assert_eq!(slot_nav(3, -1, 0), 1, "down wraps in the charm column");
        assert_eq!(slot_nav(1, 1, 0), 3, "up wraps");
        assert_eq!(slot_nav(0, 1, 0), 0, "the weapon column has one box");
    }

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
    fn slots_list_what_you_own_and_the_cursor_opens_on_what_is_worn() {
        // 무기 칸: 맨손이 맨 위, 든 것도 목록에 있고 커서가 거기서 시작한다.
        let held = Cand::Weapon(weapons::Id::Staff);
        let weapons = candidates(0, &[], &weapons::IDS);
        assert_eq!(weapons.len(), weapons::IDS.len());
        assert_eq!(
            weapons[0],
            Cand::Weapon(weapons::Id::Fists),
            "bare hands first"
        );
        assert_eq!(
            weapons[pick_start(&weapons, held)],
            held,
            "the cursor starts on it"
        );
        // 0117: 무기 칸은 가진 것만(맨손·검 + 얻은 것).
        let earned = candidates(0, &[], &[weapons::Id::Sword, weapons::Id::Hammer]);
        assert_eq!(
            earned,
            vec![
                Cand::Weapon(weapons::Id::Fists),
                Cand::Weapon(weapons::Id::Sword),
                Cand::Weapon(weapons::Id::Hammer)
            ]
        );
        // 장신구 칸(사장 2026-11-07): 빈 칸 + 가진 것, 낀 것도 목록에 있고 커서가 거기.
        let owned = [Charm::Ear, Charm::Shell];
        let list = candidates(3, &owned, &[]);
        assert_eq!(
            list,
            vec![
                Cand::Empty,
                Cand::Charm(Charm::Shell),
                Cand::Charm(Charm::Ear)
            ],
            "empty first, then owned in table order"
        );
        assert_eq!(
            list[pick_start(&list, Cand::Charm(Charm::Ear))],
            Cand::Charm(Charm::Ear)
        );
        assert_eq!(list[pick_start(&list, Cand::Empty)], Cand::Empty);
        // 빈 칸·맨손은 커서만(0085) — "착용 중"은 낀 물건에만.
        assert!(
            !wears_something(Cand::Empty) && !wears_something(Cand::Weapon(weapons::Id::Fists))
        );
        assert!(wears_something(Cand::Charm(Charm::Ear)) && wears_something(held));
        // 장신구가 없고 칸이 비었으면 바꿀 것이 없다 — 못 얻은 것은 장비 창에 없다(도감 몫).
        assert!(!has_choice(&candidates(1, &[], &[]), Cand::Empty));
        assert!(has_choice(&list, Cand::Empty));
    }
}

#[cfg(test)]
mod pane_tests {
    use super::*;

    #[test]
    fn the_description_pane_scrolls_only_when_asked_and_stays_in_range() {
        // 손대지 않으면 그대로(저절로 흐르지 않는다).
        assert_eq!(pane_step(0.0, 0.0, 1.0 / 60.0, 0, 0.0, 3), 0.0);
        // 스틱 아래(−y) = 아래 줄로, 1초면 여섯 줄 — 그러나 넘친 줄 수까지만.
        assert_eq!(pane_step(0.0, -1.0, 1.0, 0, 0.0, 3), 3.0);
        assert_eq!(
            pane_step(2.0, 1.0, 1.0, 0, 0.0, 3),
            0.0,
            "up stops at the top"
        );
        // 휠 한 칸 = 한 줄, 끌기는 줄 수.
        assert_eq!(pane_step(0.0, 0.0, 0.0, 1, 0.0, 3), 1.0);
        assert_eq!(pane_step(1.0, 0.0, 0.0, 0, 1.5, 3), 2.5);
        // 넘치지 않으면(0줄) 움직이지 않는다.
        assert_eq!(pane_step(0.0, -1.0, 1.0, 2, 4.0, 0), 0.0);
    }
}
