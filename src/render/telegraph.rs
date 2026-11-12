//! 바닥 예고(모양마다 옅은 면 + 테두리, nail 0186~0188)와 적 발밑의 것들(그림자·퍼지는 고리·뿌리 가시·엘리트 표지).
use super::*;

/// 예고 색(붉은 계열 하나 — 붉은색은 위험에만, 그림 팔레트에는 없다).
pub(super) const TELE: Color = Color::new(0.85, 0.15, 0.1, 1.0);

/// 예고 잉크(채움 = 테두리) 투명도. 0.42 채움 + 0.7 테두리(겹쳐 더 진함)에서 내렸다(2026-11-07, 사장 "더 옅게").
/// 더 내리면 밝은 돌바닥에서 다 찬 순간이 흐려진다 — 그때는 투명도가 아니라 테두리 굵기·잉크 색으로 보강(KB color-vision-check).
const TELE_A: f32 = 0.3;
/// 설정 "공격 예고"(0074)의 이 값에서 위 투명도 그대로(70%). 기본 50%면 0.3 × 5/7 ≈ 0.21.
pub(super) const TELLS_BASE: f32 = 0.7;

/// 예고 면 전체에 깔린 바탕(어디까지인지만).
const TELE_BASE_A: f32 = 0.04;
/// 못 받는 공격의 가운데 X(1px 선이라 면보다 조금 진하게).
const TELE_MARK_A: f32 = 0.5;
/// 테두리 띠 폭(m) — `Drape::ring`과 같다. 채움은 이 안쪽까지만.
pub(super) const TELE_RING_W: f32 = 0.12;

/// 뿌리 기둥(0133): 높이(m)·가라앉는 초·빛깔(보라 뿌리 — 몸빛이 아니라 뿌리라 각지게, 0037).
pub(super) const ROOT_PILLAR_H: f32 = 1.5;

pub(super) const ROOTS_SINK: f32 = 0.6;

pub(super) const ROOT_PILLAR: Color = Color::new(0.24, 0.16, 0.30, 1.0);

pub(super) const ROOT_PILLAR_TIP: Color = Color::new(0.42, 0.30, 0.52, 1.0);

/// 엘리트 발밑 표지(0112): 옅은 뼈빛 — 예고의 붉은빛·플레이어 표지(금·민트·호박)와 겹치지 않는다.
pub(super) const ELITE_MARK: Color = Color::new(0.92, 0.88, 0.78, 0.5);

/// 바닥 예고 하나: 옅은 면 + 테두리, 채움이 자라 테두리에 닿는 순간 공격. 원판·부채꼴·고리는 가운데부터, 띠는 가운데 줄에서 양옆으로.
/// 2026-11-06: 바닥 높낮이를 따라 접어 그린다(`Drape`) — 낮은 곳의 적이 높은 곳으로 치면 예고가 높은 땅 속에 묻혀 안 보였다.
pub(super) fn telegraph(
    d: &Drape,
    p: Vec3,
    facing: f32,
    k: f32,
    shape: TeleShape,
    ink_mul: f32,
    timing: bool,
) {
    // 줄지은 원(0133 뿌리 기둥): 원 하나하나를 같은 규칙으로(원 = 피할 것).
    if let TeleShape::Row { n, r, step, start } = shape {
        let (sn, cs) = facing.sin_cos();
        let dir = vec3(sn, 0.0, cs);
        for i in 0..n {
            let c = p + dir * (start + i as f32 * step);
            telegraph(d, c, facing, k, TeleShape::Disc { r }, ink_mul, timing);
        }
        return;
    }
    if !timing {
        return telegraph_density(d, p, facing, k, shape, ink_mul);
    }
    // 2026-11-07(사장 "더 옅게, 싸움에 방해되지 않게"): nail 0186·0196 꼴 — 채움과 테두리는 같은 색·같은 투명도(`TELE_A`),
    // 채움은 테두리 띠 안쪽까지만 자라 겹치지 않는다(겹치면 그 자리만 진해져 "다 찬 순간"이 안 온다), 바탕은 아주 옅게.
    let bg = with_a(TELE, TELE_BASE_A * ink_mul);
    let ink = with_a(TELE, (TELE_A * ink_mul).min(1.0));
    let inner = |r: f32| (r - TELE_RING_W).max(0.0);
    match shape {
        TeleShape::Disc { r } => {
            d.sector(p, inner(r), facing, 360.0, bg);
            d.sector(p, inner(r) * k, facing, 360.0, ink);
            d.ring(p, r, facing, 360.0, ink);
        }
        TeleShape::Sector { r, arc } => {
            d.sector(p, inner(r), facing, arc, bg);
            d.sector(p, inner(r) * k, facing, arc, ink);
            d.ring(p, r, facing, arc, ink);
        }
        TeleShape::Lane { len, w } => {
            let (s, c) = facing.sin_cos();
            let dir = vec3(s, 0.0, c);
            d.lane(p, dir, len, w, bg);
            d.lane(p, dir, len, w * k, ink);
            d.lane_edge(p, dir, len, w, ink);
        }
        TeleShape::Ring { r } => {
            d.sector(p, inner(r), facing, 360.0, bg);
            d.ring(p, inner(r) * k, facing, 360.0, ink);
            d.ring(p, r, facing, 360.0, ink);
            // 패링으로 못 받는다 = 가운데 X(색이 아니라 모양으로, nail 0187 · KB timing-parry).
            let mark = with_a(TELE, (TELE_MARK_A * ink_mul).min(1.0));
            let x = r.min(2.0) * 0.45;
            d.line(p + vec3(-x, 0.0, -x), p + vec3(x, 0.0, x), mark);
            d.line(p + vec3(-x, 0.0, x), p + vec3(x, 0.0, -x), mark);
        }
        TeleShape::Row { .. } => {}
    }
}

