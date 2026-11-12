use super::*;

/// 0135: 한 그리기 호출은 정점과 색인 둘 다 한도 아래 — Meshy 모델(색인 ≈ 정점 × 2.4)을 정점만 보고 모으면 색인이 넘쳐 잘렸다.
#[test]
fn a_batch_counts_indices_too() {
    assert!(fits(0, 0, 3000, 7000));
    // 정점은 넉넉해도(6,000 + 3,000 < 10,000) 색인이 넘치면(14,000 + 7,000) 먼저 보낸다.
    assert!(!fits(6000, 14_000, 3000, 7000));
    assert!(!fits(9000, 0, 1000, 10), "vertices alone still count");
    // 모델 하나는 늘 빈 배치에 들어간다 — models.rs `every_bundled_model_fits_one_draw_call`.
}

/// 0145 더 큰 휘두름: 초승달은 머리에서 가장 두껍고 꼬리에서 0, 강타(`weight` > 1)가 더 두껍다 — 그래도 가운데는 비어 있다.
#[test]
fn the_slash_crescent_is_thickest_at_the_blade_and_never_fills_the_middle() {
    assert_eq!(slash_inner(0.0, SLASH_GLOW, 1.0), 1.0);
    let (light, heavy) = (
        slash_inner(1.0, SLASH_GLOW, 1.0),
        slash_inner(1.0, SLASH_GLOW, 1.5),
    );
    assert!(light < 0.5 && heavy < light, "{light} {heavy}");
    assert!(heavy >= 0.15 && slash_inner(1.0, SLASH_GLOW, 9.0) >= 0.15);
    // 밝은 날은 빛 띠보다 훨씬 가늘다(바깥 변을 따라가는 선).
    assert!(1.0 - slash_inner(1.0, SLASH_EDGE, 1.0) < (1.0 - light) * 0.4);
}

/// 0142 구운 불꽃: 밑동은 그릇에 붙어 가만히, 끝으로 갈수록 크게 일렁인다 — 몫은 늘 음수 쪽(풀의 느린 바람과 다른 박자)이고
/// 법선 칸에 실으면 0보다 크다(0은 "법선 없음"으로 읽힐 수 있다).
#[test]
fn a_baked_flame_flickers_at_the_tip_and_holds_at_the_bowl() {
    assert_eq!(flame_sway(1.0, 1.0, 1.6), 0.0);
    let (mid, tip) = (flame_sway(1.3, 1.0, 1.6), flame_sway(1.6, 1.0, 1.6));
    assert!(tip < mid && mid < 0.0, "{mid} {tip}");
    assert!(1.0 + tip > 0.0 && 1.0 + flame_sway(9.0, 1.0, 1.6) > 0.0);
    // 높이가 없는 부위(납작한 빛)도 나눗셈에서 터지지 않는다.
    assert!(flame_sway(1.0, 1.0, 1.0).is_finite());
}

/// 0142 뿌리 가시: 밑동은 땅(y = 0)에 붙고 위로 갈수록 가늘어지며 끝이 가장 높다 — 솟는 동안(높이 0에 가까워도) 면이 뒤집히지 않는다.
#[test]
fn a_root_spike_tapers_from_a_wide_foot_to_a_point() {
    let (body, tip) = spike_faces(0.2, 0.09, 1.0, 0.4, vec3(0.1, 0.0, 0.0));
    assert_eq!((body.len(), tip.len()), (8, 4));
    let foot: Vec<Vec3> = body
        .iter()
        .flatten()
        .copied()
        .filter(|p| p.y == 0.0)
        .collect();
    assert!(!foot.is_empty());
    for p in &foot {
        assert!((p.x.abs() - 0.2).abs() < 1e-6 && (p.z.abs() - 0.2).abs() < 1e-6);
    }
    let top = tip.iter().flatten().map(|p| p.y).fold(f32::MIN, f32::max);
    assert!((top - 1.4).abs() < 1e-6, "{top}");
    // 목은 밑동보다 가늘다(기운 만큼 옆으로 밀린 것은 빼고 잰다).
    for p in body.iter().flatten().filter(|p| p.y == 1.0) {
        assert!((p.x - 0.1).abs() <= 0.09 + 1e-6 && p.z.abs() <= 0.09 + 1e-6);
    }
    let (low, _) = spike_faces(0.2, 0.09, 0.02, 0.0, Vec3::ZERO);
    assert!(low.iter().flatten().all(|p| p.is_finite()));
}

