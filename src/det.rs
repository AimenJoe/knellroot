//! 플랫폼과 무관한 수학(0279): `f32::sin` · `atan2` 같은 초월 함수는 네이티브에서 시스템의 libm을 부른다 — 결과의 끝자리가
//! glibc 판마다 다르다(2.39와 2.43에서 `atan2f` · `asinf` · `acosf`가 갈려, 같은 누름으로 돌린 15초짜리 판의 해시가 CI에서만
//! 달랐다). 웹(wasm)의 std는 처음부터 Rust로 옮긴 libm을 쓴다. 그래서 게임의 초월 함수는 모두 `libm` 크레이트를 지난다:
//! 어느 기계 · 어느 타깃에서나 같은 입력에 같은 비트 — 골든 트레이스가 CI에서도 뜻을 갖고, 네이티브 시험이 웹 빌드와 같은 셈을 한다.
//!
//! 쓰는 법: `use crate::det::Det;` 뒤에 `x.sin_d()` · `y.atan2_d(x)`. std의 것(`sin` · `cos` · `sin_cos` · `tan` · `atan2` ·
//! `atan` · `asin` · `acos` · `powf` · `exp` · `hypot` …)은 `clippy.toml`이 막는다. `sqrt` · `abs` · `floor` · `mul_add` ·
//! `to_radians` · `powi`는 IEEE가 결과를 정하거나 곱셈뿐이라 그대로 쓴다.
//!
//! glam의 각도 함수(`Quat::from_rotation_y` · `Vec2::from_angle` · `angle_between` …)는 안에서 std를 부른다 — 로직(월드 · 적 ·
//! 스테이지 · 이펙트)에서는 쓰지 않고 아래 `turn_y`를 쓴다(시험 `logic_keeps_off_glams_trig`). 그림(렌더 · 카메라 · 모델)은
//! 트레이스 밖이라 glam 그대로.
use macroquad::prelude::{Quat, Vec3};

pub trait Det: Sized {
    fn sin_d(self) -> Self;
    fn cos_d(self) -> Self;
    fn tan_d(self) -> Self;
    fn asin_d(self) -> Self;
    fn acos_d(self) -> Self;
    fn atan_d(self) -> Self;
    fn atan2_d(self, x: Self) -> Self;
    fn exp_d(self) -> Self;
    fn powf_d(self, n: Self) -> Self;
    fn hypot_d(self, other: Self) -> Self;
    fn sin_cos_d(self) -> (Self, Self);
}

impl Det for f32 {
    fn sin_d(self) -> f32 {
        libm::sinf(self)
    }
    fn cos_d(self) -> f32 {
        libm::cosf(self)
    }
    fn tan_d(self) -> f32 {
        libm::tanf(self)
    }
    fn asin_d(self) -> f32 {
        libm::asinf(self)
    }
    fn acos_d(self) -> f32 {
        libm::acosf(self)
    }
    fn atan_d(self) -> f32 {
        libm::atanf(self)
    }
    fn atan2_d(self, x: f32) -> f32 {
        libm::atan2f(self, x)
    }
    fn exp_d(self) -> f32 {
        libm::expf(self)
    }
    fn powf_d(self, n: f32) -> f32 {
        libm::powf(self, n)
    }
    fn hypot_d(self, other: f32) -> f32 {
        libm::hypotf(self, other)
    }
    fn sin_cos_d(self) -> (f32, f32) {
        (libm::sinf(self), libm::cosf(self))
    }
}

impl Det for f64 {
    fn sin_d(self) -> f64 {
        libm::sin(self)
    }
    fn cos_d(self) -> f64 {
        libm::cos(self)
    }
    fn tan_d(self) -> f64 {
        libm::tan(self)
    }
    fn asin_d(self) -> f64 {
        libm::asin(self)
    }
    fn acos_d(self) -> f64 {
        libm::acos(self)
    }
    fn atan_d(self) -> f64 {
        libm::atan(self)
    }
    fn atan2_d(self, x: f64) -> f64 {
        libm::atan2(self, x)
    }
    fn exp_d(self) -> f64 {
        libm::exp(self)
    }
    fn powf_d(self, n: f64) -> f64 {
        libm::pow(self, n)
    }
    fn hypot_d(self, other: f64) -> f64 {
        libm::hypot(self, other)
    }
    fn sin_cos_d(self) -> (f64, f64) {
        (libm::sin(self), libm::cos(self))
    }
}

