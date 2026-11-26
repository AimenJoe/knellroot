//! 스킬 트리의 셈(`Board`) 시험 — 옛 `skills.rs`의 시험 모듈에서 옮김(0278 R25b).
use super::*;
use crate::progression::{Branch, Cell, Key, Link, Tree, Vein, VEINS};

/// 0161: 기술 점수는 셈 — 총 레벨 + 처음 쓰러뜨린 엘리트 + 핵(보스마다 셋까지) + 종소리 조각, 남은 점수 = 그것 − 쓴 점수.
/// 화폐 "마디"(핵 + 조각)와 0158의 "보스 첫 쓰러뜨림 +1"을 이 하나가 대신한다. 처음부터 가진 단(큰 기술 1단)은 쓴 점수가 아니다.
/// 0166 → 0167: 스킬은 값 없이 주지 않고, 새 판은 점수도 없이 선다 — 첫 점수는 첫 레벨에서.
#[test]
fn points_are_levels_plus_first_elites_plus_cores_plus_shards_minus_spent() {
    assert_eq!(points(0, 0, 0, 0), 0, "0167: a new run starts with none");
    assert_eq!(points(15, 0, 0, 0), 15);
    assert_eq!(points(0, 6, 0, 0), 6);
    assert_eq!(
        points(0, 0, 6, 0),
        6,
        "every core counts, not just the first"
    );
    assert_eq!(points(0, 0, 0, 3), 3);
    assert_eq!(
        points(15, 6, 6, 3),
        30,
        "0161: everything in the game today"
    );
    let mut b = Board::default();
    assert_eq!(b.rank, [0; N], "0166: no skill is given");
    assert_eq!(
        (b.ult, b.spent()),
        (ult_ranks([]), 0),
        "0172: no great skill is given either"
    );
    let total = points(3, 1, 1, 0);
    assert_eq!(b.left(total), 5);
    assert!(b.buy(Entry::Skill(Id::Gather), total));
    assert!(b.buy(Entry::Skill(Id::Whirl), total));
    assert!(b.buy(Entry::Cell(Cell::UnbrokenStride), total));
    assert_eq!((b.spent(), b.left(total)), (3, 2));
    // 가진 것보다 많이 쓴 판(손댄 기록)의 남은 점수는 0에서 멈춘다.
    assert_eq!(Board::maxed(Pick::A).left(3), 0);
}

/// 0161 "지금 내용으로 15 + 6 + 6 + 3 = 30": 레벨 셋 × 다섯 단, 스테이지 표의 엘리트 자리, 문지기 둘 × 핵 셋, 지도에 놓인 종소리 조각.
#[test]
fn todays_content_gives_thirty_points() {
    use crate::progression::{CORE_CAP, LINES, MAX_RANK as LEVEL_RANKS};
    let levels = LINES.len() as u32 * LEVEL_RANKS as u32;
    let stages = crate::stage::STAGES;
    let elites: u32 = stages.iter().map(|d| d.elites.len() as u32).sum();
    let cores = crate::stage::REGIONS as u32 * CORE_CAP;
    let shards: u32 = stages
        .iter()
        .map(|d| {
            let (run, _) = crate::stage::StageRun::start(d);
            run.hidden
                .list
                .iter()
                .filter(|f| f.1 == crate::stage::Find::Shard)
                .count() as u32
        })
        .sum();
    // 0185: 셋째 지역(엘리트 셋·핵 셋·조각 둘)이 더해져 30 → 38. 0208: 넷째 지역(엘리트 셋·핵 셋·조각 하나) → 45.
    assert_eq!((levels, elites, cores, shards), (15, 12, 12, 6));
    let all = points(levels, elites, cores, shards);
    assert_eq!(all, 45);
    // 다 익히는 값(65)에는 못 미친다 — 고르게 한다(0160 2절). 여정의 기술 점수(지역마다 하나, 0183)를 더해도.
    assert!(all + (crate::stage::REGIONS as u32) < Board::maxed(Pick::A).spent());
}

/// 0161 → 0215 → 0270 C3: 패시브 칸은 판의 칸이다 — 칸마다 1점. 줄(쓴 점수 합)이 "언제"를, 길(바로 위 줄의 이어진 칸 —
/// 몸의 판의 칸끼리)이 "어디로"를 정한다. 첫 줄 패시브는 처음부터 산다(스킬이 무기 판으로 옮겨 갔다). 연 칸은 다시 못 산다.
#[test]
fn passive_cells_cost_one_point_and_are_gated_by_rows_and_paths() {
    let total = 30;
    let mut b = Board::default();
    let (second, third, fifth) = (
        Entry::Cell(Cell::FlowingBeat),
        Entry::Cell(Cell::ClosingBlow),
        Entry::Cell(Cell::FinalMeasure),
    );
    assert_eq!((second.price(0), second.max(), second.free()), (1, 1, 0));
    // 첫 줄 패시브는 뿌리 — 스킬 없이도 산다(0270 C3).
    assert_eq!(b.offer(second, 0), Offer::Short);
    assert_eq!(b.offer(second, total), Offer::Ready);
    // 갈래의 첫 칸(고른 호흡)을 열지 않고 둘째 칸부터.
    assert!(b.buy(second, total));
    assert!(b.cells.has(Cell::FlowingBeat) && !b.cells.has(Cell::EvenBreath));
    assert_eq!((b.spent(), b.rank(second)), (1, 1));
    assert_eq!(b.offer(second, total), Offer::Full);
    assert!(!b.buy(second, total), "an open cell is not bought twice");
    // 둘째 줄: 쓴 점수 4부터(지금 1 — 무기 판에 쓴 점수도 센다).
    assert_eq!(b.offer(third, total), Offer::Locked(3));
    assert_eq!(b.offer(fifth, total), Offer::Locked(7));
    for _ in 0..3 {
        assert!(b.buy(Entry::Skill(Id::Whirl), total));
    }
    // 줄이 열려도 길이 있어야: 맺음타(왼쪽)는 고른 호흡(왼쪽) 밑 — 흐르는 박자(오른쪽)로는 안 이어진다.
    assert_eq!(b.offer(third, total), Offer::Cut);
    assert_eq!(b.offer(Entry::Cell(Cell::GapBeat), total), Offer::Ready);
    assert!(b.buy(Entry::Cell(Cell::EvenBreath), total));
    assert_eq!(b.offer(third, total), Offer::Ready);
    // 넷째 칸을 셋째 칸보다 먼저(같은 줄).
    assert!(b.buy(Entry::Cell(Cell::GapBeat), total));
    assert!(!b.cells.has(Cell::ClosingBlow));
    // 다른 갈래에 쓴 점수도 줄을 연다: 쓴 점수 8 — 셋째 줄.
    for e in [Entry::Skill(Id::Brace), Entry::Cell(Cell::Catch)] {
        assert_eq!(b.offer(fifth, total), Offer::Locked(8 - b.spent()));
        assert!(b.buy(e, total));
    }
    assert_eq!(b.spent(), 8);
    // 마지막 박자(그 줄에 홀로 선 칸)는 끝 틈 박자 밑.
    assert!(b.buy(fifth, total));
    assert!(b.cells.has(Cell::FinalMeasure));
    // 점수가 떨어지면 열린 줄의 칸도 못 연다.
    assert_eq!(
        b.offer(Entry::Cell(Cell::Sidestep), b.spent()),
        Offer::Short
    );
    assert!(b.can_buy(total) && !b.can_buy(b.spent()));
}

