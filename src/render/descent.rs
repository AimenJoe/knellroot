//! 지역 사이 전환의 굴 장면(0174): 뿌리가 얽힌 수직 굴 — 벽을 따라 도는 큰 뿌리를 방랑자가 밟고 내려간다(올라간다).
//! 규칙(시간표·길·카메라)은 `descent.rs`. 여기는 그림: 굴 벽(깊이마다의 흙빛 띠)·뿌리들(각진 네모 단면 — 각진 것은 뿌리만, 0037)·
//! 방랑자(걷기 동작)·등잔 빛·위의 달빛과 아래의 빛·떠다니는 먼지. 월드와 같은 타깃·같은 선·같은 마무리 패스로 그린다.
//! 굴의 면은 전환마다 한 번 짓고(깊이가 흙빛을 정한다), 빛은 프레임마다 다시 받는다 — 등잔이 방랑자와 같이 움직인다.
use super::*;
use crate::descent::{self as dsc, Passage, DROP, ROOT_R, SHAFT_R};

const TAU_F: f32 = std::f32::consts::TAU;

/// 면 하나(굴 안의 자리). `glow` = 스스로 빛남(빛을 받지 않는다).
struct Face {
    p: [Vec3; 3],
    n: Vec3,
    col: Color,
    glow: bool,
}

/// 지어 둔 굴(전환 하나의 것).
pub(super) struct ShaftMesh {
    key: (u32, u32),
    faces: Vec<Face>,
}

/// 굴 벽이 이어지는 높이(위·아래 — 카메라가 보는 끝보다 넉넉히).
const WALL_TOP: f32 = 7.0;
const WALL_BOTTOM: f32 = -DROP - 8.0;
const SIDES: usize = 26;
const RING_H: f32 = 0.78;
/// 뿌리의 빛(식생의 뿌리와 같은 보라 — `flora::ROOT`), 스스로 빛나는 옹이(호박).
const ROOT: Color = Color::new(0.27, 0.15, 0.34, 1.0);
const KNOT: Color = Color::new(1.0, 0.72, 0.36, 1.0);
/// 디테일(0175 — 사장 "이제 디테일이 들어가면 될 듯"): 이끼(큰 뿌리 윗면·돌 위), 썩은 뿌리에 핀 희미한 버섯(스스로 빛남 —
/// 푸른 빛, 뿌리의 호박빛 옹이와 갈린다), 벽을 기는 검은 균사 줄(뽕나무버섯의 "신발끈" 균사다발 — KB 동물·생태 사실).
const MOSS: Color = Color::new(0.20, 0.33, 0.27, 1.0);
const CAP: Color = Color::new(0.55, 0.95, 0.82, 1.0);
const STALK: Color = Color::new(0.62, 0.58, 0.52, 1.0);
const CORD: Color = Color::new(0.07, 0.05, 0.08, 1.0);
/// 등잔 빛·달빛(굴 위의 틈)의 색.
const LANTERN: Color = Color::new(1.0, 0.70, 0.36, 1.0);
const MOONBEAM: Color = Color::new(0.52, 0.58, 0.92, 1.0);
/// 굴 속 바탕 어둠(달빛이 닿지 않는다 — `lit`의 바탕빛에 곱한다).
const CAVE_DIM: f32 = 0.8;

fn h(a: u32, b: u32) -> f32 {
    let mut v = a.wrapping_mul(0x9E37_79B9) ^ b.wrapping_mul(0x85EB_CA6B);
    v ^= v >> 15;
    v = v.wrapping_mul(0x2C1B_3C6D);
    v ^= v >> 12;
    (v & 0xFFFF) as f32 / 65535.0
}

fn mul(c: Color, k: f32) -> Color {
    Color::new(c.r * k, c.g * k, c.b * k, c.a)
}

/// 면 하나를 넣는다. 법선은 `toward`를 등지게(벽 = 축을 보게 하려면 축 밖의 점을, 뿌리 = 가운데 줄의 점을 준다).
fn face(out: &mut Vec<Face>, p: [Vec3; 3], away_from: Vec3, col: Color, glow: bool) {
    let mut n = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
    let mid = (p[0] + p[1] + p[2]) / 3.0;
    if n.dot(mid - away_from) < 0.0 {
        n = -n;
    }
    out.push(Face { p, n, col, glow });
}

