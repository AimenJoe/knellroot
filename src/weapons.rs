//! 무기 표(0051). 무기 = 공격의 **모양**(KB `concepts/action-rpg-stats-and-equipment.md`): 콤보의 타마다 시간·거리·각·피해·자세 피해,
//! 강타, RB(근접 무기 = 마법 탄, 원거리 무기 = 사격), 원거리 무기의 X(수류탄). 0048: 무기 한 줄 = (X, RB) 짝.
//!
//! 시간·거리·각·내딛기는 랩의 기준값(`attack.*`·`heavy.*`, `tuning.rs`)에 곱하는 **배율** — 손맛의 바탕은 랩 한 곳에서 맞추고,
//! 무기는 그 위의 모양만 갖는다(코드에 타별 숫자를 박지 않는다). 피해·자세 피해는 정수·점수 그대로(피해의 눈금은 `tuning::HP_UNIT` — 옛 1 = 10, 0158 N0).
//! 새 무기 = `Id` 한 줄 + `WEAPONS` 한 줄 + 동작(`tools/art/blender/wanderer.py`) + 모델 부위(같은 파일, 오른손 뼈) + 글(i18n).

use crate::i18n::Tr;
use crate::skills::Ult;
use crate::stats::Mods;
use crate::status::Kind;

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
pub const SWEEPS: [(&str, Sweep, (f32, f32)); 24] = [
    ("attack", Sweep::RightToLeft, (0.08, 0.10)),
    ("attack2", Sweep::LeftToRight, (0.08, 0.10)),
    ("attack3", Sweep::Center, (0.10, 0.12)),
    ("heavy", Sweep::LeftToRight, (0.06, 0.12)),
    ("hammer1", Sweep::RightToLeft, (0.14, 0.17)),
    ("hammer2", Sweep::Center, (0.16, 0.20)),
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
    ("spear1", Sweep::Center, (0.10, 0.12)),
    ("spear2", Sweep::Center, (0.10, 0.12)),
    ("spear3", Sweep::Center, (0.12, 0.14)),
    ("spear_dash", Sweep::Center, (0.10, 0.12)),
    ("spear_lunge", Sweep::Center, (0.08, 0.16)),
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

/// 콤보 한 타.
#[derive(Clone, Copy, Debug)]
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
}

/// RB.
#[derive(Clone, Copy, Debug)]
pub enum Rb {
    /// 근접 무기의 마법 탄(0048: 칸 3, 근접으로 맞히면 찬다).
    Magic(Bolt),
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
    },
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
    /// 그림 색(0 = 마법 라벤더, 1 = 무거운 탄 놋쇠, 2 = 꿰뚫는 탄 민트, 3 = 쇠뇌 볼트, 4 = 활의 화살 — 0231).
    pub look: u8,
    /// 크기 배율(그림·맞는 반지름, 모은 탄은 크다 — 0056).
    pub scale: f32,
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
}

/// 표 순서. 맨손은 옛 끝에(옛 기록의 무기 번호가 그대로 읽히게 — `app::legacy_weapon`), 그 뒤에 새 무기.
pub const IDS: [Id; 7] = [
    Id::Sword,
    Id::Hammer,
    Id::Staff,
    Id::Crossbow,
    Id::Fists,
    Id::Bow,
    Id::Spear,
];

impl Id {
    /// 기록 키(0066 — 든 무기는 슬롯 기록에, 키는 바꾸지 않는다).
    pub fn key(self) -> &'static str {
        match self {
            Id::Sword => "sword",
            Id::Hammer => "hammer",
            Id::Staff => "staff",
            Id::Crossbow => "crossbow",
            Id::Fists => "fists",
            Id::Bow => "bow",
            Id::Spear => "spear",
        }
    }

    pub fn by_key(k: &str) -> Option<Id> {
        IDS.into_iter().find(|i| i.key() == k)
    }
}

