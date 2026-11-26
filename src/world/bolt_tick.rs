//! 날아가는 것·늦게 터지는 것의 틱(0159 R2): 한 함수였던 `tick_bolts`(309줄)를 단계마다 메서드로 — 순서와 몸은 그대로다.
//!
//! 순서: 틱이 남긴 알림 → 자세 막대 → 폭발 고리·불똥의 나이 → 쏜 탄·공중 RB·던진 수류탄을 세움 → 탄이 난다 → 수류탄 → 금 간 땅.
use super::*;
use crate::tuning::{
    BOLT_BODY_SLACK, BOLT_FOLLOW, BOLT_STEP, JAVELIN_BACK, JAVELIN_CATCH, SMITE_DELAY,
    SMITE_RADIUS, SMITE_SEEK, SMITE_SHOW,
};

impl World {
    /// 날아가는 것·늦게 터지는 것(0048·0051): 마법 탄·쇠뇌 볼트, 수류탄, 금 간 땅(망치 여진), 자세 막대가 줄기, 폭발 고리.
    /// 탄은 이번 틱에 쏜 것을 세우고 날려 벽·땅·적에 닿으면 지운다(꿰뚫는 탄은 적을 지나간다). 칸은 채우지 않는다(근접만 채운다).
    pub(super) fn tick_bolts(&mut self, t: &Tuning, dt: f32, ev: &mut Vec<Event>) {
        if std::mem::take(&mut self.notes.beat) {
            ev.push(Event::Beat);
        }
        if std::mem::take(&mut self.notes.cast_denied) {
            ev.push(Event::CastDenied);
        }
        if std::mem::take(&mut self.player.denied_new) {
            ev.push(Event::StaminaDenied);
        }
        let wpn = self.player.weapon.def();
        self.tick_poise(t, dt);
        self.tick_blasts_and_sparks(dt);
        self.fire_cast(t, ev, wpn);
        self.fire_air(t, ev, wpn);
        self.throw_bomb(ev, wpn);
        self.fly_bolts(t, dt, ev);

        self.tick_bombs(t, dt, ev);
        self.tick_cracks(t, dt, ev);
        self.tick_smites(t, dt, ev);
    }

    fn tick_poise(&mut self, t: &Tuning, dt: f32) {
        // 자세 막대(0051): 깬 뒤 잠금이 풀리고, 맞지 않은 지 오래면 줄어든다.
        for en in &mut self.enemies {
            en.stance.lock = (en.stance.lock - dt).max(0.0);
            en.stance.broken = (en.stance.broken - dt).max(0.0);
            en.stance.frac = (en.stance.poise / poise_max(en, t).max(0.01)).clamp(0.0, 1.0);
            en.stance.idle += dt;
            if en.stance.idle >= t[P::PoiseDelay] {
                en.stance.poise = (en.stance.poise - t[P::PoiseDecay] * dt).max(0.0);
            }
        }
    }

    /// 폭발 고리·맞힌 자리 불똥의 나이.
    fn tick_blasts_and_sparks(&mut self, dt: f32) {
        for b in &mut self.blasts {
            b.age += dt;
        }
        self.blasts.retain(|b| b.age < BLAST_SHOW);
        for sp in &mut self.sparks {
            sp.age += dt;
            sp.vel.y -= 18.0 * dt;
            sp.vel *= (1.0 - 6.0 * dt).max(0.0);
            sp.pos += sp.vel * dt;
        }
        self.sparks.retain(|sp| sp.age < SPARK_LIFE);
    }

