//! 게임 규칙 하나. 카메라도 소리도 메뉴도 모른다 — `Actions`와 `Basis`(입력 기준)만 받고 `Event`를 내놓는다.
//! 틱은 단계 메서드 순서(nail 0169 R1): tick_lock → tick_player → apply_strike → apply_turn → tick_enemies → collide → cleanup.
use macroquad::prelude::*;

use crate::input::Actions;
use crate::tuning::{Tuning, P, PLAYER_HP, PLAYER_RADIUS, TICK_DT};

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
    /// 땅을 90° 비틀었다(0012). `cw` = 화면에서 보아 시계 방향. 앱이 카메라를 돌린다.
    Turn {
        cw: bool,
    },
    /// 쏟아진 적이 벽·기둥·아레나 끝에 부딪혔다.
    Slam {
        at: Vec3,
    },
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
    },
    /// 맞아서 경직(0018): 못 움직이고 못 친다. 밀려난 속도는 빠르게 준다.
    Hurt {
        t: f32,
        dur: f32,
    },
    /// 땅 돌리기: 발을 딛고 비튼다(`turn.plant s` 동안 못 움직인다 — 공짜가 아니다).
    Turn {
        t: f32,
        cw: bool,
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
    pub swings: u32,
}

impl Player {
    pub fn facing_dir(&self) -> Vec3 {
        let (s, c) = self.facing.sin_cos();
        vec3(s, 0.0, c)
    }
    pub fn invulnerable(&self) -> bool {
        matches!(self.state, PState::Dodge { .. }) || self.hurt > 0.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EState {
    Chase,
    /// 공격 예고. 다 차면 휘두른다.
    Tele {
        t: f32,
    },
    Swing {
        t: f32,
    },
    Recover {
        t: f32,
    },
    /// 땅이 도는 동안 실려 간다: 발을 떼지 못하고 버틴다(쫓지도 치지도 못함, 예고도 끊김). 멈추는 순간 휘청으로(0012).
    Carried,
    /// 휘청(CC): 쫓지도 치지도 못한다. 예고·스윙 중이었어도 끊긴다.
    Stagger {
        t: f32,
        dur: f32,
    },
    Dead,
}

pub struct Enemy {
    pub pos: Vec3,
    pub prev: Vec3,
    pub facing: f32,
    pub hp: i32,
    pub max_hp: i32,
    pub boss: bool,
    pub radius: f32,
    pub height: f32,
    /// 공격이 닿는 거리.
    pub reach: f32,
    pub state: EState,
    pub hurt: f32,
    pub knock: Vec3,
    /// 마지막으로 맞은 스윙 번호(한 스윙에 한 번만).
    pub hit_by: u32,
    /// 땅 돌리기로 쏟아지는 중 — 이때 벽에 부딪히면 벽 쿵(한 번).
    pub spilling: bool,
    /// 휘청 몸짓(렌더가 기울인다): 쏟아진 쪽(XZ) × 세기 0..1, 그리고 그때부터 지난 초. 벽 쿵이면 반대로 튄다.
    pub sway: Vec3,
    pub sway_t: f32,
}

impl Enemy {
    pub fn alive(&self) -> bool {
        self.state != EState::Dead
    }
    pub fn facing_dir(&self) -> Vec3 {
        let (s, c) = self.facing.sin_cos();
        vec3(s, 0.0, c)
    }
    /// 예고 진행 0..1(없으면 None). 렌더가 바닥 예고를 채운다.
    pub fn telegraph(&self, t: &Tuning) -> Option<f32> {
        match self.state {
            EState::Tele { t: e } => {
                let d = if self.boss {
                    t[P::BossTele]
                } else {
                    t[P::MobTele]
                };
                Some((e / d).clamp(0.0, 1.0))
            }
            _ => None,
        }
    }
}

pub struct World {
    pub name: &'static str,
    pub player: Player,
    pub enemies: Vec<Enemy>,
    pub lock: Option<usize>,
    pub obstacles: Vec<Aabb>,
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
    /// 이번 틱에 땅을 돌렸다(`cw`) — `apply_turn`이 쏟는다.
    turn_now: Option<bool>,
    /// 돌리는 중: (방향, 멈추기까지 남은 초). 0이 되는 틱에 쏟는다 — 히트스톱 중에도 줄어든다(카메라 시계와 맞춘다).
    spill_at: Option<(bool, f32)>,
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
                swings: 0,
            },
            enemies: Vec::new(),
            lock: None,
            obstacles: Vec::new(),
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
        }
    }

    pub fn spawn_mob(&mut self, pos: Vec3) {
        self.enemies.push(Enemy {
            pos,
            prev: pos,
            facing: 0.0,
            hp: 2,
            max_hp: 2,
            boss: false,
            radius: 0.35,
            height: 1.4,
            reach: 1.3,
            state: EState::Chase,
            hurt: 0.0,
            knock: Vec3::ZERO,
            hit_by: 0,
            spilling: false,
            sway: Vec3::ZERO,
            sway_t: 0.0,
        });
    }

    pub fn spawn_boss(&mut self, pos: Vec3) {
        self.enemies.push(Enemy {
            pos,
            prev: pos,
            facing: 0.0,
            hp: 30,
            max_hp: 30,
            boss: true,
            radius: 0.9,
            height: 3.2,
            reach: 3.2,
            state: EState::Chase,
            hurt: 0.0,
            knock: Vec3::ZERO,
            hit_by: 0,
            spilling: false,
            sway: Vec3::ZERO,
            sway_t: 0.0,
        });
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
                lock: self.held.lock | a.lock,
                turn_cw: self.held.turn_cw | a.turn_cw,
                turn_ccw: self.held.turn_ccw | a.turn_ccw,
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
            lock: a.lock | held.lock,
            turn_cw: a.turn_cw | held.turn_cw,
            turn_ccw: a.turn_ccw | held.turn_ccw,
            ..*a
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
            if !e.alive() {
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
        // 공격 자동 조준(0014): 락온이 아니면 휘두르는 순간 `aim.auto range m` 안의 가장 가까운 적을 본다.
        // 조준 스틱을 쥐고 있으면 그 쪽 `aim.auto cone deg` 안에서만 고른다(멀리 있는 적을 일부러 겨눌 수 있게).
        let auto_dir = if target.is_none() && a.attack {
            nearest_in_reach(
                &self.enemies,
                self.player.pos,
                t[P::AimRange],
                aim.map(|d| (d, (t[P::AimCone] * 0.5).to_radians().cos())),
            )
        } else {
            None
        };
        let pl = &mut self.player;
        pl.hurt = (pl.hurt - dt).max(0.0);
        pl.dodge_cd = (pl.dodge_cd - dt).max(0.0);
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
        pl.since_spill = (pl.since_spill + dt).min(f32::MAX);
        let world_move = move_to_world(a.move_dir, basis, pl.pos);
        // 땅 돌리기(0012): 쿼터뷰에서만, 서 있거나 스윙 후딜레이(회피와 같은 캔슬 자리)에서. ↻이 ↺보다 먼저(둘 다면 하나만).
        let turn = (a.turn_cw || a.turn_ccw).then_some(a.turn_cw);
        let can_turn = target.is_none()
            && pl.turn_cd <= 0.0
            && match pl.state {
                PState::Idle => true,
                PState::Attack { t: e, .. } => e >= t[P::AttackStartup] + t[P::AttackActive],
                _ => false,
            };
        if let (Some(cw), true) = (turn, can_turn) {
            pl.state = PState::Turn { t: 0.0, cw };
            pl.turn_cd = t[P::TurnCooldown];
            self.turn_now = Some(cw);
            // 화면이 돌아도 쥔 스틱은 돌기 전 기준 그대로(0010과 같은 규칙) — 놓거나 `move.hold` 초가 지나면 새 화면 기준.
            if stick_on {
                if let Basis::Quarter { yaw } = basis {
                    self.hold = Some((crate::camera::quarter_forward(yaw), 0.0));
                }
            }
        }

        // 상태 전이.
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
                } else if a.attack && !a.dodge && spend(pl, t[P::StaminaAttack]) {
                    pl.swings += 1;
                    // 휘두르는 쪽을 그 자리에서 정한다: 자동 조준 > 조준 스틱 > 지금 바라보는 쪽.
                    if let Some(d) = auto_dir.or(aim) {
                        pl.facing = d.x.atan2(d.z);
                    }
                    // 내딛는 쪽은 누른 순간의 입력(없으면 휘두르는 쪽, 0018).
                    let dir = if world_move.length_squared() > 1e-4 {
                        world_move.normalize()
                    } else {
                        pl.facing_dir()
                    };
                    pl.state = PState::Attack {
                        t: 0.0,
                        swing: pl.swings,
                        dir,
                    };
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
            } => {
                let total = t[P::AttackStartup] + t[P::AttackActive] + t[P::AttackRecovery];
                // 회피는 활성 구간이 끝난 뒤부터 후딜레이를 끊는다(캔슬 창 = 무기 성격).
                if a.dodge
                    && pl.dodge_cd <= 0.0
                    && e >= t[P::AttackStartup] + t[P::AttackActive]
                    && spend(pl, t[P::StaminaDodge])
                {
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
                    pl.state = PState::Attack {
                        t: e + dt,
                        swing,
                        dir: ldir,
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
            PState::Turn { t: e, cw } => {
                if e + dt >= t[P::TurnPlant] {
                    pl.state = PState::Idle;
                } else {
                    pl.state = PState::Turn { t: e + dt, cw };
                }
            }
        }

        // 이동.
        match pl.state {
            PState::Dodge { dir, .. } => {
                pl.vel = dir * (t[P::DodgeDist] / t[P::DodgeTime]);
            }
            PState::Attack { t: e, dir, .. } => {
                // 선딜 + 활성 동안 `attack.lunge m`만큼 입력한 쪽으로 내딛고, 후딜에는 선다(0018).
                let reach = t[P::AttackStartup] + t[P::AttackActive];
                pl.vel = if e < reach {
                    dir * (t[P::AttackLunge] / reach.max(1e-3))
                } else {
                    Vec3::ZERO
                };
            }
            PState::Hurt { .. } => {
                pl.vel *= (1.0 - KNOCK_DECAY * dt).max(0.0);
            }
            PState::Turn { .. } => pl.vel = Vec3::ZERO,
            PState::Idle => {
                let want = world_move * t[P::MoveSpeed];
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
            (_, PState::Attack { .. } | PState::Turn { .. } | PState::Hurt { .. }) => None,
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
    }

    /// 플레이어 스윙의 활성 구간에 범위·부채꼴 안의 적을 한 스윙에 한 번씩 때린다.
    fn apply_strike(&mut self, t: &Tuning, ev: &mut Vec<Event>) {
        let PState::Attack { t: e, swing, .. } = self.player.state else {
            return;
        };
        if e < t[P::AttackStartup] || e >= t[P::AttackStartup] + t[P::AttackActive] {
            return;
        }
        if e < t[P::AttackStartup] + TICK_DT {
            ev.push(Event::Swing);
        }
        let p = self.player.pos;
        let f = self.player.facing_dir();
        let half = (t[P::AttackArc] * 0.5).to_radians().cos();
        let mut stop = false;
        for en in &mut self.enemies {
            if !en.alive() || en.hit_by == swing {
                continue;
            }
            let d = en.pos - p;
            let dist = d.length() - en.radius;
            if dist > t[P::AttackRange] {
                continue;
            }
            let dir = if d.length() > 1e-4 { d.normalize() } else { f };
            if dir.dot(f) < half {
                continue;
            }
            en.hit_by = swing;
            en.hp -= 1;
            en.hurt = 0.15;
            en.knock = dir * t[P::Knockback] * if en.boss { 0.25 } else { 1.0 };
            ev.push(Event::Hit {
                at: en.pos + Vec3::Y * en.height * 0.6,
            });
            stop = true;
            if en.hp <= 0 {
                en.state = EState::Dead;
                self.kills += 1;
                ev.push(Event::EnemyDown { at: en.pos });
            }
        }
        if stop {
            self.hitstop = t[P::Hitstop];
        }
    }

    /// 땅 돌리기(0012). 누른 틱에 `Event::Turn`(카메라가 돈다), 그리고 카메라가 90°에 닿아 **멈추는 순간** 쏟는다:
    /// 돌던 쟁반이 급정거하면 위의 물건은 관성으로 돌던 쪽으로 계속 간다(버스가 서면 몸이 앞으로 쏠리듯 — 사장 "시작할 때가 아니라
    /// 멈출 때 흔들리는 것"). 축(주인공)에서 멀수록 많이 미끄러진다(속도 = 각속도 × 거리), 바깥 몫(`spill.outward share`)을 섞어
    /// "쏟아지게". 범위 안의 적은 모두 휘청 — 가까운 적은 덜 밀려 바로 칠 수 있고, 먼 적은 멀리 밀려 벽에 부딪힐 수 있다. 보스는 무거워서 덜.
    fn apply_turn(&mut self, t: &Tuning, ev: &mut Vec<Event>) {
        if let Some(cw) = self.turn_now.take() {
            ev.push(Event::Turn { cw });
            self.spill_at = Some((cw, crate::tuning::turn_land_secs(t)));
            // 도는 동안 범위 안의 적은 땅에 실려 버틴다 — 안 그러면 1초 동안 쫓아와 곁에 뭉쳐, 멈출 때 쏟을 거리가 없었다.
            let p = self.player.pos;
            for en in &mut self.enemies {
                if en.alive() && vec2(en.pos.x - p.x, en.pos.z - p.z).length() <= t[P::SpillRange] {
                    en.state = EState::Carried;
                }
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
            if !en.alive() {
                continue;
            }
            let d = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z);
            let r = d.length();
            if r > t[P::SpillRange] {
                if en.state == EState::Carried {
                    en.state = EState::Chase;
                }
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
        // 플레이어를 친 적의 자리(첫 하나) — 밀려나는 쪽.
        let mut hurt_player: Option<Vec3> = None;
        for en in &mut self.enemies {
            if !en.alive() {
                continue;
            }
            en.hurt = (en.hurt - dt).max(0.0);
            en.sway_t += dt;
            // 넉백은 빠르게 줄어든다.
            en.pos += en.knock * dt;
            en.knock *= (1.0 - KNOCK_DECAY * dt).max(0.0);
            if en.knock.length() < SLAM_SPEED {
                en.spilling = false;
            }
            let d = p - en.pos;
            let dist = d.length();
            let dir = if dist > 1e-4 { d / dist } else { Vec3::Z };
            let stand = if en.boss {
                en.reach - 0.6
            } else {
                en.reach - 0.3
            };
            match en.state {
                EState::Chase => {
                    en.facing = dir.x.atan2(dir.z);
                    if dist > stand {
                        let sp = if en.boss {
                            t[P::MobSpeed] * 0.8
                        } else {
                            t[P::MobSpeed]
                        };
                        en.pos += dir * sp * dt;
                    } else {
                        en.state = EState::Tele { t: 0.0 };
                    }
                }
                EState::Tele { t: e } => {
                    // 예고 동안 느리게 따라 돈다(끝 0.1초는 고정 — 읽은 대로 맞는다).
                    let dur = if en.boss {
                        t[P::BossTele]
                    } else {
                        t[P::MobTele]
                    };
                    if e < dur - 0.1 {
                        let want = dir.x.atan2(dir.z);
                        let mut diff = want - en.facing;
                        while diff > std::f32::consts::PI {
                            diff -= std::f32::consts::TAU;
                        }
                        while diff < -std::f32::consts::PI {
                            diff += std::f32::consts::TAU;
                        }
                        en.facing += diff.clamp(-2.0 * dt, 2.0 * dt);
                    }
                    en.state = if e + dt >= dur {
                        EState::Swing { t: 0.0 }
                    } else {
                        EState::Tele { t: e + dt }
                    };
                }
                EState::Swing { t: e } => {
                    if e == 0.0 {
                        // 닿는가: 거리 + 부채꼴(보스) / 거리(잡몹).
                        let arc = if en.boss {
                            (t[P::BossSwingArc] * 0.5).to_radians().cos()
                        } else {
                            -1.0
                        };
                        let reach = en.reach + PLAYER_RADIUS;
                        if dist <= reach && dir.dot(en.facing_dir()) >= arc {
                            hurt_player.get_or_insert(en.pos);
                        }
                    }
                    en.state = if e + dt >= 0.2 {
                        EState::Recover { t: 0.0 }
                    } else {
                        EState::Swing { t: e + dt }
                    };
                }
                EState::Recover { t: e } => {
                    let dur = if en.boss { 1.0 } else { 0.6 };
                    en.state = if e + dt >= dur {
                        EState::Chase
                    } else {
                        EState::Recover { t: e + dt }
                    };
                }
                EState::Carried => {}
                EState::Stagger { t: e, dur } => {
                    en.state = if e + dt >= dur {
                        EState::Chase
                    } else {
                        EState::Stagger { t: e + dt, dur }
                    };
                }
                EState::Dead => {}
            }
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
                let min = self.enemies[i].radius + self.enemies[j].radius;
                let len = d.length();
                if len < min && len > 1e-4 {
                    let push = d / len * (min - len) * 0.5;
                    self.enemies[i].pos -= push;
                    self.enemies[j].pos += push;
                }
            }
            if self.enemies[i].alive() {
                let d = self.player.pos - self.enemies[i].pos;
                let min = self.enemies[i].radius + PLAYER_RADIUS;
                let len = d.length();
                if len < min && len > 1e-4 {
                    self.player.pos += d / len * (min - len);
                }
            }
        }
        let arena = self.arena;
        let obstacles = &self.obstacles;
        let clamp = |p: Vec3, r: f32| -> Vec3 {
            let mut q = p;
            for b in obstacles {
                q = b.push_circle(q, r);
            }
            vec3(
                q.x.clamp(-arena + r, arena - r),
                0.0,
                q.z.clamp(-arena + r, arena - r),
            )
        };
        self.player.pos = clamp(self.player.pos, PLAYER_RADIUS);
        for en in &mut self.enemies {
            let q = clamp(en.pos, en.radius);
            let blocked = q.distance(en.pos) > 1e-3;
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
        .filter(|e| e.alive())
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
/// 쏟아지는 적이 이보다 느려지면 멈춘 것으로 본다(그 뒤 벽에 닿아도 벽 쿵이 아니다).
const SLAM_SPEED: f32 = 2.0;

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
        assert_eq!(w.enemies[0].hp, 1);
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

    fn turn(cw: bool) -> Actions {
        Actions {
            turn_cw: cw,
            turn_ccw: !cw,
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
        assert!(ev.contains(&Event::Turn { cw: true }));
        // 도는 동안(멈추기 전)엔 아무도 밀리지 않는다.
        let half = (crate::tuning::turn_land_secs(&t) * 0.5 / TICK_DT) as usize;
        for _ in 0..half {
            w.step(&Actions::default(), &t);
        }
        assert!(w.enemies[1].pos.distance(up * 7.0) < 1e-4);
        assert!(!matches!(w.enemies[1].state, EState::Stagger { .. }));
        for _ in 0..600 {
            if w.spill_at.is_none() {
                break;
            }
            w.step(&Actions::default(), &t);
        }
        assert!(matches!(w.enemies[1].state, EState::Stagger { .. }));
        for _ in 0..40 {
            w.step(&Actions::default(), &t);
        }
        let near = w.enemies[0].pos - up * 3.0;
        let far = w.enemies[1].pos - up * 7.0;
        assert!(
            near.dot(right) > 0.3 && far.dot(right) > 0.3,
            "{near:?} {far:?}"
        );
        assert!(far.length() > near.length() * 1.8, "{near:?} {far:?}");
        // 바깥 몫: 먼 적은 축에서도 멀어진다.
        assert!(w.enemies[1].pos.length() > 7.0);
        // 반시계면 반대쪽.
        let mut w2 = World::new("test", 30.0);
        w2.view_yaw = Some(35.0);
        w2.spawn_mob(up * 7.0);
        idle(&mut w2);
        turn_until_spill(&mut w2, false, &t);
        for _ in 0..40 {
            w2.step(&Actions::default(), &t);
        }
        assert!((w2.enemies[0].pos - up * 7.0).dot(right) < -0.3);
    }

    #[test]
    fn turning_staggers_everyone_in_range_when_it_stops_and_cuts_their_telegraph() {
        let t = Tuning::default();
        let mut w = World::new("test", 30.0);
        w.spawn_mob(vec3(0.0, 0.0, -1.2));
        w.spawn_mob(vec3(0.0, 0.0, t[P::SpillRange] + 2.0));
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

    /// 도는 동안엔 범위 안의 적이 실려 버틴다: 쫓아오지도 치지도 못한다(멈추면 휘청).
    #[test]
    fn while_the_ground_turns_the_enemies_on_it_are_carried_and_cannot_close_in() {
        let t = Tuning::default();
        let mut w = World::new("test", 30.0);
        w.spawn_mob(vec3(0.0, 0.0, -1.5));
        w.spawn_mob(vec3(0.0, 0.0, -5.0));
        w.spawn_mob(vec3(0.0, 0.0, t[P::SpillRange] + 3.0));
        w.step(&turn(true), &t);
        let mut hurt = false;
        while w.spill_at.is_some_and(|(_, left)| left > TICK_DT) {
            hurt |= w.step(&Actions::default(), &t).contains(&Event::PlayerHurt);
        }
        assert!(!hurt);
        assert_eq!(w.enemies[0].state, EState::Carried);
        assert!(w.enemies[1].pos.distance(vec3(0.0, 0.0, -5.0)) < 1e-4);
        // 범위 밖은 그대로 쫓아온다.
        assert!(w.enemies[2].pos.z < t[P::SpillRange] + 3.0 - 1.0);
        for _ in 0..5 {
            w.step(&Actions::default(), &t);
        }
        assert!(matches!(w.enemies[1].state, EState::Stagger { .. }));
    }

    #[test]
    fn the_boss_is_heavy_it_barely_slides_and_staggers_briefly() {
        let t = Tuning::default();
        let mut w = World::new("test", 30.0);
        w.spawn_boss(vec3(0.0, 0.0, -6.0));
        w.spawn_mob(vec3(0.0, 0.0, 6.0));
        idle(&mut w);
        turn_until_spill(&mut w, true, &t);
        // 보스 휘청(0.3초)이 끝나기 전에 잰다 — 그 뒤엔 다시 쫓아온다.
        for _ in 0..15 {
            w.step(&Actions::default(), &t);
        }
        let boss = (w.enemies[0].pos - vec3(0.0, 0.0, -6.0)).length();
        let mob = (w.enemies[1].pos - vec3(0.0, 0.0, 6.0)).length();
        assert!(boss * 3.0 < mob, "boss {boss} mob {mob}");
        for _ in 0..15 {
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
                turn_cw: true,
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

    #[test]
    fn a_spilled_enemy_that_hits_a_pillar_is_slammed_once() {
        let t = Tuning::default();
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
        assert_eq!(w.enemies[0].hp, 2 - t[P::SlamDamage] as i32);
    }

    #[test]
    fn holding_the_stick_through_a_turn_keeps_walking_the_same_world_way() {
        let t = Tuning::default();
        let mut w = World::new("test", 30.0);
        w.view_yaw = Some(35.0);
        let up = Actions {
            move_dir: vec2(0.0, 1.0),
            ..Default::default()
        };
        for _ in 0..20 {
            w.step(&up, &t);
        }
        w.step(
            &Actions {
                turn_cw: true,
                ..up
            },
            &t,
        );
        w.view_yaw = Some(125.0); // 앱이 카메라에서 넘기는 새 방위
        let plant = (t[P::TurnPlant] / TICK_DT).ceil() as usize;
        for _ in 0..plant + 8 {
            w.step(&up, &t);
        }
        let old_up = crate::camera::quarter_forward(35.0);
        assert!(
            w.player.vel.normalize().dot(old_up) > 0.98,
            "{:?}",
            w.player.vel
        );
        // 놓았다 다시 쥐면 새 화면 위.
        w.step(&Actions::default(), &t);
        for _ in 0..30 {
            w.step(&up, &t);
        }
        let new_up = crate::camera::quarter_forward(125.0);
        assert!(w.player.vel.normalize().dot(new_up) > 0.98);
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
