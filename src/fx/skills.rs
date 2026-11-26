//! 스킬마다의 판정 이펙트(0278 R25a — `Fx::on_skill`의 갈래를 스킬마다 이름 있는 함수로 옮겼다, 몸은 그대로).
//! 스킬의 이펙트는 정체성이다(서명이 서로 달라야 한다 — `every_skill_leaves_its_own_signature`): 묶지 않는다. 새 스킬 = `on_skill`의
//! 갈래 한 줄 + 여기 함수 하나. 불꽃(`fx_flare`)과 그을림(`fx_sear`)은 거의 같지만 둘뿐이라 따로 둔다(셋의 규칙).
use super::*;
use crate::skills::Form;

impl Fx {
    /// 휘돌이: 바퀴마다 닿는 거리까지 번지는 금빛 고리 + 둘레에서 바깥으로 튀는 빛줄 + 먼지 고리.
    pub(super) fn fx_whirl(&mut self, g: Vec3, d: crate::skills::Eff) {
        let Form::Ring { radius, gap, .. } = d.skill.form else {
            return;
        };
        self.mark_with(
            g,
            MarkKind::Ring {
                r0: radius * 0.45,
                r1: radius,
            },
            gap,
            GOLD,
            0.0,
            Stick::Body,
        );
        self.ring_sparks(
            g + Vec3::Y * 0.9,
            radius * 0.8,
            14,
            7.5,
            0.22,
            WHITE_HOT,
            GOLD,
        );
        self.dust_ring(g, radius * 0.55, 12, 5.0, 0.55, DUST);
    }

    /// 가로지르기: 떠난 자리의 잔상(큰 민트 별) + 길 전체에 오래 남는 빛 띠 + 뒤로 차이는 먼지.
    pub(super) fn fx_cut(&mut self, dir: Vec3, g: Vec3, d: crate::skills::Eff) {
        let Form::Lane { dist, half, .. } = d.skill.form else {
            return;
        };
        self.star(g + Vec3::Y * 0.9, 1.9, 0.12, MINT);
        self.star(g + Vec3::Y * 0.9, 1.0, 0.09, WHITE_HOT);
        self.mark(
            g,
            MarkKind::Streak {
                dir,
                len: dist,
                half,
            },
            0.55,
            MINT,
        );
        self.dust_kick(g, -dir, 5, 0.5);
    }

    /// 끌어모으기: 모이는 점으로 빨려드는 빛줄 + 조여드는 고리 + 한가운데의 작은 민트빛 터짐.
    pub(super) fn fx_gather(&mut self, g: Vec3, d: crate::skills::Eff) {
        let Form::Pull { range, .. } = d.skill.form else {
            return;
        };
        self.mark(
            g,
            MarkKind::Converge {
                r: range * 0.75,
                n: 12,
            },
            0.3,
            MINT,
        );
        self.mark(
            g,
            MarkKind::Ring {
                r0: range * 0.5,
                r1: 0.3,
            },
            0.28,
            MINT,
        );
        self.implode(g, range * 0.6, 14, MINT);
        self.star(g + Vec3::Y * 0.6, 1.5, 0.14, MINT);
        self.dust_ring(g, 2.4, 10, -3.2, 0.5, DUST);
    }

    /// 땅울림: 찍은 자리의 별 + 부채꼴을 가로질러 달리는 충격파 + 갈라지는 땅 + 튀어 오르는 돌과 먼지.
    pub(super) fn fx_quake(&mut self, dir: Vec3, g: Vec3, d: crate::skills::Eff) {
        let Form::Cone { range, arc_deg, .. } = d.skill.form else {
            return;
        };
        let c = g + dir * 1.2;
        self.star(c + Vec3::Y * 0.3, 2.2, 0.13, WHITE_HOT);
        self.mark(
            g,
            MarkKind::Shock {
                dir,
                r0: 0.8,
                r1: range,
                arc_deg,
            },
            0.26,
            GOLD,
        );
        self.mark(
            g,
            MarkKind::Cracks {
                dir,
                r: range * 0.92,
                n: 5,
                arc_deg: arc_deg * 0.8,
            },
            0.9,
            GOLD,
        );
        self.dust_ring(c, 0.8, 12, 4.4, 0.7, DUST);
        // 돌 부스러기: 부채꼴 안 여러 자리에서 튀어 오른다.
        self.chips(g, dir, range, arc_deg, 12);
    }

