//! 무기 표(0051). 무기 = 공격의 **모양**(KB `concepts/action-rpg-stats-and-equipment.md`): 콤보의 타마다 시간·거리·각·피해·자세 피해,
//! 강타, RB(근접 무기 = 종류의 원거리 수단 `ClassDef::toss` — 0274 ②b, 원거리 무기 = 사격), 원거리 무기의 X(수류탄). 0048: 무기 한 줄 = (X, RB) 짝.
//!
//! 시간·거리·각·내딛기는 랩의 기준값(`attack.*`·`heavy.*`, `tuning.rs`)에 곱하는 **배율** — 손맛의 바탕은 랩 한 곳에서 맞추고,
//! 무기는 그 위의 모양만 갖는다(코드에 타별 숫자를 박지 않는다). 피해·자세 피해는 정수·점수 그대로(피해의 눈금은 `tuning::HP_UNIT` — 옛 1 = 10, 0158 N0).
//! 새 무기 = `Id` 한 줄 + `WEAPONS` 한 줄(키 · 종류 · 이름 · 설명 · 나오는 곳 · 터치 그림 · 아이템 그림의 각까지 — 0278 R18 · R19) +
//! 동작(`tools/art/blender/wanderer.py`) + 모델 부위(같은 파일, 오른손 뼈) + 글(i18n).

use crate::i18n::Tr;
use crate::skills::Ult;
use crate::stats::Mods;
use crate::status::{Dose, Kind, NO_DOSE};
use crate::tuning::CAST_CHARGES;

/// 판정 모양.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    /// 앞쪽 부채꼴(각 = `arc`).
    Arc,
    /// 몸 둘레 원(회전 베기·내려찍기).
    Circle,
    /// 앞으로 곧은 좁은 길(창의 찌르기, 0272 2단계): 바라보는 쪽으로 닿는 거리까지, 반 폭 `half` m(+ 적의 몸) 안의 적을
    /// **모두** 꿰뚫는다 — 옆의 적에게는 닿지 않는다. 벽에 막힌 곳 너머로는 닿지 않는다(`World::lane_reach`).
    Lane { half: f32 },
}

impl Shape {
    /// 곧은 길이면 그 반 폭(m).
    pub fn lane_half(self) -> Option<f32> {
        match self {
            Shape::Lane { half } => Some(half),
            Shape::Arc | Shape::Circle => None,
        }
    }
}

/// 휘두르는 쪽(0087): 그 동작의 칼이 어느 쪽에서 어느 쪽으로 지나가나 — 박자 부채꼴(`World::beat_cue`)이 칼이 지나간 쪽부터 사라진다.
/// 오른쪽·왼쪽은 캐릭터 기준(모델 앞 = +Z, 오른쪽 = -X — `tools/art/blender/common.py`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sweep {
    /// 오른쪽에서 왼쪽으로 가로 베기 — 오른쪽부터 사라진다.
    RightToLeft,
    /// 왼쪽에서 오른쪽으로 — 왼쪽부터.
    LeftToRight,
    /// 찌르기·내려치기·올려 치기(세로) — 양 가장자리부터, 가운데가 마지막.
    Center,
}

/// 동작 이름 → 휘두르는 쪽과 그 동작을 만든 판정 구간(0193·0195).
/// 쪽: 판정 구간 동안 칼끝이 몸 둘레로 가는 쪽(스킨 몸의 Meshy 동작에서 잰 값). 60° 넘게 가로로 가면 그 쪽, 아니면 가운데
/// (찌르기·내려치기). 시험 `melee_swings_cross_the_front_in_the_table_direction`이 모델에서 다시 잰다 — 동작을 바꾸면 여기도.
/// 구간(시작 s, 길이 s): `tools/art/blender/wanderer_skin.py` `HIT_WINDOWS` 그대로(시험 `swing_windows_match_the_blender_table`) —
/// 동작 시계는 타의 판정 시작이 이 시작에 오게 돈다(`clip_rate` — 맨손처럼 남의 동작을 빌린 타·대시 공격도 칼이 닿는 순간에 맞는다).
/// 새 동작을 무기 표에 쓰면 여기에도 한 줄(시험 `every_weapon_motion_has_a_sweep`).
pub const SWEEPS: [(&str, Sweep, (f32, f32)); 44] = [
    ("attack", Sweep::RightToLeft, (0.08, 0.10)),
    ("attack2", Sweep::LeftToRight, (0.08, 0.10)),
    ("attack3", Sweep::Center, (0.10, 0.12)),
    ("heavy", Sweep::LeftToRight, (0.06, 0.12)),
    ("hammer1", Sweep::RightToLeft, (0.092, 0.115)),
    ("hammer2", Sweep::Center, (0.092, 0.115)),
    ("hammer3", Sweep::Center, (0.112, 0.14)),
    ("hammer_slam", Sweep::Center, (0.08, 0.16)),
    ("staff1", Sweep::RightToLeft, (0.06, 0.07)),
    ("staff2", Sweep::LeftToRight, (0.06, 0.07)),
    ("staff3", Sweep::RightToLeft, (0.06, 0.08)),
    ("staff4", Sweep::Center, (0.10, 0.13)),
    ("staff_release", Sweep::Center, (0.05, 0.11)),
    ("sword_dash", Sweep::RightToLeft, (0.08, 0.10)),
    // 맨손(0198): 잽 둘·어퍼컷·앞차기 — 모두 앞으로 내지른다.
    ("fist1", Sweep::Center, (0.048, 0.06)),
    ("fist2", Sweep::Center, (0.048, 0.06)),
    ("fist3", Sweep::Center, (0.064, 0.08)),
    ("fist_heavy", Sweep::Center, (0.048, 0.096)),
    ("fist_dash", Sweep::Center, (0.056, 0.07)),
    ("hammer_dash", Sweep::Center, (0.128, 0.16)),
    // 창(0272 2단계): 찌르기 셋·달려들며 찌르기·내달려 꿰뚫기 — 모두 앞으로 곧게.
    ("spear1", Sweep::Center, (0.092, 0.115)),
    ("spear2", Sweep::Center, (0.092, 0.115)),
    ("spear3", Sweep::Center, (0.108, 0.135)),
    ("spear_dash", Sweep::Center, (0.08, 0.1)),
    ("spear_lunge", Sweep::Center, (0.069, 0.138)),
    // 쌍단검(0274 ③): 왼손 찌르기 · 오른손 찌르기 · 오른쪽에서 가로 베기 · 올려 베기 · 달려들며 찌르기 · 꿰뚫고 지나가기.
    ("dagger1", Sweep::Center, (0.064, 0.08)),
    ("dagger2", Sweep::Center, (0.064, 0.08)),
    ("dagger3", Sweep::RightToLeft, (0.068, 0.085)),
    ("dagger4", Sweep::Center, (0.084, 0.105)),
    ("dagger_dash", Sweep::Center, (0.064, 0.08)),
    ("dagger_lunge", Sweep::Center, (0.048, 0.096)),
    // 대검(0274 ④): 오른쪽에서 휩쓸기 · 되받아 휩쓸기 · 다시 오른쪽에서 · 크게 되받아 · 머리 위에서 내려찍기 · 달려들며 올려 베기 ·
    // 강공격 = 두 손으로 모아 내려베기 — 랩 시간에 표의 배 1.3 · 1.3 · 1.3 · 1.4 · 1.6 · 1.1 · 강타 1.4.
    ("gs1", Sweep::RightToLeft, (0.104, 0.13)),
    ("gs2", Sweep::LeftToRight, (0.104, 0.13)),
    ("gs3", Sweep::RightToLeft, (0.104, 0.13)),
    ("gs4", Sweep::LeftToRight, (0.112, 0.14)),
    ("gs5", Sweep::Center, (0.128, 0.16)),
    ("gs_dash", Sweep::RightToLeft, (0.088, 0.11)),
    ("gs_heavy", Sweep::Center, (0.084, 0.168)),
    // 낫(0274 ⑤): 셋째 타 = 걸어 당기며 베기(M13 — 뻗어 걸고 몸으로 당긴다) — 랩 시간에 표의 배 1.3 · 강공격 = 검의 모아 베기(같은
    // 배 — 휜 날의 끝이 정면을 지나게 몸만 조금 더 돌린 제 동작). 앞의 두 타는 0283부터 제 동작(오른쪽에서 걸어 베기 · 되받아 걸기 —
    // 검 1 · 2타와 같은 구간), 대시도 0285부터 제 동작(내디디며 오른쪽에서 크게 걸어 베기 — 검 대시와 같은 구간 · 쪽).
    ("scythe1", Sweep::RightToLeft, (0.08, 0.10)),
    ("scythe2", Sweep::LeftToRight, (0.08, 0.10)),
    ("scythe3", Sweep::Center, (0.104, 0.13)),
    ("scythe_heavy", Sweep::Center, (0.06, 0.12)),
    ("scythe_dash", Sweep::RightToLeft, (0.08, 0.10)),
    // 완드(0274 ⑥)의 대시: 쌍단검 대시가 빌려 쓰던 내디디며 찌르기(Thrust_Slash)를 제 이름으로(0285 — 쌍단검 대시가 제 동작이 되었다).
    ("wand_dash", Sweep::Center, (0.064, 0.08)),
];

/// 동작의 휘두르는 쪽(표에 없으면 가운데).
pub fn sweep_of(anim: &str) -> Sweep {
    SWEEPS
        .iter()
        .find(|(a, _, _)| *a == anim)
        .map_or(Sweep::Center, |(_, s, _)| *s)
}

/// 동작 시계의 배(0195): 판정이 `s0`초에 시작하는 타에서 동작을 이만큼 빨리(1보다 크면) 돌리면 동작이 만든 판정 시작이 `s0`에 온다.
/// 표에 없는 동작은 1. 판정 구간의 길이도 같은 배로 맞는다(시작·길이 모두 랩 값 × 무기 표의 `time`).
pub fn clip_rate(anim: &str, s0: f32) -> f32 {
    SWEEPS
        .iter()
        .find(|(a, _, _)| *a == anim)
        .map_or(1.0, |(_, _, (c0, _))| if s0 > 1e-4 { c0 / s0 } else { 1.0 })
}

/// 평타의 동작 시각(0284): 타의 시각 `e`(초 — 몰아치기 · 분노의 배를 이미 곱한 것)에서 동작의 자리. 선딜 `s0`(들어 올림 `windup`까지
/// 곱한 것)까지는 동작이 만든 판정 시작에 닿게 — 들어 올림이 늘어난 타(`windup` > 1 — 대검)는 빨리 들어 올려 꼭대기에서 머무는
/// 꼴(ease-out)로, 그 뒤(휘두름 · 거두기)는 들어 올림을 늘이기 전의 빠르기로 돈다(무게는 들어 올림과 거둠에, 휘두름은 그대로 빠르다).
/// `windup` 1이면 옛 `e × clip_rate` 그대로.
pub fn clip_at(anim: &str, e: f32, s0: f32, windup: f32) -> f32 {
    let rate = clip_rate(anim, s0);
    if windup == 1.0 {
        return e * rate;
    }
    if e < s0 {
        let u = (e / s0.max(1e-4)).clamp(0.0, 1.0);
        s0 * rate * (1.0 - (1.0 - u) * (1.0 - u))
    } else {
        s0 * rate + (e - s0) * rate * windup
    }
}

/// 콤보 한 타. Debug는 0284 앞의 꼴에 바탕이 아닌 들어 올림만 덧붙인다(`weapon_fixture.txt` — 다른 무기의 줄은 그대로).
#[derive(Clone, Copy)]
pub struct Hit {
    /// 선딜·활성·후딜 모두에 곱한다(1 = 랩 평타 그대로, 클수록 느리다).
    pub time: f32,
    /// 거리·각·내딛기 배율.
    pub reach: f32,
    pub arc: f32,
    pub lunge: f32,
    pub shape: Shape,
    pub damage: i32,
    /// 자세 피해(0051 — 적의 자세 막대를 채운다).
    pub poise: f32,
    /// 넉백 배율.
    pub knock: f32,
    /// 동작 이름(`wanderer.glb`) — 없으면 `attack`.
    pub anim: &'static str,
    /// 캔슬 시점(0057): 활성 뒤 후딜레이의 이만큼(비율)이 지나야 다음 행동으로 끊는다 — 0 = 활성 끝나자마자. 무기 성격.
    pub cancel: f32,
    /// 쪼개기(0274 ④ — 대검, BG3 대검의 Cleave "앞의 셋까지"): 한 휘두르기가 맞히는 적의 수 상한(가까운 것부터). 0 = 상한 없음.
    pub cleave: u8,
    /// 걸어 당기기(0274 ⑤ — 낫, "휜 날이 끌어당기듯 벤다"): 맞은 적을 방랑자 쪽으로 이만큼(m) 끌어온다 — 몸 앞 `PULL_STOP` m에서
    /// 멈춘다(넉백 대신). 문지기는 끌리지 않는다(무겁다 — 끌어모으기와 같은 규칙). 0 = 넉백 그대로.
    pub pull: f32,
    /// 맞은 적에게 쌓는 속성 게이지(0274 ⑥ — 완드의 셋째 타: 저주를 새기는 찌르기 = 균열). 없으면 `NO_DOSE`.
    pub dose: Dose,
    /// 들어 올림(0284 — 대검, 사장 판정 p-gs "연타하면 그냥 빠른데"): 선딜에만 한 번 더 곱한다(1 = 랩의 선딜 · 활성 비 그대로). 무게는
    /// 들어 올림과 거둠에 — 휘두름(활성)은 `time` 그대로 빠르다. 동작은 `clip_at`이 들어 올림만 늘인다. 1보다 크면 맞힌 휘두르기마다
    /// `Event::Heft`(무거운 타의 손맛).
    pub windup: f32,
}

impl std::fmt::Debug for Hit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("Hit");
        d.field("time", &self.time)
            .field("reach", &self.reach)
            .field("arc", &self.arc)
            .field("lunge", &self.lunge)
            .field("shape", &self.shape)
            .field("damage", &self.damage)
            .field("poise", &self.poise)
            .field("knock", &self.knock)
            .field("anim", &self.anim)
            .field("cancel", &self.cancel)
            .field("cleave", &self.cleave)
            .field("pull", &self.pull)
            .field("dose", &self.dose);
        if self.windup != 1.0 {
            d.field("windup", &self.windup);
        }
        d.finish()
    }
}

/// 강타(공격 꾹, 0030).
#[derive(Clone, Copy, Debug)]
pub struct Heavy {
    /// 선딜·활성·후딜 배율, 다 모으는 시간 배율.
    pub time: f32,
    pub charge: f32,
    pub reach: f32,
    pub arc: f32,
    pub lunge: f32,
    pub shape: Shape,
    /// 덜 모았을 때 피해(다 모으면 + `heavy.full bonus`).
    pub damage: i32,
    pub poise: f32,
    /// 모으는 동안 걷기 배율(0047의 `heavy.charge walk x`에 곱한다).
    pub walk: f32,
    /// 친 자리에 금이 가고 이 초 뒤 둘레를 다시 친다(망치 여진, PoE2 지진 착상): (늦음 s, 반지름 m, 피해, 자세 피해).
    pub aftershock: Option<(f32, f32, i32, f32)>,
    /// 자세 막대가 이만큼(0..1) 넘게 찬 적은 바로 깬다(망치 — PoE2의 "깨지기 직전" 적 즉시 깨기).
    pub breaks_primed: Option<f32>,
    pub anim_charge: &'static str,
    pub anim_heavy: &'static str,
    /// 캔슬 시점(0057): 강타의 활성 뒤 후딜레이 비율.
    pub cancel: f32,
    /// 걸어 당기기(0274 ⑤ — 낫의 강공격): 맞은 적을 이만큼(m) 끌어온다(`Hit::pull`과 같은 규칙). 0 = 넉백 그대로.
    pub pull: f32,
    /// 맞은 적에게 쌓는 속성 게이지(0274 ⑥ — 완드의 강공격: 쥐었다 터뜨린 저주 = 균열 가득). 없으면 `NO_DOSE`.
    pub dose: Dose,
}

impl Heavy {
    /// 표의 줄이 `..Heavy::BASE`로 받는 바탕(0278 R18): 여진 · 깨기 · 끌기 · 속성은 그 무기만의 것이라 없는 것이 바탕이다.
    /// 시간 · 거리 · 피해 · 동작은 줄마다 적는다(바탕의 값은 배 1 · 피해 0 — 빠뜨리면 바로 보이게).
    pub const BASE: Heavy = Heavy {
        time: 1.0,
        charge: 1.0,
        reach: 1.0,
        arc: 1.0,
        lunge: 1.0,
        shape: Shape::Arc,
        damage: 0,
        poise: 0.0,
        walk: 1.0,
        aftershock: None,
        breaks_primed: None,
        anim_charge: "charge",
        anim_heavy: "heavy",
        cancel: 0.0,
        pull: 0.0,
        dose: NO_DOSE,
    };
}

/// RB.
#[derive(Clone, Copy, Debug)]
pub enum Rb {
    /// 근접 무기(과 맨손)의 RB(0048 → 0274 ②b): 종류의 원거리 수단(`ClassDef::toss` — 맨손은 `STONE_TOSS`). 칸 3(`CAST_CHARGES`)을
    /// 시간으로 채워 쓴다. 꼴 · 피해 · 속성은 종류의 칸에서(`Id::toss`).
    Toss,
    /// 원거리 무기의 사격: 탄창, 다 쓰면 `reload` 초 뒤 가득, 회피하면 한 발 장전(PoE2 착상). 칸은 쓰지 않는다.
    Shot {
        bolt: Bolt,
        mag: u8,
        reload: f32,
        /// `cast.startup`·`cast.time` 배율(쏘기는 마법보다 빠르다).
        time: f32,
        /// 모은 사격 한 발이 쓰는 볼트(0210 — 센 한 발은 탄창을 더 쓴다, PoE2의 "한 발짜리 탄창" 꼴).
        /// 이만큼 남아 있어야 모은다.
        heavy: u8,
        /// 모은 사격(0056)의 탄 — 쇠뇌 = 끝까지 당긴 무거운 볼트, 활 = 끝까지 당긴 화살.
        charged: Bolt,
        /// 손의 리듬(0274 ②b — 쇠뇌 = 장전을 다룬다 · 활 = 당기는 길이를 고른다).
        rhythm: Rhythm,
    },
}

/// 사격 무기의 손 리듬(0274 ②b "쇠뇌와 활 — 손의 리듬으로 가른다"). 버튼·조준은 같고 쥐었을 때의 박자가 다르다.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Rhythm {
    /// 쇠뇌(기술자 — 멈춰 서서 장전·설치): 빈 탄창은 `Shot::reload`초 걸려 가득 찬다(길다). 탄이 남아 있으면 쏘지 않고 멈춰 선 지
    /// `wait`초 뒤부터 `each`초마다 한 발씩 채운다(덧장전 — 비우기 전에 끊으면 빠르다). 장전하는 동안에도 수류탄은 던진다,
    /// 회피 장전은 한 발 그대로. 쥐면 한 발 쏜 뒤 모은 사격(0056 그대로).
    Crank { wait: f32, each: f32 },
    /// 활(레인저 — 움직이며 당겨 쏘기): 누르면 화살 하나를 메겨 당기고(`PState::CastCharge`) 떼면 쏜다 — 당긴 시간으로 단계:
    /// 톡 = 보통 화살 · `mid`초 넘게 = `mid_bolt`(더 빠르고 멀리, 자세 피해 큼) · 끝까지(`cast.charge s`) = `Shot::charged`
    /// (꿰뚫고 오금 — `Bolt::lame`). 당기는 동안 걷기는 `cast.walk × walk`. 탄창 대신 화살 묶음(작고 장전이 짧다).
    Draw { mid: f32, mid_bolt: Bolt, walk: f32 },
}

/// 종류의 원거리 수단(0274 ②b "RB를 종류마다" — BG3에서 클래스마다 기본 원거리 수단이 다르듯): 보통 한 번 · 쥐어 다 모은 것.
/// 꼴은 탄의 그림 번호가 가른다(`Bolt::look` — 마법은 땅을 따라 흐르고, 던지는 것은 곧게 날고, 빛은 겨눈 자리에 내려 꽂힌다).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Toss {
    pub bolt: Bolt,
    pub charged: Bolt,
    /// 한 번에 나가는 수와 이웃한 둘 사이의 각(°) — 좁은 부채꼴(0274 ③ 쌍단검의 투척 단검 셋). 나머지는 하나(0°).
    pub fan: u8,
    pub spread: f32,
    /// RB 칸의 수와 한 칸이 차는 시간의 배(랩 `cast.cooldown s`에 곱한다 — 0274 ⑥ · 0282: RB가 주된 공격인 완드(저주탄)와 지팡이(마법 탄)는
    /// 칸이 많고 빨리 찬다, 나머지는 `CAST_CHARGES` · 1). 두 손으로 엮는 지팡이(0276)의 칸 +1은 `World::cast_charges`가 얹는다.
    pub charges: f32,
    pub refill: f32,
}

impl Toss {
    /// 종류의 RB 칸이 `..Toss::BASE`로 받는 바탕(0278 R18): 하나씩(부채꼴 없음) · 칸 `CAST_CHARGES` · 차는 배 1 — 부채꼴은 쌍단검,
    /// 칸 · 차는 배는 완드 · 지팡이(0282)만 다르다. 탄 둘은 줄마다 적는다(바탕은 마법 탄).
    pub const BASE: Toss = Toss {
        bolt: MAGIC,
        charged: MAGIC,
        fan: 1,
        spread: 0.0,
        charges: CAST_CHARGES,
        refill: 1.0,
    };
}

/// 날아가는 탄의 모양.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bolt {
    /// `cast.speed`·`cast.range` 배율.
    pub speed: f32,
    pub range: f32,
    pub damage: i32,
    pub poise: f32,
    /// 적을 꿰뚫고 계속 간다.
    pub pierce: bool,
    /// 맞은 적이 이 초 휘청(0 = 없음).
    pub stagger: f32,
    /// 탄의 꼴(0278 R17 — 그림 · 이름 · 나는 법은 `Look::def`).
    pub look: Look,
    /// 크기 배율(그림·맞는 반지름, 모은 탄은 크다 — 0056). 신성한 빛은 빛기둥의 반지름 배(`tuning::SMITE_RADIUS`).
    pub scale: f32,
    /// 맞은 적에게 쌓는 속성 게이지(0274 ②b — 신성한 빛 = 울림). 없으면 `NO_DOSE`.
    pub dose: Dose,
    /// 오금(0274 ②b — 끝까지 당긴 화살): 맞은 적이 이 초 느리게 걷는다(`tuning::HAMSTRING_SLOW`, 문지기는 `ROOT_BOSS_SLOW`). 0 = 없음.
    pub lame: f32,
}

/// 원거리 무기의 X(0048: 근접 무기의 마법 자리를 수류탄이 맞바꾼다).
#[derive(Clone, Copy, Debug)]
pub struct Grenade {
    /// 도화선(초) — 땅을 튕기다 이 시간에 터진다. 쇠뇌 볼트에 맞으면 바로.
    pub fuse: f32,
    /// 한 번에 가질 수 있는 수, 하나가 다시 차는 초.
    pub count: u8,
    pub cooldown: f32,
    pub radius: f32,
    pub damage: i32,
    pub poise: f32,
    /// 떨구는 자리: 발 앞 이 거리(m) — 조준 쪽(스틱·커서), 없으면 바라보는 쪽. 멀리 던지지 않는다(0178 — 사장 "수류탄은 바로 앞에
    /// 떨어뜨리는 걸로", 예전 0051 자동 조준 7m · 0055 꾹 = 11m). 0218(사장 "사정거리가 조금만 더 길었으면"): 1.4 → 2.0 —
    /// 튕긴 뒤 터지는 자리는 2.2m → 약 3.1m(`World::grenade_lands`).
    pub drop: f32,
    /// 터진 자리의 적에게 쌓는 뿌리 게이지(0231 — 활의 뿌리 씨앗: 묶는다, `status::Kind::Root`). 0 = 화약(쇠뇌).
    pub root: f32,
}

