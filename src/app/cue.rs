//! 월드 사건 → 앱의 반응(0278 R15): 사건 하나가 내는 소리 · 화면 흔들림 · 카메라 킥 · 타격 줌 · 카메라의 일을 한 표에서.
//!
//! 예전에는 같은 `Event`를 `world_sounds`(소리)와 `step_world`의 match 셋(흔들림 · 킥 · 줌)이 따로 보았고, 셋은 `_ =>`로 끝나
//! 새 사건이 빠져도 조용했다. 여기의 `cue`는 **와일드카드 없는 match 하나** — 새 `Event`는 이 함수에서 컴파일이 깨지고, 그
//! 갈래에서 네 가지를 함께 정한다. 순수하다(월드 · 카메라 · 소리 장치를 모른다 — 시험이 그대로 부른다). 스킬마다의 소리 · 흔들림은
//! 정체성이라(0278 2.3) 갈래 안의 스킬별 match로 둔다. 이펙트(`fx`의 묶음 넷)와 튜토리얼의 셈은 따로다.
use macroquad::prelude::Vec3;

use crate::sfx::Id;
use crate::skills::Id as Sk;
use crate::status::Kind;
use crate::tuning::{
    KICK_HIT_DEG, KICK_HURT_DEG, KICK_SMASH_DEG, SHAKE_CHARGED, SHAKE_CRIT, SHAKE_DOWN, SHAKE_HIT,
    SHAKE_HURT, SHAKE_PARRY, SHAKE_PLUNGE, SHAKE_ROUSE, SHAKE_SLAM, SHAKE_SMASH, SHAKE_TURN,
    ZOOM_CRIT, ZOOM_PARRY, ZOOM_PERFECT, ZOOM_SMASH,
};
use crate::weapons::Look;
use crate::world::Event;

/// 수레 · 적의 큰 타가 화면을 흔드는 거리(m) — 멀리 있는 것이 화면을 흔들지 않게.
const NEAR_SHAKE: f32 = 14.0;
/// 수레 소리가 들리는 거리(m) — 멀리서 내내 울리지 않게.
const NEAR_SOUND: f32 = 18.0;

/// 사건 하나에 앱이 하는 일. 흔들림 · 킥 · 줌은 설정(흔들림)과 랩 `feel.zoom`을 곱하기 **전**의 값.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct Cue {
    /// 낼 소리(`None` = 일부러 조용 — 시험의 `SILENT`가 이름을 댄다).
    pub sound: Option<Id>,
    /// 화면 흔들림(trauma에 더한다).
    pub shake: f32,
    /// 카메라 킥(2026-11-06): 그 자리 쪽으로 몇 도 찼다 돌아온다 — (자리, 도).
    pub punch: Option<(Vec3, f32)>,
    /// 타격 줌(0145 `Camera::punch_in`의 몫).
    pub zoom: f32,
    /// 카메라가 할 일(땅 돌리기 · 락온을 잃음).
    pub camera: Option<CamDo>,
}

/// 사건이 카메라에 시키는 일.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CamDo {
    /// 땅 돌리기(0012): 90° 칸 수만큼 돈다.
    Turn(i32),
    /// 락온 대상을 잃었다(0181): 풀린 뒤 자유 숄더뷰에 머문다.
    LoseLock,
}

/// `cue`가 사건 밖에서 읽는 것 — 예전 match들이 읽던 것만.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct CueCtx {
    /// 박자 타의 이음(1부터 — 잇달으면 종이 오른다).
    pub chain: u32,
    /// 방랑자 자리(가까울 때만 흔들고 들리는 것).
    pub player: Vec3,
    /// 방랑자 속도(맞음의 킥은 밀려난 쪽으로).
    pub player_vel: Vec3,
    /// 버팀이 셋을 다 받아냈다(그 터뜨림은 큰 순간 — 줌).
    pub brace_full: bool,
}

impl Cue {
    const fn of(sound: Option<Id>, shake: f32) -> Cue {
        Cue {
            sound,
            shake,
            punch: None,
            zoom: 0.0,
            camera: None,
        }
    }
    const fn snd(id: Id, shake: f32) -> Cue {
        Cue::of(Some(id), shake)
    }
    const fn quiet(shake: f32) -> Cue {
        Cue::of(None, shake)
    }
    const fn zoom(mut self, z: f32) -> Cue {
        self.zoom = z;
        self
    }
    const fn punch(mut self, at: Vec3, deg: f32) -> Cue {
        self.punch = Some((at, deg));
        self
    }
}

