//! 전투 스킬 표(0150 제안 → 0151 결정, S0). 스킬 = 방랑자가 되찾은 **몸의 기술**(무기와 상관없다 — 무기의 스페셜은 RB, `weapons.rs`).
//! 줄 = (다시 쓰기까지·시동·후딜·모양·피해). 스킬은 기력을 쓰지 않고 저마다의 대기로 돈다 — 스킬이 아닌 것으로 적을 쓰러뜨리면
//! 모든 대기가 당겨진다("기세", `skill.momentum s`). 큰 기술은 대기가 아니라 게이지(평타·패링으로만 찬다).
//!
//! 코드에 스킬별 숫자를 박지 않는다 — 모양·수치는 이 표, 전체 배율·기세·게이지는 랩(`skill.*`).
//! 새 스킬 = `Id` 한 줄 + `SKILLS` 한 줄 + (새 모양이면 `Form` 한 갈래 + `world/skill_tick.rs`의 한 갈래) + 터치 그림 + 글 + 시험.
//!
//! **기술 판(0158 G0·G1)**: 스킬은 기술 점수로 배우고 올린다 — 다섯 단(2단 피해 · 3단 둘 가운데 하나 `Twist` · 4단 제 치수 ·
//! 5단 맺음 `Cap`), 큰 기술은 세 단. 단의 숫자는 아래 `RANK_*`, 스킬마다의 고르기·맺음은 표의 `picks`·`cap`, 줄은 `row`
//! (쓴 점수 합 `ROW_NEEDS`로 열린다). 판의 셈(`Board` — 점수·사기·되돌리기)은 순수하고, 월드는 끼운 둘의 `Loadout`만 받는다
//! (`Skill::at(단, 고르기)` = 그 단의 실제 값 `Eff`). 새 단 효과 = `Twist`/`Cap` 한 갈래 + 표의 값 + `skill_tick.rs`의 한 줄 + 글 + 시험.

use crate::tuning::{HP_UNIT, ULT_SLAM_DAMAGE};

/// 스킬의 모양(판정과 움직임).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Form {
    /// 몸 둘레 원을 `hits`번 친다(`gap`초 사이). 맞은 적은 바깥으로 `push` m.
    Ring {
        radius: f32,
        hits: u8,
        gap: f32,
        push: f32,
    },
    /// 바라보는 쪽으로 `dist` m를 `time`초에 가로지르며 길(반 폭 `half` m) 위의 적을 한 번씩 친다. 적의 몸은 지나간다.
    Lane { dist: f32, half: f32, time: f32 },
    /// 앞 부채꼴(`range` m · `arc_deg`°)의 적을 방랑자 앞 `to` m 한 점으로 끌어 모으고 `stagger`초 휘청이게 한다.
    /// 보스·엘리트는 끌리지 않는다.
    Pull {
        range: f32,
        arc_deg: f32,
        to: f32,
        stagger: f32,
    },
    /// 앞 부채꼴(`range` m · `arc_deg`°)을 내리찍는다 — 맞은 적은 `stagger`초 휘청(보스는 휘청이지 않는다).
    Cone {
        range: f32,
        arc_deg: f32,
        stagger: f32,
    },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Id {
    /// 휘돌이: 제자리에서 두 바퀴.
    Whirl,
    /// 가로지르기: 떼를 뚫고 반대편으로.
    Cut,
    /// 끌어모으기: 다음 쓸기의 밥을 만든다.
    Gather,
    /// 땅울림: 몰린 쪽을 한 번에.
    Quake,
}

/// 표 순서(대기 칸 `Player::skill_cd`의 자리 — 뒤에만 붙인다).
pub const IDS: [Id; 4] = [Id::Whirl, Id::Cut, Id::Gather, Id::Quake];
pub const N: usize = IDS.len();

/// 처음 끼워져 있는 둘(0151 6번 — 첫 판부터 스킬 버튼이 비지 않게).
pub const START: [Id; 2] = [Id::Whirl, Id::Cut];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Skill {
    pub id: Id,
    /// 다시 쓰기까지(초) — 쓰는 순간부터 돈다. `skill.cooldown x`를 곱한다.
    pub cooldown: f32,
    /// 시동(초): 이 순간에 판정(또는 움직임)이 시작된다. 이 안에 맞으면 끊긴다(대기는 쓴 채).
    pub startup: f32,
    /// 판정이 끝난 뒤의 후딜(초) — 회피·다른 스킬로 끊는다.
    pub recover: f32,
    /// 한 타의 피해(0 = 피해 없음)와 자세 피해.
    pub damage: i32,
    pub poise: f32,
    pub form: Form,
    /// 빌려 쓰는 동작 이름(`wanderer.glb` — 스킬 전용 동작은 S1).
    pub anim: &'static str,
    /// 판의 줄(0158 라): 쓴 점수 합이 `ROW_NEEDS[row]`에 닿아야 살 수 있다.
    pub row: u8,
    /// 3단의 둘 가운데 하나(ㄱ·ㄴ)와 5단의 맺음(0158 마).
    pub picks: [Twist; 2],
    pub cap: Cap,
}

/// 단 수(0158 다): 0 = 안 배움, 1 = 배움 … 5 = 맺음. 단마다 기술 점수 1.
pub const MAX_RANK: u8 = 5;
/// 고르기가 열리는 단·제 치수가 느는 단·맺음의 단.
pub const PICK_RANK: u8 = 3;
pub const SIZE_RANK: u8 = 4;
pub const CAP_RANK: u8 = 5;
/// 단의 숫자: 2단부터 피해 ×1.15(피해 없는 스킬은 휘청 +0.3초), 4단 제 치수 ×1.2, 5단 피해 ×1.3(합)·대기 ×0.9.
pub const RANK2_DAMAGE: f32 = 1.15;
pub const RANK2_STAGGER: f32 = 0.3;
pub const RANK4_SIZE: f32 = 1.2;
pub const RANK5_DAMAGE: f32 = 1.3;
pub const RANK5_COOLDOWN: f32 = 0.9;

