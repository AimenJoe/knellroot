//! 아이템 그림(0068)·장비 창 인형(0067)·도감 생물(0130): 작은 렌더 타깃 + 고정 카메라.
use super::*;

impl Renderer {
    /// 아이템 아이콘(0068): 처음 부를 때 한 번 찍어 두고(정한 각도), 다음부터는 그 그림을 `dest`에 붙인다.
    /// 모델·부위가 없으면 아무것도 그리지 않는다(칸은 비어 보인다).
    /// 0250: 메뉴 그림을 촘촘하게 켜면 칸의 화면 크기(물리 px × 2)로 찍는다 — 크기마다 따로 둔다(칸 크기는 화면마다 몇 가지뿐).
    pub fn draw_icon(&mut self, item: crate::item_view::Item, dest: Rect) {
        let key = crate::item_view::key(item);
        let side = self.menu_texels(dest.w, ICON_PX);
        let rt = match self.icons.get(&(key, side)) {
            Some(rt) => rt.clone(),
            None => {
                let rt = self.menu_rt(side, side);
                if !self.render_item(item, &rt, ICON_PX, 0.0) {
                    return;
                }
                self.icons.insert((key, side), rt.clone());
                rt
            }
        };
        blit(&rt, dest);
    }

    /// 메뉴 그림 타깃의 한 변(0250): 촘촘하면 칸의 물리 px × 2(줄여 붙이며 섞는다 — 계단 없이), 아니면 예전 고정 크기 `native`.
    fn menu_texels(&self, logical: f32, native: u32) -> u32 {
        if !self.menu_fine {
            return native;
        }
        let dpi = screen_dpi_scale().max(0.01);
        ((logical * dpi * 2.0).round() as u32).clamp(native, MENU_MAX_PX)
    }

    /// 메뉴 그림 타깃: 촘촘하면 선형으로 줄여 붙인다(2×2가 한 화소로 섞인다), 아니면 게임처럼 가장 가까운 화소.
    fn menu_rt(&self, w: u32, h: u32) -> RenderTarget {
        let rt = make_rt(w, h);
        if self.menu_fine {
            rt.texture.set_filter(FilterMode::Linear);
        }
        rt
    }

    /// 고른 아이템 하나를 천천히 돌려 크게(0068): 타깃 하나를 매 프레임 다시 그린다(인형과 같은 값싼 패스).
    pub fn draw_item(&mut self, item: crate::item_view::Item, dest: Rect, spin: f32) {
        let side = self.menu_texels(dest.w, ITEM_PX);
        if self
            .item_rt
            .as_ref()
            .is_none_or(|r| r.texture.width() as u32 != side)
        {
            self.item_rt = Some(self.menu_rt(side, side));
        }
        let rt = self.item_rt.clone().unwrap();
        if self.render_item(item, &rt, ITEM_PX, spin) {
            blit(&rt, dest);
        }
    }

