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

use crate::progression::{Branch, Cell, Key, Link, Tree, Vein, DEPTH, KEYS, LINKS, VEINS};
use crate::status::{Dose, Kind};
use crate::tuning::{HP_UNIT, ULT_SLAM_DAMAGE};
use crate::weapons::{Class, Grip};

/// 스킬이 "무엇을 들었을 때 되나"(0223 V3 → 0225 → 0272): 비트의 합 — 무기 종류마다 하나(`class_bit`)와 방패(`SHIELD` — 한손 계열과
/// 방패를 함께 든 꼴). 한손 종류의 비트는 "방패 없이"다: 방패를 들면 방패 판의 기술만 쓴다.
pub type Hands = u16;
/// 종류 하나의 비트(`Class::index`의 자리 — 새 종류는 표에 한 줄이면 비트가 생긴다).
pub const fn class_bit(c: Class) -> Hands {
    1 << c.index()
}
/// 한손검, 방패 없이.
pub const SWORD: Hands = class_bit(Class::Sword);
pub const HAMMER: Hands = class_bit(Class::Hammer);
pub const STAFF: Hands = class_bit(Class::Staff);
pub const CROSSBOW: Hands = class_bit(Class::Crossbow);
pub const BOW: Hands = class_bit(Class::Bow);
pub const SPEAR: Hands = class_bit(Class::Spear);
/// 쌍단검(0274 ③ — 두 손에 날 하나씩: 방패와 들지 않는다).
pub const DAGGER: Hands = class_bit(Class::Dagger);
/// 대검(0274 ④ — 두 손으로 드는 긴 날: 방패와 들지 않는다).
pub const GREATSWORD: Hands = class_bit(Class::Greatsword);
/// 한손 계열과 방패(방패 판의 기술).
pub const SHIELD: Hands = 1 << 15;
/// 묶음: 한손(방패가 있든 없든 — 0274부터 망치도) · 양손 근접(지팡이 · 창 · 쌍단검 · 대검 — 두 손이 다 찬다) · 근접 · 원거리 · 어느 것이나.
pub const ONE: Hands = SWORD | HAMMER | SHIELD;
pub const TWO: Hands = STAFF | SPEAR | DAGGER | GREATSWORD;
pub const MELEE: Hands = ONE | TWO;
pub const RANGED: Hands = CROSSBOW | BOW;
pub const ANY: Hands = MELEE | RANGED;

/// 쥔 꼴 하나의 비트 — 맨손은 어느 무기 판의 기술도 쓰지 못한다(0272: 무기 판이 없다).
pub const fn grip_bit(g: Grip) -> Hands {
    match g {
        Grip::Bare => 0,
        Grip::Class(c) => class_bit(c),
        Grip::Shield => SHIELD,
    }
}

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
    /// 바라보는 쪽으로 `dist` m를 `time`초에 가로지르며 길(반 폭 `half` m) 위의 적을 한 번씩 친다. 적의 몸은 지나간다 —
    /// `push` m 바깥으로 밀어 내며(방패 돌진, 0270 C3 — 가로지르기는 0).
    Lane {
        dist: f32,
        half: f32,
        time: f32,
        push: f32,
    },
    /// 앞 부채꼴(`range` m · `arc_deg`°)의 적을 방랑자 앞 `to` m 한 점으로 끌어 모으고 `stagger`초 휘청이게 한다.
    /// 보스·엘리트는 끌리지 않는다.
    Pull {
        range: f32,
        arc_deg: f32,
        to: f32,
        stagger: f32,
    },
    /// 앞 부채꼴(`range` m · `arc_deg`°)을 내리찍는다 — 맞은 적은 `stagger`초 휘청(보스는 휘청이지 않는다), `push` m 바깥으로
    /// 밀린다(후려치기, 0270 C3 — 땅울림은 0).
    Cone {
        range: f32,
        arc_deg: f32,
        stagger: f32,
        push: f32,
    },
    /// 눈발(0270 C3): 겨눈 쪽 `ahead` m 앞에 둘레 `radius` m의 눈발을 `secs`초 뿌린다 — `every`초마다 그 안의 적을 친다
    /// (잔불 흩기의 불티와 같은 꼴, 자리만 앞에).
    Drift {
        ahead: f32,
        radius: f32,
        secs: f32,
        every: f32,
    },
    /// 뿌리 덫(0270 C3): 겨눈 쪽 `ahead` m 앞에 씨를 심는다 — `secs`초 동안 둘레 `radius` m에 적이 들어서면 한 번 솟아 그 안의
    /// 적을 모두 찌른다(그리고 사라진다).
    Trap { ahead: f32, radius: f32, secs: f32 },
    /// 파문(0270 C3): 몸에서 둘레 `radius` m까지 `time`초에 번지는 물결 — 물결의 앞머리가 지나는 적을 한 번씩 친다.
    Wave { radius: f32, time: f32 },
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
    /// 제비걸음(0214): `range` m 안의 적에게 차례로 `hops`번 뛰어든다(`gap`초마다 — 그 앞 6할은 뛰는 걸음) — 닿은 자리 둘레
    /// `reach` m를 벤다. 가장 가까운 아직 안 벤 적부터, 적이 없으면 앞으로.
    Hop {
        range: f32,
        hops: u8,
        gap: f32,
        reach: f32,
    },
    /// 막아치기(0214): 앞 부채꼴(`range` m · `arc_deg`°)을 밀어친다 — 맞은 적은 `stagger`초 휘청, **공격을 준비하던(예고 중인) 적은
    /// 그 공격이 끊기고** 자세 피해 ×`mult`(보스는 끊기지 않고 자세 피해만). 패링이 아니다 — 창이 없다(0095 원칙 2).
    Bash {
        range: f32,
        arc_deg: f32,
        stagger: f32,
        mult: f32,
    },
    /// 바늘땀(0270 C3): 앞 좁은 부채꼴(`range` m · `arc_deg`°)을 `hits`번 찌른다(`gap`초 사이) — 찌를 때마다 새 타.
    Stab {
        range: f32,
        arc_deg: f32,
        hits: u8,
        gap: f32,
    },
    /// 종 치기(0270 C3): 앞 `range` m의 가장 가까운 적 하나를 치고 그 적에게 종을 건다 — `delay`초 뒤 그 적이 울려 둘레
    /// `radius` m를 친다(울림의 피해 = 표의 피해). 앞에 적이 없으면 허공에서 울린다.
    Bell { range: f32, delay: f32, radius: f32 },
    /// 재기(0274 — 한손검·흡수): 숨을 몰아쉬고 다시 선다 — 체력이 `heal` 돌아오고(받는 피해를 줄이지 않는다 — 되찾을 뿐, 0095
    /// 원칙 2), 둘레 `radius` m의 적을 피해 없이 `push` m 밀쳐 내며 흡수를 건다(그 적을 치면 더 되찾는다).
    Rally { heal: i32, radius: f32, push: f32 },
    /// 빠지며 베기(0274 ③ — 쌍단검): 시동 끝에 앞 부채꼴(`range` m · `arc_deg`°)을 한 번 베고, 그 기세로 `time`초 동안 `back` m 뒤로
    /// 물러난다(바라보는 쪽의 반대 — 막히면 거기서 멈춘다). 무적은 없다 — 거리를 벌릴 뿐(0095 원칙 2).
    Slip {
        range: f32,
        arc_deg: f32,
        back: f32,
        time: f32,
    },
    /// 뿌리 솟기(0214): 앞 `len` m를 `step` m 간격으로 뿌리가 `every`초마다 차례로 솟는다 — 솟은 자리 둘레 `radius` m의 적을 한 번씩
    /// 찌르고 `stagger`초 휘청(보스는 아니다).
    Erupt {
        len: f32,
        step: f32,
        every: f32,
        radius: f32,
        stagger: f32,
    },
}

impl Form {
    /// 쪽이 있는 모양인가(0217 — 터치에서 끌어 겨눈다): 몸 둘레(고리·버팀·되울림·불티)는 쪽이 없다.
    pub fn aims(&self) -> bool {
        !matches!(
            self,
            Form::Ring { .. }
                | Form::Brace { .. }
                | Form::Echo { .. }
                | Form::Field { .. }
                | Form::Wave { .. }
                | Form::Rally { .. }
        )
    }

    /// 한 대상을 겨누는 모양인가(0230 — 사장 "범위공격이면 바닥 표시가 맞고 단일타겟이라면 크로스헤어 방식"): 숄더뷰에서
    /// 크로스헤어로 나간다(던지기 = 곧게 나는 것 · 제비걸음 = 적에게 뛰어든다). 나머지 쪽이 있는 꼴(가로지르기의 길 · 뿌리 솟기의 줄 ·
    /// 부채꼴 셋)은 범위라 숄더뷰에서도 땅 위 표시로 겨눈다.
    pub fn sighted(&self) -> bool {
        matches!(
            self,
            Form::Throw { .. } | Form::Hop { .. } | Form::Bell { .. }
        )
    }
}

/// 뿌리 솟기의 뿌리 수(`len` ÷ `step`, 적어도 하나).
pub fn erupt_count(len: f32, step: f32) -> u8 {
    (len / step.max(0.1)).round().clamp(1.0, 24.0) as u8
}

/// 휘돌이의 전용 동작이 몸을 도는 쪽(+1 = 요가 느는 쪽 · −1 = 주는 쪽): 이펙트의 도는 초승달이 이 쪽으로 바퀴마다 한 바퀴 돈다.
/// 0193 스킨 몸의 원본(Meshy Skill_03)은 오른쪽으로 돈다 — `wanderer_skin.py` `SPIN`과 같아야 한다(시험).
pub const WHIRL_CLIP_TURN: f32 = -1.0;

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
    /// 제비걸음(0214): 적에게서 적으로 뛰어들며 벤다.
    Swallow,
    /// 막아치기(0214): 준비하는 공격을 끊는다.
    Bash,
    /// 뿌리 솟기(0214): 앞으로 차례로 솟는 뿌리.
    Roots,
    /// 바늘땀(0270 C3 — 결투·날카로움): 좁게 거듭 찌른다.
    Stitch,
    /// 방패 돌진(0270 C3 — 방패·흡수): 방패를 앞세워 내달려 밀어 낸다.
    Ram,
    /// 종 치기(0270 C3 — 방패·망치·울림): 한 적에게 종을 걸어 곧 울린다.
    Toll,
    /// 후려치기(0270 C3 — 망치·충격): 넓게 후려쳐 날려 보낸다.
    Swat,
    /// 파문(0270 C3 — 망치·울림): 둘레로 번지는 물결.
    Ripple,
    /// 눈발(0270 C3 — 지팡이·원거리·서리): 앞에 뿌리는 눈.
    Snow,
    /// 불길(0270 C3 — 지팡이·잔불): 앞으로 내뿜는 불.
    Flare,
    /// 뿌리 덫(0270 C3 — 활·뿌리): 심어 두는 씨.
    Snare,
    /// 심지 볼트(0272 — 쇠뇌·충격): 한 적에게 박아 곧 터뜨린다.
    Fuse,
    /// 화약 덫(0272 — 쇠뇌·충격): 묻어 두는 화약.
    Mine,
    /// 꿰뚫기(0272 2단계 — 창·관통): 한 줄을 꿰뚫으며 멀리 내달린다.
    Skewer,
    /// 장대 휩쓸기(0272 2단계 — 창·관통): 자루를 크게 한 바퀴 휘둘러 둘레의 적을 밀어낸다(에워싸였을 때의 답).
    Pole,
    /// 못 박기(0272 2단계 — 창·뿌리): 깊게 한 번 찔러 그 자리에 묶는다.
    Pin,
    /// 자루 치기(0274 — 한손검·충격): 칼자루로 짧게 쳐 준비하던 공격을 끊고 기절을 쌓는다.
    Pommel,
    /// 밀치기(0274 — 한손검·충격): 앞을 밀어 차 떼를 떼어 내고 자리를 만든다.
    Shove,
    /// 재기(0274 — 한손검·흡수): 숨을 몰아쉬고 다시 선다 — 체력이 조금 돈다.
    Rally,
    /// 빠지며 베기(0274 ③ — 쌍단검·관통): 앞을 베고 그 기세로 뒤로 물러난다(BG3 도적의 교활한 행동 — 치고 빠지기).
    Slip,
    /// 땅 가르기(0274 ④ — 대검·균열): 두 손으로 내리쳐 앞의 땅을 길게 가른다.
    Sunder,
    /// 불씨 휩쓸기(0274 ④ — 대검·잔불): 달군 날로 둘레를 크게 휩쓴다.
    Sear,
}

/// 표 순서(대기 칸 `Player::skill_cd`의 자리 — 뒤에만 붙인다).
pub const IDS: [Id; 30] = [
    Id::Whirl,
    Id::Cut,
    Id::Gather,
    Id::Quake,
    Id::Brace,
    Id::Echo,
    Id::Throw,
    Id::Embers,
    Id::Swallow,
    Id::Bash,
    Id::Roots,
    Id::Stitch,
    Id::Ram,
    Id::Toll,
    Id::Swat,
    Id::Ripple,
    Id::Snow,
    Id::Flare,
    Id::Snare,
    Id::Fuse,
    Id::Mine,
    Id::Skewer,
    Id::Pole,
    Id::Pin,
    Id::Pommel,
    Id::Shove,
    Id::Rally,
    Id::Slip,
    Id::Sunder,
    Id::Sear,
];
pub const N: usize = IDS.len();

/// 스킬 칸 둘의 처음 자리(0151 6번). 0166부터 이 둘도 값 없이 주지 않는다 — 기술 점수로 배운다. 새 판은 점수 없이
/// 선다(0167 — 첫 점수는 첫 레벨에서, 튜토리얼이 그 길을 알린다). 안 배운 스킬이 놓인 칸은 **빈 칸**이다(단 0 — 눌러도
/// 아무 일 없다, 그림은 빈 테).
pub const START: [Id; 2] = [Id::Whirl, Id::Cut];
/// 옛 기록의 몫(0166 → 0167): 처음의 둘을 값 없이 받았던 기록은 그 둘의 값을 점수로 지닌다(`Progress::gift` — 남은 점수가
/// 줄지 않게, 되돌리면 그 둘도 다른 데 쓸 수 있다). 새 판은 튜토리얼의 상 하나를 같은 칸에 받는다(`tutorial::REWARD_POINTS`,
/// 0169) — 이 수는 그 칸의 상한이기도 하다.
pub const GIFT_POINTS: u32 = 2;

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
    /// 쓸 수 있는 쥔 꼴(0223 V3 → 0225 → 0272 — 종류의 비트 `SWORD`·`HAMMER`·`STAFF`·`CROSSBOW`·`BOW`와 `SHIELD`의 합): 맞지 않는
    /// 것을 들면 그 칸은 빈 칸이다. 이 비트가 곧 그 스킬이 서는 판이다(`plates::Plate::seats`). 한손검만의 기술 = `SWORD`(방패 없이) ·
    /// 방패 판의 기술 = `SHIELD`.
    pub hands: Hands,
    /// 타입(0223 V2)과 한 타가 쌓는 게이지(보통 적의 1 가운데 몫 — 물리는 0). 단·고르기는 이 값을 바꾸지 않는다.
    /// 한 번 쓰면 보통 적은 걸리게(한 타짜리 = 1, 여러 타짜리는 나눠서) — 덜 쌓이는 적(엘리트·보스·버티는 종)은 두 번.
    pub kind: Kind,
    pub build: f32,
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
    /// 판정(움직임)이 이어지는 시간(초) — 되돌아 베기는 길을 두 번 간다.
    pub fn active(&self) -> f32 {
        match (self.skill.form, self.twist) {
            (Form::Lane { time, .. }, Some(Twist::Return)) => time * 2.0,
            // 돌아오는 걸음: 한 걸음 더(처음 자리로).
            (Form::Hop { gap, .. }, Some(Twist::HomeStep)) => self.skill.active() + gap,
            // 두 번 울림은 판정이 아니라 놓인 종이라 몸의 시간은 그대로다.
            _ => self.skill.active(),
        }
    }

    pub fn total(&self) -> f32 {
        self.skill.startup + self.active() + self.skill.recover
    }
}

