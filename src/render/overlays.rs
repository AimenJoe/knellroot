//! 덧그림: 탄·조각·폭발, 휘두름의 초승달과 박자 부채꼴, 모으기·강타, 스킬·큰 기술의 모양, 패링, 발밑 고리, 적 머리 위·락온 표시.
use super::*;

/// 탄의 색(0048·0051, `weapons::Bolt::look`): 마법 라벤더, 무거운 탄 놋쇠, 꿰뚫는 탄 민트, 쇠뇌 볼트 밝은 나무 — 붉은색(위험)은 쓰지 않는다.
/// 박자 타의 휘두름 색·발밑 고리가 퍼지는 시간(0076).
// 박자 금빛(0076 → 0098 사장 "너무 강한 노란색, 조금만 더 옅게"): 노랑을 흰빛 쪽으로(채도 낮춤) + 진하기 30% 안팎 내림.
const BEAT_GOLD: Color = Color::new(1.0, 0.9, 0.62, 0.24);
pub const BEAT_RING_LIFE: f32 = 0.35;
/// 딱 맞춰 떼기 고리가 퍼지는 시간(0078).
const PERFECT_RING_LIFE: f32 = 0.45;
const BOLT_LOOKS: [(Color, Color); 4] = [
    (
        Color::new(0.92, 0.88, 1.0, 1.0),
        Color::new(0.7, 0.6, 1.0, 0.45),
    ),
    (
        Color::new(1.0, 0.86, 0.55, 1.0),
        Color::new(0.9, 0.65, 0.3, 0.45),
    ),
    (
        Color::new(0.85, 1.0, 0.92, 1.0),
        Color::new(0.45, 0.9, 0.7, 0.45),
    ),
    (
        Color::new(0.95, 0.85, 0.65, 1.0),
        Color::new(0.8, 0.65, 0.4, 0.3),
    ),
];

impl Renderer {
    /// 배우 위에 얹는 것: 스윙 부채꼴·쟁반 테·휘청 별·락온 표시·마법 탄.
    pub(super) fn overlays(&self, w: &World, pp: Vec3, alpha: f32, t: &Tuning) {
        // 바닥 표시는 모두 땅 높낮이를 따라 접는다(0083 — 경사로·턱에서 묻히거나 뜨던 것).
        let fl = Drape {
            ground: w.ground.as_ref(),
        };
        bolts(w, alpha);
        bell_shards(w, &fl);
        bombs(w, alpha);
        cast_charge(w, pp, t, &fl);
        throw_aim(w, pp, t, &fl);
        cracks(w, &fl);
        blasts(w, &fl);
        let wpn = w.player.weapon.def();
        swing_crescent(w, pp, t, wpn);
        beat_fan(w, pp, t, wpn, &fl);
        beat_ring(w, pp, &fl);
        heavy_charge(w, pp, t, wpn, &fl);
        heavy_swing(w, pp, t, wpn, &fl);
        self.skill_shape(w, pp, t, &fl);
        ult_shape(w, pp, &fl);
        skill_zones(w, &fl);
        flask_glow(w, pp, t, &fl);
        parry_window(w, pp, t, &fl);
        parry_chain(w, pp, &fl);
        primed_heavy_rings(w, pp, &fl);
        flank_ring(w, pp, &fl);
        plunge_ring(w, pp, t, &fl);
        perfect_rings(w, pp, &fl);
        parry_ring(w, pp, &fl);
        exposed_rings(w, alpha, &fl);
        frail_rings(w, alpha, &fl);
        spill_rings(w, pp, t, &fl);
        stagger_stars(w, alpha);
        lock_mark(w, alpha, &fl);
    }

