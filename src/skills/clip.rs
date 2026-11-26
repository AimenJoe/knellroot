//! 스킬 · 궁극기의 전용 동작 `Clip`(0158 G2 — 구간의 경계 = 판정의 순간).

/// 휘돌이의 전용 동작이 몸을 도는 쪽(+1 = 요가 느는 쪽 · −1 = 주는 쪽): 이펙트의 도는 초승달이 이 쪽으로 바퀴마다 한 바퀴 돈다.
/// 0193 스킨 몸의 원본(Meshy Skill_03)은 오른쪽으로 돈다 — `wanderer_skin.py` `SPIN`과 같아야 한다(시험).
pub const WHIRL_CLIP_TURN: f32 = -1.0;

/// 스킬 전용 동작(0158 G2, `wanderer.glb`의 `sk_*`·`ult_*`): 이름과 구간 셋(시동 · 이어짐 · 후딜)의 길이(초 — 바탕값으로 만든 것).
/// 구간의 경계가 판정의 순간이다. 모델에 아직 없으면 렌더가 표의 `anim`(빌려 쓰던 동작)으로 넘어간다.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Clip {
    pub name: &'static str,
    pub seg: [f32; 3],
}

impl Clip {
    /// 상태의 시각 `e`(초) → 동작의 시각. `startup`·`active`·`recover` = 지금 단·고르기의 실제 길이.
    /// 시동은 시동 구간에 맞춰 늘이고(경계 = 판정의 순간이 어긋나지 않게), 이어짐은 `period`가 있으면 그 주기로 구간을 되풀이
    /// (휘돌이의 바퀴·되돌아 베기의 둘째 길), 없으면 구간 전체에 늘인다. 후딜은 제 빠르기로 — 길면 끝 자세에 머문다.
    pub fn at(&self, e: f32, startup: f32, active: f32, period: Option<f32>) -> f32 {
        let [s, a, r] = self.seg;
        let e = e.max(0.0);
        if e < startup {
            return s * (e / startup.max(1e-4));
        }
        let q = e - startup;
        if q < active {
            let k = match period {
                Some(p) if p > 1e-4 => (q / p).fract(),
                _ => q / active.max(1e-4),
            };
            return s + a * k;
        }
        s + a + (q - active).min(r)
    }

    /// 동작 전체의 길이(초) — 시험이 읽는다.
    #[cfg(test)]
    pub fn total(&self) -> f32 {
        self.seg.iter().sum()
    }
}

#[cfg(test)]
mod tests {

    use crate::skills::*;

    /// 0158 G2 동작: 상태의 시각 → 동작의 시각은 구간으로 맞춘다 — 시동의 끝은 늘 시동 구간의 끝(판정의 순간), 이어짐이
    /// 길어지면(셋째 바퀴·되돌아 베기) 이어짐 구간을 되풀이하고, 후딜은 후딜 구간에서 시작해 동작의 끝을 넘지 않는다.
    #[test]
    fn a_clip_maps_each_phase_onto_its_own_segment_whatever_the_rank() {
        let near = |a: f32, b: f32| (a - b).abs() < 1e-4;
        for id in IDS {
            for (rank, pick) in [(1, Pick::A), (MAX_RANK, Pick::A), (MAX_RANK, Pick::B)] {
                let e = id.def().at(rank, pick);
                let (c, s) = (e.skill.clip, e.skill);
                let period = match s.form {
                    Form::Ring { gap, .. } => Some(gap),
                    Form::Lane { time, .. } => Some(time),
                    _ => None,
                };
                let at = |t: f32| c.at(t, s.startup, e.active(), period);
                let tag = format!("{id:?} {rank} {pick:?}");
                assert!(near(at(0.0), 0.0), "{tag}: starts at 0");
                // 시동의 끝 = 시동 구간의 끝.
                assert!(
                    near(at(s.startup - 1e-5), c.seg[0]),
                    "{tag}: {}",
                    at(s.startup - 1e-5)
                );
                // 이어짐 안은 이어짐 구간 안.
                if e.active() > 0.0 {
                    for k in [0.1, 0.5, 0.9] {
                        let t = at(s.startup + e.active() * k);
                        assert!(
                            t >= c.seg[0] - 1e-4 && t <= c.seg[0] + c.seg[1] + 1e-4,
                            "{tag}: active at {t}"
                        );
                    }
                }
                // 후딜의 시작 = 후딜 구간의 시작, 끝은 동작의 끝을 넘지 않는다.
                let rec = s.startup + e.active();
                assert!(near(at(rec), c.seg[0] + c.seg[1]), "{tag}: {}", at(rec));
                assert!(near(at(rec + 0.1), c.seg[0] + c.seg[1] + 0.1), "{tag}");
                assert!(near(at(rec + 9.0), c.total()), "{tag}: holds the last pose");
                // 뒤로 가지 않는다(되풀이하는 이어짐 안에서만 감긴다).
                assert!(at(rec + 0.05) > at(rec));
            }
        }
        // 휘돌이의 셋째 바퀴: 이어짐 구간(한 바퀴)을 바퀴마다 한 번씩 — 둘째 바퀴의 가운데 = 구간의 가운데.
        let w = Id::Whirl.def().at(3, Pick::B);
        let (c, s) = (w.skill.clip, w.skill);
        assert!(near(w.active(), 0.44));
        let mid = c.at(s.startup + 0.22 + 0.11, s.startup, w.active(), Some(0.22));
        assert!(near(mid, c.seg[0] + c.seg[1] * 0.5), "{mid}");
        // 큰 기술: 세계 멈춤은 시동 구간에, 그 뒤 터지기까지는 이어짐 구간에 — 터지는 순간 = 이어짐 구간의 끝.
        for u in [Ult::Upend, Ult::Line] {
            let (c, (_, startup, _)) = (u.clip(), u.times());
            assert!(near(u.clip_at(0.0, 0.0), 0.0), "{u:?}");
            assert!(near(u.clip_at(0.0, 0.5), c.seg[0] * 0.5));
            assert!(near(u.clip_at(0.0, 1.0), c.seg[0]));
            assert!(
                near(u.clip_at(startup - 1e-5, 1.0), c.seg[0] + c.seg[1]),
                "{u:?}"
            );
            assert!(near(
                u.clip_at(startup + 0.1, 1.0),
                c.seg[0] + c.seg[1] + 0.1
            ));
            assert!(near(u.clip_at(99.0, 1.0), c.total()));
        }
        // 몰아치기는 멈춤이 없다: 시동이 곧 시동 구간.
        let c = FLURRY.clip;
        assert!(near(
            Ult::Flurry.clip_at(FLURRY.startup * 0.5, 0.0),
            c.seg[0] * 0.5
        ));
        assert!(near(Ult::Flurry.clip_at(FLURRY.startup, 0.0), c.seg[0]));
    }
}

