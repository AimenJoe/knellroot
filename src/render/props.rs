//! 스테이지 물건: 문·새긴 돌·상자·문 장치·남은 주민과 생활 물건·빛줄기·떨어진 아이템·숨은 물건·쉼터·갈 곳 빛기둥·핵.
use super::*;

impl Renderer {
    /// 문(열리면 땅으로 가라앉거나 `open` 동작)과 핵(문지기가 남긴 것, 떠서 돈다).
    /// 스테이지 문(진하기 `a` — 1 = 불투명, 옅으면 외곽선·그림자 실루엣 없이).
    pub(super) fn draw_gate(&mut self, run: &StageRun, lights: &[Light], a: f32) {
        let Some((g, c)) = run.level.gate else {
            return;
        };
        let k = (run.gate_open / GATE_TIME).clamp(0.0, 1.0);
        if k >= 1.0 {
            return;
        }
        match self.lib.get("gate") {
            Some(e) => {
                let open = e.pick(&["open"]);
                let sink = if open.is_some() { 0.0 } else { k * 4.6 };
                e.model
                    .pose(open, k * e.duration(open) * 0.999, &mut self.pose);
                let inst = Mat4::from_translation(c - Vec3::Y * sink);
                // 문은 건물이다(0140): 배우의 밝기 올림(`ACTOR_LIFT`)을 되돌려 벽·기둥과 같은 밝기로 — 센 달빛에서 돌 문이 방랑자보다 밝게 떴다.
                let k = 1.0 / ACTOR_LIFT;
                self.batch
                    .model(e, &self.pose, inst, Color::new(k, k, k, a), lights, 1.0);
            }
            None => {
                let mut b = g;
                b.max.y -= k * 4.5;
                shaded_box(b.center(), b.size(), Color::new(0.25, 0.2, 0.22, a), 0.6);
            }
        }
    }

    pub(super) fn stage_props(&mut self, run: &StageRun, w: &World, lights: &[Light]) {
        // 바닥 표시는 모두 땅 높낮이를 따라 접는다(0083 — 경사로·턱에서 묻히거나 뜨던 것).
        let fl = Drape {
            ground: w.ground.as_ref(),
        };
        // 문: 가리지 않을 때만 여기서(불투명). 옅어진 문은 옅은 지형과 함께 맨 나중에(`draw_gate`, 0091).
        if self.gate_fade >= 0.98 {
            self.draw_gate(run, lights, 1.0);
        }
        self.stone_tablets(run, lights, &fl);
        self.chests(run, w, lights, &fl);
        self.gate_lever(run, w, lights, &fl);
        way_down(run, w, &fl);
        self.residents(run, w, lights, &fl);
        light_shafts(run, w, &fl);
        way_up(run, w, &fl);
        chest_drops(run, w, &fl);
        hidden_finds(run, w, &fl);
        rest_rings(run, w, &fl);
        guide_beam(run, w, &fl);
        self.core_crystal(run, w, lights, &fl);
    }

    fn stone_tablets(&mut self, run: &StageRun, lights: &[Light], fl: &Drape) {
        // 새긴 돌(0129 C4): 조금 기운 선돌 + 새긴 줄 셋 + 밑동 이끼 — 곁에 서면 줄이 옅게 밝아진다(읽을 수 있다는 표시, 호박빛은 쓰지 않는다).
        for (i, st) in run.stones.iter().enumerate() {
            fl.shadow(*st, 0.45);
            let near = run.near_stone == Some(i);
            let yaw = (st.x * 1.7 + st.z * 0.9).sin() * 0.6;
            let lean = Quat::from_rotation_y(yaw) * Quat::from_rotation_x(0.08);
            let base = *st;
            // 모델(0139): 새긴 줄(`glow_carve`)은 스스로 빛나는 부위 — 곁에 서면 밝게, 아니면 돌보다 조금 어둡게.
            if let Some(e) = self.lib.get("stone_tablet") {
                e.model.pose(None, 0.0, &mut self.pose);
                let inst = Mat4::from_translation(base) * Mat4::from_quat(lean);
                let carve = if near { 1.15 } else { 0.42 };
                self.batch.model(e, &self.pose, inst, WHITE, lights, carve);
                continue;
            }
            let stone = Color::new(0.47, 0.48, 0.46, 1.0);
            shaded_box_rot(
                base + Vec3::Y * 0.55,
                vec3(0.62, 1.1, 0.2),
                stone,
                0.85,
                lean,
                base,
            );
            let carve = if near {
                Color::new(0.86, 0.84, 0.78, 1.0)
            } else {
                Color::new(0.3, 0.3, 0.3, 1.0)
            };
            for k in 0..3 {
                let y = 0.78 - k as f32 * 0.2;
                shaded_box_rot(
                    base + vec3(0.0, y, 0.105),
                    vec3(0.42 - k as f32 * 0.08, 0.04, 0.02),
                    carve,
                    1.0,
                    lean,
                    base,
                );
            }
            shaded_box(
                base + Vec3::Y * 0.05,
                vec3(0.75, 0.1, 0.32),
                Color::new(0.33, 0.42, 0.3, 1.0),
                0.85,
            );
        }
    }

