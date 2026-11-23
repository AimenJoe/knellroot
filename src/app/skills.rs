//! 기술 화면(0161 — 옛 나무 창·기술 판·멈춤의 기술 창을 하나로, 0160 시안 1의 방향). 화로와 멈춤이 같은 화면을 연다:
//! 화로(`edit`)에서는 올리기·열기·고르기 바꾸기·되돌리기까지, 멈춤에서는 읽기와 끼우기만.
//!
//! - 왼쪽: 갈래 셋(흐름·맞받기·자리) × 줄 셋(처음부터 / 쓴 점수 4 / 쓴 점수 8)의 **칸 그림**. 스킬 = 네모 + 단의 점 다섯,
//!   큰 기술 = 여덟모 + 마름모 점 셋, 패시브 = 동그라미. 그림은 터치 버튼의 13×13 도트(`touch::skill_icon`·`ult_icon`)와
//!   패시브의 것(`icons::cell_mask`). 밝기 넷(색만으로 말하지 않는다): 익힘 = 금빛 테·채운 면 / 지금 살 수 있음 = 민트 점선 /
//!   못 삼·잠김 = 흐림 / 고른 칸 = 밝은 바깥 테. 끼운 스킬은 칸 번호 표지(1·2·큰), 3단부터는 고르기 표지.
//! - 오른쪽: 고른 칸의 모든 것 — 그림·이름·갈래·상태, 하는 일, 단마다 한 줄(다음 단은 밝은 띠 + 값, 3단은 두 고르기를 나란히).
//!   넘치는 글은 읽는 사람이 굴린다(0068 — 오른쪽 스틱·칸 위 휠/끌기).
//! - 걷기: 십자로 칸의 자리대로(`walk` — 돌지 않는다), 확인 = 그 칸에 **할 수 있는 일의 작은 목록**(`Scene::SkillAct` — 올리기·
//!   열기는 묻는 창을 거친다 / 칸에 끼우기 / 고르기 바꾸기). 되는 일이 없으면 "안 됨" 소리만. 터치는 칸을 눌러 고르고 다시 눌러 연다.
//!
//! 셈은 `skills::Board`(순수), 자리·걷기·목록·글은 여기의 순수 함수(시험), 틀은 `frame.rs`.
use super::frame::Window;
use super::menu::Hit;
use super::*;
use crate::progression::{Cell, BRANCHES};
use crate::skills::{
    self as sk, Board, Entry, Form, Offer, Pick, ENTRIES, NE, PICK_RANK, SIZE_RANK,
};
use crate::weapons::Grip;

/// 화면의 알림(산 것·되돌린 것)이 상태 줄에 떠 있는 초.
const ALERT: f32 = 1.4;
/// 상세 칸이 넘칠 때 굴림 표시(▲▼)에 비워 두는 맨 아래 줄의 높이(창 단위).
const PANE_MARKS: f32 = 9.0;
/// 커서의 자리: 판의 칸들(`ENTRIES`의 자리) 다음이 "되돌리기" 단추.
pub(super) const RESPEC: usize = NE;

/// 따뜻한 흰빛(익힌 칸의 그림·고른 칸의 테) — 키캡 글과 같은 빛.
const CREAM: Color = ui::KEY_TEXT;
/// 금빛으로 채운 칸(연 패시브) 위의 어두운 그림.
const DARK: Color = Color::new(0.10, 0.07, 0.08, 1.0);
const GOLD: Color = Color::new(0.84, 0.58, 0.22, 1.0);
const GOLD_HI: Color = Color::new(1.0, 0.80, 0.42, 1.0);
const WARM: Color = Color::new(0.21, 0.15, 0.11, 1.0);

/// 스킬 이름·설명(글 표).
pub(super) fn skill_text(id: sk::Id) -> (&'static str, &'static str) {
    let s = s();
    match id {
        sk::Id::Whirl => (s.skill_whirl, s.skill_whirl_d),
        sk::Id::Cut => (s.skill_cut, s.skill_cut_d),
        sk::Id::Gather => (s.skill_gather, s.skill_gather_d),
        sk::Id::Quake => (s.skill_quake, s.skill_quake_d),
        sk::Id::Brace => (s.skill_brace, s.skill_brace_d),
        sk::Id::Echo => (s.skill_echo, s.skill_echo_d),
        sk::Id::Throw => (s.skill_throw, s.skill_throw_d),
        sk::Id::Embers => (s.skill_embers, s.skill_embers_d),
        sk::Id::Swallow => (s.skill_swallow, s.skill_swallow_d),
        sk::Id::Bash => (s.skill_bash, s.skill_bash_d),
        sk::Id::Roots => (s.skill_roots, s.skill_roots_d),
    }
}

/// 큰 기술 이름·설명(글 표).
pub(super) fn ult_text(u: sk::Ult) -> (&'static str, &'static str) {
    let s = s();
    match u {
        sk::Ult::Upend => (s.ult_upend, s.ult_upend_d),
        sk::Ult::Flurry => (s.ult_flurry, s.ult_flurry_d),
        sk::Ult::Line => (s.ult_line, s.ult_line_d),
    }
}

/// 칸의 이름·하는 일.
pub(super) fn entry_text(e: Entry) -> (&'static str, &'static str) {
    match e {
        Entry::Skill(id) => skill_text(id),
        Entry::Ult(u) => ult_text(u),
        Entry::Cell(c) => {
            let d = c.def();
            (tr(d.name), tr(d.desc))
        }
        Entry::Key(k) => (tr(k.name()), tr(k.desc())),
        Entry::Link(l) => (tr(l.name()), tr(l.desc())),
        Entry::Vein(v) => (tr(v.name()), tr(v.desc())),
    }
}

/// 숫자 칸의 고르기(0215 D): 표지·이름·한 줄 — I = 칸 자체, II = `Cell::pick_b`.
pub(super) fn cell_pick_text(c: Cell, b: bool) -> (&'static str, &'static str, &'static str) {
    let s = s();
    let d = c.def();
    match (b, c.pick_b()) {
        (true, Some((name, desc))) => (s.pick_mark_b, tr(name), tr(desc)),
        _ => (s.pick_mark_a, tr(d.name), tr(d.desc)),
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
        (sk::Id::Brace, Pick::A) => (s.pick_brace_a, s.pick_brace_a_d),
        (sk::Id::Brace, Pick::B) => (s.pick_brace_b, s.pick_brace_b_d),
        (sk::Id::Echo, Pick::A) => (s.pick_echo_a, s.pick_echo_a_d),
        (sk::Id::Echo, Pick::B) => (s.pick_echo_b, s.pick_echo_b_d),
        (sk::Id::Throw, Pick::A) => (s.pick_throw_a, s.pick_throw_a_d),
        (sk::Id::Throw, Pick::B) => (s.pick_throw_b, s.pick_throw_b_d),
        (sk::Id::Embers, Pick::A) => (s.pick_embers_a, s.pick_embers_a_d),
        (sk::Id::Embers, Pick::B) => (s.pick_embers_b, s.pick_embers_b_d),
        (sk::Id::Swallow, Pick::A) => (s.pick_swallow_a, s.pick_swallow_a_d),
        (sk::Id::Swallow, Pick::B) => (s.pick_swallow_b, s.pick_swallow_b_d),
        (sk::Id::Bash, Pick::A) => (s.pick_bash_a, s.pick_bash_a_d),
        (sk::Id::Bash, Pick::B) => (s.pick_bash_b, s.pick_bash_b_d),
        (sk::Id::Roots, Pick::A) => (s.pick_roots_a, s.pick_roots_a_d),
        (sk::Id::Roots, Pick::B) => (s.pick_roots_b, s.pick_roots_b_d),
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
        sk::Id::Brace => s.cap_brace,
        sk::Id::Echo => s.cap_echo,
        sk::Id::Throw => s.cap_throw,
        sk::Id::Embers => s.cap_embers,
        sk::Id::Swallow => s.cap_swallow,
        sk::Id::Bash => s.cap_bash,
        sk::Id::Roots => s.cap_roots,
    }
}

/// 배율 → "15" 같은 백분율 글(표의 수에서 — 글에 숫자를 박지 않는다).
fn pct(mult: f32) -> String {
    ((mult - 1.0).abs() * 100.0).round().to_string()
}

/// 정수면 정수로, 아니면 한 자리(6.5초·6.3초).
fn num(v: f32) -> String {
    if (v - v.round()).abs() < 1e-3 {
        format!("{v:.0}")
    } else {
        format!("{v:.1}")
    }
}

/// 스킬 `id`의 `rank`단이 주는 것(한 줄 — 3단의 고르기는 따로 그린다).
pub(super) fn rank_gives(id: sk::Id, rank: u8) -> String {
    let s = s();
    let d = id.def();
    let harmless = d.damage == 0;
    match rank {
        0 | 1 => s.sk_rank_learn.to_string(),
        // 되울림의 2단은 이어지는 시간(0158 G2).
        2 if matches!(d.form, Form::Echo { .. }) => fill(s.rk_secs, &[&num(sk::RANK2_SECS)]),
        2 if harmless => fill(s.rk_stagger, &[&format!("{:.1}", sk::RANK2_STAGGER)]),
        2 => fill(s.rk_damage, &[&pct(sk::RANK2_DAMAGE)]),
        r if r == PICK_RANK => s.sk_rank_pick.to_string(),
        r if r == SIZE_RANK => {
            let t = match d.form {
                Form::Ring { .. } | Form::Brace { .. } | Form::Echo { .. } | Form::Field { .. } => {
                    s.rk_radius
                }
                Form::Lane { .. } | Form::Erupt { .. } => s.rk_length,
                Form::Pull { .. } | Form::Throw { .. } | Form::Hop { .. } => s.rk_range,
                Form::Cone { .. } | Form::Bash { .. } => s.rk_arc,
            };
            fill(t, &[&pct(sk::RANK4_SIZE)])
        }
        _ => {
            let mut parts = Vec::new();
            if !harmless {
                // 5단의 피해는 2단의 몫에 더한 것(합 +30%).
                let more = pct(1.0 + sk::RANK5_DAMAGE - sk::RANK2_DAMAGE);
                parts.push(fill(s.rk_damage_more, &[&more]));
            }
            parts.push(fill(s.rk_cooldown, &[&pct(sk::RANK5_COOLDOWN)]));
            parts.push(fill(s.sk_cap, &[cap_text(id)]));
            parts.join(" · ")
        }
    }
}

/// 큰 기술 `u`의 `rank`단이 주는 것(표의 수에서).
pub(super) fn ult_gives(u: sk::Ult, rank: u8) -> String {
    let s = s();
    match (u, rank) {
        (_, 0 | 1) => s.sk_rank_learn.to_string(),
        (sk::Ult::Upend, 2) => fill(s.rk_stagger, &[&format!("{:.1}", sk::ULT_RANK2_STAGGER)]),
        (sk::Ult::Upend, _) => fill(s.rk_slam, &[&pct(sk::ULT_RANK3_SLAM)]),
        (sk::Ult::Flurry, 2) => fill(s.rk_flurry_secs, &[&num(sk::FLURRY.secs2)]),
        (sk::Ult::Flurry, _) => fill(
            s.rk_flurry_kill,
            &[&num(sk::FLURRY.kill_more), &num(sk::FLURRY.kill_cap)],
        ),
        (sk::Ult::Line, 2) => fill(s.rk_line_len, &[&num(sk::LINE.len2)]),
        (sk::Ult::Line, _) => fill(s.rk_line_dmg, &[&sk::LINE.damage3.to_string()]),
    }
}

/// 칸의 갈래·종류 줄: "스킬 · 흐름 · 대기 7초"(대기는 지금 단의 값) / "큰 기술 · 자리" / "패시브 · 맞받기".
pub(super) fn kind_line(board: &Board, e: Entry) -> String {
    let s = s();
    let branch = tr(e.branch().name());
    match e {
        Entry::Skill(id) => {
            let eff = id.def().at(board.rank[id.index()], board.pick[id.index()]);
            fill(s.sk_kind_skill, &[branch, &num(eff.skill.cooldown)])
        }
        Entry::Ult(_) => fill(s.sk_kind_ult, &[branch]),
        Entry::Cell(c) if c.pick_b().is_some() => fill(s.sk_kind_cell_pick, &[branch]),
        Entry::Cell(_) => fill(s.sk_kind_cell, &[branch]),
        Entry::Key(_) => fill(s.sk_kind_key, &[branch]),
        Entry::Link(l) => {
            let right = tr(BRANCHES[l.left() + 1].name());
            fill(s.sk_kind_link, &[branch, right])
        }
        Entry::Vein(v) => fill(s.sk_kind_vein, &[crate::i18n::ail_name(v.kind())]),
    }
}

/// 칸의 밝기(KB `meta-progression-unlock-planning` — 네 단): 익힘·열림 > 지금 살 수 있음 > 점수가 모자람 > 줄이 잠김.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Look {
    Owned,
    Ready,
    Short,
    Locked,
}

/// 칸의 밝기와 "다음 단을 지금 살 수 있나"(익힌 칸도 더 올릴 수 있으면 참 — 다음 단의 점이 민트).
pub(super) fn look(board: &Board, total: u32, e: Entry) -> (Look, bool) {
    let offer = board.offer(e, total);
    let ready = offer == Offer::Ready;
    let look = if board.rank(e) >= 1 {
        Look::Owned
    } else {
        match offer {
            Offer::Ready => Look::Ready,
            Offer::Short | Offer::Swap => Look::Short,
            Offer::Locked(_) | Offer::Cut | Offer::Branch(_) => Look::Locked,
            Offer::Full => Look::Owned,
        }
    };
    (look, ready)
}

/// 고른 칸의 상태 한 줄과 그 빛(민트 = 지금 할 수 있다, 흐림 = 못 한다, 금빛 = 다 했다).
pub(super) fn state_line(board: &Board, total: u32, e: Entry, edit: bool) -> (String, Color) {
    let s = s();
    let price = e.price(board.rank(e)).to_string();
    match board.offer(e, total) {
        Offer::Full if matches!(e, Entry::Cell(_) | Entry::Link(_) | Entry::Vein(_)) => {
            (s.sk_st_open_cell.to_string(), AMBER)
        }
        Offer::Full if matches!(e, Entry::Key(_)) => (s.sk_st_key_on.to_string(), AMBER),
        // 다른 맺음을 켰다(0214): 바꾸기는 값이 없다 — 화로에서.
        Offer::Swap if !edit => (s.sk_st_key_view.to_string(), INK),
        Offer::Swap => (s.sk_st_key_swap.to_string(), ui::MINT),
        Offer::Full => (s.board_full.to_string(), AMBER),
        Offer::Locked(n) => (fill(s.sk_st_locked, &[&n.to_string()]), DIM),
        // 0215: 길이 끊겼다 · 맺음은 그 갈래에 더 써야.
        Offer::Cut => match e {
            // 결의 칸(0223 11절): 길이 아니라 그 타입의 스킬이 먼저다.
            Entry::Vein(v) => (fill(s.sk_st_vein, &[crate::i18n::ail_name(v.kind())]), DIM),
            _ => (s.sk_st_cut.to_string(), DIM),
        },
        Offer::Branch(n) => (fill(s.sk_st_branch, &[&n.to_string()]), DIM),
        Offer::Short => (fill(s.sk_st_short, &[&price]), DIM),
        // 멈춤에서는 올릴 수 없다 — 어디서 하는지를 말한다.
        Offer::Ready if !edit => (s.sk_st_view.to_string(), INK),
        Offer::Ready if matches!(e, Entry::Cell(_) | Entry::Link(_) | Entry::Vein(_)) => {
            (fill(s.sk_st_open, &[&price]), ui::MINT)
        }
        Offer::Ready => (fill(s.sk_st_raise, &[&price]), ui::MINT),
    }
}