/// 네모 단면 줄기(뿌리): 점을 따라 굵기 `r0` → `r1`. 윗면이 평평하다(큰 뿌리는 그 위를 걷는다).
fn strand(out: &mut Vec<Face>, pts: &[Vec3], r0: f32, r1: f32, col: Color) {
    let n = pts.len();
    if n < 2 {
        return;
    }
    let ring = |i: usize| -> [Vec3; 4] {
        let dir = if i + 1 < n {
            pts[i + 1] - pts[i]
        } else {
            pts[i] - pts[i - 1]
        }
        .normalize_or_zero();
        let side = if dir.y.abs() > 0.9 { Vec3::X } else { Vec3::Y }
            .cross(dir)
            .normalize_or_zero();
        let up = dir.cross(side);
        let r = r0 + (r1 - r0) * i as f32 / (n - 1) as f32;
        [
            pts[i] + (side + up) * r,
            pts[i] + (-side + up) * r,
            pts[i] + (-side - up) * r,
            pts[i] + (side - up) * r,
        ]
    };
    for i in 0..n - 1 {
        let (a, b) = (ring(i), ring(i + 1));
        let mid = (pts[i] + pts[i + 1]) * 0.5;
        // 면마다 조금 다른 밝기(나무껍질의 결 — 한 빛이면 매끈한 관으로 읽힌다).
        for s in 0..4 {
            let t = (s + 1) % 4;
            let k = 0.9 + 0.2 * h(i as u32 * 7 + s as u32, (pts[0].x * 31.0) as u32);
            face(out, [a[s], a[t], b[t]], mid, mul(col, k), false);
            face(out, [a[s], b[t], b[s]], mid, mul(col, k), false);
        }
    }
}

/// 팔면체 하나(옹이·박힌 돌).
fn knob(out: &mut Vec<Face>, c: Vec3, r: Vec3, col: Color, glow: bool) {
    let x = [vec3(r.x, 0.0, 0.0), vec3(-r.x, 0.0, 0.0)];
    let y = [vec3(0.0, r.y, 0.0), vec3(0.0, -r.y, 0.0)];
    let z = [vec3(0.0, 0.0, r.z), vec3(0.0, 0.0, -r.z)];
    for a in x {
        for b in y {
            for d in z {
                face(out, [c + a, c + b, c + d], c, col, glow);
            }
        }
    }
}

/// 굴 벽 위의 한 점(각 `a`, 높이 `y`) — 벽에서 `inset`만큼 안쪽.
fn on_wall(a: f32, y: f32, inset: f32) -> Vec3 {
    let r = SHAFT_R - inset;
    vec3(r * a.sin(), y, r * a.cos())
}