    fn chests(&mut self, run: &StageRun, w: &World, lights: &[Light], fl: &Drape) {
        // 장신구 상자(0064 M3): 짙은 나무 궤 + 호박빛 자물쇠(닫힌 것만 숨 쉬듯), 열리면 뚜껑이 뒤로 젖혀지고 빛이 꺼진다.
        for (i, c) in run.chests.iter().enumerate() {
            let open = run.chest_open.get(i).copied().unwrap_or(false);
            fl.shadow(*c, 0.6);
            let pulse = 0.75 + 0.25 * (w.time * 2.4 + i as f32).sin();
            // 모델(0139): 열린 상자 = `open` 동작의 끝 자세 + 자물쇠 감춤, 닫힌 상자는 자물쇠(`glow_lock`)가 숨 쉰다.
            if let Some(e) = self.lib.get("chest") {
                let anim = e.pick(&["open"]);
                let at = if open { e.duration(anim) * 0.999 } else { 0.0 };
                let hide = open
                    .then(|| e.model.node("glow_lock"))
                    .flatten()
                    .map(|n| (n, Quat::IDENTITY, Vec3::ZERO));
                e.model.pose_with(anim, at, hide.as_slice(), &mut self.pose);
                self.batch.model(
                    e,
                    &self.pose,
                    Mat4::from_translation(*c),
                    WHITE,
                    lights,
                    0.55 + 0.6 * pulse,
                );
                if !open {
                    fl.ring(
                        *c,
                        0.9,
                        0.0,
                        360.0,
                        Color::new(1.0, 0.75, 0.35, 0.3 * pulse),
                    );
                }
                continue;
            }
            let body = Color::new(0.36, 0.25, 0.22, 1.0);
            let band = Color::new(0.55, 0.42, 0.3, 1.0);
            shaded_box(*c + Vec3::Y * 0.27, vec3(0.86, 0.54, 0.56), body, 0.85);
            shaded_box(*c + Vec3::Y * 0.27, vec3(0.9, 0.12, 0.6), band, 0.85);
            let lid_c = *c + Vec3::Y * 0.6;
            if open {
                let hinge = *c + vec3(0.0, 0.54, -0.28);
                shaded_box_rot(
                    lid_c,
                    vec3(0.88, 0.14, 0.58),
                    body,
                    0.85,
                    Quat::from_rotation_x(-1.9),
                    hinge,
                );
            } else {
                shaded_box(lid_c, vec3(0.88, 0.14, 0.58), body, 0.85);
                let lock = Color::new(1.0, 0.72 * pulse + 0.1, 0.3, 1.0);
                shaded_box(
                    *c + vec3(0.0, 0.46, 0.29),
                    vec3(0.14, 0.16, 0.04),
                    lock,
                    1.0,
                );
                fl.ring(
                    *c,
                    0.9,
                    0.0,
                    360.0,
                    Color::new(1.0, 0.75, 0.35, 0.3 * pulse),
                );
            }
        }
    }

    fn gate_lever(&mut self, run: &StageRun, w: &World, lights: &[Light], fl: &Drape) {
        // 문 장치(0113): 돌 기둥 + 쇠 손잡이. 당기기 전엔 손잡이가 위로 서 있고 호박빛이 숨 쉰다(닫힌 상자와 같은 "할 수 있다" 빛),
        // 당긴 뒤엔 손잡이가 내려가고 빛이 꺼진다.
        if let Some(l) = run.lever {
            fl.shadow(l, 0.5);
            let pulse = 0.75 + 0.25 * (w.time * 2.4).sin();
            // 모델(0139): 당긴 뒤 = `pull` 동작의 끝 자세 + 쥘 곳(`glow_grip`) 감춤.
            if let Some(e) = self.lib.get("lever") {
                let anim = e.pick(&["pull"]);
                let at = if run.gate_opened {
                    e.duration(anim) * 0.999
                } else {
                    0.0
                };
                let hide = run
                    .gate_opened
                    .then(|| e.model.node("glow_grip"))
                    .flatten()
                    .map(|n| (n, Quat::IDENTITY, Vec3::ZERO));
                e.model.pose_with(anim, at, hide.as_slice(), &mut self.pose);
                self.batch.model(
                    e,
                    &self.pose,
                    Mat4::from_translation(l),
                    WHITE,
                    lights,
                    0.55 + 0.6 * pulse,
                );
            } else {
                let stone = Color::new(0.42, 0.4, 0.44, 1.0);
                let iron = Color::new(0.3, 0.28, 0.27, 1.0);
                shaded_box(l + Vec3::Y * 0.45, vec3(0.5, 0.9, 0.5), stone, 0.85);
                let hinge = l + vec3(0.0, 0.8, 0.27);
                let tilt = if run.gate_opened { 1.0 } else { -0.5 };
                shaded_box_rot(
                    hinge + vec3(0.0, 0.32, 0.0),
                    vec3(0.1, 0.64, 0.1),
                    iron,
                    0.9,
                    Quat::from_rotation_x(tilt),
                    hinge,
                );
                if !run.gate_opened {
                    let grip = Color::new(1.0, 0.72 * pulse + 0.1, 0.3, 1.0);
                    let top = hinge + Quat::from_rotation_x(tilt) * vec3(0.0, 0.66, 0.0);
                    shaded_box(top, vec3(0.16, 0.12, 0.16), grip, 1.0);
                }
            }
            if !run.gate_opened {
                fl.ring(l, 0.9, 0.0, 360.0, Color::new(1.0, 0.75, 0.35, 0.3 * pulse));
            }
        }
    }