/// 타이밍을 끈 바닥 예고(0128, 사장 "공격 지점에 색의 농도가 짙어지는 식, 정해진 농도가 되었을 때 히트"): 자라는 앞이 없다 —
/// 자리 전체가 옅게(`TELE_DENSITY_START`)에서 테두리와 같은 농도로 짙어지고, 그 농도에 닿는 틱에 맞는다. 밝기 차이는 비율로만 보여
/// 닿는 순간을 정확히 잴 수 없다(KB melee-light… "진하기만 오르는 채움은 끝 느낌이 없다") — 그래서 "언제"는 몸이 맡는다.
fn telegraph_density(d: &Drape, p: Vec3, facing: f32, k: f32, shape: TeleShape, ink_mul: f32) {
    let a = TELE_A * ink_mul * (TELE_DENSITY_START + (1.0 - TELE_DENSITY_START) * k);
    let ink = with_a(TELE, a.min(1.0));
    let edge = with_a(TELE, (TELE_A * ink_mul).min(1.0));
    let inner = |r: f32| (r - TELE_RING_W).max(0.0);
    match shape {
        TeleShape::Disc { r } => {
            d.sector(p, inner(r), facing, 360.0, ink);
            d.ring(p, r, facing, 360.0, edge);
        }
        TeleShape::Sector { r, arc } => {
            d.sector(p, inner(r), facing, arc, ink);
            d.ring(p, r, facing, arc, edge);
        }
        TeleShape::Lane { len, w } => {
            let (s, c) = facing.sin_cos();
            let dir = vec3(s, 0.0, c);
            d.lane(p, dir, len, w, ink);
            d.lane_edge(p, dir, len, w, edge);
        }
        TeleShape::Ring { r } => {
            // 둘레 고리(울림·뿌리 파동)는 반지름 9m라 다 짙어지면 화면을 덮는다 — 채움은 반만 짙게, 테두리는 그대로.
            d.sector(p, inner(r), facing, 360.0, with_a(TELE, ink.a * 0.5));
            d.ring(p, r, facing, 360.0, edge);
            let mark = with_a(TELE, (TELE_MARK_A * ink_mul).min(1.0));
            let x = r.min(2.0) * 0.45;
            d.line(p + vec3(-x, 0.0, -x), p + vec3(x, 0.0, x), mark);
            d.line(p + vec3(-x, 0.0, x), p + vec3(x, 0.0, -x), mark);
        }
        TeleShape::Row { .. } => {}
    }
}

/// 타이밍을 끈 예고가 처음 보이는 농도(다 찬 농도의 몫).
const TELE_DENSITY_START: f32 = 0.15;

pub(super) fn ground_marks(v: &View, w: &World, alpha: f32) {
    // 바닥 예고·그림자(배우보다 먼저).
    for en in &w.enemies {
        if !en.alive() {
            continue;
        }
        let p = en.prev.lerp(en.pos, alpha);
        let drape = Drape {
            ground: w.ground.as_ref(),
        };
        drape.shadow(p, en.radius * 1.2);
        if let Some((k, shape)) = en.telegraph() {
            if v.tells > 0.0 {
                // 심은 원판(꼬투리, 0120)은 몸이 아니라 심은 자리에.
                let at = en.tele_at().unwrap_or(p);
                telegraph(
                    &drape,
                    at,
                    en.facing,
                    k,
                    shape,
                    v.tells / TELLS_BASE,
                    v.tell_timing,
                );
            }
        }
        wave_rings(en, p, &drape);
        root_spikes(en, p, &drape);
        elite_mark(en, p, w, &drape);
    }
}

