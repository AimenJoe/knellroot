//! 사건 묶음 "타격"의 갈래마다 이펙트와 적의 공격이 풀린 순간의 꼴마다(0278 R25a — `Fx::on_strike` · `enemy_blow`의 갈래를
//! 이름 있는 함수로, 몸은 그대로).
use super::*;

impl Fx {
    /// 평타: 번쩍임 한 번 — 불똥은 월드의 `sparks`(같은 스윙 = 같은 무늬)가 낸다.
    /// + 맞은 쪽으로 뿌리 진액이 튄다(0182 — 번쩍임은 맨 뒤: 같은 틱의 마지막 입자).
    pub(super) fn ev_hit(&mut self, at: Vec3, from_player: &impl Fn(Vec3) -> Vec3) {
        let d = from_player(at);
        // 멀리 튀는 방울 + 맞은 자리 곁에 쏟아지는 덩이("퍽" — 멀리 가는 것만으로는 불티처럼 읽혔다).
        self.sap(at, d, 10, 4.2, 3.6);
        self.sap(at, d, 4, 1.4, 1.6);
        self.star(at, 0.95, 0.07, WHITE_HOT);
    }

    /// 급소(0277 → 0284 — 쌍단검): 은빛의 큰 번쩍임(치명타의 금빛 별보다 크게 조금 길게) + 몸을 가르는 긴 빛줄 + 맞은 쪽에 직각으로
    /// 엇갈리는 빛줄 넷(X — 날이 지나간 자국) + 발밑의 은빛 고리 + 잠시 남는 은빛 점. 치명타(금 · 사방의 알갱이) · 패링(하늘빛 · 사방의
    /// 빛줄)과 갈린다.
    pub(super) fn ev_sneak(&mut self, at: Vec3, from_player: &impl Fn(Vec3) -> Vec3, w: &World) {
        let d = from_player(at);
        let side = vec3(-d.z, 0.0, d.x);
        // 0284(사장 판정 p-dagger "은빛 반짝은 별로 티가 안 남"): 쿼터뷰의 작은 몸 · 떼 속에서도 읽히게 — 치명타(금빛 별 1.7)보다 큰
        // 은빛 별이 조금 길게 + 흰 속, 몸을 비스듬히 가르는 긴 빛줄(양쪽으로 열리며 줄어든다), 발밑에 퍼지는 은빛 고리, 맞은 자리에
        // 잠시 남는 은빛 점. 카메라(치명타의 흔들림 · 줌 + 강타의 킥)는 `app::cue`.
        self.star(at, 2.6, 0.2, SNEAK);
        self.star(at, 1.1, 0.09, WHITE_HOT);
        let cut = (side * 0.8 + Vec3::Y * 0.6).normalize();
        for s in [1.0, -1.0] {
            self.push(Part {
                pos: at,
                vel: cut * (30.0 * s),
                age: 0.0,
                life: 0.2,
                size: (0.12, 0.03),
                color: (WHITE_HOT, fade(SNEAK)),
                gravity: 0.0,
                drag: 12.0,
                shape: Shape::Streak,
                decor: false,
            });
        }
        for (sx, sy) in [(1.0, 1.0), (-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0)] {
            let v = (side * sx + Vec3::Y * sy).normalize();
            self.push(Part {
                pos: at + v * 0.1,
                vel: v * 7.0,
                age: 0.0,
                life: 0.26,
                size: (0.16, 0.05),
                color: (WHITE_HOT, fade(SNEAK)),
                gravity: 0.0,
                drag: 5.0,
                shape: Shape::Streak,
                decor: false,
            });
        }
        let g = vec3(at.x, w.ground_y(at), at.z);
        self.mark(g, MarkKind::Ring { r0: 0.3, r1: 1.8 }, 0.32, SNEAK);
        // 남는 표: 맞은 자리에 잠시 머무는 은빛 점 하나 + 천천히 떠오르는 은빛 알갱이 여덟.
        self.push(Part {
            pos: at,
            vel: Vec3::ZERO,
            age: 0.0,
            life: 0.5,
            size: (0.24, 0.06),
            color: (WHITE_HOT, fade(SNEAK)),
            gravity: 0.0,
            drag: 0.0,
            shape: Shape::Dot,
            decor: false,
        });
        self.spray(
            at,
            Vec3::Y,
            0.8,
            8,
            1.6,
            Shape::Dot,
            0.11,
            0.5,
            -0.5,
            WHITE_HOT,
            SNEAK,
        );
    }

