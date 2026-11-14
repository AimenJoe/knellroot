//! 큰 창의 장비 탭(0067 안 D — 디아블로 콘솔식 인형 → 0164 탭): 왼쪽 = 방랑자 인형(든 무기를 든 3D, `Renderer::draw_doll`)과
//! 칸 다섯(인형 왼쪽 위 = 무기·아래 = 덧옷 0176, 장신구 셋은 오른쪽), 오른쪽 = 고른 것의 상세(그림 · 이름 · 숫자 · 설명 · 상태 `stats::lines`).
//! 칸을 먼저 고르고(십자는 칸 자리대로 커서만, 빨리 바꾸기는 공격 버튼 — 스킬 버튼 둘은 탭을 넘긴다, 0164) 확인하면 그 칸 곁에
//! 후보의 작은 목록(`Scene::EquipPick`, 스택 위 칸 — 닫으면 그 칸으로)이 뜬다. 후보에 커서가 있으면 오른쪽이 지금 것과 견준다 —
//! 바뀌는 줄만 `값 → 값 ▲▼`(색 + 화살표 + 숫자).
use super::frame::Window;
use super::menu::Hit;
use super::tabs::{Ln, PaneHead, HEAD_H};
use super::*;
use crate::charms::{self, Id as Charm};
use crate::stats::{Change, Line, Stats};

/// 칸: 0 = 무기, 1..=3 = 장신구 칸, 4 = 덧옷(0176).
pub(super) const SLOTS: usize = 2 + charms::SLOTS;
pub(super) const GARB_SLOT: usize = 1 + charms::SLOTS;

/// 인형 왼쪽 열의 칸(무기·덧옷 — 후보 목록이 오른쪽에 뜬다).
fn left_column(slot: usize) -> bool {
    slot == 0 || slot == GARB_SLOT
}

/// 칸 상자의 자리(480×270 기준 px, 왼쪽 칸 안): 무기 상자·장신구 상자의 한 변, 장신구 상자 사이, 칸의 가장자리에서 들인 몫.
const WEAPON_BOX: f32 = 40.0;
const CHARM_BOX: f32 = 34.0;
const BOX_GAP: f32 = 6.0;
const BOX_INSET: f32 = 8.0;
const BOXES_H: f32 = CHARM_BOX * 3.0 + BOX_GAP * 2.0;
/// 후보의 작은 목록: 폭(적어도 — 긴 이름이면 넓어진다), 한 번에 보이는 줄(넘치면 굴린다).
const PICK_W: f32 = 170.0;
const PICK_ROWS: usize = 6;

