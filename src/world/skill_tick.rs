//! 스킬 틱(0151, 표는 `skills.rs`): 누름 → 받아 두기(`skill_press`), 시작(`try_skill`), 상태 틱(`tick_skill`·`tick_ult`),
//! 판정(`skill_area`·`skill_pull`·`upend`), 틱 끝의 셈(`skill_books` — 기세·게이지). `tick_player`의 한 갈래다.
use super::player_tick::Ctx;
use super::*;
use crate::skills::{Form, Skill, UPEND};
use crate::tuning::{SKILL_DENIED_SHOW, SKILL_HITSTOP, ULT_HITSTOP, ULT_KILL_WINDOW};

impl World {
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
        let d = id.def();
        let dir = self.skill_dir(cx, d);
        let pl = &mut self.player;
        pl.skill_cd[id.index()] = d.cooldown * cx.t[P::SkillCooldownX];
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
    /// 없으면 바라보는 쪽.
    fn skill_dir(&self, cx: &Ctx, d: &Skill) -> Vec3 {
        let pl = &self.player;
        let facing = pl.facing_dir();
        let lock = cx
            .target
            .map(|tg| vec3(tg.x - pl.pos.x, 0.0, tg.z - pl.pos.z).normalize_or_zero())
            .filter(|v| v.length_squared() > 0.5);
        match d.form {
            Form::Ring { .. } => facing,
            Form::Lane { .. } => {
                if cx.world_move.length_squared() > 1e-4 {
                    cx.world_move.normalize()
                } else {
                    cx.aim.or(lock).unwrap_or(facing)
                }
            }
            Form::Pull { range, .. } | Form::Cone { range, .. } => cx
                .aim
                .or(lock)
                .or_else(|| nearest_in_reach(&self.enemies, pl.pos, range, None))
                .unwrap_or(facing),
        }
    }