    /// 버팀의 터뜨림: 흰 큰 별 + 닿는 거리까지 터지는 고리 둘 + 받아낸 만큼 많아지는 금빛 줄 + 먼지.
    pub(super) fn fx_brace(&mut self, g: Vec3, d: crate::skills::Eff, w: &World) {
        let Form::Brace { radius, .. } = d.skill.form else {
            return;
        };
        let n = w.player.skill.brace_soaked as usize;
        self.star(g + Vec3::Y * 0.8, 2.0 + 0.4 * n as f32, 0.13, WHITE_HOT);
        self.mark(
            g,
            MarkKind::Ring {
                r0: 0.5,
                r1: radius,
            },
            0.24,
            WHITE_HOT,
        );
        self.mark_with(
            g,
            MarkKind::Ring {
                r0: 0.3,
                r1: radius,
            },
            0.3,
            GOLD,
            0.05,
            Stick::Ground,
        );
        self.ring_sparks(
            g + Vec3::Y * 0.7,
            0.6,
            10 + 5 * n,
            9.0 + 1.5 * n as f32,
            0.26,
            WHITE_HOT,
            GOLD,
        );
        self.dust_ring(g, 0.8, 10, 4.5, 0.6, DUST);
    }

    /// 되울림을 둘렀다: 가운데에서 번지는 잔물결 셋(작게 — 닿는 거리의 반까지) + 가슴 높이의 민트빛 별.
    pub(super) fn fx_echo(&mut self, g: Vec3, d: crate::skills::Eff) {
        let Form::Echo { radius, .. } = d.skill.form else {
            return;
        };
        self.star(g + Vec3::Y * 1.1, 1.4, 0.13, MINT);
        for i in 0..3 {
            self.mark_with(
                g,
                MarkKind::Ring {
                    r0: 0.3,
                    r1: radius * 0.55,
                },
                0.4,
                if i == 0 { WHITE_HOT } else { MINT },
                0.09 * i as f32,
                Stick::Body,
            );
        }
    }

    /// 던지기: 손을 떠나는 순간의 흰 별 + 던진 쪽으로 튀는 금빛 줄.
    pub(super) fn fx_throw(&mut self, dir: Vec3, g: Vec3) {
        let hand = g + Vec3::Y * 1.0 + dir * 0.6;
        self.star(hand, 1.2, 0.09, WHITE_HOT);
        self.spray(
            hand,
            dir,
            0.3,
            6,
            8.0,
            Shape::Streak,
            0.07,
            0.18,
            2.0,
            WHITE_HOT,
            GOLD,
        );
        self.dust_kick(g, -dir, 3, 0.4);
    }

    /// 잔불 흩기: 가운데에서 가장자리로 번지는 호박빛 고리 + 사방으로 흩어지는 불티.
    pub(super) fn fx_embers(&mut self, g: Vec3, d: crate::skills::Eff) {
        let Form::Field { radius, .. } = d.skill.form else {
            return;
        };
        self.star(g + Vec3::Y * 0.6, 1.5, 0.12, GOLD);
        self.mark(
            g,
            MarkKind::Ring {
                r0: 0.4,
                r1: radius,
            },
            0.3,
            AMBER,
        );
        for _ in 0..22 {
            let a = self.r() * std::f32::consts::TAU;
            let sp = radius * (1.4 + 1.6 * self.r());
            let life = 0.5 + 0.3 * self.r();
            let up = 1.5 + 2.5 * self.r();
            self.push(Part {
                pos: g + Vec3::Y * 0.7,
                vel: vec3(a.sin() * sp, up, a.cos() * sp),
                age: 0.0,
                life,
                size: (0.11, 0.05),
                color: (GOLD, fade(AMBER)),
                gravity: 5.0,
                drag: 3.0,
                shape: Shape::Dot,
                decor: false,
            });
        }
    }

