//! 아이템 그림(0068)·장비 창 인형(0067)·도감 생물(0130): 작은 렌더 타깃 + 고정 카메라.
use super::*;

impl Renderer {
    /// 아이템 아이콘(0068): 처음 부를 때 한 번 찍어 두고(정한 각도), 다음부터는 그 그림을 `dest`에 붙인다.
    /// 모델·부위가 없으면 아무것도 그리지 않는다(칸은 비어 보인다).
    pub fn draw_icon(&mut self, item: crate::item_view::Item, dest: Rect) {
        let key = crate::item_view::key(item);
        let rt = match self.icons.get(key) {
            Some(rt) => rt.clone(),
            None => {
                let rt = make_rt(ICON_PX, ICON_PX);
                if !self.render_item(item, &rt, ICON_PX, 0.0) {
                    return;
                }
                self.icons.insert(key, rt.clone());
                rt
            }
        };
        blit(&rt, dest);
    }

    /// 고른 아이템 하나를 천천히 돌려 크게(0068): 타깃 하나를 매 프레임 다시 그린다(인형과 같은 값싼 패스).
    pub fn draw_item(&mut self, item: crate::item_view::Item, dest: Rect, spin: f32) {
        let rt = self
            .item_rt
            .get_or_insert_with(|| make_rt(ITEM_PX, ITEM_PX))
            .clone();
        if self.render_item(item, &rt, ITEM_PX, spin) {
            blit(&rt, dest);
        }
    }

    /// 아이템 부위 하나만(같은 모델의 다른 부위는 크기 0) 경계 상자 가운데에 놓고 표의 각도 + `spin`으로 돌려, 틀에 꽉 차게 그린다.
    /// 게임과 같은 툰 빛·외곽선(`Batch::model`). 그렸으면 참. 화면 패스 중에 불러도 기본 카메라로 돌려놓는다.
    fn render_item(
        &mut self,
        item: crate::item_view::Item,
        rt: &RenderTarget,
        px: u32,
        spin: f32,
    ) -> bool {
        let v = crate::item_view::view(item);
        let Some(e) = self.lib.get(v.model) else {
            return false;
        };
        let Some(i) = e.model.node(v.node) else {
            return false;
        };
        let Some((c, rad)) = e.model.rest_bounds_under(i) else {
            return false;
        };
        let hide: Vec<(usize, Quat, Vec3)> = e
            .model
            .nodes
            .iter()
            .enumerate()
            .filter(|(n, node)| node.mesh.is_some() && !e.model.is_under(*n, i))
            .map(|(n, _)| (n, Quat::IDENTITY, Vec3::ZERO))
            .collect();
        e.model.pose_with(None, 0.0, &hide, &mut self.pose);
        // 표의 각도로 놓은 뒤 화면 세로축으로 `spin`(모델 축으로 돌리면 세운 칼이 바퀴처럼 돈다).
        let rot = Quat::from_rotation_y(spin)
            * Quat::from_rotation_z(v.roll.to_radians())
            * Quat::from_rotation_x(v.pitch.to_radians())
            * Quat::from_rotation_y(v.yaw.to_radians());
        let inst = Mat4::from_quat(rot) * Mat4::from_translation(-c);
        let fovy: f32 = 0.5;
        let cam = Camera3D {
            position: vec3(0.0, 0.0, rad.max(0.01) / (fovy * 0.5).sin() * 1.02),
            target: Vec3::ZERO,
            up: Vec3::Y,
            fovy,
            aspect: Some(1.0),
            projection: Projection::Perspective,
            render_target: Some(rt.clone()),
            ..Default::default()
        };
        let saved = look();
        set_look(Look {
            eye: cam.position,
            fovy,
            rows: px as f32,
            ..saved
        });
        set_camera(&cam);
        clear_background(Color::new(0.0, 0.0, 0.0, 0.0));
        let fire = crate::stage::Light {
            pos: cam.position + vec3(-1.5, 1.5, 0.0) * rad.max(0.1),
            color: Color::new(1.0, 0.62, 0.25, 1.0),
            radius: rad * 8.0 + 2.0,
            shaft: false,
        };
        self.batch.model(e, &self.pose, inst, WHITE, &[fire], 1.0);
        self.batch.flush();
        set_look(saved);
        set_default_camera();
        true
    }

