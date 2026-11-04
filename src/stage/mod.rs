//! 스테이지(0019): 지도(글자 격자) → 레벨(벽·소품·불빛·스폰·문) + 진행 대본(구역·물결·문·보스·핵). 새 스테이지 = 파일 하나
//! (`StageDef` 한 개). 진행은 월드 밖에서 월드를 지켜보며 적을 세우고 문을 여닫는다 — 월드는 스테이지를 모른다(경계 그대로).
//! 앱은 `StageRun::tick`의 결과(`StageOut`)로 장면만 바꾼다(흐름은 `app::flow`의 순수 함수).
pub mod sanctuary;

use macroquad::prelude::*;

use crate::i18n::Tr;
use crate::species::Kind;
use crate::tuning::{FLASK_CHARGES, PLAYER_HP};
use crate::world::{Aabb, EState, PState, World};

/// 지도 한 칸(m).
pub const CELL: f32 = 2.0;
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
    /// 문(닫히면 막는 상자)과 그 가운데.
    pub gate: Option<(Aabb, Vec3)>,
}

impl Level {
    pub fn cell(&self, x: usize, z: usize) -> u8 {
        if x < self.w && z < self.h {
            self.cells[z * self.w + x]
        } else {
            b' '
        }
    }

    /// 칸 가운데의 월드 자리(지도 가운데 = 원점, 행이 늘면 +z).
    pub fn at(&self, x: usize, z: usize) -> Vec3 {
        vec3(
            (x as f32 - self.w as f32 * 0.5 + 0.5) * CELL,
            0.0,
            (z as f32 - self.h as f32 * 0.5 + 0.5) * CELL,
        )
    }

