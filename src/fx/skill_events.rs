//! 사건 묶음 "스킬 · 원거리 · 궁극기"의 갈래마다 이펙트(0278 R25a — `Fx::on_skill_event`의 갈래를 사건마다 이름 있는 함수로,
//! 몸은 그대로). 새 사건 = `on_skill_event`의 갈래 한 줄 + 여기 함수 하나.
use super::*;
use crate::det::Det;

impl Fx {
    /// 종류의 RB(0274 ②b): 던지는 것은 손끝의 쇳빛 바람 줄(손도끼·창 — 돌은 먼지 한 줌), 신성한 빛은 손 위로 솟는 금빛 —
    /// 내려 꽂히는 순간은 `Smite`.
    pub(super) fn ev_cast_thrown(&mut self, at: Vec3, w: &World) {
        let fwd = w.player.facing_dir();
        self.star(at, 0.7, 0.06, WHITE_HOT);
        self.spray(
            at,
            fwd,
            0.35,
            5,
            6.0,
            Shape::Streak,
            0.06,
            0.18,
            3.0,
            WHITE_HOT,
            STEEL,
        );
    }

    /// 땅 내려찍기(0274 ④): 날이 땅에 닿은 발 앞의 흰 별 + 앞으로 번지는 흙빛 고리 한 겹 + 튀는 돌 + 먼지.
    pub(super) fn ev_cast_quake(&mut self, at: Vec3, pp: Vec3, w: &World) {
        let fwd = w.player.facing_dir();
        let g = vec3(pp.x, w.ground_y(pp), pp.z) + fwd * 1.0;
        self.star(g + Vec3::Y * 0.3, 1.5, 0.1, WHITE_HOT);
        self.mark(
            g,
            MarkKind::Shock {
                dir: fwd,
                r0: 0.3,
                r1: 2.2,
                arc_deg: 80.0,
            },
            0.22,
            QUAKE,
        );
        self.chips(g, fwd, 1.6, 70.0, 6);
        self.dust_ring(g, 0.5, 8, 3.2, 0.5, DUST);
        let _ = at;
    }

    /// 저주탄을 날렸다(0274 ⑥ — 완드의 주된 공격): 돌끝의 작은 하늘빛 별 + 앞으로 흩어지는 잿빛 부스러기 몇(탄은 렌더가 `bolts`로).
    pub(super) fn ev_cast_curse(&mut self, at: Vec3, w: &World) {
        let fwd = w.player.facing_dir();
        self.star(at, 0.7, 0.07, SKY);
        self.spray(at, fwd, 0.4, 4, 3.0, Shape::Dot, 0.05, 0.22, 0.0, SKY, ASH);
    }

    /// 가시 씨앗을 던졌다(0274 ⑤): 손의 작은 민트 별 + 흩어지는 흙 몇(씨가 날아가는 것은 렌더가 `smites`로).
    pub(super) fn ev_cast_seed(&mut self, at: Vec3) {
        self.star(at, 0.8, 0.08, MINT);
        self.rise(at, 0.2, 4, Shape::Dot, (0.06, 0.03), 0.3, MINT, LIFE, true);
    }

    /// 수류탄을 던졌다(0278 — 빠진 이펙트): 손끝에서 던지는 쪽으로 튀는 불티 셋(심지) + 손에서 떨어지는 흙먼지 몇(꾸밈).
    /// 수류탄 자체와 터짐은 렌더 `bombs` · `Boom`.
    pub(super) fn ev_throw(&mut self, pp: Vec3, w: &World) {
        let fwd = w.player.facing_dir();
        let hand = pp + Vec3::Y * (w.player.air + 1.2) + fwd * 0.35;
        self.spray(
            hand,
            fwd + Vec3::Y * 0.6,
            0.35,
            3,
            3.5,
            Shape::Dot,
            0.06,
            0.25,
            6.0,
            WHITE_HOT,
            AMBER,
        );
        self.rise(
            hand,
            0.12,
            3,
            Shape::Puff,
            (0.12, 0.2),
            0.35,
            DUST,
            DUST,
            true,
        );
    }

    pub(super) fn ev_cast_stone(&mut self, at: Vec3, w: &World) {
        self.spray(
            at,
            w.player.facing_dir(),
            0.5,
            4,
            3.0,
            Shape::Dot,
            0.06,
            0.2,
            6.0,
            DUST,
            ASH,
        );
    }

    pub(super) fn ev_cast_light(&mut self, at: Vec3) {
        self.star(at + Vec3::Y * 0.4, 0.9, 0.09, GOLD);
        self.rise(
            at,
            0.25,
            6,
            Shape::Streak,
            (0.08, 0.04),
            0.35,
            WHITE_HOT,
            GOLD,
            false,
        );
    }