/// 굴 하나를 짓는다: 벽(띠마다 한 빛 — 깊이의 흙빛) · 큰 뿌리 · 받치는 뿌리 · 벽을 타는 뿌리 · 늘어진 잔뿌리 · 박힌 돌 · 빛나는 옹이.
fn build(from_m: u32, to_m: u32) -> ShaftMesh {
    let mut f: Vec<Face> = Vec::new();
    // 위 끝 = 얕은 쪽, 아래 끝 = 깊은 쪽(올라가는 전환도 같은 굴을 거꾸로 걷는다).
    let (top_m, bottom_m) = (from_m.min(to_m) as f32, from_m.max(to_m) as f32);
    let depth_at = |y: f32| top_m + (bottom_m - top_m) * (-y / DROP).clamp(-0.6, 1.6);
    // 벽: 고리마다 반지름·높이가 조금씩 다른 다각 기둥의 안쪽.
    let rings = ((WALL_TOP - WALL_BOTTOM) / RING_H).ceil() as usize;
    let at = |i: usize, j: usize| -> Vec3 {
        let j = j % SIDES;
        let a = std::f32::consts::TAU * (j as f32 + 0.5 * (i % 2) as f32) / SIDES as f32;
        let r = SHAFT_R + (h(i as u32, j as u32) - 0.5) * 0.55;
        let y = WALL_TOP - i as f32 * RING_H + (h(j as u32 + 40, i as u32) - 0.5) * 0.35;
        vec3(r * a.sin(), y, r * a.cos())
    };
    for i in 0..rings {
        let y = WALL_TOP - (i as f32 + 0.5) * RING_H;
        // 띠마다 한 빛(번갈아 밝고 어둡게 — KB `conveying-descent`: 띠 안에서 기둥마다 바꾸면 세로 줄무늬로 읽힌다).
        let shade = if i % 2 == 0 { 1.0 } else { 0.84 } + h(i as u32 * 7, 3) * 0.06;
        let col = mul(dsc::soil(depth_at(y)), shade);
        for j in 0..SIDES {
            let (a, b, c, d) = (at(i, j), at(i, j + 1), at(i + 1, j), at(i + 1, j + 1));
            // 축 밖의 먼 점을 등진다 = 축을 본다.
            let outside = vec3(a.x, a.y, a.z) * 3.0;
            face(&mut f, [a, b, c], outside, col, false);
            face(&mut f, [b, d, c], outside, col, false);
        }
    }
    // 큰 뿌리: 걷는 구간 밖으로도 이어진다(화면 밖에서 오고 간다), 두 끝은 가늘어진다.
    let seg = 0.05;
    let pts = |s0: f32, s1: f32| -> Vec<Vec3> {
        let n = ((s1 - s0) / seg).round() as usize;
        (0..=n).map(|i| dsc::path(s0 + seg * i as f32)).collect()
    };
    strand(&mut f, &pts(-0.1, 1.1), ROOT_R, ROOT_R, ROOT);
    strand(&mut f, &pts(-0.4, -0.1), ROOT_R * 0.45, ROOT_R, ROOT);
    strand(&mut f, &pts(1.1, 1.4), ROOT_R, ROOT_R * 0.45, ROOT);
    // 받치는 뿌리: 큰 뿌리에서 벽으로(비스듬히 아래) — 큰 뿌리가 허공에 뜬 관으로 보이지 않게.
    for k in 0..9u32 {
        let s = -0.3 + 0.2 * k as f32 + (h(k, 1) - 0.5) * 0.06;
        let from = dsc::path(s) - Vec3::Y * ROOT_R * 0.5;
        let a = dsc::angle(s) + (h(k, 2) - 0.5) * 0.5;
        let to = on_wall(a, from.y - 1.2 - 1.4 * h(k, 3), 0.1);
        let mid = (from + to) * 0.5 - Vec3::Y * 0.25;
        let col = mul(ROOT, 0.8 + 0.2 * h(k, 4));
        strand(&mut f, &[from, mid, to], 0.2, 0.34, col);
    }
    // 늘어진 잔뿌리: 큰 뿌리 밑에서 아래로.
    for k in 0..14u32 {
        let s = -0.2 + 1.4 * h(k, 11);
        let from =
            dsc::path(s) - Vec3::Y * ROOT_R * 0.9 + vec3(h(k, 12) - 0.5, 0.0, h(k, 13) - 0.5) * 0.5;
        let len = 0.7 + 1.5 * h(k, 14);
        let sway = vec3(h(k, 15) - 0.5, 0.0, h(k, 16) - 0.5) * 0.4;
        let tip = from - Vec3::Y * len + sway;
        let col = mul(ROOT, 0.75 + 0.3 * h(k, 17));
        strand(
            &mut f,
            &[from, (from + tip) * 0.5 + sway * 0.4, tip],
            0.09,
            0.03,
            col,
        );
    }
    // 벽을 타는 뿌리: 위에서 아래로 구불구불.
    for k in 0..16u32 {
        let a0 = std::f32::consts::TAU * (k as f32 + h(k, 21)) / 16.0;
        let y0 = WALL_TOP - 1.0 - (WALL_TOP - WALL_BOTTOM - 6.0) * h(k, 22);
        let len = 3.5 + 4.5 * h(k, 23);
        let r = 0.14 + 0.2 * h(k, 24);
        let line: Vec<Vec3> = (0..=5)
            .map(|i| {
                let t = i as f32 / 5.0;
                let wig = (h(k, 30 + i) - 0.5) * 0.5;
                on_wall(a0 + wig, y0 - len * t, 0.35 + r)
            })
            .collect();
        strand(
            &mut f,
            &line,
            r * 1.2,
            r * 0.5,
            mul(ROOT, 0.7 + 0.35 * h(k, 25)),
        );
    }
    // 가로지르는 뿌리: 벽에서 벽으로(가까운 두 점 — 굴 한가운데는 비워 둔다, 카메라와 방랑자 사이를 막지 않게).
    for k in 0..7u32 {
        let a0 = std::f32::consts::TAU * h(k, 41);
        let span = 0.7 + 0.5 * h(k, 42);
        let y = WALL_TOP - 2.0 - (WALL_TOP - WALL_BOTTOM - 4.0) * h(k, 43);
        let (a, b) = (
            on_wall(a0, y, 0.2),
            on_wall(a0 + span, y - 0.8 + 1.6 * h(k, 44), 0.2),
        );
        let mid = (a + b) * 0.5 * 0.9 - Vec3::Y * 0.3;
        strand(&mut f, &[a, mid, b], 0.2, 0.16, mul(ROOT, 0.8));
    }
    // 박힌 돌(큰 덩어리 — 작은 점만 흩으면 가루로 보인다)과 빛나는 옹이.
    for k in 0..18u32 {
        let a = std::f32::consts::TAU * h(k, 51);
        let y = WALL_TOP - (WALL_TOP - WALL_BOTTOM) * h(k, 52);
        let r = 0.35 + 0.45 * h(k, 53);
        let stone = mul(dsc::soil(depth_at(y) + 22.0), 1.25);
        knob(
            &mut f,
            on_wall(a, y, 0.1),
            vec3(r, r * 0.7, r),
            stone,
            false,
        );
    }
    for k in 0..7u32 {
        let s = -0.15 + 1.3 * (k as f32 + h(k, 61)) / 7.0;
        let side = if k % 2 == 0 { 1.0 } else { -1.0 };
        let a = dsc::angle(s);
        // 큰 뿌리의 옆구리(굴 축 쪽·벽 쪽 번갈아).
        let c = dsc::path(s) + vec3(a.sin(), 0.0, a.cos()) * (ROOT_R + 0.02) * side - Vec3::Y * 0.1;
        knob(&mut f, c, Vec3::splat(0.05 + 0.03 * h(k, 62)), KNOT, true);
    }
    details(&mut f, &depth_at);
    ShaftMesh {
        key: (from_m, to_m),
        faces: f,
    }
}