pub struct Weapon {
    pub id: Id,
    /// X 연타(근접 무기). 원거리 무기는 비어 있고 X가 수류탄.
    pub combo: &'static [Hit],
    /// 대시 공격(2026-11-06): 회피 끝무렵·직후에 공격 — 회피의 기세로 멀리 내딛는 한 타. 맞힌 뒤 콤보는 둘째 타부터.
    pub dash: Option<Hit>,
    pub heavy: Option<Heavy>,
    pub rb: Rb,
    pub grenade: Option<Grenade>,
    /// RB를 쥐어 다 모은 탄(0056): 근접 무기 = 모은 마법, 원거리 무기 = 모은 사격. 덜 모았으면 피해·자세 피해 반, 꿰뚫기·휘청 없음.
    pub charged: Bolt,
    /// 방랑자 모델에서 이 무기 부위 이름(다른 무기 부위는 숨긴다).
    pub node: &'static str,
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

/// 활의 화살 그림(`Bolt::look`, 0231) — 원거리 무기의 탄(곧게 난다·벼린 몫을 받는다)이다.
pub const ARROW: u8 = 4;

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
    /// 쇠뇌의 볼트인가(0180 — 벼린 쇠뇌의 피해 몫을 받는다. 근접 무기의 마법 탄은 아니다).
    pub fn is_shot(&self) -> bool {
        self.look == 3 || self.look == ARROW
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
    look: 0,
    scale: 1.0,
};

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

/// 종치기 망치 — 느리고 넓은 2타(한 걸음씩 나아감, 자세 피해 큼), 강타는 뛰어올라 둥근 내려찍기 + 여진.
const HAMMER_COMBO: [Hit; 2] = [
    Hit {
        knock: 1.5,
        cancel: 0.4,
        ..hit(1.7, 1.15, 1.25, 1.4, 20, 3.0, "hammer1")
    },
    Hit {
        shape: Shape::Circle,
        knock: 2.0,
        cancel: 0.5,
        ..hit(2.0, 1.1, 1.0, 1.2, 30, 4.0, "hammer2")
    },
];

/// 순례자 지팡이 — 빠른 4연타(0203 — 올려 베기 → 내려 베기 → 머리 위에서 내려치기), 넷째는 두 손으로 내려쳐 둘레를 친다(느림).
const STAFF_COMBO: [Hit; 4] = [
    hit(0.7, 1.15, 0.8, 0.7, 10, 0.6, "staff1"),
    hit(0.7, 1.15, 0.8, 0.7, 10, 0.6, "staff2"),
    hit(0.75, 1.2, 0.9, 0.8, 10, 0.8, "staff3"),
    Hit {
        shape: Shape::Circle,
        knock: 1.8,
        ..hit(1.3, 1.25, 1.0, 0.4, 20, 2.0, "staff4")
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

pub const WEAPONS: [Weapon; 7] = [
    Weapon {
        id: Id::Sword,
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
            aftershock: None,
            breaks_primed: None,
            anim_charge: "charge",
            anim_heavy: "heavy",
            cancel: 0.2,
        }),
        rb: Rb::Magic(MAGIC),
        grenade: None,
        // 모은 마법: 크고 꿰뚫는 빛 덩이.
        charged: Bolt {
            speed: 1.2,
            range: 1.3,
            damage: 30,
            poise: 4.0,
            pierce: true,
            stagger: 0.6,
            scale: 1.8,
            ..MAGIC
        },
        node: "sword",
        stamina: 1.0,
        forge: Some(MELEE_FORGE),
    },
    Weapon {
        id: Id::Hammer,
        combo: &HAMMER_COMBO,
        // 달려들며 몸 전체로 내려치기(0203 — 0200의 앞차기는 발이 작아 안 보였다, 사장): 웅크리며 땅을 쳐 둘레를 친다(앞차기 때 앞
        // 부채꼴로 바꾼 것을 되돌림). 0200 전엔 둘째 타의 한 바퀴 내려치기를 빌려 달리기 공격 뒤 같은 내려치기를 두 번 했다.
        dash: Some(Hit {
            shape: Shape::Circle,
            knock: 1.8,
            cancel: 0.45,
            ..hit(1.6, 1.0, 1.0, 2.0, 20, 3.0, "hammer_dash")
        }),
        heavy: Some(Heavy {
            time: 1.3,
            charge: 1.4,
            reach: 1.0,
            arc: 1.0,
            lunge: 2.2,
            shape: Shape::Circle,
            damage: 30,
            poise: 6.0,
            walk: 0.75,
            aftershock: Some((1.5, 2.6, 20, 3.0)),
            breaks_primed: Some(0.6),
            anim_charge: "hammer_charge",
            anim_heavy: "hammer_slam",
            cancel: 0.55,
        }),
        rb: Rb::Magic(Bolt {
            speed: 0.65,
            range: 0.6,
            damage: 20,
            poise: 3.0,
            stagger: 0.5,
            look: 1,
            ..MAGIC
        }),
        grenade: None,
        // 모은 무거운 탄: 느리고 크다, 맞으면 오래 휘청·자세 피해 큼.
        charged: Bolt {
            speed: 0.6,
            range: 0.8,
            damage: 40,
            poise: 8.0,
            stagger: 1.0,
            look: 1,
            scale: 2.2,
            ..MAGIC
        },
        node: "w_hammer",
        stamina: 1.7,
        forge: Some(MELEE_FORGE),
    },
    Weapon {
        id: Id::Staff,
        combo: &STAFF_COMBO,
        // 달려들며 올려 베기(0203 — 1타 동작이라 다음 누름의 2타로 원본 그대로 이어진다): 빠르고 가볍다.
        dash: Some(hit(0.8, 1.3, 0.6, 2.2, 10, 1.0, "staff1")),
        heavy: Some(Heavy {
            time: 0.9,
            charge: 0.9,
            reach: 1.0,
            arc: 1.0,
            lunge: 0.5,
            shape: Shape::Circle,
            damage: 20,
            poise: 2.5,
            walk: 1.2,
            aftershock: None,
            breaks_primed: None,
            anim_charge: "staff_charge",
            anim_heavy: "staff_release",
            cancel: 0.0,
        }),
        rb: Rb::Magic(Bolt {
            speed: 1.3,
            range: 1.2,
            pierce: true,
            look: 2,
            ..MAGIC
        }),
        grenade: None,
        // 모은 꿰뚫는 탄: 빠르고 멀리, 줄 선 적을 다 꿰뚫는다.
        charged: Bolt {
            speed: 1.6,
            range: 1.7,
            damage: 30,
            poise: 3.0,
            pierce: true,
            look: 2,
            scale: 1.5,
            ..MAGIC
        },
        node: "w_staff",
        stamina: 0.75,
        forge: Some(MELEE_FORGE),
    },
    Weapon {
        id: Id::Crossbow,
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
        rb: Rb::Shot {
            bolt: Bolt {
                speed: 1.7,
                range: 1.4,
                damage: 10,
                poise: 1.0,
                pierce: false,
                stagger: 0.0,
                look: 3,
                scale: 1.0,
            },
            mag: 4,
            reload: 2.4,
            time: 1.15,
            heavy: 2,
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
        // 모은 사격: 시위를 끝까지 당긴 무거운 볼트 — 꿰뚫고 밀어낸다(수류탄도 맞히면 터진다).
        // 0243(사장 설문 k09): 30 → 20 — 모아 쏘기만 되풀이하면 오래 17.5/초(검의 가장 센 꼴 23의 여덟 할 가까이)였다.
        charged: Bolt {
            speed: 2.2,
            range: 1.8,
            damage: 20,
            poise: 6.0,
            pierce: true,
            stagger: 0.4,
            look: 3,
            scale: 1.6,
        },
        node: "w_crossbow",
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
            aftershock: None,
            breaks_primed: None,
            anim_charge: "fist_charge",
            anim_heavy: "fist_heavy",
            cancel: 0.1,
        }),
        // 마법은 무기가 아니라 방랑자의 것 — 맨손이어도 쏜다.
        rb: Rb::Magic(MAGIC),
        grenade: None,
        charged: Bolt {
            speed: 1.2,
            range: 1.3,
            damage: 30,
            poise: 4.0,
            pierce: true,
            stagger: 0.6,
            scale: 1.8,
            ..MAGIC
        },
        // 든 것이 없다(다른 무기 부위는 모두 숨는다).
        node: "",
        stamina: 0.6,
        forge: None,
    },
    // 잔뿌리 활(0231): 쇠뇌와 같은 틀(평타 = 사격, 꾹 = 모은 사격, 특수 = 던지는 것)에서 손맛을 가른다 —
    // 화살은 가볍고 빠르고 멀리(전통 가득 6 · 다시 채우기 짧음 · 한 발 빠름), 당겨 쏘는 한 발은 화살 하나로(쇠뇌는 둘) 꿰뚫고,
    // 특수는 터지는 화약이 아니라 **뿌리 씨앗** — 피해는 작고 둘레의 적을 묶는다(뿌리 게이지 가득). 쇠뇌 = 한 방·떼 지우기,
    // 활 = 거리·잦은 손·묶어 두고 쏘기. 수치는 처음 값 — 원거리 초당 피해는 근접의 여섯~일곱 할 밑(0173, 시험).
    Weapon {
        id: Id::Bow,
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
                look: ARROW,
                scale: 1.0,
            },
            mag: 5,
            reload: 2.2,
            time: 1.3,
            heavy: 1,
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
        // 당겨 쏘는 한 발: 끝까지 당긴 화살 — 빠르고 멀리, 줄 선 적을 꿰뚫고 휘청이게 한다. 피해는 화살 그대로(10) — 화살 하나로
        // 당기니 전통이 길어 모아 쏘기만 되풀이하면 20이던 처음 값이 24/초(검의 가장 센 꼴 23의 여덟 할 18.4를 넘었다, 탐침
        // `plan_dps_probe`). 딱 맞춰 떼면 덤 +10은 그대로라 "당겨서 맞히는" 맛은 남는다.
        charged: Bolt {
            speed: 2.4,
            range: 2.1,
            damage: 10,
            poise: 3.0,
            pierce: true,
            stagger: 0.35,
            look: ARROW,
            scale: 1.4,
        },
        node: "w_bow",
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
    // 앞에서 멈춘다, `World::lunge_stops`). RB는 다른 근접 무기처럼 마법 탄.
    Weapon {
        id: Id::Spear,
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
            aftershock: None,
            breaks_primed: None,
            anim_charge: "spear_charge",
            anim_heavy: "spear_lunge",
            cancel: 0.3,
        }),
        rb: Rb::Magic(MAGIC),
        grenade: None,
        charged: Bolt {
            speed: 1.2,
            range: 1.3,
            damage: 30,
            poise: 4.0,
            pierce: true,
            stagger: 0.6,
            scale: 1.8,
            ..MAGIC
        },
        node: "w_spear",
        // 검(1.0)과 망치(1.7) 사이.
        stamina: 1.25,
        forge: Some(MELEE_FORGE),
    },
];

