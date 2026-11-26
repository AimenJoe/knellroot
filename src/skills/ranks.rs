//! 스킬의 레벨(0158 다): 단의 숫자 `RANK_*` · 고르기 `Pick`/`Twist` · 맺음 `Cap` · 그 단의 실제 값 `Eff`(`Skill::at`).
use super::*;

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
    /// 제비걸음 ㄱ "한 곳만"(0214): 걸음마다 첫 적에게 — 하나를 거듭 벤다.
    Focus,
    /// 제비걸음 ㄴ "돌아오는 걸음"(0214): 다 벤 뒤 처음 자리로 돌아온다(한 걸음 더).
    HomeStep,
    /// 막아치기 ㄱ "넓은 팔"(0214): 부채꼴이 `arc_deg`°.
    Wide { arc_deg: f32 },
    /// 막아치기 ㄴ "드러난 틈"(0214): 끊긴 적은 `secs`초 동안 받는 피해 × `mult`. 빠지며 베기 ㄴ "벌어진 틈"(0274 ③): 베인 적이(보스는 아니다).
    Expose { secs: f32, mult: f32 },
    /// 뿌리 솟기 ㄱ "둘레로"(0214): 앞으로 줄지어 대신 몸 둘레를 돌며 솟는다(반지름 = 길이의 반 · 걸음 0.4).
    Circle,
    /// 뿌리 솟기 ㄴ "묶는 뿌리"(0214): 맞은 적이 `secs`초 묶인다 — 걷지 못한다(보스는 아니다).
    Bind { secs: f32 },
    /// 바늘땀 ㄱ "파고드는 걸음"(0270 C3): 찌르는 동안 `dist` m 내딛는다. 빠지며 베기 ㄱ "먼 걸음"(0274 ③): 물러나는 거리가 `dist` m.
    Lunge { dist: f32 },
    /// 바늘땀 ㄴ "촘촘한 땀"(0270 C3): `more`번 더 찌른다.
    Stitches { more: u8 },
    /// 방패 돌진 ㄱ "벽에 박기"(0270 C3): 밀려난 적이 벽·턱에 박히면 덤 피해 `damage`(보스는 밀리기만).
    Slam { damage: i32 },
    /// 종 치기 ㄱ "두 번 울림"(0270 C3): 울린 뒤 같은 사이를 두고 한 번 더 울린다.
    Twice,
    /// 종 치기 ㄴ "큰 종"(0270 C3): 울림의 둘레가 × `mult`.
    Loud { mult: f32 },
    /// 파문 ㄱ "밀어내는 물결"(0270 C3): 물결에 맞은 적이 바깥으로 `push` m 밀린다.
    Shove { push: f32 },
    /// 눈발 ㄱ "흐르는 눈발"(0270 C3): 눈발이 겨눈 쪽으로 초속 `speed` m 흘러간다.
    Flow { speed: f32 },
    /// 눈발·뿌리 덫 ㄴ "큰 눈송이"·"넓은 덫"(0270 C3): 둘레 × `mult`.
    Big { mult: f32 },
    /// 불길 ㄴ "타는 땅"(0270 C3): 불길이 지난 부채꼴의 가운데에 `secs`초 남는 불티(잔불 흩기와 같은 불 — 초마다).
    Scorch { secs: f32 },
    /// 뿌리 덫 ㄱ "두 알"(0270 C3): 씨를 둘 — 겨눈 자리의 양옆에.
    Twin,
    /// 재기 ㄱ "깊은 숨"(0274): 기력도 `stamina` 돈다.
    Breath { stamina: f32 },
    /// 재기 ㄴ "바닥에서"(0274): 체력이 `below`(최대의 몫) 아래면 × `mult` 돈다.
    Low { below: f32, mult: f32 },
}

