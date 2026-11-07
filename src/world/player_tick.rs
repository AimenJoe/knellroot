//! 플레이어 틱(0103 T0 — 0066 R6, nail 0170 R9): 한 함수였던 `tick_player`(925줄)를 단계와 상태마다 메서드로 나눴다.
//!
//! 순서는 그대로다: 캔슬·받아 두기 → 입력 기준·조준(`Ctx`) → 시계·스태미나 → 땅 돌리기 → **상태마다 `tick_*`** → 달리기
//! → 이동 → 바라보기 → 떨어지기 → 마무리. 새 동작(나무 칸, 0102)은 그 상태의 메서드에 한 줄씩 넣는다.
//! 상태가 서로 쓰는 시작(회피·패링)은 `try_dodge`·`try_parry` 한 곳.
use super::*;

/// 이번 틱 플레이어 단계가 같이 보는 것 — 입력 기준·조준을 한 번 계산해 상태 메서드들이 나눠 쓴다.
pub(super) struct Ctx<'a> {
    /// 캔슬·받아 둔 누름이 반영된 입력(0057).
    pub a: Actions,
    pub t: &'a Tuning,
    pub dt: f32,
    /// 락온 대상(살아 있을 때).
    pub target: Option<Vec3>,
    pub basis: Basis,
    /// 조준 스틱·커서 쪽(락온이면 없음, 0014·0058).
    pub aim: Option<Vec3>,
    /// 공격·패링·내려찍기의 자동 조준(0014).
    pub auto_dir: Option<Vec3>,
    /// 마법·사격 쏘는 쪽(0048).
    pub cast_dir: Vec3,
    pub wpn: &'static weapons::Weapon,
    /// 수류탄을 떨굴 자리(0051)와 꾹 쥐어 겨눌 때의 커서 자리(0055·0058).
    pub throw_to: Option<Vec3>,
    pub hold_throw_to: Option<Vec3>,
    /// 이동 입력을 월드로(시계 뒤에 계산 — 처음 코드와 같은 자리).
    pub world_move: Vec3,
    /// 원거리 강공격을 모을 수 있나(0056).
    pub can_charge_cast: bool,
    /// 이번 틱에 나간 탄(쪽, 모은 정도)·던진 자리·내려찍기 착지 — 마무리에서 월드로.
    pub fire: Option<(Vec3, u8)>,
    pub thrown: Option<Vec3>,
    pub landed_plunge: bool,
}

impl Player {
    /// 행동은 스태미나가 조금이라도 있으면 된다(소울류처럼 0까지 쓰고 끝). 없으면 막히고 막대가 깜빡인다.
    pub(super) fn spend(&mut self, cost: f32) -> bool {
        if self.stamina > 0.0 {
            self.stamina = (self.stamina - cost).max(0.0);
            self.stamina_idle = 0.0;
            true
        } else {
            // 막힘 알림(0088): 막대 깜빡·떨림 + 머리 위 땀방울 + "안 됨" 소리. 이미 알리는 중이면(받아 둔 누름이 틱마다 다시 막힘) 새로 시작하지 않는다.
            if self.stamina_denied <= 0.0 {
                self.stamina_denied = STAMINA_DENIED_SHOW;
                self.denied_new = true;
            }
            false
        }
    }
}