    fn residents(&mut self, run: &StageRun, w: &World, lights: &[Light], fl: &Drape) {
        // 남은 주민(0122): 모델(0132, `Resident::model`)이 없으면 상자 배우 — 둥근 몸 + 큰 머리 + 귀 둘(작은 동물), 바라보는 쪽으로 코.
        // 곁의 생활 물건이 재방문 변화를 보인다 — 아궁이 불빛이 살아남 · 항아리가 바로 섬 · 천막이 꿰매짐.
        for (k, &(i, at)) in run.residents.iter().enumerate() {
            let r = &crate::residents::RESIDENTS[i];
            let face = run.resident_face.get(k).copied().unwrap_or(0.0);
            let changed = run.resident_changed.get(k).copied().unwrap_or(false);
            let q = Quat::from_rotation_y(face);
            let breath = (w.time * 1.3 + at.x).sin() * 0.02;
            fl.shadow(at, 0.5);
            // 모델(0132)이 있으면: 방랑자가 멀면 제 일(`work`), 돌아볼 거리 안이면 돌아본 자세(`look`). 자리마다 박자를 어긋낸다.
            if let Some(e) = self.lib.get(r.model) {
                let near = vec2(w.player.pos.x - at.x, w.player.pos.z - at.z).length()
                    <= crate::residents::GLANCE_RANGE;
                let anim = e.pick(if near {
                    &["look", "idle"]
                } else {
                    &["work", "idle"]
                });
                let clock = w.time + at.x * 0.37 + at.z * 0.61;
                e.model.pose(
                    anim,
                    clock.rem_euclid(e.duration(anim).max(1e-3)),
                    &mut self.pose,
                );
                let inst = Mat4::from_translation(at) * Mat4::from_rotation_y(face);
                self.batch.model(e, &self.pose, inst, WHITE, lights, 0.0);
            } else {
                shaded_box_rot(
                    at + Vec3::Y * 0.35,
                    vec3(0.6, 0.7 + breath, 0.5),
                    r.body,
                    0.85,
                    q,
                    at,
                );
                let head = at + Vec3::Y * (0.95 + breath);
                shaded_box_rot(head, vec3(0.62, 0.52, 0.56), r.body, 0.9, q, at);
                for side in [-1.0f32, 1.0] {
                    shaded_box_rot(
                        head + vec3(0.2 * side, 0.32, 0.0),
                        vec3(0.14, 0.2, 0.1),
                        r.body,
                        0.8,
                        q,
                        at,
                    );
                }
                shaded_box_rot(
                    head + vec3(0.0, -0.05, 0.3),
                    vec3(0.14, 0.1, 0.08),
                    Color::new(0.25, 0.2, 0.2, 1.0),
                    1.0,
                    q,
                    at,
                );
            }
            // 생활 물건은 제 일 쪽(옆)에.
            let spot = at + vec3(0.95, 0.0, 0.0);
            match r.work {
                crate::residents::Work::Fire => {
                    let glow = if changed { 1.5 } else { 0.8 };
                    let flick = 0.85 + 0.15 * (w.time * 7.0 + at.z).sin();
                    fl.sector(
                        spot,
                        glow,
                        0.0,
                        360.0,
                        Color::new(1.0, 0.6, 0.25, 0.22 * flick),
                    );
                    // 모델(0142): 돌을 두른 화덕 + 불꽃(뼈 `fire`, 밑동 피벗) — 꺼져 가는 불은 낮고 가늘게, 다시 찾으면 살아난 불.
                    if let Some(e) = self.lib.get("work_fire") {
                        let size = if changed { 1.0 } else { 0.5 };
                        let lick = (w.time * 11.0 + at.x).sin() * 0.06;
                        let grow = e.model.node("fire").map(|n| {
                            (
                                n,
                                Quat::from_rotation_y(w.time * 0.6),
                                vec3(
                                    size * (1.0 - lick),
                                    size * (flick + lick) * 1.05,
                                    size * (1.0 - lick),
                                ),
                            )
                        });
                        e.model
                            .pose_with(None, 0.0, grow.as_slice(), &mut self.pose);
                        fl.shadow(spot, 0.5);
                        self.batch.model(
                            e,
                            &self.pose,
                            Mat4::from_translation(spot),
                            WHITE,
                            lights,
                            if changed { 1.0 } else { 0.75 } * flick,
                        );
                        continue;
                    }
                    shaded_box(
                        spot + Vec3::Y * 0.15,
                        vec3(0.7, 0.3, 0.6),
                        Color::new(0.35, 0.32, 0.33, 1.0),
                        0.85,
                    );
                    let fh = if changed { 0.3 } else { 0.16 };
                    shaded_box(
                        spot + Vec3::Y * (0.3 + fh * 0.5),
                        vec3(0.22, fh * flick, 0.22),
                        Color::new(1.0, 0.65, 0.3, 1.0),
                        1.0,
                    );
                }
                crate::residents::Work::Water => {
                    // 모델(0142): 질항아리 — 엎어진 채 누워 물이 샌 자리 / 다시 찾으면 바로 서고 아가리에 물빛.
                    if let Some(e) = self.lib.get("work_jar") {
                        e.model.pose(None, 0.0, &mut self.pose);
                        let inst = if changed {
                            fl.sector(
                                spot + Vec3::Y * 0.56,
                                0.1,
                                0.0,
                                360.0,
                                Color::new(0.5, 0.7, 0.9, 0.5),
                            );
                            Mat4::from_translation(spot)
                        } else {
                            fl.sector(
                                spot + vec3(-0.62, 0.0, 0.05),
                                0.42,
                                0.0,
                                360.0,
                                Color::new(0.4, 0.55, 0.75, 0.25),
                            );
                            Mat4::from_translation(spot + Vec3::Y * JAR_LIE_LIFT)
                                * Mat4::from_rotation_y(0.3)
                                * Mat4::from_rotation_z(JAR_LIE)
                        };
                        fl.shadow(spot, 0.34);
                        self.batch.model(e, &self.pose, inst, WHITE, lights, 0.0);
                        continue;
                    }
                    let tilt = if changed { 0.0 } else { 0.55 };
                    let jar = Color::new(0.55, 0.4, 0.32, 1.0);
                    shaded_box_rot(
                        spot + Vec3::Y * 0.28,
                        vec3(0.38, 0.55, 0.38),
                        jar,
                        0.85,
                        Quat::from_rotation_z(tilt),
                        spot,
                    );
                    if changed {
                        fl.sector(spot, 0.3, 0.0, 360.0, Color::new(0.5, 0.7, 0.9, 0.35));
                    } else {
                        fl.sector(
                            spot + vec3(-0.45, 0.0, 0.0),
                            0.45,
                            0.0,
                            360.0,
                            Color::new(0.4, 0.55, 0.75, 0.25),
                        );
                    }
                }
                crate::residents::Work::Cloth => {
                    // 모델(0142): 장대 둘 + 줄 + 천 두 벌 — 찢어져 늘어진 두 조각(`torn`) / 다시 찾으면 꿰맨 한 장(`whole`). 하나만 보인다.
                    if let Some(e) = self.lib.get("work_tent") {
                        let hide = e
                            .model
                            .node(if changed { "torn" } else { "whole" })
                            .map(|n| (n, Quat::IDENTITY, Vec3::ZERO));
                        e.model
                            .pose_with(None, 0.0, hide.as_slice(), &mut self.pose);
                        self.batch.model(
                            e,
                            &self.pose,
                            Mat4::from_translation(spot),
                            WHITE,
                            lights,
                            0.0,
                        );
                        continue;
                    }
                    let cloth = Color::new(0.7, 0.55, 0.45, 1.0);
                    let pole = Color::new(0.35, 0.28, 0.22, 1.0);
                    for dx in [-0.55f32, 0.55] {
                        shaded_box(spot + vec3(dx, 0.6, 0.0), vec3(0.08, 1.2, 0.08), pole, 0.85);
                    }
                    if changed {
                        shaded_box(spot + Vec3::Y * 0.95, vec3(1.1, 0.5, 0.04), cloth, 0.9);
                    } else {
                        shaded_box(
                            spot + vec3(-0.3, 1.05, 0.0),
                            vec3(0.5, 0.3, 0.04),
                            cloth,
                            0.9,
                        );
                        shaded_box(
                            spot + vec3(0.32, 0.88, 0.0),
                            vec3(0.45, 0.22, 0.04),
                            cloth,
                            0.9,
                        );
                    }
                }
            }
        }
    }