    fn skill_shape(&self, w: &World, pp: Vec3, t: &Tuning, fl: &Drape) {
        // 스킬(0151): 판정의 모양을 그대로 빛으로 — 바깥 변 = 닿는 거리(판정보다 크게 그리지 않는다, 0145). 금·민트·흰빛.
        if let PState::Skill { id, t: e, dir, .. } = w.player.state {
            use crate::skills::{Form, Twist};
            // 그 단의 실제 값(0158) — 넓어진 반지름·늘어난 바퀴·되돌아가는 길을 그대로 그린다.
            let eff = w.skill_eff(id);
            let d = eff.skill;
            let e0 = e - d.startup;
            let yaw = dir.x.atan2(dir.z);
            let gold = Color::new(1.0, 0.86, 0.5, 1.0);
            let mint = Color::new(0.66, 1.0, 0.86, 1.0);
            let fx = t[crate::tuning::P::RenderFx] >= 0.5;
            if e0 >= 0.0 {
                match d.form {
                    // 휘돌이: 몸과 함께 도는 긴 초승달(머리 = 지금 보는 쪽) + 둘레 원판.
                    Form::Ring {
                        radius, hits, gap, ..
                    } => {
                        let span = gap * hits as f32;
                        let fade = 1.0 - ((e0 - span) / SKILL_GLOW_FADE).clamp(0.0, 1.0);
                        fl.sector(
                            pp + Vec3::Y * 0.06,
                            radius,
                            0.0,
                            360.0,
                            with_a(gold, 0.16 * fade),
                        );
                        if fx {
                            // 머리 = 몸이 지금 보는 쪽: 전용 동작이면 동작의 한 바퀴(이어짐 구간을 바퀴마다)를, 아니면 월드의 쪽을 따른다.
                            let head = if self.whirl_clip {
                                let turns = (e0 / gap.max(1e-3)).min(hits.saturating_sub(1) as f32);
                                yaw + WHIRL_CLIP_TURN * turns * std::f32::consts::TAU
                            } else {
                                w.player.facing
                            };
                            crescent(
                                pp + Vec3::Y * 0.95,
                                radius,
                                head + std::f32::consts::PI * 0.75,
                                std::f32::consts::PI * 1.5,
                                crate::weapons::Sweep::LeftToRight,
                                1.0,
                                with_a(gold, fade * fade),
                                1.4,
                            );
                            // 둘째 겹(0158 G2): 반 바퀴 뒤를 따라오는 흰 날 — 한 바퀴 전체가 빛으로 쓸린다.
                            crescent(
                                pp + Vec3::Y * 0.6,
                                radius * 0.82,
                                head + std::f32::consts::PI * 1.75,
                                std::f32::consts::PI * 1.2,
                                crate::weapons::Sweep::LeftToRight,
                                1.0,
                                with_a(WHITE, 0.6 * fade * fade),
                                0.9,
                            );
                        }
                    }
                    // 가로지르기: 지나온 길의 빛 띠(머리 = 몸, 꼬리로 사라진다).
                    Form::Lane { dist, half, time } => {
                        // 되돌아 베기(0158 ㄱ): 둘째 구간은 온 길을 거꾸로 — 띠도 거꾸로 선다.
                        let back = eff.twist == Some(Twist::Return) && e0 >= time;
                        let (e0, dir) = if back { (e0 - time, -dir) } else { (e0, dir) };
                        let gone = (e0 / time.max(1e-3)).clamp(0.0, 1.0) * dist;
                        let fade = 1.0 - ((e0 - time) / SKILL_GLOW_FADE).clamp(0.0, 1.0);
                        if fx && gone > 0.05 {
                            let side = vec3(dir.z, 0.0, -dir.x);
                            let head = pp + Vec3::Y * 0.9;
                            let tail = head - dir * gone;
                            for (w2, ink, a) in [(half, mint, 0.5), (half * 0.28, WHITE, 0.9)] {
                                let on = with_a(ink, a * fade * fade);
                                let off = with_a(ink, 0.0);
                                flat_mesh(
                                    &[
                                        (tail - side * w2 * 0.4, off),
                                        (tail + side * w2 * 0.4, off),
                                        (head - side * w2, on),
                                        (head + side * w2, on),
                                    ],
                                    &[0, 1, 2, 1, 3, 2],
                                );
                            }
                        }
                    }
                    // 끌어모으기: 모이는 점으로 좁아지는 민트 고리 둘.
                    Form::Pull { range, to, .. } => {
                        let k = (e0 / 0.3).clamp(0.0, 1.0);
                        if k < 1.0 {
                            let spot = pp + dir * to;
                            for lag in [0.0, 0.35] {
                                let r = range * 0.7 * (1.0 - (k - lag).clamp(0.0, 1.0)) + 0.4;
                                fl.ring(
                                    spot + Vec3::Y * 0.06,
                                    r,
                                    0.0,
                                    360.0,
                                    with_a(mint, 0.85 * (1.0 - k)),
                                );
                            }
                        }
                    }
                    // 땅울림: 내리찍은 부채꼴 — 바닥의 빛 + 앞으로 터지는 굵은 초승달.
                    Form::Cone { range, arc_deg, .. } => {
                        let k = (e0 / 0.3).clamp(0.0, 1.0);
                        if k < 1.0 {
                            fl.sector(
                                pp + Vec3::Y * 0.06,
                                range,
                                yaw,
                                arc_deg,
                                with_a(gold, 0.3 * (1.0 - k)),
                            );
                            if fx {
                                crescent(
                                    pp + Vec3::Y * 0.5,
                                    range * (0.5 + 0.5 * k),
                                    yaw,
                                    arc_deg.to_radians(),
                                    crate::weapons::Sweep::Center,
                                    1.0,
                                    with_a(gold, (1.0 - k) * (1.0 - k)),
                                    1.5,
                                );
                            }
                        }
                    }
                    // 버팀(0158 G2): 터뜨릴 둘레를 옅게 깔고, 받아낸 타만큼 금빛 알이 몸을 돈다(조여드는 고리는 `fx::Mark`).
                    Form::Brace {
                        hold, radius, most, ..
                    } => {
                        if e0 < hold {
                            let k = (e0 / hold.max(1e-3)).clamp(0.0, 1.0);
                            fl.sector(
                                pp + Vec3::Y * 0.05,
                                radius,
                                0.0,
                                360.0,
                                with_a(gold, 0.05 + 0.09 * k),
                            );
                            fl.ring(
                                pp + Vec3::Y * 0.06,
                                radius,
                                0.0,
                                360.0,
                                with_a(gold, 0.3 + 0.3 * k),
                            );
                            // 몸을 감싼 방패 빛: 가슴 높이의 짧은 호 셋이 천천히 돈다.
                            for i in 0..3 {
                                let a = w.time * 2.4 + i as f32 * std::f32::consts::TAU / 3.0;
                                fl.ring(
                                    pp + Vec3::Y * 0.9,
                                    1.05,
                                    a,
                                    70.0,
                                    Color::new(1.0, 0.97, 0.86, 0.55),
                                );
                            }
                            let n = w.player.brace_soaked.min(most);
                            for i in 0..n {
                                let a =
                                    -w.time * 5.0 + i as f32 * std::f32::consts::TAU / most as f32;
                                let p = pp + vec3(a.sin() * 0.8, 1.25, a.cos() * 0.8);
                                gem(p, 0.42, with_a(gold, 0.45));
                                gem(p, 0.22, Color::new(1.0, 0.97, 0.86, 1.0));
                            }
                        }
                    }
                    // 되울림·던지기·잔불 흩기: 몸에서 떠난 것은 남는 빛·이어지는 것(`marks`·`skill_auras`)이 그린다.
                    Form::Echo { .. } | Form::Throw { .. } | Form::Field { .. } => {}
                }
            }
        }
    }
}