/// 칸에 할 수 있는 일(확인을 누르면 뜨는 작은 목록의 줄).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Do {
    /// 다음 단을 산다·칸을 연다(화로에서만 — 묻는 창을 거친다).
    Buy,
    /// 이 스킬을 칸에 끼운다(같은 무기의 다른 칸에 있었으면 맞바꾼다): 0·1 = 주무기의 칸, 2·3 = 부무기의 칸(0223 V3).
    /// 그 무기의 갈래로 쓸 수 있을 때만 줄이 선다.
    Wear(usize),
    /// 배웠지만 지금 든 무기(주·부)의 갈래로는 쓸 수 없다(0223 V3) — 흐린 줄: 고르면 어느 무기의 스킬인지 말한다.
    Unfit,
    /// 이 큰 기술을 끼운다(하나만).
    WearUlt,
    /// 3단의 고르기를 뒤집는다(화로에서만, 값 없이).
    Flip,
    /// 올릴·열 수 있는데 여기서는 못 한다(멈춤 — 쉼터 화로에서만, 0173). 목록의 흐린 줄: 고르면 까닭을 말한다.
    Away,
}

/// 칸 `e`에 지금 할 수 있는 일만, 가장 앞이 으뜸(커서가 거기서 시작한다): 올리기·열기 → 끼우기 → 고르기 바꾸기.
/// 멈춤(`edit` 아님)에서는 끼우기만 — 올릴 수 있는 칸이면 끝에 "쉼터 화로에서" 줄(`Do::Away`). 비면 확인은 까닭을 알린다.
/// 끼우는 자리의 사정(0223 V3 — 무기 칸마다 스킬 둘): 주무기의 갈래와 그 칸 둘, 부무기를 들었으면 그 갈래와 칸 둘.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Kits {
    pub main: (Grip, [sk::Id; 2]),
    pub sub: Option<(Grip, [sk::Id; 2])>,
}

pub(super) fn node_acts(
    board: &Board,
    total: u32,
    kits: &Kits,
    great: sk::Ult,
    e: Entry,
    edit: bool,
) -> Vec<Do> {
    let mut v = Vec::new();
    let offer = board.offer(e, total);
    if edit && matches!(offer, Offer::Ready | Offer::Swap) {
        v.push(Do::Buy);
    }
    match e {
        Entry::Skill(id) if board.learned(id) => {
            let def = id.def();
            let before = v.len();
            if def.fits(kits.main.0) {
                v.extend((0..2).filter(|&k| kits.main.1[k] != id).map(Do::Wear));
            }
            let fit_sub = kits.sub.is_some_and(|(h, _)| def.fits(h));
            if let Some((_, worn)) = kits.sub.filter(|_| fit_sub) {
                v.extend((0..2).filter(|&k| worn[k] != id).map(|k| Do::Wear(k + 2)));
            }
            if v.len() == before && !def.fits(kits.main.0) && !fit_sub {
                v.push(Do::Unfit);
            }
            if edit && board.rank[id.index()] >= PICK_RANK {
                v.push(Do::Flip);
            }
        }
        Entry::Ult(u) if board.has_ult(u) && board.wearable_ult(great) != u => v.push(Do::WearUlt),
        // 숫자 칸의 고르기(0215 D): 연 칸이면 화로에서 값 없이 바꾼다.
        Entry::Cell(c) if edit && c.pick_b().is_some() && board.cells.has(c) => v.push(Do::Flip),
        _ => {}
    }
    // 멈춤(0173): 점수가 있어 올릴 수 있는 칸인데 올리는 줄이 없으면 왜인지 알 수 없었다 — 끝에 흐린 줄로 남긴다.
    if !edit && matches!(offer, Offer::Ready | Offer::Swap) {
        v.push(Do::Away);
    }
    v
}

/// 스킬 `id`를 칸 `slot`에 끼운 뒤의 두 칸: 다른 칸에 끼어 있었으면 맞바꾼다(같은 스킬 둘은 없다).
pub(super) fn wear_skill(worn: [sk::Id; 2], slot: usize, id: sk::Id) -> [sk::Id; 2] {
    let mut out = worn;
    if worn[1 - slot] == id {
        out[1 - slot] = worn[slot];
    }
    out[slot] = id;
    out
}

/// 그 스킬을 쓰는 무기 갈래의 이름들(" · "로 이은 글 — 어느 무기나 되면 그 말).
pub(super) fn hands_text(id: sk::Id) -> String {
    let s = s();
    let h = id.def().hands;
    // 한손검의 두 갈래(0225): 방패와 함께만 쓰는 것 = 방패 기술, 방패 없이만 쓰는 것 = 결투 기술.
    match h {
        sk::ANY => return s.hands_any.to_string(),
        sk::DUEL => return s.hands_duel_art.to_string(),
        sk::SHIELD => return s.hands_shield_art.to_string(),
        _ => {}
    }
    // 한손 둘(방패가 있든 없든)·양손 둘(망치·지팡이)이 다 되면 묶어서 말한다.
    let mut names = Vec::new();
    let mut left = h;
    for (bits, name) in [
        (sk::ONE, s.hands_one),
        (sk::TWO, s.hands_two),
        (sk::DUEL, s.hands_duel),
        (sk::SHIELD, s.hands_shield),
        (sk::HEAVY, s.hands_heavy),
        (sk::FOCUS, s.hands_focus),
        (sk::RANGED, s.hands_ranged),
    ] {
        if left & bits == bits {
            names.push(name);
            left &= !bits;
        }
    }
    names.join(" · ")
}

/// 든 것으로 못 쓰는 스킬을 고르면 뜨는 알림 — 한 줄 자리라 짧게(쓰는 무기의 목록은 상세 칸의 "쓰는 무기" 줄이 말한다, 0225).
fn unfit_toast() -> String {
    s().toast_unfit.to_string()
}

/// 목록 한 줄의 글.
fn act_label(board: &Board, e: Entry, d: Do, has_sub: bool) -> String {
    let s = s();
    match (d, e) {
        (Do::Buy, Entry::Cell(_) | Entry::Link(_) | Entry::Vein(_)) => {
            fill(s.act_open, &[&e.price(0).to_string()])
        }
        (Do::Buy, Entry::Key(_)) if board.key_swap(e) => s.act_key_swap.to_string(),
        (Do::Buy, Entry::Key(_)) => fill(s.act_key_on, &[&e.price(0).to_string()]),
        (Do::Buy, _) => fill(s.act_raise, &[&e.price(board.rank(e)).to_string()]),
        // 부무기가 있으면 어느 무기의 칸인지 말한다(0223 V3).
        (Do::Wear(k), _) if k >= 2 => fill(s.act_wear_sub, &[&(k - 1).to_string()]),
        (Do::Wear(k), _) if has_sub => fill(s.act_wear_main, &[&(k + 1).to_string()]),
        (Do::Wear(k), _) => fill(s.act_wear, &[&(k + 1).to_string()]),
        (Do::Unfit, _) => s.act_unfit.to_string(),
        (Do::WearUlt, _) => s.act_wear_ult.to_string(),
        (Do::Flip, Entry::Skill(id)) => {
            let (mark, name, _) = pick_text(id, board.pick[id.index()].other());
            fill(s.act_flip, &[mark, name])
        }
        (Do::Flip, Entry::Cell(c)) => {
            let (mark, name, _) = cell_pick_text(c, !board.cells.picked_b(c));
            fill(s.act_flip, &[mark, name])
        }
        (Do::Flip, _) => String::new(),
        (Do::Away, _) => s.act_away.to_string(),
    }
}

/// 화면의 자리(480×270 기준 px, 창의 몸 왼쪽 위가 원점). 칸의 자리·걷기·그림이 같은 수를 쓴다.
pub(super) mod geo {
    /// 왼쪽 칸 격자: 갈래 셋.
    pub const GRID_W: f32 = super::super::tabs::LEFT_W;
    pub const COL_W: f32 = crate::skills::LANE_W;
    /// 갈래 머리(이름 + 한 줄).
    pub const HEAD_H: f32 = 27.0;
    /// 줄 띠의 높이(첫 줄·둘째 줄 = 스킬 줄 + 패시브 줄, 셋째 줄 = 큰 기술과 패시브가 한 줄). 띠 셋의 합이 보이는 높이를 넘는다 —
    /// 격자는 위아래로 굴린다(0163: 이름표와 칸 사이를 띄우고, 줄이 늘어도 같은 꼴).
    /// 0214: 넷째 줄(스킬 + 패시브 — 첫·둘째 줄과 같은 꼴) · 맺음 줄(마름모 하나씩).
    pub const BAND_H: [f32; 6] = [72.0, 72.0, 56.0, 72.0, 56.0, 44.0];
    /// 맺음 칸의 반 대각선(마름모 — 패시브보다 크게).
    pub const KEY: f32 = 26.0;
    /// 다리 칸의 대각선(0215 — 갈래 경계 위의 작은 마름모 고리).
    pub const LINK: f32 = 15.0;
    /// 띠 안: 스킬 칸의 위·한 변, 단의 점, 패시브 칸의 위·지름.
    pub const CELL_TOP: f32 = 17.0;
    pub const SKILL: f32 = 24.0;
    pub const PIPS_TOP: f32 = 42.5;
    pub const PASSIVE_TOP: f32 = 49.0;
    pub const PASSIVE: f32 = 18.0;
    /// 굴리기: 휠·▲▼ 한 칸(기준 px), 커서가 창을 끌 때 칸 둘레에 남기는 여유(위: 이름표, 아래: 단의 점).
    pub const WHEEL: f32 = 24.0;
    pub const REVEAL_PAD: f32 = 4.0;
    /// 오른쪽 상세 칸.
    pub const DETAIL_X: f32 = super::super::tabs::RIGHT_X;
    pub const DETAIL_W: f32 = super::super::tabs::RIGHT_W;
    /// 상세의 머리(그림 + 이름·갈래·상태)와 큰 그림의 한 변.
    pub const DETAIL_HEAD: f32 = super::super::tabs::HEAD_H;
    pub const DETAIL_ICON: f32 = super::super::tabs::HEAD_ICON;
    /// 되돌리기 단추(창 아래 안내 줄의 오른쪽 끝)의 폭·높이.
    pub const RESPEC_W: f32 = 150.0;
    pub const RESPEC_H: f32 = 16.0;

    /// 띠를 그리는 차례(2026-11-23 사장 "속성 패시브가 가장 위쪽에 올라와서 보이도록"): 결의 줄(타입마다의 패시브)이 맨 위,
    /// 그 아래로 판의 줄 차례.
    pub fn band_order(row: usize) -> usize {
        if row == crate::skills::VEIN_ROW {
            0
        } else {
            row + 1
        }
    }

    pub fn band_top(row: usize) -> f32 {
        let me = band_order(row);
        HEAD_H
            + (0..BAND_H.len())
                .filter(|&r| band_order(r) < me)
                .map(|r| BAND_H[r])
                .sum::<f32>()
    }

    /// 굴리는 내용의 높이(띠 셋).
    pub fn content_h() -> f32 {
        BAND_H.iter().sum()
    }
}

/// 열린 줄의 안내·선이 걷히는 초(2026-11-23).
const ROW_FADE: f32 = 0.8;

/// 칸 하나의 자리: 가운데와 반 변(반지름).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Spot {
    pub c: Vec2,
    pub half: f32,
}

/// 판의 칸마다의 자리(`ENTRIES`의 자리). 줄·갈래마다 스킬(큰 기술)은 위, 패시브는 그 아래 — 셋째 줄만 나란히. 가로 자리는
/// 판의 길과 같은 `Entry::lane`(0215 — 선이 그림과 어긋나지 않게), 다리 칸은 패시브 줄의 갈래 경계에 작은 마름모.
pub(super) fn spots() -> [Spot; NE] {
    use geo::*;
    std::array::from_fn(|i| {
        let e = ENTRIES[i];
        let row = e.row();
        let top = band_top(row);
        let big_y = top + CELL_TOP + SKILL * 0.5;
        let small_y = top + PASSIVE_TOP + PASSIVE * 0.5;
        let (y, half) = match e {
            Entry::Key(_) => (big_y, KEY * 0.5),
            Entry::Cell(_) if row == sk::ULT_ROW as usize => (big_y, PASSIVE * 0.5),
            Entry::Cell(_) => (small_y, PASSIVE * 0.5),
            Entry::Link(_) => (small_y, LINK * 0.5),
            // 결의 줄(0223 11절): 띠 하나에 둥근 칸 넷.
            Entry::Vein(_) => (top + CELL_TOP + PASSIVE * 0.5, PASSIVE * 0.5),
            Entry::Skill(_) | Entry::Ult(_) => (big_y, SKILL * 0.5),
        };
        Spot {
            c: vec2(e.lane(), y),
            half,
        }
    })
}

/// 칸 `i`가 "보인다"고 할 구간(내용의 y — 띠 셋의 처음이 0): 커서가 그 칸에 서면 격자 창이 이 구간이 보이게 가장 적게 움직인다(0163).
/// 줄의 윗칸(스킬·큰 기술)은 띠의 머리(이름표)부터 단의 점까지, 아랫칸(패시브)은 칸부터 띠의 끝까지 — 셋째 줄은 띠 하나가 통째로.
pub(super) fn reveal_span(i: usize) -> (f32, f32) {
    use geo::*;
    let (e, sp) = (ENTRIES[i], spots()[i]);
    let row = e.row();
    let (top, bottom) = (band_top(row) - HEAD_H, band_top(row) + BAND_H[row] - HEAD_H);
    let (y, h) = (sp.c.y - HEAD_H, sp.half);
    match e {
        Entry::Cell(_) if row != 2 => (y - h - REVEAL_PAD, bottom),
        _ if row == 2 || row == sk::KEY_ROW || row == sk::VEIN_ROW => (top, bottom),
        _ => (top, y + h + 6.0 + REVEAL_PAD),
    }
}

