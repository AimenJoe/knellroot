//! 큰 기술의 몸(0251 R11 — `skill_tick.rs`에서 뺐다): 시작(게이지를 통째로) · 한 줄기 가르기 · 쟁반 뒤집기.
use super::*;
use crate::skills::{AMBUSH_AIM_COS, AMBUSH_CONE_DEG, AMBUSH_GAP, AMBUSH_HOLD, AMBUSH_PUSH};

impl World {
    /// 큰 기술 시작(끼운 하나): 게이지를 통째로 쓴다. 쟁반 뒤집기·한 줄기는 세계가 한 번 멈춘다(시동 정지 — "시간"),
    /// 한 줄기는 가를 쪽을 이 순간에 정한다(조준 > 락온 > 닿는 거리의 가장 가까운 적 > 바라보는 쪽).
    pub(super) fn start_ult(&mut self, cx: &Ctx) {
        let kind = self.player.skill.ult;
        if matches!(kind, skills::Ult::Line | skills::Ult::Gale) {
            let pl = &self.player;
            let lock = cx
                .target
                .map(|tg| vec3(tg.x - pl.pos.x, 0.0, tg.z - pl.pos.z).normalize_or_zero())
                .filter(|v| v.length_squared() > 0.5);
            let reach = if kind == skills::Ult::Gale {
                skills::GALE.len_at(pl.skill.ult_rank)
            } else {
                LINE.len_at(pl.skill.ult_rank)
            };
            let dir = cx
                .aim
                .or(lock)
                .or_else(|| nearest_in_reach(&self.enemies, pl.pos, reach, None));
            if let Some(d) = dir {
                self.player.facing = d.x.atan2(d.z);
            }
        }
        if kind == skills::Ult::Ambush {
            self.ambush_blink(cx);
        }
        if kind == skills::Ult::Rain {
            let (dir, dist) = self.rain_aim(cx);
            self.player.facing = dir.x.atan2(dir.z);
            self.player.skill.rain_at = self.place_ahead(self.player.pos, dir, dist);
        }
        let pl = &mut self.player;
        pl.skill.gauge = 0.0;
        pl.state = PState::Ult {
            kind,
            t: 0.0,
            done: false,
        };
        let (freeze, ..) = kind.times();
        self.hitstop = self.hitstop.max(freeze);
        self.skill_ev.push(Event::UltStart);
    }

    pub(in crate::world) fn tick_ult(&mut self, cx: &Ctx, kind: skills::Ult, e: f32, done: bool) {
        let e1 = e + cx.dt;
        let mut done = done;
        let (_, startup, recover) = kind.times();
        if !done && e1 >= startup {
            done = true;
            match kind {
                skills::Ult::Upend => self.upend(cx.t),
                skills::Ult::Flurry => {
                    let pl = &mut self.player;
                    pl.skill.flurry_left = FLURRY.secs_at(pl.skill.ult_rank);
                    pl.skill.flurry_total = pl.skill.flurry_left;
                    pl.skill.flurry_added = 0.0;
                    self.skill_ev.push(Event::Flurry { at: pl.pos });
                }
                skills::Ult::Line => self.line_cut(cx.t),
                skills::Ult::Bell => self.great_bell(cx.t),
                skills::Ult::Blizzard => self.blizzard(),
                skills::Ult::Rain => self.arrow_rain(),
                // 질풍 찌르기 · 그림자 습격: 찌름은 아래에서 때마다(첫 찌름이 시동 끝).
                skills::Ult::Gale | skills::Ult::Ambush => {}
            }
        }
        if kind == skills::Ult::Gale {
            for k in 0..skills::GALE.thrusts {
                let at = startup + skills::GALE.thrust_at(k);
                if e < at && e1 >= at {
                    self.gale_thrust(cx.t, k);
                }
            }
        }
        if kind == skills::Ult::Ambush {
            for k in 0..skills::AMBUSH.stabs {
                let at = startup + skills::AMBUSH.stab_at(k);
                if e < at && e1 >= at {
                    self.ambush_stab(cx.t, k);
                }
            }
        }
        self.player.state = if e1 >= startup + recover {
            PState::Idle
        } else {
            PState::Ult { kind, t: e1, done }
        };
    }