/// 사건 → 반응. 와일드카드 없는 match — 새 사건은 여기서 소리 · 흔들림 · 킥 · 줌을 정해야 컴파일된다.
pub(super) fn cue(e: &Event, cx: &CueCtx) -> Cue {
    let near = |at: Vec3, d: f32| at.distance(cx.player) < d;
    match *e {
        Event::Swing => Cue::snd(Id::Swing, 0.0),
        // 등딱지 튕김(0120)은 패링의 거울 — 같은 쇳소리.
        Event::Deflect { .. } => Cue::snd(Id::Parry, SHAKE_HIT),
        // 카메라 킥(2026-11-06, KB melee-light…: 작은 타격은 trauma²에서 0이 된다 — 맞은 쪽으로 2~3px 찼다 돌아온다).
        Event::Hit { at } => Cue::snd(Id::Hit, SHAKE_HIT).punch(at, KICK_HIT_DEG),
        Event::EnemyDown { .. } => Cue::snd(Id::Die, SHAKE_DOWN),
        // 맞음(0143): 밀려난 쪽으로 킥 — 흔들림만으로는 어디서 맞았는지가 없다.
        Event::PlayerHurt => {
            Cue::snd(Id::Hurt, SHAKE_HURT).punch(cx.player + cx.player_vel, KICK_HURT_DEG)
        }
        Event::LockOn | Event::LockOff => Cue::quiet(0.0),
        // 락온 대상을 잃으면(0181) 풀린 뒤 자유 숄더뷰에 머문다 — 버튼으로 풀면 락온한 곳(쿼터뷰·자유)으로.
        Event::LockLost => Cue {
            camera: Some(CamDo::LoseLock),
            ..Cue::quiet(0.0)
        },
        // 땅 돌리기: 카메라가 90° 돈다(입력 기준은 다음 틱부터 새 방위 — 쥔 스틱은 월드가 그대로 둔다).
        Event::Turn { quarters } => Cue {
            camera: Some(CamDo::Turn(quarters)),
            ..Cue::quiet(SHAKE_TURN)
        },
        // 0278: 벽 쿵 = 둔한 쿵.
        Event::Slam { .. } => Cue::snd(Id::Slam, SHAKE_SLAM),
        // 수레가 떠난다(0208): 둔한 덜컹 — 소리 · 흔들림 모두 가까울 때만(흔들림은 작게).
        Event::CartGo { at } => Cue::of(
            near(at, NEAR_SOUND).then_some(Id::Thwack),
            if near(at, NEAR_SHAKE) {
                SHAKE_SLAM * 0.5
            } else {
                0.0
            },
        ),
        Event::CartWarn { at } => Cue::of(near(at, NEAR_SOUND).then_some(Id::TeleLane), 0.0),
        // 사슬이 끊긴다(0208 3단계): 깊은 울림 + 2단계 전환보다 크게(글 한 줄은 `world_sounds`).
        Event::Unchain { .. } => Cue::snd(Id::Wake, SHAKE_ROUSE * 1.4),
        // 적의 공격이 풀린 순간(0269): 문지기·보스가 땅을 치는 것만 가까울 때 덜컹 — 잡몹의 타는 그림이 말한다. 소리는 없다(예고가 냈다).
        Event::EnemyStrike { at, boss, blow, .. } => {
            use crate::enemy::Blow;
            let ground = match blow {
                Blow::Volley | Blow::Hook { .. } => false,
                Blow::Around { .. }
                | Blow::Cone { .. }
                | Blow::Spot { .. }
                | Blow::Ram
                | Blow::Quake { .. }
                | Blow::Line { .. } => true,
            };
            Cue::quiet(if boss && ground && near(at, NEAR_SHAKE) {
                SHAKE_SLAM * 0.6
            } else {
                0.0
            })
        }
        // 지연 공격이 풀린다(0128): 짧게 들이쉬는 소리 — 이 뒤 `RELEASE_LEAD`초에 맞는다.
        Event::Release { .. } => Cue::snd(Id::TeleRelease, 0.0),
        // 0278: 2단계 = 부풀어 오르는 으르렁.
        Event::Rouse { .. } => Cue::snd(Id::Rouse, SHAKE_ROUSE),
        Event::Ember { .. } => Cue::snd(Id::Ember, 0.0),
        // 방패(0223): 받은 타는 둔한 '둥'(튕김의 맑은 소리와 갈린다), 깨지면 맞은 소리.
        Event::Guard { .. } => Cue::snd(Id::Thwack, SHAKE_HIT),
        Event::GuardBreak { .. } => Cue::snd(Id::Hurt, SHAKE_HURT),
        // 피해 타입(0223 V2): 화상이 붙음 = 잔불 소리, 묶임 = 뿌리가 깨는 소리, 금 = 쇳소리 — 게이지가 차는 것은 흔들지 않는다.
        Event::Ail { kind, .. } => Cue::of(ail_sound(kind), 0.0),
        // 화상이 타는 틱(초마다 — 떼에서 겹친다): 걸린 순간의 `Ail`이 잔불 소리를 낸다.
        Event::Burn { .. } => Cue::quiet(0.0),
        Event::Bleed { .. } => Cue::quiet(0.0),
        // 0270 B: 깨짐은 툭(돌 소리).
        Event::Shatter { .. } => Cue::snd(Id::Shell, SHAKE_HIT),
        // 0278: 흡수 = 작게 오르는 음(자주 나므로 `sfx`가 0.25초에 한 번만).
        Event::Drain { .. } => Cue::snd(Id::Drain, 0.0),
        // 울림·터짐은 둘레를 치는 순간이라 가볍게(울림 = 낮은 종, 터짐 = 타격).
        Event::Toll { .. } => Cue::snd(Id::Beat, SHAKE_HIT),
        Event::AilBurst { .. } => Cue::snd(Id::Hit, SHAKE_PLUNGE),
        // 바꾸는 딸깍은 `on_weapon_swap`이 낸다.
        Event::WeaponSwap { .. } => Cue::quiet(0.0),
        // 교체 공격(0272 1b): 바꾸는 딸깍 위에 몸이 달려드는 바람 — 그냥 바꾸기와 귀로 갈린다.
        Event::SwapStrike { .. } => Cue::snd(Id::Dodge, 0.0),
        // 타격 줌(0145): 센 타격의 순간에만.
        Event::Parry { .. } => Cue::snd(Id::Parry, SHAKE_PARRY).zoom(ZOOM_PARRY),
        Event::Crit { at } => Cue::snd(Id::Hit, SHAKE_CRIT)
            .punch(at, KICK_HIT_DEG)
            .zoom(ZOOM_CRIT),
        Event::Plunge { .. } => Cue::snd(Id::Hit, SHAKE_PLUNGE),
        // 0278: 다 모임 = 짧게 오르는 '찡'(떼면 센 것).
        Event::Charged => Cue::snd(Id::Charged, SHAKE_CHARGED),
        Event::Gulp { .. } => Cue::snd(Id::Gulp, 0.0),
        Event::FlaskDenied => Cue::snd(Id::Deny, 0.0),
        Event::Smash { at } => Cue::snd(Id::Hit, SHAKE_SMASH)
            .punch(at, KICK_SMASH_DEG)
            .zoom(ZOOM_SMASH),
        // 종류의 RB(0274 ②b): 꼴마다의 소리(`cast_sound`), 땅 내려찍기(0274 ④)는 땅을 친 만큼 조금 흔들린다.
        Event::Cast { look, .. } => Cue::of(
            cast_sound(look),
            if look == Look::Quake { SHAKE_HIT } else { 0.0 },
        ),
        // 신성한 빛(0274 ②b)이 꽂히는 순간 = 가장 높은 종, 여진만큼(맞은 것은 맞힘 소리가 따로).
        Event::Smite { .. } => Cue::snd(Id::Beat3, SHAKE_PLUNGE),
        // 가시 씨앗(0274 ⑤): 뿌리가 솟는 낮은 울림, 가볍게.
        Event::Sprout { .. } => Cue::snd(Id::Wake, SHAKE_HIT),
        // 창이 손에 돌아옴 = 던지기의 돌아옴과 같은 종, 조용히.
        Event::JavelinBack { .. } => Cue::snd(Id::Beat, 0.0),
        Event::CastDenied | Event::StaminaDenied => Cue::snd(Id::Deny, 0.0),
        Event::Break { at } => Cue::snd(Id::Hit, SHAKE_SMASH)
            .punch(at, KICK_SMASH_DEG)
            .zoom(ZOOM_SMASH),
        Event::LoadBreak { .. } => Cue::snd(Id::Hit, SHAKE_SMASH),
        Event::Aftershock { .. } | Event::Boom { .. } => Cue::snd(Id::Hit, SHAKE_PLUNGE),
        // 딱 맞춰 떼기(0078): 박자와 같은 "맞췄다" 종 — 가장 높은 음.
        Event::Perfect { .. } => Cue::snd(Id::Beat3, SHAKE_CHARGED).zoom(ZOOM_PERFECT),
        // 0278: 수류탄 던짐 = 도는 휘익.
        Event::Throw => Cue::snd(Id::Throw, 0.0),
        Event::Beat => Cue::snd(crate::sfx::beat_id(cx.chain), 0.0),
        Event::SkillStart { id } => Cue::snd(skill_start_sound(id), 0.0),
        // 스킬(0151): 판정이 나간 순간에 한 번(`skill_kick`). 0158 G2의 큰 순간: 땅울림이 찍는 순간, 셋을 다 받아낸 버팀의 터뜨림.
        Event::Skill { id, .. } => {
            Cue::snd(skill_sound(id), skill_kick(id)).zoom(skill_zoom(id, cx.brace_full))
        }
        Event::SkillDenied => Cue::snd(Id::Deny, 0.0),
        // 큰 기술(0151): 시동의 세계 멈춤 동안 당겨 두고(멈춤 동안 머문다), 뒤집히는 순간 한 번 더 — 깨어나는 보스만큼 흔든다.
        Event::UltStart => Cue::snd(Id::Wake, 0.0).zoom(ZOOM_SMASH),
        Event::Ult { .. } => Cue::snd(Id::Hit, SHAKE_ROUSE).zoom(ZOOM_SMASH),
        // 게이지 가득 = 가장 높은 종.
        Event::UltReady => Cue::snd(Id::Beat3, 0.0),
        Event::SkillBurst { .. } => Cue::snd(Id::Hit, SHAKE_PLUNGE),
        Event::Mend { .. } => Cue::snd(Id::Gulp, 0.0),
        // 0158 G2: 받아냄 = 쇳소리, 울림 = 가운데 종(받아냄·울림·되돎은 가볍게), 되돎 = 휘두름, 터짐·한 줄기 = 타격,
        // 손에 돌아옴 = 낮은 종, 몰아치기 = 가장 높은 종.
        Event::Interrupt { .. } | Event::Soak { .. } => Cue::snd(Id::Parry, SHAKE_HIT),
        Event::RootSpike { .. } => Cue::quiet(0.0),
        Event::EchoRing { .. } => Cue::snd(Id::Beat2, SHAKE_HIT),
        Event::ThrowTurn { .. } => Cue::snd(Id::Swing, 0.0),
        Event::ThrowBoom { .. } => Cue::snd(Id::Hit, SHAKE_PLUNGE),
        Event::ThrowBack { .. } => Cue::snd(Id::Beat, 0.0),
        // 0270 C3: 눈발 = 높은 종, 뿌리 덫 = 뿌리가 깨는 소리, 불티 = 잔불 소리 — 흔들지 않는다.
        Event::FieldPulse { kind, .. } => Cue::snd(field_sound(kind), 0.0),
        Event::Flurry { .. } => Cue::snd(Id::Beat3, 0.0),
        Event::LineCut { .. } => Cue::snd(Id::Hit, SHAKE_ROUSE).zoom(ZOOM_SMASH),
        // 0270 C3: 큰 종 = 낮은 종, 눈보라 = 가장 높은 종.
        Event::GreatBell { .. } => Cue::snd(Id::Beat, SHAKE_ROUSE).zoom(ZOOM_SMASH),
        Event::Blizzard { .. } => Cue::snd(Id::Beat3, SHAKE_PLUNGE),
        // 0272: 화살비 = 스치는 바람(시위를 떠난 화살들) — 내리는 타는 `FieldPulse`.
        Event::ArrowRain { .. } => Cue::snd(Id::Dodge, SHAKE_PLUNGE),
        // 궁극기의 여러 타(0272 2단계 · 0274 ③~⑥): 타마다 가볍게, 마지막은 큰 기술만큼(타격 · 줌).
        // 질풍 찌르기: 찌름마다 스치는 바람.
        Event::GaleThrust { last, .. } => last_or(last, Id::Hit, Id::Dodge),
        // 그림자 습격(0274 ③): 숨어듦 = 스치는 바람(조용히), 찌름마다 휘두름.
        Event::Blink { .. } => Cue::snd(Id::Dodge, 0.0),
        Event::AmbushStab { last, .. } => last_or(last, Id::Hit, Id::Swing),
        // 칼바람(0274 ④): 휩쓸기마다 휘두름.
        Event::TempestSweep { last, .. } => last_or(last, Id::Hit, Id::Swing),
        // 깨어나는 숲(0274 ⑤): 고리마다 낮은 울림(땅이 솟는다), 마지막은 둔한 소리(되찾는 소리는 `Mend`).
        Event::GrovePulse { last, .. } => last_or(last, Id::Thwack, Id::Wake),
        // 치른 값(0274 ⑥): 번지는 저주 = 쇳소리(금), 터지는 울림 = 낮은 종.
        Event::PricePulse { last, .. } => last_or(last, Id::Beat, Id::Parry),
        // 저주의 표가 새겨짐: 저주 새기기의 판정 `Skill`이 쇳소리를 낸다. 옮음 = 스치는 바람, 죽음의 울림 = 가운데 종(가볍게).
        Event::Hexed { .. } => Cue::quiet(0.0),
        Event::HexJump { .. } => Cue::snd(Id::Dodge, 0.0),
        Event::DeathKnell { .. } => Cue::snd(Id::Beat2, SHAKE_HIT),
        // 0276: 기도하는 손 = 가운데 종 · 덫이 묾 = 쇳소리(둘 다 가볍게) · 덫을 놓음 = 던짐 · 방패가 튕김 = 낮은 종 ·
        // 들이받기의 출발 = 스치는 바람(맞힌 타가 제 몫을 낸다).
        Event::Prayer { .. } => Cue::snd(Id::Beat2, SHAKE_HIT),
        Event::SnareSet { .. } => Cue::snd(Id::Swing, 0.0),
        Event::SnareSnap { .. } => Cue::snd(Id::Parry, SHAKE_HIT),
        Event::ShieldBounce { .. } => Cue::snd(Id::Beat, 0.0),
        Event::Rush { .. } => Cue::snd(Id::Dodge, 0.0),
        // 0277 → 0284: 급소 = 베어 드는 소리('슉' → '푹') — 치명타만큼 읽히게(사장 판정 p-dagger "티가 안 남"): 치명타의 흔들림 ·
        // 강타의 킥 · 치명타의 줌(빛은 금이 아니라 은 — `fx`). 분노가 가득 = 낮게 차오르는 숨, 걸어 당김 = 덩굴이 감겨 끌리는
        // 소리(둘은 조용히 — 그림 · 소리가 말한다).
        Event::Sneak { at } => Cue::snd(Id::Sneak, SHAKE_CRIT)
            .punch(at, KICK_SMASH_DEG)
            .zoom(ZOOM_CRIT),
        Event::FuryFull { .. } => Cue::snd(Id::Fury, 0.0),
        Event::Pull { .. } => Cue::snd(Id::Pull, 0.0),
    }
}