    /// 제비걸음(0214): 걸음마다 떠나는 자리의 흰 별 + 뛰어드는 쪽으로 짧은 금빛 띠 + 차이는 먼지.
    pub(super) fn fx_swallow(&mut self, dir: Vec3, g: Vec3, d: crate::skills::Eff) {
        let Form::Hop { .. } = d.skill.form else {
            return;
        };
        self.star(g + Vec3::Y * 0.9, 1.2, 0.09, WHITE_HOT);
        self.mark(
            g,
            MarkKind::Streak {
                dir,
                len: 2.2,
                half: 0.45,
            },
            0.35,
            GOLD,
        );
        self.dust_kick(g, -dir, 3, 0.4);
    }

    /// 막아치기(0214): 내민 자리의 흰 별 + 짧은 부채꼴 충격파(민트) — 끊긴 적은 따로(`Interrupt`).
    pub(super) fn fx_bash(&mut self, dir: Vec3, g: Vec3, d: crate::skills::Eff) {
        let Form::Bash { range, arc_deg, .. } = d.skill.form else {
            return;
        };
        self.star(g + dir * 0.9 + Vec3::Y * 0.9, 1.6, 0.1, WHITE_HOT);
        self.mark(
            g,
            MarkKind::Shock {
                dir,
                r0: 0.5,
                r1: range,
                arc_deg,
            },
            0.2,
            MINT,
        );
    }

    /// 바늘땀(0270 C3): 땀마다 — 바늘 끝의 작은 흰 별 + 찌른 줄을 따라 짧게 번쩍이는 흰 띠 + 앞으로 튀는 바늘빛 몇.
    pub(super) fn fx_stitch(&mut self, dir: Vec3, g: Vec3, d: crate::skills::Eff) {
        let Form::Stab { range, .. } = d.skill.form else {
            return;
        };
        let tip = g + Vec3::Y * 0.9 + dir * range * 0.75;
        self.star(tip, 0.9, 0.07, WHITE_HOT);
        self.mark(
            g,
            MarkKind::Streak {
                dir,
                len: range,
                half: 0.16,
            },
            0.14,
            WHITE_HOT,
        );
        self.spray(
            tip,
            dir,
            0.2,
            4,
            7.0,
            Shape::Streak,
            0.05,
            0.14,
            0.0,
            WHITE_HOT,
            COOL,
        );
    }

    /// 방패 돌진(0270 C3): 방패 앞의 금빛 별 + 앞으로 터지는 좁은 충격파 + 지나갈 길의 금빛 띠 + 뒤로 차이는 먼지 많이.
    pub(super) fn fx_ram(&mut self, dir: Vec3, g: Vec3, d: crate::skills::Eff) {
        let Form::Lane { dist, half, .. } = d.skill.form else {
            return;
        };
        let front = g + Vec3::Y * 0.8 + dir * 0.7;
        self.star(front, 1.5, 0.11, GOLD);
        self.mark(
            g,
            MarkKind::Shock {
                dir,
                r0: 0.5,
                r1: 1.8,
                arc_deg: 110.0,
            },
            0.22,
            GOLD,
        );
        self.mark(
            g,
            MarkKind::Streak {
                dir,
                len: dist,
                half,
            },
            0.45,
            GOLD,
        );
        self.dust_kick(g, -dir, 7, 0.55);
    }