/// 피해에 배율(눈금 10의 정수 — 반올림: 10 → 12 → 13, 20 → 23 → 26).
pub fn scaled(base: i32, mult: f32) -> i32 {
    (base as f32 * mult).round() as i32
}

/// 3단의 고르기(둘 가운데 하나) — 화로에서 값 없이 바꾼다.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Pick {
    #[default]
    A,
    B,
}

impl Pick {
    pub fn index(self) -> usize {
        self as usize
    }

    pub fn other(self) -> Pick {
        match self {
            Pick::A => Pick::B,
            Pick::B => Pick::A,
        }
    }

    /// 기록 값(바꾸지 않는다).
    pub fn key(self) -> &'static str {
        match self {
            Pick::A => "a",
            Pick::B => "b",
        }
    }

    pub fn by_key(k: &str) -> Option<Pick> {
        [Pick::A, Pick::B].into_iter().find(|p| p.key() == k)
    }
}

/// 3단 고르기가 바꾸는 쓰임(0158 마). 숫자는 표에 — 틱은 갈래만 본다.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Twist {
    /// 휘돌이 ㄱ: 도는 동안 걷는다(걷는 빠르기 × `speed`).
    Walk { speed: f32 },
    /// 휘돌이 ㄴ: 바퀴를 `more`번 더.
    Spins { more: u8 },
    /// 가로지르기 ㄱ: 끝에서 온 길을 되돌아가며 한 번 더 벤다.
    Return,
    /// 가로지르기 ㄴ: 지나는 동안 맞지 않는다.
    Ghost,
    /// 끌어모으기 ㄱ: `delay`초 뒤 모인 자리 둘레 `radius` m가 터진다(피해 `damage`).
    Burst {
        delay: f32,
        radius: f32,
        damage: i32,
    },
    /// 끌어모으기 ㄴ: 엘리트도 끌린다(보스는 여전히 끌리지 않는다).
    Heavy,
    /// 땅울림 ㄱ: 시동 동안 겨눈 쪽으로 `dist` m까지 뛰어든다 — 그 길(반 폭 `half` m)의 가장 가까운 적 `short` m 앞에서 멈춘다.
    Leap { dist: f32, short: f32, half: f32 },
    /// 땅울림 ㄴ: 찍은 부채꼴이 `secs`초 남는다 — 그 안의 적은 `slow`배로 걷는다(보스는 아니다).
    Crack { secs: f32, slow: f32 },
}

/// 5단 맺음(0158 마).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cap {
    /// 휘돌이: 마지막 바퀴에 맞은 적이 `stagger`초 휘청인다(보스는 아니다).
    Lift { stagger: f32 },
    /// 가로지르기: 이 스킬로 쓰러뜨리면 대기의 `part`를 돌려받는다(한 번 쓸 때 한 번).
    Refund { part: f32 },
    /// 끌어모으기: 끌려온 적이 `secs`초 동안 받는 피해 × `mult`.
    Frail { secs: f32, mult: f32 },
    /// 땅울림: 자세가 깨져 있는 적에게 피해 × `mult`.
    Shatter { mult: f32 },
}

/// 그 단·그 고르기의 실제 값(월드가 쓴다): 표의 줄에 단의 숫자를 입힌 것 + 열린 고르기·맺음.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Eff {
    pub skill: Skill,
    /// 3단부터.
    pub twist: Option<Twist>,
    /// 5단.
    pub cap: Option<Cap>,
}

impl Eff {
    /// 판정(움직임)이 이어지는 시간(초) — 되돌아 베기는 길을 두 번 간다.
    pub fn active(&self) -> f32 {
        match (self.skill.form, self.twist) {
            (Form::Lane { time, .. }, Some(Twist::Return)) => time * 2.0,
            _ => self.skill.active(),
        }
    }

    pub fn total(&self) -> f32 {
        self.skill.startup + self.active() + self.skill.recover
    }
}

impl Skill {
    /// `rank`단(1~5로 맞춘다 — 끼운 스킬은 배운 스킬이다)·`pick` 고르기의 실제 값.
    pub fn at(&self, rank: u8, pick: Pick) -> Eff {
        let rank = rank.clamp(1, MAX_RANK);
        let twist = (rank >= PICK_RANK).then(|| self.picks[pick.index()]);
        let cap = (rank >= CAP_RANK).then_some(self.cap);
        let dmg = if rank >= CAP_RANK {
            RANK5_DAMAGE
        } else if rank >= 2 {
            RANK2_DAMAGE
        } else {
            1.0
        };
        let size = if rank >= SIZE_RANK { RANK4_SIZE } else { 1.0 };
        // 피해 없는 스킬의 2단은 휘청 시간.
        let hold = if rank >= 2 && self.damage == 0 {
            RANK2_STAGGER
        } else {
            0.0
        };
        let form = match self.form {
            Form::Ring {
                radius,
                hits,
                gap,
                push,
            } => Form::Ring {
                radius: radius * size,
                hits: hits
                    + match twist {
                        Some(Twist::Spins { more }) => more,
                        _ => 0,
                    },
                gap,
                push,
            },
            Form::Lane { dist, half, time } => Form::Lane {
                dist: dist * size,
                half,
                time,
            },
            Form::Pull {
                range,
                arc_deg,
                to,
                stagger,
            } => Form::Pull {
                range: range * size,
                arc_deg,
                to,
                stagger: stagger + hold,
            },
            Form::Cone {
                range,
                arc_deg,
                stagger,
            } => Form::Cone {
                range,
                arc_deg: arc_deg * size,
                stagger,
            },
        };
        Eff {
            skill: Skill {
                damage: scaled(self.damage, dmg),
                cooldown: self.cooldown
                    * if rank >= CAP_RANK {
                        RANK5_COOLDOWN
                    } else {
                        1.0
                    },
                form,
                ..*self
            },
            twist,
            cap,
        }
    }
}

