//! 큰 창의 글이 칸에 드는가(0275) — GL 없이 글꼴만 읽어 잰다.
//!
//! 글을 놓는 길은 셋이다: **줄 바꿈**(`ui::wrap` — 오른쪽 칸의 몸) · **세로로 굴리기**(`ListView`·`draw_pane`) · **가로로 굴리기**
//! (`ui::Strip` — 큰 창의 탭 줄·도감의 묶음·기술 탭의 판 줄). 셋 다 넘칠 수 없다. 넘칠 수 있는 곳은 한 줄짜리 고정 칸뿐이고
//! (넘치면 `Ui::fit`이 말없이 "…"로 자른다), 그 칸들을 여기서 본다 — 칸의 폭(기준 px, 창의 배 1 — 좁은 화면에서는 글과 칸이
//! 같은 배로 준다)과 그 칸에 서는 모든 글(한국어·영어). 줄 바꿈·굴리기 쪽은 "한 낱말이 줄 폭보다 넓지 않은가"만(넓으면 낱말
//! 가운데서 끊긴다).
//!
//! 폭의 셈: `Ui::width`와 같다 — 글자마다의 advance(글꼴 단위) × 글 크기 / em, 키캡은 `ui::width_with`. macroquad의
//! `measure_text`도 커닝을 빼고 advance만 더한다(그리기만 커닝을 넣는다 — 이 글꼴의 커닝은 좁히는 쪽). DPI가 1이 아닌 화면은
//! 굽는 크기의 올림(`ceil`)으로 1% 안쪽만큼 넓게 잰다 — 그 몫은 칸마다의 여유(`SLACK`)가 받는다. 이 셈이 fontdue(macroquad가
//! 쓰는 것)의 값과 같은지는 `the_meter_matches_what_macroquad_measures`가 본다.
//!
//! 헤드리스 `tools/test/hearth.js`가 칸마다 F4 넘침 검사를 돌던 것(언어마다 25분)을 이 시험이 대신한다 — 스크립트는 흐름만.
use super::*;
use crate::i18n::Lang;
use crate::skills::{Board, Entry, Pick, ENTRIES};

const FONT: &[u8] = include_bytes!("../../assets/fonts/GowunBatang-UI-Regular.ttf");
const KO_SRC: &str = include_str!("../i18n/ko.rs");
const EN_SRC: &str = include_str!("../i18n/en.rs");
/// 반올림·DPI 올림의 몫(기준 px).
const SLACK: f32 = 0.5;
/// 키 안내 줄이 자르기 전에 줄여 보는 가장 작은 배(`Ui::hints_left`).
const HINT_MIN_K: f32 = 0.6;

/// 글꼴만 읽은 자.
struct Meter {
    face: ttf_parser::Face<'static>,
    upem: f32,
}

impl Meter {
    fn new() -> Meter {
        let face = ttf_parser::Face::parse(FONT, 0).expect("UI font");
        let upem = face.units_per_em() as f32;
        Meter { face, upem }
    }

    /// 글 크기 1px의 폭(표식 없는 조각). 글꼴에 없는 글자는 fontdue처럼 0번 글리프.
    fn em(&self, s: &str) -> f32 {
        s.chars()
            .map(|c| {
                let g = self.face.glyph_index(c).unwrap_or(ttf_parser::GlyphId(0));
                self.face.glyph_hor_advance(g).unwrap_or(0) as f32
            })
            .sum::<f32>()
            / self.upem
    }

    /// `Ui::width(s, k)`를 UI 배율 1(기준 px)로.
    fn width(&self, s: &str, k: f32) -> f32 {
        let px = ui::GLYPH * k;
        ui::width_with(s, px, |seg| self.em(seg) * px)
    }
}

/// 언어 표의 키 ↔ 글(실패 보고에 키 이름을 쓰려고 원본 `ko.rs`·`en.rs`를 읽는다).
fn table(src: &str) -> Vec<(String, String)> {
    let start = src.find("Strings {").expect("table");
    let text = &src[start..];
    let mut out = Vec::new();
    let mut off = 0;
    for line in text.split_inclusive('\n') {
        let at = off;
        off += line.len();
        let Some(rest) = line.strip_prefix("    ") else {
            continue;
        };
        let Some((name, _)) = rest.split_once(':') else {
            continue;
        };
        if name.is_empty()
            || !name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        {
            continue;
        }
        let from = at + 4 + name.len() + 1;
        let Some(q) = text[from..].find('"') else {
            continue;
        };
        let mut val = String::new();
        let mut it = text[from + q + 1..].chars();
        while let Some(c) = it.next() {
            match c {
                '"' => break,
                '\\' => match it.next() {
                    Some('n') => val.push('\n'),
                    Some('t') => val.push('\t'),
                    Some('u') => {
                        let hex: String = it.by_ref().skip(1).take_while(|c| *c != '}').collect();
                        val.extend(char::from_u32(u32::from_str_radix(&hex, 16).unwrap_or(0)));
                    }
                    // 줄 끝의 \ — 다음 줄의 앞 공백까지 건너뛴다.
                    Some('\n') => {
                        let rest: String = it.clone().collect();
                        let skip = rest.len() - rest.trim_start().len();
                        for _ in 0..rest[..skip].chars().count() {
                            it.next();
                        }
                    }
                    Some(c) => val.push(c),
                    None => {}
                },
                c => val.push(c),
            }
        }
        out.push((name.to_string(), val));
    }
    out
}

/// 자리표시자를 넓은 수로(값이 커져도 들어가야 한다).
fn wide(t: &str) -> String {
    fill(t, &["999", "999", "999", "999"])
}

/// 한 언어의 검사: 잰 칸 수와 넘친 칸.
struct Check<'a> {
    meter: &'a Meter,
    lang: Lang,
    keys: Vec<(String, String)>,
    fails: Vec<String>,
    count: usize,
}

