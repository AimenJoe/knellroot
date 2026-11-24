//! 스킬이 놓고 간 것의 틱(0251 R11 — `skill_tick.rs`에서 뺐다): 갈라진 땅·느려짐·솟은 뿌리, 던진 무기, 불티 마당, 그리고
//! 놓인 것·되울림이 적에게 닿는 타(`skill_touch`·`skill_blast`·`echo_ring`).
use super::*;

impl World {
    pub(in crate::world) fn tick_skill_fields(&mut self, t: &Tuning, dt: f32) {
        for b in &mut self.skill_bursts {
            b.left -= dt;
        }
        let due: Vec<Burst> = self
            .skill_bursts
            .iter()
            .copied()
            .filter(|b| b.left <= 0.0)
            .collect();
        self.skill_bursts.retain(|b| b.left > 0.0);
        for b in due {
            self.skill_burst(t, b);
        }
        for z in &mut self.skill_zones {
            z.left -= dt;
        }
        self.skill_zones.retain(|z| z.left > 0.0);
        for en in &mut self.enemies {
            en.marks.slow = 1.0;
            en.marks.snare = (en.marks.snare - dt).max(0.0);
            en.marks.bind = (en.marks.bind - dt).max(0.0);
            if en.boss || !en.active() {
                continue;
            }
            // 엉킨 뿌리(0214): 스킬에 맞은 적은 잠깐 느리게. 묶는 뿌리(뿌리 솟기 ㄴ)면 걷지 못한다.
            if en.marks.snare > 0.0 {
                en.marks.slow = en.marks.slow.min(crate::progression::cell::SNARE_SLOW);
            }
            if en.marks.bind > 0.0 {
                en.marks.slow = 0.0;
            }
            for z in &self.skill_zones {
                if z.holds(en.pos, en.radius) {
                    en.marks.slow = en.marks.slow.min(z.slow);
                }
            }
        }
        for sp in &mut self.skill_spikes {
            sp.age += dt;
        }
        self.skill_spikes.retain(|sp| sp.age < SPIKE_LIFE);
        self.tick_blades(t, dt);
        self.tick_fields(t, dt);
    }