    pub fn marks_of(&self, c: char) -> impl Iterator<Item = Vec3> + '_ {
        self.marks.iter().filter(move |m| m.0 == c).map(|m| m.1)
    }

    /// 월드 자리 → 칸 행(z).
    pub fn row_of(&self, p: Vec3) -> f32 {
        p.z / CELL + self.h as f32 * 0.5 - 0.5
    }

    /// 아레나 반폭(월드의 경계 — 벽이 안쪽에서 막는다).
    pub fn half_extent(&self) -> f32 {
        (self.w.max(self.h) as f32) * CELL * 0.5
    }

    /// 글자 지도 → 레벨. 표지: `#` 벽 · `.` 바닥 · ` ` 바깥 · `P` 시작 · `G` 문 · `i` 기둥 · `r` 뿌리 · `b` 화로 · `R` 쉼터 화로(0032, 화로 + 표지) · `u` 돌무더기 ·
    /// `w` 무너진 벽 조각 · `L` 종 · 그 밖의 글자는 바닥 + 표지(스폰 등, `marks`).
    pub fn parse(map: &str) -> Level {
        let rows: Vec<&str> = map.lines().filter(|l| !l.trim().is_empty()).collect();
        let h = rows.len();
        let w = rows.iter().map(|r| r.chars().count()).max().unwrap_or(0);
        let mut lv = Level {
            w,
            h,
            cells: vec![b' '; w * h],
            walls: Vec::new(),
            props: Vec::new(),
            lights: Vec::new(),
            marks: Vec::new(),
            player_start: Vec3::ZERO,
            gate: None,
        };
        let mut gate_cells = Vec::new();
        for (z, row) in rows.iter().enumerate() {
            for (x, c) in row.chars().enumerate() {
                let p = lv.at(x, z);
                let cell = match c {
                    '#' => b'#',
                    ' ' => b' ',
                    _ => b'.',
                };
                lv.cells[z * w + x] = cell;
                let prop = |kind| Prop {
                    kind,
                    pos: p,
                    yaw: ((x * 7 + z * 13) % 8) as f32 * std::f32::consts::FRAC_PI_4,
                };
                match c {
                    '#' | '.' | ' ' => {}
                    'P' => lv.player_start = p,
                    'G' => gate_cells.push(p),
                    'i' => lv.props.push(prop(PropKind::Pillar)),
                    'r' => lv.props.push(prop(PropKind::Root)),
                    'u' => lv.props.push(prop(PropKind::Rubble)),
                    'w' => lv.props.push(Prop {
                        kind: PropKind::WallPiece,
                        pos: p,
                        yaw: 0.0,
                    }),
                    'L' => lv.props.push(Prop {
                        kind: PropKind::Bell,
                        pos: p,
                        yaw: 0.0,
                    }),
                    'b' | 'R' => {
                        if c == 'R' {
                            lv.marks.push(('R', p));
                        }
                        lv.props.push(Prop {
                            kind: PropKind::Brazier,
                            pos: p,
                            yaw: 0.0,
                        });
                        lv.lights.push(Light {
                            pos: p + Vec3::Y * 1.2,
                            color: Color::new(1.0, 0.62, 0.25, 1.0),
                            radius: 7.0,
                        });
                    }
                    other => lv.marks.push((other, p)),
                }
            }
        }
        // 벽: 줄마다 이어진 '#'를 상자 하나로(부딪힘 상자 수를 줄인다).
        for z in 0..h {
            let mut x = 0;
            while x < w {
                if lv.cell(x, z) == b'#' {
                    let x0 = x;
                    while x < w && lv.cell(x, z) == b'#' {
                        x += 1;
                    }
                    let a = lv.at(x0, z);
                    let b = lv.at(x - 1, z);
                    lv.walls.push(Aabb {
                        min: vec3(a.x - CELL * 0.5, 0.0, a.z - CELL * 0.5),
                        max: vec3(b.x + CELL * 0.5, WALL_H, b.z + CELL * 0.5),
                    });
                } else {
                    x += 1;
                }
            }
        }
        if !gate_cells.is_empty() {
            let min_x = gate_cells.iter().map(|p| p.x).fold(f32::MAX, f32::min) - CELL * 0.5;
            let max_x = gate_cells.iter().map(|p| p.x).fold(f32::MIN, f32::max) + CELL * 0.5;
            let z = gate_cells[0].z;
            lv.gate = Some((
                Aabb {
                    min: vec3(min_x, 0.0, z - 0.4),
                    max: vec3(max_x, 4.5, z + 0.4),
                },
                vec3((min_x + max_x) * 0.5, 0.0, z),
            ));
        }
        lv
    }

    /// 부딪힘 상자: 벽 + 소품(+ 닫힌 문은 따로).
    pub fn obstacles(&self) -> Vec<Aabb> {
        let mut v = self.walls.clone();
        for p in &self.props {
            let h = p.kind.half();
            if h != Vec2::ZERO {
                v.push(Aabb {
                    min: vec3(p.pos.x - h.x, 0.0, p.pos.z - h.y),
                    max: vec3(p.pos.x + h.x, p.kind.height(), p.pos.z + h.y),
                });
            }
        }
        v
    }
}

/// 스테이지 정의(데이터).
pub struct StageDef {
    pub name: Tr,
    pub map: &'static str,
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
}

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
    Cleared,
}

/// 진행 결과(앱이 장면을 바꾸는 데만 쓴다).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StageOut {
    Playing,
    Cleared,
    Defeated,
    /// 쓰러졌지만 마지막으로 쉰 쉼터에서 다시 일어섰다(0032) — 앱은 카메라만 그 자리로.
    Respawned,
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
/// 이름 카드가 떠 있는 시간(초).
pub const CARD_TIME: f32 = 3.0;

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
    /// 마지막으로 쉰 쉼터와, 쓰러지면 되돌아갈 진행 단계(쉰 때의 단계 — 물결 전이면 입구, 물결을 끝냈으면 문 앞).
    pub rest_at: Option<(usize, Phase)>,
    /// 곁에 있는 쉼터(HUD·터치 버튼), 지금 쉴 수 있나(가까운 깨어 있는 적이 없다).
    pub near_rest: Option<usize>,
    pub rest_safe: bool,
    /// 마지막으로 쉰 뒤 지난 초(렌더의 퍼지는 빛).
    pub since_rest: f32,
}