    /// 대검의 무거운 평타(0284): 맞은 적의 발밑에서 먼지가 둥글게 밀려난다(땅이 받은 무게 — 번쩍임 · 진액은 함께 나는 `Hit`이 낸다).
    pub(super) fn ev_heft(&mut self, at: Vec3, w: &World) {
        let g = vec3(at.x, w.ground_y(at), at.z);
        self.dust_ring(g, 0.35, 8, 2.6, 0.55, DUST);
    }

    /// 치명타: 더 크고 금빛, 금 알갱이가 사방으로.
    pub(super) fn ev_crit(&mut self, at: Vec3, from_player: &impl Fn(Vec3) -> Vec3) {
        self.sap(at, from_player(at), 6, 5.0, 3.8);
        self.star(at, 1.7, 0.11, GOLD);
        self.spray(
            at,
            Vec3::Y,
            1.0,
            9,
            5.5,
            Shape::Dot,
            0.10,
            0.34,
            9.0,
            WHITE_HOT,
            GOLD,
        );
    }

    /// 강타: 큰 번쩍임 + 돌 부스러기 + 발밑 먼지.
    pub(super) fn ev_smash(&mut self, at: Vec3, from_player: &impl Fn(Vec3) -> Vec3, w: &World) {
        let d = from_player(at);
        self.sap(at, d, 10, 6.0, 3.4);
        self.star(at, 1.5, 0.10, WHITE_HOT);
        self.spray(
            at,
            d + Vec3::Y * 0.6,
            0.55,
            7,
            5.0,
            Shape::Dot,
            0.11,
            0.45,
            14.0,
            STONE,
            STONE,
        );
        let g = vec3(at.x, w.ground_y(at), at.z);
        self.dust_ring(g, 0.3, 6, 2.2, 0.5, DUST);
    }

    /// 패링: 차가운 흰빛 별 + 사방으로 빠른 빛줄("쨍").
    pub(super) fn ev_parry(&mut self, at: Vec3) {
        self.star(at, 1.9, 0.12, COOL);
        self.spray(
            at,
            Vec3::Y,
            1.0,
            12,
            9.5,
            Shape::Streak,
            0.07,
            0.20,
            4.0,
            WHITE_HOT,
            COOL,
        );
    }

    /// 튕김(등딱지): 작은 쇳빛 별 + 플레이어 쪽으로 되튀는 쇳빛 줄.
    pub(super) fn ev_deflect(&mut self, at: Vec3, from_player: &impl Fn(Vec3) -> Vec3) {
        let back = -from_player(at);
        self.star(at, 0.9, 0.08, STEEL);
        self.spray(
            at,
            back + Vec3::Y * 0.4,
            0.45,
            6,
            6.5,
            Shape::Streak,
            0.06,
            0.18,
            8.0,
            WHITE_HOT,
            STEEL,
        );
    }

    /// 방패가 받았다(0223): 작은 쇳빛 별 + 방패에서 옆으로 튀는 짧은 불똥 — 패링(차가운 큰 별)보다 작고 둔하게.
    pub(super) fn ev_guard(&mut self, at: Vec3, from_player: &impl Fn(Vec3) -> Vec3) {
        self.star(at, 0.7, 0.07, STEEL);
        self.spray(
            at,
            from_player(at) + Vec3::Y * 0.3,
            0.9,
            5,
            4.5,
            Shape::Streak,
            0.05,
            0.16,
            8.0,
            WHITE_HOT,
            STEEL,
        );
    }

    /// 막기가 깨졌다(0223): 큰 쇳빛 별 + 사방으로 흩어지는 불똥 — 방패가 튕겨 나간 느낌.
    pub(super) fn ev_guard_break(&mut self, at: Vec3) {
        self.star(at, 1.5, 0.12, STEEL);
        self.spray(
            at,
            Vec3::Y,
            1.0,
            10,
            6.0,
            Shape::Streak,
            0.06,
            0.3,
            9.0,
            WHITE_HOT,
            STEEL,
        );
    }

