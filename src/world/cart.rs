//! 레일 수레(0208 4절 — 작업층의 기믹 하나): 레일 한 줄을 정해진 박자로 오간다(밀어 주는 이 없이 — 까닭은 말하지 않는다).
//! 끝에서 `CART_WAIT`초 쉬고, 그 마지막 `CART_TELE`초는 갈 길 전체에 띠 예고(기존 예고 그림·소리 규칙), 그다음 `CART_SPEED`로 건너편 끝까지.
//! 움직이는 동안 닿는 몸은 한 번씩 맞는다 — 방랑자(맞음 피드백 다섯 겹은 `wanderer_takes_hit` 한 곳)도 적도(끌어들여 쓰는 수).
//! 박자는 일정하다 — 배우고 나면 사이로 지나간다. 월드의 단계 `tick_carts`(적 틱 뒤).
use super::*;
use crate::tuning::{
    CART_DAMAGE, CART_HALF_L, CART_HALF_W, CART_KNOCK, CART_SPEED, CART_TELE, CART_WAIT,
};

/// 레일 위 수레 하나.
#[derive(Clone, Debug, PartialEq)]
pub struct Cart {
    /// 레일의 두 끝(땅 위).
    pub a: Vec3,
    pub b: Vec3,
    /// 지금 자리(0 = a, 1 = b).
    pub at: f32,
    /// 지난 틱의 자리(그림 보간).
    pub prev: f32,
    /// 쉬는 중이면 남은 초, 움직이는 중이면 0.
    pub wait: f32,
    /// 다음에 갈 쪽(+1 = a → b).
    pub dir: f32,
    /// 문지기 싸움터의 레일(0208 2단계): 문지기가 2단계(체력 반 아래)이고 사슬이 끊기기 전에만 돈다.
    pub boss: bool,
    /// 이번에 지나가며 이미 친 것(방랑자 · 적 번호).
    pub hit_player: bool,
    pub hit: Vec<usize>,
}

impl Cart {
    /// `a` 끝에서 쉬는 수레. `phase`초만큼 박자를 앞당겨 둔다(수레가 여럿이면 엇갈리게).
    pub fn new(a: Vec3, b: Vec3, phase: f32) -> Self {
        Cart {
            a,
            b,
            at: 0.0,
            prev: 0.0,
            wait: (CART_WAIT - phase).max(0.0),
            dir: 1.0,
            boss: false,
            hit_player: false,
            hit: Vec::new(),
        }
    }

    pub fn len(&self) -> f32 {
        (self.b - self.a).length().max(1e-3)
    }

    pub fn pos_at(&self, k: f32) -> Vec3 {
        self.a.lerp(self.b, k.clamp(0.0, 1.0))
    }

    pub fn pos(&self) -> Vec3 {
        self.pos_at(self.at)
    }

    pub fn moving(&self) -> bool {
        self.wait <= 0.0
    }

    /// 예고 중이면 진행(0..1 — 다 차는 순간 떠난다).
    pub fn telling(&self) -> Option<f32> {
        (self.wait > 0.0 && self.wait <= CART_TELE).then(|| 1.0 - self.wait / CART_TELE)
    }

    /// 가는 쪽(단위).
    pub fn heading(&self) -> Vec3 {
        (self.b - self.a).normalize_or_zero() * self.dir
    }

    /// 한 틱: 쉬기 → 떠나기 → 끝에 닿으면 돌아서 쉬기.
    pub fn step(&mut self, dt: f32) {
        self.prev = self.at;
        if self.wait > 0.0 {
            self.wait -= dt;
            if self.wait <= 0.0 {
                self.wait = 0.0;
                self.hit_player = false;
                self.hit.clear();
            }
            return;
        }
        self.at += self.dir * CART_SPEED * dt / self.len();
        if self.at >= 1.0 || self.at <= 0.0 {
            self.at = self.at.clamp(0.0, 1.0);
            self.dir = -self.dir;
            self.wait = CART_WAIT;
        }
    }

