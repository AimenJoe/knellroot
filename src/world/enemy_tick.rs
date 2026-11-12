//! 적 틱(0159 R2): 한 함수였던 `tick_enemies`(507줄)를 단계마다 메서드로 나눴다 — 순서와 몸은 그대로다.
//!
//! 순서: 길 지도 → 알아차림 → 길(돌아가기·쫓기·턱 뛰기) → 떼 퍼지기 → 꼬투리의 물러남 → **적마다 한 틱**(`enemies_act`: 닿은 타를
//! 패링·막음·맞음으로 가른다) → 2단계 전환의 밀어냄 → 종 조각 → 남는 고리 → 돌진 미리 받기 → 패링의 결과 → 되울림 →
//! **방랑자가 맞는다**(`wanderer_takes_hit` — 체력을 깎는 한 자리, 0143). 새 피해 경로는 `hurt_player`에 친 자리를 넣는다.
use super::*;

/// 적마다 한 틱이 낸 것 — 뒤 단계(조각·고리·패링·맞음)가 이어 쓴다.
struct EnemyActs {
    /// 플레이어를 친 적의 자리(첫 하나) — 밀려나는 쪽.
    hurt_player: Option<Vec3>,
    /// 맞힌 적 가운데 가장 높은 구역 등급(0112).
    hurt_tier: u8,
    /// 튕긴 적(번호).
    parried: Vec<usize>,
    /// 이번 틱에 던진 종 조각(0125).
    thrown: Vec<Shard>,
    /// 2단계 전환이 터지며 밀어낼 쪽(0133).
    burst_push: Option<Vec3>,
    /// 맞서는 자(0107)가 못 받는 공격을 막았나.
    blocked: bool,
}

impl World {
    pub(super) fn tick_enemies(&mut self, t: &Tuning, dt: f32, ev: &mut Vec<Event>) {
        let p = self.player.pos;
        let body = p.y + self.player.air;
        let parrying = self.parry_open(t);
        self.refresh_nav();
        self.enemies_notice(p, body);
        self.enemy_ways(p, body);
        self.enemy_spread();
        self.planter_hops(p);
        let facing = self.player.facing_dir();
        // 앞쪽에서 오는가(패링은 바라보는 쪽 반원 안만 — 위치 선정도 기술, KB timing-parry). 이어 패링이 쌓이면 넓어진다(0086).
        let front_cos = parry_front_cos(
            self.player.parry_chain_left > 0.0,
            self.player.parry_chain_n,
        );
        let in_front = |at: Vec3| {
            let d = vec3(at.x - p.x, 0.0, at.z - p.z).normalize_or_zero();
            d.dot(facing) > front_cos
        };
        let EnemyActs {
            mut hurt_player,
            hurt_tier,
            mut parried,
            mut thrown,
            burst_push,
            blocked,
        } = self.enemies_act(t, dt, ev, p, body, parrying, in_front);
        // 종 조각(0125): 날아가 플레이어에 닿으면 — 앞에서 패링 중이면 되돌아가고, 아니면 맞는다. 되돌아간 조각은 종지기를 깎는다.
        // 던진 종지기가 쓰러지면 날던 조각은 사라진다(KB ranged-boss — 쓰러진 뒤에 맞지 않게).
        self.shards.append(&mut thrown);
        self.rise_push(burst_push);
        let parries_before =
            self.tick_shards(dt, ev, p, body, parrying, in_front, &mut hurt_player);
        self.tick_lingers(t, dt, ev, p, body, &mut hurt_player);
        self.parry_charges(t, p, parrying, in_front, &mut parried);
        if blocked {
            ev.push(Event::Parry {
                at: p + Vec3::Y * 1.0,
            });
        }
        self.parries_land(t, ev, p, parried);
        // 되울림(0158 G2): 이번 틱에 성공한 패링마다 둘레가 울린다(패링 창은 건드리지 않는다 — 0095 원칙 2).
        let parries = ev
            .iter()
            .filter(|e| matches!(e, Event::Parry { .. }))
            .count()
            - parries_before;
        self.echo_ring(t, parries, ev);
        self.wanderer_takes_hit(t, ev, hurt_player, hurt_tier);
    }

