//! 그림 이펙트(0141): 월드 사건·상태 변화 → 입자(번쩍임·불똥·먼지·재). 월드는 이것을 모른다 — 소리(`sfx`)와 같은 길로 앱이 듣고
//! 여기에 넘긴다. 로직은 순수하다(난수 대신 번호 해시, 그림 핸들 없음) — 렌더(`Backdrop::fx`)는 `Fx::parts`를 읽어 그리기만.
//!
//! 원칙(KB `game-feel-particles-and-light` + 0141): 타격은 겹으로(번쩍임 1~2프레임 → 불똥 → 남는 먼지), 한 이펙트는 2~3색,
//! 타격은 짧고 밝게·몸짓(먼지)은 흙빛으로 낮게, 꾸밈은 많으면 건너뛰고 타격은 건너뛰지 않는다, 빨강은 쓰지 않는다(위험 예고 전용).
use crate::det::Det;
use macroquad::prelude::*;

use crate::enemy::{Atk, Blow, EState};
use crate::world::{Event, PState, World};

/// 타격 별의 크기 배(0242).
const STAR_SCALE: f32 = 0.7;

/// 입자의 꼴(렌더가 고른다).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// 카메라를 보는 네모 화소 덩이(불똥·부스러기·빛 알갱이).
    Dot,
    /// 날아가는 쪽으로 늘어난 빛줄.
    Streak,
    /// 커지며 옅어지는 먼지·재(계단진 원판).
    Puff,
    /// 네 갈래 번쩍임(타격의 한가운데 — 아주 짧다).
    Star,
    /// 진액 방울(0182): 맞은 쪽으로 튀어 떨어지는 액체 — 빛줄처럼 그리고, 땅에 닿으면(`Fx::settle`) `Splat`이 된다.
    Drop,
    /// 바닥의 진액 얼룩(0182): 땅에 눕힌 원판, 떨어질 때 가던 쪽으로 길쭉하다(`vel` = 그 쪽). 움직이지 않는다.
    Splat,
}

#[derive(Clone, Copy, Debug)]
pub struct Part {
    pub pos: Vec3,
    pub vel: Vec3,
    pub age: f32,
    pub life: f32,
    /// 처음·끝 크기(m).
    pub size: (f32, f32),
    /// 처음·끝 빛.
    pub color: (Color, Color),
    /// 아래로 당기는 세기(m/s²)와 속도가 주는 빠르기(/s).
    pub gravity: f32,
    pub drag: f32,
    pub shape: Shape,
    /// 꾸밈(몸짓·분위기) — 입자가 많으면 건너뛴다. 타격은 거짓.
    pub decor: bool,
}

impl Part {
    /// 수명의 몇 분의 몇(0..1).
    pub fn k(&self) -> f32 {
        (self.age / self.life.max(1e-3)).clamp(0.0, 1.0)
    }

    pub fn size_now(&self) -> f32 {
        self.size.0 + (self.size.1 - self.size.0) * self.k()
    }

    pub fn color_now(&self) -> Color {
        let (a, b, k) = (self.color.0, self.color.1, self.k());
        Color::new(
            a.r + (b.r - a.r) * k,
            a.g + (b.g - a.g) * k,
            a.b + (b.b - a.b) * k,
            a.a + (b.a - a.a) * k,
        )
    }
}

/// 바닥·허공에 잠깐 남는 빛의 꼴(0158 G2 — 스킬마다 다른 서명): 사건이 만들고(`Fx::on_events`) 렌더가 그린다(`Renderer::marks`).
/// 입자가 아니라 **모양**이라 입자 수 상한을 쓰지 않는다. `k` = 수명의 몫(0..1)으로 자라고 옅어진다.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MarkKind {
    /// 바닥 고리: 반지름이 `r0` → `r1`(퍼지거나 조여든다).
    Ring { r0: f32, r1: f32 },
    /// 길을 따라 남는 빛 띠(가로지르기): `dir` 쪽 `len` m, 반 폭 `half` — 떠난 자리(꼬리)부터 가늘어지며 사라진다.
    Streak { dir: Vec3, len: f32, half: f32 },
    /// 부채꼴을 가로질러 바깥으로 달리는 충격파 호(땅울림): `r0` → `r1`.
    Shock {
        dir: Vec3,
        r0: f32,
        r1: f32,
        arc_deg: f32,
    },
    /// 땅의 금 `n`줄(`dir` 쪽 `arc_deg`° 안에서 `r` m까지 뻗는다 — 360이면 사방).
    Cracks {
        dir: Vec3,
        r: f32,
        n: u8,
        arc_deg: f32,
    },
    /// 한 점으로 모여드는 빛줄 `n`개(끌어모으기): `r` m 밖에서 안으로.
    Converge { r: f32, n: u8 },
    /// 길을 가르는 빛의 면(한 줄기): `dir` 쪽 `len` m, 반 폭 `half` — 흰 속 + 금빛 겉, 길게 남는다.
    Slash { dir: Vec3, len: f32, half: f32 },
}

/// 남는 빛이 어디에 붙나(0278 R21 — `mark_with`의 bool 둘 `follow` · `hold`; 셋째 짝 "버티되 따르지 않음"은 없다).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Stick {
    /// 놓인 자리에 남는다.
    Ground,
    /// 방랑자를 따라간다(몸에 두른 것).
    Body,
    /// 몸을 따라가되 버티는 동안에만(버팀이 끊기면 지운다).
    Bracing,
}

/// 남는 빛 하나. `delay` 뒤에 나타나 `life` 동안 산다. `follow` = 방랑자를 따라간다(몸에 두른 것),
/// `hold` = 버티는 동안에만(버팀이 끊기면 지운다).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mark {
    pub pos: Vec3,
    pub kind: MarkKind,
    pub age: f32,
    pub life: f32,
    pub delay: f32,
    pub color: Color,
    pub follow: bool,
    pub hold: bool,
}

impl Mark {
    pub fn k(&self) -> f32 {
        (self.age / self.life.max(1e-3)).clamp(0.0, 1.0)
    }

    pub fn live(&self) -> bool {
        self.delay <= 0.0
    }
}

/// 남는 빛의 수 상한(넘으면 가장 오래된 것부터 버린다).
pub const MARK_CAP: usize = 40;

/// 잔불 흩기의 불티(0158 G2 — 상태 없음, 화로 불티와 같은 꼴): 불티 `i`가 시각 `time`에 있는 자리(가운데·바닥 기준, 반지름 1의
/// 원판 안 — 렌더가 반지름을 곱한다)와 진하기(0..1). 주기마다 바닥에서 떠올라 흔들리며 식는다.
pub fn field_mote(i: usize, time: f32) -> (Vec3, f32) {
    let h = |n: u32| {
        let mut x = (i as u32).wrapping_mul(19_349_663) ^ n.wrapping_mul(83_492_791) ^ 0x9e37;
        x ^= x >> 13;
        x = x.wrapping_mul(0x5bd1_e995);
        x ^= x >> 15;
        (x & 0xffff) as f32 / 65535.0
    };
    let period = 0.9 + 0.9 * h(1);
    let ph = ((time + h(2) * period) / period).fract();
    let a = h(3) * std::f32::consts::TAU;
    // 넓이에 고르게(√) — 가장자리까지.
    let r = h(4).sqrt() * 0.96;
    let sway = (time * (2.0 + 2.0 * h(5)) + h(6) * 6.0).sin_d() * 0.05 * ph;
    let pos = vec3(
        a.sin_d() * r + sway,
        0.08 + (0.7 + 0.9 * h(7)) * ph,
        a.cos_d() * r,
    );
    let k = (ph / 0.12).min(1.0) * ((1.0 - ph) / 0.35).min(1.0);
    (pos, k)
}
/// 화살비의 화살(0272 — 상태 없음, 불티와 같은 꼴): 화살 `i`가 시각 `time`에 있는 자리(반지름 1의 원판 안 — 렌더가 반지름을
/// 곱한다, y = 남은 높이의 몫 1 → 0)와 진하기. 저마다의 박자로 위에서 곧게 내리꽂히고, 땅에 닿으면 다음 화살이 다른 박자로 온다.
pub fn rain_arrow(i: usize, time: f32) -> (Vec3, f32) {
    let h = |n: u32| {
        let mut x = (i as u32).wrapping_mul(40_503_581) ^ n.wrapping_mul(2_654_435_761) ^ 0x51ed;
        x ^= x >> 13;
        x = x.wrapping_mul(0x5bd1_e995);
        x ^= x >> 15;
        (x & 0xffff) as f32 / 65535.0
    };
    let period = 0.32 + 0.22 * h(1);
    let t = (time + h(2) * period) / period;
    let (round, ph) = (t.floor() as u32, t.fract());
    // 내릴 때마다 자리가 바뀐다(같은 자리에만 꽂히면 무늬로 읽힌다).
    let j = |n: u32| h(n.wrapping_add(round.wrapping_mul(7)));
    let a = j(3) * std::f32::consts::TAU;
    let r = j(4).sqrt() * 0.94;
    let pos = vec3(a.sin_d() * r, 1.0 - ph, a.cos_d() * r);
    (pos, (ph / 0.15).min(1.0))
}
/// 화살비에 한꺼번에 보이는 화살의 수.
pub const RAIN_ARROWS: usize = 16;

/// 불티 하나의 수(반지름 4m 기준 — 렌더가 넓이에 맞춰 늘린다, 상한 `FIELD_MOTES_MAX`).
pub const FIELD_MOTES: usize = 44;
pub const FIELD_MOTES_MAX: usize = 72;

/// 분노의 불씨(0277)가 보이기 시작하는 분노(0..1 — 적 둘에 에워싸이면 0.5).
const FURY_MOTES_FROM: f32 = 0.5;

/// 퍼지는 고리의 앞머리에서 틱마다 튀는 돌 부스러기 수(0269).
const WAVE_CHIPS: usize = 3;

/// 입자 수 상한: 꾸밈은 `DECOR_CAP`을 넘으면 새로 내지 않고, 타격은 `HARD_CAP`까지(그 뒤엔 가장 오래된 것부터 버린다).
pub const DECOR_CAP: usize = 220;
pub const HARD_CAP: usize = 420;

