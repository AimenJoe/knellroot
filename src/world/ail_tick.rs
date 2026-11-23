//! 타입 상태의 틱(0223 V2 — 규칙은 `status.rs`): 화상이 타고, 묶인 적이 서고, 울릴 때가 된 적이 둘레를 울리고, 화상 중에 묶인
//! 적이 터진다. 스킬이 남긴 것의 틱(`tick_skill_fields`) 바로 뒤에 돈다 — 그 틱이 걷는 빠르기 `slow`를 새로 적은 다음이다.
//! 타입이 스스로 낸 피해는 스킬의 타로 센다(큰 기술 게이지를 채우지 않고, 쓰러뜨려도 기세가 아니다).

use super::skill_tick::Touch;
use super::*;
use crate::progression::{vein, Vein};
use crate::status::{Kind, NO_DOSE};
use crate::tuning::{
    BLEED_DMG, BURN_DMG, BURST_DMG, BURST_RADIUS, DRAIN_HP, DRAIN_STAMINA, ROOT_BOSS_SLOW,
    TOLL_DMG, TOLL_RADIUS, TOLL_RIFT_SECS, TOLL_SHARE, TOLL_STACK,
};

impl World {
    pub(super) fn tick_ails(&mut self, t: &Tuning, dt: f32) {
        let mut ev = std::mem::take(&mut self.skill_ev);
        let mut tolls = Vec::new();
        let mut bursts = Vec::new();
        // 타다가 쓰러진 적의 남은 불(자리·겹) — 옮겨붙는 불(결, 0223 11절)이 쓴다. 쓰러진 몸의 불은 여기서 꺼진다(한 번만 옮는다).
        let mut cinders = Vec::new();
        // 흡수(0270 B): 표지가 붙은 적을 이번 틱(앞 틱의 타 — 타는 `tick_ails` 뒤에 들어온다)에 쳤나 — 틱에 한 번만 돌아온다.
        let mut drained: Option<Vec3> = None;
        for (i, en) in self.enemies.iter_mut().enumerate() {
            let struck = std::mem::take(&mut en.struck_now);
            if struck && en.alive() {
                // 언 적은 친 그 틱의 타가 크게 들어간 뒤(`hurt_by`) 깨진다.
                if en.ail.shatter() {
                    ev.push(Event::Shatter {
                        at: en.pos + Vec3::Y * en.height * 0.5,
                    });
                    if matches!(en.state, EState::Stagger { .. }) {
                        en.state = EState::Chase;
                    }
                }
                if en.ail.drain > 0.0 && drained.is_none() {
                    drained = Some(en.pos + Vec3::Y * en.height * 0.6);
                }
            }
            if !en.alive() {
                if en.ail.burn > 0 {
                    cinders.push((en.pos, std::mem::take(&mut en.ail.burn)));
                }
                continue;
            }
            let out = en.ail.tick(dt);
            // 서리(0270 B): 쌓인 만큼 느리다(얼어붙음은 휘청이 세운다). 문지기는 쌓이는 서리에 느려지지 않고, 기절·얼어붙은 짧은 초
            // 동안만 묶인 것처럼 느려진다(휘청이지 않는다).
            if en.boss {
                if en.ail.stun > 0.0 || en.ail.frozen > 0.0 {
                    en.marks.slow = en.marks.slow.min(ROOT_BOSS_SLOW);
                }
            } else {
                en.marks.slow = en.marks.slow.min(en.ail.chill());
            }
            // 날카로움: 움직인 만큼 샌다(맞아 밀린 것도 움직임이다 — 서 있으면 멎는다).
            let (leaks, dx, dz) = en.ail.walked_to(en.pos.x, en.pos.z);
            let moved = vec3(dx, 0.0, dz);
            if leaks > 0 && en.active() {
                en.hp -= hurt_by(en, BLEED_DMG * leaks as i32, false, 0, t);
                ev.push(Event::Bleed {
                    at: en.pos + Vec3::Y * en.height * 0.5,
                    dir: moved.normalize_or_zero(),
                });
                if en.hp <= 0 {
                    en.state = EState::Dead;
                    self.kills += 1;
                    self.skill_hit_tick = true;
                    ev.push(Event::EnemyDown { at: en.pos });
                    continue;
                }
            }
            // 묶임: 걷지 못한다(공격은 그대로 — 패링할 것은 남는다). 보스는 묶이지 않고 느려지기만.
            if en.ail.bound > 0.0 {
                en.marks.slow = if en.boss {
                    en.marks.slow.min(ROOT_BOSS_SLOW)
                } else {
                    0.0
                };
            }
            if out.burn > 0 && en.active() {
                // 화상: 겹마다 `BURN_DMG` — 타마다의 덤(힘)은 붙지 않고, 금·깨진 자세의 배는 받는다.
                en.hp -= hurt_by(en, BURN_DMG * out.burn as i32, false, 0, t);
                ev.push(Event::Burn {
                    at: en.pos + Vec3::Y * en.height * 0.6,
                });
                if en.hp <= 0 {
                    en.state = EState::Dead;
                    self.kills += 1;
                    self.skill_hit_tick = true;
                    ev.push(Event::EnemyDown { at: en.pos });
                    continue;
                }
            }
            if out.toll && en.active() {
                tolls.push(i);
            }
            if out.burst > 0 {
                bursts.push((en.pos, out.burst));
            }
        }
        if let Some(from) = drained {
            let pl = &mut self.player;
            pl.hp = (pl.hp + DRAIN_HP).min(pl.max_hp);
            pl.stamina = (pl.stamina + DRAIN_STAMINA).min(t[P::StaminaMax]);
            ev.push(Event::Drain { from });
        }
        // 한꺼번에 울리는 떼: 한 적이 받는 울림의 피해는 `TOLL_STACK` 번까지(떼가 클수록 한 번의 스킬이 곱절로 세지지 않게 —
        // 게이지 나눔·겹침은 그대로 번진다).
        let mut rung = vec![0u8; self.enemies.len()];
        if self.tree.veined(Vein::Ember) {
            for (at, stacks) in cinders {
                self.catch_fire(&mut ev, at, stacks);
            }
        }
        for i in tolls {
            // 울림 모으기(결): 실제로 울린 수 — 틱 끝의 셈이 큰 기술 게이지에 더한다.
            if self.toll(t, &mut ev, i, &mut rung) {
                self.vein_tolls = self.vein_tolls.saturating_add(1);
            }
        }
        // 겹침: 화상 중에 묶인 적은 쓴 겹만큼 둘레를 터뜨린다.
        for (at, n) in bursts {
            ev.push(Event::AilBurst {
                at,
                radius: BURST_RADIUS,
            });
            self.skill_blast(
                t,
                &mut ev,
                at,
                BURST_RADIUS,
                BURST_DMG * n as i32,
                NO_DOSE,
                false,
            );
        }
        self.skill_ev = ev;
    }

