//! 모델 목록(0019): `assets/models/*.glb`가 빌드 때 그대로 들어온다(`build.rs`). 이름으로 찾고, 처음 쓸 때 읽어 텍스처를 올린다.
//! 없는 이름은 `None` — 렌더가 상자로 대신 그린다(모델이 아직 없는 종류도 게임은 돈다).
//!
//! 이름 약속(Blender 스크립트 `tools/art/blender/*.py`와 같다):
//! - 부위 노드 이름이 `glow_`로 시작하면 빛을 받지 않고 스스로 빛난다(눈·핵·불꽃).
//! - 동작 이름: 공통 `idle`·`walk`·`attack`·`hurt`, 종류마다 더(`charge_windup`·`charge`·`slam`·`wave`·`wake` …).
//!   렌더는 상태 → 동작 이름 후보를 차례로 찾고(`pick`), 없으면 다음 후보, 다 없으면 첫 자세.
use std::collections::HashMap;

use macroquad::prelude::*;

use crate::model::Model;

include!(concat!(env!("OUT_DIR"), "/models_table.rs"));

pub struct Entry {
    pub model: Model,
    pub tex: Texture2D,
    /// 노드마다 스스로 빛나나(`glow_`).
    pub glow: Vec<bool>,
}

impl Entry {
    /// 후보 이름 중 처음 있는 동작.
    pub fn pick(&self, names: &[&str]) -> Option<usize> {
        names.iter().find_map(|n| self.model.anim(n))
    }

    /// 동작 길이(초). 없으면 1.
    pub fn duration(&self, anim: Option<usize>) -> f32 {
        anim.and_then(|i| self.model.anims.get(i))
            .map(|a| a.duration.max(1e-3))
            .unwrap_or(1.0)
    }
}

#[derive(Default)]
pub struct Library {
    loaded: HashMap<&'static str, Option<Entry>>,
}

impl Library {
    pub fn get(&mut self, name: &'static str) -> Option<&Entry> {
        self.loaded
            .entry(name)
            .or_insert_with(|| {
                let bytes = FILES.iter().find(|(n, _)| *n == name)?.1;
                match Model::from_bytes(bytes) {
                    Ok(model) => {
                        let tex = match &model.texture_png {
                            Some(png) => {
                                Texture2D::from_file_with_format(png, Some(ImageFormat::Png))
                            }
                            None => Texture2D::from_rgba8(1, 1, &[255, 255, 255, 255]),
                        };
                        tex.set_filter(FilterMode::Nearest);
                        let glow = model
                            .nodes
                            .iter()
                            .map(|n| n.name.starts_with("glow_"))
                            .collect();
                        Some(Entry { model, tex, glow })
                    }
                    Err(e) => {
                        error!("model {name}: {e}");
                        None
                    }
                }
            })
            .as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 0036: 모델 한 벌이 그리기 한 번의 용량 안에 든다 — 넘치면 macroquad가 조용히 잘라 뒤쪽 부위가 사라진다
    /// (둥근 방랑자의 몸·다리가 그랬다, 인덱스 ≈ 7,800 > 기본 5,000).
    #[test]
    fn every_bundled_model_fits_one_draw_call() {
        for (name, bytes) in FILES {
            let m = Model::from_bytes(bytes).unwrap();
            let v: usize = m.meshes.iter().map(|x| x.pos.len()).sum();
            let i: usize = m.meshes.iter().map(|x| x.idx.len()).sum();
            assert!(v < crate::render::MAX_DRAW_VERTICES, "{name}: {v} vertices");
            assert!(i < crate::render::MAX_DRAW_INDICES, "{name}: {i} indices");
        }
    }

    /// 들어 있는 모든 모델이 읽히고, 동작 길이가 0보다 크다(Blender에서 내보낸 꼴이 로더와 맞는지).
    #[test]
    fn every_bundled_model_parses() {
        for (name, bytes) in FILES {
            let m = Model::from_bytes(bytes).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert!(!m.meshes.is_empty(), "{name}: no meshes");
            for a in &m.anims {
                assert!(a.duration > 0.0, "{name}/{}: zero length", a.name);
            }
        }
    }
}
