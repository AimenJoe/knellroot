//! 개발 화면 글(0080, 사장 2026-11-07 "다 영어네..?"): 연구소(랩)·DEV 창. 0013은 개발 화면을 영어로 두었으나 사장이 한국어로 읽기를 원한다.
//! 한국어일 때 이 표, 영어일 때는 원래 글(랩 값은 `tuning.rs`의 이름 그대로 — 내보낸 줄과 같은 말).
//! 이 파일도 `src/i18n/` 안이라 글꼴 서브셋(`tools/fonts/subset_ui.py`)이 글자를 넣고, 글꼴 시험이 본다.
use super::{lang, Lang};

/// 한국어면 `ko`, 아니면 `en`.
pub fn d(ko: &'static str, en: &'static str) -> &'static str {
    match lang() {
        Lang::Ko => ko,
        Lang::En => en,
    }
}

/// 개발 화면 글 한 쌍(한국어, 영어). 숫자·이름은 `{0}`… + `super::fill`.
pub type Pair = (&'static str, &'static str);

pub const NO_B: Pair = ("A (B 없음)", "A (no B)");
pub const USING: Pair = ("쓰는 값", "using");
pub const STORE_B: Pair = ("지금 값을 B에 담기", "store current as B");
pub const EXPORT: Pair = ("콘솔로 내보내기", "export to console");
pub const RESET_ALL: Pair = ("모두 기본값으로", "reset all to defaults");
pub const NOTE_RESET: Pair = ("모든 값을 기본값으로 되돌림", "all values back to defaults");
pub const NOTE_STORED: Pair = ("지금 값을 B에 담음", "stored current values as slot B");
pub const NOTE_NO_B: Pair = (
    "아직 B가 없음(먼저 담기)",
    "no slot B yet (store one first)",
);
pub const NOTE_EXPORTED: Pair = (
    "콘솔로 내보냄(src/tuning.rs에 붙여 넣기)",
    "exported to console (paste into src/tuning.rs)",
);
pub const NOTE_USING: Pair = ("지금 {0} 값을 씀", "now using {0}");
pub const USING_D: Pair = (
    "A와 B 두 벌의 값을 오가며 견준다. 좌우나 확인으로 바꾼다. B는 먼저 담아야 생긴다.",
    "switch between two sets of values, A and B, to compare them. left/right or confirm. store B first.",
);
pub const STORE_B_D: Pair = (
    "지금 쓰는 값을 모두 B 벌에 담는다. 그 뒤 A로 돌아가 바꿔 보고 B와 견준다.",
    "copy every current value into set B. then go back to A, change things, and compare.",
);
pub const EXPORT_D: Pair = (
    "기본값과 다른 값을 src/tuning.rs에 붙여 넣을 줄로 콘솔에 찍는다.",
    "print every value that differs from its default to the console, as lines to paste into src/tuning.rs.",
);
pub const RESET_ALL_D: Pair = (
    "모든 묶음의 값을 기본값으로 되돌린다. B 벌은 그대로 남는다.",
    "put every value in every group back to its default. set B stays.",
);
pub const LAB_NOW: Pair = ("지금", "now");
pub const LAB_DEF: Pair = ("기본", "default");
pub const LAB_STEP: Pair = ("한 걸음", "step");
pub const LAB_CHANGED: Pair = ("기본값과 다름", "changed from default");
pub const LAB_FAST: Pair = ("꾹 + 좌우 ×10", "hold + left/right x10");
pub const LAB_RESET: Pair = ("기본값으로", "reset");
pub const LAB_COUNT: Pair = ("값 {0}개 · 바꾼 것 {1}", "{0} values · {1} changed");
pub const LAB_ALL_DEFAULT: Pair = ("모두 기본값 그대로다.", "everything is at its default.");
pub const LAB_GROUP_D: Pair = (
    "확인 = 이 묶음의 탭으로.",
    "confirm = go to this group's tab.",
);
pub const DEV_TITLE: Pair = ("개발", "DEV");
pub const DEV_ROW_D: Pair = (
    "개발 창을 연다 — 스테이지·시나리오·도구를 고른다.",
    "open the dev window: stages, scenarios, tools",
);
pub const TAB_TOOLS: Pair = ("도구", "tools");
pub const TAB_STAGE: Pair = ("스테이지", "stage");
pub const TAB_COMBAT: Pair = ("전투·적", "combat");
pub const TAB_PERF: Pair = ("성능", "perf");
pub const TAB_MOTION: Pair = ("모션", "motion");
pub const TAB_SKILLS: Pair = ("기술", "skills");
pub const MOTION_START: Pair = ("모션 보기 — {0}", "motions: {0}");
pub const MOTION_NOTE: Pair = (
    "이 무기를 든 방랑자의 동작을 하나씩 인형으로 돌려 본다 — 평타·강타·패링·회피·맞음·기술. 판정이 있는 동작은 시간 막대에 판정 구간이 호박색으로 보인다.",
    "plays each motion of the wanderer holding this weapon on the doll: hits, heavy, parry, dodge, hurt, skills. the hit window shows in amber on the time bar.",
);
pub const MOTION_TITLE: Pair = ("모션 — {0}", "motions: {0}");
pub const MOTION_HINT: Pair = (
    "위아래: 동작 · 좌우: 돌리기 · 확인: 빠르기 · 탭 넘기기: 무기 · 뒤로: 나가기",
    "up/down: motion · left/right: turn · confirm: speed · tabs: weapon · back: leave",
);
pub const MOTION_LEN: Pair = (
    "길이 {0}초 · 지금 {1}초 · 빠르기 {2}",
    "length {0}s · now {1}s · speed {2}",
);
pub const MOTION_HIT: Pair = ("판정 {0}~{1}초", "hit window {0}–{1}s");
pub const MOTION_NONE: Pair = (
    "모델에 이 동작이 없다(서기로 보인다)",
    "the model has no such motion (shows idle)",
);
pub const MOTION_PAUSED: Pair = ("멈춤", "paused");
/// 달리기 공격 줄(0202 — 콤보의 타와 같은 동작을 빌려도 줄을 따로 둔다): {0} = 빌린 동작의 이름.
pub const MOTION_DASH: Pair = ("달리기 공격 — {0}", "dash attack: {0}");
pub const DEV_HINT: Pair = (
    "좌우: 묶음 · 위아래: 고르기 · 확인: 시작 · 뒤로: 나가기",
    "left/right: group · up/down: pick · confirm: start · back: leave",
);
pub const ENTRANCE: Pair = ("첫 스테이지 입구부터", "stage 1 from the entrance");
pub const GATE: Pair = (
    "첫 스테이지 성소 문 앞(보스)",
    "stage 1 at the sanctum gate (boss)",
);
pub const VILLAGE: Pair = (
    "둘째 지역(떠난 마을) 어귀부터",
    "region 2 (village) from its edge",
);
pub const GARDEN: Pair = (
    "셋째 지역(잔뿌리의 정원) 어귀부터",
    "region 3 (rootlet garden) from its edge",
);
pub const WORKS: Pair = (
    "넷째 지역(작업층) 어귀부터",
    "region 4 (the workings) from its edge",
);
pub const PROLOGUE: Pair = (
    "서막(숲에서 깨어나 성소에 들기)부터",
    "prologue (wake in the wood, enter the sanctuary)",
);
pub const RESIDENTS: Pair = (
    "둘째 지역 주민 곁에서(말 걸기)",
    "region 2 beside a resident (talk)",
);
pub const STONES: Pair = (
    "둘째 지역 새긴 돌 곁에서(읽기)",
    "region 2 beside a carved stone (read)",
);
pub const CHEST: Pair = (
    "둘째 지역 상자 곁에서(떨어진 아이템)",
    "region 2 beside a chest (dropped loot)",
);
pub const WEAPON_CHEST: Pair = ("{1}의 상자 곁에서({0})", "beside the chest of {1} ({0})");
pub const WALL_CHEST: Pair = (
    "첫 지역 성벽길의 상자 곁에서(숫돌)",
    "region 1 beside the chest on the wall walk (a whetstone)",
);
pub const SPARE_CHEST: Pair = (
    "첫 지역 뜰 남동쪽 단의 상자 곁에서(여벌)",
    "region 1 beside the chest on the courtyard's southeast dais (a spare)",
);
pub const FIELD: Pair = (
    "첫 지역 필드 보스의 터에서(고요한 사슴)",
    "region 1 at the field boss's clearing (the Hushed Stag)",
);
pub const STAGE_NOTE: Pair = (
    "이 슬롯의 장신구·잔불로 진짜 스테이지를 돈다. 아무것도 저장하지 않는다.",
    "the real stage with this slot's charms and embers; nothing is saved",
);
pub const LAB_START: Pair = ("연구소(튜닝)", "lab (tuning)");
pub const LAB_NOTE: Pair = (
    "값을 바로 바꿔 본다. 메뉴의 설정 아래 연구소와 같은 창.",
    "change tuning live; same as the lab row under settings",
);
pub const SKILLS_START: Pair = ("기술 끼우기", "skill slots");
pub const SKILLS_NOTE: Pair = (
    "아레나에서 쓸 기술 둘과 큰 기술을 끼운다. 멈춤의 기술 화면과 같다(칸 → 확인 → 끼우기).",
    "wear the two skills and the great skill for the arenas; the pause menu's skills screen (cell, confirm, wear)",
);
pub const SKILLS_MAX: Pair = ("기술 다 올리기(켬/끔)", "max skills (on/off)");
pub const SKILLS_MAX_NOTE: Pair = (
    "모든 기술과 큰 기술을 끝 단으로, 패시브 칸을 모두 열림으로. 다시 고르면 이 슬롯의 판으로 돌아간다. 저장하지 않는다. 지금: {0}",
    "every skill and great skill at max rank, every passive open; pick again for this slot's board. nothing is saved. now: {0}",
);
pub const SKILLS_FLIP: Pair = ("고르기 바꾸기(ㄱ·ㄴ)", "flip all picks (A/B)");
pub const ARM: Pair = ("든 무기 바꾸기", "change the held weapon");
pub const ARM_NOTE: Pair = (
    "지금: {0}. 든 무기를 다음 것으로 바꾼다({1}). 스킬은 무기 종류를 탄다(맨손은 무기 스킬을 못 쓴다) — 여기서 든 무기로 끼우고 아레나에 선다. 저장하지 않는다.",
    "now: {0}. cycle the held weapon ({1}). Skills go by weapon class (bare hands wield none): wear and enter arenas with this one. nothing is saved.",
);
pub const ARM_SUB: Pair = ("보조무기 바꾸기", "change the off-hand weapon");
pub const ARM_SUB_NOTE: Pair = (
    "지금: {0}. 아레나의 보조무기를 다음 것으로 바꾼다({1}). 든 무기와 같으면 비워진다. 보조특성과 교체 공격(맞힌 직후·패링 직후의 무기 바꾸기)을 본다. 저장하지 않는다.",
    "now: {0}. cycle the arena's off-hand weapon ({1}); the same as the held weapon means none. For off-hand traits and the swap attack (swap right after a hit or a parry). nothing is saved.",
);
pub const ARM_BARE: Pair = ("보조무기 비우기(켬/끔)", "empty the off hand (on/off)");
pub const ARM_BARE_NOTE: Pair = (
    "아레나의 보조무기 슬롯을 비운다(보조무기 · 방패를 내린다) — 든 무기 종류의 빈 손의 결로 선다. 다시 고르면 되돌린다. 저장하지 않는다. 지금: {0}",
    "empty the arena's off-hand slot (no off-hand weapon, no shield) — the held weapon's empty-hand way. pick again to undo. nothing is saved. now: {0}",
);
pub const ARM_BARE_ON: Pair = ("빈 손", "empty");
pub const ARM_BARE_OFF: Pair = ("그대로", "as set");
pub const PERF_LINE: Pair = ("성능 줄 켬/끔", "perf line on/off");
pub const PERF_LINE_NOTE: Pair = (
    "화면 아래에 fps·cpu·프레임 시간·정점 수를 띄운다(F3과 같다). 연구소의 render.resident를 0·1로 바꿔 지형 상주를 견준다.",
    "show fps, cpu, frame time and vertices at the bottom (same as F3). Flip render.resident 0/1 in the lab to compare.",
);
pub const SKILLS_FLIP_NOTE: Pair = (
    "끝 단으로 올린 판의 고르기를 모두 ㄱ·ㄴ으로 뒤집는다(꺼져 있으면 켠다). 저장하지 않는다. 지금: {0}",
    "flips every pick on the maxed board (turns it on if off). nothing is saved. now: {0}",
);
pub const BOARD_SLOT: Pair = ("슬롯의 판", "the slot's board");
pub const BOARD_MAX_A: Pair = ("끝 단 · 고르기 ㄱ", "max rank · pick A");
pub const BOARD_MAX_B: Pair = ("끝 단 · 고르기 ㄴ", "max rank · pick B");
pub const SCENARIO_NOTE: Pair = ("아레나에서 숫자 키 {0} · {1}", "key {0} in the arena · {1}");

