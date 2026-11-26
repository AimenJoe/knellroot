//! 궁극기(0151 → 0158 G2 · 0272): `Ult` · 궁극기마다의 표(`UPEND` · `FLURRY` …) · 궁극기의 단.
use super::*;
use crate::progression::Branch;
use crate::status::{Dose, Kind};
use crate::tuning::{HP_UNIT, ULT_SLAM_DAMAGE};

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
    /// 큰 종(0270 C3 — 방패 판): 방패로 땅을 쳐 둘레를 크게 울린다 — 울림과 흡수를 함께 건다.
    Bell,
    /// 눈보라(0270 C3 — 지팡이 판): 몸을 따라 도는 큰 눈보라 — 둘레의 떼를 거듭 치며 얼린다.
    Blizzard,
    /// 화살비(0272 — 활 판): 겨눈 자리에 화살을 쏟아붓는다 — 그 둘레의 떼를 거듭 꿰고 묶는다.
    Rain,
    /// 질풍 찌르기(0272 2단계 — 창 판): 한 줄을 잇달아 찌르고 마지막에 깊게 꿰뚫는다.
    Gale,
    /// 그림자 습격(0274 ③ — 쌍단검 판): 한 적의 등 뒤로 숨어들어 잇달아 찌르고 마지막에 깊게 벤다.
    Ambush,
    /// 칼바람(0274 ④ — 대검 판): 제자리에서 잇달아 휘돌며 둘레를 거듭 베고 마지막에 크게 휩쓴다 — 그동안과 그 뒤 잠깐 맞아도 휘청이지 않는다.
    Tempest,
    /// 깨어나는 숲(0274 ⑤ — 낫 판): 빈 손으로 땅을 짚어 가시 뿌리의 고리를 잇달아 바깥으로 솟게 한다 — 묶고, 마지막 고리에 거둔 만큼 되살아난다.
    Grove,
    /// 치른 값(0274 ⑥ — 완드 판): 둘레의 적에게 저주를 퍼뜨려 금을 내고, 잠시 뒤 그 값이 울림으로 터진다 — 금 간 적은 더 크게.
    Price,
}

