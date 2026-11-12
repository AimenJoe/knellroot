//! 스킬 틱(0151, 표는 `skills.rs`): 누름 → 받아 두기(`skill_press`), 시작(`try_skill`), 상태 틱(`tick_skill`·`tick_ult`),
//! 판정(`skill_area`·`skill_pull`·`upend`), 스킬이 남긴 것(`tick_skill_fields` — 곧 터질 자리·갈라진 땅, 0158 G1),
//! 틱 끝의 셈(`skill_books` — 기세·게이지). `tick_player`의 한 갈래다.
//!
//! 단과 고르기(0158): 숫자는 `World::skill_eff`(= `Skill::at(단, 고르기)`)가 준 값만 쓴다 — 여기는 `Twist`·`Cap`의 갈래만 본다.
use super::player_tick::Ctx;
use super::*;
use crate::skills::{Cap, Eff, Form, Twist, UPEND};
use crate::tuning::{SKILL_DENIED_SHOW, SKILL_HITSTOP, ULT_HITSTOP, ULT_KILL_WINDOW};

/// 곧 터질 "모인 자리"(끌어모으기 ㄱ): `left`초 뒤 `at` 둘레 `radius` m에 `damage`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Burst {
    pub at: Vec3,
    pub left: f32,
    pub radius: f32,
    pub damage: i32,
}

/// 갈라진 땅(땅울림 ㄴ): 찍은 부채꼴이 `left`초 남는다 — 그 안의 적은 `slow`배로 걷는다. 렌더가 바닥에 그린다.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Zone {
    pub pos: Vec3,
    pub dir: Vec3,
    pub range: f32,
    pub arc_deg: f32,
    pub left: f32,
    pub total: f32,
    pub slow: f32,
}

impl Zone {
    /// 이 자리(몸 반지름 `radius`)가 부채꼴 안인가.
    pub fn holds(&self, at: Vec3, radius: f32) -> bool {
        let dv = vec3(at.x - self.pos.x, 0.0, at.z - self.pos.z);
        let r = dv.length();
        (at.y - self.pos.y).abs() <= HIT_DY
            && r - radius <= self.range
            && (r < 1e-3 || (dv / r).dot(self.dir) >= (self.arc_deg * 0.5).to_radians().cos())
    }
}

/// 스킬의 넓은 타 한 번(`skill_area`): 닿는 거리·부채꼴(쪽, cos 반각)·밀기·휘청·타 번호·피해·자세 피해·깨진 적에게의 배.
struct Sweep {
    reach: f32,
    cone: Option<(Vec3, f32)>,
    push: f32,
    stagger: f32,
    swing: u32,
    damage: i32,
    poise: f32,
    shatter: f32,
}

impl World {
    /// 스킬 `id`의 지금 값: 끼운 칸의 단·고르기를 입힌 것(끼우지 않은 스킬은 1단 — 시험·바꿔 끼운 직후).
    pub fn skill_eff(&self, id: skills::Id) -> Eff {
        let pl = &self.player;
        match pl.skills.iter().position(|s| *s == id) {
            Some(i) => id.def().at(pl.skill_rank[i], pl.skill_pick[i]),
            None => id.def().at(1, skills::Pick::A),
        }
    }

    /// 이번 틱의 스킬 누름 → 받아 둘 행동(0151 3절). 둘을 함께 눌렀거나(같은 틱) 터치의 큰 기술 버튼이면 큰 기술,
    /// 스킬의 시동 안(`skill.chord s`)에 **다른** 스킬 버튼이 들어와도 큰 기술 — 게이지가 가득일 때만. 그 밖에는 그 칸의 스킬.
    /// 대기가 한참 남은 스킬은 받아 두지 않고 칸만 깜빡인다(몇 초 뒤에 저절로 나가면 안 된다).
    pub(super) fn skill_press(&mut self, a: &Actions, t: &Tuning) -> Option<Next> {
        let pl = &mut self.player;
        let full = pl.gauge >= 1.0;
        if a.ult || (a.skill1 && a.skill2) {
            if full {
                return Some(Next::Ult);
            }
            if a.ult {
                return None;
            }
        }
        let slot = if a.skill1 {
            0
        } else if a.skill2 {
            1
        } else {
            return None;
        };
        if let PState::Skill { id, t: e, .. } = pl.state {
            if full && e < t[P::SkillChord] && pl.skills[slot] != id {
                return Some(Next::Ult);
            }
        }
        if pl.skill_cd[pl.skills[slot].index()] > t[P::AttackBuffer] {
            if pl.skill_denied <= 0.0 {
                self.skill_ev.push(Event::SkillDenied);
            }
            pl.skill_denied = SKILL_DENIED_SHOW;
            return None;
        }
        Some(Next::Skill(slot as u8))
    }

    /// 서 있을 때 받은 스킬 누름(캔슬로 넘어온 것 포함)을 시작한다.
    pub(super) fn try_skill(&mut self, cx: &Ctx) -> bool {
        let a = &cx.a;
        if a.ult {
            if self.player.gauge < 1.0 {
                return false;
            }
            self.start_ult();
            return true;
        }
        let slot = if a.skill1 {
            0
        } else if a.skill2 {
            1
        } else {
            return false;
        };
        let id = self.player.skills[slot];
        if self.player.skill_cd[id.index()] > 0.0 {
            return false;
        }
        let d = self.skill_eff(id);
        let dir = self.skill_dir(cx, &d);
        // 멀리 뛰기(땅울림 ㄱ): 뛰어들 거리는 누른 순간에 정한다 — 그 길의 가장 가까운 적 앞까지.
        let leap = match d.twist {
            Some(Twist::Leap { dist, short, half }) => self.leap_len(dir, dist, short, half),
            _ => 0.0,
        };
        let pl = &mut self.player;
        pl.skill_cd[id.index()] = d.skill.cooldown * cx.t[P::SkillCooldownX];
        pl.skill_leap = leap;
        pl.skill_refunded = false;
        pl.skill_ghost = false;
        pl.swings += 1;
        pl.facing = dir.x.atan2(dir.z);
        pl.state = PState::Skill {
            id,
            t: 0.0,
            dir,
            swing: pl.swings,
        };
        self.skill_ev.push(Event::SkillStart { id });
        true
    }

    /// 스킬이 나가는 쪽: 가로지르기는 쥔 스틱 쪽(떼를 비껴 뚫을 수 있게), 겨누는 스킬은 조준 > 락온 > 닿는 거리의 가장 가까운 적,
    /// 없으면 바라보는 쪽. 뛰어드는 땅울림은 뛰는 거리만큼 더 멀리서도 겨눈다.
    fn skill_dir(&self, cx: &Ctx, d: &Eff) -> Vec3 {
        let pl = &self.player;
        let facing = pl.facing_dir();
        let lock = cx
            .target
            .map(|tg| vec3(tg.x - pl.pos.x, 0.0, tg.z - pl.pos.z).normalize_or_zero())
            .filter(|v| v.length_squared() > 0.5);
        match d.skill.form {
            Form::Ring { .. } => facing,
            Form::Lane { .. } => {
                if cx.world_move.length_squared() > 1e-4 {
                    cx.world_move.normalize()
                } else {
                    cx.aim.or(lock).unwrap_or(facing)
                }
            }
            Form::Pull { range, .. } | Form::Cone { range, .. } => {
                let reach = range
                    + match d.twist {
                        Some(Twist::Leap { dist, .. }) => dist,
                        _ => 0.0,
                    };
                cx.aim
                    .or(lock)
                    .or_else(|| nearest_in_reach(&self.enemies, pl.pos, reach, None))
                    .unwrap_or(facing)
            }
        }
    }