    /// 던진 무기(G2)를 날린다: 가는 길은 곧게 `dist` m(벽·기둥·아레나 끝에 닿으면 거기서) — 끝에서 되돌거나(오는 길은 지금의
    /// 방랑자에게, 벽을 가리지 않는다 — 든 것을 잃지 않게) 터진다(박힌 자리). 길 위의 적은 길마다 한 번씩 맞는다.
    pub(super) fn tick_blades(&mut self, t: &Tuning, dt: f32) {
        if self.skill_blades.is_empty() {
            return;
        }
        let mut ev = std::mem::take(&mut self.skill_ev);
        let home = self.player.pos + Vec3::Y * (self.player.air + BLADE_HEIGHT);
        let mut keep = Vec::with_capacity(self.skill_blades.len());
        for mut b in std::mem::take(&mut self.skill_blades) {
            b.prev = b.pos;
            b.age += dt;
            let step = b.speed * dt;
            let (len, done) = if b.out {
                let mut len = step.min(b.dist - b.went);
                let mut stop = false;
                for o in &self.obstacles {
                    if let Some(d) = o.ray(b.pos, b.dir) {
                        if d <= len {
                            len = d;
                            stop = true;
                        }
                    }
                }
                let next = b.pos + b.dir * len;
                if next.x.abs() > self.arena || next.z.abs() > self.arena {
                    stop = true;
                }
                (len, stop || b.went + len >= b.dist - 1e-3)
            } else {
                let to = vec3(home.x - b.pos.x, 0.0, home.z - b.pos.z);
                let l = to.length();
                b.dir = to.normalize_or(-b.dir);
                (step.min(l), l <= step + BLADE_CATCH)
            };
            // 이 걸음의 선분에서 반 폭 안에 든 깨어 있는 적.
            let feet = b.pos.y - BLADE_HEIGHT;
            let struck: Vec<usize> = self
                .enemies
                .iter()
                .enumerate()
                .filter(|(i, en)| {
                    if !en.active()
                        || (*i < 64 && b.hits & (1 << i) != 0)
                        || (en.pos.y - feet).abs() > HIT_DY
                    {
                        return false;
                    }
                    let c = vec3(en.pos.x - b.pos.x, 0.0, en.pos.z - b.pos.z);
                    let k = c.dot(b.dir).clamp(0.0, len);
                    (c - b.dir * k).length() <= b.half + en.radius
                })
                .map(|(i, _)| i)
                .collect();
            for i in struck {
                if i < 64 {
                    b.hits |= 1 << i;
                }
                let en = &self.enemies[i];
                // 끌고 오기(ㄴ): 오는 길에 맞은 적은 방랑자 쪽으로 미끄러진다 — 보스는 끌리지 않는다(무겁다).
                let knock = if b.out {
                    b.dir * t[P::Knockback] * if en.boss { 0.1 } else { 0.5 }
                } else if b.drag > 0.0 && !en.boss && !en.rising() {
                    let to = vec3(home.x - en.pos.x, 0.0, home.z - en.pos.z);
                    let room = (to.length() - en.radius - PLAYER_RADIUS).max(0.0);
                    to.normalize_or_zero() * b.drag.min(room) * KNOCK_DECAY
                } else {
                    Vec3::ZERO
                };
                let (damage, poise) = if b.out {
                    (b.damage, b.poise)
                } else {
                    (b.back_damage, b.back_poise)
                };
                self.skill_touch(
                    t,
                    &mut ev,
                    i,
                    &Touch {
                        damage,
                        poise,
                        stagger: 0.0,
                        knock,
                        crit: true,
                        dose: b.dose,
                        flat: false,
                    },
                );
                if !b.out && b.drag > 0.0 {
                    self.enemies[i].spilling = false;
                }
            }
            b.pos += b.dir * len;
            b.went += len;
            match (b.out, done, b.stick) {
                // 박힌 자리(ㄱ): 돌아오지 않고 끝에서 터진다.
                (true, true, Some((radius, damage))) => {
                    let at = vec3(b.pos.x, feet, b.pos.z);
                    self.skill_blast(t, &mut ev, at, radius, damage, NO_DOSE, true);
                    ev.push(Event::ThrowBoom { at, radius });
                }
                (true, true, None) => {
                    b.out = false;
                    b.hits = 0;
                    ev.push(Event::ThrowTurn { at: b.pos });
                    keep.push(b);
                }
                (false, true, _) => ev.push(Event::ThrowBack { at: home }),
                _ => keep.push(b),
            }
        }
        self.skill_blades = keep;
        self.skill_ev = ev;
    }