    /// 몸(자리·반지름)이 수레 상자에 닿나 — 레일 방향 반 길이 `CART_HALF_L`, 옆 반 폭 `CART_HALF_W`.
    pub fn touches(&self, p: Vec3, r: f32) -> bool {
        let c = self.pos();
        let along = (self.b - self.a).normalize_or_zero();
        let d = vec3(p.x - c.x, 0.0, p.z - c.z);
        let l = d.dot(along).abs();
        let s = (d - along * d.dot(along)).length();
        l <= CART_HALF_L + r && s <= CART_HALF_W + r
    }
}

impl World {
    /// 레일 수레(0208): 움직이고, 움직이는 동안 닿는 몸을 친다 — 방랑자는 맞음 한 자리로, 적은 피해·밀림·휘청.
    pub(super) fn tick_carts(&mut self, t: &Tuning, dt: f32, ev: &mut Vec<Event>) {
        if self.carts.is_empty() {
            return;
        }
        let mut hurt_from = None;
        // 싸움터의 레일(0208): 2단계에 들어선 권양지기가 사슬이 끊기기 전까지만.
        let winch = self
            .enemies
            .iter()
            .any(|e| e.boss && e.alive() && e.roused && !e.unchained && !e.rising());
        for k in 0..self.carts.len() {
            if self.carts[k].boss && !winch && !self.carts[k].moving() {
                continue;
            }
            let was_telling = self.carts[k].telling().is_some();
            self.carts[k].step(dt);
            let c = &self.carts[k];
            if was_telling && c.moving() {
                ev.push(Event::CartGo { at: c.pos() });
            }
            if !was_telling && c.telling().is_some() {
                ev.push(Event::CartWarn { at: c.pos() });
            }
            if !c.moving() {
                continue;
            }
            let head = c.heading();
            if !c.hit_player && self.player.air < 0.6 && c.touches(self.player.pos, PLAYER_RADIUS) {
                self.carts[k].hit_player = true;
                // 밀리는 쪽은 수레가 가는 쪽 — 맞음은 수레 뒤에서 온 것으로.
                hurt_from = Some(self.player.pos - head * 2.0);
            }
            for i in 0..self.enemies.len() {
                let c = &self.carts[k];
                let en = &self.enemies[i];
                if !en.alive() || en.boss || c.hit.contains(&i) || !c.touches(en.pos, en.radius) {
                    continue;
                }
                self.carts[k].hit.push(i);
                let en = &mut self.enemies[i];
                en.hp -= CART_DAMAGE;
                en.struck();
                en.knock = head * CART_KNOCK;
                if !en.rising() {
                    en.state = EState::Stagger {
                        t: 0.0,
                        dur: t[P::SlamStagger] + 0.4,
                    };
                }
                ev.push(Event::Slam {
                    at: en.pos + Vec3::Y * en.height * 0.5,
                });
                if en.hp <= 0 {
                    en.state = EState::Dead;
                    self.kills += 1;
                    ev.push(Event::EnemyDown { at: en.pos });
                }
            }
        }
        if hurt_from.is_some() {
            self.wanderer_takes_hit(t, ev, hurt_from, 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lane() -> Cart {
        Cart::new(Vec3::ZERO, vec3(0.0, 0.0, -20.0), 0.0)
    }

    /// 박자: 쉬기(마지막 `CART_TELE`초는 예고) → 떠나 `CART_SPEED`로 건너편 → 돌아서 쉬기. 되풀이는 늘 같다.
    #[test]
    fn a_cart_rests_warns_runs_and_turns_back_on_a_steady_beat() {
        let mut c = lane();
        let dt = 1.0 / 60.0;
        let mut ticks = 0;
        while c.telling().is_none() {
            c.step(dt);
            ticks += 1;
        }
        assert!(
            ((ticks as f32) * dt - (CART_WAIT - CART_TELE)).abs() < 0.05,
            "warns before leaving"
        );
        while !c.moving() {
            c.step(dt);
        }
        let mut run = 0;
        while c.moving() {
            c.step(dt);
            run += 1;
        }
        assert_eq!(c.at, 1.0, "reached the far end");
        assert!(((run as f32) * dt - 20.0 / CART_SPEED).abs() < 0.05);
        assert_eq!(c.dir, -1.0, "turned back");
        assert_eq!(c.wait, CART_WAIT);
    }

    /// 닿는 상자: 레일 위·곁은 닿고, 옆으로 한 칸 비키면 안 닿는다(사이로 지나갈 수 있다).
    #[test]
    fn a_cart_touches_only_what_is_on_its_rail() {
        let c = lane();
        assert!(c.touches(vec3(0.0, 0.0, 0.5), 0.4));
        assert!(c.touches(vec3(CART_HALF_W, 0.0, 0.0), 0.3));
        assert!(
            !c.touches(vec3(CART_HALF_W + 1.0, 0.0, 0.0), 0.3),
            "a step aside is safe"
        );
        assert!(
            !c.touches(vec3(0.0, 0.0, -5.0), 0.4),
            "ahead of it, not yet"
        );
    }

    /// 움직이는 수레는 방랑자를 한 번 친다(맞음 한 자리 — 체력이 준다), 쉬는 수레는 치지 않는다. 적도 맞는다.
    #[test]
    fn a_moving_cart_hits_the_wanderer_once_and_hits_foes_too() {
        let t = Tuning::default();
        let mut w = World::new("test", 40.0);
        w.carts.push(lane());
        w.player.pos = vec3(0.0, 0.0, -6.0);
        w.player.prev = w.player.pos;
        let i = w.enemies.len();
        w.spawn_mob(vec3(0.0, 0.0, -12.0));
        w.enemies[i].hp = 500;
        let hp0 = w.player.hp;
        for _ in 0..((CART_WAIT * 60.0) as usize - 5) {
            w.player.pos = vec3(0.0, 0.0, -6.0);
            w.step(&Actions::default(), &t);
        }
        assert_eq!(w.player.hp, hp0, "a resting cart does not hurt");
        let mut hurts = 0;
        for _ in 0..240 {
            let before = w.player.hp;
            w.enemies[i].pos = vec3(0.0, 0.0, -12.0);
            w.step(&Actions::default(), &t);
            if w.player.hp < before {
                hurts += 1;
            }
            w.player.hurt.left = 0.0;
        }
        assert_eq!(hurts, 1, "once per pass");
        assert!(w.enemies[i].hp < 500, "foes are hit too");
    }
}

#[cfg(test)]
mod boss_tests {
    use super::*;

    /// 싸움터의 레일(0208): 권양지기가 2단계일 때만 돈다 — 잠든 동안·1단계·사슬이 끊긴 뒤(3단계)에는 멈춰 있다.
    #[test]
    fn the_arena_cart_runs_only_in_the_second_phase() {
        let t = Tuning::default();
        let mut w = World::new("test", 40.0);
        let mut c = Cart::new(vec3(-8.0, 0.0, -20.0), vec3(8.0, 0.0, -20.0), 0.0);
        c.boss = true;
        w.carts.push(c);
        w.spawn(crate::species::Kind::Overseer, vec3(0.0, 0.0, 10.0));
        let b = w.enemies.len() - 1;
        let run = |w: &mut World| {
            let (at0, wait0) = (w.carts[0].at, w.carts[0].wait);
            for _ in 0..120 {
                w.step(&Actions::default(), &t);
            }
            w.carts[0].at != at0 || w.carts[0].wait != wait0
        };
        assert!(!run(&mut w), "phase one: still");
        w.enemies[b].roused = true;
        w.enemies[b].state = EState::Recover { t: -1000.0 };
        assert!(run(&mut w), "phase two: it runs");
        w.enemies[b].unchained = true;
        // 움직이던 길은 끝까지 가서 선다.
        for _ in 0..600 {
            w.step(&Actions::default(), &t);
        }
        let at = w.carts[0].at;
        assert!(!run(&mut w) && w.carts[0].at == at, "phase three: it stops");
    }
}
