use super::*;
use crate::input::Actions;
use crate::tuning::Tuning;
use crate::tuning::{FLASK_CHARGES, PLAYER_HP};

/// 0149(사장 "보스가 깨어나기 전에 타겟팅이 안 돼서 엉뚱한 적에게 락온"): 깨어나는 2초 동안에도 락온은 보스를 잡는다 —
/// 더 가까이 깬 잡몹이 있어도. 아직 깨어나기 시작하지 않은 잠든 보스는 잡히지 않고(싸울 것이 아니다), 다 깨면 깃발은 내려간다.
#[test]
fn lock_on_takes_the_boss_while_it_is_still_waking() {
    let t = Tuning::default();
    let (mut run, mut w) = StageRun::start(STAGES[STAGES.len() - 1]);
    let b = run.boss.unwrap();
    let home = run.level.marks_of('W').next().expect("boss mark");
    let lock = Actions {
        lock: true,
        ..Default::default()
    };
    // 깨어나기 전: 곁에 서도 잠든 보스는 락온 후보가 아니다.
    assert!(!w.enemies[b].lockable());
    // 성소에 들어선다 → 깨어나기 시작.
    w.player.pos = home + vec3(0.0, 0.0, 4.0);
    w.player.prev = w.player.pos;
    run.tick(&mut w, 1.0 / 60.0);
    assert!(matches!(run.phase, Phase::Waking(_)), "{:?}", run.phase);
    assert!(w.enemies[b].waking && w.enemies[b].lockable() && !w.enemies[b].active());
    assert!(w.lock_candidate(), "the touch lock button shows too");
    // 더 가까운 깬 잡몹 하나(방랑자 옆 2m) — 그래도 보스가 잡힌다.
    let mob = w.spawn(
        crate::species::Kind::Husk,
        w.player.pos + vec3(2.0, 0.0, 0.0),
    );
    w.enemies[mob].state = EState::Chase;
    w.player.facing = (home - w.player.pos).x.atan2((home - w.player.pos).z);
    w.step(&lock, &t);
    assert_eq!(w.lock, Some(b), "locked {:?}, boss is {b}", w.lock);
    // 다 깨면 깃발이 내려가고 락온은 그대로다.
    for _ in 0..((WAKE_TIME * 60.0) as usize + 2) {
        w.player.hp = PLAYER_HP;
        w.step(&Actions::default(), &t);
        run.tick(&mut w, 1.0 / 60.0);
    }
    assert_eq!(run.phase, Phase::Duel);
    assert!(!w.enemies[b].waking && w.enemies[b].active());
    assert_eq!(w.lock, Some(b));
}

/// 0208: 작업층의 문은 화물 승강대(지름길) — 처음부터 문 앞 단계(문지기는 장치와 상관없이 깬다), 문은 닫혀 있다가 장치를
/// 당기면 열리고, 문지기가 깨어도 다시 닫히지 않는다. 레일 `|` 묶음마다 수레 하나.
#[test]
fn the_workings_lift_is_a_shortcut_and_the_rail_carries_a_cart() {
    let t = Tuning::default();
    let def = STAGES[3];
    assert!(def.shortcut);
    let (mut run, mut w) = StageRun::start(def);
    assert_eq!(run.phase, Phase::GateOpen);
    assert!(run.gate_closed, "the lift starts shut");
    // 레일 둘: 운반 회랑(늘 돈다) · 갱도 싸움터(문지기 2단계에만).
    assert_eq!(w.carts.len(), 2, "two rails, a cart each");
    assert_eq!(w.carts.iter().filter(|c| c.boss).count(), 1);
    let c = w.carts.iter().find(|c| !c.boss).unwrap();
    assert!((c.a - c.b).length() > 15.0, "a long rail");
    // 장치를 당긴다.
    let lever = run.lever.expect("lever");
    w.player.pos = lever;
    w.player.prev = lever;
    run.tick(&mut w, 1.0 / 60.0);
    assert!(run.near_lever);
    w.interact = true;
    run.tick(&mut w, 1.0 / 60.0);
    w.interact = false;
    assert!(!run.gate_closed && run.gate_opened && run.take_pulled());
    // 문지기 곁으로: 깨어도 승강대는 열린 채.
    let home = run.level.marks_of('W').next().unwrap();
    w.player.pos = home + vec3(0.0, 0.0, 4.0);
    w.player.prev = w.player.pos;
    run.tick(&mut w, 1.0 / 60.0);
    assert!(matches!(run.phase, Phase::Waking(_)), "{:?}", run.phase);
    assert!(!run.gate_closed, "the lift stays open");
    let _ = t;
}

/// 장치를 당기지 않아도 문지기는 깬다(승강대는 싸움터의 문이 아니다) — 승강대는 닫힌 채.
#[test]
fn the_workings_warden_wakes_without_the_lift() {
    let (mut run, mut w) = StageRun::start(STAGES[3]);
    let home = run.level.marks_of('W').next().unwrap();
    w.player.pos = home + vec3(0.0, 0.0, 4.0);
    w.player.prev = w.player.pos;
    run.tick(&mut w, 1.0 / 60.0);
    assert!(matches!(run.phase, Phase::Waking(_)), "{:?}", run.phase);
    assert!(run.gate_closed);
}

/// 0125 → 0185 → 0208: 종탑 터에 들어서면 깨진 종지기가 깨고, 이기고 핵을 거두면 바닥 길이 열린다. 정원의 이끼 덮인 문지기도
/// 이제 바닥 길을 연다(넷째 지역이 생겼다). 마지막 지역(작업층)의 문지기는 핵이 판의 끝이다 — 바닥 길이 없다. 작업층의 문은
/// 승강대(지름길)라 장치를 당기지 않아도 문지기가 깬다.
#[test]
fn the_village_bellkeeper_opens_the_way_down_and_the_last_warden_ends_the_run() {
    let t = Tuning::default();
    for (def, kind, last) in [
        (STAGES[1], Kind::Bellkeeper, false),
        (STAGES[2], Kind::Mossward, false),
        (STAGES[STAGES.len() - 1], Kind::Overseer, true),
    ] {
        let (run, w) = StageRun::start(def);
        assert_eq!(run.down_at.is_none(), last);
        warden_fight_to_core(run, w, kind, &t, last);
    }
}

fn warden_fight_to_core(mut run: StageRun, mut w: World, kind: Kind, t: &Tuning, last: bool) {
    let t = t.clone();
    let home = run.level.marks_of('W').next().expect("bellkeeper mark");
    w.player.pos = home + vec3(0.0, 0.0, 4.0);
    w.player.prev = w.player.pos;
    run.tick(&mut w, 1.0 / 60.0);
    assert!(matches!(run.phase, Phase::Waking(_)), "{:?}", run.phase);
    for _ in 0..((WAKE_TIME * 60.0) as usize + 2) {
        w.player.hp = PLAYER_HP;
        w.step(&Actions::default(), &t);
        run.tick(&mut w, 1.0 / 60.0);
    }
    assert_eq!(run.phase, Phase::Duel);
    let b = run.boss.unwrap();
    assert_eq!(w.enemies[b].kind, kind);
    w.enemies[b].state = EState::Dead;
    w.step(&Actions::default(), &t);
    run.tick(&mut w, 1.0 / 60.0);
    assert_eq!(run.phase, Phase::Core);
    w.player.pos = run.core.unwrap();
    let out = run.tick(&mut w, 1.0 / 60.0);
    if last {
        assert_eq!(out, StageOut::Cleared, "the run ends here");
    } else {
        assert_eq!(out, StageOut::CoreTaken, "the floor opens");
        assert_eq!(run.phase, Phase::Down);
    }
}