    /// 옮겨붙는 불(결, 0223 11절): `at`에서 쓰러진 적의 남은 불 `stacks`겹이 `vein::EMBER_REACH` m 안의 가장 가까운 적
    /// 하나에 옮겨붙는다(같은 층 — 전환 중인 보스는 아니다). 겹의 상한은 그대로(`Ail::kindle`).
    fn catch_fire(&mut self, ev: &mut Vec<Event>, at: Vec3, stacks: u8) {
        let near = self
            .enemies
            .iter_mut()
            .filter(|en| {
                en.active() && !en.harmless() && !en.rising() && (en.pos.y - at.y).abs() <= HIT_DY
            })
            .map(|en| {
                let d = vec3(en.pos.x - at.x, 0.0, en.pos.z - at.z).length() - en.radius;
                (d, en)
            })
            .filter(|(d, _)| *d <= vein::EMBER_REACH)
            .min_by(|a, b| a.0.total_cmp(&b.0));
        if let Some((_, en)) = near {
            // 제자리에 서 있던 적이면 알아차린다 — 모르는 채 서서 타 죽지 않게.
            en.notice();
            en.ail.kindle(stacks, en.boss);
            ev.push(Event::Ail {
                at: en.pos + Vec3::Y * en.height * 0.6,
                kind: Kind::Ember,
            });
        }
    }

    /// 울림: `i`번 적이 울린다 — 둘레 `TOLL_RADIUS` m의 적(울린 적도)이 피해를 받고(자세는 깎지 않는다 — 한 틱에 여럿이 울리면 `TOLL_STACK` 번까지만 아프다), 이웃은 울림 게이지를 나눠 받는다
    /// (가득 차면 그 적도 조금 뒤에 울린다 — 떼를 타고 번진다. 한 번 울린 적은 `TOLL_REST` 동안 다시 울리지 않는다).
    /// 겹침: 타는 적이 울리면 이웃에 불이 한 겹 옮고, 금 간 적이 울리면 이웃에도 금이 번진다.
    fn toll(&mut self, t: &Tuning, ev: &mut Vec<Event>, i: usize, rung: &mut [u8]) -> bool {
        let src = &self.enemies[i];
        if !src.alive() {
            return false;
        }
        let (at, burning, cracked) = (src.pos, src.ail.burn > 0, src.ail.rift > 0.0);
        ev.push(Event::Toll {
            at,
            radius: TOLL_RADIUS,
        });
        for j in 0..self.enemies.len() {
            let en = &self.enemies[j];
            if !en.active() || (en.pos.y - at.y).abs() > HIT_DY {
                continue;
            }
            let dv = vec3(en.pos.x - at.x, 0.0, en.pos.z - at.z);
            if dv.length() - en.radius > TOLL_RADIUS {
                continue;
            }
            let loud = rung[j] < TOLL_STACK;
            rung[j] = rung[j].saturating_add(1);
            let down = loud
                && self.skill_touch(
                    t,
                    ev,
                    j,
                    &Touch {
                        damage: TOLL_DMG,
                        poise: 0.0,
                        stagger: 0.0,
                        knock: Vec3::ZERO,
                        crit: false,
                        dose: NO_DOSE,
                        flat: true,
                    },
                );
            if down || j == i {
                continue;
            }
            let en = &mut self.enemies[j];
            afflict(en, (Kind::Knell, TOLL_SHARE), ev);
            if en.rising() {
                continue;
            }
            if burning {
                en.ail.kindle(1, en.boss);
                ev.push(Event::Ail {
                    at: en.pos + Vec3::Y * en.height * 0.6,
                    kind: Kind::Ember,
                });
            }
            if cracked {
                en.ail.crack(TOLL_RIFT_SECS);
            }
        }
        true
    }
}