/// 십자 한 걸음(0161 — KB `game-menu-scene-stack-and-focus`: 격자는 돌지 않는다): 누른 쪽에 있는 칸 가운데 가장 가깝고 곧은 것으로.
/// 그쪽에 칸이 없으면 그대로 — 다만 맨 아래에서 **아래**는 되돌리기 단추(창 아래 오른쪽, `respec`일 때 — 화로). 오른쪽 걸음은 격자에서
/// 끝난다(0261, 사장 "오른쪽 이동은 기술 쪽에서 끝나야 한다" — 단추에서 왼쪽으로 가도 떠난 자리로 돌아오지 않아 길을 잃었다).
/// 단추에서 위는 `back`(단추로 내려오기 전의 칸)으로, 옆은 그대로.
pub(super) fn walk(from: usize, dx: i32, dy: i32, respec: bool, back: usize) -> usize {
    if dx == 0 && dy == 0 {
        return from;
    }
    if from >= NE {
        return if dy < 0 { back.min(NE - 1) } else { from };
    }
    let spots = spots();
    let here = spots[from].c;
    let dir = vec2(dx as f32, dy as f32);
    // 옆 걸음은 같은 높이의 칸만(줄은 반듯하다 — 비스듬히 다른 줄로 새지 않게 좁게 보고 어긋남을 크게 친다),
    // 위아래 걸음은 가장 가까운 줄로(갈래 안의 칸은 줄마다 자리가 달라 넓게 보고 어긋남을 작게 친다).
    let (cone, weight) = if dx != 0 { (0.5, 3.0) } else { (1.2, 0.6) };
    let best = (0..NE)
        .filter(|&i| i != from)
        .filter_map(|i| {
            let d = spots[i].c - here;
            let along = d.dot(dir);
            let across = (d - dir * along).length();
            (along > 0.5 && across <= along * cone).then_some((i, along + across * weight))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i);
    match best {
        Some(i) => i,
        None if respec && dy > 0 => RESPEC,
        None => from,
    }
}

/// 칸의 꼴.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Shape {
    Square,
    Oct,
    Round,
    /// 맺음(0214).
    Diamond,
}

fn shape_of(e: Entry) -> Shape {
    match e {
        Entry::Skill(_) => Shape::Square,
        Entry::Ult(_) => Shape::Oct,
        Entry::Cell(_) | Entry::Vein(_) => Shape::Round,
        Entry::Key(_) | Entry::Link(_) => Shape::Diamond,
    }
}

/// 칸의 그림(13×13 도트)의 칸이 칠해졌나.
fn pic(e: Entry, x: usize, y: usize) -> bool {
    match e {
        Entry::Skill(id) => touch::icon_cell(touch::skill_icon(id), x, y),
        Entry::Ult(u) => touch::icon_cell(touch::ult_icon(u), x, y),
        Entry::Cell(c) => crate::icons::on(&crate::icons::cell_mask(c), x, y),
        Entry::Key(k) => crate::icons::on(&crate::icons::key_mask(k), x, y),
        // 다리 칸은 작은 마름모 고리 — 그림 대신 가운데 점(`draw_node`).
        Entry::Link(_) => false,
        Entry::Vein(v) => crate::icons::on(&crate::icons::vein_mask(v), x, y),
    }
}

/// 닫힌 꺾은선의 점선(지금 살 수 있는 칸의 테 — 색 말고도 꼴로 갈린다).
fn dashed(points: &[Vec2], dash: f32, gap: f32, thick: f32, col: Color) {
    let n = points.len();
    let mut on = true;
    let mut left = dash;
    for i in 0..n {
        let (a, b) = (points[i], points[(i + 1) % n]);
        let len = (b - a).length();
        if len < 1e-3 {
            continue;
        }
        let dir = (b - a) / len;
        let mut at = 0.0;
        while at < len {
            let step = left.min(len - at);
            if on {
                let (p, q) = (a + dir * at, a + dir * (at + step));
                draw_line(p.x, p.y, q.x, q.y, thick, col);
            }
            at += step;
            left -= step;
            if left <= 1e-3 {
                on = !on;
                left = if on { dash } else { gap };
            }
        }
    }
}

/// 꼴의 둘레 점(화면 px).
fn outline(shape: Shape, c: Vec2, half: f32) -> Vec<Vec2> {
    match shape {
        Shape::Square => vec![
            c + vec2(-half, -half),
            c + vec2(half, -half),
            c + vec2(half, half),
            c + vec2(-half, half),
        ],
        Shape::Oct => {
            let k = half * 0.42;
            vec![
                c + vec2(-k, -half),
                c + vec2(k, -half),
                c + vec2(half, -k),
                c + vec2(half, k),
                c + vec2(k, half),
                c + vec2(-k, half),
                c + vec2(-half, k),
                c + vec2(-half, -k),
            ]
        }
        Shape::Round => (0..36)
            .map(|i| {
                let a = i as f32 / 36.0 * std::f32::consts::TAU;
                c + vec2(a.cos(), a.sin()) * half
            })
            .collect(),
        Shape::Diamond => vec![
            c + vec2(0.0, -half),
            c + vec2(half, 0.0),
            c + vec2(0.0, half),
            c + vec2(-half, 0.0),
        ],
    }
}

/// 꼴을 채운다(볼록 — 가운데에서 부채로).
fn fill_shape(points: &[Vec2], c: Vec2, col: Color) {
    for i in 0..points.len() {
        draw_triangle(c, points[i], points[(i + 1) % points.len()], col);
    }
}

/// 닫힌 꺾은선.
fn stroke(points: &[Vec2], thick: f32, col: Color) {
    for i in 0..points.len() {
        let (a, b) = (points[i], points[(i + 1) % points.len()]);
        draw_line(a.x, a.y, b.x, b.y, thick, col);
    }
}

/// 칸 하나(그림까지): `half` = 화면 px의 반 변, `s` = 기준 px 하나의 화면 크기. `focus` = 고른 칸(밝은 바깥 테 — 스킬·큰 기술은
/// 아래의 단의 점까지 두른다, `Some(점 줄의 아래 끝 y)`).
pub(super) fn draw_node(e: Entry, c: Vec2, half: f32, s: f32, look: Look, focus: Option<f32>) {
    let shape = shape_of(e);
    let c = c.round();
    let pts = outline(shape, c, half);
    let line = (s * 1.2).max(1.0);
    // 면·테·그림의 빛.
    let (face, edge, body, shade) = match (look, shape) {
        (Look::Owned, Shape::Round | Shape::Diamond) => {
            (GOLD, GOLD_HI, DARK, Color::new(0.0, 0.0, 0.0, 0.0))
        }
        (Look::Owned, _) => (WARM, AMBER, CREAM, Color::new(0.04, 0.03, 0.05, 0.75)),
        (Look::Ready, _) => (
            Color::new(0.10, 0.11, 0.13, 1.0),
            ui::MINT,
            Color { a: 0.92, ..INK },
            Color::new(0.04, 0.03, 0.05, 0.7),
        ),
        (Look::Short, _) => (
            Color::new(0.11, 0.09, 0.12, 1.0),
            Color {
                a: 0.85,
                ..ui::BORDER
            },
            Color { a: 0.7, ..DIM },
            Color::new(0.04, 0.03, 0.05, 0.5),
        ),
        (Look::Locked, _) => (
            Color::new(0.07, 0.06, 0.08, 1.0),
            Color {
                a: 0.4,
                ..ui::BORDER
            },
            Color { a: 0.38, ..DIM },
            Color::new(0.04, 0.03, 0.05, 0.25),
        ),
    };
    // 결의 칸(0223 11절): 연 칸은 그 타입의 빛(적의 머리 위 표지와 같은 빛 — `status::INK`).
    let (face, edge) = match (e, look) {
        (Entry::Vein(v), Look::Owned) => {
            let [r, g, b] = crate::status::INK[v.index()];
            (
                Color::new(r * 0.8, g * 0.8, b * 0.8, 1.0),
                Color::new(r, g, b, 1.0),
            )
        }
        _ => (face, edge),
    };
    fill_shape(&pts, c, face);
    match look {
        // 지금 살 수 있는 칸: 민트 점선.
        Look::Ready => dashed(&pts, s * 3.0, s * 2.0, line, edge),
        Look::Owned => {
            stroke(&pts, (s * 1.6).max(1.5), edge);
            // 큰 기술은 테 두 겹.
            if shape == Shape::Oct {
                stroke(
                    &outline(shape, c, half - s * 2.6),
                    1.0,
                    Color { a: 0.55, ..edge },
                );
            }
        }
        Look::Short | Look::Locked => stroke(&pts, 1.0, edge),
    }
    // 다리 칸(0215): 그림 대신 가운데 작은 마름모(가졌으면 어둡게, 아니면 테 빛).
    if let Entry::Link(_) = e {
        let dot = outline(Shape::Diamond, c, (half * 0.38).max(2.0));
        fill_shape(&dot, c, if look == Look::Owned { DARK } else { edge });
    }
    // 그림: 정수 px의 칸, 칸 안에 들어가는 가장 큰 크기.
    let px = (half * 2.0 / 15.6).floor().max(1.0);
    crate::icons::draw(|x, y| pic(e, x, y), c, px, body, shade);
    if let Some(bottom) = focus {
        // 고른 칸: 밝은 바깥 테(어두운 테를 한 겹 깔아 금빛 칸 위에서도 갈린다). 둥근 칸은 둥글게, 나머지는 점까지 두른 네모.
        let r = half + s * 3.0;
        let out = match shape {
            Shape::Round | Shape::Diamond => outline(shape, c, r),
            _ => {
                let low = (bottom + s * 2.0).max(c.y + r);
                vec![
                    c + vec2(-r, -r),
                    c + vec2(r, -r),
                    vec2(c.x + r, low),
                    vec2(c.x - r, low),
                ]
            }
        };
        stroke(&out, (s * 3.2).max(3.0), Color::new(0.03, 0.02, 0.04, 0.9));
        stroke(&out, (s * 1.6).max(2.0), CREAM);
    }
}

/// 단의 점: `n`개 가운데 `have`개가 금빛, 다음 것(`next`)은 민트 테, 나머지는 흐린 테. 큰 기술은 마름모.
#[allow(clippy::too_many_arguments)]
fn draw_pips(cx: f32, y: f32, n: u8, have: u8, next: bool, diamond: bool, size: f32, fade: f32) {
    let gap = (size * 0.4).max(1.0).round();
    let size = size.round().max(2.0);
    let w = n as f32 * size + (n as f32 - 1.0) * gap;
    let x0 = (cx - w * 0.5).round();
    for i in 0..n {
        let x = x0 + i as f32 * (size + gap);
        let (c, h) = (vec2(x + size * 0.5, y + size * 0.5), size * 0.5);
        let done = i < have;
        let col = if done {
            AMBER
        } else if next && i == have {
            ui::MINT
        } else {
            Color {
                a: 0.7 * fade,
                ..ui::BORDER
            }
        };
        if diamond {
            let pts = [
                c + vec2(0.0, -h - 1.0),
                c + vec2(h + 1.0, 0.0),
                c + vec2(0.0, h + 1.0),
                c + vec2(-h - 1.0, 0.0),
            ];
            if done {
                fill_shape(&pts, c, col);
            } else {
                stroke(&pts, 1.0, col);
            }
        } else if done {
            draw_rectangle(x, y, size, size, col);
        } else {
            draw_rectangle_lines(x, y, size, size, 2.0, col);
        }
    }
}

/// 상세 칸의 글 덩이 하나(위에서 아래로 쌓는다 — 넘치면 덩이 단위로 굴린다).
enum Block {
    /// 글 줄들: 크기·빛.
    Text {
        lines: Vec<String>,
        k: f32,
        ink: Color,
    },
    /// 단 한 줄: 번호 · 글(여러 줄) · 다음 단이면 값. `state`: 0 = 익힌 단, 1 = 다음 단, 2 = 그 뒤.
    Rank {
        n: u8,
        lines: Vec<String>,
        state: u8,
        cost: Option<(String, bool)>,
    },
    /// 3단: 두 고르기를 나란히 — (이름 줄, 설명 줄들, 고른 것인가).
    Picks {
        boxes: [(String, Vec<String>, bool); 2],
        open: bool,
    },
    Gap(f32),
}

/// 상세 칸의 글 크기(본문 1 = 13px 기준)와 줄 높이(기준 px).
const K_NAME: f32 = super::tabs::K_NAME;
const K_KIND: f32 = super::tabs::K_KIND;
const K_BODY: f32 = super::tabs::K_BODY;
const K_RANK: f32 = 0.68;
const K_PICK: f32 = 0.62;
const LH_BODY: f32 = super::tabs::LH_BODY;
const LH_RANK: f32 = 9.8;
const LH_PICK: f32 = 8.8;
/// 단 줄의 번호 칸 폭.
const RANK_NUM: f32 = 11.0;

impl Block {
    /// 덩이의 높이(기준 px).
    fn height(&self) -> f32 {
        match self {
            Block::Text { lines, k, .. } => lines.len() as f32 * LH_BODY * (k / K_BODY),
            Block::Rank { lines, .. } => lines.len().max(1) as f32 * LH_RANK + 2.0,
            Block::Picks { boxes, .. } => {
                let n = boxes[0].1.len().max(boxes[1].1.len()) as f32;
                LH_RANK + n * LH_PICK + 6.0
            }
            Block::Gap(h) => *h,
        }
    }
}

impl App {
    /// 지금 쓰는 판: 기록의 것 — 개발 창에서 끝 단으로 올렸으면 그것(기록에 쓰지 않는다, 0075).
    pub(super) fn board(&self) -> Board {
        #[cfg(feature = "dev")]
        if let Some(b) = self.dev.board {
            return b;
        }
        // 개발 창에서 시작한 판·놀이터(시나리오 — 기록에 안 쓴다): 처음의 둘은 배운 것으로 — 스킬 버튼이 비지 않게
        // (0166, 무기를 다 드는 `App::arms`와 같은 갈림).
        #[cfg(feature = "dev")]
        if self.dev.run || self.stack.base() == Scene::Sandbox {
            return self.progress.board.with_starters().with_upend();
        }
        self.progress.board
    }

    /// 개발 창에서 올린 판을 쓰는 중인가(그동안 끼운 것·뒤집은 것은 슬롯에 쓰지 않는다).
    pub(super) fn dev_board_on(&self) -> bool {
        #[cfg(feature = "dev")]
        return self.dev.board.is_some();
        #[cfg(not(feature = "dev"))]
        false
    }

    /// 월드에 넘길 끼운 둘(그 단·고르기) + 큰 기술의 단.
    pub(super) fn loadout(&self) -> sk::Loadout {
        self.board()
            .loadout(self.progress.skills.0, self.progress.ult)
    }

    /// 판이 바뀌었다: 지금 판의 월드에 끼운 스킬(단·고르기)과 연 패시브 칸을 넣는다.
    pub(super) fn apply_board(&mut self) {
        self.world.set_skills(self.loadout());
        let sub = self
            .board()
            .loadout(self.progress.sub_skills.0, self.progress.ult);
        self.world.set_sub_skills(sub);
        self.world.tree = self.board().cells;
    }

    /// 되돌리기 단추를 누를 수 있나: 쓴 점수가 있고 잔불이 값만큼 있다(개발 창의 판은 되돌리지 않는다).
    fn respec_ready(&self) -> bool {
        let b = self.progress.board;
        !self.dev_board_on() && b.spent() > 0 && self.world.embers >= b.respec_cost()
    }