/// 여러 타의 궁극기: 마지막 타는 큰 기술만큼(흔들림 · 줌), 그 앞은 가볍게.
const fn last_or(last: bool, end: Id, each: Id) -> Cue {
    if last {
        Cue::snd(end, SHAKE_ROUSE).zoom(ZOOM_SMASH)
    } else {
        Cue::snd(each, SHAKE_HIT)
    }
}

/// 종류의 RB의 소리(0274 ②b — 있는 소리를 겹쳐 쓴다): 던지는 것(손도끼 · 창 · 돌 · 단검 · 방패) = 휘두름 바람, 신성한 빛을 부름 =
/// 맑은 종 둘째 음, 땅 내려찍기 = 낮은 울림(땅이 갈라진다). 마법탄 · 저주탄 · 가시 씨앗 · 사격은 소리 없이(그대로). 꼴마다 와일드카드
/// 없이(0278 R17 — 소리는 앱의 것이라 `LookDef`가 아니라 여기).
fn cast_sound(look: Look) -> Option<Id> {
    match look {
        Look::Axe | Look::Javelin | Look::Stone | Look::Knife | Look::Shield => Some(Id::Swing),
        Look::Light => Some(Id::Beat2),
        Look::Quake => Some(Id::Wake),
        Look::Magic | Look::Bolt | Look::Arrow | Look::Seed | Look::Curse => None,
    }
}

