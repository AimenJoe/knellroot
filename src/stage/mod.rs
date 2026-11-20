//! 스테이지(0019): 지도(글자 격자) → 레벨(벽·소품·불빛·스폰·문) + 진행 대본(구역·물결·문·보스·핵). 새 스테이지 = 파일 하나
//! (`StageDef` 한 개). 진행은 월드 밖에서 월드를 지켜보며 적을 세우고 문을 여닫는다 — 월드는 스테이지를 모른다(경계 그대로).
//! 앱은 `StageRun::tick`의 결과(`StageOut`)로 장면만 바꾼다(흐름은 `app::flow`의 순수 함수).
pub mod forest;
pub mod garden;
mod level;
pub mod sanctuary;
pub mod village;
pub mod workings;

use macroquad::prelude::*;

use crate::ground::{Cell, Ground};
use crate::i18n::Tr;
use crate::species::Kind;
use crate::world::{Aabb, EState, PState, World};

/// 지도 한 칸(m).
pub const CELL: f32 = 2.0;
/// 떼 마당(0244)이 비키는 거리(m): 깨어난 자리 · 쉼터 · 주민(적의 끈 밖) · 문지기 · 길목(장치·문·오르내림·상자·새긴 돌).
const HORDE_AWAY_START: f32 = 18.0;
const HORDE_AWAY_REST: f32 = 14.0;
const HORDE_AWAY_RESIDENT: f32 = 26.0;
const HORDE_AWAY_BOSS: f32 = 20.0;
const HORDE_AWAY_SPOT: f32 = 8.0;
/// 떼 마당의 반지름(칸) — 그 안의 평평한 바닥에 칸마다 넷까지 선다.
const HORDE_RADIUS: i32 = 4;
/// 벽 높이(m, 굽기에서 칸마다 조금씩 다르게 깎는다).
pub const WALL_H: f32 = 3.0;

/// 소품 종류 — 모델 이름(`assets/models/<이름>.glb`)과 부딪힘 상자.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PropKind {
    Pillar,
    Root,
    Brazier,
    Rubble,
    WallPiece,
    Bell,
}

impl PropKind {
    pub fn model(self) -> &'static str {
        match self {
            PropKind::Pillar => "pillar_broken",
            PropKind::Root => "root_clump",
            PropKind::Brazier => "brazier",
            PropKind::Rubble => "rubble",
            PropKind::WallPiece => "wall_segment",
            PropKind::Bell => "bell",
        }
    }

    /// 부딪힘 상자 반폭(XZ, m) — 0이면 지나간다.
    pub fn half(self) -> Vec2 {
        match self {
            PropKind::Pillar => vec2(0.6, 0.6),
            PropKind::Root => vec2(0.5, 0.5),
            PropKind::Brazier => vec2(0.35, 0.35),
            PropKind::Rubble => Vec2::ZERO,
            PropKind::WallPiece => vec2(1.0, 0.4),
            PropKind::Bell => vec2(0.8, 0.8),
        }
    }

    pub fn height(self) -> f32 {
        match self {
            PropKind::Pillar => 3.0,
            PropKind::Root => 1.0,
            PropKind::Brazier => 1.0,
            PropKind::Rubble => 0.6,
            PropKind::WallPiece => 3.0,
            PropKind::Bell => 2.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Prop {
    pub kind: PropKind,
    pub pos: Vec3,
    pub yaw: f32,
}

/// 불빛(화로) — 렌더가 정점에 더한다.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Light {
    pub pos: Vec3,
    pub color: Color,
    pub radius: f32,
    /// 천장 틈 빛줄기(0119 → 0124): 둥글게 퍼지지 않고 위에서 내리꽂는 기둥 — `radius`는 기둥의 가로 반지름,
    /// 그 안의 것은 높이와 상관없이 위를 보는 면부터 밝다(서면 몸이 비친다).
    pub shaft: bool,
}

/// 지도에서 읽은 레벨(정적인 것).
pub struct Level {
    pub w: usize,
    pub h: usize,
    /// 칸: '#' 벽, '.' 바닥, ' ' 바깥.
    pub cells: Vec<u8>,
    pub walls: Vec<Aabb>,
    pub props: Vec<Prop>,
    pub lights: Vec<Light>,
    /// 표지 글자 → 그 칸들의 가운데.
    pub marks: Vec<(char, Vec3)>,
    pub player_start: Vec3,
    /// 꺼진 화로(0171 — 깨어난 자리의 것): 그 자리의 화로는 불꽃도 빛도 없이 굽는다. 거기서 처음 쉬면 불이 붙는다.
    pub cold: Option<Vec3>,
    /// 문(닫히면 막는 상자)과 그 가운데.
    pub gate: Option<(Aabb, Vec3)>,
    /// 높낮이(0035 그림 → 0043 걸을 수 있게): 칸마다 높이(m). 바닥 칸은 땅 높이, 바깥 칸은 바위 단(+)·꺼진 땅(-),
    /// 벽 칸은 벽 꼭대기. None = 기본(바닥 0·벽 무작위·바깥 허공).
    pub heights: Vec<Option<f32>>,
    /// 경사로 칸(0043): 이웃 높이를 이어 준다(`ground`).
    pub ramps: Vec<bool>,
}

/// 스테이지 정의(데이터).
pub struct StageDef {
    pub name: Tr,
    pub map: &'static str,
    /// 배경 높낮이 격자(0035, 그림만 — `Level::with_heights`). 없으면 평평.
    pub heights: Option<&'static str>,
    /// 구역: (이름, 이 행보다 위(작은 z)로 들어가면).
    pub zones: &'static [(Tr, usize)],
    /// 물결: (스폰 표지, 종류) 묶음 — 앞 물결이 이만큼 남으면 다음 물결.
    pub waves: &'static [&'static [(char, Kind)]],
    /// 보스 표지.
    pub boss: (char, Kind),
    /// 시작 이야기 두 줄.
    pub intro: (Tr, Tr),
    pub obj_waves: Tr,
    pub obj_gate: Tr,
    pub obj_core: Tr,
    pub boss_wakes: Tr,
    /// 상자(지도 `C`, 위에서부터 차례)마다 든 것(0064 M3 → 0117: 장신구 또는 무기). 이미 가졌으면 잔불 `CHEST_EMBERS`.
    pub chest_loot: &'static [Loot],
    /// 문지기를 처음 이겼을 때 주는 장신구.
    pub boss_charm: Option<crate::charms::Id>,
    /// 새긴 돌(0129 C4, 지도 `N`, 위에서부터 차례)마다 새긴 글 — 곁에서 상호작용하면 화면 아래 글 판(평범한 삶의 흔적, lore 0116).
    pub stones: &'static [Tr],
    /// 숨은 물건 기록 이름의 앞(0105): `<앞>.<종류>.<그 종류의 몇째>` — 스테이지가 늘어도 섞이지 않게.
    pub find_prefix: &'static str,
    /// 지도에 서 있는 적(0113): (지도 표지, 종류) — 제자리에서 제 일을 하다 알아차리면 쫓는다. 화로에서 쉬면 다시 선다.
    /// 물결(`waves`)은 시험·PoC·특별한 기믹 스테이지에만 — 보통 스테이지는 넓은 지역 + 밀집도(사장 2026-11-08).
    pub spawns: &'static [(char, Kind)],
    /// 떼 마당(0244 — 사장 설문 k13 "적들이 바글바글한 구간이 있어야 함"): 지역마다 넓은 마당 한 곳에 약한 잡몹을 몰아 세운다
    /// (종은 차례로 돌린다, 수). 자리는 지도를 고치지 않고 `Level::horde_spots`가 고른다. 빈 표 = 없음.
    pub horde: (&'static [Kind], usize),
    /// 청소자 자리(0120 — 0116 ECO-10 "같은 종, 다른 역할"): 이 표지에 선 적은 보고도 덤비지 않고 먹는다(맞거나 동료가 깨면 싸운다).
    pub docile: &'static [char],
    /// 엘리트(0112 → 0113): (지도 표지, 종류, 성질) — 서 있는 적과 같은 꼴, 단 그 판에 한 번 쓰러뜨리면 쉬어도 다시 서지 않는다.
    /// 기록 이름은 `<find_prefix>.elite.<이 표의 몇째>`(처음 쓰러뜨린 것만 덤).
    pub elites: &'static [(char, Kind, crate::enemy::Affix)],
    /// 처음(문이 닫혀 있을 때)의 목표 글 — 서 있는 적 스테이지는 문을 여는 장치를 찾게.
    pub obj_explore: Tr,
    /// 장치를 당겼을 때의 카드(성소 "어딘가에서 문이 열렸다", 마을 "사다리가 내려왔다").
    pub card_lever: Tr,
    /// 지하 몇 m(0118, 연출 표 — 지역 사이 전환(0174)의 깊이 글·카드가 같이 쓴다). 차례대로 깊어진다(시험).
    pub depth_m: u32,
    /// 구역 등급(0112·0116, 1부터): 잡몹의 체력·자세·잔불(등급 3부터 피해 20 — 두 눈금). 깊을수록 높다(시험).
    pub tier: u8,
    /// 지역 환경(0118 T2): 원경·달빛·안개·떠다니는 것·식생 — `backdrop::SURFACE`·`UNDERGROUND`.
    pub env: &'static crate::backdrop::Env,
    /// 문(`G`)이 지름길일 뿐인 지역(0208 — 작업층의 화물 승강대): 문지기 싸움터의 문이 아니라서 처음부터 문 앞 단계(문지기는 장치와
    /// 상관없이 깬다), 문은 장치를 당겨야만 열리고 연 뒤에는 닫히지 않는다(문지기가 깨도).
    pub shortcut: bool,
    /// 서막 지역(0262 — 숲): 표의 끝에 붙고 판의 차례(내려가는 `D`·오르는 `U`) 밖이다. 문지기·핵·문·화로·상자·숨은 물건이 없고,
    /// 끝의 표지(`EXIT_GLYPH`)를 밟으면 "성소에 들다"(`StageOut::Onward`), 쓰러지면 깨어난 자리에서 다시 선다. 종소리는 그 표지에서.
    pub prologue: bool,
}

/// 판의 차례(0118, v0.5.0 T1): 성소 → 떠난 마을. 지도 `D`(내려가는 길, 핵을 거둔 뒤)로 다음, `U`(오르는 길)로 앞.
/// 서막 숲(0262)은 표의 **끝**에 붙는다 — 기록의 지역 번호(`run.stage`)·상자 번호(`지역 × 1000`)·바닥 아이템의 지역이 그대로다.
/// 판의 차례는 앞의 `REGIONS`개뿐(마지막 지역 = `STAGES[REGIONS - 1]`).
pub static STAGES: [&StageDef; 5] = [
    &sanctuary::DEF,
    &village::DEF,
    &garden::DEF,
    &workings::DEF,
    &forest::DEF,
];
/// 판의 차례에 드는 지역 수(0262 — 서막 밖).
pub const REGIONS: usize = 4;
/// 서막 숲의 자리(`STAGES`의 끝, 0262). 지역이 늘면 새 지역은 숲 **앞에** 끼우고 이 값을 민다 — 숲은 기록에 지역 번호를 남기지
/// 않는다(쉼터·상자·숨은 물건이 없고, 바닥에 내려놓은 것도 적지 않는다 — `App::note_ground`).
pub const PROLOGUE: usize = 4;
/// 서막 숲 끝의 표지(0262 — 성소로 가는 길)와 밟는 거리(m — 길 폭을 다 덮게).
pub const EXIT_GLYPH: char = 'E';
pub const EXIT_REACH: f32 = 2.4;
/// 성소에 닿는 자리(0262 — 입구 폐허의 남쪽 가장자리, "성소에 들다"가 여기서 몇 걸음 걸어 들어온다).
pub const ARRIVE_GLYPH: char = 'A';
/// 내려가는 길·오르는 길 표지(0118)와 밟는 거리(m). 표지에서 이만큼 벗어난 뒤에야 다시 밟을 수 있다.
pub const DOWN_GLYPH: char = 'D';
pub const UP_GLYPH: char = 'U';
pub const PASSAGE_REACH: f32 = 1.2;
/// 천장 틈 빛줄기 표지(0118 T2)와 기둥의 가로 반지름(m — 바닥 웅덩이와 같게).
pub const SHAFT_GLYPH: char = 'S';
pub const SHAFT_R: f32 = 1.7;
/// 상자 기록 번호(0118): 지역마다 `지역 × 이것 + 상자 번호`(성소는 그대로 — 옛 기록이 이어진다).
pub const CHESTS_PER_STAGE: u32 = 1000;
const PASSAGE_ARM: f32 = 2.6;

/// 레일 표지(0208): 수레가 오가는 줄(나란한 두 칸이 한 짝).
pub const RAIL_GLYPH: char = '|';

/// 레일 칸들을 이어진 묶음으로 — 묶음마다 (남쪽 끝, 북쪽 끝)의 가운데 줄(길이 방향이 긴 쪽).
pub fn rail_lanes(cells: &[Vec3]) -> Vec<(Vec3, Vec3)> {
    let mut seen = vec![false; cells.len()];
    let mut out = Vec::new();
    for s in 0..cells.len() {
        if seen[s] {
            continue;
        }
        let mut group = vec![s];
        seen[s] = true;
        let mut k = 0;
        while k < group.len() {
            let c = cells[group[k]];
            for (j, d) in cells.iter().enumerate() {
                if !seen[j] && (d.x - c.x).abs() <= CELL * 1.01 && (d.z - c.z).abs() <= CELL * 1.01
                {
                    seen[j] = true;
                    group.push(j);
                }
            }
            k += 1;
        }
        let pts: Vec<Vec3> = group.iter().map(|&i| cells[i]).collect();
        let (x0, x1) = pts
            .iter()
            .fold((f32::MAX, f32::MIN), |m, p| (m.0.min(p.x), m.1.max(p.x)));
        let (z0, z1) = pts
            .iter()
            .fold((f32::MAX, f32::MIN), |m, p| (m.0.min(p.z), m.1.max(p.z)));
        let (cx, cz) = ((x0 + x1) * 0.5, (z0 + z1) * 0.5);
        if z1 - z0 >= x1 - x0 {
            out.push((vec3(cx, 0.0, z1), vec3(cx, 0.0, z0)));
        } else {
            out.push((vec3(x0, 0.0, cz), vec3(x1, 0.0, cz)));
        }
    }
    out
}

/// 문을 여는 장치(0113): 지도 `V`. 곁에서 상호작용 = 당기기 — 문이 열리고 기록에 남는다(지름길, 소울 고리).
pub const LEVER_GLYPH: char = 'V';
pub const LEVER_REACH: f32 = 1.8;

/// 숨은 물건(0105, v0.4.0 G3 — 0102 6절): 핵 고르기의 등잔·한 모금이 탐험의 보상으로. 지도 글자 `F` 등잔 +1회 · `H` 한 모금 +1.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Find {
    Lamp,
    Sip,
    /// 종소리 조각(0106, v0.4.0 G2): 기술 나무 점수 +1. 이름이 출처를 정하지 않는다(lore — 종에서 떨어진 것인지 말하지 않는다).
    Shard,
    /// 덧옷(0176): 몸에 걸치는 것 — 어느 덧옷인지는 기록 이름으로(`garb::TABLE`의 `find`).
    Garb,
    /// 숫돌(0180): 무기를 벼리는 재료 — 정해진 수만 지도에 놓인다(`forge.rs`).
    Stone,
}