    /// 핵(문지기가 남긴 것): 떠서 돈다 + 발밑 고리.
    fn core_crystal(&mut self, run: &StageRun, w: &World, lights: &[Light], fl: &Drape) {
        if let (Some(core), Phase::Core) = (run.core, run.phase) {
            let bob = (w.time * 2.0).sin() * 0.15;
            let p = core + Vec3::Y * (0.9 + bob);
            match self.lib.get("core_crystal") {
                Some(e) => {
                    let a = e.pick(&["pulse", "idle"]);
                    e.model.pose(a, w.time, &mut self.pose);
                    let inst = Mat4::from_translation(p) * Mat4::from_rotation_y(w.time * 1.2);
                    self.batch.model(e, &self.pose, inst, WHITE, lights, 1.0);
                }
                None => shaded_box(p, Vec3::splat(0.45), Color::new(1.0, 0.7, 0.3, 1.0), 0.2),
            }
            fl.ring(
                core,
                1.0 + 0.15 * (w.time * 3.0).sin(),
                0.0,
                360.0,
                Color::new(1.0, 0.75, 0.35, 0.6),
            );
        }
    }
}

fn way_down(run: &StageRun, w: &World, fl: &Drape) {
    // 지역 잇기(0118): 열린 바닥 길은 아래로 빠지는 어두운 구멍(겹 원판 — 가장자리 흙빛 → 안쪽 검정) + 바닥에서 올라오는 옅은 남보라 빛,
    // 오르는 길은 마을 어귀 바닥의 빛 웅덩이(위에서 새는 빛 — 갈 곳 빛기둥과 같은 호박색, 낮게).
    if let (Some(d), Phase::Down) = (run.down_at, run.phase) {
        // 겹 원판은 바닥보다 조금 띄운다(같은 높이면 석판 면에 묻혔다 — 0118 헤드리스).
        let d = d + Vec3::Y * 0.05;
        fl.sector(d, 1.5, 0.0, 360.0, Color::new(0.24, 0.18, 0.16, 0.95));
        fl.sector(d, 1.15, 0.0, 360.0, Color::new(0.08, 0.06, 0.1, 1.0));
        fl.sector(d, 0.7, 0.0, 360.0, Color::new(0.02, 0.01, 0.04, 1.0));
        let pulse = 0.5 + 0.5 * (w.time * 1.6).sin();
        beam(
            d,
            0.8,
            1.6,
            Color::new(0.55, 0.45, 0.9, 0.08 + 0.06 * pulse),
        );
    }
}