impl Check<'_> {
    fn key(&self, text: &str) -> &str {
        self.keys
            .iter()
            .find(|(_, v)| v == text)
            .map_or("·", |(k, _)| k.as_str())
    }

    /// 한 줄 고정 칸: `text`가 배 `k`로 `room`(기준 px) 안에 드나.
    fn fit(&mut self, slot: &str, text: &str, k: f32, room: f32) {
        self.count += 1;
        let w = self.meter.width(text, k);
        if w > room + SLACK {
            let key = self.key(text).to_string();
            let line = format!(
                "{:?} {slot}: {key} {:?} — {w:.1} > {room:.1}",
                self.lang,
                ui::plain(text)
            );
            if !self.fails.contains(&line) {
                self.fails.push(line);
            }
        }
    }

    fn all<S: AsRef<str>>(
        &mut self,
        slot: &str,
        texts: impl IntoIterator<Item = S>,
        k: f32,
        room: f32,
    ) {
        for t in texts {
            self.fit(slot, t.as_ref(), k, room);
        }
    }

    /// 줄을 바꾸는 글이 `max`줄 안에 드나(`ui::wrap`과 같은 셈).
    fn lines(&mut self, slot: &str, text: &str, k: f32, room: f32, max: usize) {
        self.count += 1;
        let lines = ui::wrap_with(text, room, |t| self.meter.width(t, k));
        if lines.len() > max {
            let key = self.key(text).to_string();
            self.fails.push(format!(
                "{:?} {slot}: {key} {:?} — {} lines > {max} at {room:.1}",
                self.lang,
                ui::plain(text),
                lines.len()
            ));
        }
        self.words(slot, text, k, room);
    }

    /// 줄을 바꾸는 글: 낱말 하나가 줄 폭보다 넓지 않은가(넓으면 낱말 가운데서 끊긴다).
    fn words(&mut self, slot: &str, text: &str, k: f32, room: f32) {
        for word in text.split_whitespace() {
            if word.contains('{') {
                continue;
            }
            self.count += 1;
            let w = self.meter.width(word, k);
            if w > room + SLACK {
                let key = self.key(text).to_string();
                self.fails.push(format!(
                    "{:?} {slot}: {key} word {word:?} — {w:.1} > {room:.1}",
                    self.lang
                ));
            }
        }
    }
}

/// 이름들: 기술 칸 · 판 · 무기 · 방패 · 장신구 · 덧옷.
fn entry_names() -> Vec<&'static str> {
    ENTRIES.iter().map(|&e| skills::entry_text(e).0).collect()
}

fn weapon_names() -> Vec<&'static str> {
    crate::weapons::IDS
        .iter()
        .filter(|&&w| w != crate::weapons::Id::Fists)
        .map(|&w| weapon_name(w))
        .collect()
}

fn charm_names() -> Vec<&'static str> {
    crate::charms::TABLE.iter().map(|d| tr(d.name)).collect()
}

fn shield_names() -> Vec<&'static str> {
    crate::shield::TABLE.iter().map(|d| tr(d.name)).collect()
}

fn garb_names() -> Vec<&'static str> {
    crate::garb::TABLE.iter().map(|d| tr(d.name)).collect()
}

/// 등급이 붙는 아이템의 이름("고유 종치기 망치") — 알림 글이 쓴다(큰 창의 머리·후보 줄은 등급을 갈래 줄·둘째 줄에, 0275).
fn item_labels() -> Vec<String> {
    use crate::items::Rarity;
    let bases: Vec<&str> = weapon_names()
        .into_iter()
        .chain(shield_names())
        .chain(charm_names())
        .collect();
    let mut v = Vec::new();
    for r in [Rarity::Plain, Rarity::Sturdy, Rarity::Rare, Rarity::Unique] {
        v.extend(bases.iter().map(|b| crate::i18n::item_label(r, b)));
    }
    v
}

/// 칸 이름(장비·가방의 갈래 줄).
fn slot_names() -> Vec<String> {
    let s = s();
    let mut v = vec![
        s.menu_weapon.to_string(),
        s.menu_sub_weapon.to_string(),
        s.menu_garb.to_string(),
    ];
    v.extend(
        crate::charms::SLOT_KINDS
            .iter()
            .map(|k| tr(k.name()).to_string()),
    );
    v
}

/// 등급 말을 앞에 붙인 갈래 줄들("고급 · 주무기").
fn with_rarities(kinds: &[String]) -> Vec<String> {
    use crate::items::Rarity;
    let mut v = Vec::new();
    for r in [Rarity::Plain, Rarity::Sturdy, Rarity::Rare, Rarity::Unique] {
        v.extend(kinds.iter().map(|k| crate::i18n::with_rarity(Some(r), k)));
    }
    v
}