// 빛깔(2~3색씩 골라 쓴다). 따뜻한 흰빛·호박빛은 마무리 패스(0140)의 번짐을 받는다.
const WHITE_HOT: Color = Color::new(1.0, 0.97, 0.86, 1.0);
const AMBER: Color = Color::new(1.0, 0.72, 0.32, 1.0);
const GOLD: Color = Color::new(1.0, 0.86, 0.45, 1.0);
const COOL: Color = Color::new(0.80, 0.92, 1.0, 1.0);
const STEEL: Color = Color::new(0.74, 0.78, 0.88, 1.0);
const LAVENDER: Color = Color::new(0.82, 0.74, 1.0, 1.0);
/// 스킬의 민트(0151 — 땅을 다루는 기술의 빛).
const MINT: Color = Color::new(0.66, 1.0, 0.86, 1.0);
/// 균열의 하늘빛(0223 V2 — 패링의 `COOL`보다 짙다: 금 간 적의 표지와 같은 빛).
const SKY: Color = Color::new(0.50, 0.80, 1.0, 1.0);
// 0270 B 새 타입의 빛(`status::INK`와 같은 값): 충격 레몬 · 서리 얼음 · 흡수 풀빛.
const SHOCK: Color = Color::new(1.0, 0.96, 0.45, 1.0);
const ICE: Color = Color::new(0.55, 0.95, 1.0, 1.0);
const LIFE: Color = Color::new(0.55, 0.95, 0.45, 1.0);
const HEAL: Color = Color::new(1.0, 0.86, 0.56, 1.0);
/// 급소의 번쩍임(0277 — 쌍단검): 날 끝의 서늘한 은빛(살짝 연보라) — 평타의 따뜻한 흰빛 · 패링의 하늘빛 `COOL` · 치명타의 금과 갈린다.
const SNEAK: Color = Color::new(0.88, 0.86, 1.0, 1.0);
/// 분노의 불씨(0277 — 대검): 달군 쇠의 주황 → 식은 흙빛 금. 몰아치기의 `GOLD`보다 붉게, 빨강(예고)은 아니게(초록 0.5 넘게).
const FURY: Color = Color::new(1.0, 0.58, 0.26, 1.0);
const FURY_COOL: Color = Color::new(0.86, 0.70, 0.52, 1.0);
const FURY_HOT: Color = Color::new(1.0, 0.78, 0.45, 1.0);
/// 걸어 당김의 덩굴(0277 — 낫): 흙빛 뿌리 + 연한 새순(렌더 `pull_vines`와 같은 빛).
const VINE: Color = Color::new(0.58, 0.44, 0.30, 1.0);
const SPROUT: Color = Color::new(0.70, 0.92, 0.58, 1.0);
/// 먼지 = 바닥 석판보다 한 단 밝은 남보라 — 바닥과 같은 빛이면(처음 0.50·0.48·0.62) 깔리는 게 아니라 안 보였다.
const DUST: Color = Color::new(0.63, 0.61, 0.74, 0.72);
/// 쓰러진 적에게서 풀려나는 재(바랜 잿빛 라벤더, 0037)와 뿌리의 보라.
const ASH: Color = Color::new(0.72, 0.70, 0.80, 0.75);
const ROOT: Color = Color::new(0.56, 0.38, 0.70, 1.0);
const SMOKE: Color = Color::new(0.22, 0.21, 0.30, 0.8);
/// 적의 힘(0269 — 적의 공격이 풀린 순간): 옅은 라벤더 흰빛. 방랑자 스킬의 금·민트, 패링의 `COOL`과 갈리고 빨강(예고)은 아니다.
const QUAKE: Color = Color::new(0.92, 0.88, 1.0, 1.0);
/// 적이 친 땅의 금(0269): 뿌리 보라가 비쳐 오른다 — 식으면 흙빛(렌더의 금 꼴).
const RIFT_ROOT: Color = Color::new(0.74, 0.52, 0.96, 1.0);
/// 보스(문지기·종지기·권양지기)의 타는 잡몹의 이만큼 크게(0269).
const BOSS_BLOW: f32 = 1.45;
/// 적의 충격파가 남는 초(0269 — 방랑자 땅울림의 0.26보다 조금 길게: 맞은 쪽도 어디까지였나를 읽는다).
const BLOW_SHOCK_LIFE: f32 = 0.34;
const STONE: Color = Color::new(0.40, 0.38, 0.50, 1.0);
/// 뿌리 진액(0182 — "피 튀듯"): 적을 감은 보라 뿌리의 물. 빨강은 예고 전용이라(0037) 피 대신 이것 — 튀는 동안은 밝은 보라,
/// 식으며 짙어지고, 바닥에는 짙은 자두빛으로 남는다(남보라 바닥보다 어둡고 채도가 높아 얼룩으로 읽힌다).
const SAP: Color = Color::new(0.80, 0.50, 1.0, 1.0);
const SAP_DEEP: Color = Color::new(0.46, 0.20, 0.66, 1.0);
const SAP_STAIN: Color = Color::new(0.30, 0.10, 0.42, 0.85);
/// 바닥 얼룩이 남는 시간(초) — 방울이 나는 시간(`SAP_FLIGHT`)과 합쳐 2초 안.
pub const SPLAT_LIFE: f32 = 1.0;
/// 진액 방울이 땅에 닿지 못하면(벼랑 밖) 사라지는 시간.
const SAP_FLIGHT: f32 = 1.0;

fn fade(c: Color) -> Color {
    Color::new(c.r, c.g, c.b, 0.0)
}

#[derive(Default)]
pub struct Fx {
    pub parts: Vec<Part>,
    /// 남는 빛(0158 G2 — 렌더가 그린다).
    pub marks: Vec<Mark>,
    /// 조금 뒤에 나올 입자(한 줄기의 늦은 불똥): (남은 초, 입자).
    later: Vec<(f32, Part)>,
    /// 몰아치기의 기운을 낸 뒤 흐른 시간.
    aura_t: f32,
    /// 번호 해시의 씨(게임 난수와 따로 — 이펙트가 놀이를 바꾸지 않는다).
    seed: u32,
    was_dodging: bool,
    was_air: bool,
    /// 적마다: 지난 틱에 돌진 중이었나, 돌진 먼지를 낸 뒤 흐른 시간.
    enemy_was: Vec<(bool, f32)>,
    run_dist: f32,
    last_pos: Option<Vec3>,
    /// 분노의 불씨를 낸 뒤 흐른 시간(0277).
    fury_t: f32,
}

impl Fx {
    /// 0..1(번호마다 다른 값, 같은 차례 = 같은 값).
    fn r(&mut self) -> f32 {
        self.seed = self
            .seed
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        ((self.seed >> 8) & 0xffff) as f32 / 65535.0
    }

    /// −1..1.
    fn rs(&mut self) -> f32 {
        self.r() * 2.0 - 1.0
    }

    fn push(&mut self, p: Part) {
        let cap = if p.decor { DECOR_CAP } else { HARD_CAP };
        if self.parts.len() >= cap {
            if p.decor {
                return;
            }
            self.parts.remove(0);
        }
        self.parts.push(p);
    }

    fn mark(&mut self, pos: Vec3, kind: MarkKind, life: f32, color: Color) {
        self.mark_with(pos, kind, life, color, 0.0, Stick::Ground);
    }

    fn mark_with(
        &mut self,
        pos: Vec3,
        kind: MarkKind,
        life: f32,
        color: Color,
        delay: f32,
        stick: Stick,
    ) {
        let (follow, hold) = match stick {
            Stick::Ground => (false, false),
            Stick::Body => (true, false),
            Stick::Bracing => (true, true),
        };
        if self.marks.len() >= MARK_CAP {
            self.marks.remove(0);
        }
        self.marks.push(Mark {
            pos,
            kind,
            age: 0.0,
            life,
            delay,
            color,
            follow,
            hold,
        });
    }

    /// 바깥 고리 `r`에서 가운데로 빨려드는 빛 알갱이 `n`개(끌어모으기의 모임).
    fn implode(&mut self, at: Vec3, r: f32, n: usize, col: Color) {
        let turn = self.r() * std::f32::consts::TAU;
        for i in 0..n {
            let a = turn + i as f32 / n as f32 * std::f32::consts::TAU + self.rs() * 0.25;
            let d = vec3(a.sin_d(), 0.0, a.cos_d());
            let rr = r * (0.55 + 0.45 * self.r());
            let life = 0.22 + 0.08 * self.r();
            let h = 0.3 + 0.6 * self.r();
            self.push(Part {
                pos: at + d * rr + Vec3::Y * h,
                vel: -d * (rr / life) * 1.6,
                age: 0.0,
                life,
                size: (0.11, 0.05),
                color: (col, fade(WHITE_HOT)),
                gravity: 0.0,
                drag: 3.0,
                shape: Shape::Streak,
                decor: false,
            });
        }
    }

    /// 반지름 `r`의 고리에서 바깥으로 튀는 빛줄 `n`개(허리 높이 — 휘돌이·터뜨림·울림).
    #[allow(clippy::too_many_arguments)]
    fn ring_sparks(
        &mut self,
        at: Vec3,
        r: f32,
        n: usize,
        speed: f32,
        life: f32,
        from: Color,
        to: Color,
    ) {
        let turn = self.r() * std::f32::consts::TAU;
        for i in 0..n {
            let a = turn + i as f32 / n as f32 * std::f32::consts::TAU + self.rs() * 0.15;
            let d = vec3(a.sin_d(), 0.0, a.cos_d());
            let sp = speed * (0.7 + 0.6 * self.r());
            let l = life * (0.7 + 0.5 * self.r());
            let up = 0.5 + 1.5 * self.r();
            self.push(Part {
                pos: at + d * r,
                vel: d * sp + Vec3::Y * up,
                age: 0.0,
                life: l,
                size: (0.08, 0.04),
                color: (from, fade(to)),
                gravity: 3.0,
                drag: 2.5,
                shape: Shape::Streak,
                decor: false,
            });
        }
    }

    /// 한가운데 번쩍임 하나(네 갈래 별). 크기는 부르는 값의 `STAR_SCALE`배(0242 — 사장 설문 "별이 너무 두껍다": 1.5~2.6 m의 별이 몸을 덮었다).
    fn star(&mut self, at: Vec3, size: f32, life: f32, col: Color) {
        let size = size * STAR_SCALE;
        self.push(Part {
            pos: at,
            vel: Vec3::ZERO,
            age: 0.0,
            life,
            size: (size, size * 0.25),
            color: (col, col),
            gravity: 0.0,
            drag: 0.0,
            shape: Shape::Star,
            decor: false,
        });
    }

    /// `dir` 쪽 원뿔(벌어짐 `spread` — 0 = 한 줄, 1 = 사방)로 튀는 불똥 `n`개.
    #[allow(clippy::too_many_arguments)]
    fn spray(
        &mut self,
        at: Vec3,
        dir: Vec3,
        spread: f32,
        n: usize,
        speed: f32,
        shape: Shape,
        size: f32,
        life: f32,
        gravity: f32,
        from: Color,
        to: Color,
    ) {
        let dir = dir.normalize_or(Vec3::Y);
        for _ in 0..n {
            let wob = vec3(self.rs(), self.rs() * 0.7 + 0.25, self.rs());
            let v = (dir * (1.0 - spread) + wob * spread).normalize_or(Vec3::Y);
            let sp = speed * (0.55 + 0.75 * self.r());
            let l = life * (0.7 + 0.5 * self.r());
            self.push(Part {
                pos: at,
                vel: v * sp,
                age: 0.0,
                life: l,
                size: (size, size * 0.5),
                color: (from, fade(to)),
                gravity,
                drag: 2.5,
                shape,
                decor: false,
            });
        }
    }

    /// 진액이 튄다(0182): `at`에서 `dir`(수평, 맞은 쪽) 둘레 부채꼴로 방울 `n`개 — 셋에 하나는 굵은 덩이. 위로 솟았다 떨어져
    /// 바닥에 얼룩을 남긴다(`settle`). `up` = 위로 솟는 몫(쓰러짐은 크게 — 사방으로 터진다), `dir`가 0이면 사방.
    fn sap(&mut self, at: Vec3, dir: Vec3, n: usize, speed: f32, up: f32) {
        let side = vec3(-dir.z, 0.0, dir.x);
        let around = dir.length_squared() < 1e-4;
        for i in 0..n {
            let flat = if around {
                let a = self.r() * std::f32::consts::TAU;
                vec3(a.sin_d(), 0.0, a.cos_d())
            } else {
                // 맞은 쪽보다 옆으로(칼이 지나가는 쪽 — 베인 피는 날을 따라 튄다): 곧장 뒤로 튀면 카메라(방랑자 뒤)에서
                // 맞은 몸에 가려 안 보였다(헤드리스 화면). 좌우 번갈아 30~80°.
                let sign = if i % 2 == 0 { 1.0 } else { -1.0 };
                let a = (30.0 + 50.0 * self.r()).to_radians();
                (dir * a.cos_d() + side * (sign * a.sin_d())).normalize_or(dir)
            };
            let glob = i % 3 == 0;
            let sp = speed * (0.5 + 0.7 * self.r()) * if glob { 0.75 } else { 1.0 };
            let vel = flat * sp + Vec3::Y * (up * (0.6 + 0.8 * self.r()));
            // 락온 거리 1m ≈ 46px: 방울 4~5px, 덩이 7~8px(0.055m짜리 2px 방울은 화소 하나로 사라졌다).
            let size = if glob { 0.17 } else { 0.10 } * (0.8 + 0.4 * self.r());
            self.push(Part {
                pos: at + flat * 0.1,
                vel,
                age: 0.0,
                life: SAP_FLIGHT,
                size: (size, size * 0.8),
                color: (SAP, SAP_DEEP),
                gravity: 15.0,
                drag: 1.2,
                shape: Shape::Drop,
                decor: false,
            });
        }
    }

