//! 빛 모델: 달빛·화로·빛줄기를 정점 색으로(`lit`), 툰 띠·가장자리 빛, 그늘에 선 배우의 달빛 몫, 프레임마다 넣는 보는 눈(`Look`).
use super::*;

/// 빛을 어떻게 굽나(0033) — 렌더가 프레임마다 정한다(지형 굽기도 같은 값을 본다).
#[derive(Clone, Copy, Debug, Default)]
pub struct Look {
    /// 빛을 단계로 끊는다(툰): 달빛 2단, 화로 3단.
    pub toon: bool,
    /// 배우 가장자리 빛 세기(카메라에서 비스듬한 면일수록, 차가운 달빛 색).
    pub rim: f32,
    /// 배우 외곽선 두께(내부 해상도 px). 0 = 끔.
    pub outline_px: f32,
    /// 카메라 자리와 세로 시야각(라디안) — 가장자리 빛·외곽선 두께.
    pub eye: Vec3,
    pub fovy: f32,
    /// 렌더 타깃의 줄 수(0046 — 화면에 따라 바뀐다, 외곽선 두께를 내부 px로 셀 때).
    pub rows: f32,
    /// 조용한 바닥(0140): 줄눈·석판마다의 밝기 차를 줄인다(지형 굽기가 본다).
    pub calm: bool,
    /// 빛 대비(0140): 센 달빛 + 낮고 차가운 바탕빛 — 그림자가 또렷하다(`lit`·지형 굽기가 본다).
    pub key: bool,
    /// 적의 가독성(0170): 적 모델에만 곱하는 밝기(1 = 없음)와 달빛의 물을 빼는 몫(0 = 없음) — `enemy_tint`.
    pub enemy_lift: f32,
    pub enemy_ash: f32,
}

/// 달빛에 물든 색을 되돌리는 곱(0170): 남보라 빛을 받은 잿빛에 이것을 곱하면 붉은 쪽이 살고 푸른 쪽이 눌려 중성 잿빛이 된다.
const ASH: Color = Color::new(1.0, 0.93, 0.70, 1.0);

/// 적 모델의 색조(0170): 바닥과 같은 밝기·같은 색으로 묻히지 않게 — 더 밝게, 달빛의 물을 뺀 잿빛으로. "침식 = 색을 잃은 것"
/// (0037)이 남보라 세계에서 **색이 없어서** 눈에 띄게 한다(위험의 빨강·산 것의 파스텔은 건드리지 않는다).
pub fn enemy_tint() -> Color {
    let k = look();
    let mix = |c: f32| (1.0 + (c - 1.0) * k.enemy_ash) * k.enemy_lift;
    Color::new(mix(ASH.r), mix(ASH.g), mix(ASH.b), 1.0)
}

thread_local! {
    static LOOK: std::cell::Cell<Look> = const {
        std::cell::Cell::new(Look {
            toon: false,
            rim: 0.0,
            outline_px: 0.0,
            eye: Vec3::ZERO,
            fovy: 1.0,
            rows: INTERNAL_H as f32,
            calm: false,
            key: false,
            enemy_lift: 1.0,
            enemy_ash: 0.0,
        })
    };
}

thread_local! {
    /// 그늘에 선 배우(0140): 구운 지형의 가림 높이 지도(스테이지를 그리는 동안만) — 배우마다 광선 몇 개로 달빛 몫을 정한다.
    static ACTOR_SHADE: std::cell::RefCell<Option<std::rc::Rc<crate::shade::Occluders>>> = const { std::cell::RefCell::new(None) };
    /// 지금 그리는 배우가 받는 달빛 몫(0..1). 지형 굽기·배우 밖에서는 1.
    pub(super) static MOON_GATE: std::cell::Cell<f32> = const { std::cell::Cell::new(1.0) };
}

/// 그늘에 선 배우의 달빛 가림을 켠다(None = 끔 — 개발 아레나·인형·아이콘).
pub fn set_actor_shade(o: Option<std::rc::Rc<crate::shade::Occluders>>) {
    ACTOR_SHADE.with(|s| *s.borrow_mut() = o);
}