/// 굴의 디테일(0175): 벽의 턱(지층의 이음마다 튀어나온 얇은 선반) · 큰 뿌리의 곁가지와 윗면의 이끼 · 버섯 무리 · 균사 줄 · 작은 돌.
fn details(f: &mut Vec<Face>, depth_at: &dyn Fn(f32) -> f32) {
    use std::f32::consts::TAU;
    // 지층의 이음: 몇 띠마다 벽에서 튀어나온 얇은 턱(한 바퀴의 몇 조각) — 띠의 빛이 바뀌는 자리를 꼴로도.
    let mut y = WALL_TOP - 1.4;
    let mut band = 0u32;
    while y > WALL_BOTTOM + 1.0 {
        let pieces = 3 + (h(band, 81) * 3.0) as u32;
        for q in 0..pieces {
            let a0 = TAU * (q as f32 + h(band * 7 + q, 82) * 0.5) / pieces as f32;
            let span = 0.5 + 0.6 * h(band + q, 83);
            let col = mul(dsc::soil(depth_at(y) + 12.0), 1.15);
            const N: usize = 6;
            for i in 0..N {
                let (t0, t1) = (i as f32 / N as f32, (i + 1) as f32 / N as f32);
                let (b0, b1) = (a0 + span * t0, a0 + span * t1);
                let (o0, o1) = (on_wall(b0, y, 0.0), on_wall(b1, y, 0.0));
                let jut = 0.28 + 0.12 * h(band + q, 84 + i as u32);
                let (i0, i1) = (on_wall(b0, y, jut), on_wall(b1, y, jut));
                let lip = Vec3::Y * -0.14;
                // 윗면(위를 본다)과 앞면(축을 본다).
                face(f, [o0, o1, i1], vec3(0.0, y - 5.0, 0.0), col, false);
                face(f, [o0, i1, i0], vec3(0.0, y - 5.0, 0.0), col, false);
                face(
                    f,
                    [i0, i1, i1 + lip],
                    vec3(i0.x, y, i0.z) * 2.0,
                    mul(col, 0.8),
                    false,
                );
                face(
                    f,
                    [i0, i1 + lip, i0 + lip],
                    vec3(i0.x, y, i0.z) * 2.0,
                    mul(col, 0.8),
                    false,
                );
            }
        }
        y -= RING_H * (3.0 + 2.0 * h(band, 85));
        band += 1;
    }
    // 큰 뿌리의 곁가지: 몸에서 비스듬히 위·옆으로 돋아 끝이 가늘다(몇은 굽어 오른다).
    for k in 0..12u32 {
        let s = -0.25 + 1.5 * (k as f32 + h(k, 91)) / 12.0;
        let base = dsc::path(s);
        let a = dsc::angle(s);
        let out = vec3(a.sin(), 0.0, a.cos()) * if k % 2 == 0 { 1.0 } else { -1.0 };
        let up = 0.3 + 0.6 * h(k, 92);
        let len = 0.7 + 0.8 * h(k, 93);
        let p0 = base + out * ROOT_R * 0.7 + Vec3::Y * ROOT_R * 0.2;
        let p1 = p0 + (out + Vec3::Y * up).normalize() * len * 0.55;
        let p2 = p1 + (out * 0.6 + Vec3::Y * (up + 0.4)).normalize() * len * 0.45;
        strand(
            f,
            &[p0, p1, p2],
            0.13,
            0.03,
            mul(ROOT, 0.9 + 0.15 * h(k, 94)),
        );
    }
    // 이끼: 큰 뿌리 윗면의 둥근 조각(여섯모 — 낱장 세모는 부스러기로 읽혔다), 걷는 줄 곁 — 발밑은 비운다.
    for k in 0..16u32 {
        let s = -0.2 + 1.4 * h(k, 101);
        let a = dsc::angle(s);
        let side = vec3(a.sin(), 0.0, a.cos())
            * (0.28 + 0.12 * h(k, 102))
            * if k % 2 == 0 { 1.0 } else { -1.0 };
        let c = dsc::path(s) + Vec3::Y * (ROOT_R + 0.015) + side;
        let below = c - Vec3::Y;
        let col = mul(MOSS, 0.85 + 0.3 * h(k, 105));
        let r = 0.16 + 0.14 * h(k, 103);
        let ring: Vec<Vec3> = (0..6)
            .map(|i| {
                let t = TAU * i as f32 / 6.0 + h(k, 106);
                c + vec3(t.cos(), 0.0, t.sin()) * r * (0.7 + 0.5 * h(k * 7 + i, 107))
            })
            .collect();
        for i in 0..6 {
            face(f, [c, ring[i], ring[(i + 1) % 6]], below, col, false);
        }
    }
    // 버섯 무리: 벽과 큰 뿌리의 옆구리에 셋~다섯씩(대 + 납작한 갓, 갓이 스스로 빛난다) — 걷는 길 곁에 몇은 꼭(눈에 닿게).
    let cluster = |f: &mut Vec<Face>, k: u32, foot0: Vec3, into: Vec3| {
        let n = 3 + (h(k, 113) * 3.0) as u32;
        let along = Vec3::Y.cross(into).normalize_or_zero();
        for j in 0..n {
            let off = along * (h(k * 13 + j, 114) - 0.5) * 0.5
                + Vec3::Y * (h(k * 13 + j, 115) - 0.5) * 0.3;
            let foot = foot0 + off;
            let tall = 0.14 + 0.14 * h(k * 13 + j, 116);
            let top = foot + into * tall * 0.6 + Vec3::Y * tall;
            strand(f, &[foot, top], 0.03, 0.025, STALK);
            let r = 0.09 + 0.08 * h(k * 13 + j, 117);
            knob(f, top, vec3(r, r * 0.35, r), CAP, true);
        }
    };
    for k in 0..10u32 {
        let a = TAU * h(k, 111);
        let y = WALL_TOP - 2.0 - (WALL_TOP - WALL_BOTTOM - 4.0) * h(k, 112);
        let foot = on_wall(a, y, 0.12);
        cluster(f, k, foot, vec3(-foot.x, 0.0, -foot.z).normalize_or_zero());
    }
    for k in 0..6u32 {
        let s = -0.05 + 1.1 * (k as f32 + 0.5) / 6.0;
        let a = dsc::angle(s);
        let out = vec3(a.sin(), 0.0, a.cos()) * if k % 2 == 0 { 1.0 } else { -1.0 };
        let foot = dsc::path(s) + out * (ROOT_R + 0.02);
        cluster(f, 40 + k, foot, out);
    }
    // 균사 줄: 벽을 길게 기는 검고 가는 줄(갈래 하나씩).
    for k in 0..10u32 {
        let a0 = TAU * h(k, 121);
        let y0 = WALL_TOP - (WALL_TOP - WALL_BOTTOM - 3.0) * h(k, 122);
        let line: Vec<Vec3> = (0..=7)
            .map(|i| {
                let t = i as f32 / 7.0;
                let wig = (h(k, 130 + i) - 0.5) * 0.35;
                on_wall(a0 + t * 0.9 + wig, y0 - t * 3.2, 0.08)
            })
            .collect();
        strand(f, &line, 0.035, 0.02, CORD);
        let mid = line[3];
        let a1 = a0 + 0.35;
        strand(
            f,
            &[mid, on_wall(a1, mid.y - 1.0, 0.08)],
            0.025,
            0.015,
            CORD,
        );
    }
    // 작은 돌(벽에 박힌 자갈 — 큰 돌 사이의 크기).
    for k in 0..40u32 {
        let a = TAU * h(k, 141);
        let y = WALL_TOP - (WALL_TOP - WALL_BOTTOM) * h(k, 142);
        let r = 0.1 + 0.12 * h(k, 143);
        let col = mul(dsc::soil(depth_at(y) + 30.0), 1.1 + 0.3 * h(k, 144));
        knob(f, on_wall(a, y, 0.08), vec3(r, r * 0.8, r), col, false);
    }
}

