//! 방랑자의 동작 고르기(0251 R9 — `render::actors::player`에서 뺐다): 월드의 상태 → 틀 동작 이름의 차례 + 그 시각.
//!
//! 그리기를 모르는 순수한 셈이라 시험으로 지킨다. 렌더는 고른 이름을 모델에서 찾고(`Entry::pick`), 섞기·2차 동작·그리기만 한다.
//! 프레임을 넘어 남는 것(숨 고리·걸음 시계·겨눈 채 머묾·구른 뒤 일어나기)은 `Memory` 한 곳에.
use macroquad::math::{vec3, Vec3};

use crate::tuning::{Tuning, TICK_DT};
use crate::world::{PState, World};

/// 모델이 가진 동작(이름 → 길이). 렌더는 모델 묶음(`models::Entry`)으로, 시험은 가짜 목록으로.
pub trait Clips {
    /// 그 이름의 동작 길이(초). 없으면 None.
    fn dur(&self, name: &str) -> Option<f32>;
    fn has(&self, name: &str) -> bool {
        self.dur(name).is_some()
    }
}

/// 고른 동작: 앞에서부터 모델에 있는 첫 이름을 튼다.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pick {
    list: [&'static str; 4],
    n: usize,
    /// 동작의 시각(초).
    pub at: f32,
    /// 서 있는 동안의 숨 고리·걸음 시계처럼 되풀이되는 동작인가(아니면 끝 자세에서 멈춘다).
    pub looped: bool,
}