fn bolts(w: &World, alpha: f32) {
    // 마법 탄(0048): 라벤더 빛 덩이 + 짧은 꼬리(보간 자리).
    for b in &w.bolts {
        let p = b.prev.lerp(b.pos, alpha);
        let (core, glow) = BOLT_LOOKS[(b.def.look as usize).min(BOLT_LOOKS.len() - 1)];
        let k = if b.def.look == 3 { 0.6 } else { 1.0 } * b.def.scale;
        // 0141: 정육면체 대신 카메라를 보는 마름모 — 빛무리 + 알맹이 + 뒤로 작아지는 꼬리 셋.
        for i in 1..=3 {
            let f = i as f32;
            gem(
                p - b.dir * (0.3 * f * k),
                (0.42 - 0.09 * f) * k,
                with_a(glow, glow.a * (1.0 - 0.28 * f)),
            );
        }
        gem(p, 0.78 * k, glow);
        gem(p, 0.42 * k, core);
    }
}

fn bell_shards(w: &World, fl: &Drape) {
    // 종 조각(0125): 돌며 나는 청동 조각 — 되받아친 것은 호박빛(패링이 "돌려보냈다"로 읽히게).
    for sh in &w.shards {
        let spin = w.time * 14.0 + sh.went;
        let rot = Quat::from_rotation_y(spin) * Quat::from_rotation_x(spin * 0.7);
        let (core, glow) = if sh.reflected {
            (
                Color::new(1.0, 0.78, 0.38, 1.0),
                Color::new(1.0, 0.7, 0.3, 0.35),
            )
        } else {
            (
                Color::new(0.62, 0.48, 0.30, 1.0),
                Color::new(0.85, 0.55, 0.35, 0.22),
            )
        };
        let a = rot * vec3(0.22, 0.0, 0.0);
        let b = rot * vec3(0.0, 0.0, 0.16);
        draw_line_3d(sh.pos - a, sh.pos + a, core);
        draw_line_3d(sh.pos - b, sh.pos + b, core);
        gem(sh.pos - sh.dir * 0.3, 0.3, with_a(glow, glow.a * 0.7));
        gem(sh.pos, 0.56, glow);
        gem(sh.pos, 0.3, core);
        fl.shadow(sh.pos, 0.18);
    }
}

fn bombs(w: &World, alpha: f32) {
    // 수류탄(0051): 어두운 덩이 + 도화선이 짧아질수록 빨리 깜빡이는 불.
    for bm in &w.bombs {
        let p = bm.prev.lerp(bm.pos, alpha);
        gem(p, 0.4, Color::new(0.22, 0.19, 0.24, 1.0));
        let hz = 4.0 + 10.0 * (1.0 - (bm.fuse / 1.5).clamp(0.0, 1.0));
        if (w.time * hz) as i32 % 2 == 0 {
            spark4(p + Vec3::Y * 0.22, 0.36, Color::new(1.0, 0.8, 0.4, 1.0));
        }
    }
}

fn cast_charge(w: &World, pp: Vec3, t: &Tuning, fl: &Drape) {
    // 원거리 강공격 모으기(0056): 손에 빛이 자라고 발밑 고리가 차오른다(근접 강공격과 같은 문법), 다 모이면 깜빡인다.
    if let PState::CastCharge { t: e } = w.player.state {
        let full = t[crate::tuning::P::CastCharge].max(1e-3);
        let k = (e / full).clamp(0.0, 1.0);
        let (core, glow) = BOLT_LOOKS[(wpn_look(w) as usize).min(BOLT_LOOKS.len() - 1)];
        let hand = pp
            + Vec3::Y * (w.player.air + crate::tuning::BOLT_HEIGHT)
            + w.player.facing_dir() * 0.45;
        let size = 0.15 + 0.35 * k;
        gem(hand, size * 2.1, glow);
        gem(hand, size * 1.25, core);
        let col = if k >= 1.0 && (w.time * 12.0) as i32 % 2 == 0 {
            with_a(core, 0.95)
        } else {
            with_a(glow, 0.3 + 0.5 * k)
        };
        fl.ring(pp + Vec3::Y * 0.05, 0.9, 0.0, 360.0, with_a(glow, 0.25));
        fl.ring(pp + Vec3::Y * 0.05, 0.2 + 0.7 * k, 0.0, 360.0, col);
    }
}

fn throw_aim(w: &World, pp: Vec3, t: &Tuning, fl: &Drape) {
    // 수류탄 겨누기(0055): 떨굴 자리에 폭발 반지름 고리.
    if let PState::Throw {
        to,
        thrown: false,
        t: e,
    } = w.player.state
    {
        if let Some(g) = w.player.weapon.def().grenade {
            if e >= t[crate::tuning::P::ThrowStartup] * 0.99 {
                let y = w.ground.as_ref().map_or(0.0, |gr| gr.height(to));
                let c = vec3(to.x, y + 0.08, to.z);
                fl.ring(c, g.radius, 0.0, 360.0, Color::new(1.0, 0.82, 0.45, 0.8));
                draw_line_3d(pp + Vec3::Y * 1.0, c, Color::new(1.0, 0.82, 0.45, 0.35));
            }
        }
    }
}

fn cracks(w: &World, fl: &Drape) {
    // 금 간 땅(망치 여진): 터질 때가 가까울수록 진해지는 고리.
    for c in &w.cracks {
        let k = (1.0 - c.left / 1.5).clamp(0.0, 1.0);
        fl.ring(
            c.pos + Vec3::Y * 0.06,
            c.radius,
            0.0,
            360.0,
            Color::new(1.0, 0.75, 0.4, 0.2 + 0.5 * k),
        );
        fl.ring(
            c.pos + Vec3::Y * 0.06,
            c.radius * k,
            0.0,
            360.0,
            Color::new(1.0, 0.85, 0.5, 0.35),
        );
    }
}