    /// 아이템 부위 하나만(같은 모델의 다른 부위는 크기 0) 경계 상자 가운데에 놓고 표의 각도 + `spin`으로 돌려, 틀에 꽉 차게 그린다.
    /// 게임과 같은 툰 빛·외곽선(`Batch::model`). 그렸으면 참. 화면 패스 중에 불러도 기본 카메라로 돌려놓는다.
    /// `px` = 외곽선을 셀 줄 수 — 타깃이 촘촘해도 예전 고정 크기(`ICON_PX`·`ITEM_PX`)로 세어 선의 굵기(그림에 대한 몫)가 그대로다(0250).
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
        // 무기는 손뼈에 쥔 방향으로 붙어 있다(0193 스킨 몸) — 제 모양의 축으로 되돌린 뒤 표의 각도로.
        let undo = match item {
            crate::item_view::Item::Weapon(_) => e.model.rest_rotation(i).inverse(),
            _ => Quat::IDENTITY,
        };
        let inst = Mat4::from_quat(rot * undo) * Mat4::from_translation(-c);
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
        // 인형·아이템 그림은 어느 지역에서 열든 같은 빛으로(0249 — 지역의 빛 색은 세계에만).
        let saved_sky = sky();
        set_sky(crate::backdrop::SURFACE.light);
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
        set_sky(saved_sky);
        set_default_camera();
        true
    }

    /// 방랑자 동작의 길이(초) — 모델에 없으면 None(모션 보기의 시간 막대, 개발판).
    #[cfg(feature = "dev")]
    pub fn wanderer_anim_len(&mut self, anim: &str) -> Option<f32> {
        let e = self.lib.get("wanderer")?;
        let i = e.model.anim(anim)?;
        Some(e.duration(Some(i)))
    }

    /// 인형·도감 칸의 타깃(0190): 화면의 칸 `dest`를 게임 화면과 같은 도트 크기(한 칸 = 화면 배율 px)로 나눈 크기 —
    /// 내부 해상도 줄 수(`render.rows`)를 바꾸면 인형의 도트도 같이 바뀐다. 남는 px은 칸 가운데로 모은다. 돌려주는 것: 타깃·붙일 자리·가로세로 비.
    ///
    /// 0250(사장 "메뉴에서는 해상도가 좀 높게"): 연구소 `render.menu fine`이 켜지면(기본) 칸을 물리 px × 2로 그려 선형으로 줄여 붙인다 —
    /// 게임 화면과 달리 메뉴의 그림은 가까이 보는 것이라 도트가 아니라 또렷함이 먼저다. 외곽선 굵기는 게임 도트로 센 줄 수(`rows`)로
    /// 그대로 둔다. 예전 셈은 논리 px 칸을 물리 px 배율로 나눠, 폰(DPR 3)에서는 도트가 게임의 세 배였다.
    /// 돌려주는 것: 타깃·붙일 자리·가로세로 비·외곽선을 셀 줄 수.
    fn fit_doll_rt(&mut self, creature: bool, dest: Rect) -> (RenderTarget, Rect, f32, f32) {
        let dpi = screen_dpi_scale().max(0.01);
        // 게임의 도트 한 칸(논리 px).
        let dot = self.rt_size.2.max(1) as f32 / dpi;
        let fine = self.menu_fine;
        // 타깃 화소 하나가 덮는 논리 px.
        let texel = if fine {
            0.5 / dpi
        } else {
            self.rt_size.2.max(1) as f32
        };
        let cap = if fine { MENU_MAX_PX } else { u32::MAX };
        let (tw, th) = (
            ((dest.w / texel).floor() as u32).clamp(DOLL_W / 4, cap),
            ((dest.h / texel).floor() as u32).clamp(DOLL_H / 4, cap),
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
            let rt = make_rt(tw, th);
            if fine {
                rt.texture.set_filter(FilterMode::Linear);
            }
            *slot = Some(rt);
        }
        let rt = slot.clone().unwrap();
        let (w, h) = if fine {
            // 큰 칸이 상한에 걸리면 비를 지키며 칸 안에.
            let k = (dest.w / tw as f32).min(dest.h / th as f32);
            (tw as f32 * k, th as f32 * k)
        } else {
            (tw as f32 * texel, th as f32 * texel)
        };
        let at = Rect::new(
            (dest.x + (dest.w - w) * 0.5).round(),
            (dest.y + (dest.h - h) * 0.5).round(),
            w,
            h,
        );
        let rows = if fine { h / dot } else { th as f32 };
        (rt, at, tw as f32 / th as f32, rows)
    }

    /// 장비 창의 인형(0067 D, 디아블로식): 방랑자만(든 무기만 보이게) 고정 카메라로 작은 타깃에 그리고, 화면 `dest`에 붙인다.
    /// `spin` = 몸 방향(라디안), `time` = 숨쉬기 동작 시간. 모델이 없으면 아무것도 그리지 않는다(창이 초상 없이 그대로).
    /// 화면 패스 중에 불러도 된다: 카메라를 바꿨다가 기본 카메라로 돌려놓는다.
    pub fn draw_doll(&mut self, weapon: crate::weapons::Id, dest: Rect, spin: f32, time: f32) {
        self.draw_doll_anim(weapon, dest, spin, "idle", time, false);
    }

    /// 인형이 `anim` 동작의 `time`초 자세로(개발 창의 모션 보기 — 없는 동작이면 서기).
    pub fn draw_doll_anim(
        &mut self,
        weapon: crate::weapons::Id,
        dest: Rect,
        spin: f32,
        anim: &str,
        time: f32,
        fit: bool,
    ) {
        let (rt, dest, aspect, rows) = self.fit_doll_rt(false, dest);
        let Some(e) = self.lib.get("wanderer") else {
            return;
        };
        // 모션 보기(`fit`): 동작 내내 몸이 칸 안에 있게 — 쓰러짐·구르기·내려찍기는 서 있는 틀 밖으로 나가 잘렸다(2026-11-19 점검).
        let frame = fit.then(|| {
            *self
                .doll_fit
                .entry(anim.to_string())
                .or_insert_with(|| sweep_box(&e.model, e.pick(&[anim, "idle"])))
        });
        // 든 무기만 보인다(0051): 다른 무기 부위는 크기 0.
        let mut hide: Vec<(usize, Quat, Vec3)> = Vec::new();
        super::hide_unheld(&e.model, weapon, self.shield, false, &mut hide);
        e.model
            .pose_with(e.pick(&[anim, "idle"]), time, &hide, &mut self.pose);
        // 키 1.8 몸이 칸에 꽉 차게: 가슴 높이를 보고 조금 위에서. 모션 보기는 동작이 쓸고 가는 틀에 맞춰 물러선다.
        let (position, target) = match frame {
            Some(b) => doll_eye(b, aspect),
            None => (vec3(0.0, 1.35, 4.6), vec3(0.0, 0.85, 0.0)),
        };
        let cam = Camera3D {
            position,
            target,
            up: Vec3::Y,
            fovy: 0.48,
            aspect: Some(aspect),
            projection: Projection::Perspective,
            render_target: Some(rt.clone()),
            ..Default::default()
        };
        let saved = look();
        // 인형·아이템 그림은 어느 지역에서 열든 같은 빛으로(0249 — 지역의 빛 색은 세계에만).
        let saved_sky = sky();
        set_sky(crate::backdrop::SURFACE.light);
        set_look(Look {
            eye: cam.position,
            fovy: cam.fovy,
            rows,
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
        set_sky(saved_sky);
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
        let (rt, dest, aspect, rows) = self.fit_doll_rt(true, dest);
        let d = crate::species::def(kind);
        let h = d.height.max(0.5);
        // 칸에 맞춰 당긴다(2026-11-19 사장 "모델들이 가로로 더 채워지면"): 키(위아래 반 + 여유)와 몸 둘레(돌려도 들어가게 — 가로 반)
        // 가운데 더 먼 쪽에 선다. 예전엔 키에 상관없이 멀찍이(키 × 2.3 + 1m) 서서 칸의 반도 안 찼다.
        let half_tan = (CREATURE_FOVY * 0.5).tan();
        let need_v = h * 0.56 / half_tan;
        let need_h = (d.radius * 1.35).max(h * 0.36) / (half_tan * aspect.max(0.1));
        let dist = need_v.max(need_h) * 1.14;
        let cam = Camera3D {
            position: vec3(0.0, h * 0.5 + dist * 0.16, dist),
            target: vec3(0.0, h * 0.47, 0.0),
            up: Vec3::Y,
            fovy: CREATURE_FOVY,
            aspect: Some(aspect),
            projection: Projection::Perspective,
            render_target: Some(rt.clone()),
            ..Default::default()
        };
        let saved = look();
        // 인형·아이템 그림은 어느 지역에서 열든 같은 빛으로(0249 — 지역의 빛 색은 세계에만).
        let saved_sky = sky();
        set_sky(crate::backdrop::SURFACE.light);
        set_look(Look {
            eye: cam.position,
            fovy: cam.fovy,
            rows,
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
        set_sky(saved_sky);
        set_default_camera();
        blit(&rt, dest);
    }
}

/// 모션 보기 인형의 쉬는 틀(아래 끝·위 끝·가로 반지름, m)의 바닥값 — 서기(`idle`)가 쓸고 가는 틀과 합쳐 쉬는 틀이 된다.
/// 서서 하는 동작은 모두 그 틀로 같은 크기에 보인다(줄을 넘겨도 인형이 커졌다 작아졌다 하지 않게).
const DOLL_BASE: (f32, f32, f32) = (0.0, 1.45, 0.5);
/// 몸의 겉에서 칸 가장자리까지의 여유(m) — 든 무기의 날이 몸 밖으로 나간다.
const DOLL_MARGIN: f32 = 0.3;

/// 동작이 쓸고 가는 틀: 동작을 고르게 스물다섯 자리에서 세워 본 몸(스킨 정점)의 높이 범위와 가로 반지름(돌려 보아도 들게 — 원점 둘레),
/// 쉬는 틀 `DOLL_BASE`와 합친다. 순수 — 시험.
pub(crate) fn sweep_box(m: &crate::model::Model, anim: Option<usize>) -> (f32, f32, f32) {
    let rest = sweep_of(m, m.anim("idle"), DOLL_BASE);
    let b = sweep_of(m, anim, rest);
    // 쉬는 틀을 조금(`DOLL_SLACK` — 여유 `DOLL_MARGIN` 안)만 넘는 동작은 쉬는 틀 그대로: 서서 휘두르는 동작이 다 같은 크기로.
    let over = (rest.0 - b.0).max(b.1 - rest.1).max(b.2 - rest.2);
    if over <= DOLL_SLACK {
        rest
    } else {
        b
    }
}

/// 쉬는 틀을 이만큼(m)까지 넘는 동작은 쉬는 틀로 본다.
const DOLL_SLACK: f32 = 0.15;

fn sweep_of(
    m: &crate::model::Model,
    anim: Option<usize>,
    from: (f32, f32, f32),
) -> (f32, f32, f32) {
    let (mut lo, mut hi, mut r) = from;
    let d = anim
        .and_then(|i| m.anims.get(i))
        .map_or(0.0, |a| a.duration);
    let (mut pose, mut pos, mut nrm) = (Vec::new(), Vec::new(), Vec::new());
    // 휜 몸의 정점으로 잰다(뼈 자리만으로는 모자랐다 — 쓰러짐은 몸이 땅 밑 0.3m까지 들어가 칸 아래가 잘렸다, 2026-11-19).
    // 무기·방패 조각은 든 것만 보이니 빼고 스킨 몸만.
    let skinned: Vec<usize> = (0..m.nodes.len())
        .filter(|&i| m.nodes[i].skin.is_some())
        .collect();
    for k in 0..=24 {
        m.pose(anim, d * k as f32 / 24.0, &mut pose);
        for &i in &skinned {
            if !m.skin_into(i, &pose, &mut pos, &mut nrm) {
                continue;
            }
            for v in &pos {
                lo = lo.min(v.y);
                hi = hi.max(v.y);
                r = r.max(v.x.hypot(v.z));
            }
        }
    }
    (lo, hi, r)
}

/// 틀 `(lo, hi, r)`이 가로세로 비 `aspect`의 칸에 들게 하는 눈과 보는 곳: 틀 가운데를 조금 위에서(서 있는 인형과 같은 숙임).
/// 깊이 쪽 반지름의 반만큼 더 물러선다(가까운 쪽이 원근으로 커져도 들게).
pub(crate) fn doll_eye(b: (f32, f32, f32), aspect: f32) -> (Vec3, Vec3) {
    let (lo, hi, r) = b;
    let half = 0.24_f32.tan(); // fovy 0.48의 반
    let need_h = (hi - lo) * 0.5 + DOLL_MARGIN;
    let need_w = r + DOLL_MARGIN;
    // 깊이 쪽은 반지름의 반만 — 반지름은 원점 둘레의 가장 먼 점(꼬리 끝)이라 앞쪽이 늘 그만큼 다가오지는 않는다.
    let d = (need_h / half).max(need_w / (half * aspect.max(0.1))) + r * 0.5;
    let target = vec3(0.0, (lo + hi) * 0.5, 0.0);
    (target + vec3(0.0, d * 0.11, d), target)
}

/// 도감 생물 그림의 세로 시야(라디안).
const CREATURE_FOVY: f32 = 0.55;
