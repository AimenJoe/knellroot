//! 적 하나의 상태와 행동(0019). 종류별 수치는 `species.rs` 표, 여기는 행동 방식(근접·돌진·문지기)의 상태 기계.
//! 월드는 `Enemy::tick`을 부르고, 플레이어를 쳤는지·깨어났는지를 받아 피해·이벤트로 바꾼다(월드 경계는 그대로).
use macroquad::prelude::*;

use crate::species::{self, Behavior, Def, Kind};
use crate::tuning::{
    HURL_R, HURL_RING, PLAYER_RADIUS, RISE_BURST, RISE_TIME, ROOTS_LEN, ROOTS_R, ROOTS_SPEED,
};

/// 넉백 속도가 초당 이만큼씩 준다(월드와 같은 값).
pub use crate::world::KNOCK_DECAY;
/// 쏟아지는 적이 이보다 느려지면 멈춘 것으로 본다.
const SLAM_SPEED: f32 = 2.0;
/// 예고 끝 이만큼(초)은 방향을 고정한다 — 읽은 대로 맞는다.
const TELE_LOCK: f32 = 0.12;
/// 돌진이 닿았다고 보는 여유(몸 반지름 + 플레이어 반지름에 더한다).
const CHARGE_TOUCH: f32 = 0.15;
/// 떼 퍼지기(0076): 옆으로 비키는 세기(가는 쪽 단위 벡터에 더한다).
const SPREAD_STEER: f32 = 0.8;
/// 체력 반 아래(문지기): 예고·쉬기 배율, 걸음 배율.
const ROUSED_TELE: f32 = 0.75;
const ROUSED_RECOVER: f32 = 0.7;
const ROUSED_SPEED: f32 = 1.15;

/// 엘리트의 성질(0112, KB — 디아블로식 "있는 종 + 성질 하나"). 이름은 lore 생태(`docs/lore/ecology.md` 3) 관찰과 맞춘다:
/// 몸이 바뀐 것(변형) · 같은 동작을 되풀이하는 것(재활동) · 끝난 자리에 무언가 남는 것(잔류). 색으로 세기를 말하지 않는다(ecology 2).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Affix {
    /// 굳은: 자세 막대 × `ELITE_HARD_POISE`, 밀려나기 × `ELITE_HARD_KNOCK` — 힘 레벨의 자세 피해로 쉽게 깨지지 않는다.
    Hardened,
    /// 되풀이: 한 번 친 뒤 쉬지 않고 같은 공격을 한 번 더(예고부터 다시 — 읽을 수 있게).
    Repeating,
    /// 남는: 쓰러진 자리에 고리 예고가 남고 `ELITE_LINGER_DELAY` 뒤 터진다(패링 못 함 — 피한다).
    Lingering,
}

/// 공격 종류 — 예고 모양과 맞는 방식이 여기서 갈린다.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Atk {
    /// 둘레 내려치기(잔재).
    Smash,
    /// 앞 부채꼴 내려치기(문지기).
    Slam,
    /// 곧은 돌진(가시짐승·문지기).
    Charge,
    /// 퍼지는 뿌리 파동(문지기).
    Wave,
    /// 바닥에 심기(꼬투리, 0120): 예고를 시작할 때의 플레이어 자리(`plant_at`)에 원판 — 끝 = 판정.
    Plant,
    /// 울림 고리(깨진 종지기, 0125): 땅을 쳐 고리가 여러 겹 퍼진다(사이 `TOLL_GAP`) — 사이에 서거나 뛰어넘는다, 받지 못한다.
    Toll,
    /// 종 조각 부채꼴(깨진 종지기, 0125): 예고 끝에 조각을 던진다(월드의 탄) — 패링하면 되돌아간다.
    Shards,
    /// 뿌리 기둥 줄(문지기 2단계, 0133): 바라보는 쪽으로 기둥이 줄지어 솟는다 — 옆으로 비킨다(받지 못한다, 뛰어도 맞는다).
    Roots,
    /// 종 내던지기(종지기 2단계, 0133): 예고 첫 틱의 방랑자 자리(`plant_at`)에 종이 떨어지고(원판), 거기서 고리 한 겹.
    Hurl,
}

/// 바닥 예고 모양(렌더가 그린다).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TeleShape {
    Disc {
        r: f32,
    },
    Sector {
        r: f32,
        arc: f32,
    },
    Lane {
        len: f32,
        w: f32,
    },
    Ring {
        r: f32,
    },
    /// 줄지은 원(0133 뿌리 기둥): 바라보는 쪽으로 `start`부터 `n`개, 반지름 `r`, 가운데 사이 `step` — 원 = 피할 것(0187).
    Row {
        n: u8,
        r: f32,
        step: f32,
        start: f32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EState {
    /// 잠든 문지기(스테이지가 깨운다). 움직이지도 치지도 않는다.
    Dormant,
    /// 지도에 서 있는 적(0113)이 제자리에서 제 일을 한다 — 맞을 수는 있다(싸움에 낀 것). 월드가 알아차리면 쫓는다.
    Idle,
    /// 제자리에서 너무 멀어져 돌아가는 중(0113) — 닿으면 가득 차고 제자리로.
    Return,
    Chase,
    /// 공격 예고(`Enemy::atk`). 다 차면 친다.
    Tele {
        t: f32,
    },
    Swing {
        t: f32,
    },
    /// 돌진 중(방향은 예고 끝에 고정).
    Charge {
        t: f32,
        dir: Vec3,
    },
    /// 뿌리 파동이 퍼지는 중.
    Wave {
        t: f32,
    },
    Recover {
        t: f32,
    },
    /// 보스 2단계 전환(0133): 체력 반에서 하던 공격을 멈추고 `RISE_TIME` 동안 맞지 않는다 — `RISE_BURST`에 둘레를 밀쳐낸다.
    Rise {
        t: f32,
    },
    /// 휘청(CC).
    Stagger {
        t: f32,
        dur: f32,
    },
    Dead,
}

/// 이번 공격의 박자(0128).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tempo {
    Normal,
    /// 예고가 짧다(`FAST_TELE`, 바닥 `TELE_FLOOR`).
    Fast,
    /// 정점에서 이만큼(초) 더 버틴다 — 들어 올림은 같다.
    Delayed(f32),
    /// 연속 공격의 이어지는 타: 예고 배율.
    Chain(f32),
}

pub struct Enemy {
    pub kind: Kind,
    pub pos: Vec3,
    pub prev: Vec3,
    pub facing: f32,
    pub hp: i32,
    pub max_hp: i32,
    pub boss: bool,
    pub radius: f32,
    pub height: f32,
    pub state: EState,
    /// 지금(또는 마지막) 공격.
    pub atk: Atk,
    /// 공격 횟수(문지기 패턴 돌리기).
    pub atk_count: u32,
    /// 체력 반 아래로 깨어났다(문지기 — 핵이 밝아지고 빨라진다).
    pub roused: bool,
    /// 잠에서 깨어나는 중(0149 — 스테이지가 켠다): 아직 맞지도 움직이지도 않지만 **락온에는 잡힌다**. 깨어나는 2초 동안
    /// 보스가 락온 후보에서 빠져, 락온을 누르면 범위 안의 엉뚱한 적이 잡혔다(사장 2026-11-11).
    pub waking: bool,
    /// 이번 고리들 가운데 이미 맞힌 것(울림 고리 0125 — 고리마다 한 번).
    pub rings_hit: u8,
    /// 이번 돌진·파동이 이미 맞혔다(한 번만).
    pub landed: bool,
    pub hurt: f32,
    pub knock: Vec3,
    /// 마지막으로 맞은 스윙 번호(한 스윙에 한 번만).
    pub hit_by: u32,
    /// 땅 돌리기로 쏟아지는 중 — 이때 벽에 부딪히면 벽 쿵(한 번).
    pub spilling: bool,
    /// 휘청 몸짓(렌더가 기울인다).
    pub sway: Vec3,
    pub sway_t: f32,
    /// 죽은 뒤 지난 초(렌더가 쓰러뜨리고 지운다).
    /// 잔불을 떨궜나(0064 M1 — 쓰러진 틱에 한 번).
    pub dropped: bool,
    pub dead_t: f32,
    /// 패링당해 드러난 남은 초(0025) — 이동안 맞는 한 대가 치명타.
    pub exposed: f32,
    /// 자세 막대(0051, PoE2 Heavy Stun): 자세 피해로 차고, 다 차면 깨진다(휘청 + 드러남). 깬 뒤 `poise_lock`초는 안 찬다,
    /// 맞지 않은 지 `poise.delay s`가 지나면 줄어든다.
    pub poise: f32,
    pub poise_lock: f32,
    pub poise_idle: f32,
    /// 자세가 깨진 동안 남은 초 — 그동안 받는 피해 × `poise.break damage x`(폭딜 창, 0055). 막대가 찬 비율(HUD, 매 틱).
    pub broken: f32,
    pub poise_frac: f32,
    /// 높낮이(0043): 곧게 못 가면 월드가 주는 다음 칸 가운데(거리 지도) — 그쪽으로 걷고 공격을 고르지 않는다.
    pub way: Option<Vec3>,
    /// 돌진(2026-11-06): 돌진을 시작한 뒤 몸이 플레이어에 닿기까지의 초(예고를 굳힐 때 잰다, 길 밖이면 돌진 끝까지).
    /// 띠 예고의 채움이 예고 끝이 아니라 **닿는 순간**에 다 찬다 — 거리마다 박자가 달라 헷갈리지 않게.
    pub charge_eta: f32,
    /// 돌진이 갈 수 있는 길이(m, 0076): 월드가 벽·기둥·턱까지 재어 준다(`nav::charge_room`). 기본은 끝까지(`charge_speed × charge_time`).
    /// 몸은 여기서 멈추고 예고 띠도 여기까지 — 띠가 벽을 뚫고 그려지는데 몸은 벽에서 멈추던 것.
    pub charge_len: f32,
    /// 이번 틱에 돌진 예고가 아직 방향을 겨누고 있었다(월드가 길이·닿는 때를 다시 재라고).
    pub charge_aim: bool,
    /// 떼 퍼지기(0076): 곁의 적에게서 비키는 쪽(월드가 매 틱 준다) — 한 줄로 늘어서지 않고 둘러싼다.
    pub spread: Vec3,
    /// 엘리트(0112): 성질 하나. 없으면 보통 개체.
    pub elite: Option<Affix>,
    /// 구역 등급(0112, 1부터 — 스테이지가 세울 때 `set_tier`).
    pub tier: u8,
    /// 심은 자리(꼬투리, 0120)·다시 깡충 물러나기까지 남은 초.
    pub plant_at: Option<Vec3>,
    pub hop_cd: f32,
    /// 청소자(0120 — 마을의 잔재, 0116 ECO-10): 보고도 덤비지 않고 제자리에서 먹는다(먹으면 체력이 찬다). 맞거나 곁의 동료가 싸우면 싸운다.
    pub docile: bool,
    pub eat_t: f32,
    /// 되풀이(0112): 이번 공격이 되풀이한 둘째 번인가.
    pub repeated: bool,
    /// 박자 변주(0128): 이번 공격의 박자, 연속 공격(켜졌나·다음 간격 차례), 페인트(바꿀 공격), 지연 소리를 냈나, 결정적 굴림, 변주 끔(시험).
    pub tempo: Tempo,
    pub chain_on: bool,
    pub chain_i: u8,
    pub feint: Option<Atk>,
    /// 2단계 전환을 시작한 체력(0133) — 전환 동안 이 아래로 깎이지 않는다.
    pub rise_hp: i32,
    /// 2단계 잇기(0133): 돌진 끝에 곧장 이은 공격을 이미 했다(한 번만, 다음에 새로 고를 때 풀린다).
    pub linked: bool,
    pub cued: bool,
    pub dice: u32,
    pub steady: bool,
    /// 지도에 서 있는 적의 제자리·바라보는 쪽(0113). 없으면(물결·시나리오) 처음부터 쫓는다.
    pub home: Option<(Vec3, f32)>,
    /// 턱을 뛰어오르거나 뛰어내리는 중(0100) — 그동안 몸은 포물선을 따라가고 생각하지도 치지도 않는다.
    pub hop: Option<Hop>,
}

