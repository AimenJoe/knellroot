//! 모션 보기(개발, 사장 2026-11-15 "개발 메뉴에 무기별로 확인해 볼 수 있을 만한 메뉴가 없네"): 한 무기를 든 방랑자의 동작을 하나씩
//! 인형으로 돌려 본다 — 평타·강타·패링·회피·기술 …. 판정이 있는 동작은 시간 막대에 판정 구간을 같이 보인다(게임의 `hit_times`·
//! `heavy_times`·기술 표의 `Clip::seg` 그대로). 위에 무기 탭, 왼쪽 동작 줄, 오른쪽 인형·막대·숫자, 아래 안내(개발 창과 같은 꼴).
//! 상태는 장면이 들고 다닌다(`Scene::Motions` — 무기·빠르기·돌린 칸·재생 시각), 고른 줄 = 칸의 커서.
use super::menu::Hit;
use super::*;
use crate::i18n::dev::{self as dv, t};
use crate::weapons::{Rb, Weapon, IDS};

/// 동작 한 줄: 모델의 동작 이름과 판정 구간(게임 초 — 시작, 끝; 없으면 판정 없음), 동작 시계의 배(게임과 같이
/// `weapons::clip_rate` — 맨손이 빌린 검 동작은 빨리 돈다, 0195). 막대·시각은 게임 초.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) struct Row {
    pub anim: &'static str,
    pub window: Option<(f32, f32)>,
    pub rate: f32,
    /// 달리기 공격 줄(0202): 콤보의 타와 같은 동작이어도 따로 한 줄 — 판정 구간·시계는 달리기 공격의 것.
    pub dash: bool,
}

impl Row {
    /// 왼쪽 줄의 이름.
    pub fn label(&self) -> String {
        let name = dv::motion_label(self.anim);
        if self.dash {
            fill(t(dv::MOTION_DASH), &[&name])
        } else {
            name
        }
    }
}

/// 빠르기 칸(0 = 1×, 1 = ½, 2 = ¼, 3 = 멈춤).
/// 0243(사장 설문 k19 "모션 보는게 너무 빠른 경우도 있음"): 처음은 ½× — 확인으로 ¼× → 0.1× → 1× → 멈춤.
pub(super) const SPEEDS: [f32; 5] = [0.5, 0.25, 0.1, 1.0, 0.0];
/// 한 번 돈 끝에서 마지막 자세로 머무는 시간(초) — 끝 자세가 보이게.
const HOLD_END: f32 = 0.35;
/// 돌린 칸 하나(도).
const SPIN_STEP: f32 = 30.0;

/// 이 무기로 볼 동작들(차례 = 몸 → 무기 → 막기·피하기 → 맞음 → 기술). 같은 동작은 한 번만. 순수 — 시험.
pub(super) fn rows(w: &Weapon, tn: &Tuning) -> Vec<Row> {
    let mut out: Vec<Row> = Vec::new();
    fn put(out: &mut Vec<Row>, anim: &'static str, window: Option<(f32, f32)>) {
        if !out.iter().any(|r| r.anim == anim && !r.dash) {
            let rate = window.map_or(1.0, |(s, _)| crate::weapons::clip_rate(anim, s));
            out.push(Row {
                anim,
                window,
                rate,
                dash: false,
            });
        }
    }
    macro_rules! add {
        ($a:expr, $w:expr) => {
            put(&mut out, $a, $w)
        };
    }
    for a in [
        "idle",
        "idle_look",
        "idle_shake",
        "walk",
        "run",
        "walk_back",
        "turn_l",
        "turn_r",
    ] {
        add!(a, None);
    }
    for (k, h) in w.combo.iter().enumerate() {
        let (s0, a0, _) = crate::world::hit_times(tn, w, k as u8);
        add!(h.anim, Some((s0, s0 + a0)));
    }
    if let Some(d) = w.dash {
        let (s0, a0, _) = crate::world::hit_times(tn, w, crate::weapons::DASH);
        out.push(Row {
            anim: d.anim,
            window: Some((s0, s0 + a0)),
            rate: crate::weapons::clip_rate(d.anim, s0),
            dash: true,
        });
    }
    if let Some(h) = w.heavy {
        add!(h.anim_charge, None);
        let (s0, a0, _) = crate::world::heavy_times(tn, w);
        add!(h.anim_heavy, Some((s0, s0 + a0)));
    }
    match w.rb {
        Rb::Magic(_) => {
            add!("cast_charge", None);
            add!("cast", None);
        }
        Rb::Shot { .. } if w.id == crate::weapons::Id::Bow => {
            add!("bow_draw", None);
            add!("bow_shoot", None);
        }
        Rb::Shot { .. } => {
            add!("xbow_charge", None);
            add!("xbow_shoot", None);
        }
    }
    if w.grenade.is_some() {
        add!("throw", None);
    }
    for a in [
        "parry",
        "parry_push",
        "dodge",
        "dodge_back",
        "jump",
        "land",
        "plunge",
        "plunge_land",
        "drink",
        "pickup",
        "pull",
        "hurt",
        "fall",
        "wake",
    ] {
        add!(a, None);
    }
    let clips = crate::skills::SKILLS
        .iter()
        .map(|s| s.clip)
        .chain(crate::skills::ULTS.map(|u| u.clip()));
    for c in clips {
        let [s, a, _] = c.seg;
        add!(c.name, Some((s, s + a.max(0.03))));
    }
    out
}