fn light_shafts(run: &StageRun, w: &World, fl: &Drape) {
    // 천장 틈 빛줄기(0118 T2, KB conveying-descent "방 안의 위"): 위는 좁고 아래는 넓은 비스듬한 기둥(위가 밝다) + 바닥 빛 웅덩이.
    // 기울기는 달빛 쪽 — 땅속의 옅은 달빛과 같은 빛이 틈으로 든다.
    let lean = vec3(-LIGHT_DIR.x, 0.0, -LIGHT_DIR.z).normalize_or_zero() * 2.2;
    for s in run.level.marks_of(crate::stage::SHAFT_GLYPH) {
        let breath = 0.85 + 0.15 * (w.time * 0.7 + s.x * 0.3).sin();
        fl.sector(
            s,
            1.7,
            0.0,
            360.0,
            Color::new(0.75, 0.78, 0.95, 0.14 * breath),
        );
        fl.sector(
            s,
            1.0,
            0.0,
            360.0,
            Color::new(0.8, 0.82, 1.0, 0.12 * breath),
        );
        shaft(
            s,
            s + lean + Vec3::Y * 9.0,
            1.5,
            0.45,
            Color::new(0.75, 0.78, 0.95, 0.05 * breath),
            Color::new(0.8, 0.82, 1.0, 0.2 * breath),
        );
    }
}

/// 오르는 길(0118): 마을 어귀 바닥의 빛 웅덩이 — 위에서 새는 빛(갈 곳 빛기둥과 같은 호박색, 낮게).
fn way_up(run: &StageRun, w: &World, fl: &Drape) {
    if let Some(u) = run.up_at {
        let pulse = 0.5 + 0.5 * (w.time * 1.2).sin();
        fl.sector(u, 1.3, 0.0, 360.0, with_a(GUIDE_COLOR, 0.12 + 0.06 * pulse));
        beam(u, 0.9, 4.0, with_a(GUIDE_COLOR, 0.06 + 0.04 * pulse));
        fl.ring(u, 1.3, 0.0, 360.0, with_a(GUIDE_COLOR, 0.45));
    }
}

fn chest_drops(run: &StageRun, w: &World, fl: &Drape) {
    // 떨어진 아이템(0131): 상자에서 포물선으로 튀어나와 내려앉고, 내려앉은 뒤 천천히 돌며 떠 있는 흰 마름모 + 흰 빛기둥.
    for (j, (p, chest, age)) in run.drops.iter().enumerate() {
        let k = (age / crate::stage::DROP_SETTLE).clamp(0.0, 1.0);
        let from = run.chests.get(*chest).copied().unwrap_or(*p) + Vec3::Y * 0.7;
        let to = *p + Vec3::Y * 0.45;
        let arc = from.lerp(to, k) + Vec3::Y * (4.0 * k * (1.0 - k)) * 1.0;
        let bob = if k >= 1.0 {
            0.06 * (w.time * 2.4 + j as f32).sin()
        } else {
            0.0
        };
        let c = arc + Vec3::Y * bob;
        fl.shadow(*p, 0.22);
        let rot = Quat::from_rotation_y(w.time * 1.4 + j as f32)
            * Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)
            * Quat::from_rotation_x(std::f32::consts::FRAC_PI_4);
        // 몸은 결정(팔면체 — 0142: 정육면체는 디버그 상자로 읽혔다) + 카메라를 보는 둥근 빛무리.
        // 빛기둥 앞에 둔다(`loot_front`) — 기둥 속에 두면 흰 기둥에 흰 몸이 묻혔다.
        let c = loot_front(c);
        loot_body(
            c,
            &octa_faces(vec2(0.15, 0.15), 0.23, 0.23),
            LOOT_CHEST,
            rot,
        );
        glow_disc(c, 0.3, with_a(LOOT_CHEST, 0.3));
        if k >= 1.0 {
            loot_beam(
                fl,
                *p,
                LOOT_CHEST,
                loot_fade(*p, w.player.pos),
                w.time,
                7.0 + j as f32,
            );
        }
    }
}