/// `v`를 +y 둘레로 `rad`만큼 돌린다 — glam `Quat::from_rotation_y(rad) * v`와 같은 셈, 삼각함수만 `libm`으로.
pub fn turn_y(v: Vec3, rad: f32) -> Vec3 {
    let (s, c) = (rad * 0.5).sin_cos_d();
    Quat::from_xyzw(0.0, s, 0.0, c) * v
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 값은 std와 끝자리까지만 다르다(같은 함수다) — 그리고 돌리기는 glam의 것과 같은 쪽 · 같은 크기.
    #[test]
    #[allow(clippy::disallowed_methods)]
    fn det_math_agrees_with_std_to_the_last_places_and_turns_like_glam() {
        for i in -400..400 {
            let x = i as f32 * 0.037;
            assert!((x.sin_d() - x.sin()).abs() < 1e-6, "sin {x}");
            assert!((x.cos_d() - x.cos()).abs() < 1e-6, "cos {x}");
            assert!((x.atan_d() - x.atan()).abs() < 1e-6, "atan {x}");
            assert!((x.atan2_d(0.7) - x.atan2(0.7)).abs() < 1e-6, "atan2 {x}");
            let u = (i as f32 / 400.0).clamp(-1.0, 1.0);
            assert!((u.asin_d() - u.asin()).abs() < 1e-5, "asin {u}");
            assert!((u.acos_d() - u.acos()).abs() < 1e-5, "acos {u}");
            let e = x * 0.3;
            assert!((e.exp_d() - e.exp()).abs() <= e.exp() * 1e-6, "exp {e}");
            let v = Vec3::new(1.0, 0.5, -2.0);
            let a = turn_y(v, x);
            let b = Quat::from_rotation_y(x) * v;
            assert!((a - b).length() < 1e-5, "turn {x}");
        }
        assert!((2.0f32.powf_d(0.5) - 2.0f32.sqrt()).abs() < 1e-6);
        assert!((3.0f32.hypot_d(4.0) - 5.0).abs() < 1e-6);
    }

    /// 로직의 폴더는 glam의 각도 함수를 부르지 않는다(안에서 std의 삼각함수를 쓴다 — 플랫폼마다 끝자리가 다르다).
    #[test]
    fn logic_keeps_off_glams_trig() {
        const BANNED: [&str; 9] = [
            "from_rotation_",
            "from_axis_angle",
            "from_euler",
            "to_euler",
            "from_angle",
            "to_angle",
            "angle_between",
            "angle_to",
            "slerp",
        ];
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut stack: Vec<std::path::PathBuf> = [
            "world",
            "world.rs",
            "enemy",
            "enemy.rs",
            "stage",
            "fx",
            "fx.rs",
            "skills",
            "status.rs",
            "weapons.rs",
            "ground.rs",
            "nav.rs",
        ]
        .iter()
        .map(|p| root.join(p))
        .collect();
        let mut bad = Vec::new();
        while let Some(p) = stack.pop() {
            if p.is_dir() {
                for e in std::fs::read_dir(&p).unwrap() {
                    stack.push(e.unwrap().path());
                }
                continue;
            }
            let is_test = p.components().any(|c| c.as_os_str() == "tests")
                || p.file_name()
                    .is_some_and(|n| n == "tests.rs" || n == "traces.rs");
            if is_test || p.extension().is_none_or(|e| e != "rs") {
                continue;
            }
            let text = std::fs::read_to_string(&p).unwrap();
            for (n, line) in text.lines().enumerate() {
                if line.trim_start().starts_with("//") {
                    continue;
                }
                if let Some(b) = BANNED.iter().find(|b| line.contains(**b)) {
                    bad.push(format!("{}:{} {b}", p.display(), n + 1));
                }
            }
        }
        assert!(bad.is_empty(), "glam trig in logic:\n{}", bad.join("\n"));
    }
}