/// 위의 쌍 전부(글꼴 시험·글자 미리 굽기 `i18n::dev_texts`).
pub const PAIRS: &[Pair] = &[
    NO_B,
    USING,
    STORE_B,
    EXPORT,
    RESET_ALL,
    NOTE_RESET,
    NOTE_STORED,
    NOTE_NO_B,
    NOTE_EXPORTED,
    NOTE_USING,
    USING_D,
    STORE_B_D,
    EXPORT_D,
    RESET_ALL_D,
    LAB_NOW,
    LAB_DEF,
    LAB_STEP,
    LAB_CHANGED,
    LAB_FAST,
    LAB_RESET,
    LAB_COUNT,
    LAB_ALL_DEFAULT,
    LAB_GROUP_D,
    DEV_TITLE,
    DEV_ROW_D,
    TAB_TOOLS,
    TAB_STAGE,
    TAB_COMBAT,
    TAB_PERF,
    TAB_MOTION,
    TAB_SKILLS,
    MOTION_START,
    MOTION_NOTE,
    MOTION_TITLE,
    MOTION_HINT,
    MOTION_LEN,
    MOTION_HIT,
    MOTION_NONE,
    MOTION_PAUSED,
    MOTION_DASH,
    DEV_HINT,
    ENTRANCE,
    GATE,
    VILLAGE,
    GARDEN,
    WORKS,
    PROLOGUE,
    RESIDENTS,
    STONES,
    CHEST,
    WEAPON_CHEST,
    WALL_CHEST,
    SPARE_CHEST,
    FIELD,
    STAGE_NOTE,
    LAB_START,
    LAB_NOTE,
    SKILLS_START,
    SKILLS_NOTE,
    SKILLS_MAX,
    SKILLS_MAX_NOTE,
    SKILLS_FLIP,
    SKILLS_FLIP_NOTE,
    ARM,
    ARM_NOTE,
    ARM_SUB,
    ARM_SUB_NOTE,
    ARM_BARE,
    ARM_BARE_NOTE,
    ARM_BARE_ON,
    ARM_BARE_OFF,
    PERF_LINE,
    PERF_LINE_NOTE,
    BOARD_SLOT,
    BOARD_MAX_A,
    BOARD_MAX_B,
    SCENARIO_NOTE,
];