/// 타입 게이지가 가득 찬 순간의 소리(0223 V2 · 0270 B). 물리는 게이지가 없다.
fn ail_sound(kind: Kind) -> Option<Id> {
    Some(match kind {
        Kind::Ember => Id::Ember,
        Kind::Root => Id::Wake,
        Kind::Rift => Id::Parry,
        // 0270 B: 기절 = 타격 · 상처 = 휘두름 · 얼어붙음 = 가장 높은 종 · 흡수 = 한 모금.
        Kind::Shock => Id::Thwack,
        Kind::Sharp => Id::Swing,
        Kind::Frost => Id::Beat3,
        Kind::Drain => Id::Gulp,
        // 울림은 `Toll`이 낸다.
        Kind::Physical | Kind::Knell => return None,
    })
}

/// 놓인 것이 치는 소리(0270 C3): 눈발 = 높은 종, 뿌리 덫 = 뿌리가 깨는 소리, 그 밖(불티) = 잔불 소리.
fn field_sound(kind: Kind) -> Id {
    match kind {
        Kind::Frost => Id::Beat3,
        Kind::Root => Id::Wake,
        Kind::Physical
        | Kind::Ember
        | Kind::Knell
        | Kind::Rift
        | Kind::Shock
        | Kind::Sharp
        | Kind::Drain => Id::Ember,
    }
}