    /// 기술 화면 한 틱: 십자 = 칸 걷기, 확인·고른 칸 다시 누르기 = 그 칸의 일 목록(되돌리기 단추면 묻는 창), 뒤로·✕·창 밖 = 닫기.
    pub(super) fn tick_skills(&mut self, a: &Actions, edit: bool) {
        let mut a = *a;
        let before = self.cursor;
        // 휠·끌기: 격자 위에서는 격자 창을(0163), 상세 칸 위에서는 그 칸을 굴린다(오른쪽 스틱은 늘 상세 칸).
        self.grid_scroll(&mut a);
        self.pane_scroll(&mut a);
        let a = &a;
        let tap = a.tap.and_then(|p| self.hit_at(p));
        if let Some(Hit::Scroll(d)) = tap {
            self.view.nudge(0.0, d, geo::WHEEL * 2.0);
            return;
        }
        if a.back || a.pause || tap == Some(Hit::Close) {
            self.go(Go::Back);
            return;
        }
        self.cursor = walk(
            self.cursor.min(RESPEC),
            a.menu_right as i32 - a.menu_left as i32,
            a.menu_down as i32 - a.menu_up as i32,
            edit,
            self.respec_from,
        );
        // 단추로 내려온 칸을 기억한다 — 단추에서 위로 가면 거기로(0261).
        if self.cursor == RESPEC && before < NE {
            self.respec_from = before;
        }
        // 십자로 옮긴 커서는 창을 끈다 — 손가락으로 고른 칸은 이미 보이니 창을 옮기지 않는다(둘째 탭이 같은 자리에 닿게, 0076).
        if self.cursor != before && self.cursor < NE {
            let (y0, y1) = reveal_span(self.cursor);
            self.view.reveal(y0, y1);
        }
        // 터치·마우스: 다른 칸은 고르기만, 고른 칸을 다시 누르면 확인(장비 창과 같은 두 걸음). 단추는 한 번에.
        let confirm = match tap {
            Some(Hit::Cell(0, i)) => {
                let again = self.cursor == i || i == RESPEC;
                if i == RESPEC && self.cursor < NE {
                    self.respec_from = self.cursor;
                }
                self.cursor = i;
                again
            }
            _ => a.confirm,
        };
        if self.cursor != before {
            self.scroll = 0.0;
        }
        if !confirm {
            return;
        }
        if self.cursor >= NE {
            return self.pick_respec(edit);
        }
        let acts = self.acts_at(self.cursor, edit);
        if acts.iter().all(|d| matches!(d, Do::Away | Do::Unfit)) {
            // 할 수 있는 일이 없다 — 까닭을 상태 줄 자리에 한 번 밝게(0173: 소리만으로는 왜 안 되는지 몰랐다).
            let why = match (acts.is_empty(), ENTRIES.get(self.cursor)) {
                (false, Some(&Entry::Skill(_))) if acts.contains(&Do::Unfit) => unfit_toast(),
                (false, _) => s().toast_away.to_string(),
                (true, Some(&e)) => {
                    state_line(&self.board(), self.progress.skill_points(), e, edit).0
                }
                (true, None) => String::new(),
            };
            if !why.is_empty() {
                self.toast = Some((why, ALERT));
            }
            self.snd(crate::sfx::Id::Deny);
            return;
        }
        self.go(Go::Open(Scene::SkillAct));
    }

    /// 격자 굴리기(0163): 휠·끌기가 격자의 보이는 자리(`Hit::Field` — 칸 위도 그 안이다)에서 일어났으면 격자 창을 옮긴다.
    fn grid_scroll(&mut self, a: &mut Actions) {
        let over = a.point.is_some_and(|p| {
            self.hits
                .iter()
                .any(|(r, h)| *h == Hit::Field && r.contains(p))
        });
        if over && (a.wheel != 0 || a.drag != 0.0) {
            self.view.nudge(a.drag, a.wheel, geo::WHEEL);
            (a.wheel, a.drag) = (0, 0.0);
        }
    }

    /// 상세 칸 굴리기: 오른쪽 스틱은 늘, 휠·끌기는 그 칸 위에서 일어났을 때만(0068) — 값은 덩이 수, 그림이 끝을 자른다.
    fn pane_scroll(&mut self, a: &mut Actions) {
        let over = a.point.and_then(|p| self.hit_at(p)) == Some(Hit::Pane);
        let (wheel, drag) = if over {
            let wd = (a.wheel, a.drag);
            (a.wheel, a.drag) = (0, 0.0);
            wd
        } else {
            (0, 0.0)
        };
        let line = self.ui.u(LH_RANK).max(1.0);
        self.scroll = super::equip::pane_step(
            self.scroll,
            a.aim.y,
            TICK_DT,
            wheel,
            drag / line,
            usize::MAX >> 1,
        );
    }

    /// 칸 `i`에 지금 할 수 있는 일.
    /// 지금 든 주무기·부무기와 그 칸들(0223 V3).
    pub(super) fn kits(&self) -> Kits {
        Kits {
            // 쥔 꼴(0225): 한손 주무기는 방패와 함께인가로 갈린다. 보조무기로 바꿔 들면 왼손에 방패는 없다.
            main: (
                self.weapon.grip(self.progress.shield.is_some()),
                self.progress.skills.0,
            ),
            sub: self
                .progress
                .sub
                .map(|w| (w.grip(false), self.progress.sub_skills.0)),
        }
    }

    fn acts_at(&self, i: usize, edit: bool) -> Vec<Do> {
        let Some(&e) = ENTRIES.get(i) else {
            return Vec::new();
        };
        node_acts(
            &self.board(),
            self.progress.skill_points(),
            &self.kits(),
            self.progress.ult,
            e,
            // 개발 창의 판은 메모리의 덮어쓰기라 사지 않는다(다 올린 판이라 살 것도 없다) — 고르기 바꾸기는 된다.
            edit,
        )
        .into_iter()
        .filter(|d| !(self.dev_board_on() && *d == Do::Buy))
        .collect()
    }

    /// 되돌리기 단추: 되돌릴 것이 없거나 잔불이 모자라면 알림만, 되면 묻는다(커서는 "취소").
    fn pick_respec(&mut self, edit: bool) {
        let s = s();
        let board = self.progress.board;
        if !edit || self.dev_board_on() {
            self.snd(crate::sfx::Id::Deny);
        } else if board.spent() == 0 {
            self.toast = Some((s.board_respec_none.to_string(), ALERT));
            self.snd(crate::sfx::Id::Deny);
        } else if self.world.embers < board.respec_cost() {
            self.toast = Some((s.shop_poor.to_string(), ALERT));
            self.snd(crate::sfx::Id::Deny);
        } else {
            self.ask(flow::Ask::Respec);
        }
    }

    /// 칸의 일 목록 한 틱(기술 화면 위의 작은 창): 위아래 = 줄, 확인·줄 누르기 = 그 일, 뒤로·창 밖 = 닫기.
    pub(super) fn tick_skill_act(&mut self, a: &Actions) {
        let tap = a.tap.and_then(|p| self.hit_at(p));
        let under = self.stack.under().copied();
        let Some((at, edit)) = under.and_then(|e| match e.scene {
            Scene::Skills { host } => Some((e.cursor, host.edits())),
            _ => None,
        }) else {
            return self.go(Go::Back);
        };
        if a.back || a.pause || tap == Some(Hit::Close) {
            return self.go(Go::Back);
        }
        let acts = self.acts_at(at, edit);
        let (Some(&e), false) = (ENTRIES.get(at), acts.is_empty()) else {
            return self.go(Go::Back);
        };
        let Some(i) = self.menu_pick(a, acts.len()) else {
            return;
        };
        self.go(Go::Back);
        match (acts[i], e) {
            // 맺음 바꾸기(0214)는 값이 없다 — 묻지 않고 바로.
            (Do::Buy, Entry::Key(_)) if self.board().key_swap(e) => self.buy_skill(at),
            // 사는 일은 한 번 묻는다(0065) — 목록을 닫고 기술 화면 위에 묻는 창.
            (Do::Buy, _) => self.ask(flow::Ask::Skill(at as u8)),
            (Do::Wear(slot), Entry::Skill(id)) => {
                if slot < 2 {
                    self.progress.skills = sk::Worn(wear_skill(self.progress.skills.0, slot, id));
                } else {
                    let worn = self.progress.sub_skills.0;
                    self.progress.sub_skills = sk::Worn(wear_skill(worn, slot - 2, id));
                }
                self.after_wear();
            }
            (Do::Unfit, Entry::Skill(_)) => {
                self.toast = Some((unfit_toast(), ALERT));
                self.snd(crate::sfx::Id::Deny);
            }
            (Do::WearUlt, Entry::Ult(u)) => {
                self.progress.ult = u;
                self.after_wear();
            }
            (Do::Flip, Entry::Skill(id)) => {
                #[cfg(feature = "dev")]
                if let Some(b) = self.dev.board.as_mut() {
                    b.flip(id);
                }
                if !self.dev_board_on() {
                    self.progress.board.flip(id);
                }
                self.after_wear();
            }
            (Do::Flip, Entry::Cell(c)) => {
                #[cfg(feature = "dev")]
                if let Some(b) = self.dev.board.as_mut() {
                    b.flip_cell(c);
                }
                if !self.dev_board_on() {
                    self.progress.board.flip_cell(c);
                }
                self.after_wear();
            }
            (Do::Away, _) => {
                self.toast = Some((s().toast_away.to_string(), ALERT));
                self.snd(crate::sfx::Id::Deny);
            }
            _ => {}
        }
    }

    /// 끼우거나 고르기를 바꾼 뒤: 지금 판에 바로, 기록(개발 창의 판으로 한 것은 슬롯에 쓰지 않는다).
    fn after_wear(&mut self) {
        self.snd(crate::sfx::Id::UiConfirm);
        self.apply_board();
        if !self.dev_board_on() {
            self.persist();
        }
    }

    /// 묻는 창에서 "올리기"·"열기": 기술 점수로 그 칸의 다음 단(패시브 칸은 열림) — 지금 판의 월드에 바로 들어간다.
    pub(super) fn buy_skill(&mut self, i: usize) {
        let Some(&e) = ENTRIES.get(i) else { return };
        let total = self.progress.skill_points();
        if !self.progress.board.buy(e, total) {
            self.snd(crate::sfx::Id::Deny);
            return;
        }
        self.snd(crate::sfx::Id::Buy);
        // 막 배운 스킬은 빈 칸에 저절로(0166 — 처음 배운 스킬이 버튼에 바로 있다).
        if let Entry::Skill(id) = e {
            if self.progress.board.rank(e) == 1 {
                // 주무기의 갈래로 못 쓰고 부무기로는 쓰는 스킬이면 부무기의 빈 칸에(0223 V3).
                let kits = self.kits();
                let def = id.def();
                let to_sub = !def.fits(kits.main.0) && kits.sub.is_some_and(|(h, _)| def.fits(h));
                let board = self.progress.board;
                if to_sub {
                    let worn = self.progress.sub_skills.0;
                    self.progress.sub_skills = sk::Worn(board.wear_on_learn(worn, id));
                } else {
                    let worn = self.progress.skills.0;
                    self.progress.skills = sk::Worn(board.wear_on_learn(worn, id));
                }
            }
        }
        // 처음 배운 큰 기술도 저절로 끼워진다(0172 — 끼운 것이 아직 안 배운 것이면).
        if let Entry::Ult(u) = e {
            if !self.progress.board.has_ult(self.progress.ult) {
                self.progress.ult = u;
            }
        }
        let name = entry_text(e).0;
        let text = match e {
            Entry::Cell(_) | Entry::Link(_) | Entry::Vein(_) => fill(s().toast_cell, &[name]),
            Entry::Key(_) => fill(s().toast_key, &[name]),
            _ => {
                // 기술을 올린 알림은 단만 말한다 — 레벨의 "기술 점수 +1"(0167)을 같이 쓰면 점수를 **쓴** 자리에서 얻었다고 읽힌다.
                let rank = self.progress.board.rank(e).to_string();
                fill(s().toast_rank, &[name, &rank])
            }
        };
        self.toast = Some((text, ALERT));
        self.apply_board();
        self.persist();
    }

    /// 묻는 창에서 "되돌리기": 잔불을 내고 쓴 점수를 모두 돌려받는다(패시브 칸까지) — 스킬 칸은 빈 칸이 된다(0166 — 다시 배우면 그 자리에).
    pub(super) fn respec_skills(&mut self) {
        let mut great = self.progress.ult;
        if !self
            .progress
            .board
            .respec_paid(&mut self.world.embers, &mut great)
        {
            self.snd(crate::sfx::Id::Deny);
            return;
        }
        self.progress.embers = self.world.embers;
        self.progress.ult = great;
        self.snd(crate::sfx::Id::Buy);
        self.toast = Some((s().toast_respec.to_string(), ALERT));
        self.apply_board();
        self.persist();
    }

    /// 묻는 창의 글: "땅울림 1단 — 기술 점수 1. 올릴까요?"(큰 기술을 배우는 첫 단은 2) / "받아내기 — 기술 점수 1. 열까요?".
    pub(super) fn ask_skill_text(&self, i: usize) -> (String, &'static str) {
        let s = s();
        let Some(&e) = ENTRIES.get(i) else {
            return (String::new(), s.ask_level_yes);
        };
        let rank = self.progress.board.rank(e);
        let (name, price) = (entry_text(e).0, e.price(rank).to_string());
        match e {
            Entry::Cell(_) | Entry::Link(_) | Entry::Vein(_) => {
                (fill(s.ask_cell, &[name, &price]), s.ask_cell_yes)
            }
            Entry::Key(_) => (fill(s.ask_key, &[name, &price]), s.ask_key_yes),
            _ => {
                let next = (rank + 1).min(e.max()).to_string();
                (fill(s.ask_skill, &[name, &next, &price]), s.ask_level_yes)
            }
        }
    }

    /// "쓴 점수 6을 모두 돌려받는다 — 잔불 60. 되돌릴까요?".
    pub(super) fn ask_respec_text(&self) -> String {
        let b = &self.progress.board;
        fill(
            s().ask_respec,
            &[&b.spent().to_string(), &b.respec_cost().to_string()],
        )
    }

    /// 기술 탭 그림. `at` = 고른 칸(되돌리기 단추면 `RESPEC`).
    pub(super) fn draw_skills(&mut self, f: &Frame, at: usize, host: flow::Host) {
        let s = s();
        let edit = host.edits();
        let scene = Scene::Skills { host };
        let board = self.board();
        let total = self.progress.skill_points();
        let w = self.draw_tab_window(f, scene);
        self.draw_skill_grid(&w, &board, total, at);
        self.draw_skill_detail(&w, &board, total, at, edit);
        // 창 아래 줄: 왼쪽에 버튼 안내, 오른쪽 끝에 되돌리기 단추(화로). 단추는 창 밖이라 닫기 자리보다 먼저 넣는다.
        // 묻는 창이 위에 있으면 단추를 그리지 않는다 — 그 창의 안내 줄(가운데)이 같은 자리를 쓴다.
        let asking = matches!(self.scene(), Scene::Ask(_));
        let right = if edit && !asking {
            self.draw_respec(f, &w, at).x - w.s * 8.0
        } else if !asking {
            self.draw_view_note(f, &w)
        } else {
            w.panel.x + w.panel.w
        };
        self.tab_foot(
            f,
            &w,
            scene,
            right,
            &[(Act::Walk, s.verb_choose), (Act::Confirm, s.verb_confirm)],
        );
    }