/// 모션 보기의 동작 이름(모델의 동작 이름 → 화면 이름). 표에 없으면 동작 이름 그대로.
pub const MOTION_LABELS: &[(&str, Pair)] = &[
    ("idle", ("서기", "idle")),
    ("idle_look", ("몸짓 — 둘러보기", "fidget: look")),
    ("idle_shake", ("몸짓 — 털기", "fidget: shake")),
    ("walk", ("걷기", "walk")),
    ("run", ("달리기", "run")),
    ("walk_back", ("뒷걸음(락온)", "walk back (lock-on)")),
    ("turn_l", ("땅 돌리기 왼쪽", "turn left")),
    ("turn_r", ("땅 돌리기 오른쪽", "turn right")),
    ("attack", ("평타 1", "hit 1")),
    ("attack2", ("평타 2", "hit 2")),
    ("attack3", ("평타 3", "hit 3")),
    (
        "sword_dash",
        ("검 — 달리며 올려 베기", "sword: running rising slash"),
    ),
    ("charge", ("강타 모으기", "heavy charge")),
    ("heavy", ("강타", "heavy")),
    ("hammer1", ("평타 1", "hit 1")),
    ("hammer2", ("평타 2", "hit 2")),
    ("hammer3", ("평타 3", "hit 3")),
    ("hammer_charge", ("강타 모으기", "heavy charge")),
    ("hammer_slam", ("강타", "heavy")),
    ("staff1", ("평타 1", "hit 1")),
    ("staff2", ("평타 2", "hit 2")),
    ("staff3", ("평타 3", "hit 3")),
    ("staff4", ("평타 4", "hit 4")),
    ("staff_charge", ("강타 모으기", "heavy charge")),
    ("staff_release", ("강타", "heavy")),
    ("cast_charge", ("마법 모으기", "magic charge")),
    ("cast", ("마법", "magic")),
    ("xbow_charge", ("쇠뇌 모으기", "crossbow charge")),
    ("xbow_shoot", ("쇠뇌 쏘기", "crossbow shot")),
    ("bow_draw", ("활 당기기", "bow draw")),
    ("bow_shoot", ("활 쏘기", "bow shot")),
    ("throw", ("수류탄 던지기", "grenade")),
    ("parry", ("패링", "parry")),
    ("dodge", ("회피", "dodge")),
    ("jump", ("점프", "jump")),
    ("pickup", ("줍기·상자 열기", "pick up / open")),
    ("pull", ("장치 당기기", "pull lever")),
    ("fist1", ("주먹 — 왼 잽", "fists: left jab")),
    ("fist2", ("주먹 — 오른 잽", "fists: right jab")),
    ("fist3", ("주먹 — 어퍼컷", "fists: uppercut")),
    ("fist_charge", ("주먹 — 차기 모으기", "fists: kick charge")),
    ("fist_heavy", ("주먹 — 앞차기", "fists: front kick")),
    ("fist_dash", ("주먹 — 달리며 앞차기", "fists: running kick")),
    (
        "hammer_dash",
        ("망치 — 달리며 앞차기", "hammer: running kick"),
    ),
    ("parry_push", ("패링 성공 — 밀쳐내기", "parry: shove")),
    ("dodge_back", ("뒤로 뛰기(회피)", "back hop (dodge)")),
    ("land", ("내려서기", "land")),
    ("plunge", ("내려찍기", "plunge")),
    ("plunge_land", ("내려찍고 일어나기", "plunge landing")),
    ("drink", ("등잔 마시기", "drink")),
    ("hurt", ("맞음", "hurt")),
    ("fall", ("쓰러짐", "fall")),
    ("wake", ("깨어남", "wake")),
    ("sk_whirl", ("기술 — 휘돌이", "skill: whirl")),
    ("sk_cut", ("기술 — 가로지르기", "skill: cut")),
    ("sk_gather", ("기술 — 모아 끌기", "skill: gather")),
    ("sk_quake", ("기술 — 땅울림", "skill: quake")),
    ("sk_brace", ("기술 — 버티기", "skill: brace")),
    ("sk_echo", ("기술 — 되울림", "skill: echo")),
    ("sk_throw", ("기술 — 던지기", "skill: throw")),
    ("sk_embers", ("기술 — 잔불 흩기", "skill: embers")),
    ("ult_upend", ("큰 기술 — 쟁반 뒤집기", "ult: upend")),
    ("ult_flurry", ("큰 기술 — 몰아치기", "ult: flurry")),
    ("ult_line", ("큰 기술 — 한 줄기", "ult: line")),
    // 0270 C3
    ("sk_stitch", ("기술 — 바늘땀", "skill: needle stitch")),
    ("sk_ram", ("기술 — 방패 돌진", "skill: shield rush")),
    ("sk_toll", ("기술 — 종 치기", "skill: toll the bell")),
    ("sk_swat", ("기술 — 후려치기", "skill: swat")),
    ("sk_ripple", ("기술 — 파문", "skill: ripple")),
    ("sk_snow", ("기술 — 눈발", "skill: snowfall")),
    ("sk_flare", ("기술 — 불길", "skill: flare")),
    ("sk_snare", ("기술 — 뿌리 덫", "skill: root snare")),
    ("ult_bell", ("큰 기술 — 큰 종", "ult: great bell")),
    ("ult_blizzard", ("큰 기술 — 눈보라", "ult: blizzard")),
    // 0272
    ("sk_fuse", ("기술 — 심지 볼트", "skill: fuse bolt")),
    ("sk_mine", ("기술 — 화약 덫", "skill: powder trap")),
    ("ult_rain", ("큰 기술 — 화살비", "ult: arrow rain")),
    // 0272 2단계 — 창
    ("spear1", ("창 — 1타 찌르기", "spear: thrust 1")),
    ("spear2", ("창 — 2타 찌르기", "spear: thrust 2")),
    (
        "spear3",
        ("창 — 3타 내디뎌 찌르기", "spear: lunging thrust 3"),
    ),
    (
        "spear_dash",
        ("창 — 달리며 찌르기", "spear: running thrust"),
    ),
    ("spear_charge", ("창 — 모으기", "spear: charge")),
    (
        "spear_lunge",
        ("창 — 내달려 꿰뚫기", "spear: piercing lunge"),
    ),
    ("sk_skewer", ("기술 — 꿰뚫기", "skill: skewer")),
    ("sk_pole", ("기술 — 장대 휩쓸기", "skill: pole sweep")),
    ("sk_pin", ("기술 — 못 박기", "skill: pinning thrust")),
    (
        "ult_gale",
        ("큰 기술 — 질풍 찌르기", "ult: gale of thrusts"),
    ),
    // 0274 — 한손검의 전투 기술
    ("sk_pommel", ("기술 — 자루 치기", "skill: pommel strike")),
    ("sk_shove", ("기술 — 밀치기", "skill: shove")),
    ("sk_rally", ("기술 — 재기", "skill: rally")),
    // 0274 ③ — 쌍단검
    (
        "dagger1",
        ("쌍단검 — 1타 왼손 찌르기", "daggers: left stab 1"),
    ),
    (
        "dagger2",
        ("쌍단검 — 2타 오른손 찌르기", "daggers: right stab 2"),
    ),
    ("dagger3", ("쌍단검 — 3타 가로 베기", "daggers: slash 3")),
    (
        "dagger4",
        ("쌍단검 — 4타 올려 베기", "daggers: rising cut 4"),
    ),
    (
        "dagger_dash",
        ("쌍단검 — 달려들며 찌르기", "daggers: running stab"),
    ),
    ("dagger_charge", ("쌍단검 — 모으기", "daggers: charge")),
    (
        "dagger_lunge",
        ("쌍단검 — 꿰뚫고 지나가기", "daggers: lunge through"),
    ),
    ("sk_slip", ("기술 — 빠지며 베기", "skill: slip away")),
    (
        "ult_ambush",
        ("큰 기술 — 그림자 습격", "ult: shadow ambush"),
    ),
    // 0274 ④ — 대검
    (
        "gs1",
        ("대검 — 1타 오른쪽에서 휩쓸기", "greatsword: sweep 1"),
    ),
    (
        "gs2",
        ("대검 — 2타 되받아 휩쓸기", "greatsword: back sweep 2"),
    ),
    ("gs3", ("대검 — 3타 다시 휩쓸기", "greatsword: sweep 3")),
    (
        "gs4",
        ("대검 — 4타 크게 휩쓸기", "greatsword: wide sweep 4"),
    ),
    ("gs5", ("대검 — 5타 내려찍기", "greatsword: overhead 5")),
    (
        "gs_dash",
        ("대검 — 달려들며 올려 베기", "greatsword: running cut"),
    ),
    ("gs_charge", ("대검 — 모으기", "greatsword: charge")),
    (
        "gs_heavy",
        ("대검 — 두 손 내려베기", "greatsword: great cleave"),
    ),
    ("sk_sunder", ("기술 — 땅 가르기", "skill: sunder")),
    ("sk_sear", ("기술 — 불씨 휩쓸기", "skill: cinder sweep")),
    ("ult_tempest", ("큰 기술 — 칼바람", "ult: bladestorm")),
    // 0274 ⑤ — 낫
    (
        "scythe3",
        ("낫 — 3타 걸어 당기며 베기", "sickle: hook and drag 3"),
    ),
    ("sk_reap", ("기술 — 걸어 당기기", "skill: hook and drag")),
    ("sk_harvest", ("기술 — 생기 거두기", "skill: reap life")),
    ("ult_grove", ("큰 기술 — 깨어나는 숲", "ult: waking grove")),
    // 0274 ⑥ — 완드
    ("sk_hex", ("기술 — 저주 새기기", "skill: etch a curse")),
    (
        "sk_blight",
        ("기술 — 시드는 그늘", "skill: withering shade"),
    ),
    ("ult_price", ("큰 기술 — 치른 값", "ult: the price paid")),
];

