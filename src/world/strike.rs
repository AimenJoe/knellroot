//! 플레이어 스윙의 판정(0159 R2): 한 함수였던 `apply_strike`(275줄)를 단계마다 메서드로 — 순서와 몸은 그대로다.
//!
//! 순서: 내려찍기·늦은 고리 → 박자의 충격 → 틱이 남긴 알림 → 지금의 타(`Strike`) → 활성 구간의 첫 틱 → **맞힘**(`strike_lands`) → 튕김·히트스톱·나무 칸.
use super::*;

/// 한 스윙이 이번 틱에 맞힌 결과.
struct Landed {
    /// 하나라도 맞혔나(히트스톱·내딛기 멈춤).
    stop: bool,
    heavy_hit: bool,
    /// 처음 맞은 적의 자리·반지름·친 쪽(나무 칸).
    first_hit: Option<(Vec3, f32, Vec3)>,
    /// 등딱지에 튕긴 자리(0120).
    deflected: Option<Vec3>,
}

impl World {
    /// 플레이어 스윙의 활성 구간에 범위·부채꼴 안의 적을 한 스윙에 한 번씩 때린다.
    pub(super) fn apply_strike(&mut self, t: &Tuning, ev: &mut Vec<Event>) {
        self.plunge_and_late_ring(t, ev);
        self.flow_shock(t, ev);
        if std::mem::take(&mut self.charged_now) {
            ev.push(Event::Charged);
        }
        if std::mem::take(&mut self.gulp_now) {
            ev.push(Event::Gulp {
                at: self.player.pos + Vec3::Y * 1.2,
            });
        }
        if std::mem::take(&mut self.flask_denied_now) {
            ev.push(Event::FlaskDenied);
        }
        if std::mem::take(&mut self.prayer_now) {
            self.prayer_ring(t, ev);
        }
        if let PState::Rush {
            t: e,
            dir,
            power,
            swing,
        } = self.player.state
        {
            if e < crate::tuning::RUSH_TIME {
                self.rush_lands(t, ev, dir, power, swing);
            }
            return;
        }
        // 평타와 강타(0030)는 같은 규칙에 값만 다르다 — 무기 표의 타(0051)가 랩 기준값에 곱한다.
        let wpn = self.player.weapon.def();
        let s = match self.player.state {
            PState::Attack {
                t: e, swing, hit, ..
            } => {
                // 빈 손의 결(0276)까지 든 타 — 그림과 같은 셈(`World::hit_of`).
                let Some(h) = self.hit_of(hit) else {
                    return;
                };
                let (s0, a0, _) = hit_times(t, wpn, hit);
                Strike {
                    e,
                    swing,
                    startup: s0,
                    active: a0,
                    range: t[P::AttackRange] * h.reach,
                    arc: t[P::AttackArc] * h.arc,
                    circle: h.shape == Shape::Circle,
                    lane: h.shape.lane_half(),
                    damage: h.damage,
                    poise: h.poise,
                    knock: h.knock,
                    heavy: None,
                    perfect: false,
                    beat: self.player.beat.swing_beat,
                    dash: hit == weapons::DASH,
                    cleave: h.cleave,
                    pull: h.pull,
                    dose: h.dose,
                }
            }
            PState::Heavy {
                t: e,
                swing,
                full,
                perfect,
                ..
            } => {
                let Some(h) = self.heavy_of() else {
                    return;
                };
                let (s0, a0, _) = heavy_times(t, wpn);
                Strike {
                    e,
                    swing,
                    startup: s0,
                    active: a0,
                    range: t[P::HeavyRange] * h.reach,
                    arc: t[P::HeavyArc] * h.arc,
                    circle: h.shape == Shape::Circle,
                    lane: h.shape.lane_half(),
                    damage: h.damage
                        + if full {
                            t[P::HeavyFullBonus].round() as i32
                        } else {
                            0
                        }
                        // 다 모은 강공격의 덤(0274 ④ — 대검을 멘 보조특성): 쥐어 모은 것만, 패링 반격은 아니다.
                        + if full && swing != self.player.parry.riposte_swing {
                            self.stats.charged_dmg
                        } else {
                            0
                        }
                        + perfect as i32 * HP_UNIT,
                    poise: h.poise * if perfect { 1.5 } else { 1.0 },
                    knock: t[P::HeavyKnock] * if perfect { 1.5 } else { 1.0 },
                    heavy: Some(full),
                    perfect,
                    // 받아치는 박(0215 다리): 박자로 친 반격(`swing_pays`가 그 스윙에 표시했다).
                    beat: swing == self.player.parry.riposte_swing && self.player.beat.swing_beat,
                    dash: false,
                    cleave: 0,
                    pull: h.pull,
                    // 덜 모은 강공격은 반(0274 ⑥ — 완드의 쥐었다 터뜨리는 저주).
                    dose: if full {
                        h.dose
                    } else {
                        (h.dose.0, h.dose.1 * 0.5)
                    },
                }
            }
            _ => return,
        };
        let (e, swing) = (s.e, s.swing);
        if e < s.startup || e >= s.startup + s.active {
            return;
        }
        self.swing_begins(e, &s, wpn, ev);
        let Landed {
            stop,
            heavy_hit,
            first_hit,
            deflected,
        } = self.strike_lands(t, ev, &s, swing, wpn);
        if let (Some(at), false) = (deflected, stop) {
            self.recoil(at, ev);
            return;
        }
        if stop {
            self.hitstop = t[P::Hitstop]
                * if heavy_hit {
                    HEAVY_HITSTOP
                } else {
                    HIT_HITSTOP
                };
            // 공격자 확인(KB, Hollow Knight 반동): 맞힌 스윙은 남은 내딛기를 멈춘다 — 빗나감과 맞힘이 손에서 다르게.
            self.player.connected = swing;
            self.strike_cells(t, &s, first_hit, ev);
        }
    }