/// 발 자리 `at`에 선 배우가 받는 달빛 몫: 몸 높이 셋에서 달 쪽으로 쏜 광선 가운데 안 막힌 몫.
/// 다 막혀도 `ACTOR_SHADE_MIN`은 남긴다 — 그늘 속 적도 읽혀야 한다(외곽선·가장자리 빛은 그대로).
pub fn actor_moon(at: Vec3) -> f32 {
    ACTOR_SHADE.with(|s| match s.borrow().as_ref() {
        Some(o) => {
            let l = LIGHT_DIR.normalize();
            let open: f32 = [0.3, 0.9, 1.5]
                .iter()
                .map(|h| o.vis(at + Vec3::Y * *h, l))
                .sum::<f32>()
                / 3.0;
            ACTOR_SHADE_MIN + (1.0 - ACTOR_SHADE_MIN) * open
        }
        None => 1.0,
    })
}

pub(super) const ACTOR_SHADE_MIN: f32 = 0.35;

thread_local! {
    /// 지형 굽기 동안만 있는 달빛 가림 높이 지도(0044, `Scenery::bake`가 넣고 뺀다).
    static BAKE_SHADE: std::cell::RefCell<Option<crate::shade::Occluders>> = const { std::cell::RefCell::new(None) };
}

thread_local! {
    /// 달빛 세기(0118 T2 — 지역 환경 `Env::moonlight`, 땅속은 천장 틈으로 조금). 렌더가 프레임마다 넣는다.
    static MOONLIGHT: std::cell::Cell<f32> = const { std::cell::Cell::new(1.0) };
}

pub fn moonlight() -> f32 {
    MOONLIGHT.with(|m| m.get())
}

pub fn set_moonlight(k: f32) {
    MOONLIGHT.with(|m| m.set(k));
}

/// 굽기 동안 달빛 그림자를 켠다(None = 끔).
pub fn set_bake_shade(o: Option<crate::shade::Occluders>) {
    BAKE_SHADE.with(|s| *s.borrow_mut() = o);
}

pub fn look() -> Look {
    LOOK.with(|l| l.get())
}

pub fn set_look(l: Look) {
    LOOK.with(|c| c.set(l));
}

/// `x`(0..1)를 `n`단으로(반올림).
fn band(x: f32, n: f32) -> f32 {
    (x * n).round() / n
}

/// 빛(정점 색에 곱할 rgb): 바탕 + 달빛(고정 방향) + 가까운 화로(거리 제곱으로 준다, 면이 그쪽을 볼수록 밝다).
/// 툰(0033)이면 달빛은 2단, 화로 세기는 3단으로 끊는다 — 둥근 그러데이션 대신 또렷한 빛 웅덩이.
pub fn lit(p: Vec3, n: Vec3, lights: &[Light]) -> Vec3 {
    let toon = look().toon;
    let l = LIGHT_DIR.normalize();
    let mut d = n.dot(l).max(0.0);
    // 지형을 굽는 동안은 달빛 그림자(0044): 막히면 달빛 몫만 뺀다(화로 빛은 그대로).
    if d > 0.0 {
        d *= BAKE_SHADE.with(|s| s.borrow().as_ref().map_or(1.0, |o| o.vis(p + n * 0.05, l)));
    }
    if toon {
        d = band(d, 2.0);
    }
    d *= MOON_GATE.with(|g| g.get());
    let (ambient, moon) = if look().key {
        (AMBIENT_KEY, MOON_KEY)
    } else {
        (AMBIENT, MOON)
    };
    let mut c = ambient + moon * (d * moonlight());
    for li in lights {
        // 빛줄기(0124): 기둥 안이면 위에서 — 가운데일수록·위를 볼수록 밝다(둥근 화로 빛과 달리 높이로 줄지 않는다).
        if li.shaft {
            let d = vec2(p.x - li.pos.x, p.z - li.pos.z).length();
            if d < li.radius {
                let mut k = 1.0 - d / li.radius;
                k = k * (2.0 - k);
                if toon {
                    k = band(k, 2.0);
                }
                let up = 0.35 + 0.65 * n.y.max(0.0);
                c += vec3(li.color.r, li.color.g, li.color.b) * (SHAFT * k * up);
            }
            continue;
        }
        let dv = li.pos - p;
        let dist = dv.length();
        if dist >= li.radius {
            continue;
        }
        let mut att = (1.0 - dist / li.radius).powi(2);
        if toon {
            att = band(att, 3.0);
        }
        let facing = 0.35 + 0.65 * n.dot(dv / dist.max(1e-3)).max(0.0);
        c += vec3(li.color.r, li.color.g, li.color.b) * (FIRE * att * facing);
    }
    c
}