    /// 종 치기(0270 C3): 친 자리의 큰 흰 별 + 그 자리에서 한 번 번지는 민트 고리(종이 걸렸다) + 위로 튀는 금빛 몇.
    pub(super) fn fx_toll(&mut self, dir: Vec3, g: Vec3, d: crate::skills::Eff) {
        let Form::Bell { range, .. } = d.skill.form else {
            return;
        };
        let hit = g + dir * (range * 0.6);
        self.star(hit + Vec3::Y * 1.0, 1.7, 0.12, WHITE_HOT);
        self.mark(hit, MarkKind::Ring { r0: 0.2, r1: 1.1 }, 0.3, MINT);
        self.spray(
            hit + Vec3::Y * 1.0,
            Vec3::Y,
            0.6,
            6,
            5.0,
            Shape::Dot,
            0.08,
            0.3,
            6.0,
            GOLD,
            MINT,
        );
    }

    /// 저주 새기기(0274 ⑥): 내민 완드 끝의 하늘빛 별 + 앞으로 곧게 뻗는 가는 잿빛 줄(새김은 맞은 적의 `Hexed`가).
    pub(super) fn fx_hex(&mut self, dir: Vec3, g: Vec3, d: crate::skills::Eff) {
        let Form::Hex { .. } = d.skill.form else {
            return;
        };
        let hand = g + Vec3::Y * 1.1 + dir * 0.6;
        self.star(hand, 1.0, 0.08, SKY);
        self.spray(
            hand,
            dir,
            0.15,
            6,
            9.0,
            Shape::Streak,
            0.05,
            0.2,
            0.0,
            SKY,
            ASH,
        );
    }

    /// 시드는 그늘(0274 ⑥): 놓인 자리에서 퍼지는 하늘빛 고리 + 둘레에서 가라앉는 잿빛(그늘이 드리운다 — 치는 것은 그늘의 타).
    pub(super) fn fx_blight(&mut self, g: Vec3, d: crate::skills::Eff) {
        let Form::Drift { radius, .. } = d.skill.form else {
            return;
        };
        self.mark(
            g,
            MarkKind::Ring {
                r0: 0.3,
                r1: radius,
            },
            0.35,
            SKY,
        );
        self.dust_ring(g, radius * 0.8, 12, 1.5, 0.5, SMOKE);
    }

    /// 눈발(0270 C3): 놓인 자리 위에서 퍼지는 흰 서리빛 고리 + 위에서 내려앉는 눈송이(별 없음 — 치는 것은 눈발의 타).
    pub(super) fn fx_snow(&mut self, g: Vec3, d: crate::skills::Eff) {
        let Form::Drift { radius, .. } = d.skill.form else {
            return;
        };
        self.mark(
            g,
            MarkKind::Ring {
                r0: 0.3,
                r1: radius,
            },
            0.35,
            ICE,
        );
        for _ in 0..18 {
            let a = self.r() * std::f32::consts::TAU;
            let rr = radius * self.r().sqrt();
            let p = g + vec3(a.sin() * rr, 2.2 + self.r(), a.cos() * rr);
            let (fall, life) = (1.6 + self.r(), 0.9 + 0.4 * self.r());
            self.push(Part {
                pos: p,
                vel: vec3(0.0, -fall, 0.0),
                age: 0.0,
                life,
                size: (0.1, 0.06),
                color: (WHITE_HOT, fade(COOL)),
                gravity: 0.0,
                drag: 0.2,
                shape: Shape::Dot,
                decor: false,
            });
        }
    }

    /// 불길(0270 C3): 손끝의 호박빛 별 + 앞으로 내뿜는 부채꼴의 불 줄기(땅의 호박빛 충격파 + 앞으로 흩어지는 불티).
    pub(super) fn fx_flare(&mut self, dir: Vec3, g: Vec3, d: crate::skills::Eff) {
        let Form::Cone { range, arc_deg, .. } = d.skill.form else {
            return;
        };
        let hand = g + Vec3::Y * 1.0 + dir * 0.6;
        self.star(hand, 1.3, 0.1, AMBER);
        self.mark(
            g,
            MarkKind::Shock {
                dir,
                r0: 0.4,
                r1: range,
                arc_deg,
            },
            0.3,
            AMBER,
        );
        self.spray(
            hand,
            dir,
            (arc_deg * 0.5).to_radians().sin().max(0.2),
            16,
            range * 2.2,
            Shape::Dot,
            0.12,
            0.4,
            -1.0,
            GOLD,
            fade(AMBER),
        );
    }

