//! 게임 규칙 하나. 카메라도 소리도 메뉴도 모른다 — `Actions`와 `Basis`(입력 기준)만 받고 `Event`를 내놓는다.
//! 틱은 단계 메서드 순서(nail 0169 R1): tick_lock → tick_player → apply_strike → apply_turn → tick_enemies → collide → cleanup.
use macroquad::prelude::*;

use crate::ground::Ground;
use crate::input::Actions;
use crate::tuning::{
    Tuning, BEAT_CUE_MIN, BEAT_CUE_SHOT_ARC, BEAT_CUE_SHOT_REACH, BEAT_WINDOW, BEAT_WINDOW_MIN,
    BLAST_SHOW, BOLT_HEIGHT, BOLT_RADIUS, BOMB_BOUNCE, BOMB_GRAVITY, BOMB_SHOT_REACH, BOMB_SKID,
    CAST_CHARGES, DASH_ATTACK_LATE, FLASK_CHARGES, GROUND_STEP, HEAVY_SWAY, HIT_DY, HIT_HITSTOP,
    HIT_SWAY, LOCK_HANDOFF, P, PERFECT_RELEASE, PLAYER_HP, PLAYER_RADIUS, RIPOSTE_WINDOW,
    RUN_SPEED_X, RUN_STAMINA, STAMINA_DENIED_SHOW, STUN_CHAIN_GRACE, STUN_CHAIN_MAX, TICK_DT,
};
use crate::tuning::{PARRY_CHAIN, PARRY_CHAIN_DECAY, PARRY_CHAIN_WIDEN_DEG};
use crate::weapons::{self, Rb, Shape};

/// 축 정렬 상자(기둥·벽). 충돌·카메라 당기기에 쓴다.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }
    /// 광선(`from`, 단위 `dir`)이 상자에 들어가는 거리. 안에서 시작하면 0.
    pub fn ray(&self, from: Vec3, dir: Vec3) -> Option<f32> {
        let mut tmin = 0.0f32;
        let mut tmax = f32::INFINITY;
        for i in 0..3 {
            let (o, d, lo, hi) = (from[i], dir[i], self.min[i], self.max[i]);
            if d.abs() < 1e-6 {
                if o < lo || o > hi {
                    return None;
                }
            } else {
                let (mut t1, mut t2) = ((lo - o) / d, (hi - o) / d);
                if t1 > t2 {
                    std::mem::swap(&mut t1, &mut t2);
                }
                tmin = tmin.max(t1);
                tmax = tmax.min(t2);
                if tmin > tmax {
                    return None;
                }
            }
        }
        Some(tmin)
    }
    /// XZ 평면의 원(`p`, `r`)을 상자 밖으로 밀어낸 자리.
    pub fn push_circle(&self, p: Vec3, r: f32) -> Vec3 {
        let cx = p.x.clamp(self.min.x, self.max.x);
        let cz = p.z.clamp(self.min.z, self.max.z);
        let d = vec2(p.x - cx, p.z - cz);
        let len = d.length();
        if len >= r {
            return p;
        }
        if len < 1e-5 {
            // 가운데에 들어갔다: 가장 가까운 면으로.
            let c = self.center();
            let h = self.size() * 0.5;
            let dx = h.x - (p.x - c.x).abs();
            let dz = h.z - (p.z - c.z).abs();
            return if dx < dz {
                vec3(c.x + (h.x + r) * (p.x - c.x).signum(), p.y, p.z)
            } else {
                vec3(p.x, p.y, c.z + (h.z + r) * (p.z - c.z).signum())
            };
        }
        let n = d / len;
        vec3(cx + n.x * r, p.y, cz + n.y * r)
    }
}

/// 입력 기준. 락온이면 스틱 앞 = 대상 쪽, 옆 = 대상 둘레. 카메라가 늦게 도착해도 기준은 즉시 바뀐다.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Basis {
    /// 쿼터뷰: 화면 위 = 카메라 요(도)가 가리키는 쪽. 요는 `tuning`의 값이라 월드가 직접 읽는다.
    Quarter {
        yaw: f32,
    },
    Lock(Vec3),
    /// 락온을 푼 뒤 스틱을 쥐고 있는 동안: 풀던 순간의 락온 앞(대상 쪽)을 그대로 쓴다(0010). 화면은 아직 숄더뷰에서
    /// 돌아오는 중이라 곧바로 쿼터뷰 기준으로 바꾸면 쥔 그대로의 스틱이 최대 170° 다른 쪽을 가리켰다.
    Frozen(Vec3),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    Swing,
    Hit {
        at: Vec3,
    },
    EnemyDown {
        at: Vec3,
    },
    PlayerHurt,
    LockOn,
    LockOff,
    /// 땅을 비틀었다(0012·0023). `quarters` = 카메라가 도는 90° 칸 수(+ = 요 +90°, 화면에서 땅이 시계 방향 —
    /// 0이면 캐릭터만 돌아선다, ±2면 180°). 앱이 카메라를 돌린다.
    Turn {
        quarters: i32,
    },
    /// 쏟아진 적이 벽·기둥·아레나 끝에 부딪혔다.
    Slam {
        at: Vec3,
    },
    /// 보스가 체력 반 아래로 깨어났다(핵이 밝아지고 빨라진다).
    Rouse {
        at: Vec3,
    },
    /// 잔불을 주웠다(0064 M1) — 조각 하나의 값.
    Ember {
        value: u32,
    },
    /// 패링 성공(0025): 적의 근접 공격을 튕겼다.
    Parry {
        at: Vec3,
    },
    /// 패링으로 드러난 적에게 치명타(0025).
    Crit {
        at: Vec3,
    },
    /// 내려찍기가 땅에 닿았다(0027).
    Plunge {
        at: Vec3,
    },
    /// 강공격이 다 모였다(0030) — 떼면 센 것이 나간다는 신호.
    Charged,
    /// 등잔을 삼켰다(0031) — 체력이 올랐다.
    Gulp {
        at: Vec3,
    },
    /// 등잔을 눌렀지만 못 마신다(가득이거나 0) — HUD가 횟수를 깜빡인다.
    FlaskDenied,
    /// 강타가 맞았다(0030).
    Smash {
        at: Vec3,
    },
    /// 마법 탄이 나갔다(0048).
    Cast {
        at: Vec3,
    },
    /// 칸이 모자라 마법을 못 쐈다(0048) — HUD 칸이 깜빡인다.
    CastDenied,
    /// 기력이 모자라 행동이 막혔다(0088) — 막대가 깜빡이며 떨리고, 머리 위 땀방울, "안 됨" 소리.
    StaminaDenied,
    /// 적의 자세가 깨졌다(0051) — 휘청 + 드러남.
    Break {
        at: Vec3,
    },
    /// 망치 강타의 금 간 땅이 여진으로 터졌다(0051).
    Aftershock {
        at: Vec3,
    },
    /// 수류탄이 터졌다(0051).
    Boom {
        at: Vec3,
    },
    /// 모으기 공격을 딱 맞춰 뗐다(0051 지팡이 → 0078 모든 무기의 강타·쇠뇌 모은 사격).
    Perfect {
        at: Vec3,
    },
    /// 수류탄을 던졌다(0051).
    Throw,
    /// 박자에 맞춘 평타가 나갔다 — 스태미나를 쓰지 않았다(2026-11-07).
    Beat,
}

/// 금 간 땅(0051, 망치 여진): `left`초 뒤 둘레를 친다.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Crack {
    pub pos: Vec3,
    pub left: f32,
    pub radius: f32,
    pub damage: i32,
    pub poise: f32,
}

/// 수류탄(0051): 포물선으로 날아 땅에서 튕기다 도화선이 다하면(또는 쇠뇌 볼트에 맞으면) 터진다.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bomb {
    pub pos: Vec3,
    pub prev: Vec3,
    pub vel: Vec3,
    pub fuse: f32,
    pub radius: f32,
    pub damage: i32,
    pub poise: f32,
}

/// 맞힌 자리의 불똥(그림만 — 렌더가 읽는다, 2026-11-06): 맞은 쪽으로 튀어 떨어지며 짧게 산다.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spark {
    pub pos: Vec3,
    pub vel: Vec3,
    pub age: f32,
    /// 강타·치명타의 큰 불똥.
    pub big: bool,
}

/// 불똥이 사는 시간(초).
pub const SPARK_LIFE: f32 = 0.22;

/// 맞힌 자리 `at`에서 맞은 쪽 `dir`로 퍼지는 불똥(같은 스윙 = 같은 무늬, 난수 없이).
pub fn sparks_at(at: Vec3, dir: Vec3, swing: u32, big: bool) -> Vec<Spark> {
    let n = if big { 9 } else { 6 };
    let side = vec3(-dir.z, 0.0, dir.x);
    (0..n)
        .map(|i| {
            let h = (swing
                .wrapping_mul(2654435761)
                .wrapping_add(i as u32 * 40503)
                >> 8)
                & 0xffff;
            let r = h as f32 / 65535.0;
            let spread = (i as f32 / (n - 1) as f32 - 0.5) * 1.6 + (r - 0.5) * 0.4;
            let speed = if big { 9.0 } else { 6.5 } * (0.7 + 0.6 * r);
            let v = (dir + side * spread).normalize_or_zero() * speed + Vec3::Y * (1.5 + 3.0 * r);
            Spark {
                pos: at,
                vel: v,
                age: 0.0,
                big,
            }
        })
        .collect()
}

/// 폭발 고리(그림만 — 렌더가 읽는다).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Blast {
    pub pos: Vec3,
    pub radius: f32,
    pub age: f32,
}

/// 미리 받아 두는 다음 행동(0057).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Next {
    Attack,
    Dodge,
    Parry,
    Jump,
    Cast,
    Flask,
}

/// 마법 탄(0048): 곧게 날아 처음 닿는 깨어 있는 적 하나를 치고 사라진다. 벽·기둥·높은 땅에 닿아도 사라진다.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bolt {
    pub pos: Vec3,
    pub prev: Vec3,
    /// 단위 방향(수평).
    pub dir: Vec3,
    /// 남은 거리(m).
    pub left: f32,
    /// 무기 표의 탄 모양(피해·자세 피해·꿰뚫기·휘청·색, 0051).
    pub def: weapons::Bolt,
    /// 이미 맞힌 적(꿰뚫는 탄, 번호 비트).
    pub hits: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PState {
    Idle,
    Dodge {
        t: f32,
        dir: Vec3,
    },
    /// 휘두르기. `dir` = 내딛는 쪽(누른 순간의 입력, 없으면 바라보는 쪽, 0018).
    Attack {
        t: f32,
        swing: u32,
        dir: Vec3,
        /// 콤보의 몇째 타(0051, 무기 표 `combo`).
        hit: u8,
    },
    /// 맞아서 경직(0018): 못 움직이고 못 친다. 밀려난 속도는 빠르게 준다.
    Hurt {
        t: f32,
        dur: f32,
    },
    /// 패링(0025): `parry.window s` 동안 앞쪽 근접 공격을 튕긴다. 헛치면 `parry.time s`까지 못 움직인다. 튕기면 바로 풀린다.
    Parry {
        t: f32,
    },
    /// 공중(0027): 뛰기 전 속도를 잇고 조금만 조종한다. `plunge`면 내려앉을 때 둘레를 친다.
    Jump {
        t: f32,
        plunge: bool,
    },
    /// 내려앉은 뒤 짧은 빈틈(`jump.land s`, 내려찍기면 `plunge.land s`).
    Land {
        t: f32,
        dur: f32,
    },
    /// 강공격 모으기(0030): 평타의 활성 구간이 끝날 때 아직 공격을 쥐고 있으면. 서서(돌아볼 수는 있다) `heavy.charge s`까지 모은다.
    /// 떼면 늘 강타(덜 모았으면 약하게). 회피·패링으로 끊는다.
    Charging {
        t: f32,
    },
    /// 등잔 마시기(0031): `flask.raise s` 동안 들고 그 순간 삼킨다(`gulped`), `flask.time s`에 끝. 느리게 걷는다.
    /// 삼키기 전에 회피·피격이면 끊기고 등잔은 그대로. 마시는 동안의 누름은 버린다(연타가 쌓이지 않게).
    Drink {
        t: f32,
        gulped: bool,
    },
    /// 강타. `full` = 다 모았다(피해 + 덤, 휘청 길게, 보스도 휘청).
    Heavy {
        t: f32,
        swing: u32,
        dir: Vec3,
        full: bool,
        /// 딱 맞춰 뗐다(지팡이, 0051).
        perfect: bool,
    },
    /// 원거리 강공격 모으기(0056): RB로 한 발 쏜 뒤에도 쥐고 있으면. `cast.charge s`에 다 모인다, 떼면 모은 탄(덜 모았으면 약하게).
    /// 느리게 걷고 겨누는 쪽으로 돈다. 회피로 끊는다(칸·볼트는 떼는 순간에만 쓴다).
    CastCharge {
        t: f32,
    },
    /// 수류탄 던지기(0051, 원거리 무기의 X): `throw.startup s`에 손을 떠난다.
    Throw {
        t: f32,
        to: Vec3,
        thrown: bool,
    },
    /// 마법(0048): `cast.startup s`에 탄이 나가고(`fired`) `cast.time s`에 끝. 쏘는 동안 느리게 걷고 몸은 쏘는 쪽에 고정,
    /// 나간 뒤에는 회피로 끊는다.
    Cast {
        t: f32,
        dir: Vec3,
        fired: bool,
    },
    /// 땅 돌리기: 발을 딛고 비튼다(`turn.plant s` 동안 못 움직인다 — 공짜가 아니다). 그동안 몸이 `from` → `to`(요)로 돈다(0023).
    Turn {
        t: f32,
        from: f32,
        to: f32,
    },
}

pub struct Player {
    pub pos: Vec3,
    pub prev: Vec3,
    /// 요(라디안). 0 = +z를 본다. `facing_dir()`로 벡터.
    pub facing: f32,
    pub vel: Vec3,
    pub state: PState,
    pub hp: i32,
    pub hurt: f32,
    pub dodge_cd: f32,
    /// 땅 돌리기 대기(초).
    pub turn_cd: f32,
    /// 스태미나(0018)와 마지막으로 쓴 뒤 지난 초, 모자라서 막혔을 때 HUD 막대를 깜빡이는 남은 초.
    pub stamina: f32,
    /// 최대 체력·등잔 횟수(0064 M3 — `World::stats`가 정한다, `apply_stats`).
    pub max_hp: i32,
    pub flask_max: i32,
    pub stamina_idle: f32,
    pub stamina_denied: f32,
    /// 이번 틱에 기력이 모자라 막혔다(0088 — 새로 막힌 때만, 받아 둔 누름이 다시 막혀도 한 번) → `Event::StaminaDenied`.
    pub denied_new: bool,
    /// 경직 이음(0018 → 0099): 이음 안에서 또 맞은 횟수(0 = 첫 경직) — 경직 길이가 반씩 준다 — 와 남은 이음 창(초,
    /// 경직 + `STUN_CHAIN_GRACE`, 맞을 때마다 새로). 창이 닫히면 다음 경직은 다시 처음 길이.
    pub stun_chain: i32,
    pub stun_window: f32,
    /// 마지막 쏟음(땅 돌리기가 멈춘 순간)에서 지난 초 — 렌더의 쟁반 테.
    pub since_spill: f32,
    /// 마지막 패링 성공에서 지난 초(렌더의 번쩍임).
    pub since_parry: f32,
    /// 이어 패링(0081 → 0086): 남은 창(초)과 이번 이음에서 더 튕긴 수(0 = 첫 튕김만). 창이 닫히면 0으로.
    pub parry_chain_left: f32,
    pub parry_chain_n: u32,
    /// 모으기 공격의 패링(0089, 사장 "모으기 공격에는 패링도 달려있어야 함. 리스크가 있는 공격이라 보상"): 모은 공격(강타·모은 사격)을
    /// 뗀 뒤 남은 패링 창(초, 시작 = `parry.window s`). 그 공격 동작 안에서만 열린다.
    pub release_guard: f32,
    /// 박자(2026-11-07): 마지막 평타가 시작된 뒤 흐른 시간, 그 타의 한 스윙 시간(= 그 무기의 박자, 0 = 없음),
    /// 이번 공격 누름이 박자 창 안이었나(누른 순간에 판정 — 받아 둔 이른 누름은 거짓).
    pub since_swing: f32,
    pub beat: f32,
    pub on_beat: bool,
    /// 마지막 평타의 타 번호(대시 = `weapons::DASH`) — 박자 부채꼴(0087)이 그 타의 모양·휘두르는 쪽을 끝까지 그린다.
    pub swing_hit: u8,
    /// 박자 맞춤 피드백(0076): 잇달아 박자에 맞춘 타 수(박자를 놓친 타·새 콤보면 0), 마지막 박자 타에서 지난 초,
    /// 지금 스윙이 박자 타인가(렌더가 금빛 고리·음표·금빛 휘두름, HUD가 스태미나 막대 반짝임, 앱이 오르는 종소리).
    pub beat_chain: u32,
    pub since_beat: f32,
    pub swing_beat: bool,
    /// 마지막 딱 맞춰 떼기(0078)에서 지난 초 — 렌더가 발밑에 크게 퍼지는 금빛 고리.
    pub since_perfect: f32,
    /// 달리는 중(0076): 회피가 끝날 때 회피 버튼을 쥐고 있었다 — 떼거나 스틱을 놓거나 스태미나가 다하거나 다른 동작을 하면 끝.
    pub running: bool,
    /// 마지막으로 무언가를 맞힌 스윙 번호(맞히면 그 스윙의 내딛기를 멈춘다, 2026-11-06).
    pub connected: u32,
    /// 대시 공격을 받는 남은 시간(회피를 끊었거나 막 끝난 뒤, 2026-11-06).
    pub dash_left: f32,
    /// 공격 버튼을 한 번 누른 뒤 놓지 않고 쥔 시간(초, 0030). 연타 중 휘두르기 도중에 누른 짧은 누름이 모으기로 새지 않게.
    pub attack_hold: f32,
    /// 남은 등잔(0031), 끝난 뒤 누름을 버리는 남은 초, 못 마셔 깜빡이는 남은 초(HUD).
    pub flask: i32,
    pub flask_lock: f32,
    pub flask_denied: f32,
    /// 땅에서 뜬 높이(m)와 위 속도(0027). 충돌·카메라는 땅 자리(`pos`, y = 0)를 쓰고 렌더만 띄운다.
    pub air: f32,
    pub vy: f32,
    pub swings: u32,
    /// 마법 칸(0048, 0..=`CAST_CHARGES`): 근접으로 맞히면 차고 쏘면 1 준다. 못 쏴서 깜빡이는 남은 초(HUD).
    pub mana: f32,
    pub cast_denied: f32,
    /// 든 무기(0051, `weapons.rs` 표).
    pub weapon: weapons::Id,
    /// RB를 한 번 누른 뒤 쥔 시간(0056, `attack_hold`와 같은 셈).
    pub cast_hold: f32,
    /// 미리 받아 둔 다음 행동과 남은 초(0057 — 후딜레이를 끊는 캔슬 시점에 나간다).
    pub pending: Option<Next>,
    pub pending_left: f32,
    /// 콤보: 다음 타 번호와 그 타를 이어 갈 남은 초.
    pub combo_next: u8,
    pub combo_left: f32,
    /// 원거리 무기: 탄창의 볼트, 장전 남은 초(0 = 장전 중 아님), 수류탄(소수 = 다시 차는 중).
    pub ammo: u8,
    pub reload: f32,
    pub grenades: f32,
}

impl Player {
    pub fn facing_dir(&self) -> Vec3 {
        let (s, c) = self.facing.sin_cos();
        vec3(s, 0.0, c)
    }
    /// 공중에 떠 있나.
    #[cfg(test)]
    pub fn airborne(&self) -> bool {
        matches!(self.state, PState::Jump { .. })
    }
    pub fn invulnerable(&self) -> bool {
        matches!(self.state, PState::Dodge { .. }) || self.hurt > 0.0
    }
}

pub use crate::enemy::{Atk, EState, Enemy, TeleShape};
use crate::species::Kind;

pub struct World {
    pub name: &'static str,
    pub player: Player,
    pub enemies: Vec<Enemy>,
    pub lock: Option<usize>,
    pub obstacles: Vec<Aabb>,
    /// 걸을 수 있는 높낮이(0043). None = 평평한 땅(y 0). 있으면 배우의 `pos.y` = 발밑 땅 높이(`collide`가 맞춘다),
    /// 플레이어의 몸 높이 = `pos.y + air`.
    pub ground: Option<Ground>,
    /// 적 길찾기 격자(0076) — 땅 + 상자가 덮은 칸. 상자가 바뀌면(문) 다시 만든다.
    nav: Option<crate::nav::Nav>,
    /// 거리 지도 캐시(0100): (플레이어 칸, 지도) — 플레이어가 칸을 옮기거나 길 격자를 다시 만들 때만 새로 잰다(다익스트라는 틱마다 돌리기엔 무겁다).
    flow: Option<((usize, usize), Vec<u16>)>,
    /// 아레나 반폭(정사각형, 원점 가운데).
    pub arena: f32,
    pub hitstop: f32,
    pub time: f32,
    pub kills: u32,
    /// 랩 값의 틱 복사본(step이 받은 Tuning에서).
    lock_range: f32,
    lock_lose: f32,
    /// 쿼터뷰 화면 위를 정하는 카메라 방위(도). 앱이 매 틱 카메라에서 넘긴다(0011 — 락온을 풀면 돌 수 있다). None = 랩 값.
    pub view_yaw: Option<f32>,
    /// 지난 틱의 락온 앞(수평 단위), 락온이 아니면 None — 풀리는 틱에 `hold`로 넘긴다.
    last_lock_fwd: Option<Vec3>,
    /// 풀린 뒤 쥔 스틱의 기준(앞, 경과 초). 스틱을 놓거나 `move.hold after unlock s`가 지나면 끝.
    hold: Option<(Vec3, f32)>,
    /// 이번 틱에 땅을 돌렸다(카메라 칸 수, 쏟는 쪽 `cw`) — `apply_turn`이 쏟는다.
    turn_now: Option<(i32, bool)>,
    /// 돌리는 중: (방향, 멈추기까지 남은 초). 0이 되는 틱에 쏟는다 — 히트스톱 중에도 줄어든다(카메라 시계와 맞춘다).
    spill_at: Option<(bool, f32)>,
    /// 이번 틱에 내려찍기가 땅에 닿았다 — `apply_strike`가 친다(0027).
    plunge_now: bool,
    /// 이번 틱에 강공격이 다 모였다(`Event::Charged`를 `apply_strike`가 낸다).
    charged_now: bool,
    /// 이번 틱에 상호작용을 눌렀고 손이 비어 있다(0032) — 스테이지가 읽는다(월드는 쉼터를 모른다).
    pub interact: bool,
    /// 지금 점프 버튼이 상호작용이 되는 자리인가(0048) — 스테이지가 매 틱 적는다(쉼터 곁이고 깨어 있는 적이 가까이 없다).
    pub interact_here: bool,
    /// 날고 있는 마법 탄(0048).
    pub bolts: Vec<Bolt>,
    /// 잔불(0064 M1): 가진 수(쓰러져도 잃지 않는다 — 앱이 기록에서 넣고 기록으로 옮긴다)와 바닥에 떨어진 조각.
    pub embers: u32,
    pub drops: Vec<crate::ember::Ember>,
    /// 합친 능력치(0064 M3 — 장신구·성장). 앱이 `apply_stats`로 넣는다.
    pub stats: crate::stats::Stats,
    /// 마지막으로 잔불을 주운 시각(`time`) — HUD가 수를 잠깐 키운다.
    pub ember_at: f32,
    /// 금 간 땅(망치 여진)·날고 있는 수류탄·폭발 고리(0051).
    pub cracks: Vec<Crack>,
    pub bombs: Vec<Bomb>,
    pub blasts: Vec<Blast>,
    /// 맞힌 자리의 불똥(2026-11-06).
    pub sparks: Vec<Spark>,
    /// 이번 틱에 손을 떠난 수류탄(던질 자리).
    throw_to: Option<Vec3>,
    /// 이번 틱에 쏜 쪽과 세기(0 = 보통, 1 = 덜 모음, 2 = 다 모음, 0056) — `tick_bolts`가 탄을 세운다.
    cast_fire: Option<(Vec3, u8)>,
    cast_denied_now: bool,
    /// 이번 틱에 박자에 맞춘 평타가 나갔다(`Event::Beat`).
    beat_now: bool,
    /// 이번 틱에 등잔을 삼켰다·못 마셨다(사건은 `apply_strike`가 낸다).
    gulp_now: bool,
    flask_denied_now: bool,
    /// 히트스톱 동안 들어온 눌림(공격·회피·락온·돌리기). 멈춤이 풀리는 틱에 낸다 — 버리면 멈춘 순간 누른 버튼이 사라진다.
    held: Actions,
}

impl World {
    pub fn new(name: &'static str, arena: f32) -> Self {
        World {
            name,
            player: Player {
                pos: Vec3::ZERO,
                prev: Vec3::ZERO,
                facing: std::f32::consts::PI, // -z(화면 위)를 본다
                vel: Vec3::ZERO,
                state: PState::Idle,
                hp: PLAYER_HP,
                hurt: 0.0,
                dodge_cd: 0.0,
                turn_cd: 0.0,
                stamina: crate::tuning::SPECS[crate::tuning::idx(P::StaminaMax)].def,
                max_hp: PLAYER_HP,
                flask_max: FLASK_CHARGES,
                stamina_idle: 0.0,
                stamina_denied: 0.0,
                denied_new: false,
                stun_chain: 0,
                stun_window: 0.0,
                since_spill: f32::MAX,
                since_parry: f32::MAX,
                parry_chain_left: 0.0,
                parry_chain_n: 0,
                release_guard: 0.0,
                dash_left: 0.0,
                connected: 0,
                since_swing: f32::MAX,
                swing_hit: 0,
                beat: 0.0,
                on_beat: false,
                beat_chain: 0,
                since_beat: f32::MAX,
                swing_beat: false,
                since_perfect: f32::MAX,
                running: false,
                attack_hold: 0.0,
                flask: FLASK_CHARGES,
                flask_lock: 0.0,
                flask_denied: 0.0,
                air: 0.0,
                vy: 0.0,
                swings: 0,
                mana: CAST_CHARGES,
                cast_denied: 0.0,
                weapon: weapons::Id::Sword,
                cast_hold: 0.0,
                pending: None,
                pending_left: 0.0,
                combo_next: 0,
                combo_left: 0.0,
                ammo: 0,
                reload: 0.0,
                grenades: 0.0,
            },
            enemies: Vec::new(),
            lock: None,
            obstacles: Vec::new(),
            ground: None,
            nav: None,
            flow: None,
            arena,
            hitstop: 0.0,
            time: 0.0,
            kills: 0,
            lock_range: 18.0,
            lock_lose: 26.0,
            view_yaw: None,
            last_lock_fwd: None,
            hold: None,
            turn_now: None,
            spill_at: None,
            held: Actions::default(),
            plunge_now: false,
            charged_now: false,
            interact: false,
            interact_here: false,
            bolts: Vec::new(),
            embers: 0,
            drops: Vec::new(),
            stats: crate::stats::Stats::default(),
            ember_at: f32::MIN,
            cracks: Vec::new(),
            bombs: Vec::new(),
            blasts: Vec::new(),
            sparks: Vec::new(),
            throw_to: None,
            cast_fire: None,
            cast_denied_now: false,
            beat_now: false,
            gulp_now: false,
            flask_denied_now: false,
        }
    }

    /// 락온할 깨어 있는 적이 락온 거리 안에 있나(0053 — 터치 락온 버튼을 띄울지).
    pub fn lock_candidate(&self) -> bool {
        self.enemies
            .iter()
            .any(|e| e.active() && e.pos.distance(self.player.pos) <= self.lock_range)
    }

    /// 캔슬(0057, KB `action-roguelite-for-solo-dev` "캔슬 창이 곧 무기 성격"): 새 누름(회피 > 패링 > 점프 > 공격 > 마법 > 등잔)을
    /// `attack.buffer s` 동안 받아 둔다. 지금 동작이 그 행동을 직접 받는 자리(서 있음, 모으는 중의 회피·패링, 공중의 공격 = 내려찍기,
    /// 삼키기 전 회피)면 그대로 넘기고, 아니면 동작의 **캔슬 시점**이 오고 그 행동을 지금 할 수 있을 때 후딜레이를 끊어(서 있음으로)
    /// 넘긴다. 캔슬 시점: 평타 = 활성 뒤 + 후딜 × 타의 `cancel`(다음 타는 `combo.link`도), 강타 = 활성 뒤 + 후딜 × 강타의 `cancel`,
    /// 회피 = `dodge.cancel x`, 쏘기·던지기 = 손을 떠난 뒤, 착지 = `land.cancel x`, 마시기 = 삼킨 뒤(등잔은 빼고). 패링 헛침·맞음·돌리기·공중은 못 끊는다.
    fn cancel_or_buffer(&mut self, a: &Actions, t: &Tuning, dt: f32) -> Actions {
        let wpn = self.player.weapon.def();
        let pressed = if a.dodge {
            Some(Next::Dodge)
        } else if a.parry {
            Some(Next::Parry)
        } else if a.jump {
            Some(Next::Jump)
        } else if a.attack {
            Some(Next::Attack)
        } else if a.cast {
            Some(Next::Cast)
        } else if a.flask {
            Some(Next::Flask)
        } else {
            None
        };
        let pl = &mut self.player;
        // 박자(2026-11-07): 공격을 누른 **그 순간**이 박자 창 안이었나. 받아 뒀다 나중에 나가는 이른 누름은 창 밖으로 친다
        // (연타가 박자를 공짜로 얻지 못하게 — KB melee-light-and-charged-attack-feel "먼저 누른 것이 남아야 창을 긁지 못한다").
        // 원거리 무기는 사격 버튼(RB)이 같은 박자를 탄다(사장 2026-11-07 "원거리 무기도 박자 있는거야. 근데 버튼이 다를뿐").
        let beat_press = if matches!(wpn.rb, Rb::Shot { .. }) {
            a.cast
        } else {
            a.attack
        };
        if beat_press {
            pl.on_beat = on_beat(pl.since_swing, pl.beat);
        }
        pl.pending_left = (pl.pending_left - dt).max(0.0);
        if pressed.is_some() {
            pl.pending = pressed;
            pl.pending_left = t[P::AttackBuffer];
        }
        if pl.pending_left <= 0.0 {
            pl.pending = None;
        }
        // 마시는 동안의 등잔 누름은 버린다(0031 — 연타가 쌓이지 않게).
        if matches!(pl.state, PState::Drink { .. }) && pl.pending == Some(Next::Flask) {
            pl.pending = None;
        }
        let mut out = Actions {
            attack: false,
            dodge: false,
            parry: false,
            jump: false,
            cast: false,
            flask: false,
            ..*a
        };
        let Some(n) = pl.pending else {
            return out;
        };
        let set = |out: &mut Actions| match n {
            Next::Attack => out.attack = true,
            Next::Dodge => out.dodge = true,
            Next::Parry => out.parry = true,
            Next::Jump => out.jump = true,
            Next::Cast => out.cast = true,
            Next::Flask => out.flask = true,
        };
        // 지금 동작이 직접 받는 누름.
        let own = match pl.state {
            PState::Idle => true,
            PState::Charging { .. } => matches!(n, Next::Dodge | Next::Parry),
            PState::CastCharge { .. } => n == Next::Dodge,
            PState::Drink { gulped: false, .. } => n == Next::Dodge,
            PState::Jump { plunge: false, .. } => n == Next::Attack,
            _ => false,
        };
        if own {
            // 직접 받는 자리면 한 번만 넘긴다(되든 막히든 — 막힌 누름을 계속 다시 넣으면 스태미나가 차는 순간 원치 않은 회피가 나갔다).
            set(&mut out);
            pl.pending = None;
            return out;
        }
        // 그 행동을 지금 할 수 있나(못 하면 끊지 않고 기다린다 — 끊고 아무것도 안 나가면 손해).
        let can = match n {
            Next::Dodge => pl.dodge_cd <= 0.0 && pl.stamina > 0.0,
            Next::Parry | Next::Jump => pl.stamina > 0.0,
            Next::Attack => {
                if wpn.grenade.is_some() {
                    pl.grenades >= 1.0
                } else {
                    pl.stamina > 0.0
                }
            }
            Next::Cast => match wpn.rb {
                Rb::Magic(_) => pl.mana >= 1.0,
                Rb::Shot { .. } => pl.ammo >= 1,
            },
            Next::Flask => pl.flask > 0 && pl.hp < pl.max_hp && pl.flask_lock <= 0.0,
        };
        let ready = match pl.state {
            PState::Attack { t: e, hit, .. } => {
                let (s0, a0, r0) = hit_times(t, wpn, hit);
                let k = wpn.hit(hit).map_or(0.0, |h| h.cancel);
                let base = s0 + a0 + r0 * k;
                let last = wpn.after(hit).is_none();
                if n == Next::Attack && !last {
                    e >= base.max(s0 + a0 + r0 * t[P::ComboLink])
                } else {
                    e >= base
                }
            }
            PState::Heavy { t: e, .. } => {
                let (s0, a0, r0) = heavy_times(t, wpn);
                e >= s0 + a0 + r0 * wpn.heavy.map_or(0.0, |h| h.cancel)
            }
            PState::Dodge { t: e, .. } => e >= t[P::DodgeTime] * t[P::DodgeCancel],
            PState::Cast { fired, .. } => fired,
            PState::Throw { thrown, .. } => thrown,
            PState::Land { t: e, dur } => e >= dur * t[P::LandCancel],
            PState::Drink { gulped, .. } => gulped,
            _ => false,
        };
        if ready && can {
            if let PState::Attack { hit, .. } = pl.state {
                // 다음 타로 잇는다(마지막 타 뒤면 처음부터).
                pl.combo_next = wpn.after(hit).unwrap_or(0);
                pl.combo_left = if wpn.after(hit).is_some() {
                    t[P::ComboKeep]
                } else {
                    0.0
                };
            }
            // 회피를 끊고 나가는 공격 = 대시 공격(2026-11-06).
            if matches!(pl.state, PState::Dodge { .. }) && n == Next::Attack {
                pl.dash_left = DASH_ATTACK_LATE;
            }
            if matches!(pl.state, PState::Drink { .. }) {
                pl.flask_lock = t[P::FlaskLock];
            }
            pl.state = PState::Idle;
            set(&mut out);
        }
        out
    }

    /// 무기를 바꾼다(0051): 콤보는 처음부터, 원거리 무기는 탄창·수류탄 가득.
    pub fn set_weapon(&mut self, id: weapons::Id) {
        let pl = &mut self.player;
        pl.weapon = id;
        pl.combo_next = 0;
        pl.combo_left = 0.0;
        // 박자도 새로(0087 — 앞 무기의 박자 부채꼴이 남지 않게).
        pl.beat = 0.0;
        pl.reload = 0.0;
        let w = id.def();
        pl.ammo = match w.rb {
            Rb::Shot { mag, .. } => mag,
            Rb::Magic(_) => 0,
        };
        pl.grenades = w.grenade.map_or(0.0, |g| g.count as f32);
        if !matches!(pl.state, PState::Hurt { .. } | PState::Jump { .. }) {
            pl.state = PState::Idle;
        }
    }