    /// 진액 방울이 땅에 닿으면 얼룩이 된다(0182). `ground` = 그 자리의 땅 높이(앱은 월드의 `ground_y`).
    pub fn settle(&mut self, ground: impl Fn(Vec3) -> f32) {
        for p in &mut self.parts {
            if p.shape != Shape::Drop || p.vel.y > 0.0 {
                continue;
            }
            let g = ground(p.pos);
            if p.pos.y > g {
                continue;
            }
            let flat = vec3(p.vel.x, 0.0, p.vel.z);
            let size = p.size_now() * 2.0;
            *p = Part {
                pos: vec3(p.pos.x, g + 0.03, p.pos.z),
                vel: flat,
                age: 0.0,
                life: SPLAT_LIFE,
                size: (size, size),
                color: (SAP_STAIN, fade(SAP_STAIN)),
                gravity: 0.0,
                drag: 0.0,
                shape: Shape::Splat,
                decor: true,
            };
        }
    }

    /// 꺼진 화로에 불이 붙는다(0171 — 사건이 아니라 앱이 부른다: 월드는 화로를 모른다): 그릇 위의 번쩍임 + 솟는 불티 +
    /// 둘레로 튀는 금빛 줄. `at` = 화로의 발밑.
    pub fn kindle(&mut self, at: Vec3) {
        let bowl = at + Vec3::Y * 0.9;
        self.star(bowl + Vec3::Y * 0.15, 1.5, 0.16, WHITE_HOT);
        self.spray(
            bowl,
            Vec3::Y,
            0.45,
            26,
            3.4,
            Shape::Dot,
            0.09,
            1.1,
            1.2,
            WHITE_HOT,
            AMBER,
        );
        self.ring_sparks(bowl, 0.3, 12, 2.6, 0.5, GOLD, AMBER);
    }

    /// 아이템이 바닥에 떨어진다(0226 — 사장 "바닥에 떨어지고 비주얼적인 효과가 있어야 해"): 번쩍임 + 위로 솟는 빛 알갱이 + 둘레로
    /// 튀는 빛줄. 빛은 등급의 빛(`col`), `rank` 0~3(보통~고유) — 드물수록 크고 많다. 사건이 아니라 앱이 부른다(월드는 아이템을 모른다).
    pub fn loot_drop(&mut self, at: Vec3, col: Color, rank: u8) {
        let k = rank.min(3) as f32;
        let c = at + Vec3::Y * 0.5;
        self.star(c, 0.9 + 0.35 * k, 0.14 + 0.02 * k, WHITE_HOT);
        self.spray(
            c,
            Vec3::Y,
            0.55,
            8 + 6 * rank.min(3) as usize,
            2.6 + 0.5 * k,
            Shape::Dot,
            0.08,
            0.8 + 0.15 * k,
            2.2,
            WHITE_HOT,
            col,
        );
        self.ring_sparks(
            at + Vec3::Y * 0.15,
            0.25,
            6 + 4 * rank.min(3) as usize,
            2.0 + 0.5 * k,
            0.45,
            col,
            col,
        );
    }

    /// 아이템을 주웠다(0226): 그 자리에서 몸 쪽으로 솟는 빛 알갱이 + 짧은 번쩍임.
    pub fn loot_take(&mut self, at: Vec3, col: Color) {
        let c = at + Vec3::Y * 0.5;
        self.star(c, 0.8, 0.12, col);
        self.spray(
            c,
            Vec3::Y,
            0.3,
            10,
            3.0,
            Shape::Streak,
            0.07,
            0.4,
            0.0,
            WHITE_HOT,
            col,
        );
    }

    /// 바닥에서 둘레로 퍼지는 먼지 `n`덩이(반지름 `r`의 고리에서 바깥으로 `speed`).
    fn dust_ring(&mut self, at: Vec3, r: f32, n: usize, speed: f32, size: f32, col: Color) {
        let turn = self.r() * std::f32::consts::TAU;
        for i in 0..n {
            let a = turn + i as f32 / n as f32 * std::f32::consts::TAU + self.rs() * 0.2;
            let d = vec3(a.sin_d(), 0.0, a.cos_d());
            let s = size * (0.8 + 0.5 * self.r());
            let vel = d * speed * (0.7 + 0.5 * self.r()) + Vec3::Y * (0.3 + 0.5 * self.r());
            let life = 0.42 + 0.2 * self.r();
            self.push(Part {
                pos: at + d * r + Vec3::Y * 0.12,
                vel,
                age: 0.0,
                life,
                size: (s * 0.55, s * 1.5),
                color: (col, fade(col)),
                gravity: 0.0,
                drag: 3.5,
                shape: Shape::Puff,
                decor: true,
            });
        }
    }

    /// 한쪽으로 밀리는 먼지 `n`덩이(회피·돌진의 발밑).
    fn dust_kick(&mut self, at: Vec3, away: Vec3, n: usize, size: f32) {
        let away = away.normalize_or_zero();
        for _ in 0..n {
            let side = vec3(-away.z, 0.0, away.x) * self.rs() * 0.6;
            let s = size * (0.8 + 0.5 * self.r());
            let vel = (away + side) * (1.4 + 1.2 * self.r()) + Vec3::Y * (0.4 + 0.4 * self.r());
            let life = 0.36 + 0.18 * self.r();
            self.push(Part {
                pos: at + side * 0.3 + Vec3::Y * 0.1,
                vel,
                age: 0.0,
                life,
                size: (s * 0.5, s * 1.4),
                color: (DUST, fade(DUST)),
                gravity: 0.0,
                drag: 4.0,
                shape: Shape::Puff,
                decor: true,
            });
        }
    }

    /// 튀어 오르는 돌 부스러기 `n`개: `g`(바닥)에서 `dir` 쪽 `arc_deg`°(360 = 사방) 안, 반지름 `range`의 25~95% 자리에서.
    fn chips(&mut self, g: Vec3, dir: Vec3, range: f32, arc_deg: f32, n: usize) {
        let half = (arc_deg * 0.4).to_radians();
        let yaw = dir.x.atan2_d(dir.z);
        for _ in 0..n {
            let a = yaw + self.rs() * half;
            let rr = range * (0.25 + 0.7 * self.r());
            let p = g + vec3(a.sin_d() * rr, 0.1, a.cos_d() * rr);
            let up = 4.0 + 3.5 * self.r();
            let life = 0.45 + 0.2 * self.r();
            let size = 0.1 + 0.08 * self.r();
            self.push(Part {
                pos: p,
                vel: vec3(a.sin_d() * 1.5, up, a.cos_d() * 1.5),
                age: 0.0,
                life,
                size: (size, size * 0.7),
                color: (STONE, STONE),
                gravity: 16.0,
                drag: 0.6,
                shape: Shape::Dot,
                decor: false,
            });
        }
    }

    /// 위로 떠오르며 옅어지는 것 `n`개(재·회복의 빛 — 몸 둘레 `r` 안에서).
    #[allow(clippy::too_many_arguments)]
    fn rise(
        &mut self,
        at: Vec3,
        r: f32,
        n: usize,
        shape: Shape,
        size: (f32, f32),
        life: f32,
        from: Color,
        to: Color,
        decor: bool,
    ) {
        for _ in 0..n {
            let off = vec3(self.rs() * r, self.r() * r * 1.2, self.rs() * r);
            let l = life * (0.7 + 0.6 * self.r());
            let vel = vec3(self.rs() * 0.3, 0.7 + 0.9 * self.r(), self.rs() * 0.3);
            self.push(Part {
                pos: at + off,
                vel,
                age: 0.0,
                life: l,
                size,
                color: (from, fade(to)),
                gravity: -0.4,
                drag: 1.2,
                shape,
                decor,
            });
        }
    }

    /// 월드 사건 → 이펙트. `w`는 방금 그 틱을 마친 월드(읽기만).
    pub fn on_events(&mut self, events: &[Event], w: &World) {
        for e in events {
            // 묶음마다 제 사건만 안다(사건은 한 묶음에만 든다 — 시험 `every_event_belongs_to_at_most_one_group`).
            let _ = self.on_strike(e, w)
                || self.on_ail(e, w)
                || self.on_skill_event(e, w)
                || self.on_body(e, w);
        }
    }

    /// 사건 묶음: 타격(평타·치명타·강타·패링·막기·자세 깨기·쓰러짐·내려찍기·벽 쿵·터짐 …). 이 묶음의 사건이면 참(0251 R11 — `on_events`의 match를 나눴다).
    fn on_strike(&mut self, e: &Event, w: &World) -> bool {
        let pp = w.player.pos;
        // 맞은 쪽 = 플레이어에서 그 자리로(수평) — 불똥이 "밀어낸 쪽"으로 튄다.
        let from_player = |at: Vec3| vec3(at.x - pp.x, 0.0, at.z - pp.z).normalize_or(Vec3::Z);
        match *e {
            Event::Hit { at } => self.ev_hit(at, &from_player),
            Event::Sneak { at } => self.ev_sneak(at, &from_player),
            Event::Crit { at } => self.ev_crit(at, &from_player),
            Event::Smash { at } => self.ev_smash(at, &from_player, w),
            Event::Parry { at } => self.ev_parry(at),
            Event::Deflect { at } => self.ev_deflect(at, &from_player),
            Event::Guard { at } => self.ev_guard(at, &from_player),
            Event::GuardBreak { at } => self.ev_guard_break(at),
            Event::Break { at } => self.ev_break(at),
            Event::LoadBreak { at } => self.ev_load_break(at),
            Event::EnemyDown { at } => self.ev_enemy_down(at),
            Event::Plunge { at } | Event::Aftershock { at } => self.ev_plunge(at, w),
            Event::Slam { at } => self.ev_slam(at, w),
            Event::Boom { at } => self.ev_boom(at, w),
            Event::Gulp { at } => self.ev_gulp(at),
            Event::EnemyStrike {
                at,
                dir,
                blow,
                boss,
            } => self.enemy_blow(at, dir, blow, boss, w),
            Event::Interrupt { at } => self.ev_interrupt(at, w),
            _ => return false,
        }
        true
    }

    /// 적의 공격이 풀린 순간(0269 — 사장 "공격예고와 공격 이펙트는 다른 것 … 공격예고를 켜고 끄고와는 상관없이 보여야 하고
    /// 비주얼적으로 확실히"): 꼴마다 다른 타격 — 한가운데 별 + 닿는 거리까지 달리는 충격파 + 갈라지는 땅 + 튀는 돌. 크기는 판정과 같은
    /// 값(그림이 판정보다 크게 말하지 않는다), 보스는 별·부스러기만 `BOSS_BLOW`배. 빛은 적의 것(`QUAKE`·`RIFT_ROOT`·돌) — 빨강 없음.
    /// 퍼지는 고리·솟는 뿌리 줄의 앞머리는 렌더가(단단한 가시), 앞머리의 돌 부스러기는 `watch`가 그린다.
    fn enemy_blow(&mut self, at: Vec3, dir: Vec3, blow: Blow, boss: bool, w: &World) {
        let s = if boss { BOSS_BLOW } else { 1.0 };
        let n = |k: f32| (k * s).round() as usize;
        let g = vec3(at.x, w.ground_y(at), at.z);
        match blow {
            Blow::Around { r } => self.blow_around(r, dir, g, s, &n),
            Blow::Cone { r, arc } => self.blow_cone(r, arc, dir, g, s, &n),
            Blow::Spot { r } => self.blow_spot(r, dir, g, s, &n),
            Blow::Ram => self.blow_ram(dir, g, s, &n),
            Blow::Quake { r } => self.blow_quake(r, dir, g, s, &n),
            Blow::Line { len } => self.blow_line(len, dir, g, s, &n),
            Blow::Hook { .. } => self.blow_hook(dir, g, s, &n),
            Blow::Volley => self.blow_volley(dir, g, s, &n),
        }
    }

