//! 스킬의 모양마다의 틱(0278 R20 — `tick_skill`의 빠짐없는 match의 갈래를 모양마다 이름 있는 메서드로 옮겼다): 메서드는 공용 값
//! `SkillCx`(이 틱의 스킬 · 값 · 시각 · 자리 · 쪽 · 타 번호)와 그 모양의 수를 받는다. 새 모양 = `tick_skill`의 match 한 줄 + 여기 메서드 하나.
use super::*;
use crate::det::Det;

impl World {
    /// 바퀴(휘돌이 · 장대 휩쓸기): 제자리에서 `hits`바퀴 — 바퀴마다 둘레를 친다.
    pub(super) fn tick_ring(
        &mut self,
        k: &mut SkillCx,
        radius: f32,
        hits: u8,
        gap: f32,
        push: f32,
    ) {
        let SkillCx {
            t,
            id,
            d,
            s,
            e1,
            at,
            dir,
            ..
        } = *k;
        let crossed = |x: f32| k.crossed(x);
        let mut swing = k.swing;
        // 몸이 판정과 함께 돈다(바퀴 수 = 타 수) — 끝나면 처음 보던 쪽.
        let span = (gap * hits as f32).max(1e-3);
        let turns = ((e1 - s.startup) / span).clamp(0.0, 1.0) * hits as f32;
        self.player.facing = dir.x.atan2_d(dir.z) + turns * std::f32::consts::TAU;
        for i in 0..hits {
            if !crossed(s.startup + gap * i as f32) {
                continue;
            }
            // 바퀴마다 새 타 번호 — 같은 적을 바퀴마다 한 번씩 친다.
            if i > 0 {
                self.player.swings += 1;
                swing = self.player.swings;
            }
            self.skill_ev.push(Event::Skill { id, at, dir });
            // 바깥으로 미는 것은 마지막 바퀴만 — 첫 바퀴가 밀어내면 둘째 바퀴가 허공을 벤다. 맺음의 휘청도 마지막 바퀴에.
            let last = i + 1 == hits;
            let lift = match d.cap {
                Some(Cap::Lift { stagger }) if last => stagger,
                _ => 0.0,
            };
            self.skill_area(
                t,
                &Sweep {
                    push: if last { push } else { 0.0 },
                    stagger: lift,
                    ..self.sweep(&s, radius, swing)
                },
            );
        }
        k.swing = swing;
    }

