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
//!
//! **G2(0158)**: 스킬 여덟(버팀 `Brace` · 되울림 `Echo` · 던지기 `Throw` · 잔불 흩기 `Field`의 네 모양이 더해졌다), 큰 기술 셋
//! (`Ult` — 쟁반 뒤집기는 처음부터, 몰아치기·한 줄기는 판에서 2점으로 배운다. **끼우는 큰 기술은 하나**). 전용 동작은 `Clip`
//! (이름 + 시동·이어짐·후딜 구간의 길이 — 구간의 경계가 판정의 순간이라 `Clip::at`이 단·고르기가 바꾼 길이를 구간에 맞춘다).
//!
//! **나무 하나(0161)**: 패시브 15칸(`progression::Cell`)도 이 판의 칸이다(`Entry::Cell` — 칸마다 1점, 잠금은 줄뿐). 스킬·큰 기술은
//! 갈래 셋(흐름·맞받기·자리 — `Skill::branch`·`Ult::branch`)에 서고, 판의 칸 순서 `ENTRIES`가 곧 화면의 차례다. 기술 점수의 샘은
//! 총 레벨 + 처음 쓰러뜨린 엘리트 + 핵 + 종소리 조각(`points`) — 화폐 "마디"는 없다.

use crate::progression::{Branch, Cell, Tree, DEPTH};
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
    /// 버팀(0158 G2): `hold`초 동안 버틴다 — 밀리지도 끊기지도 않고 받는 타마다 피해 −`soak`(0 아래로는 안 내려간다), 받아낸 타를
    /// `most`번까지 센다. 끝에 둘레 `radius` m를 터뜨린다: 표의 피해 + 받아낸 타마다 `more`, 맞은 적은 바깥으로 `push` m.
    /// 회피는 버팀을 터뜨림 없이 끊는다(대기는 쓴 채).
    Brace {
        hold: f32,
        radius: f32,
        soak: i32,
        most: u8,
        more: i32,
        push: f32,
    },
    /// 되울림(0158 G2): `secs`초 동안 성공한 패링마다 둘레 `radius` m의 적이 `stagger`초 휘청인다(보스는 휘청이지 않는다).
    /// 패링 창은 그대로다(0095 원칙 2).
    Echo {
        secs: f32,
        radius: f32,
        stagger: f32,
    },
    /// 던지기(0158 G2): 든 것이 `dist` m를 곧게 날아가(초속 `speed` m, 길의 반 폭 `half` m — 뚫는다) 방랑자에게 되돌아온다.
    /// 가는 길·오는 길에 적마다 한 번씩 표의 피해. `hand` = 던진 뒤 팔이 따라가는 초(몸의 "이어짐" — 그 뒤는 후딜).
    Throw {
        dist: f32,
        half: f32,
        speed: f32,
        hand: f32,
    },
    /// 잔불 흩기(0158 G2): 선 자리 둘레 `radius` m에 `secs`초 남는 불티 — 그 안의 적이 `every`초마다 표의 피해(첫 타는 `every`초 뒤).
    /// 잔불(재화)은 쓰지 않는다.
    Field { radius: f32, secs: f32, every: f32 },
}

/// 스킬 전용 동작(0158 G2, `wanderer.glb`의 `sk_*`·`ult_*`): 이름과 구간 셋(시동 · 이어짐 · 후딜)의 길이(초 — 바탕값으로 만든 것).
/// 구간의 경계가 판정의 순간이다. 모델에 아직 없으면 렌더가 표의 `anim`(빌려 쓰던 동작)으로 넘어간다.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Clip {
    pub name: &'static str,
    pub seg: [f32; 3],
}

impl Clip {
    /// 상태의 시각 `e`(초) → 동작의 시각. `startup`·`active`·`recover` = 지금 단·고르기의 실제 길이.
    /// 시동은 시동 구간에 맞춰 늘이고(경계 = 판정의 순간이 어긋나지 않게), 이어짐은 `period`가 있으면 그 주기로 구간을 되풀이
    /// (휘돌이의 바퀴·되돌아 베기의 둘째 길), 없으면 구간 전체에 늘인다. 후딜은 제 빠르기로 — 길면 끝 자세에 머문다.
    pub fn at(&self, e: f32, startup: f32, active: f32, period: Option<f32>) -> f32 {
        let [s, a, r] = self.seg;
        let e = e.max(0.0);
        if e < startup {
            return s * (e / startup.max(1e-4));
        }
        let q = e - startup;
        if q < active {
            let k = match period {
                Some(p) if p > 1e-4 => (q / p).fract(),
                _ => q / active.max(1e-4),
            };
            return s + a * k;
        }
        s + a + (q - active).min(r)
    }

    /// 동작 전체의 길이(초) — 시험이 읽는다.
    #[cfg(test)]
    pub fn total(&self) -> f32 {
        self.seg.iter().sum()
    }
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
    /// 버팀(G2): 떼 한가운데서 맞으며 버티다 터뜨린다.
    Brace,
    /// 되울림(G2): 패링이 둘레를 울린다.
    Echo,
    /// 던지기(G2): 든 것을 던져 길을 가르고 되받는다.
    Throw,
    /// 잔불 흩기(G2): 둘레에 남는 불티.
    Embers,
}

