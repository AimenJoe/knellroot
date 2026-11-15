//! glTF 읽기(P4 스파이크, 0013): 부위 강체 모델 — 노드 트리(TRS) + 노드마다 메시 + 노드 TRS 애니메이션 채널.
//! 0193부터 스킨도 읽는다(뼈 = 노드, 정점마다 뼈 넷·무게 — `Model::skin_into`가 CPU에서 섞는다). 스킨이 없는 모델은 예전 그대로. macroquad에는 모델 로더가 없어(0.4.16 소스로 확인)
//! 여기서 읽고, 렌더가 CPU에서 행렬을 곱해 정점을 한 배치로 보낸다(`render::ModelBatch`).
//! 이 모듈은 그림 핸들을 모른다 — 텍스처는 PNG 바이트로만 들고 있고 렌더가 한 번 올린다(로직·시험이 GL 없이 돈다).
use base64::Engine;
use macroquad::prelude::*;

#[derive(Clone, Debug)]
pub struct Node {
    /// Blockbench 그룹 이름(시험·디버그에서 부위를 찾는다).
    pub name: String,
    pub parent: Option<usize>,
    pub t: Vec3,
    pub r: Quat,
    pub s: Vec3,
    pub mesh: Option<usize>,
    /// 이 노드의 메시가 스킨이면 그 스킨(0193) — 노드 자신의 행렬은 쓰지 않고 뼈 행렬로 섞는다(glTF 약속).
    pub skin: Option<usize>,
}

/// 스킨(0193): 뼈 노드들과 그 바인드의 역행렬(메시 공간 → 뼈 공간).
#[derive(Clone, Debug, Default)]
pub struct Skin {
    pub joints: Vec<usize>,
    pub ibm: Vec<Mat4>,
}

