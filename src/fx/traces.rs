//! 이펙트 트레이스(0278 R25 준비 — `fx`의 안전망): `fx.rs`에는 낸 그림을 잡는 고정값이 없었다. 여기서
//!
//! - **사건**: 시험의 사건 표본(`Event::varied` — 소리 표와 같은 것)마다 새 `Fx`(씨앗 0 — 이펙트의 난수는 `Fx::seed` 하나뿐이고
//!   벽시계를 읽지 않는다)에 넣고 45틱 흘린다(앱의 차례 `tick` → `settle`). 틱마다의 입자(꼴 · 자리 · 속도 · 나이 · 수명 · 크기 ·
//!   빛 · 당김 · 꾸밈) · 남는 빛 · 늦게 나올 입자를 1e-3으로 반올림해 FNV-1a 64로 접는다. 바탕 월드(`empty`) 하나는 사건마다 한 줄,
//!   이펙트가 월드에서 읽는 것(몰아치기 · 버팀의 수 · 뜬 높이 · 끼운 궁극기 · 든 무기 · 스킬의 레벨과 고르기)을 바꾼 월드는 **바탕과
//!   다른 것만** 적는다(줄이 사라져도 시험이 잡는다).
//! - **지켜보기**(`Fx::watch` — 사건 없는 몸짓: 회피 · 뛰기 · 착지 · 달리기 · 몰아치기와 분노의 기운 · 적의 돌진 · 퍼지는 고리):
//!   개발 시나리오마다 무기 셋으로 정한 누름 차례 10초, 앱과 같은 차례(`step` → `on_events` → `watch` → `tick` → `settle`).
//! - 앱이 바로 부르는 셋(`kindle` · `loot_drop` · `loot_take`).
//!
//! 값은 `fx_traces.txt`, 일부러 그림을 바꿨으면
//!
//! ```sh
//! FX_WRITE=1 cargo test fx_traces
//! ```
use super::*;
use crate::tuning::{Tuning, TICK_DT};

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/fx/fx_traces.txt");
/// 사건 하나를 흘리는 틱(가장 긴 입자도 1.5초 안 — 그 반쯤에서 늦은 불똥 · 남는 빛까지 본다).
const EVENT_TICKS: usize = 45;
/// 지켜보기 한 판(틱) — 10초.
const WATCH_TICKS: usize = 600;

struct Fnv(u64);

