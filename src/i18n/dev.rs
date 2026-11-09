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
pub const LAB_HINT: Pair = (
    "{0} · 기본 {1} · {2} ×10 · 확인 = 기본값",
    "default {1} | {2} x10 | confirm = default",
);
pub const DEV_TITLE: Pair = ("개발", "DEV");
pub const TAB_TOOLS: Pair = ("도구", "tools");
pub const TAB_STAGE: Pair = ("스테이지", "stage");
pub const TAB_COMBAT: Pair = ("전투·적", "combat");
pub const TAB_PERF: Pair = ("성능", "perf");
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
pub const RESIDENTS: Pair = (
    "둘째 지역 주민 곁에서(말 걸기)",
    "region 2 beside a resident (talk)",
);
pub const STONES: Pair = (
    "둘째 지역 새긴 돌 곁에서(읽기)",
    "region 2 beside a carved stone (read)",
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
    LAB_HINT,
    DEV_TITLE,
    TAB_TOOLS,
    TAB_STAGE,
    TAB_COMBAT,
    TAB_PERF,
    DEV_HINT,
    ENTRANCE,
    GATE,
    VILLAGE,
    RESIDENTS,
    STONES,
    STAGE_NOTE,
    LAB_START,
    LAB_NOTE,
    SCENARIO_NOTE,
];

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
        "boss arena",
        "보스 아레나",
        "P2 시점 전환, P3 락온 이동, 보스 예고",
    ),
    (
        "horde 20",
        "무리 20",
        "P1 쿼터뷰 읽힘, 그리기 비용, 떼싸움 느낌",
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
pub const GROUP_KO: [&str; 16] = [
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
    ("cam.lock.dist", "락온 거리"),
    ("cam.lock.height", "락온 높이"),
    ("cam.lock.shoulder", "락온 어깨 비키기"),
    ("cam.lock.fov deg", "락온 시야각(도)"),
    ("cam.lock.look ahead m", "락온 주시점 밀기(m)"),
    ("cam.lock.frame x", "락온 구도 배율"),
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
    ("lock.range", "락온 거리 한도"),
    ("lock.lose dist", "락온 풀리는 거리"),
    ("render.face shade", "면 음영"),
    ("cam.lock.yaw max deg/s", "락온 돌기 상한(도/초)"),
    ("cam.lock.near slow m", "가까우면 천천히 돌기(m)"),
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
    ("poise.per hp", "체력당 자세"),
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
    ("turn.wobble in deg", "땅 돌리기 시작 휘청(도)"),
    ("turn.wobble out deg", "땅 돌리기 끝 휘청(도)"),
    ("turn.start stagger s", "땅 돌리기 시작 적 휘청(초)"),
    ("parry.stamina gain", "패링 성공 기력 회복"),
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
