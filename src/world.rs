//! 게임 규칙 하나. 카메라도 소리도 메뉴도 모른다 — `Actions`와 `Basis`(입력 기준)만 받고 `Event`를 내놓는다.
//! 틱은 단계 메서드 순서(nail 0169 R1): tick_player → tick_enemies → apply_strike → hit_player → collide → cleanup.
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
    Hit { at: Vec3 },
    EnemyDown { at: Vec3 },
    PlayerHurt,
    LockOn,
    LockOff,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PState {
    Idle,
    Dodge { t: f32, dir: Vec3 },
    Attack { t: f32, swing: u32 },
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
    /// 지난 틱의 락온 앞(수평 단위), 락온이 아니면 None — 풀리는 틱에 `hold`로 넘긴다.
    last_lock_fwd: Option<Vec3>,
    /// 풀린 뒤 쥔 스틱의 기준(앞, 경과 초). 스틱을 놓거나 `move.hold after unlock s`가 지나면 끝.
    hold: Option<(Vec3, f32)>,
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
            last_lock_fwd: None,
            hold: None,
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
        });
    }

    pub fn lock_target(&self) -> Option<Vec3> {
        self.lock
            .and_then(|i| self.enemies.get(i))
            .filter(|e| e.alive())
            .map(|e| e.pos)
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
        if self.hitstop > 0.0 {
            self.hitstop -= dt;
            // 보간이 미끄러지지 않게 prev를 지금으로.
            self.player.prev = self.player.pos;
            for e in &mut self.enemies {
                e.prev = e.pos;
            }
            return ev;
        }
        self.time += dt;
        self.player.prev = self.player.pos;
        for e in &mut self.enemies {
            e.prev = e.pos;
        }
        self.tick_lock(a, &mut ev);
        self.tick_player(a, t, dt);
        self.apply_strike(t, &mut ev);
        self.tick_enemies(t, dt, &mut ev);
        self.collide();
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
                        Basis::Quarter { yaw: t[P::CamQYaw] }
                    }
                }
            }
        };
        let pl = &mut self.player;
        pl.hurt = (pl.hurt - dt).max(0.0);
        pl.dodge_cd = (pl.dodge_cd - dt).max(0.0);
        let world_move = move_to_world(a.move_dir, basis, pl.pos);

        // 상태 전이.
        match pl.state {
            PState::Idle => {
                if a.dodge && pl.dodge_cd <= 0.0 {
                    let dir = if world_move.length_squared() > 1e-4 {
                        world_move.normalize()
                    } else {
                        -pl.facing_dir()
                    };
                    pl.state = PState::Dodge { t: 0.0, dir };
                    pl.dodge_cd = t[P::DodgeTime] + 0.15;
                } else if a.attack {
                    pl.swings += 1;
                    pl.state = PState::Attack {
                        t: 0.0,
                        swing: pl.swings,
                    };
                }
            }
            PState::Attack { t: e, swing } => {
                let total = t[P::AttackStartup] + t[P::AttackActive] + t[P::AttackRecovery];
                // 회피는 활성 구간이 끝난 뒤부터 후딜레이를 끊는다(캔슬 창 = 무기 성격).
                if a.dodge && pl.dodge_cd <= 0.0 && e >= t[P::AttackStartup] + t[P::AttackActive] {
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
                    pl.state = PState::Attack { t: e + dt, swing };
                }
            }
            PState::Dodge { t: e, dir } => {
                if e + dt >= t[P::DodgeTime] {
                    pl.state = PState::Idle;
                } else {
                    pl.state = PState::Dodge { t: e + dt, dir };
                }
            }
        }

        // 이동.
        match pl.state {
            PState::Dodge { dir, .. } => {
                pl.vel = dir * (t[P::DodgeDist] / t[P::DodgeTime]);
            }
            PState::Attack { t: e, .. } => {
                // 스윙 중엔 멈추되 선딜 동안 짧게 전진(lunge).
                pl.vel = if e < t[P::AttackStartup] {
                    pl.facing_dir() * 3.0
                } else {
                    Vec3::ZERO
                };
            }
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

        // 바라보기: 락온이면 대상, 아니면 이동 방향으로 돈다. 스윙 중엔 고정.
        let want_yaw = match (basis, pl.state) {
            (_, PState::Attack { .. }) => None,
            (Basis::Lock(tg), _) => {
                let d = tg - pl.pos;
                Some(d.x.atan2(d.z))
            }
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
        let PState::Attack { t: e, swing } = self.player.state else {
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

    fn tick_enemies(&mut self, t: &Tuning, dt: f32, ev: &mut Vec<Event>) {
        let p = self.player.pos;
        let mut hurt_player = false;
        for en in &mut self.enemies {
            if !en.alive() {
                continue;
            }
            en.hurt = (en.hurt - dt).max(0.0);
            // 넉백은 빠르게 줄어든다.
            en.pos += en.knock * dt;
            en.knock *= (1.0 - 10.0 * dt).max(0.0);
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
                            hurt_player = true;
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
                EState::Dead => {}
            }
        }
        if hurt_player && !self.player.invulnerable() {
            self.player.hp -= 1;
            self.player.hurt = 0.6;
            ev.push(Event::PlayerHurt);
        }
    }

    /// 겹침 풀기(적끼리·플레이어와), 기둥·아레나 경계.
    fn collide(&mut self) {
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
            en.pos = clamp(en.pos, en.radius);
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