/// 0142 줍는 것의 몸: 결정은 면 여덟·둥근 덩이는 면 서른둘, 꼭짓점은 준 치수 안에 있고(둥근 덩이는 모두 반지름 위)
/// 면이 찌그러지지 않았다 — 정육면체 대신 쓰는 꼴이라 크기가 어긋나면 빛기둥·발밑 고리와 안 맞는다.
#[test]
fn loot_bodies_are_a_crystal_and_a_round_bead_of_the_asked_size() {
    let octa = octa_faces(vec2(0.15, 0.1), 0.3, 0.2);
    assert_eq!(octa.len(), 8);
    for t in &octa {
        let area = (t[1] - t[0]).cross(t[2] - t[0]).length();
        assert!(area > 1e-4, "a collapsed face");
        for p in t {
            assert!(p.x.abs() <= 0.15 + 1e-6 && p.z.abs() <= 0.1 + 1e-6);
            assert!(p.y <= 0.3 + 1e-6 && p.y >= -0.2 - 1e-6);
        }
    }
    // 위·아래 길이가 다르면 위쪽이 더 길다(조각 — 종 꼴도 상자도 아니게).
    let top = octa.iter().flatten().map(|p| p.y).fold(f32::MIN, f32::max);
    let low = octa.iter().flatten().map(|p| p.y).fold(f32::MAX, f32::min);
    assert!((top - 0.3).abs() < 1e-6 && (low + 0.2).abs() < 1e-6);
    let ball = ball_faces(0.17);
    assert_eq!(ball.len(), 32);
    for t in &ball {
        assert!((t[1] - t[0]).cross(t[2] - t[0]).length() > 1e-4);
        for p in t {
            assert!((p.length() - 0.17).abs() < 1e-5, "off the sphere: {p}");
        }
    }
}

/// 0141 휘두름의 초승달: 가로 베기는 칼이 지나간 쪽에서 시작해 머리가 반대쪽 끝에 닿고(오른쪽 → 왼쪽 = 작은 각에서 큰 각으로),
/// 꼬리는 부채꼴 밖으로 나가지 않는다. 찌르기·세로는 가운데에서 양쪽으로 벌어진다.
#[test]
fn the_slash_crescent_follows_the_blade_and_stays_inside_the_arc() {
    use crate::weapons::Sweep;
    let (yaw, arc) = (0.3f32, 2.0f32);
    for q in [0.0, 0.3, 0.7, 1.0] {
        for sw in [Sweep::RightToLeft, Sweep::LeftToRight, Sweep::Center] {
            for (tail, head) in slash_arcs(yaw, arc, sw, q) {
                for a in [tail, head] {
                    assert!((a - yaw).abs() <= arc * 0.5 + 1e-4, "{sw:?} {q}: {a}");
                }
            }
        }
    }
    let r2l = |q| slash_arcs(yaw, arc, Sweep::RightToLeft, q)[0].1;
    assert!((r2l(0.0) - (yaw - 1.0)).abs() < 1e-5 && (r2l(1.0) - (yaw + 1.0)).abs() < 1e-5);
    let l2r = |q| slash_arcs(yaw, arc, Sweep::LeftToRight, q)[0].1;
    assert!(l2r(1.0) < l2r(0.0));
    let mid = slash_arcs(yaw, arc, Sweep::Center, 1.0);
    assert_eq!(mid.len(), 2);
    assert!(mid[0].0 == yaw && mid[1].0 == yaw && mid[0].1 > yaw && mid[1].1 < yaw);
}

fn with_key<T>(key: bool, f: impl FnOnce() -> T) -> T {
    let saved = look();
    set_look(Look {
        toon: true,
        key,
        ..saved
    });
    let out = f();
    set_look(saved);
    out
}

/// 0140 빛 대비: 달빛 받는 바닥의 밝기는 그대로 두고 그늘만 어둡게 — 빛 : 그늘이 2.2배를 넘는다(예전 1.8배).
/// 그늘은 검정이 아니라 남보라(파랑이 가장 크다, 0037).
#[test]
fn key_light_darkens_the_shade_not_the_lit_floor() {
    let floor = |key: bool, gate: f32| {
        with_key(key, || {
            MOON_GATE.with(|g| g.set(gate));
            let c = lit(Vec3::ZERO, Vec3::Y, &[]);
            MOON_GATE.with(|g| g.set(1.0));
            c
        })
    };
    let (old_lit, new_lit) = (floor(false, 1.0), floor(true, 1.0));
    let (old_shade, new_shade) = (floor(false, 0.0), floor(true, 0.0));
    assert!(
        (new_lit.y / old_lit.y - 1.0).abs() < 0.06,
        "{new_lit} vs {old_lit}"
    );
    assert!(old_lit.y / old_shade.y < 2.0);
    assert!(new_lit.y / new_shade.y > 2.2, "{new_lit} / {new_shade}");
    assert!(new_shade.z > new_shade.x && new_shade.z > new_shade.y);
}

