//! 스킬이 남긴 빛(`fx::Mark`, 0158 G2)과 이어지는 것들(울림·불티·몰아치기의 기운).
use super::*;

impl Renderer {
    /// 스킬이 남긴 빛(0158 G2 — `fx::Mark`): 고리·빛 띠·충격파·땅의 금·모여드는 줄·가르는 면. 바닥의 것은 땅을 따라 접는다.
    pub(super) fn marks(&self, w: &World, marks: &[crate::fx::Mark], pp: Vec3) {
        use crate::fx::MarkKind;
        let fl = Drape {
            ground: w.ground.as_ref(),
        };
        let hot = Color::new(1.0, 1.0, 0.94, 1.0);
        for m in marks.iter().filter(|m| m.live()) {
            let k = m.k();
            let at = if m.follow {
                vec3(pp.x, w.ground_y(pp), pp.z)
            } else {
                m.pos
            };
            let out = 1.0 - (1.0 - k) * (1.0 - k);
            match m.kind {
                MarkKind::Ring { r0, r1 } => {
                    let r = r0 + (r1 - r0) * out;
                    // 조여드는 고리는 끝으로 갈수록 또렷해지고(터지기 직전), 퍼지는 고리는 옅어진다.
                    let a = if r1 < r0 {
                        0.25 + 0.65 * k
                    } else {
                        0.95 * (1.0 - k)
                    };
                    for (dr, aa) in [(0.0, 1.0), (0.13, 0.55), (0.26, 0.25)] {
                        fl.ring(
                            at + Vec3::Y * 0.06,
                            (r - dr).max(0.05),
                            0.0,
                            360.0,
                            with_a(m.color, a * aa),
                        );
                    }
                }
                MarkKind::Streak { dir, len, half } => {
                    // 꼬리(떠난 자리)부터 걷힌다 — 머리는 끝까지 남았다 옅어진다.
                    let tail = at + Vec3::Y * 0.9 + dir * (len * k * 0.7);
                    let head = at + Vec3::Y * 0.9 + dir * len;
                    let side = vec3(dir.z, 0.0, -dir.x);
                    let fade = (1.0 - k) * (1.0 - k);
                    for (w2, ink, a) in [(half, m.color, 0.55), (half * 0.3, hot, 0.95)] {
                        let on = with_a(ink, a * fade);
                        let off = with_a(ink, 0.0);
                        flat_mesh(
                            &[
                                (tail - side * w2 * 0.15, off),
                                (tail + side * w2 * 0.15, off),
                                (head - side * w2, on),
                                (head + side * w2, on),
                            ],
                            &[0, 1, 2, 1, 3, 2],
                        );
                    }
                    // 바닥에 남는 옅은 자국.
                    fl.lane(
                        at + Vec3::Y * 0.05,
                        dir,
                        len,
                        half * 1.2,
                        with_a(m.color, 0.16 * fade),
                    );
                }
                MarkKind::Shock {
                    dir,
                    r0,
                    r1,
                    arc_deg,
                } => {
                    let r = r0 + (r1 - r0) * out;
                    let yaw = dir.x.atan2(dir.z);
                    let a = 1.0 - k * k;
                    fl.sector(
                        at + Vec3::Y * 0.05,
                        r,
                        yaw,
                        arc_deg,
                        with_a(m.color, 0.2 * (1.0 - k)),
                    );
                    for (dr, col, aa) in
                        [(0.0, hot, 0.95), (0.14, m.color, 0.8), (0.3, m.color, 0.4)]
                    {
                        fl.ring(
                            at + Vec3::Y * 0.07,
                            (r - dr).max(0.05),
                            yaw,
                            arc_deg,
                            with_a(col, a * aa),
                        );
                    }
                    crescent(
                        at + Vec3::Y * 0.35,
                        r,
                        yaw,
                        arc_deg.to_radians(),
                        crate::weapons::Sweep::Center,
                        1.0,
                        with_a(m.color, a),
                        0.7,
                    );
                }
                MarkKind::Cracks { dir, r, n, arc_deg } => {
                    // 금은 빠르게 뻗고(수명의 15% 안) 천천히 식는다: 금빛 → 흙빛.
                    let grow = (k / 0.15).min(1.0);
                    let cool = ((k - 0.2) / 0.8).clamp(0.0, 1.0);
                    let dust = Color::new(0.5, 0.46, 0.42, 1.0);
                    let col = Color::new(
                        m.color.r + (dust.r - m.color.r) * cool,
                        m.color.g + (dust.g - m.color.g) * cool,
                        m.color.b + (dust.b - m.color.b) * cool,
                        0.9 * (1.0 - cool * cool),
                    );
                    let yaw = dir.x.atan2(dir.z);
                    let span = arc_deg.to_radians();
                    for i in 0..n {
                        let h = |j: u32| {
                            let x = (i as u32 * 7919 + j * 104_729).wrapping_mul(2_654_435_761);
                            ((x >> 8) & 0xffff) as f32 / 65535.0 - 0.5
                        };
                        let a = yaw - span * 0.5 + span * (i as f32 + 0.5 + h(0) * 0.6) / n as f32;
                        let reach = r * (0.75 + 0.5 * (h(1) + 0.5)) * grow;
                        let (fwd, side) =
                            (vec3(a.sin(), 0.0, a.cos()), vec3(a.cos(), 0.0, -a.sin()));
                        let segs = ((reach / 0.7).ceil() as usize).max(1);
                        let mut prev = at + fwd * 0.5;
                        for j in 1..=segs {
                            let d = 0.5 + (reach - 0.5).max(0.0) * j as f32 / segs as f32;
                            let cur = at + fwd * d + side * (h(j as u32 + 2) * 0.55);
                            let wd = 0.11 * (1.0 - 0.7 * j as f32 / segs as f32);
                            let (p0, p1) = (
                                fl.at(prev.x, prev.z, at.y, 0.05),
                                fl.at(cur.x, cur.z, at.y, 0.05),
                            );
                            flat_mesh(
                                &[
                                    (p0 - side * wd, col),
                                    (p0 + side * wd, col),
                                    (p1 - side * wd * 0.6, col),
                                    (p1 + side * wd * 0.6, col),
                                ],
                                &[0, 1, 2, 1, 3, 2],
                            );
                            prev = cur;
                        }
                    }
                }
                MarkKind::Converge { r, n } => {
                    // 바깥에서 가운데로 달려드는 빛줄: 머리(안쪽)가 밝고 꼬리(바깥)가 사라진다.
                    let a = (k / 0.15).min(1.0) * (1.0 - k * k);
                    for i in 0..n {
                        let ang = i as f32 / n as f32 * std::f32::consts::TAU + 0.3;
                        let d = vec3(ang.sin(), 0.0, ang.cos());
                        let side = vec3(d.z, 0.0, -d.x) * 0.07;
                        let head = at + Vec3::Y * 0.45 + d * (r * (1.0 - out) + 0.25);
                        let tail = at + Vec3::Y * 0.45 + d * (r * (1.0 - out * 0.55) + 0.6);
                        flat_mesh(
                            &[
                                (tail - side * 0.3, with_a(m.color, 0.0)),
                                (tail + side * 0.3, with_a(m.color, 0.0)),
                                (head - side, with_a(hot, a)),
                                (head + side, with_a(hot, a)),
                            ],
                            &[0, 1, 2, 1, 3, 2],
                        );
                    }
                }
                MarkKind::Slash { dir, len, half } => {
                    // 처음 한순간은 온통 희게, 그 뒤 금빛 겉이 길게 식는다. 먼 끝으로 갈수록 가늘다.
                    let flash = (1.0 - k / 0.12).clamp(0.0, 1.0);
                    let fade = (1.0 - k) * (1.0 - k);
                    let side = vec3(dir.z, 0.0, -dir.x);
                    let (near, far) = (at + dir * 0.4, at + dir * len);
                    let lift = Vec3::Y * 0.12;
                    // 바닥의 금빛 겉(가장자리로 사라진다) + 흰 속.
                    for (w2, ink, a) in [
                        (
                            half * (1.0 + 0.25 * flash),
                            m.color,
                            0.6 * fade + 0.4 * flash,
                        ),
                        (half * 0.34 * (1.0 - 0.6 * k), hot, fade.max(flash)),
                    ] {
                        let (on, off) = (with_a(ink, a), with_a(ink, 0.0));
                        flat_mesh(
                            &[
                                (near + lift - side * w2, off),
                                (near + lift, on),
                                (near + lift + side * w2, off),
                                (far + lift - side * w2 * 0.2, off),
                                (far + lift, on),
                                (far + lift + side * w2 * 0.2, off),
                            ],
                            &[0, 1, 3, 1, 4, 3, 1, 2, 4, 2, 5, 4],
                        );
                    }
                    // 세로로 선 빛의 면: 발밑에서 머리 위까지 — 위로 사라진다.
                    let top = Vec3::Y * (2.4 * (1.0 - 0.5 * k));
                    let base = with_a(hot, 0.55 * fade + 0.45 * flash);
                    let edge = with_a(m.color, 0.0);
                    flat_mesh(
                        &[
                            (near + lift, base),
                            (far + lift, with_a(hot, base.a * 0.5)),
                            (near + lift + top, edge),
                            (far + lift + top * 0.5, edge),
                        ],
                        &[0, 1, 2, 1, 3, 2],
                    );
                }
            }
        }
    }