impl Find {
    pub const ALL: [Find; 5] = [Find::Lamp, Find::Sip, Find::Shard, Find::Garb, Find::Stone];

    pub fn glyph(self) -> char {
        match self {
            Find::Lamp => 'F',
            Find::Sip => 'H',
            Find::Shard => 'K',
            Find::Garb => 'M',
            Find::Stone => 'O',
        }
    }

    /// 기록 이름 가운데 칸(바꾸지 않는다).
    pub fn key(self) -> &'static str {
        match self {
            Find::Lamp => "lamp",
            Find::Sip => "sip",
            Find::Shard => "shard",
            Find::Garb => "garb",
            Find::Stone => crate::forge::STONE_KEY,
        }
    }
}

/// 숨은 물건 하나의 기록 이름: `sanctuary.lamp.0`.
pub fn find_id(prefix: &str, kind: Find, nth: usize) -> String {
    format!("{prefix}.{}.{nth}", kind.key())
}

/// 엘리트 기록 이름(0112): `<앞>.elite.<표의 몇째>`.
pub fn elite_id(prefix: &str, nth: usize) -> String {
    format!("{prefix}.elite.{nth}")
}

/// 숨은 물건 곁(m) — 상자와 같다.
pub const FIND_REACH: f32 = 1.8;
/// 떨어진 아이템(0131): 상자에서 튀어나오는 거리(m), 튀어나와 내려앉는 초(그동안 줍지 못한다 — 렌더의 포물선과 같은 길이).
pub const DROP_OUT: f32 = 1.3;
pub const DROP_SETTLE: f32 = 0.45;
/// 깨어난 자리 곁에 떨어진 검(0220 — 사장 "맨손으로 시작하되 바로 옆에 검이 떨어져 있어야 도감이 제대로 작동할 듯"): `drops`의
/// 상자 번호 자리에 이 값이 든 것 — 상자에서 나온 것이 아니라 처음부터 놓여 있다.
pub const START_SWORD: usize = usize::MAX;
/// 그 검이 놓이는 자리: 시작 자리에서 이만큼(m, 월드 x·z).
const START_SWORD_AT: Vec3 = Vec3::new(1.6, 0.0, -0.8);
/// 떨어지는 쪽: 방랑자 쪽에서 이만큼(도) 비껴.
const DROP_SIDE: f32 = 65.0;
/// 같은 자리에 떨어진 아이템 사이의 거리(0226 — 겹치면 하나로 보인다).
pub const DROP_APART: f32 = 0.7;

/// 상자 곁(m)과, 든 장신구를 이미 가졌을 때 대신 주는 잔불.
pub const CHEST_REACH: f32 = 1.8;
/// 새긴 돌을 읽는 거리(m, 0129).
const STONE_REACH: f32 = 2.2;
pub const CHEST_EMBERS: u32 = 10;

/// 상자에 든 것(0117): 장신구 또는 무기 — 무기는 얻어서 쓴다(검·맨손만 처음부터).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Loot {
    Charm(crate::charms::Id),
    Weapon(crate::weapons::Id),
}
/// 상자 부딪힘 상자 반폭(가로·세로 m).
const CHEST_HALF: Vec3 = Vec3::new(0.45, 0.5, 0.3);

/// 진행 단계.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Phase {
    /// 입구(아직 뜰에 안 들어감).
    Entrance,
    /// 뜰: n번째 물결.
    Waves(usize),
    /// 문이 열렸다(성소로 들어가기 전).
    GateOpen,
    /// 성소에 들어와 문이 닫히고 문지기가 깨어나는 중(초).
    Waking(f32),
    Duel,
    /// 문지기가 쓰러지고 핵이 남았다.
    Core,
    /// 핵을 거뒀고 아래 지역이 있다(0118): 바닥 길(`D`)이 열려 있다 — 이 판에서는 문지기가 다시 서지 않는다.
    Down,
    Cleared,
}

