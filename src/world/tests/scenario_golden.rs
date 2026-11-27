//! 둘째 골든 트레이스(0278 R20 준비 — 적의 틱까지 덮는 안전망): 첫 골든(`golden.rs`)의 마당은 적이 서 있어서 패링 · 맞음 ·
//! 쓰러뜨림 같은 때가 없다 — 고르기 ㄱ/ㄴ이 그런 때에만 갈리는 스킬 일곱(되울림 · 버팀 · 재기 · 저주 새기기 · 방패 돌진 · 자루 치기 ·
//! 밀치기)은 거기서 해시가 같았다. 여기는 둘:
//!
//! - **시나리오**: 개발 시나리오(`scenario::LIST`)마다 정한 누름 차례(걷기 · 평타 · 쥐었다 떼기 · 회피 · 패링 · 스킬 둘 · RB · 궁극기 ·
//!   락온 · 땅 돌리기)로 15초 — 적이 깨어나 치고, 받아쳐지고, 쓰러진다. 검과 망치 두 판.
//! - **겨눈 경우**: 첫 마당이 못 보는 때 — 일곱 스킬마다 ㄱ/ㄴ이 갈리는 판(그리고 둘의 해시가 **다르다**는 것도 본다), 쓰러뜨림의
//!   일(쿨타임 돌려받기 · 생기 거두기의 바닥 · 옮는 저주 · 계약의 표), 맞는 동안의 재기.
//!
//! 접는 것은 첫 골든과 같은 꼴(사건 Debug를 1e-3으로 · FNV-1a 64)에 방랑자의 기력 · 게이지 · 스킬 쿨타임과 적의 표지(더 다침 ·
//! 묶임 · 저주)를 더한다. 값은 `scenario_golden.txt`, 일부러 동작을 바꿨으면
//!
//! ```sh
//! GOLDEN_WRITE=1 cargo test scenario_traces
//! ```
#![cfg(feature = "dev")]
use super::golden::{q, quantized, Fnv};
use super::*;
use crate::det::Det;
use crate::scenario;
use crate::skills::{Id as Sk, Loadout, Pick, Slot, Ult, MAX_RANK, ULT_MAX_RANK};

const FILE: &str = "src/world/tests/scenario_golden.txt";
const EXPECTED: &str = include_str!("scenario_golden.txt");
/// 시나리오 한 판(틱) — 15초.
const SCENARIO_TICKS: usize = 900;
/// 겨눈 경우 한 판 — 6초.
const CASE_TICKS: usize = 360;

/// 틱마다 접는 것: 사건 + 방랑자 + 적.
fn fold(h: &mut Fnv, w: &World, ev: &[Event]) {
    for e in ev {
        h.put(&quantized(&format!("{e:?}")));
    }
    let p = &w.player;
    let cds: f32 = p.skill.cd.iter().sum();
    h.put(&format!(
        "p {} {} {} {} {} {} {}",
        q(p.pos.x),
        q(p.pos.y),
        q(p.pos.z),
        p.hp,
        q(p.stamina),
        q(p.skill.gauge),
        q(cds)
    ));
    for e in &w.enemies {
        h.put(&format!(
            "e {} {} {} {} {} {} {} {}",
            e.hp,
            q(e.stance.poise),
            q(e.pos.x),
            q(e.pos.y),
            q(e.pos.z),
            q(e.marks.frail),
            q(e.marks.bind),
            q(e.marks.hex)
        ));
    }
}

/// `ticks`틱 — 틱마다 `act(k, &mut w)`가 (필요하면 월드를 고쳐 두고) 누름을 준다. (해시, 본 사건 모두).
fn run(
    mut w: World,
    ticks: usize,
    mut act: impl FnMut(usize, &mut World) -> Actions,
) -> (u64, Vec<Event>) {
    let t = Tuning::default();
    let mut h = Fnv::new();
    let mut all = Vec::new();
    for k in 0..ticks {
        let a = act(k, &mut w);
        let ev = w.step(&a, &t);
        fold(&mut h, &w, &ev);
        all.extend(ev);
    }
    (h.0, all)
}