/// 무기 이름표.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Id {
    Sword,
    Hammer,
    Staff,
    Crossbow,
    /// 맨손(0076, 사장 2026-11-07 "무기 장착하지 않았을 경우에는 맨손 전투") — 무기 칸을 비운 것. 짧고 빠른 주먹 3연타.
    Fists,
    /// 잔뿌리 활(0231, 사장 "쇠뇌 말고도 원거리 무기 넣어줘"): 빠른 화살·당겨 쏘는 한 발·뿌리 씨앗(묶기).
    Bow,
    /// 파수꾼의 창(0272 2단계 — 창 종류의 첫 무기): 버려진 마을 파수꾼의 창. 가장 멀리 닿고 한 줄을 꿰뚫는다.
    Spear,
    /// 빈집의 쌍단검(0274 ③ — 쌍단검 종류의 첫 무기): 두 손에 하나씩 쥐는 짧은 날 한 벌(아이템 하나). 빠르고, 등 뒤의 적에게 세다.
    Daggers,
    /// 가지치기 대검(0274 ④ — 대검 종류의 첫 무기): 잔뿌리의 정원을 돌보던 이가 뿌리를 쳐내던 긴 날. 멈추지 않고 넓게 휩쓴다.
    Greatsword,
    /// 정원지기의 낫(0274 ⑤ — 낫 종류의 첫 무기): 잔뿌리의 정원지기가 풀과 잔뿌리를 걷던 한 손의 휜 날. 걸어 당기며 벤다.
    Scythe,
    /// 잿빛 돌의 완드(0274 ⑥ — 완드 종류의 첫 무기): 끝에 잿빛 돌을 단 짧은 마법 막대. 짧게 치고, 저주탄이 주된 공격이다.
    Wand,
}

/// 표 순서(`WEAPONS`의 줄 차례에서 선다). 맨손은 옛 끝에(옛 기록의 무기 번호가 그대로 읽히게 — `app::legacy_weapon`), 그 뒤에 새 무기.
pub const IDS: [Id; WEAPONS.len()] = {
    let mut ids = [Id::Sword; WEAPONS.len()];
    let mut i = 0;
    while i < ids.len() {
        ids[i] = WEAPONS[i].id;
        i += 1;
    }
    ids
};

impl Id {
    /// 기록 키(0066 — 든 무기는 슬롯 기록에, 키는 바꾸지 않는다 — 무기 표의 줄 `Weapon::key`).
    pub fn key(self) -> &'static str {
        self.def().key
    }

    pub fn by_key(k: &str) -> Option<Id> {
        IDS.into_iter().find(|i| i.key() == k)
    }
}

pub struct Weapon {
    pub id: Id,
    /// 기록 키(0066 — 든 무기 · 가진 무기 · 벼린 단 · 아이템의 바탕 · 도감의 키가 쓴다, **바꾸지 않는다**).
    pub key: &'static str,
    /// 종류(0272 — 판 · 물리 속성 · 주특성은 종류에). 맨손은 없다(무기 칸을 비운 것).
    pub class: Option<Class>,
    /// 이름 · 설명(장비 탭 · 가게 · 도감 — 0051, i18n 표) · 도감의 나오는 곳(0191 — 상자 자리는 지역마다 `stage/<지역>.rs`의
    /// `chest_loot` 차례, 무기를 얻는 차례는 0274 ⑦).
    pub name: Tr,
    pub desc: Tr,
    pub place: Tr,
    /// 공격 버튼에 그릴 그림(0147 — 사장 "어차피 종류는 정해졌으니까": 든 무기마다 하나, `touch::attack_icon`). 탄의 꼴
    /// `LookDef::icon`(0278 R17)과 같은 꼴로 표에 둔다.
    pub icon: crate::touch::Kind,
    /// 아이템 그림의 각(도 — 돌기 · 숙이기 · 눕히기, 0068 `item_view::view`): 방랑자 모델의 오른손 부위(`node`)를 앞으로 세워
    /// (숙이기 −80°: 넓은 면이 카메라 쪽) 대각선으로 눕힌다 — 모델 공간에서 날 · 자루는 +z로 길고 넓은 면이 위(+y)를 본다.
    pub view: (f32, f32, f32),
    /// 아이템 그림을 짝으로(0277 — 같은 부위를 눕힌 각만 거울로 한 번 더).
    pub twin: bool,
    /// X 연타(근접 무기). 원거리 무기는 비어 있고 X가 수류탄.
    pub combo: &'static [Hit],
    /// 대시 공격(2026-11-06): 회피 끝무렵·직후에 공격 — 회피의 기세로 멀리 내딛는 한 타. 맞힌 뒤 콤보는 둘째 타부터.
    pub dash: Option<Hit>,
    pub heavy: Option<Heavy>,
    pub rb: Rb,
    pub grenade: Option<Grenade>,
    /// 방랑자 모델에서 이 무기 부위 이름(다른 무기 부위는 숨긴다).
    pub node: &'static str,
    /// 왼손에 쥐는 둘째 부위(0274 ③ — 쌍단검의 왼손 날: 제 뼈 `dagger_l`). 없으면 빈 글. 든 무기의 것만 보인다(`render::hide_unheld`).
    pub off_hand: &'static str,
    /// 공격 기력 배율(0117, 사장 "큰 무기는 더 많은 기력 — 높은 피해의 보정"): 평타·대시 공격 `stamina.attack`, 강타 `heavy.stamina`,
    /// 원거리 무기는 사격·모은 사격·공중 볼트에 곱한다. 맨손 < 지팡이 < 검 < 쇠뇌 < 망치
    /// (쇠뇌 0173: 0.7 → 0.9 → 0223 사장 "원거리는 기력을 더 많이 소모함": 1.2 — 20초 지속 17 → 14/초, 검의 여섯 할쯤).
    pub stamina: f32,
    /// 벼린 단(0180 — 1·2·3단에 쌓인 몫, `forge.rs`). 근접 = 타마다 피해(평타·대시·강타), 쇠뇌 = 탄창 먼저·볼트 피해는 뒤에
    /// (볼트는 10이라 +10이면 곱절 — 원거리는 근접의 여섯~일곱 할 밑을 지킨다, 0173). 맨손은 벼리지 않는다(`None`).
    pub forge: Option<[Forge; 3]>,
}

/// 벼린 한 단의 몫(그 단까지 쌓인 값).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Forge {
    /// 근접 타마다 피해 +.
    pub hit: i32,
    /// 볼트·모은 사격·수류탄 피해 +.
    pub shot: i32,
    /// 탄창 +.
    pub mag: i32,
}

/// 탄의 꼴(0278 R17 — 옛 `Bolt::look: u8`의 수마다 한 갈래): 그림 · 이름 · 터치 그림 · 나는 법이 꼴로 갈린다. 성질은
/// `Look::def`(`LookDef`) 한 곳, 소리(`app::cue`)·빛과 몸의 그림(`render::overlays`)은 그 자리의 와일드카드 없는 match.
/// 새 꼴 = 갈래 하나 + `ALL` + `code` + `def` 한 줄 + 컴파일러가 짚는 match(소리 · 그림 · 이펙트를 낼 꼴이면 `fx`).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Look {
    /// 마법 탄(지팡이 — 꿰뚫는 민트, 옛 2): 땅을 따라 흐른다.
    Magic,
    /// 쇠뇌 볼트(옛 3): 곧게 난다 · 벼린 몫을 받는다.
    Bolt,
    /// 활의 화살(0231, 옛 4).
    Arrow,
    /// 던지는 손도끼(한손검, 0274 ②b — 옛 5).
    Axe,
    /// 신성한 빛(망치 — 날지 않고 겨눈 자리에 내려 꽂힌다, 옛 6).
    Light,
    /// 던진 창(곧게 꿰뚫고 손에 돌아온다, 옛 7).
    Javelin,
    /// 돌(맨손, 옛 8).
    Stone,
    /// 투척 단검 셋(쌍단검 0274 ③, 옛 9).
    Knife,
    /// 던진 방패(0276 — 튕기고 돌아온다, 옛 10).
    Shield,
    /// 땅 내려찍기(대검 0274 ④ — 땅을 타고 달리는 충격파, 옛 11).
    Quake,
    /// 가시 씨앗(낫 0274 ⑤ — 신성한 빛과 같은 길로 떨어져 뿌리가 솟는다, 옛 12).
    Seed,
    /// 저주탄(완드 0274 ⑥ — 곧게 난다, 옛 13).
    Curse,
}

/// 탄이 가는 법(0278 R17 — `LookDef::flight`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flight {
    /// 땅을 따라 흐른다(마법 탄 · 땅 내려찍기 — 0173, `bolt_lands`).
    Ground,
    /// 곧게 난다(볼트 · 화살 · 던지는 것 · 저주탄 — 0218).
    Straight,
    /// 날지 않고 겨눈 자리에 내려 꽂힌다(신성한 빛 · 가시 씨앗 — `World::smite_spot`).
    Smite,
}

/// 꼴의 성질 한 줄(0278 R17).
#[derive(Clone, Copy)]
pub struct LookDef {
    pub flight: Flight,
    /// 원거리 무기의 사격(볼트 · 화살 — 0180 벼린 쇠뇌의 피해 몫을 받는다).
    pub shot: bool,
    /// 손으로 던지는 것(손도끼 · 창 · 돌 · 단검 · 방패 — 몸으로 그린다, 휘두름 바람 소리).
    pub thrown: bool,
    /// 멈춘 자리에서 손으로 돌아온다(던진 창 · 방패 0276).
    pub returns: bool,
    /// 맞힌 적에서 가까운 적으로 튕긴다(던진 방패 — `tuning::SHIELD_BOUNCES`).
    pub bounces: bool,
    /// 장비 탭 · 도감의 RB 이름. 사격(볼트 · 화살)은 RB 줄에 오지 않는다 — 옛 넘어감 그대로 마법탄의 이름.
    pub name: Tr,
    /// 터치 · HUD의 RB 그림. 사격은 옛 넘어감 그대로 별(`Cast`).
    pub icon: crate::touch::Kind,
}

impl Look {
    /// 꼴 모두(시험이 돈다).
    #[cfg(test)]
    pub const ALL: [Look; 12] = [
        Look::Magic,
        Look::Bolt,
        Look::Arrow,
        Look::Axe,
        Look::Light,
        Look::Javelin,
        Look::Stone,
        Look::Knife,
        Look::Shield,
        Look::Quake,
        Look::Seed,
        Look::Curse,
    ];

    /// 옛 수(`u8`) — 골든 트레이스 · 신호 표의 글(`Debug`)이 이것을 쓴다. 바꾸지 않는다.
    pub const fn code(self) -> u8 {
        match self {
            Look::Magic => 2,
            Look::Bolt => 3,
            Look::Arrow => 4,
            Look::Axe => 5,
            Look::Light => 6,
            Look::Javelin => 7,
            Look::Stone => 8,
            Look::Knife => 9,
            Look::Shield => 10,
            Look::Quake => 11,
            Look::Seed => 12,
            Look::Curse => 13,
        }
    }

    /// 꼴의 성질(와일드카드 없이 — 새 꼴은 여기서 컴파일이 깨진다).
    pub const fn def(self) -> LookDef {
        use crate::touch::Kind as K;
        const fn row(flight: Flight, name: Tr, icon: K) -> LookDef {
            LookDef {
                flight,
                shot: false,
                thrown: false,
                returns: false,
                bounces: false,
                name,
                icon,
            }
        }
        const fn throw(name: Tr, icon: K) -> LookDef {
            LookDef {
                thrown: true,
                ..row(Flight::Straight, name, icon)
            }
        }
        match self {
            Look::Magic => row(Flight::Ground, |s| s.rb_magic, K::Cast),
            Look::Bolt | Look::Arrow => LookDef {
                shot: true,
                ..row(Flight::Straight, |s| s.rb_magic, K::Cast)
            },
            Look::Axe => throw(|s| s.rb_axe, K::HandAxe),
            Look::Light => row(Flight::Smite, |s| s.rb_light, K::Smite),
            Look::Javelin => LookDef {
                returns: true,
                ..throw(|s| s.rb_javelin, K::Javelin)
            },
            Look::Stone => throw(|s| s.rb_stone, K::Pebble),
            Look::Knife => throw(|s| s.rb_knives, K::Knives),
            Look::Shield => LookDef {
                returns: true,
                bounces: true,
                ..throw(|s| s.rb_shield, K::ShieldToss)
            },
            Look::Quake => row(Flight::Ground, |s| s.rb_quake, K::Quake2),
            Look::Seed => row(Flight::Smite, |s| s.rb_seed, K::ThornSeed),
            Look::Curse => row(Flight::Straight, |s| s.rb_curse, K::CurseBolt),
        }
    }
}

/// `Debug` = 옛 수(`code`): 사건의 글(`Event::Cast { look: 5 }`)이 골든 트레이스의 해시와 신호 표(`cue_snapshot.txt`)에 그대로
/// 들어간다 — 꼴을 enum으로 바꿔도 그 글이 바뀌지 않게(0278 R17, 동작은 그대로).
impl std::fmt::Debug for Look {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}

/// 쇠뇌의 덧장전(0274 ②b): 쏘지 않고 멈춰 선 지 이 초 뒤부터, 한 발마다 이 초(빈 탄창의 장전은 표의 `reload`).
pub const XBOW_TOPUP_WAIT: f32 = 0.5;
pub const XBOW_TOPUP_EACH: f32 = 0.45;
/// 활의 당기기(0274 ②b): 반쯤 당긴 화살이 되는 초(끝까지는 랩 `cast.charge s`), 당기는 동안 걷기(`cast.walk`에 곱한다 — 쇠뇌가
/// 모으는 동안보다 조금 덜 느리다), 끝까지 당긴 화살의 오금(초).
pub const BOW_MID_DRAW: f32 = 0.3;
pub const BOW_DRAW_WALK: f32 = 1.2;
pub const BOW_LAME: f32 = 2.0;

/// 근접 무기의 벼린 단: 단마다 타의 피해 +5(반 눈금 — 0180: +10은 한 단에 문지기 싸움이 43초 → 18초(허수아비)라 반으로.
/// +5도 잔재(30)를 쓰러뜨리는 검의 대 수가 셋 → 둘로 준다, KB `weapon-upgrade-design`).
const MELEE_FORGE: [Forge; 3] = [
    Forge {
        hit: 5,
        shot: 0,
        mag: 0,
    },
    Forge {
        hit: 10,
        shot: 0,
        mag: 0,
    },
    Forge {
        hit: 15,
        shot: 0,
        mag: 0,
    },
];

impl Bolt {
    /// 원거리 무기의 사격인가(0180 — 벼린 쇠뇌의 피해 몫을 받는다. 근접 무기의 마법 탄은 아니다) — `LookDef::shot`.
    pub fn is_shot(&self) -> bool {
        self.look.def().shot
    }

    /// 손으로 던지는 것(0274 ②b — 손도끼 · 창 · 돌 · 단검 · 방패): 땅을 따라 흐르지 않고 곧게 난다 — `LookDef::thrown`.
    pub fn thrown(&self) -> bool {
        self.look.def().thrown
    }

    /// 곧게 나는 탄(볼트·화살·던지는 것·저주탄 0274 ⑥) — 마법 탄과 땅 내려찍기만 땅을 따라 흐른다(0173 → 0218).
    pub fn straight(&self) -> bool {
        self.look.def().flight == Flight::Straight
    }

    /// 멈춘 자리에서 손으로 돌아온다(던진 창 · 던진 방패 0276).
    pub fn returns(&self) -> bool {
        self.look.def().returns
    }

    /// 맞힌 적에서 가까운 적으로 튕긴다(던진 방패 0276 — `tuning::SHIELD_BOUNCES`번까지, 그 뒤 돌아온다).
    pub fn bounces(&self) -> bool {
        self.look.def().bounces
    }

    /// 날지 않고 겨눈 자리에 내려 꽂힌다(신성한 빛 — `World::smite_spot`) · 떨어져 솟는다(가시 씨앗 0274 ⑤ — 같은 길, 그림과 솟는 것만 다르다).
    pub fn smites(&self) -> bool {
        self.look.def().flight == Flight::Smite
    }
}

const fn hit(
    time: f32,
    reach: f32,
    arc: f32,
    lunge: f32,
    damage: i32,
    poise: f32,
    anim: &'static str,
) -> Hit {
    Hit {
        time,
        reach,
        arc,
        lunge,
        shape: Shape::Arc,
        damage,
        poise,
        knock: 1.0,
        anim,
        cancel: 0.0,
        cleave: 0,
        pull: 0.0,
        dose: NO_DOSE,
        windup: 1.0,
    }
}

/// 대시 공격의 타 번호(`PState::Attack::hit`) — 콤보 표 밖.
pub const DASH: u8 = u8::MAX;

impl Weapon {
    /// 타 번호 → 타(콤보 표 또는 대시 공격).
    pub fn hit(&self, i: u8) -> Option<&Hit> {
        if i == DASH {
            self.dash.as_ref()
        } else {
            self.combo.get(i as usize)
        }
    }

    /// 이 타 다음에 이을 타(마지막이면 None). 대시 공격은 첫 타 자리라 둘째 타로 잇는다.
    pub fn after(&self, i: u8) -> Option<u8> {
        let n = if i == DASH { 1 } else { i as usize + 1 };
        (n < self.combo.len()).then_some(n as u8)
    }
}

const MAGIC: Bolt = Bolt {
    speed: 1.0,
    range: 1.0,
    damage: 10,
    poise: 1.0,
    pierce: false,
    stagger: 0.0,
    look: Look::Magic,
    scale: 1.0,
    dose: NO_DOSE,
    lame: 0.0,
};

/// 지팡이(마법사)의 RB — 마법탄(0274 ②b 전의 지팡이 값 → 0282 지팡이의 **주된 공격**): 빠르고 멀리 땅을 따라 흘러 꿰뚫는다 — 피해 10.
/// 칸이 `STAFF_CHARGES`이고 한 칸이 빨리 찬다(`STAFF_REFILL` — 완드의 저주탄과 같은 꼴: 쏘는 것이 손의 대부분). 모으면 크고 멀리
/// 가며 자세 피해 · 잡몹 휘청이 크다 — 피해는 `STAFF_CHARGED`(옛 30: 칸이 빨리 차는 지팡이에서 모은 탄이 곱절 넘게 내면 마구 쏘기보다
/// 늘 세다 — 완드와 같은 까닭).
pub const MAGIC_TOSS: Toss = Toss {
    bolt: Bolt {
        speed: 1.3,
        range: 1.2,
        poise: STAFF_BOLT_POISE,
        pierce: true,
        look: Look::Magic,
        ..MAGIC
    },
    charged: Bolt {
        speed: 1.6,
        range: 1.7,
        damage: STAFF_CHARGED,
        poise: 3.0,
        pierce: true,
        stagger: 0.3,
        look: Look::Magic,
        scale: 1.5,
        ..MAGIC
    },
    charges: STAFF_CHARGES,
    refill: STAFF_REFILL,
    ..Toss::BASE
};
/// 지팡이의 RB 칸 수 · 한 칸이 차는 시간의 배(랩 `cast.cooldown s` 3초에 곱한다) · 모은 탄의 피해(0282).
pub const STAFF_CHARGES: f32 = 5.0;
pub const STAFF_REFILL: f32 = 0.38;
pub const STAFF_CHARGED: i32 = 10;
pub const STAFF_BOLT_POISE: f32 = 1.5;

/// 한손검(전사)의 RB — 던지는 손도끼: 짧고(7.2 m) 무겁다 — 피해 20 · 자세 피해 4(지팡이 마법 탄 1.5의 두 배 넘게 — 0282 전에는 1의 넷 배),
/// 맞은 잡몹이 잠깐 휘청.
/// 모으면 더 큰 도끼(30 · 자세 7). 꿰뚫지 않는다(첫 적에 박힌다).
pub const AXE_TOSS: Toss = Toss {
    bolt: Bolt {
        speed: 1.1,
        range: 0.6,
        damage: 20,
        poise: 4.0,
        stagger: 0.3,
        look: Look::Axe,
        ..MAGIC
    },
    charged: Bolt {
        speed: 1.2,
        range: 0.75,
        damage: 30,
        poise: 7.0,
        stagger: 0.6,
        look: Look::Axe,
        scale: 1.5,
        ..MAGIC
    },
    ..Toss::BASE
};

/// 망치(성직자)의 RB — 신성한 빛: 겨눈 자리(9 m 안, 줄 위의 첫 적 또는 탄이 멈추는 데)에 `SMITE_DELAY`초 뒤 빛줄기가 내려 꽂혀
/// 둘레(`SMITE_RADIUS` × `scale`)를 친다 — 피해 20 · 자세 3 · 울림 0.5(둘이면 울린다). 모으면 넓고(×1.5) 30 · 울림 가득.
pub const LIGHT_TOSS: Toss = Toss {
    bolt: Bolt {
        range: 0.75,
        damage: 20,
        poise: 3.0,
        look: Look::Light,
        dose: (Kind::Knell, 0.5),
        ..MAGIC
    },
    charged: Bolt {
        range: 0.85,
        damage: 30,
        poise: 5.0,
        look: Look::Light,
        scale: 1.5,
        dose: (Kind::Knell, 1.0),
        ..MAGIC
    },
    ..Toss::BASE
};

/// 창의 RB — 창 던지기: 곧고 멀리(15.6 m) 날아 줄 위의 적을 모두 꿰뚫고(20), 멈춘 자리에서 손으로 돌아온다(`JAVELIN_BACK` m/s —
/// 오는 길은 치지 않는다). 창이 없는 동안 RB는 못 쓰고, 평타를 누르면 창이 곧바로 손에 돌아온다(`World::recall_javelin`).
pub const JAVELIN_TOSS: Toss = Toss {
    bolt: Bolt {
        speed: 1.4,
        range: 1.3,
        damage: 20,
        poise: 2.0,
        pierce: true,
        look: Look::Javelin,
        ..MAGIC
    },
    charged: Bolt {
        speed: 1.6,
        range: 1.5,
        damage: 30,
        poise: 4.0,
        pierce: true,
        stagger: 0.5,
        look: Look::Javelin,
        scale: 1.3,
        ..MAGIC
    },
    ..Toss::BASE
};

/// 맨손의 RB — 돌 던지기: 약하다(10 · 자세 0.5, 8.4 m). 모으면 20.
pub const STONE_TOSS: Toss = Toss {
    bolt: Bolt {
        range: 0.7,
        poise: 0.5,
        look: Look::Stone,
        ..MAGIC
    },
    charged: Bolt {
        speed: 1.1,
        range: 0.8,
        damage: 20,
        poise: 2.0,
        stagger: 0.3,
        look: Look::Stone,
        scale: 1.4,
        ..MAGIC
    },
    ..Toss::BASE
};
/// 쌍단검(도적)의 RB — 투척 단검 셋(0274 ③): 좁은 부채꼴(이웃한 둘 사이 `KNIFE_SPREAD`°)로 곧게 난다 — 날마다 피해 10 · 날카로움 0.45
/// (셋이 다 박히면 덜 쌓이는 잔재(0.75)도 피를 흘린다). 멀수록 부채꼴이 벌어져 한 적에 하나만 박힌다 — 가까이서 셋. 모으면 날마다 20 · 출혈 더.
pub const KNIFE_TOSS: Toss = Toss {
    bolt: Bolt {
        speed: 1.5,
        range: 0.6,
        poise: 0.4,
        look: Look::Knife,
        dose: (Kind::Sharp, 0.45),
        ..MAGIC
    },
    charged: Bolt {
        speed: 1.7,
        range: 0.75,
        damage: 20,
        poise: 0.8,
        look: Look::Knife,
        scale: 1.3,
        dose: (Kind::Sharp, 0.6),
        ..MAGIC
    },
    fan: 3,
    spread: KNIFE_SPREAD,
    ..Toss::BASE
};
/// 투척 단검의 부채꼴 — 이웃한 두 날 사이의 각(°).
pub const KNIFE_SPREAD: f32 = 12.0;

