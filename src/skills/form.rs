//! 스킬의 모양 `Form`과 그 성질 `FormInfo`(0278 R16) · 뿌리 솟기의 수.

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
    /// 생기 거두기(0274 ⑤ — 낫·흡수): 휜 날을 몸 둘레로 한 바퀴 둘러(`time`초 — 끝에 벤다) 둘레 `radius` m의 적을 모두 베고, 거둔 적마다
    /// 체력이 `heal` 돈다(`most`마리까지 — 받는 피해를 줄이지 않는다, 되찾을 뿐 0095 원칙 2). 흡수를 걸어 그 뒤의 타도 되돌린다.
    Reap {
        radius: f32,
        heal: i32,
        most: u8,
        time: f32,
    },
    /// 저주 새기기(0274 ⑥ — 완드·균열, BG3 흑마법사의 "Hex"): 앞 `range` m 안의 적 하나(크로스헤어 > 조준 > 락온 > 가장 가까운)에게
    /// 저주를 새긴다 — 표의 피해 + 균열, `secs`초 동안 그 적에 표가 남고(`Enemy::marks.hex`), 표가 남은 채 쓰러지면 둘레 `spread` m 안의
    /// 가장 가까운 적에게 옮는다(남은 초 그대로 · 균열을 다시 가득 — `World::tick_embers`). 앞이 비면 허공에 흩어진다(대기는 쓴다).
    Hex { range: f32, secs: f32, spread: f32 },
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

/// 모양이 겨누는 법(0278 R16 — `FormInfo::aim`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Aim {
    /// 몸 둘레 — 쪽이 없다(바라보는 쪽 그대로, 터치에서 끌지 않는다).
    Round,
    /// 쪽이 있는 범위(길·줄·부채꼴·놓는 자리): 숄더뷰에서도 땅 위 표시로 겨눈다(0217).
    Area,
    /// 한 대상(0230): 숄더뷰에서 크로스헤어로 나간다.
    Sighted,
}

/// 4단(`SIZE_RANK`)에서 커지는 치수(0278 R16 — 기술 탭의 글 한 줄).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Grow {
    Radius,
    Length,
    Range,
    Arc,
}

/// 쓴 뒤 남는 것(0278 R16 — `FormInfo::leaves`): HUD·터치 칸의 남은 몫(`World::skill_active`)과 시동 흔들림(`app::cue`)이 읽는다.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Leaves {
    Nothing,
    /// 몸에 두르는 것(되울림의 울림).
    Aura,
    /// 바닥에 놓인 것(`World::skill_fields` — 불티 · 눈발/그늘, 고르기가 덤으로 남기는 것(불길 ㄴ의 타는 땅)은 그 스킬의
    /// 모양이 아니다): 놓인 동안 친다.
    Ground,
    /// 바닥에 심은 덫(`Field::trap`): 밟으면 한 번.
    Snare,
}

/// 모양의 값 — 수(payload)와 상관없는 성질 한 줄(0278 R16). 수에 따라 바뀌는 성질(판정 시간 `Skill::active`, 동작의 주기
/// `Form::clip_period`)은 바로 아래의 메서드, 동작(틱 · 미리보기 · 움직임)은 쓰는 자리의 빠짐없는 match.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FormInfo {
    /// 겨누는 법 — `Form::aims`(터치 끌기) · `Form::sighted`(크로스헤어).
    pub aim: Aim,
    /// 4단에서 커지는 치수.
    pub grows: Grow,
    /// 쓴 뒤 남는 것.
    pub leaves: Leaves,
    /// 체력을 되돌리는 꼴(재기 · 생기 거두기) — 기술 탭의 2단 · 5단 글.
    pub mends: bool,
}