#[derive(Clone, Debug, Default)]
pub struct MeshData {
    pub pos: Vec<Vec3>,
    pub nrm: Vec<Vec3>,
    pub uv: Vec<Vec2>,
    pub idx: Vec<u32>,
    /// 스킨 메시면 정점마다 뼈 넷(스킨의 `joints` 자리)과 무게(0193). 아니면 비어 있다.
    pub joints: Vec<[u16; 4]>,
    pub weights: Vec<[f32; 4]>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Prop {
    T,
    R,
    S,
}

#[derive(Clone, Debug)]
pub struct Channel {
    pub node: usize,
    pub prop: Prop,
    pub times: Vec<f32>,
    /// T·S는 xyz(w 무시), R은 쿼터니언 xyzw.
    pub values: Vec<Vec4>,
    pub step: bool,
}

#[derive(Clone, Debug)]
pub struct Anim {
    pub name: String,
    pub duration: f32,
    pub channels: Vec<Channel>,
}

#[derive(Clone, Debug)]
pub struct Model {
    pub nodes: Vec<Node>,
    pub meshes: Vec<MeshData>,
    pub anims: Vec<Anim>,
    pub skins: Vec<Skin>,
    /// 부모가 자식보다 먼저 오는 순서(자세 계산용).
    pub order: Vec<usize>,
    pub texture_png: Option<Vec<u8>>,
}

fn data_uri(uri: &str) -> Result<Vec<u8>, String> {
    let (_, b64) = uri
        .split_once(";base64,")
        .ok_or_else(|| format!("외부 파일 URI는 아직 안 읽는다(내장 data URI만): {uri:.40}"))?;
    base64::engine::general_purpose::STANDARD
        .decode(b64)
        .map_err(|e| e.to_string())
}

impl Model {
    /// `.gltf`(data URI 내장) 또는 `.glb` 바이트에서.
    pub fn from_bytes(bytes: &[u8]) -> Result<Model, String> {
        let g = gltf::Gltf::from_slice(bytes).map_err(|e| e.to_string())?;
        let mut buffers: Vec<Vec<u8>> = Vec::new();
        for b in g.document.buffers() {
            buffers.push(match b.source() {
                gltf::buffer::Source::Bin => g.blob.clone().ok_or("glb에 BIN 덩어리가 없다")?,
                gltf::buffer::Source::Uri(u) => data_uri(u)?,
            });
        }
        let get = |b: gltf::Buffer| buffers.get(b.index()).map(|v| v.as_slice());

        let mut nodes: Vec<Node> = g
            .document
            .nodes()
            .map(|n| {
                let (t, r, s) = n.transform().decomposed();
                Node {
                    name: n.name().unwrap_or("").to_string(),
                    parent: None,
                    t: Vec3::from(t),
                    r: Quat::from_array(r),
                    s: Vec3::from(s),
                    mesh: n.mesh().map(|m| m.index()),
                    skin: n.skin().map(|k| k.index()),
                }
            })
            .collect();
        for n in g.document.nodes() {
            for c in n.children() {
                nodes[c.index()].parent = Some(n.index());
            }
        }

        let mut meshes = Vec::new();
        for m in g.document.meshes() {
            let mut out = MeshData::default();
            for p in m.primitives() {
                let r = p.reader(get);
                let base = out.pos.len() as u32;
                let pos: Vec<Vec3> = r
                    .read_positions()
                    .ok_or("POSITION 없음")?
                    .map(Vec3::from)
                    .collect();
                let n = pos.len();
                out.nrm.extend(
                    r.read_normals()
                        .map(|it| it.map(Vec3::from).collect())
                        .unwrap_or_else(|| vec![Vec3::Y; n]),
                );
                out.uv.extend(
                    r.read_tex_coords(0)
                        .map(|it| it.into_f32().map(Vec2::from).collect())
                        .unwrap_or_else(|| vec![Vec2::ZERO; n]),
                );
                if let (Some(j), Some(w)) = (r.read_joints(0), r.read_weights(0)) {
                    out.joints.extend(j.into_u16());
                    out.weights.extend(w.into_f32());
                }
                out.pos.extend(pos);
                match r.read_indices() {
                    Some(ix) => out.idx.extend(ix.into_u32().map(|i| base + i)),
                    None => out.idx.extend(base..base + n as u32),
                }
            }
            meshes.push(out);
        }

        let mut anims = Vec::new();
        for a in g.document.animations() {
            let mut channels = Vec::new();
            let mut duration = 0.0f32;
            for c in a.channels() {
                let r = c.reader(get);
                let times: Vec<f32> = r.read_inputs().ok_or("애니메이션 입력 없음")?.collect();
                use gltf::animation::util::ReadOutputs;
                let (prop, mut values): (Prop, Vec<Vec4>) =
                    match r.read_outputs().ok_or("애니메이션 출력 없음")? {
                        ReadOutputs::Translations(it) => {
                            (Prop::T, it.map(|v| Vec3::from(v).extend(0.0)).collect())
                        }
                        ReadOutputs::Rotations(it) => {
                            (Prop::R, it.into_f32().map(Vec4::from).collect())
                        }
                        ReadOutputs::Scales(it) => {
                            (Prop::S, it.map(|v| Vec3::from(v).extend(0.0)).collect())
                        }
                        ReadOutputs::MorphTargetWeights(_) => continue,
                    };
                let interp = c.sampler().interpolation();
                if interp == gltf::animation::Interpolation::CubicSpline {
                    // (in-tangent, 값, out-tangent) 셋씩 — 값만 남기고 선형으로 근사.
                    values = values.chunks(3).map(|k| k[1]).collect();
                }
                duration = duration.max(times.last().copied().unwrap_or(0.0));
                channels.push(Channel {
                    node: c.target().node().index(),
                    prop,
                    times,
                    values,
                    step: interp == gltf::animation::Interpolation::Step,
                });
            }
            anims.push(Anim {
                name: a.name().unwrap_or("").to_string(),
                duration,
                channels,
            });
        }

        let mut skins = Vec::new();
        for k in g.document.skins() {
            let joints: Vec<usize> = k.joints().map(|j| j.index()).collect();
            let ibm: Vec<Mat4> = match k.reader(get).read_inverse_bind_matrices() {
                Some(it) => it.map(|m| Mat4::from_cols_array_2d(&m)).collect(),
                None => vec![Mat4::IDENTITY; joints.len()],
            };
            skins.push(Skin { joints, ibm });
        }

        let texture_png = match g.document.images().next().map(|i| i.source()) {
            Some(gltf::image::Source::Uri { uri, .. }) => Some(data_uri(uri)?),
            Some(gltf::image::Source::View { view, .. }) => {
                let b = &buffers[view.buffer().index()];
                Some(b[view.offset()..view.offset() + view.length()].to_vec())
            }
            None => None,
        };

        // 부모 먼저: 뿌리에서 깊이 우선.
        let mut order = Vec::with_capacity(nodes.len());
        let mut stack: Vec<usize> = (0..nodes.len())
            .filter(|&i| nodes[i].parent.is_none())
            .rev()
            .collect();
        while let Some(i) = stack.pop() {
            order.push(i);
            for c in (0..nodes.len()).rev() {
                if nodes[c].parent == Some(i) {
                    stack.push(c);
                }
            }
        }
        Ok(Model {
            nodes,
            meshes,
            anims,
            skins,
            order,
            texture_png,
        })
    }