/// 뛰기 한 번(0100): 어디서 어디로, 지난 초·걸리는 초·포물선 높이.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hop {
    pub from: Vec3,
    pub to: Vec3,
    pub t: f32,
    pub dur: f32,
    pub arc: f32,
}

impl Hop {
    /// 뛰어오르기는 조금 길고 높게(턱 위로 넘어가게), 뛰어내리기는 짧게.
    pub fn new(from: Vec3, to: Vec3) -> Hop {
        let up = (to.y - from.y).max(0.0);
        Hop {
            from,
            to,
            t: 0.0,
            dur: if up > 0.0 { 0.42 } else { 0.32 },
            arc: if up > 0.0 { 0.35 + up * 0.25 } else { 0.25 },
        }
    }

    /// 진행 `k`(0..1)의 자리: 옆으로는 곧게, 높이는 두 끝을 잇는 줄 + 포물선(가운데에서 `arc`), 오를 때는 일찍 올라선다.
    pub fn at(&self, k: f32) -> Vec3 {
        let k = k.clamp(0.0, 1.0);
        let up = self.to.y > self.from.y;
        // 오를 때는 높이가 먼저 오른다(턱 모서리에 발이 걸려 보이지 않게).
        let ky = if up {
            1.0 - (1.0 - k) * (1.0 - k)
        } else {
            k * k
        };
        let mut p = self.from.lerp(self.to, k);
        p.y = self.from.y + (self.to.y - self.from.y) * ky + self.arc * 4.0 * k * (1.0 - k);
        p
    }
}

/// 한 틱의 결과(월드가 피해·이벤트로 바꾼다).
#[derive(Default, Debug, PartialEq)]
pub struct Out {
    /// 지연 공격이 곧 풀린다(0128 — 앱이 소리를 낸다).
    pub release: bool,
    /// 플레이어를 쳤다.
    pub hit: bool,
    /// 이번 틱에 체력 반 아래로 깨어났다.
    pub roused: bool,
    /// 이번 틱에 던진 종 조각 수(0125 — 월드가 부채꼴로 날린다).
    pub shards: u8,
    /// 2단계 전환이 터졌다(0133 — 월드가 둘레의 방랑자를 밀어낸다).
    pub burst: bool,
}

impl Enemy {
    pub fn new(kind: Kind, pos: Vec3) -> Enemy {
        let d = species::def(kind);
        Enemy {
            kind,
            pos,
            prev: pos,
            facing: 0.0,
            hp: d.hp,
            max_hp: d.hp,
            boss: d.boss,
            radius: d.radius,
            height: d.height,
            state: EState::Chase,
            atk: Atk::Smash,
            atk_count: 0,
            roused: false,
            waking: false,
            landed: false,
            rings_hit: 0,
            hurt: 0.0,
            knock: Vec3::ZERO,
            hit_by: 0,
            spilling: false,
            sway: Vec3::ZERO,
            sway_t: 0.0,
            dead_t: 0.0,
            dropped: false,
            exposed: 0.0,
            poise: 0.0,
            poise_lock: 0.0,
            poise_idle: 0.0,
            broken: 0.0,
            poise_frac: 0.0,
            way: None,
            charge_eta: 0.0,
            charge_len: d.charge_speed * d.charge_time,
            charge_aim: false,
            spread: Vec3::ZERO,
            hop: None,
            elite: None,
            tier: 1,
            plant_at: None,
            hop_cd: 0.0,
            docile: false,
            eat_t: 0.0,
            repeated: false,
            tempo: Tempo::Normal,
            chain_on: false,
            chain_i: 0,
            feint: None,
            rise_hp: 0,
            linked: false,
            cued: false,
            dice: (pos.x.to_bits()
                ^ pos.z.to_bits().rotate_left(13)
                ^ (kind as u32).wrapping_mul(0x9E37_79B9))
                | 1,
            steady: false,
            home: None,
        }
    }

    /// 지도에 세운다(0113): 이 자리·이 쪽이 제자리, 알아차리기 전엔 제 일(`Idle`).
    pub fn place(&mut self, facing: f32) {
        self.home = Some((self.pos, facing));
        self.facing = facing;
        self.state = EState::Idle;
    }

    /// 알아차렸다(0113): 제자리·돌아가는 중이면 쫓기 시작. 이미 싸우는 중이면 그대로.
    pub fn notice(&mut self) -> bool {
        if matches!(self.state, EState::Idle | EState::Return) {
            self.state = EState::Chase;
            true
        } else {
            false
        }
    }

    /// 구역 등급(0112): 잡몹만 — 체력을 올림으로 키운다(자세·잔불은 등급을 보고 월드가 곱한다). 보스는 그대로.
    pub fn set_tier(&mut self, n: u8) {
        if self.boss {
            return;
        }
        let tier = n.max(1);
        if self.tier == tier {
            return;
        }
        self.tier = tier;
        // 늘 종 바탕에서 다시 센다(등급을 바꿔도 배율이 겹치지 않게, 엘리트면 엘리트 배율까지 — 엘리트 뒤에 불러도 같다).
        let mut hp = (self.def().hp as f32 * crate::tuning::tier(tier).hp).ceil();
        if self.elite.is_some() {
            hp = (hp * crate::tuning::ELITE_HP).ceil();
        }
        self.max_hp = hp as i32;
        self.hp = self.max_hp;
    }

    /// 엘리트로(0112): 체력 × `ELITE_HP`(올림), 몸 × `ELITE_SCALE`, 성질 하나. 보스는 엘리트가 되지 않는다.
    pub fn make_elite(&mut self, affix: Affix) {
        if self.boss {
            return;
        }
        use crate::tuning::{ELITE_HP, ELITE_SCALE};
        self.elite = Some(affix);
        self.max_hp = (self.max_hp as f32 * ELITE_HP).ceil() as i32;
        self.hp = self.max_hp;
        self.radius *= ELITE_SCALE;
        self.height *= ELITE_SCALE;
    }

    /// 공격이 끝난 뒤: 되풀이(0112)면 한 번 더 같은 공격을 예고부터, 아니면 쉰다.
    fn after_attack(&mut self) -> EState {
        // 연속 공격(0128): 같은 공격이 고르지 않은 간격으로 더(굴림 없이 — 묶음 안에서는 간격이 박자다).
        let gaps = self.rhythm().gaps;
        if self.chain_on && (self.chain_i as usize) < gaps.len() {
            let m = gaps[self.chain_i as usize];
            self.chain_i += 1;
            let atk = self.atk;
            self.begin(atk);
            self.tempo = Tempo::Chain(m);
            return self.state;
        }
        self.chain_on = false;
        // 2단계 잇기(0133): 돌진이 끝나면 쉬지 않고 곧장 한 수 더(문지기 = 앞 내려치기, 종지기 = 울림 고리) — 한 번만.
        if let Some(next) = self.link_after() {
            self.linked = true;
            self.start(next);
            return self.state;
        }
        if self.elite == Some(Affix::Repeating) && !self.repeated {
            self.repeated = true;
            let atk = self.atk;
            self.start(atk);
            self.state
        } else {
            self.repeated = false;
            EState::Recover { t: 0.0 }
        }
    }