fn blasts(w: &World, fl: &Drape) {
    // 맞힌 자리의 불똥(2026-11-06)은 입자와 함께 그린다(`Backdrop::fx`, 0141 — 두 화소 굵기의 빛줄).
    // 폭발 고리: 퍼지며 옅어진다.
    for b in &w.blasts {
        let k = (b.age / crate::tuning::BLAST_SHOW).clamp(0.0, 1.0);
        fl.ring(
            b.pos + Vec3::Y * 0.1,
            b.radius * (0.4 + 0.6 * k),
            0.0,
            360.0,
            Color::new(1.0, 0.8, 0.45, 0.9 * (1.0 - k)),
        );
    }
}

fn swing_crescent(w: &World, pp: Vec3, t: &Tuning, wpn: &crate::weapons::Weapon) {
    // 휘두름의 초승달(0141): 활성 구간 동안 칼끝이 지난 자리에 밝은 초승달이 따라가고 0.12초에 걸쳐 옅어진다 — 무기가 지나간 호
    // (판정 부채꼴은 아래 박자 부채꼴이 그대로 맡는다). 박자 타면 금빛.
    if t[crate::tuning::P::RenderFx] >= 0.5 {
        if let PState::Attack { t: e, hit, .. } = w.player.state {
            // 몰아치기(0158 G2) 동안은 평타의 시계가 빨라진다 — 빛도 같은 배로.
            let e = e * w.flurry_rate();
            let (s0, a0, _) = crate::world::hit_times(t, wpn, hit);
            if let Some(h) = wpn.hit(hit).filter(|_| e >= s0 && e < s0 + a0 + SLASH_FADE) {
                let q = ((e - s0) / a0.max(1e-3)).clamp(0.0, 1.0);
                let fade = 1.0 - ((e - s0 - a0) / SLASH_FADE).clamp(0.0, 1.0);
                let arc = if h.shape == crate::weapons::Shape::Circle {
                    360.0
                } else {
                    t[crate::tuning::P::AttackArc] * h.arc
                };
                let col = if w.player.swing_beat {
                    Color::new(1.0, 0.88, 0.55, 0.95)
                } else {
                    Color::new(0.92, 0.95, 1.0, 0.9)
                };
                crescent(
                    pp + Vec3::Y * 0.95,
                    t[crate::tuning::P::AttackRange] * h.reach,
                    w.player.facing,
                    arc.to_radians(),
                    crate::weapons::sweep_of(h.anim),
                    q,
                    with_a(col, col.a * fade),
                    1.0,
                );
            }
        }
    }
}

/// 박자 부채꼴(0087) — 이을 타가 없는 타는 활성 동안 판정 부채꼴만.
fn beat_fan(w: &World, pp: Vec3, t: &Tuning, wpn: &crate::weapons::Weapon, fl: &Drape) {
    let swing_col = if w.player.swing_beat {
        BEAT_GOLD
    } else {
        Color::new(1.0, 0.95, 0.7, 0.18)
    };
    if let Some(cue) = w.beat_cue(t) {
        // 박자 부채꼴(0087): 휘두른 부채꼴이 다음 박자까지 줄어든다 — 가장 작을 때가 박자. 칼이 지나간 쪽부터 사라진다
        // (오른쪽 → 왼쪽 베기는 오른쪽부터, 왼쪽 → 오른쪽은 왼쪽부터, 찌르기·세로는 양 가장자리부터). 창 안이면 금빛 + 테두리.
        // 캐릭터의 오른쪽 = 바라보는 쪽 − 90°(부채꼴의 작은 각 쪽).
        let gone = (cue.arc * (1.0 - cue.left) * 0.5).to_radians();
        let yaw = w.player.facing
            + match cue.sweep {
                crate::weapons::Sweep::RightToLeft => gone,
                crate::weapons::Sweep::LeftToRight => -gone,
                crate::weapons::Sweep::Center => 0.0,
            };
        let arc = cue.arc * cue.left;
        let c = pp + Vec3::Y * 0.9;
        if cue.ripe {
            fl.sector(c, cue.reach, yaw, arc, Color::new(1.0, 0.92, 0.64, 0.3));
            fl.ring(c, cue.reach, yaw, arc, Color::new(1.0, 0.95, 0.76, 0.6));
        } else {
            fl.sector(c, cue.reach, yaw, arc, swing_col);
        }
    } else if let PState::Attack { t: e, hit, .. } = w.player.state {
        // 이을 타가 없는 타(마지막 타)는 박자가 없다 — 활성 동안 판정 부채꼴만.
        let (s0, a0, _) = crate::world::hit_times(t, wpn, hit);
        if let Some(h) = wpn.hit(hit) {
            if e >= s0 && e < s0 + a0 {
                let arc = if h.shape == crate::weapons::Shape::Circle {
                    360.0
                } else {
                    t[crate::tuning::P::AttackArc] * h.arc
                };
                fl.sector(
                    pp + Vec3::Y * 0.9,
                    t[crate::tuning::P::AttackRange] * h.reach,
                    w.player.facing,
                    arc,
                    swing_col,
                );
            }
        }
    }
}

fn beat_ring(w: &World, pp: Vec3, fl: &Drape) {
    // 박자 타(0076, KB nail 0168): 발밑에서 금빛 고리가 퍼진다 — 잇달을수록 크게.
    if w.player.since_beat < BEAT_RING_LIFE {
        let k = (w.player.since_beat / BEAT_RING_LIFE).clamp(0.0, 1.0);
        let big = 1.0 + 0.25 * (w.player.beat_chain.min(3) as f32 - 1.0);
        let ease = 1.0 - (1.0 - k) * (1.0 - k);
        fl.ring(
            pp + Vec3::Y * 0.06,
            (0.5 + 1.3 * ease) * big,
            0.0,
            360.0,
            Color::new(1.0, 0.91, 0.62, 0.65 * (1.0 - k)),
        );
        fl.ring(
            pp + Vec3::Y * 0.07,
            (0.35 + 0.9 * ease) * big,
            0.0,
            360.0,
            Color::new(1.0, 0.96, 0.8, 0.45 * (1.0 - k)),
        );
    }
}