/// 대검(광전사)의 RB — 땅 내려찍기(0274 ④): 칼을 땅에 내려찍어 앞으로 달리는 짧은 충격파(7.4 m — 손도끼만큼) — 날지 않고 땅을 따라
/// 흐른다(마법 탄의 길 `bolt_lands`), 길 위의 적을 모두 친다(10 · 자세 2 · 균열 0.35 — 셋이면 금이 간다). 모으면 더 멀리(9.4 m) · 20 · 균열 0.7 ·
/// 잡몹 0.4초 휘청. 근접 무기의 RB는 평타의 4~5할(시험 `every_rb_stays_under_seven_tenths_of_melee_at_a_safe_distance`).
pub const QUAKE_TOSS: Toss = Toss {
    bolt: Bolt {
        speed: 0.9,
        range: 0.62,
        poise: 2.0,
        pierce: true,
        look: Look::Quake,
        scale: 1.2,
        dose: (Kind::Rift, 0.35),
        ..MAGIC
    },
    charged: Bolt {
        speed: 1.0,
        range: 0.78,
        damage: 20,
        poise: 4.0,
        pierce: true,
        stagger: 0.4,
        look: Look::Quake,
        scale: 1.7,
        dose: (Kind::Rift, 0.7),
        ..MAGIC
    },
    ..Toss::BASE
};

/// 방패 던지기(0276 — 하데스의 방패, 한손 무기 + 방패의 RB): 곧게 9 m를 날아 첫 적을 치고(20 · 자세 3 · 울림 0.35) 가까운 적으로
/// 튕긴다(`tuning::SHIELD_BOUNCES` · `SHIELD_BOUNCE_R`), 다 튕기거나 더 갈 적이 없으면 손으로 돌아온다. 칸을 쓰지 않고 모으지 않는다 —
/// 값은 "돌아올 때까지 막지 못한다"(그동안 RB는 막히고, 평타를 누르면 곧바로 손에 돌아온다 `World::recall_javelin`). 종류의
/// RB 칸보다 앞선다(`World::rb_toss`).
pub const SHIELD_TOSS: Toss = Toss {
    bolt: Bolt {
        speed: 1.3,
        range: 0.75,
        damage: 20,
        poise: 3.0,
        look: Look::Shield,
        dose: (Kind::Knell, 0.35),
        ..MAGIC
    },
    charged: Bolt {
        speed: 1.3,
        range: 0.75,
        damage: 20,
        poise: 3.0,
        look: Look::Shield,
        dose: (Kind::Knell, 0.35),
        ..MAGIC
    },
    ..Toss::BASE
};

/// 낫(드루이드)의 RB — 가시 씨앗(0274 ⑤): 겨눈 자리(9 m 안, 줄 위의 첫 적 또는 탄이 멈추는 데 — 신성한 빛과 같은 셈)에 `SMITE_DELAY`초 뒤
/// 떨어져 둘레(`SMITE_RADIUS` × `scale` = 1.6 m)에 가시 뿌리가 솟는다 — 피해 10 · 자세 1.5 · 뿌리 0.5(둘이면 묶인다). 모으면 넓고(2.2 m)
/// 20 · 자세 3 · 뿌리 가득(한 번에 묶인다). 한 적에게는 신성한 빛의 반 — 묶어 두는 손이다(묶인 적은 주특성으로 되살아나는 몫이 된다).
pub const SEED_TOSS: Toss = Toss {
    bolt: Bolt {
        range: 0.75,
        damage: 10,
        poise: 1.5,
        look: Look::Seed,
        scale: 1.15,
        dose: (Kind::Root, 0.5),
        ..MAGIC
    },
    charged: Bolt {
        range: 0.85,
        damage: 20,
        poise: 3.0,
        look: Look::Seed,
        scale: 1.6,
        dose: (Kind::Root, 1.0),
        ..MAGIC
    },
    ..Toss::BASE
};

/// 완드(흑마법사)의 RB — 저주탄(0274 ⑥, BG3 흑마법사의 엘드리치 블라스트 결 — 완드의 **주된 공격**): 곧게 빨리(×1.5) 12 m를 날아 첫
/// 적에 닿는다 — 피해 10 · 자세 0.5 · 균열 0.34(셋이면 금이 간다 — 금 간 적은 주특성으로 더 다친다). 칸이 다섯이고 한 칸이 빨리 찬다
/// (`refill` — 다른 종류의 RB 칸 셋 · 3초에 하나). 모으면 피해는 그대로 10이되 크고(×1.4) 줄 위를 꿰뚫으며 균열을 가득 채운다(잡몹 0.3초
/// 휘청) — 칸이 빨리 차는 완드에서 모은 탄이 피해를 곱절로 내면 마구 쏘기보다 늘 세다(모의): 모으기는 피해가 아니라 "한 번에 금을 낸다".
/// 땅을 따라 흐르지 않는다(지팡이의 마법 탄과 가르는 것 — 곧고, 보통은 꿰뚫지 않는다). 초당 피해는 그 무기 평타의 7할 밑(시험
/// `every_rb_stays_under_seven_tenths_of_melee_at_a_safe_distance`).
pub const CURSE_TOSS: Toss = Toss {
    bolt: Bolt {
        speed: 1.5,
        range: 0.85,
        damage: 10,
        poise: 0.5,
        look: Look::Curse,
        scale: 0.9,
        dose: (Kind::Rift, 0.34),
        ..MAGIC
    },
    charged: Bolt {
        speed: 1.6,
        range: 0.95,
        damage: 10,
        poise: 1.5,
        pierce: true,
        stagger: 0.3,
        look: Look::Curse,
        scale: 1.4,
        dose: (Kind::Rift, 1.0),
        ..MAGIC
    },
    charges: WAND_CHARGES,
    refill: WAND_REFILL,
    ..Toss::BASE
};
/// 완드의 RB 칸 수와 한 칸이 차는 시간의 배(랩 `cast.cooldown s` 3초 × 0.5 = 1.5초에 하나).
pub const WAND_CHARGES: f32 = 5.0;
pub const WAND_REFILL: f32 = 0.38;

/// 순례자의 검 — 기준. 3연타(0201 — Meshy Double_Combo_Attack): 오른쪽 위 → 왼쪽 대각 베기 → 올려 베기(왼 → 오른) →
/// 오른쪽에서 내리치기(좁고 길게, 자세 피해 큼).
const SWORD_COMBO: [Hit; 3] = [
    hit(1.0, 1.0, 1.0, 1.0, 10, 1.0, "attack"),
    hit(1.0, 1.0, 1.0, 1.0, 10, 1.0, "attack2"),
    Hit {
        knock: 1.6,
        cancel: 0.15,
        ..hit(1.2, 1.25, 0.45, 1.6, 20, 2.0, "attack3")
    },
];

/// 종치기 망치(0274 — 한 손에 드는 종지기의 메): 한손의 박자 3연타 — 오른쪽에서 후려치고(M11 첫 타) 되받아 치고(둘째 타)
/// 머리 위에서 내리쳐 둘레의 땅을 울린다(셋째 — 둥근 판정, 자세 피해 큼). 옛 양손의 느린 2타(1.7 · 2.0배)를 걷었다.
const HAMMER_COMBO: [Hit; 3] = [
    Hit {
        knock: 1.3,
        cancel: 0.25,
        ..hit(1.15, 1.05, 1.1, 1.1, 10, 2.0, "hammer1")
    },
    Hit {
        knock: 1.3,
        cancel: 0.3,
        ..hit(1.15, 1.05, 1.1, 1.1, 10, 2.0, "hammer2")
    },
    Hit {
        shape: Shape::Circle,
        knock: 1.8,
        cancel: 0.45,
        ..hit(1.4, 1.0, 1.0, 1.2, 30, 4.0, "hammer3")
    },
];

/// 순례자 지팡이(0282 — 마법이 먼저: RB의 마법 탄이 주된 공격, 근접은 밀어내 자리를 여는 손): 수수한 3연타 — 자루로 올려 쳐
/// 밀고(1타) · 되받아 쳐 밀고(2타) · 머리 위에서 짧게 내려쳐 크게 밀어낸다(3타 — 옛 넷째의 두 손 둘레 내려치기는 걷었다). 피해는
/// 10 · 10 · 10(옛 10 · 10 · 10 · 20), 타마다 옛 것보다 느리고(×`STAFF_SWING`) 넉백이 크다 — 붙은 적을 떼어 놓고 쏜다.
pub const STAFF_SWING: f32 = 0.9;
const STAFF_COMBO: [Hit; 3] = [
    Hit {
        knock: 1.4,
        ..hit(STAFF_SWING, 1.1, 0.8, 0.6, 10, 0.6, "staff1")
    },
    Hit {
        knock: 1.4,
        ..hit(STAFF_SWING, 1.1, 0.8, 0.6, 10, 0.6, "staff2")
    },
    Hit {
        knock: 2.0,
        cancel: 0.1,
        ..hit(1.0, 1.15, 0.9, 0.6, 10, 1.0, "staff3")
    },
];

/// 맨손 — 짧고 빠른 주먹 3연타(가까이 붙어야 닿는다, 피해 10·자세 피해 작음), 셋째는 내디디며 한 방. 동작은 왼 잽 → 오른 잽 →
/// 오른 어퍼컷(0198 — 전에는 검의 동작을 빈손으로 빌렸다).
const FISTS_COMBO: [Hit; 3] = [
    hit(0.6, 0.7, 0.8, 0.6, 10, 0.5, "fist1"),
    hit(0.6, 0.7, 0.8, 0.6, 10, 0.5, "fist2"),
    Hit {
        knock: 1.3,
        cancel: 0.1,
        ..hit(0.8, 0.8, 0.6, 1.2, 10, 1.2, "fist3")
    },
];

/// 찌르기의 길 반 폭(m — 적의 몸 반지름이 더해진다): 평타 · 마지막 찌르기 · 대시 · 강타. 좁다 — 옆에 붙은 적에게는 닿지 않는다.
pub const SPEAR_HALF: f32 = 0.35;
pub const SPEAR_HALF_WIDE: f32 = 0.45;

/// 파수꾼의 창(0272 2단계) — 가장 멀리 닿는 근접 무기(검 1.8m · 망치 2.1m · 창 2.7m)이고, 찌르기는 좁은 길 위의 적을 **모두**
/// 꿰뚫는다. 빠르기는 검과 망치 사이(타마다 ×1.15), 피해는 검과 같은 10 · 10 · 20 — 한 적에게 붙어 치면 검보다 느려 덜 낸다.
/// 셋째는 크게 내디디며 길게 찌른다(넉백 큼). 넓은 휩쓸기가 없어 둘레에 붙은 떼에 약하다(그 답은 스킬 「장대 휩쓸기」).
const SPEAR_COMBO: [Hit; 3] = [
    Hit {
        shape: Shape::Lane { half: SPEAR_HALF },
        ..hit(1.15, 1.5, 0.25, 0.8, 10, 1.0, "spear1")
    },
    Hit {
        shape: Shape::Lane { half: SPEAR_HALF },
        ..hit(1.15, 1.5, 0.25, 0.8, 10, 1.0, "spear2")
    },
    Hit {
        shape: Shape::Lane {
            half: SPEAR_HALF_WIDE,
        },
        knock: 1.5,
        cancel: 0.2,
        ..hit(1.35, 1.55, 0.25, 2.2, 20, 2.0, "spear3")
    },
];

/// 빈집의 쌍단검(0274 ③ — 도적): 두 손의 짧은 날로 빠른 4연타 — 왼손 찌르기 → 오른손 찌르기 → 오른손 가로 베기 → 올려 베기.
/// 닿는 거리는 짧고(1.4 m 남짓 — 맨손보다 조금 길다) 자세 피해는 작다(타마다 0.5 — 검의 반: 보스의 자세를 깨는 무기가 아니다).
/// 대신 등 뒤 · 휘청이는 적에게는 주특성 「급소」(`Art::sneak`)가 피해를 키운다 — 혼자 있는 적, 옆을 내준 적에게 가장 세다.
const DAGGER_COMBO: [Hit; 4] = [
    hit(0.8, 0.8, 0.55, 0.7, 10, 0.5, "dagger1"),
    hit(0.8, 0.8, 0.55, 0.7, 10, 0.5, "dagger2"),
    hit(0.85, 0.85, 0.9, 0.8, 10, 0.5, "dagger3"),
    Hit {
        knock: 1.2,
        cancel: 0.1,
        ..hit(1.05, 0.9, 0.55, 1.2, 20, 1.0, "dagger4")
    },
];
/// 쌍단검 강공격의 곧은 길 반 폭(m — 적의 몸 반지름이 더해진다): 적을 꿰뚫고 지나 등 뒤에 선다.
pub const DAGGER_HALF: f32 = 0.4;

/// 가지치기 대검(0274 ④ — 광전사, "멈추지 않는 칼"): 넓은 휩쓸기 다섯이 끊김 없이 이어진다 — 오른쪽에서 · 되받아 · 다시 오른쪽에서 ·
/// 크게 되받아 · 머리 위에서 내려찍기. 한 타가 넓고(180° 안팎 · 2.3 m) 앞의 셋까지 벤다(`cleave` — 가까운 것부터), 한 적에게는 검과
/// 비슷하거나 조금 아래(타마다 ×1.3 — 느리다). 자세 피해는 타마다 1.5(검보다 조금 크다), 끝의 내려찍기는 좁고 무겁다.
pub const GREATSWORD_CLEAVE: u8 = 3;
/// 대검의 무게(0284 — 사장 판정 p-gs "연타하면 그냥 빠른데..?"): 들어 올림(선딜 × 이만큼 — 검 5틱 → 대검 10틱 남짓), 거둠(후딜의
/// 이 몫이 지나야 다음 타로 끊는다 — 마구 눌러도 거둠을 건너뛰지 못한다 · 넷째 · 끝 타는 조금 더). 들어 올림이 있는 타가 맞으면
/// `Event::Heft`(둔한 쿵 · 강타의 킥 · 발밑 먼지 — 히트스톱은 평타 그대로: 한 칸 길게 하면 붙은 떼에 대한 초당 피해가 낫 아래로
/// 떨어졌다, 0284 결정 노트).
pub const GREATSWORD_WINDUP: f32 = 1.7;
pub const GREATSWORD_RECOVER: f32 = 0.4;
const GREATSWORD_COMBO: [Hit; 5] = [
    Hit {
        cleave: GREATSWORD_CLEAVE,
        cancel: GREATSWORD_RECOVER,
        windup: GREATSWORD_WINDUP,
        ..hit(1.3, 1.3, 1.5, 0.8, 10, 1.0, "gs1")
    },
    Hit {
        cleave: GREATSWORD_CLEAVE,
        cancel: GREATSWORD_RECOVER,
        windup: GREATSWORD_WINDUP,
        ..hit(1.3, 1.3, 1.5, 0.8, 10, 1.0, "gs2")
    },
    Hit {
        cleave: GREATSWORD_CLEAVE,
        cancel: GREATSWORD_RECOVER,
        windup: GREATSWORD_WINDUP,
        ..hit(1.3, 1.3, 1.5, 0.8, 10, 1.0, "gs3")
    },
    Hit {
        cleave: GREATSWORD_CLEAVE,
        knock: 1.4,
        cancel: GREATSWORD_RECOVER + 0.05,
        windup: GREATSWORD_WINDUP,
        ..hit(1.4, 1.35, 1.6, 1.0, 20, 2.0, "gs4")
    },
    Hit {
        cleave: GREATSWORD_CLEAVE,
        knock: 1.7,
        cancel: GREATSWORD_RECOVER + 0.1,
        windup: GREATSWORD_WINDUP,
        ..hit(1.6, 1.4, 0.6, 1.4, 20, 3.0, "gs5")
    },
];

/// 정원지기의 낫(0274 ⑤ — 드루이드, "휜 날이 끌어당기듯 벤다"): 한 손의 휜 날로 3연타 — 오른쪽에서 걸어 베고(`scythe1` — 0283
/// 전엔 검 1타의 동작), 되받아 걸고(`scythe2` — 검 2타), 셋째는 멀리 뻗어 걸어 **당기며** 벤다(앞의 적을 몸 앞으로 끌어온다 — 넉백 대신 `pull`). 한 타는 검과
/// 같은 10 · 10 · 20, 앞의 둘은 조금 넓고(×1.15) 셋째는 넓고 길게(1.4 · 1.3) 느리다(×1.3) — 한 적에게는 검과 같거나 조금 아래, 붙은
/// 떼에는 셋째가 다 걸어 검의 1.6배쯤(대검 · 망치 아래). 자세 피해는 검과 같다(자세를 부수는 무기가 아니다 — 묶고 되살아나는 손).
const SCYTHE_COMBO: [Hit; 3] = [
    hit(1.0, 1.05, 1.15, 1.0, 10, 1.0, "scythe1"),
    hit(1.0, 1.05, 1.15, 1.0, 10, 1.0, "scythe2"),
    Hit {
        knock: 0.0,
        cancel: 0.2,
        pull: SCYTHE_PULL,
        ..hit(1.3, 1.3, 1.4, 0.8, 20, 2.0, "scythe3")
    },
];
/// 낫의 셋째 타 · 강공격이 끌어오는 거리(m)와 끌려온 적이 멈추는 자리(방랑자의 몸 + 적의 몸 밖으로 m).
pub const SCYTHE_PULL: f32 = 1.2;

/// 잿빛 돌의 완드(0274 ⑥ — 흑마법사, "완드로 짧게 치고 저주탄이 주된 공격"): 짧고 빠른 3연타 — 끝의 잿빛 돌로 오른쪽에서 쳐 내고
/// (검 1타의 동작), 되받아 치고(검 2타), 셋째는 돌끝으로 짧게 찔러 **저주를 새긴다**(쌍단검의 오른 찌르기 — 균열 0.34, 저주탄 하나만큼).
/// 닿는 거리는 검의 8할 남짓(1.4~1.5 m)이고 피해는 10 · 10 · 10(검의 끝 타 20이 없다) — 붙어 치면 검보다 덜 낸다. 자세 피해는 작다.
const WAND_COMBO: [Hit; 3] = [
    hit(1.05, 0.8, 0.9, 0.7, 10, 0.5, "attack"),
    hit(1.05, 0.8, 0.9, 0.7, 10, 0.5, "attack2"),
    Hit {
        knock: 0.8,
        cancel: 0.15,
        dose: (Kind::Rift, 0.34),
        ..hit(1.05, 0.85, 0.55, 0.9, 10, 1.0, "dagger2")
    },
];
pub const SCYTHE_HEAVY_PULL: f32 = 1.6;
pub const PULL_STOP: f32 = 0.5;
/// 걸어 당긴 적과 방랑자를 잇는 덩굴(0277 — 렌더 `pull_vines`)이 보이는 초: 끌기는 넉백과 같은 길이라(`world::KNOCK_DECAY` 10/s) 0.3초면
/// 미끄러짐의 95%가 끝난다.
pub const PULL_SHOW: f32 = 0.3;