/// 5단 맺음(0158 마).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cap {
    /// 휘돌이: 마지막 바퀴에 맞은 적이 `stagger`초 휘청인다(보스는 아니다).
    Lift { stagger: f32 },
    /// 가로지르기 · 빠지며 베기(0274 ③): 이 스킬로 쓰러뜨리면 대기의 `part`를 돌려받는다(한 번 쓸 때 한 번).
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
    /// 제비걸음 "마지막 걸음"(0214): 마지막 걸음이 둘레 `reach` m를 자세 피해 × `poise`로 벤다.
    Finale { reach: f32, poise: f32 },
    /// 막아치기 "무너뜨림"(0214): 끊긴 적은 자세가 곧바로 깨진다.
    Break,
    /// 뿌리 솟기 "마지막 가시"(0214): 끝의 뿌리가 둘레 `radius` m를 크게 한 번 더(피해 `damage`).
    Spire { radius: f32, damage: i32 },
    /// 바늘땀 "꿰매기"(0270 C3): 마지막 땀에 맞은 적이 `stagger`초 휘청인다(보스는 아니다).
    Pin { stagger: f32 },
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
    /// 맺음의 휘청(`Cap::Stun` — 방패 돌진 · 재기 · 생기 거두기 · 버팀)의 초, 그 맺음이 아니면 0(0278 R20 — 네 곳의 같은 match).
    pub fn cap_stagger(&self) -> f32 {
        match self.cap {
            Some(Cap::Stun { stagger }) => stagger,
            _ => 0.0,
        }
    }

    /// 판정(움직임)이 이어지는 시간(초) — 되돌아 베기는 길을 두 번 간다.
    pub fn active(&self) -> f32 {
        // 모양마다(와일드카드 없이 — 0278 R14). 두 번 울림은 판정이 아니라 놓인 종이라 몸의 시간은 그대로다.
        match self.skill.form {
            Form::Lane { time, .. } if self.twist == Some(Twist::Return) => time * 2.0,
            // 돌아오는 걸음: 한 걸음 더(처음 자리로).
            Form::Hop { gap, .. } if self.twist == Some(Twist::HomeStep) => {
                self.skill.active() + gap
            }
            Form::Lane { .. }
            | Form::Hop { .. }
            | Form::Ring { .. }
            | Form::Pull { .. }
            | Form::Cone { .. }
            | Form::Drift { .. }
            | Form::Trap { .. }
            | Form::Wave { .. }
            | Form::Brace { .. }
            | Form::Echo { .. }
            | Form::Throw { .. }
            | Form::Field { .. }
            | Form::Bash { .. }
            | Form::Stab { .. }
            | Form::Bell { .. }
            | Form::Rally { .. }
            | Form::Slip { .. }
            | Form::Reap { .. }
            | Form::Hex { .. }
            | Form::Erupt { .. } => self.skill.active(),
        }
    }

    pub fn total(&self) -> f32 {
        self.skill.startup + self.active() + self.skill.recover
    }
}

impl Skill {
    /// `rank`단(1~5로 맞춘다 — 끼운 스킬은 배운 스킬이다)·`pick` 고르기의 실제 값.
    pub fn at(&self, rank: u8, pick: Pick) -> Eff {
        self.at_grown(rank, pick, 1.0)
    }