/// 진행 결과(앱이 장면을 바꾸는 데만 쓴다).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StageOut {
    Playing,
    /// 마지막이 아닌 지역의 핵을 거뒀다(0118) — 앱은 핵 보상만, 판은 이어진다(바닥 길이 열린다).
    CoreTaken,
    /// 열린 바닥 길·오르는 길을 밟았다(0118) — 앱이 지역 사이 전환(뿌리 굴, 0174)을 열고 지역을 바꾼다.
    Descend,
    Ascend,
    Cleared,
    Defeated,
    /// 쓰러졌지만 마지막으로 쉰 쉼터에서 다시 일어섰다(0032) — 앱은 카메라만 그 자리로.
    Respawned,
    /// 서막 숲 끝의 표지를 밟았다(0262) — 앱이 "성소에 들다"(`arrive`)를 연다(다 어두울 때 지역을 성소로 바꾼다).
    Onward,
}

/// 쉼터 곁(m): 이 안이면 "쉬기"가 뜬다.
pub const REST_REACH: f32 = 2.4;
/// 이 안에 깨어 있는 적이 있으면 쉴 수 없다(m).
pub const REST_SAFE: f32 = 10.0;

/// 문지기가 깨어나는 데 걸리는 시간(초) — 이름 카드·`wake` 동작.
pub const WAKE_TIME: f32 = 2.0;
/// 다음 물결: 앞 물결이 이만큼 이하로 남으면.
const WAVE_LEFT: usize = 1;
/// 핵을 거두는 거리(m).
const CORE_REACH: f32 = 1.4;
/// 처음 이긴 문지기의 핵(lore CANON — 0101·0105): 이 안에 들어서면 핵이 스스로 방랑자 쪽으로 다가온다(m, m/s).
const CORE_DRAW: f32 = 4.0;
const CORE_DRAW_SPEED: f32 = 2.2;
/// 이름 카드가 떠 있는 시간(초).
pub const CARD_TIME: f32 = 3.0;

/// 장신구 상자(0064 M3 — 0251 R10에서 묶음).
#[derive(Default)]
pub struct Chests {
    /// 장신구 상자(0064 M3): 자리, 열렸나(기록에서 — 한 번 연 상자는 판을 넘어 비어 있다), 곁의 닫힌 상자, 이번 틱에 연 상자(앱이 가져간다).
    pub at: Vec<Vec3>,
    pub open: Vec<bool>,
    pub near: Option<usize>,
    opened: Option<usize>,
}

/// 숨은 물건(0105 — 0251 R10에서 묶음).
#[derive(Default)]
pub struct Finds {
    /// 숨은 물건(0105): 자리·종류·기록 이름, 주웠나(기록에서), 곁의 것, 이번 틱에 주운 것(앱이 가져간다).
    pub list: Vec<(Vec3, Find, String)>,
    pub found: Vec<bool>,
    pub near: Option<usize>,
    picked: Option<usize>,
}

/// 남은 주민(0122 — 0251 R10에서 묶음).
#[derive(Default)]
pub struct Folk {
    /// 남은 주민(0122): (표 번호, 자리), 지금 바라보는 쪽(가까이 오면 돌아본다), 재방문으로 바뀌었나(앱이 기록에서),
    /// 이 판에 봤나(처음 본 틱에 앱이 기록에), 곁의 주민, 이번 틱에 말 건 주민(앱이 가져간다 — 말풍선).
    pub list: Vec<(usize, Vec3)>,
    pub face: Vec<f32>,
    pub changed: Vec<bool>,
    seen: Vec<bool>,
    seen_new: Vec<usize>,
    pub near: Option<usize>,
    talked: Option<usize>,
}

pub struct StageRun {
    pub def: &'static StageDef,
    pub level: Level,
    pub phase: Phase,
    /// 지금 구역(이름 카드를 띄울 때 바뀐다)과 카드 남은 초.
    pub zone: usize,
    pub card: Option<(Tr, f32)>,
    pub objective: Option<Tr>,
    /// 시작 이야기 남은 초.
    pub intro: f32,
    /// 이번 물결의 적 번호들.
    wave_ids: Vec<usize>,
    pub boss: Option<usize>,
    pub gate_open: f32,
    /// 문이 열려 있나(부딪힘 상자).
    pub gate_closed: bool,
    pub core: Option<Vec3>,
    pub time: f32,
    first_card: bool,
    /// 쉼터 화로들(지도 `R`).
    pub rests: Vec<Vec3>,
    /// 떼 마당의 자리(0244, 판을 세울 때 한 번 — `Level::horde_spots`).
    pub horde_at: Vec<Vec3>,
    /// 바깥에서 얹는 갈 곳(0170 — 튜토리얼이 "쉼터로"를 가리킬 때의 그 쉼터): 진행의 갈 곳(`goal`의 문·핵)이 없을 때만 쓰인다.
    /// 앱이 틱마다 다시 정한다(없으면 `None`).
    pub guide: Option<Vec3>,
    /// 마지막으로 쉰 쉼터와, 쓰러지면 되돌아갈 진행 단계(쉰 때의 단계 — 물결 전이면 입구, 물결을 끝냈으면 문 앞).
    pub rest_at: Option<(usize, Phase)>,
    /// 곁에 있는 쉼터(HUD·터치 버튼), 지금 쉴 수 있나(가까운 깨어 있는 적이 없다).
    pub near_rest: Option<usize>,
    pub rest_safe: bool,
    /// 마지막으로 쉰 뒤 지난 초(렌더의 퍼지는 빛).
    pub since_rest: f32,
    /// 장신구 상자(0064 M3 — `Chests`).
    pub chest: Chests,
    /// 떨어진 아이템(0131, 사장 "디아블로식 — 상자를 열면 아이템이 떨어짐, 바로 습득되는 것이 아니라"): 연 상자 앞 바닥에 놓이고
    /// 주워야 얻는다. (자리, 어느 상자, 떨어진 뒤 초). 곁의 것, 이번 틱에 주운 상자 번호(앱이 가져가 그 상자의 것을 준다·기록한다).
    pub drops: Vec<(Vec3, usize, f32)>,
    pub near_drop: Option<usize>,
    looted: Option<usize>,
    /// 떨어진 아이템 하나하나(0226 — 사장 "가득이면 못 주워야지, 바닥에 떨어져 있어야지"): 엘리트·문지기가 떨군 여벌과
    /// 방랑자가 가방에서 내려놓은 것(`ItemDrop`). 앱이 기록에도 적는다 — 지역을 떠났다 와도, 껐다 켜도 그 자리에 있다.
    pub item_drops: Vec<ItemDrop>,
    pub near_item: Option<usize>,
    /// 가방이 가득인가(0226 — 앱이 틱마다 알린다): 가득이면 떨어진 아이템을 줍지 못한다. `chest_block`은 가득일 때
    /// 못 줍는 상자의 것(비트 = 상자 번호 — 아직 없는 바탕이 든 상자: 잔불이 될 것은 줍는다).
    pub bag_full: bool,
    pub chest_block: u64,
    /// 가득일 때 깨어난 자리의 검(0220 — 아직 없는 바탕)도 못 줍는다.
    pub start_block: bool,
    /// 곁에 가방이 가득이라 못 줍는 것이 있다(HUD가 까닭을 말한다 — 버튼은 그대로 점프).
    pub near_blocked: bool,
    taken: Option<crate::items::Item>,
    /// 이번 틱에 주운 아이템의 자리(앱이 가져가 줍는 이펙트를 낸다) · 이번 틱에 쓰러진 문지기의 자리(앱이 가져가 그 곁에
    /// 떨굼을 놓는다 — 0226: 문지기의 것도 바로 얻지 않고 바닥에 떨어진다).
    taken_at: Option<Vec3>,
    boss_down: Option<Vec3>,
    /// 이 판에 떨어뜨린 아이템 수 — 떨굼의 씨앗에 섞는다(줍지 않고 둔 채 다시 쓰러뜨려도 다른 것이 나오게).
    item_serial: u32,
    /// 새긴 돌(0129): 자리, 곁의 돌, 이번 틱에 읽은 돌(앱이 가져간다).
    pub stones: Vec<Vec3>,
    pub near_stone: Option<usize>,
    read: Option<usize>,
    /// 도감(0130): 이 판에 이미 알린 이름, 이번 틱까지 새로 본 이름(앱이 가져간다).
    codex_told: Vec<String>,
    codex_new: Vec<String>,
    /// 이번 틱에 쉬었다(0064 M4 — 앱이 화로 가게를 연다), 잔불로 살 것이 있다(앱이 정한다 — 쉼터가 빛난다).
    rested: bool,
    /// 이번 틱에 꺼진 화로에 불이 붙었다(0171 — 앱이 가져가 기록에 적고 그 순간을 보여 준다).
    kindled: bool,
    pub shop_ready: bool,
    /// 숨은 물건(0105 — `Finds`).
    pub hidden: Finds,
    /// 엘리트(0112): 지금 서 있는 (적 번호, 기록 이름), 기록에서 이미 쓰러뜨린 것, 이번 틱에 처음 쓰러뜨린 것(앱이 가져간다).
    elite_ids: Vec<(usize, String)>,
    /// 이 판에 쓰러뜨린 엘리트(표의 몇째, 0113) — 쉬어도 다시 서지 않는다.
    elites_down: Vec<usize>,
    /// 이번 틱에 쓰러진 엘리트(표의 몇째, 쓰러진 자리, 기록에서 처음인가) — 앱이 가져가 그 자리에 여벌을 떨군다
    /// (0223 V4c → 0226, 처음이 아니면 확률로 — 0244).
    elite_loot: Vec<(usize, Vec3, bool)>,
    /// 문 장치(0113): 자리, 곁인가, 문이 열려 있나(기록 — 한 번 열면 판을 넘어 열려 있다), 이번 틱에 처음 열었다(앱이 가져간다).
    pub lever: Option<Vec3>,
    pub near_lever: bool,
    pub gate_opened: bool,
    gate_new: bool,
    /// 방랑자가 장치를 당겼다(0197 — 앱이 당기는 몸짓을 튼다).
    pulled: bool,
    beaten: Vec<String>,
    elite_new: Vec<String>,
    /// 이 문지기를 처음 이기는 판인가(앱이 기록에서 — 핵이 스스로 다가온다, lore CANON 0101).
    pub first_core: bool,
    /// 내려가는 길·오르는 길 자리(0118, 지도 `D`·`U`), 이 판에 이 지역 문지기를 이겼나(바닥 길이 열린 채, 문지기 없음),
    /// 표지에서 한 번 벗어났나(도착하자마자 되돌아가지 않게).
    pub down_at: Option<Vec3>,
    pub up_at: Option<Vec3>,
    pub down_open: bool,
    passage_armed: bool,
    /// 서막 숲 끝의 표지(0262, 지도 `E`).
    pub exit_at: Option<Vec3>,
    /// 남은 주민(0122 — `Folk`).
    pub folk: Folk,
}

