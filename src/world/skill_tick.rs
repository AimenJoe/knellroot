//! 스킬 틱(0151, 표는 `skills.rs`): 누름 → 받아 두기(`skill_press`), 시작(`try_skill`), 상태 틱(`tick_skill`·`tick_ult`),
//! 판정(`skill_area`·`skill_pull`·`upend`), 스킬이 남긴 것(`tick_skill_fields` — 곧 터질 자리·갈라진 땅, 0158 G1),
//! 틱 끝의 셈(`skill_books` — 기세·게이지). `tick_player`의 한 갈래다.
//!
//! 상태 틱(0278 R20): `tick_skill`은 머리(두 버튼 함께 · 공용 값 `SkillCx`)와 꼬리(상태 적기)만 — 모양마다의 일은 빠짐없는 match
//! `tick_form`이 `skill_tick/forms.rs`의 메서드(`tick_ring` · `tick_lane` …)로 나눈다. 넓은 타의 바탕은 `World::sweep`.
//!
//! G2(0158): 버팀(버티는 동안의 표지는 여기서 적고 맞음 길 `tick_enemies`가 읽는다 · 끝의 터뜨림), 되울림(패링 길이 `echo_ring`을
//! 부른다), 던지기(`Blade` — `tick_skill_fields`가 날린다), 잔불 흩기(`Field`), 큰 기술 셋(`start_ult`·`tick_ult` — 쟁반 뒤집기 `upend`
//! · 몰아치기 · 한 줄기 `line_cut`). 새 스킬들의 타는 `skill_touch` 한 곳을 탄다.
//!
//! 단과 고르기(0158): 숫자는 `World::skill_eff`(= `Skill::at(단, 고르기)`)가 준 값만 쓴다 — 여기는 `Twist`·`Cap`의 갈래만 본다.
use super::player_tick::Ctx;
use super::*;
use crate::skills::{Cap, Eff, Form, Twist, FLURRY, LINE, UPEND};
use crate::status::{Dose, NO_DOSE};

/// 저주 새기기(0274 ⑥): 옮은 표가 남는 가장 짧은 초 · 최종 효과(옮을 때 쓰러진 자리가 울린다)의 둘레(m).
pub const HEX_JUMP_MIN: f32 = 2.0;
pub const HEX_TOLL_RADIUS: f32 = 2.5;
use crate::tuning::{
    BASH_CUT_STAGGER, BASH_PUSH, BELL_CONE, BLADE_CATCH, BLADE_HEIGHT, ERUPT_RING_MAX, HOP_DASH,
    HOP_EMPTY, HOP_GAP, SCORCH_EVERY, SKILL_DENIED_SHOW, SKILL_HITSTOP, SPIRE_DELAY, TRAP_SPIKES,
    TRAP_TWIN_GAP, ULT_HITSTOP, ULT_KILL_WINDOW,
};

mod fields;
mod forms;
mod ult;

/// 곧 터질 "모인 자리"(끌어모으기 ㄱ): `left`초 뒤 `at` 둘레 `radius` m에 `damage`. 종 치기(0270 C3)의 종은 `on` = 종을 건 적
/// (그 적이 서 있는 자리에서 울린다 — 쓰러졌으면 마지막 자리), `dose` = 울림이 싣는 타입, `toll` = 울림으로 그린다(렌더가 종을 그린다).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Burst {
    pub at: Vec3,
    pub left: f32,
    pub radius: f32,
    pub damage: i32,
    pub on: Option<usize>,
    pub dose: Dose,
    pub toll: bool,
}

/// 갈라진 땅(땅울림 ㄴ): 찍은 부채꼴이 `left`초 남는다 — 그 안의 적은 `slow`배로 걷는다. 렌더가 바닥에 그린다.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Zone {
    pub pos: Vec3,
    pub dir: Vec3,
    pub range: f32,
    pub arc_deg: f32,
    pub left: f32,
    pub total: f32,
    pub slow: f32,
}

impl Zone {
    /// 이 자리(몸 반지름 `radius`)가 부채꼴 안인가.
    pub fn holds(&self, at: Vec3, radius: f32) -> bool {
        let dv = vec3(at.x - self.pos.x, 0.0, at.z - self.pos.z);
        let r = dv.length();
        (at.y - self.pos.y).abs() <= HIT_DY
            && r - radius <= self.range
            && (r < 1e-3 || (dv / r).dot(self.dir) >= (self.arc_deg * 0.5).to_radians().cos())
    }
}

/// 던진 무기(0158 G2, 던지기): 곧게 `dist` m를 날아가(가는 길) 방랑자에게 되돌아온다(오는 길) — 길의 반 폭 `half` m 안의 적을
/// 길마다 한 번씩 친다. `stick` = 돌아오지 않고 끝에서 터진다(둘레·피해), `drag` = 오는 길에 맞은 적을 끄는 거리. 렌더가 그 자리에 무기를 그린다.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Blade {
    pub pos: Vec3,
    pub prev: Vec3,
    pub dir: Vec3,
    pub went: f32,
    pub dist: f32,
    pub half: f32,
    pub speed: f32,
    /// 가는 길인가(거짓 = 돌아오는 길).
    pub out: bool,
    /// 이 길에서 이미 맞은 적(번호 비트 — 길이 바뀌면 비운다).
    pub hits: u64,
    pub damage: i32,
    pub poise: f32,
    /// 타가 쌓는 타입 게이지 — 던진 순간에 든 무기의 것(0225: 날아가는 동안 바꿔 든 무기의 마법력을 받지 않는다).
    pub dose: Dose,
    /// 오는 길의 피해·자세 피해(맺음이면 더 세다).
    pub back_damage: i32,
    pub back_poise: f32,
    pub drag: f32,
    pub stick: Option<(f32, i32)>,
    /// 날아온 초(렌더의 회전).
    pub age: f32,
}

/// 남은 불티(0158 G2, 잔불 흩기): `pos` 둘레 `radius` m에 `secs`초 — `every`초마다 그 안의 적을 태운다(`pulses` = 지금까지 태운 수).
/// `follow` = 방랑자를 따라온다, `kindle` = 그 안에서 쓰러진 적이 더 남기는 잔불 수. 렌더가 바닥에 그린다.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Field {
    pub pos: Vec3,
    pub radius: f32,
    pub age: f32,
    pub secs: f32,
    pub every: f32,
    pub pulses: u32,
    pub damage: i32,
    pub poise: f32,
    /// 타가 쌓는 타입 게이지 — 흩은 순간에 든 무기의 것(0225).
    pub dose: Dose,
    pub follow: bool,
    pub kindle: u32,
    /// 흐르는 쪽과 빠르기(눈발 ㄱ, 0270 C3 — m/s, 0이면 제자리).
    pub drift: Vec3,
    /// 덫(뿌리 덫, 0270 C3): 때가 아니라 적이 들어서면 한 번 솟고 사라진다.
    pub trap: bool,
    /// 쇠 덫(0276 — 쇠뇌의 빈 손 `World::set_snare`): 뿌리가 솟지 않고 쇠 이빨이 문다(그림 · 소리가 다르다).
    pub iron: bool,
    /// 덫이 솟을 때 함께 터지는 마지막 가시(뿌리 덫의 맺음 — 둘레·피해).
    pub spire: Option<(f32, i32)>,
}

impl Field {
    /// 제자리에 놓인 마당(0278 R20 — 칸 열다섯의 글이 일곱 곳에 있었다): `pos` 둘레 `radius` m에 `secs`초, `every`초마다
    /// `damage` · `poise` · `dose`로 친다. 따르기 · 남는 불씨 · 흐름 · 덫은 없다 — 쓰는 곳이 그 칸만 적는다.
    pub fn lay(
        pos: Vec3,
        radius: f32,
        secs: f32,
        every: f32,
        damage: i32,
        poise: f32,
        dose: Dose,
    ) -> Field {
        Field {
            pos,
            radius,
            age: 0.0,
            secs,
            every,
            pulses: 0,
            damage,
            poise,
            dose,
            follow: false,
            kindle: 0,
            drift: Vec3::ZERO,
            trap: false,
            iron: false,
            spire: None,
        }
    }

    /// 이 자리(몸 반지름 `radius`)가 불티 안인가.
    pub fn holds(&self, at: Vec3, radius: f32) -> bool {
        (at.y - self.pos.y).abs() <= HIT_DY
            && vec3(at.x - self.pos.x, 0.0, at.z - self.pos.z).length() - radius <= self.radius
    }

    /// 남은 몫 0..1(HUD 칸·렌더의 옅어짐).
    pub fn left(&self) -> f32 {
        (1.0 - self.age / self.secs.max(1e-3)).clamp(0.0, 1.0)
    }
}

