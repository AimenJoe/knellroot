use super::*;
use crate::input::Actions;
use crate::tuning::Tuning;
use crate::tuning::{FLASK_CHARGES, PLAYER_HP};

/// 0149(사장 "보스가 깨어나기 전에 타겟팅이 안 돼서 엉뚱한 적에게 락온"): 깨어나는 2초 동안에도 락온은 보스를 잡는다 —
/// 더 가까이 깬 잡몹이 있어도. 아직 깨어나기 시작하지 않은 잠든 보스는 잡히지 않고(싸울 것이 아니다), 다 깨면 깃발은 내려간다.
#[test]
fn lock_on_takes_the_boss_while_it_is_still_waking() {
    let t = Tuning::default();
    let (mut run, mut w) = StageRun::start(STAGES[REGIONS - 1]);
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
        (STAGES[REGIONS - 1], Kind::Overseer, true),
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
    assert!(
        w.enemies.iter().all(|e| !e.boss || e.field),
        "the warden is gone"
    );
    w.player.pos = run.down_at.unwrap();
    assert_eq!(step(&mut run, &mut w), StageOut::Descend);
}

/// 0118: 판의 차례 — 지역은 깊어지고, 마지막 지역만 바닥 길이 없다(핵 = 판의 끝). 첫 지역 말고는 오르는 길이 있다.
#[test]
fn stages_go_deeper_and_link_down_and_up() {
    // 0262: 서막 숲은 판의 차례 밖 — 앞의 `REGIONS`개만.
    for (i, def) in STAGES[..REGIONS].iter().enumerate() {
        let (run, _) = StageRun::start(def);
        let last = i + 1 == REGIONS;
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
    // 0262: 서막 숲에는 떼 마당이 없다(잔재 둘만).
    for def in STAGES.iter().filter(|d| !d.prologue) {
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
    // 0262: 등급의 차례는 판의 차례만(서막 숲은 첫 등급 — 아래 시험).
    for (i, def) in STAGES[..REGIONS].iter().enumerate() {
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
    assert!(run.boss.is_none() && w.enemies.iter().all(|e| !e.boss || e.field));
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
    assert!(w.enemies.iter().all(|e| !e.boss || e.field), "still gone");
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
fn every_hearth_has_room_around_it() {
    /// 화톳불 둘레를 비우는 거리(m) — 쉬는 거리 `REST_SAFE`보다 조금 넉넉히.
    const HEARTH_CLEAR: f32 = 10.5;
    // 2026-11-23(사장 "정원 깊은 곳, 화톳불과 적들이 너무 가까움" — 가시짐승이 화톳불에서 4.5m에 서 있었다): 서 있는 적·엘리트·떼는
    // 화톳불에서 `HEARTH_CLEAR` 밖에 둔다(쉬는 거리 `REST_SAFE`보다 넉넉히 — 쉬고 일어나 바로 싸움이 되지 않게).
    let mut close = Vec::new();
    for def in STAGES {
        let (run, w) = StageRun::start(def);
        for r in &run.rests {
            for e in w.enemies.iter().filter(|e| !e.boss) {
                let d = vec2(e.pos.x - r.x, e.pos.z - r.z).length();
                if d < HEARTH_CLEAR {
                    let cell = run.level.cell_of(e.pos);
                    close.push(format!("{} {:?} {cell:?} {d:.1}m", def.find_prefix, e.kind));
                }
            }
        }
    }
    assert!(close.is_empty(), "too near a hearth: {close:#?}");
}

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

/// 0262: 서막 숲은 지역 표의 끝 — 판의 차례(앞의 `REGIONS`개) 밖이고 그것 하나뿐. 문지기·핵·문·장치·화로·상자·숨은 물건·새긴 돌·
/// 엘리트·떼 마당·내려가는·오르는 길이 없다(기록 키를 늘리지 않는다). 땅 위·첫 등급, 끝의 표지(`E`)가 있다.
#[test]
fn the_prologue_forest_sits_at_the_end_outside_the_run_order() {
    assert_eq!(STAGES.len(), REGIONS + 1);
    assert_eq!(PROLOGUE, REGIONS);
    for (i, def) in STAGES.iter().enumerate() {
        assert_eq!(def.prologue, i == PROLOGUE, "{}", def.find_prefix);
    }
    let def = STAGES[PROLOGUE];
    let (run, w) = StageRun::start(def);
    assert!(run.boss.is_none() && w.enemies.iter().all(|e| !e.boss || e.field));
    assert!(run.rests.is_empty() && run.chest.at.is_empty() && run.hidden.list.is_empty());
    assert!(run.stones.is_empty() && run.lever.is_none());
    assert!(run.level.gate.is_none());
    // 0265: 숲 끝의 무너진 아치 — 표지 너머, 설주 둘만 막는다(가운데는 지나간다).
    let arch = run.level.arch().expect("the ruined arch");
    assert!(
        arch.z < run.exit_at.unwrap().z,
        "the arch is beyond the mark"
    );
    let jambs = w
        .obstacles
        .iter()
        .filter(|b| (b.center().z - arch.z).abs() < 1.0)
        .count();
    assert_eq!(jambs, 2);
    assert!(
        w.obstacles
            .iter()
            .all(|b| !(b.min.x..=b.max.x).contains(&arch.x)
                || !(b.min.z..=b.max.z).contains(&arch.z)),
        "the way through is open"
    );
    assert!(run.down_at.is_none() && run.up_at.is_none() && run.horde_at.is_empty());
    assert!(def.elites.is_empty() && def.chest_loot.is_empty() && def.boss_charm.is_none());
    assert_eq!((def.depth_m, def.tier), (0, 1));
    assert!(run.exit_at.is_some(), "the way out of the wood");
    // 잔재 둘(해 보는 상대) — 끈이 짧다.
    assert_eq!(w.enemies.len(), 2);
    assert!(w
        .enemies
        .iter()
        .all(|e| e.leash == crate::tuning::PROLOGUE_LEASH));
    // 숲 끝은 깨어난 자리에서 멀다(북쪽) — 짧지만 걸어 나오는 길.
    let (start, exit) = (run.level.player_start, run.exit_at.unwrap());
    assert!(exit.z < start.z - 40.0, "{start:?} → {exit:?}");
}

/// 2026-11-22(사장 "숲의 각종 지형을 그냥 통과할수 있는 버그"): 숲은 벽(`#`) 없이 바위 단(빈칸)으로 두른다 — 빈칸이 막지 않아
/// 나무·바위 단을 걸어 지났다. 깨어난 자리에서 사방으로 오래 걸어도(뛰어도) 늘 걸을 수 있는 칸 위에 있다.
#[test]
fn the_forest_edges_stop_the_wanderer() {
    let t = Tuning::default();
    for dir in [
        vec2(1.0, 0.0),
        vec2(-1.0, 0.0),
        vec2(0.0, 1.0),
        vec2(0.7, -0.7),
        vec2(-0.7, -0.7),
    ] {
        let (run, mut w) = StageRun::start(STAGES[PROLOGUE]);
        w.unhurt = true;
        for i in 0..60 * 8 {
            let a = Actions {
                move_dir: dir,
                jump: i % 40 == 0,
                ..Actions::default()
            };
            w.step(&a, &t);
            let (x, z) = run.level.cell_of(w.player.pos).expect("on the map");
            assert!(
                run.level.walkable(x, z),
                "walked onto a blank cell ({x},{z}) going {dir:?}"
            );
        }
    }
}

/// 0262: 숲에서는 입구 단계 그대로 — 목표는 "종소리를 따라가라", 종소리는 숲 끝의 표지에서(갈 곳의 빛기둥은 없다 — 종소리가 맡는다).
/// 표지를 밟으면 `Onward`(앱이 "성소에 들다"를 연다). 시작 자리 곁에서는 아니다.
#[test]
fn the_forest_bell_rings_from_its_end_and_stepping_there_goes_onward() {
    let (mut run, mut w) = StageRun::start(STAGES[PROLOGUE]);
    assert_eq!(run.phase, Phase::Entrance);
    let want: Tr = |s| s.obj_follow_bell;
    assert_eq!(
        run.objective.map(crate::i18n::tr),
        Some(crate::i18n::tr(want))
    );
    assert_eq!(run.bell_from(), run.exit_at);
    assert_eq!(run.goal(), None);
    w.enemies.clear();
    for _ in 0..30 {
        assert_eq!(
            tick(&mut run, &mut w, Actions::default()),
            StageOut::Playing
        );
    }
    let exit = run.exit_at.unwrap();
    // 표지 둘레를 비껴(옆으로 4m) 표지 줄을 넘어도 — 아치 쪽으로 가는 길은 다 걸린다(2026-11-23 사장 "트리거 안 되고 지나가는 스팟").
    let (mut side, mut ws) = StageRun::start(STAGES[PROLOGUE]);
    ws.enemies.clear();
    ws.player.pos = exit + vec3(4.0, 0.0, 1.0);
    ws.player.prev = ws.player.pos;
    assert_eq!(
        tick(&mut side, &mut ws, Actions::default()),
        StageOut::Playing
    );
    ws.player.pos = exit + vec3(4.0, 0.0, -0.2);
    ws.player.prev = ws.player.pos;
    assert_eq!(
        tick(&mut side, &mut ws, Actions::default()),
        StageOut::Onward
    );
    w.player.pos = exit + vec3(0.0, 0.0, 1.0);
    w.player.prev = w.player.pos;
    assert_eq!(tick(&mut run, &mut w, Actions::default()), StageOut::Onward);
}

/// 0262: 숲에서 쓰러지면(쉼터가 없다) 끝 장면이 아니라 깨어난 자리에서 가득 찬 채 다시 선다 — 잔재도 제자리에.
#[test]
fn falling_in_the_forest_rises_again_at_the_waking_spot() {
    let (mut run, mut w) = StageRun::start(STAGES[PROLOGUE]);
    let start = run.level.player_start;
    let homes: Vec<Vec3> = w.enemies.iter().map(|e| e.pos).collect();
    w.player.pos = start + vec3(0.0, 0.0, -20.0);
    w.enemies[0].pos += vec3(3.0, 0.0, 0.0);
    w.player.hp = 0;
    assert_eq!(run.tick(&mut w, 1.0 / 60.0), StageOut::Respawned);
    assert_eq!(w.player.pos, start);
    assert_eq!(w.player.hp, w.player.max_hp);
    let now: Vec<Vec3> = w.enemies.iter().map(|e| e.pos).collect();
    assert_eq!(now, homes, "the remnants stand at home again");
}

/// 0262: 성소에 닿는 자리(`A`)는 입구 폐허의 남쪽 가장자리 바닥 — 그 아래 줄은 벽이다. 닿으면 거기서 북쪽(성소 쪽)을 보고, 시작 이야기·
/// 카드는 없다(이름은 장면이 보인다). 새 글자를 더해도 성소의 번호 있는 것들(상자·화로·숨은 물건·장치·시작·문지기)의 자리와 차례는
/// 그대로다 — 기록의 상자 번호·숨은 물건 이름·쉰 화로 번호가 가리키는 것이 바뀌지 않게.
#[test]
fn the_sanctuary_arrival_mark_keeps_the_numbering_of_everything_else() {
    let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
    let lv = &run.level;
    let at = lv.marks_of(ARRIVE_GLYPH).collect::<Vec<_>>();
    assert_eq!(at.len(), 1);
    let (x, z) = lv.cell_of(at[0]).unwrap();
    assert!(lv.walkable(x, z) && lv.cell(x, z + 1) == b'#', "south edge");
    let cells = |g: char| -> Vec<(usize, usize)> {
        lv.marks_of(g).map(|p| lv.cell_of(p).unwrap()).collect()
    };
    // 0262 전의 지도에서 잰 자리(위에서부터 차례 = 기록 번호).
    let before: [(char, &[(usize, usize)]); 10] = [
        ('C', &[(40, 17), (2, 18), (34, 29), (2, 49)]),
        ('R', &[(30, 14), (8, 47), (25, 59)]),
        ('F', &[(40, 26)]),
        ('H', &[(26, 1)]),
        ('K', &[(16, 1), (1, 32)]),
        ('M', &[(4, 17), (3, 47)]),
        ('O', &[(35, 22)]),
        ('V', &[(2, 13)]),
        ('W', &[(21, 6)]),
        ('D', &[(21, 3)]),
    ];
    for (g, want) in before {
        assert_eq!(cells(g), want.to_vec(), "{g}");
    }
    assert_eq!(lv.cell_of(lv.player_start), Some((24, 60)));
    run.arrive_from_prologue(&mut w);
    assert_eq!(w.player.pos, at[0]);
    assert_eq!(w.player.facing, std::f32::consts::PI);
    assert_eq!((run.intro, run.card.is_none()), (0.0, true));
}

/// 0267(사장 "화톳불이랑 그냥 조명 불이랑 구분이 안 됨"): 쉬는 곳(`R`)은 땅에 놓인 화톳불, 조명(`b`)은 받침 위의 화로 — 같은 자리에 둘이
/// 겹치지 않고, 쉬는 곳의 빛이 더 넓다.
#[test]
fn rest_hearths_and_lamps_are_different_props() {
    for def in STAGES.iter() {
        let (run, _) = StageRun::start(def);
        for r in &run.rests {
            let here: Vec<PropKind> = run
                .level
                .props
                .iter()
                .filter(|p| vec2(p.pos.x - r.x, p.pos.z - r.z).length() < 0.1)
                .map(|p| p.kind)
                .collect();
            assert_eq!(here, vec![PropKind::Hearth], "{}", def.find_prefix);
        }
        let hearths = run
            .level
            .props
            .iter()
            .filter(|p| p.kind == PropKind::Hearth)
            .count();
        assert_eq!(hearths, run.rests.len(), "{}", def.find_prefix);
    }
    assert!(HEARTH_LIGHT_R > 7.0);
}

/// 필드 보스(0270 D)를 찾는다.
fn field_boss(w: &World) -> Option<&crate::enemy::Enemy> {
    w.enemies.iter().find(|e| e.field)
}

/// 0270 D: 성소에만 필드 보스가 있다 — 지도의 `Q` 자리에 엎드려 기다린다(잠든 채: 맞지도 락온에 잡히지도 않는다). 다른 지역에는
/// 표도 지도 글자도 없다. 문지기의 셈(번호 0·깨어남·핵)은 그대로다.
#[test]
fn the_sanctuary_alone_fields_a_boss_lying_at_its_mark() {
    for (i, def) in STAGES.iter().enumerate() {
        let (run, w) = StageRun::start(def);
        let marks = run.level.marks_of(FIELD_GLYPH).count();
        assert_eq!(marks, def.field.is_some() as usize, "stage {i}");
        assert_eq!(def.field.is_some(), i == 0, "stage {i}");
        assert_eq!(field_boss(&w).is_some(), i == 0, "stage {i}");
        // 문지기는 그대로 번호 0(서막 숲에는 없다).
        if let Some(b) = run.boss {
            assert!(w.enemies[b].boss && !w.enemies[b].field);
        }
    }
    let (run, w) = StageRun::start(&sanctuary::DEF);
    let e = field_boss(&w).unwrap();
    let home = run.level.marks_of(FIELD_GLYPH).next().unwrap();
    assert_eq!(e.kind, sanctuary::DEF.field.as_ref().unwrap().kind);
    assert!(e.lying() && !e.active() && !e.lockable());
    assert!(e.pos.distance(home) < 1e-3 && e.home.is_some());
    assert!(run.boss_bar(&w).is_none(), "no bar while it lies");
}

/// 0270 D: 필드 보스의 터 — 넓고(돌진하는 2.5m 몸: 가운데 둘레 16 × 16m가 다 걷는 평평한 바닥), 길에서 벗어나 있고(진행의
/// 길목 — 시작·장치·문·문지기 — 에서 멀다), 화로 곁이 아니다. 닿을 수 있는지는 `every_floor_is_reachable…`가 본다.
#[test]
fn the_field_boss_waits_in_a_wide_flat_clearing_off_the_route_and_away_from_hearths() {
    let (run, _) = StageRun::start(&sanctuary::DEF);
    let lv = &run.level;
    let home = lv.marks_of(FIELD_GLYPH).next().unwrap();
    let g = lv.ground();
    let (cx, cz) = (
        (home.x / CELL + lv.w as f32 * 0.5 - 0.5).round() as i32,
        lv.row_of(home).round() as i32,
    );
    // 터의 네모: 보스 자리에서 가로·세로로 벽까지 — 그 안이 다 같은 높이의 걷는 바닥이고 16 × 16m(8 × 8칸)를 넘는다.
    let span = |from: (i32, i32), dx: i32, dz: i32| {
        let mut n = 0;
        while lv.walkable(
            (from.0 + dx * (n + 1)) as usize,
            (from.1 + dz * (n + 1)) as usize,
        ) {
            n += 1;
        }
        n
    };
    let (z0, z1) = (cz - span((cx, cz), 0, -1), cz + span((cx, cz), 0, 1));
    // 가로는 맨 윗줄에서 잰다(보스의 줄은 곁길로 이어져 길까지 뚫려 있다).
    let (x0, x1) = (cx - span((cx, z0), -1, 0), cx + span((cx, z0), 1, 0));
    assert!(
        x1 - x0 + 1 >= 8 && z1 - z0 + 1 >= 8,
        "{x0}..{x1} × {z0}..{z1}"
    );
    // 벽에서 떨어져 엎드린다(돌진이 등 뒤 벽에 바로 막히지 않게 — 사방 6m 넘게).
    assert!(cx - x0 >= 3 && x1 - cx >= 3 && cz - z0 >= 3 && z1 - cz >= 3);
    for z in z0..=z1 {
        for x in x0..=x1 {
            let (xu, zu) = (x as usize, z as usize);
            assert!(lv.walkable(xu, zu), "({x},{z}) is not floor");
            let p = lv.at(xu, zu);
            assert!(
                (g.height(p) - home.y).abs() < 0.01,
                "({x},{z}) is not level"
            );
            // 부딪히는 소품은 가장자리 줄에만(달릴 자리를 막지 않는다).
            let edge = x == x0 || x == x1 || z == z0 || z == z1;
            let blocked = lv.props.iter().any(|q| {
                q.kind.half() != Vec2::ZERO
                    && (q.pos.x - p.x).abs() < CELL * 0.5
                    && (q.pos.z - p.z).abs() < CELL * 0.5
            });
            assert!(edge || !blocked, "({x},{z}) holds a prop");
        }
    }
    let flat = |a: Vec3| vec2(a.x - home.x, a.z - home.z).length();
    for r in &run.rests {
        assert!(flat(*r) > 30.0, "a hearth {} m away", flat(*r));
    }
    // 길목에서 멀다: 깨어난 자리·문 장치·문·문지기 — 일어나는 거리(`FIELD_WAKE_R`)·끈(`FIELD_LEASH`)이 닿지 않는다.
    let mut route = vec![lv.player_start, lv.gate.unwrap().1];
    route.extend(run.lever);
    route.extend(lv.marks_of(sanctuary::DEF.boss.0));
    for p in route {
        assert!(flat(p) > crate::tuning::FIELD_LEASH, "{p:?}");
    }
    // 떼 마당(0244)도 터를 비킨다.
    for p in &run.horde_at {
        assert!(flat(*p) >= 16.0, "the horde stands in the stag's ground");
    }
}

/// 0270 D: 필드 보스는 그 판에 한 번 — 쓰러뜨리면 화로에서 쉬어도·쓰러져 다시 서도 다시 서지 않고, 다음 판(새로 세운 지역)에
/// 다시 엎드려 있다. 쓰러뜨리지 않고 쉬면 처음부터(가득 차 엎드린 채). 앱이 "이 판에 이미 쓰러뜨렸다"고 알려도 서지 않는다.
#[test]
fn a_downed_field_boss_stays_down_through_rests_this_run_and_waits_again_next_run() {
    let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
    let i = entrance_rest(&run);
    // 다친 채 깨어 있다가 쉬면: 가득 차 엎드린 채 다시.
    {
        let e = w.enemies.iter_mut().find(|e| e.field).unwrap();
        e.hp = e.max_hp / 2;
        e.state = EState::Chase;
    }
    rest_now(&mut run, &mut w, i);
    let e = field_boss(&w).expect("still there");
    assert!(e.lying() && e.hp == e.max_hp);
    // 쓰러뜨린다.
    w.enemies.iter_mut().find(|e| e.field).unwrap().state = EState::Dead;
    tick(&mut run, &mut w, Actions::default());
    assert!(run.field.down);
    assert!(run.take_field_down().is_some());
    assert!(run.take_field_down().is_none(), "told once");
    rest_now(&mut run, &mut w, i);
    assert!(field_boss(&w).is_none(), "resting does not raise it");
    // 쓰러져 쉼터에서 다시 서도.
    w.player.hp = 0;
    w.player.flask = 0;
    assert_eq!(
        tick(&mut run, &mut w, Actions::default()),
        StageOut::Respawned
    );
    assert!(field_boss(&w).is_none(), "nor does falling");
    // 엘리트·보통 적은 여느 때처럼 다시 섰다.
    assert_eq!(
        w.enemies.iter().filter(|e| e.elite.is_some()).count(),
        sanctuary::DEF.elites.len()
    );
    // 다음 판.
    let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
    assert!(field_boss(&w).is_some_and(|e| e.lying()));
    // 같은 판을 잇거나 지역에 다시 들어설 때(앱이 알린다): 서지 않는다 — 쉬어도.
    run.set_field_down(&mut w);
    assert!(field_boss(&w).is_none());
    rest_now(&mut run, &mut w, i);
    assert!(field_boss(&w).is_none());
}

/// 0270 D: 처음 쓰러뜨리면 잔불을 더 흩고 앱에 "처음"이라 알린다(확정 떨굼·장신구·기록) — 기록에 있으면 덤 없이 "다시".
#[test]
fn a_field_boss_pays_extra_and_reports_first_only_once() {
    let fell = |beaten: bool| {
        let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
        run.set_field_beaten(beaten);
        let at = {
            let e = w.enemies.iter_mut().find(|e| e.field).unwrap();
            e.state = EState::Dead;
            e.pos
        };
        w.drops.clear();
        tick(&mut run, &mut w, Actions::default());
        let extra = w.drops.iter().map(|d| d.value).sum::<u32>();
        let told = run.take_field_down().expect("told");
        assert_eq!(told.0, at);
        (told.1, extra)
    };
    let (first, with_bonus) = fell(false);
    let (again, plain) = fell(true);
    assert!(first && !again);
    assert_eq!(with_bonus - plain, crate::tuning::FIELD_FIRST_EMBERS);
    // 처음의 합(종의 잔불 + 덤)은 성소 문지기의 것보다 뚜렷이 많다.
    let stag = crate::species::def(Kind::Stag).embers;
    let warden = crate::species::def(Kind::Warden).embers;
    assert!(stag + crate::tuning::FIELD_FIRST_EMBERS >= warden * 2);
    assert!(stag < warden, "less on later runs");
}

/// 0270 D: 일어나기 시작하면 카드와 체력 띠(문지기와 같은 띠 — 이름은 그 보스), 다 서서 싸우는 동안에도. 제자리로 돌아가는
/// 동안·엎드린 뒤에는 띠가 없다. 일어나기 시작한 틱을 앱에 한 번 알린다(낮은 땅울림 — 종은 울리지 않는다).
#[test]
fn the_field_boss_shows_its_bar_from_rising_until_it_gives_up_or_falls() {
    let t = Tuning::default();
    let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
    run.intro = 0.0;
    let home = run.level.marks_of(FIELD_GLYPH).next().unwrap();
    w.player.pos = home + vec3(-6.0, 0.0, 0.0);
    w.player.prev = w.player.pos;
    w.unhurt = true;
    // (걸어서 왔다면 구역 카드는 이미 지났다.)
    run.first_card = true;
    run.zone = run.zone_at(w.player.pos);
    assert!(!run.take_field_woke());
    for _ in 0..3 {
        w.step(&Actions::default(), &t);
        run.tick(&mut w, 1.0 / 60.0);
    }
    assert!(field_boss(&w).unwrap().waking);
    assert!(run.take_field_woke() && !run.take_field_woke(), "told once");
    assert_eq!(
        run.card.map(|c| crate::i18n::tr(c.0)),
        Some(crate::i18n::tr(|s| s.stag_wakes))
    );
    let bar = run.bar_boss(&w).expect("a bar while it rises");
    assert_eq!(bar.kind, Kind::Stag);
    for _ in 0..(crate::tuning::FIELD_WAKE * 60.0) as usize + 5 {
        w.step(&Actions::default(), &t);
        run.tick(&mut w, 1.0 / 60.0);
    }
    assert!(field_boss(&w).unwrap().active());
    assert_eq!(run.boss_bar(&w).map(|b| b.0), Some(1.0));
    // 돌아가는 동안에는 띠가 없다.
    w.enemies.iter_mut().find(|e| e.field).unwrap().state = EState::Return;
    assert!(run.boss_bar(&w).is_none());
    // 쓰러지면 없다.
    w.enemies.iter_mut().find(|e| e.field).unwrap().state = EState::Dead;
    assert!(run.boss_bar(&w).is_none());
}

/// 문지기를 지우면(핵을 거둔 뒤) 뒤의 적 번호가 하나씩 당겨진다 — 엘리트의 번호도 같이 당겨야 한다: 안 그러면 엘리트 곁의
/// 보통 적이 쓰러질 때 엘리트가 쓰러진 것으로 셌다. 필드 보스(0270 D)는 문지기가 지워져도 남는다.
#[test]
fn taking_the_core_keeps_elite_numbers_and_the_field_boss() {
    let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
    run.open_down(&mut w);
    assert!(w.enemies.iter().all(|e| !e.boss || e.field));
    assert!(field_boss(&w).is_some_and(|e| e.lying()), "the stag stays");
    // 엘리트가 아닌 적을 다 쓰러뜨려도 엘리트는 쓰러진 것으로 세지 않는다.
    for e in w
        .enemies
        .iter_mut()
        .filter(|e| e.elite.is_none() && !e.boss)
    {
        e.state = EState::Dead;
    }
    tick(&mut run, &mut w, Actions::default());
    assert!(run.take_elites_down().is_empty());
    assert!(run.take_elites_new().is_empty());
    // 엘리트 하나를 쓰러뜨리면 그것만.
    let first = w.enemies.iter().position(|e| e.elite.is_some()).unwrap();
    w.enemies[first].state = EState::Dead;
    tick(&mut run, &mut w, Actions::default());
    assert_eq!(run.take_elites_down().len(), 1);
}

/// 0272 2단계: 파수꾼의 창은 떠난 마을의 셋째 상자 — 골목 서쪽 파수꾼의 곁방(본길에서 비켜 둔 곳). 옛 상자 둘의 기록 번호(위 → 아래
/// 차례)는 그대로다: 0 = 천 · 1 = 항아리 — 그 둘을 연 옛 기록은 창의 상자를 연 것으로 읽히지 않는다. 곁방은 걸어서 닿는다
/// (`every_floor_is_reachable_and_every_spawn_can_walk_to_the_fight`가 모든 바닥을 본다).
#[test]
fn the_watchmans_spear_waits_in_the_villages_third_chest() {
    use crate::charms::Id as Charm;
    let def = &village::DEF;
    assert_eq!(
        def.chest_loot,
        &[
            Loot::Charm(Charm::Cloth),
            Loot::Charm(Charm::Jar),
            Loot::Weapon(crate::weapons::Id::Spear),
            Loot::Weapon(crate::weapons::Id::Daggers),
        ]
    );
    let (mut run, _) = StageRun::start(def);
    assert_eq!(run.chest.at.len(), 4);
    let g = run.level.ground();
    let cell = |p: Vec3| g.cell_of(p).unwrap();
    assert_eq!(
        cell(run.chest.at[0]),
        (40, 23),
        "the old chests keep their place"
    );
    assert_eq!(cell(run.chest.at[1]), (41, 39));
    let (x, z) = cell(run.chest.at[2]);
    assert!(x < 8 && (39..47).contains(&z), "west of the alley: {x} {z}");
    run.set_opened(&[0, 1]);
    assert_eq!(
        run.chest.open,
        vec![true, true, false, false],
        "an old record"
    );
}

/// 0274 ③: 빈집의 쌍단검은 떠난 마을의 넷째 상자 — 거주지 동쪽 벽 너머 빈집의 뒷방(도착에서 가깝지만 본길에서 비켜 둔 곳). 옛 상자
/// 셋의 기록 번호(위 → 아래 차례)는 그대로다: 셋을 연 옛 기록은 쌍단검의 상자를 연 것으로 읽히지 않는다. 뒷방은 걸어서 닿고 거주지와
/// 같은 높이다(`every_floor_is_reachable_and_every_spawn_can_walk_to_the_fight`가 모든 바닥을 본다).
#[test]
fn the_empty_house_daggers_wait_in_the_villages_fourth_chest() {
    let def = &village::DEF;
    assert_eq!(
        def.chest_loot.last(),
        Some(&Loot::Weapon(crate::weapons::Id::Daggers))
    );
    let (mut run, _) = StageRun::start(def);
    assert_eq!(run.chest.at.len(), 4);
    let g = run.level.ground();
    let cell = |p: Vec3| g.cell_of(p).unwrap();
    assert_eq!(
        cell(run.chest.at[2]),
        (3, 42),
        "the spear's chest keeps its place"
    );
    let (x, z) = cell(run.chest.at[3]);
    assert!(
        x > 38 && (52..59).contains(&z),
        "east of the homes: {x} {z}"
    );
    run.set_opened(&[0, 1, 2]);
    assert_eq!(
        run.chest.open,
        vec![true, true, true, false],
        "an old record"
    );
}

/// 0274 ④: 가지치기 대검은 잔뿌리의 정원의 셋째 상자 — 서쪽 샛길의 막다른 끝(되풀이 꼬투리 Y가 지키는 곳, 본길에서 비켜 둔 보상).
/// 옛 상자 둘(장신구 · 잔뿌리 활)의 기록 번호(위 → 아래 차례)는 그대로다: 둘을 연 옛 기록은 대검의 상자를 연 것으로 읽히지 않는다.
#[test]
fn the_pruners_greatsword_waits_in_the_gardens_third_chest() {
    let def = &garden::DEF;
    assert_eq!(
        def.chest_loot,
        &[
            Loot::Charm(crate::charms::Id::Feet),
            Loot::Weapon(crate::weapons::Id::Bow),
            Loot::Weapon(crate::weapons::Id::Greatsword),
        ]
    );
    let (mut run, _) = StageRun::start(def);
    assert_eq!(run.chest.at.len(), 3);
    let g = run.level.ground();
    let cell = |p: Vec3| g.cell_of(p).unwrap();
    assert_eq!(
        cell(run.chest.at[0]),
        (3, 23),
        "the charm's chest keeps its place"
    );
    assert_eq!(
        cell(run.chest.at[1]),
        (40, 34),
        "the bow's chest keeps its place"
    );
    assert_eq!(
        cell(run.chest.at[2]),
        (3, 44),
        "the end of the west side path"
    );
    run.set_opened(&[0, 1]);
    assert_eq!(run.chest.open, vec![true, true, false], "an old record");
}