    /// 인형·도감 칸의 타깃(0190): 화면의 칸 `dest`를 게임 화면과 같은 도트 크기(한 칸 = 화면 배율 px)로 나눈 크기 —
    /// 내부 해상도 줄 수(`render.rows`)를 바꾸면 인형의 도트도 같이 바뀐다. 남는 px은 칸 가운데로 모은다. 돌려주는 것: 타깃·붙일 자리·가로세로 비.
    fn fit_doll_rt(&mut self, creature: bool, dest: Rect) -> (RenderTarget, Rect, f32) {
        let px = self.rt_size.2.max(1) as f32;
        let (tw, th) = (
            ((dest.w / px).floor() as u32).max(DOLL_W / 4),
            ((dest.h / px).floor() as u32).max(DOLL_H / 4),
        );
        let slot = if creature {
            &mut self.creature_rt
        } else {
            &mut self.doll_rt
        };
        if slot
            .as_ref()
            .is_none_or(|r| (r.texture.width() as u32, r.texture.height() as u32) != (tw, th))
        {
            *slot = Some(make_rt(tw, th));
        }
        let rt = slot.clone().unwrap();
        let (w, h) = (tw as f32 * px, th as f32 * px);
        let at = Rect::new(
            (dest.x + (dest.w - w) * 0.5).round(),
            (dest.y + (dest.h - h) * 0.5).round(),
            w,
            h,
        );
        (rt, at, tw as f32 / th as f32)
    }

    /// 장비 창의 인형(0067 D, 디아블로식): 방랑자만(든 무기만 보이게) 고정 카메라로 작은 타깃에 그리고, 화면 `dest`에 붙인다.
    /// `spin` = 몸 방향(라디안), `time` = 숨쉬기 동작 시간. 모델이 없으면 아무것도 그리지 않는다(창이 초상 없이 그대로).
    /// 화면 패스 중에 불러도 된다: 카메라를 바꿨다가 기본 카메라로 돌려놓는다.
    pub fn draw_doll(&mut self, weapon: crate::weapons::Id, dest: Rect, spin: f32, time: f32) {
        let (rt, dest, aspect) = self.fit_doll_rt(false, dest);
        let Some(e) = self.lib.get("wanderer") else {
            return;
        };
        // 든 무기만 보인다(0051): 다른 무기 부위는 크기 0.
        let mut hide: Vec<(usize, Quat, Vec3)> = Vec::new();
        for w in crate::weapons::WEAPONS.iter() {
            if w.id != weapon {
                if let Some(n) = e.model.node(w.node) {
                    hide.push((n, Quat::IDENTITY, Vec3::ZERO));
                }
            }
        }
        e.model
            .pose_with(e.pick(&["idle"]), time, &hide, &mut self.pose);
        // 키 1.8 몸이 칸에 꽉 차게: 가슴 높이를 보고 조금 위에서.
        let cam = Camera3D {
            position: vec3(0.0, 1.35, 4.6),
            target: vec3(0.0, 0.85, 0.0),
            up: Vec3::Y,
            fovy: 0.48,
            aspect: Some(aspect),
            projection: Projection::Perspective,
            render_target: Some(rt.clone()),
            ..Default::default()
        };
        let saved = look();
        set_look(Look {
            eye: cam.position,
            fovy: cam.fovy,
            rows: rt.texture.height(),
            ..saved
        });
        set_camera(&cam);
        clear_background(Color::new(0.0, 0.0, 0.0, 0.0));
        // 쉼터 불빛 하나를 앞 왼쪽 위에(어두운 성소의 화로처럼 — 인형이 묻히지 않게).
        let fire = crate::stage::Light {
            pos: vec3(-1.6, 2.2, 2.4),
            color: Color::new(1.0, 0.62, 0.25, 1.0),
            radius: 6.0,
            shaft: false,
        };
        self.batch.model(
            e,
            &self.pose,
            Mat4::from_rotation_y(spin),
            WHITE,
            &[fire],
            1.0,
        );
        self.batch.flush();
        set_look(saved);
        set_default_camera();
        blit(&rt, dest);
    }

