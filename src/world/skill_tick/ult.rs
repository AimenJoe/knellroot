//! 큰 기술의 몸(0251 R11 — `skill_tick.rs`에서 뺐다): 시작(게이지를 통째로) · 한 줄기 가르기 · 쟁반 뒤집기.
use super::*;

impl World {
    /// 큰 기술 시작(끼운 하나): 게이지를 통째로 쓴다. 쟁반 뒤집기·한 줄기는 세계가 한 번 멈춘다(시동 정지 — "시간"),
    /// 한 줄기는 가를 쪽을 이 순간에 정한다(조준 > 락온 > 닿는 거리의 가장 가까운 적 > 바라보는 쪽).
    pub(super) fn start_ult(&mut self, cx: &Ctx) {
        let kind = self.player.skill.ult;
        if kind == skills::Ult::Line {
            let pl = &self.player;
            let lock = cx
                .target
                .map(|tg| vec3(tg.x - pl.pos.x, 0.0, tg.z - pl.pos.z).normalize_or_zero())
                .filter(|v| v.length_squared() > 0.5);
            let reach = LINE.len_at(pl.skill.ult_rank);
            let dir = cx
                .aim
                .or(lock)
                .or_else(|| nearest_in_reach(&self.enemies, pl.pos, reach, None));
            if let Some(d) = dir {
                self.player.facing = d.x.atan2(d.z);
            }
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