fn hidden_finds(run: &StageRun, w: &World, fl: &Drape) {
    // 숨은 물건(0105): 흔적 자리의 작은 빛 — 등잔 +1은 금빛, 한 모금 +1은 민트(0037 생명 색). 살짝 떠서 숨 쉬고 발밑에 옅은 테.
    // 생김새는 성소의 종과 닮지 않게(lore — 울리는 종은 보이지 않는다): 둥근 덩어리 하나.
    for (i, (p, kind, _)) in run.finds.iter().enumerate() {
        if run.found.get(i).copied().unwrap_or(true) {
            continue;
        }
        // 떨어진 아이템의 빛기둥(0131): 종류 색, 가까이 와야 서서히 — 뒤지는 맛은 남긴다.
        let beam_col = match kind {
            crate::stage::Find::Lamp => LOOT_LAMP,
            crate::stage::Find::Sip => LOOT_SIP,
            crate::stage::Find::Shard => LOOT_SHARD,
            crate::stage::Find::Garb => LOOT_GARB,
        };
        loot_beam(
            fl,
            *p,
            beam_col,
            loot_fade(*p, w.player.pos),
            w.time,
            i as f32,
        );
        let k = 0.5 + 0.5 * (w.time * 2.2 + i as f32 * 1.7).sin();
        // 종소리 조각(0106): 가늘고 긴 크림빛 조각이 천천히 돌며 떤다 — 덩어리(등잔·한 모금)와 모양부터 다르게, 종 꼴은 아니게.
        if *kind == crate::stage::Find::Shard {
            let col = Color::new(0.96, 0.92, 0.82, 1.0);
            let c = *p + Vec3::Y * (0.55 + 0.06 * k);
            fl.shadow(*p, 0.2);
            let tremble = (w.time * 23.0 + i as f32).sin() * 0.04;
            // 깨진 조각(0142): 한쪽이 길고 납작한 결정 — 위가 길고 아래가 짧아 종 꼴도 상자도 아니다.
            let c = loot_front(c);
            // 가는 꼴이라 테는 배율이 아니라 두께로(같은 배율이면 1px도 안 됐다).
            loot_rimmed(
                c,
                &octa_faces(vec2(0.13, 0.085), 0.34, 0.16),
                &octa_faces(vec2(0.18, 0.135), 0.4, 0.22),
                col,
                Quat::from_rotation_y(w.time * 0.9 + i as f32)
                    * Quat::from_rotation_z(0.35 + tremble),
            );
            glow_disc(c, 0.26, with_a(col, 0.14 + 0.1 * k));
            fl.ring(
                *p,
                0.5 + 0.12 * k,
                0.0,
                360.0,
                with_a(Color::new(0.95, 0.81, 0.49, 1.0), 0.2 + 0.25 * k),
            );
            continue;
        }
        // 덧옷(0176): 개어 놓은 천 — 납작하고 넓은 둥근 덩이가 바닥 가까이에서 천천히 돈다(빛나는 덩이·조각과 키부터 다르게).
        if *kind == crate::stage::Find::Garb {
            let col = Color::new(0.84, 0.78, 0.92, 1.0);
            let c = loot_front(*p + Vec3::Y * (0.32 + 0.04 * k));
            fl.shadow(*p, 0.32);
            let folded: Vec<[Vec3; 3]> = ball_faces(0.2)
                .into_iter()
                .map(|t| t.map(|v| vec3(v.x * 1.35, v.y * 0.42, v.z)))
                .collect();
            loot_body(
                c,
                &folded,
                col,
                Quat::from_rotation_y(w.time * 0.5 + i as f32),
            );
            glow_disc(c, 0.3, with_a(col, 0.18 + 0.12 * k));
            fl.ring(
                *p,
                0.62 + 0.12 * k,
                0.0,
                360.0,
                with_a(col, 0.22 + 0.22 * k),
            );
            continue;
        }
        let col = match kind {
            crate::stage::Find::Lamp => Color::new(0.95, 0.81, 0.49, 1.0),
            _ => Color::new(0.65, 0.86, 0.78, 1.0),
        };
        let c = *p + Vec3::Y * (0.45 + 0.08 * k);
        fl.shadow(*p, 0.25);
        // 둥근 덩이(0142, 0037 둥근 것 = 산 것) + 숨 쉬는 빛무리.
        let c = loot_front(c);
        loot_body(
            c,
            &ball_faces(0.17),
            col,
            Quat::from_rotation_y(w.time * 0.8 + i as f32),
        );
        glow_disc(c, 0.3 + 0.04 * k, with_a(col, 0.25 + 0.2 * k));
        fl.ring(*p, 0.6 + 0.15 * k, 0.0, 360.0, with_a(col, 0.25 + 0.25 * k));
    }
}