/// 시나리오의 누름 차례 — 상태를 보지 않는 정한 차례(같은 판 = 같은 누름).
fn script(k: usize) -> Actions {
    let a = k as f32 / 50.0;
    Actions {
        move_dir: if k % 240 < 160 {
            vec2(a.sin_d(), a.cos_d())
        } else {
            Vec2::ZERO
        },
        attack: k.is_multiple_of(11),
        attack_held: (k / 70) % 4 == 3,
        dodge: k % 97 == 45,
        parry: k % 17 == 8,
        jump: k % 130 == 100,
        cast: k % 53 == 20,
        cast_held: k % 53 == 20,
        skill1: k % 150 == 60,
        skill2: k % 150 == 120,
        ult: k == 400,
        lock: k == 30 || k == 600,
        turn_right: k % 300 == 10,
        ..Default::default()
    }
}

fn loadout(a: Sk, b: Sk) -> Loadout {
    Loadout {
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
        great: Ult::Upend,
    }
}

/// 시나리오마다 두 판(검 — 휘돌이 · 가로지르기, 망치 — 땅울림 · 끌어모으기). 모두를 합쳐 패링 · 맞음 · 쓰러뜨림이 다 있어야 한다.
fn scenario_cases(out: &mut Vec<(String, u64)>) {
    let mut seen = Vec::new();
    for e in scenario::LIST.iter() {
        for (arm, a, b) in [
            (weapons::Id::Sword, Sk::Whirl, Sk::Cut),
            (weapons::Id::Hammer, Sk::Quake, Sk::Gather),
        ] {
            let mut w = scenario::build(e.scenario);
            w.set_weapon(arm);
            w.set_skills(loadout(a, b));
            let (h, ev) = run(w, SCENARIO_TICKS, |k, w| {
                if k == 0 {
                    w.player.skill.gauge = 1.0;
                }
                script(k)
            });
            seen.extend(ev);
            out.push((format!("scenario.{:?}.{arm:?}", e.scenario), h));
        }
    }
    for (what, f) in [
        (
            "a parry",
            (|e: &Event| matches!(e, Event::Parry { .. })) as fn(&Event) -> bool,
        ),
        ("a hurt", |e| *e == Event::PlayerHurt),
        ("a kill", |e| matches!(e, Event::EnemyDown { .. })),
        ("a skill", |e| matches!(e, Event::SkillStart { .. })),
        ("an ult", |e| *e == Event::UltStart),
    ] {
        assert!(seen.iter().any(f), "the scenarios never saw {what}");
    }
}

/// 겨눈 경우의 마당: 30 m · 방랑자는 가운데에서 -z를 본다 · 적은 처음에 서 있다.
fn yard(mobs: &[(f32, f32, i32)]) -> World {
    let mut w = World::new("test", 30.0);
    w.player.facing = std::f32::consts::PI;
    for &(x, z, hp) in mobs {
        mob_with(&mut w, vec3(x, 0.0, z), hp);
    }
    idle(&mut w);
    w
}

/// 스킬 `id`를 칸 0에 최대 레벨 · `pick`으로(칸 1은 휘돌이/가로지르기 1레벨) — 맞는 것을 든다(첫 골든과 같은 차례).
fn wear(w: &mut World, id: Sk, pick: Pick) {
    let def = id.def();
    if !def.fits(w.grip()) {
        w.set_shield(Some(crate::shield::Id::Pilgrim));
        if !def.fits(w.grip()) {
            w.set_shield(None);
            let arm = weapons::IDS
                .into_iter()
                .find(|&arm| {
                    w.set_weapon(arm);
                    def.fits(w.grip())
                })
                .unwrap_or_else(|| panic!("{id:?} fits nothing"));
            w.set_weapon(arm);
        }
    }
    let other = if id == Sk::Whirl { Sk::Cut } else { Sk::Whirl };
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
}

/// 적 `i`가 이 틱에 이쪽(+z 쪽 방랑자)으로 내려치기를 시작하게 한다.
fn swing_now(w: &mut World, i: usize) {
    let en = &mut w.enemies[i];
    let to = w.player.pos - en.pos;
    en.facing = to.x.atan2_d(to.z);
    en.atk = Atk::Smash;
    en.state = EState::Tele { t: en.tele_dur() };
}