/// 스킬의 시동 소리(0151 — S0는 있는 소리를 겹쳐 쓴다, 0158 G2 스킬마다 다른 짝): 버팀은 발을 딛는 둔한 소리, 되울림은 낮은 종,
/// 잔불 흩기는 잔불 소리, 나머지는 휘두름.
fn skill_start_sound(id: Sk) -> Id {
    match id {
        Sk::Brace => Id::Thwack,
        Sk::Echo => Id::Beat,
        Sk::Embers | Sk::Flare => Id::Ember,
        // 0270 C3: 눈발 = 가장 높은 종(서리), 뿌리 덫 = 휘두름.
        Sk::Snow => Id::Beat3,
        Sk::Bash | Sk::Ram => Id::Thwack,
        Sk::Whirl
        | Sk::Cut
        | Sk::Gather
        | Sk::Quake
        | Sk::Throw
        | Sk::Swallow
        | Sk::Roots
        | Sk::Stitch
        | Sk::Toll
        | Sk::Swat
        | Sk::Ripple
        | Sk::Snare
        | Sk::Mine
        | Sk::Skewer
        | Sk::Pole
        | Sk::Pin
        | Sk::Pommel
        | Sk::Shove
        | Sk::Slip
        | Sk::Sunder
        | Sk::Reap
        | Sk::Harvest
        | Sk::Hex
        | Sk::Withdraw
        | Sk::Boomerang => Id::Swing,
        // 0280: 불씨 내리꽂기 = 잔불 소리(달군 날), 끊는 말 = 가운데 종(말 한마디가 울린다).
        Sk::Plunge => Id::Ember,
        Sk::Word => Id::Beat2,
        // 0274 ⑥: 시드는 그늘 = 낮은 울림.
        Sk::Blight => Id::Wake,
        // 0274 ④: 불씨 휩쓸기 = 잔불 소리(달군 날).
        Sk::Sear => Id::Ember,
        // 0274: 재기 = 발을 딛는 둔한 소리(숨을 몰아쉬고 선다).
        Sk::Rally => Id::Thwack,
        // 0272: 심지 볼트 = 잔불 소리(심지에 불이 붙는다).
        Sk::Fuse => Id::Ember,
    }
}