/// 시작 이야기 길이(초).
pub const INTRO_TIME: f32 = 6.0;

impl StageRun {
    /// 레벨을 세우고 월드를 만든다.
    pub fn start(def: &'static StageDef) -> (StageRun, World) {
        let level = Level::parse(def.map);
        let mut w = World::new("stage", level.half_extent());
        w.obstacles = level.obstacles();
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
            intro: INTRO_TIME,
            wave_ids: Vec::new(),
            boss: None,
            gate_open: 0.0,
            gate_closed: true,
            core: None,
            time: 0.0,
            first_card: false,
            rests: Vec::new(),
            rest_at: None,
            near_rest: None,
            rest_safe: true,
            since_rest: f32::MAX,
        };
        run.rests = run.level.marks_of('R').collect();
        // 보스는 처음부터 잠든 채 성소에 있다(보인다 — 기대감).
        if let Some(p) = run.level.marks_of(def.boss.0).next() {
            let i = w.spawn(def.boss.1, p);
            w.enemies[i].state = EState::Dormant;
            run.boss = Some(i);
        }
        (run, w)
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
        if let Some(list) = self.def.waves.get(n) {
            for (mark, kind) in list.iter() {
                let spots: Vec<Vec3> = self.level.marks_of(*mark).collect();
                for p in spots {
                    self.wave_ids.push(w.spawn(*kind, p));
                }
            }
        }
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
        self.near_rest = self
            .rests
            .iter()
            .position(|r| vec2(r.x - pp.x, r.z - pp.z).length() <= REST_REACH);
        self.rest_safe = !w
            .enemies
            .iter()
            .any(|e| e.active() && vec2(e.pos.x - pp.x, e.pos.z - pp.z).length() <= REST_SAFE);
        if let (Some(i), true) = (self.near_rest, w.interact) {
            if self.rest_safe {
                self.rest(w, i);
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
        match self.phase {
            Phase::Entrance => {
                if z >= 1 {
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
                if z >= 2 {
                    // 문이 닫히고 문지기가 깨어난다(플레이어가 문 안으로 다 들어온 뒤 닫는다).
                    let inside = self
                        .level
                        .gate
                        .is_none_or(|(g, _)| w.player.pos.z < g.min.z - 0.8);
                    if inside {
                        self.set_gate(w, true);
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
                    self.phase = Phase::Core;
                    self.objective = Some(self.def.obj_core);
                    w.lock = None;
                }
            }
            Phase::Core => {
                if let Some(c) = self.core {
                    let d = vec2(c.x - w.player.pos.x, c.z - w.player.pos.z).length();
                    if d <= CORE_REACH {
                        self.phase = Phase::Cleared;
                        self.objective = None;
                        return StageOut::Cleared;
                    }
                }
            }
            Phase::Cleared => return StageOut::Cleared,
        }
        StageOut::Playing
    }

    /// 쉬기(0032): 체력·등잔·스태미나 가득, 이 쉼터가 다시 일어설 곳. 되돌아갈 단계 = 지금(물결 중이면 물결 전부터).
    fn rest(&mut self, w: &mut World, i: usize) {
        let pl = &mut w.player;
        pl.hp = PLAYER_HP;
        pl.flask = FLASK_CHARGES;
        pl.stamina = crate::tuning::SPECS[crate::tuning::idx(crate::tuning::P::StaminaMax)].def;
        let back = match self.phase {
            Phase::Entrance | Phase::Waves(_) => Phase::Entrance,
            _ => Phase::GateOpen,
        };
        self.rest_at = Some((i, back));
        self.since_rest = 0.0;
        self.card = Some((|s| s.rested, 2.4));
    }

    /// 쓰러진 뒤 쉼터에서 다시(0032): 플레이어는 쉼터 곁에 가득 찬 채로, 진행은 쉬던 때로 — 물결은 처음부터(뜰에 들어서면
    /// 다시 온다), 문지기는 잠든 채 가득. 문지기만 남기고 다른 적은 지운다(문지기는 시작할 때 처음 세워 번호 0).
    fn respawn(&mut self, w: &mut World, i: usize, back: Phase) {
        let at = self.rests[i];
        w.enemies.retain(|e| e.boss);
        self.boss = (!w.enemies.is_empty()).then_some(0);
        if let (Some(b), Some(home)) = (self.boss, self.level.marks_of(self.def.boss.0).next()) {
            let max = w.enemies[b].max_hp;
            let e = &mut w.enemies[b];
            e.hp = max;
            e.state = EState::Dormant;
            e.roused = false;
            e.exposed = 0.0;
            e.knock = Vec3::ZERO;
            e.pos = home;
            e.prev = home;
        }
        self.wave_ids.clear();
        w.lock = None;
        w.hitstop = 0.0;
        let pl = &mut w.player;
        pl.pos = at + vec3(0.0, 0.0, 1.4);
        pl.prev = pl.pos;
        pl.vel = Vec3::ZERO;
        pl.state = PState::Idle;
        pl.air = 0.0;
        pl.hurt = 1.0;
        self.rest(w, i);
        self.since_rest = f32::MAX;
        match back {
            Phase::Entrance => {
                self.phase = Phase::Entrance;
                self.set_gate(w, true);
                self.objective = None;
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
        self.card = Some((|s| s.respawned, CARD_TIME));
    }

    /// 개발: 물결을 건너뛰고 문 앞으로(문이 열린 채) — 보스만 다시 볼 때.
    #[cfg(feature = "dev")]
    pub fn dev_skip_to_gate(&mut self, w: &mut World) {
        self.intro = 0.0;
        self.first_card = true;
        self.phase = Phase::GateOpen;
        self.objective = Some(self.def.obj_gate);
        self.set_gate(w, false);
        if let Some((g, _)) = self.level.gate {
            w.player.pos = vec3(g.center().x, 0.0, g.max.z + 4.0);
            w.player.prev = w.player.pos;
        }
        self.zone = 1;
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
mod tests {
    use super::*;
    use crate::input::Actions;
    use crate::tuning::Tuning;
    use crate::tuning::{FLASK_CHARGES, PLAYER_HP};

    #[test]
    fn the_sanctuary_map_parses_into_walls_props_lights_gate_and_marks() {
        let lv = Level::parse(sanctuary::DEF.map);
        assert!(lv.w > 10 && lv.h > 20);
        assert!(!lv.walls.is_empty() && !lv.props.is_empty() && !lv.lights.is_empty());
        assert!(lv.gate.is_some());
        assert!(lv.marks_of('W').count() == 1);
        // 줄마다 폭이 같다(글자 지도 실수 잡기).
        let rows: Vec<&str> = sanctuary::DEF
            .map
            .lines()
            .filter(|l| !l.trim().is_empty())
            .collect();
        assert!(
            rows.iter().all(|r| r.chars().count() == lv.w),
            "ragged map rows"
        );
        // 시작 자리는 바닥.
        let start = lv.player_start;
        assert!(lv.walls.iter().all(|wb| !(start.x > wb.min.x
            && start.x < wb.max.x
            && start.z > wb.min.z
            && start.z < wb.max.z)));
    }

    /// 끝까지 걸어 본다(적을 지우는 손을 대신해 월드를 직접 만진다): 입구 → 물결 → 문 열림 → 성소(문 닫힘·깨어남) → 결투 → 핵 → 클리어.
    #[test]
    fn the_stage_script_runs_from_entrance_to_clear() {
        let t = Tuning::default();
        let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
        let step = |run: &mut StageRun, w: &mut World| {
            w.step(&Actions::default(), &t);
            run.tick(w, 1.0 / 60.0)
        };
        assert_eq!(run.phase, Phase::Entrance);
        assert!(run.gate_closed);
        // 뜰로.
        let court = run.level.at(run.level.w / 2, run.def.zones[1].1 - 2);
        w.player.pos = court;
        step(&mut run, &mut w);
        assert!(matches!(run.phase, Phase::Waves(0)));
        // 물결을 지운다(쓰러뜨린 셈).
        for _ in 0..run.def.waves.len() + 1 {
            for e in w.enemies.iter_mut().filter(|e| !e.boss) {
                e.state = EState::Dead;
            }
            w.player.hp = 5;
            step(&mut run, &mut w);
        }
        assert_eq!(run.phase, Phase::GateOpen);
        assert!(!run.gate_closed);
        // 성소 안으로.
        let (g, _) = run.level.gate.unwrap();
        w.player.pos = vec3(0.0, 0.0, g.min.z - 3.0);
        step(&mut run, &mut w);
        assert!(matches!(run.phase, Phase::Waking(_)));
        assert!(run.gate_closed);
        for _ in 0..((WAKE_TIME * 60.0) as usize + 2) {
            w.player.hp = 5;
            step(&mut run, &mut w);
        }
        assert_eq!(run.phase, Phase::Duel);
        let b = run.boss.unwrap();
        assert!(w.enemies[b].active(), "awake: {:?}", w.enemies[b].state);
        assert!(run.boss_bar(&w).is_some());
        w.enemies[b].state = EState::Dead;
        step(&mut run, &mut w);
        assert_eq!(run.phase, Phase::Core);
        w.player.pos = run.core.unwrap();
        assert_eq!(step(&mut run, &mut w), StageOut::Cleared);
    }

    /// 시험용: 한 틱(월드 → 스테이지).
    fn tick(run: &mut StageRun, w: &mut World, a: Actions) -> StageOut {
        w.step(&a, &Tuning::default());
        run.tick(w, 1.0 / 60.0)
    }

    fn rest_now(run: &mut StageRun, w: &mut World, i: usize) -> StageOut {
        w.player.pos = run.rests[i] + vec3(0.0, 0.0, 1.0);
        w.player.prev = w.player.pos;
        tick(
            run,
            w,
            Actions {
                interact: true,
                ..Default::default()
            },
        )
    }

    /// 입구 쪽 쉼터(시작 자리에서 가장 가까운 것).
    fn entrance_rest(run: &StageRun) -> usize {
        let s = run.level.player_start;
        (0..run.rests.len())
            .min_by(|a, b| {
                run.rests[*a]
                    .distance(s)
                    .total_cmp(&run.rests[*b].distance(s))
            })
            .unwrap()
    }

    /// 0032: 성소 지도에 쉼터 둘(입구·문 앞), 곁에서 상호작용 = 체력·등잔 가득 + 다시 일어설 곳.
    #[test]
    fn resting_refills_and_marks_the_rest_site() {
        let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
        assert_eq!(run.rests.len(), 2);
        let i = entrance_rest(&run);
        w.player.hp = 1;
        w.player.flask = 0;
        rest_now(&mut run, &mut w, i);
        assert_eq!(run.near_rest, Some(i));
        assert_eq!((w.player.hp, w.player.flask), (PLAYER_HP, FLASK_CHARGES));
        assert_eq!(run.rest_at, Some((i, Phase::Entrance)));
    }

    /// 0032: 깨어 있는 적이 `REST_SAFE` 안이면 쉴 수 없다(잠든 문지기는 괜찮다).
    #[test]
    fn you_cannot_rest_with_an_awake_enemy_close() {
        let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
        let i = entrance_rest(&run);
        w.spawn(Kind::Husk, run.rests[i] + vec3(3.0, 0.0, 0.0));
        w.player.hp = 1;
        rest_now(&mut run, &mut w, i);
        assert!(!run.rest_safe);
        assert_eq!(run.rest_at, None);
        assert_eq!(w.player.hp, 1);
    }

    /// 0032: 쉰 뒤 쓰러지면 끝이 아니라 그 쉼터에서 — 가득 찬 채, 물결은 처음부터(다시 뜰에 들어서면 온다), 문지기는 잠든 채.
    #[test]
    fn falling_after_a_rest_rises_at_the_rest_site_and_the_waves_start_over() {
        let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
        let i = entrance_rest(&run);
        rest_now(&mut run, &mut w, i);
        // 뜰로 → 물결.
        w.player.pos = run.level.at(run.level.w / 2, run.def.zones[1].1 - 2);
        tick(&mut run, &mut w, Actions::default());
        assert!(matches!(run.phase, Phase::Waves(0)));
        assert!(w.enemies.iter().any(|e| !e.boss));
        w.player.hp = 0;
        w.player.flask = 0;
        assert_eq!(
            tick(&mut run, &mut w, Actions::default()),
            StageOut::Respawned
        );
        assert_eq!(run.phase, Phase::Entrance);
        assert!(w.player.pos.distance(run.rests[i]) < REST_REACH);
        assert_eq!((w.player.hp, w.player.flask), (PLAYER_HP, FLASK_CHARGES));
        assert!(w.enemies.iter().all(|e| e.boss), "waves cleared away");
        assert_eq!(w.enemies[run.boss.unwrap()].state, EState::Dormant);
        assert!(run.gate_closed);
        // 다시 뜰에 들어서면 첫 물결이 다시 온다.
        w.player.pos = run.level.at(run.level.w / 2, run.def.zones[1].1 - 2);
        tick(&mut run, &mut w, Actions::default());
        assert!(matches!(run.phase, Phase::Waves(0)));
    }

    /// 0032: 물결을 끝내고 문 앞 쉼터에서 쉬었으면 결투에서 쓰러져도 문 앞부터 — 문은 열려 있고, 문지기는 잠든 채 가득.
    #[test]
    fn falling_in_the_duel_after_the_gate_rest_restarts_the_duel_only() {
        let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
        w.player.pos = run.level.at(run.level.w / 2, run.def.zones[1].1 - 2);
        tick(&mut run, &mut w, Actions::default());
        for _ in 0..run.def.waves.len() + 1 {
            for e in w.enemies.iter_mut().filter(|e| !e.boss) {
                e.state = EState::Dead;
            }
            tick(&mut run, &mut w, Actions::default());
        }
        assert_eq!(run.phase, Phase::GateOpen);
        let gate_rest = 1 - entrance_rest(&run);
        rest_now(&mut run, &mut w, gate_rest);
        assert_eq!(run.rest_at, Some((gate_rest, Phase::GateOpen)));
        // 성소로 → 깨어남 → 결투.
        let (g, _) = run.level.gate.unwrap();
        w.player.pos = vec3(0.0, 0.0, g.min.z - 3.0);
        for _ in 0..((WAKE_TIME * 60.0) as usize + 4) {
            w.player.hp = PLAYER_HP;
            tick(&mut run, &mut w, Actions::default());
        }
        assert_eq!(run.phase, Phase::Duel);
        let b = run.boss.unwrap();
        w.enemies[b].hp = 3;
        w.player.hp = 0;
        assert_eq!(
            tick(&mut run, &mut w, Actions::default()),
            StageOut::Respawned
        );
        assert_eq!(run.phase, Phase::GateOpen);
        assert!(!run.gate_closed);
        let boss = &w.enemies[run.boss.unwrap()];
        assert_eq!((boss.hp, boss.state), (boss.max_hp, EState::Dormant));
        assert!(w.player.pos.distance(run.rests[gate_rest]) < REST_REACH);
    }

    #[test]
    fn falling_ends_the_stage() {
        let t = Tuning::default();
        let (mut run, mut w) = StageRun::start(&sanctuary::DEF);
        w.player.hp = 0;
        w.step(&Actions::default(), &t);
        assert_eq!(run.tick(&mut w, 1.0 / 60.0), StageOut::Defeated);
    }
}