    pub fn anim(&self, name: &str) -> Option<usize> {
        self.anims.iter().position(|a| a.name == name)
    }

    /// 애니메이션 `anim`의 `t`초(고리) 자세 → 노드마다 모델 공간 행렬.
    pub fn pose(&self, anim: Option<usize>, t: f32, out: &mut Vec<Mat4>) {
        self.pose_with(anim, t, &[], out);
    }

    /// `pose` + 덧붙이기(0039): `extra`의 노드는 자기 피벗(노드 원점)을 축으로 **모델 공간**에서 돌리고 키운다
    /// (자식도 따라간다). 키프레임 동작 위에 얹는 절차 층(`secondary.rs`) — 동작 표는 그대로 둔다.
    pub fn pose_with(
        &self,
        anim: Option<usize>,
        t: f32,
        extra: &[(usize, Quat, Vec3)],
        out: &mut Vec<Mat4>,
    ) {
        let local = self.locals(anim, t);
        self.pose_from_locals(&local, extra, out);
    }

    /// 동작 `anim`의 `t`초 자세를 노드마다의 (자리, 돌림, 크기)로 — 둘을 섞을 때(0175 걷기 섞임) 이것을 섞고 `pose_from_locals`로.
    pub fn locals(&self, anim: Option<usize>, t: f32) -> Vec<(Vec3, Quat, Vec3)> {
        let mut local: Vec<(Vec3, Quat, Vec3)> =
            self.nodes.iter().map(|n| (n.t, n.r, n.s)).collect();
        if let Some(a) = anim.and_then(|i| self.anims.get(i)) {
            let tt = if a.duration > 0.0 {
                t.rem_euclid(a.duration)
            } else {
                0.0
            };
            for c in &a.channels {
                let v = sample(c, tt);
                let l = &mut local[c.node];
                match c.prop {
                    Prop::T => l.0 = v.truncate(),
                    Prop::R => l.1 = Quat::from_vec4(v).normalize(),
                    Prop::S => l.2 = v.truncate(),
                }
            }
        }
        local
    }

    /// 노드마다의 자세(`locals`)에서 노드의 세계 행렬로. `extra` = 노드마다 덧붙이는 돌림·크기(2차 동작·숨긴 무기).
    pub fn pose_from_locals(
        &self,
        local: &[(Vec3, Quat, Vec3)],
        extra: &[(usize, Quat, Vec3)],
        out: &mut Vec<Mat4>,
    ) {
        out.clear();
        out.resize(self.nodes.len(), Mat4::IDENTITY);
        for &i in &self.order {
            let (t, r, s) = local[i];
            let m = Mat4::from_scale_rotation_translation(s, r, t);
            out[i] = match self.nodes[i].parent {
                Some(p) => out[p] * m,
                None => m,
            };
            for &(n, q, k) in extra {
                if n == i {
                    let pivot = out[i].w_axis.truncate();
                    out[i] = Mat4::from_translation(pivot)
                        * Mat4::from_scale_rotation_translation(k, q, Vec3::ZERO)
                        * Mat4::from_translation(-pivot)
                        * out[i];
                }
            }
        }
    }