/// 잠깐 뜨는 알림(`App::toast`): 큰 창에서는 오른쪽 칸 머리의 상태 줄 자리에 대신 뜬다 — 넓으면 갈래 줄까지 두 줄(`draw_head_lines`).
/// 이름을 받는 알림의 `{0}`은 가장 긴 이름으로, 나머지 자리는 넓은 수로.
fn toasts(c: &Check) -> Vec<String> {
    const NAMED: &[&str] = &[
        "toast_bought",
        "toast_charm",
        "toast_weapon",
        "toast_level",
        "toast_cell",
        "toast_key",
        "toast_rank",
        "toast_item",
        "toast_put_down",
        "toast_kept",
        "toast_unkept",
        "toast_find_garb",
        "toast_find_garb_kept",
        "toast_forged",
        "toast_journey",
        "toast_journey_took",
        "toast_codex_took",
    ];
    let names: Vec<String> = item_labels()
        .into_iter()
        .chain(crate::weapons::IDS.iter().map(|&w| forged_label(w, 3)))
        .chain(entry_names().into_iter().map(String::from))
        .chain(garb_names().into_iter().map(String::from))
        .chain(
            crate::journey::GOALS
                .iter()
                .map(|g| tr(g.title).to_string()),
        )
        .chain(
            crate::codex::ENTRIES
                .iter()
                .map(|e| tr(crate::species::def(e.kind).name).to_string()),
        )
        .collect();
    let longest = names
        .iter()
        .max_by(|a, b| c.meter.width(a, 1.0).total_cmp(&c.meter.width(b, 1.0)))
        .cloned()
        .unwrap_or_default();
    let s = s();
    let extra = [
        s.shop_poor,
        s.bag_full,
        s.board_respec_none,
        s.forge_no_stone,
        s.save_failed,
        s.tut_reward,
    ];
    c.keys
        .iter()
        .filter(|(k, _)| k.starts_with("toast_"))
        .map(|(k, v)| {
            if NAMED.contains(&k.as_str()) {
                fill(v, &[&longest, "999", "999"])
            } else {
                wide(v)
            }
        })
        .chain(extra.iter().map(|t| wide(t)))
        // 거두기의 알림은 무기 이름(등급 없이)과 돌려받는 숫돌.
        .chain(weapon_names().iter().map(|w| {
            fill(
                s.toast_recalled,
                &[w, &crate::forge::cost_to(3).0.to_string()],
            )
        }))
        .collect()
}

/// 넓은 값(비교 줄 "값 → 값 ▲"의 값).
fn stat_values() -> Vec<(&'static str, String)> {
    use crate::stats::LINES;
    LINES
        .iter()
        .map(|&l| {
            let probe = equip::stat_value(l, 1.0);
            let v = if probe.ends_with('%') {
                1.35
            } else if probe.contains('.') || probe.contains(&*fill(s().value_secs, &[""])) {
                0.12
            } else {
                999.0
            };
            let a = equip::stat_value(l, v);
            (equip::stat_name(l), format!("{a} → {a} ▲"))
        })
        .collect()
}

/// 오른쪽 칸 몸의 이름 · 값 줄(`Ln::Pair`): 이름은 값 앞까지(`draw_pane`).
fn pair_room(c: &Check, value: &str) -> f32 {
    tabs::RIGHT_W - c.meter.width(value, tabs::K_BODY) - 4.0
}

/// 왼쪽 줄 목록(`ui::list_k`)의 이름 폭: 줄 폭 `w`(기준 px)에서 앞 여백(▶·그림 칸)과 오른쪽 값을 뺀 것.
fn row_room(c: &Check, w: f32, row: &ui::Row) -> f32 {
    let px = ui::GLYPH;
    let right = w - 6.0;
    let label_x = 4.0 + px * 1.1 + if row.icon { px * 1.4 } else { 0.0 };
    let value_left = match row.kind {
        ui::RowKind::Step => {
            let arrow = c.meter.width("▶", 1.0) + 6.0;
            let vw = c.meter.width(&row.value, 1.0).max(px * 3.0) + 6.0;
            right - arrow - vw - arrow
        }
        ui::RowKind::Go => right - c.meter.width("→", 1.0),
        ui::RowKind::Plain if row.value.is_empty() => right,
        ui::RowKind::Plain => {
            let vk = if row.tag { 0.8 } else { 1.0 };
            right - c.meter.width(&row.value, vk)
        }
    };
    value_left - label_x - 6.0
}

fn rows(c: &mut Check, slot: &str, w: f32, rows: &[ui::Row]) {
    for r in rows {
        let room = row_room(c, w, r);
        c.fit(slot, &r.label, 1.0, room);
    }
}

/// 오른쪽 값을 작은 표지로(`Row::tag` — 여정·도감).
fn tagged(mut r: ui::Row) -> ui::Row {
    r.tag = true;
    r
}

/// 왼쪽 칸의 줄 목록 폭(`tabs::draw_left_list`).
const LIST_W: f32 = tabs::LEFT_W - tabs::LIST_GUTTER;

/// 큰 창 틀: 탭 이름(넘치면 굴리는 줄 — 이름 하나는 보이는 창 안에), 지갑, 아래 안내 줄, "보기만" 표지.
fn window_frame(c: &mut Check) {
    let s = s();
    let purse = format!("{} 999  ·  {} 99999", s.sk_purse_points, s.sk_purse_embers);
    c.fit("purse", &purse, 1.0, frame::WINDOW_W * 0.5);
    let purse_w = c.meter.width(&purse, 1.0);
    let arrow = c.meter.width("◀", 1.0) + 6.0;
    let view = frame::WINDOW_W - 28.0 - purse_w - 8.0 - 10.0 - 2.0 * arrow;
    let names: Vec<&str> = flow::tabs(flow::Host::Hearth)
        .iter()
        .chain(flow::tabs(flow::Host::Pause))
        .map(|&t| App::tab_name(t))
        .collect();
    c.all("tab head", names, 1.0, view - 10.0);
    // "보기만"(멈춤의 상태·기술 탭 아래 줄 오른쪽 끝).
    let note = s.view_note;
    c.fit("view note", note, tabs::K_KIND, frame::WINDOW_W * 0.5);
    // 아래 안내 줄: 탭의 일 · 탭 넘기기 · 닫기 — 넓으면 줄여(0.6배까지) 보고 그래도 넘치면 자른다. 오른쪽 끝은 되돌리기 단추·"보기만".
    let note_w = c.meter.width(note, tabs::K_KIND) + 2.0 + 8.0;
    let cap = |a: input::Act, v: &str| format!("{} {v}", ui::key(&input::prompt(a)));
    let foot = |pairs: &[(input::Act, &str)]| {
        let mut all: Vec<String> = pairs.iter().map(|(a, v)| cap(*a, v)).collect();
        all.push(cap(input::Act::Tabs, s.verb_tab));
        all.push(cap(input::Act::Back, s.verb_close));
        all.join("   ")
    };
    use input::Act;
    let walk = [(Act::Walk, s.verb_choose), (Act::Confirm, s.verb_confirm)];
    let lines = [
        // 기술(화로 — 되돌리기 단추) · 멈춤("보기만").
        (
            foot(&walk),
            frame::WINDOW_W - skills::geo::RESPEC_W - 8.0 - 2.0,
        ),
        (foot(&walk), frame::WINDOW_W - note_w - 2.0),
        // 장비(칸 고르기 — 빨리 바꾸기까지).
        (
            foot(&[
                (Act::Walk, s.verb_choose),
                (Act::Confirm, s.verb_confirm),
                (Act::Swap, s.verb_swap),
            ]),
            frame::WINDOW_W - 2.0,
        ),
        (
            foot(&[(Act::Choose, s.verb_choose), (Act::Confirm, s.ask_buy_yes)]),
            frame::WINDOW_W - 2.0,
        ),
    ];
    for (line, room) in lines {
        c.fit("foot hints", &line, HINT_MIN_K, room);
    }
}

