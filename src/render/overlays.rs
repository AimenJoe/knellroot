//! 덧그림: 탄·조각·폭발, 휘두름의 초승달과 박자 부채꼴, 모으기·강타, 스킬·큰 기술의 모양, 패링, 발밑 고리, 적 머리 위·락온 표시.
use super::*;
use crate::weapons::Look;

/// 박자 타의 휘두름 색·발밑 고리가 퍼지는 시간(0076).
// 박자 금빛(0076 → 0098 사장 "너무 강한 노란색, 조금만 더 옅게"): 노랑을 흰빛 쪽으로(채도 낮춤) + 진하기 30% 안팎 내림.
const BEAT_GOLD: Color = Color::new(1.0, 0.9, 0.62, 0.24);
pub const BEAT_RING_LIFE: f32 = 0.35;
/// 딱 맞춰 떼기 고리가 퍼지는 시간(0078).
const PERFECT_RING_LIFE: f32 = 0.45;
/// 휘두름 초승달의 진하기(0242 — 사장 설문 13-8 "조금 더 연해도 좋겠다").
const SLASH_ALPHA: f32 = 0.7;
/// 탄의 빛(알맹이 · 빛무리 — 0048·0051, 0278 R17: 꼴마다 와일드카드 없이) — 붉은색(위험)은 쓰지 않는다. 모으는 손의 빛 덩이 ·
/// 발밑 고리(`wpn_look`)도 이것이라 몸으로 그리는 꼴(던지는 것 · 땅 내려찍기 · 신성한 빛 · 가시 씨앗)에도 빛이 있다. 옛 표는 수의
/// 차례였고 끝 줄(돌의 잿빛)에 멈췄다 — 단검 · 방패 · 땅 내려찍기 · 가시 씨앗이 그 잿빛, 그대로 둔다.
fn bolt_colors(look: Look) -> (Color, Color) {
    // 꿰뚫는 마법 탄(지팡이) 민트 · 쇠뇌 볼트 밝은 나무.
    const MINT: (Color, Color) = (
        Color::new(0.85, 1.0, 0.92, 1.0),
        Color::new(0.45, 0.9, 0.7, 0.45),
    );
    const BRASS: (Color, Color) = (
        Color::new(0.95, 0.85, 0.65, 1.0),
        Color::new(0.8, 0.65, 0.4, 0.3),
    );
    // 활의 화살(0231): 잔뿌리의 연두 — 쇠뇌의 놋쇠 볼트와 갈린다(빨강은 쓰지 않는다).
    const SPROUT: (Color, Color) = (
        Color::new(0.88, 1.0, 0.78, 1.0),
        Color::new(0.55, 0.85, 0.5, 0.3),
    );
    // 0274 ②b: 손도끼 · 던진 창(쇳빛) · 신성한 빛(금빛) · 돌(잿빛).
    const STEEL: (Color, Color) = (
        Color::new(0.92, 0.94, 1.0, 1.0),
        Color::new(0.7, 0.76, 0.9, 0.35),
    );
    const GOLD: (Color, Color) = (
        Color::new(1.0, 0.95, 0.78, 1.0),
        Color::new(1.0, 0.85, 0.45, 0.4),
    );
    const ASH: (Color, Color) = (
        Color::new(0.8, 0.77, 0.72, 1.0),
        Color::new(0.6, 0.58, 0.55, 0.3),
    );
    match look {
        Look::Magic => MINT,
        Look::Bolt => BRASS,
        Look::Arrow => SPROUT,
        Look::Axe | Look::Javelin => STEEL,
        Look::Light => GOLD,
        Look::Stone | Look::Knife | Look::Shield | Look::Quake | Look::Seed => ASH,
        // 저주탄(0274 ⑥)은 하늘빛(균열).
        Look::Curse => CURSE_LOOK,
    }
}
/// 저주탄(0274 ⑥): 하늘빛 알맹이 · 빛무리(균열의 빛 `status::INK` — 보라 · 빨강은 쓰지 않는다).
const CURSE_LOOK: (Color, Color) = (
    Color::new(0.86, 0.95, 1.0, 1.0),
    Color::new(0.50, 0.80, 1.0, 0.4),
);