/// 큰 기술의 표 순서(판의 단 `Board::ult`의 자리 — 뒤에만 붙인다).
pub const ULTS: [Ult; 11] = [
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
    Ult::Price,
];
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
            Ult::Bell => "bell",
            Ult::Blizzard => "blizzard",
            Ult::Rain => "rain",
            Ult::Gale => "gale",
            Ult::Ambush => "ambush",
            Ult::Tempest => "tempest",
            Ult::Grove => "grove",
            Ult::Price => "price",
        }
    }

    pub fn by_key(k: &str) -> Option<Ult> {
        ULTS.into_iter().find(|u| u.key() == k)
    }

    /// 판의 갈래(0161 "갈래와 줄"): 몰아치기 = 흐름, 한 줄기 = 맞받기, 쟁반 뒤집기 = 자리.
    pub fn branch(self) -> Branch {
        match self {
            Ult::Upend | Ult::Blizzard | Ult::Rain | Ult::Grove => Branch::Footing,
            Ult::Flurry | Ult::Ambush | Ult::Tempest => Branch::Beat,
            Ult::Line | Ult::Bell | Ult::Gale | Ult::Price => Branch::Riposte,
        }
    }

    /// 빌려 쓰는 동작과 전용 동작(스킬의 `anim`·`clip`과 같은 뜻).
    pub fn anim(self) -> &'static str {
        match self {
            Ult::Upend => UPEND.anim,
            Ult::Flurry => FLURRY.anim,
            Ult::Line => LINE.anim,
            Ult::Bell => BELL.anim,
            Ult::Blizzard => BLIZZARD.anim,
            Ult::Rain => RAIN.anim,
            Ult::Gale => GALE.anim,
            Ult::Ambush => AMBUSH.anim,
            Ult::Tempest => TEMPEST.anim,
            Ult::Grove => GROVE.anim,
            Ult::Price => PRICE.anim,
        }
    }

    pub fn clip(self) -> Clip {
        match self {
            Ult::Upend => UPEND.clip,
            Ult::Flurry => FLURRY.clip,
            Ult::Line => LINE.clip,
            Ult::Bell => BELL.clip,
            Ult::Blizzard => BLIZZARD.clip,
            Ult::Rain => RAIN.clip,
            Ult::Gale => GALE.clip,
            Ult::Ambush => AMBUSH.clip,
            Ult::Tempest => TEMPEST.clip,
            Ult::Grove => GROVE.clip,
            Ult::Price => PRICE.clip,
        }
    }

    /// (누른 순간 세계가 멈추는 초, 그 뒤 터지기까지의 시동, 후딜) — 상태 틱과 동작 맞추기가 같이 읽는다.
    pub fn times(self) -> (f32, f32, f32) {
        match self {
            Ult::Upend => (UPEND.freeze, UPEND.startup, UPEND.recover),
            Ult::Flurry => (0.0, FLURRY.startup, FLURRY.recover),
            Ult::Line => (LINE.freeze, LINE.startup, LINE.recover),
            Ult::Bell => (BELL.freeze, BELL.startup, BELL.recover),
            Ult::Blizzard => (BLIZZARD.freeze, BLIZZARD.startup, BLIZZARD.recover),
            Ult::Rain => (RAIN.freeze, RAIN.startup, RAIN.recover),
            Ult::Gale => (GALE.freeze, GALE.startup, GALE.recover()),
            Ult::Ambush => (AMBUSH.freeze, AMBUSH.startup, AMBUSH.recover()),
            Ult::Tempest => (TEMPEST.freeze, TEMPEST.startup, TEMPEST.recover()),
            Ult::Grove => (GROVE.freeze, GROVE.startup, GROVE.recover()),
            Ult::Price => (PRICE.freeze, PRICE.startup, PRICE.recover()),
        }
    }

    /// 큰 기술 동작의 시각: 세계 멈춤(`frozen` = 멈춤이 지난 몫 0..1, 상태 시계는 0에 머문다)은 시동 구간에, 그 뒤 터지기까지의
    /// 시동은 이어짐 구간에 늘여 맞춘다 — 이어짐 구간의 끝이 터지는 순간. 멈춤이 없는 큰 기술(몰아치기)은 시동이 곧 시동 구간.
    pub fn clip_at(self, e: f32, frozen: f32) -> f32 {
        let (freeze, startup, _) = self.times();
        let c = self.clip();
        // 칼바람(0274 ④): 멈춤 동안 들어 올리고, 첫 휩쓸기(시동 끝)부터 휩쓸기마다 한 바퀴를 되풀이한 뒤 후딜.
        if self == Ult::Tempest {
            let t = &TEMPEST;
            if e <= 0.0 {
                return c.seg[0] * frozen.clamp(0.0, 1.0);
            }
            if e < startup {
                return c.seg[0];
            }
            return c.at(e - startup, 0.0, t.sweeps as f32 * t.gap, Some(t.gap));
        }
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

/// 한 줄기의 타입(0223 V2): 가르는 줄 — 닿은 적마다 금이 간다(큰 기술이라 보스도 한 번에 — 닳기 전이면).
pub const LINE_DOSE: Dose = (Kind::Rift, 2.0);

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

/// 큰 기술 "큰 종"(0270 C3 — 방패 판): 시동의 세계 멈춤 뒤 둘레 `radius` m의 깨어 있는 적을 한 번 친다 — 피해 `damage`·자세 피해
/// `poise`, 울림과 흡수를 함께(`dose`·`drain` — 큰 기술이라 보스도 한 번에, 닳기 전이면), 잡몹은 `stagger`초 휘청(보스는 아니다).
/// 2단 = 둘레 `radius2`, 3단 = 피해 `damage3`.
pub struct Bell {
    pub freeze: f32,
    pub startup: f32,
    pub recover: f32,
    pub radius: f32,
    pub radius2: f32,
    pub damage: i32,
    pub damage3: i32,
    pub poise: f32,
    pub stagger: f32,
    pub dose: Dose,
    pub drain: Dose,
    pub anim: &'static str,
    pub clip: Clip,
}

pub const BELL: Bell = Bell {
    freeze: 0.3,
    startup: 0.15,
    recover: 0.45,
    radius: 8.0,
    radius2: 10.0,
    damage: 2 * HP_UNIT,
    damage3: 3 * HP_UNIT,
    poise: 6.0,
    stagger: 0.8,
    dose: (Kind::Knell, 2.0),
    drain: (Kind::Drain, 2.0),
    anim: "hammer_slam",
    clip: Clip {
        name: "ult_bell",
        seg: [0.30, 0.15, 0.45],
    },
};

impl Bell {
    pub fn radius_at(&self, rank: u8) -> f32 {
        if rank >= 2 {
            self.radius2
        } else {
            self.radius
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

/// 큰 기술 "눈보라"(0270 C3 — 지팡이 판): 시동 뒤 몸을 따라 도는 눈보라(둘레 `radius` m · `secs`초) — `every`초마다 그 안의 적을
/// 피해 `damage`로 치고 서리를 `build`만큼 쌓는다(잔불 흩기의 불티와 같은 길 — 따르는 마당). 2단 = `secs2`초, 3단 = 피해 `damage3`.
pub struct Blizzard {
    pub freeze: f32,
    pub startup: f32,
    pub recover: f32,
    pub radius: f32,
    pub secs: f32,
    pub secs2: f32,
    pub every: f32,
    pub damage: i32,
    pub damage3: i32,
    pub build: f32,
    pub anim: &'static str,
    pub clip: Clip,
}

pub const BLIZZARD: Blizzard = Blizzard {
    freeze: 0.3,
    startup: 0.20,
    recover: 0.40,
    radius: 6.0,
    secs: 4.0,
    secs2: 5.5,
    every: 0.5,
    damage: HP_UNIT,
    damage3: 2 * HP_UNIT,
    build: 0.35,
    anim: "staff_charge",
    clip: Clip {
        name: "ult_blizzard",
        seg: [0.30, 0.20, 0.40],
    },
};

impl Blizzard {
    pub fn secs_at(&self, rank: u8) -> f32 {
        if rank >= 2 {
            self.secs2
        } else {
            self.secs
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

/// 큰 기술 "화살비"(0272 — 활 판): 시동 뒤 겨눈 쪽 `ahead` m 앞(락온·가까운 적이 그 안이면 그 적의 자리)에 둘레 `radius` m의
/// 화살비가 `secs`초 내린다 — `every`초마다 그 안의 적을 피해 `damage`로 꿰고 뿌리를 `build`만큼 쌓는다(눈보라와 같은 길 — 놓인
/// 마당이라 몸을 따르지 않는다: 멀리 놓고 물러서는 활의 큰 기술). 2단 = `secs2`초, 3단 = 피해 `damage3`.
pub struct Rain {
    pub freeze: f32,
    pub startup: f32,
    pub recover: f32,
    pub ahead: f32,
    pub radius: f32,
    pub secs: f32,
    pub secs2: f32,
    pub every: f32,
    pub damage: i32,
    pub damage3: i32,
    pub build: f32,
    pub anim: &'static str,
    pub clip: Clip,
}

pub const RAIN: Rain = Rain {
    freeze: 0.3,
    startup: 0.25,
    recover: 0.40,
    ahead: 7.0,
    radius: 4.5,
    secs: 3.2,
    secs2: 4.4,
    every: 0.4,
    damage: HP_UNIT,
    damage3: 2 * HP_UNIT,
    build: 0.35,
    anim: "bow_shoot",
    clip: Clip {
        name: "ult_rain",
        seg: [0.30, 0.25, 0.40],
    },
};

impl Rain {
    pub fn secs_at(&self, rank: u8) -> f32 {
        if rank >= 2 {
            self.secs2
        } else {
            self.secs
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

/// 큰 기술 "질풍 찌르기"(0272 2단계 — 창 판): 세계 멈춤 뒤 시동, 그 뒤 앞 한 줄(길이 `len` m · 반 폭 `half` m — 벽 · 높은 턱 앞에서
/// 끊긴다)을 `gap`초마다 `thrusts`번 찌른다 — 앞의 찌름마다 줄 위의 적을 모두 `damage`·`poise`로, 마지막 찌름은 깊게: 피해 `last`·
/// 자세 피해 `last_poise`, 날카로움(출혈)을 `dose`만큼, 잡몹은 `push` m 밀려나 `stagger`초 휘청(보스는 다치고 자세가 깎이되 밀리지도
/// 휘청이지도 않는다 — 큰 기술의 규칙). 몸은 서서 찌른다(한 줄기처럼 쪽은 누르는 순간에). 2단 = 길이 `len2`, 3단 = 피해 `damage3`·`last3`.
pub struct Gale {
    pub freeze: f32,
    pub startup: f32,
    pub thrusts: u8,
    pub gap: f32,
    /// 마지막 찌름 뒤의 후딜(초).
    pub tail: f32,
    pub len: f32,
    pub len2: f32,
    pub half: f32,
    pub damage: i32,
    pub damage3: i32,
    pub last: i32,
    pub last3: i32,
    pub poise: f32,
    pub last_poise: f32,
    pub push: f32,
    pub stagger: f32,
    pub dose: Dose,
    pub anim: &'static str,
    pub clip: Clip,
}

pub const GALE: Gale = Gale {
    freeze: 0.3,
    startup: 0.2,
    thrusts: 6,
    gap: 0.11,
    tail: 0.4,
    len: 5.5,
    len2: 7.5,
    half: 0.8,
    damage: HP_UNIT,
    damage3: 2 * HP_UNIT,
    last: 4 * HP_UNIT,
    last3: 6 * HP_UNIT,
    poise: 1.0,
    last_poise: 8.0,
    push: 2.5,
    stagger: 0.8,
    dose: (Kind::Sharp, 2.0),
    anim: "spear3",
    clip: Clip {
        name: "ult_gale",
        seg: [0.30, 0.20, 0.95],
    },
};

impl Gale {
    /// 후딜(초): 찌름이 모두 이 안에 있다 — 첫 찌름은 시동 끝, 마지막은 `(thrusts - 1) × gap` 뒤.
    pub const fn recover(&self) -> f32 {
        (self.thrusts as f32 - 1.0) * self.gap + self.tail
    }

    /// `k`째 찌름의 때(시동 끝에서 초).
    pub fn thrust_at(&self, k: u8) -> f32 {
        k as f32 * self.gap
    }

    pub fn len_at(&self, rank: u8) -> f32 {
        if rank >= 2 {
            self.len2
        } else {
            self.len
        }
    }

    /// `rank`단의 (앞 찌름의 피해, 마지막 찌름의 피해).
    pub fn damage_at(&self, rank: u8) -> (i32, i32) {
        if rank >= ULT_MAX_RANK {
            (self.damage3, self.last3)
        } else {
            (self.damage, self.last)
        }
    }
}

/// 큰 기술 "그림자 습격"(0274 ③ — 쌍단검 판, BG3 도적의 암습): 세계 멈춤 동안 `range` m 안의 한 적(락온 > 조준 쪽 > 가장 가까운)의
/// **등 뒤로** 숨어든다(벽 · 걸을 수 없는 칸이면 그 적의 이쪽 곁) — 그 뒤 시동 끝부터 `gap`초마다 `stabs`번 그 적을 찌른다: 앞의 찌름은
/// `damage` · 자세 `poise` · 균열 `light_dose`, 마지막은 깊게 `last` · 자세 `last_poise` · 날카로움 `dose`(출혈), 잡몹은 `stagger`초 휘청.
/// 등 뒤에서 찌르니 주특성 급소(`Art::sneak`)가 저절로 얹힌다. 보스는 다치고 자세가 깎이되 휘청이지 않는다(큰 기술의 규칙).
/// 적이 없으면 제자리에서 앞을 찌른다(짧은 부채꼴). 2단 = 닿는 거리 `range2`, 3단 = 피해 `damage3` · `last3`.
pub struct Ambush {
    pub freeze: f32,
    pub startup: f32,
    pub stabs: u8,
    pub gap: f32,
    pub tail: f32,
    pub range: f32,
    pub range2: f32,
    /// 찌름이 닿는 거리(숨어든 자리에서 — 적의 몸 밖으로).
    pub reach: f32,
    pub damage: i32,
    pub damage3: i32,
    pub last: i32,
    pub last3: i32,
    pub poise: f32,
    pub last_poise: f32,
    pub stagger: f32,
    pub light_dose: Dose,
    pub dose: Dose,
    pub anim: &'static str,
    pub clip: Clip,
}

pub const AMBUSH: Ambush = Ambush {
    freeze: 0.3,
    startup: 0.15,
    stabs: 4,
    gap: 0.12,
    tail: 0.4,
    range: 7.0,
    range2: 10.0,
    reach: 1.4,
    damage: HP_UNIT,
    damage3: 2 * HP_UNIT,
    last: 5 * HP_UNIT,
    last3: 8 * HP_UNIT,
    poise: 0.5,
    last_poise: 4.0,
    stagger: 0.8,
    light_dose: (Kind::Rift, 0.4),
    dose: (Kind::Sharp, 2.0),
    anim: "dagger4",
    clip: Clip {
        name: "ult_ambush",
        seg: [0.30, 0.15, 0.76],
    },
};

/// 그림자 습격의 자리 셈: 숨어든 자리와 적의 몸 사이(m) · 조준 쪽에서 이 코사인 안의 적(±45°) · 고른 적이 없을 때 찌르는 앞 부채꼴(°) ·
/// 마지막 찌름에 잡몹이 밀리는 거리(m) · 앞의 찌름에 잡몹이 휘청이는 초(등 뒤의 급소가 이어지게).
pub const AMBUSH_GAP: f32 = 0.15;
pub const AMBUSH_AIM_COS: f32 = std::f32::consts::FRAC_1_SQRT_2;
pub const AMBUSH_CONE_DEG: f32 = 100.0;
pub const AMBUSH_PUSH: f32 = 1.5;
pub const AMBUSH_HOLD: f32 = 0.2;

impl Ambush {
    /// 후딜(초): 찌름이 모두 이 안에 — 첫 찌름은 시동 끝.
    pub const fn recover(&self) -> f32 {
        (self.stabs as f32 - 1.0) * self.gap + self.tail
    }

    /// `k`째 찌름의 때(시동 끝에서 초).
    pub fn stab_at(&self, k: u8) -> f32 {
        k as f32 * self.gap
    }

    pub fn range_at(&self, rank: u8) -> f32 {
        if rank >= 2 {
            self.range2
        } else {
            self.range
        }
    }

    /// `rank`단의 (앞 찌름의 피해, 마지막 찌름의 피해).
    pub fn damage_at(&self, rank: u8) -> (i32, i32) {
        if rank >= ULT_MAX_RANK {
            (self.damage3, self.last3)
        } else {
            (self.damage, self.last)
        }
    }
}

/// 큰 기술 "칼바람"(0274 ④ — 대검 판, 광전사의 분노가 터진다): 세계 멈춤 뒤 시동 끝부터 `gap`초마다 `sweeps`번 제자리에서 휘돌며 둘레
/// `radius` m(2단 `radius2`)를 벤다 — 앞의 휩쓸기는 `damage` · 자세 `poise` · 균열 `light_dose`(밀지 않는다 — 떼가 둘레에 남는다), 마지막은
/// 크게 `last` · 자세 `last_poise` · 잔불 `dose`(화상), 잡몹은 `push` m 밀려 `stagger`초 휘청. 그동안과 시작에서 `steady`초는 맞아도 휘청이지
/// 않는다(받는 피해는 그대로 — 0095 원칙 2). 보스는 다치고 자세가 깎이되 휘청이지 않는다(큰 기술의 규칙). 3단 = 피해 `damage3` · `last3`.
pub struct Tempest {
    pub freeze: f32,
    pub startup: f32,
    pub sweeps: u8,
    pub gap: f32,
    pub tail: f32,
    pub radius: f32,
    pub radius2: f32,
    pub damage: i32,
    pub damage3: i32,
    pub last: i32,
    pub last3: i32,
    pub poise: f32,
    pub last_poise: f32,
    pub push: f32,
    pub stagger: f32,
    pub light_dose: Dose,
    pub dose: Dose,
    pub steady: f32,
    pub anim: &'static str,
    pub clip: Clip,
}

pub const TEMPEST: Tempest = Tempest {
    freeze: 0.3,
    startup: 0.2,
    sweeps: 5,
    gap: 0.16,
    tail: 0.45,
    radius: 3.0,
    radius2: 3.8,
    damage: HP_UNIT,
    damage3: 2 * HP_UNIT,
    last: 4 * HP_UNIT,
    last3: 6 * HP_UNIT,
    poise: 1.0,
    last_poise: 6.0,
    push: 2.0,
    stagger: 0.8,
    light_dose: (Kind::Rift, 0.3),
    dose: (Kind::Ember, 1.5),
    steady: 3.0,
    anim: "sk_whirl",
    // 휘돌이와 같은 한 바퀴(시동 · 한 바퀴 · 후딜) — 휩쓸기마다 한 바퀴를 되풀이한다(`Ult::clip_at`).
    clip: Clip {
        name: "ult_tempest",
        seg: [0.15, 0.22, 0.30],
    },
};

impl Tempest {
    /// 후딜(초): 휩쓸기가 모두 이 안에 — 첫 휩쓸기는 시동 끝.
    pub const fn recover(&self) -> f32 {
        (self.sweeps as f32 - 1.0) * self.gap + self.tail
    }

    /// `k`째 휩쓸기의 때(시동 끝에서 초).
    pub fn sweep_at(&self, k: u8) -> f32 {
        k as f32 * self.gap
    }

    pub fn radius_at(&self, rank: u8) -> f32 {
        if rank >= 2 {
            self.radius2
        } else {
            self.radius
        }
    }

    /// `rank`단의 (앞 휩쓸기의 피해, 마지막 휩쓸기의 피해).
    pub fn damage_at(&self, rank: u8) -> (i32, i32) {
        if rank >= ULT_MAX_RANK {
            (self.damage3, self.last3)
        } else {
            (self.damage, self.last)
        }
    }
}

/// 큰 기술 "깨어나는 숲"(0274 ⑤ — 낫 판, 드루이드의 결 "뿌리를 부려 묶고 되살아난다"): 세계 멈춤 뒤 빈 손으로 땅을 짚어(시동) 시동 끝부터
/// `gap`초마다 가시 뿌리의 고리가 몸에서 바깥으로 솟는다 — k째 고리는 둘레 `radii[k]` m(2단 × `grow2`) 안의 깨어 있는 적을 모두 찌른다
/// (가까운 적일수록 여러 번 — 한가운데의 떼를 묶는다): 앞의 고리는 `damage` · 자세 `poise` · 뿌리 `dose`(밀지 않는다 — 둘이면 묶인다,
/// 뿌리에 덜 걸리는 잔재 ×0.75도),
/// 마지막 고리는 `last`(3단 `last3`) · 자세 `last_poise` · 흡수 `last_dose`(그 뒤의 타가 되돌린다). 마지막 고리에 닿은 적마다 체력
/// `heal_each`가 돈다(많아야 `heal_most` — 받는 피해는 그대로, 되찾을 뿐 0095 원칙 2). 보스는 다치고 묶이되(느려지기만) 휘청이지 않는다.
pub struct Grove {
    pub freeze: f32,
    pub startup: f32,
    pub gap: f32,
    pub tail: f32,
    pub radii: [f32; 3],
    pub grow2: f32,
    pub damage: i32,
    pub last: i32,
    pub last3: i32,
    pub poise: f32,
    pub last_poise: f32,
    pub dose: Dose,
    pub last_dose: Dose,
    pub heal_each: i32,
    pub heal_most: i32,
    pub anim: &'static str,
    pub clip: Clip,
}

pub const GROVE: Grove = Grove {
    freeze: 0.3,
    startup: 0.25,
    gap: 0.22,
    tail: 0.5,
    radii: [2.2, 3.4, 4.6],
    grow2: 1.2,
    damage: HP_UNIT,
    last: 3 * HP_UNIT,
    last3: 5 * HP_UNIT,
    poise: 1.0,
    last_poise: 4.0,
    dose: (Kind::Root, 0.7),
    last_dose: (Kind::Drain, 1.0),
    heal_each: 4,
    heal_most: 2 * HP_UNIT,
    anim: "pickup",
    // 웅크려 빈 손으로 땅을 짚는 동작(줍기 원본 Collect_Object) — 멈춤 · 시동 · 후딜(고리가 솟는 동안 짚고 있다가 일어선다).
    clip: Clip {
        name: "ult_grove",
        seg: [0.30, 0.25, 0.94],
    },
};

impl Grove {
    /// 고리의 수.
    pub const fn pulses(&self) -> u8 {
        self.radii.len() as u8
    }

    /// 후딜(초): 고리가 모두 이 안에 — 첫 고리는 시동 끝.
    pub const fn recover(&self) -> f32 {
        (self.pulses() as f32 - 1.0) * self.gap + self.tail
    }

    /// `k`째 고리의 때(시동 끝에서 초).
    pub fn pulse_at(&self, k: u8) -> f32 {
        k as f32 * self.gap
    }

    /// `k`째 고리의 둘레(m) — 2단부터 넓다.
    pub fn radius_at(&self, k: u8, rank: u8) -> f32 {
        self.radii[(k as usize).min(self.radii.len() - 1)]
            * if rank >= 2 { self.grow2 } else { 1.0 }
    }

    /// `rank`단의 마지막 고리 피해.
    pub fn last_at(&self, rank: u8) -> i32 {
        if rank >= ULT_MAX_RANK {
            self.last3
        } else {
            self.last
        }
    }
}

/// 큰 기술 "치른 값"(0274 ⑥ — 완드 판, 흑마법사의 결 "저주로 약하게 하고 죽음의 울림을 부른다"): 세계 멈춤 뒤 완드를 들어(시동) 시동 끝에
/// 둘레 `radius` m(2단 `radius2`) 안의 깨어 있는 적 모두에게 저주가 번진다 — `curse` 피해 · 자세 `curse_poise` · `curse_dose`(균열 가득 — 보통
/// 적은 곧바로 금이 간다). `gap`초 뒤 그 값이 울림으로 터진다 — 같은 둘레의 적 모두에 `toll`(3단 `toll3`) · 자세 `toll_poise` · `toll_dose`
/// (울림 가득 — 울린 적이 둘레를 다시 친다), 금 간 적은 금의 배만큼 더 다친다(주특성 치른 값이면 더). 밀지 않는다. 보스는 다치되 휘청이지
/// 않는다. 상대를 부르지 않는다 — 무엇을 치렀는지는 화면이 말하지 않는다(0274 "계약").
pub struct Price {
    pub freeze: f32,
    pub startup: f32,
    pub gap: f32,
    pub tail: f32,
    pub radius: f32,
    pub radius2: f32,
    pub curse: i32,
    pub curse_poise: f32,
    pub curse_dose: Dose,
    pub toll: i32,
    pub toll3: i32,
    pub toll_poise: f32,
    pub toll_dose: Dose,
    pub anim: &'static str,
    pub clip: Clip,
}

pub const PRICE: Price = Price {
    freeze: 0.3,
    startup: 0.3,
    gap: 0.45,
    tail: 0.45,
    radius: 5.0,
    radius2: 6.0,
    curse: HP_UNIT,
    curse_poise: 0.5,
    curse_dose: (Kind::Rift, 1.0),
    toll: 3 * HP_UNIT,
    toll3: 5 * HP_UNIT,
    toll_poise: 3.0,
    toll_dose: (Kind::Knell, 1.0),
    anim: "cast",
    // 완드를 들어 올렸다 내리꽂는 시전(지팡이 판의 눈보라와 같은 원본 Charged_Axe_Chop) — 멈춤 · 시동(번지는 저주) · 후딜(터지는 울림).
    clip: Clip {
        name: "ult_price",
        seg: [0.30, 0.30, 0.90],
    },
};

impl Price {
    /// 후딜(초): 터지는 울림(시동 끝에서 `gap`초)이 이 안에.
    pub const fn recover(&self) -> f32 {
        self.gap + self.tail
    }

    /// `rank`단의 둘레(m).
    pub fn radius_at(&self, rank: u8) -> f32 {
        if rank >= 2 {
            self.radius2
        } else {
            self.radius
        }
    }

    /// `rank`단의 터지는 울림의 피해.
    pub fn toll_at(&self, rank: u8) -> i32 {
        if rank >= ULT_MAX_RANK {
            self.toll3
        } else {
            self.toll
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tuning::ULT_SLAM_DAMAGE;

    /// 0158 다: 큰 기술은 세 단 — 2단 휘청 +0.6초, 3단 벽에 박힌 피해 ×1.5(20 → 30). 0172: 처음부터 주지 않는다(배운다).
    #[test]
    fn the_great_skill_has_three_ranks() {
        let upend = Entry::Ult(Ult::Upend);
        assert_eq!((upend.free(), upend.max()), (0, 3));
        assert_eq!(UPEND.stagger_at(1), UPEND.stagger);
        assert!((UPEND.stagger_at(2) - UPEND.stagger - 0.6).abs() < 1e-6);
        assert_eq!(UPEND.stagger_at(3), UPEND.stagger_at(2));
        assert_eq!(
            [1, 2, 3].map(|r| UPEND.slam_at(r)),
            [ULT_SLAM_DAMAGE, ULT_SLAM_DAMAGE, 30]
        );
        assert_eq!(ULT_SLAM_DAMAGE, 20);
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
}