/// 스킬의 판정이 나간 순간의 소리(0158 G2): 휘돌이 = 휘두름(바퀴마다), 가로지르기·던지기 = 스치는 바람, 끌어모으기·버팀의 터뜨림 =
/// 둔한 소리, 땅울림 = 낮은 울림, 되울림 = 가운데 종, 잔불 흩기 = 잔불 소리.
fn skill_sound(id: Sk) -> Id {
    match id {
        Sk::Whirl => Id::Swing,
        Sk::Cut | Sk::Throw => Id::Dodge,
        Sk::Gather | Sk::Brace => Id::Thwack,
        Sk::Quake => Id::Wake,
        Sk::Echo => Id::Beat2,
        Sk::Embers => Id::Ember,
        Sk::Swallow => Id::Dodge,
        Sk::Bash => Id::Parry,
        Sk::Roots => Id::Wake,
        // 0270 C3: 바늘땀 = 땀마다 스치는 바람, 방패 돌진 = 둔한 소리, 종 치기 = 가운데 종(울림은 `Toll`이 낸다).
        Sk::Stitch => Id::Dodge,
        Sk::Ram => Id::Thwack,
        Sk::Toll => Id::Beat2,
        // 후려치기 = 타격, 파문 = 낮은 울림.
        Sk::Swat => Id::Hit,
        Sk::Ripple => Id::Wake,
        // 눈발 = 높은 종, 불길 = 잔불 소리, 뿌리 덫(심음) = 둔한 소리.
        Sk::Snow => Id::Beat3,
        Sk::Flare => Id::Ember,
        Sk::Snare => Id::Thwack,
        // 0272: 심지 볼트(쏘는 순간) = 스치는 바람, 화약 덫(묻음) = 둔한 소리 — 터짐은 `Boom`이 낸다.
        Sk::Fuse => Id::Dodge,
        Sk::Mine => Id::Thwack,
        // 0272 2단계(창): 꿰뚫기 = 스치는 바람(내달림), 장대 휩쓸기 = 휘두름, 못 박기 = 타격(박히는 소리).
        Sk::Skewer => Id::Dodge,
        Sk::Pole => Id::Swing,
        Sk::Pin => Id::Hit,
        // 0274(한손검): 자루 치기 = 타격, 밀치기 = 둔한 소리, 재기 = 가운데 종(체력이 도는 소리는 `Mend`가 낸다).
        Sk::Pommel => Id::Hit,
        Sk::Shove => Id::Thwack,
        Sk::Rally => Id::Beat2,
        // 0274 ③(쌍단검): 빠지며 베기 = 스치는 바람(물러나는 걸음 — 벤 것은 맞힘 소리가 따로).
        Sk::Slip => Id::Dodge,
        // 0274 ④(대검): 땅 가르기 = 낮은 울림(땅이 갈라진다), 불씨 휩쓸기 = 휘두름(불은 `SkillStart`의 잔불 소리).
        Sk::Sunder => Id::Wake,
        Sk::Sear => Id::Swing,
        // 0274 ⑤(낫): 걸어 당기기 = 둔한 소리(끌려오는 몸), 생기 거두기 = 휘두름(되찾는 소리는 `Mend`가 낸다).
        Sk::Reap => Id::Thwack,
        Sk::Harvest => Id::Swing,
        // 0274 ⑥(완드): 저주 새기기 = 쇳소리(금이 가는 소리), 시드는 그늘 = 낮은 울림(그늘이 드리운다).
        Sk::Hex => Id::Parry,
        Sk::Blight => Id::Wake,
        // 0280: 찌르고 물러서기 = 타격(박히는 창끝 — 물러나는 걸음은 소리 없이), 불씨 내리꽂기 = 낮은 울림(땅에 꽂힌다 — 불은
        // 시동의 잔불 소리), 돌아오는 낫 = 스치는 바람(던지기와 같은 손), 끊는 말 = 쇳소리(끊기는 소리).
        Sk::Withdraw => Id::Hit,
        Sk::Plunge => Id::Wake,
        Sk::Boomerang => Id::Dodge,
        Sk::Word => Id::Parry,
    }
}