fn rest_rings(run: &StageRun, w: &World, fl: &Drape) {
    // 쉼터(0032): 발밑 테 — 아직 안 쉰 곳은 옅게, 다시 일어설 곳은 금빛으로 숨 쉬듯. 쉬는 순간 빛이 퍼진다.
    for (i, r) in run.rests.iter().enumerate() {
        let home = run.rest_at.is_some_and(|(j, _)| j == i);
        // 잔불로 살 것이 있으면(0064 M4) 쉼터마다 호박빛 테가 하나 더 숨 쉰다 — 쓸 곳이 보이게.
        if run.shop_ready {
            let k = 0.5 + 0.5 * (w.time * 3.0 + i as f32).sin();
            fl.ring(
                *r,
                1.5 + 0.3 * k,
                0.0,
                360.0,
                Color::new(1.0, 0.7, 0.3, 0.25 + 0.3 * k),
            );
        }
        let col = if home {
            Color::new(1.0, 0.8, 0.4, 0.55 + 0.25 * (w.time * 2.0).sin())
        } else {
            Color::new(0.9, 0.7, 0.45, 0.25)
        };
        fl.ring(*r, 1.0, 0.0, 360.0, col);
        if home && run.since_rest < 0.8 {
            let k = run.since_rest / 0.8;
            fl.ring(
                *r + Vec3::Y * (0.6 * k),
                1.0 + 2.5 * k,
                0.0,
                360.0,
                Color::new(1.0, 0.85, 0.5, 0.9 * (1.0 - k)),
            );
        }
    }
}

fn guide_beam(run: &StageRun, w: &World, fl: &Drape) {
    // 갈 곳의 빛기둥(0060 W1): 멀리서도 보이는 세로 기둥 둘(바깥 옅게·안쪽 밝게) + 발밑 고리, 숨 쉬듯.
    // 열린 바닥 길(0118)은 가까이 오면 기둥을 걷는다 — 밝은 기둥 밑동이 어두운 구멍을 덮었다(헤드리스). 멀리서는 그대로 길 안내.
    let near_hole = run.phase == Phase::Down
        && run
            .goal()
            .is_some_and(|g| vec2(g.x - w.player.pos.x, g.z - w.player.pos.z).length() < 8.0);
    if let Some(g) = run.goal().filter(|_| !near_hole) {
        let pulse = 0.5 + 0.5 * (w.time * GUIDE_PULSE).sin();
        beam(
            g,
            GUIDE_BEAM_R,
            GUIDE_BEAM_H,
            with_a(GUIDE_COLOR, 0.16 + 0.08 * pulse),
        );
        beam(
            g,
            GUIDE_BEAM_R * 0.45,
            GUIDE_BEAM_H * 0.8,
            with_a(GUIDE_COLOR, 0.3 + 0.15 * pulse),
        );
        fl.ring(
            g,
            GUIDE_BEAM_R + 0.4 + 0.3 * pulse,
            0.0,
            360.0,
            with_a(GUIDE_COLOR, 0.7),
        );
    }
}

/// 갈 곳 빛(0060 W1): 색(화로와 같은 호박색 — 스테이지 내내 "이 빛 = 갈 길"), 숨 쉬는 빠르기(rad/s), 빛기둥 반지름·높이(m), 곁 모델을 물들이는 빛 반지름(m).
pub(super) const GUIDE_COLOR: Color = Color::new(1.0, 0.68, 0.3, 1.0);

pub(super) const GUIDE_PULSE: f32 = 2.4;

const GUIDE_BEAM_R: f32 = 1.4;
const GUIDE_BEAM_H: f32 = 9.0;

pub(super) const GUIDE_LIGHT_R: f32 = 8.0;

/// 문이 다 열리는 초(스테이지가 `gate_open`을 이만큼까지 올린다).
pub(super) const GATE_TIME: f32 = 1.5;

/// 문 모델의 문틀이 막는 상자보다 앞뒤로 더 나온 만큼(m, `gate.glb` ±0.62 − 상자 ±0.4).
const GATE_FRAME_EXTRA: f32 = 0.22;

/// 닫힌(또는 열리는 중인) 스테이지 문의 지금 상자 — 다 열렸으면 없음(0091). 열리며 가라앉는 만큼 위가 낮아진다.
pub(super) fn gate_box(run: &StageRun) -> Option<Aabb> {
    let (g, _) = run.level.gate?;
    let k = (run.gate_open / GATE_TIME).clamp(0.0, 1.0);
    (k < 1.0).then(|| {
        let mut b = g;
        b.max.y -= k * 4.5;
        // 모델의 문틀은 막는 상자보다 두껍다(±0.62m 대 ±0.4m) — 가림 판정은 보이는 두께로.
        b.min.z -= GATE_FRAME_EXTRA;
        b.max.z += GATE_FRAME_EXTRA;
        b
    })
}

