//! 타입 상태의 틱(0223 V2 — 규칙은 `status.rs`): 화상이 타고, 묶인 적이 서고, 울릴 때가 된 적이 둘레를 울리고, 화상 중에 묶인
//! 적이 터진다. 스킬이 남긴 것의 틱(`tick_skill_fields`) 바로 뒤에 돈다 — 그 틱이 걷는 빠르기 `slow`를 새로 적은 다음이다.
//! 타입이 스스로 낸 피해는 스킬의 타로 센다(큰 기술 게이지를 채우지 않고, 쓰러뜨려도 기세가 아니다).

use super::skill_tick::Touch;
use super::*;
use crate::status::{Kind, NO_DOSE};
use crate::tuning::{
    BURN_DMG, BURST_DMG, BURST_RADIUS, ROOT_BOSS_SLOW, TOLL_DMG, TOLL_RADIUS, TOLL_RIFT_SECS,
    TOLL_SHARE,
};

impl World {
    pub(super) fn tick_ails(&mut self, t: &Tuning, dt: f32) {
        let mut ev = std::mem::take(&mut self.skill_ev);
        let mut tolls = Vec::new();
        let mut bursts = Vec::new();
        for (i, en) in self.enemies.iter_mut().enumerate() {
            if !en.alive() {
                continue;
            }
            let out = en.ail.tick(dt);
            // 묶임: 걷지 못한다(공격은 그대로 — 패링할 것은 남는다). 보스는 묶이지 않고 느려지기만.
            if en.ail.bound > 0.0 {
                en.slow = if en.boss {
                    en.slow.min(ROOT_BOSS_SLOW)
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
            if out.toll {
                tolls.push(i);
            }
            if out.burst > 0 {
                bursts.push((en.pos, out.burst));
            }
        }
        for i in tolls {
            self.toll(t, &mut ev, i);
        }
        // 겹침: 화상 중에 묶인 적은 쓴 겹만큼 둘레를 터뜨린다.
        for (at, n) in bursts {
            ev.push(Event::AilBurst {
                at,
                radius: BURST_RADIUS,
            });
            self.skill_blast(t, &mut ev, at, BURST_RADIUS, BURST_DMG * n as i32, NO_DOSE);
        }
        self.skill_ev = ev;
    }

    /// 울림: `i`번 적이 울린다 — 둘레 `TOLL_RADIUS` m의 적(울린 적도)이 피해를 받고(자세는 깎지 않는다), 이웃은 울림 게이지를 나눠 받는다
    /// (가득 차면 그 적도 조금 뒤에 울린다 — 떼를 타고 번진다. 한 번 울린 적은 `TOLL_REST` 동안 다시 울리지 않는다).
    /// 겹침: 타는 적이 울리면 이웃에 불이 한 겹 옮고, 금 간 적이 울리면 이웃에도 금이 번진다.
    fn toll(&mut self, t: &Tuning, ev: &mut Vec<Event>, i: usize) {
        let src = &self.enemies[i];
        if !src.alive() {
            return;
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
            let down = self.skill_touch(
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
    }
}