/// 스킬의 판정이 나간 순간의 타격 줌(0158 G2의 큰 순간): 땅울림이 찍는 순간, 셋을 다 받아낸 버팀의 터뜨림(받아낸 수는 다음 버팀까지
/// 남는다). 그 밖은 줌 없이.
fn skill_zoom(id: Sk, brace_full: bool) -> f32 {
    match id {
        Sk::Quake => ZOOM_CRIT,
        Sk::Brace if brace_full => ZOOM_SMASH,
        Sk::Brace
        | Sk::Whirl
        | Sk::Cut
        | Sk::Gather
        | Sk::Echo
        | Sk::Embers
        | Sk::Throw
        | Sk::Swallow
        | Sk::Bash
        | Sk::Roots
        | Sk::Stitch
        | Sk::Ram
        | Sk::Toll
        | Sk::Swat
        | Sk::Ripple
        | Sk::Snow
        | Sk::Flare
        | Sk::Snare
        | Sk::Fuse
        | Sk::Mine
        | Sk::Skewer
        | Sk::Pole
        | Sk::Pin
        | Sk::Pommel
        | Sk::Shove
        | Sk::Rally
        | Sk::Slip
        | Sk::Sunder
        | Sk::Sear
        | Sk::Reap
        | Sk::Harvest
        | Sk::Hex
        | Sk::Blight
        | Sk::Withdraw
        | Sk::Plunge
        | Sk::Boomerang
        | Sk::Word => 0.0,
    }
}

