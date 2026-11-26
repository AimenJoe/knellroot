//! 사건 묶음 "속성"의 갈래마다 이펙트(0278 R25a — `Fx::on_ail`의 갈래를 이름 있는 함수로, 몸은 그대로).
use super::*;

impl Fx {
    /// 타입 게이지가 가득 찼다(0223 V2): 타입마다 제 빛·제 꼴 — 화상은 솟는 불티, 묶임은 조여드는 민트 고리,
    /// 금은 하늘빛 별과 튀는 줄(울림·터짐은 제 사건).
    pub(super) fn ev_ail(&mut self, at: Vec3, kind: crate::status::Kind, w: &World) {
        use crate::status::Kind;
        let g = vec3(at.x, w.ground_y(at), at.z);
        match kind {
            Kind::Ember => {
                self.star(at, 1.2, 0.12, AMBER);
                self.rise(
                    at - Vec3::Y * 0.4,
                    0.4,
                    8,
                    Shape::Dot,
                    (0.1, 0.03),
                    0.5,
                    WHITE_HOT,
                    AMBER,
                    false,
                );
            }
            Kind::Root => {
                self.mark(g, MarkKind::Ring { r0: 1.5, r1: 0.5 }, 0.25, MINT);
                self.ring_sparks(g + Vec3::Y * 0.2, 0.9, 8, -3.0, 0.22, WHITE_HOT, MINT);
            }
            Kind::Rift => {
                self.star(at, 1.7, 0.13, SKY);
                self.ring_sparks(at, 0.2, 7, 7.5, 0.2, WHITE_HOT, SKY);
            }
            // 0270 B: 기절 = 머리 위 레몬 별 + 사방으로 튀는 짧은 불꽃 · 상처 = 은빛 별 + 진액 · 얼어붙음 = 얼음 조각이
            // 몸에서 솟고 발밑에 서리 고리 · 흡수 = 풀빛 고리가 몸으로 조여든다.
            Kind::Shock => {
                self.star(at + Vec3::Y * 0.5, 1.8, 0.14, SHOCK);
                self.ring_sparks(at + Vec3::Y * 0.5, 0.3, 9, 7.0, 0.18, WHITE_HOT, SHOCK);
            }
            Kind::Sharp => {
                self.star(at, 1.4, 0.1, WHITE_HOT);
                self.sap(at, Vec3::ZERO, 6, 2.6, 2.0);
            }
            Kind::Frost => {
                self.mark(g, MarkKind::Ring { r0: 0.3, r1: 1.3 }, 0.35, ICE);
                self.spray(
                    at - Vec3::Y * 0.3,
                    Vec3::Y,
                    0.7,
                    10,
                    3.2,
                    Shape::Streak,
                    0.08,
                    0.4,
                    6.0,
                    WHITE_HOT,
                    ICE,
                );
            }
            Kind::Drain => {
                self.mark(g, MarkKind::Ring { r0: 1.6, r1: 0.4 }, 0.3, LIFE);
                self.implode(at, 1.2, 8, LIFE);
            }
            Kind::Knell | Kind::Physical => {}
        }
    }

    /// 상처에서 샜다(0270 B): 걸어가던 쪽 뒤로 진액이 떨어진다.
    pub(super) fn ev_bleed(&mut self, at: Vec3, dir: Vec3) {
        self.sap(at, -dir, 3, 1.6, 0.8);
    }

    /// 언 적이 깨졌다: 얼음 조각이 사방으로 + 흰 별.
    pub(super) fn ev_shatter(&mut self, at: Vec3, w: &World) {
        let g = vec3(at.x, w.ground_y(at), at.z);
        self.star(at, 2.2, 0.12, WHITE_HOT);
        self.spray(
            at,
            Vec3::Y,
            0.9,
            14,
            5.5,
            Shape::Dot,
            0.11,
            0.55,
            14.0,
            WHITE_HOT,
            ICE,
        );
        self.dust_ring(g, 0.4, 6, 2.4, 0.4, ICE);
    }

