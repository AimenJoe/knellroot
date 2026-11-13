//! 싸움 한 틱과 그 둘레(0019): 월드·스테이지·카메라·조준·랩, 길 안내 화살, 개발 줄, 월드 소리(0066 R3에서 옮김).
use super::*;

/// 새긴 돌 글 판이 떠 있는 초(0129).
const STONE_TIME: f32 = 6.0;

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
            self.show(flow::pause(self.scene(), self.pause_tab));
            return;
        }
        // Select(Tab) = 큰 창의 장비 탭 바로(0070 6 A → 0164, 닫으면 싸움으로).
        if a.equip {
            self.show(Go::Open(flow::SELECT_OPENS));
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
        // 새긴 돌(0129): 읽으면 글 판, 떠 있는 동안 같은 돌을 다시 누르면 닫힌다.
        if let Some(k) = self.run.take_read() {
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
            || self.progress.board.can_buy(self.progress.skill_points());
        if self.run.take_rested() {
            self.snd(crate::sfx::Id::Rest);
            let (board, total) = (self.board(), self.progress.skill_points());
            self.show(Go::Open(flow::hearth_opens(
                board.knows_a_skill(),
                board.can_buy(total),
            )));
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
            self.fx.clear();
            self.world.set_weapon(self.weapon);
            self.apply_board();
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
                Event::Soak { .. } | Event::EchoRing { .. } => SHAKE_HIT,
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
            cam.trauma = (cam.trauma + kick * self.shake as f32 / 10.0).min(1.0);
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
                cam.kick(vec3(d.x, 0.0, d.z), deg * self.shake as f32 / 10.0);
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
                cam.punch_in(zoom * t[crate::tuning::P::FeelZoom] * self.shake as f32 / 10.0);
            }
        }
        self.world_sounds(&events);
        self.tutor_watch(&events);
        // 그림 이펙트(0141): 사건·상태 변화 → 입자. 히트스톱 동안은 입자도 느리게(맞힌 순간의 번쩍임·불티가 멈춘 몸과 함께 남는다).
        self.fx.on_events(&events, &self.world);
        self.fx.watch(&self.world, TICK_DT);
        let slow = if self.world.hitstop > 0.0 { 0.25 } else { 1.0 };
        self.fx.tick(TICK_DT * slow);
        self.follow_camera();
    }

    pub(super) fn follow_camera(&mut self) {
        let target = self.world.lock_target();
        self.cam.freeze = self.world.hitstop > 0.0;
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

    /// 튜토리얼(0167)이 이 판에서 도나: 슬롯의 판이고(개발 창의 판·놀이터가 아니다) 걸음이 남았다.
    fn tutor_on(&self) -> bool {
        !self.dev_run_now() && self.stack.base() == Scene::Play && self.tutor.step().is_some()
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
            knows_skill: board.knows_a_skill(),
        }
    }

    /// 한 틱의 월드를 튜토리얼이 본다(0167): 걸은 거리·맞힌 타·회피·패링·쓴 스킬을 쌓고 끝난 걸음을 넘긴다. 걸음이
    /// 바뀌면 기록에 적는다(이어하면 그 걸음부터). 월드는 튜토리얼을 모른다 — 소리·이펙트와 같은 길.
    fn tutor_watch(&mut self, events: &[Event]) {
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
        let seen = &mut self.tutor.seen;
        seen.moved += vec2(step.x, step.z).length();
        if dodging && !self.tutor.dodging {
            seen.dodges += 1;
        }
        for e in events {
            match e {
                Event::Hit { .. } | Event::Crit { .. } => seen.hits += 1,
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
        }
    }

    /// 지금 보일 튜토리얼의 힌트 줄(없으면 그리지 않는다). 스킬 걸음의 버튼은 배운 스킬이 놓인 칸의 것.
    pub(super) fn tutor_hint(&self) -> Option<(String, &'static str)> {
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
        // 버튼 이름은 키캡으로(사장 2026-11-13 "버튼처럼") — 터치는 이름이 없다(화면 버튼이 곧 안내).
        let cap = |b| {
            let name = crate::input::bind_prompt(b);
            if name.is_empty() {
                name
            } else {
                crate::ui::key(&name)
            }
        };
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
                Event::Beat => crate::sfx::beat_id(self.world.player.beat.chain),
                // 딱 맞춰 떼기(0078): 박자와 같은 "맞췄다" 종 — 가장 높은 음.
                Event::Perfect { .. } => Id::Beat3,
                Event::Ember { .. } => Id::Ember,
                Event::Gulp { .. } => Id::Gulp,
                // 지연 공격이 풀린다(0128): 짧게 들이쉬는 소리 — 이 뒤 `RELEASE_LEAD`초에 맞는다.
                Event::Release { .. } => Id::TeleRelease,
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
                        Sk::Whirl | Sk::Cut | Sk::Gather | Sk::Quake | Sk::Throw => Id::Swing,
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
                    }
                }
                Event::SkillDenied => Id::Deny,
                Event::UltStart => Id::Wake,
                Event::Ult { .. } | Event::SkillBurst { .. } => Id::Hit,
                Event::UltReady => Id::Beat3,
                // 0158 G2(새 소리 파일 없이 있는 것을 겹친다 — 스킬마다 다른 짝): 받아냄 = 쇳소리, 울림 = 가운데 종,
                // 되돎 = 휘두름, 터짐·한 줄기 = 타격, 손에 돌아옴 = 낮은 종, 불티의 타 = 잔불 소리, 몰아치기 = 가장 높은 종.
                Event::Soak { .. } => Id::Parry,
                Event::EchoRing { .. } => Id::Beat2,
                Event::ThrowTurn { .. } => Id::Swing,
                Event::ThrowBoom { .. } | Event::LineCut { .. } => Id::Hit,
                Event::ThrowBack { .. } => Id::Beat,
                Event::FieldPulse { .. } => Id::Ember,
                Event::Flurry { .. } => Id::Beat3,
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
                    // 줄지은 원(0133 뿌리 기둥)도 피할 것 — 고리와 같은 소리.
                    crate::enemy::TeleShape::Ring { .. } | crate::enemy::TeleShape::Row { .. } => {
                        Id::TeleRing
                    }
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