pub const WEAPONS: &[Weapon] = &[
    Weapon {
        id: Id::Sword,
        key: "sword",
        class: Some(Class::Sword),
        name: |s| s.weapon_sword,
        desc: |s| s.weapon_sword_d,
        place: |s| s.cx_weapon_sword,
        icon: crate::touch::Kind::Attack,
        view: (0.0, -80.0, -45.0),
        twin: false,
        combo: &SWORD_COMBO,
        // 달려들며 올려 베기(0206 — Charged_Upward_Slash, 예전엔 1타를 빌렸다 0201): 판정은 좁고 길게, 회피 거리만큼 더 내딛는다.
        dash: Some(Hit {
            knock: 1.4,
            cancel: 0.2,
            ..hit(1.0, 1.25, 0.5, 2.4, 20, 2.0, "sword_dash")
        }),
        heavy: Some(Heavy {
            time: 1.0,
            charge: 1.0,
            reach: 1.0,
            arc: 1.0,
            lunge: 1.0,
            shape: Shape::Arc,
            damage: 20,
            poise: 3.0,
            walk: 1.0,
            anim_charge: "charge",
            anim_heavy: "heavy",
            cancel: 0.2,
            ..Heavy::BASE
        }),
        rb: Rb::Toss,
        grenade: None,
        node: "sword",
        off_hand: "",
        stamina: 1.0,
        forge: Some(MELEE_FORGE),
    },
    Weapon {
        id: Id::Hammer,
        key: "hammer",
        class: Some(Class::Hammer),
        name: |s| s.weapon_hammer,
        desc: |s| s.weapon_hammer_d,
        place: |s| s.cx_weapon_hammer,
        icon: crate::touch::Kind::Hammer,
        view: (0.0, -75.0, -40.0),
        twin: false,
        combo: &HAMMER_COMBO,
        // 달려들며 몸 전체로 내려치기(0203 — 0200의 앞차기는 발이 작아 안 보였다, 사장): 웅크리며 땅을 쳐 둘레를 친다(앞차기 때 앞
        // 부채꼴로 바꾼 것을 되돌림). 0200 전엔 둘째 타의 한 바퀴 내려치기를 빌려 달리기 공격 뒤 같은 내려치기를 두 번 했다.
        dash: Some(Hit {
            shape: Shape::Circle,
            knock: 1.8,
            cancel: 0.45,
            ..hit(1.6, 1.0, 1.0, 2.0, 20, 3.0, "hammer_dash")
        }),
        // 강공격(0274): 머리 위로 들어 앞을 내리친다 — 옛 뛰어 내려찍기(내딛기 2.2 · 둘레)를 걷고 앞 부채꼴 한 걸음, 여진(땅의 울림)은
        // 그대로 둘레를 다시 친다.
        heavy: Some(Heavy {
            time: 1.3,
            charge: 1.3,
            reach: 1.05,
            arc: 0.8,
            lunge: 1.0,
            shape: Shape::Arc,
            damage: 30,
            poise: 6.0,
            walk: 0.85,
            aftershock: Some((1.5, 2.6, 20, 3.0)),
            breaks_primed: Some(0.6),
            anim_charge: "hammer_charge",
            anim_heavy: "hammer_slam",
            cancel: 0.55,
            ..Heavy::BASE
        }),
        rb: Rb::Toss,
        grenade: None,
        node: "w_hammer",
        off_hand: "",
        // 0274: 한 손에 드는 메라 양손일 때(1.7)보다 가볍다 — 검(1.0)과 창(1.25) 위.
        stamina: 1.4,
        forge: Some(MELEE_FORGE),
    },
    Weapon {
        id: Id::Staff,
        key: "staff",
        class: Some(Class::Staff),
        name: |s| s.weapon_staff,
        desc: |s| s.weapon_staff_d,
        place: |s| s.cx_weapon_staff,
        icon: crate::touch::Kind::Staff,
        view: (0.0, -80.0, -45.0),
        twin: false,
        combo: &STAFF_COMBO,
        // 달려들며 올려 베기(0203 — 1타 동작이라 다음 누름의 2타로 원본 그대로 이어진다): 빠르고 가볍다.
        dash: Some(hit(0.8, 1.3, 0.6, 2.2, 10, 1.0, "staff1")),
        // 강공격(0282 — 밀어내 자리를 연다): 쥐었다 둘레로 터뜨려 붙은 적을 떼어 낸다 — 피해는 옛 20에서 10.
        heavy: Some(Heavy {
            time: 0.9,
            charge: 0.9,
            reach: 1.0,
            arc: 1.0,
            lunge: 0.5,
            shape: Shape::Circle,
            damage: 10,
            poise: 2.5,
            walk: 1.2,
            anim_charge: "staff_charge",
            anim_heavy: "staff_release",
            cancel: 0.0,
            ..Heavy::BASE
        }),
        rb: Rb::Toss,
        grenade: None,
        node: "w_staff",
        off_hand: "",
        stamina: 0.75,
        forge: Some(MELEE_FORGE),
    },
    Weapon {
        id: Id::Crossbow,
        key: "crossbow",
        class: Some(Class::Crossbow),
        name: |s| s.weapon_crossbow,
        desc: |s| s.weapon_crossbow_d,
        place: |s| s.cx_weapon_crossbow,
        // 쇠뇌의 평타는 사격(0223 — 수류탄은 특수, `touch::cast_icon`).
        icon: crate::touch::Kind::Crossbow,
        // 아이템 그림: 쇠뇌는 T자(개머리 세로 · 활 가로)가 읽히게 눕히지 않는다.
        view: (0.0, -70.0, 0.0),
        twin: false,
        combo: &[],
        dash: None,
        heavy: None,
        // 0173(사장 "쇠뇌는 전투도 편한데 데미지도 강해서 오버파워"): 잰 값 — 8m 밖에서 처음 4초 60~65/초(근접 무기 30~37/초의 두 배),
        // 오래 쏴도 35/초(검 25). 안전한 거리의 값은 피해로 치른다: 볼트 20 → 10(마법 탄과 같다), 한 발 0.24 → 0.36초,
        // 장전 1.2 → 1.5초, 기력 0.7 → 0.9. 고친 뒤 처음 4초 약 23/초 · 오래 15/초(근접의 여섯~일곱 할).
        // 0210(사장 "시간당 공격력이 너무 좋음"): 서서 쏘기만 쟀던 0173이 놓친 길 — 쥐고 있으면 모은 사격이 곧장 이어져
        // 볼트 하나에 50(딱 맞춤)·꿰뚫기, 장전이 기력의 쉼이 되어 오래 쏴도 35/초(검의 가장 센 꼴 23). 모은 사격은 볼트 둘(`heavy`)·
        // 피해 40 → 30, 장전 1.5 → 2.0초. 고친 뒤 모아 쏘기 18/초 · 마구 17 · 박자 14.5(오래, 근접 가장 센 꼴 23~26의 일곱 할 안팎).
        // 0243(사장 설문 k09 "아직도 쎄긴함"): 피해는 눈금(10)이라 그대로 두고 한 발 0.9 → 1.15·장전 2.0 → 2.4 —
        // 오래 쏴도 검의 반 안팎, 처음 4초도 검의 여섯 할 밑.
        // 0274 ②b(기술자 — 장전을 다룬다): 빈 탄창의 장전 2.4 → 2.8초, 대신 탄이 남았을 때 멈춰 서면 0.5초 뒤부터 0.45초마다 한 발
        // (덧장전 — 넷을 다 채워도 1.85초). 마구 쏘기만 하면 조금 느려지고, 비우기 전에 끊어 채우는 손이 빠르다.
        rb: Rb::Shot {
            bolt: Bolt {
                speed: 1.7,
                range: 1.4,
                damage: 10,
                poise: 1.0,
                pierce: false,
                stagger: 0.0,
                look: Look::Bolt,
                scale: 1.0,
                ..MAGIC
            },
            mag: 4,
            reload: 2.8,
            time: 1.15,
            heavy: 2,
            // 모은 사격: 시위를 끝까지 당긴 무거운 볼트 — 꿰뚫고 밀어낸다(수류탄도 맞히면 터진다).
            // 0243(사장 설문 k09): 30 → 20 — 모아 쏘기만 되풀이하면 오래 17.5/초(검의 가장 센 꼴 23의 여덟 할 가까이)였다.
            charged: Bolt {
                speed: 2.2,
                range: 1.8,
                damage: 20,
                poise: 6.0,
                pierce: true,
                stagger: 0.4,
                look: Look::Bolt,
                scale: 1.6,
                ..MAGIC
            },
            rhythm: Rhythm::Crank {
                wait: XBOW_TOPUP_WAIT,
                each: XBOW_TOPUP_EACH,
            },
        },
        grenade: Some(Grenade {
            fuse: 1.5,
            count: 2,
            cooldown: 4.0,
            radius: 2.2,
            damage: 30,
            poise: 5.0,
            drop: 2.0,
            root: 0.0,
        }),
        node: "w_crossbow",
        off_hand: "",
        stamina: 1.2,
        forge: Some([
            Forge {
                hit: 0,
                shot: 0,
                mag: 1,
            },
            Forge {
                hit: 0,
                shot: 5,
                mag: 1,
            },
            Forge {
                hit: 0,
                shot: 10,
                mag: 2,
            },
        ]),
    },
    Weapon {
        id: Id::Fists,
        key: "fists",
        // 0286: 맨손 = 몽크.
        class: Some(Class::Monk),
        name: |s| s.weapon_fists,
        desc: |s| s.weapon_fists_d,
        place: |s| s.cx_weapon_fists,
        icon: crate::touch::Kind::Fist,
        // 맨손은 그림이 없다(부위가 비어 `item_view::has_view` 거짓 — 장비 창이 빈 칸처럼 둔다).
        view: (0.0, 0.0, 0.0),
        twin: false,
        combo: &FISTS_COMBO,
        // 달려들며 앞차기(0199).
        dash: Some(hit(0.7, 0.8, 0.6, 2.0, 10, 1.0, "fist_dash")),
        // 모아 치기: 검보다 짧고 빠르다, 피해 10(다 모으면 + `heavy.full bonus`).
        heavy: Some(Heavy {
            time: 0.8,
            charge: 0.8,
            reach: 0.7,
            arc: 0.8,
            lunge: 1.2,
            shape: Shape::Arc,
            damage: 10,
            poise: 2.0,
            walk: 1.2,
            anim_charge: "fist_charge",
            anim_heavy: "fist_heavy",
            cancel: 0.1,
            ..Heavy::BASE
        }),
        // 몽크의 RB = 돌(0274 ②b `STONE_TOSS` — 0286부터 종류 표의 칸).
        rb: Rb::Toss,
        grenade: None,
        // 든 것이 없다(다른 무기 부위는 모두 숨는다).
        node: "",
        off_hand: "",
        // 0286: 맨손도 벼린다(손에 감는 천을 숫돌로 다진다 — 다른 근접 무기와 같은 단, 몽크가 정원 · 작업층에서 잡몹을 쓰러뜨리는 대 수의
        // 폭 안에 남게 — `tuning::tiers_keep_the_hits_to_kill_at_arrival`).
        stamina: 0.6,
        forge: Some(MELEE_FORGE),
    },
    // 잔뿌리 활(0231): 쇠뇌와 같은 틀(평타 = 사격, 꾹 = 모은 사격, 특수 = 던지는 것)에서 손맛을 가른다 —
    // 화살은 가볍고 빠르고 멀리(전통 가득 6 · 다시 채우기 짧음 · 한 발 빠름), 당겨 쏘는 한 발은 화살 하나로(쇠뇌는 둘) 꿰뚫고,
    // 특수는 터지는 화약이 아니라 **뿌리 씨앗** — 피해는 작고 둘레의 적을 묶는다(뿌리 게이지 가득). 쇠뇌 = 한 방·떼 지우기,
    // 활 = 거리·잦은 손·묶어 두고 쏘기. 수치는 처음 값 — 원거리 초당 피해는 근접의 여섯~일곱 할 밑(0173, 시험).
    Weapon {
        id: Id::Bow,
        key: "bow",
        class: Some(Class::Bow),
        name: |s| s.weapon_bow,
        desc: |s| s.weapon_bow_d,
        place: |s| s.cx_weapon_bow,
        icon: crate::touch::Kind::Bow,
        // 아이템 그림(0231): 활대가 세로로 길다 — 옆으로 눕혀 활대의 휨과 시위가 보이게.
        view: (0.0, -80.0, -90.0),
        twin: false,
        combo: &[],
        dash: None,
        heavy: None,
        rb: Rb::Shot {
            // 0243(사장 설문 k09): 처음 4초가 검과 같았다(30/초) — 전통 6 → 5, 한 발 0.95 → 1.3(피해는 눈금이라 그대로).
            bolt: Bolt {
                speed: 1.5,
                range: 1.7,
                damage: 10,
                poise: 0.8,
                pierce: false,
                stagger: 0.0,
                look: Look::Arrow,
                scale: 1.0,
                ..MAGIC
            },
            // 0274 ②b(레인저 — 화살 묶음): 전통 5 · 다시 채우기 2.2초 → 묶음 3 · 1.0초(짧고 잦다).
            mag: 3,
            reload: 1.0,
            time: 1.3,
            heavy: 1,
            // 끝까지 당긴 화살: 빠르고 멀리, 줄 선 적을 꿰뚫고 휘청이게 하고 오금을 쏜다(0274 ②b — 맞은 적이 `BOW_LAME`초 느리게).
            // 피해는 화살 그대로(10) — 화살 하나로 당기니 모아 쏘기만 되풀이하면 20이던 처음 값이 24/초(검의 가장 센 꼴 23의 여덟 할
            // 18.4를 넘었다, 탐침 `plan_dps_probe`). 딱 맞춰 떼면 덤 +10은 그대로라 "당겨서 맞히는" 맛은 남는다.
            charged: Bolt {
                speed: 2.4,
                range: 2.1,
                damage: 10,
                poise: 3.0,
                pierce: true,
                stagger: 0.35,
                look: Look::Arrow,
                scale: 1.4,
                lame: BOW_LAME,
                ..MAGIC
            },
            // 반쯤 당긴 화살(0274 ②b): 보통 화살보다 빠르고 멀리(20 m), 자세 피해 둘 배 — 꿰뚫지는 않는다.
            rhythm: Rhythm::Draw {
                mid: BOW_MID_DRAW,
                mid_bolt: Bolt {
                    speed: 2.0,
                    range: 1.7,
                    damage: 10,
                    poise: 1.6,
                    pierce: false,
                    stagger: 0.0,
                    look: Look::Arrow,
                    scale: 1.2,
                    ..MAGIC
                },
                walk: BOW_DRAW_WALK,
            },
        },
        grenade: Some(Grenade {
            fuse: 0.9,
            count: 2,
            cooldown: 6.0,
            radius: 2.4,
            damage: 10,
            poise: 1.0,
            drop: 2.0,
            root: 1.0,
        }),
        node: "w_bow",
        off_hand: "",
        stamina: 1.0,
        // 쇠뇌와 같은 꼴(전통 먼저 · 화살 피해는 뒤에).
        forge: Some([
            Forge {
                hit: 0,
                shot: 0,
                mag: 1,
            },
            Forge {
                hit: 0,
                shot: 5,
                mag: 1,
            },
            Forge {
                hit: 0,
                shot: 10,
                mag: 2,
            },
        ]),
    },
    // 파수꾼의 창(0272 2단계): 찌르기 셋(좁은 길 · 꿰뚫음) · 달려들며 찌르기 · 강타 = 내달려 꿰뚫기(길 위의 적을 모두 — 벽·턱
    // 앞에서 멈춘다, `World::lunge_stops`). RB는 창 던지기(종류의 칸, 0274 ②b).
    Weapon {
        id: Id::Spear,
        key: "spear",
        class: Some(Class::Spear),
        name: |s| s.weapon_spear,
        desc: |s| s.weapon_spear_d,
        place: |s| s.cx_weapon_spear,
        icon: crate::touch::Kind::Spear,
        // 아이템 그림(0272 2단계): 가장 긴 무기 — 대각선으로 길게(잎 꼴 촉이 위).
        view: (0.0, -80.0, -45.0),
        twin: false,
        combo: &SPEAR_COMBO,
        dash: Some(Hit {
            shape: Shape::Lane {
                half: SPEAR_HALF_WIDE,
            },
            knock: 1.4,
            cancel: 0.2,
            ..hit(1.0, 1.5, 0.25, 2.6, 20, 2.0, "spear_dash")
        }),
        heavy: Some(Heavy {
            time: 1.15,
            charge: 1.1,
            reach: 1.25,
            arc: 0.25,
            // 랩 `heavy.lunge` 0.6 m × 5 = 3 m를 시동 + 활성 동안 곧게 — 지나는 길과 그 앞 닿는 거리의 적이 모두 맞는다.
            lunge: 5.0,
            shape: Shape::Lane {
                half: SPEAR_HALF_WIDE,
            },
            damage: 20,
            poise: 3.0,
            walk: 0.9,
            anim_charge: "spear_charge",
            anim_heavy: "spear_lunge",
            cancel: 0.3,
            ..Heavy::BASE
        }),
        rb: Rb::Toss,
        grenade: None,
        node: "w_spear",
        off_hand: "",
        // 검(1.0)과 망치(1.7) 사이.
        stamina: 1.25,
        forge: Some(MELEE_FORGE),
    },
    // 빈집의 쌍단검(0274 ③): 빠른 4연타(찌르고 · 찌르고 · 베고 · 올려 벤다) · 달려들며 찌르기 · 강공격 = 꿰뚫고 지나가기(곧은 길 —
    // 창의 내달려 꿰뚫기와 같은 길: 적의 몸을 뚫고 지나 등 뒤에 선다, 벼랑 앞에서 멈춘다 — 다음 타가 급소). RB는 투척 단검 셋.
    Weapon {
        id: Id::Daggers,
        key: "daggers",
        class: Some(Class::Dagger),
        name: |s| s.weapon_daggers,
        desc: |s| s.weapon_daggers_d,
        place: |s| s.cx_weapon_daggers,
        icon: crate::touch::Kind::Daggers,
        // 아이템 그림(0274 ③): 오른손의 날을 검처럼 대각선으로 — 0277부터 거울로 한 번 더(`twin`, 두 날이 엇갈린다 —
        // 날 하나로는 한손검 · 완드와 갈리지 않았다, 터치 그림 `Kind::Daggers`와 같은 꼴).
        view: (0.0, -80.0, -45.0),
        twin: true,
        combo: &DAGGER_COMBO,
        dash: Some(Hit {
            knock: 1.2,
            cancel: 0.15,
            ..hit(0.8, 1.1, 0.5, 2.6, 20, 1.0, "dagger_dash")
        }),
        heavy: Some(Heavy {
            time: 0.8,
            charge: 0.7,
            reach: 1.0,
            arc: 0.3,
            // 랩 `heavy.lunge` 0.6 m × 4.5 = 2.7 m — 한 걸음 앞의 적을 뚫고 지나 등 뒤에 선다.
            lunge: 4.5,
            shape: Shape::Lane { half: DAGGER_HALF },
            damage: 20,
            poise: 1.5,
            walk: 1.2,
            anim_charge: "dagger_charge",
            anim_heavy: "dagger_lunge",
            cancel: 0.15,
            ..Heavy::BASE
        }),
        rb: Rb::Toss,
        grenade: None,
        node: "w_daggers",
        off_hand: "w_dagger_l",
        // 가벼운 날 넷 — 맨손(0.6)과 지팡이(0.75) 사이.
        stamina: 0.7,
        forge: Some(MELEE_FORGE),
    },
    // 가지치기 대검(0274 ④): 넓은 휩쓸기 다섯(앞의 셋까지) · 달려들며 올려 베기 · 강공격 = 두 손으로 모아 크게 내려베기(넓은 앞
    // 부채꼴 · 자세 피해 큼 — 망치처럼 땅을 울리지는 않는다). RB는 땅 내려찍기(종류의 칸).
    Weapon {
        id: Id::Greatsword,
        key: "greatsword",
        class: Some(Class::Greatsword),
        name: |s| s.weapon_greatsword,
        desc: |s| s.weapon_greatsword_d,
        place: |s| s.cx_weapon_greatsword,
        icon: crate::touch::Kind::Greatsword,
        // 아이템 그림(0274 ④): 긴 날 — 창처럼 대각선으로 길게(날끝이 위).
        view: (0.0, -80.0, -45.0),
        twin: false,
        combo: &GREATSWORD_COMBO,
        dash: Some(Hit {
            cleave: GREATSWORD_CLEAVE,
            knock: 1.4,
            cancel: 0.25,
            ..hit(1.1, 1.3, 1.0, 2.4, 20, 2.0, "gs_dash")
        }),
        heavy: Some(Heavy {
            time: 1.4,
            charge: 1.2,
            reach: 1.25,
            arc: 1.3,
            lunge: 1.2,
            shape: Shape::Arc,
            damage: 30,
            poise: 4.0,
            walk: 0.8,
            anim_charge: "gs_charge",
            anim_heavy: "gs_heavy",
            cancel: 0.4,
            ..Heavy::BASE
        }),
        rb: Rb::Toss,
        grenade: None,
        node: "w_greatsword",
        off_hand: "",
        // 창(1.25)과 망치(1.4) 사이 — 긴 날을 쉬지 않고 휘두르는 값.
        stamina: 1.3,
        forge: Some(MELEE_FORGE),
    },
    // 정원지기의 낫(0274 ⑤): 걸어 베기 둘 · 걸어 당기며 베기(셋째 — 끌어온다) · 달려들며 올려 베기(검의 것) · 강공격 = 크게 휘감아
    // 거두기(넓은 앞 부채꼴 240° — 맞은 적을 모두 몸 앞으로 끌어온다, 검 강공격의 동작). RB는 가시 씨앗(종류의 칸).
    Weapon {
        id: Id::Scythe,
        key: "scythe",
        class: Some(Class::Scythe),
        name: |s| s.weapon_scythe,
        desc: |s| s.weapon_scythe_d,
        place: |s| s.cx_weapon_scythe,
        icon: crate::touch::Kind::Sickle,
        // 아이템 그림(0274 ⑤): 휜 날이 옆으로 굽는다 — 망치처럼 덜 숙여 날의 굽음이 보이게.
        view: (0.0, -75.0, -40.0),
        twin: false,
        combo: &SCYTHE_COMBO,
        dash: Some(Hit {
            knock: 1.4,
            cancel: 0.2,
            ..hit(1.0, 1.25, 0.5, 2.4, 20, 2.0, "scythe_dash")
        }),
        heavy: Some(Heavy {
            time: 1.0,
            charge: 1.0,
            reach: 1.15,
            arc: 1.5,
            lunge: 0.8,
            shape: Shape::Arc,
            damage: 20,
            poise: 3.0,
            walk: 1.0,
            anim_charge: "scythe_charge",
            anim_heavy: "scythe_heavy",
            cancel: 0.25,
            pull: SCYTHE_HEAVY_PULL,
            ..Heavy::BASE
        }),
        rb: Rb::Toss,
        grenade: None,
        node: "w_scythe",
        off_hand: "",
        // 검(1.0)과 같다 — 한 손의 가벼운 날.
        stamina: 1.0,
        forge: Some(MELEE_FORGE),
    },
    // 잿빛 돌의 완드(0274 ⑥): 짧게 쳐 내기 둘 · 저주를 새기는 찌르기 · 달려들며 찌르기(쌍단검의 것) · 강공격 = 쥐었다 터뜨리는 저주
    // (지팡이의 모으기 · 터뜨림 동작 — 몸 둘레의 적에게 균열 가득). RB는 저주탄(종류의 칸 — 주된 공격).
    Weapon {
        id: Id::Wand,
        key: "wand",
        class: Some(Class::Wand),
        name: |s| s.weapon_wand,
        desc: |s| s.weapon_wand_d,
        place: |s| s.cx_weapon_wand,
        icon: crate::touch::Kind::Wand,
        // 아이템 그림(0274 ⑥): 짧은 막대 끝의 잿빛 돌 — 쌍단검처럼 대각선으로(돌이 위).
        view: (0.0, -80.0, -45.0),
        twin: false,
        combo: &WAND_COMBO,
        dash: Some(Hit {
            knock: 1.0,
            cancel: 0.15,
            ..hit(0.9, 0.9, 0.5, 2.0, 10, 1.0, "wand_dash")
        }),
        heavy: Some(Heavy {
            time: 0.9,
            charge: 0.9,
            reach: 0.9,
            arc: 1.0,
            lunge: 0.4,
            shape: Shape::Circle,
            damage: 20,
            poise: 1.5,
            walk: 1.2,
            anim_charge: "staff_charge",
            anim_heavy: "staff_release",
            cancel: 0.0,
            dose: (Kind::Rift, 1.0),
            ..Heavy::BASE
        }),
        rb: Rb::Toss,
        grenade: None,
        node: "w_wand",
        off_hand: "",
        // 검(1.0)과 같다 — 처음 안(지팡이와 같은 0.75)은 봇이 기력에 덜 막혀 문지기 싸움이 검보다 16% 빨랐다(보스 모의).
        stamina: 1.0,
        forge: Some(MELEE_FORGE),
    },
];

/// 무기 종류(0272): 계열(`Hand` — 화면에 드러내지 않는 묶음) → **종류** → 낱낱의 무기(`Id`). 판 · 물리 속성 · 주특성은 종류에,
/// 숫자 · 이름 · 생김새는 낱낱의 무기에. 새 종류 = 여기 한 줄 + `CLASSES` 한 줄(+ 글) — 자리(`index`)와 계열의 비트(`skills::ONE` …)는
/// 표에서 선다(0278 R19).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    Sword,
    Hammer,
    Staff,
    Crossbow,
    Bow,
    Spear,
    /// 쌍단검(0274 ③ — 도적).
    Dagger,
    /// 대검(0274 ④ — 광전사).
    Greatsword,
    /// 낫(0274 ⑤ — 드루이드).
    Scythe,
    /// 완드(0274 ⑥ — 흑마법사).
    Wand,
    /// 몽크(0286 — 맨손): 두 주먹이 다 찬다(양손 계열 · 방패와 들지 않는다), 주무기만 든다(`Quirks::one_slot`).
    Monk,
}

/// 무기의 주특성이 얹는 배(0225 → 0272 — 종류마다 하나, 0278 R21부터 합 타입: 읽는 쪽은 `art.force()` 같은 접근자 — 그 종류가 아니면
/// 1 = 배 없음): `force` = 타입 없는(물리) 스킬의 피해·자세 피해, `channel` = 스킬의 타가
/// 쌓는 타입 게이지, `sneak` = 급소(0274 ③ 쌍단검) — 등 뒤에서 · 휘청이거나 기절하거나 묶인 적에게 주는 방랑자의 모든 타(평타 ·
/// 강공격 · 탄 · 스킬 · 궁극기)의 피해 배(`world::Knack` → `hurt_by` 한 곳). 앞의 둘은 스킬에만, 급소는 손의 타 모두에.
/// `fury` = 분노(0274 ④ 대검): 둘레의 적이 많고 잃은 체력이 클수록 평타가 빨라진다 — 가장 빠를 때의 배(`World::fury`).
/// `sap` = 되살아나는 숨(0274 ⑤ 낫): 흡수가 돌려주는 체력 · 기력의 배 — 그리고 뿌리에 묶인 적도 흡수처럼 친 손에 돌려준다
/// (`World::tick_ails`의 흡수 한 곳). 받는 피해는 건드리지 않는다(되찾을 뿐 — 0095 원칙 2).
#[derive(Clone, Copy, PartialEq)]
pub enum Art {
    /// 배 없음(한손검 · 쇠뇌 · 활 · 창 · 맨손).
    Plain,
    Force(f32),
    Channel(f32),
    Sneak(f32),
    Fury(f32),
    Sap(f32),
    /// 치른 값(0274 ⑥ 완드): 균열(금)이 키우는 받는 피해의 덤 × 배(`hurt_by` 한 곳 — 금 간 적 ×1.3 → ×1.45), 그리고 금 간 채
    /// 쓰러진 적은 둘레의 적에게 울림을 남긴다(`World::tick_embers` — 죽음의 울림). 받는 피해는 건드리지 않는다.
    Hex(f32),
    /// 흐름(0286 몽크): 맞지 않고 손의 타를 이어 맞힐수록 평타가 빨라진다 — 가장 빠를 때의 배(`World::flow`, 대검의 분노와 같은 길
    /// `World::swing_rate`). 맞으면 흐름이 끊긴다. 받는 피해는 건드리지 않는다.
    Flow(f32),
}

