//! 게임 규칙 하나. 카메라도 소리도 메뉴도 모른다 — `Actions`와 `Basis`(입력 기준)만 받고 `Event`를 내놓는다.
//! 틱은 단계 메서드 순서(nail 0169 R1): tick_lock → tick_player → apply_strike → apply_turn → tick_enemies → collide → cleanup.
use macroquad::prelude::*;

use crate::ground::Ground;
use crate::input::Actions;
use crate::progression::{cell, Cell};
use crate::skills;
use crate::tuning::{
    Tuning, AIR_SHOT_HOVER, AIR_SHOT_MIN, AIR_SHOT_REACH, AIR_VOLLEY, AIR_VOLLEY_SPREAD_DEG,
    BEAT_CUE_MIN, BEAT_CUE_SHOT_ARC, BEAT_CUE_SHOT_REACH, BEAT_WINDOW, BEAT_WINDOW_MIN, BLAST_SHOW,
    BOLT_HEIGHT, BOLT_RADIUS, BOMB_BOUNCE, BOMB_GRAVITY, BOMB_SHOT_REACH, BOMB_SKID, CAST_CHARGES,
    DASH_ATTACK_LATE, ELITE_EMBERS, ELITE_HARD_POISE, ELITE_LINGER_DELAY, ELITE_LINGER_RADIUS,
    FLASK_CHARGES, GROUND_STEP, HEAVY_SWAY, HIT_DY, HIT_HITSTOP, HIT_SWAY, HP_UNIT, LOCK_HANDOFF,
    P, PERFECT_RELEASE, PLAYER_HP, PLAYER_RADIUS, RUN_SPEED_X, RUN_STAMINA, SHARD_R, SHARD_RANGE,
    SHARD_REFLECT_DAMAGE, SHARD_REFLECT_SPEED, SHARD_SPEED, STAMINA_DENIED_SHOW, STUN_CHAIN_GRACE,
    STUN_CHAIN_MAX, TICK_DT,
};
use crate::tuning::{ALERT_RADIUS, HEAR_RANGE, LEASH_RANGE, SIGHT_DY, SIGHT_RANGE};
use crate::tuning::{PARRY_CHAIN, PARRY_CHAIN_DECAY, PARRY_CHAIN_WIDEN_DEG, RIPOSTE_WINDOW};
use crate::weapons::{self, Rb, Shape};

/// 날아가는 것·늦게 터지는 것의 틱 — 단계마다 메서드(0159 R2).
mod bolt_tick;
/// 적 틱 — 단계마다 메서드(0159 R2).
mod enemy_tick;
/// 플레이어 틱 — 단계·상태마다 메서드(0103).
mod player_tick;
pub mod skill_tick;
/// 플레이어 스윙의 판정 — 단계마다 메서드(0159 R2).
mod strike;

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
    /// 등딱지가 평타·탄을 튕겼다(0120).
    Deflect {
        at: Vec3,
    },
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
    /// 지연 공격이 곧 풀린다(0128 — 정점에서 버티던 적이 내리치기 직전, 앱이 소리를 낸다).
    Release {
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
    /// 스킬(0151)을 눌러 시동이 걸렸다 — 앱이 휘두르는 소리를 낸다.
    SkillStart {
        id: skills::Id,
    },
    /// 스킬의 판정이 나간 순간(휘돌이는 바퀴마다, 가로지르기는 내딛는 순간) — 이펙트의 자리와 쪽.
    Skill {
        id: skills::Id,
        at: Vec3,
        dir: Vec3,
    },
    /// 스킬을 눌렀지만 아직 대기 중 — HUD 칸이 깜빡인다.
    SkillDenied,
    /// 큰 기술(0151)의 시동(세계가 멈춘다)과 땅이 뒤집힌 순간.
    UltStart,
    Ult {
        at: Vec3,
    },
    /// 큰 기술 게이지가 가득 찼다.
    UltReady,
    /// 끌어모으기의 "모인 자리"(0158 G1)가 터졌다.
    SkillBurst {
        at: Vec3,
    },
    /// 버팀(0158 G2)이 한 대를 받아냈다 — 받아낸 수(`n`번째).
    Soak {
        at: Vec3,
        n: u8,
    },
    /// 되울림(0158 G2): 패링이 둘레 `radius` m를 울렸다.
    EchoRing {
        at: Vec3,
        radius: f32,
    },
    /// 던지기(0158 G2): 든 것이 끝에서 되돌았다 / 끝에서 터졌다(박힌 자리) / 손으로 돌아왔다.
    ThrowTurn {
        at: Vec3,
    },
    ThrowBoom {
        at: Vec3,
        radius: f32,
    },
    ThrowBack {
        at: Vec3,
    },
    /// 잔불 흩기(0158 G2): 불티가 그 안의 적을 한 번 태웠다(`every`초마다).
    FieldPulse {
        at: Vec3,
        radius: f32,
    },
    /// 큰 기술 몰아치기(0158 G2)가 시작됐다(시동이 끝난 순간).
    Flurry {
        at: Vec3,
    },
    /// 큰 기술 한 줄기(0158 G2)가 앞 `len` m(반 폭 `half` m)를 갈랐다.
    LineCut {
        at: Vec3,
        dir: Vec3,
        len: f32,
        half: f32,
    },
}

/// 무기의 버튼 길(0151, 사장 "원거리는 공격 버튼이 전부 평타인 거고 RB는 스페셜"): 어느 무기든 X = 평타, RB = 스페셜.
/// 근접 무기는 그대로다(X 콤보 · RB 마법 탄). 원거리 무기는 평타가 사격이고 스페셜이 수류탄이라, 월드 안쪽의 이름
/// (`attack` = 수류탄 던지기, `cast` = 사격 — 0048·0051의 자리)과 맞바꾼다. 쥐고 있음도 같이(X 꾹 = 모은 사격, RB 꾹 = 멀리 던지기).
pub fn weapon_buttons(a: Actions, w: &weapons::Weapon) -> Actions {
    if matches!(w.rb, Rb::Shot { .. }) {
        Actions {
            attack: a.cast,
            attack_held: a.cast_held,
            cast: a.attack,
            cast_held: a.attack_held,
            ..a
        }
    } else {
        a
    }
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

/// 남는 엘리트(0112)가 쓰러진 자리의 고리 — `ELITE_LINGER_DELAY` 동안 예고, 다 차면 둘레를 친다(패링 못 함).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Linger {
    pub pos: Vec3,
    pub t: f32,
}

