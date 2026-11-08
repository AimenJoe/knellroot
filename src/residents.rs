//! 떠난 마을의 남은 주민(0122, v0.5.0 C7 — 0115·0116, 설계 문서 `docs/design/stages/abandoned-village-residents.md`).
//! 주민 = 표 한 줄: 기록 이름(이벤트 ID 앞), 지도 표지, 짧은 말 셋 + 재방문 뒤의 한마디, 상자 배우의 빛.
//! 규칙(CANON): 서로 떨어져 제 일을 한다 · 짧고 제한적인 말 · 방랑자에게 무심한 경계(가까이 오면 돌아본다) ·
//! 싸움에 끼지 않는다(월드의 적이 아니다 — 맞지도, 락온되지도 않는다) · 문지기를 이겨도 그대로 · 말을 걸지 않아도 진행된다.
//! 재방문(0116): 그 주민을 처음 본 판이 끝난 뒤, 다른 판에서 마을에 다시 들어서면 한 번 바뀐다(단방향, 기록).
use macroquad::prelude::Color;

use crate::i18n::Tr;

/// 생활 행동의 꼴(렌더가 곁의 물건을 그린다 — 재방문 변화도 여기).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Work {
    /// 집 아궁이를 살핀다(쉬는 화로가 아니다, 0116) — 재방문: 불빛이 조금 살아난다.
    Fire,
    /// 뿌리에서 떨어지는 물을 항아리에 받는다 — 재방문: 기울어진 항아리가 바로 서 있다.
    Water,
    /// 천을 꿰맨다 — 재방문: 찢어진 천막 일부가 꿰매져 있다.
    Cloth,
}

pub struct Resident {
    /// 기록 이름(이벤트 ID `V01-E01`의 앞).
    pub id: &'static str,
    /// 지도 표지.
    pub mark: char,
    pub work: Work,
    /// 늘 하는 짧은 말(말 걸 때마다 다음 줄, 끝나면 처음부터).
    pub lines: [Tr; 3],
    /// 재방문 뒤에는 이 한마디가 맨 앞에 선다.
    pub after: Tr,
    /// 상자 배우의 몸빛(작은 동물 — 귀여움 문법 0036은 모델 단계).
    pub body: Color,
}

pub static RESIDENTS: [Resident; 3] = [
    Resident {
        id: "V01",
        mark: '1',
        work: Work::Fire,
        lines: [|s| s.v01_a, |s| s.v01_b, |s| s.v01_c],
        after: |s| s.v01_after,
        body: Color::new(0.78, 0.55, 0.40, 1.0),
    },
    Resident {
        id: "V02",
        mark: '2',
        work: Work::Water,
        lines: [|s| s.v02_a, |s| s.v02_b, |s| s.v02_c],
        after: |s| s.v02_after,
        body: Color::new(0.55, 0.66, 0.74, 1.0),
    },
    Resident {
        id: "V03",
        mark: '3',
        work: Work::Cloth,
        lines: [|s| s.v03_a, |s| s.v03_b, |s| s.v03_c],
        after: |s| s.v03_after,
        body: Color::new(0.74, 0.66, 0.50, 1.0),
    },
];

/// 표지의 주민(없으면 None).
pub fn by_mark(c: char) -> Option<usize> {
    RESIDENTS.iter().position(|r| r.mark == c)
}

/// 말 걸기 `n`번째(0부터)에 보일 말: 재방문 뒤면 한마디가 맨 앞, 그다음 늘 하는 말 셋을 돈다.
pub fn line(r: &Resident, changed: bool, n: usize) -> Tr {
    if changed {
        if n.is_multiple_of(4) {
            r.after
        } else {
            r.lines[(n % 4) - 1]
        }
    } else {
        r.lines[n % 3]
    }
}

/// 재방문 규칙(0116): 처음 본 판(`seen_run`)이 끝난 뒤의 다른 판(`now_run`)이면 바뀐다.
pub fn revisits(seen_run: u32, now_run: u32) -> bool {
    now_run > seen_run
}

/// 이 판에 이 주민이 바뀐 모습인가(0116 재방문): 이미 바뀌었거나, 처음 본 판보다 뒤의 판. 새로 바뀌면 `changed`에 넣는다(단방향).
pub fn settle(id: &str, seen: &[(String, u32)], changed: &mut Vec<String>, runs: u32) -> bool {
    if changed.iter().any(|c| c == id) {
        return true;
    }
    let now = seen.iter().any(|(s, n)| s == id && revisits(*n, runs));
    if now {
        changed.push(id.to_string());
        changed.sort();
    }
    now
}

/// 이 거리(m) 안에 들어오면 "봤다"(기록) — 주민 자리에서 한 눈에 들어오는 만큼.
pub const SEE_RANGE: f32 = 10.0;
/// 이 거리 안이면 돌아본다(무심한 경계), 말을 걸 수 있다.
pub const GLANCE_RANGE: f32 = 5.0;
pub const TALK_RANGE: f32 = 2.2;
/// 몸 반지름(방랑자만 밀어낸다 — 공격·탄은 지나간다).
pub const BODY_R: f32 = 0.4;
/// 말풍선이 떠 있는 초(멈추지 않는다, 0116).
pub const BUBBLE_TIME: f32 = 3.5;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_cycle_and_the_revisit_word_leads_after_a_change() {
        let r = &RESIDENTS[0];
        let s = crate::i18n::s();
        assert_eq!(line(r, false, 0)(s), (r.lines[0])(s));
        assert_eq!(line(r, false, 3)(s), (r.lines[0])(s), "wraps");
        assert_eq!(line(r, true, 0)(s), (r.after)(s), "the changed word first");
        assert_eq!(line(r, true, 1)(s), (r.lines[0])(s));
        assert_eq!(line(r, true, 4)(s), (r.after)(s));
    }

    #[test]
    fn revisit_changes_stick_once_set_and_need_a_later_run() {
        let seen = vec![("V01".to_string(), 2)];
        let mut changed = Vec::new();
        assert!(!settle("V01", &seen, &mut changed, 2), "same run");
        assert!(!settle("V02", &seen, &mut changed, 5), "never seen");
        assert!(settle("V01", &seen, &mut changed, 3));
        assert_eq!(changed, vec!["V01"]);
        assert!(
            settle("V01", &[], &mut changed, 0),
            "one way: stays changed"
        );
        assert_eq!(changed.len(), 1);
    }

    #[test]
    fn a_resident_changes_only_in_a_later_run_than_first_seen() {
        assert!(!revisits(3, 3), "the same run: not yet");
        assert!(revisits(3, 4));
        let marks: Vec<char> = RESIDENTS.iter().map(|r| r.mark).collect();
        assert_eq!(marks, vec!['1', '2', '3']);
        assert_eq!(by_mark('2'), Some(1));
        assert_eq!(by_mark('a'), None);
    }
}