    /// 멀리 뛰기의 거리: `dir` 쪽 길(반 폭 `half` m) 위의 가장 가까운 깨어 있는 적 `short` m 앞까지, 많아야 `dist` m.
    /// 길에 적이 없으면 `dist` m, 바로 앞에 있으면 0(제자리에서 찍는다).
    fn leap_len(&self, dir: Vec3, dist: f32, short: f32, half: f32) -> f32 {
        let p = self.player.pos;
        let body = p.y + self.player.air;
        let mut len = dist;
        for en in &self.enemies {
            if !en.active() || (en.pos.y - body).abs() > HIT_DY {
                continue;
            }
            let dv = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z);
            let along = dv.dot(dir);
            let off = (dv - dir * along).length();
            if along <= 0.0 || off > half + en.radius {
                continue;
            }
            len = len.min(along - en.radius - short);
        }
        len.clamp(0.0, dist)
    }

    pub(super) fn tick_skill(&mut self, cx: &Ctx, id: skills::Id, e: f32, dir: Vec3, swing: u32) {
        let (t, dt) = (cx.t, cx.dt);
        let d = self.skill_eff(id);
        let s = d.skill;
        // 두 버튼 함께(0151): 시동 안에 들어온 큰 기술 누름이 이 스킬을 바꿔친다 — 쓴 대기는 돌려준다.
        if cx.a.ult && e < t[P::SkillChord] && self.player.gauge >= 1.0 {
            self.player.skill_cd[id.index()] = 0.0;
            self.player.skill_ghost = false;
            self.start_ult();
            return;
        }
        let e1 = e + dt;
        let crossed = |at: f32| e < at && e1 >= at;
        let at = self.player.pos;
        let mut swing = swing;
        self.player.skill_ghost = false;
        match s.form {
            Form::Ring {
                radius,
                hits,
                gap,
                push,
            } => {
                // 몸이 판정과 함께 돈다(바퀴 수 = 타 수) — 끝나면 처음 보던 쪽.
                let span = (gap * hits as f32).max(1e-3);
                let turns = ((e1 - s.startup) / span).clamp(0.0, 1.0) * hits as f32;
                self.player.facing = dir.x.atan2(dir.z) + turns * std::f32::consts::TAU;
                for k in 0..hits {
                    if !crossed(s.startup + gap * k as f32) {
                        continue;
                    }
                    // 바퀴마다 새 타 번호 — 같은 적을 바퀴마다 한 번씩 친다.
                    if k > 0 {
                        self.player.swings += 1;
                        swing = self.player.swings;
                    }
                    self.skill_ev.push(Event::Skill { id, at, dir });
                    // 바깥으로 미는 것은 마지막 바퀴만 — 첫 바퀴가 밀어내면 둘째 바퀴가 허공을 벤다. 맺음의 휘청도 마지막 바퀴에.
                    let last = k + 1 == hits;
                    let lift = match d.cap {
                        Some(Cap::Lift { stagger }) if last => stagger,
                        _ => 0.0,
                    };
                    self.skill_area(
                        t,
                        &Sweep {
                            reach: radius,
                            cone: None,
                            push: if last { push } else { 0.0 },
                            stagger: lift,
                            swing,
                            damage: s.damage,
                            poise: s.poise,
                            shatter: 1.0,
                        },
                    );
                }
            }
            Form::Lane { half, time, .. } => {
                if crossed(s.startup) {
                    self.skill_ev.push(Event::Skill { id, at, dir });
                }
                // 되돌아 베기(ㄱ): 끝에서 새 타 번호로 온 길을 되돌아간다 — 길 위의 적을 한 번 더 벤다.
                if matches!(d.twist, Some(Twist::Return)) && crossed(s.startup + time) {
                    self.player.swings += 1;
                    swing = self.player.swings;
                    self.skill_ev.push(Event::Skill { id, at, dir: -dir });
                }
                // 지나가는 동안 틱마다 몸 둘레를 친다(한 번 맞은 적은 다시 안 맞는다).
                if e1 >= s.startup && e < s.startup + d.active() {
                    // 스치는 몸(ㄴ): 지나는 동안 맞지 않는다.
                    self.player.skill_ghost = matches!(d.twist, Some(Twist::Ghost));
                    let kills = self.kills;
                    self.skill_area(
                        t,
                        &Sweep {
                            reach: half,
                            cone: None,
                            push: 0.0,
                            stagger: 0.0,
                            swing,
                            damage: s.damage,
                            poise: s.poise,
                            shatter: 1.0,
                        },
                    );
                    // 맺음: 이 스킬로 쓰러뜨리면 대기의 몫을 돌려받는다(한 번 쓸 때 한 번).
                    if let Some(Cap::Refund { part }) = d.cap {
                        let pl = &mut self.player;
                        if self.kills > kills && !pl.skill_refunded {
                            pl.skill_refunded = true;
                            let back = s.cooldown * t[P::SkillCooldownX] * part;
                            let cd = &mut pl.skill_cd[id.index()];
                            *cd = (*cd - back).max(0.0);
                        }
                    }
                }
            }
            Form::Pull {
                range,
                arc_deg,
                to,
                stagger,
            } => {
                if crossed(s.startup) {
                    let heavy = matches!(d.twist, Some(Twist::Heavy));
                    let frail = match d.cap {
                        Some(Cap::Frail { secs, mult }) => Some((secs, mult)),
                        _ => None,
                    };
                    let spot = self.skill_pull(id, dir, range, arc_deg, to, stagger, heavy, frail);
                    // 모인 자리(ㄱ): 곧이어 그 자리가 터진다.
                    if let Some(Twist::Burst {
                        delay,
                        radius,
                        damage,
                    }) = d.twist
                    {
                        self.skill_bursts.push(Burst {
                            at: spot,
                            left: delay,
                            radius,
                            damage,
                        });
                    }
                }
            }
            Form::Cone {
                range,
                arc_deg,
                stagger,
            } => {
                if crossed(s.startup) {
                    self.skill_ev.push(Event::Skill { id, at, dir });
                    let half = (arc_deg * 0.5).to_radians().cos();
                    let shatter = match d.cap {
                        Some(Cap::Shatter { mult }) => mult,
                        _ => 1.0,
                    };
                    self.skill_area(
                        t,
                        &Sweep {
                            reach: range,
                            cone: Some((dir, half)),
                            push: 0.0,
                            stagger,
                            swing,
                            damage: s.damage,
                            poise: s.poise,
                            shatter,
                        },
                    );
                    // 갈라진 땅(ㄴ): 찍은 부채꼴이 남는다.
                    if let Some(Twist::Crack { secs, slow }) = d.twist {
                        self.skill_zones.push(Zone {
                            pos: at,
                            dir,
                            range,
                            arc_deg,
                            left: secs,
                            total: secs,
                            slow,
                        });
                    }
                }
            }
        }
        self.player.state = if e1 >= d.total() {
            self.player.skill_ghost = false;
            PState::Idle
        } else {
            PState::Skill {
                id,
                t: e1,
                dir,
                swing,
            }
        };
    }

    /// 스킬의 넓은 타: 방랑자 둘레 `reach` m(부채꼴이면 `cone` = (쪽, cos 반각)) 안의 깨어 있는 적을 한 번씩 친다.
    /// 등딱지도 뚫는다(강타처럼). 맞은 적은 바깥으로 `push` m 밀리고, `stagger`초 휘청인다(보스는 휘청이지 않는다).
    /// 스킬의 타는 큰 기술 게이지를 채우지 않는다(`connected`를 건드리지 않는다).
    fn skill_area(&mut self, t: &Tuning, sw: &Sweep) -> u32 {
        let mut ev = std::mem::take(&mut self.skill_ev);
        let p = self.player.pos;
        let body = p.y + self.player.air;
        let facing = self.player.facing_dir();
        let mut n = 0;
        for en in &mut self.enemies {
            if !en.active() || en.hit_by == sw.swing || (en.pos.y - body).abs() > HIT_DY {
                continue;
            }
            let dv = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z);
            if dv.length() - en.radius > sw.reach {
                continue;
            }
            let dir = dv.normalize_or(facing);
            if sw.cone.is_some_and(|(f, half)| dir.dot(f) < half) {
                continue;
            }
            en.hit_by = sw.swing;
            n += 1;
            let at = en.pos + Vec3::Y * en.height * 0.6;
            let crit = en.exposed > 0.0;
            let dmg = hurt_by(
                en,
                self.stats.dmg + sw.damage,
                crit,
                self.stats.opened_dmg,
                t,
            );
            // 땅울림 맺음: 이미 자세가 깨져 있던 적에게는 배로(이 타로 깨진 적은 다음부터).
            en.hp -= if en.broken > 0.0 {
                skills::scaled(dmg, sw.shatter)
            } else {
                dmg
            };
            if crit {
                en.exposed = 0.0;
                ev.push(Event::Crit { at });
            }
            en.hurt = 0.15;
            let mass = if en.boss { 0.25 } else { 1.0 };
            en.knock = dir * (t[P::Knockback] * 0.5 + sw.push * KNOCK_DECAY) * mass;
            en.sway = dir * HEAVY_SWAY;
            en.sway_t = 0.0;
            self.sparks.extend(sparks_at(
                en.pos + Vec3::Y * en.height * 0.55,
                dir,
                sw.swing,
                true,
            ));
            add_poise(en, sw.poise * self.stats.poise, None, t, &mut ev);
            ev.push(Event::Hit { at });
            if en.hp <= 0 {
                en.state = EState::Dead;
                self.kills += 1;
                ev.push(Event::EnemyDown { at: en.pos });
            } else if sw.stagger > 0.0 && !en.boss && !en.rising() {
                en.state = EState::Stagger {
                    t: 0.0,
                    dur: sw.stagger,
                };
            }
        }
        if n > 0 {
            self.skill_hit_tick = true;
            self.hitstop = self.hitstop.max(t[P::Hitstop] * SKILL_HITSTOP);
        }
        self.skill_ev = ev;
        n
    }

    /// 끌어모으기: 앞 부채꼴의 적이 방랑자 앞 한 점으로 미끄러져 모인다(몸끼리는 `collide`가 벌린다). 피해 없음.
    /// 보스는 끌리지 않는다(무겁다 — 0150 4절). 엘리트는 `heavy`(3단 ㄴ)일 때만 끌린다. `frail`(맺음) = 끌려온 적이 잠깐 더 다친다.
    /// 모이는 점을 돌려준다.
    #[allow(clippy::too_many_arguments)]
    fn skill_pull(
        &mut self,
        id: skills::Id,
        dir: Vec3,
        range: f32,
        arc_deg: f32,
        to: f32,
        stagger: f32,
        heavy: bool,
        frail: Option<(f32, f32)>,
    ) -> Vec3 {
        let p = self.player.pos;
        let body = p.y + self.player.air;
        let spot = p + dir * to;
        let half = (arc_deg * 0.5).to_radians().cos();
        for en in &mut self.enemies {
            if !en.active() || en.boss || (en.elite.is_some() && !heavy) || en.rising() {
                continue;
            }
            if (en.pos.y - body).abs() > HIT_DY {
                continue;
            }
            let dv = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z);
            let r = dv.length();
            if r - en.radius > range || r < 1e-3 || (dv / r).dot(dir) < half {
                continue;
            }
            let go = vec3(spot.x - en.pos.x, 0.0, spot.z - en.pos.z);
            let slide = (go.length() - en.radius).max(0.0);
            let along = go.normalize_or_zero();
            en.knock = along * slide * KNOCK_DECAY;
            en.spilling = false;
            en.sway = along * HEAVY_SWAY;
            en.sway_t = 0.0;
            en.state = EState::Stagger {
                t: 0.0,
                dur: stagger,
            };
            if let Some((secs, mult)) = frail {
                en.frail = secs;
                en.frail_mult = mult;
            }
        }
        self.skill_ev.push(Event::Skill { id, at: spot, dir });
        spot
    }

    /// 스킬이 남긴 것의 틱(0158 G1) — 방랑자가 다른 일을 하는 동안에도 돈다(히트스톱 동안은 멈춘다):
    /// "모인 자리"는 때가 되면 둘레를 치고, 갈라진 땅은 그 안의 적을 느리게 한다(보스는 아니다).
    pub(super) fn tick_skill_fields(&mut self, t: &Tuning, dt: f32) {
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
            en.slow = 1.0;
            if en.boss || !en.active() {
                continue;
            }
            for z in &self.skill_zones {
                if z.holds(en.pos, en.radius) {
                    en.slow = en.slow.min(z.slow);
                }
            }
        }
    }

    /// 모인 자리가 터진다: 둘레의 깨어 있는 적에게 피해(밀지 않는다 — 모아 둔 것을 흩지 않게, 휘청도 없다).
    /// 스킬의 타라 게이지는 채우지 않고, 쓰러뜨리면 스킬 몫의 기세.
    fn skill_burst(&mut self, t: &Tuning, b: Burst) {
        let mut ev = std::mem::take(&mut self.skill_ev);
        let mut n = 0;
        for en in &mut self.enemies {
            if !en.active() || (en.pos.y - b.at.y).abs() > HIT_DY {
                continue;
            }
            let dv = vec3(en.pos.x - b.at.x, 0.0, en.pos.z - b.at.z);
            if dv.length() - en.radius > b.radius {
                continue;
            }
            n += 1;
            let at = en.pos + Vec3::Y * en.height * 0.6;
            let crit = en.exposed > 0.0;
            en.hp -= hurt_by(
                en,
                self.stats.dmg + b.damage,
                crit,
                self.stats.opened_dmg,
                t,
            );
            if crit {
                en.exposed = 0.0;
                ev.push(Event::Crit { at });
            }
            en.hurt = 0.15;
            ev.push(Event::Hit { at });
            if en.hp <= 0 {
                en.state = EState::Dead;
                self.kills += 1;
                ev.push(Event::EnemyDown { at: en.pos });
            }
        }
        if n > 0 {
            self.skill_hit_tick = true;
            self.hitstop = self.hitstop.max(t[P::Hitstop] * SKILL_HITSTOP);
        }
        ev.push(Event::SkillBurst { at: b.at });
        self.skill_ev = ev;
    }

    /// 큰 기술 시작: 게이지를 통째로 쓰고 세계가 한 번 멈춘다(시동 정지 — "시간").
    fn start_ult(&mut self) {
        let pl = &mut self.player;
        pl.gauge = 0.0;
        pl.state = PState::Ult {
            t: 0.0,
            done: false,
        };
        self.hitstop = self.hitstop.max(UPEND.freeze);
        self.skill_ev.push(Event::UltStart);
    }

    pub(super) fn tick_ult(&mut self, cx: &Ctx, e: f32, done: bool) {
        let e1 = e + cx.dt;
        let mut done = done;
        if !done && e1 >= UPEND.startup {
            done = true;
            self.upend(cx.t);
        }
        self.player.state = if e1 >= UPEND.startup + UPEND.recover {
            PState::Idle
        } else {
            PState::Ult { t: e1, done }
        };
    }

    /// 쟁반 뒤집기: 둘레의 모든 적이 바깥으로 쏟아진다 — 가까운 적일수록 멀리. 쏟기와 같은 길(`spilling`)이라 벽·기둥·턱에
    /// 박히면 `Event::Slam`, 거기에 큰 기술의 덤 피해(`slam_bonus`). 보스는 무거워 덜 밀리고 짧게 휘청인다.
    /// 단(0158): 2단 = 휘청이 길어진다(보스의 짧은 휘청은 그대로), 3단 = 벽에 박힌 피해가 는다.
    fn upend(&mut self, t: &Tuning) {
        let p = self.player.pos;
        let rank = self.player.ult_rank;
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

    /// 틱 끝의 셈(0151 5절). **기세**: 이번 틱에 쓰러진 적마다 모든 스킬의 대기가 당겨진다 — 스킬이 아닌 것으로 쓰러뜨렸으면
    /// `skill.momentum s`, 스킬(또는 큰 기술이 쏟은 뒤의 벽 박힘)이면 `skill.momentum by skill s`. **게이지**: 맞힌 휘두르기 한 번에
    /// 한 번(광역이 몇 배로 채우지 않는다)·탄이 맞은 틱·패링·딱 맞춘 떼기 — 스킬의 타는 채우지 않는다.
    pub(super) fn skill_books(&mut self, t: &Tuning, ev: &mut Vec<Event>) {
        let count = |f: fn(&Event) -> bool| ev.iter().filter(|e| f(e)).count() as f32;
        let downs = count(|e| matches!(e, Event::EnemyDown { .. }));
        let parries = count(|e| matches!(e, Event::Parry { .. }));
        let perfects = count(|e| matches!(e, Event::Perfect { .. }));
        let by_skill = self.skill_hit_tick || self.skill_kill_left > 0.0;
        let pl = &mut self.player;
        if downs > 0.0 {
            let per = if by_skill {
                t[P::SkillMomentumOwn]
            } else {
                t[P::SkillMomentum]
            };
            for cd in &mut pl.skill_cd {
                *cd = (*cd - per * downs).max(0.0);
            }
        }
        let was_full = pl.gauge >= 1.0;
        let hit = t[P::SkillGaugeHit];
        let mut gain = parries * t[P::SkillGaugeParry] + perfects * hit * 2.0;
        if pl.connected != 0 && pl.connected != pl.gauge_swing {
            pl.gauge_swing = pl.connected;
            gain += hit;
        }
        if std::mem::take(&mut self.bolt_hit_tick) {
            gain += hit;
        }
        pl.gauge = (pl.gauge + gain).min(1.0);
        if !was_full && pl.gauge >= 1.0 {
            ev.push(Event::UltReady);
        }
    }

    /// 스킬 칸 `slot`(0·1)의 준비 0..1(1 = 쓸 수 있다) — HUD·터치.
    pub fn skill_ready(&self, slot: usize, t: &Tuning) -> f32 {
        let id = self.player.skills[slot % 2];
        let full = self.skill_eff(id).skill.cooldown * t[P::SkillCooldownX];
        if full <= 0.0 {
            1.0
        } else {
            1.0 - (self.player.skill_cd[id.index()] / full).clamp(0.0, 1.0)
        }
    }

    /// 스킬 둘을 끼운다(같은 것 둘은 바로잡는다) — 그 단·고르기와 큰 기술의 단까지(0158). 스킬 이름만 주면 모두 1단.
    /// 대기는 스킬마다라 그대로 남는다.
    pub fn set_skills(&mut self, l: impl Into<skills::Loadout>) {
        let l = l.into();
        let ids = skills::sane([l.slots[0].id, l.slots[1].id]);
        let pl = &mut self.player;
        pl.skills = ids;
        for (i, id) in ids.into_iter().enumerate() {
            // 같은 스킬 둘이라 바꿔 끼운 칸은 1단.
            let same = l.slots[i].id == id;
            pl.skill_rank[i] = if same { l.slots[i].rank.max(1) } else { 1 };
            pl.skill_pick[i] = if same {
                l.slots[i].pick
            } else {
                skills::Pick::A
            };
        }
        pl.ult_rank = l.ult.max(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::Id as Sk;
    use std::f32::consts::PI;

    /// 멈춘 적 셋·화면 위(-z)를 보는 방랑자.
    fn world() -> (World, Tuning) {
        let mut w = World::new("test", 30.0);
        w.player.facing = PI;
        (w, Tuning::default())
    }

    fn still(w: &mut World) {
        for e in &mut w.enemies {
            e.hp = 90;
            e.max_hp = 90;
            e.state = EState::Recover { t: -1000.0 };
        }
    }

    fn skill(slot: usize) -> Actions {
        Actions {
            skill1: slot == 0,
            skill2: slot == 1,
            ..Default::default()
        }
    }

    fn run(w: &mut World, t: &Tuning, a: Actions, ticks: usize) -> Vec<Event> {
        let mut ev = w.step(&a, t);
        for _ in 1..ticks {
            ev.extend(w.step(&Actions::default(), t));
        }
        ev
    }

    fn flat(a: Vec3, b: Vec3) -> f32 {
        vec3(a.x - b.x, 0.0, a.z - b.z).length()
    }

    /// 휘돌이: 둘레의 적을 **모두** 두 번 치고(바퀴마다 한 번) 바깥으로 민다. 기력은 쓰지 않고 대기가 돈다.
    #[test]
    fn whirl_cuts_everyone_around_twice_and_pushes_them_out() {
        let (mut w, t) = world();
        for k in 0..6 {
            let a = k as f32 / 6.0 * std::f32::consts::TAU;
            w.spawn_mob(vec3(a.cos() * 2.2, 0.0, a.sin() * 2.2));
        }
        w.spawn_mob(vec3(0.0, 0.0, -6.0));
        still(&mut w);
        let stamina = w.player.stamina;
        let ev = run(&mut w, &t, skill(0), 80);
        assert!(ev
            .iter()
            .any(|e| matches!(e, Event::SkillStart { id: Sk::Whirl })));
        let rounds = ev
            .iter()
            .filter(|e| matches!(e, Event::Skill { id: Sk::Whirl, .. }))
            .count();
        assert_eq!(rounds, 2, "one flash per turn");
        let dmg = Sk::Whirl.def().damage + w.stats.dmg;
        for e in &w.enemies[..6] {
            assert_eq!(e.hp, 90 - 2 * dmg, "hit once per turn");
            assert!(flat(e.pos, w.player.pos) > 2.8, "pushed out: {:?}", e.pos);
        }
        assert_eq!(w.enemies[6].hp, 90, "out of reach");
        assert_eq!(w.player.stamina, stamina, "skills cost no stamina");
        assert!(w.player.skill_cd[Sk::Whirl.index()] > 4.0, "cooling down");
        assert!(matches!(w.player.state, PState::Idle));
        // 두 바퀴를 돌고 처음 보던 쪽으로 돌아온다.
        assert!(w.player.facing_dir().dot(vec3(0.0, 0.0, -1.0)) > 0.99);
    }

    /// 가로지르기: 바라보는 쪽으로 6m를 지나가며 길 위의 적을 한 번씩 베고 반대편에 선다 — 적의 몸에 막히지 않는다.
    #[test]
    fn cut_through_passes_the_pack_and_cuts_everyone_on_the_way_once() {
        let (mut w, t) = world();
        for z in [-1.6, -3.0, -4.4] {
            w.spawn_mob(vec3(0.0, 0.0, z));
        }
        w.spawn_mob(vec3(4.0, 0.0, -3.0));
        still(&mut w);
        run(&mut w, &t, skill(1), 60);
        let d = Sk::Cut.def();
        let Form::Lane { dist, .. } = d.form else {
            panic!()
        };
        assert!(
            (w.player.pos.z + dist).abs() < 0.8 && w.player.pos.x.abs() < 0.3,
            "ends {dist} m ahead: {:?}",
            w.player.pos
        );
        for e in &w.enemies[..3] {
            assert_eq!(e.hp, 90 - d.damage - w.stats.dmg);
            assert!(e.pos.z > w.player.pos.z, "left behind");
        }
        assert_eq!(w.enemies[3].hp, 90, "off the lane");
    }

    /// 끌어모으기: 앞 부채꼴의 잡몹이 방랑자 앞 한 점으로 모이고 휘청인다 — 다치지는 않는다. 뒤의 적·보스·엘리트는 그대로.
    #[test]
    fn gather_draws_the_mobs_ahead_to_one_spot_without_hurting_them() {
        let (mut w, t) = world();
        w.set_skills([Sk::Gather, Sk::Quake]);
        for p in [(-2.5, -4.5), (2.5, -4.5), (0.0, -6.0)] {
            w.spawn_mob(vec3(p.0, 0.0, p.1));
        }
        w.spawn_mob(vec3(0.0, 0.0, 9.0));
        // 부채꼴 안이지만 끌려오는 길에서는 비켜 선 보스와 엘리트.
        w.spawn_boss(vec3(4.5, 0.0, -4.5));
        let elite = w.spawn(crate::species::Kind::Husk, vec3(-4.5, 0.0, -4.5));
        w.enemies[elite].make_elite(Affix::Hardened);
        still(&mut w);
        let was: Vec<Vec3> = w.enemies.iter().map(|e| e.pos).collect();
        let ev = run(&mut w, &t, skill(0), 14);
        assert!(ev
            .iter()
            .any(|e| matches!(e, Event::Skill { id: Sk::Gather, .. })));
        assert!(
            w.enemies[..3]
                .iter()
                .all(|e| matches!(e.state, EState::Stagger { .. })),
            "the gathered stagger"
        );
        run(&mut w, &t, Actions::default(), 50);
        let spot = vec3(0.0, 0.0, -1.8);
        for e in &w.enemies[..3] {
            assert!(flat(e.pos, spot) < 1.7, "gathered: {:?}", e.pos);
            assert_eq!(e.hp, 90, "gather does no harm");
        }
        for i in [3, 4, elite] {
            assert!(flat(w.enemies[i].pos, was[i]) < 0.05, "{i} stays");
        }
        assert!(!ev.iter().any(|e| matches!(e, Event::Hit { .. })));
    }

    /// 땅울림: 앞 부채꼴만 친다 — 맞은 잡몹은 휘청이고, 보스는 다치되 휘청이지 않는다(보스전의 박자를 무너뜨리지 않는다).
    #[test]
    fn quake_slams_the_cone_ahead_and_a_boss_does_not_stagger() {
        let (mut w, t) = world();
        w.set_skills([Sk::Quake, Sk::Gather]);
        // 겨누는 쪽 = 닿는 거리의 가장 가까운 적(앞의 것). 뒤·옆의 적은 부채꼴 밖.
        w.spawn_mob(vec3(0.5, 0.0, -3.0));
        w.spawn_mob(vec3(0.0, 0.0, 4.2));
        w.spawn_mob(vec3(4.0, 0.0, 0.5));
        w.spawn_boss(vec3(-1.0, 0.0, -3.5));
        still(&mut w);
        let boss_hp = w.enemies[3].hp;
        let ev = run(&mut w, &t, skill(0), 22);
        let d = Sk::Quake.def();
        assert_eq!(w.enemies[0].hp, 90 - d.damage - w.stats.dmg);
        assert!(matches!(w.enemies[0].state, EState::Stagger { .. }));
        assert_eq!(w.enemies[1].hp, 90, "behind");
        assert_eq!(w.enemies[2].hp, 90, "to the side");
        assert!(w.enemies[3].hp < boss_hp, "the boss is hurt");
        assert!(!matches!(w.enemies[3].state, EState::Stagger { .. }));
        assert_eq!(
            ev.iter().filter(|e| matches!(e, Event::Hit { .. })).count(),
            2
        );
    }

    /// 기세(0151): 스킬이 아닌 것으로 쓰러뜨리면 모든 대기가 `skill.momentum s`씩 당겨지고, 스킬로 쓰러뜨리면 그 반.
    #[test]
    fn felling_enemies_brings_skills_back_sooner_and_skill_kills_count_half() {
        let (mut w, t) = world();
        let down = Event::EnemyDown { at: Vec3::ZERO };
        w.player.skill_cd = [5.0; skills::N];
        let mut ev = vec![down, down];
        w.skill_books(&t, &mut ev);
        let want = 5.0 - 2.0 * t[P::SkillMomentum];
        assert!(w.player.skill_cd.iter().all(|c| (c - want).abs() < 1e-5));
        w.player.skill_cd = [5.0; skills::N];
        w.skill_hit_tick = true;
        w.skill_books(&t, &mut vec![down]);
        let want = 5.0 - t[P::SkillMomentumOwn];
        assert!(w.player.skill_cd.iter().all(|c| (c - want).abs() < 1e-5));
        assert!(t[P::SkillMomentumOwn] < t[P::SkillMomentum]);
        // 실제 판: 휘돌이가 한 대 몫(체력 10)짜리 둘을 쓰러뜨리면 다른 스킬의 대기가 "스킬 몫"만큼 당겨진다.
        let (mut w, t) = world();
        w.spawn_mob(vec3(1.5, 0.0, 0.0));
        w.spawn_mob(vec3(-1.5, 0.0, 0.0));
        still(&mut w);
        for e in &mut w.enemies {
            e.hp = 10;
        }
        w.player.skill_cd[Sk::Cut.index()] = 5.0;
        let ticks = 12;
        let ev = run(&mut w, &t, skill(0), ticks);
        let downs = ev
            .iter()
            .filter(|e| matches!(e, Event::EnemyDown { .. }))
            .count();
        assert_eq!(downs, 2);
        let cd = w.player.skill_cd[Sk::Cut.index()];
        assert!(cd < 5.0 - 2.0 * t[P::SkillMomentumOwn] + 1e-3, "{cd}");
        assert!(
            cd > 5.0 - 2.0 * t[P::SkillMomentum],
            "less than a plain kill"
        );
    }

    /// 게이지(0151): 맞힌 휘두르기 한 번에 한 번만 찬다(셋을 한꺼번에 베어도), 패링은 크게, 스킬의 타는 채우지 않는다.
    /// 가득 차는 틱에 한 번 알린다.
    #[test]
    fn the_gauge_fills_once_per_swing_more_per_parry_and_never_from_skills() {
        let (mut w, t) = world();
        for x in [-0.7, 0.0, 0.7] {
            w.spawn_mob(vec3(x, 0.0, -1.4));
        }
        still(&mut w);
        let ev = run(
            &mut w,
            &t,
            Actions {
                attack: true,
                ..Default::default()
            },
            40,
        );
        let hits = ev.iter().filter(|e| matches!(e, Event::Hit { .. })).count();
        assert!(hits >= 2, "a wide swing: {hits}");
        assert!((w.player.gauge - t[P::SkillGaugeHit]).abs() < 1e-5);
        // 스킬의 타는 채우지 않는다.
        let g = w.player.gauge;
        let ev = run(&mut w, &t, skill(0), 60);
        assert!(ev.iter().any(|e| matches!(e, Event::Hit { .. })));
        assert_eq!(w.player.gauge, g);
        // 패링 한 번 = 맞힘 여러 번.
        let parry = Event::Parry { at: Vec3::ZERO };
        w.skill_books(&t, &mut vec![parry]);
        assert!((w.player.gauge - g - t[P::SkillGaugeParry]).abs() < 1e-5);
        assert!(t[P::SkillGaugeParry] >= 3.0 * t[P::SkillGaugeHit]);
        // 가득: 한 번만 알린다.
        w.player.gauge = 1.0 - t[P::SkillGaugeParry] * 0.5;
        let mut ev = vec![parry];
        w.skill_books(&t, &mut ev);
        assert_eq!(w.player.gauge, 1.0);
        assert!(ev.contains(&Event::UltReady));
        let mut ev = vec![parry];
        w.skill_books(&t, &mut ev);
        assert!(!ev.contains(&Event::UltReady), "already full");
    }

    /// 두 버튼 함께 = 큰 기술(0151 3절): 같은 틱이면 바로, 한두 틱 어긋나면 먼저 나간 스킬을 시동 안에서 바꿔치고 그 대기는
    /// 돌려준다. 게이지가 안 찼으면 그냥 스킬이 나간다. 낱 누름은 늦지 않는다(누른 틱에 시동).
    #[test]
    fn both_skill_buttons_make_the_great_skill_even_a_few_ticks_apart() {
        let both = Actions {
            skill1: true,
            skill2: true,
            ..Default::default()
        };
        let (mut w, t) = world();
        w.player.gauge = 1.0;
        let ev = w.step(&both, &t);
        assert!(matches!(w.player.state, PState::Ult { .. }));
        assert!(ev.contains(&Event::UltStart));
        assert_eq!(w.player.gauge, 0.0);
        assert!(w.player.skill_cd.iter().all(|c| *c == 0.0));
        // 두 틱 어긋나게.
        let (mut w, t) = world();
        w.player.gauge = 1.0;
        w.step(&skill(0), &t);
        assert!(
            matches!(w.player.state, PState::Skill { id: Sk::Whirl, .. }),
            "a single press starts at once"
        );
        w.step(&Actions::default(), &t);
        w.step(&skill(1), &t);
        assert!(matches!(w.player.state, PState::Ult { .. }));
        assert_eq!(w.player.skill_cd[Sk::Whirl.index()], 0.0, "refunded");
        // 시동 창이 지난 뒤의 둘째 누름은 큰 기술이 아니다 — 다음 스킬로 받아 둔다.
        let (mut w, t) = world();
        w.player.gauge = 1.0;
        w.step(&skill(0), &t);
        let late = (t[P::SkillChord] / TICK_DT).ceil() as usize + 1;
        for _ in 0..late {
            w.step(&Actions::default(), &t);
        }
        w.step(&skill(1), &t);
        assert!(matches!(
            w.player.state,
            PState::Skill { id: Sk::Whirl, .. }
        ));
        assert_eq!(w.player.gauge, 1.0);
        // 게이지가 안 찼으면 함께 눌러도 스킬 하나.
        let (mut w, t) = world();
        w.step(&both, &t);
        assert!(matches!(
            w.player.state,
            PState::Skill { id: Sk::Whirl, .. }
        ));
    }

    /// 쟁반 뒤집기: 둘레의 적이 모두 바깥으로 쏟아지고 휘청인다 — 가까운 적일수록 멀리. 피해는 벽에 박힌 적만. 보스는 덜 밀리고
    /// 짧게 휘청인다. 땅이 뒤집히기까지는 끊기지 않는다.
    #[test]
    fn upend_spills_everyone_outward_and_only_walls_hurt() {
        let mut w = World::new("test", 9.0);
        let t = Tuning::default();
        w.player.facing = PI;
        w.spawn_mob(vec3(0.0, 0.0, -2.0));
        w.spawn_mob(vec3(-3.0, 0.0, 0.0));
        // 벽(아레나 끝 x = 9)에 가까운 적 — 밀려나다 박힌다.
        w.spawn_mob(vec3(6.5, 0.0, 0.0));
        w.spawn_boss(vec3(0.0, 0.0, 4.0));
        still(&mut w);
        let boss_hp = w.enemies[3].hp;
        let was: Vec<f32> = w.enemies.iter().map(|e| flat(e.pos, Vec3::ZERO)).collect();
        w.player.gauge = 1.0;
        w.step(
            &Actions {
                ult: true,
                ..Default::default()
            },
            &t,
        );
        assert!(w.player.invulnerable(), "not interrupted before it lands");
        let mut ev = Vec::new();
        let mut staggers = [0.0f32; 4];
        // 가장 멀리 밀려난 거리(휘청이 풀리면 다시 다가온다).
        let mut now = was.clone();
        for _ in 0..200 {
            ev.extend(w.step(&Actions::default(), &t));
            for (i, e) in w.enemies.iter().enumerate() {
                if let EState::Stagger { dur, .. } = e.state {
                    staggers[i] = staggers[i].max(dur);
                }
                now[i] = now[i].max(flat(e.pos, Vec3::ZERO));
            }
        }
        assert!(ev.iter().any(|e| matches!(e, Event::Ult { .. })));
        assert!(
            now[0] - was[0] > 4.0,
            "the near one flies: {}",
            now[0] - was[0]
        );
        assert!(now[1] - was[1] > 1.0 && now[1] - was[1] < now[0] - was[0]);
        assert_eq!(w.enemies[0].hp, 90, "no direct damage");
        assert_eq!(w.enemies[1].hp, 90);
        assert!(ev.iter().any(|e| matches!(e, Event::Slam { .. })));
        assert_eq!(
            w.enemies[2].hp,
            90 - crate::tuning::ULT_SLAM_DAMAGE,
            "thrown into the wall"
        );
        assert_eq!(w.enemies[3].hp, boss_hp);
        assert!(now[3] - was[3] < now[1] - was[1] + 2.0 && now[3] - was[3] > 0.2);
        assert!(
            staggers[3] > 0.0 && staggers[3] < staggers[0],
            "a short boss stagger"
        );
        assert!((staggers[0] - UPEND.stagger).abs() < 1e-5);
        // 둘레 밖의 적은 그대로.
        let mut w = World::new("test", 30.0);
        w.spawn_mob(vec3(0.0, 0.0, -(UPEND.radius + 1.5)));
        still(&mut w);
        w.player.gauge = 1.0;
        run(
            &mut w,
            &t,
            Actions {
                ult: true,
                ..Default::default()
            },
            60,
        );
        assert!(flat(w.enemies[0].pos, vec3(0.0, 0.0, -(UPEND.radius + 1.5))) < 0.05);
        assert!(!matches!(w.enemies[0].state, EState::Stagger { .. }));
    }

    /// 대기 중에 누른 스킬은 나가지 않고 칸만 깜빡인다(한 번 알림) — 몇 초 뒤에 저절로 나가지도 않는다.
    #[test]
    fn a_cooling_skill_blinks_and_is_not_buffered() {
        let (mut w, t) = world();
        run(&mut w, &t, skill(0), 60);
        let cd = w.player.skill_cd[Sk::Whirl.index()];
        assert!(cd > 1.0);
        let ev = w.step(&skill(0), &t);
        assert!(ev.contains(&Event::SkillDenied));
        assert!(matches!(w.player.state, PState::Idle));
        assert!(w.skill_ready(0, &t) < 1.0 && w.skill_ready(1, &t) == 1.0);
        let ev = run(&mut w, &t, Actions::default(), (cd / TICK_DT) as usize + 30);
        assert!(!ev.iter().any(|e| matches!(e, Event::SkillStart { .. })));
        assert_eq!(w.skill_ready(0, &t), 1.0);
        let ev = w.step(&skill(0), &t);
        assert!(ev.contains(&Event::SkillStart { id: Sk::Whirl }));
    }

    /// 스킬의 후딜은 회피로 끊는다 — 판정이 끝난 뒤부터.
    #[test]
    fn a_skill_recovers_into_a_dodge() {
        let (mut w, t) = world();
        w.set_skills([Sk::Quake, Sk::Whirl]);
        w.step(&skill(0), &t);
        let d = Sk::Quake.def();
        let dodge = Actions {
            dodge: true,
            ..Default::default()
        };
        // 시동 중의 회피는 끊지 못한다(받아 둘 뿐).
        w.step(&dodge, &t);
        assert!(matches!(w.player.state, PState::Skill { .. }));
        let to_hit = (d.startup / TICK_DT).ceil() as usize + 2;
        for _ in 0..to_hit {
            w.step(&dodge, &t);
        }
        assert!(
            matches!(w.player.state, PState::Dodge { .. }),
            "{:?}",
            w.player.state
        );
    }

    // ---- 기술 판의 단·고르기·맺음(0158 G0·G1) ----

    use crate::skills::{Loadout, Pick, Slot, MAX_RANK};

    /// 칸 0에 `id`를 `rank`단·`pick`으로 끼운다(칸 1은 남는 스킬 1단), 큰 기술은 1단.
    fn wear(w: &mut World, id: Sk, rank: u8, pick: Pick) {
        let other = if id == Sk::Whirl { Sk::Cut } else { Sk::Whirl };
        w.set_skills(Loadout {
            slots: [
                Slot { id, rank, pick },
                Slot {
                    id: other,
                    rank: 1,
                    pick: Pick::A,
                },
            ],
            ult: 1,
        });
    }

    /// 끼운 칸의 단·고르기가 월드에 들어간다 — 같은 스킬 둘이라 바꿔 끼운 칸은 1단. 준비 막대는 그 단의 대기로 잰다.
    #[test]
    fn worn_ranks_reach_the_world_and_the_cooldown_bar_uses_the_ranked_cooldown() {
        let (mut w, t) = world();
        w.set_skills(Loadout {
            slots: [
                Slot {
                    id: Sk::Quake,
                    rank: 5,
                    pick: Pick::B,
                },
                Slot {
                    id: Sk::Quake,
                    rank: 3,
                    pick: Pick::B,
                },
            ],
            ult: 3,
        });
        assert_eq!(w.player.skills, [Sk::Quake, Sk::Whirl]);
        assert_eq!(w.player.skill_rank, [5, 1]);
        assert_eq!(w.player.skill_pick, [Pick::B, Pick::A]);
        assert_eq!(w.player.ult_rank, 3);
        assert_eq!(w.skill_eff(Sk::Quake).skill.damage, 26);
        assert_eq!(w.skill_eff(Sk::Whirl).skill.damage, 10);
        assert_eq!(w.skill_eff(Sk::Gather).twist, None, "not worn: rank 1");
        // 5단의 대기는 ×0.9 — 쓴 직후 남은 대기가 그만큼.
        w.step(&skill(0), &t);
        let cd = w.player.skill_cd[Sk::Quake.index()];
        assert!((cd - Sk::Quake.def().cooldown * 0.9).abs() < 0.05, "{cd}");
        assert!(w.skill_ready(0, &t) < 0.02);
    }

    /// 2단: 피해 +15%(10 → 12). 4단: 반지름 +20% — 1단에서는 안 닿던 적이 닿는다.
    #[test]
    fn rank_two_hits_harder_and_rank_four_reaches_further() {
        let hp_after = |rank: u8| {
            let (mut w, t) = world();
            wear(&mut w, Sk::Whirl, rank, Pick::A);
            w.spawn_mob(vec3(1.5, 0.0, 0.0));
            // 3.0m 반지름 밖·3.6m 안(몸 반지름을 빼고).
            w.spawn_mob(vec3(
                0.0,
                0.0,
                3.3 + Enemy::new(Kind::Husk, Vec3::ZERO).radius,
            ));
            still(&mut w);
            run(&mut w, &t, skill(0), 60);
            (w.enemies[0].hp, w.enemies[1].hp)
        };
        assert_eq!(hp_after(1), (90 - 2 * 10, 90));
        assert_eq!(hp_after(2), (90 - 2 * 12, 90));
        assert_eq!(hp_after(4), (90 - 2 * 12, 90 - 2 * 12), "the far one is in");
    }

    /// 휘돌이 3단 ㄱ "도는 걸음": 도는 동안 스틱 쪽으로 걷는다(걷는 빠르기보다 느리게) — 고르기가 없으면 제자리.
    #[test]
    fn whirl_pick_a_walks_while_spinning() {
        let moved = |rank: u8, pick: Pick| {
            let (mut w, t) = world();
            wear(&mut w, Sk::Whirl, rank, pick);
            // 제자리에서는 반 걸음 모자란 자리(반지름 3m + 몸 + 0.5m).
            let r = Enemy::new(Kind::Husk, Vec3::ZERO).radius;
            w.spawn_mob(vec3(0.0, 0.0, -(3.5 + r)));
            still(&mut w);
            let go = Actions {
                move_dir: vec2(0.0, 1.0),
                ..Default::default()
            };
            w.step(&Actions { skill1: true, ..go }, &t);
            let mut ticks = 1;
            while matches!(w.player.state, PState::Skill { .. }) {
                w.step(&go, &t);
                ticks += 1;
            }
            (flat(w.player.pos, Vec3::ZERO), ticks as f32 * TICK_DT, w, t)
        };
        let (d, secs, w, t) = moved(3, Pick::A);
        assert!(d > 1.0, "walked while spinning: {d}");
        assert!(
            d < t[P::MoveSpeed] * secs * 0.75,
            "but slower than a walk: {d}"
        );
        assert!(w.player.pos.z < -1.0, "towards the stick (screen up)");
        // 걸어 들어가서 닿았다: 반 걸음 밖의 적은 제자리에서는 안 닿는다.
        assert!(w.enemies[0].hp < 90, "the walk carried the ring to it");
        let (d, _, w, _) = moved(2, Pick::A);
        assert!(d < 0.05, "no pick yet: stands still ({d})");
        assert_eq!(w.enemies[0].hp, 90);
        let (d, ..) = moved(3, Pick::B);
        assert!(d < 0.05, "the other pick stands still ({d})");
    }

    /// 휘돌이 3단 ㄴ "셋째 바퀴": 세 번 돈다 — 둘레의 적이 세 번 맞고, 미는 것은 여전히 마지막 바퀴.
    #[test]
    fn whirl_pick_b_spins_a_third_time() {
        let (mut w, t) = world();
        wear(&mut w, Sk::Whirl, 3, Pick::B);
        w.spawn_mob(vec3(2.0, 0.0, 0.0));
        w.spawn_mob(vec3(-2.0, 0.0, 0.0));
        still(&mut w);
        let ev = run(&mut w, &t, skill(0), 80);
        let spins = ev
            .iter()
            .filter(|e| matches!(e, Event::Skill { id: Sk::Whirl, .. }))
            .count();
        assert_eq!(spins, 3);
        let dmg = w.skill_eff(Sk::Whirl).skill.damage;
        assert_eq!(dmg, 12);
        for e in &w.enemies {
            assert_eq!(e.hp, 90 - 3 * dmg, "once per spin");
            assert!(flat(e.pos, Vec3::ZERO) > 2.8, "pushed out by the last spin");
        }
        assert!(matches!(w.player.state, PState::Idle));
        assert!(w.player.facing_dir().dot(vec3(0.0, 0.0, -1.0)) > 0.99);
    }

    /// 가로지르기 3단 ㄱ "되돌아 베기": 끝에서 온 길을 되돌아와 처음 자리 곁에 서고, 길 위의 적은 두 번 베인다.
    #[test]
    fn cut_pick_a_dashes_back_and_cuts_again() {
        let (mut w, t) = world();
        wear(&mut w, Sk::Cut, 3, Pick::A);
        for z in [-1.6, -3.0, -4.4] {
            w.spawn_mob(vec3(0.0, 0.0, z));
        }
        w.spawn_mob(vec3(4.0, 0.0, -3.0));
        still(&mut w);
        let mut ev = w.step(&skill(0), &t);
        let mut far: f32 = 0.0;
        // 맞힐 때마다 히트스톱이 끼니 끝날 때까지 돈다.
        for _ in 0..400 {
            ev.extend(w.step(&Actions::default(), &t));
            far = far.max(-w.player.pos.z);
            if matches!(w.player.state, PState::Idle) {
                break;
            }
        }
        assert!(far > 5.0, "went the whole lane first: {far}");
        assert!(
            flat(w.player.pos, Vec3::ZERO) < 0.9,
            "back at the start: {:?}",
            w.player.pos
        );
        let dmg = w.skill_eff(Sk::Cut).skill.damage + w.stats.dmg;
        for e in &w.enemies[..3] {
            assert_eq!(e.hp, 90 - 2 * dmg, "cut going and coming");
        }
        assert_eq!(w.enemies[3].hp, 90, "off the lane");
        let flashes: Vec<Vec3> = ev
            .iter()
            .filter_map(|e| match e {
                Event::Skill {
                    id: Sk::Cut, dir, ..
                } => Some(*dir),
                _ => None,
            })
            .collect();
        assert_eq!(flashes.len(), 2);
        assert!(
            flashes[0].dot(flashes[1]) < -0.99,
            "the second one faces back"
        );
        assert!(matches!(w.player.state, PState::Idle));
        // 고르기가 없으면 한 번만, 반대편에 선다.
        let (mut w, t) = world();
        wear(&mut w, Sk::Cut, 2, Pick::A);
        w.spawn_mob(vec3(0.0, 0.0, -3.0));
        still(&mut w);
        run(&mut w, &t, skill(0), 120);
        assert!(w.player.pos.z < -5.0);
        assert_eq!(w.enemies[0].hp, 90 - 23);
    }

    /// 가로지르기 3단 ㄴ "스치는 몸": 지나는 동안에만 맞지 않는다 — 시동과 후딜은 맞는다. ㄱ을 골랐으면 내내 맞는다.
    #[test]
    fn cut_pick_b_cannot_be_hit_while_dashing() {
        let safe_ticks = |pick: Pick| {
            let (mut w, t) = world();
            wear(&mut w, Sk::Cut, 3, pick);
            w.step(&skill(0), &t);
            let mut log = vec![w.player.invulnerable()];
            while matches!(w.player.state, PState::Skill { .. }) {
                w.step(&Actions::default(), &t);
                log.push(w.player.invulnerable());
            }
            log
        };
        let b = safe_ticks(Pick::B);
        let d = Sk::Cut.def();
        let dash = (d.active() / TICK_DT).round() as usize;
        let safe = b.iter().filter(|x| **x).count();
        assert!(
            safe + 2 >= dash && safe <= dash + 2,
            "safe for the dash: {safe} of {dash}"
        );
        assert!(!b[0], "the startup can be hit");
        assert!(!b[b.len() - 1] && !b[b.len() - 3], "so can the recovery");
        assert!(safe_ticks(Pick::A).iter().all(|x| !x), "the other pick");
        // 실제로 맞지 않는다: 지나는 길에서 내리치는 적 — ㄱ이면 같은 자리에서 맞는다.
        let hurt = |pick: Pick| {
            let (mut w, t) = world();
            wear(&mut w, Sk::Cut, 3, pick);
            let into_dash = (d.startup / TICK_DT).ceil() as usize + 3;
            run(&mut w, &t, skill(0), into_dash);
            assert!(matches!(w.player.state, PState::Skill { .. }));
            let hp = w.player.hp;
            w.spawn_mob(w.player.pos + vec3(0.9, 0.0, -1.0));
            w.enemies[0].state = EState::Swing { t: 0.0 };
            let ev = w.step(&Actions::default(), &t);
            (hp - w.player.hp, ev.contains(&Event::PlayerHurt))
        };
        assert_eq!(hurt(Pick::B), (0, false), "slips through the blow");
        let (lost, ev) = hurt(Pick::A);
        assert!(lost > 0 && ev, "the same blow lands without the pick");
    }

    /// 끌어모으기 3단 ㄱ "모인 자리": 0.5초 뒤 모인 점이 터져 그 둘레의 적이 한 눈금(10) 다친다 — 멀리 있는 적은 그대로.
    /// 터짐도 스킬의 타라 게이지를 채우지 않는다. 고르기가 없으면 터지지 않는다.
    #[test]
    fn gather_pick_a_bursts_where_they_gathered() {
        let (mut w, t) = world();
        wear(&mut w, Sk::Gather, 3, Pick::A);
        for p in [(-2.5, -4.5), (2.5, -4.5), (0.0, -6.0)] {
            w.spawn_mob(vec3(p.0, 0.0, p.1));
        }
        w.spawn_mob(vec3(0.0, 0.0, 9.0));
        still(&mut w);
        let d = Sk::Gather.def();
        let pull = (d.startup / TICK_DT).ceil() as usize + 1;
        let ev = run(&mut w, &t, skill(0), pull);
        let spot = ev
            .iter()
            .find_map(|e| match e {
                Event::Skill {
                    id: Sk::Gather, at, ..
                } => Some(*at),
                _ => None,
            })
            .expect("it pulls");
        assert!(
            w.enemies.iter().all(|e| e.hp == 90),
            "the pull does no harm"
        );
        // 0.5초가 다 되기 전에는 아직.
        let ev = run(&mut w, &t, Actions::default(), 24);
        assert!(!ev.iter().any(|e| matches!(e, Event::SkillBurst { .. })));
        let ev = run(&mut w, &t, Actions::default(), 20);
        let at = ev
            .iter()
            .find_map(|e| match e {
                Event::SkillBurst { at } => Some(*at),
                _ => None,
            })
            .expect("it bursts");
        assert!(flat(at, spot) < 1e-3, "at the gather point");
        for e in &w.enemies[..3] {
            assert_eq!(e.hp, 90 - crate::tuning::HP_UNIT);
        }
        assert_eq!(w.enemies[3].hp, 90, "far behind");
        assert_eq!(w.player.gauge, 0.0, "a skill hit: no gauge");
        let ev = run(&mut w, &t, Actions::default(), 120);
        assert!(
            !ev.iter().any(|e| matches!(e, Event::SkillBurst { .. })),
            "once"
        );
        // 2단(고르기 없음)·3단 ㄴ은 터지지 않는다.
        for (rank, pick) in [(2, Pick::A), (3, Pick::B)] {
            let (mut w, t) = world();
            wear(&mut w, Sk::Gather, rank, pick);
            w.spawn_mob(vec3(0.0, 0.0, -5.0));
            still(&mut w);
            let ev = run(&mut w, &t, skill(0), 120);
            assert!(!ev.iter().any(|e| matches!(e, Event::SkillBurst { .. })));
            assert_eq!(w.enemies[0].hp, 90);
        }
    }

    /// 끌어모으기 3단 ㄴ "무거운 것도": 엘리트도 끌려온다 — 보스는 여전히 끌리지 않는다. ㄱ이면 엘리트는 그대로.
    #[test]
    fn gather_pick_b_pulls_elites_too_but_never_a_boss() {
        let pulled = |pick: Pick| {
            let (mut w, t) = world();
            wear(&mut w, Sk::Gather, 3, pick);
            // 가장 가까운 잡몹 쪽(곧장 앞)을 겨눈다 — 보스와 엘리트는 그 부채꼴 안, 끌려오는 길 밖.
            w.spawn_mob(vec3(0.0, 0.0, -3.0));
            w.spawn_boss(vec3(3.0, 0.0, -5.0));
            let elite = w.spawn(crate::species::Kind::Husk, vec3(-3.0, 0.0, -5.0));
            w.enemies[elite].make_elite(Affix::Repeating);
            still(&mut w);
            let was: Vec<Vec3> = w.enemies.iter().map(|e| e.pos).collect();
            let mut staggered = [false; 3];
            w.step(&skill(0), &t);
            for _ in 0..70 {
                w.step(&Actions::default(), &t);
                for (i, e) in w.enemies.iter().enumerate() {
                    staggered[i] |= matches!(e.state, EState::Stagger { .. });
                }
            }
            let moved: Vec<f32> = w
                .enemies
                .iter()
                .zip(&was)
                .map(|(e, p)| flat(e.pos, *p))
                .collect();
            (moved, staggered)
        };
        let (moved, staggered) = pulled(Pick::B);
        assert!(moved[0] > 0.5 && staggered[0], "the mob comes: {moved:?}");
        assert!(
            moved[2] > 2.0 && staggered[2],
            "the elite comes too: {moved:?}"
        );
        assert!(moved[1] < 0.05 && !staggered[1], "the boss never");
        let (moved, staggered) = pulled(Pick::A);
        assert!(moved[0] > 0.5);
        assert!(
            moved[2] < 0.05 && !staggered[2],
            "without the pick the elite stays"
        );
        assert!(moved[1] < 0.05 && !staggered[1]);
    }

    /// 땅울림 3단 ㄱ "멀리 뛰기": 겨눈 쪽으로 6m까지 뛰어들어 찍는다 — 그 길의 가장 가까운 적 1.5m쯤 앞에서 멈추고(먼 적에게
    /// 닿는다), 바로 앞에 적이 있으면 거의 뛰지 않는다. 길에 적이 없으면 6m를 다 뛴다. 고르기가 없으면 제자리.
    #[test]
    fn quake_pick_a_leaps_in_and_stops_short_of_the_nearest_enemy() {
        let leap = |rank: u8, pick: Pick, enemy_z: Option<f32>| {
            let (mut w, t) = world();
            wear(&mut w, Sk::Quake, rank, pick);
            if let Some(z) = enemy_z {
                w.spawn_mob(vec3(0.0, 0.0, z));
            }
            still(&mut w);
            let d = Sk::Quake.def();
            let land = (d.startup / TICK_DT).ceil() as usize + 2;
            let ev = run(&mut w, &t, skill(0), land);
            let hit = ev.iter().any(|e| matches!(e, Event::Hit { .. }));
            (-w.player.pos.z, w.enemies.first().map(|e| e.pos), hit)
        };
        // 9m 앞의 적: 1단의 부채꼴(5m)은 못 닿는다 — 뛰어들면 닿는다.
        let (went, en, hit) = leap(3, Pick::A, Some(-9.0));
        let gap = flat(vec3(0.0, 0.0, -went), en.unwrap());
        assert!(went > 4.5 && went <= 6.05, "a long leap: {went}");
        assert!(hit, "and the slam reaches it");
        assert!(gap > 1.2 && gap < 4.0, "lands short of it: {gap}");
        // 7m 앞: 6m를 다 뛰지 않고 적 앞에서 멈춘다.
        let (went, en, hit) = leap(3, Pick::A, Some(-7.0));
        let r = Enemy::new(Kind::Husk, Vec3::ZERO).radius;
        let want = 7.0 - r - 1.5;
        assert!(
            (went - want).abs() < 0.6,
            "stops 1.5 m short: {went} vs {want}"
        );
        assert!(hit && flat(vec3(0.0, 0.0, -went), en.unwrap()) > 1.3);
        // 바로 앞(2m): 큰 뜀이 없다.
        let (went, _, hit) = leap(3, Pick::A, Some(-2.0));
        assert!(went < 0.4, "a close target: no big leap ({went})");
        assert!(hit);
        // 길에 아무도 없으면 끝까지.
        let (went, ..) = leap(3, Pick::A, None);
        assert!(went > 5.2 && went <= 6.05, "{went}");
        // 고르기가 없거나 ㄴ이면 제자리에서 찍는다 — 9m의 적은 못 닿는다.
        for (rank, pick) in [(2, Pick::A), (3, Pick::B)] {
            let (went, _, hit) = leap(rank, pick, Some(-9.0));
            assert!(went.abs() < 0.05 && !hit, "{rank} {pick:?}: {went}");
        }
    }

    /// 땅울림 3단 ㄴ "갈라진 땅": 찍은 부채꼴이 2초 남는다 — 그 안의 적은 반의 빠르기로 걷고(밖의 적·보스는 그대로), 2초 뒤 사라진다.
    #[test]
    fn quake_pick_b_leaves_cracked_ground_that_slows() {
        let (mut w, t) = world();
        wear(&mut w, Sk::Quake, 3, Pick::B);
        w.spawn_mob(vec3(0.0, 0.0, -4.0));
        w.spawn_mob(vec3(0.0, 0.0, 4.0));
        w.spawn_boss(vec3(2.0, 0.0, -3.0));
        still(&mut w);
        let d = Sk::Quake.def();
        let land = (d.startup / TICK_DT).ceil() as usize + 1;
        run(&mut w, &t, skill(0), land);
        assert_eq!(w.skill_zones.len(), 1, "the slam leaves a zone");
        let z = w.skill_zones[0];
        assert!(z.holds(vec3(0.0, 0.0, -4.0), 0.0) && !z.holds(vec3(0.0, 0.0, 4.0), 0.0));
        w.step(&Actions::default(), &t);
        assert_eq!(w.enemies[0].slow, 0.5, "inside");
        assert_eq!(w.enemies[1].slow, 1.0, "behind: outside the cone");
        assert_eq!(w.enemies[2].slow, 1.0, "a boss is not slowed");
        // 실제 걸음: 안의 적과 밖의 적을 같은 거리에서 쫓게 하면 안쪽이 반만 간다.
        for e in &mut w.enemies[..2] {
            e.state = EState::Chase;
            e.knock = Vec3::ZERO;
        }
        let was = [w.enemies[0].pos, w.enemies[1].pos];
        for _ in 0..10 {
            w.step(&Actions::default(), &t);
        }
        let (slow, free) = (
            flat(w.enemies[0].pos, was[0]),
            flat(w.enemies[1].pos, was[1]),
        );
        assert!(free > 0.1, "the free one walks: {free}");
        assert!(
            (slow / free - 0.5).abs() < 0.12,
            "half speed on the cracked ground: {slow} vs {free}"
        );
        // 2초 뒤: 땅이 아물고 빠르기가 돌아온다.
        still(&mut w);
        run(&mut w, &t, Actions::default(), 125);
        assert!(w.skill_zones.is_empty());
        assert!(w.enemies.iter().all(|e| e.slow == 1.0));
        // 고르기가 없으면 남는 땅이 없다.
        let (mut w, t) = world();
        wear(&mut w, Sk::Quake, 3, Pick::A);
        w.spawn_mob(vec3(0.0, 0.0, -3.0));
        still(&mut w);
        run(&mut w, &t, skill(0), 40);
        assert!(w.skill_zones.is_empty());
    }

    /// 휘돌이 5단 맺음: 마지막 바퀴에 맞은 적이 0.4초 휘청인다 — 보스는 휘청이지 않는다. 4단까지는 휘청이 없다.
    #[test]
    fn whirl_capstone_staggers_on_the_last_spin_only() {
        let staggers = |rank: u8| {
            let (mut w, t) = world();
            wear(&mut w, Sk::Whirl, rank, Pick::A);
            w.spawn_mob(vec3(2.0, 0.0, 0.0));
            w.spawn_boss(vec3(-2.0, 0.0, 0.0));
            still(&mut w);
            let d = w.skill_eff(Sk::Whirl);
            w.step(&skill(0), &t);
            let mut first: Option<(f32, f32)> = None;
            let mut boss = false;
            let mut secs = TICK_DT;
            while matches!(w.player.state, PState::Skill { .. }) {
                w.step(&Actions::default(), &t);
                secs += TICK_DT;
                if let (EState::Stagger { dur, .. }, None) = (w.enemies[0].state, first) {
                    first = Some((secs, dur));
                }
                boss |= matches!(w.enemies[1].state, EState::Stagger { .. });
            }
            (first, boss, d)
        };
        let (first, boss, d) = staggers(MAX_RANK);
        let (at, dur) = first.expect("the mob staggers");
        assert!((dur - 0.4).abs() < 1e-6);
        assert!(
            at > d.skill.startup + d.active() - 0.05,
            "on the last spin, not the first: {at}"
        );
        assert!(!boss, "a boss does not stagger");
        let (first, boss, _) = staggers(4);
        assert!(first.is_none() && !boss, "no capstone below rank 5");
    }

    /// 가로지르기 5단 맺음: 이 스킬로 쓰러뜨리면 대기의 반을 돌려받는다 — 둘을 쓰러뜨려도 한 번. 못 쓰러뜨리면 그대로.
    #[test]
    fn cut_capstone_refunds_half_the_cooldown_on_a_kill_once() {
        let cd_after = |kills: usize, rank: u8| {
            let (mut w, t) = world();
            wear(&mut w, Sk::Cut, rank, Pick::B);
            for z in [-2.0, -4.0] {
                w.spawn_mob(vec3(0.0, 0.0, z));
            }
            still(&mut w);
            for e in w.enemies.iter_mut().take(kills) {
                e.hp = 10;
            }
            let ticks = 40;
            let ev = run(&mut w, &t, skill(0), ticks);
            let downs = ev
                .iter()
                .filter(|e| matches!(e, Event::EnemyDown { .. }))
                .count();
            assert_eq!(downs, kills);
            (w.player.skill_cd[Sk::Cut.index()], w, t)
        };
        let (none, w, t) = cd_after(0, MAX_RANK);
        let full = w.skill_eff(Sk::Cut).skill.cooldown * t[P::SkillCooldownX];
        assert!((full - Sk::Cut.def().cooldown * 0.9).abs() < 1e-4);
        let own = t[P::SkillMomentumOwn];
        let (one, ..) = cd_after(1, MAX_RANK);
        assert!(
            (none - one - (full * 0.5 + own)).abs() < 0.05,
            "half back (+ the usual momentum): {none} -> {one}"
        );
        let (two, ..) = cd_after(2, MAX_RANK);
        assert!(
            (one - two - own).abs() < 0.05,
            "the second kill only adds momentum: {one} -> {two}"
        );
        // 4단: 쓰러뜨려도 기세만.
        let (none4, ..) = cd_after(0, 4);
        let (one4, ..) = cd_after(1, 4);
        assert!((none4 - one4 - own).abs() < 0.05, "{none4} -> {one4}");
    }

    /// 끌어모으기 5단 맺음: 끌려온 적은 2초 동안 피해를 20% 더 받는다(10 → 12) — 끌려오지 않은 적은 그대로, 2초 뒤에는 풀린다.
    #[test]
    fn gather_capstone_makes_the_gathered_take_more_for_two_seconds() {
        let (mut w, t) = world();
        w.set_skills(Loadout {
            slots: [
                Slot {
                    id: Sk::Gather,
                    rank: MAX_RANK,
                    pick: Pick::B,
                },
                Slot {
                    id: Sk::Whirl,
                    rank: 1,
                    pick: Pick::A,
                },
            ],
            ult: 1,
        });
        // 앞의 적이 더 가깝다(겨누는 쪽) — 뒤의 적은 부채꼴 밖.
        w.spawn_mob(vec3(0.0, 0.0, -2.2));
        w.spawn_mob(vec3(0.0, 0.0, 2.6));
        still(&mut w);
        run(&mut w, &t, skill(0), 40);
        assert!(w.enemies[0].frail > 1.0 && w.enemies[0].frail <= 2.0);
        assert_eq!(w.enemies[1].frail, 0.0, "behind: not gathered");
        assert_eq!(w.enemies[0].hp, 90);
        // 모인 적 곁에서 휘돌이(1단, 한 타 10): 모인 적은 12씩, 뒤의 적은 10씩.
        run(&mut w, &t, skill(1), 60);
        assert_eq!(w.enemies[0].hp, 90 - 2 * 12, "{:?}", w.enemies[0].pos);
        assert_eq!(w.enemies[1].hp, 90 - 2 * 10);
        // 2초가 지나면 풀린다.
        still(&mut w);
        run(&mut w, &t, Actions::default(), 90);
        assert_eq!(w.enemies[0].frail, 0.0);
        assert_eq!(hurt_by(&w.enemies[0], 10, false, 0, &t), 10);
        // 4단에는 없다.
        let (mut w, t) = world();
        wear(&mut w, Sk::Gather, 4, Pick::A);
        w.spawn_mob(vec3(0.0, 0.0, -4.0));
        still(&mut w);
        run(&mut w, &t, skill(0), 20);
        assert_eq!(w.enemies[0].frail, 0.0);
    }

    /// 땅울림 5단 맺음: 자세가 깨져 있는 적에게 두 배 — 멀쩡한 적은 그대로(26). 4단에는 없다.
    #[test]
    fn quake_capstone_doubles_against_broken_poise() {
        let hits = |rank: u8| {
            let (mut w, t) = world();
            wear(&mut w, Sk::Quake, rank, Pick::B);
            w.spawn_mob(vec3(-1.0, 0.0, -3.0));
            w.spawn_mob(vec3(1.0, 0.0, -3.0));
            still(&mut w);
            for e in &mut w.enemies {
                e.hp = 500;
                e.max_hp = 500;
            }
            w.enemies[0].broken = 5.0;
            run(&mut w, &t, skill(0), 30);
            (500 - w.enemies[0].hp, 500 - w.enemies[1].hp, t)
        };
        let (broken, whole, t) = hits(MAX_RANK);
        assert_eq!(whole, 26);
        // 깨진 동안의 ×1.5(옛 눈금 올림)에 맺음의 두 배.
        let opened = crate::tuning::scale_up(26, t[P::PoiseBreakDamage]);
        assert_eq!(broken, 2 * opened, "double on top of the open-poise bonus");
        let (broken, whole, t) = hits(4);
        assert_eq!(whole, 23);
        assert_eq!(broken, crate::tuning::scale_up(23, t[P::PoiseBreakDamage]));
    }

    /// 큰 기술의 단: 2단 = 휘청 +0.6초(보스의 짧은 휘청은 그대로), 3단 = 벽에 박힌 피해 ×1.5(20 → 30). 직접 피해는 여전히 없다.
    #[test]
    fn upend_ranks_lengthen_the_stagger_and_harden_the_wall() {
        let cast = |rank: u8| {
            let mut w = World::new("test", 9.0);
            let t = Tuning::default();
            w.player.facing = PI;
            w.set_skills(Loadout {
                ult: rank,
                ..Loadout::from(skills::START)
            });
            w.spawn_mob(vec3(0.0, 0.0, -2.0));
            w.spawn_mob(vec3(6.5, 0.0, 0.0));
            w.spawn_boss(vec3(0.0, 0.0, 4.0));
            still(&mut w);
            let boss_hp = w.enemies[2].hp;
            w.player.gauge = 1.0;
            w.step(
                &Actions {
                    ult: true,
                    ..Default::default()
                },
                &t,
            );
            let mut durs = [0.0f32; 3];
            for _ in 0..200 {
                w.step(&Actions::default(), &t);
                for (i, e) in w.enemies.iter().enumerate() {
                    if let (EState::Stagger { dur, .. }, true) = (e.state, durs[i] == 0.0) {
                        durs[i] = dur;
                    }
                }
            }
            assert_eq!(w.enemies[0].hp, 90, "no direct damage at rank {rank}");
            assert_eq!(w.enemies[2].hp, boss_hp);
            (durs, 90 - w.enemies[1].hp)
        };
        let (d1, wall1) = cast(1);
        let (d2, wall2) = cast(2);
        let (d3, wall3) = cast(3);
        assert!((d1[0] - UPEND.stagger).abs() < 1e-5);
        assert!((d2[0] - UPEND.stagger - 0.6).abs() < 1e-5, "{d2:?}");
        assert!((d3[0] - d2[0]).abs() < 1e-5);
        for d in [d1, d2, d3] {
            assert!(
                (d[2] - UPEND.boss_stagger).abs() < 1e-5,
                "a boss's short stagger does not grow: {d:?}"
            );
        }
        assert_eq!((wall1, wall2, wall3), (20, 20, 30));
    }

    /// 어느 단·어느 고르기에서도(0151의 규칙): 보스는 끌리지 않고·스킬로 휘청이지 않고·느려지지 않고·더 다치는 표지도 붙지 않는다.
    /// 스킬의 타(되돌아 베기·셋째 바퀴·모인 자리의 터짐까지)는 큰 기술 게이지를 채우지 않고, 스킬은 기력을 쓰지 않는다.
    #[test]
    fn at_max_rank_with_every_pick_a_boss_is_never_pulled_or_staggered_and_skills_stay_free() {
        for id in skills::IDS {
            for pick in [Pick::A, Pick::B] {
                let (mut w, t) = world();
                wear(&mut w, id, MAX_RANK, pick);
                // 보스는 어느 모양에도 닿는 자리(앞 2.5m), 잡몹 하나는 그 곁. 끌어모으기는 보스가 부채꼴 안·끌려오는 길 밖에
                // 서게(끌려온 잡몹의 몸에 밀리는 것은 끌림이 아니다).
                if id == Sk::Gather {
                    w.spawn_boss(vec3(3.0, 0.0, -5.0));
                    w.spawn_mob(vec3(0.0, 0.0, -3.0));
                } else {
                    w.spawn_boss(vec3(0.6, 0.0, -2.5));
                    w.spawn_mob(vec3(-1.2, 0.0, -2.5));
                }
                still(&mut w);
                let (boss_was, boss_hp) = (w.enemies[0].pos, w.enemies[0].hp);
                let stamina = w.player.stamina;
                let tag = format!("{id:?} {pick:?}");
                let mut ev = w.step(&skill(0), &t);
                for _ in 0..150 {
                    ev.extend(w.step(&Actions::default(), &t));
                    let b = &w.enemies[0];
                    assert!(
                        !matches!(b.state, EState::Stagger { .. }),
                        "{tag}: staggered"
                    );
                    assert_eq!(b.slow, 1.0, "{tag}: slowed");
                    assert_eq!(b.frail, 0.0, "{tag}: marked");
                }
                assert!(
                    ev.iter().any(|e| matches!(e, Event::Skill { .. })),
                    "{tag}: it went off"
                );
                // 끌려오지 않았다: 밀려난 것(넉백)은 있어도 방랑자 쪽으로 오지는 않는다.
                let (was, now) = (
                    flat(boss_was, Vec3::ZERO),
                    flat(w.enemies[0].pos, w.player.pos),
                );
                if id == Sk::Gather {
                    assert!(flat(w.enemies[0].pos, boss_was) < 0.05, "{tag}: pulled");
                    assert_eq!(w.enemies[0].hp, boss_hp, "{tag}: gather alone does no harm");
                } else if !(id == Sk::Quake && pick == Pick::A) {
                    assert!(now > was - 0.6, "{tag}: drawn in ({was} -> {now})");
                }
                assert_eq!(w.player.gauge, 0.0, "{tag}: skill hits fill no gauge");
                assert_eq!(w.player.stamina, stamina, "{tag}: no stamina");
                assert!(w.player.skill_cd[id.index()] > 1.0, "{tag}: cooling down");
            }
        }
    }
}