    fn refresh_nav(&mut self) {
        // 길찾기(0043 → 0076): 몸 폭으로 곧게 못 가거나(벽·기둥·소품·턱) 높이가 다른 층이면 거리 지도를 따라
        // 앞으로 곧게 갈 수 있는 가장 먼 칸을 겨눈다(못 가면 그 자리에서 기다린다). 쫓는 적만 — 다른 상태는 길을 안 쓴다.
        if self.nav.as_ref().is_none_or(|n| n.stale(&self.obstacles)) {
            self.nav = Some(crate::nav::Nav::build(
                self.ground.as_ref(),
                &self.obstacles,
                self.arena,
            ));
            self.flow = None;
            self.home_flows.clear();
        }
    }

    fn enemies_notice(&mut self, p: Vec3, body: f32) {
        let nav = self.nav.as_ref().expect("built above");
        // 알아차림(0113): 제자리의 적은 시야(벽·턱 너머는 못 봄)·발소리·맞음으로 깨어 쫓고, 깨면 곁의 동료도 깬다.
        // 제자리에서 끈보다 멀어지고 플레이어도 그만큼 멀면 돌아간다. 플레이어가 제자리 끈 밖이면 알아차리지 않는다(되쫓기 왕복 없음).
        let mut woke: Vec<Vec3> = Vec::new();
        for en in self.enemies.iter_mut() {
            let Some((home, _)) = en.home else { continue };
            let from_home = vec3(p.x - home.x, 0.0, p.z - home.z).length();
            let d = vec3(p.x - en.pos.x, 0.0, p.z - en.pos.z).length();
            match en.state {
                EState::Idle | EState::Return => {
                    let near = from_home <= LEASH_RANGE && (body - en.pos.y).abs() <= SIGHT_DY;
                    // 청소자(0120)는 보고 들어도 먹던 일을 한다 — 맞거나 곁의 동료가 깨면 싸운다.
                    let sees = !en.docile
                        && near
                        && (d <= HEAR_RANGE || (d <= SIGHT_RANGE && nav.clear(en.pos, p, 0.1)));
                    if (sees || en.hurt > 0.0) && en.notice() {
                        woke.push(en.pos);
                    }
                }
                EState::Chase => {
                    let gone = vec3(en.pos.x - home.x, 0.0, en.pos.z - home.z).length();
                    if gone > LEASH_RANGE && from_home > LEASH_RANGE {
                        en.state = EState::Return;
                    }
                }
                _ => {}
            }
        }
        for at in woke {
            for en in self.enemies.iter_mut() {
                if en.state == EState::Idle && en.pos.distance(at) <= ALERT_RADIUS {
                    en.notice();
                }
            }
        }
    }

