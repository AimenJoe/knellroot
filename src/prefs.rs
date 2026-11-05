//! 설정 저장(0024, 첫 게임 nail `prefs.rs`의 틀): 언어·키 배치. 웹은 브라우저 localStorage의 정수 칸
//! (web/index.html `knell_device` 플러그인의 `knell_pref_get`·`knell_pref_set`, 키 "knell.pref.<칸>"),
//! 데스크톱·Android는 기록 폴더의 `prefs.txt` 한 줄(칸 값을 빈칸으로).
//!
//! 칸: 0 표식(MAGIC), 1 언어(0 = 저장 전), 2..18 키 배치(`Bindings::encode` — 14..16은 0027에 붙인 점프, 16..18은 0031 등잔, 18..20은 0032 상호작용 → 0048 마법). 0은 "없음". 표식이 다르면 통째로 기본값.
//! 칸을 뒤에 붙이면 옛 저장은 0 = 기본값으로 읽으니 MAGIC은 그대로, 칸 뜻이 바뀌면 올린다.
use crate::i18n::Lang;
use crate::input::{Bindings, MouseAim, REST_SECS, RETICLE_SIZES};

/// 0x…0002(0048): 18..20 칸이 상호작용 → 마법으로 바뀌고 등잔·마법의 기본 버튼이 바뀌었다. 옛 표식(0x…0001)은 언어만 살리고 배치는 기본값.
const MAGIC: u32 = 0x4B4E_0002;
const MAGIC_V1: u32 = 0x4B4E_0001;
/// 20 화면 흔들림(0..=10, +1로 — 0 = 저장 전 = 10), 21 든 무기(무기 표 자리 +1) — 0052·0051, 뒤에 붙인 칸이라 MAGIC 그대로.
/// 22 마우스 조준(1 끔·2 켬, 0 = 저장 전 = 켬), 23 커서 쉼(`REST_SECS` 자리 +1), 24 조준 표시 크기(+1) — 0058, 뒤에 붙인 칸이라 MAGIC 그대로.
const CELLS: usize = 25;
/// 화면 흔들림 기본값(10 = 100%).
pub const SHAKE_DEFAULT: u8 = 10;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Prefs {
    pub lang: Option<Lang>,
    pub bindings: Bindings,
    /// 화면 흔들림 0..=10(설정, 0052).
    pub shake: u8,
    /// 든 무기(무기 표의 자리, 0051).
    pub weapon: u8,
    /// 마우스 조준(0058).
    pub mouse: MouseAim,
}

fn lang_code(l: Lang) -> u32 {
    match l {
        Lang::Ko => 1,
        Lang::En => 2,
    }
}

fn lang_from(c: u32) -> Option<Lang> {
    match c {
        1 => Some(Lang::Ko),
        2 => Some(Lang::En),
        _ => None,
    }
}

fn encode(p: &Prefs) -> [u32; CELLS] {
    let mut v = [0u32; CELLS];
    v[0] = MAGIC;
    v[1] = p.lang.map_or(0, lang_code);
    v[2..20].copy_from_slice(&p.bindings.encode());
    v[20] = p.shake.min(10) as u32 + 1;
    v[21] = p.weapon as u32 + 1;
    v[22] = p.mouse.on as u32 + 1;
    v[23] = p.mouse.rest as u32 + 1;
    v[24] = p.mouse.size as u32 + 1;
    v
}

/// 0058 칸 셋: 0(저장 전·옛 저장)이나 범위 밖이면 기본값.
fn mouse_from(on: u32, rest: u32, size: u32) -> MouseAim {
    let d = MouseAim::default();
    let pick = |c: u32, n: usize, def: u8| match c {
        1.. if (c as usize) <= n => (c - 1) as u8,
        _ => def,
    };
    MouseAim {
        on: match on {
            1 => false,
            2 => true,
            _ => d.on,
        },
        rest: pick(rest, REST_SECS.len(), d.rest),
        size: pick(size, RETICLE_SIZES as usize, d.size),
    }
}

fn shake_from(c: u32) -> u8 {
    if c == 0 {
        SHAKE_DEFAULT
    } else {
        (c - 1).min(10) as u8
    }
}

fn decode(v: &[u32; CELLS]) -> Option<Prefs> {
    if v[0] == MAGIC_V1 {
        return Some(Prefs {
            lang: lang_from(v[1]),
            bindings: Bindings::default_layout(),
            shake: SHAKE_DEFAULT,
            weapon: 0,
            mouse: MouseAim::default(),
        });
    }
    if v[0] != MAGIC {
        return None;
    }
    Some(Prefs {
        lang: lang_from(v[1]),
        bindings: Bindings::decode(&v[2..20]),
        shake: shake_from(v[20]),
        weapon: v[21].saturating_sub(1).min(255) as u8,
        mouse: mouse_from(v[22], v[23], v[24]),
    })
}