    /// 자세 깨짐: 금빛 별 + 머리 높이에서 옆으로 퍼지는 금 알갱이.
    pub(super) fn ev_break(&mut self, at: Vec3) {
        self.star(at, 1.6, 0.12, GOLD);
        self.spray(
            at,
            Vec3::Y,
            1.0,
            8,
            4.5,
            Shape::Dot,
            0.09,
            0.4,
            5.0,
            GOLD,
            AMBER,
        );
    }

    /// 등짐꾼의 짐이 깨짐(0216 W2): 돌 부스러기가 사방으로 + 먼지 — 금빛 자세 깨짐(같은 틱) 위에 돌의 덩이를 더한다.
    pub(super) fn ev_load_break(&mut self, at: Vec3) {
        self.spray(
            at,
            Vec3::Y,
            1.4,
            10,
            5.0,
            Shape::Dot,
            0.14,
            0.6,
            9.0,
            STONE,
            DUST,
        );
        self.rise(at, 0.6, 6, Shape::Puff, (0.4, 0.8), 0.7, DUST, DUST, true);
    }

    /// 쓰러짐: 바랜 재가 풀려 오르고(0037 — 침식 = 색을 잃은 것), 뿌리의 보라 알갱이가 흩어진다.
    pub(super) fn ev_enemy_down(&mut self, at: Vec3) {
        let c = at + Vec3::Y * 0.5;
        // 마지막 한 번은 사방으로 터진다(0182).
        self.sap(c, Vec3::ZERO, 12, 3.0, 4.2);
        self.rise(c, 0.45, 6, Shape::Puff, (0.35, 0.75), 0.7, ASH, ASH, false);
        self.rise(c, 0.4, 7, Shape::Dot, (0.09, 0.05), 0.8, ROOT, ROOT, false);
    }

    /// 내려찍기·여진: 둘레로 먼지 고리 + 튀는 돌 부스러기.
    pub(super) fn ev_plunge(&mut self, at: Vec3, w: &World) {
        let g = vec3(at.x, w.ground_y(at), at.z);
        self.dust_ring(g, 0.5, 10, 3.6, 0.6, DUST);
        self.spray(
            g + Vec3::Y * 0.1,
            Vec3::Y,
            0.8,
            6,
            4.5,
            Shape::Dot,
            0.10,
            0.45,
            13.0,
            STONE,
            STONE,
        );
    }

    /// 벽에 부딪힌 적: 먼지 + 부스러기.
    pub(super) fn ev_slam(&mut self, at: Vec3, w: &World) {
        self.star(at, 1.0, 0.08, WHITE_HOT);
        self.dust_ring(vec3(at.x, w.ground_y(at), at.z), 0.2, 5, 1.8, 0.5, DUST);
        self.spray(
            at,
            Vec3::Y,
            0.9,
            5,
            3.5,
            Shape::Dot,
            0.10,
            0.4,
            13.0,
            STONE,
            STONE,
        );
    }

    /// 폭발: 큰 따뜻한 번쩍임 + 사방 불티 + 피어오르는 연기 + 바닥 먼지.
    pub(super) fn ev_boom(&mut self, at: Vec3, w: &World) {
        let c = at + Vec3::Y * 0.4;
        self.star(c, 2.6, 0.13, WHITE_HOT);
        self.spray(
            c,
            Vec3::Y,
            1.0,
            14,
            8.0,
            Shape::Streak,
            0.08,
            0.38,
            11.0,
            WHITE_HOT,
            AMBER,
        );
        self.rise(c, 0.6, 6, Shape::Puff, (0.5, 1.2), 0.8, SMOKE, SMOKE, false);
        self.dust_ring(vec3(at.x, w.ground_y(at), at.z), 0.6, 8, 3.0, 0.6, DUST);
    }

    /// 등잔: 따뜻한 빛 알갱이가 몸 둘레에서 떠오른다.
    pub(super) fn ev_gulp(&mut self, at: Vec3) {
        self.rise(
            at - Vec3::Y * 0.7,
            0.4,
            9,
            Shape::Dot,
            (0.08, 0.04),
            0.8,
            HEAL,
            WHITE_HOT,
            false,
        );
    }