    /// 기도하는 손(0276 — 망치의 빈 손): 한 모금을 삼킨 순간 둘레 `PRAYER_RADIUS` m의 적에게 울림 게이지(`PRAYER_KNELL`)를 쌓고
    /// 바깥으로 민다(문지기는 4분의 1). 피해는 없다 — 울림이 차면 울림의 일(둘레 피해)은 `tick_ails`가 한다. 회복은 그대로.
    fn prayer_ring(&mut self, t: &Tuning, ev: &mut Vec<Event>) {
        use crate::tuning::{PRAYER_KNELL, PRAYER_KNOCK, PRAYER_RADIUS};
        let p = self.player.pos;
        for en in &mut self.enemies {
            if !en.active() || (en.pos.y - p.y).abs() > HIT_DY {
                continue;
            }
            let d = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z);
            if d.length() - en.radius > PRAYER_RADIUS {
                continue;
            }
            let out = d.normalize_or(Vec3::Z);
            en.knock = out * t[P::Knockback] * PRAYER_KNOCK * if en.boss { 0.25 } else { 1.0 };
            afflict(en, (crate::status::Kind::Knell, PRAYER_KNELL), ev);
        }
        ev.push(Event::Prayer {
            at: p,
            radius: PRAYER_RADIUS,
        });
    }

    /// 들이받기의 타(0276 — 하데스의 방패): 내달리는 동안 몸 앞의 적을 한 번씩 — 모인 힘(`power`)만큼 피해 · 자세 피해. 잡몹은 밀려
    /// 넘어지고(휘청 + 크게 밀림), 문지기는 자세만 깎이고 거기서 멈춘다(`connected`). 등딱지도 들이받는다(방패로 치는 것이라 튕기지 않는다).
    fn rush_lands(&mut self, t: &Tuning, ev: &mut Vec<Event>, dir: Vec3, power: f32, swing: u32) {
        use crate::tuning::{RUSH_DAMAGE, RUSH_KNOCK, RUSH_POISE, RUSH_REACH, RUSH_TOPPLE};
        let lerp = |a: f32, b: f32| a + (b - a) * power;
        let damage = lerp(RUSH_DAMAGE.0 as f32, RUSH_DAMAGE.1 as f32).round() as i32;
        let poise = lerp(RUSH_POISE.0, RUSH_POISE.1);
        let topple = lerp(RUSH_TOPPLE.0, RUSH_TOPPLE.1);
        let knock = lerp(RUSH_KNOCK.0, RUSH_KNOCK.1);
        let p = self.player.pos;
        let body = p.y + self.player.air;
        let knack = self.knack();
        let mut hit_any = false;
        let mut boss_hit = false;
        for en in &mut self.enemies {
            if !en.active() || en.hit_by == swing || (en.pos.y - body).abs() > HIT_DY {
                continue;
            }
            let d = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z);
            let n = d.length();
            if n - en.radius - PLAYER_RADIUS > RUSH_REACH || (n > 1e-3 && d.dot(dir) / n < 0.2) {
                continue;
            }
            en.hit_by = swing;
            hit_any = true;
            let crit = en.exposed > 0.0;
            en.hp -= wound(
                en,
                self.stats.dmg + self.stats.heavy_dmg + damage,
                crit,
                self.stats.opened_dmg,
                knack,
                t,
                ev,
            );
            if crit {
                en.exposed = 0.0;
            }
            en.struck();
            let at = en.pos + Vec3::Y * en.height * 0.6;
            ev.push(if crit {
                Event::Crit { at }
            } else {
                Event::Smash { at }
            });
            // 옆으로 비켜 밀린다(가는 쪽 + 몸 쪽) — 길을 연다.
            let side = (d - dir * d.dot(dir)).normalize_or_zero();
            if en.boss {
                boss_hit = true;
            } else {
                en.knock = (dir + side * 0.6).normalize_or(dir) * t[P::Knockback] * knock;
                if en.hp > 0 {
                    en.state = EState::Stagger {
                        t: 0.0,
                        dur: topple,
                    };
                }
            }
            add_poise(en, poise * self.stats.poise, None, t, ev);
            if en.hp <= 0 {
                en.state = EState::Dead;
                self.kills += 1;
                ev.push(Event::EnemyDown { at: en.pos });
            }
        }
        if hit_any {
            self.hitstop = t[P::Hitstop] * HIT_HITSTOP;
            self.player.connected = if boss_hit {
                swing
            } else {
                self.player.connected
            };
            self.bolt_hit_tick = true;
        }
    }

    /// 내려찍기의 착지(0027)와 착지의 순간(0107)의 늦은 바깥 고리.
    fn plunge_and_late_ring(&mut self, t: &Tuning, ev: &mut Vec<Event>) {
        if std::mem::take(&mut self.plunge_now) {
            self.plunge(t, ev);
            // 착지의 순간(0107): 0.3초 뒤 바깥 고리 한 번 더.
            if self.tree.has(Cell::LandingBeat) {
                self.late_ring = Some(cell::LANDING_DELAY);
            }
        }
        if let Some(left) = self.late_ring {
            if left <= TICK_DT {
                self.late_ring = None;
                self.outer_ring(t, ev);
            } else {
                self.late_ring = Some(left - TICK_DT);
            }
        }
    }

    fn flow_shock(&mut self, t: &Tuning, ev: &mut Vec<Event>) {
        // 이어지는 박자(0107): 박자 셋째 타가 나가는 순간 둘레에 작은 충격(자세 피해만).
        if std::mem::take(&mut self.shock_now) {
            let p = self.player.pos;
            for en in &mut self.enemies {
                let d = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z);
                if en.active() && d.length() - en.radius <= cell::FLOW_RADIUS {
                    add_poise(en, cell::FLOW_POISE * self.stats.poise, None, t, ev);
                }
            }
            ev.push(Event::Plunge { at: p });
        }
    }

    /// 활성 구간의 첫 틱: 휘두름 사건, 강타면 금 간 땅(망치)·딱 맞춤 번쩍.
    fn swing_begins(&mut self, e: f32, s: &Strike, wpn: &weapons::Weapon, ev: &mut Vec<Event>) {
        if e < s.startup + TICK_DT {
            ev.push(Event::Swing);
            if s.heavy.is_some() {
                // 망치 강타(0051): 친 자리에 금이 가고 늦게 여진, 지팡이는 딱 맞춤 번쩍.
                if let Some((delay, radius, damage, poise)) = wpn.heavy.and_then(|h| h.aftershock) {
                    self.cracks.push(Crack {
                        pos: self.player.pos,
                        left: delay,
                        radius,
                        damage,
                        poise,
                    });
                }
                if s.perfect {
                    self.player.beat.since_perfect = 0.0;
                    ev.push(Event::Perfect {
                        at: self.player.pos + Vec3::Y * 1.0,
                    });
                }
            }
        }
    }

    /// 범위·부채꼴 안의 적을 한 스윙에 한 번씩 때린다(등딱지는 튕김): 피해·치명타·밀림·불똥·자세(나무 칸의 배)·강타의 휘청. 돌려주는 것은 맞힘의 결과(`Landed`).
    fn strike_lands(
        &mut self,
        t: &Tuning,
        ev: &mut Vec<Event>,
        s: &Strike,
        swing: u32,
        wpn: &weapons::Weapon,
    ) -> Landed {
        let primed = if s.heavy.is_some() {
            wpn.heavy.and_then(|h| h.breaks_primed)
        } else {
            None
        };
        let p = self.player.pos;
        let body = p.y + self.player.air;
        let f = self.player.facing_dir();
        let half = (s.arc * 0.5).to_radians().cos();
        let mut stop = false;
        let mut heavy_hit = false;
        let mut first_hit: Option<(Vec3, f32, Vec3)> = None;
        let measure = self.tree.keyed(crate::progression::Key::Measure);
        let rooted_riposte = self.tree.linked(Link::RootedRiposte);
        let breaking_beat = self.tree.linked(Link::BreakingBeat);
        let mut deflected: Option<Vec3> = None;
        // 결(0223 11절): 이 휘두르기가 묶인 적·금 간 적에 닿았나(맞기 전의 상태로 본다).
        let (mut bound_hit, mut cracked_hit) = (false, false);
        // 0270 C3: 기절한 적(그 자리) · 피 흘리는 적 · 흡수 걸린 적.
        let (mut stunned_at, mut bled_hit, mut sipped_hit) = (None, false, false);
        let two_hands = self.bare() == Some(weapons::Bare::TwoHands);
        // 그림자 걸음(0276 — 쌍단검의 빈 손): 회피가 끝난 뒤 첫 휘두르기는 급소로 친다(`Knack::sure`).
        let knack = Knack {
            sure: swing == self.player.off.shadow_swing
                && self.bare() == Some(weapons::Bare::ShadowStep),
            ..self.knack()
        };
        // 곧은 길(창): 벽·높은 턱에 막힌 곳까지만 닿는다 — 그 너머의 적은 꿰뚫지 않는다.
        let lane_len = s
            .lane
            .map(|_| self.lane_reach(p + Vec3::Y * self.player.air, f, s.range));
        // 쪼개기(0274 ④ — 대검): 한 휘두르기에 앞의 `cleave`까지 — 가까운 적부터 센다(틱을 넘어 `Player::cleaved`에).
        let mut order: Vec<usize> = (0..self.enemies.len()).collect();
        if s.cleave > 0 {
            if self.player.cleaved.0 != swing {
                self.player.cleaved = (swing, 0);
            }
            let dist = |i: usize| {
                let q = self.enemies[i].pos;
                vec3(q.x - p.x, 0.0, q.z - p.z).length() - self.enemies[i].radius
            };
            order.sort_by(|&a, &b| dist(a).total_cmp(&dist(b)));
        }
        for i in order {
            let en = &mut self.enemies[i];
            if !en.active() || en.hit_by == swing {
                continue;
            }
            // 높이가 다른 층의 적에게는 칼이 안 닿는다(0043).
            if (en.pos.y - body).abs() > HIT_DY {
                continue;
            }
            let d = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z);
            let dir = if d.length() > 1e-4 { d.normalize() } else { f };
            if let (Some(w), Some(len)) = (s.lane, lane_len) {
                // 길 위(앞으로 몸이 걸치고 · 옆으로 반 폭 + 몸 안)의 적을 모두.
                let along = d.dot(f);
                let off = (d - f * along).length();
                if along + en.radius < 0.0 || along - en.radius > len || off > w + en.radius {
                    continue;
                }
            } else {
                let dist = d.length() - en.radius;
                if dist > s.range {
                    continue;
                }
                if !s.circle && dir.dot(f) < half {
                    continue;
                }
            }
            if s.cleave > 0 {
                if self.player.cleaved.1 >= s.cleave {
                    continue;
                }
                self.player.cleaved.1 += 1;
            }
            en.hit_by = swing;
            // 등딱지(0120): 앞에서 든 평타는 튕긴다 — 피해·자세 없음. 강타·옆/뒤·내려찍기(다른 길)는 들어간다.
            if s.heavy.is_none() && en.guards(-dir) {
                deflected.get_or_insert(en.pos + Vec3::Y * en.height * 0.6);
                continue;
            }
            // 전환 중인 보스(강타는 닿아도 피해가 없다)는 세지 않는다.
            if !en.rising() {
                bound_hit |= en.ail.bound > 0.0 || en.marks.bind > 0.0;
                cracked_hit |= en.ail.rift > 0.0;
                if en.ail.stun > 0.0 && stunned_at.is_none() {
                    stunned_at = Some(en.pos);
                }
                bled_hit |= en.ail.bleed > 0.0;
                sipped_hit |= en.ail.drain > 0.0;
            }
            // 패링으로 드러난 적은 한 대가 치명타(0025).
            let crit = en.exposed > 0.0;
            // 끊는 박(0215 다리): 휘청이거나 드러난 적 — 치명타가 드러남을 지우기 전에 본다.
            let opened = crit || matches!(en.state, EState::Stagger { .. });
            // 레벨 힘(0104): 강타·패링 반격 +`heavy_dmg`, 콤보·대시 공격 +`light_dmg` — 장신구 `dmg`는 모든 타.
            let lv = if s.heavy.is_some() {
                self.stats.heavy_dmg
            } else {
                self.stats.light_dmg
            };
            // 박자의 사람(0214 맺음): 평타(강타·대시 공격이 아닌 콤보)는 박자에 ×1.3, 박자 밖에 ×0.75.
            let raw = self.stats.dmg + lv + s.damage;
            let raw = if measure && s.heavy.is_none() && !s.dash {
                let k = if s.beat {
                    crate::progression::key::MEASURE_ON
                } else {
                    crate::progression::key::MEASURE_OFF
                };
                (raw as f32 * k).round() as i32
            } else {
                raw
            };
            // 뿌리로 받기(0215 다리): 엉키거나 묶인 적에게 반격 ×1.5.
            let raw = if rooted_riposte
                && swing == self.player.parry.riposte_swing
                && (en.marks.snare > 0.0 || en.marks.bind > 0.0)
            {
                (raw as f32 * link::ROOTED_RIPOSTE).round() as i32
            } else {
                raw
            };
            en.hp -= wound(en, raw, crit, self.stats.opened_dmg, knack, t, ev);
            if crit {
                en.exposed = 0.0;
                ev.push(Event::Crit {
                    at: en.pos + Vec3::Y * en.height * 0.6,
                });
            }
            en.struck();
            en.knock = if s.pull > 0.0 {
                // 걸어 당기기(0274 ⑤ 낫): 넉백 대신 몸 앞(`PULL_STOP`)까지 끌어온다 — 문지기는 끌리지 않는다(무겁다).
                if en.boss {
                    Vec3::ZERO
                } else {
                    tug(en, pull_slide(d.length(), en.radius, s.pull), ev);
                    -dir * pull_slide(d.length(), en.radius, s.pull) * KNOCK_DECAY
                }
            } else {
                dir * t[P::Knockback] * s.knock * if en.boss { 0.25 } else { 1.0 }
            };
            // 맞은 쪽으로 몸이 젖는다(패링과 같은 `sway`, 2026-11-06 — 흰 번쩍만으로는 "맞았다"가 아니다, KB).
            en.sway = dir
                * if s.heavy.is_some() {
                    HEAVY_SWAY
                } else {
                    HIT_SWAY
                };
            en.sway_t = 0.0;
            self.sparks.extend(sparks_at(
                en.pos + Vec3::Y * en.height * 0.55,
                dir,
                swing,
                s.heavy.is_some() || crit,
            ));
            // 나무 칸(0107): 맺음타 ×1.5(마지막 타 뒤 박자로 이은 타) · 빈틈의 박자 ×2(굳은 적에게 박자 타) · 비껴서기 ×2(등 뒤 대시 공격).
            let mut pk = 1.0;
            if s.heavy.is_none() && s.beat {
                if swing == self.player.parry.closing_swing && self.tree.has(Cell::ClosingBlow) {
                    pk *= cell::CLOSING_POISE;
                }
                if matches!(en.state, EState::Recover { .. }) && self.tree.has(Cell::GapBeat) {
                    pk *= cell::GAP_POISE;
                }
            }
            if swing == self.player.flank_swing && self.tree.has(Cell::Sidestep) {
                pk *= cell::SIDESTEP_POISE;
            }
            if s.beat && opened && breaking_beat {
                pk *= link::BREAK_POISE;
            }
            // 두 손 내려베기(0276 — 한손검의 빈 손): 잡몹 · 정예는 크게, 문지기는 조금(자세 막대의 문턱 — 모의).
            if s.heavy.is_some() && two_hands {
                pk *= if en.boss {
                    weapons::BARE_SWORD_HEAVY_BOSS
                } else {
                    weapons::BARE_SWORD_HEAVY_POISE
                };
            }
            // 무너진 틈(0107): 깨진 적에게 다 모은 강타 = 깨짐을 처음부터.
            let was_broken = en.stance.broken > 0.0;
            add_poise(en, s.poise * self.stats.poise * pk, primed, t, ev);
            // 모은 맞받기(0107): 모으기 패링으로 튕긴 그 강타가 맞으면 자세가 곧바로 깨진다(문지기는 막대 반).
            if s.heavy.is_some()
                && swing == self.player.parry.gathered_swing
                && self.tree.has(Cell::GatheredRiposte)
            {
                let k = if en.boss { 0.5 } else { 1.0 };
                add_poise(en, poise_max(en, t) * k, None, t, ev);
            }
            if s.heavy == Some(true) && was_broken && self.tree.has(Cell::OpenBreach) {
                let dur = if en.boss {
                    t[P::PoiseBreakBoss]
                } else {
                    t[P::PoiseBreak]
                };
                en.stance.broken = dur;
                en.exposed = en.exposed.max(dur);
            }
            // 타가 싣는 속성(0274 ⑥ — 완드의 셋째 찌르기 · 강공격 = 균열). 쓰러진 적에는 걸리지 않는다(`afflict`).
            afflict(en, s.dose, ev);
            first_hit.get_or_insert((en.pos, en.radius, dir));
            ev.push(Event::Hit {
                at: en.pos + Vec3::Y * en.height * 0.6,
            });
            if let Some(full) = s.heavy {
                // 강타는 적의 상태를 바꾼다(KB: 모으기는 시간을 내준 만큼 — 휘청이 예고를 끊는다). 보스는 다 모았을 때만.
                let dur = match (en.boss, full) {
                    (true, true) => t[P::HeavyBossStagger],
                    (true, false) => 0.0,
                    (false, true) => t[P::HeavyStagger],
                    (false, false) => t[P::HeavyStagger] * 0.5,
                };
                // 전환 중인 보스(0133)는 휘청이지 않는다 — 강타가 전환을 끊지 못한다.
                if dur > 0.0 && !en.rising() {
                    en.state = EState::Stagger { t: 0.0, dur };
                }
                ev.push(Event::Smash {
                    at: en.pos + Vec3::Y * en.height * 0.5,
                });
                heavy_hit = true;
            }
            stop = true;
            if en.hp <= 0 {
                en.state = EState::Dead;
                self.kills += 1;
                ev.push(Event::EnemyDown { at: en.pos });
            }
        }
        // 결의 몫은 휘두르기 한 번에 한 번(광역이 몇 배로 돌려받지 않는다) — 틱 끝 `skill_books`가 준다.
        // (둘은 저마다 센다 — 금 간 적을 먼저 벤 휘두르기가 뒤에 닿은 묶인 적의 몫을 삼키지 않게.)
        let paid = &mut self.player.skill.vein_swing;
        if bound_hit && paid[0] != swing {
            paid[0] = swing;
            self.vein_bound = true;
        }
        if cracked_hit && paid[1] != swing {
            paid[1] = swing;
            self.vein_cracked = true;
        }
        if let (Some(at), true) = (stunned_at, paid[2] != swing) {
            paid[2] = swing;
            self.vein_rattle = Some(at);
        }
        if bled_hit && paid[3] != swing {
            paid[3] = swing;
            self.vein_bled = true;
        }
        if sipped_hit && paid[4] != swing {
            paid[4] = swing;
            self.vein_sipped = true;
        }
        Landed {
            stop,
            heavy_hit,
            first_hit,
            deflected,
        }
    }
}