impl Art {
    /// 망치의 물리력(그 밖은 1 — 배 없음).
    pub fn force(self) -> f32 {
        match self {
            Art::Force(k) => k,
            Art::Plain
            | Art::Channel(_)
            | Art::Sneak(_)
            | Art::Fury(_)
            | Art::Sap(_)
            | Art::Hex(_)
            | Art::Flow(_) => 1.0,
        }
    }

    /// 지팡이의 마법력(그 밖은 1).
    pub fn channel(self) -> f32 {
        match self {
            Art::Channel(k) => k,
            Art::Plain
            | Art::Force(_)
            | Art::Sneak(_)
            | Art::Fury(_)
            | Art::Sap(_)
            | Art::Hex(_)
            | Art::Flow(_) => 1.0,
        }
    }

    /// 쌍단검의 급소(그 밖은 1).
    pub fn sneak(self) -> f32 {
        match self {
            Art::Sneak(k) => k,
            Art::Plain
            | Art::Force(_)
            | Art::Channel(_)
            | Art::Fury(_)
            | Art::Sap(_)
            | Art::Hex(_)
            | Art::Flow(_) => 1.0,
        }
    }

    /// 대검의 분노(그 밖은 1).
    pub fn fury(self) -> f32 {
        match self {
            Art::Fury(k) => k,
            Art::Plain
            | Art::Force(_)
            | Art::Channel(_)
            | Art::Sneak(_)
            | Art::Sap(_)
            | Art::Hex(_)
            | Art::Flow(_) => 1.0,
        }
    }

    /// 낫의 되살아나는 숨(그 밖은 1).
    pub fn sap(self) -> f32 {
        match self {
            Art::Sap(k) => k,
            Art::Plain
            | Art::Force(_)
            | Art::Channel(_)
            | Art::Sneak(_)
            | Art::Fury(_)
            | Art::Hex(_)
            | Art::Flow(_) => 1.0,
        }
    }

    /// 완드의 치른 값(그 밖은 1).
    pub fn hex(self) -> f32 {
        match self {
            Art::Hex(k) => k,
            Art::Plain
            | Art::Force(_)
            | Art::Channel(_)
            | Art::Sneak(_)
            | Art::Fury(_)
            | Art::Sap(_)
            | Art::Flow(_) => 1.0,
        }
    }

    /// 몽크의 흐름(그 밖은 1).
    pub fn flow(self) -> f32 {
        match self {
            Art::Flow(k) => k,
            Art::Plain
            | Art::Force(_)
            | Art::Channel(_)
            | Art::Sneak(_)
            | Art::Fury(_)
            | Art::Sap(_)
            | Art::Hex(_) => 1.0,
        }
    }

    /// 주특성 글의 `{0}`에 넣는 수(%): 배의 덤(배 없음은 0).
    pub fn percent(self) -> i32 {
        ((self
            .force()
            .max(self.channel())
            .max(self.sneak())
            .max(self.fury())
            .max(self.sap())
            .max(self.hex())
            .max(self.flow())
            - 1.0)
            * 100.0)
            .round() as i32
    }
}

/// Debug는 옛 구조체의 꼴 그대로(배 여섯 — 고정 파일 `weapon_fixture.txt`가 이 글을 바이트로 본다, 0278 R21). 뒤에 온 배(0286 흐름)는
/// 그 배가 있을 때만 덧붙인다 — 다른 종류의 줄이 그대로이게.
impl std::fmt::Debug for Art {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("Art");
        d.field("force", &self.force())
            .field("channel", &self.channel())
            .field("sneak", &self.sneak())
            .field("fury", &self.fury())
            .field("sap", &self.sap())
            .field("hex", &self.hex());
        if self.flow() != 1.0 {
            d.field("flow", &self.flow());
        }
        d.finish()
    }
}

/// 망치의 물리력 · 지팡이의 마법력(0225 — 처음 값, 판정에서 고친다).
pub const ART_FORCE: f32 = 1.3;
pub const ART_CHANNEL: f32 = 1.5;
/// 쌍단검의 급소(0274 ③ — BG3 도적의 급소 공격): 등 뒤(적이 보는 쪽에서 `SNEAK_BACK_DEG`° 넘게 돌아선 자리)에서 · 휘청이는 ·
/// 기절한 · 묶인 적에게 피해 ×1.5. 처음 값 — 판정에서 고친다.
pub const ART_SNEAK: f32 = 1.5;
/// 급소의 "등 뒤": 적의 앞에서 이 각(°)보다 더 돌아선 자리(뒤쪽 140°의 부채꼴).
pub const SNEAK_BACK_DEG: f32 = 110.0;
/// 대검의 분노(0274 ④ — BG3 광전사의 분노 · "몰릴수록 거세다"): 평타가 가장 빠를 때의 배(+20% → 0284: +50% — 들어 올림 · 거둠이
/// 무거워진 만큼 빈 둘레와 가득의 차이가 손에 느껴지게: 마구 누른 휘두르기 사이 28틱 → 19틱, 검 17틱보다는 늘 느리다). 분노 0..1 =
/// 둘레 `FURY_RADIUS` m의 깨어 있는 적 수 ÷ `FURY_CROWD` + 잃은 체력의 몫(1에서 멈춘다 — `World::fury`). 보스 하나 = 0.25(+12.5%),
/// 넷에 에워싸이거나 체력이 바닥이면 가득. 한 휘두르기 동안은 그 처음의 빠르기에 머문다(동작이 흔들리지 않게). 값은 보스 모의로.
pub const ART_FURY: f32 = 1.5;
pub const FURY_RADIUS: f32 = 3.5;
pub const FURY_CROWD: f32 = 4.0;
/// 분노가 가득 찬 알림(0277 — `World::watch_fury`)을 다시 걸 문턱: 가득(1) 뒤 이 밑으로 내려가야 다음 가득이 또 울린다(적 하나 = 0.25 —
/// 넷이 셋이 되는 것만으로는 다시 울리지 않는다).
pub const FURY_REARM: f32 = 0.7;
/// 대검을 휘두르는 동안(평타 · 강공격 · 모으기) 맞으면 휘청이는 시간의 배(0274 ④ — 덧옷의 `Mods::flinch`와 같은 손잡이 `hurt.stun s`,
/// 밀림은 `GREATSWORD_KNOCK`). 받는 피해는 그대로다(0095 원칙 2 — 시험).
pub const GREATSWORD_FLINCH: f32 = 0.4;
/// 낫의 되살아나는 숨(0274 ⑤ — 드루이드의 "되살아난다"): 흡수가 돌려주는 체력 · 기력 ×1.5(체력 2 → 3 · 기력 6 → 9), 뿌리에 묶인 적을
/// 쳐도 그만큼 돌아온다(한 틱에 한 번 — 흡수와 같은 셈). 문지기도 묶이면(느려지기만 해도) 돌려준다.
pub const ART_SAP: f32 = 1.5;
/// 완드의 치른 값(0274 ⑥ — 흑마법사의 "저주로 약하게 한다"): 금 간 적이 받는 피해의 덤 ×1.5(잡몹 +30% → +45% · 문지기 +15% → +22.5%).
pub const ART_HEX: f32 = 1.5;
/// 치른 값의 죽음의 울림(0274 ⑥): 금 간 채 쓰러진 적의 둘레(m)와 그 안의 적마다 쌓이는 울림(한 틱에 한 적은 한 번 — 떼가 커도 곱절이
/// 되지 않게, 0223 8절).
pub const DEATH_KNELL_R: f32 = 4.0;
pub const DEATH_KNELL_DOSE: f32 = 0.5;
pub const GREATSWORD_KNOCK: f32 = 0.5;
/// 몽크의 흐름(0286 — 무협의 "끊기지 않는 흐름", BG3 몽크의 연타): 평타가 가장 빠를 때의 배(+30% — 대검의 분노 +50%보다 작다: 맨손은
/// 처음부터 빠르다). 흐름 0..1 = 맞힌 손의 타마다 `FLOW_STEP`(네 대에 가득), 방랑자가 맞으면 0, 손의 타가 `FLOW_HOLD`초 동안 닿지 않으면
/// 초당 `FLOW_FADE`씩 빠진다. 휘두르기 하나 동안은 그 처음의 빠르기에 머문다(분노와 같은 걸쇠 — `World::latch_fury`). 값은 보스 모의로.
pub const ART_FLOW: f32 = 1.3;
pub const FLOW_STEP: f32 = 0.25;
pub const FLOW_HOLD: f32 = 1.5;
pub const FLOW_FADE: f32 = 0.5;

/// 보조특성(0272 1b): 보조무기 슬롯에 멘 종류가 **손에 쥐지 않아도** 보태는 한 가지(방패의 막기와 같은 말 — 0224). 주무기로 든
/// 동안에는 없다 — 바꿔 들면 특성도 맞바뀐다. 숫자는 능력치의 있는 칸(`stats::Mods`)으로만, 능력치를 합치는 한 곳
/// (`save::Progress::stats`)이 한 줄로 얹는다. 받는 피해 · 패링 판정 시간 · 회피는 건드리지 않는다(0095 원칙 2 — 시험).
/// 값은 주특성의 반 아래(망치 주특성 +30% → 보조 +5%, 지팡이 +50% → +15%).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SubTrait {
    pub mods: Mods,
    /// 등에서도 장전된다(쇠뇌): 멘 동안 탄창이 다시 찬다 — 능력치가 아니라 월드의 규칙(`World::swap_weapons` · 시계).
    pub stowed_reload: bool,
}

impl SubTrait {
    /// 보조특성 글의 `{0}`에 넣는 수(%): 바꾸는 칸의 몫(스킬 쿨타임은 줄어드는 몫). 숫자가 없는 특성(쇠뇌)은 0.
    pub fn percent(self) -> i32 {
        let m = self.mods;
        // 쓰러뜨림의 기력은 더하는 칸 — 기력의 바탕(랩 `stamina.max` 100)에 대한 몫으로(= 더하는 수 그대로).
        let base = crate::tuning::SPECS[crate::tuning::idx(crate::tuning::P::StaminaMax)].def;
        // 등잔 회복량(0274 ② 망치)도 더하는 칸 — 한 모금의 바탕(랩 `flask.heal` 20)에 대한 몫으로.
        let sip = crate::tuning::SPECS[crate::tuning::idx(crate::tuning::P::FlaskHeal)].def;
        let d = [m.stamina_regen, m.poise, m.ail, m.skill_cd, m.ember]
            .into_iter()
            .map(|x| (x - 1.0).abs())
            .chain([
                m.kill_stamina / base,
                m.heal as f32 / sip,
                m.charged_dmg as f32 / SUB_CHARGED_BASE,
                m.kill_heal as f32 / sip,
                m.toss_dmg as f32 / SUB_TOSS_BASE,
            ])
            .fold(0.0, f32::max);
        (d * 100.0).round() as i32
    }
}

/// 보조특성의 값(0272 1b — 처음 값, 판정에서 고친다): 한손검 = 기력 회복 · 망치 = 자세 피해(0274 ②부터 등잔 회복량 — 아래) · 지팡이 = 속성 게이지 · 활 = 스킬 쿨타임.
/// 기력 회복·자세 피해는 보스 모의(`balance::sub_knob_probe` 300판)로 골랐다 — 어느 몸의 싸움도 4% 넘게 줄지 않게: 둘 다 문턱이
/// 있는 값이다(기력 회복 ×1.05부터 망치가 문지기의 틈마다 한 타를 더 넣어 −11% · 자세 피해 ×1.10이면 검이 문지기의 자세를 한 타
/// 먼저 깨 −6%). 속성 게이지·스킬 쿨타임은 봇이 스킬을 쓰지 않아 재지 못했다(셈으로만 — 0272 "1b 만든 것").
pub const SUB_REGEN: f32 = 1.04;
/// 망치의 보조특성(0274 ② — 성직자의 맛보기, 치유 — 옛 값은 자세 피해 ×1.05, 0272 1b): 등잔 한 모금 +3(바탕 20의 15% — 주특성 물리력 +30%의 반). 보스 모의
/// (`balance::class_fight_probe`)로 골랐다 — 싸움을 줄이지 않고(되찾을 뿐) 이김이 조금 는다(0274 "② 만든 것").
pub const SUB_HEAL: i32 = 3;
pub const SUB_AIL: f32 = 1.15;
pub const SUB_SKILL_CD: f32 = 0.95;
/// 창의 보조특성(0272 2단계): 적을 쓰러뜨릴 때마다 기력 +8(장신구 "굶주린 것"의 25의 셋에 하나 — 떼를 줄로 꿰는 종류의 덤).
/// 처음 안이던 최대 기력은 보스 모의(`balance::sub_knob_probe` `SIM_STAMINA`, 300판)에서 +1부터 문턱이 있었다(망치로 문지기를
/// 칠 때 −9.5% — 봇이 틈마다 한 타를 더 넣는다, 지팡이는 +5에서 −24%) — 어느 값도 4% 기준 안에 못 든다. 쓰러뜨림의 기력은 보스
/// 싸움(쓰러뜨릴 것이 없다)을 줄이지 않는다.
pub const SUB_KILL_STAMINA: f32 = 8.0;
/// 대검의 보조특성(0274 ④ — 광전사의 맛보기, 힘): **다 모은** 강공격의 피해 +3(검의 다 모은 강공격 30의 10% — 주특성 분노 +20%의 반).
/// 처음 안(강공격 · 패링 반격 모두 +2 → +1)은 보스 모의에서 "검 / 보조 대검"이 문지기를 5.7% 빨리 쓰러뜨렸다(패링 반격도 강공격이라
/// 한 대가 문턱을 넘는다 — 0272 1b의 자세 피해와 같은 꼴) — 쥐어 다 모은 것만으로 좁혔다(0274 "④ 만든 것").
pub const SUB_CHARGED: i32 = 3;
/// 보조특성 글의 몫의 바탕(검의 다 모은 강공격 — `{0}%`의 분모).
pub const SUB_CHARGED_BASE: f32 = 30.0;
/// 낫의 보조특성(0274 ⑤ — 드루이드의 맛보기, 거두는 손): 적을 쓰러뜨릴 때마다 체력 +2(등잔 한 모금 20의 10% — 주특성 되살아나는 숨
/// +50%의 다섯에 하나). 쓰러뜨릴 것이 없는 보스 싸움을 줄이지 않는다(창의 쓰러뜨림 기력과 같은 꼴 — 모의 0%).
pub const SUB_KILL_HEAL: i32 = 2;
/// 완드의 보조특성(0274 ⑥ — 흑마법사의 맛보기, 입에 붙은 말): 종류의 원거리 수단(RB — 손도끼 · 빛 · 마법 탄 …)의 피해 +2(손도끼 20의
/// 10% — 주특성 치른 값 +50%의 다섯에 하나). 봇은 RB를 쓰지 않아 보스 싸움을 줄이지 않는다(모의 0%). 사격 무기의 볼트 · 화살은 아니다.
pub const SUB_TOSS: i32 = 2;
/// 보조특성 글의 몫의 바탕(검의 손도끼 — `{0}%`의 분모).
pub const SUB_TOSS_BASE: f32 = 20.0;
/// 쌍단검의 보조특성(0274 ③ — 도적의 맛보기, 손버릇): 쓰러뜨린 적이 남기는 잔불 ×1.10(주특성 급소 +50%의 다섯에 하나). 싸움의 수
/// (피해 · 자세 · 기력 · 회피 · 막기)를 하나도 건드리지 않아 보스 싸움을 줄이지 않는다(모의 0% — 1b의 4% 기준은 저절로).
pub const SUB_EMBER: f32 = 1.10;

/// 교체 공격(0272 1b)에서 들어오는 무기가 치는 한 타: 근접 = 그 무기의 대시 공격(`Weapon::dash`), 원거리 = 보통 사격 한 발
/// (`Weapon::rb`의 `Shot` — 탄 하나를 쓴다). 새 동작·새 표 칸이 없다 — 새 무기는 표의 그 칸이 곧 교체 공격이다.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry {
    Dash,
    Shot,
}

/// 계열(0223 → 0272 — 종류의 윗층, 화면에 드러내지 않는다): 한손 근접은 왼손에 방패를 들 수 있다 · 양손 근접 · 원거리(양손, 장전).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hand {
    One,
    Two,
    Ranged,
}

/// 종류 표의 한 줄(0272): 판의 이름 · 가운데 열의 머리(물리 속성) · 대표 타입 둘(판의 왼쪽 · 오른쪽 갈래) · 큰 기술 · 주특성 ·
/// 보조특성(1b).
pub struct ClassDef {
    pub class: Class,
    pub hand: Hand,
    /// 종류의 이름(판 줄 · 장비 탭 · 도감).
    pub name: Tr,
    /// 물리 속성(참격 · 타격 · 관통 · 마법) — 그 종류의 판에서 타입 둘 사이 가운데 열의 머리.
    pub attribute: Tr,
    /// 대표 타입 둘(0270 C — 판의 왼쪽 · 오른쪽 갈래).
    pub pair: [Kind; 2],
    /// 그 종류의 큰 기술(판의 셋째 줄 가운데).
    pub great: Ult,
    /// 주특성: 스킬에 얹는 배와 그것을 말하는 한 줄(`{0}` = `Art::percent`).
    pub art: Art,
    pub main_trait: Tr,
    /// 보조특성(1b): 보조무기 슬롯에 멨을 때의 한 가지와 그것을 말하는 한 줄(`{0}` = `SubTrait::percent`).
    pub sub: SubTrait,
    pub sub_trait: Tr,
    /// 종류의 원거리 수단(0274 ②b — RB를 종류마다): 근접 종류는 제 것(손도끼 · 신성한 빛 · 마법탄 · 창 던지기 …), 사격 종류는
    /// 없다(`None` — 그 무기 표의 `Rb::Shot`). 새 종류 = 이 칸 하나(새 꼴이면 `Bolt::look` 한 갈래와 월드·그림의 한 갈래).
    pub toss: Option<Toss>,
    /// 빈 손의 결(0276): 보조무기 슬롯이 비었을 때(무기도 방패도 없음) — 하나에 전념하는 쥐는 법 · 그 이름(몸의 말) · 한 줄.
    /// 새 종류 = 이 셋(새 결이면 `Bare` 한 갈래와 월드의 그 자리 한 줄).
    pub bare: Bare,
    pub bare_name: Tr,
    pub bare_trait: Tr,
    /// 종류만의 작은 성질(0278 R19 — 월드가 "이 종류인가"를 묻던 것): 대부분 `Quirks::NONE`.
    pub quirks: Quirks,
}

/// 종류만의 작은 성질(0278 R19): 월드가 종류의 이름으로 묻지 않고 이 칸을 읽는다(`World::keen` · `World::steady_swing`).
/// 새 성질 = 칸 하나 + `NONE`의 값 + 그것을 가진 종류의 줄만(`..Quirks::NONE`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quirks {
    /// 날카로움의 덤을 받는 손(0270 C5 → 0274 ③ — 날카로움을 대표로 삼는 쌍단검, `hurt_by`의 `Knack::keen`; 맨손도 — 종류가 없어
    /// `World::keen`이 따로 본다). 방패를 들면 없다.
    pub keen: bool,
    /// 휘두르는 동안(평타 · 모으기 · 강공격) 맞으면 휘청 · 밀림의 배(0274 ④ 대검 — `GREATSWORD_FLINCH` · `GREATSWORD_KNOCK`,
    /// 받는 피해는 그대로 — `wanderer_takes_hit` 한 곳). 없으면 그대로.
    pub steady_swing: Option<(f32, f32)>,
    /// RB가 이 종류의 주된 공격이다(0281 — 완드의 저주탄, 0282 — 지팡이의 마법 탄): 방패와 들어도 방패는 막고 들이받기만 하고 던지지
    /// 않는다 — RB는 그대로 종류의 것(`World::shield_throws` 한 곳 — 지팡이는 양손이라 방패와 들지 않는다). 없으면 방패가 RB를 맡는다
    /// (한손검 · 망치 · 낫 · 맨손). 보스 모의의 봇도 이것을 보고 틈에 칸이 있는 한 쏜다(`balance::kit_strike`).
    pub rb_main: bool,
    /// 주무기만 든다(0286 — 몽크): 보조무기 슬롯을 쓰지 않는다 — 보조무기도 방패도 들지 않고(슬롯이 잠긴다), 이 종류의 무기는 보조무기로
    /// 메지도 못한다. 빈 손의 결이 늘 켜진다(`World::bare`). 앱의 장비 길(`App::wield` · `set_sub_weapon` · `set_shield`)과 기록 읽기
    /// (`save::Progress::lock_off_hand`)가 이것을 본다.
    pub one_slot: bool,
}

impl Quirks {
    pub const NONE: Quirks = Quirks {
        keen: false,
        steady_swing: None,
        rb_main: false,
        one_slot: false,
    };
}

/// 빈 손의 결(0276 — BG3 다용도 · 양손 쥐기 · 하데스의 방패를 본 것): 보조무기 슬롯을 비우면 주무기 종류마다 손이 다르게 움직인다 —
/// 보조무기가 주는 것(보조특성 · 교체 공격 · 둘째 스킬 한 쌍)과 맞바꿀 만한 것. 받는 피해 · 패링 판정 시간 · 회피 거리와 무적은
/// 건드리지 않는다(0095 원칙 2 — 시험). 월드는 `World::bare`(보조 슬롯이 비었나 = 앱이 넘긴 부무기 · 방패) 한 곳에서 묻는다.
/// 맨손은 종류가 아니라 결이 없다.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bare {
    /// 한손검 — 두 손으로 쥔다: 강공격(패링 반격 포함)이 두 손 내려베기(동작 = 망치의 내려치기, 자세 피해 × `BARE_SWORD_HEAVY_POISE`
    /// — 문지기는 `BARE_SWORD_HEAVY_BOSS`), 평타 끝 타가 넓어진다(각 × `BARE_SWORD_LAST_ARC`).
    TwoHands,
    /// 망치 — 기도하는 손: 등잔 한 모금을 삼키는 순간 둘레에 울림 고리(가까운 적에 울림 게이지 + 밀림, 회복은 그대로).
    Prayer,
    /// 지팡이 — 두 손으로 엮는다: RB 칸 +`BARE_STAFF_EXTRA`(종류의 칸 위에 — 0282), 모으는 시간 × `BARE_STAFF_CHARGE`.
    Weave,
    /// 쇠뇌 — 장치를 하나 더: 공격 버튼(수류탄 자리)이 덫을 놓는다(`world::Snare` — 밟은 잡몹은 묶이고 문지기는 느려진다).
    Snare,
    /// 활 — 가벼운 걸음: 당긴 채 걷기 × `BARE_BOW_WALK`(0.6 → 0.85), 끝까지 당긴 화살의 오금 `BARE_BOW_LAME`초.
    LightStep,
    /// 창 — 자루를 길게 잡는다: 평타 · 대시 공격의 닿는 거리 × `BARE_SPEAR_REACH`, 내달려 꿰뚫기가 한 번 더 이어진다.
    LongGrip,
    /// 쌍단검 — 그림자 걸음: 회피가 끝난 뒤(`SHADOW_STEP_LATE`초 안) 첫 타는 급소로 친다(회피 자체는 그대로).
    ShadowStep,
    /// 대검 — 남김없이 휘두른다(0274 ④ — BG3 광전사의 무모한 공격): 분노가 곱절로 빨리 차고(`BARE_FURY_RATE`), 가득 차 있으면 평타
    /// 다섯의 끝에서 멈추지 않고 둘째 타로 되돌아 이어진다(`World::next_hit`).
    Reckless,
    /// 낫 — 땅을 짚는 손(0274 ⑤ — 빈 손이 땅을 짚어 주문을 넓힌다): 뿌리 속성 스킬의 범위 × `BARE_GROUND_REACH`(4레벨의 치수 덤과
    /// 같은 칸 — `World::skill_eff`), 가시 씨앗(RB)이 솟는 둘레도 같은 배(`World::rb_bolts`).
    GroundHand,
    /// 완드 — 계약의 표(0274 ⑥ — BG3 흑마법사의 "어둠의 축복": 쓰러뜨리면 되찾는다): 저주탄에 맞은 표(`Enemy::marks.pact` —
    /// `PACT_MARK_SECS`초)가 남은 적이 쓰러지면 체력 `BARE_PACT_HEAL`이 돌고 RB 칸 `BARE_PACT_CHARGE`가 돌아온다 — 손이 저주탄으로
    /// 마무리하는 쪽으로 움직인다(`World::tick_embers`).
    PactMark,
    /// 몽크 — 열린 손(0286 — 보조무기를 들 수 없는 몽크에게는 늘 켜진다): 받아치면(패링) 흐름이 단번에 가득 찬다 — 받아친 뒤의 반격과
    /// 그 뒤의 연타가 가장 빠르게 나간다(`World::flow`).
    OpenHand,
}