    /// 도감의 생명체(0130 C5): 종의 모델을 숨쉬기 동작으로 고정 카메라에 — 키에 맞춰 칸에 차게. 모델이 아직 없는 종(상자 단계)은
    /// 게임과 같은 몸 상자. `dark`면 못 본 것 — 검은 실루엣(빛·색 없이 꼴만).
    pub fn draw_creature(
        &mut self,
        kind: crate::species::Kind,
        dest: Rect,
        spin: f32,
        time: f32,
        dark: bool,
    ) {
        let (rt, dest, aspect) = self.fit_doll_rt(true, dest);
        let d = crate::species::def(kind);
        let h = d.height.max(0.5);
        let cam = Camera3D {
            position: vec3(0.0, h * 0.75, h * 2.3 + d.radius * 2.0 + 1.0),
            target: vec3(0.0, h * 0.48, 0.0),
            up: Vec3::Y,
            fovy: 0.55,
            aspect: Some(aspect),
            projection: Projection::Perspective,
            render_target: Some(rt.clone()),
            ..Default::default()
        };
        let saved = look();
        set_look(Look {
            eye: cam.position,
            fovy: cam.fovy,
            rows: rt.texture.height(),
            ..saved
        });
        set_camera(&cam);
        clear_background(Color::new(0.0, 0.0, 0.0, 0.0));
        let shadow = Color::new(0.04, 0.035, 0.05, 1.0);
        let fire = crate::stage::Light {
            pos: vec3(-1.6, h + 1.0, 2.4),
            color: Color::new(1.0, 0.62, 0.25, 1.0),
            radius: 6.0 + h * 2.0,
            shaft: false,
        };
        if let Some(e) = self.lib.get(d.model) {
            e.model
                .pose_with(e.pick(&["idle"]), time, &[], &mut self.pose);
            let tint = if dark { shadow } else { WHITE };
            let lights: &[crate::stage::Light] = if dark {
                &[]
            } else {
                std::slice::from_ref(&fire)
            };
            self.batch.model(
                e,
                &self.pose,
                Mat4::from_rotation_y(spin),
                tint,
                lights,
                if dark { 0.0 } else { 1.0 },
            );
            self.batch.flush();
        } else {
            // 상자 단계(0125 등): 게임의 몸 상자와 같은 꼴 — 몸 + 머리 쪽 작은 상자.
            let body = if dark {
                shadow
            } else if d.boss {
                Color::new(0.45, 0.2, 0.5, 1.0)
            } else {
                Color::new(0.5, 0.35, 0.25, 1.0)
            };
            let r = d.radius;
            let rot = Quat::from_rotation_y(spin);
            shaded_box_rot(
                vec3(0.0, h * 0.5, 0.0),
                vec3(r * 2.0, h, r * 1.6),
                body,
                if dark { 1.0 } else { 0.85 },
                rot,
                Vec3::ZERO,
            );
            shaded_box_rot(
                vec3(0.0, h * 0.78, r * 0.85),
                vec3(r * 0.9, h * 0.25, 0.1),
                if dark {
                    shadow
                } else {
                    Color::new(0.2, 0.15, 0.22, 1.0)
                },
                1.0,
                rot,
                Vec3::ZERO,
            );
        }
        set_look(saved);
        set_default_camera();
        blit(&rt, dest);
    }
}
