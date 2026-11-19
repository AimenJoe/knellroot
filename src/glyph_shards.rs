//! 글꼴 하나를 아틀라스 여러 장으로 나눈다(0254). macroquad는 `Font` 하나에 아틀라스 하나 — 표의 글자가 늘어 본문 글꼴의
//! 아틀라스가 2048²가 되자 Android Chrome(핸드헬드)에서 본문 글자가 모두 검은 네모로 나왔다. 같은 화면의 큰 제목(굵게,
//! 글자가 적어 1024² 안)은 멀쩡했다 — 0049·0054(4096²에서 검은 글자)와 같은 꼴. 그래서 같은 글꼴을 여러 번 읽어 글자를
//! 나눠 담고(한 장에 `CAP`자 — 1024²에 들어가는 수), 글은 장마다의 토막으로 나눠 그린다. 이 파일은 글자 → 장의 규칙만(순수).

use std::collections::HashMap;
use std::ops::Range;

/// 한 장에 담는 글자 수. 굽는 크기(물리 ≈48px, `ui::BAKE`)의 한글 한 칸 ≈ 47×50 + 틈 4 → 1024²에 20줄 × 20칸 ≈ 400자.
/// 라틴·기호는 더 작지만 여유를 둔다(헤드리스 `texImage2D` 감싸기로 모든 장이 1024² 안인지 잰다).
pub const CAP: usize = 320;

/// 글자가 어느 장에 들어 있나. 처음 본 글자는 자리가 남은 첫 장에, 다 차면 새 장을 연다.
#[derive(Debug, Default)]
pub struct Router {
    route: HashMap<char, usize>,
    counts: Vec<usize>,
}

impl Router {
    /// 장 수(적어도 하나).
    pub fn shards(&self) -> usize {
        self.counts.len().max(1)
    }

    /// 글자의 장 — 처음이면 자리를 정한다.
    pub fn shard(&mut self, c: char) -> usize {
        if let Some(&i) = self.route.get(&c) {
            return i;
        }
        let i = match self.counts.iter().position(|&n| n < CAP) {
            Some(i) => i,
            None => {
                self.counts.push(0);
                self.counts.len() - 1
            }
        };
        self.counts[i] += 1;
        self.route.insert(c, i);
        i
    }

    /// 글을 같은 장의 글자끼리 이어진 토막으로(바이트 구간). 빈칸·줄바꿈은 앞 토막에 붙인다 — 어느 장에나 있다(ASCII는 첫 장에 먼저).
    pub fn runs(&mut self, s: &str) -> Vec<(usize, Range<usize>)> {
        let mut out: Vec<(usize, Range<usize>)> = Vec::new();
        for (at, c) in s.char_indices() {
            let end = at + c.len_utf8();
            let i = match out.last() {
                Some((prev, _)) if c == ' ' => *prev,
                _ => self.shard(c),
            };
            match out.last_mut() {
                Some((prev, r)) if *prev == i => r.end = end,
                _ => out.push((i, at..end)),
            }
        }
        out
    }

    /// 장마다 담을 글자(미리 굽기용) — 처음 본 글자는 자리를 정한다.
    pub fn split(&mut self, chars: &[char]) -> Vec<Vec<char>> {
        let mut out: Vec<Vec<char>> = Vec::new();
        for &c in chars {
            let i = self.shard(c);
            if out.len() <= i {
                out.resize(i + 1, Vec::new());
            }
            out[i].push(c);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_shard_holds_at_most_cap_glyphs_and_a_glyph_stays_where_it_went() {
        let mut r = Router::default();
        let chars: Vec<char> = ('가'..='힣').take(CAP * 2 + 5).collect();
        let parts = r.split(&chars);
        assert_eq!(parts.len(), 3);
        assert!(parts.iter().all(|p| p.len() <= CAP));
        assert_eq!(r.shards(), 3);
        // 다시 물어도 같은 장.
        assert_eq!(r.shard(chars[0]), 0);
        assert_eq!(r.shard(chars[CAP]), 1);
        assert_eq!(r.shards(), 3);
    }

    #[test]
    fn text_splits_into_runs_that_cover_it_exactly_and_spaces_join_the_run_before() {
        let mut r = Router::default();
        let mut first: Vec<char> = (32u8..127).map(char::from).collect();
        first.extend(('가'..='힣').take(CAP - first.len()));
        r.split(&first);
        let (a, b) = ('가', ('가'..='힣').nth(CAP + 10).unwrap());
        let s = format!("{a}{a} {b} {a}A");
        let runs = r.runs(&s);
        // 토막을 이으면 글 그대로.
        let joined: String = runs.iter().map(|(_, g)| &s[g.clone()]).collect();
        assert_eq!(joined, s);
        // 첫 장 "가가 " · 둘째 장 "힣.. " · 첫 장 "가A" — 빈칸은 앞 토막에 붙는다.
        let got: Vec<(usize, &str)> = runs.iter().map(|(i, g)| (*i, &s[g.clone()])).collect();
        let (b1, b2) = (format!("{b} "), format!("{a}A"));
        assert_eq!(got, vec![(0, "가가 "), (1, b1.as_str()), (0, b2.as_str())]);
    }
}