    /// `at` + 제 치수(4단의 치수 덤과 같은 칸 — 반지름 · 길이 · 사거리 · 각)를 `grow`배 더 — 땅을 짚는 손(0274 ⑤ 낫의 빈 손)이
    /// 뿌리 스킬에 얹는다(`World::skill_eff`).
    pub fn at_grown(&self, rank: u8, pick: Pick, grow: f32) -> Eff {
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
        let size = if rank >= SIZE_RANK { RANK4_SIZE } else { 1.0 } * grow;
        // 피해 없는 스킬(끌어모으기)의 2단은 휘청 시간.
        let hold = if rank >= 2 && self.damage == 0 {
            RANK2_STAGGER
        } else {
            0.0
        };
        let form = self.form.grown(&Growth {
            rank,
            size,
            dmg,
            hold,
            twist,
            cap,
        });
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

/// 단 · 고르기 · 맺음이 모양의 치수에 얹는 것(`Skill::at_grown`이 셈해 `Form::grown`에 넘긴다).
struct Growth {
    rank: u8,
    /// 4단의 제 치수 × 땅을 짚는 손.
    size: f32,
    /// 단의 피해 배(2단 · 5단).
    dmg: f32,
    /// 피해 없는 스킬의 2단 휘청 덤(초).
    hold: f32,
    twist: Option<Twist>,
    cap: Option<Cap>,
}

impl Growth {
    /// 크게(`Twist::Big`)의 배, 아니면 1.
    fn big(&self) -> f32 {
        match self.twist {
            Some(Twist::Big { mult }) => mult,
            _ => 1.0,
        }
    }

    /// 넓게 휘두르기(`Twist::Wide` — 후려치기 ㄴ, 0270 C3)면 그 각, 아니면 `arc`.
    fn wide(&self, arc: f32) -> f32 {
        match self.twist {
            Some(Twist::Wide { arc_deg }) => arc_deg,
            _ => arc,
        }
    }

    /// 오래 타는 맺음(`Cap::Smolder`)의 덤 초.
    fn smolder(&self) -> f32 {
        match self.cap {
            Some(Cap::Smolder { secs, .. }) => secs,
            _ => 0.0,
        }
    }
}

impl Form {
    /// 이 모양에 단 · 고르기 · 맺음을 입힌 것(0278 R25b — 옛 `at_grown`의 모양마다 다시 짓던 match를 칸 고치기로: 값 · 셈의 차례는
    /// 그대로, 곱은 왼쪽부터). 모양마다 무엇이 크나는 `Form::info().grows`가 화면에 말한다.
    fn grown(self, g: &Growth) -> Form {
        let size = g.size;
        let mut f = self;
        match &mut f {
            Form::Ring { radius, hits, .. } => {
                *radius *= size;
                *hits += match g.twist {
                    Some(Twist::Spins { more }) => more,
                    _ => 0,
                };
            }
            Form::Lane { dist, .. } => *dist *= size,
            Form::Stab { range, hits, .. } => {
                *range *= size;
                *hits += match g.twist {
                    Some(Twist::Stitches { more }) => more,
                    _ => 0,
                };
            }
            Form::Bell { radius, .. } => {
                *radius = *radius
                    * size
                    * match g.twist {
                        Some(Twist::Loud { mult }) => mult,
                        _ => 1.0,
                    };
            }
            // 저주 새기기(0274 ⑥): 4단의 치수는 닿는 거리 · 옮는 둘레에, 멀리 옮는 저주(ㄱ)는 옮는 둘레에.
            Form::Hex { range, spread, .. } => {
                *range *= size;
                *spread = *spread * size * g.big();
            }
            Form::Pull { range, stagger, .. } => {
                *range *= size;
                *stagger += g.hold;
            }
            Form::Cone { arc_deg, .. } | Form::Bash { arc_deg, .. } => {
                *arc_deg = g.wide(*arc_deg) * size
            }
            Form::Wave { radius, .. } => *radius *= size,
            Form::Drift { radius, secs, .. } => {
                *radius = *radius * size * g.big();
                *secs += g.smolder();
            }
            Form::Trap { radius, .. } => *radius = *radius * size * g.big(),
            // 버팀: 받아낸 타의 덤도 피해의 배를 같이 받는다(10 → 12 → 13).
            Form::Brace { radius, more, .. } => {
                *radius *= size;
                *more = scaled(*more, g.dmg);
            }
            // 되울림의 2단은 이어지는 시간(피해도 휘청도 아니다).
            Form::Echo { secs, radius, .. } => {
                *secs += if g.rank >= 2 { RANK2_SECS } else { 0.0 };
                *radius *= size;
            }
            Form::Throw { dist, .. } => *dist *= size,
            Form::Field { radius, secs, .. } => {
                *radius *= size;
                *secs += g.smolder();
            }
            Form::Hop { range, .. } => *range *= size,
            // 생기 거두기(0274 ⑤): 거둔 적마다 도는 체력이 피해의 배를 같이 받는다(4 → 5 → 5), 둘레는 4단의 치수 · 넓은 낫(ㄱ).
            Form::Reap { radius, heal, .. } => {
                *radius = *radius * size * g.big();
                *heal = scaled(*heal, g.dmg);
            }
            // 재기: 도는 체력이 피해의 배를 같이 받는다(10 → 12 → 13), 둘레는 4단의 치수.
            Form::Rally { heal, radius, .. } => {
                *heal = scaled(*heal, g.dmg);
                *radius *= size;
            }
            // 빠지며 베기: 4단은 베는 거리, 먼 걸음(ㄱ)은 물러나는 거리.
            Form::Slip { range, back, .. } => {
                *range *= size;
                if let Some(Twist::Lunge { dist }) = g.twist {
                    *back = dist;
                }
            }
            Form::Erupt { len, .. } => *len *= size,
        }
        f
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
