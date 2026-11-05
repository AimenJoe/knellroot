//! 아이템을 그림으로 보일 때(0068): 어느 모델의 어느 부위를, 어느 각도로. 표 한 줄 = 아이템 하나 — 새 아이템 = 한 줄 + 모델 부위.
//! 각도는 실루엣이 가장 잘 읽히는 쪽(KB `readable-low-poly-models` — 작은 크기에서는 실루엣·명도가 먼저). 렌더는 `Renderer::draw_item`.
use crate::charms::Id as Charm;
use crate::weapons::Id as Weapon;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Item {
    Weapon(Weapon),
    Charm(Charm),
}

/// 모델·부위와 보는 각도(도): 부위를 경계 상자 가운데에 놓고 y(돌기)·x(숙이기)·z(눕히기) 순으로 돌린다.
#[derive(Clone, Copy, Debug)]
pub struct View {
    pub model: &'static str,
    pub node: &'static str,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
}

pub fn view(item: Item) -> View {
    match item {
        // 무기는 방랑자 모델의 오른손 부위(0051). 모델 공간에서 날·자루는 +z로 길고 넓은 면이 위(+y)를 본다
        // (`wanderer.py`: 블렌더 -y로 긴 상자, 내보내면 glTF +z) — 앞으로 세워(숙이기 −80°: 넓은 면이 카메라 쪽) 대각선으로 눕힌다.
        // 쇠뇌는 T자(개머리 세로 · 활 가로)가 읽히게 눕히지 않는다.
        Item::Weapon(w) => {
            let (yaw, pitch, roll) = match w {
                Weapon::Sword | Weapon::Staff => (0.0, -80.0, -45.0),
                Weapon::Hammer => (0.0, -75.0, -40.0),
                Weapon::Crossbow => (0.0, -70.0, 0.0),
            };
            View {
                model: "wanderer",
                node: w.def().node,
                yaw,
                pitch,
                roll,
            }
        }
        // 장신구는 `charms.glb`의 부위 하나씩(이름 = 기록 키) — 위에서 비스듬히.
        Item::Charm(c) => View {
            model: "charms",
            node: crate::charms::def(c).key,
            yaw: 30.0,
            pitch: 25.0,
            roll: 0.0,
        },
    }
}

/// 아이콘 캐시 열쇠(부위 이름 — 모델끼리 겹치지 않는다, 시험).
pub fn key(item: Item) -> &'static str {
    view(item).node
}

/// 모든 아이템(시험).
#[cfg(test)]
pub fn all() -> Vec<Item> {
    crate::weapons::IDS
        .into_iter()
        .map(Item::Weapon)
        .chain(crate::charms::TABLE.iter().map(|d| Item::Charm(d.id)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_item_has_its_own_part_name() {
        let mut keys: Vec<&str> = all().into_iter().map(key).collect();
        let n = keys.len();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), n, "icon cache keys must not collide");
    }

    #[test]
    fn every_item_part_exists_in_its_model() {
        // 모델은 바이너리에 들어 있다(build.rs) — 부위 이름이 바뀌면 그림이 조용히 사라지니 시험으로.
        for item in all() {
            let v = view(item);
            let Some(bytes) = crate::models::bytes(v.model) else {
                panic!("model {} missing for {item:?}", v.model);
            };
            let m = crate::model::Model::from_bytes(bytes).expect("parses");
            let i = m
                .node(v.node)
                .unwrap_or_else(|| panic!("{item:?}: no part {} in {}", v.node, v.model));
            assert!(m.rest_bounds_under(i).is_some(), "{item:?} has no mesh");
        }
    }
}