impl Pick {
    fn of(names: &[&'static str], at: f32) -> Pick {
        let mut list = [""; 4];
        let n = names.len().min(4);
        list[..n].copy_from_slice(&names[..n]);
        Pick {
            list,
            n,
            at,
            looped: false,
        }
    }

    pub fn names(&self) -> &[&'static str] {
        &self.list[..self.n]
    }
}

/// 프레임을 넘어 남는 동작의 기억.
#[derive(Clone, Copy, Debug, Default)]
pub struct Memory {
    idle: crate::idle::Idle,
    /// 지난 프레임의 월드 시각 — 이번 프레임에 월드 시계로 지난 초(멈춤 창·히트스톱 동안 0)를 잰다.
    idle_at: f32,
    gait: crate::gait::Gait,
    /// 쇠뇌를 쏜 뒤 겨눈 채 머무는 남은 초(0197).
    aim_hold: f32,
    /// 구른 뒤 일어나기(0238 — `gait::rise_after_roll`): 지난 프레임이 앞구르기였나, 일어나기의 시각.
    was_roll: bool,
    rise: Option<f32>,
    /// 이번 프레임에 월드 시계로 지난 초(렌더의 섞기·다리 돌림이 같이 쓴다).
    pub dt: f32,
}

/// 방랑자의 땅 위 빠르기(m/s) — 걷기·서기를 가른다.
pub fn speed(w: &World) -> f32 {
    (w.player.pos - w.player.prev).length() / TICK_DT
}

/// 걷는 중인가(서기 상태에서 움직인다).
pub fn walking(w: &World) -> bool {
    matches!(w.player.state, PState::Idle) && speed(w) > 0.5
}

impl Memory {
    /// 한 프레임: 월드를 보고 틀 동작을 고르고, 기억을 앞으로.
    pub fn pick(&mut self, w: &World, t: &Tuning, clips: &impl Clips) -> Pick {
        let pl = &w.player;
        let speed = speed(w);
        let wpn = pl.weapon.def();
        // 빈 손의 결(0276 — 두 손으로 쥔 한손검은 망치의 내려치기)까지 든 강공격.
        let hv = w.heavy_of();
        let combo_anim = |hit: u8| wpn.hit(hit).map_or("attack", |h| h.anim);
        let shot = matches!(wpn.rb, crate::weapons::Rb::Shot { .. });
        // 던지는 RB(0274 ②b — 손도끼 · 창 · 돌): 머리 위로 휘둘러 던지는 동작(스킬 던지기의 것).
        // 0276: 방패를 든 한손 무기면 방패를 던진다(같은 동작).
        let toss = !shot && w.rb_bolts().0.thrown();
        // 땅 내려찍기(0274 ④ — 대검의 RB): 두 손으로 머리 위에서 내려찍는 동작(평타 끝 타 `gs5`)의 닿는 자리가 탄이 나가는 때에.
        let slam = !shot && w.rb_bolts().0.look == crate::weapons::QUAKE;
        // 서 있는 동안(0153): 숨 고리 + 조용할 때 가끔 몸짓. 조용함 = 락온이 아니고 가까이 깨어 있는 적이 없다.
        // 월드 시계로 지난 초(멈춤 창·히트스톱 동안 0) — 서 있는 몸짓·걸음 시계·섞기가 같이 쓴다.
        let wdt = (w.time - self.idle_at).clamp(0.0, 0.1);
        self.dt = wdt;
        self.idle_at = w.time;
        let idle = matches!(pl.state, PState::Idle);
        let idle_pose = {
            use crate::idle::FIDGETS;
            let standing = idle && speed <= 0.5;
            let calm = w.lock.is_none()
                && !w.enemies.iter().any(|en| {
                    en.active() && en.pos.distance(pl.pos) < crate::tuning::IDLE_CALM_RADIUS
                });
            let dur_of = |n: &str| clips.dur(n).unwrap_or(0.0);
            self.idle
                .step(wdt, standing, calm, dur_of("idle"), FIDGETS.map(dur_of))
        };
        // 쇠뇌를 쏜 뒤 겨눈 채 머무는 시계(0197): 쏘는 동안 차고, 서 있는 동안 줄고, 움직이거나 다른 동작이면 없어진다.
        let shooting = matches!(pl.state, PState::Cast { .. } | PState::CastCharge { .. }) && shot;
        self.aim_hold = if shooting {
            AIM_HOLD
        } else if idle && speed <= 0.5 {
            (self.aim_hold - wdt).max(0.0)
        } else {
            0.0
        };
        // 구른 뒤 일어나기(0238): 앞구르기가 끝나 조용한 상태로 들어가면 `dodge_rise`를 잇는다(공격·다시 회피면 바로 그 동작).
        let roll_now = matches!(pl.state, PState::Dodge { dir, .. } if dir.dot(pl.facing_dir()) >= BACKSTEP_DOT);
        self.rise = crate::gait::rise_after_roll(self.rise, self.was_roll, idle, wdt);
        self.was_roll = roll_now;
        let rise = self.rise.filter(|_| clips.has("dodge_rise"));
        // 활(0231)은 활쏘기 원본의 제 동작 — 옆으로 서서 왼손의 활로 겨눈다(없는 모델이면 쇠뇌의 것).
        let bow = wpn.id == crate::weapons::Id::Bow;
        let (shoot, draw): (&[&'static str], &[&'static str]) = if bow {
            (
                &["bow_shoot", "xbow_shoot", "cast", "attack"],
                &["bow_draw", "xbow_charge", "bow_shoot", "cast"],
            )
        } else {
            (
                &["xbow_shoot", "cast", "attack"],
                &["xbow_charge", "xbow_shoot", "cast"],
            )
        };
        let pick = match pl.state {
            // 콤보(0051): 무기 표의 타마다 동작(없는 모델이면 평타).
            // 몰아치기(0158 G2) 동안은 평타의 시계가 빨라진다 — 동작도 같은 배로 돌린다.
            // 동작 시계는 타의 판정 시작에 동작이 만든 판정 시작이 오게(0195 — 맨손·대시처럼 남의 동작을 빌린 타).
            PState::Attack { t: e, hit, .. } => {
                let s0 = crate::world::hit_times(t, wpn, hit).0;
                let rate = crate::weapons::clip_rate(combo_anim(hit), s0);
                Pick::of(
                    &[combo_anim(hit), "attack", "attack"],
                    e * w.swing_rate() * rate,
                )
            }
            // 강공격(0030): 모으는 동안 칼을 뒤로 끌어 든 자세(없는 모델이면 공격 첫 자세), 떼면 크게 내려친다 — 무기마다(0051).
            PState::Charging { t: e } => Pick::of(
                &[hv.map_or("charge", |h| h.anim_charge), "charge", "attack"],
                e,
            ),
            PState::Heavy { t: e, .. } => {
                let anim = hv.map_or("heavy", |h| h.anim_heavy);
                let s0 = crate::world::heavy_times(t, wpn).0;
                Pick::of(
                    &[anim, "heavy", "attack"],
                    e * crate::weapons::clip_rate(anim, s0),
                )
            }
            // 수류탄(0051): 오른손으로 아래에서 위로 던진다.
            PState::Throw { t: e, .. } => Pick::of(&["throw", "cast", "attack"], e),
            // 등잔(0031): 왼손으로 들어 입에 대고(들기 = `flask.raise s`) 내린다.
            PState::Drink { t: e, .. } => Pick::of(&["drink", "idle"], e),
            // 0197: 몸이 보는 쪽의 반대로 피하면 뒤로 뛰기(구르기는 앞으로 구르는 동작이다).
            PState::Dodge { t: e, dir } if dir.dot(pl.facing_dir()) < BACKSTEP_DOT => {
                Pick::of(&["dodge_back", "dodge", "walk"], e)
            }
            PState::Dodge { t: e, .. } => Pick::of(&["dodge", "walk"], e),
            PState::Hurt { t: e, .. } | PState::Recoil { t: e } => Pick::of(&["hurt", "idle"], e),
            // 0034: 도는 쪽으로 윗몸이 먼저 감긴다(오른쪽 = 요가 준다).
            PState::Turn { t: e, from, to } => Pick::of(
                if crate::world::turn_is_right(from, to) {
                    &["turn_r", "turn", "idle"]
                } else {
                    &["turn_l", "turn", "idle"]
                },
                e,
            ),
            PState::Parry { t: e } => Pick::of(&["parry", "attack"], e),
            // 방패 막기(0223): 방패를 내민 자리에 머문다(`block` — 끝 자세에서 멈춘다).
            PState::Block { t: e } => Pick::of(&["block", "parry", "attack"], e),
            // 들이받기(0276): 방패 돌진(스킬)의 밀기 동작 — 당겨 든 자리(`RUSH_CLIP_FROM`)부터, 내달리는 동안이 미는 구간에 오게.
            PState::Rush { t: e, .. } => {
                Pick::of(&["sk_ram", "parry_push", "block"], RUSH_CLIP_FROM + e)
            }
            // 마법(0048): 손을 앞으로 내미는 자세(전용 동작이 생기기 전엔 패링 자세).
            PState::Cast { t: e, .. } if shot => Pick::of(shoot, e),
            // 던지기(0274 ②b): 손을 떠나는 자리(`TOSS_RELEASE`)가 탄이 나가는 때(`cast.startup s`)에 오게 돌린다.
            PState::Cast { t: e, .. } if slam => Pick::of(
                &["gs5", "hammer3", "cast"],
                e * SLAM_RELEASE / t[crate::tuning::P::CastStartup].max(1e-3),
            ),
            PState::Cast { t: e, .. } if toss => Pick::of(
                &["sk_throw", "throw", "cast"],
                e * TOSS_RELEASE / t[crate::tuning::P::CastStartup].max(1e-3),
            ),
            PState::Cast { t: e, .. } => Pick::of(&["cast", "parry", "attack"], e),
            // 원거리 강공격 모으기(0056): 쇠뇌는 어깨에 대고 시위를 당긴 채, 마법은 두 손을 모아 빛을 키우는 자세를 쥔다.
            // 활(0274 ②b — 당기는 길이를 고른다): 당기는 동작(0.3초)을 끝까지 당기는 시간(`cast.charge s`)에 맞춰 늘인다.
            PState::CastCharge { t: e } if bow => Pick::of(
                draw,
                e * BOW_DRAW_CLIP / t[crate::tuning::P::CastCharge].max(1e-3),
            ),
            PState::CastCharge { t: e } if shot => Pick::of(draw, e),
            // 내려찍을 것을 모으는 동안은 머리 위로 든 자세에 머문다(0274 ④).
            PState::CastCharge { t: e } if slam => {
                Pick::of(&["gs5", "hammer3", "cast"], e.min(SLAM_WINDUP))
            }
            // 던질 것을 모으는 동안은 머리 위로 든 자세에 머문다.
            PState::CastCharge { t: e } if toss => {
                Pick::of(&["sk_throw", "throw", "cast"], e.min(TOSS_WINDUP))
            }
            PState::CastCharge { t: e } => Pick::of(&["cast_charge", "cast", "parry"], e),
            // 공중(0027): 방랑자의 jump·plunge·land 동작(없는 모델이면 회피·공격 자세로).
            // 공중 RB(0111): 떠 있는 동안 쏘는 자세(쇠뇌는 겨누고 쏘기, 마법은 손을 내밂).
            // 0196: 떠 있는 동안 쏜 순간(놓는 자리)부터 흐른다 — 예전엔 한 장을 세워 두었다.
            PState::Jump { plunge: false, .. } if pl.ranged.hover > 0.0 && shot => Pick::of(
                shoot,
                XBOW_RELEASE + (crate::tuning::AIR_SHOT_HOVER - pl.ranged.hover),
            ),
            PState::Jump { plunge: false, .. } if pl.ranged.hover > 0.0 => Pick::of(
                &["cast", "jump"],
                CAST_RELEASE + (crate::tuning::AIR_SHOT_HOVER - pl.ranged.hover),
            ),
            // 점프(0196): 위 속도로 동작의 자리를 고른다 — 오르는 동안 앞 반, 꼭대기 = 가운데, 내려오는 동안 뒤 반(턱에서 떨어지면
            // 꼭대기에서 시작). 예전엔 꼭대기 한 장을 세워 두었다.
            PState::Jump { plunge: false, .. } => Pick::of(
                &["jump", "dodge", "idle"],
                JUMP_CLIP * jump_phase(pl.vy, crate::tuning::jump_ballistics(t).0),
            ),
            // 내려찍기: 높이로 동작의 자리를 고른다 — 꼭대기에서 칼을 들고, 땅에 가까울수록 꽂는 자세로(예전엔 두 장을 바꿔 끼웠다).
            PState::Jump { plunge: true, .. } => Pick::of(
                &["plunge", "attack"],
                PLUNGE_STRIKE * plunge_phase(pl.air, t[crate::tuning::P::JumpHeight]),
            ),
            // 0202: 내려찍기로 내려앉으면 칼을 꽂은 웅크림에서 일어난다(예전엔 보통 착지라 내려찍은 자세가 한 틱뿐이었다).
            PState::Land {
                t: e,
                dur,
                plunge: true,
            } => Pick::of(
                &["plunge_land", "land", "idle"],
                PLUNGE_LAND_CLIP * e / dur.max(1e-3),
            ),
            PState::Land { t: e, dur, .. } => {
                Pick::of(&["land", "idle"], LAND_CLIP * e / dur.max(1e-3))
            }
            // 스킬·큰 기술(0151 → 0158 G2): 전용 동작(`sk_*`·`ult_*`)이 모델에 있으면 그것을 — 상태의 구간(시동·이어짐·후딜)을
            // 동작의 구간에 맞춘다(`Clip::at`: 단·고르기가 길이를 바꿔도 판정의 순간에 그 자세). 없으면 빌려 쓰던 동작(표의 `anim`).
            PState::Skill { id, t: el, .. } => {
                let eff = w.skill_eff(id);
                let sk = eff.skill;
                if clips.has(sk.clip.name) {
                    let at = sk
                        .clip
                        .at(el, sk.startup, eff.active(), clip_period(&sk.form));
                    Pick::of(&[sk.clip.name, sk.anim, "attack"], at)
                } else {
                    Pick::of(&[sk.anim, "heavy", "attack"], el)
                }
            }
            PState::Ult { kind, t: el, .. } => {
                let clip = kind.clip();
                if clips.has(clip.name) {
                    // 세계 멈춤 동안은 상태 시계가 0에 머문다 — 멈춤이 지난 몫으로 시동 구간을 돌린다.
                    let (freeze, ..) = kind.times();
                    let frozen = 1.0 - (w.hitstop / freeze.max(1e-3)).clamp(0.0, 1.0);
                    Pick::of(
                        &[clip.name, kind.anim(), "attack"],
                        kind.clip_at(el, frozen),
                    )
                } else {
                    Pick::of(&[kind.anim(), "heavy", "attack"], el)
                }
            }
            // 구른 뒤 일어나기(0238).
            PState::Idle if rise.is_some() => {
                Pick::of(&["dodge_rise", "idle"], rise.unwrap_or(0.0))
            }
            // 패링이 튕긴 뒤(0199 → 0202): 민 자리에서 거둔다 — 걷기보다 먼저(튕기자마자 걸으면 밀기가 사라져 "방패 동작이
            // 안 보인다"). 튕기면 월드가 곧장 서기로 돌린다(반격하라고), 그 짧은 동안만.
            PState::Idle
                if pl.parry.since < PARRY_PUSH_FIRM
                    || (pl.parry.since < PARRY_PUSH_CLIP && speed <= 0.5) =>
            {
                Pick::of(&["parry_push", "idle"], pl.parry.since)
            }
            // 달리기(0076): `run` 동작이 없으면 걷기를 빠르게(발 구르기가 속도를 따라간다).
            // 걸음 시계는 쌓는다(0175 — `w.time × 빠르기`는 가속하는 동안 걸음 주기를 건너뛰었다, `gait.rs`).
            PState::Idle if pl.running && speed > 0.5 => Pick::of(
                &["run", "walk"],
                self.gait.advance(wdt, true, (speed / 4.0).clamp(0.6, 2.2)),
            ),
            PState::Idle if speed > 0.5 => {
                let (names, _) = move_anim(pl.pos - pl.prev, pl.facing_dir(), pl.faces_away);
                let k = if names[0] == "walk_back" {
                    WALK_BACK_RATE
                } else {
                    1.0
                };
                Pick::of(
                    names,
                    self.gait
                        .advance(wdt, true, (speed / 4.0).clamp(0.6, 1.6) * k),
                )
            }
            // 쇠뇌를 쏜 뒤(0197): 서 있으면 잠깐 겨눈 채(쏘기의 끝 자세).
            PState::Idle if self.aim_hold > 0.0 => Pick::of(shoot, XBOW_HOLD_AT),
            PState::Idle => match idle_pose {
                crate::idle::Pose::Loop(t) => Pick::of(&["idle"], t),
                crate::idle::Pose::Fidget(i, t) => Pick::of(&[crate::idle::FIDGETS[i], "idle"], t),
            },
        };
        // 걷지 않는 동안 걸음 시계를 세운다(다시 걸으면 첫 자세에서). 움직이는 서기 상태면 — 튕긴 팔·일어나기가 걷기를 가린
        // 동안에도 — 시계는 그대로 둔다(예전 그대로).
        if !(idle && speed > 0.5) {
            self.gait.advance(wdt, false, 0.0);
        }
        Pick {
            looped: idle,
            ..pick
        }
    }
}

/// 걷는 동작 고르기(0034 → 0235): 몸이 걷는 쪽이 아닌 것을 볼 때(락온 · 쥔 조준 · 막기 — `Player::faces_away`)는 걷는 쪽이 옆이면 게걸음(`strafe_l`·`strafe_r`), 뒤면 뒷걸음.
/// 아니면 늘 앞으로 걷는다(몸이 걷는 쪽을 본다). 없는 모델이면 `walk`로. 수류탄을 겨눈 채 뒤로 걸으면 앞으로 걷는 동작이
/// 뒤로 미끄러졌다(사장 "수류탄 조준 중에 뒤로 걷는 모션 이상하다", 0235) — 락온만 보던 것을 `faces_away`로.
pub fn move_anim(step: Vec3, facing: Vec3, faces_away: bool) -> (&'static [&'static str], f32) {
    if !faces_away {
        return (&["walk"], 0.0);
    }
    let v = vec3(step.x, 0.0, step.z);
    let fwd = v.dot(facing);
    // 오른쪽 = 앞 × 위(카메라·입력과 같은 축). 가는 쪽이 바라보는 쪽에서 오른쪽으로 몇 도인가.
    let side = v.dot(facing.cross(Vec3::Y));
    let rho = side.atan2(fwd);
    // 0197: 게걸음 동작이 없다 — 앞으로 걷는 동작의 다리(엉덩이)를 가는 쪽으로 돌리고 윗몸은 과녁에 둔다(`LEG_TURN_MAX`까지).
    // 뒤쪽 반은 뒷걸음의 다리를 가는 쪽의 반대로. 모델 공간의 돌림(+ = 왼쪽) = −(오른쪽으로 간 각).
    if rho.abs() <= BACK_FROM.to_radians() {
        (&["walk"], -rho.clamp(-LEG_TURN_MAX, LEG_TURN_MAX))
    } else {
        let back = (-side).atan2(-fwd);
        (
            &["walk_back", "walk"],
            -back.clamp(-LEG_TURN_MAX, LEG_TURN_MAX),
        )
    }
}