/// 바닥에 떨어진 아이템 하나(0226). `wait` = 방랑자가 한 번 곁을 떠나야 주울 수 있다 — 발치에 내려놓은 것이 곧바로 "줍기"가 되어
/// 같은 버튼의 다른 일(쉬기)을 가리지 않게, 그리고 방금 내려놓은 것을 도로 줍지 않게.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemDrop {
    pub pos: Vec3,
    pub item: crate::items::Item,
    /// 떨어진 뒤 초.
    pub age: f32,
    pub wait: bool,
}

/// 시작 이야기 길이(초).
pub const INTRO_TIME: f32 = 6.0;

/// 점프(상호작용) 버튼이 지금 하는 일(0166): HUD의 동사와 터치 버튼의 그림이 같은 답을 쓴다. 차례는 월드가 처리하는 차례와 같다 —
/// 줍기(떨어진 것 · 숨은 물건) > 열기 > 당기기 > 말 걸기 > 읽기 > 쉬기.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Near {
    Pick,
    Open,
    Pull,
    Talk,
    Read,
    /// 쉼터 곁. `safe`가 아니면(적이 가까이) 쉴 수 없다 — 버튼은 그대로 점프다.
    Rest {
        safe: bool,
    },
    /// 곁에 떨어진 아이템이 있지만 가방이 가득이다(0226) — 버튼은 그대로 점프, HUD가 까닭을 말한다.
    Full,
}

impl StageRun {
    /// 곁에 있는 할 일(없으면 `None`).
    pub fn near(&self) -> Option<Near> {
        if self.hidden.near.is_some() || self.near_drop.is_some() || self.near_item.is_some() {
            Some(Near::Pick)
        } else if self.chest.near.is_some() {
            Some(Near::Open)
        } else if self.near_lever {
            Some(Near::Pull)
        } else if self.folk.near.is_some() {
            Some(Near::Talk)
        } else if self.near_stone.is_some() {
            Some(Near::Read)
        } else if self.near_rest.is_some() && self.rest_safe {
            Some(Near::Rest { safe: true })
        } else if self.near_blocked {
            Some(Near::Full)
        } else {
            self.near_rest.map(|_| Near::Rest { safe: false })
        }
    }

    /// 레벨을 세우고 월드를 만든다.
    pub fn start(def: &'static StageDef) -> (StageRun, World) {
        let mut level = Level::parse(def.map);
        if let Some(g) = def.heights {
            level = level.with_heights(g).settle();
        }
        let mut w = World::new("stage", level.half_extent());
        w.obstacles = level.obstacles();
        w.ground = Some(level.ground());
        if let Some((g, _)) = level.gate {
            w.obstacles.push(g);
        }
        w.player.pos = level.player_start;
        w.player.prev = w.player.pos;
        w.player.facing = std::f32::consts::PI; // 위(북쪽, -z)를 본다
        let mut run = StageRun {
            def,
            level,
            phase: Phase::Entrance,
            zone: 0,
            card: None,
            objective: None,
            guide: None,
            intro: INTRO_TIME,
            wave_ids: Vec::new(),
            boss: None,
            gate_open: 0.0,
            gate_closed: true,
            core: None,
            time: 0.0,
            first_card: false,
            rests: Vec::new(),
            horde_at: Vec::new(),
            rest_at: None,
            near_rest: None,
            rest_safe: true,
            since_rest: f32::MAX,
            chest: Chests::default(),
            drops: Vec::new(),
            near_drop: None,
            looted: None,
            item_drops: Vec::new(),
            near_item: None,
            bag_full: false,
            chest_block: 0,
            start_block: false,
            near_blocked: false,
            taken: None,
            taken_at: None,
            boss_down: None,
            item_serial: 0,
            stones: Vec::new(),
            near_stone: None,
            read: None,
            codex_told: Vec::new(),
            codex_new: Vec::new(),
            rested: false,
            kindled: false,
            shop_ready: false,
            hidden: Finds::default(),
            elite_ids: Vec::new(),
            elites_down: Vec::new(),
            elite_loot: Vec::new(),
            lever: None,
            near_lever: false,
            gate_opened: false,
            gate_new: false,
            pulled: false,
            beaten: Vec::new(),
            elite_new: Vec::new(),
            first_core: false,
            down_at: None,
            up_at: None,
            down_open: false,
            passage_armed: true,
            exit_at: None,
            folk: Folk::default(),
        };
        // 주민(0122): 표지마다 — 처음엔 제 일 쪽을 본다(0132), 몸은 방랑자만 밀어낸다.
        for (c, p) in run.level.marks.clone() {
            if let Some(i) = crate::residents::by_mark(c) {
                run.folk.list.push((i, p));
                run.folk.face.push(crate::residents::WORK_FACE);
                w.bodies.push((p, crate::residents::BODY_R));
            }
        }
        run.folk.changed = vec![false; run.folk.list.len()];
        run.folk.seen = vec![false; run.folk.list.len()];
        run.down_at = run.level.marks_of(DOWN_GLYPH).next();
        run.up_at = run.level.marks_of(UP_GLYPH).next();
        run.exit_at = run.level.marks_of(EXIT_GLYPH).next();
        run.rests = run.level.marks_of('R').collect();
        run.chest.at = run.level.marks_of('C').collect();
        run.stones = run.level.marks_of('N').collect();
        run.chest.open = vec![false; run.chest.at.len()];
        for kind in Find::ALL {
            let at: Vec<Vec3> = run.level.marks_of(kind.glyph()).collect();
            for (n, p) in at.into_iter().enumerate() {
                run.hidden
                    .list
                    .push((p, kind, find_id(def.find_prefix, kind, n)));
            }
        }
        run.hidden.found = vec![false; run.hidden.list.len()];
        for c in &run.chest.at {
            w.obstacles.push(crate::world::Aabb {
                min: *c - vec3(CHEST_HALF.x, 0.0, CHEST_HALF.z),
                max: *c + vec3(CHEST_HALF.x, CHEST_HALF.y * 2.0, CHEST_HALF.z),
            });
        }
        // 레일 수레(0208): 지도의 레일 `|` 묶음마다 하나 — 남쪽 끝에서 쉰다(묶음마다 박자를 조금 엇갈려).
        w.carts = rail_lanes(&run.level.marks_of(RAIL_GLYPH).collect::<Vec<_>>())
            .into_iter()
            .enumerate()
            .map(|(i, (a, b))| {
                let y = |p: Vec3| w.ground.as_ref().map_or(p.y, |g| g.height(p));
                let (a, b) = (vec3(a.x, y(a), a.z), vec3(b.x, y(b), b.z));
                crate::world::cart::Cart::new(a, b, i as f32 * 0.9)
            })
            .collect();
        // 마지막 구역(문지기 싸움터)의 레일은 문지기 2단계에만 돈다(0208 6절).
        for c in &mut w.carts {
            c.boss = run.zone_at((c.a + c.b) * 0.5) + 1 >= def.zones.len();
            if c.boss {
                c.wait = crate::tuning::CART_WAIT;
            }
        }
        // 보스는 처음부터 잠든 채 성소에 있다(보인다 — 기대감).
        if let Some(p) = run.level.marks_of(def.boss.0).next() {
            let i = w.spawn(def.boss.1, p);
            w.enemies[i].state = EState::Dormant;
            run.boss = Some(i);
        }
        // 서 있는 적(0113): 처음부터 제자리에. 문 장치 자리.
        run.lever = run.level.marks_of(LEVER_GLYPH).next();
        // 떼 마당(0244): 깨어난 자리·화로·주민·문지기 싸움터·길목에서 떨어진 가장 넓은 바닥.
        if def.horde.1 > 0 {
            let mut avoid: Vec<(Vec3, f32)> = vec![(run.level.player_start, HORDE_AWAY_START)];
            avoid.extend(run.rests.iter().map(|p| (*p, HORDE_AWAY_REST)));
            avoid.extend(run.folk.list.iter().map(|(_, p)| (*p, HORDE_AWAY_RESIDENT)));
            avoid.extend(run.level.marks_of(def.boss.0).map(|p| (p, HORDE_AWAY_BOSS)));
            for c in [LEVER_GLYPH, DOWN_GLYPH, UP_GLYPH, 'C', 'N'] {
                avoid.extend(run.level.marks_of(c).map(|p| (p, HORDE_AWAY_SPOT)));
            }
            avoid.extend(run.level.gate.map(|(_, c)| (c, HORDE_AWAY_SPOT)));
            let last = def.zones.len().saturating_sub(1);
            let zone_of = |p: Vec3| run.zone_at(p);
            run.horde_at = run
                .level
                .horde_spots(def.horde.1, &avoid, |p| zone_of(p) < last);
        }
        if def.waves.is_empty() {
            run.populate(&mut w);
            run.objective = Some(def.obj_explore);
        }
        // 문 장치도 물결도 없는 지역(0118)은 처음부터 문이 열린 단계. 문이 지름길인 지역(0208)도 — 문은 `set_gate`가 닫아 둔다.
        // 서막 숲(0262)은 입구 단계 그대로 — 문도 싸움터도 없다(문 앞 단계는 마지막 구역에서 문지기를 깨운다).
        if !def.prologue && ((run.lever.is_none() && def.waves.is_empty()) || def.shortcut) {
            run.phase = Phase::GateOpen;
            run.set_gate(&mut w, false);
            run.objective = Some(def.obj_gate);
        }
        (run, w)
    }

