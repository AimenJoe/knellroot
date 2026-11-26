//! 적의 틱 — 상태마다의 몸(0278 R20b: `Enemy::tick`의 `match self.state` 갈래를 상태마다 이름 있는 메서드로 옮겼다, 몸은 그대로).
//! `tick`은 머리(시계 · 넉백 · 산 것 · 2·3단계 전환 · 쫓는 쪽과 빠르기)와 꼬리(사슬)만 하고, 갈래마다 한 줄로 여기를 부른다.
//! 새 상태 = `EState` 한 갈래 + `tick`의 빠짐없는 match 한 줄 + 여기 메서드 하나.
use super::*;

/// 틱 밖의 형편: 방랑자 자리 · 틱 길이 · 종의 표 · 방랑자까지의 거리와 쪽 · 이번 틱의 빠르기(2단계 · 사슬 · 느려짐 · 짐).
#[derive(Clone, Copy)]
pub(super) struct Ctx {
    pub player: Vec3,
    pub dt: f32,
    pub d: &'static Def,
    pub dist: f32,
    pub dir: Vec3,
    pub speed: f32,
}

/// 상태의 몸이 틱을 거기서 끝내나(옛 `return out` — 꼬리의 사슬도 건너뛴다).
pub(super) enum Flow {
    On,
    Stop,
}

/// 떼 퍼지기(0076): 가는 쪽의 옆으로만 비킨다(뒤로 물러서지 않는다), 앞이 막혔으면 조금 늦춘다.
pub(super) fn steer(go: Vec3, spread: Vec3) -> (Vec3, f32) {
    let side = spread - go * spread.dot(go);
    let slow = if spread.dot(go) < 0.0 {
        (1.0 + spread.dot(go) * 0.5).max(0.5)
    } else {
        1.0
    };
    ((go + side * SPREAD_STEER).normalize_or(go), slow)
}

impl Enemy {
    /// 엎드린 필드 보스(0270 D): 깨우면(`stir`) `FIELD_WAKE` 동안 일어서 쫓고, 아니면 엎드린 채(돌아와 눕는 중이면 마저 눕는다).
    pub(super) fn tick_dormant(&mut self, cx: &Ctx) {
        let Ctx { dt, dir, .. } = *cx;
        if self.waking {
            self.turn_toward(dir, 1.5, dt);
            self.stand = (self.stand + dt / crate::tuning::FIELD_WAKE).min(1.0);
            if self.stand >= 1.0 {
                self.waking = false;
                self.state = EState::Chase;
            }
        } else {
            self.stand = (self.stand - dt / crate::tuning::FIELD_SETTLE).max(0.0);
        }
    }

    pub(super) fn tick_idle(&mut self, cx: &Ctx) {
        let Ctx { dt, speed, .. } = *cx;
        if let Some((_, face)) = self.home {
            let (s, c) = face.sin_cos();
            self.turn_toward(vec3(s, 0.0, c), 2.0, dt);
        }
        // 청소자(0120): 싸움이 없으면 먹는다 — `EAT_TIME`마다 체력 한 눈금(`HP_UNIT`, lore ecology 청소자 "잔해를 먹고 회복").
        if self.docile && self.hp < self.max_hp {
            self.eat_t += dt;
            if self.eat_t >= crate::tuning::EAT_TIME {
                self.eat_t = 0.0;
                self.hp = (self.hp + crate::tuning::HP_UNIT).min(self.max_hp);
            }
        }
        // 0242(사장 설문 2-3 "너무 고정적으로 서 있다"): 청소자는 한 자리에서 먹다가(`GRAZE_STAY`) 제자리 둘레
        // `GRAZE_R` 안의 다음 자리로 느릿느릿 옮겨 간다 — 싸움 규칙(보고도 덤비지 않음)은 그대로.
        if self.docile {
            if let Some((home, _)) = self.home {
                self.forage(home, speed, dt);
            }
        }
    }