/// 0215 B → 0270 C3: 판의 길(몸의 판의 칸끼리 — 패시브·다리·맺음). 갈래 안에서는 바로 아래 줄의 가로로 가까운 칸만
/// 이어진다(왼쪽 ↔ 오른쪽은 아니다 — 그 줄에 홀로 선 칸은 갈래의 칸 모두와), 갈래가 다르면 다리 칸만 잇는다. 다리는 같은 줄의
/// 맞닿은 칸에서 들어서고, 들어서면 양쪽 갈래의 바로 아래 줄로 간다. 스킬·큰 기술·결은 길 밖이다(무기 판의 칸). 이미 가진 칸은
/// 길을 묻지 않는다(옛 기록·단 올리기).
#[test]
fn the_board_has_paths_and_bridges_join_neighbouring_branches() {
    use Entry::{Cell as C, Key as K, Link as L, Skill as S, Ult as U};
    // 판의 줄 아홉(화면의 자리), 몸의 길의 줄 다섯.
    let lines: Vec<usize> = ENTRIES.iter().map(|e| e.line()).collect();
    assert_eq!((0..9).filter(|l| lines.contains(l)).count(), 9, "{lines:?}");
    assert!(lines.iter().all(|&l| l < 9));
    let paths: Vec<usize> = ENTRIES.iter().filter_map(|e| e.path_line()).collect();
    assert_eq!((0..5).filter(|l| paths.contains(l)).count(), 5, "{paths:?}");
    // 첫 줄 패시브 말고 몸의 판의 모든 칸은 위에서 들어서는 길이 있다(갇힌 칸이 없다) — 스킬·큰 기술·결의 칸은 길 밖이다.
    for &e in &ENTRIES {
        if !e.on_path() {
            assert!(!ENTRIES.iter().any(|&f| f.feeds(e) || e.feeds(f)), "{e:?}");
            assert!(Board::default().reachable(e) || matches!(e, Entry::Vein(_)));
            continue;
        }
        assert!(
            e.path_line() == Some(0) || ENTRIES.iter().any(|&f| f.feeds(e)),
            "{e:?} has no way in"
        );
    }
    // 갈래 안: 왼쪽 ↔ 오른쪽은 안 이어지고, 홀로 선 칸은 둘 다와.
    assert!(
        C(Cell::EvenBreath).feeds(C(Cell::ClosingBlow))
            && !C(Cell::EvenBreath).feeds(C(Cell::GapBeat))
    );
    assert!(
        C(Cell::ClosingBlow).feeds(C(Cell::FinalMeasure))
            && C(Cell::GapBeat).feeds(C(Cell::FinalMeasure))
    );
    assert!(
        C(Cell::FinalMeasure).feeds(C(Cell::HeldBreath))
            && C(Cell::FinalMeasure).feeds(C(Cell::GatheringBeat))
    );
    assert!(C(Cell::HeldBreath).feeds(K(Key::Measure)));
    assert!(
        !C(Cell::ClosingBlow).feeds(C(Cell::EvenBreath)),
        "paths only go down"
    );
    assert!(
        !S(Id::Whirl).feeds(C(Cell::EvenBreath)),
        "skills are off the path"
    );
    assert!(!U(Ult::Flurry).feeds(C(Cell::HeldBreath)));
    // 갈래가 달라 가까워도 안 이어진다 — 다리만.
    assert!(!C(Cell::GapBeat).feeds(C(Cell::GatheredRiposte)));
    assert!(C(Cell::GapBeat).feeds(L(Link::RiposteBeat)));
    assert!(C(Cell::GatheredRiposte).feeds(L(Link::RiposteBeat)));
    assert!(L(Link::RiposteBeat).feeds(C(Cell::FinalMeasure)));
    assert!(L(Link::ParryStep).feeds(C(Cell::TiltingGround)));
    // 흐름 ↔ 자리는 맞닿지 않는다(가장 먼 혼종은 비싸다).
    for &e in &ENTRIES {
        if e.branch() == Branch::Footing && !matches!(e, Entry::Link(_)) {
            assert!(!L(Link::RiposteBeat).feeds(e), "{e:?}");
        }
    }
    // 다리를 건너: 흐름만 걸은 판이 맞받기의 둘째 줄 칸을 산다.
    let total = 40;
    let mut b = Board::default();
    for e in [
        S(Id::Cut),
        S(Id::Cut),
        S(Id::Cut),
        C(Cell::FlowingBeat),
        C(Cell::GapBeat),
    ] {
        assert!(b.buy(e, total), "{e:?}");
    }
    let breach = C(Cell::GatheredRiposte);
    assert_eq!(b.offer(breach, total), Offer::Cut);
    assert_eq!(b.offer(L(Link::RiposteBeat), total), Offer::Ready);
    assert!(b.buy(L(Link::RiposteBeat), total));
    assert_eq!(b.offer(breach, total), Offer::Ready);
    // 다리 칸은 어느 갈래의 점수도 아니다.
    assert_eq!(
        (
            b.branch_spent(Branch::Beat),
            b.branch_spent(Branch::Riposte)
        ),
        (5, 0)
    );
    // 가진 칸은 길을 묻지 않는다(옛 기록 — 길이 끊긴 채 산 칸).
    let mut old = Board::default();
    old.cells.open_cell(Cell::HeldBreath);
    assert!(old.reachable(C(Cell::HeldBreath)));
    assert!(!old.reachable(C(Cell::GatheringBeat)));
}

