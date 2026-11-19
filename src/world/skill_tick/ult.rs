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
