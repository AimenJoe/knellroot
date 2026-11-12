//! 기록(0064 M2): 판을 넘어 남는 것 + 판 안의 자리(마지막으로 쉰 쉼터). 설정(`prefs`)과 따로.
//!
//! - 형식: `키=값` 줄(첫 게임 nail `profile.rs` — 키 이름은 바꾸지 않는다, 모르는 키는 지우지 않고 그대로 들고 다닌다,
//!   깨진 줄은 건너뛴다). 그래서 뒤에 칸을 더해도 옛 기록이 그대로 읽힌다.
//! - 자리(슬롯 셋, 0066 — KB `roguelite-save-slots-and-continue`): 웹은 localStorage `knell.save.0`·`.1`·`.2` + 마지막 슬롯
//!   `knell.slot`(web/index.html `knell_rec_*`, 칸 번호 인자), 데스크톱·Android는 설정과 같은 폴더의 `save0.txt`·`save1.txt`·
//!   `save2.txt`·`slot.txt`(임시 파일에 쓰고 이름 바꾸기 — 쓰다 꺼져도 옛 기록이 남는다). 슬롯 1 = 옛 한 칸 그대로(옮기는 코드 없음).
//! - 든 무기도 기록에(`weapon`, 0066 — 설정 칸에 두면 슬롯 사이에 샜다).
//! - 쓰는 때는 앱이 정한다: 바뀐 것이 있을 때만(`Progress` 비교).
use std::collections::BTreeMap;

/// 키 이름(바꾸지 않는다).
pub mod key {
    /// 형식 판(지금 1).
    pub const VERSION: &str = "v";
    /// 잔불(M1, 쓰러져도 잃지 않는다).
    pub const EMBERS: &str = "embers";
    /// 스테이지를 끝낸 횟수.
    pub const CLEARS: &str = "clears";
    /// 판 안의 자리: 마지막으로 쉰 쉼터 번호(없으면 판 밖 — 다음은 입구부터).
    pub const RUN_REST: &str = "run.rest";
    /// 그 쉼터에서 다시 서면 문이 열린 뒤부터인가(1) 입구부터인가(0) — 0032의 되돌아갈 단계.
    pub const RUN_GATE: &str = "run.gate";
    /// 쉰 곳의 지역(0118, `stage::STAGES` 차례)과 이 판에 이긴 문지기 수 — 없으면 0(옛 기록 = 성소).
    pub const RUN_STAGE: &str = "run.stage";
    pub const RUN_DOWN: &str = "run.down";
    /// 장신구(M3): 가진 것("shell,ear")·낀 것(세 칸 "shell,,ear") — `charms.rs`의 키.
    pub const CHARMS_OWNED: &str = "charm.owned";
    pub const CHARMS_WORN: &str = "charm.worn";
    /// 연 상자 번호들("0,2") — 판을 넘어 남는다(한 번 연 상자는 다시 비어 있다).
    pub const CHESTS: &str = "chests";
    /// 성장(M4): 핵으로 고른 횟수(갈래마다), 가게에서 산 등잔 기름.
    pub const GROW_HP: &str = "grow.hp";
    pub const GROW_FLASK: &str = "grow.flask";
    pub const GROW_HEAL: &str = "grow.heal";
    pub const SHOP_OIL: &str = "shop.oil";
    /// 든 무기(0066: prefs 칸 21에서 옮김 — 슬롯 사이에 새지 않게).
    pub const WEAPON: &str = "weapon";
    /// 끼운 스킬 둘(0151 — 값은 `skills::Id::key`).
    pub const SKILL1: &str = "skill.1";
    pub const SKILL2: &str = "skill.2";
    /// 기술 판(0158): 스킬마다의 단 `skill.rank.<스킬 키>`(0~5)·3단의 고르기 `skill.pick.<스킬 키>`("a"·"b"),
    /// 큰 기술의 단 `ult.rank.<큰 기술 키>`(1~3). 뒤에 스킬 키(`skills::Id::key`·`skills::UPEND_KEY`)를 붙인다.
    /// 처음 그대로인 칸은 쓰지 않는다 — 키가 없는 옛 기록은 처음의 둘 1단·큰 기술 1단.
    pub const SKILL_RANK: &str = "skill.rank.";
    pub const SKILL_PICK: &str = "skill.pick.";
    pub const ULT_RANK: &str = "ult.rank.";
    /// 끼운 큰 기술(0158 G2 — 값은 `skills::Ult::key`). 쟁반 뒤집기(처음의 것)면 쓰지 않는다.
    pub const ULT_WORN: &str = "ult.worn";
    /// 가진 무기(0117): 무기 기록 키들("sword,hammer") — 맨손은 늘 있다. 키가 없는 옛 기록(판을 한 번이라도 한 것)은 다 가진 것으로.
    pub const WEAPONS: &str = "weapons";
    /// 쌓인 플레이 시간(초, 0081 — 이어하기 목록의 "시간"). 개발 판·메뉴·타이틀에 있는 동안은 세지 않는다.
    pub const PLAY: &str = "play";
    /// 잔불 레벨(0104, v0.4.0 G1): 줄마다 올린 단 0~5.
    pub const LV_LIFE: &str = "lv.life";
    pub const LV_STAMINA: &str = "lv.stamina";
    pub const LV_MIGHT: &str = "lv.might";
    /// 숨은 물건(0105): 주운 것의 기록 이름들("sanctuary.lamp.0,sanctuary.sip.1").
    pub const FINDS: &str = "finds";
    /// 처음 쓰러뜨린 엘리트(0112): 기록 이름들("sanctuary.elite.0,…") — 처음 덤은 한 번만.
    pub const ELITES: &str = "elites";
    /// 시작한 판 수(0122 — 주민 재방문 "처음 본 판이 끝난 뒤의 다른 판").
    pub const RUNS: &str = "runs";
    /// 주민을 처음 본 판(`V01:3,…`)·재방문으로 바뀐 주민(`V01,…`, 단방향).
    pub const RESIDENTS_SEEN: &str = "residents.seen";
    pub const RESIDENTS_CHANGED: &str = "residents.changed";
    /// 도감(0130): 본 것의 이름(`codex::obs_key`, 쉼표).
    pub const CODEX: &str = "codex";
    /// 문 장치로 연 문(0113): 스테이지 이름 앞(`find_prefix`)들 — 한 번 열면 판을 넘어 열려 있다(지름길).
    pub const GATES: &str = "gates";
    /// 문지기 핵 점수(0105): 이긴 횟수, `progression::CORE_CAP`까지.
    pub const CORE_WARDEN: &str = "core.warden";
    /// 깨진 종지기 핵 점수(0125, v0.5.0 C3): 둘째 지역 문지기를 이긴 횟수, `CORE_CAP`까지.
    pub const CORE_BELL: &str = "core.bellkeeper";
    /// v0.4.0 옮기기를 했다(0105 — 핵 고르기 → 생명·숨은 물건·핵 점수). 한 번만.
    pub const MIGRATED_V04: &str = "migrated.v04";
    /// 기술 나무(0106)의 옛 꼴: 갈래마다 **앞에서부터** 연 칸 수. 0161부터는 `tree.cells.*`가 없을 때만 읽는다("앞에서부터 N칸").
    /// 쓸 때는 앞에서부터 이어 연 수를 적는다(옛 판이 읽어도 가진 것보다 많이 열리지 않는다). 목표 핀 `tree.pin`은 0161에서 걷었다 —
    /// 읽지도 쓰지도 않는다(옛 기록의 줄은 모르는 키처럼 그대로 지닌다).
    pub const TREE_BEAT: &str = "tree.beat";
    pub const TREE_RIPOSTE: &str = "tree.riposte";
    pub const TREE_FOOTING: &str = "tree.footing";
    /// 연 패시브 칸(0161 — 칸마다 따로): `tree.cells.<갈래>` = 연 자리들("1,2,4" — 1이 첫 칸). 연 칸이 없는 갈래는 쓰지 않는다.
    pub const TREE_CELLS: [&str; 3] = [
        "tree.cells.beat",
        "tree.cells.riposte",
        "tree.cells.footing",
    ];
    pub const TREE_COUNTS: [&str; 3] = [TREE_BEAT, TREE_RIPOSTE, TREE_FOOTING];
}

/// 글 기록(키 → 값). 모르는 키도 지키며 들고 다닌다.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Record {
    map: BTreeMap<String, String>,
}