    fn enemy_ways(&mut self, p: Vec3, body: f32) {
        let nav = self.nav.as_ref().expect("built above");
        // 돌아가는 적의 길: 곧게 가면 바로, 아니면 제자리로 가는 거리 지도를 따라.
        for en in self
            .enemies
            .iter_mut()
            .filter(|e| e.state == EState::Return)
        {
            let Some((home, _)) = en.home else { continue };
            en.way = if nav.clear(en.pos, home, en.radius) {
                None
            } else {
                let cell = nav.g.cell_of(home).unwrap_or((0, 0));
                let flow = self
                    .home_flows
                    .entry(cell)
                    .or_insert_with(|| nav.flow(home));
                Some(nav.waypoint(flow, en.pos, en.radius).unwrap_or(home))
            };
        }
        let chasing = |en: &Enemy| en.state == EState::Chase;
        if self.enemies.iter().any(chasing) {
            let cell = nav.g.cell_of(p).unwrap_or((usize::MAX, usize::MAX));
            if self.flow.as_ref().is_none_or(|(c, _)| *c != cell) {
                self.flow = Some((cell, nav.flow(p)));
            }
            let dist = &self.flow.as_ref().expect("filled above").1;
            for en in self.enemies.iter_mut().filter(|e| chasing(e)) {
                // 턱을 뛰어오르기·뛰어내리기(0100): 길의 다음 칸이 뛰어야 하는 칸이고 변 앞에 왔으면 뛴다. 문지기(보스)는 걷기만.
                if en.hop.is_none() && !en.boss {
                    if let Some((land, _)) = nav.hop(dist, en.pos, en.radius) {
                        en.hop = Some(crate::enemy::Hop::new(en.pos, land));
                        en.way = None;
                        continue;
                    }
                }
                let same =
                    nav.g.cell_of(en.pos).is_some() && nav.g.cell_of(en.pos) == nav.g.cell_of(p);
                en.way = if (body - en.pos.y).abs() <= HIT_DY
                    && (same || nav.clear(en.pos, p, en.radius))
                {
                    None
                } else {
                    Some(nav.waypoint(dist, en.pos, en.radius).unwrap_or(en.pos))
                };
            }
        }
    }

    fn enemy_spread(&mut self) {
        // 떼 퍼지기(0076): 곁(몸 사이 `SPREAD_GAP` 안)의 적에게서 비키는 쪽 — 가까울수록 세게.
        let spots: Vec<(Vec3, f32, bool)> = self
            .enemies
            .iter()
            .map(|e| (e.pos, e.radius, e.active()))
            .collect();
        for (i, en) in self.enemies.iter_mut().enumerate() {
            let mut push = Vec3::ZERO;
            for (j, &(q, r, live)) in spots.iter().enumerate() {
                if i == j || !live {
                    continue;
                }
                let d = vec3(en.pos.x - q.x, 0.0, en.pos.z - q.z);
                let range = en.radius + r + SPREAD_GAP;
                let l = d.length();
                if l < range && l > 1e-4 {
                    push += d / l * (1.0 - l / range);
                }
            }
            en.spread = push;
        }
    }