/// 오른쪽 칸의 머리(`tabs::draw_pane_head`): 이름 · 갈래 줄 · 상태 줄 — 그림 칸이 있으면 그만큼 좁다. 등급은 이름이 아니라
/// 갈래 줄에(0275). 알림은 상태 줄 자리에 — 한 줄에 안 들면 갈래 줄까지 두 줄.
fn pane_heads(c: &mut Check) {
    let s = s();
    let full = tabs::RIGHT_W;
    let beside = tabs::RIGHT_W - tabs::HEAD_ICON - 6.0;
    let (kn, kk) = (tabs::K_NAME, tabs::K_KIND);
    // 이름(그림 곁): 무기·방패·장신구·덧옷(장비·가방·가게·벼리기).
    let icon_names: Vec<&str> = weapon_names()
        .into_iter()
        .chain(shield_names())
        .chain(charm_names())
        .chain(garb_names())
        .collect();
    c.all("pane name (icon)", icon_names, kn, beside);
    // 이름(그림 없이): 빈 칸·맨손 · 가방의 단추 · 레벨 줄 · 여정 · 도감 · 가게의 기름·방패.
    let plain: Vec<String> = [
        s.charm_empty,
        s.garb_none,
        s.weapon_fists,
        s.bag_tidy,
        s.bag_empty,
        s.status_passives,
        s.shop_oil,
        s.codex_unknown,
    ]
    .iter()
    .map(|t| t.to_string())
    .chain(
        crate::progression::LINES
            .iter()
            .map(|&l| status::line_name(l).to_string()),
    )
    .chain(
        crate::journey::GOALS
            .iter()
            .map(|g| tr(g.title).to_string()),
    )
    .chain(
        crate::codex::ENTRIES
            .iter()
            .map(|e| tr(crate::species::def(e.kind).name).to_string()),
    )
    .chain(shield_names().into_iter().map(String::from))
    .collect();
    c.all("pane name", plain, kn, full);
    // 갈래 줄(그림 곁): 칸 이름 · 벼린 단 · 등급, 가게의 장신구, 벼리기.
    let mut slots = slot_names();
    slots.extend(slot_names().iter().map(|n| fill(s.forge_named, &[n, "3"])));
    let mut kinds = with_rarities(&slots);
    kinds.push(s.codex_tab_garb.to_string());
    kinds.push(fill(s.shop_kind, &[s.codex_tab_charms, "999"]));
    let (st, em) = crate::forge::COST
        .iter()
        .fold((0, 0), |a, &(x, y)| (a.0.max(x), a.1.max(y)));
    kinds.extend([
        fill(s.forge_kind, &[&st.to_string(), &em.to_string(), "3"]),
        fill(
            s.forge_kind_recall,
            &[&crate::forge::cost_to(3).0.to_string(), "3"],
        ),
        s.forge_kind_full.to_string(),
    ]);
    c.all("pane kind (icon)", &kinds, kk, beside);
    // 갈래 줄(그림 없이): 가방 · 가게의 기름·방패 · 도감 · 여정 · 레벨.
    let mut kinds = vec![s.menu_bag.to_string()];
    for t in [s.stat_flask, s.menu_shield] {
        kinds.push(fill(s.shop_kind, &[t, "999"]));
    }
    for k in crate::charms::SLOT_KINDS {
        kinds.push(format!("{} · {}", s.codex_tab_charms, tr(k.name())));
    }
    for g in &crate::journey::GOALS {
        let branch = match g.kind {
            crate::journey::Branch::Main => s.journey_main,
            crate::journey::Branch::Find => s.journey_find,
        };
        kinds.push(format!(
            "{branch} · {}",
            tr(crate::stage::STAGES[g.stage].name)
        ));
    }
    for &l in &crate::progression::LINES {
        kinds.push(format!("{} · {}", status::line_sub(l), wide(s.lv_rank_of)));
    }
    c.all("pane kind", &kinds, kk, full);
    // 상태 줄(그림 곁): 장비·가방의 아이템 · 가게·벼리기.
    let mut states: Vec<String> = [
        s.equip_spare,
        s.equip_worn,
        s.bag_st_new,
        s.bag_st_kept,
        s.bag_st_spare,
        s.bag_st_last,
        s.shop_sold,
        s.shop_can,
        s.shop_poor,
        s.forge_no_stone,
        s.forge_can,
        s.forge_full,
        s.forge_recall_note,
    ]
    .iter()
    .map(|t| t.to_string())
    .collect();
    states.extend(slot_names());
    c.all("pane state (icon)", &states, kk, beside);
    // 상태 줄(그림 없이): 여정 · 레벨 · 여벌 정리 · 빈 가방.
    let states: Vec<String> = [
        s.journey_claimed,
        s.journey_ready,
        s.lv_full_d,
        s.sk_st_view,
    ]
    .iter()
    .map(|t| t.to_string())
    .chain(
        [s.toast_tidy, s.journey_doing, s.lv_ready]
            .iter()
            .map(|t| wide(t)),
    )
    .chain([fill(s.bag_count, &["30", "30"])])
    .collect();
    c.all("pane state", &states, kk, full);
    // 알림: 두 줄 안(가장 좁은 머리 — 그림 곁).
    for t in toasts(c) {
        c.lines("toast (2 lines)", &t, kk, beside, 2);
    }
}