#[cfg(test)]
mod clip_facing {
    use crate::det::Det;
    use crate::model::Model;
    use macroquad::prelude::*;

    /// 0193: 기술 동작의 몸 방향(`wanderer_skin.py` `SKILL_FACE`)과 같은 표 — 판정의 순간에 무엇이 정면이어야 하나.
    enum Face {
        Tip,
        Hips,
        Hold,
        Spin,
        /// 활을 겨누는 동작(0272 — 화살비): 겨누는 줄(오른손 → 활을 든 왼손)이 정면 — 몸은 옆으로 선다.
        Aim,
    }
    const FACES: [(&str, Face); 45] = [
        ("sk_whirl", Face::Spin),
        ("sk_cut", Face::Tip),
        ("sk_gather", Face::Hips),
        ("sk_quake", Face::Tip),
        ("sk_brace", Face::Hold),
        ("sk_echo", Face::Hips),
        ("sk_throw", Face::Tip),
        ("sk_embers", Face::Hips),
        ("ult_upend", Face::Hips),
        ("ult_flurry", Face::Hips),
        ("ult_line", Face::Tip),
        ("sk_swallow", Face::Tip),
        ("sk_bash", Face::Hips),
        ("sk_roots", Face::Tip),
        ("sk_stitch", Face::Tip),
        ("sk_ram", Face::Hold),
        ("sk_toll", Face::Tip),
        ("sk_swat", Face::Tip),
        ("sk_ripple", Face::Hips),
        ("sk_snow", Face::Hips),
        ("sk_flare", Face::Hips),
        ("sk_snare", Face::Hips),
        ("ult_bell", Face::Hips),
        ("ult_blizzard", Face::Hips),
        // 0272: 쇠뇌 판의 둘 · 활 판의 큰 기술(겨누는 줄 — 활을 든 왼손이 정면).
        ("sk_fuse", Face::Hips),
        ("sk_mine", Face::Hips),
        ("ult_rain", Face::Aim),
        // 0272 2단계: 창 판의 셋 · 질풍 찌르기(세계가 멈춘 동안 당긴 채 — 몸이 정면).
        ("sk_skewer", Face::Tip),
        ("sk_pin", Face::Tip),
        ("sk_pole", Face::Tip),
        ("ult_gale", Face::Hips),
        // 0274 한손검.
        ("sk_pommel", Face::Tip),
        ("sk_shove", Face::Hips),
        ("sk_rally", Face::Hold),
        // 0274 ③ 쌍단검.
        ("sk_slip", Face::Tip),
        ("ult_ambush", Face::Hips),
        // 0274 ④ 대검.
        ("sk_sunder", Face::Tip),
        ("sk_sear", Face::Tip),
        ("ult_tempest", Face::Spin),
        // 0274 ⑤ 낫.
        ("sk_reap", Face::Hips),
        ("sk_harvest", Face::Spin),
        ("ult_grove", Face::Hips),
        // 0274 ⑥ 완드.
        ("sk_hex", Face::Hips),
        ("sk_blight", Face::Hips),
        ("ult_price", Face::Hips),
    ];