/// 표 순서(대기 칸 `Player::skill_cd`의 자리 — 뒤에만 붙인다).
pub const IDS: [Id; 8] = [
    Id::Whirl,
    Id::Cut,
    Id::Gather,
    Id::Quake,
    Id::Brace,
    Id::Echo,
    Id::Throw,
    Id::Embers,
];
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
    /// 빌려 쓰는 동작 이름(`wanderer.glb`) — 전용 동작 `clip`이 모델에 없을 때 쓴다.
    pub anim: &'static str,
    /// 전용 동작(0158 G2)과 그 구간의 길이.
    pub clip: Clip,
    /// 판의 줄(0158 라): 쓴 점수 합이 `ROW_NEEDS[row]`에 닿아야 살 수 있다.
    pub row: u8,
    /// 판의 갈래(0161 "갈래와 줄").
    pub branch: Branch,
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
/// 단의 숫자: 2단부터 피해 ×1.15(피해 없는 스킬은 휘청 +0.3초 — 되울림은 이어지는 시간 +1초), 4단 제 치수 ×1.2,
/// 5단 피해 ×1.3(합)·대기 ×0.9.
pub const RANK2_DAMAGE: f32 = 1.15;
pub const RANK2_STAGGER: f32 = 0.3;
pub const RANK2_SECS: f32 = 1.0;
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
    /// 휘돌이 ㄱ·버팀 ㄱ: 도는(버티는) 동안 걷는다(걷는 빠르기 × `speed`).
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
    /// 버팀 ㄴ "되쏘기": 버티는 동안 받아낸 탄·조각을 터뜨릴 때 쏜 쪽으로 되쏜다.
    Reflect,
    /// 되울림 ㄱ "울리는 틈": 울림이 닿은 적에게 자세 피해 `poise`.
    Rattle { poise: f32 },
    /// 되울림 ㄴ "당기는 박자": 울린 패링마다 모든 스킬의 대기 −`secs`초.
    Tempo { secs: f32 },
    /// 던지기 ㄱ "박힌 자리": 돌아오지 않고 끝에서 터진다(둘레 `radius` m · 피해 `damage`).
    Stick { radius: f32, damage: i32 },
    /// 던지기 ㄴ "끌고 오기": 돌아오는 길에 맞은 적을 방랑자 쪽으로 `dist` m 끈다(보스는 아니다).
    Drag { dist: f32 },
    /// 잔불 흩기 ㄱ "따르는 불티": 불티가 방랑자를 따라온다.
    Follow,
    /// 잔불 흩기 ㄴ "남는 불씨": 불티 속에서 쓰러진 적이 잔불(재화)을 `embers`개 더 남긴다.
    Kindle { embers: u32 },
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
    /// 버팀: 터뜨림에 맞은 적이 `stagger`초 휘청인다(보스는 아니다).
    Stun { stagger: f32 },
    /// 되울림: 울림이 닿은 적에게 피해 `damage`(보스도 다치되 휘청이지 않는다).
    Toll { damage: i32 },
    /// 던지기: 돌아오는 길의 피해 × `damage`·자세 피해 × `poise`.
    Homing { damage: f32, poise: f32 },
    /// 잔불 흩기: 이어지는 시간 +`secs`초, 불티의 타마다 자세 피해 `poise`.
    Smolder { secs: f32, poise: f32 },
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
        // 피해 없는 스킬(끌어모으기)의 2단은 휘청 시간.
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
            // 버팀: 받아낸 타의 덤도 피해의 배를 같이 받는다(10 → 12 → 13).
            Form::Brace {
                hold,
                radius,
                soak,
                most,
                more,
                push,
            } => Form::Brace {
                hold,
                radius: radius * size,
                soak,
                most,
                more: scaled(more, dmg),
                push,
            },
            // 되울림의 2단은 이어지는 시간(피해도 휘청도 아니다).
            Form::Echo {
                secs,
                radius,
                stagger,
            } => Form::Echo {
                secs: secs + if rank >= 2 { RANK2_SECS } else { 0.0 },
                radius: radius * size,
                stagger,
            },
            Form::Throw {
                dist,
                half,
                speed,
                hand,
            } => Form::Throw {
                dist: dist * size,
                half,
                speed,
                hand,
            },
            Form::Field {
                radius,
                secs,
                every,
            } => Form::Field {
                radius: radius * size,
                secs: secs
                    + match cap {
                        Some(Cap::Smolder { secs, .. }) => secs,
                        _ => 0.0,
                    },
                every,
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
            Form::Brace { hold, .. } => hold,
            Form::Throw { hand, .. } => hand,
            Form::Pull { .. } | Form::Cone { .. } | Form::Echo { .. } | Form::Field { .. } => 0.0,
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
        clip: Clip {
            name: "sk_whirl",
            seg: [0.15, 0.22, 0.30],
        },
        row: 0,
        branch: Branch::Beat,
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
        clip: Clip {
            name: "sk_cut",
            seg: [0.12, 0.20, 0.25],
        },
        row: 0,
        branch: Branch::Beat,
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
        clip: Clip {
            name: "sk_gather",
            seg: [0.20, 0.0, 0.30],
        },
        row: 0,
        branch: Branch::Footing,
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
        clip: Clip {
            name: "sk_quake",
            seg: [0.30, 0.0, 0.40],
        },
        row: 1,
        branch: Branch::Beat,
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
    // ---- G2(0158): 나머지 넷 ----
    Skill {
        id: Id::Brace,
        cooldown: 12.0,
        startup: 0.10,
        recover: 0.35,
        damage: HP_UNIT,
        poise: 2.0,
        form: Form::Brace {
            hold: 1.2,
            radius: 3.0,
            soak: HP_UNIT,
            most: 3,
            more: HP_UNIT,
            push: 1.0,
        },
        anim: "parry",
        clip: Clip {
            name: "sk_brace",
            seg: [0.10, 1.20, 0.35],
        },
        row: 0,
        branch: Branch::Riposte,
        picks: [Twist::Walk { speed: 0.45 }, Twist::Reflect],
        cap: Cap::Stun { stagger: 1.0 },
    },
    Skill {
        id: Id::Echo,
        cooldown: 12.0,
        startup: 0.25,
        recover: 0.20,
        damage: 0,
        poise: 0.0,
        form: Form::Echo {
            secs: 4.0,
            radius: 4.0,
            stagger: 0.8,
        },
        anim: "cast_charge",
        clip: Clip {
            name: "sk_echo",
            seg: [0.25, 0.0, 0.20],
        },
        row: 1,
        branch: Branch::Riposte,
        picks: [Twist::Rattle { poise: 2.5 }, Twist::Tempo { secs: 0.6 }],
        cap: Cap::Toll { damage: HP_UNIT },
    },
    Skill {
        id: Id::Throw,
        cooldown: 8.0,
        startup: 0.18,
        recover: 0.25,
        damage: HP_UNIT,
        poise: 1.5,
        form: Form::Throw {
            dist: 8.0,
            half: 0.9,
            speed: 22.0,
            hand: 0.10,
        },
        anim: "throw",
        clip: Clip {
            name: "sk_throw",
            seg: [0.18, 0.10, 0.25],
        },
        row: 1,
        branch: Branch::Footing,
        picks: [
            Twist::Stick {
                radius: 2.5,
                damage: 2 * HP_UNIT,
            },
            Twist::Drag { dist: 2.0 },
        ],
        cap: Cap::Homing {
            damage: 1.5,
            poise: 2.0,
        },
    },
    Skill {
        id: Id::Embers,
        cooldown: 10.0,
        startup: 0.25,
        recover: 0.30,
        damage: HP_UNIT,
        poise: 0.0,
        form: Form::Field {
            radius: 4.0,
            secs: 3.0,
            every: 1.0,
        },
        anim: "cast",
        clip: Clip {
            name: "sk_embers",
            seg: [0.25, 0.0, 0.30],
        },
        row: 1,
        branch: Branch::Footing,
        picks: [Twist::Follow, Twist::Kindle { embers: 1 }],
        cap: Cap::Smolder {
            secs: 1.0,
            poise: 1.0,
        },
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
            Id::Brace => "brace",
            Id::Echo => "echo",
            Id::Throw => "throw",
            Id::Embers => "embers",
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

/// 큰 기술(0151 → 0158 G2: 셋 — **끼우는 것은 하나**). 게이지를 통째로 쓴다.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Ult {
    /// 쟁반 뒤집기: 둘레의 적을 바깥으로 쏟는다(처음부터 1단).
    #[default]
    Upend,
    /// 몰아치기: 잠깐 동안 평타가 기력 없이·빠르게·모두 박자 타로.
    Flurry,
    /// 한 줄기: 세계를 멈추고 앞의 긴 한 줄을 가른다(보스전의 큰 기술).
    Line,
}

/// 큰 기술의 표 순서(판의 단 `Board::ult`의 자리 — 뒤에만 붙인다).
pub const ULTS: [Ult; 3] = [Ult::Upend, Ult::Flurry, Ult::Line];
pub const NU: usize = ULTS.len();

/// 큰 기술을 판에서 배우는 값(기술 점수 — 그 뒤 단마다 1).
pub const ULT_LEARN: u32 = 2;

impl Ult {
    pub fn index(self) -> usize {
        ULTS.iter()
            .position(|u| *u == self)
            .expect("ULTS has every Ult")
    }

    /// 기록 키(`ult.rank.<키>`·`ult.worn`의 값 — 바꾸지 않는다).
    pub fn key(self) -> &'static str {
        match self {
            Ult::Upend => UPEND_KEY,
            Ult::Flurry => "flurry",
            Ult::Line => "line",
        }
    }

    pub fn by_key(k: &str) -> Option<Ult> {
        ULTS.into_iter().find(|u| u.key() == k)
    }

    /// 판의 갈래(0161 "갈래와 줄"): 몰아치기 = 흐름, 한 줄기 = 맞받기, 쟁반 뒤집기 = 자리.
    pub fn branch(self) -> Branch {
        match self {
            Ult::Upend => Branch::Footing,
            Ult::Flurry => Branch::Beat,
            Ult::Line => Branch::Riposte,
        }
    }

    /// 빌려 쓰는 동작과 전용 동작(스킬의 `anim`·`clip`과 같은 뜻).
    pub fn anim(self) -> &'static str {
        match self {
            Ult::Upend => UPEND.anim,
            Ult::Flurry => FLURRY.anim,
            Ult::Line => LINE.anim,
        }
    }

    pub fn clip(self) -> Clip {
        match self {
            Ult::Upend => UPEND.clip,
            Ult::Flurry => FLURRY.clip,
            Ult::Line => LINE.clip,
        }
    }

    /// (누른 순간 세계가 멈추는 초, 그 뒤 터지기까지의 시동, 후딜) — 상태 틱과 동작 맞추기가 같이 읽는다.
    pub fn times(self) -> (f32, f32, f32) {
        match self {
            Ult::Upend => (UPEND.freeze, UPEND.startup, UPEND.recover),
            Ult::Flurry => (0.0, FLURRY.startup, FLURRY.recover),
            Ult::Line => (LINE.freeze, LINE.startup, LINE.recover),
        }
    }

    /// 큰 기술 동작의 시각: 세계 멈춤(`frozen` = 멈춤이 지난 몫 0..1, 상태 시계는 0에 머문다)은 시동 구간에, 그 뒤 터지기까지의
    /// 시동은 이어짐 구간에 늘여 맞춘다 — 이어짐 구간의 끝이 터지는 순간. 멈춤이 없는 큰 기술(몰아치기)은 시동이 곧 시동 구간.
    pub fn clip_at(self, e: f32, frozen: f32) -> f32 {
        let (freeze, startup, _) = self.times();
        let c = self.clip();
        if freeze <= 0.0 {
            return c.at(e, startup, 0.0, None);
        }
        if e <= 0.0 {
            return c.seg[0] * frozen.clamp(0.0, 1.0);
        }
        c.at(e, 0.0, startup, None)
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
    pub clip: Clip,
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
    clip: Clip {
        name: "ult_upend",
        seg: [0.30, 0.20, 0.50],
    },
};