/// 오른쪽 칸 몸의 한 줄짜리(`Ln::Head`·`Ln::Pair`의 이름).
fn pane_lines(c: &mut Check) {
    let s = s();
    // 상태 탭의 패시브 묶음 머리: "◆ 맺음" · 연 칸의 이름.
    let heads: Vec<String> = entry_names()
        .into_iter()
        .map(|n| format!("◆ {n}"))
        .chain([s.equip_status.to_string()])
        .collect();
    c.all("pane head line", heads, tabs::K_BODY, tabs::RIGHT_W - 6.0);
    for (name, value) in stat_values() {
        let room = pair_room(c, &value);
        c.fit("pane pair", name, tabs::K_BODY, room);
    }
    // 무기의 숫자(견주면 "지금 → 다음 ▲").
    let st = crate::save::Progress::default().stats();
    let t = st.tune(&Tuning::default());
    let nums: Vec<Vec<equip::WVal>> = crate::weapons::IDS
        .iter()
        .map(|w| equip::weapon_numbers(w.def(), &t, &st))
        .collect();
    for a in &nums {
        for v in a {
            let widest = nums
                .iter()
                .flatten()
                .filter(|b| b.line == v.line)
                .map(|b| format!("{} → {} ▲", v.text, b.text))
                .max_by(|x, y| c.meter.width(x, 1.0).total_cmp(&c.meter.width(y, 1.0)))
                .unwrap_or_default();
            let room = pair_room(c, &widest);
            c.fit("pane pair (weapon)", v.line.name(), tabs::K_BODY, room);
        }
    }
    let value = format!("+{}", 999);
    let room = pair_room(c, &value);
    c.fit("pane pair", s.bag_value, tabs::K_BODY, room);
}

/// 줄을 바꾸는 글: 오른쪽 칸의 몸(`pane_wrap` — 폭 `RIGHT_W`, 배 `K_BODY`)과 기술 탭 고르기 상자의 한 줄.
fn wrapped(c: &mut Check) {
    let texts: Vec<String> = c.keys.iter().map(|(_, v)| v.clone()).collect();
    for t in &texts {
        c.words("pane body", t, tabs::K_BODY, tabs::RIGHT_W);
    }
    let box_w = (skills::geo::DETAIL_W - skills::RANK_NUM - 4.0) * 0.5 - 6.0;
    for &e in &ENTRIES {
        for d in pick_descs(e) {
            c.words("pick box", d, skills::K_PICK, box_w);
        }
    }
}

/// 3단 고르기 둘(스킬)·숫자 칸 고르기 둘(패시브)의 (표지, 이름, 한 줄).
fn picks(e: Entry) -> Vec<(&'static str, &'static str, &'static str)> {
    match e {
        Entry::Skill(id) => vec![
            skills::pick_text(id, Pick::A),
            skills::pick_text(id, Pick::B),
        ],
        Entry::Cell(cell) if cell.pick_b().is_some() => {
            vec![
                skills::cell_pick_text(cell, false),
                skills::cell_pick_text(cell, true),
            ]
        }
        _ => Vec::new(),
    }
}

fn pick_descs(e: Entry) -> Vec<&'static str> {
    picks(e).into_iter().map(|p| p.2).collect()
}