/// 0091: 높은 땅에 선 문은 상자도 그 땅 위에 — 가운데만 땅에 올리고 상자를 0에 두면 위 한 자락이 시야 가림 판정·막기 밖이었다.
#[test]
fn the_gate_box_stands_on_the_ground_under_the_gate() {
    let (run, _) = StageRun::start(&sanctuary::DEF);
    let (b, c) = run.level.gate.expect("sanctuary has a gate");
    assert!(
        c.y > 0.5,
        "the sanctuary gate stands on raised ground ({})",
        c.y
    );
    assert!(
        (b.min.y - c.y).abs() < 1e-4,
        "box floor {} vs ground {}",
        b.min.y,
        c.y
    );
    assert!((b.max.y - (c.y + 4.5)).abs() < 1e-4);
}

#[test]
fn the_sanctuary_map_parses_into_walls_props_lights_gate_and_marks() {
    let lv = Level::parse(sanctuary::DEF.map);
    assert!(lv.w > 10 && lv.h > 20);
    assert!(!lv.walls.is_empty() && !lv.props.is_empty() && !lv.lights.is_empty());
    assert!(lv.gate.is_some());
    assert!(lv.marks_of('W').count() == 1);
    // 줄마다 폭이 같다(글자 지도 실수 잡기).
    let rows: Vec<&str> = sanctuary::DEF
        .map
        .lines()
        .filter(|l| !l.trim().is_empty())
        .collect();
    assert!(
        rows.iter().all(|r| r.chars().count() == lv.w),
        "ragged map rows"
    );
    // 시작 자리는 바닥.
    let start = lv.player_start;
    assert!(lv.walls.iter().all(|wb| !(start.x > wb.min.x
        && start.x < wb.max.x
        && start.z > wb.min.z
        && start.z < wb.max.z)));
}

/// 0035: 배경 높이 격자는 지도와 같은 크기이고, 걸을 수 있는 칸·문·표지 칸에는 높이가 없다(월드는 평평한 땅 그대로).
#[test]
fn heights_match_the_map_and_every_ramp_joins_two_levels() {
    let g = sanctuary::DEF.heights.expect("sanctuary has heights");
    let map: Vec<&str> = sanctuary::DEF
        .map
        .lines()
        .filter(|l| !l.trim().is_empty())
        .collect();
    let hs: Vec<&str> = g.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(map.len(), hs.len(), "rows");
    for (z, (m, h)) in map.iter().zip(&hs).enumerate() {
        assert_eq!(m.chars().count(), h.chars().count(), "row {z} width");
        for (x, (mc, hc)) in m.chars().zip(h.chars()).enumerate() {
            assert!(
                hc == '.' || hc == '/' || hc.is_digit(36),
                "row {z} col {x}: {hc:?}"
            );
            if hc == '/' {
                assert!(mc != ' ' && mc != '#', "row {z} col {x}: ramp on {mc:?}");
            }
        }
    }
    let (run, _) = StageRun::start(&sanctuary::DEF);
    let ground = run.level.ground();
    for z in 0..run.level.h {
        for x in 0..run.level.w {
            if run.level.ramps[z * run.level.w + x] {
                assert!(
                    matches!(ground.at(x, z), Cell::Ramp { from, to, .. } if from != to),
                    "ramp at row {z} col {x} joins two different levels: {:?}",
                    ground.at(x, z)
                );
            }
        }
    }
    assert!(
        run.level.heights.iter().any(|h| h.is_some_and(|v| v > 0.0)),
        "some rise"
    );
    assert!(
        run.level.heights.iter().any(|h| h.is_some_and(|v| v < 0.0)),
        "some sink"
    );
}

/// 0043: 플레이어는 모든 바닥 칸에 갈 수 있고(걸음·경사로 + 0.5m 턱 점프 + 뛰어내리기), 모든 스폰·보스 표지에서
/// 적이 걸어서(경사로로만) 플레이어 시작 자리·문 앞까지 온다 — 높낮이가 길을 끊지 않았나.
#[test]
fn every_floor_is_reachable_and_every_spawn_can_walk_to_the_fight() {
    // 0118: 모든 지역.
    for def in STAGES {
        let (run, _) = StageRun::start(def);
        let lv = &run.level;
        let g = lv.ground();
        // 플레이어 규칙: 이웃 칸 가운데 높이로 — 오르기는 점프 높이(0.9m) 안의 0.5m까지, 내리기는 아무리 깊어도.
        let start = g.cell_of(lv.player_start).unwrap();
        let mut seen = vec![false; lv.w * lv.h];
        let mut q = vec![start];
        seen[start.1 * lv.w + start.0] = true;
        while let Some((x, z)) = q.pop() {
            let here = g.center(x, z).y;
            for (dx, dz) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (x as i32 + dx, z as i32 + dz);
                if nx < 0 || nz < 0 || nx as usize >= lv.w || nz as usize >= lv.h {
                    continue;
                }
                let (nx, nz) = (nx as usize, nz as usize);
                if seen[nz * lv.w + nx] || matches!(g.at(nx, nz), Cell::Solid) {
                    continue;
                }
                if g.center(nx, nz).y - here <= 0.5 + 1e-3 {
                    seen[nz * lv.w + nx] = true;
                    q.push((nx, nz));
                }
            }
        }
        for z in 0..lv.h {
            for x in 0..lv.w {
                if lv.walkable(x, z) {
                    assert!(seen[z * lv.w + x], "player cannot reach row {z} col {x}");
                }
            }
        }
        // 적 규칙: 길찾기 거리 지도(`Nav::flow`, 소품·상자가 덮은 칸은 막힘 — 0076)로 — 시작 자리와 문 앞(뜰 쪽) 어디서든.
        // 문이 없는 지역(0118 마을)은 시작 자리만.
        let gate = lv.gate.map(|g| g.1 + vec3(0.0, 0.0, CELL * 2.0));
        let nav = crate::nav::Nav::build(Some(&g), &lv.obstacles(), lv.half_extent());
        for goal in std::iter::once(lv.player_start).chain(gate) {
            let dist = nav.flow(goal);
            for (c, at) in &lv.marks {
                if c.is_ascii_lowercase() || c.is_ascii_uppercase() && *c != 'R' {
                    let (x, z) = g.cell_of(*at).unwrap();
                    assert!(
                        dist[z * lv.w + x] != u16::MAX,
                        "spawn {c} at row {z} col {x} cannot walk to {goal:?}"
                    );
                }
            }
        }
    }
}

