//! 싸움 화면의 글·막대(0019). 그린 화면(`render::Frame`) 안에 같은 정수 배율로 붙인다. 월드·스테이지를 읽기만 한다.
//!
//! 자리: 왼쪽 위 체력(마름모)·스태미나 · 오른쪽 위 목표 · 가운데 위 구역 카드 · 아래 가운데 보스 띠 · 가운데 시작 이야기.
use macroquad::prelude::*;

use crate::i18n::{self, tr};
use crate::render::Frame;
use crate::stage::{StageRun, CARD_TIME, INTRO_TIME};
use crate::tuning::{idx, CAST_CHARGES, P, SPECS};
use crate::ui::{Ui, AMBER, DIM, INK};
use crate::weapons::Rb;
use crate::world::World;

const HP_ON: Color = Color::new(0.88, 0.6, 0.23, 1.0);
const HP_OFF: Color = Color::new(0.2, 0.18, 0.17, 0.9);
const STAMINA: Color = Color::new(0.55, 0.78, 0.5, 1.0);
const DENIED: Color = Color::new(1.0, 0.3, 0.25, 1.0);
/// 등잔(0031): 놋쇠 몸 + 불.
const FLASK_ON: Color = Color::new(0.62, 0.5, 0.32, 1.0);
const FLASK_FLAME: Color = Color::new(1.0, 0.8, 0.35, 1.0);
/// 마법 칸(0048): 라벤더(탄과 같은 색). 쇠뇌 탄창·수류탄(0051): 밝은 나무.
const MANA: Color = Color::new(0.75, 0.66, 1.0, 1.0);
const AMMO: Color = Color::new(0.95, 0.82, 0.55, 1.0);
const BOSS: Color = Color::new(0.72, 0.62, 0.42, 1.0);
const BOSS_ROUSED: Color = Color::new(0.95, 0.55, 0.2, 1.0);
/// 자세 막대(0055): 밝은 크림, 깨지면 금빛.
const POISE: Color = Color::new(0.92, 0.88, 0.78, 0.9);
const POISE_BROKEN: Color = Color::new(1.0, 0.82, 0.3, 1.0);
/// 잔불 수(0064 M1)와 주웠을 때 커지는 시간(초).
const EMBER: Color = Color::new(1.0, 0.74, 0.32, 1.0);
const EMBER_POP: f32 = 0.35;

const BAR_BG: Color = Color::new(0.05, 0.05, 0.06, 0.85);

/// 0..1 → 들어올 때·나갈 때 `edge`초씩 부드럽게(카드·이야기).
fn fade_in_out(left: f32, total: f32, edge: f32) -> f32 {
    let shown = total - left;
    (shown / edge).min(left / edge).clamp(0.0, 1.0)
}

/// UI px 하나(480×270 기준 UI 배율, 0041).
fn u(f: &Frame) -> f32 {
    f.ui
}

/// `under_menu`: 메뉴(멈춤)가 위에 뜬다 — 아래 가운데 보스 띠는 메뉴 안내와 겹쳐 그리지 않는다(KB hud-readability: 창이 덮는 HUD는 뺀다).
/// 갈 곳이 화면 밖일 때 가장자리 화살(0060 W1, nail 0159 출구 표시와 같은 장치): `at` 화면 자리, `ang` 화면 방향(라디안, y 아래),
/// 호박색 삼각형이 바깥을 가리키고 안쪽에 작은 마름모(갈 곳), 어두운 테두리로 어느 바닥 위에서도, 맥박. 색만이 아니라 모양으로 읽힌다.
pub fn draw_goal_arrow(f: &Frame, at: Vec2, ang: f32, time: f32) {
    let s = f.ui * (7.0 + 1.0 * (time * 4.0).sin());
    let d = vec2(ang.cos(), ang.sin());
    let n = vec2(-d.y, d.x);
    // 길쭉한 화살촉(길이 2 : 폭 1.1) — 작은 크기에서도 어느 쪽인지 읽히게.
    let tip = at + d * s * 1.2;
    let (l, r) = (
        at - d * s * 0.8 + n * s * 0.55,
        at - d * s * 0.8 - n * s * 0.55,
    );
    let shade = Color::new(0.05, 0.04, 0.06, 0.8);
    let grow = |p: Vec2| at + (p - at) * 1.35;
    draw_triangle(grow(tip), grow(l), grow(r), shade);
    draw_triangle(tip, l, r, AMBER);
    let c = at - d * s * 1.9;
    let k = s * 0.45;
    draw_poly(c.x, c.y, 4, k + f.ui * 1.2, 0.0, shade);
    draw_poly(c.x, c.y, 4, k, 0.0, AMBER);
}

