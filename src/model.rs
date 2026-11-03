//! glTF 읽기(P4 스파이크, 0013): 부위 강체 모델 — 노드 트리(TRS) + 노드마다 메시 + 노드 TRS 애니메이션 채널.
//! 스키닝 없음(KB reference/assets/retro-3d-models-blockbench-gltf.md). macroquad에는 모델 로더가 없어(0.4.16 소스로 확인)
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
}

#[derive(Clone, Debug, Default)]
pub struct MeshData {
    pub pos: Vec<Vec3>,
    pub nrm: Vec<Vec3>,
    pub uv: Vec<Vec2>,
    pub idx: Vec<u32>,
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
            order,
            texture_png,
        })
    }

    pub fn anim(&self, name: &str) -> Option<usize> {
        self.anims.iter().position(|a| a.name == name)
    }

    /// 애니메이션 `anim`의 `t`초(고리) 자세 → 노드마다 모델 공간 행렬.
    pub fn pose(&self, anim: Option<usize>, t: f32, out: &mut Vec<Mat4>) {
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
        out.clear();
        out.resize(self.nodes.len(), Mat4::IDENTITY);
        for &i in &self.order {
            let (t, r, s) = local[i];
            let m = Mat4::from_scale_rotation_translation(s, r, t);
            out[i] = match self.nodes[i].parent {
                Some(p) => out[p] * m,
                None => m,
            };
        }
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
}
