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
        // 평타와 강타(0030)는 같은 규칙에 값만 다르다 — 무기 표의 타(0051)가 랩 기준값에 곱한다.
        let wpn = self.player.weapon.def();
        let s = match self.player.state {
            PState::Attack {
                t: e, swing, hit, ..
            } => {
                let Some(h) = wpn.hit(hit) else {
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
                    damage: h.damage,
                    poise: h.poise,
                    knock: h.knock,
                    heavy: None,
                    perfect: false,
                    beat: self.player.swing_beat,
                    dash: hit == weapons::DASH,
                }
            }
            PState::Heavy {
                t: e,
                swing,
                full,
                perfect,
                ..
            } => {
                let Some(h) = wpn.heavy else {
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
                    damage: h.damage
                        + if full {
                            t[P::HeavyFullBonus].round() as i32
                        } else {
                            0
                        }
                        + perfect as i32 * HP_UNIT,
                    poise: h.poise * if perfect { 1.5 } else { 1.0 },
                    knock: t[P::HeavyKnock] * if perfect { 1.5 } else { 1.0 },
                    heavy: Some(full),
                    perfect,
                    beat: false,
                    dash: false,
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
                    self.player.since_perfect = 0.0;
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
        let mut deflected: Option<Vec3> = None;
        for en in &mut self.enemies {
            if !en.active() || en.hit_by == swing {
                continue;
            }
            // 높이가 다른 층의 적에게는 칼이 안 닿는다(0043).
            if (en.pos.y - body).abs() > HIT_DY {
                continue;
            }
            let d = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z);
            let dist = d.length() - en.radius;
            if dist > s.range {
                continue;
            }
            let dir = if d.length() > 1e-4 { d.normalize() } else { f };
            if !s.circle && dir.dot(f) < half {
                continue;
            }
            en.hit_by = swing;
            // 등딱지(0120): 앞에서 든 평타는 튕긴다 — 피해·자세 없음. 강타·옆/뒤·내려찍기(다른 길)는 들어간다.
            if s.heavy.is_none() && en.guards(-dir) {
                deflected.get_or_insert(en.pos + Vec3::Y * en.height * 0.6);
                continue;
            }
            // 패링으로 드러난 적은 한 대가 치명타(0025).
            let crit = en.exposed > 0.0;
            // 레벨 힘(0104): 강타·패링 반격 +`heavy_dmg`, 콤보·대시 공격 +`light_dmg` — 장신구 `dmg`는 모든 타.
            let lv = if s.heavy.is_some() {
                self.stats.heavy_dmg
            } else {
                self.stats.light_dmg
            };
            en.hp -= hurt_by(
                en,
                self.stats.dmg + lv + s.damage,
                crit,
                self.stats.opened_dmg,
                t,
            );
            if crit {
                en.exposed = 0.0;
                ev.push(Event::Crit {
                    at: en.pos + Vec3::Y * en.height * 0.6,
                });
            }
            en.hurt = 0.15;
            en.knock = dir * t[P::Knockback] * s.knock * if en.boss { 0.25 } else { 1.0 };
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
                if swing == self.player.closing_swing && self.tree.has(Cell::ClosingBlow) {
                    pk *= cell::CLOSING_POISE;
                }
                if matches!(en.state, EState::Recover { .. }) && self.tree.has(Cell::GapBeat) {
                    pk *= cell::GAP_POISE;
                }
            }
            if swing == self.player.flank_swing && self.tree.has(Cell::Sidestep) {
                pk *= cell::SIDESTEP_POISE;
            }
            // 무너진 틈(0107): 깨진 적에게 다 모은 강타 = 깨짐을 처음부터.
            let was_broken = en.broken > 0.0;
            add_poise(en, s.poise * self.stats.poise * pk, primed, t, ev);
            // 모은 맞받기(0107): 모으기 패링으로 튕긴 그 강타가 맞으면 자세가 곧바로 깨진다(문지기는 막대 반).
            if s.heavy.is_some()
                && swing == self.player.gathered_swing
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
                en.broken = dur;
                en.exposed = en.exposed.max(dur);
            }
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
                if dur > 0.0 {
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
        Landed {
            stop,
            heavy_hit,
            first_hit,
            deflected,
        }
    }
}