/// 무기 종류(0272): 계열(`Hand` — 화면에 드러내지 않는 묶음) → **종류** → 낱낱의 무기(`Id`). 판 · 물리 속성 · 주특성은 종류에,
/// 숫자 · 이름 · 생김새는 낱낱의 무기에. 새 종류 = 여기 한 줄 + `index` 한 갈래 + `CLASSES` 한 줄(+ 글).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    Sword,
    Hammer,
    Staff,
    Crossbow,
    Bow,
    Spear,
}

/// 무기의 주특성이 스킬에 얹는 배(0225 → 0272 — 종류마다): `force` = 타입 없는(물리) 스킬의 피해·자세 피해, `channel` = 스킬의 타가
/// 쌓는 타입 게이지.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Art {
    pub force: f32,
    pub channel: f32,
}

impl Art {
    /// 배 없음(한손검·쇠뇌·활·맨손).
    pub const PLAIN: Art = Art {
        force: 1.0,
        channel: 1.0,
    };

    /// 주특성 글의 `{0}`에 넣는 수(%): 둘 가운데 큰 배의 덤.
    pub fn percent(self) -> i32 {
        ((self.force.max(self.channel) - 1.0) * 100.0).round() as i32
    }
}

/// 망치의 물리력 · 지팡이의 마법력(0225 — 처음 값, 판정에서 고친다).
pub const ART_FORCE: f32 = 1.3;
pub const ART_CHANNEL: f32 = 1.5;

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
        // 최대 기력은 더하는 칸이라 바탕(랩 `stamina.max`)에 대한 몫으로.
        let base = crate::tuning::SPECS[crate::tuning::idx(crate::tuning::P::StaminaMax)].def;
        let d = [m.stamina_regen, m.poise, m.ail, m.skill_cd]
            .into_iter()
            .map(|x| (x - 1.0).abs())
            .chain([m.stamina_max / base])
            .fold(0.0, f32::max);
        (d * 100.0).round() as i32
    }
}