    /// 막아치기가 준비하던 공격을 끊었다(0214): 흰 별 + 금빛 고리 + 바깥으로 튀는 줄.
    pub(super) fn ev_interrupt(&mut self, at: Vec3, w: &World) {
        self.star(at, 1.6, 0.12, WHITE_HOT);
        let g = vec3(at.x, w.ground_y(at), at.z);
        self.mark(g, MarkKind::Ring { r0: 0.3, r1: 1.3 }, 0.22, GOLD);
        self.ring_sparks(at, 0.4, 8, 6.5, 0.2, WHITE_HOT, GOLD);
    }
}
impl Fx {
    /// 둘레 내려치기: 몸 둘레 사방으로 충격파 + 금 + 돌.
    pub(super) fn blow_around(
        &mut self,
        r: f32,
        dir: Vec3,
        g: Vec3,
        s: f32,
        n: &impl Fn(f32) -> usize,
    ) {
        // 별은 몸 앞(바라보는 쪽 — 방랑자 쪽)에: 몸 가운데면 몸에 가렸다(헤드리스 화면).
        self.star(
            g + dir * (r * 0.55) + Vec3::Y * 0.5,
            2.0 * s,
            0.12,
            WHITE_HOT,
        );
        let shock = MarkKind::Shock {
            dir,
            r0: 0.4,
            r1: r,
            arc_deg: 360.0,
        };
        self.mark(g, shock, BLOW_SHOCK_LIFE, QUAKE);
        let cracks = MarkKind::Cracks {
            dir,
            r: r * 0.85,
            n: n(6.0) as u8,
            arc_deg: 360.0,
        };
        self.mark(g, cracks, 0.8, RIFT_ROOT);
        self.chips(g, dir, r, 360.0, n(9.0));
        self.dust_ring(g, r * 0.5, n(10.0), 3.5, 0.55, DUST);
    }

    /// 앞 부채꼴: 부채꼴을 가로질러 달리는 충격파 + 그 안의 금 + 돌(방랑자의 땅울림과 같은 꼴, 빛만 적의 것).
    pub(super) fn blow_cone(
        &mut self,
        r: f32,
        arc: f32,
        dir: Vec3,
        g: Vec3,
        s: f32,
        n: &impl Fn(f32) -> usize,
    ) {
        let c = g + dir * (r * 0.45);
        self.star(c + Vec3::Y * 0.4, 2.0 * s, 0.12, WHITE_HOT);
        let shock = MarkKind::Shock {
            dir,
            r0: 0.5,
            r1: r,
            arc_deg: arc,
        };
        self.mark(g, shock, BLOW_SHOCK_LIFE, QUAKE);
        let cracks = MarkKind::Cracks {
            dir,
            r: r * 0.9,
            n: n(4.0) as u8,
            arc_deg: arc * 0.8,
        };
        self.mark(g, cracks, 0.8, RIFT_ROOT);
        self.chips(g, dir, r, arc, n(9.0));
        self.dust_ring(c, 0.6, n(9.0), 3.6, 0.6, DUST);
    }

    /// 한 자리가 터진다(꼬투리·떨어진 종): 그 자리에서 위로 솟는 빛줄 + 원판 끝까지 퍼지는 고리 + 사방의 금.
    pub(super) fn blow_spot(
        &mut self,
        r: f32,
        dir: Vec3,
        g: Vec3,
        s: f32,
        n: &impl Fn(f32) -> usize,
    ) {
        self.star(g + Vec3::Y * 0.5, 2.2 * s, 0.13, WHITE_HOT);
        self.mark(g, MarkKind::Ring { r0: 0.3, r1: r }, 0.3, QUAKE);
        let cracks = MarkKind::Cracks {
            dir,
            r: r * 0.9,
            n: n(6.0) as u8,
            arc_deg: 360.0,
        };
        self.mark(g, cracks, 0.8, RIFT_ROOT);
        self.spray(
            g + Vec3::Y * 0.2,
            Vec3::Y,
            0.45,
            n(10.0),
            6.5,
            Shape::Streak,
            0.08,
            0.35,
            9.0,
            WHITE_HOT,
            QUAKE,
        );
        self.chips(g, dir, r, 360.0, n(8.0));
        self.dust_ring(g, r * 0.6, n(10.0), 3.2, 0.55, DUST);
    }