    /// 이번 틱에 쏜 RB를 세운다(모은 정도마다 다른 탄, 0056·0078): 근접 무기 = 종류의 원거리 수단(0274 ②b — 신성한 빛은 날지 않고
    /// 겨눈 자리에 `Smite`로), 사격 무기 = 볼트·화살(활의 반쯤 당긴 화살은 표의 `mid_bolt`).
    fn fire_cast(&mut self, t: &Tuning, ev: &mut Vec<Event>, _wpn: &weapons::Weapon) {
        if let Some((dir, power)) = self.notes.cast_fire.take() {
            let (plain, c) = self.rb_bolts();
            // 가벼운 걸음(0276 — 활의 빈 손): 끝까지 당긴 화살의 오금이 길다.
            let light_step = self.bare() == Some(weapons::Bare::LightStep);
            let pl = &self.player;
            let pos = pl.pos + Vec3::Y * (pl.air + BOLT_HEIGHT) + dir * (PLAYER_RADIUS + 0.2);
            let def = match power {
                0 => plain,
                // 반쯤 당긴 화살(0274 ②b).
                1 if pl.weapon.draws() => match pl.weapon.rhythm() {
                    Some(weapons::Rhythm::Draw { mid_bolt, .. }) => mid_bolt,
                    Some(weapons::Rhythm::Crank { .. }) | None => plain,
                },
                // 덜 모은 탄(0056): 모은 탄의 반(옛 눈금으로 올림, 0158 N0) — 꿰뚫기·휘청 없음.
                1 => weapons::Bolt {
                    damage: crate::tuning::half_up(c.damage),
                    poise: c.poise * 0.5,
                    pierce: false,
                    stagger: 0.0,
                    scale: 1.0 + (c.scale - 1.0) * 0.4,
                    dose: (c.dose.0, c.dose.1 * 0.5),
                    lame: 0.0,
                    ..c
                },
                // 딱 맞춰 뗀 모은 탄(0078): 강타의 딱 맞춤과 같은 덤 — 피해 +한 눈금(`HP_UNIT`)·자세 피해 ×1.5.
                3 => weapons::Bolt {
                    damage: c.damage + HP_UNIT,
                    poise: c.poise * 1.5,
                    ..c
                },
                _ => c,
            };
            let def = if light_step && def.lame > 0.0 {
                weapons::Bolt {
                    lame: weapons::BARE_BOW_LAME,
                    ..def
                }
            } else {
                def
            };
            if power == 3 {
                self.player.beat.since_perfect = 0.0;
                ev.push(Event::Perfect { at: pos });
            }
            ev.push(Event::Cast {
                at: pos,
                look: def.look,
            });
            // 신성한 빛(0274 ②b): 날지 않는다 — 겨눈 자리(미리보기와 같은 셈 `smite_spot`)에 내려 꽂힌다.
            if def.smites() {
                let at = self.smite_spot(dir, def, t);
                self.smites.push(smite_of(at, pos, &def));
                return;
            }
            let left = t[P::CastRange] * def.range;
            // 좁은 부채꼴(0274 ③ — 투척 단검 셋): 겨눈 쪽을 가운데로 `spread`°씩 벌려 하나씩 — 나머지 종류는 하나.
            let (fan, spread) = self
                .rb_toss()
                .map_or((1, 0.0), |t| (t.fan.max(1), t.spread));
            for k in 0..fan {
                let off = (k as f32 - (fan - 1) as f32 * 0.5) * spread;
                let d = Quat::from_rotation_y(off.to_radians()) * dir;
                let d = self.shot_line(pos, d, &def, left);
                self.bolts.push(Bolt {
                    pos,
                    prev: pos,
                    dir: d,
                    left,
                    def,
                    hits: 0,
                    veined: [false; 5],
                    back: false,
                    bounce: 0,
                });
            }
        }
    }