    /// 종류 표(`species.rs`)의 적 하나를 세운다.
    /// 능력치를 바꾼다(0064 M3 — 장신구를 갈아 끼우거나 성장했을 때). 체력·등잔은 새 최대를 넘지 않게, `refill`이면 가득.
    pub fn apply_stats(&mut self, s: crate::stats::Stats, refill: bool) {
        self.stats = s;
        let pl = &mut self.player;
        pl.max_hp = s.max_hp;
        pl.flask_max = s.flask;
        if refill {
            pl.hp = s.max_hp;
            pl.flask = s.flask;
        } else {
            pl.hp = pl.hp.min(s.max_hp);
            pl.flask = pl.flask.min(s.flask);
        }
    }

    pub fn spawn(&mut self, kind: Kind, pos: Vec3) -> usize {
        self.enemies.push(Enemy::new(kind, pos));
        self.enemies.len() - 1
    }

    /// 잡몹(뿌리의 잔재) — 개발 시나리오·시험용.
    #[cfg(any(test, feature = "dev"))]
    pub fn spawn_mob(&mut self, pos: Vec3) {
        self.spawn(Kind::Husk, pos);
    }

    /// 보스(문지기) — 개발 시나리오·시험용.
    #[cfg(any(test, feature = "dev"))]
    pub fn spawn_boss(&mut self, pos: Vec3) {
        self.spawn(Kind::Warden, pos);
    }

    pub fn lock_target(&self) -> Option<Vec3> {
        self.lock
            .and_then(|i| self.enemies.get(i))
            .filter(|e| e.alive())
            .map(|e| e.pos)
    }

    /// 땅 돌리기 준비 0..1(1 = 쓸 수 있음). 락온 중엔 0 — 숄더뷰에서는 돌리지 않는다(0012).
    pub fn turn_ready(&self, t: &Tuning) -> f32 {
        if self.lock_target().is_some() {
            return 0.0;
        }
        let cd = t[P::TurnCooldown];
        if cd <= 0.0 {
            1.0
        } else {
            1.0 - (self.player.turn_cd / cd).clamp(0.0, 1.0)
        }
    }

    /// 내려찍기(0027): 내려앉은 자리 둘레 `plunge.range m` 안의 적을 친다 — 드러난 적이면 치명타도.
    fn plunge(&mut self, t: &Tuning, ev: &mut Vec<Event>) {
        let p = self.player.pos;
        self.player.swings += 1;
        let swing = self.player.swings;
        let mut hit = false;
        for en in &mut self.enemies {
            if !en.active() || (en.pos.y - p.y).abs() > HIT_DY {
                continue;
            }
            let d = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z);
            if d.length() - en.radius > t[P::PlungeRange] {
                continue;
            }
            let crit = en.exposed > 0.0;
            en.hp -= hurt_by(en, self.stats.dmg + t[P::PlungeDamage] as i32, crit, t);
            en.exposed = 0.0;
            en.hit_by = swing;
            // 근접으로 맞히면 마법 칸이 찬다(0048).
            self.player.mana = (self.player.mana + t[P::CastPerHit]).min(CAST_CHARGES);
            add_poise(en, 2.0, None, t, ev);
            en.hurt = 0.15;
            en.knock = d.normalize_or_zero() * t[P::Knockback] * if en.boss { 0.25 } else { 1.2 };
            if en.hp <= 0 {
                en.state = EState::Dead;
                self.kills += 1;
                ev.push(Event::EnemyDown { at: en.pos });
            }
            let at = en.pos + Vec3::Y * en.height * 0.6;
            ev.push(if crit {
                Event::Crit { at }
            } else {
                Event::Hit { at }
            });
            hit = true;
        }
        ev.push(Event::Plunge { at: p });
        if hit {
            self.hitstop = self.hitstop.max(t[P::Hitstop] * 1.5);
        }
    }

    /// 패링 창이 열려 있나(0025).
    /// 패링 창이 열려 있나: 패링 동작의 창 안이거나, 막 튕긴 뒤 이어 패링 시간 안(0081).
    pub fn parry_open(&self, t: &Tuning) -> bool {
        let pl = &self.player;
        matches!(pl.state, PState::Parry { t: e } if e < t[P::ParryWindow])
            || pl.parry_chain_left > 0.0
            || (pl.release_guard > 0.0
                && matches!(pl.state, PState::Heavy { .. } | PState::Cast { .. }))
    }

    #[cfg(test)]
    pub fn basis(&self) -> Basis {
        match self.lock_target() {
            Some(p) => Basis::Lock(p),
            None => Basis::Quarter { yaw: 0.0 },
        }
    }

    /// 한 틱. 히트스톱 중엔 로직 시계만 멈춘다.
    pub fn step(&mut self, a: &Actions, t: &Tuning) -> Vec<Event> {
        // 장신구·성장이 휜 랩 값(0064 M3) — 이 틱의 로직은 이것만 읽는다.
        let tuned = self.stats.tune(t);
        let t = &tuned;
        let mut ev = Vec::new();
        let dt = TICK_DT;
        if let Some((_, left)) = &mut self.spill_at {
            *left -= dt;
        }
        if self.hitstop > 0.0 {
            self.hitstop -= dt;
            self.held = Actions {
                move_dir: a.move_dir,
                attack: self.held.attack | a.attack,
                dodge: self.held.dodge | a.dodge,
                parry: self.held.parry | a.parry,
                jump: self.held.jump | a.jump,
                cast: self.held.cast | a.cast,
                lock: self.held.lock | a.lock,
                turn_right: self.held.turn_right | a.turn_right,
                turn_left: self.held.turn_left | a.turn_left,
                ..Actions::default()
            };
            // 보간이 미끄러지지 않게 prev를 지금으로.
            self.player.prev = self.player.pos;
            for e in &mut self.enemies {
                e.prev = e.pos;
            }
            return ev;
        }
        let held = std::mem::take(&mut self.held);
        let merged = Actions {
            attack: a.attack | held.attack,
            dodge: a.dodge | held.dodge,
            parry: a.parry | held.parry,
            jump: a.jump | held.jump,
            cast: a.cast | held.cast,
            lock: a.lock | held.lock,
            turn_right: a.turn_right | held.turn_right,
            turn_left: a.turn_left | held.turn_left,
            ..*a
        };
        let idle = matches!(self.player.state, PState::Idle);
        // 점프 버튼이 상호작용도 한다(0048): 쉼터 곁이고 안전하면 쉬기, 아니면 점프.
        let via_jump = merged.jump && self.interact_here && idle;
        self.interact = (merged.interact || via_jump) && idle;
        let merged = Actions {
            jump: merged.jump && !via_jump,
            ..merged
        };
        let a = &merged;
        self.time += dt;
        self.player.prev = self.player.pos;
        for e in &mut self.enemies {
            e.prev = e.pos;
        }
        self.tick_lock(a, &mut ev);
        self.tick_player(a, t, dt);
        self.apply_strike(t, &mut ev);
        self.tick_bolts(t, dt, &mut ev);
        self.apply_turn(t, &mut ev);
        self.tick_enemies(t, dt, &mut ev);
        self.collide(t, &mut ev);
        self.cleanup(&mut ev);
        self.tick_embers(t, dt, &mut ev);
        ev
    }

    /// 잔불(0064 M1): 막 쓰러진 적은 제 값을 흩고, 바닥의 조각은 튀고·끌리고·주워진다.
    fn tick_embers(&mut self, t: &Tuning, dt: f32, ev: &mut Vec<Event>) {
        for (i, en) in self.enemies.iter_mut().enumerate() {
            if !en.alive() && !en.dropped {
                en.dropped = true;
                let seed = (self.time * 1000.0) as u32 ^ (i as u32).wrapping_mul(2654435761);
                // 잔불 주머니(0064 M3): 값에 배.
                let value = (en.def().embers as f32 * self.stats.ember).round() as u32;
                self.drops
                    .extend(crate::ember::scatter(en.pos, value, seed));
                // 굶주린 뿌리: 쓰러질 때마다 스태미나.
                if self.stats.kill_stamina > 0.0 {
                    let pl = &mut self.player;
                    pl.stamina = (pl.stamina + self.stats.kill_stamina).min(t[P::StaminaMax]);
                }
            }
        }
        let to = self.player.pos + Vec3::Y * 0.9;
        let ground = self.ground.as_ref();
        let mut got = 0;
        self.drops.retain_mut(|e| {
            let g = ground.map_or(0.0, |g| g.height(e.pos));
            if crate::ember::step(e, to, g, dt) {
                got += e.value;
                ev.push(Event::Ember { value: e.value });
                false
            } else {
                true
            }
        });
        if got > 0 {
            self.embers += got;
            self.ember_at = self.time;
        }
    }

    fn tick_lock(&mut self, a: &Actions, ev: &mut Vec<Event>) {
        // 대상이 쓰러지면 조준 기준(락온 중에는 그 대상 자리) 가까이의 다음 적으로 넘어간다 — 둘레 `LOCK_HANDOFF` 안에
        // 아무도 없으면 풀린다(사장 2026-11-06, 소울류의 "다음 적으로 넘기기"). 사라진 대상은 버튼 없이 풀린다.
        if let Some(i) = self.lock {
            match self.enemies.get(i) {
                Some(e) if e.alive() => {}
                dead => {
                    let at = dead.map(|e| e.pos);
                    self.lock = at.and_then(|at| self.handoff_target(at));
                    if self.lock.is_none() {
                        ev.push(Event::LockOff);
                    }
                }
            }
        }
        if !a.lock {
            return;
        }
        if self.lock.is_some() {
            self.lock = None;
            ev.push(Event::LockOff);
            return;
        }
        // 가까움 + 바라보는 쪽 + 보스 우대. 점수가 낮을수록 좋다.
        let p = self.player.pos;
        let f = self.player.facing_dir();
        let mut best: Option<(f32, usize)> = None;
        for (i, e) in self.enemies.iter().enumerate() {
            if !e.active() {
                continue;
            }
            let d = e.pos - p;
            let dist = d.length();
            if dist > self.range_lock() {
                continue;
            }
            let cos = if dist > 1e-4 { d.dot(f) / dist } else { 1.0 };
            let score =
                dist / self.range_lock() + (1.0 - cos) * 0.5 - if e.boss { 0.5 } else { 0.0 };
            if best.is_none_or(|(s, _)| score < s) {
                best = Some((score, i));
            }
        }
        if let Some((_, i)) = best {
            self.lock = Some(i);
            ev.push(Event::LockOn);
        }
    }

    /// 쓰러진 락온 대상 자리 `at`에서 가장 가까운 깨어 있는 적(둘레 `LOCK_HANDOFF` 안, 플레이어에게서 락온 거리 안).
    fn handoff_target(&self, at: Vec3) -> Option<usize> {
        let p = self.player.pos;
        self.enemies
            .iter()
            .enumerate()
            .filter(|(_, e)| e.active() && e.pos.distance(p) <= self.range_lock())
            .map(|(i, e)| (vec3(e.pos.x - at.x, 0.0, e.pos.z - at.z).length(), i))
            .filter(|(d, _)| *d <= LOCK_HANDOFF)
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, i)| i)
    }

    fn range_lock(&self) -> f32 {
        // 틱 안에서 Tuning을 들고 다니지 않으려고 여기만 상수로 읽는다 — 랩 값은 step()에서 받은 t.
        self.lock_range
    }
}

impl World {
    fn tick_player(&mut self, a: &Actions, t: &Tuning, dt: f32) {
        // 캔슬(0057): 누름은 미리 받아 두고, 지금 동작의 캔슬 시점이 오면 후딜레이를 끊고 그 행동으로 넘어간다.
        let cancelled = self.cancel_or_buffer(a, t, dt);
        let a = &cancelled;
        let started_from = std::mem::discriminant(&self.player.state);
        self.lock_range = t[P::LockRange];
        self.lock_lose = t[P::LockLose];
        // 입력 기준: 쿼터뷰는 화면(= 월드 고정 요), 락온은 대상 기준, 락온을 푼 뒤 쥔 스틱은 풀던 순간 그대로(0010).
        let target = self
            .lock
            .and_then(|i| self.enemies.get(i))
            .filter(|e| e.alive())
            .map(|e| e.pos);
        let stick_on = a.move_dir.length() > HOLD_STICK;
        let basis = match target {
            Some(tg) => {
                let d = vec3(tg.x - self.player.pos.x, 0.0, tg.z - self.player.pos.z);
                self.last_lock_fwd = (d.length_squared() > 1e-4).then(|| d.normalize());
                self.hold = None;
                Basis::Lock(tg)
            }
            None => {
                if let Some(f) = self.last_lock_fwd.take() {
                    if stick_on {
                        self.hold = Some((f, 0.0));
                    }
                }
                match self.hold {
                    Some((f, held)) if stick_on && held < t[P::MoveHoldMax] => {
                        self.hold = Some((f, held + dt));
                        Basis::Frozen(f)
                    }
                    _ => {
                        self.hold = None;
                        Basis::Quarter {
                            yaw: self.view_yaw.unwrap_or(t[P::CamQYaw]),
                        }
                    }
                }
            }
        };
        // 조준(오른쪽 스틱, 0014): 이동과 같은 기준으로 월드 방향. 락온이면 쓰지 않는다(대상을 본다).
        // 커서(0058): 앱이 땅에 비춘 점이 있으면 스틱 대신 그 점 쪽 — 마우스는 오른쪽 스틱과 같은 일을 한다.
        let aim_point = a
            .aim_at
            .filter(|_| target.is_none())
            .map(|q| vec3(q.x, self.player.pos.y, q.y));
        let aim = if target.is_some() {
            None
        } else if let Some(q) = aim_point {
            let d = vec3(q.x - self.player.pos.x, 0.0, q.z - self.player.pos.z);
            (d.length() > CURSOR_DEAD_M).then(|| d.normalize())
        } else if a.aim.length() > AIM_STICK {
            let w = move_to_world(a.aim.normalize(), basis, self.player.pos);
            (w.length_squared() > 1e-6).then(|| w.normalize())
        } else {
            None
        };
        // 공중에서 누른 공격 = 내려찍기(0027) — 자동 조준도 같이.
        let pressed_plunge =
            a.attack && matches!(self.player.state, PState::Jump { plunge: false, .. });
        // 공격 자동 조준(0014): 락온이 아니면 휘두르는 순간 `aim.auto range m` 안의 가장 가까운 적을 본다.
        // 조준 스틱을 쥐고 있으면 그 쪽 `aim.auto cone deg` 안에서만 고른다(멀리 있는 적을 일부러 겨눌 수 있게).
        let auto_dir = if target.is_none() && (a.attack || a.parry || pressed_plunge) {
            nearest_in_reach(
                &self.enemies,
                self.player.pos,
                t[P::AimRange],
                aim.map(|d| (d, (t[P::AimCone] * 0.5).to_radians().cos())),
            )
        } else {
            None
        };
        // 마법 쏘는 쪽(0048): 조준 스틱 > 락온 대상 > `cast.range m` 안의 가장 가까운 적 > 바라보는 쪽.
        let charging_cast = matches!(self.player.state, PState::CastCharge { .. });
        let cast_dir = if a.cast || charging_cast {
            let lock_dir = target.map(|tg| {
                let d = vec3(tg.x - self.player.pos.x, 0.0, tg.z - self.player.pos.z);
                d.normalize_or_zero()
            });
            aim.or(lock_dir.filter(|d| d.length_squared() > 0.5))
                .or_else(|| nearest_in_reach(&self.enemies, self.player.pos, t[P::CastRange], None))
                .unwrap_or_else(|| self.player.facing_dir())
        } else {
            self.player.facing_dir()
        };
        // 수류탄을 떨굴 자리(0051): 조준 스틱 쪽 가까운 거리 > 락온 대상 > 먼 거리 안 가장 가까운 적 > 바라보는 쪽.
        let wpn = self.player.weapon.def();
        let throw_to = wpn.grenade.map(|g| {
            let p0 = self.player.pos;
            let clamp_to = |q: Vec3| {
                let d = vec3(q.x - p0.x, 0.0, q.z - p0.z);
                let n = d.length().clamp(2.0, g.far);
                p0 + d.normalize_or(self.player.facing_dir()) * n
            };
            if let Some(q) = aim_point.filter(|_| aim.is_some()) {
                // 커서는 자리까지 준다(0058): 그 자리로, 2m..먼 거리 안으로.
                clamp_to(q)
            } else if let Some(d) = aim {
                p0 + d * g.near
            } else if let Some(tg) = target {
                clamp_to(tg)
            } else if let Some(d) = nearest_in_reach(&self.enemies, p0, g.far, None) {
                let e = self
                    .enemies
                    .iter()
                    .filter(|e| e.active())
                    .min_by(|a, b| a.pos.distance(p0).total_cmp(&b.pos.distance(p0)));
                e.map_or(p0 + d * g.near, |e| clamp_to(e.pos))
            } else {
                p0 + self.player.facing_dir() * g.near
            }
        });
        // 꾹 쥐어 겨누는 수류탄(0055)도 커서가 있으면 그 자리(0058) — 없으면 가장 멀리.
        let hold_throw_to = wpn.grenade.and_then(|g| {
            let p0 = self.player.pos;
            aim_point.filter(|_| aim.is_some()).map(|q| {
                let d = vec3(q.x - p0.x, 0.0, q.z - p0.z);
                p0 + d.normalize_or(self.player.facing_dir()) * d.length().clamp(2.0, g.far)
            })
        });
        let mut fire: Option<(Vec3, u8)> = None;
        let mut thrown: Option<Vec3> = None;
        let was_dodging = matches!(self.player.state, PState::Dodge { .. });
        let pl = &mut self.player;
        // 콤보·원거리 무기 시계(0051).
        pl.combo_left = (pl.combo_left - dt).max(0.0);
        if let Rb::Shot { mag, .. } = wpn.rb {
            if pl.reload > 0.0 {
                pl.reload -= dt;
                if pl.reload <= 0.0 {
                    pl.reload = 0.0;
                    pl.ammo = mag;
                }
            }
        }
        if let Some(g) = wpn.grenade {
            pl.grenades = (pl.grenades + dt / g.cooldown.max(0.1)).min(g.count as f32);
        }
        pl.hurt = (pl.hurt - dt).max(0.0);
        pl.cast_denied = (pl.cast_denied - dt).max(0.0);
        pl.dodge_cd = (pl.dodge_cd - dt).max(0.0);
        pl.since_parry = (pl.since_parry + dt).min(f32::MAX);
        pl.parry_chain_left = (pl.parry_chain_left - dt).max(0.0);
        if pl.parry_chain_left <= 0.0 {
            pl.parry_chain_n = 0;
        }
        pl.dash_left = (pl.dash_left - dt).max(0.0);
        pl.since_swing = (pl.since_swing + dt).min(f32::MAX);
        pl.since_beat = (pl.since_beat + dt).min(f32::MAX);
        pl.since_perfect = (pl.since_perfect + dt).min(f32::MAX);
        // 스태미나: 마지막으로 쓴 뒤 `stamina.regen delay s`가 지나야 찬다.
        pl.stamina_denied = (pl.stamina_denied - dt).max(0.0);
        pl.stun_window = (pl.stun_window - dt).max(0.0);
        pl.release_guard = (pl.release_guard - dt).max(0.0);
        pl.stamina_idle += dt;
        if pl.stamina_idle >= t[P::StaminaDelay] {
            pl.stamina = (pl.stamina + t[P::StaminaRegen] * dt).min(t[P::StaminaMax]);
        }
        // 행동은 스태미나가 조금이라도 있으면 된다(소울류처럼 0까지 쓰고 끝). 없으면 막히고 막대가 깜빡인다.
        let spend = |pl: &mut Player, cost: f32| -> bool {
            if pl.stamina > 0.0 {
                pl.stamina = (pl.stamina - cost).max(0.0);
                pl.stamina_idle = 0.0;
                true
            } else {
                // 막힘 알림(0088): 막대 깜빡·떨림 + 머리 위 땀방울 + "안 됨" 소리. 이미 알리는 중이면(받아 둔 누름이 틱마다 다시 막힘) 새로 시작하지 않는다.
                if pl.stamina_denied <= 0.0 {
                    pl.stamina_denied = STAMINA_DENIED_SHOW;
                    pl.denied_new = true;
                }
                false
            }
        };
        pl.turn_cd = (pl.turn_cd - dt).max(0.0);
        pl.flask_lock = (pl.flask_lock - dt).max(0.0);
        pl.flask_denied = (pl.flask_denied - dt).max(0.0);
        // 새 누름이면 0부터, 쥐고 있으면 늘고, 놓으면 0.
        pl.attack_hold = if a.attack || !a.attack_held {
            0.0
        } else {
            pl.attack_hold + dt
        };
        pl.cast_hold = if a.cast || !a.cast_held {
            0.0
        } else {
            pl.cast_hold + dt
        };
        // 원거리 강공격(0056)을 모을 수 있나 — 떼는 순간 칸(마법)·볼트(쇠뇌) 하나를 쓴다.
        let can_charge_cast = match wpn.rb {
            Rb::Magic(_) => pl.mana >= 1.0,
            Rb::Shot { .. } => pl.ammo >= 1,
        };
        pl.since_spill = (pl.since_spill + dt).min(f32::MAX);
        let world_move = move_to_world(a.move_dir, basis, pl.pos);
        // 땅 돌리기(0012): 쿼터뷰에서만, 서 있거나 스윙 후딜레이(회피와 같은 캔슬 자리)에서. ↻이 ↺보다 먼저(둘 다면 하나만).
        // 땅 돌리기(0012·0023): 쿼터뷰에서만, 서 있거나 스윙 후딜레이(회피와 같은 캔슬 자리)에서. 오른쪽 버튼이 왼쪽보다 먼저.
        let turn = (a.turn_right || a.turn_left).then_some(a.turn_right);
        let can_turn = target.is_none()
            && pl.turn_cd <= 0.0
            && match pl.state {
                PState::Idle => true,
                PState::Attack { t: e, hit, .. } => {
                    let (s0, a0, _) = hit_times(t, pl.weapon.def(), hit);
                    e >= s0 + a0
                }
                _ => false,
            };
        if let (Some(right), true) = (turn, can_turn) {
            // 캐릭터 기준 90°(0028): 몸이 제 오른쪽(왼쪽)으로 돌아서고 화면도 같은 쪽으로 90° 돈다.
            let plan = plan_turn(right, pl.facing);
            pl.state = PState::Turn {
                t: 0.0,
                from: pl.facing,
                to: plan.facing,
            };
            pl.turn_cd = t[P::TurnCooldown];
            // 쏟는 쪽: 카메라가 도는 쪽(+ = 시계).
            let cw = plan.quarters > 0;
            self.turn_now = Some((plan.quarters, cw));
            // 쥔 스틱은 곧바로 새 화면 기준 — 몸과 화면이 같이 돌았으니 그대로 새 정면으로 직진한다(0028).
            // 예전엔 돌기 전 기준을 붙잡아(0010) 돌린 뒤 엉뚱한 쪽으로 걸었다.
            self.hold = None;
        }

        // 상태 전이.
        let mut landed_plunge = false;
        match pl.state {
            PState::Idle => {
                if a.dodge && pl.dodge_cd <= 0.0 && spend(pl, t[P::StaminaDodge]) {
                    let dir = if world_move.length_squared() > 1e-4 {
                        world_move.normalize()
                    } else {
                        -pl.facing_dir()
                    };
                    pl.state = PState::Dodge { t: 0.0, dir };
                    pl.dodge_cd = t[P::DodgeTime] + 0.15;
                } else if a.jump && spend(pl, t[P::JumpStamina]) {
                    // 점프(0027): 뛰기 전 속도를 그대로 잇는다(달리다 뛰면 멀리).
                    pl.state = PState::Jump {
                        t: 0.0,
                        plunge: false,
                    };
                    pl.vy = crate::tuning::jump_ballistics(t).0;
                    pl.air = 0.0;
                } else if a.parry && spend(pl, t[P::ParryStamina]) {
                    // 패링(0025): 가까운 적 쪽으로 돌아서 막는다(공격 자동 조준과 같은 고르기).
                    if let Some(d) = auto_dir.or(aim) {
                        pl.facing = d.x.atan2(d.z);
                    }
                    pl.state = PState::Parry { t: 0.0 };
                } else if a.attack && !a.dodge && wpn.grenade.is_some() {
                    // 원거리 무기의 X = 수류탄(0048·0051): 남아 있으면 던지고, 없으면 칸이 깜빡인다.
                    if pl.grenades >= 1.0 {
                        pl.grenades -= 1.0;
                        let to = throw_to.unwrap_or(pl.pos);
                        let d = vec3(to.x - pl.pos.x, 0.0, to.z - pl.pos.z);
                        if d.length() > 0.1 {
                            pl.facing = d.x.atan2(d.z);
                        }
                        pl.state = PState::Throw {
                            t: 0.0,
                            to,
                            thrown: false,
                        };
                    } else {
                        pl.cast_denied = 0.6;
                        self.cast_denied_now = true;
                    }
                } else if a.attack && !a.dodge && {
                    // 박자에 맞춘 평타 콤보(이어지는 타)는 스태미나를 쓰지 않는다(2026-11-07, 사장). 대시 공격·패링 반격은 늘 쓴다.
                    let riposte = pl.since_parry <= RIPOSTE_WINDOW && wpn.heavy.is_some();
                    let dash = pl.dash_left > 0.0 && wpn.dash.is_some();
                    let free = pl.on_beat && pl.combo_left > 0.0 && !riposte && !dash;
                    pl.on_beat = false;
                    let go = free || spend(pl, t[P::StaminaAttack]);
                    if go {
                        // 박자를 놓친 타·새 콤보면 잇단 수는 0(놓침은 벌하지 않는다 — 소리 없음, 수만 0, KB).
                        pl.swing_beat = free;
                        if free {
                            self.beat_now = true;
                            pl.beat_chain += 1;
                            pl.since_beat = 0.0;
                        } else {
                            pl.beat_chain = 0;
                        }
                    }
                    go
                } {
                    pl.swings += 1;
                    // 휘두르는 쪽을 그 자리에서 정한다: 자동 조준 > 조준 스틱 > 지금 바라보는 쪽.
                    if let Some(d) = auto_dir.or(aim) {
                        pl.facing = d.x.atan2(d.z);
                    }
                    // 내딛는 쪽 = 진행 방향(0018): 누른 순간의 이동 입력, 없으면 아직 움직이던 쪽, 서 있으면 바라보는 쪽.
                    // 휘두르는 쪽(바라보는 쪽 — 자동 조준·조준 스틱)과는 따로다.
                    let moving = vec3(pl.vel.x, 0.0, pl.vel.z);
                    let dir = if world_move.length_squared() > 1e-4 {
                        world_move.normalize()
                    } else if moving.length() > LUNGE_MOVING {
                        moving.normalize()
                    } else {
                        pl.facing_dir()
                    };
                    // 콤보(0051): 끝난 지 `combo.keep s` 안이면 다음 타부터.
                    let hit = if pl.combo_left > 0.0 {
                        pl.combo_next
                    } else {
                        0
                    };
                    pl.combo_left = 0.0;
                    let riposte = pl.since_parry <= RIPOSTE_WINDOW && wpn.heavy.is_some();
                    let dash = pl.dash_left > 0.0 && wpn.dash.is_some();
                    pl.dash_left = 0.0;
                    if riposte {
                        pl.beat = 0.0;
                        // 패링 반격(2026-11-06): 튕긴 직후의 공격은 모으기 없이 다 모은 강타 — 드러난 적에게 치명타가 더해진다.
                        // 반격을 고르면 방어를 내려놓는다 — 이어 패링도 닫힌다(0081).
                        pl.since_parry = f32::MAX;
                        pl.parry_chain_left = 0.0;
                        pl.parry_chain_n = 0;
                        pl.state = PState::Heavy {
                            t: 0.0,
                            swing: pl.swings,
                            dir: pl.facing_dir(),
                            full: true,
                            perfect: false,
                        };
                    } else {
                        let hit = if dash { weapons::DASH } else { hit };
                        let (s0, a0, r0) = hit_times(t, wpn, hit);
                        pl.since_swing = 0.0;
                        pl.beat = s0 + a0 + r0;
                        pl.swing_hit = hit;
                        pl.state = PState::Attack {
                            t: 0.0,
                            swing: pl.swings,
                            dir,
                            hit,
                        };
                    }
                } else if a.cast && matches!(wpn.rb, Rb::Shot { .. }) {
                    // 원거리 무기의 RB = 사격(0051): 탄창이 비면 장전이 끝날 때까지 못 쏜다.
                    if let Rb::Shot { reload, time, .. } = wpn.rb {
                        // 박자(0071·0077): 앞 사격이 시작된 뒤 그 한 발의 시간(= 공격 속도) ± 창 안에 누른 사격은 볼트를 쓰지 않는다 —
                        // 근접의 "박자 평타 = 스태미나 0"과 같은 틀, 쇠뇌의 값은 볼트. 첫 발·받아 둔 이른 누름은 쓴다.
                        let free = pl.on_beat;
                        pl.on_beat = false;
                        if pl.ammo > 0 {
                            if free {
                                self.beat_now = true;
                                pl.beat_chain += 1;
                                pl.since_beat = 0.0;
                            } else {
                                pl.beat_chain = 0;
                                pl.ammo -= 1;
                                if pl.ammo == 0 {
                                    pl.reload = reload;
                                }
                            }
                            pl.swing_beat = free;
                            pl.since_swing = 0.0;
                            pl.beat = t[P::CastTime] * time;
                            pl.facing = cast_dir.x.atan2(cast_dir.z);
                            pl.state = PState::Cast {
                                t: 0.0,
                                dir: cast_dir,
                                fired: false,
                            };
                        } else {
                            pl.cast_denied = 0.6;
                            self.cast_denied_now = true;
                        }
                    }
                } else if a.cast {
                    if pl.mana >= 1.0 {
                        pl.mana -= 1.0;
                        pl.facing = cast_dir.x.atan2(cast_dir.z);
                        pl.state = PState::Cast {
                            t: 0.0,
                            dir: cast_dir,
                            fired: false,
                        };
                    } else {
                        pl.cast_denied = 0.6;
                        self.cast_denied_now = true;
                    }
                } else if a.flask && pl.flask_lock <= 0.0 {
                    // 등잔(0031): 가득이거나 0이면 몸짓 없이 깜빡만(빈 병 몸짓으로 묶지 않는다 — 급할 때 연타해도).
                    if pl.flask > 0 && pl.hp < pl.max_hp {
                        pl.state = PState::Drink {
                            t: 0.0,
                            gulped: false,
                        };
                    } else {
                        pl.flask_denied = 0.6;
                        self.flask_denied_now = true;
                    }
                }
            }
            PState::Drink { t: e, gulped } => {
                // 다시 누르기는 버린다. 삼키기 전 회피는 끊기(등잔 그대로) — 어깨 버튼 실수는 회피 한 번으로 무른다.
                if !gulped && a.dodge && pl.dodge_cd <= 0.0 && spend(pl, t[P::StaminaDodge]) {
                    let dir = if world_move.length_squared() > 1e-4 {
                        world_move.normalize()
                    } else {
                        -pl.facing_dir()
                    };
                    pl.state = PState::Dodge { t: 0.0, dir };
                    pl.dodge_cd = t[P::DodgeTime] + 0.15;
                } else {
                    let e2 = e + dt;
                    let mut gulped = gulped;
                    if !gulped && e2 >= t[P::FlaskRaise] {
                        gulped = true;
                        pl.flask -= 1;
                        pl.hp = (pl.hp + t[P::FlaskHeal].round() as i32).min(pl.max_hp);
                        self.gulp_now = true;
                    }
                    if e2 >= t[P::FlaskTime] {
                        pl.state = PState::Idle;
                        pl.flask_lock = t[P::FlaskLock];
                    } else {
                        pl.state = PState::Drink { t: e2, gulped };
                    }
                }
            }
            PState::Hurt { t: e, dur } => {
                pl.state = if e + dt >= dur {
                    PState::Idle
                } else {
                    PState::Hurt { t: e + dt, dur }
                };
            }
            PState::Attack {
                t: e,
                swing,
                dir: ldir,
                hit,
            } => {
                let (s0, a0, r0) = hit_times(t, wpn, hit);
                let total = s0 + a0 + r0;
                let after = e >= s0 + a0;
                let after_hit = wpn.after(hit);
                // 회피·패링·점프·마법·다음 타는 캔슬(0057, `cancel_or_buffer`)이 맡는다 — 여기서는 모으기와 끝만.
                if a.attack_held
                    && pl.attack_hold >= t[P::HeavyHoldStart]
                    && after
                    && wpn.heavy.is_some()
                {
                    // 강공격(0030): 평타는 누르는 순간 나갔고, 한 번 누른 채 `heavy.hold start s`를 쥐고 있으면(활성이 끝난 뒤) 모으기로.
                    // "그 순간 눌려 있나"만 보면 연타 중 휘두르기 도중의 짧은 누름이 활성 끝에 걸려 모으기로 샜다.
                    pl.state = PState::Charging { t: 0.0 };
                } else if e + dt >= total {
                    pl.state = PState::Idle;
                    // 마지막 타 뒤에는 처음으로, 아니면 잠깐 다음 타를 기억한다.
                    pl.combo_next = after_hit.unwrap_or(0);
                    pl.combo_left = if after_hit.is_some() {
                        t[P::ComboKeep]
                    } else {
                        0.0
                    };
                } else {
                    pl.state = PState::Attack {
                        t: e + dt,
                        swing,
                        dir: ldir,
                        hit,
                    };
                }
            }
            PState::Charging { t: e } => {
                if a.dodge && pl.dodge_cd <= 0.0 && spend(pl, t[P::StaminaDodge]) {
                    let dir = if world_move.length_squared() > 1e-4 {
                        world_move.normalize()
                    } else {
                        -pl.facing_dir()
                    };
                    pl.state = PState::Dodge { t: 0.0, dir };
                    pl.dodge_cd = t[P::DodgeTime] + 0.15;
                } else if a.parry && spend(pl, t[P::ParryStamina]) {
                    if let Some(d) = auto_dir.or(aim) {
                        pl.facing = d.x.atan2(d.z);
                    }
                    pl.state = PState::Parry { t: 0.0 };
                } else if a.attack_held {
                    let full = t[P::HeavyCharge] * wpn.heavy.map_or(1.0, |h| h.charge);
                    self.charged_now |= e < full && e + dt >= full;
                    pl.state = PState::Charging { t: e + dt };
                } else if spend(pl, t[P::HeavyStamina]) {
                    let h = wpn.heavy.expect("charging only with a heavy");
                    let full_at = t[P::HeavyCharge] * h.charge;
                    // 떼면 강타 — 자동 조준·조준 스틱 쪽으로, 내딛는 쪽은 이동 입력(없으면 바라보는 쪽).
                    if let Some(d) = auto_dir.or(aim) {
                        pl.facing = d.x.atan2(d.z);
                    }
                    let dir = if world_move.length_squared() > 1e-4 {
                        world_move.normalize()
                    } else {
                        pl.facing_dir()
                    };
                    pl.swings += 1;
                    // 모으기 공격의 패링(0089): 떼는 순간부터 패링 창 — 모으는 동안 무방비였던 값.
                    pl.release_guard = t[P::ParryWindow];
                    pl.state = PState::Heavy {
                        t: 0.0,
                        swing: pl.swings,
                        dir,
                        full: e >= full_at,
                        // 딱 맞춰 떼기(0078 — 모든 무기): 다 모인 뒤 창 안에 떼면 덤.
                        perfect: e >= full_at && e <= full_at + PERFECT_RELEASE,
                    };
                } else {
                    pl.state = PState::Idle;
                }
            }
            PState::Heavy {
                t: e,
                swing,
                dir: ldir,
                full,
                perfect,
            } => {
                let (s0, a0, r0) = heavy_times(t, wpn);
                let total = s0 + a0 + r0;
                if a.dodge && pl.dodge_cd <= 0.0 && e >= s0 + a0 && spend(pl, t[P::StaminaDodge]) {
                    let dir = if world_move.length_squared() > 1e-4 {
                        world_move.normalize()
                    } else {
                        -pl.facing_dir()
                    };
                    pl.state = PState::Dodge { t: 0.0, dir };
                    pl.dodge_cd = t[P::DodgeTime] + 0.15;
                } else if e + dt >= total {
                    pl.state = PState::Idle;
                } else {
                    pl.state = PState::Heavy {
                        t: e + dt,
                        swing,
                        dir: ldir,
                        full,
                        perfect,
                    };
                }
            }
            PState::Cast { t: e, dir, fired } => {
                if fired && a.dodge && pl.dodge_cd <= 0.0 && spend(pl, t[P::StaminaDodge]) {
                    let ddir = if world_move.length_squared() > 1e-4 {
                        world_move.normalize()
                    } else {
                        -pl.facing_dir()
                    };
                    pl.state = PState::Dodge { t: 0.0, dir: ddir };
                    pl.dodge_cd = t[P::DodgeTime] + 0.15;
                } else {
                    let e2 = e + dt;
                    let mut fired = fired;
                    let k = match wpn.rb {
                        Rb::Shot { time, .. } => time,
                        Rb::Magic(_) => 1.0,
                    };
                    if !fired && e2 >= t[P::CastStartup] * k {
                        fired = true;
                        fire = Some((dir, 0));
                    }
                    pl.state = if fired
                        && a.cast_held
                        && pl.cast_hold >= t[P::CastHoldStart]
                        && can_charge_cast
                    {
                        // 원거리 강공격(0056): 쏜 뒤에도 쥐고 있으면 모으기로(평타 → 강타와 같은 틀, 0030).
                        PState::CastCharge { t: 0.0 }
                    } else if e2 >= t[P::CastTime] * k {
                        PState::Idle
                    } else {
                        PState::Cast { t: e2, dir, fired }
                    };
                }
            }
            PState::CastCharge { t: e } => {
                let full = t[P::CastCharge];
                if a.dodge && pl.dodge_cd <= 0.0 && spend(pl, t[P::StaminaDodge]) {
                    let ddir = if world_move.length_squared() > 1e-4 {
                        world_move.normalize()
                    } else {
                        -pl.facing_dir()
                    };
                    pl.state = PState::Dodge { t: 0.0, dir: ddir };
                    pl.dodge_cd = t[P::DodgeTime] + 0.15;
                } else if a.cast_held && can_charge_cast {
                    self.charged_now |= e < full && e + dt >= full;
                    pl.state = PState::CastCharge { t: e + dt };
                } else if can_charge_cast {
                    // 떼면 모은 탄 — 칸·볼트 하나, 다 모았으면 무기 표의 `charged`, 덜 모았으면 약하게. 쏘는 동작의 뒷부분으로 이어진다.
                    match wpn.rb {
                        Rb::Magic(_) => pl.mana -= 1.0,
                        Rb::Shot { reload, .. } => {
                            pl.ammo -= 1;
                            if pl.ammo == 0 {
                                pl.reload = reload;
                            }
                        }
                    }
                    pl.facing = cast_dir.x.atan2(cast_dir.z);
                    // 모으기 공격의 패링(0089): 원거리 무기의 모은 사격도 떼는 순간부터 패링 창(근접 무기의 RB 마법은 무기의 공격이 아니라 뺀다, 0077·0078과 같은 경계).
                    if matches!(wpn.rb, Rb::Shot { .. }) {
                        pl.release_guard = t[P::ParryWindow];
                    }
                    // 딱 맞춰 떼기(0078): 원거리 무기의 모으기 공격(사격 꾹)도 다 모인 뒤 창 안에 떼면 덤 — 근접 무기의 RB 마법은 무기의
                    // 공격이 아니라 빼 둔다(박자와 같은 경계, 0077).
                    let perfect = matches!(wpn.rb, Rb::Shot { .. })
                        && e >= full
                        && e <= full + PERFECT_RELEASE;
                    fire = Some((
                        cast_dir,
                        match (e >= full, perfect) {
                            (_, true) => 3,
                            (true, false) => 2,
                            _ => 1,
                        },
                    ));
                    let k = match wpn.rb {
                        Rb::Shot { time, .. } => time,
                        Rb::Magic(_) => 1.0,
                    };
                    pl.state = PState::Cast {
                        t: t[P::CastStartup] * k,
                        dir: cast_dir,
                        fired: true,
                    };
                } else {
                    pl.state = PState::Idle;
                }
            }
            PState::Throw {
                t: e,
                to,
                thrown: done,
            } => {
                if done && a.dodge && pl.dodge_cd <= 0.0 && spend(pl, t[P::StaminaDodge]) {
                    let ddir = if world_move.length_squared() > 1e-4 {
                        world_move.normalize()
                    } else {
                        -pl.facing_dir()
                    };
                    pl.state = PState::Dodge { t: 0.0, dir: ddir };
                    pl.dodge_cd = t[P::DodgeTime] + 0.15;
                } else if !done && e + dt >= t[P::ThrowStartup] && a.attack_held {
                    // 꾹 쥐면 겨누기(0055): 손을 든 채 멈추고, 떨굴 자리 = 조준 스틱·이동 쪽(없으면 바라보는 쪽) 가장 멀리.
                    // 떼면 그 자리로. 톡(손을 떠나기 전에 뗌)은 자동 조준 자리(0051).
                    let dir = aim
                        .or((world_move.length_squared() > 1e-4).then(|| world_move.normalize()))
                        .unwrap_or_else(|| pl.facing_dir());
                    pl.facing = dir.x.atan2(dir.z);
                    let far = wpn.grenade.map_or(8.0, |g| g.far);
                    pl.state = PState::Throw {
                        t: t[P::ThrowStartup] - 1e-4,
                        to: hold_throw_to.unwrap_or(pl.pos + dir * far),
                        thrown: false,
                    };
                } else {
                    let e2 = e + dt;
                    let mut done = done;
                    if !done && e2 >= t[P::ThrowStartup] {
                        done = true;
                        thrown = Some(to);
                    }
                    pl.state = if e2 >= t[P::ThrowTime] {
                        PState::Idle
                    } else {
                        PState::Throw {
                            t: e2,
                            to,
                            thrown: done,
                        }
                    };
                }
            }
            PState::Dodge { t: e, dir } => {
                if e + dt >= t[P::DodgeTime] {
                    pl.state = PState::Idle;
                    // 회피가 끝난 직후의 공격도 대시 공격(늦게 누른 것도 받아 준다).
                    pl.dash_left = DASH_ATTACK_LATE;
                    // 회피는 누르는 순간 그대로, 끝났을 때 아직 쥐고 있으면 달리기(0076, KB nail 0209) — 톡 누름은 회피보다 짧아
                    // 예전과 같다. 회피의 기세를 그대로 잇는다(속도를 0으로 떨어뜨리지 않는다).
                    pl.running = a.dodge_held && pl.stamina > 0.0;
                } else {
                    pl.state = PState::Dodge { t: e + dt, dir };
                }
            }
            PState::Jump { t: e, plunge } => {
                let mut plunge = plunge;
                if a.attack && !plunge && spend(pl, t[P::StaminaAttack]) {
                    plunge = true;
                    if let Some(d) = auto_dir.or(aim) {
                        pl.facing = d.x.atan2(d.z);
                    }
                }
                // 포물선을 그대로(틱마다 위치 = v·dt − g·dt²/2) — 앞 오일러 적분은 높이가 6% 모자랐다.
                let g =
                    crate::tuning::jump_ballistics(t).1 * if plunge { PLUNGE_FALL } else { 1.0 };
                pl.air += pl.vy * dt - 0.5 * g * dt * dt;
                pl.vy -= g * dt;
                if pl.air <= 0.0 && e > 0.0 {
                    pl.air = 0.0;
                    pl.vy = 0.0;
                    let dur = if plunge {
                        t[P::PlungeLand]
                    } else {
                        t[P::JumpLand]
                    };
                    pl.state = PState::Land { t: 0.0, dur };
                    landed_plunge = plunge;
                } else {
                    pl.state = PState::Jump { t: e + dt, plunge };
                }
            }
            PState::Land { t: e, dur } => {
                pl.state = if e + dt >= dur {
                    PState::Idle
                } else {
                    PState::Land { t: e + dt, dur }
                };
            }
            PState::Parry { t: e } => {
                pl.state = if e + dt >= t[P::ParryTime] {
                    PState::Idle
                } else {
                    PState::Parry { t: e + dt }
                };
            }
            PState::Turn { t: e, from, to } => {
                let e2 = e + dt;
                let k = (e2 / t[P::TurnPlant].max(1e-3)).clamp(0.0, 1.0);
                // 천천히-빠르게-천천히(0034): 낮췄다가 휙 돌고 선다 — 동작 `turn_r`·`turn_l`과 같은 박자.
                let k = k * k * (3.0 - 2.0 * k);
                pl.facing = from + angle_diff(to, from) * k;
                if e2 >= t[P::TurnPlant] {
                    pl.facing = to;
                    pl.state = PState::Idle;
                } else {
                    pl.state = PState::Turn { t: e2, from, to };
                }
            }
        }

        // 달리기(0076): 쥔 동안·스틱을 기울인 동안·스태미나가 있는 동안·서 있는(걷는) 동안만. 달리는 동안은 스태미나가 차지 않고
        // 공격하면 대시 공격(회피에서 나온 것과 같은 한 타 — 소울류 달리며 치기).
        if pl.running {
            let keep = pl.state == PState::Idle
                && a.dodge_held
                && world_move.length_squared() > 0.04
                && pl.stamina > 0.0;
            if keep {
                pl.stamina = (pl.stamina - RUN_STAMINA * dt).max(0.0);
                pl.stamina_idle = 0.0;
                pl.dash_left = pl.dash_left.max(DASH_ATTACK_LATE);
            } else {
                pl.running = false;
            }
        }

        // 이동.
        match pl.state {
            PState::Dodge { dir, .. } => {
                pl.vel = dir * (t[P::DodgeDist] / t[P::DodgeTime]);
            }
            PState::Attack {
                t: e,
                dir,
                hit,
                swing,
            } => {
                // 선딜 + 활성 동안 `attack.lunge m` × 타의 배율만큼 입력한 쪽으로 내딛고, 후딜에는 선다(0018·0051).
                let (s0, a0, _) = hit_times(t, wpn, hit);
                let reach = s0 + a0;
                let lunge = t[P::AttackLunge] * wpn.hit(hit).map_or(1.0, |h| h.lunge);
                pl.vel = if e < reach && pl.connected != swing {
                    dir * (lunge / reach.max(1e-3))
                } else {
                    Vec3::ZERO
                };
            }
            PState::Hurt { .. } => {
                pl.vel *= (1.0 - KNOCK_DECAY * dt).max(0.0);
            }
            // 패링은 걸음을 멈출 때와 같은 감속으로 선다 — 그 자리에서 0이면 2차 동작의 몸이 뒤로 출렁여
            // "뒤로 물러나는" 느낌이 났다(사장 2026-11-06).
            PState::Parry { .. } => {
                let v = pl.vel.length();
                let slow = (v - t[P::MoveAccel] * dt).max(0.0);
                pl.vel = if v > 1e-4 {
                    pl.vel * (slow / v)
                } else {
                    Vec3::ZERO
                };
            }
            PState::Turn { .. } | PState::Land { .. } => pl.vel = Vec3::ZERO,
            PState::Heavy { t: e, dir, .. } => {
                let (s0, a0, _) = heavy_times(t, wpn);
                let reach = s0 + a0;
                let lunge = t[P::HeavyLunge] * wpn.heavy.map_or(1.0, |h| h.lunge);
                pl.vel = if e < reach {
                    dir * (lunge / reach.max(1e-3))
                } else {
                    Vec3::ZERO
                };
            }
            PState::Jump { .. } if world_move.length_squared() < 1e-4 => {
                // 공중에서 손을 놓으면 뛰기 전 속도 그대로(멈추는 쪽으로 조종하지 않는다).
            }
            PState::Jump { .. } => {
                // 공중: 뛰기 전 속도를 잇고 `jump.air control x`만큼만 바꾼다.
                let want = world_move * t[P::MoveSpeed];
                let diff = want - pl.vel;
                let max = t[P::MoveAccel] * t[P::JumpAirControl] * dt;
                pl.vel += if diff.length() > max {
                    diff.normalize() * max
                } else {
                    diff
                };
            }
            PState::Idle
            | PState::Drink { .. }
            | PState::Charging { .. }
            | PState::Cast { .. }
            | PState::CastCharge { .. }
            | PState::Throw { .. } => {
                // 마시는 동안·강공격을 모으는 동안(0047)·마법을 쏘는 동안(0048)은 느리게 걷는다.
                let slow = match pl.state {
                    PState::Drink { .. } => t[P::FlaskWalk],
                    PState::Charging { .. } => {
                        t[P::HeavyChargeWalk] * wpn.heavy.map_or(1.0, |h| h.walk)
                    }
                    PState::Cast { .. } | PState::Throw { .. } | PState::CastCharge { .. } => {
                        t[P::CastWalk]
                    }
                    PState::Idle if pl.running => RUN_SPEED_X,
                    _ => 1.0,
                };
                let want = world_move * t[P::MoveSpeed] * slow;
                let diff = want - pl.vel;
                let max = t[P::MoveAccel] * dt;
                pl.vel += if diff.length() > max {
                    diff.normalize() * max
                } else {
                    diff
                };
            }
        }
        pl.pos += pl.vel * dt;

        // 바라보기: 락온이면 대상, 조준 스틱을 쥐면 그 쪽(움직이며 다른 쪽을 볼 수 있다, 0014), 아니면 이동 방향. 스윙 중엔 고정.
        let want_yaw = match (basis, pl.state) {
            (
                _,
                PState::Attack { .. }
                | PState::Heavy { .. }
                | PState::Turn { .. }
                | PState::Hurt { .. }
                | PState::Parry { .. }
                | PState::Jump { plunge: true, .. }
                | PState::Cast { .. }
                | PState::Throw { .. }
                | PState::Land { .. },
            ) => None,
            (Basis::Lock(tg), _) => {
                let d = tg - pl.pos;
                Some(d.x.atan2(d.z))
            }
            _ if aim.is_some() => aim.map(|d| d.x.atan2(d.z)),
            (Basis::Quarter { .. } | Basis::Frozen(_), _) if world_move.length_squared() > 1e-4 => {
                Some(world_move.x.atan2(world_move.z))
            }
            _ => None,
        };
        if let Some(w) = want_yaw {
            let mut diff = w - pl.facing;
            while diff > std::f32::consts::PI {
                diff -= std::f32::consts::TAU;
            }
            while diff < -std::f32::consts::PI {
                diff += std::f32::consts::TAU;
            }
            let max = t[P::TurnSpeed].to_radians() * dt;
            pl.facing += diff.clamp(-max, max);
        }
        // 점프가 아닌 동작(회피·공격 …) 중에 턱에서 떨어졌다(0043): 동작은 그대로, 몸만 떨어진다.
        if !matches!(pl.state, PState::Jump { .. }) && pl.air > 0.0 {
            let g = crate::tuning::jump_ballistics(t).1;
            pl.air += pl.vy * dt - 0.5 * g * dt * dt;
            pl.vy -= g * dt;
            if pl.air <= 0.0 {
                pl.air = 0.0;
                pl.vy = 0.0;
            }
        }
        self.plunge_now |= landed_plunge;
        // 받아 둔 행동이 쓰였다(새 동작이 시작됐다) — 끝나서 서게 된 것(`Idle`)이면 남겨 둔다(다음 틱에 나간다).
        let now = std::mem::discriminant(&self.player.state);
        if now != started_from && !matches!(self.player.state, PState::Idle) {
            self.player.pending = None;
        }
        if fire.is_some() {
            self.cast_fire = fire;
        }
        if thrown.is_some() {
            self.throw_to = thrown;
        }
        // 원거리 무기는 회피하면 한 발 장전(0051, PoE2 착상) — 회피를 공격 리듬에 넣는다.
        if !was_dodging && matches!(self.player.state, PState::Dodge { .. }) {
            if let Rb::Shot { mag, .. } = wpn.rb {
                let pl = &mut self.player;
                if pl.ammo < mag {
                    pl.ammo += 1;
                    if pl.ammo == mag {
                        pl.reload = 0.0;
                    }
                }
            }
        }
    }