/// 0215 A: 맺음은 그 갈래에 쓴 점수 `KEY_BRANCH` 10부터 — 줄(22)이 열리고 길이 이어져도. 다른 맺음으로 바꿀 때도 같다.
#[test]
fn a_keystone_needs_ten_points_in_its_own_branch() {
    use Entry::{Cell as C, Skill as S, Ult as U};
    assert_eq!(KEY_BRANCH, 10);
    let total = 40;
    let mut b = Board::default();
    // 맞받기의 길(휘돌이 · 가로지르기 · 모으기는 줄만 연다 — 0270 C3: 길은 몸의 판의 칸끼리): 받아내기 → 모은 맞받기 →
    // 서 있는 자 → 되찾는 생기, 그 갈래의 점수는 막아서기·되울림·줄 긋기까지.
    for e in [[S(Id::Whirl); 5], [S(Id::Cut); 5]].concat() {
        assert!(b.buy(e, total));
    }
    for e in [
        S(Id::Brace),
        C(Cell::Catch),
        S(Id::Echo),
        C(Cell::GatheredRiposte),
        U(Ult::Line),
        C(Cell::OneWhoStands),
        C(Cell::Mend),
        S(Id::Gather),
        S(Id::Gather),
        S(Id::Gather),
        S(Id::Gather),
    ] {
        assert!(b.buy(e, total), "{e:?}");
    }
    assert_eq!(b.spent(), 22);
    assert_eq!(b.branch_spent(Branch::Riposte), 8);
    let edge = Entry::Key(Key::Edge);
    assert_eq!(b.offer(edge, total), Offer::Branch(2));
    // 흐름은 10이지만 길이 없다 — 흐름의 넷째 줄 패시브가 없다.
    assert_eq!(b.branch_spent(Branch::Beat), 10);
    assert_eq!(b.offer(Entry::Key(Key::Measure), total), Offer::Cut);
    assert!(b.buy(S(Id::Brace), total) && b.buy(S(Id::Brace), total));
    assert_eq!(b.offer(edge, total), Offer::Ready);
    assert!(b.buy(edge, total));
    // 바꾸기도 그 갈래에 10을 써 두어야(자리는 6 — 길도 없다).
    let rooted = Entry::Key(Key::Rooted);
    assert_eq!(b.offer(rooted, total), Offer::Cut);
}

/// 0215 D: 숫자 칸 셋은 둘 가운데 하나 — 열면 I, 값 없이 뒤집는다(연 칸만). 기록 꼴로 왕복하고, 열리지 않은 칸의 고르기는
/// 바로잡을 때 지운다.
#[test]
fn number_cells_pick_one_of_two_and_flip_for_free() {
    use crate::progression::TREE;
    let numbers: Vec<Cell> = TREE
        .iter()
        .flat_map(|r| r.iter().filter(|d| d.number).map(|d| d.cell))
        .collect();
    assert_eq!(
        numbers,
        [Cell::EvenBreath, Cell::Catch, Cell::UnbrokenStride]
    );
    for row in &TREE {
        for d in row {
            assert_eq!(d.cell.pick_b().is_some(), d.number, "{:?}", d.cell);
        }
    }
    let mut b = Board::default();
    assert!(!b.flip_cell(Cell::Catch), "not open yet");
    assert!(b.buy(Entry::Skill(Id::Brace), 9) && b.buy(Entry::Cell(Cell::Catch), 9));
    assert!(b.cells.picked_a(Cell::Catch) && !b.cells.picked_b(Cell::Catch));
    let spent = b.spent();
    assert!(b.flip_cell(Cell::Catch));
    assert!(b.cells.picked_b(Cell::Catch) && !b.cells.picked_a(Cell::Catch));
    assert_eq!(b.spent(), spent, "free");
    assert!(b.cells.has(Cell::Catch), "still open");
    assert!(!b.flip_cell(Cell::TurnItBack), "not a number cell");
    // 바로잡기: 닫힌 칸의 고르기 비트는 지운다.
    let mut odd = b.cells;
    odd.picks[0] = 0xff;
    assert_eq!(odd.sane().picks[0], 0);
    assert_eq!(odd.sane().picks[1], b.cells.picks[1]);
}

