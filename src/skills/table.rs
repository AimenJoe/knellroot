//! 스킬 표: `Id` · `Skill` · `SKILLS`(줄 = 대기 · 시동 · 후딜 · 모양 · 피해).
use super::*;
use crate::progression::Branch;
use crate::status::{Dose, Kind};
use crate::tuning::HP_UNIT;
use crate::weapons::Grip;

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
    /// 걸어 당기기(0274 ⑤ — 낫·뿌리): 휜 날을 크게 걸어 베며 앞의 적을 몸 앞으로 끌어온다(BG3 드루이드의 덩굴 휘감기 결).
    Reap,
    /// 생기 거두기(0274 ⑤ — 낫·흡수): 몸 둘레를 한 바퀴 거두며 벤 적마다 체력을 되찾는다.
    Harvest,
    /// 저주 새기기(0274 ⑥ — 완드·균열): 한 적에게 저주를 새긴다 — 금이 가고, 그 적이 쓰러지면 가까운 적에게 옮는다(BG3 흑마법사의 Hex 결).
    Hex,
    /// 시드는 그늘(0274 ⑥ — 완드·균열): 겨눈 쪽 앞에 그늘을 드리운다 — 그 안의 적을 거듭 치며 금을 쌓는다.
    Blight,
    /// 찌르고 물러서기(0280 — 창·날카로움): 길고 좁게 한 번 찌르고 창끝을 겨눈 채 물러선다(창의 닿는 거리를 지킨다).
    Withdraw,
    /// 불씨 내리꽂기(0280 — 대검·잔불): 달군 날을 땅에 내리꽂아 둘레로 불길을 번지게 한다(떼 한가운데의 분노).
    Plunge,
    /// 돌아오는 낫(0280 — 낫·참격): 낫을 돌려 던진다 — 넓게 돌며 베고 손으로 돌아온다(끌어오는 날을 고를 수 있다).
    Boomerang,
    /// 끊는 말(0280 — 완드·마법): 뜻 모를 말 한마디로 앞의 준비하던 공격을 끊는다.
    Word,
    /// 손바닥 치기(0286 — 몽크·충격): 손바닥으로 짧게 밀쳐 쳐 준비하던 공격을 끊고 기절을 쌓는다.
    Palm,
    /// 할퀴기(0286 — 몽크·날카로움): 앞을 다섯 번 빠르게 할퀸다 — 손톱이 출혈을 쌓는다.
    Rake,
    /// 날아 차기(0286 — 몽크·충격): 가까운 적에게서 적으로 두 번 날아들며 찬다.
    Kick,
    /// 쓸어 차기(0286 — 몽크·타격): 몸을 낮춰 둘레를 한 바퀴 쓸어 차 밀어낸다(에워싸였을 때의 답).
    Sweep,
}

