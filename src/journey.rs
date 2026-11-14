//! 여정(0183 — 사장 2026-11-14 "좋은 건 넣어야지"): 지역마다의 목표 표. 큰 창의 여정 탭이 그리고, 이룬 것은 "받기"로 작은 보상.
//!
//! 꼴은 nail 저널(KB `roguelite-journal-goals`)과 같다 — 표 하나 + 진행은 **이미 있는 기록에서 다시 센다**(문·핵·숨은 물건·엘리트·
//! 도감·읽은 새긴 돌) + 기록에는 "받았다"만(`save::key::JOURNEY`). 목표를 더하거나 수를 바꿔도 쌓인 기록으로 바로 맞춰진다.
//!
//! 두 갈래(PR 21 리뷰): 주요 = 지역의 길(화면 목표 글과 같은 일 — 문·문지기), 발견 = 놓쳐도 길이 막히지 않는 것(숨은 것·관찰·새긴 돌).
//! 주민은 목표에 넣지 않는다(lore — 주민은 보상·진행의 수단이 아니다). 새 화폐는 없다: 보상은 잔불 또는 기술 점수 하나.
use crate::codex::{obs_key, Obs};
use crate::i18n::Tr;
use crate::save::Progress;
use crate::species::Kind;
use crate::stage::{elite_id, find_id, Find, StageDef, STAGES};

/// 목표의 갈래.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Branch {
    /// 지역의 길(놓칠 수 없다).
    Main,
    /// 놓쳐도 길이 막히지 않는 것.
    Find,
}

/// 받으면 얻는 것(새 화폐는 만들지 않는다 — lore growth).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reward {
    /// 잔불(지금 지닌 것에 더한다).
    Embers(u32),
    /// 기술 점수 하나(기록의 셈 `Progress::skill_points`가 받은 목표에서 센다).
    Point,
}

/// 무엇을 세나(모두 이미 있는 기록에서).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Need {
    /// 이 지역의 문을 장치로 열었다(`gates`).
    Gate,
    /// 이 지역의 문지기를 넘었다(핵 점수 ≥ 1).
    Keeper,
    /// 이 지역의 숨은 물건(이 종류)을 다 찾았다.
    Finds(Find),
    /// 이 지역의 엘리트를 다 넘었다(처음 쓰러뜨린 것).
    Elites,
    /// 이 지역의 새긴 돌을 다 읽었다.
    Read,
    /// 도감에 이 관찰이 적혔다(종·관찰).
    Seen(Kind, Obs),
}

pub struct Goal {
    /// 기록 이름(바꾸지 않는다 — 받은 목표가 이것으로 남는다). 쉼표를 넣지 않는다.
    pub id: &'static str,
    /// 지역(`stage::STAGES`의 몇째, 0-based).
    pub stage: usize,
    pub kind: Branch,
    pub title: Tr,
    /// 오른쪽 칸의 풀이(어디서·무엇을 — 정답을 다 말하지 않는다).
    pub hint: Tr,
    pub need: Need,
    pub reward: Reward,
}