    /// 위 지역에서 내려와 닿았다(0118): 시작 이야기 없이 시작 자리(`P`)에, 이름 카드.
    pub fn arrive_from_above(&mut self, w: &mut World) {
        self.intro = 0.0;
        self.first_card = true;
        self.passage_armed = false;
        let p = self.level.player_start;
        w.player.pos = p;
        w.player.prev = p;
        w.player.facing = std::f32::consts::PI;
        self.zone = self.zone_at(p);
        self.card = Some((self.def.name, CARD_TIME));
    }

    /// 서막 숲에서 왔다(0262 "성소에 들다"): 시작 이야기·구역 카드 없이(이름은 장면이 보인다) 입구 폐허 남쪽 가장자리(`A`)에서
    /// 북쪽(성소 쪽)을 본다. 표지가 없는 지도면 시작 자리.
    pub fn arrive_from_prologue(&mut self, w: &mut World) {
        self.intro = 0.0;
        self.first_card = true;
        self.passage_armed = false;
        let p = self
            .level
            .marks_of(ARRIVE_GLYPH)
            .next()
            .unwrap_or(self.level.player_start);
        w.player.pos = p;
        w.player.prev = p;
        w.player.facing = std::f32::consts::PI;
        self.zone = self.zone_at(p);
    }

    /// 서막 숲에서 쓰러졌다(0262 — 쉼터가 없다): 깨어난 자리에서 가득 찬 채 다시 선다. 서 있는 적도 다시.
    fn respawn_at_start(&mut self, w: &mut World) {
        w.lingers.clear();
        w.shards.clear();
        w.lock = None;
        w.hitstop = 0.0;
        let full = w.stats.max_stamina();
        let pl = &mut w.player;
        pl.pos = self.level.player_start;
        pl.prev = pl.pos;
        pl.vel = Vec3::ZERO;
        pl.state = PState::Idle;
        pl.air = 0.0;
        pl.hurt.left = 1.0;
        pl.hp = pl.max_hp;
        pl.flask = pl.flask_max;
        pl.ranged.mana = crate::tuning::CAST_CHARGES;
        pl.stamina = full;
        self.populate(w);
        self.zone = self.zone_at(w.player.pos);
        self.card = Some((|s| s.respawned, CARD_TIME));
    }

    /// 아래 지역에서 올라왔다(0118): 이 판에 이긴 문지기는 없고 바닥 길은 열린 채, 바닥 길 곁(남쪽)에 선다.
    pub fn arrive_from_below(&mut self, w: &mut World) {
        self.intro = 0.0;
        self.first_card = true;
        self.open_down(w);
        self.passage_armed = false;
        if let Some(d) = self.down_at {
            let p = d + vec3(0.0, 0.0, PASSAGE_ARM - 0.4);
            w.player.pos = p;
            w.player.prev = p;
            w.player.facing = 0.0;
        }
        self.zone = self.zone_at(w.player.pos);
        self.card = Some((self.def.zones[self.zone].0, CARD_TIME));
    }

    /// 이 판에 이 지역 문지기를 이겼다(0118): 문지기를 지우고 문을 열고 바닥 길 단계로.
    pub fn open_down(&mut self, w: &mut World) {
        self.down_open = true;
        w.enemies.retain(|e| !e.boss);
        self.boss = None;
        self.core = None;
        self.phase = Phase::Down;
        self.objective = Some(|s| s.obj_descend);
        self.set_gate(w, false);
    }

    fn zone_at(&self, p: Vec3) -> usize {
        let row = self.level.row_of(p);
        let mut z = 0;
        for (i, (_, edge)) in self.def.zones.iter().enumerate() {
            if row < *edge as f32 {
                z = i;
            }
        }
        z
    }

    fn set_gate(&mut self, w: &mut World, closed: bool) {
        // 지름길 문(0208 승강대): 장치를 당겨야만 열리고, 연 뒤에는 닫히지 않는다 — 단계가 바라는 것과 상관없이.
        let closed = if self.def.shortcut {
            !self.gate_opened
        } else {
            closed
        };
        if closed == self.gate_closed {
            return;
        }
        self.gate_closed = closed;
        if let Some((g, _)) = self.level.gate {
            if closed {
                w.obstacles.push(g);
            } else {
                w.obstacles.retain(|o| *o != g);
            }
        }
    }

    fn spawn_wave(&mut self, w: &mut World, n: usize) {
        self.wave_ids.clear();
        self.elite_ids.clear();
        if let Some(list) = self.def.waves.get(n) {
            for (mark, kind) in list.iter() {
                let spots: Vec<Vec3> = self.level.marks_of(*mark).collect();
                for p in spots {
                    let i = w.spawn(*kind, p);
                    w.enemies[i].set_tier(self.def.tier);
                    self.wave_ids.push(i);
                }
            }
        }
    }

    /// 서 있는 적을 세운다(0113): 보스 말고 모두 지우고 표대로 제자리에 — 엘리트는 이 판에 쓰러뜨리지 않은 것만.
    /// 시작할 때와 화로에서 쉴 때(쓰러져 다시 설 때 포함, 소울 화톳불).
    fn populate(&mut self, w: &mut World) {
        if !self.def.waves.is_empty() {
            return;
        }
        w.enemies.retain(|e| e.boss);
        self.boss = (!w.enemies.is_empty()).then_some(0);
        w.lock = None;
        w.lingers.clear();
        w.shards.clear();
        self.elite_ids.clear();
        // 바라보는 쪽은 자리마다 정해진 값(자리로 섞는다) — 늘어선 적이 한 쪽만 보지 않게.
        let face = |p: Vec3| {
            let h = (p.x * 12.9898 + p.z * 78.233).sin() * 43758.547;
            (h - h.floor()) * std::f32::consts::TAU
        };
        for &(mark, kind) in self.def.spawns {
            let spots: Vec<Vec3> = self.level.marks_of(mark).collect();
            for p in spots {
                let i = w.spawn(kind, p);
                w.enemies[i].set_tier(self.def.tier);
                w.enemies[i].place(face(p));
                w.enemies[i].docile = self.def.docile.contains(&mark);
                if self.def.prologue {
                    w.enemies[i].leash = crate::tuning::PROLOGUE_LEASH;
                }
            }
        }
        // 떼 마당(0244): 한 등급 아래의 잡몹을 몰아 — 화로에서 쉬면 다시 선다(다른 서 있는 적과 같다).
        let (kinds, _) = self.def.horde;
        if !kinds.is_empty() {
            for (k, p) in self.horde_at.clone().into_iter().enumerate() {
                let i = w.spawn(kinds[k % kinds.len()], p);
                w.enemies[i].set_tier(self.def.tier.saturating_sub(1).max(1));
                w.enemies[i].place(face(p));
            }
        }
        for (k, &(mark, kind, affix)) in self.def.elites.iter().enumerate() {
            if self.elites_down.contains(&k) {
                continue;
            }
            if let Some(p) = self.level.marks_of(mark).next() {
                let i = w.spawn(kind, p);
                w.enemies[i].set_tier(self.def.tier);
                w.enemies[i].make_elite(affix);
                w.enemies[i].place(face(p));
                self.elite_ids.push((i, elite_id(self.def.find_prefix, k)));
            }
        }
    }

    /// 기록의 문(0113): 이미 열었으면 처음부터 열린 채(지름길).
    pub fn set_gate_opened(&mut self, w: &mut World, open: bool) {
        if !open || self.gate_opened {
            return;
        }
        self.gate_opened = true;
        if self.phase == Phase::Entrance {
            self.phase = Phase::GateOpen;
            self.objective = Some(self.def.obj_gate);
        }
        self.set_gate(w, false);
    }

    /// 이번 틱에 처음 문을 열었다(0113) — 앱이 한 번 가져가 기록에.
    pub fn take_pulled(&mut self) -> bool {
        std::mem::take(&mut self.pulled)
    }

    pub fn take_gate_new(&mut self) -> bool {
        std::mem::take(&mut self.gate_new)
    }

    /// 쓰러진 엘리트(0112): 처음이면 잔불 `ELITE_FIRST_EMBERS`를 더 흩고 앱에 알린다(기록·알림).
    fn tick_elites(&mut self, w: &mut World) {
        let mut down = Vec::new();
        self.elite_ids.retain(|(i, id)| match w.enemies.get(*i) {
            Some(e) if e.alive() => true,
            Some(e) => {
                down.push((e.pos, id.clone()));
                false
            }
            None => false,
        });
        for (pos, id) in down {
            let first = !self.beaten.contains(&id);
            if let Some(k) =
                (0..self.def.elites.len()).find(|&k| elite_id(self.def.find_prefix, k) == id)
            {
                self.elites_down.push(k);
                self.elite_loot.push((k, pos, first));
            }
            if self.beaten.contains(&id) {
                continue;
            }
            let seed = (self.time * 1000.0) as u32 ^ 0x5eed_e11e;
            w.drops.extend(crate::ember::scatter(
                pos,
                crate::tuning::ELITE_FIRST_EMBERS,
                seed,
            ));
            self.beaten.push(id.clone());
            self.elite_new.push(id);
        }
    }

    /// 이번 틱에 쓰러진 엘리트들(표의 자리 — 처음이든 아니든, 0223 V4c): 앱이 한 번 가져가 여벌을 준다.
    pub fn take_elites_down(&mut self) -> Vec<(usize, Vec3, bool)> {
        std::mem::take(&mut self.elite_loot)
    }