/// 스킬의 판정이 나간 순간의 흔들림(0151). 몸에 두르는 것(되울림)·바닥에 놓는 것(`Form::info().leaves` — 불티 · 눈발 · 그늘 · 덫)은
/// 흔들지 않는다 — 울림·불티·덫의 타가 제 몫을 낸다(0278 R14: 이름 목록이던 때 화약 덫 · 시드는 그늘이 빠져 있었다). 바늘땀(0270 C3)은
/// 땀마다 — 가볍게.
pub(super) fn skill_kick(id: Sk) -> f32 {
    let form = id.def().form;
    if form.info().leaves != crate::skills::Leaves::Nothing {
        0.0
    } else if id == Sk::Stitch {
        SHAKE_HIT
    } else {
        SHAKE_PLUNGE
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use macroquad::prelude::vec3;

    /// 사건마다의 반응을 적어 둔 표(0278 R15 — 옛 match 넷에서 만든 것: `the_cue_table_matches_its_snapshot`).
    /// 일부러 반응을 바꿨으면 `CUE_WRITE=1 cargo test cue_table`로 다시 쓴다.
    pub(in crate::app) const SNAPSHOT: &str =
        concat!(env!("CARGO_MANIFEST_DIR"), "/src/app/cue_snapshot.txt");

    /// 견줄 표본: 사건의 표본(`Event::samples`) + 반응이 실린 값으로 갈리는 것들(스킬 38 · 타입 · 탄의 꼴 · 적의 큰 타 · 여러 타의
    /// 마지막) × 맥락 넷(박자 이음 0·1·2·3, 가까이·14~18m·멀리, 버팀을 다 받아냄).
    pub(in crate::app) fn cases() -> Vec<(Event, CueCtx)> {
        let events = Event::varied();
        let ctxs = [
            CueCtx {
                chain: 0,
                player: Vec3::ZERO,
                player_vel: vec3(0.5, 0.0, 0.0),
                brace_full: false,
            },
            CueCtx {
                chain: 1,
                player: vec3(3.0, 0.0, 0.0),
                player_vel: Vec3::ZERO,
                brace_full: true,
            },
            CueCtx {
                chain: 2,
                player: vec3(1.0, 0.0, 14.0),
                player_vel: vec3(0.0, 0.0, -1.0),
                brace_full: false,
            },
            CueCtx {
                chain: 3,
                player: vec3(40.0, 0.0, 0.0),
                player_vel: Vec3::ZERO,
                brace_full: true,
            },
        ];
        events
            .iter()
            .flat_map(|e| ctxs.iter().map(move |cx| (*e, *cx)))
            .collect()
    }

    pub(in crate::app) fn line(e: &Event, cx: &CueCtx, c: &Cue) -> String {
        format!(
            "{e:?} | chain {} at {:?} brace {} => sound {:?} shake {:?} punch {:?} zoom {:?} camera {:?}\n",
            cx.chain, cx.player, cx.brace_full, c.sound, c.shake, c.punch, c.zoom, c.camera
        )
    }

    /// 0278 R15: 사건마다의 반응(소리 · 흔들림 · 킥 · 줌 · 카메라)이 적어 둔 표와 같다 — 표는 옛 `sound_of`와 `step_world`의 match
    /// 셋에서 만들었다(옮기기 전에 둘을 견준 시험이 같음을 보였다).
    #[test]
    fn the_cue_table_matches_its_snapshot() {
        let now: String = cases()
            .iter()
            .map(|(e, cx)| line(e, cx, &cue(e, cx)))
            .collect();
        if std::env::var("CUE_WRITE").is_ok() {
            std::fs::write(SNAPSHOT, &now).unwrap();
            return;
        }
        let was = std::fs::read_to_string(SNAPSHOT).expect("cue_snapshot.txt");
        let diff: Vec<String> = was
            .lines()
            .zip(now.lines())
            .filter(|(a, b)| a != b)
            .map(|(a, b)| format!("was {a}\nnow {b}"))
            .take(12)
            .collect();
        assert!(
            diff.is_empty() && was.lines().count() == now.lines().count(),
            "{} lines was, {} now:\n{}",
            was.lines().count(),
            now.lines().count(),
            diff.join("\n")
        );
    }
}