    /// 날아가는 것·늦게 터지는 것(0048·0051): 마법 탄·쇠뇌 볼트, 수류탄, 금 간 땅(망치 여진), 자세 막대가 줄기, 폭발 고리.
    /// 탄은 이번 틱에 쏜 것을 세우고 날려 벽·땅·적에 닿으면 지운다(꿰뚫는 탄은 적을 지나간다). 칸은 채우지 않는다(근접만 채운다).
    fn tick_bolts(&mut self, t: &Tuning, dt: f32, ev: &mut Vec<Event>) {
        if std::mem::take(&mut self.beat_now) {
            ev.push(Event::Beat);
        }
        if std::mem::take(&mut self.cast_denied_now) {
            ev.push(Event::CastDenied);
        }
        if std::mem::take(&mut self.player.denied_new) {
            ev.push(Event::StaminaDenied);
        }
        let wpn = self.player.weapon.def();
        // 자세 막대(0051): 깬 뒤 잠금이 풀리고, 맞지 않은 지 오래면 줄어든다.
        for en in &mut self.enemies {
            en.poise_lock = (en.poise_lock - dt).max(0.0);
            en.broken = (en.broken - dt).max(0.0);
            en.poise_frac = (en.poise / poise_max(en, t).max(0.01)).clamp(0.0, 1.0);
            en.poise_idle += dt;
            if en.poise_idle >= t[P::PoiseDelay] {
                en.poise = (en.poise - t[P::PoiseDecay] * dt).max(0.0);
            }
        }
        for b in &mut self.blasts {
            b.age += dt;
        }
        self.blasts.retain(|b| b.age < BLAST_SHOW);
        for sp in &mut self.sparks {
            sp.age += dt;
            sp.vel.y -= 18.0 * dt;
            sp.vel *= (1.0 - 6.0 * dt).max(0.0);
            sp.pos += sp.vel * dt;
        }
        self.sparks.retain(|sp| sp.age < SPARK_LIFE);
        if let Some((dir, power)) = self.cast_fire.take() {
            let pl = &self.player;
            let pos = pl.pos + Vec3::Y * (pl.air + BOLT_HEIGHT) + dir * (PLAYER_RADIUS + 0.2);
            let c = wpn.charged;
            let def = match (power, wpn.rb) {
                (0, Rb::Magic(b)) => b,
                (0, Rb::Shot { bolt, .. }) => bolt,
                // 덜 모은 탄(0056): 모은 탄의 반 — 꿰뚫기·휘청 없음.
                (1, _) => weapons::Bolt {
                    damage: (c.damage + 1) / 2,
                    poise: c.poise * 0.5,
                    pierce: false,
                    stagger: 0.0,
                    scale: 1.0 + (c.scale - 1.0) * 0.4,
                    ..c
                },
                // 딱 맞춰 뗀 모은 탄(0078): 강타의 딱 맞춤과 같은 덤 — 피해 +1·자세 피해 ×1.5.
                (3, _) => weapons::Bolt {
                    damage: c.damage + 1,
                    poise: c.poise * 1.5,
                    ..c
                },
                _ => c,
            };
            if power == 3 {
                self.player.since_perfect = 0.0;
                ev.push(Event::Perfect { at: pos });
            }
            self.bolts.push(Bolt {
                pos,
                prev: pos,
                dir,
                left: t[P::CastRange] * def.range,
                def,
                hits: 0,
            });
            ev.push(Event::Cast { at: pos });
        }
        if let (Some(to), Some(g)) = (self.throw_to.take(), wpn.grenade) {
            // 포물선: 날아가는 시간을 거리로 정하고(가까우면 낮게·빨리) 그 시간에 `to`에 닿는 속도.
            let pl = &self.player;
            let from = pl.pos + Vec3::Y * (pl.air + BOLT_HEIGHT);
            let ground_to = self.ground.as_ref().map_or(0.0, |gr| gr.height(to));
            let flat = vec3(to.x - from.x, 0.0, to.z - from.z);
            let tf = (flat.length() / 10.0).clamp(0.45, 0.9);
            let vy = (ground_to + 0.2 - from.y + 0.5 * BOMB_GRAVITY * tf * tf) / tf;
            self.bombs.push(Bomb {
                pos: from,
                prev: from,
                vel: flat / tf + Vec3::Y * vy,
                fuse: g.fuse,
                radius: g.radius,
                damage: g.damage,
                poise: g.poise,
            });
            ev.push(Event::Throw);
        }
        let step = t[P::CastSpeed] * dt;
        let mut keep = Vec::with_capacity(self.bolts.len());
        for mut b in std::mem::take(&mut self.bolts) {
            b.prev = b.pos;
            let mut len = (step * b.def.speed).min(b.left);
            // 벽·기둥: 이번 걸음 안에서 먼저 닿는 상자까지만.
            let mut gone = false;
            for o in &self.obstacles {
                if let Some(d) = o.ray(b.pos, b.dir) {
                    if d <= len {
                        len = d;
                        gone = true;
                    }
                }
            }
            // 쇠뇌 볼트가 수류탄을 맞히면 그 자리에서 터진다(0051, PoE2 착상).
            if b.def.look == 3 {
                for bomb in &mut self.bombs {
                    let k = (bomb.pos - b.pos).dot(b.dir).clamp(0.0, len);
                    if (b.pos + b.dir * k).distance(bomb.pos) <= BOMB_SHOT_REACH {
                        bomb.fuse = 0.0;
                        gone = true;
                    }
                }
            }
            // 적: 걸음 선분에서 맞는 깨어 있는 적(높이가 몸 안). 꿰뚫는 탄은 모두, 아니면 가장 가까운 하나.
            let mut struck: Vec<(usize, f32)> = Vec::new();
            for (i, en) in self.enemies.iter().enumerate() {
                if !en.active()
                    || (i < 64 && b.hits & (1 << i) != 0)
                    || b.pos.y < en.pos.y - 0.2
                    || b.pos.y > en.pos.y + en.height + 0.2
                {
                    continue;
                }
                let c = vec3(en.pos.x, b.pos.y, en.pos.z);
                let k = (c - b.pos).dot(b.dir).clamp(0.0, len);
                if (b.pos + b.dir * k).distance(c) <= en.radius + BOLT_RADIUS * b.def.scale {
                    struck.push((i, k));
                }
            }
            struck.sort_by(|a, b| a.1.total_cmp(&b.1));
            if !b.def.pierce {
                struck.truncate(1);
            }
            for &(i, k) in &struck {
                if i < 64 {
                    b.hits |= 1 << i;
                }
                let en = &mut self.enemies[i];
                let crit = en.exposed > 0.0;
                en.hp -= hurt_by(en, self.stats.dmg + b.def.damage, crit, t);
                en.exposed = 0.0;
                en.hurt = 0.15;
                en.knock = b.dir * t[P::Knockback] * if en.boss { 0.1 } else { 0.5 };
                let at = b.pos + b.dir * k;
                ev.push(if crit {
                    Event::Crit { at }
                } else {
                    Event::Hit { at }
                });
                if b.def.stagger > 0.0 && !en.boss && en.hp > 0 {
                    en.state = EState::Stagger {
                        t: 0.0,
                        dur: b.def.stagger,
                    };
                }
                add_poise(en, b.def.poise, None, t, ev);
                if en.hp <= 0 {
                    en.state = EState::Dead;
                    self.kills += 1;
                    ev.push(Event::EnemyDown { at: en.pos });
                }
            }
            if !b.def.pierce && !struck.is_empty() {
                continue;
            }
            b.pos += b.dir * len;
            b.left -= len;
            // 땅: 탄보다 높은 땅(턱·벽 칸)에 들어가면 사라진다.
            if let Some(g) = &self.ground {
                let solid = g.cell_of(b.pos).is_none_or(|(x, z)| {
                    g.height_in(x, z, b.pos).is_none_or(|h| h > b.pos.y - 0.1)
                });
                gone |= solid;
            }
            if !gone
                && b.left > 1e-3
                && b.pos.x.abs() <= self.arena + 2.0
                && b.pos.z.abs() <= self.arena + 2.0
            {
                keep.push(b);
            }
        }
        self.bolts = keep;

        // 수류탄(0051): 중력으로 날다 땅에서 튕기고(줄어드는 위·옆 속도) 벽에서 되튄다, 도화선이 다하면 터진다.
        let mut booms: Vec<(Vec3, f32, i32, f32)> = Vec::new();
        for bomb in &mut self.bombs {
            bomb.prev = bomb.pos;
            bomb.fuse -= dt;
            bomb.vel.y -= BOMB_GRAVITY * dt;
            let next = bomb.pos + bomb.vel * dt;
            if self.obstacles.iter().any(|o| {
                next.x > o.min.x
                    && next.x < o.max.x
                    && next.z > o.min.z
                    && next.z < o.max.z
                    && next.y < o.max.y
            }) {
                bomb.vel.x *= -BOMB_SKID;
                bomb.vel.z *= -BOMB_SKID;
            } else {
                bomb.pos = next;
            }
            let floor = self.ground.as_ref().map_or(0.0, |g| g.height(bomb.pos)) + 0.15;
            if bomb.pos.y < floor {
                bomb.pos.y = floor;
                if bomb.vel.y < 0.0 {
                    bomb.vel.y = -bomb.vel.y * BOMB_BOUNCE;
                    bomb.vel.x *= BOMB_SKID;
                    bomb.vel.z *= BOMB_SKID;
                }
            }
            if bomb.fuse <= 0.0 {
                booms.push((bomb.pos, bomb.radius, bomb.damage, bomb.poise));
            }
        }
        self.bombs.retain(|b| b.fuse > 0.0);
        for (at, radius, damage, poise) in booms {
            self.burst(at, radius, damage, poise, t, ev);
            self.blasts.push(Blast {
                pos: at,
                radius,
                age: 0.0,
            });
            ev.push(Event::Boom { at });
        }
        // 금 간 땅(0051): 시간이 되면 둘레를 친다(땅 돌리기 중에도 시계는 간다 — 돌리고 떨구기).
        let mut quakes = Vec::new();
        for c in &mut self.cracks {
            c.left -= dt;
            if c.left <= 0.0 {
                quakes.push(*c);
            }
        }
        self.cracks.retain(|c| c.left > 0.0);
        for c in quakes {
            self.burst(c.pos, c.radius, c.damage, c.poise, t, ev);
            self.blasts.push(Blast {
                pos: c.pos,
                radius: c.radius,
                age: 0.0,
            });
            ev.push(Event::Aftershock { at: c.pos });
        }
    }

    /// 둘레 피해(수류탄·여진): 반지름 안의 깨어 있는 적(높이 차 `HIT_DY` 안)에게 피해·자세 피해·바깥으로 밀기.
    fn burst(
        &mut self,
        at: Vec3,
        radius: f32,
        damage: i32,
        poise: f32,
        t: &Tuning,
        ev: &mut Vec<Event>,
    ) {
        for en in &mut self.enemies {
            if !en.active() || (en.pos.y - at.y).abs() > HIT_DY + 0.3 {
                continue;
            }
            let d = vec3(en.pos.x - at.x, 0.0, en.pos.z - at.z);
            if d.length() - en.radius > radius {
                continue;
            }
            let crit = en.exposed > 0.0;
            en.hp -= hurt_by(en, self.stats.dmg + damage, crit, t);
            en.exposed = 0.0;
            en.hurt = 0.15;
            en.knock = d.normalize_or_zero() * t[P::Knockback] * if en.boss { 0.2 } else { 1.4 };
            let hit_at = en.pos + Vec3::Y * en.height * 0.6;
            ev.push(if crit {
                Event::Crit { at: hit_at }
            } else {
                Event::Hit { at: hit_at }
            });
            add_poise(en, poise, None, t, ev);
            if en.hp <= 0 {
                en.state = EState::Dead;
                self.kills += 1;
                ev.push(Event::EnemyDown { at: en.pos });
            }
        }
        self.hitstop = self.hitstop.max(t[P::Hitstop]);
    }