    /// 왼쪽: 갈래 머리 · 줄 띠 · 칸들. 누를 자리(칸)를 채운다.
    fn draw_skill_grid(&mut self, w: &Window, board: &Board, total: u32, at: usize) {
        let s = s();
        let (k, u) = (w.k, w.s);
        // 갈래 머리: 이름(금빛) + 한 줄.
        for b in BRANCHES {
            let cx = w.at(geo::COL_W * (b.index() as f32 + 0.5), 0.0).x;
            self.ui
                .text_center(tr(b.name()), cx, w.at(0.0, 0.0).y, k, AMBER);
            let motto = self.ui.fit(tr(b.motto()), (geo::COL_W - 4.0) * u, 0.62 * k);
            self.ui
                .text_center(&motto, cx, w.at(0.0, 14.5).y, 0.62 * k, DIM);
        }
        // 굴리는 자리(0163): 갈래 머리 아래 ~ 몸의 끝. 내용(띠 셋)이 넘치면 창을 옮겨 본다 — 처음 그릴 때는 커서 칸이 보이게.
        let room = w.body_h - geo::HEAD_H;
        let (mut off, fresh) = self.view.span(room, geo::content_h(), u);
        if fresh && at < NE {
            let (y0, y1) = reveal_span(at);
            off = self.view.reveal(y0, y1);
        }
        let port = w.rect(0.0, geo::HEAD_H, geo::GRID_W, room);
        let (port_top, port_bottom) = (port.y, port.y + port.h);
        ui::clip(Some(port));
        // 줄 띠: 가는 선 + 오른쪽 끝의 이름. 잠긴 줄만 "쓴 점수 N부터 · n점 더"(2026-11-23 사장 "열린 후엔 필요 없음") — 열리는 순간
        // 선이 이름 쪽으로 걷히며 이름과 함께 사라진다(`ROW_FADE`). 결의 줄은 띠 이름 그대로. 글은 다 보일 때만(잘린 글은 그리지 않는다).
        let spent = board.spent();
        let open_mask: u8 = (0..sk::ROW_NEEDS.len())
            .filter(|&r| r != sk::VEIN_ROW && spent >= sk::ROW_NEEDS[r])
            .fold(0, |m, r| m | (1 << r));
        let (seen, opened) = &mut self.skill_rows;
        let was = seen.unwrap_or(open_mask);
        for (r, t) in opened.iter_mut().enumerate() {
            if open_mask & !was & (1 << r) != 0 {
                *t = self.clock;
            }
        }
        *seen = Some(open_mask);
        let opened_at = *opened;
        for (row, (&need, &since)) in sk::ROW_NEEDS.iter().zip(opened_at.iter()).enumerate() {
            let open = open_mask & (1 << row) != 0;
            // 열린 줄: 열린 뒤 `ROW_FADE`초 동안만 걷히며 보이고 그 뒤엔 없다.
            let fade = if open {
                1.0 - ((self.clock - since) / ROW_FADE).clamp(0.0, 1.0)
            } else {
                1.0
            };
            if fade <= 0.0 {
                continue;
            }
            let label = if row == sk::VEIN_ROW {
                s.sk_row_vein.to_string()
            } else {
                fill(
                    s.sk_row_locked,
                    &[&need.to_string(), &(need.saturating_sub(spent)).to_string()],
                )
            };
            let top = geo::band_top(row) - off;
            let lk = 0.6 * k;
            let lw = self.ui.width(&label, lk);
            let right = w.at(geo::GRID_W, 0.0).x;
            let y = w.at(0.0, top).y;
            let ink = Color {
                a: if row == sk::VEIN_ROW { 1.0 } else { 0.8 } * fade,
                ..DIM
            };
            let shown = y >= port_top - 0.5 && y + self.ui.px(lk) <= port_bottom + 0.5;
            if shown {
                self.ui.text(&label, right - lw, y, lk, ink);
            }
            let ly = (y + self.ui.px(lk) * 0.55).round();
            let line_end = if shown { right - lw - u * 5.0 } else { right };
            // 걷힘: 선의 왼끝이 이름 쪽으로 따라간다(빠르게 시작해 느리게).
            let wipe = 1.0 - fade * fade;
            let line_start = w.origin.x + (line_end - w.origin.x) * wipe;
            draw_rectangle(
                line_start,
                ly,
                line_end - line_start,
                1.0,
                Color {
                    a: 0.55 * fade,
                    ..ui::BORDER
                },
            );
        }
        // 칸.
        let worn = self.progress.skills.0;
        let sub_worn = self.progress.sub.map(|_| self.progress.sub_skills.0);
        let great = board.wearable_ult(self.progress.ult);
        let spots = spots();
        // 판의 길(0215): 칸 사이 선 — 칸보다 먼저(칸의 면이 끝을 덮는다).
        self.draw_paths(w, board, &spots, off);
        let mut cells = Vec::new();
        // 튜토리얼의 배우기 걸음(0209): 지금 살 수 있는 칸에 반짝임.
        let guide = self.tutor_menu() == Some(flow::Tab::Skills)
            && flow::tab_of(self.scene()).is_some_and(|(h, _)| h.edits());
        for (i, &e) in ENTRIES.iter().enumerate() {
            let sp = spots[i];
            let (c, half) = (w.at(sp.c.x, sp.c.y - off), sp.half * u);
            // 보이는 자리 밖의 칸은 건너뛴다(걸친 칸은 가위가 자른다).
            let reach = half + u * 8.0;
            if c.y + reach < port_top || c.y - reach > port_bottom {
                continue;
            }
            let (lk, ready) = look(board, total, e);
            let fade = if lk == Look::Locked { 0.5 } else { 1.0 };
            let top = geo::band_top(e.row()) - off;
            let pips_end = w.at(0.0, top + geo::PIPS_TOP + 3.0).y;
            draw_node(e, c, half, u, lk, (i == at).then_some(pips_end));
            let at_c = c;
            // 귀퉁이 표지(글)는 칸이 다 보일 때만.
            let whole = c.y - half - u * 3.0 >= port_top && c.y + half <= port_bottom;
            match e {
                Entry::Skill(id) => {
                    let py = w.at(0.0, top + geo::PIPS_TOP).y.round();
                    draw_pips(c.x, py, e.max(), board.rank(e), ready, false, u * 3.0, fade);
                    // 끼운 칸 번호는 배운 스킬에만(안 배운 스킬이 놓인 칸은 빈 칸이다, 0166).
                    let slot = worn.iter().position(|x| *x == id);
                    if let (Some(slot), true) = (slot.filter(|_| board.learned(id)), whole) {
                        self.node_badge(c, half, u, k, &(slot + 1).to_string(), true);
                    }
                    // 부무기의 칸(0223 V3)에 끼운 스킬: 같은 귀퉁이 아래에 테만 금빛인 번호 — 주무기의 것(금빛 면)과 갈린다.
                    let sub_slot = sub_worn.and_then(|sw| sw.iter().position(|x| *x == id));
                    if let (Some(j), true) = (sub_slot.filter(|_| board.learned(id)), whole) {
                        let drop = if slot.is_some() { u * 10.0 } else { 0.0 };
                        self.sub_badge(c, half, u, k, &(j + 1).to_string(), drop);
                    }
                    if board.rank(e) >= PICK_RANK && whole {
                        let mark = pick_text(id, board.pick[id.index()]).0;
                        self.node_badge(c, half, u, k, mark, false);
                    }
                }
                Entry::Ult(ult) => {
                    let py = w.at(0.0, top + geo::PIPS_TOP).y.round();
                    draw_pips(c.x, py, e.max(), board.rank(e), ready, true, u * 3.0, fade);
                    // 끼운 큰 기술의 표시(0172 — "큰" 글은 걷었다): 고를 것이 있을 때만(둘 이상 배웠다) 금빛 마름모.
                    // 하나만 배웠으면 그것이 끼운 것이다 — 표시가 없다.
                    if ult == great && board.ults_known() > 1 && whole {
                        self.node_badge(c, half, u, k, "◆", true);
                    }
                }
                // 연 숫자 칸(0215 D): 지금의 고르기 표지(I·II).
                Entry::Cell(c) if c.pick_b().is_some() && board.cells.has(c) && whole => {
                    let mark = cell_pick_text(c, board.cells.picked_b(c)).0;
                    self.node_badge(at_c, half, u, k, mark, false);
                }
                Entry::Cell(_) | Entry::Key(_) | Entry::Link(_) | Entry::Vein(_) => {}
            }
            if guide && ready && whole && board.rank(e) == 0 {
                self.ui.sparkle(c + vec2(half, -half), u * 4.0);
            }
            // 누를 자리는 그림보다 넉넉히(터치) — 보이는 자리 안으로만.
            let r = half + u * 3.0;
            if let Some(hit) = Rect::new(c.x - r, c.y - r, r * 2.0, r * 2.0).intersect(port) {
                cells.push((hit, Hit::Cell(0, i)));
            }
        }
        ui::clip(None);
        // 더 있다는 표시(▲▼ — 누르면 한 걸음): 격자의 오른쪽 끝, 위·아래. 누를 자리는 칸보다 먼저.
        let (above, below) = self.view.span_more();
        let mk = 0.6 * k;
        let (mw, mh) = (self.ui.width("▲", mk), self.ui.px(mk));
        let mx = w.at(geo::GRID_W, 0.0).x - mw;
        for (on, glyph, y, d) in [
            (above, "▲", port_top + u * 9.0, -1),
            (below, "▼", port_bottom - mh - u, 1),
        ] {
            if on {
                self.ui.text(glyph, mx, y, mk, AMBER);
                let pad = u * 5.0;
                self.hits.push((
                    Rect::new(mx - pad, y - pad, mw + pad * 2.0, mh + pad * 2.0),
                    Hit::Scroll(d),
                ));
            }
        }
        self.hits.extend(cells);
        self.hits.push((port, Hit::Field));
    }

    /// 판의 길(0215 B): 이어진 칸 사이 선. 둘 다 가졌으면 금빛, 가진 칸에서 아직 안 가진 칸으로 갈 수 있으면 옅은 민트, 나머지는
    /// 흐린 테 빛. 위아래 선은 위 칸의 아래(스킬·큰 기술은 단의 점 밑)에서 아래 칸의 위로, 같은 줄(다리)은 옆에서 옆으로.
    fn draw_paths(&self, w: &Window, board: &Board, spots: &[Spot; NE], off: f32) {
        let u = w.s;
        for (i, &a) in ENTRIES.iter().enumerate() {
            for (j, &b) in ENTRIES.iter().enumerate() {
                if !a.feeds(b) || (a.line() == b.line() && i > j && b.feeds(a)) {
                    continue;
                }
                let (sa, sb) = (spots[i], spots[j]);
                let (p, q) = if a.line() == b.line() {
                    let d = if sb.c.x > sa.c.x { 1.0 } else { -1.0 };
                    (
                        vec2(sa.c.x + d * sa.half, sa.c.y),
                        vec2(sb.c.x - d * sb.half, sb.c.y),
                    )
                } else {
                    let low = match a {
                        Entry::Skill(_) | Entry::Ult(_) => {
                            geo::band_top(a.row()) + geo::PIPS_TOP + 4.0
                        }
                        _ => sa.c.y + sa.half,
                    };
                    (vec2(sa.c.x, low), vec2(sb.c.x, sb.c.y - sb.half))
                };
                let (ha, hb) = (board.rank(a) > 0, board.rank(b) > 0);
                let (ink, thick) = if ha && hb {
                    (Color { a: 0.9, ..GOLD }, (u * 1.4).max(1.5))
                } else if ha || hb {
                    (
                        Color {
                            a: 0.55,
                            ..ui::MINT
                        },
                        1.0,
                    )
                } else {
                    (
                        Color {
                            a: 0.32,
                            ..ui::BORDER
                        },
                        1.0,
                    )
                };
                let (p, q) = (w.at(p.x, p.y - off), w.at(q.x, q.y - off));
                draw_line(p.x, p.y, q.x, q.y, thick, ink);
            }
        }
    }

    /// 칸 귀퉁이의 작은 표지: 끼운 칸 번호(오른쪽 위, 금빛 면) · 고르기(왼쪽 위, 어두운 면).
    fn node_badge(&self, c: Vec2, half: f32, u: f32, k: f32, text: &str, worn: bool) {
        let side = (u * 9.0).round();
        let (x, y) = if worn {
            (c.x + half - side * 0.55, c.y - half - side * 0.2)
        } else {
            (c.x - half - side * 0.45, c.y - half - side * 0.2)
        };
        let (x, y) = (x.round(), y.round());
        let (face, ink) = if worn {
            (AMBER, DARK)
        } else {
            (Color::new(0.07, 0.05, 0.09, 1.0), CREAM)
        };
        draw_rectangle(x - 1.0, y - 1.0, side + 2.0, side + 2.0, DARK);
        draw_rectangle(x, y, side, side, face);
        let tk = 0.56 * k;
        let tw = self.ui.width(text, tk);
        let ty = y + (side - self.ui.px(tk)) * 0.5;
        // 표지 글은 그림자 없이 또렷하게(작은 글 — 어두운 면 위 밝은 글, 금빛 면 위 어두운 글).
        self.ui.text(text, x + (side - tw) * 0.5, ty, tk, ink);
    }

    /// 부무기 칸에 끼운 스킬의 표지: 오른쪽 위 귀퉁이(주무기의 표지가 있으면 그 아래 `drop`) — 어두운 면 + 금빛 테·글.
    fn sub_badge(&self, c: Vec2, half: f32, u: f32, k: f32, text: &str, drop: f32) {
        let side = (u * 9.0).round();
        let (x, y) = (
            (c.x + half - side * 0.55).round(),
            (c.y - half - side * 0.2 + drop).round(),
        );
        draw_rectangle(x - 1.0, y - 1.0, side + 2.0, side + 2.0, AMBER);
        draw_rectangle(x, y, side, side, Color::new(0.07, 0.05, 0.09, 1.0));
        let tk = 0.56 * k;
        let tw = self.ui.width(text, tk);
        let ty = y + (side - self.ui.px(tk)) * 0.5;
        self.ui.text(text, x + (side - tw) * 0.5, ty, tk, AMBER);
    }