    /// 땅 가르기(0274 ④): 내리친 자리의 흰 별 + 좁게 앞으로 달리는 충격파 + 길게 갈라지는 땅(균열의 보랏빛 — 생물의 색이 아니라
    /// 이펙트) + 튀는 돌.
    pub(super) fn fx_sunder(&mut self, dir: Vec3, g: Vec3, d: crate::skills::Eff) {
        let Form::Cone { range, arc_deg, .. } = d.skill.form else {
            return;
        };
        let c = g + dir * 0.9;
        self.star(c + Vec3::Y * 0.3, 2.0, 0.12, WHITE_HOT);
        self.mark(
            g,
            MarkKind::Shock {
                dir,
                r0: 0.6,
                r1: range,
                arc_deg,
            },
            0.24,
            QUAKE,
        );
        self.mark(
            g,
            MarkKind::Cracks {
                dir,
                r: range,
                n: 3,
                arc_deg: arc_deg * 0.5,
            },
            1.0,
            RIFT_ROOT,
        );
        self.dust_ring(c, 0.6, 10, 4.0, 0.6, DUST);
        self.chips(g, dir, range, arc_deg, 8);
    }

    /// 불씨 휩쓸기(0274 ④): 날이 지난 넓은 호의 잔불빛 충격파 + 날을 따라 흩어지는 불씨.
    pub(super) fn fx_sear(&mut self, dir: Vec3, g: Vec3, d: crate::skills::Eff) {
        let Form::Cone { range, arc_deg, .. } = d.skill.form else {
            return;
        };
        let hand = g + Vec3::Y * 0.9 + dir * 0.8;
        self.star(hand, 1.2, 0.1, AMBER);
        self.mark(
            g,
            MarkKind::Shock {
                dir,
                r0: 0.5,
                r1: range,
                arc_deg,
            },
            0.28,
            AMBER,
        );
        self.spray(
            g + Vec3::Y * 0.8,
            dir,
            (arc_deg * 0.5).to_radians().sin().max(0.4),
            18,
            range * 2.0,
            Shape::Dot,
            0.12,
            0.45,
            -1.0,
            GOLD,
            fade(AMBER),
        );
    }

    /// 뿌리 덫(0270 C3): 심은 자리의 작은 흙빛 고리 + 튀는 흙 몇(씨가 땅에 들었다).
    pub(super) fn fx_snare(&mut self, g: Vec3, d: crate::skills::Eff) {
        let Form::Trap { radius, .. } = d.skill.form else {
            return;
        };
        self.mark(
            g,
            MarkKind::Ring {
                r0: radius,
                r1: 0.2,
            },
            0.4,
            ROOT,
        );
        self.dust_ring(g, 0.2, 6, 1.6, 0.35, DUST);
    }

    /// 심지 볼트(0272): 쇠뇌 끝의 흰 번쩍임 + 날아간 쪽으로 남는 짧은 노란 빛 띠 + 뒤로 튀는 심지 불똥.
    pub(super) fn fx_fuse(&mut self, dir: Vec3, g: Vec3, d: crate::skills::Eff) {
        let Form::Bell { .. } = d.skill.form else {
            return;
        };
        let muzzle = g + Vec3::Y * 1.0 + dir * 0.7;
        self.star(muzzle, 1.2, 0.1, WHITE_HOT);
        self.mark(
            g + dir * 0.7,
            MarkKind::Streak {
                dir,
                len: 2.4,
                half: 0.12,
            },
            0.25,
            SHOCK,
        );
        self.spray(
            muzzle,
            -dir,
            0.5,
            5,
            5.0,
            Shape::Dot,
            0.07,
            0.3,
            7.0,
            WHITE_HOT,
            AMBER,
        );
    }

