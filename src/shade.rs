//! 달빛 그림자(0044): 굽기 때 "이 점에서 달 쪽으로 쏜 광선이 무엇에 막히나"를 높이 지도로 본다. 렌더가 지형을 굽는 동안
//! `lit`이 이것을 써서 달빛 몫만 가린다(바닥은 빛 지도 텍스처, 벽·옹벽·소품은 정점). 움직이지 않는 것만 — 캐릭터는 0045.
//!
//! 높이 지도: 0.25m 칸마다 그 자리에서 가장 높은 단단한 것의 꼭대기(벽·땅·바위 단·소품 상자). 광선이 그 칸에서 꼭대기보다 낮으면 막힘.
use macroquad::prelude::*;

#[derive(Clone, Debug)]
pub struct Occluders {
    /// 칸 (0,0)의 최소 모서리(월드 x, z).
    pub origin: Vec2,
    pub res: f32,
    pub w: usize,
    pub h: usize,
    pub top: Vec<f32>,
    max_top: f32,
}

impl Occluders {
    /// `w`×`h` 칸(`res` m), 꼭대기는 처음엔 아주 낮게(허공).
    pub fn new(origin: Vec2, res: f32, w: usize, h: usize) -> Occluders {
        Occluders {
            origin,
            res,
            w,
            h,
            top: vec![f32::MIN; w * h],
            max_top: f32::MIN,
        }
    }

    pub fn center(&self, i: usize, j: usize) -> Vec2 {
        self.origin + vec2(i as f32 + 0.5, j as f32 + 0.5) * self.res
    }

    /// 칸의 꼭대기를 `y`로 올린다(이미 더 높으면 그대로).
    pub fn raise(&mut self, i: usize, j: usize, y: f32) {
        let t = &mut self.top[j * self.w + i];
        *t = t.max(y);
        self.max_top = self.max_top.max(y);
    }

    /// 상자(XZ 최소·최대, 꼭대기 `y`)가 덮는 칸을 올린다.
    pub fn raise_box(&mut self, min: Vec2, max: Vec2, y: f32) {
        let lo = ((min - self.origin) / self.res).floor();
        let hi = ((max - self.origin) / self.res).ceil();
        for j in lo.y.max(0.0) as usize..(hi.y.max(0.0) as usize).min(self.h) {
            for i in lo.x.max(0.0) as usize..(hi.x.max(0.0) as usize).min(self.w) {
                let c = self.center(i, j);
                if c.x >= min.x && c.x <= max.x && c.y >= min.y && c.y <= max.y {
                    self.raise(i, j, y);
                }
            }
        }
    }

    fn top_at(&self, x: f32, z: f32) -> f32 {
        let i = ((x - self.origin.x) / self.res).floor();
        let j = ((z - self.origin.y) / self.res).floor();
        if i < 0.0 || j < 0.0 || i as usize >= self.w || j as usize >= self.h {
            return f32::MIN;
        }
        self.top[j as usize * self.w + i as usize]
    }

    /// 점 `p`에서 `to_light`(단위, 빛 쪽) 방향으로 막히지 않으면 1, 막히면 0. 출발 칸은 건너뛴다(제 면에 막히지 않게).
    pub fn vis(&self, p: Vec3, to_light: Vec3) -> f32 {
        let step = self.res * 0.5;
        let mut q = p + to_light * self.res * 0.75;
        for _ in 0..400 {
            if q.y > self.max_top {
                return 1.0;
            }
            if q.y < self.top_at(q.x, q.z) - 0.02 {
                return 0.0;
            }
            q += to_light * step;
        }
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 10m 마당(0.25m 칸) 가운데에 높이 3m 벽(x 0..1). 빛은 +x 쪽 위에서 45°.
    #[test]
    fn a_wall_shades_the_floor_on_the_far_side_from_the_light() {
        let mut o = Occluders::new(vec2(-5.0, -5.0), 0.25, 40, 40);
        for j in 0..40 {
            for i in 0..40 {
                o.raise(i, j, 0.0);
            }
        }
        o.raise_box(vec2(0.0, -5.0), vec2(1.0, 5.0), 3.0);
        let l = vec3(1.0, 1.0, 0.0).normalize();
        // 벽 왼쪽(빛 반대쪽) 1m: 그늘(벽 높이 3m, 45° → 3m 그림자).
        assert_eq!(o.vis(vec3(-1.0, 0.0, 0.0), l), 0.0);
        assert_eq!(o.vis(vec3(-2.5, 0.0, 0.0), l), 0.0);
        // 그림자 끝 너머·벽 오른쪽(빛 쪽): 밝다.
        assert_eq!(o.vis(vec3(-3.5, 0.0, 0.0), l), 1.0);
        assert_eq!(o.vis(vec3(2.0, 0.0, 0.0), l), 1.0);
        // 벽 꼭대기: 밝다.
        assert_eq!(o.vis(vec3(0.5, 3.0, 0.0), l), 1.0);
    }
}
