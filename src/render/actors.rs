//! 배우: 적·방랑자·던진 무기 — 상태 → 동작 이름과 시각, 준비 동작의 몸 신호, 휘청 기울기, 모델이 없을 때의 상자 배우.
use super::*;

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
                let shake = (get_time() as f32 * 90.0).sin() * crate::tuning::HIT_TREMBLE * struck;
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
            squash = 1.0 - 0.2 * en.sway.length() * (-12.0 * en.sway_t).exp();
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
        let delayed = matches!(en.tempo, crate::enemy::Tempo::Delayed(_));
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
            let shake = (get_time() as f32 * 70.0).sin() * tremble;
            p += vec3(en.facing.cos(), 0.0, -en.facing.sin()) * shake;
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
            let hide = hurl_bell(en)
                .and(e.model.node("bell"))
                .map(|n| (n, Quat::IDENTITY, Vec3::ZERO));
            e.model.pose_with(anim, at, hide.as_slice(), &mut self.pose);
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
            let tint = if flash {
                Color::new(1.8, 1.8, 1.8, 1.0)
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
    ) {
        let pl = &w.player;
        // 무적 깜빡임: 짝수 프레임은 밝게.
        let blink = pl.hurt.left > 0.0 && (w.time * 20.0) as i32 % 2 == 0;
        // 카메라가 몸에 닿을 만큼 가까우면 옅게(가림 당기기의 마지막 수단).
        let close = (cam.closeness(pp) - CAM_MIN_DIST).clamp(0.0, 1.0);
        // 락온 중 몸이 과녁을 가리면 옅게(0040): 카메라를 어깨 너머로 비켜 둬도 가까운 큰 대상은 다 비켜나지 못한다.
        let hide = match w.lock.and_then(|i| w.enemies.get(i)).filter(|e| e.alive()) {
            Some(e) if cam.mode == Mode::Lock => occlusion(
                cam.rig.pos,
                pp + Vec3::Y * (pl.air + PLAYER_HEIGHT * 0.45),
                PLAYER_RADIUS * 1.2,
                e.pos + Vec3::Y * (e.height * 0.5),
                e.radius.max(e.height * 0.4),
            ),
            _ => 0.0,
        };
        let want =
            1.0 - (1.0 - t[crate::tuning::P::RenderLockFade]) * (hide / 0.35).clamp(0.0, 1.0);
        self.lock_fade += (want - self.lock_fade) * (1.0 - (-10.0 * get_frame_time()).exp());
        let a = (0.25 + 0.75 * close) * self.lock_fade;
        if let Some(e) = self.lib.get("wanderer") {
            let speed = (pl.pos - pl.prev).length() / TICK_DT;
            let wpn = pl.weapon.def();
            let hv = wpn.heavy;
            let combo_anim = |hit: u8| wpn.hit(hit).map_or("attack", |h| h.anim);
            let owned: [&str; 3];
            // 서 있는 동안(0153): 숨 고리 + 조용할 때 가끔 몸짓. 조용함 = 락온이 아니고 가까이 깨어 있는 적이 없다.
            let idle_pose = {
                use crate::idle::FIDGETS;
                let dt = (w.time - self.idle_at).clamp(0.0, 0.1);
                self.idle_at = w.time;
                let standing = matches!(pl.state, PState::Idle) && speed <= 0.5;
                let calm = w.lock.is_none()
                    && !w.enemies.iter().any(|en| {
                        en.active() && en.pos.distance(pl.pos) < crate::tuning::IDLE_CALM_RADIUS
                    });
                let dur_of = |n: &str| e.model.anim(n).map_or(0.0, |i| e.duration(Some(i)));
                self.idle
                    .step(dt, standing, calm, dur_of("idle"), FIDGETS.map(dur_of))
            };
            let (names, at): (&[&str], f32) = match pl.state {
                // 콤보(0051): 무기 표의 타마다 동작(없는 모델이면 평타).
                // 몰아치기(0158 G2) 동안은 평타의 시계가 빨라진다 — 동작도 같은 배로 돌린다.
                PState::Attack { t: e, hit, .. } => {
                    owned = [combo_anim(hit), "attack", "attack"];
                    (&owned, e * w.flurry_rate())
                }
                // 강공격(0030): 모으는 동안 칼을 뒤로 끌어 든 자세(없는 모델이면 공격 첫 자세), 떼면 크게 내려친다 — 무기마다(0051).
                PState::Charging { t: e } => {
                    owned = [hv.map_or("charge", |h| h.anim_charge), "charge", "attack"];
                    (&owned, e)
                }
                PState::Heavy { t: e, .. } => {
                    owned = [hv.map_or("heavy", |h| h.anim_heavy), "heavy", "attack"];
                    (&owned, e)
                }
                // 수류탄(0051): 오른손으로 아래에서 위로 던진다.
                PState::Throw { t: e, .. } => (&["throw", "cast", "attack"], e),
                // 등잔(0031): 왼손으로 들어 입에 대고(들기 = `flask.raise s`) 내린다.
                PState::Drink { t: e, .. } => (&["drink", "idle"], e),
                PState::Dodge { t: e, .. } => (&["dodge", "walk"], e),
                PState::Hurt { t: e, .. } => (&["hurt", "idle"], e),
                PState::Recoil { t: e } => (&["hurt", "idle"], e),
                // 0034: 도는 쪽으로 윗몸이 먼저 감긴다(오른쪽 = 요가 준다).
                PState::Turn { t: e, from, to } => (
                    if crate::world::turn_is_right(from, to) {
                        &["turn_r", "turn", "idle"]
                    } else {
                        &["turn_l", "turn", "idle"]
                    },
                    e,
                ),
                PState::Parry { t: e } => (&["parry", "attack"], e),
                // 마법(0048): 손을 앞으로 내미는 자세(전용 동작이 생기기 전엔 패링 자세).
                PState::Cast { t: e, .. } if matches!(wpn.rb, crate::weapons::Rb::Shot { .. }) => {
                    (&["xbow_shoot", "cast", "attack"], e)
                }
                PState::Cast { t: e, .. } => (&["cast", "parry", "attack"], e),
                // 원거리 강공격 모으기(0056): 쇠뇌는 어깨에 대고 시위를 당긴 채, 마법은 두 손을 모아 빛을 키우는 자세를 쥔다.
                PState::CastCharge { t: e }
                    if matches!(wpn.rb, crate::weapons::Rb::Shot { .. }) =>
                {
                    (&["xbow_charge", "xbow_shoot", "cast"], e)
                }
                PState::CastCharge { t: e } => (&["cast_charge", "cast", "parry"], e),
                // 공중(0027): 방랑자의 jump·plunge·land 동작(없는 모델이면 회피·공격 자세로).
                // 공중 RB(0111): 떠 있는 동안 쏘는 자세(쇠뇌는 겨누고 쏘기, 마법은 손을 내밂).
                PState::Jump { plunge: false, .. }
                    if pl.ranged.hover > 0.0
                        && matches!(wpn.rb, crate::weapons::Rb::Shot { .. }) =>
                {
                    (&["xbow_shoot", "cast"], 0.1)
                }
                PState::Jump { plunge: false, .. } if pl.ranged.hover > 0.0 => {
                    (&["cast", "jump"], 0.12)
                }
                PState::Jump { plunge: false, .. } => (&["jump", "dodge", "idle"], 0.1),
                // 내려찍기: 떨어지는 동안 칼을 들고 있다가 땅에 가까우면 꽂는 자세로.
                PState::Jump { plunge: true, .. } => (
                    &["plunge", "attack"],
                    if pl.air < 0.35 { 0.29 } else { 0.12 },
                ),
                PState::Land { t: e, .. } => (&["land", "idle"], e),
                // 스킬·큰 기술(0151 → 0158 G2): 전용 동작(`sk_*`·`ult_*`)이 모델에 있으면 그것을 — 상태의 구간(시동·이어짐·후딜)을
                // 동작의 구간에 맞춘다(`Clip::at`: 단·고르기가 길이를 바꿔도 판정의 순간에 그 자세). 없으면 빌려 쓰던 동작(표의 `anim`).
                PState::Skill { id, t: el, .. } => {
                    let eff = w.skill_eff(id);
                    let sk = eff.skill;
                    if e.model.anim(sk.clip.name).is_some() {
                        owned = [sk.clip.name, sk.anim, "attack"];
                        let at = sk
                            .clip
                            .at(el, sk.startup, eff.active(), clip_period(&sk.form));
                        (&owned, at)
                    } else {
                        owned = [sk.anim, "heavy", "attack"];
                        (&owned, el)
                    }
                }
                PState::Ult { kind, t: el, .. } => {
                    let clip = kind.clip();
                    if e.model.anim(clip.name).is_some() {
                        // 세계 멈춤 동안은 상태 시계가 0에 머문다 — 멈춤이 지난 몫으로 시동 구간을 돌린다.
                        let (freeze, ..) = kind.times();
                        let frozen = 1.0 - (w.hitstop / freeze.max(1e-3)).clamp(0.0, 1.0);
                        owned = [clip.name, kind.anim(), "attack"];
                        (&owned, kind.clip_at(el, frozen))
                    } else {
                        owned = [kind.anim(), "heavy", "attack"];
                        (&owned, el)
                    }
                }
                // 달리기(0076): `run` 동작이 없으면 걷기를 빠르게(발 구르기가 속도를 따라간다).
                PState::Idle if pl.running && speed > 0.5 => {
                    (&["run", "walk"], w.time * (speed / 4.0).clamp(0.6, 2.2))
                }
                PState::Idle if speed > 0.5 => (
                    move_anim(pl.pos - pl.prev, pl.facing_dir(), w.lock.is_some()),
                    w.time * (speed / 4.0).clamp(0.6, 1.6),
                ),
                PState::Idle => match idle_pose {
                    crate::idle::Pose::Loop(t) => (&["idle"], t),
                    crate::idle::Pose::Fidget(i, t) => {
                        owned = [crate::idle::FIDGETS[i], "idle", "idle"];
                        (&owned, t)
                    }
                },
            };
            let anim = e.pick(names);
            let looped = matches!(pl.state, PState::Idle);
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
            for w2 in crate::weapons::WEAPONS.iter() {
                if w2.id != pl.weapon || away {
                    if let Some(n) = e.model.node(w2.node) {
                        self.sec_extra.push((n, Quat::IDENTITY, Vec3::ZERO));
                    }
                }
            }
            e.model.pose_with(anim, at, &self.sec_extra, &mut self.pose);
            // 맞으면 몸이 밀리는 쪽으로 젖혀졌다 떨며 돌아온다(0020) — 발을 축으로, 경직 동안.
            let recoil = match pl.state {
                PState::Hurt { t: e, dur } => {
                    let left = (1.0 - e / dur.max(1e-3)).clamp(0.0, 1.0);
                    let away = vec3(pl.vel.x, 0.0, pl.vel.z).normalize_or_zero();
                    let jitter = (w.time * 70.0).sin() * 0.08 * left;
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
                    vec3((w.time * 61.0).sin(), 0.0, (w.time * 47.0).cos()) * 0.018
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
                } if self.whirl_clip => dir.x.atan2(dir.z),
                _ => pl.facing,
            };
            let inst = Mat4::from_translation(pp + Vec3::Y * pl.air + tremble)
                * Mat4::from_quat(recoil)
                * Mat4::from_rotation_y(yaw);
            // 맞은 순간(0143): 몸 전체가 희게 — 히트스톱 동안 그대로 멈춰 있다(KB: 흰 실루엣 → 맞은 자세). 그 뒤는 무적 깜빡임.
            let tint = if pl.hurt.since < HURT_WHITE {
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
                PState::Parry { .. } | PState::Recoil { .. } => Color::new(0.7, 0.85, 1.0, 1.0),
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
        for b in &w.skill_blades {
            let p = b.prev.lerp(b.pos, alpha);
            // 눕혀서(긴 축이 수평) 세로축으로 빠르게 — 부메랑처럼.
            let rot = Quat::from_rotation_y(b.age * BLADE_SPIN)
                * Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
            let inst = Mat4::from_translation(p)
                * Mat4::from_scale(Vec3::splat(BLADE_SCALE))
                * Mat4::from_quat(rot)
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
}

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
            let w = stage.map(|s| s.wake_progress()).unwrap_or(0.0);
            match e.pick(&["wake"]) {
                Some(a) => (Some(a), w.clamp(0.0, 0.999) * e.duration(Some(a))),
                None => (e.pick(&["idle"]), 0.0),
            }
        }
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
    ("gatekeeper", "slam", 0.9, 1.0),
    ("gatekeeper", "wave", 1.0, 1.12),
    ("shellback", "attack", 0.6, 0.72),
    ("pod", "plant", 0.45, 0.6),
    ("bellkeeper", "toll", 1.0, 1.12),
    ("bellkeeper", "slam", 0.9, 1.0),
    ("bellkeeper", "throw", 0.6, 0.7),
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

/// 걷는 동작 고르기(0034): 락온 중엔 몸이 대상을 보니 걷는 쪽이 옆이면 게걸음(`strafe_l`·`strafe_r`), 뒤면 뒷걸음.
/// 락온이 아니면 늘 앞으로 걷는다(몸이 걷는 쪽을 본다). 없는 모델이면 `walk`로.
pub fn move_anim(step: Vec3, facing: Vec3, locked: bool) -> &'static [&'static str] {
    if !locked {
        return &["walk"];
    }
    let v = vec3(step.x, 0.0, step.z);
    let fwd = v.dot(facing);
    // 오른쪽 = 앞 × 위(카메라·입력과 같은 축).
    let side = v.dot(facing.cross(Vec3::Y));
    if side.abs() > fwd.abs() * 1.2 {
        if side > 0.0 {
            &["strafe_r", "walk"]
        } else {
            &["strafe_l", "walk"]
        }
    } else if fwd < 0.0 {
        &["walk_back", "walk"]
    } else {
        &["walk"]
    }
}

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
    let swing = (-3.0 * t).exp() * (std::f32::consts::TAU * 2.0 * t).sin();
    let (s, c) = (clock * 9.0).sin_cos();
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

/// 스킬 동작의 이어짐 구간을 되풀이하는 주기(0158 G2): 휘돌이는 바퀴마다, 가로지르기는 길마다(되돌아 베기의 둘째 길).
fn clip_period(form: &crate::skills::Form) -> Option<f32> {
    match *form {
        crate::skills::Form::Ring { gap, .. } => Some(gap),
        crate::skills::Form::Lane { time, .. } => Some(time),
        _ => None,
    }
}