/// 깨진 종지기의 종 조각(0125): 곧게 날아간다 — 앞에서 패링하면 종지기 쪽으로 되돌아가(`reflected`) 종지기를 깎는다.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shard {
    pub pos: Vec3,
    pub dir: Vec3,
    pub went: f32,
    pub reflected: bool,
    /// 던진 종지기(적 번호).
    pub from: usize,
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
    /// 스킬 칸(0 = 스킬 1, 1 = 스킬 2)과 큰 기술(0151).
    Skill(u8),
    Ult,
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
    /// 등딱지에 튕겼다(0120): 짧게 뒤로 튀고 콤보가 끊긴다(`RECOIL_TIME` 동안 못 움직이고 못 친다). 패링당한 것보다 짧다.
    Recoil {
        t: f32,
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
    /// 스킬(0151, `skills.rs`): 시동 → 판정(모양마다) → 후딜. `dir` = 누른 순간에 정한 쪽, `swing` = 이 스킬의 타 번호
    /// (한 번 맞은 적을 다시 치지 않는다 — 휘돌이는 바퀴마다 새 번호).
    Skill {
        id: skills::Id,
        t: f32,
        dir: Vec3,
        swing: u32,
    },
    /// 큰 기술(0151 → 0158 G2: 끼운 하나 `kind`): 시동 뒤 터지고(`done` — 땅이 뒤집힌다·몰아치기가 시작된다·한 줄을 가른다) 후딜.
    Ult {
        kind: skills::Ult,
        t: f32,
        done: bool,
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
    /// 맞은 뒤(0143): 무적 시간, 맞은 뒤 흐른 시간과 그 타의 세기(그림이 읽는다), 이어 맞음.
    pub hurt: HurtState,
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
    /// 마지막 쏟음(땅 돌리기가 멈춘 순간)에서 지난 초 — 렌더의 쟁반 테.
    pub since_spill: f32,
    /// 끼운 스킬·큰 기술의 상태(0151·0158): 끼운 것과 단, 대기, 게이지, 스킬마다의 진행(버팀·되울림·몰아치기 …).
    pub skill: SkillState,
    /// 패링과 그 뒤(0076~0090·0107): 마지막 패링 뒤의 시간, 이어 패링, 떼기 지킴, 패링이 연 다음 타(강공격·반격·맞받기·맺음타).
    pub parry: ParryState,
    /// 회피가 적 등 뒤에서 끝났나(비껴서기, 0107)와 그 대시 공격의 스윙 번호.
    pub flank: bool,
    pub flank_swing: u32,
    /// 무기 박자(0071·0076): 마지막 평타 뒤의 시간과 그 타의 박자, 박자에 맞았나, 이어 맞춘 수.
    pub beat: BeatState,
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
    /// 마법·사격·수류탄(0048·0051·0111): 마법 칸, 공중 사격과 체공, 쇠뇌의 탄과 재장전, 수류탄.
    pub ranged: RangedState,
    /// 든 무기(0051, `weapons.rs` 표).
    pub weapon: weapons::Id,
    /// 미리 받아 둔 다음 행동과 남은 초(0057 — 후딜레이를 끊는 캔슬 시점에 나간다).
    pub pending: Option<Next>,
    pub pending_left: f32,
    /// 콤보: 다음 타 번호와 그 타를 이어 갈 남은 초.
    pub combo_next: u8,
    pub combo_left: f32,
}

/// 끼운 스킬·큰 기술의 상태(0151·0158): 끼운 것과 단, 대기, 게이지, 스킬마다의 진행(버팀·되울림·몰아치기 …). `Player::skill`.
pub struct SkillState {
    /// 스킬(0151): 끼운 둘, 스킬마다 남은 대기(초, `skills::IDS`의 자리), 큰 기술 게이지(0..1), 게이지를 채운 마지막 휘두르기
    /// (한 번 휘두르기에 한 번만), 대기 중에 눌러 칸이 깜빡이는 남은 초.
    pub worn: [skills::Id; 2],
    pub cd: [f32; skills::N],
    pub gauge: f32,
    pub gauge_swing: u32,
    pub denied: f32,
    /// 기술 판(0158): 끼운 칸의 단·고르기, 큰 기술의 단 — 앱이 `set_skills`로 넣는다(월드는 기록을 모른다).
    pub rank: [u8; 2],
    pub pick: [skills::Pick; 2],
    pub ult_rank: u8,
    /// 끼운 큰 기술(0158 G2 — 하나).
    pub ult: skills::Ult,
    /// 버팀(0158 G2): 이번 틱에 버티는 중이면 타마다 덜어내는 피해(0 = 버티지 않는다 — 스킬 틱이 틱마다 적는다), 받아낸 조각을
    /// 되쏘나(ㄴ), 이번 버팀에서 받아낸 타 수와 그 상한(터뜨림의 덤 — 다음 버팀까지 남아 앱이 읽는다).
    pub brace: i32,
    pub reflect: bool,
    pub brace_soaked: u8,
    pub brace_most: u8,
    /// 되울림(0158 G2): 남은 초와 처음 길이(HUD 칸).
    pub echo_left: f32,
    pub echo_total: f32,
    /// 몰아치기(0158 G2): 남은 초·처음 길이(+ 더해진 몫 — HUD가 줄어드는 게이지로 그린다)·쓰러뜨려 더한 초(상한까지).
    pub flurry_left: f32,
    pub flurry_total: f32,
    pub flurry_added: f32,
    /// 지금 스킬의 덤 상태(0158 G1): 뛰어드는 거리(땅울림 ㄱ — 누른 순간에 정한다), 대기를 이미 돌려받았나(가로지르기 맺음),
    /// 지나는 동안 맞지 않는 중인가(가로지르기 ㄴ — 스킬 틱이 틱마다 적는다).
    pub leap: f32,
    pub refunded: bool,
    pub ghost: bool,
}

/// 패링과 그 뒤(0076~0090·0107): 마지막 패링 뒤의 시간, 이어 패링, 떼기 지킴, 패링이 연 다음 타(강공격·반격·맞받기·맺음타). `Player::parry`.
pub struct ParryState {
    /// 마지막 패링 성공에서 지난 초(렌더의 번쩍임).
    pub since: f32,
    /// 이어 패링(0081 → 0086): 남은 창(초)과 이번 이음에서 더 튕긴 수(0 = 첫 튕김만). 창이 닫히면 0으로.
    pub chain_left: f32,
    pub chain_n: u32,
    /// 모으기 공격의 패링(0089, 사장 "모으기 공격에는 패링도 달려있어야 함. 리스크가 있는 공격이라 보상"): 모은 공격(강타·모은 사격)을
    /// 뗀 뒤 남은 패링 창(초, 시작 = `parry.window s`). 그 공격 동작 안에서만 열린다.
    pub release_guard: f32,
    /// 나무 칸 동작(0107)의 표지: 마지막 타 뒤 박자로 이은 스윙(맺음타), 다음 강타가 다 모여 있다(끝맺음), 반격 스윙(되돌려 치기),
    /// 모으기 패링으로 튕긴 강타 스윙(모은 맞받기).
    pub closing_swing: u32,
    pub closing_ready: bool,
    pub primed_heavy: bool,
    pub riposte_swing: u32,
    pub gathered_swing: u32,
}

/// 무기 박자(0071·0076): 마지막 평타 뒤의 시간과 그 타의 박자, 박자에 맞았나, 이어 맞춘 수. `Player::beat`.
pub struct BeatState {
    /// 박자(2026-11-07): 마지막 평타가 시작된 뒤 흐른 시간, 그 타의 한 스윙 시간(= 그 무기의 박자, 0 = 없음),
    /// 이번 공격 누름이 박자 창 안이었나(누른 순간에 판정 — 받아 둔 이른 누름은 거짓).
    pub since_swing: f32,
    pub period: f32,
    pub on: bool,
    /// 마지막 평타의 타 번호(대시 = `weapons::DASH`) — 박자 부채꼴(0087)이 그 타의 모양·휘두르는 쪽을 끝까지 그린다.
    pub swing_hit: u8,
    /// 박자 맞춤 피드백(0076): 잇달아 박자에 맞춘 타 수(박자를 놓친 타·새 콤보면 0), 마지막 박자 타에서 지난 초,
    /// 지금 스윙이 박자 타인가(렌더가 금빛 고리·음표·금빛 휘두름, HUD가 스태미나 막대 반짝임, 앱이 오르는 종소리).
    pub chain: u32,
    pub since: f32,
    pub swing_beat: bool,
    /// 마지막 딱 맞춰 떼기(0078)에서 지난 초 — 렌더가 발밑에 크게 퍼지는 금빛 고리.
    pub since_perfect: f32,
}

/// 마법·사격·수류탄(0048·0051·0111): 마법 칸, 공중 사격과 체공, 쇠뇌의 탄과 재장전, 수류탄. `Player::ranged`.
pub struct RangedState {
    /// 마법 칸(0048 → 0111, 0..=`CAST_CHARGES`): `cast.cooldown s`마다 한 칸 차고 쏘면 1 준다. 못 쏴서 깜빡이는 남은 초(HUD).
    pub mana: f32,
    pub cast_denied: f32,
    /// 공중 RB(0111): 이번 점프에 이미 쐈나(땅에 서면 지운다), 떨어지기를 멈춘 남은 초(체공).
    pub air_shot: bool,
    pub hover: f32,
    /// RB를 한 번 누른 뒤 쥔 시간(0056, `attack_hold`와 같은 셈).
    pub cast_hold: f32,
    /// 원거리 무기: 탄창의 볼트, 장전 남은 초(0 = 장전 중 아님), 수류탄(소수 = 다시 차는 중).
    pub ammo: u8,
    pub reload: f32,
    pub grenades: f32,
}

/// 맞은 뒤(0143): 무적 시간, 맞은 뒤 흐른 시간과 그 타의 세기(그림이 읽는다), 이어 맞음. `Player::hurt`.
pub struct HurtState {
    pub left: f32,
    /// 마지막으로 맞은 뒤 지난 초·그때 잃은 체력(0143) — HUD가 맞음 테·부서지는 체력 칸을 그린다(로직 시계: 히트스톱 동안 멈춰 있다).
    pub since: f32,
    pub blow: i32,
    /// 경직 이음(0018 → 0099): 이음 안에서 또 맞은 횟수(0 = 첫 경직) — 경직 길이가 반씩 준다 — 와 남은 이음 창(초,
    /// 경직 + `STUN_CHAIN_GRACE`, 맞을 때마다 새로). 창이 닫히면 다음 경직은 다시 처음 길이.
    pub stun_chain: i32,
    pub stun_window: f32,
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
        // 큰 기술(0151)은 땅이 뒤집히기까지 끊기지 않는다 — 가득 모은 게이지가 한 대에 날아가지 않게.
        // 가로지르기 ㄴ(0158): 지나는 동안.
        matches!(
            self.state,
            PState::Dodge { .. } | PState::Ult { done: false, .. }
        ) || (self.skill.ghost && matches!(self.state, PState::Skill { .. }))
            || self.hurt.left > 0.0
    }
}