/// 락온 중 이 각(도)보다 뒤로 가면 뒷걸음(0197).
const BACK_FROM: f32 = 110.0;
/// 다리(엉덩이)를 가는 쪽으로 돌리는 가장 큰 각(라디안, 0197) — 넘으면 윗몸이 너무 비틀린다.
pub const LEG_TURN_MAX: f32 = 1.4;
/// 뒷걸음의 걸음 시계 배(0197 → 0198): 싸움 자세 뒷걸음(Walk_Fight_Back)은 보폭이 작고 주기가 길어(발 빠르기 0.32 vs 걷기 0.90),
/// 같은 빠르기로 돌리면 발이 땅을 미끄러진다 — 발이 가는 빠르기를 걷기와 맞춘 배(시험 `walk_back_steps_keep_pace_with_the_ground`).
pub const WALK_BACK_RATE: f32 = 2.8;
/// 이만큼(안쪽 곱)보다 뒤로 피하면 뒤로 뛰기(0197 — 몸이 보는 쪽의 반대로 가는 회피: 입력 없이 누름·락온 뒷걸음).
pub const BACKSTEP_DOT: f32 = -0.5;
/// 쇠뇌를 쏜 뒤 서 있으면 이만큼(초) 겨눈 채 머문다(0197 — 쏠 때마다 정면 ↔ 옆으로 홱 돌지 않게, 끝나면 `AIM_BLEND`로 천천히 내린다).
pub const AIM_HOLD: f32 = 0.7;
/// 들이받기(0276)가 트는 방패 돌진 동작(`sk_ram` — 0.15초에 당겨 든 자리)의 시작 시각.
pub const RUSH_CLIP_FROM: f32 = 0.15;
/// 겨눈 채 머무는 자세의 시각(쏘기 동작의 끝 바로 앞).
pub const XBOW_HOLD_AT: f32 = 0.235;

