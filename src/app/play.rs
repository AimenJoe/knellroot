//! 싸움 한 틱과 그 둘레(0019): 월드·스테이지·카메라·조준·랩, 길 안내 화살, 개발 줄, 월드 소리(0066 R3에서 옮김).
use super::*;

/// 새긴 돌 글 판이 떠 있는 초(0129).
const STONE_TIME: f32 = 6.0;

impl App {
    /// 싸움 한 틱(0251 R5에서 단계로): 연출 → 멈춤·장비 창 → 월드 → 쓰러짐 문 → 스테이지 → 말풍선·새긴 돌 → 스테이지가 낸 것.
    pub(super) fn tick_play(&mut self, a: Actions) {
        // 연출(0251 R5 — 굴·핵의 반응·쓰러짐·깨어남·불 붙음): 그동안 조작이 없다. 깨어나는 장면만 세로 폰이면 멈춤이 먼저(0082).
        if self.tick_interlude(&a) {
            return;
        }
        // 세로로 든 폰(0082): 회전 안내가 덮인 동안 싸움이 흐르지 않게 멈춤으로(nail 0269 꼴).
        // 앱이 내려갔다 오면(0257, nail 0298) 멈춤 창이 열려 있다 — 소리 없이(돌아온 순간 확인음이 나지 않게).
        if a.pause || crate::screen::portrait_blocked() || std::mem::take(&mut self.went_away) {
            self.show(flow::pause(self.scene(), self.pause_tab));
            return;
        }
        // Select(Tab) = 큰 창의 장비 탭 바로(0070 6 A → 0164, 닫으면 싸움으로).
        if a.equip {
            self.show(Go::Open(flow::SELECT_OPENS));
            return;
        }
        self.play_world(a);
        if self.play_fell() {
            return;
        }
        self.tell_bag();
        let out = self.run.tick(&mut self.world, TICK_DT);
        self.play_talk();
        self.play_stage_out(out);
    }

    /// 월드의 수평 방향 → 지금 입력 기준의 스틱(쿼터뷰 위 = `quarter_forward`, 오른쪽 = 앞 × 위 — `world::move_to_world`의 거꾸로).
    fn stick_toward(&self, want: Vec3) -> Vec2 {
        let fwd = crate::camera::quarter_forward(self.cam.basis_yaw_deg(&self.t));
        let right = fwd.cross(Vec3::Y);
        let want = vec3(want.x, 0.0, want.z).normalize_or_zero();
        vec2(want.dot(right), want.dot(fwd))
    }

    /// 헤드리스 시험의 도움(0262 — 개발판, 시험 깃발 4): 서막 숲에서 방랑자가 숲 끝까지 저절로 걷는다 — 적의 길찾기 거리 지도를
    /// 따라가고 턱 앞에서는 뛴다. 키보드로 숲길(쿼터뷰가 35° 돌아 있다)을 시간으로 걷게 하면 느린 러너에서 어긋난다.
    #[cfg(feature = "dev")]
    fn prologue_autopilot(&self, a: &mut Actions) {
        if crate::screen::test_flags() & 4 == 0 || !self.run.def.prologue {
            return;
        }
        let Some(exit) = self.run.exit_at else { return };
        let nav = crate::nav::Nav::build(
            self.world.ground.as_ref(),
            &self.world.obstacles,
            self.run.level.half_extent(),
        );
        let dist = nav.flow(exit);
        let p = self.world.player.pos;
        let r = crate::tuning::PLAYER_RADIUS;
        let to = match nav.hop(&dist, p, r) {
            Some((land, _)) => {
                a.jump = self.world.player.air <= 0.0;
                land
            }
            None => nav.waypoint(&dist, p, r).unwrap_or(exit),
        };
        a.move_dir = self.stick_toward(to - p);
    }

    /// 월드 한 틱 + 줍기 몸짓 + 플레이 시간.
    fn play_world(&mut self, a: Actions) {
        #[cfg(feature = "dev")]
        let a = {
            let mut a = a;
            self.prologue_autopilot(&mut a);
            a
        };
        let a = self.camera_input(a);
        let a = self.cursor_aim(a);
        self.step_world(&a);
        // 줍기 몸짓(0196): 서서 가만히 있는 동안만 흐르고, 다 돌았거나 움직이면 그친다.
        let pl = &self.world.player;
        let still =
            matches!(pl.state, crate::world::PState::Idle) && pl.pos.distance(pl.prev) < 1e-4;
        self.gesture = self
            .gesture
            .map(|(n, g)| (n, g + TICK_DT))
            .filter(|(n, g)| still && *g < crate::render::gesture_len(n));
        // 플레이 시간(0081): 싸움 화면의 로직 틱만 센다(멈춤·메뉴·타이틀·개발 판은 빼고). 기록에는 다음 저장 때 같이 들어간다.
        if !self.dev_run_now() {
            self.play_acc += TICK_DT;
            if self.play_acc >= 1.0 {
                self.play_acc -= 1.0;
                self.progress.play = self.progress.play.saturating_add(1);
            }
        }
    }

    /// 체력이 다했으면 쓰러짐을 연다 — 그 틱의 스테이지는 흐르지 않는다.
    fn play_fell(&mut self) -> bool {
        // 체력이 다했다(0175): 쓰러짐을 연다 — 쉼터로 옮기는 것(스테이지가 하던 것)은 그 장면의 어둠 속에서(`rise_after_fall`).
        if self.world.player.hp <= 0 {
            self.interlude = Some(interlude::Interlude::Fall(crate::fall::Fall::default()));
            return true;
        }
        false
    }

    /// 주민 말풍선·새긴 돌 글 판(스테이지가 이 틱에 알린 것).
    fn play_talk(&mut self) {
        // 주민 말풍선(0122): 같은 주민에게 말풍선이 떠 있는 동안 다시 누르면 다음 말, 아니면 처음 말.
        if let Some(k) = self.run.take_talked() {
            let n = match self.bubble {
                Some((b, n, _)) if b == k => n + 1,
                _ => 0,
            };
            self.bubble = Some((k, n, crate::residents::BUBBLE_TIME));
        }
        // 새긴 돌(0129): 읽으면 글 판, 떠 있는 동안 같은 돌을 다시 누르면 닫힌다.
        if let Some(k) = self.run.take_read() {
            // 여정(0183): 읽은 새긴 돌을 기록에(판을 넘어 남는다).
            let id = crate::journey::read_id(self.run.def.find_prefix, k);
            if !self.progress.read.contains(&id) {
                self.progress.read.push(id);
                self.progress.read.sort();
            }
            self.stone_card = match self.stone_card {
                Some((b, _)) if b == k => None,
                _ => Some((k, STONE_TIME)),
            };
        }
        if let Some((_, t)) = self.stone_card.as_mut() {
            *t -= TICK_DT;
        }
        self.stone_card = self.stone_card.filter(|c| c.1 > 0.0);
        if let Some((_, _, t)) = self.bubble.as_mut() {
            *t -= TICK_DT;
        }
        self.bubble = self.bubble.filter(|b| b.2 > 0.0);
    }