    /// 사건 묶음: 피해 타입(0223 V2 — 걸림·화상·울림·터짐)·솟은 뿌리·회복. 이 묶음의 사건이면 참(0251 R11 — `on_events`의 match를 나눴다).
    fn on_ail(&mut self, e: &Event, w: &World) -> bool {
        match *e {
            Event::Ail { at, kind } => self.ev_ail(at, kind, w),
            Event::Bleed { at, dir } => self.ev_bleed(at, dir),
            Event::Shatter { at } => self.ev_shatter(at, w),
            Event::Drain { from } => self.ev_drain(from, w),
            Event::Burn { at } => self.ev_burn(at),
            Event::Toll { at, radius } => self.ev_toll(at, radius, w),
            Event::AilBurst { at, radius } => self.ev_ail_burst(at, radius, w),
            Event::RootSpike { at } => self.ev_root_spike(at, w),
            Event::Mend { at } => self.ev_mend(at),
            _ => return false,
        }
        true
    }

    /// 사건 묶음: 마법·딱 맞춘 떼기·스킬과 큰 기술(0151 → 0158 G2). 이 묶음의 사건이면 참(0251 R11 — `on_events`의 match를 나눴다).
    fn on_skill_event(&mut self, e: &Event, w: &World) -> bool {
        let pp = w.player.pos;
        match *e {
            Event::Cast {
                at,
                look:
                    crate::weapons::Look::Axe
                    | crate::weapons::Look::Javelin
                    | crate::weapons::Look::Knife,
            } => self.ev_cast_thrown(at, w),
            Event::Cast {
                at,
                look: crate::weapons::Look::Quake,
            } => self.ev_cast_quake(at, pp, w),
            Event::Cast {
                at,
                look: crate::weapons::Look::Curse,
            } => self.ev_cast_curse(at, w),
            Event::Cast {
                at,
                look: crate::weapons::Look::Seed,
            } => self.ev_cast_seed(at),
            Event::Throw => self.ev_throw(pp, w),
            Event::Cast {
                at,
                look: crate::weapons::Look::Stone,
            } => self.ev_cast_stone(at, w),
            Event::Cast {
                at,
                look: crate::weapons::Look::Light,
            } => self.ev_cast_light(at),
            Event::Smite { at, radius } => self.ev_smite(at, radius),
            Event::Sprout { at, radius } => self.ev_sprout(at, radius),
            Event::JavelinBack { at } => self.ev_javelin_back(at),
            Event::Prayer { at, radius } => self.ev_prayer(at, radius, w),
            Event::SnareSet { at } => self.ev_snare_set(at),
            Event::SnareSnap { at } => self.ev_snare_snap(at),
            Event::ShieldBounce { at } => self.ev_shield_bounce(at, w),
            Event::Rush { at, dir, power } => self.ev_rush(at, dir, power, w),
            Event::Cast { at, .. } => self.ev_cast_magic(at),
            Event::Perfect { at } => self.ev_perfect(at),
            // 스킬(0151 → 0158 G2 — 금·민트·흰빛, 빨강 없음). 스킬마다 다른 서명: 한가운데 번쩍임 + 빛줄 + 먼지에
            // 바닥에 남는 빛(`marks`)이 그 스킬의 꼴을 그린다.
            Event::SkillStart { id } => self.on_skill_start(id, w),
            Event::Skill { id, at, dir } => self.on_skill(id, at, dir, w),
            Event::SkillBurst { at } => self.ev_skill_burst(at, w),
            Event::Soak { at, n } => self.ev_soak(at, n, w),
            Event::EchoRing { at, radius } => self.ev_echo_ring(at, radius, w),
            Event::ThrowTurn { at } => self.ev_throw_turn(at),
            Event::ThrowBoom { at, radius } => self.ev_throw_boom(at, radius, w),
            Event::ThrowBack { at } => self.ev_throw_back(at),
            Event::FieldPulse { at, radius, kind } => self.ev_field_pulse(at, radius, kind, w),
            Event::Flurry { at } => self.ev_flurry(at, w),
            Event::Swing if w.player.skill.flurry_left > 0.0 => self.ev_flurry_swing(pp, w),
            Event::GaleThrust {
                at,
                dir,
                len,
                half,
                last,
            } => self.ev_gale_thrust(at, dir, len, half, last, w),
            Event::TempestSweep { at, radius, last } => self.ev_tempest_sweep(at, radius, last, w),
            Event::PricePulse { at, radius, last } => self.ev_price_pulse(at, radius, last, w),
            Event::Hexed { at } => self.ev_hexed(at),
            Event::HexJump { from, to } => self.ev_hex_jump(from, to),
            Event::DeathKnell { at } => self.ev_death_knell(at, w),
            Event::GrovePulse { at, radius, last } => self.ev_grove_pulse(at, radius, last, w),
            Event::Blink { from, to } => self.ev_blink(from, to, w),
            Event::AmbushStab { at, dir, last } => self.ev_ambush_stab(at, dir, last, w),
            Event::LineCut { at, dir, len, half } => self.ev_line_cut(at, dir, len, half, w),
            Event::UltStart => self.ev_ult_start(pp, w),
            Event::GreatBell { at, radius } => self.ev_great_bell(at, radius, w),
            Event::ArrowRain { from, at, radius } => self.ev_arrow_rain(from, at, radius, w),
            Event::Blizzard { at, radius } => self.ev_blizzard(at, radius, w),
            Event::Ult { at } => self.ev_ult(at, w),
            _ => return false,
        }
        true
    }

    /// 사건 묶음: 게이지·모으기의 신호, 문지기의 2·3단계, 방랑자가 맞음. 이 묶음의 사건이면 참(0251 R11 — `on_events`의 match를 나눴다).
    fn on_body(&mut self, e: &Event, w: &World) -> bool {
        let pp = w.player.pos;
        match *e {
            // 게이지가 가득: 가슴 높이의 금빛 번쩍임.
            Event::UltReady => self.star(pp + Vec3::Y * 1.2, 1.2, 0.12, GOLD),
            // 다 모였다: 손 높이의 작은 금빛 번쩍임.
            Event::Charged => self.star(pp + Vec3::Y * 1.0, 0.9, 0.09, GOLD),
            // 보스가 깨어남(2단계) · 사슬이 끊김(0208 3단계): 호박빛 불티가 사방으로 + 넓은 먼지 고리.
            Event::Rouse { at } | Event::Unchain { at } => {
                self.star(at, 2.8, 0.14, AMBER);
                self.spray(
                    at,
                    Vec3::Y,
                    1.0,
                    16,
                    8.5,
                    Shape::Streak,
                    0.09,
                    0.45,
                    6.0,
                    WHITE_HOT,
                    AMBER,
                );
                self.dust_ring(vec3(at.x, w.ground_y(at), at.z), 1.2, 12, 4.5, 0.8, DUST);
            }
            // 교체 공격(0272 1b): 발밑에서 조여드는 금빛 고리(등의 것이 손으로 온다) + 손 높이의 금빛 별 + 치는 쪽으로 뻗는 빛줄.
            // 다 모임(별 하나) · 딱 맞춰 떼기(사방의 알갱이) · 패링(차가운 빛)과 갈린다 — 고리가 **안으로** 오고 빛줄이 한쪽으로 간다.
            Event::SwapStrike { at, dir, .. } => {
                let g = vec3(at.x, w.ground_y(at), at.z);
                let hand = at + Vec3::Y * (w.player.air + 1.0);
                self.mark(g, MarkKind::Ring { r0: 1.6, r1: 0.5 }, 0.24, GOLD);
                self.star(hand + dir * 0.4, 1.1, 0.10, GOLD);
                self.spray(
                    hand,
                    dir,
                    0.35,
                    7,
                    8.0,
                    Shape::Streak,
                    0.07,
                    0.18,
                    3.0,
                    WHITE_HOT,
                    GOLD,
                );
            }
            // 분노가 가득(0277 — 대검): 머리 위로 몰아쉬는 숨(불씨 빛줄이 위로 솟는다) + 발밑에서 밖으로 번지는 주황 고리 — 그 뒤는
            // 분노가 남는 동안의 불씨(`watch`).
            Event::FuryFull { at } => {
                let g = vec3(at.x, w.ground_y(at), at.z);
                self.mark(g, MarkKind::Ring { r0: 0.4, r1: 1.5 }, 0.32, FURY);
                self.spray(
                    at + Vec3::Y * 1.1,
                    Vec3::Y,
                    0.45,
                    9,
                    5.0,
                    Shape::Streak,
                    0.08,
                    0.32,
                    -1.0,
                    WHITE_HOT,
                    FURY,
                );
            }
            // 걸어 당김(0277 — 낫): 끌려오는 적의 발밑에서 흙과 새순 부스러기가 방랑자 쪽으로 쓸린다(덩굴 자체는 렌더 `pull_vines`).
            Event::Pull { at } => {
                let g = vec3(at.x, w.ground_y(at), at.z);
                let toward = vec3(pp.x - at.x, 0.0, pp.z - at.z).normalize_or(Vec3::Z);
                self.dust_kick(g, toward, 3, 0.4);
                self.spray(
                    g + Vec3::Y * 0.2,
                    toward + Vec3::Y * 0.5,
                    0.5,
                    5,
                    3.5,
                    Shape::Dot,
                    0.08,
                    0.35,
                    9.0,
                    SPROUT,
                    VINE,
                );
            }
            // 맞음: 가슴 높이의 흰 별 + 옅은 라벤더 알갱이(빨강은 쓰지 않는다 — 화면의 붉은 테는 HUD의 몫).
            Event::PlayerHurt => {
                let c = pp + Vec3::Y * 0.9;
                self.star(c, 1.3, 0.09, WHITE_HOT);
                self.spray(
                    c,
                    Vec3::Y,
                    1.0,
                    6,
                    4.0,
                    Shape::Dot,
                    0.08,
                    0.3,
                    8.0,
                    WHITE_HOT,
                    LAVENDER,
                );
            }
            _ => return false,
        }
        true
    }

    /// 스킬의 시동(0158 G2): 몸에 걸리는 조짐 — 버팀은 조여드는 고리(버티는 동안), 되울림·잔불 흩기는 손에 모이는 빛.
    fn on_skill_start(&mut self, id: crate::skills::Id, w: &World) {
        use crate::skills::{Form, Id as Sk};
        let pp = w.player.pos;
        let g = vec3(pp.x, w.ground_y(pp), pp.z);
        let d = w.skill_eff(id);
        match id {
            Sk::Brace => {
                let Form::Brace { hold, radius, .. } = d.skill.form else {
                    return;
                };
                // 반짝이며 조여드는 고리 둘(버티는 동안 몸을 따라간다 — 끊기면 지운다).
                for (delay, col) in [(0.0, GOLD), (hold * 0.5, WHITE_HOT)] {
                    self.mark_with(
                        g,
                        MarkKind::Ring {
                            r0: radius,
                            r1: 0.9,
                        },
                        d.skill.startup + hold - delay,
                        col,
                        delay,
                        Stick::Bracing,
                    );
                }
                self.dust_ring(g, 0.4, 6, 1.6, 0.45, DUST);
            }
            Sk::Echo => self.implode(g, 1.8, 8, MINT),
            Sk::Embers => self.implode(g, 1.6, 8, AMBER),
            // 나머지는 시동의 조짐이 없다 — 판정의 서명(`on_skill`)만(와일드카드 없이, 0278 R14).
            Sk::Whirl
            | Sk::Cut
            | Sk::Gather
            | Sk::Quake
            | Sk::Throw
            | Sk::Swallow
            | Sk::Bash
            | Sk::Roots
            | Sk::Stitch
            | Sk::Ram
            | Sk::Toll
            | Sk::Swat
            | Sk::Ripple
            | Sk::Snow
            | Sk::Flare
            | Sk::Snare
            | Sk::Fuse
            | Sk::Mine
            | Sk::Skewer
            | Sk::Pole
            | Sk::Pin
            | Sk::Pommel
            | Sk::Shove
            | Sk::Rally
            | Sk::Slip
            | Sk::Sunder
            | Sk::Sear
            | Sk::Reap
            | Sk::Harvest
            | Sk::Hex
            | Sk::Blight
            | Sk::Withdraw
            | Sk::Plunge
            | Sk::Boomerang
            | Sk::Word => {}
        }
    }