    /// 스킨 메시 노드 `i`를 자세 `pose`로 섞은 정점·법선(모델 공간, 0193 — 정점마다 Σ 무게 × 뼈 행렬 × 바인드 역행렬).
    /// 스킨이 아니면 거짓(부르는 쪽이 노드 행렬로 그린다).
    pub fn skin_into(
        &self,
        i: usize,
        pose: &[Mat4],
        pos: &mut Vec<Vec3>,
        nrm: &mut Vec<Vec3>,
    ) -> bool {
        let (Some(k), Some(mi)) = (self.nodes[i].skin, self.nodes[i].mesh) else {
            return false;
        };
        let (skin, mesh) = (&self.skins[k], &self.meshes[mi]);
        if mesh.joints.len() != mesh.pos.len() {
            return false;
        }
        let mats: Vec<Mat4> = skin
            .joints
            .iter()
            .zip(&skin.ibm)
            .map(|(&j, ibm)| pose.get(j).copied().unwrap_or(Mat4::IDENTITY) * *ibm)
            .collect();
        pos.clear();
        nrm.clear();
        for v in 0..mesh.pos.len() {
            let (js, ws) = (mesh.joints[v], mesh.weights[v]);
            let mut m = Mat4::ZERO;
            let mut total = 0.0;
            for c in 0..4 {
                if ws[c] > 0.0 {
                    if let Some(jm) = mats.get(js[c] as usize) {
                        m += *jm * ws[c];
                        total += ws[c];
                    }
                }
            }
            if total <= 1e-6 {
                m = Mat4::IDENTITY;
            } else if (total - 1.0).abs() > 1e-3 {
                m *= 1.0 / total;
            }
            pos.push(m.transform_point3(mesh.pos[v]));
            nrm.push(m.transform_vector3(mesh.nrm[v]).normalize_or_zero());
        }
        true
    }

    pub fn node(&self, name: &str) -> Option<usize> {
        self.nodes.iter().position(|n| n.name == name)
    }

    /// 노드 `n`이 `i` 자신이거나 그 아래인가.
    pub fn is_under(&self, mut n: usize, i: usize) -> bool {
        loop {
            if n == i {
                return true;
            }
            match self.nodes[n].parent {
                Some(p) => n = p,
                None => return false,
            }
        }
    }

    /// 쉬는 자세에서 노드 `i` 아래 메시 정점의 경계 상자 가운데와 그 가운데에서 가장 먼 점까지(모델 공간) — 아이템 그림(0068)의 틀 맞추기.
    /// 쉬는 자세에서 마디 `i`의 월드 회전 — 손뼈에 붙은 무기를 제 모양의 축으로 되돌려 보일 때(아이콘·던진 무기, 0193).
    pub fn rest_rotation(&self, i: usize) -> Quat {
        let mut pose = Vec::new();
        self.pose(None, 0.0, &mut pose);
        pose.get(i)
            .map_or(Quat::IDENTITY, |m| m.to_scale_rotation_translation().1)
    }

    pub fn rest_bounds_under(&self, i: usize) -> Option<(Vec3, f32)> {
        let mut pose = Vec::new();
        self.pose(None, 0.0, &mut pose);
        let pts: Vec<Vec3> = self
            .nodes
            .iter()
            .enumerate()
            .filter(|(n, node)| node.mesh.is_some() && self.is_under(*n, i))
            .flat_map(|(n, node)| {
                let m = pose[n];
                self.meshes[node.mesh.unwrap()]
                    .pos
                    .iter()
                    .map(move |p| m.transform_point3(*p))
            })
            .collect();
        if pts.is_empty() {
            return None;
        }
        let lo = pts.iter().fold(Vec3::splat(f32::MAX), |a, p| a.min(*p));
        let hi = pts.iter().fold(Vec3::splat(f32::MIN), |a, p| a.max(*p));
        let c = (lo + hi) * 0.5;
        let r = pts.iter().map(|p| p.distance(c)).fold(0.0, f32::max);
        Some((c, r))
    }

