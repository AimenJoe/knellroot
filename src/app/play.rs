//! 싸움 한 틱과 그 둘레(0019): 월드·스테이지·카메라·조준·랩, 길 안내 화살, 개발 줄, 월드 소리(0066 R3에서 옮김).
use super::*;

impl App {
    /// 싸움 한 틱: 멈춤 → 월드 → 스테이지 → 기록·가게·소리·알림 → 끝났으면 끝 장면.
    pub(super) fn tick_play(&mut self, a: Actions) {
        // 지역 사이 지층 단면 전환(0118): 덮인 동안 월드·멈춤·장비 창은 멈추고, 가운데에서 지역을 바꾼다.
        if let Some(mut p) = self.passage.take() {
            let (swap, done) = p.step(TICK_DT);
            if swap {
                self.cross_to(p.to, p.up);
            }
            if !done {
                self.passage = Some(p);
            }
            return;
        }
        // 세로로 든 폰(0082): 회전 안내가 덮인 동안 싸움이 흐르지 않게 멈춤으로(nail 0269 꼴).
        if a.pause || crate::screen::portrait_blocked() {
            self.go(flow::pause(self.scene()));
            return;
        }
        // Select(Tab) = 장비 창 바로(0070 6 A — 멈춤을 거치지 않는다, 닫으면 싸움으로).
        if a.equip {
            self.go(Go::Open(Scene::Equip));
            return;
        }
        let a = self.cursor_aim(a);
        self.step_world(&a);
        // 플레이 시간(0081): 싸움 화면의 로직 틱만 센다(멈춤·메뉴·타이틀·개발 판은 빼고). 기록에는 다음 저장 때 같이 들어간다.
        if !self.dev_run_now() {
            self.play_acc += TICK_DT;
            if self.play_acc >= 1.0 {
                self.play_acc -= 1.0;
                self.progress.play = self.progress.play.saturating_add(1);
            }
        }
        let mut out = self.run.tick(&mut self.world, TICK_DT);
        // 주민 말풍선(0122): 같은 주민에게 말풍선이 떠 있는 동안 다시 누르면 다음 말, 아니면 처음 말.
        if let Some(k) = self.run.take_talked() {
            let n = match self.bubble {
                Some((b, n, _)) if b == k => n + 1,
                _ => 0,
            };
            self.bubble = Some((k, n, crate::residents::BUBBLE_TIME));
        }
        if let Some((_, _, t)) = self.bubble.as_mut() {
            *t -= TICK_DT;
        }
        self.bubble = self.bubble.filter(|b| b.2 > 0.0);
        // 이 지역에서 쉰 적이 없어도 이 판에 다른 지역에서 쉬었으면 거기서 다시 선다(0118).
        if out == crate::stage::StageOut::Defeated && self.respawn_elsewhere() {
            out = crate::stage::StageOut::Respawned;
        }
        self.sync_progress(out);
        // 내려가는 길·오르는 길(0118): 지층 단면 전환을 연다.
        let cross = match out {
            crate::stage::StageOut::Descend => Some((self.stage + 1, false)),
            crate::stage::StageOut::Ascend => self.stage.checked_sub(1).map(|s| (s, true)),
            _ => None,
        };
        if let Some((to, up)) = cross.filter(|&(to, _)| to < crate::stage::STAGES.len()) {
            self.snd(crate::sfx::Id::Wake);
            self.passage = Some(crate::descent::Passage::new(
                to,
                up,
                self.run.def.depth_m,
                crate::stage::STAGES[to].depth_m,
            ));
        }
        // 쉼터 화로(0064 M4 → 0104): 쉬면 화로(레벨·가게)가 열린다. 올릴 단·살 것이 있으면 쉼터가 빛난다.
        self.run.shop_ready = crate::growth::can_buy(
            self.world.embers,
            &self.progress.owned,
            &self.progress.growth,
        ) || self.progress.level.can_raise(self.world.embers)
            || self.progress.tree.can_open(self.progress.tree_points());
        if self.run.take_rested() {
            self.snd(crate::sfx::Id::Rest);
            self.go(Go::Open(Scene::Hearth));
        }
        // 문지기가 깨어나는 순간(0064 M5): 낮은 땅울림.
        let waking = matches!(self.run.phase, crate::stage::Phase::Waking(_));
        if waking && !self.was_waking {
            self.snd(crate::sfx::Id::Wake);
        }
        self.was_waking = waking;
        if out == crate::stage::StageOut::Respawned {
            // 쉼터에서 다시 일어섰다(0032): 카메라를 그 자리로 바로(멀리서 날아오지 않게).
            self.cam = Camera::new(self.world.player.pos, &self.t);
        }
        if let Some(next) = flow::after_stage(out) {
            self.ending = 0.0;
            self.go(Go::Root(next));
        }
    }