    /// 스킬의 판정이 나간 순간(0151 → 0158 G2): 스킬마다의 서명.
    fn on_skill(&mut self, id: crate::skills::Id, at: Vec3, dir: Vec3, w: &World) {
        use crate::skills::Id as Sk;
        let g = vec3(at.x, w.ground_y(at), at.z);
        let d = w.skill_eff(id);
        // 스킬마다 한 갈래(와일드카드 없이 — 0278 R14), 몸은 `fx/skills.rs`의 `fx_<스킬>`(R25a): 모양은 그 스킬의 것이라 그 안에서 풀어 쓴다.
        match id {
            Sk::Whirl => self.fx_whirl(g, d),
            Sk::Cut => self.fx_cut(dir, g, d),
            Sk::Gather => self.fx_gather(g, d),
            Sk::Quake => self.fx_quake(dir, g, d),
            Sk::Brace => self.fx_brace(g, d, w),
            Sk::Echo => self.fx_echo(g, d),
            Sk::Throw => self.fx_throw(dir, g),
            Sk::Embers => self.fx_embers(g, d),
            Sk::Swallow => self.fx_swallow(dir, g, d),
            Sk::Bash => self.fx_bash(dir, g, d),
            Sk::Stitch => self.fx_stitch(dir, g, d),
            Sk::Ram => self.fx_ram(dir, g, d),
            Sk::Toll => self.fx_toll(dir, g, d),
            Sk::Hex => self.fx_hex(dir, g, d),
            Sk::Blight => self.fx_blight(g, d),
            Sk::Snow => self.fx_snow(g, d),
            Sk::Flare => self.fx_flare(dir, g, d),
            Sk::Sunder => self.fx_sunder(dir, g, d),
            Sk::Sear => self.fx_sear(dir, g, d),
            Sk::Snare => self.fx_snare(g, d),
            Sk::Fuse => self.fx_fuse(dir, g, d),
            Sk::Mine => self.fx_mine(g, d),
            Sk::Skewer => self.fx_skewer(dir, g, d),
            Sk::Pole => self.fx_pole(dir, g, d),
            Sk::Pin => self.fx_pin(dir, g, d),
            Sk::Swat => self.fx_swat(dir, g, d),
            Sk::Ripple => self.fx_ripple(g, d),
            Sk::Roots => self.fx_roots(dir, g, d),
            Sk::Pommel => self.fx_pommel(dir, g, d),
            Sk::Shove => self.fx_shove(dir, g, d),
            Sk::Slip => self.fx_slip(dir, g, d),
            Sk::Reap => self.fx_reap(dir, g, d),
            Sk::Harvest => self.fx_harvest(g, d),
            Sk::Rally => self.fx_rally(g, d),
            Sk::Withdraw => self.fx_withdraw(dir, g, d),
            Sk::Plunge => self.fx_plunge(g, d),
            Sk::Boomerang => self.fx_boomerang(dir, g, d),
            Sk::Word => self.fx_word(dir, g, d),
        }
    }

    /// 상태 변화 → 몸짓 먼지(사건이 없는 것): 회피 시작, 뛰어오름·내려앉음, 달리기, 적의 돌진, 퍼지는 고리의 앞머리(0269). 틱마다 한 번.
    pub fn watch(&mut self, w: &World, dt: f32) {
        let pl = &w.player;
        let feet = vec3(pl.pos.x, w.ground_y(pl.pos), pl.pos.z);
        let dodging = matches!(pl.state, PState::Dodge { .. });
        if dodging && !self.was_dodging {
            if let PState::Dodge { dir, .. } = pl.state {
                self.dust_kick(feet, -dir, 4, 0.5);
            }
        }
        self.was_dodging = dodging;
        let air = matches!(pl.state, PState::Jump { .. });
        if air && !self.was_air {
            self.dust_ring(feet, 0.15, 4, 1.4, 0.4, DUST);
        }
        // 내려찍기의 착지는 `Event::Plunge`가 더 크게 낸다 — 여기는 보통 착지만.
        if !air && self.was_air && matches!(pl.state, PState::Land { .. }) {
            self.dust_ring(feet, 0.2, 5, 1.8, 0.45, DUST);
        }
        self.was_air = air;
        // 달리기(회피를 쥔 채): 1.3m마다 발뒤로 한 덩이.
        if let Some(last) = self.last_pos {
            let step = vec3(pl.pos.x - last.x, 0.0, pl.pos.z - last.z);
            if pl.running && !air && step.length() < 1.0 {
                self.run_dist += step.length();
                if self.run_dist > 1.3 {
                    self.run_dist = 0.0;
                    self.dust_kick(feet, -step, 1, 0.38);
                }
            } else {
                self.run_dist = 0.0;
            }
        }
        self.last_pos = Some(pl.pos);
        // 0158 G2: 버팀이 끊기면(회피·맞음) 조여들던 고리를 지운다 — 버티는 동안에만 보이는 것.
        let bracing = matches!(
            pl.state,
            PState::Skill {
                id: crate::skills::Id::Brace,
                ..
            }
        );
        if !bracing {
            self.marks.retain(|m| !m.hold);
        }
        // 던진 무기의 꼬리: 날아가는 동안 틱마다 지나온 자리에 금빛 줄(굽은 길이 남는다).
        for b in &w.skill_blades {
            let side = vec3(b.dir.z, 0.0, -b.dir.x) * (b.age * 31.0).sin_d();
            let vel = -b.dir * 2.0 + side * 2.5;
            let part = Part {
                pos: b.pos + side * 0.35,
                vel,
                age: 0.0,
                life: 0.2,
                size: (0.1, 0.03),
                color: (GOLD, fade(WHITE_HOT)),
                gravity: 0.0,
                drag: 2.0,
                shape: Shape::Streak,
                decor: false,
            };
            self.push(part);
        }
        // 몰아치기의 기운: 이어지는 동안 몸 둘레에서 금빛 알갱이가 솟는다(0.05초마다 하나 — 적게, 오래).
        if pl.skill.flurry_left > 0.0 {
            self.aura_t += dt;
            if self.aura_t >= 0.05 {
                self.aura_t = 0.0;
                let a = self.r() * std::f32::consts::TAU;
                let rr = 0.35 + 0.35 * self.r();
                let up = 1.6 + 1.2 * self.r();
                let h = 0.2 + 0.8 * self.r();
                self.push(Part {
                    pos: feet + vec3(a.sin_d() * rr, h, a.cos_d() * rr),
                    vel: vec3(0.0, up, 0.0),
                    age: 0.0,
                    life: 0.45,
                    size: (0.09, 0.03),
                    color: (GOLD, fade(WHITE_HOT)),
                    gravity: -0.5,
                    drag: 1.0,
                    shape: Shape::Dot,
                    decor: false,
                });
            }
        }
        // 분노(0277 — 대검): 차오른 만큼 몸에서 불씨가 솟는다 — 반쯤(0.5)부터 보이고, 가득이면 0.06초마다 하나(가득 = 흰 속의 주황, 그
        // 밑은 식은 흙빛 금). 몰아치기의 금빛 알갱이(아래에서 고르게)와 달리 어깨 높이에서 위로 빠르게 · 옆으로 흔들리며 — 숨 쉬는 열기.
        let fury = w.fury();
        if fury >= FURY_MOTES_FROM {
            self.fury_t += dt;
            let every = 0.045 + 0.15 * (1.0 - fury) / (1.0 - FURY_MOTES_FROM);
            if self.fury_t >= every {
                self.fury_t = 0.0;
                let a = self.r() * std::f32::consts::TAU;
                let rr = 0.25 + 0.2 * self.r();
                let h = 0.6 + 0.6 * self.r();
                let full = fury >= 1.0;
                let side = vec3(a.cos_d(), 0.0, -a.sin_d()) * (self.rs() * 0.6);
                let up = 2.2 + 1.4 * self.r();
                self.push(Part {
                    pos: feet + vec3(a.sin_d() * rr, h, a.cos_d() * rr),
                    vel: vec3(0.0, up, 0.0) + side,
                    age: 0.0,
                    life: 0.45,
                    size: (if full { 0.17 } else { 0.13 }, 0.04),
                    color: if full {
                        (FURY_HOT, fade(FURY))
                    } else {
                        (FURY, fade(FURY_COOL))
                    },
                    gravity: -1.0,
                    drag: 1.2,
                    shape: Shape::Dot,
                    decor: false,
                });
            }
        } else {
            self.fury_t = 0.0;
        }
        // 적: 돌진하는 동안 발밑 먼지(0.09초마다). 공격이 풀린 순간의 타격은 `Event::EnemyStrike`(0269)가 낸다.
        self.enemy_was.resize(w.enemies.len(), (false, 0.0));
        for (i, en) in w.enemies.iter().enumerate() {
            let (was_charge, mut since) = self.enemy_was[i];
            let charge = matches!(en.state, EState::Charge { .. });
            let g = vec3(en.pos.x, w.ground_y(en.pos), en.pos.z);
            if charge {
                since += dt;
                if !was_charge || since > 0.09 {
                    since = 0.0;
                    self.dust_kick(g, -en.facing_dir(), 2, 0.3 + 0.35 * en.radius);
                }
            }
            self.enemy_was[i] = (charge, since);
            self.wave_debris(en, w);
        }
    }

    /// 퍼지는 고리(뿌리 파동·울림 고리·떨어진 종의 고리 — 0269)의 앞머리를 따라 튀는 돌 부스러기: 고리가 "땅을 뒤집으며 오는 것"으로
    /// 읽히게(단단한 가시는 렌더). 틱마다 앞머리 둘레 `WAVE_CHIPS`개 — 꾸밈이라 많으면 건너뛴다(가시가 몫을 한다).
    fn wave_debris(&mut self, en: &crate::enemy::Enemy, w: &World) {
        if !en.alive() || !matches!(en.atk, Atk::Wave | Atk::Toll | Atk::Hurl) {
            return;
        }
        let c = en.wave_center();
        let cg = vec3(c.x, w.ground_y(c), c.z);
        for front in en.wave_fronts() {
            for _ in 0..WAVE_CHIPS {
                let a = self.r() * std::f32::consts::TAU;
                let d = vec3(a.sin_d(), 0.0, a.cos_d());
                let at = cg + d * front;
                let at = vec3(at.x, w.ground_y(at) + 0.08, at.z);
                let size = 0.09 + 0.07 * self.r();
                let rock = self.r() < 0.6;
                let vel = d * (1.5 + 2.0 * self.r()) + Vec3::Y * (2.5 + 2.5 * self.r());
                let life = 0.35 + 0.15 * self.r();
                self.push(Part {
                    pos: at,
                    vel,
                    age: 0.0,
                    life,
                    size: (size, size * 0.6),
                    color: if rock {
                        (STONE, STONE)
                    } else {
                        (QUAKE, fade(DUST))
                    },
                    gravity: 15.0,
                    drag: 0.8,
                    shape: Shape::Dot,
                    decor: true,
                });
            }
        }
    }