/// 0161 "갈래와 줄"의 표 그대로: 줄마다 갈래마다 선 칸(스킬·큰 기술·패시브). 판의 칸 순서는 줄 → 갈래 → 스킬 먼저.
#[test]
fn the_board_stands_as_the_0161_table_says() {
    use Branch::{Beat, Footing, Riposte};
    use Entry::{Cell as C, Link as L, Skill as S, Ult as U};
    let table: [(usize, Branch, &[Entry]); 51] = [
        (
            0,
            Beat,
            &[
                S(Id::Whirl),
                S(Id::Cut),
                C(Cell::EvenBreath),
                C(Cell::FlowingBeat),
            ],
        ),
        (
            0,
            Riposte,
            &[S(Id::Brace), C(Cell::Catch), C(Cell::TurnItBack)],
        ),
        (
            0,
            Footing,
            &[S(Id::Gather), C(Cell::UnbrokenStride), C(Cell::Sidestep)],
        ),
        // 0270 C3: 무기 판을 채운 새 스킬 — 옛 판의 자리는 그 줄의 끝.
        (0, Beat, &[S(Id::Stitch)]),
        (0, Riposte, &[S(Id::Toll)]),
        (0, Beat, &[S(Id::Swat)]),
        (0, Footing, &[S(Id::Flare), S(Id::Snare), S(Id::Fuse)]),
        // 0272 2단계: 창 판.
        (0, Beat, &[S(Id::Skewer)]),
        (0, Riposte, &[S(Id::Pin)]),
        // 0274: 한손검 판의 전투 기술.
        (0, Riposte, &[S(Id::Pommel)]),
        // 0274 ④: 대검 판.
        (0, Beat, &[S(Id::Sear)]),
        // 0274 ⑤: 낫 판.
        (0, Footing, &[S(Id::Reap)]),
        // 0274 ⑥: 완드 판.
        (0, Riposte, &[S(Id::Hex)]),
        (
            1,
            Beat,
            &[S(Id::Quake), C(Cell::ClosingBlow), C(Cell::GapBeat)],
        ),
        (
            1,
            Riposte,
            &[S(Id::Echo), C(Cell::GatheredRiposte), C(Cell::OpenBreach)],
        ),
        (
            1,
            Footing,
            &[
                S(Id::Throw),
                S(Id::Embers),
                C(Cell::TiltingGround),
                C(Cell::LandingBeat),
            ],
        ),
        // 다리 칸(0215): 잇는 둘 가운데 왼쪽 갈래로 센다.
        (1, Beat, &[L(Link::RiposteBeat)]),
        (1, Riposte, &[L(Link::ParryStep)]),
        (1, Riposte, &[S(Id::Ram), S(Id::Ripple)]),
        (1, Footing, &[S(Id::Snow), S(Id::Mine)]),
        (1, Footing, &[S(Id::Pole)]),
        (1, Footing, &[S(Id::Shove)]),
        (1, Riposte, &[S(Id::Rally)]),
        // 0274 ③: 쌍단검 판.
        (1, Beat, &[S(Id::Slip)]),
        (1, Footing, &[S(Id::Sunder)]),
        (1, Riposte, &[S(Id::Harvest)]),
        (1, Footing, &[S(Id::Blight)]),
        (2, Beat, &[U(Ult::Flurry), C(Cell::FinalMeasure)]),
        (2, Riposte, &[U(Ult::Line), C(Cell::OneWhoStands)]),
        (2, Footing, &[U(Ult::Upend), C(Cell::PathOpeningStep)]),
        (2, Riposte, &[U(Ult::Bell)]),
        (2, Footing, &[U(Ult::Blizzard), U(Ult::Rain)]),
        (2, Riposte, &[U(Ult::Gale)]),
        (2, Beat, &[U(Ult::Ambush)]),
        (2, Beat, &[U(Ult::Tempest)]),
        (2, Footing, &[U(Ult::Grove)]),
        (2, Riposte, &[U(Ult::Price)]),
        (
            3,
            Beat,
            &[S(Id::Swallow), C(Cell::HeldBreath), C(Cell::GatheringBeat)],
        ),
        (
            3,
            Riposte,
            &[S(Id::Bash), C(Cell::SecondWind), C(Cell::Mend)],
        ),
        (
            3,
            Footing,
            &[S(Id::Roots), C(Cell::Planted), C(Cell::Snare)],
        ),
        (3, Beat, &[L(Link::BreakingBeat)]),
        (3, Riposte, &[L(Link::RootedRiposte)]),
        (4, Beat, &[Entry::Key(Key::Measure)]),
        (4, Riposte, &[Entry::Key(Key::Edge)]),
        (4, Footing, &[Entry::Key(Key::Rooted)]),
        // 결의 줄(0223 11절): 타입 넷 — 빛깔만 그 타입의 스킬이 많은 갈래에 기댄다.
        (
            5,
            Footing,
            &[Entry::Vein(Vein::Ember), Entry::Vein(Vein::Root)],
        ),
        (5, Riposte, &[Entry::Vein(Vein::Knell)]),
        (5, Beat, &[Entry::Vein(Vein::Rift)]),
        (
            5,
            Beat,
            &[Entry::Vein(Vein::Shock), Entry::Vein(Vein::Sharp)],
        ),
        (5, Footing, &[Entry::Vein(Vein::Frost)]),
        (5, Riposte, &[Entry::Vein(Vein::Drain)]),
    ];
    let flat: Vec<Entry> = table
        .iter()
        .flat_map(|(_, _, es)| es.iter().copied())
        .collect();
    assert_eq!(flat, ENTRIES, "ENTRIES is the table read row by row");
    for (row, branch, es) in table {
        for &e in es {
            assert_eq!((e.row(), e.branch()), (row, branch), "{e:?}");
        }
    }
    assert_eq!(NE, 81);
    for (i, e) in ENTRIES.iter().enumerate() {
        assert!(!ENTRIES[i + 1..].contains(e), "{e:?} twice");
    }
    // 모든 스킬·큰 기술·패시브 칸이 판에 있다.
    for id in IDS {
        assert!(ENTRIES.contains(&S(id)), "{id:?}");
    }
    for u in ULTS {
        assert!(ENTRIES.contains(&U(u)), "{u:?}");
    }
    for row in &crate::progression::TREE {
        for d in row {
            assert!(ENTRIES.contains(&C(d.cell)), "{:?}", d.cell);
        }
    }
}

/// 0158 다: 단은 차례로 하나씩·단마다 1점, 다섯에서 멈춘다. 점수가 떨어지면 못 산다. 3단을 사면 고르기는 ㄱ.
#[test]
fn ranks_are_bought_in_order_one_point_each_and_stop_at_five() {
    let mut b = Board::default();
    let whirl = Entry::Skill(Id::Whirl);
    assert_eq!(b.offer(whirl, 0), Offer::Short);
    assert!(!b.buy(whirl, 0));
    for want in 1..=MAX_RANK {
        let spent = b.spent();
        assert_eq!(b.offer(whirl, 5), Offer::Ready);
        assert!(b.buy(whirl, 5));
        assert_eq!(b.rank(whirl), want);
        assert_eq!(b.spent(), spent + 1, "one point per rank");
    }
    assert_eq!(
        b.pick[Id::Whirl.index()],
        Pick::A,
        "rank 3 starts on pick a"
    );
    assert_eq!(b.offer(whirl, 99), Offer::Full);
    assert!(!b.buy(whirl, 99));
    assert_eq!(b.left(5), 0);
    assert_eq!(b.offer(Entry::Skill(Id::Cut), 5), Offer::Short);
    assert!(!b.can_buy(5) && b.can_buy(6));
    // 안 배운 스킬의 첫 점 = 배움(1단).
    let mut b = Board::default();
    assert!(!b.learned(Id::Gather));
    assert!(b.buy(Entry::Skill(Id::Gather), 1));
    assert!(b.learned(Id::Gather));
    assert_eq!((b.rank(Entry::Skill(Id::Gather)), b.spent()), (1, 1));
}