/// 빈 손의 결 값(0276 — 처음 값, 판정에서 고친다). 보스 모의(`balance::bare_hand_probe`)로 골랐다 — 어느 싸움도 같은 무기 + 가장
/// 좋은 보조무기 조합보다 4% 넘게 빨라지지 않게(0276 "만든 것").
/// 두 손 내려베기의 자세 피해: 잡몹 · 정예 ×1.6(밀려 넘어지는 손), 문지기 ×1.05 — 문지기의 자세 막대에는 문턱이 있다(모의: ×1.1부터
/// 종지기와의 싸움이 5% 빨라진다 · 패링 반격도 강공격이라 보통 손의 봇에게도 든다).
pub const BARE_SWORD_HEAVY_POISE: f32 = 1.6;
pub const BARE_SWORD_HEAVY_BOSS: f32 = 1.05;
pub const BARE_SWORD_HEAVY_REACH: f32 = 1.1;
pub const BARE_SWORD_LAST_ARC: f32 = 1.5;
/// 두 손으로 엮는 지팡이의 RB 칸 덤(0282: 종류의 칸 `Toss::charges` 위에 하나 — 옛 값은 칸의 수 넷을 그대로 적었다: 칸 셋 + 1).
pub const BARE_STAFF_EXTRA: f32 = 1.0;
pub const BARE_STAFF_CHARGE: f32 = 0.6;
/// 활을 당긴 채 걷기(`cast.walk` 0.5에 곱한다 — 0.85): 보통은 `BOW_DRAW_WALK` 1.2(0.6).
pub const BARE_BOW_WALK: f32 = 1.7;
pub const BARE_BOW_LAME: f32 = 3.0;
pub const BARE_SPEAR_REACH: f32 = 1.2;
/// 그림자 걸음: 회피가 끝난 뒤 첫 타가 급소인 창(초).
pub const SHADOW_STEP_LATE: f32 = 0.8;
/// 남김없이 휘두른다(대검의 빈 손): 분노가 차는 배(둘레의 적 · 잃은 체력의 몫에 곱한다).
pub const BARE_FURY_RATE: f32 = 2.0;
/// 땅을 짚는 손(낫의 빈 손): 뿌리 스킬 · 가시 씨앗의 범위 배.
pub const BARE_GROUND_REACH: f32 = 1.25;
/// 계약의 표(완드의 빈 손): 저주탄의 표가 남는 초 · 표가 남은 적이 쓰러질 때 도는 체력(등잔 한 모금 20의 15%) · 돌아오는 RB 칸.
pub const PACT_MARK_SECS: f32 = 4.0;
pub const BARE_PACT_HEAL: i32 = 3;
pub const BARE_PACT_CHARGE: f32 = 1.0;

/// 종류 표(판 줄의 차례 — `plates::PLATES`가 이 차례로 선다). 대검 · 창은 뒤에 한 줄씩(0272 2단계).
pub const CLASSES: &[ClassDef] = &[
    ClassDef {
        class: Class::Sword,
        hand: Hand::One,
        name: |s| s.class_sword,
        attribute: |s| s.attr_slash,
        // 0274: 전사의 결 — 충격 · 흡수(전투 기술: 자루 치기 · 밀치기 · 재기). 옛 짝 날카로움 · 균열의 스킬은 트리에 그대로 서 있다
        // (가운데 열 — 단검 트리가 오면(0274 ③) 그리로 옮긴다).
        pair: [Kind::Shock, Kind::Drain],
        great: Ult::Flurry,
        art: Art::Plain,
        main_trait: |s| s.trait_sword,
        sub: SubTrait {
            mods: Mods {
                stamina_regen: SUB_REGEN,
                ..Mods::NONE
            },
            stowed_reload: false,
        },
        sub_trait: |s| s.subtrait_sword,
        toss: Some(AXE_TOSS),
        bare: Bare::TwoHands,
        bare_name: |s| s.bare_sword,
        bare_trait: |s| s.bare_sword_d,
        quirks: Quirks::NONE,
    },
    // 망치(0274 — 성직자의 결): 한 손에 드는 종지기의 메 — 방패와 함께 든다(한손검처럼, 방패를 들면 방패 트리).
    ClassDef {
        class: Class::Hammer,
        hand: Hand::One,
        name: |s| s.class_hammer,
        attribute: |s| s.attr_strike,
        pair: [Kind::Shock, Kind::Knell],
        great: Ult::Upend,
        art: Art::Force(ART_FORCE),
        main_trait: |s| s.trait_hammer,
        sub: SubTrait {
            mods: Mods {
                heal: SUB_HEAL,
                ..Mods::NONE
            },
            stowed_reload: false,
        },
        sub_trait: |s| s.subtrait_hammer,
        toss: Some(LIGHT_TOSS),
        bare: Bare::Prayer,
        bare_name: |s| s.bare_hammer,
        bare_trait: |s| s.bare_hammer_d,
        quirks: Quirks::NONE,
    },
    ClassDef {
        class: Class::Staff,
        hand: Hand::Two,
        name: |s| s.class_staff,
        attribute: |s| s.attr_magic,
        pair: [Kind::Frost, Kind::Ember],
        great: Ult::Blizzard,
        art: Art::Channel(ART_CHANNEL),
        main_trait: |s| s.trait_staff,
        sub: SubTrait {
            mods: Mods {
                ail: SUB_AIL,
                ..Mods::NONE
            },
            stowed_reload: false,
        },
        sub_trait: |s| s.subtrait_staff,
        toss: Some(MAGIC_TOSS),
        bare: Bare::Weave,
        bare_name: |s| s.bare_staff,
        bare_trait: |s| s.bare_staff_d,
        // 0282: 마법 탄이 주된 공격(완드와 같은 성질 — 지팡이는 양손이라 방패와 들지 않는다: 모의의 봇이 칸이 있는 한 쏜다).
        quirks: Quirks {
            rb_main: true,
            ..Quirks::NONE
        },
    },
    ClassDef {
        class: Class::Crossbow,
        hand: Hand::Ranged,
        name: |s| s.class_crossbow,
        attribute: |s| s.attr_pierce,
        pair: [Kind::Ember, Kind::Shock],
        great: Ult::Line,
        art: Art::Plain,
        main_trait: |s| s.trait_crossbow,
        sub: SubTrait {
            mods: Mods::NONE,
            stowed_reload: true,
        },
        sub_trait: |s| s.subtrait_crossbow,
        toss: None,
        bare: Bare::Snare,
        bare_name: |s| s.bare_crossbow,
        bare_trait: |s| s.bare_crossbow_d,
        quirks: Quirks::NONE,
    },
    ClassDef {
        class: Class::Bow,
        hand: Hand::Ranged,
        name: |s| s.class_bow,
        attribute: |s| s.attr_pierce,
        pair: [Kind::Root, Kind::Frost],
        great: Ult::Rain,
        art: Art::Plain,
        main_trait: |s| s.trait_bow,
        sub: SubTrait {
            mods: Mods {
                skill_cd: SUB_SKILL_CD,
                ..Mods::NONE
            },
            stowed_reload: false,
        },
        sub_trait: |s| s.subtrait_bow,
        toss: None,
        bare: Bare::LightStep,
        bare_name: |s| s.bare_bow,
        bare_trait: |s| s.bare_bow_d,
        quirks: Quirks::NONE,
    },
    // 창(0272 2단계): 양손 근접 · 관통 · 뿌리와 날카로움. 주특성은 무브셋 그 자체(가장 긴 거리 · 한 줄 꿰뚫기) — 스킬에 얹는 배는 없다.
    ClassDef {
        class: Class::Spear,
        hand: Hand::Two,
        name: |s| s.class_spear,
        attribute: |s| s.attr_pierce,
        pair: [Kind::Root, Kind::Sharp],
        great: Ult::Gale,
        art: Art::Plain,
        main_trait: |s| s.trait_spear,
        sub: SubTrait {
            mods: Mods {
                kill_stamina: SUB_KILL_STAMINA,
                ..Mods::NONE
            },
            stowed_reload: false,
        },
        sub_trait: |s| s.subtrait_spear,
        toss: Some(JAVELIN_TOSS),
        bare: Bare::LongGrip,
        bare_name: |s| s.bare_spear,
        bare_trait: |s| s.bare_spear_d,
        quirks: Quirks::NONE,
    },
    // 쌍단검(0274 ③ — 도적의 결): 두 손에 날 하나씩이라 방패와 들지 않는다(양손 근접) · 관통(찌르는 날 — D&D · BG3의 단검은 찌름
    // 피해, 한손검(전사)은 참격) · 날카로움과 균열(출혈을 내고 약점을 연다 — 옛 한손검 트리의 넷을 넘겨받았다) · 주특성 급소.
    ClassDef {
        class: Class::Dagger,
        hand: Hand::Two,
        name: |s| s.class_dagger,
        attribute: |s| s.attr_pierce,
        pair: [Kind::Sharp, Kind::Rift],
        great: Ult::Ambush,
        art: Art::Sneak(ART_SNEAK),
        main_trait: |s| s.trait_dagger,
        sub: SubTrait {
            mods: Mods {
                ember: SUB_EMBER,
                ..Mods::NONE
            },
            stowed_reload: false,
        },
        sub_trait: |s| s.subtrait_dagger,
        toss: Some(KNIFE_TOSS),
        bare: Bare::ShadowStep,
        bare_name: |s| s.bare_dagger,
        bare_trait: |s| s.bare_dagger_d,
        quirks: Quirks {
            keen: true,
            ..Quirks::NONE
        },
    },
    // 대검(0274 ④ — 광전사의 결): 두 손으로 드는 긴 날(방패와 들지 않는다) · 참격(한손검과 같은 물리 속성 — 둘 다 베는 날이다: 갈리는
    // 것은 속성 짝 · 무브셋 · 주특성) · 균열과 잔불(분노 — 쪼개고 태운다) · 주특성 분노 · 보조특성 힘(강공격 피해).
    ClassDef {
        class: Class::Greatsword,
        hand: Hand::Two,
        name: |s| s.class_greatsword,
        attribute: |s| s.attr_slash,
        pair: [Kind::Rift, Kind::Ember],
        great: Ult::Tempest,
        art: Art::Fury(ART_FURY),
        main_trait: |s| s.trait_greatsword,
        sub: SubTrait {
            mods: Mods {
                charged_dmg: SUB_CHARGED,
                ..Mods::NONE
            },
            stowed_reload: false,
        },
        sub_trait: |s| s.subtrait_greatsword,
        toss: Some(QUAKE_TOSS),
        bare: Bare::Reckless,
        bare_name: |s| s.bare_greatsword,
        bare_trait: |s| s.bare_greatsword_d,
        quirks: Quirks {
            steady_swing: Some((GREATSWORD_FLINCH, GREATSWORD_KNOCK)),
            ..Quirks::NONE
        },
    },
    // 낫(0274 ⑤ — 드루이드의 결): 한 손의 휜 날(한손검 · 망치처럼 방패와 든다 — 방패를 들면 방패 트리, 빈 손이면 땅을 짚는 손) ·
    // 참격(휜 날로 벤다 — 한손검 · 대검과 같은 물리 속성: 갈리는 것은 속성 짝 · 무브셋 · 주특성) · 뿌리와 흡수(묶고 되살아난다) ·
    // 주특성 되살아나는 숨 · 보조특성 거두는 손 · RB 가시 씨앗.
    ClassDef {
        class: Class::Scythe,
        hand: Hand::One,
        name: |s| s.class_scythe,
        attribute: |s| s.attr_slash,
        pair: [Kind::Root, Kind::Drain],
        great: Ult::Grove,
        art: Art::Sap(ART_SAP),
        main_trait: |s| s.trait_scythe,
        sub: SubTrait {
            mods: Mods {
                kill_heal: SUB_KILL_HEAL,
                ..Mods::NONE
            },
            stowed_reload: false,
        },
        sub_trait: |s| s.subtrait_scythe,
        toss: Some(SEED_TOSS),
        bare: Bare::GroundHand,
        bare_name: |s| s.bare_scythe,
        bare_trait: |s| s.bare_scythe_d,
        quirks: Quirks::NONE,
    },
    // 완드(0274 ⑥ — 흑마법사의 결, 계약 — 방랑자도 모르는 계약): 한 손에 드는 짧은 막대 — **0281부터 한손**(방패와 든다, 사장
    // 2026-11-27 "완드도 사실 방패 가능하긴 해야할것 같은데"): 방패를 들어도 완드 트리는 그대로이고(방패 트리와 함께), RB는 주된 공격인
    // 저주탄 그대로다(`Quirks::rb_main` — 방패는 막고 들이받을 뿐 던지지 않는다). 마법(물리 속성 — 지팡이와 같다: 갈리는 것은 속성 짝 ·
    // 곧은 저주탄 · 주특성) · 균열과 울림(저주로 약하게 하고 죽음의 울림을 부른다) · 주특성 치른 값 · 보조특성 입에 붙은 말 ·
    // RB 저주탄 · 빈 손 계약의 표.
    ClassDef {
        class: Class::Wand,
        hand: Hand::One,
        name: |s| s.class_wand,
        attribute: |s| s.attr_magic,
        pair: [Kind::Rift, Kind::Knell],
        great: Ult::Price,
        art: Art::Hex(ART_HEX),
        main_trait: |s| s.trait_wand,
        sub: SubTrait {
            mods: Mods {
                toss_dmg: SUB_TOSS,
                ..Mods::NONE
            },
            stowed_reload: false,
        },
        sub_trait: |s| s.subtrait_wand,
        toss: Some(CURSE_TOSS),
        bare: Bare::PactMark,
        bare_name: |s| s.bare_wand,
        bare_trait: |s| s.bare_wand_d,
        quirks: Quirks {
            rb_main: true,
            ..Quirks::NONE
        },
    },
    // 몽크(0286 — 사장 "맨손 일땐, 몽크도 넣어줘. 대신 몽크는 보조무기를 들수 없어."): 맨손이 종류가 되었다 — 두 주먹이 다 차서 방패와
    // 들지 않고(양손 계열), 보조무기 슬롯이 잠긴다(`Quirks::one_slot` — 무기도 방패도). 타격(주먹과 발) · 충격과 날카로움(급소를 쳐
    // 흔들고, 손톱으로 할퀸다 — 맨손은 0270 C5부터 날카로움의 덤을 받는 손이다 `Quirks::keen`) · 주특성 흐름 · 빈 손 열린 손(늘 켜진다) ·
    // RB 돌(맨손의 것 그대로 — 손끝으로 튕긴다). 보조특성은 없다(보조무기로 메지 못한다 — 칸은 비어 있고 글은 잠긴 슬롯을 말한다).
    ClassDef {
        class: Class::Monk,
        hand: Hand::Two,
        name: |s| s.class_monk,
        attribute: |s| s.attr_strike,
        pair: [Kind::Shock, Kind::Sharp],
        great: Ult::Palms,
        art: Art::Flow(ART_FLOW),
        main_trait: |s| s.trait_monk,
        sub: SubTrait {
            mods: Mods::NONE,
            stowed_reload: false,
        },
        sub_trait: |s| s.subtrait_monk,
        toss: Some(STONE_TOSS),
        bare: Bare::OpenHand,
        bare_name: |s| s.bare_monk,
        bare_trait: |s| s.bare_monk_d,
        quirks: Quirks {
            keen: true,
            one_slot: true,
            ..Quirks::NONE
        },
    },
];

impl Class {
    /// 표의 자리(`CLASSES` — 스킬의 `hands` 비트 · 판의 차례가 이것을 쓴다): 표에서 찾는다 — 새 종류는 표의 한 줄로 자리를 받는다
    /// (시험 `every_weapon_has_a_class_and_the_class_table_is_in_order` — 표에 없는 종류는 컴파일 때 멈춘다).
    pub const fn index(self) -> usize {
        let mut i = 0;
        while i < CLASSES.len() {
            if CLASSES[i].class as u8 == self as u8 {
                return i;
            }
            i += 1;
        }
        panic!("CLASSES has every Class")
    }

    pub fn def(self) -> &'static ClassDef {
        &CLASSES[self.index()]
    }
}

/// 손에 쥔 꼴(0225 → 0272 → 0281): 스킬과 판이 "지금 무엇을 쥐었나"를 묻는 한 곳. 종류는 `Id::class` 하나에서 오고, 이 꼴은 거기에
/// **방패를 들었나**만 더한다 — 한손 계열과 방패 = 그 종류의 판 **과** 방패 판(`Shield(종류)` — 0281 멀티클래스: 둘 다 쓴다),
/// 그 밖에는 그 종류의 판(`Class`). 무기 판이 없는 꼴(`Bare`)은 0286부터 쓰는 무기가 없다(맨손 = 몽크의 판) — 판의 셈이 지닌다.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Grip {
    /// 무기 판이 없는 꼴(0286까지의 맨손 — 지금은 쥐는 무기가 없다, 판 · 스킬의 셈이 지닌다).
    Bare,
    /// 그 종류의 무기(방패 없이, 또는 방패와 들지 않는 종류).
    Class(Class),
    /// 한손 계열(한손 종류 · 맨손)과 방패 — 방패 판 + 든 종류의 판(맨손이면 `None` — 방패 판만). 판의 쪽에서 본 방패 판의 꼴은
    /// `Shield(None)`이다(`plates::Plate::grip`).
    Shield(Option<Class>),
}

impl Id {
    /// 무기의 종류(0272 — 무기 표의 줄 `Weapon::class`). 0286부터 맨손도 종류(몽크)다.
    pub fn class(self) -> Option<Class> {
        self.def().class
    }

    /// 계열(0223 → 0272): 종류의 것(0286부터 맨손 = 몽크 = 양손 — 종류가 없는 무기는 없다).
    pub fn hand(self) -> Hand {
        self.class().map_or(Hand::One, |c| c.def().hand)
    }

    /// 손에 쥔 꼴(0225 → 0272 → 0281): 한손 계열은 왼손의 방패로 갈린다 — 방패와 함께면 제 종류의 판과 방패 판(`Shield(종류)`),
    /// 아니면 제 종류의 판. 스킬이 이것으로 쓸 수 있나를 가른다(`skills::Skill::fits`), 판 줄의 금빛 점도 이것(`plates::Plate::holds`).
    pub fn grip(self, shield: bool) -> Grip {
        match self.class() {
            c if shield && self.takes_shield() => Grip::Shield(c),
            Some(c) => Grip::Class(c),
            None => Grip::Bare,
        }
    }

    /// 방패와 드나(0223 → 0281): 한손 계열(한손 종류 · 맨손) — 방패가 막기 · 들이받기를 맡고 쥔 꼴이 방패 트리를 더한다.
    pub fn takes_shield(self) -> bool {
        self.hand() == Hand::One
    }

    /// 주무기만 드는 무기인가(0286 — 종류 표의 `Quirks::one_slot`, 맨손 = 몽크): 보조무기 슬롯이 잠긴다(무기도 방패도 없다).
    pub fn one_slot(self) -> bool {
        self.class().is_some_and(|c| c.def().quirks.one_slot)
    }

    /// 보조무기로 멜 수 있나(0286): 종류가 있고 주무기만 드는 종류가 아니다 — 맨손은 메지 않는다.
    pub fn slings(self) -> bool {
        self.class().is_some_and(|c| !c.def().quirks.one_slot)
    }

    /// 방패와 들면 RB가 방패 던지기인가(0276 → 0281): 방패와 들고, 종류의 RB가 주된 공격이 아니다(`Quirks::rb_main` — 완드는
    /// 저주탄 그대로). 월드(`World::shield_throws`)와 장비 탭의 글이 같이 읽는다.
    pub fn throws_shield(self) -> bool {
        self.takes_shield() && !self.class().is_some_and(|c| c.def().quirks.rb_main)
    }

    /// 무기의 주특성이 스킬에 얹는 배(0225 — 사장 "망치는 물리력을, 지팡이는 마법력에 대한 특색" → 0272 종류의 것).
    pub fn art(self) -> Art {
        self.class().map_or(Art::Plain, |c| c.def().art)
    }

    /// 보조무기 슬롯에 멨을 때의 보조특성(0272 1b) — 보조무기로 메지 못하는 종류(0286 몽크 — 맨손)는 없다.
    pub fn sub_trait(self) -> Option<SubTrait> {
        self.class()
            .filter(|c| !c.def().quirks.one_slot)
            .map(|c| c.def().sub)
    }

    /// 보조무기 슬롯이 비었을 때의 결(0276) — 몽크(0286)는 슬롯이 늘 비어 늘 켜진다(`World::bare`).
    pub fn bare(self) -> Option<Bare> {
        self.class().map(|c| c.def().bare)
    }

    /// 교체 공격으로 들어올 때의 한 타(0272 1b): 대시 공격이 있으면 그것, 사격 무기면 한 발.
    pub fn entry(self) -> Option<Entry> {
        let w = self.def();
        match w.rb {
            _ if w.dash.is_some() => Some(Entry::Dash),
            Rb::Shot { .. } => Some(Entry::Shot),
            Rb::Toss => None,
        }
    }

    /// RB의 원거리 수단(0274 ②b): 근접 무기 = 종류의 칸(`ClassDef::toss`), 맨손 = 돌. 사격 무기는 없다(표의 `Rb::Shot`).
    pub fn toss(self) -> Option<Toss> {
        match self.def().rb {
            Rb::Toss => Some(self.class().map_or(STONE_TOSS, |c| {
                c.def().toss.expect("a melee class has its own toss")
            })),
            Rb::Shot { .. } => None,
        }
    }

    /// RB의 탄 둘(0274 ②b — 한 곳에서): 보통 한 번 · 모은 것. 근접 무기 = 종류의 원거리 수단, 사격 무기 = 표의 볼트·모은 사격.
    pub fn bolts(self) -> (Bolt, Bolt) {
        match self.def().rb {
            Rb::Shot { bolt, charged, .. } => (bolt, charged),
            Rb::Toss => {
                let t = self.toss().unwrap_or(STONE_TOSS);
                (t.bolt, t.charged)
            }
        }
    }