    /// 쉬는 자세에서 노드 `i` 아래 메시 정점의 가운데(모델 공간) — 2차 동작이 "늦게 따라오는 덩어리" 자리로 쓴다.
    pub fn rest_centroid_under(&self, i: usize) -> Option<Vec3> {
        let mut pose = Vec::new();
        self.pose(None, 0.0, &mut pose);
        let under = |mut n: usize| loop {
            if n == i {
                return true;
            }
            match self.nodes[n].parent {
                Some(p) => n = p,
                None => return false,
            }
        };
        let (mut sum, mut cnt) = (Vec3::ZERO, 0usize);
        for (n, node) in self.nodes.iter().enumerate() {
            if let (Some(m), true) = (node.mesh, under(n)) {
                for p in &self.meshes[m].pos {
                    sum += pose[n].transform_point3(*p);
                    cnt += 1;
                }
            }
        }
        (cnt > 0).then(|| sum / cnt as f32)
    }
}

fn sample(c: &Channel, t: f32) -> Vec4 {
    let n = c.times.len();
    if n == 0 || c.values.is_empty() {
        return Vec4::ZERO;
    }
    if t <= c.times[0] || n == 1 {
        return c.values[0];
    }
    if t >= c.times[n - 1] {
        return c.values[n - 1];
    }
    let i = c.times.partition_point(|&x| x <= t) - 1;
    if c.step {
        return c.values[i];
    }
    let (t0, t1) = (c.times[i], c.times[i + 1]);
    let k = ((t - t0) / (t1 - t0).max(1e-6)).clamp(0.0, 1.0);
    let (a, b) = (c.values[i], c.values[i + 1]);
    if c.prop == Prop::R {
        let qa = Quat::from_vec4(a).normalize();
        let qb = Quat::from_vec4(b).normalize();
        Vec4::from(qa.slerp(qb, k))
    } else {
        a.lerp(b, k)
    }
}

/// 두 자세를 섞는다(0175): `w` = 0이면 `a`, 1이면 `b`. 자리·크기는 직선, 돌림은 구면.
pub fn blend_locals(
    a: &[(Vec3, Quat, Vec3)],
    b: &[(Vec3, Quat, Vec3)],
    w: f32,
) -> Vec<(Vec3, Quat, Vec3)> {
    let w = w.clamp(0.0, 1.0);
    a.iter()
        .zip(b)
        .map(|(x, y)| (x.0.lerp(y.0, w), x.1.slerp(y.1, w), x.2.lerp(y.2, w)))
        .collect()
}

/// 시험용 모델(코드로 찍은 Blockbench 꼴 — `tools/art/make_test_model.py`). 바이너리에 넣어 웹에서 파일을 따로 받지 않는다.
#[cfg(test)]
pub const TEST_CRITTER: &str = include_str!("../assets/models/test_critter.gltf");

#[cfg(test)]
mod tests {
    use super::*;

    fn critter() -> Model {
        Model::from_bytes(TEST_CRITTER.as_bytes()).unwrap()
    }

    #[test]
    fn the_test_critter_loads_with_its_part_tree_meshes_texture_and_animations() {
        let m = critter();
        assert_eq!(m.nodes.len(), 7);
        assert_eq!(m.meshes.len(), 6);
        assert!(m
            .texture_png
            .as_ref()
            .is_some_and(|p| p.starts_with(b"\x89PNG")));
        assert!(m.anim("walk").is_some() && m.anim("idle").is_some());
        let head = m.nodes.iter().position(|n| n.name == "head").unwrap();
        let body = m.nodes.iter().position(|n| n.name == "body").unwrap();
        assert_eq!(m.nodes[head].parent, Some(body));
        // 부모가 먼저.
        let at = |i| m.order.iter().position(|&x| x == i).unwrap();
        assert!(at(body) < at(head));
        let verts: usize = m.meshes.iter().map(|x| x.pos.len()).sum();
        assert!(verts > 100);
    }