    /// 한 줄기(G2): 바라보는 쪽 한 줄(반 폭 `LINE.half` m)을 가른다 — 벽·기둥 앞에서 끊긴다. 줄 위의 깨어 있는 적이 모두 다치고
    /// 자세가 크게 깎인다. 보스도 같다(보스전의 큰 기술) — 휘청은 자세가 깨져서 생기는 것뿐이다.
    pub(super) fn line_cut(&mut self, t: &Tuning) {
        let mut ev = std::mem::take(&mut self.skill_ev);
        let p = self.player.pos;
        let body = p.y + self.player.air;
        let dir = self.player.facing_dir();
        let rank = self.player.skill.ult_rank;
        let mut len = LINE.len_at(rank);
        let eye = p + Vec3::Y * (self.player.air + BLADE_HEIGHT);
        for o in &self.obstacles {
            if let Some(d) = o.ray(eye, dir) {
                len = len.min(d);
            }
        }
        let damage = LINE.damage_at(rank);
        for i in 0..self.enemies.len() {
            let en = &self.enemies[i];
            if !en.active() || (en.pos.y - body).abs() > HIT_DY {
                continue;
            }
            let dv = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z);
            let along = dv.dot(dir);
            let off = (dv - dir * along).length();
            if along < -en.radius || along - en.radius > len || off > LINE.half + en.radius {
                continue;
            }
            let knock = dir * t[P::Knockback] * if en.boss { 0.25 } else { 1.0 };
            self.sparks.extend(sparks_at(
                en.pos + Vec3::Y * en.height * 0.55,
                dir,
                self.player.swings,
                true,
            ));
            self.skill_touch(
                t,
                &mut ev,
                i,
                &Touch {
                    damage,
                    poise: LINE.poise,
                    stagger: 0.0,
                    knock,
                    crit: true,
                    dose: self.dosed(skills::LINE_DOSE),
                    flat: false,
                },
            );
        }
        self.hitstop = self.hitstop.max(t[P::Hitstop] * ULT_HITSTOP);
        ev.push(Event::LineCut {
            at: p,
            dir,
            len,
            half: LINE.half,
        });
        self.skill_ev = ev;
    }

    /// 질풍 찌르기(0272 2단계)의 `k`째 찌름: 바라보는 쪽 한 줄(벽 · 높은 턱 앞에서 끊긴다 — `lane_reach`) 위의 깨어 있는 적을 모두
    /// 찌른다. 앞의 찌름은 가볍게(밀지 않는다 — 떼가 줄 위에 남는다), 마지막 찌름은 깊게: 큰 피해 · 큰 자세 피해 · 출혈, 잡몹은 밀려나
    /// 휘청인다(보스는 밀리지도 휘청이지도 않는다 — `skill_touch`가 보스의 휘청을 거르고, 밀기는 여기서 뺀다).
    pub(super) fn gale_thrust(&mut self, t: &Tuning, k: u8) {
        let g = &skills::GALE;
        let mut ev = std::mem::take(&mut self.skill_ev);
        let p = self.player.pos;
        let body = p.y + self.player.air;
        let dir = self.player.facing_dir();
        let rank = self.player.skill.ult_rank;
        let last = k + 1 >= g.thrusts;
        let len = self.lane_reach(p + Vec3::Y * self.player.air, dir, g.len_at(rank));
        let (light, deep) = g.damage_at(rank);
        let dose = if last {
            self.dosed(g.dose)
        } else {
            crate::status::NO_DOSE
        };
        for i in 0..self.enemies.len() {
            let en = &self.enemies[i];
            if !en.active() || (en.pos.y - body).abs() > HIT_DY {
                continue;
            }
            let dv = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z);
            let along = dv.dot(dir);
            let off = (dv - dir * along).length();
            if along < -en.radius || along - en.radius > len || off > g.half + en.radius {
                continue;
            }
            let knock = if last && !en.boss {
                dir * g.push * KNOCK_DECAY
            } else {
                Vec3::ZERO
            };
            if last {
                self.sparks.extend(sparks_at(
                    en.pos + Vec3::Y * en.height * 0.55,
                    dir,
                    self.player.swings,
                    true,
                ));
            }
            self.skill_touch(
                t,
                &mut ev,
                i,
                &Touch {
                    damage: if last { deep } else { light },
                    poise: if last { g.last_poise } else { g.poise },
                    stagger: if last { g.stagger } else { 0.0 },
                    knock,
                    crit: last,
                    dose,
                    flat: false,
                },
            );
        }
        if last {
            self.hitstop = self.hitstop.max(t[P::Hitstop] * ULT_HITSTOP);
        }
        ev.push(Event::GaleThrust {
            at: p,
            dir,
            len,
            half: g.half,
            last,
        });
        self.skill_ev = ev;
    }

    /// 그림자 습격(0274 ③)의 시작: 닿는 거리 안의 한 적(락온 > 조준 쪽 ±45° > 가장 가까운)을 고르고 그 적의 **등 뒤**로 숨어든다 —
    /// 등 뒤가 막혔으면(벽 · 기둥 · 걸을 수 없는 칸 · 높이가 다른 땅) 옆, 그도 막혔으면 이쪽 곁. 다 막혔으면 제자리에서 그 적 쪽을 본다.
    /// 숨어든 뒤에는 그 적을 본다. 적이 없으면 제자리(앞을 찌른다).
    fn ambush_blink(&mut self, cx: &Ctx) {
        let pl = &self.player;
        let p = pl.pos;
        let body = p.y + pl.air;
        let range = skills::AMBUSH.range_at(pl.skill.ult_rank);
        let reachable = |en: &crate::enemy::Enemy| {
            en.active()
                && !en.harmless()
                && (en.pos.y - body).abs() <= HIT_DY
                && vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z).length() - en.radius <= range
        };
        let dist = |en: &crate::enemy::Enemy| vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z).length();
        let locked = cx.target.and_then(|tg| {
            (0..self.enemies.len()).find(|&i| {
                let en = &self.enemies[i];
                reachable(en) && vec3(en.pos.x - tg.x, 0.0, en.pos.z - tg.z).length() < 0.5
            })
        });
        let aimed = cx.aim.and_then(|a| {
            (0..self.enemies.len())
                .filter(|&i| {
                    let en = &self.enemies[i];
                    let d = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z).normalize_or_zero();
                    reachable(en) && d.dot(a) > AMBUSH_AIM_COS
                })
                .min_by(|&a, &b| dist(&self.enemies[a]).total_cmp(&dist(&self.enemies[b])))
        });
        let nearest = || {
            (0..self.enemies.len())
                .filter(|&i| reachable(&self.enemies[i]))
                .min_by(|&a, &b| dist(&self.enemies[a]).total_cmp(&dist(&self.enemies[b])))
        };
        let pick = locked.or(aimed).or_else(nearest);
        self.player.skill.ambush_on = pick;
        let Some(i) = pick else {
            return;
        };
        let en = &self.enemies[i];
        let (q, back) = (en.pos, -en.facing_dir());
        let gap = en.radius + PLAYER_RADIUS + AMBUSH_GAP;
        let toward = vec3(p.x - q.x, 0.0, p.z - q.z).normalize_or(back);
        let turn = |v: Vec3, deg: f32| {
            let (s, c) = deg.to_radians().sin_cos();
            vec3(v.x * c + v.z * s, 0.0, -v.x * s + v.z * c)
        };
        let spot = [
            back,
            turn(back, 60.0),
            turn(back, -60.0),
            turn(back, 120.0),
            turn(back, -120.0),
            toward,
        ]
        .into_iter()
        .map(|d| q + d * gap)
        .find(|&s| self.ambush_fits(q, s));
        let from = self.player.pos;
        if let Some(s) = spot {
            let to = if self.ground.is_some() {
                vec3(s.x, self.ground_y(s), s.z)
            } else {
                vec3(s.x, from.y, s.z)
            };
            let pl = &mut self.player;
            pl.pos = to;
            pl.prev = to;
            pl.vel = Vec3::ZERO;
            self.skill_ev.push(Event::Blink { from, to });
        }
        let pl = &mut self.player;
        let d = vec3(q.x - pl.pos.x, 0.0, q.z - pl.pos.z);
        if d.length_squared() > 1e-4 {
            pl.facing = d.x.atan2(d.z);
        }
    }

    /// 숨어들 자리 `s`가 서는 곳인가: 그 적(`q`)에서 `s`까지 벽 · 기둥에 막히지 않고, 땅이 있는 지역이면 걸을 수 있는 칸이며 그 적과 같은
    /// 높이(한 걸음 `GROUND_STEP` 안)다.
    fn ambush_fits(&self, q: Vec3, s: Vec3) -> bool {
        let d = vec3(s.x - q.x, 0.0, s.z - q.z);
        let len = d.length();
        let dir = d / len.max(1e-4);
        let eye = q + Vec3::Y * BLADE_HEIGHT;
        if self
            .obstacles
            .iter()
            .any(|o| o.ray(eye, dir).is_some_and(|t| t < len + PLAYER_RADIUS))
        {
            return false;
        }
        match &self.ground {
            None => true,
            Some(g) => g.cell_of(s).is_some_and(|(x, z)| {
                g.height_in(x, z, s)
                    .is_some_and(|h| (h - q.y).abs() <= GROUND_STEP)
            }),
        }
    }

    /// 그림자 습격의 `k`째 찌름: 고른 적이 닿는 거리(`AMBUSH.reach` + 몸) 안이면 그 적만 — 아니면(없거나 멀어졌으면) 앞 부채꼴의
    /// 깨어 있는 적. 앞의 찌름은 가볍게(균열 · 잡몹은 잠깐 휘청 — 등 뒤의 급소가 이어진다), 마지막은 깊게(출혈 · 휘청 · 조금 밀림).
    /// 보스는 다치고 자세가 깎이되 휘청이지도 밀리지도 않는다(`skill_touch`가 휘청을 거른다).
    pub(super) fn ambush_stab(&mut self, t: &Tuning, k: u8) {
        let a = &skills::AMBUSH;
        let mut ev = std::mem::take(&mut self.skill_ev);
        let last = k + 1 >= a.stabs;
        let p = self.player.pos;
        let body = p.y + self.player.air;
        let near = |en: &crate::enemy::Enemy| {
            en.active()
                && (en.pos.y - body).abs() <= HIT_DY
                && vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z).length() - en.radius <= a.reach
        };
        let held = self
            .player
            .skill
            .ambush_on
            .filter(|&i| self.enemies.get(i).is_some_and(near));
        if let Some(i) = held {
            let q = self.enemies[i].pos;
            let d = vec3(q.x - p.x, 0.0, q.z - p.z);
            if d.length_squared() > 1e-4 {
                self.player.facing = d.x.atan2(d.z);
            }
        }
        let dir = self.player.facing_dir();
        let cone = (AMBUSH_CONE_DEG * 0.5).to_radians().cos();
        let targets: Vec<usize> = match held {
            Some(i) => vec![i],
            None => (0..self.enemies.len())
                .filter(|&i| {
                    let en = &self.enemies[i];
                    let d = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z).normalize_or(dir);
                    near(en) && d.dot(dir) >= cone
                })
                .collect(),
        };
        let rank = self.player.skill.ult_rank;
        let (light, deep) = a.damage_at(rank);
        for i in targets {
            let boss = self.enemies[i].boss;
            let at = self.enemies[i].pos + Vec3::Y * self.enemies[i].height * 0.55;
            if last {
                self.sparks
                    .extend(sparks_at(at, dir, self.player.swings, true));
            }
            let knock = if last && !boss {
                dir * AMBUSH_PUSH * KNOCK_DECAY
            } else {
                Vec3::ZERO
            };
            self.skill_touch(
                t,
                &mut ev,
                i,
                &Touch {
                    damage: if last { deep } else { light },
                    poise: if last { a.last_poise } else { a.poise },
                    stagger: if last { a.stagger } else { AMBUSH_HOLD },
                    knock,
                    crit: last,
                    dose: self.dosed(if last { a.dose } else { a.light_dose }),
                    flat: false,
                },
            );
        }
        if last {
            self.hitstop = self.hitstop.max(t[P::Hitstop] * ULT_HITSTOP);
        }
        ev.push(Event::AmbushStab { at: p, dir, last });
        self.skill_ev = ev;
    }

    /// 큰 종(0270 C3): 둘레의 깨어 있는 적을 한 번 친다 — 울림과 흡수를 함께 건다(큰 기술이라 보스도 다치고 걸리지만 휘청이지
    /// 않는다 — `skill_touch`가 보스의 휘청을 거른다). 2단 = 넓게, 3단 = 세게.
    pub(super) fn great_bell(&mut self, t: &Tuning) {
        let mut ev = std::mem::take(&mut self.skill_ev);
        let p = self.player.pos;
        let body = p.y + self.player.air;
        let rank = self.player.skill.ult_rank;
        let radius = skills::BELL.radius_at(rank);
        let damage = skills::BELL.damage_at(rank);
        let (dose, drain) = (
            self.dosed(skills::BELL.dose),
            self.dosed(skills::BELL.drain),
        );
        for i in 0..self.enemies.len() {
            let en = &self.enemies[i];
            if !en.active() || (en.pos.y - body).abs() > HIT_DY {
                continue;
            }
            let dv = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z);
            if dv.length() - en.radius > radius {
                continue;
            }
            let knock = dv.normalize_or_zero() * t[P::Knockback] * if en.boss { 0.2 } else { 1.0 };
            let down = self.skill_touch(
                t,
                &mut ev,
                i,
                &Touch {
                    damage,
                    poise: skills::BELL.poise,
                    stagger: skills::BELL.stagger,
                    knock,
                    crit: true,
                    dose,
                    flat: false,
                },
            );
            if !down {
                afflict(&mut self.enemies[i], drain, &mut ev);
            }
        }
        self.hitstop = self.hitstop.max(t[P::Hitstop] * ULT_HITSTOP);
        ev.push(Event::GreatBell { at: p, radius });
        self.skill_ev = ev;
    }

    /// 눈보라(0270 C3): 몸을 따라 도는 눈보라를 일으킨다 — 치는 것은 따르는 마당(`tick_fields` — 잔불 흩기의 따르는 불티와 같은 길).
    pub(super) fn blizzard(&mut self) {
        let p = self.player.pos;
        let rank = self.player.skill.ult_rank;
        let b = &skills::BLIZZARD;
        self.skill_fields.push(Field {
            pos: p,
            radius: b.radius,
            age: 0.0,
            secs: b.secs_at(rank),
            every: b.every,
            pulses: 0,
            damage: b.damage_at(rank),
            poise: 0.0,
            dose: self.dosed((crate::status::Kind::Frost, b.build)),
            follow: true,
            kindle: 0,
            drift: Vec3::ZERO,
            trap: false,
            spire: None,
        });
        self.skill_kill_left = ULT_KILL_WINDOW;
        self.skill_ev.push(Event::Blizzard {
            at: p,
            radius: b.radius,
        });
    }

    /// 화살비(0272)를 내릴 쪽과 거리: 조준(그 쪽 `ahead` m) > 락온한 적의 자리 > 닿는 거리의 가장 가까운 적의 자리 >
    /// 바라보는 쪽 `ahead` m. 적의 자리는 닿는 거리(`ahead` + 둘레)까지만 — 벽 너머로는 놓지 않는다(`place_ahead`).
    fn rain_aim(&self, cx: &Ctx) -> (Vec3, f32) {
        let r = &skills::RAIN;
        let pl = &self.player;
        let reach = r.ahead + r.radius;
        let flat = |p: Vec3| vec3(p.x - pl.pos.x, 0.0, p.z - pl.pos.z);
        if let Some(d) = cx.aim {
            return (d, r.ahead);
        }
        let near = self
            .enemies
            .iter()
            .filter(|e| e.active())
            .map(|e| flat(e.pos))
            .filter(|v| v.length() <= reach)
            .min_by(|a, b| a.length().total_cmp(&b.length()));
        match cx.target.map(flat).or(near) {
            Some(v) if v.length() > 0.5 => (v.normalize(), v.length().min(reach)),
            _ => (pl.facing_dir(), r.ahead),
        }
    }

    /// 화살비(0272): 정해 둔 자리에 화살이 내린다 — 치는 것은 놓인 마당(`tick_fields` — 눈보라와 같은 길, 몸을 따르지 않는다).
    /// 화살마다 뿌리가 쌓여 그 안의 떼가 묶인다(보스는 느려지기만 — 타입의 규칙 그대로, 휘청은 없다).
    pub(super) fn arrow_rain(&mut self) {
        let r = &skills::RAIN;
        let rank = self.player.skill.ult_rank;
        let at = self.player.skill.rain_at;
        self.skill_fields.push(Field {
            pos: at,
            radius: r.radius,
            age: 0.0,
            secs: r.secs_at(rank),
            every: r.every,
            pulses: 0,
            damage: r.damage_at(rank),
            poise: 0.0,
            dose: self.dosed((crate::status::Kind::Root, r.build)),
            follow: false,
            kindle: 0,
            drift: Vec3::ZERO,
            trap: false,
            spire: None,
        });
        self.skill_kill_left = ULT_KILL_WINDOW;
        self.skill_ev.push(Event::ArrowRain {
            from: self.player.pos,
            at,
            radius: r.radius,
        });
    }

    /// 쟁반 뒤집기: 둘레의 모든 적이 바깥으로 쏟아진다 — 가까운 적일수록 멀리. 쏟기와 같은 길(`spilling`)이라 벽·기둥·턱에
    /// 박히면 `Event::Slam`, 거기에 큰 기술의 덤 피해(`slam_bonus`). 보스는 무거워 덜 밀리고 짧게 휘청인다.
    /// 단(0158): 2단 = 휘청이 길어진다(보스의 짧은 휘청은 그대로), 3단 = 벽에 박힌 피해가 는다.
    pub(super) fn upend(&mut self, t: &Tuning) {
        let p = self.player.pos;
        let rank = self.player.skill.ult_rank;
        let (stagger, slam) = (UPEND.stagger_at(rank), UPEND.slam_at(rank));
        for en in &mut self.enemies {
            if !en.active() || en.rising() {
                continue;
            }
            let dv = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z);
            let r = dv.length();
            if r - en.radius > UPEND.radius {
                continue;
            }
            let out = dv.normalize_or(self.player.facing_dir());
            let k = (r / UPEND.radius).clamp(0.0, 1.0);
            let mass = if en.boss { t[P::SpillBossMass] } else { 1.0 };
            let slide = (UPEND.near + (UPEND.far - UPEND.near) * k) / mass.max(1.0);
            en.knock = out * slide * KNOCK_DECAY;
            en.spilling = true;
            en.slam_bonus = slam;
            en.sway = out;
            en.sway_t = 0.0;
            en.state = EState::Stagger {
                t: 0.0,
                dur: if en.boss { UPEND.boss_stagger } else { stagger },
            };
        }
        self.player.since_spill = 0.0;
        self.skill_kill_left = ULT_KILL_WINDOW;
        self.hitstop = self.hitstop.max(t[P::Hitstop] * ULT_HITSTOP);
        self.skill_ev.push(Event::Ult { at: p });
    }
}