/// 떨어진 아이템의 빛기둥 색(0131 — 종류마다, 갈 곳의 호박빛 `GUIDE_COLOR`는 쓰지 않는다: KB wayfinding "그 색 = 갈 길").
const LOOT_LAMP: Color = Color::new(0.98, 0.9, 0.62, 1.0);
const LOOT_SIP: Color = Color::new(0.62, 0.92, 0.8, 1.0);
const LOOT_SHARD: Color = Color::new(0.96, 0.94, 0.86, 1.0);
/// 덧옷(0176): 옅은 라벤더 — 금빛(등잔)·민트(한 모금)·크림(조각)과 갈린다.
const LOOT_GARB: Color = Color::new(0.86, 0.8, 0.98, 1.0);
const LOOT_CHEST: Color = Color::new(0.9, 0.93, 1.0, 1.0);
/// 빛기둥이 다 보이는 거리·사라지는 거리(m) — 가까이 와야 서서히(사장 2026-11-09 "가까이서만 서서히").
const LOOT_NEAR: f32 = 10.0;
const LOOT_FAR: f32 = 14.0;

/// 방랑자와의 거리로 빛기둥의 세기 0..1.
fn loot_fade(at: Vec3, player: Vec3) -> f32 {
    let d = vec2(at.x - player.x, at.z - player.z).length();
    ((LOOT_FAR - d) / (LOOT_FAR - LOOT_NEAR)).clamp(0.0, 1.0)
}

/// 떨어진 아이템의 빛기둥(0131, 디아블로식 — 갈 곳 기둥보다 훨씬 가늘고 낮게): 안쪽 밝은 심 + 바깥 옅은 껍질 + 위로 오르는 반짝임 셋 + 발밑 테.
fn loot_beam(fl: &Drape, p: Vec3, col: Color, fade: f32, time: f32, seed: f32) {
    if fade <= 0.01 {
        return;
    }
    let pulse = 0.85 + 0.15 * (time * 2.0 + seed).sin();
    beam(p, 0.1, 2.6, with_a(col, 0.55 * fade * pulse));
    beam(p, 0.26, 2.0, with_a(col, 0.16 * fade));
    fl.ring(p, 0.42, 0.0, 360.0, with_a(col, 0.45 * fade));
    for k in 0..3 {
        let u = (time * 0.55 + k as f32 / 3.0 + seed * 0.37).fract();
        let a = (1.0 - u) * fade;
        let x = (seed * 1.3 + k as f32 * 2.1 + time).sin() * 0.08;
        gem(p + vec3(x, 0.2 + u * 2.3, -x), 0.09, with_a(col, a));
    }
}

/// 줍는 것의 몸 음영(0142): 스스로 빛나는 것이라 그늘진 면도 조금은 남긴다(면이 읽힐 만큼은 진하게).
const LOOT_SHADE: f32 = 0.5;
/// 줍는 것의 몸을 빛기둥 앞으로 당기는 거리(m — 기둥 바깥 반지름보다 조금 더)·어두운 테의 두께(몸의 배율)·테를 뒤로 미는 거리.
const LOOT_FRONT: f32 = 0.3;
const LOOT_RIM: f32 = 1.35;
const LOOT_RIM_BACK: f32 = 0.06;

/// 빛기둥 앞(카메라 쪽)의 자리 — 줍는 것의 몸이 기둥의 빛에 묻히지 않게.
fn loot_front(p: Vec3) -> Vec3 {
    p + (look().eye - p).normalize_or_zero() * LOOT_FRONT
}

/// 줍는 것의 몸(0142): 면마다 음영 + 뒤에 깔린 어두운 같은 꼴(조금 크게 — 밝은 빛기둥·바닥 위에서 테가 된다).
fn loot_body(c: Vec3, faces: &[[Vec3; 3]], col: Color, rot: Quat) {
    let grown: Vec<[Vec3; 3]> = faces.iter().map(|t| t.map(|p| p * LOOT_RIM)).collect();
    loot_rimmed(c, faces, &grown, col, rot);
}

/// `loot_body`의 속: 테의 꼴(`rim`)을 따로 받는다.
fn loot_rimmed(c: Vec3, faces: &[[Vec3; 3]], rim: &[[Vec3; 3]], col: Color, rot: Quat) {
    let back = c - (look().eye - c).normalize_or_zero() * LOOT_RIM_BACK;
    let dark = Color::new(col.r * 0.2, col.g * 0.17, col.b * 0.28, 1.0);
    shaded_tris(back, rim, dark, 0.0, rot);
    shaded_tris(c, faces, col, LOOT_SHADE, rot);
}
/// 엎어진 항아리(0142): 눕힌 각(라디안)·배가 땅에 닿게 올리는 높이(m).
const JAR_LIE: f32 = 1.38;
const JAR_LIE_LIFT: f32 = 0.2;