    /// 개발 아레나(PoC) 한 틱: 숫자 키로 시나리오, 멈춤 = 개발 메뉴로.
    #[cfg(feature = "dev")]
    pub(super) fn tick_sandbox(&mut self, a: Actions) {
        if a.pause {
            self.go(Go::Root(Scene::Title));
            self.go(Go::Open(Scene::Dev { tab: 0 }));
            return;
        }
        if let Some(n) = a.scenario {
            self.sc = if n == 0 {
                scenario::next(self.sc)
            } else {
                scenario::by_number(n).unwrap_or(self.sc)
            };
            self.world = scenario::build(self.sc);
            self.world.set_weapon(self.weapon);
            self.cam = Camera::new(self.world.player.pos, &self.t);
        }
        let a = self.cursor_aim(a);
        self.step_world(&a);
    }

    /// 커서 → 땅 위의 점(0058): 지난 프레임 화면 자리로 NDC, 카메라 리그의 시선이 플레이어 발 높이 면에 닿는 점.
    /// 락온 중이면 쓰지 않는다(오른쪽 스틱처럼 — 대상을 본다). 화면 밖이면 없음.
    pub(super) fn cursor_aim(&self, mut a: Actions) -> Actions {
        a.aim_at = None;
        let (Some(c), Some(f)) = (a.cursor, self.frame) else {
            return a;
        };
        if self.world.lock_target().is_some() || f.w < 1.0 || f.h < 1.0 {
            return a;
        }
        let ndc = vec2(
            (c.x - f.ox) / f.w * 2.0 - 1.0,
            1.0 - (c.y - f.oy) / f.h * 2.0,
        );
        if ndc.x.abs() > 1.0 || ndc.y.abs() > 1.0 {
            return a;
        }
        let rig = &self.cam.rig;
        let dir = crate::camera::screen_ray(rig, f.w / f.h, ndc);
        a.aim_at = crate::camera::ground_hit(rig.pos, dir, self.world.player.pos.y)
            .map(|p| vec2(p.x, p.z));
        a
    }