/// 적 `i`가 이 틱에 이쪽으로 내려치기를 준비하기 시작한다(예고의 처음부터).
fn wind_up(w: &mut World, i: usize) {
    swing_now(w, i);
    w.enemies[i].state = EState::Tele { t: 0.0 };
}

fn press_skill() -> Actions {
    Actions {
        skill1: true,
        ..Default::default()
    }
}

/// 한 경우: 마당을 세우고(`setup`), 첫 틱에 스킬 칸 0, 그 뒤 `act`. 스킬이 정말 나갔는지 본다.
fn skill_case(
    id: Sk,
    pick: Pick,
    setup: &dyn Fn(&mut World),
    act: &dyn Fn(usize, &mut World) -> Actions,
) -> (u64, Vec<Event>) {
    let mut w = yard(&[]);
    w.enemies.clear();
    setup(&mut w);
    wear(&mut w, id, pick);
    let (h, ev) = run(w, CASE_TICKS, |k, w| {
        if k == 0 {
            press_skill()
        } else {
            act(k, w)
        }
    });
    assert!(
        ev.contains(&Event::SkillStart { id }),
        "{id:?} {pick:?} never went off"
    );
    (h, ev)
}

fn nothing(_: usize, _: &mut World) -> Actions {
    Actions::default()
}