    /// 신성한 빛이 내려 꽂혔다(0274 ②b): 가운데 큰 흰 별 + 바닥에 퍼지는 금빛 고리 둘 + 위로 솟는 빛줄 + 둘레로 튀는 줄.
    pub(super) fn ev_smite(&mut self, at: Vec3, radius: f32) {
        self.star(at + Vec3::Y * 0.8, 2.2, 0.12, WHITE_HOT);
        self.mark(
            at,
            MarkKind::Ring {
                r0: radius * 0.3,
                r1: radius,
            },
            0.32,
            GOLD,
        );
        self.mark_with(
            at,
            MarkKind::Ring {
                r0: 0.2,
                r1: radius * 0.75,
            },
            0.3,
            WHITE_HOT,
            0.05,
            Stick::Ground,
        );
        self.rise(
            at,
            radius * 0.6,
            10,
            Shape::Streak,
            (0.1, 0.05),
            0.5,
            WHITE_HOT,
            GOLD,
            false,
        );
        self.ring_sparks(
            at + Vec3::Y * 0.3,
            radius * 0.5,
            10,
            6.0,
            0.22,
            WHITE_HOT,
            GOLD,
        );
    }

    /// 가시 씨앗이 떨어져 뿌리가 솟았다(0274 ⑤): 박히는 작은 흰 별 + 바닥에 번지는 뿌리빛(민트) 고리 + 튀는 흙 + 먼지 — 솟는
    /// 가시 뿌리 자체는 렌더가(`skill_spikes`).
    pub(super) fn ev_sprout(&mut self, at: Vec3, radius: f32) {
        self.star(at + Vec3::Y * 0.3, 1.2, 0.1, WHITE_HOT);
        self.mark(
            at,
            MarkKind::Ring {
                r0: radius * 0.3,
                r1: radius,
            },
            0.35,
            MINT,
        );
        self.chips(at, Vec3::Z, radius, 360.0, 6);
        self.dust_ring(at, radius * 0.4, 10, 3.0, 0.5, DUST);
    }

    /// 던진 창이 손에 돌아왔다(0274 ②b): 손 높이의 작은 쇳빛 별.
    pub(super) fn ev_javelin_back(&mut self, at: Vec3) {
        self.star(at, 0.8, 0.07, WHITE_HOT);
        self.spray(
            at,
            Vec3::Y,
            1.0,
            3,
            2.5,
            Shape::Dot,
            0.06,
            0.2,
            4.0,
            WHITE_HOT,
            STEEL,
        );
    }

    /// 기도하는 손(0276 — 망치의 빈 손): 삼킨 자리에서 바닥으로 퍼지는 금빛 고리 둘 + 손 위의 작은 흰 별 + 오르는 빛줄.
    pub(super) fn ev_prayer(&mut self, at: Vec3, radius: f32, w: &World) {
        let g = vec3(at.x, w.ground_y(at), at.z);
        self.star(at + Vec3::Y * 1.3, 1.0, 0.09, WHITE_HOT);
        self.mark(
            g,
            MarkKind::Ring {
                r0: 0.4,
                r1: radius,
            },
            0.4,
            GOLD,
        );
        self.mark_with(
            g,
            MarkKind::Ring {
                r0: 0.2,
                r1: radius * 0.8,
            },
            0.36,
            WHITE_HOT,
            0.08,
            Stick::Ground,
        );
        self.rise(
            g,
            radius * 0.5,
            8,
            Shape::Streak,
            (0.08, 0.04),
            0.45,
            WHITE_HOT,
            GOLD,
            false,
        );
    }

    /// 쇠 덫을 놓았다(0276): 땅에 닿는 작은 쇳빛 번쩍 + 먼지 한 줌.
    pub(super) fn ev_snare_set(&mut self, at: Vec3) {
        self.star(at + Vec3::Y * 0.2, 0.7, 0.07, STEEL);
        self.dust_ring(at, 0.3, 5, 2.0, 0.4, DUST);
    }

    /// 쇠 덫이 물었다: 흰 별 + 조여드는 쇳빛 고리 + 튀는 쇳조각.
    pub(super) fn ev_snare_snap(&mut self, at: Vec3) {
        self.star(at + Vec3::Y * 0.4, 1.4, 0.1, WHITE_HOT);
        self.mark(at, MarkKind::Ring { r0: 1.1, r1: 0.3 }, 0.22, STEEL);
        self.ring_sparks(at + Vec3::Y * 0.3, 0.35, 8, 5.0, 0.2, WHITE_HOT, STEEL);
    }

    /// 던진 방패가 튕겼다(0276): 금빛 별 + 작은 고리.
    pub(super) fn ev_shield_bounce(&mut self, at: Vec3, w: &World) {
        self.star(at, 1.1, 0.08, GOLD);
        self.mark(
            vec3(at.x, w.ground_y(at), at.z),
            MarkKind::Ring { r0: 0.3, r1: 1.0 },
            0.2,
            GOLD,
        );
    }

    /// 들이받기가 나갔다(0276): 가는 쪽으로 남는 쇳빛 띠 + 발밑 먼지(모인 만큼 길다).
    pub(super) fn ev_rush(&mut self, at: Vec3, dir: Vec3, power: f32, w: &World) {
        let (d0, d1) = crate::tuning::RUSH_DIST;
        let g = vec3(at.x, w.ground_y(at), at.z);
        self.mark(
            g,
            MarkKind::Streak {
                dir,
                len: d0 + (d1 - d0) * power,
                half: 0.35,
            },
            0.35,
            STEEL,
        );
        self.dust_ring(g, 0.4, 6, 3.0, 0.5, DUST);
    }