/// 여정 목표(지역 차례 · 주요 먼저). 새 지역 = 이 표에 줄을 더한다(아이디는 `<지역>.<무엇>`).
pub static GOALS: [Goal; 10] = [
    Goal {
        id: "s.gate",
        stage: 0,
        kind: Branch::Main,
        title: |s| s.jg_s_gate,
        hint: |s| s.jg_s_gate_d,
        need: Need::Gate,
        reward: Reward::Embers(30),
    },
    Goal {
        id: "s.warden",
        stage: 0,
        kind: Branch::Main,
        title: |s| s.jg_s_warden,
        hint: |s| s.jg_s_warden_d,
        need: Need::Keeper,
        reward: Reward::Embers(60),
    },
    Goal {
        id: "s.shards",
        stage: 0,
        kind: Branch::Find,
        title: |s| s.jg_s_shards,
        hint: |s| s.jg_s_shards_d,
        need: Need::Finds(Find::Shard),
        reward: Reward::Point,
    },
    Goal {
        id: "s.elites",
        stage: 0,
        kind: Branch::Find,
        title: |s| s.jg_s_elites,
        hint: |s| s.jg_s_elites_d,
        need: Need::Elites,
        reward: Reward::Embers(30),
    },
    Goal {
        id: "s.roused",
        stage: 0,
        kind: Branch::Find,
        title: |s| s.jg_s_roused,
        hint: |s| s.jg_s_roused_d,
        need: Need::Seen(Kind::Warden, Obs::Roused),
        reward: Reward::Embers(20),
    },
    Goal {
        id: "v.keeper",
        stage: 1,
        kind: Branch::Main,
        title: |s| s.jg_v_keeper,
        hint: |s| s.jg_v_keeper_d,
        need: Need::Keeper,
        reward: Reward::Embers(70),
    },
    Goal {
        id: "v.stones",
        stage: 1,
        kind: Branch::Find,
        title: |s| s.jg_v_stones,
        hint: |s| s.jg_v_stones_d,
        need: Need::Read,
        reward: Reward::Point,
    },
    Goal {
        id: "v.feeding",
        stage: 1,
        kind: Branch::Find,
        title: |s| s.jg_v_feeding,
        hint: |s| s.jg_v_feeding_d,
        need: Need::Seen(Kind::Husk, Obs::Docile),
        reward: Reward::Embers(20),
    },
    Goal {
        id: "v.elites",
        stage: 1,
        kind: Branch::Find,
        title: |s| s.jg_v_elites,
        hint: |s| s.jg_v_elites_d,
        need: Need::Elites,
        reward: Reward::Embers(30),
    },
    Goal {
        id: "v.whet",
        stage: 1,
        kind: Branch::Find,
        title: |s| s.jg_v_whet,
        hint: |s| s.jg_v_whet_d,
        need: Need::Finds(Find::Stone),
        reward: Reward::Embers(20),
    },
];

/// 지도에 이 글자가 몇 개.
fn glyphs(def: &StageDef, c: char) -> u32 {
    def.map.chars().filter(|&x| x == c).count() as u32
}

/// 읽은 새긴 돌의 기록 이름.
pub fn read_id(prefix: &str, nth: usize) -> String {
    format!("{prefix}.{nth}")
}

/// (지금, 목표) — 바 대신 "n/m"으로 보인다(한 번에 끝나는 것은 0/1·1/1).
pub fn count(g: &Goal, p: &Progress) -> (u32, u32) {
    let def = STAGES[g.stage];
    let pre = def.find_prefix;
    let has = |n: u32, f: &dyn Fn(usize) -> String, list: &[String]| {
        (0..n as usize).filter(|&k| list.contains(&f(k))).count() as u32
    };
    match g.need {
        Need::Gate => (p.gates.iter().any(|x| x == pre) as u32, 1),
        Need::Keeper => {
            let cores = match g.stage {
                0 => p.cores,
                _ => p.bell_cores,
            };
            ((cores > 0) as u32, 1)
        }
        Need::Finds(kind) => {
            let n = glyphs(def, kind.glyph());
            (has(n, &|k| find_id(pre, kind, k), &p.finds), n)
        }
        Need::Elites => {
            let n = def.elites.len() as u32;
            (has(n, &|k| elite_id(pre, k), &p.elites), n)
        }
        Need::Read => {
            let n = def.stones.len() as u32;
            (has(n, &|k| read_id(pre, k), &p.read), n)
        }
        Need::Seen(k, o) => (p.codex.contains(&obs_key(k, o)) as u32, 1),
    }
}

pub fn done(g: &Goal, p: &Progress) -> bool {
    let (a, b) = count(g, p);
    b > 0 && a >= b
}

pub fn claimed(g: &Goal, p: &Progress) -> bool {
    p.journey.iter().any(|x| x == g.id)
}

/// 이룬 채 아직 받지 않았다.
pub fn ready(g: &Goal, p: &Progress) -> bool {
    done(g, p) && !claimed(g, p)
}

/// 그 지역에 닿았나(목록에 보이나): 첫 지역은 늘, 다음 지역은 앞 지역의 문지기를 넘은 뒤(내려가는 길이 그때 열린다).
pub fn reached(stage: usize, p: &Progress) -> bool {
    match stage {
        0 => true,
        1 => p.cores > 0,
        _ => false,
    }
}