    #[test]
    fn a_rigid_part_follows_its_parent_and_its_rotation_channel() {
        let m = critter();
        let mut pose = Vec::new();
        m.pose(None, 0.0, &mut pose);
        let head = m.nodes.iter().position(|n| n.name == "head").unwrap();
        // 머리 피벗 = 뿌리 + 몸통(0,0.7,0) + 머리(0,0.45,0).
        let p = pose[head].transform_point3(Vec3::ZERO);
        assert!((p - vec3(0.0, 1.15, 0.0)).length() < 1e-5, "{p:?}");
        // walk 0.2초: 왼다리가 +30°(X축) — 발끝(0,-0.7,0)이 앞(+z)으로 간다.
        let walk = m.anim("walk");
        m.pose(walk, 0.2, &mut pose);
        let leg = m.nodes.iter().position(|n| n.name == "leg_l").unwrap();
        let foot = pose[leg].transform_point3(vec3(0.0, -0.7, 0.0));
        let want = 0.7 * 30f32.to_radians().sin();
        assert!((foot.z.abs() - want).abs() < 1e-3, "{foot:?}");
        // 고리: 0.8초 뒤 같은 자세.
        let mut later = Vec::new();
        m.pose(walk, 1.0, &mut later);
        assert!(pose[leg].abs_diff_eq(later[leg], 1e-5));
    }

    // 0193: 방랑자는 스킨 몸(뼈 가중치로 CPU에서 휜다) — 쉬는 자세에서 바인드 자리 그대로, 동작이 몸을 움직이고 무기는 손을 따른다.
    fn wanderer() -> Model {
        Model::from_bytes(include_bytes!("../assets/models/wanderer.glb")).unwrap()
    }

    fn skinned_body(m: &Model, pose: &[Mat4]) -> Vec<Vec3> {
        let i = m
            .nodes
            .iter()
            .position(|n| n.skin.is_some())
            .expect("skinned body");
        let (mut p, mut n) = (Vec::new(), Vec::new());
        assert!(m.skin_into(i, pose, &mut p, &mut n));
        assert_eq!(p.len(), n.len());
        p
    }

    #[test]
    fn the_wanderer_body_is_skinned_and_stands_where_the_old_one_stood() {
        let m = wanderer();
        assert_eq!(m.skins.len(), 1);
        let mut rest = Vec::new();
        m.pose(None, 0.0, &mut rest);
        let p = skinned_body(&m, &rest);
        let lo = p.iter().fold(Vec3::splat(f32::MAX), |a, v| a.min(*v));
        let hi = p.iter().fold(Vec3::splat(f32::MIN), |a, v| a.max(*v));
        assert!((hi.y - lo.y - 1.4).abs() < 0.05, "height {}", hi.y - lo.y);
        assert!(lo.y.abs() < 0.03, "feet on the ground: {}", lo.y);
        let mid = (lo + hi) * 0.5;
        assert!(mid.x.abs() < 0.25 && mid.z.abs() < 0.25, "centred: {mid}");
    }

    #[test]
    fn a_swing_moves_the_skinned_body_and_the_blade_turns_in_the_fist() {
        let m = wanderer();
        let a = m.anim("attack").unwrap();
        let (mut p0, mut p1) = (Vec::new(), Vec::new());
        m.pose(Some(a), 0.0, &mut p0);
        m.pose(Some(a), 0.125, &mut p1);
        let moved = skinned_body(&m, &p0)
            .iter()
            .zip(skinned_body(&m, &p1))
            .map(|(a, b)| a.distance(b))
            .fold(0.0, f32::max);
        assert!(moved > 0.1, "the body bends: {moved}");
        // 무기는 주먹 자리의 `weapon` 뼈에 달려 있고(0193), 그 뼈는 돌기만 한다 — 주먹을 떠나지 않는다.
        let hand = m.node("mixamorig:RightHand").unwrap();
        let grip = m.node("weapon").unwrap();
        let sword = m.node("sword").unwrap();
        assert!(m.is_under(sword, grip) && m.is_under(grip, hand));
        let gap = |pose: &[Mat4]| pose[grip].w_axis.distance(pose[hand].w_axis);
        assert!(
            (gap(&p0) - gap(&p1)).abs() < 1e-3,
            "the grip stays in the fist"
        );
        let turn = |pose: &[Mat4]| pose[grip].to_scale_rotation_translation().1;
        assert!(
            turn(&p0).angle_between(turn(&p1)) > 0.2,
            "the blade turns with the swing"
        );
    }
}