/// 끝까지 걸어 본다(0113): 입구(서 있는 적, 문 닫힘) → 문 장치 당기기(문 열림·기록 알림 한 번) → 성소(문 닫힘·깨어남) → 결투 → 핵 → 클리어.
#[test]
fn the_stage_script_runs_from_entrance_to_clear() {
    let t = Tuning::default();
    let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
    let step = |run: &mut StageRun, w: &mut World| {
        w.step(&Actions::default(), &t);
        run.tick(w, 1.0 / 60.0)
    };
    assert_eq!(run.phase, Phase::Entrance);
    assert!(run.gate_closed);
    assert_eq!(run.goal(), None, "the way to the gate is to be found");
    // 0170: 바깥에서 얹은 갈 곳(튜토리얼의 쉼터)은 진행의 갈 곳이 없을 때만 — 문이 열리면 문이 먼저다.
    let hearth = run.rests[0];
    run.guide = Some(hearth);
    assert_eq!(run.goal(), Some(hearth));
    run.guide = None;
    assert!(run.objective.is_some());
    // 문 장치를 당긴다.
    pull_lever(&mut run, &mut w);
    assert_eq!(run.phase, Phase::GateOpen);
    assert!(!run.gate_closed && run.gate_opened);
    assert!(run.take_gate_new(), "the app records it");
    assert!(!run.take_gate_new(), "once");
    assert!(run.take_pulled(), "the app plays the pull (0197)");
    assert!(!run.take_pulled());
    run.guide = Some(hearth);
    assert_eq!(
        run.goal(),
        run.level.gate.map(|(_, c)| c),
        "the open gate comes first"
    );
    run.guide = None;
    // 0060 W1: 문이 열리면 갈 곳 = 문. 싸우는 동안은 없다.
    assert_eq!(run.goal(), run.level.gate.map(|(_, c)| c));
    // 성소 안으로.
    let (g, _) = run.level.gate.unwrap();
    w.player.pos = vec3(g.center().x, w.player.pos.y, g.min.z - 3.0);
    step(&mut run, &mut w);
    assert!(matches!(run.phase, Phase::Waking(_)), "{:?}", run.phase);
    assert!(run.gate_closed);
    assert_eq!(run.goal(), None, "no guide while the fight is on");
    for _ in 0..((WAKE_TIME * 60.0) as usize + 2) {
        w.player.hp = PLAYER_HP;
        step(&mut run, &mut w);
    }
    assert_eq!(run.phase, Phase::Duel);
    let b = run.boss.unwrap();
    assert!(w.enemies[b].active(), "awake: {:?}", w.enemies[b].state);
    assert!(run.boss_bar(&w).is_some());
    w.enemies[b].state = EState::Dead;
    step(&mut run, &mut w);
    assert_eq!(run.phase, Phase::Core);
    assert_eq!(run.goal(), run.core, "then the core");
    // 0226: 쓰러진 자리를 한 번 알린다 — 앱이 그 곁에 문지기의 것을 떨군다(바로 얻지 않는다).
    assert_eq!(run.take_boss_down(), run.core);
    assert_eq!(run.take_boss_down(), None, "told once");
    w.player.pos = run.core.unwrap();
    // 0118: 아래 지역이 있으면 핵은 판을 끝내지 않고 바닥 길을 연다 — 길이 갈 곳, 밟으면 내려간다.
    assert_eq!(step(&mut run, &mut w), StageOut::CoreTaken);
    assert_eq!(run.phase, Phase::Down);
    assert!(!run.gate_closed, "the way back out is open");
    assert_eq!(run.goal(), run.down_at);
    assert!(w.enemies.iter().all(|e| !e.boss), "the warden is gone");
    w.player.pos = run.down_at.unwrap();
    assert_eq!(step(&mut run, &mut w), StageOut::Descend);
}

/// 0118: 판의 차례 — 지역은 깊어지고, 마지막 지역만 바닥 길이 없다(핵 = 판의 끝). 첫 지역 말고는 오르는 길이 있다.
#[test]
fn stages_go_deeper_and_link_down_and_up() {
    for (i, def) in STAGES.iter().enumerate() {
        let (run, _) = StageRun::start(def);
        let last = i + 1 == STAGES.len();
        assert_eq!(
            run.down_at.is_none(),
            last,
            "stage {i}: a way down unless last"
        );
        assert_eq!(
            run.up_at.is_some(),
            i > 0,
            "stage {i}: a way up unless first"
        );
        if i > 0 {
            assert!(def.depth_m > STAGES[i - 1].depth_m, "stage {i} is deeper");
            assert_ne!(def.find_prefix, STAGES[i - 1].find_prefix);
        }
    }
}

/// 0121(0115 전투 안전 설계 — "AI보다 레벨 배치로 먼저"): 주민 자리(`1`·`2`·`3`)는 모든 서 있는 적·엘리트·문지기의 제자리에서
/// 끈(`LEASH_RANGE`) + 3m 밖 — 쫓아오는 적도 주민 곁까지는 오지 않는다. 화로와도 겹치지 않는다.
/// 0244(사장 설문 k13): 지역마다 떼 마당 하나 — 표의 수만큼 서고, 모두 걸을 수 있는 땅이고, 깨어난 자리·쉼터·주민·문지기
/// 싸움터에서 떨어져 있고, 서로 몰려 있다(마당 지름 안). 화로에서 쉬어 다시 세워도 같은 자리.
#[test]
fn every_stage_has_one_crowded_yard_away_from_rests_start_and_residents() {
    for def in STAGES {
        let (mut run, mut w) = StageRun::start(def);
        let n = def.horde.1;
        assert_eq!(
            run.horde_at.len(),
            n,
            "{}: yard holds the table's count",
            def.find_prefix
        );
        let c = run.horde_at.iter().copied().sum::<Vec3>() / n as f32;
        for p in &run.horde_at {
            assert!(
                run.level.on_floor(*p),
                "{}: on the floor {p:?}",
                def.find_prefix
            );
            assert!(
                vec2(p.x - c.x, p.z - c.z).length() <= (HORDE_RADIUS as f32 + 1.0) * CELL,
                "crowded"
            );
            let far = |a: Vec3, r: f32| vec2(p.x - a.x, p.z - a.z).length() >= r;
            assert!(far(run.level.player_start, HORDE_AWAY_START));
            assert!(
                run.rests.iter().all(|r| far(*r, HORDE_AWAY_REST)),
                "{}: away from rests",
                def.find_prefix
            );
            assert!(run
                .folk
                .list
                .iter()
                .all(|(_, r)| far(*r, HORDE_AWAY_RESIDENT)));
            assert!(
                run.zone_at(*p) + 1 < def.zones.len(),
                "not in the warden's arena"
            );
        }
        let before = run.horde_at.clone();
        let count = |w: &World| {
            w.enemies
                .iter()
                .filter(|e| {
                    before
                        .iter()
                        .any(|p| vec2(e.pos.x - p.x, e.pos.z - p.z).length() < 0.6)
                })
                .count()
        };
        assert!(count(&w) >= n, "{}: they stand there", def.find_prefix);
        run.populate(&mut w);
        assert_eq!(run.horde_at, before);
        assert!(count(&w) >= n);
    }
}

#[test]
fn residents_live_beyond_every_enemy_leash() {
    use crate::tuning::LEASH_RANGE;
    for def in STAGES {
        let (run, _) = StageRun::start(def);
        let foes: Vec<Vec3> = def
            .spawns
            .iter()
            .map(|s| s.0)
            .chain(def.elites.iter().map(|e| e.0))
            .chain(std::iter::once(def.boss.0))
            .flat_map(|c| run.level.marks_of(c).collect::<Vec<_>>())
            .collect();
        for r in ['1', '2', '3'] {
            for home in run.level.marks_of(r) {
                for f in &foes {
                    let d = vec2(home.x - f.x, home.z - f.z).length();
                    assert!(
                        d >= LEASH_RANGE + 3.0,
                        "{} resident {r} {d:.1} m from a foe",
                        def.find_prefix
                    );
                }
            }
        }
    }
    let (run, _) = StageRun::start(STAGES[1]);
    assert_eq!(
        ['1', '2', '3']
            .iter()
            .filter(|c| run.level.marks_of(**c).next().is_some())
            .count(),
        3,
        "three residents in the village"
    );
}

