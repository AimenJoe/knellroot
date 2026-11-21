//! 시야 밖 거르기(0258): 카메라의 보기 사다리꼴(프러스텀) 여섯 면으로 상자·구가 화면에 들 수 있는지 본다(순수 — 시험).
//! 거르기 전에는 지도 전체의 지형·적을 프레임마다 다 그렸다(정원 ≈ 35만 정점 × 색·법선 두 패스, 적 40마리의 자세·복사).
//! 틀리게 거르면 화면 가장자리에서 물건이 사라지므로 늘 넉넉하게: 들 수도 있으면 그린다(면에서 바깥으로 완전히 넘어간 것만 뺀다).

use macroquad::prelude::*;

/// 여섯 면(왼·오른·아래·위·가까운·먼) — `n·p + d >= 0`이 안쪽.
#[derive(Clone, Copy, Debug)]
pub struct Frustum {
    planes: [Vec4; 6],
}

impl Frustum {
    /// 보기·투영을 곱한 행렬(클립 공간으로 보내는 것)에서 면을 뽑는다(Gribb–Hartmann). OpenGL 꼴(−w ≤ z ≤ w).
    pub fn from_matrix(m: Mat4) -> Frustum {
        let r = [m.row(0), m.row(1), m.row(2), m.row(3)];
        let raw = [
            r[3] + r[0],
            r[3] - r[0],
            r[3] + r[1],
            r[3] - r[1],
            r[3] + r[2],
            r[3] - r[2],
        ];
        Frustum {
            planes: raw.map(|p| {
                let l = p.truncate().length();
                if l > 0.0 {
                    p / l
                } else {
                    p
                }
            }),
        }
    }

    /// 아무것도 거르지 않는 것(카메라가 없을 때·시험).
    pub fn all() -> Frustum {
        Frustum {
            planes: [Vec4::new(0.0, 0.0, 0.0, 1.0); 6],
        }
    }

    /// 구가 보일 수 있나.
    pub fn sphere(&self, c: Vec3, r: f32) -> bool {
        self.planes.iter().all(|p| p.truncate().dot(c) + p.w >= -r)
    }

    /// 축 정렬 상자가 보일 수 있나: 면마다 그 면 쪽으로 가장 나간 꼭짓점이 바깥이면 뺀다.
    pub fn aabb(&self, min: Vec3, max: Vec3) -> bool {
        self.planes.iter().all(|p| {
            let n = p.truncate();
            let far = vec3(
                if n.x >= 0.0 { max.x } else { min.x },
                if n.y >= 0.0 { max.y } else { min.y },
                if n.z >= 0.0 { max.z } else { min.z },
            );
            n.dot(far) + p.w >= 0.0
        })
    }
}

/// 정점들의 경계 상자(빈 목록이면 원점 하나).
pub fn bounds(points: impl Iterator<Item = Vec3>) -> (Vec3, Vec3) {
    let mut lo = Vec3::splat(f32::MAX);
    let mut hi = Vec3::splat(f32::MIN);
    for p in points {
        lo = lo.min(p);
        hi = hi.max(p);
    }
    if lo.x > hi.x {
        (Vec3::ZERO, Vec3::ZERO)
    } else {
        (lo, hi)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cam() -> Frustum {
        // 원점 뒤 10m·위 6m에서 −z를 내려다보는 60° 카메라(쿼터뷰 비슷하게).
        let view = Mat4::look_at_rh(vec3(0.0, 6.0, 10.0), Vec3::ZERO, Vec3::Y);
        let proj = Mat4::perspective_rh_gl(60f32.to_radians(), 16.0 / 9.0, 0.1, 200.0);
        Frustum::from_matrix(proj * view)
    }

    #[test]
    fn what_the_camera_looks_at_is_kept_and_what_is_behind_or_far_aside_is_dropped() {
        let f = cam();
        assert!(f.sphere(Vec3::ZERO, 0.5));
        assert!(f.aabb(vec3(-1.0, 0.0, -1.0), vec3(1.0, 2.0, 1.0)));
        // 카메라 뒤.
        assert!(!f.sphere(vec3(0.0, 6.0, 30.0), 1.0));
        assert!(!f.aabb(vec3(-1.0, 0.0, 25.0), vec3(1.0, 2.0, 27.0)));
        // 옆으로 멀리.
        assert!(!f.sphere(vec3(80.0, 0.0, 0.0), 1.0));
        // 먼 면 너머.
        assert!(!f.sphere(vec3(0.0, 0.0, -250.0), 1.0));
    }

    #[test]
    fn something_that_only_pokes_into_the_view_is_kept() {
        let f = cam();
        // 가운데는 밖이지만 반지름이 화면 안으로 들어오는 구 · 한쪽 끝만 들어오는 긴 상자.
        let edge = (0..200)
            .map(|i| i as f32 * 0.25)
            .find(|x| !f.sphere(vec3(*x, 0.0, 0.0), 0.0))
            .unwrap();
        assert!(f.sphere(vec3(edge + 1.0, 0.0, 0.0), 2.0));
        assert!(f.aabb(vec3(-2.0, 0.0, -1.0), vec3(500.0, 1.0, 1.0)));
    }

    #[test]
    fn the_bounds_of_points_hold_every_point() {
        let pts = [
            vec3(1.0, -2.0, 3.0),
            vec3(-4.0, 5.0, 0.0),
            vec3(0.0, 0.0, 9.0),
        ];
        let (lo, hi) = bounds(pts.iter().copied());
        assert_eq!(lo, vec3(-4.0, -2.0, 0.0));
        assert_eq!(hi, vec3(1.0, 5.0, 9.0));
        assert!(Frustum::all().aabb(lo, hi));
    }
}