    /// 사격 무기의 손 리듬(0274 ②b) — 근접 무기는 없다.
    pub fn rhythm(self) -> Option<Rhythm> {
        match self.def().rb {
            Rb::Shot { rhythm, .. } => Some(rhythm),
            Rb::Toss => None,
        }
    }

    /// 누르면 들어 겨누고 떼면 던지는 RB인가(0274 ②b — 창 던지기: 던지면 손이 비어 "쏜 뒤 쥐어 모으기"를 못 한다).
    pub fn winds(self) -> bool {
        self.toss().is_some_and(|t| t.bolt.returns())
    }

    /// 활처럼 누르면 당기는 무기인가(0274 ②b — `Rhythm::Draw`).
    pub fn draws(self) -> bool {
        matches!(self.rhythm(), Some(Rhythm::Draw { .. }))
    }

    /// 원거리 무기인가(0223 — 양손, 평타 버튼 = 사격·특수 버튼 = 수류탄).
    pub fn is_ranged(self) -> bool {
        matches!(self.def().rb, Rb::Shot { .. })
    }

    pub fn def(self) -> &'static Weapon {
        WEAPONS
            .iter()
            .find(|w| w.id == self)
            .expect("WEAPONS has every Id")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 0278 R18 · R19: 무기 표 · 종류 표의 수(콤보 · 대시 · 강공격 · RB · 수류탄 · 부위 · 기력 · 벼린 단 · 종류의 계열 · 짝 ·
    /// 궁극기 · 주특성 · 보조특성 · RB 칸 · 빈 손)를 `..Toss::BASE`·`..Heavy::BASE`로 줄여 쓰고 칸을 옮겨도 값이 그대로다 —
    /// Debug 글이 고정 파일과 같다. 일부러 값을 바꿨으면 `WEAPON_FIXTURE_WRITE=1 cargo test weapon_fixture`.
    #[test]
    fn weapon_and_class_rows_match_the_weapon_fixture() {
        const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/weapon_fixture.txt");
        let mut text = String::new();
        for w in WEAPONS {
            text += &format!(
                "{:?}\ncombo {:?}\ndash {:?}\nheavy {:?}\nrb {:?}\ngrenade {:?}\nnode {:?} {:?} stamina {:?} forge {:?}\n",
                w.id, w.combo, w.dash, w.heavy, w.rb, w.grenade, w.node, w.off_hand, w.stamina, w.forge
            );
        }
        for c in CLASSES {
            text += &format!(
                "{:?} {:?} {:?} {:?}\nart {:?}\nsub {:?}\ntoss {:?}\nbare {:?}\n",
                c.class, c.hand, c.pair, c.great, c.art, c.sub, c.toss, c.bare
            );
        }
        for (name, t) in [("stone", STONE_TOSS), ("shield", SHIELD_TOSS)] {
            text += &format!("{name} {t:?}\n");
        }
        if std::env::var("WEAPON_FIXTURE_WRITE").is_ok() {
            std::fs::write(FILE, &text).expect("write weapon_fixture.txt");
            return;
        }
        let was = std::fs::read_to_string(FILE).expect("weapon_fixture.txt");
        assert_eq!(text, was, "a weapon or class row changed");
    }

    /// 0284: 평타의 동작 시각 — 들어 올림이 없으면 옛 `e × clip_rate` 그대로, 있으면(대검) 판정 시작 `s0`에 동작이 만든 판정 시작이
    /// 오고(그 앞은 빨리 들어 올려 꼭대기에서 머문다 — 앞 반에 들어 올림의 4분의 3), 그 뒤는 들어 올림을 늘이기 전의 빠르기.
    #[test]
    fn a_wound_up_swing_lingers_at_the_top_then_swings_at_full_speed() {
        let (c0, _) = SWEEPS.iter().find(|r| r.0 == "gs1").unwrap().2;
        for e in [0.0, 0.05, 0.2, 0.4] {
            assert_eq!(clip_at("gs1", e, c0, 1.0), e * clip_rate("gs1", c0));
        }
        let s0 = c0 * GREATSWORD_WINDUP;
        assert!((clip_at("gs1", s0, s0, GREATSWORD_WINDUP) - c0).abs() < 1e-5);
        assert!(clip_at("gs1", s0 * 0.5, s0, GREATSWORD_WINDUP) >= 0.75 * c0 - 1e-5);
        let step = clip_at("gs1", s0 + 0.1, s0, GREATSWORD_WINDUP) - c0;
        assert!(
            (step - 0.1).abs() < 1e-5,
            "the sweep itself is not slowed: {step}"
        );
        for h in &GREATSWORD_COMBO {
            assert_eq!(h.windup, GREATSWORD_WINDUP, "{}", h.anim);
        }
        assert!(WEAPONS.iter().filter(|w| w.id != Id::Greatsword).all(|w| w
            .combo
            .iter()
            .chain(w.dash.iter())
            .all(|h| h.windup == 1.0)));
    }

    /// 0272: 종류 표 — 줄의 자리 = `Class::index`, 무기마다 종류 하나(맨손만 없다), 계열·주특성의 배는 종류의 것.
    /// 대표 타입 둘은 서로 다른 타입이고, 주특성의 수는 배에서 나온다(망치 +30% · 지팡이 +50% · 나머지는 배가 없다).
    #[test]
    fn every_weapon_has_a_class_and_the_class_table_is_in_order() {
        for (i, c) in CLASSES.iter().enumerate() {
            assert_eq!(c.class.index(), i, "{:?}", c.class);
            assert_eq!(c.class.def().class, c.class);
            let [a, b] = c.pair;
            assert!(a != b && a != Kind::Physical && b != Kind::Physical);
            assert!(c.art.force() >= 1.0 && c.art.channel() >= 1.0);
        }
        let greats: Vec<Ult> = CLASSES.iter().map(|c| c.great).collect();
        for (i, g) in greats.iter().enumerate() {
            assert!(!greats[i + 1..].contains(g), "{g:?} on two classes");
        }
        for id in IDS {
            // 0286: 맨손도 종류(몽크)다 — 종류가 없는 무기는 없다.
            assert!(id.class().is_some(), "{id:?}");
            // 원거리 계열 = 사격 무기(표의 `rb`와 종류 표의 계열이 어긋나지 않는다).
            assert_eq!(id.hand() == Hand::Ranged, id.is_ranged(), "{id:?}");
        }
        assert_eq!(Id::Sword.class(), Some(Class::Sword));
        assert_eq!(Id::Crossbow.class(), Some(Class::Crossbow));
        assert_eq!(Id::Bow.class(), Some(Class::Bow));
        // 0286: 맨손 = 몽크 — 두 주먹이 다 차 방패와 들지 않는다(양손 계열), 주특성은 흐름.
        assert_eq!(Id::Fists.class(), Some(Class::Monk));
        assert_eq!(Id::Fists.hand(), Hand::Two, "both fists are full");
        assert!(!Id::Fists.takes_shield());
        assert_eq!(Id::Fists.art().flow(), ART_FLOW);
        assert_eq!(Class::Monk.def().art.percent(), 30);
        assert_eq!(
            (Id::Sword.hand(), Id::Hammer.hand(), Id::Spear.hand()),
            (Hand::One, Hand::One, Hand::Two)
        );
        assert_eq!(Id::Hammer.art().force(), ART_FORCE);
        assert_eq!(Id::Staff.art().channel(), ART_CHANNEL);
        assert_eq!(Class::Hammer.def().art.percent(), 30);
        assert_eq!(Class::Staff.def().art.percent(), 50);
        assert_eq!(Class::Bow.def().art.percent(), 0);
        // 쥔 꼴: 한손 계열만 방패로 갈린다 — 0281부터 방패와 함께면 제 종류의 판과 방패 판(멀티클래스).
        assert_eq!(Id::Sword.grip(false), Grip::Class(Class::Sword));
        assert_eq!(Id::Sword.grip(true), Grip::Shield(Some(Class::Sword)));
        // 0274: 망치도 한손.
        assert_eq!(Id::Hammer.grip(true), Grip::Shield(Some(Class::Hammer)));
        assert_eq!(Id::Hammer.grip(false), Grip::Class(Class::Hammer));
        // 0274 ⑤: 낫도 한손.
        assert_eq!(Id::Scythe.hand(), Hand::One);
        assert_eq!(Id::Scythe.grip(true), Grip::Shield(Some(Class::Scythe)));
        assert_eq!(Id::Scythe.grip(false), Grip::Class(Class::Scythe));
        assert_eq!(Id::Scythe.art().sap(), ART_SAP);
        // 0281: 완드도 한손.
        assert_eq!(Id::Wand.hand(), Hand::One);
        assert_eq!(Id::Wand.grip(true), Grip::Shield(Some(Class::Wand)));
        assert_eq!(Id::Fists.grip(false), Grip::Class(Class::Monk));
        assert_eq!(Id::Fists.grip(true), Grip::Class(Class::Monk), "no shield");
        assert_eq!(Id::Staff.grip(true), Grip::Class(Class::Staff));
        assert_eq!(Id::Bow.grip(true), Grip::Class(Class::Bow));
    }

    /// 0272 1b: 보조특성 — 종류마다 한 가지(숫자 칸 하나, 또는 쇠뇌의 "등에서도 장전된다"). 받는 피해 · 패링 판정 시간 · 회피 ·
    /// 밀림을 건드리지 않고(0095 원칙 2 — 방어는 손이다), 능력치를 낫게만 바꾸며, 주특성의 반을 넘지 않는다.
    #[test]
    fn every_class_has_one_small_sub_trait_that_leaves_defence_alone() {
        use crate::stats::Stats;
        let none = Stats::default();
        for c in CLASSES.iter() {
            // 0286: 보조무기로 메지 못하는 종류(몽크)는 보조특성이 없다 — 칸은 비어 있고 글은 잠긴 슬롯을 말한다.
            if c.quirks.one_slot {
                assert_eq!(c.sub.mods, Mods::NONE, "{:?}", c.class);
                assert!(!c.sub.stowed_reload && c.sub.percent() == 0);
                continue;
            }
            let got = Stats::compute(&[c.sub.mods]);
            assert_eq!(
                (
                    got.taken,
                    got.parry,
                    got.dodge_cost,
                    got.knock,
                    got.flinch,
                    got.max_hp
                ),
                (
                    none.taken,
                    none.parry,
                    none.dodge_cost,
                    none.knock,
                    none.flinch,
                    none.max_hp
                ),
                "{:?}",
                c.class
            );
            // 한 가지: 숫자 칸이 하나 바뀌거나(나머지는 그대로), 숫자 없이 월드의 규칙 하나.
            let knobs = [
                got.stamina_regen != none.stamina_regen,
                got.poise != none.poise,
                got.ail != none.ail,
                got.skill_cd != none.skill_cd,
                got.kill_stamina != none.kill_stamina,
                got.heal != none.heal,
                got.ember != none.ember,
                got.charged_dmg != none.charged_dmg,
                got.kill_heal != none.kill_heal,
                got.toss_dmg != none.toss_dmg,
            ];
            let n = knobs.iter().filter(|k| **k).count();
            assert_eq!(n + c.sub.stowed_reload as usize, 1, "{:?}", c.class);
            let only = Stats {
                stamina_regen: none.stamina_regen,
                poise: none.poise,
                ail: none.ail,
                skill_cd: none.skill_cd,
                kill_stamina: none.kill_stamina,
                heal: none.heal,
                ember: none.ember,
                charged_dmg: none.charged_dmg,
                kill_heal: none.kill_heal,
                toss_dmg: none.toss_dmg,
                ..got
            };
            assert_eq!(only, none, "{:?}: nothing else moves", c.class);
            assert!(got.stamina_regen >= 1.0 && got.poise >= 1.0 && got.ail >= 1.0);
            assert!(got.skill_cd <= 1.0 && got.kill_stamina >= 0.0 && got.heal >= 0);
            assert!(got.ember >= 1.0 && got.charged_dmg >= 0 && got.kill_heal >= 0);
            assert!(got.toss_dmg >= 0);
            // 작게: 어느 것도 15%를 넘지 않고, 주특성의 배가 있는 종류는 그 반 아래.
            let p = c.sub.percent();
            assert_eq!(p == 0, c.sub.stowed_reload, "{:?}", c.class);
            assert!(p <= 15, "{:?}: {p}%", c.class);
            if c.art.percent() > 0 {
                assert!(p * 2 <= c.art.percent(), "{:?}", c.class);
            }
        }
        assert_eq!(Class::Sword.def().sub.percent(), 4);
        // 0274 ②: 망치 = 등잔 한 모금 +3(바탕 20의 15%) — 주특성(물리력 +30%)의 반.
        assert_eq!(Class::Hammer.def().sub.percent(), 15);
        assert_eq!(Class::Hammer.def().sub.mods.heal, SUB_HEAL);
        assert_eq!(Class::Staff.def().sub.percent(), 15);
        assert_eq!(Class::Bow.def().sub.percent(), 5);
        assert_eq!(Class::Spear.def().sub.percent(), 8, "stamina +8 per kill");
        // 0274 ③: 쌍단검 = 쓰러뜨린 적의 잔불 +10%(손버릇) — 주특성 급소 +50%의 다섯에 하나.
        assert_eq!(Class::Dagger.def().sub.percent(), 10);
        assert_eq!(Class::Dagger.def().art.percent(), 50);
        // 0274 ④: 대검 = 다 모은 강공격 피해 +3(검의 다 모은 강공격 30의 10% — 힘) · 주특성 분노(0284: +20% → +50%)의 반 아래.
        assert_eq!(Class::Greatsword.def().sub.percent(), 10);
        assert_eq!(Class::Greatsword.def().sub.mods.charged_dmg, SUB_CHARGED);
        assert_eq!(Class::Greatsword.def().art.percent(), 50);
        // 0274 ⑤: 낫 = 쓰러뜨릴 때마다 체력 +2(등잔 한 모금 20의 10% — 거두는 손) · 주특성 되살아나는 숨 +50%의 다섯에 하나.
        assert_eq!(Class::Scythe.def().sub.percent(), 10);
        assert_eq!(Class::Scythe.def().sub.mods.kill_heal, SUB_KILL_HEAL);
        assert_eq!(Class::Scythe.def().art.percent(), 50);
        assert!(Class::Crossbow.def().sub.stowed_reload);
        assert_eq!(Id::Fists.sub_trait(), None, "the monk is never slung");
        assert_eq!(Id::Hammer.sub_trait(), Some(Class::Hammer.def().sub));
    }

    /// 0272 1b: 교체 공격의 한 타는 표의 있는 칸이다 — 근접(맨손도) = 대시 공격, 사격 무기 = 보통 한 발. 모든 무기에 있다.
    #[test]
    fn every_weapon_has_an_entry_strike_from_its_own_row() {
        for id in IDS {
            let want = if id.is_ranged() {
                Entry::Shot
            } else {
                Entry::Dash
            };
            assert_eq!(id.entry(), Some(want), "{id:?}");
            assert_eq!(want == Entry::Dash, id.def().dash.is_some(), "{id:?}");
        }
    }

    /// 0274 ②b "RB를 종류마다": 근접 종류는 저마다 제 원거리 수단(꼴이 서로 다르다 — 클래스의 결), 사격 종류는 칸이 비고 무기 표의
    /// 사격을 쓴다. 맨손(0286부터 몽크)은 돌 — 가장 약하다. 손도끼는 자세 피해가 가장 크고, 빛은 울림을 싣고, 창은 꿰뚫어 돌아온다.
    #[test]
    fn every_melee_kind_has_its_own_rb_and_ranged_kinds_shoot() {
        let mut looks = Vec::new();
        for c in CLASSES.iter() {
            assert_eq!(c.toss.is_some(), c.hand != Hand::Ranged, "{:?}", c.class);
            if let Some(t) = c.toss {
                assert_eq!(t.bolt.look, t.charged.look, "{:?}", c.class);
                // 모은 것은 더 세다 — RB가 주된 공격인 종류(`Quirks::rb_main` — 완드 0274 ⑥ · 지팡이 0282)만은 피해가 아니라 꿰뚫음 ·
                // 자세 피해(완드는 균열 가득)로(칸이 빨리 차는 주된 공격이라 피해를 곱절로 내면 마구 쏘기보다 늘 세다 — 모의).
                assert!(
                    t.charged.damage > t.bolt.damage
                        || (c.quirks.rb_main
                            && t.charged.damage == t.bolt.damage
                            && t.charged.pierce
                            && t.charged.poise > t.bolt.poise),
                    "{:?}",
                    c.class
                );
                assert!(
                    !looks.contains(&t.bolt.look),
                    "{:?}: one form per kind",
                    c.class
                );
                looks.push(t.bolt.look);
            }
        }
        // 0286: 돌은 몽크(맨손)의 것 하나.
        assert_eq!(Class::Monk.def().toss, Some(STONE_TOSS));
        for id in IDS {
            assert_eq!(id.toss().is_some(), !id.is_ranged(), "{id:?}");
        }
        assert_eq!(Id::Fists.toss(), Some(STONE_TOSS));
        assert_eq!(
            Id::Staff.toss(),
            Some(MAGIC_TOSS),
            "the mage keeps the bolt"
        );
        let rb = |id: Id| id.bolts().0;
        for id in [Id::Sword, Id::Hammer, Id::Staff, Id::Spear] {
            assert!(rb(Id::Fists).damage <= rb(id).damage, "{id:?}");
            assert!(
                rb(Id::Sword).poise >= rb(id).poise,
                "{id:?}: the axe breaks stances"
            );
        }
        assert!(
            rb(Id::Sword).range < rb(Id::Staff).range,
            "the axe is short"
        );
        assert!(rb(Id::Hammer).smites() && rb(Id::Hammer).dose.0 == Kind::Knell);
        // 0274 ⑤: 가시 씨앗도 날지 않고 겨눈 자리에 떨어진다(신성한 빛과 같은 길) — 뿌리를 싣는다.
        assert!(rb(Id::Scythe).smites() && rb(Id::Scythe).dose.0 == Kind::Root);
        assert!(
            rb(Id::Scythe).damage < rb(Id::Hammer).damage,
            "the seed binds, the light strikes"
        );
        assert!(rb(Id::Spear).returns() && rb(Id::Spear).pierce);
        assert!(
            rb(Id::Spear).range > rb(Id::Sword).range,
            "the spear flies far"
        );
        // 사격 무기의 리듬은 둘이 다르다(쇠뇌 = 장전, 활 = 당기기).
        assert!(matches!(Id::Crossbow.rhythm(), Some(Rhythm::Crank { .. })));
        assert!(Id::Bow.draws() && !Id::Crossbow.draws() && !Id::Sword.draws());
    }

    #[test]
    fn every_id_has_one_row_and_melee_weapons_have_a_combo() {
        for id in IDS {
            assert_eq!(WEAPONS.iter().filter(|w| w.id == id).count(), 1, "{id:?}");
            let w = id.def();
            match w.rb {
                Rb::Toss => assert!(
                    !w.combo.is_empty() && w.grenade.is_none() && id.toss().is_some(),
                    "{id:?}: melee = X combo + RB toss"
                ),
                Rb::Shot { mag, .. } => assert!(
                    mag > 0 && w.grenade.is_some(),
                    "{id:?}: ranged = RB shot + X grenade"
                ),
            }
        }
        assert_eq!(Id::by_key("fists"), Some(Id::Fists));
    }

    /// 0087: 무기 표의 모든 평타·대시·강타 동작은 휘두르는 쪽 표에 있다(빠지면 박자 부채꼴이 말없이 가운데로 사라진다).
    #[test]
    fn every_weapon_motion_has_a_sweep() {
        for w in WEAPONS.iter() {
            let anims = w
                .combo
                .iter()
                .chain(w.dash.iter())
                .map(|h| h.anim)
                .chain(w.heavy.iter().map(|h| h.anim_heavy));
            for a in anims {
                assert!(SWEEPS.iter().any(|(n, _, _)| *n == a), "{:?}: {a}", w.id);
            }
        }
        // 쪽이 동작과 맞는지는 모델에서 다시 잰다(0193 — `swing_tests`). 검의 타는 번갈아 되돌아온다.
        assert_ne!(sweep_of("attack"), sweep_of("attack2"));
    }

    // ── 눈금 옮기기(0158 N0): 체력·피해가 ×10이 되어도 몇 대에 쓰러지는가는 그대로 ──────────────────────────────

    /// 옮기기 전의 수(작은 정수 — 0158 N0 직전의 표를 그대로 적어 둔다, 고치지 않는다).
    /// 무기마다: 콤보 타들 · 대시 공격 · 강타(덜 모음) · 여진 · RB(마법/사격) · 모은 탄 · 수류탄. 없는 것은 0.
    struct Old {
        id: Id,
        combo: &'static [i32],
        dash: i32,
        heavy: i32,
        aftershock: i32,
        rb: i32,
        charged: i32,
        grenade: i32,
    }
    const OLD_WEAPONS: [Old; 11] = [
        Old {
            id: Id::Sword,
            combo: &[1, 1, 2],
            dash: 2,
            heavy: 2,
            aftershock: 0,
            // 0274 ②b: RB를 종류마다 — 마법 탄 1 → 던지는 손도끼 2(모은 것은 3 그대로). 값을 바꾼 것이라 이 줄도 같이 고친다.
            rb: 2,
            charged: 3,
            grenade: 0,
        },
        Old {
            id: Id::Hammer,
            // 0274: 한손 3연타(옛 2 · 3 → 1 · 1 · 3) — 눈금을 옮긴 것이 아니라 무브셋을 바꾼 것이라 이 줄도 같이 고친다.
            combo: &[1, 1, 3],
            dash: 2,
            heavy: 3,
            aftershock: 2,
            // 0274 ②b: 무거운 탄 2 · 4 → 신성한 빛 2 · 3(둘레를 친다).
            rb: 2,
            charged: 3,
            grenade: 0,
        },
        // 0282: 지팡이는 눈금을 옮긴 뒤 다시 짰다(마법이 먼저 — 3연타 · 강공격 한 눈금 · 모은 탄 한 눈금) — 새 값의 열의 하나.
        // 눈금을 옮기던 때의 값은 평타 [1, 1, 1, 2] · 강공격 2 · 모은 탄 3이었다.
        Old {
            id: Id::Staff,
            combo: &[1, 1, 1],
            dash: 1,
            heavy: 1,
            aftershock: 0,
            rb: 1,
            charged: 1,
            grenade: 0,
        },
        Old {
            id: Id::Crossbow,
            combo: &[],
            dash: 0,
            heavy: 0,
            aftershock: 0,
            // 0173: 쇠뇌를 낮췄다(사격 2 → 1, 모은 사격 5 → 4) — 눈금을 옮긴 것이 아니라 값을 바꾼 것이라 이 표의 줄도 같이 고친다.
            // 0210: 모은 사격 4 → 3(같은 까닭). 0243: 모은 사격 3 → 2(같은 까닭).
            rb: 1,
            charged: 2,
            grenade: 3,
        },
        Old {
            id: Id::Fists,
            combo: &[1, 1, 1],
            dash: 1,
            heavy: 1,
            aftershock: 0,
            // 0274 ②b: 돌 던지기 — 모은 것 3 → 2(약하다).
            rb: 1,
            charged: 2,
            grenade: 0,
        },
        // 0231: 눈금을 옮긴 뒤에 생긴 무기 — 옛 눈금으로 적어 둔다(새 수의 열의 하나).
        Old {
            id: Id::Bow,
            combo: &[],
            dash: 0,
            heavy: 0,
            aftershock: 0,
            rb: 1,
            charged: 1,
            grenade: 1,
        },
        // 0272 2단계: 창도 그 뒤 — 옛 눈금으로.
        Old {
            id: Id::Spear,
            combo: &[1, 1, 2],
            dash: 2,
            heavy: 2,
            aftershock: 0,
            // 0274 ②b: 창 던지기 2(모은 것 3 그대로).
            rb: 2,
            charged: 3,
            grenade: 0,
        },
        // 0274 ③: 쌍단검도 그 뒤 — 옛 눈금으로(투척 단검은 날 하나의 피해).
        Old {
            id: Id::Daggers,
            combo: &[1, 1, 1, 2],
            dash: 2,
            heavy: 2,
            aftershock: 0,
            rb: 1,
            charged: 2,
            grenade: 0,
        },
        // 0274 ④: 대검도 그 뒤 — 옛 눈금으로(땅 내려찍기는 충격파 한 번의 피해).
        Old {
            id: Id::Greatsword,
            combo: &[1, 1, 1, 2, 2],
            dash: 2,
            heavy: 3,
            aftershock: 0,
            rb: 1,
            charged: 2,
            grenade: 0,
        },
        // 0274 ⑤: 낫도 그 뒤 — 옛 눈금으로(가시 씨앗은 솟는 뿌리 한 번의 피해).
        Old {
            id: Id::Scythe,
            combo: &[1, 1, 2],
            dash: 2,
            heavy: 2,
            aftershock: 0,
            rb: 1,
            charged: 2,
            grenade: 0,
        },
        // 0274 ⑥: 완드도 그 뒤 — 옛 눈금으로(저주탄은 한 눈금, 모은 것은 둘).
        Old {
            id: Id::Wand,
            combo: &[1, 1, 1],
            dash: 1,
            heavy: 2,
            aftershock: 0,
            rb: 1,
            charged: 1,
            grenade: 0,
        },
    ];
    /// 옛 랩 값·상수: 다 모은 강타 덤 · 딱 맞춘 떼기 덤 · 내려찍기 · 치명타 덤 · 되받은 종 조각 · 큰 기술의 벽 쿵.
    const OLD_FULL_BONUS: i32 = 1;
    const OLD_PERFECT: i32 = 1;
    const OLD_PLUNGE: i32 = 2;
    const OLD_CRIT: i32 = 2;
    const OLD_SHARD_REFLECT: i32 = 3;
    const OLD_ULT_SLAM: i32 = 2;
    /// 옛 스킬 피해(표 순서: 휘돌이 · 가로지르기 · 끌어모으기 · 땅울림).
    const OLD_SKILLS: [i32; 4] = [1, 2, 0, 2];
    /// 옛 종 체력(표 `species::ALL` 순서: 잔재 · 가시짐승 · 문지기 · 등딱지짐승 · 꼬투리 · 종지기).
    /// 이끼 덮인 문지기·이끼먹이(0185)는 눈금을 옮긴 뒤에 생겼다 — 옛 눈금으로 64·2(새 640·20의 열의 하나). 작업층의 넷(0212)도
    /// 그 뒤 — 권양지기는 0216 W4에서 1200 → 1600(옛 눈금 160). 고요한 사슴(0270 D)도 그 뒤 — 옛 눈금으로 새 체력의 열의 하나.
    const OLD_SPECIES_HP: [i32; 12] = [3, 2, 40, 5, 2, 58, 96, 160, 4, 6, 2, 150];
    /// 옛 등급 2·엘리트 배율(올림) — `tuning::TIERS[1].hp`·`ELITE_HP`.
    const OLD_TIER2: f32 = 1.5;
    const OLD_ELITE: f32 = 2.0;

    /// 한 무기의 모든 타(이름, 옛 피해, 새 피해) — 강타는 덜 모음·다 모음·딱 맞춤, 모은 탄은 덜 모음·다 모음·딱 맞춤(원거리만).
    fn attacks(o: &Old) -> Vec<(String, i32, i32)> {
        use crate::tuning::{half_up, idx, HP_UNIT, P, SPECS};
        let w = o.id.def();
        let full = SPECS[idx(P::HeavyFullBonus)].def as i32;
        let mut v = Vec::new();
        assert_eq!(w.combo.len(), o.combo.len(), "{:?} combo", o.id);
        for (i, (h, &old)) in w.combo.iter().zip(o.combo).enumerate() {
            v.push((format!("combo {i}"), old, h.damage));
        }
        v.push(("dash".into(), o.dash, w.dash.map_or(0, |h| h.damage)));
        let heavy = w.heavy.map_or(0, |h| h.damage);
        v.push(("heavy".into(), o.heavy, heavy));
        if w.heavy.is_some() {
            v.push(("heavy full".into(), o.heavy + OLD_FULL_BONUS, heavy + full));
            v.push((
                "heavy perfect".into(),
                o.heavy + OLD_FULL_BONUS + OLD_PERFECT,
                heavy + full + HP_UNIT,
            ));
        }
        let shock = w.heavy.and_then(|h| h.aftershock).map_or(0, |a| a.2);
        v.push(("aftershock".into(), o.aftershock, shock));
        let ranged = o.id.is_ranged();
        let (rb, charged) = o.id.bolts();
        v.push(("rb".into(), o.rb, rb.damage));
        let c = charged.damage;
        v.push(("charged".into(), o.charged, c));
        v.push(("half charged".into(), (o.charged + 1) / 2, half_up(c)));
        if ranged {
            v.push(("perfect shot".into(), o.charged + OLD_PERFECT, c + HP_UNIT));
        }
        v.push((
            "grenade".into(),
            o.grenade,
            w.grenade.map_or(0, |g| g.damage),
        ));
        v
    }

    /// 무기와 상관없는 타(이름, 옛, 새): 내려찍기 · 스킬 넷 · 되받은 종 조각 · 큰 기술의 벽 쿵.
    fn other_attacks() -> Vec<(String, i32, i32)> {
        use crate::tuning::{idx, P, SHARD_REFLECT_DAMAGE, SPECS, ULT_SLAM_DAMAGE};
        let mut v = vec![(
            "plunge".to_string(),
            OLD_PLUNGE,
            SPECS[idx(P::PlungeDamage)].def as i32,
        )];
        for (sk, old) in crate::skills::SKILLS.iter().zip(OLD_SKILLS) {
            v.push((format!("skill {:?}", sk.id), old, sk.damage));
        }
        v.push(("shard".into(), OLD_SHARD_REFLECT, SHARD_REFLECT_DAMAGE));
        v.push(("ult slam".into(), OLD_ULT_SLAM, ULT_SLAM_DAMAGE));
        v
    }

    /// 적의 체력들(이름, 옛, 새): 종마다 등급 1, 잡몹은 등급 2·엘리트·등급 2 엘리트까지(옛 셈 = 작은 정수에 올림).
    fn healths() -> Vec<(String, i32, i32)> {
        use crate::enemy::{Affix, Enemy};
        use macroquad::prelude::Vec3;
        let up = |v: i32, k: f32| (v as f32 * k).ceil() as i32;
        let mut v = Vec::new();
        assert_eq!(crate::species::ALL.len(), OLD_SPECIES_HP.len());
        for (d, old) in crate::species::ALL.iter().zip(OLD_SPECIES_HP) {
            v.push((format!("{:?}", d.kind), old, d.hp));
            if d.boss {
                continue;
            }
            let make = |tier: u8, elite: bool| {
                let mut e = Enemy::new(d.kind, Vec3::ZERO);
                e.set_tier(tier);
                if elite {
                    e.make_elite(Affix::Hardened);
                }
                assert_eq!(e.hp, e.max_hp);
                e.max_hp
            };
            v.push((
                format!("{:?} tier 2", d.kind),
                up(old, OLD_TIER2),
                make(2, false),
            ));
            v.push((
                format!("{:?} elite", d.kind),
                up(old, OLD_ELITE),
                make(1, true),
            ));
            v.push((
                format!("{:?} tier 2 elite", d.kind),
                up(up(old, OLD_TIER2), OLD_ELITE),
                make(2, true),
            ));
        }
        v
    }

    /// 0158 N0 판정 기준: 모든 무기의 모든 타 × 모든 적(등급·엘리트 포함)에서 쓰러뜨리는 데 드는 대 수가 옮기기 전과 같다.
    /// 옛 수는 위의 표(작은 정수)에서, 새 수는 지금 표에서 — `ceil(새 체력 / 새 피해) == ceil(옛 체력 / 옛 피해)`.
    #[test]
    fn hits_to_kill_every_enemy_with_every_attack_are_what_they_were_before_the_scale_move() {
        let mut all = other_attacks();
        for o in &OLD_WEAPONS {
            for (name, old, new) in attacks(o) {
                all.push((format!("{:?} {name}", o.id), old, new));
            }
        }
        assert_eq!(
            OLD_WEAPONS.len(),
            WEAPONS.len(),
            "every weapon is in the old table"
        );
        let mut pairs = 0;
        for (foe, old_hp, new_hp) in healths() {
            for (atk, old_dmg, new_dmg) in &all {
                assert_eq!(
                    *old_dmg == 0,
                    *new_dmg == 0,
                    "{atk}: harmless stays harmless"
                );
                if *old_dmg == 0 {
                    continue;
                }
                let (old, new) = (
                    (old_hp + old_dmg - 1) / old_dmg,
                    (new_hp + new_dmg - 1) / new_dmg,
                );
                assert_eq!(new, old, "{atk} on {foe}: {new_dmg} into {new_hp}");
                pairs += 1;
            }
        }
        assert!(pairs > 500, "{pairs}");
    }

    /// 0158 N0: 옮기기만 했다 — 모든 체력·피해는 옛 수의 꼭 열 배다(대 수가 같은 것보다 센 조건: 덤·배율이 섞여도 그대로).
    #[test]
    fn every_health_and_damage_number_is_exactly_ten_times_the_old_one() {
        use crate::tuning::{idx, HP_UNIT, P, SPECS};
        assert_eq!(HP_UNIT, 10);
        for (foe, old, new) in healths() {
            assert_eq!(new, old * HP_UNIT, "{foe}");
        }
        for o in &OLD_WEAPONS {
            for (name, old, new) in attacks(o) {
                assert_eq!(new, old * HP_UNIT, "{:?} {name}", o.id);
            }
        }
        for (name, old, new) in other_attacks() {
            assert_eq!(new, old * HP_UNIT, "{name}");
        }
        assert_eq!(SPECS[idx(P::ParryCrit)].def as i32, OLD_CRIT * HP_UNIT);
    }

    /// 0158 N0: 종 체력·무기 피해·스킬 피해·장신구 보정은 모두 눈금(`HP_UNIT`)의 배수다 — 옛 한 칸보다 잔 수는 아직 없다
    /// (올림 규칙 `scale_up`·`half_up`이 옛 결과를 지키는 바탕. %를 쓰는 성장(G0)이 이 시험을 고친다).
    #[test]
    fn species_health_and_weapon_skill_and_charm_numbers_are_multiples_of_the_hp_unit() {
        use crate::tuning::HP_UNIT;
        let whole = |v: i32, what: &str| assert_eq!(v % HP_UNIT, 0, "{what}: {v}");
        for d in crate::species::ALL {
            whole(d.hp, &format!("{:?} hp", d.kind));
        }
        for w in WEAPONS.iter() {
            for h in w.combo.iter().chain(w.dash.iter()) {
                whole(h.damage, &format!("{:?} {}", w.id, h.anim));
            }
            if let Some(h) = w.heavy {
                whole(h.damage, &format!("{:?} heavy", w.id));
                if let Some((_, _, d, _)) = h.aftershock {
                    whole(d, &format!("{:?} aftershock", w.id));
                }
            }
            let (rb, charged) = w.id.bolts();
            whole(rb.damage, &format!("{:?} rb", w.id));
            whole(charged.damage, &format!("{:?} charged", w.id));
            if let Some(Rhythm::Draw { mid_bolt, .. }) = w.id.rhythm() {
                whole(mid_bolt.damage, &format!("{:?} mid draw", w.id));
            }
            if let Some(g) = w.grenade {
                whole(g.damage, &format!("{:?} grenade", w.id));
            }
        }
        for s in crate::skills::SKILLS.iter() {
            whole(s.damage, &format!("{:?}", s.id));
        }
        for c in crate::charms::TABLE.iter() {
            let m = c.mods;
            for v in [
                m.hp,
                m.heal,
                m.dmg,
                m.taken,
                m.heavy_dmg,
                m.light_dmg,
                m.opened_dmg,
            ] {
                whole(v, &format!("{:?}", c.id));
            }
        }
        whole(crate::tuning::PLAYER_HP, "player hp");
        whole(crate::tuning::ULT_SLAM_DAMAGE, "ult slam");
        whole(crate::tuning::SHARD_REFLECT_DAMAGE, "shard");
    }
}

#[cfg(test)]
mod swing_tests {
    use super::*;
    use crate::det::Det;
    use crate::model::Model;
    use crate::tuning::Tuning;
    use macroquad::prelude::*;