impl Skill {
    /// 판정(움직임)이 이어지는 시간(초).
    pub fn active(&self) -> f32 {
        match self.form {
            Form::Ring { hits, gap, .. } => gap * hits.saturating_sub(1) as f32,
            Form::Lane { time, .. } => time,
            Form::Pull { .. } | Form::Cone { .. } => 0.0,
        }
    }

    /// 처음부터 끝까지(초) — 표 규칙 시험만 읽는다(틱은 그 단의 `Eff::total`).
    #[cfg(test)]
    pub fn total(&self) -> f32 {
        self.startup + self.active() + self.recover
    }
}

pub const SKILLS: [Skill; N] = [
    Skill {
        id: Id::Whirl,
        cooldown: 7.0,
        startup: 0.15,
        recover: 0.3,
        damage: 10,
        poise: 1.5,
        form: Form::Ring {
            radius: 3.0,
            hits: 2,
            gap: 0.22,
            push: 1.5,
        },
        anim: "staff4",
        row: 0,
        picks: [Twist::Walk { speed: 0.6 }, Twist::Spins { more: 1 }],
        cap: Cap::Lift { stagger: 0.4 },
    },
    Skill {
        id: Id::Cut,
        cooldown: 8.0,
        startup: 0.12,
        recover: 0.25,
        damage: 20,
        poise: 2.0,
        form: Form::Lane {
            dist: 6.0,
            half: 1.2,
            time: 0.2,
        },
        anim: "attack3",
        row: 0,
        picks: [Twist::Return, Twist::Ghost],
        cap: Cap::Refund { part: 0.5 },
    },
    Skill {
        id: Id::Gather,
        cooldown: 9.0,
        startup: 0.2,
        recover: 0.3,
        damage: 0,
        poise: 0.0,
        form: Form::Pull {
            range: 7.0,
            arc_deg: 120.0,
            to: 1.8,
            stagger: 0.6,
        },
        anim: "charge",
        row: 0,
        picks: [
            Twist::Burst {
                delay: 0.5,
                radius: 2.5,
                damage: HP_UNIT,
            },
            Twist::Heavy,
        ],
        cap: Cap::Frail {
            secs: 2.0,
            mult: 1.2,
        },
    },
    Skill {
        id: Id::Quake,
        cooldown: 10.0,
        startup: 0.3,
        recover: 0.4,
        damage: 20,
        poise: 4.0,
        form: Form::Cone {
            range: 5.0,
            arc_deg: 100.0,
            stagger: 0.8,
        },
        anim: "hammer_slam",
        row: 1,
        picks: [
            Twist::Leap {
                dist: 6.0,
                short: 1.5,
                half: 1.0,
            },
            Twist::Crack {
                secs: 2.0,
                slow: 0.5,
            },
        ],
        cap: Cap::Shatter { mult: 2.0 },
    },
];

impl Id {
    /// 표의 자리(대기 칸).
    pub fn index(self) -> usize {
        IDS.iter()
            .position(|i| *i == self)
            .expect("IDS has every Id")
    }

    /// 기록 키(끼운 칸 `skill.1`·`skill.2`의 값 — 바꾸지 않는다).
    pub fn key(self) -> &'static str {
        match self {
            Id::Whirl => "whirl",
            Id::Cut => "cut",
            Id::Gather => "gather",
            Id::Quake => "quake",
        }
    }

    pub fn by_key(k: &str) -> Option<Id> {
        IDS.into_iter().find(|i| i.key() == k)
    }

    pub fn def(self) -> &'static Skill {
        SKILLS
            .iter()
            .find(|s| s.id == self)
            .expect("SKILLS has every Id")
    }
}

/// 큰 기술 "쟁반 뒤집기"(0151): 땅을 크게 비틀어 둘레의 적을 바깥으로 쏟는다. 피해는 없다 — 벽·턱에 박힌 적만 다친다
/// (`slam.damage`의 `slam_x`배). 게이지를 통째로 쓴다.
pub struct Upend {
    /// 누른 순간 세계가 멈추는 초(히트스톱 — "시간", KB special-move-impact)와 그 뒤 땅이 뒤집히기까지의 시동(초).
    pub freeze: f32,
    pub startup: f32,
    pub recover: f32,
    pub radius: f32,
    /// 밀려나는 거리(m): 가까운 적이 가장 멀리(`near`), 테두리의 적이 가장 덜(`far`).
    pub near: f32,
    pub far: f32,
    /// 휘청(초) — 보스는 `boss_stagger`.
    pub stagger: f32,
    pub boss_stagger: f32,
    pub anim: &'static str,
}

pub const UPEND: Upend = Upend {
    freeze: 0.3,
    startup: 0.12,
    recover: 0.5,
    radius: 9.0,
    near: 6.0,
    far: 2.5,
    stagger: 2.0,
    boss_stagger: 0.6,
    anim: "hammer_slam",
};

