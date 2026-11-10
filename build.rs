//! `assets/models/*.glb`를 바이너리에 넣는 표를 만든다(`src/models.rs`가 `include!`). 파일을 넣으면 그대로 등록된다 —
//! 웹·Android에서 파일을 따로 받지 않는다(첫 게임처럼 한 덩어리). 이름 = 파일 이름(확장자 뺀 것).
use std::{env, fs, path::Path};

fn main() {
    build_hash();
    pad_keys_cfg();
    let dir = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("assets/models");
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut names: Vec<(String, String)> = fs::read_dir(&dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "glb"))
                .map(|p| {
                    println!("cargo:rerun-if-changed={}", p.display());
                    (
                        p.file_stem().unwrap().to_string_lossy().into_owned(),
                        p.display().to_string(),
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    let mut out = String::from("pub static FILES: &[(&str, &[u8])] = &[\n");
    for (name, path) in &names {
        out.push_str(&format!("    ({name:?}, include_bytes!({path:?})),\n"));
    }
    out.push_str("];\n");
    let dst = Path::new(&env::var("OUT_DIR").unwrap()).join("models_table.rs");
    fs::write(dst, out).unwrap();
}

/// 빌드한 커밋(0079, 사장 2026-11-07 "Dev 빌드는 화면에 버전 옆에 커밋 해시"): CI는 `GITHUB_SHA`, 손 빌드는 `git rev-parse`,
/// 둘 다 없으면 빈 글자(타이틀이 해시를 빼고 그린다). 앞 7자만 — 개발 웹의 `build.txt`(전체 해시)와 앞이 같다.
fn build_hash() {
    println!("cargo:rerun-if-env-changed=GITHUB_SHA");
    let root = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).to_path_buf();
    let git = root.join(".git");
    if git.exists() {
        println!("cargo:rerun-if-changed={}", git.join("HEAD").display());
        println!("cargo:rerun-if-changed={}", git.join("refs").display());
    }
    let full = env::var("GITHUB_SHA")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| {
            std::process::Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(&root)
                .output()
                .ok()
                .filter(|o| o.status.success())
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        });
    let short: String = full.unwrap_or_default().chars().take(7).collect();
    println!("cargo:rustc-env=BUILD_HASH={short}");
}

/// `cfg(pad_keys)`(0137): 패드를 `gamepads` 크레이트가 아니라 패치한 miniquad가 키(F13~F24)·`joystick_axes`로 넘기는 플랫폼 —
/// Android(0022, KEYCODE_BUTTON_*)와 iOS(0137, GameController 폴링). `input.rs`가 이 이름 하나로 두 길을 가른다.
fn pad_keys_cfg() {
    println!("cargo::rustc-check-cfg=cfg(pad_keys)");
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if os == "android" || os == "ios" {
        println!("cargo:rustc-cfg=pad_keys");
    }
}