/// 겨눈 경우들(이름, 해시). `split` = ㄱ/ㄴ의 해시가 달라야 하는 스킬.
fn aimed_cases(out: &mut Vec<(String, u64)>) {
    type Setup = Box<dyn Fn(&mut World)>;
    type Act = Box<dyn Fn(usize, &mut World) -> Actions>;
    let mobs = |list: Vec<(f32, f32, i32)>| -> Setup {
        Box::new(move |w: &mut World| {
            for &(x, z, hp) in &list {
                mob_with(w, vec3(x, 0.0, z), hp);
            }
            idle(w);
        })
    };
    type Must = fn(&Event) -> bool;
    let cases: Vec<(&str, Sk, Setup, Act, Must)> = vec![
        // 되울림: 울림을 두른 뒤 앞의 잔재가 내려치고 받아친다 — 둘레의 둘이 울림을 받는다(ㄱ 자세 · ㄴ 되울림 자신의 쿨타임이 당겨진다).
        (
            "echo.parry",
            Sk::Echo,
            mobs(vec![(0.0, -1.2, 90), (2.5, 0.5, 90), (-2.0, 1.5, 90)]),
            Box::new(|k, w| {
                if k == 40 {
                    swing_now(w, 0);
                    w.player.facing = std::f32::consts::PI;
                    return Actions {
                        parry: true,
                        ..Default::default()
                    };
                }
                Actions::default()
            }),
            |e| matches!(e, Event::EchoRing { .. }),
        ),
        // 버팀: 버티는 동안 걷는다(ㄱ 걷기 · ㄴ 되쏘기는 서 있다), 앞의 둘이 친다 — 받아낸 만큼 터뜨림.
        (
            "brace.walk_and_hit",
            Sk::Brace,
            mobs(vec![(0.0, -1.3, 90), (1.4, -0.4, 90)]),
            Box::new(|k, w| {
                if k == 25 {
                    swing_now(w, 0);
                    swing_now(w, 1);
                }
                Actions {
                    move_dir: vec2(0.6, 0.8),
                    ..Default::default()
                }
            }),
            |e| matches!(e, Event::Skill { id: Sk::Brace, .. }),
        ),
        // 재기: 바닥(체력 · 기력 낮음)에서 쓰고, 후딜 동안 맞는다(ㄱ 고른 숨 = 기력 · ㄴ 바닥에서 = 곱절 체력).
        (
            "rally.low_and_hurt",
            Sk::Rally,
            Box::new(|w: &mut World| {
                mob_with(w, vec3(0.0, 0.0, -1.3), 90);
                mob_with(w, vec3(1.6, 0.0, 0.6), 90);
                idle(w);
                w.player.hp = w.player.max_hp / 5;
                w.player.stamina = 10.0;
            }),
            Box::new(|k, w| {
                if k == 20 {
                    swing_now(w, 0);
                }
                Actions::default()
            }),
            |e| *e == Event::PlayerHurt,
        ),
        // 저주 새기기: 표를 새긴 적을 쓰러뜨린다 — 둘레의 둘에게 옮는다(ㄱ 크게 = 더 멀리 · ㄴ 두 갈래 = 둘).
        (
            "hex.kill_spreads",
            Sk::Hex,
            mobs(vec![
                (0.0, -1.4, 90),
                (1.2, -2.6, 90),
                (-1.4, -2.8, 90),
                (0.0, -4.6, 90),
            ]),
            Box::new(|k, w| {
                if k == 60 {
                    w.enemies[0].hp = 1;
                }
                Actions {
                    attack: k == 61,
                    ..Default::default()
                }
            }),
            |e| matches!(e, Event::HexJump { .. }),
        ),
        // 방패 돌진: 기둥 앞의 적을 밀어 박고(ㄱ), 지나는 동안 곁의 적이 친다(ㄴ 스치는 몸 = 안 맞는다).
        (
            "ram.wall_and_swing",
            Sk::Ram,
            Box::new(|w: &mut World| {
                mob_with(w, vec3(0.0, 0.0, -1.5), 300);
                mob_with(w, vec3(1.3, 0.0, -2.5), 300);
                idle(w);
                w.obstacles.push(Aabb {
                    min: vec3(-1.5, 0.0, -4.2),
                    max: vec3(1.5, 3.0, -3.4),
                });
            }),
            Box::new(|k, w| {
                if k == 1 {
                    swing_now(w, 1);
                }
                Actions::default()
            }),
            |e| matches!(e, Event::Hit { .. }),
        ),
        // 자루 치기: 앞의 잔재가 공격을 준비하는 중(끊긴다 — ㄴ 드러난 틈), 옆 55°의 적(ㄱ 넓게 = 맞는다).
        (
            "pommel.winding_and_side",
            Sk::Pommel,
            mobs(vec![(0.0, -1.3, 300), (1.5, -1.05, 300)]),
            Box::new(|k, w| {
                if k == 1 {
                    wind_up(w, 0);
                }
                Actions::default()
            }),
            |e| matches!(e, Event::Interrupt { .. }),
        ),
        // 손바닥 치기(0286 몽크): 앞의 잔재가 공격을 준비하는 중(끊긴다 — ㄴ 드러난 틈), 옆 50°의 적(ㄱ 넓은 손 = 맞는다).
        (
            "palm.winding_and_side",
            Sk::Palm,
            mobs(vec![(0.0, -1.3, 300), (1.2, -1.0, 300)]),
            Box::new(|k, w| {
                if k == 1 {
                    wind_up(w, 0);
                }
                Actions::default()
            }),
            |e| matches!(e, Event::Interrupt { .. }),
        ),
        // 밀치기: 기둥 앞의 적(ㄱ 벽에 박기)과 옆 95°의 적(ㄴ 넓게).
        (
            "shove.wall_and_side",
            Sk::Shove,
            Box::new(|w: &mut World| {
                mob_with(w, vec3(0.0, 0.0, -1.4), 300);
                mob_with(w, vec3(1.6, 0.0, 0.15), 300);
                idle(w);
                w.obstacles.push(Aabb {
                    min: vec3(-1.5, 0.0, -3.0),
                    max: vec3(1.5, 3.0, -2.3),
                });
            }),
            Box::new(nothing),
            |e| matches!(e, Event::Hit { .. }),
        ),
        // 쿨타임 돌려받기(맺음): 가로지르기 · 빠지며 베기 · 꿰뚫기로 약한 적을 쓰러뜨린다.
        (
            "cut.refund_kill",
            Sk::Cut,
            mobs(vec![(0.0, -1.5, 5), (0.3, -3.5, 5)]),
            Box::new(nothing),
            |e| matches!(e, Event::EnemyDown { .. }),
        ),
        (
            "slip.refund_kill",
            Sk::Slip,
            mobs(vec![(0.0, -1.3, 5), (0.5, -1.6, 300)]),
            Box::new(nothing),
            |e| matches!(e, Event::EnemyDown { .. }),
        ),
        (
            "skewer.refund_kill",
            Sk::Skewer,
            mobs(vec![(0.0, -1.5, 5), (0.2, -3.0, 5)]),
            Box::new(nothing),
            |e| matches!(e, Event::EnemyDown { .. }),
        ),
        // 생기 거두기: 바닥(체력 낮음)에서 둘레를 거둔다 — 하나는 쓰러진다(ㄴ 바닥에서 = 곱절).
        (
            "harvest.low_kill",
            Sk::Harvest,
            Box::new(|w: &mut World| {
                mob_with(w, vec3(0.0, 0.0, -1.3), 5);
                mob_with(w, vec3(1.4, 0.0, 0.5), 300);
                mob_with(w, vec3(-1.2, 0.0, 0.9), 300);
                idle(w);
                w.player.hp = w.player.max_hp / 5;
            }),
            Box::new(nothing),
            |e| matches!(e, Event::EnemyDown { .. }),
        ),
    ];
    for (name, id, setup, act, must) in &cases {
        let (a, ea) = skill_case(*id, Pick::A, setup.as_ref(), act.as_ref());
        let (b, eb) = skill_case(*id, Pick::B, setup.as_ref(), act.as_ref());
        assert!(
            ea.iter().any(must) && eb.iter().any(must),
            "{name}: the moment this case is for never came"
        );
        // 일곱(첫 골든에서 ㄱ/ㄴ이 같던 것)은 여기서 갈려야 한다 — 같으면 이 판이 그 고르기를 못 본다.
        if [
            Sk::Echo,
            Sk::Brace,
            Sk::Rally,
            Sk::Hex,
            Sk::Ram,
            Sk::Pommel,
            Sk::Shove,
            Sk::Palm,
        ]
        .contains(id)
        {
            assert_ne!(a, b, "{name}: picks A and B hash the same");
        }
        out.push((format!("case.{name}.A"), a));
        out.push((format!("case.{name}.B"), b));
    }
    // 계약의 표(완드의 빈 손): 저주탄을 맞힌 적을 쓰러뜨리면 체력 · RB 칸이 돌아온다.
    let mut w = yard(&[(0.0, -3.0, 90), (1.0, -6.0, 90)]);
    w.set_weapon(weapons::Id::Wand);
    w.set_open_hand(true);
    w.player.hp = w.player.max_hp / 2;
    let (h, ev) = run(w, CASE_TICKS, |k, w| {
        if k == 50 {
            w.enemies[0].hp = 1;
        }
        Actions {
            cast: k == 0 || k == 51,
            cast_held: k == 0 || k == 51,
            ..Default::default()
        }
    });
    assert!(
        ev.iter().any(|e| matches!(e, Event::EnemyDown { .. })),
        "pact: the marked one fell"
    );
    out.push(("case.pact.kill".into(), h));
}