    /// 흡수가 돌아왔다: 친 적에서 방랑자 쪽으로 풀빛 알갱이가 솟는다(방랑자 곁에서도 조금).
    pub(super) fn ev_drain(&mut self, from: Vec3, w: &World) {
        let to = w.player.pos + Vec3::Y * 1.0;
        let d = (to - from).normalize_or(Vec3::Y);
        self.spray(
            from,
            d,
            0.25,
            5,
            5.0,
            Shape::Streak,
            0.07,
            0.3,
            0.0,
            WHITE_HOT,
            LIFE,
        );
        self.rise(
            to - Vec3::Y * 0.6,
            0.3,
            4,
            Shape::Dot,
            (0.07, 0.03),
            0.5,
            LIFE,
            WHITE_HOT,
            false,
        );
    }

    /// 화상이 탔다: 몸에서 솟는 불티 몇(꾸밈 — 많으면 건너뛴다).
    pub(super) fn ev_burn(&mut self, at: Vec3) {
        self.rise(
            at - Vec3::Y * 0.3,
            0.35,
            5,
            Shape::Dot,
            (0.09, 0.02),
            0.45,
            AMBER,
            AMBER,
            true,
        );
    }

    /// 울렸다: 쇳빛 고리 둘이 닿는 거리까지 번지고 한가운데 별.
    pub(super) fn ev_toll(&mut self, at: Vec3, radius: f32, w: &World) {
        let g = vec3(at.x, w.ground_y(at), at.z);
        self.star(g + Vec3::Y * 1.0, 1.5, 0.12, WHITE_HOT);
        for (i, col) in [WHITE_HOT, STEEL].into_iter().enumerate() {
            self.mark_with(
                g,
                MarkKind::Ring {
                    r0: 0.4,
                    r1: radius,
                },
                0.3,
                col,
                0.08 * i as f32,
                Stick::Ground,
            );
        }
        self.ring_sparks(g + Vec3::Y * 0.9, 0.5, 8, 8.0, 0.2, WHITE_HOT, STEEL);
    }

    /// 화상 중에 묶여 터졌다: 큰 호박빛 별 + 번지는 고리 + 사방의 불티 + 바닥 먼지.
    pub(super) fn ev_ail_burst(&mut self, at: Vec3, radius: f32, w: &World) {
        let g = vec3(at.x, w.ground_y(at), at.z);
        self.star(g + Vec3::Y * 0.8, 2.4, 0.13, WHITE_HOT);
        self.mark(
            g,
            MarkKind::Ring {
                r0: 0.4,
                r1: radius,
            },
            0.26,
            AMBER,
        );
        self.ring_sparks(g + Vec3::Y * 0.7, 0.3, 12, 9.0, 0.24, WHITE_HOT, AMBER);
        self.dust_ring(g, 0.5, 8, 3.2, 0.5, DUST);
    }

    /// 뿌리 하나가 솟았다(0214): 흙이 튀고 먼지가 핀다(뿌리 자체는 렌더가 `skill_spikes`로).
    pub(super) fn ev_root_spike(&mut self, at: Vec3, w: &World) {
        let g = vec3(at.x, w.ground_y(at), at.z);
        self.dust_ring(g, 0.3, 6, 2.6, 0.45, DUST);
        self.spray(
            g + Vec3::Y * 0.2,
            Vec3::Y,
            0.6,
            4,
            4.5,
            Shape::Dot,
            0.08,
            0.4,
            14.0,
            STONE,
            STONE,
        );
    }

    /// 되찾는 생기(0214): 등잔보다 작게 — 패링으로 되찾은 한 눈금.
    pub(super) fn ev_mend(&mut self, at: Vec3) {
        self.rise(
            at - Vec3::Y * 0.6,
            0.3,
            5,
            Shape::Dot,
            (0.07, 0.035),
            0.6,
            HEAL,
            WHITE_HOT,
            false,
        );
    }
}