/// 빠르기 칸의 글.
fn speed_label(speed: u8) -> String {
    let x = SPEEDS[speed as usize % SPEEDS.len()];
    if x == 0.0 {
        t(dv::MOTION_PAUSED).to_string()
    } else if x == 0.5 {
        "½×".to_string()
    } else if x == 0.25 {
        "¼×".to_string()
    } else {
        format!("{x}×")
    }
}

impl App {
    /// 모션 보기 한 틱: 뒤로·✕·창 밖 = 나가기, 탭 넘기기·탭 누르기 = 무기, 위아래·굴리기·줄 누르기 = 동작, 좌우 = 돌리기,
    /// 확인 = 빠르기(1 → ½ → ¼ → 멈춤). 동작을 바꾸면 처음부터.
    pub(super) fn tick_motions(&mut self, a: &Actions, weapon: u8, speed: u8, spin: i8, t_ms: u32) {
        let a = &self.small_strip_input(a);
        let tap = a.tap.and_then(|p| self.hit_at(p));
        if a.back || a.pause || tap == Some(Hit::Close) {
            self.go(Go::Back);
            return;
        }
        let n_w = IDS.len() as i32;
        let wstep = a.tab_next as i32 - a.tab_prev as i32;
        let to_w = match tap {
            Some(Hit::Cell(1, i)) => i as i32,
            _ => (weapon as i32 + wstep).rem_euclid(n_w),
        } as u8;
        let w = IDS[to_w as usize % IDS.len()].def();
        let list = rows(w, &self.t);
        let before = self.cursor;
        ui::menu_input(&mut self.cursor, list.len(), a.menu_up, a.menu_down);
        let scroll = match tap {
            Some(Hit::Scroll(d)) => d,
            _ => 0,
        };
        self.view
            .scroll((a.drag, a.wheel + scroll), &mut self.cursor, list.len());
        if let Some(Hit::Row(i)) = tap {
            self.cursor = i;
        }
        self.cursor = self.cursor.min(list.len().saturating_sub(1));
        let speed = if a.confirm {
            (speed + 1) % SPEEDS.len() as u8
        } else {
            speed
        };
        let spin = spin.wrapping_add(a.menu_right as i8 - a.menu_left as i8);
        let restart = to_w != weapon || self.cursor != before;
        let t_ms = if restart {
            0
        } else {
            let dt = get_frame_time().min(0.1) * SPEEDS[speed as usize % SPEEDS.len()];
            t_ms.saturating_add((dt * 1000.0) as u32)
        };
        if to_w != weapon || a.confirm {
            self.snd(crate::sfx::Id::UiMove);
        }
        self.stack.retag(Scene::Motions {
            weapon: to_w,
            speed,
            spin,
            t_ms,
        });
    }

