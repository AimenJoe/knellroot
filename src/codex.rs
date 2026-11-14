//! 도감(0130, v0.5.0 C5 — 0116 "적·장신구 탭, 본 것은 그림 + 관찰, 못 본 것은 실루엣 + 나오는 곳, 관찰과 해석을 가른다").
//! 적 한 줄 = 종 하나. 관찰 줄은 본 만큼 열린다: 처음 본 때(관찰 `·` + 추측 `?`), 같은 종의 다른 역할(청소자), 엘리트 성질, 반 아래 깨어남.
//! 기록은 본 것의 이름 목록(`save::key::CODEX`, 이름은 바꾸지 않는다): `<종>` · `<종>.docile` · `<종>.elite.<성질>` · `<종>.roused`.
//! 글 규칙(lore ecology 10.1): 관찰한 사실과 쓴 이의 추측을 가르고, 과거를 다 말하지 않는다. 뿌리의 의도·핵의 정체는 쓰지 않는다.
use crate::enemy::{Affix, Enemy};
use crate::i18n::Tr;
use crate::species::Kind;

/// 관찰 줄이 열리는 때.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Obs {
    /// 처음 본 때 — 관찰(`·`).
    Seen,
    /// 처음 본 때 — 추측(`?`, 쓴 이의 생각).
    Guess,
    /// 같은 종의 다른 역할을 본 때(청소자, 0120).
    Docile,
    /// 그 성질의 엘리트를 본 때(0112).
    Elite(Affix),
    /// 체력 반 아래로 깨어난 것을 본 때(보스).
    Roused,
}

pub struct Entry {
    pub kind: Kind,
    /// 나오는 곳(못 본 것에도 보인다).
    pub place: Tr,
    pub lines: &'static [(Obs, Tr)],
}

/// 표 순서 = 도감 순서(만나는 차례).
pub static ENTRIES: [Entry; 7] = [
    Entry {
        kind: Kind::Husk,
        place: |s| s.cx_place_both,
        lines: &[
            (Obs::Seen, |s| s.cx_husk_1),
            (Obs::Seen, |s| s.cx_husk_2),
            (Obs::Guess, |s| s.cx_husk_g),
            (Obs::Docile, |s| s.cx_husk_docile),
            (Obs::Elite(Affix::Hardened), |s| s.cx_elite_hardened),
            (Obs::Elite(Affix::Lingering), |s| s.cx_elite_lingering),
        ],
    },
    Entry {
        kind: Kind::Thorn,
        place: |s| s.cx_place_both,
        lines: &[
            (Obs::Seen, |s| s.cx_thorn_1),
            (Obs::Seen, |s| s.cx_thorn_2),
            (Obs::Guess, |s| s.cx_thorn_g),
            (Obs::Elite(Affix::Repeating), |s| s.cx_elite_repeating),
        ],
    },
    Entry {
        kind: Kind::Warden,
        place: |s| s.cx_place_warden,
        lines: &[
            (Obs::Seen, |s| s.cx_warden_1),
            (Obs::Seen, |s| s.cx_warden_2),
            (Obs::Guess, |s| s.cx_warden_g),
            (Obs::Roused, |s| s.cx_roused),
        ],
    },
    Entry {
        kind: Kind::Shell,
        place: |s| s.cx_place_village,
        lines: &[
            (Obs::Seen, |s| s.cx_shell_1),
            (Obs::Seen, |s| s.cx_shell_2),
            (Obs::Guess, |s| s.cx_shell_g),
            (Obs::Elite(Affix::Hardened), |s| s.cx_elite_hardened),
        ],
    },
    Entry {
        kind: Kind::Pod,
        place: |s| s.cx_place_village,
        lines: &[
            (Obs::Seen, |s| s.cx_pod_1),
            (Obs::Seen, |s| s.cx_pod_2),
            (Obs::Guess, |s| s.cx_pod_g),
            (Obs::Elite(Affix::Repeating), |s| s.cx_elite_repeating),
        ],
    },
    Entry {
        kind: Kind::Bellkeeper,
        place: |s| s.cx_place_bell,
        lines: &[
            (Obs::Seen, |s| s.cx_bell_1),
            (Obs::Seen, |s| s.cx_bell_2),
            (Obs::Guess, |s| s.cx_bell_g),
            (Obs::Roused, |s| s.cx_roused),
        ],
    },
    Entry {
        kind: Kind::Mossward,
        place: |s| s.cx_place_sill,
        lines: &[
            (Obs::Seen, |s| s.cx_moss_1),
            (Obs::Seen, |s| s.cx_moss_2),
            (Obs::Guess, |s| s.cx_moss_g),
            (Obs::Roused, |s| s.cx_roused),
        ],
    },
];