    /// 플레이어 스윙의 활성 구간에 범위·부채꼴 안의 적을 한 스윙에 한 번씩 때린다.
    fn apply_strike(&mut self, t: &Tuning, ev: &mut Vec<Event>) {
        if std::mem::take(&mut self.plunge_now) {
            self.plunge(t, ev);
        }
        if std::mem::take(&mut self.charged_now) {
            ev.push(Event::Charged);
        }
        if std::mem::take(&mut self.gulp_now) {
            ev.push(Event::Gulp {
                at: self.player.pos + Vec3::Y * 1.2,
            });
        }
        if std::mem::take(&mut self.flask_denied_now) {
            ev.push(Event::FlaskDenied);
        }
        // 평타와 강타(0030)는 같은 규칙에 값만 다르다 — 무기 표의 타(0051)가 랩 기준값에 곱한다.
        let wpn = self.player.weapon.def();
        let s = match self.player.state {
            PState::Attack {
                t: e, swing, hit, ..
            } => {
                let Some(h) = wpn.hit(hit) else {
                    return;
                };
                let (s0, a0, _) = hit_times(t, wpn, hit);
                Strike {
                    e,
                    swing,
                    startup: s0,
                    active: a0,
                    range: t[P::AttackRange] * h.reach,
                    arc: t[P::AttackArc] * h.arc,
                    circle: h.shape == Shape::Circle,
                    damage: h.damage,
                    poise: h.poise,
                    knock: h.knock,
                    heavy: None,
                    perfect: false,
                }
            }
            PState::Heavy {
                t: e,
                swing,
                full,
                perfect,
                ..
            } => {
                let Some(h) = wpn.heavy else {
                    return;
                };
                let (s0, a0, _) = heavy_times(t, wpn);
                Strike {
                    e,
                    swing,
                    startup: s0,
                    active: a0,
                    range: t[P::HeavyRange] * h.reach,
                    arc: t[P::HeavyArc] * h.arc,
                    circle: h.shape == Shape::Circle,
                    damage: h.damage
                        + if full {
                            t[P::HeavyFullBonus].round() as i32
                        } else {
                            0
                        }
                        + perfect as i32,
                    poise: h.poise * if perfect { 1.5 } else { 1.0 },
                    knock: t[P::HeavyKnock] * if perfect { 1.5 } else { 1.0 },
                    heavy: Some(full),
                    perfect,
                }
            }
            _ => return,
        };
        let (e, swing) = (s.e, s.swing);
        if e < s.startup || e >= s.startup + s.active {
            return;
        }
        if e < s.startup + TICK_DT {
            ev.push(Event::Swing);
            if s.heavy.is_some() {
                // 망치 강타(0051): 친 자리에 금이 가고 늦게 여진, 지팡이는 딱 맞춤 번쩍.
                if let Some((delay, radius, damage, poise)) = wpn.heavy.and_then(|h| h.aftershock) {
                    self.cracks.push(Crack {
                        pos: self.player.pos,
                        left: delay,
                        radius,
                        damage,
                        poise,
                    });
                }
                if s.perfect {
                    self.player.since_perfect = 0.0;
                    ev.push(Event::Perfect {
                        at: self.player.pos + Vec3::Y * 1.0,
                    });
                }
            }
        }
        let primed = if s.heavy.is_some() {
            wpn.heavy.and_then(|h| h.breaks_primed)
        } else {
            None
        };
        let p = self.player.pos;
        let body = p.y + self.player.air;
        let f = self.player.facing_dir();
        let half = (s.arc * 0.5).to_radians().cos();
        let mut stop = false;
        let mut heavy_hit = false;
        for en in &mut self.enemies {
            if !en.active() || en.hit_by == swing {
                continue;
            }
            // 높이가 다른 층의 적에게는 칼이 안 닿는다(0043).
            if (en.pos.y - body).abs() > HIT_DY {
                continue;
            }
            let d = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z);
            let dist = d.length() - en.radius;
            if dist > s.range {
                continue;
            }
            let dir = if d.length() > 1e-4 { d.normalize() } else { f };
            if !s.circle && dir.dot(f) < half {
                continue;
            }
            en.hit_by = swing;
            // 근접으로 맞히면 마법 칸이 찬다(0048, HLD·Death's Door — 원거리만 쏘며 버티지 않게).
            self.player.mana = (self.player.mana + t[P::CastPerHit]).min(CAST_CHARGES);
            // 패링으로 드러난 적은 한 대가 치명타(0025).
            let crit = en.exposed > 0.0;
            en.hp -= hurt_by(en, self.stats.dmg + s.damage, crit, t);
            if crit {
                en.exposed = 0.0;
                ev.push(Event::Crit {
                    at: en.pos + Vec3::Y * en.height * 0.6,
                });
            }
            en.hurt = 0.15;
            en.knock = dir * t[P::Knockback] * s.knock * if en.boss { 0.25 } else { 1.0 };
            // 맞은 쪽으로 몸이 젖는다(패링과 같은 `sway`, 2026-11-06 — 흰 번쩍만으로는 "맞았다"가 아니다, KB).
            en.sway = dir
                * if s.heavy.is_some() {
                    HEAVY_SWAY
                } else {
                    HIT_SWAY
                };
            en.sway_t = 0.0;
            self.sparks.extend(sparks_at(
                en.pos + Vec3::Y * en.height * 0.55,
                dir,
                swing,
                s.heavy.is_some() || crit,
            ));
            add_poise(en, s.poise, primed, t, ev);
            ev.push(Event::Hit {
                at: en.pos + Vec3::Y * en.height * 0.6,
            });
            if let Some(full) = s.heavy {
                // 강타는 적의 상태를 바꾼다(KB: 모으기는 시간을 내준 만큼 — 휘청이 예고를 끊는다). 보스는 다 모았을 때만.
                let dur = match (en.boss, full) {
                    (true, true) => t[P::HeavyBossStagger],
                    (true, false) => 0.0,
                    (false, true) => t[P::HeavyStagger],
                    (false, false) => t[P::HeavyStagger] * 0.5,
                };
                if dur > 0.0 {
                    en.state = EState::Stagger { t: 0.0, dur };
                }
                ev.push(Event::Smash {
                    at: en.pos + Vec3::Y * en.height * 0.5,
                });
                heavy_hit = true;
            }
            stop = true;
            if en.hp <= 0 {
                en.state = EState::Dead;
                self.kills += 1;
                ev.push(Event::EnemyDown { at: en.pos });
            }
        }
        if stop {
            self.hitstop = t[P::Hitstop]
                * if heavy_hit {
                    HEAVY_HITSTOP
                } else {
                    HIT_HITSTOP
                };
            // 공격자 확인(KB, Hollow Knight 반동): 맞힌 스윙은 남은 내딛기를 멈춘다 — 빗나감과 맞힘이 손에서 다르게.
            self.player.connected = swing;
        }
    }

    /// 땅 돌리기(0012). 누른 틱에 `Event::Turn`(카메라가 돈다), 그리고 카메라가 90°에 닿아 **멈추는 순간** 쏟는다:
    /// 돌던 쟁반이 급정거하면 위의 물건은 관성으로 돌던 쪽으로 계속 간다(버스가 서면 몸이 앞으로 쏠리듯 — 사장 "시작할 때가 아니라
    /// 멈출 때 흔들리는 것"). 축(주인공)에서 멀수록 많이 미끄러진다(속도 = 각속도 × 거리), 바깥 몫(`spill.outward share`)을 섞어
    /// "쏟아지게". 범위 안의 적은 모두 휘청 — 가까운 적은 덜 밀려 바로 칠 수 있고, 먼 적은 멀리 밀려 벽에 부딪힐 수 있다. 보스는 무거워서 덜.
    fn apply_turn(&mut self, t: &Tuning, ev: &mut Vec<Event>) {
        if let Some((quarters, cw)) = self.turn_now.take() {
            ev.push(Event::Turn { quarters });
            // 도는 동안 적은 그대로 움직인다(0029: 예전엔 범위 안의 적을 실어 붙잡았다 — 1초 가까이 묶는 CC였다).
            self.spill_at = Some((cw, crate::tuning::turn_land_secs_q(t, quarters)));
            // 시작 휘청(0085): 화면이 돌기 시작하는 순간 범위 안의 적이 짧게 휘청인다(예고가 끊긴다) — 땅이 비틀리는 첫 흔들림.
            // 몸은 도는 쪽(둘레 방향)으로 젖힌다. 0029처럼 묶어 두지는 않는다(짧은 휘청 하나).
            let p = self.player.pos;
            for en in &mut self.enemies {
                if !en.active() {
                    continue;
                }
                let d = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z);
                if d.length() > t[P::SpillRange] {
                    continue;
                }
                let mass = if en.boss { t[P::SpillBossMass] } else { 1.0 };
                let dur = t[P::TurnStartStagger] / mass.max(1.0);
                if dur <= 0.0 {
                    continue;
                }
                en.state = EState::Stagger { t: 0.0, dur };
                en.sway = spill_dir(d, cw, 0.0) * 0.5;
                en.sway_t = 0.0;
            }
        }
        let Some((cw, left)) = self.spill_at else {
            return;
        };
        if left > 1e-6 {
            return;
        }
        self.spill_at = None;
        self.player.since_spill = 0.0;
        // 멈추는 순간 세상이 한 박자 멈춘다(카메라의 "탁"과 같은 틱 — KB special-move-impact "시간").
        self.hitstop = self.hitstop.max(t[P::Hitstop]);
        let p = self.player.pos;
        for en in &mut self.enemies {
            if !en.active() {
                continue;
            }
            let d = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z);
            let r = d.length();
            if r > t[P::SpillRange] {
                continue;
            }
            let mass = if en.boss { t[P::SpillBossMass] } else { 1.0 };
            let slide = (r * t[P::SpillPerM]).min(t[P::SpillMax]) / mass;
            if r > 1e-3 {
                let dir = spill_dir(d, cw, t[P::SpillOut]);
                en.knock = dir * slide * KNOCK_DECAY;
                en.spilling = slide > 0.05;
                // 몸짓 세기: 많이 미끄러질수록 크게, 가까운 적·보스도 보일 만큼은(0.35).
                let max = t[P::SpillMax].max(1e-3);
                en.sway = dir * (slide / max).clamp(0.35, 1.0);
                en.sway_t = 0.0;
            }
            let dur = if en.boss {
                t[P::SpillBossStagger]
            } else {
                t[P::SpillStagger]
            };
            en.state = if dur > 0.0 {
                EState::Stagger { t: 0.0, dur }
            } else {
                EState::Chase
            };
        }
    }

    fn tick_enemies(&mut self, t: &Tuning, dt: f32, ev: &mut Vec<Event>) {
        let p = self.player.pos;
        let body = p.y + self.player.air;
        let parrying = self.parry_open(t);
        // 길찾기(0043 → 0076): 몸 폭으로 곧게 못 가거나(벽·기둥·소품·턱) 높이가 다른 층이면 거리 지도를 따라
        // 앞으로 곧게 갈 수 있는 가장 먼 칸을 겨눈다(못 가면 그 자리에서 기다린다). 쫓는 적만 — 다른 상태는 길을 안 쓴다.
        if self.nav.as_ref().is_none_or(|n| n.stale(&self.obstacles)) {
            self.nav = Some(crate::nav::Nav::build(
                self.ground.as_ref(),
                &self.obstacles,
                self.arena,
            ));
            self.flow = None;
        }
        let nav = self.nav.as_ref().expect("built above");
        let chasing = |en: &Enemy| en.state == EState::Chase;
        if self.enemies.iter().any(chasing) {
            let cell = nav.g.cell_of(p).unwrap_or((usize::MAX, usize::MAX));
            if self.flow.as_ref().is_none_or(|(c, _)| *c != cell) {
                self.flow = Some((cell, nav.flow(p)));
            }
            let dist = &self.flow.as_ref().expect("filled above").1;
            for en in self.enemies.iter_mut().filter(|e| chasing(e)) {
                // 턱을 뛰어오르기·뛰어내리기(0100): 길의 다음 칸이 뛰어야 하는 칸이고 변 앞에 왔으면 뛴다. 문지기(보스)는 걷기만.
                if en.hop.is_none() && !en.boss {
                    if let Some((land, _)) = nav.hop(dist, en.pos, en.radius) {
                        en.hop = Some(crate::enemy::Hop::new(en.pos, land));
                        en.way = None;
                        continue;
                    }
                }
                let same =
                    nav.g.cell_of(en.pos).is_some() && nav.g.cell_of(en.pos) == nav.g.cell_of(p);
                en.way = if (body - en.pos.y).abs() <= HIT_DY
                    && (same || nav.clear(en.pos, p, en.radius))
                {
                    None
                } else {
                    Some(nav.waypoint(dist, en.pos, en.radius).unwrap_or(en.pos))
                };
            }
        }
        // 떼 퍼지기(0076): 곁(몸 사이 `SPREAD_GAP` 안)의 적에게서 비키는 쪽 — 가까울수록 세게.
        let spots: Vec<(Vec3, f32, bool)> = self
            .enemies
            .iter()
            .map(|e| (e.pos, e.radius, e.active()))
            .collect();
        for (i, en) in self.enemies.iter_mut().enumerate() {
            let mut push = Vec3::ZERO;
            for (j, &(q, r, live)) in spots.iter().enumerate() {
                if i == j || !live {
                    continue;
                }
                let d = vec3(en.pos.x - q.x, 0.0, en.pos.z - q.z);
                let range = en.radius + r + SPREAD_GAP;
                let l = d.length();
                if l < range && l > 1e-4 {
                    push += d / l * (1.0 - l / range);
                }
            }
            en.spread = push;
        }
        let facing = self.player.facing_dir();
        // 앞쪽에서 오는가(패링은 바라보는 쪽 반원 안만 — 위치 선정도 기술, KB timing-parry). 이어 패링이 쌓이면 넓어진다(0086).
        let front_cos = parry_front_cos(
            self.player.parry_chain_left > 0.0,
            self.player.parry_chain_n,
        );
        let in_front = |at: Vec3| {
            let d = vec3(at.x - p.x, 0.0, at.z - p.z).normalize_or_zero();
            d.dot(facing) > front_cos
        };
        // 플레이어를 친 적의 자리(첫 하나) — 밀려나는 쪽.
        let mut hurt_player: Option<Vec3> = None;
        let mut parried: Vec<usize> = Vec::new();
        for (i, en) in self.enemies.iter_mut().enumerate() {
            // 뛰는 중(0100): 몸은 포물선을 따라가고 생각하지도 치지도 않는다. 쓰러지면 그 자리에서 내려앉는다.
            if en.hop.is_some() && !en.alive() {
                en.hop = None;
            }
            if let Some(mut h) = en.hop {
                h.t += dt;
                let d = vec3(h.to.x - h.from.x, 0.0, h.to.z - h.from.z);
                if d.length_squared() > 1e-6 {
                    en.facing = d.x.atan2(d.z);
                }
                en.pos = h.at(h.t / h.dur);
                en.hop = if h.t >= h.dur {
                    en.pos = h.to;
                    None
                } else {
                    Some(h)
                };
                continue;
            }
            let out = en.tick(p, dt);
            // 돌진 길이(0076): 벽·기둥·턱·아레나 끝까지 — 예고 띠와 몸이 같은 길이.
            if en.atk == Atk::Charge && matches!(en.state, EState::Tele { .. }) {
                let d = en.def();
                let room = nav.charge_room(
                    &self.obstacles,
                    self.arena,
                    en.pos,
                    en.facing_dir(),
                    en.radius,
                    d.charge_speed * d.charge_time,
                );
                en.set_charge_room(room, p);
            }
            // 바닥을 훑는 파동은 높이 떠 있으면 넘는다(0027) — 높은 단 위도 마찬가지(0043).
            let above = body - en.pos.y;
            let jumped = en.atk == Atk::Wave && above > t[P::JumpClear];
            let reach = above.abs() <= HIT_DY;
            if out.hit && !jumped && reach {
                if parrying && en.parryable() && in_front(en.pos) {
                    parried.push(i);
                } else {
                    hurt_player.get_or_insert(en.pos);
                }
            }
            if out.roused {
                ev.push(Event::Rouse {
                    at: en.pos + Vec3::Y * en.height * 0.6,
                });
            }
        }
        // 곧게 오는 돌진은 닿기 조금 앞(`parry.reach m`)에서부터 받는다(nail PARRY_REACH 0258).
        if parrying {
            for (i, en) in self.enemies.iter().enumerate() {
                if let EState::Charge { dir, .. } = en.state {
                    let to = vec3(p.x - en.pos.x, 0.0, p.z - en.pos.z);
                    let near = to.length() <= en.radius + PLAYER_RADIUS + t[P::ParryReach];
                    let coming = dir.dot(to.normalize_or_zero()) > 0.5;
                    if !en.landed && near && coming && in_front(en.pos) && !parried.contains(&i) {
                        parried.push(i);
                    }
                }
            }
        }
        if !parried.is_empty() {
            for &i in &parried {
                let en = &mut self.enemies[i];
                let dur = if en.boss {
                    t[P::ParryBossStagger]
                } else {
                    t[P::ParryStagger]
                };
                let away = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z).normalize_or_zero();
                en.landed = true;
                en.state = EState::Stagger { t: 0.0, dur };
                en.exposed = dur;
                en.knock = away * PARRY_PUSH * KNOCK_DECAY * if en.boss { 0.3 } else { 1.0 };
                en.sway = away * 0.8;
                en.sway_t = 0.0;
                ev.push(Event::Parry {
                    at: en.pos + Vec3::Y * en.height * 0.6,
                });
            }
            // 튕기면 곧바로 움직일 수 있다(반격하라고) — 세상은 한 박자 멈춘다(명중보다 크게, KB).
            // 이어 패링(0081)으로 튕겼으면 그 사이 시작한 동작(회피 등)은 그대로 둔다.
            if matches!(self.player.state, PState::Parry { .. }) {
                self.player.state = PState::Idle;
            }
            // 이어 패링(0081 → 0086): 처음 튕기면 창 `PARRY_CHAIN`, 이어지는 동안 더 튕길 때마다 남은 창에 반씩 줄어드는 몫을 더하고
            // 받는 반원이 넓어진다. 한 틱에 여럿을 튕기면 하나씩 센다(둘러싸였을 때 가장 빨리 넓어진다).
            let pl = &mut self.player;
            for _ in 0..parried.len() {
                (pl.parry_chain_left, pl.parry_chain_n) =
                    parry_chain_add(pl.parry_chain_left, pl.parry_chain_n);
            }
            pl.since_parry = 0.0;
            // 튕기면 기력이 찬다(0088): 0까지 쓰고 튕겨도 반격을 바로 낼 수 있게. 한 틱에 여럿을 튕겨도 한 번.
            pl.stamina = (pl.stamina + t[P::ParryStaminaGain]).min(t[P::StaminaMax]);
            self.hitstop = self.hitstop.max(t[P::Hitstop] * PARRY_HITSTOP);
            // 같은 틱이라도 뒤·옆에서 맞은 것·받을 수 없는 파동은 그대로 맞는다(패링은 앞쪽 반원만).
        }
        if let (Some(from), false) = (hurt_player, self.player.invulnerable()) {
            let taken = 1 + self.stats.taken;
            let pl = &mut self.player;
            pl.hp -= taken;
            // 맞은 뒤 무적은 짧게(예전 0.6초 — 그 동안 다시 맞을 수 없어 경직 줄이기가 쓸 데가 없었다).
            pl.hurt = t[P::HitIframes];
            // 경직 이음(0018 → 0099, 사장 "다구리 맞더라도 틈이 생겨서 도망갈 수 있게" — 이어 패링의 반대): 경직과 그 뒤
            // `STUN_CHAIN_GRACE` 동안 또 맞으면 경직이 그전의 반 — 맞을수록 짧아져 떼 속에서도 빠져나갈 틈이 생긴다.
            (pl.stun_chain, pl.stun_window) =
                stun_chain_next(pl.stun_window, pl.stun_chain, t[P::HitStun]);
            let dur = t[P::HitStun] * 0.5f32.powi(pl.stun_chain);
            if dur > 0.0 {
                pl.state = PState::Hurt { t: 0.0, dur };
            } else if matches!(pl.state, PState::Drink { .. }) {
                // 맞으면 마시기는 끊긴다(삼키기 전이면 등잔 그대로, 0031).
                pl.state = PState::Idle;
            }
            let away = vec3(pl.pos.x - from.x, 0.0, pl.pos.z - from.z).normalize_or_zero();
            pl.vel = away * t[P::HitKnock] * KNOCK_DECAY;
            ev.push(Event::PlayerHurt);
        }
    }

    /// 겹침 풀기(적끼리·플레이어와), 기둥·아레나 경계. 쏟아지던 적이 벽에 막히면 벽 쿵(피해 + 휘청 연장, 0012).
    fn collide(&mut self, t: &Tuning, ev: &mut Vec<Event>) {
        let n = self.enemies.len();
        for i in 0..n {
            for j in (i + 1)..n {
                if !self.enemies[i].alive() || !self.enemies[j].alive() {
                    continue;
                }
                let d = self.enemies[j].pos - self.enemies[i].pos;
                let d = vec3(d.x, 0.0, d.z);
                let min = self.enemies[i].radius + self.enemies[j].radius;
                let len = d.length();
                if len < min && len > 1e-4 {
                    let push = d / len * (min - len) * 0.5;
                    self.enemies[i].pos -= push;
                    self.enemies[j].pos += push;
                }
            }
            // 회피 중에는 적을 뚫고 지나간다(0020: 떼 사이에 끼어 죽지 않게). 끝났을 때 겹쳐 있으면 다음 틱에 밀려 나온다.
            let passing = matches!(self.player.state, PState::Dodge { .. });
            if self.enemies[i].alive() && !passing {
                let d = self.player.pos - self.enemies[i].pos;
                let d = vec3(d.x, 0.0, d.z);
                let min = self.enemies[i].radius + PLAYER_RADIUS;
                let len = d.length();
                if len < min && len > 1e-4 {
                    self.player.pos += d / len * (min - len);
                }
            }
        }
        let arena = self.arena;
        let obstacles = &self.obstacles;
        let ground = self.ground.as_ref();
        // 높낮이(0043): 발 높이 `feet`에서 걸을 수 없는 칸 밖으로 민 뒤, y = 그 자리 땅 높이.
        let clamp = |p: Vec3, r: f32, feet: f32, step_up: f32, can_drop: bool| -> Vec3 {
            let mut q = p;
            for b in obstacles {
                q = b.push_circle(q, r);
            }
            if let Some(g) = ground {
                q = g.push_circle(q, r, feet, step_up, can_drop);
            }
            let q = vec3(
                q.x.clamp(-arena + r, arena - r),
                0.0,
                q.z.clamp(-arena + r, arena - r),
            );
            vec3(q.x, ground.map_or(0.0, |g| g.height(q)), q.z)
        };
        let pl = &mut self.player;
        let old = pl.pos.y;
        let feet = old + pl.air;
        let step_up = if pl.air > 0.0 { 0.0 } else { GROUND_STEP };
        pl.pos = clamp(pl.pos, PLAYER_RADIUS, feet, step_up, true);
        // 몸 높이는 이어진다: 땅이 꺼지면(뛰어내림) 그만큼 떠서 떨어지고, 단에 뛰어오르면 그 위에 선다.
        let jumping = matches!(pl.state, PState::Jump { .. });
        if jumping || pl.air > 0.0 {
            pl.air = (feet - pl.pos.y).max(0.0);
        } else if pl.pos.y < old - GROUND_STEP {
            pl.air = old - pl.pos.y;
            pl.vy = 0.0;
            if matches!(pl.state, PState::Idle | PState::Land { .. }) {
                pl.state = PState::Jump {
                    t: TICK_DT,
                    plunge: false,
                };
            }
        }
        for en in &mut self.enemies {
            // 뛰는 몸(0100)은 포물선이 자리를 정한다 — 턱·땅에 밀지 않는다.
            if en.hop.is_some() {
                continue;
            }
            let feet = en.pos.y;
            let q = clamp(en.pos, en.radius, feet, GROUND_STEP, false);
            let blocked = vec2(q.x - en.pos.x, q.z - en.pos.z).length() > 1e-3;
            en.pos = q;
            if !(blocked && en.spilling && en.alive()) {
                continue;
            }
            en.spilling = false;
            en.knock = Vec3::ZERO;
            // 몸은 부딪힌 쪽에서 되튄다.
            en.sway = -en.sway.normalize_or_zero();
            en.sway_t = 0.0;
            en.hp -= t[P::SlamDamage].round() as i32;
            en.hurt = 0.15;
            if let EState::Stagger { t: e, dur } = en.state {
                en.state = EState::Stagger {
                    t: e,
                    dur: dur + t[P::SlamStagger],
                };
            }
            ev.push(Event::Slam {
                at: en.pos + Vec3::Y * en.height * 0.5,
            });
            if en.hp <= 0 {
                en.state = EState::Dead;
                self.kills += 1;
                ev.push(Event::EnemyDown { at: en.pos });
            }
        }
    }

    fn cleanup(&mut self, ev: &mut Vec<Event>) {
        if let Some(i) = self.lock {
            if let Some(d) = self
                .enemies
                .get(i)
                .filter(|e| e.alive())
                .map(|e| e.pos.distance(self.player.pos))
            {
                if d > self.lock_lose {
                    self.lock = None;
                    ev.push(Event::LockOff);
                }
            }
        }
    }
}

/// 땅 돌리기 계획(0028): 카메라가 도는 90° 칸 수와 캐릭터의 새 요.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TurnPlan {
    /// + = 요 +90°(화면에서 땅이 시계 방향 — 왼쪽 버튼), - = 요 -90°(오른쪽 버튼). 늘 ±1.
    pub quarters: i32,
    /// 캐릭터의 새 요 = 누르기 전 캐릭터의 오른쪽(왼쪽) 면.
    pub facing: f32,
}

/// 땅 돌리기에서 몸이 오른쪽으로 도나(위에서 보아 시계 = 요가 준다) — 렌더가 `turn_r`·`turn_l`을 고른다(0034).
pub fn turn_is_right(from: f32, to: f32) -> bool {
    let mut d = (to - from) % std::f32::consts::TAU;
    if d > std::f32::consts::PI {
        d -= std::f32::consts::TAU;
    } else if d < -std::f32::consts::PI {
        d += std::f32::consts::TAU;
    }
    d < 0.0
}

/// 캐릭터 기준으로 90°(0028, 0023을 고침): 오른쪽 버튼이면 캐릭터가 제 오른쪽으로 90° 돌아서고, 화면도 같은 쪽으로 90° 돈다
/// (화면 오른쪽에 있던 것이 위로). 몸과 화면이 같은 각만큼 도니 캐릭터가 화면에서 보는 쪽은 그대로 — 쥔 스틱 그대로 새 정면으로
/// 직진한다. 0023은 "오른쪽 면이 화면 위"로 맞춰서 걷던 쪽에 따라 화면이 0°·90°·180°, 도는 쪽도 바뀌었다.
pub fn plan_turn(right: bool, facing: f32) -> TurnPlan {
    use std::f32::consts::FRAC_PI_2;
    // 요가 줄면 오른쪽으로 돈다(위에서 보아 시계): 북(π) → 동(π/2). 카메라 요 -90°도 같은 회전(화면 오른쪽 → 위).
    let (quarters, turn) = if right {
        (-1, -FRAC_PI_2)
    } else {
        (1, FRAC_PI_2)
    };
    TurnPlan {
        quarters,
        facing: angle_diff(facing + turn, 0.0),
    }
}

/// `a - b`를 -π..π로.
fn angle_diff(a: f32, b: f32) -> f32 {
    let mut d = (a - b) % std::f32::consts::TAU;
    if d > std::f32::consts::PI {
        d -= std::f32::consts::TAU;
    } else if d < -std::f32::consts::PI {
        d += std::f32::consts::TAU;
    }
    d
}

/// 한 번의 휘두르기(평타·강타, 0030).
struct Strike {
    e: f32,
    swing: u32,
    startup: f32,
    active: f32,
    range: f32,
    arc: f32,
    /// 몸 둘레 원(회전·내려찍기, 0051) — 각을 보지 않는다.
    circle: bool,
    damage: i32,
    poise: f32,
    knock: f32,
    /// 강타면 Some(다 모았나).
    heavy: Option<bool>,
    perfect: bool,
}

/// 콤보 한 타의 선딜·활성·후딜(0051: 랩 평타 × 무기 표의 `time`).
/// 이어 패링 한 번 더(0086): (남은 창, 더 튕긴 수) → 다음. 창이 닫혀 있으면 첫 튕김(창 `PARRY_CHAIN`, 수 0),
/// 열려 있으면 수 + 1, 남은 창 + `PARRY_CHAIN × PARRY_CHAIN_DECAY^수`.
pub fn parry_chain_add(left: f32, n: u32) -> (f32, u32) {
    if left <= 0.0 {
        (PARRY_CHAIN, 0)
    } else {
        let n = n + 1;
        (left + PARRY_CHAIN * PARRY_CHAIN_DECAY.powi(n as i32), n)
    }
}

/// 패링이 받는 반원의 경계(바라보는 쪽과의 내적이 이것보다 커야): 평소 `PARRY_FRONT`, 이어 패링이 열려 있으면 더 튕긴 수만큼
/// 반각을 넓힌다 — 둘레를 다 덮으면 −1 아래(뒤에서 오는 것도).
pub fn parry_front_cos(chain_open: bool, n: u32) -> f32 {
    if !chain_open || n == 0 {
        return PARRY_FRONT;
    }
    let half = PARRY_FRONT.acos() + (PARRY_CHAIN_WIDEN_DEG * n as f32).to_radians();
    if half >= std::f32::consts::PI {
        -1.01
    } else {
        half.cos()
    }
}

/// 경직 이음(0099): 남은 이음 창 `window`·지금까지 이은 수 `n`, 첫 경직 `stun` → (새 이은 수, 새 창).
/// 창이 열려 있으면 하나 더(경직 = 처음 × 0.5ⁿ, n은 `STUN_CHAIN_MAX`까지 — 바닥 = 처음의 1/4), 닫혀 있으면 처음부터.
/// 창은 이번 경직 + `STUN_CHAIN_GRACE`.
pub fn stun_chain_next(window: f32, n: i32, stun: f32) -> (i32, f32) {
    let n = if window > 0.0 {
        (n + 1).min(STUN_CHAIN_MAX)
    } else {
        0
    };
    (n, stun * 0.5f32.powi(n) + STUN_CHAIN_GRACE)
}

/// 박자 부채꼴(0087, 사장 2026-11-07 "공격시 나오는 부채꼴이 타이밍에 맞게 줄어들면, 가장 작아졌을때가 올바른 타이밍"):
/// 스윙 시작 뒤 `since`초, 부채꼴이 나타난 때 `start`(평타 = 활성 시작, 사격 = 0), 박자 `beat`.
/// (남은 몫 1 → `BEAT_CUE_MIN`, 박자 창 안인가). 나타나기 전·창이 닫힌 뒤는 없음. 박자 순간에 가장 작고, 창이 닫힐 때까지 그대로.
pub fn beat_shrink(since: f32, start: f32, beat: f32) -> Option<(f32, bool)> {
    let close = beat + (beat * BEAT_WINDOW).max(BEAT_WINDOW_MIN);
    if beat <= 0.0 || since < start || since > close {
        return None;
    }
    let p = ((since - start) / (beat - start).max(1e-3)).clamp(0.0, 1.0);
    Some((1.0 - p * (1.0 - BEAT_CUE_MIN), on_beat(since, beat)))
}

/// 박자 부채꼴 한 장(0087): 렌더가 이대로 그린다.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BeatCue {
    /// 거리(m)·각(도, 원 = 360).
    pub reach: f32,
    pub arc: f32,
    /// 칼이 지나간 쪽부터 사라진다.
    pub sweep: weapons::Sweep,
    /// 남은 몫(1 → `BEAT_CUE_MIN`).
    pub left: f32,
    /// 지금 누르면 박자(창 안).
    pub ripe: bool,
}

impl World {
    /// 지금 그릴 박자 부채꼴(0087): 다음 누름이 박자를 탈 수 있을 때만.
    /// 평타 — 이을 타가 있는 타의 활성 시작부터(마지막 타·반격·모으기·강타엔 없음, 맞거나 넘어지면 없음),
    /// 사격 — 쏜 순간부터 앞으로 좁은 부채꼴(가운데로 줄어든다).
    pub fn beat_cue(&self, t: &Tuning) -> Option<BeatCue> {
        let pl = &self.player;
        let wpn = pl.weapon.def();
        if let Rb::Shot { .. } = wpn.rb {
            let (left, ripe) = beat_shrink(pl.since_swing, 0.0, pl.beat)?;
            return Some(BeatCue {
                reach: BEAT_CUE_SHOT_REACH,
                arc: BEAT_CUE_SHOT_ARC,
                sweep: weapons::Sweep::Center,
                left,
                ripe,
            });
        }
        let live = match pl.state {
            PState::Attack { hit, .. } => hit == pl.swing_hit && wpn.after(hit).is_some(),
            PState::Idle | PState::Dodge { .. } | PState::Parry { .. } => pl.combo_left > 0.0,
            _ => false,
        };
        if !live {
            return None;
        }
        let h = wpn.hit(pl.swing_hit)?;
        let (s0, _, _) = hit_times(t, wpn, pl.swing_hit);
        let (left, ripe) = beat_shrink(pl.since_swing, s0, pl.beat)?;
        Some(BeatCue {
            reach: t[P::AttackRange] * h.reach,
            arc: if h.shape == weapons::Shape::Circle {
                360.0
            } else {
                t[P::AttackArc] * h.arc
            },
            sweep: weapons::sweep_of(h.anim),
            left,
            ripe,
        })
    }
}

/// 박자 창 안인가(2026-11-07): 마지막 평타가 시작된 뒤 `since`초, 그 무기의 박자(그 타의 한 스윙 시간) `beat` ± `BEAT_WINDOW` × 박자.
pub fn on_beat(since: f32, beat: f32) -> bool {
    beat > 0.0 && (since - beat).abs() <= (beat * BEAT_WINDOW).max(BEAT_WINDOW_MIN)
}

pub fn hit_times(t: &Tuning, w: &weapons::Weapon, hit: u8) -> (f32, f32, f32) {
    let k = w.hit(hit).map_or(1.0, |h| h.time);
    (
        t[P::AttackStartup] * k,
        t[P::AttackActive] * k,
        t[P::AttackRecovery] * k,
    )
}

/// 강타의 선딜·활성·후딜(랩 강타 × 무기 표의 `time`).
pub fn heavy_times(t: &Tuning, w: &weapons::Weapon) -> (f32, f32, f32) {
    let k = w.heavy.map_or(1.0, |h| h.time);
    (
        t[P::HeavyStartup] * k,
        t[P::HeavyActive] * k,
        t[P::HeavyRecovery] * k,
    )
}

/// 자세 막대의 크기(0051): 잡몹 = 바탕 + 체력 × 배, 보스는 따로.
pub fn poise_max(en: &Enemy, t: &Tuning) -> f32 {
    if en.boss {
        t[P::PoiseBoss]
    } else {
        t[P::PoiseBase] + en.max_hp as f32 * t[P::PoisePerHp]
    }
}

/// 플레이어가 주는 피해(0055): 드러난 적은 치명타 덤, 자세가 깨진 동안은 × `poise.break damage x`(올림) — 폭딜 창.
fn hurt_by(en: &Enemy, base: i32, crit: bool, t: &Tuning) -> i32 {
    let d = base + if crit { t[P::ParryCrit] as i32 } else { 0 };
    if en.broken > 0.0 {
        (d as f32 * t[P::PoiseBreakDamage]).ceil() as i32
    } else {
        d
    }
}

/// 자세 피해를 준다(0051). 다 차면(또는 `primed` 비율을 넘은 적을 무거운 타가 치면) 깨진다: 휘청 + 드러남(다음 한 대 치명타).
fn add_poise(en: &mut Enemy, amount: f32, primed: Option<f32>, t: &Tuning, ev: &mut Vec<Event>) {
    if amount <= 0.0 || en.poise_lock > 0.0 || !en.alive() {
        return;
    }
    let max = poise_max(en, t);
    let was_primed = primed.is_some_and(|k| en.poise >= max * k);
    en.poise += amount;
    en.poise_idle = 0.0;
    if en.poise >= max || was_primed {
        en.poise = 0.0;
        en.poise_lock = t[P::PoiseLock];
        let dur = if en.boss {
            t[P::PoiseBreakBoss]
        } else {
            t[P::PoiseBreak]
        };
        en.state = EState::Stagger { t: 0.0, dur };
        en.exposed = en.exposed.max(dur);
        en.broken = dur;
        ev.push(Event::Break {
            at: en.pos + Vec3::Y * en.height * 0.7,
        });
    }
}

/// 강타의 히트스톱 = 타격 히트스톱 × 이만큼(평타보다 무겁게, 패링보다는 짧게).
const HEAVY_HITSTOP: f32 = 2.0;

/// 내려찍기는 이만큼 빨리 떨어진다(공중에서 누르면 곧장 내리꽂게).
const PLUNGE_FALL: f32 = 2.5;

/// 패링: 바라보는 쪽과 적 쪽의 내적이 이보다 커야 막는다(≈ 앞 156°).
const PARRY_FRONT: f32 = 0.2;
/// 떼 퍼지기(0076): 몸 사이가 이만큼(m) 안이면 서로 비킨다.
const SPREAD_GAP: f32 = 0.9;
/// 튕긴 적을 미는 거리(m).
const PARRY_PUSH: f32 = 0.8;
/// 패링 성공의 히트스톱 = 타격 히트스톱 × 이만큼(명중보다 크게).
const PARRY_HITSTOP: f32 = 2.5;

/// 이보다 빠르게(m/s) 움직이던 중이면 입력을 놓았어도 그 쪽으로 내딛는다.
const LUNGE_MOVING: f32 = 1.0;

/// 조준 스틱이 이보다 세면 "겨누는 중".
const AIM_STICK: f32 = 0.35;
/// 커서 점이 플레이어에게서 이만큼(m) 안이면 겨누지 않는다(0058 — 발밑을 가리키면 방향이 마구 돈다).
const CURSOR_DEAD_M: f32 = 0.5;

/// `reach` 안의 가장 가까운 산 적 쪽(XZ 단위). `cone`이 있으면 (방향, cos 반각) 안의 적만.
fn nearest_in_reach(
    enemies: &[Enemy],
    from: Vec3,
    reach: f32,
    cone: Option<(Vec3, f32)>,
) -> Option<Vec3> {
    enemies
        .iter()
        .filter(|e| e.active())
        .filter_map(|e| {
            let d = vec3(e.pos.x - from.x, 0.0, e.pos.z - from.z);
            let dist = d.length() - e.radius;
            let dir = d.normalize_or_zero();
            let ok =
                dist <= reach && dir != Vec3::ZERO && cone.is_none_or(|(c, cos)| dir.dot(c) >= cos);
            ok.then_some((dist, dir))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, d)| d)
}

/// 넉백 속도가 초당 이만큼씩 줄어든다(틱마다 `1 - k·dt`) — 그래서 처음 속도 v의 넉백은 모두 합쳐 v/k만큼 민다.
pub const KNOCK_DECAY: f32 = 10.0;

/// 쏟아지는 방향(XZ 단위). `d` = 축(주인공) → 적. 화면 시계 방향 돌리기의 접선은 (−d.z, d.x)라(카메라 요 +90°와 같은 회전),
/// 멈출 때 관성으로 계속 가는 쪽이 그 접선. 반시계면 반대. 바깥 몫 `out`을 섞는다.
pub fn spill_dir(d: Vec3, cw: bool, out: f32) -> Vec3 {
    let r = vec3(d.x, 0.0, d.z).length().max(1e-6);
    let n = vec3(d.x, 0.0, d.z) / r;
    let along = if cw {
        vec3(-n.z, 0.0, n.x)
    } else {
        vec3(n.z, 0.0, -n.x)
    };
    (along * (1.0 - out) + n * out).normalize_or_zero()
}

/// 락온이 풀린 뒤 이만큼보다 세게 쥐고 있으면 "쥔 그대로"로 본다(놓으면 쿼터뷰 기준).
const HOLD_STICK: f32 = 0.3;