    /// 스테이지가 낸 것: 다른 지역에서 다시 서기 · 기록 · 뿌리 굴 · 화로 · 문지기 깸 · 끝.
    fn play_stage_out(&mut self, mut out: crate::stage::StageOut) {
        // 이 지역에서 쉰 적이 없어도 이 판에 다른 지역에서 쉬었으면 거기서 다시 선다(0118).
        if out == crate::stage::StageOut::Defeated && self.respawn_elsewhere() {
            out = crate::stage::StageOut::Respawned;
        }
        self.sync_progress(out);
        // 내려가는 길·오르는 길(0118 → 0174): 뿌리 굴 전환을 연다.
        let cross = match out {
            crate::stage::StageOut::Descend => Some((self.stage + 1, false)),
            crate::stage::StageOut::Ascend => self.stage.checked_sub(1).map(|s| (s, true)),
            _ => None,
        };
        if let Some((to, up)) = cross.filter(|&(to, _)| to < crate::stage::REGIONS) {
            self.snd(crate::sfx::Id::Wake);
            self.interlude = Some(interlude::Interlude::Passage(crate::descent::Passage::new(
                to,
                up,
                self.run.def.depth_m,
                crate::stage::STAGES[to].depth_m,
            )));
        }
        // 서막 숲 끝(0262): "성소에 들다" — 다 어두울 때 지역을 성소로(`arrive_sanctuary`).
        if out == crate::stage::StageOut::Onward && self.interlude.is_none() {
            self.interlude = Some(interlude::Interlude::Arrive(Default::default()));
        }
        // 쉼터 화로(0064 M4 → 0104): 쉬면 화로(레벨·가게)가 열린다. 올릴 단·살 것이 있으면 쉼터가 빛난다.
        self.run.shop_ready = crate::growth::can_buy(
            self.world.embers,
            &self.progress.owned,
            &self.progress.growth,
            &self.progress.shields,
            !self.progress.bag.full(),
        ) || self.progress.level.can_raise(self.world.embers)
            || self.progress.board.can_buy(self.progress.skill_points())
            || self.can_forge();
        if self.run.take_rested() {
            self.snd(crate::sfx::Id::Rest);
            if self.run.take_kindled() {
                // 꺼진 화로에서 처음 쉰다(0171): 불이 붙는다 — 기록에 적고, 그 순간을 본 뒤에 창을 연다.
                self.progress.lit = true;
                self.persist();
                if let Some((i, _)) = self.run.rest_at {
                    self.fx.kindle(self.run.rests[i]);
                }
                self.interlude = Some(interlude::Interlude::Kindle(interlude::KINDLE_TIME));
            } else {
                self.open_hearth();
            }
        }
        // 문지기가 깨어나는 순간(0064 M5): 낮은 땅울림.
        let waking = matches!(self.run.phase, crate::stage::Phase::Waking(_));
        if waking && !self.heard.was_waking {
            self.snd(crate::sfx::Id::Wake);
        }
        self.heard.was_waking = waking;
        if out == crate::stage::StageOut::Respawned {
            // 쉼터에서 다시 일어섰다(0032): 카메라를 그 자리로 바로(멀리서 날아오지 않게).
            self.cam = self.cam.renew(self.world.player.pos, &self.t);
        }
        // 마지막 핵을 쥐었다(0216 W3): 끝 장면 전에 핵의 반응 — 세계가 멈추고 잔상·먼 종소리.
        if out == crate::stage::StageOut::Cleared {
            self.interlude = Some(interlude::Interlude::Echo(0.0));
            return;
        }
        if let Some(next) = flow::after_stage(out) {
            self.ending = 0.0;
            self.go(Go::Root(next));
        }
    }

    /// 연출 칸 한 틱(0251 R5): 무엇을 할지는 순수한 `Interlude::step`이 정하고, 여기서는 그 차례대로 한다.
    /// 연출이 이 틱을 가졌으면 참(싸움은 흐르지 않는다). 깨어나는 장면은 세로 폰이면 쉬고(멈춤 창이 먼저 — 예전 차례 그대로).
    fn tick_interlude(&mut self, a: &Actions) -> bool {
        let Some(now) = self.interlude else {
            return false;
        };
        if now.waits_for_landscape() && crate::screen::portrait_blocked() {
            return false;
        }
        let touch = crate::input::device() == crate::input::Device::Touch;
        let (next, beat) = now.step(interlude::Input { actions: *a, touch }, TICK_DT);
        // 다음 칸을 세계를 움직이기 전에 둔다(깨어나는 동안의 튜토리얼·종소리 길 안내가 "깨어나는 중"을 읽는다 — 예전 차례).
        self.interlude = next;
        if let Some((to, up)) = beat.cross {
            self.cross_to(to, up);
        }
        if beat.arrive {
            self.arrive_sanctuary();
        }
        // "성소에 들다"(0262)의 카메라 자리 — 그 장면이 아니면 없다.
        self.cam.shot = match self.interlude {
            Some(interlude::Interlude::Arrive(ar)) if !ar.arrived() => {
                // 아치 앞(0265): 표지 뒤 낮은 자리에서 아치를 올려다본다.
                let at = self.run.exit_at.unwrap_or(self.world.player.pos);
                let gate = self.run.level.arch().unwrap_or(at + Vec3::NEG_Z * 4.0);
                let (eye, look) = ar.gate_eye(at, gate);
                Some((eye, look, ar.gate_weight()))
            }
            Some(interlude::Interlude::Arrive(ar)) if ar.arrived() => {
                let at = self
                    .run
                    .level
                    .marks_of(crate::stage::ARRIVE_GLYPH)
                    .next()
                    .unwrap_or(self.run.level.player_start);
                let far = self.run.level.gate.map_or(at + Vec3::NEG_Z * 60.0, |g| g.1);
                let (eye, look) = ar.eye(at, far);
                Some((eye, look, ar.weight()))
            }
            _ => None,
        };
        if let Some(id) = beat.sound {
            self.snd(id);
        }
        if let Some(c) = beat.close {
            self.cam.close = c;
        }
        if beat.hold_intro {
            self.run.intro = crate::stage::INTRO_TIME;
        }
        match beat.hold {
            interlude::Hold::Untouched => {}
            interlude::Hold::Freeze => {
                // 세계가 멈춘 동안: 보간이 지난 틱과 이번 틱 사이를 오가지 않게 서 있는 자리로(KB 고정 타임스텝 — 히트스톱과 같다).
                self.world.player.prev = self.world.player.pos;
                for e in &mut self.world.enemies {
                    e.prev = e.pos;
                }
                self.follow_camera();
            }
            interlude::Hold::Idle => self.step_world(&Actions::default()),
            interlude::Hold::Walk(k) => {
                // 성소 쪽(북, 월드 −z)으로 걷는다 — 아치가 있으면 그 가운데로 향해(아치 앞 마당 옆에서 트리거됐어도 설주에 걸리지 않게,
                // 2026-11-23), 아치를 지나면 곧장 북쪽.
                let p = self.world.player.pos;
                let want = match self.run.level.arch() {
                    Some(a) if p.z > a.z + 0.5 => vec3(a.x - p.x, 0.0, (a.z - 1.0) - p.z),
                    _ => Vec3::NEG_Z,
                };
                let a = Actions {
                    move_dir: self.stick_toward(want) * k,
                    ..Actions::default()
                };
                self.step_world(&a);
            }
            interlude::Hold::IdleStage => {
                self.step_world(&Actions::default());
                let _ = self.run.tick(&mut self.world, TICK_DT);
            }
        }
        match beat.then {
            interlude::Then::Nothing => {}
            interlude::Then::Rise => {
                // 쓰러짐의 어둠 속: 쉼터 곁으로. 쉰 적이 없으면 끝 장면으로 갔다 — 연출은 거기서 끝.
                self.rise_after_fall();
                if self.scene() != Scene::Play {
                    self.interlude = None;
                }
                return true;
            }
            interlude::Then::Hearth => self.open_hearth(),
            interlude::Then::Ending => {
                self.ending = 0.0;
                self.go(Go::Root(Scene::Ending { won: true }));
            }
            interlude::Then::Woke => {
                self.progress.woke = true;
                self.persist();
            }
        }
        if beat.release_close {
            self.cam.close = 0.0;
        }
        true
    }

    /// 쓰러짐의 어둠 속(0175): 스테이지가 쉼터 곁으로 옮긴다(쉰 적이 없으면 다른 지역의 쉼터 · 거기도 없으면 끝 장면).
    /// 그전에는 쓰러진 그 틱에 이 일을 했다 — 이제 쓰러짐 장면이 다 어두워진 순간에.
    fn rise_after_fall(&mut self) {
        let mut out = self.run.tick(&mut self.world, TICK_DT);
        if out == crate::stage::StageOut::Defeated && self.respawn_elsewhere() {
            out = crate::stage::StageOut::Respawned;
        }
        self.sync_progress(out);
        if out == crate::stage::StageOut::Respawned {
            let close = self.cam.close;
            self.cam = self.cam.renew(self.world.player.pos, &self.t);
            self.cam.close = close;
        }
        if let Some(next) = flow::after_stage(out) {
            self.ending = 0.0;
            self.go(Go::Root(next));
        }
    }

