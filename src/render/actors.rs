//! 배우: 적·방랑자·던진 무기 — 상태 → 동작 이름과 시각, 준비 동작의 몸 신호, 휘청 기울기, 모델이 없을 때의 상자 배우.
use super::*;
use crate::det::Det;

impl Renderer {
    pub(super) fn enemy(
        &mut self,
        en: &Enemy,
        w: &World,
        alpha: f32,
        stage: Option<&StageRun>,
        lights: &[Light],
        fall: f32,
    ) {
        let mut p = en.prev.lerp(en.pos, alpha);
        let mut tilt = Quat::IDENTITY;
        let mut squash = 1.0;
        // 맞은 순간(2026-11-06, KB 사쿠라이): 세상이 멈춘 동안 맞은 축으로 떨고, 몸이 맞은 쪽으로 젖었다 돌아온다.
        let struck = (en.hurt / 0.15).clamp(0.0, 1.0);
        if struck > 0.0 && en.sway.length() > 1e-4 {
            if w.hitstop > 0.0 {
                let shake =
                    (get_time() as f32 * 90.0).sin_d() * crate::tuning::HIT_TREMBLE * struck;
                p += en.sway.normalize() * shake;
            }
            if !matches!(en.state, EState::Stagger { .. }) {
                let v = en.sway * 0.6 * struck;
                tilt = Quat::from_axis_angle(Vec3::Y.cross(v).normalize(), v.length());
                squash = 1.0 - 0.12 * struck;
            }
        }
        if let EState::Stagger { t: e, dur } = en.state {
            let left = ((dur - e) / 0.25).clamp(0.0, 1.0);
            let v = lean(en.sway, en.sway_t, w.time, left);
            if v.length() > 1e-4 {
                tilt = Quat::from_axis_angle(Vec3::Y.cross(v).normalize(), v.length());
            }
            // 움찔: 처음 순간 눌렸다 펴진다.
            squash = 1.0 - 0.2 * en.sway.length() * (-12.0 * en.sway_t).exp_d();
        }
        // 죽음: 뒤로 넘어지며 가라앉는다.
        if fall > 0.0 {
            let back = -en.facing_dir();
            tilt = Quat::from_axis_angle(Vec3::Y.cross(back).normalize(), fall.min(0.5) * 2.6);
        }
        let sink = Vec3::Y * (-0.6 * (fall - 0.5).max(0.0) * 2.0);
        // 끌린 자국: 미끄러지는 동안 발 뒤에 흙.
        let speed = en.knock.length();
        if speed > 1.5 && fall == 0.0 {
            let back = -en.knock / speed;
            for i in 1..=3 {
                let k = i as f32;
                draw_cube(
                    p + back * (0.22 * k) + Vec3::Y * 0.03,
                    vec3(0.22, 0.05, 0.22) * (1.0 - 0.2 * k),
                    None,
                    Color::new(0.45, 0.4, 0.33, 0.8 - 0.2 * k),
                );
            }
        }
        let flash = en.hurt > 0.0;
        let model = en.def().model;
        // 문지기가 깨어나는 동안 눈·핵이 밝아진다.
        let mut glow = match (en.state, stage) {
            // 엎드린 필드 보스(0270 D)의 눈·핵은 일어선 만큼.
            (EState::Dormant, _) if en.field => 0.15 + 0.85 * en.stand,
            (EState::Dormant, Some(run)) => 0.15 + 0.85 * run.wake_progress(),
            _ if en.roused => 1.0,
            _ => 0.85,
        };
        // 준비 동작(0073): 예고 동안 눈·핵이 차오르고, 정점에서 버티는 동안 힘을 주어 떨고, 닿는 순간 한 번 번쩍인다.
        if let EState::Tele { t } = en.state {
            let k = (t / en.tele_dur().max(1e-3)).clamp(0.0, 1.0);
            glow += WINDUP_GLOW * k;
        }
        // 들어 올리는 동안 몸이 늘어나며(부피 그대로 — KB believable-cute-character-motion) 뒤로 젖고, 내리칠 때 바로 돌아온다.
        let mut stretch = 1.0;
        // 지연 공격(0128): 들어 올림은 같은 빠르기(`lift_dur`), 정점에서 더 젖히고 더 떨며 버틴다 — 늦게 친다는 것이 몸에 보이게.
        let delayed = matches!(en.rhythm.tempo, crate::enemy::Tempo::Delayed(_));
        if let EState::Tele { t } = en.state {
            let dur = en.tele_dur().max(1e-3);
            let lift = en.lift_dur().max(1e-3);
            let snap = WINDUP_SNAP.min(dur * 0.25);
            let u = if t < dur - snap {
                let r = (t / (lift * WINDUP_REACH)).min(1.0);
                let held = if delayed {
                    1.0 + (DELAY_LEAN - 1.0) * ((t - lift * WINDUP_REACH) / 0.2).clamp(0.0, 1.0)
                } else {
                    1.0
                };
                (1.0 - (1.0 - r) * (1.0 - r)) * held
            } else {
                1.0 - (t - (dur - snap)) / snap
            };
            stretch = 1.0 + WINDUP_STRETCH * u;
            if tilt == Quat::IDENTITY {
                let back = -en.facing_dir();
                tilt = Quat::from_axis_angle(Vec3::Y.cross(back).normalize(), WINDUP_LEAN * u);
            }
        }
        if let Some(h) = windup_hold(en) {
            let tremble = WINDUP_TREMBLE * if delayed { DELAY_TREMBLE } else { 1.0 };
            let shake = (get_time() as f32 * 70.0).sin_d() * tremble;
            p += vec3(en.facing.cos_d(), 0.0, -en.facing.sin_d()) * shake;
            // 번쩍임은 눈·핵만(몸 전체가 밝아지면 맞았을 때의 흰 번쩍임과 헷갈린다).
            glow += WINDUP_GLINT_GAIN * (1.0 - h / WINDUP_GLINT).max(0.0);
        }
        // 2단계 전환(0133): 무릎 꿇은 동안 핵·눈이 차오르다 터지는 틱에 가장 밝고, 일어서며 2단계 밝기로.
        if let EState::Rise { t } = en.state {
            use crate::tuning::{RISE_BURST, RISE_TIME};
            glow += if t < RISE_BURST {
                RISE_GLOW * (t / RISE_BURST)
            } else {
                RISE_GLOW * (1.0 - (t - RISE_BURST) / (RISE_TIME - RISE_BURST))
            };
        }
        if let Some(e) = self.lib.get(model) {
            let (anim, at) = enemy_anim(en, model, e, w.time, stage);
            // 종 내던지기(0133): 종이 손을 떠난 뒤엔 손의 종을 감춘다(날아가는·떨어진 종은 따로 그린다).
            let mut hide: Vec<(usize, Quat, Vec3)> = hurl_bell(en)
                .and(e.model.node("bell"))
                .map(|n| (n, Quat::IDENTITY, Vec3::ZERO))
                .into_iter()
                .collect();
            // 등짐꾼의 짐(0216 W2 → 0234): 모델의 `load` 부위 — 자세가 처음 깨지면 깨져 사라진다.
            if en.load_lost {
                hide.extend(
                    e.model
                        .node("load")
                        .map(|n| (n, Quat::IDENTITY, Vec3::ZERO)),
                );
            }
            let p_pose = crate::prof::span("e.pose");
            e.model.pose_with(anim, at, &hide, &mut self.pose);
            drop(p_pose);
            let inst = Mat4::from_translation(p + sink)
                * Mat4::from_quat(tilt)
                * Mat4::from_rotation_y(en.facing)
                * Mat4::from_scale(
                    vec3(1.0 / stretch.sqrt(), squash * stretch, 1.0 / stretch.sqrt())
                        * if en.elite.is_some() {
                            crate::tuning::ELITE_SCALE
                        } else {
                            1.0
                        },
                );
            // 산 것(0185 — 이끼먹이)은 제 파스텔 그대로: 잿빛 보정은 침식된 적의 몫이다(0037·0170).
            let tint = if flash {
                Color::new(1.8, 1.8, 1.8, 1.0)
            } else if en.harmless() {
                WHITE
            } else if en.ail.frozen > 0.0 {
                // 언 적(0270 C5): 몸이 얼음빛으로 — 잿빛보다 밝게(바닥에서 떠 보이게, 0170). 얼음 조각은 `overlays::ice_coats`.
                frozen_tint(super::light::enemy_tint())
            } else {
                super::light::enemy_tint()
            };
            self.batch.model(e, &self.pose, inst, tint, lights, glow);
            // 손을 떠난 종(0133): 성소 종 소품의 종 몸만(나무 틀은 감춘다), 날아가는 동안 돈다.
            if let (Some(q), Some(b)) = (hurl_bell(en), self.lib.get("bell")) {
                let frame = b
                    .model
                    .node("bell_frame")
                    .map(|n| (n, Quat::IDENTITY, Vec3::ZERO));
                b.model
                    .pose_with(None, 0.0, frame.as_slice(), &mut self.pose);
                let spin = if matches!(en.state, EState::Tele { .. }) {
                    w.time * 9.0
                } else {
                    0.0
                };
                let inst = Mat4::from_translation(q - Vec3::Y * (1.05 * HURL_BELL_SCALE))
                    * Mat4::from_rotation_y(spin)
                    * Mat4::from_scale(Vec3::splat(HURL_BELL_SCALE));
                self.batch.model(b, &self.pose, inst, WHITE, lights, 0.0);
            }
        } else {
            let base = if en.boss {
                Color::new(0.45, 0.2, 0.5, 1.0)
            } else {
                Color::new(0.5, 0.35, 0.25, 1.0)
            };
            let col = if flash { WHITE } else { base };
            let crouch = matches!(en.state, EState::Tele { .. });
            let h = en.height * if crouch { 0.85 } else { 1.0 } * squash;
            actor(p + sink, en.facing_dir(), en.radius, h, col, 0.6, tilt);
            // 깨진 종지기(0125, 상자 단계): 등 뒤에 끄는 깨진 종 — 울리기 예고 동안 들어 올린다. 모델(0132 `bellkeeper.glb`)이 있으면 쓰지 않는다.
            if en.kind == crate::species::Kind::Bellkeeper {
                let lift = match (en.state, en.atk) {
                    (EState::Tele { t }, crate::enemy::Atk::Toll) => {
                        (t / en.tele_dur().max(1e-3)).clamp(0.0, 1.0) * 1.2
                    }
                    _ => 0.0,
                };
                let at = p + sink - en.facing_dir() * (en.radius + 0.7) + Vec3::Y * (0.6 + lift);
                let bronze = if flash {
                    WHITE
                } else {
                    Color::new(0.55, 0.42, 0.26, 1.0)
                };
                draw_cube(at, vec3(1.2, 1.2, 1.2), None, bronze);
                draw_cube(at + Vec3::Y * 0.75, vec3(0.7, 0.3, 0.7), None, bronze);
                draw_cube_wires(at, vec3(1.22, 1.22, 1.22), Color::new(0.2, 0.14, 0.1, 1.0));
            }
        }
    }