/// 보조특성의 값(0272 1b — 처음 값, 판정에서 고친다): 한손검 = 기력 회복 · 망치 = 자세 피해 · 지팡이 = 속성 게이지 · 활 = 스킬 쿨타임.
/// 기력 회복·자세 피해는 보스 모의(`balance::sub_knob_probe` 300판)로 골랐다 — 어느 몸의 싸움도 4% 넘게 줄지 않게: 둘 다 문턱이
/// 있는 값이다(기력 회복 ×1.05부터 망치가 문지기의 틈마다 한 타를 더 넣어 −11% · 자세 피해 ×1.10이면 검이 문지기의 자세를 한 타
/// 먼저 깨 −6%). 속성 게이지·스킬 쿨타임은 봇이 스킬을 쓰지 않아 재지 못했다(셈으로만 — 0272 "1b 만든 것").
pub const SUB_REGEN: f32 = 1.04;
pub const SUB_POISE: f32 = 1.05;
pub const SUB_AIL: f32 = 1.15;
pub const SUB_SKILL_CD: f32 = 0.95;
/// 창의 보조특성(0272 2단계): 최대 기력 +5(바탕 100의 5%) — 보스 모의(`balance::sub_trait_fight_length`)로 잰다.
pub const SUB_STAMINA: f32 = 5.0;

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
}