    /// 오른쪽: 고른 칸의 상세(또는 되돌리기 단추의 설명) + 아래 되돌리기 단추(화로).
    fn draw_skill_detail(&mut self, w: &Window, board: &Board, total: u32, at: usize, edit: bool) {
        let s = s();
        let (k, u) = (w.k, w.s);
        let (x0, dw) = (geo::DETAIL_X, geo::DETAIL_W);
        let text_x = x0 + geo::DETAIL_ICON + 6.0;
        let text_w = (dw - geo::DETAIL_ICON - 6.0) * u;
        // 잠깐 뜨는 알림(산 것·되돌린 것·안 되는 까닭)은 상태 줄 자리에.
        let alert = self.toast.as_ref().map(|(t, _)| t.clone());
        let entry = ENTRIES.get(at).copied();
        let (name, kind, state) = match entry {
            Some(e) => {
                let (lk, _) = look(board, total, e);
                let c = w.at(x0 + geo::DETAIL_ICON * 0.5, geo::DETAIL_ICON * 0.5 + 1.0);
                draw_node(e, c, geo::DETAIL_ICON * 0.5 * u, u, lk, None);
                (
                    entry_text(e).0,
                    kind_line(board, e),
                    state_line(board, total, e, edit),
                )
            }
            None => {
                let b = self.progress.board;
                let cost = fill(s.shop_embers, &[&b.respec_cost().to_string()]);
                let state = if b.spent() == 0 {
                    (s.board_respec_none.to_string(), DIM)
                } else if self.world.embers < b.respec_cost() {
                    (s.shop_poor.to_string(), DIM)
                } else {
                    (fill(s.sk_st_respec, &[&b.spent().to_string()]), ui::MINT)
                };
                (s.board_respec, cost, state)
            }
        };
        let name_x = if entry.is_some() { text_x } else { x0 };
        let name_w = if entry.is_some() { text_w } else { dw * u };
        let owned = entry.is_none_or(|e| board.rank(e) >= 1);
        let name = self.ui.fit(name, name_w, K_NAME * k);
        let p = w.at(name_x, 0.0);
        self.ui
            .text(&name, p.x, p.y, K_NAME * k, if owned { AMBER } else { INK });
        let kind = self.ui.fit(&kind, name_w, K_KIND * k);
        self.ui.text(&kind, p.x, w.at(0.0, 15.0).y, K_KIND * k, DIM);
        let (state, ink) = match alert {
            Some(t) => (t, AMBER),
            None => state,
        };
        let state = self.ui.fit(&state, name_w, K_KIND * k);
        self.ui
            .text(&state, p.x, w.at(0.0, 25.0).y, K_KIND * k, ink);

        // 몸: 하는 일 + 단마다 한 줄. 넘치면 덩이 단위로 굴린다.
        let width = dw * u;
        let blocks = self.detail_blocks(board, total, entry, width, k);
        let (top, full) = (geo::DETAIL_HEAD, w.body_h);
        let heights: Vec<f32> = blocks.iter().map(Block::height).collect();
        // 넘치면 맨 아래 한 줄은 굴림 표시(▲▼)의 자리로 비운다 — 표시가 단 줄의 값("1점") 위에 겹치지 않게(0223 V2·V3에서 상세에
        // 타입·쓰는 무기 줄이 늘어 넘치는 칸이 많아졌다).
        let bottom = if heights.iter().sum::<f32>() > full - top + 0.5 {
            full - PANE_MARKS
        } else {
            full
        };
        let room = bottom - top;
        // 보이는 첫 덩이: 뒤의 덩이들이 다 들어가는 가장 앞자리까지만 내려간다.
        let mut max_first = 0;
        while heights[max_first..].iter().sum::<f32>() > room + 0.5 && max_first + 1 < blocks.len()
        {
            max_first += 1;
        }
        self.scroll = self.scroll.clamp(0.0, max_first as f32);
        let first = self.scroll.round() as usize;
        let mut y = top;
        let mut shown = first;
        for b in blocks.iter().skip(first) {
            let h = b.height();
            if y + h > bottom + 0.5 {
                break;
            }
            self.draw_block(w, b, x0, y, dw);
            y += h;
            shown += 1;
        }
        if max_first > 0 {
            // 비워 둔 맨 아래 줄의 오른쪽 끝에 나란히: 위로 더 있으면 ▲, 아래로 더 있으면 ▼.
            let mw = self.ui.width("▼", 0.7 * k);
            let ax = w.at(x0 + dw, 0.0).x - mw;
            let ay = w.at(0.0, full).y - self.ui.px(0.7 * k);
            if first > 0 {
                self.ui.text("▲", ax - mw * 1.5, ay, 0.7 * k, DIM);
            }
            if shown < blocks.len() {
                self.ui.text("▼", ax, ay, 0.7 * k, DIM);
            }
        }
        self.hits.push((w.rect(x0, top, dw, full - top), Hit::Pane));
    }

    /// 되돌리기 단추(화로): 창 아래 안내 줄의 오른쪽 끝, 늘 같은 자리(0160 시안의 "되돌리기 · 잔불 55"). 그 자리를 돌려준다.
    fn draw_respec(&mut self, f: &Frame, w: &Window, at: usize) -> Rect {
        let s = s();
        let (bw, bh) = (geo::RESPEC_W * w.s, geo::RESPEC_H * w.s);
        let mid = f.oy + f.h - self.ui.px(1.0) * 1.7;
        let r = Rect::new(
            (w.panel.x + w.panel.w - bw).round(),
            (mid - bh * 0.5).round(),
            bw,
            bh,
        );
        let cost = self.progress.board.respec_cost().to_string();
        let label = fill(s.sk_respec_button, &[s.board_respec, &cost]);
        self.respec_button(r, &label, at >= NE, self.respec_ready(), w.k);
        self.hits.push((r, Hit::Cell(0, RESPEC)));
        r
    }

    /// 되돌리기 단추: 눌리는 단추의 꼴(`ui::button`) — 못 누를 때는 글이 흐리다. 고르면 밝은 테(칸의 포커스와 같은 빛).
    fn respec_button(&self, r: Rect, label: &str, sel: bool, ready: bool, k: f32) {
        let u = self.ui.u(1.0);
        draw_rectangle(
            r.x + u * 1.5,
            r.y + u * 1.5,
            r.w,
            r.h,
            Color::new(0.02, 0.01, 0.03, 0.9),
        );
        draw_rectangle(
            r.x,
            r.y,
            r.w,
            r.h,
            if sel { ui::PANEL_LIGHT } else { ui::PANEL },
        );
        let edge = if sel { CREAM } else { ui::BORDER };
        self.ui.edge(r, if sel { u * 1.5 } else { u }, edge);
        let tk = 0.8 * k;
        let label = self.ui.fit(label, r.w - u * 8.0, tk);
        let ink = match (ready, sel) {
            (true, true) => INK,
            (true, false) => DIM,
            (false, _) => Color { a: 0.45, ..DIM },
        };
        self.ui.text_center(
            &label,
            r.x + r.w * 0.5,
            r.y + (r.h - self.ui.px(tk)) * 0.5,
            tk,
            ink,
        );
    }

    /// 상세 칸의 글 덩이들(폭 `width` 화면 px에 맞춰 줄을 나눈다).
    fn detail_blocks(
        &self,
        board: &Board,
        total: u32,
        entry: Option<Entry>,
        width: f32,
        k: f32,
    ) -> Vec<Block> {
        let s = s();
        let u = width / geo::DETAIL_W;
        let wrap = |t: &str, w: f32, tk: f32| ui::wrap(&self.ui, t, w, tk * k);
        let Some(e) = entry else {
            return vec![Block::Text {
                lines: wrap(s.board_respec_d, width, K_BODY),
                k: K_BODY,
                ink: INK,
            }];
        };
        let rank = board.rank(e);
        let offer = board.offer(e, total);
        let what = match e {
            Entry::Cell(c) if c.pick_b().is_some() => s.sk_cell_pick,
            _ => entry_text(e).1,
        };
        let mut v = vec![
            Block::Text {
                lines: wrap(what, width, K_BODY),
                k: K_BODY,
                ink: INK,
            },
            Block::Gap(5.0),
        ];
        // 타입(0223 V2): 타입 있는 스킬·큰 기술은 그 타입의 빛으로 한 줄 — 무엇이 쌓이고, 차면 어떻게 되나.
        let dose = match e {
            Entry::Skill(id) => id.def().dose(),
            Entry::Ult(sk::Ult::Line) => sk::LINE_DOSE,
            _ => crate::status::NO_DOSE,
        };
        // 쓰는 무기(0223 V3): 스킬마다 쓸 수 있는 갈래 — 지금 든 주무기와 안 맞으면 밝게 알린다.
        if let Entry::Skill(id) = e {
            let fits = id.def().fits(self.kits().main.0);
            v.push(Block::Text {
                lines: wrap(&fill(s.sk_hands, &[&hands_text(id)]), width, K_BODY),
                k: K_BODY,
                ink: if fits { DIM } else { AMBER },
            });
            v.push(Block::Gap(5.0));
        }
        if let Some(i) = dose.0.slot() {
            let [r, g, b] = crate::status::INK[i];
            v.push(Block::Text {
                lines: wrap(crate::i18n::ail_desc(dose.0), width, K_BODY),
                k: K_BODY,
                ink: Color::new(r, g, b, 1.0),
            });
            v.push(Block::Gap(5.0));
        }
        // 단 줄: 익힌 단은 밝게, 다음 단은 밝은 띠 + 값, 그 뒤는 흐리게.
        let state = |n: u8| -> u8 {
            if n <= rank {
                0
            } else if n == rank + 1 {
                1
            } else {
                2
            }
        };
        let cost = |n: u8| -> Option<(String, bool)> {
            (n == rank + 1).then(|| {
                (
                    fill(s.sk_cost, &[&e.price(rank).to_string()]),
                    offer == Offer::Ready,
                )
            })
        };
        let rank_w = |n: u8| -> f32 {
            let c = cost(n).map_or(0.0, |(t, _)| self.ui.width(&t, K_RANK * k) + u * 6.0);
            width - RANK_NUM * u - c - u * 2.0
        };
        match e {
            Entry::Skill(id) => {
                for n in 1..=sk::MAX_RANK {
                    v.push(Block::Rank {
                        n,
                        lines: wrap(&rank_gives(id, n), rank_w(n), K_RANK),
                        state: state(n),
                        cost: cost(n),
                    });
                    if n == PICK_RANK {
                        let box_w = (width - RANK_NUM * u - u * 4.0) * 0.5 - u * 6.0;
                        let one = |p: Pick| {
                            let (mark, name, desc) = pick_text(id, p);
                            (
                                format!("{mark} {name}"),
                                wrap(desc, box_w, K_PICK),
                                rank >= PICK_RANK && board.pick[id.index()] == p,
                            )
                        };
                        v.push(Block::Picks {
                            boxes: [one(Pick::A), one(Pick::B)],
                            open: rank >= PICK_RANK,
                        });
                        v.push(Block::Gap(2.0));
                    }
                }
            }
            Entry::Ult(ult) => {
                for n in 1..=sk::ULT_MAX_RANK {
                    v.push(Block::Rank {
                        n,
                        lines: wrap(&ult_gives(ult, n), rank_w(n), K_RANK),
                        state: state(n),
                        cost: cost(n),
                    });
                }
            }
            // 숫자 칸(0215 D): 두 고르기를 나란히(스킬의 3단과 같은 그림) — 연 칸이면 지금 것에 금빛.
            Entry::Cell(c) if c.pick_b().is_some() => {
                let box_w = (width - RANK_NUM * u - u * 4.0) * 0.5 - u * 6.0;
                let open = board.cells.has(c);
                let one = |b: bool| {
                    let (mark, name, desc) = cell_pick_text(c, b);
                    (
                        format!("{mark} {name}"),
                        wrap(desc, box_w, K_PICK),
                        open && board.cells.picked_b(c) == b,
                    )
                };
                v.push(Block::Picks {
                    boxes: [one(false), one(true)],
                    open,
                });
            }
            // 패시브 칸·맺음·다리: 하는 일이 곧 전부 — 값은 상태 줄이 말한다.
            Entry::Cell(_) | Entry::Key(_) | Entry::Link(_) | Entry::Vein(_) => {}
        }
        v
    }

    /// 덩이 하나를 그린다(기준 자리 `x`·`y`, 폭 `dw`).
    fn draw_block(&self, w: &Window, b: &Block, x: f32, y: f32, dw: f32) {
        let (k, u) = (w.k, w.s);
        match b {
            Block::Gap(_) => {}
            Block::Text { lines, k: tk, ink } => {
                let lh = LH_BODY * (tk / K_BODY);
                for (i, line) in lines.iter().enumerate() {
                    let p = w.at(x, y + lh * i as f32);
                    self.ui.text(line, p.x, p.y, tk * k, *ink);
                }
            }
            Block::Rank {
                n,
                lines,
                state,
                cost,
            } => {
                let h = b.height();
                if *state == 1 {
                    // 다음 단: 밝은 띠 + 왼쪽 세로 줄(살 수 있으면 민트).
                    let r = w.rect(x, y, dw, h - 1.0);
                    draw_rectangle(r.x, r.y, r.w, r.h, ui::PANEL_LIGHT);
                    let bar = match cost {
                        Some((_, true)) => ui::MINT,
                        _ => ui::BORDER,
                    };
                    draw_rectangle(r.x, r.y, (u * 1.5).max(2.0), r.h, bar);
                }
                let (num_ink, ink) = match state {
                    0 => (AMBER, INK),
                    1 => (CREAM, INK),
                    _ => (Color { a: 0.75, ..DIM }, Color { a: 0.8, ..DIM }),
                };
                let p = w.at(x + 4.0, y + 1.0);
                self.ui.text(&n.to_string(), p.x, p.y, K_RANK * k, num_ink);
                for (i, line) in lines.iter().enumerate() {
                    let p = w.at(x + RANK_NUM, y + 1.0 + LH_RANK * i as f32);
                    self.ui.text(line, p.x, p.y, K_RANK * k, ink);
                }
                if let Some((t, ready)) = cost {
                    let tw = self.ui.width(t, K_RANK * k);
                    let p = w.at(x + dw - 3.0, y + 1.0);
                    let c = if *ready { ui::MINT } else { DIM };
                    self.ui.text(t, p.x - tw, p.y, K_RANK * k, c);
                }
            }
            Block::Picks { boxes, open } => {
                let h = b.height() - 2.0;
                let bw = (dw - RANK_NUM - 4.0) * 0.5;
                for (i, (name, desc, chosen)) in boxes.iter().enumerate() {
                    let bx = x + RANK_NUM + (bw + 4.0) * i as f32;
                    let r = w.rect(bx, y, bw, h);
                    // 고른 쪽: 금빛 테 + 따뜻한 면 + 왼쪽 위 채운 표지. 3단 전이면 둘 다 같은 흐린 테(미리 보기).
                    let (face, edge, ink, sub) = match (chosen, open) {
                        (true, _) => (Color { a: 0.9, ..WARM }, AMBER, AMBER, INK),
                        (false, true) => (
                            Color::new(0.0, 0.0, 0.0, 0.0),
                            Color {
                                a: 0.5,
                                ..ui::BORDER
                            },
                            Color { a: 0.75, ..DIM },
                            Color { a: 0.65, ..DIM },
                        ),
                        (false, false) => (
                            Color::new(0.0, 0.0, 0.0, 0.0),
                            Color {
                                a: 0.8,
                                ..ui::BORDER
                            },
                            INK,
                            DIM,
                        ),
                    };
                    draw_rectangle(r.x, r.y, r.w, r.h, face);
                    self.ui
                        .edge(r, if *chosen { (u * 1.2).max(2.0) } else { 1.0 }, edge);
                    let name = self.ui.fit(name, (bw - 6.0) * u, K_RANK * k);
                    let p = w.at(bx + 3.0, y + 2.0);
                    self.ui.text(&name, p.x, p.y, K_RANK * k, ink);
                    for (j, line) in desc.iter().enumerate() {
                        let p = w.at(bx + 3.0, y + 2.0 + LH_RANK + LH_PICK * j as f32);
                        self.ui.text(line, p.x, p.y, K_PICK * k, sub);
                    }
                }
            }
        }
    }