    /// 마법 탄 발사: 손끝의 작은 라벤더 번쩍임.
    pub(super) fn ev_cast_magic(&mut self, at: Vec3) {
        self.star(at, 0.8, 0.07, LAVENDER);
        self.spray(
            at,
            Vec3::Y,
            1.0,
            4,
            2.5,
            Shape::Dot,
            0.07,
            0.25,
            2.0,
            WHITE_HOT,
            LAVENDER,
        );
    }

    /// 딱 맞춰 떼기: 금 알갱이가 사방으로.
    pub(super) fn ev_perfect(&mut self, at: Vec3) {
        self.star(at, 1.2, 0.09, GOLD);
        self.spray(
            at,
            Vec3::Y,
            1.0,
            8,
            5.0,
            Shape::Dot,
            0.08,
            0.35,
            5.0,
            WHITE_HOT,
            GOLD,
        );
    }

    /// 모인 자리(0158 G1, 끌어모으기 ㄱ): 모인 점에서 터지는 민트빛 별 + 바깥으로 퍼지는 고리와 먼지 + 위로 튀는 빛줄.
    pub(super) fn ev_skill_burst(&mut self, at: Vec3, w: &World) {
        let g = vec3(at.x, w.ground_y(at), at.z);
        self.star(g + Vec3::Y * 0.7, 2.0, 0.12, MINT);
        self.mark(g, MarkKind::Ring { r0: 0.4, r1: 2.5 }, 0.3, MINT);
        self.dust_ring(g, 0.5, 10, 4.2, 0.55, DUST);
        self.spray(
            g + Vec3::Y * 0.5,
            Vec3::Y,
            1.0,
            8,
            6.0,
            Shape::Streak,
            0.07,
            0.25,
            4.0,
            WHITE_HOT,
            MINT,
        );
    }

    /// 버팀이 한 대를 받아냈다: 맞은 자리의 흰 별 + 사방으로 튕기는 금빛 줄(받아낼수록 많이) + 몸 둘레로 번지는 고리.
    pub(super) fn ev_soak(&mut self, at: Vec3, n: u8, w: &World) {
        self.star(at, 1.3, 0.1, WHITE_HOT);
        self.spray(
            at,
            Vec3::Y,
            1.0,
            5 + 3 * n as usize,
            7.5,
            Shape::Streak,
            0.07,
            0.22,
            6.0,
            WHITE_HOT,
            GOLD,
        );
        let g = vec3(at.x, w.ground_y(at), at.z);
        self.mark(g, MarkKind::Ring { r0: 0.5, r1: 1.4 }, 0.2, GOLD);
    }

    /// 되울림이 퍼졌다: 종처럼 번지는 고리 셋(가운데에서 닿는 거리까지) + 민트빛 줄.
    pub(super) fn ev_echo_ring(&mut self, at: Vec3, radius: f32, w: &World) {
        let g = vec3(at.x, w.ground_y(at), at.z);
        self.star(g + Vec3::Y * 1.0, 1.8, 0.12, WHITE_HOT);
        for (i, col) in [WHITE_HOT, MINT, MINT].into_iter().enumerate() {
            self.mark_with(
                g,
                MarkKind::Ring {
                    r0: 0.6,
                    r1: radius,
                },
                0.34,
                col,
                0.07 * i as f32,
                Stick::Ground,
            );
        }
        self.ring_sparks(g + Vec3::Y * 0.9, 0.8, 12, 9.0, 0.24, WHITE_HOT, MINT);
    }

    /// 던진 것이 끝에서 되돌았다: 금빛 별 + 둘레로 튀는 줄.
    pub(super) fn ev_throw_turn(&mut self, at: Vec3) {
        self.star(at, 1.3, 0.1, GOLD);
        self.ring_sparks(at - Vec3::Y * 0.2, 0.15, 8, 5.0, 0.2, WHITE_HOT, GOLD);
    }

    /// 박힌 자리: 큰 별 + 둘레로 퍼지는 고리 둘 + 사방의 금빛 줄 + 바닥 먼지.
    pub(super) fn ev_throw_boom(&mut self, at: Vec3, radius: f32, w: &World) {
        let g = vec3(at.x, w.ground_y(at), at.z);
        self.star(g + Vec3::Y * 0.7, 2.6, 0.13, WHITE_HOT);
        self.mark(
            g,
            MarkKind::Ring {
                r0: 0.3,
                r1: radius,
            },
            0.28,
            GOLD,
        );
        self.mark_with(
            g,
            MarkKind::Ring {
                r0: 0.2,
                r1: radius * 0.7,
            },
            0.3,
            WHITE_HOT,
            0.05,
            Stick::Ground,
        );
        self.spray(
            g + Vec3::Y * 0.5,
            Vec3::Y,
            1.0,
            14,
            8.5,
            Shape::Streak,
            0.08,
            0.34,
            9.0,
            WHITE_HOT,
            GOLD,
        );
        self.dust_ring(g, 0.6, 9, 3.4, 0.6, DUST);
    }