impl Record {
    pub fn parse(text: &str) -> Record {
        let mut map = BTreeMap::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                let k = k.trim();
                if !k.is_empty() {
                    map.insert(k.to_string(), v.trim().to_string());
                }
            }
        }
        Record { map }
    }

    pub fn to_text(&self) -> String {
        let mut s = String::new();
        for (k, v) in &self.map {
            s.push_str(k);
            s.push('=');
            s.push_str(v);
            s.push('\n');
        }
        s
    }

    pub fn get(&self, k: &str) -> Option<&str> {
        self.map.get(k).map(String::as_str)
    }

    pub fn num(&self, k: &str) -> Option<u32> {
        self.get(k)?.parse().ok()
    }

    pub fn set(&mut self, k: &str, v: impl ToString) {
        self.map.insert(k.to_string(), v.to_string());
    }

    pub fn remove(&mut self, k: &str) {
        self.map.remove(k);
    }
}

/// 판 안의 자리: 쉼터 번호와 문이 열린 뒤인지, 어느 지역(0118)의 쉼터인지, 이 판에 이긴 문지기 수.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RunAt {
    pub rest: usize,
    pub gate_open: bool,
    pub stage: usize,
    pub down: usize,
}

/// 기록에서 게임이 쓰는 것(앱이 들고 있다가 바뀌면 쓴다).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Progress {
    pub embers: u32,
    pub clears: u32,
    pub run: Option<RunAt>,
    pub owned: Vec<crate::charms::Id>,
    pub worn: [Option<crate::charms::Id>; crate::charms::SLOTS],
    pub chests: Vec<u32>,
    pub growth: crate::growth::Growth,
    /// 잔불 레벨(0104).
    pub level: crate::progression::Level,
    /// 주운 숨은 물건의 기록 이름(0105).
    pub finds: Vec<String>,
    /// 처음 쓰러뜨린 엘리트의 기록 이름(0112).
    pub elites: Vec<String>,
    /// 시작한 판 수·주민을 처음 본 판·바뀐 주민(0122).
    pub runs: u32,
    pub residents_seen: Vec<(String, u32)>,
    pub residents_changed: Vec<String>,
    /// 도감에 적힌 것(0130).
    pub codex: Vec<String>,
    /// 문 장치로 연 문(0113).
    pub gates: Vec<String>,
    /// 성소 문지기 핵 점수(0105, `CORE_CAP`까지).
    pub cores: u32,
    /// 깨진 종지기 핵 점수(0125, `CORE_CAP`까지).
    pub bell_cores: u32,
    /// 든 무기(없으면 옛 prefs 칸의 값을 쓴다).
    pub weapon: Option<crate::weapons::Id>,
    /// 가진 무기(0117 — 무기는 얻어서 쓴다). 맨손은 넣지 않는다(늘 있다).
    pub weapons: Vec<crate::weapons::Id>,
    /// 끼운 스킬 둘(0151). 기록에 없으면 처음의 둘(`skills::START`). 배운 스킬만 끼운다(0158).
    pub skills: crate::skills::Worn,
    /// 끼운 큰 기술(0158 G2 — 하나). 기록에 없으면 쟁반 뒤집기. 배운 것만 끼운다.
    pub ult: crate::skills::Ult,
    /// 기술 판(0158 → 0161): 스킬마다의 단·고르기, 큰 기술의 단, 연 패시브 칸(`board.cells` — 옛 `tree`).
    pub board: crate::skills::Board,
    /// 쌓인 플레이 시간(초, 0081).
    pub play: u32,
    /// 원래 기록(모르는 키를 지키려고 — 쓸 때 이 위에 덮는다).
    base: Record,
}

impl Progress {
    pub fn from_record(r: Record) -> Progress {
        let run = r.num(key::RUN_REST).map(|i| RunAt {
            rest: i as usize,
            gate_open: r.num(key::RUN_GATE) == Some(1),
            stage: r.num(key::RUN_STAGE).unwrap_or(0) as usize,
            down: r.num(key::RUN_DOWN).unwrap_or(0) as usize,
        });
        let owned = crate::charms::decode_owned(r.get(key::CHARMS_OWNED).unwrap_or(""));
        let worn = crate::charms::decode_worn(r.get(key::CHARMS_WORN).unwrap_or(""), &owned);
        let mut chests: Vec<u32> = r
            .get(key::CHESTS)
            .unwrap_or("")
            .split(',')
            .filter_map(|c| c.trim().parse().ok())
            .collect();
        chests.sort();
        chests.dedup();
        let small = |k: &str| r.num(k).unwrap_or(0).min(u8::MAX as u32) as u8;
        let growth = crate::growth::Growth {
            hp: small(key::GROW_HP),
            flask: small(key::GROW_FLASK),
            heal: small(key::GROW_HEAL),
            oil: r.num(key::SHOP_OIL) == Some(1),
        };
        let rank = |k: &str| {
            r.num(k)
                .unwrap_or(0)
                .min(crate::progression::MAX_RANK as u32) as u8
        };
        let level = crate::progression::Level {
            life: rank(key::LV_LIFE),
            stamina: rank(key::LV_STAMINA),
            might: rank(key::LV_MIGHT),
        };
        let mut finds: Vec<String> = r
            .get(key::FINDS)
            .unwrap_or("")
            .split(',')
            .map(str::trim)
            .filter(|f| !f.is_empty())
            .map(str::to_string)
            .collect();
        finds.sort();
        finds.dedup();
        let mut elites: Vec<String> = r
            .get(key::ELITES)
            .unwrap_or("")
            .split(',')
            .map(str::trim)
            .filter(|f| !f.is_empty())
            .map(str::to_string)
            .collect();
        elites.sort();
        elites.dedup();
        let mut gates: Vec<String> = r
            .get(key::GATES)
            .unwrap_or("")
            .split(',')
            .map(str::trim)
            .filter(|f| !f.is_empty())
            .map(str::to_string)
            .collect();
        gates.sort();
        gates.dedup();
        let cores = r
            .num(key::CORE_WARDEN)
            .unwrap_or(0)
            .min(crate::progression::CORE_CAP);
        let bell_cores = r
            .num(key::CORE_BELL)
            .unwrap_or(0)
            .min(crate::progression::CORE_CAP);
        let migrated = r.num(key::MIGRATED_V04) == Some(1);
        // 패시브 칸(0161): 갈래마다 `tree.cells.*`(연 자리들)가 있으면 그것, 없으면 옛 꼴(연 수 N = 앞에서부터 N칸).
        let cells = {
            use crate::progression::{Tree, DEPTH};
            let mut t = Tree::default();
            for b in 0..3 {
                t.cells[b] = match r.get(key::TREE_CELLS[b]) {
                    Some(list) => list
                        .split(',')
                        .filter_map(|c| c.trim().parse::<u8>().ok())
                        .filter(|d| (1..=DEPTH).contains(d))
                        .fold(0, |m, d| m | 1 << (d - 1)),
                    None => {
                        let n = r.num(key::TREE_COUNTS[b]).unwrap_or(0).min(DEPTH as u32) as u8;
                        Tree::from_counts([n, 0, 0]).cells[0]
                    }
                };
            }
            t
        };
        let residents_seen: Vec<(String, u32)> = r
            .get(key::RESIDENTS_SEEN)
            .unwrap_or("")
            .split(',')
            .filter_map(|e| {
                let (id, n) = e.trim().split_once(':')?;
                Some((id.to_string(), n.parse().ok()?))
            })
            .collect();
        let mut residents_changed: Vec<String> = r
            .get(key::RESIDENTS_CHANGED)
            .unwrap_or("")
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();
        residents_changed.sort();
        residents_changed.dedup();
        let mut codex: Vec<String> = r
            .get(key::CODEX)
            .unwrap_or("")
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();
        codex.sort();
        codex.dedup();
        let runs = r.num(key::RUNS).unwrap_or(0);
        // 기술 판(0158): 키가 없으면 처음 그대로. 범위·점수는 아래에서 바로잡는다(`Board::sane`).
        let board = {
            use crate::skills::{Board, Pick, IDS, ULTS};
            let mut b = Board::default();
            let small = |k: String| r.num(&k).map(|n| n.min(u8::MAX as u32) as u8);
            for (i, id) in IDS.into_iter().enumerate() {
                if let Some(n) = small(format!("{}{}", key::SKILL_RANK, id.key())) {
                    b.rank[i] = n;
                }
                b.pick[i] = r
                    .get(&format!("{}{}", key::SKILL_PICK, id.key()))
                    .and_then(Pick::by_key)
                    .unwrap_or_default();
            }
            for (i, u) in ULTS.into_iter().enumerate() {
                if let Some(n) = small(format!("{}{}", key::ULT_RANK, u.key())) {
                    b.ult[i] = n;
                }
            }
            b.cells = cells;
            b
        };
        let r_has_no_weapons = r.get(key::WEAPONS).is_none();
        let r_weapons = r.get(key::WEAPONS).map(str::to_string);
        let mut p = Progress {
            growth,
            level,
            finds,
            elites,
            runs,
            residents_seen,
            residents_changed,
            codex,
            gates,
            cores,
            bell_cores,
            weapon: r.get(key::WEAPON).and_then(crate::weapons::Id::by_key),
            skills: {
                let at = |k: &str, d: crate::skills::Id| {
                    r.get(k).and_then(crate::skills::Id::by_key).unwrap_or(d)
                };
                crate::skills::Worn(crate::skills::sane([
                    at(key::SKILL1, crate::skills::START[0]),
                    at(key::SKILL2, crate::skills::START[1]),
                ]))
            },
            ult: r
                .get(key::ULT_WORN)
                .and_then(crate::skills::Ult::by_key)
                .unwrap_or_default(),
            board,
            weapons: r_weapons
                .map(|v| {
                    v.split(',')
                        .filter_map(|k| crate::weapons::Id::by_key(k.trim()))
                        .filter(|&w| w != crate::weapons::Id::Fists)
                        .collect()
                })
                .unwrap_or_default(),
            play: r.num(key::PLAY).unwrap_or(0),
            embers: r.num(key::EMBERS).unwrap_or(0),
            clears: r.num(key::CLEARS).unwrap_or(0),
            run,
            owned,
            worn,
            chests,
            base: r,
        };
        if !migrated {
            migrate_v04(&mut p);
        }
        // 0117: 가진 무기 키가 없는 옛 기록은 — 판을 한 번이라도 했으면 무기를 다 가진 채로(잃는 것 없이), 새 기록은 검만.
        if r_has_no_weapons && p.has_any() {
            p.weapons = crate::weapons::IDS
                .into_iter()
                .filter(|&w| w != crate::weapons::Id::Fists)
                .collect();
        }
        // 기술 판(0158): 가진 점수 안으로 바로잡고, 안 배운 스킬이 끼워져 있으면 처음의 둘로.
        p.board = p.board.sane(p.skill_points());
        p.skills = crate::skills::Worn(p.board.wearable(p.skills.0));
        p.ult = p.board.wearable_ult(p.ult);
        p
    }