    /// 0193: 판정·초승달은 정면 부채꼴이다 — 모델의 칼끝도 판정 구간 가운데에 정면을 지나고(`wanderer_skin.py` `face_hit`),
    /// 표의 쪽으로 가야 초승달이 칼을 따라간다. 판정 구간은 게임의 셈(`hit_times`·`heavy_times`)을 동작 시계로 옮긴 것(`clip_rate`,
    /// 0195). 맨손(0195)은 칼끝 대신 주먹(`weapon` 뼈)으로 — 검의 동작을 빌려도 주먹이 표의 쪽으로 가야 한다. 대시 공격도.
    #[test]
    fn melee_swings_cross_the_front_in_the_table_direction() {
        let m = Model::from_bytes(include_bytes!("../assets/models/wanderer.glb")).unwrap();
        let t = Tuning::default();
        let grip = m.node("weapon").unwrap();
        let mut rest = Vec::new();
        m.pose(None, 0.0, &mut rest);
        let mut pose = Vec::new();
        for w in WEAPONS.iter().filter(|w| !w.combo.is_empty()) {
            // 끝점: 무기 = 쉬는 자세에서 주먹에서 가장 먼 정점, 맨손 = 주먹.
            let tip: Option<(usize, Vec3)> = (w.id != Id::Fists).then(|| {
                let node = m.node(w.node).unwrap();
                let mesh = &m.meshes[m.nodes[node].mesh.unwrap()];
                let g = rest[grip].w_axis.truncate();
                let far = mesh
                    .pos
                    .iter()
                    .copied()
                    .max_by(|a, b| {
                        let d = |p: Vec3| rest[node].transform_point3(p).distance(g);
                        d(*a).total_cmp(&d(*b))
                    })
                    .unwrap();
                (node, far)
            });
            // (동작, 판정 시간, 들어 올림 — 0284: 대검의 평타는 판정 시작까지만 느린 동작 시계 `clip_at`).
            let mut swings: Vec<(&str, (f32, f32, f32), f32)> = (0..w.combo.len())
                .map(|k| {
                    let h = &w.combo[k];
                    (h.anim, crate::world::hit_times(&t, w, k as u8), h.windup)
                })
                .collect();
            if let Some(d) = w.dash {
                swings.push((d.anim, crate::world::hit_times(&t, w, DASH), d.windup));
            }
            if let Some(h) = w.heavy {
                swings.push((h.anim_heavy, crate::world::heavy_times(&t, w), 1.0));
            }
            for (anim, (s0, a0, _), windup) in swings {
                let ai = m.anim(anim).unwrap_or_else(|| panic!("{anim}"));
                // 0198: 맨손의 왼 잽은 왼손, 앞차기는 오른발이 친다. 0274 ③: 쌍단검 1타도 왼손의 날이 찌른다.
                let limb = match anim {
                    "fist1" | "dagger1" => m.node("mixamorig:LeftHand"),
                    "fist_heavy" | "fist_dash" => m.node("mixamorig:RightFoot"),
                    _ => None,
                };
                let c0 = clip_at(anim, s0, s0, windup);
                let (s0, a0) = (c0, clip_at(anim, s0 + a0, s0, windup) - c0);
                // 끝점의 몸 둘레 각(도 — 정면 0, 왼쪽 +), 구간을 여덟으로 나눠 풀어 이은 것.
                let (mut yaws, mut last) = (Vec::new(), None::<f32>);
                for j in 0..=8 {
                    m.pose(Some(ai), s0 + a0 * j as f32 / 8.0, &mut pose);
                    let p = match (limb, tip) {
                        (Some(l), _) => pose[l].w_axis.truncate(),
                        (None, Some((node, far))) => pose[node].transform_point3(far),
                        (None, None) => pose[grip].w_axis.truncate(),
                    };
                    let mut y = p.x.atan2_d(p.z).to_degrees();
                    if let Some(l) = last {
                        y = l + (y - l + 540.0).rem_euclid(360.0) - 180.0;
                    }
                    last = Some(y);
                    yaws.push(y);
                }
                let mid = (yaws[4] + 540.0).rem_euclid(360.0) - 180.0;
                let span = yaws[8] - yaws[0];
                // 주먹·발(맨손, 앞차기)은 칼끝보다 몸에 가까워 각이 크게 흔들린다 — 가운데는 덜 엄하게, 가로는 45°부터.
                let (front, wide) = if tip.is_some() && limb.is_none() {
                    (20.0, 60.0)
                } else {
                    (35.0, 45.0)
                };
                assert!(
                    mid.abs() < front,
                    "{:?} {anim}: crosses the front mid-swing ({mid:.0}°)",
                    w.id
                );
                let want = if span.abs() < wide {
                    Sweep::Center
                } else if span > 0.0 {
                    Sweep::RightToLeft
                } else {
                    Sweep::LeftToRight
                };
                assert_eq!(sweep_of(anim), want, "{:?} {anim}: goes {span:+.0}°", w.id);
            }
        }
    }

    /// 0195: 동작 시계를 맞추는 구간은 Blender 쪽 표(`HIT_WINDOWS`)와 같다 — 한쪽만 고치면 칼이 닿는 순간과 판정이 어긋난다.
    #[test]
    fn swing_windows_match_the_blender_table() {
        let py = include_str!("../tools/art/blender/wanderer_skin.py");
        let start = py.find("HIT_WINDOWS = {").unwrap();
        let body = &py[start..start + py[start..].find('}').unwrap()];
        let mut n = 0;
        for part in body
            .split("\"")
            .skip(1)
            .step_by(2)
            .zip(body.split('(').skip(1))
        {
            let (name, nums) = part;
            let nums: Vec<f32> = nums
                .split(')')
                .next()
                .unwrap()
                .split(',')
                .map(|x| x.trim().parse().unwrap())
                .collect();
            let (_, _, (c0, ca)) = SWEEPS
                .iter()
                .find(|(a, _, _)| *a == name)
                .unwrap_or_else(|| panic!("{name} is not in SWEEPS"));
            assert_eq!((*c0, *ca), (nums[0], nums[1]), "{name}");
            n += 1;
        }
        assert_eq!(n, SWEEPS.len());
    }

    /// 0195 → 0198: 제 동작은 제 판정 구간으로 구웠으니 1배, 남의 구간으로 구운 동작을 빌린 타(대시 공격)는 동작이 만든 판정 시작이
    /// 그 타의 판정 시작에 오게 빨리(늦게) 돈다.
    #[test]
    fn borrowed_swings_play_faster_so_the_blow_lands_on_the_hit() {
        let t = Tuning::default();
        let fists = Id::Fists.def();
        let rate = |k: u8| {
            let anim = fists.hit(k).unwrap().anim;
            let s0 = crate::world::hit_times(&t, fists, k).0;
            let c0 = SWEEPS.iter().find(|(a, _, _)| *a == anim).unwrap().2 .0;
            let r = clip_rate(anim, s0);
            assert!((s0 * r - c0).abs() < 1e-5, "{anim}: {r}");
            r
        };
        for k in 0..fists.combo.len() as u8 {
            assert!(
                (rate(k) - 1.0).abs() < 0.02,
                "fists' own clips play as baked"
            );
        }
        assert!(
            (rate(DASH) - 1.0).abs() < 0.02,
            "the running kick is baked for the dash (0199)"
        );
        assert_eq!(clip_rate("idle", 0.1), 1.0);
    }
}

#[cfg(test)]
mod look_tests {
    use super::*;
    use crate::touch::Kind as K;

    /// 0278 R17: 꼴은 모두 쓰인다 — 무기 · 맨손 · 방패의 줄(종류의 RB 톡 · 모은 것, 사격의 탄 · 반쯤 당긴 화살, 돌, 방패 던지기) 가운데
    /// 적어도 하나가 그 꼴을 낸다(죽은 갈래가 없다). 옛 수 0(라벤더) · 1(놋쇠)은 낼 줄이 없어 갈래를 두지 않았다.
    #[test]
    fn every_look_is_thrown_by_some_row() {
        let mut used = Vec::new();
        for id in IDS {
            let (a, b) = id.bolts();
            used.extend([a.look, b.look]);
            if let Some(Rhythm::Draw { mid_bolt, .. }) = id.rhythm() {
                used.push(mid_bolt.look);
            }
        }
        for t in [STONE_TOSS, SHIELD_TOSS] {
            used.extend([t.bolt.look, t.charged.look]);
        }
        for l in Look::ALL {
            assert!(used.contains(&l), "{l:?} is used by no row");
        }
    }

    /// 0278 R17: `def()`의 값은 옛 `u8` 술어 · 이름 · 그림과 같다(이행 때 옛 술어와 꼴마다 견준 표를 그대로 박았다 — 옛 수 `code`도).
    #[test]
    fn look_defs_keep_the_old_behaviour() {
        use Flight::{Ground, Smite, Straight};
        let s = crate::i18n::s();
        #[rustfmt::skip]
        let want: [(Look, u8, Flight, bool, bool, bool, bool, &str, K); 12] = [
            // 꼴, 옛 수, 나는 법, 사격, 던짐, 돌아옴, 튕김, 이름, 그림
            (Look::Magic, 2, Ground, false, false, false, false, s.rb_magic, K::Cast),
            (Look::Bolt, 3, Straight, true, false, false, false, s.rb_magic, K::Cast),
            (Look::Arrow, 4, Straight, true, false, false, false, s.rb_magic, K::Cast),
            (Look::Axe, 5, Straight, false, true, false, false, s.rb_axe, K::HandAxe),
            (Look::Light, 6, Smite, false, false, false, false, s.rb_light, K::Smite),
            (Look::Javelin, 7, Straight, false, true, true, false, s.rb_javelin, K::Javelin),
            (Look::Stone, 8, Straight, false, true, false, false, s.rb_stone, K::Pebble),
            (Look::Knife, 9, Straight, false, true, false, false, s.rb_knives, K::Knives),
            (Look::Shield, 10, Straight, false, true, true, true, s.rb_shield, K::ShieldToss),
            (Look::Quake, 11, Ground, false, false, false, false, s.rb_quake, K::Quake2),
            (Look::Seed, 12, Smite, false, false, false, false, s.rb_seed, K::ThornSeed),
            (Look::Curse, 13, Straight, false, false, false, false, s.rb_curse, K::CurseBolt),
        ];
        for (l, code, flight, shot, thrown, returns, bounces, name, icon) in want {
            let d = l.def();
            assert_eq!(l.code(), code, "{l:?}");
            assert_eq!(format!("{l:?}"), code.to_string(), "Debug = the old number");
            assert_eq!(d.flight, flight, "{l:?}");
            assert_eq!(
                (d.shot, d.thrown, d.returns, d.bounces),
                (shot, thrown, returns, bounces),
                "{l:?}"
            );
            assert_eq!((d.name)(s), name, "{l:?}");
            assert_eq!(d.icon, icon, "{l:?}");
        }
        let mut codes: Vec<u8> = Look::ALL.iter().map(|l| l.code()).collect();
        codes.dedup();
        assert_eq!(codes.len(), Look::ALL.len(), "codes are distinct");
    }
}