/// 0223 11절: 결의 칸은 그 타입을 싣는 스킬을 배워야 열린다(쓴 점수·판의 길과 상관없이), 값은 1점, 어느 갈래의 점수도
/// 아니고(맺음의 문턱을 당기지 않는다), 되돌리면 닫힌다. 타입마다 그 타입의 스킬이 판에 있다(열 수 없는 결은 없다).
#[test]
fn a_vein_opens_once_a_skill_of_its_type_is_known_and_counts_for_no_branch() {
    use crate::progression::BRANCHES;
    let total = 30;
    let mut b = Board::default();
    for v in VEINS {
        assert_eq!(b.offer(Entry::Vein(v), total), Offer::Cut, "{v:?}");
        assert!(
            IDS.into_iter().any(|id| id.def().kind == v.kind()),
            "{v:?} has a skill"
        );
    }
    assert_eq!(Id::Cut.def().kind, Vein::Rift.kind());
    assert!(b.buy(Entry::Skill(Id::Cut), total));
    let rift = Entry::Vein(Vein::Rift);
    assert_eq!(b.offer(rift, total), Offer::Ready);
    assert_eq!(
        b.offer(rift, 1),
        Offer::Short,
        "the point went on the skill"
    );
    assert_eq!(b.offer(Entry::Vein(Vein::Ember), total), Offer::Cut);
    let (spent, by_branch) = (b.spent(), BRANCHES.map(|x| b.branch_spent(x)));
    assert!(b.buy(rift, total));
    assert!(b.cells.veined(Vein::Rift) && !b.cells.veined(Vein::Ember));
    assert_eq!(b.spent(), spent + 1);
    assert_eq!(BRANCHES.map(|x| b.branch_spent(x)), by_branch);
    assert_eq!(b.offer(rift, total), Offer::Full);
    assert!(!b.buy(rift, total));
    let mut great = Ult::Upend;
    b.respec(&mut great);
    assert!(!b.cells.veined(Vein::Rift));
    assert_eq!(b.spent(), 0);
}

/// 0158 라: 줄은 **쓴 점수의 합**으로 열린다 — 둘째 줄(땅울림) 4, 큰 기술 줄 8. 어디에 썼든 상관없다. 잠긴 칸은 점수가 있어도 못 산다.
#[test]
fn rows_open_by_points_spent_wherever_they_went() {
    // 끝의 0은 결의 줄(0223 11절) — 점수가 아니라 그 타입의 스킬이 연다.
    assert_eq!(ROW_NEEDS, [0, 4, 8, 14, 22, 0]);
    for id in [Id::Whirl, Id::Cut, Id::Gather, Id::Brace] {
        assert_eq!(Entry::Skill(id).row(), 0, "{id:?}");
    }
    for id in [Id::Quake, Id::Echo, Id::Throw, Id::Embers] {
        assert_eq!(Entry::Skill(id).row(), 1, "{id:?}");
    }
    for u in ULTS {
        assert_eq!(Entry::Ult(u).row(), ULT_ROW as usize);
    }
    let (quake, ult) = (Entry::Skill(Id::Quake), Entry::Ult(Ult::Flurry));
    let total = 23;
    let mut b = Board::default();
    assert_eq!(b.offer(quake, total), Offer::Locked(4));
    assert_eq!(b.offer(ult, total), Offer::Locked(8));
    assert!(!b.buy(quake, total) && !b.buy(ult, total));
    // 셋을 한 스킬에 쓰고 패시브 하나(0270 C3 — 몸의 판의 첫 줄): 쓴 점수 넷이면 열린다.
    for n in 1..=3 {
        assert!(b.buy(Entry::Skill(Id::Cut), total));
        assert_eq!(b.offer(quake, total), Offer::Locked(4 - n));
    }
    assert!(b.buy(Entry::Cell(Cell::FlowingBeat), total));
    assert_eq!(b.offer(quake, total), Offer::Ready);
    assert_eq!(b.offer(ult, total), Offer::Locked(4));
    assert!(b.buy(quake, total) && b.buy(quake, total));
    // 다른 갈래에 쓴 점수도 줄을 연다.
    assert!(b.buy(Entry::Skill(Id::Gather), total));
    assert_eq!(b.offer(ult, total), Offer::Locked(1));
    assert!(b.buy(Entry::Cell(Cell::GapBeat), total));
    assert_eq!(b.spent(), 8);
    assert_eq!(b.offer(ult, total), Offer::Ready);
    // 큰 기술: 첫 단 2점, 그 뒤 1점씩 — 넷.
    assert!(b.buy(ult, total) && b.buy(ult, total) && b.buy(ult, total));
    assert_eq!(b.spent(), 12);
    assert_eq!(
        (b.ult[Ult::Flurry.index()], b.offer(ult, total)),
        (ULT_MAX_RANK, Offer::Full)
    );
    // 다 익히는 데 드는 점수(0158 라 → 0161 → 0166 → 0172 → 0214): 스킬 열하나 × 5 = 55(처음의 둘도 값이 있다), 큰 기술 셋 × 4 = 12
    // (쟁반 뒤집기도 배운다), 패시브 스물하나 × 1 = 21, 맺음 하나 1 — 89.
    let all = Board::maxed(Pick::A);
    let paid = |pred: fn(&Entry) -> bool| -> u32 {
        ENTRIES
            .iter()
            .filter(|e| pred(e))
            .map(|&e| e.paid(all.rank(e)))
            .sum()
    };
    // 0270 C3: 스킬·큰 기술은 무기 판을 채우며 늘었다 — 스킬마다 5, 큰 기술마다 4.
    assert_eq!(
        paid(|e| matches!(e, Entry::Skill(_))),
        N as u32 * MAX_RANK as u32
    );
    assert_eq!(paid(|e| matches!(e, Entry::Ult(_))), NU as u32 * 4);
    assert_eq!(paid(|e| matches!(e, Entry::Cell(_))), 21);
    assert_eq!(paid(|e| matches!(e, Entry::Key(_))), 1, "one keystone");
    assert_eq!(
        paid(|e| matches!(e, Entry::Link(_))),
        4,
        "0215: four bridges"
    );
    assert_eq!(
        paid(|e| matches!(e, Entry::Vein(_))),
        VEINS.len() as u32,
        "0223: a vein per type"
    );
    assert_eq!(
        all.spent(),
        (N * MAX_RANK as usize + NU * 4 + 21 + 1 + 4 + VEINS.len()) as u32
    );
    // 판의 칸 순서: 줄 번호가 내려가지 않는다.
    let rows: Vec<usize> = ENTRIES.iter().map(|e| e.row()).collect();
    assert!(rows.windows(2).all(|w| w[0] <= w[1]), "{rows:?}");
}