    /// 쉬었다: 큰 창이 화로로 열린다(0164 — 아는 스킬이 없고 배울 점수가 있으면 기술 탭에서, 0166).
    fn open_hearth(&mut self) {
        let (board, total) = (self.board(), self.progress.skill_points());
        self.show(Go::Open(flow::hearth_opens(
            board.knows_a_skill(),
            board.can_buy(total),
        )));
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
            self.dev.sc = if n == 0 {
                scenario::next(self.dev.sc)
            } else {
                scenario::by_number(n).unwrap_or(self.dev.sc)
            };
            self.world = scenario::build(self.dev.sc);
            self.fx.clear();
            self.arm_world();
            self.apply_board();
            self.cam = self.cam.renew(self.world.player.pos, &self.t);
        }
        let a = self.camera_input(a);
        let a = self.cursor_aim(a);
        self.step_world(&a);
    }

    /// 시점 입력(0181): 거리 단계(십자 ↑·↓ · 키보드 ↑·↓ · 휠 · 꼬집기)를 카메라에, 자유 숄더뷰면 오른쪽 스틱·키보드 ←·→·잡힌 마우스·
    /// 화면 끌기를 시점 돌리기로 — 그때 오른쪽 스틱은 조준이 아니다(월드에 넘기지 않는다). 입력에 지금이 자유 숄더뷰인지 알린다(커서 잡기).
    pub(super) fn camera_input(&mut self, mut a: Actions) -> Actions {
        use crate::tuning::{
            FREE_MOUSE_DEG_PER_PX, FREE_STICK_PITCH_DEG, FREE_STICK_YAW_DEG, FREE_TOUCH_DEG_PER_PX,
        };
        let player = self.world.player.pos;
        for _ in 0..a.zoom.unsigned_abs().min(3) {
            self.cam.step_zoom(a.zoom.signum(), player);
        }
        if a.view {
            self.cam.toggle_view(player);
        }
        if self.cam.mode == Mode::Free {
            let px = if crate::input::device() == crate::input::Device::Touch {
                FREE_TOUCH_DEG_PER_PX
            } else {
                FREE_MOUSE_DEG_PER_PX
            }
            .to_radians();
            // 설정의 민감도(0191 — 스틱·마우스·끌기 모두에 같은 배율).
            let sens = |i: u8| crate::prefs::LOOK_SENS[i as usize] as f32 / 100.0;
            let (ky, kp) = (sens(self.opts.look_sens.0), sens(self.opts.look_sens.1));
            // 터치로 끌어 겨누는 손가락(0217)은 시점이 아니라 조준이다 — 숄더뷰에서도 그대로 넘긴다.
            let stick = if a.aim_drag { Vec2::ZERO } else { a.aim };
            let dyaw = (stick.x * FREE_STICK_YAW_DEG.to_radians() * TICK_DT + a.look.x * px) * ky;
            let dpitch =
                (-stick.y * FREE_STICK_PITCH_DEG.to_radians() * TICK_DT + a.look.y * px) * kp;
            self.cam.look(dyaw, dpitch);
            if !a.aim_drag {
                a.aim = Vec2::ZERO;
            }
            a.cursor = None;
            // 숄더뷰의 원거리는 카메라가 보는 쪽으로(0219 — 사장 "지금은 캐릭터가 움직이는 방향으로 쏘니까 이상함").
            a.aim_ahead = true;
        }
        // 패드 조준 보조(0219): 엄지로 겨눈 것만 — 마우스·터치 끌기는 손이 정확하다.
        a.aim_assist = if crate::input::device() == crate::input::Device::Pad {
            crate::tuning::PAD_AIM_ASSIST_DEG
        } else {
            0.0
        };
        crate::input::set_free_look(self.cam.mode == Mode::Free);
        a.zoom = 0;
        a.look = Vec2::ZERO;
        a.view = false;
        a
    }

    /// 커서 → 땅 위의 점(0058): 지난 프레임 화면 자리로 NDC, 카메라 리그의 시선이 플레이어 발 높이 면에 닿는 점.
    /// 락온 중이면 쓰지 않는다(오른쪽 스틱처럼 — 대상을 본다). 화면 밖이면 없음.
    pub(super) fn cursor_aim(&self, mut a: Actions) -> Actions {
        a.aim_at = None;
        a.aim_sight = None;
        // 숄더뷰 크로스헤어(0222): 화면 가운데의 시선이 닿는 적이 있으면 그 적 쪽으로 — 원거리가 "그곳으로" 간다.
        // 커서(`aim_at`)가 아니라 `aim_sight`로(0227): `aim_at`은 몸을 돌리고 근접 자동 조준의 원뿔을 정한다 — 크로스헤어가 적을
        // 지날 때마다 걷던 몸이 그 적에게 돌아갔다.
        if self.cam.mode == Mode::Free {
            a.aim_sight = self.crosshair_target().map(|p| vec2(p.x, p.z));
            return a;
        }
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

    /// 숄더뷰 크로스헤어(0222)가 겨누는 적: 화면 가운데에서 나가는 시선(카메라 리그)이 지나는 깨어 있는 적의 몸(가운데 높이,
    /// 반지름 몸 + 볼트 몫) 가운데 가장 가까운 것 — 지금 원거리의 사거리 안. 없으면 None(카메라 앞으로 곧게 쏜다, `aim_ahead`).
    pub(super) fn crosshair_target(&self) -> Option<Vec3> {
        let f = self.frame?;
        if f.w < 1.0 || f.h < 1.0 || self.world.lock_target().is_some() {
            return None;
        }
        let rig = &self.cam.rig;
        let dir = crate::camera::screen_ray(rig, f.w / f.h, Vec2::ZERO);
        let w = &self.world;
        let range = match w.player.weapon.def().rb {
            crate::weapons::Rb::Magic(b) => b.range,
            crate::weapons::Rb::Shot { bolt, .. } => bolt.range,
        };
        let reach = self.t[crate::tuning::P::CastRange] * range;
        let pp = w.player.pos;
        w.enemies
            .iter()
            .filter(|en| en.active() && !en.harmless())
            .filter(|en| vec2(en.pos.x - pp.x, en.pos.z - pp.z).length() <= reach + en.radius)
            .filter_map(|en| {
                let c = en.pos + Vec3::Y * (en.height * 0.5);
                let t = (c - rig.pos).dot(dir);
                let miss = (rig.pos + dir * t - c).length();
                (t > 0.0 && miss <= en.radius + crate::tuning::BOLT_RADIUS).then_some((t, en))
            })
            // 벽 너머의 적은 잡지 않는다(0228) — 크로스헤어가 호박빛으로 "맞는다"고 말하면 탄이 거기 닿아야 한다.
            .filter(|(_, en)| w.shot_clear(en.pos, en.radius, &self.t))
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, en)| en.pos)
    }

    fn on_weapon_swap(&mut self, to: crate::weapons::Id) {
        self.weapon = to;
        // 개발 놀이터(아레나): 든 무기는 개발 창이 정한 것(가진 것이 아닐 수 있다) — 기록의 무기·칸·가방을 건드리지 않는다.
        #[cfg(feature = "dev")]
        if self.stack.base() == Scene::Sandbox {
            self.snd(crate::sfx::Id::UiConfirm);
            return;
        }
        self.progress.weapon = Some(to);
        self.progress.sub = self.world.player.sub;
        // 스킬 칸도 무기를 따라 맞바뀐다(0223 V3 — 월드는 이미 바꿨다).
        std::mem::swap(&mut self.progress.skills, &mut self.progress.sub_skills);
        self.progress.bag.swap_weapons();
        // 벼린 단(0180)은 든 무기에만 — 바꿔 든 무기의 것으로 능력치를 다시 합친다(V1b에서 빠뜨렸다: 바꿔 들어도 옛 무기의 단이 남았다).
        // 탄창은 월드가 바꿔 든 그대로(가득이었나를 옛 무기의 능력치로 재면, 벼려 늘린 탄창이 바꿔 들 때마다 차오른다).
        let ammo = self.world.player.ranged.ammo;
        self.world.apply_stats(self.progress.stats(), false);
        self.world.player.ranged.ammo = ammo.min(self.world.mag_of(to));
        self.persist();
        self.snd(crate::sfx::Id::UiConfirm);
    }

    pub(super) fn step_world(&mut self, a: &Actions) {
        // 버튼 → 월드의 일(0255): 근접 공격(패드 X·왼쪽 클릭·J) = `attack`, 원거리 공격(RB·오른쪽 클릭·U) = `cast` — 무기와 상관없이.
        // 원거리 무기의 `attack`은 수류탄이다. 터치만 든 무기 버튼이 사격이라 `input`이 터치의 둘을 맞바꿔 넘긴다.
        let (cam, t) = (&mut self.cam, &self.t);
        // 입력 기준(0011 → 0181): 쿼터뷰는 그 방위, 자유 숄더뷰는 카메라가 보는 쪽.
        self.world.view_yaw = Some(cam.basis_yaw_deg(t));
        use crate::tuning::{
            KICK_HIT_DEG, KICK_SMASH_DEG, SHAKE_CHARGED, SHAKE_CRIT, SHAKE_DOWN, SHAKE_HIT,
            SHAKE_HURT, SHAKE_PARRY, SHAKE_PLUNGE, SHAKE_ROUSE, SHAKE_SLAM, SHAKE_SMASH,
            SHAKE_TURN,
        };
        #[cfg(feature = "dev")]
        {
            self.world.unhurt = crate::screen::test_flags() & 1 != 0;
        }
        let events = self.world.step(a, t);
        // 바꿔 든 무기를 기록에(0223 — 다음 판·멈춤의 장비 탭도 그것으로).
        if let Some(to) = events.iter().find_map(|e| match e {
            Event::WeaponSwap { to } => Some(*to),
            _ => None,
        }) {
            self.on_weapon_swap(to);
        }
        let (cam, t) = (&mut self.cam, &self.t);
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
                // 수레가 떠나는 덜컹(0208): 가까울 때만 작게 — 멀리 있는 수레가 화면을 흔들지 않게.
                Event::CartGo { at } => {
                    if at.distance(self.world.player.pos) < 14.0 {
                        SHAKE_SLAM * 0.5
                    } else {
                        0.0
                    }
                }
                Event::CartWarn { .. } => 0.0,
                Event::Rouse { .. } => SHAKE_ROUSE,
                // 사슬이 끊긴다(0208 — 3단계): 2단계 전환보다 크게.
                Event::Unchain { .. } => SHAKE_ROUSE * 1.4,
                Event::Parry { .. } => SHAKE_PARRY,
                Event::Deflect { .. } => SHAKE_HIT,
                Event::Crit { .. } => SHAKE_CRIT,
                Event::Plunge { .. } => SHAKE_PLUNGE,
                Event::Smash { .. } => SHAKE_SMASH,
                Event::Charged => SHAKE_CHARGED,
                Event::Gulp { .. }
                | Event::Mend { .. }
                | Event::FlaskDenied
                | Event::Ember { .. } => 0.0,
                Event::Cast { .. } | Event::CastDenied | Event::Throw => 0.0,
                Event::WeaponSwap { .. } => 0.0,
                Event::Guard { .. } => SHAKE_HIT,
                Event::GuardBreak { .. } => SHAKE_HURT,
                Event::StaminaDenied | Event::Release { .. } => 0.0,
                Event::Break { .. } | Event::LoadBreak { .. } => SHAKE_SMASH,
                Event::Aftershock { .. } | Event::Boom { .. } => SHAKE_PLUNGE,
                Event::Perfect { .. } => SHAKE_CHARGED,
                // 락온 대상을 잃으면(0181) 풀린 뒤 자유 숄더뷰에 머문다 — 버튼으로 풀면 락온한 곳(쿼터뷰·자유)으로.
                Event::LockLost => {
                    cam.lose_lock();
                    0.0
                }
                Event::Swing | Event::LockOn | Event::LockOff | Event::Beat => 0.0,
                // 스킬(0151): 판정이 나간 순간에 한 번, 큰 기술은 깨어나는 보스만큼.
                // 되울림·잔불 흩기의 시동(몸에 두르는 것·흩는 것)은 흔들지 않는다 — 울림·불티의 타가 제 몫을 낸다.
                Event::Skill {
                    id: crate::skills::Id::Echo | crate::skills::Id::Embers,
                    ..
                } => 0.0,
                Event::Skill { .. } | Event::SkillBurst { .. } => SHAKE_PLUNGE,
                Event::Ult { .. } | Event::LineCut { .. } => SHAKE_ROUSE,
                // 0158 G2: 터지는 것은 내려찍기만큼, 받아냄·울림·되돎은 가볍게, 불티의 타·손에 돌아옴·몰아치기의 시작은 조용히.
                Event::ThrowBoom { .. } => SHAKE_PLUNGE,
                Event::Soak { .. } | Event::EchoRing { .. } | Event::Interrupt { .. } => SHAKE_HIT,
                Event::RootSpike { .. } => 0.0,
                // 피해 타입(0223 V2): 울림·터짐은 둘레를 치는 순간이라 가볍게, 화상이 타는 것·게이지가 차는 것은 조용히.
                Event::Toll { .. } => SHAKE_HIT,
                Event::AilBurst { .. } => SHAKE_PLUNGE,
                Event::Ail { .. } | Event::Burn { .. } => 0.0,
                Event::ThrowTurn { .. }
                | Event::ThrowBack { .. }
                | Event::FieldPulse { .. }
                | Event::Flurry { .. } => 0.0,
                Event::SkillStart { .. }
                | Event::SkillDenied
                | Event::UltStart
                | Event::UltReady => 0.0,
            };
            // 화면 흔들림 설정(0052): 0..=10 → 0..100%.
            cam.trauma = (cam.trauma + kick * self.opts.shake as f32 / 10.0).min(1.0);
            // 카메라 킥(2026-11-06, KB melee-light…: 작은 타격은 trauma²에서 0이 된다 — 맞은 쪽으로 2~3px 찼다 돌아온다).
            let punch = match e {
                Event::Hit { at } | Event::Crit { at } => Some((at, KICK_HIT_DEG)),
                Event::Smash { at } | Event::Break { at } => Some((at, KICK_SMASH_DEG)),
                // 맞음(0143): 밀려난 쪽으로 — 흔들림만으로는 어디서 맞았는지가 없다.
                Event::PlayerHurt => Some((
                    self.world.player.pos + self.world.player.vel,
                    crate::tuning::KICK_HURT_DEG,
                )),
                _ => None,
            };
            if let Some((at, deg)) = punch {
                let d = at - self.world.player.pos;
                cam.kick(vec3(d.x, 0.0, d.z), deg * self.opts.shake as f32 / 10.0);
            }
            // 타격 줌(0145): 센 타격의 순간에만 — 흔들림 설정과 랩 `feel.zoom`을 따른다.
            use crate::tuning::{ZOOM_CRIT, ZOOM_PARRY, ZOOM_PERFECT, ZOOM_SMASH};
            let zoom = match e {
                Event::Smash { .. } | Event::Break { .. } => ZOOM_SMASH,
                Event::Crit { .. } => ZOOM_CRIT,
                Event::Parry { .. } => ZOOM_PARRY,
                Event::Perfect { .. } => ZOOM_PERFECT,
                // 큰 기술(0151): 시동의 세계 멈춤 동안 당겨 두고(멈춤 동안 머문다), 뒤집히는 순간 한 번 더.
                Event::UltStart | Event::Ult { .. } | Event::LineCut { .. } => ZOOM_SMASH,
                // 0158 G2의 큰 순간: 땅울림이 찍는 순간, 셋을 다 받아낸 버팀의 터뜨림(받아낸 수는 다음 버팀까지 남는다).
                Event::Skill {
                    id: crate::skills::Id::Quake,
                    ..
                } => ZOOM_CRIT,
                Event::Skill {
                    id: crate::skills::Id::Brace,
                    ..
                } if self.world.player.skill.brace_soaked >= self.world.player.skill.brace_most
                    && self.world.player.skill.brace_most > 0 =>
                {
                    ZOOM_SMASH
                }
                _ => 0.0,
            };
            if zoom > 0.0 {
                cam.punch_in(zoom * t[crate::tuning::P::FeelZoom] * self.opts.shake as f32 / 10.0);
            }
        }
        self.world_sounds(&events);
        self.tutor_watch(&events);
        self.knell_watch();
        // 그림 이펙트(0141): 사건·상태 변화 → 입자. 히트스톱 동안은 입자도 느리게(맞힌 순간의 번쩍임·불티가 멈춘 몸과 함께 남는다).
        self.fx.on_events(&events, &self.world);
        self.fx.watch(&self.world, TICK_DT);
        let slow = if self.world.hitstop > 0.0 { 0.25 } else { 1.0 };
        self.fx.tick(TICK_DT * slow);
        let w = &self.world;
        self.fx.settle(|p| w.ground_y(p));
        self.follow_camera();
    }

    pub(super) fn follow_camera(&mut self) {
        let target = self.world.lock_target();
        self.cam.freeze = self.world.hitstop > 0.0;
        // 락온 = 숄더뷰. 풀리면 락온한 곳으로(잃었으면 자유 숄더뷰 — `lose_lock`), 아니면 지금 모드(쿼터뷰 · 자유) 그대로(0181).
        let mode = match (target, self.cam.mode) {
            (Some(_), _) => Mode::Lock,
            (None, Mode::Lock) => self.cam.unlock_mode(),
            (None, m) => m,
        };
        self.cam.set_mode(mode, target, self.world.player.pos);
        self.cam.update(
            TICK_DT,
            &self.t,
            self.world.player.pos,
            target,
            &self.world.obstacles,
        );
    }

    /// 튜토리얼(0167)이 이 판에서 보나: 슬롯의 판이고(개발 창의 판·놀이터가 아니다) 깨어나는 첫 장면(0171)이 끝났다.
    /// 걸음을 다 지났어도 본 것은 쌓인다 — 상황 힌트(0209 T2)가 읽는다.
    fn tutor_live(&self) -> bool {
        !self.dev_run_now() && self.stack.base() == Scene::Play && !self.cutscene()
    }

    /// 튜토리얼의 걸음이 도나: 위 + 걸음이 남았다.
    fn tutor_on(&self) -> bool {
        self.tutor_live() && self.tutor.step().is_some()
    }

    /// 튜토리얼이 읽는 지금의 사실 — 잔불·다음 레벨 값·총 레벨·기술 판.
    fn tutor_facts(&self) -> crate::tutorial::Facts {
        let (level, board) = (self.progress.level, self.progress.board);
        crate::tutorial::Facts {
            embers: self.world.embers,
            cost: crate::progression::LINES
                .iter()
                .find_map(|&l| level.next_cost(l)),
            level: level.total(),
            spent: board.spent() > 0,
            // 쓸 수 있는 스킬(0225): 배웠어도 지금 쥔 꼴과 안 맞으면 누를 것이 없다 — "쓰기" 걸음이 막히지 않게.
            knows_skill: board.knows_a_skill()
                && self.world.player.skill.rank.iter().any(|r| *r > 0),
            can_learn: board.can_buy(self.progress.skill_points()),
            gift: self.progress.gift,
        }
    }

    /// 한 틱의 월드를 튜토리얼이 본다(0167): 걸은 거리·맞힌 타·회피·패링·쓴 스킬을 쌓고 끝난 걸음을 넘긴다. 걸음이
    /// 바뀌면 기록에 적는다(이어하면 그 걸음부터). 월드는 튜토리얼을 모른다 — 소리·이펙트와 같은 길.
    fn tutor_watch(&mut self, events: &[Event]) {
        // 길 안내(0170)는 틱마다 다시 정한다 — 튜토리얼이 꺼졌으면 걷는다.
        self.run.guide = None;
        if !self.tutor_live() {
            self.tip = None;
            return;
        }
        // 다 지난 판도 맞힌 타·모으기는 센다(상황 힌트 0209 T2).
        for e in events {
            match e {
                Event::Hit { .. } | Event::Crit { .. } => self.tutor.seen.hits += 1,
                Event::Charged => self.tutor.seen.charged += 1,
                // 0223의 새 것들(상황 힌트가 본다): 바꿔 듦 · 방패로 받음 · 타입의 일이 걸림.
                Event::WeaponSwap { .. } => self.tutor.seen.swaps += 1,
                Event::Guard { .. } => self.tutor.seen.blocks += 1,
                Event::Ail { .. } | Event::Toll { .. } => self.tutor.seen.ails += 1,
                // 등잔을 한 번 마셔 보면 그 뒤로는 알리지 않는다(기록의 비트 — 판을 넘어, 0213).
                Event::Gulp { .. } => {
                    self.tutor.seen.gulps += 1;
                    let bit = crate::tutorial::Tip::Flask.bit();
                    if self.progress.tips & bit == 0 {
                        self.progress.tips |= bit;
                        self.persist();
                    }
                }
                _ => {}
            }
        }
        self.tip_watch();
        if !self.tutor_on() {
            return;
        }
        let pl = &self.world.player;
        let step = pl.pos - pl.prev;
        let dodging = matches!(pl.state, crate::world::PState::Dodge { .. });
        // 패링 걸음의 시계는 깨어 있는 적이 곁에 있을 때만 간다(적이 없는데 "못 했다"고 넘기지 않는다).
        let fight = self
            .world
            .enemies
            .iter()
            .any(|e| e.active() && (e.pos - pl.pos).length() < crate::tutorial::FIGHT_RANGE);
        // 곁에서 시작한 적의 공격 예고(0209 T3 — 공격 번호로 한 번씩, 소리와 같은 셈).
        self.tell_seen.resize(self.world.enemies.len(), u32::MAX);
        let mut tells = 0;
        for (i, en) in self.world.enemies.iter().enumerate() {
            if en.telegraph().is_some() && self.tell_seen[i] != en.atk_count {
                self.tell_seen[i] = en.atk_count;
                if (en.pos - pl.pos).length() < crate::tutorial::FIGHT_RANGE {
                    tells += 1;
                }
            }
        }
        let seen = &mut self.tutor.seen;
        seen.tells += tells;
        seen.moved += vec2(step.x, step.z).length();
        if dodging && !self.tutor.dodging {
            seen.dodges += 1;
        }
        for e in events {
            match e {
                Event::Parry { .. } => seen.parries += 1,
                Event::SkillStart { .. } => seen.skills += 1,
                _ => {}
            }
        }
        self.tutor.dodging = dodging;
        let facts = self.tutor_facts();
        if self.tutor.tick(&facts, fight, TICK_DT) {
            self.progress.tutorial = self.tutor.at;
            self.persist();
            // 걸음이 넘어갔다(0171) — 빛만 바뀌던 것에 소리 하나. 상이 오는 틱은 상의 소리가 대신한다.
            if !self.tutor.owes_reward(&facts) {
                self.snd(crate::sfx::Id::Recall);
            }
        }
        // 길 안내(0170 — KB wayfinding: 갈 곳이 화면에 말한다): 걸음이 쉼터에서 할 일이면 가장 가까운 쉼터에 갈 곳의
        // 빛기둥·화살을 얹는다. 싸우는 동안은 비운다(화살이 싸움을 가리지 않게).
        let to_hearth = matches!(
            self.tutor.step(),
            Some(crate::tutorial::Step::Learn | crate::tutorial::Step::Level)
        ) && self
            .tutor
            .step()
            .is_some_and(|s| s.shown(&facts) && !s.gathering(&facts));
        if to_hearth && !fight {
            let at = self.world.player.pos;
            self.run.guide = self
                .run
                .rests
                .iter()
                .copied()
                .min_by(|a, b| a.distance(at).total_cmp(&b.distance(at)));
        }
        // 상(0169): 몸의 걸음 넷을 마쳤다 — 기술 점수 하나. 기록의 몫에 적어 한 번만(이어한 판도 지금의 상태로 본다).
        // 잔불·레벨은 건드리지 않는다(0109·0123의 "한 판 ≈ 한 레벨"은 그대로 — 첫 스킬만 그 뒤로 밀리지 않게).
        if self.tutor.owes_reward(&facts) {
            self.progress.gift = crate::tutorial::REWARD_POINTS;
            self.toast = Some((crate::i18n::s().tut_reward.to_string(), TOAST_TIME));
            self.snd(crate::sfx::Id::Buy);
            self.persist();
        }
    }

    /// 상황 힌트(0209 T2): 떠 있으면 시간을 줄이고(할 일을 하면 먼저 걷힌다), 없으면 때가 된 첫 것을 띄운다 — 띄우는
    /// 순간 기록의 비트에 적어 판을 넘어 한 번뿐.
    fn tip_watch(&mut self) {
        use crate::tutorial::{next_tip, Situation, Tip, TIP_TIME};
        let pl = &self.world.player;
        let now = Situation {
            hp_low: pl.hp * 2 < pl.max_hp,
            has_flask: pl.flask > 0,
            big_foe: self.world.enemies.iter().any(|e| {
                e.active()
                    && (e.boss || e.elite.is_some())
                    && (e.pos - pl.pos).length() < crate::tutorial::FIGHT_RANGE
            }),
            locked: self.world.lock.is_some(),
            has_sub: pl.sub.is_some(),
            can_block: self.world.can_block(),
            unfit: (0..2).any(|i| pl.skill.rank[i] == 0 && pl.skill.known[i] > 0),
            spare: !self.progress.bag.extras().is_empty(),
            bag_blocked: self.run.near_blocked,
        };
        // 등잔은 체력이 반 위로 돌아오면 다시 알릴 수 있다(마셔 보기 전까지 — 기록의 비트는 마실 때 선다, 0213).
        let flask = Tip::Flask.bit();
        if !now.hp_low {
            self.tips_now &= !flask;
        }
        // 판이 가려진 동안(시작 이야기·구역 카드·보스 띠 — `hud::draw`가 판을 그리지 않는다)에는 힌트의 시계를 세우고 새로
        // 띄우지도 않는다: 안 보이는 사이에 한 번뿐인 힌트가 지나가 버렸다(구역 카드 밑에서 "바꿔 든다"가 사라졌다).
        if self.run.intro > 0.0
            || self.run.card.is_some()
            || self.run.boss_bar(&self.world).is_some()
        {
            return;
        }
        if let Some((t, left)) = self.tip {
            // 떠 있는 동안(남은 시간 > 0) 할 일을 하면 걷히고, 걷힌 뒤에는 `TIP_REST`만큼 쉰다(남은 시간이 0 아래로 —
            // 여럿이 때를 맞아도 줄지어 덮지 않는다). 급한 것(등잔)은 쉼을 기다리지 않는다.
            let left = if left > 0.0 && t.done(&now, &self.tutor.seen) {
                0.0
            } else {
                left - TICK_DT
            };
            let urgent = left <= 0.0
                && (self.progress.tips | self.tips_now) & flask == 0
                && Tip::Flask.due(self.tutor.at, &now, &self.tutor.seen);
            self.tip = (left > -crate::tutorial::TIP_REST && !urgent).then_some((t, left));
            if !urgent {
                return;
            }
        }
        let shown = self.progress.tips | self.tips_now;
        if let Some(t) = next_tip(shown, self.tutor.at, &now, &self.tutor.seen) {
            self.tip = Some((t, TIP_TIME));
            self.tips_now |= t.bit();
            if t != Tip::Flask {
                self.progress.tips |= t.bit();
                self.persist();
            }
            self.snd(crate::sfx::Id::Recall);
        }
    }

    /// 튜토리얼이 큰 창 안에서 시키는 일의 탭(0209 — 창을 열어 하는 걸음에 반짝임): 배우기 = 기술 탭, 올리기 = 상태 탭
    /// (잔불이 모였을 때만). 그 탭의 머리와 할 수 있는 칸·줄이 반짝인다.
    pub(super) fn tutor_menu(&self) -> Option<crate::app::flow::Tab> {
        use crate::app::flow::Tab;
        use crate::tutorial::Step;
        if !self.tutor_on() {
            return None;
        }
        let f = self.tutor_facts();
        match self.tutor.step()? {
            Step::Learn if f.can_learn => Some(Tab::Skills),
            Step::Level if Step::Level.shown(&f) && !Step::Level.gathering(&f) => Some(Tab::Status),
            _ => None,
        }
    }

    /// 지금 보일 튜토리얼의 힌트 줄(없으면 그리지 않는다). 스킬 걸음의 버튼은 배운 스킬이 놓인 칸의 것. 상황 힌트가
    /// 떠 있으면 그것이 먼저다(몇 초뿐).
    pub(super) fn tutor_hint(&self) -> Option<crate::tutorial::Hint> {
        // 버튼 이름은 키캡으로(사장 2026-11-13 "버튼처럼") — 터치는 이름이 없다(화면 버튼이 곧 안내).
        let cap = |b| {
            let name = crate::input::bind_prompt(b);
            if name.is_empty() {
                name
            } else {
                crate::ui::key(&name)
            }
        };
        if let (Some((t, left)), true) = (self.tip, self.tutor_live()) {
            // 걷힌 뒤의 쉼(남은 시간 0 아래)에는 그리지 않는다 — 걸음의 힌트가 그 자리에 돌아온다.
            if left > 0.0 {
                // 쇠뇌는 모으기가 RB(사격)에 있다(0213) — 모으기 힌트만: 바꿔 들기의 평타는 손의 버튼 그대로다.
                let shots = t == crate::tutorial::Tip::Charge
                    && matches!(
                        self.world.player.weapon.def().rb,
                        crate::weapons::Rb::Shot { .. }
                    );
                return Some(t.hint(crate::i18n::s(), |b| {
                    cap(match b {
                        crate::input::Bind::Attack if shots => crate::input::Bind::Cast,
                        b => b,
                    })
                }));
            }
        }
        if !self.tutor_on() {
            return None;
        }
        let board = self.progress.board;
        let slot = self
            .progress
            .skills
            .0
            .iter()
            .position(|&id| board.learned(id))
            .unwrap_or(0);
        self.tutor
            .hint(crate::i18n::s(), &self.tutor_facts(), slot, cap)
    }

    pub(super) fn in_stage(&self) -> bool {
        match self.scene() {
            #[cfg(feature = "dev")]
            Scene::Sandbox => false,
            _ => true,
        }
    }

    /// 튜토리얼이 벨 상대를 찾게 하는 걸음(공격·패링)에서 가장 가까운 적의 자리(0170) — 곁에 깨어 있는 적이 없을 때만
    /// (싸우는 동안은 화살이 필요 없다). 갈 곳(`StageRun::goal`)이 따로 있으면 그것이 먼저다.
    fn tutor_foe(&self) -> Option<Vec3> {
        use crate::tutorial::{Step, FIGHT_RANGE};
        if !self.tutor_on() || !matches!(self.tutor.step(), Some(Step::Attack | Step::Parry)) {
            return None;
        }
        let at = self.world.player.pos;
        let near = |e: &&crate::enemy::Enemy| e.pos.distance(at) < FIGHT_RANGE;
        if self.world.enemies.iter().filter(near).any(|e| e.active()) {
            return None;
        }
        self.world
            .enemies
            .iter()
            .filter(|e| e.alive())
            .map(|e| e.pos)
            .min_by(|a, b| a.distance(at).total_cmp(&b.distance(at)))
    }

    pub(super) fn draw_goal_arrow(&mut self, f: &Frame) {
        // 갈 곳(호박색)이 먼저, 없으면 튜토리얼의 벨 상대(잿빛 — 갈 길의 색을 쓰지 않는다).
        let (target, ink) = match (self.run.goal(), self.tutor_foe()) {
            (Some(g), _) => (Some(g), AMBER),
            (None, foe) => (foe, hud::FOE_INK),
        };
        let mark = target.and_then(|g| {
            let (ndc, front) =
                crate::camera::world_to_ndc(&self.cam.view, f.w / f.h.max(1.0), g + Vec3::Y);
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
            hud::draw_edge_arrow(f, self.above_hint(f, p), -ang, self.world.time, ink);
        }
    }

    /// 화면 가장자리의 표지(화살·물결)가 튜토리얼의 힌트 판에 걸리면 판 바로 위로 올린다(0171 — 판과 화살이 서로를 가렸다).
    fn above_hint(&self, f: &Frame, p: Vec2) -> Vec2 {
        let Some(h) = self.tutor_hint() else { return p };
        let r = hud::hint_rect(&self.ui, f, &h);
        let m = 10.0 * f.ui;
        if p.x > r.x - m && p.x < r.x + r.w + m && p.y > r.y - m {
            vec2(p.x, r.y - m)
        } else {
            p
        }
    }

    /// 종소리의 길 안내(0171): 닫힌 문의 장치를 찾는 동안 — 다른 길 안내(갈 곳의 빛기둥·튜토리얼의 쉼터)가 없고 장치가
    /// 멀 때만 — 가끔 종이 운다. 싸우는 동안은 울리지 않는다(`knell::Knell`). 월드는 모른다(소리·이펙트와 같은 길).
    fn knell_watch(&mut self) {
        let at = self.world.player.pos;
        let lost = self.stack.base() == Scene::Play
            && !self.cutscene()
            && self.run.goal().is_none()
            && self
                .run
                .bell_from()
                .is_some_and(|b| b.distance(at) > crate::knell::NEAR);
        let fight = self
            .world
            .enemies
            .iter()
            .any(|e| e.active() && e.pos.distance(at) < crate::tutorial::FIGHT_RANGE);
        if self.knell.tick(TICK_DT, lost, fight) {
            self.snd(crate::sfx::Id::Knell);
        }
    }

    /// 종이 운 뒤 잠깐, 소리가 온 쪽의 화면 가장자리에 물결(화면 안이면 그 자리에 고리). 소리는 좌우로 나뉘지 않으니
    /// (모노) 방향은 그림이 말한다.
    pub(super) fn draw_knell(&self, f: &Frame) {
        let (Some(k), Some(from)) = (self.knell.wave(), self.run.bell_from()) else {
            return;
        };
        let (ndc, front) =
            crate::camera::world_to_ndc(&self.cam.view, f.w / f.h.max(1.0), from + Vec3::Y);
        let px = |n: Vec2| {
            vec2(
                f.ox + (n.x + 1.0) * 0.5 * f.w,
                f.oy + (1.0 - n.y) * 0.5 * f.h,
            )
        };
        match crate::camera::edge_marker(ndc, front, 0.0) {
            Some((at, ang)) => hud::draw_knell(f, self.above_hint(f, px(at)), Some(-ang), k),
            None => hud::draw_knell(f, px(ndc), None, k),
        }
    }

    /// 주민 말풍선(0122, nail 말 걸기 말풍선 꼴): 그 주민 머리 위, 판 + 테두리 + 짧은 말. 처음·끝에 옅게.
    pub(super) fn draw_bubble(&mut self, f: &Frame) {
        let Some((k, n, t)) = self.bubble else { return };
        let Some(&(i, at)) = self.run.folk.list.get(k) else {
            return;
        };
        let r = &crate::residents::RESIDENTS[i];
        let changed = self.run.folk.changed.get(k).copied().unwrap_or(false);
        let text = tr(crate::residents::line(r, changed, n));
        let (ndc, front) =
            crate::camera::world_to_ndc(&self.cam.view, f.w / f.h.max(1.0), at + Vec3::Y * 1.75);
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

    /// 새긴 돌 글 판(0129 C4): 화면 위쪽 가운데, 돌빛 판 + 글(줄 바꿈), 들어오고 나갈 때 옅게. 멈추지 않는다.
    pub(super) fn draw_stone_card(&mut self, f: &Frame) {
        let Some((k, t)) = self.stone_card else {
            return;
        };
        let Some(&text) = self.run.def.stones.get(k) else {
            return;
        };
        let text = tr(text);
        let a = (t / 0.3).min((STONE_TIME - t) / 0.2).clamp(0.0, 1.0);
        let kk = 1.0;
        let max_w = (f.w * 0.6).min(f.ui * 320.0);
        // 새긴 줄(`\n`)마다 끊고, 그 안에서만 폭에 맞춰 나눈다 — 문장이 어색하게 갈리지 않게.
        let lines: Vec<String> = text
            .split('\n')
            .flat_map(|part| ui::wrap(&self.ui, part, max_w, kk))
            .collect();
        let gap = self.ui.px(kk) * 1.45;
        let w = lines
            .iter()
            .map(|l| self.ui.width(l, kk))
            .fold(0.0, f32::max);
        let pad = 8.0 * f.ui;
        let h = gap * lines.len() as f32;
        let cx = f.ox + f.w * 0.5;
        let rect = Rect::new(
            cx - w * 0.5 - pad,
            // 화면 위쪽(목표 글 아래) — 가운데의 방랑자 몸을 가리지 않게.
            f.oy + f.h * 0.14,
            w + pad * 2.0,
            h + pad * 1.6,
        );
        draw_rectangle(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            Color::new(0.16, 0.15, 0.17, 0.88 * a),
        );
        draw_rectangle_lines(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            1.5 * f.ui,
            Color::new(0.62, 0.6, 0.58, 0.9 * a),
        );
        for (i, line) in lines.iter().enumerate() {
            self.ui.text_center(
                line,
                cx,
                rect.y + pad + gap * i as f32,
                kk,
                Color::new(0.88, 0.86, 0.82, a),
            );
        }
    }

    /// 플레이어 머리 위의 화면 자리(카메라 뒤면 없음).
    fn head_on_screen(&self, f: &Frame) -> Option<Vec2> {
        let pl = &self.world.player;
        let head = pl.pos + Vec3::Y * (pl.air + 1.9);
        let (ndc, front) = crate::camera::world_to_ndc(&self.cam.view, f.w / f.h.max(1.0), head);
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
        if pl.beat.since < hud::BEAT_NOTE_LIFE {
            hud::draw_beat_note(f, p, pl.beat.since / hud::BEAT_NOTE_LIFE, pl.beat.chain);
        }
        if pl.stamina_denied > 0.0 {
            let k = 1.0 - pl.stamina_denied / crate::tuning::STAMINA_DENIED_SHOW;
            hud::draw_sweat(f, p, k.clamp(0.0, 1.0));
        }
    }

    /// 월드 사건·상태 변화 → 소리(0064 M5). 월드는 소리를 모른다 — 여기서만 듣는다.
    pub(super) fn world_sounds(&mut self, events: &[Event]) {
        use crate::sfx::Id;
        let mut typed: Vec<Id> = Vec::new();
        for e in events {
            let id = match e {
                Event::Swing => Id::Swing,
                Event::Hit { .. } | Event::Crit { .. } => Id::Hit,
                Event::Plunge { .. } | Event::Smash { .. } | Event::Aftershock { .. } => Id::Hit,
                Event::Break { .. } | Event::Boom { .. } | Event::LoadBreak { .. } => Id::Hit,
                Event::EnemyDown { .. } => Id::Die,
                Event::PlayerHurt => Id::Hurt,
                // 등딱지 튕김(0120)은 패링의 거울 — 같은 쇳소리.
                Event::Parry { .. } | Event::Deflect { .. } => Id::Parry,
                // 방패(0223): 받은 타는 둔한 '둥'(튕김의 맑은 소리와 갈린다), 깨지면 맞은 소리.
                Event::Guard { .. } => Id::Thwack,
                Event::GuardBreak { .. } => Id::Hurt,
                Event::Beat => crate::sfx::beat_id(self.world.player.beat.chain),
                // 딱 맞춰 떼기(0078): 박자와 같은 "맞췄다" 종 — 가장 높은 음.
                Event::Perfect { .. } => Id::Beat3,
                Event::Ember { .. } => Id::Ember,
                Event::Gulp { .. } | Event::Mend { .. } => Id::Gulp,
                // 지연 공격이 풀린다(0128): 짧게 들이쉬는 소리 — 이 뒤 `RELEASE_LEAD`초에 맞는다.
                Event::Release { .. } => Id::TeleRelease,
                // 수레가 떠난다(0208): 둔한 덜컹 — 가까울 때만(멀리서 내내 울리지 않게).
                Event::CartGo { at } if at.distance(self.world.player.pos) < 18.0 => Id::Thwack,
                Event::CartWarn { at } if at.distance(self.world.player.pos) < 18.0 => Id::TeleLane,
                // 사슬이 끊긴다(0208 3단계): 깊은 울림 + 글 한 줄.
                Event::Unchain { .. } => {
                    self.run.card = Some((|s| s.overseer_unchained, crate::stage::CARD_TIME));
                    Id::Wake
                }
                Event::FlaskDenied | Event::CastDenied | Event::StaminaDenied => Id::Deny,
                // 스킬(0151 — S0는 있는 소리를 겹쳐 쓴다, 전용 소리는 S1): 시동 = 휘두름, 큰 기술 시동 = 깊은 울림, 터짐 = 타격,
                // 게이지 가득 = 가장 높은 종.
                // 0158 G2: 스킬마다 다른 짝(있는 소리를 겹친다). 시동 — 버팀은 발을 딛는 둔한 소리, 되울림은 낮은 종,
                // 잔불 흩기는 잔불 소리, 나머지는 휘두름. 판정 — 휘돌이 = 휘두름(바퀴마다), 가로지르기·던지기 = 스치는 바람,
                // 끌어모으기·버팀의 터뜨림 = 둔한 소리, 땅울림 = 낮은 울림, 되울림 = 가운데 종, 잔불 흩기 = 잔불 소리.
                Event::SkillStart { id } => {
                    use crate::skills::Id as Sk;
                    match id {
                        Sk::Brace => Id::Thwack,
                        Sk::Echo => Id::Beat,
                        Sk::Embers => Id::Ember,
                        Sk::Bash => Id::Thwack,
                        Sk::Whirl
                        | Sk::Cut
                        | Sk::Gather
                        | Sk::Quake
                        | Sk::Throw
                        | Sk::Swallow
                        | Sk::Roots => Id::Swing,
                    }
                }
                Event::Skill { id, .. } => {
                    use crate::skills::Id as Sk;
                    match id {
                        Sk::Whirl => Id::Swing,
                        Sk::Cut | Sk::Throw => Id::Dodge,
                        Sk::Gather | Sk::Brace => Id::Thwack,
                        Sk::Quake => Id::Wake,
                        Sk::Echo => Id::Beat2,
                        Sk::Embers => Id::Ember,
                        Sk::Swallow => Id::Dodge,
                        Sk::Bash => Id::Parry,
                        Sk::Roots => Id::Wake,
                    }
                }
                Event::SkillDenied => Id::Deny,
                Event::UltStart => Id::Wake,
                Event::Ult { .. } | Event::SkillBurst { .. } => Id::Hit,
                Event::UltReady => Id::Beat3,
                // 0158 G2(새 소리 파일 없이 있는 것을 겹친다 — 스킬마다 다른 짝): 받아냄 = 쇳소리, 울림 = 가운데 종,
                // 되돎 = 휘두름, 터짐·한 줄기 = 타격, 손에 돌아옴 = 낮은 종, 불티의 타 = 잔불 소리, 몰아치기 = 가장 높은 종.
                Event::Soak { .. } | Event::Interrupt { .. } => Id::Parry,
                Event::EchoRing { .. } => Id::Beat2,
                Event::ThrowTurn { .. } => Id::Swing,
                Event::ThrowBoom { .. } | Event::LineCut { .. } => Id::Hit,
                Event::ThrowBack { .. } => Id::Beat,
                Event::FieldPulse { .. } => Id::Ember,
                // 피해 타입(0223 V2): 화상이 붙음·탐 = 잔불 소리, 묶임 = 뿌리가 깨는 소리, 금 = 쇳소리, 울림 = 낮은 종, 터짐 = 타격.
                Event::Ail { kind, .. } => match kind {
                    crate::status::Kind::Ember => Id::Ember,
                    crate::status::Kind::Root => Id::Wake,
                    crate::status::Kind::Rift => Id::Parry,
                    _ => continue,
                },
                Event::Toll { .. } => Id::Beat,
                Event::AilBurst { .. } => Id::Hit,
                Event::Flurry { .. } => Id::Beat3,
                _ => continue,
            };
            // 타입의 일(0223 V2)은 떼에서 한 틱에 여럿이 겹친다(스물이 한꺼번에 울린다) — 같은 소리는 틱에 한 번만.
            if matches!(
                e,
                Event::Ail { .. } | Event::Toll { .. } | Event::AilBurst { .. }
            ) {
                if typed.contains(&id) {
                    continue;
                }
                typed.push(id);
            }
            self.snd(id);
            if id == Id::Hit {
                self.snd(Id::Thwack);
            }
        }
        // 회피를 시작한 틱.
        let dodging = matches!(self.world.player.state, crate::world::PState::Dodge { .. });
        if dodging && !self.heard.was_dodging {
            self.snd(Id::Dodge);
        }
        self.heard.was_dodging = dodging;
        // 적의 예고가 시작한 틱: 모양마다 다른 소리(소리만 듣고도 가르게, 0064 R4).
        let n = self.world.enemies.len();
        self.heard.tele_was.resize(n, u32::MAX);
        let mut cues = Vec::new();
        for (i, en) in self.world.enemies.iter().enumerate() {
            let tele = en.telegraph();
            // 껍질 받기(0185)는 예고 모양이 없다 — 웅크리는 첫 틱에 돌 소리 하나.
            if en.shelled() && self.heard.tele_was[i] != en.atk_count {
                self.heard.tele_was[i] = en.atk_count;
                cues.push(Id::Shell);
            }
            // 공격 번호로 센다(0128): 연속 공격의 다음 타·페인트로 바뀐 공격도 제 소리를 낸다.
            let fresh = tele.is_some() && self.heard.tele_was[i] != en.atk_count;
            if let (Some((_, shape)), true) = (tele, fresh) {
                cues.push(match shape {
                    crate::enemy::TeleShape::Disc { .. } => Id::TeleDisk,
                    crate::enemy::TeleShape::Sector { .. } => Id::TeleFan,
                    crate::enemy::TeleShape::Lane { .. } => Id::TeleLane,
                    // 줄지은 원(0133 뿌리 기둥)도 피할 것 — 고리와 같은 소리.
                    crate::enemy::TeleShape::Ring { .. } | crate::enemy::TeleShape::Row { .. } => {
                        Id::TeleRing
                    }
                });
            }
            if tele.is_some() {
                self.heard.tele_was[i] = en.atk_count;
            }
        }
        for id in cues {
            self.snd(id);
        }
    }

    pub(super) fn dev_sandbox(&self) -> bool {
        #[cfg(feature = "dev")]
        {
            self.scene() == Scene::Sandbox
        }
        #[cfg(not(feature = "dev"))]
        {
            false
        }
    }

    /// 개발 줄(기본 글꼴, 영어 — 개발 전용 화면은 예외, 첫 게임 0013): 측정·시나리오.
    pub(super) fn draw_dev(&self, f: &Frame) {
        let p = &self.perf;
        let mode = match self.cam.mode {
            Mode::Quarter => "QUARTER",
            Mode::Lock => "LOCK-ON",
            Mode::Free => "FREE",
        };
        let y = f.oy + f.h - 40.0;
        #[cfg(feature = "dev")]
        if self.scene() == Scene::Sandbox {
            let e = scenario::entry(self.dev.sc);
            draw_text(
                format!(
                    "{} [{}]  checks: {}  (1-9 scenario, ESC title)",
                    e.name, mode, e.checks
                ),
                f.ox + 10.0,
                y - (9.0 * f.ui).max(16.0) * 2.0 - 16.0,
                16.0,
                GRAY,
            );
        }
        // 기기에서 읽히게(사장 2026-11-20 "성능줄은 작아서 잘 안 보임"): 크기는 UI 배율을 따르고(폰의 물리 화소에서 16px는 깨알),
        // 중요한 수(fps·frame·cpu)는 윗줄에 따로, 어두운 판 위에.
        let size = (9.0 * f.ui).max(16.0);
        let lines = [
            format!(
                "fps {}  frame {:.1}ms  cpu {:.1}ms (max {:.1})",
                get_fps(),
                p.frame_avg_ms,
                p.cpu_avg_ms,
                p.cpu_max_ms,
            ),
            format!(
                "{} {mode}{} cam {:.0}deg  batches {} (max {})  verts {} (+{} resident)",
                self.world.name,
                self.cam
                    .blending()
                    .map(|e| format!(" blend {e:.2}s"))
                    .unwrap_or_default(),
                self.cam.azimuth_deg(),
                p.draws,
                p.draws_max,
                self.renderer.vertices(),
                self.renderer.resident_vertices()
            ),
        ];
        let pad = size * 0.3;
        let w = lines
            .iter()
            .map(|l| measure_text(l, None, size as u16, 1.0).width)
            .fold(0.0, f32::max);
        let top = y - size * lines.len() as f32 - pad + size * 0.25;
        draw_rectangle(
            f.ox + 10.0 - pad,
            top,
            w + pad * 2.0,
            size * lines.len() as f32 + pad * 2.0,
            Color::new(0.0, 0.0, 0.0, 0.6),
        );
        for (i, l) in lines.iter().enumerate() {
            draw_text(
                l,
                f.ox + 10.0,
                top + pad + size * (i as f32 + 0.8),
                size,
                if i == 0 { YELLOW } else { LIGHTGRAY },
            );
        }
    }
}