/// 0130 C5: 깨어난 적을 가까이(9m) 보면 도감에 한 번 — 멀면 없고, 같은 판에 다시 알리지 않는다. 청소자는 역할도.
#[test]
fn seeing_an_awake_foe_up_close_writes_the_codex_once() {
    let (mut run, mut w) = StageRun::start(STAGES[1]);
    let i = w
        .enemies
        .iter()
        .position(|e| e.docile)
        .expect("a scavenger in the village");
    let at = w.enemies[i].pos;
    w.player.pos = at + vec3(0.0, 0.0, crate::codex::SEE + 8.0);
    w.player.prev = w.player.pos;
    tick(&mut run, &mut w, Actions::default());
    let far = run.take_codex();
    assert!(
        !far.contains(&"husk.docile".to_string()),
        "too far: {far:?}"
    );
    w.player.pos = at + vec3(0.0, 0.0, 3.0);
    w.player.prev = w.player.pos;
    tick(&mut run, &mut w, Actions::default());
    let near = run.take_codex();
    assert!(near.contains(&"husk.docile".to_string()), "{near:?}");
    // 종 이름은 먼 데서 다른 잔재를 봤으면 그때 이미 — 합쳐서 한 번.
    let husk = far.iter().chain(&near).filter(|k| *k == "husk").count();
    assert_eq!(husk, 1, "the kind is told once: {far:?} {near:?}");
    tick(&mut run, &mut w, Actions::default());
    assert!(
        run.take_codex().iter().all(|k| k != "husk"),
        "told once per run"
    );
}

/// 0129 C4: 새긴 돌은 지도 표지 수 = 글 수, 곁에서 상호작용 = 읽기(다른 일을 하지 않는다), 멀면 아무것도 없다.
#[test]
fn carved_stones_have_one_text_each_and_read_with_the_interact_button() {
    for def in STAGES {
        let (run, _) = StageRun::start(def);
        assert_eq!(run.stones.len(), def.stones.len(), "marks = texts");
    }
    let (mut run, mut w) = StageRun::start(STAGES[1]);
    assert_eq!(run.stones.len(), 3);
    let at = run.stones[2];
    w.player.pos = at + vec3(0.0, 0.0, 6.0);
    w.player.prev = w.player.pos;
    tick(&mut run, &mut w, Actions::default());
    assert_eq!(run.near_stone, None);
    w.player.pos = at + vec3(0.0, 0.0, 1.2);
    w.player.prev = w.player.pos;
    tick(&mut run, &mut w, Actions::default());
    assert_eq!(run.near_stone, Some(2));
    assert!(w.interact_here, "the jump button reads it");
    tick(
        &mut run,
        &mut w,
        Actions {
            interact: true,
            ..Default::default()
        },
    );
    assert_eq!(run.take_read(), Some(2));
    assert_eq!(run.take_read(), None, "once per press");
}