/// 0214: 맺음은 다섯째 줄(쓴 점수 22) — 1점에 하나를 켜고, 켠 뒤에는 다른 맺음으로 값 없이 바꾼다(하나만). 바꾸기는 "살 것"이
/// 아니다(쉼터가 빛나지 않는다). 되돌리면 꺼진다. 옛 꼴의 수는 넷째 줄 칸을 열지 않는다(`LEGACY_DEPTH`).
#[test]
fn only_one_keystone_is_lit_and_swapping_it_is_free() {
    let total = 40;
    let mut b = Board::default();
    let (measure, edge) = (Entry::Key(Key::Measure), Entry::Key(Key::Edge));
    assert_eq!(b.offer(measure, total), Offer::Locked(22));
    // 넷째 줄 패시브는 14부터.
    assert_eq!(b.offer(Entry::Cell(Cell::Mend), total), Offer::Locked(14));
    // 흐름의 길(0215 → 0270 C3 몸의 판): 고른 호흡 → 맺음타 → 마지막 박자 → 이어지는 숨(스킬·큰 기술은 줄만 연다).
    let flow = [
        [Entry::Skill(Id::Whirl); 5].as_slice(),
        &[Entry::Skill(Id::Cut); 5],
        &[
            Entry::Cell(Cell::EvenBreath),
            Entry::Skill(Id::Quake),
            Entry::Cell(Cell::ClosingBlow),
            Entry::Ult(Ult::Flurry),
            Entry::Cell(Cell::FinalMeasure),
            Entry::Cell(Cell::HeldBreath),
        ],
    ]
    .concat();
    for e in flow {
        assert!(b.buy(e, total), "{e:?}");
    }
    assert_eq!(b.spent(), 17);
    assert_eq!(b.offer(measure, total), Offer::Locked(5));
    for _ in 0..4 {
        assert!(b.buy(Entry::Skill(Id::Brace), total));
    }
    assert_eq!(b.offer(measure, total), Offer::Locked(1));
    assert!(b.buy(Entry::Cell(Cell::Catch), total));
    assert_eq!(b.spent(), 22);
    assert_eq!(b.offer(measure, total), Offer::Ready);
    assert!(b.buy(measure, total));
    assert_eq!((b.spent(), b.cells.key), (23, Some(Key::Measure)));
    assert_eq!(b.offer(measure, total), Offer::Full);
    // 다른 맺음도 그 갈래의 길 끝에서(0215 → 0270 C3): 맞받기의 넷째 줄 패시브가 없으면 끊겨 있다.
    assert_eq!(b.offer(edge, 23), Offer::Cut);
    for e in [
        Entry::Skill(Id::Echo),
        Entry::Cell(Cell::GatheredRiposte),
        Entry::Ult(Ult::Line),
        Entry::Cell(Cell::OneWhoStands),
        Entry::Cell(Cell::Mend),
    ] {
        assert!(b.buy(e, total), "{e:?}");
    }
    assert!(b.branch_spent(Branch::Riposte) >= KEY_BRANCH);
    let spent = b.spent();
    // 다른 맺음: 값 없이 바꾼다 — 점수가 없어도.
    assert_eq!(b.offer(edge, spent), Offer::Swap);
    assert!(!b.can_buy(spent), "a swap is not something to buy");
    assert!(b.buy(edge, spent));
    assert_eq!((b.spent(), b.cells.key), (spent, Some(Key::Edge)));
    assert_eq!(b.rank(measure), 0);
    let mut great = Ult::Upend;
    b.respec(&mut great);
    assert_eq!(b.cells.key, None);
}

/// 0158 나: 되돌리기는 쓴 점수를 모두 돌려준다 — 스킬은 모두 안 배운 것으로(0166: 처음의 둘도)·큰 기술 1단, 고르기는 지워진다.
/// 값은 `30 + 5 × 쓴 점수`. 끼운 큰 기술은 쟁반 뒤집기로.
#[test]
fn a_respec_returns_every_point_and_clears_picks() {
    assert_eq!(Board::default().respec_cost(), RESPEC_BASE);
    let total = 12;
    let mut b = Board::default();
    for e in [Entry::Skill(Id::Whirl); 5] {
        assert!(b.buy(e, total));
    }
    assert!(b.flip(Id::Whirl));
    assert!(b.buy(Entry::Cell(Cell::EvenBreath), total));
    assert!(b.buy(Entry::Skill(Id::Quake), total) && b.buy(Entry::Skill(Id::Gather), total));
    assert_eq!(b.spent(), 8);
    assert_eq!(b.respec_cost(), 30 + 5 * 8);
    // 0161: 패시브 칸도 쓴 점수다 — 값에 들어가고, 되돌리면 같이 닫힌다.
    assert!(
        b.buy(Entry::Cell(Cell::UnbrokenStride), total)
            && b.buy(Entry::Cell(Cell::ClosingBlow), total)
    );
    assert!(b.cells.flip_cell(Cell::UnbrokenStride));
    assert_eq!((b.spent(), b.respec_cost()), (10, 30 + 5 * 10));
    let mut great = Ult::Upend;
    b.respec(&mut great);
    assert_eq!(b, Board::default());
    assert_eq!(b.cells.count(), 0, "a respec closes every passive cell");
    assert!(!b.cells.has(Cell::UnbrokenStride) && !b.cells.has(Cell::ClosingBlow));
    assert_eq!(
        b.cells.picks, [0; 3],
        "0215: the passive picks are cleared too"
    );
    assert_eq!((b.spent(), b.left(total)), (0, total));
    assert_eq!(b.pick, [Pick::A; N]);
    assert!(
        IDS.iter().all(|&id| !b.learned(id)),
        "no skill stays learned — not even the first two"
    );
    assert_eq!(great, Ult::Upend);
}

/// 되돌리기는 잔불을 낸다 — 모자라면 아무 일도 없고(판·잔불 그대로), 쓴 점수가 없으면 낼 것도 없다.
#[test]
fn a_respec_charges_embers_and_does_nothing_when_short_or_unspent() {
    let total = 10;
    let mut b = Board::default();
    let mut embers = 500;
    let mut great = Ult::Upend;
    assert!(!b.respec_paid(&mut embers, &mut great), "nothing spent");
    assert_eq!(embers, 500);
    assert!(b.buy(Entry::Skill(Id::Gather), total));
    assert!(b.buy(Entry::Skill(Id::Gather), total));
    assert!(b.buy(Entry::Skill(Id::Cut), total));
    let (cost, before) = (b.respec_cost(), b);
    assert_eq!(cost, 30 + 5 * 3);
    let mut poor = cost - 1;
    assert!(!b.respec_paid(&mut poor, &mut great), "one ember short");
    assert_eq!((b, poor), (before, cost - 1));
    assert!(b.respec_paid(&mut embers, &mut great));
    assert_eq!(embers, 500 - cost);
    assert_eq!((b, b.left(total)), (Board::default(), total));
    // 다시 찍으면 값은 처음 값부터.
    assert_eq!(b.respec_cost(), RESPEC_BASE);
}