    fn planter_hops(&mut self, p: Vec3) {
        let nav = self.nav.as_ref().expect("built above");
        // 꼬투리(0120): 너무 가까이 오면 한 번 깡충 물러난다 — 반대쪽부터 옆으로 조금씩 돌려 보며 곧게 갈 수 있는 바닥으로.
        for en in self.enemies.iter_mut() {
            let d = en.def();
            if d.behavior != crate::species::Behavior::Planter
                || en.hop.is_some()
                || en.hop_cd > 0.0
                || !matches!(en.state, EState::Chase | EState::Recover { .. })
            {
                continue;
            }
            let to = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z);
            if to.length() > d.retreat {
                continue;
            }
            let away = to.normalize_or(Vec3::Z);
            for deg in [0.0f32, 35.0, -35.0, 70.0, -70.0] {
                let dir = Quat::from_rotation_y(deg.to_radians()) * away;
                let mut land = en.pos + dir * crate::tuning::POD_HOP;
                if nav.g.cell_of(land).is_none() || !nav.clear(en.pos, land, en.radius) {
                    continue;
                }
                land.y = nav.g.height(land);
                en.hop = Some(crate::enemy::Hop::new(en.pos, land));
                en.hop_cd = crate::tuning::POD_HOP_CD;
                en.state = EState::Chase;
                en.plant_at = None;
                break;
            }
        }
    }

    /// 적마다 한 틱(뛰는 중이면 포물선만): 생각·움직임(`Enemy::tick`), 돌진 길이, 종 조각 던지기, 닿은 타를 패링·막음·맞음으로 가른다. 돌려주는 것은 그 결과(`EnemyActs`).
    #[allow(clippy::too_many_arguments)]
    fn enemies_act(
        &mut self,
        t: &Tuning,
        dt: f32,
        ev: &mut Vec<Event>,
        p: Vec3,
        body: f32,
        parrying: bool,
        in_front: impl Fn(Vec3) -> bool,
    ) -> EnemyActs {
        let nav = self.nav.as_ref().expect("built above");
        // 플레이어를 친 적의 자리(첫 하나) — 밀려나는 쪽.
        let mut hurt_player: Option<Vec3> = None;
        // 맞힌 적 가운데 가장 높은 구역 등급(0112 — 등급 3부터 한 대 2).
        let mut hurt_tier: u8 = 1;
        let mut parried: Vec<usize> = Vec::new();
        // 이번 틱에 던진 종 조각(0125 — 적을 도는 동안은 목록을 빌릴 수 없어 모았다가 넣는다).
        let mut thrown: Vec<Shard> = Vec::new();
        // 2단계 전환이 터지며 밀어낼 쪽(0133).
        let mut burst_push: Option<Vec3> = None;
        let mut blocked = false;
        let stand = self.tree.has(Cell::OneWhoStands)
            && self.player.parry_chain_left > 0.0
            && self.player.parry_chain_n >= cell::STAND_CHAIN;
        for (i, en) in self.enemies.iter_mut().enumerate() {
            // 뛰는 중(0100): 몸은 포물선을 따라가고 생각하지도 치지도 않는다. 쓰러지면 그 자리에서 내려앉는다.
            if en.hop.is_some() && !en.alive() {
                en.hop = None;
            }
            if let Some(mut h) = en.hop {
                h.t += dt;
                let d = vec3(h.to.x - h.from.x, 0.0, h.to.z - h.from.z);
                if d.length_squared() > 1e-6 {
                    en.facing = d.x.atan2(d.z);
                }
                en.pos = h.at(h.t / h.dur);
                en.hop = if h.t >= h.dur {
                    en.pos = h.to;
                    None
                } else {
                    Some(h)
                };
                continue;
            }
            let out = en.tick(p, dt);
            // 돌진 길이(0076): 벽·기둥·턱·아레나 끝까지 — 예고 띠와 몸이 같은 길이.
            // 뿌리 기둥 줄(0133)도 같은 길이 칸 — 벽·기둥을 뚫고 솟지 않는다.
            if matches!(en.atk, Atk::Charge | Atk::Roots) && matches!(en.state, EState::Tele { .. })
            {
                let d = en.def();
                let max = if en.atk == Atk::Roots {
                    crate::tuning::ROOTS_LEN
                } else {
                    d.charge_speed * d.charge_time
                };
                let room = nav.charge_room(
                    &self.obstacles,
                    self.arena,
                    en.pos,
                    en.facing_dir(),
                    en.radius,
                    max,
                );
                en.set_charge_room(room, p);
            }
            // 바닥을 훑는 파동은 높이 떠 있으면 넘는다(0027) — 높은 단 위도 마찬가지(0043).
            let above = body - en.pos.y;
            // 울림 고리(0125)도 바닥을 훑는다 — 뛰면 넘는다.
            // 종이 떨어진 자리의 고리(0133)도 — 떨어지는 순간(원판)은 아니다.
            let floor_ring = matches!(en.atk, Atk::Wave | Atk::Toll)
                || (en.atk == Atk::Hurl && matches!(en.state, EState::Wave { .. }));
            let jumped = floor_ring && above > t[P::JumpClear];
            // 2단계 전환이 터진다(0133): 몸 둘레의 방랑자를 밀어낸다(피해 없음 — 튕김처럼 잠깐 못 움직인다).
            if out.burst {
                let away = vec3(p.x - en.pos.x, 0.0, p.z - en.pos.z);
                if away.length() - en.radius <= crate::tuning::RISE_PUSH_R {
                    burst_push = Some(away.normalize_or(-en.facing_dir()));
                }
            }
            // 종 조각(0125): 플레이어 쪽 가운데로 부채꼴(가슴 높이).
            if out.shards > 0 {
                let aim = vec3(p.x - en.pos.x, 0.0, p.z - en.pos.z).normalize_or(en.facing_dir());
                let n = out.shards as usize;
                let spread = en.def().arc.to_radians();
                for k in 0..n {
                    let f = if n == 1 {
                        0.0
                    } else {
                        k as f32 / (n - 1) as f32 - 0.5
                    };
                    let dir = Quat::from_rotation_y(f * spread) * aim;
                    thrown.push(Shard {
                        pos: en.pos + dir * (en.radius + 0.2) + Vec3::Y * 1.1,
                        dir,
                        went: 0.0,
                        reflected: false,
                        from: i,
                    });
                }
            }
            let reach = above.abs() <= HIT_DY;
            if out.hit && !jumped && reach {
                if parrying && en.parryable() && in_front(en.pos) {
                    parried.push(i);
                } else if parrying && !en.parryable() && in_front(en.pos) && stand {
                    // 맞서는 자(0107): 이어 패링이 쌓인 동안은 못 받는 공격도 막는다(튕기지는 못한다 — 드러남·휘청 없음).
                    blocked = true;
                } else {
                    hurt_player.get_or_insert(en.pos);
                    hurt_tier = hurt_tier.max(en.tier);
                }
            }
            if out.release {
                ev.push(Event::Release {
                    at: en.pos + Vec3::Y * en.height * 0.8,
                });
            }
            if out.roused {
                ev.push(Event::Rouse {
                    at: en.pos + Vec3::Y * en.height * 0.6,
                });
            }
        }
        EnemyActs {
            hurt_player,
            hurt_tier,
            parried,
            thrown,
            burst_push,
            blocked,
        }
    }

    fn rise_push(&mut self, burst_push: Option<Vec3>) {
        // 버티는 동안(0158 G2 버팀)은 2단계 전환의 밀어냄에도 밀리지 않는다.
        if let (Some(away), false) = (burst_push, self.bracing()) {
            let pl = &mut self.player;
            pl.vel = away * crate::tuning::RISE_PUSH;
            pl.state = PState::Recoil { t: 0.0 };
            pl.pending = None;
        }
    }

    /// 종 조각(0125)이 난다: 방랑자에 닿으면 패링(되돌아감)·받아 둠(버팀 ㄴ)·맞음, 되돌아간 조각은 던진 적을 깎는다. 돌려주는 것은 이 단계에 들어설 때 `ev`에 있던 패링 수(되울림이 이번 틱의 패링을 세는 기준).
    #[allow(clippy::too_many_arguments)]
    fn tick_shards(
        &mut self,
        dt: f32,
        ev: &mut Vec<Event>,
        p: Vec3,
        body: f32,
        parrying: bool,
        in_front: impl Fn(Vec3) -> bool,
        hurt_player: &mut Option<Vec3>,
    ) -> usize {
        let mut shard_hits: Vec<(usize, Vec3)> = Vec::new();
        // 되쏘기(0158 G2, 버팀 ㄴ): 버티는 동안 몸에 닿은 조각은 받아 두었다가 터뜨릴 때 던진 쪽으로 되쏜다.
        let catching = self.bracing() && self.player.skill_reflect && !self.player.invulnerable();
        let mut caught: Vec<usize> = Vec::new();
        // 이번 틱의 패링 수(0158 G2 되울림) — 이 단계가 낸 `Event::Parry`를 센다.
        let parries_before = ev
            .iter()
            .filter(|e| matches!(e, Event::Parry { .. }))
            .count();
        self.shards.retain_mut(|s| {
            let alive = self.enemies.get(s.from).is_some_and(|e| e.alive());
            if !alive {
                return false;
            }
            let speed = SHARD_SPEED
                * if s.reflected {
                    SHARD_REFLECT_SPEED
                } else {
                    1.0
                };
            s.pos += s.dir * speed * dt;
            s.went += speed * dt;
            if s.went > SHARD_RANGE {
                return false;
            }
            if s.reflected {
                let b = &self.enemies[s.from];
                let d = vec2(b.pos.x - s.pos.x, b.pos.z - s.pos.z).length();
                if d <= b.radius + SHARD_R {
                    shard_hits.push((s.from, s.pos));
                    return false;
                }
                return true;
            }
            let d = vec2(p.x - s.pos.x, p.z - s.pos.z).length();
            if d <= SHARD_R + PLAYER_RADIUS && (s.pos.y - body - 1.0).abs() <= HIT_DY + 0.6 {
                if parrying && in_front(s.pos - s.dir) {
                    let b = self.enemies[s.from].pos;
                    s.dir = vec3(b.x - s.pos.x, 0.0, b.z - s.pos.z).normalize_or(-s.dir);
                    s.reflected = true;
                    s.went = 0.0;
                    ev.push(Event::Parry { at: s.pos });
                    return true;
                }
                if catching {
                    caught.push(s.from);
                }
                hurt_player.get_or_insert(s.pos);
                return false;
            }
            true
        });
        self.brace_caught.append(&mut caught);
        for (i, at) in shard_hits {
            let en = &mut self.enemies[i];
            if en.rising() {
                ev.push(Event::Deflect { at });
                continue;
            }
            en.hp -= SHARD_REFLECT_DAMAGE;
            en.hurt = 0.15;
            ev.push(Event::Hit { at });
            if en.hp <= 0 {
                en.state = EState::Dead;
                self.kills += 1;
                ev.push(Event::EnemyDown { at: en.pos });
            }
        }
        parries_before
    }

    fn tick_lingers(
        &mut self,
        t: &Tuning,
        dt: f32,
        ev: &mut Vec<Event>,
        p: Vec3,
        body: f32,
        hurt_player: &mut Option<Vec3>,
    ) {
        // 남는 엘리트의 고리(0112): 다 차면 둘레를 친다 — 바닥을 훑는 것이라 높이 뛰면 넘는다(뿌리 파동과 같다), 패링은 못 한다.
        for l in &mut self.lingers {
            l.t += dt;
            if l.t >= ELITE_LINGER_DELAY {
                let flat = vec3(p.x - l.pos.x, 0.0, p.z - l.pos.z).length();
                let above = body - l.pos.y;
                if flat <= ELITE_LINGER_RADIUS + PLAYER_RADIUS
                    && above <= t[P::JumpClear]
                    && above >= -HIT_DY
                {
                    hurt_player.get_or_insert(l.pos);
                }
                self.blasts.push(Blast {
                    pos: l.pos,
                    radius: ELITE_LINGER_RADIUS,
                    age: 0.0,
                });
                ev.push(Event::Boom { at: l.pos });
            }
        }
        self.lingers.retain(|l| l.t < ELITE_LINGER_DELAY);
    }

    fn parry_charges(
        &self,
        t: &Tuning,
        p: Vec3,
        parrying: bool,
        in_front: impl Fn(Vec3) -> bool,
        parried: &mut Vec<usize>,
    ) {
        // 곧게 오는 돌진은 닿기 조금 앞(`parry.reach m`)에서부터 받는다(nail PARRY_REACH 0258).
        if parrying {
            for (i, en) in self.enemies.iter().enumerate() {
                if let EState::Charge { dir, .. } = en.state {
                    let to = vec3(p.x - en.pos.x, 0.0, p.z - en.pos.z);
                    let near = to.length() <= en.radius + PLAYER_RADIUS + t[P::ParryReach];
                    let coming = dir.dot(to.normalize_or_zero()) > 0.5;
                    if !en.landed && near && coming && in_front(en.pos) && !parried.contains(&i) {
                        parried.push(i);
                    }
                }
            }
        }
    }

    /// 튕긴 적마다 휘청·드러남·밀림(받아내기 0107), 방랑자는 곧바로 움직이고 이어 패링 창이 열린다(0081 → 0086), 세상은 한 박자 멈춘다.
    fn parries_land(&mut self, t: &Tuning, ev: &mut Vec<Event>, p: Vec3, parried: Vec<usize>) {
        if !parried.is_empty() {
            let catch = self.tree.has(Cell::Catch);
            for &i in &parried {
                let en = &mut self.enemies[i];
                let dur = if en.boss {
                    t[P::ParryBossStagger]
                } else {
                    t[P::ParryStagger]
                };
                let away = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z).normalize_or_zero();
                en.landed = true;
                en.state = EState::Stagger { t: 0.0, dur };
                en.exposed = dur;
                en.knock = away * PARRY_PUSH * KNOCK_DECAY * if en.boss { 0.3 } else { 1.0 };
                en.sway = away * 0.8;
                en.sway_t = 0.0;
                // 받아내기(0107): 패링이 적의 자세 막대를 채운다(문지기는 반).
                if catch {
                    let k = if en.boss { 0.5 } else { 1.0 };
                    let amount = poise_max(en, t) * cell::CATCH_POISE * k;
                    add_poise(en, amount, None, t, ev);
                }
                ev.push(Event::Parry {
                    at: en.pos + Vec3::Y * en.height * 0.6,
                });
            }
            // 모은 맞받기(0107): 모으기 패링(떼는 순간의 창)으로 튕겼으면 그 강타 스윙을 기억한다 — 맞으면 자세가 깨진다.
            if let (PState::Heavy { swing, .. }, true) =
                (self.player.state, self.player.release_guard > 0.0)
            {
                self.player.gathered_swing = swing;
            }
            // 튕기면 곧바로 움직일 수 있다(반격하라고) — 세상은 한 박자 멈춘다(명중보다 크게, KB).
            // 이어 패링(0081)으로 튕겼으면 그 사이 시작한 동작(회피 등)은 그대로 둔다.
            if matches!(self.player.state, PState::Parry { .. }) {
                self.player.state = PState::Idle;
            }
            // 이어 패링(0081 → 0086): 처음 튕기면 창 `PARRY_CHAIN`, 이어지는 동안 더 튕길 때마다 남은 창에 반씩 줄어드는 몫을 더하고
            // 받는 반원이 넓어진다. 한 틱에 여럿을 튕기면 하나씩 센다(둘러싸였을 때 가장 빨리 넓어진다).
            let pl = &mut self.player;
            for _ in 0..parried.len() {
                (pl.parry_chain_left, pl.parry_chain_n) =
                    parry_chain_add(pl.parry_chain_left, pl.parry_chain_n);
            }
            pl.since_parry = 0.0;
            // 튕기면 기력이 찬다(0088): 0까지 쓰고 튕겨도 반격을 바로 낼 수 있게. 한 틱에 여럿을 튕겨도 한 번.
            pl.stamina = (pl.stamina + t[P::ParryStaminaGain]).min(t[P::StaminaMax]);
            self.hitstop = self.hitstop.max(t[P::Hitstop] * PARRY_HITSTOP);
            // 같은 틱이라도 뒤·옆에서 맞은 것·받을 수 없는 파동은 그대로 맞는다(패링은 앞쪽 반원만).
        }
    }

    /// 방랑자가 맞는다 — **체력을 깎는 한 자리**(0143). 적이 주는 피해는 모두 `hurt_player`(친 자리)로 모여 여기로 온다: 맞음 피드백 다섯 겹(멈춤·붉은 테·체력 칸·흰 몸·밀림)이 여기서 난다. 새 피해 경로도 이 길을 탄다.
    fn wanderer_takes_hit(
        &mut self,
        t: &Tuning,
        ev: &mut Vec<Event>,
        hurt_player: Option<Vec3>,
        hurt_tier: u8,
    ) {
        if let (Some(from), false) = (hurt_player, self.player.invulnerable()) {
            // 한 대 = 한 눈금(`HP_UNIT`, 0158 N0), 높은 등급의 적은 두 눈금.
            let blow = if hurt_tier >= crate::tuning::TIER_HEAVY_HITS {
                2 * HP_UNIT
            } else {
                HP_UNIT
            };
            // 버팀(0158 G2): 버티는 동안은 타마다 덜 다치고(0 아래로는 안 내려간다) 밀리지도 끊기지도 않는다 — 받아낸 타를 센다.
            let braced = self.bracing();
            let taken = if braced {
                (blow + self.stats.taken - self.player.skill_brace).max(0)
            } else {
                blow + self.stats.taken
            };
            let pl = &mut self.player;
            pl.hp -= taken;
            // 나가기 전에 맞았으면 받아 둔 패링은 버린다(0110) — 맞은 뒤 뒤늦게 나가 헛침 굳음이 생기지 않게.
            if pl.pending == Some(Next::Parry) {
                pl.pending = None;
            }
            // 맞은 뒤 무적은 짧게(예전 0.6초 — 그 동안 다시 맞을 수 없어 경직 줄이기가 쓸 데가 없었다).
            pl.hurt = t[P::HitIframes];
            if braced {
                pl.brace_soaked = (pl.brace_soaked + 1).min(pl.brace_most);
                ev.push(Event::Soak {
                    at: pl.pos + Vec3::Y * (pl.air + 1.0),
                    n: pl.brace_soaked,
                });
            }
            // 다 받아낸 타(피해 0)는 맞음 피드백(붉은 테·흰 몸·멈춤)을 내지 않는다 — 받아냈다는 불똥만.
            let felt = taken > 0 || !braced;
            if felt {
                pl.since_hurt = 0.0;
                pl.hurt_blow = taken;
            }
            if !braced {
                // 경직 이음(0018 → 0099, 사장 "다구리 맞더라도 틈이 생겨서 도망갈 수 있게" — 이어 패링의 반대): 경직과 그 뒤
                // `STUN_CHAIN_GRACE` 동안 또 맞으면 경직이 그전의 반 — 맞을수록 짧아져 떼 속에서도 빠져나갈 틈이 생긴다.
                (pl.stun_chain, pl.stun_window) =
                    stun_chain_next(pl.stun_window, pl.stun_chain, t[P::HitStun]);
                let dur = t[P::HitStun] * 0.5f32.powi(pl.stun_chain);
                if dur > 0.0 {
                    pl.state = PState::Hurt { t: 0.0, dur };
                } else if matches!(pl.state, PState::Drink { .. }) {
                    // 맞으면 마시기는 끊긴다(삼키기 전이면 등잔 그대로, 0031).
                    pl.state = PState::Idle;
                }
                let away = vec3(pl.pos.x - from.x, 0.0, pl.pos.z - from.z).normalize_or_zero();
                pl.vel = away * t[P::HitKnock] * KNOCK_DECAY;
            }
            if felt {
                // 맞은 순간 세계가 멈춘다(0143): 때릴 때보다 길게, 센 타격은 더 길게. 이어 맞으면 경직처럼 반씩(떼 속에서 화면이 얼어붙지 않게).
                let heavy = if taken >= crate::tuning::HURT_HEAVY_BLOW {
                    crate::tuning::HURT_HEAVY
                } else {
                    1.0
                };
                let stop = t[P::Hitstop]
                    * crate::tuning::HURT_HITSTOP
                    * heavy
                    * 0.5f32.powi(pl.stun_chain);
                self.hitstop = self.hitstop.max(stop);
                ev.push(Event::PlayerHurt);
            }
        }
    }
}
