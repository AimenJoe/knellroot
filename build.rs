//! `assets/models/*.glb`를 바이너리에 넣는 표를 만든다(`src/models.rs`가 `include!`). 파일을 넣으면 그대로 등록된다 —
//! 웹·Android에서 파일을 따로 받지 않는다(첫 게임처럼 한 덩어리). 이름 = 파일 이름(확장자 뺀 것).
use std::{env, fs, path::Path};

fn main() {
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