/// 0166 → 0167: 처음의 둘(휘돌이·가로지르기)도 값 없이 주지 않는다 — 새 판은 스킬을 하나도 모르고 점수도 없다. 첫 레벨이
/// 첫 점수이고, 무엇을 먼저 배울지는 방랑자가 고른다(첫 줄의 스킬 넷·패시브 여섯 가운데). 큰 기술도 하나도 없다(0172 — 쟁반
/// 뒤집기도 배운다): 끼울 것이 없으니 월드에 넘기는 큰 기술의 단은 0이다.
#[test]
fn a_fresh_board_knows_no_skill_and_has_no_points() {
    let b = Board::default();
    assert!(IDS.iter().all(|&id| !b.learned(id)) && !b.knows_a_skill());
    assert!(ULTS.iter().all(|&u| !b.has_ult(u)) && b.ults_known() == 0);
    assert_eq!(b.loadout(START, Ult::Upend).ult, 0, "no great skill yet");
    assert_eq!((points(0, 0, 0, 0), b.spent()), (0, 0));
    assert!(!b.can_buy(0), "nothing to learn before the first level");
    // 첫 레벨의 한 점: 첫 줄의 어느 스킬이든, 몸의 판의 첫 줄 패시브든 — 하나만(0270 C3 — 몸의 판의 뿌리는 첫 줄 패시브).
    let one = points(1, 0, 0, 0);
    assert_eq!(
        b.offer(Entry::Cell(Cell::ClosingBlow), one),
        Offer::Locked(4)
    );
    for e in [
        Entry::Cell(Cell::Catch),
        Entry::Skill(Id::Whirl),
        Entry::Skill(Id::Gather),
        Entry::Skill(Id::Brace),
    ] {
        let mut pick = b;
        assert_eq!(pick.offer(e, one), Offer::Ready, "{e:?}");
        assert!(pick.buy(e, one));
        assert_eq!(pick.left(one), 0);
        assert_eq!(pick.offer(Entry::Skill(Id::Cut), one), Offer::Short);
    }
    // 옛 기록의 몫은 처음의 둘의 값과 같다.
    assert_eq!(Board::default().with_starters().spent(), GIFT_POINTS);
}

/// 0166: 막 배운 스킬은 빈 칸(안 배운 스킬이 놓인 자리)에 저절로 끼워진다 — 칸 1부터. 빈 칸이 없으면 그대로(끼우는 것은
/// 기술 화면에서), 이미 끼워져 있으면 그대로. 끼운 둘의 단은 판에서 — 안 배운 스킬이 놓인 칸은 단 0(빈 칸)이다.
#[test]
fn a_newly_learned_skill_fills_the_first_empty_slot() {
    let mut b = Board::default();
    let l = b.loadout(START, Ult::Upend);
    assert_eq!(
        (l.slots[0].rank, l.slots[1].rank),
        (0, 0),
        "both slots start empty"
    );
    assert!(b.buy(Entry::Skill(Id::Gather), 4));
    let worn = b.wear_on_learn(START, Id::Gather);
    assert_eq!(worn, [Id::Gather, Id::Cut], "into slot 1");
    let l = b.loadout(worn, Ult::Upend);
    assert_eq!((l.slots[0].rank, l.slots[1].rank), (1, 0));
    assert!(b.buy(Entry::Skill(Id::Whirl), 4));
    let worn = b.wear_on_learn(worn, Id::Whirl);
    assert_eq!(worn, [Id::Gather, Id::Whirl], "the second one into slot 2");
    assert!(b.buy(Entry::Skill(Id::Brace), 4));
    assert_eq!(
        b.wear_on_learn(worn, Id::Brace),
        worn,
        "no empty slot: left as worn"
    );
    assert_eq!(b.wear_on_learn(worn, Id::Whirl), worn, "already worn");
}

/// 3단의 고르기는 값 없이 뒤집는다(쓴 점수 그대로) — 3단이 안 된 스킬은 뒤집히지 않는다.
#[test]
fn a_pick_flips_for_free_from_rank_three() {
    let mut b = Board::default();
    assert!(!b.flip(Id::Cut), "rank 1 has no pick");
    b.rank[Id::Cut.index()] = PICK_RANK;
    let spent = b.spent();
    assert!(b.flip(Id::Cut));
    assert_eq!(b.pick[Id::Cut.index()], Pick::B);
    assert!(b.flip(Id::Cut));
    assert_eq!(b.pick[Id::Cut.index()], Pick::A);
    assert_eq!(b.spent(), spent);
    assert_eq!(Pick::by_key(Pick::B.key()), Some(Pick::B));
    assert_eq!(Pick::by_key("c"), None);
}

/// 손댄 기록을 바로잡는다: 범위 밖의 단은 안으로, 3단이 안 된 고르기는 ㄱ, 가진 점수보다 많이 쓴 판은 처음으로.
#[test]
fn a_board_from_a_record_is_made_sane_against_the_points_it_has() {
    let odd = Board {
        rank: ranks([0, 9, 2, 3, 0, 0, 7]),
        pick: [Pick::B; N],
        ult: ult_ranks([0, 9]),
        cells: Tree::default(),
    };
    let b = odd.sane(40);
    assert_eq!(b.rank, ranks([0, MAX_RANK, 2, 3, 0, 0, MAX_RANK]));
    assert_eq!(
        b.pick,
        picks([
            Pick::A,
            Pick::B,
            Pick::A,
            Pick::B,
            Pick::A,
            Pick::A,
            Pick::B
        ])
    );
    assert_eq!(b.ult, ult_ranks([0, ULT_MAX_RANK]));
    assert_eq!(odd.sane(b.spent()), b, "exactly enough points");
    assert_eq!(odd.sane(b.spent() - 1), Board::default(), "one short");
    assert_eq!(Board::default().sane(0), Board::default());
    // 0161: 넘치면 먼저 스킬·큰 기술만 처음으로 — 연 패시브 칸은 남는다(칸만으로 점수 안이면). 칸만으로도 넘으면 칸까지.
    let cells = Tree::with(&[Cell::EvenBreath, Cell::Catch, Cell::LandingBeat]);
    let over = Board { cells, ..odd };
    assert_eq!(over.sane(b.spent() + 3).cells, cells, "enough for all");
    let kept = over.sane(b.spent() + 2);
    assert_eq!(
        kept,
        Board {
            cells,
            ..Board::default()
        },
        "two short: the skills reset, the cells stay"
    );
    assert_eq!(kept.spent(), 3);
    assert_eq!(over.sane(2), Board::default(), "not even the cells fit");
    // 옛 꼴로 적힌 칸·범위 밖 비트는 한 꼴로 바로잡힌다.
    let legacy = Board {
        cells: Tree {
            open: [2, 0, 0],
            cells: [0x80, 0, 0],
            key: None,
            links: 0,
            picks: [0; 3],
            veins: 0,
            vein_picks: 0,
        },
        ..Board::default()
    };
    assert_eq!(legacy.sane(9).cells, Tree::from_counts([2, 0, 0]));
}

