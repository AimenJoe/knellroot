//! 스킬 틱(0151, 표는 `skills.rs`): 누름 → 받아 두기(`skill_press`), 시작(`try_skill`), 상태 틱(`tick_skill`·`tick_ult`),
//! 판정(`skill_area`·`skill_pull`·`upend`), 스킬이 남긴 것(`tick_skill_fields` — 곧 터질 자리·갈라진 땅, 0158 G1),
//! 틱 끝의 셈(`skill_books` — 기세·게이지). `tick_player`의 한 갈래다.
//!
//! G2(0158): 버팀(버티는 동안의 표지는 여기서 적고 맞음 길 `tick_enemies`가 읽는다 · 끝의 터뜨림), 되울림(패링 길이 `echo_ring`을
//! 부른다), 던지기(`Blade` — `tick_skill_fields`가 날린다), 잔불 흩기(`Field`), 큰 기술 셋(`start_ult`·`tick_ult` — 쟁반 뒤집기 `upend`
//! · 몰아치기 · 한 줄기 `line_cut`). 새 스킬들의 타는 `skill_touch` 한 곳을 탄다.
//!
//! 단과 고르기(0158): 숫자는 `World::skill_eff`(= `Skill::at(단, 고르기)`)가 준 값만 쓴다 — 여기는 `Twist`·`Cap`의 갈래만 본다.
use super::player_tick::Ctx;
use super::*;
use crate::skills::{Cap, Eff, Form, Twist, FLURRY, LINE, UPEND};
use crate::tuning::{
    BLADE_CATCH, BLADE_HEIGHT, SKILL_DENIED_SHOW, SKILL_HITSTOP, ULT_HITSTOP, ULT_KILL_WINDOW,
};

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

/// 던진 무기(0158 G2, 던지기): 곧게 `dist` m를 날아가(가는 길) 방랑자에게 되돌아온다(오는 길) — 길의 반 폭 `half` m 안의 적을
/// 길마다 한 번씩 친다. `stick` = 돌아오지 않고 끝에서 터진다(둘레·피해), `drag` = 오는 길에 맞은 적을 끄는 거리. 렌더가 그 자리에 무기를 그린다.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Blade {
    pub pos: Vec3,
    pub prev: Vec3,
    pub dir: Vec3,
    pub went: f32,
    pub dist: f32,
    pub half: f32,
    pub speed: f32,
    /// 가는 길인가(거짓 = 돌아오는 길).
    pub out: bool,
    /// 이 길에서 이미 맞은 적(번호 비트 — 길이 바뀌면 비운다).
    pub hits: u64,
    pub damage: i32,
    pub poise: f32,
    /// 오는 길의 피해·자세 피해(맺음이면 더 세다).
    pub back_damage: i32,
    pub back_poise: f32,
    pub drag: f32,
    pub stick: Option<(f32, i32)>,
    /// 날아온 초(렌더의 회전).
    pub age: f32,
}

/// 남은 불티(0158 G2, 잔불 흩기): `pos` 둘레 `radius` m에 `secs`초 — `every`초마다 그 안의 적을 태운다(`pulses` = 지금까지 태운 수).
/// `follow` = 방랑자를 따라온다, `kindle` = 그 안에서 쓰러진 적이 더 남기는 잔불 수. 렌더가 바닥에 그린다.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Field {
    pub pos: Vec3,
    pub radius: f32,
    pub age: f32,
    pub secs: f32,
    pub every: f32,
    pub pulses: u32,
    pub damage: i32,
    pub poise: f32,
    pub follow: bool,
    pub kindle: u32,
}

impl Field {
    /// 이 자리(몸 반지름 `radius`)가 불티 안인가.
    pub fn holds(&self, at: Vec3, radius: f32) -> bool {
        (at.y - self.pos.y).abs() <= HIT_DY
            && vec3(at.x - self.pos.x, 0.0, at.z - self.pos.z).length() - radius <= self.radius
    }

    /// 남은 몫 0..1(HUD 칸·렌더의 옅어짐).
    pub fn left(&self) -> f32 {
        (1.0 - self.age / self.secs.max(1e-3)).clamp(0.0, 1.0)
    }
}

