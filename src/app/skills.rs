//! 기술 화면(0161 — 옛 나무 창·기술 판·멈춤의 기술 창을 하나로, 0160 시안 1의 방향). 화로와 멈춤이 같은 화면을 연다:
//! 화로(`edit`)에서는 올리기·열기·고르기 바꾸기·되돌리기까지, 멈춤에서는 읽기와 끼우기만.
//!
//! - 맨 위: **판 줄**(0270 C2 — 몸 · 결투 · 방패 · 망치 · 지팡이 · 원거리, `plates::PLATES`). 판 하나만 보인다 — 열면 든 무기의 판
//!   (맨손은 결투의 꼴). 맨 윗줄에서 위 = 판 줄, 판 줄에서 옆 = 판 바꾸기, 아래 = 격자로. 든 무기·부무기의 판에 금빛 표지.
//! - 왼쪽: 몸의 판 = 갈래 셋(흐름·맞받기·자리)의 패시브·다리·맺음(줄은 쓴 점수 합), 무기 판 = 대표 타입 두 갈래 + 가운데 × 줄 셋
//!   (첫 스킬 / 둘째 스킬 / 결과 큰 기술 — 줄은 그 판에 쓴 점수, `plates::offer_on`)의 **칸 그림**. 스킬 = 네모 + 단의 점 다섯,
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
use crate::plates::{self, Col, Plate, Seat, PLATES};
use crate::progression::{Cell, BRANCHES};
use crate::skills::{
    self as sk, Board, Entry, Form, Offer, Pick, ENTRIES, NE, PICK_RANK, SIZE_RANK,
};
use crate::weapons::Grip;

/// 화면의 알림(산 것·되돌린 것)이 상태 줄에 떠 있는 초.
const ALERT: f32 = 1.4;
/// 상세 칸이 넘칠 때 굴림 표시(▲▼)에 비워 두는 맨 아래 줄의 높이(창 단위).
const PANE_MARKS: f32 = 9.0;
/// 커서의 자리: 판의 칸들(`ENTRIES`의 자리) 다음이 "되돌리기" 단추, 그다음이 판 줄(0270 C2 — 거기서 옆 걸음 = 판 바꾸기).
pub(super) const RESPEC: usize = NE;
pub(super) const STRIP: usize = NE + 1;

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
        sk::Id::Stitch => (s.skill_stitch, s.skill_stitch_d),
        sk::Id::Ram => (s.skill_ram, s.skill_ram_d),
        sk::Id::Toll => (s.skill_toll, s.skill_toll_d),
        sk::Id::Swat => (s.skill_swat, s.skill_swat_d),
        sk::Id::Ripple => (s.skill_ripple, s.skill_ripple_d),
        sk::Id::Snow => (s.skill_snow, s.skill_snow_d),
        sk::Id::Flare => (s.skill_flare, s.skill_flare_d),
        sk::Id::Snare => (s.skill_snare, s.skill_snare_d),
        sk::Id::Fuse => (s.skill_fuse, s.skill_fuse_d),
        sk::Id::Mine => (s.skill_mine, s.skill_mine_d),
        sk::Id::Skewer => (s.skill_skewer, s.skill_skewer_d),
        sk::Id::Pole => (s.skill_pole, s.skill_pole_d),
        sk::Id::Pin => (s.skill_pin, s.skill_pin_d),
        sk::Id::Pommel => (s.skill_pommel, s.skill_pommel_d),
        sk::Id::Shove => (s.skill_shove, s.skill_shove_d),
        sk::Id::Rally => (s.skill_rally, s.skill_rally_d),
        sk::Id::Slip => (s.skill_slip, s.skill_slip_d),
    }
}

