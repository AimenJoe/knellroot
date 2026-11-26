//! 골든 트레이스(0278 R12 — 리팩터링의 안전망): 스킬마다(최대 레벨 · 고르기 ㄱ/ㄴ) · 궁극기마다(최대 레벨) · 무기마다의 RB(톡 ·
//! 쥐었다 떼기) · 방패 던지기를 **같은 고정 마당**에서 한 번씩 쓰고, 정한 틱 동안의 사건 차례와 틱마다의 적 체력 · 자세 · 자리,
//! 방랑자의 자리 · 체력을 해시 하나로 접는다. 값은 `golden.txt`(한 줄에 `이름 해시`).
//!
//! 동작을 바꾸지 않는 리팩터링(R14~R25)은 이 해시가 그대로여야 한다. 일부러 동작을 바꿨다면(값 · 규칙) 다시 쓴다:
//!
//! ```sh
//! GOLDEN_WRITE=1 cargo test golden_traces
//! ```
//!
//! 수는 1e-3으로 반올림해 접는다(libm의 끝자리 차이로 뒤집히지 않게). 해시는 FNV-1a 64(러스트 판과 상관없이 같다).
//! 월드는 이 길에서 씨앗 없는 난수를 쓰지 않는다(잔불 흩기의 씨앗은 월드 시각에서 — 같은 마당이면 같다).
use super::*;
use crate::skills::{Loadout, Pick, Slot, Ult, IDS, MAX_RANK, ULTS, ULT_MAX_RANK};

const FILE: &str = "src/world/tests/golden.txt";
const EXPECTED: &str = include_str!("golden.txt");
/// 한 경우의 길이(틱) — 놓인 것 · 돌아오는 것 · 늦게 터지는 것이 다 끝나게 6초.
const TICKS: usize = 360;

/// FNV-1a 64.
pub(super) struct Fnv(pub(super) u64);