    /// 모션 보기 그림: 창 틀 · 무기 탭 줄 · 왼쪽 동작 목록 · 오른쪽 인형 + 시간 막대(판정 구간) + 숫자 · 아래 안내.
    pub(super) fn draw_motions(&mut self, f: &Frame, weapon: u8, speed: u8, spin: i8, t_ms: u32) {
        let id = IDS[weapon as usize % IDS.len()];
        let w = id.def();
        self.ui.dim(0.45);
        let u = self.ui.u(1.0);
        let px = self.ui.px(1.0);
        let panel = self.panel_rect(f, 440.0, 226.0);
        let title = fill(t(dv::MOTION_TITLE), &[weapon_name(id)]);
        let top = self.ui.panel(panel, Some(&title));
        let x = self.ui.close(panel);
        self.hits.push((x, Hit::Close));
        let (lx, rx_end) = (panel.x + u * 10.0, panel.x + panel.w - u * 10.0);
        // 무기 탭: 개발 창의 묶음 탭과 같은 꼴 — 넘치면 가로로 굴린다(0257).
        let tab_h = px * 1.6;
        let names: Vec<&str> = IDS.iter().map(|&i| weapon_name(i)).collect();
        let rects = self.ui.button_strip(
            &mut self.small_strip,
            &names,
            weapon as usize,
            lx,
            rx_end,
            top,
            tab_h,
        );
        for (i, r) in rects.into_iter().enumerate() {
            self.hits.push((r, Hit::Cell(1, i)));
        }
        let tabs_end = top + tab_h;
        let body = tabs_end + u * 8.0;
        let hint_y = panel.y + panel.h - u * 8.0 - px;
        let split = panel.x + panel.w * 0.36;
        draw_line(split, body, split, hint_y - u * 4.0, u.max(1.0), ui::BORDER);
        let list = rows(w, &self.t);
        let labels: Vec<String> = list.iter().map(Row::label).collect();
        let ui_rows: Vec<ui::Row> = labels
            .iter()
            .map(String::as_str)
            .map(ui::Row::plain)
            .collect();
        let area = Rect::new(
            lx - u * 4.0,
            body,
            split - lx - u * 4.0,
            hint_y - u * 6.0 - body,
        );
        let cursor = self.cursor.min(list.len().saturating_sub(1));
        let hits = self.ui.list(&ui_rows, cursor, &mut self.view, area);
        self.push_hits(&hits);
        let Some(row) = list.get(cursor).copied() else {
            return;
        };
        // 오른쪽: 인형(위) · 시간 막대 · 숫자 두 줄(아래).
        let dx = split + u * 8.0;
        let dw = rx_end - dx;
        let info_h = px * 1.3 * 2.0 + u * 10.0;
        let doll = Rect::new(dx, body, dw, hint_y - u * 6.0 - body - info_h);
        let len = self
            .renderer
            .wanderer_anim_len(row.anim)
            .map(|d| d / row.rate);
        let dur = len.unwrap_or(1.0);
        let cycle = dur + HOLD_END;
        let at = (t_ms as f32 / 1000.0) % cycle;
        let shown = at.min(dur * 0.999);
        self.renderer.draw_doll_anim(
            id,
            doll,
            (spin as f32 * SPIN_STEP).to_radians(),
            row.anim,
            shown * row.rate,
            true,
        );
        let bar = Rect::new(dx, doll.y + doll.h + u * 4.0, dw, u * 4.0);
        draw_rectangle(bar.x, bar.y, bar.w, bar.h, Color::new(1.0, 1.0, 1.0, 0.12));
        if let Some((s, e)) = row.window {
            let (a, b) = ((s / dur).clamp(0.0, 1.0), (e / dur).clamp(0.0, 1.0));
            draw_rectangle(
                bar.x + bar.w * a,
                bar.y,
                bar.w * (b - a).max(0.004),
                bar.h,
                AMBER,
            );
        }
        let head = bar.x + bar.w * (shown / dur).clamp(0.0, 1.0);
        draw_rectangle(head - u * 0.5, bar.y - u * 2.0, u, bar.h + u * 4.0, INK);
        let line1 = match len {
            Some(d) => fill(
                t(dv::MOTION_LEN),
                &[
                    &format!("{d:.2}"),
                    &format!("{shown:.2}"),
                    &speed_label(speed),
                ],
            ),
            None => t(dv::MOTION_NONE).to_string(),
        };
        let line2 = match row.window {
            Some((s, e)) => format!(
                "{} · {}",
                row.anim,
                fill(t(dv::MOTION_HIT), &[&format!("{s:.2}"), &format!("{e:.2}")])
            ),
            None => row.anim.to_string(),
        };
        let y0 = bar.y + bar.h + u * 4.0;
        let l1 = self.ui.fit(&line1, dw, 0.9);
        let l2 = self.ui.fit(&line2, dw, 0.9);
        self.ui.text(&l1, dx, y0, 0.9, INK);
        self.ui.text(&l2, dx, y0 + px * 1.3, 0.9, DIM);
        let hint = self.ui.fit(t(dv::MOTION_HINT), panel.w - u * 20.0, 0.85);
        self.ui.text(&hint, lx, hint_y, 0.85, DIM);
        self.push_outside(panel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 모션 보기는 무기마다 그 무기의 평타·강타·특수 동작과 공통 동작·기술을 다 보이고, 모든 줄은 모델에 있는 동작이며
    /// 판정 구간은 동작 길이 안이다(게임의 시간 셈 그대로).
    #[test]
    fn every_weapon_lists_its_own_motions_and_they_exist_in_the_model() {
        let m = crate::model::Model::from_bytes(include_bytes!("../../assets/models/wanderer.glb"))
            .unwrap();
        let tn = Tuning::default();
        for id in IDS {
            let w = id.def();
            let list = rows(w, &tn);
            for h in w.combo {
                assert!(list.iter().any(|r| r.anim == h.anim), "{id:?}: {}", h.anim);
            }
            // 달리기 공격은 콤보의 타와 같은 동작이어도 따로 한 줄(0202).
            assert_eq!(
                list.iter().filter(|r| r.dash).count(),
                w.dash.iter().count(),
                "{id:?}"
            );
            for a in ["idle", "walk", "parry", "dodge", "sk_whirl", "ult_line"] {
                assert!(list.iter().any(|r| r.anim == a), "{id:?}: {a}");
            }
            let mut seen = std::collections::HashSet::new();
            for r in &list {
                assert!(seen.insert((r.anim, r.dash)), "{id:?}: {} twice", r.anim);
                let ai = m
                    .anim(r.anim)
                    .unwrap_or_else(|| panic!("{id:?}: no clip {}", r.anim));
                if let Some((s, e)) = r.window {
                    let d = m.anims[ai].duration / r.rate;
                    assert!(
                        0.0 <= s && s < e && e <= d + 1e-3,
                        "{id:?} {}: window {s}..{e} in {d}",
                        r.anim
                    );
                }
                assert!(!dv::motion_label(r.anim).is_empty());
            }
        }
    }

    /// 모션 보기 인형은 동작 내내 칸 안에 있다: 동작이 쓸고 가는 뼈 자리를 다 담고, 제자리 동작(서기·몸짓·걷기·돌기)은 같은 틀(같은 크기)이다.
    #[test]
    fn the_motion_doll_frames_the_whole_sweep_and_keeps_standing_moves_the_same_size() {
        use crate::render::items::{doll_eye, sweep_box};
        let m = crate::model::Model::from_bytes(include_bytes!("../../assets/models/wanderer.glb"))
            .unwrap();
        let idle = sweep_box(&m, m.anim("idle"));
        for a in ["idle_look", "walk", "attack"] {
            assert_eq!(
                idle,
                sweep_box(&m, m.anim(a)),
                "standing moves share a frame: {a}"
            );
        }
        let fall = sweep_box(&m, m.anim("fall"));
        assert!(
            fall.2 > idle.2 || fall.0 < idle.0,
            "the fall leaves the standing frame: {fall:?}"
        );
        for a in ["fall", "dodge", "plunge_land", "wake", "jump", "ult_upend"] {
            let b = sweep_box(&m, m.anim(a));
            let (eye, at) = doll_eye(b, 1.6);
            // 틀의 꼭짓점이 모두 시야(fovy 0.48) 안 — 위·아래 끝, 가장 가까운 쪽.
            for p in [
                vec3(0.0, b.0, b.2),
                vec3(0.0, b.1, b.2),
                vec3(b.2, b.1, 0.0),
                vec3(-b.2, b.0, 0.0),
            ] {
                let fwd = (at - eye).normalize();
                let right = fwd.cross(Vec3::Y).normalize();
                let up = right.cross(fwd);
                let to = p - eye;
                let z = to.dot(fwd);
                let half = 0.24_f32.tan();
                assert!(
                    (to.dot(up) / z).abs() <= half,
                    "{a}: {p} above/below the doll view"
                );
                assert!(
                    (to.dot(right) / z).abs() <= half * 1.6,
                    "{a}: {p} beside the doll view"
                );
            }
        }
    }
}