    fn deg(a: f32) -> f32 {
        (a.to_degrees() + 540.0).rem_euclid(360.0) - 180.0
    }

    /// 기술을 누르는 순간·끝나 서기로 돌아갈 때 몸이 홱 돌지 않고(정면 ±25°), 판정의 순간에 칼끝·몸이 정면이며,
    /// 휘돌이는 이어짐 구간(바퀴마다 되풀이되는 몫)에 이펙트와 같은 쪽으로 한 바퀴 돈다.
    #[test]
    fn skill_clips_start_and_end_facing_front_and_hit_in_front() {
        let m = Model::from_bytes(include_bytes!("../../assets/models/wanderer.glb")).unwrap();
        let (hips, sword, grip) = (
            m.node("mixamorig:Hips").unwrap(),
            m.node("sword").unwrap(),
            m.node("weapon").unwrap(),
        );
        let mut rest = Vec::new();
        m.pose(None, 0.0, &mut rest);
        let g = rest[grip].w_axis.truncate();
        let far = m.meshes[m.nodes[sword].mesh.unwrap()]
            .pos
            .iter()
            .copied()
            .max_by(|a, b| {
                let d = |p: Vec3| rest[sword].transform_point3(p).distance(g);
                d(*a).total_cmp(&d(*b))
            })
            .unwrap();
        let clips = crate::skills::SKILLS
            .iter()
            .map(|s| s.clip)
            .chain(crate::skills::ULTS.map(|u| u.clip()));
        let mut pose = Vec::new();
        let mut aim = Vec::new();
        let mut body = |ai, t: f32| {
            m.pose(Some(ai), t, &mut pose);
            let z = pose[hips].z_axis.truncate();
            let p = pose[sword].transform_point3(far);
            (z.x.atan2_d(z.z), p.x.atan2_d(p.z))
        };
        for clip in clips {
            let [s, a, r] = clip.seg;
            let ai = m.anim(clip.name).unwrap_or_else(|| panic!("{}", clip.name));
            let face = &FACES.iter().find(|(n, _)| *n == clip.name).unwrap().1;
            for t in [0.0, s + a + r - 0.005] {
                let y = deg(body(ai, t).0);
                assert!(
                    y.abs() < 25.0,
                    "{}: the body faces front at {t:.2}s ({y:.0}°)",
                    clip.name
                );
            }
            let (h, tip) = body(ai, s);
            match face {
                Face::Tip => assert!(
                    deg(tip).abs() < 25.0,
                    "{}: the blade is in front at the hit ({:.0}°)",
                    clip.name,
                    deg(tip)
                ),
                Face::Hips => assert!(
                    deg(h).abs() < 25.0,
                    "{}: the body faces front at the hit ({:.0}°)",
                    clip.name,
                    deg(h)
                ),
                Face::Aim => {
                    let (rh, lh) = (
                        m.node("mixamorig:RightHand").unwrap(),
                        m.node("mixamorig:LeftHand").unwrap(),
                    );
                    for t in [s, s + a - 0.005] {
                        m.pose(Some(ai), t, &mut aim);
                        let d = aim[lh].w_axis.truncate() - aim[rh].w_axis.truncate();
                        let y = deg(d.x.atan2_d(d.z));
                        assert!(
                            y.abs() < 25.0,
                            "{}: the bow points ahead from the draw to the loose ({y:.0}°)",
                            clip.name
                        );
                    }
                }
                Face::Hold => {
                    for t in [s, s + a - 0.005] {
                        let y = deg(body(ai, t).0);
                        assert!(
                            y.abs() < 25.0,
                            "{}: the body faces front while holding ({y:.0}°)",
                            clip.name
                        );
                    }
                }
                Face::Spin => {
                    let mut turned = 0.0;
                    let mut last = body(ai, s).0;
                    for j in 1..=20 {
                        let y = body(
                            ai,
                            s + a * j as f32 / 20.0 - if j == 20 { 0.005 } else { 0.0 },
                        )
                        .0;
                        turned += deg(y - last);
                        last = y;
                    }
                    let want = crate::skills::WHIRL_CLIP_TURN * 360.0;
                    assert!(
                        (turned - want).abs() < 30.0,
                        "{}: one turn per beat, the way the effect turns ({turned:.0}°)",
                        clip.name
                    );
                }
            }
        }
    }
}