/// 종의 기록 이름(바꾸지 않는다 — `_` 없는 match: 새 종이면 컴파일러가 묻는다).
pub fn kind_key(k: Kind) -> &'static str {
    match k {
        Kind::Husk => "husk",
        Kind::Thorn => "thorn",
        Kind::Warden => "warden",
        Kind::Shell => "shell",
        Kind::Pod => "pod",
        Kind::Bellkeeper => "bellkeeper",
        Kind::Mossward => "mossward",
    }
}

fn affix_key(a: Affix) -> &'static str {
    match a {
        Affix::Hardened => "hardened",
        Affix::Repeating => "repeating",
        Affix::Lingering => "lingering",
    }
}

/// 이 줄의 기록 이름(이것이 기록에 있으면 줄이 열린다).
pub fn obs_key(k: Kind, o: Obs) -> String {
    let kk = kind_key(k);
    match o {
        Obs::Seen | Obs::Guess => kk.to_string(),
        Obs::Docile => format!("{kk}.docile"),
        Obs::Elite(a) => format!("{kk}.elite.{}", affix_key(a)),
        Obs::Roused => format!("{kk}.roused"),
    }
}

/// 이 적을 지금 보면 적히는 이름들(종 + 역할·성질·깨어남).
pub fn seen_keys(e: &Enemy) -> Vec<String> {
    let mut v = vec![obs_key(e.kind, Obs::Seen)];
    if e.docile {
        v.push(obs_key(e.kind, Obs::Docile));
    }
    if let Some(a) = e.elite {
        v.push(obs_key(e.kind, Obs::Elite(a)));
    }
    if e.roused {
        v.push(obs_key(e.kind, Obs::Roused));
    }
    v
}

/// 장신구가 나오는 곳(얻지 못한 것에 보인다 — `_` 없는 match).
pub fn charm_place(id: crate::charms::Id) -> Tr {
    use crate::charms::Id as C;
    match id {
        C::Shell | C::Feet | C::Hunger | C::Pouch => |s| s.cx_charm_shop,
        C::Ear => |s| s.cx_charm_ear,
        C::Hand => |s| s.cx_charm_hand,
        C::Cloth => |s| s.cx_charm_cloth,
        C::Jar => |s| s.cx_charm_jar,
        C::BellShard => |s| s.cx_charm_bellshard,
    }
}

/// 본 것으로 적는 거리(m) — 적 시야(9m)와 같게: 깨어난 적을 이만큼 가까이에서 보면.
pub const SEE: f32 = 9.0;

/// 이 종을 봤나.
pub fn seen(record: &[String], k: Kind) -> bool {
    record.iter().any(|s| s == kind_key(k))
}

/// 열린 줄(관찰이면 거짓, 추측이면 참 — 줄 머리 `·` / `?`).
pub fn open_lines(record: &[String], e: &Entry) -> Vec<(bool, Tr)> {
    e.lines
        .iter()
        .filter(|(o, _)| record.contains(&obs_key(e.kind, *o)))
        .map(|&(o, t)| (o == Obs::Guess, t))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 0130: 모든 종이 도감에 한 번, 줄은 본 만큼 열린다 — 처음 보면 관찰·추측, 청소자·엘리트·깨어남은 그때.
    #[test]
    fn every_kind_has_one_entry_and_lines_open_as_you_see_more() {
        for d in crate::species::ALL.iter() {
            let n = ENTRIES.iter().filter(|e| e.kind == d.kind).count();
            assert_eq!(n, 1, "{:?}", d.kind);
        }
        let husk = &ENTRIES[0];
        assert!(open_lines(&[], husk).is_empty());
        let rec = vec!["husk".to_string()];
        let first = open_lines(&rec, husk);
        assert_eq!(first.len(), 3, "two observations and a guess");
        assert_eq!(first.iter().filter(|l| l.0).count(), 1, "one guess");
        let mut e = Enemy::new(Kind::Husk, macroquad::prelude::Vec3::ZERO);
        e.docile = true;
        e.make_elite(Affix::Lingering);
        let rec = seen_keys(&e);
        assert_eq!(rec, vec!["husk", "husk.docile", "husk.elite.lingering"]);
        assert_eq!(open_lines(&rec, husk).len(), 5);
        assert!(seen(&rec, Kind::Husk) && !seen(&rec, Kind::Thorn));
    }
}