    /// 화약 덫(0272): 묻은 자리로 조여드는 노란 고리 + 튀는 흙 + 심지의 불똥 몇(화약이 땅에 들었다).
    pub(super) fn fx_mine(&mut self, g: Vec3, d: crate::skills::Eff) {
        let Form::Trap { radius, .. } = d.skill.form else {
            return;
        };
        self.mark(
            g,
            MarkKind::Ring {
                r0: radius,
                r1: 0.2,
            },
            0.4,
            SHOCK,
        );
        self.dust_ring(g, 0.2, 6, 1.6, 0.35, DUST);
        self.spray(
            g + Vec3::Y * 0.2,
            Vec3::Y,
            0.8,
            3,
            3.0,
            Shape::Dot,
            0.06,
            0.3,
            6.0,
            WHITE_HOT,
            AMBER,
        );
    }

    /// 꿰뚫기(0272 2단계 — 창): 떠난 자리의 강철빛 별 + 길 전체에 남는 좁은 흰 띠 + 길 끝의 작은 고리(멈춰 선 자리) +
    /// 앞으로 튀는 강철빛 줄 + 뒤로 차이는 먼지.
    pub(super) fn fx_skewer(&mut self, dir: Vec3, g: Vec3, d: crate::skills::Eff) {
        let Form::Lane { dist, half, .. } = d.skill.form else {
            return;
        };
        self.star(g + Vec3::Y * 0.9, 1.5, 0.1, STEEL);
        self.mark(
            g,
            MarkKind::Streak {
                dir,
                len: dist,
                half: half * 0.6,
            },
            0.45,
            WHITE_HOT,
        );
        self.mark(
            g + dir * dist,
            MarkKind::Ring { r0: 0.3, r1: 1.2 },
            0.3,
            STEEL,
        );
        self.spray(
            g + Vec3::Y * 0.9 + dir * 0.6,
            dir,
            0.15,
            6,
            11.0,
            Shape::Streak,
            0.06,
            0.2,
            1.0,
            WHITE_HOT,
            STEEL,
        );
        self.dust_kick(g, -dir, 4, 0.5);
    }

    /// 장대 휩쓸기(0272 2단계 — 창): 자루가 지난 둘레 끝까지 퍼지는 온 바퀴 충격파(강철빛) + 둘레 끝의 바깥으로 튀는 빛줄 +
    /// 넓은 먼지 고리 — 휘돌이(금빛 고리 둘)와 갈린다.
    pub(super) fn fx_pole(&mut self, dir: Vec3, g: Vec3, d: crate::skills::Eff) {
        let Form::Ring { radius, .. } = d.skill.form else {
            return;
        };
        self.star(g + Vec3::Y * 0.9, 1.6, 0.11, WHITE_HOT);
        self.mark(
            g,
            MarkKind::Shock {
                dir,
                r0: radius * 0.4,
                r1: radius,
                arc_deg: 360.0,
            },
            0.3,
            STEEL,
        );
        self.ring_sparks(
            g + Vec3::Y * 0.8,
            radius * 0.95,
            16,
            6.5,
            0.24,
            WHITE_HOT,
            STEEL,
        );
        self.dust_ring(g, radius * 0.7, 16, 5.5, 0.6, DUST);
    }