    pub(super) fn step_world(&mut self, a: &Actions) {
        let (cam, t) = (&mut self.cam, &self.t);
        self.world.view_yaw = Some(cam.quarter_yaw_deg(t));
        use crate::tuning::{
            KICK_HIT_DEG, KICK_SMASH_DEG, SHAKE_CHARGED, SHAKE_CRIT, SHAKE_DOWN, SHAKE_HIT,
            SHAKE_HURT, SHAKE_PARRY, SHAKE_PLUNGE, SHAKE_ROUSE, SHAKE_SLAM, SHAKE_SMASH,
            SHAKE_TURN,
        };
        let events = self.world.step(a, t);
        for &e in &events {
            let kick = match e {
                Event::Hit { .. } => SHAKE_HIT,
                Event::PlayerHurt => SHAKE_HURT,
                Event::EnemyDown { .. } => SHAKE_DOWN,
                // 땅 돌리기: 카메라가 90° 돈다(입력 기준은 다음 틱부터 새 방위 — 쥔 스틱은 월드가 그대로 둔다).
                Event::Turn { quarters } => {
                    cam.turn(quarters, t);
                    SHAKE_TURN
                }
                Event::Slam { .. } => SHAKE_SLAM,
                Event::Rouse { .. } => SHAKE_ROUSE,
                Event::Parry { .. } => SHAKE_PARRY,
                Event::Deflect { .. } => SHAKE_HIT,
                Event::Crit { .. } => SHAKE_CRIT,
                Event::Plunge { .. } => SHAKE_PLUNGE,
                Event::Smash { .. } => SHAKE_SMASH,
                Event::Charged => SHAKE_CHARGED,
                Event::Gulp { .. } | Event::FlaskDenied | Event::Ember { .. } => 0.0,
                Event::Cast { .. } | Event::CastDenied | Event::Throw => 0.0,
                Event::StaminaDenied | Event::Release { .. } => 0.0,
                Event::Break { .. } => SHAKE_SMASH,
                Event::Aftershock { .. } | Event::Boom { .. } => SHAKE_PLUNGE,
                Event::Perfect { .. } => SHAKE_CHARGED,
                Event::Swing | Event::LockOn | Event::LockOff | Event::Beat => 0.0,
            };
            // 화면 흔들림 설정(0052): 0..=10 → 0..100%.
            cam.trauma = (cam.trauma + kick * self.shake as f32 / 10.0).min(1.0);
            // 카메라 킥(2026-11-06, KB melee-light…: 작은 타격은 trauma²에서 0이 된다 — 맞은 쪽으로 2~3px 찼다 돌아온다).
            let punch = match e {
                Event::Hit { at } | Event::Crit { at } => Some((at, KICK_HIT_DEG)),
                Event::Smash { at } | Event::Break { at } => Some((at, KICK_SMASH_DEG)),
                _ => None,
            };
            if let Some((at, deg)) = punch {
                let d = at - self.world.player.pos;
                cam.kick(vec3(d.x, 0.0, d.z), deg * self.shake as f32 / 10.0);
            }
        }
        self.world_sounds(&events);
        self.follow_camera();
    }

    pub(super) fn follow_camera(&mut self) {
        let target = self.world.lock_target();
        self.cam.set_mode(
            if target.is_some() {
                Mode::Lock
            } else {
                Mode::Quarter
            },
            target,
            self.world.player.pos,
        );
        self.cam.update(
            TICK_DT,
            &self.t,
            self.world.player.pos,
            target,
            &self.world.obstacles,
        );
    }

    pub(super) fn in_stage(&self) -> bool {
        match self.scene() {
            #[cfg(feature = "dev")]
            Scene::Sandbox => false,
            _ => true,
        }
    }

    pub(super) fn draw_goal_arrow(&mut self, f: &Frame) {
        let mark = self.run.goal().and_then(|g| {
            let (ndc, front) =
                crate::camera::world_to_ndc(&self.cam.rig, f.w / f.h.max(1.0), g + Vec3::Y);
            crate::camera::edge_marker(ndc, front, GOAL_ARROW_INSET)
        });
        self.goal_off = match mark {
            Some(_) => self.goal_off + get_frame_time(),
            None => 0.0,
        };
        if let (Some((at, ang)), true) = (mark, self.goal_off > GOAL_ARROW_DELAY) {
            let p = vec2(
                f.ox + (at.x + 1.0) * 0.5 * f.w,
                f.oy + (1.0 - at.y) * 0.5 * f.h,
            );
            hud::draw_goal_arrow(f, p, -ang, self.world.time);
        }
    }