impl World {
    pub(super) fn tick_player(&mut self, a: &Actions, t: &Tuning, dt: f32) {
        // 캔슬(0057): 누름은 미리 받아 두고, 지금 동작의 캔슬 시점이 오면 후딜레이를 끊고 그 행동으로 넘어간다.
        let cancelled = self.cancel_or_buffer(a, t, dt);
        let started_from = std::mem::discriminant(&self.player.state);
        let was_dodging = matches!(self.player.state, PState::Dodge { .. });
        let mut cx = self.player_ctx(cancelled, t, dt);
        self.tick_player_clocks(&mut cx);
        self.try_turn(&cx);

        // 상태 전이.
        match self.player.state {
            PState::Idle => self.tick_idle(&cx),
            PState::Drink { t: e, gulped } => self.tick_drink(&cx, e, gulped),
            PState::Hurt { t: e, dur } => {
                self.player.state = if e + dt >= dur {
                    PState::Idle
                } else {
                    PState::Hurt { t: e + dt, dur }
                };
            }
            PState::Attack {
                t: e,
                swing,
                dir,
                hit,
            } => self.tick_attack(&cx, e, swing, dir, hit),
            PState::Charging { t: e } => self.tick_charging(&cx, e),
            PState::Heavy {
                t: e,
                swing,
                dir,
                full,
                perfect,
            } => self.tick_heavy(&cx, e, swing, dir, full, perfect),
            PState::Cast { t: e, dir, fired } => self.tick_cast(&mut cx, e, dir, fired),
            PState::CastCharge { t: e } => self.tick_cast_charge(&mut cx, e),
            PState::Throw { t: e, to, thrown } => self.tick_throw(&mut cx, e, to, thrown),
            PState::Dodge { t: e, dir } => self.tick_dodge(&cx, e, dir),
            PState::Jump { t: e, plunge } => self.tick_jump(&mut cx, e, plunge),
            PState::Land { t: e, dur } => {
                self.player.state = if e + dt >= dur {
                    PState::Idle
                } else {
                    PState::Land { t: e + dt, dur }
                };
            }
            PState::Parry { t: e } => {
                self.player.state = if e + dt >= t[P::ParryTime] {
                    PState::Idle
                } else {
                    PState::Parry { t: e + dt }
                };
            }
            PState::Turn { t: e, from, to } => self.tick_turning(&cx, e, from, to),
        }

        self.tick_run(&cx);
        self.move_player(&cx);
        self.face_player(&cx);
        self.fall_player(&cx);
        self.finish_player_tick(cx, started_from, was_dodging);
    }