/// 큰 기술의 단(0158 다): 처음부터 1단, 큰 기술 줄이 열리면 2단(휘청 +0.6초 — 보스의 짧은 휘청은 그대로)·3단(벽에 박힌 피해 ×1.5).
pub const ULT_MAX_RANK: u8 = 3;
pub const ULT_RANK2_STAGGER: f32 = 0.6;
pub const ULT_RANK3_SLAM: f32 = 1.5;
/// 큰 기술의 줄(`ROW_NEEDS`의 자리)과 기록 키(`ult.rank.<키>` — 바꾸지 않는다).
pub const ULT_ROW: u8 = 2;
pub const UPEND_KEY: &str = "upend";

impl Upend {
    /// `rank`단의 휘청(초)·벽에 박힌 적의 덤 피해.
    pub fn stagger_at(&self, rank: u8) -> f32 {
        self.stagger + if rank >= 2 { ULT_RANK2_STAGGER } else { 0.0 }
    }

    pub fn slam_at(&self, rank: u8) -> i32 {
        if rank >= ULT_MAX_RANK {
            scaled(ULT_SLAM_DAMAGE, ULT_RANK3_SLAM)
        } else {
            ULT_SLAM_DAMAGE
        }
    }
}

/// 끼운 칸 둘(기록·앱이 들고 다닌다) — 기본은 처음의 둘.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Worn(pub [Id; 2]);

impl Default for Worn {
    fn default() -> Self {
        Worn(START)
    }
}

/// 끼운 칸 둘을 바로잡는다(기록에서 읽을 때): 같은 스킬이 두 칸이면 둘째를 표에서 비어 있는 첫 스킬로.
pub fn sane(slots: [Id; 2]) -> [Id; 2] {
    if slots[0] != slots[1] {
        return slots;
    }
    let other = IDS
        .into_iter()
        .find(|i| *i != slots[0])
        .expect("more than one skill");
    [slots[0], other]
}

/// 월드에 넘기는 끼운 칸 하나(0158): 스킬·그 단·그 고르기. 월드는 기록을 모른다 — 이것만 받는다.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slot {
    pub id: Id,
    pub rank: u8,
    pub pick: Pick,
}

/// 끼운 둘 + 큰 기술의 단.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Loadout {
    pub slots: [Slot; 2],
    pub ult: u8,
}

/// 스킬만 주면 1단(시험·옛 부름).
impl From<[Id; 2]> for Loadout {
    fn from(ids: [Id; 2]) -> Self {
        Loadout {
            slots: ids.map(|id| Slot {
                id,
                rank: 1,
                pick: Pick::A,
            }),
            ult: 1,
        }
    }
}

/// 기술 점수(0158 나): 줍는 물건이 아니라 셈 — 총 레벨 + 처음 쓰러뜨린 엘리트 수 + 처음 이긴 보스 수.
pub fn points(level: u32, elites: u32, bosses: u32) -> u32 {
    level + elites + bosses
}

/// 줄이 열리는 때(쓴 점수 합): 첫 줄 · 둘째 줄 · 큰 기술 줄(0158 라).
pub const ROW_NEEDS: [u32; 3] = [0, 4, 8];
/// 되돌리기 값(잔불): `RESPEC_BASE + RESPEC_PER × 쓴 점수`.
pub const RESPEC_BASE: u32 = 30;
pub const RESPEC_PER: u32 = 5;

/// 판의 칸: 스킬 하나 또는 큰 기술.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Entry {
    Skill(Id),
    Ult,
}

/// 판의 줄 순서(화면도 이 순서).
pub const ENTRIES: [Entry; N + 1] = [
    Entry::Skill(Id::Whirl),
    Entry::Skill(Id::Cut),
    Entry::Skill(Id::Gather),
    Entry::Skill(Id::Quake),
    Entry::Ult,
];

impl Entry {
    pub fn max(self) -> u8 {
        match self {
            Entry::Skill(_) => MAX_RANK,
            Entry::Ult => ULT_MAX_RANK,
        }
    }

    pub fn row(self) -> usize {
        match self {
            Entry::Skill(id) => id.def().row as usize,
            Entry::Ult => ULT_ROW as usize,
        }
    }

    /// 값 없이 가진 단(처음의 둘과 큰 기술은 1단).
    pub fn free(self) -> u8 {
        match self {
            Entry::Skill(id) => START.contains(&id) as u8,
            Entry::Ult => 1,
        }
    }
}

/// 칸 하나의 다음 단을 지금 살 수 있나.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Offer {
    /// 살 수 있다(점수 1).
    Ready,
    /// 점수가 없다.
    Short,
    /// 줄이 아직 안 열렸다 — 점수를 이만큼 더 써야 열린다.
    Locked(u32),
    /// 끝 단.
    Full,
}

/// 기술 판(기록 `skill.rank.*`·`skill.pick.*`·`ult.rank.*`): 스킬마다의 단과 고르기, 큰 기술의 단.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Board {
    pub rank: [u8; N],
    pub pick: [Pick; N],
    pub ult: u8,
}

impl Default for Board {
    fn default() -> Self {
        Board {
            rank: IDS.map(|id| Entry::Skill(id).free()),
            pick: [Pick::A; N],
            ult: Entry::Ult.free(),
        }
    }
}

impl Board {
    pub fn rank(&self, e: Entry) -> u8 {
        match e {
            Entry::Skill(id) => self.rank[id.index()],
            Entry::Ult => self.ult,
        }
    }

    pub fn learned(&self, id: Id) -> bool {
        self.rank[id.index()] >= 1
    }

    /// 쓴 점수: 산 단만 센다(처음부터 가진 단은 값이 없다).
    pub fn spent(&self) -> u32 {
        ENTRIES
            .iter()
            .map(|&e| self.rank(e).saturating_sub(e.free()) as u32)
            .sum()
    }

    /// 남은 점수.
    pub fn left(&self, total: u32) -> u32 {
        total.saturating_sub(self.spent())
    }