    /// 주민 말풍선(0122, nail 말 걸기 말풍선 꼴): 그 주민 머리 위, 판 + 테두리 + 짧은 말. 처음·끝에 옅게.
    pub(super) fn draw_bubble(&mut self, f: &Frame) {
        let Some((k, n, t)) = self.bubble else { return };
        let Some(&(i, at)) = self.run.residents.get(k) else {
            return;
        };
        let r = &crate::residents::RESIDENTS[i];
        let changed = self.run.resident_changed.get(k).copied().unwrap_or(false);
        let text = tr(crate::residents::line(r, changed, n));
        let (ndc, front) =
            crate::camera::world_to_ndc(&self.cam.rig, f.w / f.h.max(1.0), at + Vec3::Y * 1.75);
        if !front {
            return;
        }
        let a = (t / 0.25)
            .min((crate::residents::BUBBLE_TIME - t) / 0.15)
            .clamp(0.0, 1.0);
        let k = 1.0;
        let (tw, th) = (self.ui.width(text, k), self.ui.px(k));
        let pad = 5.0 * f.ui;
        let cx = (f.ox + (ndc.x + 1.0) * 0.5 * f.w)
            .clamp(f.ox + tw * 0.5 + pad, f.ox + f.w - tw * 0.5 - pad);
        let bottom = (f.oy + (1.0 - ndc.y) * 0.5 * f.h).max(f.oy + th + pad * 3.0);
        let rect = Rect::new(
            cx - tw * 0.5 - pad,
            bottom - th - pad * 2.0,
            tw + pad * 2.0,
            th + pad * 2.0,
        );
        draw_rectangle(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            Color::new(0.08, 0.07, 0.11, 0.86 * a),
        );
        draw_rectangle_lines(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            1.5 * f.ui,
            Color::new(0.85, 0.72, 0.5, 0.9 * a),
        );
        // 꼬리(말하는 쪽 아래로).
        draw_triangle(
            vec2(cx - 4.0 * f.ui, rect.bottom()),
            vec2(cx + 4.0 * f.ui, rect.bottom()),
            vec2(cx, rect.bottom() + 5.0 * f.ui),
            Color::new(0.85, 0.72, 0.5, 0.9 * a),
        );
        self.ui
            .text_center(text, cx, rect.y + pad, k, Color::new(0.95, 0.92, 0.85, a));
    }

    /// 플레이어 머리 위의 화면 자리(카메라 뒤면 없음).
    fn head_on_screen(&self, f: &Frame) -> Option<Vec2> {
        let pl = &self.world.player;
        let head = pl.pos + Vec3::Y * (pl.air + 1.9);
        let (ndc, front) = crate::camera::world_to_ndc(&self.cam.rig, f.w / f.h.max(1.0), head);
        front.then(|| {
            vec2(
                f.ox + (ndc.x + 1.0) * 0.5 * f.w,
                f.oy + (1.0 - ndc.y) * 0.5 * f.h,
            )
        })
    }

    /// 머리 위 표시: 박자 타의 음표(0076), 기력이 모자라 막힌 땀방울(0088).
    pub(super) fn draw_beat_note(&mut self, f: &Frame) {
        let Some(p) = self.head_on_screen(f) else {
            return;
        };
        let pl = &self.world.player;
        if pl.since_beat < hud::BEAT_NOTE_LIFE {
            hud::draw_beat_note(f, p, pl.since_beat / hud::BEAT_NOTE_LIFE, pl.beat_chain);
        }
        if pl.stamina_denied > 0.0 {
            let k = 1.0 - pl.stamina_denied / crate::tuning::STAMINA_DENIED_SHOW;
            hud::draw_sweat(f, p, k.clamp(0.0, 1.0));
        }
    }