    /// 칸의 일 목록(기술 화면 위의 작은 창): 아래에 기술 화면을 그 커서 그대로 깔고, 고른 칸 곁에 되는 일만.
    pub(super) fn draw_skill_act(&mut self, f: &Frame) {
        let Some(under) = self.stack.under().copied() else {
            return;
        };
        let Scene::Skills { host } = under.scene else {
            return;
        };
        let edit = host.edits();
        let (cursor, view) = (self.cursor, self.view);
        (self.cursor, self.view) = (under.cursor, under.view);
        self.draw_layered(f, under.scene);
        (self.cursor, self.view) = (cursor, view);
        // 작은 창은 누를 자리를 제 줄과 창 밖만 남긴다(묻는 창과 같다).
        self.hits.clear();
        #[cfg(feature = "dev")]
        crate::ui::check::forget();
        let Some(&e) = ENTRIES.get(under.cursor) else {
            return;
        };
        let board = self.board();
        let rows: Vec<ui::Row> = self
            .acts_at(under.cursor, edit)
            .into_iter()
            .map(|d| ui::Row {
                dim: matches!(d, Do::Away | Do::Unfit),
                ..ui::Row::plain(act_label(&board, e, d, self.progress.sub.is_some()))
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
        // 줄의 글 앞 여백(▶ 자리)·오른쪽 여백(`ui::list`와 같은 셈) + 창 안 여백 + 두 칸의 여유(딱 맞추면 가장 긴 줄이
        // 반올림에 걸려 잘린다 — 줄이 하나뿐인 "큰 기술로 끼우기"가 그랬다).
        let pw = widest + u * 24.0 + px * 1.1;
        let ph = row_h * rows.len() as f32 + u * 8.0;
        // 고른 칸의 오른쪽에(안 들어가면 왼쪽에), 화면 안으로.
        let w = self.window_geo(f);
        let sp = spots()[under.cursor];
        let (c, half) = (w.at(sp.c.x, sp.c.y - under.view.offset()), sp.half * w.s);
        let gap = u * 7.0;
        let mut x = c.x + half + gap;
        if x + pw > f.ox + f.w - u * 6.0 {
            x = c.x - half - gap - pw;
        }
        let x = x.max(f.ox + u * 6.0).round();
        let y = (c.y - half)
            .min(w.panel.y + w.panel.h - ph - u * 4.0)
            .max(w.panel.y + u * 4.0)
            .round();
        let panel = Rect::new(x, y, pw, ph);
        // 아래 화면을 조금 눌러 작은 창이 뜨게(고른 칸은 여전히 보인다).
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
        let hits = self.ui.list(&rows, self.cursor, &mut self.view, area);
        self.push_hits(&hits);
        self.push_outside(panel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::progression::Cell;
    use sk::{Id, Ult};

    fn at(e: Entry) -> usize {
        ENTRIES.iter().position(|x| *x == e).unwrap()
    }

    /// 처음의 둘을 배운 판(0166: 새 판은 아무것도 모른다 — "배운 스킬"의 규칙은 이 판으로 본다).
    fn started() -> Board {
        Board::default().with_starters()
    }

    /// 칸의 자리: 모든 칸이 격자 안에 있고 서로 겹치지 않는다. 갈래는 왼쪽부터 흐름·맞받기·자리, 줄은 위부터 — 스킬은 그 줄의
    /// 패시브보다 위에(셋째 줄만 나란히).
    #[test]
    fn every_cell_has_its_own_spot_in_its_branch_column_and_row_band() {
        let sp = spots();
        for (i, &e) in ENTRIES.iter().enumerate() {
            let (c, h) = (sp[i].c, sp[i].half);
            assert!(h > 0.0, "{e:?} has a spot");
            if let Entry::Link(l) = e {
                // 다리 칸(0215): 두 갈래의 경계 위.
                assert_eq!(c.x, geo::COL_W * (l.left() + 1) as f32, "{e:?} boundary");
            } else if let Entry::Vein(v) = e {
                // 결의 칸(0223 11절): 판의 폭을 넷으로 나눈 자리(갈래의 기둥 밖).
                assert_eq!(c.x, sk::LANE_VEIN[v.index()], "{e:?} lane");
                assert!(c.x - h >= 0.0 && c.x + h <= geo::GRID_W, "{e:?} grid");
            } else {
                let col = geo::COL_W * e.branch().index() as f32;
                assert!(
                    c.x - h >= col && c.x + h <= col + geo::COL_W,
                    "{e:?} column"
                );
            }
            let top = geo::band_top(e.row());
            assert!(
                c.y - h >= top + 9.0 && c.y + h <= top + geo::BAND_H[e.row()],
                "{e:?} band"
            );
            for (j, o) in sp.iter().enumerate().skip(i + 1) {
                let apart = (c - o.c).abs();
                assert!(
                    apart.x >= h + o.half + 2.0 || apart.y >= h + o.half + 2.0,
                    "{e:?} overlaps {:?}",
                    ENTRIES[j]
                );
            }
        }
        // 스킬은 같은 줄·같은 갈래의 패시브보다 위에 선다(첫 줄·둘째 줄).
        let (whirl, breath) = (
            sp[at(Entry::Skill(Id::Whirl))],
            sp[at(Entry::Cell(Cell::EvenBreath))],
        );
        assert!(whirl.c.y < breath.c.y);
        assert!(whirl.half > breath.half, "skills are the bigger cells");
        assert!(geo::DETAIL_X + geo::DETAIL_W <= super::super::frame::WINDOW_W - 16.0);
    }

    /// 걷기(KB: 격자는 돌지 않는다): 가장자리에서 바깥쪽을 눌러도 그대로, 어느 칸에서 떠나도 모든 칸에 닿는다.
    /// 옆 걸음은 같은 높이의 이웃으로, 아래 걸음은 같은 갈래의 아래 칸으로.
    /// 0163: 격자는 위아래로 굴린다 — 칸마다 "보인다"고 할 구간이 내용 안에 있고 보이는 높이(몸 195 − 머리)에 들어가며,
    /// 십자로 맨 위에서 맨 아래까지 내려갔다 올라오면 창이 그 칸들을 따라간다(가장 적게).
    #[test]
    fn the_grid_scrolls_to_keep_the_cursor_cell_in_view() {
        let room = 195.0 - geo::HEAD_H;
        assert!(
            geo::content_h() > room,
            "the bands no longer fit — that is why it scrolls"
        );
        for i in 0..NE {
            let (y0, y1) = reveal_span(i);
            assert!(
                y0 >= -0.01 && y1 <= geo::content_h() + 0.01,
                "{:?}",
                ENTRIES[i]
            );
            assert!(y1 - y0 <= room, "{:?} fits the view", ENTRIES[i]);
            // 칸 자체가 그 구간 안에 있다.
            let sp = spots()[i];
            let (top, bottom) = (
                sp.c.y - sp.half - geo::HEAD_H,
                sp.c.y + sp.half - geo::HEAD_H,
            );
            assert!(top >= y0 && bottom <= y1, "{:?}", ENTRIES[i]);
        }
        let mut view = ui::ListView::default();
        let visible = |view: &ui::ListView, i: usize| {
            let (y0, y1) = reveal_span(i);
            y0 >= view.offset() - 0.01 && y1 <= view.offset() + room + 0.01
        };
        view.span(room, geo::content_h(), 2.0);
        let mut at_ = at(Entry::Skill(Id::Whirl));
        assert_eq!(view.offset(), 0.0);
        // 아래로 끝까지: 걸음마다 그 칸이 보인다. 첫 두 줄은 창이 움직이지 않고(이미 보인다) 셋째 줄에서 내려간다.
        let mut moved_at = None;
        for step in 0..14 {
            let next = walk(at_, 0, 1, false, 0);
            if next == at_ {
                break;
            }
            at_ = next;
            let before = view.offset();
            let (y0, y1) = reveal_span(at_);
            view.reveal(y0, y1);
            assert!(visible(&view, at_), "{:?}", ENTRIES[at_]);
            if view.offset() != before && moved_at.is_none() {
                moved_at = Some((step, ENTRIES[at_].row()));
            }
        }
        assert_eq!(ENTRIES[at_].row(), sk::KEY_ROW, "down to the keys");
        assert!(
            moved_at.is_some_and(|m| m.1 >= 1),
            "the veins and the first band need no scroll: {moved_at:?}"
        );
        assert_eq!(
            view.offset(),
            geo::content_h() - room,
            "the bottom of the board"
        );
        // 다시 위로: 맨 위 칸에 서면 창도 맨 위(갈래 머리 아래 첫 이름표가 보인다).
        for _ in 0..14 {
            at_ = walk(at_, 0, -1, false, 0);
            let (y0, y1) = reveal_span(at_);
            view.reveal(y0, y1);
            assert!(visible(&view, at_));
        }
        assert_eq!(
            ENTRIES[at_].row(),
            sk::VEIN_ROW,
            "the veins are the top band"
        );
        assert_eq!(view.offset(), 0.0);
    }

    #[test]
    fn walking_follows_the_layout_never_wraps_and_reaches_every_cell() {
        let whirl = at(Entry::Skill(Id::Whirl));
        let cut = at(Entry::Skill(Id::Cut));
        let brace = at(Entry::Skill(Id::Brace));
        let gather = at(Entry::Skill(Id::Gather));
        assert_eq!(walk(whirl, 1, 0, true, 0), cut);
        assert_eq!(walk(cut, 1, 0, true, 0), brace);
        assert_eq!(walk(brace, 1, 0, true, 0), gather);
        assert_eq!(walk(gather, -1, 0, true, 0), brace);
        assert_eq!(walk(cut, -1, 0, true, 0), whirl);
        // 가장자리: 왼쪽 끝에서 왼쪽·맨 위에서 위·맨 아래에서 아래는 그대로(돌지 않는다).
        assert_eq!(walk(whirl, -1, 0, true, 0), whirl);
        let flurry = at(Entry::Ult(Ult::Flurry));
        let measure = at(Entry::Key(crate::progression::Key::Measure));
        // 결의 줄(0223 11절)은 맨 위(2026-11-23 사장 "속성 패시브가 가장 위쪽에") — 첫 줄 위로 오르면 거기, 거기서 위는 그대로.
        // 맨 아래는 맺음 줄.
        let ember = at(Entry::Vein(crate::progression::Vein::Ember));
        let rift = at(Entry::Vein(crate::progression::Vein::Rift));
        let up = walk(whirl, 0, -1, true, 0);
        assert!(matches!(ENTRIES[up], Entry::Vein(_)), "{:?}", ENTRIES[up]);
        assert_eq!(walk(ember, 0, -1, true, 0), ember);
        assert!(matches!(ENTRIES[walk(ember, 0, 1, true, 0)].row(), 0));
        assert_eq!(walk(measure, 0, 1, false, 0), measure);
        assert_eq!(walk(rift, 1, 0, false, 0), rift);
        assert_eq!(
            walk(rift, 1, 0, true, 0),
            rift,
            "right ends on the board (0261)"
        );
        assert_eq!(walk(flurry, -1, 0, true, 0), flurry);
        // 0214: 큰 기술 줄 아래로 넷째 줄(새 스킬 → 그 아래 패시브), 그 아래 맺음.
        let swallow = at(Entry::Skill(Id::Swallow));
        let held = at(Entry::Cell(Cell::HeldBreath));
        assert_eq!(walk(flurry, 0, 1, true, 0), swallow);
        assert_eq!(walk(swallow, 0, 1, true, 0), held);
        assert_eq!(walk(held, 0, 1, true, 0), measure);
        assert_eq!(walk(measure, 0, -1, true, 0), held);
        assert_eq!(
            walk(measure, 1, 0, true, 0),
            at(Entry::Key(crate::progression::Key::Edge))
        );
        // 아래로: 스킬 → 그 아래 패시브 → 다음 줄의 스킬 → … → 큰 기술.
        let breath = at(Entry::Cell(Cell::EvenBreath));
        assert_eq!(walk(whirl, 0, 1, true, 0), breath);
        assert_eq!(walk(breath, 0, -1, true, 0), whirl);
        let quake = at(Entry::Skill(Id::Quake));
        assert_eq!(walk(breath, 0, 1, true, 0), quake);
        let closing = at(Entry::Cell(Cell::ClosingBlow));
        assert_eq!(walk(quake, 0, 1, true, 0), closing);
        assert_eq!(walk(closing, 0, 1, true, 0), flurry);
        assert_eq!(
            walk(flurry, 1, 0, true, 0),
            at(Entry::Cell(Cell::FinalMeasure))
        );
        // 되돌리기 단추(창 아래 오른쪽 — 화로에서만): 맨 아래에서 **아래**로만 닿는다 — 오른쪽 걸음은 격자에서 끝난다(0261).
        // 단추에서 위는 내려오기 전의 칸(`back`)으로, 옆·아래는 그대로.
        // 오른쪽 아래 칸은 맺음 줄의 끝(결의 줄이 맨 위로 간 뒤, 2026-11-23).
        let rooted = at(Entry::Key(crate::progression::Key::Rooted));
        let step = rooted;
        assert_eq!(walk(step, 1, 0, true, 0), step);
        assert_eq!(walk(step, 0, 1, true, 0), RESPEC);
        assert_eq!(walk(measure, 0, 1, true, 0), RESPEC);
        assert_eq!(walk(step, 1, 0, false, 0), step, "no button in the pause");
        assert_eq!(walk(step, 0, 1, false, 0), step);
        assert_eq!(
            walk(gather, 1, 0, true, 0),
            gather,
            "the right edge is the end"
        );
        assert_eq!(
            walk(RESPEC, 0, -1, true, measure),
            measure,
            "back where it came from"
        );
        assert_eq!(walk(RESPEC, 0, -1, true, step), step);
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1)] {
            assert_eq!(walk(RESPEC, dx, dy, true, measure), RESPEC);
        }
        // 어느 칸에서도 오른쪽으로는 단추에 닿지 않는다.
        for i in 0..NE {
            assert_ne!(walk(i, 1, 0, true, 0), RESPEC, "{:?}", ENTRIES[i]);
        }
        assert_eq!(walk(whirl, 0, 0, true, 0), whirl);
        // 모든 칸에 닿는다(네 쪽 걸음으로) — 어느 칸에서 떠나도.
        for start in 0..NE {
            let mut seen = vec![false; NE + 1];
            let mut todo = vec![start];
            seen[start] = true;
            while let Some(i) = todo.pop() {
                for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let j = walk(i, dx, dy, true, 0);
                    assert!(j <= RESPEC);
                    if !seen[j] {
                        seen[j] = true;
                        todo.push(j);
                    }
                }
            }
            assert!(seen.iter().all(|s| *s), "from {:?}", ENTRIES[start]);
        }
    }

    /// 확인의 목록에는 되는 일만: 화로에서 살 수 있으면 올리기·열기가 맨 앞(으뜸), 배운 스킬은 끼우지 않은 칸에 끼우기,
    /// 3단부터 고르기 바꾸기(화로에서만). 멈춤에서는 끼우기만 — 점수가 있어 올릴 수 있는 칸이면 끝에 "쉼터 화로에서" 줄(0173),
    /// 잠긴 칸·점수가 모자란 칸은 빈 목록(확인 = 까닭 알림).
    #[test]
    fn the_action_list_holds_only_what_applies_with_the_primary_row_first() {
        let worn = sk::START;
        let kits = Kits {
            main: (Grip::Duel, worn),
            sub: None,
        };
        let b = started();
        let acts = |b: &Board, total, e, edit| node_acts(b, total, &kits, Ult::Upend, e, edit);
        // 0166: 새 판에서는 끼워 둔 자리의 스킬도 안 배운 것 — 화로에서 배우기만, 멈춤에서는 "쉼터 화로에서"만(0173).
        let fresh = Board::default();
        assert_eq!(acts(&fresh, 2, Entry::Skill(Id::Whirl), true), [Do::Buy]);
        assert_eq!(acts(&fresh, 2, Entry::Skill(Id::Whirl), false), [Do::Away]);
        // 점수가 없으면 멈춤에서도 화로에서도 아무것도(까닭은 "점수가 모자란다").
        assert!(acts(&fresh, 0, Entry::Skill(Id::Whirl), false).is_empty());
        // 휘돌이(칸 1에 끼움, 1단): 화로에서 점수가 있으면 올리기 + 칸 2에 끼우기.
        assert_eq!(
            acts(&b, 3, Entry::Skill(Id::Whirl), true),
            [Do::Buy, Do::Wear(1)]
        );
        assert_eq!(acts(&b, 0, Entry::Skill(Id::Whirl), true), [Do::Wear(1)]);
        assert_eq!(
            acts(&b, 3, Entry::Skill(Id::Whirl), false),
            [Do::Wear(1), Do::Away]
        );
        assert_eq!(acts(&b, 0, Entry::Skill(Id::Whirl), false), [Do::Wear(1)]);
        // 안 배운 스킬: 화로에서는 올리기(배움)만, 멈춤에서는 "쉼터 화로에서"만.
        assert_eq!(acts(&b, 3, Entry::Skill(Id::Gather), true), [Do::Buy]);
        assert_eq!(acts(&b, 3, Entry::Skill(Id::Gather), false), [Do::Away]);
        // 잠긴 줄: 아무것도.
        assert!(acts(&b, 30, Entry::Skill(Id::Quake), true).is_empty());
        assert!(acts(&b, 30, Entry::Ult(Ult::Flurry), true).is_empty());
        // 패시브: 화로에서 열기만(멈춤에서는 "쉼터 화로에서") — 연 뒤에는 아무것도. 길이 끊긴 칸(0215)은 아무것도.
        let flow = Entry::Cell(Cell::FlowingBeat);
        assert_eq!(acts(&b, 3, flow, true), [Do::Buy]);
        assert_eq!(acts(&b, 3, flow, false), [Do::Away]);
        assert!(
            acts(&b, 3, Entry::Cell(Cell::Catch), true).is_empty(),
            "cut off"
        );
        let mut open = b;
        assert!(open.buy(flow, 3));
        assert!(acts(&open, 3, flow, true).is_empty());
        // 숫자 칸(0215): 연 뒤에는 화로에서 고르기 바꾸기.
        let breath = Entry::Cell(Cell::EvenBreath);
        assert!(open.buy(breath, 9));
        assert_eq!(acts(&open, 3, breath, true), [Do::Flip]);
        assert!(acts(&open, 3, breath, false).is_empty());
        // 끼우지 않은 배운 스킬: 두 칸 다. 3단부터 고르기 바꾸기(화로에서만).
        let mut m = started();
        m.rank[Id::Gather.index()] = 3;
        assert_eq!(
            acts(&m, 9, Entry::Skill(Id::Gather), true),
            [Do::Buy, Do::Wear(0), Do::Wear(1), Do::Flip]
        );
        assert_eq!(
            acts(&m, 9, Entry::Skill(Id::Gather), false),
            [Do::Wear(0), Do::Wear(1), Do::Away]
        );
        // 다 익힌 스킬: 올리기는 없다.
        m.rank[Id::Gather.index()] = 5;
        assert_eq!(
            acts(&m, 9, Entry::Skill(Id::Gather), true),
            [Do::Wear(0), Do::Wear(1), Do::Flip]
        );
        // 큰 기술: 끼운 것(쟁반 뒤집기)에는 끼우기가 없고, 배운 다른 것에는 있다.
        let all = Board::maxed(Pick::A);
        assert!(acts(&all, 99, Entry::Ult(Ult::Upend), true).is_empty());
        assert_eq!(acts(&all, 99, Entry::Ult(Ult::Line), false), [Do::WearUlt]);
        assert!(node_acts(&all, 99, &kits, Ult::Line, Entry::Ult(Ult::Line), false).is_empty());
        // 0223 V3: 스킬은 그 갈래의 무기 칸에만 — 한손 검으로는 땅울림(양손)을 끼울 줄이 없고 까닭 줄이 선다. 망치를 부무기로
        // 들면 부무기의 칸(2·3)에 끼우는 줄이 서고, 두 무기에 다 되는 스킬은 네 칸 모두.
        assert_eq!(
            node_acts(&all, 99, &kits, Ult::Upend, Entry::Skill(Id::Quake), false),
            [Do::Unfit]
        );
        let both = Kits {
            main: (Grip::Duel, worn),
            sub: Some((Grip::Heavy, [Id::Quake, Id::Gather])),
        };
        let at = |id| node_acts(&all, 99, &both, Ult::Upend, Entry::Skill(id), false);
        assert_eq!(
            at(Id::Quake),
            [Do::Wear(3)],
            "already in the hammer's first slot"
        );
        assert_eq!(
            at(Id::Brace),
            [Do::Wear(0), Do::Wear(1), Do::Wear(2), Do::Wear(3)]
        );
        assert_eq!(at(Id::Swallow), [Do::Wear(0), Do::Wear(1)], "one hand only");
    }

    /// 끼우기: 다른 칸에 끼어 있던 스킬이면 맞바꾼다 — 같은 스킬이 두 칸에 서지 않는다.
    #[test]
    fn wearing_a_skill_swaps_it_out_of_the_other_slot() {
        let worn = [Id::Whirl, Id::Cut];
        assert_eq!(wear_skill(worn, 0, Id::Gather), [Id::Gather, Id::Cut]);
        assert_eq!(wear_skill(worn, 1, Id::Gather), [Id::Whirl, Id::Gather]);
        assert_eq!(wear_skill(worn, 1, Id::Whirl), [Id::Cut, Id::Whirl], "swap");
        assert_eq!(wear_skill(worn, 0, Id::Cut), [Id::Cut, Id::Whirl], "swap");
        assert_eq!(wear_skill(worn, 0, Id::Whirl), worn);
    }

    /// 밝기 넷: 익힌 칸은 점수가 없어도 익힌 빛(더 올릴 수 있으면 다음 점이 민트), 안 익힌 칸은 살 수 있음 / 점수 모자람 / 줄 잠김.
    #[test]
    fn a_cell_looks_owned_ready_short_or_locked() {
        // 0166: 새 판의 휘돌이는 살 수 있는 칸일 뿐이다.
        let fresh = Board::default();
        assert_eq!(
            look(&fresh, 2, Entry::Skill(Id::Whirl)),
            (Look::Ready, true)
        );
        assert_eq!(
            look(&fresh, 0, Entry::Skill(Id::Whirl)),
            (Look::Short, false)
        );
        let b = started();
        assert_eq!(look(&b, 5, Entry::Skill(Id::Whirl)), (Look::Owned, true));
        assert_eq!(look(&b, 0, Entry::Skill(Id::Whirl)), (Look::Owned, false));
        assert_eq!(look(&b, 5, Entry::Skill(Id::Gather)), (Look::Ready, true));
        assert_eq!(look(&b, 0, Entry::Skill(Id::Gather)), (Look::Short, false));
        assert_eq!(look(&b, 5, Entry::Skill(Id::Quake)), (Look::Locked, false));
        assert_eq!(
            look(&b, 5, Entry::Cell(Cell::EvenBreath)),
            (Look::Ready, true)
        );
        assert_eq!(
            look(&b, 5, Entry::Cell(Cell::FinalMeasure)),
            (Look::Locked, false)
        );
        let all = Board::maxed(Pick::A);
        for e in ENTRIES {
            // 0214: 켜지 않은 맺음은 바꿀 수 있을 뿐(살 것이 아니다 — 흐리게).
            let want = match e {
                Entry::Key(k) if all.cells.key != Some(k) => (Look::Short, false),
                _ => (Look::Owned, false),
            };
            assert_eq!(look(&all, 99, e), want, "{e:?}");
        }
    }

    /// 상태 줄: 화로에서는 살 수 있으면 값과 함께, 멈춤에서는 어디서 올리는지. 잠긴 칸은 몇 점 더, 다 익힌 칸·연 칸은 그렇다고.
    #[test]
    fn the_state_line_says_what_can_be_done_or_why_not() {
        let s = s();
        let b = started();
        let whirl = Entry::Skill(Id::Whirl);
        assert_eq!(
            state_line(&b, 5, whirl, true).0,
            fill(s.sk_st_raise, &["1"])
        );
        assert_eq!(state_line(&b, 5, whirl, false).0, s.sk_st_view);
        assert_eq!(
            state_line(&b, 0, whirl, true).0,
            fill(s.sk_st_short, &["1"])
        );
        assert_eq!(
            state_line(&b, 5, Entry::Skill(Id::Quake), true).0,
            fill(s.sk_st_locked, &["2"]),
            "two of the four are already spent"
        );
        let flow = Entry::Cell(Cell::FlowingBeat);
        assert_eq!(state_line(&b, 5, flow, true).0, fill(s.sk_st_open, &["1"]));
        // 0215: 길이 끊긴 칸 · 맺음은 그 갈래에 더.
        let catch = Entry::Cell(Cell::Catch);
        assert_eq!(state_line(&b, 5, catch, true).0, s.sk_st_cut);
        let all = Board::maxed(Pick::A);
        assert_eq!(state_line(&all, 99, whirl, true).0, s.board_full);
        assert_eq!(state_line(&all, 99, catch, true).0, s.sk_st_open_cell);
        let link = Entry::Link(crate::progression::Link::ParryStep);
        assert_eq!(state_line(&all, 99, link, true).0, s.sk_st_open_cell);
        let mut keyless = all;
        keyless.cells.key = None;
        keyless.rank[Id::Brace.index()] = 0;
        keyless.rank[Id::Echo.index()] = 0;
        keyless.rank[Id::Bash.index()] = 0;
        keyless.ult[Ult::Line.index()] = 0;
        let edge = Entry::Key(crate::progression::Key::Edge);
        // 맞받기에 남은 것은 패시브 일곱.
        assert_eq!(keyless.branch_spent(crate::progression::Branch::Riposte), 7);
        assert_eq!(
            state_line(&keyless, 99, edge, true).0,
            fill(s.sk_st_branch, &["3"])
        );
        // 큰 기술을 배우는 첫 단은 2점.
        let mut open = Board::default();
        open.rank[Id::Whirl.index()] = 5;
        open.rank[Id::Cut.index()] = 5;
        // 줄 긋기까지의 길은 이미 있다(되울림 밑의 모은 맞받기 — 길을 묻지 않는 가진 칸).
        open.cells.open_cell(Cell::GatheredRiposte);
        assert_eq!(
            state_line(&open, 13, Entry::Ult(Ult::Line), true).0,
            fill(s.sk_st_raise, &["2"])
        );
        assert_eq!(
            state_line(&open, 12, Entry::Ult(Ult::Line), true).0,
            fill(s.sk_st_short, &["2"])
        );
    }

    /// 단마다 주는 것(표의 수에서): 2단 = 피해(끌어모으기는 휘청, 되울림은 이어지는 시간), 3단 = 고르기, 4단 = 제 치수(모양마다 이름),
    /// 5단 = 피해 더·대기·맺음. 큰 기술은 저마다의 수.
    #[test]
    fn each_rank_line_says_what_that_rank_gives() {
        let s = s();
        assert_eq!(rank_gives(Id::Whirl, 1), s.sk_rank_learn);
        assert_eq!(rank_gives(Id::Whirl, 2), fill(s.rk_damage, &["15"]));
        assert_eq!(rank_gives(Id::Gather, 2), fill(s.rk_stagger, &["0.3"]));
        assert_eq!(rank_gives(Id::Echo, 2), fill(s.rk_secs, &["1"]));
        for id in sk::IDS {
            assert_eq!(rank_gives(id, 3), s.sk_rank_pick, "{id:?}");
            let (a, b) = (pick_text(id, Pick::A), pick_text(id, Pick::B));
            assert_ne!(a.1, b.1, "{id:?}: two different picks");
            assert!(!a.2.is_empty() && !b.2.is_empty());
            let five = rank_gives(id, 5);
            assert!(five.contains(cap_text(id)), "{id:?}: {five}");
            assert!(five.contains(&fill(s.rk_cooldown, &["10"])), "{id:?}");
            assert_eq!(
                five.contains(&fill(s.rk_damage_more, &["15"])),
                id.def().damage > 0,
                "{id:?}: damage only where there is damage"
            );
        }
        for (id, t) in [
            (Id::Whirl, s.rk_radius),
            (Id::Cut, s.rk_length),
            (Id::Gather, s.rk_range),
            (Id::Quake, s.rk_arc),
            (Id::Brace, s.rk_radius),
            (Id::Throw, s.rk_range),
        ] {
            assert_eq!(rank_gives(id, 4), fill(t, &["20"]), "{id:?}");
        }
        assert_eq!(ult_gives(Ult::Upend, 2), fill(s.rk_stagger, &["0.6"]));
        assert_eq!(ult_gives(Ult::Upend, 3), fill(s.rk_slam, &["50"]));
        assert_eq!(ult_gives(Ult::Flurry, 2), fill(s.rk_flurry_secs, &["6.5"]));
        assert_eq!(
            ult_gives(Ult::Flurry, 3),
            fill(s.rk_flurry_kill, &["0.5", "3"])
        );
        assert_eq!(ult_gives(Ult::Line, 2), fill(s.rk_line_len, &["15"]));
        assert_eq!(ult_gives(Ult::Line, 3), fill(s.rk_line_dmg, &["60"]));
        // 갈래·종류 줄: 대기는 지금 단의 값(5단은 ×0.9).
        let mut b = Board::default();
        assert_eq!(
            kind_line(&b, Entry::Skill(Id::Whirl)),
            fill(s.sk_kind_skill, &[s.tree_beat, "7"])
        );
        b.rank[Id::Whirl.index()] = 5;
        assert!(kind_line(&b, Entry::Skill(Id::Whirl)).contains("6.3"));
        assert_eq!(
            kind_line(&b, Entry::Ult(Ult::Upend)),
            fill(s.sk_kind_ult, &[s.tree_footing])
        );
        assert_eq!(
            kind_line(&b, Entry::Cell(Cell::TurnItBack)),
            fill(s.sk_kind_cell, &[s.tree_riposte])
        );
        assert_eq!(
            kind_line(&b, Entry::Cell(Cell::Catch)),
            fill(s.sk_kind_cell_pick, &[s.tree_riposte])
        );
        assert_eq!(
            kind_line(&b, Entry::Link(crate::progression::Link::ParryStep)),
            fill(s.sk_kind_link, &[s.tree_riposte, s.tree_footing])
        );
        // 칸마다 이름과 하는 일이 있고 이름은 모두 다르다.
        let names: Vec<&str> = ENTRIES.iter().map(|e| entry_text(*e).0).collect();
        for (i, n) in names.iter().enumerate() {
            assert!(!n.is_empty() && !entry_text(ENTRIES[i]).1.is_empty());
            assert!(!names[i + 1..].contains(n), "{n} twice");
        }
    }
}