/// 종류 표(판 줄의 차례 — `plates::PLATES`가 이 차례로 선다). 대검 · 창은 뒤에 한 줄씩(0272 2단계).
pub const CLASSES: [ClassDef; 6] = [
    ClassDef {
        class: Class::Sword,
        hand: Hand::One,
        name: |s| s.class_sword,
        attribute: |s| s.attr_slash,
        pair: [Kind::Sharp, Kind::Rift],
        great: Ult::Flurry,
        art: Art::PLAIN,
        main_trait: |s| s.trait_sword,
        sub: SubTrait {
            mods: Mods {
                stamina_regen: SUB_REGEN,
                ..Mods::NONE
            },
            stowed_reload: false,
        },
        sub_trait: |s| s.subtrait_sword,
    },
    ClassDef {
        class: Class::Hammer,
        hand: Hand::Two,
        name: |s| s.class_hammer,
        attribute: |s| s.attr_strike,
        pair: [Kind::Shock, Kind::Knell],
        great: Ult::Upend,
        art: Art {
            force: ART_FORCE,
            channel: 1.0,
        },
        main_trait: |s| s.trait_hammer,
        sub: SubTrait {
            mods: Mods {
                poise: SUB_POISE,
                ..Mods::NONE
            },
            stowed_reload: false,
        },
        sub_trait: |s| s.subtrait_hammer,
    },
    ClassDef {
        class: Class::Staff,
        hand: Hand::Two,
        name: |s| s.class_staff,
        attribute: |s| s.attr_magic,
        pair: [Kind::Frost, Kind::Ember],
        great: Ult::Blizzard,
        art: Art {
            force: 1.0,
            channel: ART_CHANNEL,
        },
        main_trait: |s| s.trait_staff,
        sub: SubTrait {
            mods: Mods {
                ail: SUB_AIL,
                ..Mods::NONE
            },
            stowed_reload: false,
        },
        sub_trait: |s| s.subtrait_staff,
    },
    ClassDef {
        class: Class::Crossbow,
        hand: Hand::Ranged,
        name: |s| s.class_crossbow,
        attribute: |s| s.attr_pierce,
        pair: [Kind::Ember, Kind::Shock],
        great: Ult::Line,
        art: Art::PLAIN,
        main_trait: |s| s.trait_crossbow,
        sub: SubTrait {
            mods: Mods::NONE,
            stowed_reload: true,
        },
        sub_trait: |s| s.subtrait_crossbow,
    },
    ClassDef {
        class: Class::Bow,
        hand: Hand::Ranged,
        name: |s| s.class_bow,
        attribute: |s| s.attr_pierce,
        pair: [Kind::Root, Kind::Frost],
        great: Ult::Rain,
        art: Art::PLAIN,
        main_trait: |s| s.trait_bow,
        sub: SubTrait {
            mods: Mods {
                skill_cd: SUB_SKILL_CD,
                ..Mods::NONE
            },
            stowed_reload: false,
        },
        sub_trait: |s| s.subtrait_bow,
    },
    // 창(0272 2단계): 양손 근접 · 관통 · 뿌리와 날카로움. 주특성은 무브셋 그 자체(가장 긴 거리 · 한 줄 꿰뚫기) — 스킬에 얹는 배는 없다.
    ClassDef {
        class: Class::Spear,
        hand: Hand::Two,
        name: |s| s.class_spear,
        attribute: |s| s.attr_pierce,
        pair: [Kind::Root, Kind::Sharp],
        great: Ult::Gale,
        art: Art::PLAIN,
        main_trait: |s| s.trait_spear,
        sub: SubTrait {
            mods: Mods {
                stamina_max: SUB_STAMINA,
                ..Mods::NONE
            },
            stowed_reload: false,
        },
        sub_trait: |s| s.subtrait_spear,
    },
];