/// 기술 탭: 판 줄 · 갈래 머리 · 줄 띠 · 상세의 머리 · 고르기 상자 · 되돌리기 단추 · 칸의 일 목록.
fn skills_tab(c: &mut Check) {
    use crate::plates::{Plate, PLATES};
    use skills::geo;
    let s = s();
    // 판 줄(0275 — 넘치면 굴린다): 이름 하나(+ 표지 자리)는 보이는 창 안에.
    let arrow = c.meter.width("◀", skills::STRIP_K) + 4.0;
    let view = geo::GRID_W - 2.0 * arrow;
    for &p in &PLATES {
        c.fit(
            "plate strip",
            skills::plate_name(p),
            skills::STRIP_K,
            view - skills::STRIP_PAD,
        );
    }
    // 갈래 머리(몸의 판): 이름 · 한 줄. 무기 판: 대표 타입 둘 · 가운데 물리 속성.
    let col = geo::COL_W - 4.0;
    for b in crate::progression::BRANCHES {
        c.fit("branch head", tr(b.name()), 1.0, col);
        c.fit("branch motto", tr(b.motto()), 0.62, col);
    }
    for &p in &PLATES {
        if let Some([a, b]) = p.pair() {
            c.fit("plate column", skills::kind_name(a), 1.0, col);
            c.fit("plate column", skills::kind_name(b), 1.0, col);
        }
        if let Some(t) = p.attribute() {
            c.fit("plate column", tr(t), 1.0, col);
        }
    }
    // 줄 띠의 이름표(잠긴 줄 — 오른쪽 끝, ▼ 자리를 비운다).
    let band = geo::GRID_W - c.meter.width("▼", 0.6) - 4.0 - 20.0;
    c.fit("band label", &wide(s.sk_row_locked), 0.6, band);
    c.fit("band label", &wide(s.sk_row_plate), 0.6, band);
    // 상세의 머리: 칸이면 그림 곁, 판 줄·되돌리기면 칸 폭 전체.
    let beside = geo::DETAIL_W - geo::DETAIL_ICON - 6.0;
    let (kn, kk) = (tabs::K_NAME, tabs::K_KIND);
    c.all("detail name", entry_names(), kn, beside);
    let plate_names: Vec<&str> = PLATES.iter().map(|&p| skills::plate_name(p)).collect();
    c.all("detail name", plate_names, kn, geo::DETAIL_W);
    c.fit("detail name", s.board_respec, kn, geo::DETAIL_W);
    let boards = [
        Board::default(),
        Board::maxed(Pick::A),
        Board::maxed(Pick::B),
    ];
    let mut kinds = Vec::new();
    let mut states = Vec::new();
    for b in &boards {
        for &e in &ENTRIES {
            kinds.push(skills::kind_line(b, e));
            for &p in &PLATES {
                if p.has(e) {
                    for total in [0, 3, 99] {
                        for edit in [false, true] {
                            states.push(skills::state_line(p, b, total, e, edit).0);
                        }
                    }
                }
            }
        }
    }
    c.all("detail kind", &kinds, kk, beside);
    c.all("detail state", &states, kk, beside);
    let mut wide_kinds = vec![s.sk_plate_kind_body.to_string(), wide(s.sk_plate_spent)];
    for &p in &PLATES {
        if let Some([a, b]) = p.pair() {
            let t = if p == Plate::Shield {
                s.sk_plate_kind_shield
            } else {
                s.sk_plate_kind_grip
            };
            wide_kinds.push(fill(t, &[skills::kind_name(a), skills::kind_name(b)]));
        }
    }
    wide_kinds.extend([
        fill(s.shop_embers, &["999"]),
        s.board_respec_none.to_string(),
        s.shop_poor.to_string(),
        wide(s.sk_st_respec),
    ]);
    c.all(
        "detail kind/state (strip, respec)",
        &wide_kinds,
        kk,
        geo::DETAIL_W,
    );
    // 고르기 상자의 이름("ㄱ 이름").
    let bw = (geo::DETAIL_W - skills::RANK_NUM - 4.0) * 0.5;
    for &e in &ENTRIES {
        for (mark, name, _) in picks(e) {
            // 이름도 상자 폭에서 줄을 바꾼다(0275) — 두 줄 안.
            c.lines(
                "pick box name",
                &format!("{mark} {name}"),
                skills::K_RANK,
                bw - 6.0,
                2,
            );
        }
    }
    // 되돌리기 단추.
    let label = fill(s.sk_respec_button, &[s.board_respec, "999"]);
    c.fit("respec button", &label, 0.8, geo::RESPEC_W - 8.0);
    // 칸의 일 목록(작은 창 — 가장 긴 줄에 맞춰 넓어진다): 화면 안에 들어가나.
    let mut acts = Vec::new();
    for b in &boards {
        for &e in &ENTRIES {
            for d in [
                skills::Do::Buy,
                skills::Do::Wear(0),
                skills::Do::Wear(2),
                skills::Do::Unfit,
                skills::Do::WearUlt,
                skills::Do::Flip,
                skills::Do::Away,
            ] {
                for sub in [false, true] {
                    acts.push(skills::act_label(b, e, d, sub));
                }
            }
        }
    }
    acts.sort();
    acts.dedup();
    c.all(
        "skill act row",
        &acts,
        1.0,
        480.0 - 12.0 - 24.0 - ui::GLYPH * 1.1,
    );
}

/// 장비 탭: 후보의 작은 목록(`EquipPick` — 가장 긴 줄에 맞춰 넓어지되 왼쪽 칸 안). 줄 = 이름(벼린 단까지) + 작은 둘째 줄
/// (등급 · 낀 칸, 0275).
fn equip_tab(c: &mut Check) {
    let s = s();
    let names: Vec<String> = crate::weapons::IDS
        .iter()
        .map(|&w| forged_label(w, 3))
        .chain(shield_names().into_iter().map(String::from))
        .chain(charm_names().into_iter().map(String::from))
        .chain(garb_names().into_iter().map(String::from))
        .chain([s.charm_empty, s.garb_none].map(String::from))
        .collect();
    // 줄의 앞(▶·그림 칸 2.5글자)·창 안 여백(24) — `draw_equip`의 폭 셈. 넓어져도 왼쪽 칸(-4) 안.
    let room = tabs::LEFT_W - 4.0 - ui::GLYPH * 2.5 - 24.0;
    c.all("equip pick row", &names, 1.0, room);
    let tags: Vec<String> = slot_names()
        .into_iter()
        .chain([s.equip_worn.to_string()])
        .collect();
    let subs: Vec<String> = with_rarities(&tags);
    c.all("equip pick sub line", &subs, ui::LIST_SUB_K, room);
}

/// 가방 탭: 수 · 여벌 정리 단추 · 칸의 일 목록.
fn bag_tab(c: &mut Check) {
    let s = s();
    let grid = bag::COLS as f32 * bag::CELL + (bag::COLS - 1) as f32 * bag::GAP;
    let count = fill(s.bag_count, &["30", "30"]);
    c.fit(
        "bag count",
        &count,
        tabs::K_KIND * 1.15,
        grid - bag::TIDY_W - 4.0,
    );
    c.fit(
        "bag tidy button",
        s.bag_tidy,
        tabs::K_KIND,
        bag::TIDY_W - 6.0,
    );
    let acts = [
        s.bag_do_main.to_string(),
        s.bag_do_sub.to_string(),
        s.bag_do_wear.to_string(),
        s.bag_do_keep.to_string(),
        s.bag_do_unkeep.to_string(),
        s.bag_do_drop.to_string(),
        wide(s.bag_do_salvage),
        s.bag_do_away.to_string(),
        s.bag_do_don.to_string(),
        s.bag_do_doff.to_string(),
    ];
    c.all(
        "bag act row",
        acts,
        1.0,
        480.0 - 12.0 - 24.0 - ui::GLYPH * 1.1,
    );
}