/// 스틱 벡터(x 오른쪽, y 앞)를 월드 이동 벡터로. 쿼터뷰: 화면 위 = -z. 락온: 앞 = 대상 쪽, 오른쪽 = 그 둘레.
pub fn move_to_world(stick: Vec2, basis: Basis, player: Vec3) -> Vec3 {
    if stick.length_squared() < 1e-6 {
        return Vec3::ZERO;
    }
    let (fwd, right) = match basis {
        Basis::Quarter { yaw } => {
            let f = crate::camera::quarter_forward(yaw);
            (f, f.cross(Vec3::Y))
        }
        Basis::Frozen(f) => (f, f.cross(Vec3::Y)),
        Basis::Lock(tg) => {
            let d = vec3(tg.x - player.x, 0.0, tg.z - player.z);
            let f = if d.length_squared() > 1e-4 {
                d.normalize()
            } else {
                vec3(0.0, 0.0, -1.0)
            };
            (f, f.cross(Vec3::Y))
        }
    };
    right * stick.x + fwd * stick.y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn charms_raise_the_health_cap_and_make_hits_hurt_more() {
        use crate::charms::{def, Id};
        use crate::stats::Stats;
        let mut w = World::new("t", 20.0);
        // 단단한 껍질 + 가벼운 발: 최대 체력 +1, 받는 피해 +1.
        w.apply_stats(
            Stats::compute(&[def(Id::Shell).mods, def(Id::Feet).mods]),
            true,
        );
        assert_eq!(w.player.max_hp, crate::tuning::PLAYER_HP + 1);
        assert_eq!(w.player.hp, w.player.max_hp, "refilled");
        // 장신구를 빼면 체력은 새 최대를 넘지 않는다.
        w.apply_stats(Stats::default(), false);
        assert_eq!(w.player.hp, crate::tuning::PLAYER_HP);
        // 받는 피해: 같은 한 대가 1 대신 2.
        w.apply_stats(Stats::compute(&[def(Id::Feet).mods]), true);
        let before = w.player.hp;
        w.spawn(crate::species::Kind::Husk, vec3(0.6, 0.0, 0.0));
        let t = crate::tuning::Tuning::default();
        for _ in 0..240 {
            w.step(&Actions::default(), &t);
            if w.player.hp < before {
                break;
            }
        }
        assert_eq!(before - w.player.hp, 2);
    }

    #[test]
    fn a_fallen_enemy_drops_its_embers_once_and_they_come_to_the_player() {
        let mut w = World::new("t", 20.0);
        w.player.pos = Vec3::ZERO;
        let i = w.spawn(crate::species::Kind::Thorn, vec3(3.0, 0.0, 0.0));
        w.enemies[i].hp = 0;
        w.enemies[i].state = crate::enemy::EState::Dead;
        let t = crate::tuning::Tuning::default();
        let a = Actions::default();
        let mut picked = 0;
        for _ in 0..180 {
            for e in w.step(&a, &t) {
                if let Event::Ember { value } = e {
                    picked += value;
                }
            }
        }
        let want = crate::species::def(crate::species::Kind::Thorn).embers;
        assert_eq!(picked, want, "every piece reaches the player");
        assert_eq!(w.embers, want);
        assert!(w.drops.is_empty());
        assert!(w.enemies[i].dropped, "dropped once, not every tick");
    }

    fn world_with_boss() -> World {
        let mut w = World::new("test", 20.0);
        w.spawn_boss(vec3(0.0, 0.0, -6.0));
        w
    }

    #[test]
    fn quarter_basis_moves_up_toward_minus_z_and_lock_basis_toward_target() {
        let m = move_to_world(vec2(0.0, 1.0), Basis::Quarter { yaw: 0.0 }, Vec3::ZERO);
        assert!(m.z < -0.99 && m.x.abs() < 1e-6);
        let m = move_to_world(vec2(0.0, 1.0), Basis::Lock(vec3(5.0, 0.0, 0.0)), Vec3::ZERO);
        assert!(m.x > 0.99, "forward is toward the target: {m:?}");
        let r = move_to_world(
            vec2(1.0, 0.0),
            Basis::Lock(vec3(0.0, 0.0, -5.0)),
            Vec3::ZERO,
        );
        assert!(r.x > 0.99, "right of a target in -z is +x: {r:?}");
    }

    #[test]
    fn quarter_basis_follows_the_camera_yaw_so_stick_up_is_screen_up() {
        // 요 90°: 카메라가 +x 쪽에서 본다 → 화면 위 = -x.
        let m = move_to_world(vec2(0.0, 1.0), Basis::Quarter { yaw: 90.0 }, Vec3::ZERO);
        assert!(m.x < -0.99 && m.z.abs() < 1e-4, "{m:?}");
        let r = move_to_world(vec2(1.0, 0.0), Basis::Quarter { yaw: 90.0 }, Vec3::ZERO);
        assert!(
            r.z < -0.99,
            "right of screen is -z when looking from +x: {r:?}"
        );
    }

    #[test]
    fn lock_button_toggles_and_the_basis_switches_the_same_tick() {
        let t = Tuning::default();
        let mut w = world_with_boss();
        let ev = w.step(
            &Actions {
                lock: true,
                ..Default::default()
            },
            &t,
        );
        assert!(ev.contains(&Event::LockOn));
        assert_eq!(w.basis(), Basis::Lock(w.enemies[0].pos));
        let ev = w.step(
            &Actions {
                lock: true,
                ..Default::default()
            },
            &t,
        );
        assert!(ev.contains(&Event::LockOff));
        assert!(matches!(w.basis(), Basis::Quarter { .. }));
    }

    #[test]
    fn releasing_lock_while_holding_the_stick_keeps_walking_the_same_way_until_the_stick_is_let_go()
    {
        let t = Tuning::default();
        let mut w = world_with_boss();
        w.player.pos = vec3(0.0, 0.0, -14.0); // 보스(0,-8) 위쪽 — 쿼터뷰 기준과 락온 기준이 거의 반대
        w.step(
            &Actions {
                lock: true,
                ..Default::default()
            },
            &t,
        );
        let back = Actions {
            move_dir: vec2(0.0, -1.0),
            ..Default::default()
        }; // 보스에게서 물러나기
        for _ in 0..30 {
            w.step(&back, &t);
        }
        let before = w.player.vel.normalize();
        // 쥔 채로 풀기: 걷는 방향이 그대로.
        w.step(&Actions { lock: true, ..back }, &t);
        for _ in 0..5 {
            w.step(&back, &t);
        }
        assert!(w.lock.is_none());
        assert!(
            w.player.vel.normalize().dot(before) > 0.98,
            "{:?} vs {before:?}",
            w.player.vel
        );
        // 놓았다 다시 쥐면 쿼터뷰 기준(화면 아래 = 반대쪽).
        w.step(&Actions::default(), &t);
        for _ in 0..30 {
            w.step(&back, &t);
        }
        let quarter_down = move_to_world(
            vec2(0.0, -1.0),
            Basis::Quarter { yaw: t[P::CamQYaw] },
            w.player.pos,
        );
        assert!(w.player.vel.normalize().dot(quarter_down) > 0.98);
    }

    #[test]
    fn the_unlock_hold_ends_by_itself_after_the_hold_time() {
        let t = Tuning::default();
        let mut w = world_with_boss();
        w.player.pos = vec3(0.0, 0.0, -14.0);
        w.step(
            &Actions {
                lock: true,
                ..Default::default()
            },
            &t,
        );
        let back = Actions {
            move_dir: vec2(0.0, -1.0),
            ..Default::default()
        };
        w.step(&back, &t);
        w.step(&Actions { lock: true, ..back }, &t);
        let ticks = (t[P::MoveHoldMax] / TICK_DT).ceil() as usize + 2;
        for _ in 0..ticks {
            w.step(&back, &t);
        }
        assert!(w.hold.is_none());
    }

    #[test]
    fn a_swing_hits_once_per_enemy_and_triggers_hitstop() {
        let t = Tuning::default();
        let mut w = World::new("test", 20.0);
        w.spawn_mob(vec3(0.0, 0.0, -1.0));
        let mut hits = 0;
        let mut a = Actions {
            attack: true,
            ..Default::default()
        };
        for _ in 0..60 {
            let ev = w.step(&a, &t);
            a.attack = false;
            hits += ev.iter().filter(|e| matches!(e, Event::Hit { .. })).count();
        }
        assert_eq!(hits, 1);
        assert_eq!(w.enemies[0].hp, w.enemies[0].max_hp - 1);
    }

    #[test]
    fn lock_drops_when_the_target_dies() {
        let t = Tuning::default();
        let mut w = World::new("test", 20.0);
        w.spawn_mob(vec3(0.0, 0.0, -1.0));
        w.step(
            &Actions {
                lock: true,
                ..Default::default()
            },
            &t,
        );
        assert!(w.lock.is_some());
        w.enemies[0].state = EState::Dead;
        let ev = w.step(&Actions::default(), &t);
        assert!(ev.contains(&Event::LockOff) && w.lock.is_none());
    }

    #[test]
    fn dodging_player_is_not_hurt_by_a_boss_swing() {
        let t = Tuning::default();
        let mut w = world_with_boss();
        w.enemies[0].pos = vec3(0.0, 0.0, -2.0);
        w.enemies[0].facing = 0.0; // +z = 플레이어 쪽
        w.enemies[0].state = EState::Swing { t: 0.0 };
        w.player.state = PState::Dodge {
            t: 0.0,
            dir: vec3(1.0, 0.0, 0.0),
        };
        let ev = w.step(&Actions::default(), &t);
        assert!(!ev.contains(&Event::PlayerHurt));
        assert_eq!(w.player.hp, PLAYER_HP);
    }

    /// 0027: 점프는 체공 시간 뒤에 내려앉고(포물선), 스태미나를 쓰고, 공중에서 또 뛸 수 없다.
    #[test]
    fn a_jump_rises_and_lands_after_its_air_time() {
        let t = Tuning::default();
        let mut w = World::new("test", 20.0);
        let st = w.player.stamina;
        w.step(
            &Actions {
                jump: true,
                ..Default::default()
            },
            &t,
        );
        assert!(w.player.airborne());
        assert!(w.player.stamina < st);
        let mut peak = 0.0f32;
        let mut ticks = 1;
        while w.player.airborne() && ticks < 200 {
            // 공중에서 또 눌러도 다시 뛰지 않는다.
            w.step(
                &Actions {
                    jump: true,
                    ..Default::default()
                },
                &t,
            );
            peak = peak.max(w.player.air);
            ticks += 1;
        }
        let air = ticks as f32 * TICK_DT;
        assert!((peak - t[P::JumpHeight]).abs() < 0.05, "peak {peak}");
        assert!((air - t[P::JumpTime]).abs() < 0.05, "air {air}s");
        assert!(matches!(w.player.state, PState::Land { .. }));
        assert_eq!(w.player.air, 0.0);
    }

    #[test]
    fn a_running_jump_carries_its_speed() {
        let t = Tuning::default();
        let mut w = World::new("test", 30.0);
        w.view_yaw = Some(0.0);
        let run = Actions {
            move_dir: vec2(0.0, 1.0),
            ..Default::default()
        };
        for _ in 0..60 {
            w.step(&run, &t);
        }
        let v = w.player.vel.length();
        let z0 = w.player.pos.z;
        w.step(&Actions { jump: true, ..run }, &t);
        // 손을 놓아도 공중에서는 거의 그대로 간다.
        while w.player.airborne() {
            w.step(&Actions::default(), &t);
        }
        let flew = z0 - w.player.pos.z;
        assert!(flew > v * t[P::JumpTime] * 0.6, "flew {flew} m at {v} m/s");
    }

    /// 높이 떠 있으면 바닥을 훑는 뿌리 파동을 넘는다(엘든 링: 점프로 낮은 공격 피하기). 땅에 있으면 맞는다.
    #[test]
    fn a_high_jump_clears_the_root_wave() {
        let t = Tuning::default();
        for jump in [true, false] {
            let mut w = world_with_boss();
            let en = &mut w.enemies[0];
            en.pos = vec3(0.0, 0.0, -3.0);
            en.atk = Atk::Wave;
            en.state = EState::Wave { t: 0.0 };
            en.landed = false;
            let mut hurt = false;
            for i in 0..40 {
                // 앞(-3m)에서 퍼지는 파동이 닿기 조금 전(9 m/s → 약 0.3초)에 뛴다.
                let a = Actions {
                    jump: jump && i == 8,
                    ..Default::default()
                };
                hurt |= w.step(&a, &t).contains(&Event::PlayerHurt);
            }
            assert_eq!(hurt, !jump, "jump {jump}");
        }
    }

    /// 공중에서 공격 = 내려찍기: 내려앉는 순간 둘레의 적을 `plunge.damage`만큼.
    #[test]
    fn a_plunge_hits_everyone_around_the_landing_spot() {
        let t = Tuning::default();
        let mut w = World::new("test", 20.0);
        w.spawn_mob(vec3(1.0, 0.0, 0.0));
        w.spawn_mob(vec3(-1.0, 0.0, 0.3));
        w.spawn_mob(vec3(0.0, 0.0, 6.0)); // 멀다
        idle(&mut w);
        w.step(
            &Actions {
                jump: true,
                ..Default::default()
            },
            &t,
        );
        w.step(
            &Actions {
                attack: true,
                ..Default::default()
            },
            &t,
        );
        let mut ev = Vec::new();
        for _ in 0..120 {
            idle(&mut w);
            ev.extend(w.step(&Actions::default(), &t));
        }
        assert!(ev.iter().any(|e| matches!(e, Event::Plunge { .. })));
        let dmg = t[P::PlungeDamage] as i32;
        assert_eq!(w.enemies[0].hp, w.enemies[0].max_hp - dmg);
        assert_eq!(w.enemies[1].hp, w.enemies[1].max_hp - dmg);
        assert_eq!(w.enemies[2].hp, w.enemies[2].max_hp, "out of range");
    }

    /// 패링 시험용: 잡몹 하나를 플레이어 바로 앞(-z)에 두고 내려치기를 막 휘두르게.
    fn mob_swinging_in_front(t: &Tuning) -> World {
        let mut w = World::new("test", 20.0);
        w.spawn_mob(vec3(0.0, 0.0, -1.2));
        let en = &mut w.enemies[0];
        en.facing = 0.0; // +z = 플레이어 쪽
        en.atk = Atk::Smash;
        en.state = EState::Tele { t: en.tele_dur() };
        w.player.facing = std::f32::consts::PI; // -z = 적 쪽
        let _ = t;
        w
    }

    /// 0025: 창 안에 앞에서 오는 근접 공격은 튕긴다 — 피해 없음, 적은 휘청·드러남, 플레이어는 바로 풀린다.
    #[test]
    fn a_parry_in_the_window_deflects_a_blow_from_the_front() {
        let t = Tuning::default();
        let mut w = mob_swinging_in_front(&t);
        let mut ev = w.step(
            &Actions {
                parry: true,
                ..Default::default()
            },
            &t,
        );
        for _ in 0..30 {
            if ev.iter().any(|e| matches!(e, Event::Parry { .. })) {
                break;
            }
            ev = w.step(&Actions::default(), &t);
        }
        assert!(
            ev.iter().any(|e| matches!(e, Event::Parry { .. })),
            "parried"
        );
        assert_eq!(w.player.hp, PLAYER_HP);
        assert!(matches!(w.enemies[0].state, EState::Stagger { .. }));
        assert!(w.enemies[0].exposed > 0.0);
        assert_eq!(w.player.state, PState::Idle, "free to riposte");
    }

    /// 2026-11-06: 달리다 패링하면 그 자리에서 0이 아니라 걸음을 멈출 때의 감속으로 선다(뒤로 출렁이지 않게) — 앞으로만 조금 미끄러진다.
    #[test]
    fn a_parry_from_a_run_brakes_like_a_stop_not_a_wall() {
        let t = Tuning::default();
        let mut w = World::new("test", 20.0);
        w.player.vel = vec3(0.0, 0.0, -5.0);
        w.player.state = PState::Parry { t: 0.0 };
        let z0 = w.player.pos.z;
        w.step(&Actions::default(), &t);
        assert!(
            w.player.vel.z < -1.0,
            "still moving the first tick: {:?}",
            w.player.vel
        );
        for _ in 0..20 {
            w.step(&Actions::default(), &t);
        }
        assert!(
            w.player.vel.length() < 1e-4,
            "stopped within a third of a second"
        );
        let slid = z0 - w.player.pos.z;
        assert!(slid > 0.1 && slid < 0.6, "slides forward a little: {slid}");
    }

    #[test]
    fn a_parry_does_not_cover_your_back_or_a_root_wave() {
        let t = Tuning::default();
        // 등 뒤.
        let mut w = mob_swinging_in_front(&t);
        w.player.facing = 0.0; // 적을 등진다
        w.player.state = PState::Parry { t: 0.0 };
        let mut hurt = false;
        for _ in 0..30 {
            hurt |= w.step(&Actions::default(), &t).contains(&Event::PlayerHurt);
        }
        assert!(hurt, "a blow from behind lands");
        // 파동(못 받는 공격).
        let mut w = world_with_boss();
        let en = &mut w.enemies[0];
        en.pos = vec3(0.0, 0.0, -2.0);
        en.atk = Atk::Wave;
        en.state = EState::Wave { t: 0.0 };
        en.landed = false;
        w.player.facing = std::f32::consts::PI;
        let mut hurt = false;
        for _ in 0..60 {
            if !matches!(w.player.state, PState::Parry { .. } | PState::Hurt { .. }) {
                w.player.state = PState::Parry { t: 0.0 };
            }
            hurt |= w.step(&Actions::default(), &t).contains(&Event::PlayerHurt);
        }
        assert!(hurt, "the root wave cannot be parried");
    }

    /// 헛치면 `parry.time s`까지 묶인다(연타가 공짜가 되지 않게) — 스태미나도 쓴다.
    #[test]
    fn a_whiffed_parry_roots_you_and_costs_stamina() {
        let t = Tuning::default();
        let mut w = World::new("test", 20.0);
        let before = w.player.stamina;
        w.step(
            &Actions {
                parry: true,
                ..Default::default()
            },
            &t,
        );
        assert!(matches!(w.player.state, PState::Parry { .. }));
        assert!(w.player.stamina < before);
        let mv = Actions {
            move_dir: vec2(0.0, 1.0),
            ..Default::default()
        };
        let start = w.player.pos;
        for _ in 0..((t[P::ParryTime] / TICK_DT) as usize - 2) {
            w.step(&mv, &t);
        }
        assert!(
            (w.player.pos - start).length() < 1e-4,
            "rooted while recovering"
        );
        for _ in 0..6 {
            w.step(&mv, &t);
        }
        assert_eq!(w.player.state, PState::Idle);
    }

    /// 곧게 오는 돌진은 몸에 닿기 전(`parry.reach m`)에 받는다.
    #[test]
    fn a_charge_is_parried_just_before_it_lands() {
        let t = Tuning::default();
        let mut w = World::new("test", 20.0);
        w.spawn(Kind::Thorn, vec3(0.0, 0.0, -3.0));
        let en = &mut w.enemies[0];
        en.atk = Atk::Charge;
        en.state = EState::Charge {
            t: 0.0,
            dir: vec3(0.0, 0.0, 1.0),
        };
        en.facing = 0.0;
        w.player.facing = std::f32::consts::PI;
        let mut parried = false;
        for _ in 0..40 {
            let near = (w.enemies[0].pos - w.player.pos).length()
                < 0.45 + PLAYER_RADIUS + t[P::ParryReach];
            let a = Actions {
                parry: near && !matches!(w.player.state, PState::Parry { .. }),
                ..Default::default()
            };
            let ev = w.step(&a, &t);
            if ev.iter().any(|e| matches!(e, Event::Parry { .. })) {
                parried = true;
                break;
            }
        }
        assert!(parried);
        assert_eq!(w.player.hp, PLAYER_HP);
    }

    /// 드러난 적에게는 한 대가 치명타(+`parry.crit`), 그 뒤엔 보통.
    #[test]
    fn an_exposed_enemy_takes_a_critical_hit_once() {
        let t = Tuning::default();
        let mut w = World::new("test", 20.0);
        w.spawn_boss(vec3(0.0, 0.0, -1.5));
        idle(&mut w);
        w.enemies[0].exposed = 1.0;
        let hp = w.enemies[0].hp;
        let mut ev = w.step(
            &Actions {
                attack: true,
                ..Default::default()
            },
            &t,
        );
        for _ in 0..30 {
            ev.extend(w.step(&Actions::default(), &t));
        }
        assert!(ev.iter().any(|e| matches!(e, Event::Crit { .. })));
        assert_eq!(w.enemies[0].hp, hp - 1 - t[P::ParryCrit] as i32);
        assert_eq!(w.enemies[0].exposed, 0.0);
    }

    /// 0034: 오른쪽 버튼의 몸 돌기는 `turn_is_right`(렌더가 `turn_r`을 고른다), 왼쪽은 아니다.
    #[test]
    fn the_turn_animation_side_follows_the_button() {
        use std::f32::consts::PI;
        for facing in [PI, 0.0, 1.0, -2.5] {
            let r = plan_turn(true, facing);
            assert!(turn_is_right(facing, r.facing), "{facing}");
            let l = plan_turn(false, facing);
            assert!(!turn_is_right(facing, l.facing), "{facing}");
        }
    }

    /// 0028: 캐릭터 기준 90° — 오른쪽 버튼이면 몸이 제 오른쪽으로 90°, 화면도 늘 오른쪽으로 90°(요 -90°). 바라보는 쪽과 무관.
    #[test]
    fn a_turn_is_always_ninety_degrees_the_buttons_way_for_body_and_screen() {
        use std::f32::consts::{FRAC_PI_2, PI};
        let close = |a: f32, b: f32| angle_diff(a, b).abs() < 1e-3;
        for facing in [PI, 0.0, FRAC_PI_2, -FRAC_PI_2, PI - 0.5, 0.7] {
            let r = plan_turn(true, facing);
            assert_eq!(r.quarters, -1, "{facing}");
            assert!(close(r.facing, facing - FRAC_PI_2), "{facing}: {r:?}");
            let l = plan_turn(false, facing);
            assert_eq!(l.quarters, 1, "{facing}");
            assert!(close(l.facing, facing + FRAC_PI_2), "{facing}: {l:?}");
        }
        // 북(π)을 보고 오른쪽 → 동쪽을 보고, 화면이 오른쪽으로 90°라 동쪽이 화면 위.
        assert!(close(plan_turn(true, PI).facing, FRAC_PI_2));
        assert!(crate::camera::quarter_forward(-90.0).dot(vec3(1.0, 0.0, 0.0)) > 0.999);
    }

    #[test]
    fn turning_turns_the_body_to_the_new_front() {
        let t = Tuning::default();
        let mut w = World::new("test", 20.0);
        w.view_yaw = Some(0.0);
        w.player.facing = 0.0; // 화면 아래(카메라 쪽)를 본다
        let ev = w.step(
            &Actions {
                turn_right: true,
                ..Default::default()
            },
            &t,
        );
        assert!(ev.contains(&Event::Turn { quarters: -1 }), "{ev:?}");
        for _ in 0..((t[P::TurnPlant] / TICK_DT).ceil() as usize + 2) {
            w.step(&Actions::default(), &t);
        }
        // 캐릭터의 오른쪽이던 서쪽(-x)을 본다.
        assert!(
            w.player.facing_dir().dot(vec3(-1.0, 0.0, 0.0)) > 0.999,
            "{}",
            w.player.facing
        );
    }

    #[test]
    fn a_dodge_passes_through_enemies_but_walking_does_not() {
        let t = Tuning::default();
        let mut w = World::new("test", 20.0);
        w.view_yaw = Some(0.0);
        w.spawn_mob(vec3(0.0, 0.0, -1.5));
        idle(&mut w);
        let a = Actions {
            move_dir: vec2(0.0, 1.0),
            dodge: true,
            ..Default::default()
        };
        w.step(&a, &t);
        for _ in 0..30 {
            w.step(
                &Actions {
                    move_dir: vec2(0.0, 1.0),
                    ..Default::default()
                },
                &t,
            );
            idle(&mut w);
        }
        assert!(
            w.player.pos.z < -1.5 - 0.3,
            "dodged through: z {}",
            w.player.pos.z
        );
        // 걸어서는 못 지나간다.
        let mut w = World::new("test", 20.0);
        w.view_yaw = Some(0.0);
        w.spawn_mob(vec3(0.0, 0.0, -1.5));
        for _ in 0..60 {
            w.step(
                &Actions {
                    move_dir: vec2(0.0, 1.0),
                    ..Default::default()
                },
                &t,
            );
            idle(&mut w);
        }
        assert!(
            w.player.pos.z > -1.5,
            "walking is blocked: z {}",
            w.player.pos.z
        );
    }

    /// 기본 자세(화면 위를 봄)에서 카메라가 `cw`(요 +90°) 쪽으로 돌게 하는 버튼 — 위를 보면 왼쪽 버튼이 요 +90°다(0023).
    fn turn(cw: bool) -> Actions {
        Actions {
            turn_left: cw,
            turn_right: !cw,
            ..Default::default()
        }
    }

    /// 시험용: 적이 쫓지도 치지도 않게 세워 둔다.
    fn idle(w: &mut World) {
        for e in &mut w.enemies {
            e.state = EState::Recover { t: -1000.0 };
        }
    }

    /// 돌리고 멈추는(쏟는) 틱까지.
    fn turn_until_spill(w: &mut World, cw: bool, t: &Tuning) -> Vec<Event> {
        let mut ev = w.step(&turn(cw), t);
        // 시작 휘청(0085)이 세워 둔 적을 깨우지 않게 — 이 시험들은 멈춘 적으로 쏟기만 본다.
        idle(w);
        for _ in 0..600 {
            if w.spill_at.is_none() {
                break;
            }
            ev.extend(w.step(&Actions::default(), t));
        }
        ev
    }

    /// 화면 시계 방향(요 35°, 화면 위 = `quarter_forward(35)`): 돌다가 **멈추는 순간** 화면 위에 선 적이 돌던 쪽(화면 오른쪽)으로
    /// 쏟아진다(급정거 — 관성으로 계속 간다). 도는 동안엔 그대로.
    #[test]
    fn turning_spills_enemies_along_the_turn_when_it_stops_and_farther_ones_slide_more() {
        let t = Tuning::default();
        let mut w = World::new("test", 30.0);
        w.view_yaw = Some(35.0);
        let up = crate::camera::quarter_forward(35.0);
        let right = up.cross(Vec3::Y);
        w.spawn_mob(up * 3.0);
        w.spawn_mob(up * 7.0);
        idle(&mut w);
        let ev = w.step(&turn(true), &t);
        assert!(ev.contains(&Event::Turn { quarters: 1 }));
        idle(&mut w); // 시작 휘청(0085)이 세워 둔 적을 깨우지 않게
                      // 도는 동안(멈추기 전)엔 아무도 밀리지 않는다.
        let half = (crate::tuning::turn_land_secs_q(&t, 1) * 0.5 / TICK_DT) as usize;
        for _ in 0..half {
            w.step(&Actions::default(), &t);
        }
        assert!(!matches!(w.enemies[1].state, EState::Stagger { .. }));
        for _ in 0..600 {
            if w.spill_at.is_none() {
                break;
            }
            w.step(&Actions::default(), &t);
        }
        assert!(matches!(w.enemies[1].state, EState::Stagger { .. }));
        // 0029: 밀림은 조금(먼 적도 `spill.slide max m`까지) — 휘청(0.3초)이 끝나기 전에 잰다.
        let (near0, far0) = (w.enemies[0].pos, w.enemies[1].pos);
        for _ in 0..15 {
            w.step(&Actions::default(), &t);
        }
        let near = w.enemies[0].pos - near0;
        let far = w.enemies[1].pos - far0;
        assert!(
            near.dot(right) > 0.05 && far.dot(right) > 0.1,
            "{near:?} {far:?}"
        );
        assert!(far.length() > near.length() * 1.5, "{near:?} {far:?}");
        assert!(far.length() <= t[P::SpillMax] + 1e-3, "{far:?}");
        // 반시계면 반대쪽.
        let mut w2 = World::new("test", 30.0);
        w2.view_yaw = Some(35.0);
        w2.spawn_mob(up * 7.0);
        idle(&mut w2);
        turn_until_spill(&mut w2, false, &t);
        let at = w2.enemies[0].pos;
        for _ in 0..15 {
            w2.step(&Actions::default(), &t);
        }
        assert!((w2.enemies[0].pos - at).dot(right) < -0.1);
    }

    #[test]
    fn turning_staggers_everyone_in_range_when_it_stops_and_cuts_their_telegraph() {
        let t = Tuning::default();
        let mut w = World::new("test", 30.0);
        w.spawn_mob(vec3(0.0, 0.0, -1.2));
        w.spawn_mob(vec3(0.0, 0.0, t[P::SpillRange] + 6.0));
        idle(&mut w);
        w.step(&turn(true), &t);
        while w.spill_at.is_some_and(|(_, left)| left > 2.0 * TICK_DT) {
            w.step(&Actions::default(), &t);
        }
        w.enemies[0].state = EState::Tele { t: 0.4 };
        for _ in 0..600 {
            if w.spill_at.is_none() {
                break;
            }
            w.step(&Actions::default(), &t);
        }
        assert!(matches!(w.enemies[0].state, EState::Stagger { .. }));
        assert!(!matches!(w.enemies[1].state, EState::Stagger { .. }));
        // 휘청 동안은 치지 못한다(가까이 붙어 있어도).
        let mut hurt = false;
        for _ in 0..((t[P::SpillStagger] / TICK_DT) as usize - 2) {
            hurt |= w.step(&Actions::default(), &t).contains(&Event::PlayerHurt);
        }
        assert!(!hurt);
    }

    /// 0029: 도는 동안 적을 붙잡지 않는다 — 그대로 쫓아온다. 멈출 때 범위 안이면 짧게 휘청할 뿐.
    #[test]
    fn while_the_ground_turns_the_enemies_keep_coming() {
        let t = Tuning::default();
        let mut w = World::new("test", 30.0);
        w.spawn_mob(vec3(0.0, 0.0, -5.0));
        w.step(&turn(true), &t);
        while w.spill_at.is_some_and(|(_, left)| left > TICK_DT) {
            w.step(&Actions::default(), &t);
        }
        assert!(
            w.enemies[0].pos.z > -5.0 + 1.0,
            "closed in: {:?}",
            w.enemies[0].pos
        );
        for _ in 0..3 {
            w.step(&Actions::default(), &t);
        }
        assert!(matches!(w.enemies[0].state, EState::Stagger { .. }));
        // 휘청은 짧다.
        for _ in 0..((t[P::SpillStagger] / TICK_DT) as usize + 2) {
            w.step(&Actions::default(), &t);
        }
        assert!(!matches!(w.enemies[0].state, EState::Stagger { .. }));
    }

    #[test]
    fn the_boss_is_heavy_it_barely_slides_and_staggers_briefly() {
        let t = Tuning::default();
        let mut w = World::new("test", 30.0);
        w.spawn_boss(vec3(0.0, 0.0, -6.0));
        w.spawn_mob(vec3(0.0, 0.0, 6.0));
        idle(&mut w);
        turn_until_spill(&mut w, true, &t);
        // 멈춘 자리에서 잰다(도는 동안 둘 다 쫓아왔다). 보스 휘청(0.1초)이 끝나기 전에.
        let (b0, m0) = (w.enemies[0].pos, w.enemies[1].pos);
        for _ in 0..5 {
            w.step(&Actions::default(), &t);
        }
        let boss = (w.enemies[0].pos - b0).length();
        let mob = (w.enemies[1].pos - m0).length();
        assert!(boss * 3.0 < mob, "boss {boss} mob {mob}");
        for _ in 0..8 {
            w.step(&Actions::default(), &t);
        }
        let EState::Stagger { .. } = w.enemies[1].state else {
            panic!("mob should still be staggered: {:?}", w.enemies[1].state)
        };
        assert!(!matches!(w.enemies[0].state, EState::Stagger { .. }));
    }

    #[test]
    fn turning_has_a_cooldown_plants_the_player_and_is_off_while_locked() {
        let t = Tuning::default();
        let mut w = world_with_boss();
        let walk = Actions {
            move_dir: vec2(0.0, 1.0),
            ..Default::default()
        };
        w.step(
            &Actions {
                turn_right: true,
                ..walk
            },
            &t,
        );
        assert!(matches!(w.player.state, PState::Turn { .. }));
        w.step(&walk, &t);
        assert_eq!(w.player.vel, Vec3::ZERO, "planted while twisting");
        let ev = w.step(&turn(true), &t);
        assert!(
            !ev.iter().any(|e| matches!(e, Event::Turn { .. })),
            "cooldown"
        );
        // 대기가 끝나도 락온 중엔 안 된다.
        w.player.turn_cd = 0.0;
        w.player.state = PState::Idle;
        w.hitstop = 0.0;
        w.step(
            &Actions {
                lock: true,
                ..Default::default()
            },
            &t,
        );
        assert!(w.lock.is_some());
        let ev = w.step(&turn(true), &t);
        assert!(!ev.iter().any(|e| matches!(e, Event::Turn { .. })));
        assert_eq!(w.turn_ready(&t), 0.0);
    }

    /// 기본값(0029)은 밀림이 짧아 거의 부딪히지 않는다 — 랩에서 밀림·벽 피해를 키웠을 때의 규칙을 본다.
    #[test]
    fn a_spilled_enemy_that_hits_a_pillar_is_slammed_once() {
        let mut t = Tuning::default();
        for (p, v) in [
            (P::SpillPerM, 0.35),
            (P::SpillMax, 3.0),
            (P::SlamDamage, 1.0),
        ] {
            let i = crate::tuning::SPECS.iter().position(|s| s.p == p).unwrap();
            t.set(i, v);
        }
        let mut w = World::new("test", 30.0);
        w.view_yaw = Some(0.0); // 화면 위 = -z, ↻이 멈추면 쏟아지는 쪽 = +x(화면 오른쪽, + 바깥)
        w.spawn_mob(vec3(0.0, 0.0, -7.0));
        w.obstacles.push(Aabb {
            min: vec3(1.0, 0.0, -9.0),
            max: vec3(3.0, 3.0, -6.0),
        });
        idle(&mut w);
        let mut slams = 0;
        let mut ev = turn_until_spill(&mut w, true, &t);
        for _ in 0..60 {
            slams += ev
                .iter()
                .filter(|e| matches!(e, Event::Slam { .. }))
                .count();
            ev = w.step(&Actions::default(), &t);
        }
        assert_eq!(slams, 1);
        assert_eq!(
            w.enemies[0].hp,
            w.enemies[0].max_hp - t[P::SlamDamage] as i32
        );
    }

    /// 0028: 어느 쪽으로 걷다 누르든 쥔 스틱 그대로 새 정면(돌기 전 캐릭터의 오른쪽·왼쪽)으로 직진한다 — 몸과 화면이 같이 90° 도니까.
    /// (0023은 위로 걸을 때만 그랬다: 오른쪽으로 걷다 오른쪽 버튼이면 화면이 180° 돌고 쥔 스틱이 왔던 길로 되돌아갔다.)
    #[test]
    fn after_a_turn_the_held_stick_walks_straight_to_the_new_front() {
        let t = Tuning::default();
        let yaw0 = 35.0;
        for stick in [
            vec2(0.0, 1.0),
            vec2(1.0, 0.0),
            vec2(0.0, -1.0),
            vec2(-1.0, 0.0),
        ] {
            for right in [true, false] {
                let mut w = World::new("test", 30.0);
                w.view_yaw = Some(yaw0);
                let go = Actions {
                    move_dir: stick,
                    ..Default::default()
                };
                for _ in 0..30 {
                    w.step(&go, &t);
                }
                let f = w.player.facing_dir();
                let want = if right {
                    f.cross(Vec3::Y)
                } else {
                    -f.cross(Vec3::Y)
                };
                let ev = w.step(
                    &Actions {
                        turn_right: right,
                        turn_left: !right,
                        ..go
                    },
                    &t,
                );
                let q = if right { -1 } else { 1 };
                assert!(ev.contains(&Event::Turn { quarters: q }), "{ev:?}");
                w.view_yaw = Some(yaw0 + 90.0 * q as f32); // 앱이 카메라에서 넘기는 새 방위(입력 기준은 즉시)
                let plant = (t[P::TurnPlant] / TICK_DT).ceil() as usize;
                for _ in 0..plant + 12 {
                    w.step(&go, &t);
                }
                let v = w.player.vel.normalize();
                assert!(
                    v.dot(want) > 0.98 && w.player.facing_dir().dot(want) > 0.98,
                    "stick {stick} right {right}: walks {v:?}, wants {want:?}"
                );
            }
        }
    }

    /// 강공격 시험용: 누름 + 쥐기를 `ticks`틱, 그 뒤 놓고 `after`틱. 지나간 상태·사건을 모은다.
    fn hold_attack(
        w: &mut World,
        t: &Tuning,
        ticks: usize,
        after: usize,
    ) -> (Vec<PState>, Vec<Event>) {
        let (mut st, mut ev) = (Vec::new(), Vec::new());
        for i in 0..ticks + after {
            let held = i < ticks;
            ev.extend(w.step(
                &Actions {
                    attack: i == 0,
                    attack_held: held,
                    ..Default::default()
                },
                t,
            ));
            st.push(w.player.state);
        }
        (st, ev)
    }

    /// 0030: 톡 누름(사람 0.08~0.15초)은 평타 하나 — 모으기로 들어가지 않는다(평타는 누르는 순간, KB melee 노트).
    #[test]
    fn a_tap_is_a_light_attack_and_never_charges() {
        let t = Tuning::default();
        for ticks in 1..=9 {
            let mut w = World::new("test", 20.0);
            let (st, _) = hold_attack(&mut w, &t, ticks, 60);
            assert!(
                matches!(st[0], PState::Attack { .. }),
                "pressed {ticks}: {:?}",
                st[0]
            );
            assert!(
                !st.iter()
                    .any(|s| matches!(s, PState::Charging { .. } | PState::Heavy { .. })),
                "held {ticks} ticks"
            );
        }
    }

    /// 0030: 쥐고 있으면 평타의 활성이 끝난 뒤 모으기, 떼면 늘 강타 — 다 모았으면 `full`, 다 모이는 틱에 `Charged` 한 번.
    #[test]
    fn holding_charges_and_letting_go_always_swings_a_heavy() {
        let t = Tuning::default();
        let into = ((t[P::AttackStartup] + t[P::AttackActive]).max(t[P::HeavyHoldStart]) / TICK_DT)
            .ceil() as usize
            + 1;
        let full = (t[P::HeavyCharge] / TICK_DT).ceil() as usize;
        for (extra, want_full) in [(3, false), (full + 5, true)] {
            let mut w = World::new("test", 20.0);
            let (st, ev) = hold_attack(&mut w, &t, into + extra, 3);
            assert!(
                st.iter().any(|s| matches!(s, PState::Charging { .. })),
                "charged"
            );
            let heavy = st.iter().find_map(|s| match s {
                PState::Heavy { full, .. } => Some(*full),
                _ => None,
            });
            assert_eq!(heavy, Some(want_full), "extra {extra}");
            let charged = ev.iter().filter(|e| matches!(e, Event::Charged)).count();
            assert_eq!(charged, want_full as usize);
        }
    }

    fn press(t: &Tuning, w: &mut World, a: Actions, ticks: usize) -> Vec<Event> {
        let mut ev = w.step(&a, t);
        for _ in 1..ticks {
            ev.extend(w.step(&Actions::default(), t));
        }
        ev
    }

    /// 0048: RB 마법 — 한 칸을 쓰고 탄이 날아가 앞의 적을 친다(자동 조준).
    #[test]
    fn a_cast_spends_a_charge_and_the_bolt_hits() {
        let t = Tuning::default();
        let mut w = World::new("test", 30.0);
        w.spawn_mob(vec3(1.0, 0.0, -6.0));
        w.enemies[0].hp = 9;
        idle(&mut w);
        let ev = press(
            &t,
            &mut w,
            Actions {
                cast: true,
                ..Default::default()
            },
            90,
        );
        assert!(ev.iter().any(|e| matches!(e, Event::Cast { .. })));
        assert_eq!(w.player.mana, CAST_CHARGES - 1.0);
        let magic = match w.player.weapon.def().rb {
            weapons::Rb::Magic(b) => b.damage,
            weapons::Rb::Shot { bolt, .. } => bolt.damage,
        };
        assert_eq!(w.enemies[0].hp, 9 - magic);
        assert!(w.bolts.is_empty());
        assert!(matches!(w.player.state, PState::Idle));
    }

    /// 0058: 커서를 비춘 땅 위의 점은 오른쪽 스틱처럼 겨눈다 — 가까운 적(자동 조준)이 아니라 커서 쪽 적을 맞힌다.
    /// 락온 중이면 커서는 쓰지 않는다(대상을 본다 — 스틱과 같다).
    #[test]
    fn the_cursor_point_aims_like_the_right_stick_and_lock_on_ignores_it() {
        let t = Tuning::default();
        let cast_at = |lock: bool| {
            let mut w = World::new("test", 30.0);
            w.spawn_mob(vec3(0.0, 0.0, -4.0)); // 가까운 앞쪽(자동 조준·락온이 고를 적)
            w.spawn_mob(vec3(7.0, 0.0, 0.0)); // 커서 쪽
            for e in &mut w.enemies {
                e.hp = 9;
            }
            idle(&mut w);
            if lock {
                press(
                    &t,
                    &mut w,
                    Actions {
                        lock: true,
                        ..Default::default()
                    },
                    2,
                );
                assert!(w.lock.is_some());
            }
            press(
                &t,
                &mut w,
                Actions {
                    cast: true,
                    aim_at: Some(vec2(7.0, 0.0)),
                    ..Default::default()
                },
                90,
            );
            (w.enemies[0].hp, w.enemies[1].hp)
        };
        let (near, cursor) = cast_at(false);
        assert_eq!(near, 9, "the near enemy is left alone");
        assert!(cursor < 9, "the bolt went the cursor's way");
        let (near, cursor) = cast_at(true);
        assert!(
            near < 9 && cursor == 9,
            "locked: the target, not the cursor"
        );
    }

    /// 0058: 커서는 자리까지 준다 — 수류탄 톡은 커서 자리에(2m..먼 거리로 자른다).
    #[test]
    fn a_tossed_grenade_lands_at_the_cursor() {
        let t = Tuning::default();
        let g = weapons::Id::Crossbow.def().grenade.unwrap();
        let mut w = World::new("test", 40.0);
        w.set_weapon(weapons::Id::Crossbow);
        w.view_yaw = Some(0.0);
        let p0 = w.player.pos;
        let want = vec2(p0.x + 4.0, p0.z + 3.0);
        w.step(
            &Actions {
                attack: true,
                aim_at: Some(want),
                ..Default::default()
            },
            &t,
        );
        let to = match w.player.state {
            PState::Throw { to, .. } => to,
            ref s => panic!("{s:?}"),
        };
        assert!(vec2(to.x, to.z).distance(want) < 0.05, "{to:?}");
        assert!(want.distance(vec2(p0.x, p0.z)) < g.far);
    }

    /// 0048: 칸이 모자라면 쏘지 않고(몸짓 없음) 칸이 깜빡인다. 근접으로 맞히면 `cast.per hit`씩 찬다.
    #[test]
    fn melee_hits_refill_the_magic_and_an_empty_cast_is_denied() {
        let t = Tuning::default();
        let mut w = World::new("test", 30.0);
        w.player.mana = 0.5;
        let ev = press(
            &t,
            &mut w,
            Actions {
                cast: true,
                ..Default::default()
            },
            2,
        );
        assert!(ev.iter().any(|e| matches!(e, Event::CastDenied)));
        assert!(matches!(w.player.state, PState::Idle));
        assert!(w.player.cast_denied > 0.0);
        w.spawn_mob(vec3(0.0, 0.0, -1.2));
        w.enemies[0].hp = 50;
        w.enemies[0].max_hp = 50;
        idle(&mut w);
        press(
            &t,
            &mut w,
            Actions {
                attack: true,
                ..Default::default()
            },
            40,
        );
        assert_eq!(w.enemies[0].hp, 49);
        assert!((w.player.mana - (0.5 + t[P::CastPerHit])).abs() < 1e-4);
    }

    /// 0048: 탄은 기둥에 막힌다.
    #[test]
    fn a_bolt_stops_at_a_pillar() {
        let t = Tuning::default();
        let mut w = World::new("test", 30.0);
        w.obstacles.push(Aabb {
            min: vec3(-1.0, 0.0, -3.5),
            max: vec3(1.0, 3.0, -2.5),
        });
        w.spawn_mob(vec3(0.0, 0.0, -6.0));
        w.enemies[0].hp = 9;
        idle(&mut w);
        press(
            &t,
            &mut w,
            Actions {
                cast: true,
                ..Default::default()
            },
            90,
        );
        assert_eq!(w.enemies[0].hp, 9);
        assert!(w.bolts.is_empty());
    }

    /// 0048: 점프 버튼이 상호작용도 한다 — 스테이지가 "여기서 쉴 수 있다"고 적은 자리면 쉬기(뛰지 않음), 아니면 점프.
    #[test]
    fn the_jump_button_interacts_where_the_stage_allows() {
        let t = Tuning::default();
        let jump = Actions {
            jump: true,
            ..Default::default()
        };
        let mut w = World::new("test", 30.0);
        w.interact_here = true;
        w.step(&jump, &t);
        assert!(w.interact);
        assert!(!matches!(w.player.state, PState::Jump { .. }));
        let mut w = World::new("test", 30.0);
        w.step(&jump, &t);
        assert!(!w.interact);
        assert!(matches!(w.player.state, PState::Jump { .. }));
    }

    fn mob_with(w: &mut World, at: Vec3, hp: i32) -> usize {
        w.spawn_mob(at);
        let i = w.enemies.len() - 1;
        w.enemies[i].hp = hp;
        w.enemies[i].max_hp = hp;
        i
    }

    /// 0051: 연타하면 검은 1 → 2 → 3타로 이어지고, 마지막 타 뒤에는 처음으로 돌아간다.
    #[test]
    fn mashing_the_sword_runs_its_three_hit_combo() {
        let t = Tuning::default();
        let mut w = World::new("test", 30.0);
        mob_with(&mut w, vec3(0.0, 0.0, -1.3), 99);
        idle(&mut w);
        let mut seen = Vec::new();
        for i in 0..240 {
            let a = Actions {
                attack: i % 6 == 0,
                ..Default::default()
            };
            w.step(&a, &t);
            idle(&mut w);
            w.player.stamina = 100.0;
            if let PState::Attack { hit, .. } = w.player.state {
                if seen.last() != Some(&hit) {
                    seen.push(hit);
                }
            }
        }
        assert!(seen.windows(3).any(|x| x == [0, 1, 2]), "{seen:?}");
        assert!(seen.windows(2).any(|x| x == [2, 0]), "wraps: {seen:?}");
    }

    /// 0051: 망치 강타는 친 자리에 금을 내고, 늦게 여진이 둘레를 다시 친다.
    #[test]
    fn the_hammer_slam_leaves_an_aftershock() {
        let t = Tuning::default();
        let mut w = World::new("test", 30.0);
        w.set_weapon(weapons::Id::Hammer);
        let far = mob_with(&mut w, vec3(0.0, 0.0, 8.0), 50);
        idle(&mut w);
        w.step(
            &Actions {
                attack: true,
                attack_held: true,
                ..Default::default()
            },
            &t,
        );
        let hold = Actions {
            attack_held: true,
            ..Default::default()
        };
        for _ in 0..200 {
            w.player.stamina = 100.0;
            w.step(&hold, &t);
            if matches!(w.player.state, PState::Charging { t: e } if e > 1.0) {
                break;
            }
        }
        let mut ev = Vec::new();
        for _ in 0..30 {
            ev.extend(w.step(&Actions::default(), &t));
        }
        assert_eq!(w.cracks.len(), 1, "a crack under the slam");
        // 여진 자리로 적을 옮겨 두고 기다린다.
        w.enemies[far].pos = w.cracks[0].pos + vec3(0.8, 0.0, 0.0);
        idle(&mut w);
        for _ in 0..150 {
            ev.extend(w.step(&Actions::default(), &t));
            idle(&mut w);
        }
        assert!(ev.iter().any(|e| matches!(e, Event::Aftershock { .. })));
        assert!(w.enemies[far].hp < 50, "aftershock hits");
        assert!(w.cracks.is_empty());
    }

    /// 0051: 지팡이는 다 모인 뒤 짧은 창 안에 떼야 "딱 맞춤", 늦게 떼면 보통 강타.
    #[test]
    fn every_weapon_rewards_a_perfectly_timed_heavy_release() {
        // 0078(사장 2026-11-07): 딱 맞춰 떼기는 지팡이만이 아니라 강타가 있는 모든 무기의 규칙.
        let t = Tuning::default();
        for id in weapons::IDS.into_iter().filter(|i| i.def().heavy.is_some()) {
            let w0 = id.def();
            let full = t[P::HeavyCharge] * w0.heavy.unwrap().charge;
            for (extra, want) in [(0.05, true), (0.6, false)] {
                let mut w = World::new("test", 30.0);
                w.set_weapon(id);
                w.step(
                    &Actions {
                        attack: true,
                        attack_held: true,
                        ..Default::default()
                    },
                    &t,
                );
                let hold = Actions {
                    attack_held: true,
                    ..Default::default()
                };
                let mut charged_for = 0.0;
                for _ in 0..400 {
                    w.player.stamina = 100.0;
                    w.step(&hold, &t);
                    if let PState::Charging { t: e } = w.player.state {
                        charged_for = e;
                        if e >= full + extra {
                            break;
                        }
                    }
                }
                assert!(charged_for >= full);
                w.step(&Actions::default(), &t);
                let got = matches!(w.player.state, PState::Heavy { perfect: true, .. });
                assert_eq!(got, want, "{id:?}: released {extra}s after full");
            }
        }
    }

    /// 0051: 쇠뇌 — RB는 탄창에서 쏘고, 비면 장전 뒤 가득, 회피하면 한 발 장전. 칸(마법)은 쓰지 않는다.
    #[test]
    fn the_crossbow_shoots_from_a_magazine_and_reloads() {
        let t = Tuning::default();
        let mut w = World::new("test", 40.0);
        w.set_weapon(weapons::Id::Crossbow);
        let (mag, reload) = match weapons::Id::Crossbow.def().rb {
            Rb::Shot { mag, reload, .. } => (mag, reload),
            _ => unreachable!(),
        };
        assert_eq!(w.player.ammo, mag);
        let mana = w.player.mana;
        let cast = Actions {
            cast: true,
            ..Default::default()
        };
        for _ in 0..mag {
            press(&t, &mut w, cast, 40);
        }
        assert_eq!(w.player.ammo, 0);
        assert_eq!(w.player.mana, mana, "no magic spent");
        let ev = press(&t, &mut w, cast, 2);
        assert!(ev.iter().any(|e| matches!(e, Event::CastDenied)), "empty");
        // 회피 한 번 = 한 발.
        press(
            &t,
            &mut w,
            Actions {
                dodge: true,
                ..Default::default()
            },
            2,
        );
        assert_eq!(w.player.ammo, 1);
        // 장전이 끝나면 가득.
        press(
            &t,
            &mut w,
            Actions::default(),
            (reload / TICK_DT) as usize + 10,
        );
        assert_eq!(w.player.ammo, mag);
    }

    /// 0051: 쇠뇌의 X = 수류탄 — 날아가 도화선이 다하면 둘레를 친다(셀 수 있고 다시 찬다).
    #[test]
    fn a_grenade_flies_bounces_and_blows_up() {
        let t = Tuning::default();
        let mut w = World::new("test", 40.0);
        w.set_weapon(weapons::Id::Crossbow);
        let g = weapons::Id::Crossbow.def().grenade.unwrap();
        let m = mob_with(&mut w, vec3(0.0, 0.0, -5.0), 20);
        idle(&mut w);
        let mut ev = w.step(
            &Actions {
                attack: true,
                ..Default::default()
            },
            &t,
        );
        assert!((w.player.grenades - (g.count as f32 - 1.0)).abs() < 0.05);
        for _ in 0..((g.fuse + 0.6) / TICK_DT) as usize {
            ev.extend(w.step(&Actions::default(), &t));
            idle(&mut w);
            w.enemies[m].pos = vec3(0.0, 0.0, -5.0);
        }
        assert!(ev.iter().any(|e| matches!(e, Event::Throw)));
        assert!(ev.iter().any(|e| matches!(e, Event::Boom { .. })));
        assert!(
            w.enemies[m].hp <= 20 - g.damage,
            "blast hits: {}",
            w.enemies[m].hp
        );
        assert!(w.bombs.is_empty());
    }

    /// 0055: 쇠뇌 X를 꾹 쥐면 손을 든 채 겨누다가 떼면 가장 먼 자리로 던진다(톡은 가까운 자동 조준 자리).
    #[test]
    fn holding_the_grenade_aims_far() {
        let t = Tuning::default();
        let g = weapons::Id::Crossbow.def().grenade.unwrap();
        let mut w = World::new("test", 40.0);
        w.set_weapon(weapons::Id::Crossbow);
        let hold = Actions {
            attack_held: true,
            move_dir: vec2(1.0, 0.0),
            ..Default::default()
        };
        w.view_yaw = Some(0.0);
        w.step(
            &Actions {
                attack: true,
                ..hold
            },
            &t,
        );
        for _ in 0..40 {
            w.step(&hold, &t);
        }
        let PState::Throw {
            to, thrown: false, ..
        } = w.player.state
        else {
            panic!("aiming: {:?}", w.player.state)
        };
        let d = vec3(to.x - w.player.pos.x, 0.0, to.z - w.player.pos.z).length();
        assert!((d - g.far).abs() < 0.2, "aims at the far ring: {d}");
        let mut ev = Vec::new();
        for _ in 0..3 {
            ev.extend(w.step(&Actions::default(), &t));
        }
        assert!(ev.iter().any(|e| matches!(e, Event::Throw)));
        assert_eq!(w.bombs.len(), 1);
    }

    /// RB를 누른 채 쥔다: 한 발 쏘고 모으기로 들어가 `ticks`틱 쥔 뒤 뗀다 — 뗀 직후의 탄들.
    fn charge_cast(w: &mut World, t: &Tuning, ticks: usize) -> Vec<Bolt> {
        let held = Actions {
            cast_held: true,
            ..Default::default()
        };
        w.step(&Actions { cast: true, ..held }, t);
        for _ in 0..200 {
            if matches!(w.player.state, PState::CastCharge { .. }) {
                break;
            }
            w.step(&held, t);
        }
        assert!(
            matches!(w.player.state, PState::CastCharge { .. }),
            "{:?}",
            w.player.state
        );
        for _ in 0..ticks {
            w.step(&held, t);
        }
        w.bolts.clear();
        w.step(&Actions::default(), t);
        w.bolts.clone()
    }

    /// 0056: RB를 쥐면 한 발 쏜 뒤 모으기 — 다 모아 떼면 무기 표의 모은 탄(크고 세다), 칸을 하나 더 쓴다.
    #[test]
    fn holding_rb_charges_a_stronger_magic_bolt() {
        let t = Tuning::default();
        let full = (t[P::CastCharge] / TICK_DT).ceil() as usize + 2;
        let mut w = World::new("test", 40.0);
        let got = charge_cast(&mut w, &t, full);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].def, weapons::Id::Sword.def().charged);
        assert_eq!(
            w.player.mana,
            CAST_CHARGES - 2.0,
            "one for the tap, one for the charge"
        );
        // 덜 모으고 떼면 약하게(반 피해, 꿰뚫기 없음).
        let mut w = World::new("test", 40.0);
        let got = charge_cast(&mut w, &t, 3);
        let c = weapons::Id::Sword.def().charged;
        assert_eq!(got[0].def.damage, (c.damage + 1) / 2);
        assert!(!got[0].def.pierce);
    }

    /// 0056: 쇠뇌는 모은 사격 — 볼트 하나를 쓰고 꿰뚫는다. 볼트가 없으면 모으지 않는다.
    #[test]
    fn the_crossbow_charges_a_piercing_shot() {
        let t = Tuning::default();
        let full = (t[P::CastCharge] / TICK_DT).ceil() as usize;
        let past_perfect = (PERFECT_RELEASE / TICK_DT).ceil() as usize + 2;
        let charged = weapons::Id::Crossbow.def().charged;
        let mut w = World::new("test", 40.0);
        w.set_weapon(weapons::Id::Crossbow);
        let mag = w.player.ammo;
        let got = charge_cast(&mut w, &t, full + past_perfect);
        assert!(got[0].def.pierce && got[0].def.damage == charged.damage);
        assert_eq!(w.player.ammo, mag - 2);
        // 딱 맞춰 떼기(0078): 다 모인 바로 뒤에 떼면 피해 +1 — 강타와 같은 규칙.
        let mut w = World::new("test", 40.0);
        w.set_weapon(weapons::Id::Crossbow);
        let got = charge_cast(&mut w, &t, full + 2);
        assert!(got[0].def.pierce && got[0].def.damage == charged.damage + 1);
        // 근접 무기의 RB 마법은 무기의 모으기 공격이 아니다 — 딱 맞춤 없음.
        let mut w = World::new("test", 40.0);
        let got = charge_cast(&mut w, &t, full + 2);
        assert_eq!(got[0].def.damage, weapons::Id::Sword.def().charged.damage);
        // 볼트가 다 떨어지면 쥐어도 모으기로 가지 않는다.
        w.player.ammo = 1;
        let held = Actions {
            cast_held: true,
            ..Default::default()
        };
        w.step(&Actions { cast: true, ..held }, &t);
        for _ in 0..80 {
            w.step(&held, &t);
            assert!(!matches!(w.player.state, PState::CastCharge { .. }));
        }
    }

    /// 0057 시험: (누름, 바라는 상태).
    type Check = (Actions, fn(&PState) -> bool);

    /// 0057: 누른 틱부터 몇 틱 뒤에 상태가 바뀌었나(못 바뀌면 None).
    fn ticks_until(
        w: &mut World,
        t: &Tuning,
        a: Actions,
        want: fn(&PState) -> bool,
        max: usize,
    ) -> Option<usize> {
        w.step(&a, t);
        if want(&w.player.state) {
            return Some(0);
        }
        for i in 1..max {
            w.step(&Actions::default(), t);
            if want(&w.player.state) {
                return Some(i);
            }
        }
        None
    }

    /// 0057: 평타의 후딜레이는 다음 행동(회피·패링·점프·마법)으로 끊긴다 — 활성 구간이 끝나면 바로, 그 전에 누르면 받아 뒀다가 그때.
    #[test]
    fn attack_recovery_cancels_into_any_next_action() {
        let t = Tuning::default();
        let w0 = weapons::Id::Sword.def();
        let (s0, a0, r0) = hit_times(&t, w0, 0);
        let active_end = ((s0 + a0) / TICK_DT).ceil() as usize;
        let checks: [Check; 4] = [
            (
                Actions {
                    dodge: true,
                    ..Default::default()
                },
                |s| matches!(s, PState::Dodge { .. }),
            ),
            (
                Actions {
                    parry: true,
                    ..Default::default()
                },
                |s| matches!(s, PState::Parry { .. }),
            ),
            (
                Actions {
                    jump: true,
                    ..Default::default()
                },
                |s| matches!(s, PState::Jump { .. }),
            ),
            (
                Actions {
                    cast: true,
                    ..Default::default()
                },
                |s| matches!(s, PState::Cast { .. }),
            ),
        ];
        for (press, want) in checks {
            let mut w = World::new("test", 30.0);
            w.step(
                &Actions {
                    attack: true,
                    ..Default::default()
                },
                &t,
            );
            // 활성 중에 누른다 → 활성이 끝나는 틱 즈음에 나간다(후딜레이 전부를 기다리지 않는다).
            let got = ticks_until(&mut w, &t, press, want, 60).expect("cancelled");
            assert!(got <= active_end + 1, "{press:?}: {got} ticks");
            assert!(
                ((got as f32) * TICK_DT) < s0 + a0 + r0 - TICK_DT,
                "before the recovery ends"
            );
        }
    }

    /// 0057: 회피의 뒷부분(`dodge.cancel x` 뒤)은 공격·점프로 끊긴다 — 회피 초반에 누른 것은 받아 뒀다가 그때.
    #[test]
    fn a_dodge_flows_into_an_attack_or_a_jump() {
        let t = Tuning::default();
        let cancel_at = (t[P::DodgeTime] * t[P::DodgeCancel] / TICK_DT).ceil() as usize;
        let checks: [Check; 2] = [
            (
                Actions {
                    attack: true,
                    ..Default::default()
                },
                |s| matches!(s, PState::Attack { .. }),
            ),
            (
                Actions {
                    jump: true,
                    ..Default::default()
                },
                |s| matches!(s, PState::Jump { .. }),
            ),
        ];
        for (press, want) in checks {
            let mut w = World::new("test", 30.0);
            w.step(
                &Actions {
                    dodge: true,
                    ..Default::default()
                },
                &t,
            );
            assert!(matches!(w.player.state, PState::Dodge { .. }));
            let got = ticks_until(&mut w, &t, press, want, 60).expect("flows");
            assert!(
                got + 1 >= cancel_at.saturating_sub(1) && got <= cancel_at + 1,
                "{press:?}: {got} vs {cancel_at}"
            );
        }
    }

    /// 0057: 캔슬 시점은 무기 성격 — 망치는 후딜레이의 앞부분이 지나야 끊긴다(검보다 늦게).
    #[test]
    fn a_hammer_commits_longer_than_a_sword() {
        let t = Tuning::default();
        let when = |id: weapons::Id| {
            let mut w = World::new("test", 30.0);
            w.set_weapon(id);
            w.step(
                &Actions {
                    attack: true,
                    ..Default::default()
                },
                &t,
            );
            // 회피를 연타한다(너무 일찍 한 번 누른 것은 `attack.buffer s`가 지나면 사라진다 — 무거운 무기의 대가).
            (1..120)
                .find(|_| {
                    w.step(
                        &Actions {
                            dodge: true,
                            ..Default::default()
                        },
                        &t,
                    );
                    matches!(w.player.state, PState::Dodge { .. })
                })
                .expect("cancels at some point")
        };
        let (sword, hammer) = (when(weapons::Id::Sword), when(weapons::Id::Hammer));
        let h = weapons::Id::Hammer.def();
        let (s0, a0, r0) = hit_times(&t, h, 0);
        let want = ((s0 + a0 + r0 * h.combo[0].cancel) / TICK_DT).ceil() as usize;
        assert!(hammer > sword, "hammer {hammer} vs sword {sword}");
        assert!(hammer + 2 >= want, "{hammer} vs {want}");
    }

    /// 0057: 한 번 누른 회피는 한 번만 — 받아 둔 누름이 쓰이면 지운다.
    #[test]
    fn one_press_is_one_action() {
        let t = Tuning::default();
        let mut w = World::new("test", 30.0);
        let mut dodges = 0;
        let mut was = false;
        for i in 0..90 {
            w.step(
                &Actions {
                    dodge: i == 0,
                    ..Default::default()
                },
                &t,
            );
            let now = matches!(w.player.state, PState::Dodge { .. });
            if now && !was {
                dodges += 1;
            }
            was = now;
        }
        assert_eq!(dodges, 1);
    }

    /// 0051: 자세 막대가 다 차면 깨진다 — 휘청 + 드러남(다음 한 대 치명타), 깬 뒤 잠깐은 다시 안 찬다.
    #[test]
    fn filling_the_poise_bar_breaks_the_enemy() {
        let t = Tuning::default();
        let mut w = World::new("test", 30.0);
        let i = mob_with(&mut w, vec3(0.0, 0.0, -3.0), 99);
        let max = poise_max(&w.enemies[i], &t);
        let mut ev = Vec::new();
        add_poise(&mut w.enemies[i], max * 0.5, None, &t, &mut ev);
        assert!(ev.is_empty());
        add_poise(&mut w.enemies[i], max * 0.6, None, &t, &mut ev);
        assert!(ev.iter().any(|e| matches!(e, Event::Break { .. })));
        assert!(matches!(w.enemies[i].state, EState::Stagger { .. }));
        assert!(w.enemies[i].exposed > 0.0);
        // 0055: 깨진 동안은 모든 타격이 × `poise.break damage x`(폭딜 창) — 첫 대는 치명타 덤까지.
        assert!(w.enemies[i].broken > 0.0);
        let base = 2;
        let want = (base as f32 * t[P::PoiseBreakDamage]).ceil() as i32;
        assert_eq!(hurt_by(&w.enemies[i], base, false, &t), want);
        assert!(hurt_by(&w.enemies[i], base, true, &t) > want);
        w.enemies[i].broken = 0.0;
        assert_eq!(hurt_by(&w.enemies[i], base, false, &t), base);
        let before = w.enemies[i].poise;
        add_poise(&mut w.enemies[i], max, None, &t, &mut ev);
        assert_eq!(w.enemies[i].poise, before, "locked after a break");
        // 망치처럼 "깨지기 직전"을 깨는 타는 반만 차 있어도 깬다.
        let j = mob_with(&mut w, vec3(2.0, 0.0, -3.0), 99);
        let mut ev = Vec::new();
        add_poise(&mut w.enemies[j], max * 0.7, None, &t, &mut ev);
        add_poise(&mut w.enemies[j], 0.1, Some(0.6), &t, &mut ev);
        assert!(ev.iter().any(|e| matches!(e, Event::Break { .. })));
    }

    /// 0047: 모으는 동안 느리게 걷는다(제자리가 아니다) — 걷기의 `heavy.charge walk x`배쯤.
    #[test]
    fn charging_walks_slowly() {
        let t = Tuning::default();
        let mut w = World::new("test", 30.0);
        w.view_yaw = Some(0.0);
        let walk = Actions {
            move_dir: vec2(0.0, 1.0),
            ..Default::default()
        };
        w.step(
            &Actions {
                attack: true,
                attack_held: true,
                ..walk
            },
            &t,
        );
        let hold = Actions {
            attack_held: true,
            ..walk
        };
        for _ in 0..120 {
            if matches!(w.player.state, PState::Charging { .. }) {
                break;
            }
            w.step(&hold, &t);
        }
        assert!(matches!(w.player.state, PState::Charging { .. }));
        for _ in 0..30 {
            w.step(&hold, &t);
        }
        assert!(matches!(w.player.state, PState::Charging { .. }));
        let v = w.player.vel.length();
        let want = t[P::MoveSpeed] * t[P::HeavyChargeWalk];
        assert!((v - want).abs() < want * 0.1, "{v} m/s, want {want}");
    }

    /// 0047: 뛰어 있는 동안에도 스틱 쪽으로 몸을 돌린다(공중에서 내려찍기를 겨눈다). 내려찍기가 시작되면 고정.
    #[test]
    fn a_jump_turns_toward_the_stick() {
        let t = Tuning::default();
        let mut w = World::new("test", 30.0);
        w.view_yaw = Some(0.0);
        let f0 = w.player.facing;
        w.step(
            &Actions {
                jump: true,
                ..Default::default()
            },
            &t,
        );
        let side = Actions {
            move_dir: vec2(1.0, 0.0),
            ..Default::default()
        };
        for _ in 0..20 {
            w.step(&side, &t);
        }
        assert!(w.player.airborne());
        let turned = angle_diff(w.player.facing, f0).abs();
        assert!(turned > 1.0, "turned {turned} rad in the air");
    }

    /// 0030: 다 모은 강타 = `heavy.damage` + 덤, 잡몹은 길게 휘청(예고를 끊는다), 보스는 다 모았을 때만 짧게.
    #[test]
    fn a_full_heavy_hits_hard_and_staggers() {
        let t = Tuning::default();
        let full = (t[P::HeavyCharge] / TICK_DT).ceil() as usize;
        for boss in [false, true] {
            for charged in [false, true] {
                let mut w = World::new("test", 20.0);
                if boss {
                    w.spawn_boss(vec3(0.0, 0.0, -1.5));
                } else {
                    w.spawn_mob(vec3(0.0, 0.0, -1.2));
                    w.enemies[0].hp = 50;
                    w.enemies[0].max_hp = 50;
                }
                idle(&mut w);
                let hp0 = w.enemies[0].hp;
                // 모으기에 들어설 때까지 쥔다(평타가 맞으면 히트스톱으로 틱이 멈추니 틱 수로 세지 않는다).
                let mut ev = w.step(
                    &Actions {
                        attack: true,
                        attack_held: true,
                        ..Default::default()
                    },
                    &t,
                );
                let hold = Actions {
                    attack_held: true,
                    ..Default::default()
                };
                for _ in 0..120 {
                    if matches!(w.player.state, PState::Charging { .. }) {
                        break;
                    }
                    ev.extend(w.step(&hold, &t));
                }
                // 다 모은 뒤 딱 맞춤 창(0078)을 지나서 뗀다 — 이 시험은 "다 모음"만 본다(딱 맞춤은 따로).
                let past_perfect = (PERFECT_RELEASE / TICK_DT).ceil() as usize + 2;
                for _ in 0..if charged { full + past_perfect } else { 3 } {
                    ev.extend(w.step(&hold, &t));
                }
                assert!(matches!(w.player.state, PState::Charging { .. }));
                // 평타 한 대는 이미 맞았다. 이제 놓는다.
                let hp1 = w.enemies[0].hp;
                assert_eq!(hp0 - hp1, 1, "the light hit first");
                assert!(!ev.iter().any(|e| matches!(e, Event::Smash { .. })));
                let mut ev = Vec::new();
                let mut staggered = false;
                for _ in 0..30 {
                    ev.extend(w.step(&Actions::default(), &t));
                    staggered |= matches!(w.enemies[0].state, EState::Stagger { .. });
                    idle_unless_staggered(&mut w);
                }
                assert!(
                    ev.iter().any(|e| matches!(e, Event::Smash { .. })),
                    "boss {boss} charged {charged}"
                );
                let want = w.player.weapon.def().heavy.unwrap().damage
                    + if charged {
                        t[P::HeavyFullBonus] as i32
                    } else {
                        0
                    };
                assert_eq!(hp1 - w.enemies[0].hp, want, "boss {boss} charged {charged}");
                assert_eq!(staggered, !boss || charged, "boss {boss} charged {charged}");
            }
        }
    }

    /// `idle`과 같지만 휘청은 그대로 둔다(강타의 휘청을 보려고).
    fn idle_unless_staggered(w: &mut World) {
        for e in &mut w.enemies {
            if !matches!(e.state, EState::Stagger { .. }) {
                e.state = EState::Recover { t: -1000.0 };
            }
        }
    }

    /// 등잔 시험용: 한 번 누른 뒤 `ticks`틱 그대로(누름 없음). 사건을 모은다.
    fn drink(w: &mut World, t: &Tuning, ticks: usize) -> Vec<Event> {
        let mut ev = w.step(
            &Actions {
                flask: true,
                ..Default::default()
            },
            t,
        );
        for _ in 0..ticks {
            ev.extend(w.step(&Actions::default(), t));
        }
        ev
    }

    /// 0031: 한 모금 = 체력 + `flask.heal`(가득을 넘지 않는다), 등잔 −1, 삼키는 순간 한 번.
    #[test]
    fn a_sip_heals_and_spends_one_charge() {
        let t = Tuning::default();
        let mut w = World::new("test", 20.0);
        w.player.hp = 1;
        let ev = drink(&mut w, &t, 80); // 마시기 0.8초 + 누름 버리기 0.3초를 넘게
        assert_eq!(w.player.hp, 1 + t[P::FlaskHeal] as i32);
        assert_eq!(w.player.flask, FLASK_CHARGES - 1);
        assert_eq!(
            ev.iter()
                .filter(|e| matches!(e, Event::Gulp { .. }))
                .count(),
            1
        );
        w.player.hp = PLAYER_HP - 1;
        drink(&mut w, &t, 80);
        assert_eq!(w.player.hp, PLAYER_HP, "never above full");
    }

    /// 0031: 가득이거나 등잔이 0이면 몸짓 없이 깜빡만 — 등잔·자세 그대로.
    #[test]
    fn a_full_or_empty_flask_press_does_nothing_but_flash() {
        let t = Tuning::default();
        for (hp, charges) in [(PLAYER_HP, FLASK_CHARGES), (1, 0)] {
            let mut w = World::new("test", 20.0);
            w.player.hp = hp;
            w.player.flask = charges;
            let ev = drink(&mut w, &t, 0);
            assert_eq!(w.player.state, PState::Idle, "hp {hp} charges {charges}");
            assert_eq!(w.player.flask, charges);
            assert!(ev.contains(&Event::FlaskDenied));
            assert!(w.player.flask_denied > 0.0);
        }
    }

    /// 0031(사장 "물약이 더 큰 문제 아냐?"): RB 연타 — 누름 1~9틱 × 사이 2~14틱 × 3초. 마시기는 끊기지 않고(들어간 만큼 삼킨다),
    /// 마신 수는 3초 ÷ (`flask.time` + `flask.lock`)을 넘지 않는다(누름이 쌓여 연달아 마시지 않는다). 가득이면 0번.
    #[test]
    fn mashing_the_flask_sips_one_at_a_time() {
        let t = Tuning::default();
        let ticks = 180;
        let most = (ticks as f32 * TICK_DT / (t[P::FlaskTime] + t[P::FlaskLock])).ceil() as usize;
        for press in 1..=9usize {
            for gap in 2..=14usize {
                for full in [false, true] {
                    let mut w = World::new("test", 20.0);
                    w.player.flask = 10;
                    let (mut starts, mut gulps, mut was) = (0, 0, false);
                    for i in 0..ticks {
                        if !full {
                            w.player.hp = 1; // 늘 다쳐 있다(급한 상황)
                        }
                        let k = i % (press + gap);
                        let ev = w.step(
                            &Actions {
                                flask: k == 0,
                                ..Default::default()
                            },
                            &t,
                        );
                        gulps += ev
                            .iter()
                            .filter(|e| matches!(e, Event::Gulp { .. }))
                            .count();
                        let now = matches!(w.player.state, PState::Drink { .. });
                        starts += (now && !was) as usize;
                        was = now;
                    }
                    let tag = format!("press {press} gap {gap} full {full}");
                    if full {
                        assert_eq!(starts, 0, "{tag}");
                    } else {
                        assert!(starts >= 1 && starts <= most, "{tag}: {starts} > {most}");
                        // 끝까지 간 마시기는 모두 삼켰다(다시 누르기로 끊기지 않는다) — 마지막 것은 아직 들고 있을 수 있다.
                        assert!(
                            gulps + 1 >= starts && gulps <= starts,
                            "{tag}: {starts} starts {gulps} gulps"
                        );
                    }
                }
            }
        }
    }

    /// 0031: 삼키기 전 회피는 끊기 — 등잔·체력 그대로(어깨 버튼 실수를 무른다). 삼킨 뒤 회피는 안 먹는다.
    #[test]
    fn a_dodge_before_the_gulp_takes_the_flask_back() {
        let t = Tuning::default();
        let mut w = World::new("test", 20.0);
        w.player.hp = 1;
        drink(&mut w, &t, 5);
        w.player.dodge_cd = 0.0;
        w.step(
            &Actions {
                dodge: true,
                ..Default::default()
            },
            &t,
        );
        assert!(matches!(w.player.state, PState::Dodge { .. }));
        assert_eq!((w.player.flask, w.player.hp), (FLASK_CHARGES, 1));
        // 0057: 삼킨 뒤에는 내리는 동작(후딜레이)을 회피로 끊는다 — 등잔·체력은 이미 바뀌었다(0031의 "삼킨 뒤 회피는 안 먹는다"를 바꿈).
        let mut w = World::new("test", 20.0);
        w.player.hp = 1;
        drink(&mut w, &t, (t[P::FlaskRaise] / TICK_DT) as usize + 3);
        w.player.dodge_cd = 0.0;
        w.step(
            &Actions {
                dodge: true,
                ..Default::default()
            },
            &t,
        );
        assert!(matches!(w.player.state, PState::Dodge { .. }));
        assert_eq!(
            (w.player.flask, w.player.hp),
            (FLASK_CHARGES - 1, 1 + t[P::FlaskHeal].round() as i32)
        );
    }

    /// 0031: 삼키기 전에 맞으면 끊기고 등잔 그대로(대가는 맞은 피해와 시간).
    #[test]
    fn a_hit_before_the_gulp_keeps_the_flask() {
        let t = Tuning::default();
        let mut w = mob_swinging_in_front(&t);
        w.player.hp = 3;
        let mut hurt = drink(&mut w, &t, 0).contains(&Event::PlayerHurt);
        for _ in 0..((t[P::FlaskRaise] / TICK_DT) as usize - 2) {
            if hurt {
                break;
            }
            hurt |= w.step(&Actions::default(), &t).contains(&Event::PlayerHurt);
        }
        assert!(hurt, "the blow landed before the gulp");
        assert!(!matches!(w.player.state, PState::Drink { .. }));
        assert_eq!((w.player.flask, w.player.hp), (FLASK_CHARGES, 2));
    }

    /// 0030(사장 "급하면 막 연타할텐데"): 연타는 모으기로 새지 않는다 — 누름 길이 1~9틱(≤0.15초) × 사이 2~14틱,
    /// 누르는 때가 휘두르기의 어느 자리에 걸리든. 예전(쥐고 있나만 봄)엔 휘두르기 도중의 짧은 누름이 활성 끝에 걸려 강타가 나갔다.
    #[test]
    fn mashing_never_charges_a_heavy() {
        let t = Tuning::default();
        for press in 1..=9usize {
            for gap in 2..=14usize {
                let mut w = World::new("test", 20.0);
                let period = press + gap;
                let mut swings = 0;
                for i in 0..240 {
                    let k = i % period;
                    w.player.stamina = 100.0; // 스태미나가 바닥나 멈추지 않게(연타만 본다)
                    w.step(
                        &Actions {
                            attack: k == 0,
                            attack_held: k < press,
                            ..Default::default()
                        },
                        &t,
                    );
                    assert!(
                        !matches!(
                            w.player.state,
                            PState::Charging { .. } | PState::Heavy { .. }
                        ),
                        "press {press} gap {gap} tick {i}"
                    );
                    if matches!(w.player.state, PState::Attack { t: e, .. } if e == 0.0) {
                        swings += 1;
                    }
                }
                assert!(swings >= 3, "press {press} gap {gap}: {swings} swings");
            }
        }
    }

    /// 0030: 모으는 중에도 회피로 끊는다(강타는 나가지 않는다).
    #[test]
    fn a_dodge_cancels_the_charge() {
        let t = Tuning::default();
        let into = ((t[P::AttackStartup] + t[P::AttackActive]).max(t[P::HeavyHoldStart]) / TICK_DT)
            .ceil() as usize
            + 1;
        let mut w = World::new("test", 20.0);
        hold_attack(&mut w, &t, into + 5, 0);
        assert!(matches!(w.player.state, PState::Charging { .. }));
        w.player.dodge_cd = 0.0;
        w.step(
            &Actions {
                dodge: true,
                attack_held: true,
                ..Default::default()
            },
            &t,
        );
        assert!(matches!(w.player.state, PState::Dodge { .. }));
    }

    /// 0030(KB melee 노트 "모으기는 시간을 내준 만큼"): 평타 + 다 모은 강타의 초당 피해가 같은 시간 평타만 친 것보다 확실히 크다.
    #[test]
    fn a_full_heavy_pays_back_the_time_it_takes() {
        let t = Tuning::default();
        let light = t[P::AttackStartup] + t[P::AttackActive] + t[P::AttackRecovery];
        let heavy_time = (t[P::AttackStartup] + t[P::AttackActive]).max(t[P::HeavyHoldStart])
            + t[P::HeavyCharge]
            + t[P::HeavyStartup]
            + t[P::HeavyActive]
            + t[P::HeavyRecovery];
        let heavy_dmg =
            1.0 + weapons::Id::Sword.def().heavy.unwrap().damage as f32 + t[P::HeavyFullBonus];
        let (h, l) = (heavy_dmg / heavy_time, 1.0 / light);
        assert!(h > l * 1.2, "heavy {h:.2}/s vs light {l:.2}/s");
    }

    #[test]
    fn a_press_during_hitstop_is_kept_for_when_the_world_moves_again() {
        let t = Tuning::default();
        let mut w = world_with_boss();
        w.hitstop = 0.1;
        w.step(
            &Actions {
                lock: true,
                ..Default::default()
            },
            &t,
        );
        assert!(w.lock.is_none(), "frozen");
        let mut ev = Vec::new();
        for _ in 0..10 {
            ev.extend(w.step(&Actions::default(), &t));
        }
        assert!(ev.contains(&Event::LockOn) && w.lock.is_some());
    }

    /// 2026-11-06(타격감, KB melee-light…): 평타가 맞으면 6프레임쯤 멈추고(0.06초 × `HIT_HITSTOP`), 맞은 쪽으로 불똥이 튀고
    /// 몸이 젖고, 공격자는 남은 내딛기를 멈춘다(빗나감과 맞힘이 손에서 다르게).
    #[test]
    fn a_landed_light_hit_stops_the_world_sprays_sparks_and_ends_the_lunge() {
        let t = Tuning::default();
        let mut w = World::new("test", 20.0);
        w.spawn_mob(vec3(0.0, 0.0, -1.3));
        w.player.facing = std::f32::consts::PI;
        let mut landed = false;
        for i in 0..20 {
            let ev = w.step(
                &Actions {
                    attack: i == 0,
                    ..Default::default()
                },
                &t,
            );
            if ev.iter().any(|e| matches!(e, Event::Hit { .. })) {
                landed = true;
                break;
            }
        }
        assert!(landed);
        assert!((w.hitstop - t[P::Hitstop] * HIT_HITSTOP).abs() < 1e-5);
        assert!(w.hitstop >= 0.09, "about six frames, not three");
        assert!(!w.sparks.is_empty(), "sparks fly");
        let away = w.sparks.iter().map(|s| s.vel.z).sum::<f32>();
        assert!(away < 0.0, "they fly the way the blow went (-z)");
        assert!(w.enemies[0].sway.length() > 0.2, "the body gives");
        while w.hitstop > 0.0 {
            w.step(&Actions::default(), &t);
        }
        w.step(&Actions::default(), &t);
        assert!(w.player.vel.length() < 1e-4, "the lunge stops on contact");
    }

    /// 2026-11-07: 무기마다 박자 = 그 타의 한 스윙 시간. 박자 창(± `BEAT_WINDOW`) 안에 **누른** 다음 평타는 스태미나를 쓰지 않는다.
    /// 일찍 눌러 받아 둔 것·연타는 박자에 나가도 쓴다(연타가 공짜가 되지 않게), 첫 타도 쓴다.
    #[test]
    fn an_attack_pressed_on_the_weapons_beat_costs_no_stamina_but_mashing_does() {
        assert!(on_beat(0.43, 0.43) && on_beat(0.43 * 1.14, 0.43) && !on_beat(0.43 * 1.2, 0.43));
        assert!(!on_beat(0.2, 0.43) && !on_beat(0.43, 0.0));
        let t = Tuning::default();
        for weapon in [weapons::Id::Sword, weapons::Id::Staff, weapons::Id::Hammer] {
            // 박자에 맞춰 누름.
            let mut w = World::new("test", 20.0);
            w.set_weapon(weapon);
            w.step(
                &Actions {
                    attack: true,
                    ..Default::default()
                },
                &t,
            );
            let after_first = w.player.stamina;
            assert!(after_first < t[P::StaminaMax], "the first swing costs");
            let beat = w.player.beat;
            assert!(beat > 0.25, "{weapon:?} beat {beat}");
            let mut free = false;
            for _ in 0..120 {
                let press = w.player.since_swing >= beat - TICK_DT * 0.5
                    && w.player.since_swing < beat + TICK_DT;
                let before = w.player.stamina;
                let ev = w.step(
                    &Actions {
                        attack: press,
                        ..Default::default()
                    },
                    &t,
                );
                if press {
                    assert!(ev.contains(&Event::Beat), "{weapon:?}: on the beat");
                    assert!(
                        w.player.stamina >= before,
                        "{weapon:?}: no stamina for the beat swing"
                    );
                    // 피드백(0076): 이 스윙은 박자 타, 잇단 수 1, 고리·음표 시계가 0부터.
                    assert!(
                        w.player.swing_beat && w.player.beat_chain == 1,
                        "{weapon:?}"
                    );
                    assert!(w.player.since_beat < TICK_DT * 2.0);
                    free = true;
                    break;
                }
            }
            assert!(free, "{weapon:?}");
            // 연타(매 틱 누름): 박자에 나가는 일이 있어도 받아 둔 이른 누름이라 늘 쓴다.
            let mut w = World::new("test", 20.0);
            w.set_weapon(weapon);
            let mut beats = 0;
            for _ in 0..90 {
                let ev = w.step(
                    &Actions {
                        attack: true,
                        ..Default::default()
                    },
                    &t,
                );
                beats += ev.iter().filter(|e| **e == Event::Beat).count();
            }
            assert_eq!(beats, 0, "{weapon:?}: mashing never gets the beat for free");
            assert!(!w.player.swing_beat && w.player.beat_chain == 0);
        }
    }

    /// 0087: 박자 부채꼴은 나타난 뒤 줄기만 하고, 박자 순간에 가장 작으며(그 뒤 창이 닫힐 때까지 그대로), 금빛(ripe) = 박자 창 안.
    #[test]
    fn the_beat_fan_shrinks_to_its_smallest_on_the_beat() {
        let (start, beat) = (0.08, 0.43);
        assert_eq!(
            beat_shrink(0.05, start, beat),
            None,
            "not before it appears"
        );
        assert_eq!(beat_shrink(start, start, beat).map(|c| c.0), Some(1.0));
        let mut last = 2.0;
        let mut k = start;
        while k <= beat {
            let (left, ripe) = beat_shrink(k, start, beat).unwrap();
            assert!(left <= last, "only shrinks");
            assert_eq!(
                ripe,
                on_beat(k, beat),
                "gold exactly inside the window at {k}"
            );
            last = left;
            k += 0.005;
        }
        let (min, ripe) = beat_shrink(beat, start, beat).unwrap();
        assert!((min - BEAT_CUE_MIN).abs() < 1e-5 && ripe);
        let (held, ripe) = beat_shrink(beat + 0.05, start, beat).unwrap();
        assert!(
            (held - BEAT_CUE_MIN).abs() < 1e-5 && ripe,
            "held smallest until the window closes"
        );
        assert_eq!(
            beat_shrink(beat + 0.1, start, beat),
            None,
            "gone once the window closes"
        );
        assert_eq!(beat_shrink(0.2, 0.0, 0.0), None, "no beat, no fan");
    }

    /// 0087: 검 1타(오른쪽 → 왼쪽)는 활성부터 박자 부채꼴, 박자에 금빛으로 가장 작고, 그때 누르면 박자 타.
    /// 이을 타가 없는 마지막 타엔 없다. 쇠뇌는 쏜 순간부터 좁은 부채꼴이 가운데로 줄어든다.
    #[test]
    fn the_beat_fan_follows_the_swing_and_peaks_when_a_press_counts() {
        let t = Tuning::default();
        let press = |w: &mut World, attack: bool, cast: bool| {
            w.step(
                &Actions {
                    attack,
                    cast,
                    ..Default::default()
                },
                &t,
            )
        };
        let mut w = World::new("test", 20.0);
        w.set_weapon(weapons::Id::Sword);
        assert_eq!(w.beat_cue(&t), None);
        press(&mut w, true, false);
        let (s0, _, _) = hit_times(&t, w.player.weapon.def(), 0);
        let mut seen = None;
        let mut beat = false;
        for _ in 0..90 {
            let cue = w.beat_cue(&t);
            if w.player.since_swing < s0 - 1e-4 {
                assert_eq!(cue, None, "not before the hit is live");
            }
            if let Some(c) = cue {
                assert_eq!(c.sweep, weapons::Sweep::RightToLeft);
                seen = Some(c);
                if c.ripe && (w.player.since_swing - w.player.beat).abs() < TICK_DT * 0.6 {
                    assert!((c.left - BEAT_CUE_MIN).abs() < 0.05, "smallest on the beat");
                    let ev = press(&mut w, true, false);
                    assert!(
                        ev.contains(&Event::Beat),
                        "a press while it is smallest is on the beat"
                    );
                    beat = true;
                    break;
                }
            }
            press(&mut w, false, false);
        }
        assert!(seen.is_some() && beat);
        // 둘째 타(왼쪽 → 오른쪽)를 박자로 쳤다 — 이제 부채꼴은 그 타의 쪽.
        for _ in 0..12 {
            press(&mut w, false, false);
        }
        assert_eq!(
            w.beat_cue(&t).map(|c| c.sweep),
            Some(weapons::Sweep::LeftToRight)
        );
        // 셋째(마지막) 타 찌르기: 이을 타가 없으니 박자도 없다.
        let mut w = World::new("test", 20.0);
        w.set_weapon(weapons::Id::Sword);
        w.player.combo_next = 2;
        w.player.combo_left = 1.0;
        press(&mut w, true, false);
        assert!(matches!(w.player.state, PState::Attack { hit: 2, .. }));
        for _ in 0..60 {
            assert_eq!(w.beat_cue(&t), None, "the finisher has no beat");
            press(&mut w, false, false);
        }
        // 쇠뇌: 쏜 순간부터, 가운데로.
        let mut w = World::new("test", 20.0);
        w.set_weapon(weapons::Id::Crossbow);
        press(&mut w, false, true);
        let c = w.beat_cue(&t).expect("a shot starts its beat fan");
        assert_eq!(c.sweep, weapons::Sweep::Center);
        assert!(c.left > 0.9 && !c.ripe);
    }

    /// 2026-11-06: 회피를 끊고(또는 막 끝나고) 누른 공격은 무기의 대시 공격 — 멀리 내딛고, 다음 타는 콤보 둘째 타.
    #[test]
    fn attacking_out_of_a_dodge_is_a_dash_attack_that_chains_into_the_second_hit() {
        let t = Tuning::default();
        let mut w = World::new("test", 20.0);
        w.step(
            &Actions {
                dodge: true,
                move_dir: vec2(0.0, 1.0),
                ..Default::default()
            },
            &t,
        );
        assert!(matches!(w.player.state, PState::Dodge { .. }));
        let mut dashed = false;
        for i in 0..40 {
            let a = Actions {
                attack: i == 8,
                ..Default::default()
            };
            w.step(&a, &t);
            if let PState::Attack { hit, .. } = w.player.state {
                assert_eq!(
                    hit,
                    weapons::DASH,
                    "the attack out of a dodge is the dash attack"
                );
                dashed = true;
                break;
            }
        }
        assert!(dashed);
        let wpn = w.player.weapon.def();
        assert_eq!(
            wpn.after(weapons::DASH),
            Some(1),
            "then the combo goes on from its second hit"
        );
        // 그냥 선 채로 누른 공격은 첫 타.
        let mut w = World::new("test", 20.0);
        w.step(
            &Actions {
                attack: true,
                ..Default::default()
            },
            &t,
        );
        assert!(matches!(w.player.state, PState::Attack { hit: 0, .. }));
    }

    /// 2026-11-06: 패링으로 튕긴 직후의 공격은 모으기 없이 다 모은 강타(반격). 창이 지나면 평타.
    #[test]
    fn attacking_right_after_a_parry_is_a_full_heavy_riposte() {
        let t = Tuning::default();
        let mut w = mob_swinging_in_front(&t);
        w.player.state = PState::Parry { t: 0.0 };
        let mut parried = false;
        for _ in 0..60 {
            parried |= w
                .step(&Actions::default(), &t)
                .iter()
                .any(|e| matches!(e, Event::Parry { .. }));
            if parried {
                break;
            }
        }
        assert!(parried);
        // 세상이 한 박자 멈춘 동안(히트스톱)은 틱이 돌지 않는다 — 풀리고 누른다.
        while w.hitstop > 0.0 {
            w.step(&Actions::default(), &t);
        }
        w.step(
            &Actions {
                attack: true,
                ..Default::default()
            },
            &t,
        );
        assert!(
            matches!(w.player.state, PState::Heavy { full: true, .. }),
            "riposte: {:?}",
            w.player.state
        );
        // 창이 지난 뒤면 평타.
        let mut w = World::new("test", 20.0);
        w.player.since_parry = RIPOSTE_WINDOW + 0.1;
        w.step(
            &Actions {
                attack: true,
                ..Default::default()
            },
            &t,
        );
        assert!(matches!(w.player.state, PState::Attack { .. }));
    }

    /// 2026-11-06: 락온한 적이 쓰러지면 그 자리에서 가장 가까운 적으로 넘어가고, 둘레에 아무도 없으면 풀린다.
    #[test]
    fn a_fallen_lock_target_hands_off_to_the_nearest_enemy_or_lets_go() {
        let t = Tuning::default();
        let mut w = World::new("test", 20.0);
        w.spawn_mob(vec3(0.0, 0.0, -4.0)); // 0: 락온 대상
        w.spawn_mob(vec3(9.0, 0.0, 0.0)); // 1: 플레이어에게는 가깝지만 대상 자리에서 멀다
        w.spawn_mob(vec3(2.0, 0.0, -7.0)); // 2: 대상 자리에서 가장 가깝다
        w.lock = Some(0);
        w.enemies[0].hp = 0;
        w.enemies[0].state = EState::Dead;
        let ev = w.step(&Actions::default(), &t);
        assert_eq!(w.lock, Some(2), "next to where the target fell");
        assert!(!ev.contains(&Event::LockOff), "stays in lock-on");
        // 그 적도 쓰러지고, 남은 적(1)은 둘레 밖 → 풀린다.
        w.enemies[2].hp = 0;
        w.enemies[2].state = EState::Dead;
        let ev = w.step(&Actions::default(), &t);
        assert_eq!(w.lock, None);
        assert!(ev.contains(&Event::LockOff));
    }

    fn swing_at(w: &mut World, aim: Vec2) {
        let t = Tuning::default();
        w.step(
            &Actions {
                attack: true,
                aim,
                ..Default::default()
            },
            &t,
        );
    }

    #[test]
    fn an_attack_turns_to_the_nearest_enemy_within_reach() {
        let t = Tuning::default();
        let mut w = World::new("test", 30.0);
        w.view_yaw = Some(0.0);
        w.spawn_mob(vec3(2.0, 0.0, 0.0)); // 가까운 적(오른쪽)
        w.spawn_mob(vec3(0.0, 0.0, -2.5)); // 조금 먼 적(앞)
        w.player.facing = std::f32::consts::PI; // -z를 본다
        swing_at(&mut w, Vec2::ZERO);
        assert!(
            w.player.facing_dir().x > 0.99,
            "{:?}",
            w.player.facing_dir()
        );
        // 범위 밖이면 돌지 않는다.
        let mut w = World::new("test", 30.0);
        w.spawn_mob(vec3(t[P::AimRange] + 2.0, 0.0, 0.0));
        w.player.facing = std::f32::consts::PI;
        swing_at(&mut w, Vec2::ZERO);
        assert!(w.player.facing_dir().z < -0.99);
    }

    #[test]
    fn with_the_aim_stick_held_the_attack_picks_the_nearest_enemy_that_way() {
        let mut w = World::new("test", 30.0);
        w.view_yaw = Some(0.0); // 화면 위 = -z
        w.spawn_mob(vec3(1.5, 0.0, 0.0)); // 가장 가깝지만 오른쪽
        w.spawn_mob(vec3(0.0, 0.0, -2.5)); // 스틱 쪽(위)
        swing_at(&mut w, vec2(0.0, 1.0));
        assert!(
            w.player.facing_dir().z < -0.99,
            "{:?}",
            w.player.facing_dir()
        );
        // 그 쪽에 적이 없으면 스틱 쪽 그대로.
        let mut w = World::new("test", 30.0);
        w.view_yaw = Some(0.0);
        w.spawn_mob(vec3(1.5, 0.0, 0.0));
        swing_at(&mut w, vec2(-1.0, 0.0));
        assert!(
            w.player.facing_dir().x < -0.99,
            "{:?}",
            w.player.facing_dir()
        );
    }

    #[test]
    fn the_right_stick_turns_the_player_without_moving() {
        let t = Tuning::default();
        let mut w = World::new("test", 30.0);
        w.view_yaw = Some(0.0);
        w.player.facing = std::f32::consts::PI;
        for _ in 0..30 {
            w.step(
                &Actions {
                    aim: vec2(1.0, 0.0),
                    ..Default::default()
                },
                &t,
            );
        }
        assert!(w.player.facing_dir().x > 0.99);
        assert!(w.player.pos.length() < 1e-4);
        // 걸으면서 다른 쪽을 본다.
        for _ in 0..30 {
            w.step(
                &Actions {
                    move_dir: vec2(0.0, 1.0),
                    aim: vec2(-1.0, 0.0),
                    ..Default::default()
                },
                &t,
            );
        }
        assert!(w.player.facing_dir().x < -0.99 && w.player.vel.z < -1.0);
    }

    /// 휘두르며 입력한 쪽으로 `attack.lunge m` 내딛는다(0018) — 휘두르는 쪽(자동 조준)과 달라도.
    #[test]
    fn an_attack_steps_toward_the_held_direction() {
        let t = Tuning::default();
        let mut w = World::new("test", 30.0);
        w.view_yaw = Some(0.0); // 화면 오른쪽 = +x
        w.step(
            &Actions {
                attack: true,
                move_dir: vec2(1.0, 0.0),
                ..Default::default()
            },
            &t,
        );
        let reach = t[P::AttackStartup] + t[P::AttackActive];
        for _ in 0..((reach / TICK_DT) as usize + 5) {
            w.step(
                &Actions {
                    move_dir: vec2(1.0, 0.0),
                    ..Default::default()
                },
                &t,
            );
        }
        let lunge = t[P::AttackLunge];
        assert!(
            (w.player.pos.x - lunge).abs() < 0.1 && w.player.pos.z.abs() < 1e-3,
            "{:?}",
            w.player.pos
        );
    }

    /// 진행 방향으로 내딛는다(0018): 옆으로 걷다 치면 옆으로 — 휘두르는 쪽(가까운 적)은 앞이어도. 입력을 막 놓아도 움직이던 쪽으로.
    #[test]
    fn the_lunge_follows_the_travel_direction_not_the_swing_direction() {
        let t = Tuning::default();
        let mut w = World::new("test", 30.0);
        w.view_yaw = Some(0.0); // 화면 위 = -z, 오른쪽 = +x
        w.spawn_mob(vec3(0.0, 0.0, -2.0)); // 앞에 적 → 휘두르는 쪽은 앞
        w.enemies[0].state = EState::Recover { t: -1000.0 };
        let right = Actions {
            move_dir: vec2(1.0, 0.0),
            ..Default::default()
        };
        for _ in 0..20 {
            w.step(&right, &t);
        }
        let x0 = w.player.pos.x;
        let to_enemy = (w.enemies[0].pos - w.player.pos).normalize();
        // 입력을 놓는 틱에 공격: 아직 오른쪽으로 움직이던 중.
        w.step(
            &Actions {
                attack: true,
                ..Default::default()
            },
            &t,
        );
        assert!(
            w.player.facing_dir().dot(to_enemy) > 0.99,
            "swing faces the enemy"
        );
        for _ in 0..12 {
            w.step(&Actions::default(), &t);
        }
        assert!(
            w.player.pos.x - x0 > t[P::AttackLunge] * 0.8,
            "{} -> {}",
            x0,
            w.player.pos.x
        );
        assert!(w.player.pos.z.abs() < 0.05);
    }

    fn hit_by_swing(w: &mut World, t: &Tuning) -> Vec<Event> {
        let i = w.enemies.len();
        w.spawn_mob(w.player.pos + vec3(0.0, 0.0, -1.0));
        w.enemies[i].facing = 0.0; // +z = 플레이어 쪽
        w.enemies[i].state = EState::Swing { t: 0.0 };
        let ev = w.step(&Actions::default(), t);
        w.enemies[i].state = EState::Dead;
        ev
    }

    /// 맞으면 잠깐 경직(못 움직임) — 경직 중 또 맞으면 반씩 짧아지고, 이음 창(0099)까지 지난 뒤 맞으면 다시 처음 길이(0018).
    #[test]
    fn a_hit_stuns_briefly_and_hits_during_the_stun_halve_it() {
        let t = Tuning::default();
        let mut w = World::new("test", 30.0);
        let ev = hit_by_swing(&mut w, &t);
        assert!(ev.contains(&Event::PlayerHurt));
        let PState::Hurt { dur: d0, .. } = w.player.state else {
            panic!("{:?}", w.player.state)
        };
        assert!((d0 - t[P::HitStun]).abs() < 1e-4);
        // 경직 중엔 걷지 못한다(밀려난 것만 줄어든다).
        let before = w.player.pos;
        for _ in 0..3 {
            w.step(
                &Actions {
                    move_dir: vec2(1.0, 0.0),
                    ..Default::default()
                },
                &t,
            );
        }
        assert!((w.player.pos.x - before.x).abs() < 1e-3);
        // 무적이 끝나고 아직 경직 중일 때 또 맞는다 → 반.
        while w.player.hurt > 0.0 {
            w.step(&Actions::default(), &t);
        }
        assert!(matches!(w.player.state, PState::Hurt { .. }));
        hit_by_swing(&mut w, &t);
        let PState::Hurt { dur: d1, .. } = w.player.state else {
            panic!()
        };
        assert!((d1 - d0 * 0.5).abs() < 1e-4, "{d1} vs {d0}");
        // 풀린 뒤 맞으면 다시 처음 길이.
        for _ in 0..60 {
            w.step(&Actions::default(), &t);
        }
        assert_eq!(w.player.state, PState::Idle);
        hit_by_swing(&mut w, &t);
        let PState::Hurt { dur: d2, .. } = w.player.state else {
            panic!()
        };
        assert!((d2 - d0).abs() < 1e-4);
    }

    /// 0099(사장 "다구리 맞더라도 틈이 생겨서 도망갈 수 있게"): 경직이 풀린 뒤에도 `STUN_CHAIN_GRACE` 안에 또 맞으면 경직이 그전의 반,
    /// 창이 지나면 다시 처음 길이. 일정한 박자로 계속 맞으면 경직이 줄어 맞는 사이사이 움직일 틈이 커진다.
    #[test]
    fn hits_soon_after_a_stun_keep_halving_it_so_a_mob_leaves_gaps() {
        let stun = 0.35;
        assert_eq!(
            stun_chain_next(0.0, 3, stun).0,
            0,
            "window closed: from the top"
        );
        let (n, w) = stun_chain_next(0.1, 0, stun);
        assert_eq!(n, 1);
        assert!((w - (stun * 0.5 + STUN_CHAIN_GRACE)).abs() < 1e-5);
        // 0.4초마다 맞는 몰매: 맞을 때마다 경직이 짧아지고, 맞은 사이에 움직일 수 있는 시간은 늘어난다.
        let (mut n, mut w) = (0, 0.0);
        let mut free = Vec::new();
        for _ in 0..4 {
            (n, w) = stun_chain_next(w, n, stun);
            let dur = stun * 0.5f32.powi(n);
            free.push(0.4 - dur);
            w -= 0.4;
        }
        assert!(free.windows(2).all(|f| f[1] >= f[0]), "{free:?}");
        assert!(free[0] > 0.0 && free[3] > 0.3);
        // 바닥(0099 사장 "바닥을 두자"): 두 번 이은 뒤로는 처음의 1/4에서 멈춘다 — 맞은 몸의 반응은 남는다.
        assert_eq!(n, STUN_CHAIN_MAX);
        assert!((stun * 0.5f32.powi(n) - stun * 0.25).abs() < 1e-6);

        // 월드에서: 경직이 풀린 뒤(서 있음) 창 안에 맞으면 반, 창이 지나면 처음 길이.
        let t = Tuning::default();
        let mut w = World::new("test", 30.0);
        hit_by_swing(&mut w, &t);
        while !matches!(w.player.state, PState::Idle) {
            w.step(&Actions::default(), &t);
        }
        assert!(w.player.stun_window > 0.0, "still inside the grace");
        hit_by_swing(&mut w, &t);
        let PState::Hurt { dur, .. } = w.player.state else {
            panic!("{:?}", w.player.state)
        };
        assert!((dur - t[P::HitStun] * 0.5).abs() < 1e-4, "{dur}");
        for _ in 0..((t[P::HitStun] + STUN_CHAIN_GRACE) / TICK_DT) as usize + 5 {
            w.step(&Actions::default(), &t);
        }
        hit_by_swing(&mut w, &t);
        let PState::Hurt { dur, .. } = w.player.state else {
            panic!()
        };
        assert!(
            (dur - t[P::HitStun]).abs() < 1e-4,
            "fresh after the grace: {dur}"
        );
    }

    /// 회피·공격은 스태미나를 쓰고, 없으면 막히며(막대가 깜빡인다) 마지막 사용 뒤 잠깐 쉬면 다시 찬다(0018).
    #[test]
    fn stamina_gates_dodges_and_comes_back_after_a_pause() {
        let t = Tuning::default();
        let mut w = World::new("test", 30.0);
        let full = w.player.stamina;
        let dodge = Actions {
            dodge: true,
            ..Default::default()
        };
        let mut dodges = 0;
        for _ in 0..600 {
            if w.player.state == PState::Idle && w.player.dodge_cd <= 0.0 {
                let before = w.player.stamina;
                w.step(&dodge, &t);
                if matches!(w.player.state, PState::Dodge { .. }) {
                    dodges += 1;
                    assert!(w.player.stamina < before);
                } else {
                    break;
                }
            } else {
                w.step(&Actions::default(), &t);
            }
            if w.player.stamina <= 0.0 {
                break;
            }
        }
        let max_dodges = (full / t[P::StaminaDodge]).ceil() as i32;
        assert!(dodges <= max_dodges && dodges >= 2, "{dodges}");
        // 다 쓰면 막히고 막대가 깜빡인다.
        while !(w.player.state == PState::Idle && w.player.dodge_cd <= 0.0) {
            w.step(&Actions::default(), &t);
        }
        if w.player.stamina <= 0.0 {
            w.step(&dodge, &t);
            assert!(!matches!(w.player.state, PState::Dodge { .. }));
            assert!(w.player.stamina_denied > 0.0);
        }
        // 쉬면 찬다.
        for _ in 0..((t[P::StaminaDelay] + full / t[P::StaminaRegen]) / TICK_DT) as usize + 10 {
            w.step(&Actions::default(), &t);
        }
        assert!((w.player.stamina - t[P::StaminaMax]).abs() < 1e-3);
    }

    /// 높낮이 시험 마당(0043): 5×6칸(2m). 맨 윗줄(가장 -z)은 높이 `ledge`인 단, 나머지는 바닥 0.
    fn ledge_yard(ledge: f32) -> World {
        use crate::ground::Cell::Flat;
        let mut cells = vec![Flat(0.0); 5 * 6];
        for c in cells.iter_mut().take(5) {
            *c = Flat(ledge);
        }
        let mut w = World::new("test", 30.0);
        w.ground = Some(Ground::new(5, 6, 2.0, cells));
        w.view_yaw = Some(0.0);
        w.player.pos = vec3(0.0, 0.0, 3.0);
        w
    }

    fn up() -> Actions {
        Actions {
            move_dir: vec2(0.0, 1.0),
            ..Default::default()
        }
    }

    /// 걸어서는 0.5m 턱에 막히고, 달려와 뛰면 올라선다(사장: "경사로 + 낮은 턱 점프").
    #[test]
    fn a_low_ledge_blocks_walking_and_a_jump_climbs_it() {
        let t = Tuning::default();
        let mut w = ledge_yard(0.5);
        for _ in 0..150 {
            w.step(&up(), &t);
        }
        assert_eq!(w.player.pos.y, 0.0, "walking stays below");
        let edge = -6.0 + 2.0; // 단 칸의 아래 변(z)
        assert!(
            w.player.pos.z > edge,
            "stopped at the ledge: z {}",
            w.player.pos.z
        );
        // 조금 물러났다가 달려와 뛴다.
        w.player.pos.z = edge + 2.0;
        for _ in 0..10 {
            w.step(&up(), &t);
        }
        w.step(&Actions { jump: true, ..up() }, &t);
        while w.player.airborne() {
            w.step(&up(), &t);
        }
        // 내려앉은 뒤에는 손을 놓는다(시험 격자 밖으로 걸어 나가지 않게).
        for _ in 0..30 {
            w.step(&Actions::default(), &t);
        }
        assert!(
            (w.player.pos.y - 0.5).abs() < 1e-4 && w.player.air == 0.0,
            "stands on the ledge: y {} air {}",
            w.player.pos.y,
            w.player.air
        );
        assert!(w.player.pos.z < edge);
    }

    /// 1m 벼랑은 점프(0.9m)로도 못 오른다.
    #[test]
    fn a_tall_cliff_cannot_be_jumped() {
        let t = Tuning::default();
        let mut w = ledge_yard(1.0);
        let edge = -4.0;
        w.player.pos.z = edge + 2.0;
        for _ in 0..10 {
            w.step(&up(), &t);
        }
        w.step(&Actions { jump: true, ..up() }, &t);
        while w.player.airborne() {
            w.step(&up(), &t);
        }
        assert_eq!(w.player.pos.y, 0.0);
        assert!(w.player.pos.z > edge);
    }

    /// 단 위에서 걸어 나가면 떨어져 아래에 선다(몸 높이는 이어진다).
    #[test]
    fn walking_off_a_ledge_falls_and_lands_below() {
        let t = Tuning::default();
        let mut w = ledge_yard(1.0);
        w.player.pos = vec3(0.0, 1.0, -5.0);
        w.step(&Actions::default(), &t);
        assert_eq!(w.player.pos.y, 1.0);
        let down = Actions {
            move_dir: vec2(0.0, -1.0),
            ..Default::default()
        };
        let mut flew = false;
        let mut lowest_body = f32::MAX;
        for _ in 0..90 {
            let body = w.player.pos.y + w.player.air;
            w.step(&down, &t);
            let now = w.player.pos.y + w.player.air;
            assert!(
                now <= body + 1e-4 && body - now < 0.5,
                "body height is continuous: {body} → {now}"
            );
            flew |= w.player.air > 0.2;
            lowest_body = lowest_body.min(now);
        }
        assert!(flew, "fell through the air");
        assert_eq!((w.player.pos.y, w.player.air), (0.0, 0.0));
    }

    /// 0100(사장 "적들도 높은 곳으로 점프하거나 낮은 곳으로 뛰어내릴 수 있어야"): 경사로 없는 1m 단 위의 플레이어에게 아래 잔재가
    /// 뛰어올라 와 친다. 단 위의 잔재는 아래 플레이어에게 뛰어내려 온다. 뛰는 동안은 치지 않는다.
    #[test]
    fn enemies_hop_up_and_drop_down_ledges_to_reach_you() {
        let t = Tuning::default();
        // 뛰어오르기.
        let mut w = ledge_yard(1.0);
        w.player.pos = vec3(0.0, 1.0, -5.0);
        let i = w.spawn(Kind::Husk, vec3(0.0, 0.0, 3.0));
        let (mut hopped, mut on_top) = (false, false);
        for _ in 0..900 {
            w.step(&Actions::default(), &t);
            if let Some(h) = w.enemies[i].hop {
                hopped = true;
                assert!(h.to.y > h.from.y + 0.9, "up the 1m ledge: {h:?}");
            }
            on_top |= w.enemies[i].hop.is_none() && (w.enemies[i].pos.y - 1.0).abs() < 1e-3;
            if w.player.hp < PLAYER_HP {
                break;
            }
        }
        assert!(hopped && on_top, "hopped up: {:?}", w.enemies[i].pos);
        assert!(w.player.hp < PLAYER_HP, "and reached the player");
        // 뛰어내리기.
        let mut w = ledge_yard(1.0);
        w.player.pos = vec3(0.0, 0.0, 3.0);
        let i = w.spawn(Kind::Husk, vec3(0.0, 1.0, -5.0));
        let mut dropped = false;
        for _ in 0..900 {
            w.step(&Actions::default(), &t);
            if let Some(h) = w.enemies[i].hop {
                assert!(h.to.y < h.from.y - 0.9, "down: {h:?}");
                dropped = true;
            }
            if w.player.hp < PLAYER_HP {
                break;
            }
        }
        assert!(dropped, "dropped down: {:?}", w.enemies[i].pos);
        assert!(w.player.hp < PLAYER_HP, "and reached the player");
    }

    /// 0100: 뛰는 포물선은 두 끝을 지나고, 오를 때는 턱 위로 넘어간다(가운데 높이가 위 끝보다 높다).
    #[test]
    fn a_hop_arcs_over_the_ledge_and_lands_where_it_aimed() {
        let h = crate::enemy::Hop::new(vec3(0.0, 0.0, 0.0), vec3(0.0, 1.0, -1.2));
        assert_eq!(h.at(0.0), h.from);
        assert!((h.at(1.0) - h.to).length() < 1e-5);
        assert!(h.at(0.5).y > 1.0, "clears the lip: {:?}", h.at(0.5));
        let d = crate::enemy::Hop::new(vec3(0.0, 2.0, 0.0), vec3(0.0, 0.0, 1.0));
        assert!(d.dur < h.dur, "dropping is quicker");
    }

    /// 높이가 다른 단: 아래 잔재의 발톱도, 플레이어의 칼도 닿지 않는다. 뛰어오를 수 없는 높이(1.5m — `ENEMY_CLIMB` 1m 위)면
    /// 경사로가 없을 때 적은 아래에서 기다린다(0100 전에는 1m 단도 그랬다).
    #[test]
    fn different_levels_do_not_trade_blows() {
        let t = Tuning::default();
        // 아래 잔재는 1.5m 단 위에 가만히 선 플레이어를 못 친다.
        let mut w = ledge_yard(1.5);
        w.player.pos = vec3(0.0, 1.5, -4.4);
        let i = w.spawn(Kind::Husk, vec3(0.0, 0.0, -3.0));
        for _ in 0..400 {
            w.step(&Actions::default(), &t);
        }
        assert_eq!(w.player.hp, PLAYER_HP);
        assert!(
            w.enemies[i].pos.y == 0.0 && w.enemies[i].pos.z > -4.0,
            "the husk stays below: {:?}",
            w.enemies[i].pos
        );
        // 단 끝에서 아래 잔재 쪽으로 휘둘러도 안 닿는다(수평으로는 닿는 거리).
        let mut w = ledge_yard(1.0);
        w.player.pos = vec3(0.0, 1.0, -4.5);
        w.player.facing = 0.0;
        let i = w.spawn(Kind::Husk, vec3(0.0, 0.0, -3.2));
        w.enemies[i].state = EState::Dormant;
        w.step(
            &Actions {
                attack: true,
                ..Default::default()
            },
            &t,
        );
        for _ in 0..40 {
            w.step(&Actions::default(), &t);
        }
        assert_eq!(w.enemies[i].hp, w.enemies[i].max_hp);
    }

    /// 경사로가 있으면 적은 돌아서 단 위로 올라온다.
    #[test]
    fn enemies_take_the_ramp_up_to_the_player() {
        use crate::ground::Cell::{Flat, Ramp};
        let t = Tuning::default();
        let mut w = ledge_yard(1.0);
        // 맨 오른쪽 열 둘째 줄을 경사로(0 → 1, -z 쪽이 높다)로.
        if let Some(g) = &mut w.ground {
            g.cells[5 + 4] = Ramp {
                x_axis: false,
                from: 1.0,
                to: 0.0,
            };
            g.cells[4] = Flat(1.0);
        }
        w.player.pos = vec3(-4.0, 1.0, -5.0);
        let i = w.spawn(Kind::Husk, vec3(-4.0, 0.0, 3.0));
        let mut climbed = false;
        for _ in 0..900 {
            w.step(&Actions::default(), &t);
            climbed |= w.enemies[i].pos.y > 0.99 && w.enemies[i].pos.z < -4.0;
            if w.player.hp < PLAYER_HP {
                break;
            }
        }
        assert!(climbed, "climbed the ramp: {:?}", w.enemies[i].pos);
        assert!(w.player.hp < PLAYER_HP, "and reached the player");
    }

    #[test]
    fn pillars_push_actors_out() {
        let b = Aabb {
            min: vec3(-1.0, 0.0, -1.0),
            max: vec3(1.0, 2.0, 1.0),
        };
        let p = b.push_circle(vec3(0.9, 0.0, 0.0), 0.3);
        assert!((p.x - 1.3).abs() < 1e-5, "{p:?}");
        assert_eq!(b.ray(vec3(0.0, 1.0, 5.0), vec3(0.0, 0.0, -1.0)), Some(4.0));
        assert_eq!(b.ray(vec3(3.0, 1.0, 5.0), vec3(0.0, 0.0, -1.0)), None);
    }

    /// 사장 2026-11-07 "적들의 길찾기가 좀 멍청함"(0076): 기둥이 사이에 있으면 박혀 비비지 않고 돌아와 붙는다.
    #[test]
    fn an_enemy_walks_around_a_pillar_to_reach_you() {
        let t = Tuning::default();
        let mut w = World::new("test", 12.0);
        w.obstacles.push(Aabb {
            min: vec3(-1.5, 0.0, -3.0),
            max: vec3(1.5, 3.0, -1.5),
        });
        w.spawn(Kind::Husk, vec3(0.0, 0.0, -6.0));
        let mut reached = None;
        for tick in 0..600 {
            w.step(&Actions::default(), &t);
            let en = &w.enemies[0];
            if matches!(en.state, EState::Tele { .. }) {
                reached = Some(tick);
                break;
            }
        }
        // 잔재 2.6m/s, 돌아가는 길 ≈ 7m → 3초 남짓.
        let tick = reached.expect("got to the player and wound up");
        assert!(tick < 240, "took {tick} ticks");
    }

    /// 0076: 돌진 예고 띠 = 맞는 곳. 띠 가장자리 바로 안은 맞고 바로 밖은 안 맞는다(폭도 끝도).
    #[test]
    fn the_charge_lane_is_exactly_where_the_charge_hits() {
        let t = Tuning::default();
        // (옆 가장자리에서 바깥으로 m, 끝에서 앞으로 m — 둘 중 하나만, 둘 다 None이면 띠 가운데).
        for (side, along, hit) in [
            (None, None, true),
            (Some(-0.05), None, true),
            (Some(0.05), None, false),
            (None, Some(-0.05), true),
            (None, Some(0.05), false),
        ] {
            let mut w = World::new("test", 20.0);
            w.player.pos = vec3(0.0, 0.0, 0.0);
            let i = w.spawn(Kind::Thorn, vec3(0.0, 0.0, -6.5));
            // 예고가 굳을 때까지 돌리고 띠를 읽는다.
            let mut lane = None;
            for _ in 0..60 {
                w.step(&Actions::default(), &t);
                if let Some((_, TeleShape::Lane { len, w: lw })) = w.enemies[i].telegraph() {
                    lane = Some((len, lw));
                }
                if matches!(w.enemies[i].state, EState::Charge { .. }) {
                    break;
                }
            }
            let (len, lw) = lane.expect("a lane");
            // 플레이어를 띠 가장자리(옆) 또는 끝(앞)으로 옮겨 놓고 돌진을 맞는다.
            let en = &w.enemies[i];
            let f = en.facing_dir();
            let right = vec3(-f.z, 0.0, f.x);
            let at = match (side, along) {
                (Some(s), _) => en.pos + f * (len * 0.5) + right * (lw * 0.5 + s),
                (_, Some(a)) => en.pos + f * (len + a),
                _ => en.pos + f * (len * 0.5),
            };
            w.player.pos = at;
            w.player.prev = at;
            let mut got = false;
            for _ in 0..60 {
                let before = w.player.hp;
                w.step(&Actions::default(), &t);
                got |= w.player.hp < before;
                w.player.pos = at;
            }
            assert_eq!(got, hit, "side {side:?} along {along:?}: lane {len}×{lw}");
        }
    }

    /// 0076: 돌진은 기둥 앞에서 멈추고 예고 띠도 거기까지(띠가 기둥을 뚫고 그려지던 것).
    #[test]
    fn a_charge_stops_at_a_pillar_and_its_lane_says_so() {
        let t = Tuning::default();
        let mut w = World::new("test", 20.0);
        let i = w.spawn(Kind::Thorn, vec3(0.0, 0.0, -6.0));
        w.step(&Actions::default(), &t);
        assert!(matches!(w.enemies[i].state, EState::Tele { .. }));
        // 예고가 시작된 뒤 사이에 기둥이 생겼다(플레이어가 기둥 뒤로 숨은 것과 같다).
        w.obstacles.push(Aabb {
            min: vec3(-1.0, 0.0, -3.0),
            max: vec3(1.0, 3.0, -2.0),
        });
        w.step(&Actions::default(), &t);
        let Some((_, TeleShape::Lane { len, .. })) = w.enemies[i].telegraph() else {
            panic!("lane")
        };
        let room = 6.0 - 3.0 - w.enemies[i].radius;
        assert!(
            (len - (room + w.enemies[i].touch())).abs() < 0.1,
            "lane {len}"
        );
        for _ in 0..120 {
            w.step(&Actions::default(), &t);
        }
        assert!(
            w.enemies[i].pos.z < -3.0 + 0.01,
            "stopped before the pillar: {:?}",
            w.enemies[i].pos
        );
        assert_eq!(w.player.hp, w.player.max_hp, "never reached you");
    }

    /// 0076(사장 2026-11-07 "회피 꾹 눌렀을때 달리는거"): 톡 누름은 회피만, 회피가 끝날 때까지 쥐고 있으면 달린다 —
    /// 걷기보다 빠르고 스태미나를 쓰며, 떼면 걷는다. 달리다 치면 대시 공격.
    #[test]
    fn holding_dodge_runs_after_the_dodge_and_a_tap_does_not() {
        let t = Tuning::default();
        let up = vec2(0.0, 1.0);
        let run = |held_ticks: usize| -> World {
            let mut w = World::new("test", 40.0);
            for i in 0..90 {
                w.step(
                    &Actions {
                        dodge: i == 0,
                        dodge_held: i < held_ticks,
                        move_dir: up,
                        ..Default::default()
                    },
                    &t,
                );
            }
            w
        };
        // 톡(0.1초): 회피(0.3초 남짓)보다 짧다 — 예전과 같이 걷는다.
        let tap = run(6);
        assert!(!tap.player.running);
        let walk = tap.player.vel.length();
        assert!((walk - t[P::MoveSpeed]).abs() < 0.3, "walks {walk}");
        // 쥠: 회피 뒤 달린다.
        let held = run(90);
        assert!(held.player.running, "runs while held");
        let speed = held.player.vel.length();
        assert!(speed > walk * 1.3, "faster: {speed} vs {walk}");
        assert!(
            held.player.stamina < tap.player.stamina,
            "running costs stamina"
        );
        // 떼면 다음 틱에 걷는다.
        let mut w = held;
        w.step(
            &Actions {
                move_dir: up,
                ..Default::default()
            },
            &t,
        );
        assert!(!w.player.running);
        // 달리다 치면 대시 공격.
        let mut w = run(90);
        w.step(
            &Actions {
                attack: true,
                dodge_held: true,
                move_dir: up,
                ..Default::default()
            },
            &t,
        );
        assert!(
            matches!(
                w.player.state,
                PState::Attack {
                    hit: weapons::DASH,
                    ..
                }
            ),
            "{:?}",
            w.player.state
        );
        assert!(!w.player.running);
    }

    /// 0076: 스태미나가 다하면 달리기가 끝난다(쥐고 있어도).
    #[test]
    fn running_stops_when_stamina_runs_out() {
        let t = Tuning::default();
        let mut w = World::new("test", 200.0);
        let secs = t[P::StaminaMax] / RUN_STAMINA + 1.0;
        let mut was = false;
        let mut ended_at = None;
        for i in 0..((secs / TICK_DT) as usize) {
            w.step(
                &Actions {
                    dodge: i == 0,
                    dodge_held: true,
                    move_dir: vec2(0.0, 1.0),
                    ..Default::default()
                },
                &t,
            );
            if was && !w.player.running && ended_at.is_none() {
                ended_at = Some(w.player.stamina);
            }
            was = w.player.running;
        }
        let left = ended_at.expect("the run ended while still held");
        assert!(left <= 1e-3, "it ended because stamina ran out: {left}");
    }

    /// 0076(사장 2026-11-07 "무기 장착하지 않았을 경우에는 맨손 전투"): 맨손도 콤보로 친다 — 가깝고 가볍다.
    #[test]
    fn bare_hands_still_fight() {
        let t = Tuning::default();
        let mut w = World::new("test", 20.0);
        w.set_weapon(weapons::Id::Fists);
        let i = mob_with(&mut w, vec3(0.0, 0.0, 1.0), 10);
        w.enemies[i].state = EState::Dormant;
        w.enemies[i].state = EState::Chase;
        let hp = w.enemies[i].hp;
        for k in 0..30 {
            w.step(
                &Actions {
                    attack: k == 0,
                    ..Default::default()
                },
                &t,
            );
        }
        assert_eq!(w.enemies[i].hp, hp - 1, "a punch lands for 1");
        let fists = weapons::Id::Fists.def();
        let sword = weapons::Id::Sword.def();
        assert!(
            fists.combo[0].reach < sword.combo[0].reach
                && fists.combo[0].time < sword.combo[0].time
        );
    }

    /// 0077(사장 2026-11-07 "원거리 무기도 박자 있는거야. 근데 버튼이 다를뿐"): 쇠뇌는 사격(RB)이 박자를 탄다 —
    /// 앞 사격에서 한 발의 시간(= 공격 속도) ± 창 안에 누른 사격은 볼트를 쓰지 않는다, 연사는 쓴다.
    #[test]
    fn a_crossbow_shot_on_the_beat_keeps_its_bolt_and_mashing_spends_them() {
        let t = Tuning::default();
        let shoot = |w: &mut World, press: bool| {
            w.step(
                &Actions {
                    cast: press,
                    ..Default::default()
                },
                &t,
            )
        };
        let mut w = World::new("test", 20.0);
        w.set_weapon(weapons::Id::Crossbow);
        let full = w.player.ammo;
        shoot(&mut w, true);
        assert_eq!(w.player.ammo, full - 1, "the first shot spends a bolt");
        let beat = w.player.beat;
        assert!(beat > 0.2, "beat {beat}");
        let mut got = false;
        for _ in 0..120 {
            let press = w.player.since_swing >= beat - TICK_DT * 0.5;
            let ev = shoot(&mut w, press);
            if press {
                assert!(ev.contains(&Event::Beat), "on the beat");
                assert_eq!(w.player.ammo, full - 1, "no bolt for the beat shot");
                assert!(w.player.beat_chain == 1 && w.player.since_beat < TICK_DT * 2.0);
                got = true;
                break;
            }
        }
        assert!(got);
        // 연사(매 틱 누름)는 박자를 공짜로 얻지 못한다.
        let mut w = World::new("test", 20.0);
        w.set_weapon(weapons::Id::Crossbow);
        let mut beats = 0;
        for _ in 0..30 {
            beats += shoot(&mut w, true)
                .iter()
                .filter(|e| **e == Event::Beat)
                .count();
        }
        assert_eq!(beats, 0);
        assert!(w.player.ammo < full, "mashing spends bolts");
        // 창은 아무리 빨라도 ±60ms 아래로 좁아지지 않는다.
        assert!(on_beat(0.24 + 0.055, 0.24) && !on_beat(0.24 + 0.07, 0.24));
    }

    /// 0081(사장 2026-11-07): 튕기면 패링 창이 `PARRY_CHAIN`만큼 이어져, 비슷한 때 오는 앞쪽 공격도 같이 튕긴다.
    /// 패링 동작의 창(0.18초)이 이미 닫힌 뒤에 오는 둘째 공격으로 본다. 이어지는 창보다 늦으면 맞는다.
    #[test]
    fn a_parry_extends_its_window_to_deflect_blows_close_behind() {
        let t = Tuning::default();
        let first = 9; // 패링을 누르고 9틱(0.15초) 뒤 첫 공격 — 창(0.18초) 끝 무렵
        for (gap, deflected) in [(9_usize, true), (18, false)] {
            let mut w = World::new("test", 20.0);
            w.player.facing = std::f32::consts::PI; // -z = 적 쪽
            for (x, at) in [(-0.5_f32, first), (0.5, first + gap)] {
                w.spawn_mob(vec3(x, 0.0, -1.2));
                let en = w.enemies.last_mut().unwrap();
                en.facing = 0.0;
                en.atk = Atk::Smash;
                // `at`틱 뒤에 내리친다(예고 끝 → 다음 틱 휘두름의 첫 틱에 맞음).
                en.state = EState::Tele {
                    t: en.tele_dur() - (at as f32 - 1.0) * TICK_DT,
                };
            }
            let mut parries = 0;
            let mut ev = w.step(
                &Actions {
                    parry: true,
                    ..Default::default()
                },
                &t,
            );
            for _ in 0..120 {
                parries += ev
                    .iter()
                    .filter(|e| matches!(e, Event::Parry { .. }))
                    .count();
                ev = w.step(&Actions::default(), &t);
            }
            parries += ev
                .iter()
                .filter(|e| matches!(e, Event::Parry { .. }))
                .count();
            if deflected {
                assert_eq!(parries, 2, "gap {gap}: both deflected");
                assert_eq!(w.player.hp, PLAYER_HP);
            } else {
                assert_eq!(parries, 1, "gap {gap}: only the first");
                assert!(w.player.hp < PLAYER_HP, "the late blow lands");
            }
        }
    }

    /// 0089(사장 "모으기 공격에는 패링도 달려있어야 함. 리스크가 있는 공격이라 보상"): 모은 강타를 떼는 순간 앞에서 오는 공격은 튕기고
    /// 강타는 끊기지 않는다. 떼지 않고 서 있으면 같은 공격에 맞는다.
    #[test]
    fn releasing_a_charged_heavy_deflects_a_blow_from_the_front() {
        let t = Tuning::default();
        for release in [true, false] {
            let mut w = World::new("test", 20.0);
            w.set_weapon(weapons::Id::Sword);
            w.player.facing = std::f32::consts::PI;
            w.spawn_mob(vec3(0.0, 0.0, -1.2));
            let en = w.enemies.last_mut().unwrap();
            en.facing = 0.0;
            en.atk = Atk::Smash;
            // 두 틱 뒤 내리친다 — 강타의 활성(선딜 0.06초) 전이라 강타가 먼저 끊지 않는다.
            en.state = EState::Tele {
                t: en.tele_dur() - TICK_DT,
            };
            w.player.state = if release {
                PState::Charging { t: 2.0 }
            } else {
                PState::Idle
            };
            let mut parried = false;
            for _ in 0..12 {
                let ev = w.step(&Actions::default(), &t);
                parried |= ev.iter().any(|e| matches!(e, Event::Parry { .. }));
                if parried {
                    assert!(
                        matches!(w.player.state, PState::Heavy { .. }),
                        "the heavy keeps going"
                    );
                }
            }
            assert_eq!(parried, release, "release {release}");
            assert_eq!(w.player.hp == PLAYER_HP, release, "release {release}");
        }
    }

    /// 0089: 모으기 공격의 패링 창은 뗀 뒤 `parry.window s` 동안만, 근접 강타·쇠뇌 모은 사격 모두. 반격(모으지 않은 강타)·근접 무기의 모은 마법엔 없다.
    #[test]
    fn the_charged_release_guard_lasts_one_parry_window_and_only_for_charged_attacks() {
        let t = Tuning::default();
        let open_ticks = |w: &mut World| {
            let mut n = 0;
            for _ in 0..40 {
                w.step(&Actions::default(), &t);
                n += w.parry_open(&t) as usize;
            }
            n
        };
        let want = (t[P::ParryWindow] / TICK_DT).round() as usize;
        let mut w = World::new("test", 20.0);
        w.set_weapon(weapons::Id::Hammer);
        w.player.state = PState::Charging { t: 2.0 };
        let n = open_ticks(&mut w);
        assert!(
            n + 1 >= want && n <= want,
            "heavy guard {n} ticks, want ≈{want}"
        );
        // 쇠뇌: 모은 사격을 떼면.
        let mut w = World::new("test", 20.0);
        w.set_weapon(weapons::Id::Crossbow);
        w.player.state = PState::CastCharge { t: 2.0 };
        let n = open_ticks(&mut w);
        assert!(n + 1 >= want && n <= want, "shot guard {n}");
        // 검의 RB 마법을 모아 뗀 것 — 무기의 공격이 아니라 없다.
        let mut w = World::new("test", 20.0);
        w.set_weapon(weapons::Id::Sword);
        w.player.state = PState::CastCharge { t: 2.0 };
        assert_eq!(open_ticks(&mut w), 0, "charged magic has no guard");
        // 반격(튕긴 뒤 바로 강타 — 모으지 않았다)도 없다.
        let mut w = World::new("test", 20.0);
        w.set_weapon(weapons::Id::Sword);
        w.player.since_parry = 0.0;
        w.step(
            &Actions {
                attack: true,
                ..Default::default()
            },
            &t,
        );
        assert!(matches!(w.player.state, PState::Heavy { full: true, .. }));
        assert_eq!(open_ticks(&mut w), 0, "a riposte is not a charged attack");
    }

    /// 0088(사장 2026-11-07 "패링 성공시 어느정도 기력 회복이 되어야 반격 바로 할수 있을듯"): 기력 마지막 한 줌으로 튕겨도
    /// 튕긴 순간 `parry.stamina gain`이 차서 반격(다 모은 강타)이 바로 나간다. 0으로 두면 반격이 막힌다.
    #[test]
    fn a_parry_on_the_last_breath_still_leaves_stamina_for_the_riposte() {
        for (gain, riposte) in [(None, true), (Some(0.0), false)] {
            let mut t = Tuning::default();
            if let Some(g) = gain {
                t.bend(P::ParryStaminaGain, g);
            }
            let mut w = World::new("test", 20.0);
            w.player.facing = std::f32::consts::PI;
            w.spawn_mob(vec3(0.0, 0.0, -1.2));
            let en = w.enemies.last_mut().unwrap();
            en.facing = 0.0;
            en.atk = Atk::Smash;
            en.state = EState::Tele {
                t: en.tele_dur() - 4.0 * TICK_DT,
            };
            w.player.stamina = 1.0;
            let mut ev = w.step(
                &Actions {
                    parry: true,
                    ..Default::default()
                },
                &t,
            );
            assert_eq!(w.player.stamina, 0.0, "the parry spent the last of it");
            let mut parried = false;
            for _ in 0..30 {
                if ev.iter().any(|e| matches!(e, Event::Parry { .. })) {
                    parried = true;
                    break;
                }
                ev = w.step(&Actions::default(), &t);
            }
            assert!(parried, "gain {gain:?}");
            assert!((w.player.stamina - t[P::ParryStaminaGain]).abs() < 1e-3);
            // 튕긴 멈춤(히트스톱) 동안 누른 것도 받아 둔다 — 몇 틱 안에 반격이 나간다.
            let mut ev = w.step(
                &Actions {
                    attack: true,
                    ..Default::default()
                },
                &t,
            );
            let mut heavy = false;
            let mut denied = false;
            for _ in 0..12 {
                denied |= ev.contains(&Event::StaminaDenied);
                heavy |= matches!(w.player.state, PState::Heavy { full: true, .. });
                ev = w.step(&Actions::default(), &t);
            }
            assert_eq!(heavy, riposte, "gain {gain:?}: riposte {heavy}");
            assert_eq!(denied, !riposte, "a blocked riposte says so");
        }
    }

    /// 0088(사장 "기력이 없어 행동하지 못할때의 피드백이 부족함"): 막히면 `Event::StaminaDenied`(소리)·막대 알림이 한 번 —
    /// 받아 둔 누름이 틱마다 다시 막혀도 다시 알리지 않고, 알림이 끝난 뒤 새로 누르면 다시.
    #[test]
    fn running_out_of_stamina_says_so_once_per_press() {
        let t = Tuning::default();
        let mut w = World::new("test", 20.0);
        w.player.stamina = 0.0;
        w.player.stamina_idle = 0.0;
        let press = Actions {
            attack: true,
            ..Default::default()
        };
        let count = |ev: &[Event]| ev.iter().filter(|e| **e == Event::StaminaDenied).count();
        let mut n = count(&w.step(&press, &t));
        assert_eq!(n, 1, "the blocked press is announced");
        assert!(w.player.stamina_denied > 0.0 && matches!(w.player.state, PState::Idle));
        for _ in 0..((STAMINA_DENIED_SHOW / TICK_DT) as usize - 2) {
            n += count(&w.step(&Actions::default(), &t));
        }
        assert_eq!(n, 1, "the buffered retry stays quiet");
        for _ in 0..4 {
            w.step(&Actions::default(), &t);
        }
        w.player.stamina = 0.0;
        assert_eq!(
            count(&w.step(&press, &t)),
            1,
            "a new press later is announced again"
        );
    }

    /// 0085(사장 2026-11-07 "화면돌리기의 휘청은 cc 포함"): 돌기 시작하는 순간 범위 안의 적이 짧게 휘청이고(예고가 끊긴다),
    /// 범위 밖은 그대로. 보스는 무게만큼 짧게.
    #[test]
    fn turning_staggers_enemies_in_range_briefly_when_it_starts() {
        let t = Tuning::default();
        let mut w = World::new("test", 30.0);
        w.spawn_mob(vec3(0.0, 0.0, -2.0));
        w.spawn_mob(vec3(0.0, 0.0, t[P::SpillRange] + 6.0));
        w.spawn_boss(vec3(2.5, 0.0, 0.0));
        for en in &mut w.enemies {
            en.state = EState::Tele { t: 0.1 };
        }
        w.step(&turn(true), &t);
        let dur = |e: &Enemy| match e.state {
            EState::Stagger { dur, .. } => Some(dur),
            _ => None,
        };
        assert_eq!(
            dur(&w.enemies[0]),
            Some(t[P::TurnStartStagger]),
            "near: staggered, telegraph cut"
        );
        assert_eq!(dur(&w.enemies[1]), None, "out of range: untouched");
        let boss = dur(&w.enemies[2]).expect("the boss too");
        assert!(boss < t[P::TurnStartStagger], "but shorter: {boss}");
        // 짧다 — 땅이 멈추기(쏟기) 전에 풀린다.
        for _ in 0..((t[P::TurnStartStagger] / TICK_DT) as usize + 2) {
            w.step(&Actions::default(), &t);
        }
        assert_eq!(dur(&w.enemies[0]), None);
        assert!(w.spill_at.is_some(), "still turning");
    }

    /// 0086(사장 2026-11-07): 이어 패링이 쌓이면 남은 창이 반씩 줄어드는 몫만큼 늘고(0.2 → +0.1 → +0.05), 받는 반원은 넓어진다
    /// (앞 ±78° → ±123° → 둘레 전부).
    #[test]
    fn parry_chains_stack_halving_time_and_widen_their_arc() {
        let (l0, n0) = parry_chain_add(0.0, 0);
        assert_eq!((l0, n0), (PARRY_CHAIN, 0), "first parry opens the chain");
        let (l1, n1) = parry_chain_add(0.05, n0);
        assert!((l1 - (0.05 + PARRY_CHAIN * 0.5)).abs() < 1e-6 && n1 == 1);
        let (l2, n2) = parry_chain_add(l1, n1);
        assert!((l2 - (l1 + PARRY_CHAIN * 0.25)).abs() < 1e-6 && n2 == 2);
        assert_eq!(
            parry_front_cos(false, 3),
            PARRY_FRONT,
            "no chain: the front half"
        );
        assert_eq!(parry_front_cos(true, 0), PARRY_FRONT);
        let side = parry_front_cos(true, 1);
        assert!(side < 0.0 && side > -0.6, "±123°: the sides too ({side})");
        assert!(parry_front_cos(true, 3) < -1.0, "all around");
    }

    /// 0086: 앞에서 둘을 함께 튕기면(이음 +1) 0.1초 뒤 옆(90°)에서 오는 것도 튕긴다 — 하나만 튕겼으면 옆은 맞는다.
    #[test]
    fn a_stacked_parry_chain_catches_a_blow_from_the_side() {
        let t = Tuning::default();
        for (front_mobs, side_deflected) in [(2usize, true), (1, false)] {
            let mut w = World::new("test", 20.0);
            w.player.facing = std::f32::consts::PI; // -z = 앞
            let mut put = |w: &mut World, at: Vec3, face: f32, ticks: usize| {
                w.spawn_mob(at);
                let en = w.enemies.last_mut().unwrap();
                en.facing = face;
                en.atk = Atk::Smash;
                en.state = EState::Tele {
                    t: en.tele_dur() - (ticks as f32 - 1.0) * TICK_DT,
                };
            };
            // 패링을 누르면 가장 가까운 적 쪽으로 돌아선다 — 앞 오른쪽(0.3, -1.0)이 가장 가깝게.
            let fronts = [vec3(0.3, 0.0, -1.0), vec3(-0.35, 0.0, -1.05)];
            for at in fronts.iter().take(front_mobs) {
                put(&mut w, *at, 0.0, 3);
            }
            // 왼쪽 옆(바라보는 쪽에서 ≈ 107° — 평소 반원 ±78° 밖, 한 번 쌓인 ±123° 안)에서 12틱 뒤:
            // 패링 동작 창(0.18초 = 11틱)이 닫힌 뒤라 이어 패링으로만 받는다.
            put(
                &mut w,
                vec3(-1.2, 0.0, 0.0),
                std::f32::consts::FRAC_PI_2,
                3 + 12,
            );
            let mut parries = 0;
            let mut ev = w.step(
                &Actions {
                    parry: true,
                    ..Default::default()
                },
                &t,
            );
            for _ in 0..90 {
                parries += ev
                    .iter()
                    .filter(|e| matches!(e, Event::Parry { .. }))
                    .count();
                ev = w.step(&Actions::default(), &t);
            }
            parries += ev
                .iter()
                .filter(|e| matches!(e, Event::Parry { .. }))
                .count();
            let side_hit = w.player.hp < PLAYER_HP;
            assert_eq!(
                !side_hit, side_deflected,
                "{front_mobs} in front: parries {parries}, hp {}",
                w.player.hp
            );
        }
    }
}
