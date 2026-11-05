//! 적 하나의 상태와 행동(0019). 종류별 수치는 `species.rs` 표, 여기는 행동 방식(근접·돌진·문지기)의 상태 기계.
//! 월드는 `Enemy::tick`을 부르고, 플레이어를 쳤는지·깨어났는지를 받아 피해·이벤트로 바꾼다(월드 경계는 그대로).
use macroquad::prelude::*;

use crate::species::{self, Behavior, Def, Kind};
use crate::tuning::PLAYER_RADIUS;

/// 넉백 속도가 초당 이만큼씩 준다(월드와 같은 값).
pub use crate::world::KNOCK_DECAY;
/// 쏟아지는 적이 이보다 느려지면 멈춘 것으로 본다.
const SLAM_SPEED: f32 = 2.0;
/// 예고 끝 이만큼(초)은 방향을 고정한다 — 읽은 대로 맞는다.
const TELE_LOCK: f32 = 0.12;
/// 체력 반 아래(문지기): 예고·쉬기 배율, 걸음 배율.
const ROUSED_TELE: f32 = 0.75;
const ROUSED_RECOVER: f32 = 0.7;
const ROUSED_SPEED: f32 = 1.15;

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
}

/// 바닥 예고 모양(렌더가 그린다).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TeleShape {
    Disc { r: f32 },
    Sector { r: f32, arc: f32 },
    Lane { len: f32, w: f32 },
    Ring { r: f32 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EState {
    /// 잠든 문지기(스테이지가 깨운다). 움직이지도 치지도 않는다.
    Dormant,
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
    /// 휘청(CC).
    Stagger {
        t: f32,
        dur: f32,
    },
    Dead,
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
}

/// 한 틱의 결과(월드가 피해·이벤트로 바꾼다).
#[derive(Default, Debug, PartialEq)]
pub struct Out {
    /// 플레이어를 쳤다.
    pub hit: bool,
    /// 이번 틱에 체력 반 아래로 깨어났다.
    pub roused: bool,
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
            landed: false,
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
        }
    }

    pub fn def(&self) -> &'static Def {
        species::def(self.kind)
    }

    pub fn alive(&self) -> bool {
        self.state != EState::Dead
    }

    /// 싸움에 끼어 있나 — 잠든 적(`Dormant`)은 몸은 있어도(부딪힘) 맞지도, 락온·조준·땅 돌리기에 잡히지도 않는다.
    pub fn active(&self) -> bool {
        !matches!(self.state, EState::Dead | EState::Dormant)
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
        self.atk != Atk::Wave
    }

    /// 지금 공격의 예고 길이(초).
    pub fn tele_dur(&self) -> f32 {
        let d = self.def();
        self.tele_mul()
            * match self.atk {
                Atk::Smash | Atk::Slam => d.tele,
                Atk::Charge => d.charge_tele,
                Atk::Wave => d.wave_tele,
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
            Atk::Charge => TeleShape::Lane {
                len: d.charge_speed * d.charge_time + d.radius,
                w: (d.radius + PLAYER_RADIUS) * 2.0,
            },
            Atk::Wave => TeleShape::Ring { r: d.wave_radius },
        }
    }

    /// 예고 진행 0..1과 모양(없으면 None). 렌더가 바닥 예고를 채운다.
    pub fn telegraph(&self) -> Option<(f32, TeleShape)> {
        match self.state {
            EState::Tele { t: e } => Some((
                (e / self.tele_dur().max(1e-3)).clamp(0.0, 1.0),
                self.tele_shape(),
            )),
            _ => None,
        }
    }

    /// 퍼지는 파동의 지금 반지름(파동 중일 때).
    pub fn wave_front(&self) -> Option<f32> {
        match self.state {
            EState::Wave { t } => Some(t * self.def().wave_speed),
            _ => None,
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
        self.atk = atk;
        self.atk_count += 1;
        self.landed = false;
        self.state = EState::Tele { t: 0.0 };
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
            Behavior::Warden => {
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
        self.exposed = (self.exposed - dt).max(0.0);
        self.sway_t += dt;
        // 넉백은 빠르게 줄어든다.
        self.pos += self.knock * dt;
        self.knock *= (1.0 - KNOCK_DECAY * dt).max(0.0);
        if self.knock.length() < SLAM_SPEED {
            self.spilling = false;
        }
        if d.boss && !self.roused && self.hp * 2 <= self.max_hp {
            self.roused = true;
            out.roused = true;
        }
        let to = vec3(player.x - self.pos.x, 0.0, player.z - self.pos.z);
        let dist = to.length();
        let dir = if dist > 1e-4 { to / dist } else { Vec3::Z };
        let speed = d.speed * if self.roused { ROUSED_SPEED } else { 1.0 };
        match self.state {
            EState::Dormant | EState::Dead => {}
            EState::Chase => {
                if let Some(w) = self.way {
                    let to = vec3(w.x - self.pos.x, 0.0, w.z - self.pos.z);
                    let l = to.length();
                    if l > 0.05 {
                        let d = to / l;
                        self.turn_toward(d, 6.0, dt);
                        self.pos += d * (speed * dt).min(l);
                    }
                } else {
                    self.turn_toward(dir, 6.0, dt);
                    match self.choose(dist) {
                        Some(atk) => self.start(atk),
                        None => self.pos += dir * speed * dt,
                    }
                }
            }
            EState::Tele { t: e } => {
                let dur = self.tele_dur();
                if e < dur - TELE_LOCK {
                    let rate = if self.atk == Atk::Charge { 4.0 } else { 2.0 };
                    self.turn_toward(dir, rate, dt);
                }
                self.state = if e + dt < dur {
                    EState::Tele { t: e + dt }
                } else {
                    match self.atk {
                        Atk::Smash | Atk::Slam => EState::Swing { t: 0.0 },
                        Atk::Charge => EState::Charge {
                            t: 0.0,
                            dir: self.facing_dir(),
                        },
                        Atk::Wave => EState::Wave { t: 0.0 },
                    }
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
                    EState::Recover { t: 0.0 }
                } else {
                    EState::Swing { t: e + dt }
                };
            }
            EState::Charge { t: e, dir: cdir } => {
                self.pos += cdir * d.charge_speed * dt;
                if !self.landed && dist <= d.radius + PLAYER_RADIUS + 0.15 {
                    self.landed = true;
                    out.hit = true;
                }
                self.state = if e + dt >= d.charge_time {
                    EState::Recover { t: 0.0 }
                } else {
                    EState::Charge {
                        t: e + dt,
                        dir: cdir,
                    }
                };
            }
            EState::Wave { t: e } => {
                let front = (e + dt) * d.wave_speed;
                if !self.landed && (dist - front).abs() <= 0.7 {
                    self.landed = true;
                    out.hit = true;
                }
                self.state = if front >= d.wave_radius {
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
        let out = en.tick(Vec3::ZERO, 1.0 / 60.0);
        assert!(out.roused && en.roused);
        assert!(en.tele_dur() < calm || en.atk != Atk::Slam);
        // 다시는 안 깨어난다.
        assert!(!en.tick(Vec3::ZERO, 1.0 / 60.0).roused);
    }

    #[test]
    fn a_dormant_warden_does_nothing() {
        let mut en = Enemy::new(Kind::Warden, vec3(0.0, 0.0, -2.0));
        en.state = EState::Dormant;
        assert_eq!(run(&mut en, Vec3::ZERO, 300), 0);
        assert_eq!(en.state, EState::Dormant);
    }
}