fn heavy_charge(w: &World, pp: Vec3, t: &Tuning, wpn: &crate::weapons::Weapon, fl: &Drape) {
    // 강공격(0030): 모으는 동안 발밑 고리가 안쪽에서 차오르고, 다 차면 금빛으로 깜빡인다. 강타의 활성 구간엔 넓은 부채꼴.
    if let PState::Charging { t: e } = w.player.state {
        let full =
            (t[crate::tuning::P::HeavyCharge] * wpn.heavy.map_or(1.0, |h| h.charge)).max(1e-3);
        let k = (e / full).clamp(0.0, 1.0);
        let col = if k >= 1.0 {
            let on = (w.time * 12.0) as i32 % 2 == 0;
            Color::new(1.0, 0.85, 0.35, if on { 0.95 } else { 0.55 })
        } else {
            Color::new(1.0, 0.7, 0.4, 0.35 + 0.4 * k)
        };
        fl.ring(
            pp + Vec3::Y * 0.05,
            0.9,
            0.0,
            360.0,
            with_a(col, col.a * 0.5),
        );
        fl.ring(pp + Vec3::Y * 0.05, 0.2 + 0.7 * k, 0.0, 360.0, col);
    }
}

/// 강타의 활성 구간: 넓은 부채꼴 + 초승달(0145).
fn heavy_swing(w: &World, pp: Vec3, t: &Tuning, wpn: &crate::weapons::Weapon, fl: &Drape) {
    if let (PState::Heavy { t: e, full, .. }, Some(h)) = (w.player.state, wpn.heavy) {
        let (s0, a0, _) = crate::world::heavy_times(t, wpn);
        let (s1, arc) = (
            s0 + a0,
            if h.shape == crate::weapons::Shape::Circle {
                360.0
            } else {
                t[crate::tuning::P::HeavyArc] * h.arc
            },
        );
        if e >= s0 && e < s1 {
            fl.sector(
                pp + Vec3::Y * 0.9,
                t[crate::tuning::P::HeavyRange] * h.reach,
                w.player.facing,
                arc,
                if full {
                    Color::new(1.0, 0.8, 0.4, 0.25)
                } else {
                    Color::new(1.0, 0.9, 0.65, 0.18)
                },
            );
        }
        // 강타의 초승달(0145): 평타보다 두껍고 오래 남는다 — 다 모았으면 호박빛. 강타는 가운데에서 양쪽으로 벌어진다.
        if t[crate::tuning::P::RenderFx] >= 0.5 && e >= s0 && e < s1 + HEAVY_SLASH_FADE {
            let q = ((e - s0) / a0.max(1e-3)).clamp(0.0, 1.0);
            let fade = 1.0 - ((e - s1) / HEAVY_SLASH_FADE).clamp(0.0, 1.0);
            let col = if full {
                Color::new(1.0, 0.78, 0.4, 1.0)
            } else {
                Color::new(1.0, 0.93, 0.75, 0.9)
            };
            crescent(
                pp + Vec3::Y * 0.95,
                t[crate::tuning::P::HeavyRange] * h.reach,
                w.player.facing,
                arc.to_radians(),
                crate::weapons::Sweep::Center,
                q,
                with_a(col, col.a * fade * fade),
                if full { 1.5 } else { 1.25 },
            );
        }
    }
}

fn ult_shape(w: &World, pp: Vec3, fl: &Drape) {
    // 큰 기술(0151 → 0158 G2, 끼운 하나). 쟁반 뒤집기: 시동은 발밑으로 모이는 금빛 고리, 뒤집히는 순간부터 둘레 끝까지 퍼지는 고리 둘.
    // 몰아치기: 시동 동안 발밑으로 조여드는 금빛 고리. 한 줄기: 가르기 전에 가를 길의 테가 먼저 선다(세계가 멈춘 동안 읽힌다).
    if let PState::Ult { kind, t: e, done } = w.player.state {
        let gold = Color::new(1.0, 0.86, 0.5, 1.0);
        match kind {
            crate::skills::Ult::Upend => {
                let u = &crate::skills::UPEND;
                if !done {
                    let k = (e / u.startup.max(1e-3)).clamp(0.0, 1.0);
                    fl.ring(
                        pp + Vec3::Y * 0.06,
                        2.5 * (1.0 - k) + 0.3,
                        0.0,
                        360.0,
                        with_a(gold, 0.9),
                    );
                } else {
                    let k = ((e - u.startup) / u.recover.max(1e-3)).clamp(0.0, 1.0);
                    for lag in [0.0, 0.25] {
                        let q = (k * 1.6 - lag).clamp(0.0, 1.0);
                        fl.ring(
                            pp + Vec3::Y * 0.06,
                            u.radius * q,
                            0.0,
                            360.0,
                            with_a(gold, 0.9 * (1.0 - k)),
                        );
                    }
                    fl.sector(
                        pp + Vec3::Y * 0.05,
                        u.radius,
                        0.0,
                        360.0,
                        with_a(gold, 0.12 * (1.0 - k)),
                    );
                }
            }
            crate::skills::Ult::Flurry => {
                let u = &crate::skills::FLURRY;
                if !done {
                    let k = (e / u.startup.max(1e-3)).clamp(0.0, 1.0);
                    for lag in [0.0, 0.4] {
                        let q = (k * 1.4 - lag).clamp(0.0, 1.0);
                        fl.ring(
                            pp + Vec3::Y * 0.06,
                            2.2 * (1.0 - q) + 0.5,
                            0.0,
                            360.0,
                            with_a(gold, 0.85 * q.max(0.25)),
                        );
                    }
                }
            }
            crate::skills::Ult::Line => {
                let u = &crate::skills::LINE;
                if !done {
                    let dir = w.player.facing_dir();
                    let len = u.len_at(w.player.ult_rank);
                    let frozen = 1.0 - (w.hitstop / u.freeze.max(1e-3)).clamp(0.0, 1.0);
                    let k = if e > 0.0 { 1.0 } else { frozen };
                    let at = pp + Vec3::Y * 0.06;
                    fl.lane(at, dir, len * k, u.half * 2.0, with_a(gold, 0.10 + 0.1 * k));
                    fl.lane_edge(at, dir, len * k, u.half * 2.0, with_a(gold, 0.75));
                    // 칼끝에 모이는 빛.
                    let tip = pp + Vec3::Y * 1.3 + dir * 0.5;
                    gem(tip, 0.3 + 0.7 * k, with_a(gold, 0.5));
                    spark4(tip, 0.5 + 1.1 * k, Color::new(1.0, 1.0, 0.94, 1.0));
                }
            }
        }
    }
}