/// 동작 이름의 화면 이름(표에 없으면 동작 이름 그대로 — 모델의 동작 이름은 오른쪽 칸에 따로 보인다).
pub fn motion_label(anim: &str) -> String {
    match MOTION_LABELS.iter().find(|(a, _)| *a == anim) {
        Some((_, p)) => t(*p).to_string(),
        None => anim.to_string(),
    }
}

/// 쌍에서 지금 언어의 것.
pub fn t(p: Pair) -> &'static str {
    d(p.0, p.1)
}

/// 랩 값 이름(`SPECS[i].name`)의 화면 이름. 한국어면 표의 것(없으면 원래 이름).
pub fn spec_label(name: &'static str) -> &'static str {
    match lang() {
        Lang::Ko => SPEC_KO
            .iter()
            .find(|(k, _)| *k == name)
            .map_or(name, |(_, v)| v),
        Lang::En => name,
    }
}

/// 개발 시나리오(`scenario::LIST`의 영어 이름) → (한국어 이름, 무엇을 보나). F3 아레나 줄은 기본 글꼴이라 영어 그대로.
pub const SCENARIO_KO: &[(&str, &str, &str)] = &[
    (
        "hushed stag",
        "고요한 사슴",
        "0270 D 터에 엎드려 기다린다(다가가거나 쏘면 일어난다), 뿔 쓸기, 돌진과 긴 틈, 뒷발로 일어섰다 내려 딛기: 원판 + 뿌리 줄",
    ),
    (
        "hushed stag phase two",
        "고요한 사슴 2단계",
        "0270 D 시작하자마자 전환, 쉬는 틈이 짧다, 돌아서 한 번 더 돌진, 뿌리 줄 셋이 부채꼴로",
    ),
    (
        "a line of husks",
        "한 줄로 선 잔재",
        "0272 창: 찌르기가 줄을 꿰뚫는다, 옆이 빈다, 내달려 꿰뚫기가 뚫고 지나간다",
    ),
    (
        "chained winchkeeper",
        "사슬에 묶인 권양지기",
        "0208 사슬 안에서 내려치기·사슬 휘두르기, 떨어지면 갈고리·추, 넷에 하나 아래에서 사슬이 끊긴다",
    ),
    (
        "moss-clad warden",
        "이끼 덮인 문지기",
        "0185 낮게 쓸기·껍질 받기(앞 평타는 튕김, 옆·뒤·강타는 든다)·틈새 돌진(뒤에 늦게 돈다)",
    ),
    (
        "moss-clad warden phase two",
        "이끼 덮인 문지기 2단계",
        "0185 시작하자마자 전환·짧은 쉼·쓸기 뒤 돌진",
    ),
    (
        "warden phase two",
        "문지기 2단계",
        "0133 시작하자마자 전환(맞지 않음·밀쳐냄)·뿌리 기둥 줄·돌진 뒤 내려치기",
    ),
    (
        "bellkeeper phase two",
        "종지기 2단계",
        "0133 시작하자마자 전환(맞지 않음·밀쳐냄)·선 자리에 종 내던지기 + 고리·돌진 뒤 울림",
    ),
    (
        "broken bellkeeper",
        "깨진 종지기",
        "0125 울림 고리(뛰거나 구르며 지나기)·종 조각 부채꼴(패링하면 되돌아감)·종 끌기 돌진",
    ),
    ("elites", "엘리트", "0112 엘리트 성질: 굳은·되풀이·남는"),
    (
        "shellback + pods + scavenger",
        "등딱지짐승 + 꼬투리 + 청소자",
        "0120 앞 막기(앞 평타는 튕김)·심은 원판·깡충 물러나기·청소자는 덤비지 않음",
    ),
    (
        "packbearer + repeaters",
        "등짐꾼 + 되풀이꾼",
        "0216 W2 등의 짐은 자세가 깨지면 깨진다(밀고 오지 못하고 느려짐) · 0212 되풀이꾼의 같은 박자",
    ),
    (
        "boss arena",
        "보스 아레나",
        "P2 시점 전환, P3 락온 이동, 보스 예고",
    ),
    (
        "horde 20",
        "무리 20",
        "P1 쿼터뷰 읽힘, 그리기 비용, 떼싸움 느낌",
    ),
    (
        "skills: horde 24 + full gauge",
        "기술 연습(무리 24)",
        "0151 큰 기술 가득 — 모으고 쓸기, 기세(쓰러뜨리면 기술이 빨리 돌아옴), 두 버튼 함께 = 큰 기술",
    ),
    ("pillars + boss", "기둥 + 보스", "P2 카메라 벽 당기기, 가림"),
    ("empty", "빈 아레나", "이동 실험(빠르기·가속·돌기·회피)"),
    (
        "tray spill",
        "쏟기",
        "P6 땅 돌리기: 쏟기·휘청·벽 쿵·멀미 없음",
    ),
    (
        "stress 40",
        "부하 40",
        "P5 성능: 잡몹 40, CPU ms·그리기 수(F3)",
    ),
    (
        "stress 80",
        "부하 80",
        "P5 성능: 잡몹 80, CPU ms·그리기 수(F3)",
    ),
    (
        "husk + thorn",
        "잔재 + 가시짐승",
        "0073 준비 동작 하나씩(들기·버팀·내리침, 뒷발 구르기)",
    ),
];

/// 시나리오의 화면 이름·설명(한국어면 표의 것, 없거나 영어면 원래 글).
pub fn scenario_text(name: &'static str, checks: &'static str) -> (&'static str, &'static str) {
    match lang() {
        Lang::Ko => SCENARIO_KO
            .iter()
            .find(|(k, ..)| *k == name)
            .map_or((name, checks), |(_, n, c)| (n, c)),
        Lang::En => (name, checks),
    }
}

/// 랩 묶음 이름(`lab::GROUPS` 순서와 같다 — 시험).
pub const GROUP_KO: [&str; 17] = [
    "카메라: 쿼터뷰",
    "카메라: 락온",
    "카메라: 전환·벽",
    "이동·회피",
    "공격·콤보",
    "강공격",
    "패링",
    "점프·내려찍기",
    "땅 돌리기·쏟기",
    "락온·자동 조준",
    "피격·스태미나",
    "등잔",
    "마법·던지기",
    "기술",
    "자세",
    "동작",
    "그림(P5 스위치)",
];

/// 랩 값 이름 → 한국어(단위는 뒤에).
pub const SPEC_KO: &[(&str, &str)] = &[
    ("cam.quarter.pitch deg", "쿼터뷰 숙임(도)"),
    ("cam.quarter.dist", "쿼터뷰 거리"),
    ("cam.quarter.fov deg", "쿼터뷰 시야각(도)"),
    ("cam.quarter.follow 1/s", "쿼터뷰 따라가기(/초)"),
    ("cam.quarter.yaw deg", "쿼터뷰 방향(도)"),
    ("cam.lock.fov deg", "락온 시야각(도)"),
    ("cam.free.dist", "숄더뷰 거리"),
    ("cam.free.height", "숄더뷰 높이"),
    ("cam.free.look ahead m", "숄더뷰 주시점 앞(m)"),
    ("cam.free.look height m", "숄더뷰 주시점 높이(m)"),
    ("cam.lock.yaw 1/s", "락온 돌기 빠르기(/초)"),
    ("cam.blend s", "시점 전환 시간(초)"),
    ("cam.collide r", "벽 당기기 반지름"),
    ("move.speed", "걷기 빠르기"),
    ("move.accel", "걷기 가속"),
    ("move.turn deg/s", "몸 돌기(도/초)"),
    ("dodge.dist", "회피 거리"),
    ("dodge.time s", "회피 시간(초)"),
    ("attack.startup s", "평타 선딜(초)"),
    ("attack.active s", "평타 활성(초)"),
    ("attack.recovery s", "평타 후딜(초)"),
    ("attack.range", "평타 거리"),
    ("attack.arc deg", "평타 각(도)"),
    ("feel.hitstop s", "히트스톱(초)"),
    ("feel.knockback", "넉백"),
    ("feel.zoom", "강타·패링 때 카메라 당김(0끔)"),
    ("skill.cooldown x", "기술 대기 배율"),
    ("skill.momentum s", "쓰러뜨리면 기술 대기 당김(초)"),
    ("skill.momentum by skill s", "기술로 쓰러뜨리면 당김(초)"),
    ("skill.chord s", "두 버튼 함께 누름 창(초)"),
    ("skill.gauge per hit", "큰 기술: 맞힐 때 차는 몫"),
    ("skill.gauge per parry", "큰 기술: 패링 때 차는 몫"),
    ("render.enemy lift", "적 밝기(1 = 예전)"),
    ("render.enemy ash", "적 잿빛(0 = 예전)"),
    ("lock.range", "락온 거리 한도"),
    ("lock.lose dist", "락온 풀리는 거리"),
    ("render.face shade", "면 음영"),
    ("cam.lock.yaw max deg/s", "락온 돌기 상한(도/초)"),
    ("cam.lock.near slow m", "가까우면 천천히 돌기(m)"),
    ("cam.lock.catch deg/s", "뒤로 돌면 따라잡기(도/초)"),
    ("cam.lock.catch at deg", "따라잡기 시작 각(도)"),
    ("cam.pull out 1/s", "벽 당긴 뒤 풀기(/초)"),
    ("cam.quarter.lead s", "쿼터뷰 앞서 보기(초)"),
    ("cam.blend turn x", "전환 돌기 배율"),
    ("cam.blend exit x", "락온 풀기 전환 배율"),
    ("move.hold after unlock s", "락온 푼 뒤 방향 유지(초)"),
    (
        "cam.quarter.yaw mode 0fix 1fol 2snap",
        "쿼터뷰 방향 방식(0고정 1따라 2끊어)",
    ),
    ("turn.camera s", "땅 돌리기 카메라(초)"),
    ("turn.overshoot deg", "땅 돌리기 넘침(도)"),
    ("turn.settle s", "땅 돌리기 자리잡기(초)"),
    ("turn.snap kick", "땅 돌리기 끝 튕김"),
    ("turn.plant s", "땅 돌리기 버팀(초)"),
    ("turn.cooldown s", "땅 돌리기 대기(초)"),
    ("spill.range m", "쏟기 범위(m)"),
    ("spill.slide per m", "쏟기 미끄럼(m당)"),
    ("spill.slide max m", "쏟기 미끄럼 상한(m)"),
    ("spill.outward share", "쏟기 바깥쪽 몫"),
    ("spill.stagger s", "쏟기 휘청(초)"),
    ("spill.boss mass", "쏟기 보스 무게"),
    ("spill.boss stagger s", "쏟기 보스 휘청(초)"),
    ("spill.slam dmg", "벽 쿵 피해"),
    ("spill.slam stagger +s", "벽 쿵 휘청 더하기(초)"),
    ("render.horizon haze", "지평선 아지랑이"),
    ("aim.auto range m", "자동 조준 거리(m)"),
    ("aim.auto cone deg", "자동 조준 각(도)"),
    ("attack.lunge m", "평타 내딛기(m)"),
    ("hurt.stun s", "맞은 경직(초)"),
    ("hurt.iframes s", "맞은 뒤 무적(초)"),
    ("hurt.knock m", "맞은 밀림(m)"),
    ("stamina.max", "스태미나 최대"),
    ("stamina.regen /s", "스태미나 회복(/초)"),
    ("stamina.regen delay s", "스태미나 회복 대기(초)"),
    ("stamina.dodge cost", "회피 스태미나"),
    ("stamina.attack cost", "평타 스태미나"),
    ("parry.window s", "패링 창(초)"),
    ("parry.time s", "패링 동작(초)"),
    ("parry.stamina cost", "패링 스태미나"),
    ("parry.stagger s", "패링 휘청(초)"),
    ("parry.boss stagger s", "패링 보스 휘청(초)"),
    ("parry.reach m", "돌진 받는 거리(m)"),
    ("parry.crit bonus", "드러난 적 치명 피해"),
    ("jump.height m", "점프 높이(m)"),
    ("jump.air s", "체공(초)"),
    ("jump.stamina cost", "점프 스태미나"),
    ("jump.air control x", "공중 조종 배율"),
    ("jump.clear m", "파동 넘는 높이(m)"),
    ("jump.land s", "착지 빈틈(초)"),
    ("plunge.damage", "내려찍기 피해"),
    ("plunge.range m", "내려찍기 범위(m)"),
    ("plunge.land s", "내려찍기 빈틈(초)"),
    ("heavy.charge s", "강공격 다 모으기(초)"),
    ("heavy.startup s", "강공격 선딜(초)"),
    ("heavy.active s", "강공격 활성(초)"),
    ("heavy.recovery s", "강공격 후딜(초)"),
    ("heavy.full bonus", "다 모음 피해 덤"),
    ("heavy.range", "강공격 거리"),
    ("heavy.arc deg", "강공격 각(도)"),
    ("heavy.lunge m", "강공격 내딛기(m)"),
    ("heavy.knock x", "강공격 넉백 배율"),
    ("heavy.stagger s", "강공격 휘청(초)"),
    ("heavy.boss stagger s", "강공격 보스 휘청(초)"),
    ("heavy.stamina", "강공격 스태미나"),
    ("heavy.hold start s", "모으기 들어서는 쥠(초)"),
    ("flask.heal", "등잔 회복"),
    ("flask.raise s", "등잔 들기(초)"),
    ("flask.time s", "등잔 동작(초)"),
    ("flask.lock s", "등잔 다시 마시기 대기(초)"),
    ("flask.walk x", "마시며 걷기 배율"),
    ("render.toon 0off 1on", "툰 빛(0끔 1켬)"),
    ("render.rim", "가장자리 빛"),
    ("render.outline px", "외곽선(px)"),
    ("anim.tail gain", "꼬리 흔들림"),
    ("anim.tail hz", "꼬리 빠르기(Hz)"),
    ("anim.tail damp", "꼬리 감쇠"),
    ("anim.head gain", "머리 흔들림"),
    ("anim.lean gain", "몸 기울기"),
    ("anim.squash", "찌그러짐"),
    ("render.lock fade", "락온 가림 옅게"),
    ("render.actor shadow", "배우 그림자"),
    ("heavy.charge walk x", "모으며 걷기 배율"),
    ("cast.speed m/s", "마법 탄 빠르기(m/초)"),
    ("cast.range m", "마법 탄 거리(m)"),
    ("cast.cooldown s", "마법 한 칸이 차는 초"),
    ("cast.startup s", "마법 선딜(초)"),
    ("cast.time s", "마법 동작(초)"),
    ("cast.walk x", "마법 쏘며 걷기 배율"),
    ("combo.keep s", "콤보 이어지는 시간(초)"),
    ("combo.link x", "콤보 끊기 시점 배율"),
    ("attack.buffer s", "누름 받아 두기(초)"),
    ("poise.base", "자세 기본"),
    ("poise.per 10hp", "체력 10당 자세"),
    ("poise.boss", "보스 자세"),
    ("poise.break s", "자세 깨짐(초)"),
    ("poise.break boss s", "보스 자세 깨짐(초)"),
    ("poise.lock s", "깬 뒤 안 차는 시간(초)"),
    ("poise.delay s", "자세 줄기 대기(초)"),
    ("poise.decay /s", "자세 줄기(/초)"),
    ("throw.startup s", "던지기 선딜(초)"),
    ("throw.time s", "던지기 동작(초)"),
    ("poise.break damage x", "깨진 동안 피해 배율"),
    ("cast.hold start s", "원거리 모으기 들어서는 쥠(초)"),
    ("cast.charge s", "원거리 다 모으기(초)"),
    ("dodge.cancel x", "회피 끊기 시점 배율"),
    ("land.cancel x", "착지 끊기 시점 배율"),
    ("render.edge 0off 1on", "화면 선(0끔 1켬)"),
    ("render.edge dark", "어두운 선 세기"),
    ("render.edge light", "밝은 선 세기"),
    ("render.edge depth", "깊이 선 문턱"),
    ("render.edge crease", "모서리 선 문턱"),
    ("render.fog 0off 1on", "높이 안개(0끔 1켬)"),
    ("render.motes 0off 1on", "떠다니는 입자(0끔 1켬)"),
    ("render.sky 0off 1on", "하늘·원경(0끔 1켬)"),
    ("render.flora 0off 1on", "식생(0끔 1켬)"),
    ("render.resident 0off 1on", "지형 GPU 상주(0끔 1켬)"),
    ("render.cull 0off 1on", "시야 밖 거르기(0끔 1켬)"),
    ("render.rows", "내부 해상도 줄 수"),
    ("render.ssaa 0off 1on", "2배로 그려 줄이기(0끔 1켬)"),
    (
        "render.menu fine 0off 1on",
        "메뉴 그림 화면 해상도(0끔 = 게임 도트 1켬)",
    ),
    ("turn.wobble in deg", "땅 돌리기 시작 휘청(도)"),
    ("turn.wobble out deg", "땅 돌리기 끝 휘청(도)"),
    ("turn.start stagger s", "땅 돌리기 시작 적 휘청(초)"),
    ("parry.stamina gain", "패링 성공 기력 회복"),
    ("render.floor calm 0off 1on", "조용한 바닥(0끔 1켬)"),
    ("render.key light 0old 1new", "빛 대비·진한 그림자(0끔 1켬)"),
    ("render.grade", "어두운 중간 밝게(세기)"),
    ("render.bloom", "빛 번짐 세기"),
    ("render.vignette", "가장자리 그늘 세기"),
    ("render.fx 0off 1on", "타격·먼지 입자(0끔 1켬)"),
    ("render.wind", "풀·나무·불 흔들림(세기)"),
    ("hurt.edge", "맞을 때 가장자리 빛(0끔)"),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tuning::SPECS;

    /// 모든 랩 값에 한국어 이름이 하나씩, 표에 없는 이름은 없다(값을 더하거나 이름을 바꾸면 여기도).
    #[test]
    fn every_lab_value_has_one_korean_name() {
        for s in SPECS.iter() {
            assert_eq!(
                SPEC_KO.iter().filter(|(k, _)| *k == s.name).count(),
                1,
                "{}",
                s.name
            );
        }
        for (k, _) in SPEC_KO {
            assert!(SPECS.iter().any(|s| s.name == *k), "stale: {k}");
        }
        assert_eq!(GROUP_KO.len(), crate::lab::GROUPS.len());
        for e in crate::scenario::LIST.iter() {
            assert!(SCENARIO_KO.iter().any(|(k, ..)| *k == e.name), "{}", e.name);
        }
    }
}