/// 0122: 주민은 월드의 적이 아니다(맞지도·락온되지도 않는다 — 공격이 지나간다), 몸은 방랑자만 밀어낸다.
/// 곁에서 점프 버튼 = 말 걸기(다른 일을 하지 않는다), 가까이 오면 돌아보고, 보면 한 번 기록한다.
#[test]
fn residents_talk_glance_and_stay_out_of_the_fight() {
    let (mut run, mut w) = StageRun::start(STAGES[1]);
    assert_eq!(run.folk.list.len(), 3);
    assert_eq!(w.bodies.len(), 3);
    for &(_, at) in &run.folk.list {
        assert!(
            w.enemies.iter().all(|e| e.pos.distance(at) > 1.0),
            "not an enemy"
        );
    }
    let (_, at) = run.folk.list[0];
    // 멀리서는 안 보인다 → 다가가면 기록 한 번.
    w.player.pos = at + vec3(0.0, 0.0, crate::residents::SEE_RANGE + 6.0);
    tick(&mut run, &mut w, Actions::default());
    assert!(run.take_residents_seen().is_empty());
    w.player.pos = at + vec3(0.0, 0.0, 1.6);
    w.player.prev = w.player.pos;
    let face0 = run.folk.face[0];
    for _ in 0..90 {
        tick(&mut run, &mut w, Actions::default());
    }
    assert_eq!(run.take_residents_seen(), vec![0], "seen once");
    assert!(run.take_residents_seen().is_empty());
    let want = (w.player.pos.x - at.x).atan2(w.player.pos.z - at.z);
    let off = |f: f32| {
        ((want - f + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI)
            .abs()
    };
    assert!(
        off(run.folk.face[0]) <= off(face0).max(0.05),
        "turned toward the wanderer"
    );
    assert_eq!(run.folk.near, Some(0));
    assert!(w.interact_here);
    tick(
        &mut run,
        &mut w,
        Actions {
            interact: true,
            ..Default::default()
        },
    );
    assert_eq!(run.take_talked(), Some(0));
    assert!(run.rest_at.is_none(), "talking does nothing else");
    // 몸이 방랑자를 밀어낸다.
    w.player.pos = at + vec3(0.05, 0.0, 0.0);
    tick(&mut run, &mut w, Actions::default());
    assert!(
        w.player.pos.distance(at) >= crate::residents::BODY_R + crate::tuning::PLAYER_RADIUS - 0.05
    );
}

/// 0112·0116 T3: 구역 등급은 깊을수록 높고, 서 있는 적·엘리트가 그 등급으로 선다(보스는 그대로).
#[test]
fn deeper_regions_field_higher_tier_mobs() {
    for (i, def) in STAGES.iter().enumerate() {
        if i > 0 {
            assert!(def.tier >= STAGES[i - 1].tier);
        }
        let (run, w) = StageRun::start(def);
        for e in &w.enemies {
            // 떼 마당(0244)은 한 등급 아래.
            let yard = run
                .horde_at
                .iter()
                .any(|p| vec2(e.pos.x - p.x, e.pos.z - p.z).length() < 0.05);
            assert_eq!(
                e.tier,
                if e.boss {
                    1
                } else if yard {
                    def.tier.saturating_sub(1).max(1)
                } else {
                    def.tier
                },
                "{}",
                def.find_prefix
            );
        }
    }
    assert_eq!(STAGES[1].tier, 2, "0116: the village is tier 2");
}

/// 0118 T2: 땅 위 지역만 하늘·달·반딧불·나무, 땅속은 달이 없고 달빛이 약하며 흙이 떨어진다. 땅속엔 천장 틈 빛줄기가 있다.
#[test]
fn regions_below_ground_have_no_moon_and_light_through_ceiling_cracks() {
    for def in STAGES {
        let e = def.env;
        let below = def.depth_m > 0;
        assert_eq!(e.moon, !below, "{}", def.find_prefix);
        assert_eq!(e.fireflies || e.trees, !below);
        if below {
            assert!(e.falling && e.moonlight < 1.0);
            let (run, _) = StageRun::start(def);
            assert!(
                run.level.marks_of(SHAFT_GLYPH).next().is_some(),
                "light comes from somewhere"
            );
        }
    }
}

/// 0118: 내려와 닿으면 오르는 길 곁이지만 바로 올라가지 않는다 — 한 번 벗어났다 다시 밟아야. 장치 없는 지역은 문이 열린 단계.
#[test]
fn arriving_does_not_bounce_back_and_the_way_up_works_after_leaving_it() {
    let t = Tuning::default();
    let (mut run, mut w) = StageRun::start(STAGES[1]);
    assert_eq!(run.phase, Phase::GateOpen, "no lever, no waves: open");
    run.arrive_from_above(&mut w);
    let up = run.up_at.unwrap();
    w.player.pos = up;
    w.step(&Actions::default(), &t);
    assert_eq!(
        run.tick(&mut w, 1.0 / 60.0),
        StageOut::Playing,
        "not right away"
    );
    w.player.pos = up + vec3(0.0, 0.0, -PASSAGE_ARM - 0.5);
    w.step(&Actions::default(), &t);
    run.tick(&mut w, 1.0 / 60.0);
    w.player.pos = up;
    w.step(&Actions::default(), &t);
    assert_eq!(run.tick(&mut w, 1.0 / 60.0), StageOut::Ascend);
}

/// 0118: 아래에서 올라오면 이 판에 이긴 문지기는 없고 바닥 길이 열린 채 — 쉬었다 쓰러져도 그대로.
#[test]
fn coming_back_up_finds_the_warden_gone_and_the_floor_open_even_after_a_fall() {
    let (mut run, mut w) = StageRun::start(STAGES[0]);
    run.arrive_from_below(&mut w);
    assert_eq!(run.phase, Phase::Down);
    assert!(run.boss.is_none() && w.enemies.iter().all(|e| !e.boss));
    assert!(!run.gate_closed);
    let d = run.down_at.unwrap();
    assert!(
        w.player.pos.distance(d) > PASSAGE_REACH,
        "beside the hole, not in it"
    );
    rest_now(&mut run, &mut w, 0);
    w.player.hp = 0;
    assert_eq!(run.tick(&mut w, 1.0 / 60.0), StageOut::Respawned);
    assert_eq!(run.phase, Phase::Down);
    assert!(w.enemies.iter().all(|e| !e.boss), "still gone");
}

/// 문 장치 곁에서 상호작용(0113).
fn pull_lever(run: &mut StageRun, w: &mut World) {
    let l = run.lever.expect("the sanctuary has a lever");
    w.player.pos = l + vec3(1.0, 0.0, 0.0);
    w.player.prev = w.player.pos;
    tick(run, w, Actions::default());
    tick(
        run,
        w,
        Actions {
            interact: true,
            ..Default::default()
        },
    );
}

/// 지도에 서 있는 보통 적의 수(엘리트 빼고).
/// 지도 표지의 보통 적 + 떼 마당(0244).
fn placed_count(run: &StageRun) -> usize {
    run.def
        .spawns
        .iter()
        .map(|(c, _)| run.level.marks_of(*c).count())
        .sum::<usize>()
        + run.horde_at.len()
}

/// 0113: 지도의 적은 처음부터 제자리에(알아차리기 전) — 보통 적은 표의 표지 수만큼, 엘리트는 표의 차례·성질대로.
#[test]
fn placed_enemies_stand_at_their_marks_from_the_start() {
    let (run, w) = StageRun::start(&sanctuary::DEF);
    let plain: Vec<_> = w
        .enemies
        .iter()
        .filter(|e| !e.boss && e.elite.is_none())
        .collect();
    assert_eq!(plain.len(), placed_count(&run));
    assert!(plain.len() >= 20, "a real zone: {}", plain.len());
    assert!(plain
        .iter()
        .all(|e| e.state == EState::Idle && e.home.is_some()));
    let elites: Vec<_> = w
        .enemies
        .iter()
        .filter_map(|e| e.elite.map(|a| (e.kind, a)))
        .collect();
    let want: Vec<_> = sanctuary::DEF.elites.iter().map(|e| (e.1, e.2)).collect();
    assert_eq!(elites, want);
    assert!(sanctuary::DEF.waves.is_empty(), "no waves in a normal zone");
}

/// 0113: 화로 곁은 처음부터 안전하다 — 서 있는 적이 화로 곁의 플레이어를 알아차리지 않는다(쉴 수 있다).
#[test]
fn every_hearth_starts_safe_from_the_placed_enemies() {
    // 0118: 모든 지역.
    for def in STAGES {
        for i in 0..StageRun::start(def).0.rests.len() {
            let (mut run, mut w) = StageRun::start(def);
            // 쉬기 전에 곁에 잠깐 서 있는다(쉬면 적이 제자리로 돌아가 알아차림이 지워진다).
            w.player.pos = run.rests[i] + vec3(0.0, 0.0, 1.0);
            w.player.prev = w.player.pos;
            for _ in 0..10 {
                tick(&mut run, &mut w, Actions::default());
            }
            let woke: Vec<_> = w
                .enemies
                .iter()
                .filter(|e| !e.boss && e.state != EState::Idle)
                .map(|e| e.pos)
                .collect();
            assert!(woke.is_empty(), "hearth {i}: {woke:?} noticed");
            rest_now(&mut run, &mut w, i);
            assert!(run.rest_safe, "hearth {i} at {:?}", run.rests[i]);
            assert_eq!(run.rest_at.map(|r| r.0), Some(i));
        }
    }
}

/// 0113: 기록에 문이 열려 있으면 처음부터 열린 채(지름길) — 목표는 문.
#[test]
fn a_gate_opened_before_stays_open() {
    let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
    run.set_gate_opened(&mut w, true);
    assert_eq!(run.phase, Phase::GateOpen);
    assert!(!run.gate_closed);
    assert!(!run.take_gate_new(), "nothing new to record");
    assert_eq!(run.goal(), run.level.gate.map(|(_, c)| c));
}

/// 시험용: 한 틱(월드 → 스테이지).
fn tick(run: &mut StageRun, w: &mut World, a: Actions) -> StageOut {
    w.step(&a, &Tuning::default());
    run.tick(w, 1.0 / 60.0)
}

fn rest_now(run: &mut StageRun, w: &mut World, i: usize) -> StageOut {
    w.player.pos = run.rests[i] + vec3(0.0, 0.0, 1.0);
    w.player.prev = w.player.pos;
    tick(
        run,
        w,
        Actions {
            interact: true,
            ..Default::default()
        },
    )
}

/// 입구 쪽 쉼터(시작 자리에서 가장 가까운 것).
fn entrance_rest(run: &StageRun) -> usize {
    let s = run.level.player_start;
    (0..run.rests.len())
        .min_by(|a, b| {
            run.rests[*a]
                .distance(s)
                .total_cmp(&run.rests[*b].distance(s))
        })
        .unwrap()
}

/// 0032: 성소 지도에 쉼터 둘(입구·문 앞), 곁에서 상호작용 = 체력·등잔 가득 + 다시 일어설 곳.
/// 0220(사장 "맨손으로 시작하되 바로 옆에 검이 떨어져 있어야 도감이 제대로 작동할 듯"): 첫 지역의 깨어난 자리 곁(걸을 수 있는
/// 바닥, 화로와 겹치지 않게)에 검이 떨어져 있고, 곧바로 주울 수 있다 — 주우면 그 표지(`START_SWORD`)가 나온다. 두 번 놓지 않는다.
#[test]
fn the_starting_sword_lies_by_the_waking_spot_and_can_be_picked_up() {
    let (mut run, mut w) = StageRun::start(STAGES[0]);
    run.intro = 0.0;
    run.lay_start_sword();
    run.lay_start_sword();
    assert_eq!(run.drops.len(), 1, "laid once");
    let p = run.drops[0].0;
    let start = run.level.player_start;
    let d = vec2(p.x - start.x, p.z - start.z).length();
    assert!((1.0..2.5).contains(&d), "right beside the start: {d}");
    let (x, z) = (
        (p.x / CELL + run.level.w as f32 * 0.5).floor() as usize,
        (p.z / CELL + run.level.h as f32 * 0.5).floor() as usize,
    );
    assert!(run.level.walkable(x, z), "on walkable floor");
    assert!(
        run.rests.iter().all(|r| r.distance(p) > 1.2),
        "not on the hearth"
    );
    w.player.pos = p;
    w.player.prev = p;
    w.interact = true;
    run.tick(&mut w, 0.016);
    assert_eq!(run.take_looted(), Some(START_SWORD));
    assert!(run.drops.is_empty());
}

/// 0226: 떨어진 아이템은 주워야 얻는다 — 내려앉기 전엔 못 줍고, **가방이 가득이면 못 줍는다**(바닥에 남는다, 버튼은 그대로
/// 점프·HUD가 까닭을 말한다). 자리가 나면 줍는다. 같은 자리에 떨어진 것은 옆으로 비킨다.
#[test]
fn a_fallen_item_is_picked_up_unless_the_bag_is_full() {
    use crate::items::{Base, Item};
    let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
    run.intro = 0.0;
    let at = w.player.pos;
    let sword = Item::plain(Base::Weapon(crate::weapons::Id::Sword));
    run.drop_item(at, sword, false);
    run.drop_item(at, sword, false);
    assert_eq!(run.item_serial(), 2);
    assert!(
        run.item_drops[0].pos.distance(run.item_drops[1].pos) > 0.5,
        "side by side, not stacked"
    );
    run.tick(&mut w, 0.016);
    assert_eq!(run.near_item, None, "still falling");
    for _ in 0..40 {
        run.tick(&mut w, 0.016);
    }
    assert!(run.near_item.is_some());
    assert_eq!(run.near(), Some(Near::Pick));
    // 가득이면: 곁에 있어도 줍기가 아니다.
    run.bag_full = true;
    run.tick(&mut w, 0.016);
    assert_eq!(run.near_item, None);
    assert!(run.near_blocked);
    assert_eq!(run.near(), Some(Near::Full));
    assert!(!w.interact_here, "the button stays a jump");
    w.interact = true;
    run.tick(&mut w, 0.016);
    assert_eq!(run.take_item(), None);
    assert_eq!(run.item_drops.len(), 2, "left on the ground");
    // 자리가 났다.
    run.bag_full = false;
    w.interact = false;
    run.tick(&mut w, 0.016);
    assert!(w.interact_here);
    w.interact = true;
    run.tick(&mut w, 0.016);
    assert_eq!(run.take_item().map(|(it, _)| it), Some(sword));
    assert_eq!(run.take_item(), None, "taken once");
    assert_eq!(run.item_drops.len(), 1);
    // 발치에 내려놓은 것은 한 번 곁을 떠나야 줍는다(쉬기를 가리지 않는다 · 도로 줍지 않는다).
    run.item_drops.clear();
    w.interact = false;
    run.drop_item(at, sword, true);
    for _ in 0..40 {
        run.tick(&mut w, 0.016);
    }
    assert_eq!(run.near_item, None, "just put down");
    assert!(!run.near_blocked);
    w.player.pos = at + Vec3::X * (FIND_REACH + 0.5);
    run.tick(&mut w, 0.016);
    w.player.pos = at;
    run.tick(&mut w, 0.016);
    assert_eq!(
        run.near_item,
        Some(0),
        "back again — an item like any other"
    );
    // 기록에서 되놓은 것은 바로 주울 수 있다.
    run.item_drops.clear();
    run.lay_item(at, sword);
    run.tick(&mut w, 0.016);
    assert_eq!(run.near_item, Some(0));
}

/// 0226: 떨굼은 걸을 수 있는 땅에만 놓인다 — 벽 속을 겨누면 떨군 자리 쪽으로 물러나고, 줄 위에 땅이 없으면 가장 가까운
/// 땅 칸으로. 땅이면 그대로.
#[test]
fn drops_land_only_on_the_floor() {
    let (run, w) = StageRun::start(&sanctuary::DEF);
    let lv = &run.level;
    let start = w.player.pos;
    assert!(lv.on_floor(start));
    assert_eq!(lv.floor_toward(start, start), start, "already on the floor");
    // 가장 가까운 벽 칸의 가운데를 겨눈다.
    let (sx, sz) = lv.cell_of(start).unwrap();
    let wall = (0..lv.h)
        .flat_map(|z| (0..lv.w).map(move |x| (x, z)))
        .filter(|&(x, z)| !lv.walkable(x, z))
        .min_by_key(|&(x, z)| x.abs_diff(sx) + z.abs_diff(sz))
        .unwrap();
    let into = lv.at(wall.0, wall.1);
    assert!(!lv.on_floor(into));
    let got = lv.floor_toward(start, into);
    assert!(lv.on_floor(got), "pulled back onto the floor");
    assert!(got.distance(start) <= into.distance(start) + 0.01);
    // 떨군 자리도 벽 속이면: 가장 가까운 땅 칸.
    let lost = lv.floor_toward(into, into);
    assert!(lv.on_floor(lost));
    assert!(vec2(lost.x - into.x, lost.z - into.z).length() < CELL * 2.0);
}

/// 0226: 가방이 가득이면 깨어난 자리의 검(아직 없는 바탕)도 못 줍는다 — 자리가 나면 줍는다.
#[test]
fn a_full_bag_also_holds_back_the_starting_sword() {
    let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
    run.intro = 0.0;
    run.drops.push((w.player.pos, START_SWORD, DROP_SETTLE));
    run.bag_full = true;
    run.start_block = true;
    run.tick(&mut w, 0.016);
    assert_eq!(run.near_drop, None);
    assert_eq!(run.near(), Some(Near::Full));
    run.bag_full = false;
    run.tick(&mut w, 0.016);
    assert_eq!(run.near_drop, Some(0));
}

/// 0226: 가방이 가득일 때 상자의 것 — 아직 없는 바탕(칸이 든다)은 못 줍고, 이미 가진 것(잔불이 된다)은 줍는다.
#[test]
fn a_full_bag_holds_back_only_the_chest_loot_that_needs_a_slot() {
    let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
    run.intro = 0.0;
    let at = w.player.pos;
    run.drops.push((at, 1, DROP_SETTLE));
    run.bag_full = true;
    run.chest_block = 1 << 1;
    run.tick(&mut w, 0.016);
    assert_eq!(run.near_drop, None);
    assert_eq!(run.near(), Some(Near::Full));
    run.chest_block = 0;
    run.tick(&mut w, 0.016);
    assert_eq!(run.near_drop, Some(0), "embers need no room");
    assert!(!run.near_blocked);
}

#[test]
fn a_chest_opens_once_with_the_interact_button_and_stays_open_from_the_record() {
    let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
    assert_eq!(
        run.chest.at.len(),
        sanctuary::DEF.chest_loot.len(),
        "loot per chest"
    );
    run.intro = 0.0;
    let c = run.chest.at[1];
    w.player.pos = c + vec3(0.0, 0.0, 1.2);
    w.player.prev = w.player.pos;
    run.tick(&mut w, 0.016);
    assert_eq!(run.chest.near, Some(1));
    assert!(w.interact_here, "the jump button opens it");
    w.interact = true;
    run.tick(&mut w, 0.016);
    assert_eq!(run.take_opened(), Some(1));
    assert_eq!(run.take_opened(), None, "taken once");
    w.interact = true;
    run.tick(&mut w, 0.016);
    assert_eq!(run.take_opened(), None, "an open chest stays open");
    assert_eq!(run.chest.near, None);
    // 0131: 열면 얻는 것이 아니라 상자 앞 바닥에 떨어진다 — 내려앉기 전엔 못 줍고, 주우면 그 상자의 것.
    assert_eq!(run.drops.len(), 1);
    assert!(
        run.drops[0].0.distance(c) > 1.0,
        "out in front of the chest"
    );
    assert_eq!(run.take_looted(), None, "not taken by opening");
    assert!(
        run.drops[0].0.distance(w.player.pos) > 0.6,
        "not under the wanderer's feet"
    );
    w.player.pos = run.drops[0].0;
    w.interact = false;
    run.tick(&mut w, 0.016);
    assert_eq!(run.near_drop, None, "still flying");
    for _ in 0..40 {
        run.tick(&mut w, 0.016);
    }
    assert_eq!(run.near_drop, Some(0));
    w.interact = true;
    run.tick(&mut w, 0.016);
    assert_eq!(run.take_looted(), Some(1));
    assert!(run.drops.is_empty());
    // 기록에서 연 상자는 새 판에서도 열려 있다.
    let (mut run, _) = StageRun::start(&sanctuary::DEF);
    run.set_opened(&[0, 9]);
    assert_eq!(run.chest.open, vec![true, false, false, false]);
}

/// 0105·0106 → 0123: 숨은 물건 넷(등잔 하나 · 한 모금 하나 · 종소리 조각 둘 — 덜 후하게) + 덧옷 둘(0176) + 숫돌 하나(0180)는 걸어서 닿는 바닥에 있고,
/// 점프 버튼으로 한 번 줍고, 기록의 것은 새 판에 없다.
#[test]
fn hidden_finds_are_reachable_picked_once_and_gone_from_the_record() {
    let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
    let kinds: Vec<Find> = run.hidden.list.iter().map(|f| f.1).collect();
    assert_eq!(
        kinds,
        vec![
            Find::Lamp,
            Find::Sip,
            Find::Shard,
            Find::Shard,
            Find::Garb,
            Find::Garb,
            Find::Stone
        ]
    );
    let ids: Vec<&str> = run.hidden.list.iter().map(|f| f.2.as_str()).collect();
    assert_eq!(
        ids,
        vec![
            "sanctuary.lamp.0",
            "sanctuary.sip.0",
            "sanctuary.shard.0",
            "sanctuary.shard.1",
            "sanctuary.garb.0",
            "sanctuary.garb.1",
            "sanctuary.stone.0"
        ]
    );
    for (p, _, id) in &run.hidden.list {
        // `Level::at`의 거꾸로.
        let cell = (
            (p.x / CELL + run.level.w as f32 * 0.5 - 0.5).round() as usize,
            (p.z / CELL + run.level.h as f32 * 0.5 - 0.5).round() as usize,
        );
        assert_eq!(run.level.at(cell.0, cell.1), vec3(p.x, 0.0, p.z));
        assert!(run.level.walkable(cell.0, cell.1), "{id} on a floor");
    }
    run.intro = 0.0;
    let (p, kind, id) = run.hidden.list[1].clone();
    w.player.pos = p + vec3(0.0, 0.0, 1.0);
    w.player.prev = w.player.pos;
    run.tick(&mut w, 0.016);
    assert_eq!(run.hidden.near, Some(1));
    assert!(w.interact_here, "the jump button picks it up");
    w.interact = true;
    run.tick(&mut w, 0.016);
    assert_eq!(run.take_picked(), Some((kind, id.clone())));
    assert_eq!(run.take_picked(), None, "taken once");
    w.interact = true;
    run.tick(&mut w, 0.016);
    assert_eq!(run.take_picked(), None, "gone after picking");
    let (mut run, _) = StageRun::start(&sanctuary::DEF);
    run.set_found(&[id, "elsewhere.sip.0".to_string()]);
    assert_eq!(
        run.hidden.found,
        vec![false, true, false, false, false, false, false]
    );
}

/// 0105(lore CANON 0101): 처음 이긴 문지기의 핵은 다가오면 스스로 방랑자 쪽으로 온다. 그다음 판부터는 그 자리에 있다.
#[test]
fn the_first_core_drifts_to_the_wanderer_later_ones_wait() {
    for first in [true, false] {
        let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
        run.intro = 0.0;
        run.first_core = first;
        let c0 = w.player.pos + vec3(3.0, 0.0, 0.0);
        run.core = Some(c0);
        run.phase = Phase::Core;
        for _ in 0..30 {
            run.tick(&mut w, 1.0 / 60.0);
        }
        let moved = run.core.map_or(0.0, |c| c.distance(c0));
        if first {
            assert!(moved > 0.5, "the first core comes to you ({moved})");
        } else {
            assert!(moved < 1e-4, "later cores wait");
        }
    }
}

#[test]
fn resuming_from_a_record_stands_at_that_rest_site_with_the_same_progress() {
    // 문 앞 쉼터에서 쉰 기록(문이 열린 뒤) → 새 판에서 이어하기 = 그 쉼터 곁, 문은 열려 있다.
    let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
    let last = run.rests.len() - 1;
    run.resume(
        &mut w,
        crate::save::RunAt {
            rest: last,
            gate_open: true,
            ..Default::default()
        },
    );
    assert_eq!(run.phase, Phase::GateOpen);
    assert!(!run.gate_closed);
    assert_eq!(run.intro, 0.0, "no opening story when continuing");
    let at = run.rests[last];
    assert!(vec2(w.player.pos.x - at.x, w.player.pos.z - at.z).length() < 2.0);
    assert_eq!(
        run.run_at(),
        Some(crate::save::RunAt {
            rest: last,
            gate_open: true,
            ..Default::default()
        })
    );
    // 서 있는 적은 쉼터에서 다시 섰다(0113).
    assert_eq!(
        w.enemies
            .iter()
            .filter(|e| !e.boss && e.elite.is_none())
            .count(),
        placed_count(&run)
    );
    assert!(
        run.gate_opened,
        "an old run record moves the open gate into the record"
    );
    assert!(run.take_gate_new());
    // 입구 쉼터·물결 전 기록이면 입구 단계.
    let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
    run.resume(
        &mut w,
        crate::save::RunAt {
            rest: 0,
            gate_open: false,
            ..Default::default()
        },
    );
    assert_eq!(run.phase, Phase::Entrance);
    assert!(run.gate_closed);
    // 지도에 없는 쉼터 번호면 아무것도 하지 않는다(처음부터).
    let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
    run.resume(
        &mut w,
        crate::save::RunAt {
            rest: 99,
            gate_open: true,
            ..Default::default()
        },
    );
    assert_eq!(run.phase, Phase::Entrance);
    assert_eq!(run.run_at(), None);
}

#[test]
fn resting_refills_and_marks_the_rest_site() {
    let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
    assert_eq!(run.rests.len(), 3, "gate, root hollow, entrance");
    let i = entrance_rest(&run);
    w.player.hp = 10;
    w.player.flask = 0;
    rest_now(&mut run, &mut w, i);
    assert_eq!(run.near_rest, Some(i));
    assert_eq!((w.player.hp, w.player.flask), (PLAYER_HP, FLASK_CHARGES));
    assert_eq!(run.rest_at, Some((i, Phase::Entrance)));
}

/// 0171: 방랑자는 화로 곁에서 깨어난다 — 시작 자리는 가장 가까운 쉼터의 곁(몇 걸음)이되 쉬는 거리 바로 밖이고
/// (일어나자마자 "쉬기"가 뜨지 않게), 깨어 있을 적이 볼 수 없는 자리다.
#[test]
fn the_wanderer_wakes_beside_a_hearth() {
    let (run, w) = StageRun::start(&sanctuary::DEF);
    let at = w.player.pos;
    let near = run
        .rests
        .iter()
        .map(|r| r.distance(at))
        .fold(f32::MAX, f32::min);
    assert!(near > REST_REACH && near < 4.0, "{near}");
    assert_eq!(run.near(), None, "nothing to press on waking");
    let foe = w
        .enemies
        .iter()
        .map(|e| e.pos.distance(at))
        .fold(f32::MAX, f32::min);
    assert!(foe > REST_SAFE, "no one stands over the sleeper: {foe}");
}

/// 0171: 꺼진 화로 — 시작 자리에서 가장 가까운 쉼터가 꺼져 있다가, **거기서** 처음 쉴 때 불이 붙는다(한 번).
/// 다른 화로에서 쉬어도 그 화로는 꺼진 채다. 끄지 않은 판(옛 기록)에는 아무 일도 없다.
#[test]
fn the_cold_hearth_lights_when_first_rested_at() {
    let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
    let home = entrance_rest(&run);
    // 끄지 않았으면 쉬어도 불 붙는 일은 없다.
    rest_now(&mut run, &mut w, home);
    assert!(run.level.cold.is_none() && !run.take_kindled());
    let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
    run.chill_hearth();
    assert_eq!(
        run.level.cold,
        Some(run.rests[home]),
        "the one by the start"
    );
    // 다른 화로에서 쉰다: 그대로 꺼져 있다.
    let other = (0..run.rests.len()).find(|&i| i != home).unwrap();
    for e in &mut w.enemies {
        e.state = EState::Dead;
    }
    rest_now(&mut run, &mut w, other);
    assert!(run.take_rested() && !run.take_kindled());
    assert!(run.level.cold.is_some());
    // 그 화로에서 쉰다: 불이 붙는다 — 한 번만 알린다.
    rest_now(&mut run, &mut w, home);
    assert!(run.take_rested() && run.take_kindled());
    assert_eq!(run.level.cold, None);
    rest_now(&mut run, &mut w, home);
    assert!(!run.take_kindled(), "lit stays lit");
}

/// 0171 종소리의 길 안내: 닫힌 문의 장치를 찾는 동안에만 종소리가 온 곳이 있다 — 장치를 당기면(문이 열리면) 갈 곳이 맡는다.
#[test]
fn the_bell_calls_from_the_lever_until_the_gate_opens() {
    let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
    assert_eq!(run.bell_from(), run.lever);
    assert!(run.bell_from().is_some() && run.goal().is_none());
    pull_lever(&mut run, &mut w);
    assert_eq!(run.bell_from(), None);
    assert!(run.goal().is_some());
}

/// 0032: 깨어 있는 적이 `REST_SAFE` 안이면 쉴 수 없다(잠든 문지기는 괜찮다).
#[test]
fn you_cannot_rest_with_an_awake_enemy_close() {
    let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
    let i = entrance_rest(&run);
    w.spawn(Kind::Husk, run.rests[i] + vec3(3.0, 0.0, 0.0));
    w.player.hp = 10;
    rest_now(&mut run, &mut w, i);
    assert!(!run.rest_safe);
    assert_eq!(run.rest_at, None);
    assert_eq!(w.player.hp, 10);
}

/// 0032 → 0113: 쉰 뒤 쓰러지면 그 쉼터에서 — 가득 찬 채, 서 있는 적은 다시 서고(소울 화톳불), 이 판에 쓰러뜨린 엘리트는
/// 다시 서지 않는다, 문지기는 잠든 채.
#[test]
fn falling_after_a_rest_rises_there_and_the_zone_stands_again_but_not_downed_elites() {
    let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
    let i = entrance_rest(&run);
    rest_now(&mut run, &mut w, i);
    // 보통 적 둘과 첫 엘리트를 쓰러뜨린다.
    let mut down = 0;
    for e in w
        .enemies
        .iter_mut()
        .filter(|e| !e.boss && e.elite.is_none())
    {
        e.state = EState::Dead;
        down += 1;
        if down == 2 {
            break;
        }
    }
    let first = w.enemies.iter().position(|e| e.elite.is_some()).unwrap();
    w.enemies[first].state = EState::Dead;
    tick(&mut run, &mut w, Actions::default());
    w.player.hp = 0;
    w.player.flask = 0;
    assert_eq!(
        tick(&mut run, &mut w, Actions::default()),
        StageOut::Respawned
    );
    assert_eq!(run.phase, Phase::Entrance);
    assert!(w.player.pos.distance(run.rests[i]) < REST_REACH);
    assert_eq!((w.player.hp, w.player.flask), (PLAYER_HP, FLASK_CHARGES));
    let plain = w
        .enemies
        .iter()
        .filter(|e| e.alive() && !e.boss && e.elite.is_none())
        .count();
    assert_eq!(plain, placed_count(&run), "everyone stands again");
    assert_eq!(
        w.enemies.iter().filter(|e| e.elite.is_some()).count(),
        sanctuary::DEF.elites.len() - 1,
        "the downed elite stays down this run"
    );
    assert_eq!(w.enemies[run.boss.unwrap()].state, EState::Dormant);
    assert!(run.gate_closed);
}

/// 0112 → 0113: 엘리트를 처음 쓰러뜨리면 잔불을 더 흩고 앱에 한 번 알린다 — 기록에 있으면(다음 판) 덤은 없다.
/// 한 틱에 여럿이 쓰러져도(수류탄) 다 센다.
#[test]
fn elites_pay_extra_the_first_time_only() {
    let first_drop = |beaten: &[String]| {
        let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
        run.set_beaten(beaten);
        for e in w.enemies.iter_mut().filter(|e| e.elite.is_some()) {
            e.state = EState::Dead;
        }
        w.drops.clear();
        tick(&mut run, &mut w, Actions::default());
        let extra = w.drops.iter().map(|d| d.value).sum::<u32>();
        (run.take_elites_new(), extra)
    };
    let (news, extra) = first_drop(&[]);
    assert_eq!(news.len(), sanctuary::DEF.elites.len());
    assert_eq!(news[0], elite_id("sanctuary", 0));
    // 쓰러진 적의 제 잔불은 두 판이 같다 — 차이가 처음 덤.
    let (again, plain) = first_drop(&news);
    assert!(again.is_empty(), "the first-time bonus is once");
    assert_eq!(
        extra - plain,
        crate::tuning::ELITE_FIRST_EMBERS * news.len() as u32
    );
}

/// 0032 → 0113: 문을 연 뒤 문 앞 쉼터에서 쉬었으면 결투에서 쓰러져도 문 앞부터 — 문은 열려 있고, 문지기는 잠든 채 가득.
#[test]
fn falling_in_the_duel_after_the_gate_rest_restarts_the_duel_only() {
    let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
    pull_lever(&mut run, &mut w);
    assert_eq!(run.phase, Phase::GateOpen);
    // 문 앞 쉼터 = 문에 가장 가까운 것.
    let gc = run.level.gate.unwrap().1;
    let gate_rest = (0..run.rests.len())
        .min_by(|a, b| {
            run.rests[*a]
                .distance(gc)
                .total_cmp(&run.rests[*b].distance(gc))
        })
        .unwrap();
    rest_now(&mut run, &mut w, gate_rest);
    assert_eq!(run.rest_at, Some((gate_rest, Phase::GateOpen)));
    // 성소로 → 깨어남 → 결투.
    let (g, _) = run.level.gate.unwrap();
    w.player.pos = vec3(g.center().x, w.player.pos.y, g.min.z - 3.0);
    for _ in 0..((WAKE_TIME * 60.0) as usize + 4) {
        w.player.hp = PLAYER_HP;
        tick(&mut run, &mut w, Actions::default());
    }
    assert_eq!(run.phase, Phase::Duel);
    let b = run.boss.unwrap();
    w.enemies[b].hp = 30;
    w.player.hp = 0;
    assert_eq!(
        tick(&mut run, &mut w, Actions::default()),
        StageOut::Respawned
    );
    assert_eq!(run.phase, Phase::GateOpen);
    assert!(!run.gate_closed);
    let boss = &w.enemies[run.boss.unwrap()];
    assert_eq!((boss.hp, boss.state), (boss.max_hp, EState::Dormant));
    assert!(w.player.pos.distance(run.rests[gate_rest]) < REST_REACH);
}

#[test]
fn falling_ends_the_stage() {
    let t = Tuning::default();
    let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
    w.player.hp = 0;
    w.step(&Actions::default(), &t);
    assert_eq!(run.tick(&mut w, 1.0 / 60.0), StageOut::Defeated);
}