/// 새 스킬들의 타 하나(`skill_touch`): 피해(0 = 없음)·자세 피해·휘청(보스는 휘청이지 않는다)·밀려나는 속도·치명타를 쓰나
/// (남는 피해 — 불티·울림 — 는 드러난 틈을 쓰지 않는다: 반격의 치명타를 먹어 치우지 않게).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Touch {
    pub(super) damage: i32,
    pub(super) poise: f32,
    pub(super) stagger: f32,
    pub(super) knock: Vec3,
    pub(super) crit: bool,
    /// 싣는 타입과 쌓는 양(0223 V2).
    pub(super) dose: Dose,
    /// 타마다의 피해 덤(`Stats::dmg`)을 뺀다 — 타입이 스스로 내는 타(울림).
    pub(super) flat: bool,
}

/// 스킬 틱의 공용 값(0278 R20 — `tick_skill`이 세우고 모양마다의 메서드 `forms.rs`가 읽는다): 이 틱의 입력 맥락 · 스킬 · 그 레벨의
/// 값(`skill_eff`) · 지난 틱과 이번 틱의 시각 · 쓴 자리 · 쪽. `swing`만 메서드가 바꾼다(바퀴 · 땀 · 걸음마다 새 타 번호 — 끝에서
/// 상태에 적는다).
pub(super) struct SkillCx<'a, 'c> {
    pub(super) cx: &'a Ctx<'c>,
    pub(super) t: &'c Tuning,
    pub(super) id: skills::Id,
    pub(super) d: Eff,
    pub(super) s: skills::Skill,
    pub(super) e: f32,
    pub(super) e1: f32,
    pub(super) at: Vec3,
    pub(super) dir: Vec3,
    pub(super) swing: u32,
}

impl SkillCx<'_, '_> {
    /// 스킬 시각 `at`초를 이번 틱에 지났나(지난 틱 < `at` ≤ 이번 틱).
    pub(super) fn crossed(&self, at: f32) -> bool {
        self.e < at && self.e1 >= at
    }
}

/// 모양의 틱이 끝난 뒤: `Go` = 공용 끝(상태 적기)으로, `Left` = 이미 다른 상태로 나갔다(버팀 중 회피 — 끝을 건너뛴다).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Flow {
    Go,
    Left,
}

/// 스킬의 넓은 타 한 번(`skill_area`): 닿는 거리·부채꼴(쪽, cos 반각)·밀기·휘청·타 번호·피해·자세 피해·깨진 적에게의 배.
struct Sweep {
    reach: f32,
    cone: Option<(Vec3, f32)>,
    push: f32,
    stagger: f32,
    swing: u32,
    damage: i32,
    poise: f32,
    shatter: f32,
    /// 싣는 타입과 쌓는 양(0223 V2).
    dose: Dose,
}

impl World {
    /// 타가 싣는 타입 게이지에 아이템 옵션의 배(`Stats::ail`)를 입힌다(0223 V4) — 스킬의 타가 나가는 자리마다.
    pub(super) fn dosed(&self, d: Dose) -> Dose {
        // 지팡이의 마법력(0225): 스킬의 타가 쌓는 타입 게이지에 무기의 배.
        (d.0, d.1 * self.stats.ail * self.player.weapon.art().channel)
    }

    /// 스킬 `id`의 지금 값: 끼운 칸의 단·고르기를 입힌 것(끼우지 않은 스킬은 1단 — 시험·바꿔 끼운 직후).
    pub fn skill_eff(&self, id: skills::Id) -> Eff {
        let pl = &self.player;
        // 땅을 짚는 손(0274 ⑤ — 낫의 빈 손): 뿌리 스킬의 치수가 더 자란다(4레벨의 치수 덤과 같은 칸).
        let grow = if self.bare() == Some(weapons::Bare::GroundHand)
            && id.def().kind == crate::status::Kind::Root
        {
            weapons::BARE_GROUND_REACH
        } else {
            1.0
        };
        let mut eff = match pl.skill.worn.iter().position(|s| *s == id) {
            Some(i) => id.def().at_grown(pl.skill.rank[i], pl.skill.pick[i], grow),
            None => id.def().at_grown(1, skills::Pick::A, grow),
        };
        // 망치의 물리력(0225): 타입 없는 스킬의 피해·자세 피해에 무기의 배(타입 있는 스킬은 `dosed`가 지팡이의 배를 얹는다).
        let force = pl.weapon.art().force;
        // 망치의 힘은 타입 없는 스킬과 망치의 타입 충격(0270 B)에 얹는다.
        if matches!(
            eff.skill.kind,
            crate::status::Kind::Physical | crate::status::Kind::Shock
        ) && force != 1.0
        {
            eff.skill.damage = (eff.skill.damage as f32 * force).round() as i32;
            eff.skill.poise *= force;
        }
        eff
    }

    /// 이번 틱의 스킬 누름 → 받아 둘 행동(0151 3절). 둘을 함께 눌렀거나(같은 틱) 터치의 큰 기술 버튼이면 큰 기술,
    /// 스킬의 시동 안(`skill.chord s`)에 **다른** 스킬 버튼이 들어와도 큰 기술 — 게이지가 가득일 때만. 그 밖에는 그 칸의 스킬.
    /// 대기가 한참 남은 스킬은 받아 두지 않고 칸만 깜빡인다(몇 초 뒤에 저절로 나가면 안 된다).
    pub(super) fn skill_press(&mut self, a: &Actions, t: &Tuning) -> Option<Next> {
        let pl = &mut self.player;
        let full = pl.skill.gauge >= 1.0;
        if a.ult || (a.skill1 && a.skill2) {
            if full {
                return Some(Next::Ult);
            }
            if a.ult {
                return None;
            }
        }
        let slot = if a.skill1 {
            0
        } else if a.skill2 {
            1
        } else {
            return None;
        };
        if let PState::Skill { id, t: e, .. } = pl.state {
            if full && e < t[P::SkillChord] && pl.skill.worn[slot] != id {
                return Some(Next::Ult);
            }
        }
        // 다른 칸을 방금(`skill.chord s` 안에) 눌렀다 — 시동이 서지 않은 칸(못 쓰는·빈 칸)이어도 함께로 본다.
        if full
            && pl
                .skill
                .chord
                .is_some_and(|(other, _)| other as usize != slot)
        {
            pl.skill.chord = None;
            return Some(Next::Ult);
        }
        pl.skill.chord = Some((slot as u8, t[P::SkillChord]));
        // 빈 칸(0166 — 안 배운 스킬이 놓인 자리, 단 0): 눌러도 아무 일 없다(받아 두지도, 깜빡이지도 않는다).
        if pl.skill.rank[slot] == 0 {
            return None;
        }
        if pl.skill.cd[pl.skill.worn[slot].index()] > t[P::AttackBuffer] {
            if pl.skill.denied <= 0.0 {
                self.skill_ev.push(Event::SkillDenied);
            }
            pl.skill.denied = SKILL_DENIED_SHOW;
            return None;
        }
        Some(Next::Skill(slot as u8))
    }

    /// 서 있을 때 받은 스킬 누름(캔슬로 넘어온 것 포함)을 시작한다.
    pub(super) fn try_skill(&mut self, cx: &Ctx) -> bool {
        let a = &cx.a;
        if a.ult {
            if self.player.skill.gauge < 1.0 {
                return false;
            }
            self.start_ult(cx);
            return true;
        }
        let slot = if a.skill1 {
            0
        } else if a.skill2 {
            1
        } else {
            return false;
        };
        let id = self.player.skill.worn[slot];
        if self.player.skill.rank[slot] == 0 || self.player.skill.cd[id.index()] > 0.0 {
            return false;
        }
        let d = self.skill_eff(id);
        let dir = self.skill_dir(cx, &d);
        // 멀리 뛰기(땅울림 ㄱ): 뛰어들 거리는 누른 순간에 정한다 — 그 길의 가장 가까운 적 앞까지.
        let leap = match d.twist {
            Some(Twist::Leap { dist, short, half }) => self.leap_len(dir, dist, short, half),
            _ => 0.0,
        };
        let pl = &mut self.player;
        pl.skill.cd[id.index()] = d.skill.cooldown * cx.t[P::SkillCooldownX] * self.stats.skill_cd;
        pl.skill.leap = leap;
        pl.skill.refunded = false;
        pl.skill.ghost = false;
        pl.skill.brace = 0;
        if matches!(d.skill.form, Form::Brace { .. }) {
            pl.skill.brace_soaked = 0;
            self.brace_caught.clear();
        }
        let pl = &mut self.player;
        pl.skill.hop_home = pl.pos;
        pl.skill.hop_from = pl.pos;
        pl.skill.hop_to = pl.pos;
        pl.skill.hop_first = None;
        self.hop_seen.clear();
        let pl = &mut self.player;
        pl.swings += 1;
        pl.facing = dir.x.atan2(dir.z);
        pl.state = PState::Skill {
            id,
            t: 0.0,
            dir,
            swing: pl.swings,
        };
        self.skill_ev.push(Event::SkillStart { id });
        true
    }