    /// 손에 돌아왔다: 손 높이의 작은 금빛 별.
    pub(super) fn ev_throw_back(&mut self, at: Vec3) {
        self.star(at, 0.9, 0.08, GOLD);
        self.spray(
            at,
            Vec3::Y,
            1.0,
            4,
            3.0,
            Shape::Dot,
            0.07,
            0.22,
            4.0,
            WHITE_HOT,
            GOLD,
        );
    }

    /// 불티의 타: 가장자리에서 가운데로 훑는 호박빛 고리 + 솟는 불티.
    pub(super) fn ev_field_pulse(
        &mut self,
        at: Vec3,
        radius: f32,
        kind: crate::status::Kind,
        w: &World,
    ) {
        let g = vec3(at.x, w.ground_y(at), at.z);
        // 불티는 호박빛으로 솟고(잔불), 눈발은 흰 서리빛으로 내리고(0270 C3), 덫은 흙빛 뿌리 부스러기가 튄다.
        let (ring, from, to, lift, height) = match kind {
            crate::status::Kind::Frost => (ICE, WHITE_HOT, fade(COOL), -1.2, 1.6),
            crate::status::Kind::Root => (ROOT, STONE, fade(DUST), 3.5, 0.2),
            // 화약 덫(0272): 노란 충격빛 고리와 위로 튀는 흰 불똥.
            crate::status::Kind::Shock => (SHOCK, WHITE_HOT, fade(AMBER), 4.5, 0.2),
            // 시드는 그늘(0274 ⑥ — 균열): 하늘빛 고리가 조여들고 잿빛 부스러기가 천천히 가라앉는다.
            crate::status::Kind::Rift => (SKY, SKY, fade(ASH), -0.8, 1.3),
            _ => (AMBER, GOLD, fade(AMBER), 2.2, 0.1),
        };
        self.mark(
            g,
            MarkKind::Ring {
                r0: radius,
                r1: radius * 0.25,
            },
            0.3,
            ring,
        );
        for _ in 0..12 {
            let a = self.r() * std::f32::consts::TAU;
            let rr = radius * self.r().sqrt();
            let p = g + vec3(a.sin_d() * rr, height, a.cos_d() * rr);
            let life = 0.45 + 0.3 * self.r();
            let up = lift * (1.0 + 0.7 * self.r());
            self.push(Part {
                pos: p,
                vel: vec3(0.0, up, 0.0),
                age: 0.0,
                life,
                size: (0.1, 0.04),
                color: (from, to),
                gravity: if lift > 3.0 { 9.0 } else { -0.5 },
                drag: 1.5,
                shape: Shape::Dot,
                decor: false,
            });
        }
    }

    /// 몰아치기가 시작됐다: 금빛 큰 별 + 위로 솟는 줄 + 발밑에서 퍼지는 고리 둘.
    pub(super) fn ev_flurry(&mut self, at: Vec3, w: &World) {
        let g = vec3(at.x, w.ground_y(at), at.z);
        self.star(g + Vec3::Y * 1.0, 2.6, 0.14, GOLD);
        self.spray(
            g + Vec3::Y * 0.4,
            Vec3::Y,
            0.45,
            16,
            9.0,
            Shape::Streak,
            0.08,
            0.4,
            4.0,
            WHITE_HOT,
            GOLD,
        );
        self.mark(g, MarkKind::Ring { r0: 0.4, r1: 2.6 }, 0.3, GOLD);
        self.mark_with(
            g,
            MarkKind::Ring { r0: 0.3, r1: 1.8 },
            0.32,
            WHITE_HOT,
            0.08,
            Stick::Ground,
        );
    }

    /// 몰아치기 동안의 평타: 휘두를 때마다 바라보는 쪽으로 금빛 줄이 흩어진다.
    pub(super) fn ev_flurry_swing(&mut self, pp: Vec3, w: &World) {
        let f = w.player.facing_dir();
        self.spray(
            pp + Vec3::Y * 1.0 + f * 0.8,
            f + Vec3::Y * 0.2,
            0.5,
            5,
            7.0,
            Shape::Streak,
            0.06,
            0.18,
            3.0,
            WHITE_HOT,
            GOLD,
        );
    }

    /// 질풍 찌르기(0272 2단계): 찌름마다 창끝의 작은 흰 별 + 줄을 따라 짧게 번쩍이는 좁은 띠(곧 사라진다 — 잇달아 겹친다),
    /// 마지막 찌름은 큰 별 + 길게 남는 빛의 면(강철빛) + 앞으로 튀는 빛줄 + 뒤로 차이는 먼지.
    pub(super) fn ev_gale_thrust(
        &mut self,
        at: Vec3,
        dir: Vec3,
        len: f32,
        half: f32,
        last: bool,
        w: &World,
    ) {
        let g = vec3(at.x, w.ground_y(at), at.z);
        let tip = g + Vec3::Y * 0.95 + dir * 1.6;
        if last {
            self.star(tip, 2.6, 0.13, WHITE_HOT);
            self.mark(g, MarkKind::Slash { dir, len, half }, 0.6, STEEL);
            self.spray(
                tip,
                dir,
                0.2,
                12,
                14.0,
                Shape::Streak,
                0.07,
                0.3,
                1.0,
                WHITE_HOT,
                STEEL,
            );
            self.dust_kick(g, -dir, 6, 0.6);
        } else {
            self.star(tip, 0.9, 0.06, WHITE_HOT);
            self.mark(
                g,
                MarkKind::Streak {
                    dir,
                    len,
                    half: half * 0.35,
                },
                0.12,
                WHITE_HOT,
            );
        }
    }