/// 커서 조준 표시(0058): 고리 + 가운데 점, 어두운 테두리로 어느 바닥에서도 보이게. 크기 단계 0 작게·1 보통·2 크게.
/// 붉은색(위험)은 쓰지 않는다 — 적의 예고와 섞이지 않게.
pub fn draw_reticle(f: &Frame, at: Vec2, size: u8) {
    let r = [3.0, 4.5, 6.5][(size as usize).min(2)] * f.ui;
    let w = (f.ui * 0.9).max(1.0);
    let shade = Color::new(0.05, 0.04, 0.06, 0.75);
    draw_circle_lines(at.x, at.y, r, w + 2.0, shade);
    draw_circle_lines(at.x, at.y, r, w, INK);
    draw_circle(at.x, at.y, w + 1.0, shade);
    draw_circle(at.x, at.y, w * 0.6, INK);
}

pub fn draw(ui: &Ui, f: &Frame, w: &World, run: Option<&StageRun>, under_menu: bool) {
    let s = u(f);
    let (x0, y0) = (f.ox + 8.0 * s, f.oy + 8.0 * s);
    // 체력: 마름모.
    let pl = &w.player;
    let max_hp = pl.max_hp.max(pl.hp);
    for i in 0..max_hp {
        let c = vec2(x0 + 4.0 * s + i as f32 * 10.0 * s, y0 + 4.0 * s);
        let col = if i < pl.hp { HP_ON } else { HP_OFF };
        let r = 3.5 * s;
        draw_triangle(c + vec2(0.0, -r), c + vec2(r, 0.0), c + vec2(0.0, r), col);
        draw_triangle(c + vec2(0.0, -r), c + vec2(-r, 0.0), c + vec2(0.0, r), col);
    }
    // 스태미나(0018): 모자라서 막히면 잠깐 붉게 깜빡인다(막은 이유가 보이게 — KB action-reuse-gating).
    let smax = SPECS[idx(P::StaminaMax)].def.max(1.0);
    let k = (pl.stamina / smax).clamp(0.0, 1.0);
    let (bx, by, bw, bh) = (x0, y0 + 11.0 * s, 60.0 * s, 2.0 * s);
    draw_rectangle(bx, by, bw, bh, BAR_BG);
    let blink = pl.stamina_denied > 0.0 && (w.time * 16.0) as i32 % 2 == 0;
    draw_rectangle(bx, by, bw * k, bh, if blink { DENIED } else { STAMINA });
    // 등잔(0031): 남은 것은 불 켠 등잔, 쓴 것은 꺼진 등잔. 못 마시면(가득·0) 잠깐 깜빡인다.
    let fblink = pl.flask_denied > 0.0 && (w.time * 16.0) as i32 % 2 == 0;
    for i in 0..pl.flask_max.max(pl.flask) {
        let (lx, ly) = (x0 + i as f32 * 9.0 * s, y0 + 16.0 * s);
        let lit = i < pl.flask;
        let body = match (fblink, lit) {
            (true, _) => DENIED,
            (false, true) => FLASK_ON,
            (false, false) => HP_OFF,
        };
        draw_rectangle(lx, ly + 3.0 * s, 5.0 * s, 4.0 * s, body);
        draw_rectangle(lx + 1.0 * s, ly + 2.0 * s, 3.0 * s, 1.0 * s, body);
        if lit {
            let c = vec2(lx + 2.5 * s, ly + 0.5 * s);
            draw_triangle(
                c + vec2(0.0, -1.5 * s),
                c + vec2(1.3 * s, 1.5 * s),
                c + vec2(-1.3 * s, 1.5 * s),
                FLASK_FLAME,
            );
        }
    }
    // 잔불(0064 M1): 등잔 줄 아래에 호박 마름모 + 수. 주우면 잠깐 커진다. 하나도 없으면 그리지 않는다.
    if w.embers > 0 {
        let pop = 1.0 + 0.35 * (1.0 - ((w.time - w.ember_at) / EMBER_POP).clamp(0.0, 1.0));
        let c = vec2(x0 + 3.0 * s, y0 + 33.0 * s);
        let r = 3.0 * s * pop;
        draw_triangle(
            c + vec2(0.0, -r),
            c + vec2(r * 0.8, 0.0),
            c + vec2(0.0, r),
            EMBER,
        );
        draw_triangle(
            c + vec2(0.0, -r),
            c + vec2(-r * 0.8, 0.0),
            c + vec2(0.0, r),
            EMBER,
        );
        let k = 0.8 * pop.min(1.15);
        ui.text(
            &w.embers.to_string(),
            x0 + 9.0 * s,
            c.y - ui.px(k) * 0.55,
            k,
            EMBER,
        );
    }
    // 마법 칸(0048): 등잔 오른쪽에 둥근 칸 셋 — 찬 만큼 채워지고, 못 쏘면 깜빡인다. 근접으로 맞히면 찬다.
    // 원거리 무기(0051)는 그 자리에 탄창(세운 막대 — 장전 중이면 아래에서 차오른다) + 수류탄(둥근 칸).
    let mblink = pl.cast_denied > 0.0 && (w.time * 16.0) as i32 % 2 == 0;
    let mx0 = x0 + pl.flask_max.max(pl.flask) as f32 * 9.0 * s + 6.0 * s;
    let wpn = pl.weapon.def();
    let pip = |i: i32, fill: f32, col: Color| {
        let c = vec2(mx0 + 3.0 * s + i as f32 * 8.0 * s, y0 + 20.5 * s);
        draw_circle(c.x, c.y, 3.0 * s, if mblink { DENIED } else { HP_OFF });
        if fill > 0.0 && !mblink {
            let col = if fill >= 1.0 {
                col
            } else {
                Color::new(col.r, col.g, col.b, 0.55)
            };
            draw_circle(c.x, c.y, 3.0 * s * fill.sqrt(), col);
        }
    };
    match (wpn.rb, wpn.grenade) {
        (Rb::Shot { mag, reload, .. }, g) => {
            for i in 0..mag as i32 {
                let (bx, by) = (mx0 + i as f32 * 4.0 * s, y0 + 17.0 * s);
                draw_rectangle(bx, by, 2.5 * s, 7.0 * s, HP_OFF);
                let k = if (i as u8) < pl.ammo {
                    1.0
                } else if pl.reload > 0.0 {
                    1.0 - (pl.reload / reload.max(0.01)).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                if k > 0.0 {
                    draw_rectangle(bx, by + 7.0 * s * (1.0 - k), 2.5 * s, 7.0 * s * k, AMMO);
                }
            }
            if let Some(g) = g {
                let gx = mag as f32 * 4.0 * s + 4.0 * s;
                for i in 0..g.count as i32 {
                    let c = vec2(mx0 + gx + 3.0 * s + i as f32 * 8.0 * s, y0 + 20.5 * s);
                    draw_circle(c.x, c.y, 3.0 * s, if mblink { DENIED } else { HP_OFF });
                    let fill = (pl.grenades - i as f32).clamp(0.0, 1.0);
                    if fill > 0.0 && !mblink {
                        let col = if fill >= 1.0 {
                            AMMO
                        } else {
                            Color::new(AMMO.r, AMMO.g, AMMO.b, 0.5)
                        };
                        draw_circle(c.x, c.y, 3.0 * s * fill.sqrt(), col);
                    }
                }
            }
        }
        (Rb::Magic(_), _) => {
            for i in 0..CAST_CHARGES as i32 {
                pip(i, (pl.mana - i as f32).clamp(0.0, 1.0), MANA);
            }
        }
    }

    let Some(run) = run else { return };
    // 상자 곁(0064 M3): "키 열기"(쉼터보다 먼저 — 같은 버튼이 상자를 연다).
    if run.near_chest.is_some() && !under_menu {
        let key = crate::input::prompt(crate::input::Act::Interact);
        let verb = i18n::s().chest_verb;
        let text = if key.is_empty() {
            verb.to_string()
        } else {
            format!("{key}  {verb}")
        };
        ui.text_center(&text, f.ox + f.w * 0.5, f.oy + f.h * 0.78, 1.0, AMBER);
    } else if run.near_rest.is_some() && !under_menu {
        let y = f.oy + f.h * 0.78;
        let cx = f.ox + f.w * 0.5;
        if run.rest_safe {
            let key = crate::input::prompt(crate::input::Act::Interact);
            let text = if key.is_empty() {
                s_rest().to_string()
            } else {
                format!("{key}  {}", s_rest())
            };
            ui.text_center(&text, cx, y, 1.0, AMBER);
        } else {
            ui.text_center(i18n::s().rest_blocked, cx, y, 1.0, DIM);
        }
    }
    let cx = f.ox + f.w * 0.5;
    // 목표(오른쪽 위).
    if let Some(o) = run.objective {
        let text = format!("◇ {}", tr(o));
        let tw = ui.width(&text, 1.0);
        ui.text(&text, f.ox + f.w - tw - 8.0 * s, y0, 1.0, INK);
    }
    // 구역 카드(가운데 위): 이름 + 가는 줄.
    if let Some((name, left)) = run.card.filter(|_| !under_menu) {
        let a = fade_in_out(left, CARD_TIME.max(left), 0.5);
        let y = f.oy + f.h * 0.16;
        let text = tr(name);
        let col = Color::new(INK.r, INK.g, INK.b, a);
        ui.text_center(text, cx, y, 2.0, col);
        let tw = ui.width(text, 2.0);
        let ly = y + ui.px(2.0) * 1.15;
        draw_rectangle(
            cx - tw * 0.5,
            ly,
            tw,
            s,
            Color::new(AMBER.r, AMBER.g, AMBER.b, a * 0.7),
        );
    }
    // 시작 이야기(가운데): 스테이지 이름 + 두 줄.
    // 시작 이야기도 창 밑에서는 그리지 않는다(창 면이 비쳐 글이 겹쳤다, 0066).
    if run.intro > 0.0 && !under_menu {
        let a = fade_in_out(run.intro, INTRO_TIME, 0.8);
        let y = f.oy + f.h * 0.34;
        ui.text_center(
            tr(run.def.name),
            cx,
            y,
            2.0,
            Color::new(AMBER.r, AMBER.g, AMBER.b, a),
        );
        // 긴 언어에서도 화면 안에(nail `wrap`): 넘치면 줄을 바꿔 아래로 쌓는다.
        let (l1, l2) = run.def.intro;
        let max_w = f.w * 0.9;
        let y2 = ui.text_center_wrapped(
            tr(l1),
            cx,
            y + ui.px(2.0) * 1.6,
            1.0,
            Color::new(INK.r, INK.g, INK.b, a),
            max_w,
        );
        ui.text_center_wrapped(
            tr(l2),
            cx,
            y2 + ui.px(1.0) * 0.15,
            1.0,
            Color::new(DIM.r, DIM.g, DIM.b, a),
            max_w,
        );
    }
    // 보스 띠(아래 가운데): 이름 + 막대(깨어나면 색이 달아오른다).
    if let Some((k, roused)) = run.boss_bar(w).filter(|_| !under_menu) {
        let boss = run.boss.and_then(|i| w.enemies.get(i));
        let name = boss.map(|b| tr(b.def().name)).unwrap_or("");
        let bw = f.w * 0.5;
        let bh = 3.0 * s;
        let by = f.oy + f.h - 18.0 * s;
        let bx = cx - bw * 0.5;
        ui.text(name, bx, by - ui.px(1.0) - 2.0 * s, 1.0, INK);
        draw_rectangle(bx - s, by - s, bw + 2.0 * s, bh + 2.0 * s, BAR_BG);
        draw_rectangle(bx, by, bw * k, bh, if roused { BOSS_ROUSED } else { BOSS });
        // 자세 막대(0051·0055): 체력 띠 아래 가는 줄 — 차오를수록 밝게, 깨지면(폭딜 창) 금빛으로 깜빡인다.
        if let Some(b) = boss {
            let py = by + bh + 2.0 * s;
            let ph = 1.5 * s;
            draw_rectangle(bx - s, py - s * 0.5, bw + 2.0 * s, ph + s, BAR_BG);
            if b.broken > 0.0 {
                let on = (w.time * 10.0) as i32 % 2 == 0;
                draw_rectangle(bx, py, bw, ph, if on { POISE_BROKEN } else { POISE });
            } else {
                draw_rectangle(bx, py, bw * b.poise_frac, ph, POISE);
            }
        }
    }
}

/// 끝 화면 아래 줄: 걸린 시간·쓰러뜨린 수.
pub fn clear_line(run: &StageRun, w: &World) -> String {
    let secs = run.time as u32;
    let time = format!("{}:{:02}", secs / 60, secs % 60);
    i18n::fill(i18n::s().clear_sub, &[&time, &w.kills.to_string()])
}

/// 쉼터 동사(0032).
fn s_rest() -> &'static str {
    i18n::s().rest_verb
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cards_fade_in_and_out() {
        assert_eq!(fade_in_out(3.0, 3.0, 0.5), 0.0);
        assert_eq!(fade_in_out(1.5, 3.0, 0.5), 1.0);
        assert!(fade_in_out(0.1, 3.0, 0.5) < 0.3);
    }
}