    /// 스킬이 나가는 쪽: 가로지르기는 쥔 스틱 쪽(떼를 비껴 뚫을 수 있게), 겨누는 스킬은 조준 > 락온 > 닿는 거리의 가장 가까운 적,
    /// 없으면 바라보는 쪽. 뛰어드는 땅울림은 뛰는 거리만큼 더 멀리서도 겨눈다.
    fn skill_dir(&self, cx: &Ctx, d: &Eff) -> Vec3 {
        let pl = &self.player;
        let facing = pl.facing_dir();
        let lock = cx
            .target
            .map(|tg| vec3(tg.x - pl.pos.x, 0.0, tg.z - pl.pos.z).normalize_or_zero())
            .filter(|v| v.length_squared() > 0.5);
        match d.skill.form {
            Form::Ring { .. }
            | Form::Brace { .. }
            | Form::Echo { .. }
            | Form::Field { .. }
            | Form::Wave { .. }
            | Form::Rally { .. }
            | Form::Reap { .. } => facing,
            // 눈발·뿌리 덫(0270 C3): 놓을 자리 쪽 — 조준 > 락온 > (터치의 톡이면 놓는 거리의 가장 가까운 적) > 바라보는 쪽.
            // 곁의 적을 찾아 놓으면 앞에 둔 자리가 엉뚱한 쪽으로 간다.
            Form::Drift { ahead, radius, .. } | Form::Trap { ahead, radius, .. } => cx
                .aim
                .or(lock)
                .or_else(|| self.tap_seek(cx, ahead + radius))
                .unwrap_or(facing),
            // 던지기: (숄더뷰) 크로스헤어 > 조준 > 락온 > 닿는 거리의 가장 가까운 적 > 바라보는 쪽.
            Form::Throw { dist, .. } => cx
                .sight
                .or(cx.aim)
                .or(lock)
                .or_else(|| nearest_in_reach(&self.enemies, pl.pos, dist, None))
                .unwrap_or(facing),
            // 터치로 끌어 겨눈 것(0217)은 걷는 쪽보다 앞선다 — 손가락이 가리킨 쪽으로 간다.
            Form::Lane { dist, .. } => {
                if let Some(d) = cx.aim.filter(|_| cx.a.aim_drag) {
                    d
                } else if cx.world_move.length_squared() > 1e-4 {
                    cx.world_move.normalize()
                } else {
                    cx.aim
                        .or(lock)
                        .or_else(|| self.tap_seek(cx, dist))
                        .unwrap_or(facing)
                }
            }
            // 제비걸음(0214): 첫 걸음의 쪽은 걸음마다 다시 정한다 — 처음 보는 쪽만(조준 > 락온 > 바라보는 쪽).
            // 터치의 톡(0217)이면 바라보는 쪽 대신 닿는 거리의 가장 가까운 적.
            // 숄더뷰에서는 크로스헤어 쪽이 먼저(0230 — 한 대상을 겨누는 꼴).
            Form::Hop { range, .. } => cx
                .sight
                .or(cx.aim)
                .or(lock)
                .or_else(|| self.tap_seek(cx, range))
                .unwrap_or(facing),
            Form::Erupt { len, .. } => cx
                .aim
                .or(lock)
                .or_else(|| self.tap_seek(cx, len))
                .unwrap_or(facing),
            // 뿌리 솟기·막아치기(0214)·바늘땀·종 치기(0270 C3): 조준 > 락온 > 닿는 거리의 가장 가까운 적 > 바라보는 쪽.
            // 종 치기는 한 대상이라 숄더뷰에서는 크로스헤어가 먼저(0230).
            Form::Bell { range, .. } => cx
                .sight
                .or(cx.aim)
                .or(lock)
                .or_else(|| nearest_in_reach(&self.enemies, pl.pos, range + 1.0, None))
                .unwrap_or(facing),
            // 저주 새기기(0274 ⑥): 한 대상 — 숄더뷰에서는 크로스헤어가 먼저, 없으면 닿는 거리의 가장 가까운 적.
            Form::Hex { range, .. } => cx
                .sight
                .or(cx.aim)
                .or(lock)
                .or_else(|| nearest_in_reach(&self.enemies, pl.pos, range + 1.0, None))
                .unwrap_or(facing),
            Form::Bash { range, .. } | Form::Stab { range, .. } | Form::Slip { range, .. } => cx
                .aim
                .or(lock)
                .or_else(|| nearest_in_reach(&self.enemies, pl.pos, range + 1.0, None))
                .unwrap_or(facing),
            Form::Pull { range, .. } | Form::Cone { range, .. } => {
                let reach = range
                    + match d.twist {
                        Some(Twist::Leap { dist, .. }) => dist,
                        _ => 0.0,
                    };
                cx.aim
                    .or(lock)
                    .or_else(|| nearest_in_reach(&self.enemies, pl.pos, reach, None))
                    .unwrap_or(facing)
            }
        }
    }

    /// 터치의 톡(0217): 끌지 않고 뗀 버튼은 `reach` m 안의 가장 가까운 적 쪽 — 찾지 않는 꼴도 톡이면 찾는다.
    fn tap_seek(&self, cx: &Ctx, reach: f32) -> Option<Vec3> {
        cx.a.auto_aim
            .then(|| nearest_in_reach(&self.enemies, self.player.pos, reach, None))
            .flatten()
    }