    pub(super) fn tick_skill(&mut self, cx: &Ctx, id: skills::Id, e: f32, dir: Vec3, swing: u32) {
        let (t, dt) = (cx.t, cx.dt);
        let d = id.def();
        // 두 버튼 함께(0151): 시동 안에 들어온 큰 기술 누름이 이 스킬을 바꿔친다 — 쓴 대기는 돌려준다.
        if cx.a.ult && e < t[P::SkillChord] && self.player.gauge >= 1.0 {
            self.player.skill_cd[id.index()] = 0.0;
            self.start_ult();
            return;
        }
        let e1 = e + dt;
        let crossed = |at: f32| e < at && e1 >= at;
        let at = self.player.pos;
        let mut swing = swing;
        match d.form {
            Form::Ring {
                radius,
                hits,
                gap,
                push,
            } => {
                // 몸이 판정과 함께 돈다(바퀴 수 = 타 수) — 끝나면 처음 보던 쪽.
                let span = (gap * hits as f32).max(1e-3);
                let turns = ((e1 - d.startup) / span).clamp(0.0, 1.0) * hits as f32;
                self.player.facing = dir.x.atan2(dir.z) + turns * std::f32::consts::TAU;
                for k in 0..hits {
                    if !crossed(d.startup + gap * k as f32) {
                        continue;
                    }
                    // 바퀴마다 새 타 번호 — 같은 적을 바퀴마다 한 번씩 친다.
                    if k > 0 {
                        self.player.swings += 1;
                        swing = self.player.swings;
                    }
                    self.skill_ev.push(Event::Skill { id, at, dir });
                    // 바깥으로 미는 것은 마지막 바퀴만 — 첫 바퀴가 밀어내면 둘째 바퀴가 허공을 벤다.
                    let out = if k + 1 == hits { push } else { 0.0 };
                    self.skill_area(t, d, radius, None, out, 0.0, swing);
                }
            }
            Form::Lane { half, .. } => {
                if crossed(d.startup) {
                    self.skill_ev.push(Event::Skill { id, at, dir });
                }
                // 지나가는 동안 틱마다 몸 둘레를 친다(한 번 맞은 적은 다시 안 맞는다).
                if e1 >= d.startup && e < d.startup + d.active() {
                    self.skill_area(t, d, half, None, 0.0, 0.0, swing);
                }
            }
            Form::Pull {
                range,
                arc_deg,
                to,
                stagger,
            } => {
                if crossed(d.startup) {
                    self.skill_pull(id, dir, range, arc_deg, to, stagger);
                }
            }
            Form::Cone {
                range,
                arc_deg,
                stagger,
            } => {
                if crossed(d.startup) {
                    self.skill_ev.push(Event::Skill { id, at, dir });
                    let half = (arc_deg * 0.5).to_radians().cos();
                    self.skill_area(t, d, range, Some((dir, half)), 0.0, stagger, swing);
                }
            }
        }
        self.player.state = if e1 >= d.total() {
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
    #[allow(clippy::too_many_arguments)]
    fn skill_area(
        &mut self,
        t: &Tuning,
        d: &Skill,
        reach: f32,
        cone: Option<(Vec3, f32)>,
        push: f32,
        stagger: f32,
        swing: u32,
    ) -> u32 {
        let mut ev = std::mem::take(&mut self.skill_ev);
        let p = self.player.pos;
        let body = p.y + self.player.air;
        let facing = self.player.facing_dir();
        let mut n = 0;
        for en in &mut self.enemies {
            if !en.active() || en.hit_by == swing || (en.pos.y - body).abs() > HIT_DY {
                continue;
            }
            let dv = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z);
            if dv.length() - en.radius > reach {
                continue;
            }
            let dir = dv.normalize_or(facing);
            if cone.is_some_and(|(f, half)| dir.dot(f) < half) {
                continue;
            }
            en.hit_by = swing;
            n += 1;
            let at = en.pos + Vec3::Y * en.height * 0.6;
            let crit = en.exposed > 0.0;
            en.hp -= hurt_by(
                en,
                self.stats.dmg + d.damage,
                crit,
                self.stats.opened_dmg,
                t,
            );
            if crit {
                en.exposed = 0.0;
                ev.push(Event::Crit { at });
            }
            en.hurt = 0.15;
            let mass = if en.boss { 0.25 } else { 1.0 };
            en.knock = dir * (t[P::Knockback] * 0.5 + push * KNOCK_DECAY) * mass;
            en.sway = dir * HEAVY_SWAY;
            en.sway_t = 0.0;
            self.sparks.extend(sparks_at(
                en.pos + Vec3::Y * en.height * 0.55,
                dir,
                swing,
                true,
            ));
            add_poise(en, d.poise * self.stats.poise, None, t, &mut ev);
            ev.push(Event::Hit { at });
            if en.hp <= 0 {
                en.state = EState::Dead;
                self.kills += 1;
                ev.push(Event::EnemyDown { at: en.pos });
            } else if stagger > 0.0 && !en.boss && !en.rising() {
                en.state = EState::Stagger {
                    t: 0.0,
                    dur: stagger,
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
    /// 보스·엘리트는 끌리지 않는다(무겁다 — 0150 4절, 엘리트까지 끄는 것은 다듬기 몫).
    fn skill_pull(
        &mut self,
        id: skills::Id,
        dir: Vec3,
        range: f32,
        arc_deg: f32,
        to: f32,
        stagger: f32,
    ) {
        let p = self.player.pos;
        let body = p.y + self.player.air;
        let spot = p + dir * to;
        let half = (arc_deg * 0.5).to_radians().cos();
        for en in &mut self.enemies {
            if !en.active() || en.boss || en.elite.is_some() || en.rising() {
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
        }
        self.skill_ev.push(Event::Skill { id, at: spot, dir });
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
    fn upend(&mut self, t: &Tuning) {
        let p = self.player.pos;
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
            en.slam_bonus = crate::tuning::ULT_SLAM_DAMAGE;
            en.sway = out;
            en.sway_t = 0.0;
            en.state = EState::Stagger {
                t: 0.0,
                dur: if en.boss {
                    UPEND.boss_stagger
                } else {
                    UPEND.stagger
                },
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
        let full = id.def().cooldown * t[P::SkillCooldownX];
        if full <= 0.0 {
            1.0
        } else {
            1.0 - (self.player.skill_cd[id.index()] / full).clamp(0.0, 1.0)
        }
    }

    /// 스킬 둘을 끼운다(같은 것 둘은 바로잡는다). 대기는 스킬마다라 그대로 남는다.
    pub fn set_skills(&mut self, slots: [skills::Id; 2]) {
        self.player.skills = skills::sane(slots);
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
            e.hp = 9;
            e.max_hp = 9;
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
            assert_eq!(e.hp, 9 - 2 * dmg, "hit once per turn");
            assert!(flat(e.pos, w.player.pos) > 2.8, "pushed out: {:?}", e.pos);
        }
        assert_eq!(w.enemies[6].hp, 9, "out of reach");
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
            assert_eq!(e.hp, 9 - d.damage - w.stats.dmg);
            assert!(e.pos.z > w.player.pos.z, "left behind");
        }
        assert_eq!(w.enemies[3].hp, 9, "off the lane");
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
            assert_eq!(e.hp, 9, "gather does no harm");
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
        assert_eq!(w.enemies[0].hp, 9 - d.damage - w.stats.dmg);
        assert!(matches!(w.enemies[0].state, EState::Stagger { .. }));
        assert_eq!(w.enemies[1].hp, 9, "behind");
        assert_eq!(w.enemies[2].hp, 9, "to the side");
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
        // 실제 판: 휘돌이가 체력 1짜리 둘을 쓰러뜨리면 다른 스킬의 대기가 "스킬 몫"만큼 당겨진다.
        let (mut w, t) = world();
        w.spawn_mob(vec3(1.5, 0.0, 0.0));
        w.spawn_mob(vec3(-1.5, 0.0, 0.0));
        still(&mut w);
        for e in &mut w.enemies {
            e.hp = 1;
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
        assert_eq!(w.enemies[0].hp, 9, "no direct damage");
        assert_eq!(w.enemies[1].hp, 9);
        assert!(ev.iter().any(|e| matches!(e, Event::Slam { .. })));
        assert_eq!(
            w.enemies[2].hp,
            9 - crate::tuning::ULT_SLAM_DAMAGE,
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
}