impl Form {
    /// 모양의 성질(0278 R16): 와일드카드 없는 한 곳 — 새 모양은 여기서 컴파일이 깨지고, 이 줄을 채우면 겨누기 · 남는 몫 ·
    /// 흔들림 · 기술 탭의 글이 따라온다.
    pub const fn info(&self) -> FormInfo {
        const fn row(aim: Aim, grows: Grow, leaves: Leaves) -> FormInfo {
            FormInfo {
                aim,
                grows,
                leaves,
                mends: false,
            }
        }
        use Aim::{Area, Round, Sighted};
        use Grow::{Arc, Length, Radius, Range};
        match self {
            Form::Ring { .. } => row(Round, Radius, Leaves::Nothing),
            Form::Lane { .. } => row(Area, Length, Leaves::Nothing),
            Form::Pull { .. } => row(Area, Range, Leaves::Nothing),
            Form::Cone { .. } => row(Area, Arc, Leaves::Nothing),
            Form::Drift { .. } => row(Area, Radius, Leaves::Ground),
            Form::Trap { .. } => row(Area, Radius, Leaves::Snare),
            Form::Wave { .. } => row(Round, Radius, Leaves::Nothing),
            Form::Brace { .. } => row(Round, Radius, Leaves::Nothing),
            Form::Echo { .. } => row(Round, Radius, Leaves::Aura),
            Form::Throw { .. } => row(Sighted, Range, Leaves::Nothing),
            Form::Field { .. } => row(Round, Radius, Leaves::Ground),
            Form::Hop { .. } => row(Sighted, Range, Leaves::Nothing),
            Form::Bash { .. } => row(Area, Arc, Leaves::Nothing),
            Form::Stab { .. } => row(Area, Range, Leaves::Nothing),
            Form::Bell { .. } => row(Sighted, Radius, Leaves::Nothing),
            Form::Rally { .. } => FormInfo {
                mends: true,
                ..row(Round, Radius, Leaves::Nothing)
            },
            Form::Slip { .. } => row(Area, Range, Leaves::Nothing),
            Form::Reap { .. } => FormInfo {
                mends: true,
                ..row(Round, Radius, Leaves::Nothing)
            },
            Form::Hex { .. } => row(Sighted, Range, Leaves::Nothing),
            Form::Erupt { .. } => row(Area, Length, Leaves::Nothing),
        }
    }

    /// 쪽이 있는 모양인가(0217 — 터치에서 끌어 겨눈다): 몸 둘레는 쪽이 없다(`Aim::Round`).
    pub const fn aims(&self) -> bool {
        !matches!(self.info().aim, Aim::Round)
    }

    /// 한 대상을 겨누는 모양인가(0230 — 사장 "범위공격이면 바닥 표시가 맞고 단일타겟이라면 크로스헤어 방식"): 숄더뷰에서
    /// 크로스헤어로 나간다. 범위(길 · 줄 · 부채꼴)는 숄더뷰에서도 땅 위 표시로 겨눈다.
    pub const fn sighted(&self) -> bool {
        matches!(self.info().aim, Aim::Sighted)
    }

    /// 스킬 동작의 이어짐 구간을 되풀이하는 주기(0158 G2): 휘돌이는 바퀴마다, 가로지르기는 길마다(되돌아 베기의 둘째 길),
    /// 제비걸음(0214)은 걸음마다. 나머지 모양은 이어짐 구간 전체에 늘인다(수에 따라 바뀌어 `info`가 아니라 여기 — 0278 R16).
    pub fn clip_period(&self) -> Option<f32> {
        match *self {
            Form::Ring { gap, .. } => Some(gap),
            Form::Lane { time, .. } => Some(time),
            Form::Hop { gap, .. } => Some(gap),
            Form::Pull { .. }
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
            | Form::Erupt { .. } => None,
        }
    }
}

/// 뿌리 솟기의 뿌리 수(`len` ÷ `step`, 적어도 하나).
pub fn erupt_count(len: f32, step: f32) -> u8 {
    (len / step.max(0.1)).round().clamp(1.0, 24.0) as u8
}