/// 큰 기술 "몰아치기"(0158 G2): 시동 뒤 `secs`초 동안 평타가 기력을 쓰지 않고 `speed`배 빠르며 모든 타가 박자 타로 친다.
/// 2단 = `secs2`초, 3단 = 그동안 쓰러뜨릴 때마다 +`kill_more`초(더해지는 몫은 `kill_cap`초까지).
pub struct Flurry {
    pub startup: f32,
    pub recover: f32,
    pub secs: f32,
    pub secs2: f32,
    pub speed: f32,
    pub kill_more: f32,
    pub kill_cap: f32,
    pub anim: &'static str,
    pub clip: Clip,
}

pub const FLURRY: Flurry = Flurry {
    startup: 0.3,
    recover: 0.2,
    secs: 5.0,
    secs2: 6.5,
    speed: 1.4,
    kill_more: 0.5,
    kill_cap: 3.0,
    anim: "staff_charge",
    clip: Clip {
        name: "ult_flurry",
        seg: [0.30, 0.0, 0.20],
    },
};

impl Flurry {
    /// `rank`단의 이어지는 시간(초)과 쓰러뜨릴 때마다 더해지는 초(3단부터).
    pub fn secs_at(&self, rank: u8) -> f32 {
        if rank >= 2 {
            self.secs2
        } else {
            self.secs
        }
    }

    pub fn kill_more_at(&self, rank: u8) -> f32 {
        if rank >= ULT_MAX_RANK {
            self.kill_more
        } else {
            0.0
        }
    }
}

/// 큰 기술 "한 줄기"(0158 G2): 시동의 세계 멈춤 뒤 앞 `len` m 한 줄(반 폭 `half` m)을 가른다 — 피해 `damage`·자세 피해 `poise`.
/// 보스도 다치고 자세가 깎인다(자세가 깨져서 생기는 휘청 말고는 휘청이지 않는다). 2단 = 길이 `len2`, 3단 = 피해 `damage3`.
pub struct Line {
    pub freeze: f32,
    pub startup: f32,
    pub recover: f32,
    pub len: f32,
    pub len2: f32,
    pub half: f32,
    pub damage: i32,
    pub damage3: i32,
    pub poise: f32,
    pub anim: &'static str,
    pub clip: Clip,
}

pub const LINE: Line = Line {
    freeze: 0.3,
    startup: 0.15,
    recover: 0.45,
    len: 12.0,
    len2: 15.0,
    half: 1.2,
    damage: 4 * HP_UNIT,
    damage3: 6 * HP_UNIT,
    poise: 12.0,
    anim: "heavy",
    clip: Clip {
        name: "ult_line",
        seg: [0.30, 0.15, 0.45],
    },
};

impl Line {
    pub fn len_at(&self, rank: u8) -> f32 {
        if rank >= 2 {
            self.len2
        } else {
            self.len
        }
    }

    pub fn damage_at(&self, rank: u8) -> i32 {
        if rank >= ULT_MAX_RANK {
            self.damage3
        } else {
            self.damage
        }
    }
}

/// 큰 기술의 단(0158 다): 쟁반 뒤집기는 처음부터 1단, 큰 기술 줄이 열리면 2단(휘청 +0.6초 — 보스의 짧은 휘청은 그대로)·
/// 3단(벽에 박힌 피해 ×1.5). 몰아치기·한 줄기는 줄이 열리면 2점으로 배우고(1단) 2·3단은 1점씩.
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

/// 끼운 둘 + 끼운 큰 기술(`great`)과 그 단(`ult`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Loadout {
    pub slots: [Slot; 2],
    pub ult: u8,
    pub great: Ult,
}

/// 스킬만 주면 1단·쟁반 뒤집기(시험·옛 부름).
impl From<[Id; 2]> for Loadout {
    fn from(ids: [Id; 2]) -> Self {
        Loadout {
            slots: ids.map(|id| Slot {
                id,
                rank: 1,
                pick: Pick::A,
            }),
            ult: 1,
            great: Ult::Upend,
        }
    }
}

/// 기술 점수(0158 나 → 0161): 줍는 화폐가 아니라 셈 — 총 레벨 + 처음 쓰러뜨린 엘리트 수 + **핵**(보스를 이길 때마다, 보스마다
/// `progression::CORE_CAP`까지 — 부르는 쪽이 자른 수) + 주운 **종소리 조각**. 핵·조각은 옛 나무 화폐 "마디"였다 — 이제 점수의 샘이다.
/// 0158의 "보스 첫 쓰러뜨림 +1"은 핵의 수로 바뀌었다. 지금 내용으로 15 + 6 + 6 + 3 = 30.
pub fn points(level: u32, elites: u32, cores: u32, shards: u32) -> u32 {
    level + elites + cores + shards
}

/// 줄이 열리는 때(쓴 점수 합): 첫 줄 · 둘째 줄 · 큰 기술 줄(0158 라).
pub const ROW_NEEDS: [u32; 3] = [0, 4, 8];
/// 되돌리기 값(잔불): `RESPEC_BASE + RESPEC_PER × 쓴 점수`.
pub const RESPEC_BASE: u32 = 30;
pub const RESPEC_PER: u32 = 5;