    /// 돌진이 닿은·멈춘 자리: 가슴 높이 별 + 가던 쪽으로 터지는 빛줄 + 앞쪽 충격파 호 + 앞으로 밀리는 먼지.
    pub(super) fn blow_ram(&mut self, dir: Vec3, g: Vec3, s: f32, n: &impl Fn(f32) -> usize) {
        let c = g + Vec3::Y * 0.8;
        self.star(c, 1.8 * s, 0.11, WHITE_HOT);
        self.spray(
            c,
            dir,
            0.55,
            n(10.0),
            7.0,
            Shape::Streak,
            0.08,
            0.3,
            6.0,
            WHITE_HOT,
            QUAKE,
        );
        let shock = MarkKind::Shock {
            dir,
            r0: 0.3,
            r1: 1.8 * s,
            arc_deg: 110.0,
        };
        self.mark(g - dir * 0.3, shock, 0.22, QUAKE);
        self.dust_kick(g, dir, n(5.0), 0.5);
    }

    /// 땅을 친 고리의 시작: 친 자리의 큰 쿵 + 짧은 고리 + 사방으로 뻗는 금(앞머리는 렌더·`watch`).
    pub(super) fn blow_quake(
        &mut self,
        r: f32,
        dir: Vec3,
        g: Vec3,
        s: f32,
        n: &impl Fn(f32) -> usize,
    ) {
        self.star(g + Vec3::Y * 0.6, 2.4 * s, 0.13, WHITE_HOT);
        self.mark(g, MarkKind::Ring { r0: 0.5, r1: 3.0 }, 0.3, QUAKE);
        let cracks = MarkKind::Cracks {
            dir,
            r: r.min(4.0),
            n: n(7.0) as u8,
            arc_deg: 360.0,
        };
        self.mark(g, cracks, 1.0, RIFT_ROOT);
        self.chips(g, dir, 2.5, 360.0, n(10.0));
        self.dust_ring(g, 0.8, n(12.0), 4.5, 0.7, DUST);
    }

    /// 뿌리 기둥 줄의 시작: 몸 앞의 쿵 + 줄을 따라 뻗는 금(기둥은 렌더).
    pub(super) fn blow_line(
        &mut self,
        len: f32,
        dir: Vec3,
        g: Vec3,
        s: f32,
        n: &impl Fn(f32) -> usize,
    ) {
        let c = g + dir * 1.2;
        self.star(c + Vec3::Y * 0.4, 1.8 * s, 0.12, WHITE_HOT);
        let cracks = MarkKind::Cracks {
            dir,
            r: len,
            n: 2,
            arc_deg: 8.0,
        };
        self.mark(g, cracks, 1.0, RIFT_ROOT);
        self.chips(c, dir, 1.6, 50.0, n(7.0));
        self.dust_kick(c, dir, n(6.0), 0.55);
    }

    /// 갈고리가 손을 떠난다: 손의 번쩍임 + 날아가는 쪽 빛줄(사슬은 렌더).
    pub(super) fn blow_hook(&mut self, dir: Vec3, g: Vec3, s: f32, n: &impl Fn(f32) -> usize) {
        let hand = g + Vec3::Y * 1.3 + dir * 0.8;
        self.star(hand, 1.1 * s, 0.08, WHITE_HOT);
        self.spray(
            hand,
            dir,
            0.2,
            n(6.0),
            7.0,
            Shape::Streak,
            0.06,
            0.2,
            0.0,
            WHITE_HOT,
            STEEL,
        );
    }

    /// 종 조각을 던졌다: 손의 번쩍임 + 던진 쪽 빛줄(조각은 월드의 탄).
    pub(super) fn blow_volley(&mut self, dir: Vec3, g: Vec3, s: f32, n: &impl Fn(f32) -> usize) {
        let hand = g + Vec3::Y * 1.3 + dir * 0.7;
        self.star(hand, 1.3 * s, 0.09, WHITE_HOT);
        self.spray(
            hand,
            dir,
            0.4,
            n(8.0),
            6.5,
            Shape::Streak,
            0.07,
            0.22,
            2.0,
            WHITE_HOT,
            STEEL,
        );
    }
}