impl Class {
    /// 표의 자리(`CLASSES` — 스킬의 `hands` 비트 · 판의 차례가 이것을 쓴다). `_` 없는 match — 새 종류는 여기서 자리를 받는다.
    pub const fn index(self) -> usize {
        match self {
            Class::Sword => 0,
            Class::Hammer => 1,
            Class::Staff => 2,
            Class::Crossbow => 3,
            Class::Bow => 4,
            Class::Spear => 5,
        }
    }

    pub fn def(self) -> &'static ClassDef {
        &CLASSES[self.index()]
    }
}

/// 손에 쥔 꼴(0225 → 0272): 스킬과 판이 "지금 무엇을 쥐었나"를 묻는 한 곳. 종류는 `Id::class` 하나에서 오고, 이 꼴은 거기에
/// **방패를 들었나**만 더한다 — 한손 계열과 방패 = 방패 판의 기술(`Shield`), 그 밖에는 그 종류의 판(`Class`), 맨손 = 무기 판 없음(`Bare`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Grip {
    /// 맨손, 방패 없이 — 무기 판이 없다(공통 판만).
    Bare,
    /// 그 종류의 무기(한손 종류는 방패 없이).
    Class(Class),
    /// 한손 계열(한손 종류 · 맨손)과 방패 — 방패 판.
    Shield,
}

impl Id {
    /// 무기의 종류(0272). `_` 없는 match — 새 무기는 여기서 종류를 고르게 된다. 맨손은 종류가 아니다(무기 칸을 비운 것).
    pub fn class(self) -> Option<Class> {
        match self {
            Id::Sword => Some(Class::Sword),
            Id::Hammer => Some(Class::Hammer),
            Id::Staff => Some(Class::Staff),
            Id::Crossbow => Some(Class::Crossbow),
            Id::Bow => Some(Class::Bow),
            Id::Spear => Some(Class::Spear),
            Id::Fists => None,
        }
    }

    /// 계열(0223 → 0272): 종류의 것 — 맨손은 한손.
    pub fn hand(self) -> Hand {
        self.class().map_or(Hand::One, |c| c.def().hand)
    }

    /// 손에 쥔 꼴(0225 → 0272): 한손 계열은 왼손의 방패로 갈린다 — 방패와 함께면 방패 판(`Shield`), 아니면 제 종류의 판.
    /// 스킬이 이것으로 쓸 수 있나를 가른다(`skills::Skill::fits`), 판 줄의 금빛 점도 이것(`plates::Plate::held`).
    pub fn grip(self, shield: bool) -> Grip {
        match self.class() {
            _ if shield && self.hand() == Hand::One => Grip::Shield,
            Some(c) => Grip::Class(c),
            None => Grip::Bare,
        }
    }

    /// 무기의 주특성이 스킬에 얹는 배(0225 — 사장 "망치는 물리력을, 지팡이는 마법력에 대한 특색" → 0272 종류의 것).
    pub fn art(self) -> Art {
        self.class().map_or(Art::PLAIN, |c| c.def().art)
    }