    pub fn offer(&self, e: Entry, total: u32) -> Offer {
        let need = ROW_NEEDS[e.row()];
        if self.rank(e) >= e.max() {
            Offer::Full
        } else if self.spent() < need {
            Offer::Locked(need - self.spent())
        } else if self.left(total) == 0 {
            Offer::Short
        } else {
            Offer::Ready
        }
    }

    /// 다음 단을 산다(점수 1). 3단에 닿으면 고르기는 ㄱ.
    pub fn buy(&mut self, e: Entry, total: u32) -> bool {
        if self.offer(e, total) != Offer::Ready {
            return false;
        }
        match e {
            Entry::Skill(id) => {
                let i = id.index();
                self.rank[i] += 1;
                if self.rank[i] == PICK_RANK {
                    self.pick[i] = Pick::A;
                }
            }
            Entry::Ult => self.ult += 1,
        }
        true
    }

    /// 살 수 있는 칸이 있나(쉼터가 빛난다).
    pub fn can_buy(&self, total: u32) -> bool {
        ENTRIES
            .iter()
            .any(|&e| self.offer(e, total) == Offer::Ready)
    }

    /// 3단의 고르기를 뒤집는다(값 없이) — 3단이 안 됐으면 그대로.
    pub fn flip(&mut self, id: Id) -> bool {
        let i = id.index();
        if self.rank[i] < PICK_RANK {
            return false;
        }
        self.pick[i] = self.pick[i].other();
        true
    }

    /// 되돌리기 값(잔불).
    pub fn respec_cost(&self) -> u32 {
        RESPEC_BASE + RESPEC_PER * self.spent()
    }

    /// 전부 되돌린다: 처음의 둘만 1단·큰 기술 1단, 고르기는 지운다. 끼운 칸에 안 배운 스킬이 남으면 처음의 둘로.
    pub fn respec(&mut self, worn: &mut [Id; 2]) {
        *self = Board::default();
        *worn = self.wearable(*worn);
    }

    /// 잔불을 내고 되돌린다(화로): 쓴 점수가 없거나 잔불이 모자라면 아무 일도 없다(거짓).
    pub fn respec_paid(&mut self, embers: &mut u32, worn: &mut [Id; 2]) -> bool {
        let cost = self.respec_cost();
        if self.spent() == 0 || *embers < cost {
            return false;
        }
        *embers -= cost;
        self.respec(worn);
        true
    }

    /// 끼울 수 있는 둘: 둘 다 배운 스킬이면 그대로, 아니면 처음의 둘.
    pub fn wearable(&self, worn: [Id; 2]) -> [Id; 2] {
        if worn.iter().all(|&id| self.learned(id)) {
            worn
        } else {
            START
        }
    }

    /// 기록에서 읽은 판을 바로잡는다: 단은 범위 안으로(처음의 둘·큰 기술은 1단 아래로 내려가지 않는다), 3단이 안 된 고르기는 지우고,
    /// 가진 점수보다 많이 썼으면(손댄 기록) 판 전체를 처음으로 — 점수는 그대로 남으니 다시 찍으면 된다.
    pub fn sane(mut self, total: u32) -> Board {
        for (i, id) in IDS.into_iter().enumerate() {
            let e = Entry::Skill(id);
            self.rank[i] = self.rank[i].clamp(e.free(), e.max());
            if self.rank[i] < PICK_RANK {
                self.pick[i] = Pick::A;
            }
        }
        self.ult = self.ult.clamp(Entry::Ult.free(), ULT_MAX_RANK);
        if self.spent() > total {
            Board::default()
        } else {
            self
        }
    }

    /// 모두 끝 단(개발 창 — 기록에 쓰지 않는다).
    #[cfg(any(test, feature = "dev"))]
    pub fn maxed(pick: Pick) -> Board {
        Board {
            rank: [MAX_RANK; N],
            pick: [pick; N],
            ult: ULT_MAX_RANK,
        }
    }