/// 큰 기술 이름·설명(글 표).
pub(super) fn ult_text(u: sk::Ult) -> (&'static str, &'static str) {
    let s = s();
    match u {
        sk::Ult::Upend => (s.ult_upend, s.ult_upend_d),
        sk::Ult::Flurry => (s.ult_flurry, s.ult_flurry_d),
        sk::Ult::Line => (s.ult_line, s.ult_line_d),
        sk::Ult::Bell => (s.ult_bell, s.ult_bell_d),
        sk::Ult::Blizzard => (s.ult_blizzard, s.ult_blizzard_d),
        sk::Ult::Rain => (s.ult_rain, s.ult_rain_d),
        sk::Ult::Gale => (s.ult_gale, s.ult_gale_d),
        sk::Ult::Ambush => (s.ult_ambush, s.ult_ambush_d),
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

/// 속성 패시브의 고르기(0274 ② — 울림): 표지·이름·한 줄 — I = 결 자체, II = `Vein::pick_b`.
pub(super) fn vein_pick_text(
    v: crate::progression::Vein,
    b: bool,
) -> (&'static str, &'static str, &'static str) {
    let s = s();
    match (b, v.pick_b()) {
        (true, Some((name, desc))) => (s.pick_mark_b, tr(name), tr(desc)),
        _ => (s.pick_mark_a, tr(v.name()), tr(v.desc())),
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
        (sk::Id::Stitch, Pick::A) => (s.pick_stitch_a, s.pick_stitch_a_d),
        (sk::Id::Stitch, Pick::B) => (s.pick_stitch_b, s.pick_stitch_b_d),
        (sk::Id::Ram, Pick::A) => (s.pick_ram_a, s.pick_ram_a_d),
        (sk::Id::Ram, Pick::B) => (s.pick_ram_b, s.pick_ram_b_d),
        (sk::Id::Toll, Pick::A) => (s.pick_toll_a, s.pick_toll_a_d),
        (sk::Id::Toll, Pick::B) => (s.pick_toll_b, s.pick_toll_b_d),
        (sk::Id::Swat, Pick::A) => (s.pick_swat_a, s.pick_swat_a_d),
        (sk::Id::Swat, Pick::B) => (s.pick_swat_b, s.pick_swat_b_d),
        (sk::Id::Ripple, Pick::A) => (s.pick_ripple_a, s.pick_ripple_a_d),
        (sk::Id::Ripple, Pick::B) => (s.pick_ripple_b, s.pick_ripple_b_d),
        (sk::Id::Snow, Pick::A) => (s.pick_snow_a, s.pick_snow_a_d),
        (sk::Id::Snow, Pick::B) => (s.pick_snow_b, s.pick_snow_b_d),
        (sk::Id::Flare, Pick::A) => (s.pick_flare_a, s.pick_flare_a_d),
        (sk::Id::Flare, Pick::B) => (s.pick_flare_b, s.pick_flare_b_d),
        (sk::Id::Snare, Pick::A) => (s.pick_snare_a, s.pick_snare_a_d),
        (sk::Id::Snare, Pick::B) => (s.pick_snare_b, s.pick_snare_b_d),
        (sk::Id::Fuse, Pick::A) => (s.pick_fuse_a, s.pick_fuse_a_d),
        (sk::Id::Fuse, Pick::B) => (s.pick_fuse_b, s.pick_fuse_b_d),
        (sk::Id::Mine, Pick::A) => (s.pick_mine_a, s.pick_mine_a_d),
        (sk::Id::Mine, Pick::B) => (s.pick_mine_b, s.pick_mine_b_d),
        (sk::Id::Skewer, Pick::A) => (s.pick_skewer_a, s.pick_skewer_a_d),
        (sk::Id::Skewer, Pick::B) => (s.pick_skewer_b, s.pick_skewer_b_d),
        (sk::Id::Pole, Pick::A) => (s.pick_pole_a, s.pick_pole_a_d),
        (sk::Id::Pole, Pick::B) => (s.pick_pole_b, s.pick_pole_b_d),
        (sk::Id::Pin, Pick::A) => (s.pick_pin_a, s.pick_pin_a_d),
        (sk::Id::Pin, Pick::B) => (s.pick_pin_b, s.pick_pin_b_d),
        (sk::Id::Pommel, Pick::A) => (s.pick_pommel_a, s.pick_pommel_a_d),
        (sk::Id::Pommel, Pick::B) => (s.pick_pommel_b, s.pick_pommel_b_d),
        (sk::Id::Shove, Pick::A) => (s.pick_shove_a, s.pick_shove_a_d),
        (sk::Id::Shove, Pick::B) => (s.pick_shove_b, s.pick_shove_b_d),
        (sk::Id::Rally, Pick::A) => (s.pick_rally_a, s.pick_rally_a_d),
        (sk::Id::Rally, Pick::B) => (s.pick_rally_b, s.pick_rally_b_d),
        (sk::Id::Slip, Pick::A) => (s.pick_slip_a, s.pick_slip_a_d),
        (sk::Id::Slip, Pick::B) => (s.pick_slip_b, s.pick_slip_b_d),
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
        sk::Id::Stitch => s.cap_stitch,
        sk::Id::Ram => s.cap_ram,
        sk::Id::Toll => s.cap_toll,
        sk::Id::Swat => s.cap_swat,
        sk::Id::Ripple => s.cap_ripple,
        sk::Id::Snow => s.cap_snow,
        sk::Id::Flare => s.cap_flare,
        sk::Id::Snare => s.cap_snare,
        sk::Id::Fuse => s.cap_fuse,
        sk::Id::Mine => s.cap_mine,
        sk::Id::Skewer => s.cap_skewer,
        sk::Id::Pole => s.cap_pole,
        sk::Id::Pin => s.cap_pin,
        sk::Id::Pommel => s.cap_pommel,
        sk::Id::Shove => s.cap_shove,
        sk::Id::Rally => s.cap_rally,
        sk::Id::Slip => s.cap_slip,
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
    // 재기(0274): 단이 올리는 것은 피해가 아니라 도는 체력.
    let mends = matches!(d.form, Form::Rally { .. });
    match rank {
        0 | 1 => s.sk_rank_learn.to_string(),
        2 if mends => fill(s.rk_heal, &[&pct(sk::RANK2_DAMAGE)]),
        // 되울림의 2단은 이어지는 시간(0158 G2).
        2 if matches!(d.form, Form::Echo { .. }) => fill(s.rk_secs, &[&num(sk::RANK2_SECS)]),
        2 if harmless => fill(s.rk_stagger, &[&format!("{:.1}", sk::RANK2_STAGGER)]),
        2 => fill(s.rk_damage, &[&pct(sk::RANK2_DAMAGE)]),
        r if r == PICK_RANK => s.sk_rank_pick.to_string(),
        r if r == SIZE_RANK => {
            let t = match d.form {
                Form::Ring { .. }
                | Form::Brace { .. }
                | Form::Echo { .. }
                | Form::Field { .. }
                | Form::Bell { .. }
                | Form::Wave { .. }
                | Form::Drift { .. }
                | Form::Trap { .. }
                | Form::Rally { .. } => s.rk_radius,
                Form::Lane { .. } | Form::Erupt { .. } => s.rk_length,
                Form::Pull { .. }
                | Form::Throw { .. }
                | Form::Hop { .. }
                | Form::Stab { .. }
                | Form::Slip { .. } => s.rk_range,
                Form::Cone { .. } | Form::Bash { .. } => s.rk_arc,
            };
            fill(t, &[&pct(sk::RANK4_SIZE)])
        }
        _ => {
            let mut parts = Vec::new();
            // 5단의 피해(재기는 도는 체력)는 2단의 몫에 더한 것(합 +30%).
            let more = pct(1.0 + sk::RANK5_DAMAGE - sk::RANK2_DAMAGE);
            if mends {
                parts.push(fill(s.rk_heal_more, &[&more]));
            } else if !harmless {
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
        (sk::Ult::Bell, 2) => fill(s.rk_bell_radius, &[&num(sk::BELL.radius2)]),
        (sk::Ult::Bell, _) => fill(s.rk_line_dmg, &[&sk::BELL.damage3.to_string()]),
        (sk::Ult::Blizzard, 2) => fill(s.rk_flurry_secs, &[&num(sk::BLIZZARD.secs2)]),
        (sk::Ult::Blizzard, _) => fill(s.rk_line_dmg, &[&sk::BLIZZARD.damage3.to_string()]),
        (sk::Ult::Rain, 2) => fill(s.rk_flurry_secs, &[&num(sk::RAIN.secs2)]),
        (sk::Ult::Rain, _) => fill(s.rk_line_dmg, &[&sk::RAIN.damage3.to_string()]),
        (sk::Ult::Gale, 2) => fill(s.rk_line_len, &[&num(sk::GALE.len2)]),
        (sk::Ult::Gale, _) => fill(s.rk_line_dmg, &[&sk::GALE.last3.to_string()]),
        (sk::Ult::Ambush, 2) => fill(s.rk_ambush_range, &[&num(sk::AMBUSH.range2)]),
        (sk::Ult::Ambush, _) => fill(s.rk_line_dmg, &[&sk::AMBUSH.last3.to_string()]),
    }
}

/// 판의 이름(0270 C2 → 0272: 공통 · 종류의 이름 · 방패).
pub(super) fn plate_name(p: Plate) -> &'static str {
    tr(p.name())
}

/// 타입의 이름 — 물리도 이름이 있다(무기 판의 갈래 머리·스킬의 종류 줄).
pub(super) fn kind_name(k: crate::status::Kind) -> &'static str {
    match k {
        crate::status::Kind::Physical => s().sk_physical,
        _ => crate::i18n::ail_name(k),
    }
}

/// 칸의 갈래·종류 줄: "스킬 · 균열 · 대기 7초"(스킬은 무기 판의 칸이라 갈래 대신 타입 — 0270 C2, 대기는 지금 단의 값) /
/// "큰 기술 · 방패 · 망치"(그 큰 기술이 서는 판) / "패시브 · 맞받기".
pub(super) fn kind_line(board: &Board, e: Entry) -> String {
    let s = s();
    let branch = tr(e.branch().name());
    match e {
        Entry::Skill(id) => {
            let eff = id.def().at(board.rank[id.index()], board.pick[id.index()]);
            fill(
                s.sk_kind_skill,
                &[kind_name(id.def().kind), &num(eff.skill.cooldown)],
            )
        }
        Entry::Ult(u) => {
            let on: Vec<&str> = PLATES
                .iter()
                .filter(|p| p.great() == Some(u))
                .map(|&p| plate_name(p))
                .collect();
            fill(s.sk_kind_ult, &[&on.join(" · ")])
        }
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
/// 판 `plate`에서 본 것(0270 C2 — 무기 판은 그 판의 줄, `plates::offer_on`).
pub(super) fn look(plate: Plate, board: &Board, total: u32, e: Entry) -> (Look, bool) {
    let offer = plates::offer_on(plate, board, e, total);
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
pub(super) fn state_line(
    plate: Plate,
    board: &Board,
    total: u32,
    e: Entry,
    edit: bool,
) -> (String, Color) {
    let s = s();
    let price = e.price(board.rank(e)).to_string();
    match plates::offer_on(plate, board, e, total) {
        Offer::Full if matches!(e, Entry::Cell(_) | Entry::Link(_) | Entry::Vein(_)) => {
            (s.sk_st_open_cell.to_string(), AMBER)
        }
        Offer::Full if matches!(e, Entry::Key(_)) => (s.sk_st_key_on.to_string(), AMBER),
        // 다른 맺음을 켰다(0214): 바꾸기는 값이 없다 — 화로에서.
        Offer::Swap if !edit => (s.sk_st_key_view.to_string(), INK),
        Offer::Swap => (s.sk_st_key_swap.to_string(), ui::MINT),
        Offer::Full => (s.board_full.to_string(), AMBER),
        // 무기 판의 줄은 그 판에 쓴 점수로 연다(0270 C2).
        Offer::Locked(n) if plate.seat(e).is_some() => {
            (fill(s.sk_st_plate_locked, &[&n.to_string()]), DIM)
        }
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
    plate: Plate,
    board: &Board,
    total: u32,
    kits: &Kits,
    great: sk::Ult,
    e: Entry,
    edit: bool,
) -> Vec<Do> {
    let mut v = Vec::new();
    let offer = plates::offer_on(plate, board, e, total);
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
        // 속성 패시브의 고르기(0274 ② — 울림): 연 결이면 화로에서 값 없이 바꾼다.
        Entry::Vein(vn) if edit && vn.pick_b().is_some() && board.cells.veined(vn) => {
            v.push(Do::Flip)
        }
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

/// 그 스킬을 쓰는 무기 종류의 이름들(" · "로 이은 글 — 어느 무기나 되면 그 말, 0272). 한손 종류는 방패로 갈린다: 방패를 들어도
/// 되면 이름만, 방패 없이만 되면 "한손검(방패 없이)", 방패와 함께만 되면 "방패(한손 무기와 함께)".
pub(super) fn hands_text(id: sk::Id) -> String {
    use crate::weapons::{Hand, CLASSES};
    let s = s();
    let h = id.def().hands;
    if h == sk::ANY {
        return s.hands_any.to_string();
    }
    let shield = h & sk::SHIELD != 0;
    let mut one = false;
    let mut names: Vec<String> = Vec::new();
    for c in CLASSES.iter().filter(|c| h & sk::class_bit(c.class) != 0) {
        let name = tr(c.name);
        if c.hand == Hand::One {
            one = true;
            if !shield {
                names.push(fill(s.hands_no_shield, &[name]));
                continue;
            }
        }
        names.push(name.to_string());
    }
    // 방패 비트는 "한손 무기와 방패"다 — 한손 종류가 다 이름에 들지 않았으면(0274: 망치도 한손) 방패 줄을 따로 적는다.
    let all_one = CLASSES
        .iter()
        .filter(|c| c.hand == Hand::One)
        .all(|c| h & sk::class_bit(c.class) != 0);
    if shield && !(one && all_one) {
        names.push(s.hands_shield.to_string());
    }
    names.join(" · ")
}

/// 든 것으로 못 쓰는 스킬을 고르면 뜨는 알림 — 한 줄 자리라 짧게(쓰는 무기의 목록은 상세 칸의 "쓰는 무기" 줄이 말한다, 0225).
fn unfit_toast() -> String {
    s().toast_unfit.to_string()
}

/// 목록 한 줄의 글.
pub(super) fn act_label(board: &Board, e: Entry, d: Do, has_sub: bool) -> String {
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
        (Do::Flip, Entry::Vein(v)) => {
            let (mark, name, _) = vein_pick_text(v, !board.cells.vein_picked_b(v));
            fill(s.act_flip, &[mark, name])
        }
        (Do::Flip, _) => String::new(),
        (Do::Away, _) => s.act_away.to_string(),
    }
}

/// 화면의 자리(480×270 기준 px, 창의 몸 왼쪽 위가 원점). 칸의 자리·걷기·그림이 같은 수를 쓴다.
pub(super) mod geo {
    use crate::plates::Plate;

    /// 왼쪽 칸 격자: 갈래 셋.
    pub const GRID_W: f32 = super::super::tabs::LEFT_W;
    pub const COL_W: f32 = crate::skills::LANE_W;
    /// 판 줄(0270 C2 — 몸 · 결투 · 방패 · 망치 · 지팡이 · 원거리): 격자 자리의 맨 위, 갈래 머리 위.
    pub const STRIP_H: f32 = 17.0;
    /// 갈래 머리(이름 + 한 줄).
    pub const HEAD_H: f32 = 27.0;
    /// 굴리는 자리의 위(판 줄 + 갈래 머리).
    pub const TOP: f32 = STRIP_H + HEAD_H;
    /// 몸의 판의 줄 띠(0270 C2 — 스킬·큰 기술·결이 무기 판으로 가 패시브·다리의 줄 넷과 맺음 줄만 남았다): 띠 위의 이름표("쓴 점수
    /// N부터") 자리 + 작은 칸 하나. 띠의 합이 보이는 높이를 넘으면 격자는 위아래로 굴린다(0163).
    pub const BODY_BAND: [f32; 5] = [34.0, 34.0, 34.0, 34.0, 42.0];
    /// 몸의 판: 띠 안에서 칸의 위.
    pub const BODY_TOP: f32 = 11.0;
    /// 무기 판의 줄 띠(셋 — 첫 스킬 · 둘째 스킬 · 결과 큰 기술): 이름표 + 스킬 칸 + 단의 점.
    pub const WEAPON_BAND: [f32; 3] = [50.0; 3];
    /// 맺음 칸의 반 대각선(마름모 — 패시브보다 크게).
    pub const KEY: f32 = 26.0;
    /// 다리 칸의 대각선(0215 — 갈래 경계 위의 작은 마름모 고리).
    pub const LINK: f32 = 15.0;
    /// 띠 안: 스킬 칸의 위·한 변, 단의 점, 패시브 칸의 지름.
    pub const CELL_TOP: f32 = 17.0;
    pub const SKILL: f32 = 24.0;
    pub const PIPS_TOP: f32 = 42.5;
    pub const PASSIVE: f32 = 18.0;
    /// 굴리기: 휠·▲▼ 한 칸(기준 px).
    pub const WHEEL: f32 = 24.0;
    /// 오른쪽 상세 칸.
    pub const DETAIL_X: f32 = super::super::tabs::RIGHT_X;
    pub const DETAIL_W: f32 = super::super::tabs::RIGHT_W;
    /// 상세의 머리(그림 + 이름·갈래·상태)와 큰 그림의 한 변.
    pub const DETAIL_HEAD: f32 = super::super::tabs::HEAD_H;
    pub const DETAIL_ICON: f32 = super::super::tabs::HEAD_ICON;
    /// 되돌리기 단추(창 아래 안내 줄의 오른쪽 끝)의 폭·높이.
    pub const RESPEC_W: f32 = 150.0;
    pub const RESPEC_H: f32 = 16.0;

    /// 판의 줄 띠들.
    pub fn bands(plate: Plate) -> &'static [f32] {
        match plate {
            Plate::Body => &BODY_BAND,
            Plate::Class(_) | Plate::Shield => &WEAPON_BAND,
        }
    }

    pub fn band_top(plate: Plate, row: usize) -> f32 {
        TOP + bands(plate)[..row].iter().sum::<f32>()
    }

    /// 굴리는 내용의 높이(띠의 합).
    pub fn content_h(plate: Plate) -> f32 {
        bands(plate).iter().sum()
    }
}

/// 판 줄(0270 C2 → 0275): 이름의 글 배(창의 배 `Window::k`에 곱한다 — 줄이지 않는다), 칸에서 이름 곁에 비우는 폭(양옆 —
/// 오른쪽 귀퉁이의 금빛 점 자리, 기준 px), 칸 사이 틈. 다 들면 남는 폭을 나눠 채우고, 넘치면 가로로 굴린다(`ui::strip_tabs`).
pub(super) const STRIP_K: f32 = 0.58;
pub(super) const STRIP_PAD: f32 = 12.0;
const STRIP_GAP: f32 = 2.0;

/// 열린 줄의 안내·선이 걷히는 초(2026-11-23).
const ROW_FADE: f32 = 0.8;

/// 칸 하나의 자리: 가운데와 반 변(반지름).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Spot {
    pub c: Vec2,
    pub half: f32,
}

/// 칸 `e`가 판 `plate`의 몇째 줄 띠에 서나(몸의 판 = 판의 줄 `Entry::row`, 무기 판 = 자리의 줄).
pub(super) fn row_on(plate: Plate, e: Entry) -> usize {
    match plate {
        Plate::Body => e.row(),
        Plate::Class(_) | Plate::Shield => plate.seat(e).map_or(0, |s| s.row as usize),
    }
}

/// `ENTRIES`의 자리.
fn index_of(e: Entry) -> usize {
    ENTRIES.iter().position(|x| *x == e).unwrap_or(0)
}

/// 판 `plate`의 칸마다의 자리(`ENTRIES`의 자리, 판의 차례). 몸의 판은 가로 자리를 판의 길과 같은 `Entry::lane`으로(0215 —
/// 선이 그림과 어긋나지 않게, 다리 칸은 갈래 경계의 작은 마름모), 세로만 줄 띠로 당긴다. 무기 판(0270 C2)은 갈래 셋(왼쪽 = 대표
/// 타입 A · 가운데 · 오른쪽 = B) × 줄 셋 — 한 칸 자리에 여럿이면 갈래 폭을 나눠 나란히.
pub(super) fn spots_on(plate: Plate) -> Vec<(usize, Spot)> {
    use geo::*;
    match plate {
        Plate::Body => plate
            .entries()
            .into_iter()
            .map(|i| {
                let e = ENTRIES[i];
                let top = band_top(plate, e.row()) + BODY_TOP;
                let (size, half) = match e {
                    Entry::Key(_) => (KEY, KEY * 0.5),
                    Entry::Link(_) => (PASSIVE, LINK * 0.5),
                    _ => (PASSIVE, PASSIVE * 0.5),
                };
                (
                    i,
                    Spot {
                        c: vec2(e.lane(), top + size * 0.5),
                        half,
                    },
                )
            })
            .collect(),
        Plate::Class(_) | Plate::Shield => {
            let seats = plate.seats();
            seats
                .iter()
                .map(|s| {
                    let peers: Vec<&Seat> = seats
                        .iter()
                        .filter(|o| o.row == s.row && o.col == s.col)
                        .collect();
                    let k = peers.iter().position(|o| o.entry == s.entry).unwrap_or(0);
                    let col = match s.col {
                        Col::Left => 0.0,
                        Col::Mid => 1.0,
                        Col::Right => 2.0,
                    };
                    // 한 자리에 넷 넘게 서면(0274 — 한손검의 가운데 열) 갈래 폭을 나누면 칸이 겹친다: 칸 사이를 칸 폭 + 2로 벌려
                    // 갈래의 가운데를 두고 옆 갈래 쪽으로 넘친다(옆 갈래는 그 줄에 하나뿐이라 닿지 않는다 — 시험).
                    let n = peers.len() as f32;
                    let step = (COL_W / n).max(SKILL + 2.0);
                    let x = COL_W * (col + 0.5) + step * (k as f32 - (n - 1.0) * 0.5);
                    let y = band_top(plate, s.row as usize) + CELL_TOP + SKILL * 0.5;
                    let half = match s.entry {
                        Entry::Vein(_) => PASSIVE * 0.5,
                        _ => SKILL * 0.5,
                    };
                    (
                        index_of(s.entry),
                        Spot {
                            c: vec2(x, y),
                            half,
                        },
                    )
                })
                .collect()
        }
    }
}

/// 판 `plate`에서 칸 `i`(`ENTRIES`의 자리)의 자리 — 그 판에 서지 않으면 없다.
pub(super) fn spot_on(plate: Plate, i: usize) -> Option<Spot> {
    spots_on(plate)
        .into_iter()
        .find(|(j, _)| *j == i)
        .map(|(_, s)| s)
}

/// 판의 칸을 읽는 차례(위에서 아래로, 왼쪽에서 오른쪽으로).
fn reading(plate: Plate) -> Vec<(usize, Spot)> {
    let mut v = spots_on(plate);
    v.sort_by(|a, b| {
        a.1.c
            .y
            .total_cmp(&b.1.c.y)
            .then(a.1.c.x.total_cmp(&b.1.c.x))
    });
    v
}

/// 칸 `i`가 "보인다"고 할 구간(내용의 y — 띠의 처음이 0): 커서가 그 칸에 서면 격자 창이 이 구간이 보이게 가장 적게 움직인다(0163).
/// 0270 C2부터 띠마다 칸 줄이 하나라 띠 하나가 통째로(이름표부터 단의 점까지).
pub(super) fn reveal_span(plate: Plate, i: usize) -> (f32, f32) {
    let row = ENTRIES.get(i).map_or(0, |&e| row_on(plate, e));
    let top = geo::band_top(plate, row) - geo::TOP;
    (top, top + geo::bands(plate)[row])
}

/// 판 `plate`를 열 때 설 칸(0161 → 0270 C2, `ENTRIES`의 자리): 올릴 수 있는 화면(`edit`)이면 그 판에서 지금 살 수 있는 첫 칸(읽는
/// 차례), 없으면(또는 읽기 전용이면) 배운 첫 스킬, 그것도 없으면 왼쪽 위 칸.
pub(super) fn first_focus_on(plate: Plate, board: &Board, total: u32, edit: bool) -> usize {
    let order = reading(plate);
    order
        .iter()
        .find(|(i, _)| edit && plates::offer_on(plate, board, ENTRIES[*i], total) == Offer::Ready)
        .or_else(|| {
            order
                .iter()
                .find(|(i, _)| matches!(ENTRIES[*i], Entry::Skill(id) if board.learned(id)))
        })
        .or(order.first())
        .map_or(STRIP, |(i, _)| *i)
}

/// 십자 한 걸음(0161 — KB `game-menu-scene-stack-and-focus`: 격자는 돌지 않는다): 판 `plate`의 칸 가운데 누른 쪽에 있는 가장
/// 가깝고 곧은 것으로(위아래는 곧은 것이 없으면 그쪽 줄의 가장 가까운 칸). 그쪽에 칸이 없으면 — 위는 판 줄(`STRIP`, 0270 C2),
/// 아래는 되돌리기 단추(창 아래 오른쪽, `respec`일 때 — 화로), 옆은 그대로. 오른쪽 걸음은 격자에서 끝난다(0261, 사장 "오른쪽 이동은
/// 기술 쪽에서 끝나야 한다"). 단추에서 위는 `back`(격자를 떠나기 전의 칸 — 이 판에 없으면 왼쪽 위 칸)으로, 판 줄에서 아래는
/// 늘 맨 윗줄로(떠난 칸이 거기 있으면 그 칸).
/// 판 줄의 옆 걸음(판 바꾸기)은 부른 쪽이 한다 — 여기서는 판 줄에 머문다.
pub(super) fn walk(
    plate: Plate,
    from: usize,
    dx: i32,
    dy: i32,
    respec: bool,
    back: usize,
) -> usize {
    if dx == 0 && dy == 0 {
        return from;
    }
    let spots = spots_on(plate);
    let first = reading(plate).first().map_or(STRIP, |(i, _)| *i);
    let left = spots.iter().find(|(i, _)| *i == back).map(|(_, s)| s.c.y);
    let top = spots
        .iter()
        .map(|(_, s)| s.c.y)
        .fold(f32::INFINITY, f32::min);
    match from {
        // 판 줄에서 아래 = 맨 윗줄(떠난 칸이 맨 윗줄에 있으면 거기, 아니면 왼쪽 위 — 늘 맨 윗줄로 내려온다).
        STRIP if dy > 0 => return if left == Some(top) { back } else { first },
        STRIP => return STRIP,
        RESPEC if dy < 0 => return if left.is_some() { back } else { first },
        RESPEC => return RESPEC,
        _ => {}
    }
    let Some(here) = spots.iter().find(|(i, _)| *i == from).map(|(_, s)| s.c) else {
        return first;
    };
    // 옆 걸음은 같은 줄의 칸만(줄은 반듯하다 — 0270 C2: 칸이 성긴 무기 판에서 비스듬히 다른 줄로 샜다). 위아래 걸음은 그쪽의
    // **바로 다음 줄**에서 가로로 가장 가까운 칸(0270 C2 — 비스듬한 칸을 부채꼴로 보면 칸이 성긴 줄을 건너뛰었다: 몸의 판 셋째 줄 ·
    // 무기 판의 둘째 줄).
    let best = if dx != 0 {
        let dir = vec2(dx as f32, 0.0);
        spots
            .iter()
            .filter(|(i, _)| *i != from)
            .filter_map(|(i, s)| {
                let d = s.c - here;
                let along = d.dot(dir);
                (along > 0.5 && d.y.abs() < 0.5).then_some((*i, along))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(i, _)| i)
    } else {
        let ahead = |s: &Spot| (s.c.y - here.y) * dy.signum() as f32;
        let next = spots
            .iter()
            .map(|(_, s)| ahead(s))
            .filter(|&d| d > 0.5)
            .fold(f32::INFINITY, f32::min);
        spots
            .iter()
            .filter(|(_, s)| (ahead(s) - next).abs() < 0.5)
            .min_by(|a, b| {
                (a.1.c.x - here.x)
                    .abs()
                    .total_cmp(&(b.1.c.x - here.x).abs())
            })
            .map(|(i, _)| *i)
    };
    match best {
        Some(i) => i,
        None if dy < 0 => STRIP,
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
    /// 3단: 두 고르기를 나란히 — (이름 줄들, 설명 줄들, 고른 것인가). 이름도 상자 폭에서 줄을 바꾼다(0275).
    Picks {
        boxes: [(Vec<String>, Vec<String>, bool); 2],
        open: bool,
    },
    Gap(f32),
}

/// 상세 칸의 글 크기(본문 1 = 13px 기준)와 줄 높이(기준 px).
const K_NAME: f32 = super::tabs::K_NAME;
const K_BODY: f32 = super::tabs::K_BODY;
pub(super) const K_RANK: f32 = 0.68;
pub(super) const K_PICK: f32 = 0.62;
const LH_BODY: f32 = super::tabs::LH_BODY;
const LH_RANK: f32 = 9.8;
const LH_PICK: f32 = 8.8;
/// 단 줄의 번호 칸 폭.
pub(super) const RANK_NUM: f32 = 11.0;

impl Block {
    /// 덩이의 높이(기준 px).
    fn height(&self) -> f32 {
        match self {
            Block::Text { lines, k, .. } => lines.len() as f32 * LH_BODY * (k / K_BODY),
            Block::Rank { lines, .. } => lines.len().max(1) as f32 * LH_RANK + 2.0,
            Block::Picks { boxes, .. } => {
                let tall = |b: &(Vec<String>, Vec<String>, bool)| {
                    b.0.len().max(1) as f32 * LH_RANK + b.1.len() as f32 * LH_PICK
                };
                tall(&boxes[0]).max(tall(&boxes[1])) + 6.0
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

    /// 든 무기·보조무기의 판(0270 C2 → 0272): 쥔 꼴의 판(`Plate::held`) — 스킬을 끼우는 셈(`kits`·`World::grip`)과 같은 꼴.
    /// 맨손은 무기 판이 없다(공통 판에서 연다 — 한손검의 판을 빌려 쓰지 않는다). 판 줄의 금빛 표지와 판을 고르는 차례가 쓴다.
    pub(super) fn held_plates(&self) -> (Option<Plate>, Option<Plate>) {
        let kits = self.kits();
        (
            Plate::held(kits.main.0),
            kits.sub.and_then(|(g, _)| Plate::held(g)),
        )
    }

    /// 보이는 판을 바꾼다: 격자 창·줄의 열림 연출은 판마다 처음부터(줄 띠가 다르다).
    pub(super) fn set_plate(&mut self, p: Plate) {
        if self.skill_plate != p {
            self.skill_plate = p;
            self.view = ui::ListView::default();
            self.skill_rows.0 = None;
        }
    }

    /// 기술 탭을 열 때(0270 C2): 든 무기의 판과 그 판에서 설 칸(`first_focus_on`).
    pub(super) fn open_plate(&mut self, edit: bool) {
        let p = self.held_plates().0.unwrap_or(Plate::Body);
        self.set_plate(p);
        // 판 줄은 처음부터 — 그 판이 보이게(손으로 굴려 둔 창을 이어 가지 않는다, 0275).
        self.plate_strip = ui::Strip::default();
        self.cursor = first_focus_on(p, &self.board(), self.progress.skill_points(), edit);
    }

    /// 커서의 칸이 보이는 판에 없으면(성장 탭의 기술 줄에서 넘어온 칸 등) 그 칸이 서는 판으로 — 든 무기 · 부무기의 판 먼저.
    pub(super) fn sync_plate(&mut self) {
        let Some(&e) = ENTRIES.get(self.cursor) else {
            return;
        };
        if !self.skill_plate.has(e) {
            let (main, sub) = self.held_plates();
            let prefer: Vec<Plate> = [main, sub].into_iter().flatten().collect();
            self.set_plate(plates::home(e, &prefer));
        }
    }

    /// 기술 화면 한 틱: 십자 = 칸 걷기(맨 윗줄에서 위 = 판 줄, 판 줄에서 옆 = 판 바꾸기), 확인·고른 칸 다시 누르기 = 그 칸의 일 목록
    /// (되돌리기 단추면 묻는 창, 판 줄이면 격자로), 판 줄의 판 누르기 = 그 판, 뒤로·✕·창 밖 = 닫기.
    pub(super) fn tick_skills(&mut self, a: &Actions, edit: bool) {
        self.sync_plate();
        // 판 줄이 넘쳐 굴리는 동안(0275) 그 위의 끌기·휠은 판 줄을.
        let mut a = super::strip_input(&mut self.plate_strip, a);
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
        // 판 줄의 판을 눌렀다: 그 판으로, 커서는 판 줄에(상세가 그 판을 말한다).
        if let Some(Hit::Cell(1, p)) = tap {
            if self.cursor < NE {
                self.respec_from = self.cursor;
            }
            self.set_plate(PLATES[p.min(PLATES.len() - 1)]);
            self.cursor = STRIP;
            self.scroll = 0.0;
            return;
        }
        let (dx, dy) = (
            a.menu_right as i32 - a.menu_left as i32,
            a.menu_down as i32 - a.menu_up as i32,
        );
        // 판 줄에서 옆 = 판 바꾸기(가장자리에서 돌지 않는다).
        if self.cursor == STRIP && dx != 0 && dy == 0 {
            let i = self.skill_plate.index() as i32 + dx;
            if (0..PLATES.len() as i32).contains(&i) {
                self.set_plate(PLATES[i as usize]);
            }
            return;
        }
        // 판 줄에서 확인 = 격자로(아래 걸음과 같다).
        let (dy, confirm_strip) = if self.cursor == STRIP && a.confirm {
            (1, true)
        } else {
            (dy, false)
        };
        self.cursor = walk(
            self.skill_plate,
            self.cursor.min(STRIP),
            dx,
            dy,
            edit,
            self.respec_from,
        );
        // 격자를 떠난 칸을 기억한다 — 단추에서 위·판 줄에서 아래로 가면 거기로(0261).
        if self.cursor >= NE && before < NE {
            self.respec_from = before;
        }
        // 십자로 옮긴 커서는 창을 끈다 — 손가락으로 고른 칸은 이미 보이니 창을 옮기지 않는다(둘째 탭이 같은 자리에 닿게, 0076).
        if self.cursor != before && self.cursor < NE {
            let (y0, y1) = reveal_span(self.skill_plate, self.cursor);
            self.view.reveal(y0, y1);
        }
        if confirm_strip {
            self.scroll = 0.0;
            return;
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
        if !confirm || self.cursor == STRIP {
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
                    let total = self.progress.skill_points();
                    state_line(self.skill_plate, &self.board(), total, e, edit).0
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
            self.skill_plate,
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
            (Do::Flip, Entry::Vein(v)) => {
                #[cfg(feature = "dev")]
                if let Some(b) = self.dev.board.as_mut() {
                    b.flip_vein(v);
                }
                if !self.dev_board_on() {
                    self.progress.board.flip_vein(v);
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
        // 보이는 판의 셈으로(0270 C2 — 무기 판은 그 판의 줄): 단·기록은 같은 판(`Board`)에 남는다.
        if !plates::buy_on(self.skill_plate, &mut self.progress.board, e, total) {
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
        self.sync_plate();
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

    /// 판 줄(0270 C2): 판 여섯의 이름을 나란히 — 보이는 판은 금빛 글 + 밑줄, 커서가 판 줄에 있으면 그 판에 밝은 테(칸의 포커스와
    /// 같은 빛). 든 무기의 판은 오른쪽 위에 금빛 점, 부무기의 판은 금빛 고리(칸의 `node_badge`·`sub_badge`와 같은 갈림). 살 칸이
    /// 있다는 민트 점은 판 줄에 두지 않는다(사장 2026-11-24 "기술 서브탭의 new 인디케이터는 필요없을듯" — 탭 머리의 점과 칸의 민트로
    /// 넉넉하다). 누를 자리는 판마다.
    fn draw_plate_strip(&mut self, w: &Window, at: usize) {
        let (k, u) = (w.k, w.s);
        let (main, sub) = self.held_plates();
        // 칸의 폭은 이름대로(0272): 이름 + 표지 자리. 판이 여덟(창)이 되어 영어 이름이 다 못 서자 글자를 줄였는데(0272 2단계),
        // 0275부터는 줄이지 않는다 — 큰 창의 탭 줄(0233)처럼 다 들면 남는 폭을 나눠 채우고, 넘치면 가로로 굴린다(끌기·휠,
        // 보이는 판이 바뀌면 그 판이 보이게 가장 적게, 양끝 ◀▶ = 더 있다). 가장자리에 걸친 이름은 잘린 채로(넘침 검사 밖).
        let tk = STRIP_K * k;
        let widths: Vec<f32> = PLATES
            .iter()
            .map(|&p| self.ui.width(plate_name(p), tk) + STRIP_PAD * u)
            .collect();
        let arrow = self.ui.width("◀", tk) + u * 4.0;
        let (x0, x1) = (w.at(0.0, 0.0).x, w.at(geo::GRID_W, 0.0).x);
        let h = (geo::STRIP_H - 3.0) * u;
        let (cells, view) = ui::strip_tabs(
            &mut self.plate_strip,
            &widths,
            x0,
            x1,
            w.origin.y,
            h,
            STRIP_GAP * u,
            arrow,
            self.skill_plate.index(),
        );
        if let Some(v) = view {
            // 포커스의 테(칸 밖으로 한 칸)가 창 끝에서 잘리지 않을 만큼만 넓혀 자른다.
            ui::clip(Some(Rect::new(
                v.x - u * 1.5,
                v.y - u * 2.0,
                v.w + u * 3.0,
                v.h + u * 4.0,
            )));
        }
        for (i, &p) in PLATES.iter().enumerate() {
            let (x, cw) = cells[i];
            let r = Rect::new(x, w.origin.y, cw, h);
            let whole = match view {
                Some(v) if x + cw <= v.x || x >= v.x + v.w => continue,
                Some(v) => x >= v.x - 0.5 && x + cw <= v.x + v.w + 0.5,
                None => true,
            };
            let shown = p == self.skill_plate;
            if shown {
                draw_rectangle(r.x, r.y, r.w, r.h, ui::PANEL_LIGHT);
                draw_rectangle(r.x, r.y + r.h - u, r.w, u.max(1.0), AMBER);
            }
            let ink = if shown { AMBER } else { DIM };
            let name = plate_name(p);
            let ty = r.y + (r.h - self.ui.px(tk)) * 0.5;
            if whole {
                self.ui.text_center(name, r.x + r.w * 0.5, ty, tk, ink);
            } else {
                let tw = self.ui.width(name, tk);
                self.ui
                    .plain_text(name, r.x + (r.w - tw) * 0.5, ty, tk, ink);
            }
            let dot = (u * 2.2).max(2.0);
            let (dx, dy) = (r.x + r.w - dot * 1.6, r.y + dot * 1.4);
            if main == Some(p) {
                draw_circle(dx, dy, dot, AMBER);
            } else if sub == Some(p) {
                draw_circle_lines(dx, dy, dot, 1.0, AMBER);
            }
            if at == STRIP && shown {
                let o = Rect::new(r.x - u, r.y - u, r.w + u * 2.0, r.h + u * 2.0);
                self.ui.edge(o, (u * 1.6).max(2.0), CREAM);
            }
            // 누를 자리는 보이는 몫만.
            let hit = match view {
                Some(v) => {
                    let l = x.max(v.x);
                    Rect::new(l, r.y, (x + cw).min(v.x + v.w) - l, r.h)
                }
                None => r,
            };
            self.hits.push((hit, Hit::Cell(1, i)));
        }
        if let Some(v) = view {
            ui::clip(None);
            let (l, r) = self.plate_strip.more();
            let ink = |more: bool| if more { INK } else { Color { a: 0.35, ..DIM } };
            let ty = w.origin.y + (h - self.ui.px(tk)) * 0.5;
            self.ui.text_center("◀", x0 + arrow * 0.5, ty, tk, ink(l));
            self.ui
                .text_center("▶", v.x + v.w + arrow * 0.5, ty, tk, ink(r));
        }
    }

    /// 왼쪽: 판 줄 · 갈래 머리 · 줄 띠 · 칸들. 누를 자리(판·칸)를 채운다.
    fn draw_skill_grid(&mut self, w: &Window, board: &Board, total: u32, at: usize) {
        let s = s();
        let (k, u) = (w.k, w.s);
        let plate = self.skill_plate;
        self.draw_plate_strip(w, at);
        // 갈래 머리: 몸의 판 = 이름(금빛) + 한 줄, 무기 판 = 대표 타입 둘(그 타입의 빛 `status::INK`)과 가운데 "그 밖".
        let head = geo::STRIP_H;
        match plate {
            Plate::Body => {
                for b in BRANCHES {
                    let cx = w.at(geo::COL_W * (b.index() as f32 + 0.5), 0.0).x;
                    self.ui
                        .text_center(tr(b.name()), cx, w.at(0.0, head).y, k, AMBER);
                    let motto = self.ui.fit(tr(b.motto()), (geo::COL_W - 4.0) * u, 0.62 * k);
                    self.ui
                        .text_center(&motto, cx, w.at(0.0, head + 14.5).y, 0.62 * k, DIM);
                }
            }
            Plate::Class(_) | Plate::Shield => {
                // 가운데 열의 머리 = 그 종류의 물리 속성(참격 · 타격 · 관통 · 마법 — 방패 판은 방어, 0272).
                let [a, b] = plate.pair().unwrap_or([crate::status::Kind::Physical; 2]);
                let mid = plate.attribute().map_or("", tr);
                for (col, kind) in [(0.0, Some(a)), (1.0, None), (2.0, Some(b))] {
                    let cx = w.at(geo::COL_W * (col + 0.5), 0.0).x;
                    let (name, ink) = match kind.and_then(|k| k.slot().map(|i| (k, i))) {
                        Some((kind, i)) => {
                            let [r, g, b] = crate::status::INK[i];
                            (kind_name(kind), Color::new(r, g, b, 1.0))
                        }
                        None => (mid, DIM),
                    };
                    let name = self.ui.fit(name, (geo::COL_W - 4.0) * u, k);
                    self.ui
                        .text_center(&name, cx, w.at(0.0, head + 5.0).y, k, ink);
                }
            }
        }
        // 굴리는 자리(0163): 갈래 머리 아래 ~ 몸의 끝. 내용(띠)이 넘치면 창을 옮겨 본다 — 처음 그릴 때는 커서 칸이 보이게.
        let room = w.body_h - geo::TOP;
        let (mut off, fresh) = self.view.span(room, geo::content_h(plate), u);
        if fresh && at < NE {
            let (y0, y1) = reveal_span(plate, at);
            off = self.view.reveal(y0, y1);
        }
        let port = w.rect(0.0, geo::TOP, geo::GRID_W, room);
        let (port_top, port_bottom) = (port.y, port.y + port.h);
        ui::clip(Some(port));
        // 줄 띠: 가는 선 + 오른쪽 끝의 이름. 잠긴 줄만 "쓴 점수 N부터 · n점 더"(2026-11-23 사장 "열린 후엔 필요 없음") — 열리는 순간
        // 선이 이름 쪽으로 걷히며 이름과 함께 사라진다(`ROW_FADE`). 몸의 판은 쓴 점수 합(`ROW_NEEDS`), 무기 판은 그 판에 쓴 점수
        // (`plates::PLATE_ROWS`, 0270 C2). 글은 다 보일 때만(잘린 글은 그리지 않는다).
        let bands = geo::bands(plate);
        let (spent, needs): (u32, Vec<u32>) = match plate {
            Plate::Body => (board.spent(), sk::ROW_NEEDS[..bands.len()].to_vec()),
            Plate::Class(_) | Plate::Shield => (plate.spent(board), plates::PLATE_ROWS.to_vec()),
        };
        let open_mask: u8 = (0..bands.len())
            .filter(|&r| spent >= needs[r])
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
        for (row, (&need, &since)) in needs.iter().zip(opened_at.iter()).enumerate() {
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
            let label = fill(
                match plate {
                    Plate::Body => s.sk_row_locked,
                    Plate::Class(_) | Plate::Shield => s.sk_row_plate,
                },
                &[&need.to_string(), &(need.saturating_sub(spent)).to_string()],
            );
            let top = geo::band_top(plate, row) - off;
            let lk = 0.6 * k;
            let lw = self.ui.width(&label, lk);
            // 오른쪽 끝은 굴림 표시(▲▼)의 자리로 비운다 — 띠가 당겨진 몸의 판(0270 C2)에서 맺음 줄의 이름이 ▼와 겹쳤다.
            let right = w.at(geo::GRID_W, 0.0).x - self.ui.width("▼", lk) - u * 4.0;
            let y = w.at(0.0, top).y;
            let ink = Color {
                a: 0.8 * fade,
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
        let spots = spots_on(plate);
        // 판의 길(0215): 칸 사이 선 — 칸보다 먼저(칸의 면이 끝을 덮는다). 무기 판에는 길이 없다(줄은 그 판의 점수로 연다).
        if plate == Plate::Body {
            self.draw_paths(w, board, &spots, off);
        }
        let mut cells = Vec::new();
        // 튜토리얼의 배우기 걸음(0209): 지금 살 수 있는 칸에 반짝임.
        let guide = self.tutor_menu() == Some(flow::Tab::Skills)
            && flow::tab_of(self.scene()).is_some_and(|(h, _)| h.edits());
        for &(i, sp) in &spots {
            let e = ENTRIES[i];
            let (c, half) = (w.at(sp.c.x, sp.c.y - off), sp.half * u);
            // 보이는 자리 밖의 칸은 건너뛴다(걸친 칸은 가위가 자른다).
            let reach = half + u * 8.0;
            if c.y + reach < port_top || c.y - reach > port_bottom {
                continue;
            }
            let (lk, ready) = look(plate, board, total, e);
            let fade = if lk == Look::Locked { 0.5 } else { 1.0 };
            let pips_top = sp.c.y - geo::SKILL * 0.5 - geo::CELL_TOP + geo::PIPS_TOP - off;
            let pips_end = w.at(0.0, pips_top + 3.0).y;
            draw_node(e, c, half, u, lk, (i == at).then_some(pips_end));
            let at_c = c;
            // 귀퉁이 표지(글)는 칸이 다 보일 때만.
            let whole = c.y - half - u * 3.0 >= port_top && c.y + half <= port_bottom;
            match e {
                Entry::Skill(id) => {
                    let py = w.at(0.0, pips_top).y.round();
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
                    let py = w.at(0.0, pips_top).y.round();
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

    /// 판의 길(0215 B — 몸의 판): 이어진 칸 사이 선. 둘 다 가졌으면 금빛, 가진 칸에서 아직 안 가진 칸으로 갈 수 있으면 옅은 민트,
    /// 나머지는 흐린 테 빛. 위아래 선은 위 칸의 아래에서 아래 칸의 위로, 같은 줄(다리)은 옆에서 옆으로. 판에 선 칸끼리만.
    fn draw_paths(&self, w: &Window, board: &Board, spots: &[(usize, Spot)], off: f32) {
        let u = w.s;
        for &(i, sa) in spots {
            let a = ENTRIES[i];
            for &(j, sb) in spots {
                let b = ENTRIES[j];
                if !a.feeds(b) || (a.line() == b.line() && i > j && b.feeds(a)) {
                    continue;
                }
                let (p, q) = if a.line() == b.line() {
                    let d = if sb.c.x > sa.c.x { 1.0 } else { -1.0 };
                    (
                        vec2(sa.c.x + d * sa.half, sa.c.y),
                        vec2(sb.c.x - d * sb.half, sb.c.y),
                    )
                } else {
                    (
                        vec2(sa.c.x, sa.c.y + sa.half),
                        vec2(sb.c.x, sb.c.y - sb.half),
                    )
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
        let entry = ENTRIES.get(at).copied();
        let plate = self.skill_plate;
        let (name, kind, state) = match entry {
            Some(e) => {
                let (lk, _) = look(plate, board, total, e);
                let c = w.at(x0 + geo::DETAIL_ICON * 0.5, geo::DETAIL_ICON * 0.5 + 1.0);
                draw_node(e, c, geo::DETAIL_ICON * 0.5 * u, u, lk, None);
                (
                    entry_text(e).0,
                    kind_line(board, e),
                    state_line(plate, board, total, e, edit),
                )
            }
            // 판 줄(0270 C2): 판의 이름 · 종류(대표 타입 둘) · 그 판에 쓴 점수.
            None if at == STRIP => {
                let kind = match plate {
                    Plate::Body => s.sk_plate_kind_body.to_string(),
                    Plate::Class(_) | Plate::Shield => {
                        let [a, b] = plate.pair().unwrap_or([crate::status::Kind::Physical; 2]);
                        let t = if plate == Plate::Shield {
                            s.sk_plate_kind_shield
                        } else {
                            s.sk_plate_kind_grip
                        };
                        fill(t, &[kind_name(a), kind_name(b)])
                    }
                };
                let spent = fill(s.sk_plate_spent, &[&plate.spent(board).to_string()]);
                (plate_name(plate), kind, (spent, DIM))
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
        // 갈래 줄 · 상태 줄(알림이 길면 두 줄로 — `draw_head_lines`, 0275).
        self.draw_head_lines(w, name_x, name_w, &kind, &state);

        // 몸: 하는 일 + 단마다 한 줄. 넘치면 덩이 단위로 굴린다.
        let width = dw * u;
        let blocks = self.detail_blocks(board, total, entry, at, width, k);
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
        self.respec_button(r, &label, at == RESPEC, self.respec_ready(), w.k);
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
        at: usize,
        width: f32,
        k: f32,
    ) -> Vec<Block> {
        let s = s();
        let u = width / geo::DETAIL_W;
        let wrap = |t: &str, w: f32, tk: f32| ui::wrap(&self.ui, t, w, tk * k);
        let plate = self.skill_plate;
        // 판 줄(0270 C2): 판이 무엇인지 + 든 무기·부무기의 판이면 그렇다고(금빛 — 판 줄의 표지와 같은 빛).
        if entry.is_none() && at == STRIP {
            let what = match plate {
                Plate::Body => s.sk_plate_body_d,
                Plate::Class(_) => s.sk_plate_grip_d,
                Plate::Shield => s.sk_plate_shield_d,
            };
            let mut v = vec![Block::Text {
                lines: wrap(what, width, K_BODY),
                k: K_BODY,
                ink: INK,
            }];
            let (main, sub) = self.held_plates();
            for (on, t) in [(main, s.sk_plate_main), (sub, s.sk_plate_sub)] {
                if on == Some(plate) {
                    v.push(Block::Gap(5.0));
                    v.push(Block::Text {
                        lines: wrap(t, width, K_BODY),
                        k: K_BODY,
                        ink: AMBER,
                    });
                }
            }
            return v;
        }
        let Some(e) = entry else {
            return vec![Block::Text {
                lines: wrap(s.board_respec_d, width, K_BODY),
                k: K_BODY,
                ink: INK,
            }];
        };
        let rank = board.rank(e);
        let offer = plates::offer_on(plate, board, e, total);
        let what = match e {
            Entry::Cell(c) if c.pick_b().is_some() => s.sk_cell_pick,
            Entry::Vein(v) if v.pick_b().is_some() => s.sk_cell_pick,
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
                                wrap(&format!("{mark} {name}"), box_w, K_RANK),
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
                        wrap(&format!("{mark} {name}"), box_w, K_RANK),
                        wrap(desc, box_w, K_PICK),
                        open && board.cells.picked_b(c) == b,
                    )
                };
                v.push(Block::Picks {
                    boxes: [one(false), one(true)],
                    open,
                });
            }
            // 울림의 속성 패시브(0274 ②): 숫자 칸과 같은 그림 — 두 고르기를 나란히, 연 결이면 지금 것에 금빛.
            Entry::Vein(vn) if vn.pick_b().is_some() => {
                let box_w = (width - RANK_NUM * u - u * 4.0) * 0.5 - u * 6.0;
                let open = board.cells.veined(vn);
                let one = |b: bool| {
                    let (mark, name, desc) = vein_pick_text(vn, b);
                    (
                        wrap(&format!("{mark} {name}"), box_w, K_RANK),
                        wrap(desc, box_w, K_PICK),
                        open && board.cells.vein_picked_b(vn) == b,
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
                    for (j, line) in name.iter().enumerate() {
                        let p = w.at(bx + 3.0, y + 2.0 + LH_RANK * j as f32);
                        self.ui.text(line, p.x, p.y, K_RANK * k, ink);
                    }
                    let top = LH_RANK * name.len().max(1) as f32;
                    for (j, line) in desc.iter().enumerate() {
                        let p = w.at(bx + 3.0, y + 2.0 + top + LH_PICK * j as f32);
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
        let Some(sp) = spot_on(self.skill_plate, under.cursor) else {
            return;
        };
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
    use crate::weapons::Class;

    const SWORD: Plate = Plate::Class(Class::Sword);

    /// 0272: "쓰는 무기" 줄은 종류의 이름으로 — 한손 종류는 방패로 갈린다(방패 없이만 되면 그렇게, 방패와 함께만 되면 방패),
    /// 쇠뇌와 활은 따로, 모든 종류와 방패에 다 되면 "어느 무기나". 판의 이름도 종류 표에서 온다.
    #[test]
    fn the_hands_line_names_weapon_classes_and_the_plates_are_named_by_class() {
        let s = s();
        assert_eq!(hands_text(Id::Gather), s.hands_any);
        // 0274 ③: 가로지르기는 쌍단검의 것(두 손이 다 찬 종류라 "방패 없이"를 붙이지 않는다), 자루 치기는 한손검만.
        assert_eq!(hands_text(Id::Cut), s.class_dagger);
        assert_eq!(
            hands_text(Id::Pommel),
            fill(s.hands_no_shield, &[s.class_sword])
        );
        assert_eq!(hands_text(Id::Bash), s.hands_shield);
        assert_eq!(
            hands_text(Id::Whirl),
            format!("{} · {}", s.class_sword, s.class_hammer)
        );
        assert_eq!(
            hands_text(Id::Toll),
            format!("{} · {}", s.class_hammer, s.hands_shield)
        );
        assert_eq!(hands_text(Id::Snare), s.class_bow);
        assert_eq!(hands_text(Id::Fuse), s.class_crossbow);
        assert_eq!(
            hands_text(Id::Embers),
            format!("{} · {}", s.class_staff, s.class_crossbow)
        );
        let names: Vec<&str> = PLATES.iter().map(|&p| plate_name(p)).collect();
        assert_eq!(
            names,
            [
                s.plate_common,
                s.class_sword,
                s.class_hammer,
                s.class_staff,
                s.class_crossbow,
                s.class_bow,
                s.class_spear,
                s.class_dagger,
                s.plate_shield
            ]
        );
        // 가운데 열의 머리 = 종류의 물리 속성(방패 판은 방어) — "그 밖"은 없다.
        let mids: Vec<&str> = PLATES[1..]
            .iter()
            .map(|p| tr(p.attribute().unwrap()))
            .collect();
        assert_eq!(
            mids,
            [
                s.attr_slash,
                s.attr_strike,
                s.attr_magic,
                s.attr_pierce,
                s.attr_pierce,
                s.attr_pierce,
                s.attr_pierce,
                s.attr_guard
            ]
        );
        // 큰 기술의 종류 줄은 그것이 서는 판의 이름.
        let b = Board::default();
        assert_eq!(
            kind_line(&b, Entry::Ult(Ult::Rain)),
            fill(s.sk_kind_ult, &[s.class_bow])
        );
        assert_eq!(
            kind_line(&b, Entry::Ult(Ult::Line)),
            fill(s.sk_kind_ult, &[s.class_crossbow])
        );
    }
    use crate::progression::Cell;
    use sk::{Id, Ult};

    fn at(e: Entry) -> usize {
        ENTRIES.iter().position(|x| *x == e).unwrap()
    }

    /// 칸이 서는 판(결투 판 먼저 — 시험의 무기는 한손 검).
    fn on(e: Entry) -> Plate {
        plates::home(e, &[SWORD])
    }

    /// 처음의 둘을 배운 판(0166: 새 판은 아무것도 모른다 — "배운 스킬"의 규칙은 이 판으로 본다).
    fn started() -> Board {
        Board::default().with_starters()
    }

    /// 칸의 자리(0270 C2 — 판마다): 판에 선 칸은 모두 격자 안·제 줄 띠 안에 있고 서로 겹치지 않는다. 몸의 판은 갈래(흐름·맞받기·자리)의
    /// 기둥 안(다리 칸은 경계 위), 무기 판은 갈래 셋(왼쪽 = 대표 타입 A · 가운데 · 오른쪽 = B)의 기둥 안.
    #[test]
    fn every_cell_has_its_own_spot_in_its_column_and_row_band_on_every_plate() {
        for plate in PLATES {
            let sp = spots_on(plate);
            assert!(!sp.is_empty(), "{plate:?}");
            for (n, &(i, s)) in sp.iter().enumerate() {
                let (e, c, h) = (ENTRIES[i], s.c, s.half);
                assert!(h > 0.0, "{e:?} has a spot");
                assert!(c.x - h >= 0.0 && c.x + h <= geo::GRID_W, "{e:?} grid");
                let col = match (plate, e) {
                    (Plate::Body, Entry::Link(l)) => {
                        // 다리 칸(0215): 두 갈래의 경계 위.
                        assert_eq!(c.x, geo::COL_W * (l.left() + 1) as f32, "{e:?} boundary");
                        None
                    }
                    (Plate::Body, _) => Some(e.branch().index()),
                    (Plate::Class(_) | Plate::Shield, _) => {
                        Some(match plate.seat(e).unwrap().col {
                            Col::Left => 0,
                            Col::Mid => 1,
                            Col::Right => 2,
                        })
                    }
                };
                // 한 자리에 넷 넘게 선 갈래(0274 — 한손검의 가운데 열)는 옆 갈래 쪽으로 넘친다: 겹침은 아래가 본다.
                let crowded = matches!(plate, Plate::Class(_) | Plate::Shield)
                    && plate.seat(e).is_some_and(|me| {
                        plate
                            .seats()
                            .iter()
                            .filter(|o| o.row == me.row && o.col == me.col)
                            .count()
                            > 3
                    });
                if let Some(col) = col.filter(|_| !crowded) {
                    let x0 = geo::COL_W * col as f32;
                    assert!(
                        c.x - h >= x0 && c.x + h <= x0 + geo::COL_W,
                        "{plate:?} {e:?} column"
                    );
                }
                let row = row_on(plate, e);
                let top = geo::band_top(plate, row);
                // 띠 위의 이름표 자리를 비운다.
                assert!(
                    c.y - h >= top + 9.0 && c.y + h <= top + geo::bands(plate)[row],
                    "{plate:?} {e:?} band"
                );
                for &(j, o) in sp.iter().skip(n + 1) {
                    let apart = (c - o.c).abs();
                    assert!(
                        apart.x >= h + o.half + 2.0 || apart.y >= h + o.half + 2.0,
                        "{plate:?} {e:?} overlaps {:?}",
                        ENTRIES[j]
                    );
                }
            }
        }
        // 몸의 판은 가로 자리가 판의 길 그대로(0215 — 선이 그림과 어긋나지 않게).
        for (i, s) in spots_on(Plate::Body) {
            assert_eq!(s.c.x, ENTRIES[i].lane(), "{:?}", ENTRIES[i]);
        }
        // 무기 판의 스킬·큰 기술은 결보다 큰 칸.
        let duel = SWORD;
        let (cut, shock) = (
            spot_on(duel, at(Entry::Skill(Id::Pommel))).unwrap(),
            spot_on(duel, at(Entry::Vein(crate::progression::Vein::Shock))).unwrap(),
        );
        assert!(cut.half > shock.half && cut.c.y < shock.c.y);
        // 판 줄은 격자 머리 위, 상세 칸은 창 안.
        assert!(geo::TOP == geo::STRIP_H + geo::HEAD_H);
        assert!(geo::DETAIL_X + geo::DETAIL_W <= super::super::frame::WINDOW_W - 16.0);
    }

    /// 판은 칸을 나눠 갖는다(0270 C2): 몸의 판 = 패시브·다리·맺음, 무기 판 = 스킬·결·큰 기술 — 모든 칸이 어느 판엔가 선다
    /// (헤드리스 `hearth.js`가 판마다 돌아 모든 칸을 지난다).
    #[test]
    fn every_entry_stands_on_some_plate_and_the_body_keeps_the_passives() {
        for (i, &e) in ENTRIES.iter().enumerate() {
            let on: Vec<Plate> = PLATES.into_iter().filter(|p| p.has(e)).collect();
            // 어느 트리에도 서지 않는 결(`plates::HOMELESS` — 비어 있다: 0274 ②의 균열은 `CARRIED`로 한손검 트리에).
            if matches!(e, Entry::Vein(v) if plates::HOMELESS.contains(&v.kind())) {
                assert!(on.is_empty(), "{e:?}");
                continue;
            }
            assert!(!on.is_empty(), "{e:?} stands on no plate");
            assert_eq!(
                on.contains(&Plate::Body),
                matches!(e, Entry::Cell(_) | Entry::Key(_) | Entry::Link(_)),
                "{e:?}"
            );
            for p in on {
                assert!(spot_on(p, i).is_some(), "{p:?} {e:?}");
            }
        }
    }

    /// 0163: 격자는 위아래로 굴린다 — 판마다 칸의 "보인다"고 할 구간이 내용 안에 있고 보이는 높이(몸 195 − 판 줄 − 머리)에
    /// 들어가며, 십자로 맨 위에서 맨 아래까지 내려갔다 올라오면 창이 그 칸들을 따라간다(가장 적게). 무기 판의 줄 셋은 한 화면.
    #[test]
    fn the_grid_scrolls_to_keep_the_cursor_cell_in_view() {
        let room = 195.0 - geo::TOP;
        for plate in PLATES {
            for (i, s) in spots_on(plate) {
                let (y0, y1) = reveal_span(plate, i);
                assert!(y0 >= -0.01 && y1 <= geo::content_h(plate) + 0.01);
                assert!(y1 - y0 <= room, "{:?} fits the view", ENTRIES[i]);
                let (top, bottom) = (s.c.y - s.half - geo::TOP, s.c.y + s.half - geo::TOP);
                assert!(top >= y0 && bottom <= y1, "{:?}", ENTRIES[i]);
            }
            if plate != Plate::Body {
                assert!(geo::content_h(plate) <= room, "{plate:?}: one screen");
            }
        }
        let body = Plate::Body;
        assert!(
            geo::content_h(body) > room,
            "the body bands no longer fit — that is why it scrolls"
        );
        let mut view = ui::ListView::default();
        let visible = |view: &ui::ListView, i: usize| {
            let (y0, y1) = reveal_span(body, i);
            y0 >= view.offset() - 0.01 && y1 <= view.offset() + room + 0.01
        };
        view.span(room, geo::content_h(body), 2.0);
        let mut at_ = at(Entry::Cell(Cell::EvenBreath));
        assert_eq!(view.offset(), 0.0);
        for _ in 0..14 {
            let next = walk(body, at_, 0, 1, false, 0);
            if next == at_ {
                break;
            }
            at_ = next;
            let (y0, y1) = reveal_span(body, at_);
            view.reveal(y0, y1);
            assert!(visible(&view, at_), "{:?}", ENTRIES[at_]);
        }
        assert_eq!(ENTRIES[at_].row(), sk::KEY_ROW, "down to the keys");
        assert_eq!(view.offset(), geo::content_h(body) - room, "the bottom");
        for _ in 0..14 {
            let next = walk(body, at_, 0, -1, false, 0);
            if next == STRIP {
                break;
            }
            at_ = next;
            let (y0, y1) = reveal_span(body, at_);
            view.reveal(y0, y1);
            assert!(visible(&view, at_));
        }
        assert_eq!(ENTRIES[at_].row(), 0, "the first passives are the top band");
        assert_eq!(view.offset(), 0.0);
    }

    /// 걷기(KB: 격자는 돌지 않는다): 몸의 판 — 옆 걸음은 같은 줄의 이웃으로, 아래는 아래 줄로, 맨 윗줄에서 위 = 판 줄,
    /// 맨 아래에서 아래 = 되돌리기 단추(화로에서만), 오른쪽 걸음은 격자에서 끝난다(0261). 판 줄에서 아래 = 떠난 칸(이 판에 없으면
    /// 왼쪽 위 칸), 단추에서 위 = 떠난 칸.
    #[test]
    fn walking_the_body_plate_follows_the_layout_and_never_wraps() {
        let body = Plate::Body;
        let w = |from, dx, dy| walk(body, from, dx, dy, true, 0);
        let breath = at(Entry::Cell(Cell::EvenBreath));
        let flow = at(Entry::Cell(Cell::FlowingBeat));
        let catch = at(Entry::Cell(Cell::Catch));
        let sidestep = at(Entry::Cell(Cell::Sidestep));
        assert_eq!(w(breath, 1, 0), flow);
        assert_eq!(w(flow, 1, 0), catch);
        assert_eq!(w(catch, -1, 0), flow);
        assert_eq!(w(breath, -1, 0), breath, "the left edge");
        assert_eq!(w(sidestep, 1, 0), sidestep, "the right edge is the end");
        // 맨 윗줄에서 위 = 판 줄, 판 줄에서 위·옆은 판 줄(옆 = 판 바꾸기는 부른 쪽이).
        assert_eq!(w(breath, 0, -1), STRIP);
        assert_eq!(w(sidestep, 0, -1), STRIP);
        assert_eq!(w(STRIP, 0, -1), STRIP);
        assert_eq!(w(STRIP, 1, 0), STRIP);
        assert_eq!(
            walk(body, STRIP, 0, 1, true, catch),
            catch,
            "back where it left"
        );
        assert_eq!(
            walk(body, STRIP, 0, 1, true, at(Entry::Skill(Id::Whirl))),
            breath,
            "not on this plate — the top left"
        );
        // 아래로: 첫 줄 → 둘째 줄 → … → 맺음.
        let closing = at(Entry::Cell(Cell::ClosingBlow));
        assert_eq!(w(breath, 0, 1), closing);
        assert_eq!(w(closing, 0, -1), breath);
        let measure = at(Entry::Key(crate::progression::Key::Measure));
        let rooted = at(Entry::Key(crate::progression::Key::Rooted));
        assert_eq!(
            w(measure, 1, 0),
            at(Entry::Key(crate::progression::Key::Edge))
        );
        assert_eq!(w(measure, 0, 1), RESPEC);
        assert_eq!(
            walk(body, measure, 0, 1, false, 0),
            measure,
            "no button in the pause"
        );
        assert_eq!(w(rooted, 1, 0), rooted);
        assert_eq!(walk(body, RESPEC, 0, -1, true, measure), measure);
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1)] {
            assert_eq!(walk(body, RESPEC, dx, dy, true, measure), RESPEC);
        }
        for (i, _) in spots_on(body) {
            assert_ne!(w(i, 1, 0), RESPEC, "{:?}", ENTRIES[i]);
            assert_ne!(w(i, 1, 0), STRIP, "{:?}", ENTRIES[i]);
        }
        assert_eq!(w(breath, 0, 0), breath);
    }

    /// 무기 판(0270 C2)의 걷기: 같은 줄은 옆으로, 갈래에 칸이 빈 줄도 위아래로 막히지 않고(그쪽 줄의 가장 가까운 칸), 맨 윗줄에서 위 = 판 줄.
    /// 어느 판에서든 어느 칸·판 줄·단추에서 떠나도 그 판의 모든 칸과 판 줄·단추에 닿는다.
    #[test]
    fn walking_reaches_every_cell_of_every_plate_from_anywhere() {
        let duel = SWORD;
        let w = |from, dx, dy| walk(duel, from, dx, dy, true, 0);
        // 0274 ③: 한손검 판 = 왼쪽 충격(자루 치기 · 밀치기 · 결) · 가운데(휘돌이 · 끌어모으기 / — / 몰아치기) · 오른쪽 흡수(버팀 · 재기 · 결).
        let whirl = at(Entry::Skill(Id::Whirl));
        let (brace, rally, shove) = (
            at(Entry::Skill(Id::Brace)),
            at(Entry::Skill(Id::Rally)),
            at(Entry::Skill(Id::Shove)),
        );
        let flurry = at(Entry::Ult(Ult::Flurry));
        assert_eq!(w(whirl, 0, -1), STRIP);
        assert_eq!(w(brace, 1, 0), brace, "the right edge");
        assert_eq!(
            w(brace, 0, 1),
            rally,
            "the right branch goes down its column"
        );
        // 셋째 줄 왼쪽은 충격의 결.
        assert_eq!(
            w(shove, 0, 1),
            at(Entry::Vein(crate::progression::Vein::Shock)),
            "the left branch goes down its column"
        );
        assert_eq!(w(flurry, 0, 1), RESPEC);
        // 쌍단검 판(0274 ③) = 왼쪽 날카로움(바늘땀 · 제비걸음 · 결) · 가운데(끌어모으기 · 빠지며 베기 · 그림자 습격) · 오른쪽 균열
        // (가로지르기 · 던지기 · 결) — 옛 한손검 판의 넷이 옮겨 와 세 줄이 다 찬다.
        let dag = Plate::Class(Class::Dagger);
        let d = |from, dx, dy| walk(dag, from, dx, dy, true, 0);
        let (stitch, cut, swallow, throw, slip) = (
            at(Entry::Skill(Id::Stitch)),
            at(Entry::Skill(Id::Cut)),
            at(Entry::Skill(Id::Swallow)),
            at(Entry::Skill(Id::Throw)),
            at(Entry::Skill(Id::Slip)),
        );
        assert_eq!(d(stitch, 0, -1), STRIP);
        assert_eq!(d(cut, 0, -1), STRIP);
        assert_eq!(d(cut, 1, 0), cut, "the right edge");
        assert_eq!(d(cut, 0, 1), throw);
        assert_eq!(d(swallow, 1, 0), slip);
        assert_eq!(d(slip, 1, 0), throw);
        assert_eq!(
            d(swallow, 0, 1),
            at(Entry::Vein(crate::progression::Vein::Sharp))
        );
        assert_eq!(d(at(Entry::Ult(Ult::Ambush)), 0, 1), RESPEC);
        for plate in PLATES {
            // 위아래 걸음은 줄을 건너뛰지 않는다: 어느 칸에서든 바로 다음 줄로. 옆 걸음은 줄을 떠나지 않는다.
            let sp = spots_on(plate);
            for &(i, here) in &sp {
                for dx in [1, -1] {
                    let to = spot_on(plate, walk(plate, i, dx, 0, true, 0)).unwrap();
                    assert_eq!(
                        to.c.y, here.c.y,
                        "{plate:?} {:?} leaves its row",
                        ENTRIES[i]
                    );
                }
                for dy in [1, -1] {
                    let next = sp
                        .iter()
                        .map(|(_, s)| (s.c.y - here.c.y) * dy as f32)
                        .filter(|&d| d > 0.5)
                        .fold(f32::INFINITY, f32::min);
                    if let Some(&(_, to)) = sp
                        .iter()
                        .find(|(j, _)| *j == walk(plate, i, 0, dy, true, 0))
                    {
                        assert!(
                            ((to.c.y - here.c.y) * dy as f32 - next).abs() < 0.5,
                            "{plate:?} {:?} skips a row",
                            ENTRIES[i]
                        );
                    } else {
                        assert!(
                            next.is_infinite(),
                            "{plate:?} {:?} leaves the grid early",
                            ENTRIES[i]
                        );
                    }
                }
            }
            let cells: Vec<usize> = spots_on(plate).into_iter().map(|(i, _)| i).collect();
            let mut starts = cells.clone();
            starts.extend([STRIP, RESPEC]);
            for &start in &starts {
                let mut seen = vec![start];
                let mut todo = vec![start];
                while let Some(i) = todo.pop() {
                    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                        let j = walk(plate, i, dx, dy, true, 0);
                        assert!(j == STRIP || j == RESPEC || cells.contains(&j));
                        if !seen.contains(&j) {
                            seen.push(j);
                            todo.push(j);
                        }
                    }
                }
                assert_eq!(seen.len(), cells.len() + 2, "{plate:?} from {start}");
            }
        }
    }

    /// 탭을 열 때 설 칸(0161 → 0270 C2 — 판마다): 화로에서는 그 판에서 지금 살 수 있는 첫 칸(읽는 차례), 살 것이 없거나 읽기
    /// 전용(멈춤)이면 배운 첫 스킬, 그것도 없으면 왼쪽 위 칸.
    #[test]
    fn a_plate_opens_on_its_first_buyable_cell_or_first_learned_skill() {
        let duel = SWORD;
        let b = Board::default();
        // 한손검 판의 첫 줄(읽는 차례: 왼쪽 자루 치기 → 가운데 휘돌이 · 끌어모으기 → 오른쪽 버팀 — 0274 ③).
        let pommel = at(Entry::Skill(Id::Pommel));
        assert_eq!(first_focus_on(duel, &b, 5, true), pommel);
        assert_eq!(first_focus_on(duel, &b, 0, true), pommel, "nothing to buy");
        assert_eq!(first_focus_on(duel, &b, 5, false), pommel, "view only");
        let mut full = Board::default();
        full.rank[Id::Pommel.index()] = sk::MAX_RANK;
        full.rank[Id::Whirl.index()] = sk::MAX_RANK;
        assert_eq!(
            first_focus_on(duel, &full, 15, true),
            at(Entry::Skill(Id::Gather))
        );
        assert_eq!(first_focus_on(duel, &full, 10, true), pommel, "all spent");
        let mut gather = Board::default();
        gather.rank[Id::Gather.index()] = 1;
        assert_eq!(
            first_focus_on(duel, &gather, 1, false),
            at(Entry::Skill(Id::Gather))
        );
        // 몸의 판: 길이 이어진 칸이 없으면(스킬을 아직 모른다) 살 것이 없다 — 왼쪽 위 칸.
        let breath = at(Entry::Cell(Cell::EvenBreath));
        assert_eq!(first_focus_on(Plate::Body, &b, 5, true), breath);
        assert_eq!(first_focus_on(Plate::Body, &started(), 5, true), breath);
        assert_eq!(
            first_focus_on(Plate::Body, &Board::maxed(Pick::A), 99, true),
            breath
        );
    }

    /// 확인의 목록에는 되는 일만: 화로에서 살 수 있으면 올리기·열기가 맨 앞(으뜸), 배운 스킬은 끼우지 않은 칸에 끼우기,
    /// 3단부터 고르기 바꾸기(화로에서만). 멈춤에서는 끼우기만 — 점수가 있어 올릴 수 있는 칸이면 끝에 "쉼터 화로에서" 줄(0173),
    /// 잠긴 칸·점수가 모자란 칸은 빈 목록(확인 = 까닭 알림).
    #[test]
    fn the_action_list_holds_only_what_applies_with_the_primary_row_first() {
        let worn = sk::START;
        let kits = Kits {
            main: (Grip::Class(Class::Sword), worn),
            sub: None,
        };
        let b = started();
        let acts =
            |b: &Board, total, e, edit| node_acts(on(e), b, total, &kits, Ult::Upend, e, edit);
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
            acts(&b, 3, Entry::Cell(Cell::ClosingBlow), true).is_empty(),
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
        assert!(node_acts(
            SWORD,
            &all,
            99,
            &kits,
            Ult::Line,
            Entry::Ult(Ult::Line),
            false
        )
        .is_empty());
        // 0223 V3: 스킬은 그 갈래의 무기 칸에만 — 한손 검으로는 땅울림(양손)을 끼울 줄이 없고 까닭 줄이 선다. 망치를 부무기로
        // 들면 부무기의 칸(2·3)에 끼우는 줄이 서고, 두 무기에 다 되는 스킬은 네 칸 모두.
        assert_eq!(
            node_acts(
                SWORD,
                &all,
                99,
                &kits,
                Ult::Upend,
                Entry::Skill(Id::Quake),
                false
            ),
            [Do::Unfit]
        );
        let both = Kits {
            main: (Grip::Class(Class::Sword), worn),
            sub: Some((Grip::Class(Class::Hammer), [Id::Quake, Id::Gather])),
        };
        let at = |id| node_acts(SWORD, &all, 99, &both, Ult::Upend, Entry::Skill(id), false);
        assert_eq!(
            at(Id::Quake),
            [Do::Wear(3)],
            "already in the hammer's first slot"
        );
        assert_eq!(
            at(Id::Brace),
            [Do::Wear(0), Do::Wear(1), Do::Wear(2), Do::Wear(3)]
        );
        assert_eq!(at(Id::Pommel), [Do::Wear(0), Do::Wear(1)], "one hand only");
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
            look(
                on(Entry::Skill(Id::Whirl)),
                &fresh,
                2,
                Entry::Skill(Id::Whirl)
            ),
            (Look::Ready, true)
        );
        assert_eq!(
            look(
                on(Entry::Skill(Id::Whirl)),
                &fresh,
                0,
                Entry::Skill(Id::Whirl)
            ),
            (Look::Short, false)
        );
        let b = started();
        assert_eq!(
            look(on(Entry::Skill(Id::Whirl)), &b, 5, Entry::Skill(Id::Whirl)),
            (Look::Owned, true)
        );
        assert_eq!(
            look(on(Entry::Skill(Id::Whirl)), &b, 0, Entry::Skill(Id::Whirl)),
            (Look::Owned, false)
        );
        assert_eq!(
            look(
                on(Entry::Skill(Id::Gather)),
                &b,
                5,
                Entry::Skill(Id::Gather)
            ),
            (Look::Ready, true)
        );
        assert_eq!(
            look(
                on(Entry::Skill(Id::Gather)),
                &b,
                0,
                Entry::Skill(Id::Gather)
            ),
            (Look::Short, false)
        );
        assert_eq!(
            look(on(Entry::Skill(Id::Quake)), &b, 5, Entry::Skill(Id::Quake)),
            (Look::Locked, false)
        );
        assert_eq!(
            look(
                on(Entry::Cell(Cell::EvenBreath)),
                &b,
                5,
                Entry::Cell(Cell::EvenBreath)
            ),
            (Look::Ready, true)
        );
        assert_eq!(
            look(
                on(Entry::Cell(Cell::FinalMeasure)),
                &b,
                5,
                Entry::Cell(Cell::FinalMeasure)
            ),
            (Look::Locked, false)
        );
        let all = Board::maxed(Pick::A);
        for e in ENTRIES {
            // 0214: 켜지 않은 맺음은 바꿀 수 있을 뿐(살 것이 아니다 — 흐리게).
            let want = match e {
                Entry::Key(k) if all.cells.key != Some(k) => (Look::Short, false),
                _ => (Look::Owned, false),
            };
            assert_eq!(look(on(e), &all, 99, e), want, "{e:?}");
        }
    }

    /// 상태 줄: 화로에서는 살 수 있으면 값과 함께, 멈춤에서는 어디서 올리는지. 잠긴 칸은 몇 점 더, 다 익힌 칸·연 칸은 그렇다고.
    #[test]
    fn the_state_line_says_what_can_be_done_or_why_not() {
        let s = s();
        let b = started();
        let whirl = Entry::Skill(Id::Whirl);
        assert_eq!(
            state_line(on(whirl), &b, 5, whirl, true).0,
            fill(s.sk_st_raise, &["1"])
        );
        assert_eq!(state_line(on(whirl), &b, 5, whirl, false).0, s.sk_st_view);
        assert_eq!(
            state_line(on(whirl), &b, 0, whirl, true).0,
            fill(s.sk_st_short, &["1"])
        );
        assert_eq!(
            state_line(
                on(Entry::Skill(Id::Quake)),
                &b,
                5,
                Entry::Skill(Id::Quake),
                true
            )
            .0,
            fill(s.sk_st_plate_locked, &["2"]),
            "the hammer's second row opens at 3 spent on its board — whirl is 1"
        );
        // 몸의 판은 쓴 점수 합(0270 C2 — 바꾸지 않았다): 셋째 줄은 8부터, 처음의 둘이 2.
        let last = Entry::Cell(Cell::FinalMeasure);
        assert_eq!(
            state_line(Plate::Body, &b, 5, last, true).0,
            fill(s.sk_st_locked, &["6"])
        );
        let flow = Entry::Cell(Cell::FlowingBeat);
        assert_eq!(
            state_line(on(flow), &b, 5, flow, true).0,
            fill(s.sk_st_open, &["1"])
        );
        // 0215: 길이 끊긴 칸 · 맺음은 그 갈래에 더. 몸의 판의 첫 줄 패시브는 뿌리(0270 C3).
        let catch = Entry::Cell(Cell::Catch);
        assert_eq!(
            state_line(on(catch), &b, 5, catch, true).0,
            fill(s.sk_st_open, &["1"])
        );
        let mut deep = b;
        deep.rank[Id::Whirl.index()] = 4;
        let closing = Entry::Cell(Cell::ClosingBlow);
        assert_eq!(
            state_line(on(closing), &deep, 9, closing, true).0,
            s.sk_st_cut
        );
        let all = Board::maxed(Pick::A);
        assert_eq!(state_line(on(whirl), &all, 99, whirl, true).0, s.board_full);
        assert_eq!(
            state_line(on(catch), &all, 99, catch, true).0,
            s.sk_st_open_cell
        );
        let link = Entry::Link(crate::progression::Link::ParryStep);
        assert_eq!(
            state_line(on(link), &all, 99, link, true).0,
            s.sk_st_open_cell
        );
        let mut keyless = all;
        keyless.cells.key = None;
        keyless.rank[Id::Brace.index()] = 0;
        keyless.rank[Id::Echo.index()] = 0;
        keyless.rank[Id::Bash.index()] = 0;
        keyless.rank[Id::Ram.index()] = 0;
        keyless.rank[Id::Toll.index()] = 0;
        keyless.rank[Id::Ripple.index()] = 0;
        // 0274: 한손검의 자루 치기 · 재기도 맞받기 갈래.
        keyless.rank[Id::Pommel.index()] = 0;
        keyless.rank[Id::Rally.index()] = 0;
        keyless.ult[Ult::Line.index()] = 0;
        keyless.ult[Ult::Bell.index()] = 0;
        keyless.rank[Id::Pin.index()] = 0;
        keyless.ult[Ult::Gale.index()] = 0;
        let edge = Entry::Key(crate::progression::Key::Edge);
        // 맞받기에 남은 것은 패시브 일곱.
        assert_eq!(keyless.branch_spent(crate::progression::Branch::Riposte), 7);
        assert_eq!(
            state_line(on(edge), &keyless, 99, edge, true).0,
            fill(s.sk_st_branch, &["3"])
        );
        // 큰 기술을 배우는 첫 단은 2점 — 지팡이 판의 셋째 줄(그 판에 쓴 점수 6).
        let staff = Plate::Class(Class::Staff);
        let line = Entry::Ult(Ult::Blizzard);
        let mut open = Board::default();
        open.rank[Id::Gather.index()] = 5;
        assert_eq!(
            state_line(staff, &open, 13, line, true).0,
            fill(s.sk_st_plate_locked, &["1"])
        );
        open.rank[Id::Brace.index()] = 1;
        assert_eq!(
            state_line(staff, &open, 8, line, true).0,
            fill(s.sk_st_raise, &["2"])
        );
        assert_eq!(
            state_line(staff, &open, 7, line, true).0,
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
            fill(s.sk_kind_skill, &[s.sk_physical, "7"])
        );
        b.rank[Id::Whirl.index()] = 5;
        assert!(kind_line(&b, Entry::Skill(Id::Whirl)).contains("6.3"));
        assert_eq!(
            kind_line(&b, Entry::Ult(Ult::Upend)),
            fill(s.sk_kind_ult, &[s.class_hammer])
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