    /// 보조무기 슬롯에 멨을 때의 보조특성(0272 1b) — 맨손은 종류가 아니라 없다.
    pub fn sub_trait(self) -> Option<SubTrait> {
        self.class().map(|c| c.def().sub)
    }

    /// 교체 공격으로 들어올 때의 한 타(0272 1b): 대시 공격이 있으면 그것, 사격 무기면 한 발.
    pub fn entry(self) -> Option<Entry> {
        let w = self.def();
        match w.rb {
            _ if w.dash.is_some() => Some(Entry::Dash),
            Rb::Shot { .. } => Some(Entry::Shot),
            Rb::Magic(_) => None,
        }
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

    /// 0272: 종류 표 — 줄의 자리 = `Class::index`, 무기마다 종류 하나(맨손만 없다), 계열·주특성의 배는 종류의 것.
    /// 대표 타입 둘은 서로 다른 타입이고, 주특성의 수는 배에서 나온다(망치 +30% · 지팡이 +50% · 나머지는 배가 없다).
    #[test]
    fn every_weapon_has_a_class_and_the_class_table_is_in_order() {
        for (i, c) in CLASSES.iter().enumerate() {
            assert_eq!(c.class.index(), i, "{:?}", c.class);
            assert_eq!(c.class.def().class, c.class);
            let [a, b] = c.pair;
            assert!(a != b && a != Kind::Physical && b != Kind::Physical);
            assert!(c.art.force >= 1.0 && c.art.channel >= 1.0);
        }
        let greats: Vec<Ult> = CLASSES.iter().map(|c| c.great).collect();
        for (i, g) in greats.iter().enumerate() {
            assert!(!greats[i + 1..].contains(g), "{g:?} on two classes");
        }
        for id in IDS {
            assert_eq!(id.class().is_none(), id == Id::Fists, "{id:?}");
            // 원거리 계열 = 사격 무기(표의 `rb`와 종류 표의 계열이 어긋나지 않는다).
            assert_eq!(id.hand() == Hand::Ranged, id.is_ranged(), "{id:?}");
        }
        assert_eq!(Id::Sword.class(), Some(Class::Sword));
        assert_eq!(Id::Crossbow.class(), Some(Class::Crossbow));
        assert_eq!(Id::Bow.class(), Some(Class::Bow));
        assert_eq!(Id::Fists.hand(), Hand::One, "bare hands can hold a shield");
        assert_eq!(Id::Fists.art(), Art::PLAIN);
        assert_eq!(
            (Id::Sword.hand(), Id::Hammer.hand()),
            (Hand::One, Hand::Two)
        );
        assert_eq!(Id::Hammer.art().force, ART_FORCE);
        assert_eq!(Id::Staff.art().channel, ART_CHANNEL);
        assert_eq!(Class::Hammer.def().art.percent(), 30);
        assert_eq!(Class::Staff.def().art.percent(), 50);
        assert_eq!(Class::Bow.def().art.percent(), 0);
        // 쥔 꼴: 한손 계열만 방패로 갈린다.
        assert_eq!(Id::Sword.grip(false), Grip::Class(Class::Sword));
        assert_eq!(Id::Sword.grip(true), Grip::Shield);
        assert_eq!(Id::Fists.grip(false), Grip::Bare);
        assert_eq!(Id::Fists.grip(true), Grip::Shield);
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
                got.stamina_max != none.stamina_max,
            ];
            let n = knobs.iter().filter(|k| **k).count();
            assert_eq!(n + c.sub.stowed_reload as usize, 1, "{:?}", c.class);
            let only = Stats {
                stamina_regen: none.stamina_regen,
                poise: none.poise,
                ail: none.ail,
                skill_cd: none.skill_cd,
                stamina_max: none.stamina_max,
                ..got
            };
            assert_eq!(only, none, "{:?}: nothing else moves", c.class);
            assert!(got.stamina_regen >= 1.0 && got.poise >= 1.0 && got.ail >= 1.0);
            assert!(got.skill_cd <= 1.0 && got.stamina_max >= 0.0);
            // 작게: 어느 것도 15%를 넘지 않고, 주특성의 배가 있는 종류는 그 반 아래.
            let p = c.sub.percent();
            assert_eq!(p == 0, c.sub.stowed_reload, "{:?}", c.class);
            assert!(p <= 15, "{:?}: {p}%", c.class);
            if c.art.percent() > 0 {
                assert!(p * 2 <= c.art.percent(), "{:?}", c.class);
            }
        }
        assert_eq!(Class::Sword.def().sub.percent(), 4);
        assert_eq!(Class::Hammer.def().sub.percent(), 5);
        assert_eq!(Class::Staff.def().sub.percent(), 15);
        assert_eq!(Class::Bow.def().sub.percent(), 5);
        assert_eq!(Class::Spear.def().sub.percent(), 5, "max stamina +5 on 100");
        assert!(Class::Crossbow.def().sub.stowed_reload);
        assert_eq!(Id::Fists.sub_trait(), None, "bare hands are no class");
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