    /// 기술 점수(0158 나 → 0161)의 합: 총 레벨 + 처음 쓰러뜨린 엘리트 수 + 핵(문지기마다 `CORE_CAP`까지) + 주운 종소리 조각 —
    /// 셈이라 옛 기록도 가진다(옛 나무 화폐 "마디" = 핵 + 조각이 그대로 점수가 된다).
    pub fn skill_points(&self) -> u32 {
        use crate::progression::{shards, CORE_CAP};
        crate::skills::points(
            self.level.total(),
            self.elites.len() as u32,
            self.cores.min(CORE_CAP) + self.bell_cores.min(CORE_CAP),
            shards(&self.finds),
        )
    }

    /// 이 무기를 가졌나(0117): 맨손·검은 늘, 그 밖은 얻은 것만.
    pub fn owns_weapon(&self, w: crate::weapons::Id) -> bool {
        matches!(w, crate::weapons::Id::Fists | crate::weapons::Id::Sword)
            || self.weapons.contains(&w)
    }

    /// 가진 무기(표 순서, 맨손 포함).
    pub fn owned_weapons(&self) -> Vec<crate::weapons::Id> {
        crate::weapons::IDS
            .into_iter()
            .filter(|&w| self.owns_weapon(w))
            .collect()
    }

    pub fn to_record(&self) -> Record {
        let mut r = self.base.clone();
        r.set(key::VERSION, 1);
        r.set(key::EMBERS, self.embers);
        r.set(key::CLEARS, self.clears);
        r.set(key::CHARMS_OWNED, crate::charms::encode_owned(&self.owned));
        r.set(key::CHARMS_WORN, crate::charms::encode_worn(&self.worn));
        let chests: Vec<String> = self.chests.iter().map(|c| c.to_string()).collect();
        r.set(key::CHESTS, chests.join(","));
        r.set(key::GROW_HP, self.growth.hp);
        r.set(key::GROW_FLASK, self.growth.flask);
        r.set(key::GROW_HEAL, self.growth.heal);
        r.set(key::SHOP_OIL, self.growth.oil as u8);
        r.set(key::PLAY, self.play);
        r.set(key::LV_LIFE, self.level.life);
        r.set(key::LV_STAMINA, self.level.stamina);
        r.set(key::LV_MIGHT, self.level.might);
        r.set(key::FINDS, self.finds.join(","));
        r.set(key::ELITES, self.elites.join(","));
        r.set(key::RUNS, self.runs);
        r.set(
            key::RESIDENTS_SEEN,
            self.residents_seen
                .iter()
                .map(|(id, n)| format!("{id}:{n}"))
                .collect::<Vec<_>>()
                .join(","),
        );
        r.set(key::RESIDENTS_CHANGED, self.residents_changed.join(","));
        r.set(key::CODEX, self.codex.join(","));
        r.set(key::GATES, self.gates.join(","));
        r.set(key::CORE_WARDEN, self.cores);
        r.set(key::CORE_BELL, self.bell_cores);
        r.set(key::MIGRATED_V04, 1);
        // 패시브 칸(0161): 새 키는 연 자리들, 옛 키는 앞에서부터 이어 연 수(키 이름은 그대로 — 옛 판이 읽어도 터지지 않는다).
        for b in 0..3 {
            let cells = self.board.cells;
            r.set(key::TREE_COUNTS[b], cells.prefix(b));
            let open: Vec<String> = (0..crate::progression::DEPTH)
                .filter(|d| cells.mask(b) & (1 << d) != 0)
                .map(|d| (d + 1).to_string())
                .collect();
            if open.is_empty() {
                r.remove(key::TREE_CELLS[b]);
            } else {
                r.set(key::TREE_CELLS[b], open.join(","));
            }
        }
        match self.weapon {
            Some(w) => r.set(key::WEAPON, w.key()),
            None => r.remove(key::WEAPON),
        }
        // 처음의 둘이면 쓰지 않는다(기록이 없는 슬롯이 "기록 있음"이 되지 않게).
        if self.skills.0 == crate::skills::START {
            r.remove(key::SKILL1);
            r.remove(key::SKILL2);
        } else {
            r.set(key::SKILL1, self.skills.0[0].key());
            r.set(key::SKILL2, self.skills.0[1].key());
        }
        // 기술 판(0158): 처음 그대로인 칸은 쓰지 않는다(빈 슬롯이 "기록 있음"이 되지 않게, 옛 판으로 읽어도 그대로).
        {
            use crate::skills::{Entry, Ult, IDS, PICK_RANK, ULTS};
            for (i, id) in IDS.into_iter().enumerate() {
                let (rk, pk) = (
                    format!("{}{}", key::SKILL_RANK, id.key()),
                    format!("{}{}", key::SKILL_PICK, id.key()),
                );
                if self.board.rank[i] == Entry::Skill(id).free() {
                    r.remove(&rk);
                } else {
                    r.set(&rk, self.board.rank[i]);
                }
                if self.board.rank[i] >= PICK_RANK {
                    r.set(&pk, self.board.pick[i].key());
                } else {
                    r.remove(&pk);
                }
            }
            for (i, u) in ULTS.into_iter().enumerate() {
                let uk = format!("{}{}", key::ULT_RANK, u.key());
                if self.board.ult[i] == Entry::Ult(u).free() {
                    r.remove(&uk);
                } else {
                    r.set(&uk, self.board.ult[i]);
                }
            }
            if self.ult == Ult::Upend {
                r.remove(key::ULT_WORN);
            } else {
                r.set(key::ULT_WORN, self.ult.key());
            }
        }
        r.set(
            key::WEAPONS,
            self.weapons
                .iter()
                .map(|w| w.key())
                .collect::<Vec<_>>()
                .join(","),
        );
        match self.run {
            Some(at) => {
                r.set(key::RUN_REST, at.rest);
                r.set(key::RUN_GATE, at.gate_open as u8);
                r.set(key::RUN_STAGE, at.stage);
                r.set(key::RUN_DOWN, at.down);
            }
            None => {
                r.remove(key::RUN_REST);
                r.remove(key::RUN_GATE);
                r.remove(key::RUN_STAGE);
                r.remove(key::RUN_DOWN);
            }
        }
        r
    }