    /// 끼운 둘을 월드에 넘길 꼴로.
    pub fn loadout(&self, worn: [Id; 2]) -> Loadout {
        Loadout {
            slots: worn.map(|id| Slot {
                id,
                rank: self.rank[id.index()],
                pick: self.pick[id.index()],
            }),
            ult: self.ult,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_skill_has_one_row_a_key_and_sane_numbers() {
        for id in IDS {
            assert_eq!(SKILLS.iter().filter(|s| s.id == id).count(), 1, "{id:?}");
            assert_eq!(Id::by_key(id.key()), Some(id));
            let s = id.def();
            assert!((5.0..=14.0).contains(&s.cooldown), "{id:?} cooldown");
            // 시동은 짧다(0150 3절: 0.15~0.3초) — 누른 것이 바로 나가야 "빵빵"이다.
            assert!((0.1..=0.3).contains(&s.startup), "{id:?} startup");
            assert!(s.total() < 1.0, "{id:?} is over within a second");
            assert!(s.damage >= 0 && s.damage <= 20, "{id:?}: wide, not strong");
        }
        let keys: Vec<_> = IDS.iter().map(|i| i.key()).collect();
        for (i, k) in keys.iter().enumerate() {
            assert!(!keys[i + 1..].contains(k), "{k} twice");
        }
        assert_ne!(START[0], START[1]);
        assert_eq!(Id::Whirl.index(), 0);
    }

    /// 끌어모으기는 다치게 하지 않는다 — 다음 쓸기의 밥을 만들 뿐(0150 4절).
    #[test]
    fn gather_sets_up_and_does_not_hurt() {
        let g = Id::Gather.def();
        assert_eq!(g.damage, 0);
        assert!(matches!(g.form, Form::Pull { stagger, .. } if stagger > 0.0));
    }

    /// 0158 나: 기술 점수는 셈 — 총 레벨 + 처음 쓰러뜨린 엘리트 + 처음 이긴 보스, 남은 점수 = 그것 − 쓴 점수.
    /// 처음부터 가진 단(처음의 둘·큰 기술 1단)은 쓴 점수가 아니다.
    #[test]
    fn points_are_levels_plus_first_elites_plus_first_bosses_minus_spent() {
        assert_eq!(points(0, 0, 0), 0);
        assert_eq!(points(15, 6, 2), 23, "0158: everything in the game today");
        let mut b = Board::default();
        assert_eq!(b.rank, [1, 1, 0, 0]);
        assert_eq!((b.ult, b.spent()), (1, 0), "the free ranks cost nothing");
        let total = points(3, 1, 1);
        assert_eq!(b.left(total), 5);
        assert!(b.buy(Entry::Skill(Id::Gather), total));
        assert!(b.buy(Entry::Skill(Id::Whirl), total));
        assert_eq!((b.spent(), b.left(total)), (2, 3));
        // 가진 것보다 많이 쓴 판(손댄 기록)의 남은 점수는 0에서 멈춘다.
        assert_eq!(Board::maxed(Pick::A).left(3), 0);
    }

    /// 0158 다: 단은 차례로 하나씩·단마다 1점, 다섯에서 멈춘다. 점수가 떨어지면 못 산다. 3단을 사면 고르기는 ㄱ.
    #[test]
    fn ranks_are_bought_in_order_one_point_each_and_stop_at_five() {
        let mut b = Board::default();
        let whirl = Entry::Skill(Id::Whirl);
        assert_eq!(b.offer(whirl, 0), Offer::Short);
        assert!(!b.buy(whirl, 0));
        for want in 2..=MAX_RANK {
            let spent = b.spent();
            assert_eq!(b.offer(whirl, 4), Offer::Ready);
            assert!(b.buy(whirl, 4));
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
        assert_eq!(b.left(4), 0);
        assert_eq!(b.offer(Entry::Skill(Id::Cut), 4), Offer::Short);
        assert!(!b.can_buy(4) && b.can_buy(5));
        // 안 배운 스킬의 첫 점 = 배움(1단).
        let mut b = Board::default();
        assert!(!b.learned(Id::Gather));
        assert!(b.buy(Entry::Skill(Id::Gather), 1));
        assert!(b.learned(Id::Gather));
        assert_eq!((b.rank(Entry::Skill(Id::Gather)), b.spent()), (1, 1));
    }

    /// 0158 라: 줄은 **쓴 점수의 합**으로 열린다 — 둘째 줄(땅울림) 4, 큰 기술 줄 8. 어디에 썼든 상관없다. 잠긴 칸은 점수가 있어도 못 산다.
    #[test]
    fn rows_open_by_points_spent_wherever_they_went() {
        assert_eq!(ROW_NEEDS, [0, 4, 8]);
        for id in [Id::Whirl, Id::Cut, Id::Gather] {
            assert_eq!(Entry::Skill(id).row(), 0, "{id:?}");
        }
        assert_eq!(Entry::Skill(Id::Quake).row(), 1);
        assert_eq!(Entry::Ult.row(), ULT_ROW as usize);
        let (quake, ult) = (Entry::Skill(Id::Quake), Entry::Ult);
        let total = 23;
        let mut b = Board::default();
        assert_eq!(b.offer(quake, total), Offer::Locked(4));
        assert_eq!(b.offer(ult, total), Offer::Locked(8));
        assert!(!b.buy(quake, total) && !b.buy(ult, total));
        // 넷을 한 스킬에 다 써도 열린다.
        for n in 1..=4 {
            assert!(b.buy(Entry::Skill(Id::Cut), total));
            assert_eq!(
                b.offer(quake, total),
                if n < 4 {
                    Offer::Locked(4 - n)
                } else {
                    Offer::Ready
                }
            );
        }
        assert_eq!(b.offer(ult, total), Offer::Locked(4));
        assert!(b.buy(quake, total) && b.buy(quake, total));
        assert!(b.buy(Entry::Skill(Id::Gather), total) && b.buy(Entry::Skill(Id::Whirl), total));
        assert_eq!(b.spent(), 8);
        assert_eq!(b.offer(ult, total), Offer::Ready);
        assert!(b.buy(ult, total) && b.buy(ult, total));
        assert_eq!((b.ult, b.offer(ult, total)), (ULT_MAX_RANK, Offer::Full));
        // 다 익히는 데 드는 점수: 스킬 넷 × 5 − 처음 둘 + 큰 기술 둘 = 20.
        assert_eq!(Board::maxed(Pick::A).spent(), 20);
    }

    /// 0158 나: 되돌리기는 쓴 점수를 모두 돌려준다 — 처음의 둘만 1단·큰 기술 1단, 고르기는 지워진다. 값은 `30 + 5 × 쓴 점수`.
    /// 끼운 칸에 안 배운 스킬이 남으면 처음의 둘로, 배운 것만 끼워져 있었으면 그대로.
    #[test]
    fn a_respec_returns_every_point_clears_picks_and_fixes_the_worn_slots() {
        assert_eq!(Board::default().respec_cost(), RESPEC_BASE);
        let total = 12;
        let mut b = Board::default();
        for e in [Entry::Skill(Id::Whirl); 4] {
            assert!(b.buy(e, total));
        }
        assert!(b.flip(Id::Whirl));
        assert!(b.buy(Entry::Skill(Id::Quake), total) && b.buy(Entry::Skill(Id::Gather), total));
        assert_eq!(b.spent(), 6);
        assert_eq!(b.respec_cost(), 30 + 5 * 6);
        let mut worn = [Id::Quake, Id::Whirl];
        b.respec(&mut worn);
        assert_eq!(b, Board::default());
        assert_eq!((b.spent(), b.left(total)), (0, total));
        assert_eq!(b.pick, [Pick::A; N]);
        assert_eq!(worn, START, "Quake is no longer learned");
        let mut worn = [Id::Cut, Id::Whirl];
        b.respec(&mut worn);
        assert_eq!(worn, [Id::Cut, Id::Whirl], "both starters: kept as worn");
    }

    /// 되돌리기는 잔불을 낸다 — 모자라면 아무 일도 없고(판·잔불·끼운 것 그대로), 쓴 점수가 없으면 낼 것도 없다.
    #[test]
    fn a_respec_charges_embers_and_does_nothing_when_short_or_unspent() {
        let total = 10;
        let mut b = Board::default();
        let mut worn = [Id::Gather, Id::Cut];
        let mut embers = 500;
        assert!(!b.respec_paid(&mut embers, &mut worn), "nothing spent");
        assert_eq!(embers, 500);
        assert!(b.buy(Entry::Skill(Id::Gather), total));
        assert!(b.buy(Entry::Skill(Id::Gather), total));
        assert!(b.buy(Entry::Skill(Id::Cut), total));
        let (cost, before) = (b.respec_cost(), b);
        assert_eq!(cost, 30 + 5 * 3);
        let mut poor = cost - 1;
        assert!(!b.respec_paid(&mut poor, &mut worn), "one ember short");
        assert_eq!((b, poor, worn), (before, cost - 1, [Id::Gather, Id::Cut]));
        assert!(b.respec_paid(&mut embers, &mut worn));
        assert_eq!(embers, 500 - cost);
        assert_eq!((b, b.left(total)), (Board::default(), total));
        assert_eq!(worn, START, "Gather went back to unlearned");
        // 다시 찍으면 값은 처음 값부터.
        assert_eq!(b.respec_cost(), RESPEC_BASE);
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

    /// 0158 다: 단의 숫자 — 피해는 2단 ×1.15·5단 ×1.3을 반올림(10 → 12 → 13, 20 → 23 → 26), 피해 없는 끌어모으기는
    /// 2단에 휘청 +0.3초, 4단에 제 치수 ×1.2(반지름·길이·거리·각도), 5단에 대기 ×0.9. 1단은 표 그대로.
    #[test]
    fn rank_numbers_for_each_skill() {
        assert_eq!(
            (scaled(10, RANK2_DAMAGE), scaled(10, RANK5_DAMAGE)),
            (12, 13)
        );
        assert_eq!(
            (scaled(20, RANK2_DAMAGE), scaled(20, RANK5_DAMAGE)),
            (23, 26)
        );
        let dmg = |id: Id| [1, 2, 3, 4, 5].map(|r| id.def().at(r, Pick::A).skill.damage);
        assert_eq!(dmg(Id::Whirl), [10, 12, 12, 12, 13]);
        assert_eq!(dmg(Id::Cut), [20, 23, 23, 23, 26]);
        assert_eq!(dmg(Id::Quake), [20, 23, 23, 23, 26]);
        assert_eq!(dmg(Id::Gather), [0; 5], "gather never hurts by itself");
        for id in IDS {
            let d = id.def();
            assert_eq!(
                d.at(1, Pick::A).skill,
                *d,
                "{id:?}: rank 1 is the table row"
            );
            assert_eq!(d.at(0, Pick::B).skill, *d, "an unlearned rank reads as 1");
            for r in 1..=4 {
                assert_eq!(d.at(r, Pick::A).skill.cooldown, d.cooldown, "{id:?} {r}");
            }
            let cd = d.at(5, Pick::A).skill.cooldown;
            assert!((cd - d.cooldown * 0.9).abs() < 1e-5, "{id:?} cooldown {cd}");
            // 고르기는 3단부터, 맺음은 5단에만.
            for r in 1..=5u8 {
                let e = d.at(r, Pick::B);
                assert_eq!(e.twist, (r >= 3).then_some(d.picks[1]), "{id:?} {r}");
                assert_eq!(e.cap, (r >= 5).then_some(d.cap), "{id:?} {r}");
            }
            assert_eq!(d.at(3, Pick::A).twist, Some(d.picks[0]));
        }
        // 2단: 끌어모으기의 휘청 0.6 → 0.9초.
        let stagger = |r| match Id::Gather.def().at(r, Pick::A).skill.form {
            Form::Pull { stagger, .. } => stagger,
            _ => unreachable!(),
        };
        assert_eq!(stagger(1), 0.6);
        assert!((stagger(2) - 0.9).abs() < 1e-6 && (stagger(5) - 0.9).abs() < 1e-6);
        // 4단: 제 치수만 ×1.2 — 다른 치수는 그대로.
        let form = |id: Id, r| id.def().at(r, Pick::A).skill.form;
        for r in [1, 3] {
            assert_eq!(form(Id::Cut, r), Id::Cut.def().form, "rank {r}");
        }
        assert!(
            matches!(form(Id::Whirl, 4), Form::Ring { radius, hits: 2, .. } if (radius - 3.6).abs() < 1e-5)
        );
        assert!(
            matches!(form(Id::Cut, 4), Form::Lane { dist, half, .. } if (dist - 7.2).abs() < 1e-5 && half == 1.2)
        );
        assert!(
            matches!(form(Id::Gather, 5), Form::Pull { range, arc_deg, .. } if (range - 8.4).abs() < 1e-5 && arc_deg == 120.0)
        );
        assert!(
            matches!(form(Id::Quake, 4), Form::Cone { range, arc_deg, .. } if range == 5.0 && (arc_deg - 120.0).abs() < 1e-4)
        );
    }

    /// 0158 마: 고르기와 맺음의 표 — 스킬마다 둘(서로 다르다)과 하나, 값은 결정문의 것. 길이가 달라지는 고르기를 골라도 1초 안에 끝난다.
    #[test]
    fn every_skill_has_two_different_picks_a_capstone_and_stays_under_a_second() {
        for id in IDS {
            let d = id.def();
            assert_ne!(d.picks[0], d.picks[1], "{id:?}");
            for p in [Pick::A, Pick::B] {
                let e = d.at(MAX_RANK, p);
                assert!(e.total() < 1.0, "{id:?} {p:?}: {}", e.total());
                assert!(e.active() >= d.active());
            }
        }
        assert_eq!(
            Id::Whirl.def().picks,
            [Twist::Walk { speed: 0.6 }, Twist::Spins { more: 1 }]
        );
        assert_eq!(Id::Cut.def().picks, [Twist::Return, Twist::Ghost]);
        assert!(matches!(
            Id::Gather.def().picks,
            [Twist::Burst { delay, damage: HP_UNIT, .. }, Twist::Heavy] if delay == 0.5
        ));
        assert!(matches!(
            Id::Quake.def().picks,
            [Twist::Leap { dist, short, .. }, Twist::Crack { secs, slow }]
                if dist == 6.0 && short == 1.5 && secs == 2.0 && slow == 0.5
        ));
        assert_eq!(Id::Whirl.def().cap, Cap::Lift { stagger: 0.4 });
        assert_eq!(Id::Cut.def().cap, Cap::Refund { part: 0.5 });
        assert_eq!(
            Id::Gather.def().cap,
            Cap::Frail {
                secs: 2.0,
                mult: 1.2
            }
        );
        assert_eq!(Id::Quake.def().cap, Cap::Shatter { mult: 2.0 });
        // 셋째 바퀴는 바퀴 수로, 되돌아 베기는 길 두 번으로 길어진다.
        assert!(matches!(
            Id::Whirl.def().at(3, Pick::B).skill.form,
            Form::Ring { hits: 3, .. }
        ));
        let cut = Id::Cut.def();
        assert_eq!(cut.at(3, Pick::A).active(), cut.active() * 2.0);
        assert_eq!(cut.at(3, Pick::B).active(), cut.active());
    }

    /// 0158 다: 큰 기술은 세 단 — 2단 휘청 +0.6초, 3단 벽에 박힌 피해 ×1.5(20 → 30). 처음부터 1단.
    #[test]
    fn the_great_skill_has_three_ranks() {
        assert_eq!((Entry::Ult.free(), Entry::Ult.max()), (1, 3));
        assert_eq!(UPEND.stagger_at(1), UPEND.stagger);
        assert!((UPEND.stagger_at(2) - UPEND.stagger - 0.6).abs() < 1e-6);
        assert_eq!(UPEND.stagger_at(3), UPEND.stagger_at(2));
        assert_eq!(
            [1, 2, 3].map(|r| UPEND.slam_at(r)),
            [ULT_SLAM_DAMAGE, ULT_SLAM_DAMAGE, 30]
        );
        assert_eq!(ULT_SLAM_DAMAGE, 20);
    }

    /// 손댄 기록을 바로잡는다: 범위 밖의 단은 안으로, 3단이 안 된 고르기는 ㄱ, 가진 점수보다 많이 쓴 판은 처음으로.
    #[test]
    fn a_board_from_a_record_is_made_sane_against_the_points_it_has() {
        let odd = Board {
            rank: [0, 9, 2, 3],
            pick: [Pick::B; N],
            ult: 0,
        };
        let b = odd.sane(23);
        assert_eq!(b.rank, [1, MAX_RANK, 2, 3]);
        assert_eq!(b.pick, [Pick::A, Pick::B, Pick::A, Pick::B]);
        assert_eq!(b.ult, 1);
        assert_eq!(odd.sane(b.spent()), b, "exactly enough points");
        assert_eq!(odd.sane(b.spent() - 1), Board::default(), "one short");
        assert_eq!(Board::default().sane(0), Board::default());
    }

    /// 월드에 넘기는 끼운 둘: 그 스킬의 단·고르기 + 큰 기술의 단. 끼울 수 있는 것은 배운 스킬뿐.
    #[test]
    fn the_loadout_carries_the_worn_ranks_and_only_learned_skills_are_wearable() {
        let mut b = Board::default();
        b.rank = [3, 1, 0, 5];
        b.pick[Id::Whirl.index()] = Pick::B;
        b.ult = 2;
        let l = b.loadout([Id::Quake, Id::Whirl]);
        assert_eq!(
            l.slots,
            [
                Slot {
                    id: Id::Quake,
                    rank: 5,
                    pick: Pick::A
                },
                Slot {
                    id: Id::Whirl,
                    rank: 3,
                    pick: Pick::B
                }
            ]
        );
        assert_eq!(l.ult, 2);
        assert_eq!(b.wearable([Id::Quake, Id::Whirl]), [Id::Quake, Id::Whirl]);
        assert_eq!(b.wearable([Id::Gather, Id::Whirl]), START);
        let plain: Loadout = [Id::Cut, Id::Gather].into();
        assert!(plain.slots.iter().all(|s| s.rank == 1) && plain.ult == 1);
    }

    #[test]
    fn two_slots_never_hold_the_same_skill() {
        assert_eq!(sane([Id::Cut, Id::Quake]), [Id::Cut, Id::Quake]);
        assert_eq!(sane([Id::Cut, Id::Cut]), [Id::Cut, Id::Whirl]);
        assert_eq!(sane([Id::Whirl, Id::Whirl]), [Id::Whirl, Id::Cut]);
    }
}