/// 가장자리 빛(0033): 카메라에서 비스듬히 돌아선 면일수록(`1 - n·v`의 제곱) 차가운 빛을 더한다 — 어두운 바닥 위 실루엣.
pub(super) fn rim(p: Vec3, n: Vec3, k: Look) -> Vec3 {
    if k.rim <= 0.0 {
        return Vec3::ZERO;
    }
    let v = (k.eye - p).normalize_or_zero();
    let e = 1.0 - n.dot(v).max(0.0);
    RIM * (k.rim * e * e)
}

const RIM: Vec3 = Vec3::new(0.55, 0.62, 0.85);

// 0037: 해 질 녘 — 바탕빛은 보랏빛을 조금, 달빛은 차갑게(파스텔 주인공이 그 위에서 선다).
const AMBIENT: Vec3 = Vec3::new(0.29, 0.28, 0.40);
const MOON: Vec3 = Vec3::new(0.44, 0.45, 0.62);
/// 빛 대비(0140): 달빛을 주광으로 세게, 바탕빛은 낮고 차갑게 — 달빛 받는 바닥의 밝기는 그대로(0.51 → 0.52) 두고 그림자만 어둡게(0.29 → 0.21),
/// 빛 : 그늘 = 1.8 : 1 → 2.5 : 1. 달을 바로 보는 면은 0.73 → 0.83(더 올리면 배우처럼 그리는 돌 문이 하얗게 뜬다). 그늘의 색은 검정이 아니라 남보라(0037).
const AMBIENT_KEY: Vec3 = Vec3::new(0.21, 0.21, 0.36);
const MOON_KEY: Vec3 = Vec3::new(0.62, 0.64, 0.86);
/// 달빛 받는 바닥에 견준 그늘의 밝기(붉은 칸 기준 — 툰이면 바닥의 달빛 몫은 반 단): 캐릭터 그림자(가림판)를 구운 그림자와 같은 진하기로.
pub fn floor_shade() -> f32 {
    let (a, m) = if look().key {
        (AMBIENT_KEY, MOON_KEY)
    } else {
        (AMBIENT, MOON)
    };
    let k = 0.5 * moonlight();
    1.0 - a.y / (a.y + m.y * k)
}
const FIRE: f32 = 1.5;
/// 구운 불꽃의 끝이 일렁이는 몫(0142, 밑동 0 → 끝 이만큼).
const FLAME_SWAY: f32 = 0.9;

/// 구운 불꽃의 바람 몫(음수 = 빠른 일렁임): 부위의 밑동(`y0`)은 0 — 그릇에 붙어 있다 — 끝(`y1`)으로 갈수록 제곱으로 커진다.
pub fn flame_sway(y: f32, y0: f32, y1: f32) -> f32 {
    let up = ((y - y0) / (y1 - y0).max(1e-3)).clamp(0.0, 1.0);
    -FLAME_SWAY * up * up
}

/// 빛줄기 세기(0124 — 땅속 달빛 0.4 아래에서 몸이 확 비치게).
const SHAFT: f32 = 1.1;
/// 움직이는 모델에 곱하는 밝기(배경과 떼어 읽히게).
pub(super) const ACTOR_LIFT: f32 = 1.35;

/// 고정 빛 방향 = 달(0044: 왼쪽 앞 위 45° — 벽·단·기둥의 그림자가 뜰 바닥 오른쪽·뒤로 떨어져 쿼터뷰에서 보이게;
/// 예전 오른쪽 앞에서는 그림자가 벽 바깥으로 떨어졌다). 면마다 `1 - shade + shade * max(0, n·l)`을 곱한다 — 기본 셰이더가 normal을 안 읽어 정점 색으로 굽는다
/// (KB reference/macroquad/3d-rendering.md). shade 0이면 민짜(전처럼), 1이면 그늘진 면이 검정에 가깝다.
pub const LIGHT_DIR: Vec3 = Vec3::new(-0.55, 0.65, 0.35);
