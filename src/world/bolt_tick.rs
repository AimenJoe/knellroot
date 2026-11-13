//! 날아가는 것·늦게 터지는 것의 틱(0159 R2): 한 함수였던 `tick_bolts`(309줄)를 단계마다 메서드로 — 순서와 몸은 그대로다.
//!
//! 순서: 틱이 남긴 알림 → 자세 막대 → 폭발 고리·불똥의 나이 → 쏜 탄·공중 RB·던진 수류탄을 세움 → 탄이 난다 → 수류탄 → 금 간 땅.
use super::*;

impl World {
    /// 날아가는 것·늦게 터지는 것(0048·0051): 마법 탄·쇠뇌 볼트, 수류탄, 금 간 땅(망치 여진), 자세 막대가 줄기, 폭발 고리.
    /// 탄은 이번 틱에 쏜 것을 세우고 날려 벽·땅·적에 닿으면 지운다(꿰뚫는 탄은 적을 지나간다). 칸은 채우지 않는다(근접만 채운다).
    pub(super) fn tick_bolts(&mut self, t: &Tuning, dt: f32, ev: &mut Vec<Event>) {
        if std::mem::take(&mut self.beat_now) {
            ev.push(Event::Beat);
        }
        if std::mem::take(&mut self.cast_denied_now) {
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
    }

    fn tick_poise(&mut self, t: &Tuning, dt: f32) {
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

    /// 이번 틱에 쏜 마법 탄·볼트를 세운다(모은 정도마다 다른 탄, 0056·0078).
    fn fire_cast(&mut self, t: &Tuning, ev: &mut Vec<Event>, wpn: &weapons::Weapon) {
        if let Some((dir, power)) = self.cast_fire.take() {
            let pl = &self.player;
            let pos = pl.pos + Vec3::Y * (pl.air + BOLT_HEIGHT) + dir * (PLAYER_RADIUS + 0.2);
            let c = wpn.charged;
            let def = match (power, wpn.rb) {
                (0, Rb::Magic(b)) => b,
                (0, Rb::Shot { bolt, .. }) => bolt,
                // 덜 모은 탄(0056): 모은 탄의 반(옛 눈금으로 올림, 0158 N0) — 꿰뚫기·휘청 없음.
                (1, _) => weapons::Bolt {
                    damage: crate::tuning::half_up(c.damage),
                    poise: c.poise * 0.5,
                    pierce: false,
                    stagger: 0.0,
                    scale: 1.0 + (c.scale - 1.0) * 0.4,
                    ..c
                },
                // 딱 맞춰 뗀 모은 탄(0078): 강타의 딱 맞춤과 같은 덤 — 피해 +한 눈금(`HP_UNIT`)·자세 피해 ×1.5.
                (3, _) => weapons::Bolt {
                    damage: c.damage + HP_UNIT,
                    poise: c.poise * 1.5,
                    ..c
                },
                _ => c,
            };
            if power == 3 {
                self.player.beat.since_perfect = 0.0;
                ev.push(Event::Perfect { at: pos });
            }
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
    }

    fn fire_air(&mut self, t: &Tuning, ev: &mut Vec<Event>, wpn: &weapons::Weapon) {
        // 공중 RB(0111): 쏜 높이에서 겨눈 적(쏘는 쪽 ±15°·`cast.range m` 안 가장 가까운)의 몸통으로, 없으면 앞
        // `AIR_SHOT_REACH`m 땅 위 몸통 높이로 비스듬히 내려 쏜다. 쇠뇌는 같은 자리로 볼트 `AIR_VOLLEY`발을 부채꼴로.
        if let Some(flat) = self.air_fire.take() {
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
            let (def, count) = match wpn.rb {
                Rb::Magic(b) => (b, 1),
                Rb::Shot { bolt, .. } => (bolt, AIR_VOLLEY),
            };
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
                });
            }
            ev.push(Event::Cast { at: from });
        }
    }

    /// 이번 틱에 던진 수류탄(0051)을 세운다.
    fn throw_bomb(&mut self, ev: &mut Vec<Event>, wpn: &weapons::Weapon) {
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
    }

    /// 탄이 난다: 벽·기둥·수류탄·적(꿰뚫는 탄은 모두, 아니면 가장 가까운 하나)·땅에 닿으면 사라진다.
    fn fly_bolts(&mut self, t: &Tuning, dt: f32, ev: &mut Vec<Event>) {
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
                let en = &mut self.enemies[i];
                // 등딱지(0120): 앞에서 온 탄도 튕긴다(탄은 거기서 멈춘다 — 방랑자는 튀지 않는다).
                if en.guards(-b.dir) {
                    ev.push(Event::Deflect {
                        at: b.pos + b.dir * k,
                    });
                    continue;
                }
                let crit = en.exposed > 0.0;
                en.hp -= hurt_by(
                    en,
                    self.stats.dmg + b.def.damage,
                    crit,
                    self.stats.opened_dmg,
                    t,
                );
                en.exposed = 0.0;
                en.hurt = 0.15;
                en.knock = b.dir * t[P::Knockback] * if en.boss { 0.1 } else { 0.5 };
                self.bolt_hit_tick = true;
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
            } else {
                // 평평한 시험 판: 내려 쏜 탄(0111)은 바닥(0)에 닿으면 사라진다.
                gone |= b.pos.y < 0.1;
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
    }

    fn tick_bombs(&mut self, t: &Tuning, dt: f32, ev: &mut Vec<Event>) {
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
            self.burst(c.pos, c.radius, c.damage, c.poise, t, ev);
            self.blasts.push(Blast {
                pos: c.pos,
                radius: c.radius,
                age: 0.0,
            });
            ev.push(Event::Aftershock { at: c.pos });
        }
    }
}