    pub fn def(&self) -> &'static Def {
        species::def(self.kind)
    }

    /// 2단계(0133)에서 돌진 끝에 곧장 이을 공격(없으면 None — 1단계, 보스 아님, 이미 이었음).
    fn link_after(&self) -> Option<Atk> {
        if !(self.boss && self.roused) || self.linked || self.atk != Atk::Charge {
            return None;
        }
        match self.def().behavior {
            Behavior::Warden => Some(Atk::Slam),
            Behavior::Bellkeeper => Some(Atk::Toll),
            _ => None,
        }
    }

    /// 2단계 전환 중(0133) — 맞지 않는다(튕긴다), 자세도 안 쌓인다.
    pub fn rising(&self) -> bool {
        matches!(self.state, EState::Rise { .. })
    }

    pub fn alive(&self) -> bool {
        self.state != EState::Dead
    }

    /// 싸움에 끼어 있나 — 잠든 적(`Dormant`)은 몸은 있어도(부딪힘) 맞지도, 락온·조준·땅 돌리기에 잡히지도 않는다.
    pub fn active(&self) -> bool {
        !matches!(self.state, EState::Dead | EState::Dormant)
    }

    /// 락온에 잡히나(0149): 싸움에 낀 적 + 깨어나는 중인 잠든 적. 그냥 잠든 적은 아니다(싸울 것이 아직 아니다).
    pub fn lockable(&self) -> bool {
        self.active() || (self.waking && self.state == EState::Dormant)
    }

    pub fn facing_dir(&self) -> Vec3 {
        let (s, c) = self.facing.sin_cos();
        vec3(s, 0.0, c)
    }

    fn tele_mul(&self) -> f32 {
        if self.roused {
            ROUSED_TELE
        } else {
            1.0
        }
    }

    /// 패링으로 튕길 수 있는 공격인가(0025). 퍼지는 뿌리 파동은 못 받는다 — 피해야 한다(예고 고리 가운데 X, nail 0187).
    pub fn parryable(&self) -> bool {
        !matches!(
            self.atk,
            Atk::Wave | Atk::Plant | Atk::Toll | Atk::Roots | Atk::Hurl
        )
    }

    /// 이번 공격의 고리 겹 수(뿌리 파동 1, 울림 고리 `rings` — 체력 반 아래 +1).
    pub fn ring_count(&self) -> u8 {
        match self.atk {
            Atk::Toll => self.def().rings + self.roused as u8,
            _ => 1,
        }
    }

    /// 이번에 던질 종 조각 수(체력 반 아래 +2).
    pub fn shard_count(&self) -> u8 {
        self.def().shards + 2 * self.roused as u8
    }

    /// 앞을 막나(등딱지, 0120): `from` = 적에게서 친 쪽으로의 방향. 바라보는 쪽 `guard_arc` 안이고, 휘청·자세가 깨진 동안이 아니면.
    pub fn guards(&self, from: Vec3) -> bool {
        // 2단계 전환(0133) 동안은 어느 쪽에서 와도 튕긴다.
        if self.rising() {
            return true;
        }
        let d = self.def();
        d.guard_arc > 0.0
            && self.alive()
            && self.broken <= 0.0
            && !matches!(self.state, EState::Stagger { .. } | EState::Dormant)
            && self.facing_dir().dot(from.normalize_or_zero())
                >= (d.guard_arc * 0.5).to_radians().cos()
    }

    /// 예고를 그릴 자리: 심은 원판(꼬투리)·종이 떨어질 자리(종지기 2단계)면 그 자리, 아니면 몸.
    pub fn tele_at(&self) -> Option<Vec3> {
        matches!(self.atk, Atk::Plant | Atk::Hurl)
            .then_some(self.plant_at)
            .flatten()
    }

    /// 퍼지는 고리의 가운데(종 내던지기는 종이 떨어진 자리, 0133 — 아니면 몸).
    pub fn wave_center(&self) -> Vec3 {
        match self.atk {
            Atk::Hurl => self.plant_at.unwrap_or(self.pos),
            _ => self.pos,
        }
    }

    /// 퍼지는 고리가 닿는 끝(m).
    fn wave_reach(&self) -> f32 {
        match self.atk {
            Atk::Hurl => HURL_RING,
            _ => self.def().wave_radius,
        }
    }

    /// 뿌리 기둥 줄(0133)의 앞머리: 몸에서 바라보는 쪽으로 몇 m까지 솟았나(솟는 중일 때만).
    pub fn roots_front(&self) -> Option<f32> {
        match (self.state, self.atk) {
            (EState::Wave { t }, Atk::Roots) => {
                Some((self.radius + t * ROOTS_SPEED).min(self.charge_len))
            }
            _ => None,
        }
    }

    /// 지금 공격의 예고 길이(초) = 들어 올림 + 지연 버팀(0128).
    pub fn tele_dur(&self) -> f32 {
        self.lift_dur()
            + match self.tempo {
                Tempo::Delayed(h) => h,
                _ => 0.0,
            }
    }

    /// 들어 올려 정점에 닿기까지의 예고(지연 버팀 빼고) — 렌더가 준비 동작을 이 길이로 든다.
    pub fn lift_dur(&self) -> f32 {
        let base = self.base_tele();
        match self.tempo {
            Tempo::Fast => (base * crate::tuning::FAST_TELE)
                .max(crate::tuning::TELE_FLOOR)
                .min(base),
            Tempo::Chain(m) => (base * m).max(crate::tuning::TELE_FLOOR),
            _ => base,
        }
    }

    /// 종의 예고 길이(공격마다, 반 아래면 짧게).
    fn base_tele(&self) -> f32 {
        let d = self.def();
        self.tele_mul()
            * match self.atk {
                Atk::Smash | Atk::Slam => d.tele,
                Atk::Charge => d.charge_tele,
                Atk::Wave => d.wave_tele,
                Atk::Plant | Atk::Shards => d.tele,
                Atk::Toll | Atk::Roots | Atk::Hurl => d.wave_tele,
            }
    }

    fn recover_dur(&self) -> f32 {
        let d = self.def();
        d.recover * if self.roused { ROUSED_RECOVER } else { 1.0 }
    }

    /// 지금 공격의 예고 모양.
    pub fn tele_shape(&self) -> TeleShape {
        let d = self.def();
        match self.atk {
            Atk::Smash => TeleShape::Disc {
                r: d.reach + PLAYER_RADIUS,
            },
            Atk::Slam => TeleShape::Sector {
                r: d.reach + PLAYER_RADIUS,
                arc: d.arc,
            },
            // 띠 = 플레이어 가운데가 들어가면 맞는 곳(0076): 몸이 지나는 길 + 닿는 거리(몸 + 플레이어 + 여유) — 폭도 끝도.
            // 예전 띠는 폭이 여유만큼 좁고 끝이 짧아, 띠 밖에 섰는데 맞았다.
            Atk::Charge => TeleShape::Lane {
                len: self.charge_len + self.touch(),
                w: self.touch() * 2.0,
            },
            Atk::Wave => TeleShape::Ring { r: d.wave_radius },
            // 심은 원판 = 플레이어 가운데가 들어가면 맞는 곳.
            Atk::Plant => TeleShape::Disc { r: d.plant_r },
            Atk::Toll => TeleShape::Ring { r: d.wave_radius },
            // 조각이 나가는 부채꼴(되받아칠 수 있는 것 = 띠·부채꼴 모양, 0187 — 피할 것은 고리).
            Atk::Shards => TeleShape::Sector {
                r: crate::tuning::SHARD_SHOW,
                arc: d.arc,
            },
            // 줄지은 원(0133): 플레이어 가운데가 들어가면 맞는 곳 — 원끼리 겹치게(사이로 빠질 틈이 있어 보이지 않게).
            Atk::Roots => {
                let r = ROOTS_R + PLAYER_RADIUS;
                let step = r * 1.3;
                TeleShape::Row {
                    n: ((self.charge_len - self.radius) / step).ceil().max(1.0) as u8,
                    r,
                    step,
                    start: self.radius + r * 0.5,
                }
            }
            Atk::Hurl => TeleShape::Disc {
                r: HURL_R + PLAYER_RADIUS,
            },
        }
    }

    /// 예고 진행 0..1과 모양(없으면 None). 렌더가 바닥 예고를 채운다.
    /// 돌진은 예고 + 달려오는 동안까지 한 번에 차서 **몸이 닿는 순간** 다 찬다(띠는 남은 길만, 닿으면 사라짐).
    pub fn telegraph(&self) -> Option<(f32, TeleShape)> {
        let charge = self.atk == Atk::Charge;
        let whole = self.tele_dur() + if charge { self.charge_eta } else { 0.0 };
        let k = |e: f32| (e / whole.max(1e-3)).clamp(0.0, 1.0);
        match self.state {
            EState::Tele { t: e } => Some((k(e), self.tele_shape())),
            EState::Charge { t: e, .. } if !self.landed => {
                let d = self.def();
                let TeleShape::Lane { len, w } = self.tele_shape() else {
                    return None;
                };
                let left = len - (e * d.charge_speed).min(self.charge_len);
                (left > self.touch() * 0.5)
                    .then(|| (k(self.tele_dur() + e), TeleShape::Lane { len: left, w }))
            }
            _ => None,
        }
    }

    /// 돌진이 닿는 거리(몸 가운데 ↔ 플레이어 가운데).
    pub fn touch(&self) -> f32 {
        self.radius + PLAYER_RADIUS + CHARGE_TOUCH
    }

    /// 돌진이 지금 방향으로 나가면 몸이 플레이어에 닿기까지의 초(`charge_eta`). 길 밖이면 돌진 끝까지.
    fn eta_to(&self, player: Vec3) -> f32 {
        let d = self.def();
        let f = self.facing_dir();
        let to = vec3(player.x - self.pos.x, 0.0, player.z - self.pos.z);
        let along = to.dot(f);
        let side = (to - f * along).length();
        let touch = self.touch();
        let end = self.charge_len / d.charge_speed;
        if along <= 0.0 || side > touch {
            return end;
        }
        // 옆으로 비켜 서 있으면 몸 가장자리가 닿는 때(가운데 줄 위가 아니면 조금 늦다).
        let reach = (touch * touch - side * side).max(0.0).sqrt();
        ((along - reach).max(0.0) / d.charge_speed).min(end)
    }

    /// 월드가 잰 돌진 길이(`charge_len`)를 넣는다. 아직 겨누는 중(`charge_aim`)이면 닿는 때도 다시 잰다.
    pub fn set_charge_room(&mut self, room: f32, player: Vec3) {
        self.charge_len = room;
        if self.charge_aim {
            self.charge_eta = self.eta_to(player);
        }
    }

    /// 퍼지는 고리들의 지금 반지름(뿌리 파동 하나, 울림 고리 여럿 — 뒤 고리는 `TOLL_GAP`만큼 늦다). 아직 안 나온 고리는 없다.
    pub fn wave_fronts(&self) -> Vec<f32> {
        match self.state {
            EState::Wave { t } if self.atk != Atk::Roots => (0..self.ring_count())
                .map(|k| t * self.def().wave_speed - k as f32 * crate::tuning::TOLL_GAP)
                .filter(|&r| r > 0.0 && r <= self.wave_reach())
                .collect(),
            _ => Vec::new(),
        }
    }

    fn turn_toward(&mut self, dir: Vec3, rate: f32, dt: f32) {
        let want = dir.x.atan2(dir.z);
        let mut diff = want - self.facing;
        while diff > std::f32::consts::PI {
            diff -= std::f32::consts::TAU;
        }
        while diff < -std::f32::consts::PI {
            diff += std::f32::consts::TAU;
        }
        self.facing += diff.clamp(-rate * dt, rate * dt);
    }

    fn start(&mut self, atk: Atk) {
        self.begin(atk);
        self.chain_on = false;
        self.chain_i = 0;
        if !self.steady {
            self.roll_tempo();
        }
    }

    /// 공격 하나를 연다(박자는 보통 — 굴림은 `start`가).
    fn begin(&mut self, atk: Atk) {
        let d = self.def();
        self.atk = atk;
        self.atk_count += 1;
        self.landed = false;
        self.rings_hit = 0;
        // 돌진 길이 칸은 뿌리 기둥 줄(0133)의 길이로도 쓴다 — 월드가 벽·기둥 앞까지로 줄인다(`set_charge_room`).
        self.charge_len = if atk == Atk::Roots {
            ROOTS_LEN
        } else {
            d.charge_speed * d.charge_time
        };
        self.state = EState::Tele { t: 0.0 };
        self.tempo = Tempo::Normal;
        self.feint = None;
        self.cued = false;
    }

    /// 결정적 굴림 0..1(xorshift — 같은 자리·같은 순서면 같은 박자, 시험이 잰다).
    fn roll(&mut self) -> f32 {
        let mut x = self.dice;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.dice = x;
        (x >> 8) as f32 / (1u32 << 24) as f32
    }

    /// 이 적의 박자 변주(엘리트는 지연이 잦고 연속 공격을 한다, 0128 "보스·엘리트 많이, 잡몹 조금").
    pub fn rhythm(&self) -> species::Rhythm {
        let mut r = self.def().rhythm;
        if self.elite.is_some() {
            r.delay += crate::tuning::ELITE_DELAY_BONUS;
            if r.chain < crate::tuning::ELITE_CHAIN {
                r.chain = crate::tuning::ELITE_CHAIN;
                r.gaps = crate::tuning::ELITE_GAPS;
            }
        }
        r
    }

    /// 공격을 시작할 때 박자를 굴린다(0128): 페인트(반 아래 보스) → 지연·빠름 → 연속.
    fn roll_tempo(&mut self) {
        let r = self.rhythm();
        let swing = matches!(self.atk, Atk::Smash | Atk::Slam);
        if !(swing || self.atk == Atk::Shards) {
            return;
        }
        if swing && self.boss && self.roused && self.roll() < r.feint {
            self.feint = match self.def().behavior {
                Behavior::Warden => Some(Atk::Wave),
                Behavior::Bellkeeper => Some(Atk::Toll),
                _ => None,
            };
            if self.feint.is_some() {
                return;
            }
        }
        let x = self.roll();
        if x < r.delay {
            let long = self.roll() < 0.5;
            self.tempo = Tempo::Delayed(crate::tuning::DELAY_HOLDS[long as usize]);
        } else if x < r.delay + r.fast {
            self.tempo = Tempo::Fast;
        }
        if swing && !r.gaps.is_empty() && self.roll() < r.chain {
            self.chain_on = true;
        }
    }

    /// 쫓다가 어떤 공격을 고를까(행동 방식마다).
    fn choose(&self, dist: f32) -> Option<Atk> {
        let d = self.def();
        match d.behavior {
            Behavior::Melee => (dist <= d.reach - 0.3).then_some(Atk::Smash),
            Behavior::Charger => {
                if dist <= d.reach - 0.2 {
                    Some(Atk::Smash)
                } else if dist <= d.charge_range && dist > 2.5 {
                    Some(Atk::Charge)
                } else {
                    None
                }
            }
            // 깨진 종지기(0125): 가까우면 울림 고리, 멀면 조각과 돌진을 번갈아.
            Behavior::Bellkeeper => {
                // 2단계(0133): 떨어져 있으면 셋에 한 번 종 내던지기.
                if self.roused
                    && dist > d.reach
                    && dist <= d.charge_range
                    && self.atk_count % 3 == 1
                {
                    return Some(Atk::Hurl);
                }
                // 붙어 있으면 울림 고리와 종 휘두르기(앞 부채꼴 — 옆으로 비키면 빈다)를 번갈아: 고리만 이어지면 뛰기만 하게 된다(0125 헤드리스).
                if dist <= d.reach {
                    Some(if self.atk == Atk::Toll {
                        Atk::Slam
                    } else {
                        Atk::Toll
                    })
                } else if dist <= d.charge_range {
                    Some(if self.atk_count.is_multiple_of(2) {
                        Atk::Shards
                    } else {
                        Atk::Charge
                    })
                } else {
                    None
                }
            }
            // 심기(0120): 거리 안이면 심는다(물러나기는 월드가 길을 보고 뛴다 — `hop_cd`).
            Behavior::Planter => (dist <= d.plant_range).then_some(Atk::Plant),
            Behavior::Warden => {
                // 2단계(0133): 넷에 한 번 뿌리 기둥 줄 — 바로 붙어 있을 땐 아니다(내려치기가 낫다).
                if self.roused && self.atk_count % 4 == 1 && dist > 3.0 && dist <= ROOTS_LEN - 1.0 {
                    return Some(Atk::Roots);
                }
                if self.atk_count % 3 == 2 && dist <= d.wave_radius - 1.0 {
                    Some(Atk::Wave)
                } else if dist <= d.reach - 0.5 {
                    Some(Atk::Slam)
                } else if dist <= d.charge_range && dist > 5.0 {
                    Some(Atk::Charge)
                } else {
                    None
                }
            }
        }
    }

    /// 한 틱. `player` = 플레이어 자리.
    pub fn tick(&mut self, player: Vec3, dt: f32) -> Out {
        let mut out = Out::default();
        if !self.alive() {
            self.dead_t += dt;
            return out;
        }
        let d = self.def();
        self.hurt = (self.hurt - dt).max(0.0);
        self.hop_cd = (self.hop_cd - dt).max(0.0);
        self.exposed = (self.exposed - dt).max(0.0);
        self.sway_t += dt;
        // 넉백은 빠르게 줄어든다. 굳은 엘리트(0112)는 덜 밀린다.
        let budge = if self.elite == Some(Affix::Hardened) {
            crate::tuning::ELITE_HARD_KNOCK
        } else {
            1.0
        };
        self.pos += self.knock * budge * dt;
        self.knock *= (1.0 - KNOCK_DECAY * dt).max(0.0);
        if self.knock.length() < SLAM_SPEED {
            self.spilling = false;
        }
        // 2단계 전환(0133): 체력 반에서 한 번 — 하던 공격을 버리고 `RISE_TIME` 동안 무릎 꿇었다 터지며 일어선다.
        if d.boss && !self.roused && self.hp * 2 <= self.max_hp && self.state != EState::Dormant {
            self.roused = true;
            self.rise_hp = self.hp.max(1);
            self.state = EState::Rise { t: 0.0 };
            self.plant_at = None;
            self.feint = None;
            self.chain_on = false;
            self.knock = Vec3::ZERO;
        }
        let to = vec3(player.x - self.pos.x, 0.0, player.z - self.pos.z);
        let dist = to.length();
        let dir = if dist > 1e-4 { to / dist } else { Vec3::Z };
        let speed = d.speed * if self.roused { ROUSED_SPEED } else { 1.0 };
        self.charge_aim = false;
        // 떼 퍼지기(0076): 가는 쪽의 옆으로만 비킨다(뒤로 물러서지 않는다), 앞이 막혔으면 조금 늦춘다.
        let steer = |go: Vec3, spread: Vec3| -> (Vec3, f32) {
            let side = spread - go * spread.dot(go);
            let slow = if spread.dot(go) < 0.0 {
                (1.0 + spread.dot(go) * 0.5).max(0.5)
            } else {
                1.0
            };
            ((go + side * SPREAD_STEER).normalize_or(go), slow)
        };
        match self.state {
            EState::Dormant | EState::Dead => {}
            EState::Idle => {
                if let Some((_, face)) = self.home {
                    let (s, c) = face.sin_cos();
                    self.turn_toward(vec3(s, 0.0, c), 2.0, dt);
                }
                // 청소자(0120): 싸움이 없으면 먹는다 — `EAT_TIME`마다 체력 1(lore ecology 청소자 "잔해를 먹고 회복").
                if self.docile && self.hp < self.max_hp {
                    self.eat_t += dt;
                    if self.eat_t >= crate::tuning::EAT_TIME {
                        self.eat_t = 0.0;
                        self.hp += 1;
                    }
                }
            }
            EState::Return => {
                let Some((home, face)) = self.home else {
                    self.state = EState::Chase;
                    return out;
                };
                let goal = self.way.unwrap_or(home);
                let to = vec3(goal.x - self.pos.x, 0.0, goal.z - self.pos.z);
                let l = to.length();
                if l > 0.05 {
                    self.turn_toward(to / l, 6.0, dt);
                    self.pos += to / l * (speed * dt).min(l);
                }
                if vec3(home.x - self.pos.x, 0.0, home.z - self.pos.z).length() <= 0.4 {
                    // 제자리에 닿으면 가득 찬다(끌고 가서 조금씩 깎는 꼼수를 막는다 — 소울의 돌아가는 적).
                    self.state = EState::Idle;
                    self.facing = face;
                    self.hp = self.max_hp;
                    self.poise = 0.0;
                    self.repeated = false;
                }
            }
            EState::Chase => {
                if let Some(w) = self.way {
                    let to = vec3(w.x - self.pos.x, 0.0, w.z - self.pos.z);
                    let l = to.length();
                    if l > 0.05 {
                        let (go, slow) = steer(to / l, self.spread);
                        self.turn_toward(go, 6.0, dt);
                        self.pos += go * (speed * slow * dt).min(l);
                    }
                } else {
                    self.turn_toward(dir, 6.0, dt);
                    match self.choose(dist) {
                        Some(atk) => {
                            self.linked = false;
                            self.start(atk);
                            if matches!(atk, Atk::Plant | Atk::Hurl) {
                                self.plant_at = Some(vec3(player.x, self.pos.y, player.z));
                            }
                        }
                        None => {
                            let (go, slow) = steer(dir, self.spread);
                            self.pos += go * speed * slow * dt;
                        }
                    }
                }
            }
            EState::Tele { t: e } => {
                // 페인트(0128): 들어 올림의 `FEINT_AT`에서 끊고 다른 공격의 예고로(그 예고는 그 공격의 모양·소리).
                if let Some(next) = self.feint {
                    if e >= self.lift_dur() * crate::tuning::FEINT_AT {
                        self.begin(next);
                        return out;
                    }
                }
                let dur = self.tele_dur();
                // 지연이 풀리기 직전 소리(0128 "몸 + 소리") — 맞기 `RELEASE_LEAD`초 전, 한 번.
                if matches!(self.tempo, Tempo::Delayed(_))
                    && !self.cued
                    && dur - (e + dt) <= crate::tuning::RELEASE_LEAD
                {
                    self.cued = true;
                    out.release = true;
                }
                if e < dur - TELE_LOCK {
                    let rate = if self.atk == Atk::Charge { 4.0 } else { 2.0 };
                    self.turn_toward(dir, rate, dt);
                    // 방향이 굳기 전까지는 닿는 때를 다시 잰다(굳은 뒤엔 그대로 — 채움이 일정한 빠르기로).
                    if self.atk == Atk::Charge {
                        self.charge_aim = true;
                        self.charge_eta = self.eta_to(player);
                    }
                }
                self.state = if e + dt < dur {
                    EState::Tele { t: e + dt }
                } else {
                    match self.atk {
                        Atk::Smash | Atk::Slam | Atk::Plant | Atk::Shards | Atk::Hurl => {
                            EState::Swing { t: 0.0 }
                        }
                        Atk::Toll | Atk::Roots => EState::Wave { t: 0.0 },
                        Atk::Charge => EState::Charge {
                            t: 0.0,
                            dir: self.facing_dir(),
                        },
                        Atk::Wave => EState::Wave { t: 0.0 },
                    }
                };
            }
            EState::Swing { t: e } if self.atk == Atk::Shards => {
                // 종 조각(0125): 휘두르는 첫 틱에 던진다(탄은 월드가 날린다).
                if e == 0.0 {
                    out.shards = self.shard_count();
                }
                self.state = if e + dt >= 0.25 {
                    self.after_attack()
                } else {
                    EState::Swing { t: e + dt }
                };
            }
            EState::Swing { t: e } if self.atk == Atk::Hurl => {
                // 종이 떨어진다(0133): 그 순간 원판 안이면 맞고, 곧 떨어진 자리에서 고리 한 겹이 퍼진다.
                if e == 0.0 {
                    if let Some(at) = self.plant_at {
                        if vec2(player.x - at.x, player.z - at.z).length() <= HURL_R + PLAYER_RADIUS
                        {
                            out.hit = true;
                        }
                    }
                }
                self.state = if e + dt >= 0.2 {
                    EState::Wave { t: 0.0 }
                } else {
                    EState::Swing { t: e + dt }
                };
            }
            EState::Swing { t: e } if self.atk == Atk::Plant => {
                // 심은 원판이 터진다(0120): 그 순간 플레이어 가운데가 원판 안이면 맞는다.
                if e == 0.0 {
                    if let Some(at) = self.plant_at {
                        if vec2(player.x - at.x, player.z - at.z).length() <= d.plant_r {
                            out.hit = true;
                        }
                    }
                }
                self.state = if e + dt >= 0.2 {
                    self.plant_at = None;
                    self.after_attack()
                } else {
                    EState::Swing { t: e + dt }
                };
            }
            EState::Swing { t: e } => {
                if e == 0.0 {
                    let arc = if self.atk == Atk::Slam {
                        (d.arc * 0.5).to_radians().cos()
                    } else {
                        -1.0
                    };
                    if dist <= d.reach + PLAYER_RADIUS && dir.dot(self.facing_dir()) >= arc {
                        out.hit = true;
                    }
                }
                self.state = if e + dt >= 0.2 {
                    self.after_attack()
                } else {
                    EState::Swing { t: e + dt }
                };
            }
            EState::Charge { t: e, dir: cdir } => {
                // 벽·기둥·턱 앞(`charge_len`)에서 멈춘다. 닿았나는 움직인 뒤의 몸으로(마지막 자리도 맞는다 — 띠 끝과 같게).
                let went = (e * d.charge_speed).min(self.charge_len);
                let step = (d.charge_speed * dt).min(self.charge_len - went).max(0.0);
                let from = self.pos;
                self.pos += cdir * step;
                // 이번 틱에 몸이 쓸고 간 선분과 플레이어 사이 — 틱마다 점으로 재면 띠 가장자리에 빈틈이 생긴다.
                let to_p = vec3(player.x - from.x, 0.0, player.z - from.z);
                let k = to_p.dot(cdir).clamp(0.0, step);
                let near = (to_p - cdir * k).length();
                if !self.landed && near <= self.touch() {
                    self.landed = true;
                    out.hit = true;
                }
                let done = went + step >= self.charge_len - 1e-4;
                self.state = if e + dt >= d.charge_time || done {
                    self.after_attack()
                } else {
                    EState::Charge {
                        t: e + dt,
                        dir: cdir,
                    }
                };
            }
            EState::Wave { t: e } if self.atk == Atk::Roots => {
                // 뿌리 기둥 줄(0133): 앞머리가 지나는 자리의 방랑자를 한 번 친다 — 바라보는 쪽 줄 위(옆 `ROOTS_R` 안).
                let f = self.facing_dir();
                let to = vec3(player.x - self.pos.x, 0.0, player.z - self.pos.z);
                let along = to.dot(f);
                let side = (to - f * along).length();
                let front = (self.radius + (e + dt) * ROOTS_SPEED).min(self.charge_len);
                if self.rings_hit == 0
                    && along > 0.0
                    && side <= ROOTS_R + PLAYER_RADIUS
                    && along <= front
                    && along >= front - ROOTS_SPEED * 0.12 - 0.6
                {
                    self.rings_hit = 1;
                    out.hit = true;
                }
                self.state = if front >= self.charge_len {
                    EState::Recover { t: 0.0 }
                } else {
                    EState::Wave { t: e + dt }
                };
            }
            EState::Wave { t: e } => {
                // 고리마다 한 번(울림 고리는 여럿 — 뒤 고리는 `TOLL_GAP`만큼 늦다, 0125). 마지막 고리가 끝까지 가면 쉰다.
                let n = self.ring_count();
                let lead = (e + dt) * d.wave_speed;
                let reach = self.wave_reach();
                // 종 내던지기(0133)의 고리는 종이 떨어진 자리가 가운데.
                let c = self.wave_center();
                let from_c = vec2(player.x - c.x, player.z - c.z).length();
                for k in 0..n {
                    let front = lead - k as f32 * crate::tuning::TOLL_GAP;
                    if front > 0.0
                        && front <= reach
                        && self.rings_hit & (1 << k) == 0
                        && (from_c - front).abs() <= 0.7
                    {
                        self.rings_hit |= 1 << k;
                        out.hit = true;
                    }
                }
                let last = lead - (n.max(1) - 1) as f32 * crate::tuning::TOLL_GAP;
                self.state = if last >= reach {
                    if self.atk == Atk::Hurl {
                        self.plant_at = None;
                    }
                    EState::Recover { t: 0.0 }
                } else {
                    EState::Wave { t: e + dt }
                };
            }
            EState::Recover { t: e } => {
                self.state = if e + dt >= self.recover_dur() {
                    EState::Chase
                } else {
                    EState::Recover { t: e + dt }
                };
            }
            EState::Rise { t: e } => {
                // 2단계 전환(0133): 방랑자 쪽을 보며 무릎 꿇고 버틴다 — 체력은 시작한 값 아래로 깎이지 않는다. 터지는 틱에 한 번 밀쳐낸다.
                self.turn_toward(dir, 3.0, dt);
                self.hp = self.hp.max(self.rise_hp);
                if e < RISE_BURST && e + dt >= RISE_BURST {
                    out.burst = true;
                    out.roused = true;
                }
                self.state = if e + dt >= RISE_TIME {
                    EState::Chase
                } else {
                    EState::Rise { t: e + dt }
                };
            }
            EState::Stagger { t: e, dur } => {
                self.state = if e + dt >= dur {
                    EState::Chase
                } else {
                    EState::Stagger { t: e + dt, dur }
                };
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(en: &mut Enemy, player: Vec3, ticks: usize) -> usize {
        (0..ticks)
            .filter(|_| en.tick(player, 1.0 / 60.0).hit)
            .count()
    }

    /// 0125: 깨진 종지기 — 가까우면 울림 고리와 종 휘두르기를 번갈아, 멀면 조각과 돌진을 번갈아, 아주 멀면 쫓기만.
    #[test]
    fn the_bellkeeper_alternates_toll_and_swing_up_close_and_shards_and_charge_afar() {
        let mut en = Enemy::new(Kind::Bellkeeper, Vec3::ZERO);
        let near = en.def().reach - 0.5;
        let mut seen = Vec::new();
        for _ in 0..4 {
            let a = en.choose(near).unwrap();
            seen.push(a);
            en.start(a);
        }
        assert_eq!(seen, vec![Atk::Toll, Atk::Slam, Atk::Toll, Atk::Slam]);
        // 휘두르기는 앞 부채꼴(둘레 원판이 아니다) — 옆으로 비키면 빈다.
        assert!(matches!(en.tele_shape(), TeleShape::Sector { .. }) || en.atk != Atk::Slam);
        let far = en.def().charge_range - 1.0;
        let a = en.choose(far).unwrap();
        en.start(a);
        let b = en.choose(far).unwrap();
        assert!(
            matches!(
                (a, b),
                (Atk::Shards, Atk::Charge) | (Atk::Charge, Atk::Shards)
            ),
            "{a:?} {b:?}"
        );
        assert_eq!(en.choose(en.def().charge_range + 1.0), None);
    }

    /// 0128: 지연 = 들어 올림은 같고 버팀만 길다, 풀리기 `RELEASE_LEAD`초 전에 소리 한 번. 빠름 = 짧되 바닥 위.
    #[test]
    fn delayed_swings_hold_longer_and_cue_before_the_hit_and_fast_ones_keep_a_floor() {
        use crate::tuning::{DELAY_HOLDS, RELEASE_LEAD, TELE_FLOOR};
        let mut en = Enemy::new(Kind::Bellkeeper, Vec3::ZERO);
        en.steady = true;
        en.start(Atk::Slam);
        let normal = en.tele_dur();
        en.tempo = Tempo::Delayed(DELAY_HOLDS[1]);
        assert_eq!(en.lift_dur(), normal, "the lift is the same");
        assert!((en.tele_dur() - normal - DELAY_HOLDS[1]).abs() < 1e-5);
        let p = vec3(0.0, 0.0, -3.0);
        let (mut cue_at, mut hit_at, mut t) = (None, None, 0.0);
        for _ in 0..200 {
            let out = en.tick(p, 1.0 / 60.0);
            t += 1.0 / 60.0;
            if out.release {
                assert!(cue_at.is_none(), "one cue");
                cue_at = Some(t);
            }
            if matches!(en.state, EState::Swing { .. }) && hit_at.is_none() {
                hit_at = Some(t);
            }
        }
        let lead = hit_at.unwrap() - cue_at.unwrap();
        assert!(
            (lead - RELEASE_LEAD).abs() < 0.04,
            "cue {lead:.3}s before the hit"
        );
        en.tempo = Tempo::Fast;
        assert!(en.tele_dur() < normal && en.tele_dur() >= TELE_FLOOR - 1e-5);
    }

    /// 0128: 연속 공격 = 같은 공격이 표의 간격(예고 배율) 차례로 더, 다 치면 쉰다. 페인트 = 반 아래 보스가 들다가 다른 공격으로.
    #[test]
    fn chains_follow_the_gap_table_and_feints_switch_the_attack_mid_lift() {
        let mut en = Enemy::new(Kind::Warden, Vec3::ZERO);
        en.steady = true;
        en.start(Atk::Slam);
        en.chain_on = true;
        let p = vec3(0.0, 0.0, -2.0);
        let mut tempos = Vec::new();
        for _ in 0..600 {
            let was = en.atk_count;
            en.tick(p, 1.0 / 60.0);
            if en.atk_count != was {
                tempos.push(en.tempo);
            }
            if matches!(en.state, EState::Recover { .. }) {
                break;
            }
        }
        let gaps = en.def().rhythm.gaps;
        let want: Vec<Tempo> = gaps.iter().map(|&m| Tempo::Chain(m)).collect();
        assert_eq!(tempos, want, "hits after the first follow the uneven gaps");
        assert!(matches!(en.state, EState::Recover { .. }), "then rests");

        let mut b = Enemy::new(Kind::Bellkeeper, Vec3::ZERO);
        b.steady = true;
        b.roused = true;
        b.start(Atk::Slam);
        b.feint = Some(Atk::Toll);
        let lift = b.lift_dur();
        let mut switched_at = None;
        for i in 0..120 {
            b.tick(p, 1.0 / 60.0);
            if b.atk == Atk::Toll && switched_at.is_none() {
                switched_at = Some(i as f32 / 60.0);
            }
        }
        let at = switched_at.expect("feinted into the toll");
        assert!(
            (at - lift * crate::tuning::FEINT_AT).abs() < 0.05,
            "{at} vs {lift}"
        );
    }

    /// 0128 "보스·엘리트 많이, 잡몹 조금": 굴림의 몫이 표대로 — 잡몹은 지연만 가끔, 엘리트는 연속 공격도, 보스는 넷 다(페인트는 반 아래만).
    #[test]
    fn rhythm_odds_follow_the_table_mobs_little_bosses_much() {
        let sample = |kind: Kind, elite: bool, roused: bool| {
            let mut en = Enemy::new(kind, vec3(1.3, 0.0, 2.7));
            if elite {
                en.make_elite(Affix::Hardened);
            }
            en.roused = roused;
            let n = 2000;
            let (mut delay, mut fast, mut chain, mut feint) = (0, 0, 0, 0);
            for _ in 0..n {
                en.start(Atk::Slam);
                delay += matches!(en.tempo, Tempo::Delayed(_)) as i32;
                fast += (en.tempo == Tempo::Fast) as i32;
                chain += en.chain_on as i32;
                feint += en.feint.is_some() as i32;
            }
            let f = |c: i32| c as f32 / n as f32;
            (f(delay), f(fast), f(chain), f(feint))
        };
        let (d, f, c, x) = sample(Kind::Husk, false, false);
        assert!(
            (d - 0.25).abs() < 0.04 && f == 0.0 && c == 0.0 && x == 0.0,
            "husk {d} {f} {c} {x}"
        );
        let (d, _, c, _) = sample(Kind::Husk, true, false);
        assert!(
            d > 0.38 && c > 0.2,
            "elite husk delays more and chains: {d} {c}"
        );
        let (_, _, _, x) = sample(Kind::Bellkeeper, false, false);
        assert_eq!(x, 0.0, "no feints before half health");
        let (d, f, c, x) = sample(Kind::Bellkeeper, false, true);
        assert!(
            d > 0.1 && f > 0.05 && c > 0.2 && x > 0.25,
            "bellkeeper roused {d} {f} {c} {x}"
        );
        let mut steady = Enemy::new(Kind::Bellkeeper, Vec3::ZERO);
        steady.steady = true;
        steady.start(Atk::Slam);
        assert_eq!(steady.tempo, Tempo::Normal);
    }

    /// 0116 T3: 등급 배율은 한 번만 적용한다(재설정 호출로 체력이 누적 증가하지 않게).
    #[test]
    fn setting_the_same_region_tier_twice_does_not_stack_health() {
        let mut en = Enemy::new(Kind::Husk, Vec3::ZERO);
        let base = en.max_hp;
        en.set_tier(2);
        let tier_two = (base as f32 * crate::tuning::tier(2).hp).ceil() as i32;
        assert_eq!(en.max_hp, tier_two);
        en.hp -= 1;
        en.set_tier(2);
        assert_eq!(en.max_hp, tier_two);
        assert_eq!(
            en.hp,
            tier_two - 1,
            "reapplying the tier must not refill health"
        );
        // 등급을 바꾸면 종 바탕에서 다시 센다(겹치지 않는다), 엘리트 뒤에 불러도 같다.
        en.set_tier(1);
        assert_eq!(en.max_hp, base);
        let mut elite = Enemy::new(Kind::Husk, Vec3::ZERO);
        elite.set_tier(2);
        elite.make_elite(Affix::Hardened);
        let mut late = Enemy::new(Kind::Husk, Vec3::ZERO);
        late.make_elite(Affix::Hardened);
        late.set_tier(2);
        assert_eq!(elite.max_hp, late.max_hp, "order does not matter");
    }

    /// 0112: 엘리트 = 체력 × `ELITE_HP`(올림)·몸 × `ELITE_SCALE`·성질 하나. 보스는 엘리트가 되지 않는다.
    #[test]
    fn an_elite_has_more_life_a_bigger_body_and_one_affix() {
        use crate::tuning::{ELITE_HP, ELITE_SCALE};
        let mut en = Enemy::new(Kind::Husk, Vec3::ZERO);
        let (hp, r) = (en.max_hp, en.radius);
        en.make_elite(Affix::Hardened);
        assert_eq!(en.max_hp, (hp as f32 * ELITE_HP).ceil() as i32);
        assert_eq!(en.hp, en.max_hp);
        assert!((en.radius - r * ELITE_SCALE).abs() < 1e-5);
        assert_eq!(en.elite, Some(Affix::Hardened));
        let mut boss = Enemy::new(Kind::Warden, Vec3::ZERO);
        let bhp = boss.max_hp;
        boss.make_elite(Affix::Repeating);
        assert_eq!((boss.elite, boss.max_hp), (None, bhp));
    }

    /// 0112 되풀이: 한 번 친 뒤 쉬지 않고 같은 공격을 예고부터 한 번 더 — 그다음엔 쉰다. 두 번 다 맞힌다.
    #[test]
    fn a_repeating_elite_strikes_twice_then_rests() {
        let mut en = Enemy::new(Kind::Husk, vec3(0.0, 0.0, -1.0));
        en.make_elite(Affix::Repeating);
        let player = Vec3::ZERO;
        let mut teles = 0;
        let mut hits = 0;
        let mut was_tele = false;
        let mut rested = false;
        for _ in 0..240 {
            let out = en.tick(player, 1.0 / 60.0);
            hits += out.hit as usize;
            let tele = matches!(en.state, EState::Tele { .. });
            if tele && !was_tele {
                teles += 1;
            }
            was_tele = tele;
            if matches!(en.state, EState::Recover { .. }) {
                rested = true;
                break;
            }
        }
        assert!(rested);
        assert_eq!(
            (teles, hits),
            (2, 2),
            "two telegraphed blows before resting"
        );
        // 보통 개체는 한 번 치고 쉰다.
        let mut plain = Enemy::new(Kind::Husk, vec3(0.0, 0.0, -1.0));
        let mut hits = 0;
        for _ in 0..240 {
            hits += plain.tick(player, 1.0 / 60.0).hit as usize;
            if matches!(plain.state, EState::Recover { .. }) {
                break;
            }
        }
        assert_eq!(hits, 1);
    }

    /// 0112 굳은: 같은 밀어내기에 덜 밀린다.
    #[test]
    fn a_hardened_elite_budges_less() {
        let mut a = Enemy::new(Kind::Husk, Vec3::ZERO);
        let mut b = Enemy::new(Kind::Husk, Vec3::ZERO);
        b.make_elite(Affix::Hardened);
        for en in [&mut a, &mut b] {
            en.state = EState::Stagger { t: 0.0, dur: 1.0 };
            en.knock = vec3(8.0, 0.0, 0.0);
            for _ in 0..10 {
                en.tick(vec3(0.0, 0.0, 20.0), 1.0 / 60.0);
            }
        }
        assert!(b.pos.x < a.pos.x * 0.5, "{} vs {}", b.pos.x, a.pos.x);
    }

    #[test]
    fn a_husk_walks_up_telegraphs_a_disc_and_smashes_once() {
        let mut en = Enemy::new(Kind::Husk, vec3(0.0, 0.0, -5.0));
        let mut seen_tele = false;
        let mut hits = 0;
        for _ in 0..400 {
            if let Some((_, TeleShape::Disc { .. })) = en.telegraph() {
                seen_tele = true;
            }
            hits += en.tick(Vec3::ZERO, 1.0 / 60.0).hit as usize;
            if hits > 0 {
                break;
            }
        }
        assert!(seen_tele && hits == 1);
    }

    #[test]
    fn a_thornbeast_charges_in_a_straight_line_from_range_and_hits_once() {
        let mut en = Enemy::new(Kind::Thorn, vec3(0.0, 0.0, -6.0));
        en.tick(Vec3::ZERO, 1.0 / 60.0);
        assert_eq!(en.atk, Atk::Charge);
        assert!(matches!(en.telegraph(), Some((_, TeleShape::Lane { .. }))));
        let hits = run(&mut en, Vec3::ZERO, 120);
        assert_eq!(hits, 1, "one hit per charge");
        // 지나쳐 간다(곧게).
        assert!(en.pos.z > -1.0 && en.pos.x.abs() < 0.1, "{:?}", en.pos);
    }

    /// 2026-11-06: 돌진 띠의 채움은 예고 끝이 아니라 몸이 플레이어에 닿는 틱에 다 찬다 — 거리가 달라도.
    #[test]
    fn a_charge_lane_fills_up_exactly_when_the_body_reaches_you() {
        for far in [3.5_f32, 5.0, 6.5] {
            let mut en = Enemy::new(Kind::Thorn, vec3(0.0, 0.0, -far));
            let mut last_k = 0.0;
            let mut hit_k = None;
            for _ in 0..240 {
                let k = en.telegraph().map(|(k, _)| k);
                if let Some(k) = k {
                    assert!(
                        k + 1e-4 >= last_k,
                        "the fill never goes back: {k} < {last_k}"
                    );
                    last_k = k;
                }
                if en.tick(Vec3::ZERO, 1.0 / 60.0).hit {
                    hit_k = Some(last_k);
                    break;
                }
            }
            let k = hit_k.expect("the charge lands");
            assert!(k > 0.96, "{far} m: full when it lands, got {k}");
            assert!(en.telegraph().is_none(), "the lane goes away once it lands");
        }
    }

    #[test]
    fn the_warden_cycles_slam_and_wave_and_wakes_up_faster_at_half_hp() {
        let mut en = Enemy::new(Kind::Warden, vec3(0.0, 0.0, -2.0));
        let mut atks = Vec::new();
        for _ in 0..1200 {
            if let EState::Tele { t: 0.0 } = en.state {
                atks.push(en.atk);
            }
            en.tick(Vec3::ZERO, 1.0 / 60.0);
        }
        assert!(
            atks.contains(&Atk::Slam) && atks.contains(&Atk::Wave),
            "{atks:?}"
        );
        let calm = en.tele_dur();
        en.hp = en.max_hp / 2;
        en.tick(Vec3::ZERO, 1.0 / 60.0);
        // 2단계 전환(0133): 반에서 전환이 시작되고, 터지는 틱에 한 번 `roused`가 나간다.
        assert!(en.roused && en.rising());
        let mut bursts = 0;
        for _ in 0..(crate::tuning::RISE_TIME * 60.0) as usize + 5 {
            bursts += en.tick(Vec3::ZERO, 1.0 / 60.0).roused as u32;
        }
        assert_eq!(bursts, 1);
        assert!(!en.rising());
        assert!(en.tele_dur() < calm || en.atk != Atk::Slam);
    }

    /// 0133: 반에서 하던 공격을 버리고 전환 — 그동안 튕기고(사방), 체력이 시작한 값 아래로 안 깎이고, 터지는 틱에 한 번 밀쳐낸다.
    #[test]
    fn a_boss_at_half_hp_kneels_untouchable_then_bursts_once_and_rises() {
        for kind in [Kind::Warden, Kind::Bellkeeper] {
            let mut en = Enemy::new(kind, vec3(0.0, 0.0, -3.0));
            en.state = EState::Tele { t: 0.1 };
            en.hp = en.max_hp / 2;
            en.tick(Vec3::ZERO, 1.0 / 60.0);
            assert!(en.rising(), "{kind:?}: the attack is dropped");
            assert!(
                en.guards(Vec3::X) && en.guards(-Vec3::Z),
                "{kind:?}: deflects from any side"
            );
            let start = en.hp;
            let mut bursts = Vec::new();
            for i in 0..200 {
                en.hp -= 1; // 어디서 깎여도(넓은 타·되받아친 조각) 전환 동안은 되돌아온다
                let out = en.tick(Vec3::ZERO, 1.0 / 60.0);
                if out.burst {
                    bursts.push(i);
                }
                if en.rising() {
                    assert!(en.hp >= start, "{kind:?}: hp held during the rise");
                }
            }
            assert_eq!(bursts.len(), 1, "{kind:?}: one burst");
            let at = bursts[0] as f32 / 60.0;
            assert!(
                (at - crate::tuning::RISE_BURST).abs() < 0.05,
                "{kind:?}: burst at {at}"
            );
            assert!(!en.rising() && en.roused, "{kind:?}: phase two");
            // 잡몹은 전환하지 않는다.
            let mut mob = Enemy::new(Kind::Husk, Vec3::ZERO);
            mob.hp = 1;
            mob.tick(vec3(0.0, 0.0, 5.0), 1.0 / 60.0);
            assert!(!mob.rising());
        }
    }

    /// 0133 잇기: 2단계의 돌진이 끝나면 쉬지 않고 곧장 한 수(문지기 = 앞 내려치기, 종지기 = 울림) — 한 번만.
    #[test]
    fn phase_two_links_a_follow_up_straight_after_a_charge_once() {
        for (kind, next) in [(Kind::Warden, Atk::Slam), (Kind::Bellkeeper, Atk::Toll)] {
            let mut en = Enemy::new(kind, Vec3::ZERO);
            en.roused = true;
            en.atk = Atk::Charge;
            assert_eq!(en.after_attack(), EState::Tele { t: 0.0 }, "{kind:?}");
            assert_eq!(en.atk, next, "{kind:?}");
            // 이은 공격이 끝나면 쉰다(돌진이 아니니 더 잇지 않고, 이미 이었다).
            assert!(
                matches!(en.after_attack(), EState::Recover { .. }),
                "{kind:?}"
            );
            // 1단계에서는 잇지 않는다.
            let mut calm = Enemy::new(kind, Vec3::ZERO);
            calm.atk = Atk::Charge;
            assert!(
                matches!(calm.after_attack(), EState::Recover { .. }),
                "{kind:?}"
            );
        }
    }

    /// 0133 문지기 2단계: 뿌리 기둥 줄 — 바라보는 쪽 줄 위의 방랑자를 앞머리가 지날 때 한 번 친다, 옆으로 비키면 빈다. 받지 못한다.
    #[test]
    fn warden_roots_hit_along_its_line_once_and_miss_a_step_aside() {
        let run = |player: Vec3| -> u32 {
            let mut en = Enemy::new(Kind::Warden, Vec3::ZERO);
            en.roused = true;
            en.facing = 0.0; // +Z
            en.atk = Atk::Roots;
            en.charge_len = ROOTS_LEN;
            en.state = EState::Wave { t: 0.0 };
            let mut hits = 0;
            for _ in 0..120 {
                hits += en.tick(player, 1.0 / 60.0).hit as u32;
                if !matches!(en.state, EState::Wave { .. }) {
                    break;
                }
            }
            hits
        };
        assert_eq!(run(vec3(0.0, 0.0, 6.0)), 1, "on the line");
        assert_eq!(run(vec3(0.6, 0.0, 9.0)), 1, "near the line, far along");
        assert_eq!(run(vec3(2.5, 0.0, 6.0)), 0, "a step aside");
        assert_eq!(run(vec3(0.0, 0.0, -4.0)), 0, "behind");
        let mut en = Enemy::new(Kind::Warden, Vec3::ZERO);
        en.begin(Atk::Roots);
        assert!(!en.parryable());
        assert!(matches!(en.tele_shape(), TeleShape::Row { n, .. } if n >= 5));
        // 벽·기둥 앞에서 멈춘다(월드가 길이를 줄인다 — 돌진과 같은 칸): 줄도 짧고, 그 너머는 안 맞는다.
        en.set_charge_room(4.0, Vec3::ZERO);
        let TeleShape::Row { n, .. } = en.tele_shape() else {
            unreachable!()
        };
        assert!(n <= 3, "short row before a pillar: {n}");
        en.facing = 0.0;
        en.state = EState::Wave { t: 0.0 };
        let mut hits = 0;
        for _ in 0..120 {
            hits += en.tick(vec3(0.0, 0.0, 7.0), 1.0 / 60.0).hit as u32;
        }
        assert_eq!(hits, 0, "beyond the pillar");
        // 2단계에서만 고른다.
        let mut seen = false;
        let mut w = Enemy::new(Kind::Warden, vec3(0.0, 0.0, -7.0));
        w.roused = true;
        for _ in 0..3000 {
            if w.atk == Atk::Roots {
                seen = true;
                break;
            }
            w.tick(Vec3::ZERO, 1.0 / 60.0);
            w.pos = vec3(0.0, 0.0, -7.0);
        }
        assert!(seen, "phase two picks roots");
    }

    /// 0133 종지기 2단계: 종 내던지기 — 예고 첫 틱의 자리에 떨어지고(원판), 떨어진 자리가 가운데인 고리 한 겹.
    #[test]
    fn bellkeeper_hurls_the_bell_to_where_you_stood_then_rings_from_there() {
        let mut en = Enemy::new(Kind::Bellkeeper, Vec3::ZERO);
        en.roused = true;
        en.atk_count = 1; // 셋에 한 번
        let target = vec3(0.0, 0.0, 8.0);
        en.tick(target, 1.0 / 60.0);
        assert_eq!(en.atk, Atk::Hurl);
        assert_eq!(en.plant_at.map(|p| (p.x, p.z)), Some((0.0, 8.0)));
        assert_eq!(en.tele_at(), en.plant_at);
        // 자리를 떠나도 떨어지는 곳은 그대로 — 원판 밖이면 안 맞는다.
        let mut hit_disc = false;
        let away = vec3(5.0, 0.0, 8.0);
        while matches!(en.state, EState::Tele { .. }) {
            hit_disc |= en.tick(away, 1.0 / 60.0).hit;
        }
        hit_disc |= en.tick(away, 1.0 / 60.0).hit;
        assert!(!hit_disc, "stepped out of the disc");
        // 그다음 고리는 떨어진 자리에서 퍼진다: 거기서 5m = 고리가 닿는다(몸에서는 9m 넘게 떨어져 있어도).
        assert_eq!(en.wave_center(), target);
        let mut ring = false;
        for _ in 0..240 {
            ring |= en.tick(away, 1.0 / 60.0).hit;
            if matches!(en.state, EState::Recover { .. }) {
                break;
            }
        }
        assert!(ring, "the ring from the landing reaches");
        assert!(en.plant_at.is_none(), "cleared after");
        // 원판 안이면 떨어지는 순간 맞는다.
        let mut en = Enemy::new(Kind::Bellkeeper, Vec3::ZERO);
        en.roused = true;
        en.atk_count = 1;
        en.tick(target, 1.0 / 60.0);
        let mut hit = false;
        for _ in 0..120 {
            hit |= en.tick(target, 1.0 / 60.0).hit;
            if matches!(en.state, EState::Wave { .. }) {
                break;
            }
        }
        assert!(hit, "under the bell");
    }

    #[test]
    fn a_pod_plants_where_you_stood_and_only_hits_if_you_stay() {
        // 0120: 심는 순간의 플레이어 자리에 원판 — 끝 = 판정. 비켜 서면 맞지 않는다.
        for (stay, want) in [(true, 1), (false, 0)] {
            let mut en = Enemy::new(Kind::Pod, vec3(0.0, 0.0, -7.0));
            let mut hits = 0;
            let mut p = Vec3::ZERO;
            for _ in 0..240 {
                if en.plant_at.is_some() && !stay {
                    p = vec3(3.0, 0.0, 0.0);
                }
                hits += en.tick(p, 1.0 / 60.0).hit as usize;
                if en.state == (EState::Recover { t: 0.0 }) {
                    break;
                }
            }
            assert_eq!(hits, want, "stay {stay}");
            assert!(
                !en.parryable() || en.atk != Atk::Plant,
                "a planted disc is to be avoided, not parried"
            );
        }
        // 멀면 다가오기만 한다.
        let mut far = Enemy::new(Kind::Pod, vec3(0.0, 0.0, -20.0));
        assert_eq!(run(&mut far, Vec3::ZERO, 60), 0);
        assert!(far.plant_at.is_none() && far.pos.z > -20.0);
    }

    #[test]
    fn a_shellback_guards_its_front_only_and_not_while_broken() {
        let mut en = Enemy::new(Kind::Shell, vec3(0.0, 0.0, -1.5));
        en.facing = 0.0; // +z = 플레이어 쪽
        assert!(en.guards(vec3(0.0, 0.0, 1.0)), "front");
        assert!(en.guards(vec3(0.5, 0.0, 1.0)), "inside 120°");
        assert!(!en.guards(vec3(1.0, 0.0, 0.0)), "side");
        assert!(!en.guards(vec3(0.0, 0.0, -1.0)), "back");
        en.broken = 1.0;
        assert!(
            !en.guards(vec3(0.0, 0.0, 1.0)),
            "broken poise drops the guard"
        );
        assert!(
            !Enemy::new(Kind::Husk, Vec3::ZERO).guards(Vec3::Z),
            "husks never guard"
        );
    }

    #[test]
    fn a_dormant_warden_does_nothing() {
        let mut en = Enemy::new(Kind::Warden, vec3(0.0, 0.0, -2.0));
        en.state = EState::Dormant;
        assert_eq!(run(&mut en, Vec3::ZERO, 300), 0);
        assert_eq!(en.state, EState::Dormant);
    }
}