impl Renderer {
    /// 굴 장면 한 프레임(0174): 월드 대신 그린다 — 같은 타깃·같은 선·같은 마무리 패스. `clock` = 실제 시간(걷기 동작·먼지).
    /// `upper`·`lower` = 굴의 위·아래 지역의 환경(달이 있나 · 안개 빛).
    pub fn draw_shaft(
        &mut self,
        p: &Passage,
        weapon: crate::weapons::Id,
        t: &Tuning,
        clock: f32,
        (upper, lower): (&crate::backdrop::Env, &crate::backdrop::Env),
    ) -> Frame {
        use crate::tuning::P;
        if self.shaft.as_ref().map(|m| m.key) != Some((p.from_m, p.to_m)) {
            self.shaft = Some(build(p.from_m, p.to_m));
        }
        // 그림자 가림판은 월드의 것 — 이 프레임에는 없다.
        self.light_vp = None;
        self.batch.casters.clear();
        self.batch.caster_idx.clear();
        let (iw, ih) = self.fit_screen();
        let (eye, target) = dsc::eye(p);
        let c3 = Camera3D {
            position: eye,
            target,
            up: Vec3::Y,
            fovy: dsc::EYE_FOV,
            aspect: Some(iw as f32 / ih as f32),
            projection: Projection::Perspective,
            render_target: Some(self.rt.clone()),
            viewport: None,
            z_near: 0.1,
            z_far: 200.0,
        };
        set_look(Look {
            toon: t[P::RenderToon] >= 0.5,
            rim: t[P::RenderRim],
            outline_px: t[P::RenderOutline],
            eye,
            fovy: c3.fovy,
            rows: ih as f32,
            calm: t[P::RenderFloorCalm] >= 0.5,
            key: t[P::RenderKeyLight] >= 0.5,
            enemy_lift: t[P::RenderEnemyLift],
            enemy_ash: t[P::RenderEnemyAsh],
        });
        set_camera(&c3);
        set_actor_shade(None);
        // 굴 속에는 달빛(한쪽에서 오는 빛)이 없다 — 위의 틈에서 떨어지는 빛줄기와 등잔뿐.
        set_moonlight(0.0);
        let s = p.travel();
        let dark = mul(lower.fog_deep, 0.35 + 0.25 * (1.0 - s));
        clear_background(Color::new(dark.r, dark.g, dark.b, 1.0));

        let (feet, yaw) = dsc::walker(p);
        let fwd = vec3(yaw.sin(), 0.0, yaw.cos());
        // 빛 셋: 등잔(방랑자 곁) · 위의 달빛(위 지역에 달이 있을 때, 위에 가까울수록) · 아래에서 올라오는 닿을 곳의 빛.
        let near_top = (1.0 - s * 1.6).clamp(0.0, 1.0);
        let near_bottom = ((s - 0.45) * 1.8).clamp(0.0, 1.0);
        // 등잔은 살짝 일렁인다(불꽃 — 두 사인의 합, 실제 시간).
        let flick = 1.0 + 0.06 * (clock * 9.3).sin() + 0.04 * (clock * 23.7 + 1.3).sin();
        let mut lights = vec![Light {
            pos: feet + Vec3::Y * 1.25 + fwd * 0.45,
            color: mul(LANTERN, flick),
            radius: 7.5 * (0.97 + 0.03 * flick),
            shaft: false,
        }];
        if upper.moon && near_top > 0.0 {
            lights.push(Light {
                pos: vec3(0.0, WALL_TOP, 0.0),
                color: mul(MOONBEAM, 0.75 * near_top),
                radius: SHAFT_R * 1.25,
                shaft: true,
            });
        }
        if near_bottom > 0.0 {
            lights.push(Light {
                pos: vec3(0.0, -DROP - 3.5, 0.0),
                color: mul(lower.fog, 1.6 * near_bottom),
                radius: 9.0,
                shaft: false,
            });
        }

        let edges_on = t[P::RenderEdge] >= 0.5;
        self.batch.record = edges_on;
        self.batch.recorded.clear();
        // 굴의 면: 빛을 다시 받아 덩어리로 보낸다(한 번에 정점 9,000 아래).
        if let Some(mesh) = &self.shaft {
            let mut verts: Vec<Vertex> = Vec::with_capacity(9000);
            let mut idx: Vec<u16> = Vec::with_capacity(9000);
            for fc in &mesh.faces {
                let mid = (fc.p[0] + fc.p[1] + fc.p[2]) / 3.0;
                let k = if fc.glow {
                    Vec3::splat(1.1)
                } else {
                    // 바탕빛은 굴 속이라 조금 낮게, 등잔·빛줄기의 몫은 그대로.
                    let base = lit(mid, fc.n, &[]) * CAVE_DIM;
                    base + (lit(mid, fc.n, &lights) - lit(mid, fc.n, &[]))
                };
                let col = Color::new(fc.col.r * k.x, fc.col.g * k.y, fc.col.b * k.z, 1.0);
                let b = verts.len() as u16;
                for q in fc.p {
                    verts.push(crate::scenery::with_normal(
                        Vertex::new(q.x, q.y, q.z, 0.0, 0.0, col),
                        fc.n,
                    ));
                }
                idx.extend_from_slice(&[b, b + 1, b + 2]);
                if verts.len() >= 8997 {
                    self.batch.raw(&verts, &idx, None, 1.0);
                    verts.clear();
                    idx.clear();
                }
            }
            if !verts.is_empty() {
                self.batch.raw(&verts, &idx, None, 1.0);
            }
        }
        // 방랑자: 걷기 동작, 든 무기만 보인다(0051).
        if let Some(e) = self.lib.get("wanderer") {
            let mut hide: Vec<(usize, Quat, Vec3)> = Vec::new();
            for w in crate::weapons::WEAPONS.iter() {
                if w.id != weapon {
                    if let Some(n) = e.model.node(w.node) {
                        hide.push((n, Quat::IDENTITY, Vec3::ZERO));
                    }
                }
            }
            e.model
                .pose_with(e.pick(&["walk", "idle"]), clock, &hide, &mut self.pose);
            self.batch.model(
                e,
                &self.pose,
                Mat4::from_translation(feet) * Mat4::from_rotation_y(yaw),
                WHITE,
                &lights,
                1.0,
            );
        }
        self.batch.flush();
        self.batch.record = false;

        // 빛줄기: 위의 틈에서 떨어지는 달빛(위에 가까울 때), 아래에서 올라오는 닿을 곳의 빛(아래에 가까울 때).
        if upper.moon && near_top > 0.0 {
            shaft(
                vec3(0.0, feet.y - 4.0, 0.0),
                vec3(0.0, WALL_TOP + 2.0, 0.0),
                0.6,
                2.6,
                with_a(MOONBEAM, 0.0),
                with_a(MOONBEAM, 0.22 * near_top),
            );
        }
        if near_bottom > 0.0 {
            beam(
                vec3(0.0, -DROP - 7.0, 0.0),
                SHAFT_R * 0.75,
                7.0,
                with_a(mul(lower.fog, 1.8), 0.2 * near_bottom),
            );
        }
        // 등잔의 빛무리와 떠다니는 먼지(내려가면 위로 흐르는 듯, 실제 시간으로).
        glow_disc(
            feet + Vec3::Y * 1.2 + fwd * 0.4,
            0.9 * flick,
            with_a(LANTERN, 0.16 * flick),
        );
        // 발걸음(0175): 걷기 동작(0.8초)의 두 디딤마다 발밑에서 흙 부스러기가 튀어 떨어진다 — 뿌리를 밟는 무게.
        let step = 0.4;
        let since = clock.rem_euclid(step);
        let n_step = (clock / step).floor() as u32;
        let side = vec3(fwd.z, 0.0, -fwd.x)
            * if n_step.is_multiple_of(2) {
                0.14
            } else {
                -0.14
            };
        for k in 0..5u32 {
            let life = 0.45;
            if since > life {
                break;
            }
            let u = since / life;
            let dir = vec3(
                h(n_step * 5 + k, 151) - 0.5,
                0.0,
                h(n_step * 5 + k, 152) - 0.5,
            ) * 0.9;
            let p = feet + side + dir * u + Vec3::Y * (0.12 * u * 4.0 * (1.0 - u) - 0.25 * u * u);
            gem(
                p,
                0.035 + 0.02 * h(k, 153),
                with_a(Color::new(0.62, 0.5, 0.42, 1.0), 0.8 * (1.0 - u)),
            );
        }
        // 위에서 떨어지는 흙 알갱이 몇(굴이 살아 있다 — 가끔, 곧게).
        for k in 0..6u32 {
            let period = 2.2 + 1.6 * h(k, 161);
            let ph = (clock / period + h(k, 162)).rem_euclid(1.0);
            let a = TAU_F * h(k, 163) + (clock / period).floor() * 1.7;
            let r = SHAFT_R * (0.55 + 0.35 * h(k, 164));
            let y = feet.y + 5.0 - ph * 9.0;
            gem(
                vec3(r * a.sin(), y, r * a.cos()),
                0.045,
                with_a(Color::new(0.72, 0.6, 0.5, 1.0), 0.7 * (1.0 - ph)),
            );
        }
        let drift = if p.up { -0.25 } else { 0.25 };
        for k in 0..36u32 {
            let a = std::f32::consts::TAU * h(k, 71) + clock * 0.05 * (h(k, 72) - 0.5);
            let r = SHAFT_R * (0.15 + 0.75 * h(k, 73));
            let span = 9.0;
            let y = feet.y + 4.5
                - (h(k, 74) * span + clock * drift * (0.4 + h(k, 75))).rem_euclid(span);
            let near = 1.0 - ((y - feet.y - 1.0).abs() / 5.0).clamp(0.0, 1.0);
            gem(
                vec3(r * a.sin(), y, r * a.cos()),
                0.05 + 0.04 * h(k, 76),
                with_a(Color::new(0.9, 0.82, 0.7, 1.0), 0.15 + 0.45 * near),
            );
        }
        flush_lines();
        self.edge_ready = edges_on && self.normal_pass(&c3, (iw, ih));
        self.edge_uniform = vec4(
            t[P::RenderEdgeDepth],
            t[P::RenderEdgeDark],
            t[P::RenderEdgeLight],
            t[P::RenderEdgeCrease],
        );
        self.grade_uniform = vec4(
            t[P::RenderGrade],
            t[P::RenderBloom],
            t[P::RenderVignette],
            if self.edge_ready { 1.0 } else { 0.0 },
        );
        self.present()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::descent::{PATH_R, SWEEP};

    /// 0174: 굴의 벽은 모두 굴 안(축)을 보고, 큰 뿌리의 윗면은 방랑자가 걷는 높이에 있다 — 발이 뿌리에 묻히거나 뜨지 않는다.
    #[test]
    fn the_wall_faces_inward_and_the_great_root_carries_the_walker() {
        let m = build(0, 40);
        assert!(
            m.faces.len() > 500 && m.faces.len() * 3 < 30_000,
            "{}",
            m.faces.len()
        );
        // 벽 면(가장 바깥 — 반지름이 큰 면)은 축을 본다.
        for f in &m.faces {
            let mid = (f.p[0] + f.p[1] + f.p[2]) / 3.0;
            let r = vec2(mid.x, mid.z).length();
            if r > SHAFT_R - 0.05 && !f.glow && f.n.y.abs() < 0.5 && r > SHAFT_R + 0.2 {
                assert!(
                    f.n.dot(vec3(-mid.x, 0.0, -mid.z)) > 0.0,
                    "wall faces the axis"
                );
            }
            assert!(f.n.length() > 0.9, "a degenerate face");
        }
        // 걷는 구간의 여러 자리에서: 발 바로 아래 큰 뿌리의 윗면이 있다(위를 보는 면, 발보다 조금 아래).
        for i in 0..=10 {
            let s = i as f32 / 10.0;
            let feet = dsc::path(s) + Vec3::Y * ROOT_R;
            let under = m.faces.iter().any(|f| {
                let mid = (f.p[0] + f.p[1] + f.p[2]) / 3.0;
                f.n.y > 0.8
                    && vec2(mid.x - feet.x, mid.z - feet.z).length() < 0.6
                    && (0.0..0.25).contains(&(feet.y - mid.y))
            });
            assert!(under, "no root top under the walker at s={s}");
        }
        // 굴 한가운데(카메라와 방랑자 사이)는 비어 있다: 축 둘레 1.2m 안에는 면이 없다.
        assert!(m
            .faces
            .iter()
            .flat_map(|f| f.p)
            .all(|q| vec2(q.x, q.z).length() > 1.2));
        // 길은 굴을 `SWEEP`만큼 돈다(상수가 서로 맞는지 — 길의 반지름은 벽 안).
        assert!(PATH_R + ROOT_R < SHAFT_R - 0.4 && SWEEP > 1.5);
    }
}