pub use crate::enemy::{Affix, Atk, EState, Enemy, TeleShape};
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
    /// 제자리로 가는 거리 지도(0113) — 제자리 칸마다 한 번 만들어 둔다(제자리는 움직이지 않는다).
    home_flows: std::collections::HashMap<(usize, usize), Vec<u16>>,
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
    /// 이어지는 박자(0107): 이번 틱에 박자 셋째 타가 나갔다 — 둘레 충격.
    shock_now: bool,
    /// 착지의 순간(0107): 늦은 바깥 고리까지 남은 초.
    late_ring: Option<f32>,
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
    /// 남는 엘리트의 고리(0112).
    pub lingers: Vec<Linger>,
    /// 날아가는 종 조각(0125).
    pub shards: Vec<Shard>,
    /// 방랑자만 밀어내는 둥근 몸(0122 — 주민): 적·탄·공격은 지나간다(싸움에 끼지 않는다). (가운데, 반지름).
    pub bodies: Vec<(Vec3, f32)>,
    /// 합친 능력치(0064 M3 — 장신구·성장). 앱이 `apply_stats`로 넣는다.
    pub stats: crate::stats::Stats,
    /// 연 기술 나무(0106) — 칸 동작은 그 동사 자리에서 `tree.has`(0107). 앱이 기록에서 넣는다.
    pub tree: crate::progression::Tree,
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
    /// 공중 RB(0111): 이번 틱에 공중에서 쏜 쪽(수평) — `tick_bolts`가 내려 쏘는 탄을 만든다.
    air_fire: Option<Vec3>,
    cast_denied_now: bool,
    /// 스킬(0151): 플레이어 틱이 낸 사건(틱 끝에 내보낸다), 이번 틱에 스킬의 타가 맞았나·스킬이 남긴 처치로 치는 남은 초
    /// (기세의 몫을 가른다 — 스킬로 쓰러뜨리면 덜 당겨진다).
    skill_ev: Vec<Event>,
    skill_hit_tick: bool,
    skill_kill_left: f32,
    /// 스킬이 남긴 것(0158 G1): 곧 터질 "모인 자리"(끌어모으기 ㄱ), 남은 갈라진 땅(땅울림 ㄴ — 렌더가 읽는다).
    skill_bursts: Vec<skill_tick::Burst>,
    pub skill_zones: Vec<skill_tick::Zone>,
    /// 0158 G2: 던진 무기(던지기 — 렌더가 그 자리에 무기를 그린다), 남은 불티(잔불 흩기), 버티는 동안 받아낸 조각을 던진 적(버팀 ㄴ).
    pub skill_blades: Vec<skill_tick::Blade>,
    pub skill_fields: Vec<skill_tick::Field>,
    brace_caught: Vec<usize>,
    /// 이번 틱에 방랑자의 탄이 적을 맞혔다(게이지 — `skill_books`).
    bolt_hit_tick: bool,
    /// 이번 틱에 박자에 맞춘 평타가 나갔다(`Event::Beat`).
    beat_now: bool,
    /// 이번 틱에 등잔을 삼켰다·못 마셨다(사건은 `apply_strike`가 낸다).
    gulp_now: bool,
    flask_denied_now: bool,
    /// 히트스톱 동안 들어온 눌림(공격·회피·락온·돌리기). 멈춤이 풀리는 틱에 낸다 — 버리면 멈춘 순간 누른 버튼이 사라진다.
    held: Actions,
    /// 이 틱에 공격을 쥐고 있나(0120 — 등딱지 튕김이 모으기를 끊지 않게).
    attack_down: bool,
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
                hurt: HurtState {
                    left: 0.0,
                    stun_chain: 0,
                    stun_window: 0.0,
                    since: f32::MAX,
                    blow: 0,
                },
                dodge_cd: 0.0,
                turn_cd: 0.0,
                stamina: crate::tuning::SPECS[crate::tuning::idx(P::StaminaMax)].def,
                max_hp: PLAYER_HP,
                flask_max: FLASK_CHARGES,
                stamina_idle: 0.0,
                stamina_denied: 0.0,
                denied_new: false,
                since_spill: f32::MAX,
                skill: SkillState {
                    worn: skills::START,
                    cd: [0.0; skills::N],
                    gauge: 0.0,
                    gauge_swing: 0,
                    denied: 0.0,
                    rank: [1; 2],
                    pick: [skills::Pick::A; 2],
                    ult_rank: 1,
                    ult: skills::Ult::Upend,
                    brace: 0,
                    reflect: false,
                    brace_soaked: 0,
                    brace_most: 0,
                    echo_left: 0.0,
                    echo_total: 0.0,
                    flurry_left: 0.0,
                    flurry_total: 0.0,
                    flurry_added: 0.0,
                    leap: 0.0,
                    refunded: false,
                    ghost: false,
                },
                parry: ParryState {
                    since: f32::MAX,
                    chain_left: 0.0,
                    chain_n: 0,
                    release_guard: 0.0,
                    closing_swing: 0,
                    closing_ready: false,
                    primed_heavy: false,
                    riposte_swing: 0,
                    gathered_swing: 0,
                },
                flank: false,
                flank_swing: 0,
                dash_left: 0.0,
                connected: 0,
                beat: BeatState {
                    since_swing: f32::MAX,
                    swing_hit: 0,
                    period: 0.0,
                    on: false,
                    chain: 0,
                    since: f32::MAX,
                    swing_beat: false,
                    since_perfect: f32::MAX,
                },
                running: false,
                attack_hold: 0.0,
                flask: FLASK_CHARGES,
                flask_lock: 0.0,
                flask_denied: 0.0,
                air: 0.0,
                vy: 0.0,
                swings: 0,
                ranged: RangedState {
                    mana: CAST_CHARGES,
                    cast_denied: 0.0,
                    air_shot: false,
                    hover: 0.0,
                    cast_hold: 0.0,
                    ammo: 0,
                    reload: 0.0,
                    grenades: 0.0,
                },
                weapon: weapons::Id::Sword,
                pending: None,
                pending_left: 0.0,
                combo_next: 0,
                combo_left: 0.0,
            },
            enemies: Vec::new(),
            lock: None,
            obstacles: Vec::new(),
            ground: None,
            nav: None,
            flow: None,
            home_flows: std::collections::HashMap::new(),
            arena,
            hitstop: 0.0,
            skill_ev: Vec::new(),
            skill_hit_tick: false,
            skill_kill_left: 0.0,
            skill_bursts: Vec::new(),
            skill_zones: Vec::new(),
            skill_blades: Vec::new(),
            skill_fields: Vec::new(),
            brace_caught: Vec::new(),
            bolt_hit_tick: false,
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
            attack_down: false,
            plunge_now: false,
            shock_now: false,
            late_ring: None,
            charged_now: false,
            interact: false,
            interact_here: false,
            bolts: Vec::new(),
            embers: 0,
            drops: Vec::new(),
            lingers: Vec::new(),
            shards: Vec::new(),
            bodies: Vec::new(),
            stats: crate::stats::Stats::default(),
            tree: crate::progression::Tree::default(),
            ember_at: f32::MIN,
            cracks: Vec::new(),
            bombs: Vec::new(),
            blasts: Vec::new(),
            sparks: Vec::new(),
            throw_to: None,
            cast_fire: None,
            air_fire: None,
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
            .any(|e| e.lockable() && e.pos.distance(self.player.pos) <= self.lock_range)
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
        } else if let Some(n) = self.skill_press(a, t) {
            Some(n)
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
        // 스킬의 판정이 끝났나(그 단의 실제 길이로 — 바퀴가 늘거나 길을 되돌아가면 길어진다, 0158).
        // 버팀(0158 G2)은 버티는 동안 회피를 직접 받는다(터뜨림 없이 끊는다).
        let (skill_done, brace_hold) = match self.player.state {
            PState::Skill { id, t: e, .. } => {
                let d = self.skill_eff(id);
                let end = d.skill.startup + d.active();
                let hold = matches!(d.skill.form, skills::Form::Brace { .. });
                (e >= end, hold && e >= d.skill.startup && e < end)
            }
            _ => (false, false),
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
            pl.beat.on = on_beat(pl.beat.since_swing, pl.beat.period);
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
        // 경직 중의 패링 누름은 받아 두지 않는다(0110, 사장 2026-11-08): 풀린 뒤 늦게 나가면 창은 엇나가고
        // 헛침 굳음(`parry.time s`)만 남는다. 패링은 누른 그 순간의 행동 — 회피 등은 그대로 받아 둔다(경직에서 빠져나가기, 0099).
        if matches!(pl.state, PState::Hurt { .. }) && pl.pending == Some(Next::Parry) {
            pl.pending = None;
        }
        let mut out = Actions {
            attack: false,
            dodge: false,
            parry: false,
            jump: false,
            cast: false,
            flask: false,
            skill1: false,
            skill2: false,
            ult: false,
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
            Next::Skill(0) => out.skill1 = true,
            Next::Skill(_) => out.skill2 = true,
            Next::Ult => out.ult = true,
        };
        // 지금 동작이 직접 받는 누름.
        let own = match pl.state {
            PState::Idle => true,
            PState::Charging { .. } => matches!(n, Next::Dodge | Next::Parry),
            PState::CastCharge { .. } => n == Next::Dodge,
            PState::Drink { gulped: false, .. } => n == Next::Dodge,
            // 공중: 공격 = 내려찍기(0027), RB = 내려 쏘기(0111).
            PState::Jump { plunge: false, .. } => matches!(n, Next::Attack | Next::Cast),
            // 스킬의 시동 안에 들어온 큰 기술(두 버튼 함께, 0151) — 스킬 틱이 바꿔친다.
            PState::Skill { t: e, .. } => {
                (n == Next::Ult && e < t[P::SkillChord]) || (n == Next::Dodge && brace_hold)
            }
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
                    pl.ranged.grenades >= 1.0
                } else {
                    pl.stamina > 0.0
                }
            }
            Next::Cast => match wpn.rb {
                Rb::Magic(_) => pl.ranged.mana >= 1.0,
                Rb::Shot { .. } => pl.ranged.ammo >= 1,
            },
            Next::Flask => pl.flask > 0 && pl.hp < pl.max_hp && pl.flask_lock <= 0.0,
            Next::Skill(slot) => pl.skill.cd[pl.skill.worn[slot as usize % 2].index()] <= 0.0,
            Next::Ult => pl.skill.gauge >= 1.0,
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
            // 스킬·큰 기술의 후딜(0151): 판정이 끝나면 회피·다른 스킬·평타로 끊는다.
            PState::Skill { .. } => skill_done,
            PState::Ult { done, .. } => done,
            _ => false,
        };
        if ready && can {
            if let PState::Attack { hit, .. } = pl.state {
                // 다음 타로 잇는다(마지막 타 뒤면 처음부터). 맺음타(0107·0108)는 마지막 타 뒤에도 콤보를 기억한다 —
                // 박자에 누른 마지막 타 뒤의 누름은 대개 이 캔슬 길로 나간다(끝까지 기다리지 않는다).
                let after = wpn.after(hit);
                let closing = after.is_none() && self.tree.has(Cell::ClosingBlow);
                pl.combo_next = after.unwrap_or(0);
                pl.parry.closing_ready = closing;
                pl.combo_left = if after.is_some() || closing {
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
        pl.beat.period = 0.0;
        pl.ranged.reload = 0.0;
        let w = id.def();
        let mag = self.mag_of(id);
        let pl = &mut self.player;
        pl.ranged.ammo = mag;
        pl.ranged.grenades = w.grenade.map_or(0.0, |g| g.count as f32);
        if !matches!(pl.state, PState::Hurt { .. } | PState::Jump { .. }) {
            pl.state = PState::Idle;
        }
    }

    /// 그 무기의 탄창(0180 — 표의 탄창 + 벼린 단의 몫 `Stats::mag`, 사격 무기가 아니면 0).
    pub fn mag_of(&self, id: weapons::Id) -> u8 {
        match id.def().rb {
            Rb::Shot { mag, .. } => (mag as i32 + self.stats.mag).clamp(1, u8::MAX as i32) as u8,
            Rb::Magic(_) => 0,
        }
    }

    /// 종류 표(`species.rs`)의 적 하나를 세운다.
    /// 능력치를 바꾼다(0064 M3 — 장신구를 갈아 끼우거나 성장했을 때). 체력·등잔은 새 최대를 넘지 않게, `refill`이면 가득.
    pub fn apply_stats(&mut self, s: crate::stats::Stats, refill: bool) {
        // 탄창(0180): 가득이었거나 `refill`이면 새 탄창으로 가득, 아니면 새 탄창 안으로.
        let was_full = self.player.ranged.ammo >= self.mag_of(self.player.weapon);
        self.stats = s;
        let mag = self.mag_of(self.player.weapon);
        let pl = &mut self.player;
        pl.ranged.ammo = if refill || was_full {
            mag
        } else {
            pl.ranged.ammo.min(mag)
        };
        pl.max_hp = s.max_hp;
        pl.flask_max = s.flask;
        if refill {
            pl.hp = s.max_hp;
            pl.flask = s.flask;
            pl.stamina = s.max_stamina();
        } else {
            pl.hp = pl.hp.min(s.max_hp);
            pl.flask = pl.flask.min(s.flask);
            pl.stamina = pl.stamina.min(s.max_stamina());
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

    /// 점 `p` 자리의 땅 높이(땅이 없는 개발 아레나는 0) — 그림 쪽(이펙트·바닥 표시)이 묻는다.
    pub fn ground_y(&self, p: Vec3) -> f32 {
        self.ground.as_ref().map_or(0.0, |g| g.height(p))
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
            en.hp -= hurt_by(
                en,
                self.stats.dmg + t[P::PlungeDamage] as i32,
                crit,
                self.stats.opened_dmg,
                t,
            );
            en.exposed = 0.0;
            en.hit_by = swing;
            add_poise(en, 2.0 * self.stats.poise, None, t, ev);
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
            // 맞힌 내려찍기도 휘두르기 한 번(게이지 — `skill_books`).
            self.player.connected = swing;
            self.hitstop = self.hitstop.max(t[P::Hitstop] * 1.5);
        }
    }

    /// 패링 창이 열려 있나(0025).
    /// 패링 창이 열려 있나: 패링 동작의 창 안이거나, 막 튕긴 뒤 이어 패링 시간 안(0081).
    pub fn parry_open(&self, t: &Tuning) -> bool {
        let pl = &self.player;
        matches!(pl.state, PState::Parry { t: e } if e < t[P::ParryWindow])
            || pl.parry.chain_left > 0.0
            || (pl.parry.release_guard > 0.0
                && matches!(pl.state, PState::Heavy { .. } | PState::Cast { .. }))
    }

    #[cfg(test)]
    pub fn basis(&self) -> Basis {
        match self.lock_target() {
            Some(p) => Basis::Lock(p),
            None => Basis::Quarter { yaw: 0.0 },
        }
    }

    /// 이 틱의 랩 값: 장신구·성장이 휜 것(`Stats::tune`) + 몰아치기(0158 G2) 동안의 평타 빠르기(공격 속도 장신구와 같은 길 —
    /// 평타의 선딜·활성·후딜을 `FLURRY.speed`로 나눈다).
    pub fn tuned(&self, t: &Tuning) -> Tuning {
        let mut o = self.stats.tune(t);
        let rate = self.flurry_rate();
        if rate != 1.0 {
            for p in [P::AttackStartup, P::AttackActive, P::AttackRecovery] {
                o.bend(p, o[p] / rate);
            }
        }
        o
    }

    /// 몰아치기 동안의 평타 빠르기 배(아니면 1) — 렌더가 평타 동작을 같은 배로 돌린다.
    pub fn flurry_rate(&self) -> f32 {
        if self.player.skill.flurry_left > 0.0 {
            skills::FLURRY.speed
        } else {
            1.0
        }
    }

    /// 한 틱. 히트스톱 중엔 로직 시계만 멈춘다.
    pub fn step(&mut self, a: &Actions, t: &Tuning) -> Vec<Event> {
        // 장신구·성장이 휜 랩 값(0064 M3) — 이 틱의 로직은 이것만 읽는다.
        let tuned = self.tuned(t);
        let t = &tuned;
        let mut ev = Vec::new();
        let dt = TICK_DT;
        self.attack_down = a.attack_held;
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
                skill1: self.held.skill1 | a.skill1,
                skill2: self.held.skill2 | a.skill2,
                ult: self.held.ult | a.ult,
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
            skill1: a.skill1 | held.skill1,
            skill2: a.skill2 | held.skill2,
            ult: a.ult | held.ult,
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
        // 무기의 버튼 길(0151): X = 그 무기의 평타, RB = 그 무기의 스페셜.
        let merged = weapon_buttons(merged, self.player.weapon.def());
        let a = &merged;
        self.time += dt;
        self.player.prev = self.player.pos;
        for e in &mut self.enemies {
            e.prev = e.pos;
        }
        self.skill_hit_tick = false;
        self.skill_kill_left = (self.skill_kill_left - dt).max(0.0);
        self.tick_lock(a, &mut ev);
        self.tick_player(a, t, dt);
        self.tick_skill_fields(t, dt);
        ev.append(&mut self.skill_ev);
        self.apply_strike(t, &mut ev);
        self.tick_bolts(t, dt, &mut ev);
        self.apply_turn(t, &mut ev);
        self.tick_enemies(t, dt, &mut ev);
        self.collide(t, &mut ev);
        self.cleanup(&mut ev);
        self.tick_embers(t, dt, &mut ev);
        self.skill_books(t, &mut ev);
        ev
    }

    /// 잔불(0064 M1): 막 쓰러진 적은 제 값을 흩고, 바닥의 조각은 튀고·끌리고·주워진다.
    fn tick_embers(&mut self, t: &Tuning, dt: f32, ev: &mut Vec<Event>) {
        for (i, en) in self.enemies.iter_mut().enumerate() {
            if !en.alive() && !en.dropped {
                en.dropped = true;
                let seed = (self.time * 1000.0) as u32 ^ (i as u32).wrapping_mul(2654435761);
                // 잔불 주머니(0064 M3): 값에 배. 엘리트(0112)는 × `ELITE_EMBERS`.
                let elite = if en.elite.is_some() { ELITE_EMBERS } else { 1 };
                // 구역 등급(0112): 종 값 × 등급 배율(올림) — 잔재 2 → 4, 가시짐승 3 → 5(등급 2).
                let base = (en.def().embers as f32 * crate::tuning::tier(en.tier).embers).ceil();
                // 남는 불씨(0158 G2, 잔불 흩기 ㄴ): 불티 속에서 쓰러졌으면 잔불을 더 남긴다.
                let value = (base * elite as f32 * self.stats.ember).round() as u32 + en.kindled;
                // 남는 엘리트(0112): 쓰러진 자리에 고리가 남는다.
                if en.elite == Some(Affix::Lingering) {
                    self.lingers.push(Linger {
                        pos: en.pos,
                        t: 0.0,
                    });
                }
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
            // 깨어나는 중인 보스도 잡힌다(0149) — 아니면 그 2초 동안 락온이 범위 안의 다른 적으로 간다.
            if !e.lockable() {
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
            .filter(|(_, e)| e.lockable() && e.pos.distance(p) <= self.range_lock())
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
            en.hp -= hurt_by(en, self.stats.dmg + damage, crit, self.stats.opened_dmg, t);
            en.exposed = 0.0;
            en.hurt = 0.15;
            en.knock = d.normalize_or_zero() * t[P::Knockback] * if en.boss { 0.2 } else { 1.4 };
            let hit_at = en.pos + Vec3::Y * en.height * 0.6;
            ev.push(if crit {
                Event::Crit { at: hit_at }
            } else {
                Event::Hit { at: hit_at }
            });
            add_poise(en, poise * self.stats.poise, None, t, ev);
            if en.hp <= 0 {
                en.state = EState::Dead;
                self.kills += 1;
                ev.push(Event::EnemyDown { at: en.pos });
            }
        }
        self.hitstop = self.hitstop.max(t[P::Hitstop]);
    }

    /// 등딱지에 튕겼다(0120): 콤보·받아 둔 누름을 끊고 짧게 뒤로 튄다. 소리·불똥은 앱이 `Event::Deflect`로.
    /// 공격을 쥐고 있으면(강타를 모으려는 평타) 튀지 않는다 — 강타가 등딱지를 뚫는 길이라 첫 평타의 튕김이 모으기를 끊으면 안 된다.
    fn recoil(&mut self, at: Vec3, ev: &mut Vec<Event>) {
        ev.push(Event::Deflect { at });
        if self.attack_down {
            return;
        }
        let pl = &mut self.player;
        let back = vec3(pl.pos.x - at.x, 0.0, pl.pos.z - at.z).normalize_or_zero();
        pl.vel = back * crate::tuning::RECOIL_PUSH;
        pl.state = PState::Recoil { t: 0.0 };
        pl.pending = None;
    }

    /// 맞힌 스윙 뒤의 나무 칸(0107): 되돌려 치기(반격이 둘레도 휘청) · 길을 여는 발(대시 공격이 맞힌 적을 지나쳐 등 뒤에).
    fn strike_cells(
        &mut self,
        t: &Tuning,
        s: &Strike,
        first: Option<(Vec3, f32, Vec3)>,
        ev: &mut Vec<Event>,
    ) {
        let _ = (t, ev);
        if s.swing == self.player.parry.riposte_swing && self.tree.has(Cell::TurnItBack) {
            let p = self.player.pos;
            for en in &mut self.enemies {
                let d = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z);
                if en.active()
                    && en.hit_by != s.swing
                    && d.length() - en.radius <= cell::TURN_BACK_RADIUS
                {
                    en.state = EState::Stagger {
                        t: 0.0,
                        dur: cell::TURN_BACK_STAGGER,
                    };
                    en.sway = d.normalize_or_zero() * 0.6;
                    en.sway_t = 0.0;
                }
            }
        }
        if s.dash && self.tree.has(Cell::PathOpeningStep) {
            if let Some((at, r, dir)) = first {
                let to = at + dir * (r + PLAYER_RADIUS + 0.2);
                self.player.pos = vec3(to.x, self.player.pos.y, to.z);
                self.player.vel = Vec3::ZERO;
            }
        }
    }

    /// 착지의 순간(0107): 내려찍기의 늦은 바깥 고리 — 처음 고리 밖, `LANDING_RANGE`배 안의 적만.
    fn outer_ring(&mut self, t: &Tuning, ev: &mut Vec<Event>) {
        let p = self.player.pos;
        let (inner, outer) = (t[P::PlungeRange], t[P::PlungeRange] * cell::LANDING_RANGE);
        for en in &mut self.enemies {
            if !en.active() || (en.pos.y - p.y).abs() > HIT_DY {
                continue;
            }
            let d = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z).length() - en.radius;
            if d <= inner || d > outer {
                continue;
            }
            en.hp -= hurt_by(
                en,
                self.stats.dmg + t[P::PlungeDamage] as i32,
                false,
                self.stats.opened_dmg,
                t,
            );
            add_poise(en, 2.0 * self.stats.poise, None, t, ev);
            en.hurt = 0.15;
            if en.hp <= 0 {
                en.state = EState::Dead;
                self.kills += 1;
                ev.push(Event::EnemyDown { at: en.pos });
            }
            ev.push(Event::Hit {
                at: en.pos + Vec3::Y * en.height * 0.6,
            });
        }
        ev.push(Event::Plunge { at: p });
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
        let tilt = self.tree.has(Cell::TiltingGround);
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
                // 기우는 땅(0107): 바깥으로 1m 더 — 둘러싼 대형이 흩어진다.
                if tilt {
                    en.knock += d / r * cell::TILT_PUSH * KNOCK_DECAY;
                }
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

    /// 버팀(0158 G2)으로 버티는 중인가 — 스킬 틱이 이번 틱에 적어 둔 것(상태가 스킬일 때만).
    pub fn bracing(&self) -> bool {
        self.player.skill.brace > 0 && matches!(self.player.state, PState::Skill { .. })
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
            // 가로지르기(0151)도 길 위의 적을 뚫고 간다.
            let passing = matches!(
                self.player.state,
                PState::Dodge { .. }
                    | PState::Skill {
                        id: skills::Id::Cut,
                        ..
                    }
            );
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
        // 주민(0122): 방랑자만 밀어낸다.
        for &(c, r) in &self.bodies {
            let d = vec3(self.player.pos.x - c.x, 0.0, self.player.pos.z - c.z);
            let min = r + PLAYER_RADIUS;
            let len = d.length();
            if len < min && len > 1e-4 {
                self.player.pos += d / len * (min - len);
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
            if en.rising() {
                continue;
            }
            // 몸은 부딪힌 쪽에서 되튄다.
            en.sway = -en.sway.normalize_or_zero();
            en.sway_t = 0.0;
            en.hp -= t[P::SlamDamage].round() as i32 + std::mem::take(&mut en.slam_bonus);
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
    /// 박자에 맞춘 평타인가·대시 공격인가(나무 칸, 0107).
    beat: bool,
    dash: bool,
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
            let (left, ripe) = beat_shrink(pl.beat.since_swing, 0.0, pl.beat.period)?;
            return Some(BeatCue {
                reach: BEAT_CUE_SHOT_REACH,
                arc: BEAT_CUE_SHOT_ARC,
                sweep: weapons::Sweep::Center,
                left,
                ripe,
            });
        }
        let live = match pl.state {
            PState::Attack { hit, .. } => {
                hit == pl.beat.swing_hit
                    && (wpn.after(hit).is_some() || self.tree.has(Cell::ClosingBlow))
            }
            PState::Idle | PState::Dodge { .. } | PState::Parry { .. } => pl.combo_left > 0.0,
            _ => false,
        };
        if !live {
            return None;
        }
        let h = wpn.hit(pl.beat.swing_hit)?;
        let (s0, _, _) = hit_times(t, wpn, pl.beat.swing_hit);
        let (left, ripe) = beat_shrink(pl.beat.since_swing, s0, pl.beat.period)?;
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

/// 자세 막대의 크기(0051): 잡몹 = 바탕 + 체력(옛 눈금 — `HP_UNIT`으로 나눈 칸 수) × 배, 보스는 따로.
/// 자세는 눈금을 옮기지 않았다(0158 N0) — 체력이 ×10이 되어도 막대는 그대로.
pub fn poise_max(en: &Enemy, t: &Tuning) -> f32 {
    if en.boss {
        t[P::PoiseBoss]
    } else {
        let hard = if en.elite == Some(Affix::Hardened) {
            ELITE_HARD_POISE
        } else {
            1.0
        };
        // 등급(0112): 체력이 커진 몫에 더해 × 등급 자세 배율(정본의 계산 순서).
        (t[P::PoiseBase] + en.max_hp as f32 / HP_UNIT as f32 * t[P::PoisePerHp])
            * hard
            * crate::tuning::tier(en.tier).poise
    }
}

/// 플레이어가 주는 피해(0055): 드러난 적은 치명타 덤, 자세가 깨진 동안은 × `poise.break damage x`(옛 눈금으로 올림 —
/// `tuning::scale_up`, 0158 N0: 옛 3 × 1.5 → 5가 30 × 1.5 → 50) — 폭딜 창.
/// `opened` = 열린 틈의 타 덤(장신구 "깨진 종 조각", 0127) — 드러났거나(치명타) 자세가 깨진 적에게만.
fn hurt_by(en: &Enemy, base: i32, crit: bool, opened: i32, t: &Tuning) -> i32 {
    // 보스 2단계 전환(0133) 동안은 들어가지 않는다(근접·탄은 `guards`가 튕긴다 — 나머지 넓은 타도 0).
    if en.rising() {
        return 0;
    }
    let open = crit || en.broken > 0.0;
    let d = base + if crit { t[P::ParryCrit] as i32 } else { 0 } + if open { opened } else { 0 };
    let d = if en.broken > 0.0 {
        crate::tuning::scale_up(d, t[P::PoiseBreakDamage])
    } else {
        d
    };
    // 끌어모으기 맺음(0158 G1): 모인 적은 잠깐 더 다친다.
    if en.frail > 0.0 {
        skills::scaled(d, en.frail_mult)
    } else {
        d
    }
}

/// 자세 피해를 준다(0051). 다 차면(또는 `primed` 비율을 넘은 적을 무거운 타가 치면) 깨진다: 휘청 + 드러남(다음 한 대 치명타).
fn add_poise(en: &mut Enemy, amount: f32, primed: Option<f32>, t: &Tuning, ev: &mut Vec<Event>) {
    if amount <= 0.0 || en.poise_lock > 0.0 || !en.alive() || en.rising() {
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
mod tests;