impl Fnv {
    fn new() -> Self {
        Fnv(0xcbf2_9ce4_8422_2325)
    }
    fn put(&mut self, s: &str) {
        for b in s.bytes().chain(std::iter::once(0xff)) {
            self.0 ^= b as u64;
            self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
}

fn q(x: f32) -> i64 {
    let v = (x as f64 * 1000.0).round() as i64;
    if v == 0 {
        0
    } else {
        v
    }
}

fn qv(v: Vec3) -> String {
    format!("{},{},{}", q(v.x), q(v.y), q(v.z))
}

fn qc(c: Color) -> String {
    format!("{},{},{},{}", q(c.r), q(c.g), q(c.b), q(c.a))
}

fn part(p: &Part) -> String {
    format!(
        "{:?} {} {} {} {} {} {} {} {} {} {} {}",
        p.shape,
        qv(p.pos),
        qv(p.vel),
        q(p.age),
        q(p.life),
        q(p.size.0),
        q(p.size.1),
        qc(p.color.0),
        qc(p.color.1),
        q(p.gravity),
        q(p.drag),
        p.decor
    )
}

fn kind(k: &MarkKind) -> String {
    match *k {
        MarkKind::Ring { r0, r1 } => format!("ring {} {}", q(r0), q(r1)),
        MarkKind::Streak { dir, len, half } => {
            format!("streak {} {} {}", qv(dir), q(len), q(half))
        }
        MarkKind::Shock {
            dir,
            r0,
            r1,
            arc_deg,
        } => format!("shock {} {} {} {}", qv(dir), q(r0), q(r1), q(arc_deg)),
        MarkKind::Cracks { dir, r, n, arc_deg } => {
            format!("cracks {} {} {n} {}", qv(dir), q(r), q(arc_deg))
        }
        MarkKind::Converge { r, n } => format!("converge {} {n}", q(r)),
        MarkKind::Slash { dir, len, half } => {
            format!("slash {} {} {}", qv(dir), q(len), q(half))
        }
    }
}

/// 이펙트의 지금(입자 · 남는 빛 · 늦게 나올 입자)을 접는다.
fn fold(h: &mut Fnv, fx: &Fx) {
    h.put(&format!(
        "n {} {} {}",
        fx.parts.len(),
        fx.marks.len(),
        fx.later.len()
    ));
    for p in &fx.parts {
        h.put(&part(p));
    }
    for m in &fx.marks {
        h.put(&format!(
            "m {} {} {} {} {} {} {} {}",
            qv(m.pos),
            kind(&m.kind),
            q(m.age),
            q(m.life),
            q(m.delay),
            qc(m.color),
            m.follow,
            m.hold
        ));
    }
    for (left, p) in &fx.later {
        h.put(&format!("l {} {}", q(*left), part(p)));
    }
}

/// 사건 하나를 바탕의 `Fx`에 넣고 흘린 해시.
fn event_hash(w: &World, e: &Event) -> u64 {
    let mut fx = Fx::default();
    let mut h = Fnv::new();
    fx.on_events(std::slice::from_ref(e), w);
    fold(&mut h, &fx);
    for _ in 0..EVENT_TICKS {
        fx.tick(TICK_DT);
        fx.settle(|p| w.ground_y(p));
        fold(&mut h, &fx);
    }
    h.0
}

fn base() -> World {
    World::new("empty", 16.0)
}

/// 이펙트가 월드에서 읽는 것을 바꾼 월드들(이름, 고친 월드).
fn contexts() -> Vec<(String, World)> {
    let mut out = Vec::new();
    let mut w = base();
    w.player.skill.flurry_left = 2.0;
    out.push(("flurry".to_string(), w));
    let mut w = base();
    w.player.skill.brace_soaked = 3;
    out.push(("brace3".to_string(), w));
    let mut w = base();
    w.player.air = 1.5;
    out.push(("air".to_string(), w));
    for u in crate::skills::ULTS {
        let mut w = base();
        w.player.skill.ult = u;
        out.push((format!("ult.{u:?}"), w));
    }
    for arm in crate::weapons::IDS {
        let mut w = base();
        w.set_weapon(arm);
        out.push((format!("weapon.{arm:?}"), w));
    }
    out
}

/// 사건 표본마다: 바탕 한 줄 + 바탕과 다른 월드만.
fn event_cases(out: &mut Vec<(String, u64)>) {
    let events = Event::varied();
    let w0 = base();
    let base_hash: Vec<u64> = events.iter().map(|e| event_hash(&w0, e)).collect();
    for (e, h) in events.iter().zip(&base_hash) {
        out.push((format!("event {e:?}"), *h));
    }
    for (name, w) in contexts() {
        for (e, h0) in events.iter().zip(&base_hash) {
            let h = event_hash(&w, e);
            if h != *h0 {
                out.push((format!("{name} {e:?}"), h));
            }
        }
    }
    // 스킬의 레벨 · 고르기(`World::skill_eff`): 칸 0에 최대 레벨로 끼운 월드에서 시동과 판정.
    let (at, dir) = (vec3(1.0, 0.0, -2.0), Vec3::Z);
    for id in crate::skills::IDS {
        for pick in [crate::skills::Pick::A, crate::skills::Pick::B] {
            let mut w = base();
            w.player.skill.worn[0] = id;
            w.player.skill.rank[0] = crate::skills::MAX_RANK;
            w.player.skill.pick[0] = pick;
            for e in [Event::SkillStart { id }, Event::Skill { id, at, dir }] {
                out.push((format!("max.{pick:?} {e:?}"), event_hash(&w, &e)));
            }
        }
    }
}

/// 지켜보기의 누름 차례 — 상태를 보지 않는 정한 차례: 걷기 · 달리기(회피를 쥔 채) · 회피 · 뛰기 · 평타 · 스킬 · 궁극기.
fn script(k: usize) -> crate::input::Actions {
    let a = k as f32 / 40.0;
    crate::input::Actions {
        move_dir: if k % 200 < 150 {
            vec2(a.sin(), a.cos())
        } else {
            Vec2::ZERO
        },
        attack: k.is_multiple_of(13),
        dodge: k % 89 == 30,
        dodge_held: (k / 60) % 5 == 2,
        parry: k % 19 == 7,
        jump: k % 110 == 70,
        cast: k % 61 == 25,
        skill1: k % 140 == 50,
        skill2: k % 140 == 110,
        ult: k == 300,
        ..Default::default()
    }
}

#[cfg(feature = "dev")]
fn watch_cases(out: &mut Vec<(String, u64)>) {
    use crate::skills::{Id as Sk, Loadout, Pick, Slot, Ult, MAX_RANK, ULT_MAX_RANK};
    let t = Tuning::default();
    for e in crate::scenario::LIST.iter() {
        for (arm, a, b, great) in [
            (crate::weapons::Id::Sword, Sk::Whirl, Sk::Cut, Ult::Flurry),
            (
                crate::weapons::Id::Greatsword,
                Sk::Brace,
                Sk::Gather,
                Ult::Upend,
            ),
            (
                crate::weapons::Id::Crossbow,
                Sk::Gather,
                Sk::Gather,
                Ult::Line,
            ),
        ] {
            let mut w = crate::scenario::build(e.scenario);
            w.set_weapon(arm);
            w.set_skills(Loadout {
                slots: [
                    Slot {
                        id: a,
                        rank: MAX_RANK,
                        pick: Pick::A,
                    },
                    Slot {
                        id: b,
                        rank: MAX_RANK,
                        pick: Pick::B,
                    },
                ],
                ult: ULT_MAX_RANK,
                great,
            });
            w.player.skill.gauge = 1.0;
            let mut fx = Fx::default();
            let mut h = Fnv::new();
            for k in 0..WATCH_TICKS {
                let ev = w.step(&script(k), &t);
                fx.on_events(&ev, &w);
                fx.watch(&w, TICK_DT);
                fx.tick(TICK_DT);
                fx.settle(|p| w.ground_y(p));
                fold(&mut h, &fx);
            }
            out.push((format!("watch {:?}.{arm:?}", e.scenario), h.0));
        }
    }
}

#[cfg(not(feature = "dev"))]
fn watch_cases(_: &mut Vec<(String, u64)>) {}

/// 앱이 바로 부르는 것(꺼진 화로에 불 · 떨어진 아이템의 번쩍임과 줍기).
fn direct_cases(out: &mut Vec<(String, u64)>) {
    let w = base();
    let at = vec3(1.0, 0.0, -2.0);
    let runs: [(&str, fn(&mut Fx, Vec3)); 5] = [
        ("kindle", |fx, at| fx.kindle(at)),
        ("loot_drop.0", |fx, at| fx.loot_drop(at, GOLD, 0)),
        ("loot_drop.1", |fx, at| fx.loot_drop(at, SKY, 1)),
        ("loot_drop.2", |fx, at| fx.loot_drop(at, MINT, 2)),
        ("loot_take", |fx, at| fx.loot_take(at, LAVENDER)),
    ];
    for (name, f) in runs {
        let mut fx = Fx::default();
        let mut h = Fnv::new();
        f(&mut fx, at);
        fold(&mut h, &fx);
        for _ in 0..EVENT_TICKS {
            fx.tick(TICK_DT);
            fx.settle(|p| w.ground_y(p));
            fold(&mut h, &fx);
        }
        out.push((format!("direct {name}"), h.0));
    }
}

fn all_cases() -> Vec<(String, u64)> {
    let mut out = Vec::new();
    event_cases(&mut out);
    watch_cases(&mut out);
    direct_cases(&mut out);
    out
}

/// 0278 R25 준비: 사건 · 지켜보기 · 바로 부르는 것마다 낸 이펙트(입자 · 남는 빛)가 적어 둔 값과 같다.
#[test]
fn fx_traces() {
    let now: String = all_cases()
        .iter()
        .map(|(name, h)| format!("{name} = {h:016x}\n"))
        .collect();
    if std::env::var("FX_WRITE").is_ok() {
        std::fs::write(FILE, &now).unwrap();
        return;
    }
    let was = std::fs::read_to_string(FILE).expect("fx_traces.txt");
    let diff: Vec<String> = was
        .lines()
        .zip(now.lines())
        .filter(|(a, b)| a != b)
        .map(|(a, b)| format!("was {a}\nnow {b}"))
        .take(12)
        .collect();
    assert!(
        diff.is_empty() && was.lines().count() == now.lines().count(),
        "{} lines was, {} now:\n{}",
        was.lines().count(),
        now.lines().count(),
        diff.join("\n")
    );
}

/// 같은 사건 = 같은 그림(세 번 돌려 같다 — 이펙트에는 씨앗 없는 난수도 벽시계도 없다).
#[test]
fn fx_traces_are_deterministic() {
    let w = base();
    let events = Event::varied();
    let once: Vec<u64> = events.iter().map(|e| event_hash(&w, e)).collect();
    for _ in 0..2 {
        let again: Vec<u64> = events.iter().map(|e| event_hash(&w, e)).collect();
        assert_eq!(once, again);
    }
}