    /// 이번 틱에 처음 쓰러뜨린 엘리트들(0112, 한 틱에 여럿 — 수류탄) — 앱이 한 번 가져가 기록·알림에.
    pub fn take_elites_new(&mut self) -> Vec<String> {
        std::mem::take(&mut self.elite_new)
    }

    /// 기록의 쓰러뜨린 엘리트들(처음 덤은 한 번만).
    pub fn set_beaten(&mut self, ids: &[String]) {
        self.beaten = ids.to_vec();
    }

    fn alive_in_wave(&self, w: &World) -> usize {
        self.wave_ids
            .iter()
            .filter(|&&i| w.enemies.get(i).is_some_and(|e| e.alive()))
            .count()
    }

    /// 한 틱(월드 틱 뒤). 구역 카드·물결·문·보스·핵을 진행한다.
    pub fn tick(&mut self, w: &mut World, dt: f32) -> StageOut {
        self.time += dt;
        self.tick_elites(w);
        self.intro = (self.intro - dt).max(0.0);
        if let Some((tr, t)) = self.card {
            self.card = (t > dt).then_some((tr, t - dt));
        }
        if self.gate_closed {
            self.gate_open = (self.gate_open - dt).max(0.0);
        } else {
            self.gate_open = (self.gate_open + dt).min(1.5);
        }
        self.since_rest = (self.since_rest + dt).min(f32::MAX);
        if w.player.hp <= 0 {
            // 서막 숲(0262)은 쉼터가 없다 — 깨어난 자리에서 다시.
            if self.def.prologue {
                self.respawn_at_start(w);
                return StageOut::Respawned;
            }
            // 쉼터에서 쉰 적이 있으면 거기서 다시(0032) — 없으면 끝.
            return match self.rest_at {
                Some((i, back)) => {
                    self.respawn(w, i, back);
                    StageOut::Respawned
                }
                None => StageOut::Defeated,
            };
        }
        // 쉼터(0032): 곁이면 "쉬기", 깨어 있는 적이 가까우면 막는다.
        let pp = w.player.pos;
        // 내려가는 길·오르는 길(0118): 표지에서 한 번 벗어나야 다시 밟힌다.
        let flat = |a: Vec3| vec2(a.x - pp.x, a.z - pp.z).length();
        let near = |at: Option<Vec3>| at.is_some_and(|a| flat(a) <= PASSAGE_REACH);
        if !self.passage_armed
            && [self.down_at, self.up_at]
                .iter()
                .flatten()
                .all(|&a| flat(a) >= PASSAGE_ARM)
        {
            self.passage_armed = true;
        }
        self.near_rest = self
            .rests
            .iter()
            .position(|r| vec2(r.x - pp.x, r.z - pp.z).length() <= REST_REACH);
        // 쉴 수 있나: 싸우는 중인 적이 가까우면 막는다 — 제자리에서 제 일을 하는 적(0113)은 세지 않는다.
        self.rest_safe = !w.enemies.iter().any(|e| {
            e.active()
                && !matches!(e.state, EState::Idle | EState::Return)
                && vec2(e.pos.x - pp.x, e.pos.z - pp.z).length() <= REST_SAFE
        });
        // 상자(0064 M3): 곁의 닫힌 상자는 적이 가까워도 연다(쉬기와 달리 몸이 묶이지 않는다).
        self.chest.near = self.chest.at.iter().enumerate().position(|(i, c)| {
            !self.chest.open[i] && vec2(c.x - pp.x, c.z - pp.z).length() <= CHEST_REACH
        });
        // 도감(0130): 깨어난 적을 가까이(적 시야만큼) 보면 그 종·역할·성질·깨어남을 한 번씩 알린다.
        for e in w.enemies.iter().filter(|e| e.active() && e.alive()) {
            if vec2(e.pos.x - pp.x, e.pos.z - pp.z).length() > crate::codex::SEE {
                continue;
            }
            for k in crate::codex::seen_keys(e) {
                if !self.codex_told.contains(&k) {
                    self.codex_told.push(k.clone());
                    self.codex_new.push(k);
                }
            }
        }
        // 새긴 돌(0129): 곁이면 읽기(싸움 중에도 — 글 판은 멈추지 않는다).
        self.near_stone = self
            .stones
            .iter()
            .position(|s| vec2(s.x - pp.x, s.z - pp.z).length() <= STONE_REACH);
        // 떨어진 아이템(0131): 튀어나온 뒤 잠깐(`DROP_SETTLE`)은 줍지 못한다 — 여는 누름이 곧바로 줍지 않게.
        for d in &mut self.drops {
            d.2 += dt;
        }
        let reach = |p: &Vec3, age: f32| {
            age >= DROP_SETTLE && vec2(p.x - pp.x, p.z - pp.z).length() <= FIND_REACH
        };
        for d in &mut self.item_drops {
            d.age += dt;
            // 내려놓은 것: 한 번 곁을 떠나면 여느 떨어진 것과 같다.
            if d.wait && !reach(&d.pos, d.age.max(DROP_SETTLE)) {
                d.wait = false;
            }
        }
        // 가방이 가득이면(0226) 칸이 드는 것은 줍지 못한다 — 바닥에 남는다. 잔불이 될 상자의 것은 줍는다.
        let (full, block, sword) = (self.bag_full, self.chest_block, self.start_block);
        let held = |chest: usize| {
            full && if chest == START_SWORD {
                sword
            } else {
                chest < 64 && block >> chest & 1 == 1
            }
        };
        self.near_drop = self
            .drops
            .iter()
            .position(|(p, chest, age)| reach(p, *age) && !held(*chest));
        let by_item = self
            .item_drops
            .iter()
            .position(|d| !d.wait && reach(&d.pos, d.age));
        self.near_item = by_item.filter(|_| !full);
        self.near_blocked = (full && by_item.is_some())
            || self
                .drops
                .iter()
                .any(|(p, chest, age)| reach(p, *age) && held(*chest));
        // 숨은 물건(0105): 상자와 같은 꼴 — 곁이면 점프 버튼이 줍기.
        self.hidden.near = self
            .hidden
            .list
            .iter()
            .enumerate()
            .position(|(i, (p, _, _))| {
                !self.hidden.found[i] && vec2(p.x - pp.x, p.z - pp.z).length() <= FIND_REACH
            });
        // 문 장치(0113): 문이 닫혀 있을 때만 곁이면 당기기.
        self.near_lever = !self.gate_opened
            && self
                .lever
                .is_some_and(|l| vec2(l.x - pp.x, l.z - pp.z).length() <= LEVER_REACH);
        // 주민(0122): 보면 기록, 가까이 오면 돌아보고(무심한 경계 — 천천히), 멀어지면 제 일 쪽으로. 곁이면 말 걸기.
        self.folk.near = None;
        for (k, &(_, at)) in self.folk.list.iter().enumerate() {
            let d = flat(at);
            if d <= crate::residents::SEE_RANGE && !self.folk.seen[k] {
                self.folk.seen[k] = true;
                self.folk.seen_new.push(k);
            }
            let home = crate::residents::WORK_FACE;
            let want = if d <= crate::residents::GLANCE_RANGE {
                (pp.x - at.x).atan2(pp.z - at.z)
            } else {
                home
            };
            let f = &mut self.folk.face[k];
            let diff = (want - *f + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
                - std::f32::consts::PI;
            *f += diff.clamp(-1.5 * dt, 1.5 * dt);
            if d <= crate::residents::TALK_RANGE && self.folk.near.is_none() {
                self.folk.near = Some(k);
            }
        }
        // 점프 버튼이 다음 틱부터 쉬기·열기·줍기가 되는 자리(0048): 곁이고(쉬기는 안전할 때만) — 아니면 그대로 점프.
        w.interact_here = (self.near_rest.is_some() && self.rest_safe)
            || self.near_drop.is_some()
            || self.near_item.is_some()
            || self.chest.near.is_some()
            || self.hidden.near.is_some()
            || self.near_lever
            || self.folk.near.is_some()
            || self.near_stone.is_some();
        // 같은 버튼: 줍기(떨어진 것 · 숨은 물건) > 열기 > 당기기 > 쉬기(HUD 안내와 같은 차례).
        if let (Some(k), true) = (self.near_drop, w.interact) {
            let (_, chest, _) = self.drops.remove(k);
            self.looted = Some(chest);
            self.near_drop = None;
        } else if let (Some(k), true) = (self.near_item, w.interact) {
            let d = self.item_drops.remove(k);
            self.taken = Some(d.item);
            self.taken_at = Some(d.pos);
            self.near_item = None;
        } else if let (Some(i), true) = (self.hidden.near, w.interact) {
            self.hidden.found[i] = true;
            self.hidden.picked = Some(i);
        } else if let (Some(i), true) = (self.chest.near, w.interact) {
            self.chest.open[i] = true;
            self.chest.opened = Some(i);
            // 상자 앞(방랑자 쪽)으로 튀어나와 떨어진다(0131) — 기록은 주울 때.
            let c = self.chest.at[i];
            // 방랑자 쪽에서 옆으로 비껴(발밑에 떨어지면 몸에 가려 빛기둥이 안 보였다, 헤드리스).
            let to = vec3(pp.x - c.x, 0.0, pp.z - c.z).normalize_or(Vec3::Z);
            let side = Quat::from_rotation_y(DROP_SIDE.to_radians()) * to;
            self.drops.push((c + side * DROP_OUT, i, 0.0));
        } else if let (true, true) = (self.near_lever, w.interact) {
            self.gate_opened = true;
            self.gate_new = true;
            self.pulled = true;
            self.near_lever = false;
            self.set_gate(w, false);
            if self.phase == Phase::Entrance {
                self.phase = Phase::GateOpen;
                self.objective = Some(self.def.obj_gate);
            }
            self.card = Some((self.def.card_lever, CARD_TIME));
        } else if let (Some(k), true) = (self.folk.near, w.interact) {
            // 말 걸기(0122): 멈추지 않는다 — 앱이 머리 위 말풍선을 띄운다(다시 누르면 다음 줄).
            self.folk.talked = Some(k);
        } else if let (Some(k), true) = (self.near_stone, w.interact) {
            self.read = Some(k);
        } else if let (Some(i), true) = (self.near_rest, w.interact) {
            if self.rest_safe {
                // 꺼진 화로(0171)에서 처음 쉰다: 불이 붙는다 — 지형이 다시 구워진다(빛·불꽃).
                if self
                    .level
                    .cold
                    .is_some_and(|c| c.distance(self.rests[i]) < 0.1)
                {
                    self.level.cold = None;
                    self.kindled = true;
                }
                self.rest(w, i);
                self.rested = true;
            } else {
                self.card = Some((|s| s.rest_blocked, 1.6));
            }
        }
        // 구역 카드: 시작 이야기가 끝나면 첫 구역, 그다음은 새 구역에 처음 들어설 때.
        let z = self.zone_at(w.player.pos);
        if !self.first_card && self.intro <= 0.0 {
            self.first_card = true;
            self.card = Some((self.def.zones[self.zone].0, CARD_TIME));
        }
        if z > self.zone {
            self.zone = z;
            self.card = Some((self.def.zones[z].0, CARD_TIME));
        }
        // 서막 숲 끝(0262): 표지를 밟으면 성소로.
        if self.def.prologue && self.exit_at.is_some_and(|e| flat(e) <= EXIT_REACH) {
            return StageOut::Onward;
        }
        match self.phase {
            // 물결 스테이지(시험·기믹)만 둘째 구역에 들어서면 물결 — 서 있는 적 스테이지는 문 장치가 문을 연다(0113).
            Phase::Entrance => {
                if z >= 1 && !self.def.waves.is_empty() {
                    self.phase = Phase::Waves(0);
                    self.spawn_wave(w, 0);
                    self.objective = Some(self.def.obj_waves);
                }
            }
            Phase::Waves(n) => {
                if self.alive_in_wave(w) <= WAVE_LEFT && n + 1 < self.def.waves.len() {
                    let keep: Vec<usize> = self.wave_ids.clone();
                    self.phase = Phase::Waves(n + 1);
                    self.spawn_wave(w, n + 1);
                    self.wave_ids.extend(keep);
                } else if self.alive_in_wave(w) == 0 {
                    self.phase = Phase::GateOpen;
                    self.set_gate(w, false);
                    self.objective = Some(self.def.obj_gate);
                }
            }
            Phase::GateOpen => {
                // 마지막 구역(성소)에 들어서면.
                if z + 1 >= self.def.zones.len() {
                    // 문이 닫히고 문지기가 깨어난다(플레이어가 문 안으로 다 들어온 뒤 닫는다).
                    let inside = self.def.shortcut
                        || self
                            .level
                            .gate
                            .is_none_or(|(g, _)| w.player.pos.z < g.min.z - 0.8);
                    if inside {
                        self.set_gate(w, true);
                        // 깨어나는 동안에도 락온에는 잡힌다(0149).
                        if let Some(b) = self.boss.and_then(|i| w.enemies.get_mut(i)) {
                            b.waking = true;
                        }
                        self.phase = Phase::Waking(0.0);
                        self.objective = None;
                        self.card = Some((self.def.boss_wakes, WAKE_TIME + 1.0));
                    }
                }
            }
            Phase::Waking(t) => {
                if t + dt >= WAKE_TIME {
                    if let Some(b) = self.boss.and_then(|i| w.enemies.get_mut(i)) {
                        b.state = EState::Chase;
                        b.waking = false;
                    }
                    self.phase = Phase::Duel;
                } else {
                    self.phase = Phase::Waking(t + dt);
                }
            }
            Phase::Duel => {
                let dead = self
                    .boss
                    .and_then(|i| w.enemies.get(i))
                    .is_none_or(|b| !b.alive());
                if dead {
                    self.core = self.boss.and_then(|i| w.enemies.get(i)).map(|b| b.pos);
                    self.boss_down = self.core;
                    self.phase = Phase::Core;
                    self.objective = Some(self.def.obj_core);
                    w.lock = None;
                }
            }
            Phase::Core => {
                // 처음 이긴 문지기의 핵은 손을 뻗으면 스스로 다가온다(lore CANON, 0101·0105) — 그다음부터는 평소대로 거둔다.
                if let (Some(c), true) = (self.core.as_mut(), self.first_core) {
                    let to = vec3(w.player.pos.x - c.x, 0.0, w.player.pos.z - c.z);
                    let d = to.length();
                    if d <= CORE_DRAW && d > 1e-3 {
                        *c += to / d * (CORE_DRAW_SPEED * dt).min(d);
                    }
                }
                if let Some(c) = self.core {
                    let d = vec2(c.x - w.player.pos.x, c.z - w.player.pos.z).length();
                    if d <= CORE_REACH {
                        // 아래 지역이 있으면 바닥 길이 열린다(0118) — 판은 이어진다.
                        if self.down_at.is_some() {
                            self.open_down(w);
                            self.card = Some((|s| s.card_way_down, CARD_TIME));
                            return StageOut::CoreTaken;
                        }
                        self.phase = Phase::Cleared;
                        self.objective = None;
                        return StageOut::Cleared;
                    }
                }
            }
            Phase::Down => {
                if self.passage_armed && near(self.down_at) {
                    return StageOut::Descend;
                }
            }
            Phase::Cleared => return StageOut::Cleared,
        }
        // 오르는 길(0118): 문지기와 싸우는 동안 말고는 언제나.
        if self.passage_armed
            && near(self.up_at)
            && !matches!(self.phase, Phase::Waking(_) | Phase::Duel)
        {
            return StageOut::Ascend;
        }
        StageOut::Playing
    }

    /// 쉬기(0032): 체력·등잔·스태미나 가득, 이 쉼터가 다시 일어설 곳. 되돌아갈 단계 = 지금(물결 중이면 물결 전부터).
    fn rest(&mut self, w: &mut World, i: usize) {
        let full = w.stats.max_stamina();
        let pl = &mut w.player;
        pl.hp = pl.max_hp;
        pl.flask = pl.flask_max;
        pl.ranged.mana = crate::tuning::CAST_CHARGES;
        pl.stamina = full;
        let back = match self.phase {
            Phase::Entrance | Phase::Waves(_) => Phase::Entrance,
            _ => Phase::GateOpen,
        };
        self.rest_at = Some((i, back));
        // 화로에서 쉬면 서 있는 적이 다시 선다(0113, 소울 화톳불 — 엘리트·보스 빼고).
        self.populate(w);
        self.since_rest = 0.0;
        self.card = Some((|s| s.rested, 2.4));
    }

    /// 쓰러진 뒤 쉼터에서 다시(0032): 플레이어는 쉼터 곁에 가득 찬 채로, 진행은 쉬던 때로 — 물결은 처음부터(뜰에 들어서면
    /// 다시 온다), 문지기는 잠든 채 가득. 문지기만 남기고 다른 적은 지운다(문지기는 시작할 때 처음 세워 번호 0).
    fn respawn(&mut self, w: &mut World, i: usize, back: Phase) {
        let at = self.rests[i];
        // 이 판에 이긴 문지기는 다시 서지 않는다(0118).
        if self.down_open {
            w.enemies.retain(|e| !e.boss);
        }
        // 기록의 문(0113)이 열려 있으면 문 앞 단계로.
        let back = if self.gate_opened {
            Phase::GateOpen
        } else {
            back
        };
        w.enemies.retain(|e| e.boss);
        self.boss = (!w.enemies.is_empty()).then_some(0);
        if let (Some(b), Some(home)) = (self.boss, self.level.marks_of(self.def.boss.0).next()) {
            let max = w.enemies[b].max_hp;
            let e = &mut w.enemies[b];
            e.hp = max;
            e.state = EState::Dormant;
            e.waking = false;
            e.roused = false;
            e.exposed = 0.0;
            // 타입 상태(0223 V2)도 처음부터 — 지난 싸움의 화상·금·닳은 게이지가 다시 선 문지기에 남지 않게.
            e.ail = Default::default();
            e.knock = Vec3::ZERO;
            e.pos = home;
            e.prev = home;
        }
        self.wave_ids.clear();
        self.elite_ids.clear();
        w.lingers.clear();
        w.shards.clear();
        w.lock = None;
        w.hitstop = 0.0;
        let pl = &mut w.player;
        pl.pos = at + vec3(0.0, 0.0, 1.4);
        pl.prev = pl.pos;
        pl.vel = Vec3::ZERO;
        pl.state = PState::Idle;
        pl.air = 0.0;
        pl.hurt.left = 1.0;
        self.rest(w, i);
        self.since_rest = f32::MAX;
        match back {
            Phase::Entrance => {
                self.phase = Phase::Entrance;
                self.set_gate(w, true);
                self.objective = self.def.waves.is_empty().then_some(self.def.obj_explore);
            }
            _ => {
                self.phase = Phase::GateOpen;
                self.set_gate(w, false);
                self.objective = Some(self.def.obj_gate);
            }
        }
        self.rest_at = Some((i, back));
        self.zone = self.zone_at(w.player.pos);
        self.core = None;
        if self.down_open {
            self.open_down(w);
        }
        self.passage_armed = false;
        self.card = Some((|s| s.respawned, CARD_TIME));
    }

    /// 이번 틱에 말 건 주민(0122) — 앱이 한 번 가져간다.
    /// 이번 틱까지 도감에 새로 본 이름(0130).
    pub fn take_codex(&mut self) -> Vec<String> {
        std::mem::take(&mut self.codex_new)
    }

    /// 이번 틱에 읽은 새긴 돌(0129 — 앱이 글 판을 띄운다).
    pub fn take_read(&mut self) -> Option<usize> {
        self.read.take()
    }

    pub fn take_talked(&mut self) -> Option<usize> {
        self.folk.talked.take()
    }

    /// 이 판에 처음 본 주민들(0122) — 앱이 한 번 가져가 기록에.
    pub fn take_residents_seen(&mut self) -> Vec<usize> {
        std::mem::take(&mut self.folk.seen_new)
    }

    /// 깨어난 자리의 화로를 끈다(0171 — 아직 불을 붙인 적 없는 기록의 첫 지역): 시작 자리에서 가장 가까운 쉼터.
    pub fn chill_hearth(&mut self) {
        let from = self.level.player_start;
        self.level.cold = self
            .rests
            .iter()
            .copied()
            .min_by(|a, b| a.distance(from).total_cmp(&b.distance(from)));
    }

    /// 이번 틱에 꺼진 화로에 불이 붙었나(0171) — 앱이 한 번 가져간다.
    pub fn take_kindled(&mut self) -> bool {
        std::mem::take(&mut self.kindled)
    }

    /// 이번 틱에 쉬었나(0064 M4) — 앱이 한 번 가져가 화로 가게를 연다(쓰러진 뒤 다시 서는 것은 아니다).
    pub fn take_rested(&mut self) -> bool {
        std::mem::take(&mut self.rested)
    }

    /// 이번 틱에 연 상자(0064 M3) — 앱이 한 번 가져가 여는 소리를 낸다(0131부터 얻는 것은 주울 때 — `take_looted`).
    pub fn take_opened(&mut self) -> Option<usize> {
        self.chest.opened.take()
    }

    /// 이번 틱에 주운 떨어진 아이템의 상자 번호(0131) — 앱이 그 상자의 것을 주고 기록한다.
    /// 깨어난 자리 곁에 검을 놓는다(0220 — 앱이 그 기록이 아직 검이 없을 때만 부른다). 곧바로 주울 수 있다.
    pub fn lay_start_sword(&mut self) {
        let at = self.level.player_start + START_SWORD_AT;
        if !self.drops.iter().any(|d| d.1 == START_SWORD) {
            self.drops.push((at, START_SWORD, DROP_SETTLE));
        }
    }

    pub fn take_looted(&mut self) -> Option<usize> {
        self.looted.take()
    }

    /// 아이템을 바닥에 떨군다(0226): 엘리트가 쓰러진 자리·핵을 거둔 자리·방랑자가 내려놓은 발치(`wait` — 한 번 곁을 떠나야
    /// 줍는다). 떨어지는 동안(`DROP_SETTLE`)은 줍지 못한다. 같은 자리에 겹치면 옆으로 비킨다. 높이는 부른 쪽이 땅에 맞춘다.
    pub fn drop_item(&mut self, at: Vec3, item: crate::items::Item, wait: bool) {
        let n = self
            .item_drops
            .iter()
            .filter(|d| vec2(d.pos.x - at.x, d.pos.z - at.z).length() < DROP_APART * 3.0)
            .count();
        let turn = Quat::from_rotation_y(n as f32 * 2.4);
        let pos = if n == 0 {
            at
        } else {
            at + turn * Vec3::X * DROP_APART
        };
        self.item_drops.push(ItemDrop {
            pos,
            item,
            age: 0.0,
            wait,
        });
        self.item_serial = self.item_serial.wrapping_add(1);
    }

    /// 기록에서 되놓는다(0226 — 이어하기·지역에 다시 들어설 때): 이미 내려앉은 채, 그 자리 그대로.
    pub fn lay_item(&mut self, pos: Vec3, item: crate::items::Item) {
        self.item_drops.push(ItemDrop {
            pos,
            item,
            age: DROP_SETTLE,
            wait: false,
        });
    }

    /// 이번 틱에 주운 아이템(0226)과 그 자리 — 앱이 한 번 가져가 가방에 넣는다.
    pub fn take_item(&mut self) -> Option<(crate::items::Item, Vec3)> {
        let item = self.taken.take()?;
        Some((item, self.taken_at.take().unwrap_or_default()))
    }

    /// 이번 틱에 쓰러진 문지기의 자리(0226) — 앱이 한 번 가져가 그 곁에 떨굼을 놓는다.
    pub fn take_boss_down(&mut self) -> Option<Vec3> {
        self.boss_down.take()
    }

    /// 이 판에 떨어뜨린 아이템 수(떨굼의 씨앗에).
    pub fn item_serial(&self) -> u32 {
        self.item_serial
    }

    /// 이번 틱에 주운 숨은 물건(0105): (종류, 기록 이름) — 앱이 한 번 가져가 기록·능력치에.
    pub fn take_picked(&mut self) -> Option<(Find, String)> {
        let i = self.hidden.picked.take()?;
        self.hidden.list.get(i).map(|(_, k, id)| (*k, id.clone()))
    }

    /// 기록의 주운 숨은 물건들(판을 넘어 없다).
    pub fn set_found(&mut self, ids: &[String]) {
        for (i, (_, _, id)) in self.hidden.list.iter().enumerate() {
            if ids.contains(id) {
                self.hidden.found[i] = true;
            }
        }
    }

    /// 기록의 연 상자들(판을 넘어 비어 있다).
    pub fn set_opened(&mut self, opened: &[u32]) {
        for &i in opened {
            if let Some(o) = self.chest.open.get_mut(i as usize) {
                *o = true;
            }
        }
    }

    /// 기록할 판 안의 자리(0064 M2): 마지막으로 쉰 쉼터와 되돌아갈 단계. 쉰 적이 없으면 없다(다음은 입구부터).
    pub fn run_at(&self) -> Option<crate::save::RunAt> {
        self.rest_at.map(|(i, back)| crate::save::RunAt {
            rest: i,
            gate_open: back != Phase::Entrance || self.gate_opened,
            ..Default::default()
        })
    }

    /// 이어하기(0064 M2): 시작 이야기 없이 기록의 쉼터에서 다시 선다 — 쓰러진 뒤 다시 서는 것과 같은 규칙(0032),
    /// 글만 "쉬었다". 기록의 쉼터 번호가 지도에 없으면(지도가 바뀌었다) 입구부터.
    pub fn resume(&mut self, w: &mut World, at: crate::save::RunAt) {
        if at.rest >= self.rests.len() {
            return;
        }
        self.intro = 0.0;
        self.first_card = true;
        // 옛 기록(물결 시절, 0113 전): 판 안의 "문이 열렸다"는 이제 기록의 문 — 한 번 옮겨 둔다.
        if at.gate_open && !self.gate_opened && self.def.waves.is_empty() {
            self.gate_opened = true;
            self.gate_new = true;
        }
        let back = if at.gate_open || self.gate_opened {
            Phase::GateOpen
        } else {
            Phase::Entrance
        };
        self.respawn(w, at.rest, back);
        self.card = Some((self.def.zones[self.zone].0, CARD_TIME));
    }

    /// 개발: 물결·서 있는 적을 건너뛰고 문 앞으로(문이 열린 채) — 보스만 다시 볼 때. 보스 말고는 지운다(0113 — 문 앞 가시짐승이 바로 덮쳤다).
    #[cfg(feature = "dev")]
    pub fn dev_skip_to_gate(&mut self, w: &mut World) {
        w.enemies.retain(|e| e.boss);
        self.boss = (!w.enemies.is_empty()).then_some(0);
        self.elite_ids.clear();
        w.lock = None;
        self.intro = 0.0;
        self.first_card = true;
        self.phase = Phase::GateOpen;
        self.objective = Some(self.def.obj_gate);
        self.set_gate(w, false);
        if let Some((g, _)) = self.level.gate {
            w.player.pos = vec3(g.center().x, 0.0, g.max.z + 4.0);
            w.player.prev = w.player.pos;
        }
        self.zone = self.def.zones.len().saturating_sub(2);
    }

    /// 보스 체력 띠를 띄울까(깨어나는 중부터 쓰러질 때까지).
    pub fn boss_bar(&self, w: &World) -> Option<(f32, bool)> {
        match self.phase {
            Phase::Waking(_) | Phase::Duel => self
                .boss
                .and_then(|i| w.enemies.get(i))
                .map(|b| (b.hp.max(0) as f32 / b.max_hp as f32, b.roused)),
            _ => None,
        }
    }

    /// 지금 가야 할 곳(0060 W1 길 안내): 문이 열렸으면 문, 핵이 남았으면 핵. 싸우는 동안·입구 길은 없음 — 그때는 바깥에서
    /// 얹은 갈 곳(`guide`, 0170 — 튜토리얼의 쉼터)이 있으면 그것.
    /// 렌더는 여기에 호박색 빛기둥을, HUD는 화면 밖이면 가장자리 화살을 그린다.
    pub fn goal(&self) -> Option<Vec3> {
        match self.phase {
            Phase::GateOpen => self.level.gate.map(|(_, c)| c),
            Phase::Core => self.core,
            Phase::Down => self.down_at,
            _ => None,
        }
        .or(self.guide)
    }

    /// 종소리가 들려오는 곳(0171 — 세계 법칙 D): 닫힌 문의 장치를 찾는 동안은 그 장치. 문이 열리면 갈 곳(`goal`)이 맡는다.
    pub fn bell_from(&self) -> Option<Vec3> {
        // 서막 숲(0262): 숲 끝의 표지 — 종소리를 따라 성소로 걸어 나온다.
        if self.def.prologue {
            return self.exit_at;
        }
        (self.phase == Phase::Entrance && !self.gate_opened)
            .then_some(self.lever)
            .flatten()
    }

    /// 문지기가 깨어나는 진행(0..1, 렌더의 `wake` 동작). 깨어나기 전 0, 다 깨면 1.
    pub fn wake_progress(&self) -> f32 {
        match self.phase {
            Phase::Entrance | Phase::Waves(_) | Phase::GateOpen => 0.0,
            Phase::Waking(t) => (t / WAKE_TIME).clamp(0.0, 1.0),
            _ => 1.0,
        }
    }
}

#[cfg(test)]
mod tests;