    #[test]
    fn every_id_has_one_row_and_melee_weapons_have_a_combo() {
        for id in IDS {
            assert_eq!(WEAPONS.iter().filter(|w| w.id == id).count(), 1, "{id:?}");
            let w = id.def();
            match w.rb {
                Rb::Magic(_) => assert!(
                    !w.combo.is_empty() && w.grenade.is_none(),
                    "{id:?}: melee = X combo + RB magic"
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
    const OLD_WEAPONS: [Old; 7] = [
        Old {
            id: Id::Sword,
            combo: &[1, 1, 2],
            dash: 2,
            heavy: 2,
            aftershock: 0,
            rb: 1,
            charged: 3,
            grenade: 0,
        },
        Old {
            id: Id::Hammer,
            combo: &[2, 3],
            dash: 2,
            heavy: 3,
            aftershock: 2,
            rb: 2,
            charged: 4,
            grenade: 0,
        },
        Old {
            id: Id::Staff,
            combo: &[1, 1, 1, 2],
            dash: 1,
            heavy: 2,
            aftershock: 0,
            rb: 1,
            charged: 3,
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
            rb: 1,
            charged: 3,
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
            rb: 1,
            charged: 3,
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
        let (rb, ranged) = match w.rb {
            Rb::Magic(b) => (b.damage, false),
            Rb::Shot { bolt, .. } => (bolt.damage, true),
        };
        v.push(("rb".into(), o.rb, rb));
        let c = w.charged.damage;
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
            let rb = match w.rb {
                Rb::Magic(b) => b.damage,
                Rb::Shot { bolt, .. } => bolt.damage,
            };
            whole(rb, &format!("{:?} rb", w.id));
            whole(w.charged.damage, &format!("{:?} charged", w.id));
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
            let mut swings: Vec<(&str, (f32, f32, f32))> = (0..w.combo.len())
                .map(|k| (w.combo[k].anim, crate::world::hit_times(&t, w, k as u8)))
                .collect();
            if let Some(d) = w.dash {
                swings.push((d.anim, crate::world::hit_times(&t, w, DASH)));
            }
            if let Some(h) = w.heavy {
                swings.push((h.anim_heavy, crate::world::heavy_times(&t, w)));
            }
            for (anim, (s0, a0, _)) in swings {
                let ai = m.anim(anim).unwrap_or_else(|| panic!("{anim}"));
                // 0198: 맨손의 왼 잽은 왼손, 앞차기는 오른발이 친다.
                let limb = match anim {
                    "fist1" => m.node("mixamorig:LeftHand"),
                    "fist_heavy" | "fist_dash" => m.node("mixamorig:RightFoot"),
                    _ => None,
                };
                let r = clip_rate(anim, s0);
                let (s0, a0) = (s0 * r, a0 * r);
                // 끝점의 몸 둘레 각(도 — 정면 0, 왼쪽 +), 구간을 여덟으로 나눠 풀어 이은 것.
                let (mut yaws, mut last) = (Vec::new(), None::<f32>);
                for j in 0..=8 {
                    m.pose(Some(ai), s0 + a0 * j as f32 / 8.0, &mut pose);
                    let p = match (limb, tip) {
                        (Some(l), _) => pose[l].w_axis.truncate(),
                        (None, Some((node, far))) => pose[node].transform_point3(far),
                        (None, None) => pose[grip].w_axis.truncate(),
                    };
                    let mut y = p.x.atan2(p.z).to_degrees();
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