    /// 남은 불티(G2): `every`초마다 그 안의 적을 태운다(첫 타는 `every`초 뒤 — 끝나는 순간의 타까지 치고 사라진다). 따르는 불티는
    /// 방랑자를 따라온다. 남는 불씨는 그 안에 선 적에게 표지를 적어 둔다(쓰러질 때 `tick_embers`가 읽는다).
    pub(super) fn tick_fields(&mut self, t: &Tuning, dt: f32) {
        let pp = self.player.pos;
        let mut due: Vec<Field> = Vec::new();
        let enemies = &self.enemies;
        for f in &mut self.skill_fields {
            f.age += dt;
            if f.follow {
                f.pos = pp;
            }
            // 흐르는 눈발(0270 C3 ㄱ).
            if f.drift != Vec3::ZERO {
                f.pos += f.drift * dt;
            }
            // 덫(0270 C3 — 뿌리 덫): 깨어 있는 적이 들어서면 한 번 솟고 사라진다.
            if f.trap {
                if enemies
                    .iter()
                    .any(|en| en.active() && f.holds(en.pos, en.radius))
                {
                    f.pulses += 1;
                    due.push(*f);
                    f.age = f.secs;
                }
                continue;
            }
            let next = f.every * (f.pulses + 1) as f32;
            if f.age + 1e-4 >= next && next <= f.secs + 1e-4 {
                f.pulses += 1;
                due.push(*f);
            }
        }
        for en in &mut self.enemies {
            en.kindled = self
                .skill_fields
                .iter()
                .filter(|f| f.holds(en.pos, en.radius))
                .map(|f| f.kindle)
                .max()
                .unwrap_or(0);
        }
        if !due.is_empty() {
            let mut ev = std::mem::take(&mut self.skill_ev);
            for f in due {
                for i in 0..self.enemies.len() {
                    let en = &self.enemies[i];
                    if !en.active() || !f.holds(en.pos, en.radius) {
                        continue;
                    }
                    self.skill_touch(
                        t,
                        &mut ev,
                        i,
                        &Touch {
                            damage: f.damage,
                            poise: f.poise,
                            stagger: 0.0,
                            knock: Vec3::ZERO,
                            crit: false,
                            dose: f.dose,
                            flat: false,
                        },
                    );
                }
                if f.trap {
                    // 솟는 덫: 씨 자리와 둘레에서 뿌리가 솟는다(그림 — 뿌리 솟기의 뿌리와 같은 것).
                    let g = vec3(f.pos.x, self.ground_y(f.pos), f.pos.z);
                    for k in 0..=TRAP_SPIKES {
                        let a = k as f32 / TRAP_SPIKES as f32 * std::f32::consts::TAU;
                        let r = if k == TRAP_SPIKES {
                            0.0
                        } else {
                            f.radius * 0.6
                        };
                        self.skill_spikes.push(Spike {
                            pos: g + vec3(a.sin() * r, 0.0, a.cos() * r),
                            age: 0.0,
                            lean: vec3(a.sin() * 0.15, 0.0, a.cos() * 0.15),
                        });
                    }
                    ev.push(Event::RootSpike { at: g });
                }
                ev.push(Event::FieldPulse {
                    at: f.pos,
                    radius: f.radius,
                    kind: f.dose.0,
                });
                // 덫의 마지막 가시(뿌리 덫의 맺음): 솟은 자리가 크게 한 번 더.
                if let Some((radius, damage)) = f.spire {
                    let at = f.pos;
                    self.skill_blast(t, &mut ev, at, radius, damage, NO_DOSE, true);
                    ev.push(Event::RootSpike { at });
                }
            }
            self.skill_ev = ev;
        }
        self.skill_fields.retain(|f| f.age + 1e-4 < f.secs);
    }

    /// 새 스킬들의 타 하나(G2 — 던진 무기·불티·울림·한 줄기): 피해는 여느 타와 같은 길(`hurt_by` — 장신구의 덤·깨진 동안의 배·
    /// 더 다치는 표지), 자세 피해, 쓰러짐, 휘청(보스·전환 중인 보스는 휘청이지 않는다 — 이미 더 길게 휘청이는 적은 그대로).
    /// 스킬의 타라 큰 기술 게이지를 채우지 않고(`connected`를 건드리지 않는다), 쓰러뜨리면 스킬 몫의 기세. 쓰러뜨렸으면 참.
    pub(in crate::world) fn skill_touch(
        &mut self,
        t: &Tuning,
        ev: &mut Vec<Event>,
        i: usize,
        c: &Touch,
    ) -> bool {
        let snare = self.snare_secs();
        let en = &mut self.enemies[i];
        if !en.boss {
            en.marks.snare = en.marks.snare.max(snare);
        }
        let at = en.pos + Vec3::Y * en.height * 0.6;
        if c.damage > 0 {
            let crit = c.crit && en.exposed > 0.0;
            let bonus = if c.flat { 0 } else { self.stats.dmg };
            en.hp -= hurt_by(en, bonus + c.damage, crit, self.stats.opened_dmg, t);
            if crit {
                en.exposed = 0.0;
                ev.push(Event::Crit { at });
            }
            en.struck();
            ev.push(Event::Hit { at });
            self.skill_hit_tick = true;
        }
        if c.knock != Vec3::ZERO {
            en.knock = c.knock;
            en.sway = c.knock.normalize_or_zero() * HIT_SWAY;
            en.sway_t = 0.0;
        }
        add_poise(en, c.poise * self.stats.poise, None, t, ev);
        if en.hp <= 0 {
            en.state = EState::Dead;
            self.kills += 1;
            ev.push(Event::EnemyDown { at: en.pos });
            return true;
        }
        afflict(en, c.dose, ev);
        if c.stagger > 0.0 && !en.boss && !en.rising() {
            let longer = matches!(en.state, EState::Stagger { t: e, dur } if dur - e >= c.stagger);
            if !longer {
                en.state = EState::Stagger {
                    t: 0.0,
                    dur: c.stagger,
                };
            }
        }
        false
    }