/// 스킬 동작의 이어짐 구간을 되풀이하는 주기(0158 G2): 휘돌이는 바퀴마다, 가로지르기는 길마다(되돌아 베기의 둘째 길).
/// 나머지 모양은 이어짐 구간 전체에 늘인다(와일드카드 없이 — 0278 R14).
fn clip_period(form: &crate::skills::Form) -> Option<f32> {
    use crate::skills::Form;
    match *form {
        Form::Ring { gap, .. } => Some(gap),
        Form::Lane { time, .. } => Some(time),
        // 제비걸음(0214): 걸음마다 찌름을 되풀이한다.
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

/// 점프 동작의 길이(초, `wanderer_skin.py` — 뜨는 순간부터 닿는 순간까지).
pub const JUMP_CLIP: f32 = 0.5;
/// 닿은 뒤 무릎을 받는 동작의 길이(초) — 착지 상태의 길이에 맞춰 돈다.
pub const LAND_CLIP: f32 = 0.15;
/// 내려찍기 동작에서 칼이 땅에 닿는 자리(초).
pub const PLUNGE_STRIKE: f32 = 0.29;
/// 내려찍고 내려앉은 뒤 웅크렸다 일어나는 동작의 길이(초, 0202) — 착지 상태(`plunge.land s`)에 맞춰 돈다.
pub const PLUNGE_LAND_CLIP: f32 = 0.35;
/// 패링 성공의 밀쳐내기 길이(초, 0199).
pub const PARRY_PUSH_CLIP: f32 = 0.4;
/// 튕긴 뒤 이만큼(초)은 걸어도 거두는 팔을 보인다(0202) — 그 뒤로는 서 있을 때만.
pub const PARRY_PUSH_FIRM: f32 = 0.2;
/// 쇠뇌 쏘기·마법 동작의 놓는 자리(초) — 떠 있는 동안 쏠 때 여기서부터.
pub const XBOW_RELEASE: f32 = 0.058;
/// 던지기(0274 ②b — `sk_throw`): 손을 떠나는 자리(초)와 모으는 동안 머무는 든 자세(초). 활 당기기 동작의 길이(초 — `bow_draw`).
pub const TOSS_RELEASE: f32 = 0.2;
/// 땅 내려찍기(0274 ④ — `gs5`): 날이 땅에 닿는 자리(초 — 판정 구간의 시작)와 모으는 동안 머무는 든 자세(초).
pub const SLAM_RELEASE: f32 = 0.128;
pub const SLAM_WINDUP: f32 = 0.09;
pub const TOSS_WINDUP: f32 = 0.12;
pub const BOW_DRAW_CLIP: f32 = 0.3;
pub const CAST_RELEASE: f32 = 0.075;

/// 점프 동작의 자리(0..1, 0196): 오르는 동안 0 → 0.5(꼭대기), 내려오는 동안 0.5 → 1(처음 위 속도만큼 빨라지면 끝 — 다리를 내린 자세).
pub fn jump_phase(vy: f32, v0: f32) -> f32 {
    let v0 = v0.max(1e-3);
    if vy >= 0.0 {
        0.5 * (1.0 - vy / v0)
    } else {
        0.5 + 0.5 * (-vy / v0)
    }
    .clamp(0.0, 1.0)
}

/// 내려찍기 동작의 자리(0..1, 0196): 점프 높이에서 0(칼을 든다) → 땅에서 1(꽂는다).
pub fn plunge_phase(air: f32, height: f32) -> f32 {
    (1.0 - air / height.max(1e-3)).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests;