    /// 이어할 것이 있나(타이틀 "이어하기"): 판 밖의 것이 하나라도 쌓였거나 판 안의 자리가 있으면.
    pub fn has_any(&self) -> bool {
        self.run.is_some()
            || self.play > 0
            || self.embers > 0
            || self.clears > 0
            || !self.owned.is_empty()
            || self.growth != crate::growth::Growth::default()
            || self.level != crate::progression::Level::default()
            || !self.finds.is_empty()
            || self.cores > 0
            || self.bell_cores > 0
            || self.board.cells.count() > 0
    }

    /// 장신구를 뺀 능력치(성장만) — 장비 창이 "장신구가 바꾸는 줄"을 고를 때의 기본값(0104).
    pub fn stats_without_charms(&self) -> crate::stats::Stats {
        crate::stats::Stats::compute(&[
            self.growth.mods(),
            self.level.mods(),
            crate::progression::find_mods(&self.finds),
        ])
    }

    /// 낀 장신구 + 성장(핵 고르기·레벨)으로 합친 능력치.
    pub fn stats(&self) -> crate::stats::Stats {
        let mut mods = crate::charms::mods(&self.worn);
        mods.push(self.growth.mods());
        mods.push(self.level.mods());
        mods.push(crate::progression::find_mods(&self.finds));
        crate::stats::Stats::compute(&mods)
    }
}

/// v0.4.0 옮기기(0105, 0096 L5 "잃은 것 없이"): 핵 고르기로 얻은 것을 새 자리로 — 한 번만(깃발 `migrated.v04`).
/// - 체력 +n → 생명 n단(5단을 넘는 몫은 `growth.hp`에 남아 그대로 보정).
/// - 등잔 +n → 성소의 숨은 등잔을 주운 것으로(하나), 남는 몫은 `growth.flask`에.
/// - 한 모금 +n → 성소의 숨은 한 모금을 주운 것으로(둘까지), 남는 몫은 `growth.heal`에.
/// - 이긴 횟수 → 문지기 핵 점수(`CORE_CAP`까지).
pub fn migrate_v04(p: &mut Progress) {
    use crate::stage::{find_id, Find};
    let prefix = crate::stage::sanctuary::DEF.find_prefix;
    let room = crate::progression::MAX_RANK - p.level.life;
    let moved = p.growth.hp.min(room);
    p.level.life += moved;
    p.growth.hp -= moved;
    let mut take = |kind: Find, have: &mut u8, slots: usize| {
        for n in 0..slots {
            let id = find_id(prefix, kind, n);
            if *have > 0 && !p.finds.contains(&id) {
                p.finds.push(id);
                *have -= 1;
            }
        }
    };
    let (mut flask, mut heal) = (p.growth.flask, p.growth.heal);
    take(Find::Lamp, &mut flask, 1);
    take(Find::Sip, &mut heal, 2);
    p.growth.flask = flask;
    p.growth.heal = heal;
    p.finds.sort();
    p.cores = p.cores.max(p.clears.min(crate::progression::CORE_CAP));
}

/// 기록 슬롯 수(0066, KB `roguelite-save-slots-and-continue`, nail 0078).
pub const SLOTS: usize = 3;
/// 저장 칸: 0..SLOTS = 슬롯 기록, `LAST` = 마지막으로 쓴 슬롯 번호.
const LAST: u32 = SLOTS as u32;

#[cfg(target_arch = "wasm32")]
mod store {
    extern "C" {
        fn knell_rec_len(cell: u32) -> u32;
        fn knell_rec_read(cell: u32, ptr: *mut u8, len: u32);
        fn knell_rec_write(cell: u32, ptr: *const u8, len: u32) -> u32;
    }

    pub fn load(cell: u32) -> String {
        // SAFETY: 칸의 길이를 돌려주는 JS 함수, 그다음 그 길이만큼 이 버퍼에 바이트를 쓰는 JS 함수(web/index.html).
        let n = unsafe { knell_rec_len(cell) } as usize;
        let mut buf = vec![0u8; n];
        if n > 0 {
            unsafe { knell_rec_read(cell, buf.as_mut_ptr(), n as u32) };
        }
        String::from_utf8(buf).unwrap_or_default()
    }