/// 0158 G2 → 0172: 큰 기술은 셋 — 셋 다 큰 기술 줄(쓴 점수 8)이 열리면 **2점**으로 배우고 2·3단은 1점씩(쟁반 뒤집기도 —
/// 처음부터 주는 것은 없다). 점수가 1뿐이면 배우지 못한다. 처음 배운 것이 끼울 것이고, 되돌리면 하나도 없다.
#[test]
fn the_great_skills_are_learned_for_two_points_once_eight_are_spent() {
    // 0270 C3: 무기 판마다 하나 — 다섯(큰 종·눈보라가 뒤에 붙었다).
    assert_eq!(
        ULTS,
        [
            Ult::Upend,
            Ult::Flurry,
            Ult::Line,
            Ult::Bell,
            Ult::Blizzard,
            Ult::Rain,
            Ult::Gale,
            Ult::Ambush,
            Ult::Tempest,
            Ult::Grove,
            Ult::Price
        ]
    );
    for u in ULTS {
        assert_eq!(Ult::by_key(u.key()), Some(u));
        assert!(u.clip().name.starts_with("ult_"));
    }
    assert_eq!(Ult::Upend.key(), "upend", "the G0 record key");
    let (flurry, line) = (Entry::Ult(Ult::Flurry), Entry::Ult(Ult::Line));
    assert_eq!((flurry.free(), line.free()), (0, 0));
    assert_eq!(
        (flurry.price(0), flurry.price(1), flurry.price(2)),
        (2, 1, 1)
    );
    let upend = Entry::Ult(Ult::Upend);
    assert_eq!((upend.free(), upend.price(0), upend.price(1)), (0, 2, 1));
    assert_eq!((flurry.paid(1), flurry.paid(3)), (2, 4));
    assert_eq!(upend.paid(3), 4, "0172: the same as the others");
    let mut b = Board::default();
    assert_eq!(b.offer(flurry, 23), Offer::Locked(8));
    // 몰아치기까지의 길(0215): 휘돌이 → 고른 호흡 → 땅울림 → 맺음타.
    for e in [
        Entry::Skill(Id::Whirl),
        Entry::Skill(Id::Whirl),
        Entry::Skill(Id::Whirl),
        Entry::Skill(Id::Whirl),
        Entry::Cell(Cell::EvenBreath),
        Entry::Skill(Id::Quake),
        Entry::Cell(Cell::ClosingBlow),
        Entry::Skill(Id::Cut),
    ] {
        assert!(b.buy(e, 23), "{e:?}");
    }
    assert_eq!(b.spent(), 8);
    assert!(!b.has_ult(Ult::Flurry));
    // 아직 하나도 안 배웠다: 끼울 것이 없다(끼운 자리의 이름은 그대로 — 그 단이 0이다).
    assert_eq!((b.ults_known(), b.wearable_ult(Ult::Line)), (0, Ult::Line));
    assert_eq!(b.loadout(START, Ult::Line).ult, 0);
    assert_eq!(b.offer(flurry, 9), Offer::Short, "one point is not enough");
    assert!(!b.buy(flurry, 9));
    assert_eq!(b.offer(flurry, 10), Offer::Ready);
    assert!(b.buy(flurry, 10));
    assert!(b.has_ult(Ult::Flurry));
    // 배운 것이 하나면 그것이 끼울 것이다 — 끼운 자리가 무엇을 가리키든.
    assert_eq!(b.wearable_ult(Ult::Upend), Ult::Flurry);
    assert_eq!(b.loadout(START, Ult::Upend).great, Ult::Flurry);
    assert_eq!((b.spent(), b.left(10)), (10, 0));
    assert!(b.buy(flurry, 12) && b.buy(flurry, 12));
    assert_eq!(b.offer(flurry, 99), Offer::Full);
    assert_eq!(b.spent(), 12);
    // 줄 긋기는 무기 판의 칸이라(0270 C3) 몸의 판의 길을 묻지 않는다 — 줄만 열리면.
    assert_eq!(b.offer(line, 99), Offer::Ready);
    assert!(b.buy(Entry::Cell(Cell::FlowingBeat), 99) && b.buy(Entry::Cell(Cell::GapBeat), 99));
    assert!(b.buy(line, 16));
    assert_eq!((b.ult, b.ults_known()), (ult_ranks([0, 3, 1]), 2));
    // 쟁반 뒤집기는 자리의 길 끝(모으기 → 멈추지 않는 걸음 → 던지기 → 기우는 땅).
    for e in [
        Entry::Skill(Id::Gather),
        Entry::Cell(Cell::UnbrokenStride),
        Entry::Skill(Id::Throw),
        Entry::Cell(Cell::TiltingGround),
    ] {
        assert!(b.buy(e, 99), "{e:?}");
    }
    assert!(b.buy(upend, 22));
    assert_eq!((b.ult, b.spent()), (ult_ranks([1, 3, 1]), 22));
    // 옛 기록의 꼴: 쟁반 뒤집기를 지닌 판(이미 올린 단은 그대로).
    assert_eq!(Board::default().with_upend().ult, ult_ranks([1]));
    assert_eq!(b.with_upend(), b);
    // 되돌리면 배운 큰 기술이 모두 돌아간다 — 끼울 것이 없다(자리의 이름은 그대로).
    let mut great = Ult::Line;
    b.respec(&mut great);
    assert_eq!((b.ult, great), (ult_ranks([]), Ult::Line));
    assert_eq!(b.loadout(START, great).ult, 0);
}