    pub(super) fn tick_return(&mut self, cx: &Ctx) -> Flow {
        let Ctx { dt, speed, .. } = *cx;
        let Some((home, face)) = self.home else {
            self.state = EState::Chase;
            return Flow::Stop;
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
            self.stance.poise = 0.0;
            // 타입 상태(0223 V2)도 걷힌다 — 가득 찬 적이 화상으로 다시 깎이지 않게.
            self.ail = Default::default();
            self.rhythm.repeated = false;
            // 필드 보스(0270 D)는 서서 기다리지 않는다 — 다시 엎드린다(처음부터: 1단계·공격 차례·드러남도 지운다).
            if self.field {
                self.state = EState::Dormant;
                self.waking = false;
                self.roused = false;
                self.atk_count = 0;
                self.linked = false;
                self.exposed = 0.0;
                self.stance = Stance::default();
                self.knock = Vec3::ZERO;
            }
        }
        Flow::On
    }

    pub(super) fn tick_chase(&mut self, cx: &Ctx) {
        let Ctx {
            player,
            dt,
            dist,
            dir,
            speed,
            ..
        } = *cx;
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

    pub(super) fn tick_tele(&mut self, e: f32, cx: &Ctx, out: &mut Out) -> Flow {
        let Ctx {
            player, dt, d, dir, ..
        } = *cx;
        // 페인트(0128): 들어 올림의 `FEINT_AT`에서 끊고 다른 공격의 예고로(그 예고는 그 공격의 모양·소리).
        if let Some(next) = self.rhythm.feint {
            if e >= self.lift_dur() * crate::tuning::FEINT_AT {
                self.begin(next);
                return Flow::Stop;
            }
        }
        let dur = self.tele_dur();
        // 지연이 풀리기 직전 소리(0128 "몸 + 소리") — 맞기 `RELEASE_LEAD`초 전, 한 번.
        if matches!(self.rhythm.tempo, Tempo::Delayed(_))
            && !self.cued
            && dur - (e + dt) <= crate::tuning::RELEASE_LEAD
        {
            self.cued = true;
            out.release = true;
        }
        if e < dur - TELE_LOCK {
            let rate = match self.atk {
                // 고요한 사슴(0270 D): 긁는 동안 빨리 돈다 — 2단계에 지나친 뒤 돌아서 한 번 더 이을 만큼.
                Atk::Charge if d.behavior == Behavior::Stag => crate::tuning::STAG_CHARGE_TURN,
                Atk::Charge => 4.0,
                // 웅크린 껍질은 느리게 돈다(0185) — 옆으로 돌아 들어갈 틈.
                Atk::Shell => crate::tuning::SHELL_TURN,
                _ => 2.0,
            };
            self.turn_toward(dir, rate, dt);
            // 방향이 굳기 전까지는 닿는 때를 다시 잰다(굳은 뒤엔 그대로 — 채움이 일정한 빠르기로).
            if self.atk == Atk::Charge {
                self.charge.aim = true;
                self.charge.eta = self.eta_to(player);
            }
        }
        self.state = if e + dt < dur {
            EState::Tele { t: e + dt }
        } else {
            match self.atk {
                Atk::Smash | Atk::Slam | Atk::Plant | Atk::Shards | Atk::Hurl | Atk::Stomp => {
                    EState::Swing { t: 0.0 }
                }
                Atk::Toll | Atk::Roots | Atk::Hook => EState::Wave { t: 0.0 },
                Atk::Charge => EState::Charge {
                    t: 0.0,
                    dir: self.facing_dir(),
                },
                Atk::Wave => EState::Wave { t: 0.0 },
                // 껍질을 푼다 — 곧장 쉰다(틈).
                Atk::Shell => EState::Recover { t: 0.0 },
            }
        };
        Flow::On
    }

    pub(super) fn tick_stomp(&mut self, cx: &Ctx, out: &mut Out) {
        let Ctx { dist, .. } = *cx;
        // 내려 딛기(0270 D): 앞발이 닿는 틱 — 몸 둘레 원판 안이면 맞는다(받지 못한다). 다음 틱부터 뿌리 줄이 솟는다.
        if dist <= STOMP_R + PLAYER_RADIUS {
            out.hit = true;
        }
        out.blow = self.blow_at(self.pos);
        self.state = EState::Wave { t: 0.0 };
    }

    pub(super) fn tick_shards(&mut self, e: f32, cx: &Ctx, out: &mut Out) {
        let Ctx { dt, .. } = *cx;
        // 종 조각(0125): 휘두르는 첫 틱에 던진다(탄은 월드가 날린다).
        if e == 0.0 {
            out.shards = self.shard_count();
            out.blow = self.blow_at(self.pos);
        }
        self.state = if e + dt >= 0.25 {
            self.after_attack()
        } else {
            EState::Swing { t: e + dt }
        };
    }

    pub(super) fn tick_hurl(&mut self, e: f32, cx: &Ctx, out: &mut Out) {
        let Ctx { player, dt, .. } = *cx;
        // 종이 떨어진다(0133): 그 순간 원판 안이면 맞고, 곧 떨어진 자리에서 고리 한 겹이 퍼진다.
        if e == 0.0 {
            if let Some(at) = self.plant_at {
                if vec2(player.x - at.x, player.z - at.z).length() <= HURL_R + PLAYER_RADIUS {
                    out.hit = true;
                }
                out.blow = self.blow_at(at);
            }
        }
        self.state = if e + dt >= 0.2 {
            EState::Wave { t: 0.0 }
        } else {
            EState::Swing { t: e + dt }
        };
    }

    pub(super) fn tick_plant(&mut self, e: f32, cx: &Ctx, out: &mut Out) {
        let Ctx { player, dt, d, .. } = *cx;
        // 심은 원판이 터진다(0120): 그 순간 플레이어 가운데가 원판 안이면 맞는다.
        if e == 0.0 {
            if let Some(at) = self.plant_at {
                if vec2(player.x - at.x, player.z - at.z).length() <= d.plant_r {
                    out.hit = true;
                }
                out.blow = self.blow_at(at);
            }
        }
        self.state = if e + dt >= 0.2 {
            self.plant_at = None;
            self.after_attack()
        } else {
            EState::Swing { t: e + dt }
        };
    }

    pub(super) fn tick_swing(&mut self, e: f32, cx: &Ctx, out: &mut Out) {
        let Ctx {
            dt, d, dist, dir, ..
        } = *cx;
        if e == 0.0 {
            let arc = if self.atk == Atk::Slam {
                (d.arc * 0.5).to_radians().cos()
            } else {
                -1.0
            };
            if dist <= d.reach + PLAYER_RADIUS && dir.dot(self.facing_dir()) >= arc {
                out.hit = true;
            }
            out.blow = self.blow_at(self.pos);
        }
        self.state = if e + dt >= 0.2 {
            self.after_attack()
        } else {
            EState::Swing { t: e + dt }
        };
    }

    pub(super) fn tick_charge(&mut self, e: f32, cdir: Vec3, cx: &Ctx, out: &mut Out) {
        let Ctx { player, dt, d, .. } = *cx;
        // 벽·기둥·턱 앞(`charge_len`)에서 멈춘다. 닿았나는 움직인 뒤의 몸으로(마지막 자리도 맞는다 — 띠 끝과 같게).
        let went = (e * d.charge_speed).min(self.charge.len);
        let step = (d.charge_speed * dt).min(self.charge.len - went).max(0.0);
        let from = self.pos;
        self.pos += cdir * step;
        // 이번 틱에 몸이 쓸고 간 선분과 플레이어 사이 — 틱마다 점으로 재면 띠 가장자리에 빈틈이 생긴다.
        let to_p = vec3(player.x - from.x, 0.0, player.z - from.z);
        let k = to_p.dot(cdir).clamp(0.0, step);
        let near = (to_p - cdir * k).length();
        if !self.landed && near <= self.touch() {
            self.landed = true;
            out.hit = true;
            // 닿은 자리(몸 앞, 방랑자 쪽) — 0269.
            out.blow = self.blow_at(self.pos + cdir * self.radius);
        }
        let done = went + step >= self.charge.len - 1e-4;
        let over = e + dt >= d.charge_time || done;
        // 닿지 않고 멈췄다 — 멈춘 자리에서 쿵(0269: 빗나간 돌진도 끝이 보인다).
        if over && !self.landed {
            out.blow = self.blow_at(self.pos + cdir * self.radius);
        }
        self.state = if over {
            self.after_attack()
        } else {
            EState::Charge {
                t: e + dt,
                dir: cdir,
            }
        };
    }

    pub(super) fn tick_hook(&mut self, e: f32, cx: &Ctx, out: &mut Out) {
        let Ctx { player, dt, .. } = *cx;
        if e == 0.0 {
            out.blow = self.blow_at(self.pos);
        }
        // 갈고리(0208): 앞머리가 지나는 자리의 방랑자를 한 번 걸어(`pull` — 월드가 끌어당긴다), 끝까지 가면 거둔다.
        let f = self.facing_dir();
        let to = vec3(player.x - self.pos.x, 0.0, player.z - self.pos.z);
        let along = to.dot(f);
        let side = (to - f * along).length();
        let speed = crate::tuning::HOOK_SPEED;
        let front = (self.radius + (e + dt) * speed).min(self.charge.len);
        if self.rings_hit == 0
            && along > 0.0
            && side <= crate::tuning::HOOK_R + PLAYER_RADIUS
            && along <= front
            && along >= front - speed * 0.12 - 0.6
        {
            self.rings_hit = 1;
            out.hit = true;
            out.pull = true;
        }
        self.state = if front >= self.charge.len {
            EState::Recover { t: 0.0 }
        } else {
            EState::Wave { t: e + dt }
        };
    }

    pub(super) fn tick_roots(&mut self, e: f32, cx: &Ctx, out: &mut Out) {
        let Ctx { player, dt, .. } = *cx;
        if e == 0.0 {
            // 내려 딛기(0270 D)의 뿌리 줄: 딛는 순간(`Around`)은 이미 말했다 — 줄이 솟기 시작하는 것을 따로.
            out.blow = if self.atk == Atk::Stomp {
                Some((
                    self.pos,
                    Blow::Line {
                        len: self.charge.len,
                    },
                ))
            } else {
                self.blow_at(self.pos)
            };
        }
        // 뿌리 기둥 줄(0133): 앞머리가 지나는 자리의 방랑자를 한 번 친다 — 줄 위(옆 `ROOTS_R` 안). 줄이 여럿이어도
        // (0270 D 세 줄 부채꼴) 한 번만.
        let to = vec3(player.x - self.pos.x, 0.0, player.z - self.pos.z);
        let reach = self.roots_reach();
        let raw = self.radius + (e + dt) * ROOTS_SPEED;
        for (yaw, len) in self.root_lines() {
            let f = self.dir_at(yaw);
            let along = to.dot(f);
            let side = (to - f * along).length();
            let front = raw.min(len);
            if self.rings_hit == 0
                && raw <= len + ROOTS_SPEED * dt
                && along > 0.0
                && side <= ROOTS_R + PLAYER_RADIUS
                && along <= front
                && along >= front - ROOTS_SPEED * 0.12 - 0.6
            {
                self.rings_hit = 1;
                out.hit = true;
            }
        }
        self.state = if raw >= reach {
            EState::Recover { t: 0.0 }
        } else {
            EState::Wave { t: e + dt }
        };
    }

    pub(super) fn tick_wave(&mut self, e: f32, cx: &Ctx, out: &mut Out) {
        let Ctx { player, dt, d, .. } = *cx;
        // 땅을 친 순간(0269) — 종 내던지기의 고리는 떨어진 순간(`Spot`)이 이미 말했다.
        if e == 0.0 && self.atk != Atk::Hurl {
            out.blow = self.blow_at(self.pos);
        }
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

    pub(super) fn tick_recover(&mut self, e: f32, cx: &Ctx) {
        let Ctx { dt, .. } = *cx;
        self.state = if e + dt >= self.recover_dur() {
            EState::Chase
        } else {
            EState::Recover { t: e + dt }
        };
    }

    pub(super) fn tick_rise(&mut self, e: f32, cx: &Ctx, out: &mut Out) {
        let Ctx { dt, dir, .. } = *cx;
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

    pub(super) fn tick_stagger(&mut self, e: f32, dur: f32, cx: &Ctx) {
        let Ctx { dt, .. } = *cx;
        self.state = if e + dt >= dur {
            EState::Chase
        } else {
            EState::Stagger { t: e + dt, dur }
        };
    }
}