impl Skill {
    /// 그 갈래의 무기로 쓸 수 있나(0223 V3).
    pub fn fits(&self, g: Grip) -> bool {
        self.hands & grip_bit(g) != 0
    }

    /// 이 스킬의 타 하나가 싣는 타입과 쌓는 양(0223 V2).
    pub fn dose(&self) -> Dose {
        (self.kind, self.build)
    }

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
            Form::Lane {
                dist,
                half,
                time,
                push,
            } => Form::Lane {
                dist: dist * size,
                half,
                time,
                push,
            },
            Form::Stab {
                range,
                arc_deg,
                hits,
                gap,
            } => Form::Stab {
                range: range * size,
                arc_deg,
                hits: hits
                    + match twist {
                        Some(Twist::Stitches { more }) => more,
                        _ => 0,
                    },
                gap,
            },
            Form::Bell {
                range,
                delay,
                radius,
            } => Form::Bell {
                range,
                delay,
                radius: radius
                    * size
                    * match twist {
                        Some(Twist::Loud { mult }) => mult,
                        _ => 1.0,
                    },
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
                push,
            } => Form::Cone {
                range,
                // 넓게 휘두르기(후려치기 ㄴ, 0270 C3).
                arc_deg: match twist {
                    Some(Twist::Wide { arc_deg }) => arc_deg,
                    _ => arc_deg,
                } * size,
                stagger,
                push,
            },
            Form::Wave { radius, time } => Form::Wave {
                radius: radius * size,
                time,
            },
            Form::Drift {
                ahead,
                radius,
                secs,
                every,
            } => Form::Drift {
                ahead,
                radius: radius
                    * size
                    * match twist {
                        Some(Twist::Big { mult }) => mult,
                        _ => 1.0,
                    },
                secs: secs
                    + match cap {
                        Some(Cap::Smolder { secs, .. }) => secs,
                        _ => 0.0,
                    },
                every,
            },
            Form::Trap {
                ahead,
                radius,
                secs,
            } => Form::Trap {
                ahead,
                radius: radius
                    * size
                    * match twist {
                        Some(Twist::Big { mult }) => mult,
                        _ => 1.0,
                    },
                secs,
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
            Form::Hop {
                range,
                hops,
                gap,
                reach,
            } => Form::Hop {
                range: range * size,
                hops,
                gap,
                reach,
            },
            Form::Bash {
                range,
                arc_deg,
                stagger,
                mult,
            } => Form::Bash {
                range,
                arc_deg: match twist {
                    Some(Twist::Wide { arc_deg }) => arc_deg,
                    _ => arc_deg,
                } * size,
                stagger,
                mult,
            },
            // 재기: 도는 체력이 피해의 배를 같이 받는다(10 → 12 → 13), 둘레는 4단의 치수.
            Form::Rally { heal, radius, push } => Form::Rally {
                heal: scaled(heal, dmg),
                radius: radius * size,
                push,
            },
            // 빠지며 베기: 4단은 베는 거리, 먼 걸음(ㄱ)은 물러나는 거리.
            Form::Slip {
                range,
                arc_deg,
                back,
                time,
            } => Form::Slip {
                range: range * size,
                arc_deg,
                back: match twist {
                    Some(Twist::Lunge { dist }) => dist,
                    _ => back,
                },
                time,
            },
            Form::Erupt {
                len,
                step,
                every,
                radius,
                stagger,
            } => Form::Erupt {
                len: len * size,
                step,
                every,
                radius,
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
            Form::Ring { hits, gap, .. } | Form::Stab { hits, gap, .. } => {
                gap * hits.saturating_sub(1) as f32
            }
            Form::Lane { time, .. } | Form::Wave { time, .. } | Form::Slip { time, .. } => time,
            Form::Brace { hold, .. } => hold,
            Form::Throw { hand, .. } => hand,
            Form::Hop { hops, gap, .. } => gap * hops as f32,
            Form::Erupt {
                len, step, every, ..
            } => every * erupt_count(len, step).saturating_sub(1) as f32,
            Form::Pull { .. }
            | Form::Cone { .. }
            | Form::Echo { .. }
            | Form::Field { .. }
            | Form::Bash { .. }
            | Form::Bell { .. }
            | Form::Drift { .. }
            | Form::Trap { .. }
            | Form::Rally { .. } => 0.0,
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
        // 0274 ④: 대검도 휘돈다(광전사의 휩쓸기 — 대검 판의 가운데 열).
        hands: ONE | GREATSWORD,
        kind: Kind::Physical,
        build: 0.0,
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
            push: 0.0,
        },
        anim: "attack3",
        clip: Clip {
            name: "sk_cut",
            seg: [0.12, 0.20, 0.25],
        },
        row: 0,
        // 0274 ④: 대검도 떼를 뚫고 지나간다(대검 판의 균열 계열 첫 줄).
        hands: DAGGER | GREATSWORD,
        kind: Kind::Rift,
        build: 1.0,
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
        hands: ANY,
        kind: Kind::Frost,
        build: 0.6,
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
            push: 0.0,
        },
        anim: "hammer_slam",
        clip: Clip {
            name: "sk_quake",
            seg: [0.30, 0.0, 0.40],
        },
        row: 1,
        hands: HAMMER,
        kind: Kind::Shock,
        build: 1.0,
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
        hands: MELEE & !DAGGER,
        kind: Kind::Drain,
        build: 1.0,
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
        // 0274 ④: 한손검(전사)도 — 방패 없이 받아치면 둘레가 울린다(한손검 판 가운데 둘째 줄이 비어 있었다).
        hands: SHIELD | SWORD | STAFF | RANGED,
        kind: Kind::Knell,
        build: 1.0,
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
        hands: DAGGER | HAMMER,
        kind: Kind::Rift,
        build: 0.5,
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
        hands: STAFF | CROSSBOW,
        kind: Kind::Ember,
        build: 0.5,
        branch: Branch::Footing,
        picks: [Twist::Follow, Twist::Kindle { embers: 1 }],
        cap: Cap::Smolder {
            secs: 1.0,
            poise: 1.0,
        },
    },
    // 0214 넷째 줄: 갈래마다 새 스킬 하나.
    Skill {
        id: Id::Swallow,
        cooldown: 9.0,
        startup: 0.12,
        recover: 0.25,
        damage: HP_UNIT,
        poise: 1.5,
        form: Form::Hop {
            range: 6.0,
            hops: 3,
            gap: 0.15,
            reach: 1.6,
        },
        anim: "attack3",
        clip: Clip {
            name: "sk_swallow",
            seg: [0.12, 0.45, 0.25],
        },
        row: 3,
        hands: DAGGER,
        kind: Kind::Sharp,
        build: 0.4,
        branch: Branch::Beat,
        picks: [Twist::Focus, Twist::HomeStep],
        cap: Cap::Finale {
            reach: 2.6,
            poise: 2.0,
        },
    },
    Skill {
        id: Id::Bash,
        cooldown: 9.0,
        startup: 0.15,
        recover: 0.35,
        damage: HP_UNIT,
        poise: 2.0,
        form: Form::Bash {
            range: 2.8,
            arc_deg: 100.0,
            stagger: 0.5,
            mult: 3.0,
        },
        anim: "parry_push",
        clip: Clip {
            name: "sk_bash",
            seg: [0.15, 0.0, 0.35],
        },
        row: 3,
        hands: SHIELD,
        kind: Kind::Knell,
        build: 1.0,
        branch: Branch::Riposte,
        picks: [
            Twist::Wide { arc_deg: 180.0 },
            Twist::Expose {
                secs: 2.0,
                mult: 1.3,
            },
        ],
        cap: Cap::Break,
    },
    Skill {
        id: Id::Roots,
        cooldown: 10.0,
        startup: 0.25,
        recover: 0.35,
        damage: HP_UNIT,
        poise: 1.5,
        form: Form::Erupt {
            len: 7.0,
            step: 1.0,
            every: 0.05,
            radius: 1.0,
            stagger: 0.5,
        },
        anim: "hammer_slam",
        clip: Clip {
            name: "sk_roots",
            seg: [0.25, 0.30, 0.35],
        },
        row: 3,
        hands: STAFF | BOW | SPEAR,
        kind: Kind::Root,
        build: 1.0,
        branch: Branch::Footing,
        picks: [Twist::Circle, Twist::Bind { secs: 1.2 }],
        cap: Cap::Spire {
            radius: 2.0,
            damage: 2 * HP_UNIT,
        },
    },
    // 0270 C3: 무기 판의 빈 자리를 채우는 스킬.
    Skill {
        id: Id::Stitch,
        cooldown: 6.0,
        startup: 0.10,
        recover: 0.25,
        damage: HP_UNIT,
        poise: 0.6,
        form: Form::Stab {
            range: 2.8,
            arc_deg: 50.0,
            hits: 3,
            gap: 0.10,
        },
        anim: "attack3",
        clip: Clip {
            name: "sk_stitch",
            seg: [0.10, 0.20, 0.25],
        },
        row: 0,
        hands: DAGGER | SPEAR,
        kind: Kind::Sharp,
        build: 0.45,
        branch: Branch::Beat,
        picks: [Twist::Lunge { dist: 2.5 }, Twist::Stitches { more: 2 }],
        cap: Cap::Pin { stagger: 0.5 },
    },
    Skill {
        id: Id::Ram,
        cooldown: 9.0,
        startup: 0.15,
        recover: 0.30,
        damage: HP_UNIT,
        poise: 2.5,
        form: Form::Lane {
            dist: 4.5,
            half: 1.1,
            time: 0.25,
            push: 2.0,
        },
        anim: "parry_push",
        clip: Clip {
            name: "sk_ram",
            seg: [0.15, 0.25, 0.30],
        },
        row: 1,
        hands: SHIELD,
        kind: Kind::Drain,
        build: 1.0,
        branch: Branch::Riposte,
        picks: [
            Twist::Slam {
                damage: 2 * HP_UNIT,
            },
            Twist::Ghost,
        ],
        cap: Cap::Stun { stagger: 0.8 },
    },
    Skill {
        id: Id::Toll,
        cooldown: 8.0,
        startup: 0.20,
        recover: 0.30,
        damage: HP_UNIT,
        poise: 2.0,
        form: Form::Bell {
            range: 2.8,
            delay: 0.8,
            radius: 3.0,
        },
        anim: "staff4",
        clip: Clip {
            name: "sk_toll",
            seg: [0.20, 0.0, 0.30],
        },
        row: 0,
        hands: SHIELD | HAMMER,
        kind: Kind::Knell,
        build: 0.5,
        branch: Branch::Riposte,
        picks: [Twist::Twice, Twist::Loud { mult: 1.5 }],
        cap: Cap::Toll { damage: HP_UNIT },
    },
    Skill {
        id: Id::Swat,
        cooldown: 7.0,
        startup: 0.22,
        recover: 0.35,
        damage: HP_UNIT,
        poise: 3.0,
        form: Form::Cone {
            range: 3.2,
            arc_deg: 150.0,
            stagger: 0.3,
            push: 2.5,
        },
        anim: "hammer1",
        clip: Clip {
            name: "sk_swat",
            seg: [0.22, 0.0, 0.35],
        },
        row: 0,
        hands: HAMMER,
        kind: Kind::Shock,
        build: 1.0,
        branch: Branch::Beat,
        picks: [
            Twist::Slam {
                damage: 2 * HP_UNIT,
            },
            Twist::Wide { arc_deg: 220.0 },
        ],
        cap: Cap::Shatter { mult: 1.5 },
    },
    Skill {
        id: Id::Ripple,
        cooldown: 10.0,
        startup: 0.25,
        recover: 0.35,
        damage: 2 * HP_UNIT,
        poise: 2.0,
        form: Form::Wave {
            radius: 6.5,
            time: 0.30,
        },
        anim: "hammer_slam",
        clip: Clip {
            name: "sk_ripple",
            seg: [0.25, 0.30, 0.35],
        },
        row: 1,
        hands: HAMMER,
        kind: Kind::Knell,
        build: 1.0,
        branch: Branch::Riposte,
        picks: [
            Twist::Shove { push: 2.5 },
            Twist::Crack {
                secs: 2.0,
                slow: 0.5,
            },
        ],
        cap: Cap::Lift { stagger: 0.5 },
    },
    Skill {
        id: Id::Snow,
        cooldown: 10.0,
        startup: 0.25,
        recover: 0.30,
        damage: HP_UNIT,
        poise: 0.0,
        form: Form::Drift {
            ahead: 5.0,
            radius: 3.0,
            secs: 3.0,
            every: 1.0,
        },
        anim: "cast",
        clip: Clip {
            name: "sk_snow",
            seg: [0.25, 0.0, 0.30],
        },
        row: 1,
        hands: STAFF | BOW,
        kind: Kind::Frost,
        build: 0.4,
        branch: Branch::Footing,
        picks: [Twist::Flow { speed: 1.5 }, Twist::Big { mult: 1.4 }],
        cap: Cap::Smolder {
            secs: 1.0,
            poise: 1.0,
        },
    },
    Skill {
        id: Id::Flare,
        cooldown: 7.0,
        startup: 0.20,
        recover: 0.30,
        damage: HP_UNIT,
        poise: 1.0,
        form: Form::Cone {
            range: 4.2,
            arc_deg: 70.0,
            stagger: 0.0,
            push: 0.5,
        },
        anim: "cast",
        clip: Clip {
            name: "sk_flare",
            seg: [0.20, 0.0, 0.30],
        },
        row: 0,
        hands: STAFF | CROSSBOW,
        kind: Kind::Ember,
        build: 1.0,
        branch: Branch::Footing,
        picks: [Twist::Wide { arc_deg: 120.0 }, Twist::Scorch { secs: 2.0 }],
        cap: Cap::Frail {
            secs: 2.0,
            mult: 1.2,
        },
    },
    Skill {
        id: Id::Snare,
        cooldown: 8.0,
        startup: 0.18,
        recover: 0.25,
        damage: HP_UNIT,
        poise: 1.5,
        form: Form::Trap {
            ahead: 4.0,
            radius: 2.0,
            secs: 8.0,
        },
        anim: "throw",
        clip: Clip {
            name: "sk_snare",
            seg: [0.18, 0.0, 0.25],
        },
        row: 0,
        hands: BOW,
        kind: Kind::Root,
        build: 1.0,
        branch: Branch::Footing,
        picks: [Twist::Twin, Twist::Big { mult: 1.5 }],
        cap: Cap::Spire {
            radius: 2.5,
            damage: 2 * HP_UNIT,
        },
    },
    // 심지 볼트(0272 — 쇠뇌 판의 충격 갈래, 첫 줄): 종 치기의 꼴(`Form::Bell`)을 멀리서 — 볼트를 박고 곧 터뜨린다.
    Skill {
        id: Id::Fuse,
        cooldown: 8.0,
        startup: 0.20,
        recover: 0.30,
        damage: HP_UNIT,
        poise: 2.0,
        form: Form::Bell {
            range: 9.0,
            delay: 0.8,
            radius: 2.5,
        },
        anim: "xbow_shoot",
        clip: Clip {
            name: "sk_fuse",
            seg: [0.20, 0.0, 0.30],
        },
        row: 0,
        hands: CROSSBOW,
        kind: Kind::Shock,
        build: 0.5,
        branch: Branch::Footing,
        picks: [Twist::Twice, Twist::Loud { mult: 1.5 }],
        cap: Cap::Toll { damage: HP_UNIT },
    },
    // 화약 덫(0272 — 쇠뇌 판의 충격 갈래, 둘째 줄): 뿌리 덫의 꼴(`Form::Trap`)에 화약 — 밟으면 터진다.
    Skill {
        id: Id::Mine,
        cooldown: 10.0,
        startup: 0.18,
        recover: 0.25,
        damage: 2 * HP_UNIT,
        poise: 3.0,
        form: Form::Trap {
            ahead: 4.0,
            radius: 2.2,
            secs: 8.0,
        },
        anim: "throw",
        clip: Clip {
            name: "sk_mine",
            seg: [0.18, 0.0, 0.25],
        },
        row: 1,
        hands: CROSSBOW,
        kind: Kind::Shock,
        build: 1.0,
        branch: Branch::Footing,
        picks: [Twist::Twin, Twist::Big { mult: 1.5 }],
        cap: Cap::Spire {
            radius: 3.0,
            damage: 2 * HP_UNIT,
        },
    },
    // 0272 2단계 창 판: 가운데(관통) 둘과 뿌리 갈래 하나 — 있는 모양을 다시 쓴다(가로지르기의 길 · 휘돌이의 고리 · 바늘땀의 찌름).
    // 꿰뚫기: 좁은 길(반 폭 0.7 m — 가로지르기 1.2)로 더 멀리(7 m — 6) 내달리며 꿰뚫는다. 물리, 적의 몸을 뚫고 지나간다.
    Skill {
        id: Id::Skewer,
        cooldown: 8.0,
        startup: 0.14,
        recover: 0.30,
        damage: 2 * HP_UNIT,
        poise: 2.5,
        form: Form::Lane {
            dist: 7.0,
            half: 0.7,
            time: 0.22,
            push: 0.0,
        },
        anim: "spear3",
        clip: Clip {
            name: "sk_skewer",
            seg: [0.14, 0.22, 0.30],
        },
        row: 0,
        hands: SPEAR,
        kind: Kind::Physical,
        build: 0.0,
        branch: Branch::Beat,
        picks: [Twist::Return, Twist::Ghost],
        cap: Cap::Refund { part: 0.5 },
    },
    // 장대 휩쓸기: 자루를 한 바퀴 크게(3.6 m — 휘돌이 3.0) 휘둘러 둘레의 적을 멀리(3 m) 밀어낸다. 둘레에 붙은 떼에 약한 창의 답.
    Skill {
        id: Id::Pole,
        cooldown: 10.0,
        startup: 0.20,
        recover: 0.35,
        damage: 2 * HP_UNIT,
        poise: 3.0,
        form: Form::Ring {
            radius: 3.6,
            hits: 1,
            gap: 0.2,
            push: 3.0,
        },
        anim: "staff4",
        clip: Clip {
            name: "sk_pole",
            seg: [0.20, 0.0, 0.35],
        },
        row: 1,
        hands: SPEAR,
        kind: Kind::Physical,
        build: 0.0,
        branch: Branch::Footing,
        picks: [Twist::Walk { speed: 0.5 }, Twist::Spins { more: 1 }],
        cap: Cap::Lift { stagger: 0.6 },
    },
    // 못 박기: 좁고 깊게(3.6 m · 24°) 한 번 찔러 뿌리를 가득(1.0 — 뿌리에 무른 적은 한 번에, 잔재처럼 덜 쌓이는 적은 두 번째 찌름 ㄴ에) — 묶이면 걷지 못한다.
    Skill {
        id: Id::Pin,
        cooldown: 8.0,
        startup: 0.16,
        recover: 0.30,
        damage: 2 * HP_UNIT,
        poise: 2.0,
        form: Form::Stab {
            range: 3.6,
            arc_deg: 24.0,
            hits: 1,
            gap: 0.10,
        },
        anim: "spear3",
        clip: Clip {
            name: "sk_pin",
            seg: [0.16, 0.0, 0.30],
        },
        row: 0,
        hands: SPEAR,
        kind: Kind::Root,
        build: 1.0,
        branch: Branch::Riposte,
        picks: [Twist::Lunge { dist: 2.0 }, Twist::Stitches { more: 1 }],
        cap: Cap::Pin { stagger: 0.6 },
    },
    // ---- 0274: 한손검 = 전투 기술(충격 · 흡수) ----
    // 자루 치기: 칼자루로 짧고 좁게(2.2 m · 70°) 친다 — 막아치기의 꼴(`Form::Bash`): 준비하던 잡몹의 공격을 끊고 충격(기절)을
    // 한 번에 쌓는다. 보스는 끊기지 않고 자세만 더 깎인다(보스 규칙 그대로).
    Skill {
        id: Id::Pommel,
        cooldown: 7.0,
        startup: 0.12,
        recover: 0.28,
        damage: HP_UNIT,
        poise: 2.5,
        form: Form::Bash {
            range: 2.2,
            arc_deg: 70.0,
            stagger: 0.6,
            mult: 2.0,
        },
        anim: "attack3",
        clip: Clip {
            name: "sk_pommel",
            seg: [0.12, 0.0, 0.28],
        },
        row: 0,
        hands: SWORD,
        kind: Kind::Shock,
        build: 1.0,
        branch: Branch::Riposte,
        picks: [
            Twist::Wide { arc_deg: 120.0 },
            Twist::Expose {
                secs: 2.0,
                mult: 1.2,
            },
        ],
        cap: Cap::Break,
    },
    // 밀치기: 앞을 밀어 찬다(2.4 m · 140°) — 피해는 작고 멀리(3.5 m) 밀어내 자리를 만든다, 충격을 반쯤 쌓는다.
    Skill {
        id: Id::Shove,
        cooldown: 8.0,
        startup: 0.14,
        recover: 0.30,
        damage: HP_UNIT,
        poise: 1.5,
        form: Form::Cone {
            range: 2.4,
            arc_deg: 140.0,
            stagger: 0.5,
            push: 3.5,
        },
        anim: "fist_heavy",
        clip: Clip {
            name: "sk_shove",
            seg: [0.14, 0.0, 0.30],
        },
        row: 1,
        hands: SWORD,
        kind: Kind::Shock,
        build: 0.5,
        branch: Branch::Footing,
        picks: [
            Twist::Slam {
                damage: 2 * HP_UNIT,
            },
            Twist::Wide { arc_deg: 220.0 },
        ],
        cap: Cap::Frail {
            secs: 2.0,
            mult: 1.2,
        },
    },
    // 재기: 숨을 몰아쉬고 다시 선다 — 체력 10(한 눈금 — 등잔 한 모금의 반), 쿨타임은 가장 긴 14초. 곁(2.4 m)의 적은 피해 없이
    // 밀쳐 내며 흡수를 건다. 받는 피해는 줄이지 않는다(되찾을 뿐 — 0095 원칙 2). 값은 보스 모의로(0274 "② 만든 것").
    Skill {
        id: Id::Rally,
        cooldown: 14.0,
        startup: 0.20,
        recover: 0.35,
        damage: 0,
        poise: 0.0,
        form: Form::Rally {
            heal: HP_UNIT,
            radius: 2.4,
            push: 1.5,
        },
        anim: "parry",
        clip: Clip {
            name: "sk_rally",
            seg: [0.20, 0.0, 0.35],
        },
        row: 1,
        hands: SWORD,
        kind: Kind::Drain,
        build: 1.0,
        branch: Branch::Riposte,
        picks: [
            Twist::Breath { stamina: 20.0 },
            Twist::Low {
                below: 0.35,
                mult: 2.0,
            },
        ],
        // 맺음은 체력을 더 돌리지 않는다(모의: 5레벨에 체력 + 10이면 종지기를 이기는 몫이 70 → 97% — 등잔을 대신했다) — 밀쳐 낸 적이
        // 잠깐 휘청인다(보스는 아니다, 버팀의 맺음과 같은 꼴).
        cap: Cap::Stun { stagger: 0.8 },
    },
    // 빠지며 베기(0274 ③ — 쌍단검 · 관통, BG3 도적의 교활한 행동): 앞(2.4 m · 90°)을 한 번 베고 그 기세로 0.18초에 3 m 뒤로 물러난다 —
    // 떼에 붙었다 빠지는 손. 무적은 없다(거리만). 먼 걸음(ㄱ) = 4.5 m · 벌어진 틈(ㄴ) = 베인 적이 2초 피해 +25%(보스는 아니다) ·
    // 최종 효과 = 이 스킬로 쓰러뜨리면 쿨타임의 반을 돌려받는다.
    Skill {
        id: Id::Slip,
        cooldown: 7.0,
        startup: 0.10,
        recover: 0.20,
        damage: 2 * HP_UNIT,
        poise: 1.0,
        form: Form::Slip {
            range: 2.4,
            arc_deg: 90.0,
            back: 3.0,
            time: 0.18,
        },
        anim: "dagger3",
        clip: Clip {
            name: "sk_slip",
            seg: [0.10, 0.18, 0.20],
        },
        row: 1,
        hands: DAGGER,
        kind: Kind::Physical,
        build: 0.0,
        branch: Branch::Beat,
        picks: [
            Twist::Lunge { dist: 4.5 },
            Twist::Expose {
                secs: 2.0,
                mult: 1.25,
            },
        ],
        cap: Cap::Refund { part: 0.5 },
    },
    // ---- 0274 ④: 대검 = 광전사(균열 · 잔불) ----
    // 땅 가르기: 두 손으로 내리쳐 앞의 땅을 길게(6 m · 좁은 36°) 가른다 — 줄 선 떼를 한 번에, 균열을 한 번에 쌓는다. 넓게 가르기(ㄱ) ·
    // 갈라진 땅(ㄴ — 땅울림의 ㄴ과 같은 꼴: 금 간 자리에 선 적이 느려진다) · 최종 효과 = 갈린 적이 2초 동안 피해를 더 받는다.
    Skill {
        id: Id::Sunder,
        cooldown: 9.0,
        startup: 0.30,
        recover: 0.40,
        damage: 2 * HP_UNIT,
        poise: 3.5,
        form: Form::Cone {
            range: 6.0,
            arc_deg: 36.0,
            stagger: 0.5,
            push: 0.0,
        },
        anim: "gs5",
        clip: Clip {
            name: "sk_sunder",
            seg: [0.30, 0.0, 0.40],
        },
        row: 1,
        hands: GREATSWORD,
        kind: Kind::Rift,
        build: 1.0,
        branch: Branch::Footing,
        picks: [
            Twist::Wide { arc_deg: 70.0 },
            Twist::Crack {
                secs: 2.5,
                slow: 0.5,
            },
        ],
        cap: Cap::Frail {
            secs: 2.0,
            mult: 1.2,
        },
    },
    // 불씨 휩쓸기: 달군 날로 둘레를 크게(3 m · 220°) 휩쓴다 — 맞은 적에 잔불을 한 번에 쌓고 조금 밀어낸다. 다 도는 휩쓸기(ㄱ — 300°) ·
    // 타는 땅(ㄴ — 불길의 ㄴ과 같은 꼴: 휩쓴 자리에 불티가 남는다) · 최종 효과 = 자세가 깨진 적에게 피해 ×1.5.
    Skill {
        id: Id::Sear,
        cooldown: 8.0,
        startup: 0.18,
        recover: 0.35,
        damage: 2 * HP_UNIT,
        poise: 2.0,
        form: Form::Cone {
            range: 3.0,
            arc_deg: 220.0,
            stagger: 0.3,
            push: 1.2,
        },
        anim: "gs4",
        clip: Clip {
            name: "sk_sear",
            seg: [0.18, 0.0, 0.35],
        },
        row: 0,
        hands: GREATSWORD,
        kind: Kind::Ember,
        build: 1.0,
        branch: Branch::Beat,
        picks: [Twist::Wide { arc_deg: 300.0 }, Twist::Scorch { secs: 3.0 }],
        cap: Cap::Shatter { mult: 1.5 },
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
            Id::Swallow => "swallow",
            Id::Bash => "bash",
            Id::Roots => "roots",
            Id::Stitch => "stitch",
            Id::Ram => "ram",
            Id::Toll => "toll",
            Id::Swat => "swat",
            Id::Ripple => "ripple",
            Id::Snow => "snow",
            Id::Flare => "flare",
            Id::Snare => "snare",
            Id::Fuse => "fuse",
            Id::Mine => "mine",
            Id::Skewer => "skewer",
            Id::Pole => "pole",
            Id::Pin => "pin",
            Id::Pommel => "pommel",
            Id::Shove => "shove",
            Id::Rally => "rally",
            Id::Slip => "slip",
            Id::Sunder => "sunder",
            Id::Sear => "sear",
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
}