/// 0140: 캐릭터 그림자(가림판)의 진하기 = 구운 그림자의 진하기 — 둘이 다르면 그림자가 두 가지로 보인다.
#[test]
fn the_actor_shadow_is_as_dark_as_the_baked_shadow() {
    for key in [false, true] {
        with_key(key, || {
            let open = lit(Vec3::ZERO, Vec3::Y, &[]);
            MOON_GATE.with(|g| g.set(0.0));
            let shade = lit(Vec3::ZERO, Vec3::Y, &[]);
            MOON_GATE.with(|g| g.set(1.0));
            let want = 1.0 - shade.y / open.y;
            assert!(
                (floor_shade() - want).abs() < 0.01,
                "key {key}: {}",
                floor_shade()
            );
        });
    }
}

/// 0140: 그늘에 선 배우는 달빛을 덜 받지만 다 잃지는 않는다(그늘 속 적도 읽혀야 한다), 가림 지도가 없으면(개발 아레나·인형) 그대로.
#[test]
fn an_actor_in_a_baked_shadow_gets_less_moon_but_never_none() {
    assert_eq!(actor_moon(Vec3::ZERO), 1.0);
    // 20m 마당, 달 쪽(-x, +z 앞)으로 높이 6m 벽 — 그 뒤(달 반대쪽)는 그늘.
    let mut o = crate::shade::Occluders::new(vec2(-10.0, -10.0), 0.25, 80, 80);
    for j in 0..80 {
        for i in 0..80 {
            o.raise(i, j, 0.0);
        }
    }
    let l = LIGHT_DIR.normalize();
    let wall = vec2(l.x, l.z).normalize() * 2.0;
    o.raise_box(wall - vec2(1.5, 1.5), wall + vec2(1.5, 1.5), 6.0);
    set_actor_shade(Some(std::rc::Rc::new(o)));
    let shaded = actor_moon(Vec3::ZERO);
    let open =
        actor_moon(vec3(-wall.x * 4.0, 0.0, -wall.y * 4.0) + vec3(wall.y, 0.0, -wall.x) * 3.0);
    set_actor_shade(None);
    assert_eq!(shaded, ACTOR_SHADE_MIN);
    assert_eq!(open, 1.0);
    assert!(ACTOR_SHADE_MIN >= 0.3);
}

/// 0124: 빛줄기 기둥 안에 선 몸은 밖보다 뚜렷이 밝고(땅속 달빛 0.4에서도), 위를 보는 면이 더 밝다 — 높이로 줄지 않는다.
#[test]
fn a_light_shaft_lights_whoever_stands_in_it() {
    set_moonlight(0.4);
    let shaft = crate::stage::Light {
        pos: vec3(0.0, 1.0, 0.0),
        color: Color::new(0.78, 0.82, 1.0, 1.0),
        radius: crate::stage::SHAFT_R,
        shaft: true,
    };
    let side = vec3(1.0, 0.0, 0.0);
    let sum = |v: Vec3| v.x + v.y + v.z;
    let inside = sum(lit(vec3(0.2, 1.2, 0.0), side, &[shaft]));
    let outside = sum(lit(vec3(3.0, 1.2, 0.0), side, &[shaft]));
    assert!(inside > outside * 1.5, "{inside} vs {outside}");
    let head = sum(lit(vec3(0.2, 1.8, 0.0), Vec3::Y, &[shaft]));
    assert!(head > inside, "the top catches more");
    let high = sum(lit(vec3(0.2, 4.0, 0.0), side, &[shaft]));
    assert!((high - inside).abs() < 1e-4, "a column, not a bulb");
    set_moonlight(1.0);
}