    /// 입자를 `dt`만큼 움직이고 다한 것을 치운다.
    pub fn tick(&mut self, dt: f32) {
        for p in &mut self.parts {
            p.age += dt;
            if p.shape == Shape::Splat {
                continue;
            }
            p.vel.y -= p.gravity * dt;
            p.vel *= (1.0 - p.drag * dt).max(0.0);
            p.pos += p.vel * dt;
        }
        self.parts.retain(|p| p.age < p.life);
        // 늦게 나올 입자·남는 빛(0158 G2).
        let mut due = Vec::new();
        self.later.retain_mut(|(left, p)| {
            *left -= dt;
            if *left <= 0.0 {
                due.push(*p);
                false
            } else {
                true
            }
        });
        for p in due {
            self.push(p);
        }
        for m in &mut self.marks {
            if m.delay > 0.0 {
                m.delay -= dt;
            } else {
                m.age += dt;
            }
        }
        self.marks.retain(|m| m.age < m.life);
    }

    /// 판이 바뀔 때(다른 스테이지·시나리오): 남은 입자와 지켜보던 상태를 비운다.
    pub fn clear(&mut self) {
        let seed = self.seed;
        *self = Fx {
            seed,
            ..Default::default()
        };
    }
}

/// 화로에서 오르는 불티(상태 없음 — KB "환경 입자"): 화로 `k`의 `i`번째 불티가 시각 `time`에 있는 자리(화로 위 높이 0 기준)와
/// 진하기(0..1). 주기마다 그릇에서 떠올라 흔들리며 식는다.
pub fn brazier_ember(k: usize, i: usize, time: f32) -> (Vec3, f32) {
    let h = |n: u32| {
        let mut x = (k as u32).wrapping_mul(73_856_093)
            ^ (i as u32).wrapping_mul(19_349_663)
            ^ n.wrapping_mul(83_492_791);
        x ^= x >> 13;
        x = x.wrapping_mul(0x5bd1_e995);
        x ^= x >> 15;
        (x & 0xffff) as f32 / 65535.0
    };
    let period = 1.5 + 1.3 * h(1);
    let ph = ((time + h(2) * period) / period).fract();
    let a = h(3) * std::f32::consts::TAU;
    let r = 0.05 + 0.16 * h(4);
    let sway = (time * (1.5 + 2.0 * h(5)) + h(6) * 6.0).sin_d() * 0.10 * ph;
    let pos = vec3(
        a.sin_d() * r + sway,
        0.15 + (0.9 + 0.7 * h(7)) * ph,
        a.cos_d() * r,
    );
    // 떠오르자마자 밝고, 위로 갈수록 식는다(끝 30%에서 사라짐).
    let k = (ph / 0.1).min(1.0) * ((1.0 - ph) / 0.3).min(1.0);
    (pos, k)
}
/// 화로 하나의 불티 수.
pub const BRAZIER_EMBERS: usize = 5;