    /// 입력 기준·조준·자동 조준·쏘는 쪽·떨굴 자리(0010·0014·0048·0051·0055·0058).
    fn player_ctx<'a>(&mut self, a: Actions, t: &'a Tuning, dt: f32) -> Ctx<'a> {
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
        Ctx {
            a,
            t,
            dt,
            target,
            basis,
            aim,
            auto_dir,
            cast_dir,
            wpn,
            throw_to,
            hold_throw_to,
            world_move: Vec3::ZERO,
            can_charge_cast: false,
            fire: None,
            thrown: None,
            landed_plunge: false,
        }
    }

    /// 시계·스태미나 차기·누르고 있는 시간. 끝에서 `world_move`·`can_charge_cast`를 채운다(장전이 끝난 뒤의 탄으로).
    fn tick_player_clocks(&mut self, cx: &mut Ctx) {
        let (a, t, dt, wpn) = (&cx.a, cx.t, cx.dt, cx.wpn);
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
        cx.can_charge_cast = match wpn.rb {
            Rb::Magic(_) => pl.mana >= 1.0,
            Rb::Shot { .. } => pl.ammo >= 1,
        };
        pl.since_spill = (pl.since_spill + dt).min(f32::MAX);
        cx.world_move = move_to_world(a.move_dir, cx.basis, pl.pos);
    }

    /// 땅 돌리기(0012·0023): 쿼터뷰에서만, 서 있거나 스윙 후딜레이(회피와 같은 캔슬 자리)에서. 오른쪽 버튼이 왼쪽보다 먼저.
    fn try_turn(&mut self, cx: &Ctx) {
        let (a, t) = (&cx.a, cx.t);
        let pl = &mut self.player;
        let turn = (a.turn_right || a.turn_left).then_some(a.turn_right);
        let can_turn = cx.target.is_none()
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
    }

    /// 회피 시작(0029): 눌렀고·대기가 끝났고·스태미나를 쓸 수 있으면. 쪽은 이동 입력, 없으면 뒤로.
    /// 상태마다 다른 조건(삼키기 전·쏜 뒤·후딜레이 …)은 부르는 쪽이 `&&`로 앞에 둔다.
    fn try_dodge(&mut self, cx: &Ctx) -> bool {
        let pl = &mut self.player;
        if !(cx.a.dodge && pl.dodge_cd <= 0.0 && pl.spend(cx.t[P::StaminaDodge])) {
            return false;
        }
        let dir = if cx.world_move.length_squared() > 1e-4 {
            cx.world_move.normalize()
        } else {
            -pl.facing_dir()
        };
        pl.state = PState::Dodge { t: 0.0, dir };
        pl.dodge_cd = cx.t[P::DodgeTime] + 0.15;
        true
    }

    /// 패링 시작(0025): 가까운 적 쪽으로 돌아서 막는다(공격 자동 조준과 같은 고르기).
    fn try_parry(&mut self, cx: &Ctx) -> bool {
        let pl = &mut self.player;
        if !(cx.a.parry && pl.spend(cx.t[P::ParryStamina])) {
            return false;
        }
        if let Some(d) = cx.auto_dir.or(cx.aim) {
            pl.facing = d.x.atan2(d.z);
        }
        pl.state = PState::Parry { t: 0.0 };
        true
    }

    fn tick_idle(&mut self, cx: &Ctx) {
        let (t, wpn) = (cx.t, cx.wpn);
        if self.try_dodge(cx) {
            return;
        }
        let a = &cx.a;
        if a.jump && self.player.spend(t[P::JumpStamina]) {
            // 점프(0027): 뛰기 전 속도를 그대로 잇는다(달리다 뛰면 멀리).
            let pl = &mut self.player;
            pl.state = PState::Jump {
                t: 0.0,
                plunge: false,
            };
            pl.vy = crate::tuning::jump_ballistics(t).0;
            pl.air = 0.0;
        } else if self.try_parry(cx) {
        } else if a.attack && !a.dodge && wpn.grenade.is_some() {
            self.throw_grenade(cx);
        } else if a.attack && !a.dodge && self.swing_pays(cx) {
            self.start_swing(cx);
        } else if a.cast && matches!(wpn.rb, Rb::Shot { .. }) {
            self.start_shot(cx);
        } else if a.cast {
            let pl = &mut self.player;
            if pl.mana >= 1.0 {
                pl.mana -= 1.0;
                pl.facing = cx.cast_dir.x.atan2(cx.cast_dir.z);
                pl.state = PState::Cast {
                    t: 0.0,
                    dir: cx.cast_dir,
                    fired: false,
                };
            } else {
                pl.cast_denied = 0.6;
                self.cast_denied_now = true;
            }
        } else if a.flask && self.player.flask_lock <= 0.0 {
            // 등잔(0031): 가득이거나 0이면 몸짓 없이 깜빡만(빈 병 몸짓으로 묶지 않는다 — 급할 때 연타해도).
            let pl = &mut self.player;
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

    /// 원거리 무기의 X = 수류탄(0048·0051): 남아 있으면 던지고, 없으면 칸이 깜빡인다.
    fn throw_grenade(&mut self, cx: &Ctx) {
        let pl = &mut self.player;
        if pl.grenades >= 1.0 {
            pl.grenades -= 1.0;
            let to = cx.throw_to.unwrap_or(pl.pos);
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
    }

    /// 평타를 낼 값을 치른다: 박자에 맞춘 이어지는 타는 스태미나를 쓰지 않는다(2026-11-07, 사장). 대시 공격·패링 반격은 늘 쓴다.
    /// 박자를 놓친 타·새 콤보면 잇단 수는 0(놓침은 벌하지 않는다 — 소리 없음, 수만 0, KB).
    fn swing_pays(&mut self, cx: &Ctx) -> bool {
        let wpn = cx.wpn;
        let pl = &mut self.player;
        let riposte = pl.since_parry <= RIPOSTE_WINDOW && wpn.heavy.is_some();
        let dash = pl.dash_left > 0.0 && wpn.dash.is_some();
        let free = pl.on_beat && pl.combo_left > 0.0 && !riposte && !dash;
        pl.on_beat = false;
        let go = free || pl.spend(cx.t[P::StaminaAttack]);
        if go {
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
    }

    /// 평타·대시 공격·패링 반격 시작(0018·0051·2026-11-06).
    fn start_swing(&mut self, cx: &Ctx) {
        let (t, wpn) = (cx.t, cx.wpn);
        let pl = &mut self.player;
        pl.swings += 1;
        // 휘두르는 쪽을 그 자리에서 정한다: 자동 조준 > 조준 스틱 > 지금 바라보는 쪽.
        if let Some(d) = cx.auto_dir.or(cx.aim) {
            pl.facing = d.x.atan2(d.z);
        }
        // 내딛는 쪽 = 진행 방향(0018): 누른 순간의 이동 입력, 없으면 아직 움직이던 쪽, 서 있으면 바라보는 쪽.
        // 휘두르는 쪽(바라보는 쪽 — 자동 조준·조준 스틱)과는 따로다.
        let moving = vec3(pl.vel.x, 0.0, pl.vel.z);
        let dir = if cx.world_move.length_squared() > 1e-4 {
            cx.world_move.normalize()
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
    }

    /// 원거리 무기의 RB = 사격(0051): 탄창이 비면 장전이 끝날 때까지 못 쏜다.
    fn start_shot(&mut self, cx: &Ctx) {
        let Rb::Shot { reload, time, .. } = cx.wpn.rb else {
            return;
        };
        let pl = &mut self.player;
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
            pl.beat = cx.t[P::CastTime] * time;
            pl.facing = cx.cast_dir.x.atan2(cx.cast_dir.z);
            pl.state = PState::Cast {
                t: 0.0,
                dir: cx.cast_dir,
                fired: false,
            };
        } else {
            pl.cast_denied = 0.6;
            self.cast_denied_now = true;
        }
    }

    /// 등잔 마시기(0031): 다시 누르기는 버린다. 삼키기 전 회피는 끊기(등잔 그대로) — 어깨 버튼 실수는 회피 한 번으로 무른다.
    fn tick_drink(&mut self, cx: &Ctx, e: f32, gulped: bool) {
        if !gulped && self.try_dodge(cx) {
            return;
        }
        let t = cx.t;
        let pl = &mut self.player;
        let e2 = e + cx.dt;
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

    /// 평타 동작: 회피·패링·점프·마법·다음 타는 캔슬(0057, `cancel_or_buffer`)이 맡는다 — 여기서는 모으기와 끝만.
    fn tick_attack(&mut self, cx: &Ctx, e: f32, swing: u32, dir: Vec3, hit: u8) {
        let (a, t, wpn) = (&cx.a, cx.t, cx.wpn);
        let pl = &mut self.player;
        let (s0, a0, r0) = hit_times(t, wpn, hit);
        let total = s0 + a0 + r0;
        let after = e >= s0 + a0;
        let after_hit = wpn.after(hit);
        if a.attack_held && pl.attack_hold >= t[P::HeavyHoldStart] && after && wpn.heavy.is_some() {
            // 강공격(0030): 평타는 누르는 순간 나갔고, 한 번 누른 채 `heavy.hold start s`를 쥐고 있으면(활성이 끝난 뒤) 모으기로.
            // "그 순간 눌려 있나"만 보면 연타 중 휘두르기 도중의 짧은 누름이 활성 끝에 걸려 모으기로 샜다.
            pl.state = PState::Charging { t: 0.0 };
        } else if e + cx.dt >= total {
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
                t: e + cx.dt,
                swing,
                dir,
                hit,
            };
        }
    }

    /// 강공격 모으기(0030·0047): 회피·패링으로 끊고, 쥐면 모으고, 떼면 강타(덜 모았으면 약하게).
    fn tick_charging(&mut self, cx: &Ctx, e: f32) {
        if self.try_dodge(cx) || self.try_parry(cx) {
            return;
        }
        let (a, t, dt, wpn) = (&cx.a, cx.t, cx.dt, cx.wpn);
        if a.attack_held {
            let full = t[P::HeavyCharge] * wpn.heavy.map_or(1.0, |h| h.charge);
            self.charged_now |= e < full && e + dt >= full;
            self.player.state = PState::Charging { t: e + dt };
            return;
        }
        let pl = &mut self.player;
        if pl.spend(t[P::HeavyStamina]) {
            let h = wpn.heavy.expect("charging only with a heavy");
            let full_at = t[P::HeavyCharge] * h.charge;
            // 떼면 강타 — 자동 조준·조준 스틱 쪽으로, 내딛는 쪽은 이동 입력(없으면 바라보는 쪽).
            if let Some(d) = cx.auto_dir.or(cx.aim) {
                pl.facing = d.x.atan2(d.z);
            }
            let dir = if cx.world_move.length_squared() > 1e-4 {
                cx.world_move.normalize()
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

    /// 강타: 활성이 끝난 뒤엔 회피로 끊는다.
    fn tick_heavy(&mut self, cx: &Ctx, e: f32, swing: u32, dir: Vec3, full: bool, perfect: bool) {
        let (s0, a0, r0) = heavy_times(cx.t, cx.wpn);
        let total = s0 + a0 + r0;
        if e >= s0 + a0 && self.try_dodge(cx) {
            return;
        }
        self.player.state = if e + cx.dt >= total {
            PState::Idle
        } else {
            PState::Heavy {
                t: e + cx.dt,
                swing,
                dir,
                full,
                perfect,
            }
        };
    }

    /// 마법·사격(0048·0051): 쏜 뒤엔 회피로 끊고, 쏜 뒤에도 쥐고 있으면 모으기로(0056).
    fn tick_cast(&mut self, cx: &mut Ctx, e: f32, dir: Vec3, fired: bool) {
        if fired && self.try_dodge(cx) {
            return;
        }
        let (a, t, wpn) = (&cx.a, cx.t, cx.wpn);
        let pl = &mut self.player;
        let e2 = e + cx.dt;
        let mut fired = fired;
        let k = match wpn.rb {
            Rb::Shot { time, .. } => time,
            Rb::Magic(_) => 1.0,
        };
        if !fired && e2 >= t[P::CastStartup] * k {
            fired = true;
            cx.fire = Some((dir, 0));
        }
        pl.state =
            if fired && a.cast_held && pl.cast_hold >= t[P::CastHoldStart] && cx.can_charge_cast {
                // 원거리 강공격(0056): 쏜 뒤에도 쥐고 있으면 모으기로(평타 → 강타와 같은 틀, 0030).
                PState::CastCharge { t: 0.0 }
            } else if e2 >= t[P::CastTime] * k {
                PState::Idle
            } else {
                PState::Cast { t: e2, dir, fired }
            };
    }

    /// 원거리 강공격 모으기(0056).
    fn tick_cast_charge(&mut self, cx: &mut Ctx, e: f32) {
        if self.try_dodge(cx) {
            return;
        }
        let (a, t, dt, wpn) = (&cx.a, cx.t, cx.dt, cx.wpn);
        let full = t[P::CastCharge];
        if a.cast_held && cx.can_charge_cast {
            self.charged_now |= e < full && e + dt >= full;
            self.player.state = PState::CastCharge { t: e + dt };
            return;
        }
        let pl = &mut self.player;
        if !cx.can_charge_cast {
            pl.state = PState::Idle;
            return;
        }
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
        pl.facing = cx.cast_dir.x.atan2(cx.cast_dir.z);
        // 모으기 공격의 패링(0089): 원거리 무기의 모은 사격도 떼는 순간부터 패링 창(근접 무기의 RB 마법은 무기의 공격이 아니라 뺀다, 0077·0078과 같은 경계).
        if matches!(wpn.rb, Rb::Shot { .. }) {
            pl.release_guard = t[P::ParryWindow];
        }
        // 딱 맞춰 떼기(0078): 원거리 무기의 모으기 공격(사격 꾹)도 다 모인 뒤 창 안에 떼면 덤 — 근접 무기의 RB 마법은 무기의
        // 공격이 아니라 빼 둔다(박자와 같은 경계, 0077).
        let perfect = matches!(wpn.rb, Rb::Shot { .. }) && e >= full && e <= full + PERFECT_RELEASE;
        cx.fire = Some((
            cx.cast_dir,
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
            dir: cx.cast_dir,
            fired: true,
        };
    }

    /// 수류탄 던지기(0051·0055): 손을 떠난 뒤엔 회피로 끊고, 손을 떠나기 전 꾹 쥐면 겨누기.
    fn tick_throw(&mut self, cx: &mut Ctx, e: f32, to: Vec3, done: bool) {
        if done && self.try_dodge(cx) {
            return;
        }
        let (a, t, wpn) = (&cx.a, cx.t, cx.wpn);
        let pl = &mut self.player;
        if !done && e + cx.dt >= t[P::ThrowStartup] && a.attack_held {
            // 꾹 쥐면 겨누기(0055): 손을 든 채 멈추고, 떨굴 자리 = 조준 스틱·이동 쪽(없으면 바라보는 쪽) 가장 멀리.
            // 떼면 그 자리로. 톡(손을 떠나기 전에 뗌)은 자동 조준 자리(0051).
            let dir = cx
                .aim
                .or((cx.world_move.length_squared() > 1e-4).then(|| cx.world_move.normalize()))
                .unwrap_or_else(|| pl.facing_dir());
            pl.facing = dir.x.atan2(dir.z);
            let far = wpn.grenade.map_or(8.0, |g| g.far);
            pl.state = PState::Throw {
                t: t[P::ThrowStartup] - 1e-4,
                to: cx.hold_throw_to.unwrap_or(pl.pos + dir * far),
                thrown: false,
            };
            return;
        }
        let e2 = e + cx.dt;
        let mut done = done;
        if !done && e2 >= t[P::ThrowStartup] {
            done = true;
            cx.thrown = Some(to);
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

    /// 회피(0029·0076): 끝나면 대시 공격 늦은 창, 아직 쥐고 있으면 달리기.
    fn tick_dodge(&mut self, cx: &Ctx, e: f32, dir: Vec3) {
        let pl = &mut self.player;
        if e + cx.dt >= cx.t[P::DodgeTime] {
            pl.state = PState::Idle;
            // 회피가 끝난 직후의 공격도 대시 공격(늦게 누른 것도 받아 준다).
            pl.dash_left = DASH_ATTACK_LATE;
            // 회피는 누르는 순간 그대로, 끝났을 때 아직 쥐고 있으면 달리기(0076, KB nail 0209) — 톡 누름은 회피보다 짧아
            // 예전과 같다. 회피의 기세를 그대로 잇는다(속도를 0으로 떨어뜨리지 않는다).
            pl.running = cx.a.dodge_held && pl.stamina > 0.0;
        } else {
            pl.state = PState::Dodge { t: e + cx.dt, dir };
        }
    }

    /// 점프(0027): 공중에서 누른 공격 = 내려찍기, 포물선 그대로, 내려앉으면 `Land`.
    fn tick_jump(&mut self, cx: &mut Ctx, e: f32, plunge: bool) {
        let (t, dt) = (cx.t, cx.dt);
        let pl = &mut self.player;
        let mut plunge = plunge;
        if cx.a.attack && !plunge && pl.spend(t[P::StaminaAttack]) {
            plunge = true;
            if let Some(d) = cx.auto_dir.or(cx.aim) {
                pl.facing = d.x.atan2(d.z);
            }
        }
        // 포물선을 그대로(틱마다 위치 = v·dt − g·dt²/2) — 앞 오일러 적분은 높이가 6% 모자랐다.
        let g = crate::tuning::jump_ballistics(t).1 * if plunge { PLUNGE_FALL } else { 1.0 };
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
            cx.landed_plunge = plunge;
        } else {
            pl.state = PState::Jump { t: e + dt, plunge };
        }
    }

    /// 땅 돌리기 동작(0034): 천천히-빠르게-천천히 — 낮췄다가 휙 돌고 선다(동작 `turn_r`·`turn_l`과 같은 박자).
    fn tick_turning(&mut self, cx: &Ctx, e: f32, from: f32, to: f32) {
        let t = cx.t;
        let pl = &mut self.player;
        let e2 = e + cx.dt;
        let k = (e2 / t[P::TurnPlant].max(1e-3)).clamp(0.0, 1.0);
        let k = k * k * (3.0 - 2.0 * k);
        pl.facing = from + angle_diff(to, from) * k;
        if e2 >= t[P::TurnPlant] {
            pl.facing = to;
            pl.state = PState::Idle;
        } else {
            pl.state = PState::Turn { t: e2, from, to };
        }
    }

    /// 달리기(0076): 쥔 동안·스틱을 기울인 동안·스태미나가 있는 동안·서 있는(걷는) 동안만. 달리는 동안은 스태미나가 차지 않고
    /// 공격하면 대시 공격(회피에서 나온 것과 같은 한 타 — 소울류 달리며 치기).
    fn tick_run(&mut self, cx: &Ctx) {
        let pl = &mut self.player;
        if !pl.running {
            return;
        }
        let keep = pl.state == PState::Idle
            && cx.a.dodge_held
            && cx.world_move.length_squared() > 0.04
            && pl.stamina > 0.0;
        if keep {
            pl.stamina = (pl.stamina - RUN_STAMINA * cx.dt).max(0.0);
            pl.stamina_idle = 0.0;
            pl.dash_left = pl.dash_left.max(DASH_ATTACK_LATE);
        } else {
            pl.running = false;
        }
    }

    /// 이동: 상태마다 속도를 정하고 한 번 옮긴다.
    fn move_player(&mut self, cx: &Ctx) {
        let (t, dt, wpn, world_move) = (cx.t, cx.dt, cx.wpn, cx.world_move);
        let pl = &mut self.player;
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
    }

    /// 바라보기: 락온이면 대상, 조준 스틱을 쥐면 그 쪽(움직이며 다른 쪽을 볼 수 있다, 0014), 아니면 이동 방향. 스윙 중엔 고정.
    fn face_player(&mut self, cx: &Ctx) {
        let pl = &mut self.player;
        let want_yaw = match (cx.basis, pl.state) {
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
            _ if cx.aim.is_some() => cx.aim.map(|d| d.x.atan2(d.z)),
            (Basis::Quarter { .. } | Basis::Frozen(_), _)
                if cx.world_move.length_squared() > 1e-4 =>
            {
                Some(cx.world_move.x.atan2(cx.world_move.z))
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
            let max = cx.t[P::TurnSpeed].to_radians() * cx.dt;
            pl.facing += diff.clamp(-max, max);
        }
    }

    /// 점프가 아닌 동작(회피·공격 …) 중에 턱에서 떨어졌다(0043): 동작은 그대로, 몸만 떨어진다.
    fn fall_player(&mut self, cx: &Ctx) {
        let pl = &mut self.player;
        if !matches!(pl.state, PState::Jump { .. }) && pl.air > 0.0 {
            let g = crate::tuning::jump_ballistics(cx.t).1;
            let dt = cx.dt;
            pl.air += pl.vy * dt - 0.5 * g * dt * dt;
            pl.vy -= g * dt;
            if pl.air <= 0.0 {
                pl.air = 0.0;
                pl.vy = 0.0;
            }
        }
    }

    /// 마무리: 이번 틱에 생긴 것을 월드로, 받아 둔 누름을 지우고, 원거리 무기의 회피 장전.
    fn finish_player_tick(
        &mut self,
        cx: Ctx,
        started_from: std::mem::Discriminant<PState>,
        was_dodging: bool,
    ) {
        self.plunge_now |= cx.landed_plunge;
        // 받아 둔 행동이 쓰였다(새 동작이 시작됐다) — 끝나서 서게 된 것(`Idle`)이면 남겨 둔다(다음 틱에 나간다).
        let now = std::mem::discriminant(&self.player.state);
        if now != started_from && !matches!(self.player.state, PState::Idle) {
            self.player.pending = None;
        }
        if cx.fire.is_some() {
            self.cast_fire = cx.fire;
        }
        if cx.thrown.is_some() {
            self.throw_to = cx.thrown;
        }
        // 원거리 무기는 회피하면 한 발 장전(0051, PoE2 착상) — 회피를 공격 리듬에 넣는다.
        if !was_dodging && matches!(self.player.state, PState::Dodge { .. }) {
            if let Rb::Shot { mag, .. } = cx.wpn.rb {
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
}