/// 0091(사장 "문은 투명화되지 않아서 시야를 가림"): 닫힌 문이 카메라와 플레이어 사이에 있으면 가림으로 잡힌다(지형 덩어리와 같은 판정),
/// 문 위쪽(높은 땅 위의 문 꼭대기)을 지나는 시선도. 다 열린 문은 상자가 없다.
#[test]
fn a_closed_gate_between_camera_and_player_counts_as_an_occluder() {
    let (mut run, _) = crate::stage::StageRun::start(&crate::stage::sanctuary::DEF);
    let (g, c) = run.level.gate.unwrap();
    let b = gate_box(&run).expect("closed at the start");
    // 문 안쪽 2m에 선 플레이어, 문 바깥 위에서 내려다보는 카메라(락온 숄더뷰가 문을 등지고 물러섰을 때).
    let player = vec3(c.x, c.y, g.min.z - 2.0);
    let cam = vec3(c.x, c.y + 5.0, g.max.z + 1.5);
    let pts = crate::camera::view_points(player, PLAYER_HEIGHT, PLAYER_RADIUS + 0.15, cam, None);
    assert!(!crate::camera::occluders(&pts, cam, &[b]).is_empty());
    // 문 꼭대기 바로 아래를 지나는 시선(땅 위 4.3m — 상자를 0에 두면 4.5m에서 끝나 놓쳤다): 상자가 땅에 붙어 있어야 잡힌다.
    let head = vec3(c.x, c.y + 4.3, g.min.z - 0.1);
    let low_cam = vec3(c.x, c.y + 4.3, g.max.z + 3.0);
    assert!(!crate::camera::occluders(&[head], low_cam, &[b]).is_empty());
    run.gate_open = GATE_TIME;
    assert!(gate_box(&run).is_none(), "an open gate hides nothing");
}

/// 0073: 예고 동안 동작은 정점까지 들어 올리고(`WINDUP_REACH`), 버티다가, 마지막 `WINDUP_SNAP`초에 내리쳐
/// 예고가 끝나는 틱(= 맞는 틱)에 내리친 자세 — 맞음이 그림보다 먼저 오지 않는다.
#[test]
fn a_windup_rises_holds_at_the_peak_then_snaps_down_on_the_hit_tick() {
    let (dur, peak, impact) = (0.9, 0.9, 1.0);
    assert_eq!(windup_clock(0.0, dur, dur, peak, impact), 0.0);
    assert!((windup_clock(dur * WINDUP_REACH, dur, dur, peak, impact) - peak).abs() < 1e-4);
    let hold = dur - WINDUP_SNAP - 0.01;
    assert!(
        (windup_clock(hold, dur, dur, peak, impact) - peak).abs() < 1e-4,
        "holds"
    );
    assert!(
        (windup_clock(dur, dur, dur, peak, impact) - impact).abs() < 1e-4,
        "struck on the hit tick"
    );
    // 늘 앞으로만(되감기 없음).
    let mut last = -1.0;
    for i in 0..=90 {
        let c = windup_clock(dur * i as f32 / 90.0, dur, dur, peak, impact);
        assert!(c >= last - 1e-5);
        last = c;
    }
    // 짧은 예고도 내리침 시간이 예고의 1/4을 넘지 않는다.
    assert!((windup_clock(0.2 - 0.05 - 1e-3, 0.2, 0.2, peak, impact) - peak).abs() < 0.02);
}

/// 0073: 내리치는 열쇠 표의 동작은 모델에 있고, 정점 < 맞음 < 동작 길이.
#[test]
fn strike_keys_name_real_clips_in_order() {
    for &(model, clip, peak, impact) in STRIKE_KEYS {
        let bytes = crate::models::bytes(model).unwrap_or_else(|| panic!("{model} missing"));
        let m = crate::model::Model::from_bytes(bytes).unwrap();
        let a = m
            .anim(clip)
            .unwrap_or_else(|| panic!("{model} has no {clip}"));
        let len = m.anims[a].duration;
        assert!(
            0.0 < peak && peak < impact && impact < len,
            "{model}/{clip}"
        );
    }
}