    /// 신성한 빛이 내려 꽂힐 자리(0274 ②b — 쏠 때와 미리보기가 같은 셈): `flat` 쪽으로 마법 탄이 가는 길(`shot_reach` — 벽·벼랑에서
    /// 멈춘다) 위에서 줄(몸 + `SMITE_SEEK` m) 안의 첫 적 발밑, 없으면 그 길이 멈추는 자리. 높이는 그 자리의 땅.
    pub fn smite_spot(&self, flat: Vec3, def: weapons::Bolt, t: &Tuning) -> Vec3 {
        let p = self.player.pos;
        let reach = self.shot_reach(flat, def, t);
        let first = self
            .enemies
            .iter()
            .filter(|en| en.active() && !en.harmless())
            .filter_map(|en| {
                let d = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z);
                let along = d.dot(flat);
                let off = (d - flat * along).length();
                (along > 0.0 && along <= reach + en.radius && off <= en.radius + SMITE_SEEK)
                    .then_some((along, en.pos))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0));
        let at = first.map_or(p + flat * reach, |(_, q)| q);
        vec3(at.x, self.ground_y(at), at.z)
    }

    /// 지금 떼면(누르면) 나갈 RB의 탄(0274 ②b — 미리보기가 쏠 때와 같은 것을 그리게): 모으는 중이면 모은 것, 활은 당긴 단계
    /// (톡 · 반쯤 · 끝까지), 아니면 보통 한 번.
    pub fn aim_bolt(&self, t: &Tuning) -> weapons::Bolt {
        let (plain, charged) = self.rb_bolts();
        match self.player.state {
            PState::CastCharge { t: e } => match self.player.weapon.rhythm() {
                Some(weapons::Rhythm::Draw { mid, mid_bolt, .. }) => {
                    if e >= t[P::CastCharge] {
                        charged
                    } else if e >= mid {
                        mid_bolt
                    } else {
                        plain
                    }
                }
                Some(weapons::Rhythm::Crank { .. }) | None => charged,
            },
            _ => plain,
        }
    }

    /// 던진 창이 날고 있나(0274 ②b — 가는 길이든 돌아오는 길이든). 그동안 RB는 못 쓰고 손에는 창이 없다(`weapon_away`).
    pub fn javelin_out(&self) -> bool {
        self.bolts
            .iter()
            .any(|b| b.def.look == weapons::Look::Javelin)
    }

    /// 평타를 누르면 던진 창이 곧바로 손에 돌아온다(0274 ②b — 창이 없는 동안에도 평타를 막지 않는다).
    /// 던진 방패(0276)도 같은 길로 부른다.
    pub(super) fn recall_javelin(&mut self) {
        if !self.rb_out() {
            return;
        }
        self.bolts.retain(|b| !b.def.returns());
        let pl = &self.player;
        self.skill_ev.push(Event::JavelinBack {
            at: pl.pos + Vec3::Y * (pl.air + BOLT_HEIGHT),
        });
    }

    /// 곧 내려 꽂힐 신성한 빛(0274 ②b): 시간이 되면 둘레를 친다(울림을 쌓는다).
    fn tick_smites(&mut self, t: &Tuning, dt: f32, ev: &mut Vec<Event>) {
        if self.smites.is_empty() {
            return;
        }
        let mut due = Vec::new();
        for s in &mut self.smites {
            s.left -= dt;
            if s.left <= 0.0 && !s.struck {
                s.struck = true;
                due.push(*s);
            }
        }
        self.smites.retain(|s| s.left > -SMITE_SHOW);
        for s in due {
            self.burst(s.pos, s.radius, s.damage, s.poise, s.dose, t, ev);
            if s.look == weapons::Look::Seed {
                // 가시 씨앗(0274 ⑤): 떨어진 자리 둘레에 가시 뿌리가 솟는다 — 그림은 솟은 뿌리(`skill_spikes`, 뿌리 솟기와 같은 것).
                self.sprout_ring(s.pos, s.radius);
                ev.push(Event::Sprout {
                    at: s.pos,
                    radius: s.radius,
                });
            } else {
                ev.push(Event::Smite {
                    at: s.pos,
                    radius: s.radius,
                });
            }
        }
    }

    /// 둘레 `radius` m의 고리에 가시 뿌리를 세운다(그림만 — 가시 씨앗 · 깨어나는 숲 0274 ⑤): 가운데 하나 + 둘레에 고르게(둘레 1.1 m마다
    /// 하나, 여섯에서 스물넷), 바깥으로 기울게.
    pub(super) fn sprout_ring(&mut self, at: Vec3, radius: f32) {
        let n = ((std::f32::consts::TAU * radius / 1.1).round() as usize).clamp(6, 24);
        let phase = (at.x * 1.7 + at.z * 0.9).fract() * std::f32::consts::TAU;
        let mut spikes = vec![Spike {
            pos: vec3(at.x, self.ground_y(at), at.z),
            age: 0.0,
            lean: Vec3::ZERO,
        }];
        for k in 0..n {
            let a = phase + k as f32 / n as f32 * std::f32::consts::TAU;
            let out = vec3(a.sin(), 0.0, a.cos());
            let r = radius * if k % 2 == 0 { 0.85 } else { 0.55 };
            let p = at + out * r;
            spikes.push(Spike {
                pos: vec3(p.x, self.ground_y(p), p.z),
                age: 0.0,
                lean: out * 0.2,
            });
        }
        self.skill_spikes.extend(spikes);
    }

    fn fire_air(&mut self, t: &Tuning, ev: &mut Vec<Event>, wpn: &weapons::Weapon) {
        // 공중 RB(0111): 쏜 높이에서 겨눈 적(쏘는 쪽 ±15°·`cast.range m` 안 가장 가까운)의 몸통으로, 없으면 앞
        // `AIR_SHOT_REACH`m 땅 위 몸통 높이로 비스듬히 내려 쏜다. 쇠뇌는 같은 자리로 볼트 `AIR_VOLLEY`발을 부채꼴로.
        if let Some(flat) = self.notes.air_fire.take() {
            let pl = &self.player;
            let from = pl.pos + Vec3::Y * (pl.air + BOLT_HEIGHT) + flat * (PLAYER_RADIUS + 0.2);
            let cone = 15f32.to_radians().cos();
            let reach = self
                .enemies
                .iter()
                .filter(|en| en.active())
                .filter_map(|en| {
                    let d = vec3(en.pos.x - pl.pos.x, 0.0, en.pos.z - pl.pos.z);
                    let n = d.length();
                    (n <= t[P::CastRange] && n > 1e-3 && d.dot(flat) / n >= cone).then_some(n)
                })
                .min_by(f32::total_cmp)
                .unwrap_or(AIR_SHOT_REACH)
                .max(AIR_SHOT_MIN);
            let def = self.rb_bolts().0;
            let count = match wpn.rb {
                Rb::Toss => 1,
                Rb::Shot { .. } => AIR_VOLLEY,
            };
            // 신성한 빛(0274 ②b)은 공중에서도 날지 않는다 — 겨눈 자리(내려 쏘는 탄이 가는 땅)에 내려 꽂힌다.
            if def.smites() {
                let aim = pl.pos + flat * reach;
                let at = vec3(aim.x, self.ground_y(aim), aim.z);
                self.smites.push(smite_of(at, from, &def));
                ev.push(Event::Cast {
                    at: from,
                    look: def.look,
                });
                return;
            }
            let spread = AIR_VOLLEY_SPREAD_DEG.to_radians();
            for i in 0..count {
                let k = if count > 1 {
                    i as f32 / (count - 1) as f32 * 2.0 - 1.0
                } else {
                    0.0
                };
                let (sn, cs) = (spread * k).sin_cos();
                let side = vec3(flat.x * cs + flat.z * sn, 0.0, -flat.x * sn + flat.z * cs);
                let aim = pl.pos + side * reach;
                let floor = self.ground.as_ref().map_or(pl.pos.y, |gr| gr.height(aim));
                let to = vec3(aim.x, floor + BOLT_HEIGHT * 0.6, aim.z);
                let dir = (to - from).normalize_or(side);
                self.bolts.push(Bolt {
                    pos: from,
                    prev: from,
                    dir,
                    left: t[P::CastRange] * def.range,
                    def,
                    hits: 0,
                    veined: [false; 5],
                    back: false,
                    bounce: 0,
                });
            }
            ev.push(Event::Cast {
                at: from,
                look: def.look,
            });
        }
    }

    /// 이번 틱에 던진 수류탄(0051)을 세운다.
    fn throw_bomb(&mut self, ev: &mut Vec<Event>, wpn: &weapons::Weapon) {
        if let (Some(to), Some(g)) = (self.notes.throw_to.take(), wpn.grenade) {
            // 장치를 하나 더(0276 — 쇠뇌의 빈 손): 수류탄 대신 그 자리에 쇠 덫을 놓는다.
            if self.bare() == Some(weapons::Bare::Snare) {
                self.set_snare(to, ev);
                return;
            }
            let bomb = self.new_bomb(to, &g);
            self.bombs.push(bomb);
            ev.push(Event::Throw);
        }
    }

    /// 쇠 덫을 놓는다(0276): 덫(`skill_tick::Field` — 뿌리 덫과 같은 길: 깨어 있는 적이 둘레에 들어서면 한 번 물고 사라진다)에
    /// 쇠(`iron`) — 무는 타는 `SNARE_DAMAGE` · 뿌리 게이지 `SNARE_ROOT`(보통 적은 한 번에 속박, 문지기는 덜 — 종의 배). 놓인 쇠 덫은
    /// `SNARE_MAX`개까지(넘으면 가장 오래된 것이 사라진다).
    fn set_snare(&mut self, to: Vec3, ev: &mut Vec<Event>) {
        use crate::tuning::{
            SNARE_DAMAGE, SNARE_MAX, SNARE_POISE, SNARE_RADIUS, SNARE_ROOT, SNARE_SECS,
        };
        let pos = vec3(to.x, self.ground_y(to), to.z);
        let mine: Vec<usize> = self
            .skill_fields
            .iter()
            .enumerate()
            .filter(|(_, f)| f.iron())
            .map(|(i, _)| i)
            .collect();
        if mine.len() >= SNARE_MAX {
            self.skill_fields.remove(mine[0]);
        }
        self.skill_fields.push(skill_tick::Field {
            kind: skill_tick::FieldKind::Trap {
                iron: true,
                spire: None,
            },
            ..skill_tick::Field::lay(
                pos,
                SNARE_RADIUS,
                SNARE_SECS,
                SNARE_SECS,
                SNARE_DAMAGE,
                SNARE_POISE,
                (crate::status::Kind::Root, SNARE_ROOT),
            )
        });
        ev.push(Event::SnareSet { at: pos });
    }

    /// 손을 떠나는 수류탄(0051): 포물선 — 날아가는 시간을 거리로 정하고(가까우면 낮게·빨리) 그 시간에 `to`에 닿는 속도.
    fn new_bomb(&self, to: Vec3, g: &weapons::Grenade) -> Bomb {
        let pl = &self.player;
        let from = pl.pos + Vec3::Y * (pl.air + BOLT_HEIGHT);
        let ground_to = self.ground.as_ref().map_or(0.0, |gr| gr.height(to));
        let flat = vec3(to.x - from.x, 0.0, to.z - from.z);
        let tf = (flat.length() / 10.0).clamp(0.3, 0.9);
        let vy = (ground_to + 0.2 - from.y + 0.5 * BOMB_GRAVITY * tf * tf) / tf;
        Bomb {
            pos: from,
            prev: from,
            vel: flat / tf + Vec3::Y * vy,
            fuse: g.fuse,
            radius: g.radius,
            damage: g.damage + self.stats.shot_dmg,
            poise: g.poise,
            root: g.root,
        }
    }

    /// 수류탄이 한 걸음 구른다: 중력, 벽에서 되튐, 땅에서 튕김(줄어드는 위·옆 속도). 틱과 미리보기(`grenade_lands`)가 같이 쓴다.
    fn roll_bomb(&self, bomb: &mut Bomb, dt: f32) {
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
    }

    /// 지금 `to`로 던지면 터지는 자리(0218 — 미리보기가 진실하게): 같은 포물선·같은 튕김을 도화선이 다할 때까지 돌려 본다.
    pub fn grenade_lands(&self, to: Vec3) -> Option<Vec3> {
        let g = self.player.weapon.def().grenade?;
        // 쇠 덫(0276)은 튕기지 않는다 — 떨군 자리에 놓인다.
        if self.bare() == Some(weapons::Bare::Snare) {
            return Some(vec3(to.x, self.ground_y(to), to.z));
        }
        let mut bomb = self.new_bomb(to, &g);
        while bomb.fuse > 0.0 {
            self.roll_bomb(&mut bomb, TICK_DT);
        }
        Some(bomb.pos)
    }

    /// 쇠뇌 볼트의 길(0218 — 사장 "땅을 타고 가면 안 되고, 직선이어야지"): 수평으로 겨눈 쪽(`flat`)의 줄 위에 선 첫 적이 있으면
    /// 그 몸통 높이로 비스듬히 곧게, 없으면 쏜 높이 그대로 곧게. 화면 위의 쪽(수평)은 그대로라 조준 표시와 어긋나지 않는다.
    /// 마법 탄은 그대로 땅을 따라 흐른다(0173) — 수평 쪽을 돌려준다.
    pub(super) fn shot_line(
        &self,
        from: Vec3,
        flat: Vec3,
        def: &weapons::Bolt,
        reach: f32,
    ) -> Vec3 {
        if !def.straight() || flat.y.abs() > 1e-4 {
            return flat;
        }
        let first = self
            .enemies
            .iter()
            .filter(|en| en.active() && !en.harmless())
            .filter_map(|en| {
                let d = vec3(en.pos.x - from.x, 0.0, en.pos.z - from.z);
                let along = d.dot(flat);
                let off = (d - flat * along).length();
                (along > 0.0 && along <= reach && off <= en.radius + BOLT_RADIUS * def.scale)
                    .then_some((along, en.pos.y + en.height * 0.5))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0));
        // 0274 ②b: 같은 땅의 적이면(몸통이 쏜 높이에서 몸 판정의 여유 안) 기울이지 않는다 — 꿰뚫는 것(던진 창 · 모은 볼트)이 첫 적의
        // 몸통으로 조금 숙여 날다 그 뒤의 땅에 박히던 것. 높낮이가 다른 적에게만 비스듬히(0218 그대로).
        match first {
            Some((along, y)) if (y - from.y).abs() > BOLT_BODY_SLACK => {
                (flat * along + Vec3::Y * (y - from.y)).normalize_or(flat)
            }
            _ => flat,
        }
    }

    /// 지금 `flat` 쪽으로 쏘면 탄이 멈추는 데까지의 수평 거리(0218 — 미리보기가 진실하게): 쏠 때와 같은 자리·같은 길로,
    /// 같은 벽·땅 규칙을 돌려 본다(적은 빼고 — 맞으면 거기서 멈추는 것은 보인다).
    pub fn shot_reach(&self, flat: Vec3, def: weapons::Bolt, t: &Tuning) -> f32 {
        let pl = &self.player;
        let from = pl.pos + Vec3::Y * (pl.air + BOLT_HEIGHT) + flat * (PLAYER_RADIUS + 0.2);
        let left = t[P::CastRange] * def.range;
        let mut b = Bolt {
            pos: from,
            prev: from,
            dir: self.shot_line(from, flat, &def, left),
            left,
            def,
            hits: 0,
            veined: [false; 5],
            back: false,
            bounce: 0,
        };
        let step = t[P::CastSpeed] * TICK_DT * def.speed;
        for _ in 0..2000 {
            let mut len = step.min(b.left);
            let mut gone = false;
            for o in &self.obstacles {
                if let Some(d) = o.ray(b.pos, b.dir) {
                    if d <= len {
                        len = d;
                        gone = true;
                    }
                }
            }
            b.pos += b.dir * len;
            b.left -= len;
            gone |= self.bolt_lands(&mut b, len);
            if gone || b.left <= 1e-3 {
                break;
            }
        }
        vec2(b.pos.x - pl.pos.x, b.pos.z - pl.pos.z).length()
    }

    /// 지금 든 무기의 탄이 그 적(`at`, 몸 반지름 `radius`)에게 닿나(0228 — 사장 "벽 너머의 적에게 조준 보정이 들어가는 건 아니지?"):
    /// 그 쪽으로 쏘면 멈추는 거리(`shot_reach` — 쏠 때와 같은 길·벽·땅 규칙)가 몸까지 가면 참. 조준 보정(패드 보조·크로스헤어·
    /// 가까운 적 찾기)은 이것이 거짓인 적에게 붙지 않는다 — 벽에 대고 쏘게 하는 보정은 보정이 아니다.
    pub fn shot_clear(&self, at: Vec3, radius: f32, t: &Tuning) -> bool {
        let p = self.player.pos;
        let v = vec3(at.x - p.x, 0.0, at.z - p.z);
        let n = v.length();
        if n < 1e-3 {
            return true;
        }
        let def = self.rb_bolts().0;
        self.shot_reach(v / n, def, t) + radius >= n - 0.05
    }

    /// 걸음을 마친 탄과 땅(0173·0218): 마법 탄(수평)은 땅을 따라 흐르고 높은 벼랑·벽 칸에서 사라진다. 쇠뇌 볼트와 내려 쏜 탄은
    /// 곧게 가다 땅에 닿으면 사라진다. 사라지면 참.
    fn bolt_lands(&self, b: &mut Bolt, len: f32) -> bool {
        if let Some(g) = &self.ground {
            let floor = g.cell_of(b.pos).and_then(|(x, z)| g.height_in(x, z, b.pos));
            let level = b.dir.y.abs() < 1e-4 && !b.def.straight();
            match floor {
                None => true,
                Some(h) if level => {
                    let want = h + BOLT_HEIGHT;
                    if want - b.pos.y > BOLT_STEP {
                        true
                    } else {
                        let reach = (len * BOLT_FOLLOW).max(0.02);
                        b.pos.y += (want - b.pos.y).clamp(-reach, reach);
                        // 막 넘은 턱의 모서리에 묻히지 않게.
                        b.pos.y = b.pos.y.max(h + 0.15);
                        false
                    }
                }
                Some(h) => h > b.pos.y - 0.1,
            }
        } else {
            // 평평한 시험 판: 내려 쏜 탄(0111)은 바닥(0)에 닿으면 사라진다.
            b.pos.y < 0.1
        }
    }

    /// 탄이 난다: 벽·기둥·수류탄·적(꿰뚫는 탄은 모두, 아니면 가장 가까운 하나)·땅에 닿으면 사라진다.
    fn fly_bolts(&mut self, t: &Tuning, dt: f32, ev: &mut Vec<Event>) {
        let step = t[P::CastSpeed] * dt;
        let pl = &self.player;
        let home = pl.pos + Vec3::Y * (pl.air + BOLT_HEIGHT);
        let mut keep = Vec::with_capacity(self.bolts.len());
        for mut b in std::mem::take(&mut self.bolts) {
            b.prev = b.pos;
            // 돌아오는 창(0274 ②b): 지금의 방랑자에게 곧게(벽을 가리지 않는다 — 든 것을 잃지 않게), 치지 않는다.
            if b.back {
                let to = home - b.pos;
                let l = to.length();
                let go = JAVELIN_BACK * dt;
                if l <= go + JAVELIN_CATCH {
                    ev.push(Event::JavelinBack { at: home });
                } else {
                    b.dir = to / l;
                    b.pos += b.dir * go;
                    keep.push(b);
                }
                continue;
            }
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
            if b.def.look == weapons::Look::Bolt {
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
                // 높이: 탄이 몸의 높이 안(위아래 `BOLT_BODY_SLACK` 여유 — 땅을 따라 흐르는 탄이 턱·경사로를 넘는 사이에도 맞게).
                if !en.active()
                    || (i < 64 && b.hits & (1 << i) != 0)
                    || b.pos.y < en.pos.y - BOLT_BODY_SLACK
                    || b.pos.y > en.pos.y + en.height + BOLT_BODY_SLACK
                {
                    continue;
                }
                let c = vec3(en.pos.x, b.pos.y, en.pos.z);
                let k = (c - b.pos).dot(b.dir).clamp(0.0, len);
                if (b.pos + b.dir * k).distance(c) <= en.radius + BOLT_RADIUS * b.def.scale {
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
                let knack = self.knack();
                let en = &mut self.enemies[i];
                // 등딱지(0120): 앞에서 온 탄도 튕긴다(탄은 거기서 멈춘다 — 방랑자는 튀지 않는다).
                if en.guards(-b.dir) {
                    ev.push(Event::Deflect {
                        at: b.pos + b.dir * k,
                    });
                    continue;
                }
                // 결(0223 11절): 탄도 무기의 타다 — 묶인 적·금 간 적에 닿았나(탄 하나에 한 번 — 꿰뚫어도).
                if !b.veined[0] && (en.ail.bound > 0.0 || en.marks.bind > 0.0) {
                    b.veined[0] = true;
                    self.veins.bound = true;
                }
                if !b.veined[1] && en.ail.rift > 0.0 {
                    b.veined[1] = true;
                    self.veins.cracked = true;
                }
                // 0270 C3: 기절한 적 · 피 흘리는 적 · 흡수 걸린 적.
                if !b.veined[2] && en.ail.stun > 0.0 {
                    b.veined[2] = true;
                    self.veins.rattle = Some(en.pos);
                }
                if !b.veined[3] && en.ail.bleed > 0.0 {
                    b.veined[3] = true;
                    self.veins.bled = true;
                }
                if !b.veined[4] && en.ail.drain > 0.0 {
                    b.veined[4] = true;
                    self.veins.sipped = true;
                }
                let crit = en.exposed > 0.0;
                en.hp -= wound(
                    en,
                    self.stats.dmg
                        + b.def.damage
                        + if b.def.is_shot() {
                            self.stats.shot_dmg
                        } else {
                            0
                        },
                    crit,
                    self.stats.opened_dmg,
                    knack,
                    t,
                    ev,
                );
                en.exposed = 0.0;
                en.struck();
                en.knock = b.dir * t[P::Knockback] * if en.boss { 0.1 } else { 0.5 };
                self.notes.bolt_hit = true;
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
                add_poise(en, b.def.poise * self.stats.poise, None, t, ev);
                // 종류의 RB가 싣는 속성(0274 ②b)과 끝까지 당긴 화살의 오금.
                afflict(en, b.def.dose, ev);
                // 계약의 표(0274 ⑥ — 완드의 빈 손): 저주탄에 맞은 적에 표가 남는다(쓰러지면 값이 돌아오는 것은 빈 손일 때만 — `tick_embers`).
                if b.def.look == weapons::Look::Curse {
                    en.marks.pact = weapons::PACT_MARK_SECS;
                }
                if b.def.lame > 0.0 && en.hp > 0 {
                    en.marks.hamstring = en.marks.hamstring.max(b.def.lame);
                }
                if en.hp <= 0 {
                    en.state = EState::Dead;
                    self.kills += 1;
                    ev.push(Event::EnemyDown { at: en.pos });
                }
            }
            // 던진 방패(0276 — 하데스의 방패): 맞힌 적에서 둘레 `SHIELD_BOUNCE_R` m 안의 아직 안 맞은 가장 가까운 적으로 튕긴다
            // (`SHIELD_BOUNCES`번까지) — 더 갈 적이 없으면 거기서 손으로 돌아온다.
            if b.def.bounces() && !struck.is_empty() {
                let (_, k) = struck[0];
                let at = b.pos + b.dir * k;
                let next = (b.bounce < SHIELD_BOUNCES)
                    .then(|| {
                        self.enemies
                            .iter()
                            .enumerate()
                            .filter(|(i, en)| {
                                en.active()
                                    && !en.harmless()
                                    && !(*i < 64 && b.hits & (1 << *i) != 0)
                                    && (en.pos.y - (at.y - BOLT_HEIGHT)).abs() <= HIT_DY
                            })
                            .map(|(_, en)| (vec3(en.pos.x - at.x, 0.0, en.pos.z - at.z), en.radius))
                            .filter(|(d, r)| d.length() - r <= SHIELD_BOUNCE_R)
                            .min_by(|a, b| a.0.length().total_cmp(&b.0.length()))
                    })
                    .flatten();
                b.prev = b.pos;
                b.pos = at;
                match next {
                    Some((d, _)) if d.length() > 1e-3 => {
                        b.dir = d.normalize();
                        b.left = d.length() + 1.0;
                        b.bounce += 1;
                        ev.push(Event::ShieldBounce { at });
                    }
                    _ => {
                        b.back = true;
                        b.hits = 0;
                    }
                }
                keep.push(b);
                continue;
            }
            if !b.def.pierce && !struck.is_empty() {
                continue;
            }
            b.pos += b.dir * len;
            b.left -= len;
            // 땅(0173 → 0218): 마법 탄은 땅을 따라 흐르고(사장 2026-11-13 "고도 차이가 있을 때 원거리 공격이 안 맞는다"), 쇠뇌 볼트는
            // 곧게 난다(사장 2026-11-16 "땅을 타고 가면 안 되고, 직선이어야지" — 높이는 쏠 때 줄 위의 적 몸통으로 겨눈다, `shot_line`).
            gone |= self.bolt_lands(&mut b, len);
            if !gone
                && b.left > 1e-3
                && b.pos.x.abs() <= self.arena + 2.0
                && b.pos.z.abs() <= self.arena + 2.0
            {
                keep.push(b);
            } else if b.def.returns() {
                // 던진 창(0274 ②b): 멈춘 자리에서 돌아온다.
                b.back = true;
                b.hits = 0;
                keep.push(b);
            }
        }
        self.bolts = keep;
    }

    fn tick_bombs(&mut self, t: &Tuning, dt: f32, ev: &mut Vec<Event>) {
        // 수류탄(0051): 중력으로 날다 땅에서 튕기고(줄어드는 위·옆 속도) 벽에서 되튄다, 도화선이 다하면 터진다.
        let mut booms: Vec<Bomb> = Vec::new();
        let mut bombs = std::mem::take(&mut self.bombs);
        for bomb in &mut bombs {
            self.roll_bomb(bomb, dt);
            if bomb.fuse <= 0.0 {
                booms.push(*bomb);
            }
        }
        bombs.retain(|b| b.fuse > 0.0);
        self.bombs = bombs;
        for b in booms {
            let at = b.pos;
            let dose = (crate::status::Kind::Root, b.root);
            self.burst(at, b.radius, b.damage, b.poise, dose, t, ev);
            self.blasts.push(Blast {
                pos: at,
                radius: b.radius,
                age: 0.0,
                root: b.root > 0.0,
            });
            ev.push(Event::Boom { at });
        }
    }

    fn tick_cracks(&mut self, t: &Tuning, dt: f32, ev: &mut Vec<Event>) {
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
            self.burst(
                c.pos,
                c.radius,
                c.damage,
                c.poise,
                crate::status::NO_DOSE,
                t,
                ev,
            );
            self.blasts.push(Blast {
                pos: c.pos,
                radius: c.radius,
                age: 0.0,
                root: false,
            });
            ev.push(Event::Aftershock { at: c.pos });
        }
    }
}

/// 신성한 빛 하나(0274 ②b): 탄 표의 피해·자세·속성, 반지름 = `SMITE_RADIUS` × `scale`, `SMITE_DELAY`초 뒤.
fn smite_of(at: Vec3, from: Vec3, def: &weapons::Bolt) -> Smite {
    Smite {
        look: def.look,
        from,
        pos: at,
        left: SMITE_DELAY,
        radius: SMITE_RADIUS * def.scale,
        damage: def.damage,
        poise: def.poise,
        dose: def.dose,
        struck: false,
    }
}