impl Fnv {
    pub(super) fn new() -> Self {
        Fnv(0xcbf2_9ce4_8422_2325)
    }
    pub(super) fn put(&mut self, s: &str) {
        for b in s.bytes().chain(std::iter::once(0xff)) {
            self.0 ^= b as u64;
            self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
}

pub(super) fn q(x: f32) -> i64 {
    let v = (x as f64 * 1000.0).round() as i64;
    // -0 → 0
    if v == 0 {
        0
    } else {
        v
    }
}

/// 사건의 Debug 글에서 수(소수점 · 지수가 든 것)를 1e-3으로 반올림한다 — 갈래 이름 · id · bool · 정수는 그대로.
pub(super) fn quantized(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < b.len() {
        let c = b[i] as char;
        let starts_num = c.is_ascii_digit()
            || (c == '-' && i + 1 < b.len() && (b[i + 1] as char).is_ascii_digit());
        // 이름 안의 숫자(예: `Vec3`)는 수가 아니다.
        let in_word = i > 0 && {
            let p = b[i - 1] as char;
            p.is_ascii_alphanumeric() || p == '_'
        };
        if starts_num && !in_word {
            let mut j = i + 1;
            while j < b.len() {
                let d = b[j] as char;
                let exp_sign = (d == '-' || d == '+') && matches!(b[j - 1] as char, 'e' | 'E');
                if d.is_ascii_digit() || d == '.' || d == 'e' || d == 'E' || exp_sign {
                    j += 1;
                } else {
                    break;
                }
            }
            let tok = &s[i..j];
            if tok.contains(['.', 'e', 'E']) {
                match tok.parse::<f64>() {
                    Ok(v) => out.push_str(&q(v as f32).to_string()),
                    Err(_) => out.push_str(tok),
                }
            } else {
                out.push_str(tok);
            }
            i = j;
        } else {
            out.push(c);
            i += 1;
        }
    }
    out
}

/// 고정 마당: 30 m 둘레 · 방랑자는 가운데에서 화면 위(-z)를 본다 · 보스 하나(앞 6 m) · 잡몹 다섯(거리 · 쪽을 정해서).
/// 처음에는 모두 서 있다(쫓지도 치지도 않는다) — 맞으면 깨어나는 것은 월드의 규칙대로.
pub(super) fn yard() -> World {
    let mut w = World::new("test", 30.0);
    w.player.facing = std::f32::consts::PI;
    w.spawn_boss(vec3(0.0, 0.0, -6.0));
    for (x, z, hp) in [
        (0.0, -2.2, 60),
        (1.8, -3.5, 60),
        (-2.6, -1.0, 40),
        (3.0, 1.5, 120),
        (-1.0, -9.0, 60),
    ] {
        mob_with(&mut w, vec3(x, 0.0, z), hp);
    }
    for e in &mut w.enemies {
        e.state = EState::Recover { t: -1000.0 };
    }
    w
}

/// 스킬 `id`를 쓸 수 있는 것을 든다(정한 차례 — 같은 스킬은 늘 같은 것): 검 → 검과 방패 → 무기 표의 차례.
pub(super) fn arm_for(w: &mut World, id: crate::skills::Id) {
    let def = id.def();
    if def.fits(w.grip()) {
        return;
    }
    w.set_shield(Some(crate::shield::Id::Pilgrim));
    if def.fits(w.grip()) {
        return;
    }
    w.set_shield(None);
    for arm in weapons::IDS {
        w.set_weapon(arm);
        if def.fits(w.grip()) {
            return;
        }
    }
    panic!("{id:?} fits nothing");
}

/// 그 궁극기를 가진 종류의 첫 무기(종류의 것이 아닌 궁극기는 검).
pub(super) fn arm_for_ult(w: &mut World, u: Ult) {
    let arm = weapons::IDS
        .into_iter()
        .find(|i| i.class().is_some_and(|c| c.def().great == u))
        .unwrap_or(weapons::Id::Sword);
    w.set_weapon(arm);
}

/// 첫 틱의 누름 `first`, 그 뒤 `held`가 참인 틱 동안 `hold`, 나머지는 빈 입력으로 `TICKS`틱 — 접은 해시.
/// `went` = 그 경우가 정말 나갔다는 사건(스킬의 시동 · 궁극기 · RB) — 한 번도 안 보이면 마당이 잘못 선 것이다.
fn trace(
    mut w: World,
    first: Actions,
    hold: Actions,
    held: usize,
    went: &dyn Fn(&Event) -> bool,
) -> u64 {
    let mut seen = false;
    let t = Tuning::default();
    let mut h = Fnv::new();
    for k in 0..TICKS {
        let a = if k == 0 {
            first
        } else if k < held {
            hold
        } else {
            Actions::default()
        };
        for e in w.step(&a, &t) {
            seen |= went(&e);
            h.put(&quantized(&format!("{e:?}")));
        }
        let p = &w.player;
        h.put(&format!(
            "p {} {} {} {}",
            q(p.pos.x),
            q(p.pos.y),
            q(p.pos.z),
            p.hp
        ));
        for e in &w.enemies {
            h.put(&format!(
                "e {} {} {} {} {}",
                e.hp,
                q(e.stance.poise),
                q(e.pos.x),
                q(e.pos.y),
                q(e.pos.z)
            ));
        }
    }
    assert!(seen, "the case never went off");
    h.0
}

/// (이름, 해시) — 모든 경우.
fn cases() -> Vec<(String, u64)> {
    let mut out = Vec::new();
    // 스킬: 최대 레벨 · 고르기 ㄱ/ㄴ.
    for id in IDS {
        for pick in [Pick::A, Pick::B] {
            let mut w = yard();
            arm_for(&mut w, id);
            let other = if id == crate::skills::Id::Whirl {
                crate::skills::Id::Cut
            } else {
                crate::skills::Id::Whirl
            };
            w.set_skills(Loadout {
                slots: [
                    Slot {
                        id,
                        rank: MAX_RANK,
                        pick,
                    },
                    Slot {
                        id: other,
                        rank: 1,
                        pick: Pick::A,
                    },
                ],
                ult: 1,
                great: Ult::Upend,
            });
            let press = Actions {
                skill1: true,
                ..Default::default()
            };
            let name = format!("skill.{id:?}.{pick:?}");
            out.push((
                name,
                trace(w, press, Actions::default(), 0, &|e| {
                    *e == Event::SkillStart { id }
                }),
            ));
        }
    }
    // 궁극기: 최대 레벨, 그 종류의 무기로.
    for u in ULTS {
        let mut w = yard();
        arm_for_ult(&mut w, u);
        w.set_skills(Loadout {
            slots: [crate::skills::Id::Whirl, crate::skills::Id::Cut].map(|id| Slot {
                id,
                rank: 1,
                pick: Pick::A,
            }),
            ult: ULT_MAX_RANK,
            great: u,
        });
        w.player.skill.gauge = 1.0;
        let press = Actions {
            ult: true,
            ..Default::default()
        };
        out.push((
            format!("ult.{u:?}"),
            trace(w, press, Actions::default(), 0, &|e| *e == Event::UltStart),
        ));
    }
    // RB: 무기마다 톡(한 번) · 쥐었다 떼기(1.5초 — 모으는 것 · 당기는 것은 다 모인다), 그리고 방패 던지기.
    let tap = Actions {
        cast: true,
        cast_held: true,
        ..Default::default()
    };
    let held = Actions {
        cast_held: true,
        ..Default::default()
    };
    for arm in weapons::IDS {
        for (how, hold_ticks) in [("tap", 1), ("hold", 90)] {
            let mut w = yard();
            w.set_weapon(arm);
            out.push((
                format!("rb.{arm:?}.{how}"),
                trace(w, tap, held, hold_ticks, &|e| {
                    matches!(e, Event::Cast { .. })
                }),
            ));
        }
    }
    let mut w = yard();
    w.set_weapon(weapons::Id::Sword);
    w.set_shield(Some(crate::shield::Id::Pilgrim));
    out.push((
        "rb.shield.tap".into(),
        trace(w, tap, held, 1, &|e| matches!(e, Event::Cast { .. })),
    ));
    out
}

/// 0278 R12: 골든 트레이스. 다르면 어느 경우인지 말한다. `GOLDEN_WRITE=1`이면 파일을 다시 쓴다.
#[test]
fn golden_traces() {
    let got = cases();
    let text: String = got.iter().map(|(n, h)| format!("{n} {h:016x}\n")).collect();
    if std::env::var("GOLDEN_WRITE").is_ok_and(|v| v == "1") {
        std::fs::write(FILE, &text).expect("write golden.txt");
        eprintln!("golden: wrote {} cases to {FILE}", got.len());
        return;
    }
    let want: std::collections::BTreeMap<&str, &str> =
        EXPECTED.lines().filter_map(|l| l.split_once(' ')).collect();
    let mut bad = Vec::new();
    for (name, h) in &got {
        let h = format!("{h:016x}");
        match want.get(name.as_str()) {
            Some(w) if *w == h => {}
            Some(w) => bad.push(format!("{name}: {w} → {h}")),
            None => bad.push(format!("{name}: missing in golden.txt (got {h})")),
        }
    }
    for name in want.keys() {
        if !got.iter().any(|(n, _)| n == name) {
            bad.push(format!("{name}: in golden.txt but no longer a case"));
        }
    }
    assert!(
        bad.is_empty(),
        "golden traces changed ({} of {}) — if on purpose, GOLDEN_WRITE=1 cargo test golden_traces:\n{}",
        bad.len(),
        got.len(),
        bad.join("\n")
    );
}

/// 반올림이 수만 고치고 이름 · id는 두는가.
#[test]
fn golden_quantizes_numbers_only() {
    assert_eq!(
        quantized("Skill { id: Whirl, at: Vec3(0.12345, -0.0000001, 2.0), n: 3 }"),
        "Skill { id: Whirl, at: Vec3(123, 0, 2000), n: 3 }"
    );
    assert_eq!(
        quantized("Cast { at: Vec3(1e-7, 1.5, 0.0), look: 3 }"),
        "Cast { at: Vec3(0, 1500, 0), look: 3 }"
    );
}