    /// 이어지는 것들(0158 G2 — 월드의 상태에서): 되울림의 울림(몸 둘레에서 맴도는 호 + 닿는 거리의 옅은 테), 잔불 흩기의 불티
    /// (가장자리의 호박빛 테 + 떠오르는 불티), 몰아치기의 기운(발밑의 금빛 고리 + 몸을 도는 별 둘), 던진 무기의 빛.
    pub(super) fn skill_auras(&self, w: &World, pp: Vec3, alpha: f32, t: &Tuning) {
        let fl = Drape {
            ground: w.ground.as_ref(),
        };
        let pl = &w.player;
        let gold = Color::new(1.0, 0.86, 0.5, 1.0);
        let mint = Color::new(0.66, 1.0, 0.86, 1.0);
        let amber = Color::new(1.0, 0.72, 0.32, 1.0);
        let hot = Color::new(1.0, 0.97, 0.86, 1.0);
        let fx = t[crate::tuning::P::RenderFx] >= 0.5;
        let feet = vec3(pp.x, w.ground_y(pp), pp.z);
        if pl.skill.echo_left > 0.0 {
            let fade = (pl.skill.echo_left / 0.4).min(1.0);
            let breathe = 0.5 + 0.5 * (w.time * 6.0).sin();
            if let crate::skills::Form::Echo { radius, .. } =
                w.skill_eff(crate::skills::Id::Echo).skill.form
            {
                fl.ring(
                    feet + Vec3::Y * 0.05,
                    radius,
                    0.0,
                    360.0,
                    with_a(mint, 0.16 * fade),
                );
            }
            // 종의 울림: 마주 도는 호 두 쌍(가슴 높이·발밑).
            for (h, r, spin, a) in [(0.9, 1.0, 3.2, 0.7), (0.06, 1.35, -2.1, 0.5)] {
                for i in 0..2 {
                    fl.ring(
                        feet + Vec3::Y * h,
                        r + 0.06 * breathe,
                        w.time * spin + i as f32 * std::f32::consts::PI,
                        95.0,
                        with_a(if i == 0 { hot } else { mint }, a * fade),
                    );
                }
            }
        }
        for f in &w.skill_fields {
            let c = vec3(f.pos.x, w.ground_y(f.pos), f.pos.z);
            // 끝 0.4초에 옅어진다, 타가 가까울수록 테가 밝아진다(다음 타의 박자).
            let fade = ((f.secs - f.age) / 0.4).clamp(0.0, 1.0) * (f.age / 0.15).min(1.0);
            let beat = (f.age / f.every.max(1e-3)).fract();
            fl.sector(
                c + Vec3::Y * 0.04,
                f.radius,
                0.0,
                360.0,
                with_a(amber, (0.06 + 0.05 * beat) * fade),
            );
            for (dr, a) in [(0.0, 0.75), (0.14, 0.4), (0.3, 0.18)] {
                fl.ring(
                    c + Vec3::Y * 0.06,
                    f.radius - dr,
                    0.0,
                    360.0,
                    with_a(
                        if dr == 0.0 { gold } else { amber },
                        a * fade * (0.6 + 0.4 * beat),
                    ),
                );
            }
            if fx {
                let n = ((crate::fx::FIELD_MOTES as f32 * (f.radius / 4.0).powi(2)) as usize)
                    .min(crate::fx::FIELD_MOTES_MAX);
                for i in 0..n {
                    let (off, k) = crate::fx::field_mote(i, w.time);
                    if k <= 0.03 {
                        continue;
                    }
                    let p = c + vec3(off.x * f.radius, off.y, off.z * f.radius);
                    // 떠오르자마자 흰 금빛, 위로 갈수록 호박빛으로 식는다.
                    let cool = (off.y / 1.3).clamp(0.0, 1.0);
                    let col = Color::new(
                        1.0,
                        hot.g + (amber.g - hot.g) * cool,
                        hot.b + (amber.b - hot.b) * cool,
                        k * fade,
                    );
                    gem(p, 0.14 - 0.05 * cool, col);
                }
            }
        }
        if pl.skill.flurry_left > 0.0 {
            let fade = (pl.skill.flurry_left / 0.5).min(1.0);
            let breathe = 0.5 + 0.5 * (w.time * 9.0).sin();
            for (r, a) in [(0.7, 0.8), (0.95, 0.4)] {
                fl.ring(
                    feet + Vec3::Y * 0.06,
                    r + 0.07 * breathe,
                    0.0,
                    360.0,
                    with_a(gold, a * fade * (0.6 + 0.4 * breathe)),
                );
            }
            if fx {
                for i in 0..2 {
                    let a = w.time * 11.0 + i as f32 * std::f32::consts::PI;
                    let p = pp
                        + vec3(
                            a.sin() * 0.75,
                            pl.air + 1.0 + 0.25 * (a * 0.5).sin(),
                            a.cos() * 0.75,
                        );
                    gem(p, 0.34, with_a(gold, 0.4 * fade));
                    spark4(p, 0.42, with_a(hot, fade));
                }
            }
        }
        for b in &w.skill_blades {
            let p = b.prev.lerp(b.pos, alpha);
            let g = vec3(p.x, w.ground_y(p), p.z);
            fl.shadow(g + Vec3::Y * 0.02, 0.32);
            gem(p, 1.0, with_a(gold, 0.28));
            if fx {
                // 도는 날의 번짐: 날을 따라 도는 초승달 둘.
                for turn in [0.0, std::f32::consts::PI] {
                    crescent(
                        p,
                        0.8,
                        b.age * BLADE_SPIN + turn,
                        2.4,
                        crate::weapons::Sweep::LeftToRight,
                        1.0,
                        with_a(gold, 0.9),
                        1.3,
                    );
                }
            }
        }
    }
}