/// 성장(상태) 탭: 이름·총 레벨 · 막대의 글 · 묶음 머리 · 레벨 줄 · 기술 줄.
fn status_tab(c: &mut Check) {
    use status::*;
    let s = s();
    c.fit("status name", s.status_name, 1.0, DOLL_W);
    c.fit("status level", &wide(s.level_title), tabs::K_KIND, DOLL_W);
    c.fit("status bar", &wide(s.status_hp), tabs::K_KIND, BAR_W);
    c.fit("status bar", &wide(s.status_stamina), tabs::K_KIND, BAR_W);
    let kk = tabs::K_KIND;
    c.fit("status group head", s.status_levels, kk, PICK_W);
    for w in weapon_names() {
        c.fit(
            "status group head",
            &format!("{} — {w}", s.status_skills),
            kk,
            PICK_W,
        );
    }
    let (x, right) = (PICK_X + 6.0, PICK_X + PICK_W - 6.0);
    let pips = PIP * crate::progression::MAX_RANK as f32
        + PIP_GAP * (crate::progression::MAX_RANK - 1) as f32;
    let cost_w = [fill(s.shop_embers, &["999"]), s.lv_full_d.to_string()]
        .iter()
        .map(|t| c.meter.width(t, kk))
        .fold(0.0, f32::max);
    for &l in &crate::progression::LINES {
        c.fit(
            "status level row",
            line_name(l),
            1.0,
            right - pips - 6.0 - x,
        );
        c.fit(
            "status level sub",
            line_sub(l),
            kk,
            right - x - cost_w - 4.0,
        );
    }
    // 기술 줄: 그림 · 이름 · 버튼 키캡(큰 기술은 두 버튼) · →.
    let tag = ui::key(&input::ult_prompt());
    let tag_w = c.meter.width(&tag, kk) + 5.0;
    let arrow = c.meter.width("→", kk);
    let room = right - (x + CHARM_H) - arrow - tag_w - 4.0;
    let names: Vec<&str> = ENTRIES
        .iter()
        .filter(|e| matches!(e, Entry::Skill(_) | Entry::Ult(_)))
        .map(|&e| skills::entry_text(e).0)
        .chain([s.charm_empty, s.status_passives])
        .collect();
    c.all("status skill row", names, 0.85, room);
}

/// 왼쪽 줄 목록: 가게 · 여정 · 도감 · 설정.
fn left_lists(c: &mut Check) {
    let s = s();
    // 가게: 물건 … 값, 벼리기 "검 +3 벼리기" … "숫돌 9 · 잔불 999", 거두기.
    let mut shop: Vec<ui::Row> = crate::growth::SHOP
        .iter()
        .map(|&(w, _)| ui::Row::value(shop::ware_name(w), fill(s.shop_item, &["9999"])))
        .collect();
    shop.extend(
        crate::growth::SHOP
            .iter()
            .map(|&(w, _)| ui::Row::value(shop::ware_name(w), s.shop_sold)),
    );
    for w in crate::weapons::IDS {
        // 벼리기 줄의 이름은 다음 단(+1~+3), 값은 표의 가장 큰 값 · 거두기는 3단에서 돌려받는 숫돌.
        let (st, em) = crate::forge::COST
            .iter()
            .fold((0, 0), |a, &(x, y)| (a.0.max(x), a.1.max(y)));
        shop.push(ui::Row::value(
            fill(s.forge_row, &[&forged_label(w, 3)]),
            fill(s.forge_cost, &[&st.to_string(), &em.to_string()]),
        ));
        shop.push(ui::Row::value(forged_label(w, 3), s.forge_full));
        shop.push(ui::Row::value(
            fill(s.forge_recall_row, &[&forged_label(w, 3)]),
            fill(s.forge_back, &[&crate::forge::cost_to(3).0.to_string()]),
        ));
    }
    rows(c, "shop row", LIST_W, &shop);
    // 여정: 목표 … "12/12"(작은 표지).
    let journey: Vec<ui::Row> = crate::journey::GOALS
        .iter()
        .map(|g| tagged(ui::Row::value(tr(g.title), "99/99")))
        .collect();
    rows(c, "journey row", LIST_W, &journey);
    // 도감: 이름 … ◆.
    let codex: Vec<ui::Row> = crate::codex::ENTRIES
        .iter()
        .map(|e| tr(crate::species::def(e.kind).name))
        .chain(charm_names())
        .chain(garb_names())
        .chain(weapon_names())
        .map(|n| tagged(ui::Row::value(n, "◆")))
        .collect();
    rows(c, "codex row", LIST_W, &codex);
    // 도감의 묶음 머리(넘치면 굴린다): 이름 하나는 보이는 창 안에.
    let tk = 0.85;
    let arrow = c.meter.width("◀", tk) + 6.0;
    let view = tabs::LEFT_W - 6.0 - 4.0 - 2.0 * arrow;
    for n in [
        s.menu_journey,
        s.codex_tab_foes,
        s.codex_tab_charms,
        s.codex_tab_garb,
        s.codex_tab_weapons,
    ] {
        c.fit("codex group", n, tk, view);
    }
    // 설정: 이름 ◀ 값 ▶ (값은 가장 넓은 것으로).
    let pct = "100%".to_string();
    let on = [s.value_on, s.value_off]
        .iter()
        .max_by(|a, b| c.meter.width(a, 1.0).total_cmp(&c.meter.width(b, 1.0)))
        .map(|t| t.to_string())
        .unwrap_or_default();
    let reticle = [s.reticle_small, s.reticle_medium, s.reticle_large]
        .iter()
        .max_by(|a, b| c.meter.width(a, 1.0).total_cmp(&c.meter.width(b, 1.0)))
        .map(|t| t.to_string())
        .unwrap_or_default();
    let langs = Lang::ALL
        .iter()
        .map(|l| l.strings().lang_name)
        .max_by(|a, b| c.meter.width(a, 1.0).total_cmp(&c.meter.width(b, 1.0)))
        .unwrap_or_default();
    let settings = vec![
        ui::Row::step(s.menu_language, langs),
        ui::Row::step(s.menu_fullscreen, on.clone()),
        ui::Row::step(s.menu_shake, pct.clone()),
        ui::Row::step(s.menu_volume, pct.clone()),
        ui::Row::step(s.menu_tells, pct.clone()),
        ui::Row::step(s.menu_tell_timing, on.clone()),
        ui::Row::step(s.menu_look_yaw, pct.clone()),
        ui::Row::step(s.menu_look_pitch, pct.clone()),
        ui::Row::step(s.menu_touch_size, pct),
        ui::Row::step(s.menu_mouse_aim, on),
        ui::Row::step(s.menu_cursor_rest, fill(s.value_secs, &["2.5"])),
        ui::Row::step(s.menu_reticle, reticle),
        ui::Row::go(s.menu_controls),
        ui::Row::plain(s.menu_restart),
        ui::Row::plain(s.menu_to_title),
    ];
    rows(c, "settings row", LIST_W, &settings);
    // 설정의 오른쪽 머리: 이름 · 지금 값.
    let names: Vec<String> = settings.iter().map(|r| r.label.clone()).collect();
    c.all("settings pane name", &names, tabs::K_NAME, tabs::RIGHT_W);
}