impl Renderer {
    /// 배우 위에 얹는 것: 스윙 부채꼴·쟁반 테·휘청 별·락온 표시·마법 탄.
    pub(super) fn overlays(&self, w: &World, pp: Vec3, alpha: f32, t: &Tuning) {
        // 바닥 표시는 모두 땅 높낮이를 따라 접는다(0083 — 경사로·턱에서 묻히거나 뜨던 것).
        let fl = Drape {
            ground: w.ground.as_ref(),
        };
        bolts(w, alpha);
        smites(w, &fl);
        bell_shards(w, &fl);
        bombs(w, alpha);
        carts(w, alpha);
        skill_spikes(w);
        cast_charge(w, pp, t, &fl);
        rush_charge(w, pp, &fl);
        fury_glow(w, pp, &fl);
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
        ail_marks(w, alpha, &fl);
        ice_coats(w, alpha);
        spill_rings(w, pp, t, &fl);
        stagger_stars(w, alpha);
        pull_vines(w, pp, alpha);
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
                                yaw + crate::skills::WHIRL_CLIP_TURN * turns * std::f32::consts::TAU
                            } else {
                                w.player.facing
                            };
                            crescent(
                                pp + Vec3::Y * 0.95,
                                radius,
                                head + std::f32::consts::PI * 0.75,
                                std::f32::consts::PI * 1.5,
                                crate::weapons::Sweep::LeftToRight,
                                (1.0, 0.0),
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
                                (1.0, 0.0),
                                with_a(WHITE, 0.6 * fade * fade),
                                0.9,
                            );
                        }
                    }
                    // 가로지르기: 지나온 길의 빛 띠(머리 = 몸, 꼬리로 사라진다).
                    // 방패 돌진(0270 C3)은 같은 띠를 금빛으로 — 방패의 쇠.
                    Form::Lane {
                        dist, half, time, ..
                    } => {
                        let mint = if id == crate::skills::Id::Ram {
                            gold
                        } else {
                            mint
                        };
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
                    // 불길(0270 C3): 같은 부채꼴을 호박빛으로 — 휘두름이 아니라 내뿜음이라 초승달은 없다.
                    Form::Cone { range, arc_deg, .. } => {
                        let k = (e0 / 0.3).clamp(0.0, 1.0);
                        let flame = id == crate::skills::Id::Flare;
                        if k < 1.0 {
                            let ink = if flame {
                                Color::new(1.0, 0.72, 0.32, 1.0)
                            } else {
                                gold
                            };
                            fl.sector(
                                pp + Vec3::Y * 0.06,
                                range,
                                yaw,
                                arc_deg,
                                with_a(ink, 0.3 * (1.0 - k)),
                            );
                            if fx && !flame {
                                crescent(
                                    pp + Vec3::Y * 0.5,
                                    range * (0.5 + 0.5 * k),
                                    yaw,
                                    arc_deg.to_radians(),
                                    crate::weapons::Sweep::Center,
                                    (1.0, 0.0),
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
                            let n = w.player.skill.brace_soaked.min(most);
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
                    // 0214: 제비걸음·뿌리 솟기는 이펙트(빛 띠·뿌리)가, 막아치기는 부채꼴 번쩍임이 말한다.
                    Form::Echo { .. }
                    | Form::Throw { .. }
                    | Form::Field { .. }
                    | Form::Hop { .. }
                    | Form::Bash { .. }
                    | Form::Erupt { .. }
                    | Form::Bell { .. }
                    | Form::Hex { .. }
                    | Form::Drift { .. }
                    | Form::Trap { .. } => {}
                    // 재기(0274): 몸에서 밀쳐 내는 거리까지 번지는 민트 고리 — 피해가 아니라 밀어냄과 흡수의 자리.
                    Form::Rally { radius, .. } => {
                        let k = (e0 / 0.3).clamp(0.0, 1.0);
                        if k < 1.0 {
                            fl.ring(
                                pp + Vec3::Y * 0.06,
                                radius * (0.35 + 0.65 * k),
                                0.0,
                                360.0,
                                with_a(mint, 0.85 * (1.0 - k)),
                            );
                        }
                    }
                    // 생기 거두기(0274 ⑤): 몸과 함께 도는 날이 지나간 둘레가 풀빛으로 차오르고(한 바퀴 = `time`), 벤 순간 둘레의 테가 남는다.
                    Form::Reap { radius, time, .. } => {
                        let life = Color::new(0.55, 0.95, 0.45, 1.0);
                        let k = (e0 / time.max(1e-3)).clamp(0.0, 1.0);
                        let fade = 1.0 - ((e0 - time) / SKILL_GLOW_FADE).clamp(0.0, 1.0);
                        if fade > 0.0 {
                            let arc = 360.0 * k;
                            fl.sector(
                                pp + Vec3::Y * 0.05,
                                radius,
                                yaw + crate::skills::WHIRL_CLIP_TURN * arc.to_radians() * 0.5,
                                arc,
                                with_a(life, 0.14 * fade),
                            );
                            if k >= 1.0 {
                                fl.ring(
                                    pp + Vec3::Y * 0.07,
                                    radius,
                                    0.0,
                                    360.0,
                                    with_a(life, 0.8 * fade),
                                );
                            }
                        }
                    }
                    // 파문(0270 C3): 몸에서 번지는 물결의 앞머리(흰 테 + 민트 테), 지나간 안쪽은 옅게.
                    Form::Wave { radius, time } => {
                        let k = (e0 / time.max(1e-3)).clamp(0.0, 1.0);
                        let fade = 1.0 - ((e0 - time) / SKILL_GLOW_FADE).clamp(0.0, 1.0);
                        if fade > 0.0 {
                            let r = (radius * k).max(0.3);
                            fl.sector(pp + Vec3::Y * 0.05, r, 0.0, 360.0, with_a(mint, 0.1 * fade));
                            fl.ring(
                                pp + Vec3::Y * 0.07,
                                r,
                                0.0,
                                360.0,
                                with_a(WHITE, 0.85 * fade),
                            );
                            if fx {
                                fl.ring(
                                    pp + Vec3::Y * 0.07,
                                    (r - 0.35).max(0.2),
                                    0.0,
                                    360.0,
                                    with_a(mint, 0.6 * fade),
                                );
                            }
                        }
                    }
                    // 빠지며 베기(0274 ③): 벤 순간 앞 부채꼴이 번쩍이고 물러나는 동안 사라진다 — 벤 자리에 남는다(몸은 물러난다).
                    Form::Slip {
                        range,
                        arc_deg,
                        back,
                        time,
                    } => {
                        let fade = (1.0 - e0 / (time + 0.1)).clamp(0.0, 1.0);
                        if fade > 0.0 {
                            let gone = (e0 / time.max(1e-3)).clamp(0.0, 1.0) * back;
                            let at = pp + dir * gone + Vec3::Y * 0.06;
                            fl.sector(at, range, yaw, arc_deg, with_a(mint, 0.26 * fade));
                            if fx {
                                fl.ring(at, range, yaw, arc_deg, with_a(WHITE, 0.8 * fade));
                            }
                        }
                    }
                    // 바늘땀(0270 C3): 땀마다 좁은 부채꼴이 번쩍이고 곧 사라진다(가운데 흰 바늘 줄).
                    Form::Stab {
                        range,
                        arc_deg,
                        hits,
                        gap,
                    } => {
                        let k = e0 / gap.max(1e-3);
                        let since = k - k.floor().min(hits.saturating_sub(1) as f32);
                        let fade = (1.0 - since * gap / 0.12).clamp(0.0, 1.0);
                        if fade > 0.0 {
                            fl.sector(
                                pp + Vec3::Y * 0.06,
                                range,
                                yaw,
                                arc_deg,
                                with_a(mint, 0.28 * fade),
                            );
                            if fx {
                                fl.line(
                                    pp + Vec3::Y * 0.07 + dir * 0.4,
                                    pp + Vec3::Y * 0.07 + dir * range,
                                    with_a(WHITE, 0.9 * fade),
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

fn bolts(w: &World, alpha: f32) {
    // 마법 탄(0048): 라벤더 빛 덩이 + 꼬리(보간 자리).
    // 0242(사장 "습작 같다"): 꼬리 마름모 셋 → 가늘어지는 띠 + 둥근 빛무리 + 숨 쉬는 알맹이와 작은 별.
    for b in &w.bolts {
        let p = b.prev.lerp(b.pos, alpha);
        // 던지는 것(0274 ②b): 빛 덩이가 아니라 몸 — 손도끼는 날을 앞세워 세로로 돌고, 돌은 굴러 간다, 창은 모델로(`javelins`).
        if b.def.thrown() {
            thrown(w, b, p);
            continue;
        }
        // 땅 내려찍기(0274 ④ — 대검의 RB): 빛 덩이가 아니라 땅을 타고 달리는 물결 — 앞머리의 가로 띠(흙빛 + 흰 속) · 지나온 금 ·
        // 앞머리에서 솟았다 가라앉는 돌 조각.
        if b.def.look == Look::Quake {
            quake_wave(w, b, p);
            continue;
        }
        // 저주탄(0274 ⑥ — 완드의 주된 공격): 잿빛 돌 조각 같은 어두운 알맹이를 하늘빛 빛무리가 감싸고, 뒤에 꺾인 금(지그재그 띠)이 남는다.
        if b.def.look == Look::Curse {
            curse_bolt(w, b, p);
            continue;
        }
        let (core, glow) = bolt_colors(b.def.look);
        let k = if b.def.is_shot() { 0.6 } else { 1.0 } * b.def.scale;
        let pulse = 1.0 + 0.12 * (w.time * 30.0 + b.left).sin();
        glow_disc(p, 0.62 * k, with_a(glow, glow.a * 0.8));
        trail(p, p - b.dir * (1.5 * k), 0.5 * k, with_a(glow, 0.7));
        trail(p, p - b.dir * (0.8 * k), 0.2 * k, with_a(core, 0.85));
        gem(p, 0.5 * k * pulse, glow);
        gem(p, 0.3 * k * pulse, core);
        if !b.def.is_shot() {
            spark4(p, 0.62 * k * pulse, with_a(core, 0.9));
        }
    }
}

/// 던지는 손도끼의 몸(0277 — 키웠다): (가운데, 크기, 빛) — 크기 배 1에서. 자루 가운데가 돌림의 가운데라 도는 몸이 그리는 가장 먼 데는
/// `drawn_reach`이고, 그것이 탄의 맞는 반지름(`tuning::BOLT_RADIUS`) 안이다(시험 — 그림이 판정보다 크지 않게).
pub(super) const AXE_PARTS: [(Vec3, Vec3, Color); 2] = [
    (
        Vec3::new(0.0, -0.04, 0.0),
        Vec3::new(0.07, 0.46, 0.07),
        Color::new(0.55, 0.40, 0.26, 1.0),
    ),
    (
        Vec3::new(0.0, 0.17, 0.07),
        Vec3::new(0.07, 0.20, 0.28),
        Color::new(0.90, 0.92, 0.97, 1.0),
    ),
];
/// 손도끼를 감싸는 옅은 빛무리의 반지름(크기 배 1).
pub(super) const AXE_GLOW: f32 = 0.34;

/// 상자들(가운데 · 크기)이 원점 둘레로 어떻게 돌아도 닿는 가장 먼 거리 — 모서리까지(시험이 판정과 견준다).
#[cfg(test)]
pub(super) fn drawn_reach(parts: &[(Vec3, Vec3)]) -> f32 {
    parts
        .iter()
        .map(|(c, s)| (c.abs() + *s * 0.5).length())
        .fold(0.0, f32::max)
}

/// 던지는 RB의 몸(0274 ②b): 손도끼 = 쇳빛 날 + 나무 자루를 나는 쪽의 옆 축으로 세로로 돌린다(날이 보이게), 돌 = 굴러 가는 잿빛
/// 덩이. 꼬리는 옅은 바람 띠 하나(빛 덩이가 아니다 — 마법과 갈린다). 창은 모델 부위로(`Renderer::javelins`) — 여기서는 바람 띠만.
fn thrown(w: &World, b: &crate::world::Bolt, p: Vec3) {
    let k = b.def.scale;
    let wind = Color::new(0.92, 0.94, 1.0, 0.28);
    if b.back {
        trail(p, p - b.dir * 1.0, 0.12, with_a(wind, 0.18));
        return;
    }
    trail(p, p - b.dir * (1.2 * k), 0.22 * k, wind);
    let side = vec3(-b.dir.z, 0.0, b.dir.x).normalize_or(Vec3::X);
    let spin = w.time * 18.0 + b.left;
    match b.def.look {
        Look::Axe => {
            // 자루(나무)와 머리(쇠): 자루 가운데를 돌림의 가운데로. 0277: 게임 거리에서 읽히게 키웠다 — 굵은 자루 · 넓은 날, 그래도 도는
            // 몸 전체가 탄의 맞는 반지름(`BOLT_RADIUS` × 크기 배) 안이다(`AXE_PARTS` — 시험). 옅은 쇳빛 빛무리가 날을 감싼다.
            let rot = Quat::from_axis_angle(side, spin)
                * Quat::from_rotation_arc(Vec3::Y, b.dir.normalize_or(Vec3::Z));
            glow_disc(p, AXE_GLOW * k, Color::new(0.86, 0.9, 1.0, 0.24));
            for (at, size, col) in AXE_PARTS {
                shaded_box_rot(p + at * k, size * k, col, 0.6, rot, p);
            }
        }
        // 투척 단검(0274 ③): 날을 앞세워 곧게 — 돌지 않는다(쇠 날 + 짧은 가죽 자루).
        Look::Knife => {
            let rot = Quat::from_rotation_arc(Vec3::Y, b.dir.normalize_or(Vec3::Z));
            let s = 1.2 * k;
            shaded_box_rot(
                p,
                vec3(0.05, 0.28, 0.02) * s,
                Color::new(0.90, 0.92, 0.97, 1.0),
                0.6,
                rot,
                p,
            );
            shaded_box_rot(
                p - b.dir * (0.2 * s),
                vec3(0.035, 0.12, 0.035) * s,
                Color::new(0.42, 0.30, 0.24, 1.0),
                0.6,
                rot,
                p,
            );
        }
        Look::Stone => {
            let rot = Quat::from_axis_angle(side, spin * 0.7);
            shaded_tris(
                p,
                &octa_faces(vec2(0.13 * k, 0.11 * k), 0.12 * k, 0.1 * k),
                Color::new(0.58, 0.55, 0.52, 1.0),
                0.7,
                rot,
            );
        }
        // 창 · 방패는 모델 부위로 그린다(`Renderer::javelins` · `shields`) — 여기서는 바람 띠만. 던지지 않는 꼴은 이 길에 오지 않는다.
        Look::Javelin
        | Look::Shield
        | Look::Magic
        | Look::Bolt
        | Look::Arrow
        | Look::Light
        | Look::Quake
        | Look::Seed
        | Look::Curse => {}
    }
}

/// 저주탄의 몸(0274 ⑥): 하늘빛 빛무리 + 어두운 잿빛 알맹이(돌기가 돈다) + 그 속의 작은 하늘빛 별, 꼬리는 나는 쪽의 옆으로 꺾이는 가는
/// 하늘빛 금(세 마디 — 마법 탄의 곧은 빛 띠와 갈린다).
fn curse_bolt(w: &World, b: &crate::world::Bolt, p: Vec3) {
    let (core, glow) = CURSE_LOOK;
    let k = b.def.scale;
    let pulse = 1.0 + 0.1 * (w.time * 24.0 + b.left).sin();
    glow_disc(p, 0.75 * k, with_a(glow, 0.36));
    let side = vec3(-b.dir.z, 0.0, b.dir.x).normalize_or(Vec3::X) * (0.14 * k);
    let back = b.dir * (0.6 * k);
    let mut at = p;
    for i in 0..3 {
        let next = at - back + if i % 2 == 0 { side } else { -side };
        trail(at, next, 0.12 * k, with_a(glow, 0.8 - 0.2 * i as f32));
        at = next;
    }
    let rot = Quat::from_rotation_y(w.time * 7.0 + b.left) * Quat::from_rotation_x(w.time * 5.0);
    shaded_tris(
        p,
        &octa_faces(vec2(0.17 * k, 0.14 * k), 0.17 * k, 0.14 * k),
        Color::new(0.36, 0.35, 0.38, 1.0),
        0.7,
        rot,
    );
    spark4(p, 0.38 * k * pulse, with_a(core, 0.95));
}

/// 분노(0277 — 대검의 주특성 `World::fury`): 반(`FURY_GLOW_FROM`)부터 몸에 숨 쉬는 주황 빛무리 + 발밑의 옅은 주황 고리, 분노만큼 짙다 —
/// 가득이면 숨이 빨라지고 고리가 또렷하다(솟는 불씨는 이펙트 `Fx::watch`). 빨강은 쓰지 않는다(초록 0.5 넘게).
fn fury_glow(w: &World, pp: Vec3, fl: &Drape) {
    let f = w.fury();
    if f < FURY_GLOW_FROM {
        return;
    }
    let k = (f - FURY_GLOW_FROM) / (1.0 - FURY_GLOW_FROM);
    let full = f >= 1.0;
    let breath = 0.75 + 0.25 * (w.time * if full { 7.0 } else { 4.0 }).sin();
    let ember = Color::new(1.0, 0.6, 0.28, 1.0);
    let chest = pp + Vec3::Y * (w.player.air + 0.7);
    glow_disc(
        chest,
        0.75 + 0.15 * breath,
        with_a(ember, (0.10 + 0.18 * k) * breath),
    );
    fl.ring(
        pp + Vec3::Y * 0.05,
        0.62,
        0.0,
        360.0,
        with_a(ember, (0.18 + 0.32 * k) * breath),
    );
}

/// 분노의 빛무리가 보이기 시작하는 분노(0..1 — 적 둘에 에워싸이면 0.5, `Fx`의 불씨와 같은 문턱).
const FURY_GLOW_FROM: f32 = 0.5;

/// 걸어 당김의 덩굴(0277 — 낫의 셋째 타 · 강공격 · 걸어 당기기): 끌려오는 적(`Enemy::tug` — 미끄러지는 동안)과 방랑자의 손을 잇는
/// 흙빛 뿌리 줄 + 연한 새순의 속, 가운데가 처지며 꿈틀대고 남은 몫만큼 옅어진다. 마디에 새순 둘. 빨강 · 보라(생물의 빛)는 쓰지 않는다.
fn pull_vines(w: &World, pp: Vec3, alpha: f32) {
    for (i, en) in w.enemies.iter().enumerate() {
        if en.tug <= 0.0 || !en.alive() {
            continue;
        }
        let k = (en.tug / crate::weapons::PULL_SHOW).clamp(0.0, 1.0);
        let hand = pp + Vec3::Y * (w.player.air + 0.75);
        let body = en.prev.lerp(en.pos, alpha) + Vec3::Y * en.height * 0.5;
        let phase = w.time * 18.0 + i as f32 * 1.7;
        let a = 0.35 + 0.65 * k;
        vine(hand, body, 0.17, with_a(VINE_DARK, 0.95 * a), phase);
        vine(hand, body, 0.065, with_a(VINE_LIGHT, 0.9 * a), phase);
        for f in [0.35f32, 0.7] {
            gem(vine_at(hand, body, f, phase), 0.14, with_a(VINE_LIGHT, a));
        }
    }
}

const VINE_DARK: Color = Color::new(0.46, 0.34, 0.23, 1.0);
const VINE_LIGHT: Color = Color::new(0.72, 0.93, 0.60, 1.0);
/// 덩굴이 처지고(m) 꿈틀대는(m) 몫.
const VINE_SAG: f32 = 0.18;
const VINE_WIGGLE: f32 = 0.10;

/// 덩굴의 한 점(`f` = 0 손 → 1 적): 곧은 줄에서 가운데가 처지고, 옆으로 1.5 물결 꿈틀댄다.
fn vine_at(a: Vec3, b: Vec3, f: f32, phase: f32) -> Vec3 {
    let along = b - a;
    let side = vec3(-along.z, 0.0, along.x).normalize_or(Vec3::X);
    let bend = (f * std::f32::consts::PI).sin();
    a + along * f - Vec3::Y * (VINE_SAG * bend)
        + side * ((f * std::f32::consts::TAU * 1.5 + phase).sin() * VINE_WIGGLE * bend)
}

/// 덩굴 띠(카메라를 보는 굵기 `w` — 적 쪽으로 조금 가늘어진다).
fn vine(a: Vec3, b: Vec3, w: f32, col: Color, phase: f32) {
    const N: usize = 12;
    let view = (a - look().eye).normalize_or(Vec3::NEG_Z);
    let mut pts = Vec::with_capacity((N + 1) * 2);
    let mut idx = Vec::with_capacity(N * 6);
    for i in 0..=N {
        let f = i as f32 / N as f32;
        let c = vine_at(a, b, f, phase);
        let next =
            vine_at(a, b, (f + 0.02).min(1.0), phase) - vine_at(a, b, (f - 0.02).max(0.0), phase);
        let side = next.cross(view).normalize_or(Vec3::X);
        let half = w * 0.5 * (1.0 - 0.35 * f);
        pts.push((c + side * half, col));
        pts.push((c - side * half, col));
    }
    for i in 0..N {
        let q = (i * 2) as u16;
        idx.extend_from_slice(&[q, q + 1, q + 2, q + 1, q + 3, q + 2]);
    }
    flat_mesh(&pts, &idx);
}

/// 땅 내려찍기의 물결(0274 ④): 땅 높이에서 나는 쪽의 가로로 선 앞머리(흙빛 띠 + 흰 속) + 뒤로 남는 금(가는 띠) + 앞머리에서 솟았다
/// 가라앉는 돌 조각 셋. 빨강은 쓰지 않는다.
///
/// 0277(사장 판정 123-5 "작다"): 옛 앞머리는 옆으로 0.75 m × 크기 배 — 판정(탄의 맞는 반지름 `BOLT_RADIUS` 0.35 × 크기 배)의 두 배라
/// 맞지 않은 적 위로도 지나갔다. 폭은 판정 안으로 줄이고(`QUAKE_HALF`), 대신 **높이와 밝기로** 키운다 — 앞머리에서 흙이 솟는 기둥
/// (`QUAKE_CREST`), 더 크고 높이 튀는 돌 조각, 굵어진 흰 속.
fn quake_wave(w: &World, b: &crate::world::Bolt, p: Vec3) {
    let k = b.def.scale;
    let g = vec3(p.x, w.ground_y(p) + 0.08, p.z);
    let dir = vec3(b.dir.x, 0.0, b.dir.z).normalize_or(Vec3::Z);
    let side = vec3(-dir.z, 0.0, dir.x);
    let dust = Color::new(0.86, 0.80, 0.70, 0.62);
    let core = Color::new(1.0, 0.97, 0.9, 0.9);
    let half = QUAKE_HALF * k;
    trail(g + side * half, g - side * half, 0.40 * k, dust);
    trail(
        g + side * half * 0.8 + dir * 0.05,
        g - side * half * 0.8 + dir * 0.05,
        0.18 * k,
        core,
    );
    // 솟는 흙: 앞머리 가운데에서 위로 가늘어지는 흙빛 기둥(흔들리며 숨 쉰다).
    let breathe = 0.85 + 0.15 * (w.time * 22.0 + b.left * 5.0).sin();
    trail(
        g + Vec3::Y * (QUAKE_CREST * k * breathe),
        g,
        QUAKE_HALF * 1.6 * k,
        with_a(dust, 0.5),
    );
    trail(g, g - dir * (1.8 * k), 0.14 * k, with_a(dust, 0.45));
    let rock = Color::new(0.66, 0.62, 0.58, 1.0);
    for (j, off) in QUAKE_ROCKS.into_iter().enumerate() {
        let bob = ((w.time * 14.0 + b.left * 9.0 + j as f32 * 2.1).sin() * 0.5 + 0.5)
            * QUAKE_CREST
            * 0.8
            * k;
        let at = g + side * off * k + Vec3::Y * bob - dir * 0.1;
        let rot = Quat::from_rotation_y(w.time * 6.0 + j as f32);
        shaded_tris(
            at,
            &octa_faces(
                vec2(QUAKE_ROCK * k, QUAKE_ROCK * 0.8 * k),
                QUAKE_ROCK * 1.4 * k,
                QUAKE_ROCK * 0.6 * k,
            ),
            rock,
            0.7,
            rot,
        );
    }
}

/// 땅 내려찍기의 앞머리 반 폭 · 솟는 흙의 높이 · 돌 조각의 옆 자리와 반지름(크기 배 1, m — 0277). 옆으로는 판정(`BOLT_RADIUS`) 안(시험).
pub(super) const QUAKE_HALF: f32 = 0.33;
pub(super) const QUAKE_CREST: f32 = 0.85;
pub(super) const QUAKE_ROCKS: [f32; 3] = [-0.2, 0.0, 0.2];
pub(super) const QUAKE_ROCK: f32 = 0.13;

/// 신성한 빛(0274 ②b — 망치의 RB): 내려 꽂히기까지 바닥에 금빛 고리가 조여들고 위에서 가는 빛줄이 짙어진다(`SMITE_DELAY`),
/// 꽂히는 순간의 번쩍임·튀는 빛은 이펙트(`Event::Smite`). 바닥 표시는 땅을 따라 접는다. 빨강은 쓰지 않는다.
fn smites(w: &World, fl: &Drape) {
    let gold = Color::new(1.0, 0.9, 0.6, 1.0);
    let white = Color::new(1.0, 0.98, 0.9, 1.0);
    for s in &w.smites {
        if s.look == Look::Seed {
            seed(w, s, fl);
            continue;
        }
        // 꽂힌 뒤: 굵은 흰 기둥이 위로 가늘어지며 옅어진다.
        if s.struck {
            let f = (1.0 + s.left / crate::tuning::SMITE_SHOW).clamp(0.0, 1.0);
            let r = s.radius * (0.35 + 0.25 * (1.0 - f));
            shaft(
                s.pos,
                s.pos + Vec3::Y * 9.0,
                r,
                r * 0.5,
                with_a(white, 0.75 * f),
                with_a(gold, 0.0),
            );
            fl.sector(
                s.pos + Vec3::Y * 0.06,
                s.radius,
                0.0,
                360.0,
                with_a(gold, 0.35 * f),
            );
            continue;
        }
        let k = (1.0 - s.left / crate::tuning::SMITE_DELAY).clamp(0.0, 1.0);
        let g = s.pos + Vec3::Y * 0.06;
        fl.sector(g, s.radius, 0.0, 360.0, with_a(gold, 0.1 + 0.15 * k));
        fl.ring(g, s.radius, 0.0, 360.0, with_a(gold, 0.35 + 0.4 * k));
        fl.ring(
            g,
            s.radius * (1.0 - 0.7 * k),
            0.0,
            360.0,
            with_a(white, 0.5 * k),
        );
        // 위에서 내려오는 빛줄: 가늘게 시작해 꽂히기 직전 굵어진다(빛 기둥의 높이 = 겨눈 땅 위 7 m).
        let r = 0.08 + 0.32 * k * k;
        shaft(
            s.pos,
            s.pos + Vec3::Y * 7.0,
            r,
            r * 1.4,
            with_a(white, 0.15 + 0.45 * k),
            with_a(gold, 0.0),
        );
    }
}

/// 가시 씨앗(0274 ⑤ — 낫의 RB): 손에서 겨눈 자리로 포물선을 그리며 나는 가시 돋친 씨(흙빛 몸 + 민트 가시)와 떨어질 자리의 조여드는
/// 민트 고리. 떨어진 뒤는 옅어지는 민트 판 — 솟는 가시 뿌리는 `skill_spikes`, 번쩍임은 이펙트(`Event::Sprout`). 빨강은 쓰지 않는다.
fn seed(w: &World, s: &crate::world::Smite, fl: &Drape) {
    let mint = Color::new(0.66, 1.0, 0.86, 1.0);
    let g = s.pos + Vec3::Y * 0.06;
    if s.struck {
        let f = (1.0 + s.left / crate::tuning::SMITE_SHOW).clamp(0.0, 1.0);
        fl.sector(g, s.radius, 0.0, 360.0, with_a(mint, 0.25 * f));
        return;
    }
    let k = (1.0 - s.left / crate::tuning::SMITE_DELAY).clamp(0.0, 1.0);
    fl.sector(g, s.radius, 0.0, 360.0, with_a(mint, 0.08 + 0.12 * k));
    fl.ring(g, s.radius, 0.0, 360.0, with_a(mint, 0.35 + 0.4 * k));
    fl.ring(
        g,
        s.radius * (1.0 - 0.7 * k),
        0.0,
        360.0,
        with_a(WHITE, 0.4 * k),
    );
    // 나는 씨: 손(떠난 자리)에서 떨어질 자리로 — 높이는 포물선(가운데에서 1.6 m 더 높다).
    let at = s.from.lerp(s.pos + Vec3::Y * 0.3, k) + Vec3::Y * (4.0 * k * (1.0 - k) * 1.6);
    let rot = Quat::from_rotation_y(w.time * 9.0) * Quat::from_rotation_x(w.time * 6.0);
    shaded_tris(
        at,
        &ball_faces(0.13),
        Color::new(0.46, 0.34, 0.22, 1.0),
        0.8,
        rot,
    );
    shaded_tris(
        at,
        &octa_faces(vec2(0.05, 0.05), 0.22, 0.22),
        mint,
        0.9,
        rot,
    );
}

fn bell_shards(w: &World, fl: &Drape) {
    // 종 조각(0125): 돌며 나는 청동 조각 — 되받아친 것은 호박빛(패링이 "돌려보냈다"로 읽히게).
    // 0242: 1px 선 두 개 → 음영 있는 납작한 청동 조각(면 여덟) + 꼬리 띠.
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
        glow_disc(sh.pos, 0.42, glow);
        trail(
            sh.pos,
            sh.pos - sh.dir * 0.9,
            0.3,
            with_a(glow, glow.a * 1.6),
        );
        shaded_tris(
            sh.pos,
            &octa_faces(vec2(0.22, 0.05), 0.15, 0.12),
            core,
            0.7,
            rot,
        );
        fl.shadow(sh.pos, 0.18);
    }
}

/// 뿌리 솟기(0214)의 뿌리: 문지기의 뿌리 기둥과 같은 꼴(굵은 밑동 → 뾰족한 끝)을 따뜻한 흙빛으로 — 빠르게 솟았다 가라앉는다.
fn skill_spikes(w: &World) {
    use crate::world::SPIKE_LIFE;
    const BODY: Color = Color::new(0.36, 0.25, 0.16, 1.0);
    const TIP: Color = Color::new(0.80, 0.62, 0.36, 1.0);
    for (i, sp) in w.skill_spikes.iter().enumerate() {
        let k = sp.age / SPIKE_LIFE;
        let rise = if k < 0.12 {
            k / 0.12
        } else if k > 0.6 {
            ((1.0 - k) / 0.4).max(0.0)
        } else {
            1.0
        };
        let h = 1.2 * rise * if i % 2 == 0 { 1.0 } else { 0.8 };
        if h < 0.02 {
            continue;
        }
        let (body, tip) = spike_faces(0.2, 0.08, h, 0.38 * rise, sp.lean);
        let turn = Quat::from_rotation_y(i as f32 * 0.9);
        shaded_tris(sp.pos, &body, BODY, 0.8, turn);
        shaded_tris(sp.pos, &tip, TIP, 0.9, turn);
    }
}

fn carts(w: &World, alpha: f32) {
    // 레일 수레(0208, 회색 박스 — 그림은 코드 상자): 녹슨 짐칸 + 바퀴 넷 + 짐(돌덩이). 땅 높이를 따른다.
    const BODY: Color = Color::new(0.44, 0.31, 0.24, 1.0);
    const RIM: Color = Color::new(0.30, 0.22, 0.18, 1.0);
    const WHEEL: Color = Color::new(0.24, 0.23, 0.25, 1.0);
    const LOAD: Color = Color::new(0.42, 0.41, 0.44, 1.0);
    for c in &w.carts {
        let k = c.prev + (c.at - c.prev) * alpha;
        let mut p = c.pos_at(k);
        p.y = w.ground.as_ref().map_or(p.y, |g| g.height(p));
        let (hw, hl) = (crate::tuning::CART_HALF_W, crate::tuning::CART_HALF_L);
        let along_z = (c.b - c.a).z.abs() >= (c.b - c.a).x.abs();
        let size = |x: f32, y: f32, l: f32| {
            if along_z {
                vec3(x, y, l)
            } else {
                vec3(l, y, x)
            }
        };
        shaded_box(
            p + Vec3::Y * 0.75,
            size(hw * 1.8, 0.7, hl * 1.8),
            BODY,
            0.55,
        );
        shaded_box(p + Vec3::Y * 1.12, size(hw * 1.9, 0.08, hl * 1.9), RIM, 0.5);
        shaded_box(p + Vec3::Y * 1.2, size(hw * 1.1, 0.35, hl * 1.0), LOAD, 0.6);
        for (sx, sl) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
            let o = if along_z {
                vec3(sx * hw * 0.8, 0.3, sl * hl * 0.6)
            } else {
                vec3(sl * hl * 0.6, 0.3, sx * hw * 0.8)
            };
            shaded_box(p + o, size(0.18, 0.5, 0.5), WHEEL, 0.5);
        }
    }
}

fn bombs(w: &World, alpha: f32) {
    // 수류탄(0051): 어두운 덩이 + 도화선이 짧아질수록 빨리 깜빡이는 불. 뿌리 씨앗(0231): 갈색 씨 + 깜빡이는 연두 싹.
    for bm in &w.bombs {
        let p = bm.prev.lerp(bm.pos, alpha);
        let seed = bm.root > 0.0;
        let (body, spark) = if seed {
            (
                Color::new(0.42, 0.30, 0.20, 1.0),
                Color::new(0.66, 1.0, 0.72, 1.0),
            )
        } else {
            (
                Color::new(0.22, 0.19, 0.24, 1.0),
                Color::new(1.0, 0.8, 0.4, 1.0),
            )
        };
        // 0242: 납작한 마름모 → 음영 있는 덩이(씨는 길쭉하게 돈다) + 불 뒤의 빛무리.
        let r = if seed { 0.15 } else { 0.19 };
        if seed {
            let spin = Quat::from_rotation_y(w.time * 5.0) * Quat::from_rotation_x(w.time * 3.0);
            shaded_tris(
                p,
                &octa_faces(vec2(r, r), r * 1.6, r * 1.3),
                body,
                0.6,
                spin,
            );
        } else {
            shaded_tris(p, &ball_faces(r), body, 0.6, Quat::IDENTITY);
        }
        let hz = 4.0 + 10.0 * (1.0 - (bm.fuse / 1.5).clamp(0.0, 1.0));
        if (w.time * hz) as i32 % 2 == 0 {
            let at = p + Vec3::Y * (r + 0.08);
            glow_disc(at, 0.3, with_a(spark, 0.45));
            spark4(at, 0.36, spark);
        }
    }
}

/// 들이받기에 모인 힘(0276 — 하데스의 방패): 막는 동안 발밑의 쇳빛 테가 둘레를 따라 차오르고, 들이받을 수 있으면(`RUSH_MIN`) 앞으로
/// 들이받을 길이 옅게 선다(모인 만큼 길다). 가득이면 테가 금빛으로 깜빡인다. 빨강 없음.
fn rush_charge(w: &World, pp: Vec3, fl: &Drape) {
    if !matches!(w.player.state, PState::Block { .. }) {
        return;
    }
    let k = w.player.class.rush.clamp(0.0, 1.0);
    let steel = Color::new(0.86, 0.9, 1.0, 1.0);
    let gold = Color::new(1.0, 0.86, 0.5, 1.0);
    let c = pp + Vec3::Y * 0.05;
    fl.ring(c, 0.95, 0.0, 360.0, with_a(steel, 0.2));
    let full = k >= 1.0;
    let blink = full && (w.time * 10.0) as i32 % 2 == 0;
    let col = if full { gold } else { steel };
    // 안에서 바깥 테로 자라는 테(모인 만큼).
    fl.ring(
        c,
        0.25 + 0.7 * k,
        0.0,
        360.0,
        with_a(col, if blink { 1.0 } else { 0.4 + 0.4 * k }),
    );
    if k >= crate::tuning::RUSH_MIN {
        let (d0, d1) = crate::tuning::RUSH_DIST;
        let dist = d0 + (d1 - d0) * k;
        fl.lane(
            c,
            w.player.facing_dir(),
            dist,
            0.45,
            with_a(col, 0.08 + 0.12 * k),
        );
    }
}

fn cast_charge(w: &World, pp: Vec3, t: &Tuning, fl: &Drape) {
    // 원거리 강공격 모으기(0056): 손에 빛이 자라고 발밑 고리가 차오른다(근접 강공격과 같은 문법), 다 모이면 깜빡인다.
    if let PState::CastCharge { t: e } = w.player.state {
        // 두 손으로 엮는 지팡이(0276)는 빨리 모인다 — 월드와 같은 셈.
        let full = w.cast_full(t).max(1e-3);
        let k = (e / full).clamp(0.0, 1.0);
        let (core, glow) = bolt_colors(wpn_look(w));
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
        // 활(0274 ②b): 반쯤 당긴 단계의 금 — 차오르는 고리가 넘으면 그 단계(자세 피해·거리), 끝까지면 깜빡임(꿰뚫고 오금).
        if let Some(crate::weapons::Rhythm::Draw { mid, .. }) = w.player.weapon.rhythm() {
            let m = (mid / full).clamp(0.0, 1.0);
            let on = k >= m;
            fl.ring(
                pp + Vec3::Y * 0.05,
                0.2 + 0.7 * m,
                0.0,
                360.0,
                with_a(core, if on { 0.85 } else { 0.35 }),
            );
        }
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
                // 터지는 자리(0218 — 튕긴 뒤): 떨굴 자리 `to`가 아니다.
                let Some(c) = w.grenade_lands(to) else {
                    return;
                };
                let c = c + Vec3::Y * 0.08;
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
        // 뿌리 씨앗(0231)은 뿌리의 빛(`status::INK`의 뿌리 — 민트)으로.
        let c = if b.root {
            let [r, g, bl] = crate::status::INK[1];
            Color::new(r, g, bl, 0.9 * (1.0 - k))
        } else {
            Color::new(1.0, 0.8, 0.45, 0.9 * (1.0 - k))
        };
        fl.ring(
            b.pos + Vec3::Y * 0.1,
            b.radius * (0.4 + 0.6 * k),
            0.0,
            360.0,
            c,
        );
    }
}

fn swing_crescent(w: &World, pp: Vec3, t: &Tuning, wpn: &crate::weapons::Weapon) {
    // 휘두름의 초승달(0141): 활성 구간 동안 칼끝이 지난 자리에 밝은 초승달이 따라가고 0.12초에 걸쳐 옅어진다 — 무기가 지나간 호
    // (판정 부채꼴은 아래 박자 부채꼴이 그대로 맡는다). 박자 타면 금빛.
    if t[crate::tuning::P::RenderFx] >= 0.5 {
        if let PState::Attack { t: e, hit, .. } = w.player.state {
            // 몰아치기(0158 G2) 동안은 평타의 시계가 빨라진다 — 빛도 같은 배로.
            let e = e * w.swing_rate();
            let (s0, a0, _) = crate::world::hit_times(t, wpn, hit);
            if let Some(h) = w
                .hit_of(hit)
                .filter(|_| e >= s0 && e < s0 + a0 + SLASH_FADE)
            {
                let q = ((e - s0) / a0.max(1e-3)).clamp(0.0, 1.0);
                // 활성 뒤: 휘두른 쪽으로(세로는 양 끝에서) 줄어들며 옅어진다(0195).
                let gone = ((e - s0 - a0) / SLASH_FADE).clamp(0.0, 1.0);
                let fade = 1.0 - gone * gone;
                let arc = crate::world::swing_arc(
                    h.shape,
                    t[crate::tuning::P::AttackArc] * h.arc,
                    t[crate::tuning::P::AttackRange] * h.reach,
                );
                let col = if w.player.beat.swing_beat {
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
                    (q, gone),
                    with_a(col, col.a * fade),
                    1.0,
                );
            }
        }
    }
}

/// 박자 부채꼴(0087) — 이을 타가 없는 타는 활성 동안 판정 부채꼴만.
fn beat_fan(w: &World, pp: Vec3, t: &Tuning, wpn: &crate::weapons::Weapon, fl: &Drape) {
    let swing_col = if w.player.beat.swing_beat {
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
        if let Some(h) = w.hit_of(hit) {
            if e >= s0 && e < s0 + a0 {
                let arc = crate::world::swing_arc(
                    h.shape,
                    t[crate::tuning::P::AttackArc] * h.arc,
                    t[crate::tuning::P::AttackRange] * h.reach,
                );
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
    if w.player.beat.since < BEAT_RING_LIFE {
        let k = (w.player.beat.since / BEAT_RING_LIFE).clamp(0.0, 1.0);
        let big = 1.0 + 0.25 * (w.player.beat.chain.min(3) as f32 - 1.0);
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
    if let (PState::Heavy { t: e, full, .. }, Some(h)) = (w.player.state, w.heavy_of()) {
        let (s0, a0, _) = crate::world::heavy_times(t, wpn);
        let (s1, arc) = (
            s0 + a0,
            crate::world::swing_arc(
                h.shape,
                t[crate::tuning::P::HeavyArc] * h.arc,
                t[crate::tuning::P::HeavyRange] * h.reach,
            ),
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
        // 강타의 초승달(0145): 평타보다 두껍고 오래 남는다 — 다 모았으면 호박빛. 쪽은 동작의 표대로(0195 — 지팡이 강타는 가로로 돈다,
        // 내려치기는 가운데에서 펴졌다가 양 끝에서부터 사라진다).
        if t[crate::tuning::P::RenderFx] >= 0.5 && e >= s0 && e < s1 + HEAVY_SLASH_FADE {
            let q = ((e - s0) / a0.max(1e-3)).clamp(0.0, 1.0);
            let gone = ((e - s1) / HEAVY_SLASH_FADE).clamp(0.0, 1.0);
            let fade = 1.0 - gone;
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
                crate::weapons::sweep_of(h.anim_heavy),
                (q, gone),
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
                    let len = u.len_at(w.player.skill.ult_rank);
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
            // 질풍 찌르기(0272 2단계): 세계가 멈춘 동안 찌를 줄의 테가 먼저 선다(한 줄기와 같은 꼴 — 강철빛, 좁게), 찌르는 동안은
            // 줄의 면이 옅게 남는다(찌름마다의 빛은 이펙트가).
            crate::skills::Ult::Gale => {
                let u = &crate::skills::GALE;
                let steel = Color::new(0.80, 0.86, 1.0, 1.0);
                let dir = w.player.facing_dir();
                let len = w.lane_reach(w.player.pos, dir, u.len_at(w.player.skill.ult_rank));
                let at = pp + Vec3::Y * 0.06;
                if !done {
                    let frozen = 1.0 - (w.hitstop / u.freeze.max(1e-3)).clamp(0.0, 1.0);
                    let k = if e > 0.0 { 1.0 } else { frozen };
                    fl.lane(
                        at,
                        dir,
                        len * k,
                        u.half * 2.0,
                        with_a(steel, 0.10 + 0.1 * k),
                    );
                    fl.lane_edge(at, dir, len * k, u.half * 2.0, with_a(steel, 0.75));
                } else {
                    let k = ((e - u.startup) / u.recover().max(1e-3)).clamp(0.0, 1.0);
                    fl.lane_edge(at, dir, len, u.half * 2.0, with_a(steel, 0.5 * (1.0 - k)));
                }
            }
            // 깨어나는 숲(0274 ⑤): 세계가 멈춘 동안 고리 셋의 자리(뿌리빛 테)가 몸에서 바깥으로 차례로 서고, 솟는 동안 아직 솟지 않은
            // 고리의 테가 남는다(솟는 순간의 빛 · 뿌리는 이펙트 · `skill_spikes`가).
            crate::skills::Ult::Grove => {
                let u = &crate::skills::GROVE;
                let mint = Color::new(0.66, 1.0, 0.86, 1.0);
                let rank = w.player.skill.ult_rank;
                let at = pp + Vec3::Y * 0.06;
                let frozen = 1.0 - (w.hitstop / u.freeze.max(1e-3)).clamp(0.0, 1.0);
                for k in 0..u.pulses() {
                    let r = u.radius_at(k, rank);
                    let due = u.startup + u.pulse_at(k);
                    if e > due + 0.05 {
                        continue;
                    }
                    let grow = if e > 0.0 {
                        1.0
                    } else {
                        (frozen * u.pulses() as f32 - k as f32).clamp(0.0, 1.0)
                    };
                    fl.ring(at, r * grow, 0.0, 360.0, with_a(mint, 0.7 * grow));
                }
            }
            // 치른 값(0274 ⑥): 세계가 멈춘 동안 저주가 번질 둘레의 균열빛(하늘) 테가 몸에서 바깥으로 자라고, 번진 뒤 터지기까지는 그 테
            // 안쪽에서 울림빛(쇳빛) 테가 조여든다(터지는 순간의 빛은 이펙트가). 보라 · 빨강은 쓰지 않는다.
            crate::skills::Ult::Price => {
                let u = &crate::skills::PRICE;
                let rift = Color::new(0.50, 0.80, 1.0, 1.0);
                let knell = Color::new(0.80, 0.84, 0.94, 1.0);
                let r = u.radius_at(w.player.skill.ult_rank);
                let at = pp + Vec3::Y * 0.06;
                let frozen = 1.0 - (w.hitstop / u.freeze.max(1e-3)).clamp(0.0, 1.0);
                if e <= 0.0 {
                    fl.ring(at, r * frozen, 0.0, 360.0, with_a(rift, 0.75 * frozen));
                } else if e < u.startup + u.gap {
                    let k = ((e - u.startup) / u.gap.max(1e-3)).clamp(0.0, 1.0);
                    fl.sector(at, r, 0.0, 360.0, with_a(rift, 0.08));
                    fl.ring(at, r, 0.0, 360.0, with_a(rift, 0.6));
                    if e > u.startup {
                        fl.ring(
                            at,
                            r * (1.0 - 0.8 * k),
                            0.0,
                            360.0,
                            with_a(knell, 0.4 + 0.5 * k),
                        );
                    }
                }
            }
            // 칼바람(0274 ④): 세계가 멈춘 동안 휩쓸 둘레의 잔불빛 테가 몸에서 바깥으로 자라고, 휘도는 동안 그 테가 옅게 남는다
            // (휩쓸기마다의 빛은 이펙트가).
            crate::skills::Ult::Tempest => {
                let u = &crate::skills::TEMPEST;
                let ember = Color::new(1.0, 0.72, 0.36, 1.0);
                let r = u.radius_at(w.player.skill.ult_rank);
                let at = pp + Vec3::Y * 0.06;
                if !done {
                    let frozen = 1.0 - (w.hitstop / u.freeze.max(1e-3)).clamp(0.0, 1.0);
                    let k = if e > 0.0 { 1.0 } else { frozen };
                    fl.ring(at, 0.4 + (r - 0.4) * k, 0.0, 360.0, with_a(ember, 0.85));
                    fl.sector(at, r * k, 0.0, 360.0, with_a(ember, 0.10 * k));
                } else {
                    let k = ((e - u.startup) / u.recover().max(1e-3)).clamp(0.0, 1.0);
                    fl.ring(at, r, 0.0, 360.0, with_a(ember, 0.55 * (1.0 - k)));
                }
            }
            // 그림자 습격(0274 ③): 세계가 멈춘 동안 고른 적의 발밑에 강철빛 테가 조여들고(숨어들 적), 찌르는 동안 그 테가 옅게 남는다.
            crate::skills::Ult::Ambush => {
                let u = &crate::skills::AMBUSH;
                let steel = Color::new(0.80, 0.86, 1.0, 1.0);
                if let Some(en) = w.player.skill.ambush_on.and_then(|i| w.enemies.get(i)) {
                    let c = vec3(en.pos.x, w.ground_y(en.pos) + 0.06, en.pos.z);
                    let r = en.radius + 0.4;
                    if !done {
                        let frozen = 1.0 - (w.hitstop / u.freeze.max(1e-3)).clamp(0.0, 1.0);
                        let k = if e > 0.0 { 1.0 } else { frozen };
                        fl.ring(c, r + 1.6 * (1.0 - k), 0.0, 360.0, with_a(steel, 0.85));
                        fl.sector(c, r, 0.0, 360.0, with_a(steel, 0.12 * k));
                    } else {
                        let k = ((e - u.startup) / u.recover().max(1e-3)).clamp(0.0, 1.0);
                        fl.ring(c, r, 0.0, 360.0, with_a(steel, 0.6 * (1.0 - k)));
                    }
                }
            }
            // 큰 종(0270 C3): 세계가 멈춘 동안 둘레 끝에서 발밑으로 조여드는 민트 테(울릴 둘레를 먼저 보인다), 울린 뒤 번지는 테.
            crate::skills::Ult::Bell => {
                let u = &crate::skills::BELL;
                let r = u.radius_at(w.player.skill.ult_rank);
                let mint = Color::new(0.66, 1.0, 0.86, 1.0);
                if !done {
                    let frozen = 1.0 - (w.hitstop / u.freeze.max(1e-3)).clamp(0.0, 1.0);
                    let k = if e > 0.0 { 1.0 } else { frozen * 0.6 };
                    fl.ring(pp + Vec3::Y * 0.06, r, 0.0, 360.0, with_a(mint, 0.6));
                    fl.ring(
                        pp + Vec3::Y * 0.06,
                        r * (1.0 - k) + 0.4,
                        0.0,
                        360.0,
                        with_a(gold, 0.9),
                    );
                } else {
                    let k = ((e - u.startup) / u.recover.max(1e-3)).clamp(0.0, 1.0);
                    fl.sector(
                        pp + Vec3::Y * 0.05,
                        r,
                        0.0,
                        360.0,
                        with_a(mint, 0.14 * (1.0 - k)),
                    );
                }
            }
            // 눈보라(0270 C3): 시동 동안 둘레로 커지는 서리빛 테(일 자리) — 인 뒤는 따르는 마당의 그림.
            crate::skills::Ult::Blizzard => {
                let u = &crate::skills::BLIZZARD;
                if !done {
                    let k = (e / u.startup.max(1e-3)).clamp(0.0, 1.0);
                    fl.ring(
                        pp + Vec3::Y * 0.06,
                        u.radius * (0.3 + 0.7 * k),
                        0.0,
                        360.0,
                        with_a(Color::new(0.8, 0.95, 1.0, 1.0), 0.85),
                    );
                }
            }
            // 화살비(0272): 시동 동안 내릴 자리의 테가 먼저 선다(세계가 멈춘 동안 읽힌다) — 바깥 테 + 안에서 차오르는 면.
            // 내린 뒤는 놓인 마당의 그림.
            crate::skills::Ult::Rain => {
                let u = &crate::skills::RAIN;
                if !done {
                    let k = (e / u.startup.max(1e-3)).clamp(0.0, 1.0);
                    let at = w.player.skill.rain_at;
                    let c = vec3(at.x, w.ground_y(at), at.z);
                    let root = Color::new(0.82, 0.66, 0.96, 1.0);
                    fl.ring(c + Vec3::Y * 0.06, u.radius, 0.0, 360.0, with_a(root, 0.9));
                    fl.sector(
                        c + Vec3::Y * 0.05,
                        u.radius * (0.25 + 0.75 * k),
                        0.0,
                        360.0,
                        with_a(root, 0.12),
                    );
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
    let guard = w.player.parry.release_guard > 0.0
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
    if w.player.parry.chain_left > 0.0 && !matches!(w.player.state, PState::Parry { .. }) {
        let stand = w.tree.has(Cell::OneWhoStands)
            && w.player.parry.chain_n >= crate::progression::cell::STAND_CHAIN;
        let (fill_c, edge_c) = if stand {
            ((1.0, 0.78, 0.4), (1.0, 0.85, 0.5))
        } else {
            ((0.6, 0.85, 1.0), (0.75, 0.9, 1.0))
        };
        let c = crate::world::parry_front_cos(true, w.player.parry.chain_n);
        let arc = if c < -1.0 {
            360.0
        } else {
            (c.acos().to_degrees() * 2.0).min(360.0)
        };
        let fade = (w.player.parry.chain_left / crate::tuning::PARRY_CHAIN).min(1.0);
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
    if w.player.parry.primed_heavy {
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
    if let PState::Land { t: e, dur, .. } = w.player.state {
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
    if w.player.beat.since_perfect < PERFECT_RING_LIFE {
        let k = (w.player.beat.since_perfect / PERFECT_RING_LIFE).clamp(0.0, 1.0);
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
    if w.player.parry.since < 0.35 {
        let k = (w.player.parry.since / 0.35).clamp(0.0, 1.0);
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
        if en.marks.frail > 0.0 && en.alive() {
            let p = en.prev.lerp(en.pos, alpha);
            fl.ring(
                p + Vec3::Y * 0.04,
                en.radius + 0.25,
                0.0,
                360.0,
                Color::new(0.66, 1.0, 0.86, 0.3 + 0.5 * en.marks.frail.min(1.0)),
            );
        }
    }
}

fn ail_marks(w: &World, alpha: f32, fl: &Drape) {
    // 타입 상태(0223 V2): 머리 위 작은 마름모 줄 — 쌓이는 게이지는 흐리고 작게 자라고, 걸린 것은 밝게 뛴다(화상은 겹만큼).
    // 발밑: 묶인 적은 민트 고리 둘, 금 간 적은 하늘빛 고리(남은 시간만큼 옅어진다).
    for en in &w.enemies {
        if !en.alive() {
            continue;
        }
        let a = &en.ail;
        let p = en.prev.lerp(en.pos, alpha);
        let ink = |i: usize| {
            let [r, g, b] = crate::status::INK[i];
            Color::new(r, g, b, 1.0)
        };
        if a.bound > 0.0 {
            let k = 0.35 + 0.5 * a.bound.min(1.0);
            let ink = ink(1);
            for r in [0.2, 0.34] {
                fl.ring(
                    p + Vec3::Y * 0.05,
                    en.radius + r,
                    0.0,
                    360.0,
                    Color::new(ink.r, ink.g, ink.b, k),
                );
            }
        }
        if a.rift > 0.0 {
            let ink = ink(3);
            let beat = 0.75 + 0.25 * (w.time * 9.0).sin();
            fl.ring(
                p + Vec3::Y * 0.06,
                en.radius + 0.48,
                0.0,
                360.0,
                Color::new(ink.r, ink.g, ink.b, (0.3 + 0.5 * a.rift.min(1.0)) * beat),
            );
        }
        // 오금(0274 ②b — 끝까지 당긴 화살): 발밑에 끊긴 연두 고리(활의 화살 빛) — 넷으로 끊겨 절뚝이듯 천천히 돈다, 남은 시간만큼 옅어진다.
        if en.marks.hamstring > 0.0 {
            let k = 0.3 + 0.5 * (en.marks.hamstring / 1.0).min(1.0);
            let turn = w.time * 0.7;
            for q in 0..4 {
                fl.ring(
                    p + Vec3::Y * 0.055,
                    en.radius + 0.28,
                    turn + q as f32 * std::f32::consts::FRAC_PI_2,
                    50.0,
                    Color::new(0.85, 1.0, 0.72, k),
                );
            }
        }
        // 저주의 표(0274 ⑥ — 저주 새기기): 발밑에 셋으로 끊긴 하늘빛 고리가 거꾸로 돌고, 머리 위에 작은 잿빛 마름모가 하늘빛 테를 두르고
        // 뜬다 — 표가 다해 갈수록 옅어진다(옮을 표다). 보라 · 빨강은 쓰지 않는다.
        if en.marks.hex > 0.0 {
            let k = 0.35 + 0.5 * (en.marks.hex / 2.0).min(1.0);
            let sky = Color::new(0.50, 0.80, 1.0, k);
            let turn = -w.time * 1.4;
            for q in 0..3 {
                fl.ring(
                    p + Vec3::Y * 0.06,
                    en.radius + 0.38,
                    turn + q as f32 * std::f32::consts::TAU / 3.0,
                    70.0,
                    sky,
                );
            }
            let top = p + Vec3::Y * (en.height + 0.55 + 0.06 * (w.time * 3.0).sin());
            gem(top, 0.34, with_a(sky, 0.5 * k));
            gem(top, 0.2, Color::new(0.40, 0.39, 0.42, k));
        }
        // 0270 B: 언 적은 발밑에 두꺼운 얼음 고리 둘, 흡수 표지는 숨 쉬는 풀빛 고리, 기절한 적은 머리 둘레를 도는 레몬 별 셋.
        if a.frozen > 0.0 {
            let ink = ink(6);
            for (r, k) in [(0.12, 0.85), (0.26, 0.55)] {
                fl.ring(
                    p + Vec3::Y * 0.05,
                    en.radius + r,
                    0.0,
                    360.0,
                    Color::new(ink.r, ink.g, ink.b, k),
                );
            }
        }
        if a.drain > 0.0 {
            let ink = ink(7);
            let beat = 0.6 + 0.4 * (w.time * 4.0).sin().abs();
            fl.ring(
                p + Vec3::Y * 0.07,
                en.radius + 0.36,
                0.0,
                360.0,
                Color::new(ink.r, ink.g, ink.b, (0.25 + 0.4 * a.drain.min(1.0)) * beat),
            );
        }
        if a.stun > 0.0 {
            let ink = ink(4);
            let c = p + Vec3::Y * (en.height + 0.25);
            for k in 0..3 {
                let ang = w.time * 6.0 + k as f32 * std::f32::consts::TAU / 3.0;
                let r = en.radius * 0.8 + 0.2;
                spark4(c + vec3(ang.sin() * r, 0.0, ang.cos() * r), 0.22, ink);
            }
        }
        // 머리 위 줄: (빛, 크기, 진하기, 꼴) — 걸린 것 먼저, 그다음 쌓이는 게이지.
        use crate::status::{Mark, MARK};
        let mut pips: Vec<(Color, f32, f32, Mark)> = Vec::new();
        let beat = 1.0 + 0.18 * (w.time * 8.0).sin();
        for _ in 0..a.burn {
            pips.push((ink(0), 0.26 * beat, 1.0, MARK[0]));
        }
        if a.bound > 0.0 {
            pips.push((ink(1), 0.26 * beat, 1.0, MARK[1]));
        }
        if a.toll_in.is_some() {
            pips.push((ink(2), 0.3 * beat, 1.0, MARK[2]));
        }
        if a.rift > 0.0 {
            pips.push((ink(3), 0.26 * beat, 1.0, MARK[3]));
        }
        for (i, on) in [(4, a.stun), (5, a.bleed), (6, a.frozen), (7, a.drain)] {
            if on > 0.0 {
                pips.push((ink(i), 0.26 * beat, 1.0, MARK[i]));
            }
        }
        for (i, g) in a.gauge.iter().enumerate() {
            if *g > 0.02 {
                pips.push((ink(i), 0.1 + 0.12 * g.min(1.0), 0.6, MARK[i]));
            }
        }
        if pips.is_empty() {
            continue;
        }
        let top = p + Vec3::Y * (en.height + 0.55);
        let (right, _) = facing_axes(top);
        let gap = 0.3;
        let x0 = -(pips.len() as f32 - 1.0) * 0.5 * gap;
        for (i, (ink, size, k, mark)) in pips.into_iter().enumerate() {
            let at = top + right * (x0 + i as f32 * gap);
            let dark = Color::new(0.07, 0.06, 0.12, 0.8 * k);
            let col = Color::new(ink.r, ink.g, ink.b, k);
            match mark {
                Mark::Gem => {
                    gem(at, size + 0.07, dark);
                    gem(at, size, col);
                }
                // 별(충격): 네 갈래.
                Mark::Star => {
                    spark4(at, size * 1.5 + 0.08, dark);
                    spark4(at, size * 1.5, col);
                }
                // 조각(날카로움·서리): 위아래로 길쭉한 두 마름모.
                Mark::Shard => {
                    for dy in [-0.4, 0.4] {
                        gem(at + Vec3::Y * size * dy, size * 0.7 + 0.06, dark);
                        gem(at + Vec3::Y * size * dy, size * 0.7, col);
                    }
                }
                // 방울(흡수): 아래가 큰 마름모 + 위의 작은 끝.
                Mark::Drop => {
                    gem(at, size + 0.07, dark);
                    gem(at, size, col);
                    gem(at + Vec3::Y * size * 0.9, size * 0.4, col);
                }
            }
        }
    }
}

/// 언 적의 얼음(0270 C5 — 사장 "넣기"): 몸에서 바깥으로 돋은 얼음 조각 다섯과 발치의 낮은 조각 셋 — 면 음영이 있는 길쭉한 팔면체
/// (`octa_faces` · `shaded_tris`). 몸빛은 `actors::frozen_tint`. 얼음이 풀리는 끝 `ICE_MELT`초에 줄어들고, 깨지는 순간(`Event::Shatter` —
/// `Ail::shatter`가 `frozen`을 0으로) 사라진다 — 흩어지는 얼음은 이펙트(`fx`)의 몫. 빨강은 쓰지 않는다.
fn ice_coats(w: &World, alpha: f32) {
    let [r, g, b] = crate::status::INK[crate::status::Kind::Frost.slot().unwrap_or(6)];
    let pale = |k: f32| Color::new(r + (1.0 - r) * k, g + (1.0 - g) * k, b + (1.0 - b) * k, 1.0);
    let (body, tip) = (pale(ICE_PALE), pale(0.85));
    for (n, en) in w.enemies.iter().enumerate() {
        if !en.alive() || en.ail.frozen <= 0.0 {
            continue;
        }
        let grow = (en.ail.frozen / ICE_MELT).min(1.0);
        let p = en.prev.lerp(en.pos, alpha);
        let scale = en.height.clamp(0.8, 4.0) * grow;
        // 적마다 다른(그러나 프레임마다 같은) 돌림 — 모든 언 적이 같은 꼴로 보이지 않게.
        let turn = en.facing + n as f32 * 1.7;
        for (j, &(ang, up, at, len)) in ICE_SHARDS.iter().enumerate() {
            let a = turn + ang;
            let out = vec3(a.cos(), 0.0, -a.sin());
            let dir = (out + Vec3::Y * up).normalize();
            // 몸 둘레(반지름의 0.8) · 키의 `at` 높이에서 바깥으로.
            let root = p + out * en.radius * 0.8 + Vec3::Y * en.height * at;
            let l = len * scale;
            let w = l * 0.26;
            let rot = Quat::from_rotation_arc(Vec3::Y, dir);
            let faces = octa_faces(vec2(w, w * 0.8), l * 0.75, l * 0.25);
            shaded_tris(root + dir * l * 0.2, &faces, body, 0.35, rot);
            // 끝의 빛 한 점(작은 마름모) — 짝수 조각만.
            if j % 2 == 0 {
                gem(root + dir * l * 0.9, 0.09 * scale.min(1.6), tip);
            }
        }
    }
}

/// 얼음 조각(0270 C5 — `ice_coats`): (몸 둘레의 각 · 위로 드는 몫 · 키에서의 높이 · 키 1m당 길이). 바닥 가까이의 셋은 발치의 조각.
const ICE_SHARDS: [(f32, f32, f32, f32); 8] = [
    (0.3, 0.5, 0.7, 0.5),
    (2.2, 0.3, 0.5, 0.45),
    (4.1, 0.8, 0.8, 0.42),
    (5.4, 0.2, 0.35, 0.42),
    (3.0, 0.4, 0.6, 0.45),
    (1.2, 2.2, 0.05, 0.45),
    (3.3, 2.6, 0.05, 0.4),
    (5.0, 2.0, 0.05, 0.42),
];
/// 얼음이 풀리기 전 줄어드는 초 · 조각의 빛(서리 빛을 흰빛 쪽으로 이만큼).
const ICE_MELT: f32 = 0.3;
const ICE_PALE: f32 = 0.6;

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
                let at = p + vec3(a.cos() * r, en.height + 0.3, a.sin() * r);
                // 0242: 별 뒤에 작은 빛무리 — 마른 십자만으로는 습작처럼 보였다.
                glow_disc(at, 0.2, Color::new(1.0, 0.85, 0.4, 0.35 * fade));
                spark4(at, 0.34, Color::new(1.0, 0.92, 0.45, fade));
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

/// 초승달이 덮는 각(라디안): `(꼬리, 머리)` 쌍들. 머리 = 밝고 두꺼운 끝, 꼬리 = 옅어지는 끝(0195 — 사장 "기본적으로 휘두르는
/// 방향대로, 수직 기술은 양끝에서부터 사라지게").
/// - 가로 베기: 머리 = 칼끝이 지금 있는 각(활성 `q`에 한쪽 끝에서 반대쪽으로 — 캐릭터의 오른쪽 = 바라보는 쪽 − 반각), 꼬리는 지나온
///   쪽(길어야 `SLASH_TAIL`). 활성 뒤(`gone` 0 → 1)에는 꼬리가 머리를 따라잡는다 — 휘두른 쪽으로 사라진다.
/// - 찌르기·세로(가운데): 머리 = 바라보는 쪽(칼이 내려온 자리), 꼬리 = 양 끝. 활성 앞쪽에 가운데에서 양쪽으로 펴지고
///   (`CENTER_SPREAD`), 활성 뒤에는 양 끝에서부터 가운데로 줄어 사라진다.
pub fn slash_arcs(
    yaw: f32,
    arc: f32,
    sweep: crate::weapons::Sweep,
    q: f32,
    gone: f32,
) -> Vec<(f32, f32)> {
    use crate::weapons::Sweep;
    let half = arc * 0.5;
    let (q, gone) = (q.clamp(0.0, 1.0), gone.clamp(0.0, 1.0));
    match sweep {
        Sweep::RightToLeft => {
            let head = yaw - half + arc * q;
            let tail = (head - SLASH_TAIL).max(yaw - half);
            vec![(tail + (head - tail) * gone, head)]
        }
        Sweep::LeftToRight => {
            let head = yaw + half - arc * q;
            let tail = (head + SLASH_TAIL).min(yaw + half);
            vec![(tail + (head - tail) * gone, head)]
        }
        Sweep::Center => {
            let w = half * (q / CENTER_SPREAD).min(1.0) * (1.0 - gone);
            vec![(yaw + w, yaw), (yaw - w, yaw)]
        }
    }
}
/// 세로 초승달이 다 펴지는 활성의 몫.
const CENTER_SPREAD: f32 = 0.4;
const SLASH_TAIL: f32 = 2.2;

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
    (q, gone): (f32, f32),
    col: Color,
    weight: f32,
) {
    const N: usize = 14;
    let hot = Color::new(1.0, 1.0, 0.96, col.a);
    for (tail, head) in slash_arcs(yaw, arc, sweep, q, gone) {
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
                let k = ink.a * s * SLASH_ALPHA;
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
fn wpn_look(w: &World) -> Look {
    w.rb_bolts().1.look
}

/// 터치로 끌어 겨누는 동안의 땅 위 미리보기(0217 — 브롤스타즈 꼴): 떼면 무엇이 어디로 가는지.
/// 쏘기 = 탄이 닿는 거리의 줄, 수류탄 = 떨굴 자리의 폭발 고리, 스킬 = 그 꼴(줄·부채꼴·던지는 줄).
/// 색은 옅은 흰 금빛 하나 — 빨강은 적의 예고만 쓴다(0141).
pub(super) fn aim_preview(w: &World, pp: Vec3, t: &Tuning, show: Option<AimShow>) {
    let (Some(show), Some(dir)) = (show, w.aim_line) else {
        return;
    };
    let fl = Drape {
        ground: w.ground.as_ref(),
    };
    // 밤의 바닥 위에서도 띠로 읽히게(처음 0.16·0.55는 헤드리스 화면에서 거의 안 보였다) — 0222(사장 "지금보다 연하게"): 0.32·0.95에서
    // 내렸다. 몸은 은은하게, 테로 길을 읽는다.
    let fill = Color::new(1.0, 0.96, 0.84, 0.18);
    let edge = Color::new(1.0, 0.95, 0.8, 0.6);
    let foot = pp + Vec3::Y * 0.05;
    let yaw = dir.x.atan2(dir.z);
    let lane = |len: f32, wid: f32| {
        fl.lane(foot, dir, len, wid, fill);
        fl.lane_edge(foot, dir, len, wid, edge);
    };
    let fan = |range: f32, arc: f32| {
        fl.sector(foot, range, yaw, arc, fill);
        fl.ring(foot, range, yaw, arc, edge);
    };
    match show {
        // 진실한 표시(0218): 길이는 탄을 쏠 때와 같은 길·같은 벽·땅 규칙으로 돌려 본 멈추는 자리(`World::shot_reach`), 폭은 탄이
        // 맞히는 폭. 모으는 중이면 모은 탄의 길.
        // 0274 ②b: 지금 떼면 나갈 것(`World::aim_bolt` — 활은 당긴 단계까지). 신성한 빛은 줄이 아니라 내려 꽂힐 자리의 고리
        // (`World::smite_spot` — 쏠 때와 같은 셈)와 거기까지의 가는 줄.
        AimShow::Cast => {
            let def = w.aim_bolt(t);
            if def.smites() {
                let c = w.smite_spot(dir, def, t) + Vec3::Y * 0.06;
                let r = crate::tuning::SMITE_RADIUS * def.scale;
                fl.line(foot, c, edge);
                fl.sector(c, r, 0.0, 360.0, fill);
                fl.ring(c, r, 0.0, 360.0, edge);
            } else {
                lane(
                    w.shot_reach(dir, def, t),
                    2.0 * crate::tuning::BOLT_RADIUS * def.scale,
                );
            }
        }
        // 터지는 자리(0218): 떨굴 자리가 아니라 같은 포물선·튕김을 돌려 본 도화선 끝의 자리(`World::grenade_lands`).
        AimShow::Grenade => {
            if let Some(g) = w.player.weapon.def().grenade {
                if let Some(c) = w.grenade_lands(pp + dir * g.drop) {
                    let c = c + Vec3::Y * 0.06;
                    fl.sector(c, g.radius, 0.0, 360.0, fill);
                    fl.ring(c, g.radius, 0.0, 360.0, edge);
                    fl.line(foot, c, edge);
                }
            }
        }
        AimShow::Skill(i) => {
            let Some(id) = w.player.skill.worn.get(i).copied() else {
                return;
            };
            use crate::skills::Form;
            match w.skill_eff(id).skill.form {
                Form::Lane { dist, half, .. } => lane(dist, half * 2.0),
                Form::Throw { dist, half, .. } => lane(dist, half * 2.0),
                Form::Hop { range, .. } => lane(range, 0.6),
                Form::Erupt { len, radius, .. } => lane(len, radius * 2.0),
                Form::Pull { range, arc_deg, .. }
                | Form::Cone { range, arc_deg, .. }
                | Form::Bash { range, arc_deg, .. }
                | Form::Stab { range, arc_deg, .. }
                | Form::Slip { range, arc_deg, .. } => fan(range, arc_deg),
                Form::Bell { range, .. } | Form::Hex { range, .. } => lane(range, 0.6),
                // 놓는 것(0270 C3): 놓일 자리까지의 가는 줄 + 그 자리의 고리.
                Form::Drift { ahead, radius, .. } | Form::Trap { ahead, radius, .. } => {
                    lane(ahead, 0.3);
                    let c = w.player.pos + dir * ahead;
                    let c = vec3(c.x, w.ground_y(c) + 0.06, c.z);
                    fl.sector(c, radius, 0.0, 360.0, fill);
                    fl.ring(c, radius, 0.0, 360.0, edge);
                }
                Form::Ring { .. }
                | Form::Brace { .. }
                | Form::Echo { .. }
                | Form::Field { .. }
                | Form::Wave { .. }
                | Form::Rally { .. }
                | Form::Reap { .. } => {}
            }
        }
    }
}