    /// 칼바람(0274 ④): 휩쓸기마다 둘레 끝까지 번지는 강철빛 고리 + 둘레에서 바깥으로 튀는 빛줄, 마지막은 큰 별 + 잔불빛 고리 둘 +
    /// 튀는 불씨 + 먼지 고리.
    pub(super) fn ev_tempest_sweep(&mut self, at: Vec3, radius: f32, last: bool, w: &World) {
        let g = vec3(at.x, w.ground_y(at), at.z);
        if last {
            self.star(g + Vec3::Y * 1.0, 2.8, 0.14, WHITE_HOT);
            for i in 0..2 {
                self.mark_with(
                    g,
                    MarkKind::Ring {
                        r0: radius * 0.3,
                        r1: radius * (1.0 + 0.2 * i as f32),
                    },
                    0.45,
                    if i == 0 { WHITE_HOT } else { AMBER },
                    0.08 * i as f32,
                    Stick::Ground,
                );
            }
            self.ring_sparks(g + Vec3::Y * 0.9, radius * 0.8, 18, 9.0, 0.3, AMBER, GOLD);
            self.dust_ring(g, radius * 0.6, 16, 6.0, 0.7, DUST);
        } else {
            self.mark_with(
                g,
                MarkKind::Ring {
                    r0: radius * 0.5,
                    r1: radius,
                },
                0.16,
                STEEL,
                0.0,
                Stick::Ground,
            );
            self.ring_sparks(
                g + Vec3::Y * 0.9,
                radius * 0.75,
                10,
                6.5,
                0.18,
                WHITE_HOT,
                STEEL,
            );
        }
    }

    /// 치른 값(0274 ⑥): 첫 박 = 몸에서 둘레 끝까지 번지는 하늘빛(균열) 고리 + 잿빛 연기가 땅을 쓴다, 마지막 박 = 큰 흰 별 + 둘레
    /// 끝에서 안으로 조여드는 쇳빛(울림) 고리 둘 + 솟는 쇳빛 — 보라 · 빨강은 쓰지 않는다.
    pub(super) fn ev_price_pulse(&mut self, at: Vec3, radius: f32, last: bool, w: &World) {
        let g = vec3(at.x, w.ground_y(at), at.z);
        if last {
            self.star(g + Vec3::Y * 1.2, 2.4, 0.14, WHITE_HOT);
            for (r0, life) in [(radius, 0.4), (radius * 0.7, 0.55)] {
                self.mark_with(
                    g,
                    MarkKind::Ring { r0, r1: 0.4 },
                    life,
                    STEEL,
                    0.06,
                    Stick::Ground,
                );
            }
            self.rise(
                g,
                radius * 0.8,
                16,
                Shape::Dot,
                (0.1, 0.05),
                0.7,
                STEEL,
                COOL,
                false,
            );
        } else {
            self.mark_with(
                g,
                MarkKind::Ring {
                    r0: 0.4,
                    r1: radius,
                },
                0.35,
                SKY,
                0.0,
                Stick::Ground,
            );
            self.dust_ring(g, radius * 0.6, 14, 3.0, 0.5, SMOKE);
        }
    }

    /// 저주의 표가 새겨졌다(0274 ⑥): 그 적 몸의 작은 하늘빛 별 + 몸으로 모여드는 잿빛.
    pub(super) fn ev_hexed(&mut self, at: Vec3) {
        self.star(at, 1.0, 0.1, SKY);
        self.implode(at, 0.9, 8, ASH);
    }

    /// 저주가 옮았다(0274 ⑥): 쓰러진 자리에서 다음 적으로 잇는 하늘빛 줄(점 여럿이 그 길을 따라 난다) + 닿은 몸의 별.
    pub(super) fn ev_hex_jump(&mut self, from: Vec3, to: Vec3) {
        let d = to - from;
        for k in 0..8 {
            let p = from + d * (k as f32 / 8.0);
            self.push(Part {
                pos: p,
                vel: d.normalize_or_zero() * 2.0,
                age: 0.0,
                life: 0.25 + 0.03 * k as f32,
                size: (0.08, 0.03),
                color: (SKY, fade(ASH)),
                gravity: 0.0,
                drag: 1.0,
                shape: Shape::Dot,
                decor: false,
            });
        }
        self.star(to, 0.9, 0.08, SKY);
    }

    /// 죽음의 울림(0274 ⑥ — 주특성 치른 값): 금 간 채 쓰러진 자리에서 번지는 쇳빛(울림) 고리 한 겹.
    pub(super) fn ev_death_knell(&mut self, at: Vec3, w: &World) {
        let g = vec3(at.x, w.ground_y(at), at.z);
        self.mark_with(
            g,
            MarkKind::Ring {
                r0: 0.3,
                r1: crate::weapons::DEATH_KNELL_R,
            },
            0.35,
            STEEL,
            0.0,
            Stick::Ground,
        );
    }