fn skill_zones(w: &World, fl: &Drape) {
    // 갈라진 땅(0158 G1, 땅울림 ㄴ): 남아 있는 동안 바닥의 부채꼴(면은 옅게·테는 또렷하게 — 판정 그대로), 끝에서 사라진다.
    for z in &w.skill_zones {
        let k = (z.left / z.total.max(1e-3)).clamp(0.0, 1.0);
        let a = (k / 0.3).min(1.0);
        let yaw = z.dir.x.atan2(z.dir.z);
        let dust = Color::new(0.86, 0.78, 0.62, 1.0);
        let at = vec3(z.pos.x, w.ground_y(z.pos), z.pos.z);
        fl.sector(
            at + Vec3::Y * 0.05,
            z.range,
            yaw,
            z.arc_deg,
            with_a(dust, 0.22 * a),
        );
        fl.ring(
            at + Vec3::Y * 0.06,
            z.range,
            yaw,
            z.arc_deg,
            with_a(dust, 0.7 * a),
        );
    }
}

fn flask_glow(w: &World, pp: Vec3, t: &Tuning, fl: &Drape) {
    // 등잔(0031): 삼키는 순간 따뜻한 빛이 발밑에서 퍼진다.
    if let PState::Drink { t: e, gulped: true } = w.player.state {
        let k = ((e - t[crate::tuning::P::FlaskRaise]) / 0.35).clamp(0.0, 1.0);
        if k < 1.0 {
            fl.ring(
                pp + Vec3::Y * (0.05 + 0.8 * k),
                0.5 + 0.6 * k,
                0.0,
                360.0,
                Color::new(1.0, 0.82, 0.45, 0.8 * (1.0 - k)),
            );
        }
    }
}

fn parry_window(w: &World, pp: Vec3, t: &Tuning, fl: &Drape) {
    // 패링(0025): 창이 열린 동안 앞에 옅은 푸른 반달, 성공하면 둘레로 퍼지는 흰 고리.
    // 모으기 공격을 뗀 직후의 패링 창(0089)도 같은 반달 — 떼는 순간 "막기도 한다"가 보인다.
    let guard = w.player.release_guard > 0.0
        && matches!(w.player.state, PState::Heavy { .. } | PState::Cast { .. });
    let window = match w.player.state {
        PState::Parry { t: e } => e < t[crate::tuning::P::ParryWindow],
        _ => guard,
    };
    if window {
        fl.sector(
            pp + Vec3::Y * 0.8,
            1.4,
            w.player.facing,
            150.0,
            Color::new(0.6, 0.85, 1.0, 0.15),
        );
        fl.ring(
            pp + Vec3::Y * 0.8,
            1.4,
            w.player.facing,
            150.0,
            Color::new(0.75, 0.9, 1.0, 0.4),
        );
    }
}

fn parry_chain(w: &World, pp: Vec3, fl: &Drape) {
    // 이어 패링(0081·0086): 열린 동안 같은 푸른 반달을 지금 받는 넓이로 — 쌓일수록 둘레로 벌어진다(둘레 전부면 고리).
    // 맞서는 자(0108): 못 받는 공격까지 막는 동안은 호박색 — 지금 파동도 막는다는 표시.
    if w.player.parry_chain_left > 0.0 && !matches!(w.player.state, PState::Parry { .. }) {
        let stand = w.tree.has(Cell::OneWhoStands)
            && w.player.parry_chain_n >= crate::progression::cell::STAND_CHAIN;
        let (fill_c, edge_c) = if stand {
            ((1.0, 0.78, 0.4), (1.0, 0.85, 0.5))
        } else {
            ((0.6, 0.85, 1.0), (0.75, 0.9, 1.0))
        };
        let c = crate::world::parry_front_cos(true, w.player.parry_chain_n);
        let arc = if c < -1.0 {
            360.0
        } else {
            (c.acos().to_degrees() * 2.0).min(360.0)
        };
        let fade = (w.player.parry_chain_left / crate::tuning::PARRY_CHAIN).min(1.0);
        fl.sector(
            pp + Vec3::Y * 0.8,
            1.4,
            w.player.facing,
            arc,
            Color::new(fill_c.0, fill_c.1, fill_c.2, 0.12 * fade),
        );
        fl.ring(
            pp + Vec3::Y * 0.8,
            1.4,
            w.player.facing,
            arc,
            Color::new(edge_c.0, edge_c.1, edge_c.2, 0.35 * fade),
        );
    }
}

fn primed_heavy_rings(w: &World, pp: Vec3, fl: &Drape) {
    // 끝맺음(0108): 다음 강타가 다 모여 있는 동안 발밑에 금빛 고리 두 겹이 숨 쉰다(딱 맞춤 고리와 같은 색 — "모였다").
    if w.player.primed_heavy {
        let k = 0.5 + 0.5 * (w.time * 4.0).sin();
        for (r, a) in [(0.75, 0.8), (1.05, 0.45)] {
            fl.ring(
                pp + Vec3::Y * 0.06,
                r + 0.08 * k,
                0.0,
                360.0,
                Color::new(1.0, 0.88, 0.45, a * (0.6 + 0.4 * k)),
            );
        }
    }
}