    pub(super) fn player(
        &mut self,
        w: &World,
        cam: &Camera,
        pp: Vec3,
        t: &Tuning,
        lights: &[Light],
        scene: Option<(&'static str, f32)>,
    ) {
        let pl = &w.player;
        // 무적 깜빡임: 짝수 프레임은 밝게.
        let blink = pl.hurt.left > 0.0 && (w.time * 20.0) as i32 % 2 == 0;
        // 카메라가 몸에 닿을 만큼 가까우면 옅게(가림 당기기의 마지막 수단).
        let close = (cam.closeness(pp) - CAM_MIN_DIST).clamp(0.0, 1.0);
        // 락온 중 몸이 과녁을 가리면 옅게(0040): 카메라를 어깨 너머로 비켜 둬도 가까운 큰 대상은 다 비켜나지 못한다.
        let hide = match w.lock.and_then(|i| w.enemies.get(i)).filter(|e| e.alive()) {
            Some(e) if cam.mode == Mode::Lock => occlusion(
                cam.view.pos,
                pp + Vec3::Y * (pl.air + PLAYER_HEIGHT * 0.45),
                PLAYER_RADIUS * 1.2,
                e.pos + Vec3::Y * (e.height * 0.5),
                e.radius.max(e.height * 0.4),
            ),
            _ => 0.0,
        };
        let want =
            1.0 - (1.0 - t[crate::tuning::P::RenderLockFade]) * (hide / 0.35).clamp(0.0, 1.0);
        self.lock_fade += (want - self.lock_fade) * (1.0 - (-10.0 * get_frame_time()).exp_d());
        let a = (0.25 + 0.75 * close) * self.lock_fade;
        if let Some(e) = self.lib.get("wanderer") {
            // 상태 → 동작 이름·시각은 렌더 밖의 순수한 셈(0251 R9 — `anim.rs`, 시험).
            let pick = self.anim.pick(w, t, e);
            let wdt = self.anim.dt;
            let (names, at) = (pick.names(), pick.at);
            // 장면의 자세(0171 깨어남 · 0175 쓰러짐): 장면의 시계가 그 동작을 튼다. 그 동작이 없는 모델이면 상태대로.
            let scene_clip: [&str; 1];
            let (names, at): (&[&str], f32) = match scene {
                Some((clip, at)) if e.model.anim(clip).is_some() => {
                    scene_clip = [clip];
                    (&scene_clip, at)
                }
                _ => (names, at),
            };
            let wake = scene;
            let anim = e.pick(names);
            let looped = pick.looped && wake.is_none();
            let dur = e.duration(anim);
            let at = if looped { at } else { at.min(dur * 0.999) };
            // 키프레임 동작 위에 2차 동작(0039): 꼬리·머리가 늦게 따라오고 몸은 가속하는 쪽으로, 점프·착지에 늘어나고 찌그러진다.
            let rig = *self.sec_rig.get_or_insert_with(|| Rig::of(&e.model));
            self.sec.update(
                &rig,
                get_frame_time(),
                pp,
                pl.facing,
                pl.air,
                pl.vy,
                t,
                &mut self.sec_extra,
            );
            // 든 무기만 보인다(0051): 다른 무기 부위는 크기 0.
            // 던진 동안(0158 G2 던지기)은 든 것도 손에 없다 — 날아가는 자리에 그린다(`blades`).
            let away = w.weapon_away();
            // 던진 방패(0276)가 날고 있는 동안은 왼손에도 없다 — 날아가는 자리에 그린다(`shields`).
            let shield = pl.shield.filter(|_| !w.shield_out());
            super::hide_unheld(&e.model, pl.weapon, shield, away, &mut self.sec_extra);
            // 락온 중 옆·비스듬히 걸을 때(0197): 다리(엉덩이)를 가는 쪽으로 돌리고 윗몸(첫 등뼈)을 되돌려 과녁을 본다 — 부드럽게 따라간다.
            let leg_want = if crate::anim::walking(w) {
                crate::anim::move_anim(pl.pos - pl.prev, pl.facing_dir(), pl.faces_away).1
            } else {
                0.0
            };
            self.leg_turn += (leg_want - self.leg_turn) * (1.0 - (-LEG_TURN_RATE * wdt).exp_d());
            if self.leg_turn.abs() > 1e-3 {
                if let (Some(h), Some(sp)) = (
                    e.model.node("mixamorig:Hips"),
                    e.model.node("mixamorig:Spine"),
                ) {
                    self.sec_extra
                        .push((h, Quat::from_rotation_y(self.leg_turn), Vec3::ONE));
                    self.sec_extra
                        .push((sp, Quat::from_rotation_y(-self.leg_turn), Vec3::ONE));
                }
            }
            // 조용한 동작(서기·걷기·달리기·몸짓)으로 들어갈 때는 지난 자세에서 짧게 섞는다(0175, `gait.rs`) — 누르는 순간 보여야 하는
            // 동작(공격·회피·패링 …)은 바로. 깨어나는 장면 동안은 섞지 않는다.
            let target = e.model.locals(anim, at);
            let calm = anim
                .and_then(|i| e.model.anims.get(i))
                .is_some_and(|a| crate::gait::calm(&a.name));
            if anim != self.shown_anim {
                // 겨눈 자세(옆으로 선 몸)에서 내릴 때는 천천히(0197).
                let from_aim = self
                    .shown_anim
                    .and_then(|i| e.model.anims.get(i))
                    .is_some_and(|a| a.name.starts_with("xbow_"));
                self.blend_len = if from_aim {
                    AIM_BLEND
                } else {
                    crate::gait::BLEND
                };
                self.blend_left = if calm && wake.is_none() && self.shown.len() == target.len() {
                    self.blend_from = self.shown.clone();
                    self.blend_len
                } else {
                    0.0
                };
                self.shown_anim = anim;
            }
            self.shown = if self.blend_left > 0.0 {
                let k = crate::gait::weight_over(self.blend_left, self.blend_len);
                self.blend_left = (self.blend_left - wdt).max(0.0);
                crate::model::blend_locals(&self.blend_from, &target, k)
            } else {
                target
            };
            e.model
                .pose_from_locals(&self.shown, &self.sec_extra, &mut self.pose);
            // 맞으면 몸이 밀리는 쪽으로 젖혀졌다 떨며 돌아온다(0020) — 발을 축으로, 경직 동안.
            let recoil = match pl.state {
                PState::Hurt { t: e, dur } => {
                    let left = (1.0 - e / dur.max(1e-3)).clamp(0.0, 1.0);
                    let away = vec3(pl.vel.x, 0.0, pl.vel.z).normalize_or_zero();
                    let jitter = (w.time * 70.0).sin_d() * 0.08 * left;
                    let ang = RECOIL * left * left + jitter;
                    if away.length() > 0.5 {
                        Quat::from_axis_angle(Vec3::Y.cross(away).normalize(), ang)
                    } else {
                        Quat::IDENTITY
                    }
                }
                _ => Quat::IDENTITY,
            };
            // 강공격이 다 모이면 몸이 잘게 떤다(떼라는 신호, 0034).
            let tremble = match pl.state {
                PState::Charging { t: e } if e >= t[crate::tuning::P::HeavyCharge] => {
                    vec3((w.time * 61.0).sin_d(), 0.0, (w.time * 47.0).cos_d()) * 0.018
                }
                _ => Vec3::ZERO,
            };
            // 휘돌이(0158 G2): 전용 동작이 몸의 한 바퀴를 제 안에 담고 있다 — 그때는 몸을 누른 순간의 쪽에 세워 두고 동작이 돌린다
            // (월드의 바라보는 쪽까지 돌리면 두 번 돈다). 빌린 동작이면 예전대로 월드의 쪽을 따라 돈다.
            self.whirl_clip = e
                .model
                .anim(crate::skills::Id::Whirl.def().clip.name)
                .is_some();
            let yaw = match pl.state {
                PState::Skill {
                    id: crate::skills::Id::Whirl,
                    dir,
                    ..
                } if self.whirl_clip => dir.x.atan2_d(dir.z),
                _ => pl.facing,
            };
            let inst = Mat4::from_translation(pp + Vec3::Y * pl.air + tremble)
                * Mat4::from_quat(recoil)
                * Mat4::from_rotation_y(yaw);
            // 맞은 순간(0143): 몸 전체가 희게 — 히트스톱 동안 그대로 멈춰 있다(KB: 흰 실루엣 → 맞은 자세). 그 뒤는 무적 깜빡임.
            let tint = if wake.is_some() {
                // 장면의 자세 동안(쓰러짐 — 월드가 멈춰 맞은 시계가 서 있다)은 흰 몸·깜빡임 없이.
                Color::new(1.0, 1.0, 1.0, a)
            } else if pl.hurt.since < HURT_WHITE {
                Color::new(3.0, 3.0, 3.0, a)
            } else if blink {
                Color::new(1.8, 1.8, 1.8, a)
            } else {
                Color::new(1.0, 1.0, 1.0, a)
            };
            self.batch.model(e, &self.pose, inst, tint, lights, 1.0);
        } else {
            let pcol = match pl.state {
                _ if blink => WHITE,
                PState::Dodge { .. } => Color::new(0.6, 0.9, 1.0, 0.6),
                PState::Hurt { .. } => Color::new(0.95, 0.45, 0.4, 1.0),
                PState::Attack { .. } => Color::new(1.0, 0.85, 0.5, 1.0),
                PState::Charging { .. } => Color::new(1.0, 0.7, 0.35, 1.0),
                PState::Heavy { .. } => Color::new(1.0, 0.6, 0.3, 1.0),
                PState::Drink { .. } => Color::new(0.75, 0.9, 0.6, 1.0),
                PState::Turn { .. } => Color::new(0.7, 0.95, 0.75, 1.0),
                PState::Parry { .. }
                | PState::Recoil { .. }
                | PState::Block { .. }
                | PState::Rush { .. } => Color::new(0.7, 0.85, 1.0, 1.0),
                PState::Cast { .. } | PState::Throw { .. } | PState::CastCharge { .. } => {
                    Color::new(0.75, 0.7, 1.0, 1.0)
                }
                PState::Jump { .. } | PState::Land { .. } => Color::new(0.9, 0.8, 0.6, 1.0),
                PState::Skill { .. } | PState::Ult { .. } => Color::new(0.6, 1.0, 0.85, 1.0),
                PState::Idle => Color::new(0.85, 0.75, 0.55, 1.0),
            };
            actor(
                pp + Vec3::Y * pl.air,
                pl.facing_dir(),
                PLAYER_RADIUS,
                PLAYER_HEIGHT,
                with_a(pcol, pcol.a * a),
                t[crate::tuning::P::Shade],
                Quat::IDENTITY,
            );
        }
    }

    /// 던진 무기(0158 G2, 던지기): 든 무기의 부위만(다른 부위는 크기 0) 날아가는 자리에 눕혀 돌린다. 모델·부위가 없으면(맨손)
    /// 그리지 않는다 — 빛(`skill_auras`)이 대신 보인다.
    pub(super) fn blades(&mut self, w: &World, alpha: f32, lights: &[Light]) {
        self.javelins(w, alpha, lights);
        self.shields(w, alpha, lights);
        if w.skill_blades.is_empty() {
            return;
        }
        let wpn = w.player.weapon.def();
        let Some(e) = self.lib.get("wanderer") else {
            return;
        };
        let Some(node) = e.model.node(wpn.node) else {
            return;
        };
        let Some((c, _)) = e.model.rest_bounds_under(node) else {
            return;
        };
        let hide: Vec<(usize, Quat, Vec3)> = e
            .model
            .nodes
            .iter()
            .enumerate()
            .filter(|(n, nd)| nd.mesh.is_some() && !e.model.is_under(*n, node))
            .map(|(n, _)| (n, Quat::IDENTITY, Vec3::ZERO))
            .collect();
        e.model.pose_with(None, 0.0, &hide, &mut self.pose);
        let undo = e.model.rest_rotation(node).inverse();
        for b in &w.skill_blades {
            let p = b.prev.lerp(b.pos, alpha);
            // 눕혀서(긴 축이 수평) 세로축으로 빠르게 — 부메랑처럼.
            let rot = Quat::from_rotation_y(b.age * BLADE_SPIN)
                * Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
            let inst = Mat4::from_translation(p)
                * Mat4::from_scale(Vec3::splat(BLADE_SCALE))
                * Mat4::from_quat(rot * undo)
                * Mat4::from_translation(-c);
            self.batch.model(
                e,
                &self.pose,
                inst,
                Color::new(1.6, 1.5, 1.3, 1.0),
                lights,
                1.0,
            );
        }
    }

    /// 던진 창(0274 ②b): 방랑자 모델의 창 부위(`w_spear` — 손에서는 감춘다 `weapon_away`)를 날아가는 쪽으로 곧게 눕혀 그린다 —
    /// 촉(부위의 +z)이 앞, 촉 끝이 탄의 자리. 돌아오는 길은 손 쪽을 향해 꼬리부터 날아온다(촉은 여전히 바깥을 본다).
    fn javelins(&mut self, w: &World, alpha: f32, lights: &[Light]) {
        if !w.javelin_out() {
            return;
        }
        let Some(e) = self.lib.get("wanderer") else {
            return;
        };
        let Some(node) = e.model.node(crate::weapons::Id::Spear.def().node) else {
            return;
        };
        let Some((c, rad)) = e.model.rest_bounds_under(node) else {
            return;
        };
        let hide: Vec<(usize, Quat, Vec3)> = e
            .model
            .nodes
            .iter()
            .enumerate()
            .filter(|(n, nd)| nd.mesh.is_some() && !e.model.is_under(*n, node))
            .map(|(n, _)| (n, Quat::IDENTITY, Vec3::ZERO))
            .collect();
        e.model.pose_with(None, 0.0, &hide, &mut self.pose);
        let undo = e.model.rest_rotation(node).inverse();
        for b in w
            .bolts
            .iter()
            .filter(|b| b.def.look == crate::weapons::Look::Javelin)
        {
            let p = b.prev.lerp(b.pos, alpha);
            // 돌아오는 길은 꼬리가 앞(촉은 던진 쪽을 본 채 당겨 오듯).
            let tip = if b.back { -b.dir } else { b.dir };
            let rot = Quat::from_rotation_arc(Vec3::Z, tip.normalize_or(Vec3::Z));
            let k = JAVELIN_SCALE * b.def.scale;
            let inst = Mat4::from_translation(p - tip * rad * k * 0.85)
                * Mat4::from_scale(Vec3::splat(k))
                * Mat4::from_quat(rot * undo)
                * Mat4::from_translation(-c);
            self.batch.model(
                e,
                &self.pose,
                inst,
                Color::new(1.4, 1.35, 1.25, 1.0),
                lights,
                1.0,
            );
        }
    }
}

impl Renderer {
    /// 던진 방패(0276 — 하데스의 방패): 방랑자 모델의 방패 부위(왼손에서는 감춘다)를 날아가는 자리에 눕혀 세로축으로 빠르게 돌린다 —
    /// 원반처럼. 돌아오는 길도 같다.
    fn shields(&mut self, w: &World, alpha: f32, lights: &[Light]) {
        let Some(id) = w.player.shield.filter(|_| w.shield_out()) else {
            return;
        };
        let Some(e) = self.lib.get("wanderer") else {
            return;
        };
        let Some(node) = e.model.node(id.def().node) else {
            return;
        };
        let Some((c, _)) = e.model.rest_bounds_under(node) else {
            return;
        };
        let hide: Vec<(usize, Quat, Vec3)> = e
            .model
            .nodes
            .iter()
            .enumerate()
            .filter(|(n, nd)| nd.mesh.is_some() && !e.model.is_under(*n, node))
            .map(|(n, _)| (n, Quat::IDENTITY, Vec3::ZERO))
            .collect();
        e.model.pose_with(None, 0.0, &hide, &mut self.pose);
        let undo = e.model.rest_rotation(node).inverse();
        for b in w
            .bolts
            .iter()
            .filter(|b| b.def.look == crate::weapons::Look::Shield)
        {
            let p = b.prev.lerp(b.pos, alpha);
            let rot =
                Quat::from_rotation_y(w.time * SHIELD_SPIN) * Quat::from_rotation_x(SHIELD_TILT);
            let inst = Mat4::from_translation(p)
                * Mat4::from_scale(Vec3::splat(SHIELD_SCALE))
                * Mat4::from_quat(rot * undo)
                * Mat4::from_translation(-c);
            self.batch.model(
                e,
                &self.pose,
                inst,
                Color::new(1.5, 1.45, 1.35, 1.0),
                lights,
                1.0,
            );
        }
    }
}

/// 던진 방패(0276): 도는 빠르기(라디안/초) · 눕힌 각(라디안 — 원반처럼) · 크기 배. 0277: 1.1 → 1.75 — 쿼터뷰에서 손바닥만 했다(122-6).
/// 키워도 도는 판의 반지름이 탄의 맞는 반지름(`BOLT_RADIUS` × 방패 던지기의 크기 배) 안이다(시험 — 그림이 판정보다 크지 않게).
const SHIELD_SPIN: f32 = 22.0;
const SHIELD_TILT: f32 = std::f32::consts::FRAC_PI_2;
pub(super) const SHIELD_SCALE: f32 = 1.75;

/// 준비 동작 단서(0073): 예고 끝까지 눈·핵 빛 +, 버티는 동안 떨림(m, 몸 옆으로), 정점 번쩍임 길이(초)·세기.
const WINDUP_GLOW: f32 = 0.6;
/// 2단계 전환(0133)이 터지는 틱의 눈·핵 밝기 더함.
const RISE_GLOW: f32 = 2.2;

/// 종 내던지기(0133): 날아가는 종이 그리는 포물선 높이(m)·크기(성소 종 소품의 종 몸을 키워 쓴다).
const HURL_ARC: f32 = 4.5;
const HURL_BELL_SCALE: f32 = 1.2;

/// 종 내던지기(0133)에서 종이 손을 떠났다면 그 종의 자리(날아가는 중 = 포물선, 떨어진 뒤 = 떨어진 자리). 아니면 None.
fn hurl_bell(en: &Enemy) -> Option<Vec3> {
    if en.atk != crate::enemy::Atk::Hurl {
        return None;
    }
    let target = en.plant_at?;
    let hand = en.pos + en.facing_dir() * 0.9 + Vec3::Y * 2.6;
    match en.state {
        EState::Tele { t } => {
            let k = t / en.tele_dur().max(1e-3);
            let throw = crate::tuning::HURL_THROW;
            if k < throw {
                return None;
            }
            let u = ((k - throw) / (1.0 - throw)).clamp(0.0, 1.0);
            let mut q = hand.lerp(target, u);
            q.y += HURL_ARC * 4.0 * u * (1.0 - u);
            Some(q)
        }
        EState::Swing { .. } | EState::Wave { .. } => Some(target),
        _ => None,
    }
}
/// 들어 올리는 동안 키 늘림(배)·뒤로 젖힘(라디안).
const WINDUP_STRETCH: f32 = 0.06;
const WINDUP_LEAN: f32 = 0.1;
const WINDUP_TREMBLE: f32 = 0.025;
/// 지연 공격(0128)의 몸 신호: 더 젖힘(배)·더 떨림(배).
const DELAY_LEAN: f32 = 1.8;
const DELAY_TREMBLE: f32 = 2.0;
const WINDUP_GLINT: f32 = 0.1;
const WINDUP_GLINT_GAIN: f32 = 1.2;
/// 맞았을 때 몸이 젖혀지는 각(라디안, 경직 처음).
const RECOIL: f32 = 0.35;
/// 죽은 적이 넘어져 사라지는 데 걸리는 초.
pub(super) const DEATH_TIME: f32 = 1.2;
/// 언 몸(0270 C5 — `frozen_tint`): 서리 빛으로 섞는 몫 · 그 빛의 밝기(잿빛보다 밝게 — 0170).
const ICE_TINT: f32 = 0.8;
const ICE_LIFT: f32 = 2.0;

/// 적 상태 → (동작, 그 동작의 몇 초). 후보 이름을 차례로 찾고(모델마다 가진 동작이 다르다), 예고 동안은 진행에 맞춰 든다:
/// `_windup` 동작이면 예고 전체에, 한 동작(`attack`)이면 앞 절반이 예고 · 뒤 절반이 치기.
fn enemy_anim(
    en: &Enemy,
    model: &str,
    e: &Entry,
    clock: f32,
    stage: Option<&StageRun>,
) -> (Option<usize>, f32) {
    let phase = (en.pos.x * 0.37 + en.pos.z * 0.61).rem_euclid(1.0);
    let moving = (en.pos - en.prev).length() > 0.005;
    let windup: &[&str] = match en.atk {
        Atk::Charge => &["charge_windup", "attack"],
        Atk::Slam => &["slam", "attack"],
        Atk::Wave => &["wave", "slam", "attack"],
        Atk::Smash => &["attack"],
        Atk::Plant => &["plant", "attack"],
        Atk::Toll => &["toll", "wave", "slam", "attack"],
        Atk::Shards => &["throw", "attack"],
        Atk::Roots => &["roots", "slam", "attack"],
        Atk::Hurl => &["hurl", "toll", "throw", "attack"],
        Atk::Hook => &["hook", "charge_windup", "throw", "attack"],
        Atk::Shell => &["shell", "guard", "idle"],
        // 내려 딛기(0270 D): 뒷발로 일어섰다 앞발로 쿵.
        Atk::Stomp => &["rear", "slam", "attack"],
    };
    let once = |names: &[&str], k: f32| {
        let a = e.pick(names);
        (a, k.clamp(0.0, 0.999) * e.duration(a))
    };
    let is_windup =
        |a: Option<usize>| a.is_some_and(|i| e.model.anims[i].name.ends_with("_windup"));
    // 뛰기(턱 오르내림 0100 · 꼬투리 물러나기 0120): 모델에 `hop`이 있으면 뛰기 진행대로 한 번(0132), 없으면 상태대로.
    if let Some(h) = en.hop {
        if e.pick(&["hop"]).is_some() {
            return once(&["hop"], h.t / h.dur.max(1e-3));
        }
    }
    match en.state {
        EState::Dormant => {
            // 필드 보스(0270 D)는 제 시계로 일어서고 눕는다(`Enemy::stand`) — 문지기는 스테이지의 깨어남.
            let w = if en.field {
                en.stand
            } else {
                stage.map(|s| s.wake_progress()).unwrap_or(0.0)
            };
            match e.pick(&["wake"]) {
                Some(a) => (Some(a), w.clamp(0.0, 0.999) * e.duration(Some(a))),
                None => (e.pick(&["idle"]), 0.0),
            }
        }
        // 0242: 청소자가 먹는 자리를 옮기는 동안은 걷는다.
        EState::Idle if moving => (e.pick(&["walk", "idle"]), clock * 0.6 + phase),
        EState::Idle => (e.pick(&["idle", "walk"]), clock * 0.7 + phase),
        EState::Return => (e.pick(&["walk", "run", "idle"]), clock + phase),
        EState::Chase if moving => (e.pick(&["walk", "run", "idle"]), clock * 1.0 + phase),
        EState::Chase => (e.pick(&["idle", "walk"]), clock + phase),
        EState::Tele { t } => {
            let dur = en.tele_dur().max(1e-3);
            let lift = en.lift_dur().max(1e-3);
            let a = e.pick(windup);
            if is_windup(a) {
                // 지연(0128): 들어 올림까지는 같은 빠르기로 85%, 버팀, 마지막 순간 끝까지.
                let snap = WINDUP_SNAP.min(dur * 0.25);
                let k = if t < dur - snap {
                    (t / (lift - snap).max(1e-3)).min(1.0) * 0.85
                } else {
                    0.85 + 0.15 * ((t - (dur - snap)) / snap).clamp(0.0, 1.0)
                };
                once(windup, k)
            } else {
                // 2026-11-07(0073): 들어 올림 → 정점에서 버팀 → 마지막 순간 내리침. 내리친 자세 = 맞는 틱.
                let (peak, impact) = strike_keys(model, a, e);
                (a, windup_clock(t, dur, lift, peak, impact))
            }
        }
        EState::Swing { t } => {
            let a = e.pick(windup);
            if is_windup(a) {
                once(&["attack"], 0.5 + t * 2.0)
            } else {
                // 맞는 틱부터 제 빠르기로 끝맺음.
                let (_, impact) = strike_keys(model, a, e);
                (a, (impact + t).min(e.duration(a) * 0.999))
            }
        }
        EState::Charge { .. } => (e.pick(&["charge", "run", "walk"]), clock * 1.5),
        // 울림 고리(0125 `Atk::Toll`)도 이 상태 — 예고에서 고른 동작(`toll` …)을 이어 튼다(0132).
        EState::Wave { t } => {
            let a = e.pick(windup);
            let (_, impact) = strike_keys(model, a, e);
            (a, (impact + t).min(e.duration(a) * 0.999))
        }
        EState::Recover { t } => match e.pick(&["recover"]) {
            Some(a) => (Some(a), t.min(e.duration(Some(a)) * 0.999)),
            None => (e.pick(&["idle"]), clock + phase),
        },
        // 2단계 전환(0133): 깨어남 동작을 거꾸로 무릎 꿇고(0.5초) 버티다가, 터지는 틱부터 일어선다.
        EState::Rise { t } => {
            use crate::tuning::{RISE_BURST, RISE_TIME};
            let k = if t < 0.5 {
                1.0 - t / 0.5
            } else if t < RISE_BURST {
                0.0
            } else {
                (t - RISE_BURST) / (RISE_TIME - RISE_BURST)
            };
            once(&["rise", "wake", "hurt"], k)
        }
        EState::Stagger { .. } => once(&["hurt", "idle"], 0.3),
        EState::Dead => once(&["hurt", "idle"], 0.999),
    }
}

/// 한 동작(`attack`·`slam`·`wave`)의 내리치는 열쇠(0073): (정점 = 다 들어 올린 초, 맞음 = 내리친 초).
/// Blender 스크립트(`tools/art/blender/<모델>.py`)의 키 시각과 같다 — 동작을 고치면 여기도. 표에 없으면 동작의 40%·50%.
pub(super) const STRIKE_KEYS: &[(&str, &str, f32, f32)] = &[
    ("root_husk", "attack", 0.5, 0.62),
    ("repeater", "attack", 0.5, 0.62), // 잔재의 동작 표 그대로(0234)
    ("gatekeeper", "slam", 0.9, 1.0),
    ("gatekeeper", "wave", 1.0, 1.12),
    ("shellback", "attack", 0.6, 0.72),
    ("pod", "plant", 0.45, 0.6),
    ("bellkeeper", "toll", 1.0, 1.12),
    ("bellkeeper", "slam", 0.9, 1.0),
    ("bellkeeper", "throw", 0.6, 0.7),
    ("mossward", "slam", 0.9, 1.0),
    // 껍질(0185): 0.5에 다 웅크리고 거기서 버틴다 — 맞음은 거의 같은 자세(풀기는 `recover`).
    ("mossward", "shell", 0.5, 0.55),
    // 권양지기(0234): 문지기의 두 발 표 + 갈고리(오른팔 내던지기)·추(두 팔로 들어 던지기).
    ("winchkeeper", "slam", 0.9, 1.0),
    ("winchkeeper", "wave", 1.0, 1.12),
    ("winchkeeper", "hook", 0.8, 0.9),
    ("winchkeeper", "hurl", 0.9, 1.0),
    // 고요한 사슴(0270 D): 뿔 쓸기 · 뒷발로 일어섰다 내려 딛기(앞발이 닿는 1.0).
    ("hushed_stag", "slam", 0.9, 1.0),
    ("hushed_stag", "rear", 0.9, 1.0),
];

fn strike_keys(model: &str, a: Option<usize>, e: &Entry) -> (f32, f32) {
    let Some(i) = a else { return (0.0, 0.0) };
    let name = e.model.anims[i].name.as_str();
    STRIKE_KEYS
        .iter()
        .find(|(m, n, _, _)| *m == model && *n == name)
        .map(|&(_, _, p, h)| (p, h))
        .unwrap_or_else(|| {
            let d = e.duration(a);
            (d * 0.4, d * 0.5)
        })
}

/// 예고가 다 들어 올리는 진행(0073): 예고의 이만큼에서 정점에 닿고, 그 뒤는 버팀.
pub(crate) const WINDUP_REACH: f32 = 0.6;
/// 마지막 내리침(초) — 정점 → 맞음. 이 동안은 이미 늦다(KB: 적의 공격 자체는 빠르고 짧게).
pub(crate) const WINDUP_SNAP: f32 = 0.08;

/// 예고 `t`초(길이 `dur`)의 동작 시각: 들어 올림(천천히 들어가고 멈추게) → 정점에서 버팀 → 마지막 `WINDUP_SNAP` 동안 내리침.
pub(super) fn windup_clock(t: f32, dur: f32, lift: f32, peak: f32, impact: f32) -> f32 {
    let snap = WINDUP_SNAP.min(dur * 0.25);
    let reach = (lift * WINDUP_REACH).min(dur - snap);
    if t < reach {
        // 천천히 들어가고 천천히 멈춘다(smoothstep) — 들기 전 움츠림(동작 앞쪽 키)이 눈에 남게. 처음엔 빠르게 들었더니 움츠림이 3프레임에 지나갔다.
        let u = t / reach.max(1e-3);
        peak * u * u * (3.0 - 2.0 * u)
    } else if t < dur - snap {
        peak
    } else {
        let u = ((t - (dur - snap)) / snap).clamp(0.0, 1.0);
        peak + (impact - peak) * u
    }
}

/// 버팀(0073): 예고에서 정점에 닿은 뒤 흐른 초(아니면 None). 이 동안 몸이 떨리고, 닿는 순간 눈·핵이 한 번 번쩍인다 —
/// 바닥 예고 그림이 없어도 몸으로 읽히게.
fn windup_hold(en: &Enemy) -> Option<f32> {
    let EState::Tele { t } = en.state else {
        return None;
    };
    let start = en.lift_dur().max(1e-3) * WINDUP_REACH;
    (t >= start).then_some(t - start)
}

/// 다리 돌림이 따라가는 빠르기(1/초).
const LEG_TURN_RATE: f32 = 12.0;
/// 겨눈 자세에서 서기로 섞는 초(보통은 `gait::BLEND`).
const AIM_BLEND: f32 = 0.35;

/// 휘청 기울기(가장 큰 각, 라디안). 세기 1(가장 멀리 미끄러진 적)이 이만큼 꺾인다.
const LEAN_MAX: f32 = 0.6;
/// 비틀비틀 작은 원 흔들림(라디안).
const DIZZY: f32 = 0.1;

/// 휘청 몸짓: 머리가 기우는 쪽(XZ) × 각(라디안). 버스가 급정거할 때처럼 발은 바닥에 끌리고 몸은 미끄러지는 쪽으로 꺾인다 —
/// 빠르게 기울었다 앞뒤로 두어 번 흔들리며 잦아들고(감쇠 진동), 휘청이 끝날 때까지 작게 원을 그린다(`left` 1 → 0이면 사라짐).
/// `sway` = 쏟아진 쪽 × 세기, `t` = 그때부터 지난 초, `clock` = 원 흔들림 위상.
pub fn lean(sway: Vec3, t: f32, clock: f32, left: f32) -> Vec3 {
    let k = sway.length();
    if k < 1e-4 {
        return Vec3::ZERO;
    }
    let dir = sway / k;
    // 감쇠 진동(초당 두 번): 0.125초에 가장 크게(≈ 0.7), 0.375초에 반대로 한 번, 1초면 거의 선다.
    let swing = (-3.0 * t).exp_d() * (std::f32::consts::TAU * 2.0 * t).sin_d();
    let (s, c) = (clock * 9.0).sin_cos_d();
    dir * (LEAN_MAX * k * swing) + vec3(c, 0.0, s) * (DIZZY * left)
}

/// 상자 몸 + 작은 코. `dir`은 바라보는 단위 벡터(XZ). `tilt`는 발(바닥 가운데)을 축으로 몸 전체를 기울인다.
fn actor(p: Vec3, dir: Vec3, r: f32, h: f32, col: Color, shade: f32, tilt: Quat) {
    let body = vec3(r * 2.0, h, r * 2.0);
    let center = p + Vec3::Y * (h * 0.5);
    shaded_box_rot(center, body, col, shade, tilt, p);
    box_wires(center, body, Color::new(0.05, 0.05, 0.05, 1.0), tilt, p);
    shaded_box_rot(
        p + dir * (r + 0.1) + Vec3::Y * (h * 0.8),
        vec3(0.2, 0.2, 0.2),
        Color::new(0.1, 0.1, 0.1, 1.0),
        shade,
        tilt,
        p,
    );
}

/// 맞은 순간 방랑자의 몸이 희게 뜨는 시간(초, 로직 시계 — 히트스톱 동안은 멈춰 있어 실제로는 그만큼 더 길다).
const HURT_WHITE: f32 = 0.05;

/// 던진 무기(0158 G2)가 도는 빠르기(rad/s)와 손에 든 때보다 크게 보이는 배(날아가는 것은 작게 읽힌다).
pub(super) const BLADE_SPIN: f32 = 26.0;

const BLADE_SCALE: f32 = 1.25;
/// 던진 창(0274 ②b)을 그리는 배 — 손에 든 것과 같은 크기(창은 길어 날아가도 읽힌다).
const JAVELIN_SCALE: f32 = 1.0;

/// 줍기·상자 열기 몸짓의 길이(초, 0196 — 앱이 이만큼 흐르면 그친다).
pub const PICKUP_CLIP: f32 = 0.9;
/// 장치 당기기 몸짓의 길이(초, 0197).
pub const PULL_CLIP: f32 = 0.65;

/// 손으로 하는 일의 몸짓 길이(초) — 모르는 이름은 0(바로 그친다).
pub fn gesture_len(name: &str) -> f32 {
    match name {
        "pickup" => PICKUP_CLIP,
        "pull" => PULL_CLIP,
        _ => 0.0,
    }
}

/// 언 적의 몸빛(0270 C5): 잿빛 보정(`enemy_tint`)을 밝은 서리 빛(`status::INK` 서리 × `ICE_LIFT`) 쪽으로 `ICE_TINT`만큼 — 빨강은 덜,
/// 초록·파랑은 더, 밝기는 잿빛보다 위(바닥에서 떠 보여야 한다, 0170).
fn frozen_tint(ash: Color) -> Color {
    let [r, g, b] = crate::status::INK[crate::status::Kind::Frost.slot().unwrap_or(6)];
    let ice = |a: f32, c: f32| a + (c * ICE_LIFT - a) * ICE_TINT;
    Color::new(ice(ash.r, r), ice(ash.g, g), ice(ash.b, b), 1.0)
}