/// 표 순서(대기 칸 `Player::skill_cd`의 자리 — 뒤에만 붙인다).
pub const IDS: [Id; 42] = [
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
    Id::Reap,
    Id::Harvest,
    Id::Hex,
    Id::Blight,
    Id::Withdraw,
    Id::Plunge,
    Id::Boomerang,
    Id::Word,
    Id::Palm,
    Id::Rake,
    Id::Kick,
    Id::Sweep,
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

impl Skill {
    /// 그 갈래의 무기로 쓸 수 있나(0223 V3).
    pub fn fits(&self, g: Grip) -> bool {
        self.hands & grip_bit(g) != 0
    }

    /// 이 스킬의 타 하나가 싣는 타입과 쌓는 양(0223 V2).
    pub fn dose(&self) -> Dose {
        (self.kind, self.build)
    }
}

impl Skill {
    /// 판정(움직임)이 이어지는 시간(초).
    pub fn active(&self) -> f32 {
        match self.form {
            Form::Ring { hits, gap, .. } | Form::Stab { hits, gap, .. } => {
                gap * hits.saturating_sub(1) as f32
            }
            Form::Lane { time, .. }
            | Form::Wave { time, .. }
            | Form::Slip { time, .. }
            | Form::Reap { time, .. } => time,
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
            | Form::Hex { .. }
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
        // 0274 ④: 대검도 휘돈다(광전사의 휩쓸기 — 대검 판의 가운데 열). 0274 ⑤: 낫은 한손이지만 휘돌지 않는다(드루이드의 판에 서지
        // 않게 묶음에서 뺀다 — 낫의 둘레 베기는 생기 거두기). 0281: 완드도 한손이 되었지만 휘돌지 않는다(그 전처럼 완드 판에 서지 않게).
        hands: (ONE & !SCYTHE & !WAND) | GREATSWORD,
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
        // 0274 ⑥: 완드(흑마법사)는 버티지 않는다 — 떼 한가운데서 맞으며 버티는 결이 아니다(완드 판에 서지 않게 묶음에서 뺀다).
        hands: MELEE & !DAGGER & !WAND,
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
        // 0274 ⑥: 완드(흑마법사)도 — 받아친 손이 둘레를 울린다(완드 판의 울림 계열 둘째 줄).
        hands: SHIELD | SWORD | STAFF | RANGED | WAND,
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
        // 0274 ⑤: 낫(드루이드)도 — 땅을 다스리는 손(낫 판의 뿌리 계열 둘째 줄).
        hands: STAFF | BOW | SPEAR | SCYTHE,
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
        // 0274 ⑥: 완드(흑마법사)도 — 한 적에게 울림을 걸어 곧 터뜨린다(완드 판의 울림 계열 첫 줄 — 죽음의 울림을 부른다).
        hands: SHIELD | HAMMER | WAND,
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
    // ---- 0274 ⑤: 낫 = 드루이드(뿌리 · 흡수) ----
    // 걸어 당기기: 휜 날을 앞으로 크게 걸어(3.2 m · 150°) 베며 맞은 적을 몸 앞으로 끌어온다(부채꼴의 `push`가 음수 = 끌기 — 몸 앞
    // `weapons::PULL_STOP`에서 멈춘다, 문지기는 끌리지 않는다) · 뿌리를 반 남짓 쌓는다. 넓게 걸기(ㄱ 240°) · 엉킨 땅(ㄴ — 걸어 당긴
    // 부채꼴이 2.5초 남아 그 안의 적이 반 빠르기로 걷는다: 땅울림 ㄴ과 같은 꼴) · 최종 효과 = 끌려온 적이 2초 동안 피해를 더 받는다.
    Skill {
        id: Id::Reap,
        cooldown: 7.0,
        startup: 0.18,
        recover: 0.30,
        damage: HP_UNIT,
        poise: 1.5,
        form: Form::Cone {
            range: 3.2,
            arc_deg: 150.0,
            stagger: 0.4,
            push: -1.6,
        },
        anim: "scythe3",
        clip: Clip {
            name: "sk_reap",
            seg: [0.18, 0.0, 0.30],
        },
        row: 0,
        hands: SCYTHE,
        kind: Kind::Root,
        build: 0.6,
        branch: Branch::Footing,
        picks: [
            Twist::Wide { arc_deg: 240.0 },
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
    // 생기 거두기: 휜 날을 몸 둘레로 한 바퀴 둘러(2.6 m) 거둔다 — 거둔 적마다 체력 4(넷까지 — 많아야 16), 흡수를 가득 걸어 그 뒤의 타도
    // 되돌린다. 넓은 낫(ㄱ — 둘레 ×1.3) · 바닥에서(ㄴ — 체력 35% 아래면 두 배: 재기의 ㄴ과 같은 꼴) · 최종 효과 = 거둔 적이 0.6초 휘청인다.
    Skill {
        id: Id::Harvest,
        cooldown: 11.0,
        startup: 0.15,
        recover: 0.30,
        damage: HP_UNIT,
        poise: 1.0,
        form: Form::Reap {
            radius: 2.6,
            heal: 4,
            most: 4,
            time: 0.22,
        },
        anim: "sk_whirl",
        clip: Clip {
            name: "sk_harvest",
            seg: [0.15, 0.22, 0.30],
        },
        row: 1,
        hands: SCYTHE,
        kind: Kind::Drain,
        build: 1.0,
        branch: Branch::Riposte,
        picks: [
            Twist::Big { mult: 1.3 },
            Twist::Low {
                below: 0.35,
                mult: 2.0,
            },
        ],
        cap: Cap::Stun { stagger: 0.6 },
    },
    // ---- 0274 ⑥: 완드 = 흑마법사(균열 · 울림) ----
    // 저주 새기기: 앞 8 m 안의 한 적에게 저주를 새긴다 — 피해 10 · 균열 가득(보통 적은 곧바로 금이 간다), 표는 6초 남고 그 적이 표를 지닌 채
    // 쓰러지면 둘레 4 m의 가장 가까운 적에게 옮는다(남은 초 그대로 · 균열 다시 가득). 멀리 옮는 저주(ㄱ — 옮는 둘레 ×1.5) · 두 갈래(ㄴ —
    // 가까운 둘에게 옮는다: 종 치기의 두 번 울림과 같은 칸) · 최종 효과 = 옮을 때 쓰러진 자리가 울린다(둘레 피해 10 · 울림 — 되울림의 맺음과 같은 칸).
    Skill {
        id: Id::Hex,
        cooldown: 9.0,
        startup: 0.22,
        recover: 0.30,
        damage: HP_UNIT,
        poise: 0.5,
        form: Form::Hex {
            range: 8.0,
            secs: 6.0,
            spread: 4.0,
        },
        anim: "cast",
        clip: Clip {
            name: "sk_hex",
            seg: [0.22, 0.0, 0.30],
        },
        row: 0,
        hands: WAND,
        kind: Kind::Rift,
        build: 1.0,
        branch: Branch::Riposte,
        picks: [Twist::Big { mult: 1.5 }, Twist::Twice],
        cap: Cap::Toll { damage: HP_UNIT },
    },
    // 시드는 그늘: 겨눈 쪽 4.5 m 앞에 둘레 2.6 m의 그늘을 4초 드리운다 — 1초마다 그 안의 적을 치고(10) 균열을 쌓는다(0.35 — 셋이면 금).
    // 흐르는 그늘(ㄱ — 겨눈 쪽으로 흘러간다) · 넓은 그늘(ㄴ — 둘레 ×1.4) · 최종 효과 = 1초 더 남고 타마다 자세 피해 1(눈발과 같은 칸들).
    Skill {
        id: Id::Blight,
        cooldown: 10.0,
        startup: 0.25,
        recover: 0.30,
        damage: HP_UNIT,
        poise: 0.0,
        form: Form::Drift {
            ahead: 4.5,
            radius: 2.6,
            secs: 4.0,
            every: 1.0,
        },
        anim: "cast",
        clip: Clip {
            name: "sk_blight",
            seg: [0.25, 0.0, 0.30],
        },
        row: 1,
        hands: WAND,
        kind: Kind::Rift,
        build: 0.35,
        branch: Branch::Footing,
        picks: [Twist::Flow { speed: 1.5 }, Twist::Big { mult: 1.4 }],
        cap: Cap::Smolder {
            secs: 1.0,
            poise: 1.0,
        },
    },
    // ---- 0280: 빈 노드 넷(창 · 대검 · 낫 · 완드 트리의 둘째 줄) — 있는 모양에 새 수. 동작은 0283부터 제 것(Meshy API로 받음 —
    // 예전엔 같은 구간 길이의 있는 전용 동작을 빌렸다, `anim`은 모델에 없을 때 넘어가는 것) ----
    // 찌르고 물러서기(창 · 날카로움 — 오른쪽 둘째 줄): 앞을 길고 좁게(3.4 m · 40°) 한 번 찔러 출혈을 한 번에 쌓고, 창끝을 겨눈 채 0.22초에
    // 3 m 물러선다(빠지며 베기의 꼴 `Form::Slip` — 닿는 거리를 지킨다). 무적은 없다. 동작 `sk_withdraw`(0283 — 두 손으로 낮게 내질러 뻗은 채 머문다).
    // 한 걸음 더(ㄱ 4.5 m) · 벌어진 상처(ㄴ — 찔린 적이 2초 피해 +25%, 문지기는 아니다) · 최종 효과 = 쓰러뜨리면 쿨타임의 반.
    Skill {
        id: Id::Withdraw,
        cooldown: 8.0,
        startup: 0.14,
        recover: 0.30,
        damage: 2 * HP_UNIT,
        poise: 1.5,
        form: Form::Slip {
            range: 3.4,
            arc_deg: 40.0,
            back: 3.0,
            time: 0.22,
        },
        anim: "spear3",
        clip: Clip {
            name: "sk_withdraw",
            seg: [0.14, 0.22, 0.30],
        },
        row: 1,
        hands: SPEAR,
        kind: Kind::Sharp,
        build: 1.0,
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
    // 불씨 내리꽂기(대검 · 잔불 — 오른쪽 둘째 줄): 달군 날을 두 손으로 땅에 내리꽂아 둘레(5 m)로 0.3초에 번지는 불길(파문의 꼴
    // `Form::Wave`) — 지나는 적마다 20 · 잔불 가득. 밀어내지 않는다(분노는 둘레의 떼로 찬다). 동작 `sk_plunge`(0283 — 두 손으로 들어 내리꽂고 꽂은 채 웅크린다).
    // 넓은 불길(ㄱ — 둘레 ×1.3) · 타는 땅(ㄴ — 내리꽂은 둘레에 불티 3초) · 최종 효과 = 불길이 0.5초 휘청(문지기는 아니다).
    Skill {
        id: Id::Plunge,
        cooldown: 10.0,
        startup: 0.25,
        recover: 0.35,
        damage: 2 * HP_UNIT,
        poise: 2.0,
        form: Form::Wave {
            radius: 5.0,
            time: 0.30,
        },
        anim: "gs5",
        clip: Clip {
            name: "sk_plunge",
            seg: [0.25, 0.30, 0.35],
        },
        row: 1,
        hands: GREATSWORD,
        kind: Kind::Ember,
        build: 1.0,
        branch: Branch::Footing,
        picks: [Twist::Big { mult: 1.3 }, Twist::Scorch { secs: 3.0 }],
        cap: Cap::Lift { stagger: 0.5 },
    },
    // 돌아오는 낫(낫 · 참격 — 가운데 둘째 줄): 휜 날을 돌려 던진다(던지기의 꼴 `Form::Throw`) — 7 m를 넓게(반 폭 1.3 m — 던지기 0.9) 조금
    // 느리게(초속 18 m — 22) 돌며 가는 길 · 오는 길에 적마다 10. 동작 `sk_boomerang`(0283 — 옆으로 휘둘러 던지고 받으려 손을 든다). 끌어오는 날(ㄱ — 돌아오는 길에 맞은 적을
    // 2.5 m 끌어온다, 문지기는 아니다: 드루이드의 끌기) · 넓게 도는 날(ㄴ — 반 폭 ×1.5) · 최종 효과 = 돌아오는 길 피해 ×1.5 · 자세 ×2.
    Skill {
        id: Id::Boomerang,
        cooldown: 8.0,
        startup: 0.18,
        recover: 0.25,
        damage: HP_UNIT,
        poise: 1.5,
        form: Form::Throw {
            dist: 7.0,
            half: 1.3,
            speed: 18.0,
            hand: 0.10,
        },
        anim: "throw",
        clip: Clip {
            name: "sk_boomerang",
            seg: [0.18, 0.10, 0.25],
        },
        row: 1,
        hands: SCYTHE,
        kind: Kind::Physical,
        build: 0.0,
        branch: Branch::Footing,
        picks: [Twist::Drag { dist: 2.5 }, Twist::Big { mult: 1.5 }],
        cap: Cap::Homing {
            damage: 1.5,
            poise: 2.0,
        },
    },
    // 끊는 말(완드 · 마법 — 가운데 둘째 줄): 뜻 모를 말 한마디를 앞(4 m · 70°)에 던진다(막아치기의 꼴 `Form::Bash`) — 공격을 준비하던 잡몹은
    // 끊기고 자세 피해 ×2, 닿은 적은 밀려난다. 보스는 끊기지 않고 자세만(보스 규칙 그대로). 동작 `sk_word`(0283 — 한 팔을 곧게 뻗어 가리킨다).
    // 넓게 퍼지는 말(ㄱ 120°) · 갈라진 틈(ㄴ — 끊긴 적 2초 피해 +20%) · 최종 효과 = 끊긴 적은 자세가 곧바로 무너진다.
    Skill {
        id: Id::Word,
        cooldown: 9.0,
        startup: 0.20,
        recover: 0.30,
        damage: HP_UNIT,
        poise: 2.0,
        form: Form::Bash {
            range: 4.0,
            arc_deg: 70.0,
            stagger: 0.5,
            mult: 2.0,
        },
        anim: "cast",
        clip: Clip {
            name: "sk_word",
            seg: [0.20, 0.0, 0.30],
        },
        row: 1,
        hands: WAND,
        kind: Kind::Physical,
        build: 0.0,
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
    // ---- 0286: 몽크(맨손 — 충격 · 날카로움). 있는 모양에 새 수. 동작은 0286 처음에는 같은 구간 길이의 있는 전용 동작을 빌린다(시동 ·
    // 이어짐 · 후딜이 그 동작과 같다) ----
    // 손바닥 치기(충격 첫 줄): 손바닥으로 짧고 조금 넓게(2.0 m · 80°) 밀쳐 친다 — 막아치기의 꼴(`Form::Bash`): 준비하던 잡몹의 공격을 끊고
    // 자세 피해 ×2.5 · 충격을 한 번에 쌓는다(기절). 보스는 끊기지 않고 자세만(보스 규칙 그대로). 자루 치기(한손검)보다 짧게 닿고
    // 자세를 더 깎는다. 넓은 손(ㄱ 120°) · 드러난 틈(ㄴ — 끊긴 적 2초 피해 +25%) · 최종 효과 = 끊긴 적은 자세가 곧바로 무너진다.
    Skill {
        id: Id::Palm,
        cooldown: 7.0,
        startup: 0.12,
        recover: 0.28,
        damage: HP_UNIT,
        poise: 3.0,
        form: Form::Bash {
            range: 2.0,
            arc_deg: 80.0,
            stagger: 0.7,
            mult: 2.5,
        },
        anim: "fist3",
        clip: Clip {
            name: "sk_pommel",
            seg: [0.12, 0.0, 0.28],
        },
        row: 0,
        hands: MONK,
        kind: Kind::Shock,
        build: 1.0,
        branch: Branch::Riposte,
        picks: [
            Twist::Wide { arc_deg: 120.0 },
            Twist::Expose {
                secs: 2.0,
                mult: 1.25,
            },
        ],
        cap: Cap::Break,
    },
    // 할퀴기(날카로움 첫 줄): 앞(1.9 m · 80°)을 0.05초마다 다섯 번 할퀸다(바늘땀의 꼴 `Form::Stab` — 좁고 깊게 찌르는 바늘땀보다 짧고 넓고
    // 잦다) — 타마다 10 · 출혈 0.3(다섯이 다 닿으면 보통 적은 피를 흘린다 — 날카로움에 무른 정도 0.75인 잔재도), 쿨타임 8초. 파고드는 손(ㄱ — 할퀴는 동안 1.5 m 내딛는다) · 더 할퀸다
    // (ㄴ — 둘 더) · 최종 효과 = 마지막 할퀴기에 맞은 적이 0.5초 휘청.
    Skill {
        id: Id::Rake,
        cooldown: 8.0,
        startup: 0.10,
        recover: 0.25,
        damage: HP_UNIT,
        poise: 0.5,
        form: Form::Stab {
            range: 1.9,
            arc_deg: 80.0,
            hits: 5,
            gap: 0.05,
        },
        anim: "fist1",
        clip: Clip {
            name: "sk_stitch",
            seg: [0.10, 0.20, 0.25],
        },
        row: 0,
        hands: MONK,
        kind: Kind::Sharp,
        build: 0.3,
        branch: Branch::Beat,
        picks: [Twist::Lunge { dist: 1.5 }, Twist::Stitches { more: 2 }],
        cap: Cap::Pin { stagger: 0.5 },
    },
    // 날아 차기(충격 둘째 줄): 6 m 안의 적에게서 적으로 세 번 날아들며 찬다(제비걸음의 꼴 `Form::Hop` — 베는 날이 아니라 발이라 자세를
    // 더 깎고 충격을 쌓는다: 10 · 자세 2.0 · 충격 0.4, 닿는 둘레 1.8 m). 한 적만(ㄱ) · 돌아오는 걸음(ㄴ) · 최종 효과 = 마지막 착지가
    // 둘레 2.5 m를 흔든다(자세 피해 ×2).
    Skill {
        id: Id::Kick,
        cooldown: 9.0,
        startup: 0.12,
        recover: 0.25,
        damage: HP_UNIT,
        poise: 2.0,
        form: Form::Hop {
            range: 6.0,
            hops: 3,
            gap: 0.15,
            reach: 1.8,
        },
        anim: "fist_dash",
        clip: Clip {
            name: "sk_swallow",
            seg: [0.12, 0.45, 0.25],
        },
        row: 1,
        hands: MONK,
        kind: Kind::Shock,
        build: 0.4,
        branch: Branch::Beat,
        picks: [Twist::Focus, Twist::HomeStep],
        cap: Cap::Finale {
            reach: 2.5,
            poise: 2.0,
        },
    },
    // 쓸어 차기(가운데 둘째 줄 — 타격): 몸을 낮춰 둘레(2.6 m)를 두 바퀴 쓸어 차고 마지막 바퀴에 1.2 m 밀어낸다(휘돌이의 꼴 `Form::Ring` —
    // 휘돌이보다 가깝고 쿨타임이 길다: 처음 안은 장대 휩쓸기처럼 한 바퀴였는데 보스 모의의 잡몹 스물에서 몽크의 둘레 답이 모자라 두 바퀴로,
    // 0286 "모의"). 몽크의 둘레 답(맨손은 한 적씩 친다). 걸으며 찬다(ㄱ) · 한 바퀴 더(ㄴ) · 최종 효과 = 마지막 바퀴에 쓸린 적이 0.8초 휘청.
    Skill {
        id: Id::Sweep,
        cooldown: 9.0,
        startup: 0.15,
        recover: 0.30,
        damage: HP_UNIT,
        poise: 2.0,
        form: Form::Ring {
            radius: 2.6,
            hits: 2,
            gap: 0.22,
            push: 1.2,
        },
        anim: "fist_heavy",
        clip: Clip {
            name: "sk_whirl",
            seg: [0.15, 0.22, 0.30],
        },
        row: 1,
        hands: MONK,
        kind: Kind::Physical,
        build: 0.0,
        branch: Branch::Footing,
        picks: [Twist::Walk { speed: 0.5 }, Twist::Spins { more: 1 }],
        cap: Cap::Lift { stagger: 0.8 },
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
            Id::Reap => "reap",
            Id::Harvest => "harvest",
            Id::Hex => "hex",
            Id::Blight => "blight",
            Id::Withdraw => "withdraw",
            Id::Plunge => "plunge",
            Id::Boomerang => "boomerang",
            Id::Word => "word",
            Id::Palm => "palm",
            Id::Rake => "rake",
            Id::Kick => "kick",
            Id::Sweep => "sweep",
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::status::Kind;
    use crate::tuning::HP_UNIT;
    use crate::weapons::{Class, Grip};

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
        // 0280: 전용 동작은 빌릴 수 있다 — 같은 동작을 쓰는 스킬은 구간의 길이도 같아야 한다(경계 = 판정의 순간이 그대로 맞게).
        // 0280의 넷이 빌렸던 것은 0283에 제 동작(`sk_withdraw` · `sk_plunge` · `sk_boomerang` · `sk_word`)으로 옮겼다 — 지금은 빌린 것이 없다.
        let clips: Vec<Clip> = IDS.iter().map(|i| i.def().clip).collect();
        let mut shared = 0;
        for (i, c) in clips.iter().enumerate() {
            for o in &clips[i + 1..] {
                if o.name == c.name {
                    assert_eq!(o.seg, c.seg, "{} is shared with other segments", c.name);
                    shared += 1;
                }
            }
        }
        // 0286: 몽크의 넷이 같은 구간의 있는 동작을 빌린다(손바닥 치기 = 자루 치기 · 할퀴기 = 바늘땀 · 날아 차기 = 제비걸음 · 쓸어 차기 =
        // 장대 휩쓸기).
        assert_eq!(
            shared, 4,
            "the monk's four borrow until their own clips come"
        );
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

    /// 0158 마: 고르기와 맺음의 표 — 스킬마다 둘(서로 다르다)과 하나, 값은 결정문의 것. 길이가 달라지는 고르기를 골라도 1초 안에 끝난다.
    /// 0223 V3 → 0225 → 0272: 스킬마다 쓸 수 있는 쥔 꼴이 있다 — 무기 판마다(종류 다섯 + 방패) 고를 것이 넉넉하고(넷 이상),
    /// 첫 줄의 스킬은 판마다 둘 이상(0270 C3 — 그 판을 든 새 판의 첫 스킬이 막히지 않게). 한손검 판과 방패 판에는 서로에게 없는 기술이
    /// 둘 이상씩(0281부터 한손검과 방패는 둘 다 쓴다). 망치만의 것·지팡이 쪽의 것도 있다. 맨손은 무기 판의 기술을 쓰지 못한다.
    #[test]
    fn every_grip_has_skills_to_choose_and_the_first_row_fits_the_first_sword() {
        use crate::weapons::CLASSES;
        let grips: Vec<Grip> = CLASSES
            .iter()
            .map(|c| Grip::Class(c.class))
            .chain([Grip::Shield(None)])
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
        for c in CLASSES {
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
        // 한손으로는 방패가 있어야만 되는 것(방패 판의 기술)·한손검만의 기술. 0281: 한손검과 방패는 둘 다 쓴다(멀티클래스).
        let split = |with: Grip, without: Grip| {
            SKILLS
                .iter()
                .filter(|s| s.fits(with) && !s.fits(without))
                .count()
        };
        let sword = Grip::Class(Class::Sword);
        let shield = Grip::Shield(None);
        let both = Grip::Shield(Some(Class::Sword));
        assert!(split(shield, sword) >= 2, "shield arts");
        assert!(split(sword, shield) >= 2, "the sword's own");
        assert_eq!(split(sword, both) + split(shield, both), 0, "both trees");
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
}
