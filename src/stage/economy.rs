//! 벌이를 잰다(KB `guides/roguelite-economy-check` 꼴): 잔불(레벨 값)과 기술 점수(스킬 레벨 값)의 들어오는 곳을 **실제 배치**에서
//! 세어 지역마다의 벌이 · 도착 레벨 · 기술 점수의 상한을 찍는다. 수치를 바꾸지 않는다 — 재기만 한다(무시 시험).
//!   cargo test economy_probe -- --ignored --nocapture
//! 셈의 가정: 지역마다 서 있는 것을 한 번씩 다 쓰러뜨리고(떼 마당 · 엘리트 · 문지기 · 필드 보스), 여정과 도감 보상을 다 받는다.
//! 여벌을 거둔 잔불 · 잔불 주머니(× 1.25) · 불티 속 남는 불씨는 빼고 센다(덤). 시간은 재지 않는다.
use super::*;
use crate::journey::{Reward, GOALS};
use crate::progression::{Level, CORE_CAP, LINES};
use crate::tuning::{ELITE_FIRST_EMBERS, FIELD_FIRST_EMBERS};

/// 지역 하나의 벌이(잔불).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Take {
    /// 서 있는 적과 떼 마당(쉬면 다시 선다): 수 · 잔불.
    mobs: (u32, u32),
    /// 엘리트(판마다 한 번): 수 · 잔불(처음 보상 빼고).
    elites: (u32, u32),
    keeper: u32,
    field: u32,
    /// 기록에서 처음일 때만: 엘리트마다 `ELITE_FIRST_EMBERS` + 필드 보스 `FIELD_FIRST_EMBERS`.
    firsts: u32,
    /// 여정의 잔불 · 기술 점수.
    journey: (u32, u32),
    shards: u32,
}

impl Take {
    /// 쉬고 다시 돌기(서 있는 적만).
    fn lap(&self) -> u32 {
        self.mobs.1
    }
    /// 다음 판에 다시 다 깨기(처음 보상 없이).
    fn rerun(&self) -> u32 {
        self.mobs.1 + self.elites.1 + self.keeper + self.field
    }
    /// 처음 다 깨기.
    fn first(&self) -> u32 {
        self.rerun() + self.firsts
    }
}

fn take(stage: usize) -> Take {
    let def = STAGES[stage];
    let (run, w) = StageRun::start(def);
    let mut t = Take::default();
    for en in &w.enemies {
        let v = en.ember_worth(1.0);
        if en.boss && en.field {
            t.field += v;
            t.firsts += FIELD_FIRST_EMBERS;
        } else if en.boss {
            t.keeper += v;
        } else if en.elite.is_some() {
            t.elites = (t.elites.0 + 1, t.elites.1 + v);
            t.firsts += ELITE_FIRST_EMBERS;
        } else {
            t.mobs = (t.mobs.0 + 1, t.mobs.1 + v);
        }
    }
    for g in GOALS.iter().filter(|g| g.stage == stage) {
        match g.reward {
            Reward::Embers(n) => t.journey.0 += n,
            Reward::Point => t.journey.1 += 1,
        }
    }
    t.shards = run.level.marks_of(Find::Shard.glyph()).count() as u32;
    t
}

/// `embers`로 닿는 총 레벨과 남는 잔불(어느 줄이든 값은 총 레벨로 — 0109).
fn level_for(mut embers: u32) -> (u32, u32) {
    let mut lv = Level::default();
    loop {
        let next = LINES.iter().find_map(|&l| lv.next_cost(l).map(|c| (l, c)));
        match next {
            Some((l, c)) if embers >= c => {
                embers -= c;
                lv = lv.raised(l);
            }
            _ => return (lv.total(), embers),
        }
    }
}

/// 총 레벨 `n`까지의 값.
fn cost_to(n: u32) -> u32 {
    let mut lv = Level::default();
    let mut sum = 0;
    while lv.total() < n {
        let Some((l, c)) = LINES.iter().find_map(|&l| lv.next_cost(l).map(|c| (l, c))) else {
            break;
        };
        sum += c;
        lv = lv.raised(l);
    }
    sum
}

/// 도감 보상의 합(적 · 물건 — 맨손은 도감에 없다).
fn codex_total() -> u32 {
    let things =
        crate::charms::TABLE.len() + crate::garb::TABLE.len() + crate::weapons::IDS.len() - 1;
    crate::codex::FOE_EMBERS * crate::species::ALL.len() as u32
        + crate::codex::THING_EMBERS * things as u32
}