/// 2026-11-06: 예고는 땅 높낮이를 따라 그린다 — 높은 칸 위의 점은 그 칸 높이로, 벽·격자 밖은 적의 발 높이로.
#[test]
fn telegraphs_drape_over_the_ground_steps() {
    use crate::ground::{Cell, Ground};
    // 2×1 칸(칸 2m): 왼쪽 0m, 오른쪽 1m 턱. 가운데 = 원점.
    let g = Ground::new(2, 1, 2.0, vec![Cell::Flat(0.0), Cell::Flat(1.0)]);
    let d = Drape { ground: Some(&g) };
    assert!((d.at(-1.0, 0.0, 0.0, 0.0).y - 0.0).abs() < 1e-5);
    assert!(
        (d.at(1.0, 0.0, 0.0, 0.0).y - 1.0).abs() < 1e-5,
        "up on the step"
    );
    assert!(
        (d.at(9.0, 0.0, 0.3, 0.0).y - 0.3).abs() < 1e-5,
        "off the grid: the enemy's feet"
    );
    let flat = Drape { ground: None };
    assert!((flat.at(1.0, 0.0, 0.5, 0.02).y - 0.52).abs() < 1e-5);
    // 0083: 모든 바닥 표시가 이 길로 — 그림 가운데의 땅 위 높이를 지킨다(발밑 고리 0, 가슴 높이 휘두름 0.9).
    assert!(
        d.rise(vec3(-1.0, 0.0, 0.0)).abs() < 1e-5,
        "a mark at the feet"
    );
    assert!(
        (d.rise(vec3(-1.0, 0.9, 0.0)) - 0.9).abs() < 1e-5,
        "a swing arc at the chest"
    );
    assert!(
        (d.rise(vec3(1.0, 1.05, 0.0)) - 0.05).abs() < 1e-5,
        "standing on the step"
    );
    assert!(
        flat.rise(vec3(0.0, 0.5, 0.0)).abs() < 1e-5,
        "no ground: draw at c.y as before"
    );
    // 0.05 위 고리의 가운데가 낮은 칸이면, 턱 위로 뻗은 쪽은 턱 높이 + 0.05(예전엔 발 높이에 평평하게 그려 턱 속에 묻혔다).
    let up = d.rise(vec3(-1.0, 0.05, 0.0));
    assert!((d.at(1.0, 0.0, 0.05, up).y - 1.05).abs() < 1e-5);
}

/// 0046: 흔한 화면은 테두리 없이 정수 배율로 꽉 차고, 브라우저 창(주소창만큼 낮은 높이)도 테두리가 배율보다 작다.
#[test]
fn haze_is_full_looking_down_in_the_quarter_view_and_gone_in_the_shoulder_view() {
    let quarter = -(35f32.to_radians().sin());
    let shoulder = -(19f32.to_radians().sin());
    assert_eq!(haze_by_pitch(quarter), 1.0);
    assert_eq!(haze_by_pitch(shoulder), 0.0);
    let mid = haze_by_pitch((quarter + shoulder) * 0.5);
    assert!(mid > 0.0 && mid < 1.0);
}

#[test]
fn the_internal_resolution_fills_the_screen_with_integer_pixels() {
    assert_eq!(fit_internal(1920.0, 1080.0), (640, 360, 3));
    assert_eq!(fit_internal(1280.0, 720.0), (640, 360, 2));
    assert_eq!(fit_internal(3840.0, 2160.0), (640, 360, 6));
    // 0041의 고정 640×360은 이 창에서 ×2(1280×720)라 둘레가 넓게 검었다.
    let (w, h, s) = fit_internal(1920.0, 969.0);
    assert_eq!(s, 3);
    assert!(1920 - w * s < s && 969 - h * s < s, "{w}×{h}×{s}");
    // 20:9 폰·16:10 핸드헬드는 옆·위아래를 더 본다(세로 시야각 그대로).
    assert_eq!(fit_internal(2400.0, 1080.0), (800, 360, 3));
    assert_eq!(fit_internal(1920.0, 1200.0), (640, 400, 3));
    // 세로 폰은 4:3까지만 — 나머지는 테두리.
    let (w, h, _) = fit_internal(1170.0, 2532.0);
    assert!(h * 4 <= w * 3 + 4, "{w}×{h}");
    // 아주 작은 창도 0이 되지 않는다.
    assert!(fit_internal(10.0, 10.0).0 >= 1);
}

#[test]
fn a_spilled_enemy_leans_the_way_it_slides_then_rocks_back_and_settles() {
    let sway = vec3(1.0, 0.0, 0.0);
    // 처음엔 미끄러지는 쪽으로 크게(머리가 그쪽으로).
    let peak = (1..30)
        .map(|i| lean(sway, i as f32 / 60.0, 0.0, 0.0).x)
        .fold(f32::MIN, f32::max);
    assert!(peak > 0.3, "visible lean: {peak} rad");
    // 그다음 반대로 한 번 흔들린다.
    let back = (15..60)
        .map(|i| lean(sway, i as f32 / 60.0, 0.0, 0.0).x)
        .fold(f32::MAX, f32::min);
    assert!(back < -0.1, "rocks back: {back}");
    // 휘청이 끝날 즈음엔 거의 서 있다.
    assert!(lean(sway, 1.0, 0.0, 0.0).length() < 0.05);
    // 세기가 약하면(가까운 적·보스) 작게.
    let small = (1..30)
        .map(|i| lean(sway * 0.35, i as f32 / 60.0, 0.0, 0.0).x)
        .fold(f32::MIN, f32::max);
    assert!(small < peak * 0.5);
}