    /// 깨어나는 숲(0274 ⑤): 고리마다 그 둘레까지 번지는 뿌리빛(민트) 고리 + 둘레의 먼지 + 튀는 흙, 마지막은 큰 흰 별 + 풀빛(흡수)
    /// 고리 + 위로 떠오르는 풀빛 — 솟는 가시 뿌리 자체는 렌더가(`skill_spikes`).
    pub(super) fn ev_grove_pulse(&mut self, at: Vec3, radius: f32, last: bool, w: &World) {
        let g = vec3(at.x, w.ground_y(at), at.z);
        self.mark_with(
            g,
            MarkKind::Ring {
                r0: radius * 0.5,
                r1: radius,
            },
            0.3,
            MINT,
            0.0,
            Stick::Ground,
        );
        self.dust_ring(g, radius * 0.9, 14, 2.0, 0.5, DUST);
        self.chips(g, Vec3::Z, radius, 360.0, 8);
        if last {
            self.star(g + Vec3::Y * 1.0, 2.4, 0.14, WHITE_HOT);
            self.mark_with(
                g,
                MarkKind::Ring {
                    r0: 0.3,
                    r1: radius * 1.1,
                },
                0.5,
                LIFE,
                0.06,
                Stick::Ground,
            );
            self.rise(
                g,
                radius * 0.7,
                16,
                Shape::Dot,
                (0.1, 0.05),
                0.7,
                LIFE,
                MINT,
                false,
            );
        }
    }

    /// 그림자 습격(0274 ③): 떠난 자리와 닿은 자리에 그늘빛 연기 한 줌씩 + 두 자리를 잇는 옅은 띠(숨어든 길).
    pub(super) fn ev_blink(&mut self, from: Vec3, to: Vec3, w: &World) {
        for p in [from, to] {
            let g = vec3(p.x, w.ground_y(p), p.z);
            self.dust_ring(g, 0.5, 10, 2.4, 0.55, SMOKE);
        }
        let way = vec3(to.x - from.x, 0.0, to.z - from.z);
        let len = way.length();
        if len > 0.3 {
            let g = vec3(to.x, w.ground_y(to), to.z);
            self.mark(
                g,
                MarkKind::Streak {
                    dir: way / len,
                    len,
                    half: 0.25,
                },
                0.25,
                ASH,
            );
        }
    }

    /// 그림자 습격의 찌름: 날끝의 작은 흰 별 + 짧은 강철빛 띠, 마지막은 큰 별 + 남는 짧은 빛의 면 + 앞으로 튀는 빛줄.
    pub(super) fn ev_ambush_stab(&mut self, at: Vec3, dir: Vec3, last: bool, w: &World) {
        let g = vec3(at.x, w.ground_y(at), at.z);
        let tip = g + Vec3::Y * 0.9 + dir * 1.0;
        if last {
            self.star(tip, 2.2, 0.12, WHITE_HOT);
            self.mark(
                g,
                MarkKind::Slash {
                    dir,
                    len: 1.8,
                    half: 0.35,
                },
                0.5,
                STEEL,
            );
            self.spray(
                tip,
                dir,
                0.25,
                10,
                11.0,
                Shape::Streak,
                0.06,
                0.25,
                1.0,
                WHITE_HOT,
                STEEL,
            );
        } else {
            self.star(tip, 0.8, 0.06, WHITE_HOT);
            self.mark(
                g,
                MarkKind::Streak {
                    dir,
                    len: 1.5,
                    half: 0.12,
                },
                0.1,
                STEEL,
            );
        }
    }

    /// 한 줄기: 길을 가르는 빛의 면(흰 속 + 금빛 겉, 길게 남는다) + 선 자리의 큰 별 + 길을 따라 늦게 터지는 불똥.
    pub(super) fn ev_line_cut(&mut self, at: Vec3, dir: Vec3, len: f32, half: f32, w: &World) {
        let g = vec3(at.x, w.ground_y(at), at.z);
        self.mark(g, MarkKind::Slash { dir, len, half }, 0.75, GOLD);
        self.star(g + Vec3::Y * 1.0 + dir * 0.8, 3.0, 0.14, WHITE_HOT);
        self.dust_kick(g, -dir, 5, 0.6);
        let side = vec3(dir.z, 0.0, -dir.x);
        let n = (len / 1.1).ceil() as usize;
        for i in 0..n {
            let along = (i as f32 + 0.5) / n as f32 * len;
            let delay = 0.03 + along / len * 0.2;
            for sgn in [-1.0f32, 1.0] {
                let life = 0.3 + 0.15 * self.r();
                let part = Part {
                    pos: g + dir * along + Vec3::Y * 0.3,
                    vel: side * sgn * (2.5 + 2.5 * self.r()) + Vec3::Y * (3.0 + 3.0 * self.r()),
                    age: 0.0,
                    life,
                    size: (0.09, 0.04),
                    color: (WHITE_HOT, fade(GOLD)),
                    gravity: 9.0,
                    drag: 2.0,
                    shape: Shape::Streak,
                    decor: false,
                };
                self.later.push((delay, part));
            }
        }
    }