    /// 멀리 뛰기의 거리: `dir` 쪽 길(반 폭 `half` m) 위의 가장 가까운 깨어 있는 적 `short` m 앞까지, 많아야 `dist` m.
    /// 길에 적이 없으면 `dist` m, 바로 앞에 있으면 0(제자리에서 찍는다).
    fn leap_len(&self, dir: Vec3, dist: f32, short: f32, half: f32) -> f32 {
        let p = self.player.pos;
        let body = p.y + self.player.air;
        let mut len = dist;
        for en in &self.enemies {
            if !en.active() || (en.pos.y - body).abs() > HIT_DY {
                continue;
            }
            let dv = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z);
            let along = dv.dot(dir);
            let off = (dv - dir * along).length();
            if along <= 0.0 || off > half + en.radius {
                continue;
            }
            len = len.min(along - en.radius - short);
        }
        len.clamp(0.0, dist)
    }

    pub(super) fn tick_skill(&mut self, cx: &Ctx, id: skills::Id, e: f32, dir: Vec3, swing: u32) {
        let (t, dt) = (cx.t, cx.dt);
        let d = self.skill_eff(id);
        let s = d.skill;
        // 두 버튼 함께(0151): 시동 안에 들어온 큰 기술 누름이 이 스킬을 바꿔친다 — 쓴 대기는 돌려준다.
        if cx.a.ult && e < t[P::SkillChord] && self.player.skill.gauge >= 1.0 {
            self.player.skill.cd[id.index()] = 0.0;
            self.player.skill.ghost = false;
            self.player.skill.brace = 0;
            self.start_ult(cx);
            return;
        }
        let e1 = e + dt;
        let at = self.player.pos;
        self.player.skill.ghost = false;
        self.player.skill.brace = 0;
        let mut k = SkillCx {
            cx,
            t,
            id,
            d,
            s,
            e,
            e1,
            at,
            dir,
            swing,
        };
        if self.tick_form(&mut k) == Flow::Left {
            return;
        }
        let swing = k.swing;
        self.player.state = if e1 >= d.total() {
            self.player.skill.ghost = false;
            self.player.skill.brace = 0;
            PState::Idle
        } else {
            PState::Skill {
                id,
                t: e1,
                dir,
                swing,
            }
        };
    }

    /// 모양마다 한 갈래(와일드카드 없이 — 새 모양은 여기서 컴파일이 깨진다): 그 모양의 메서드(`skill_tick/forms.rs`)가 이 틱의 일을 한다.
    fn tick_form(&mut self, k: &mut SkillCx) -> Flow {
        match k.s.form {
            Form::Ring {
                radius,
                hits,
                gap,
                push,
            } => self.tick_ring(k, radius, hits, gap, push),
            Form::Lane {
                half, time, push, ..
            } => self.tick_lane(k, half, time, push),
            Form::Stab {
                range,
                arc_deg,
                hits,
                gap,
            } => self.tick_stab(k, range, arc_deg, hits, gap),
            Form::Bell {
                range,
                delay,
                radius,
            } => self.tick_bell(k, range, delay, radius),
            Form::Hex { range, secs, .. } => self.tick_hex(k, range, secs),
            Form::Wave { radius, time } => self.tick_wave(k, radius, time),
            Form::Pull {
                range,
                arc_deg,
                to,
                stagger,
            } => self.tick_pull(k, range, arc_deg, to, stagger),
            Form::Cone {
                range,
                arc_deg,
                stagger,
                push,
            } => self.tick_cone(k, range, arc_deg, stagger, push),
            Form::Hop {
                range,
                hops,
                gap,
                reach,
            } => self.tick_hop(k, range, hops, gap, reach),
            Form::Rally { heal, radius, push } => self.tick_rally(k, heal, radius, push),
            Form::Bash {
                range,
                arc_deg,
                stagger,
                mult,
            } => self.tick_bash(k, range, arc_deg, stagger, mult),
            Form::Reap {
                radius,
                heal,
                most,
                time,
            } => self.tick_reap(k, radius, heal, most, time),
            Form::Slip { range, arc_deg, .. } => self.tick_slip(k, range, arc_deg),
            Form::Erupt {
                len,
                step,
                every,
                radius,
                stagger,
            } => self.tick_erupt(k, len, step, every, radius, stagger),
            Form::Brace { .. } => return self.tick_brace(k),
            Form::Echo { secs, .. } => self.tick_echo(k, secs),
            Form::Throw {
                dist, half, speed, ..
            } => self.tick_throw_skill(k, dist, half, speed),
            Form::Drift {
                ahead,
                radius,
                secs,
                every,
            } => self.tick_drift(k, ahead, radius, secs, every),
            Form::Trap {
                ahead,
                radius,
                secs,
            } => self.tick_trap(k, ahead, radius, secs),
            Form::Field {
                radius,
                secs,
                every,
            } => self.tick_field(k, radius, secs, every),
        }
        Flow::Go
    }

    /// 스킬 `s`의 넓은 타 바탕(0278 R20 — 아홉 곳의 같은 글): 그 몸 둘레 `reach` m, 표의 피해 · 자세 피해 · 타입(`dosed`),
    /// 부채꼴 · 밀기 · 휘청 없음 · 깨진 적에게 배 없음. 모양은 바뀌는 칸만 적는다(`Sweep { push, ..self.sweep(…) }`).
    fn sweep(&self, s: &skills::Skill, reach: f32, swing: u32) -> Sweep {
        Sweep {
            reach,
            cone: None,
            push: 0.0,
            stagger: 0.0,
            swing,
            damage: s.damage,
            poise: s.poise,
            shatter: 1.0,
            dose: self.dosed(s.dose()),
        }
    }

    /// 쿨타임 돌려받기(맺음 `Cap::Refund` — 가로지르기 · 꿰뚫기 · 방패 돌진의 길, 빠지며 베기): `kills`(타 앞의 셈)보다 지금 더
    /// 쓰러뜨렸으면 그 스킬의 쿨타임 × `part`를 돌려받는다 — **한 번 쓸 때 한 번**(`refunded`, `try_skill`이 비운다: 틱마다 치는
    /// 길에서 거듭 받지 않게. 한 번만 베는 빠지며 베기는 깃발이 늘 비어 있어 같다 — 0278 R20에서 둘을 하나로).
    fn refund_on_kill(&mut self, k: &SkillCx, part: f32, kills: u32) {
        let pl = &mut self.player;
        if self.kills > kills && !pl.skill.refunded {
            pl.skill.refunded = true;
            let back = k.s.cooldown * k.t[P::SkillCooldownX] * self.stats.skill_cd * part;
            let cd = &mut pl.skill.cd[k.id.index()];
            *cd = (*cd - back).max(0.0);
        }
    }

    /// 스킬의 넓은 타: 방랑자 둘레 `reach` m(부채꼴이면 `cone` = (쪽, cos 반각)) 안의 깨어 있는 적을 한 번씩 친다.
    /// 등딱지도 뚫는다(강타처럼). 맞은 적은 바깥으로 `push` m 밀리고, `stagger`초 휘청인다(보스는 휘청이지 않는다).
    /// 스킬의 타는 큰 기술 게이지를 채우지 않는다(`connected`를 건드리지 않는다).
    fn skill_area(&mut self, t: &Tuning, sw: &Sweep) -> u32 {
        let mut ev = std::mem::take(&mut self.skill_ev);
        let p = self.player.pos;
        let body = p.y + self.player.air;
        let facing = self.player.facing_dir();
        let snare = self.snare_secs();
        let mut n = 0;
        let knack = self.knack();
        for en in &mut self.enemies {
            if !en.active() || en.hit_by == sw.swing || (en.pos.y - body).abs() > HIT_DY {
                continue;
            }
            let dv = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z);
            if dv.length() - en.radius > sw.reach {
                continue;
            }
            let dir = dv.normalize_or(facing);
            if sw.cone.is_some_and(|(f, half)| dir.dot(f) < half) {
                continue;
            }
            en.hit_by = sw.swing;
            n += 1;
            if !en.boss {
                en.marks.snare = en.marks.snare.max(snare);
            }
            let at = en.pos + Vec3::Y * en.height * 0.6;
            let crit = en.exposed > 0.0;
            let dmg = wound(
                en,
                self.stats.dmg + sw.damage,
                crit,
                self.stats.opened_dmg,
                knack,
                t,
                &mut ev,
            );
            // 땅울림 맺음: 이미 자세가 깨져 있던 적에게는 배로(이 타로 깨진 적은 다음부터).
            en.hp -= if en.stance.broken > 0.0 {
                skills::scaled(dmg, sw.shatter)
            } else {
                dmg
            };
            if crit {
                en.exposed = 0.0;
                ev.push(Event::Crit { at });
            }
            en.struck();
            let mass = if en.boss { 0.25 } else { 1.0 };
            en.knock = if sw.push < 0.0 {
                // 끌기(0274 ⑤ — 걸어 당기기: 음수 `push`): 몸 앞까지 끌어온다, 문지기는 끌리지 않는다(무겁다).
                if en.boss {
                    Vec3::ZERO
                } else {
                    tug(en, pull_slide(dv.length(), en.radius, -sw.push), &mut ev);
                    -dir * pull_slide(dv.length(), en.radius, -sw.push) * KNOCK_DECAY
                }
            } else {
                dir * (t[P::Knockback] * 0.5 + sw.push * KNOCK_DECAY) * mass
            };
            en.sway = dir * HEAVY_SWAY;
            en.sway_t = 0.0;
            self.sparks.extend(sparks_at(
                en.pos + Vec3::Y * en.height * 0.55,
                dir,
                sw.swing,
                true,
            ));
            add_poise(en, sw.poise * self.stats.poise, None, t, &mut ev);
            afflict(en, sw.dose, &mut ev);
            ev.push(Event::Hit { at });
            if en.hp <= 0 {
                en.state = EState::Dead;
                self.kills += 1;
                ev.push(Event::EnemyDown { at: en.pos });
            } else if sw.stagger > 0.0 && !en.boss && !en.rising() {
                en.state = EState::Stagger {
                    t: 0.0,
                    dur: sw.stagger,
                };
            }
        }
        if n > 0 {
            self.skill_hit_tick = true;
            self.hitstop = self.hitstop.max(t[P::Hitstop] * SKILL_HITSTOP);
        }
        self.skill_ev = ev;
        n
    }

    /// 끌어모으기: 앞 부채꼴의 적이 방랑자 앞 한 점으로 미끄러져 모인다(몸끼리는 `collide`가 벌린다). 피해 없음.
    /// 보스는 끌리지 않는다(무겁다 — 0150 4절). 엘리트는 `heavy`(3단 ㄴ)일 때만 끌린다. `frail`(맺음) = 끌려온 적이 잠깐 더 다친다.
    /// 모이는 점을 돌려준다.
    fn skill_pull(&mut self, k: &SkillCx, range: f32, arc_deg: f32, to: f32, stagger: f32) -> Vec3 {
        let (id, dir, d) = (k.id, k.dir, k.d);
        let heavy = matches!(d.twist, Some(Twist::Heavy));
        let frail = match d.cap {
            Some(Cap::Frail { secs, mult }) => Some((secs, mult)),
            _ => None,
        };
        let p = self.player.pos;
        let body = p.y + self.player.air;
        let spot = p + dir * to;
        let half = (arc_deg * 0.5).to_radians().cos();
        // 표의 타입 그대로(`skill_eff`는 피해 · 자세 · 치수만 바꾼다 — 타입과 쌓는 양은 레벨 · 고르기 · 망치의 배와 상관없다).
        let dose = self.dosed(d.skill.dose());
        for en in &mut self.enemies {
            if !en.active() || en.boss || (en.elite.is_some() && !heavy) || en.rising() {
                continue;
            }
            if (en.pos.y - body).abs() > HIT_DY {
                continue;
            }
            let dv = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z);
            let r = dv.length();
            if r - en.radius > range || r < 1e-3 || (dv / r).dot(dir) < half {
                continue;
            }
            let go = vec3(spot.x - en.pos.x, 0.0, spot.z - en.pos.z);
            let slide = (go.length() - en.radius).max(0.0);
            let along = go.normalize_or_zero();
            en.knock = along * slide * KNOCK_DECAY;
            en.spilling = false;
            en.sway = along * HEAVY_SWAY;
            en.sway_t = 0.0;
            en.state = EState::Stagger {
                t: 0.0,
                dur: stagger,
            };
            if let Some((secs, mult)) = frail {
                en.marks.frail = secs;
                en.marks.frail_mult = mult;
            }
            afflict(en, dose, &mut self.skill_ev);
        }
        self.skill_ev.push(Event::Skill { id, at: spot, dir });
        spot
    }

    /// 스킬이 남긴 것의 틱(0158 G1) — 방랑자가 다른 일을 하는 동안에도 돈다(히트스톱 동안은 멈춘다):
    /// "모인 자리"는 때가 되면 둘레를 치고, 갈라진 땅은 그 안의 적을 느리게 한다(보스는 아니다).
    /// 엉킨 뿌리(0214): 스킬에 맞은 적이 느리게 걷는 초(칸이 없으면 0).
    fn snare_secs(&self) -> f32 {
        if self.tree.has(crate::progression::Cell::Snare) {
            crate::progression::cell::SNARE_SECS
        } else {
            0.0
        }
    }

    /// 모인 자리가 터진다: 둘레의 깨어 있는 적에게 피해(밀지 않는다 — 모아 둔 것을 흩지 않게, 휘청도 없다).
    /// 스킬의 타라 게이지는 채우지 않고, 쓰러뜨리면 스킬 몫의 기세.
    /// 제비걸음(0214)의 다음 걸음이 닿을 자리: `range` m 안의 가장 가까운 아직 안 벤 적(한 곳만이면 첫 적, 그 적이 없으면 가까운 적)의
    /// 이쪽 앞 — 적이 없으면 `dir` 쪽으로 `HOP_EMPTY` m.
    fn hop_target(&mut self, range: f32, focus: bool, dir: Vec3) -> Vec3 {
        let p = self.player.pos;
        let body = p.y + self.player.air;
        let near = |en: &crate::enemy::Enemy| {
            en.active()
                && (en.pos.y - body).abs() <= HIT_DY
                && vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z).length() - en.radius <= range
        };
        let first = self
            .player
            .skill
            .hop_first
            .filter(|&i| focus && self.enemies.get(i).is_some_and(near));
        let pick = first.or_else(|| {
            let fresh = |i: &usize| !self.hop_seen.contains(i);
            let by_dist = |i: &usize| {
                let en = &self.enemies[*i];
                vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z).length()
            };
            let all: Vec<usize> = (0..self.enemies.len())
                .filter(|&i| near(&self.enemies[i]))
                .collect();
            all.iter()
                .copied()
                .filter(fresh)
                .min_by(|a, b| by_dist(a).total_cmp(&by_dist(b)))
                .or_else(|| {
                    all.iter()
                        .copied()
                        .min_by(|a, b| by_dist(a).total_cmp(&by_dist(b)))
                })
        });
        match pick {
            Some(i) => {
                self.hop_seen.push(i);
                if self.player.skill.hop_first.is_none() {
                    self.player.skill.hop_first = Some(i);
                }
                let en = &self.enemies[i];
                let way = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z);
                let d = way.length();
                // 적의 이쪽 앞에 선다(몸이 겹치지 않게).
                p + way.normalize_or(dir) * (d - en.radius - PLAYER_RADIUS - HOP_GAP).max(0.0)
            }
            None => p + dir * HOP_EMPTY,
        }
    }

    /// 막아치기(0214): 앞 부채꼴의 깨어 있는 적을 한 번씩 친다. 공격을 준비하던(예고 중인) 적은 그 공격이 끊기고(더 길게 휘청)
    /// 자세 피해 × `mult` — 드러난 틈(ㄴ)이면 한동안 더 다치고, 무너뜨림(맺음)이면 자세가 곧바로 깨진다. 보스는 끊기지 않고 자세
    /// 피해만 더 받는다. 나머지는 `stagger`초 휘청(보스는 아니다).
    fn skill_bash(&mut self, k: &SkillCx, range: f32, arc_deg: f32, stagger: f32, mult: f32) {
        let (t, d, dir, swing) = (k.t, &k.d, k.dir, k.swing);
        let mut ev = std::mem::take(&mut self.skill_ev);
        let p = self.player.pos;
        let body = p.y + self.player.air;
        let half = (arc_deg * 0.5).to_radians().cos();
        let snare = self.snare_secs();
        let poise_k = self.stats.poise;
        let dose = self.dosed(d.skill.dose());
        let mut n = 0;
        let knack = self.knack();
        for en in &mut self.enemies {
            if !en.active() || en.hit_by == swing || (en.pos.y - body).abs() > HIT_DY {
                continue;
            }
            let dv = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z);
            if dv.length() - en.radius > range || dv.normalize_or(dir).dot(dir) < half {
                continue;
            }
            en.hit_by = swing;
            n += 1;
            if !en.boss {
                en.marks.snare = en.marks.snare.max(snare);
            }
            let at = en.pos + Vec3::Y * en.height * 0.6;
            let winding = en.telegraph().is_some() && !en.rising();
            let crit = en.exposed > 0.0;
            en.hp -= wound(
                en,
                self.stats.dmg + d.skill.damage,
                crit,
                self.stats.opened_dmg,
                knack,
                t,
                &mut ev,
            );
            if crit {
                en.exposed = 0.0;
                ev.push(Event::Crit { at });
            }
            en.struck();
            en.knock =
                dv.normalize_or(dir) * BASH_PUSH * KNOCK_DECAY * if en.boss { 0.25 } else { 1.0 };
            ev.push(Event::Hit { at });
            let pk = if winding { mult } else { 1.0 };
            add_poise(en, d.skill.poise * poise_k * pk, None, t, &mut ev);
            afflict(en, dose, &mut ev);
            if en.hp <= 0 {
                en.state = EState::Dead;
                self.kills += 1;
                ev.push(Event::EnemyDown { at: en.pos });
                continue;
            }
            if winding && !en.boss {
                // 끊는다: 준비하던 공격은 나가지 않고 더 길게 휘청인다.
                en.state = EState::Stagger {
                    t: 0.0,
                    dur: stagger * BASH_CUT_STAGGER,
                };
                ev.push(Event::Interrupt { at });
                if let Some(Twist::Expose { secs, mult }) = d.twist {
                    en.marks.frail = en.marks.frail.max(secs);
                    en.marks.frail_mult = mult;
                }
                if matches!(d.cap, Some(Cap::Break)) {
                    add_poise(en, poise_max(en, t), None, t, &mut ev);
                }
            } else if !en.boss && !en.rising() && !matches!(en.state, EState::Stagger { .. }) {
                en.state = EState::Stagger {
                    t: 0.0,
                    dur: stagger,
                };
            }
        }
        if n > 0 {
            self.skill_hit_tick = true;
            self.hitstop = self.hitstop.max(t[P::Hitstop] * SKILL_HITSTOP);
        }
        self.skill_ev = ev;
    }

    /// 재기(0274)의 밀쳐 냄: 둘레 `radius` m의 깨어 있는 적을 바깥으로 `push` m(보스는 넷의 하나) — 피해·자세 없이 흡수만 건다.
    fn skill_rally(&mut self, _t: &Tuning, radius: f32, push: f32, stun: f32, dose: Dose) {
        let mut ev = std::mem::take(&mut self.skill_ev);
        let p = self.player.pos;
        let body = p.y + self.player.air;
        let facing = self.player.facing_dir();
        for en in &mut self.enemies {
            if !en.active() || (en.pos.y - body).abs() > HIT_DY {
                continue;
            }
            let dv = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z);
            if dv.length() - en.radius > radius {
                continue;
            }
            let dir = dv.normalize_or(facing);
            let mass = if en.boss { 0.25 } else { 1.0 };
            en.knock = dir * push * KNOCK_DECAY * mass;
            afflict(en, dose, &mut ev);
            if stun > 0.0 && !en.boss && !en.rising() && en.alive() {
                en.state = EState::Stagger { t: 0.0, dur: stun };
            }
        }
        self.skill_ev = ev;
    }

    /// 뿌리 솟기(0214)의 뿌리 하나: `spot` 둘레 `radius` m의 깨어 있는 적을 한 번씩(이번 스킬에 아직 안 맞은 적만) 찌르고 휘청이게,
    /// 묶는 뿌리(ㄴ)면 `bind`초 묶는다(보스는 휘청·묶임 없음). 솟은 뿌리는 그림으로 남는다(`skill_spikes`).
    fn skill_root(
        &mut self,
        cx: &SkillCx,
        spot: Vec3,
        radius: f32,
        stagger: f32,
        bind: f32,
        k: u8,
    ) {
        let (t, swing, s) = (cx.t, cx.swing, &cx.s);
        let ground = self.ground.as_ref().map_or(spot.y, |g| g.height(spot));
        let at = vec3(spot.x, ground, spot.z);
        let side = if k.is_multiple_of(2) { 1.0 } else { -1.0 };
        self.skill_spikes.push(Spike {
            pos: at,
            age: 0.0,
            lean: vec3(side * 0.12, 0.0, 0.05),
        });
        let mut ev = std::mem::take(&mut self.skill_ev);
        ev.push(Event::RootSpike { at });
        let snare = self.snare_secs();
        let poise_k = self.stats.poise;
        let dose = self.dosed(s.dose());
        let mut n = 0;
        let knack = self.knack();
        for en in &mut self.enemies {
            if !en.active() || en.hit_by == swing || (en.pos.y - at.y).abs() > HIT_DY {
                continue;
            }
            let dv = vec3(en.pos.x - at.x, 0.0, en.pos.z - at.z);
            if dv.length() - en.radius > radius {
                continue;
            }
            en.hit_by = swing;
            n += 1;
            if !en.boss {
                en.marks.snare = en.marks.snare.max(snare);
                en.marks.bind = en.marks.bind.max(bind);
            }
            let hit_at = en.pos + Vec3::Y * en.height * 0.6;
            let crit = en.exposed > 0.0;
            en.hp -= wound(
                en,
                self.stats.dmg + s.damage,
                crit,
                self.stats.opened_dmg,
                knack,
                t,
                &mut ev,
            );
            if crit {
                en.exposed = 0.0;
                ev.push(Event::Crit { at: hit_at });
            }
            en.struck();
            ev.push(Event::Hit { at: hit_at });
            add_poise(en, s.poise * poise_k, None, t, &mut ev);
            afflict(en, dose, &mut ev);
            if en.hp <= 0 {
                en.state = EState::Dead;
                self.kills += 1;
                ev.push(Event::EnemyDown { at: en.pos });
            } else if stagger > 0.0
                && !en.boss
                && !en.rising()
                && !matches!(en.state, EState::Stagger { .. })
            {
                en.state = EState::Stagger {
                    t: 0.0,
                    dur: stagger,
                };
            }
        }
        if n > 0 {
            self.skill_hit_tick = true;
            self.hitstop = self.hitstop.max(t[P::Hitstop] * SKILL_HITSTOP);
        }
        self.skill_ev = ev;
    }

    fn skill_burst(&mut self, t: &Tuning, mut b: Burst) {
        // 종(0270 C3): 건 적이 서 있는 자리에서 울린다.
        if let Some(en) = b.on.and_then(|i| self.enemies.get(i)) {
            b.at = en.pos;
        }
        let mut ev = std::mem::take(&mut self.skill_ev);
        let snare = self.snare_secs();
        let mut n = 0;
        let knack = self.knack();
        for en in &mut self.enemies {
            if !en.active() || (en.pos.y - b.at.y).abs() > HIT_DY {
                continue;
            }
            let dv = vec3(en.pos.x - b.at.x, 0.0, en.pos.z - b.at.z);
            if dv.length() - en.radius > b.radius {
                continue;
            }
            n += 1;
            if !en.boss {
                en.marks.snare = en.marks.snare.max(snare);
            }
            let at = en.pos + Vec3::Y * en.height * 0.6;
            let crit = en.exposed > 0.0;
            en.hp -= wound(
                en,
                self.stats.dmg + b.damage,
                crit,
                self.stats.opened_dmg,
                knack,
                t,
                &mut ev,
            );
            if crit {
                en.exposed = 0.0;
                ev.push(Event::Crit { at });
            }
            en.struck();
            ev.push(Event::Hit { at });
            if en.hp <= 0 {
                en.state = EState::Dead;
                self.kills += 1;
                ev.push(Event::EnemyDown { at: en.pos });
            } else {
                afflict(en, b.dose, &mut ev);
            }
        }
        if n > 0 {
            self.skill_hit_tick = true;
            self.hitstop = self.hitstop.max(t[P::Hitstop] * SKILL_HITSTOP);
        }
        // 종(0270 C3)은 울림으로 — 울림 타입이 둘레를 치는 그림과 소리를 같이 쓴다. 심지 볼트(0272 — 충격)는 화약이 터진다.
        ev.push(match (b.toll, b.dose.0) {
            (true, crate::status::Kind::Shock) => Event::Boom { at: b.at },
            (true, _) => Event::Toll {
                at: b.at,
                radius: b.radius,
            },
            (false, _) => Event::SkillBurst { at: b.at },
        });
        self.skill_ev = ev;
    }

    /// 놓는 것(눈발·뿌리 덫, 0270 C3)의 자리: `dir` 쪽 `ahead` m — 벽·기둥 앞에서 멈춘다(벽 너머에 놓지 않는다), 땅 높이에.
    fn place_ahead(&self, at: Vec3, dir: Vec3, ahead: f32) -> Vec3 {
        let eye = at + Vec3::Y * (self.player.air + BLADE_HEIGHT);
        let mut len = ahead;
        for o in &self.obstacles {
            if let Some(d) = o.ray(eye, dir) {
                len = len.min((d - PLAYER_RADIUS).max(0.0));
            }
        }
        let p = at + dir * len;
        vec3(p.x, self.ground_y(p), p.z)
    }

    /// 벽에 박기(0270 C3 — 방패 돌진·후려치기 ㄱ): 이 타(`swing`)에 맞아 밀려난 적이 벽·턱에 박히면 덤 피해(쟁반 뒤집기와 같은 길
    /// `spilling` — 보스는 밀리기만).
    fn slam_into_walls(&mut self, swing: u32, damage: i32) {
        for en in &mut self.enemies {
            if en.hit_by == swing && en.alive() && !en.boss && !en.rising() {
                en.spilling = true;
                en.slam_bonus = damage;
            }
        }
    }

    /// 종 치기(0270 C3)·심지 볼트(0272)가 걸 적: 앞 쪽(`dir` ± 60°) `range` m 안의 가장 가까운 깨어 있는 적 — 벽·기둥에
    /// 가린 적은 아니다(멀리 가는 심지 볼트가 벽을 뚫지 않게).
    fn bell_target(&self, dir: Vec3, range: f32) -> Option<usize> {
        let p = self.player.pos;
        let body = p.y + self.player.air;
        let eye = p + Vec3::Y * (self.player.air + BLADE_HEIGHT);
        (0..self.enemies.len())
            .filter(|&i| {
                let en = &self.enemies[i];
                let dv = vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z);
                let (len, to) = (dv.length(), dv.normalize_or(dir));
                en.active()
                    && (en.pos.y - body).abs() <= HIT_DY
                    && len - en.radius <= range
                    && to.dot(dir) >= BELL_CONE
                    && !self
                        .obstacles
                        .iter()
                        .any(|o| o.ray(eye, to).is_some_and(|d| d < len - en.radius))
            })
            .min_by(|&a, &b| {
                let d = |i: usize| {
                    let v = self.enemies[i].pos - p;
                    vec3(v.x, 0.0, v.z).length()
                };
                d(a).total_cmp(&d(b))
            })
    }

    /// 쓰러진 자리에서 일어나는 것(0274 ⑥ — 완드, `tick_embers`가 이 틱에 막 쓰러진 적 `fallen`을 채운 뒤):
    /// - **죽음의 울림**(주특성 치른 값 `Art::hex`): 금 간 채 쓰러진 적의 둘레 `DEATH_KNELL_R` m의 적에게 울림 `DEATH_KNELL_DOSE` — 한
    ///   틱에 한 적은 한 번만(떼가 커도 곱절이 되지 않게, 0223 8절).
    /// - **옮는 저주**(저주 새기기의 표 `marks.hex`): 표를 지닌 채 쓰러지면 둘레(그 스킬의 `spread`)의 가장 가까운 표 없는 적(두 갈래 ㄴ이면
    ///   둘)에게 남은 초 그대로(`HEX_JUMP_MIN`초 아래면 그만큼) 옮고 균열을 다시 가득 쌓는다. 최종 효과면 쓰러진 자리가 울린다.
    /// - **계약의 표**(빈 손의 결 `Bare::PactMark`): 저주탄의 표(`marks.pact`)가 남은 적이 쓰러지면 체력 · RB 칸이 돌아온다.
    pub(in crate::world) fn after_deaths(&mut self, t: &Tuning, ev: &mut Vec<Event>) {
        let _ = t;
        if self.fallen.is_empty() {
            return;
        }
        let fallen = std::mem::take(&mut self.fallen);
        let knell = self.player.weapon.art().hex > 1.0;
        let pact = self.bare() == Some(weapons::Bare::PactMark);
        let mut knells: Vec<Vec3> = Vec::new();
        let mut hexes: Vec<(Vec3, f32)> = Vec::new();
        let mut paid = 0;
        for &i in &fallen {
            let en = &mut self.enemies[i];
            if knell && en.ail.rift > 0.0 {
                knells.push(en.pos);
            }
            if en.marks.hex > 0.0 {
                hexes.push((en.pos, en.marks.hex));
                en.marks.hex = 0.0;
            }
            if en.marks.pact > 0.0 {
                en.marks.pact = 0.0;
                if pact {
                    paid += 1;
                }
            }
        }
        self.fallen = fallen;
        if !knells.is_empty() {
            for en in &mut self.enemies {
                if !en.active() || en.harmless() || en.hp <= 0 {
                    continue;
                }
                let near = knells.iter().any(|p| {
                    vec3(en.pos.x - p.x, 0.0, en.pos.z - p.z).length() - en.radius
                        <= weapons::DEATH_KNELL_R
                        && (en.pos.y - p.y).abs() <= HIT_DY
                });
                if near {
                    afflict(
                        en,
                        (crate::status::Kind::Knell, weapons::DEATH_KNELL_DOSE),
                        ev,
                    );
                }
            }
            for at in knells {
                ev.push(Event::DeathKnell { at });
            }
        }
        if !hexes.is_empty() {
            let d = self.skill_eff(skills::Id::Hex);
            // 저주 새기기의 모양에서(그 스킬의 것 — 다른 모양이면 옮지 않는다).
            let spread = if let Form::Hex { spread, .. } = d.skill.form {
                spread
            } else {
                0.0
            };
            let n = if matches!(d.twist, Some(Twist::Twice)) {
                2
            } else {
                1
            };
            let dose = self.dosed(d.skill.dose());
            for (at, left) in hexes {
                let mut near: Vec<(f32, usize)> = self
                    .enemies
                    .iter()
                    .enumerate()
                    .filter(|(_, en)| {
                        en.active()
                            && !en.harmless()
                            && en.hp > 0
                            && en.marks.hex <= 0.0
                            && (en.pos.y - at.y).abs() <= HIT_DY
                    })
                    .map(|(i, en)| {
                        (
                            vec3(en.pos.x - at.x, 0.0, en.pos.z - at.z).length() - en.radius,
                            i,
                        )
                    })
                    .filter(|(r, _)| *r <= spread)
                    .collect();
                near.sort_by(|a, b| a.0.total_cmp(&b.0));
                for &(_, i) in near.iter().take(n) {
                    let en = &mut self.enemies[i];
                    en.marks.hex = left.max(HEX_JUMP_MIN);
                    afflict(en, dose, ev);
                    ev.push(Event::HexJump {
                        from: at + Vec3::Y * 0.8,
                        to: en.pos + Vec3::Y * en.height * 0.6,
                    });
                }
                if let Some(Cap::Toll { damage }) = d.cap {
                    self.skill_bursts.push(Burst {
                        at,
                        left: 0.0,
                        radius: HEX_TOLL_RADIUS,
                        damage,
                        on: None,
                        dose: (crate::status::Kind::Knell, 0.5),
                        toll: true,
                    });
                }
            }
        }
        if paid > 0 {
            let charges = self.cast_charges();
            let pl = &mut self.player;
            pl.hp = (pl.hp + weapons::BARE_PACT_HEAL * paid).min(pl.max_hp);
            pl.ranged.mana =
                (pl.ranged.mana + weapons::BARE_PACT_CHARGE * paid as f32).min(charges);
            ev.push(Event::Mend {
                at: pl.pos + Vec3::Y * 1.0,
            });
        }
    }

    /// 틱 끝의 셈(0151 5절). **기세**: 이번 틱에 쓰러진 적마다 모든 스킬의 대기가 당겨진다 — 스킬이 아닌 것으로 쓰러뜨렸으면
    /// `skill.momentum s`, 스킬(또는 큰 기술이 쏟은 뒤의 벽 박힘)이면 `skill.momentum by skill s`. **게이지**: 맞힌 휘두르기 한 번에
    /// 한 번(광역이 몇 배로 채우지 않는다)·탄이 맞은 틱·패링·딱 맞춘 떼기 — 스킬의 타는 채우지 않는다.
    pub(super) fn skill_books(&mut self, t: &Tuning, ev: &mut Vec<Event>) {
        let count = |f: fn(&Event) -> bool| ev.iter().filter(|e| f(e)).count() as f32;
        let downs = count(|e| matches!(e, Event::EnemyDown { .. }));
        let parries = count(|e| matches!(e, Event::Parry { .. }));
        let perfects = count(|e| matches!(e, Event::Perfect { .. }));
        let by_skill = self.skill_hit_tick || self.skill_kill_left > 0.0;
        let gathering = self.tree.has(crate::progression::Cell::GatheringBeat);
        // 결(0223 11절): 이번 틱에 무기의 타가 묶인 적·금 간 적에 닿았나, 몇이 울렸나 — 칸을 열었을 때만 몫이 있다.
        use crate::progression::{vein, Vein};
        let bound = std::mem::take(&mut self.vein_bound) && self.tree.veined(Vein::Root);
        let cracked = std::mem::take(&mut self.vein_cracked) && self.tree.veined(Vein::Rift);
        // 탄이 맞은 틱(게이지): 큰 기술이 없어도 여기서 걷는다 — 안 걷으면 배운 뒤의 첫 틱에 옛 것이 한 번 찬다.
        let bolt_hit = std::mem::take(&mut self.bolt_hit_tick);
        let tolls = std::mem::take(&mut self.vein_tolls).min(vein::KNELL_TOLLS);
        // 울림의 결(0274 ②): I = 울림 모으기(궁극기 게이지) · II = 울려서 되찾기(체력 — 쉬는 초 동안은 없다).
        let mending = self.tree.vein_picked_b(Vein::Knell);
        let mends = if mending && self.player.skill.mend_rest <= 0.0 {
            tolls as i32
        } else {
            0
        };
        let tolls = if self.tree.veined(Vein::Knell) && !mending {
            tolls as f32
        } else {
            0.0
        };
        // 0270 C3의 결 넷: 흔들린 머리(둘레의 자세) · 흐르는 걸음(빠르기) · 한 모금씩(등잔) — 얼음 속 불씨는 `tick_ails`가 낸다.
        let rattle =
            std::mem::take(&mut self.vein_rattle).filter(|_| self.tree.veined(Vein::Shock));
        let bled = std::mem::take(&mut self.vein_bled) && self.tree.veined(Vein::Sharp);
        let sipped = std::mem::take(&mut self.vein_sipped) && self.tree.veined(Vein::Drain);
        self.vein_shards = 0;
        if let Some(at) = rattle {
            for en in &mut self.enemies {
                let d = vec3(en.pos.x - at.x, 0.0, en.pos.z - at.z).length();
                if en.active() && d > 0.3 && d - en.radius <= vein::SHOCK_REACH {
                    add_poise(en, vein::SHOCK_RATTLE, None, t, ev);
                }
            }
            ev.push(Event::Aftershock { at });
        }
        let pl = &mut self.player;
        if mends > 0 && pl.hp > 0 {
            if pl.hp < pl.max_hp {
                pl.hp = (pl.hp + mends * vein::KNELL_MEND).min(pl.max_hp);
                ev.push(Event::Mend {
                    at: pl.pos + Vec3::Y * 1.0,
                });
            }
            pl.skill.mend_rest = vein::KNELL_MEND_REST;
        }
        if bled {
            pl.skill.stride = vein::SHARP_SECS;
        }
        if sipped {
            pl.skill.sip = (pl.skill.sip + vein::DRAIN_SIP).min(1.0);
            if pl.skill.sip >= 1.0 - 1e-4 && pl.flask < pl.flask_max {
                pl.flask += 1;
                pl.skill.sip = 0.0;
            }
        }
        // 뿌리의 숨: 묶인 적을 맞힌 타 — 기력이 조금 돌아온다. 금을 따라: 금 간 적을 맞힌 타 — 모든 스킬 대기가 조금 당겨진다.
        if bound {
            pl.stamina = (pl.stamina + vein::ROOT_BREATH).min(t[P::StaminaMax]);
        }
        if cracked {
            for cd in &mut pl.skill.cd {
                *cd = (*cd - vein::RIFT_PULL).max(0.0);
            }
        }
        if downs > 0.0 {
            let per = if by_skill {
                t[P::SkillMomentumOwn]
            } else {
                t[P::SkillMomentum]
            };
            for cd in &mut pl.skill.cd {
                *cd = (*cd - per * downs).max(0.0);
            }
        }
        // 몰아치기 3단(G2): 그동안 쓰러뜨릴 때마다 조금 더 이어진다 — 더해지는 몫은 `kill_cap`초까지.
        if downs > 0.0 && pl.skill.flurry_left > 0.0 {
            let more = (FLURRY.kill_more_at(pl.skill.ult_rank) * downs)
                .min(FLURRY.kill_cap - pl.skill.flurry_added)
                .max(0.0);
            pl.skill.flurry_left += more;
            pl.skill.flurry_total += more;
            pl.skill.flurry_added += more;
        }
        // 교체 공격의 창(0272 1b): 내 무기의 타(평타·강공격 — 휘두르기 한 번에 한 번 · 탄)가 닿았거나 받아친 틱에 열린다.
        // 스킬의 타는 열지 않는다(`connected`를 건드리지 않는다 — 게이지와 같은 셈).
        let linked = pl.connected != 0 && pl.connected != pl.link_swing;
        if linked {
            pl.link_swing = pl.connected;
        }
        if linked || bolt_hit || parries > 0.0 {
            pl.swap_link = crate::tuning::SWAP_LINK;
        }
        // 큰 기술을 안 배웠으면(0172 — 단 0) 게이지는 없다: 차지 않으니 가득도·발동도 없다.
        if pl.skill.ult_rank == 0 {
            pl.skill.gauge = 0.0;
            return;
        }
        let was_full = pl.skill.gauge >= 1.0;
        let hit = t[P::SkillGaugeHit];
        let mut gain = parries * t[P::SkillGaugeParry] + perfects * hit * 2.0;
        if pl.connected != 0 && pl.connected != pl.skill.gauge_swing {
            pl.skill.gauge_swing = pl.connected;
            gain += hit;
            // 다잡은 박(0214): 박자 타가 맞으면 반 번 더.
            if gathering && pl.beat.swing_beat {
                gain += hit * crate::progression::cell::GATHERING_BEAT;
            }
        }
        if bolt_hit {
            gain += hit;
        }
        // 울림 모으기: 울림 하나가 평타 한 번만큼.
        gain += hit * tolls;
        pl.skill.gauge = (pl.skill.gauge + gain).min(1.0);
        if !was_full && pl.skill.gauge >= 1.0 {
            ev.push(Event::UltReady);
        }
    }

    /// 스킬 칸 `slot`(0·1)의 준비 0..1(1 = 쓸 수 있다) — HUD·터치.
    pub fn skill_ready(&self, slot: usize, t: &Tuning) -> f32 {
        let id = self.player.skill.worn[slot % 2];
        let full = self.skill_eff(id).skill.cooldown * t[P::SkillCooldownX] * self.stats.skill_cd;
        if full <= 0.0 {
            1.0
        } else {
            1.0 - (self.player.skill.cd[id.index()] / full).clamp(0.0, 1.0)
        }
    }

    /// 스킬 칸 `slot`의 스킬이 남긴 것이 이어지는 몫 0..1(0 = 없음) — 되울림의 울림, 바닥에 놓는 모양(`Form::info().leaves` —
    /// 잔불 흩기의 불티 · 눈발 · 시드는 그늘 · 뿌리 덫 · 화약 덫)의 남은 것(HUD·터치 칸). 이름 목록이 아니라 모양에서 — 0278 R14
    /// 앞에는 시드는 그늘이 목록에 없어 늘 0이었다.
    pub fn skill_active(&self, slot: usize) -> f32 {
        let pl = &self.player;
        let id = pl.skill.worn[slot % 2];
        let trap = match id.def().form.info().leaves {
            skills::Leaves::Nothing => return 0.0,
            skills::Leaves::Aura => {
                if pl.skill.echo_left > 0.0 {
                    return (pl.skill.echo_left / pl.skill.echo_total.max(1e-3)).clamp(0.0, 1.0);
                }
                return 0.0;
            }
            skills::Leaves::Ground => false,
            skills::Leaves::Snare => true,
        };
        // 놓인 것(0270 C3 — 눈발·뿌리 덫도, 0272 화약 덫, 0274 ⑥ 시드는 그늘): 그 스킬의 타입·꼴(덫인가)의 것만.
        self.skill_fields
            .iter()
            .filter(|f| f.dose.0 == id.def().kind && f.trap == trap)
            .map(|f| f.left())
            .fold(0.0, f32::max)
    }

    /// 몰아치기(·눈보라 0270 C3 ·화살비 0272)가 이어지는 몫 0..1(0 = 아니다) — HUD가 큰 기술 게이지를 줄어드는 막대로 그린다.
    pub fn ult_active(&self) -> f32 {
        let pl = &self.player;
        if pl.skill.flurry_left > 0.0 {
            (pl.skill.flurry_left / pl.skill.flurry_total.max(1e-3)).clamp(0.0, 1.0)
        } else if pl.skill.ult == skills::Ult::Blizzard {
            self.skill_fields
                .iter()
                .filter(|f| f.follow && f.dose.0 == crate::status::Kind::Frost)
                .map(|f| f.left())
                .fold(0.0, f32::max)
        } else if pl.skill.ult == skills::Ult::Rain {
            self.skill_fields
                .iter()
                .filter(|f| !f.follow && !f.trap && f.dose.0 == crate::status::Kind::Root)
                .map(|f| f.left())
                .fold(0.0, f32::max)
        } else {
            0.0
        }
    }

    /// 든 것이 손을 떠나 있나(던지기, G2) — 렌더가 손의 무기를 감추고 날아가는 자리에 그린다.
    pub fn weapon_away(&self) -> bool {
        !self.skill_blades.is_empty() || self.javelin_out()
    }

    /// 스킬 둘을 끼운다(같은 것 둘은 바로잡는다) — 그 단·고르기와 큰 기술의 단까지(0158). 스킬 이름만 주면 모두 1단.
    /// 대기는 스킬마다라 그대로 남는다.
    pub fn set_skills(&mut self, l: impl Into<skills::Loadout>) {
        let l = l.into();
        let ids = skills::sane([l.slots[0].id, l.slots[1].id]);
        let pl = &mut self.player;
        pl.skill.worn = ids;
        for (i, id) in ids.into_iter().enumerate() {
            // 같은 스킬 둘이라 바꿔 끼운 칸은 1단. 단 0 = 빈 칸(0166 — 안 배운 스킬이 놓인 자리: 눌러도 아무 일 없다).
            let same = l.slots[i].id == id;
            pl.skill.known[i] = if same { l.slots[i].rank } else { 1 };
            pl.skill.pick[i] = if same {
                l.slots[i].pick
            } else {
                skills::Pick::A
            };
        }
        // 단 0 = 큰 기술을 아직 안 배웠다(0172): 게이지가 차지 않는다 — 두 버튼 함께도 아무 일 없다.
        pl.skill.ult_rank = l.ult;
        pl.skill.ult = l.great;
        self.refit_skills();
    }

    /// 부무기의 칸 둘(0223 V3): 바꿔 들면 이 둘이 버튼에 온다. 큰 기술은 무기와 상관없이 하나다(`set_skills`의 것).
    pub fn set_sub_skills(&mut self, l: impl Into<skills::Loadout>) {
        let l = l.into();
        let ids = skills::sane([l.slots[0].id, l.slots[1].id]);
        let sub = &mut self.player.skill.sub;
        sub.0 = ids;
        for (i, id) in ids.into_iter().enumerate() {
            let same = l.slots[i].id == id;
            sub.1[i] = if same { l.slots[i].rank } else { 1 };
            sub.2[i] = if same {
                l.slots[i].pick
            } else {
                skills::Pick::A
            };
        }
    }
}

#[cfg(test)]
mod tests;
