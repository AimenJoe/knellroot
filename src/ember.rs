//! 잔불(0064 M1): 떼가 떨구는 화폐. 쓰러져도 잃지 않는다(0026, KB `roguelite-healing-and-currency`).
//!
//! 줍는 손맛(KB): 쓰러진 자리에서 **튀어나오고**(위·옆으로 흩어져 땅에 튄다) → `EMBER_DELAY` 뒤 가까우면 **자석**처럼 끌려와
//! → 닿으면 줍는다. 값은 몇 조각으로 나눠 흩는다(한 덩어리보다 "쏟아진다"). 바닥에 남은 것은 사라지지 않는다.
//! 월드의 일부(`World::drops`)지만 순수 함수라 시험은 여기서.
use macroquad::prelude::*;

use crate::tuning::{
    EMBER_BOUNCE, EMBER_DELAY, EMBER_GRAVITY, EMBER_MAGNET_R, EMBER_PICK_R, EMBER_PIECES,
    EMBER_POP_SIDE, EMBER_POP_UP, EMBER_PULL, EMBER_PULL_MAX,
};

/// 떨어진 잔불 조각 하나.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ember {
    pub pos: Vec3,
    pub vel: Vec3,
    /// 떨어진 뒤 초(자석은 `EMBER_DELAY` 뒤부터).
    pub age: f32,
    pub value: u32,
    /// 끌려오는 중(한 번 끌리면 끝까지 — 멀어져도 떨어지지 않는다).
    pub pulled: bool,
}

fn h01(seed: u32, k: u32) -> f32 {
    let mut v = seed.wrapping_mul(0x9E37_79B9) ^ k.wrapping_mul(0x85EB_CA6B);
    v ^= v >> 15;
    v = v.wrapping_mul(0x2C1B_3C6D);
    v ^= v >> 12;
    (v & 0xffff) as f32 / 65535.0
}

/// `value`를 조각 여럿으로 나눠 `at`에서 흩는다(조각 수 ≤ `EMBER_PIECES`, 값 합 = `value`).
pub fn scatter(at: Vec3, value: u32, seed: u32) -> Vec<Ember> {
    if value == 0 {
        return Vec::new();
    }
    let n = value.min(EMBER_PIECES);
    let (each, extra) = (value / n, value % n);
    (0..n)
        .map(|i| {
            let a = (i as f32 + h01(seed, i)) / n as f32 * std::f32::consts::TAU;
            let side = EMBER_POP_SIDE * (0.6 + 0.4 * h01(seed, 10 + i));
            let up = EMBER_POP_UP * (0.8 + 0.4 * h01(seed, 20 + i));
            Ember {
                pos: at + Vec3::Y * 0.4,
                vel: vec3(a.cos() * side, up, a.sin() * side),
                age: 0.0,
                value: each + (i < extra) as u32,
                pulled: false,
            }
        })
        .collect()
}

/// 한 틱. `ground`는 그 자리 땅 높이, `to`는 플레이어 가슴. 주웠으면 참(조각을 지운다).
pub fn step(e: &mut Ember, to: Vec3, ground: f32, dt: f32) -> bool {
    e.age += dt;
    let d = to - e.pos;
    if e.age >= EMBER_DELAY && (e.pulled || d.length() <= EMBER_MAGNET_R) {
        e.pulled = true;
    }
    if e.pulled {
        // 끌려온다: 플레이어 쪽으로 가속(빨라지는 상한), 땅·중력 없이.
        let dir = d.normalize_or_zero();
        e.vel += dir * EMBER_PULL * dt;
        let along = e.vel.dot(dir);
        // 옆으로 새는 속도는 빨리 죽인다(곧게 날아들게).
        e.vel = dir * along.max(0.0) + (e.vel - dir * along) * (1.0 - 8.0 * dt).max(0.0);
        if e.vel.length() > EMBER_PULL_MAX {
            e.vel = e.vel.normalize() * EMBER_PULL_MAX;
        }
        e.pos += e.vel * dt;
    } else {
        e.vel.y -= EMBER_GRAVITY * dt;
        e.pos += e.vel * dt;
        let floor = ground + 0.12;
        if e.pos.y < floor {
            e.pos.y = floor;
            e.vel.y = -e.vel.y * EMBER_BOUNCE;
            e.vel.x *= 0.6;
            e.vel.z *= 0.6;
            if e.vel.y < 0.4 {
                e.vel.y = 0.0;
            }
        }
    }
    (to - e.pos).length() <= EMBER_PICK_R
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tuning::TICK_DT;

    #[test]
    fn a_drop_splits_its_value_into_a_few_pieces_that_add_up() {
        for value in [1u32, 2, 3, 7, 40] {
            let e = scatter(Vec3::ZERO, value, 3);
            assert_eq!(e.iter().map(|x| x.value).sum::<u32>(), value);
            assert!(e.len() as u32 <= EMBER_PIECES && !e.is_empty());
            assert!(e.iter().all(|x| x.value > 0 && x.vel.y > 0.0));
        }
        assert!(scatter(Vec3::ZERO, 0, 3).is_empty());
    }

    #[test]
    fn embers_pop_settle_on_the_ground_then_fly_to_a_nearby_player() {
        let player = vec3(4.0, 1.0, 0.0);
        let mut e = scatter(Vec3::ZERO, 1, 9)[0];
        // 처음 `EMBER_DELAY`초는 끌리지 않는다(튀어나오는 것이 보이게).
        let mut t = 0.0;
        while t + TICK_DT < EMBER_DELAY {
            assert!(!step(&mut e, player, 0.0, TICK_DT));
            assert!(!e.pulled);
            t += TICK_DT;
        }
        // 그 뒤 자석 안이면 1.5초 안에 줍는다.
        let mut picked = false;
        for _ in 0..90 {
            if step(&mut e, player, 0.0, TICK_DT) {
                picked = true;
                break;
            }
        }
        assert!(picked);
    }

    #[test]
    fn far_embers_wait_on_the_ground_and_never_vanish() {
        let far = vec3(40.0, 1.0, 0.0);
        let mut e = scatter(Vec3::ZERO, 1, 5)[0];
        for _ in 0..600 {
            assert!(!step(&mut e, far, 0.0, TICK_DT));
        }
        assert!(!e.pulled);
        assert!(
            (e.pos.y - 0.12).abs() < 0.05,
            "resting on the ground: {}",
            e.pos.y
        );
        // 걸어오면 끌려온다.
        let near = e.pos + vec3(EMBER_MAGNET_R * 0.8, 1.0, 0.0);
        assert!((0..120).any(|_| step(&mut e, near, 0.0, TICK_DT)));
    }
}