    /// 가로지르기 · 꿰뚫기 · 방패 돌진: 곧은 길을 지나며 틱마다 몸 둘레를 친다(움직임은 `player_tick`).
    pub(super) fn tick_lane(&mut self, k: &mut SkillCx, half: f32, time: f32, push: f32) {
        let SkillCx {
            t,
            id,
            d,
            s,
            e,
            e1,
            at,
            dir,
            ..
        } = *k;
        let crossed = |x: f32| k.crossed(x);
        let mut swing = k.swing;
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
            // 방패 돌진의 맺음(0270 C3): 밀려난 적이 휘청인다(보스는 아니다).
            let stun = d.cap_stagger();
            let n = self.skill_area(
                t,
                &Sweep {
                    push,
                    stagger: stun,
                    ..self.sweep(&s, half, swing)
                },
            );
            // 벽에 박기(방패 돌진 ㄱ): 이 길에 맞아 밀려난 적이 벽·턱에 박히면 덤 피해.
            if let (Some(Twist::Slam { damage }), true) = (d.twist, n > 0) {
                self.slam_into_walls(swing, damage);
            }
            // 맺음: 이 스킬로 쓰러뜨리면 대기의 몫을 돌려받는다(한 번 쓸 때 한 번).
            if let Some(Cap::Refund { part }) = d.cap {
                self.refund_on_kill(k, part, kills);
            }
        }
        k.swing = swing;
    }

    /// 바늘땀(0270 C3): 좁은 부채꼴을 `gap`초마다 찌른다 — 찌를 때마다 새 타 번호(같은 적을 땀마다 한 번씩).
    /// 꿰매기(맺음)는 마지막 땀에 휘청.
    pub(super) fn tick_stab(
        &mut self,
        k: &mut SkillCx,
        range: f32,
        arc_deg: f32,
        hits: u8,
        gap: f32,
    ) {
        let SkillCx {
            t, id, d, s, dir, ..
        } = *k;
        let crossed = |x: f32| k.crossed(x);
        let mut swing = k.swing;
        let half = (arc_deg * 0.5).to_radians().cos_d();
        for i in 0..hits {
            if !crossed(s.startup + gap * i as f32) {
                continue;
            }
            if i > 0 {
                self.player.swings += 1;
                swing = self.player.swings;
            }
            let at = self.player.pos;
            self.skill_ev.push(Event::Skill { id, at, dir });
            let pin = match d.cap {
                Some(Cap::Pin { stagger }) if i + 1 == hits => stagger,
                _ => 0.0,
            };
            self.skill_area(
                t,
                &Sweep {
                    cone: Some((dir, half)),
                    stagger: pin,
                    ..self.sweep(&s, range, swing)
                },
            );
        }
        k.swing = swing;
    }

    /// 종 치기(0270 C3): 앞의 가장 가까운 적 하나를 치고 종을 건다 — `delay`초 뒤 그 적이 울린다(두 번 울림 ㄱ이면 한 번 더).
    pub(super) fn tick_bell(&mut self, k: &mut SkillCx, range: f32, delay: f32, radius: f32) {
        let SkillCx {
            t,
            id,
            d,
            s,
            at,
            dir,
            ..
        } = *k;
        let crossed = |x: f32| k.crossed(x);
        if crossed(s.startup) {
            self.skill_ev.push(Event::Skill { id, at, dir });
            let target = self.bell_target(dir, range);
            let dose = self.dosed(s.dose());
            // 앞이 비면 겨눈 쪽 허공에서(벽 너머에 걸지 않는다 — 심지 볼트는 멀리 간다, 0272).
            let mut spot = self.place_ahead(at, dir, range * 0.7);
            if let Some(i) = target {
                let en = &self.enemies[i];
                spot = en.pos;
                let knock = dir * t[P::Knockback] * if en.boss { 0.25 } else { 0.6 };
                let mut ev = std::mem::take(&mut self.skill_ev);
                self.skill_touch(
                    t,
                    &mut ev,
                    i,
                    &Touch {
                        damage: s.damage,
                        poise: s.poise,
                        stagger: 0.0,
                        knock,
                        crit: true,
                        dose,
                        flat: false,
                    },
                );
                self.skill_ev = ev;
                self.hitstop = self.hitstop.max(t[P::Hitstop] * SKILL_HITSTOP);
            }
            let more = match d.cap {
                Some(Cap::Toll { damage }) => damage,
                _ => 0,
            };
            let rings = if matches!(d.twist, Some(Twist::Twice)) {
                2
            } else {
                1
            };
            for i in 1..=rings {
                self.skill_bursts.push(Burst {
                    at: spot,
                    left: delay * i as f32,
                    radius,
                    damage: s.damage + more,
                    dose,
                    kind: BurstKind::Toll { on: target },
                });
            }
        }
    }

    /// 저주 새기기(0274 ⑥): 앞의 한 적(종 치기와 같은 고르기 — 벽 너머는 아니다)에게 저주를 새긴다 — 표의 피해 · 균열, 표가
    /// `secs`초 남는다(쓰러지면 옮는다 — `after_deaths`). 앞이 비면 허공에 흩어진다.
    pub(super) fn tick_hex(&mut self, k: &mut SkillCx, range: f32, secs: f32) {
        let SkillCx {
            t, id, s, at, dir, ..
        } = *k;
        let crossed = |x: f32| k.crossed(x);
        if crossed(s.startup) {
            self.skill_ev.push(Event::Skill { id, at, dir });
            if let Some(i) = self.bell_target(dir, range) {
                let dose = self.dosed(s.dose());
                let mut ev = std::mem::take(&mut self.skill_ev);
                self.skill_touch(
                    t,
                    &mut ev,
                    i,
                    &Touch {
                        damage: s.damage,
                        poise: s.poise,
                        stagger: 0.0,
                        knock: Vec3::ZERO,
                        crit: false,
                        dose,
                        flat: false,
                    },
                );
                let en = &mut self.enemies[i];
                if en.hp > 0 {
                    en.marks.hex = en.marks.hex.max(secs);
                    ev.push(Event::Hexed {
                        at: en.pos + Vec3::Y * en.height * 0.6,
                    });
                }
                self.skill_ev = ev;
            }
        }
    }

    /// 파문(0270 C3) · 불씨 내리꽂기(0280): 물결의 앞머리가 몸에서 둘레로 번진다 — 지나는 적을 한 번씩(같은 타 번호). 밀어내는
    /// 물결(ㄱ)은 바깥으로 밀고, 갈라진 땅(ㄴ)은 물결이 지난 둘레가 남아 늦춘다, 타는 땅(ㄴ)은 둘레에 불티를 남긴다, 맺음은 휘청.
    pub(super) fn tick_wave(&mut self, k: &mut SkillCx, radius: f32, time: f32) {
        let SkillCx {
            t,
            id,
            d,
            s,
            e,
            e1,
            at,
            dir,
            ..
        } = *k;
        let crossed = |x: f32| k.crossed(x);
        let swing = k.swing;
        if crossed(s.startup) {
            self.skill_ev.push(Event::Skill { id, at, dir });
            if let Some(Twist::Crack { secs, slow }) = d.twist {
                self.skill_zones.push(Zone {
                    pos: at,
                    dir,
                    range: radius,
                    arc_deg: 360.0,
                    left: secs + time,
                    total: secs + time,
                    slow,
                });
            }
            // 타는 땅(불씨 내리꽂기 ㄴ, 0280): 내리꽂은 자리 둘레에 불티가 남는다(불길 ㄴ과 같은 불 — 반지름은 번지는 둘레의 반).
            if let Some(Twist::Scorch { secs }) = d.twist {
                self.skill_fields.push(Field::lay(
                    at,
                    radius * 0.5,
                    secs,
                    SCORCH_EVERY,
                    s.damage,
                    0.0,
                    self.dosed(s.dose()),
                ));
            }
        }
        if e1 >= s.startup && e < s.startup + time {
            let grown = ((e1 - s.startup) / time.max(1e-3)).clamp(0.0, 1.0);
            let push = match d.twist {
                Some(Twist::Shove { push }) => push,
                _ => 0.0,
            };
            let lift = match d.cap {
                Some(Cap::Lift { stagger }) => stagger,
                _ => 0.0,
            };
            self.skill_area(
                t,
                &Sweep {
                    push,
                    stagger: lift,
                    ..self.sweep(&s, radius * grown, swing)
                },
            );
        }
    }

    /// 끌어모으기: 앞 부채꼴의 적을 한 점으로 모은다(`skill_pull`) — 모인 자리(ㄱ)는 곧이어 터진다.
    pub(super) fn tick_pull(
        &mut self,
        k: &mut SkillCx,
        range: f32,
        arc_deg: f32,
        to: f32,
        stagger: f32,
    ) {
        let SkillCx { d, s, .. } = *k;
        let crossed = |x: f32| k.crossed(x);
        if crossed(s.startup) {
            let spot = self.skill_pull(k, range, arc_deg, to, stagger);
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
                    dose: NO_DOSE,
                    kind: BurstKind::Blast,
                });
            }
        }
    }

    /// 부채꼴(땅울림 · 후려치기 · 불길 · 밀치기 …): 앞 부채꼴을 한 번 친다 — 남는 것(불티 · 갈라진 땅)과 맺음의 표지는 고르기대로.
    pub(super) fn tick_cone(
        &mut self,
        k: &mut SkillCx,
        range: f32,
        arc_deg: f32,
        stagger: f32,
        push: f32,
    ) {
        let SkillCx {
            t,
            id,
            d,
            s,
            at,
            dir,
            ..
        } = *k;
        let crossed = |x: f32| k.crossed(x);
        let swing = k.swing;
        if crossed(s.startup) {
            self.skill_ev.push(Event::Skill { id, at, dir });
            let half = (arc_deg * 0.5).to_radians().cos_d();
            let shatter = match d.cap {
                Some(Cap::Shatter { mult }) => mult,
                _ => 1.0,
            };
            let n = self.skill_area(
                t,
                &Sweep {
                    cone: Some((dir, half)),
                    push,
                    stagger,
                    shatter,
                    ..self.sweep(&s, range, swing)
                },
            );
            // 벽에 박기(후려치기 ㄱ, 0270 C3).
            if let (Some(Twist::Slam { damage }), true) = (d.twist, n > 0) {
                self.slam_into_walls(swing, damage);
            }
            // 불길의 맺음(0270 C3): 불에 닿은 적이 잠깐 더 다친다(보스는 아니다 — 끌어모으기의 맺음과 같은 표지).
            if let Some(Cap::Frail { secs, mult }) = d.cap {
                for en in &mut self.enemies {
                    if en.hit_by == swing && en.alive() && !en.boss {
                        en.marks.mark_frail(secs, mult);
                    }
                }
            }
            // 타는 땅(불길 ㄴ): 부채꼴 가운데에 불티가 남는다.
            if let Some(Twist::Scorch { secs }) = d.twist {
                self.skill_fields.push(Field::lay(
                    at + dir * range * 0.5,
                    range * 0.45,
                    secs,
                    SCORCH_EVERY,
                    s.damage,
                    0.0,
                    self.dosed(s.dose()),
                ));
            }
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

    /// 제비걸음(0214): 걸음마다 가장 가까운 아직 안 벤 적(한 곳만 = 첫 적)에게 뛰어들어(`gap`의 앞 6할) 닿은 자리 둘레를 벤다.
    /// 돌아오는 걸음(ㄴ)은 다 벤 뒤 한 걸음 더 — 처음 자리로. 마지막 걸음(맺음)은 넓게·자세를 더.
    pub(super) fn tick_hop(&mut self, k: &mut SkillCx, range: f32, hops: u8, gap: f32, reach: f32) {
        let SkillCx {
            t, id, d, s, dir, ..
        } = *k;
        let crossed = |x: f32| k.crossed(x);
        let mut swing = k.swing;
        let home = matches!(d.twist, Some(Twist::HomeStep));
        let steps = hops + home as u8;
        for i in 0..steps {
            let start = s.startup + gap * i as f32;
            if crossed(start) {
                let back = i == hops;
                let to = if back {
                    self.player.skill.hop_home
                } else {
                    self.hop_target(range, matches!(d.twist, Some(Twist::Focus)), dir)
                };
                let pl = &mut self.player;
                let way = vec3(to.x - pl.pos.x, 0.0, to.z - pl.pos.z);
                if way.length_squared() > 1e-4 {
                    pl.facing = way.x.atan2_d(way.z);
                }
                pl.skill.hop_from = pl.pos;
                pl.skill.hop_to = vec3(to.x, pl.pos.y, to.z);
                if !back {
                    self.skill_ev.push(Event::Skill {
                        id,
                        at: self.player.pos,
                        dir: way.normalize_or(dir),
                    });
                }
            }
            if i < hops && crossed(start + gap * HOP_DASH) {
                // 걸음마다 새 타 번호 — 같은 적도 걸음마다 한 번씩.
                if i > 0 {
                    self.player.swings += 1;
                    swing = self.player.swings;
                }
                let last = i + 1 == hops;
                let (r, pk) = match d.cap {
                    Some(Cap::Finale { reach, poise }) if last => (reach, poise),
                    _ => (reach, 1.0),
                };
                self.skill_area(
                    t,
                    &Sweep {
                        poise: s.poise * pk,
                        ..self.sweep(&s, r, swing)
                    },
                );
            }
        }
        k.swing = swing;
    }

    /// 재기(0274): 시동이 끝나는 순간 체력이 돈다(바닥에서 ㄴ이면 곱절, 맺음이면 더) — 고른 숨 ㄱ이면 기력도. 곁의 적은
    /// 피해 없이 밀쳐 내며 흡수를 건다(보스는 밀리지 않는다 — 걸리기만).
    pub(super) fn tick_rally(&mut self, k: &mut SkillCx, heal: i32, radius: f32, push: f32) {
        let SkillCx {
            t,
            id,
            d,
            s,
            at,
            dir,
            ..
        } = *k;
        let crossed = |x: f32| k.crossed(x);
        if crossed(s.startup) {
            self.skill_ev.push(Event::Skill { id, at, dir });
            let max_st = t[P::StaminaMax];
            let pl = &mut self.player;
            let low = match d.twist {
                Some(Twist::Low { below, mult }) if (pl.hp as f32) < below * pl.max_hp as f32 => {
                    mult
                }
                _ => 1.0,
            };
            let gain = skills::scaled(heal, low);
            pl.hp = (pl.hp + gain).min(pl.max_hp);
            if let Some(Twist::Breath { stamina }) = d.twist {
                pl.stamina = (pl.stamina + stamina).min(max_st);
            }
            self.skill_ev.push(Event::Mend {
                at: pl.pos + Vec3::Y * 1.0,
            });
            // 맺음: 밀쳐 낸 적이 잠깐 휘청인다(보스는 아니다).
            let stun = d.cap_stagger();
            self.skill_rally(t, radius, push, stun, self.dosed(s.dose()));
        }
    }

    /// 막아치기(0214): 앞 부채꼴을 밀어친다 — 공격을 준비하던 적은 그 공격이 끊긴다(`skill_bash`).
    pub(super) fn tick_bash(
        &mut self,
        k: &mut SkillCx,
        range: f32,
        arc_deg: f32,
        stagger: f32,
        mult: f32,
    ) {
        let SkillCx { id, s, at, dir, .. } = *k;
        let crossed = |x: f32| k.crossed(x);
        if crossed(s.startup) {
            self.skill_ev.push(Event::Skill { id, at, dir });
            self.skill_bash(k, range, arc_deg, stagger, mult);
        }
    }

    /// 생기 거두기(0274 ⑤): 날을 몸 둘레로 한 바퀴 두르고(`time`초 — 몸의 한 바퀴는 전용 동작 `sk_harvest`가 담는다: 월드의
    /// 쪽은 돌리지 않는다) 끝에 둘레를 모두 벤다 — 거둔 적마다 체력이 돈다(`most`마리까지, 바닥에서 ㄴ이면 곱절 — 받는 피해는
    /// 그대로다), 흡수를 건다. 맺음이면 거둔 적이 휘청인다(보스는 아니다 — `skill_area`).
    pub(super) fn tick_reap(
        &mut self,
        k: &mut SkillCx,
        radius: f32,
        heal: i32,
        most: u8,
        time: f32,
    ) {
        let SkillCx {
            t,
            id,
            d,
            s,
            at,
            dir,
            ..
        } = *k;
        let crossed = |x: f32| k.crossed(x);
        let swing = k.swing;
        if crossed(s.startup + time) {
            self.skill_ev.push(Event::Skill { id, at, dir });
            let stun = d.cap_stagger();
            let n = self.skill_area(
                t,
                &Sweep {
                    stagger: stun,
                    ..self.sweep(&s, radius, swing)
                },
            );
            if n > 0 {
                let pl = &mut self.player;
                let low = match d.twist {
                    Some(Twist::Low { below, mult })
                        if (pl.hp as f32) < below * pl.max_hp as f32 =>
                    {
                        mult
                    }
                    _ => 1.0,
                };
                let gain = skills::scaled(heal * n.min(most as u32) as i32, low);
                pl.hp = (pl.hp + gain).min(pl.max_hp);
                self.skill_ev.push(Event::Mend {
                    at: pl.pos + Vec3::Y * 1.0,
                });
            }
        }
    }

    /// 빠지며 베기(0274 ③): 시동 끝에 앞 부채꼴을 한 번 벤다 — 물러나는 걸음은 플레이어 틱(`player_tick`)이 `time`초 동안.
    /// 벌어진 틈(ㄴ)이면 베인 적이 한동안 더 다치고(보스는 아니다), 최종 효과면 이 스킬로 쓰러뜨린 때 쿨타임의 몫을 돌려받는다.
    pub(super) fn tick_slip(&mut self, k: &mut SkillCx, range: f32, arc_deg: f32) {
        let SkillCx {
            t,
            id,
            d,
            s,
            at,
            dir,
            ..
        } = *k;
        let crossed = |x: f32| k.crossed(x);
        let swing = k.swing;
        if crossed(s.startup) {
            self.skill_ev.push(Event::Skill { id, at, dir });
            let kills = self.kills;
            let half = (arc_deg * 0.5).to_radians().cos_d();
            self.skill_area(
                t,
                &Sweep {
                    cone: Some((dir, half)),
                    ..self.sweep(&s, range, swing)
                },
            );
            if let Some(Twist::Expose { secs, mult }) = d.twist {
                for en in &mut self.enemies {
                    if en.hit_by == swing && en.alive() && !en.boss {
                        en.marks.mark_frail(secs, mult);
                    }
                }
            }
            if let Some(Cap::Refund { part }) = d.cap {
                self.refund_on_kill(k, part, kills);
            }
        }
    }

    /// 뿌리 솟기(0214): 뿌리가 `every`초마다 하나씩 솟는다 — 앞으로 줄지어(또는 ㄱ 둘레로 돌며). 한 번 쓸 때 적마다 한 번.
    pub(super) fn tick_erupt(
        &mut self,
        k: &mut SkillCx,
        len: f32,
        step: f32,
        every: f32,
        radius: f32,
        stagger: f32,
    ) {
        let SkillCx {
            id, d, s, at, dir, ..
        } = *k;
        let crossed = |x: f32| k.crossed(x);
        let n = skills::erupt_count(len, step);
        let circle = matches!(d.twist, Some(Twist::Circle));
        let bind = match d.twist {
            Some(Twist::Bind { secs }) => secs,
            _ => 0.0,
        };
        for i in 0..n {
            if !crossed(s.startup + every * i as f32) {
                continue;
            }
            let spot = if circle {
                let a = dir.x.atan2_d(dir.z) + i as f32 / n as f32 * std::f32::consts::TAU;
                at + vec3(a.sin_d(), 0.0, a.cos_d()) * (len * 0.5).min(ERUPT_RING_MAX)
            } else {
                at + dir * (step * (i + 1) as f32)
            };
            if i == 0 {
                self.skill_ev.push(Event::Skill { id, at, dir });
            }
            self.skill_root(k, spot, radius, stagger, bind, i);
            if i + 1 == n {
                if let Some(Cap::Spire { radius, damage }) = d.cap {
                    self.skill_bursts.push(Burst {
                        at: spot,
                        left: SPIRE_DELAY,
                        radius,
                        damage,
                        dose: NO_DOSE,
                        kind: BurstKind::Blast,
                    });
                }
            }
        }
    }

    /// 버팀(G2): 버티는 동안은 표지만 적는다(덜 다치고·밀리지 않고·끊기지 않는 것은 맞음 길이 한다) — 회피는 터뜨림 없이 끊는다.
    /// 끝에 둘레를 터뜨린다: 표의 피해 + 받아낸 타마다 덤.
    pub(super) fn tick_brace(&mut self, k: &mut SkillCx) -> Flow {
        // 칸이 여섯이라(인자가 여덟이 된다) 모양에서 다시 푼다 — `tick_skill`이 버팀일 때만 부른다.
        let Form::Brace {
            hold,
            radius,
            soak,
            most,
            more,
            push,
        } = k.s.form
        else {
            unreachable!("tick_brace on {:?}", k.s.form)
        };
        let SkillCx {
            cx,
            t,
            id,
            d,
            s,
            e1,
            at,
            dir,
            ..
        } = *k;
        let crossed = |x: f32| k.crossed(x);
        let mut swing = k.swing;
        let end = s.startup + hold;
        if e1 >= s.startup && e1 < end {
            if cx.a.dodge && self.try_dodge(cx) {
                return Flow::Left;
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
            let stun = d.cap_stagger();
            self.skill_area(
                t,
                &Sweep {
                    push,
                    stagger: stun,
                    damage: s.damage + more * n,
                    ..self.sweep(&s, radius, swing)
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
        k.swing = swing;
        Flow::Go
    }

    /// 되울림(G2): 시동 뒤 몸에 울림을 두른다 — 그동안의 패링은 `echo_ring`이 둘레로 퍼뜨린다.
    pub(super) fn tick_echo(&mut self, k: &mut SkillCx, secs: f32) {
        let SkillCx { id, s, at, dir, .. } = *k;
        let crossed = |x: f32| k.crossed(x);
        if crossed(s.startup) {
            let pl = &mut self.player;
            pl.skill.echo_left = secs;
            pl.skill.echo_total = secs;
            self.skill_ev.push(Event::Skill { id, at, dir });
        }
    }

    /// 던지기(G2): 시동 끝에 손을 떠난다 — 그 뒤는 `tick_skill_fields`가 날린다(몸은 후딜 뒤 자유).
    pub(super) fn tick_throw_skill(&mut self, k: &mut SkillCx, dist: f32, half: f32, speed: f32) {
        let SkillCx {
            id, d, s, at, dir, ..
        } = *k;
        let crossed = |x: f32| k.crossed(x);
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
                dose: self.dosed(s.dose()),
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

    /// 눈발(0270 C3): 겨눈 쪽 앞에 눈발을 뿌린다 — 치는 것은 `tick_fields`(잔불 흩기의 불티와 같은 길). 흐르는 눈발(ㄱ)은
    /// 겨눈 쪽으로 흘러간다.
    pub(super) fn tick_drift(
        &mut self,
        k: &mut SkillCx,
        ahead: f32,
        radius: f32,
        secs: f32,
        every: f32,
    ) {
        let SkillCx {
            id, d, s, at, dir, ..
        } = *k;
        let crossed = |x: f32| k.crossed(x);
        if crossed(s.startup) {
            let pos = self.place_ahead(at, dir, ahead);
            let smolder = match d.cap {
                Some(Cap::Smolder { poise, .. }) => poise,
                _ => 0.0,
            };
            self.skill_fields.push(Field {
                kind: FieldKind::Drift {
                    flow: match d.twist {
                        Some(Twist::Flow { speed }) => dir * speed,
                        _ => Vec3::ZERO,
                    },
                },
                ..Field::lay(
                    pos,
                    radius,
                    secs,
                    every,
                    s.damage,
                    smolder,
                    self.dosed(s.dose()),
                )
            });
            self.skill_ev.push(Event::Skill { id, at: pos, dir });
        }
    }

    /// 뿌리 덫(0270 C3): 겨눈 쪽 앞에 씨를 심는다(두 알 ㄱ이면 양옆에 둘) — 적이 들어서면 `tick_fields`가 한 번 솟게 한다.
    pub(super) fn tick_trap(&mut self, k: &mut SkillCx, ahead: f32, radius: f32, secs: f32) {
        let SkillCx {
            id, d, s, at, dir, ..
        } = *k;
        let crossed = |x: f32| k.crossed(x);
        if crossed(s.startup) {
            let pos = self.place_ahead(at, dir, ahead);
            let side = vec3(dir.z, 0.0, -dir.x) * TRAP_TWIN_GAP;
            let spots: Vec<Vec3> = if matches!(d.twist, Some(Twist::Twin)) {
                vec![pos + side, pos - side]
            } else {
                vec![pos]
            };
            for p in spots {
                self.skill_fields.push(Field {
                    kind: FieldKind::Trap {
                        iron: false,
                        spire: match d.cap {
                            Some(Cap::Spire { radius, damage }) => Some((radius, damage)),
                            _ => None,
                        },
                    },
                    ..Field::lay(
                        p,
                        radius,
                        secs,
                        secs,
                        s.damage,
                        s.poise,
                        self.dosed(s.dose()),
                    )
                });
            }
            self.skill_ev.push(Event::Skill { id, at: pos, dir });
        }
    }

    /// 잔불 흩기(G2): 선 자리에 불티가 남는다 — 태우는 것은 `tick_skill_fields`.
    pub(super) fn tick_field(&mut self, k: &mut SkillCx, radius: f32, secs: f32, every: f32) {
        let SkillCx {
            id, d, s, at, dir, ..
        } = *k;
        let crossed = |x: f32| k.crossed(x);
        if crossed(s.startup) {
            let smolder = match d.cap {
                Some(Cap::Smolder { poise, .. }) => poise,
                _ => 0.0,
            };
            self.skill_fields.push(Field {
                kind: FieldKind::Embers {
                    follow: matches!(d.twist, Some(Twist::Follow)),
                    kindle: match d.twist {
                        Some(Twist::Kindle { embers }) => embers,
                        _ => 0,
                    },
                },
                ..Field::lay(
                    at,
                    radius,
                    secs,
                    every,
                    s.damage,
                    smolder,
                    self.dosed(s.dose()),
                )
            });
            self.skill_ev.push(Event::Skill { id, at, dir });
        }
    }
}
