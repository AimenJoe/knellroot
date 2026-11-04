//! 게임 규칙 하나. 카메라도 소리도 메뉴도 모른다 — `Actions`와 `Basis`(입력 기준)만 받고 `Event`를 내놓는다.
//! 틱은 단계 메서드 순서(nail 0169 R1): tick_lock → tick_player → apply_strike → apply_turn → tick_enemies → collide → cleanup.
use macroquad::prelude::*;

use crate::ground::Ground;
use crate::input::Actions;
use crate::tuning::{
    Tuning, BLAST_SHOW, BOLT_HEIGHT, BOLT_RADIUS, BOMB_BOUNCE, BOMB_GRAVITY, BOMB_SHOT_REACH,
    BOMB_SKID, CAST_CHARGES, FLASK_CHARGES, GROUND_STEP, HIT_DY, P, PLAYER_HP, PLAYER_RADIUS,
    TICK_DT,
};
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
    /// 지팡이 강타를 딱 맞춰 뗐다(0051).
    Perfect {
        at: Vec3,
    },
    /// 수류탄을 던졌다(0051).
    Throw,
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

/// 폭발 고리(그림만 — 렌더가 읽는다).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Blast {
    pub pos: Vec3,
    pub radius: f32,
    pub age: f32,
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
    pub stamina_idle: f32,
    pub stamina_denied: f32,
    /// 경직 중 또 맞은 횟수(0 = 첫 경직) — 경직 길이가 반씩 준다.
    pub stun_chain: i32,
    /// 마지막 쏟음(땅 돌리기가 멈춘 순간)에서 지난 초 — 렌더의 쟁반 테.
    pub since_spill: f32,
    /// 마지막 패링 성공에서 지난 초(렌더의 번쩍임).
    pub since_parry: f32,
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
    /// 콤보: 다음 타 번호와 그 타를 이어 갈 남은 초, 미리 받아 둔 누름의 남은 초.
    pub combo_next: u8,
    pub combo_left: f32,
    pub buffer: f32,
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
    /// 금 간 땅(망치 여진)·날고 있는 수류탄·폭발 고리(0051).
    pub cracks: Vec<Crack>,
    pub bombs: Vec<Bomb>,
    pub blasts: Vec<Blast>,
    /// 이번 틱에 손을 떠난 수류탄(던질 자리).
    throw_to: Option<Vec3>,
    /// 이번 틱에 쏜 쪽 — `tick_bolts`가 탄을 세운다.
    cast_fire: Option<Vec3>,
    cast_denied_now: bool,
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
                stamina_idle: 0.0,
                stamina_denied: 0.0,
                stun_chain: 0,
                since_spill: f32::MAX,
                since_parry: f32::MAX,
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
                combo_next: 0,
                combo_left: 0.0,
                buffer: 0.0,
                ammo: 0,
                reload: 0.0,
                grenades: 0.0,
            },
            enemies: Vec::new(),
            lock: None,
            obstacles: Vec::new(),
            ground: None,
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
            cracks: Vec::new(),
            bombs: Vec::new(),
            blasts: Vec::new(),
            throw_to: None,
            cast_fire: None,
            cast_denied_now: false,
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

    /// 무기를 바꾼다(0051): 콤보는 처음부터, 원거리 무기는 탄창·수류탄 가득.
    pub fn set_weapon(&mut self, id: weapons::Id) {
        let pl = &mut self.player;
        pl.weapon = id;
        pl.combo_next = 0;
        pl.combo_left = 0.0;
        pl.buffer = 0.0;
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
            en.hp -= hurt_by(en, t[P::PlungeDamage] as i32, crit, t);
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
    pub fn parry_open(&self, t: &Tuning) -> bool {
        matches!(self.player.state, PState::Parry { t: e } if e < t[P::ParryWindow])
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
        ev
    }

    fn tick_lock(&mut self, a: &Actions, ev: &mut Vec<Event>) {
        // 대상이 죽었거나 너무 멀면 풀린다(버튼 없이).
        if let Some(i) = self.lock {
            let gone = self.enemies.get(i).is_none_or(|e| !e.alive());
            if gone {
                self.lock = None;
                ev.push(Event::LockOff);
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

    fn range_lock(&self) -> f32 {
        // 틱 안에서 Tuning을 들고 다니지 않으려고 여기만 상수로 읽는다 — 랩 값은 step()에서 받은 t.
        self.lock_range
    }
}

impl World {
    fn tick_player(&mut self, a: &Actions, t: &Tuning, dt: f32) {
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
        let aim = if target.is_none() && a.aim.length() > AIM_STICK {
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
        let cast_dir = if a.cast {
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
            if let Some(d) = aim {
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
        let mut fire: Option<Vec3> = None;
        let mut thrown: Option<Vec3> = None;
        let was_dodging = matches!(self.player.state, PState::Dodge { .. });
        let pl = &mut self.player;
        // 콤보·원거리 무기 시계(0051).
        pl.combo_left = (pl.combo_left - dt).max(0.0);
        pl.buffer = (pl.buffer - dt).max(0.0);
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
        // 스태미나: 마지막으로 쓴 뒤 `stamina.regen delay s`가 지나야 찬다.
        pl.stamina_denied = (pl.stamina_denied - dt).max(0.0);
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
                pl.stamina_denied = 0.4;
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
                } else if a.attack && !a.dodge && spend(pl, t[P::StaminaAttack]) {
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
                    pl.state = PState::Attack {
                        t: 0.0,
                        swing: pl.swings,
                        dir,
                        hit,
                    };
                } else if a.cast && matches!(wpn.rb, Rb::Shot { .. }) {
                    // 원거리 무기의 RB = 사격(0051): 탄창이 비면 장전이 끝날 때까지 못 쏜다.
                    if let Rb::Shot { reload, .. } = wpn.rb {
                        if pl.ammo > 0 {
                            pl.ammo -= 1;
                            if pl.ammo == 0 {
                                pl.reload = reload;
                            }
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
                    if pl.flask > 0 && pl.hp < PLAYER_HP {
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
                        pl.hp = (pl.hp + t[P::FlaskHeal].round() as i32).min(PLAYER_HP);
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
                if a.attack {
                    pl.buffer = t[P::AttackBuffer];
                }
                let last = hit as usize + 1 >= wpn.combo.len();
                // 회피는 활성 구간이 끝난 뒤부터 후딜레이를 끊는다(캔슬 창 = 무기 성격).
                if a.dodge && pl.dodge_cd <= 0.0 && after && spend(pl, t[P::StaminaDodge]) {
                    let dir = if world_move.length_squared() > 1e-4 {
                        world_move.normalize()
                    } else {
                        -pl.facing_dir()
                    };
                    pl.state = PState::Dodge { t: 0.0, dir };
                    pl.dodge_cd = t[P::DodgeTime] + 0.15;
                } else if a.parry && after && spend(pl, t[P::ParryStamina]) {
                    if let Some(d) = auto_dir.or(aim) {
                        pl.facing = d.x.atan2(d.z);
                    }
                    pl.state = PState::Parry { t: 0.0 };
                } else if a.cast && matches!(wpn.rb, Rb::Magic(_)) && pl.mana >= 1.0 && after {
                    // 마법(0048)도 회피처럼 평타 후딜을 끊는다 — 베고 바로 쏘기.
                    pl.mana -= 1.0;
                    pl.facing = cast_dir.x.atan2(cast_dir.z);
                    pl.state = PState::Cast {
                        t: 0.0,
                        dir: cast_dir,
                        fired: false,
                    };
                } else if a.attack_held
                    && pl.attack_hold >= t[P::HeavyHoldStart]
                    && after
                    && wpn.heavy.is_some()
                {
                    // 강공격(0030): 평타는 누르는 순간 나갔고, 한 번 누른 채 `heavy.hold start s`를 쥐고 있으면(활성이 끝난 뒤) 모으기로.
                    // "그 순간 눌려 있나"만 보면 연타 중 휘두르기 도중의 짧은 누름이 활성 끝에 걸려 모으기로 샜다.
                    pl.state = PState::Charging { t: 0.0 };
                } else if pl.buffer > 0.0
                    && !last
                    && e >= s0 + a0 + r0 * t[P::ComboLink]
                    && spend(pl, t[P::StaminaAttack])
                {
                    // 콤보(0051): 후딜 앞부분이 지나면 미리 받아 둔 누름으로 다음 타. 다시 겨눈다.
                    pl.buffer = 0.0;
                    pl.swings += 1;
                    if let Some(d) = auto_dir.or(aim) {
                        pl.facing = d.x.atan2(d.z);
                    }
                    let dir = if world_move.length_squared() > 1e-4 {
                        world_move.normalize()
                    } else {
                        pl.facing_dir()
                    };
                    pl.state = PState::Attack {
                        t: 0.0,
                        swing: pl.swings,
                        dir,
                        hit: hit + 1,
                    };
                } else if e + dt >= total {
                    pl.state = PState::Idle;
                    // 마지막 타 뒤에는 처음으로, 아니면 잠깐 다음 타를 기억한다.
                    pl.combo_next = if last { 0 } else { hit + 1 };
                    pl.combo_left = if last { 0.0 } else { t[P::ComboKeep] };
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
                    pl.state = PState::Heavy {
                        t: 0.0,
                        swing: pl.swings,
                        dir,
                        full: e >= full_at,
                        // 지팡이(0051): 다 모인 뒤 창 안에 떼면 딱 맞춤.
                        perfect: h.perfect.is_some_and(|w| e >= full_at && e <= full_at + w),
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
                        fire = Some(dir);
                    }
                    pl.state = if e2 >= t[P::CastTime] * k {
                        PState::Idle
                    } else {
                        PState::Cast { t: e2, dir, fired }
                    };
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
                        to: pl.pos + dir * far,
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

        // 이동.
        match pl.state {
            PState::Dodge { dir, .. } => {
                pl.vel = dir * (t[P::DodgeDist] / t[P::DodgeTime]);
            }
            PState::Attack { t: e, dir, hit, .. } => {
                // 선딜 + 활성 동안 `attack.lunge m` × 타의 배율만큼 입력한 쪽으로 내딛고, 후딜에는 선다(0018·0051).
                let (s0, a0, _) = hit_times(t, wpn, hit);
                let reach = s0 + a0;
                let lunge =
                    t[P::AttackLunge] * wpn.combo.get(hit as usize).map_or(1.0, |h| h.lunge);
                pl.vel = if e < reach {
                    dir * (lunge / reach.max(1e-3))
                } else {
                    Vec3::ZERO
                };
            }
            PState::Hurt { .. } => {
                pl.vel *= (1.0 - KNOCK_DECAY * dt).max(0.0);
            }
            PState::Turn { .. } | PState::Parry { .. } | PState::Land { .. } => pl.vel = Vec3::ZERO,
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
            | PState::Throw { .. } => {
                // 마시는 동안·강공격을 모으는 동안(0047)·마법을 쏘는 동안(0048)은 느리게 걷는다.
                let slow = match pl.state {
                    PState::Drink { .. } => t[P::FlaskWalk],
                    PState::Charging { .. } => {
                        t[P::HeavyChargeWalk] * wpn.heavy.map_or(1.0, |h| h.walk)
                    }
                    PState::Cast { .. } | PState::Throw { .. } => t[P::CastWalk],
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
        if std::mem::take(&mut self.cast_denied_now) {
            ev.push(Event::CastDenied);
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
        if let Some(dir) = self.cast_fire.take() {
            let pl = &self.player;
            let pos = pl.pos + Vec3::Y * (pl.air + BOLT_HEIGHT) + dir * (PLAYER_RADIUS + 0.2);
            let def = match wpn.rb {
                Rb::Magic(b) => b,
                Rb::Shot { bolt, .. } => bolt,
            };
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
                if (b.pos + b.dir * k).distance(c) <= en.radius + BOLT_RADIUS {
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
                en.hp -= hurt_by(en, b.def.damage, crit, t);
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
            en.hp -= hurt_by(en, damage, crit, t);
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
                let Some(h) = wpn.combo.get(hit as usize) else {
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
            en.hp -= hurt_by(en, s.damage, crit, t);
            if crit {
                en.exposed = 0.0;
                ev.push(Event::Crit {
                    at: en.pos + Vec3::Y * en.height * 0.6,
                });
            }
            en.hurt = 0.15;
            en.knock = dir * t[P::Knockback] * s.knock * if en.boss { 0.25 } else { 1.0 };
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
            self.hitstop = t[P::Hitstop] * if heavy_hit { HEAVY_HITSTOP } else { 1.0 };
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
        // 높낮이(0043): 곧게 못 가거나 높이가 다른 층이면 거리 지도를 따라 돌아간다(못 가면 그 자리에서 기다린다).
        if let Some(g) = &self.ground {
            let dist = g.flow(p);
            for en in &mut self.enemies {
                en.way = if (body - en.pos.y).abs() <= HIT_DY && g.straight_walkable(en.pos, p) {
                    None
                } else {
                    Some(g.next_step(&dist, en.pos).unwrap_or(en.pos))
                };
            }
        }
        let facing = self.player.facing_dir();
        // 앞쪽에서 오는가(패링은 바라보는 쪽 반원 안만 — 위치 선정도 기술, KB timing-parry).
        let in_front = |at: Vec3| {
            let d = vec3(at.x - p.x, 0.0, at.z - p.z).normalize_or_zero();
            d.dot(facing) > PARRY_FRONT
        };
        // 플레이어를 친 적의 자리(첫 하나) — 밀려나는 쪽.
        let mut hurt_player: Option<Vec3> = None;
        let mut parried: Vec<usize> = Vec::new();
        for (i, en) in self.enemies.iter_mut().enumerate() {
            let out = en.tick(p, dt);
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
            self.player.state = PState::Idle;
            self.player.since_parry = 0.0;
            self.hitstop = self.hitstop.max(t[P::Hitstop] * PARRY_HITSTOP);
            // 같은 틱에 다른 적에게 맞은 것은 그대로 맞는다(떼 속 패링은 하나만 막는다).
        }
        if let (Some(from), false) = (hurt_player, self.player.invulnerable()) {
            let pl = &mut self.player;
            pl.hp -= 1;
            // 맞은 뒤 무적은 짧게(예전 0.6초 — 그 동안 다시 맞을 수 없어 경직 줄이기가 쓸 데가 없었다).
            pl.hurt = t[P::HitIframes];
            // 경직(0018): 경직 중 또 맞으면 반씩 — 떼에 갇혀 계속 맞아도 손을 놓치지 않게.
            pl.stun_chain = if matches!(pl.state, PState::Hurt { .. }) {
                pl.stun_chain + 1
            } else {
                0
            };
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
pub fn hit_times(t: &Tuning, w: &weapons::Weapon, hit: u8) -> (f32, f32, f32) {
    let k = w.combo.get(hit as usize).map_or(1.0, |h| h.time);
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
/// 튕긴 적을 미는 거리(m).
const PARRY_PUSH: f32 = 0.8;
/// 패링 성공의 히트스톱 = 타격 히트스톱 × 이만큼(명중보다 크게).
const PARRY_HITSTOP: f32 = 2.5;

/// 이보다 빠르게(m/s) 움직이던 중이면 입력을 놓았어도 그 쪽으로 내딛는다.
const LUNGE_MOVING: f32 = 1.0;

/// 조준 스틱이 이보다 세면 "겨누는 중".
const AIM_STICK: f32 = 0.35;

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
        assert_eq!(w.enemies[0].hp, 9 - t[P::CastDamage].round() as i32);
        assert!(w.bolts.is_empty());
        assert!(matches!(w.player.state, PState::Idle));
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
    fn the_staff_rewards_a_perfectly_timed_release() {
        let t = Tuning::default();
        let w0 = weapons::Id::Staff.def();
        let full = t[P::HeavyCharge] * w0.heavy.unwrap().charge;
        for (extra, want) in [(0.05, true), (0.6, false)] {
            let mut w = World::new("test", 30.0);
            w.set_weapon(weapons::Id::Staff);
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
            assert_eq!(got, want, "released {extra}s after full");
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
                for _ in 0..if charged { full + 2 } else { 3 } {
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
                let want = t[P::HeavyDamage] as i32
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
        // 삼킨 뒤에는 회피로 끊기지 않는다.
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
        assert!(matches!(w.player.state, PState::Drink { gulped: true, .. }));
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
        let heavy_dmg = 1.0 + t[P::HeavyDamage] + t[P::HeavyFullBonus];
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

    /// 맞으면 잠깐 경직(못 움직임) — 경직 중 또 맞으면 반씩 짧아지고, 경직이 풀린 뒤 맞으면 다시 처음 길이(0018).
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

    /// 1m 단 위의 플레이어: 아래 잔재의 발톱도, 플레이어의 칼도 닿지 않는다. 경사로가 없으면 적은 아래에서 기다린다.
    #[test]
    fn different_levels_do_not_trade_blows() {
        let t = Tuning::default();
        // 아래 잔재는 1m 단 위에 가만히 선 플레이어를 못 친다.
        let mut w = ledge_yard(1.0);
        w.player.pos = vec3(0.0, 1.0, -4.4);
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
}