fn flank_ring(w: &World, pp: Vec3, fl: &Drape) {
    // 비껴서기(0108): 적 등 뒤를 잡은 대시 공격 창 동안 발밑에 민트 고리가 빠르게 뛴다 — 지금 치면 자세 두 배.
    if w.player.flank {
        let k = 0.5 + 0.5 * (w.time * 14.0).sin();
        fl.ring(
            pp + Vec3::Y * 0.06,
            0.9,
            0.0,
            360.0,
            Color::new(0.65, 0.95, 0.8, 0.5 + 0.4 * k),
        );
    }
}

fn plunge_ring(w: &World, pp: Vec3, t: &Tuning, fl: &Drape) {
    // 내려찍기(0027): 내려앉은 자리에서 범위만큼 퍼지는 고리(`plunge.range m`).
    if let PState::Land { t: e, dur } = w.player.state {
        if (dur - t[crate::tuning::P::PlungeLand]).abs() < 1e-4 && e < 0.25 {
            let k = e / 0.25;
            let r = t[crate::tuning::P::PlungeRange] * (0.4 + 0.6 * k);
            fl.ring(
                pp,
                r,
                0.0,
                360.0,
                Color::new(1.0, 0.85, 0.55, 0.9 * (1.0 - k)),
            );
        }
    }
}

fn perfect_rings(w: &World, pp: Vec3, fl: &Drape) {
    // 딱 맞춰 떼기(0078): 발밑에서 크게 퍼지는 금빛 고리 둘(박자 고리보다 넓고 길게).
    if w.player.since_perfect < PERFECT_RING_LIFE {
        let k = (w.player.since_perfect / PERFECT_RING_LIFE).clamp(0.0, 1.0);
        let ease = 1.0 - (1.0 - k) * (1.0 - k);
        for (r0, r1, a) in [(0.6, 3.2, 0.95), (0.4, 2.2, 0.6)] {
            fl.ring(
                pp + Vec3::Y * 0.08,
                r0 + (r1 - r0) * ease,
                0.0,
                360.0,
                Color::new(1.0, 0.9, 0.5, a * (1.0 - k)),
            );
        }
    }
}

/// 패링 성공: 둘레로 퍼지는 흰 고리.
fn parry_ring(w: &World, pp: Vec3, fl: &Drape) {
    if w.player.since_parry < 0.35 {
        let k = (w.player.since_parry / 0.35).clamp(0.0, 1.0);
        fl.ring(
            pp + Vec3::Y * 0.05,
            0.6 + 2.4 * k,
            0.0,
            360.0,
            Color::new(0.85, 0.95, 1.0, 0.55 * (1.0 - k)),
        );
    }
}

fn exposed_rings(w: &World, alpha: f32, fl: &Drape) {
    // 드러난 적(치명타 기회): 발밑에 금빛 고리가 깜빡인다.
    for en in &w.enemies {
        if en.exposed > 0.0 && en.alive() && (w.time * 10.0) as i32 % 2 == 0 {
            let p = en.prev.lerp(en.pos, alpha);
            fl.ring(
                p,
                en.radius + 0.45,
                0.0,
                360.0,
                Color::new(1.0, 0.82, 0.35, 0.85),
            );
        }
    }
}

fn frail_rings(w: &World, alpha: f32, fl: &Drape) {
    // 더 다치는 적(0158 G1, 끌어모으기 맺음): 발밑에 민트빛 고리 — 남은 시간만큼 옅어진다(드러난 적의 금빛 고리보다 좁게).
    for en in &w.enemies {
        if en.frail > 0.0 && en.alive() {
            let p = en.prev.lerp(en.pos, alpha);
            fl.ring(
                p + Vec3::Y * 0.04,
                en.radius + 0.25,
                0.0,
                360.0,
                Color::new(0.66, 1.0, 0.86, 0.3 + 0.5 * en.frail.min(1.0)),
            );
        }
    }
}

fn spill_rings(w: &World, pp: Vec3, t: &Tuning, fl: &Drape) {
    // 땅 돌리기(0012): 쟁반의 테 — 멈추는 순간 범위 고리가 밝게 떴다 옅어지고, 안쪽 물결이 바깥으로 퍼진다.
    let shown = 0.5;
    if w.player.since_spill < shown {
        let k = (w.player.since_spill / shown).clamp(0.0, 1.0);
        let range = t[crate::tuning::P::SpillRange];
        fl.ring(
            pp,
            range,
            0.0,
            360.0,
            Color::new(0.7, 0.95, 0.75, 0.8 * (1.0 - k)),
        );
        fl.ring(
            pp,
            range * (0.25 + 0.75 * k),
            0.0,
            360.0,
            Color::new(0.7, 0.95, 0.75, 0.5 * (1.0 - k)),
        );
    }
}

fn stagger_stars(w: &World, alpha: f32) {
    // 휘청: 머리 위를 도는 작은 별 셋.
    for en in &w.enemies {
        if let EState::Stagger { t: e, dur } = en.state {
            let p = en.prev.lerp(en.pos, alpha);
            let fade = ((dur - e) / 0.2).clamp(0.0, 1.0);
            for i in 0..3 {
                let a = w.time * 6.0 + i as f32 * std::f32::consts::TAU / 3.0;
                let r = en.radius + 0.15;
                spark4(
                    p + vec3(a.cos() * r, en.height + 0.3, a.sin() * r),
                    0.34,
                    Color::new(1.0, 0.92, 0.45, fade),
                );
            }
        }
    }
}