fn wave_rings(en: &Enemy, p: Vec3, drape: &Drape) {
    // 퍼지는 고리(뿌리 파동 하나·울림 고리 여럿, 0125) — 고리 사이가 설 자리로 읽히게 하나하나 따로.
    // 종 내던지기(0133)의 고리는 종이 떨어진 자리가 가운데.
    let wc = if en.atk == crate::enemy::Atk::Hurl {
        en.wave_center()
    } else {
        p
    };
    for front in en.wave_fronts() {
        drape.ring(wc, front, 0.0, 360.0, with_a(TELE, 0.45));
        drape.ring(wc, (front - 0.5).max(0.1), 0.0, 360.0, with_a(TELE, 0.18));
    }
}

fn root_spikes(en: &Enemy, p: Vec3, drape: &Drape) {
    // 뿌리 기둥 줄(0133): 앞머리까지 기둥이 솟고(지나간 자리일수록 다 솟음), 쉬는 동안 가라앉는다.
    let sink_k = match (en.state, en.atk) {
        (EState::Recover { t }, crate::enemy::Atk::Roots) => Some(1.0 - t / ROOTS_SINK),
        _ => None,
    };
    if let Some(front) = en
        .roots_front()
        .or(sink_k.filter(|k| *k > 0.0).map(|_| en.charge_len))
    {
        let f = en.facing_dir();
        let mut s = en.radius + 0.5;
        let mut i = 0;
        while s <= front {
            let rise = ((front - s) / 1.6).clamp(0.0, 1.0) * sink_k.unwrap_or(1.0).max(0.0);
            let h = ROOT_PILLAR_H * rise * if i % 2 == 0 { 1.0 } else { 0.75 };
            if h > 0.02 {
                let side = if i % 2 == 0 { 0.25 } else { -0.25 };
                let at = drape.at(
                    p.x + f.x * s + f.z * side,
                    p.z + f.z * s - f.x * side,
                    p.y,
                    0.0,
                );
                // 뿌리 가시(0142 — 네모 기둥 대신): 굵은 밑동에서 가늘어지다 뾰족한 끝, 하나씩 엇갈려 기운다.
                // 각진 것은 뿌리(0037)라 면은 각진 채로. 판정(띠)은 그대로다 — 그림만.
                let lean = vec3(f.z, 0.0, -f.x) * (side * 0.5) + f * 0.08;
                let (body, tip) = spike_faces(0.21, 0.09, h, 0.42 * rise, lean);
                let turn = Quat::from_rotation_y(i as f32 * 0.9);
                shaded_tris(at, &body, ROOT_PILLAR, 0.8, turn);
                shaded_tris(at, &tip, ROOT_PILLAR_TIP, 0.9, turn);
            }
            s += 0.8;
            i += 1;
        }
    }
}

fn elite_mark(en: &Enemy, p: Vec3, w: &World, drape: &Drape) {
    // 엘리트(0112): 발밑의 옅은 뼈빛 표지 — 성질마다 모양이 다르다(색으로 세기를 말하지 않는다, lore ecology 2).
    // 굳은 = 굵은 고리 하나, 되풀이 = 고리 둘, 남는 = 끊어진 고리(쓰러진 뒤 남을 것).
    if let Some(affix) = en.elite {
        let r = en.radius + 0.35;
        match affix {
            crate::enemy::Affix::Hardened => {
                drape.ring(p, r, 0.0, 360.0, ELITE_MARK);
                drape.ring(p, r - TELE_RING_W, 0.0, 360.0, ELITE_MARK);
            }
            crate::enemy::Affix::Repeating => {
                drape.ring(p, r, 0.0, 360.0, ELITE_MARK);
                drape.ring(p, r * 0.72, 0.0, 360.0, ELITE_MARK);
            }
            crate::enemy::Affix::Lingering => {
                let spin = w.time * 0.6;
                for i in 0..6 {
                    let yaw = spin + i as f32 * std::f32::consts::TAU / 6.0;
                    drape.ring(p, r, yaw, 32.0, ELITE_MARK);
                }
            }
        }
    }
}

pub(super) fn linger_tells(v: &View, w: &World) {
    // 남는 엘리트의 고리(0112): 뿌리 파동과 같은 "패링 못 함" 고리 예고(가운데 X), 다 차는 순간 터진다.
    for l in &w.lingers {
        let drape = Drape {
            ground: w.ground.as_ref(),
        };
        let k = (l.t / crate::tuning::ELITE_LINGER_DELAY).clamp(0.0, 1.0);
        telegraph(
            &drape,
            l.pos,
            0.0,
            k,
            TeleShape::Ring {
                r: crate::tuning::ELITE_LINGER_RADIUS,
            },
            (v.tells / TELLS_BASE).max(0.6),
            v.tell_timing,
        );
    }
}