#[cfg(target_arch = "wasm32")]
mod store {
    extern "C" {
        fn knell_pref_get(i: u32) -> u32;
        fn knell_pref_set(i: u32, v: u32);
    }

    pub fn load_all() -> [u32; super::CELLS] {
        let mut v = [0u32; super::CELLS];
        for (i, x) in v.iter_mut().enumerate() {
            // SAFETY: 정수 하나를 받아 정수를 돌려주는 JS 함수(web/index.html).
            *x = unsafe { knell_pref_get(i as u32) };
        }
        v
    }

    pub fn save_all(v: &[u32; super::CELLS]) {
        for (i, x) in v.iter().enumerate() {
            // SAFETY: 위와 같음.
            unsafe { knell_pref_set(i as u32, *x) };
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod store {
    use super::CELLS;
    use std::path::{Path, PathBuf};

    /// 앱 전용 폴더(nail `profile::data_dir`과 같은 자리 고르기). Android는 프로세스 이름(= 패키지 이름, 개발판은 `.dev`)에서.
    pub fn data_dir() -> Option<PathBuf> {
        if let Some(d) = std::env::var_os("KNELL_SAVE_DIR") {
            return Some(PathBuf::from(d));
        }
        if cfg!(target_os = "android") {
            let cmdline = std::fs::read_to_string("/proc/self/cmdline").unwrap_or_default();
            let pkg = cmdline.split(['\0', ':']).next().unwrap_or("").trim();
            if pkg.is_empty() {
                return None;
            }
            return Some(PathBuf::from(format!("/data/data/{pkg}/files")));
        }
        if cfg!(target_os = "windows") {
            return Some(PathBuf::from(std::env::var_os("APPDATA")?).join("knellroot"));
        }
        if cfg!(any(target_os = "macos", target_os = "ios")) {
            return Some(
                PathBuf::from(std::env::var_os("HOME")?)
                    .join("Library/Application Support/knellroot"),
            );
        }
        match std::env::var_os("XDG_DATA_HOME") {
            Some(d) => Some(PathBuf::from(d).join("knellroot")),
            None => Some(PathBuf::from(std::env::var_os("HOME")?).join(".local/share/knellroot")),
        }
    }

    fn path() -> Option<PathBuf> {
        data_dir().map(|d| d.join("prefs.txt"))
    }

    /// 파일 한 줄: 칸 값을 빈칸으로. 없거나 깨졌으면 0(= 저장 전).
    pub fn read_from(p: &Path) -> [u32; CELLS] {
        let mut v = [0u32; CELLS];
        if let Ok(s) = std::fs::read_to_string(p) {
            for (x, w) in v.iter_mut().zip(s.split_whitespace()) {
                *x = w.parse().unwrap_or(0);
            }
        }
        v
    }

    /// 임시 파일에 쓰고 이름을 바꾼다(쓰다 꺼져도 옛 저장이 남는다).
    pub fn write_to(p: &Path, v: &[u32; CELLS]) -> bool {
        let result = (|| {
            if let Some(d) = p.parent() {
                std::fs::create_dir_all(d)?;
            }
            let line: Vec<String> = v.iter().map(|x| x.to_string()).collect();
            let tmp = p.with_extension("tmp");
            std::fs::write(&tmp, line.join(" "))?;
            std::fs::rename(&tmp, p)
        })();
        result.is_ok()
    }

    pub fn load_all() -> [u32; CELLS] {
        path().map(|p| read_from(&p)).unwrap_or([0; CELLS])
    }

    pub fn save_all(v: &[u32; CELLS]) {
        if let Some(p) = path() {
            write_to(&p, v);
        }
    }
}

/// 기록 폴더(데스크톱·Android) — 기록(`save.rs`)도 같은 폴더를 쓴다.
#[cfg(not(target_arch = "wasm32"))]
pub fn data_dir() -> Option<std::path::PathBuf> {
    store::data_dir()
}

pub fn load() -> Option<Prefs> {
    decode(&store::load_all())
}

pub fn save(p: &Prefs) {
    store::save_all(&encode(p));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{Bind, Pad};
    use macroquad::prelude::MouseButton;

    #[test]
    fn prefs_round_trip_and_unknown_storage_reads_as_defaults() {
        let mut b = Bindings::default_layout();
        b.set_key(Bind::Attack, 5);
        b.set_pad(Bind::Parry, Pad::R1);
        let p = Prefs {
            lang: Some(Lang::En),
            bindings: b,
            shake: 4,
            weapon: 2,
            mouse: MouseAim {
                on: false,
                rest: 3,
                size: 0,
            },
        };
        assert_eq!(decode(&encode(&p)), Some(p));
        assert_eq!(decode(&[0; CELLS]), None, "nothing saved yet");
        let mut bad = encode(&p);
        bad[2] = 999; // 범위 밖 키 → 배치는 기본값
        assert_eq!(decode(&bad).unwrap().bindings, Bindings::default_layout());
        // 0027 전 저장(점프 칸이 0): 나머지는 그대로, 점프만 기본값.
        let mut old = encode(&p);
        old[14] = 0;
        old[15] = 0;
        let got = decode(&old).unwrap().bindings;
        assert_eq!(got.keys[..6], b.keys[..6]);
        assert_eq!(got.pads[Bind::Jump as usize], Pad::South);
        // 0031 전 저장(등잔 칸이 0): 등잔만 기본값(R·Y, 0048).
        let plain = Prefs {
            lang: Some(Lang::En),
            bindings: Bindings::default_layout(),
            shake: SHAKE_DEFAULT,
            weapon: 0,
            mouse: MouseAim::default(),
        };
        let mut old = encode(&plain);
        old[16..].fill(0);
        let got = decode(&old).unwrap().bindings;
        assert_eq!(got, Bindings::default_layout());
        // 사용자가 R1을 이미 다른 동작(패링)에 붙여 둔 옛 저장이면: 배치는 그대로, 등잔은 비어 있는 버튼.
        let mut old = encode(&p);
        old[16..].fill(0);
        let got = decode(&old).unwrap().bindings;
        assert_eq!(got.keys[..7], b.keys[..7]);
        assert_eq!(got.pads[..7], b.pads[..7]);
        assert_eq!(b.pads[Bind::Parry as usize], Pad::R1);
        assert_ne!(got.pads[Bind::Flask as usize], Pad::R1);
        // 0048 전 저장(표식 0x…0001 — 18..20 칸이 상호작용이었다): 언어는 살리고 배치는 기본값(등잔 Y·마법 RB).
        let mut v1 = encode(&p);
        v1[0] = MAGIC_V1;
        let got = decode(&v1).unwrap();
        assert_eq!(got.lang, Some(Lang::En));
        assert_eq!(got.bindings, Bindings::default_layout());
        assert_eq!(got.bindings.pad(Bind::Flask), Pad::North);
        assert_eq!(got.bindings.pad(Bind::Cast), Pad::R1);
        // 0052 전 저장(흔들림·무기 칸이 0): 흔들림 기본값 10, 무기는 첫째(검).
        let mut old = encode(&p);
        old[20] = 0;
        old[21] = 0;
        let got = decode(&old).unwrap();
        assert_eq!((got.shake, got.weapon), (SHAKE_DEFAULT, 0));
        assert_eq!(decode(&encode(&p)).unwrap().shake, 4);
    }

    /// 0058: 마우스 버튼 칸이 저장되고(키 칸 뒤 번호), 마우스 조준 칸 셋이 0058 전 저장(0)이나 범위 밖이면 기본값(켬·1초·보통).
    #[test]
    fn mouse_slots_and_mouse_aim_cells_round_trip() {
        use crate::input::{Slot, NSLOTS};
        let mut b = Bindings::keyboard_only();
        b.set_key(Bind::Parry, (NSLOTS - 1) as u8);
        assert_eq!(b.key(Bind::Parry), Slot::Mouse(MouseButton::Middle));
        let p = Prefs {
            lang: Some(Lang::Ko),
            bindings: b,
            shake: SHAKE_DEFAULT,
            weapon: 0,
            mouse: MouseAim {
                on: true,
                rest: 2,
                size: 2,
            },
        };
        assert_eq!(decode(&encode(&p)), Some(p));
        let mut old = encode(&p);
        old[22..].fill(0);
        assert_eq!(decode(&old).unwrap().mouse, MouseAim::default());
        old[23] = 99;
        old[24] = 99;
        assert_eq!(decode(&old).unwrap().mouse, MouseAim::default());
        let mut bad = encode(&p);
        bad[2] = NSLOTS as u32 + 1;
        assert_eq!(decode(&bad).unwrap().bindings, Bindings::default_layout());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn the_native_prefs_file_round_trips() {
        let dir = std::env::temp_dir().join(format!("knell-prefs-test-{}", std::process::id()));
        let p = dir.join("prefs.txt");
        assert_eq!(store::read_from(&p), [0u32; CELLS]);
        let prefs = Prefs {
            lang: Some(Lang::Ko),
            bindings: Bindings::default_layout(),
            shake: 7,
            weapon: 1,
            mouse: MouseAim::default(),
        };
        assert!(store::write_to(&p, &encode(&prefs)));
        assert_eq!(decode(&store::read_from(&p)), Some(prefs));
        let _ = std::fs::remove_dir_all(dir);
    }
}