/// 재는 식이 실제 규칙과 같은가: 잔재 하나의 값은 등급 배율의 올림이고, 레벨 값의 합은 표와 같다.
#[test]
fn the_probe_counts_with_the_games_own_rules() {
    assert_eq!(cost_to(1), crate::progression::LEVEL_BASE);
    assert_eq!(level_for(cost_to(7)), (7, 0));
    assert_eq!(level_for(cost_to(7) - 1).0, 6);
    let sanctuary = take(0);
    // 서막 밖 첫 지역: 엘리트 셋 · 문지기 · 필드 보스가 선다.
    assert_eq!(sanctuary.elites.0, 3);
    assert!(sanctuary.keeper > 0 && sanctuary.field > 0);
    assert_eq!(
        sanctuary.firsts,
        3 * ELITE_FIRST_EMBERS + FIELD_FIRST_EMBERS
    );
    assert!(sanctuary.first() > sanctuary.rerun() && sanctuary.rerun() > sanctuary.lap());
}

#[test]
#[ignore = "재는 시험 — cargo test economy_probe -- --ignored --nocapture"]
fn economy_probe() {
    let names = ["성소", "마을", "정원", "작업층"];
    let takes: Vec<Take> = (0..REGIONS).map(take).collect();
    let prologue = take(PROLOGUE).first();
    println!("\n== 지역마다의 잔불(등급 · 적 수 · 값) ==");
    println!("지역     등급  서 있는 적(수/잔불)  엘리트  문지기  필드  처음 보상 | 처음 다 깨기  쉬고 한 바퀴  다음 판 | 여정");
    for (i, t) in takes.iter().enumerate() {
        println!(
            "{:<6} {:>4} {:>10}/{:<6} {:>6} {:>6} {:>5} {:>8} | {:>10} {:>10} {:>8} | {:>4}",
            names[i],
            STAGES[i].tier,
            t.mobs.0,
            t.mobs.1,
            t.elites.1,
            t.keeper,
            t.field,
            t.firsts,
            t.first(),
            t.lap(),
            t.rerun(),
            t.journey.0
        );
    }
    println!("서막 숲: {prologue}");
    let codex = codex_total();
    let journey: u32 = takes.iter().map(|t| t.journey.0).sum();
    let firsts: u32 = takes.iter().map(Take::first).sum::<u32>() + prologue;
    let rerun: u32 = takes.iter().map(Take::rerun).sum();
    println!("\n== 한 번씩 받는 것 ==");
    println!(
        "쓰러뜨림 {firsts} + 여정 {journey} + 도감 {codex} = {}",
        firsts + journey + codex
    );
    println!("다음 판을 다 깨면 +{rerun}");

    println!("\n== 도착 레벨(전부 잡고 여정을 다 받고 모두 레벨에 — 도감은 따로) ==");
    let mut sum = prologue;
    for (i, t) in takes.iter().enumerate() {
        sum += t.first() + t.journey.0;
        let (lv, left) = level_for(sum);
        let next = names.get(i + 1).copied().unwrap_or("끝");
        println!(
            "{} 뒤({next} 도착): 누적 {sum} → Lv{lv}(남는 잔불 {left}) · 쉬고 한 바퀴 더 돌면 +{}",
            names[i],
            t.lap()
        );
    }
    println!("도감을 다 받으면 +{codex}(지역에 고르게 들지는 않는다)");

    println!("\n== 쓸 곳 ==");
    let top = level_for(u32::MAX).0;
    let levels = cost_to(top);
    let shop: u32 = crate::growth::SHOP.iter().map(|&(_, p)| p).sum();
    let forge: u32 = crate::forge::COST.iter().map(|&(_, e)| e).sum();
    println!("레벨 {top}까지 {levels} · 가게 전부 {shop} · 강화 무기 하나 {forge}");
    for lv in [5, 7, 10, top] {
        println!("  Lv{lv}까지 {}", cost_to(lv));
    }
    let once = firsts + journey + codex;
    let need = levels + shop + 2 * forge;
    println!(
        "레벨 + 가게 + 무기 둘 강화 = {need}: 한 번씩 받는 것 {once} + 다음 판 {:.1}번",
        need.saturating_sub(once) as f32 / rerun as f32
    );

    println!("\n== 기술 점수 ==");
    let elites: u32 = takes.iter().map(|t| t.elites.0).sum();
    let shards: u32 = takes.iter().map(|t| t.shards).sum();
    let jp: u32 = takes.iter().map(|t| t.journey.1).sum();
    let cores = CORE_CAP * REGIONS as u32;
    println!(
        "상한 {} = 레벨 {top} + 엘리트 {elites} + 핵 {cores}(문지기마다 {CORE_CAP}) + 조각 {shards} + 여정 {jp}",
        top + elites + cores + shards + jp
    );
    println!(
        "첫 판(문지기 한 번씩): 레벨 + {}",
        elites + REGIONS as u32 + shards + jp
    );
    for p in crate::plates::PLATES {
        let max: u32 = crate::skills::ENTRIES
            .iter()
            .filter(|&&e| p.has(e))
            .map(|&e| e.paid(e.max()))
            .sum();
        println!("  트리 {:<10} 다 올리면 {max}", crate::i18n::tr(p.name()));
    }
}