fn lock_mark(w: &World, alpha: f32, fl: &Drape) {
    // 락온 표시: 대상 머리 위 작은 마름모 + 땅의 고리.
    if let Some(tg) = w.lock.and_then(|i| w.enemies.get(i)).filter(|e| e.alive()) {
        let p = tg.prev.lerp(tg.pos, alpha);
        let bob = (w.time * 4.0).sin() * 0.1;
        let top = p + Vec3::Y * (tg.height + 0.9 + bob);
        let s = 0.25;
        for (a, b) in [
            (Vec3::X, Vec3::Y),
            (Vec3::Y, -Vec3::X),
            (-Vec3::X, -Vec3::Y),
            (-Vec3::Y, Vec3::X),
        ] {
            line(top + a * s, top + b * s, YELLOW);
        }
        fl.ring(
            p,
            tg.radius + 0.3,
            0.0,
            360.0,
            Color::new(1.0, 0.9, 0.3, 0.8),
        );
    }
}

/// 휘두름의 초승달이 활성 구간 뒤에 옅어지는 시간(초).
const SLASH_FADE: f32 = 0.2;
/// 강타의 초승달이 남는 시간(초).
const HEAVY_SLASH_FADE: f32 = 0.3;

/// 초승달이 덮는 각(라디안): `(꼬리, 머리)` 쌍들. 머리 = 칼끝이 지금 있는 각, 꼬리 = 지나온 쪽(길어야 `SLASH_TAIL`).
/// 가로 베기는 한쪽 끝에서 반대쪽으로(캐릭터의 오른쪽 = 바라보는 쪽 − 반각), 찌르기·세로는 가운데에서 양쪽으로.
pub fn slash_arcs(yaw: f32, arc: f32, sweep: crate::weapons::Sweep, q: f32) -> Vec<(f32, f32)> {
    use crate::weapons::Sweep;
    let half = arc * 0.5;
    let q = q.clamp(0.0, 1.0);
    match sweep {
        Sweep::RightToLeft => {
            let head = yaw - half + arc * q;
            vec![((head - SLASH_TAIL).max(yaw - half), head)]
        }
        Sweep::LeftToRight => {
            let head = yaw + half - arc * q;
            vec![((head + SLASH_TAIL).min(yaw + half), head)]
        }
        Sweep::Center => vec![(yaw, yaw + half * q), (yaw, yaw - half * q)],
    }
}
const SLASH_TAIL: f32 = 2.2;

/// 휘돌이의 전용 동작이 몸을 도는 쪽(+1 = 요가 느는 쪽 — 월드의 도는 쪽과 같다).
const WHIRL_CLIP_TURN: f32 = 1.0;

/// 스킬의 빛(0151)이 판정 뒤에 남는 초.
const SKILL_GLOW_FADE: f32 = 0.25;
/// 초승달의 두께(머리에서, 닿는 거리의 몫): 빛 띠·그 안의 밝은 날 — `weight`를 곱한다(강타는 더 두껍게, 0145).
pub(super) const SLASH_GLOW: f32 = 0.6;

pub(super) const SLASH_EDGE: f32 = 0.16;

/// 초승달의 안쪽 반지름 몫(0..1): 머리(`s` = 1)에서 `thick × weight`만큼 두껍고 꼬리(`s` = 0)에서 0 — 반지름의 0.15보다 안으로는
/// 들어가지 않는다(가운데가 메워지면 초승달이 아니라 원판이다).
pub fn slash_inner(s: f32, thick: f32, weight: f32) -> f32 {
    (1.0 - thick * weight * s.clamp(0.0, 1.0)).max(0.15)
}

/// 초승달: 머리 쪽이 두껍고 밝고, 꼬리로 갈수록 가늘어지며 사라진다(바깥 변 = 닿는 거리 — 판정보다 크게 그리지 않는다).
/// 두 겹(0145 — 사장 "더 큰 휘두름"): 넓은 빛 띠(무기 빛깔, 안쪽으로 옅어진다) + 바깥 변을 따라가는 밝은 날(흰빛).
/// `weight` = 두께 배율(평타 1, 강타 1.25~1.5).
#[allow(clippy::too_many_arguments)]
pub(super) fn crescent(
    c: Vec3,
    reach: f32,
    yaw: f32,
    arc: f32,
    sweep: crate::weapons::Sweep,
    q: f32,
    col: Color,
    weight: f32,
) {
    const N: usize = 14;
    let hot = Color::new(1.0, 1.0, 0.96, col.a);
    for (tail, head) in slash_arcs(yaw, arc, sweep, q) {
        if (head - tail).abs() < 0.02 {
            continue;
        }
        // (두께, 바깥 빛, 안쪽 빛의 투명도 몫): 빛 띠는 안으로 사라지고, 날은 고르게 밝다.
        for (thick, ink, inner_a) in [(SLASH_GLOW, col, 0.12), (SLASH_EDGE, hot, 0.85)] {
            let mut pts = Vec::with_capacity((N + 1) * 2);
            let mut idx = Vec::with_capacity(N * 6);
            for i in 0..=N {
                let s = i as f32 / N as f32;
                let a = tail + (head - tail) * s;
                let dir = vec3(a.sin(), 0.0, a.cos());
                // 꼬리 쪽도 한동안 보이게(예전 s^1.5는 머리 근처만 남았다).
                let k = ink.a * s;
                pts.push((c + dir * reach, with_a(ink, k)));
                pts.push((
                    c + dir * reach * slash_inner(s, thick, weight),
                    with_a(ink, k * inner_a),
                ));
            }
            for i in 0..N as u16 {
                let b = i * 2;
                idx.extend_from_slice(&[b, b + 1, b + 2, b + 1, b + 3, b + 2]);
            }
            flat_mesh(&pts, &idx);
        }
    }
}

/// 든 무기의 모은 탄 색(0056).
fn wpn_look(w: &World) -> u8 {
    w.player.weapon.def().charged.look
}