    /// 못 박기(0272 2단계 — 창·뿌리): 창끝의 흰 별 + 찌른 줄의 짧은 띠 + 꽂힌 자리에서 조여드는 뿌리빛 고리 + 땅으로 튀는 뿌리빛.
    pub(super) fn fx_pin(&mut self, dir: Vec3, g: Vec3, d: crate::skills::Eff) {
        let Form::Stab { range, .. } = d.skill.form else {
            return;
        };
        let tip = g + dir * range * 0.8;
        self.star(tip + Vec3::Y * 0.9, 1.3, 0.09, WHITE_HOT);
        self.mark(tip, MarkKind::Ring { r0: 1.1, r1: 0.25 }, 0.4, RIFT_ROOT);
        self.mark(
            g,
            MarkKind::Streak {
                dir,
                len: range,
                half: 0.12,
            },
            0.16,
            WHITE_HOT,
        );
        self.spray(
            tip + Vec3::Y * 0.5,
            -Vec3::Y,
            0.6,
            5,
            4.0,
            Shape::Dot,
            0.07,
            0.3,
            3.0,
            RIFT_ROOT,
            ROOT,
        );
    }

    /// 후려치기(0270 C3): 휘두른 쪽의 큰 노란 별 + 넓은 부채꼴 충격파 + 옆으로 흩어지는 돌 부스러기.
    pub(super) fn fx_swat(&mut self, dir: Vec3, g: Vec3, d: crate::skills::Eff) {
        let Form::Cone { range, arc_deg, .. } = d.skill.form else {
            return;
        };
        let c = g + dir * 1.0;
        self.star(c + Vec3::Y * 0.8, 2.0, 0.12, SHOCK);
        self.mark(
            g,
            MarkKind::Shock {
                dir,
                r0: 0.6,
                r1: range,
                arc_deg,
            },
            0.24,
            SHOCK,
        );
        self.chips(g, dir, range, arc_deg, 8);
        self.dust_kick(g, dir, 4, 0.5);
    }

    /// 파문(0270 C3): 내리찍은 자리의 별 + 둘레까지 번지는 흰 고리와 늦게 따라오는 민트 고리 + 둥근 먼지.
    pub(super) fn fx_ripple(&mut self, g: Vec3, d: crate::skills::Eff) {
        let Form::Wave { radius, time } = d.skill.form else {
            return;
        };
        self.star(g + Vec3::Y * 0.3, 1.8, 0.12, WHITE_HOT);
        self.mark_with(
            g,
            MarkKind::Ring {
                r0: 0.4,
                r1: radius,
            },
            time + 0.1,
            QUAKE,
            0.0,
            Stick::Body,
        );
        self.mark_with(
            g,
            MarkKind::Ring {
                r0: 0.3,
                r1: radius * 0.9,
            },
            time + 0.15,
            MINT,
            0.08,
            Stick::Body,
        );
        self.dust_ring(g, 0.6, 14, 5.5, 0.6, DUST);
    }

    /// 뿌리 솟기(0214): 뿌리가 솟을 길을 따라 먼저 달리는 땅의 금(호박빛) — 뿌리와 흙은 하나씩(`RootSpike`).
    pub(super) fn fx_roots(&mut self, dir: Vec3, g: Vec3, d: crate::skills::Eff) {
        let Form::Erupt { len, .. } = d.skill.form else {
            return;
        };
        self.mark(
            g,
            MarkKind::Cracks {
                dir,
                r: len,
                n: 2,
                arc_deg: 8.0,
            },
            0.8,
            AMBER,
        );
        self.star(g + dir * 0.8 + Vec3::Y * 0.3, 1.3, 0.1, AMBER);
    }

    /// 자루 치기(0274): 칼자루가 닿는 가까운 자리의 노란 별(충격) + 짧고 좁은 부채꼴 충격파 — 끊긴 적은 따로(`Interrupt`).
    pub(super) fn fx_pommel(&mut self, dir: Vec3, g: Vec3, d: crate::skills::Eff) {
        let Form::Bash { range, arc_deg, .. } = d.skill.form else {
            return;
        };
        self.star(g + dir * 0.7 + Vec3::Y * 1.0, 1.4, 0.09, SHOCK);
        self.mark(
            g,
            MarkKind::Shock {
                dir,
                r0: 0.4,
                r1: range,
                arc_deg,
            },
            0.18,
            SHOCK,
        );
    }