/// 한 언어의 모든 칸을 잰다.
fn check_lang(meter: &Meter, lang: Lang, src: &str) -> (usize, Vec<String>) {
    crate::i18n::set_lang(lang);
    let mut c = Check {
        meter,
        lang,
        keys: table(src),
        fails: Vec::new(),
        count: 0,
    };
    window_frame(&mut c);
    pane_heads(&mut c);
    pane_lines(&mut c);
    wrapped(&mut c);
    skills_tab(&mut c);
    equip_tab(&mut c);
    bag_tab(&mut c);
    status_tab(&mut c);
    left_lists(&mut c);
    (c.count, c.fails)
}

/// 0275: 큰 창의 한 줄짜리 고정 칸에 서는 글은 모두 그 칸에 든다(한국어·영어) — 든다면 `Ui::fit`이 자를 일이 없다.
/// 넘치면 칸·글을 고친다(줄 바꿈·굴리기로 옮기거나 두 언어의 글을 줄인다) — 시험을 느슨하게 하지 않는다.
#[test]
fn big_window_text_fits_its_slots() {
    let meter = Meter::new();
    let mut fails = Vec::new();
    let mut total = 0;
    for (lang, src) in [(Lang::Ko, KO_SRC), (Lang::En, EN_SRC)] {
        let (n, f) = check_lang(&meter, lang, src);
        assert!(n > 1000, "{lang:?}: only {n} strings measured");
        total += n;
        fails.extend(f);
    }
    assert!(
        fails.is_empty(),
        "{} of {total} texts overflow their slot (width > room, base px at UI scale 1):\n{}",
        fails.len(),
        fails.join("\n")
    );
}

/// 표를 읽는 셈이 표 전부를 읽는다(키 수 = `Strings`의 필드 수, 글은 실제 표의 글).
#[test]
fn the_source_tables_parse_to_every_key() {
    for (src, strings) in [(KO_SRC, &crate::i18n::KO), (EN_SRC, &crate::i18n::EN)] {
        let t = table(src);
        let all: Vec<&str> = crate::i18n::all(strings);
        assert!(t.len() >= all.len(), "{} keys < {}", t.len(), all.len());
        for (k, v) in &t {
            assert!(
                all.contains(&v.as_str()),
                "{k}: {v:?} is not a table string"
            );
        }
    }
}

/// 자가 macroquad의 셈과 같다: macroquad는 fontdue로 글자를 굽고 그 advance를 더해 늘린다(`measure_text` — 커닝 없음).
/// 같은 글꼴을 fontdue로 읽어 굽는 크기(48px)의 advance 합을 늘린 값과 이 자의 값을 견준다.
#[test]
fn the_meter_matches_what_macroquad_measures() {
    let meter = Meter::new();
    let font = fontdue::Font::from_bytes(FONT, fontdue::FontSettings::default()).unwrap();
    let bake = 48.0;
    for (text, k) in [
        ("순례자의 검", 1.0),
        ("Pilgrim's Sword", 1.0),
        ("Crossbow · Hammer", 0.58),
        ("되돌리기 · 잔불 55", 0.66),
    ] {
        let px = ui::GLYPH * k;
        let fd: f32 = text
            .chars()
            .map(|c| font.metrics(c, bake).advance_width)
            .sum::<f32>()
            * px
            / bake;
        let ours = meter.width(text, k);
        assert!((fd - ours).abs() < 0.01, "{text}: fontdue {fd} vs {ours}");
    }
    // 키캡은 칸 여백까지(`ui::width_with`).
    let plain = meter.width("ESC", 1.0);
    assert!(meter.width(&ui::key("ESC"), 1.0) > plain + ui::GLYPH * 0.5);
}
