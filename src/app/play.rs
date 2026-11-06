//! 싸움 한 틱과 그 둘레(0019): 월드·스테이지·카메라·조준·랩, 길 안내 화살, 개발 줄, 월드 소리(0066 R3에서 옮김).
use super::*;

impl App {
    /// 싸움 한 틱: 멈춤 → 월드 → 스테이지 → 기록·가게·소리·알림 → 끝났으면 끝 장면.
    pub(super) fn tick_play(&mut self, a: Actions) {
        if a.pause {
            self.go(flow::pause(self.scene()));
            return;
        }
        // Select(Tab) = 장비 창 바로(0070 6 A — 멈춤을 거치지 않는다, 닫으면 싸움으로).
        if a.equip {
            self.go(Go::Open(Scene::Equip));
            return;
        }
        let a = self.lab_filter(a);
        let a = self.cursor_aim(a);
        self.step_world(&a);
        let out = self.run.tick(&mut self.world, TICK_DT);
        self.sync_progress(out);
        // 쉼터 화로(0064 M4): 쉬면 가게가 열린다. 살 것이 있으면 쉼터가 빛난다.
        self.run.shop_ready = crate::growth::can_buy(
            self.world.embers,
            &self.progress.owned,
            &self.progress.growth,
        );
        if self.run.take_rested() {
            self.snd(crate::sfx::Id::Rest);
            self.go(Go::Open(Scene::Shop));
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
            self.go(Go::Open(Scene::Dev));
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
        let a = self.lab_filter(a);
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

    /// 랩이 열려 있으면 커서 키는 랩이 먹고, 이동·공격·회피·락온은 그대로 게임으로(바꾸며 바로 느끼기).
    pub(super) fn lab_filter(&mut self, a: Actions) -> Actions {
        if !self.lab.open {
            return a;
        }
        if let Some(text) = self.lab.update(&a, &mut self.t) {
            info!("--- tuning export ---\n{}", text);
        }
        Actions {
            move_dir: a.move_dir,
            aim: a.aim,
            cursor: a.cursor,
            attack: a.attack,
            attack_held: a.attack_held,
            cast: a.cast,
            cast_held: a.cast_held,
            flask: a.flask,
            dodge: a.dodge,
            parry: a.parry,
            jump: a.jump,
            lock: a.lock,
            turn_right: a.turn_right,
            turn_left: a.turn_left,
            ..Actions::default()
        }
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
                Event::Crit { .. } => SHAKE_CRIT,
                Event::Plunge { .. } => SHAKE_PLUNGE,
                Event::Smash { .. } => SHAKE_SMASH,
                Event::Charged => SHAKE_CHARGED,
                Event::Gulp { .. } | Event::FlaskDenied | Event::Ember { .. } => 0.0,
                Event::Cast { .. } | Event::CastDenied | Event::Throw => 0.0,
                Event::Break { .. } => SHAKE_SMASH,
                Event::Aftershock { .. } | Event::Boom { .. } => SHAKE_PLUNGE,
                Event::Perfect { .. } => SHAKE_CHARGED,
                Event::Swing | Event::LockOn | Event::LockOff => 0.0,
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
                Event::Parry { .. } => Id::Parry,
                Event::Ember { .. } => Id::Ember,
                Event::Gulp { .. } => Id::Gulp,
                Event::FlaskDenied | Event::CastDenied => Id::Deny,
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
        self.tele_was.resize(n, false);
        let mut cues = Vec::new();
        for (i, en) in self.world.enemies.iter().enumerate() {
            let tele = en.telegraph();
            if let (Some((_, shape)), false) = (tele, self.tele_was[i]) {
                cues.push(match shape {
                    crate::enemy::TeleShape::Disc { .. } => Id::TeleDisk,
                    crate::enemy::TeleShape::Sector { .. } => Id::TeleFan,
                    crate::enemy::TeleShape::Lane { .. } => Id::TeleLane,
                    crate::enemy::TeleShape::Ring { .. } => Id::TeleRing,
                });
            }
            self.tele_was[i] = tele.is_some();
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
                    "{} [{}]  checks: {}  (1-7 scenario, ESC title)",
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