/// 새 스킬들의 타 하나(`skill_touch`): 피해(0 = 없음)·자세 피해·휘청(보스는 휘청이지 않는다)·밀려나는 속도·치명타를 쓰나
/// (남는 피해 — 불티·울림 — 는 드러난 틈을 쓰지 않는다: 반격의 치명타를 먹어 치우지 않게).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Touch {
    damage: i32,
    poise: f32,
    stagger: f32,
    knock: Vec3,
    crit: bool,
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
        match pl.skill.worn.iter().position(|s| *s == id) {
            Some(i) => id.def().at(pl.skill.rank[i], pl.skill.pick[i]),
            None => id.def().at(1, skills::Pick::A),
        }
    }

    /// 이번 틱의 스킬 누름 → 받아 둘 행동(0151 3절). 둘을 함께 눌렀거나(같은 틱) 터치의 큰 기술 버튼이면 큰 기술,
    /// 스킬의 시동 안(`skill.chord s`)에 **다른** 스킬 버튼이 들어와도 큰 기술 — 게이지가 가득일 때만. 그 밖에는 그 칸의 스킬.
    /// 대기가 한참 남은 스킬은 받아 두지 않고 칸만 깜빡인다(몇 초 뒤에 저절로 나가면 안 된다).
    pub(super) fn skill_press(&mut self, a: &Actions, t: &Tuning) -> Option<Next> {
        let pl = &mut self.player;
        let full = pl.skill.gauge >= 1.0;
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
            if full && e < t[P::SkillChord] && pl.skill.worn[slot] != id {
                return Some(Next::Ult);
            }
        }
        // 빈 칸(0166 — 안 배운 스킬이 놓인 자리, 단 0): 눌러도 아무 일 없다(받아 두지도, 깜빡이지도 않는다).
        if pl.skill.rank[slot] == 0 {
            return None;
        }
        if pl.skill.cd[pl.skill.worn[slot].index()] > t[P::AttackBuffer] {
            if pl.skill.denied <= 0.0 {
                self.skill_ev.push(Event::SkillDenied);
            }
            pl.skill.denied = SKILL_DENIED_SHOW;
            return None;
        }
        Some(Next::Skill(slot as u8))
    }

    /// 서 있을 때 받은 스킬 누름(캔슬로 넘어온 것 포함)을 시작한다.
    pub(super) fn try_skill(&mut self, cx: &Ctx) -> bool {
        let a = &cx.a;
        if a.ult {
            if self.player.skill.gauge < 1.0 {
                return false;
            }
            self.start_ult(cx);
            return true;
        }
        let slot = if a.skill1 {
            0
        } else if a.skill2 {
            1
        } else {
            return false;
        };
        let id = self.player.skill.worn[slot];
        if self.player.skill.rank[slot] == 0 || self.player.skill.cd[id.index()] > 0.0 {
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
        pl.skill.cd[id.index()] = d.skill.cooldown * cx.t[P::SkillCooldownX];
        pl.skill.leap = leap;
        pl.skill.refunded = false;
        pl.skill.ghost = false;
        pl.skill.brace = 0;
        if matches!(d.skill.form, Form::Brace { .. }) {
            pl.skill.brace_soaked = 0;
            self.brace_caught.clear();
        }
        let pl = &mut self.player;
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
            Form::Ring { .. } | Form::Brace { .. } | Form::Echo { .. } | Form::Field { .. } => {
                facing
            }
            // 던지기: 조준 > 락온 > 닿는 거리의 가장 가까운 적 > 바라보는 쪽.
            Form::Throw { dist, .. } => cx
                .aim
                .or(lock)
                .or_else(|| nearest_in_reach(&self.enemies, pl.pos, dist, None))
                .unwrap_or(facing),
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
        if cx.a.ult && e < t[P::SkillChord] && self.player.skill.gauge >= 1.0 {
            self.player.skill.cd[id.index()] = 0.0;
            self.player.skill.ghost = false;
            self.player.skill.brace = 0;
            self.start_ult(cx);
            return;
        }
        let e1 = e + dt;
        let crossed = |at: f32| e < at && e1 >= at;
        let at = self.player.pos;
        let mut swing = swing;
        self.player.skill.ghost = false;
        self.player.skill.brace = 0;
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
                    self.player.skill.ghost = matches!(d.twist, Some(Twist::Ghost));
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
                        if self.kills > kills && !pl.skill.refunded {
                            pl.skill.refunded = true;
                            let back = s.cooldown * t[P::SkillCooldownX] * part;
                            let cd = &mut pl.skill.cd[id.index()];
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
            // 버팀(G2): 버티는 동안은 표지만 적는다(덜 다치고·밀리지 않고·끊기지 않는 것은 맞음 길이 한다) — 회피는 터뜨림 없이 끊는다.
            // 끝에 둘레를 터뜨린다: 표의 피해 + 받아낸 타마다 덤.
            Form::Brace {
                hold,
                radius,
                soak,
                most,
                more,
                push,
            } => {
                let end = s.startup + hold;
                if e1 >= s.startup && e1 < end {
                    if cx.a.dodge && self.try_dodge(cx) {
                        return;
                    }
                    let pl = &mut self.player;
                    pl.skill.brace = soak;
                    pl.skill.brace_most = most;
                    pl.skill.reflect = matches!(d.twist, Some(Twist::Reflect));
                }
                if crossed(end) {
                    self.player.swings += 1;
                    swing = self.player.swings;
                    let n = self.player.skill.brace_soaked as i32;
                    self.skill_ev.push(Event::Skill { id, at, dir });
                    let stun = match d.cap {
                        Some(Cap::Stun { stagger }) => stagger,
                        _ => 0.0,
                    };
                    self.skill_area(
                        t,
                        &Sweep {
                            reach: radius,
                            cone: None,
                            push,
                            stagger: stun,
                            swing,
                            damage: s.damage + more * n,
                            poise: s.poise,
                            shatter: 1.0,
                        },
                    );
                    // 되쏘기(ㄴ): 받아 둔 조각을 던진 쪽으로 — 패링으로 되돌린 조각과 같은 길(`reflected`).
                    for from in std::mem::take(&mut self.brace_caught) {
                        let Some(en) = self.enemies.get(from).filter(|en| en.alive()) else {
                            continue;
                        };
                        let pos = at + Vec3::Y * 1.1;
                        let to = vec3(en.pos.x - pos.x, 0.0, en.pos.z - pos.z);
                        self.shards.push(Shard {
                            pos,
                            dir: to.normalize_or(dir),
                            went: 0.0,
                            reflected: true,
                            from,
                        });
                    }
                }
            }
            // 되울림(G2): 시동 뒤 몸에 울림을 두른다 — 그동안의 패링은 `echo_ring`이 둘레로 퍼뜨린다.
            Form::Echo { secs, .. } => {
                if crossed(s.startup) {
                    let pl = &mut self.player;
                    pl.skill.echo_left = secs;
                    pl.skill.echo_total = secs;
                    self.skill_ev.push(Event::Skill { id, at, dir });
                }
            }
            // 던지기(G2): 시동 끝에 손을 떠난다 — 그 뒤는 `tick_skill_fields`가 날린다(몸은 후딜 뒤 자유).
            Form::Throw {
                dist, half, speed, ..
            } => {
                if crossed(s.startup) {
                    let (back_damage, back_poise) = match d.cap {
                        Some(Cap::Homing { damage, poise }) => {
                            (skills::scaled(s.damage, damage), s.poise * poise)
                        }
                        _ => (s.damage, s.poise),
                    };
                    let pos = at + Vec3::Y * (self.player.air + BLADE_HEIGHT) + dir * 0.5;
                    self.skill_blades.push(Blade {
                        pos,
                        prev: pos,
                        dir,
                        went: 0.0,
                        dist,
                        half,
                        speed,
                        out: true,
                        hits: 0,
                        damage: s.damage,
                        poise: s.poise,
                        back_damage,
                        back_poise,
                        drag: match d.twist {
                            Some(Twist::Drag { dist }) => dist,
                            _ => 0.0,
                        },
                        stick: match d.twist {
                            Some(Twist::Stick { radius, damage }) => Some((radius, damage)),
                            _ => None,
                        },
                        age: 0.0,
                    });
                    self.skill_ev.push(Event::Skill { id, at, dir });
                }
            }
            // 잔불 흩기(G2): 선 자리에 불티가 남는다 — 태우는 것은 `tick_skill_fields`.
            Form::Field {
                radius,
                secs,
                every,
            } => {
                if crossed(s.startup) {
                    self.skill_fields.push(Field {
                        pos: at,
                        radius,
                        age: 0.0,
                        secs,
                        every,
                        pulses: 0,
                        damage: s.damage,
                        poise: match d.cap {
                            Some(Cap::Smolder { poise, .. }) => poise,
                            _ => 0.0,
                        },
                        follow: matches!(d.twist, Some(Twist::Follow)),
                        kindle: match d.twist {
                            Some(Twist::Kindle { embers }) => embers,
                            _ => 0,
                        },
                    });
                    self.skill_ev.push(Event::Skill { id, at, dir });
                }
            }
        }
        self.player.state = if e1 >= d.total() {
            self.player.skill.ghost = false;
            self.player.skill.brace = 0;
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
        self.tick_blades(t, dt);
        self.tick_fields(t, dt);
    }

    /// 던진 무기(G2)를 날린다: 가는 길은 곧게 `dist` m(벽·기둥·아레나 끝에 닿으면 거기서) — 끝에서 되돌거나(오는 길은 지금의
    /// 방랑자에게, 벽을 가리지 않는다 — 든 것을 잃지 않게) 터진다(박힌 자리). 길 위의 적은 길마다 한 번씩 맞는다.
    fn tick_blades(&mut self, t: &Tuning, dt: f32) {
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
                    self.skill_blast(t, &mut ev, at, radius, damage);
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
    fn tick_fields(&mut self, t: &Tuning, dt: f32) {
        let pp = self.player.pos;
        let mut due: Vec<Field> = Vec::new();
        for f in &mut self.skill_fields {
            f.age += dt;
            if f.follow {
                f.pos = pp;
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
                        },
                    );
                }
                ev.push(Event::FieldPulse {
                    at: f.pos,
                    radius: f.radius,
                });
            }
            self.skill_ev = ev;
        }
        self.skill_fields.retain(|f| f.age + 1e-4 < f.secs);
    }

    /// 새 스킬들의 타 하나(G2 — 던진 무기·불티·울림·한 줄기): 피해는 여느 타와 같은 길(`hurt_by` — 장신구의 덤·깨진 동안의 배·
    /// 더 다치는 표지), 자세 피해, 쓰러짐, 휘청(보스·전환 중인 보스는 휘청이지 않는다 — 이미 더 길게 휘청이는 적은 그대로).
    /// 스킬의 타라 큰 기술 게이지를 채우지 않고(`connected`를 건드리지 않는다), 쓰러뜨리면 스킬 몫의 기세. 쓰러뜨렸으면 참.
    fn skill_touch(&mut self, t: &Tuning, ev: &mut Vec<Event>, i: usize, c: &Touch) -> bool {
        let en = &mut self.enemies[i];
        let at = en.pos + Vec3::Y * en.height * 0.6;
        if c.damage > 0 {
            let crit = c.crit && en.exposed > 0.0;
            en.hp -= hurt_by(
                en,
                self.stats.dmg + c.damage,
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

    /// 둘레 터짐(G2 — 던지기의 박힌 자리): `at` 둘레 `radius` m의 깨어 있는 적에게 피해, 바깥으로 민다(보스는 조금).
    fn skill_blast(&mut self, t: &Tuning, ev: &mut Vec<Event>, at: Vec3, radius: f32, damage: i32) {
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
                    crit: true,
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
    pub(super) fn echo_ring(&mut self, t: &Tuning, parries: usize, ev: &mut Vec<Event>) {
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

    /// 큰 기술 시작(끼운 하나): 게이지를 통째로 쓴다. 쟁반 뒤집기·한 줄기는 세계가 한 번 멈춘다(시동 정지 — "시간"),
    /// 한 줄기는 가를 쪽을 이 순간에 정한다(조준 > 락온 > 닿는 거리의 가장 가까운 적 > 바라보는 쪽).
    fn start_ult(&mut self, cx: &Ctx) {
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

    pub(super) fn tick_ult(&mut self, cx: &Ctx, kind: skills::Ult, e: f32, done: bool) {
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
    fn line_cut(&mut self, t: &Tuning) {
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
    fn upend(&mut self, t: &Tuning) {
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
            for cd in &mut pl.skill.cd {
                *cd = (*cd - per * downs).max(0.0);
            }
        }
        // 몰아치기 3단(G2): 그동안 쓰러뜨릴 때마다 조금 더 이어진다 — 더해지는 몫은 `kill_cap`초까지.
        if downs > 0.0 && pl.skill.flurry_left > 0.0 {
            let more = (FLURRY.kill_more_at(pl.skill.ult_rank) * downs)
                .min(FLURRY.kill_cap - pl.skill.flurry_added)
                .max(0.0);
            pl.skill.flurry_left += more;
            pl.skill.flurry_total += more;
            pl.skill.flurry_added += more;
        }
        let was_full = pl.skill.gauge >= 1.0;
        let hit = t[P::SkillGaugeHit];
        let mut gain = parries * t[P::SkillGaugeParry] + perfects * hit * 2.0;
        if pl.connected != 0 && pl.connected != pl.skill.gauge_swing {
            pl.skill.gauge_swing = pl.connected;
            gain += hit;
        }
        if std::mem::take(&mut self.bolt_hit_tick) {
            gain += hit;
        }
        pl.skill.gauge = (pl.skill.gauge + gain).min(1.0);
        if !was_full && pl.skill.gauge >= 1.0 {
            ev.push(Event::UltReady);
        }
    }

    /// 스킬 칸 `slot`(0·1)의 준비 0..1(1 = 쓸 수 있다) — HUD·터치.
    pub fn skill_ready(&self, slot: usize, t: &Tuning) -> f32 {
        let id = self.player.skill.worn[slot % 2];
        let full = self.skill_eff(id).skill.cooldown * t[P::SkillCooldownX];
        if full <= 0.0 {
            1.0
        } else {
            1.0 - (self.player.skill.cd[id.index()] / full).clamp(0.0, 1.0)
        }
    }

    /// 스킬 칸 `slot`의 스킬이 남긴 것이 이어지는 몫 0..1(0 = 없음) — 되울림의 울림·잔불 흩기의 불티(HUD·터치 칸).
    pub fn skill_active(&self, slot: usize) -> f32 {
        let pl = &self.player;
        match pl.skill.worn[slot % 2] {
            skills::Id::Echo if pl.skill.echo_left > 0.0 => {
                (pl.skill.echo_left / pl.skill.echo_total.max(1e-3)).clamp(0.0, 1.0)
            }
            skills::Id::Embers => self
                .skill_fields
                .iter()
                .map(|f| f.left())
                .fold(0.0, f32::max),
            _ => 0.0,
        }
    }

    /// 몰아치기가 이어지는 몫 0..1(0 = 아니다) — HUD가 큰 기술 게이지를 줄어드는 막대로 그린다.
    pub fn ult_active(&self) -> f32 {
        let pl = &self.player;
        if pl.skill.flurry_left > 0.0 {
            (pl.skill.flurry_left / pl.skill.flurry_total.max(1e-3)).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// 든 것이 손을 떠나 있나(던지기, G2) — 렌더가 손의 무기를 감추고 날아가는 자리에 그린다.
    pub fn weapon_away(&self) -> bool {
        !self.skill_blades.is_empty()
    }

    /// 스킬 둘을 끼운다(같은 것 둘은 바로잡는다) — 그 단·고르기와 큰 기술의 단까지(0158). 스킬 이름만 주면 모두 1단.
    /// 대기는 스킬마다라 그대로 남는다.
    pub fn set_skills(&mut self, l: impl Into<skills::Loadout>) {
        let l = l.into();
        let ids = skills::sane([l.slots[0].id, l.slots[1].id]);
        let pl = &mut self.player;
        pl.skill.worn = ids;
        for (i, id) in ids.into_iter().enumerate() {
            // 같은 스킬 둘이라 바꿔 끼운 칸은 1단. 단 0 = 빈 칸(0166 — 안 배운 스킬이 놓인 자리: 눌러도 아무 일 없다).
            let same = l.slots[i].id == id;
            pl.skill.rank[i] = if same { l.slots[i].rank } else { 1 };
            pl.skill.pick[i] = if same {
                l.slots[i].pick
            } else {
                skills::Pick::A
            };
        }
        pl.skill.ult_rank = l.ult.max(1);
        pl.skill.ult = l.great;
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

    /// 0166: 안 배운 스킬이 놓인 칸은 빈 칸(단 0)이다 — 눌러도 시작하지 않고, 받아 두지도 "안 됨"을 알리지도 않는다.
    /// 다른 칸의 배운 스킬은 그대로 나가고, 두 버튼 "함께"의 큰 기술도 그대로다(큰 기술은 처음부터 가진다).
    #[test]
    fn an_empty_slot_does_nothing_when_pressed() {
        use crate::skills::{Loadout, Pick, Slot, Ult};
        let (mut w, t) = world();
        w.spawn_mob(vec3(0.0, 0.0, -2.0));
        still(&mut w);
        let slot = |id, rank| Slot {
            id,
            rank,
            pick: Pick::A,
        };
        w.set_skills(Loadout {
            slots: [slot(Sk::Whirl, 0), slot(Sk::Cut, 1)],
            ult: 1,
            great: Ult::Upend,
        });
        assert_eq!(w.player.skill.rank, [0, 1]);
        let ev = run(&mut w, &t, skill(0), 30);
        assert!(
            !ev.iter().any(|e| matches!(
                e,
                Event::SkillStart { .. } | Event::Skill { .. } | Event::SkillDenied
            )),
            "{ev:?}"
        );
        assert!(matches!(w.player.state, PState::Idle));
        assert_eq!(w.player.skill.cd[Sk::Whirl.index()], 0.0, "no cooldown");
        assert_eq!(w.enemies[0].hp, 90);
        // 배운 칸은 그대로.
        let ev = run(&mut w, &t, skill(1), 5);
        assert!(ev
            .iter()
            .any(|e| matches!(e, Event::SkillStart { id: Sk::Cut })));
        // 둘 다 빈 칸이어도 게이지가 차면 "함께" = 큰 기술.
        let (mut w, t) = world();
        w.set_skills(Loadout {
            slots: [slot(Sk::Whirl, 0), slot(Sk::Cut, 0)],
            ult: 1,
            great: Ult::Upend,
        });
        w.player.skill.gauge = 1.0;
        let both = Actions {
            skill1: true,
            skill2: true,
            ..Default::default()
        };
        run(&mut w, &t, both, 3);
        assert!(
            matches!(w.player.state, PState::Ult { .. }),
            "{:?}",
            w.player.state
        );
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
        assert!(w.player.skill.cd[Sk::Whirl.index()] > 4.0, "cooling down");
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
        w.player.skill.cd = [5.0; skills::N];
        let mut ev = vec![down, down];
        w.skill_books(&t, &mut ev);
        let want = 5.0 - 2.0 * t[P::SkillMomentum];
        assert!(w.player.skill.cd.iter().all(|c| (c - want).abs() < 1e-5));
        w.player.skill.cd = [5.0; skills::N];
        w.skill_hit_tick = true;
        w.skill_books(&t, &mut vec![down]);
        let want = 5.0 - t[P::SkillMomentumOwn];
        assert!(w.player.skill.cd.iter().all(|c| (c - want).abs() < 1e-5));
        assert!(t[P::SkillMomentumOwn] < t[P::SkillMomentum]);
        // 실제 판: 휘돌이가 한 대 몫(체력 10)짜리 둘을 쓰러뜨리면 다른 스킬의 대기가 "스킬 몫"만큼 당겨진다.
        let (mut w, t) = world();
        w.spawn_mob(vec3(1.5, 0.0, 0.0));
        w.spawn_mob(vec3(-1.5, 0.0, 0.0));
        still(&mut w);
        for e in &mut w.enemies {
            e.hp = 10;
        }
        w.player.skill.cd[Sk::Cut.index()] = 5.0;
        let ticks = 12;
        let ev = run(&mut w, &t, skill(0), ticks);
        let downs = ev
            .iter()
            .filter(|e| matches!(e, Event::EnemyDown { .. }))
            .count();
        assert_eq!(downs, 2);
        let cd = w.player.skill.cd[Sk::Cut.index()];
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
        assert!((w.player.skill.gauge - t[P::SkillGaugeHit]).abs() < 1e-5);
        // 스킬의 타는 채우지 않는다.
        let g = w.player.skill.gauge;
        let ev = run(&mut w, &t, skill(0), 60);
        assert!(ev.iter().any(|e| matches!(e, Event::Hit { .. })));
        assert_eq!(w.player.skill.gauge, g);
        // 패링 한 번 = 맞힘 여러 번.
        let parry = Event::Parry { at: Vec3::ZERO };
        w.skill_books(&t, &mut vec![parry]);
        assert!((w.player.skill.gauge - g - t[P::SkillGaugeParry]).abs() < 1e-5);
        assert!(t[P::SkillGaugeParry] >= 3.0 * t[P::SkillGaugeHit]);
        // 가득: 한 번만 알린다.
        w.player.skill.gauge = 1.0 - t[P::SkillGaugeParry] * 0.5;
        let mut ev = vec![parry];
        w.skill_books(&t, &mut ev);
        assert_eq!(w.player.skill.gauge, 1.0);
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
        w.player.skill.gauge = 1.0;
        let ev = w.step(&both, &t);
        assert!(matches!(w.player.state, PState::Ult { .. }));
        assert!(ev.contains(&Event::UltStart));
        assert_eq!(w.player.skill.gauge, 0.0);
        assert!(w.player.skill.cd.iter().all(|c| *c == 0.0));
        // 두 틱 어긋나게.
        let (mut w, t) = world();
        w.player.skill.gauge = 1.0;
        w.step(&skill(0), &t);
        assert!(
            matches!(w.player.state, PState::Skill { id: Sk::Whirl, .. }),
            "a single press starts at once"
        );
        w.step(&Actions::default(), &t);
        w.step(&skill(1), &t);
        assert!(matches!(w.player.state, PState::Ult { .. }));
        assert_eq!(w.player.skill.cd[Sk::Whirl.index()], 0.0, "refunded");
        // 시동 창이 지난 뒤의 둘째 누름은 큰 기술이 아니다 — 다음 스킬로 받아 둔다.
        let (mut w, t) = world();
        w.player.skill.gauge = 1.0;
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
        assert_eq!(w.player.skill.gauge, 1.0);
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
        w.player.skill.gauge = 1.0;
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
        w.player.skill.gauge = 1.0;
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
        let cd = w.player.skill.cd[Sk::Whirl.index()];
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
            great: skills::Ult::Upend,
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
            great: skills::Ult::Upend,
        });
        assert_eq!(w.player.skill.worn, [Sk::Quake, Sk::Whirl]);
        assert_eq!(w.player.skill.rank, [5, 1]);
        assert_eq!(w.player.skill.pick, [Pick::B, Pick::A]);
        assert_eq!(w.player.skill.ult_rank, 3);
        assert_eq!(w.skill_eff(Sk::Quake).skill.damage, 26);
        assert_eq!(w.skill_eff(Sk::Whirl).skill.damage, 10);
        assert_eq!(w.skill_eff(Sk::Gather).twist, None, "not worn: rank 1");
        // 5단의 대기는 ×0.9 — 쓴 직후 남은 대기가 그만큼.
        w.step(&skill(0), &t);
        let cd = w.player.skill.cd[Sk::Quake.index()];
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
        assert_eq!(w.player.skill.gauge, 0.0, "a skill hit: no gauge");
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
            (w.player.skill.cd[Sk::Cut.index()], w, t)
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
            great: skills::Ult::Upend,
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
            w.player.skill.gauge = 1.0;
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
                for k in 0..150 {
                    // 되울림(G2)은 패링이 있어야 울린다: 울림을 두른 뒤 잡몹이 앞에서 내려치게 하고 받아친다(보스는 둘레 안).
                    let a = if id == Sk::Echo && k == 30 {
                        let en = &mut w.enemies[1];
                        en.pos = vec3(0.0, 0.0, -1.2);
                        en.facing = 0.0;
                        en.atk = Atk::Smash;
                        en.state = EState::Tele { t: en.tele_dur() };
                        w.player.facing = PI;
                        Actions {
                            parry: true,
                            ..Default::default()
                        }
                    } else {
                        Actions::default()
                    };
                    ev.extend(w.step(&a, &t));
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
                // 패링 한 번은 패링의 몫만 채운다(울림의 타는 채우지 않는다).
                let parries = ev
                    .iter()
                    .filter(|e| matches!(e, Event::Parry { .. }))
                    .count();
                if id == Sk::Echo {
                    assert_eq!(parries, 1, "{tag}: the echo test parries once");
                    assert!(
                        ev.iter().any(|e| matches!(e, Event::EchoRing { .. })),
                        "{tag}"
                    );
                }
                let want = parries as f32 * t[P::SkillGaugeParry];
                assert!(
                    (w.player.skill.gauge - want).abs() < 1e-5,
                    "{tag}: skill hits fill no gauge ({})",
                    w.player.skill.gauge
                );
                assert_eq!(w.player.stamina, stamina, "{tag}: no stamina");
                assert!(w.player.skill.cd[id.index()] > 1.0, "{tag}: cooling down");
            }
        }
    }

    // ---- G2(0158): 나머지 스킬 넷 · 큰 기술 둘 ----

    use crate::skills::Ult;

    /// 칸 0에 `id`를 끼우고 큰 기술 `great`를 `ult`단으로.
    fn wear_ult(w: &mut World, great: Ult, ult: u8) {
        w.set_skills(Loadout {
            slots: [
                Slot {
                    id: Sk::Whirl,
                    rank: 1,
                    pick: Pick::A,
                },
                Slot {
                    id: Sk::Cut,
                    rank: 1,
                    pick: Pick::A,
                },
            ],
            ult,
            great,
        });
    }

    /// 앞 1.2m에서 이 틱에 내려치는 잔재(맞으면 한 눈금).
    fn swing_at_player(w: &mut World, i: usize) {
        let p = w.player.pos;
        let en = &mut w.enemies[i];
        en.pos = p + vec3(0.0, 0.0, -1.2);
        en.prev = en.pos;
        en.facing = 0.0;
        en.atk = Atk::Smash;
        en.state = EState::Swing { t: 0.0 };
    }

    fn ticks(secs: f32) -> usize {
        (secs / TICK_DT).ceil() as usize
    }

    /// 버팀: 버티는 동안 맞아도 밀리지도 끊기지도 않고 한 대 몫(10)을 덜어 다치지 않는다 — 받아낸 타는 셋까지 센다.
    /// 끝의 터뜨림은 둘레 3m에 10 + 받아낸 타마다 10(셋이면 40). 아무것도 안 받아냈으면 10.
    #[test]
    fn brace_soaks_blows_without_being_moved_and_bursts_harder_for_each() {
        let burst = |blows: usize| {
            let (mut w, t) = world();
            wear(&mut w, Sk::Brace, 1, Pick::A);
            let hitter = w.spawn(Kind::Husk, vec3(0.0, 0.0, -1.2));
            let side = w.spawn(Kind::Husk, vec3(2.0, 0.0, 1.0));
            w.spawn_mob(vec3(0.0, 0.0, 6.0));
            still(&mut w);
            let (hp, at) = (w.player.hp, w.player.pos);
            let mut ev = run(&mut w, &t, skill(0), ticks(0.12));
            for _ in 0..blows {
                swing_at_player(&mut w, hitter);
                ev.extend(run(&mut w, &t, Actions::default(), 2));
                assert!(
                    matches!(w.player.state, PState::Skill { id: Sk::Brace, .. }),
                    "not interrupted: {:?}",
                    w.player.state
                );
                w.enemies[hitter].state = EState::Recover { t: -1000.0 };
                ev.extend(run(&mut w, &t, Actions::default(), ticks(0.2)));
            }
            ev.extend(run(&mut w, &t, Actions::default(), 120));
            let soaks = ev
                .iter()
                .filter(|e| matches!(e, Event::Soak { .. }))
                .count();
            assert_eq!(soaks, blows, "every blow is soaked");
            assert_eq!(w.player.hp, hp, "a blow of 10 minus 10");
            assert!(flat(w.player.pos, at) < 0.05, "not pushed");
            assert!(!ev.contains(&Event::PlayerHurt), "no hurt feedback");
            (
                90 - w.enemies[side].hp,
                w.enemies[2].hp,
                w.player.skill.brace_soaked,
            )
        };
        assert_eq!(burst(0), (10, 90, 0), "the far one is out of reach");
        assert_eq!(burst(1), (20, 90, 1));
        assert_eq!(burst(3), (40, 90, 3));
        assert_eq!(burst(4), (40, 90, 3), "three at most");
        // 버티지 않으면 같은 타가 맞히고 경직시킨다.
        let (mut w, t) = world();
        w.spawn_mob(vec3(0.0, 0.0, -1.2));
        still(&mut w);
        swing_at_player(&mut w, 0);
        let ev = run(&mut w, &t, Actions::default(), 2);
        assert!(ev.contains(&Event::PlayerHurt) && w.player.hp < PLAYER_HP);
    }

    /// 버팀: 버티는 동안 회피는 터뜨림 없이 버팀을 끊는다 — 대기는 쓴 채. 시동 동안·후딜의 회피 받기는 다른 스킬과 같다.
    #[test]
    fn a_dodge_breaks_the_brace_without_a_burst_and_the_cooldown_stays_spent() {
        let (mut w, t) = world();
        wear(&mut w, Sk::Brace, 1, Pick::A);
        w.spawn_mob(vec3(1.5, 0.0, 0.0));
        still(&mut w);
        let mut ev = run(&mut w, &t, skill(0), ticks(0.4));
        assert!(matches!(w.player.state, PState::Skill { .. }));
        ev.extend(w.step(
            &Actions {
                dodge: true,
                ..Default::default()
            },
            &t,
        ));
        assert!(
            matches!(w.player.state, PState::Dodge { .. }),
            "{:?}",
            w.player.state
        );
        ev.extend(run(&mut w, &t, Actions::default(), 120));
        assert!(!ev
            .iter()
            .any(|e| matches!(e, Event::Skill { id: Sk::Brace, .. })));
        assert_eq!(w.enemies[0].hp, 90, "no burst");
        assert!(w.player.skill.cd[Sk::Brace.index()] > 5.0, "spent");
    }

    /// 버팀 3단 ㄱ "버티는 걸음": 버티는 동안 스틱 쪽으로 느리게 걷는다 — 고르기가 없거나 ㄴ이면 제자리.
    #[test]
    fn brace_pick_a_walks_while_bracing() {
        let moved = |rank: u8, pick: Pick| {
            let (mut w, t) = world();
            wear(&mut w, Sk::Brace, rank, pick);
            let go = Actions {
                move_dir: vec2(0.0, 1.0),
                ..Default::default()
            };
            w.step(&Actions { skill1: true, ..go }, &t);
            let mut n = 1;
            while matches!(w.player.state, PState::Skill { .. }) {
                w.step(&go, &t);
                n += 1;
            }
            (flat(w.player.pos, Vec3::ZERO), n as f32 * TICK_DT, t)
        };
        let (d, secs, t) = moved(3, Pick::A);
        assert!(d > 1.0, "walked: {d}");
        assert!(d < t[P::MoveSpeed] * secs * 0.5, "slowly: {d}");
        for (rank, pick) in [(2, Pick::A), (3, Pick::B)] {
            let (d, ..) = moved(rank, pick);
            assert!(d < 0.05, "{rank} {pick:?}: {d}");
        }
    }

    /// 버팀 3단 ㄴ "되쏘기": 버티는 동안 받아낸 종 조각을 터뜨릴 때 던진 종지기 쪽으로 되쏜다 — 되돌린 조각이 종지기를 깎는다.
    /// ㄱ이면 받아낼 뿐(종지기는 그대로).
    #[test]
    fn brace_pick_b_sends_soaked_shards_back_at_their_thrower() {
        let hurt_keeper = |pick: Pick| {
            let (mut w, t) = world();
            wear(&mut w, Sk::Brace, 3, pick);
            let b = w.spawn(Kind::Bellkeeper, vec3(0.0, 0.0, -8.0));
            still(&mut w);
            let hp = w.enemies[b].hp;
            run(&mut w, &t, skill(0), ticks(0.15));
            w.shards.push(Shard {
                pos: vec3(0.0, 1.1, -1.6),
                dir: vec3(0.0, 0.0, 1.0),
                went: 0.0,
                reflected: false,
                from: b,
            });
            let mut soaked = false;
            for _ in 0..200 {
                soaked |= w
                    .step(&Actions::default(), &t)
                    .iter()
                    .any(|e| matches!(e, Event::Soak { .. }));
                w.enemies[b].state = EState::Recover { t: -1000.0 };
            }
            assert!(soaked, "{pick:?}: the shard was soaked");
            hp - w.enemies[b].hp
        };
        assert_eq!(hurt_keeper(Pick::B), crate::tuning::SHARD_REFLECT_DAMAGE);
        assert_eq!(hurt_keeper(Pick::A), 0);
    }

    /// 버팀 5단 맺음: 터뜨림에 맞은 적이 1초 휘청인다 — 보스는 휘청이지 않는다. 4단에는 없다.
    #[test]
    fn brace_capstone_staggers_the_burst_but_not_a_boss() {
        let staggers = |rank: u8| {
            let (mut w, t) = world();
            wear(&mut w, Sk::Brace, rank, Pick::A);
            w.spawn_mob(vec3(2.0, 0.0, 0.0));
            w.spawn_boss(vec3(-2.2, 0.0, 0.0));
            still(&mut w);
            let mut most = [0.0f32; 2];
            w.step(&skill(0), &t);
            for _ in 0..150 {
                w.step(&Actions::default(), &t);
                for (i, e) in w.enemies.iter().enumerate() {
                    if let EState::Stagger { dur, .. } = e.state {
                        most[i] = most[i].max(dur);
                    }
                }
            }
            most
        };
        let five = staggers(MAX_RANK);
        assert!((five[0] - 1.0).abs() < 1e-5, "{five:?}");
        assert_eq!(five[1], 0.0, "a boss does not stagger");
        assert_eq!(staggers(4)[0], 0.0);
    }

    /// 앞에서 이 틱에 내려치는 잔재(쪽 −z)와 둘레의 적들 — 되울림의 판.
    fn echo_world(rank: u8, pick: Pick) -> (World, Tuning) {
        let (mut w, t) = world();
        wear(&mut w, Sk::Echo, rank, pick);
        w.spawn_mob(vec3(0.0, 0.0, -1.2)); // 0: 내려친다(패링당한다)
        w.spawn_mob(vec3(3.0, 0.0, 0.5)); // 1: 둘레 4m 안
        w.spawn_mob(vec3(0.0, 0.0, 7.0)); // 2: 밖
        w.spawn_boss(vec3(-2.6, 0.0, 1.0)); // 3: 안의 보스
        still(&mut w);
        (w, t)
    }

    /// 앞의 잔재가 내려치게 하고 패링한다 — 이 판의 사건과 패링 창이 열려 있던 틱 수.
    fn parry_the_front(w: &mut World, t: &Tuning) -> (Vec<Event>, usize) {
        let en = &mut w.enemies[0];
        en.facing = 0.0;
        en.atk = Atk::Smash;
        en.state = EState::Tele { t: en.tele_dur() };
        w.player.facing = PI;
        let mut ev = w.step(
            &Actions {
                parry: true,
                ..Default::default()
            },
            t,
        );
        let mut open = w.parry_open(t) as usize;
        for _ in 0..40 {
            ev.extend(w.step(&Actions::default(), t));
            open += w.parry_open(t) as usize;
        }
        (ev, open)
    }

    /// 되울림: 4초 동안 성공한 패링마다 둘레 4m의 모든 적이 0.8초 휘청인다 — 밖의 적·보스는 아니다, 피해는 없다.
    /// **패링 창은 그대로**(0095 원칙 2): 울림이 있어도 없어도 창이 열린 틱 수가 같다. 울림이 없으면 둘레는 그대로.
    #[test]
    fn echo_makes_each_parry_stagger_everyone_around_and_leaves_the_window_alone() {
        let (mut w, t) = echo_world(1, Pick::A);
        run(&mut w, &t, skill(0), ticks(0.3));
        assert!(
            (w.player.skill.echo_left - 4.0).abs() < 0.1,
            "{}",
            w.player.skill.echo_left
        );
        still(&mut w);
        let (ev, open_echo) = parry_the_front(&mut w, &t);
        assert!(ev.iter().any(|e| matches!(e, Event::Parry { .. })));
        assert!(ev.iter().any(|e| matches!(e, Event::EchoRing { .. })));
        assert!(
            matches!(w.enemies[1].state, EState::Stagger { dur, .. } if (dur - 0.8).abs() < 1e-5),
            "{:?}",
            w.enemies[1].state
        );
        assert!(
            !matches!(w.enemies[2].state, EState::Stagger { .. }),
            "outside"
        );
        assert!(
            !matches!(w.enemies[3].state, EState::Stagger { .. }),
            "a boss"
        );
        assert!(
            w.enemies.iter().skip(1).all(|e| e.hp == e.max_hp),
            "no damage"
        );
        // 울림 없이 같은 패링.
        let (mut w, t) = echo_world(1, Pick::A);
        let (ev, open_plain) = parry_the_front(&mut w, &t);
        assert!(ev.iter().any(|e| matches!(e, Event::Parry { .. })));
        assert!(!ev.iter().any(|e| matches!(e, Event::EchoRing { .. })));
        assert!(!matches!(w.enemies[1].state, EState::Stagger { .. }));
        assert_eq!(open_echo, open_plain, "the parry window is untouched");
        // 2단: 5초. 4초가 지나면 끝(1단).
        let (mut w, t) = echo_world(2, Pick::A);
        run(&mut w, &t, skill(0), ticks(0.3));
        assert!((w.player.skill.echo_total - 5.0).abs() < 1e-5);
        let (mut w, t) = echo_world(1, Pick::A);
        run(&mut w, &t, skill(0), ticks(0.3) + ticks(4.1));
        assert_eq!(w.player.skill.echo_left, 0.0);
        still(&mut w);
        let (ev, _) = parry_the_front(&mut w, &t);
        assert!(
            !ev.iter().any(|e| matches!(e, Event::EchoRing { .. })),
            "over"
        );
    }

    /// 되울림 3단 ㄱ "울리는 틈": 울림이 닿은 적의 자세가 깎인다 · ㄴ "당기는 박자": 울린 패링마다 모든 스킬의 대기 −0.6초.
    #[test]
    fn echo_picks_add_poise_damage_or_pull_cooldowns() {
        let echo = |pick: Pick| {
            let (mut w, t) = echo_world(3, pick);
            run(&mut w, &t, skill(0), ticks(0.3));
            w.player.skill.cd[Sk::Whirl.index()] = 5.0;
            still(&mut w);
            let cd0 = w.player.skill.cd[Sk::Whirl.index()];
            let (_, _) = parry_the_front(&mut w, &t);
            let spent = cd0 - w.player.skill.cd[Sk::Whirl.index()];
            (w.enemies[1].poise, w.enemies[3].poise, spent)
        };
        let (poise_a, boss_a, spent_a) = echo(Pick::A);
        let (poise_b, _, spent_b) = echo(Pick::B);
        assert!(poise_a > 0.0 && boss_a > 0.0, "{poise_a} {boss_a}");
        assert_eq!(poise_b, 0.0);
        assert!(
            (spent_b - spent_a - 0.6).abs() < 0.02,
            "{spent_a} vs {spent_b}"
        );
    }

    /// 되울림 5단 맺음: 울림이 닿은 적에게 피해 10 — 보스도 다치되 휘청이지 않는다. 4단에는 없다.
    #[test]
    fn echo_capstone_hurts_everyone_echoed_but_staggers_no_boss() {
        let hurt = |rank: u8| {
            let (mut w, t) = echo_world(rank, Pick::B);
            run(&mut w, &t, skill(0), ticks(0.3));
            still(&mut w);
            let boss = w.enemies[3].hp;
            parry_the_front(&mut w, &t);
            assert!(!matches!(w.enemies[3].state, EState::Stagger { .. }));
            (90 - w.enemies[1].hp, boss - w.enemies[3].hp)
        };
        assert_eq!(hurt(MAX_RANK), (10, 10));
        assert_eq!(hurt(4), (0, 0));
    }

    /// 던지고 다 돌아올 때까지(또는 `n`틱).
    fn throw_all(w: &mut World, t: &Tuning, a: Actions) -> Vec<Event> {
        let mut ev = w.step(&a, t);
        for _ in 0..400 {
            ev.extend(w.step(&Actions::default(), t));
            if matches!(w.player.state, PState::Idle) && w.skill_blades.is_empty() {
                break;
            }
        }
        ev
    }

    /// 던지기: 든 것이 8m를 곧게 날아가 뚫고 돌아온다 — 길 위의 적은 가는 길·오는 길에 10씩, 길 밖은 그대로. 후딜 뒤에는 무기가
    /// 날고 있어도 움직이고 평타를 낼 수 있다(무기는 손에 없는 것으로 그린다). 돌아오면 손에.
    #[test]
    fn throw_flies_out_through_the_lane_and_back_cutting_both_ways() {
        let (mut w, t) = world();
        wear(&mut w, Sk::Throw, 1, Pick::A);
        for z in [-3.0, -6.5] {
            w.spawn_mob(vec3(0.0, 0.0, z));
        }
        w.spawn_mob(vec3(3.0, 0.0, -4.0));
        w.spawn_mob(vec3(0.0, 0.0, -10.5));
        still(&mut w);
        let d = Sk::Throw.def();
        let mut ev = run(&mut w, &t, skill(0), ticks(d.startup + 0.4 + 0.05));
        assert!(
            matches!(w.player.state, PState::Idle),
            "free after the recover"
        );
        assert!(w.weapon_away(), "still flying");
        // 무기가 없는 동안에도 평타.
        ev.extend(w.step(
            &Actions {
                attack: true,
                ..Default::default()
            },
            &t,
        ));
        assert!(matches!(w.player.state, PState::Attack { .. }));
        ev.extend(throw_all(&mut w, &t, Actions::default()));
        assert!(!w.weapon_away(), "back in hand");
        assert!(ev.iter().any(|e| matches!(e, Event::ThrowTurn { .. })));
        assert!(ev.iter().any(|e| matches!(e, Event::ThrowBack { .. })));
        assert_eq!(w.enemies[0].hp, 90 - 20, "out and back");
        assert_eq!(w.enemies[1].hp, 90 - 20);
        assert_eq!(w.enemies[2].hp, 90, "off the lane");
        assert_eq!(w.enemies[3].hp, 90, "beyond 8 m");
        assert_eq!(w.player.skill.gauge, 0.0, "a skill hit: no gauge");
    }

    /// 던지기 3단 ㄱ "박힌 자리": 돌아오지 않고 끝에서 터진다(둘레 2.5m · 피해 20) — 길 위의 적은 한 번만.
    #[test]
    fn throw_pick_a_sticks_at_the_far_end_and_bursts() {
        let (mut w, t) = world();
        wear(&mut w, Sk::Throw, 3, Pick::A);
        w.spawn_mob(vec3(0.0, 0.0, -3.0));
        w.spawn_mob(vec3(1.8, 0.0, -8.0)); // 끝 곁
        still(&mut w);
        let ev = throw_all(&mut w, &t, skill(0));
        assert!(ev.iter().any(|e| matches!(e, Event::ThrowBoom { .. })));
        assert!(!ev.iter().any(|e| matches!(e, Event::ThrowBack { .. })));
        assert_eq!(w.enemies[0].hp, 90 - 12, "once on the way out (rank 3)");
        assert_eq!(w.enemies[1].hp, 90 - 20, "the burst");
        assert!(!w.weapon_away());
    }

    /// 던지기 3단 ㄴ "끌고 오기": 돌아오는 길에 맞은 적이 방랑자 쪽으로 2m 끌려온다 — 보스는 아니다. 2단(고르기 없음)이면 그대로.
    #[test]
    fn throw_pick_b_drags_the_hit_back_two_metres_but_never_a_boss() {
        let pulled = |rank: u8, pick: Pick| {
            let (mut w, t) = world();
            wear(&mut w, Sk::Throw, rank, pick);
            // 보스는 끌려오는 잡몹의 길 밖·던진 길 위(뒤쪽)에.
            w.spawn_mob(vec3(0.0, 0.0, -4.5));
            w.spawn_boss(vec3(0.0, 0.0, -7.6));
            still(&mut w);
            let was: Vec<Vec3> = w.enemies.iter().map(|e| e.pos).collect();
            throw_all(&mut w, &t, skill(0));
            run(&mut w, &t, Actions::default(), 40);
            (
                flat(was[0], Vec3::ZERO) - flat(w.enemies[0].pos, Vec3::ZERO),
                flat(was[1], Vec3::ZERO) - flat(w.enemies[1].pos, Vec3::ZERO),
            )
        };
        let (mob, boss) = pulled(3, Pick::B);
        assert!((mob - 2.0).abs() < 0.5, "dragged ~2 m: {mob}");
        assert!(boss < 0.2, "a boss is not dragged: {boss}");
        let (mob, _) = pulled(2, Pick::A);
        assert!(mob < 0.5, "no pick: {mob}");
    }

    /// 던지기 5단 맺음: 돌아오는 길의 피해 ×1.5(13 → 20)·자세 피해 두 배. 4단은 오는 길도 같은 피해.
    #[test]
    fn throw_capstone_makes_the_way_back_hit_harder() {
        let legs = |rank: u8| {
            let (mut w, t) = world();
            wear(&mut w, Sk::Throw, rank, Pick::B);
            w.spawn_mob(vec3(0.0, 0.0, -4.0));
            still(&mut w);
            for e in &mut w.enemies {
                e.hp = 500;
                e.max_hp = 500;
            }
            let mut hits = Vec::new();
            let mut poise = Vec::new();
            let mut hp = 500;
            let mut last_poise = 0.0;
            let mut ev = w.step(&skill(0), &t);
            for _ in 0..300 {
                ev.extend(w.step(&Actions::default(), &t));
                let e = &w.enemies[0];
                if e.hp != hp {
                    hits.push(hp - e.hp);
                    poise.push(e.poise - last_poise);
                    hp = e.hp;
                }
                last_poise = e.poise;
            }
            (hits, poise)
        };
        let (hits, poise) = legs(MAX_RANK);
        assert_eq!(hits, [13, 20]);
        assert!((poise[1] - 2.0 * poise[0]).abs() < 1e-3, "{poise:?}");
        assert_eq!(legs(4).0, [12, 12]);
    }

    /// 잔불 흩기: 선 자리 둘레 4m에 3초 — 그 안의 적이 1초마다 10(첫 타는 1초 뒤), 밖의 적은 그대로. 잔불(재화)은 쓰지 않는다.
    /// 방랑자가 떠나도 불티는 그 자리에 남는다.
    #[test]
    fn embers_burn_everyone_inside_every_second_for_three_seconds_for_free() {
        let (mut w, t) = world();
        wear(&mut w, Sk::Embers, 1, Pick::A);
        w.spawn_mob(vec3(2.5, 0.0, 0.0));
        w.spawn_mob(vec3(0.0, 0.0, 6.5));
        still(&mut w);
        w.embers = 50;
        let d = Sk::Embers.def();
        run(&mut w, &t, skill(0), ticks(d.startup + 0.9));
        assert_eq!(w.enemies[0].hp, 90, "not yet");
        assert_eq!(w.skill_active(0) > 0.0, true, "the HUD cell shows it");
        // 떠난다.
        let away = Actions {
            move_dir: vec2(1.0, 0.0),
            ..Default::default()
        };
        let mut pulses = 0;
        for _ in 0..ticks(3.0) {
            pulses += w
                .step(&away, &t)
                .iter()
                .filter(|e| matches!(e, Event::FieldPulse { .. }))
                .count();
        }
        assert_eq!(pulses, 3);
        assert_eq!(w.enemies[0].hp, 90 - 30);
        assert_eq!(w.enemies[1].hp, 90, "outside");
        assert!(w.skill_fields.is_empty() && w.skill_active(0) == 0.0);
        assert_eq!(w.embers, 50, "costs no embers");
        assert_eq!(w.player.skill.gauge, 0.0);
    }

    /// 잔불 흩기 3단 ㄱ "따르는 불티": 불티가 방랑자를 따라와 옮겨 간 자리의 적을 태운다 — ㄴ이면 처음 자리에 남는다.
    #[test]
    fn embers_pick_a_follows_the_wanderer() {
        let burnt = |pick: Pick| {
            let (mut w, t) = world();
            wear(&mut w, Sk::Embers, 3, pick);
            w.spawn_mob(vec3(0.0, 0.0, -9.0));
            still(&mut w);
            let d = Sk::Embers.def();
            run(&mut w, &t, skill(0), ticks(d.startup + d.recover + 0.05));
            assert_eq!(w.skill_fields.len(), 1);
            // 방랑자가 적 곁으로 옮겨 간다(걸음과 상관없이 — 따라오는지만 본다).
            w.player.pos = vec3(0.0, 0.0, -7.0);
            for _ in 0..ticks(3.0) {
                w.step(&Actions::default(), &t);
                w.enemies[0].state = EState::Recover { t: -1000.0 };
            }
            90 - w.enemies[0].hp
        };
        assert_eq!(burnt(Pick::A), 36, "three pulses of 12");
        assert_eq!(burnt(Pick::B), 0);
    }

    /// 잔불 흩기 3단 ㄴ "남는 불씨": 불티 속에서 쓰러진 적이 잔불을 하나 더 남긴다 — ㄱ이면 여느 때와 같다.
    #[test]
    fn embers_pick_b_makes_the_fallen_drop_one_more_ember() {
        let dropped = |pick: Pick| {
            let (mut w, t) = world();
            wear(&mut w, Sk::Embers, 3, pick);
            w.spawn_mob(vec3(2.0, 0.0, 0.0));
            still(&mut w);
            w.enemies[0].hp = 5;
            for _ in 0..ticks(1.5) {
                w.step(&skill(0), &t);
                if !w.drops.is_empty() {
                    break;
                }
            }
            assert!(!w.enemies[0].alive());
            w.drops.iter().map(|d| d.value).sum::<u32>()
        };
        assert_eq!(dropped(Pick::B), dropped(Pick::A) + 1);
    }

    /// 잔불 흩기 5단 맺음: 1초 더(넷째 타) + 불티가 자세도 깎는다. 4단은 세 타·자세 그대로.
    #[test]
    fn embers_capstone_lasts_a_second_longer_and_wears_down_poise() {
        let burn = |rank: u8| {
            let (mut w, t) = world();
            wear(&mut w, Sk::Embers, rank, Pick::B);
            w.spawn_mob(vec3(2.0, 0.0, 0.0));
            still(&mut w);
            w.enemies[0].hp = 500;
            let ev = run(&mut w, &t, skill(0), ticks(5.5));
            let pulses = ev
                .iter()
                .filter(|e| matches!(e, Event::FieldPulse { .. }))
                .count();
            (
                pulses,
                w.enemies[0].poise > 0.0 || w.enemies[0].broken > 0.0,
            )
        };
        assert_eq!(burn(MAX_RANK), (4, true));
        assert_eq!(burn(4), (3, false));
    }

    /// 기세(0151)는 새 스킬들의 쓰러뜨림에도: 던진 무기·불티로 쓰러뜨리면 다른 스킬의 대기가 "스킬 몫"만큼 당겨진다.
    #[test]
    fn kills_by_the_new_skills_pull_cooldowns_as_skill_kills() {
        for id in [Sk::Throw, Sk::Embers] {
            let (mut w, t) = world();
            wear(&mut w, id, 1, Pick::A);
            w.spawn_mob(vec3(0.0, 0.0, -2.5));
            still(&mut w);
            w.enemies[0].hp = 5;
            w.player.skill.cd[Sk::Quake.index()] = 5.0;
            let mut ev = Vec::new();
            let mut n = 0;
            ev.extend(w.step(&skill(0), &t));
            while !ev.iter().any(|e| matches!(e, Event::EnemyDown { .. })) && n < 200 {
                ev.extend(w.step(&Actions::default(), &t));
                n += 1;
            }
            let cd = w.player.skill.cd[Sk::Quake.index()];
            let want = 5.0 - (n + 1) as f32 * TICK_DT - t[P::SkillMomentumOwn];
            assert!((cd - want).abs() < 0.02, "{id:?}: {cd} vs {want}");
        }
    }

    /// 큰 기술 몰아치기: 시동 0.3초 뒤 5초 동안 평타가 기력을 쓰지 않고 1.4배 빠르며 모든 타가 박자 타(`Event::Beat`)다.
    /// 끝나면 여느 평타(기력을 쓴다). 끼운 큰 기술이 나간다(쟁반 뒤집기가 아니다).
    #[test]
    fn flurry_makes_basic_attacks_free_fast_and_on_the_beat_for_five_seconds() {
        let swing_ticks = |w: &mut World, t: &Tuning| {
            w.step(
                &Actions {
                    attack: true,
                    ..Default::default()
                },
                t,
            );
            let mut n = 1;
            while matches!(w.player.state, PState::Attack { .. }) {
                w.step(&Actions::default(), t);
                n += 1;
            }
            n
        };
        let (mut w, t) = world();
        let plain = swing_ticks(&mut w, &t);
        let (mut w, t) = world();
        wear_ult(&mut w, Ult::Flurry, 1);
        w.player.skill.gauge = 1.0;
        let ev = run(
            &mut w,
            &t,
            Actions {
                ult: true,
                ..Default::default()
            },
            ticks(0.35),
        );
        assert!(ev.iter().any(|e| matches!(e, Event::Flurry { .. })));
        assert!(
            !ev.iter().any(|e| matches!(e, Event::Ult { .. })),
            "not Upend"
        );
        assert!((w.player.skill.flurry_left - 5.0).abs() < 0.1);
        assert_eq!(w.player.skill.gauge, 0.0);
        let stamina = w.player.stamina;
        let mut beats = 0;
        let fast = swing_ticks(&mut w, &t);
        for _ in 0..3 {
            let ev = w.step(
                &Actions {
                    attack: true,
                    ..Default::default()
                },
                &t,
            );
            beats += ev.contains(&Event::Beat) as usize;
            run(&mut w, &t, Actions::default(), 30);
        }
        assert_eq!(beats, 3, "every swing is a beat hit");
        assert_eq!(w.player.stamina, stamina, "no stamina");
        let k = plain as f32 / fast as f32;
        assert!((k - 1.4).abs() < 0.15, "1.4x faster: {plain} vs {fast}");
        assert!(w.ult_active() > 0.0);
        // 끝나면 여느 평타.
        run(&mut w, &t, Actions::default(), ticks(5.0));
        assert_eq!((w.player.skill.flurry_left, w.ult_active()), (0.0, 0.0));
        let s0 = w.player.stamina;
        swing_ticks(&mut w, &t);
        assert!(w.player.stamina < s0);
    }

    /// 몰아치기의 단: 2단 6.5초, 3단은 그동안 쓰러뜨릴 때마다 +0.5초 — 더해지는 몫은 3초까지.
    #[test]
    fn flurry_ranks_lengthen_it_and_rank_three_feeds_on_kills_up_to_three_seconds() {
        let (mut w, t) = world();
        wear_ult(&mut w, Ult::Flurry, 2);
        w.player.skill.gauge = 1.0;
        let ult = Actions {
            ult: true,
            ..Default::default()
        };
        run(&mut w, &t, ult, ticks(0.32));
        assert!((w.player.skill.flurry_total - 6.5).abs() < 1e-4);
        let down = Event::EnemyDown { at: Vec3::ZERO };
        let left = w.player.skill.flurry_left;
        w.skill_books(&t, &mut vec![down]);
        assert_eq!(w.player.skill.flurry_left, left, "rank 2 does not feed");
        let (mut w, t) = world();
        wear_ult(&mut w, Ult::Flurry, 3);
        w.player.skill.gauge = 1.0;
        run(&mut w, &t, ult, ticks(0.32));
        let left = w.player.skill.flurry_left;
        w.skill_books(&t, &mut vec![down, down]);
        assert!((w.player.skill.flurry_left - left - 1.0).abs() < 1e-4);
        for _ in 0..10 {
            w.skill_books(&t, &mut vec![down]);
        }
        assert!(
            (w.player.skill.flurry_left - left - 3.0).abs() < 1e-4,
            "capped"
        );
        assert!((w.player.skill.flurry_added - 3.0).abs() < 1e-4);
    }

    /// 큰 기술 한 줄기: 시동에 세계가 멈추고(0.3초) 앞 12m 한 줄(반 폭 1.2m)을 가른다 — 줄 위의 적은 40·자세 크게, 보스도 다치고
    /// 자세가 깎이되 휘청이지 않는다(막대가 안 깨지면). 줄 밖·12m 너머는 그대로. 2단 15m, 3단 60.
    #[test]
    fn line_cuts_one_long_lane_ahead_hurting_bosses_too() {
        let cut = |rank: u8| {
            let mut w = World::new("test", 30.0);
            let t = Tuning::default();
            w.player.facing = PI;
            wear_ult(&mut w, Ult::Line, rank);
            for z in [-3.0, -10.5, -13.5] {
                w.spawn_mob(vec3(0.0, 0.0, z));
            }
            w.spawn_mob(vec3(3.0, 0.0, -5.0));
            w.spawn_boss(vec3(0.5, 0.0, -6.5));
            still(&mut w);
            let boss = w.enemies[4].hp;
            w.player.skill.gauge = 1.0;
            w.step(
                &Actions {
                    ult: true,
                    ..Default::default()
                },
                &t,
            );
            assert!(w.hitstop >= LINE.freeze - 1e-4, "the world stops");
            let mut ev = Vec::new();
            let mut staggered = false;
            for _ in 0..120 {
                ev.extend(w.step(&Actions::default(), &t));
                staggered |= matches!(w.enemies[4].state, EState::Stagger { .. });
            }
            assert!(ev.iter().any(|e| matches!(e, Event::LineCut { .. })));
            assert!(!staggered, "a boss is not staggered");
            assert!(w.enemies[4].poise > 0.0, "but its poise is worn");
            assert_eq!(w.player.skill.gauge, 0.0);
            let lost: Vec<i32> = w.enemies[..4].iter().map(|e| 90 - e.hp).collect();
            (lost, boss - w.enemies[4].hp)
        };
        assert_eq!(cut(1), (vec![40, 40, 0, 0], 40));
        assert_eq!(cut(2), (vec![40, 40, 40, 0], 40), "15 m");
        assert_eq!(cut(3), (vec![60, 60, 60, 0], 60));
    }

    /// 끼우는 큰 기술은 하나: 게이지가 차면 끼운 것이 나간다 — 쟁반 뒤집기를 끼웠으면 몰아치기·한 줄기는 나가지 않는다.
    #[test]
    fn only_the_worn_great_skill_goes_off() {
        for (great, want) in [(Ult::Upend, 0), (Ult::Flurry, 1), (Ult::Line, 2)] {
            let (mut w, t) = world();
            wear_ult(&mut w, great, 1);
            w.spawn_mob(vec3(0.0, 0.0, -3.0));
            still(&mut w);
            w.player.skill.gauge = 1.0;
            let ev = run(
                &mut w,
                &t,
                Actions {
                    ult: true,
                    ..Default::default()
                },
                90,
            );
            let got = [
                ev.iter().any(|e| matches!(e, Event::Ult { .. })),
                ev.iter().any(|e| matches!(e, Event::Flurry { .. })),
                ev.iter().any(|e| matches!(e, Event::LineCut { .. })),
            ];
            for (i, g) in got.into_iter().enumerate() {
                assert_eq!(g, i == want, "{great:?}");
            }
        }
    }
}