/// 판의 칸: 스킬 하나 · 큰 기술 하나 · 패시브 칸 하나(0161).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Entry {
    Skill(Id),
    Ult(Ult),
    Cell(Cell),
}

/// 판의 칸 수: 스킬 여덟 + 큰 기술 셋 + 패시브 열다섯.
pub const NE: usize = N + NU + 3 * DEPTH as usize;

/// 판의 칸 순서(0161 "갈래와 줄" — 화면의 차례이기도 하다): 줄(처음부터 · 쓴 점수 4 · 쓴 점수 8)마다 갈래(흐름 · 맞받기 · 자리)
/// 순으로, 갈래 안에서는 스킬(큰 기술) 먼저·패시브 다음. 대기 칸의 자리(`IDS`)와는 따로다. 묻는 창 `Ask::Skill`이 이 자리를 든다.
pub const ENTRIES: [Entry; NE] = [
    // 1 — 처음부터
    Entry::Skill(Id::Whirl),
    Entry::Skill(Id::Cut),
    Entry::Cell(Cell::EvenBreath),
    Entry::Cell(Cell::FlowingBeat),
    Entry::Skill(Id::Brace),
    Entry::Cell(Cell::Catch),
    Entry::Cell(Cell::TurnItBack),
    Entry::Skill(Id::Gather),
    Entry::Cell(Cell::UnbrokenStride),
    Entry::Cell(Cell::Sidestep),
    // 2 — 쓴 점수 4
    Entry::Skill(Id::Quake),
    Entry::Cell(Cell::ClosingBlow),
    Entry::Cell(Cell::GapBeat),
    Entry::Skill(Id::Echo),
    Entry::Cell(Cell::GatheredRiposte),
    Entry::Cell(Cell::OpenBreach),
    Entry::Skill(Id::Throw),
    Entry::Skill(Id::Embers),
    Entry::Cell(Cell::TiltingGround),
    Entry::Cell(Cell::LandingBeat),
    // 3 — 쓴 점수 8
    Entry::Ult(Ult::Flurry),
    Entry::Cell(Cell::FinalMeasure),
    Entry::Ult(Ult::Line),
    Entry::Cell(Cell::OneWhoStands),
    Entry::Ult(Ult::Upend),
    Entry::Cell(Cell::PathOpeningStep),
];

impl Entry {
    /// 끝 단(패시브 칸은 열림 = 1).
    pub fn max(self) -> u8 {
        match self {
            Entry::Skill(_) => MAX_RANK,
            Entry::Ult(_) => ULT_MAX_RANK,
            Entry::Cell(_) => 1,
        }
    }

    pub fn row(self) -> usize {
        match self {
            Entry::Skill(id) => id.def().row as usize,
            Entry::Ult(_) => ULT_ROW as usize,
            Entry::Cell(c) => c.row(),
        }
    }

    pub fn branch(self) -> Branch {
        match self {
            Entry::Skill(id) => id.def().branch,
            Entry::Ult(u) => u.branch(),
            Entry::Cell(c) => c.branch(),
        }
    }

    /// 값 없이 가진 단(처음의 둘과 쟁반 뒤집기는 1단).
    pub fn free(self) -> u8 {
        match self {
            Entry::Skill(id) => START.contains(&id) as u8,
            Entry::Ult(u) => (u == Ult::Upend) as u8,
            Entry::Cell(_) => 0,
        }
    }

    /// `rank`단에서 다음 단으로 올리는 값(기술 점수): 단마다 1 — 큰 기술을 배우는 첫 단만 `ULT_LEARN`. 패시브 칸은 1.
    pub fn price(self, rank: u8) -> u32 {
        match self {
            Entry::Ult(_) if rank == 0 => ULT_LEARN,
            _ => 1,
        }
    }

    /// `rank`단까지 쓴 점수(값 없이 가진 단은 빼고).
    pub fn paid(self, rank: u8) -> u32 {
        (self.free()..rank).map(|r| self.price(r)).sum()
    }
}

/// 칸 하나의 다음 단을 지금 살 수 있나.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Offer {
    /// 살 수 있다(점수 `Entry::price`).
    Ready,
    /// 점수가 모자란다.
    Short,
    /// 줄이 아직 안 열렸다 — 점수를 이만큼 더 써야 열린다.
    Locked(u32),
    /// 끝 단.
    Full,
}

/// 기술 판(기록 `skill.rank.*`·`skill.pick.*`·`ult.rank.*`·`tree.cells.*`): 스킬마다의 단과 고르기, 큰 기술마다의 단
/// (`ULTS`의 자리), 연 패시브 칸(0161 — 같은 점수로 산다).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Board {
    pub rank: [u8; N],
    pub pick: [Pick; N],
    pub ult: [u8; NU],
    pub cells: Tree,
}

impl Default for Board {
    fn default() -> Self {
        Board {
            rank: IDS.map(|id| Entry::Skill(id).free()),
            pick: [Pick::A; N],
            ult: ULTS.map(|u| Entry::Ult(u).free()),
            cells: Tree::default(),
        }
    }
}

impl Board {
    /// 칸의 단(패시브 칸은 열렸으면 1).
    pub fn rank(&self, e: Entry) -> u8 {
        match e {
            Entry::Skill(id) => self.rank[id.index()],
            Entry::Ult(u) => self.ult[u.index()],
            Entry::Cell(c) => self.cells.has(c) as u8,
        }
    }

    pub fn learned(&self, id: Id) -> bool {
        self.rank[id.index()] >= 1
    }

    pub fn has_ult(&self, u: Ult) -> bool {
        self.ult[u.index()] >= 1
    }