    /// 둘레 터짐(G2 — 던지기의 박힌 자리 · 0223 V2 화상 중에 묶여 터짐): `at` 둘레 `radius` m의 깨어 있는 적에게 피해, 바깥으로 민다(보스는 조금).
    #[allow(clippy::too_many_arguments)]
    pub(in crate::world) fn skill_blast(
        &mut self,
        t: &Tuning,
        ev: &mut Vec<Event>,
        at: Vec3,
        radius: f32,
        damage: i32,
        dose: Dose,
        own: bool,
    ) {
        let mut n = 0;
        for i in 0..self.enemies.len() {
            let en = &self.enemies[i];
            if !en.active() || (en.pos.y - at.y).abs() > HIT_DY {
                continue;
            }
            let dv = vec3(en.pos.x - at.x, 0.0, en.pos.z - at.z);
            if dv.length() - en.radius > radius {
                continue;
            }
            let knock = dv.normalize_or_zero() * t[P::Knockback] * if en.boss { 0.2 } else { 1.2 };
            self.skill_touch(
                t,
                ev,
                i,
                &Touch {
                    damage,
                    poise: 0.0,
                    stagger: 0.0,
                    knock,
                    // 스킬이 낸 터짐은 타의 덤·드러난 틈의 치명타를 받고, 타입이 스스로 낸 터짐(`own`이 거짓)은 화상·울림처럼
                    // 덤 없이 — 드러난 틈도 쓰지 않는다.
                    crit: own,
                    dose,
                    flat: !own,
                },
            );
            n += 1;
        }
        if n > 0 {
            self.hitstop = self.hitstop.max(t[P::Hitstop] * SKILL_HITSTOP);
        }
    }

    /// 되울림(G2): 울림이 남아 있는 동안 성공한 패링 `parries`번마다 둘레의 적이 휘청인다(보스는 아니다). 패링 창·받는 반원은
    /// 건드리지 않는다(0095 원칙 2) — 이미 성공한 패링에 얹힐 뿐. 고르기: 자세 피해(ㄱ)·모든 대기 당기기(ㄴ), 맺음: 피해.
    pub(in crate::world) fn echo_ring(&mut self, t: &Tuning, parries: usize, ev: &mut Vec<Event>) {
        if parries == 0 || self.player.skill.echo_left <= 0.0 {
            return;
        }
        let d = self.skill_eff(skills::Id::Echo);
        let Form::Echo {
            radius, stagger, ..
        } = d.skill.form
        else {
            return;
        };
        let poise = match d.twist {
            Some(Twist::Rattle { poise }) => poise,
            _ => 0.0,
        };
        let damage = match d.cap {
            Some(Cap::Toll { damage }) => damage,
            _ => 0,
        };
        let p = self.player.pos;
        let body = p.y + self.player.air;
        for _ in 0..parries {
            for i in 0..self.enemies.len() {
                let en = &self.enemies[i];
                if !en.active() || (en.pos.y - body).abs() > HIT_DY {
                    continue;
                }
                let dv = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z);
                if dv.length() - en.radius > radius {
                    continue;
                }
                self.skill_touch(
                    t,
                    ev,
                    i,
                    &Touch {
                        damage,
                        poise,
                        stagger,
                        knock: Vec3::ZERO,
                        crit: false,
                        dose: self.dosed(d.skill.dose()),
                        flat: false,
                    },
                );
            }
            if let Some(Twist::Tempo { secs }) = d.twist {
                for cd in &mut self.player.skill.cd {
                    *cd = (*cd - secs).max(0.0);
                }
            }
        }
        ev.push(Event::EchoRing { at: p, radius });
    }
}