/// 큰 기술의 표 순서(판의 단 `Board::ult`의 자리 — 뒤에만 붙인다).
pub const ULTS: [Ult; 9] = [
    Ult::Upend,
    Ult::Flurry,
    Ult::Line,
    Ult::Bell,
    Ult::Blizzard,
    Ult::Rain,
    Ult::Gale,
    Ult::Ambush,
    Ult::Tempest,
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
        }
    }

    pub fn by_key(k: &str) -> Option<Ult> {
        ULTS.into_iter().find(|u| u.key() == k)
    }

    /// 판의 갈래(0161 "갈래와 줄"): 몰아치기 = 흐름, 한 줄기 = 맞받기, 쟁반 뒤집기 = 자리.
    pub fn branch(self) -> Branch {
        match self {
            Ult::Upend | Ult::Blizzard | Ult::Rain => Branch::Footing,
            Ult::Flurry | Ult::Ambush | Ult::Tempest => Branch::Beat,
            Ult::Line | Ult::Bell | Ult::Gale => Branch::Riposte,
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

/// 시험: 앞의 몇만 적은 스킬의 단(나머지 0) — 스킬이 늘어도 시험의 표가 그대로 읽히게.
#[cfg(test)]
pub fn ranks<const K: usize>(head: [u8; K]) -> [u8; N] {
    let mut out = [0; N];
    out[..K].copy_from_slice(&head);
    out
}

/// 시험: 앞의 몇만 적은 큰 기술의 단(나머지 0).
#[cfg(test)]
pub fn ult_ranks<const K: usize>(head: [u8; K]) -> [u8; NU] {
    let mut out = [0; NU];
    out[..K].copy_from_slice(&head);
    out
}

/// 시험: 앞의 몇만 적은 고르기(나머지 ㄱ).
#[cfg(test)]
pub fn picks<const K: usize>(head: [Pick; K]) -> [Pick; N] {
    let mut out = [Pick::A; N];
    out[..K].copy_from_slice(&head);
    out
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
/// 처음부터 가진 점수는 없다(0167) — 옛 기록의 몫 `GIFT_POINTS`는 기록이 따로 더한다(`Progress::skill_points`).
pub fn points(level: u32, elites: u32, cores: u32, shards: u32) -> u32 {
    level + elites + cores + shards
}

/// 줄이 열리는 때(쓴 점수 합): 첫 줄 · 둘째 줄 · 큰 기술 줄(0158 라) · 넷째 줄(새 스킬·패시브) · 맺음 줄(0214 — Diablo IV의
/// 묶음처럼 깊은 줄일수록 더 써야 열린다. 있는 줄의 열림은 그대로 — 이미 산 칸이 잠기지 않게).
pub const ROW_NEEDS: [u32; 6] = [0, 4, 8, 14, 22, 0];
/// 맺음의 줄(`ROW_NEEDS`의 자리).
pub const KEY_ROW: usize = 4;
/// 결의 줄(0223 11절 — 판의 맨 아래 띠): 쓴 점수로 열리지 않는다(`ROW_NEEDS` 0) — 그 타입의 스킬을 배웠나만 본다
/// (`Board::reachable`). 가로 자리는 판의 폭을 넷으로 나눈 가운데들.
pub const VEIN_ROW: usize = 5;
pub const LANE_VEIN: [f32; 8] = [17.0, 51.0, 84.0, 118.0, 152.0, 186.0, 219.0, 253.0];
/// 맺음을 켜는 데 그 갈래에 써 둔 점수(0215 A — 갈래의 스킬 단 + 큰 기술 단 + 패시브 칸, 다리 칸은 어느 갈래에도 세지 않는다).
pub const KEY_BRANCH: u32 = 10;
/// 판의 가로 자리(0215 B — 화면의 칸 자리 `app::skills::spots`도 이것을 쓴다): 갈래 폭 안에서 칸 둘이면 `LANE_2`, 하나면
/// `LANE_1`, 큰 기술 줄은 `LANE_ULT`(큰 기술 · 패시브). 다리 칸은 두 갈래의 경계(`LANE_W`의 배수).
pub const LANE_W: f32 = 90.0;
pub const LANE_2: [f32; 2] = [23.0, 67.0];
pub const LANE_1: f32 = 45.0;
pub const LANE_ULT: [f32; 2] = [30.0, 66.0];
/// 이어지는 가로 거리: 갈래 안의 왼쪽(23)·가운데(45)·오른쪽(67) 가운데 왼쪽↔오른쪽(44)만 안 이어진다.
pub const LANE_DX: f32 = 30.0;
/// 되돌리기 값(잔불): `RESPEC_BASE + RESPEC_PER × 쓴 점수`.
pub const RESPEC_BASE: u32 = 30;
pub const RESPEC_PER: u32 = 5;

/// 판의 칸: 스킬 하나 · 큰 기술 하나 · 패시브 칸 하나(0161) · 맺음 하나(0214 — 셋 가운데 하나만 켠다).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Entry {
    Skill(Id),
    Ult(Ult),
    Cell(Cell),
    Key(Key),
    /// 다리 칸(0215 C — 두 갈래의 경계, 시너지).
    Link(Link),
    /// 결의 칸(0223 11절 — 피해 타입마다 하나).
    Vein(Vein),
}

/// 판의 칸 수: 스킬 + 큰 기술 + 패시브(갈래마다 `DEPTH`) + 맺음 셋 + 다리 넷.
pub const NE: usize = N + NU + 3 * DEPTH as usize + KEYS.len() + LINKS.len() + VEINS.len();

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
    // 0270 C3 — 무기 판의 첫 줄(옛 판의 자리는 갈래의 첫 줄 끝)
    Entry::Skill(Id::Stitch),
    Entry::Skill(Id::Toll),
    Entry::Skill(Id::Swat),
    Entry::Skill(Id::Flare),
    Entry::Skill(Id::Snare),
    // 0272 — 쇠뇌 판의 충격 갈래 첫 줄
    Entry::Skill(Id::Fuse),
    // 0272 2단계 — 창 판의 첫 줄(가운데 · 뿌리 갈래)
    Entry::Skill(Id::Skewer),
    Entry::Skill(Id::Pin),
    // 0274 — 한손검 판의 충격 갈래 첫 줄
    Entry::Skill(Id::Pommel),
    // 0274 ④ — 대검 판의 잔불 갈래 첫 줄
    Entry::Skill(Id::Sear),
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
    Entry::Link(Link::RiposteBeat),
    Entry::Link(Link::ParryStep),
    // 0270 C3 — 무기 판의 둘째 줄
    Entry::Skill(Id::Ram),
    Entry::Skill(Id::Ripple),
    Entry::Skill(Id::Snow),
    // 0272 — 쇠뇌 판의 충격 갈래 둘째 줄
    Entry::Skill(Id::Mine),
    // 0272 2단계 — 창 판의 둘째 줄(가운데)
    Entry::Skill(Id::Pole),
    // 0274 — 한손검 판의 둘째 줄(충격 · 흡수)
    Entry::Skill(Id::Shove),
    Entry::Skill(Id::Rally),
    // 0274 ③ — 쌍단검 판의 둘째 줄(가운데)
    Entry::Skill(Id::Slip),
    // 0274 ④ — 대검 판의 둘째 줄(균열)
    Entry::Skill(Id::Sunder),
    // 3 — 쓴 점수 8
    Entry::Ult(Ult::Flurry),
    Entry::Cell(Cell::FinalMeasure),
    Entry::Ult(Ult::Line),
    Entry::Cell(Cell::OneWhoStands),
    Entry::Ult(Ult::Upend),
    Entry::Cell(Cell::PathOpeningStep),
    // 0270 C3 — 무기 판마다 큰 기술 하나: 방패 판·지팡이 판
    Entry::Ult(Ult::Bell),
    Entry::Ult(Ult::Blizzard),
    // 0272 — 활 판의 큰 기술
    Entry::Ult(Ult::Rain),
    // 0272 2단계 — 창 판의 큰 기술
    Entry::Ult(Ult::Gale),
    // 0274 ③ — 쌍단검 판의 큰 기술
    Entry::Ult(Ult::Ambush),
    // 0274 ④ — 대검 판의 큰 기술
    Entry::Ult(Ult::Tempest),
    // 4 — 쓴 점수 14(0214)
    Entry::Skill(Id::Swallow),
    Entry::Cell(Cell::HeldBreath),
    Entry::Cell(Cell::GatheringBeat),
    Entry::Skill(Id::Bash),
    Entry::Cell(Cell::SecondWind),
    Entry::Cell(Cell::Mend),
    Entry::Skill(Id::Roots),
    Entry::Cell(Cell::Planted),
    Entry::Cell(Cell::Snare),
    Entry::Link(Link::BreakingBeat),
    Entry::Link(Link::RootedRiposte),
    // 5 — 쓴 점수 22: 맺음(하나만)
    Entry::Key(Key::Measure),
    Entry::Key(Key::Edge),
    Entry::Key(Key::Rooted),
    // 결(0223 11절) — 그 타입의 스킬을 배우면
    Entry::Vein(Vein::Ember),
    Entry::Vein(Vein::Root),
    Entry::Vein(Vein::Knell),
    Entry::Vein(Vein::Rift),
    // 0270 C3: 새 타입 넷의 결
    Entry::Vein(Vein::Shock),
    Entry::Vein(Vein::Sharp),
    Entry::Vein(Vein::Frost),
    Entry::Vein(Vein::Drain),
];

impl Entry {
    /// 끝 단(패시브 칸은 열림 = 1).
    pub fn max(self) -> u8 {
        match self {
            Entry::Skill(_) => MAX_RANK,
            Entry::Ult(_) => ULT_MAX_RANK,
            Entry::Cell(_) | Entry::Key(_) | Entry::Link(_) | Entry::Vein(_) => 1,
        }
    }

    pub fn row(self) -> usize {
        match self {
            Entry::Skill(id) => id.def().row as usize,
            Entry::Ult(_) => ULT_ROW as usize,
            Entry::Cell(c) => c.row(),
            Entry::Key(_) => KEY_ROW,
            Entry::Link(l) => l.row(),
            Entry::Vein(_) => VEIN_ROW,
        }
    }

    /// 갈래 — 다리 칸은 잇는 둘 가운데 왼쪽(화면의 빛깔만 — 갈래의 점수 `Board::branch_spent`에는 세지 않는다).
    pub fn branch(self) -> Branch {
        match self {
            Entry::Skill(id) => id.def().branch,
            Entry::Ult(u) => u.branch(),
            Entry::Cell(c) => c.branch(),
            Entry::Key(k) => k.branch(),
            Entry::Link(l) => crate::progression::BRANCHES[l.left()],
            Entry::Vein(v) => v.branch(),
        }
    }

    /// 작은 칸(패시브·맺음·다리)인가 — 줄 안에서 스킬·큰 기술 아래에 선다.
    pub fn small(self) -> bool {
        matches!(
            self,
            Entry::Cell(_) | Entry::Key(_) | Entry::Link(_) | Entry::Vein(_)
        )
    }

    /// 판의 길의 줄(0215 B, 0~7 — 첫 줄 스킬 · 첫 줄 패시브 · 둘째 줄 스킬 · 둘째 줄 패시브(+ 다리) · 큰 기술 줄 · 넷째 줄 스킬 ·
    /// 넷째 줄 패시브(+ 다리) · 맺음): 줄(`row`)마다 스킬 줄과 패시브 줄 — 큰 기술 줄과 맺음 줄은 하나.
    pub fn line(self) -> usize {
        let row = self.row();
        let base = [0, 2, 4, 5, 7, 8][row];
        let one_line = row == ULT_ROW as usize || row == KEY_ROW || row == VEIN_ROW;
        base + usize::from(self.small() && !one_line)
    }

    /// 판의 가로 자리(0215 B — 셋째 갈래의 오른쪽 끝이 `3 × LANE_W`). 같은 줄·갈래의 같은 크기 칸 가운데 몇째인가로 정한다
    /// (`ENTRIES`의 차례), 큰 기술 줄은 큰 기술 먼저.
    pub fn lane(self) -> f32 {
        if let Entry::Link(l) = self {
            return LANE_W * (l.left() + 1) as f32;
        }
        if let Entry::Vein(v) = self {
            return LANE_VEIN[v.index()];
        }
        let (row, b) = (self.row(), self.branch());
        let x0 = LANE_W * b.index() as f32;
        let peers = |small: bool| {
            ENTRIES.iter().filter(move |e| {
                e.row() == row
                    && e.branch() == b
                    && !matches!(e, Entry::Link(_) | Entry::Vein(_))
                    && e.small() == small
            })
        };
        if row == ULT_ROW as usize {
            let k = peers(false).chain(peers(true)).position(|&e| e == self);
            return x0 + LANE_ULT[k.unwrap_or(0).min(1)];
        }
        let n = peers(self.small()).count();
        let k = peers(self.small()).position(|&e| e == self).unwrap_or(0);
        x0 + if n == 1 { LANE_1 } else { LANE_2[k.min(1)] }
    }

    /// 몸의 판의 길에 서는 칸인가(0270 C3): 패시브·다리·맺음. 스킬·큰 기술·결은 무기 판의 칸이라(0270 C) 길 밖이다 — 그 칸들의
    /// 열림은 무기 판의 줄(`plates::offer_on`)이 정한다.
    pub fn on_path(self) -> bool {
        matches!(self, Entry::Cell(_) | Entry::Link(_) | Entry::Key(_))
    }

    /// 몸의 판의 길의 줄(0270 C3, 0~4 — 첫 줄 패시브 · 둘째 줄 패시브(+ 다리) · 큰 기술 줄의 패시브 · 넷째 줄 패시브(+ 다리) · 맺음):
    /// 스킬·큰 기술이 빠진 `line`을 당긴 것. 길 밖의 칸은 `None`.
    pub fn path_line(self) -> Option<usize> {
        if !self.on_path() {
            return None;
        }
        [1, 3, 4, 6, 7].iter().position(|&l| l == self.line())
    }

    /// 몸의 판에서 같은 줄·갈래의 길 칸이 이것 하나뿐인가(다리 빼고) — 홀로 선 칸은 위·아래 줄의 갈래 칸 모두와 잇는다.
    fn alone(self) -> bool {
        let (row, b) = (self.row(), self.branch());
        ENTRIES
            .iter()
            .filter(|e| {
                e.on_path() && !matches!(e, Entry::Link(_)) && e.row() == row && e.branch() == b
            })
            .count()
            == 1
    }

    /// 판의 길(0215 B → 0270 C3 몸의 판): `self`를 가졌으면 `to`가 이어진다. 길은 몸의 판의 칸(패시브·다리·맺음)끼리만 — 첫 줄
    /// 패시브가 뿌리다(스킬이 무기 판으로 옮겨 가 몸의 판이 스킬 밑에서 끊기지 않게). 갈래 안에서는 바로 아래 줄의 가로로 가까운 칸으로
    /// (그 줄에 홀로 선 칸이면 갈래의 칸 모두와), 다리는 같은 줄의 맞닿은 칸에서 들어서고, 들어선 다리에서는 같은 줄·바로 아래 줄의
    /// 가까운 칸으로(양쪽 갈래) 간다.
    pub fn feeds(self, to: Entry) -> bool {
        let (Some(a), Some(b)) = (self.path_line(), to.path_line()) else {
            return false;
        };
        let near = (self.lane() - to.lane()).abs() <= LANE_DX;
        match (self, to) {
            (Entry::Link(_), Entry::Link(_)) => false,
            (Entry::Link(_), _) => near && (b == a || b == a + 1),
            (_, Entry::Link(_)) => near && b == a,
            _ => b == a + 1 && self.branch() == to.branch() && (near || self.alone() || to.alone()),
        }
    }

    /// 값 없이 가진 단: 없다(0172 — 쟁반 뒤집기도 점수로 배운다. 0166의 "처음부터 주는 것은 없다"를 큰 기술까지). 옛 기록이
    /// 값 없이 받았던 것은 기록의 몫으로 지닌다(`Board::with_upend` + `Progress::ult_gift`).
    pub fn free(self) -> u8 {
        0
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
    /// 다른 맺음이 켜져 있다 — 값 없이 바꿀 수 있다(0214). "살 것"으로 치지 않는다(쉼터의 빛·민트 점·반짝임은 없다).
    Swap,
    /// 길이 끊겼다(0215 B) — 바로 위 줄에서 이어진 칸을 먼저 가져야 한다.
    Cut,
    /// 맺음: 그 갈래에 점수를 이만큼 더 써야 한다(0215 A — `KEY_BRANCH`).
    Branch(u32),
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
    /// 칸의 단(패시브 칸은 열렸으면 1, 맺음은 켰으면 1).
    pub fn rank(&self, e: Entry) -> u8 {
        match e {
            Entry::Skill(id) => self.rank[id.index()],
            Entry::Ult(u) => self.ult[u.index()],
            Entry::Cell(c) => self.cells.has(c) as u8,
            Entry::Key(k) => self.cells.keyed(k) as u8,
            Entry::Link(l) => self.cells.linked(l) as u8,
            Entry::Vein(v) => self.cells.veined(v) as u8,
        }
    }

    /// 갈래 `b`에 쓴 점수(0215 A): 그 갈래의 스킬·큰 기술·패시브 칸 — 맺음·다리는 세지 않는다.
    pub fn branch_spent(&self, b: Branch) -> u32 {
        ENTRIES
            .iter()
            .filter(|e| {
                e.branch() == b && !matches!(e, Entry::Key(_) | Entry::Link(_) | Entry::Vein(_))
            })
            .map(|&e| e.paid(self.rank(e)))
            .sum()
    }

    /// 길이 이어졌나(0215 B → 0270 C3): 몸의 판의 첫 줄 패시브는 늘(뿌리), 그 밖은 가진 칸 가운데 이것으로 이어지는 것이 있어야.
    /// 이미 가진 칸(단을 올리는 것·옛 기록)은 묻지 않는다.
    pub fn reachable(&self, e: Entry) -> bool {
        // 결의 칸(0223 11절): 그 타입을 싣는 스킬을 하나라도 배웠어야 한다.
        if let Entry::Vein(v) = e {
            return self.rank(e) > 0
                || IDS
                    .into_iter()
                    .any(|id| self.learned(id) && id.def().kind == v.kind());
        }
        // 스킬·큰 기술은 무기 판의 줄로 연다(0270 C) — 몸의 판의 길을 묻지 않는다.
        !e.on_path()
            || e.path_line() == Some(0)
            || self.rank(e) > 0
            || ENTRIES
                .iter()
                .any(|&from| self.rank(from) > 0 && from.feeds(e))
    }

    /// 맺음을 바꾸는 일인가(0214): 다른 맺음을 이미 켰으면 이 맺음은 값 없이 바꿔 켠다(화로에서).
    pub fn key_swap(&self, e: Entry) -> bool {
        matches!(e, Entry::Key(k) if self.cells.key.is_some_and(|on| on != k))
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
        } else if !self.reachable(e) {
            Offer::Cut
        } else if let (Entry::Key(_), true) = (e, self.branch_spent(e.branch()) < KEY_BRANCH) {
            Offer::Branch(KEY_BRANCH - self.branch_spent(e.branch()))
        } else if self.key_swap(e) {
            // 켠 맺음이 있으면 값은 이미 냈다 — 바꾸기는 값이 없다(그 갈래의 점수는 바꿀 때도 본다).
            Offer::Swap
        } else if self.left(total) < e.price(self.rank(e)) {
            Offer::Short
        } else {
            Offer::Ready
        }
    }

    /// 다음 단을 산다(점수 `Entry::price`). 3단에 닿으면 고르기는 ㄱ.
    pub fn buy(&mut self, e: Entry, total: u32) -> bool {
        if !matches!(self.offer(e, total), Offer::Ready | Offer::Swap) {
            return false;
        }
        self.raise(e);
        true
    }

    /// 다음 단을 올린다 — 묻지 않는다(살 수 있나는 부른 쪽이: `buy` · 무기 판의 `plates::buy_on`, 0270 C2). 3단에 닿으면 고르기는 ㄱ.
    pub fn raise(&mut self, e: Entry) {
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
            Entry::Key(k) => self.cells.key = Some(k),
            Entry::Link(l) => self.cells.open_link(l),
            Entry::Vein(v) => self.cells.open_vein(v),
        }
    }

    /// 칸 하나를 처음(안 배움)으로 — 묻지 않는다(0270 C5: 어느 판에도 서지 않는 칸, `plates::drop_unseated`). 그 칸에 쓴 점수는
    /// 남은 점수로 돌아간다(`spent`는 산 단만 센다). 스킬의 고르기·숫자 칸의 고르기도 지운다.
    pub fn forget(&mut self, e: Entry) {
        match e {
            Entry::Skill(id) => {
                self.rank[id.index()] = 0;
                self.pick[id.index()] = Pick::A;
            }
            Entry::Ult(u) => self.ult[u.index()] = 0,
            Entry::Cell(c) => self.cells.close_cell(c),
            Entry::Key(k) => {
                if self.cells.key == Some(k) {
                    self.cells.key = None;
                }
            }
            Entry::Link(l) => self.cells.links &= !(1 << l.index()),
            Entry::Vein(v) => {
                self.cells.veins &= !(1 << v.index());
                self.cells.vein_picks &= !(1 << v.index());
            }
        }
    }

    /// 숫자 칸의 고르기를 뒤집는다(0215 D — 값 없이, 화로에서). 연 숫자 칸만.
    pub fn flip_cell(&mut self, c: Cell) -> bool {
        self.cells.flip_cell(c)
    }

    /// 속성 패시브의 고르기를 뒤집는다(0274 ② — 값 없이, 화로에서). 연 결 가운데 고를 것이 있는 것만.
    pub fn flip_vein(&mut self, v: Vein) -> bool {
        self.cells.flip_vein(v)
    }

    /// 살 수 있는 칸이 있나 — 판 하나의 셈(시험). 앱(쉼터의 빛·탭 머리의 점·튜토리얼)은 판마다 보는 `plates::can_buy`(0270 C2).
    #[cfg(test)]
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

    /// 전부 되돌린다: 스킬·큰 기술은 모두 안 배운 것으로(0166 — 처음의 둘도, 0172 — 쟁반 뒤집기도), 고르기는 지우고 패시브 칸도
    /// 모두 닫는다(0161). 끼운 자리는 그대로 둔다(빈 칸이 된다 — 다시 배우면 그 자리에 있다. 큰 기술도).
    pub fn respec(&mut self, great: &mut Ult) {
        *self = Board::default();
        *great = self.wearable_ult(*great);
    }

    /// 잔불을 내고 되돌린다(화로): 쓴 점수가 없거나 잔불이 모자라면 아무 일도 없다(거짓).
    pub fn respec_paid(&mut self, embers: &mut u32, great: &mut Ult) -> bool {
        let cost = self.respec_cost();
        if self.spent() == 0 || *embers < cost {
            return false;
        }
        *embers -= cost;
        self.respec(great);
        true
    }

    /// 막 배운 스킬 `id`를 빈 칸에(0166): 칸 1부터 보아 안 배운 스킬이 놓인 칸이 있으면 거기에 끼운다 — 처음 배운 스킬이
    /// 기술 화면에서 한 번 더 끼우지 않아도 버튼에 있다. 빈 칸이 없거나 이미 끼워져 있으면 그대로.
    pub fn wear_on_learn(&self, worn: [Id; 2], id: Id) -> [Id; 2] {
        if worn.contains(&id) {
            return worn;
        }
        match worn.iter().position(|&w| !self.learned(w)) {
            Some(slot) => {
                let mut out = worn;
                out[slot] = id;
                out
            }
            None => worn,
        }
    }

    /// 끼울 수 있는 큰 기술: 배운 것이면 그대로, 아니면 배운 것 가운데 첫째(표 순서). 하나도 안 배웠으면(0172 — 새 판) 그대로
    /// 돌려준다 — 그 단은 0이라 월드에서는 큰 기술이 없다(`Loadout::ult` 0: 게이지가 차지 않는다).
    pub fn wearable_ult(&self, great: Ult) -> Ult {
        if self.has_ult(great) {
            great
        } else {
            ULTS.into_iter().find(|&u| self.has_ult(u)).unwrap_or(great)
        }
    }

    /// 배운 큰 기술의 수(0172 — 둘 이상일 때만 기술 탭이 끼운 것을 표시한다).
    pub fn ults_known(&self) -> usize {
        ULTS.into_iter().filter(|&u| self.has_ult(u)).count()
    }

    /// 쟁반 뒤집기를 배운 판 — 0172 전에는 새 판이 이 꼴이었다(큰 기술 하나는 처음부터). 옛 기록을 읽을 때·개발 시나리오 판이
    /// 쓴다(이미 올린 단은 그대로).
    pub fn with_upend(mut self) -> Board {
        let i = Ult::Upend.index();
        self.ult[i] = self.ult[i].max(1);
        self
    }

    /// 기록에서 읽은 판을 바로잡는다: 단은 범위 안으로(처음의 둘·쟁반 뒤집기는 1단 아래로 내려가지 않는다), 3단이 안 된 고르기는
    /// 지우고, 가진 점수보다 많이 썼으면 처음으로 — 점수는 그대로 남으니 다시 찍으면 된다. 처음으로 돌리는 차례(0161): 먼저 스킬·
    /// 큰 기술만(연 패시브 칸은 남긴다 — 옛 기록의 칸은 핵·조각으로 열었고 그 둘은 지금도 점수라 칸만으로는 넘지 않는다. 0158의
    /// "보스 첫 쓰러뜨림 +1"이 핵의 수로 바뀌어 개발판 기록이 한두 점 넘칠 수 있다), 그래도 넘으면(손댄 기록) 칸까지.
    pub fn sane(self, total: u32) -> Board {
        let mut b = self.sane_ranks(total);
        // 결의 칸(0223 11절): 그 타입의 스킬을 모르면 닫는다(넘쳐서 스킬을 돌려받은 기록·손댄 기록).
        for v in VEINS {
            if !IDS
                .into_iter()
                .any(|id| b.learned(id) && id.def().kind == v.kind())
            {
                b.cells.veins &= !(1 << v.index());
            }
        }
        b
    }

    fn sane_ranks(mut self, total: u32) -> Board {
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

    /// 배운 스킬이 하나라도 있나(0166 — 없으면 스킬 칸 둘이 다 비어 있다).
    pub fn knows_a_skill(&self) -> bool {
        IDS.into_iter().any(|id| self.learned(id))
    }

    /// 처음의 둘(`START`)을 배운 판 — 0166 전에는 새 판이 이 꼴이었다. 옛 기록을 읽을 때·개발 시나리오 판·시험이 쓴다
    /// (이미 올린 단은 그대로).
    pub fn with_starters(mut self) -> Board {
        for id in START {
            self.rank[id.index()] = self.rank[id.index()].max(1);
        }
        self
    }

    /// 모두 끝 단·패시브 칸 모두 열림(개발 창 — 기록에 쓰지 않는다).
    #[cfg(any(test, feature = "dev"))]
    pub fn maxed(pick: Pick) -> Board {
        Board {
            rank: [MAX_RANK; N],
            pick: [pick; N],
            ult: [ULT_MAX_RANK; NU],
            // 맺음은 하나만(0214) — 첫째를 켠다.
            cells: Tree {
                key: Some(KEYS[0]),
                ..Tree::full()
            },
        }
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
    /// 화폐 "마디"(핵 + 조각)와 0158의 "보스 첫 쓰러뜨림 +1"을 이 하나가 대신한다. 처음부터 가진 단(큰 기술 1단)은 쓴 점수가 아니다.
    /// 0166 → 0167: 스킬은 값 없이 주지 않고, 새 판은 점수도 없이 선다 — 첫 점수는 첫 레벨에서.
    #[test]
    fn points_are_levels_plus_first_elites_plus_cores_plus_shards_minus_spent() {
        assert_eq!(points(0, 0, 0, 0), 0, "0167: a new run starts with none");
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
        assert_eq!(b.rank, [0; N], "0166: no skill is given");
        assert_eq!(
            (b.ult, b.spent()),
            (ult_ranks([]), 0),
            "0172: no great skill is given either"
        );
        let total = points(3, 1, 1, 0);
        assert_eq!(b.left(total), 5);
        assert!(b.buy(Entry::Skill(Id::Gather), total));
        assert!(b.buy(Entry::Skill(Id::Whirl), total));
        assert!(b.buy(Entry::Cell(Cell::UnbrokenStride), total));
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
        let cores = crate::stage::REGIONS as u32 * CORE_CAP;
        let shards: u32 = stages
            .iter()
            .map(|d| {
                let (run, _) = crate::stage::StageRun::start(d);
                run.hidden
                    .list
                    .iter()
                    .filter(|f| f.1 == crate::stage::Find::Shard)
                    .count() as u32
            })
            .sum();
        // 0185: 셋째 지역(엘리트 셋·핵 셋·조각 둘)이 더해져 30 → 38. 0208: 넷째 지역(엘리트 셋·핵 셋·조각 하나) → 45.
        assert_eq!((levels, elites, cores, shards), (15, 12, 12, 6));
        let all = points(levels, elites, cores, shards);
        assert_eq!(all, 45);
        // 다 익히는 값(65)에는 못 미친다 — 고르게 한다(0160 2절). 여정의 기술 점수(지역마다 하나, 0183)를 더해도.
        assert!(all + (crate::stage::REGIONS as u32) < Board::maxed(Pick::A).spent());
    }

    /// 0161 → 0215 → 0270 C3: 패시브 칸은 판의 칸이다 — 칸마다 1점. 줄(쓴 점수 합)이 "언제"를, 길(바로 위 줄의 이어진 칸 —
    /// 몸의 판의 칸끼리)이 "어디로"를 정한다. 첫 줄 패시브는 처음부터 산다(스킬이 무기 판으로 옮겨 갔다). 연 칸은 다시 못 산다.
    #[test]
    fn passive_cells_cost_one_point_and_are_gated_by_rows_and_paths() {
        let total = 30;
        let mut b = Board::default();
        let (second, third, fifth) = (
            Entry::Cell(Cell::FlowingBeat),
            Entry::Cell(Cell::ClosingBlow),
            Entry::Cell(Cell::FinalMeasure),
        );
        assert_eq!((second.price(0), second.max(), second.free()), (1, 1, 0));
        // 첫 줄 패시브는 뿌리 — 스킬 없이도 산다(0270 C3).
        assert_eq!(b.offer(second, 0), Offer::Short);
        assert_eq!(b.offer(second, total), Offer::Ready);
        // 갈래의 첫 칸(고른 호흡)을 열지 않고 둘째 칸부터.
        assert!(b.buy(second, total));
        assert!(b.cells.has(Cell::FlowingBeat) && !b.cells.has(Cell::EvenBreath));
        assert_eq!((b.spent(), b.rank(second)), (1, 1));
        assert_eq!(b.offer(second, total), Offer::Full);
        assert!(!b.buy(second, total), "an open cell is not bought twice");
        // 둘째 줄: 쓴 점수 4부터(지금 1 — 무기 판에 쓴 점수도 센다).
        assert_eq!(b.offer(third, total), Offer::Locked(3));
        assert_eq!(b.offer(fifth, total), Offer::Locked(7));
        for _ in 0..3 {
            assert!(b.buy(Entry::Skill(Id::Whirl), total));
        }
        // 줄이 열려도 길이 있어야: 맺음타(왼쪽)는 고른 호흡(왼쪽) 밑 — 흐르는 박자(오른쪽)로는 안 이어진다.
        assert_eq!(b.offer(third, total), Offer::Cut);
        assert_eq!(b.offer(Entry::Cell(Cell::GapBeat), total), Offer::Ready);
        assert!(b.buy(Entry::Cell(Cell::EvenBreath), total));
        assert_eq!(b.offer(third, total), Offer::Ready);
        // 넷째 칸을 셋째 칸보다 먼저(같은 줄).
        assert!(b.buy(Entry::Cell(Cell::GapBeat), total));
        assert!(!b.cells.has(Cell::ClosingBlow));
        // 다른 갈래에 쓴 점수도 줄을 연다: 쓴 점수 8 — 셋째 줄.
        for e in [Entry::Skill(Id::Brace), Entry::Cell(Cell::Catch)] {
            assert_eq!(b.offer(fifth, total), Offer::Locked(8 - b.spent()));
            assert!(b.buy(e, total));
        }
        assert_eq!(b.spent(), 8);
        // 마지막 박자(그 줄에 홀로 선 칸)는 끝 틈 박자 밑.
        assert!(b.buy(fifth, total));
        assert!(b.cells.has(Cell::FinalMeasure));
        // 점수가 떨어지면 열린 줄의 칸도 못 연다.
        assert_eq!(
            b.offer(Entry::Cell(Cell::Sidestep), b.spent()),
            Offer::Short
        );
        assert!(b.can_buy(total) && !b.can_buy(b.spent()));
    }

    /// 0215 B → 0270 C3: 판의 길(몸의 판의 칸끼리 — 패시브·다리·맺음). 갈래 안에서는 바로 아래 줄의 가로로 가까운 칸만
    /// 이어진다(왼쪽 ↔ 오른쪽은 아니다 — 그 줄에 홀로 선 칸은 갈래의 칸 모두와), 갈래가 다르면 다리 칸만 잇는다. 다리는 같은 줄의
    /// 맞닿은 칸에서 들어서고, 들어서면 양쪽 갈래의 바로 아래 줄로 간다. 스킬·큰 기술·결은 길 밖이다(무기 판의 칸). 이미 가진 칸은
    /// 길을 묻지 않는다(옛 기록·단 올리기).
    #[test]
    fn the_board_has_paths_and_bridges_join_neighbouring_branches() {
        use Entry::{Cell as C, Key as K, Link as L, Skill as S, Ult as U};
        // 판의 줄 아홉(화면의 자리), 몸의 길의 줄 다섯.
        let lines: Vec<usize> = ENTRIES.iter().map(|e| e.line()).collect();
        assert_eq!((0..9).filter(|l| lines.contains(l)).count(), 9, "{lines:?}");
        assert!(lines.iter().all(|&l| l < 9));
        let paths: Vec<usize> = ENTRIES.iter().filter_map(|e| e.path_line()).collect();
        assert_eq!((0..5).filter(|l| paths.contains(l)).count(), 5, "{paths:?}");
        // 첫 줄 패시브 말고 몸의 판의 모든 칸은 위에서 들어서는 길이 있다(갇힌 칸이 없다) — 스킬·큰 기술·결의 칸은 길 밖이다.
        for &e in &ENTRIES {
            if !e.on_path() {
                assert!(!ENTRIES.iter().any(|&f| f.feeds(e) || e.feeds(f)), "{e:?}");
                assert!(Board::default().reachable(e) || matches!(e, Entry::Vein(_)));
                continue;
            }
            assert!(
                e.path_line() == Some(0) || ENTRIES.iter().any(|&f| f.feeds(e)),
                "{e:?} has no way in"
            );
        }
        // 갈래 안: 왼쪽 ↔ 오른쪽은 안 이어지고, 홀로 선 칸은 둘 다와.
        assert!(
            C(Cell::EvenBreath).feeds(C(Cell::ClosingBlow))
                && !C(Cell::EvenBreath).feeds(C(Cell::GapBeat))
        );
        assert!(
            C(Cell::ClosingBlow).feeds(C(Cell::FinalMeasure))
                && C(Cell::GapBeat).feeds(C(Cell::FinalMeasure))
        );
        assert!(
            C(Cell::FinalMeasure).feeds(C(Cell::HeldBreath))
                && C(Cell::FinalMeasure).feeds(C(Cell::GatheringBeat))
        );
        assert!(C(Cell::HeldBreath).feeds(K(Key::Measure)));
        assert!(
            !C(Cell::ClosingBlow).feeds(C(Cell::EvenBreath)),
            "paths only go down"
        );
        assert!(
            !S(Id::Whirl).feeds(C(Cell::EvenBreath)),
            "skills are off the path"
        );
        assert!(!U(Ult::Flurry).feeds(C(Cell::HeldBreath)));
        // 갈래가 달라 가까워도 안 이어진다 — 다리만.
        assert!(!C(Cell::GapBeat).feeds(C(Cell::GatheredRiposte)));
        assert!(C(Cell::GapBeat).feeds(L(Link::RiposteBeat)));
        assert!(C(Cell::GatheredRiposte).feeds(L(Link::RiposteBeat)));
        assert!(L(Link::RiposteBeat).feeds(C(Cell::FinalMeasure)));
        assert!(L(Link::ParryStep).feeds(C(Cell::TiltingGround)));
        // 흐름 ↔ 자리는 맞닿지 않는다(가장 먼 혼종은 비싸다).
        for &e in &ENTRIES {
            if e.branch() == Branch::Footing && !matches!(e, Entry::Link(_)) {
                assert!(!L(Link::RiposteBeat).feeds(e), "{e:?}");
            }
        }
        // 다리를 건너: 흐름만 걸은 판이 맞받기의 둘째 줄 칸을 산다.
        let total = 40;
        let mut b = Board::default();
        for e in [
            S(Id::Cut),
            S(Id::Cut),
            S(Id::Cut),
            C(Cell::FlowingBeat),
            C(Cell::GapBeat),
        ] {
            assert!(b.buy(e, total), "{e:?}");
        }
        let breach = C(Cell::GatheredRiposte);
        assert_eq!(b.offer(breach, total), Offer::Cut);
        assert_eq!(b.offer(L(Link::RiposteBeat), total), Offer::Ready);
        assert!(b.buy(L(Link::RiposteBeat), total));
        assert_eq!(b.offer(breach, total), Offer::Ready);
        // 다리 칸은 어느 갈래의 점수도 아니다.
        assert_eq!(
            (
                b.branch_spent(Branch::Beat),
                b.branch_spent(Branch::Riposte)
            ),
            (5, 0)
        );
        // 가진 칸은 길을 묻지 않는다(옛 기록 — 길이 끊긴 채 산 칸).
        let mut old = Board::default();
        old.cells.open_cell(Cell::HeldBreath);
        assert!(old.reachable(C(Cell::HeldBreath)));
        assert!(!old.reachable(C(Cell::GatheringBeat)));
    }

    /// 0215 A: 맺음은 그 갈래에 쓴 점수 `KEY_BRANCH` 10부터 — 줄(22)이 열리고 길이 이어져도. 다른 맺음으로 바꿀 때도 같다.
    #[test]
    fn a_keystone_needs_ten_points_in_its_own_branch() {
        use Entry::{Cell as C, Skill as S, Ult as U};
        assert_eq!(KEY_BRANCH, 10);
        let total = 40;
        let mut b = Board::default();
        // 맞받기의 길(휘돌이 · 가로지르기 · 모으기는 줄만 연다 — 0270 C3: 길은 몸의 판의 칸끼리): 받아내기 → 모은 맞받기 →
        // 서 있는 자 → 되찾는 생기, 그 갈래의 점수는 막아서기·되울림·줄 긋기까지.
        for e in [[S(Id::Whirl); 5], [S(Id::Cut); 5]].concat() {
            assert!(b.buy(e, total));
        }
        for e in [
            S(Id::Brace),
            C(Cell::Catch),
            S(Id::Echo),
            C(Cell::GatheredRiposte),
            U(Ult::Line),
            C(Cell::OneWhoStands),
            C(Cell::Mend),
            S(Id::Gather),
            S(Id::Gather),
            S(Id::Gather),
            S(Id::Gather),
        ] {
            assert!(b.buy(e, total), "{e:?}");
        }
        assert_eq!(b.spent(), 22);
        assert_eq!(b.branch_spent(Branch::Riposte), 8);
        let edge = Entry::Key(Key::Edge);
        assert_eq!(b.offer(edge, total), Offer::Branch(2));
        // 흐름은 10이지만 길이 없다 — 흐름의 넷째 줄 패시브가 없다.
        assert_eq!(b.branch_spent(Branch::Beat), 10);
        assert_eq!(b.offer(Entry::Key(Key::Measure), total), Offer::Cut);
        assert!(b.buy(S(Id::Brace), total) && b.buy(S(Id::Brace), total));
        assert_eq!(b.offer(edge, total), Offer::Ready);
        assert!(b.buy(edge, total));
        // 바꾸기도 그 갈래에 10을 써 두어야(자리는 6 — 길도 없다).
        let rooted = Entry::Key(Key::Rooted);
        assert_eq!(b.offer(rooted, total), Offer::Cut);
    }

    /// 0215 D: 숫자 칸 셋은 둘 가운데 하나 — 열면 I, 값 없이 뒤집는다(연 칸만). 기록 꼴로 왕복하고, 열리지 않은 칸의 고르기는
    /// 바로잡을 때 지운다.
    #[test]
    fn number_cells_pick_one_of_two_and_flip_for_free() {
        use crate::progression::TREE;
        let numbers: Vec<Cell> = TREE
            .iter()
            .flat_map(|r| r.iter().filter(|d| d.number).map(|d| d.cell))
            .collect();
        assert_eq!(
            numbers,
            [Cell::EvenBreath, Cell::Catch, Cell::UnbrokenStride]
        );
        for row in &TREE {
            for d in row {
                assert_eq!(d.cell.pick_b().is_some(), d.number, "{:?}", d.cell);
            }
        }
        let mut b = Board::default();
        assert!(!b.flip_cell(Cell::Catch), "not open yet");
        assert!(b.buy(Entry::Skill(Id::Brace), 9) && b.buy(Entry::Cell(Cell::Catch), 9));
        assert!(b.cells.picked_a(Cell::Catch) && !b.cells.picked_b(Cell::Catch));
        let spent = b.spent();
        assert!(b.flip_cell(Cell::Catch));
        assert!(b.cells.picked_b(Cell::Catch) && !b.cells.picked_a(Cell::Catch));
        assert_eq!(b.spent(), spent, "free");
        assert!(b.cells.has(Cell::Catch), "still open");
        assert!(!b.flip_cell(Cell::TurnItBack), "not a number cell");
        // 바로잡기: 닫힌 칸의 고르기 비트는 지운다.
        let mut odd = b.cells;
        odd.picks[0] = 0xff;
        assert_eq!(odd.sane().picks[0], 0);
        assert_eq!(odd.sane().picks[1], b.cells.picks[1]);
    }

    /// 0161 "갈래와 줄"의 표 그대로: 줄마다 갈래마다 선 칸(스킬·큰 기술·패시브). 판의 칸 순서는 줄 → 갈래 → 스킬 먼저.
    #[test]
    fn the_board_stands_as_the_0161_table_says() {
        use Branch::{Beat, Footing, Riposte};
        use Entry::{Cell as C, Link as L, Skill as S, Ult as U};
        let table: [(usize, Branch, &[Entry]); 45] = [
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
            // 0270 C3: 무기 판을 채운 새 스킬 — 옛 판의 자리는 그 줄의 끝.
            (0, Beat, &[S(Id::Stitch)]),
            (0, Riposte, &[S(Id::Toll)]),
            (0, Beat, &[S(Id::Swat)]),
            (0, Footing, &[S(Id::Flare), S(Id::Snare), S(Id::Fuse)]),
            // 0272 2단계: 창 판.
            (0, Beat, &[S(Id::Skewer)]),
            (0, Riposte, &[S(Id::Pin)]),
            // 0274: 한손검 판의 전투 기술.
            (0, Riposte, &[S(Id::Pommel)]),
            // 0274 ④: 대검 판.
            (0, Beat, &[S(Id::Sear)]),
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
            // 다리 칸(0215): 잇는 둘 가운데 왼쪽 갈래로 센다.
            (1, Beat, &[L(Link::RiposteBeat)]),
            (1, Riposte, &[L(Link::ParryStep)]),
            (1, Riposte, &[S(Id::Ram), S(Id::Ripple)]),
            (1, Footing, &[S(Id::Snow), S(Id::Mine)]),
            (1, Footing, &[S(Id::Pole)]),
            (1, Footing, &[S(Id::Shove)]),
            (1, Riposte, &[S(Id::Rally)]),
            // 0274 ③: 쌍단검 판.
            (1, Beat, &[S(Id::Slip)]),
            (1, Footing, &[S(Id::Sunder)]),
            (2, Beat, &[U(Ult::Flurry), C(Cell::FinalMeasure)]),
            (2, Riposte, &[U(Ult::Line), C(Cell::OneWhoStands)]),
            (2, Footing, &[U(Ult::Upend), C(Cell::PathOpeningStep)]),
            (2, Riposte, &[U(Ult::Bell)]),
            (2, Footing, &[U(Ult::Blizzard), U(Ult::Rain)]),
            (2, Riposte, &[U(Ult::Gale)]),
            (2, Beat, &[U(Ult::Ambush)]),
            (2, Beat, &[U(Ult::Tempest)]),
            (
                3,
                Beat,
                &[S(Id::Swallow), C(Cell::HeldBreath), C(Cell::GatheringBeat)],
            ),
            (
                3,
                Riposte,
                &[S(Id::Bash), C(Cell::SecondWind), C(Cell::Mend)],
            ),
            (
                3,
                Footing,
                &[S(Id::Roots), C(Cell::Planted), C(Cell::Snare)],
            ),
            (3, Beat, &[L(Link::BreakingBeat)]),
            (3, Riposte, &[L(Link::RootedRiposte)]),
            (4, Beat, &[Entry::Key(Key::Measure)]),
            (4, Riposte, &[Entry::Key(Key::Edge)]),
            (4, Footing, &[Entry::Key(Key::Rooted)]),
            // 결의 줄(0223 11절): 타입 넷 — 빛깔만 그 타입의 스킬이 많은 갈래에 기댄다.
            (
                5,
                Footing,
                &[Entry::Vein(Vein::Ember), Entry::Vein(Vein::Root)],
            ),
            (5, Riposte, &[Entry::Vein(Vein::Knell)]),
            (5, Beat, &[Entry::Vein(Vein::Rift)]),
            (
                5,
                Beat,
                &[Entry::Vein(Vein::Shock), Entry::Vein(Vein::Sharp)],
            ),
            (5, Footing, &[Entry::Vein(Vein::Frost)]),
            (5, Riposte, &[Entry::Vein(Vein::Drain)]),
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
        assert_eq!(NE, 75);
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

    /// 0158 다: 단은 차례로 하나씩·단마다 1점, 다섯에서 멈춘다. 점수가 떨어지면 못 산다. 3단을 사면 고르기는 ㄱ.
    #[test]
    fn ranks_are_bought_in_order_one_point_each_and_stop_at_five() {
        let mut b = Board::default();
        let whirl = Entry::Skill(Id::Whirl);
        assert_eq!(b.offer(whirl, 0), Offer::Short);
        assert!(!b.buy(whirl, 0));
        for want in 1..=MAX_RANK {
            let spent = b.spent();
            assert_eq!(b.offer(whirl, 5), Offer::Ready);
            assert!(b.buy(whirl, 5));
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
        assert_eq!(b.left(5), 0);
        assert_eq!(b.offer(Entry::Skill(Id::Cut), 5), Offer::Short);
        assert!(!b.can_buy(5) && b.can_buy(6));
        // 안 배운 스킬의 첫 점 = 배움(1단).
        let mut b = Board::default();
        assert!(!b.learned(Id::Gather));
        assert!(b.buy(Entry::Skill(Id::Gather), 1));
        assert!(b.learned(Id::Gather));
        assert_eq!((b.rank(Entry::Skill(Id::Gather)), b.spent()), (1, 1));
    }

    /// 0223 11절: 결의 칸은 그 타입을 싣는 스킬을 배워야 열린다(쓴 점수·판의 길과 상관없이), 값은 1점, 어느 갈래의 점수도
    /// 아니고(맺음의 문턱을 당기지 않는다), 되돌리면 닫힌다. 타입마다 그 타입의 스킬이 판에 있다(열 수 없는 결은 없다).
    #[test]
    fn a_vein_opens_once_a_skill_of_its_type_is_known_and_counts_for_no_branch() {
        use crate::progression::BRANCHES;
        let total = 30;
        let mut b = Board::default();
        for v in VEINS {
            assert_eq!(b.offer(Entry::Vein(v), total), Offer::Cut, "{v:?}");
            assert!(
                IDS.into_iter().any(|id| id.def().kind == v.kind()),
                "{v:?} has a skill"
            );
        }
        assert_eq!(Id::Cut.def().kind, Vein::Rift.kind());
        assert!(b.buy(Entry::Skill(Id::Cut), total));
        let rift = Entry::Vein(Vein::Rift);
        assert_eq!(b.offer(rift, total), Offer::Ready);
        assert_eq!(
            b.offer(rift, 1),
            Offer::Short,
            "the point went on the skill"
        );
        assert_eq!(b.offer(Entry::Vein(Vein::Ember), total), Offer::Cut);
        let (spent, by_branch) = (b.spent(), BRANCHES.map(|x| b.branch_spent(x)));
        assert!(b.buy(rift, total));
        assert!(b.cells.veined(Vein::Rift) && !b.cells.veined(Vein::Ember));
        assert_eq!(b.spent(), spent + 1);
        assert_eq!(BRANCHES.map(|x| b.branch_spent(x)), by_branch);
        assert_eq!(b.offer(rift, total), Offer::Full);
        assert!(!b.buy(rift, total));
        let mut great = Ult::Upend;
        b.respec(&mut great);
        assert!(!b.cells.veined(Vein::Rift));
        assert_eq!(b.spent(), 0);
    }

    /// 0158 라: 줄은 **쓴 점수의 합**으로 열린다 — 둘째 줄(땅울림) 4, 큰 기술 줄 8. 어디에 썼든 상관없다. 잠긴 칸은 점수가 있어도 못 산다.
    #[test]
    fn rows_open_by_points_spent_wherever_they_went() {
        // 끝의 0은 결의 줄(0223 11절) — 점수가 아니라 그 타입의 스킬이 연다.
        assert_eq!(ROW_NEEDS, [0, 4, 8, 14, 22, 0]);
        for id in [Id::Whirl, Id::Cut, Id::Gather, Id::Brace] {
            assert_eq!(Entry::Skill(id).row(), 0, "{id:?}");
        }
        for id in [Id::Quake, Id::Echo, Id::Throw, Id::Embers] {
            assert_eq!(Entry::Skill(id).row(), 1, "{id:?}");
        }
        for u in ULTS {
            assert_eq!(Entry::Ult(u).row(), ULT_ROW as usize);
        }
        let (quake, ult) = (Entry::Skill(Id::Quake), Entry::Ult(Ult::Flurry));
        let total = 23;
        let mut b = Board::default();
        assert_eq!(b.offer(quake, total), Offer::Locked(4));
        assert_eq!(b.offer(ult, total), Offer::Locked(8));
        assert!(!b.buy(quake, total) && !b.buy(ult, total));
        // 셋을 한 스킬에 쓰고 패시브 하나(0270 C3 — 몸의 판의 첫 줄): 쓴 점수 넷이면 열린다.
        for n in 1..=3 {
            assert!(b.buy(Entry::Skill(Id::Cut), total));
            assert_eq!(b.offer(quake, total), Offer::Locked(4 - n));
        }
        assert!(b.buy(Entry::Cell(Cell::FlowingBeat), total));
        assert_eq!(b.offer(quake, total), Offer::Ready);
        assert_eq!(b.offer(ult, total), Offer::Locked(4));
        assert!(b.buy(quake, total) && b.buy(quake, total));
        // 다른 갈래에 쓴 점수도 줄을 연다.
        assert!(b.buy(Entry::Skill(Id::Gather), total));
        assert_eq!(b.offer(ult, total), Offer::Locked(1));
        assert!(b.buy(Entry::Cell(Cell::GapBeat), total));
        assert_eq!(b.spent(), 8);
        assert_eq!(b.offer(ult, total), Offer::Ready);
        // 큰 기술: 첫 단 2점, 그 뒤 1점씩 — 넷.
        assert!(b.buy(ult, total) && b.buy(ult, total) && b.buy(ult, total));
        assert_eq!(b.spent(), 12);
        assert_eq!(
            (b.ult[Ult::Flurry.index()], b.offer(ult, total)),
            (ULT_MAX_RANK, Offer::Full)
        );
        // 다 익히는 데 드는 점수(0158 라 → 0161 → 0166 → 0172 → 0214): 스킬 열하나 × 5 = 55(처음의 둘도 값이 있다), 큰 기술 셋 × 4 = 12
        // (쟁반 뒤집기도 배운다), 패시브 스물하나 × 1 = 21, 맺음 하나 1 — 89.
        let all = Board::maxed(Pick::A);
        let paid = |pred: fn(&Entry) -> bool| -> u32 {
            ENTRIES
                .iter()
                .filter(|e| pred(e))
                .map(|&e| e.paid(all.rank(e)))
                .sum()
        };
        // 0270 C3: 스킬·큰 기술은 무기 판을 채우며 늘었다 — 스킬마다 5, 큰 기술마다 4.
        assert_eq!(
            paid(|e| matches!(e, Entry::Skill(_))),
            N as u32 * MAX_RANK as u32
        );
        assert_eq!(paid(|e| matches!(e, Entry::Ult(_))), NU as u32 * 4);
        assert_eq!(paid(|e| matches!(e, Entry::Cell(_))), 21);
        assert_eq!(paid(|e| matches!(e, Entry::Key(_))), 1, "one keystone");
        assert_eq!(
            paid(|e| matches!(e, Entry::Link(_))),
            4,
            "0215: four bridges"
        );
        assert_eq!(
            paid(|e| matches!(e, Entry::Vein(_))),
            VEINS.len() as u32,
            "0223: a vein per type"
        );
        assert_eq!(
            all.spent(),
            (N * MAX_RANK as usize + NU * 4 + 21 + 1 + 4 + VEINS.len()) as u32
        );
        // 판의 칸 순서: 줄 번호가 내려가지 않는다.
        let rows: Vec<usize> = ENTRIES.iter().map(|e| e.row()).collect();
        assert!(rows.windows(2).all(|w| w[0] <= w[1]), "{rows:?}");
    }

    /// 0214: 맺음은 다섯째 줄(쓴 점수 22) — 1점에 하나를 켜고, 켠 뒤에는 다른 맺음으로 값 없이 바꾼다(하나만). 바꾸기는 "살 것"이
    /// 아니다(쉼터가 빛나지 않는다). 되돌리면 꺼진다. 옛 꼴의 수는 넷째 줄 칸을 열지 않는다(`LEGACY_DEPTH`).
    #[test]
    fn only_one_keystone_is_lit_and_swapping_it_is_free() {
        let total = 40;
        let mut b = Board::default();
        let (measure, edge) = (Entry::Key(Key::Measure), Entry::Key(Key::Edge));
        assert_eq!(b.offer(measure, total), Offer::Locked(22));
        // 넷째 줄 패시브는 14부터.
        assert_eq!(b.offer(Entry::Cell(Cell::Mend), total), Offer::Locked(14));
        // 흐름의 길(0215 → 0270 C3 몸의 판): 고른 호흡 → 맺음타 → 마지막 박자 → 이어지는 숨(스킬·큰 기술은 줄만 연다).
        let flow = [
            [Entry::Skill(Id::Whirl); 5].as_slice(),
            &[Entry::Skill(Id::Cut); 5],
            &[
                Entry::Cell(Cell::EvenBreath),
                Entry::Skill(Id::Quake),
                Entry::Cell(Cell::ClosingBlow),
                Entry::Ult(Ult::Flurry),
                Entry::Cell(Cell::FinalMeasure),
                Entry::Cell(Cell::HeldBreath),
            ],
        ]
        .concat();
        for e in flow {
            assert!(b.buy(e, total), "{e:?}");
        }
        assert_eq!(b.spent(), 17);
        assert_eq!(b.offer(measure, total), Offer::Locked(5));
        for _ in 0..4 {
            assert!(b.buy(Entry::Skill(Id::Brace), total));
        }
        assert_eq!(b.offer(measure, total), Offer::Locked(1));
        assert!(b.buy(Entry::Cell(Cell::Catch), total));
        assert_eq!(b.spent(), 22);
        assert_eq!(b.offer(measure, total), Offer::Ready);
        assert!(b.buy(measure, total));
        assert_eq!((b.spent(), b.cells.key), (23, Some(Key::Measure)));
        assert_eq!(b.offer(measure, total), Offer::Full);
        // 다른 맺음도 그 갈래의 길 끝에서(0215 → 0270 C3): 맞받기의 넷째 줄 패시브가 없으면 끊겨 있다.
        assert_eq!(b.offer(edge, 23), Offer::Cut);
        for e in [
            Entry::Skill(Id::Echo),
            Entry::Cell(Cell::GatheredRiposte),
            Entry::Ult(Ult::Line),
            Entry::Cell(Cell::OneWhoStands),
            Entry::Cell(Cell::Mend),
        ] {
            assert!(b.buy(e, total), "{e:?}");
        }
        assert!(b.branch_spent(Branch::Riposte) >= KEY_BRANCH);
        let spent = b.spent();
        // 다른 맺음: 값 없이 바꾼다 — 점수가 없어도.
        assert_eq!(b.offer(edge, spent), Offer::Swap);
        assert!(!b.can_buy(spent), "a swap is not something to buy");
        assert!(b.buy(edge, spent));
        assert_eq!((b.spent(), b.cells.key), (spent, Some(Key::Edge)));
        assert_eq!(b.rank(measure), 0);
        let mut great = Ult::Upend;
        b.respec(&mut great);
        assert_eq!(b.cells.key, None);
    }

    /// 0158 나: 되돌리기는 쓴 점수를 모두 돌려준다 — 스킬은 모두 안 배운 것으로(0166: 처음의 둘도)·큰 기술 1단, 고르기는 지워진다.
    /// 값은 `30 + 5 × 쓴 점수`. 끼운 큰 기술은 쟁반 뒤집기로.
    #[test]
    fn a_respec_returns_every_point_and_clears_picks() {
        assert_eq!(Board::default().respec_cost(), RESPEC_BASE);
        let total = 12;
        let mut b = Board::default();
        for e in [Entry::Skill(Id::Whirl); 5] {
            assert!(b.buy(e, total));
        }
        assert!(b.flip(Id::Whirl));
        assert!(b.buy(Entry::Cell(Cell::EvenBreath), total));
        assert!(b.buy(Entry::Skill(Id::Quake), total) && b.buy(Entry::Skill(Id::Gather), total));
        assert_eq!(b.spent(), 8);
        assert_eq!(b.respec_cost(), 30 + 5 * 8);
        // 0161: 패시브 칸도 쓴 점수다 — 값에 들어가고, 되돌리면 같이 닫힌다.
        assert!(
            b.buy(Entry::Cell(Cell::UnbrokenStride), total)
                && b.buy(Entry::Cell(Cell::ClosingBlow), total)
        );
        assert!(b.cells.flip_cell(Cell::UnbrokenStride));
        assert_eq!((b.spent(), b.respec_cost()), (10, 30 + 5 * 10));
        let mut great = Ult::Upend;
        b.respec(&mut great);
        assert_eq!(b, Board::default());
        assert_eq!(b.cells.count(), 0, "a respec closes every passive cell");
        assert!(!b.cells.has(Cell::UnbrokenStride) && !b.cells.has(Cell::ClosingBlow));
        assert_eq!(
            b.cells.picks, [0; 3],
            "0215: the passive picks are cleared too"
        );
        assert_eq!((b.spent(), b.left(total)), (0, total));
        assert_eq!(b.pick, [Pick::A; N]);
        assert!(
            IDS.iter().all(|&id| !b.learned(id)),
            "no skill stays learned — not even the first two"
        );
        assert_eq!(great, Ult::Upend);
    }

    /// 되돌리기는 잔불을 낸다 — 모자라면 아무 일도 없고(판·잔불 그대로), 쓴 점수가 없으면 낼 것도 없다.
    #[test]
    fn a_respec_charges_embers_and_does_nothing_when_short_or_unspent() {
        let total = 10;
        let mut b = Board::default();
        let mut embers = 500;
        let mut great = Ult::Upend;
        assert!(!b.respec_paid(&mut embers, &mut great), "nothing spent");
        assert_eq!(embers, 500);
        assert!(b.buy(Entry::Skill(Id::Gather), total));
        assert!(b.buy(Entry::Skill(Id::Gather), total));
        assert!(b.buy(Entry::Skill(Id::Cut), total));
        let (cost, before) = (b.respec_cost(), b);
        assert_eq!(cost, 30 + 5 * 3);
        let mut poor = cost - 1;
        assert!(!b.respec_paid(&mut poor, &mut great), "one ember short");
        assert_eq!((b, poor), (before, cost - 1));
        assert!(b.respec_paid(&mut embers, &mut great));
        assert_eq!(embers, 500 - cost);
        assert_eq!((b, b.left(total)), (Board::default(), total));
        // 다시 찍으면 값은 처음 값부터.
        assert_eq!(b.respec_cost(), RESPEC_BASE);
    }

    /// 0166 → 0167: 처음의 둘(휘돌이·가로지르기)도 값 없이 주지 않는다 — 새 판은 스킬을 하나도 모르고 점수도 없다. 첫 레벨이
    /// 첫 점수이고, 무엇을 먼저 배울지는 방랑자가 고른다(첫 줄의 스킬 넷·패시브 여섯 가운데). 큰 기술도 하나도 없다(0172 — 쟁반
    /// 뒤집기도 배운다): 끼울 것이 없으니 월드에 넘기는 큰 기술의 단은 0이다.
    #[test]
    fn a_fresh_board_knows_no_skill_and_has_no_points() {
        let b = Board::default();
        assert!(IDS.iter().all(|&id| !b.learned(id)) && !b.knows_a_skill());
        assert!(ULTS.iter().all(|&u| !b.has_ult(u)) && b.ults_known() == 0);
        assert_eq!(b.loadout(START, Ult::Upend).ult, 0, "no great skill yet");
        assert_eq!((points(0, 0, 0, 0), b.spent()), (0, 0));
        assert!(!b.can_buy(0), "nothing to learn before the first level");
        // 첫 레벨의 한 점: 첫 줄의 어느 스킬이든, 몸의 판의 첫 줄 패시브든 — 하나만(0270 C3 — 몸의 판의 뿌리는 첫 줄 패시브).
        let one = points(1, 0, 0, 0);
        assert_eq!(
            b.offer(Entry::Cell(Cell::ClosingBlow), one),
            Offer::Locked(4)
        );
        for e in [
            Entry::Cell(Cell::Catch),
            Entry::Skill(Id::Whirl),
            Entry::Skill(Id::Gather),
            Entry::Skill(Id::Brace),
        ] {
            let mut pick = b;
            assert_eq!(pick.offer(e, one), Offer::Ready, "{e:?}");
            assert!(pick.buy(e, one));
            assert_eq!(pick.left(one), 0);
            assert_eq!(pick.offer(Entry::Skill(Id::Cut), one), Offer::Short);
        }
        // 옛 기록의 몫은 처음의 둘의 값과 같다.
        assert_eq!(Board::default().with_starters().spent(), GIFT_POINTS);
    }

    /// 0166: 막 배운 스킬은 빈 칸(안 배운 스킬이 놓인 자리)에 저절로 끼워진다 — 칸 1부터. 빈 칸이 없으면 그대로(끼우는 것은
    /// 기술 화면에서), 이미 끼워져 있으면 그대로. 끼운 둘의 단은 판에서 — 안 배운 스킬이 놓인 칸은 단 0(빈 칸)이다.
    #[test]
    fn a_newly_learned_skill_fills_the_first_empty_slot() {
        let mut b = Board::default();
        let l = b.loadout(START, Ult::Upend);
        assert_eq!(
            (l.slots[0].rank, l.slots[1].rank),
            (0, 0),
            "both slots start empty"
        );
        assert!(b.buy(Entry::Skill(Id::Gather), 4));
        let worn = b.wear_on_learn(START, Id::Gather);
        assert_eq!(worn, [Id::Gather, Id::Cut], "into slot 1");
        let l = b.loadout(worn, Ult::Upend);
        assert_eq!((l.slots[0].rank, l.slots[1].rank), (1, 0));
        assert!(b.buy(Entry::Skill(Id::Whirl), 4));
        let worn = b.wear_on_learn(worn, Id::Whirl);
        assert_eq!(worn, [Id::Gather, Id::Whirl], "the second one into slot 2");
        assert!(b.buy(Entry::Skill(Id::Brace), 4));
        assert_eq!(
            b.wear_on_learn(worn, Id::Brace),
            worn,
            "no empty slot: left as worn"
        );
        assert_eq!(b.wear_on_learn(worn, Id::Whirl), worn, "already worn");
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
    /// 0223 V3 → 0225 → 0272: 스킬마다 쓸 수 있는 쥔 꼴이 있다 — 무기 판마다(종류 다섯 + 방패) 고를 것이 넉넉하고(넷 이상),
    /// 첫 줄의 스킬은 판마다 둘 이상(0270 C3 — 그 판을 든 새 판의 첫 스킬이 막히지 않게). 한손검은 방패로 갈린다: 방패 판의 기술
    /// (방패와 함께만)과 한손검만의 기술(방패 없이만)이 둘 이상씩. 망치만의 것·지팡이 쪽의 것도 있다. 맨손은 무기 판의 기술을 쓰지 못한다.
    #[test]
    fn every_grip_has_skills_to_choose_and_the_first_row_fits_the_first_sword() {
        use crate::weapons::CLASSES;
        let grips: Vec<Grip> = CLASSES
            .iter()
            .map(|c| Grip::Class(c.class))
            .chain([Grip::Shield])
            .collect();
        for &g in &grips {
            let n = SKILLS.iter().filter(|s| s.fits(g)).count();
            assert!(n >= 4, "{g:?}: {n}");
        }
        for s in &SKILLS {
            assert!(s.hands != 0 && s.hands & !ANY == 0, "{:?}", s.id);
            assert!(!s.fits(Grip::Bare), "{:?} with bare hands", s.id);
        }
        // 묶음은 종류 표에서 온다: 종류마다 비트 하나(서로 다르다 · 방패의 비트와 겹치지 않는다).
        let mut all = SHIELD;
        for c in &CLASSES {
            let bit = class_bit(c.class);
            assert_eq!(bit.count_ones(), 1);
            assert_eq!(all & bit, 0, "{:?}", c.class);
            all |= bit;
        }
        assert_eq!(all, ANY, "ANY is every class and the shield");
        for &g in &grips {
            let first = SKILLS.iter().filter(|s| s.row == 0 && s.fits(g)).count();
            assert!(first >= 2, "{g:?}: {first} first-row skills");
        }
        // 한손으로는 방패가 있어야만 되는 것(방패 판의 기술)·방패가 없어야만 되는 것(한손검만의 기술).
        let split = |with: Grip, without: Grip| {
            SKILLS
                .iter()
                .filter(|s| s.fits(with) && !s.fits(without))
                .count()
        };
        let sword = Grip::Class(Class::Sword);
        assert!(split(Grip::Shield, sword) >= 2, "shield arts");
        assert!(split(sword, Grip::Shield) >= 2, "the sword's own");
        assert!(SKILLS.iter().any(|s| s.hands == HAMMER), "the hammer's own");
        assert!(SKILLS.iter().any(|s| s.hands == SHIELD));
        // 0272: 쇠뇌와 활은 서로 다른 종류다 — 저마다 제 것만의 스킬이 있다.
        assert!(
            SKILLS.iter().any(|s| s.hands == CROSSBOW),
            "the crossbow's own"
        );
        assert!(SKILLS.iter().any(|s| s.hands == BOW), "the bow's own");
        // 망치는 물리 스킬을, 지팡이는 타입 있는 스킬을 쥔다 — 지팡이로 쓰는 스킬에는 타입이 있다.
        for s in SKILLS.iter().filter(|s| s.fits(Grip::Class(Class::Staff))) {
            assert_ne!(s.kind, Kind::Physical, "{:?} in a staff's hands", s.id);
        }
        assert!(SKILLS
            .iter()
            .any(|s| s.fits(Grip::Class(Class::Hammer)) && s.kind == Kind::Physical));
    }

    /// 0223 V2: 타입 있는 스킬은 게이지를 쌓고(물리는 0), 네 타입 모두 적어도 한 스킬이 싣는다 — 어느 타입도 판에서 못 고르는 일이 없다.
    #[test]
    fn typed_skills_build_their_gauge_and_every_type_has_a_skill() {
        for s in &SKILLS {
            assert_eq!(s.kind == Kind::Physical, s.build == 0.0, "{:?}", s.id);
            assert!((0.0..=1.0).contains(&s.build), "{:?}", s.id);
        }
        for k in crate::status::TYPED {
            assert!(SKILLS.iter().any(|s| s.kind == k), "{k:?}");
        }
        assert!(LINE_DOSE.0 != Kind::Physical && LINE_DOSE.1 > 0.0);
    }

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

    /// 손댄 기록을 바로잡는다: 범위 밖의 단은 안으로, 3단이 안 된 고르기는 ㄱ, 가진 점수보다 많이 쓴 판은 처음으로.
    #[test]
    fn a_board_from_a_record_is_made_sane_against_the_points_it_has() {
        let odd = Board {
            rank: ranks([0, 9, 2, 3, 0, 0, 7]),
            pick: [Pick::B; N],
            ult: ult_ranks([0, 9]),
            cells: Tree::default(),
        };
        let b = odd.sane(40);
        assert_eq!(b.rank, ranks([0, MAX_RANK, 2, 3, 0, 0, MAX_RANK]));
        assert_eq!(
            b.pick,
            picks([
                Pick::A,
                Pick::B,
                Pick::A,
                Pick::B,
                Pick::A,
                Pick::A,
                Pick::B
            ])
        );
        assert_eq!(b.ult, ult_ranks([0, ULT_MAX_RANK]));
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
                key: None,
                links: 0,
                picks: [0; 3],
                veins: 0,
                vein_picks: 0,
            },
            ..Board::default()
        };
        assert_eq!(legacy.sane(9).cells, Tree::from_counts([2, 0, 0]));
    }

    /// 월드에 넘기는 끼운 둘: 그 스킬의 단·고르기 + 큰 기술의 단. 안 배운 스킬이 놓인 칸은 단 0 — 빈 칸이다(0166).
    #[test]
    fn the_loadout_carries_the_worn_ranks_and_an_unlearned_slot_is_empty() {
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
        let l = b.loadout([Id::Gather, Id::Whirl], Ult::Upend);
        assert_eq!(
            (l.slots[0].id, l.slots[0].rank),
            (Id::Gather, 0),
            "unlearned: an empty slot"
        );
        assert_eq!(l.slots[1].rank, 3);
        let plain: Loadout = [Id::Cut, Id::Gather].into();
        assert!(plain.slots.iter().all(|s| s.rank == 1) && plain.ult == 1);
    }

    #[test]
    fn two_slots_never_hold_the_same_skill() {
        assert_eq!(sane([Id::Cut, Id::Quake]), [Id::Cut, Id::Quake]);
        assert_eq!(sane([Id::Cut, Id::Cut]), [Id::Cut, Id::Whirl]);
        assert_eq!(sane([Id::Whirl, Id::Whirl]), [Id::Whirl, Id::Cut]);
    }

    /// 0158 G2 → 0172: 큰 기술은 셋 — 셋 다 큰 기술 줄(쓴 점수 8)이 열리면 **2점**으로 배우고 2·3단은 1점씩(쟁반 뒤집기도 —
    /// 처음부터 주는 것은 없다). 점수가 1뿐이면 배우지 못한다. 처음 배운 것이 끼울 것이고, 되돌리면 하나도 없다.
    #[test]
    fn the_great_skills_are_learned_for_two_points_once_eight_are_spent() {
        // 0270 C3: 무기 판마다 하나 — 다섯(큰 종·눈보라가 뒤에 붙었다).
        assert_eq!(
            ULTS,
            [
                Ult::Upend,
                Ult::Flurry,
                Ult::Line,
                Ult::Bell,
                Ult::Blizzard,
                Ult::Rain,
                Ult::Gale,
                Ult::Ambush,
                Ult::Tempest
            ]
        );
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
        let upend = Entry::Ult(Ult::Upend);
        assert_eq!((upend.free(), upend.price(0), upend.price(1)), (0, 2, 1));
        assert_eq!((flurry.paid(1), flurry.paid(3)), (2, 4));
        assert_eq!(upend.paid(3), 4, "0172: the same as the others");
        let mut b = Board::default();
        assert_eq!(b.offer(flurry, 23), Offer::Locked(8));
        // 몰아치기까지의 길(0215): 휘돌이 → 고른 호흡 → 땅울림 → 맺음타.
        for e in [
            Entry::Skill(Id::Whirl),
            Entry::Skill(Id::Whirl),
            Entry::Skill(Id::Whirl),
            Entry::Skill(Id::Whirl),
            Entry::Cell(Cell::EvenBreath),
            Entry::Skill(Id::Quake),
            Entry::Cell(Cell::ClosingBlow),
            Entry::Skill(Id::Cut),
        ] {
            assert!(b.buy(e, 23), "{e:?}");
        }
        assert_eq!(b.spent(), 8);
        assert!(!b.has_ult(Ult::Flurry));
        // 아직 하나도 안 배웠다: 끼울 것이 없다(끼운 자리의 이름은 그대로 — 그 단이 0이다).
        assert_eq!((b.ults_known(), b.wearable_ult(Ult::Line)), (0, Ult::Line));
        assert_eq!(b.loadout(START, Ult::Line).ult, 0);
        assert_eq!(b.offer(flurry, 9), Offer::Short, "one point is not enough");
        assert!(!b.buy(flurry, 9));
        assert_eq!(b.offer(flurry, 10), Offer::Ready);
        assert!(b.buy(flurry, 10));
        assert!(b.has_ult(Ult::Flurry));
        // 배운 것이 하나면 그것이 끼울 것이다 — 끼운 자리가 무엇을 가리키든.
        assert_eq!(b.wearable_ult(Ult::Upend), Ult::Flurry);
        assert_eq!(b.loadout(START, Ult::Upend).great, Ult::Flurry);
        assert_eq!((b.spent(), b.left(10)), (10, 0));
        assert!(b.buy(flurry, 12) && b.buy(flurry, 12));
        assert_eq!(b.offer(flurry, 99), Offer::Full);
        assert_eq!(b.spent(), 12);
        // 줄 긋기는 무기 판의 칸이라(0270 C3) 몸의 판의 길을 묻지 않는다 — 줄만 열리면.
        assert_eq!(b.offer(line, 99), Offer::Ready);
        assert!(b.buy(Entry::Cell(Cell::FlowingBeat), 99) && b.buy(Entry::Cell(Cell::GapBeat), 99));
        assert!(b.buy(line, 16));
        assert_eq!((b.ult, b.ults_known()), (ult_ranks([0, 3, 1]), 2));
        // 쟁반 뒤집기는 자리의 길 끝(모으기 → 멈추지 않는 걸음 → 던지기 → 기우는 땅).
        for e in [
            Entry::Skill(Id::Gather),
            Entry::Cell(Cell::UnbrokenStride),
            Entry::Skill(Id::Throw),
            Entry::Cell(Cell::TiltingGround),
        ] {
            assert!(b.buy(e, 99), "{e:?}");
        }
        assert!(b.buy(upend, 22));
        assert_eq!((b.ult, b.spent()), (ult_ranks([1, 3, 1]), 22));
        // 옛 기록의 꼴: 쟁반 뒤집기를 지닌 판(이미 올린 단은 그대로).
        assert_eq!(Board::default().with_upend().ult, ult_ranks([1]));
        assert_eq!(b.with_upend(), b);
        // 되돌리면 배운 큰 기술이 모두 돌아간다 — 끼울 것이 없다(자리의 이름은 그대로).
        let mut great = Ult::Line;
        b.respec(&mut great);
        assert_eq!((b.ult, great), (ult_ranks([]), Ult::Line));
        assert_eq!(b.loadout(START, great).ult, 0);
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

#[cfg(test)]
mod clip_facing {
    use crate::model::Model;
    use macroquad::prelude::*;

    /// 0193: 기술 동작의 몸 방향(`wanderer_skin.py` `SKILL_FACE`)과 같은 표 — 판정의 순간에 무엇이 정면이어야 하나.
    enum Face {
        Tip,
        Hips,
        Hold,
        Spin,
        /// 활을 겨누는 동작(0272 — 화살비): 겨누는 줄(오른손 → 활을 든 왼손)이 정면 — 몸은 옆으로 선다.
        Aim,
    }
    const FACES: [(&str, Face); 39] = [
        ("sk_whirl", Face::Spin),
        ("sk_cut", Face::Tip),
        ("sk_gather", Face::Hips),
        ("sk_quake", Face::Tip),
        ("sk_brace", Face::Hold),
        ("sk_echo", Face::Hips),
        ("sk_throw", Face::Tip),
        ("sk_embers", Face::Hips),
        ("ult_upend", Face::Hips),
        ("ult_flurry", Face::Hips),
        ("ult_line", Face::Tip),
        ("sk_swallow", Face::Tip),
        ("sk_bash", Face::Hips),
        ("sk_roots", Face::Tip),
        ("sk_stitch", Face::Tip),
        ("sk_ram", Face::Hold),
        ("sk_toll", Face::Tip),
        ("sk_swat", Face::Tip),
        ("sk_ripple", Face::Hips),
        ("sk_snow", Face::Hips),
        ("sk_flare", Face::Hips),
        ("sk_snare", Face::Hips),
        ("ult_bell", Face::Hips),
        ("ult_blizzard", Face::Hips),
        // 0272: 쇠뇌 판의 둘 · 활 판의 큰 기술(겨누는 줄 — 활을 든 왼손이 정면).
        ("sk_fuse", Face::Hips),
        ("sk_mine", Face::Hips),
        ("ult_rain", Face::Aim),
        // 0272 2단계: 창 판의 셋 · 질풍 찌르기(세계가 멈춘 동안 당긴 채 — 몸이 정면).
        ("sk_skewer", Face::Tip),
        ("sk_pin", Face::Tip),
        ("sk_pole", Face::Tip),
        ("ult_gale", Face::Hips),
        // 0274 한손검.
        ("sk_pommel", Face::Tip),
        ("sk_shove", Face::Hips),
        ("sk_rally", Face::Hold),
        // 0274 ③ 쌍단검.
        ("sk_slip", Face::Tip),
        ("ult_ambush", Face::Hips),
        // 0274 ④ 대검.
        ("sk_sunder", Face::Tip),
        ("sk_sear", Face::Tip),
        ("ult_tempest", Face::Spin),
    ];

    fn deg(a: f32) -> f32 {
        (a.to_degrees() + 540.0).rem_euclid(360.0) - 180.0
    }

    /// 기술을 누르는 순간·끝나 서기로 돌아갈 때 몸이 홱 돌지 않고(정면 ±25°), 판정의 순간에 칼끝·몸이 정면이며,
    /// 휘돌이는 이어짐 구간(바퀴마다 되풀이되는 몫)에 이펙트와 같은 쪽으로 한 바퀴 돈다.
    #[test]
    fn skill_clips_start_and_end_facing_front_and_hit_in_front() {
        let m = Model::from_bytes(include_bytes!("../assets/models/wanderer.glb")).unwrap();
        let (hips, sword, grip) = (
            m.node("mixamorig:Hips").unwrap(),
            m.node("sword").unwrap(),
            m.node("weapon").unwrap(),
        );
        let mut rest = Vec::new();
        m.pose(None, 0.0, &mut rest);
        let g = rest[grip].w_axis.truncate();
        let far = m.meshes[m.nodes[sword].mesh.unwrap()]
            .pos
            .iter()
            .copied()
            .max_by(|a, b| {
                let d = |p: Vec3| rest[sword].transform_point3(p).distance(g);
                d(*a).total_cmp(&d(*b))
            })
            .unwrap();
        let clips = super::SKILLS
            .iter()
            .map(|s| s.clip)
            .chain(super::ULTS.map(|u| u.clip()));
        let mut pose = Vec::new();
        let mut aim = Vec::new();
        let mut body = |ai, t: f32| {
            m.pose(Some(ai), t, &mut pose);
            let z = pose[hips].z_axis.truncate();
            let p = pose[sword].transform_point3(far);
            (z.x.atan2(z.z), p.x.atan2(p.z))
        };
        for clip in clips {
            let [s, a, r] = clip.seg;
            let ai = m.anim(clip.name).unwrap_or_else(|| panic!("{}", clip.name));
            let face = &FACES.iter().find(|(n, _)| *n == clip.name).unwrap().1;
            for t in [0.0, s + a + r - 0.005] {
                let y = deg(body(ai, t).0);
                assert!(
                    y.abs() < 25.0,
                    "{}: the body faces front at {t:.2}s ({y:.0}°)",
                    clip.name
                );
            }
            let (h, tip) = body(ai, s);
            match face {
                Face::Tip => assert!(
                    deg(tip).abs() < 25.0,
                    "{}: the blade is in front at the hit ({:.0}°)",
                    clip.name,
                    deg(tip)
                ),
                Face::Hips => assert!(
                    deg(h).abs() < 25.0,
                    "{}: the body faces front at the hit ({:.0}°)",
                    clip.name,
                    deg(h)
                ),
                Face::Aim => {
                    let (rh, lh) = (
                        m.node("mixamorig:RightHand").unwrap(),
                        m.node("mixamorig:LeftHand").unwrap(),
                    );
                    for t in [s, s + a - 0.005] {
                        m.pose(Some(ai), t, &mut aim);
                        let d = aim[lh].w_axis.truncate() - aim[rh].w_axis.truncate();
                        let y = deg(d.x.atan2(d.z));
                        assert!(
                            y.abs() < 25.0,
                            "{}: the bow points ahead from the draw to the loose ({y:.0}°)",
                            clip.name
                        );
                    }
                }
                Face::Hold => {
                    for t in [s, s + a - 0.005] {
                        let y = deg(body(ai, t).0);
                        assert!(
                            y.abs() < 25.0,
                            "{}: the body faces front while holding ({y:.0}°)",
                            clip.name
                        );
                    }
                }
                Face::Spin => {
                    let mut turned = 0.0;
                    let mut last = body(ai, s).0;
                    for j in 1..=20 {
                        let y = body(
                            ai,
                            s + a * j as f32 / 20.0 - if j == 20 { 0.005 } else { 0.0 },
                        )
                        .0;
                        turned += deg(y - last);
                        last = y;
                    }
                    let want = super::WHIRL_CLIP_TURN * 360.0;
                    assert!(
                        (turned - want).abs() < 30.0,
                        "{}: one turn per beat, the way the effect turns ({turned:.0}°)",
                        clip.name
                    );
                }
            }
        }
    }
}