mod ail;
mod skill_events;
mod skills;
mod strike;
#[cfg(test)]
mod traces;

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> World {
        World::new("empty", 16.0)
    }

    /// 스킬·큰 기술이 내는 모든 사건(0158 G2) — 스킬 여덟의 시동과 판정, 남긴 것들의 사건, 큰 기술 셋.
    /// 적의 공격이 풀린 순간(0269) — 꼴마다 하나, 잡몹·보스 둘 다.
    fn enemy_strikes(at: Vec3) -> Vec<Event> {
        let dir = Vec3::Z;
        let blows = [
            Blow::Around { r: 1.6 },
            Blow::Cone { r: 3.0, arc: 110.0 },
            Blow::Spot { r: 1.4 },
            Blow::Ram,
            Blow::Quake { r: 9.0 },
            Blow::Line { len: 11.0 },
            Blow::Hook { len: 9.0 },
            Blow::Volley,
        ];
        blows
            .iter()
            .flat_map(|&blow| {
                [false, true].map(|boss| Event::EnemyStrike {
                    at,
                    dir,
                    blow,
                    boss,
                })
            })
            .collect()
    }

    fn skill_events() -> Vec<Event> {
        let (at, dir) = (vec3(0.0, 0.0, 1.0), Vec3::Z);
        let mut ev = Vec::new();
        for id in crate::skills::IDS {
            ev.push(Event::SkillStart { id });
            ev.push(Event::Skill { id, at, dir });
        }
        ev.extend([
            Event::SkillBurst { at },
            Event::Soak { at, n: 2 },
            Event::EchoRing { at, radius: 4.0 },
            Event::ThrowTurn { at },
            Event::ThrowBoom { at, radius: 2.5 },
            Event::ThrowBack { at },
            Event::FieldPulse {
                at,
                radius: 4.0,
                kind: crate::status::Kind::Ember,
            },
            Event::Ail {
                at,
                kind: crate::status::Kind::Ember,
            },
            Event::Ail {
                at,
                kind: crate::status::Kind::Root,
            },
            Event::Ail {
                at,
                kind: crate::status::Kind::Rift,
            },
            // 0270 B: 새 타입 넷의 걸림과 그 일(샘·깨짐·돌아옴).
            Event::Ail {
                at,
                kind: crate::status::Kind::Shock,
            },
            Event::Ail {
                at,
                kind: crate::status::Kind::Sharp,
            },
            Event::Ail {
                at,
                kind: crate::status::Kind::Frost,
            },
            Event::Ail {
                at,
                kind: crate::status::Kind::Drain,
            },
            Event::Bleed { at, dir },
            Event::Shatter { at },
            Event::Drain { from: at },
            Event::Burn { at },
            Event::Toll { at, radius: 4.0 },
            Event::AilBurst { at, radius: 2.5 },
            Event::Flurry { at },
            Event::LineCut {
                at,
                dir,
                len: 12.0,
                half: 1.2,
            },
            Event::UltStart,
            Event::Ult { at },
            // 0270 C3: 큰 기술 둘.
            Event::GreatBell { at, radius: 8.0 },
            Event::Blizzard { at, radius: 6.0 },
            // 0272: 활 판의 큰 기술.
            Event::ArrowRain {
                from: Vec3::ZERO,
                at,
                radius: 4.5,
            },
            // 0272 2단계: 창 판의 큰 기술.
            Event::GaleThrust {
                at,
                dir,
                len: 5.5,
                half: 0.8,
                last: true,
            },
            // 0274 ③: 쌍단검 판의 큰 기술.
            Event::Blink {
                from: at - dir * 4.0,
                to: at,
            },
            Event::AmbushStab {
                at,
                dir,
                last: false,
            },
            // 0274 ⑥: 완드 판의 큰 기술 · 저주의 표 · 죽음의 울림.
            Event::PricePulse {
                at,
                radius: 5.0,
                last: false,
            },
            Event::PricePulse {
                at,
                radius: 5.0,
                last: true,
            },
            Event::Hexed { at },
            Event::HexJump {
                from: at,
                to: at + dir * 3.0,
            },
            Event::DeathKnell { at },
            // 0274 ⑤: 낫 판의 큰 기술.
            Event::GrovePulse {
                at,
                radius: 2.2,
                last: false,
            },
            Event::GrovePulse {
                at,
                radius: 4.6,
                last: true,
            },
            // 0274 ④: 대검 판의 큰 기술.
            Event::TempestSweep {
                at,
                radius: 3.0,
                last: false,
            },
            Event::TempestSweep {
                at,
                radius: 3.0,
                last: true,
            },
            Event::AmbushStab {
                at,
                dir,
                last: true,
            },
        ]);
        ev
    }

    /// 0158 G2: 스킬마다 서명이 다르다 — 판정이 나간 순간에 남는 빛(`marks`)의 꼴·수가 스킬 여덟에서 서로 다르고,
    /// 타격인 스킬은 한가운데 번쩍임(0.15초 안)을 낸다. 큰 기술 셋도 저마다의 꼴을 남긴다.
    #[test]
    fn every_skill_leaves_its_own_signature() {
        let w = world();
        let (at, dir) = (vec3(0.0, 0.0, 1.0), Vec3::Z);
        let sign = |e: Event| {
            let mut fx = Fx::default();
            fx.on_events(&[e], &w);
            for p in fx.parts.iter().filter(|p| p.shape == Shape::Star) {
                assert!(p.life <= 0.15, "{e:?}: the flash lingers {}", p.life);
            }
            let kinds: Vec<String> = fx
                .marks
                .iter()
                .map(|m| format!("{:?}", std::mem::discriminant(&m.kind)))
                .collect();
            let stars = fx.parts.iter().filter(|p| p.shape == Shape::Star).count();
            assert!(!fx.marks.is_empty() || !fx.parts.is_empty(), "{e:?}");
            (
                kinds,
                stars,
                fx.parts.len(),
                fx.marks.first().map(|m| m.color),
            )
        };
        let mut seen = Vec::new();
        for id in crate::skills::IDS {
            let s = sign(Event::Skill { id, at, dir });
            assert!(s.1 >= 1 || !s.0.is_empty(), "{id:?}: a flash or a mark");
            assert!(!seen.contains(&s), "{id:?} looks like another skill");
            seen.push(s);
        }
        // 큰 기술: 쟁반 뒤집기 = 땅의 금, 한 줄기 = 빛의 면, 몰아치기 = 고리 — 서로 다른 꼴.
        let has = |e: Event, f: fn(&MarkKind) -> bool| {
            let mut fx = Fx::default();
            fx.on_events(&[e], &w);
            fx.marks.iter().any(|m| f(&m.kind))
        };
        assert!(has(Event::Ult { at }, |k| matches!(
            k,
            MarkKind::Cracks { .. }
        )));
        let line = Event::LineCut {
            at,
            dir,
            len: 12.0,
            half: 1.2,
        };
        assert!(has(line, |k| matches!(k, MarkKind::Slash { .. })));
        assert!(has(Event::Flurry { at }, |k| matches!(
            k,
            MarkKind::Ring { .. }
        )));
        // 한 줄기의 불똥은 늦게 — 길을 따라 차례로 터진다.
        let mut fx = Fx::default();
        fx.on_events(&[line], &w);
        let now = fx.parts.len();
        assert!(!fx.later.is_empty());
        for _ in 0..20 {
            fx.tick(crate::tuning::TICK_DT);
        }
        assert!(fx.later.is_empty(), "all out within a third of a second");
        assert!(fx.parts.len() > now / 2, "the late sparks came out");
    }

    /// 남는 빛은 다 사라지고(가장 긴 것도 1.5초 안), 수는 상한을 넘지 않는다. 버티는 동안의 고리는 버팀이 끊기면 지워진다.
    #[test]
    fn marks_fade_out_are_capped_and_the_brace_ring_goes_when_the_hold_ends() {
        let mut w = world();
        let mut fx = Fx::default();
        fx.on_events(&skill_events(), &w);
        assert!(!fx.marks.is_empty());
        for _ in 0..120 {
            fx.on_events(&[Event::Ult { at: Vec3::ZERO }], &w);
        }
        assert!(fx.marks.len() <= MARK_CAP);
        for _ in 0..100 {
            fx.tick(crate::tuning::TICK_DT);
        }
        assert!(fx.marks.is_empty(), "{} left", fx.marks.len());
        // 버팀: 시동에 조여드는 고리가 걸리고, 버티는 동안은 남는다 — 상태가 풀리면 다음 틱에 지워진다.
        let id = crate::skills::Id::Brace;
        w.player.state = PState::Skill {
            id,
            t: 0.0,
            dir: Vec3::Z,
            swing: 1,
        };
        let mut fx = Fx::default();
        fx.on_events(&[Event::SkillStart { id }], &w);
        fx.watch(&w, crate::tuning::TICK_DT);
        assert!(fx.marks.iter().any(|m| m.hold && m.follow));
        w.player.state = PState::Idle;
        fx.watch(&w, crate::tuning::TICK_DT);
        assert!(fx.marks.iter().all(|m| !m.hold));
    }

    /// 잔불 흩기의 불티(상태 없음): 원판 안에서만 놀고(반지름 1·높이 2m 안) 주기 끝에서는 꺼져 있다.
    #[test]
    fn field_motes_stay_inside_the_disc_and_fade_out() {
        for i in 0..FIELD_MOTES_MAX {
            let mut seen = 0.0f32;
            for s in 0..300 {
                let (p, a) = field_mote(i, s as f32 * 0.02);
                assert!((0.0..=1.0).contains(&a));
                assert!(
                    p.y > 0.0 && p.y < 2.0 && vec2(p.x, p.z).length() < 1.05,
                    "{p}"
                );
                seen = seen.max(a);
            }
            assert!(seen > 0.9);
        }
        assert!(FIELD_MOTES <= FIELD_MOTES_MAX);
    }

    /// 타격은 겹이다: 한가운데 번쩍임(아주 짧다)이 꼭 있고, 치명타·강타·패링은 평타보다 크다.
    #[test]
    fn every_hit_event_flashes_and_bigger_hits_flash_bigger() {
        let w = world();
        let at = vec3(0.0, 1.0, 2.0);
        let star = |e: Event| {
            let mut fx = Fx::default();
            fx.on_events(&[e], &w);
            let s = fx
                .parts
                .iter()
                .find(|p| p.shape == Shape::Star)
                .copied()
                .unwrap_or_else(|| panic!("{e:?}: no flash"));
            assert!(s.life <= 0.15, "{e:?}: the flash lingers {}", s.life);
            s.size.0
        };
        let hit = star(Event::Hit { at });
        for e in [
            Event::Crit { at },
            Event::Smash { at },
            Event::Parry { at },
            Event::Break { at },
            Event::Boom { at },
        ] {
            assert!(star(e) > hit, "{e:?}");
        }
        star(Event::Deflect { at });
        star(Event::PlayerHurt);
    }

    /// 빨강은 위험 예고 전용(0037) — 어떤 사건의 입자도 붉지 않다. 패링은 차갑고(파랑 ≥ 빨강) 평타는 따뜻하다.
    #[test]
    fn no_effect_is_red_and_a_parry_reads_cool() {
        let w = world();
        let at = vec3(0.0, 1.0, 2.0);
        let all = [
            Event::Hit { at },
            Event::Crit { at },
            Event::Smash { at },
            Event::Parry { at },
            Event::Deflect { at },
            Event::Guard { at },
            Event::GuardBreak { at },
            Event::Break { at },
            Event::EnemyDown { at },
            Event::Plunge { at },
            Event::Aftershock { at },
            Event::Slam { at },
            Event::Boom { at },
            Event::Gulp { at },
            Event::Cast {
                at,
                look: crate::weapons::Look::Magic,
            },
            Event::Cast {
                at,
                look: crate::weapons::Look::Axe,
            },
            Event::Cast {
                at,
                look: crate::weapons::Look::Light,
            },
            Event::Cast {
                at,
                look: crate::weapons::Look::Quake,
            },
            Event::Cast {
                at,
                look: crate::weapons::Look::Javelin,
            },
            Event::Cast {
                at,
                look: crate::weapons::Look::Stone,
            },
            Event::Smite { at, radius: 1.4 },
            Event::Cast {
                at,
                look: crate::weapons::Look::Seed,
            },
            Event::Sprout { at, radius: 1.6 },
            Event::Cast {
                at,
                look: crate::weapons::Look::Curse,
            },
            Event::JavelinBack { at },
            Event::Prayer { at, radius: 3.0 },
            Event::SnareSet { at },
            Event::SnareSnap { at },
            Event::ShieldBounce { at },
            Event::Rush {
                at,
                dir: Vec3::Z,
                power: 0.6,
            },
            Event::Perfect { at },
            Event::Charged,
            Event::Rouse { at },
            Event::PlayerHurt,
            Event::SwapStrike {
                at,
                dir: Vec3::Z,
                to: crate::weapons::Id::Hammer,
            },
            Event::Sneak { at },
            Event::FuryFull { at },
            Event::Pull { at },
        ];
        let red = |c: Color| c.r > 0.75 && c.g < 0.45 && c.b < 0.45;
        for e in all
            .into_iter()
            .chain(skill_events())
            .chain(enemy_strikes(at))
        {
            let mut fx = Fx::default();
            fx.on_events(&[e], &w);
            // 늦게 나오는 입자까지 본다.
            let mut seen = fx.parts.clone();
            seen.extend(fx.later.iter().map(|(_, p)| *p));
            // 시동의 조짐은 일부 스킬에만 있다(나머지는 판정의 순간에 보인다).
            assert!(
                !seen.is_empty() || !fx.marks.is_empty() || matches!(e, Event::SkillStart { .. }),
                "{e:?}: nothing to see"
            );
            for p in &seen {
                for c in [p.color.0, p.color.1] {
                    assert!(!red(c), "{e:?}: red {c:?}");
                }
            }
            for m in &fx.marks {
                assert!(!red(m.color), "{e:?}: red mark {:?}", m.color);
            }
        }
        let mut fx = Fx::default();
        fx.on_events(&[Event::Parry { at }], &w);
        let s = fx.parts.iter().find(|p| p.shape == Shape::Star).unwrap();
        assert!(s.color.0.b >= s.color.0.r);
    }

    /// 입자는 다 사라진다(가장 긴 것도 1.5초 안), 같은 사건 = 같은 그림(난수 없음).
    #[test]
    fn particles_die_and_the_same_events_draw_the_same_picture() {
        let w = world();
        let ev = [
            Event::Boom { at: Vec3::ZERO },
            Event::EnemyDown { at: Vec3::ZERO },
            Event::Gulp { at: Vec3::Y },
        ];
        let run = || {
            let mut fx = Fx::default();
            fx.on_events(&ev, &w);
            for _ in 0..10 {
                fx.tick(crate::tuning::TICK_DT);
            }
            fx.parts.iter().map(|p| p.pos).collect::<Vec<_>>()
        };
        assert_eq!(run(), run());
        let mut fx = Fx::default();
        fx.on_events(&ev, &w);
        for _ in 0..90 {
            fx.tick(crate::tuning::TICK_DT);
        }
        assert!(fx.parts.is_empty(), "{} left", fx.parts.len());
    }

    /// 꾸밈(먼지)은 많으면 건너뛰지만 타격은 건너뛰지 않는다 — 떼싸움 한가운데서도 맞힌 번쩍임은 뜬다. 그래도 끝없이 늘지는 않는다.
    #[test]
    fn dust_is_capped_but_hits_always_show() {
        let w = world();
        let mut fx = Fx::default();
        for _ in 0..80 {
            fx.on_events(&[Event::Plunge { at: Vec3::ZERO }], &w);
        }
        let dust = fx.parts.iter().filter(|p| p.decor).count();
        assert!(dust <= DECOR_CAP, "{dust}");
        fx.on_events(&[Event::Hit { at: Vec3::Y }], &w);
        assert!(fx.parts.last().is_some_and(|p| p.shape == Shape::Star));
        for _ in 0..200 {
            fx.on_events(&[Event::Crit { at: Vec3::Y }], &w);
        }
        assert!(fx.parts.len() <= HARD_CAP);
    }

    /// 회피를 시작한 틱에 발밑 먼지가 반대쪽으로 밀리고, 가만히 서 있으면 아무것도 안 난다(목적이 있을 때만).
    #[test]
    fn a_dodge_kicks_dust_backwards_and_standing_still_is_clean() {
        let mut w = world();
        let mut fx = Fx::default();
        for _ in 0..30 {
            fx.watch(&w, crate::tuning::TICK_DT);
        }
        assert!(fx.parts.is_empty(), "idle dust");
        w.player.state = PState::Dodge {
            t: 0.0,
            dir: Vec3::Z,
        };
        fx.watch(&w, crate::tuning::TICK_DT);
        assert!(!fx.parts.is_empty());
        assert!(fx.parts.iter().all(|p| p.decor && p.shape == Shape::Puff));
        assert!(
            fx.parts.iter().map(|p| p.vel.z).sum::<f32>() < 0.0,
            "dust trails behind"
        );
        let n = fx.parts.len();
        fx.watch(&w, crate::tuning::TICK_DT);
        assert_eq!(fx.parts.len(), n, "once per dodge");
    }

    /// 화로 불티: 그릇 위에서만 놀고(반지름 0.4m·높이 2m 안) 주기 끝에서는 꺼져 있다 — 튀어 사라지지 않는다.
    #[test]
    fn brazier_embers_rise_above_the_bowl_and_fade_out() {
        for k in 0..6 {
            for i in 0..BRAZIER_EMBERS {
                let mut seen = 0.0f32;
                for s in 0..400 {
                    let (p, a) = brazier_ember(k, i, s as f32 * 0.02);
                    assert!((0.0..=1.0).contains(&a));
                    assert!(
                        p.y > 0.0 && p.y < 2.0 && vec2(p.x, p.z).length() < 0.4,
                        "{p}"
                    );
                    if p.y > 1.4 {
                        assert!(a < 0.75);
                    }
                    seen = seen.max(a);
                }
                assert!(seen > 0.9);
            }
        }
    }

    /// 진액(0182): 맞은 쪽(플레이어 → 적)으로 튀고, 떨어져 바닥 얼룩이 되고, 얼룩은 제자리에서 2초 안에 사라진다.
    /// 강타는 평타보다 많이 흘리고, 쓰러짐은 사방으로 터진다.
    #[test]
    fn sap_sprays_away_from_the_player_and_stains_the_floor() {
        let w = world();
        let pp = w.player.pos;
        let at = pp + vec3(0.0, 1.0, 2.0);
        let drops = |fx: &Fx| fx.parts.iter().filter(|p| p.shape == Shape::Drop).count();
        let mut fx = Fx::default();
        fx.on_events(&[Event::Hit { at }], &w);
        let hit = drops(&fx);
        assert!(hit >= 5, "{hit}");
        for p in fx.parts.iter().filter(|p| p.shape == Shape::Drop) {
            assert!(
                p.vel.z > 0.0,
                "a drop flew back at the wanderer: {:?}",
                p.vel
            );
            assert!(
                p.vel.x.abs() > 0.3 * p.vel.z,
                "drops fly out to the side, not hidden behind the body"
            );
            assert!(p.vel.y > 0.0, "drops leap before they fall");
        }
        let mut smash = Fx::default();
        smash.on_events(&[Event::Hit { at }, Event::Smash { at }], &w);
        assert!(drops(&smash) > hit);
        // 떨어져 얼룩이 된다 — 바닥 높이 그대로, 움직이지 않는다.
        let floor = pp.y;
        let mut landed = 0.0;
        for i in 0..60 {
            fx.tick(crate::tuning::TICK_DT);
            fx.settle(|_| floor);
            if drops(&fx) == 0 && landed == 0.0 {
                landed = i as f32 * crate::tuning::TICK_DT;
            }
        }
        assert!(
            landed > 0.15,
            "the drops hung in the air a moment: {landed}"
        );
        let stains: Vec<Part> = fx
            .parts
            .iter()
            .filter(|p| p.shape == Shape::Splat)
            .copied()
            .collect();
        assert!(stains.len() >= hit, "every drop left a stain");
        for s in &stains {
            assert!((s.pos.y - floor).abs() < 0.05);
            assert!(s.pos.z > at.z - 0.2, "stains lie on the far side");
        }
        let still = stains[0].pos;
        fx.tick(0.1);
        assert!(fx.parts.iter().any(|p| p.pos == still));
        for _ in 0..120 {
            fx.tick(crate::tuning::TICK_DT);
            fx.settle(|_| floor);
        }
        assert!(fx.parts.is_empty(), "stains fade within two seconds");
        // 쓰러짐: 사방(앞뒤 둘 다)으로.
        let mut down = Fx::default();
        down.on_events(&[Event::EnemyDown { at }], &w);
        let zs: Vec<f32> = down
            .parts
            .iter()
            .filter(|p| p.shape == Shape::Drop)
            .map(|p| p.vel.z)
            .collect();
        assert!(
            zs.iter().any(|z| *z > 0.0) && zs.iter().any(|z| *z < 0.0),
            "{zs:?}"
        );
    }

    /// 0278 R12: 일부러 이펙트가 없는 사건(갈래의 이름) — 소리 · HUD · 카메라만 쓰거나 다른 사건이 그림을 맡는다.
    const NO_FX: &[&str] = &[
        "Swing",
        "LockOn",
        "LockOff",
        "LockLost",
        "Turn",
        "CartGo",
        "CartWarn",
        "Release",
        "Ember",
        "WeaponSwap",
        "FlaskDenied",
        "CastDenied",
        "StaminaDenied",
        "Beat",
        "SkillDenied",
    ];

    /// 0278 R12 사건 덮개(이펙트): 모든 사건은 묶음 하나가 맡거나 `NO_FX`에 있다 — 그리고 `NO_FX`의 것은 정말 어느 묶음에도 없다.
    #[test]
    fn every_event_has_one_fx_group_or_none_on_purpose() {
        let w = world();
        let mut bad = Vec::new();
        for e in Event::samples() {
            let name = e.variant();
            let mut fx = Fx::default();
            let n = [
                fx.on_strike(&e, &w),
                fx.on_ail(&e, &w),
                fx.on_skill_event(&e, &w),
                fx.on_body(&e, &w),
            ]
            .iter()
            .filter(|b| **b)
            .count();
            let none = NO_FX.contains(&name.as_str());
            if (n == 1) == none || n > 1 {
                bad.push(format!("{name}: {n} groups, listed no-fx {none}"));
            }
        }
        assert!(
            bad.is_empty(),
            "give these one fx group or list them in NO_FX:\n{}",
            bad.join("\n")
        );
        for n in NO_FX {
            assert!(
                Event::samples().iter().any(|e| e.variant() == *n),
                "{n} is no event"
            );
        }
    }

    /// 0251 R11: `on_events`를 묶음 넷으로 나눴다 — 어느 사건도 두 묶음에 들지 않고, 그림을 내는 사건은 한 묶음이 맡는다.
    #[test]
    fn every_event_belongs_to_at_most_one_group() {
        let w = world();
        let at = vec3(1.0, 0.0, -2.0);
        let mut events = skill_events();
        events.extend([
            Event::Hit { at },
            Event::Crit { at },
            Event::Smash { at },
            Event::Parry { at },
            Event::Deflect { at },
            Event::Guard { at },
            Event::GuardBreak { at },
            Event::Break { at },
            Event::LoadBreak { at },
            Event::EnemyDown { at },
            Event::Plunge { at },
            Event::Aftershock { at },
            Event::Slam { at },
            Event::Boom { at },
            Event::Gulp { at },
            Event::Interrupt { at },
            Event::Ail {
                at,
                kind: crate::status::Kind::Ember,
            },
            Event::Burn { at },
            Event::Toll { at, radius: 2.0 },
            Event::AilBurst { at, radius: 2.0 },
            Event::RootSpike { at },
            Event::Mend { at },
            Event::Cast {
                at,
                look: crate::weapons::Look::Magic,
            },
            Event::Cast {
                at,
                look: crate::weapons::Look::Axe,
            },
            Event::Cast {
                at,
                look: crate::weapons::Look::Light,
            },
            Event::Cast {
                at,
                look: crate::weapons::Look::Quake,
            },
            Event::Cast {
                at,
                look: crate::weapons::Look::Javelin,
            },
            Event::Cast {
                at,
                look: crate::weapons::Look::Stone,
            },
            Event::Smite { at, radius: 1.4 },
            Event::Cast {
                at,
                look: crate::weapons::Look::Seed,
            },
            Event::Sprout { at, radius: 1.6 },
            Event::Cast {
                at,
                look: crate::weapons::Look::Curse,
            },
            Event::Sneak { at },
            Event::FuryFull { at },
            Event::Pull { at },
            Event::JavelinBack { at },
            Event::Prayer { at, radius: 3.0 },
            Event::SnareSet { at },
            Event::SnareSnap { at },
            Event::ShieldBounce { at },
            Event::Rush {
                at,
                dir: Vec3::Z,
                power: 0.6,
            },
            Event::Perfect { at },
            Event::UltStart,
            Event::Ult { at },
            Event::UltReady,
            Event::Charged,
            Event::Rouse { at },
            Event::Unchain { at },
            Event::PlayerHurt,
        ]);
        events.extend(enemy_strikes(at));
        for e in &events {
            let mut fx = Fx::default();
            let n = [
                fx.on_strike(e, &w),
                fx.on_ail(e, &w),
                fx.on_skill_event(e, &w),
                fx.on_body(e, &w),
            ]
            .iter()
            .filter(|b| **b)
            .count();
            assert!(n <= 1, "{e:?} is drawn by {n} groups");
            let mut one = Fx::default();
            one.on_events(std::slice::from_ref(e), &w);
            assert_eq!(
                one.parts.len(),
                fx.parts.len(),
                "{e:?}: the dispatcher draws what its group draws"
            );
        }
    }

    /// 0269(사장 "공격에 대한 이펙트는 … 비주얼적으로 확실히 표시되어야 함"): 적의 공격이 풀린 순간은 꼴마다 한가운데 번쩍임(짧게)과
    /// 건너뛰지 않는 입자를 낸다 — 땅을 치는 꼴은 남는 꼴(충격파·고리·금)도. 보스의 별은 잡몹보다 크다. 빛은 방랑자 스킬의 금·민트가 아니다.
    #[test]
    fn every_enemy_strike_flashes_and_shakes_the_ground_and_bosses_hit_bigger() {
        let w = world();
        let at = vec3(0.0, 0.0, 2.0);
        let mut mob_star = Vec::new();
        for e in enemy_strikes(at) {
            let Event::EnemyStrike { blow, boss, .. } = e else {
                unreachable!()
            };
            let mut fx = Fx::default();
            fx.on_events(&[e], &w);
            let star = fx
                .parts
                .iter()
                .find(|p| p.shape == Shape::Star)
                .unwrap_or_else(|| panic!("{e:?}: no flash"));
            assert!(star.life <= 0.15, "{e:?}: the flash lingers");
            let hard = fx.parts.iter().filter(|p| !p.decor).count();
            assert!(hard >= 5, "{e:?}: only {hard} parts that always show");
            let grounded = !matches!(blow, Blow::Hook { .. } | Blow::Volley);
            assert!(
                !grounded || !fx.marks.is_empty(),
                "{e:?}: the ground does not show the blow"
            );
            for m in &fx.marks {
                assert!(
                    m.color != GOLD && m.color != MINT,
                    "{e:?}: a skill's colour"
                );
            }
            if boss {
                let (_, mob) = mob_star.pop().expect("mob first");
                assert!(star.size.0 > mob, "{e:?}: a boss hits bigger");
            } else {
                mob_star.push((blow, star.size.0));
            }
        }
    }

    /// 0269: 퍼지는 고리의 앞머리를 따라 돌 부스러기가 튄다(늘 — 예고 설정은 렌더의 일이라 여기 없다), 고리가 없으면 조용하다.
    #[test]
    fn a_spreading_ring_kicks_up_stones_along_its_front() {
        let mut w = world();
        w.spawn_boss(vec3(0.0, 0.0, -6.0));
        let i = w.enemies.len() - 1;
        let mut fx = Fx::default();
        fx.watch(&w, 1.0 / 60.0);
        let quiet = fx.parts.len();
        w.enemies[i].atk = Atk::Wave;
        w.enemies[i].state = EState::Wave { t: 0.3 };
        let fronts = w.enemies[i].wave_fronts();
        assert_eq!(fronts.len(), 1);
        fx.watch(&w, 1.0 / 60.0);
        let c = w.enemies[i].pos;
        let new: Vec<_> = fx.parts[quiet..].to_vec();
        assert_eq!(new.len(), WAVE_CHIPS);
        for p in new {
            let r = vec2(p.pos.x - c.x, p.pos.z - c.z).length();
            assert!(
                (r - fronts[0]).abs() < 0.05,
                "a chip at {r} m, the front at {}",
                fronts[0]
            );
        }
    }

    /// 0277: 급소는 평타와 다른 번쩍임이다 — 별의 빛이 서늘한 은빛(평타는 따뜻한 흰빛)이고, 직각으로 엇갈리는 빛줄 넷이 함께 나간다.
    /// 같은 틱의 평타 번쩍임 위에 얹혀도(사건 둘) 갈려 읽힌다.
    #[test]
    fn a_sneak_hit_glints_cold_and_crossed_apart_from_a_plain_hit() {
        let w = world();
        let at = vec3(0.0, 1.0, 2.0);
        let look = |e: Event| {
            let mut fx = Fx::default();
            fx.on_events(&[e], &w);
            fx.parts
        };
        let hit = look(Event::Hit { at });
        let sneak = look(Event::Sneak { at });
        let star = |ps: &[Part]| *ps.iter().find(|p| p.shape == Shape::Star).unwrap();
        let (h, k) = (star(&hit), star(&sneak));
        assert!(
            k.color.0.b > k.color.0.r && h.color.0.r > h.color.0.b,
            "cold glint vs warm flash"
        );
        assert!(k.size.0 > h.size.0 && k.life <= 0.15);
        let streaks: Vec<&Part> = sneak.iter().filter(|p| p.shape == Shape::Streak).collect();
        assert!(
            sneak.iter().any(|p| p.shape == Shape::Dot && p.life > 0.3),
            "a lingering glitter"
        );
        assert_eq!(streaks.len(), 4, "an X of four streaks");
        // 엇갈림: 위로 둘 · 아래로 둘, 맞은 쪽(z)으로는 가지 않는다.
        assert_eq!(streaks.iter().filter(|p| p.vel.y > 0.0).count(), 2);
        assert!(streaks.iter().all(|p| p.vel.z.abs() < 1e-3));
    }

    /// 0277: 분노의 불씨는 분노만큼 — 분노가 없는 종류(검) · 둘레가 빈 대검에는 없고, 넷에 에워싸인 대검은 1초에 열 남짓이 솟는다.
    #[test]
    fn fury_embers_rise_only_while_the_greatsword_is_furious() {
        let count = |weapon: crate::weapons::Id, foes: usize| {
            let mut w = world();
            w.set_weapon(weapon);
            for i in 0..foes {
                let a = i as f32 * std::f32::consts::FRAC_PI_2;
                let at = w.player.pos + vec3(a.sin_d(), 0.0, a.cos_d()) * 1.5;
                let k = w.spawn(crate::species::Kind::Husk, at);
                w.enemies[k].state = EState::Recover { t: -1000.0 };
            }
            let mut fx = Fx::default();
            let mut n = 0;
            for _ in 0..60 {
                fx.watch(&w, crate::tuning::TICK_DT);
                n += fx
                    .parts
                    .iter()
                    .filter(|p| p.age == 0.0 && p.shape == Shape::Dot)
                    .count();
                fx.tick(crate::tuning::TICK_DT);
            }
            n
        };
        use crate::weapons::Id;
        assert_eq!(count(Id::Sword, 4), 0, "no fury, no embers");
        assert_eq!(count(Id::Greatsword, 0), 0, "alone, calm");
        let full = count(Id::Greatsword, 4);
        assert!((10..=25).contains(&full), "{full} a second");
        assert!(count(Id::Greatsword, 2) < full, "half fury burns slower");
    }
}