    /// 쓴 점수: 산 단만 센다(처음부터 가진 단은 값이 없다, 큰 기술을 배운 첫 단은 2점) + 연 패시브 칸마다 1.
    pub fn spent(&self) -> u32 {
        ENTRIES.iter().map(|&e| e.paid(self.rank(e))).sum()
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
        } else if self.left(total) < e.price(self.rank(e)) {
            Offer::Short
        } else {
            Offer::Ready
        }
    }

    /// 다음 단을 산다(점수 `Entry::price`). 3단에 닿으면 고르기는 ㄱ.
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
            Entry::Ult(u) => self.ult[u.index()] += 1,
            Entry::Cell(c) => self.cells.open_cell(c),
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

    /// 전부 되돌린다: 처음의 둘만 1단·쟁반 뒤집기 1단, 고르기는 지우고 패시브 칸도 모두 닫는다(0161). 끼운 칸에 안 배운 스킬이
    /// 남으면 처음의 둘로, 끼운 큰 기술은 쟁반 뒤집기로.
    pub fn respec(&mut self, worn: &mut [Id; 2], great: &mut Ult) {
        *self = Board::default();
        *worn = self.wearable(*worn);
        *great = self.wearable_ult(*great);
    }

    /// 잔불을 내고 되돌린다(화로): 쓴 점수가 없거나 잔불이 모자라면 아무 일도 없다(거짓).
    pub fn respec_paid(&mut self, embers: &mut u32, worn: &mut [Id; 2], great: &mut Ult) -> bool {
        let cost = self.respec_cost();
        if self.spent() == 0 || *embers < cost {
            return false;
        }
        *embers -= cost;
        self.respec(worn, great);
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

    /// 끼울 수 있는 큰 기술: 배운 것이면 그대로, 아니면 쟁반 뒤집기(늘 가진 것).
    pub fn wearable_ult(&self, great: Ult) -> Ult {
        if self.has_ult(great) {
            great
        } else {
            Ult::Upend
        }
    }

    /// 기록에서 읽은 판을 바로잡는다: 단은 범위 안으로(처음의 둘·쟁반 뒤집기는 1단 아래로 내려가지 않는다), 3단이 안 된 고르기는
    /// 지우고, 가진 점수보다 많이 썼으면 처음으로 — 점수는 그대로 남으니 다시 찍으면 된다. 처음으로 돌리는 차례(0161): 먼저 스킬·
    /// 큰 기술만(연 패시브 칸은 남긴다 — 옛 기록의 칸은 핵·조각으로 열었고 그 둘은 지금도 점수라 칸만으로는 넘지 않는다. 0158의
    /// "보스 첫 쓰러뜨림 +1"이 핵의 수로 바뀌어 개발판 기록이 한두 점 넘칠 수 있다), 그래도 넘으면(손댄 기록) 칸까지.
    pub fn sane(mut self, total: u32) -> Board {
        self.cells = self.cells.sane();
        for (i, id) in IDS.into_iter().enumerate() {
            let e = Entry::Skill(id);
            self.rank[i] = self.rank[i].clamp(e.free(), e.max());
            if self.rank[i] < PICK_RANK {
                self.pick[i] = Pick::A;
            }
        }
        for (i, u) in ULTS.into_iter().enumerate() {
            self.ult[i] = self.ult[i].clamp(Entry::Ult(u).free(), ULT_MAX_RANK);
        }
        if self.spent() <= total {
            return self;
        }
        let cells = Board {
            cells: self.cells,
            ..Board::default()
        };
        if cells.spent() <= total {
            cells
        } else {
            Board::default()
        }
    }

    /// 모두 끝 단·패시브 칸 모두 열림(개발 창 — 기록에 쓰지 않는다).
    #[cfg(any(test, feature = "dev"))]
    pub fn maxed(pick: Pick) -> Board {
        Board {
            rank: [MAX_RANK; N],
            pick: [pick; N],
            ult: [ULT_MAX_RANK; NU],
            cells: Tree::full(),
        }
    }

    /// 화면을 열 때 설 칸(0161, `ENTRIES`의 자리): 올릴 수 있는 화면(`edit`)이면 지금 살 수 있는 첫 칸, 없으면(또는 읽기 전용이면)
    /// 배운 첫 스킬 — 처음의 둘이 늘 있다.
    pub fn first_focus(&self, total: u32, edit: bool) -> usize {
        let ready = ENTRIES
            .iter()
            .position(|&e| edit && self.offer(e, total) == Offer::Ready);
        let learned = ENTRIES
            .iter()
            .position(|&e| matches!(e, Entry::Skill(id) if self.learned(id)));
        ready.or(learned).unwrap_or(0)
    }

    /// 끼운 둘과 끼운 큰 기술을 월드에 넘길 꼴로.
    pub fn loadout(&self, worn: [Id; 2], great: Ult) -> Loadout {
        let great = self.wearable_ult(great);
        Loadout {
            slots: worn.map(|id| Slot {
                id,
                rank: self.rank[id.index()],
                pick: self.pick[id.index()],
            }),
            ult: self.ult[great.index()],
            great,
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
            // 버팀만 길다(버티는 것이 그 스킬이다) — 나머지는 1초 안에 끝난다.
            if matches!(s.form, Form::Brace { .. }) {
                assert!(s.total() < 2.0, "{id:?}: a hold, but a short one");
            } else {
                assert!(s.total() < 1.0, "{id:?} is over within a second");
            }
            assert!(s.damage >= 0 && s.damage <= 20, "{id:?}: wide, not strong");
            // 전용 동작(G2): 구간의 길이 = 표의 바탕값(경계가 판정의 순간).
            assert!(s.clip.name.starts_with("sk_"), "{id:?}");
            assert!((s.clip.seg[0] - s.startup).abs() < 1e-6, "{id:?} startup");
            assert!((s.clip.seg[1] - s.active()).abs() < 1e-6, "{id:?} active");
            assert!((s.clip.seg[2] - s.recover).abs() < 1e-6, "{id:?} recover");
        }
        let clips: Vec<_> = IDS.iter().map(|i| i.def().clip.name).collect();
        for (i, c) in clips.iter().enumerate() {
            assert!(!clips[i + 1..].contains(c), "{c} twice");
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

    /// 0161: 기술 점수는 셈 — 총 레벨 + 처음 쓰러뜨린 엘리트 + 핵(보스마다 셋까지) + 종소리 조각, 남은 점수 = 그것 − 쓴 점수.
    /// 화폐 "마디"(핵 + 조각)와 0158의 "보스 첫 쓰러뜨림 +1"을 이 하나가 대신한다. 처음부터 가진 단(처음의 둘·큰 기술 1단)은 쓴 점수가 아니다.
    #[test]
    fn points_are_levels_plus_first_elites_plus_cores_plus_shards_minus_spent() {
        assert_eq!(points(0, 0, 0, 0), 0);
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
        assert_eq!(b.rank, [1, 1, 0, 0, 0, 0, 0, 0]);
        assert_eq!(
            (b.ult, b.spent()),
            ([1, 0, 0], 0),
            "the free ranks cost nothing"
        );
        let total = points(3, 1, 1, 0);
        assert_eq!(b.left(total), 5);
        assert!(b.buy(Entry::Skill(Id::Gather), total));
        assert!(b.buy(Entry::Skill(Id::Whirl), total));
        assert!(b.buy(Entry::Cell(Cell::Catch), total));
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
        let cores = stages.len() as u32 * CORE_CAP;
        let shards: u32 = stages
            .iter()
            .map(|d| {
                let (run, _) = crate::stage::StageRun::start(d);
                run.finds
                    .iter()
                    .filter(|f| f.1 == crate::stage::Find::Shard)
                    .count() as u32
            })
            .sum();
        assert_eq!((levels, elites, cores, shards), (15, 6, 6, 3));
        assert_eq!(points(levels, elites, cores, shards), 30);
        // 다 익히는 값 63의 반이 조금 안 된다(0160 2절).
        assert!(2 * 30 < Board::maxed(Pick::A).spent());
    }

    /// 0161: 패시브 칸은 판의 칸이다 — 칸마다 1점, 잠금은 줄(쓴 점수 합)뿐. 갈래의 앞 칸을 열지 않아도 같은 줄의 뒤 칸이 열리고,
    /// 둘째 줄 칸은 쓴 점수 4, 셋째 줄 칸은 8부터 — 어디에 썼든 상관없다. 연 칸은 다시 못 산다.
    #[test]
    fn passive_cells_cost_one_point_and_are_gated_by_rows_not_by_order() {
        let total = 30;
        let mut b = Board::default();
        let (second, third, fifth) = (
            Entry::Cell(Cell::FlowingBeat),
            Entry::Cell(Cell::ClosingBlow),
            Entry::Cell(Cell::FinalMeasure),
        );
        assert_eq!((second.price(0), second.max(), second.free()), (1, 1, 0));
        assert_eq!(b.offer(second, 0), Offer::Short);
        // 갈래의 첫 칸(고른 호흡)을 열지 않고 둘째 칸부터.
        assert_eq!(b.offer(second, total), Offer::Ready);
        assert!(b.buy(second, total));
        assert!(b.cells.has(Cell::FlowingBeat) && !b.cells.has(Cell::EvenBreath));
        assert_eq!((b.spent(), b.rank(second)), (1, 1));
        assert_eq!(b.offer(second, total), Offer::Full);
        assert!(!b.buy(second, total), "an open cell is not bought twice");
        // 둘째 줄: 쓴 점수 4부터(지금 1) — 스킬에 셋을 더 쓰면 열린다.
        assert_eq!(b.offer(third, total), Offer::Locked(3));
        assert_eq!(b.offer(fifth, total), Offer::Locked(7));
        for _ in 0..3 {
            assert!(b.buy(Entry::Skill(Id::Whirl), total));
        }
        assert_eq!(b.offer(third, total), Offer::Ready);
        // 넷째 칸을 셋째 칸보다 먼저.
        assert!(b.buy(Entry::Cell(Cell::GapBeat), total));
        assert!(!b.cells.has(Cell::ClosingBlow));
        // 패시브에 쓴 점수도 줄을 연다: 칸 셋을 더 열면 쓴 점수 8 — 셋째 줄.
        for c in [Cell::Catch, Cell::TurnItBack, Cell::Sidestep] {
            assert_eq!(b.offer(fifth, total), Offer::Locked(8 - b.spent()));
            assert!(b.buy(Entry::Cell(c), total));
        }
        assert_eq!(b.spent(), 8);
        assert!(b.buy(fifth, total));
        assert!(b.cells.has(Cell::FinalMeasure));
        // 점수가 떨어지면 열린 줄의 칸도 못 연다.
        assert_eq!(
            b.offer(Entry::Cell(Cell::EvenBreath), b.spent()),
            Offer::Short
        );
        assert!(b.can_buy(total) && !b.can_buy(b.spent()));
    }

    /// 0161 "갈래와 줄"의 표 그대로: 줄마다 갈래마다 선 칸(스킬·큰 기술·패시브). 판의 칸 순서는 줄 → 갈래 → 스킬 먼저.
    #[test]
    fn the_board_stands_as_the_0161_table_says() {
        use Branch::{Beat, Footing, Riposte};
        use Entry::{Cell as C, Skill as S, Ult as U};
        let table: [(usize, Branch, &[Entry]); 9] = [
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
            (2, Beat, &[U(Ult::Flurry), C(Cell::FinalMeasure)]),
            (2, Riposte, &[U(Ult::Line), C(Cell::OneWhoStands)]),
            (2, Footing, &[U(Ult::Upend), C(Cell::PathOpeningStep)]),
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
        assert_eq!(NE, 26);
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

    /// 화면을 열 때 설 칸(0161): 화로에서는 지금 살 수 있는 첫 칸, 살 것이 없거나 읽기 전용(멈춤)이면 배운 첫 스킬.
    #[test]
    fn the_screen_opens_on_the_first_buyable_cell_or_the_first_learned_skill() {
        let b = Board::default();
        let at = |e: Entry| ENTRIES.iter().position(|x| *x == e).unwrap();
        assert_eq!(b.first_focus(5, true), at(Entry::Skill(Id::Whirl)));
        assert_eq!(
            b.first_focus(0, true),
            at(Entry::Skill(Id::Whirl)),
            "nothing to buy"
        );
        assert_eq!(
            b.first_focus(5, false),
            at(Entry::Skill(Id::Whirl)),
            "view only"
        );
        // 첫 칸(휘돌이)을 다 올렸으면 그다음 살 수 있는 칸.
        let mut full = Board::default();
        full.rank[Id::Whirl.index()] = MAX_RANK;
        assert_eq!(full.first_focus(9, true), at(Entry::Skill(Id::Cut)));
        assert_eq!(
            full.first_focus(4, true),
            at(Entry::Skill(Id::Whirl)),
            "all spent"
        );
        // 다 익힌 판: 살 것이 없다 — 배운 첫 스킬.
        assert_eq!(Board::maxed(Pick::A).first_focus(99, true), 0);
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
        for id in [Id::Whirl, Id::Cut, Id::Gather, Id::Brace] {
            assert_eq!(Entry::Skill(id).row(), 0, "{id:?}");
        }
        for id in [Id::Quake, Id::Echo, Id::Throw, Id::Embers] {
            assert_eq!(Entry::Skill(id).row(), 1, "{id:?}");
        }
        for u in ULTS {
            assert_eq!(Entry::Ult(u).row(), ULT_ROW as usize);
        }
        let (quake, ult) = (Entry::Skill(Id::Quake), Entry::Ult(Ult::Upend));
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
        assert_eq!((b.ult[0], b.offer(ult, total)), (ULT_MAX_RANK, Offer::Full));
        // 다 익히는 데 드는 점수(0158 라 → 0161): 스킬 여덟 × 5 − 처음 둘 = 38, 큰 기술 셋 × 4 − 쟁반 뒤집기의 배움 2 = 10,
        // 패시브 열다섯 × 1 = 15 — 63.
        let all = Board::maxed(Pick::A);
        let paid = |pred: fn(&Entry) -> bool| -> u32 {
            ENTRIES
                .iter()
                .filter(|e| pred(e))
                .map(|&e| e.paid(all.rank(e)))
                .sum()
        };
        assert_eq!(paid(|e| matches!(e, Entry::Skill(_))), 38);
        assert_eq!(paid(|e| matches!(e, Entry::Ult(_))), 10);
        assert_eq!(paid(|e| matches!(e, Entry::Cell(_))), 15);
        assert_eq!(all.spent(), 63);
        // 판의 칸 순서: 줄 번호가 내려가지 않는다.
        let rows: Vec<usize> = ENTRIES.iter().map(|e| e.row()).collect();
        assert!(rows.windows(2).all(|w| w[0] <= w[1]), "{rows:?}");
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
        // 0161: 패시브 칸도 쓴 점수다 — 값에 들어가고, 되돌리면 같이 닫힌다.
        assert!(b.buy(Entry::Cell(Cell::Catch), total) && b.buy(Entry::Cell(Cell::GapBeat), total));
        assert_eq!((b.spent(), b.respec_cost()), (8, 30 + 5 * 8));
        let mut worn = [Id::Quake, Id::Whirl];
        let mut great = Ult::Upend;
        b.respec(&mut worn, &mut great);
        assert_eq!(b, Board::default());
        assert_eq!(b.cells.count(), 0, "a respec closes every passive cell");
        assert!(!b.cells.has(Cell::Catch) && !b.cells.has(Cell::GapBeat));
        assert_eq!((b.spent(), b.left(total)), (0, total));
        assert_eq!(b.pick, [Pick::A; N]);
        assert_eq!(worn, START, "Quake is no longer learned");
        let mut worn = [Id::Cut, Id::Whirl];
        b.respec(&mut worn, &mut great);
        assert_eq!(worn, [Id::Cut, Id::Whirl], "both starters: kept as worn");
        assert_eq!(great, Ult::Upend);
    }

    /// 되돌리기는 잔불을 낸다 — 모자라면 아무 일도 없고(판·잔불·끼운 것 그대로), 쓴 점수가 없으면 낼 것도 없다.
    #[test]
    fn a_respec_charges_embers_and_does_nothing_when_short_or_unspent() {
        let total = 10;
        let mut b = Board::default();
        let mut worn = [Id::Gather, Id::Cut];
        let mut embers = 500;
        let mut great = Ult::Upend;
        assert!(
            !b.respec_paid(&mut embers, &mut worn, &mut great),
            "nothing spent"
        );
        assert_eq!(embers, 500);
        assert!(b.buy(Entry::Skill(Id::Gather), total));
        assert!(b.buy(Entry::Skill(Id::Gather), total));
        assert!(b.buy(Entry::Skill(Id::Cut), total));
        let (cost, before) = (b.respec_cost(), b);
        assert_eq!(cost, 30 + 5 * 3);
        let mut poor = cost - 1;
        assert!(
            !b.respec_paid(&mut poor, &mut worn, &mut great),
            "one ember short"
        );
        assert_eq!((b, poor, worn), (before, cost - 1, [Id::Gather, Id::Cut]));
        assert!(b.respec_paid(&mut embers, &mut worn, &mut great));
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
        // G2: 버팀·던지기·잔불 흩기는 한 눈금에서, 되울림은 피해가 없다(맺음의 피해는 따로).
        for id in [Id::Brace, Id::Throw, Id::Embers] {
            assert_eq!(dmg(id), [10, 12, 12, 12, 13], "{id:?}");
        }
        assert_eq!(dmg(Id::Echo), [0; 5]);
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
        // G2의 4단: 버팀·되울림·잔불 흩기는 반지름, 던지기는 거리.
        assert!(
            matches!(form(Id::Brace, 4), Form::Brace { radius, hold, .. } if (radius - 3.6).abs() < 1e-5 && hold == 1.2)
        );
        assert!(
            matches!(form(Id::Echo, 4), Form::Echo { radius, .. } if (radius - 4.8).abs() < 1e-5)
        );
        assert!(
            matches!(form(Id::Throw, 4), Form::Throw { dist, half, .. } if (dist - 9.6).abs() < 1e-5 && half == 0.9)
        );
        assert!(
            matches!(form(Id::Embers, 4), Form::Field { radius, secs, .. } if (radius - 4.8).abs() < 1e-5 && secs == 3.0)
        );
        // 되울림의 2단은 이어지는 시간 +1초(휘청은 그대로), 잔불 흩기의 맺음은 +1초, 버팀의 받아낸 타 덤은 피해와 같이 오른다.
        let echo = |r| match form(Id::Echo, r) {
            Form::Echo { secs, stagger, .. } => (secs, stagger),
            _ => unreachable!(),
        };
        assert_eq!(
            (echo(1), echo(2), echo(5)),
            ((4.0, 0.8), (5.0, 0.8), (5.0, 0.8))
        );
        let field = |r| match form(Id::Embers, r) {
            Form::Field { secs, every, .. } => (secs, every),
            _ => unreachable!(),
        };
        assert_eq!((field(4), field(5)), ((3.0, 1.0), (4.0, 1.0)));
        let more = |r| match form(Id::Brace, r) {
            Form::Brace {
                more, soak, most, ..
            } => (more, soak, most),
            _ => unreachable!(),
        };
        assert_eq!(
            (more(1), more(2), more(5)),
            ((10, 10, 3), (12, 10, 3), (13, 10, 3))
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
                let long = if id == Id::Brace { 2.0 } else { 1.0 };
                assert!(e.total() < long, "{id:?} {p:?}: {}", e.total());
                assert!(e.active() >= d.active());
            }
        }
        // G2의 고르기와 맺음(0158 "G2" 표의 값).
        assert_eq!(
            Id::Brace.def().picks,
            [Twist::Walk { speed: 0.45 }, Twist::Reflect]
        );
        assert_eq!(
            Id::Echo.def().picks,
            [Twist::Rattle { poise: 2.5 }, Twist::Tempo { secs: 0.6 }]
        );
        assert_eq!(
            Id::Throw.def().picks,
            [
                Twist::Stick {
                    radius: 2.5,
                    damage: 20
                },
                Twist::Drag { dist: 2.0 }
            ]
        );
        assert_eq!(
            Id::Embers.def().picks,
            [Twist::Follow, Twist::Kindle { embers: 1 }]
        );
        assert_eq!(Id::Brace.def().cap, Cap::Stun { stagger: 1.0 });
        assert_eq!(Id::Echo.def().cap, Cap::Toll { damage: 10 });
        assert_eq!(
            Id::Throw.def().cap,
            Cap::Homing {
                damage: 1.5,
                poise: 2.0
            }
        );
        assert_eq!(
            Id::Embers.def().cap,
            Cap::Smolder {
                secs: 1.0,
                poise: 1.0
            }
        );
        // 표의 바탕값(0158 G2): 대기·시동·후딜과 모양의 수.
        let row = |id: Id| {
            let d = id.def();
            (d.cooldown, d.startup, d.recover, d.form)
        };
        assert_eq!(
            row(Id::Brace),
            (
                12.0,
                0.10,
                0.35,
                Form::Brace {
                    hold: 1.2,
                    radius: 3.0,
                    soak: 10,
                    most: 3,
                    more: 10,
                    push: 1.0
                }
            )
        );
        assert_eq!(
            row(Id::Echo),
            (
                12.0,
                0.25,
                0.20,
                Form::Echo {
                    secs: 4.0,
                    radius: 4.0,
                    stagger: 0.8
                }
            )
        );
        assert!(matches!(
            row(Id::Throw),
            (cd, st, rc, Form::Throw { dist, .. }) if cd == 8.0 && st == 0.18 && rc == 0.25 && dist == 8.0
        ));
        assert_eq!(
            row(Id::Embers),
            (
                10.0,
                0.25,
                0.30,
                Form::Field {
                    radius: 4.0,
                    secs: 3.0,
                    every: 1.0
                }
            )
        );
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
        let upend = Entry::Ult(Ult::Upend);
        assert_eq!((upend.free(), upend.max()), (1, 3));
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
            rank: [0, 9, 2, 3, 0, 0, 7, 0],
            pick: [Pick::B; N],
            ult: [0, 9, 0],
            cells: Tree::default(),
        };
        let b = odd.sane(40);
        assert_eq!(b.rank, [1, MAX_RANK, 2, 3, 0, 0, MAX_RANK, 0]);
        assert_eq!(
            b.pick,
            [
                Pick::A,
                Pick::B,
                Pick::A,
                Pick::B,
                Pick::A,
                Pick::A,
                Pick::B,
                Pick::A
            ]
        );
        assert_eq!(b.ult, [1, ULT_MAX_RANK, 0]);
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
            },
            ..Board::default()
        };
        assert_eq!(legacy.sane(9).cells, Tree::from_counts([2, 0, 0]));
    }

    /// 월드에 넘기는 끼운 둘: 그 스킬의 단·고르기 + 큰 기술의 단. 끼울 수 있는 것은 배운 스킬뿐.
    #[test]
    fn the_loadout_carries_the_worn_ranks_and_only_learned_skills_are_wearable() {
        let mut b = Board::default();
        b.rank[..4].copy_from_slice(&[3, 1, 0, 5]);
        b.pick[Id::Whirl.index()] = Pick::B;
        b.ult[0] = 2;
        let l = b.loadout([Id::Quake, Id::Whirl], Ult::Upend);
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
        assert_eq!((l.ult, l.great), (2, Ult::Upend));
        // 끼운 큰 기술이 안 배운 것이면 쟁반 뒤집기로, 배웠으면 그것과 그 단.
        assert_eq!(b.loadout(START, Ult::Line).great, Ult::Upend);
        b.ult[Ult::Line.index()] = 3;
        let l = b.loadout(START, Ult::Line);
        assert_eq!((l.great, l.ult), (Ult::Line, 3));
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

    /// 0158 G2: 큰 기술은 셋 — 쟁반 뒤집기는 처음부터 1단, 몰아치기·한 줄기는 큰 기술 줄(쓴 점수 8)이 열리면 **2점**으로 배우고
    /// 2·3단은 1점씩. 점수가 1뿐이면 배우지 못한다.
    #[test]
    fn the_other_great_skills_are_learned_for_two_points_once_eight_are_spent() {
        assert_eq!(ULTS, [Ult::Upend, Ult::Flurry, Ult::Line]);
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
        assert_eq!(Entry::Ult(Ult::Upend).price(1), 1);
        assert_eq!((flurry.paid(1), flurry.paid(3)), (2, 4));
        assert_eq!(Entry::Ult(Ult::Upend).paid(3), 2);
        let mut b = Board::default();
        assert_eq!(b.offer(flurry, 23), Offer::Locked(8));
        for _ in 0..4 {
            assert!(b.buy(Entry::Skill(Id::Whirl), 23) && b.buy(Entry::Skill(Id::Cut), 23));
        }
        assert_eq!(b.spent(), 8);
        assert!(!b.has_ult(Ult::Flurry));
        assert_eq!(b.offer(flurry, 9), Offer::Short, "one point is not enough");
        assert!(!b.buy(flurry, 9));
        assert_eq!(b.offer(flurry, 10), Offer::Ready);
        assert!(b.buy(flurry, 10));
        assert!(b.has_ult(Ult::Flurry));
        assert_eq!((b.spent(), b.left(10)), (10, 0));
        assert!(b.buy(flurry, 12) && b.buy(flurry, 12));
        assert_eq!(b.offer(flurry, 99), Offer::Full);
        assert_eq!(b.spent(), 12);
        assert!(b.buy(line, 14));
        assert_eq!(b.ult, [1, 3, 1]);
        // 되돌리면 배운 큰 기술도 돌아가고, 끼운 큰 기술은 쟁반 뒤집기로.
        let (mut worn, mut great) = (START, Ult::Line);
        b.respec(&mut worn, &mut great);
        assert_eq!((b.ult, great), ([1, 0, 0], Ult::Upend));
        assert_eq!(b.wearable_ult(Ult::Flurry), Ult::Upend);
    }

    /// 0158 G2: 몰아치기는 5초(2단 6.5초, 3단은 쓰러뜨릴 때마다 +0.5초 — 3초까지)·1.4배, 한 줄기는 12m(2단 15m)·피해 40(3단 60).
    #[test]
    fn flurry_and_line_rank_numbers() {
        assert_eq!((FLURRY.startup, FLURRY.speed), (0.3, 1.4));
        assert_eq!([1, 2, 3].map(|r| FLURRY.secs_at(r)), [5.0, 6.5, 6.5]);
        assert_eq!([1, 2, 3].map(|r| FLURRY.kill_more_at(r)), [0.0, 0.0, 0.5]);
        assert_eq!(FLURRY.kill_cap, 3.0);
        assert_eq!((LINE.freeze, LINE.half), (0.3, 1.2));
        assert_eq!([1, 2, 3].map(|r| LINE.len_at(r)), [12.0, 15.0, 15.0]);
        assert_eq!([1, 2, 3].map(|r| LINE.damage_at(r)), [40, 40, 60]);
        assert!(LINE.poise >= 10.0, "very large poise damage");
    }

    /// 0158 G2 동작: 상태의 시각 → 동작의 시각은 구간으로 맞춘다 — 시동의 끝은 늘 시동 구간의 끝(판정의 순간), 이어짐이
    /// 길어지면(셋째 바퀴·되돌아 베기) 이어짐 구간을 되풀이하고, 후딜은 후딜 구간에서 시작해 동작의 끝을 넘지 않는다.
    #[test]
    fn a_clip_maps_each_phase_onto_its_own_segment_whatever_the_rank() {
        let near = |a: f32, b: f32| (a - b).abs() < 1e-4;
        for id in IDS {
            for (rank, pick) in [(1, Pick::A), (MAX_RANK, Pick::A), (MAX_RANK, Pick::B)] {
                let e = id.def().at(rank, pick);
                let (c, s) = (e.skill.clip, e.skill);
                let period = match s.form {
                    Form::Ring { gap, .. } => Some(gap),
                    Form::Lane { time, .. } => Some(time),
                    _ => None,
                };
                let at = |t: f32| c.at(t, s.startup, e.active(), period);
                let tag = format!("{id:?} {rank} {pick:?}");
                assert!(near(at(0.0), 0.0), "{tag}: starts at 0");
                // 시동의 끝 = 시동 구간의 끝.
                assert!(
                    near(at(s.startup - 1e-5), c.seg[0]),
                    "{tag}: {}",
                    at(s.startup - 1e-5)
                );
                // 이어짐 안은 이어짐 구간 안.
                if e.active() > 0.0 {
                    for k in [0.1, 0.5, 0.9] {
                        let t = at(s.startup + e.active() * k);
                        assert!(
                            t >= c.seg[0] - 1e-4 && t <= c.seg[0] + c.seg[1] + 1e-4,
                            "{tag}: active at {t}"
                        );
                    }
                }
                // 후딜의 시작 = 후딜 구간의 시작, 끝은 동작의 끝을 넘지 않는다.
                let rec = s.startup + e.active();
                assert!(near(at(rec), c.seg[0] + c.seg[1]), "{tag}: {}", at(rec));
                assert!(near(at(rec + 0.1), c.seg[0] + c.seg[1] + 0.1), "{tag}");
                assert!(near(at(rec + 9.0), c.total()), "{tag}: holds the last pose");
                // 뒤로 가지 않는다(되풀이하는 이어짐 안에서만 감긴다).
                assert!(at(rec + 0.05) > at(rec));
            }
        }
        // 휘돌이의 셋째 바퀴: 이어짐 구간(한 바퀴)을 바퀴마다 한 번씩 — 둘째 바퀴의 가운데 = 구간의 가운데.
        let w = Id::Whirl.def().at(3, Pick::B);
        let (c, s) = (w.skill.clip, w.skill);
        assert!(near(w.active(), 0.44));
        let mid = c.at(s.startup + 0.22 + 0.11, s.startup, w.active(), Some(0.22));
        assert!(near(mid, c.seg[0] + c.seg[1] * 0.5), "{mid}");
        // 큰 기술: 세계 멈춤은 시동 구간에, 그 뒤 터지기까지는 이어짐 구간에 — 터지는 순간 = 이어짐 구간의 끝.
        for u in [Ult::Upend, Ult::Line] {
            let (c, (_, startup, _)) = (u.clip(), u.times());
            assert!(near(u.clip_at(0.0, 0.0), 0.0), "{u:?}");
            assert!(near(u.clip_at(0.0, 0.5), c.seg[0] * 0.5));
            assert!(near(u.clip_at(0.0, 1.0), c.seg[0]));
            assert!(
                near(u.clip_at(startup - 1e-5, 1.0), c.seg[0] + c.seg[1]),
                "{u:?}"
            );
            assert!(near(
                u.clip_at(startup + 0.1, 1.0),
                c.seg[0] + c.seg[1] + 0.1
            ));
            assert!(near(u.clip_at(99.0, 1.0), c.total()));
        }
        // 몰아치기는 멈춤이 없다: 시동이 곧 시동 구간.
        let c = FLURRY.clip;
        assert!(near(
            Ult::Flurry.clip_at(FLURRY.startup * 0.5, 0.0),
            c.seg[0] * 0.5
        ));
        assert!(near(Ult::Flurry.clip_at(FLURRY.startup, 0.0), c.seg[0]));
    }
}