/// 칸 커서의 공간 이동(칸이 놓인 자리대로): 왼쪽 열 = 무기(위)·덧옷(아래), 오른쪽 열 = 장신구 셋. `up`(+1 = 위)은 그 열 안에서
/// 돌고, `right`(+1 = 오른쪽)는 같은 높이 쪽으로 — 무기 ↔ 맨 위 장신구, 덧옷 ↔ 맨 아래 장신구, 가운데 장신구 → 무기.
pub(super) fn slot_nav(cur: usize, up: i32, right: i32) -> usize {
    let mut c = cur;
    if right > 0 && left_column(c) {
        c = if c == 0 { 1 } else { charms::SLOTS };
    } else if right < 0 && !left_column(c) {
        c = if c == charms::SLOTS { GARB_SLOT } else { 0 };
    }
    if up != 0 {
        if left_column(c) {
            c = if c == 0 { GARB_SLOT } else { 0 };
        } else {
            let n = charms::SLOTS as i32;
            c = 1 + ((c as i32 - 1 - up).rem_euclid(n)) as usize;
        }
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
    /// 덧옷 칸 비우기(0176).
    Bare,
    Garb(crate::garb::Id),
}

/// 칸의 후보 목록(표 순서): 무기 칸 = 맨손 + 무기 넷, 장신구 칸 = 빈 칸 + **가진** 장신구만.
/// 못 얻은 것은 장비 창에 보이지 않는다 — 검은 실루엣 같은 "찾을 것" 표시는 도감 몫(사장 2026-11-05, 0068).
/// 그 칸에 **지금 낀 것도** 목록에 있고("착용 중") 목록을 열면 커서가 거기서 시작한다(`pick_start`) — 사장 2026-11-07
/// "오른쪽 목록에서 장착된 장비에 커서가 가 있는 상태여야". 0070 8 가(낀 것 빼기)를 걷음. 비우기(맨손·빈 칸)는 맨 위.
pub(super) fn candidates(
    slot: usize,
    owned: &[Charm],
    arms: &[weapons::Id],
    garbs: &[crate::garb::Id],
) -> Vec<Cand> {
    if slot == GARB_SLOT {
        // 덧옷 칸 = 벗기 + 가진 덧옷(0176).
        return std::iter::once(Cand::Bare)
            .chain(garbs.iter().map(|&g| Cand::Garb(g)))
            .collect();
    }
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
    !matches!(
        c,
        Cand::Empty | Cand::Bare | Cand::Weapon(weapons::Id::Fists)
    )
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

/// 칸 `slot`에 낄 수 있는 다음(`step` ±1) 장신구: 빈 칸 + 가진 것 가운데 다른 칸에 없는 것(표 순서), 끝에서 감는다.
pub(super) fn next_charm(
    owned: &[Charm],
    worn: &[Option<Charm>; charms::SLOTS],
    slot: usize,
    step: i32,
) -> Option<crate::charms::Id> {
    let mut opts: Vec<Option<crate::charms::Id>> = vec![None];
    for d in crate::charms::TABLE.iter() {
        let elsewhere = worn
            .iter()
            .enumerate()
            .any(|(j, w)| j != slot && *w == Some(d.id));
        if owned.contains(&d.id) && !elsewhere {
            opts.push(Some(d.id));
        }
    }
    let at = opts.iter().position(|o| *o == worn[slot]).unwrap_or(0) as i32;
    opts[(at + step).rem_euclid(opts.len() as i32) as usize]
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
        Line::OpenedDmg => s.stat_opened,
        Line::Knock => s.stat_knock,
        Line::Flinch => s.stat_flinch,
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
        | Line::LightDmg
        | Line::OpenedDmg => format!("+{}", int()),
        Line::AtkSpeed
        | Line::DodgeCost
        | Line::Ember
        | Line::StaminaRegen
        | Line::Poise
        | Line::Knock
        | Line::Flinch => pct(),
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
        let shot_dmg = bolt.damage + dmg + st.shot_dmg;
        out.push(WVal {
            line: WLine::Shot,
            text: shot_dmg.to_string(),
            score: shot_dmg as f32,
        });
        // 공격 속도 = 한 발의 시간 = 박자(0077 — 근접과 같은 줄).
        let shot = t[P::CastTime] * time;
        out.push(WVal {
            line: WLine::Speed,
            text: secs(shot),
            score: shot,
        });
        // 벼린 단의 탄창(0180).
        let mag = (mag as i32 + st.mag).max(1);
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

/// 장신구의 세계 안의 말(0129): 설명보다 옅고 조금 푸르스름하게 — 수치와 섞여 읽히지 않게.
pub(super) const FLAVOR: Color = Color::new(0.55, 0.56, 0.62, 1.0);

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
        Cand::Garb(id) => Some(crate::item_view::Item::Garb(id)),
        Cand::Empty | Cand::Bare => None,
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
            GARB_SLOT => self.progress.garb.map_or(Cand::Bare, Cand::Garb),
            i => self.progress.worn[i - 1].map_or(Cand::Empty, Cand::Charm),
        }
    }

    /// 가진 덧옷(0176 — 주운 숨은 물건에서).
    pub(super) fn garbs(&self) -> Vec<crate::garb::Id> {
        crate::garb::owned(&self.progress.finds)
    }

    /// 칸의 후보(가진 것에서).
    fn cands(&self, slot: usize) -> Vec<Cand> {
        candidates(slot, &self.progress.owned, &self.arms(), &self.garbs())
    }

    /// 줄·머리에 쓰는 이름: 무기는 벼린 단이 붙는다(0180 "검 +2").
    fn cand_label(&self, c: Cand) -> String {
        match c {
            Cand::Weapon(w) => self.forged_name(w),
            _ => self.cand_name(c).to_string(),
        }
    }

    fn cand_name(&self, c: Cand) -> &'static str {
        match c {
            Cand::Weapon(w) => weapon_name(w),
            Cand::Empty => s().charm_empty,
            Cand::Charm(id) => tr(charms::def(id).name),
            Cand::Bare => s().garb_none,
            Cand::Garb(id) => tr(crate::garb::def(id).name),
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
            Cand::Bare if self.garbs().is_empty() => s.garb_none_yet,
            Cand::Bare => s.garb_none_d,
            Cand::Garb(id) => tr(crate::garb::def(id).desc),
        }
    }

    /// 칸 `slot`에 `c`를 끼우면 능력치(무기는 능력치를 바꾸지 않는다).
    fn stats_with(&self, slot: usize, c: Cand) -> Stats {
        let mut p = self.progress.clone();
        if slot == GARB_SLOT {
            p.garb = match c {
                Cand::Garb(id) => Some(id),
                _ => None,
            };
        } else if slot > 0 {
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

    /// 장비 탭 한 틱: 십자 = 칸(자리대로), 공격 버튼 = 바로 다음 것으로 돌려 끼우기, 확인·고른 칸 다시 누르기 = 후보 목록,
    /// 뒤로·Select·✕·창 밖 = 닫기.
    pub(super) fn tick_equip(&mut self, a: &Actions) {
        let mut a = *a;
        let before = self.cursor;
        // 칸 고르기에는 굴릴 목록이 없다 — 휠·끌기는 어디서든 오른쪽 상세 칸을.
        if a.wheel != 0 || a.drag != 0.0 {
            let line = self.ui.u(tabs::LH_BODY).max(1.0);
            self.scroll = pane_step(
                self.scroll,
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
        if a.back || a.pause || a.equip || tap == Some(Hit::Close) {
            self.go(Go::Back);
            return;
        }
        // 십자·스틱은 커서만 옮긴다 — 위아래는 장신구 칸 셋 안, 좌우는 무기 칸 ↔ 장신구 칸(칸이 놓인 자리대로,
        // 사장 2026-11-06 "좌우로는 커서가 움직이는게 더 직관적"). 빨리 바꾸기는 공격 버튼 하나(다음 것으로 돈다, 0164 — 스킬 버튼
        // 둘은 탭을 넘긴다). 그 버튼이 메뉴의 확인·누르기와 겹치는 장치(키보드 J·마우스 클릭)에서는 꺼진다 — `input::swap_free`.
        self.cursor = slot_nav(
            self.cursor,
            a.menu_up as i32 - a.menu_down as i32,
            a.menu_right as i32 - a.menu_left as i32,
        );
        if self.cursor != before {
            self.scroll = 0.0;
        }
        if a.attack && !a.confirm && input::swap_free() {
            match self.cursor {
                0 => self.next_weapon(1),
                GARB_SLOT => self.step_garb(1),
                i => self.step_charm(i - 1, 1),
            }
            self.scroll = 0.0;
            self.snd(crate::sfx::Id::UiMove);
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
            if !has_choice(&self.cands(slot), self.slot_now(slot)) {
                // 바꿀 것이 없다(설명 칸이 얻는 곳을 말한다).
                self.snd(crate::sfx::Id::Deny);
                return;
            }
            self.go(Go::Open(Scene::EquipPick { slot: slot as u8 }));
            self.cursor = pick_start(&self.cands(slot), self.slot_now(slot));
        }
    }

    /// 후보 목록 한 틱: 위아래·굴리기, 확인·탭 = 끼우고 칸으로(못 얻은 것은 "안 됨"), 뒤로 = 그냥 칸으로.
    pub(super) fn tick_equip_pick(&mut self, a: &Actions, slot: usize) {
        let mut a = *a;
        let before = self.cursor;
        // 휠·끌기가 상세 칸 위면 상세를, 아니면 후보 목록을 굴린다.
        self.pane_scroll_lines(&mut a);
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
            if other != slot && has_choice(&self.cands(other), self.slot_now(other)) {
                self.go(Go::Open(Scene::EquipPick { slot: other as u8 }));
                self.cursor = pick_start(&self.cands(other), self.slot_now(other));
            }
            return;
        }
        let list = self.cands(slot);
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
            c if slot == GARB_SLOT => {
                self.progress.garb = match c {
                    Cand::Garb(id) => Some(id),
                    _ => None,
                };
                self.world.apply_stats(self.progress.stats(), false);
                self.persist();
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

    /// 장비 탭 그림(0164): 왼쪽 = 인형과 칸 넷(고르는 것), 오른쪽 = 고른 것의 상세(그림 · 이름 · 숫자 · 설명). `pick`이 있으면
    /// 그 칸 곁에 후보의 작은 목록이 뜨고(커서 = 지금 커서) 오른쪽은 커서가 놓인 후보를 지금 것과 견준다. 칸 커서는 `slot_cursor`.
    pub(super) fn draw_equip(
        &mut self,
        f: &Frame,
        host: flow::Host,
        slot_cursor: usize,
        pick: Option<usize>,
    ) {
        let s = s();
        let scene = Scene::Equip { host };
        let w = self.draw_tab_window(f, scene);
        let u = w.s;

        // 칸 상자(0068 — 디아블로 인형처럼 칸에는 아이템 그림): 인형 왼쪽 위 = 무기·아래 = 덧옷(0176), 장신구 셋은 오른쪽.
        // 인형은 그 사이 가운데.
        let boxes_top = (w.body_h - BOXES_H) * 0.5;
        let mut boxes = [Rect::default(); SLOTS];
        boxes[0] = w.rect(BOX_INSET, boxes_top, WEAPON_BOX, WEAPON_BOX);
        boxes[GARB_SLOT] = w.rect(
            BOX_INSET,
            boxes_top + BOXES_H - WEAPON_BOX,
            WEAPON_BOX,
            WEAPON_BOX,
        );
        for (i, b) in boxes.iter_mut().enumerate().skip(1).take(charms::SLOTS) {
            *b = w.rect(
                tabs::LEFT_W - CHARM_BOX - BOX_INSET,
                boxes_top + (CHARM_BOX + BOX_GAP) * (i - 1) as f32,
                CHARM_BOX,
                CHARM_BOX,
            );
        }
        // 인형: 정수 배율(게임 화면과 같은 도트 굵기)로 몸의 높이·두 칸 사이에 들어가는 가장 큰 것.
        let room_x = boxes[0].x + boxes[0].w + u * 4.0;
        let room_w = boxes[1].x - u * 4.0 - room_x;
        let m = (w.body_h * u / crate::render::DOLL_H as f32)
            .min(room_w / crate::render::DOLL_W as f32)
            .floor()
            .max(1.0);
        let (dw, dh) = (
            crate::render::DOLL_W as f32 * m,
            crate::render::DOLL_H as f32 * m,
        );
        let doll = Rect::new(
            (room_x + (room_w - dw) * 0.5).round(),
            (w.origin.y + (w.body_h * u - dh) * 0.5).round(),
            dw,
            dh,
        );
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

        // 오른쪽: 고른 것(후보 목록이 열렸으면 그 후보) — 머리(도는 그림 · 이름 · 칸 · 낀 곳) + 숫자·바뀌는 줄 + 설명.
        let list = pick.map(|slot| self.cands(slot));
        let slot = pick.unwrap_or(slot_cursor);
        let here = self.slot_now(slot);
        let focus = match &list {
            Some(l) => l.get(self.cursor).copied().unwrap_or(Cand::Empty),
            None => here,
        };
        let slot_name = |slot: usize| {
            if slot == 0 {
                s.menu_weapon.to_string()
            } else if slot == GARB_SLOT {
                s.menu_garb.to_string()
            } else {
                fill(s.menu_charm_slot, &[&slot.to_string()])
            }
        };
        let state = match (pick, focus) {
            (Some(_), c) if c == here && wears_something(c) => (s.equip_worn.to_string(), AMBER),
            (Some(_), Cand::Charm(id)) => (
                self.progress
                    .worn
                    .iter()
                    .position(|x| *x == Some(id))
                    .map(|j| slot_name(j + 1))
                    .unwrap_or_default(),
                DIM,
            ),
            _ => (String::new(), DIM),
        };
        let item = item_of(focus);
        let icon = self.draw_pane_head(
            &w,
            &PaneHead {
                name: self.cand_name(focus),
                gold: wears_something(focus),
                // 벼린 단(0180)은 갈래 줄에 — 머리의 이름 칸은 긴 무기 이름으로 이미 찬다.
                kind: match focus {
                    Cand::Weapon(w) if self.progress.forged.rank(w) > 0 => fill(
                        s.forge_named,
                        &[&slot_name(slot), &self.progress.forged.rank(w).to_string()],
                    ),
                    _ => slot_name(slot),
                },
                state,
                icon: item.is_some(),
            },
        );
        if let Some(item) = item {
            self.draw_pane_item(item, icon);
        }
        let lines = self.equip_lines(&w, slot, focus, pick.is_some());
        self.draw_pane(&w, HEAD_H, &lines);

        // 아래 안내: 칸 고르기에는 빨리 바꾸기(공격 버튼 — 쓸 수 있는 장치에서만 뜬다).
        let right = w.panel.x + w.panel.w;
        let mut acts = vec![(Act::Walk, s.verb_choose), (Act::Confirm, s.verb_confirm)];
        if pick.is_none() {
            acts.push((Act::Swap, s.verb_swap));
        }
        self.tab_foot(f, &w, scene, right, &acts);

        // 후보의 작은 목록: 고른 칸 곁(무기 칸은 오른쪽에, 장신구 칸은 왼쪽에 — 인형 위), 몸 안으로.
        if let (Some(slot), Some(list)) = (pick, list) {
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
                            .position(|x| *x == Some(id))
                            .map(|j| slot_name(j + 1))
                            .unwrap_or_default(),
                        _ => String::new(),
                    };
                    ui::Row::value(self.cand_label(c), mark)
                        .with_icon(true)
                        .tagged()
                })
                .collect();
            let px = self.ui.px(w.k);
            let row_h = px * 1.75;
            // 폭: 가장 긴 줄(이름 + 표지)이 잘리지 않게 — 줄의 앞 여백(▶·그림 칸)·뒤 여백은 `ui::list_k`와 같은 셈. 왼쪽 칸을 넘지 않는다.
            let widest = rows
                .iter()
                .map(|r| {
                    let tag = if r.value.is_empty() {
                        0.0
                    } else {
                        self.ui.width(&r.value, 0.8 * w.k) + u * 6.0
                    };
                    self.ui.width(&r.label, w.k) + tag
                })
                .fold(0.0, f32::max);
            let pw = (widest + px * 2.5 + u * 24.0)
                .max(PICK_W * u)
                .min((tabs::LEFT_W - 4.0) * u);
            let ph = row_h * rows.len().clamp(1, PICK_ROWS) as f32 + u * 8.0;
            let b = boxes[slot];
            let x = if left_column(slot) {
                b.x + b.w + u * 5.0
            } else {
                b.x - u * 5.0 - pw
            };
            let x = x.min(w.at(tabs::LEFT_W, 0.0).x - pw).max(w.origin.x);
            let (top, bottom) = (w.origin.y, w.origin.y + w.body_h * u);
            let y = b.y.min(bottom - ph).max(top).round();
            let panel = Rect::new(x.round(), y, pw, ph);
            draw_rectangle(
                panel.x + u * 2.0,
                panel.y + u * 2.0,
                pw,
                ph,
                Color::new(0.01, 0.0, 0.02, 0.7),
            );
            draw_rectangle(
                panel.x,
                panel.y,
                pw,
                ph,
                Color {
                    a: 1.0,
                    ..ui::PANEL
                },
            );
            self.ui.edge(panel, self.ui.u(1.0), AMBER);
            let area = Rect::new(
                panel.x + u * 3.0,
                panel.y + u * 4.0,
                pw - u * 6.0,
                ph - u * 8.0,
            );
            let mut hits = self
                .ui
                .list_k(&rows, self.cursor, &mut self.view, area, w.k);
            // 넘치면 목록의 오른쪽 위·아래 구석 밖에 ▲▼(누를 자리).
            let (up, down) = self.view.more(rows.len());
            let tk = 0.7 * w.k;
            let ax = panel.x + pw + u * 2.0;
            for (n, (on, glyph, ay)) in [
                (up, "▲", panel.y),
                (down, "▼", panel.y + ph - self.ui.px(tk)),
            ]
            .into_iter()
            .enumerate()
            {
                if on {
                    self.ui.text(glyph, ax, ay, tk, AMBER);
                    let g = self.ui.px(tk);
                    hits.arrows[n] = Some(Rect::new(ax - g * 0.5, ay - g * 0.5, g * 2.0, g * 2.0));
                }
            }
            // 줄마다 아이템 그림(빈 칸은 비워 둔다) — 아이콘은 한 번 찍어 둔 것(0068).
            for (r, c) in hits.rows.iter().zip(list.iter()) {
                if let (true, Some(item)) = (r.w > 0.0, item_of(*c)) {
                    let slot_r = self.ui.list_icon(*r, w.k);
                    let side = crate::render::ICON_PX as f32
                        * icon_scale(crate::render::ICON_PX, slot_r.w);
                    self.renderer.draw_icon(
                        item,
                        Rect::new(
                            (slot_r.x + (slot_r.w - side) * 0.5).round(),
                            (slot_r.y + (slot_r.h - side) * 0.5).round(),
                            side,
                            side,
                        ),
                    );
                }
            }
            // 목록의 줄이 아래 화면의 어느 자리보다 먼저 눌린다.
            let mut front = super::menu::hits_of(&hits);
            front.append(&mut self.hits);
            self.hits = front;
        }
    }

    /// 오른쪽 칸의 몸: 칸 `slot`의 `focus`(지금 것·후보) — 무기는 숫자(후보면 지금 무기와 견줘 "값 → 값 ▲▼"), 장신구 후보는
    /// 바뀌는 상태 줄, 그 아래 설명(장신구는 세계 안의 말까지). 칸 고르기에서는 끝에 지금의 상태(장신구가 바꾸는 줄 — 0104).
    fn equip_lines(&self, w: &Window, slot: usize, focus: Cand, picking: bool) -> Vec<Ln> {
        let s = s();
        let now_stats = self.progress.stats();
        // 벼린 단(0180)은 그 무기를 들었을 때만 — 후보마다 그 무기를 든 능력치로 센다.
        let numbers = |id: weapons::Id| {
            let mut p = self.progress.clone();
            p.weapon = Some(id);
            let st = p.stats();
            weapon_numbers(id.def(), &st.tune(&self.t), &st)
        };
        let arrow = |ch: Change| match ch {
            Change::Better => ("▲", AMBER),
            Change::Worse => ("▼", WORSE),
            Change::Same => ("", INK),
        };
        let mut lines: Vec<Ln> = Vec::new();
        if let Cand::Weapon(id) = focus {
            let now = numbers(self.weapon);
            for (v, ch) in weapon_compare(&now, &numbers(id)) {
                let was = now.iter().find(|a| a.line == v.line);
                let (val, col) = match (was, ch.filter(|c| picking && *c != Change::Same)) {
                    (Some(a), Some(c)) => {
                        let (mark, col) = arrow(c);
                        (format!("{} → {} {mark}", a.text, v.text), col)
                    }
                    _ => (v.text.clone(), INK),
                };
                lines.push(Ln::Pair(v.line.name().to_string(), val, col));
            }
        } else if picking {
            let then = self.stats_with(slot, focus);
            let changed: Vec<_> = crate::stats::lines(&now_stats, Some(&then))
                .into_iter()
                .filter_map(|(l, v, t)| t.filter(|(_, c)| *c != Change::Same).map(|t| (l, v, t)))
                .collect();
            if changed.is_empty() {
                lines.push(Ln::Text(s.equip_no_change.to_string(), DIM));
            }
            for (l, v, (t2, ch)) in changed {
                let (mark, col) = arrow(ch);
                let val = format!("{} → {} {mark}", stat_value(l, v), stat_value(l, t2));
                lines.push(Ln::Pair(stat_name(l).to_string(), val, col));
            }
        }
        if !lines.is_empty() {
            lines.push(Ln::Gap);
        }
        lines.extend(self.pane_wrap(w, self.cand_desc(focus), DIM));
        let flavor = match focus {
            Cand::Charm(id) => Some(tr(charms::def(id).flavor)),
            Cand::Garb(id) => Some(tr(crate::garb::def(id).flavor)),
            _ => None,
        };
        if let Some(f) = flavor {
            lines.push(Ln::Gap);
            lines.extend(self.pane_wrap(w, f, FLAVOR));
        }
        if !picking {
            // 상태 = 장신구가 바꾸는 줄(+ 늘 보이는 줄) — 레벨 몫은 값에는 들어가되 줄을 늘리지 않는다(상태 탭이 보인다, 0104).
            lines.push(Ln::Gap);
            lines.push(Ln::Head(s.equip_status.to_string()));
            let bare = self.progress.stats_without_charms();
            for (l, v, _) in crate::stats::lines_from(&bare, &now_stats, None) {
                lines.push(Ln::Pair(stat_name(l).to_string(), stat_value(l, v), INK));
            }
        }
        lines
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
        assert_eq!(line(&sword, WLine::Damage).text, "10·10·20");
        assert_eq!(line(&hammer, WLine::Damage).text, "20·30");
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
        // 장신구 피해 +10은 모든 타에.
        assert_eq!(
            line(
                &weapon_numbers(
                    weapons::Id::Sword.def(),
                    &t,
                    &crate::stats::Stats::compute(&[crate::stats::Mods {
                        dmg: 10,
                        ..Default::default()
                    }]),
                ),
                WLine::Damage
            )
            .text,
            "20·20·30"
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
        // 왼쪽 열 = 무기(0, 위)·덧옷(4, 아래 — 0176), 오른쪽 열 = 장신구 1..=3(위에서 아래로).
        assert_eq!(
            slot_nav(0, 0, 1),
            1,
            "right from the weapon lands level with it"
        );
        assert_eq!(
            slot_nav(GARB_SLOT, 0, 1),
            3,
            "right from the garment lands level with it"
        );
        assert_eq!(slot_nav(3, 0, -1), GARB_SLOT);
        assert_eq!(slot_nav(1, 0, -1), 0);
        assert_eq!(slot_nav(2, 0, -1), 0);
        assert_eq!(slot_nav(1, 0, 1), 1, "nothing further right");
        assert_eq!(slot_nav(0, 0, -1), 0);
        assert_eq!(slot_nav(1, -1, 0), 2, "down");
        assert_eq!(slot_nav(3, -1, 0), 1, "down wraps in the charm column");
        assert_eq!(slot_nav(1, 1, 0), 3, "up wraps");
        assert_eq!(
            slot_nav(0, -1, 0),
            GARB_SLOT,
            "the left column has two boxes"
        );
        assert_eq!(slot_nav(GARB_SLOT, 1, 0), 0);
        // 칸은 다섯 다 닿는다.
        let mut seen = [false; SLOTS];
        let mut c = 0;
        for (up, right) in [(0, 0), (0, 1), (-1, 0), (-1, 0), (0, -1)] {
            c = slot_nav(c, up, right);
            seen[c] = true;
        }
        assert!(seen.iter().all(|&x| x), "{seen:?}");
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
        let weapons = candidates(0, &[], &weapons::IDS, &[]);
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
        let earned = candidates(0, &[], &[weapons::Id::Sword, weapons::Id::Hammer], &[]);
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
        let list = candidates(3, &owned, &[], &[]);
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
        assert!(!has_choice(&candidates(1, &[], &[], &[]), Cand::Empty));
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