    /// 월드 사건·상태 변화 → 소리(0064 M5). 월드는 소리를 모른다 — 여기서만 듣는다.
    pub(super) fn world_sounds(&mut self, events: &[Event]) {
        use crate::sfx::Id;
        for e in events {
            let id = match e {
                Event::Swing => Id::Swing,
                Event::Hit { .. } | Event::Crit { .. } => Id::Hit,
                Event::Plunge { .. } | Event::Smash { .. } | Event::Aftershock { .. } => Id::Hit,
                Event::Break { .. } | Event::Boom { .. } => Id::Hit,
                Event::EnemyDown { .. } => Id::Die,
                Event::PlayerHurt => Id::Hurt,
                // 등딱지 튕김(0120)은 패링의 거울 — 같은 쇳소리.
                Event::Parry { .. } | Event::Deflect { .. } => Id::Parry,
                Event::Beat => crate::sfx::beat_id(self.world.player.beat_chain),
                // 딱 맞춰 떼기(0078): 박자와 같은 "맞췄다" 종 — 가장 높은 음.
                Event::Perfect { .. } => Id::Beat3,
                Event::Ember { .. } => Id::Ember,
                Event::Gulp { .. } => Id::Gulp,
                // 지연 공격이 풀린다(0128): 짧게 들이쉬는 소리 — 이 뒤 `RELEASE_LEAD`초에 맞는다.
                Event::Release { .. } => Id::TeleRelease,
                Event::FlaskDenied | Event::CastDenied | Event::StaminaDenied => Id::Deny,
                _ => continue,
            };
            self.snd(id);
            if id == Id::Hit {
                self.snd(Id::Thwack);
            }
        }
        // 회피를 시작한 틱.
        let dodging = matches!(self.world.player.state, crate::world::PState::Dodge { .. });
        if dodging && !self.was_dodging {
            self.snd(Id::Dodge);
        }
        self.was_dodging = dodging;
        // 적의 예고가 시작한 틱: 모양마다 다른 소리(소리만 듣고도 가르게, 0064 R4).
        let n = self.world.enemies.len();
        self.tele_was.resize(n, u32::MAX);
        let mut cues = Vec::new();
        for (i, en) in self.world.enemies.iter().enumerate() {
            let tele = en.telegraph();
            // 공격 번호로 센다(0128): 연속 공격의 다음 타·페인트로 바뀐 공격도 제 소리를 낸다.
            let fresh = tele.is_some() && self.tele_was[i] != en.atk_count;
            if let (Some((_, shape)), true) = (tele, fresh) {
                cues.push(match shape {
                    crate::enemy::TeleShape::Disc { .. } => Id::TeleDisk,
                    crate::enemy::TeleShape::Sector { .. } => Id::TeleFan,
                    crate::enemy::TeleShape::Lane { .. } => Id::TeleLane,
                    crate::enemy::TeleShape::Ring { .. } => Id::TeleRing,
                });
            }
            if tele.is_some() {
                self.tele_was[i] = en.atk_count;
            }
        }
        for id in cues {
            self.snd(id);
        }
    }

    pub(super) fn dev_sandbox(&self) -> bool {
        #[cfg(feature = "dev")]
        if self.scene() == Scene::Sandbox {
            return true;
        }
        false
    }

    /// 개발 줄(기본 글꼴, 영어 — 개발 전용 화면은 예외, 첫 게임 0013): 측정·시나리오.
    pub(super) fn draw_dev(&self, f: &Frame) {
        let p = &self.perf;
        let mode = match self.cam.mode {
            Mode::Quarter => "QUARTER",
            Mode::Lock => "LOCK-ON",
        };
        let y = f.oy + f.h - 40.0;
        #[cfg(feature = "dev")]
        if self.scene() == Scene::Sandbox {
            let e = scenario::entry(self.sc);
            draw_text(
                format!(
                    "{} [{}]  checks: {}  (1-9 scenario, ESC title)",
                    e.name, mode, e.checks
                ),
                f.ox + 10.0,
                y - 20.0,
                16.0,
                GRAY,
            );
        }
        draw_text(
            format!(
                "{} {mode}{} cam {:.0}deg fps {}  cpu {:.1}ms (max {:.1})  frame {:.1}ms  batches {} (max {})  verts {}",
                self.world.name,
                self.cam.blending().map(|e| format!(" blend {e:.2}s")).unwrap_or_default(),
                self.cam.azimuth_deg(),
                get_fps(),
                p.cpu_avg_ms,
                p.cpu_max_ms,
                p.frame_avg_ms,
                p.draws,
                p.draws_max,
                self.renderer.vertices()
            ),
            f.ox + 10.0,
            y,
            16.0,
            YELLOW,
        );
    }
}