    /// 큰 기술의 시동(끼운 것마다): 쟁반 뒤집기는 발밑으로 모이는 금빛, 몰아치기는 솟는 금빛, 한 줄기는 흰 별.
    pub(super) fn ev_ult_start(&mut self, pp: Vec3, w: &World) {
        let g = vec3(pp.x, w.ground_y(pp), pp.z);
        match w.player.skill.ult {
            crate::skills::Ult::Upend => {
                self.star(pp + Vec3::Y * 0.4, 1.4, 0.3, GOLD);
                self.dust_ring(g, 2.6, 12, -2.6, 0.45, DUST);
                self.mark(g, MarkKind::Ring { r0: 2.8, r1: 0.4 }, 0.3, GOLD);
            }
            crate::skills::Ult::Flurry => {
                self.star(pp + Vec3::Y * 1.0, 1.2, 0.2, GOLD);
                self.implode(g, 2.2, 10, GOLD);
            }
            crate::skills::Ult::Line => {
                self.star(pp + Vec3::Y * 1.1, 1.6, 0.3, WHITE_HOT);
                self.implode(g, 2.4, 12, WHITE_HOT);
            }
            // 0270 C3: 큰 종 = 방패 앞에 모이는 민트빛, 눈보라 = 발밑으로 빨려드는 서리빛.
            crate::skills::Ult::Bell => {
                self.star(pp + Vec3::Y * 0.9, 1.4, 0.3, MINT);
                self.implode(g, 2.6, 12, MINT);
            }
            crate::skills::Ult::Blizzard => {
                self.star(pp + Vec3::Y * 1.2, 1.3, 0.3, ICE);
                self.implode(g, 3.0, 14, ICE);
            }
            // 0272: 화살비 = 머리 위로 겨눈 살촉의 금빛 + 발밑으로 모이는 뿌리빛.
            crate::skills::Ult::Rain => {
                self.star(pp + Vec3::Y * 1.7, 1.3, 0.3, GOLD);
                self.implode(g, 2.6, 10, RIFT_ROOT);
            }
            // 0272 2단계: 질풍 찌르기 = 앞으로 겨눈 창끝의 강철빛 + 발밑으로 모이는 흰빛.
            crate::skills::Ult::Gale => {
                let f = w.player.facing_dir();
                self.star(pp + Vec3::Y * 1.1 + f * 1.2, 1.4, 0.3, STEEL);
                self.implode(g, 2.4, 12, WHITE_HOT);
            }
            // 0274 ③: 그림자 습격 = 숨어든 자리(등 뒤)에서 날끝의 강철빛 + 발밑으로 빨려드는 그늘빛(연기) — 숨어든 연기는 `Blink`가.
            crate::skills::Ult::Ambush => {
                let f = w.player.facing_dir();
                self.star(pp + Vec3::Y * 1.0 + f * 0.6, 1.2, 0.3, STEEL);
                self.implode(g, 2.0, 10, SMOKE);
            }
            // 0274 ⑥: 치른 값 = 든 완드 끝의 하늘빛 별 + 발밑으로 모이는 잿빛 연기(값이 걷히기 전).
            crate::skills::Ult::Price => {
                self.star(pp + Vec3::Y * 1.5, 1.4, 0.3, SKY);
                self.implode(g, 3.0, 14, SMOKE);
            }
            // 0274 ⑤: 깨어나는 숲 = 땅을 짚는 손의 민트 별 + 발밑으로 모이는 뿌리빛(땅이 깨어나기 전).
            crate::skills::Ult::Grove => {
                self.star(g + Vec3::Y * 0.3, 1.4, 0.3, MINT);
                self.implode(g, 3.0, 14, MINT);
                self.dust_ring(g, 2.4, 12, -2.4, 0.45, DUST);
            }
            // 0274 ④: 칼바람 = 들어 올린 날의 잔불빛 + 발밑으로 모이는 먼지(분노가 터지기 전).
            crate::skills::Ult::Tempest => {
                self.star(pp + Vec3::Y * 1.4, 1.4, 0.3, AMBER);
                self.implode(g, 2.6, 12, AMBER);
                self.dust_ring(g, 2.2, 12, -2.4, 0.45, DUST);
            }
        }
    }

    /// 큰 종(0270 C3): 큰 흰 별 + 둘레 끝까지 차례로 번지는 고리 셋(흰빛 → 민트) + 위로 솟는 민트·금빛.
    pub(super) fn ev_great_bell(&mut self, at: Vec3, radius: f32, w: &World) {
        let g = vec3(at.x, w.ground_y(at), at.z);
        self.star(g + Vec3::Y * 1.0, 3.0, 0.15, WHITE_HOT);
        for i in 0..3 {
            self.mark_with(
                g,
                MarkKind::Ring {
                    r0: 0.5,
                    r1: radius * (1.0 - 0.15 * i as f32),
                },
                0.45,
                if i == 0 { WHITE_HOT } else { MINT },
                0.1 * i as f32,
                Stick::Body,
            );
        }
        self.spray(
            g + Vec3::Y * 0.6,
            Vec3::Y,
            1.0,
            16,
            6.0,
            Shape::Dot,
            0.1,
            0.5,
            4.0,
            MINT,
            GOLD,
        );
        self.dust_ring(g, 1.2, 14, 6.0, 0.8, DUST);
    }

