//! 플레이어 틱(0103 T0 — 0066 R6, nail 0170 R9): 한 함수였던 `tick_player`(925줄)를 단계와 상태마다 메서드로 나눴다.
//!
//! 순서는 그대로다: 캔슬·받아 두기 → 입력 기준·조준(`Ctx`) → 시계·스태미나 → 땅 돌리기 → **상태마다 `tick_*`** → 달리기
//! → 이동 → 바라보기 → 떨어지기 → 마무리. 새 동작(나무 칸, 0102)은 그 상태의 메서드에 한 줄씩 넣는다.
//! 상태가 서로 쓰는 시작(회피·패링)은 `try_dodge`·`try_parry` 한 곳.
use super::*;
use crate::progression::{cell, Cell, Link};

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
    /// 숄더뷰의 크로스헤어 쪽(0230 — 크로스헤어의 적 `aim_sight` > 카메라 앞, 패드면 보조까지): 한 대상을 겨누는 스킬
    /// (`Form::sighted`)이 쓴다. 숄더뷰가 아니거나 손가락으로 끈 쪽이 있으면 없음.
    pub sight: Option<Vec3>,
    /// 공격·패링·내려찍기의 자동 조준(0014).
    pub auto_dir: Option<Vec3>,
    /// 마법·사격 쏘는 쪽(0048).
    pub cast_dir: Vec3,
    pub wpn: &'static weapons::Weapon,
    /// 수류탄을 떨굴 자리(0051)와 꾹 쥐어 겨눌 때의 커서 자리(0055·0058).
    pub throw_to: Option<Vec3>,
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
        // 던진 창(0274 ②b): 평타를 누르면 곧바로 손에 돌아온다 — 창이 없는 동안에도 평타는 막히지 않는다.
        if cx.a.attack {
            self.recall_javelin();
        }

        // 교체 공격(0272 1b): 이 틱에 바꿔 든 무기가 들어오며 친다(`weapon_swap`이 세웠다) — 다른 누름보다 먼저. 이 틱에 못 나가면 버린다.
        let entry = std::mem::take(&mut self.player.entry);
        // 상태 전이.
        match self.player.state {
            PState::Idle if entry => self.start_entry(&cx),
            PState::Idle => self.tick_idle(&cx),
            PState::Drink { t: e, gulped } => self.tick_drink(&cx, e, gulped),
            PState::Hurt { t: e, dur } => {
                self.player.state = if e + dt >= dur {
                    PState::Idle
                } else {
                    PState::Hurt { t: e + dt, dur }
                };
            }
            PState::Recoil { t: e } => {
                self.player.state = if e + dt >= crate::tuning::RECOIL_TIME {
                    PState::Idle
                } else {
                    PState::Recoil { t: e + dt }
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
            PState::Land { t: e, dur, plunge } => {
                self.player.state = if e + dt >= dur {
                    PState::Idle
                } else {
                    PState::Land {
                        t: e + dt,
                        dur,
                        plunge,
                    }
                };
            }
            PState::Block { t: e } => self.tick_block(&cx, e),
            PState::Rush {
                t: e,
                dir,
                power,
                swing,
            } => self.tick_rush(&cx, e, dir, power, swing),
            PState::Parry { t: e } => {
                // 방패(0223): 패링 창이 지나도 쥐고 있으면 막기로 잇는다 — 헛친 패링의 굳음 대신 방패 뒤에 선다.
                self.player.state =
                    if e + dt >= t[P::ParryWindow] && cx.a.parry_held && self.can_block() {
                        PState::Block { t: 0.0 }
                    } else if e + dt >= t[P::ParryTime] {
                        PState::Idle
                    } else {
                        PState::Parry { t: e + dt }
                    };
            }
            PState::Turn { t: e, from, to } => self.tick_turning(&cx, e, from, to),
            PState::Skill {
                id,
                t: e,
                dir,
                swing,
            } => self.tick_skill(&cx, id, e, dir, swing),
            PState::Ult { kind, t: e, done } => self.tick_ult(&cx, kind, e, done),
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
        // 터치로 끌어 겨눈 것(0217)은 락온보다 앞선다 — 손가락이 가리킨 쪽이 그 순간의 뜻이다.
        let dragged = a.aim_drag && a.aim.length() > AIM_STICK;
        let aim = if dragged {
            let w = move_to_world(a.aim.normalize(), basis, self.player.pos);
            (w.length_squared() > 1e-6).then(|| w.normalize())
        } else if target.is_some() {
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
        // 원거리가 겨누는 쪽(0219): 손이 겨눈 쪽(끌기·스틱·커서) > 숄더뷰의 카메라 앞(`aim_ahead` — 근접 자동 조준은 그대로라
        // `aim`에 섞지 않는다). 패드면 그 쪽 ± `aim_assist`° 안의 적에게 붙인다(엄지의 몫 — 끌기·커서는 붙이지 않는다).
        // 크로스헤어가 적을 겨누면(0222 → 0227 `aim_sight`) 그 적 쪽 — 원거리의 쪽만이고 몸·근접은 모른다.
        let sight = a
            .aim_sight
            .map(|q| vec3(q.x - self.player.pos.x, 0.0, q.y - self.player.pos.z))
            .filter(|d| d.length() > CURSOR_DEAD_M)
            .map(|d| d.normalize());
        let ahead = (a.aim_ahead && target.is_none())
            .then(|| sight.unwrap_or_else(|| move_to_world(vec2(0.0, 1.0), basis, self.player.pos)))
            .filter(|w| w.length_squared() > 1e-6)
            .map(|w| w.normalize());
        let ranged_aim = aim.or(ahead).map(|d| {
            if a.aim_assist > 0.0 && !dragged && aim_point.is_none() && sight.is_none() {
                self.assist_aim(d, a.aim_assist, t)
            } else {
                d
            }
        });
        // 0221의 늘 떠 있는 바닥 헤어라인은 걷었다(0222 — 사장 "조준 전 바닥의 헤어라인 필요없음"): 손이 겨눌 때만.
        self.aim_line = ranged_aim;
        let sight = ranged_aim.filter(|_| a.aim_ahead && !dragged && target.is_none());
        // 마법 쏘는 쪽(0048): 조준 스틱 > 락온 대상 > `cast.range m` 안의 가장 가까운 적 > 바라보는 쪽.
        // 쇠뇌(0213, 사장 "자동 조준 없애고")는 가까운 적을 찾지 않는다 — 조준(스틱·커서) > 락온 > 바라보는 쪽. 겨누는 것이 손의 일.
        let charging_cast = matches!(self.player.state, PState::CastCharge { .. });
        let cast_dir = if a.cast || charging_cast {
            let lock_dir = target.map(|tg| {
                let d = vec3(tg.x - self.player.pos.x, 0.0, tg.z - self.player.pos.z);
                d.normalize_or_zero()
            });
            // 터치의 톡(0217)은 쇠뇌도 가까운 적에게 — 끌 틈이 없는 손의 몫.
            let seek = !matches!(self.player.weapon.def().rb, Rb::Shot { .. }) || a.auto_aim;
            ranged_aim
                .or(lock_dir.filter(|d| d.length_squared() > 0.5))
                .or_else(|| {
                    // 찾는 것은 탄이 닿는 적만(0228 — 벽 너머의 적을 찾아 벽에 쏘지 않는다).
                    seek.then(|| self.nearest_shootable(t[P::CastRange], t))
                        .flatten()
                })
                .unwrap_or_else(|| self.player.facing_dir())
        } else {
            self.player.facing_dir()
        };
        // 수류탄을 떨굴 자리(0051 → 0178): 발 바로 앞 — 조준 스틱 쪽 > 커서 쪽 > 바라보는 쪽. 적을 찾아 멀리 던지지 않는다.
        let wpn = self.player.weapon.def();
        let throw_to = wpn.grenade.map(|g| {
            let p0 = self.player.pos;
            let toward_cursor = aim_point.filter(|_| aim.is_some()).and_then(|q| {
                let d = vec3(q.x - p0.x, 0.0, q.z - p0.z);
                (d.length_squared() > 1e-4).then(|| d.normalize())
            });
            // 터치의 톡(0217)은 가까운 적 쪽으로 떨군다(멀리 던지지는 않는다 — 쪽만).
            let near = a
                .auto_aim
                .then(|| nearest_in_reach(&self.enemies, p0, t[P::CastRange], None));
            let dir = toward_cursor
                .or(ranged_aim)
                .or(near.flatten())
                .unwrap_or_else(|| self.player.facing_dir());
            p0 + dir * g.drop
        });
        Ctx {
            a,
            t,
            dt,
            target,
            basis,
            aim,
            sight,
            auto_dir,
            cast_dir,
            wpn,
            throw_to,
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
        let mag = self.mag_of(self.player.weapon);
        // 던진 창 · 방패(0274 ②b · 0276)가 날고 있나, 방패가 RB를 맡나(방패 던지기는 칸을 쓰지 않고 모으지 않는다), RB 칸의 수.
        let javelin = self.rb_out();
        let shield_rb = self.shield_throws();
        let charges = self.cast_charges();
        // 한 칸이 차는 시간의 배(0274 ⑥ — 완드의 저주탄은 빨리 찬다, 나머지는 1).
        let refill = self.rb_toss().map_or(1.0, |r| r.refill);
        let pl = &mut self.player;
        // 그림자 걸음(0276)의 창.
        pl.off.shadow = (pl.off.shadow - dt).max(0.0);
        // 콤보·원거리 무기 시계(0051).
        pl.combo_left = (pl.combo_left - dt).max(0.0);
        // 뿌리내린 몸(0214): 땅에 서서 움직이지 않은 시간 — 걷거나 구르거나 뜨면 처음부터.
        let still = pl.air <= 0.0 && vec3(pl.vel.x, 0.0, pl.vel.z).length() < 0.3;
        pl.hurt.still = if still { pl.hurt.still + dt } else { 0.0 };
        pl.parry.deflect = (pl.parry.deflect - dt).max(0.0);
        if let Rb::Shot { .. } = wpn.rb {
            if pl.ranged.reload > 0.0 {
                pl.ranged.reload -= dt;
                if pl.ranged.reload <= 0.0 {
                    pl.ranged.reload = 0.0;
                    pl.ranged.ammo = mag;
                }
            }
        }
        // 쇠뇌의 덧장전(0274 ②b — `Rhythm::Crank`): 탄이 남아 있고 쏘지 않고 멈춰 서 있으면 `wait`초 뒤부터 `each`초마다 한 발.
        // 쏘거나 모으거나·구르거나 뛰거나·맞거나·걸으면 처음부터 — 수류탄을 던지는 동안(서 있다)은 이어 간다.
        if let Some(weapons::Rhythm::Crank { wait, each }) = pl.weapon.rhythm() {
            let busy = matches!(
                pl.state,
                PState::Cast { .. }
                    | PState::CastCharge { .. }
                    | PState::Dodge { .. }
                    | PState::Jump { .. }
                    | PState::Hurt { .. }
                    | PState::Recoil { .. }
            );
            if busy || !still || pl.ranged.reload > 0.0 || pl.ranged.ammo >= mag {
                pl.ranged.topup = 0.0;
            } else {
                pl.ranged.topup += dt;
                if pl.ranged.topup >= wait + each {
                    pl.ranged.ammo += 1;
                    pl.ranged.topup = wait;
                }
            }
        }
        if let Some(g) = wpn.grenade {
            pl.ranged.grenades =
                (pl.ranged.grenades + dt / g.cooldown.max(0.1)).min(g.count as f32);
        }
        // 멘 쇠뇌(보조특성, 0272 1b): 등에서도 장전이 돈다 — 손에 든 것과 같은 빠르기로, 다 차면 가득(`Stowed::full`).
        // 다른 무기의 멘 탄창은 그대로 멈춰 있다(0223).
        if let (Some(sub), Some(l)) = (pl.sub, pl.sub_load.as_mut()) {
            if l.reload > 0.0 && sub.sub_trait().is_some_and(|s| s.stowed_reload) {
                l.reload -= dt;
                if l.reload <= 0.0 {
                    l.reload = 0.0;
                    l.full = true;
                }
            }
        }
        // 마법 칸(0111): 수류탄처럼 시간으로 찬다 — 한 칸 `cast.cooldown s`.
        if let Rb::Toss = wpn.rb {
            pl.ranged.mana =
                (pl.ranged.mana + dt / (t[P::CastCooldown] * refill).max(0.1)).min(charges);
        }
        // 공중 RB(0111): 땅에 서면 다음 점프에 다시 쏠 수 있다.
        if !matches!(pl.state, PState::Jump { .. }) {
            pl.ranged.air_shot = false;
            pl.ranged.hover = 0.0;
        }
        pl.hurt.left = (pl.hurt.left - dt).max(0.0);
        pl.ranged.cast_denied = (pl.ranged.cast_denied - dt).max(0.0);
        pl.dodge_cd = (pl.dodge_cd - dt).max(0.0);
        pl.parry.since = (pl.parry.since + dt).min(f32::MAX);
        pl.hurt.since = (pl.hurt.since + dt).min(f32::MAX);
        pl.parry.chain_left = (pl.parry.chain_left - dt).max(0.0);
        if pl.parry.chain_left <= 0.0 {
            pl.parry.chain_n = 0;
        }
        pl.dash_left = (pl.dash_left - dt).max(0.0);
        // 비껴서기(0107·0108): 등 뒤를 잡은 표지는 대시 공격 창이 닫히면 같이 꺼진다.
        if pl.dash_left <= 0.0 {
            pl.flank = false;
        }
        pl.beat.since_swing = (pl.beat.since_swing + dt).min(f32::MAX);
        pl.beat.since = (pl.beat.since + dt).min(f32::MAX);
        pl.beat.since_perfect = (pl.beat.since_perfect + dt).min(f32::MAX);
        // 스태미나: 마지막으로 쓴 뒤 `stamina.regen delay s`가 지나야 찬다.
        pl.stamina_denied = (pl.stamina_denied - dt).max(0.0);
        pl.hurt.stun_window = (pl.hurt.stun_window - dt).max(0.0);
        pl.parry.release_guard = (pl.parry.release_guard - dt).max(0.0);
        pl.stamina_idle += dt;
        if pl.stamina_idle >= t[P::StaminaDelay] {
            pl.stamina = (pl.stamina + t[P::StaminaRegen] * dt).min(t[P::StaminaMax]);
        }
        pl.turn_cd = (pl.turn_cd - dt).max(0.0);
        // 스킬의 대기(0151)는 저마다 돈다.
        for cd in &mut pl.skill.cd {
            *cd = (*cd - dt).max(0.0);
        }
        pl.skill.denied = (pl.skill.denied - dt).max(0.0);
        pl.skill.chord = pl
            .skill
            .chord
            .map(|(s, left)| (s, left - dt))
            .filter(|&(_, left)| left > 0.0);
        pl.skill.stride = (pl.skill.stride - dt).max(0.0);
        pl.skill.mend_rest = (pl.skill.mend_rest - dt).max(0.0);
        // 되울림·몰아치기(0158 G2)의 남은 시간 — 로직 시계라 히트스톱 동안은 줄지 않는다.
        pl.skill.echo_left = (pl.skill.echo_left - dt).max(0.0);
        pl.skill.flurry_left = (pl.skill.flurry_left - dt).max(0.0);
        pl.skill.steady = (pl.skill.steady - dt).max(0.0);
        pl.flask_lock = (pl.flask_lock - dt).max(0.0);
        pl.flask_denied = (pl.flask_denied - dt).max(0.0);
        // 새 누름이면 0부터, 쥐고 있으면 늘고, 놓으면 0.
        pl.attack_hold = if a.attack || !a.attack_held {
            0.0
        } else {
            pl.attack_hold + dt
        };
        pl.ranged.cast_hold = if a.cast || !a.cast_held {
            0.0
        } else {
            pl.ranged.cast_hold + dt
        };
        // 원거리 강공격(0056)을 모을 수 있나 — 떼는 순간 칸(마법)·볼트(쇠뇌) 하나를 쓴다. 던진 창이 날고 있으면 못 한다(0274 ②b).
        cx.can_charge_cast = match wpn.rb {
            Rb::Toss => pl.ranged.mana >= 1.0 && !javelin && !shield_rb,
            Rb::Shot { heavy, .. } => pl.ranged.ammo >= heavy.max(1),
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
    pub(super) fn try_dodge(&mut self, cx: &Ctx) -> bool {
        // 가벼운 구르기(0215 — 멈추지 않는 걸음 II): 회피 기력 ×0.8.
        let cost = cx.t[P::StaminaDodge]
            * if self.tree.picked_b(Cell::UnbrokenStride) {
                cell::LIGHT_ROLL
            } else {
                1.0
            };
        let pl = &mut self.player;
        // 이어지는 숨(0214)·디딘 받기(0215): 기력 없이 나가는 회피 하나.
        let free = pl.beat.free_dodge;
        if !(cx.a.dodge && pl.dodge_cd <= 0.0 && (free || pl.spend(cost))) {
            return false;
        }
        pl.beat.free_dodge = false;
        let dir = if cx.world_move.length_squared() > 1e-4 {
            cx.world_move.normalize()
        } else {
            -pl.facing_dir()
        };
        pl.state = PState::Dodge { t: 0.0, dir };
        pl.dodge_cd = cx.t[P::DodgeTime] + 0.15;
        true
    }
}

/// 막기를 푸는 누름(0223): 막는 동안 이 가운데 하나를 누르면 그 동작이 나간다.
fn block_breaks(a: &Actions) -> bool {
    a.dodge || a.jump || a.attack || a.cast || a.flask || a.skill1 || a.skill2 || a.ult
}

impl World {
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

    /// 방패 막기(0223): 쥔 동안 머문다. 떼거나 다른 것을 누르면 그 틱에 그 동작으로(막다가 치기·피하기가 늦지 않게).
    /// 들이받기(0276 — 하데스의 방패): 막는 동안 힘이 모이고(`OffHand::rush`, `RUSH_FULL`초에 가득), 다른 것을 누르지 않고 패링
    /// 버튼만 놓으면 모인 만큼 앞으로 들이받는다(`RUSH_MIN`보다 덜 모였으면 그냥 내린다 — 톡 막기가 새지 않게). 막기의 기력 셈은 그대로.
    fn tick_block(&mut self, cx: &Ctx, e: f32) {
        if !cx.a.parry_held || block_breaks(&cx.a) || !self.can_block() {
            let power = std::mem::take(&mut self.player.off.rush);
            if !cx.a.parry_held
                && !block_breaks(&cx.a)
                && self.can_block()
                && power >= crate::tuning::RUSH_MIN
                && self.start_rush(cx, power)
            {
                return;
            }
            self.player.state = PState::Idle;
            self.tick_idle(cx);
            return;
        }
        // 막는 동안은 기력이 차지 않는다 — 방패 뒤에 오래 서면 칠 기력이 없다.
        let pl = &mut self.player;
        pl.stamina_idle = 0.0;
        if e <= 0.0 {
            pl.off.rush = 0.0;
        }
        pl.off.rush = (pl.off.rush + cx.dt / crate::tuning::RUSH_FULL).min(1.0);
        pl.state = PState::Block { t: e + cx.dt };
    }

    /// 들이받기 시작(0276): 평타 한 번 값의 기력(없으면 막힌다 — 그냥 내린다). 쪽 = 이동 입력 > 바라보는 쪽(막는 동안 본 쪽).
    fn start_rush(&mut self, cx: &Ctx, power: f32) -> bool {
        let pl = &mut self.player;
        if !pl.spend(cx.t[P::StaminaAttack]) {
            return false;
        }
        let dir = if let Some(d) = cx.auto_dir.or(cx.aim).filter(|_| cx.target.is_none()) {
            d
        } else {
            pl.facing_dir()
        };
        pl.facing = dir.x.atan2(dir.z);
        pl.swings += 1;
        pl.state = PState::Rush {
            t: 0.0,
            dir,
            power,
            swing: pl.swings,
        };
        self.skill_ev.push(Event::Rush {
            at: self.player.pos,
            dir,
            power,
        });
        true
    }

    /// 들이받기: 내달린 뒤 짧게 굳는다(`RUSH_RECOVER`), 그 뒤 선다. 굳는 동안은 회피로 끊는다(`cancel_ready`).
    fn tick_rush(&mut self, cx: &Ctx, e: f32, dir: Vec3, power: f32, swing: u32) {
        let e2 = e + cx.dt;
        self.player.state = if e2 >= crate::tuning::RUSH_TIME + crate::tuning::RUSH_RECOVER {
            PState::Idle
        } else {
            PState::Rush {
                t: e2,
                dir,
                power,
                swing,
            }
        };
    }

    fn tick_idle(&mut self, cx: &Ctx) {
        let (t, wpn) = (cx.t, cx.wpn);
        if self.try_dodge(cx) {
            return;
        }
        let a = &cx.a;
        // 방패(0223): 누르는 것 없이 패링 버튼만 쥐고 있으면 막는다(튕긴 뒤·공격을 마친 뒤에도 쥐고 있으면 다시 방패 뒤로).
        if a.parry_held && !block_breaks(a) && !a.parry && self.can_block() {
            self.player.state = PState::Block { t: 0.0 };
            return;
        }
        if a.jump && self.player.spend(t[P::JumpStamina]) {
            // 점프(0027): 뛰기 전 속도를 그대로 잇는다(달리다 뛰면 멀리).
            let pl = &mut self.player;
            pl.state = PState::Jump {
                t: 0.0,
                plunge: false,
            };
            pl.vy = crate::tuning::jump_ballistics(t).0;
            pl.air = 0.0;
        } else if self.try_parry(cx) || self.try_skill(cx) {
        } else if a.attack && !a.dodge && wpn.grenade.is_some() {
            self.throw_grenade(cx);
        } else if a.cast && self.shield_throws() {
            // 방패 던지기(0276 — 하데스의 방패): 종류의 RB보다 앞선다. 칸을 쓰지 않고 모으지 않는다 — 날아가 돌아올 때까지 다시 못 던진다.
            if self.rb_out() {
                self.player.ranged.cast_denied = 0.6;
                self.cast_denied_now = true;
            } else {
                let pl = &mut self.player;
                pl.facing = cx.cast_dir.x.atan2(cx.cast_dir.z);
                pl.state = PState::Cast {
                    t: 0.0,
                    dir: cx.cast_dir,
                    fired: false,
                };
            }
        } else if a.attack && !a.dodge && self.swing_pays(cx) {
            self.start_swing(cx);
        } else if a.aim_hold
            && a.cast_held
            && self.player.ranged.cast_hold >= t[P::CastHoldStart]
            && cx.can_charge_cast
        {
            // 터치로 끌어 겨누는 동안(0217)은 쏘지 않고 쥐고 있으니, 쥔 채로 모으기에 든다 — 떼는 순간 모은 탄.
            // 활(0274 ②b)은 누르는 순간부터 당기니 같은 길(`start_shot`)로.
            if self.player.weapon.draws() {
                self.start_shot(cx);
            } else {
                let pl = &mut self.player;
                pl.facing = cx.cast_dir.x.atan2(cx.cast_dir.z);
                pl.state = PState::CastCharge { t: 0.0 };
            }
        } else if a.cast && matches!(wpn.rb, Rb::Shot { .. }) {
            self.start_shot(cx);
        } else if a.cast {
            let javelin = self.rb_out();
            let winds = self.player.weapon.winds();
            let pl = &mut self.player;
            if winds && pl.ranged.mana >= 1.0 && !javelin {
                // 창 던지기(0274 ②b): 던지면 손에 창이 없어 "쏜 뒤 쥐어 모으기"를 못 한다 — 누르면 들어 겨누고 떼면 던진다(`tick_wind`).
                pl.facing = cx.cast_dir.x.atan2(cx.cast_dir.z);
                pl.state = PState::CastCharge { t: 0.0 };
            } else if pl.ranged.mana >= 1.0 && !javelin {
                pl.ranged.mana -= 1.0;
                pl.facing = cx.cast_dir.x.atan2(cx.cast_dir.z);
                pl.state = PState::Cast {
                    t: 0.0,
                    dir: cx.cast_dir,
                    fired: false,
                };
            } else {
                pl.ranged.cast_denied = 0.6;
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
        // 수류탄도 기력을 쓴다(0117) — 평타 한 번 값.
        if pl.ranged.grenades >= 1.0 && !pl.spend(cx.t[P::StaminaAttack]) {
            return;
        }
        if pl.ranged.grenades >= 1.0 {
            pl.ranged.grenades -= 1.0;
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
            pl.ranged.cast_denied = 0.6;
            self.cast_denied_now = true;
        }
    }

    /// 평타를 낼 값을 치른다: 박자에 맞춘 이어지는 타는 스태미나를 쓰지 않는다(2026-11-07, 사장). 대시 공격·패링 반격은 늘 쓴다.
    /// 박자를 놓친 타·새 콤보면 잇단 수는 0(놓침은 벌하지 않는다 — 소리 없음, 수만 0, KB).
    fn swing_pays(&mut self, cx: &Ctx) -> bool {
        let wpn = cx.wpn;
        let pl = &mut self.player;
        let riposte = pl.parry.since <= RIPOSTE_WINDOW && wpn.heavy.is_some();
        let dash = pl.dash_left > 0.0 && wpn.dash.is_some();
        // 몰아치기(0158 G2): 그동안의 평타는 모두 기력 없이 나가는 박자 타.
        let flurry = pl.skill.flurry_left > 0.0;
        let free = flurry || (pl.beat.on && pl.combo_left > 0.0 && !riposte && !dash);
        // 받아치는 박(0215 다리): 패링 반격도 박자 타로 친다(값은 그대로 치른다).
        let beat = free || (riposte && self.tree.linked(Link::RiposteBeat));
        pl.beat.on = false;
        let go = free || pl.spend(cx.t[P::StaminaAttack] * wpn.stamina);
        if go {
            pl.beat.swing_beat = beat;
            if beat {
                self.beat_now = true;
                pl.beat.chain += 1;
                pl.beat.since = 0.0;
                let tree = self.tree;
                // 맺음타(0107): 마지막 타 뒤 박자로 이은 이 타는 자세 ×1.5(스윙 번호는 `start_swing`이 하나 올린다).
                if pl.parry.closing_ready {
                    pl.parry.closing_swing = pl.swings + 1;
                }
                // 이어지는 박자(0107): 박자 타 셋마다 둘레 충격.
                if tree.has(Cell::FlowingBeat) && pl.beat.chain.is_multiple_of(cell::FLOW_EVERY) {
                    self.shock_now = true;
                }
                // 이어지는 숨(0214): 박자 셋을 이으면 다음 회피가 기력 없이.
                if tree.has(Cell::HeldBreath) && pl.beat.chain >= cell::HELD_BREATH_CHAIN {
                    pl.beat.free_dodge = true;
                }
                // 끝맺음(0107): 박자를 다섯 이은 뒤 박자로 낸 마지막 타 → 다음 강타는 다 모여 있다.
                let hit = if pl.combo_left > 0.0 {
                    pl.combo_next
                } else {
                    0
                };
                if tree.has(Cell::FinalMeasure)
                    && pl.beat.chain >= cell::FINAL_CHAIN
                    && wpn.after(hit).is_none()
                {
                    pl.parry.primed_heavy = true;
                }
            } else {
                pl.beat.chain = 0;
            }
            pl.parry.closing_ready = false;
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
        let riposte = pl.parry.since <= RIPOSTE_WINDOW && wpn.heavy.is_some();
        let dash = pl.dash_left > 0.0 && wpn.dash.is_some();
        pl.dash_left = 0.0;
        if riposte {
            pl.beat.period = 0.0;
            // 패링 반격(2026-11-06): 튕긴 직후의 공격은 모으기 없이 다 모은 강타 — 드러난 적에게 치명타가 더해진다.
            // 반격을 고르면 방어를 내려놓는다 — 이어 패링도 닫힌다(0081).
            pl.parry.since = f32::MAX;
            pl.parry.chain_left = 0.0;
            pl.parry.chain_n = 0;
            // 되돌려 치기(0107): 이 스윙이 반격.
            pl.parry.riposte_swing = pl.swings;
            pl.state = PState::Heavy {
                t: 0.0,
                swing: pl.swings,
                dir: pl.facing_dir(),
                full: true,
                perfect: false,
            };
        } else {
            let hit = if dash { weapons::DASH } else { hit };
            // 비껴서기(0107): 적 등 뒤에서 끝난 회피의 대시 공격.
            if dash && pl.flank {
                pl.flank_swing = pl.swings;
            }
            pl.flank = false;
            let (s0, a0, r0) = hit_times(t, wpn, hit);
            pl.beat.since_swing = 0.0;
            pl.beat.period = s0 + a0 + r0;
            pl.beat.swing_hit = hit;
            pl.state = PState::Attack {
                t: 0.0,
                swing: pl.swings,
                dir,
                hit,
            };
        }
    }

    /// 교체 공격의 한 타(0272 1b): 막 바꿔 든 무기(`cx.wpn`)가 들어오며 친다 — 근접 = 대시 공격(값은 대시 공격 그대로: 기력을
    /// 치르고 박자 타가 아니다), 사격 무기 = 지금 겨눈 쪽으로 보통 한 발(탄 하나 · 기력). 반격 대신 나가는 타다 — 반격 창과
    /// 이어 패링은 닫힌다(패링 하나에 보상 하나). 판정 시간·무적은 없다.
    fn start_entry(&mut self, cx: &Ctx) {
        let Some(entry) = self.player.weapon.entry() else {
            return;
        };
        let pl = &mut self.player;
        pl.parry.since = f32::MAX;
        pl.parry.chain_left = 0.0;
        pl.parry.chain_n = 0;
        pl.beat.on = false;
        pl.combo_left = 0.0;
        match entry {
            weapons::Entry::Dash => {
                pl.dash_left = DASH_ATTACK_LATE;
                if self.swing_pays(cx) {
                    self.start_swing(cx);
                }
            }
            // 교체 공격은 당기지 않고 곧장 한 발(활도 — 0274 ②b).
            weapons::Entry::Shot => self.start_shot_as(cx, false),
        }
        let pl = &self.player;
        if matches!(pl.state, PState::Attack { .. } | PState::Cast { .. }) {
            self.skill_ev.push(Event::SwapStrike {
                at: pl.pos,
                dir: pl.facing_dir(),
                to: pl.weapon,
            });
        }
    }

    /// 원거리 무기의 RB = 사격(0051): 탄창이 비면 장전이 끝날 때까지 못 쏜다. 활(0274 ②b — `Rhythm::Draw`)은 쏘지 않고 화살을
    /// 메겨 당기기 시작한다(`CastCharge`) — 화살은 떼는 순간에 쓴다(당기다 구르면 잃지 않는다), 기력과 박자는 누르는 순간에.
    fn start_shot(&mut self, cx: &Ctx) {
        self.start_shot_as(cx, self.player.weapon.draws());
    }

    /// `start_shot` — `draws`면 당기기 시작, 아니면 곧장 한 발(쇠뇌 · 교체 공격으로 들어오는 활).
    fn start_shot_as(&mut self, cx: &Ctx, draws: bool) {
        let Rb::Shot { reload, time, .. } = cx.wpn.rb else {
            return;
        };
        let pl = &mut self.player;
        // 사격도 기력을 쓴다(0117, 사장 "원거리 공격은 탄환도 쓰지만 기력도 — 그렇지 않으면 너무 세다"): 볼트가 있을 때만 치르고,
        // 기력이 없으면 막힌다(땀방울·막대 떨림). 박자 사격은 볼트만 아낀다.
        if pl.ranged.ammo > 0 && !pl.spend(cx.t[P::StaminaAttack] * cx.wpn.stamina) {
            pl.beat.on = false;
            return;
        }
        // 박자(0071·0077): 앞 사격이 시작된 뒤 그 한 발의 시간(= 공격 속도) ± 창 안에 누른 사격은 볼트를 쓰지 않는다 —
        // 근접의 "박자 평타 = 스태미나 0"과 같은 틀, 쇠뇌의 값은 볼트. 첫 발·받아 둔 이른 누름은 쓴다.
        let free = pl.beat.on;
        pl.beat.on = false;
        if pl.ranged.ammo > 0 {
            if free {
                self.beat_now = true;
                pl.beat.chain += 1;
                pl.beat.since = 0.0;
            } else {
                pl.beat.chain = 0;
                if !draws {
                    pl.ranged.ammo -= 1;
                    // 회피로 넣은 한 발을 장전 중에 쏘면 장전은 이어 간다(처음부터 다시 세지 않는다, 0210).
                    if pl.ranged.ammo == 0 && pl.ranged.reload <= 0.0 {
                        pl.ranged.reload = reload;
                    }
                }
            }
            pl.beat.swing_beat = free;
            pl.beat.since_swing = 0.0;
            pl.beat.period = cx.t[P::CastTime] * time;
            pl.facing = cx.cast_dir.x.atan2(cx.cast_dir.z);
            pl.state = if draws {
                PState::CastCharge { t: 0.0 }
            } else {
                PState::Cast {
                    t: 0.0,
                    dir: cx.cast_dir,
                    fired: false,
                }
            };
        } else {
            pl.ranged.cast_denied = 0.6;
            self.cast_denied_now = true;
        }
    }

    /// 등잔 마시기(0031): 다시 누르기는 버린다. 삼키기 전 회피는 끊기(등잔 그대로) — 어깨 버튼 실수는 회피 한 번으로 무른다.
    fn tick_drink(&mut self, cx: &Ctx, e: f32, gulped: bool) {
        if !gulped && self.try_dodge(cx) {
            return;
        }
        let t = cx.t;
        let prays = self.bare() == Some(weapons::Bare::Prayer);
        let pl = &mut self.player;
        let e2 = e + cx.dt;
        let mut gulped = gulped;
        if !gulped && e2 >= t[P::FlaskRaise] {
            gulped = true;
            pl.flask -= 1;
            pl.hp = (pl.hp + t[P::FlaskHeal].round() as i32).min(pl.max_hp);
            self.gulp_now = true;
            // 기도하는 손(0276 — 망치의 빈 손): 삼킨 순간 둘레에 울림 고리(`apply_strike`가 낸다).
            self.prayer_now = prays;
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
        let after_hit = self.next_hit(hit);
        let pl = &mut self.player;
        let (s0, a0, r0) = hit_times(t, wpn, hit);
        let total = s0 + a0 + r0;
        let after = e >= s0 + a0;
        if a.attack_held && pl.attack_hold >= t[P::HeavyHoldStart] && after && wpn.heavy.is_some() {
            // 강공격(0030): 평타는 누르는 순간 나갔고, 한 번 누른 채 `heavy.hold start s`를 쥐고 있으면(활성이 끝난 뒤) 모으기로.
            // "그 순간 눌려 있나"만 보면 연타 중 휘두르기 도중의 짧은 누름이 활성 끝에 걸려 모으기로 샜다.
            // 끝맺음(0107): 다 모인 채로 시작한다.
            let full = t[P::HeavyCharge] * wpn.heavy.map_or(1.0, |h| h.charge);
            let start = if pl.parry.primed_heavy { full } else { 0.0 };
            pl.parry.primed_heavy = false;
            pl.state = PState::Charging { t: start };
        } else if e + cx.dt >= total {
            pl.state = PState::Idle;
            // 마지막 타 뒤에는 처음으로, 아니면 잠깐 다음 타를 기억한다.
            pl.combo_next = after_hit.unwrap_or(0);
            // 맺음타(0107): 마지막 타 뒤에도 콤보를 잠깐 기억한다 — 박자에 누르면 첫 타가 박자 타(스태미나 0)로 이어진다.
            let closing = after_hit.is_none() && self.tree.has(Cell::ClosingBlow);
            pl.parry.closing_ready = closing;
            pl.combo_left = if after_hit.is_some() || closing {
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
        if pl.spend(t[P::HeavyStamina] * cx.wpn.stamina) {
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
            pl.parry.release_guard = t[P::ParryWindow];
            // 고른 호흡(0107): 딱 맞춰 뗀 강타는 강타 기력을 돌려준다.
            let perfect = e >= full_at && e <= full_at + PERFECT_RELEASE;
            if perfect && self.tree.picked_a(Cell::EvenBreath) {
                pl.stamina =
                    (pl.stamina + t[P::HeavyStamina] * cx.wpn.stamina).min(t[P::StaminaMax]);
            }
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
    /// 자루를 길게 잡은 창(0276 — 창의 빈 손)은 내달려 꿰뚫기가 끝나는 순간 한 번 더 내달린다(새 스윙 — 앞의 적을 다시 꿰뚫는다,
    /// 기력 · 덤은 없다). 벼랑 앞에서 멈춘 꿰뚫기는 잇지 않는다.
    fn tick_heavy(&mut self, cx: &Ctx, e: f32, swing: u32, dir: Vec3, full: bool, perfect: bool) {
        let (s0, a0, r0) = heavy_times(cx.t, cx.wpn);
        let total = s0 + a0 + r0;
        let lane = cx.wpn.heavy.is_some_and(|h| h.shape.lane_half().is_some());
        if lane
            && self.bare() == Some(weapons::Bare::LongGrip)
            && swing != self.player.off.relunge
            && e < s0 + a0
            && e + cx.dt >= s0 + a0
            && !self.lunge_stops(dir, 0.3)
        {
            let pl = &mut self.player;
            pl.swings += 1;
            pl.off.relunge = pl.swings;
            pl.state = PState::Heavy {
                t: 0.0,
                swing: pl.swings,
                dir,
                full,
                perfect: false,
            };
            return;
        }
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
            Rb::Toss => 1.0,
        };
        if !fired && e2 >= t[P::CastStartup] * k {
            fired = true;
            cx.fire = Some((dir, 0));
        }
        // 활(0274 ②b)은 쏜 뒤 쥐어도 다시 당기지 않는다 — 새 누름이 `start_shot`으로 메긴다(기력·화살을 치르고).
        pl.state = if fired
            && !pl.weapon.draws()
            && a.cast_held
            && pl.ranged.cast_hold >= t[P::CastHoldStart]
            && cx.can_charge_cast
        {
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
        if self.player.weapon.draws() {
            self.tick_draw(cx, e);
            return;
        }
        if self.player.weapon.winds() {
            self.tick_wind(cx, e);
            return;
        }
        let (a, t, dt, wpn) = (&cx.a, cx.t, cx.dt, cx.wpn);
        // 두 손으로 엮는 지팡이(0276)는 빨리 모은다.
        let full = self.cast_full(t);
        if a.cast_held && cx.can_charge_cast {
            self.charged_now |= e < full && e + dt >= full;
            if ranged_aim_given(&cx.a) {
                // 겨누는 손(끌기·스틱·숄더뷰의 카메라, 0217·0219)을 몸이 따라 돈다 — 뗄 때 어디로 갈지 보이게.
                self.player.facing = cx.cast_dir.x.atan2(cx.cast_dir.z);
            }
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
            Rb::Toss => pl.ranged.mana -= 1.0,
            Rb::Shot { reload, heavy, .. } => {
                // 모은 사격은 볼트 `heavy`발(0210 — 모으기를 잇달아 하면 장전이 곧 온다).
                pl.ranged.ammo = pl.ranged.ammo.saturating_sub(heavy.max(1));
                if pl.ranged.ammo == 0 && pl.ranged.reload <= 0.0 {
                    pl.ranged.reload = reload;
                }
                // 모은 사격은 강타 값(0117) — 이미 모았으니 막지 않고 0까지 쓴다(소울류).
                pl.stamina = (pl.stamina - t[P::HeavyStamina] * wpn.stamina).max(0.0);
                pl.stamina_idle = 0.0;
            }
        }
        pl.facing = cx.cast_dir.x.atan2(cx.cast_dir.z);
        // 모으기 공격의 패링(0089): 원거리 무기의 모은 사격도 떼는 순간부터 패링 창(근접 무기의 RB 마법은 무기의 공격이 아니라 뺀다, 0077·0078과 같은 경계).
        if matches!(wpn.rb, Rb::Shot { .. }) {
            pl.parry.release_guard = t[P::ParryWindow];
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
            Rb::Toss => 1.0,
        };
        pl.state = PState::Cast {
            t: t[P::CastStartup] * k,
            dir: cx.cast_dir,
            fired: true,
        };
    }

    /// 창 던지기를 들어 겨누기(0274 ②b — `Id::winds`): 쥔 동안 모으고, 떼면 칸 하나로 던진다 — 톡(`cast.hold start s` 전) = 보통 창,
    /// 그 뒤 = 덜 모은 창, 다 모으면(`cast.charge s`) 모은 창. 근접 무기의 RB라 딱 맞춤 덤은 없다(0077 · 0078 경계 그대로).
    /// 톡은 보통 RB와 같은 길이(던지는 자리를 앞당길 뿐).
    fn tick_wind(&mut self, cx: &mut Ctx, e: f32) {
        let (a, t, dt) = (&cx.a, cx.t, cx.dt);
        let full = t[P::CastCharge];
        if a.cast_held && cx.can_charge_cast {
            self.charged_now |= e < full && e + dt >= full;
            if ranged_aim_given(&cx.a) {
                self.player.facing = cx.cast_dir.x.atan2(cx.cast_dir.z);
            }
            self.player.state = PState::CastCharge { t: e + dt };
            return;
        }
        let pl = &mut self.player;
        if !cx.can_charge_cast {
            pl.state = PState::Idle;
            return;
        }
        pl.ranged.mana -= 1.0;
        let power = if e >= full {
            2
        } else if e >= t[P::CastHoldStart] {
            1
        } else {
            0
        };
        pl.facing = cx.cast_dir.x.atan2(cx.cast_dir.z);
        cx.fire = Some((cx.cast_dir, power));
        pl.state = PState::Cast {
            t: e.min(t[P::CastStartup]),
            dir: cx.cast_dir,
            fired: true,
        };
    }

    /// 활 당기기(0274 ②b — `Rhythm::Draw`): 쥔 동안 당기고(겨누는 손을 몸이 따른다), 떼면 당긴 시간의 단계로 쏜다 —
    /// 톡 = 보통 화살(0) · `mid`초 넘게 = 반쯤 당긴 화살(1) · 끝까지 = 끝까지 당긴 화살(2 — 꿰뚫고 오금) · 다 당긴 뒤 창 안에 떼면
    /// 딱 맞춤(3). 화살은 떼는 순간 하나(박자에 맞춰 메긴 것은 공짜 — `swing_beat`), 끝까지 당긴 것은 강타 값의 기력을 더 쓴다.
    /// 쏘는 동작의 뒷부분으로 잇는다 — 톡은 보통 사격과 같은 길이(쏘는 자리를 앞당길 뿐), 오래 당긴 것은 놓는 자리부터.
    fn tick_draw(&mut self, cx: &mut Ctx, e: f32) {
        let (a, t, dt, wpn) = (&cx.a, cx.t, cx.dt, cx.wpn);
        let Rb::Shot { reload, time, .. } = wpn.rb else {
            return;
        };
        let Some(weapons::Rhythm::Draw { mid, .. }) = self.player.weapon.rhythm() else {
            return;
        };
        let full = t[P::CastCharge];
        if a.cast_held {
            self.charged_now |= e < full && e + dt >= full;
            if ranged_aim_given(&cx.a) {
                self.player.facing = cx.cast_dir.x.atan2(cx.cast_dir.z);
            }
            self.player.state = PState::CastCharge { t: e + dt };
            return;
        }
        let pl = &mut self.player;
        if pl.ranged.ammo == 0 {
            pl.state = PState::Idle;
            return;
        }
        if !pl.beat.swing_beat {
            pl.ranged.ammo -= 1;
            if pl.ranged.ammo == 0 && pl.ranged.reload <= 0.0 {
                pl.ranged.reload = reload;
            }
        }
        let perfect = e >= full && e <= full + PERFECT_RELEASE;
        let power = if perfect {
            3
        } else if e >= full {
            2
        } else if e >= mid {
            1
        } else {
            0
        };
        if power >= 2 {
            // 끝까지 당긴 화살은 강타 값(0117)을 더 낸다 — 옛 "한 발 + 모은 사격"(평타 값 + 강타 값)과 같은 값, 막지 않고 0까지(소울류).
            let more = t[P::HeavyStamina] * wpn.stamina;
            pl.stamina = (pl.stamina - more).max(0.0);
            pl.stamina_idle = 0.0;
            // 모으기 공격의 패링(0089)과 같은 경계: 끝까지 당겨 놓는 순간부터 패링 창.
            pl.parry.release_guard = t[P::ParryWindow];
        }
        pl.facing = cx.cast_dir.x.atan2(cx.cast_dir.z);
        cx.fire = Some((cx.cast_dir, power));
        pl.state = PState::Cast {
            t: e.min(t[P::CastStartup] * time),
            dir: cx.cast_dir,
            fired: true,
        };
    }

    /// 수류탄 떨구기(0051 → 0178): 손을 떠난 뒤엔 회피로 끊는다. 쥐고 있어도 겨누지 않는다(0055의 꾹 = 멀리를 걷었다 — 늘 발 앞).
    fn tick_throw(&mut self, cx: &mut Ctx, e: f32, to: Vec3, done: bool) {
        if done && self.try_dodge(cx) {
            return;
        }
        let t = cx.t;
        let pl = &mut self.player;
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
            // 비껴서기(0107): 회피가 가까운 적의 등 뒤(그 적이 바라보는 쪽의 반대 반평면)에서 끝났다.
            if self.tree.has(Cell::Sidestep) {
                let p = self.player.pos;
                self.player.flank = self.enemies.iter().any(|en| {
                    let d = vec3(p.x - en.pos.x, 0.0, p.z - en.pos.z);
                    en.active()
                        && d.length() - en.radius <= cell::SIDESTEP_RANGE
                        && d.dot(en.facing_dir()) < 0.0
                });
            }
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
            pl.ranged.hover = 0.0;
            if let Some(d) = cx.auto_dir.or(cx.aim) {
                pl.facing = d.x.atan2(d.z);
            }
        } else if cx.a.cast && !plunge && !pl.ranged.air_shot {
            // 공중 RB(0111, 사장 2026-11-08): 근접 무기 = 마법 한 칸을 내려 쏘고, 쇠뇌 = 볼트를 부채꼴로 내려 쏜다(탄창은 안 쓴다).
            // 쏘는 순간 잠깐 떠 있다(체공) — 한 점프에 한 번이라 연타로 떠 있지 못한다.
            let javelin = self.rb_out();
            let shield_rb = self.shield_throws();
            let pl = &mut self.player;
            let ok = match cx.wpn.rb {
                // 방패(0276): 칸 없이 — 날고 있지 않으면.
                Rb::Toss if shield_rb && !javelin => true,
                Rb::Toss if pl.ranged.mana >= 1.0 && !javelin && !shield_rb => {
                    pl.ranged.mana -= 1.0;
                    true
                }
                Rb::Toss => {
                    pl.ranged.cast_denied = 0.6;
                    self.cast_denied_now = true;
                    false
                }
                // 쇠뇌 내려 쏘기(0111)도 기력(0117) — 볼트는 그대로 안 쓴다.
                Rb::Shot { .. } => pl.spend(t[P::StaminaAttack] * cx.wpn.stamina),
            };
            let pl = &mut self.player;
            if ok {
                pl.ranged.air_shot = true;
                pl.ranged.hover = AIR_SHOT_HOVER;
                pl.vy = pl.vy.max(0.0);
                pl.facing = cx.cast_dir.x.atan2(cx.cast_dir.z);
                self.air_fire = Some(cx.cast_dir);
            }
        }
        let pl = &mut self.player;
        // 포물선을 그대로(틱마다 위치 = v·dt − g·dt²/2) — 앞 오일러 적분은 높이가 6% 모자랐다.
        // 체공(0111) 동안은 그 높이에 멈춘다(오르던 속도는 버린다).
        if pl.ranged.hover > 0.0 {
            pl.ranged.hover = (pl.ranged.hover - dt).max(0.0);
            pl.vy = 0.0;
        } else {
            let g = crate::tuning::jump_ballistics(t).1 * if plunge { PLUNGE_FALL } else { 1.0 };
            pl.air += pl.vy * dt - 0.5 * g * dt * dt;
            pl.vy -= g * dt;
        }
        if pl.air <= 0.0 && e > 0.0 {
            pl.air = 0.0;
            pl.vy = 0.0;
            let dur = if plunge {
                t[P::PlungeLand]
            } else {
                t[P::JumpLand]
            };
            pl.state = PState::Land {
                t: 0.0,
                dur,
                plunge,
            };
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
            // 멈추지 않는 걸음(0107): 달리기 기력 ×0.7.
            let k = if self.tree.picked_a(Cell::UnbrokenStride) {
                cell::STRIDE_RUN
            } else {
                1.0
            };
            pl.stamina = (pl.stamina - RUN_STAMINA * k * cx.dt).max(0.0);
            pl.stamina_idle = 0.0;
            pl.dash_left = pl.dash_left.max(DASH_ATTACK_LATE);
        } else {
            pl.running = false;
        }
    }

    /// 이동: 상태마다 속도를 정하고 한 번 옮긴다.
    fn move_player(&mut self, cx: &Ctx) {
        let (t, dt, wpn, world_move) = (cx.t, cx.dt, cx.wpn, cx.world_move);
        // 스킬 중이면 그 단의 실제 값(0158).
        let d_skill = match self.player.state {
            PState::Skill { id, .. } => Some(self.skill_eff(id)),
            _ => None,
        };
        // 모으는 걸음(0215 — 고른 호흡 II): 강타를 모으는 동안 ×1.4.
        let charge_walk = if self.tree.picked_b(Cell::EvenBreath) {
            cell::CHARGE_WALK
        } else {
            1.0
        };
        // 내달리는 강타(창 — 길 모양, 0272 2단계)는 벼랑 · 턱 끝에서 멈춘다(떨어지며 꿰뚫지 않는다).
        let lunge_halts = match self.player.state {
            PState::Heavy { dir, .. }
                if wpn.heavy.is_some_and(|h| h.shape.lane_half().is_some()) =>
            {
                let step = t[P::HeavyLunge] * wpn.heavy.map_or(1.0, |h| h.lunge);
                let (s0, a0, _) = heavy_times(t, wpn);
                self.lunge_stops(dir, step / (s0 + a0).max(1e-3) * dt + 0.05)
            }
            _ => false,
        };
        let light_step = self.bare() == Some(weapons::Bare::LightStep);
        let pl = &mut self.player;
        match pl.state {
            PState::Dodge { dir, .. } => {
                pl.vel = dir * (t[P::DodgeDist] / t[P::DodgeTime]);
            }
            // 들이받기(0276): 모인 힘만큼의 거리를 `RUSH_TIME`초에 — 문지기에 닿으면(`connected`) 거기서 멈춘다.
            PState::Rush {
                t: e,
                dir,
                power,
                swing,
            } => {
                let (d0, d1) = crate::tuning::RUSH_DIST;
                let dist = d0 + (d1 - d0) * power;
                pl.vel = if e < crate::tuning::RUSH_TIME && pl.connected != swing {
                    dir * (dist / crate::tuning::RUSH_TIME)
                } else {
                    Vec3::ZERO
                };
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
            PState::Hurt { .. } | PState::Recoil { .. } => {
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
            PState::Turn { .. } | PState::Land { .. } | PState::Ult { .. } => pl.vel = Vec3::ZERO,
            // 스킬(0151): 가로지르기는 판정 구간 동안 `dist` m를 곧게 — 되돌아 베기(0158 ㄱ)면 같은 길을 되돌아온다.
            // 휘돌이의 도는 걸음(ㄱ)은 돌면서 느리게 걷고, 땅울림의 멀리 뛰기(ㄱ)는 시동 동안 정해 둔 거리를 뛰어든다.
            PState::Skill { id, t: e, dir, .. } => {
                use crate::skills::{Form, Twist};
                let d = d_skill.unwrap_or_else(|| id.def().at(1, crate::skills::Pick::A));
                let s = d.skill;
                pl.vel = match (s.form, d.twist) {
                    (Form::Lane { dist, time, .. }, twist) if e >= s.startup => {
                        let v = dist / time.max(1e-3);
                        if e < s.startup + time {
                            dir * v
                        } else if twist == Some(Twist::Return) && e < s.startup + time * 2.0 {
                            -dir * v
                        } else {
                            Vec3::ZERO
                        }
                    }
                    // 버팀의 버티는 걸음(0158 G2 ㄱ)도 같은 걸음 — 버티는 동안만.
                    (Form::Ring { .. }, Some(Twist::Walk { speed }))
                    | (Form::Brace { .. }, Some(Twist::Walk { speed }))
                        if !matches!(s.form, Form::Brace { .. })
                            || (e >= s.startup && e < s.startup + d.active()) =>
                    {
                        let want = world_move * t[P::MoveSpeed] * speed;
                        let diff = want - pl.vel;
                        let max = t[P::MoveAccel] * dt;
                        pl.vel
                            + if diff.length() > max {
                                diff.normalize() * max
                            } else {
                                diff
                            }
                    }
                    // 빠지며 베기(0274 ③): 벤 뒤 `time`초 동안 `back` m 뒤로(먼 걸음 ㄱ은 `Skill::at`이 거리를 바꿔 둔다).
                    (Form::Slip { back, time, .. }, _)
                        if e >= s.startup && e < s.startup + time =>
                    {
                        -dir * (back / time.max(1e-3))
                    }
                    // 바늘땀의 파고드는 걸음(0270 C3 ㄱ): 찌르는 동안 앞으로 내딛는다.
                    (Form::Stab { .. }, Some(Twist::Lunge { dist }))
                        if e >= s.startup && e < s.startup + d.active() =>
                    {
                        dir * (dist / d.active().max(1e-3))
                    }
                    (Form::Cone { .. }, Some(Twist::Leap { .. })) if e < s.startup => {
                        dir * (pl.skill.leap / s.startup.max(1e-3))
                    }
                    // 제비걸음(0214): 걸음마다 앞 6할 동안 닿을 자리로 뛴다.
                    (Form::Hop { gap, .. }, _) if e >= s.startup && e < s.startup + d.active() => {
                        let into = (e - s.startup) % gap.max(1e-3);
                        let dash = gap * crate::tuning::HOP_DASH;
                        if into < dash {
                            let way = pl.skill.hop_to - pl.skill.hop_from;
                            vec3(way.x, 0.0, way.z) / dash.max(1e-3)
                        } else {
                            Vec3::ZERO
                        }
                    }
                    _ => Vec3::ZERO,
                };
            }
            PState::Heavy { t: e, dir, .. } => {
                let (s0, a0, _) = heavy_times(t, wpn);
                let reach = s0 + a0;
                let lunge = t[P::HeavyLunge] * wpn.heavy.map_or(1.0, |h| h.lunge);
                pl.vel = if e < reach && !lunge_halts {
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
            | PState::Block { .. }
            | PState::Drink { .. }
            | PState::Charging { .. }
            | PState::Cast { .. }
            | PState::CastCharge { .. }
            | PState::Throw { .. } => {
                // 마시는 동안·강공격을 모으는 동안(0047)·마법을 쏘는 동안(0048)은 느리게 걷는다.
                let slow = match pl.state {
                    PState::Drink { .. } => t[P::FlaskWalk],
                    PState::Charging { .. } => {
                        t[P::HeavyChargeWalk] * wpn.heavy.map_or(1.0, |h| h.walk) * charge_walk
                    }
                    // 활을 당기는 동안(0274 ②b)은 쇠뇌를 모을 때보다 조금 덜 느리다 — 움직이며 당겨 쏘는 무기.
                    // 가벼운 걸음(0276 — 활의 빈 손)은 거의 걷는 빠르기로.
                    PState::CastCharge { .. } => match pl.weapon.rhythm() {
                        Some(weapons::Rhythm::Draw { .. }) if light_step => {
                            t[P::CastWalk] * weapons::BARE_BOW_WALK
                        }
                        Some(weapons::Rhythm::Draw { walk, .. }) => t[P::CastWalk] * walk,
                        _ => t[P::CastWalk],
                    },
                    PState::Cast { .. } | PState::Throw { .. } => t[P::CastWalk],
                    PState::Block { .. } => crate::tuning::BLOCK_WALK,
                    PState::Idle if pl.running => RUN_SPEED_X,
                    _ => 1.0,
                };
                // 흐르는 걸음(결, 0270 C3): 피 흘리는 적을 친 뒤 잠깐 빨리 걷는다.
                let stride = if pl.skill.stride > 0.0 {
                    crate::progression::vein::SHARP_STRIDE
                } else {
                    1.0
                };
                let want = world_move * t[P::MoveSpeed] * slow * stride;
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
        pl.faces_away = matches!(cx.basis, Basis::Lock(_))
            || cx.aim.is_some()
            || matches!(pl.state, PState::Block { .. });
        let want_yaw = match (cx.basis, pl.state) {
            (
                _,
                PState::Attack { .. }
                | PState::Heavy { .. }
                | PState::Turn { .. }
                | PState::Hurt { .. }
                | PState::Recoil { .. }
                | PState::Parry { .. }
                | PState::Jump { plunge: true, .. }
                | PState::Cast { .. }
                | PState::Throw { .. }
                | PState::Land { .. }
                | PState::Skill { .. }
                | PState::Ult { .. },
            ) => None,
            (Basis::Lock(tg), _) => {
                let d = tg - pl.pos;
                Some(d.x.atan2(d.z))
            }
            _ if cx.aim.is_some() => cx.aim.map(|d| d.x.atan2(d.z)),
            // 막는 동안(0223)은 걷는 쪽으로 돌지 않는다 — 방패를 든 채 뒤로 물러설 수 있게(락온·조준 스틱은 위에서 돌린다).
            (_, PState::Block { .. }) => None,
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
        // 그림자 걸음(0276 — 쌍단검의 빈 손): 회피가 끝나면(끊고 나간 공격까지) 창이 열리고, 그 뒤 첫 휘두르기(평타 · 대시 · 강공격)가
        // 그 표지를 받는다 — 헛쳐도 쓴다. 창이 닫히거나 다른 휘두르기가 먼저면 없다.
        if self.bare() == Some(weapons::Bare::ShadowStep) {
            let pl = &mut self.player;
            if was_dodging && !matches!(pl.state, PState::Dodge { .. }) {
                pl.off.shadow = weapons::SHADOW_STEP_LATE;
                pl.off.shadow_from = match pl.state {
                    PState::Attack { swing, .. } | PState::Heavy { swing, .. } => swing - 1,
                    _ => pl.swings,
                };
            }
            if pl.off.shadow > 0.0 && pl.swings > pl.off.shadow_from {
                if let PState::Attack { swing, .. } | PState::Heavy { swing, .. } = pl.state {
                    pl.off.shadow_swing = swing;
                }
                pl.off.shadow = 0.0;
            }
        }
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
            if let Rb::Shot { .. } = cx.wpn.rb {
                let mag = self.mag_of(self.player.weapon);
                let pl = &mut self.player;
                if pl.ranged.ammo < mag {
                    pl.ranged.ammo += 1;
                    if pl.ranged.ammo == mag {
                        pl.ranged.reload = 0.0;
                    }
                }
            }
        }
    }
}

impl World {
    /// 패드 조준 보조(0219): 겨눈 쪽 `d` ± `deg`° 안, 지금 원거리의 사거리 안에 선 깨어 있는 적 가운데 겨눈 쪽에 가장 가까운 적 쪽.
    /// 없으면 `d` 그대로 — 원뿔 밖의 적을 찾아 돌지 않는다(0213).
    /// 탄이 닿는(`shot_clear`) 가장 가까운 깨어 있는 적 쪽(0048 마법 탄·0217 터치 톡의 찾기 — 0228부터 벽 너머는 빼고).
    pub(super) fn nearest_shootable(&self, reach: f32, t: &Tuning) -> Option<Vec3> {
        let p = self.player.pos;
        let mut near: Vec<(f32, &crate::enemy::Enemy)> = self
            .enemies
            .iter()
            .filter(|e| e.active())
            .map(|e| (vec2(e.pos.x - p.x, e.pos.z - p.z).length() - e.radius, e))
            .filter(|(d, _)| *d <= reach)
            .collect();
        near.sort_by(|a, b| a.0.total_cmp(&b.0));
        near.into_iter()
            .find(|(_, e)| self.shot_clear(e.pos, e.radius, t))
            .map(|(_, e)| vec3(e.pos.x - p.x, 0.0, e.pos.z - p.z).normalize_or_zero())
            .filter(|d| *d != Vec3::ZERO)
    }

    pub(super) fn assist_aim(&self, d: Vec3, deg: f32, t: &Tuning) -> Vec3 {
        let reach = t[P::CastRange] * self.rb_bolts().0.range;
        let cos = deg.to_radians().cos();
        let p = self.player.pos;
        self.enemies
            .iter()
            .filter(|en| en.active() && !en.harmless())
            .filter_map(|en| {
                let v = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z);
                let n = v.length();
                let c = if n > 1e-3 { v.dot(d) / n } else { -1.0 };
                (n <= reach + en.radius && c >= cos).then_some((c, v / n, en))
            })
            // 벽 너머의 적에게는 붙지 않는다(0228) — 원뿔 안에서 가까운 차례로 보고 탄이 닿는 첫 적.
            .filter(|(_, _, en)| self.shot_clear(en.pos, en.radius, t))
            .map(|(c, v, _)| (c, v))
            .max_by(|a, b| a.0.total_cmp(&b.0))
            .map_or(d, |(_, v)| v)
    }
}

/// 손이 원거리를 겨누고 있나(0219 → 0221): 터치 끌기·스틱·커서·숄더뷰의 카메라 앞 — 헤어라인의 "바라보는 쪽"은 아니다.
fn ranged_aim_given(a: &Actions) -> bool {
    a.aim_drag || a.aim_ahead || a.aim.length() > AIM_STICK || a.aim_at.is_some()
}
