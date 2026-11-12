//! 전투 스킬 표(0150 제안 → 0151 결정, S0). 스킬 = 방랑자가 되찾은 **몸의 기술**(무기와 상관없다 — 무기의 스페셜은 RB, `weapons.rs`).
//! 줄 = (다시 쓰기까지·시동·후딜·모양·피해). 스킬은 기력을 쓰지 않고 저마다의 대기로 돈다 — 스킬이 아닌 것으로 적을 쓰러뜨리면
//! 모든 대기가 당겨진다("기세", `skill.momentum s`). 큰 기술은 대기가 아니라 게이지(평타·패링으로만 찬다).
//!
//! 코드에 스킬별 숫자를 박지 않는다 — 모양·수치는 이 표, 전체 배율·기세·게이지는 랩(`skill.*`).
//! 새 스킬 = `Id` 한 줄 + `SKILLS` 한 줄 + (새 모양이면 `Form` 한 갈래 + `world/skill_tick.rs`의 한 갈래) + 터치 그림 + 글 + 시험.

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

    /// 처음부터 끝까지(초).
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

    #[test]
    fn two_slots_never_hold_the_same_skill() {
        assert_eq!(sane([Id::Cut, Id::Quake]), [Id::Cut, Id::Quake]);
        assert_eq!(sane([Id::Cut, Id::Cut]), [Id::Cut, Id::Whirl]);
        assert_eq!(sane([Id::Whirl, Id::Whirl]), [Id::Whirl, Id::Cut]);
    }
}
