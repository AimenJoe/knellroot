//! 판에서 일어난 것을 기록(`save::Progress`)에 적는 규칙(0251 R7 — `App::sync_progress`에서 뺐다).
//!
//! 앱은 스테이지가 이 틱에 알린 것(주운 것·처음 본 것·연 문·쓰러뜨린 엘리트·쥔 핵 …)을 여기에 넘기고, 돌아온 값으로 소리·몸짓·알림만 낸다.
//! 기록의 목록은 정렬하고 겹치지 않게 둔다(같은 판이면 같은 글 — 저장은 바뀌었을 때만 쓴다).
use crate::save::Progress;

/// 핵을 쥐었을 때(0105 · 0125 · 0185 · 0208): 문지기마다 `CORE_CAP`까지.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreTaken {
    /// 새 수(상한이면 None — "더는 스미지 않는다").
    pub count: Option<u32>,
    /// 이 문지기를 처음 이겼다(핵이 몸에 스민다는 카드).
    pub first: bool,
}

/// 숨은 물건을 주웠을 때(0105 · 0176).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Found {
    /// 덧옷이면 그 덧옷.
    pub garb: Option<crate::garb::Id>,
    /// 아무것도 안 걸쳤어서 바로 걸쳤다(장신구가 빈 칸에 끼워지는 것과 같은 꼴).
    pub put_on: bool,
}

fn add_sorted(list: &mut Vec<String>, id: String) {
    if !list.contains(&id) {
        list.push(id);
        list.sort();
    }
}

impl Progress {
    /// 도감(0130): 처음 본 것들.
    pub fn note_codex(&mut self, seen: Vec<String>) {
        if seen.is_empty() {
            return;
        }
        for k in seen {
            if !self.codex.contains(&k) {
                self.codex.push(k);
            }
        }
        self.codex.sort();
    }

    /// 주민(0122): 처음 본 판(재방문 규칙 — 그 판이 끝난 뒤의 다른 판).
    pub fn note_resident_seen(&mut self, id: &str) {
        if !self.residents_seen.iter().any(|(s, _)| s == id) {
            self.residents_seen.push((id.to_string(), self.runs));
        }
    }

    /// 숨은 물건(0105): 기록 이름 — 덧옷(0176)은 아무것도 안 걸쳤으면 바로 걸친다.
    pub fn note_find(&mut self, id: String) -> Found {
        let garb = crate::garb::of_find(&id);
        add_sorted(&mut self.finds, id);
        let put_on = garb.is_some() && self.garb.is_none();
        if put_on {
            self.garb = garb;
        }
        Found { garb, put_on }
    }

    /// 문 장치(0113): 처음 연 문(판을 넘어 열려 있다).
    pub fn note_gate(&mut self, id: String) {
        add_sorted(&mut self.gates, id);
    }

    /// 엘리트(0112): 처음 쓰러뜨린 것들.
    pub fn note_elites(&mut self, new: Vec<String>) {
        for id in new {
            if !self.elites.contains(&id) {
                self.elites.push(id);
            }
        }
        self.elites.sort();
    }

    /// 핵을 쥐었다(지역 `stage`의 문지기): 그 문지기의 핵 수를 하나(상한까지). 첫 지역은 끝낸 판 수(`clears`)도 센다 —
    /// "처음"은 첫 지역만 끝낸 판 수로, 다른 지역은 그 문지기의 핵이 0이었나로 본다(예전 그대로).
    pub fn take_core(&mut self, stage: usize) -> CoreTaken {
        let cap = crate::progression::CORE_CAP;
        let first = if stage == 0 {
            self.clears == 0
        } else {
            *self.cores_of(stage) == 0
        };
        let cores = self.cores_of(stage);
        let count = (*cores < cap).then(|| {
            *cores += 1;
            *cores
        });
        if stage == 0 {
            self.clears += 1;
        }
        CoreTaken { count, first }
    }

    fn cores_of(&mut self, stage: usize) -> &mut u32 {
        match stage {
            0 => &mut self.cores,
            1 => &mut self.bell_cores,
            2 => &mut self.moss_cores,
            _ => &mut self.overseer_cores,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notes_are_sorted_and_never_doubled() {
        let mut p = Progress::default();
        p.note_codex(vec!["thorn".into(), "husk".into(), "thorn".into()]);
        p.note_codex(vec!["husk".into()]);
        assert_eq!(p.codex, vec!["husk", "thorn"]);
        p.note_gate("village".into());
        p.note_gate("sanctuary".into());
        p.note_gate("village".into());
        assert_eq!(p.gates, vec!["sanctuary", "village"]);
        p.note_elites(vec!["b".into(), "a".into()]);
        p.note_elites(vec!["a".into()]);
        assert_eq!(p.elites, vec!["a", "b"]);
    }

    /// 주민을 처음 본 판은 한 번만 적힌다(그 뒤의 판에서 다시 봐도 처음 본 판 그대로).
    #[test]
    fn a_resident_keeps_the_run_it_was_first_seen_in() {
        let mut p = Progress::default();
        p.runs = 2;
        p.note_resident_seen("fire");
        p.runs = 5;
        p.note_resident_seen("fire");
        assert_eq!(p.residents_seen, vec![("fire".to_string(), 2)]);
    }

    /// 덧옷: 아무것도 안 걸쳤으면 주운 것을 바로 걸치고, 걸친 게 있으면 그대로 둔다(가진 것에는 든다).
    #[test]
    fn a_found_garment_is_worn_only_when_nothing_is() {
        let cloak = crate::garb::TABLE[0].find;
        let bark = crate::garb::TABLE[1].find;
        let mut p = Progress::default();
        let f = p.note_find(cloak.into());
        assert!(f.put_on && f.garb.is_some());
        assert_eq!(p.garb, f.garb);
        let g = p.note_find(bark.into());
        assert!(!g.put_on && g.garb.is_some());
        assert_eq!(p.garb, f.garb, "the first one stays on");
        let h = p.note_find("sanctuary.lamp.0".into());
        assert_eq!((h.garb, h.put_on), (None, false));
        assert_eq!(p.finds.len(), 3);
    }

    /// 핵: 문지기마다 상한까지, 그 뒤로는 수가 없다. 첫 지역만 끝낸 판 수를 센다. "처음"은 그 문지기의 첫 승리.
    #[test]
    fn cores_count_per_warden_up_to_the_cap_and_only_the_first_region_counts_clears() {
        let cap = crate::progression::CORE_CAP;
        let mut p = Progress::default();
        let first = p.take_core(0);
        assert_eq!(
            first,
            CoreTaken {
                count: Some(1),
                first: true
            }
        );
        for _ in 1..cap {
            assert!(!p.take_core(0).first);
        }
        assert_eq!(
            p.take_core(0),
            CoreTaken {
                count: None,
                first: false
            }
        );
        assert_eq!(p.cores, cap);
        assert_eq!(
            p.clears,
            cap + 1,
            "every win in the first region is a clear"
        );
        let bell = p.take_core(1);
        assert_eq!(
            bell,
            CoreTaken {
                count: Some(1),
                first: true
            }
        );
        assert_eq!((p.bell_cores, p.moss_cores, p.overseer_cores), (1, 0, 0));
        assert_eq!(p.clears, cap + 1, "deeper wins are not clears");
        assert!(p.take_core(2).first);
        assert!(p.take_core(3).first);
        assert_eq!((p.moss_cores, p.overseer_cores), (1, 1));
    }
}