/// 보이는 목표(표 차례).
pub fn visible(p: &Progress) -> Vec<&'static Goal> {
    GOALS.iter().filter(|g| reached(g.stage, p)).collect()
}

/// 받은 목표의 기술 점수(`Progress::skill_points`의 몫).
pub fn points(claimed: &[String]) -> u32 {
    GOALS
        .iter()
        .filter(|g| g.reward == Reward::Point && claimed.iter().any(|c| c == g.id))
        .count() as u32
}

/// 받을 것이 있나(탭 머리의 점).
pub fn any_ready(p: &Progress) -> bool {
    visible(p).iter().any(|g| ready(g, p))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 아이디는 겹치지 않고 쉼표가 없다(기록 칸이 쉼표 목록), 모든 목표는 닿을 수 있다(세는 수가 0이 아니다 — 지도에 그 글자가 있다).
    #[test]
    fn goals_have_unique_ids_and_every_goal_can_be_reached() {
        for (i, g) in GOALS.iter().enumerate() {
            assert!(!g.id.contains(','), "{}", g.id);
            assert!(GOALS[..i].iter().all(|h| h.id != g.id), "{}", g.id);
            assert!(g.stage < STAGES.len(), "{}", g.id);
            let (_, n) = count(g, &Progress::default());
            assert!(n > 0, "{}: nothing to count", g.id);
            assert!(!crate::i18n::tr(g.title).is_empty());
        }
        // 지역마다 주요 목표가 먼저 하나 이상.
        for st in 0..STAGES.len() {
            let mine: Vec<_> = GOALS.iter().filter(|g| g.stage == st).collect();
            assert!(
                !mine.is_empty() && mine[0].kind == Branch::Main,
                "stage {st}"
            );
        }
    }

    /// 진행은 기록에서 다시 센다: 빈 기록은 아무것도 이루지 않았고, 문을 연 기록은 문 목표를, 셋 다 읽으면 새긴 돌 목표를 이룬다.
    /// 받은 것은 다시 받을 수 없고, 기술 점수 목표를 받으면 기술 점수가 하나 는다.
    #[test]
    fn progress_is_counted_from_the_record_and_a_claim_counts_once() {
        let mut p = Progress::default();
        assert!(GOALS.iter().all(|g| !done(g, &p)));
        p.gates.push("sanctuary".into());
        let gate = &GOALS[0];
        assert!(ready(gate, &p));
        let stones = GOALS.iter().find(|g| g.id == "v.stones").unwrap();
        for k in 0..2 {
            p.read.push(read_id("village", k));
        }
        assert_eq!(count(stones, &p), (2, 3));
        p.read.push(read_id("village", 2));
        assert!(done(stones, &p));
        let before = p.skill_points();
        p.journey.push(stones.id.into());
        assert!(!ready(stones, &p));
        assert_eq!(p.skill_points(), before + 1);
        // 성소의 기술 점수 목표만 받아도 하나.
        assert_eq!(points(&["s.shards".into(), "s.gate".into()]), 1);
    }

    /// 마을의 목표는 성소의 문지기를 넘은 뒤에 보인다(내려가는 길이 그때 열린다).
    #[test]
    fn the_next_regions_goals_appear_after_its_way_down_opens() {
        let mut p = Progress::default();
        assert!(visible(&p).iter().all(|g| g.stage == 0));
        p.cores = 1;
        assert!(visible(&p).iter().any(|g| g.stage == 1));
    }

    /// 보상은 작다(0183 — 새 화폐 없음, 잔불은 한 단 값(50 + 25×레벨)을 넘지 않는다, 기술 점수는 지역마다 하나까지).
    #[test]
    fn rewards_stay_small() {
        for g in &GOALS {
            if let Reward::Embers(n) = g.reward {
                assert!(
                    n <= crate::progression::LEVEL_BASE + crate::progression::LEVEL_STEP,
                    "{}",
                    g.id
                );
            }
        }
        for st in 0..STAGES.len() {
            let pts = GOALS
                .iter()
                .filter(|g| g.stage == st && g.reward == Reward::Point)
                .count();
            assert!(pts <= 1, "stage {st}");
        }
    }
}
