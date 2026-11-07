//! Lab(KB guides/poc-and-lab-before-content.md, 0070): 실행 중에 `Tuning`을 바꾸고, 슬롯 A/B를 번갈아 느끼고,
//! `tuning.rs`에 붙여 넣을 줄을 내보낸다. 0066 창 틀 안의 개발 창이다(`app::lab_screen` — 타이틀·멈춤 메뉴의 설정 아래 줄 "연구소"·F1).
//! 창이 열린 동안 게임은 멈추고, 바꾼 값은 바로 `Tuning`에 들어가 창을 닫은 뒤에도 계속 쓰인다(판을 다시 해도 —
//! 게임을 끄면 사라진다: 진실은 `tuning.rs` 하나, 0013). 개발 전용 화면이라 글은 영어 ASCII.
use crate::tuning::{Tuning, SPECS};

/// 값 묶음(이름 앞머리로 가른다 — 위에서부터 처음 맞는 묶음). 새 값은 이름만 맞게 지으면 저절로 들어간다(시험이 본다).
pub const GROUPS: &[(&str, &[&str])] = &[
    ("camera: quarter view", &["cam.quarter."]),
    ("camera: lock-on view", &["cam.lock."]),
    ("camera: blend & walls", &["cam."]),
    ("move & dodge", &["move.", "dodge."]),
    ("attack & combo", &["attack.", "combo.", "feel."]),
    ("heavy attack", &["heavy."]),
    ("parry", &["parry."]),
    ("jump & plunge", &["jump.", "plunge.", "land."]),
    ("ground turn & spill", &["turn.", "spill."]),
    ("lock-on & auto aim", &["lock.", "aim."]),
    ("hurt & stamina", &["hurt.", "stamina."]),
    ("flask", &["flask."]),
    ("magic & throw", &["cast.", "throw."]),
    ("poise", &["poise."]),
    ("animation", &["anim."]),
    ("render (P5 switches)", &["render."]),
];

/// 값 `name`의 묶음 번호.
pub fn group_of(name: &str) -> Option<usize> {
    GROUPS
        .iter()
        .position(|(_, pre)| pre.iter().any(|p| name.starts_with(p)))
}

/// 묶음 `g`에 든 값의 `SPECS` 번호(표 순서).
pub fn members(g: usize) -> Vec<usize> {
    (0..SPECS.len())
        .filter(|&i| group_of(SPECS[i].name) == Some(g))
        .collect()
}

/// 값 글(걸음 크기에 맞춘 자릿수).
pub fn value_text(v: f32, step: f32) -> String {
    if step >= 1.0 {
        format!("{v:.0}")
    } else if step >= 0.1 {
        format!("{v:.2}")
    } else {
        format!("{v:.3}")
    }
}

/// 기본값에서 바뀌었나.
pub fn changed(t: &Tuning, i: usize) -> bool {
    (t.get(i) - SPECS[i].def).abs() > 1e-6
}

/// 랩의 상태: 슬롯 B(담아 둔 값들)와 지금 어느 쪽을 쓰나, 마지막 알림.
#[derive(Default)]
pub struct Lab {
    slot: Option<Tuning>,
    pub using_b: bool,
    pub note: String,
}

impl Lab {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_b(&self) -> bool {
        self.slot.is_some()
    }

    /// 지금 값들을 슬롯 B에(지금은 A).
    pub fn store(&mut self, t: &Tuning) {
        self.slot = Some(t.clone());
        self.using_b = false;
        self.note = "stored current values as slot B".into();
    }

    /// A ↔ B.
    pub fn swap(&mut self, t: &mut Tuning) {
        if let Some(b) = self.slot.as_mut() {
            std::mem::swap(t, b);
            self.using_b = !self.using_b;
            self.note = format!("now using {}", if self.using_b { "B" } else { "A" });
        } else {
            self.note = "no slot B yet (store one first)".into();
        }
    }

    /// 콘솔에 낼 `tuning.rs` 줄.
    pub fn export(&mut self, t: &Tuning) -> String {
        self.note = "exported to console (paste into src/tuning.rs)".into();
        t.export()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_lab_value_sits_in_exactly_one_group_and_no_group_is_empty() {
        for s in SPECS.iter() {
            assert!(group_of(s.name).is_some(), "{} has no group", s.name);
        }
        let total: usize = (0..GROUPS.len()).map(|g| members(g).len()).sum();
        assert_eq!(total, SPECS.len());
        for (g, (name, _)) in GROUPS.iter().enumerate() {
            assert!(!members(g).is_empty(), "{name} is empty");
        }
    }

    #[test]
    fn slot_b_swaps_and_comes_back() {
        let mut lab = Lab::new();
        let mut t = Tuning::default();
        lab.swap(&mut t);
        assert!(!lab.using_b, "nothing to swap with yet");
        lab.store(&t);
        t.set(0, t.get(0) + SPECS[0].step);
        let a = t.get(0);
        lab.swap(&mut t);
        assert!(lab.using_b && t.get(0) == SPECS[0].def);
        lab.swap(&mut t);
        assert!(!lab.using_b && t.get(0) == a);
    }
}