    /// 화살비(0272): 쏜 자리에서 위로 솟는 금빛 살 한 다발 + 내릴 자리에서 둘레 끝까지 번지는 뿌리빛 고리 둘.
    pub(super) fn ev_arrow_rain(&mut self, from: Vec3, at: Vec3, radius: f32, w: &World) {
        let g = vec3(at.x, w.ground_y(at), at.z);
        let up = (vec3(at.x - from.x, 0.0, at.z - from.z).normalize_or_zero() * 0.35 + Vec3::Y)
            .normalize();
        self.star(from + Vec3::Y * 1.6, 2.0, 0.12, WHITE_HOT);
        self.spray(
            from + Vec3::Y * 1.6,
            up,
            0.25,
            12,
            16.0,
            Shape::Streak,
            0.07,
            0.45,
            1.0,
            WHITE_HOT,
            GOLD,
        );
        for i in 0..2 {
            self.mark_with(
                g,
                MarkKind::Ring {
                    r0: 0.6,
                    r1: radius * (1.0 - 0.2 * i as f32),
                },
                0.45,
                RIFT_ROOT,
                0.12 * i as f32,
                Stick::Ground,
            );
        }
    }

    /// 눈보라(0270 C3): 흰 서리빛 별 + 둘레의 서리빛 고리 + 몸 둘레를 휘감아 오르는 눈송이.
    pub(super) fn ev_blizzard(&mut self, at: Vec3, radius: f32, w: &World) {
        let g = vec3(at.x, w.ground_y(at), at.z);
        self.star(g + Vec3::Y * 1.0, 2.4, 0.14, ICE);
        self.mark(
            g,
            MarkKind::Ring {
                r0: 1.0,
                r1: radius,
            },
            0.4,
            ICE,
        );
        for i in 0..24 {
            let a = i as f32 / 24.0 * std::f32::consts::TAU;
            let d = vec3(a.sin_d(), 0.0, a.cos_d());
            let side = vec3(d.z, 0.0, -d.x);
            let (life, up) = (0.8 + 0.4 * self.r(), 0.3 + self.r());
            self.push(Part {
                pos: g + d * radius * 0.6 + Vec3::Y * up,
                vel: side * 6.0 + Vec3::Y * 1.5,
                age: 0.0,
                life,
                size: (0.12, 0.06),
                color: (WHITE_HOT, fade(COOL)),
                gravity: 0.0,
                drag: 1.0,
                shape: Shape::Dot,
                decor: false,
            });
        }
    }

    /// 쟁반 뒤집기: 가장 큰 별 + 두 겹 먼지 고리 + 사방의 금빛 줄 — 사방으로 뻗는 땅의 금과 테두리의 먼지 벽(0158 G2).
    pub(super) fn ev_ult(&mut self, at: Vec3, w: &World) {
        let g = vec3(at.x, w.ground_y(at), at.z);
        let r = crate::skills::UPEND.radius;
        self.star(g + Vec3::Y * 0.8, 3.2, 0.15, GOLD);
        self.dust_ring(g, 1.0, 14, 7.5, 0.9, DUST);
        self.dust_ring(g, 2.5, 14, 5.0, 0.7, DUST);
        self.spray(
            g + Vec3::Y * 0.5,
            Vec3::Y,
            1.0,
            18,
            11.0,
            Shape::Streak,
            0.09,
            0.4,
            5.0,
            WHITE_HOT,
            GOLD,
        );
        self.mark(
            g,
            MarkKind::Cracks {
                dir: Vec3::Z,
                r: r * 0.85,
                n: 11,
                arc_deg: 360.0,
            },
            1.1,
            GOLD,
        );
        self.mark(g, MarkKind::Ring { r0: 1.0, r1: r }, 0.4, GOLD);
        // 테두리의 먼지 벽: 둘레를 따라 솟는 큰 먼지(꾸밈이 아니다 — 쟁반의 끝이 어디인지 보여 준다).
        let turn = self.r() * std::f32::consts::TAU;
        for i in 0..20 {
            let a = turn + i as f32 / 20.0 * std::f32::consts::TAU;
            let d = vec3(a.sin_d(), 0.0, a.cos_d());
            let life = 0.7 + 0.3 * self.r();
            let up = 1.6 + 1.4 * self.r();
            self.later.push((
                0.16,
                Part {
                    pos: g + d * (r - 0.4) + Vec3::Y * 0.2,
                    vel: d * 1.2 + Vec3::Y * up,
                    age: 0.0,
                    life,
                    size: (0.7, 1.7),
                    color: (DUST, fade(DUST)),
                    gravity: 0.0,
                    drag: 2.2,
                    shape: Shape::Puff,
                    decor: false,
                },
            ));
        }
    }
}