fn all_cases() -> Vec<(String, u64)> {
    let mut out = Vec::new();
    scenario_cases(&mut out);
    aimed_cases(&mut out);
    out
}

/// 0278 R20 준비: 둘째 골든 트레이스. 다르면 어느 경우인지 말한다. `GOLDEN_WRITE=1`이면 파일을 다시 쓴다.
#[test]
fn scenario_traces() {
    let got = all_cases();
    let text: String = got.iter().map(|(n, h)| format!("{n} {h:016x}\n")).collect();
    if std::env::var("GOLDEN_WRITE").is_ok_and(|v| v == "1") {
        std::fs::write(FILE, &text).expect("write scenario_golden.txt");
        eprintln!("scenario golden: wrote {} cases to {FILE}", got.len());
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
            None => bad.push(format!("{name}: missing in scenario_golden.txt (got {h})")),
        }
    }
    for name in want.keys() {
        if !got.iter().any(|(n, _)| n == name) {
            bad.push(format!(
                "{name}: in scenario_golden.txt but no longer a case"
            ));
        }
    }
    assert!(
        bad.is_empty(),
        "scenario traces changed ({} of {}) — if on purpose, GOLDEN_WRITE=1 cargo test scenario_traces:\n{}",
        bad.len(),
        got.len(),
        bad.join("\n")
    );
}