    /// 밀치기(0274): 내지른 발 앞의 흰 별 + 앞으로 밀리는 넓은 충격파(민트) + 앞으로 차이는 먼지.
    pub(super) fn fx_shove(&mut self, dir: Vec3, g: Vec3, d: crate::skills::Eff) {
        let Form::Cone { range, arc_deg, .. } = d.skill.form else {
            return;
        };
        self.star(g + dir * 0.8 + Vec3::Y * 0.5, 1.5, 0.1, WHITE_HOT);
        self.mark(
            g,
            MarkKind::Shock {
                dir,
                r0: 0.5,
                r1: range,
                arc_deg,
            },
            0.22,
            MINT,
        );
        self.dust_kick(g, dir, 5, 0.6);
    }

    /// 빠지며 베기(0274 ③): 벤 자리의 흰 별 + 앞을 가르는 짧은 강철빛 면 + 뒤로 남는 옅은 걸음 띠와 먼지.
    pub(super) fn fx_slip(&mut self, dir: Vec3, g: Vec3, d: crate::skills::Eff) {
        let Form::Slip { range, back, .. } = d.skill.form else {
            return;
        };
        self.star(g + dir * 0.9 + Vec3::Y * 0.9, 1.2, 0.08, WHITE_HOT);
        self.mark(
            g,
            MarkKind::Slash {
                dir,
                len: range,
                half: 0.2,
            },
            0.25,
            STEEL,
        );
        self.mark(
            g - dir * back,
            MarkKind::Streak {
                dir: -dir,
                len: back,
                half: 0.3,
            },
            0.3,
            ASH,
        );
        self.dust_kick(g, -dir, 4, 0.5);
    }

    /// 걸어 당기기(0274 ⑤): 날이 지난 넓은 호의 민트(뿌리) 충격파가 **안쪽으로** 조여들고, 끌려오는 흙이 몸 쪽으로 쓸린다.
    pub(super) fn fx_reap(&mut self, dir: Vec3, g: Vec3, d: crate::skills::Eff) {
        let Form::Cone { range, arc_deg, .. } = d.skill.form else {
            return;
        };
        self.star(g + Vec3::Y * 0.9 + dir * range * 0.6, 1.2, 0.1, MINT);
        self.mark(
            g,
            MarkKind::Shock {
                dir,
                r0: range,
                r1: 0.6,
                arc_deg,
            },
            0.28,
            MINT,
        );
        self.dust_kick(g + dir * range * 0.7, -dir, 6, 0.5);
    }

    /// 생기 거두기(0274 ⑤): 둘레 끝까지 번지는 풀빛(흡수) 고리 + 둘레에서 몸으로 빨려 드는 풀빛 — 체력이 도는 빛은 `Mend`가 낸다.
    pub(super) fn fx_harvest(&mut self, g: Vec3, d: crate::skills::Eff) {
        let Form::Reap { radius, .. } = d.skill.form else {
            return;
        };
        self.mark_with(
            g,
            MarkKind::Ring {
                r0: radius * 0.4,
                r1: radius,
            },
            0.3,
            LIFE,
            0.0,
            Stick::Ground,
        );
        self.implode(g + Vec3::Y * 0.6, radius, 12, LIFE);
        self.dust_ring(g, radius * 0.7, 10, 3.0, 0.4, DUST);
    }

    /// 재기(0274): 몸에서 번지는 민트 고리 + 둥근 먼지 — 체력이 도는 빛은 `Mend`가 낸다.
    pub(super) fn fx_rally(&mut self, g: Vec3, d: crate::skills::Eff) {
        let Form::Rally { radius, .. } = d.skill.form else {
            return;
        };
        self.mark_with(
            g,
            MarkKind::Ring {
                r0: 0.3,
                r1: radius,
            },
            0.3,
            MINT,
            0.0,
            Stick::Body,
        );
        self.dust_ring(g, 0.4, 8, 3.0, 0.4, DUST);
    }
}