    pub fn save(cell: u32, text: &str) -> bool {
        // SAFETY: 이 바이트를 읽기만 하는 JS 함수(web/index.html). 길이 0이면 지운다.
        unsafe { knell_rec_write(cell, text.as_ptr(), text.len() as u32) == 1 }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod store {
    use std::path::PathBuf;

    /// 슬롯 1 = 옛 한 칸 `save0.txt` 그대로(옮기는 코드 없음), 마지막 슬롯 = `slot.txt`.
    fn path(cell: u32) -> Option<PathBuf> {
        let name = if cell == super::LAST {
            "slot.txt".to_string()
        } else {
            format!("save{cell}.txt")
        };
        crate::prefs::data_dir().map(|d| d.join(name))
    }

    pub fn load(cell: u32) -> String {
        path(cell)
            .and_then(|p| std::fs::read_to_string(p).ok())
            .unwrap_or_default()
    }

    pub fn save(cell: u32, text: &str) -> bool {
        let Some(p) = path(cell) else { return false };
        if text.is_empty() {
            return std::fs::remove_file(&p).is_ok() || !p.exists();
        }
        let result = (|| {
            if let Some(d) = p.parent() {
                std::fs::create_dir_all(d)?;
            }
            let tmp = p.with_extension("tmp");
            std::fs::write(&tmp, text)?;
            std::fs::rename(&tmp, &p)
        })();
        result.is_ok()
    }
}

pub fn load(slot: usize) -> Progress {
    Progress::from_record(Record::parse(&store::load(slot as u32)))
}

/// 쓴다(실패하면 거짓 — 사생활 보호 모드·디스크 등).
pub fn save(slot: usize, p: &Progress) -> bool {
    store::save(slot as u32, &p.to_record().to_text())
}

/// 슬롯의 기록을 지운다(슬롯 창 "지우기"·"덮어쓰기").
pub fn wipe(slot: usize) -> bool {
    store::save(slot as u32, "")
}

/// 마지막으로 쓴 슬롯(없거나 깨졌으면 0 = 슬롯 1 — 옛 한 칸 기록이 있는 곳).
pub fn last_slot() -> usize {
    parse_slot(&store::load(LAST))
}

pub fn set_last_slot(slot: usize) -> bool {
    store::save(LAST, &slot.to_string())
}

fn parse_slot(text: &str) -> usize {
    text.trim()
        .parse::<usize>()
        .ok()
        .filter(|&s| s < SLOTS)
        .unwrap_or(0)
}

/// 슬롯 창 한 줄의 요약(기록에서 바로 계산되는 것, KB).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Summary {
    /// 장(챕터) 번호, 1부터 — 지금은 스테이지 하나라 늘 1(v0.4.0 구역 잇기 0069 T1에서 `run.stage`로).
    pub chapter: u32,
    /// 쌓인 플레이 시간(초).
    pub play: u32,
}

impl Progress {
    /// 기록이 있는 슬롯이면 요약(장·지역·시간, 사장 2026-11-07 — 그 밖은 뺌), 비었으면 없음.
    pub fn summary(&self) -> Option<Summary> {
        self.has_any().then_some(Summary {
            chapter: 1,
            play: self.play,
        })
    }
}

/// 플레이 시간 글 "h:mm"(초는 뺀다 — 슬롯 줄은 대강이면 된다).
pub fn play_text(secs: u32) -> String {
    format!("{}:{:02}", secs / 3600, secs / 60 % 60)
}

/// 슬롯 셋의 요약(타이틀 "이어하기"가 보이나, 슬롯 창의 줄).
pub fn summaries() -> [Option<Summary>; SLOTS] {
    std::array::from_fn(|i| load(i).summary())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_record_round_trips_keeps_unknown_keys_and_skips_broken_lines() {
        let text =
            "embers=12\nfuture.key=hello = world\n\nbroken line\n=nokey\n# comment\nclears=x\n";
        let r = Record::parse(text);
        assert_eq!(r.num("embers"), Some(12));
        assert_eq!(r.get("future.key"), Some("hello = world"));
        assert_eq!(r.num("clears"), None, "not a number");
        let again = Record::parse(&r.to_text());
        assert_eq!(again, r);
        // 게임이 쓰는 칸을 바꿔 써도 모르는 키는 남는다.
        let mut p = Progress::from_record(r);
        p.embers = 30;
        let out = p.to_record();
        assert_eq!(out.get("future.key"), Some("hello = world"));
        assert_eq!(out.num(key::EMBERS), Some(30));
        assert_eq!(out.num(key::VERSION), Some(1));
    }

    /// 0112: 처음 쓰러뜨린 엘리트는 기록에 남는다(키 `elites`, 겹치지 않게).
    /// 0130: 도감 기록은 정렬된 이름 목록으로 돌아온다(빈 기록 = 없음).
    #[test]
    fn the_codex_round_trips() {
        let p = Progress {
            codex: vec!["husk".into(), "husk.docile".into(), "warden.roused".into()],
            ..Progress::default()
        };
        let back = Progress::from_record(Record::parse(&p.to_record().to_text()));
        assert_eq!(back.codex, p.codex);
        assert!(Progress::from_record(Record::parse("v=1\n"))
            .codex
            .is_empty());
    }

    /// 0125: 깨진 종지기 핵은 따로 센다(셋까지) — 기술 점수는 두 문지기 핵을 더한다(0161). 키가 없는 옛 기록은 0.
    #[test]
    fn bellkeeper_cores_are_kept_apart_and_add_to_skill_points() {
        let p = Progress {
            cores: 2,
            bell_cores: 9,
            ..Progress::default()
        };
        let back = Progress::from_record(Record::parse(&p.to_record().to_text()));
        assert_eq!(back.cores, 2);
        assert_eq!(back.bell_cores, crate::progression::CORE_CAP, "capped");
        assert_eq!(back.skill_points(), 2 + crate::progression::CORE_CAP);
        assert!(back.has_any());
        let old = Progress::from_record(Record::parse("v=1\ncore.warden=1\n"));
        assert_eq!((old.cores, old.bell_cores), (1, 0));
    }

    /// 0117: 무기는 얻어서 쓴다 — 새 기록은 맨손·검만, 얻은 무기는 기록에 남고, 키가 없는 옛 기록(판을 한 번이라도 한)은 다 가진 채로.
    #[test]
    fn weapons_are_earned_and_old_records_keep_them_all() {
        use crate::weapons::Id;
        let fresh = Progress::default();
        assert_eq!(fresh.owned_weapons(), vec![Id::Sword, Id::Fists]);
        // 빈 목록도 키로 남아 다시 읽어도 옮기기가 돌지 않는다.
        let mut p = Progress {
            play: 3,
            ..Progress::default()
        };
        let back = Progress::from_record(Record::parse(&p.to_record().to_text()));
        assert!(!back.owns_weapon(Id::Hammer), "{:?}", back.weapons);
        p.weapons.push(Id::Hammer);
        let back = Progress::from_record(Record::parse(&p.to_record().to_text()));
        assert!(back.owns_weapon(Id::Hammer));
        assert!(!back.owns_weapon(Id::Staff) && !back.owns_weapon(Id::Crossbow));
        // 옛 기록: 무기 키 없음 + 판을 했다 → 다 가진다. 아무것도 없는 기록은 새 기록과 같다.
        let old = Progress::from_record(Record::parse(
            "v=1
embers=12
weapon=crossbow
",
        ));
        assert!(crate::weapons::IDS.iter().all(|&w| old.owns_weapon(w)));
        let empty = Progress::from_record(Record::parse(""));
        assert_eq!(empty.owned_weapons(), vec![Id::Sword, Id::Fists]);
    }

    /// 0122: 판 수·주민을 처음 본 판·바뀐 주민은 기록에 남고, 키가 없는 옛 기록은 0·빈 것(NPC-07).
    #[test]
    fn residents_and_runs_round_trip_and_old_records_start_empty() {
        let mut p = Progress {
            runs: 4,
            ..Progress::default()
        };
        p.residents_seen = vec![("V01".into(), 2), ("V03".into(), 4)];
        p.residents_changed = vec!["V01".into()];
        let back = Progress::from_record(Record::parse(&p.to_record().to_text()));
        assert_eq!(back.runs, 4);
        assert_eq!(back.residents_seen, p.residents_seen);
        assert_eq!(back.residents_changed, p.residents_changed);
        let old = Progress::from_record(Record::parse("v=1\nembers=9\n"));
        assert_eq!(
            (
                old.runs,
                old.residents_seen.len(),
                old.residents_changed.len()
            ),
            (0, 0, 0)
        );
    }

    #[test]
    fn beaten_elites_round_trip() {
        let mut p = Progress::default();
        p.elites = vec!["sanctuary.elite.1".into(), "sanctuary.elite.0".into()];
        let r = p.to_record();
        assert_eq!(
            r.get(key::ELITES),
            Some("sanctuary.elite.1,sanctuary.elite.0")
        );
        let back = Progress::from_record(Record::parse(
            &(r.to_text() + "elites=sanctuary.elite.0,sanctuary.elite.1,sanctuary.elite.0\n"),
        ));
        assert_eq!(back.elites, vec!["sanctuary.elite.0", "sanctuary.elite.1"]);
        // 0113: 문 장치로 연 문도 같은 꼴(키 `gates`).
        let mut p = Progress::default();
        p.gates = vec!["sanctuary".into()];
        let back = Progress::from_record(Record::parse(&p.to_record().to_text()));
        assert_eq!(back.gates, vec!["sanctuary"]);
    }

    #[test]
    fn progress_keeps_the_rest_site_and_drops_it_when_the_run_ends() {
        let mut p = Progress::default();
        assert!(!p.has_any(), "nothing yet");
        p.run = Some(RunAt {
            rest: 1,
            gate_open: true,
            stage: 1,
            down: 1,
        });
        let back = Progress::from_record(Record::parse(&p.to_record().to_text()));
        assert_eq!(back.run, p.run);
        assert!(back.has_any());
        // 판이 끝나면 자리는 지우고 남는 것만.
        let mut done = back.clone();
        done.run = None;
        done.clears = 1;
        let r = done.to_record();
        assert_eq!(r.get(key::RUN_REST), None);
        assert_eq!(r.get(key::RUN_GATE), None);
        let back = Progress::from_record(r);
        assert_eq!(back.run, None);
        assert!(back.has_any(), "a clear stays");
    }

    /// 0105(0096 L5): 옛 핵 고르기는 잃는 것 없이 옮겨지고, 두 번째 읽기에서는 다시 옮기지 않는다.
    #[test]
    fn v03_core_picks_move_to_levels_finds_and_core_points_once() {
        let old = "v=1\nembers=40\nclears=5\ngrow.hp=3\ngrow.flask=2\ngrow.heal=2\nshop.oil=1\n";
        let before = {
            // 옛 셈(핵 고르기 그대로): 체력 +3, 등잔 +2 + 기름, 한 모금 +2 — 기록의 수는 몫의 수 그대로, 한 몫 = 10(0158 N0).
            let g = crate::growth::Growth {
                hp: 3,
                flask: 2,
                heal: 2,
                oil: true,
            };
            crate::stats::Stats::compute(&[g.mods()])
        };
        let p = Progress::from_record(Record::parse(old));
        assert_eq!(p.level.life, 3);
        assert_eq!(p.growth.hp, 0);
        assert_eq!(
            p.finds,
            vec![
                "sanctuary.lamp.0".to_string(),
                "sanctuary.sip.0".to_string(),
                "sanctuary.sip.1".to_string()
            ]
        );
        assert_eq!(
            (p.growth.flask, p.growth.heal),
            (1, 0),
            "the second lamp stays as leftover"
        );
        assert_eq!(p.cores, crate::progression::CORE_CAP, "five wins, capped");
        let after = p.stats();
        assert_eq!(
            (after.max_hp, after.flask, after.heal),
            (before.max_hp, before.flask, before.heal),
            "nothing lost"
        );
        assert_eq!(
            (after.max_hp, after.heal),
            (80, 20),
            "old 5 + 3 hearts and +2 sip, times ten"
        );
        // 다시 읽어도 그대로(깃발).
        let again = Progress::from_record(Record::parse(&p.to_record().to_text()));
        assert_eq!(again.level, p.level);
        assert_eq!(again.finds, p.finds);
        assert_eq!(again.growth, p.growth);
        assert_eq!(again.stats(), after);

        // 생명을 이미 올렸으면 5단을 넘는 몫은 남은 보정으로.
        let p = Progress::from_record(Record::parse("v=1\ngrow.hp=3\nlv.life=4\n"));
        assert_eq!((p.level.life, p.growth.hp), (5, 2));
        // 새 기록은 옮길 것이 없다.
        let p = Progress::from_record(Record::parse(""));
        assert!(p.finds.is_empty() && p.cores == 0 && !p.has_any());
    }

    /// 0161: 패시브 칸은 칸마다 따로 남는다(`tree.cells.<갈래>` = 연 자리들) — 사이가 빈 나무도 그대로 왕복하고, 옛 키에는
    /// 앞에서부터 이어 연 수가 적힌다. 연 칸이 없는 갈래는 새 키를 쓰지 않는다. 고친 기록의 이상한 자리는 걸러진다.
    #[test]
    fn open_cells_round_trip_one_by_one() {
        use crate::progression::{Cell, Tree};
        // 칸 넷(쓴 점수 4)을 가질 만큼의 점수: 핵 셋 + 조각 하나.
        let rich = Progress {
            cores: 3,
            finds: vec!["sanctuary.shard.0".into()],
            ..Progress::default()
        };
        let mut p = rich.clone();
        p.board.cells = Tree::with(&[
            Cell::EvenBreath,
            Cell::FlowingBeat,
            Cell::Sidestep,
            Cell::FinalMeasure,
        ]);
        assert_eq!((p.board.spent(), p.skill_points()), (4, 4));
        let r = p.to_record();
        assert_eq!(r.get("tree.cells.beat"), Some("1,2,5"));
        assert_eq!(r.get("tree.cells.riposte"), None, "nothing open there");
        assert_eq!(r.get("tree.cells.footing"), Some("2"));
        assert_eq!(
            (
                r.num(key::TREE_BEAT),
                r.num(key::TREE_RIPOSTE),
                r.num(key::TREE_FOOTING)
            ),
            (Some(2), Some(0), Some(0)),
            "the old keys keep the leading run"
        );
        let back = Progress::from_record(Record::parse(&r.to_text()));
        assert_eq!(back.board, p.board);
        assert!(back.has_any());
        // 칸을 다 닫으면(되돌리기) 새 키가 지워지고 옛 키는 0 — 다시 읽어도 닫힌 채.
        let mut shut = back.clone();
        shut.board.cells = Tree::default();
        let r = shut.to_record();
        assert_eq!(r.get("tree.cells.beat"), None);
        assert_eq!(r.num(key::TREE_BEAT), Some(0));
        assert_eq!(
            Progress::from_record(Record::parse(&r.to_text()))
                .board
                .cells,
            Tree::default()
        );
        // 손댄 기록: 범위 밖 자리·글자는 버리고 아는 자리만.
        let mut odd = rich.to_record();
        odd.set("tree.cells.beat", "0,3,9,x,3");
        assert_eq!(
            Progress::from_record(odd).board.cells,
            Tree::with(&[Cell::ClosingBlow])
        );
    }

    /// 0161: 옛 기록(`tree.beat = N` — 갈래마다 연 수)은 앞에서부터 N칸을 연 것으로 읽히고, 그 칸들은 쓴 점수다 — 핵·조각이
    /// 이제 기술 점수라 잃는 것이 없다. 옛 목표 핀(`tree.pin`)은 읽지 않고 줄만 그대로 지닌다. 큰 수는 다섯에서 멈춘다.
    #[test]
    fn old_tree_counts_read_as_the_first_cells_and_count_as_spent_points() {
        use crate::progression::{Cell, Tree};
        // v0.5 기록: 핵 셋 + 둘째 문지기 핵 하나 + 조각 둘 = 마디 6, 박자 2칸·발걸음 3칸을 열었다(쓴 마디 5). 레벨 4.
        let old = Progress::from_record(Record::parse(
            "v=1\nmigrated.v04=1\nlv.life=2\nlv.might=2\ncore.warden=3\ncore.bellkeeper=1\n\
             finds=sanctuary.shard.0,sanctuary.shard.1,sanctuary.lamp.0\ntree.beat=2\ntree.footing=3\ntree.pin=1.3\n",
        ));
        assert_eq!(old.board.cells, Tree::from_counts([2, 0, 3]));
        for c in [
            Cell::EvenBreath,
            Cell::FlowingBeat,
            Cell::UnbrokenStride,
            Cell::Sidestep,
            Cell::TiltingGround,
        ] {
            assert!(old.board.cells.has(c), "{c:?}");
        }
        assert!(!old.board.cells.has(Cell::ClosingBlow) && !old.board.cells.has(Cell::Catch));
        // 점수: 레벨 4 + 핵 4 + 조각 2 = 10, 쓴 점수 5(연 칸) — 남은 5.
        assert_eq!(old.skill_points(), 10);
        assert_eq!(
            (old.board.spent(), old.board.left(old.skill_points())),
            (5, 5)
        );
        // 다시 쓰면 새 키가 생기고 옛 키·옛 핀 줄은 남는다.
        let r = old.to_record();
        assert_eq!(r.get("tree.cells.beat"), Some("1,2"));
        assert_eq!(r.get("tree.cells.footing"), Some("1,2,3"));
        assert_eq!(r.num(key::TREE_FOOTING), Some(3));
        assert_eq!(r.get("tree.pin"), Some("1.3"), "kept like any unknown key");
        assert_eq!(Progress::from_record(r).board.cells, old.board.cells);
        // 새 키가 있으면 옛 키보다 먼저다(새 판이 칸을 닫은 뒤의 기록).
        let both = Progress::from_record(Record::parse(
            "core.warden=3\ntree.beat=3\ntree.cells.beat=4\n",
        ));
        assert_eq!(both.board.cells, Tree::with(&[Cell::GapBeat]));
        // 고친 기록의 큰 수는 다섯 칸.
        let big = Progress::from_record(Record::parse(
            "core.warden=3\ncore.bellkeeper=3\ntree.riposte=40\n",
        ));
        assert_eq!(big.board.cells, Tree::from_counts([0, 5, 0]));
        // 가진 점수보다 많이 연 기록(손댄 것)은 터지지 않고 닫힌 채로 읽힌다.
        let broke = Progress::from_record(Record::parse("tree.beat=5\ntree.footing=5\n"));
        assert_eq!(broke.board, crate::skills::Board::default());
        assert_eq!(broke.skill_points(), 0);
    }

    /// 0161: 개발판 기록(0158 G0·G2 — 마디로 연 칸 + 기술 점수로 올린 단)이 새 셈에서 한두 점 넘치면(옛 "보스 첫 쓰러뜨림 +1"이
    /// 핵의 수로 바뀌어서) 스킬·큰 기술만 처음으로 돌아가고 연 패시브 칸은 남는다 — 점수는 남아 있어 다시 찍으면 된다.
    #[test]
    fn a_dev_record_one_point_over_keeps_its_cells_and_resets_the_skills() {
        use crate::progression::Tree;
        // 핵 1 + 핵 1(= 옛 마디 2 → 칸 둘) · 레벨 2 + 옛 보스 둘 +2 = 옛 점수 4를 스킬에 다 썼다. 새 점수는 레벨 2 + 핵 2 = 4, 쓴 것 6.
        let p = Progress::from_record(Record::parse(
            "lv.life=2\ncore.warden=1\ncore.bellkeeper=1\ntree.beat=2\nskill.rank.whirl=3\nskill.rank.cut=3\n",
        ));
        assert_eq!(p.skill_points(), 4);
        assert_eq!(p.board.cells, Tree::from_counts([2, 0, 0]));
        assert_eq!(p.board.rank[..2], [1, 1], "back to the starters");
        assert_eq!((p.board.spent(), p.board.left(4)), (2, 2));
    }

    /// 0104: 레벨은 `lv.*` 셋으로 남고, 고친 기록의 큰 수는 5단에서 멈춘다. 레벨만 있어도 기록이 있는 슬롯.
    #[test]
    fn levels_survive_the_record_and_clamp_to_five() {
        let mut p = Progress::default();
        p.level = crate::progression::Level {
            life: 2,
            stamina: 5,
            might: 1,
        };
        assert!(p.has_any());
        let r = p.to_record();
        assert_eq!(
            (
                r.num(key::LV_LIFE),
                r.num(key::LV_STAMINA),
                r.num(key::LV_MIGHT)
            ),
            (Some(2), Some(5), Some(1))
        );
        let back = Progress::from_record(Record::parse(&r.to_text()));
        assert_eq!(back.level, p.level);
        assert_eq!(back.stats(), p.stats());
        let mut odd = r.clone();
        odd.set(key::LV_MIGHT, 99);
        assert_eq!(Progress::from_record(odd).level.might, 5);
    }

    #[test]
    fn charms_and_opened_chests_survive_the_record() {
        use crate::charms::Id;
        let mut p = Progress::default();
        p.owned = vec![Id::Shell, Id::Ear];
        p.worn = [None, Some(Id::Ear), None];
        p.chests = vec![0, 2];
        p.growth = crate::growth::Growth {
            hp: 2,
            flask: 1,
            heal: 0,
            oil: true,
        };
        let back = Progress::from_record(Record::parse(&p.to_record().to_text()));
        assert_eq!(
            (back.owned.clone(), back.worn, back.chests.clone()),
            (p.owned, p.worn, p.chests)
        );
        assert!(back.has_any());
        assert_eq!(back.stats().parry, 0.04, "the worn charm counts");
        assert_eq!(back.growth, p.growth);
        let base = crate::stats::Stats::default();
        assert_eq!(
            back.stats().max_hp,
            base.max_hp + 20,
            "a saved count of two is still two old points (0158 N0)"
        );
        assert_eq!(
            back.stats().flask,
            base.flask + 2,
            "a core pick and the oil"
        );
    }

    #[test]
    fn an_empty_or_garbage_store_reads_as_a_fresh_start() {
        assert_eq!(
            Progress::from_record(Record::parse("")),
            Progress::default()
        );
        let p = Progress::from_record(Record::parse("\u{0}\u{1}garbage"));
        assert_eq!((p.embers, p.clears, p.run), (0, 0, None));
        assert!(p.owned.is_empty() && p.chests.is_empty());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn the_native_store_writes_reads_and_wipes_the_file() {
        let dir = std::env::temp_dir().join(format!("knell-save-test-{}", std::process::id()));
        std::env::set_var("KNELL_SAVE_DIR", &dir);
        let mut p = Progress::default();
        p.embers = 7;
        p.run = Some(RunAt {
            rest: 0,
            gate_open: false,
            ..RunAt::default()
        });
        assert!(save(0, &p));
        let got = load(0);
        assert_eq!((got.embers, got.run), (7, p.run));
        // 슬롯 1 = 옛 한 칸 파일 그대로(옮기는 코드 없음, 0066).
        assert!(dir.join("save0.txt").exists());
        // 슬롯마다 따로: 슬롯 3에 쓴 것이 슬롯 1·2에 새지 않는다.
        let other = Progress {
            clears: 2,
            ..Progress::default()
        };
        assert!(save(2, &other));
        assert!(dir.join("save2.txt").exists());
        assert_eq!(load(2).clears, 2);
        assert_eq!(load(0).clears, 0);
        assert_eq!(load(1).summary(), None, "untouched slot is empty");
        let sums = summaries();
        assert!(sums[0].is_some() && sums[1].is_none() && sums[2].is_some());
        // 마지막 슬롯 칸.
        assert_eq!(last_slot(), 0);
        assert!(set_last_slot(2));
        assert_eq!(last_slot(), 2);
        assert!(wipe(0));
        assert!(!load(0).has_any());
        assert_eq!(load(2).clears, 2, "wiping one slot keeps the others");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 0151: 끼운 스킬 둘은 기록에 남는다 — 가장 뒤의 스킬까지 왕복하고, 기록에 없으면(옛 기록) 처음의 둘, 같은 스킬 둘이면 바로잡는다.
    /// 처음의 둘 그대로면 키를 쓰지 않는다(빈 슬롯이 "기록 있음"이 되지 않게). 0158: 끼운 것은 배운 스킬이어야 한다.
    #[test]
    fn worn_skills_round_trip_and_old_records_get_the_first_two() {
        use crate::skills::{Board, Id, Pick, Worn, IDS, START};
        assert_eq!(Progress::default().skills.0, START);
        let last = *IDS.last().unwrap();
        let learned = Progress {
            level: crate::progression::Level {
                life: 5,
                stamina: 1,
                ..Default::default()
            },
            // 여덟을 다 배운 판(쓴 점수 6 — 레벨이 그만큼 있다).
            board: Board {
                rank: [1; crate::skills::N],
                pick: [Pick::A; crate::skills::N],
                ult: [1, 0, 0],
                ..Board::default()
            },
            ..Progress::default()
        };
        for worn in [[last, Id::Gather], [Id::Cut, Id::Whirl]] {
            let p = Progress {
                skills: Worn(worn),
                ..learned.clone()
            };
            let back = Progress::from_record(Record::parse(&p.to_record().to_text()));
            assert_eq!(back.skills.0, worn);
        }
        let old = Progress::from_record(Record::parse("clears=1\n"));
        assert_eq!(old.skills.0, START);
        let mut twice = learned.to_record();
        twice.set(key::SKILL1, "quake");
        twice.set(key::SKILL2, "quake");
        assert_eq!(
            Progress::from_record(twice).skills.0,
            [Id::Quake, Id::Whirl]
        );
        let text = Progress::default().to_record().to_text();
        assert!(!text.contains("skill.") && !text.contains("ult."), "{text}");
        assert_eq!(Progress::default().summary(), None);
    }

    /// 0158: 기술 판은 스킬마다의 단(`skill.rank.<키>`)·3단의 고르기(`skill.pick.<키>`)·큰 기술의 단(`ult.rank.upend`)으로 남는다.
    /// 처음 그대로인 칸은 쓰지 않고, 모르는 스킬의 키(다음 판의 것)는 그대로 지닌다.
    #[test]
    fn the_skill_board_round_trips_ranks_picks_and_the_ult_rank() {
        use crate::skills::{Board, Id, Pick, Worn};
        let p = Progress {
            level: crate::progression::Level {
                life: 5,
                stamina: 5,
                might: 5,
            },
            // 휘돌이 5·ㄴ, 가로지르기 3·ㄱ, 끌어모으기 2, 땅울림 1, 큰 기술 3 — 쓴 점수 4 + 2 + 2 + 1 + 2 = 11.
            board: Board {
                rank: [5, 3, 2, 1, 0, 0, 0, 0],
                pick: [
                    Pick::B,
                    Pick::A,
                    Pick::A,
                    Pick::A,
                    Pick::A,
                    Pick::A,
                    Pick::A,
                    Pick::A,
                ],
                ult: [3, 0, 0],
                ..Board::default()
            },
            skills: Worn([Id::Quake, Id::Gather]),
            ..Progress::default()
        };
        assert_eq!((p.board.spent(), p.skill_points()), (11, 15));
        let mut r = p.to_record();
        let text = r.to_text();
        for line in [
            "skill.rank.whirl=5\n",
            "skill.pick.whirl=b\n",
            "skill.rank.cut=3\n",
            "skill.pick.cut=a\n",
            "skill.rank.gather=2\n",
            "skill.rank.quake=1\n",
            "ult.rank.upend=3\n",
        ] {
            assert!(text.contains(line), "{line:?} in {text}");
        }
        assert!(!text.contains("skill.pick.gather"), "no pick below rank 3");
        r.set("skill.rank.vault", 2);
        r.set("ult.rank.onrush", 2);
        let back = Progress::from_record(Record::parse(&r.to_text()));
        assert_eq!(back.board, p.board);
        assert_eq!(back.skills, p.skills);
        let again = back.to_record();
        assert_eq!(again.num("skill.rank.vault"), Some(2), "unknown keys stay");
        assert_eq!(again.num("ult.rank.onrush"), Some(2));
        // 처음 판으로 돌아가면 키가 지워진다(옛 판으로 읽어도 처음 그대로).
        let mut reset = back.clone();
        reset.board = Board::default();
        let text = reset.to_record().to_text();
        assert!(!text.contains("skill.rank.whirl") && !text.contains("ult.rank.upend"));
    }

    /// 0158: 판의 키가 없는 기록(지금까지의 모든 기록)은 처음의 둘 1단·나머지 0단·큰 기술 1단으로 읽히고, 점수는 셈이라 저절로 있다
    /// (총 레벨 + 처음 쓰러뜨린 엘리트 + 이긴 적 있는 문지기). 안 배운 스킬이 끼워져 있으면 처음의 둘로.
    #[test]
    fn a_record_without_board_keys_reads_as_the_starters_and_already_has_points() {
        use crate::skills::{Board, Id, START};
        let old = Progress::from_record(Record::parse(
            "v=1\nmigrated.v04=1\nlv.life=2\nlv.might=1\nelites=sanctuary.elite.0,hamlet.elite.1\n\
             core.warden=3\ncore.bellkeeper=1\nskill.1=gather\nskill.2=quake\n",
        ));
        assert_eq!(old.board, Board::default());
        assert_eq!(old.board.rank, [1, 1, 0, 0, 0, 0, 0, 0]);
        assert_eq!(old.board.ult, [1, 0, 0]);
        assert_eq!(old.ult, crate::skills::Ult::Upend);
        assert_eq!(
            old.skills.0, START,
            "S0 let anything be worn — not learned now"
        );
        // 레벨 3 + 엘리트 2 + 핵 넷(문지기 셋 + 종지기 하나 — 0161: 핵마다 1) = 9, 쓴 것 없음.
        assert_eq!(old.skill_points(), 9);
        assert_eq!(old.board.left(old.skill_points()), 9);
        // 배운 스킬 하나 + 안 배운 스킬 하나여도 처음의 둘로.
        let half = Progress::from_record(Record::parse("skill.1=whirl\nskill.2=gather\n"));
        assert_eq!(half.skills.0, START);
        assert!(!half.board.learned(Id::Gather));
    }

    /// 0158: 손댄 기록 — 범위 밖의 단은 범위 안으로(처음의 둘·큰 기술은 1단 아래로 안 내려간다), 이상한 고르기는 ㄱ,
    /// 가진 점수보다 많이 쓴 판은 통째로 처음으로(터지지 않는다 — 점수는 남아 있다).
    #[test]
    fn a_hand_edited_board_is_clamped_or_ignored() {
        use crate::skills::{Board, Pick, MAX_RANK, START};
        let rich = "lv.life=5\nlv.stamina=5\nlv.might=5\n";
        let odd = Progress::from_record(Record::parse(&format!(
            "{rich}skill.rank.quake=99\nskill.rank.whirl=0\nskill.rank.gather=x\nult.rank.upend=0\n\
             skill.pick.quake=zzz\nskill.pick.cut=b\nskill.1=quake\nskill.2=cut\n"
        )));
        assert_eq!(odd.board.rank, [1, 1, 0, MAX_RANK, 0, 0, 0, 0]);
        assert_eq!(odd.board.ult, [1, 0, 0]);
        assert_eq!(
            odd.board.pick,
            [Pick::A; crate::skills::N],
            "{:?}",
            odd.board
        );
        assert_eq!(
            odd.skills.0[0],
            crate::skills::Id::Quake,
            "learned: stays worn"
        );
        // 큰 기술의 큰 수도 세 단에서 멈춘다.
        let ult = Progress::from_record(Record::parse(&format!("{rich}ult.rank.upend=250\n")));
        assert_eq!(ult.board.ult[0], crate::skills::ULT_MAX_RANK);
        // 점수 0인 기록에 끝 단들 → 처음 판, 끼운 것도 처음의 둘.
        let broke = Progress::from_record(Record::parse(
            "skill.rank.whirl=5\nskill.rank.cut=5\nskill.rank.gather=5\nskill.rank.quake=5\n\
             ult.rank.upend=3\nskill.pick.whirl=b\nskill.1=quake\nskill.2=gather\n",
        ));
        assert_eq!(broke.board, Board::default());
        assert_eq!(broke.skills.0, START);
        assert_eq!(broke.skill_points(), 0);
    }

    /// 0158 G2: 새 스킬 넷의 단·고르기(`skill.rank.brace` …)와 큰 기술 둘의 단(`ult.rank.flurry`·`ult.rank.line`), 끼운 큰 기술
    /// (`ult.worn`)이 기록에 남는다. 쟁반 뒤집기를 끼웠으면 키를 쓰지 않는다. 안 배운 큰 기술이 끼워진 기록은 쟁반 뒤집기로.
    #[test]
    fn the_g2_skills_and_the_worn_great_skill_round_trip() {
        use crate::skills::{Board, Id, Pick, Ult, Worn};
        let mut board = Board::default();
        // 버팀 3·ㄴ(3) + 되울림 5·ㄱ(5) + 던지기 1(1) + 잔불 흩기 2(2) + 몰아치기 2단(3) + 한 줄기 1단(2) = 16.
        for (id, rank, pick) in [
            (Id::Brace, 3, Pick::B),
            (Id::Echo, 5, Pick::A),
            (Id::Throw, 1, Pick::A),
            (Id::Embers, 2, Pick::A),
        ] {
            board.rank[id.index()] = rank;
            board.pick[id.index()] = pick;
        }
        board.ult = [1, 2, 1];
        let p = Progress {
            level: crate::progression::Level {
                life: 5,
                stamina: 5,
                might: 5,
            },
            elites: vec!["a".into()],
            board,
            skills: Worn([Id::Echo, Id::Brace]),
            ult: Ult::Flurry,
            ..Progress::default()
        };
        assert_eq!((p.board.spent(), p.skill_points()), (16, 16));
        let text = p.to_record().to_text();
        for line in [
            "skill.rank.brace=3\n",
            "skill.pick.brace=b\n",
            "skill.rank.echo=5\n",
            "skill.pick.echo=a\n",
            "skill.rank.throw=1\n",
            "skill.rank.embers=2\n",
            "ult.rank.flurry=2\n",
            "ult.rank.line=1\n",
            "ult.worn=flurry\n",
            "skill.1=echo\n",
            "skill.2=brace\n",
        ] {
            assert!(text.contains(line), "{line:?} in {text}");
        }
        assert!(!text.contains("ult.rank.upend"), "rank 1 is not written");
        let back = Progress::from_record(Record::parse(&text));
        assert_eq!(
            (back.board, back.skills, back.ult),
            (p.board, p.skills, p.ult)
        );
        // 쟁반 뒤집기를 끼우면 키가 지워진다.
        let mut upend = back.clone();
        upend.ult = Ult::Upend;
        assert!(!upend.to_record().to_text().contains("ult.worn"));
        // 안 배운 큰 기술이 끼워진 기록·모르는 값 → 쟁반 뒤집기.
        for worn in ["line", "zzz"] {
            let odd =
                Progress::from_record(Record::parse(&format!("lv.life=3\nult.worn={worn}\n")));
            assert_eq!(odd.ult, Ult::Upend, "{worn}");
        }
        // G0·G1이 쓴 기록(스킬 넷 + 쟁반 뒤집기의 단)은 그대로 읽힌다 — 새 칸은 0단, 끼운 큰 기술은 쟁반 뒤집기.
        let g1 = Progress::from_record(Record::parse(
            "lv.life=5\nlv.stamina=5\nlv.might=5\nskill.rank.whirl=5\nskill.pick.whirl=b\nskill.rank.cut=3\n\
             skill.pick.cut=a\nskill.rank.gather=2\nskill.rank.quake=1\nult.rank.upend=3\nskill.1=quake\nskill.2=gather\n",
        ));
        assert_eq!(g1.board.rank, [5, 3, 2, 1, 0, 0, 0, 0]);
        assert_eq!(g1.board.pick[..2], [Pick::B, Pick::A]);
        assert_eq!((g1.board.ult, g1.ult), ([3, 0, 0], Ult::Upend));
        assert_eq!(g1.skills.0, [Id::Quake, Id::Gather]);
        assert_eq!(g1.board.spent(), 11);
    }

    #[test]
    fn the_last_slot_cell_falls_back_to_slot_one_and_weapons_round_trip() {
        assert_eq!(parse_slot(""), 0, "no cell yet = the old single record");
        assert_eq!(parse_slot("2"), 2);
        assert_eq!(parse_slot("7"), 0, "out of range");
        assert_eq!(parse_slot(" 1\n"), 1);
        // 가장 큰 값까지 왕복(KB: 범위를 자르는 코드는 새 값에서 조용히 틀린다).
        for w in crate::weapons::IDS {
            let p = Progress {
                weapon: Some(w),
                ..Progress::default()
            };
            let back = Progress::from_record(Record::parse(&p.to_record().to_text()));
            assert_eq!(back.weapon, Some(w));
        }
        // 무기만 바꾼 슬롯은 "기록 있음"이 아니다(빈 슬롯 그대로).
        let only_weapon = Progress {
            weapon: Some(crate::weapons::Id::Staff),
            ..Progress::default()
        };
        assert_eq!(only_weapon.summary(), None);
        let some = Progress {
            clears: 1,
            embers: 9,
            owned: vec![crate::charms::Id::Ear],
            ..Progress::default()
        };
        // 요약은 장 · 시간만(0081 — 지역은 장에서 정해진다), 완주·잔불·장신구 수는 뺐다.
        assert_eq!(
            some.summary(),
            Some(Summary {
                chapter: 1,
                play: 0
            })
        );
        // 플레이 시간은 기록에 남고(키 `play`), 시간만 있어도 "기록 있음"이다.
        let played = Progress {
            play: 3725,
            ..Progress::default()
        };
        let back = Progress::from_record(Record::parse(&played.to_record().to_text()));
        assert_eq!(back.play, 3725);
        assert_eq!(back.summary().map(|m| m.play), Some(3725));
        assert_eq!(play_text(3725), "1:02");
        assert_eq!(play_text(59), "0:00");
    }
}
